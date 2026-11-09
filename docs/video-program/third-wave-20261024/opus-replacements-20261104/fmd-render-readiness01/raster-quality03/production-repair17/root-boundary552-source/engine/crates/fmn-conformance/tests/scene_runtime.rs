//! The fm-5xm primitive Scene corpus: 25 named scenes driven through the
//! public `fmn` facade, Proscenium lifecycle, Choreo rational clock and FramePacket,
//! materialized only from the immutable packet at the Lumen boundary, then
//! rasterized into certified self-goldens.
//!
//! Each artifact locks the complete three-frame sequence (two play samples +
//! one wait sample), not merely terminal geometry. The terminal packet is also
//! rendered at {1,4,16} threads so the Scene integration participates in PG-5
//! rather than relying only on Lumen's isolated engine corpus.

use fmn::builtins::{PRIMITIVE_SCENE_NAMES, primitive_scene};
use fmn::prelude::{
    CaptureReason, FramePacket, IntegrationError, RuntimeConfig, SceneSink, Srgb, run_scene,
};
use fmn_conformance::golden::{GoldenStore, Scope};
use fmn_hash::{Schema, Writer};
use fmn_render::bin::{Binning, ScreenMap, Tiling, Viewport};
use fmn_render::engine::{FrameConfig, FrameJob, encode_frame};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;
use std::path::PathBuf;

const WIDTH: u32 = 96;
const HEIGHT: u32 = 54;
const SCALE: f64 = 20.0;
const FPS: u32 = 8;
const TILING: Tiling = Tiling {
    macro_tile: 64,
    fine_tile: 8,
};
const CORPUS_SCHEMA: Schema = Schema::new(*b"FMNS", 13, 1, 0);

fn store() -> GoldenStore {
    GoldenStore::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens"),
        "scene_runtime",
        Scope::Certified,
    )
    .expect("golden store")
}

fn frame_config() -> FrameConfig {
    FrameConfig::new(
        Viewport {
            width: WIDTH,
            height: HEIGHT,
        },
        ScreenMap {
            scale: SCALE,
            origin: [f64::from(WIDTH) / 2.0, f64::from(HEIGHT) / 2.0],
            y_up: true,
        },
        Srgb::from_rgb8(0x22, 0x22, 0x22).to_linear(1.0),
    )
}

fn render(packet: &FramePacket, threads: usize) -> Result<Vec<u8>, IntegrationError> {
    let stage = packet.materialize_stage();
    let config = frame_config();
    let mut plan = RenderPlan::new();
    let camera_revision = u64::try_from(packet.frame_index())
        .map_err(|_| IntegrationError::new("lumen", "negative frame index reached the renderer"))?;
    plan.sync(&stage, camera_revision)
        .map_err(|error| IntegrationError::new("lumen", error.to_string()))?;
    let mono = MonoTable::build(&plan, config.map)
        .map_err(|error| IntegrationError::new("lumen", error.to_string()))?;
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .map_err(|error| IntegrationError::new("lumen", error.to_string()))?;
    let job = FrameJob::new(&plan, &mono, &binning, config)
        .map_err(|error| IntegrationError::new("lumen", error.to_string()))?;
    let frame = job
        .render(threads)
        .map_err(|error| IntegrationError::new("lumen", error.to_string()))?;
    encode_frame(&frame).map_err(|error| IntegrationError::new("lumen", error.to_string()))
}

struct LumenSink {
    frames: Vec<(CaptureReason, i64, i64, Vec<u8>)>,
    last: Option<FramePacket>,
}

impl LumenSink {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
            last: None,
        }
    }

    fn artifact(&self, name: &str) -> Vec<u8> {
        let mut writer = Writer::new(CORPUS_SCHEMA);
        writer.put_str(name).put_u64(self.frames.len() as u64);
        for (reason, frame, segment_frame, bytes) in &self.frames {
            writer
                .put_u8(match reason {
                    CaptureReason::Segment => 0,
                    CaptureReason::Show => 1,
                    CaptureReason::SkippedPreview => 2,
                    CaptureReason::PresenterHold => 3,
                })
                .put_i64(*frame)
                .put_i64(*segment_frame)
                .put_bytes(bytes);
        }
        writer.finish().expect("corpus artifact encodes")
    }
}

impl SceneSink for LumenSink {
    fn capture(
        &mut self,
        reason: CaptureReason,
        packet: FramePacket,
    ) -> Result<(), IntegrationError> {
        let bytes = render(&packet, 1)?;
        self.frames
            .push((reason, packet.frame_index(), packet.segment_frame(), bytes));
        self.last = Some(packet);
        Ok(())
    }
}

#[test]
fn twenty_five_scene_sequences_are_bit_locked_and_thread_invariant() {
    assert_eq!(PRIMITIVE_SCENE_NAMES.len(), 25);
    let store = store();
    let mut failures = Vec::new();
    for (index, &name) in PRIMITIVE_SCENE_NAMES.iter().enumerate() {
        let mut program = primitive_scene(name).expect("listed primitive scene resolves");
        let mut sink = LumenSink::new();
        let completed = run_scene(
            &mut program,
            RuntimeConfig {
                fps: FPS,
                ..RuntimeConfig::default()
            },
            index as u64,
            &mut sink,
        )
        .expect("scene runs through the public fmn facade");
        assert_eq!(completed.report().play_count, 2, "{name}");
        assert_eq!(sink.frames.len(), 3, "{name}");

        if let Err(error) = store.check(name, &sink.artifact(name)) {
            failures.push(error.to_string());
        }

        let terminal = sink.last.as_ref().expect("wait emitted a terminal frame");
        let scalar = &sink.frames.last().expect("terminal bytes").3;
        for threads in [4, 16] {
            assert_eq!(
                render(terminal, threads).expect("threaded render"),
                *scalar,
                "{name} drifted at {threads} threads"
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
