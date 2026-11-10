//! The segment-purity classifier (fm-3xk acceptance): the effect matrix
//! (each impure construct demotes, each pure construct passes), the
//! demotion path for unknown effects, journal-record contents, and the
//! frame-parallel equivalence core — a pure segment reconstructed
//! out-of-order at simulated {1, 4, 16} workers is bit-identical to the
//! serial emission.

use fmn_anim::frame::{FramePacket, play_segment, wait_segment};
use fmn_anim::purity::{ImpureEffect, Purity, SegmentKind, reconstruct_pure_frame};
use fmn_anim::{
    AnimConfig, AnimError, AnimState, Animation, FrameSegment, RationalFrameClock,
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

fn rng() -> RngRoot {
    RngRoot::from_seed(42)
}

fn stateful_reasons(purity: &Purity) -> &[ImpureEffect] {
    match purity {
        Purity::Pure => &[],
        Purity::Stateful(effects) => effects,
    }
}

// ------------------------------------------------------ the effect matrix

#[test]
fn pure_method_animation_segment_classifies_pure() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 2.0, 0.2)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");

    assert_eq!(report.kind, SegmentKind::Play);
    assert_eq!(report.purity, Purity::Pure);
    assert!(
        report.begin_state.is_some(),
        "a pure segment carries its worker input"
    );
    // BN-02: the f64 0.2 is strictly greater than 1/5, and six frames
    // genuinely do not cover it — exact upward rounding gives 7.
    assert_eq!(report.n_frames, 7);
}

#[test]
fn dt_updater_demotes_with_the_recorded_reason() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    stage
        .add_dt_updater(mob, |_stage, _me, _dt| {}, false)
        .expect("registers");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");

    assert!(stateful_reasons(&report.purity).contains(&ImpureEffect::DtUpdater));
    assert!(report.begin_state.is_none(), "stateful carries no snapshot");
}

#[test]
fn scene_updater_demotes_even_on_a_bystander_mobject() {
    let mut stage = Stage::new();
    let animated = square(&mut stage);
    let bystander = square(&mut stage);
    stage.add_to_scene(animated).expect("rooted");
    stage.add_to_scene(bystander).expect("rooted");
    stage
        .add_updater(bystander, |_stage, _me| {}, false)
        .expect("registers");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, animated, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert!(stateful_reasons(&report.purity).contains(&ImpureEffect::SceneUpdater));
}

#[test]
fn always_redraw_binding_demotes_through_the_updater_probe() {
    let mut stage = Stage::new();
    let animated = square(&mut stage);
    stage.add_to_scene(animated).expect("rooted");
    let container =
        stage.always_redraw(|stage| stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]])));
    stage.add_to_scene(container).expect("rooted");

    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, animated, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert!(
        stateful_reasons(&report.purity).contains(&ImpureEffect::SceneUpdater),
        "always_redraw binds as an updater and must demote: {:?}",
        report.purity
    );
}

#[test]
fn unknown_animations_demote_by_default() {
    // The conservative rule (R20): an animation that does not declare a
    // pure signature demotes its segment even with zero updaters anywhere.
    struct Custom {
        state: AnimState,
    }
    impl Animation for Custom {
        fn state(&self) -> &AnimState {
            &self.state
        }
        fn state_mut(&mut self) -> &mut AnimState {
            &mut self.state
        }
        fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _alpha: f64) {}
    }

    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations: Vec<Box<dyn Animation>> = vec![Box::new(Custom {
        state: AnimState::new(mob, AnimConfig::default()),
    })];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert_eq!(
        stateful_reasons(&report.purity),
        &[ImpureEffect::UnclassifiedAnimation]
    );
}

#[test]
fn updater_on_the_animation_owned_target_demotes() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let built = mob
        .animate()
        .shift([1.0, 0.0, 0.0])
        .and_then(|b| b.build(&mut stage))
        .expect("builds");
    // The target is not rooted, but the animation ticks it every frame.
    stage
        .add_dt_updater(built.target, |_stage, _me, _dt| {}, false)
        .expect("registers");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![prepare_animation(built, &mut stage).expect("prepares")];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert!(stateful_reasons(&report.purity).contains(&ImpureEffect::DtUpdater));
}

#[test]
fn a_static_value_tracker_does_not_demote() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let tracker = stage.add_value_tracker(3.5);
    stage.add_to_scene(tracker).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    assert_eq!(
        report.purity,
        Purity::Pure,
        "tracker state without updaters is static during the segment"
    );
}

// ------------------------------------------------------------------ waits

#[test]
fn wait_classification_matrix() {
    // Pure wait: nothing changes frame to frame.
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let report = wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        0.2,
        None,
        false,
        &mut |_| {},
    )
    .expect("waits");
    assert_eq!(report.kind, SegmentKind::Wait);
    assert_eq!(report.purity, Purity::Pure);
    assert!(report.begin_state.is_some());

    // A stop condition demotes by vocabulary.
    let mut condition = |_stage: &Stage| true;
    let report = wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        0.2,
        Some(&mut condition),
        false,
        &mut |_| {},
    )
    .expect("waits");
    assert!(stateful_reasons(&report.purity).contains(&ImpureEffect::StopCondition));

    // An updater demotes the wait like any segment.
    stage
        .add_dt_updater(mob, |_stage, _me, _dt| {}, false)
        .expect("registers");
    let report = wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        0.2,
        None,
        false,
        &mut |_| {},
    )
    .expect("waits");
    assert!(stateful_reasons(&report.purity).contains(&ImpureEffect::DtUpdater));
}

