//! In-memory raster authoring over Atlas and Marionette's immutable resources.
//! Preparation never touches a scene; publication changes only its image axis.

use crate::{
    BridgeMobject, PyRuntimeError, PyValueError, StaleHandleError, install_native_tree,
    native_error, stage_error, with_stage,
};
use fmn_mobject::{ImageColorSpace, ImageResource, ImageSampler, RenderPrimitive};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};

#[path = "portal_raster_init.rs"]
mod initialization;

#[path = "portal_raster_transition.rs"]
mod transition;

const MAX_PIXELS: u64 = 16_777_216;
const MAX_ENCODED_BYTES: usize = 64 * 1024 * 1024;

/// Owned preparation value; shared pixels are immutable, not a writable view.
#[pyclass(frozen, name = "_RasterImage")]
pub(super) struct RasterImage {
    pub(super) resource: ImageResource,
}

#[pymethods]
impl RasterImage {
    #[new]
    fn new(py: Python<'_>, width: u32, height: u32, rgba: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let count = u64::from(width) * u64::from(height);
        if width == 0 || height == 0 || count > MAX_PIXELS {
            return Err(PyValueError::new_err(
                "raster dimensions exceed the 16M-pixel budget or are empty",
            ));
        }
        if rgba.as_bytes().len() as u64 != count * 4 {
            return Err(PyValueError::new_err(
                "raster input must contain exactly width * height * 4 RGBA8 bytes",
            ));
        }
        let bytes = rgba.as_bytes();
        py.detach(|| {
            ImageResource::rgba8(
                width,
                height,
                bytes.to_vec(),
                ImageColorSpace::Srgb,
                ImageSampler::default(),
            )
            .map(|resource| Self { resource })
            .map_err(native_error)
        })
    }

    #[staticmethod]
    pub(super) fn decode(py: Python<'_>, encoded: &Bound<'_, PyBytes>) -> PyResult<Self> {
        let bytes = encoded.as_bytes();
        if bytes.len() > MAX_ENCODED_BYTES {
            return Err(PyValueError::new_err(
                "raster input exceeds the 64 MiB encoded budget",
            ));
        }
        py.detach(|| {
            // Bound the decoder BEFORE allocation/decompression, not afterwards.
            let image = fmn_library::ImageMobject::from_bytes_with_limits(
                bytes,
                &fmn_codec::PngLimits {
                    max_pixels: MAX_PIXELS,
                    ..fmn_codec::PngLimits::default()
                },
                &fmn_codec::JpegLimits {
                    max_pixels: MAX_PIXELS,
                },
            )
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
            let mut object: fmn_mobject::Mobject = image.into();
            Ok(Self {
                resource: object.image.take().ok_or_else(|| {
                    PyRuntimeError::new_err("native image decoder returned no resource")
                })?,
            })
        })
    }

    fn __copy__(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.__copy__()
    }

    #[getter]
    fn size(&self) -> (u32, u32) {
        (self.resource.width(), self.resource.height())
    }

    fn pixels<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.resource.pixels())
    }
}

#[pyfunction(signature = (target, image, factory, height=4.0, opacity=1.0, z_index=0))]
fn _build_raster_image<'py>(
    target: &Bound<'py, BridgeMobject>,
    image: &RasterImage,
    factory: &Bound<'py, PyAny>,
    height: f64,
    opacity: f64,
    z_index: i32,
) -> PyResult<Bound<'py, PyList>> {
    let width = height * f64::from(image.resource.width()) / f64::from(image.resource.height());
    if !height.is_finite()
        || height <= 0.0
        || height > f64::from(f32::MAX)
        || !width.is_finite()
        || width > f64::from(f32::MAX)
        || !opacity.is_finite()
        || opacity.abs() > f64::from(f32::MAX)
    {
        return Err(PyValueError::new_err(
            "image size and opacity must be finite and f32-representable; height must be positive",
        ));
    }
    let object = fmn_library::ImageMobject::from_resource(image.resource.clone())
        .map_err(native_error)?
        .with_height(height)
        .with_opacity(opacity)
        .with_z_index(z_index);
    install_native_tree(target, factory, object)
}

