//! Timeline acceptance (fm-hfe, §9.4): the compiled schedule, labels, the
//! canonical round trip, and the property the whole design exists for —
//! **`seek(t)` is `play-through-to(t)`**, bit for bit, for pure content.
//!
//! Frame states are compared through the record plane at `f32::to_bits`
//! identity, the same bar the §9.5 frame-parallel equivalence tests use: a
//! sought frame either *is* the played frame or it is a bug.

use fmn_anim::animation::{AnimError, Animation};
use fmn_anim::purity::SegmentKind;
use fmn_anim::timeline::{
    PlannedSegment, Step, TIMELINE_SCHEMA, Timeline, TimelineError, TimelinePlan,
};
use fmn_anim::{
    AnimationGroup, ClockError, FramePacket, RationalFrameClock, Succession, play_segment,
    prepare_animation, wait_segment,
};
use fmn_core::rng::RngRoot;
use fmn_hash::Writer;
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mob, Mobject, Stage};

/// The tests' frame rate. Every authored duration below is an exact binary
/// fraction on this grid, so the schedule is the schedule and BN-02's
/// f64-duration rounding (clock.rs's business) stays out of the assertions.
const FPS: u32 = 8;

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

fn encoded_plan(
    fps: u32,
    segments: &[(SegmentKind, f64, i64, i64)],
    labels: &[(&str, u64, i64)],
) -> Vec<u8> {
    let mut writer = Writer::new(TIMELINE_SCHEMA);
    writer.put_u32(fps);
    writer.put_u32(u32::try_from(segments.len()).expect("small segment table"));
    for &(kind, run_time, base_frame, n_frames) in segments {
        writer.put_u8(match kind {
            SegmentKind::Play => 0,
            SegmentKind::Wait => 1,
        });
        writer.put_f64(run_time);
        writer.put_i64(base_frame);
        writer.put_i64(n_frames);
    }
    writer.put_u32(u32::try_from(labels.len()).expect("small label table"));
    for &(name, segment, frame) in labels {
        writer.put_str(name);
        writer.put_u64(segment);
        writer.put_i64(frame);
    }
    writer.finish().expect("forged plan is correctly framed")
}

/// The record plane and independent placement of one mobject, as raw bits.
fn point_bits(stage: &Stage, mob: Mob) -> Vec<u32> {
    let mut bits: Vec<u32> = stage
        .get(mob)
        .expect("live")
        .buffer
        .read_column("point")
        .expect("point column")
        .iter()
        .map(|v| v.to_bits())
        .collect();
    for coefficient in stage.placement(mob).expect("live").coefficients() {
        bits.extend(
            coefficient
                .to_bits()
                .to_le_bytes()
                .chunks(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("4 bytes"))),
        );
    }
    bits
}

