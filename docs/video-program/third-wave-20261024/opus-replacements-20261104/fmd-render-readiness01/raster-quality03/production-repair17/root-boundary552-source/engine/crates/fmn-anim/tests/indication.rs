//! Indication-family semantics (fm-cye, §9.4 family 4): Indicate's
//! there-and-back swell, TurnInsideOut's reversal under the C-1 ruling,
//! WiggleOutThenIn's absolute pose, the VShowPassingFlash gaussian
//! window with style restore, and ApplyWave's phased nudge.

use fmn_anim::animation::Animation;
use fmn_anim::{
    INDICATION_YELLOW, RateFunc, VShowPassingFlash, WiggleOutThenIn, apply_wave, indicate,
    show_creation_then_destruction, turn_inside_out,
};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

fn vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    entry
        .buffer
        .write_range("fill_rgba", 0, &[0.2, 0.4, 0.6, 1.0].repeat(points.len()));
    entry
        .buffer
        .write_range("stroke_rgba", 0, &[1.0, 0.0, 0.0, 0.8].repeat(points.len()));
    entry
        .buffer
        .write_range("stroke_width", 0, &vec![2.0; points.len()]);
    mob
}

/// A 2-curve straight-line vmobject with x = [0, 1, 2, 3, 4].
fn line5(stage: &mut Stage) -> Mob {
    vmob(
        stage,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ],
    )
}

fn coords(stage: &Stage, mob: Mob, lane: usize) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation)]
    stage
        .get_points(mob)
        .unwrap()
        .into_iter()
        .map(|point| point[lane] as f32)
        .collect()
}

fn column(stage: &Stage, mob: Mob, field: &str) -> Vec<f32> {
    stage.get(mob).unwrap().buffer.read_column(field).unwrap()
}

fn assert_close(actual: &[f32], expected: &[f32], tol: f32) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() <= tol, "{actual:?} !~ {expected:?}");
    }
}

#[test]
fn indicate_swells_flushes_and_returns() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = indicate(&mut stage, mob, 1.2, None).unwrap();
    anim.begin(&mut stage).unwrap();
    // there_and_back peaks at α = 0.5: fully at the scaled, yellow target.
    anim.interpolate(&mut stage, 0.5);
    assert_close(&coords(&stage, mob, 0), &[-0.4, 0.8, 2.0, 3.2, 4.4], 1e-4);
    let fill = column(&stage, mob, "fill_rgba");
    assert_close(&fill[..3], &INDICATION_YELLOW, 1e-6);
    // And back: the finish state is the original.
    anim.finish(&mut stage);
    assert_close(&coords(&stage, mob, 0), &[0.0, 1.0, 2.0, 3.0, 4.0], 1e-4);
    assert_close(
        &column(&stage, mob, "fill_rgba")[..3],
        &[0.2, 0.4, 0.6],
        1e-5,
    );
}

#[test]
fn turn_inside_out_lands_on_the_reversed_run() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    stage
        .get_mut(mob)
        .unwrap()
        .buffer
        .write_range("stroke_width", 0, &[1.0, 2.0, 3.0, 4.0, 5.0]);
    let mut anim = turn_inside_out(&mut stage, mob, std::f64::consts::FRAC_PI_2).unwrap();
    anim.begin(&mut stage).unwrap();
    anim.finish(&mut stage);
    // data[::-1]: points and their row-mates travel together.
    assert_close(&coords(&stage, mob, 0), &[4.0, 3.0, 2.0, 1.0, 0.0], 1e-5);
    assert_close(
        &column(&stage, mob, "stroke_width"),
        &[5.0, 4.0, 3.0, 2.0, 1.0],
        1e-5,
    );
}

