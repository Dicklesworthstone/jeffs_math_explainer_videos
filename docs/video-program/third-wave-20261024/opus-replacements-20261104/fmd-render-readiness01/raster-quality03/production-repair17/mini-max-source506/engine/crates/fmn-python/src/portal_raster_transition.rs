//! Frozen raster transitions. Lumen owns sampling and transfer; Marionette owns
//! publication. No live object or Python callback is retained by the plan.
use super::{_replace_raster_image, RasterImage};
use crate::{BridgeMobject, PyValueError, StaleHandleError, native_error, with_stage};
use fmn_core::color::PremulRgba;
use fmn_mobject::{ImageColorSpace, ImageResource, ImageSampler, ImageWrap, RenderPrimitive};
use fmn_render::texture::{SamplerPolicy, Texture, TextureEncoding, TextureWrap};
use pyo3::prelude::*;
use std::sync::Arc;

const MAX_PIXELS: u64 = 16_777_216;
// Four f32 lanes per decoded texel. Count all endpoints conservatively before
// the first decode; missing dark sides share the corresponding light texture.
const MAX_DECODED_TEXELS: u64 = 16_777_216;

fn count(image: &ImageResource) -> u64 {
    u64::from(image.width()) * u64::from(image.height())
}

fn policy(sampler: ImageSampler) -> SamplerPolicy {
    let wrap = |value| match value {
        ImageWrap::Repeat => TextureWrap::Repeat,
        ImageWrap::ClampToEdge => TextureWrap::ClampToEdge,
        ImageWrap::MirroredRepeat => TextureWrap::MirroredRepeat,
    };
    SamplerPolicy {
        wrap_u: wrap(sampler.wrap_u),
        wrap_v: wrap(sampler.wrap_v),
    }
}

fn decode(image: &ImageResource) -> PyResult<Arc<Texture>> {
    let encoding = match image.color_space() {
        ImageColorSpace::Srgb => TextureEncoding::Srgb,
        ImageColorSpace::Linear => TextureEncoding::Linear,
        ImageColorSpace::Gamma(value) => TextureEncoding::Gamma(value),
    };
    Texture::from_rgba8(image.width(), image.height(), image.pixels(), encoding)
        .map(Arc::new)
        .map_err(native_error)
}

fn shape(left: &ImageResource, right: &ImageResource) -> PyResult<(u32, u32)> {
    let size = (
        left.width().max(right.width()),
        left.height().max(right.height()),
    );
    if u64::from(size.0) * u64::from(size.1) > MAX_PIXELS {
        return Err(PyValueError::new_err(
            "raster transition output exceeds its 16M-pixel budget",
        ));
    }
    Ok(size)
}

fn resources(
    object: &Bound<'_, BridgeMobject>,
) -> PyResult<(Option<ImageResource>, RenderPrimitive)> {
    with_stage(object, |stage, mob| {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("raster transform endpoint is stale"))?;
        Ok((entry.image_resource().cloned(), entry.render_primitive()))
    })?
}

fn endpoints(
    left: &Bound<'_, BridgeMobject>,
    right: &Bound<'_, BridgeMobject>,
) -> PyResult<Option<(ImageResource, ImageResource)>> {
    let (a, a_kind) = resources(left)?;
    let (b, b_kind) = resources(right)?;
    match (a, b) {
        (None, None) => Ok(None),
        (Some(a), Some(b))
            if matches!(
                (a_kind, b_kind),
                (RenderPrimitive::ImageQuad, RenderPrimitive::ImageQuad)
                    | (RenderPrimitive::TriangleMesh, RenderPrimitive::TriangleMesh)
                    | (
                        RenderPrimitive::SurfaceGrid { .. },
                        RenderPrimitive::SurfaceGrid { .. }
                    )
            ) =>
        {
            Ok(Some((a, b)))
        }
        _ => Err(PyValueError::new_err(
            "raster transform endpoints require compatible native image primitives",
        )),
    }
}

fn decoded_texels(start: &ImageResource, end: &ImageResource) -> PyResult<u64> {
    if start.sampler() != end.sampler() {
        return Err(PyValueError::new_err(
            "raster transition endpoints must use the same sampler",
        ));
    }
    // Constant materials retain their original descriptor; no decode or
    // intermediate lattice is needed, even for very large identical endpoints.
    if start == end {
        return Ok(0);
    }
    let total = count(start)
        + count(end)
        + start.dark_image().map_or(0, count)
        + end.dark_image().map_or(0, count);
    if total > MAX_DECODED_TEXELS {
        return Err(PyValueError::new_err(
            "raster transition exceeds its 256 MiB decoded endpoint budget",
        ));
    }
    shape(start, end)?;
    if start.dark_image().is_some() || end.dark_image().is_some() {
        shape(
            start.dark_image().unwrap_or(start),
            end.dark_image().unwrap_or(end),
        )?;
    }
    Ok(total)
}