/// Bit-exact fingerprint of a frame: the packet's own header plus every
/// record field of every rooted family member, as raw f32 bits — the §9.5
/// equivalence bar, restored into the packet's own stage (handles are
/// stage-scoped, so a frame is only readable where it was frozen).
fn frame_bits(stage: &mut Stage, packet: &FramePacket) -> Vec<u32> {
    stage.restore(packet.state());
    let mut bits: Vec<u32> = vec![u32::try_from(packet.segment_frame()).expect("small")];
    for root in stage.roots().to_vec() {
        for member in stage.family(root) {
            let fields: Vec<String> = stage
                .get(member)
                .expect("family resolves")
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

/// Author the same three-step timeline every seek test uses: two plays with
/// a wait between them, on one mobject so state accumulates visibly.
fn authored(stage: &mut Stage, fps: u32) -> (Timeline, Mob) {
    let mob = square(stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut timeline = Timeline::new(fps).expect("fps");
    timeline.label("start");
    timeline
        .play(vec![shift_animation(stage, mob, 2.0, 0.5)])
        .expect("play authored");
    timeline.label("hold");
    timeline.wait(0.25).expect("wait authored");
    timeline.label("finale");
    timeline
        .play(vec![shift_animation(stage, mob, -1.0, 0.25)])
        .expect("play authored");
    timeline.label("end");
    (timeline, mob)
}

/// Play a timeline straight through, returning one fingerprint per frame.
fn play_through(timeline: &mut Timeline, stage: &mut Stage) -> Vec<Vec<u32>> {
    let mut packets = Vec::new();
    timeline
        .render(stage, &rng(), &mut |packet| packets.push(packet))
        .expect("renders");
    packets
        .iter()
        .map(|packet| frame_bits(stage, packet))
        .collect()
}

// -------------------------------------------------------------- the plan

#[test]
fn compile_schedules_every_step_on_the_grid() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");

    assert_eq!(plan.fps(), FPS);
    assert_eq!(plan.segments().len(), 3);
    // 0.5 s at 8 fps = 4 frames; 0.25 s = 2 frames.
    let kinds: Vec<SegmentKind> = plan.segments().iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [SegmentKind::Play, SegmentKind::Wait, SegmentKind::Play]
    );
    let counts: Vec<i64> = plan.segments().iter().map(|s| s.n_frames).collect();
    assert_eq!(counts, [4, 2, 2]);
    let bases: Vec<i64> = plan.segments().iter().map(|s| s.base_frame).collect();
    assert_eq!(bases, [0, 4, 6]);
    assert_eq!(plan.total_frames(), 8);
    assert!((plan.duration() - 1.0).abs() < 1e-12);
}

#[test]
fn the_compiled_schedule_is_the_schedule_the_drivers_run() {
    // The plan is derived without touching a stage; the drivers derive the
    // same numbers while running. Any disagreement would make every seek
    // land on the wrong frame, so it is worth asserting directly.
    let mut stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");
    let mut emitted = Vec::new();
    let reports = timeline
        .render(&mut stage, &rng(), &mut |packet| {
            emitted.push(packet.frame_index());
        })
        .expect("renders");

    assert_eq!(reports.len(), plan.segments().len());
    for (report, planned) in reports.iter().zip(plan.segments()) {
        assert_eq!(report.n_frames, planned.n_frames);
        assert_eq!(report.base_frame, planned.base_frame);
        assert_eq!(report.kind, planned.kind);
    }
    assert_eq!(emitted, (1..=plan.total_frames()).collect::<Vec<_>>());
}

#[test]
fn labels_resolve_to_the_frames_they_mark() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");

    assert_eq!(plan.frame_of_label("start"), Some(1));
    assert_eq!(plan.frame_of_label("hold"), Some(5));
    assert_eq!(plan.frame_of_label("finale"), Some(7));
    // A label authored after the last step marks the end.
    assert_eq!(plan.frame_of_label("end"), Some(8));
    assert_eq!(plan.frame_of_label("nowhere"), None);
}

#[test]
fn locate_and_frame_at_time_agree_with_the_grid() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");

    assert_eq!(plan.locate(1), Some((0, 1)));
    assert_eq!(plan.locate(4), Some((0, 4)));
    assert_eq!(plan.locate(5), Some((1, 1)));
    assert_eq!(plan.locate(8), Some((2, 2)));
    assert_eq!(plan.locate(0), None);
    assert_eq!(plan.locate(9), None);

    // Sample k covers ((k-1)/fps, k/fps].
    assert_eq!(plan.frame_at_time(0.0), 1);
    assert_eq!(plan.frame_at_time(1.0 / 8.0), 1);
    assert_eq!(plan.frame_at_time(0.5), 4);
    assert_eq!(plan.frame_at_time(1.0), 8);
    assert_eq!(plan.frame_at_time(99.0), 8, "clamped into the schedule");
    assert_eq!(plan.frame_at_time(f64::NAN), 1, "total, never a panic");
}

#[test]
fn a_public_planned_segment_reports_end_frame_overflow() {
    let ordinary = PlannedSegment {
        kind: SegmentKind::Wait,
        run_time: 0.25,
        base_frame: 2,
        n_frames: 3,
    };
    assert_eq!(ordinary.end_frame(), Some(5));

    let overflowing = PlannedSegment {
        kind: SegmentKind::Wait,
        run_time: 1.0,
        base_frame: i64::MAX,
        n_frames: 1,
    };
    assert_eq!(overflowing.end_frame(), None);
}

// -------------------------------------------------------- canonical bytes

#[test]
fn a_plan_round_trips_through_its_canonical_bytes() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");

    let bytes = plan.to_bytes().expect("encodes");
    let decoded = TimelinePlan::from_bytes(&bytes).expect("decodes");
    assert_eq!(decoded, plan);
    assert_eq!(decoded.to_bytes().expect("re-encodes"), bytes);
    assert_eq!(
        decoded.content_id().expect("addresses"),
        plan.content_id().expect("addresses")
    );
    // The schedule a loaded plan describes is the schedule the authored one
    // described: same frames, same labels, no scene code involved.
    assert_eq!(decoded.total_frames(), plan.total_frames());
    assert_eq!(decoded.frame_of_label("finale"), Some(7));
    assert_eq!(decoded.locate(5), Some((1, 1)));
}

