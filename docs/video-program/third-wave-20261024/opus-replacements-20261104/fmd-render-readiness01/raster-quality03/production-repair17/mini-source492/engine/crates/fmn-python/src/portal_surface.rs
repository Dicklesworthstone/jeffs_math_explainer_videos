//! Live surface geometry on the existing Stage/view protocol.
//! Sampling is Atlas's job. This seam validates and commits its complete result
//! without replacing the surface handle, family, material, or updater owner.
use super::*;

#[path = "portal_surface_init.rs"]
mod initialization;
#[cfg(feature = "gauntlet")]
pub use initialization::run_portal_gauntlet_surface_lifecycle;

fn grid(entry: &fmn_mobject::Entry) -> PyResult<Option<(usize, usize)>> {
    let RenderPrimitive::SurfaceGrid { resolution } = entry.render_primitive() else {
        return Ok(None);
    };
    let (nu, nv) = resolution;
    if nu < 2
        || nv < 2
        || nu.checked_mul(nv) != Some(entry.buffer.len())
        || entry.buffer.len() > fmn_library::SURFACE_SAMPLING_BUDGET.max_samples()
    {
        return Err(PyValueError::new_err(format!(
            "surface topology is inconsistent or exceeds the {}-point budget",
            fmn_library::SURFACE_SAMPLING_BUDGET.max_samples()
        )));
    }
    for key in ["point", "d_normal_point"] {
        if !entry
            .buffer
            .schema()
            .fields()
            .iter()
            .any(|f| f.name == key && f.width == 3)
        {
            return Err(PyValueError::new_err(
                "surface geometry requires point and normal-control vec3 fields",
            ));
        }
    }
    Ok(Some(resolution))
}

#[pyfunction]
fn _surface_grid_resolution(target: &Bound<'_, BridgeMobject>) -> PyResult<Option<(usize, usize)>> {
    with_stage(target, |stage, mob| {
        grid(
            stage
                .get(mob)
                .ok_or_else(|| StaleHandleError::new_err("surface is stale"))?,
        )
    })?
}

struct Geometry {
    resolution: (usize, usize),
    points: Vec<f32>,
    normal_points: Vec<f32>,
}

fn world_column(entry: &fmn_mobject::Entry, key: &str) -> PyResult<Vec<f32>> {
    let input = entry
        .buffer
        .read_column(key)
        .ok_or_else(|| PyValueError::new_err("surface field is missing"))?;
    let mut output = Vec::with_capacity(input.len());
    // An identity placement is a no-op, as in `Stage::bake_placement`.
    // Evaluating it anyway turns every -0.0 into +0.0 and breaks the
    // native-bit contract of sampled surfaces.
    let placement = entry.placement();
    let identity = placement.is_identity();
    for point in input.as_chunks::<3>().0 {
        let local = point.map(f64::from);
        let world = if identity {
            local
        } else {
            placement.apply_point(local)
        };
        if world
            .iter()
            .any(|v| !v.is_finite() || v.abs() > f64::from(f32::MAX))
        {
            return Err(PyValueError::new_err(
                "surface geometry must be finite and f32-representable",
            ));
        }
        #[allow(clippy::cast_possible_truncation)]
        output.extend(world.map(|v| v as f32));
    }
    Ok(output)
}

#[pyfunction]
fn _copy_surface_geometry(
    target: &Bound<'_, BridgeMobject>,
    source: &Bound<'_, BridgeMobject>,
) -> PyResult<()> {
    // Read the complete source first, releasing its engine before borrowing the
    // target. Source and target may share an engine or be in different nurseries.
    let geometry = with_stage(source, |stage, mob| -> PyResult<Geometry> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("source surface is stale"))?;
        Ok(Geometry {
            resolution: grid(entry)?
                .ok_or_else(|| PyTypeError::new_err("source must be a native UV surface"))?,
            points: world_column(entry, "point")?,
            normal_points: world_column(entry, "d_normal_point")?,
        })
    })??;
    with_stage(target, |stage, mob| -> PyResult<()> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("target surface is stale"))?;
        if grid(entry)? != Some(geometry.resolution) {
            return Err(PyValueError::new_err(
                "live surface regeneration requires matching native UV topology",
            ));
        }
        // This also preserves any additional schema-declared pointlike fields
        // in world space. Neither bake nor the writes below invoke host code.
        stage.bake_placement(mob).map_err(stage_error)?;
        let entry = stage
            .get_mut(mob)
            .ok_or_else(|| StaleHandleError::new_err("target surface is stale"))?;
        entry.buffer.write_range("point", 0, &geometry.points);
        entry
            .buffer
            .write_range("d_normal_point", 0, &geometry.normal_points);
        Ok(())
    })?
}

