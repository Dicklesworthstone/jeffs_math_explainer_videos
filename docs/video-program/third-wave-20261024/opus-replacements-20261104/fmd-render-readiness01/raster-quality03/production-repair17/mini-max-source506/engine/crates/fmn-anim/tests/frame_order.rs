//! The update-order corpus (fm-x79 acceptance): the six-step frame order
//! as semantic regression tests — each scene asserts observable state at
//! specific frames, never pixels — plus the FramePacket freeze contract
//! (immutability by construction, O(touched) cost, derivable RNG forks).

use std::cell::RefCell;
use std::rc::Rc;

use fmn_anim::frame::{
    FramePacket, abort_open_play, complete_play_frame, finish_open_play,
    finish_open_play_animations, open_play, play_segment, prepare_play_frame, wait_segment,
    wait_segment_with_boundary,
};
use fmn_anim::{
    AnimConfig, AnimError, AnimState, Animation, FrameSample, RationalFrameClock, RationalTime,
    prepare_animation,
};
use fmn_core::rng::RngRoot;
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mob, Mobject, Stage};

fn square(stage: &mut Stage) -> Mob {
    stage.add(Mobject::from_points(&[
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ]))
}

/// A linear-rate `.animate` shift of `dx`, prepared into a boxed animation.
fn shift_animation(stage: &mut Stage, mob: Mob, dx: f64, run_time: f64) -> Box<dyn Animation> {
    let builder = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn_core::rate::linear),
            ..AnimateArgs::default()
        })
        .and_then(|b| b.shift([dx, 0.0, 0.0]))
        .expect("chain records");
    prepare_animation(builder, stage).expect("prepares")
}

#[test]
fn host_finish_boundary_resumes_before_host_zero_dt_and_updates_native_once() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let ticks = Rc::new(RefCell::new(0usize));
    let seen_ticks = Rc::clone(&ticks);
    stage
        .add_dt_updater(
            mob,
            move |_stage, current, dt| {
                if current == mob {
                    assert_eq!(dt, 0.0);
                    *seen_ticks.borrow_mut() += 1;
                }
            },
            false,
        )
        .expect("updater installs");

    let mut animation = shift_animation(&mut stage, mob, 1.0, 1.0 / 30.0);
    animation.state_mut().config.suspend_mobject_updating = true;
    let mut animations = vec![animation];
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut open = open_play(&mut stage, &clock, &mut animations).expect("opens");
    while prepare_play_frame(&mut stage, &mut clock, &mut animations, &mut open)
        .expect("prepares")
        .is_some()
    {
        complete_play_frame(
            &mut stage,
            &clock,
            &rng(),
            &mut open,
            false,
            &mut |_, _| {},
            &mut |_| {},
        )
        .expect("completes");
    }
    assert!(stage.is_updating_suspended(mob));
    assert_eq!(*ticks.borrow(), 0);

    finish_open_play_animations(&mut stage, &mut animations, &mut open)
        .expect("animation lifecycles finish");
    assert!(!stage.is_updating_suspended(mob));
    assert_eq!(
        *ticks.borrow(),
        1,
        "resume_updating performs its own zero-dt pass"
    );

    finish_open_play(&mut stage, &mut animations, open).expect("native epilogue finishes");
    assert_eq!(*ticks.borrow(), 2);
}

fn rng() -> RngRoot {
    RngRoot::from_seed(42)
}

/// A custom setup hook that mutates the arena before refusing. Static
/// validation cannot predict arbitrary extension behavior, so the driver
/// transaction must contain it.
struct MutatingBeginFailure {
    state: AnimState,
}

impl Animation for MutatingBeginFailure {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        stage.shift(mobject, [9.0, 0.0, 0.0]);
        let transient = stage.add(Mobject::new());
        stage.add_to_scene(transient).expect("transient root");
        Err(AnimError::EmptyMobject)
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {}
}

// ---------------------------------------------------------- the six steps

#[test]
fn failed_begin_restores_the_stage_before_any_frame_is_emitted() {
    let mut stage = Stage::new();
    let first = square(&mut stage);
    let failing = square(&mut stage);
    stage.add_to_scene(first).expect("first root");
    stage.add_to_scene(failing).expect("failing root");
    let mut animations: Vec<Box<dyn Animation>> = vec![
        shift_animation(&mut stage, first, 2.0, 1.0),
        Box::new(MutatingBeginFailure {
            state: AnimState::new(failing, AnimConfig::default()),
        }),
    ];
    let before = stage.snapshot_bytes().expect("begin state encodes");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut emitted = 0;

    let error = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| emitted += 1,
    )
    .expect_err("the second begin refuses");

    assert_eq!(error, AnimError::EmptyMobject);
    assert_eq!(clock.now().frames(), 0);
    assert_eq!(emitted, 0);
    assert_eq!(
        stage.snapshot_bytes().expect("restored state encodes"),
        before,
        "earlier begins and the failing hook leave no arena mutation"
    );
    assert!(!stage.is_animating(first));
    assert!(!stage.is_animating(failing));
}