#[test]
fn a_changed_schedule_changes_the_content_id() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");

    let mut other = Timeline::new(FPS).expect("fps");
    other.wait(0.25).expect("authored");
    let other = other.compile().expect("compiles");
    assert_ne!(
        plan.content_id().expect("addresses"),
        other.content_id().expect("addresses")
    );
}

#[test]
fn corrupt_bytes_are_a_named_error() {
    let mut stage = Stage::new();
    let (timeline, _mob) = authored(&mut stage, FPS);
    let plan = timeline.compile().expect("compiles");
    let bytes = plan.to_bytes().expect("encodes");

    let mut flipped = bytes.clone();
    let last = flipped.len() - 40;
    flipped[last] ^= 0xff;
    assert!(matches!(
        TimelinePlan::from_bytes(&flipped),
        Err(TimelineError::Serial(_))
    ));

    assert!(TimelinePlan::from_bytes(&[]).is_err());
    assert!(TimelinePlan::from_bytes(&bytes[..bytes.len() - 1]).is_err());
    assert_eq!(TIMELINE_SCHEMA.magic, *b"FMNA");
}

#[test]
fn decoded_plans_refuse_impossible_clock_segment_and_label_state() {
    let cases = [
        (encoded_plan(0, &[], &[]), "fps"),
        (
            encoded_plan(8, &[(SegmentKind::Wait, f64::INFINITY, 0, 0)], &[]),
            "segment run time",
        ),
        (
            encoded_plan(8, &[(SegmentKind::Wait, 0.25, 1, 2)], &[]),
            "segment base frame",
        ),
        (
            encoded_plan(8, &[(SegmentKind::Wait, 0.25, 0, 3)], &[]),
            "segment frame count",
        ),
        (
            encoded_plan(8, &[(SegmentKind::Wait, 0.25, 0, 2)], &[("outside", 2, 2)]),
            "label segment index",
        ),
        (
            encoded_plan(8, &[(SegmentKind::Wait, 0.25, 0, 2)], &[("wrong", 0, 2)]),
            "label frame",
        ),
        (
            encoded_plan(
                8,
                &[
                    (SegmentKind::Wait, 0.125, 0, 1),
                    (SegmentKind::Wait, 0.125, 1, 1),
                ],
                &[("later", 1, 2), ("earlier", 0, 1)],
            ),
            "label order",
        ),
    ];

    for (bytes, expected) in cases {
        assert!(matches!(
            TimelinePlan::from_bytes(&bytes),
            Err(TimelineError::Malformed(actual)) if actual == expected
        ));
    }
}

#[test]
fn cumulative_frame_overflow_is_a_typed_refusal_for_compile_and_plan_load() {
    let run_time = (i64::MAX / 2) as f64;
    let n_frames = RationalFrameClock::new(1)
        .expect("fps")
        .segment(run_time)
        .expect("one segment is representable")
        .n_frames();
    assert!(n_frames.checked_add(n_frames).is_none());

    let mut timeline = Timeline::new(1).expect("fps");
    timeline.wait(run_time).expect("first step");
    timeline
        .wait(run_time)
        .expect("second step is individually valid");
    assert_eq!(
        timeline.compile().unwrap_err(),
        AnimError::Clock(ClockError::FrameCounterOverflow)
    );

    let bytes = encoded_plan(
        1,
        &[
            (SegmentKind::Wait, run_time, 0, n_frames),
            (SegmentKind::Wait, run_time, n_frames, n_frames),
        ],
        &[],
    );
    assert!(matches!(
        TimelinePlan::from_bytes(&bytes),
        Err(TimelineError::Malformed("segment frame range"))
    ));
}

#[test]
fn labels_on_zero_frame_tails_resolve_to_the_existing_boundary() {
    let mut empty = Timeline::new(FPS).expect("fps");
    empty.label("only");
    empty.wait(0.0).expect("zero-frame wait");
    let empty = empty.compile().expect("compiles");
    assert_eq!(empty.total_frames(), 0);
    assert_eq!(empty.frame_of_label("only"), Some(0));

    let mut tail = Timeline::new(FPS).expect("fps");
    tail.wait(0.25).expect("two frames");
    tail.label("tail");
    tail.wait(0.0).expect("zero-frame tail");
    let tail = tail.compile().expect("compiles");
    assert_eq!(tail.total_frames(), 2);
    assert_eq!(tail.frame_of_label("tail"), Some(2));
    assert_eq!(
        TimelinePlan::from_bytes(&tail.to_bytes().expect("encodes")).expect("decodes"),
        tail
    );
}