#[test]
fn wiggle_is_an_absolute_pose_and_returns_home() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let original = coords(&stage, mob, 0);
    let mut anim = WiggleOutThenIn::new(mob);
    assert_eq!(anim.state().config.run_time, 2.0);
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.3);
    let pose = coords(&stage, mob, 0);
    anim.interpolate(&mut stage, 0.3);
    assert_close(&coords(&stage, mob, 0), &pose, 1e-6);
    anim.finish(&mut stage);
    assert_close(&coords(&stage, mob, 0), &original, 1e-4);
}

#[test]
fn v_show_passing_flash_windows_widths_then_restores_style() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = VShowPassingFlash::new(mob)
        .with_time_width(0.3)
        .with_taper_width(0.0);
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert!(anim.is_remover());
    anim.begin(&mut stage).unwrap();
    // μ at α=0.5 is 0.5: full width at the middle point, zero outside 3σ.
    anim.interpolate(&mut stage, 0.5);
    let widths = column(&stage, mob, "stroke_width");
    assert!((widths[2] - 2.0).abs() < 1e-5, "peak keeps its width");
    assert_eq!(widths[0], 0.0, "outside the 3σ support");
    assert_eq!(widths[4], 0.0);
    // finish restores every member's style from the start.
    anim.finish(&mut stage);
    assert_close(&column(&stage, mob, "stroke_width"), &[2.0; 5], 1e-6);
}

#[test]
fn show_creation_then_destruction_is_the_wide_flash() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let anim = show_creation_then_destruction(mob);
    assert_eq!(anim.state().config.name, "ShowCreationThenDestruction");
    assert!(anim.is_remover());
}

#[test]
fn apply_wave_nudges_mid_animation_and_settles() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = apply_wave(&stage, mob, [0.0, 1.0, 0.0], 0.2);
    assert_eq!(anim.state().config.run_time, 1.0);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.5);
    assert!(
        coords(&stage, mob, 1).iter().any(|&y| y.abs() > 1e-3),
        "the wave displaces interior points"
    );
    anim.finish(&mut stage);
    assert_close(&coords(&stage, mob, 1), &[0.0; 5], 1e-4);
}

// --------------------------------------- fm-jh7: surrounding family + Broadcast
use fmn_anim::{
    animation_on_surrounding_rectangle, broadcast, flash_around, restore,
    show_creation_then_destruction_around, show_creation_then_fade_around,
    show_passing_flash_around,
};

#[test]
fn flash_around_wiring_carries_the_reference_defaults() {
    let mut stage = Stage::new();
    let path = vmob(
        &mut stage,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ],
    );
    let flash = flash_around(path, 1.0, 0.0);
    assert_eq!(flash.state().config.name, "FlashAround");
    assert!(flash.state().config.remover, "the flash removes its path");
    assert_eq!(flash.state().mobject(), path, "the sweep rides the path");
}