#[test]
fn scene_updaters_observe_post_interpolation_state() {
    // The always_redraw property: a follower updater reads the animated
    // mobject's position and must see THIS frame's interpolation, not the
    // previous frame's.
    let mut stage = Stage::new();
    let animated = square(&mut stage);
    let follower = square(&mut stage);
    stage.add_to_scene(animated).expect("rooted");
    stage.add_to_scene(follower).expect("rooted");
    stage
        .add_updater(
            follower,
            move |stage, me| {
                let x = stage.get_center(animated)[0];
                stage.set_x(me, x);
            },
            false,
        )
        .expect("registers");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let rng = rng();
    let mut animations = vec![shift_animation(&mut stage, animated, 2.0, 1.0)];

    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &rng,
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    // Each packet froze at step 5; restoring it reconstructs that frame's
    // post-step-4 state exactly.
    assert_eq!(packets.len(), 30);
    for packet in &packets {
        stage.restore(packet.state());
        let animated_x = stage.get_center(animated)[0];
        let follower_x = stage.get_center(follower)[0];
        let alpha = packet.alpha();
        assert!(
            (animated_x - 2.0 * alpha).abs() < 1e-5,
            "animated at alpha {alpha}: {animated_x}"
        );
        assert!(
            (follower_x - animated_x).abs() < 1e-5,
            "follower must see this frame's interpolation: {follower_x} vs {animated_x}"
        );
    }
}

#[test]
fn stepped_release_window_preserves_step_four_and_capture_order() {
    let mut stage = Stage::new();
    let animated = square(&mut stage);
    let follower = square(&mut stage);
    stage.add_to_scene(animated).expect("rooted");
    stage.add_to_scene(follower).expect("rooted");
    stage
        .add_updater(
            follower,
            move |stage, me| {
                stage.set_x(me, stage.get_center(animated)[0]);
            },
            false,
        )
        .expect("registers");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let root = rng();
    let mut animations = vec![shift_animation(&mut stage, animated, 2.0, 1.0)];
    let mut open = open_play(&mut stage, &clock, &mut animations).expect("opens");
    let mut packets = Vec::new();

    while let Some(release) =
        prepare_play_frame(&mut stage, &mut clock, &mut animations, &mut open).expect("prepares")
    {
        assert_eq!(stage.time(), release.time.to_f64());
        // Model a host-language updater. It runs after interpolation and
        // clock advance, but before the native scene updater and freeze.
        stage.shift(animated, [0.25, 0.0, 0.0]);
        complete_play_frame(
            &mut stage,
            &clock,
            &root,
            &mut open,
            true,
            &mut |_, _| {},
            &mut |packet| packets.push(packet),
        )
        .expect("completes");
    }
    finish_open_play(&mut stage, &mut animations, open).expect("finishes");

    assert_eq!(packets.len(), 30);
    for packet in packets {
        stage.restore(packet.state());
        let expected = 2.0 * packet.alpha() + 0.25;
        let animated_x = stage.get_center(animated)[0];
        let follower_x = stage.get_center(follower)[0];
        assert!((animated_x - expected).abs() < 1e-5);
        assert!((follower_x - expected).abs() < 1e-5);
    }
}

#[test]
fn aborting_a_released_frame_clears_animation_lifecycle_without_landing_endpoint() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let ticks = Rc::new(RefCell::new(0usize));
    let seen_ticks = Rc::clone(&ticks);
    stage
        .add_dt_updater(
            mob,
            move |_stage, me, _dt| {
                if me == mob {
                    *seen_ticks.borrow_mut() += 1;
                }
            },
            false,
        )
        .expect("updater");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 3.0, 1.0)];
    animations[0].state_mut().config.suspend_mobject_updating = true;
    let mut open = open_play(&mut stage, &clock, &mut animations).expect("opens");
    let release = prepare_play_frame(&mut stage, &mut clock, &mut animations, &mut open)
        .expect("prepares")
        .expect("one frame");
    assert!(stage.is_animating(mob));
    assert!(stage.is_updating_suspended(mob));
    assert_eq!(*ticks.borrow(), 0);
    let partial_x = stage.get_center(mob)[0];
    assert!(partial_x > 0.0 && partial_x < 3.0);

    let report = abort_open_play(&mut stage, &mut animations, open);
    assert_eq!(report.n_frames, 30);
    assert_eq!(clock.now(), release.time);
    assert!(!stage.is_animating(mob));
    assert!(!stage.is_updating_suspended(mob));
    assert_eq!(
        *ticks.borrow(),
        0,
        "abort must not call the resumed updater"
    );
    assert_eq!(stage.get_center(mob)[0], partial_x, "abort is not finish");
}

