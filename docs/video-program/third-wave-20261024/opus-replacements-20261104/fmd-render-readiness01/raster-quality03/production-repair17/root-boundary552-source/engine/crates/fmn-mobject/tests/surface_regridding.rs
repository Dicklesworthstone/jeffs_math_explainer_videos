//! Actual Stage regridding and sampled-geometry publication, without mocks.
use fmn_mobject::stage::SurfaceGridUpdate;
use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, RenderPrimitive, Stage};

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
                for (a, b) in entry.buffer.read(row, field).unwrap().iter().zip(expected) {
                    assert!((a - b).abs() < 2e-6, "row={row} field={field}: {a} != {b}");
                }
            }
        }
    }
}

#[test]
fn explicit_regridding_refines_coarsens_and_reshapes_equal_record_counts() {
    let mut stage = Stage::new();
    let mob = grid(&mut stage, (2, 3));
    for shape in [(7, 5), (3, 4), (4, 3), (2, 2)] {
        stage.resample_surface_grid(mob, shape).unwrap();
        assert_grid(&stage, mob, shape);
    }
    let entry = stage.get_mut(mob).unwrap();
    let view = entry.buffer.export_view(true);
    let revision = entry.buffer.revision();
    stage.resample_surface_grid(mob, (2, 2)).unwrap();
    assert_eq!(stage.get(mob).unwrap().buffer.revision(), revision);
    assert!(view.is_attached_to(&stage.get(mob).unwrap().buffer));
}

#[test]
fn geometry_rebuild_preserves_paint_uvs_custom_fields_and_native_owners() {
    let mut stage = Stage::new();
    let mob = grid(&mut stage, (2, 3));
    let child = stage.add(Mobject::new());
    stage.attach(mob, child).unwrap();
    stage.add_to_scene(mob).unwrap();
    let updater = stage.add_updater(mob, |_, _| {}, false).unwrap();
    let saved = stage.save_state(mob).unwrap();
    stage.shift(mob, [4.0, -2.0, 3.0]);
    let old_view = stage.get_mut(mob).unwrap().buffer.export_view(true);
    let shape = (4, 5);
    let points = vec![2.0; 3 * shape.0 * shape.1];
    let normals = vec![3.0; points.len()];
    let update =
        SurfaceGridUpdate::for_geometry(stage.get(mob).unwrap(), shape, &points, &normals).unwrap();
    assert_eq!(stage.get(mob).unwrap().buffer.len(), 6);
    stage.apply_surface_grid_update(mob, update).unwrap();
    let entry = stage.get(mob).unwrap();
    assert!(entry.placement().is_identity());
    assert_eq!(
        entry.render_primitive(),
        RenderPrimitive::SurfaceGrid { resolution: shape }
    );
    assert_eq!(entry.buffer.read_column("point").unwrap(), points);
    assert_eq!(entry.buffer.read_column("d_normal_point").unwrap(), normals);
    assert_eq!(entry.submobjects(), &[child]);
    assert_eq!(stage.roots(), &[mob]);
    assert_eq!(stage.saved_state(mob), Some(saved));
    assert_eq!(stage.updater_ids(mob), vec![updater]);
    assert!(!old_view.is_attached_to(&entry.buffer));
    for u in 0..shape.0 {
        for v in 0..shape.1 {
            let x = u as f32 / (shape.0 - 1) as f32;
            let y = v as f32 / (shape.1 - 1) as f32;
            let row = u * shape.1 + v;
            assert_eq!(entry.buffer.read(row, "im_coords").unwrap(), [y, x]);
            assert_eq!(entry.buffer.read(row, "rgba").unwrap(), [x, y, 0.5, 1.0]);
            assert!((entry.buffer.read(row, "custom").unwrap()[0] - (x + 2.0 * y)).abs() < 2e-6);
        }
    }
}

#[test]
fn equal_shape_geometry_rebuild_keeps_live_views_and_frozen_snapshots() {
    let mut stage = Stage::new();
    let mob = grid(&mut stage, (2, 3));
    let view = stage.get_mut(mob).unwrap().buffer.export_view(true);
    let snapshot = stage.snapshot();
    let update =
        SurfaceGridUpdate::for_geometry(stage.get(mob).unwrap(), (2, 3), &[2.0; 18], &[3.0; 18])
            .unwrap();
    stage.apply_surface_grid_update(mob, update).unwrap();
    assert!(view.is_attached_to(&stage.get(mob).unwrap().buffer));
    assert_eq!(view.read(0, "point").unwrap(), [2.0; 3]);
    assert_grid(&snapshot.materialize(), mob, (2, 3));
}

