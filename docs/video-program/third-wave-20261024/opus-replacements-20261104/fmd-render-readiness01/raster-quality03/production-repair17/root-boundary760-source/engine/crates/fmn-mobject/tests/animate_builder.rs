//! `.animate` builder fixture corpus (fm-yra acceptance): chained-call
//! recording against a build-time target, once-only animation arguments,
//! the override no-chaining rule in both directions, dynamic target lookup,
//! and the prepare_animation contract.

use fmn_core::constants::{ORIGIN, RIGHT, UP};
use fmn_mobject::animate::{AnimateArgs, AnimateError, OverrideAnimation};
use fmn_mobject::{IntoAnimate, Mobject, Stage};

fn square(stage: &mut Stage) -> fmn_mobject::Mob {
    stage.add(Mobject::from_points(&[
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ]))
}

#[test]
fn recording_applies_to_the_target_copy_not_the_source() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let before = stage.get_center(mob);

    let built = mob
        .animate()
        .shift([2.0, 0.0, 0.0])
        .and_then(|b| b.scale(2.0))
        .and_then(|b| b.build(&mut stage))
        .expect("chain builds");

    // Source untouched; target carries the composed result.
    assert_eq!(stage.get_center(mob), before);
    let target_center = stage.get_center(built.target);
    assert!((target_center[0] - 2.0).abs() < 1e-6);
    assert!((stage.get_width(built.target) - 2.0).abs() < 1e-6);
    assert_eq!(built.source, mob);
    assert_ne!(built.target, mob);
}

#[test]
fn rotate_records_against_the_target_copy() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[
        [-1.0, -0.5, 0.0],
        [1.0, -0.5, 0.0],
        [1.0, 0.5, 0.0],
        [-1.0, 0.5, 0.0],
    ]));

    let built = mob
        .animate()
        .rotate(std::f64::consts::FRAC_PI_2)
        .and_then(|builder| builder.build(&mut stage))
        .expect("rotation builds");

    assert!((stage.get_width(mob) - 2.0).abs() < 1e-9);
    assert!((stage.get_height(mob) - 1.0).abs() < 1e-9);
    assert!((stage.get_width(built.target) - 1.0).abs() < 1e-9);
    assert!((stage.get_height(built.target) - 2.0).abs() < 1e-9);
}

#[test]
fn opacity_records_against_every_rgba_field_in_the_target_family() {
    let mut stage = Stage::new();
    let parent = square(&mut stage);
    let child = square(&mut stage);
    stage
        .attach(parent, child)
        .expect("fixture family is acyclic");

    let built = parent
        .animate()
        .set_opacity(0.25)
        .and_then(|builder| builder.build(&mut stage))
        .expect("opacity builds");
    let target_family = stage.family(built.target);

    for source in [parent, child] {
        let rgba = stage
            .get(source)
            .and_then(|entry| entry.buffer.read_column("rgba"))
            .expect("source rgba");
        assert!(rgba.iter().skip(3).step_by(4).all(|alpha| *alpha == 0.0));
    }
    for target in target_family {
        let rgba = stage
            .get(target)
            .and_then(|entry| entry.buffer.read_column("rgba"))
            .expect("target rgba");
        assert!(rgba.iter().skip(3).step_by(4).all(|alpha| *alpha == 0.25));
    }
}

#[test]
fn anim_args_are_set_once_per_chain() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);

    let args = AnimateArgs {
        run_time: Some(2.0),
        lag_ratio: Some(0.1),
        ..AnimateArgs::default()
    };
    let builder = mob
        .animate()
        .set_anim_args(args)
        .expect("first args pass is fine");
    // The second pass is the Reference's ValueError.
    assert_eq!(
        builder.clone().set_anim_args(AnimateArgs::default()),
        Err(AnimateError::ArgsAlreadySet)
    );
    let built = builder
        .shift([1.0, 0.0, 0.0])
        .and_then(|b| b.build(&mut stage))
        .expect("builds");
    assert_eq!(built.args.run_time, Some(2.0));
    assert_eq!(built.args.lag_ratio, Some(0.1));
}

#[test]
fn overridden_animations_do_not_chain_in_either_direction() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let ov = OverrideAnimation { name: "fade_stub" };

    // Override after a chained call: refused.
    assert_eq!(
        mob.animate()
            .shift([1.0, 0.0, 0.0])
            .and_then(|b| b.with_override(ov)),
        Err(AnimateError::OverrideNotChainable)
    );
    // Chained call after an override: refused.
    assert_eq!(
        mob.animate()
            .with_override(ov)
            .and_then(|b| b.shift([1.0, 0.0, 0.0])),
        Err(AnimateError::OverrideNotChainable)
    );
    // A second override: refused.
    assert_eq!(
        mob.animate()
            .with_override(ov)
            .and_then(|b| b.with_override(ov)),
        Err(AnimateError::OverrideNotChainable)
    );
    // An override alone builds, carrying its marker.
    let built = mob
        .animate()
        .with_override(ov)
        .and_then(|b| b.build(&mut stage))
        .expect("override alone builds");
    assert_eq!(built.overridden, Some(ov));
}