/// Publish newly sampled positions at a different UV resolution. The shared
/// Marionette owner retains and regrids destination paint/UV/custom records;
/// this bridge only reads the source's world-space geometry.
#[pyfunction]
fn _regrid_surface_geometry(
    target: &Bound<'_, BridgeMobject>,
    source: &Bound<'_, BridgeMobject>,
) -> PyResult<()> {
    let geometry = with_stage(source, |stage, mob| -> PyResult<Geometry> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("source surface is stale"))?;
        Ok(Geometry {
            resolution: grid(entry)?
                .ok_or_else(|| PyTypeError::new_err("source must be a native UV surface"))?,
            points: world_column(entry, "point")?,
            normal_points: world_column(entry, "d_normal_point")?,
        })
    })??;
    with_stage(target, |stage, mob| -> PyResult<()> {
        let entry = stage
            .get(mob)
            .ok_or_else(|| StaleHandleError::new_err("target surface is stale"))?;
        let update = fmn_mobject::stage::SurfaceGridUpdate::for_geometry(
            entry,
            geometry.resolution,
            &geometry.points,
            &geometry.normal_points,
        )
        .map_err(native_error)?;
        stage
            .apply_surface_grid_update(mob, update)
            .map_err(native_error)
    })?
}

/// Align proxies even when one is scene-bound and the other is detached.
/// Prepare both immutable entry copies before changing either actual owner.
/// The native Stage operation remains the only UV resampling implementation.
#[pyfunction]
fn _align_surface_grids(
    left: &Bound<'_, BridgeMobject>,
    right: &Bound<'_, BridgeMobject>,
) -> PyResult<()> {
    let mut preparation = Stage::new();
    let a = copy_proxy_entry_into(left, &mut preparation)?;
    let b = copy_proxy_entry_into(right, &mut preparation)?;
    let (a_update, b_update) = fmn_mobject::stage::prepare_surface_grid_alignment(
        preparation
            .get(a)
            .ok_or_else(|| StaleHandleError::new_err("left surface is stale"))?,
        preparation
            .get(b)
            .ok_or_else(|| StaleHandleError::new_err("right surface is stale"))?,
    )
    .map_err(native_error)?;
    // No authored code runs between preparation and publication. The Stage
    // also checks the captured record contents before accepting each update.
    if let Some(update) = a_update {
        with_stage(left, |stage, mob| {
            stage
                .apply_surface_grid_update(mob, update)
                .map_err(native_error)
        })??;
    }
    if let Some(update) = b_update {
        with_stage(right, |stage, mob| {
            stage
                .apply_surface_grid_update(mob, update)
                .map_err(native_error)
        })??;
    }
    Ok(())
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    initialization::install(module)?;
    module.add_function(wrap_pyfunction!(_surface_grid_resolution, module)?)?;
    module.add_function(wrap_pyfunction!(_copy_surface_geometry, module)?)?;
    module.add_function(wrap_pyfunction!(_regrid_surface_geometry, module)?)?;
    module.add_function(wrap_pyfunction!(_align_surface_grids, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_surface_regridding_and_native_output() {
        crate::with_python_test_module("live surface regridding", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/surface_regridding.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_surface_regridding_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn fallible_native_surface_sampling() {
        crate::with_python_test_module("fallible surface sampling", |py, _module, globals| {
            let code = std::ffi::CString::new(include_str!("../tests/native_surface_sampling.py"))
                .unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_surface_sampling")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn native_surface_alignment_and_morph_output() {
        crate::with_python_test_module("UV surface alignment", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/surface_alignment.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_surface_alignment_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn live_surface_wireframes_and_native_output() {
        crate::with_python_test_module("live surface wireframe", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/live_surface_mesh.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_live_surface_mesh_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn live_surface_regeneration_and_native_output() {
        crate::with_python_test_module("live surface geometry", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/surface_geometry.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_surface_geometry_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
