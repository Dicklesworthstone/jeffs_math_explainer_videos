//! A real imperative Scene drives both the observer and the FMTL sink.
use std::cell::Cell;
use std::rc::Rc;

use fmn_anim::{FramePacket, RationalFrameClock, prepare_animation};
use fmn_core::rng::RngRoot;
use fmn_hash::sha256;
use fmn_mobject::{AnimateArgs, Mobject, Snapshot, Stage};
use fmn_scene::recording::{RecordingError, SceneBundleRecorder};
use fmn_scene::{
    BundleError, BundleExportLimits, BundleSegmentKind, CaptureReason, IntegrationError,
    LifecycleEvent, PlayOverrides, RuntimeConfig, Scene, SceneSink, TimelineBundle,
};

struct ObservedRecording {
    recorder: SceneBundleRecorder,
    frames: Vec<Vec<u8>>,
}

impl SceneSink for ObservedRecording {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        self.recorder.event(event)
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        self.frames.push(packet.state().to_bytes().unwrap());
        self.recorder.capture(reason, packet)
    }
}

fn scene(fps: u32) -> Scene {
    let config = RuntimeConfig {
        fps,
        ..RuntimeConfig::default()
    };
    Scene::new(config, 19).unwrap()
}

#[test]
fn imperative_play_wait_and_show_replay_every_observed_snapshot_without_callbacks() {
    let mut source = scene(30);
    let moving = source
        .add_mobject(Mobject::from_points(&[[0.0, 0.0, 0.0]]))
        .unwrap();
    let following = source
        .add_mobject(Mobject::from_points(&[[0.0, 1.0, 0.0]]))
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let counter = calls.clone();
    source
        .stage_mut()
        .add_updater(
            following,
            move |stage, me| {
                counter.set(counter.get() + 1);
                // This depends on post-interpolation geometry, not just endpoints.
                let x = stage.get_center(moving)[0];
                stage.set_x(me, x * x);
            },
            false,
        )
        .unwrap();
    let animation = prepare_animation(
        moving
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.1),
                rate_func: Some(fmn_core::rate::there_and_back),
                ..AnimateArgs::default()
            })
            .unwrap()
            .shift([2.0, 0.0, 0.0])
            .unwrap(),
        source.stage_mut(),
    )
    .unwrap();
    let mut sink = ObservedRecording {
        recorder: SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap(),
        frames: Vec::new(),
    };
    source
        .play(vec![animation], PlayOverrides::default(), &mut sink)
        .unwrap();
    source.wait(Some(0.0), &mut sink).unwrap();
    source.show(&mut sink).unwrap();
    source.wait(Some(0.1), &mut sink).unwrap();
    let calls_before_playback = calls.get();
    assert!(calls_before_playback > 0);
    let artifact = sink.recorder.finish().unwrap();
    assert_eq!(artifact.digest, sha256(&artifact.bytes));
    assert_eq!(artifact.frame_count as usize, sink.frames.len());
    assert_eq!(artifact.segment_count, 4);
    let bundle = TimelineBundle::from_bytes(&artifact.bytes).unwrap();
    for segment in 0..4 {
        assert_eq!(
            bundle.segment_kind(segment),
            Some(BundleSegmentKind::Stateful)
        );
    }
    // Reverse/random-access playback must not execute the original updater.
    // The persist honesty clause (fmn-mobject persist.rs): updater callables
    // never serialize, so a replayed stage carries the observed state with no
    // callables, and its re-encoding is the observed capture decoded and
    // re-encoded — not the live bytes, which also name the bound updater
    // (fm-sa9o: 9 bytes = one `(u64 id, u8 kind)` updater slot). Compare to
    // that contract, and prove the capture did record the follow-updater.
    for index in (0..artifact.frame_count).rev() {
        let replayed_stage = bundle.stage_at(index).unwrap();
        let replayed = replayed_stage.snapshot().to_bytes().unwrap();
        let observed = Snapshot::from_bytes(&sink.frames[index as usize], &replayed_stage)
            .expect("the observed capture decodes");
        assert_eq!(
            observed.updaters.entries.len(),
            1,
            "frame {index}: the live capture names exactly the follow-updater's mobject"
        );
        assert_eq!(observed.updaters.entries[0].1.len(), 1);
        let observed_without_callables = observed.snapshot.materialize().snapshot();
        assert_eq!(
            replayed,
            observed_without_callables.to_bytes().unwrap(),
            "frame {index}: replay differs from the observed state"
        );
    }
    // Planted negative: the comparison sees real state. Replaying frame 0
    // against a later observed frame whose state differs (the follower moved)
    // must fail, so a dropped or stale state field cannot pass unnoticed.
    let replayed_first = bundle.stage_at(0).unwrap();
    let decode = |bytes: &[u8]| {
        Snapshot::from_bytes(bytes, &replayed_first)
            .unwrap()
            .snapshot
            .materialize()
            .snapshot()
            .to_bytes()
            .unwrap()
    };
    let first = decode(&sink.frames[0]);
    let different = sink
        .frames
        .iter()
        .map(|bytes| decode(bytes))
        .find(|bytes| *bytes != first)
        .expect("the follow-updater changes the observed state across frames");
    assert_ne!(replayed_first.snapshot().to_bytes().unwrap(), different);
    assert_eq!(calls.get(), calls_before_playback);
}

