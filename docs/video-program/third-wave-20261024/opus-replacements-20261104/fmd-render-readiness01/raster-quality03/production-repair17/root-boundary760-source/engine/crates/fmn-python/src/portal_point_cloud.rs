//! Point-cloud records use the normal cached hooks with their native draw tag.
use super::*;

fn point_cloud_schema(target: &Bound<'_, BridgeMobject>) -> PyResult<RecordSchema> {
    let schema = parse_schema(target)?;
    for (name, width) in [("point", 3), ("radius", 1), ("rgba", 4), ("glow_factor", 1)] {
        if schema.field_width(name) != Some(width) {
            return Err(PyValueError::new_err(
                "dot-cloud records require point[3], radius[1], rgba[4] and glow_factor[1]",
            ));
        }
    }
    Ok(schema)
}

/// Allocate once, with native identity established BEFORE any authored hook.
/// A copy/save_state made inside init_points must already be a dot cloud.
#[pyfunction]
fn _initialize_point_cloud(target: &Bound<'_, BridgeMobject>) -> PyResult<()> {
    {
        let cell = target.try_borrow()?;
        if cell.engine.is_some() || cell.initialized {
            return Err(PyRuntimeError::new_err(
                "point-cloud engine initialization requires a fresh detached target",
            ));
        }
    }
    let schema = point_cloud_schema(target)?;
    let value = Mobject::from_buffer(RecordBuffer::new(schema, 0).map_err(record_error_to_py)?)
        .with_render_primitive(RenderPrimitive::DotCloud);
    {
        // Schema descriptors may execute Python, including reentrant init or
        // scene adoption. Never overwrite the result of those operations.
        let mut cell = target.try_borrow_mut()?;
        if cell.engine.is_some() || cell.initialized {
            return Err(PyRuntimeError::new_err(
                "point-cloud ownership changed during schema validation",
            ));
        }
        cell.nursery = Some(Nursery::new(value));
        cell.initialized = true;
    }
    // Same cached native hook protocol as BridgeMobject::_engine_init. No
    // Stage or proxy borrow survives a hook, and original Python errors win.
    for hook in ["init_data", "init_points", "init_uniforms"] {
        crossing::record(CrossingClass::MethodDispatch);
        method_cache::call_cached0(target.as_any(), hook)?;
    }
    Ok(())
}

/// Validate constructor output without replacing records, placement or owners.
#[pyfunction]
fn _validate_point_cloud(target: &Bound<'_, BridgeMobject>) -> PyResult<()> {
    let schema = point_cloud_schema(target)?;
    if target.try_borrow()?.engine.is_some() {
        return Err(PyRuntimeError::new_err(
            "point-cloud construction requires a detached target",
        ));
    }
    with_stage(target, |stage, mob| -> PyResult<()> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("point-cloud construction target is stale"))?;
        if entry.buffer.schema() != &schema || entry.render_primitive() != RenderPrimitive::DotCloud
        {
            return Err(PyValueError::new_err(
                "point-cloud schema or native primitive changed during construction",
            ));
        }
        for name in ["point", "radius", "rgba", "glow_factor"] {
            let values = entry
                .buffer
                .read_column(name)
                .ok_or_else(|| PyValueError::new_err("point-cloud record field is missing"))?;
            if values.iter().any(|value| !value.is_finite())
                || (matches!(name, "radius" | "glow_factor")
                    && values.iter().any(|&value| value < 0.0))
            {
                return Err(PyValueError::new_err(
                    "point-cloud records must be finite with non-negative radii and glow factors",
                ));
            }
        }
        let aa = entry.uniforms().anti_alias_width;
        if !aa.is_finite() || aa < 0.0 {
            return Err(PyValueError::new_err(
                "point-cloud anti-alias width must be finite and non-negative",
            ));
        }
        if entry
            .placement()
            .coefficients()
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(PyValueError::new_err(
                "point-cloud placement must be finite",
            ));
        }
        Ok(())
    })?
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_initialize_point_cloud, module)?)?;
    module.add_function(wrap_pyfunction!(_validate_point_cloud, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_point_cloud_records_and_native_rendering() {
        crate::with_python_test_module("authored point clouds", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/native_point_cloud_lifecycle.py"))
                    .unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_point_cloud_lifecycle")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn live_point_cloud_materials_and_retained_frames() {
        crate::with_python_test_module("live point-cloud materials", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/native_point_cloud_materials.py"))
                    .unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_point_cloud_materials")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