// ------------------------------------------------------ seek == playthrough

#[test]
fn seek_equals_play_through_for_pure_content() {
    let mut stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut stage, FPS);
    let played = play_through(&mut timeline, &mut stage);
    let total = timeline.compile().expect("compiles").total_frames();
    assert_eq!(played.len(), usize::try_from(total).expect("frames"));

    for frame in 1..=total {
        let packet = timeline.seek(&mut stage, &rng(), frame).expect("seeks");
        assert_eq!(
            packet.frame_index(),
            frame,
            "the packet knows which frame it is"
        );
        assert_eq!(
            frame_bits(&mut stage, &packet),
            played[usize::try_from(frame - 1).expect("index")],
            "frame {frame} drifted between seek and play-through"
        );
    }
}

#[test]
fn seek_is_repeatable_and_direction_independent() {
    let mut stage = Stage::new();
    let (mut timeline, mob) = authored(&mut stage, FPS);

    timeline.seek(&mut stage, &rng(), 3).expect("seeks");
    let forward = point_bits(&stage, mob);
    timeline.seek(&mut stage, &rng(), 7).expect("seeks");
    timeline.seek(&mut stage, &rng(), 3).expect("seeks back");
    assert_eq!(stage.time().to_bits(), (3.0 / f64::from(FPS)).to_bits());
    assert_eq!(
        point_bits(&stage, mob),
        forward,
        "backwards lands identically"
    );
    timeline.seek(&mut stage, &rng(), 3).expect("seeks again");
    assert_eq!(point_bits(&stage, mob), forward, "and is idempotent");
}

#[test]
fn seek_lands_inside_a_wait_segment() {
    let mut stage = Stage::new();
    let (mut timeline, mob) = authored(&mut stage, FPS);
    let after_play = {
        timeline.seek(&mut stage, &rng(), 4).expect("seeks");
        point_bits(&stage, mob)
    };
    // Nothing moves during a wait with no updaters.
    for frame in 5..=6 {
        timeline.seek(&mut stage, &rng(), frame).expect("seeks");
        assert_eq!(point_bits(&stage, mob), after_play, "frame {frame}");
    }
}

#[test]
fn seek_replays_a_stateful_segment_frame_by_frame() {
    // A dt-updater demotes the segment (§9.5), so there is no snapshot
    // shortcut: the frames *are* the state, and seek must replay them. The
    // updater accumulates every step, which is exactly what a
    // reconstruct-from-alpha shortcut would get wrong.
    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    stage
        .add_dt_updater(
            mob,
            |stage, me, dt| {
                if let Some(entry) = stage.get_mut(me)
                    && let Some(mut point) = entry.buffer.read(0, "point")
                {
                    point[1] += dt as f32;
                    entry.buffer.write(0, "point", &point);
                }
            },
            false,
        )
        .expect("registers");

    let mut timeline = Timeline::new(FPS).expect("fps");
    timeline
        .play(vec![shift_animation(&mut stage, mob, 2.0, 0.5)])
        .expect("authored");
    let plan = timeline.compile().expect("compiles");
    assert_eq!(plan.total_frames(), 4);

    let played = play_through(&mut timeline, &mut stage);
    for frame in 1..=plan.total_frames() {
        let packet = timeline.seek(&mut stage, &rng(), frame).expect("seeks");
        assert_eq!(
            stage.time().to_bits(),
            (frame as f64 / f64::from(FPS)).to_bits(),
            "stage clock at stateful frame {frame}"
        );
        assert_eq!(
            frame_bits(&mut stage, &packet),
            played[usize::try_from(frame - 1).expect("index")],
            "stateful frame {frame}"
        );
    }
}

#[test]
fn seek_records_checkpoints_as_it_passes_them() {
    let mut stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut stage, FPS);
    assert_eq!(timeline.checkpointed_segments(), Vec::<usize>::new());
    timeline.seek(&mut stage, &rng(), 7).expect("seeks");
    // The base plus the two boundaries it crossed on the way.
    assert_eq!(timeline.checkpointed_segments(), vec![0, 1, 2]);
    timeline.clear_checkpoints();
    assert_eq!(timeline.checkpointed_segments(), vec![0]);
}

