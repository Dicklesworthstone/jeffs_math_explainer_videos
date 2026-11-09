//! UV-aware record alignment for sampled surfaces (§12.4).
//!
//! Resolution is topology, not merely a flat record count. Resampling uses
//! normalized UV coordinates and bilinear interpolation of every record lane,
//! including normal control points, paint and texture coordinates. No authored
//! sampler runs during alignment. This module is a child of `stage` so it can
//! publish a buffer and its durable topology together without replacing identity.

use super::{Entry, Mob, Placement, Stage};
use crate::{RecordBuffer, RenderPrimitive, StageError};

/// Maximum number of records in an aligned UV surface.
///
/// Surface grids have a separate resource contract from one-dimensional
/// curve and field sampling: a 301-by-301 chart has 90,601 vertices. This
/// ceiling admits such charts, while bounding the component-wise maximum
/// used by alignment as well as explicit regridding (fm-1rb8).
pub const MAX_SURFACE_GRID_POINTS: usize = 262_144;

/// A validated, prepared surface update. Preparation never changes a live entry.
/// Alignment preserves placement; geometry replacement bakes it. Materials,
/// family, updaters and saved state retain their existing owners in both cases.
pub struct SurfaceGridUpdate {
    before: RecordBuffer,
    before_placement: Placement,
    old_resolution: (usize, usize),
    resolution: (usize, usize),
    buffer: RecordBuffer,
    placement: Placement,
}

fn grid_count(resolution: (usize, usize)) -> Result<usize, StageError> {
    let (u, v) = resolution;
    if u < 2 || v < 2 {
        return Err(StageError::SurfaceGrid(
            "both UV dimensions must be at least two",
        ));
    }
    u.checked_mul(v)
        .filter(|&count| count <= MAX_SURFACE_GRID_POINTS)
        .ok_or(StageError::SurfaceGrid(
            "UV grid exceeds the 262144-point budget",
        ))
}

fn resolution(entry: &Entry) -> Result<(usize, usize), StageError> {
    let RenderPrimitive::SurfaceGrid { resolution } = entry.render_primitive() else {
        return Err(StageError::SurfaceGrid(
            "alignment requires two UV-grid surfaces",
        ));
    };
    if grid_count(resolution)? != entry.buffer.len() {
        return Err(StageError::SurfaceGrid(
            "UV topology does not match the record count",
        ));
    }
    for key in ["point", "d_normal_point"] {
        if entry.buffer.schema().field_width(key) != Some(3) {
            return Err(StageError::SurfaceGrid(
                "surface point and normal-control fields must be vec3",
            ));
        }
    }
    Ok(resolution)
}

// Exact integer stations preserve existing knots and endpoints. A legal
// skinny grid can have 131,072 stations along one axis, so multiplying two
// axis indices in usize would overflow on wasm32. Widen BEFORE multiplying;
// the grid budget bounds this product below 2^36, exactly representable in
// both u64 and f64. The resulting indices still fit either pointer width.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn station(index: usize, old: usize, new: usize) -> (usize, usize, f64) {
    let numerator = index as u64 * (old - 1) as u64;
    let denominator = (new - 1) as u64;
    let low = (numerator / denominator) as usize;
    (
        low,
        (low + 1).min(old - 1),
        (numerator % denominator) as f64 / denominator as f64,
    )
}

fn lerp(a: f64, b: f64, alpha: f64) -> f64 {
    if alpha == 0.0 || a == b {
        a
    } else if alpha == 1.0 {
        b
    } else {
        (1.0 - alpha) * a + alpha * b
    }
}

fn prepare(entry: &Entry, shape: (usize, usize)) -> Result<Option<SurfaceGridUpdate>, StageError> {
    let old = resolution(entry)?;
    let count = grid_count(shape)?;
    // Validate even equal-topology pairs, before the peer can be changed.
    let columns: Vec<_> = entry
        .buffer
        .schema()
        .fields()
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                field.width,
                entry.buffer.read_column(&field.name).unwrap_or_default(),
            )
        })
        .collect();
    if columns
        .iter()
        .any(|(_, _, data)| data.iter().any(|v| !v.is_finite()))
    {
        return Err(StageError::SurfaceGrid(
            "surface records must be finite before alignment",
        ));
    }
    if shape == old {
        return Ok(None);
    }
    let mut buffer =
        RecordBuffer::new(entry.buffer.schema().clone(), count).map_err(StageError::Record)?;
    for (name, width, data) in columns {
        let mut values = Vec::with_capacity(count * width);
        for u in 0..shape.0 {
            let (u0, u1, fu) = station(u, old.0, shape.0);
            for v in 0..shape.1 {
                let (v0, v1, fv) = station(v, old.1, shape.1);
                for lane in 0..width {
                    let sample = |ui, vi| f64::from(data[(ui * old.1 + vi) * width + lane]);
                    let a = lerp(sample(u0, v0), sample(u0, v1), fv);
                    let b = lerp(sample(u1, v0), sample(u1, v1), fv);
                    #[allow(clippy::cast_possible_truncation)]
                    let value = lerp(a, b, fu) as f32;
                    if !value.is_finite() {
                        return Err(StageError::SurfaceGrid(
                            "resampled surface records must be finite",
                        ));
                    }
                    values.push(value);
                }
            }
        }
        buffer.write_range(&name, 0, &values);
    }
    Ok(Some(SurfaceGridUpdate {
        before: entry.buffer.snapshot_clone(),
        before_placement: entry.placement(),
        old_resolution: old,
        resolution: shape,
        buffer,
        placement: entry.placement(),
    }))
}