// ------------------------------------------ frame-parallel equivalence

/// Bit-exact fingerprint of a packet: header fields plus every record field and
/// affine placement of every rooted family member.
fn fingerprint(stage: &mut Stage, packet: &FramePacket) -> Vec<u32> {
    stage.restore(packet.state());
    let mut bits: Vec<u32> = Vec::new();
    bits.push(u32::try_from(packet.segment_frame()).expect("small"));
    bits.extend(
        packet
            .alpha()
            .to_bits()
            .to_le_bytes()
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().expect("4 bytes"))),
    );
    for root in stage.roots().to_vec() {
        for member in stage.family(root) {
            let entry = stage.get(member).expect("family resolves");
            let fields: Vec<String> = entry
                .buffer
                .schema()
                .fields()
                .iter()
                .map(|f| f.name.clone())
                .collect();
            for field in fields {
                let column = stage
                    .get(member)
                    .and_then(|e| e.buffer.read_column(&field))
                    .expect("column reads");
                bits.extend(column.iter().map(|v| v.to_bits()));
            }
            for coefficient in stage
                .placement(member)
                .expect("family resolves")
                .coefficients()
            {
                bits.extend(
                    coefficient
                        .to_bits()
                        .to_le_bytes()
                        .chunks(4)
                        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("4 bytes"))),
                );
            }
        }
    }
    bits
}

#[test]
fn pure_segment_reconstructs_bit_identically_at_any_worker_count() {
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let root = rng();
    let mut animations = vec![shift_animation(&mut stage, mob, 2.0, 1.0)];

    let mut serial: Vec<FramePacket> = Vec::new();
    let report = play_segment(
        &mut stage,
        &mut clock,
        &root,
        &mut animations,
        false,
        &mut |p| serial.push(p),
    )
    .expect("plays");
    assert_eq!(report.purity, Purity::Pure);
    assert_eq!(serial.len(), 30);

    let serial_prints: Vec<Vec<u32>> = serial.iter().map(|p| fingerprint(&mut stage, p)).collect();

    // Rebuild the sample plan the segment ran under.
    let plan_clock = RationalFrameClock::new(30).expect("fps");
    let segment: FrameSegment = plan_clock.segment(report.run_time).expect("plan");
    let samples: Vec<_> = segment.samples().collect();

    // Simulated worker pools: round-robin frame assignment, workers
    // interleaved — every schedule must produce the serial bits (§10.5's
    // contract: a frame's bits are a function of snapshot + alpha + keyed
    // RNG, never of order).
    for workers in [1usize, 4, 16] {
        let mut order: Vec<usize> = Vec::new();
        for lane in 0..workers {
            for frame in (lane..samples.len()).step_by(workers) {
                order.push(frame);
            }
        }
        for &index in order.iter().rev() {
            let packet = reconstruct_pure_frame(
                &mut stage,
                &mut animations,
                &report,
                &root,
                &samples[index],
            )
            .expect("reconstructs");
            assert_eq!(packet.frame_index(), serial[index].frame_index());
            assert_eq!(packet.time(), serial[index].time());
            assert_eq!(
                stage.time().to_bits(),
                packet.time().to_f64().to_bits(),
                "worker Stage time at frame {index}"
            );
            assert_eq!(
                fingerprint(&mut stage, &packet),
                serial_prints[index],
                "frame {index} at {workers} workers must be bit-identical"
            );
        }
    }
}

#[test]
fn reconstruction_refuses_stateful_and_skipped_segments() {
    // Stateful: a dt updater demotes; no begin state to reconstruct from.
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    stage
        .add_dt_updater(mob, |_stage, _me, _dt| {}, false)
        .expect("registers");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let root = rng();
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &root,
        &mut animations,
        false,
        &mut |_| {},
    )
    .expect("plays");
    let sample = clock
        .segment(0.1)
        .expect("segment")
        .samples()
        .next()
        .expect("nonempty");
    assert_eq!(
        reconstruct_pure_frame(&mut stage, &mut animations, &report, &root, &sample).err(),
        Some(AnimError::SegmentNotPure)
    );

    // Skipped: pure but nothing was emitted, so nothing reconstructs.
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(30).expect("fps");
    let mut animations = vec![shift_animation(&mut stage, mob, 1.0, 0.1)];
    let report = play_segment(
        &mut stage,
        &mut clock,
        &root,
        &mut animations,
        true,
        &mut |_| {},
    )
    .expect("plays");
    assert_eq!(report.purity, Purity::Pure);
    assert!(report.begin_state.is_none());
    assert_eq!(
        reconstruct_pure_frame(&mut stage, &mut animations, &report, &root, &sample).err(),
        Some(AnimError::SegmentNotPure)
    );
}