#[test]
fn time_advances_before_scene_updaters_run() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let times: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&times);
    stage
        .add_updater(
            mob,
            move |stage, me| {
                // The starting/target copies share this callable and tick in
                // step 1 (pre-advance); isolate step 4's scene pass.
                if me == mob {
                    sink.borrow_mut().push(stage.time());
                }
            },
            false,
        )
        .expect("registers");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");

    let observed = times.borrow();
    // 0.1 s at 30 fps → 4 frames (BN-02 upward rounding), plus the
    // finish-pass dt=0 tick.
    assert_eq!(observed.len(), 5);
    for (index, observed) in observed[..4].iter().enumerate() {
        let frame = (index + 1) as f64;
        assert_eq!(
            observed.to_bits(),
            (frame / 30.0).to_bits(),
            "frame {} must expose the rational clock's one conversion",
            index + 1
        );
    }
    assert_eq!(
        observed[4].to_bits(),
        observed[3].to_bits(),
        "the finish updater pass advances no time"
    );
}

#[test]
fn animated_mobject_updaters_run_in_step_four_unless_suspended() {
    for suspend in [false, true] {
        let mut stage = Stage::new();
        let mob = square(&mut stage);
        stage.add_to_scene(mob).expect("rooted");
        let source_ticks = Rc::new(RefCell::new(0usize));
        let sink = Rc::clone(&source_ticks);
        stage
            .add_dt_updater(
                mob,
                move |_stage, me, _dt| {
                    // Copies share this callable; count only the scene's own.
                    if me == mob {
                        *sink.borrow_mut() += 1;
                    }
                },
                false,
            )
            .expect("registers");

        let mut clock = RationalFrameClock::new(30).expect("fps");
        let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 1.0)];
        animations[0].state_mut().config.suspend_mobject_updating = suspend;
        play_segment(
            &mut stage,
            &mut clock,
            &rng(),
            &mut animations,
            false,
            &mut |_| {},
        )
        .expect("plays");

        let ticks = *source_ticks.borrow();
        if suspend {
            // Nothing during the play; then finish's resume runs its single
            // update(0) (C-5: exactly once), and the now-resumed mobject
            // ticks once more in the shared finish pass — exactly as the
            // Reference's finish_animations sequence does.
            assert_eq!(ticks, 2, "suspended play ticks only at finish");
        } else {
            // 30 frames + the finish-pass dt=0 tick.
            assert_eq!(ticks, 31, "unsuspended play ticks every frame");
        }
    }
}

#[test]
fn starting_and_target_copies_tick_through_animation_update_mobjects() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let all_ticks = Rc::new(RefCell::new(Vec::<f64>::new()));
    let sink = Rc::clone(&all_ticks);
    stage
        .add_dt_updater(
            mob,
            move |_stage, _me, dt| {
                sink.borrow_mut().push(dt);
            },
            false,
        )
        .expect("registers");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 1.0)];
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");

    // Per frame: starting + target (step 1) + the source (step 4) = 3
    // ticks; 30 frames → 90; plus the finish-pass dt=0 source tick.
    let ticks = all_ticks.borrow();
    assert_eq!(ticks.len(), 91);
    let total: f64 = ticks.iter().sum();
    assert!(
        (total - 3.0).abs() < 1e-9,
        "three full run_times of dt across source+copies, got {total}"
    );
}

