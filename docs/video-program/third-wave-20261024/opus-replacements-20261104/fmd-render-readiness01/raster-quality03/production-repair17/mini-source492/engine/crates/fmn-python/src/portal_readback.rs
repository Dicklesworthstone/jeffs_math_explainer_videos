//! Observational in-memory capture over Marionette and Lumen.
//!
//! The caller freezes the Python family graph before this boundary. Native
//! entries are copied by value into a private Stage: capture never adopts a
//! proxy, advances a clock, invokes an updater, or opens an output generation.

#[path = "portal_capture_output.rs"]
mod output;

use std::path::PathBuf;
use std::sync::OnceLock;

use fmn_core::color::Srgb;
use fmn_frame::{FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::{Mob, Mobject, Stage};
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap,
    Tiling, Viewport,
};
use fmn_studio::{TerminalPreview, TerminalProtocol, TuiLimits};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

use crate::{
    BridgeMobject, PyCameraCore, PyCameraFrameCore, PyRuntimeError, PyValueError, StaleHandleError,
    native_error, stage_error,
};

const MAX_NODES: usize = 16_384;
const MAX_EDGES: usize = 131_072;
const MAX_RECORD_BYTES: usize = 256 * 1024 * 1024;
const MAX_PIXELS: u64 = 16_777_216;

/// Immutable RGBA output; no live scene, proxy, callback or renderer is retained.
#[pyclass(frozen, name = "_CameraCapture")]
pub(crate) struct CameraCapture {
    rgba: FrameBuffer,
    width: u32,
    height: u32,
    png: OnceLock<Vec<u8>>,
}

fn graph(
    children: &Bound<'_, PyList>,
    roots: &Bound<'_, PyList>,
    count: usize,
) -> PyResult<(Vec<Vec<usize>>, Vec<usize>)> {
    if count > MAX_NODES || children.len() != count || roots.len() > MAX_NODES {
        return Err(PyValueError::new_err(
            "camera capture family exceeds its node budget or has invalid adjacency",
        ));
    }
    let mut adjacency = Vec::with_capacity(count);
    let mut incoming = vec![0usize; count];
    let mut edges = 0usize;
    for row in children.iter() {
        let row = row.cast::<PyList>()?;
        edges = edges
            .checked_add(row.len())
            .ok_or_else(|| PyValueError::new_err("camera capture edge count overflow"))?;
        if edges > MAX_EDGES {
            return Err(PyValueError::new_err(
                "camera capture family exceeds its edge budget",
            ));
        }
        let mut values = Vec::with_capacity(row.len());
        for value in row.iter() {
            let child: usize = value.extract()?;
            if child >= count {
                return Err(PyValueError::new_err(
                    "camera capture child index is out of range",
                ));
            }
            incoming[child] += 1;
            values.push(child);
        }
        adjacency.push(values);
    }
    // Kahn's traversal validates even unreachable nodes supplied to this
    // private-but-callable API. No recursion or native copy precedes it.
    let mut ready: Vec<usize> = incoming
        .iter()
        .enumerate()
        .filter_map(|(index, &degree)| (degree == 0).then_some(index))
        .collect();
    let mut cursor = 0;
    while cursor < ready.len() {
        let node = ready[cursor];
        cursor += 1;
        for &child in &adjacency[node] {
            incoming[child] -= 1;
            if incoming[child] == 0 {
                ready.push(child);
            }
        }
    }
    if ready.len() != count {
        return Err(PyValueError::new_err(
            "camera capture family contains a cycle",
        ));
    }
    let roots: Vec<usize> = roots.extract()?;
    if roots.iter().any(|&root| root >= count) {
        return Err(PyValueError::new_err(
            "camera capture root index is out of range",
        ));
    }
    Ok((adjacency, roots))
}

fn copy_entry(source: &Stage, mob: Mob, target: &mut Stage, bytes: &mut usize) -> PyResult<Mob> {
    let entry = source
        .get(mob)
        .ok_or_else(|| StaleHandleError::new_err("camera capture source no longer resolves"))?;
    *bytes = entry
        .buffer
        .len()
        .checked_mul(entry.buffer.schema().stride())
        .and_then(|lanes| lanes.checked_mul(4))
        .and_then(|size| bytes.checked_add(size))
        .filter(|&size| size <= MAX_RECORD_BYTES)
        .ok_or_else(|| PyValueError::new_err("camera capture exceeds its 256 MiB record budget"))?;
    source.copy_entry_into(mob, target).map_err(stage_error)
}