#[test]
fn animation_on_surrounding_rectangle_carries_one_member() {
    let mut stage = Stage::new();
    let rect = vmob(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
    let inner = Box::new(show_creation_then_destruction(rect));
    let group = animation_on_surrounding_rectangle(&mut stage, inner).expect("one-member group");
    assert_eq!(group.state().config.name, "AnimationOnSurroundingRectangle");
    assert_eq!(group.animations().len(), 1);
}

#[test]
fn the_around_variants_name_and_wrap_their_rect_animation() {
    let mut stage = Stage::new();
    let rect = vmob(&mut stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);

    let flash = show_passing_flash_around(&mut stage, rect, 1.0).expect("flash around");
    assert_eq!(flash.state().config.name, "ShowPassingFlashAround");
    assert_eq!(flash.animations().len(), 1);

    let destruction =
        show_creation_then_destruction_around(&mut stage, rect).expect("destruction around");
    assert_eq!(
        destruction.state().config.name,
        "ShowCreationThenDestructionAround"
    );
    assert_eq!(destruction.animations().len(), 1);

    let fade = show_creation_then_fade_around(&mut stage, rect, 0.0, true).expect("fade around");
    assert_eq!(fade.state().config.name, "ShowCreationThenFadeAround");
    assert_eq!(fade.animations().len(), 1);
}

#[test]
fn broadcast_wires_the_lagged_restores_with_reference_defaults() {
    let mut stage = Stage::new();
    let mut restores: Vec<Box<dyn Animation>> = Vec::new();
    for index in 0..3 {
        let circle = vmob(
            &mut stage,
            &[
                [f64::from(index), 0.0, 0.0],
                [f64::from(index) + 1.0, 0.0, 0.0],
            ],
        );
        stage.save_state(circle).expect("saved for the restore");
        restores.push(Box::new(restore(&stage, circle).expect("restore member")));
    }
    let group = broadcast(&mut stage, restores, 3.0, 0.2, true).expect("broadcast");
    assert_eq!(group.state().config.name, "Broadcast");
    assert_eq!(group.state().config.run_time, 3.0);
    assert_eq!(group.state().config.lag_ratio, 0.2);
    assert!(group.state().config.remover);
    assert_eq!(group.animations().len(), 3);
}

#[test]
fn flashy_fade_in_fades_the_mobject_while_an_outline_copy_flashes() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = fmn_anim::flashy_fade_in(&mut stage, mob, 2.0, 0.0, 1.0).unwrap();
    assert_eq!(anim.state().config.name, "FlashyFadeIn");
    // The container holds the mobject and its outline copy, in that order.
    let members = stage
        .get(anim.state().mobject())
        .unwrap()
        .submobjects()
        .to_vec();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0], mob);
    let outline = members[1];
    assert_ne!(outline, mob);
    // The outline is stroke-only: fill alpha 0, stroke width 2 at alpha 1.
    let fill = column(&stage, outline, "fill_rgba");
    assert!(fill.iter().skip(3).step_by(4).all(|a| *a == 0.0));
    let stroke = column(&stage, outline, "stroke_rgba");
    assert!(stroke.iter().skip(3).step_by(4).all(|a| *a == 1.0));
    assert!(
        column(&stage, outline, "stroke_width")
            .iter()
            .all(|w| *w == 2.0)
    );
    // The original paint is untouched by building the outline.
    let original = column(&stage, mob, "fill_rgba");
    assert!(original.iter().skip(3).step_by(4).all(|a| *a == 1.0));

    anim.begin(&mut stage).unwrap();
    anim.interpolate(&mut stage, 0.0);
    let start = column(&stage, mob, "fill_rgba");
    assert!(
        start.iter().skip(3).step_by(4).all(|a| *a == 0.0),
        "{start:?}"
    );
    anim.interpolate(&mut stage, 0.5);
    let middle = column(&stage, mob, "fill_rgba");
    assert!(
        middle
            .iter()
            .skip(3)
            .step_by(4)
            .all(|a| *a > 0.0 && *a < 1.0),
        "{middle:?}"
    );
    anim.finish(&mut stage);
    let end = column(&stage, mob, "fill_rgba");
    assert_close(&end, &[0.2, 0.4, 0.6, 1.0].repeat(5), 1e-6);
}

#[test]
fn flashy_fade_in_squishes_the_fade_into_its_lag_window() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let mut anim = fmn_anim::flashy_fade_in(&mut stage, mob, 2.0, 0.5, 1.0).unwrap();
    anim.begin(&mut stage).unwrap();
    // Before fade_lag the fade has not started.
    anim.interpolate(&mut stage, 0.4);
    let early = column(&stage, mob, "fill_rgba");
    assert!(
        early.iter().skip(3).step_by(4).all(|a| *a == 0.0),
        "{early:?}"
    );
    anim.interpolate(&mut stage, 0.75);
    let later = column(&stage, mob, "fill_rgba");
    assert!(
        later.iter().skip(3).step_by(4).all(|a| *a > 0.0),
        "{later:?}"
    );
    let gone = line5(&mut stage);
    stage.delete(gone).unwrap();
    assert!(fmn_anim::flashy_fade_in(&mut stage, gone, 2.0, 0.0, 1.0).is_err());
}