#[test]
fn clock_rounding_and_early_stop_preserve_emitted_counts() {
    for fps in [1, 3, 10, 24, 30, 60, 144, 1001] {
        let mut source = scene(fps);
        let mut sink = SceneBundleRecorder::new(fps, BundleExportLimits::default()).unwrap();
        source.wait(Some(0.1), &mut sink).unwrap();
        source.wait_until(1.0, &mut |_| true, &mut sink).unwrap();
        let observed = sink.frame_count();
        let artifact = sink.finish().unwrap();
        assert_eq!(u64::from(artifact.frame_count), observed);
        assert_eq!(
            TimelineBundle::from_bytes(&artifact.bytes).unwrap().fps(),
            fps
        );
    }
}

#[test]
fn static_terminal_capture_is_explicit_and_deterministic() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[1.0, 2.0, 0.0]]));
    stage.add_to_scene(mob).unwrap();
    let export = || {
        let mut sink = SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap();
        sink.capture_terminal_still(&stage).unwrap();
        sink.finish().unwrap()
    };
    let a = export();
    let b = export();
    assert_eq!(a.bytes, b.bytes);
    assert_eq!(a.frame_count, 1);
    let restored = TimelineBundle::from_bytes(&a.bytes)
        .unwrap()
        .stage_at(0)
        .unwrap();
    assert_eq!(
        restored.snapshot().to_bytes().unwrap(),
        stage.snapshot().to_bytes().unwrap()
    );
}

#[test]
fn frame_limit_is_typed_sticky_and_never_yields_a_partial_bundle() {
    let mut source = scene(30);
    let limits = BundleExportLimits {
        max_frames: 1,
        ..BundleExportLimits::default()
    };
    let mut sink = SceneBundleRecorder::new(30, limits).unwrap();
    assert!(source.wait(Some(0.1), &mut sink).is_err());
    assert_eq!(sink.frame_count(), 1);
    assert!(source.show(&mut sink).is_err());
    assert!(matches!(
        sink.finish(),
        Err(RecordingError::Bundle(BundleError::FrameLimitExceeded {
            frames: 2,
            max_frames: 1
        }))
    ));
}

#[test]
fn capture_budget_includes_destination_tables() {
    let stage = Stage::new();
    let snapshot_bytes = stage.snapshot().to_bytes().unwrap().len();
    let limits = BundleExportLimits {
        max_frames: 10,
        max_capture_bytes: snapshot_bytes,
    };
    let mut sink = SceneBundleRecorder::new(30, limits).unwrap();
    assert!(sink.capture_terminal_still(&stage).is_err());
    assert!(matches!(
        sink.finish(),
        Err(RecordingError::Bundle(
            BundleError::CaptureLimitExceeded { .. }
        ))
    ));
}

#[test]
fn unsupported_capture_and_clock_mismatch_fail_closed() {
    let stage = Stage::new();
    let clock = RationalFrameClock::new(30).unwrap();
    let rng = RngRoot::from_seed(0);
    let reasons = [
        CaptureReason::SkippedPreview,
        CaptureReason::PresenterHold,
        CaptureReason::Segment,
    ];
    for reason in reasons {
        let mut sink = SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap();
        let packet = FramePacket::freeze_barrier(&stage, &clock, &rng);
        assert!(sink.capture(reason, packet).is_err());
        assert!(sink.finish().is_err());
    }
    let mut source = scene(24);
    let mut sink = SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap();
    assert!(source.wait(Some(0.25), &mut sink).is_err());
    assert!(sink.finish().is_err());
}

