//! Shared front-door adapter: all negotiated CPU formats and real render teams.

use fmn::rendering::{NativeFramePipeline, RenderError};
use fmn_core::color::LinearRgba;
use fmn_frame::convert::{rgba_to_nv12, rgba16f_to_p010, rgba16f_to_rgba8, swap_rb8};
use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameLayout, PixelFormat};
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, Stage};
use fmn_output::{EmitterConfig, OrderedEmitter, SinkBinding, SinkWrite};
use fmn_platform::topology::HardwareTopology;
use fmn_render::{
    EngineIdentity, FrameConfig, RetainedFrameRenderer, RetainedFrameRendererConfig, ScreenMap,
    Tiling, Viewport,
};
use fmn_runtime::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent, SurfaceSpec};
use std::sync::{Arc, Mutex};

fn plan(format: OutputPixelFormat) -> ExecutionPlan {
    let mut surface = SurfaceSpec::lumen(32, 24);
    if format != OutputPixelFormat::Rgba8 {
        surface.working_bytes_per_pixel += 4;
    }
    // Two small processor groups guarantee independent teams even on a
    // single-core CI host, without spawning workstation-sized worker pools.
    let topology = HardwareTopology::from_group_sizes(&[2, 2]).unwrap();
    ExecutionPlan::derive(
        PlanRequest::certified(RenderIntent::Offline, surface, format).with_max_frames_in_flight(3),
        &topology,
        None,
    )
    .unwrap()
}

fn config(plan: &ExecutionPlan) -> RetainedFrameRendererConfig {
    RetainedFrameRendererConfig {
        frame: FrameConfig::new(
            Viewport {
                width: 32,
                height: 24,
            },
            ScreenMap {
                scale: 4.0,
                origin: [16.0, 12.0],
                y_up: true,
            },
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        ),
        tiling: Tiling {
            macro_tile: plan.macro_tile,
            fine_tile: plan.fine_tile,
        },
        engine: EngineIdentity::certified(),
        threads: 1,
    }
}

fn shape() -> Mobject {
    let mut records = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    records.write_range(
        "point",
        0,
        &[-1.5, -1.0, 0.0, 0.0, 1.25, 0.0, 1.5, -1.0, 0.0],
    );
    records.write_range("fill_rgba", 0, &[0.2, 0.6, 1.0, 1.0].repeat(3));
    Mobject::from_buffer(records)
}

/// The serial two-step route for 8-bit wires (the pipeline's fused kernels
/// must reproduce it byte for byte), and the direct binary16 kernel for P010,
/// whose precision beyond eight bits the two-step route would discard.
fn convert_reference(frame: &FrameBuffer, format: PixelFormat) -> Vec<u8> {
    if format == PixelFormat::P010 {
        let mut output = FrameBuffer::new(FrameLayout::tight(format, 32, 24).unwrap());
        rgba16f_to_p010(frame, &mut output, ColorRange::Limited, ChromaSiting::Left).unwrap();
        return output.as_bytes().to_vec();
    }
    let mut rgba = FrameBuffer::new(FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap());
    rgba16f_to_rgba8(frame, &mut rgba).unwrap();
    if format == PixelFormat::Rgba8 {
        return rgba.as_bytes().to_vec();
    }
    let mut output = FrameBuffer::new(FrameLayout::tight(format, 32, 24).unwrap());
    match format {
        PixelFormat::Bgra8 => swap_rb8(&rgba, &mut output),
        PixelFormat::Nv12 => {
            rgba_to_nv12(&rgba, &mut output, ColorRange::Limited, ChromaSiting::Left)
        }
        _ => panic!("not a converted CPU format"),
    }
    .unwrap();
    output.as_bytes().to_vec()
}

#[test]
fn all_cpu_formats_match_serial_lumen_on_multiple_render_teams() {
    for (negotiated, format) in [
        (OutputPixelFormat::Rgba8, PixelFormat::Rgba8),
        (OutputPixelFormat::Bgra8, PixelFormat::Bgra8),
        (OutputPixelFormat::Nv12, PixelFormat::Nv12),
        (OutputPixelFormat::P010, PixelFormat::P010),
    ] {
        let plan = plan(negotiated);
        assert_eq!(plan.render_teams.len(), 2);
        let config = config(&plan);
        let received = Arc::new(Mutex::new(Vec::new()));
        let output = received.clone();
        let emitter = OrderedEmitter::new(
            EmitterConfig::new(
                FrameLayout::tight(format, 32, 24).unwrap(),
                plan.frames_in_flight,
                0,
            )
            .unwrap(),
            vec![SinkBinding::reliable(
                "record",
                move |sequence, frame: &FrameBuffer| {
                    output
                        .lock()
                        .unwrap()
                        .push((sequence, frame.as_bytes().to_vec()));
                    Ok(SinkWrite::Consumed)
                },
            )],
        )
        .unwrap();
        let mut pipeline =
            NativeFramePipeline::new(plan.clone(), config, None, emitter.handle()).unwrap();
        let mut serial = RetainedFrameRenderer::new(config).unwrap();
        let mut stage = Stage::new();
        let mob = stage.add(shape());
        stage.add_to_scene(mob).unwrap();
        let mut expected = Vec::new();
        for sequence in 0..12 {
            serial.render(&stage, 0).unwrap();
            expected.push((sequence, convert_reference(serial.frame(), format)));
            pipeline.capture(&stage, sequence).unwrap();
            stage.shift(mob, [0.1, 0.0, 0.0]);
            if sequence % 4 == 3 {
                let barrier = pipeline.flush().unwrap();
                assert_eq!(
                    (barrier.submitted, barrier.emitted),
                    (sequence + 1, sequence + 1)
                );
                assert_eq!(barrier.outstanding_slots, 0);
                assert_eq!(emitter.stats().published, sequence + 1);
            }
        }
        drop(stage);
        let stats = pipeline.finish().unwrap();
        let report = emitter.finish().unwrap();
        assert_eq!(stats.emitted, 12);
        assert_eq!(stats.barriers, 3);
        assert_eq!(stats.outstanding_slots, 0);
        assert!(stats.max_in_flight <= plan.frames_in_flight);
        assert!(stats.render_team_frames.iter().all(|count| *count > 0));
        assert_eq!(report.stats.outstanding, 0);
        assert_eq!(report.stats.emitted, 12);
        assert_eq!(*received.lock().unwrap(), expected);
        assert_ne!(expected[0].1, expected[11].1);
    }
}