#[test]
fn skip_mode_matches_played_final_state_and_emits_nothing() {
    // BN-10: a skipped segment must leave dt-updaters and positions exactly
    // where a played segment does.
    let run = |skip: bool| -> (Vec<u8>, f64, usize, usize, i64) {
        let mut stage = Stage::new();
        let mob = square(&mut stage);
        stage.add_to_scene(mob).expect("rooted");
        let nonlinear = square(&mut stage);
        stage.shift(nonlinear, [1.0, 0.0, 0.0]);
        stage.add_to_scene(nonlinear).expect("nonlinear root");
        let calls = Rc::new(RefCell::new(0usize));
        let seen_calls = Rc::clone(&calls);
        stage
            .add_dt_updater(
                nonlinear,
                move |stage, me, dt| {
                    *seen_calls.borrow_mut() += 1;
                    let x = stage.get_center(me)[0];
                    stage.shift(me, [dt * (1.0 + x), 0.0, 0.0]);
                },
                false,
            )
            .expect("registers");
        let mut clock = RationalFrameClock::new(30).expect("fps");
        let mut animations = vec![shift_animation(&mut stage, mob, 2.0, 1.0)];
        let mut emitted = 0usize;
        play_segment(
            &mut stage,
            &mut clock,
            &rng(),
            &mut animations,
            skip,
            &mut |_| emitted += 1,
        )
        .expect("plays");
        (
            stage.snapshot_bytes().expect("canonical arena"),
            stage.time(),
            *calls.borrow(),
            emitted,
            clock.now().frames(),
        )
    };

    let (played, time_played, calls_played, emitted_played, frames_played) = run(false);
    let (skipped, time_skipped, calls_skipped, emitted_skipped, frames_skipped) = run(true);

    assert_eq!(
        played, skipped,
        "the complete terminal arena is bit-identical"
    );
    assert_eq!(calls_played, 31, "30 frames plus the dt=0 finish pass");
    assert_eq!(
        calls_played, calls_skipped,
        "updater call sequence identical"
    );
    assert!((time_played - time_skipped).abs() < 1e-9);
    assert_eq!(frames_played, frames_skipped, "the clock advances equally");
    assert_eq!(emitted_played, 30);
    assert_eq!(emitted_skipped, 0, "skip captures and emits nothing");
}

#[test]
fn unequal_run_times_in_one_play_clip_through_the_pipeline() {
    let mut stage = Stage::new();
    let mob_a = square(&mut stage);
    let mob_b = square(&mut stage);
    stage.add_to_scene(mob_a).expect("rooted");
    stage.add_to_scene(mob_b).expect("rooted");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![
        shift_animation(&mut stage, mob_a, 1.0, 0.5),
        shift_animation(&mut stage, mob_b, 2.0, 1.0),
    ];

    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    let mut at = |index: usize| -> (f64, f64) {
        stage.restore(packets[index].state());
        (stage.get_center(mob_a)[0], stage.get_center(mob_b)[0])
    };
    assert_eq!(packets.len(), 30, "the progression covers the longest");
    // Frame 15 (t = 0.5): the short animation is exactly done, the long
    // one halfway.
    let (a_mid, b_mid) = at(14);
    assert!((a_mid - 1.0).abs() < 1e-5, "short done at its run_time");
    assert!((b_mid - 1.0).abs() < 1e-5, "long halfway");
    // Beyond its run_time the short animation's alpha exceeds 1 and the
    // pipeline clips: it must hold, not overshoot.
    let (a_late, _) = at(22);
    assert!((a_late - 1.0).abs() < 1e-5, "clipped alpha holds the pose");
    let (a_final, b_final) = at(29);
    assert!((a_final - 1.0).abs() < 1e-5);
    assert!((b_final - 2.0).abs() < 1e-5);
}

#[test]
fn final_sample_exceeds_run_time_and_clamps_to_one() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    // 0.71 s at 30 fps → 22 frames; the last grid time 22/30 > 0.71.
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.71)];
    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    assert_eq!(packets.len(), 22);
    let last = packets.last().expect("nonempty");
    assert_eq!(last.alpha(), 1.0, "the packet's segment alpha clamps");
    assert!(
        (stage.get_center(mob)[0] - 1.0).abs() < 1e-6,
        "exactly at the target"
    );
}

