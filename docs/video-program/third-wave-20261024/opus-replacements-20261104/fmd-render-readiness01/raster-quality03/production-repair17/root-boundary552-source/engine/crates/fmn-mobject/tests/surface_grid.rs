//! Real Marionette UV alignment, including ownership and refusal paths.
use fmn_mobject::stage::prepare_surface_grid_alignment;
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema, RenderPrimitive, Stage, StageError};

fn grid(stage: &mut Stage, shape: (usize, usize)) -> Mob {
    let schema = RecordSchema::new(
        &[
            ("point", 3),
            ("d_normal_point", 3),
            ("rgba", 4),
            ("im_coords", 2),
            ("custom", 1),
        ],
        &["point"],
        &["point", "d_normal_point"],
    )
    .unwrap();
    let mut buffer = RecordBuffer::new(schema, shape.0 * shape.1).unwrap();
    for u in 0..shape.0 {
        for v in 0..shape.1 {
            let x = u as f32 / (shape.0 - 1) as f32;
            let y = v as f32 / (shape.1 - 1) as f32;
            let row = u * shape.1 + v;
            buffer.write(row, "point", &[x, y, x * y]);
            buffer.write(row, "d_normal_point", &[x, y, x * y + 0.25]);
            buffer.write(row, "rgba", &[x, y, 0.5, 1.0]);
            buffer.write(row, "im_coords", &[y, x]);
            buffer.write(row, "custom", &[x + 2.0 * y]);
        }
    }
    stage.add(
        Mobject::from_buffer(buffer)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: shape }),
    )
}

fn assert_grid(stage: &Stage, mob: Mob, shape: (usize, usize)) {
    let entry = stage.get(mob).unwrap();
    assert_eq!(
        entry.render_primitive(),
        RenderPrimitive::SurfaceGrid { resolution: shape }
    );
    assert_eq!(entry.buffer.len(), shape.0 * shape.1);
    for u in 0..shape.0 {
        for v in 0..shape.1 {
            let x = u as f32 / (shape.0 - 1) as f32;
            let y = v as f32 / (shape.1 - 1) as f32;
            let row = u * shape.1 + v;
            for (field, expected) in [
                ("point", vec![x, y, x * y]),
                ("d_normal_point", vec![x, y, x * y + 0.25]),
                ("rgba", vec![x, y, 0.5, 1.0]),
                ("im_coords", vec![y, x]),
                ("custom", vec![x + 2.0 * y]),
            ] {
                let actual = entry.buffer.read(row, field).unwrap();
                assert_eq!(actual.len(), expected.len());
                for (a, b) in actual.iter().zip(expected) {
                    assert!((a - b).abs() < 2e-6, "row={row} field={field}: {a} != {b}");
                }
            }
        }
    }
}

#[test]
fn different_resolutions_align_every_lane_on_uv_not_flat_record_indices() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 3));
    let b = grid(&mut stage, (4, 2));
    stage.align_points(a, b).unwrap();
    assert_grid(&stage, a, (4, 3));
    assert_grid(&stage, b, (4, 3));
    assert!(stage.is_aligned_with(a, b));
}

#[test]
fn equal_counts_do_not_hide_different_triangle_topology() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 3));
    let b = grid(&mut stage, (3, 2));
    assert!(!stage.is_aligned_with(a, b));
    stage.align_points(a, b).unwrap();
    assert_grid(&stage, a, (3, 3));
    assert_grid(&stage, b, (3, 3));
}

#[test]
fn alignment_is_a_noop_for_matching_grids_and_keeps_views_attached() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (3, 4));
    let b = grid(&mut stage, (3, 4));
    let view = stage.get_mut(a).unwrap().buffer.export_view(true);
    let revision = stage.get(a).unwrap().buffer.revision();
    stage.align_surface_points(a, b).unwrap();
    assert_eq!(stage.get(a).unwrap().buffer.revision(), revision);
    assert!(view.is_attached_to(&stage.get(a).unwrap().buffer));
}

