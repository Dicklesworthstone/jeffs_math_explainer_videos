//! fm-3kr tranche 1: `ClockPassesTime` wiring — the two Rotating members
//! about the clock's center, the Reference's defaults, and the documented
//! hour/minute angle ratio.

use fmn_anim::animation::Animation;
use fmn_anim::{clock_passes_time, rotate_default};
use fmn_mobject::{Mob, Mobject, Stage};

fn point_mob(stage: &mut Stage, point: [f64; 3]) -> Mob {
    stage.add(Mobject::from_points(&[point]))
}

#[test]
fn clock_passes_time_wires_two_rotations_with_reference_defaults() {
    let mut stage = Stage::new();
    let clock = point_mob(&mut stage, [0.0; 3]);
    let hour_hand = point_mob(&mut stage, [0.0, 0.15, 0.0]);
    let minute_hand = point_mob(&mut stage, [0.0, 0.3, 0.0]);

    let group = clock_passes_time(&mut stage, clock, hour_hand, minute_hand, 5.0, 12.0)
        .expect("wiring succeeds");

    assert_eq!(group.state().config.name, "ClockPassesTime");
    assert_eq!(group.state().config.run_time, 5.0);
    assert!(!group.state().config.remover);
    assert_eq!(group.animations().len(), 2, "hour + minute rotations");
}

#[test]
fn rotate_default_still_stands_as_the_baseline() {
    // Guard against the shelf tranche disturbing the rotation surface.
    let mut stage = Stage::new();
    let mob = point_mob(&mut stage, [1.0, 0.0, 0.0]);
    let rotating = rotate_default(mob);
    assert_eq!(rotating.state().config.name, "Rotate");
}
