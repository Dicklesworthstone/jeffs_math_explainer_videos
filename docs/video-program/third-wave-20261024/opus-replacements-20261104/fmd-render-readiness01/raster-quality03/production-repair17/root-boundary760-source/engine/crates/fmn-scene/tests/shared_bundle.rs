//! Compiled frame jobs must preserve the existing reader, purity and clock.

use std::cell::Cell;
use std::rc::Rc;

use fmn_anim::{Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::timeline_bundle::{
    BundleReadError, BundleSegmentKind, SharedTimelineBundle, TimelineBundle, TimelineFrameCache,
    TimelineFrameJob, export_timeline_bundle,
};

fn export(rate_func: fn(f64) -> f64, duration: f64, fps: u32) -> Vec<u8> {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]));
    let tracker = stage.add_complex_value_tracker(2.0, -3.0);
    stage.add_many_to_scene(&[mob, tracker]).unwrap();
    let args = AnimateArgs {
        run_time: Some(duration),
        rate_func: Some(rate_func),
        ..AnimateArgs::default()
    };
    let builders = [
        mob.animate()
            .set_anim_args(args)
            .unwrap()
            .shift([2.0, 1.0, 0.0])
            .unwrap(),
        tracker
            .animate()
            .set_anim_args(args)
            .unwrap()
            .set_complex_value(6.0, 5.0)
            .unwrap(),
    ];
    let animations = builders
        .into_iter()
        .map(|builder| prepare_animation(builder, &mut stage).unwrap())
        .collect();
    let mut timeline = Timeline::new(fps).unwrap();
    timeline.play(animations).unwrap();
    export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(7)).unwrap()
}

fn state(stage: &Stage) -> (Vec<u8>, u64) {
    (stage.snapshot().to_bytes().unwrap(), stage.time().to_bits())
}

#[test]
fn owned_jobs_are_send_sync_and_outlive_the_reader_in_reverse_parallel_order() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SharedTimelineBundle>();
    assert_send_sync::<TimelineFrameJob>();
    for rate_func in [rate::linear as fn(f64) -> f64, rate::there_and_back] {
        let bytes = export(rate_func, 1.0, 8);
        let serial = TimelineBundle::from_bytes(&bytes).unwrap();
        let expected: Vec<_> = (0..serial.frame_count())
            .map(|index| state(&serial.stage_at(index).unwrap()))
            .collect();
        let kind = serial.segment_kind(0).unwrap();
        let shared = serial.into_shared().unwrap();
        assert_eq!(shared.segment_kind(0), Some(kind));
        let jobs: Vec<_> = (0..shared.frame_count())
            .rev()
            .map(|index| shared.frame_job(index).unwrap())
            .collect();
        drop(shared);
        drop(bytes);
        let handles: Vec<_> = jobs
            .into_iter()
            .map(|job| {
                std::thread::spawn(move || {
                    let mut cache = TimelineFrameCache::default();
                    let first = cache.materialize(&job).unwrap();
                    let before = state(&first);
                    let mut mutated = first;
                    mutated.shift(mutated.roots()[0], [999.0, 0.0, 0.0]);
                    assert_eq!(state(&cache.materialize(&job).unwrap()), before);
                    (job.index() as usize, before)
                })
            })
            .collect();
        for handle in handles {
            let (index, actual) = handle.join().unwrap();
            assert_eq!(actual, expected[index]);
        }
    }
}

#[test]
fn pure_endpoints_are_decoded_once_and_binary64_frame_boundaries_stay_exact() {
    for duration in [0.2, f64::from_bits(0.2f64.to_bits() - 1)] {
        let bytes = export(rate::linear, duration, 10);
        let serial = TimelineBundle::from_bytes(&bytes).unwrap();
        assert_eq!(serial.segment_kind(0), Some(BundleSegmentKind::Pure));
        assert_eq!(serial.frame_count(), if duration == 0.2 { 3 } else { 2 });
        let shared = SharedTimelineBundle::from_bytes(&bytes).unwrap();
        let mut cache = TimelineFrameCache::default();
        for _ in 0..3 {
            for index in (0..shared.frame_count()).rev() {
                assert_eq!(
                    state(
                        &cache
                            .materialize(&shared.frame_job(index).unwrap())
                            .unwrap()
                    ),
                    state(&serial.stage_at(index).unwrap())
                );
            }
        }
        assert_eq!(
            cache.decoded_snapshots(),
            2,
            "pure frames must not decode their endpoints repeatedly"
        );
        assert!(shared.snapshot_bytes() > 0);
    }
}

#[test]
fn dt_updaters_remain_recorded_and_are_never_reexecuted_by_workers() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(0.0);
    stage.add_to_scene(tracker).unwrap();
    let count = Rc::new(Cell::new(0));
    let seen = count.clone();
    stage
        .add_dt_updater(
            tracker,
            move |stage, mob, dt| {
                if dt > 0.0 {
                    seen.set(seen.get() + 1);
                }
                let value = stage.tracker_value(mob).unwrap();
                stage.set_tracker_value(mob, value + dt).unwrap();
            },
            false,
        )
        .unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    timeline.wait(0.5).unwrap();
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(7)).unwrap();
    assert_eq!(count.get(), 4);
    let serial = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(serial.segment_kind(0), Some(BundleSegmentKind::Stateful));
    let shared = SharedTimelineBundle::from_bytes(&bytes).unwrap();
    let mut cache = TimelineFrameCache::default();
    for index in [3, 0, 2, 1] {
        let job = shared.frame_job(index).unwrap();
        assert_eq!(job.kind(), BundleSegmentKind::Stateful);
        assert_eq!(
            state(&cache.materialize(&job).unwrap()),
            state(&serial.stage_at(index).unwrap())
        );
    }
    assert_eq!(cache.decoded_snapshots(), 4);
    assert_eq!(count.get(), 4);
}

#[test]
fn malformed_inputs_and_out_of_range_jobs_keep_the_readers_refusal() {
    let mut bytes = export(rate::linear, 0.5, 8);
    let shared = SharedTimelineBundle::from_bytes(&bytes).unwrap();
    for index in [shared.frame_count(), u32::MAX] {
        assert!(matches!(
            shared.frame_job(index),
            Err(BundleReadError::FrameOutOfRange { .. })
        ));
    }
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    assert!(TimelineBundle::from_bytes(&bytes).is_err());
    assert!(SharedTimelineBundle::from_bytes(&bytes).is_err());
    let mut stage = Stage::new();
    let bytes = export_timeline_bundle(
        Timeline::new(8).unwrap(),
        &mut stage,
        &RngRoot::from_seed(0),
    )
    .unwrap();
    let empty = SharedTimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(empty.frame_count(), 0);
    assert_eq!(empty.snapshot_bytes(), 0);
    assert!(empty.frame_job(0).is_err());
}