#[test]
fn seek_outside_the_schedule_is_a_named_error() {
    let mut stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut stage, FPS);
    assert_eq!(
        timeline.seek(&mut stage, &rng(), 9).err(),
        Some(AnimError::SeekOutOfRange { frame: 9, total: 8 })
    );
    assert_eq!(
        timeline.seek(&mut stage, &rng(), 0).err(),
        Some(AnimError::SeekOutOfRange { frame: 0, total: 8 })
    );
}

// -------------------------------------------------------------- authoring

#[test]
fn a_timeline_compiles_to_the_same_primitives_a_scene_calls() {
    // The load-bearing claim: an authored timeline is sugar. Rendering one
    // and calling the drivers by hand produce the same frames.
    let mut sugar_stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut sugar_stage, FPS);
    let sugar = play_through(&mut timeline, &mut sugar_stage);

    let mut stage = Stage::new();
    let mob = square(&mut stage);
    stage.add_to_scene(mob).expect("rooted");
    let mut clock = RationalFrameClock::new(FPS).expect("fps");
    let mut packets = Vec::new();
    let mut collect = |packet: FramePacket| packets.push(packet);
    // Authored up front, exactly as the timeline authors them — `.animate`
    // freezes its target when the chain is prepared, so both sides must
    // prepare at the same moment or they are not the same scene.
    let mut first = vec![shift_animation(&mut stage, mob, 2.0, 0.5)];
    let mut second = vec![shift_animation(&mut stage, mob, -1.0, 0.25)];
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut first,
        false,
        &mut collect,
    )
    .expect("plays");
    wait_segment(
        &mut stage,
        &mut clock,
        &rng(),
        0.25,
        None,
        false,
        &mut collect,
    )
    .expect("waits");
    play_segment(
        &mut stage,
        &mut clock,
        &rng(),
        &mut second,
        false,
        &mut collect,
    )
    .expect("plays");
    let by_hand: Vec<Vec<u32>> = packets
        .iter()
        .map(|packet| frame_bits(&mut stage, packet))
        .collect();

    assert_eq!(sugar, by_hand);
}

#[test]
fn a_composition_is_one_timeline_step() {
    // Composition operators are animations, so a whole composed tree is a
    // single step — the two halves of §9.4 meeting.
    let mut stage = Stage::new();
    let a = square(&mut stage);
    let b = square(&mut stage);
    stage.add_to_scene(a).expect("rooted");
    stage.add_to_scene(b).expect("rooted");
    let members = vec![
        shift_animation(&mut stage, a, 1.0, 1.0),
        shift_animation(&mut stage, b, 1.0, 1.0),
    ];
    let group = AnimationGroup::with_lag_ratio(&mut stage, members, 0.5).expect("group builds");
    let first = shift_animation(&mut stage, a, -1.0, 0.5);
    let succession =
        Succession::new(&mut stage, vec![first, Box::new(group)]).expect("succession builds");

    let mut timeline = Timeline::new(30).expect("fps");
    timeline.play(vec![Box::new(succession)]).expect("authored");
    let plan = timeline.compile().expect("compiles");
    // 0.5 s, then the group's 1.5 s timeline: 2 s ⇒ 60 frames.
    assert_eq!(plan.segments().len(), 1);
    assert_eq!(plan.total_frames(), 60);

    let mut count = 0;
    timeline
        .render(&mut stage, &rng(), &mut |_packet| count += 1)
        .expect("renders");
    assert_eq!(count, 60);
}

#[test]
fn an_empty_play_step_is_refused_by_name() {
    let mut timeline = Timeline::new(FPS).expect("fps");
    assert_eq!(
        timeline.play(Vec::new()).err(),
        Some(AnimError::EmptyComposition)
    );
    assert!(timeline.is_empty());
}

#[test]
fn authoring_invalidates_recorded_checkpoints() {
    let mut stage = Stage::new();
    let (mut timeline, _mob) = authored(&mut stage, FPS);
    timeline.seek(&mut stage, &rng(), 7).expect("seeks");
    assert!(timeline.checkpointed_segments().len() > 1);
    timeline.wait(0.25).expect("authored");
    assert_eq!(
        timeline.checkpointed_segments(),
        vec![0],
        "a changed schedule cannot keep states recorded under the old one"
    );
    assert!(matches!(timeline.steps().last(), Some(Step::Wait(_))));
}
