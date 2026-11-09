//! Native Transform keeps primary and auxiliary points in one coordinate space.
use fmn_anim::{AnimConfig, Animation, PathFunc, RateFunc, Transform};
use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, Stage};

fn geometry(stage: &mut Stage, primary: bool) -> Mob {
    let fields = if primary {
        vec![
            ("point", 3),
            ("d_normal_point", 3),
            ("anchor", 3),
            ("rgba", 4),
        ]
    } else {
        vec![("d_normal_point", 3), ("anchor", 3), ("rgba", 4)]
    };
    let pointlike = if primary {
        vec!["point", "d_normal_point", "anchor"]
    } else {
        vec!["d_normal_point", "anchor"]
    };
    let mut records =
        RecordBuffer::new(RecordSchema::new(&fields, &[], &pointlike).unwrap(), 2).unwrap();
    for (row, x) in [0.0_f32, 2.0].into_iter().enumerate() {
        if primary {
            records.write(row, "point", &[x, 0.0, 0.0]);
        }
        records.write(row, "d_normal_point", &[x, 0.0, 1.0]);
        records.write(row, "anchor", &[x, 1.0, 0.0]);
        records.write(row, "rgba", &[0.2, 0.4, 0.6, 1.0]);
    }
    stage.add(Mobject::from_buffer(records))
}

fn animation(source: Mob, target: Mob) -> Transform {
    Transform::new(source, target).with_config(AnimConfig {
        rate_func: RateFunc::linear(),
        ..AnimConfig::default()
    })
}

fn world(stage: &Stage, mob: Mob, field: &str) -> Vec<[f64; 3]> {
    let entry = stage.get(mob).unwrap();
    entry
        .buffer
        .read_column(field)
        .unwrap()
        .as_chunks::<3>()
        .0
        .iter()
        .map(|p| entry.placement().apply_point(p.map(f64::from)))
        .collect()
}

fn assert_column(stage: &Stage, mob: Mob, field: &str, expected: &[[f64; 3]]) {
    for (actual, expected) in world(stage, mob, field).iter().zip(expected) {
        for (actual, expected) in actual.iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-5,
                "{field}: actual {actual}, expected {expected}"
            );
        }
    }
}

#[test]
fn locked_normal_seeds_follow_primary_vertices_in_live_records() {
    let mut stage = Stage::new();
    let source = geometry(&mut stage, true);
    let target = stage.copy_family(source).unwrap();
    stage.shift(target, [8.0, -4.0, 2.0]);
    let view = stage.get_mut(source).unwrap().buffer.export_view(true);
    let mut anim = animation(source, target);
    anim.begin(&mut stage).unwrap();
    assert!(
        stage
            .get(source)
            .unwrap()
            .buffer
            .is_locked("d_normal_point")
    );
    for alpha in [0.25, 0.75, 1.0, 0.5] {
        anim.interpolate(&mut stage, alpha);
        assert_column(
            &stage,
            source,
            "d_normal_point",
            &[
                [8.0 * alpha, -4.0 * alpha, 1.0 + 2.0 * alpha],
                [2.0 + 8.0 * alpha, -4.0 * alpha, 1.0 + 2.0 * alpha],
            ],
        );
        assert_eq!(
            f64::from(view.read(0, "d_normal_point").unwrap()[2]),
            1.0 + 2.0 * alpha
        );
        assert_eq!(stage.placement(source), Some(Placement::IDENTITY));
        assert_eq!(view.read(0, "rgba").unwrap(), [0.2, 0.4, 0.6, 1.0]);
    }
    anim.finish(&mut stage);
}

#[test]
fn reading_an_intermediate_frame_never_compounds_normal_placement() {
    let mut stage = Stage::new();
    let source = geometry(&mut stage, true);
    let target = stage.copy_family(source).unwrap();
    stage.shift(target, [8.0, 0.0, 0.0]);
    let mut anim = animation(source, target);
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.25);
    assert!(stage.bake_placement(source).unwrap());
    anim.interpolate(&mut stage, 0.75);
    assert_column(&stage, source, "point", &[[6.0, 0.0, 0.0], [8.0, 0.0, 0.0]]);
    assert_column(
        &stage,
        source,
        "d_normal_point",
        &[[6.0, 0.0, 1.0], [8.0, 0.0, 1.0]],
    );
    assert_column(
        &stage,
        source,
        "anchor",
        &[[6.0, 1.0, 0.0], [8.0, 1.0, 0.0]],
    );
    anim.finish(&mut stage);
}

#[test]
fn rotated_normals_follow_the_same_world_endpoint_path_as_vertices() {
    let mut stage = Stage::new();
    let source = geometry(&mut stage, true);
    let target = stage.copy_family(source).unwrap();
    stage.rotate(
        target,
        std::f64::consts::FRAC_PI_2,
        [1.0, 0.0, 0.0],
        Some([0.0; 3]),
        None,
    );
    stage.shift(target, [3.0, 1.0, 2.0]);
    let from = world(&stage, source, "d_normal_point");
    let to = world(&stage, target, "d_normal_point");
    let view = stage.get_mut(source).unwrap().buffer.export_view(false);
    let mut anim = animation(source, target);
    anim.begin(&mut stage).unwrap();
    for alpha in [0.0, 0.25, 0.5, 1.0] {
        anim.interpolate(&mut stage, alpha);
        let expected: Vec<_> = from
            .iter()
            .zip(&to)
            .map(|(&a, &b)| PathFunc::Straight.eval(a, b, alpha))
            .collect();
        assert_column(&stage, source, "d_normal_point", &expected);
        assert!(view.is_attached_to(&stage.get(source).unwrap().buffer));
    }
    anim.finish(&mut stage);
}

#[test]
fn differing_normal_geometry_is_baked_before_placement_interpolation() {
    let mut stage = Stage::new();
    let source = geometry(&mut stage, true);
    let target = stage.copy_family(source).unwrap();
    stage
        .get_mut(target)
        .unwrap()
        .buffer
        .write(0, "d_normal_point", &[0.0, 0.0, 2.0]);
    stage.scale_about(target, 2.0, Some([0.0; 3]), None);
    let original_target = world(&stage, target, "d_normal_point");
    let mut anim = animation(source, target);
    anim.begin(&mut stage).unwrap();
    assert_ne!(
        anim.target_copy(),
        Some(target),
        "baking must not mutate the authored target"
    );
    anim.interpolate(&mut stage, 0.5);
    assert_column(
        &stage,
        source,
        "d_normal_point",
        &[[0.0, 0.0, 2.5], [3.0, 0.0, 1.5]],
    );
    assert_eq!(world(&stage, target, "d_normal_point"), original_target);
    anim.finish(&mut stage);
}

#[test]
fn auxiliary_only_custom_schemas_have_the_same_live_view_contract() {
    let mut stage = Stage::new();
    let source = geometry(&mut stage, false);
    let target = stage.copy_family(source).unwrap();
    stage.shift(target, [4.0, 2.0, 0.0]);
    let from = stage.copy_family(source).unwrap();
    let view = stage.get_mut(source).unwrap().buffer.export_view(true);
    // The public field lerp supports records without the primary point field;
    // Animation's family eligibility remains its existing separate contract.
    fmn_anim::transform::interpolate_fields(
        &mut stage,
        source,
        from,
        target,
        0.5,
        PathFunc::Straight,
    );
    assert_eq!(view.read(0, "d_normal_point").unwrap(), [2.0, 1.0, 1.0]);
    assert_eq!(view.read(0, "anchor").unwrap(), [2.0, 2.0, 0.0]);
    assert_eq!(stage.placement(source), Some(Placement::IDENTITY));
}