/// Prepare a new geometric sample of an existing surface, at any admitted UV
/// resolution. Only positions and normal-control points come from the sampler;
/// all other record fields retain their normalized-UV interpolants. Materials,
/// uniforms and graph ownership are not part of this operation.
///
/// Input points are world-space. Auxiliary pointlike fields are baked through
/// the old placement before that placement is reset, matching geometry writes.
/// No entry is mutated until [`Stage::apply_surface_grid_update`] succeeds.
///
/// # Errors
/// Rejects malformed grids, nonfinite records, unrepresentable world-space
/// auxiliary points and inconsistent sample lengths before publication.
fn prepare_surface_grid_geometry(
    entry: &Entry,
    shape: (usize, usize),
    points: &[f32],
    normal_points: &[f32],
) -> Result<SurfaceGridUpdate, StageError> {
    let count = grid_count(shape)?;
    if points.len() != 3 * count || normal_points.len() != 3 * count {
        return Err(StageError::SurfaceGrid(
            "surface sample lengths do not match the UV grid",
        ));
    }
    if points
        .iter()
        .chain(normal_points)
        .any(|value| !value.is_finite())
    {
        return Err(StageError::SurfaceGrid(
            "surface geometry samples must be finite",
        ));
    }
    let mut update = match prepare(entry, shape)? {
        Some(update) => update,
        None => SurfaceGridUpdate {
            before: entry.buffer.snapshot_clone(),
            before_placement: entry.placement(),
            old_resolution: shape,
            resolution: shape,
            buffer: entry.buffer.deep_clone(),
            placement: entry.placement(),
        },
    };
    // The geometry sampler replaces these two fields. Every other declared
    // vec3 pointlike lane still belongs to the destination and must stay in
    // world space when its retained placement is discarded.
    if !update.placement.is_identity() {
        let keys = update.buffer.schema().pointlike_keys().to_vec();
        for key in keys {
            if key == "point"
                || key == "d_normal_point"
                || update.buffer.schema().field_width(&key) != Some(3)
            {
                continue;
            }
            let mut column = update.buffer.read_column(&key).unwrap_or_default();
            for point in column.as_chunks_mut::<3>().0 {
                let world = update.placement.apply_point(point.map(f64::from));
                if world
                    .iter()
                    .any(|value| !value.is_finite() || value.abs() > f64::from(f32::MAX))
                {
                    return Err(StageError::SurfaceGrid(
                        "surface auxiliary points exceed finite f32 storage",
                    ));
                }
                #[allow(clippy::cast_possible_truncation)]
                {
                    *point = world.map(|value| value as f32);
                }
            }
            update.buffer.write_range(&key, 0, &column);
        }
    }
    update.buffer.write_range("point", 0, points);
    update
        .buffer
        .write_range("d_normal_point", 0, normal_points);
    update.placement = Placement::IDENTITY;
    Ok(update)
}

impl SurfaceGridUpdate {
    /// Prepare an atomic geometry replacement, resampling all other record
    /// fields onto `shape` and baking auxiliary pointlike fields to world space.
    /// The input point and normal-control columns are already world-space.
    ///
    /// # Errors
    /// Invalid dimensions, schemas, numeric values or sample lengths refuse
    /// before any live records, placement or topology can be changed.
    pub fn for_geometry(
        entry: &Entry,
        shape: (usize, usize),
        points: &[f32],
        normal_points: &[f32],
    ) -> Result<Self, StageError> {
        prepare_surface_grid_geometry(entry, shape, points, normal_points)
    }
}

/// Prepare both sides on the component-wise maximum UV resolution. Neither
/// side is published if either topology, schema, numeric data or budget fails.
/// Equal record counts with different UV shapes still require alignment.
pub fn prepare_surface_grid_alignment(
    left: &Entry,
    right: &Entry,
) -> Result<(Option<SurfaceGridUpdate>, Option<SurfaceGridUpdate>), StageError> {
    let a = resolution(left)?;
    let b = resolution(right)?;
    if left.buffer.schema() != right.buffer.schema() {
        return Err(StageError::SchemaMismatch);
    }
    let shape = (a.0.max(b.0), a.1.max(b.1));
    grid_count(shape)?;
    Ok((prepare(left, shape)?, prepare(right, shape)?))
}