#[test]
fn mob_targets_resolve_at_build_time() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let anchor = square(&mut stage);

    // Record a next_to against the anchor, THEN move the anchor: the build
    // must see the anchor's position at build time (dynamic target lookup),
    // not where it was when the chain was written.
    let builder = mob
        .animate()
        .next_to(anchor, RIGHT, 0.25, ORIGIN)
        .expect("records");
    stage.shift(anchor, [5.0, 3.0, 0.0]);
    let built = builder.build(&mut stage).expect("builds");

    let anchor_right = stage.get_right(anchor);
    let target_left = stage.get_left(built.target);
    assert!((target_left[0] - (anchor_right[0] + 0.25)).abs() < 1e-6);
    assert!((stage.get_center(built.target)[1] - stage.get_center(anchor)[1]).abs() < 1e-6);
}

#[test]
fn stale_handles_fail_the_build_by_name() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    let other = square(&mut stage);

    // Dead source.
    let doomed = square(&mut stage);
    let chain = doomed.animate().shift([1.0, 0.0, 0.0]).expect("records");
    stage.delete(doomed).unwrap();
    assert_eq!(
        chain.build(&mut stage),
        Err(AnimateError::StaleHandle(doomed))
    );

    // Dead Mob target inside a command — named precisely.
    let chain = mob.animate().move_to(other, UP).expect("records");
    stage.delete(other).unwrap();
    assert_eq!(
        chain.build(&mut stage),
        Err(AnimateError::StaleHandle(other))
    );
}

#[test]
fn prepare_animation_contract_accepts_builders_and_built() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);

    // A builder prepares by building.
    let built = mob
        .animate()
        .shift([1.0, 1.0, 0.0])
        .expect("records")
        .prepare(&mut stage)
        .expect("builder prepares");
    // A built animation prepares as identity.
    let target = built.target;
    let again = built.prepare(&mut stage).expect("identity prepare");
    assert_eq!(again.target, target);
    // (A bare method is unrepresentable: only these two types implement
    // IntoAnimate — the typed form of the Reference's TypeError.)
}

fn vsquare(stage: &mut Stage) -> fmn_mobject::Mob {
    stage.add(Mobject::vector_from_points(&[
        [-0.5, -0.5, 0.0],
        [0.0, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.0, 0.0],
        [0.5, 0.5, 0.0],
    ]))
}

fn lanes(stage: &Stage, mob: fmn_mobject::Mob, field: &str) -> Vec<f32> {
    stage.get(mob).unwrap().buffer.read_column(field).unwrap()
}

