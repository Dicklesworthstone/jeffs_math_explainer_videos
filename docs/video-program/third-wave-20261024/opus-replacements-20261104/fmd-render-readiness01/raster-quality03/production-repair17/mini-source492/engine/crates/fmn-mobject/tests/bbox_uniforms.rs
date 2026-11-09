//! Bounding-box invalidation properties and the uniform surface (fm-jru).
//!
//! Covers the two acceptance items the positional corpus does not: bbox
//! dirty-flag propagation (mutation through *any* channel dirties ancestors,
//! exactly once) and the uniform read/write round-trip plus the Appendix-C
//! rulings (C-2/BN-07, C-7) at the Stage level.

use fmn_core::constants::UP;
use fmn_mobject::{BoundingBox, JointType, Mob, Mobject, Stage};

fn leaf(stage: &mut Stage, pts: &[[f64; 3]]) -> Mob {
    stage.add(Mobject::from_points(pts))
}

#[test]
fn bounding_box_is_lazy() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]]);
    // First read materializes; repeated reads do not.
    for _ in 0..5 {
        let _ = stage.get_bounding_box(m);
    }
    assert_eq!(stage.bbox_materializations(m), 1);
}

#[test]
fn point_write_channel_invalidates_exactly_once() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    let _ = stage.get_bounding_box(m);
    assert_eq!(stage.bbox_materializations(m), 1);

    // Mutate through the raw RecordBuffer write channel (not the positional
    // API): the box must invalidate.
    stage
        .get_mut(m)
        .unwrap()
        .buffer
        .write(1, "point", &[3.0, 4.0, 0.0]);

    // One recompute, no matter how many times it is read afterward.
    for _ in 0..4 {
        let bb = stage.get_bounding_box(m);
        assert_eq!(bb.max, [3.0, 4.0, 0.0]);
    }
    assert_eq!(stage.bbox_materializations(m), 2);
}

#[test]
fn writable_point_views_force_boxes_but_unrelated_views_do_not() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    let _ = stage.get_bounding_box(m);
    let baseline = stage.bbox_materializations(m);

    let point_view = stage
        .get_mut(m)
        .expect("live")
        .buffer
        .export_field_view("point", true)
        .expect("point field");
    let _ = stage.get_bounding_box(m);
    let _ = stage.get_bounding_box(m);
    assert_eq!(
        stage.bbox_materializations(m),
        baseline + 2,
        "each observation must refresh while foreign point writes are possible"
    );
    drop(point_view);

    // Detach bumps the point revision once, after which normal laziness resumes.
    let _ = stage.get_bounding_box(m);
    let settled = stage.bbox_materializations(m);
    let rgba_view = stage
        .get_mut(m)
        .expect("live")
        .buffer
        .export_field_view("rgba", true)
        .expect("rgba field");
    let _ = stage.get_bounding_box(m);
    let _ = stage.get_bounding_box(m);
    assert_eq!(
        stage.bbox_materializations(m),
        settled,
        "a nonpoint field cannot move a bounding box"
    );
    drop(rgba_view);
}

#[test]
fn caller_installed_box_is_identifiable_and_revision_scoped() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]]);
    let transformed = BoundingBox {
        min: [0.0, -1.0, 0.0],
        mid: [0.0, 0.0, 0.0],
        max: [2.0, 1.0, 0.0],
    };

    stage.install_bounding_box_cache(m, transformed);
    assert_eq!(stage.installed_bounding_box_cache(m), Some(transformed));
    assert_eq!(stage.get_bounding_box(m), transformed);

    stage
        .get_mut(m)
        .expect("live")
        .buffer
        .write(0, "point", &[4.0, 5.0, 0.0]);
    assert_eq!(stage.installed_bounding_box_cache(m), None);
    assert_eq!(stage.get_bounding_box(m).max, [4.0, 5.0, 0.0]);
    assert_eq!(
        stage.installed_bounding_box_cache(m),
        None,
        "an ordinary lazy rematerialization is not a caller-installed box"
    );
}

#[test]
fn positional_op_invalidates() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[0.0, 0.0, 0.0], [2.0, 2.0, 0.0]]);
    assert_eq!(stage.get_center(m), [1.0, 1.0, 0.0]);
    stage.shift(m, [1.0, 0.0, 0.0]);
    assert_eq!(stage.get_center(m), [2.0, 1.0, 0.0]);
}