#[test]
fn zero_frame_budget_refuses_even_terminal_stills() {
    let limits = BundleExportLimits {
        max_frames: 0,
        ..BundleExportLimits::default()
    };
    let mut sink = SceneBundleRecorder::new(30, limits).unwrap();
    assert!(sink.capture_terminal_still(&Stage::new()).is_err());
    assert_eq!(sink.frame_count(), 0);
    assert!(sink.finish().is_err());
    assert!(SceneBundleRecorder::new(0, BundleExportLimits::default()).is_err());
}

#[derive(Clone)]
enum TraceItem {
    Event(LifecycleEvent),
    Capture(CaptureReason, FramePacket),
}

#[derive(Default)]
struct CaptureTrace(Vec<TraceItem>);

impl SceneSink for CaptureTrace {
    fn event(&mut self, event: LifecycleEvent) -> Result<(), IntegrationError> {
        self.0.push(TraceItem::Event(event));
        Ok(())
    }

    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        self.0.push(TraceItem::Capture(reason, packet));
        Ok(())
    }
}

fn recorded_trace(items: Vec<TraceItem>) -> Result<(), RecordingError> {
    let mut recorder = SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap();
    // Continue after adapter errors to verify that ignored failures stay sticky.
    for item in items {
        let _ = match item {
            TraceItem::Event(event) => recorder.event(event),
            TraceItem::Capture(reason, packet) => recorder.capture(reason, packet),
        };
    }
    recorder.finish().map(|_| ())
}

#[test]
fn missing_repeated_or_out_of_order_frames_are_not_silently_retimed() {
    let mut source = scene(30);
    let mut trace = CaptureTrace::default();
    source.wait(Some(0.1), &mut trace).unwrap();
    assert!(recorded_trace(trace.0.clone()).is_ok());
    let captures: Vec<usize> = trace
        .0
        .iter()
        .enumerate()
        .filter_map(|(index, item)| matches!(item, TraceItem::Capture(_, _)).then_some(index))
        .collect();
    assert!(captures.len() > 1);

    let mut missing_first = trace.0.clone();
    missing_first.remove(captures[0]);
    assert!(recorded_trace(missing_first).is_err());

    let mut missing_last = trace.0.clone();
    missing_last.remove(*captures.last().unwrap());
    assert!(recorded_trace(missing_last).is_err());

    let mut duplicated = trace.0.clone();
    duplicated.insert(captures[0], duplicated[captures[0]].clone());
    assert!(recorded_trace(duplicated).is_err());

    let mut reordered = trace.0.clone();
    reordered.swap(captures[0], captures[1]);
    assert!(recorded_trace(reordered).is_err());

    let mut sampled_show = trace.0;
    let TraceItem::Capture(reason, _) = &mut sampled_show[captures[0]] else {
        panic!("the trace item must be a capture");
    };
    *reason = CaptureReason::Show;
    assert!(recorded_trace(sampled_show).is_err());
}

#[test]
fn altered_or_missing_segment_boundaries_cannot_finalize() {
    let mut source = scene(30);
    let mut trace = CaptureTrace::default();
    source.wait(Some(0.1), &mut trace).unwrap();
    let finish = trace
        .0
        .iter()
        .position(|item| {
            matches!(item,
                TraceItem::Event(event) if event.phase == fmn_scene::LifecyclePhase::FinishSegment
            )
        })
        .unwrap();

    let mut mismatched = trace.0.clone();
    let TraceItem::Event(event) = &mut mismatched[finish] else {
        panic!("the trace item must be an event");
    };
    event.play_index += 1;
    assert!(recorded_trace(mismatched).is_err());

    let mut unfinished = trace.0;
    unfinished.remove(finish);
    assert!(recorded_trace(unfinished).is_err());
}

#[test]
fn canonical_output_budget_accepts_exact_size_and_refuses_one_byte_less() {
    let stage = Stage::new();
    let record = || {
        let mut recorder = SceneBundleRecorder::new(30, BundleExportLimits::default()).unwrap();
        recorder.capture_terminal_still(&stage).unwrap();
        recorder
    };
    let expected = record().finish().unwrap();
    let exact = record()
        .finish_with_max_bytes(expected.bytes.len())
        .unwrap();
    assert_eq!(exact.bytes, expected.bytes);
    let limit = expected.bytes.len() - 1;
    assert!(matches!(record().finish_with_max_bytes(limit),
        Err(RecordingError::OutputLimit { needed, limit: actual })
            if needed == expected.bytes.len() && actual == limit));
    assert!(matches!(
        record().finish_with_max_bytes(0),
        Err(RecordingError::OutputLimit { limit: 0, .. })
    ));
}
