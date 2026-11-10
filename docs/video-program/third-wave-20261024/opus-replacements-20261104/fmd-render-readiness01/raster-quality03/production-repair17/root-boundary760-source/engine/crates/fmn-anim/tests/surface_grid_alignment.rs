//! Native Transform must never treat a UV grid as a flat point cloud.
use fmn_anim::{AnimConfig, Animation, RateFunc, Transform};
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema, RenderPrimitive, Stage};

fn surface(stage: &mut Stage, shape: (usize, usize), height: f32) -> Mob {
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
        &["point"],
        &["point", "d_normal_point"],
    )
    .unwrap();
    let mut data = RecordBuffer::new(schema, shape.0 * shape.1).unwrap();
    for u in 0..shape.0 {
        for v in 0..shape.1 {
            let x = u as f32 / (shape.0 - 1) as f32;
            let y = v as f32 / (shape.1 - 1) as f32;
            let i = u * shape.1 + v;
            data.write(i, "point", &[x, y, height]);
            data.write(i, "d_normal_point", &[x, y, height + 0.25]);
            data.write(i, "rgba", &[x, y, 0.5, 1.0]);
        }
    }
    stage.add(
        Mobject::from_buffer(data)
            .with_render_primitive(RenderPrimitive::SurfaceGrid { resolution: shape }),
    )
}

fn morph(a: Mob, b: Mob) -> Transform {
    Transform::new(a, b).with_config(AnimConfig {
        rate_func: RateFunc::linear(),
        ..AnimConfig::default()
    })
}

fn check(stage: &Stage, mob: Mob, shape: (usize, usize), height: f64) {
    let entry = stage.get(mob).unwrap();
    assert_eq!(
        entry.render_primitive(),
        RenderPrimitive::SurfaceGrid { resolution: shape }
    );
    assert_eq!(entry.buffer.len(), shape.0 * shape.1);
    let points = stage.get_points(mob).unwrap();
    for u in 0..shape.0 {
        for v in 0..shape.1 {
            let i = u * shape.1 + v;
            let expected = [
                u as f64 / (shape.0 - 1) as f64,
                v as f64 / (shape.1 - 1) as f64,
                height,
            ];
            for (a, b) in points[i].iter().zip(expected) {
                assert!((a - b).abs() < 2e-6);
            }
            let normal = entry.buffer.read(i, "d_normal_point").unwrap();
            let world = entry.placement().apply_point([
                normal[0] as f64,
                normal[1] as f64,
                normal[2] as f64,
            ]);
            assert!((world[2] - height - 0.25).abs() < 2e-6);
        }
    }
}

#[test]
fn different_resolutions_have_valid_topology_at_every_intermediate_sample() {
    let mut stage = Stage::new();
    let a = surface(&mut stage, (2, 2), 0.0);
    let b = surface(&mut stage, (3, 4), 2.0);
    let mut animation = morph(a, b);
    animation.begin(&mut stage).unwrap();
    for alpha in [0.0, 0.25, 0.75, 1.0, 0.5] {
        animation.interpolate(&mut stage, alpha);
        check(&stage, a, (3, 4), 2.0 * alpha);
        check(&stage, b, (3, 4), 2.0);
    }
    animation.finish(&mut stage);
}

#[test]
fn equal_record_counts_still_copy_the_target_before_topology_alignment() {
    let mut stage = Stage::new();
    let a = surface(&mut stage, (2, 3), 0.0);
    let b = surface(&mut stage, (3, 2), 2.0);
    let mut animation = morph(a, b);
    animation.begin(&mut stage).unwrap();
    assert_ne!(animation.target_copy(), Some(b));
    animation.interpolate(&mut stage, 0.5);
    check(&stage, a, (3, 3), 1.0);
    check(&stage, b, (3, 2), 2.0);
    animation.finish(&mut stage);
}

#[test]
fn copied_begin_state_and_live_views_do_not_corrupt_the_aligned_grid() {
    let mut stage = Stage::new();
    let a = surface(&mut stage, (2, 3), 0.0);
    let b = surface(&mut stage, (4, 2), 2.0);
    let old_view = stage.get_mut(a).unwrap().buffer.export_view(true);
    let snapshot = stage.snapshot();
    let mut animation = morph(a, b);
    animation.begin(&mut stage).unwrap();
    assert!(!old_view.is_attached_to(&stage.get(a).unwrap().buffer));
    assert_eq!(old_view.read(0, "point").unwrap(), [0.0, 0.0, 0.0]);
    let live_view = stage.get_mut(a).unwrap().buffer.export_view(true);
    for alpha in [0.25, 0.5, 0.75, 1.0] {
        animation.interpolate(&mut stage, alpha);
        check(&stage, a, (4, 3), 2.0 * alpha);
        assert!((f64::from(live_view.read(0, "point").unwrap()[2]) - 2.0 * alpha).abs() < 1e-6);
    }
    check(&snapshot.materialize(), a, (2, 3), 0.0);
    animation.finish(&mut stage);
}