#[test]
fn leaf_mutation_dirties_ancestor_exactly_once() {
    let mut stage = Stage::new();
    let parent = stage.add(Mobject::new());
    let child = leaf(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    stage.attach(parent, child).unwrap();

    // Prime both boxes.
    let _ = stage.get_bounding_box(parent);
    let base_parent = stage.bbox_materializations(parent);

    // A single leaf mutation, then many ancestor reads → one ancestor recompute.
    stage
        .get_mut(child)
        .unwrap()
        .buffer
        .write(1, "point", &[5.0, 5.0, 0.0]);
    for _ in 0..3 {
        let bb = stage.get_bounding_box(parent);
        assert_eq!(bb.max, [5.0, 5.0, 0.0]);
    }
    assert_eq!(stage.bbox_materializations(parent), base_parent + 1);
}

#[test]
fn structural_change_invalidates_ancestor_box() {
    let mut stage = Stage::new();
    let parent = leaf(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    assert_eq!(stage.get_bounding_box(parent).max, [1.0, 1.0, 0.0]);

    // Attaching a child that extends further must grow the parent box.
    let child = leaf(&mut stage, &[[4.0, 4.0, 0.0]]);
    stage.attach(parent, child).unwrap();
    assert_eq!(stage.get_bounding_box(parent).max, [4.0, 4.0, 0.0]);

    // Detaching it shrinks the box back.
    stage.detach(parent, child);
    assert_eq!(stage.get_bounding_box(parent).max, [1.0, 1.0, 0.0]);
}

#[test]
fn uniform_read_write_round_trip() {
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[0.0, 0.0, 0.0]]);

    // Defaults are the Reference defaults.
    assert_eq!(stage.uniforms(m).unwrap().anti_alias_width, 1.5);

    // Write directly (the future Python-bridge `mobject.uniforms[...] = ...`).
    {
        let u = stage.uniforms_mut(m).unwrap();
        u.anti_alias_width = 2.0;
        u.is_fixed_in_frame = 1.0;
        u.joint_type = JointType::Miter;
        u.shading = [0.1, 0.2, 0.3];
        u.clip_planes[2] = [1.0, 0.0, 0.0, -1.0];
    }
    let u = stage.uniforms(m).unwrap();
    assert_eq!(u.anti_alias_width, 2.0);
    assert_eq!(u.is_fixed_in_frame, 1.0);
    assert_eq!(u.joint_type, JointType::Miter);
    assert_eq!(u.shading(), [0.1, 0.2, 0.3]);
    assert_eq!(u.clip_planes[2], [1.0, 0.0, 0.0, -1.0]);

    // The recursing setters reach the whole family.
    let child = leaf(&mut stage, &[[1.0, 1.0, 0.0]]);
    stage.attach(m, child).unwrap();
    stage.set_anti_alias_width(m, 3.0, true);
    assert_eq!(stage.uniforms(child).unwrap().anti_alias_width, 3.0);
}

#[test]
fn c2_scale_stroke_with_zoom_reads_correct_uniform() {
    // BN-07: independent of flat_stroke, unlike the Reference bug.
    let mut stage = Stage::new();
    let m = leaf(&mut stage, &[[0.0, 0.0, 0.0]]);
    stage.set_flat_stroke(m, true, false);
    stage.set_scale_stroke_with_zoom(m, false, false);
    assert!(!stage.get_scale_stroke_with_zoom(m)); // Reference would say true

    stage.set_flat_stroke(m, false, false);
    stage.set_scale_stroke_with_zoom(m, true, false);
    assert!(stage.get_scale_stroke_with_zoom(m));
}

#[test]
fn c7_use_winding_fill_changes_no_output_bits() {
    // On a real fixture scene, toggling use_winding_fill must alter nothing but
    // the accepted flag — no point moves, the box is untouched.
    let mut stage = Stage::new();
    let m = leaf(
        &mut stage,
        &[[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [0.5, 2.0, 0.0]],
    );
    let before_pts = stage.get(m).unwrap().buffer.read_column("point").unwrap();
    let before_box = stage.get_bounding_box(m);

    stage.use_winding_fill(m, true, true);

    assert!(stage.uniforms(m).unwrap().use_winding_fill);
    let after_pts = stage.get(m).unwrap().buffer.read_column("point").unwrap();
    assert_eq!(before_pts, after_pts, "no point bits may change");
    assert_eq!(
        before_box,
        stage.get_bounding_box(m),
        "box must be untouched"
    );
    // A directional query is unaffected too.
    let _ = UP;
    assert_eq!(stage.get_top(m), [0.0, 2.0, 0.0]);
}