#[test]
fn resize_detaches_views_but_preserves_identity_placement_children_and_saved_state() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 2));
    let b = grid(&mut stage, (3, 4));
    let child = stage.add(Mobject::new());
    stage.attach(a, child).unwrap();
    stage.shift(a, [4.0, -2.0, 3.0]);
    stage.add_to_scene(a).unwrap();
    let updater = stage.add_updater(a, |_, _| {}, false).unwrap();
    let saved = stage.save_state(a).unwrap();
    let placement = stage.placement(a);
    let view = stage.get_mut(a).unwrap().buffer.export_view(true);
    let first = view.read(0, "point").unwrap();
    stage.align_surface_points(a, b).unwrap();
    assert!(!view.is_attached_to(&stage.get(a).unwrap().buffer));
    assert_eq!(view.read(0, "point").unwrap(), first);
    assert_eq!(stage.placement(a), placement);
    assert_eq!(stage.get(a).unwrap().submobjects(), &[child]);
    assert_eq!(stage.updater_ids(a), vec![updater]);
    assert_eq!(stage.saved_state(a), Some(saved));
    assert_eq!(stage.roots(), &[a]);
    assert_grid(&stage, a, (3, 4));
}

#[test]
fn begin_state_snapshots_keep_the_original_grid_and_records() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 3));
    let b = grid(&mut stage, (4, 5));
    let snapshot = stage.snapshot();
    stage.align_surface_points(a, b).unwrap();
    assert_grid(&stage, a, (4, 5));
    assert_grid(&snapshot.materialize(), a, (2, 3));
}

#[test]
fn excessive_common_grid_refuses_before_changing_either_side() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (513, 2));
    let b = grid(&mut stage, (2, 513));
    assert!(matches!(
        stage.align_points(a, b),
        Err(StageError::SurfaceGrid(_))
    ));
    assert_grid(&stage, a, (513, 2));
    assert_grid(&stage, b, (2, 513));
}

#[test]
fn malformed_peer_topology_refuses_before_resizing_the_valid_side() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 2));
    let b = grid(&mut stage, (3, 3));
    stage.get_mut(b).unwrap().buffer.resize(8).unwrap();
    assert!(matches!(
        stage.align_points(a, b),
        Err(StageError::SurfaceGrid(_))
    ));
    assert_grid(&stage, a, (2, 2));
    assert_eq!(stage.get(b).unwrap().buffer.len(), 8);
}

#[test]
fn nonfinite_peer_refuses_before_either_grid_is_published() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 3));
    let b = grid(&mut stage, (3, 2));
    stage
        .get_mut(b)
        .unwrap()
        .buffer
        .write(0, "custom", &[f32::NAN]);
    assert!(matches!(
        stage.align_points(a, b),
        Err(StageError::SurfaceGrid(_))
    ));
    assert_grid(&stage, a, (2, 3));
    assert_eq!(stage.get(b).unwrap().buffer.len(), 6);
}

#[test]
fn prepared_alignment_does_not_overwrite_subsequent_live_record_edits() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 2));
    let b = grid(&mut stage, (3, 3));
    let (left, _) =
        prepare_surface_grid_alignment(stage.get(a).unwrap(), stage.get(b).unwrap()).unwrap();
    let view = stage.get_mut(a).unwrap().buffer.export_view(true);
    view.write(0, "custom", &[42.0]);
    assert!(matches!(
        stage.apply_surface_grid_update(a, left.unwrap()),
        Err(StageError::SurfaceGrid(_))
    ));
    assert_eq!(stage.get(a).unwrap().buffer.len(), 4);
    assert_eq!(view.read(0, "custom").unwrap(), [42.0]);
}

#[test]
fn mixed_grid_and_point_records_are_not_silently_flattened() {
    let mut stage = Stage::new();
    let a = grid(&mut stage, (2, 2));
    let other = stage.add(Mobject::from_buffer(
        stage.get(a).unwrap().buffer.deep_clone(),
    ));
    assert!(matches!(
        stage.align_points(a, other),
        Err(StageError::SurfaceGrid(_))
    ));
    assert_grid(&stage, a, (2, 2));
}