impl Stage {
    /// Publish prepared records, placement and UV topology together. Preserve
    /// live views when record count is unchanged; resized generations detach.
    /// Refuse stale preparation rather than overwrite later record/placement edits.
    pub fn apply_surface_grid_update(
        &mut self,
        mob: Mob,
        update: SurfaceGridUpdate,
    ) -> Result<(), StageError> {
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        if resolution(entry)? != update.old_resolution
            || !entry.placement().same_bits(update.before_placement)
            || entry.buffer.schema() != update.before.schema()
            || entry
                .buffer
                .schema()
                .fields()
                .iter()
                .any(|field| !entry.buffer.column_eq(&update.before, &field.name))
        {
            return Err(StageError::SurfaceGrid(
                "surface changed after alignment was prepared",
            ));
        }
        if entry.buffer.len() == update.buffer.len() {
            // set_data/become intentionally detaches even equal-size views.
            // Regridding is an in-place edit instead: prepare all field copies
            // before writing, then use the existing snapshot-aware write path.
            let columns: Vec<_> = update
                .buffer
                .schema()
                .fields()
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        update.buffer.read_column(&field.name).unwrap_or_default(),
                    )
                })
                .collect();
            for (name, values) in columns {
                let written = entry.buffer.write_range(&name, 0, &values);
                debug_assert!(written, "prepared surface column has the validated layout");
            }
        } else if !entry.buffer.assign_from(&update.buffer) {
            return Err(StageError::SchemaMismatch);
        }
        entry.render_primitive = RenderPrimitive::SurfaceGrid {
            resolution: update.resolution,
        };
        entry.set_placement(update.placement);
        Ok(())
    }

    /// Regrid stored surface records without evaluating a geometry recipe.
    /// Supports refinement, coarsening and equal-count UV reshaping with the
    /// same bilinear record owner used by Transform alignment. Placement and
    /// every non-record owner are retained; equal shapes are true no-ops.
    ///
    /// # Errors
    /// A stale handle, invalid grid or nonfinite record refuses before mutation.
    pub fn resample_surface_grid(
        &mut self,
        mob: Mob,
        shape: (usize, usize),
    ) -> Result<(), StageError> {
        if let Some(update) = prepare(self.try_get(mob)?, shape)? {
            self.apply_surface_grid_update(mob, update)?;
        }
        Ok(())
    }

    /// Align only one pair of sampled UV entries. This is the same owner used
    /// by native Transform and by the optional host-Python portal.
    pub fn align_surface_points(&mut self, a: Mob, b: Mob) -> Result<(), StageError> {
        let (left, right) = prepare_surface_grid_alignment(self.try_get(a)?, self.try_get(b)?)?;
        if let Some(update) = left {
            self.apply_surface_grid_update(a, update)?;
        }
        if let Some(update) = right {
            self.apply_surface_grid_update(b, update)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod dense_grid_tests {
    use super::*;
    use crate::{Mobject, RecordSchema};

    fn surface(shape: (usize, usize)) -> Mobject {
        let schema = RecordSchema::new(
            &[("point", 3), ("d_normal_point", 3), ("temperature", 1)],
            &["point"],
            &["point", "d_normal_point"],
        )
        .expect("surface schema");
        let mut buffer = RecordBuffer::new(schema, shape.0 * shape.1).expect("bounded grid");
        for u in 0..shape.0 {
            for v in 0..shape.1 {
                #[allow(clippy::cast_precision_loss)]
                let (x, y) = (
                    u as f32 / (shape.0 - 1) as f32,
                    v as f32 / (shape.1 - 1) as f32,
                );
                let row = u * shape.1 + v;
                buffer.write(row, "point", &[x, y, x + y]);
                buffer.write(row, "d_normal_point", &[x, y, x + y + 1.0]);
                buffer.write(row, "temperature", &[x + 2.0 * y]);
            }
        }
        Mobject::from_buffer(buffer)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: shape })
    }

    #[test]
    fn dense_grid_limits_are_checked_before_resampling() {
        assert_eq!(grid_count((301, 301)), Ok(90_601));
        assert_eq!(grid_count((512, 512)), Ok(MAX_SURFACE_GRID_POINTS));
        assert_eq!(grid_count((2, 131_072)), Ok(MAX_SURFACE_GRID_POINTS));
        for shape in [(513, 512), (2, 131_073), (usize::MAX, 2), (2, usize::MAX)] {
            assert!(grid_count(shape).is_err(), "{shape:?}");
        }
        // Construction may admit empty/strip grids, but alignment still
        // requires actual two-dimensional topology.
        for shape in [(0, 301), (301, 0), (1, 301), (301, 1)] {
            assert!(grid_count(shape).is_err(), "{shape:?}");
        }
    }

    #[test]
    fn skinny_grid_stations_do_not_depend_on_pointer_width() {
        assert!(131_070_u64 * 131_071 > u64::from(u32::MAX));
        assert_eq!(station(131_070, 131_072, 131_071), (131_071, 131_071, 0.0));
        assert_eq!(station(65_535, 131_072, 131_071), (65_535, 65_536, 0.5));
        assert_eq!(station(0, 131_072, 131_071), (0, 1, 0.0));
        assert_eq!(station(1, 131_072, 131_071), (1, 2, 1.0 / 131_070.0));
    }

    #[test]
    fn widened_stations_preserve_all_previously_admitted_bits() {
        for old in [2, 3, 101, 256, 32_768] {
            for new in [2, 3, 101, 256, 32_768] {
                for index in 0..new {
                    let numerator = index * (old - 1);
                    let low = numerator / (new - 1);
                    #[allow(clippy::cast_precision_loss)]
                    let fraction = (numerator % (new - 1)) as f64 / (new - 1) as f64;
                    let actual = station(index, old, new);
                    assert_eq!((actual.0, actual.1), (low, (low + 1).min(old - 1)));
                    assert_eq!(actual.2.to_bits(), fraction.to_bits());
                }
            }
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn dense_alignment_interpolates_geometry_normals_and_custom_records() {
        let mut stage = Stage::new();
        let coarse = stage.add(surface((2, 2)));
        let dense = stage.add(surface((301, 301)));
        let before = stage.get(dense).unwrap().buffer.snapshot_clone();
        stage
            .align_surface_points(coarse, dense)
            .expect("dense alignment");
        let aligned = stage.get(coarse).unwrap();
        assert_eq!(resolution(aligned).unwrap(), (301, 301));
        let points = aligned.buffer.read_column("point").unwrap();
        let normals = aligned.buffer.read_column("d_normal_point").unwrap();
        let temperature = aligned.buffer.read_column("temperature").unwrap();
        assert_eq!(temperature.len(), 90_601);
        for u in 0..301 {
            for v in 0..301 {
                let row = u * 301 + v;
                let (x, y) = (u as f64 / 300.0, v as f64 / 300.0);
                for (actual, expected) in points[3 * row..3 * row + 3].iter().zip([x, y, x + y]) {
                    assert!((f64::from(*actual) - expected).abs() < 1e-6);
                }
                assert!((f64::from(normals[3 * row + 2]) - (x + y + 1.0)).abs() < 1e-6);
                assert!((f64::from(temperature[row]) - (x + 2.0 * y)).abs() < 1e-6);
            }
        }
        for key in ["point", "d_normal_point", "temperature"] {
            assert!(stage.get(dense).unwrap().buffer.column_eq(&before, key));
        }
        stage
            .resample_surface_grid(coarse, (2, 2))
            .expect("coarsen");
        let original = stage.add(surface((2, 2)));
        for key in ["point", "d_normal_point", "temperature"] {
            assert!(
                stage
                    .get(coarse)
                    .unwrap()
                    .buffer
                    .column_eq(&stage.get(original).unwrap().buffer, key)
            );
        }
    }

    #[test]
    fn individually_legal_charts_cannot_bypass_the_alignment_product_limit() {
        let mut stage = Stage::new();
        let a = stage.add(surface((512, 2)));
        let b = stage.add(surface((2, 513)));
        let before_a = stage.get(a).unwrap().buffer.snapshot_clone();
        let before_b = stage.get(b).unwrap().buffer.snapshot_clone();
        assert!(stage.align_surface_points(a, b).is_err());
        assert_eq!(resolution(stage.get(a).unwrap()).unwrap(), (512, 2));
        assert_eq!(resolution(stage.get(b).unwrap()).unwrap(), (2, 513));
        for key in ["point", "d_normal_point", "temperature"] {
            assert!(stage.get(a).unwrap().buffer.column_eq(&before_a, key));
            assert!(stage.get(b).unwrap().buffer.column_eq(&before_b, key));
        }
    }

    #[test]
    fn maximum_skinny_chart_refines_without_losing_its_last_knot() {
        let mut stage = Stage::new();
        let mob = stage.add(surface((131_071, 2)));
        stage
            .resample_surface_grid(mob, (131_072, 2))
            .expect("skinny refinement");
        let entry = stage.get(mob).unwrap();
        assert_eq!(entry.buffer.len(), MAX_SURFACE_GRID_POINTS);
        let points = entry.buffer.read_column("point").unwrap();
        assert_eq!(&points[..3], &[0.0, 0.0, 0.0]);
        assert_eq!(&points[points.len() - 3..], &[1.0, 1.0, 2.0]);
    }
}