#[test]
fn prepared_geometry_refuses_subsequent_retained_placement_or_record_edits() {
    for edit_placement in [false, true] {
        let mut stage = Stage::new();
        let mob = grid(&mut stage, (2, 3));
        let update = SurfaceGridUpdate::for_geometry(
            stage.get(mob).unwrap(),
            (3, 4),
            &[2.0; 36],
            &[3.0; 36],
        )
        .unwrap();
        if edit_placement {
            stage.shift(mob, [1.0, 0.0, 0.0]);
        } else {
            stage
                .get_mut(mob)
                .unwrap()
                .buffer
                .write(0, "rgba", &[0.0; 4]);
        }
        assert!(stage.apply_surface_grid_update(mob, update).is_err());
        assert_eq!(stage.get(mob).unwrap().buffer.len(), 6);
        if edit_placement {
            assert_eq!(stage.placement(mob).unwrap().translation(), [1.0, 0.0, 0.0]);
        } else {
            assert_eq!(
                stage.get(mob).unwrap().buffer.read(0, "rgba").unwrap(),
                [0.0; 4]
            );
        }
    }
}

#[test]
fn geometry_admission_refuses_invalid_lengths_samples_and_budgets_before_mutation() {
    let mut stage = Stage::new();
    let mob = grid(&mut stage, (2, 3));
    for (shape, points, normals) in [
        ((2, 3), vec![0.0; 17], vec![0.0; 18]),
        ((2, 3), vec![0.0; 18], vec![0.0; 19]),
        ((2, 3), vec![f32::NAN; 18], vec![0.0; 18]),
        ((2, 3), vec![0.0; 18], vec![f32::INFINITY; 18]),
        ((1, 3), vec![0.0; 9], vec![0.0; 9]),
        ((usize::MAX, 2), vec![], vec![]),
        ((257, 257), vec![], vec![]),
    ] {
        assert!(
            SurfaceGridUpdate::for_geometry(stage.get(mob).unwrap(), shape, &points, &normals)
                .is_err()
        );
        assert_grid(&stage, mob, (2, 3));
    }
}

#[test]
fn geometry_rebuild_bakes_auxiliary_pointlikes_but_never_color_or_texture_lanes() {
    let mut stage = Stage::new();
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("hint", 3), ("rgb", 3)],
        &["point"],
        &["point", "d_normal_point", "hint"],
    )
    .unwrap();
    let mut buffer = RecordBuffer::new(schema, 4).unwrap();
    for row in 0..4 {
        buffer.write(row, "hint", &[1.0, 2.0, 3.0]);
        buffer.write(row, "rgb", &[0.25, 0.5, 1.0]);
    }
    let mob = stage.add(
        Mobject::from_buffer(buffer)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: (2, 2) }),
    );
    stage.shift(mob, [4.0, -1.0, 2.0]);
    let update =
        SurfaceGridUpdate::for_geometry(stage.get(mob).unwrap(), (3, 3), &[0.0; 27], &[0.1; 27])
            .unwrap();
    stage.apply_surface_grid_update(mob, update).unwrap();
    for row in 0..9 {
        assert_eq!(
            stage.get(mob).unwrap().buffer.read(row, "hint").unwrap(),
            [5.0, 1.0, 5.0]
        );
        assert_eq!(
            stage.get(mob).unwrap().buffer.read(row, "rgb").unwrap(),
            [0.25, 0.5, 1.0]
        );
    }
}

#[test]
fn auxiliary_world_overflow_refuses_geometry_rebuild_without_resetting_placement() {
    let mut stage = Stage::new();
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("hint", 3)],
        &["point"],
        &["point", "d_normal_point", "hint"],
    )
    .unwrap();
    let buffer = RecordBuffer::new(schema, 4).unwrap();
    let mob = stage.add(
        Mobject::from_buffer(buffer)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: (2, 2) }),
    );
    stage
        .set_placement(mob, Placement::from_translation([f64::MAX, 0.0, 0.0]))
        .unwrap();
    let placement = stage.placement(mob);
    assert!(
        SurfaceGridUpdate::for_geometry(stage.get(mob).unwrap(), (3, 3), &[0.0; 27], &[0.1; 27])
            .is_err()
    );
    assert_eq!(stage.placement(mob), placement);
    assert_eq!(stage.get(mob).unwrap().buffer.len(), 4);
}
