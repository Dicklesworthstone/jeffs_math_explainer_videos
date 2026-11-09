//! Constructor-only UV-grid publication. Atlas samples; Marionette owns data.
use super::*;

#[path = "portal_point_cloud.rs"]
mod point_cloud;

/// Install a native candidate's geometry into the schema initialized by the
/// authored class, or finalize that class's own point table. This is distinct
/// from live regridding: empty grids and strips are valid at construction.
#[pyfunction(signature = (target, resolution, source=None))]
fn _initialize_surface_grid(
    target: &Bound<'_, BridgeMobject>,
    resolution: (usize, usize),
    source: Option<&Bound<'_, BridgeMobject>>,
) -> PyResult<()> {
    if target.try_borrow()?.engine.is_some() {
        return Err(PyRuntimeError::new_err(
            "surface initialization requires a detached target",
        ));
    }
    let limit = fmn_library::SURFACE_SAMPLING_BUDGET.max_samples();
    let (nu, nv) = resolution;
    let count = nu
        .checked_mul(nv)
        .filter(|&n| nu <= limit && nv <= limit && n <= limit)
        .ok_or_else(|| {
            PyValueError::new_err("surface initialization exceeds its UV-grid budget")
        })?;
    let schema = parse_schema(target)?;
    for (key, width) in [("point", 3), ("d_normal_point", 3), ("rgba", 4)] {
        if !schema
            .fields()
            .iter()
            .any(|f| f.name == key && f.width == width)
        {
            return Err(PyValueError::new_err(
                "surface initialization requires point/normal vec3 and rgba vec4 fields",
            ));
        }
    }
    let geometry = source
        .map(|source| {
            if target.is(source) {
                return Err(PyValueError::new_err("surface candidate must be distinct"));
            }
            with_stage(source, |stage, mob| -> PyResult<_> {
                let entry = stage
                    .get(mob)
                    .ok_or_else(|| StaleHandleError::new_err("surface candidate is stale"))?;
                if entry.render_primitive() != (RenderPrimitive::SurfaceGrid { resolution })
                    || entry.buffer.len() != count
                {
                    return Err(PyValueError::new_err(
                        "surface candidate has mismatched native UV topology",
                    ));
                }
                if ["point", "d_normal_point"].iter().any(|key| {
                    !entry
                        .buffer
                        .schema()
                        .fields()
                        .iter()
                        .any(|field| field.name == *key && field.width == 3)
                }) {
                    return Err(PyValueError::new_err(
                        "surface candidate geometry must use vec3 fields",
                    ));
                }
                Ok((
                    world_column(entry, "point")?,
                    world_column(entry, "d_normal_point")?,
                ))
            })?
        })
        .transpose()?;
    // Schema extraction can invoke Python. Recheck ownership after it, before
    // any write. No Python callbacks occur during preparation or publication.
    if target.try_borrow()?.engine.is_some() {
        return Err(PyRuntimeError::new_err(
            "surface initialization requires a detached target",
        ));
    }
    with_stage(target, |stage, mob| -> PyResult<()> {
        let original = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("surface is stale"))?;
        if original.buffer.schema() != &schema {
            return Err(PyValueError::new_err(
                "surface schema changed during initialization",
            ));
        }
        if !original.submobjects().is_empty() {
            return Err(PyValueError::new_err(
                "surface initialization cannot replace an arena family",
            ));
        }
        if !matches!(
            original.render_primitive(),
            RenderPrimitive::Vector | RenderPrimitive::SurfaceGrid { .. }
        ) {
            return Err(PyTypeError::new_err(
                "surface initialization cannot replace another render primitive",
            ));
        }
        if geometry.is_none() && original.buffer.len() != count {
            return Err(PyValueError::new_err(
                "authored surface points must match resolution",
            ));
        }
        // Bake only a temporary root copy. A failed allocation or validation
        // cannot partially mutate the target, its placement or exported views.
        let mut preparation = Stage::new();
        let copy = stage
            .copy_entry_into(mob, &mut preparation)
            .map_err(stage_error)?;
        preparation.bake_placement(copy).map_err(stage_error)?;
        let entry = preparation
            .get(copy)
            .ok_or_else(|| StaleHandleError::new_err("surface preparation is stale"))?;
        let mut buffer = entry.buffer.snapshot_clone();
        if let Some((points, normals)) = &geometry {
            buffer
                .resize_preserving_order(count)
                .map_err(record_error_to_py)?;
            buffer.write_range("point", 0, points);
            buffer.write_range("d_normal_point", 0, normals);
        } else {
            // Authored point tables receive the same finite world-space proof
            // as sampled candidates before they gain durable surface topology.
            world_column(entry, "point")?;
            world_column(entry, "d_normal_point")?;
        }
        // Empty Surface/SGroup roots are containers, not drawable UV grids.
        // Keeping them unstructured is essential for family-level become and
        // Transform, which query topology only on actual surface leaves.
        let primitive = if count == 0 {
            RenderPrimitive::Vector
        } else {
            RenderPrimitive::SurfaceGrid { resolution }
        };
        let value = Mobject::from_buffer(buffer)
            .with_uniforms(*entry.uniforms())
            .with_z_index(preparation.z_index(copy))
            .with_render_primitive(primitive);
        let candidate = stage.add(value);
        // The one native become operation retains the root's handle, saved
        // state, pins and updater ownership. Its schema/family checks are kept.
        let result = stage
            .become_mobject(mob, candidate, false)
            .map_err(stage_error);
        let cleanup = stage.delete(candidate).map_err(stage_error);
        result?;
        cleanup?;
        Ok(())
    })?
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_initialize_surface_grid, module)?)?;
    point_cloud::install(module)?;
    Ok(())
}

/// Authored Surface hooks through construction, regeneration and native
/// rendering, including the stock solids' subclass hooks (fm-5wq.13). Runs
/// `tests/native_surface_lifecycle.py` and returns how many cases ran.
///
/// # Errors
/// Returns the actual Python/native acceptance failure.
#[cfg(any(test, feature = "gauntlet"))]
pub fn run_portal_gauntlet_surface_lifecycle() -> Result<u64, String> {
    crate::with_python_test_module("authored surface initialization", |py, _module, globals| {
        let code = std::ffi::CString::new(include_str!("../tests/native_surface_lifecycle.py"))
            .map_err(|error| error.to_string())?;
        py.run(code.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        globals
            .get_item("run_native_surface_lifecycle")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "surface lifecycle entry point is absent".to_owned())?
            .call0()
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<u64>())
            .map_err(|error| error.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_surface_lifecycle_and_native_rendering() {
        assert_eq!(super::run_portal_gauntlet_surface_lifecycle().unwrap(), 15);
    }

    #[test]
    fn authored_axial_solids_and_native_rendering() {
        crate::with_python_test_module("authored axial solids", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/native_axial_solids.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_axial_solids")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn authored_solid_lifecycle_and_native_rendering() {
        crate::with_python_test_module("authored solid initialization", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/native_solid_lifecycle.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_solid_lifecycle")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