#[pymethods]
impl CameraCapture {
    #[new]
    #[pyo3(signature = (core, frame, background, light, objects, children, roots, threads=1))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        core: &Bound<'_, PyCameraCore>,
        frame: &Bound<'_, PyCameraFrameCore>,
        background: [f64; 4],
        light: [f64; 3],
        objects: &Bound<'_, PyList>,
        children: &Bound<'_, PyList>,
        roots: &Bound<'_, PyList>,
        threads: usize,
    ) -> PyResult<Self> {
        if !(1..=96).contains(&threads) {
            return Err(PyValueError::new_err(
                "camera capture threads must lie in 1..=96",
            ));
        }
        if !background
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(PyValueError::new_err(
                "camera background RGBA must be finite in 0..=1",
            ));
        }
        let mut camera = core.try_borrow()?.camera.clone();
        *camera.frame_mut() = frame.try_borrow()?.frame.clone();
        let (width, height) = camera.pixel_shape();
        if u64::from(width) * u64::from(height) > MAX_PIXELS {
            return Err(PyValueError::new_err(
                "camera capture exceeds the 16M-pixel budget",
            ));
        }
        let background = Srgb {
            r: background[0],
            g: background[1],
            b: background[2],
        }
        .to_linear(background[3]);
        camera.set_background(background).map_err(native_error)?;
        camera
            .set_light_source_position(light)
            .map_err(native_error)?;
        let (adjacency, roots) = graph(children, roots, objects.len())?;
        let mut stage = Stage::new();
        let mut handles = Vec::with_capacity(objects.len());
        let mut record_bytes = 0usize;
        for object in objects.iter() {
            let object = object.cast::<BridgeMobject>()?;
            let object = object.try_borrow()?;
            let handle = if let (Some(engine), Some(mob)) = (&object.engine, object.mob) {
                let scene = engine.scene.try_borrow().map_err(|_| {
                    PyRuntimeError::new_err("camera capture cannot read a mutably borrowed Scene")
                })?;
                copy_entry(scene.stage(), mob, &mut stage, &mut record_bytes)?
            } else if let Some(nursery) = &object.nursery {
                copy_entry(&nursery.stage, nursery.root, &mut stage, &mut record_bytes)?
            } else {
                return Err(StaleHandleError::new_err(
                    "camera capture source is uninitialized",
                ));
            };
            handles.push(handle);
        }
        for (parent, children) in adjacency.iter().enumerate() {
            for &child in children {
                stage
                    .attach(handles[parent], handles[child])
                    .map_err(stage_error)?;
            }
        }
        // Camera.capture consumes explicit painter order. Unlike Scene.add it
        // must not re-sort caller-provided roots by their current z_index.
        let slot = stage.add(Mobject::new());
        stage.add_to_scene(slot).map_err(stage_error)?;
        let roots: Vec<Mob> = roots.into_iter().map(|index| handles[index]).collect();
        stage.replace_in_scene(slot, &roots).map_err(stage_error)?;
        let config = FrameConfig::new(
            Viewport { width, height },
            ScreenMap {
                scale: f64::from(height) / fmn_core::constants::FRAME_HEIGHT,
                origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                y_up: true,
            },
            background,
        )
        .with_aa_policy(camera.aa_policy());
        let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
            frame: config,
            tiling: Tiling::default(),
            engine: EngineIdentity::fast(),
            threads,
        })
        .map_err(native_error)?;
        // Camera projection is essential even for 2D: this route owns the
        // +Y-up scene to top-row-first framebuffer orientation.
        renderer
            .render_with_camera(&stage, &camera)
            .map_err(native_error)?;
        let mut rgba = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, width, height).map_err(native_error)?,
        );
        fmn_frame::convert::rgba16f_to_rgba8(renderer.frame(), &mut rgba).map_err(native_error)?;
        Ok(Self {
            rgba,
            width,
            height,
            png: OnceLock::new(),
        })
    }

    #[getter]
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn pixels<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.rgba.as_bytes())
    }

    /// Encode this immutable snapshot through the owned PNG codec. Lazy caching
    /// avoids encoding at all for array-only clients, or again on redisplay.
    fn png<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let bytes = self.png.get_or_init(|| {
            fmn_codec::png::encode_rgba8(
                self.width,
                self.height,
                self.rgba.as_bytes(),
                fmn_codec::deflate::CompressionLevel::Default,
            )
        });
        PyBytes::new(py, bytes)
    }

    /// Encode this frozen frame for an explicitly selected terminal protocol.
    /// This returns bytes only: no ambient terminal, file, subprocess or scene
    /// is accessed. Kitty carries the same lossless PNG as `png()`; sixel uses
    /// the Studio encoder's documented 216-color/one-bit-alpha preview palette.
    #[pyo3(signature = (protocol="kitty", *, max_bytes=16_777_216))]
    fn terminal_bytes<'py>(
        &self,
        py: Python<'py>,
        protocol: &str,
        max_bytes: usize,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let protocol = match protocol {
            "kitty" => TerminalProtocol::Kitty,
            "sixel" => TerminalProtocol::Sixel,
            _ => return Err(PyValueError::new_err("protocol must be 'kitty' or 'sixel'")),
        };
        let limits = TuiLimits::default();
        if max_bytes == 0 || max_bytes > limits.max_encoded_bytes {
            return Err(PyValueError::new_err("max_bytes must be in 1..=134217728"));
        }
        if u64::from(self.width) * u64::from(self.height) > limits.max_pixels as u64 {
            return Err(PyValueError::new_err(
                "terminal preview exceeds the 3840x2160 pixel budget",
            ));
        }
        let limits = TuiLimits {
            max_encoded_bytes: max_bytes,
            ..limits
        };
        let bytes = py.detach(|| -> PyResult<Vec<u8>> {
            let encoder = TerminalPreview::new(protocol, limits).map_err(native_error)?;
            let mut bytes = Vec::new();
            match protocol {
                TerminalProtocol::Kitty => {
                    let png = self.png.get_or_init(|| {
                        fmn_codec::png::encode_rgba8(
                            self.width,
                            self.height,
                            self.rgba.as_bytes(),
                            fmn_codec::deflate::CompressionLevel::Default,
                        )
                    });
                    encoder.write_png(&mut bytes, png).map_err(native_error)?;
                }
                TerminalProtocol::Sixel => encoder
                    .write_rgba8(&mut bytes, self.width, self.height, self.rgba.as_bytes())
                    .map_err(native_error)?,
            }
            Ok(bytes)
        })?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// Encode and prepare a native no-clobber PNG without publishing it yet.
    #[pyo3(signature = (destination, threads=1))]
    fn prepare_png(
        &self,
        py: Python<'_>,
        destination: PathBuf,
        threads: usize,
    ) -> PyResult<output::PreparedPng> {
        py.detach(|| output::prepare(&self.rgba, self.width, self.height, destination, threads))
    }

    /// Publish this frozen frame, without recapture or scene lifecycle effects.
    #[pyo3(signature = (destination, threads=1))]
    fn save_png(
        &self,
        py: Python<'_>,
        destination: PathBuf,
        threads: usize,
    ) -> PyResult<(PathBuf, u64, String)> {
        output::save(
            py,
            &self.rgba,
            self.width,
            self.height,
            destination,
            threads,
        )
    }

    /// IPython/Jupyter's image protocol without Pillow, files or a second
    /// renderer. Redisplay cannot run authored callbacks or advance a Scene.
    fn _repr_png_<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        self.png(py)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_terminal_preview_acceptance_suite() {
        crate::with_python_test_module("terminal preview", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/terminal_preview.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native terminal snapshots preserve pixels and scene state");
        });
    }

    #[test]
    fn production_rich_preview_acceptance_suite() {
        crate::with_python_test_module("rich preview", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/camera_snapshot.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native PNG snapshots and observational console preview");
        });
    }

    #[test]
    fn production_camera_readback_acceptance_suite() {
        crate::with_python_test_module("camera readback", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/camera_readback.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native camera readback preserves scene state and pixels");
        });
    }

    /// fm-5wq.46: the semantic witness through the portal's scene render and
    /// camera readback reads right side up, and a planted vertical mirror of
    /// each frame fails the orientation oracles.
    #[test]
    fn portal_routes_pass_the_semantic_sanity_oracles() {
        use pyo3::types::{PyAnyMethods, PyDictMethods};
        crate::with_python_test_module("semantic witness", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/semantic_witness.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("portal frames satisfy the semantic sanity oracles");
            let routes: u64 = globals
                .get_item("semantic_witness_report")
                .unwrap()
                .expect("the suite reports its routes")
                .get_item("routes")
                .unwrap()
                .extract()
                .unwrap();
            assert_eq!(routes, 2);
        });
    }
}