#[test]
fn incompatible_ring_capacity_or_identity_is_refused_before_work() {
    for wrong_ring in [true, false] {
        let plan = plan(OutputPixelFormat::Rgba8);
        let mut config = config(&plan);
        if !wrong_ring {
            config.engine = EngineIdentity::fast();
        }
        let capacity = if wrong_ring { 1 } else { plan.frames_in_flight };
        let emitter = OrderedEmitter::new(
            EmitterConfig::new(
                FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap(),
                capacity,
                0,
            )
            .unwrap(),
            vec![SinkBinding::reliable("unused", |_, _: &FrameBuffer| {
                Ok(SinkWrite::Consumed)
            })],
        )
        .unwrap();
        let error = NativeFramePipeline::new(plan, config, None, emitter.handle());
        assert!(matches!(error, Err(RenderError::InvalidOptions(_))));
        assert_eq!(emitter.stats().reserved, 0);
        emitter.cancel();
        assert!(emitter.finish().is_err());
    }
}

#[test]
fn negotiated_layout_mismatch_fails_closed_without_freezing() {
    let plan = plan(OutputPixelFormat::Rgba8);
    let config = config(&plan);
    let emitter = OrderedEmitter::new(
        EmitterConfig::new(
            FrameLayout::tight(PixelFormat::Bgra8, 32, 24).unwrap(),
            plan.frames_in_flight,
            0,
        )
        .unwrap(),
        vec![SinkBinding::reliable("unused", |_, _: &FrameBuffer| {
            Ok(SinkWrite::Consumed)
        })],
    )
    .unwrap();
    let mut pipeline = NativeFramePipeline::new(plan, config, None, emitter.handle()).unwrap();
    assert!(matches!(
        pipeline.capture(&Stage::new(), 0),
        Err(RenderError::InvalidOptions(_))
    ));
    assert!(pipeline.finish().is_err());
    let failure = emitter.finish().unwrap_err();
    assert_eq!(failure.report.stats.outstanding, 0);
    assert_eq!(failure.report.stats.published, 0);
}

#[test]
fn drain_barriers_never_grant_artifact_publication() {
    use fmn_output::{FrameSink, SinkFailure};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct CommitWitness(Arc<AtomicBool>);
    impl FrameSink for CommitWitness {
        fn write_frame(&mut self, _: u64, _: &FrameBuffer) -> Result<SinkWrite, SinkFailure> {
            Ok(SinkWrite::Consumed)
        }
        fn finish(&mut self) -> Result<(), SinkFailure> {
            self.0.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
    let plan = plan(OutputPixelFormat::Rgba8);
    let config = config(&plan);
    let committed = Arc::new(AtomicBool::new(false));
    let emitter = OrderedEmitter::new(
        EmitterConfig::new(
            FrameLayout::tight(PixelFormat::Rgba8, 32, 24).unwrap(),
            plan.frames_in_flight,
            0,
        )
        .unwrap(),
        vec![SinkBinding::reliable(
            "commit-witness",
            CommitWitness(committed.clone()),
        )],
    )
    .unwrap();
    let mut pipeline = NativeFramePipeline::new(plan, config, None, emitter.handle()).unwrap();
    let mut stage = Stage::new();
    let mob = stage.add(shape());
    stage.add_to_scene(mob).unwrap();
    for sequence in 0..3 {
        pipeline.capture(&stage, sequence).unwrap();
        let barrier = pipeline.flush().unwrap();
        assert_eq!(barrier.emitted, sequence + 1);
        assert!(!committed.load(Ordering::SeqCst));
    }
    let stats = pipeline.finish().unwrap();
    assert_eq!((stats.emitted, stats.barriers), (3, 3));
    assert!(!committed.load(Ordering::SeqCst));
    emitter.finish().unwrap();
    assert!(committed.load(Ordering::SeqCst));
}