#[test]
fn time_span_re_windows_inside_a_longer_play() {
    let mut stage = Stage::new();
    let spanned = square(&mut stage);
    let steady = square(&mut stage);
    stage.add_to_scene(spanned).expect("rooted");
    stage.add_to_scene(steady).expect("rooted");

    let spanned_builder = spanned
        .animate()
        .set_anim_args(AnimateArgs {
            rate_func: Some(fmn_core::rate::linear),
            time_span: Some((0.5, 1.5)),
            ..AnimateArgs::default()
        })
        .and_then(|b| b.shift([2.0, 0.0, 0.0]))
        .expect("records");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![
        prepare_animation(spanned_builder, &mut stage).expect("prepares"),
        shift_animation(&mut stage, steady, 1.0, 2.0),
    ];

    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    let mut at = |index: usize| -> f64 {
        stage.restore(packets[index].state());
        stage.get_center(spanned)[0]
    };
    assert_eq!(
        packets.len(),
        60,
        "the play runs the longest run_time (2 s)"
    );
    assert!(at(14).abs() < 1e-5, "still parked before the span opens");
    assert!(
        (at(29) - 1.0).abs() < 1e-5,
        "halfway through the span at t = 1"
    );
    assert!(
        (at(44) - 2.0).abs() < 1e-5,
        "done exactly as the span closes at t = 1.5"
    );
    assert!((at(59) - 2.0).abs() < 1e-5, "holds after the span");
}

// -------------------------------------------------------- wait semantics

#[test]
fn wait_emits_frames_and_runs_an_initial_zero_dt_pass() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let dts: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = Rc::clone(&dts);
    stage
        .add_dt_updater(
            mob,
            move |_stage, _me, dt| sink.borrow_mut().push(dt),
            false,
        )
        .expect("registers");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut emitted = 0usize;
    wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        0.5,
        None,
        false,
        &mut |_| emitted += 1,
    )
    .expect("waits");

    assert_eq!(emitted, 15);
    let observed = dts.borrow();
    assert_eq!(observed.len(), 16, "initial dt=0 pass + one per frame");
    assert_eq!(observed[0], 0.0, "the Reference's update_mobjects(dt=0)");
    assert!((observed[1] - 1.0 / 30.0).abs() < 1e-12);
}

#[test]
fn serial_boundary_runs_after_updaters_and_before_packet_freeze() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    stage
        .add_dt_updater(
            mob,
            |stage, me, dt| {
                if dt > 0.0 {
                    stage.shift(me, [1.0, 0.0, 0.0]);
                }
            },
            false,
        )
        .expect("updater");

    let mut clock = RationalFrameClock::new(1).expect("fps");
    let mut seen_before_boundary_write = Vec::new();
    let mut packets = Vec::new();
    wait_segment_with_boundary(
        &mut stage,
        &mut clock,
        &rng(),
        1.0,
        None,
        false,
        &mut |stage, time| {
            assert_eq!(time, RationalTime::zero(1) + 1);
            seen_before_boundary_write.push(stage.get_center(mob)[0]);
            stage.shift(mob, [10.0, 0.0, 0.0]);
        },
        &mut |packet| packets.push(packet),
    )
    .expect("waits");

    assert_eq!(
        seen_before_boundary_write,
        [1.0],
        "the Scene updater must have run before the boundary"
    );
    assert_eq!(packets.len(), 1);
    assert_eq!(
        packets[0].materialize_stage().get_center(mob)[0],
        11.0,
        "the immutable packet must include the boundary mutation"
    );
}

#[test]
fn wait_until_stops_after_the_frame_where_the_condition_turns_true() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut emitted = 0usize;
    // Condition: scene time reaches 0.2 s — true first at frame 6 (6/30).
    let mut condition = |stage: &Stage| stage.time() >= 0.2 - 1e-12;
    let report = wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        60.0,
        Some(&mut condition),
        false,
        &mut |_| emitted += 1,
    )
    .expect("waits");

    assert_eq!(emitted, 6, "the triggering frame is emitted, then it stops");
    assert_eq!(clock.now().frames(), 6);
    assert_eq!(report.n_frames, 6, "the journal records realized frames");
}

#[test]
fn skipped_wait_advances_time_without_emitting() {
    let run = |skip: bool| -> (Vec<u8>, usize, i64) {
        let mut stage = Stage::new();
        let mob = square(&mut stage);
        stage.shift(mob, [1.0, 0.0, 0.0]);
        stage.add_to_scene(mob).expect("rooted");
        stage
            .add_dt_updater(
                mob,
                |stage, me, dt| {
                    let x = stage.get_center(me)[0];
                    stage.shift(me, [dt * (1.0 + x), 0.0, 0.0]);
                },
                false,
            )
            .expect("updater");
        let mut clock = RationalFrameClock::new(30).expect("fps");
        let mut emitted = 0usize;
        wait_segment(&mut stage, &mut clock, &rng(), 1.0, None, skip, &mut |_| {
            emitted += 1
        })
        .expect("waits");
        (
            stage.snapshot_bytes().expect("canonical arena"),
            emitted,
            clock.now().frames(),
        )
    };

    let (played, emitted_played, frames_played) = run(false);
    let (skipped, emitted_skipped, frames_skipped) = run(true);
    assert_eq!(played, skipped, "wait skip is state-identical");
    assert_eq!(emitted_played, 30);
    assert_eq!(emitted_skipped, 0);
    assert_eq!(frames_played, 30);
    assert_eq!(frames_skipped, frames_played);
}

