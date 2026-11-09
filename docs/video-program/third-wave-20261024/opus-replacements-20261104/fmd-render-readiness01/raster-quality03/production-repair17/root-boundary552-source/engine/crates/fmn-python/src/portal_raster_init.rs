//! Constructor-time image admission without replacing the authored record schema.
//!
//! Atlas owns the canonical quad. This boundary admits its columns or an
//! authored six-row table into Lumen's existing image primitive. It does not
//! decode pixels, create a second renderer, or replace a Python family.

use super::RasterImage;
use crate::{
    BridgeMobject, PyRuntimeError, PyValueError, StaleHandleError, parse_schema, stage_error,
    with_stage,
};
use fmn_mobject::{Mobject, RenderPrimitive, Stage};
use pyo3::prelude::*;

#[pyfunction]
fn _initialize_raster_image(
    target: &Bound<'_, BridgeMobject>,
    image: &RasterImage,
) -> PyResult<()> {
    if target.try_borrow()?.engine.is_some() {
        return Err(PyRuntimeError::new_err(
            "image initialization requires a detached target; use set_image for live pixels",
        ));
    }
    if image.resource.dark_image().is_some() {
        return Err(PyValueError::new_err(
            "image quads require an unpaired resource",
        ));
    }
    let schema = parse_schema(target)?;
    for (name, width) in [("point", 3), ("im_coords", 2), ("opacity", 1)] {
        if !schema
            .fields()
            .iter()
            .any(|field| field.name == name && field.width == width)
        {
            return Err(PyValueError::new_err(
                "image initialization requires point vec3, im_coords vec2, and opacity scalar fields",
            ));
        }
    }
    // Reading authored schema descriptors can execute Python, including an
    // attempt to adopt this object into a Scene. Recheck before borrowing it.
    if target.try_borrow()?.engine.is_some() {
        return Err(PyRuntimeError::new_err(
            "image ownership changed during initialization",
        ));
    }
    with_stage(target, |stage, mob| -> PyResult<()> {
        let original = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("image initialization target is stale"))?;
        if original.buffer.schema() != &schema {
            return Err(PyValueError::new_err(
                "image schema changed during initialization",
            ));
        }
        if original.buffer.len() != 6 {
            return Err(PyValueError::new_err(
                "an image quad requires exactly six records",
            ));
        }
        if !original.submobjects().is_empty() {
            return Err(PyValueError::new_err(
                "image initialization cannot replace an arena family",
            ));
        }
        if !matches!(
            original.render_primitive(),
            RenderPrimitive::Vector | RenderPrimitive::ImageQuad
        ) {
            return Err(PyValueError::new_err(
                "image initialization cannot replace another render primitive",
            ));
        }
        for name in ["point", "im_coords", "opacity"] {
            let column = original
                .buffer
                .read_column(name)
                .ok_or_else(|| PyValueError::new_err("image column is missing"))?;
            if column.iter().any(|value| !value.is_finite()) {
                return Err(PyValueError::new_err(
                    "image points, texture coordinates and opacity must be finite",
                ));
            }
        }
        // Work on a private root copy: placement baking and allocation failure
        // must not mutate the target or an exported view during admission.
        let mut preparation = Stage::new();
        let copy = stage
            .copy_entry_into(mob, &mut preparation)
            .map_err(stage_error)?;
        preparation.bake_placement(copy).map_err(stage_error)?;
        let entry = preparation
            .get(copy)
            .ok_or_else(|| StaleHandleError::new_err("image preparation is stale"))?;
        if entry
            .buffer
            .read_column("point")
            .is_none_or(|points| points.iter().any(|value| !value.is_finite()))
        {
            return Err(PyValueError::new_err(
                "image placement must produce finite record coordinates",
            ));
        }
        // Admission is a metadata-only operation. The private baked copy
        // above proves representability; it is NOT the data to publish. Keep
        // the original object-space records and affine placement bit-for-bit.
        let original = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("image target is stale"))?;
        let placement = original.placement();
        let scratch = original.buffer.snapshot_clone();
        let value = Mobject::from_buffer(scratch.snapshot_clone())
            .with_uniforms(*original.uniforms())
            .with_z_index(stage.z_index(mob))
            .with_image_resource(image.resource.clone());
        let candidate = stage.add(value);
        if let Err(error) = stage.set_placement(candidate, placement) {
            let _ = stage.delete(candidate);
            return Err(stage_error(error));
        }
        // Generic become intentionally replaces storage even at equal sizes
        // (V6). Run that existing metadata transfer against private scratch
        // storage, then put back the exact original RecordBuffer. No Python
        // callback or proxy observation occurs while this Stage is borrowed.
        // This preserves live views without changing become/copy semantics.
        let retained = std::mem::replace(
            &mut stage.get_mut(mob).expect("prechecked image target").buffer,
            scratch,
        );
        let result = stage
            .become_mobject(mob, candidate, false)
            .map_err(stage_error);
        stage.get_mut(mob).expect("prechecked image target").buffer = retained;
        let cleanup = stage.delete(candidate).map_err(stage_error);
        result?;
        cleanup?;
        Ok(())
    })?
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_initialize_raster_image, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;

    #[test]
    fn production_raster_initialization_admission() {
        crate::with_python_test_module("image initialization", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/raster_initialization.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native image initialization admission");
        });
    }

    #[test]
    fn production_raster_subclass_lifecycle() {
        crate::with_python_test_module("image subclass lifecycle", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/raster_lifecycle.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native image lifecycle definitions");
            globals
                .get_item("run_raster_lifecycle")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .expect("native image subclass lifecycle");
        });
    }
}