#[pyfunction(signature = (target, dark=false))]
fn _read_raster_image(target: &Bound<'_, BridgeMobject>, dark: bool) -> PyResult<RasterImage> {
    with_stage(target, |stage, mob| {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("raster target is stale"))?;
        let image = entry
            .image_resource()
            .ok_or_else(|| PyValueError::new_err("mobject has no native image resource"))?;
        let selected = if dark {
            image
                .dark_image()
                .ok_or_else(|| PyValueError::new_err("mobject has no dark-side image resource"))?
        } else {
            image
        };
        Ok(RasterImage {
            resource: selected.clone().without_dark_image(),
        })
    })?
}

#[pyfunction(signature = (target, image, dark_image=None))]
fn _replace_raster_image(
    target: &Bound<'_, BridgeMobject>,
    image: &RasterImage,
    dark_image: Option<&RasterImage>,
) -> PyResult<()> {
    let mut candidate = image.resource.clone();
    if let Some(dark) = dark_image {
        candidate = candidate
            .with_dark_image(dark.resource.clone())
            .map_err(native_error)?;
    }
    with_stage(target, |stage, mob| -> PyResult<()> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("raster target is stale"))?;
        if entry.image_resource().is_none()
            || !matches!(
                entry.render_primitive(),
                RenderPrimitive::ImageQuad
                    | RenderPrimitive::SurfaceGrid { .. }
                    | RenderPrimitive::TriangleMesh
            )
        {
            return Err(PyValueError::new_err(
                "pixel replacement requires an existing native image or textured surface",
            ));
        }
        if entry.render_primitive() == RenderPrimitive::ImageQuad
            && candidate.dark_image().is_some()
        {
            return Err(PyValueError::new_err(
                "image quads do not accept dark-side textures",
            ));
        }
        // No geometry bake/resize, owner change, callback or frame-clock step.
        stage
            .set_image_resource(mob, Some(candidate))
            .map_err(stage_error)?;
        Ok(())
    })?
}

/// Compare complete immutable descriptors as well as pixels, including the
/// dark side. Checkpoints must not reuse a mirror after an image-only edit.
#[pyfunction]
fn _raster_images_equal(
    left: &Bound<'_, BridgeMobject>,
    right: &Bound<'_, BridgeMobject>,
) -> PyResult<bool> {
    let read = |object: &Bound<'_, BridgeMobject>| -> PyResult<Option<ImageResource>> {
        with_stage(object, |stage, mob| {
            stage
                .get(mob)
                .map(|entry| entry.image_resource().cloned())
                .ok_or_else(|| StaleHandleError::new_err("raster comparison target is stale"))
        })?
    };
    Ok(read(left)? == read(right)?)
}

pub(crate) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RasterImage>()?;
    initialization::install(module)?;
    transition::install(module)?;
    module.add_function(wrap_pyfunction!(_raster_images_equal, module)?)?;
    module.add_function(wrap_pyfunction!(_build_raster_image, module)?)?;
    module.add_function(wrap_pyfunction!(_read_raster_image, module)?)?;
    module.add_function(wrap_pyfunction!(_replace_raster_image, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_raster_frame_and_texture_acceptance() {
        crate::with_python_test_module("raster frames and textures", |py, _module, globals| {
            for text in [
                include_str!("../tests/raster_textures.py"),
                include_str!("../tests/raster_frames.py"),
            ] {
                let source = std::ffi::CString::new(text).unwrap();
                py.run(source.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("native live texture frames and history");
            }
        });
    }

    #[test]
    fn production_raster_authoring_acceptance() {
        crate::with_python_test_module("raster authoring", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/raster_authoring.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native raster authoring and snapshots");
        });
    }
}