// ------------------------------------------------- scene membership edges

#[test]
fn begin_roots_an_unrooted_animated_mobject() {
    let mut stage = Stage::new();
    let mob = square(&mut stage); // never added to the scene
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert!(stage.roots().contains(&mob), "begin adds it to the scene");
}

#[test]
fn remover_animations_leave_the_scene_at_finish() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    animations[0].state_mut().config.remover = true;
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert!(
        !stage.roots().contains(&mob),
        "clean_up_from_scene removes the remover's mobject"
    );
    assert!(
        stage.contains(mob),
        "removal is scene membership, not death"
    );
}

// ----------------------------------------------------- FramePacket freeze

#[test]
fn packets_are_isolated_from_later_scene_mutation() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 2.0, 1.0)];
    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    // Mutate the live scene hard after capture.
    stage.set_x(mob, 999.0);
    stage.update(1.0);

    // A held packet still reconstructs its frame's exact front-end state.
    let mid = &packets[14]; // frame 15, alpha 0.5
    assert_eq!(mid.segment_frame(), 15);
    stage.restore(mid.state());
    assert!(
        (stage.get_center(mob)[0] - 1.0).abs() < 1e-5,
        "the frozen frame is alpha 0.5, untouched by later writes"
    );
}

#[test]
fn packet_freeze_cost_is_o_touched() {
    let mut stage = Stage::new();
    let mobs: Vec<Mob> = (0..16).map(|_| square(&mut stage)).collect();
    for &m in &mobs {
        stage.add_to_scene(m).expect("rooted");
    }
    let ids_before: Vec<usize> = mobs
        .iter()
        .map(|m| stage.get(*m).unwrap().buffer.storage_id())
        .collect();

    let clock = RationalFrameClock::new(30).expect("fps");
    let sample = FrameSample {
        frame: 1,
        time: RationalTime::zero(30) + 1,
        alpha: 1.0 / 30.0,
    };
    let _packet = FramePacket::freeze(&stage, &clock, &rng(), &sample);

    // Freezing copied nothing.
    for (m, id) in mobs.iter().zip(&ids_before) {
        assert_eq!(stage.get(*m).unwrap().buffer.storage_id(), *id);
    }
    // Affine motion touches the independent placement and therefore does not
    // unshare object-space record storage at all.
    stage.set_x(mobs[3], 5.0);
    for (m, id) in mobs.iter().zip(&ids_before) {
        assert_eq!(
            stage.get(*m).unwrap().buffer.storage_id(),
            *id,
            "placement must not clone the RecordBuffer"
        );
    }

    // A real object-space geometry write still unshares exactly that entry.
    let points = stage.get_points(mobs[3]).expect("points");
    stage.set_points(mobs[3], &points).expect("geometry write");
    for (i, (m, id)) in mobs.iter().zip(&ids_before).enumerate() {
        let now = stage.get(*m).unwrap().buffer.storage_id();
        if i == 3 {
            assert_ne!(now, *id, "touched entry unshares");
        } else {
            assert_eq!(now, *id, "untouched entries still share");
        }
    }
}

#[test]
fn packet_rng_forks_are_pure_and_derivable() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let root = rng();
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.2)];
    let mut packets: Vec<FramePacket> = Vec::new();
    play_segment(
        &mut stage,
        &mut clock,
        &root,
        &mut animations,
        false,
        &mut |p| packets.push(p),
    )
    .expect("plays");

    let packet = &packets[2];
    // The packet's fork is exactly the keyed per-frame fork of the root.
    let mut from_packet = packet.rng_fork("streamlines");
    let mut direct = root
        .substream("streamlines")
        .fork_frame(packet.frame_index().cast_unsigned());
    for _ in 0..64 {
        assert_eq!(from_packet.next_u64(), direct.next_u64());
    }
    // Clones agree (pure derivation, no hidden shared state).
    let clone = packet.clone();
    let mut a = packet.rng_fork("dots");
    let mut b = clone.rng_fork("dots");
    for _ in 0..64 {
        assert_eq!(a.next_u64(), b.next_u64());
    }
}