#[test]
fn style_commands_record_against_the_target_family_only() {
    use fmn_core::color::Srgb;
    let mut stage = Stage::new();
    let parent = vsquare(&mut stage);
    let child = vsquare(&mut stage);
    stage.attach(parent, child).unwrap();
    let red = Srgb {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    let blue = Srgb {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
    let built = parent
        .animate()
        .set_color(red)
        .and_then(|b| b.set_fill(Some(blue), Some(0.5)))
        .and_then(|b| b.set_stroke(None, Some(7.0), Some(0.75)))
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    for (source, target) in [parent, child].into_iter().zip(stage.family(built.target)) {
        // The source keeps its unstyled (zero) paint lanes.
        assert!(lanes(&stage, source, "fill_rgba").iter().all(|v| *v == 0.0));
        let fill = lanes(&stage, target, "fill_rgba");
        let stroke = lanes(&stage, target, "stroke_rgba");
        let widths = lanes(&stage, target, "stroke_width");
        for (fill, stroke) in fill.chunks(4).zip(stroke.chunks(4)) {
            // set_fill's later colour wins on fill; stroke keeps set_color's
            // red, and only its alpha was rewritten.
            assert_eq!(fill, [0.0, 0.0, 1.0, 0.5]);
            assert_eq!(stroke, [1.0, 0.0, 0.0, 0.75]);
        }
        assert!(widths.iter().all(|w| *w == 7.0));
    }
}

#[test]
fn fade_and_color_with_opacity_write_alpha_lanes() {
    use fmn_core::color::Srgb;
    let mut stage = Stage::new();
    let mob = vsquare(&mut stage);
    let faded = mob
        .animate()
        .fade(0.75)
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    for field in ["fill_rgba", "stroke_rgba"] {
        let column = lanes(&stage, faded.target, field);
        assert!(column.iter().skip(3).step_by(4).all(|a| *a == 0.25));
    }
    let green = Srgb {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    let tinted = mob
        .animate()
        .set_color_with_opacity(green, 0.4)
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    for field in ["fill_rgba", "stroke_rgba"] {
        for rgba in lanes(&stage, tinted.target, field).chunks(4) {
            assert_eq!(rgba, [0.0, 1.0, 0.0, 0.4]);
        }
    }
    // A base mobject's single `rgba` column takes set_color too.
    let dots = square(&mut stage);
    let colored = dots
        .animate()
        .set_color(green)
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    for rgba in lanes(&stage, colored.target, "rgba").chunks(4) {
        assert_eq!(&rgba[..3], [0.0, 1.0, 0.0]);
    }
}

#[test]
fn transform_commands_match_the_stage_operations() {
    use fmn_core::constants::{LEFT, OUT};
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.shift(mob, [1.0, 0.0, 0.0]);
    let anchor = square(&mut stage);
    stage.shift(anchor, [0.0, 3.0, 0.0]);
    stage.set_width(anchor, 4.0, true);

    // rotate about an explicit point vs the stage's own rotate.
    let built = mob
        .animate()
        .rotate_about(std::f64::consts::FRAC_PI_2, OUT, Some(ORIGIN))
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    let center = stage.get_center(built.target);
    assert!(
        center[0].abs() < 1e-6 && (center[1] - 1.0).abs() < 1e-6,
        "{center:?}"
    );

    // scale about an edge keeps that edge fixed.
    let left_before = stage.get_edge_center(mob, LEFT);
    let built = mob
        .animate()
        .scale_about(3.0, None, Some(LEFT))
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    let left_after = stage.get_edge_center(built.target, LEFT);
    assert!((left_after[0] - left_before[0]).abs() < 1e-6);
    assert!((stage.get_width(built.target) - 3.0).abs() < 1e-6);

    // flip about UP mirrors x through the center; the box is unchanged.
    let built = mob
        .animate()
        .flip(UP)
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    assert!((stage.get_center(built.target)[0] - 1.0).abs() < 1e-6);

    // apply_matrix is about the origin.
    let built = mob
        .animate()
        .apply_matrix([[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]])
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    assert!((stage.get_center(built.target)[0] - 2.0).abs() < 1e-6);
    assert!((stage.get_width(built.target) - 2.0).abs() < 1e-6);

    // set_x / set_y / match_* read the other mobject at build time.
    let built = mob
        .animate()
        .set_x(-2.0)
        .and_then(|b| b.set_y(0.5))
        .and_then(|b| b.match_width(anchor))
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    let center = stage.get_center(built.target);
    assert!((center[0] + 2.0).abs() < 1e-6 && (center[1] - 0.5).abs() < 1e-6);
    assert!((stage.get_width(built.target) - 4.0).abs() < 1e-6);
    let built = mob
        .animate()
        .match_y(anchor)
        .and_then(|b| b.build(&mut stage))
        .unwrap();
    assert!((stage.get_center(built.target)[1] - 3.0).abs() < 1e-6);
    // The source never moved.
    assert!((stage.get_center(mob)[0] - 1.0).abs() < 1e-6);
}

#[test]
fn arrange_records_over_the_target_children_and_stale_match_targets_refuse() {
    let mut stage = Stage::new();
    let group = stage.add(Mobject::new());
    let a = square(&mut stage);
    let b = square(&mut stage);
    stage.attach(group, a).unwrap();
    stage.attach(group, b).unwrap();
    let built = group
        .animate()
        .arrange(RIGHT, 0.25)
        .and_then(|builder| builder.build(&mut stage))
        .unwrap();
    let family = stage.family(built.target);
    let (ta, tb) = (family[1], family[2]);
    let gap = stage.get_edge_center(tb, fmn_core::constants::LEFT)[0]
        - stage.get_edge_center(ta, RIGHT)[0];
    assert!((gap - 0.25).abs() < 1e-6, "{gap}");
    assert!(stage.get_center(built.target)[0].abs() < 1e-6);
    // The source children still overlap.
    assert_eq!(stage.get_center(a), stage.get_center(b));

    let other = square(&mut stage);
    let recording = group.animate().match_x(other).unwrap();
    stage.delete(other).unwrap();
    assert!(matches!(
        recording.build(&mut stage),
        Err(AnimateError::StaleHandle(m)) if m == other
    ));
}