#[derive(Clone)]
struct Plane {
    left: Arc<Texture>,
    right: Arc<Texture>,
    size: (u32, u32),
}

impl Plane {
    fn sample(&self, alpha: f64, sampler: ImageSampler) -> PyResult<ImageResource> {
        let (width, height) = self.size;
        let length = usize::try_from(u64::from(width) * u64::from(height) * 4)
            .map_err(|_| PyValueError::new_err("raster transition byte length overflow"))?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(native_error)?;
        let sampling = policy(sampler);
        for y in 0..height {
            for x in 0..width {
                let uv = [
                    (f64::from(x) + 0.5) / f64::from(width),
                    (f64::from(y) + 0.5) / f64::from(height),
                ];
                let a = self.left.sample(uv, sampling).premultiply();
                let b = self.right.sample(uv, sampling).premultiply();
                let lerp = |start: f64, end: f64| (1.0 - alpha) * start + alpha * end;
                let color = PremulRgba {
                    r: lerp(a.r, b.r),
                    g: lerp(a.g, b.g),
                    b: lerp(a.b, b.b),
                    a: lerp(a.a, b.a),
                }
                .unpremultiply();
                bytes.extend_from_slice(&color.to_srgb().to_rgb8());
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                bytes.push((color.a.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
        ImageResource::rgba8(width, height, bytes, ImageColorSpace::Srgb, sampler)
            .map_err(native_error)
    }
}

/// Immutable start/end materials and their bounded, decoded sampling plans.
#[derive(Clone)]
#[pyclass(frozen, from_py_object, name = "_RasterTransition")]
struct RasterTransition {
    start: ImageResource,
    end: ImageResource,
    light: Option<Plane>,
    dark: Option<Plane>,
}

#[pymethods]
impl RasterTransition {
    #[new]
    #[pyo3(signature = (target, image, dark_image=None))]
    fn new(
        target: &Bound<'_, BridgeMobject>,
        image: &RasterImage,
        dark_image: Option<&RasterImage>,
    ) -> PyResult<Self> {
        let (start, primitive) = with_stage(target, |stage, mob| {
            let entry = stage
                .get(mob)
                .ok_or_else(|| StaleHandleError::new_err("raster transition target is stale"))?;
            let start = entry.image_resource().cloned().ok_or_else(|| {
                PyValueError::new_err("raster transition requires a native image material")
            })?;
            Ok::<_, PyErr>((start, entry.render_primitive()))
        })??;
        let mut end = image.resource.clone();
        if let Some(dark) = dark_image {
            end = end
                .with_dark_image(dark.resource.clone())
                .map_err(native_error)?;
        }
        if !matches!(
            primitive,
            RenderPrimitive::ImageQuad
                | RenderPrimitive::SurfaceGrid { .. }
                | RenderPrimitive::TriangleMesh
        ) || (primitive == RenderPrimitive::ImageQuad && end.dark_image().is_some())
        {
            return Err(PyValueError::new_err(
                "raster transition requires a compatible image or textured surface",
            ));
        }
        if decoded_texels(&start, &end)? == 0 {
            return Ok(Self {
                start,
                end,
                light: None,
                dark: None,
            });
        }
        let light_shape = shape(&start, &end)?;
        let has_dark = start.dark_image().is_some() || end.dark_image().is_some();
        let dark_shape = if has_dark {
            Some(shape(
                start.dark_image().unwrap_or(&start),
                end.dark_image().unwrap_or(&end),
            )?)
        } else {
            None
        };
        // All shape/storage checks precede endpoint conversion. This plan owns
        // only immutable resources; native proxies never cross the GIL boundary.
        target.py().detach(move || {
            let left = decode(&start)?;
            let right = decode(&end)?;
            let dark = if let Some(size) = dark_shape {
                Some(Plane {
                    left: start
                        .dark_image()
                        .map_or_else(|| Ok(left.clone()), decode)?,
                    right: end.dark_image().map_or_else(|| Ok(right.clone()), decode)?,
                    size,
                })
            } else {
                None
            };
            Ok(Self {
                start,
                end,
                light: Some(Plane {
                    left,
                    right,
                    size: light_shape,
                }),
                dark,
            })
        })
    }

    /// Inspect complete native materials without decoding or modifying either endpoint.
    #[staticmethod]
    fn required_texels(
        left: &Bound<'_, BridgeMobject>,
        right: &Bound<'_, BridgeMobject>,
    ) -> PyResult<u64> {
        endpoints(left, right)?.map_or(Ok(0), |(a, b)| decoded_texels(&a, &b))
    }

    /// Build the same interpolation plan for Transform's actual aligned endpoints.
    #[staticmethod]
    fn between(
        left: &Bound<'_, BridgeMobject>,
        right: &Bound<'_, BridgeMobject>,
    ) -> PyResult<Option<Self>> {
        match endpoints(left, right)? {
            None => Ok(None),
            Some((_, end)) => Self::new(left, &RasterImage { resource: end }, None).map(Some),
        }
    }

    fn matches(
        &self,
        left: &Bound<'_, BridgeMobject>,
        right: &Bound<'_, BridgeMobject>,
    ) -> PyResult<bool> {
        Ok(endpoints(left, right)?.is_some_and(|(a, b)| a == self.start && b == self.end))
    }

    #[getter]
    fn start_has_dark(&self) -> bool {
        self.start.dark_image().is_some()
    }

    #[getter]
    fn end_has_dark(&self) -> bool {
        self.end.dark_image().is_some()
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.clone()
    }

    #[getter]
    fn has_dark(&self) -> bool {
        self.start.dark_image().is_some() || self.end.dark_image().is_some()
    }

    /// Produce an independent immutable material without advancing a scene.
    fn sample(&self, py: Python<'_>, alpha: f64) -> PyResult<RasterImage> {
        if !alpha.is_finite() {
            return Err(PyValueError::new_err(
                "raster transition alpha must be finite",
            ));
        }
        // Exact endpoint resources include original transfer, sampler, dimensions
        // and hidden transparent RGB. Overshooting rate curves clamp coverage.
        if alpha <= 0.0 {
            return Ok(RasterImage {
                resource: self.start.clone(),
            });
        }
        if alpha >= 1.0 {
            return Ok(RasterImage {
                resource: self.end.clone(),
            });
        }
        let Some(light) = &self.light else {
            return Ok(RasterImage {
                resource: self.start.clone(),
            });
        };
        py.detach(|| {
            let mut resource = light.sample(alpha, self.start.sampler())?;
            if let Some(dark) = &self.dark {
                resource = resource
                    .with_dark_image(dark.sample(alpha, self.start.sampler())?)
                    .map_err(native_error)?;
            }
            Ok(RasterImage { resource })
        })
    }

    /// Prepare the whole light/dark result, then publish one image revision.
    fn apply(&self, target: &Bound<'_, BridgeMobject>, alpha: f64) -> PyResult<()> {
        let candidate = self.sample(target.py(), alpha)?;
        _replace_raster_image(target, &candidate, None)
    }
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<RasterTransition>()
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;

    #[test]
    fn production_transform_material_kernel_acceptance() {
        crate::with_python_test_module("transform materials", |py, _module, globals| {
            for source in [
                include_str!("../tests/raster_transform_kernel.py"),
                include_str!("../tests/raster_transform.py"),
            ] {
                let text = std::ffi::CString::new(source).unwrap();
                py.run(text.as_c_str(), Some(globals), Some(globals))
                    .inspect_err(|error| error.print(py))
                    .expect("native Transform material plans and lifecycle");
            }
            globals
                .get_item("run_raster_transform_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .expect("native Transform lifecycle and rendered frames");
        });
    }

    #[test]
    fn production_raster_animation_acceptance() {
        crate::with_python_test_module("raster animation", |py, _module, globals| {
            let text =
                std::ffi::CString::new(include_str!("../tests/raster_animation.py")).unwrap();
            py.run(text.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native raster animation lifecycle and frames");
        });
    }

    #[test]
    fn production_raster_transition_kernel_acceptance() {
        crate::with_python_test_module("raster transition", |py, _module, globals| {
            let text = std::ffi::CString::new(include_str!("../tests/raster_transition_kernel.py"))
                .unwrap();
            py.run(text.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native raster transition colors and resources");
        });
    }
}
