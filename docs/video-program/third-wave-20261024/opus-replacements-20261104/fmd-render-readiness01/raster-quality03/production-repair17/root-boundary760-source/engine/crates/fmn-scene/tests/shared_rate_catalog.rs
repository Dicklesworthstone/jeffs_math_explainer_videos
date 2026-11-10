//! fm-34mh: validated wire rate tags must survive the worker-sharing boundary.
use fmn_anim::{RateFunc, Timeline};
use fmn_hash::Writer;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::timeline_bundle::{
    BundleReadError, BundleSegmentKind, SharedTimelineBundle, TIMELINE_BUNDLE_SCHEMA,
    TimelineBundle, TimelineFrameCache, bundle_engine_version,
};

// Construct a wire vector independently of the exporter's function-address
// classification. This exercises every legal rate tag even in a build where
// the exporter conservatively demotes a rate it cannot identify by address.
fn pure_wire_vector(tag: u8) -> Vec<u8> {
    let mut stage = Stage::new();
    let moving = stage.add(Mobject::from_points(&[[0.0, 1.0, 0.0]]));
    let anchor = stage.add(Mobject::new());
    stage.add_many_to_scene(&[moving, anchor]).unwrap();
    let begin = stage.snapshot_bytes().unwrap();
    stage.shift_many(&[moving, anchor], [4.0, -2.0, 0.0]);
    let end = stage.snapshot_bytes().unwrap();
    let mut timeline = Timeline::new(8).unwrap();
    timeline.wait(1.0).unwrap();
    let mut writer = Writer::new(TIMELINE_BUNDLE_SCHEMA);
    writer.put_str(&bundle_engine_version());
    writer.put_u32(8);
    writer.put_bytes(&timeline.compile().unwrap().to_bytes().unwrap());
    writer.put_u32(1);
    writer.put_u8(0); // Pure endpoint-reconstruction segment.
    writer.put_bytes(&begin);
    writer.put_bytes(&end);
    writer.put_u8(0); // Straight path.
    writer.put_u8(tag);
    writer.finish().unwrap()
}

fn assert_shared_replay(bytes: &[u8]) {
    let serial = TimelineBundle::from_bytes(bytes).unwrap();
    let expected: Vec<_> = (0..serial.frame_count())
        .map(|index| {
            let stage = serial.stage_at(index).unwrap();
            (stage.snapshot_bytes().unwrap(), stage.time().to_bits())
        })
        .collect();
    let shared = serial.into_shared().unwrap();
    assert_eq!(shared.segment_kind(0), Some(BundleSegmentKind::Pure));
    let jobs: Vec<_> = (0..shared.frame_count())
        .rev()
        .map(|index| shared.frame_job(index).unwrap())
        .collect();
    drop(shared);
    std::thread::spawn(move || {
        let mut cache = TimelineFrameCache::default();
        for job in jobs {
            let stage = cache.materialize(&job).unwrap();
            assert_eq!(
                (stage.snapshot_bytes().unwrap(), stage.time().to_bits()),
                expected[job.index() as usize]
            );
        }
    })
    .join()
    .unwrap();
}

#[test]
fn every_wire_catalog_rate_replays_identically_on_a_worker() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RateFunc>();
    for tag in 0..8 {
        assert_shared_replay(&pure_wire_vector(tag));
    }
}

#[test]
fn checked_in_readme_demo_replays_without_regeneration() {
    let bytes = include_bytes!("../../../demo/wasm/bundle.fmtl");
    let serial = TimelineBundle::from_bytes(bytes).unwrap();
    assert_eq!(serial.fps(), 30);
    assert_eq!(serial.frame_count(), 45);
    assert_shared_replay(bytes);
}

#[test]
fn unknown_wire_rate_tags_still_fail_closed() {
    for tag in 8..=u8::MAX {
        let bytes = pure_wire_vector(tag);
        assert!(matches!(
            TimelineBundle::from_bytes(&bytes),
            Err(BundleReadError::PlanInconsistent("rate tag"))
        ));
        assert!(matches!(
            SharedTimelineBundle::from_bytes(&bytes),
            Err(BundleReadError::PlanInconsistent("rate tag"))
        ));
    }
}
