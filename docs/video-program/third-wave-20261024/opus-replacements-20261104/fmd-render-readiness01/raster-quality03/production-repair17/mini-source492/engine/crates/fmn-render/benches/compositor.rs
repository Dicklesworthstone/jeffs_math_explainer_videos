#![feature(test)]
#![forbid(unsafe_code)]

extern crate test;

use fmn_core::color::LinearRgba;
use fmn_mobject::{JointType, Mobject, Placement, RecordBuffer, RecordSchema, Stage};
use fmn_render::{
    Binning, EngineIdentity, EngineKind, FrameConfig, FrameJob, MonoPiece, MonoTable, RenderPlan,
    RowScratch, ScreenMap, Tier, Tiling, Viewport,
};
use std::hint::black_box;
use test::Bencher;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const STROKE_HEIGHT: u32 = HEIGHT / 2;
const LAYERS: usize = 32;

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64, fill: [f32; 4]) -> Mobject {
    let corners = [[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]];
    let mut points = vec![[corners[0][0], corners[0][1], 0.0]];
    for pair in corners.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        points.push([0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1]), 0.0]);
        points.push([b[0], b[1], 0.0]);
    }

    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    for (i, point) in points.iter().enumerate() {
        buffer.write(
            i,
            "point",
            &[point[0] as f32, point[1] as f32, point[2] as f32],
        );
        buffer.write(i, "fill_rgba", &fill);
    }
    Mobject::from_buffer(buffer)
}

struct Fixture {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
}

fn stroked_chain(segment_count: usize) -> Mobject {
    let x_lo = 16.0;
    let x_hi = f64::from(WIDTH) - 16.0;
    let span = (x_hi - x_lo) / segment_count as f64;
    let centre_y = f64::from(STROKE_HEIGHT) * 0.5;
    let mut points = Vec::with_capacity(2 * segment_count + 1);
    points.push([x_lo, centre_y, 0.0]);
    for segment in 0..segment_count {
        let x0 = x_lo + segment as f64 * span;
        let x2 = x0 + span;
        let y2 = centre_y
            + if segment.is_multiple_of(2) {
                20.0
            } else {
                -20.0
            };
        let handle_y = centre_y
            + if segment.is_multiple_of(2) {
                -56.0
            } else {
                56.0
            };
        points.push([0.5 * (x0 + x2), handle_y, 0.0]);
        points.push([x2, y2, 0.0]);
    }

    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    for (i, point) in points.iter().enumerate() {
        buffer.write(
            i,
            "point",
            &[point[0] as f32, point[1] as f32, point[2] as f32],
        );
        buffer.write(i, "fill_rgba", &[0.0, 0.0, 0.0, 0.0]);
        buffer.write(i, "stroke_rgba", &[0.2, 0.7, 0.9, 1.0]);
        buffer.write(i, "stroke_width", &[600.0]);
    }
    Mobject::from_buffer(buffer)
}

struct StrokeFixture {
    plan: RenderPlan,
    mono: MonoTable,
    binning: Binning,
    config: FrameConfig,
}

impl StrokeFixture {
    fn new(segment_count: usize) -> Self {
        Self::with_placement(segment_count, Placement::IDENTITY)
    }

    fn with_placement(segment_count: usize, placement: Placement) -> Self {
        let mut stage = Stage::new();
        let mob = stage.add(stroked_chain(segment_count));
        stage.add_to_scene(mob).expect("live benchmark stroke");
        stage
            .uniforms_mut(mob)
            .expect("live benchmark stroke")
            .joint_type = JointType::Miter;
        stage.apply_affine(mob, placement);

        let config = FrameConfig::new(
            Viewport {
                width: WIDTH,
                height: STROKE_HEIGHT,
            },
            ScreenMap::default(),
            LinearRgba {
                r: 0.02,
                g: 0.02,
                b: 0.02,
                a: 1.0,
            },
        );
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0)
            .expect("valid stroke benchmark fixture");
        let mono =
            MonoTable::build(&plan, config.map).expect("bounded stroke benchmark monotone table");
        let binning = Binning::build(
            &plan,
            config.viewport,
            Tiling {
                macro_tile: 128,
                fine_tile: 16,
            },
            config.map,
        )
        .expect("bounded benchmark binning");
        Self {
            plan,
            mono,
            binning,
            config,
        }
    }

    fn render(&self) {
        let job = self.compile(EngineIdentity::certified());
        black_box(job.render(1).expect("benchmark stroke render"));
    }

    fn compile(&self, identity: EngineIdentity) -> FrameJob<'_> {
        FrameJob::with_identity(&self.plan, &self.mono, &self.binning, self.config, identity)
            .expect("coherent benchmark stroke artifacts")
    }
}

impl Fixture {
    fn new() -> Self {
        let mut stage = Stage::new();
        for layer in 0..LAYERS {
            let t = layer as f32 / LAYERS as f32;
            let mob = stage.add(rectangle(
                -1.0,
                -1.0,
                f64::from(WIDTH) + 1.0,
                f64::from(HEIGHT) + 1.0,
                [0.2 + 0.6 * t, 0.7 - 0.5 * t, 0.3 + 0.4 * t, 0.08],
            ));
            stage.add_to_scene(mob).expect("live benchmark layer");
        }

        let config = FrameConfig::new(
            Viewport {
                width: WIDTH,
                height: HEIGHT,
            },
            ScreenMap::default(),
            LinearRgba {
                r: 0.02,
                g: 0.02,
                b: 0.02,
                a: 1.0,
            },
        );
        let mut plan = RenderPlan::new();
        plan.sync(&stage, 0).expect("valid fill benchmark fixture");
        let mono =
            MonoTable::build(&plan, config.map).expect("bounded fill benchmark monotone table");
        let binning = Binning::build(
            &plan,
            config.viewport,
            Tiling {
                macro_tile: 128,
                fine_tile: 16,
            },
            config.map,
        )
        .expect("bounded benchmark binning");
        Self {
            plan,
            mono,
            binning,
            config,
        }
    }

    fn render(&self, identity: EngineIdentity) {
        let job =
            FrameJob::with_identity(&self.plan, &self.mono, &self.binning, self.config, identity)
                .expect("coherent benchmark artifacts");
        black_box(job.render(1).expect("benchmark render"));
    }
}

#[bench]
fn layered_translucent_compositor(bench: &mut Bencher) {
    let fixture = Fixture::new();
    bench.iter(|| {
        fixture.render(EngineIdentity::certified());
    });
}

#[bench]
fn layered_translucent_compositor_compiled_tier(bench: &mut Bencher) {
    let fixture = Fixture::new();
    bench.iter(|| {
        fixture.render(EngineIdentity {
            tier: Tier::COMPILED,
            ..EngineIdentity::certified()
        });
    });
}

#[bench]
fn layered_translucent_compositor_fast_scalar(bench: &mut Bencher) {
    let fixture = Fixture::new();
    bench.iter(|| {
        fixture.render(EngineIdentity {
            engine: EngineKind::FastCpu,
            ..EngineIdentity::certified()
        });
    });
}

#[bench]
fn layered_translucent_compositor_fast_compiled_tier(bench: &mut Bencher) {
    let fixture = Fixture::new();
    bench.iter(|| {
        fixture.render(EngineIdentity::fast());
    });
}

#[test]
fn profile_one_layered_frame() {
    Fixture::new().render(EngineIdentity::certified());
}

/// Fixed-work paired samples for workers where wall-clock noise swamps libtest's
/// independently calibrated runs. Alternate order to avoid giving one route
/// every cold-cache or frequency-ramp position. Run with `--nocapture`.
#[test]
fn profile_paired_layered_frames() {
    const FRAMES: usize = 32;
    let fixture = Fixture::new();
    for engine in [EngineKind::CertifiedCpu, EngineKind::FastCpu] {
        let scalar = EngineIdentity {
            engine,
            ..EngineIdentity::certified()
        };
        let tier = EngineIdentity {
            tier: Tier::COMPILED,
            ..scalar
        };
        for _ in 0..3 {
            fixture.render(scalar);
            fixture.render(tier);
        }
        for sample in 0..10 {
            let identities = if sample % 2 == 0 {
                [scalar, tier]
            } else {
                [tier, scalar]
            };
            for identity in identities {
                let start = std::time::Instant::now();
                for _ in 0..FRAMES {
                    fixture.render(identity);
                }
                println!(
                    "compositor_sample engine={engine:?} tier={} sample={sample} frames={FRAMES} elapsed_ns={}",
                    identity.tier.name(),
                    start.elapsed().as_nanos()
                );
            }
        }
    }
}

fn benchmark_column_roots(bench: &mut Bencher, width: u32) {
    let mut pieces = Vec::with_capacity(64);
    for index in 0_u32..64 {
        let nudge = f64::from(index % 8) * 0.000_125;
        let (x0, x2) = if index % 2 == 0 {
            (0.125 + nudge, f64::from(width) - 0.125 - nudge)
        } else {
            (f64::from(width) - 0.125 - nudge, 0.125 + nudge)
        };
        pieces.push(MonoPiece {
            p0: [x0, 0.031_25],
            p1: [0.5 * (x0 + x2), 0.375 + nudge],
            p2: [x2, 0.968_75],
        });
    }
    let mut scratch = RowScratch::for_tile(width).expect("benchmark row scratch");
    bench.iter(|| {
        black_box(scratch.fill_row(black_box(&pieces), [0.0, 0.0], 0, 0, width));
    });
}

macro_rules! column_root_benches {
    ($name:ident, $width:expr) => {
        #[bench]
        fn $name(bench: &mut Bencher) {
            benchmark_column_roots(bench, $width);
        }
    };
}

column_root_benches!(column_roots_02, 2);
column_root_benches!(column_roots_04, 4);
column_root_benches!(column_roots_08, 8);
column_root_benches!(column_roots_16, 16);
column_root_benches!(column_roots_32, 32);
column_root_benches!(column_roots_64, 64);

fn benchmark_stroke_sdf(bench: &mut Bencher, segment_count: usize) {
    let fixture = StrokeFixture::new(segment_count);
    bench.iter(|| fixture.render());
}

macro_rules! stroke_sdf_benches {
    ($name:ident, $segment_count:expr) => {
        #[bench]
        fn $name(bench: &mut Bencher) {
            benchmark_stroke_sdf(bench, $segment_count);
        }
    };
}

stroke_sdf_benches!(stroke_sdf_chain_08, 8);
stroke_sdf_benches!(stroke_sdf_chain_32, 32);
stroke_sdf_benches!(stroke_sdf_chain_64, 64);

fn benchmark_point_transform(bench: &mut Bencher, segment_count: usize, placement: Placement) {
    let fixture = StrokeFixture::with_placement(segment_count, placement);
    bench.bytes = u64::try_from(segment_count * 3 * 3 * size_of::<f64>())
        .expect("benchmark byte count fits u64");
    bench.iter(|| {
        black_box(fixture.compile(EngineIdentity::certified()));
    });
}

macro_rules! point_transform_benches {
    ($translation:ident, $uniform_scale:ident, $general_affine:ident, $segment_count:expr) => {
        #[bench]
        fn $translation(bench: &mut Bencher) {
            benchmark_point_transform(
                bench,
                $segment_count,
                Placement::from_translation([17.0, -31.0, 0.0]),
            );
        }

        #[bench]
        fn $uniform_scale(bench: &mut Bencher) {
            benchmark_point_transform(
                bench,
                $segment_count,
                Placement::new(
                    [[1.125, 0.0, 0.0], [0.0, 1.125, 0.0], [0.0, 0.0, 1.125]],
                    [17.0, -31.0, 0.0],
                ),
            );
        }

        #[bench]
        fn $general_affine(bench: &mut Bencher) {
            benchmark_point_transform(
                bench,
                $segment_count,
                Placement::new(
                    [[1.125, 0.25, 0.0], [-0.375, 0.875, 0.0], [0.0, 0.0, 1.0]],
                    [17.0, -31.0, 0.0],
                ),
            );
        }
    };
}

point_transform_benches!(
    point_transform_translation_0008,
    point_transform_uniform_scale_0008,
    point_transform_general_affine_0008,
    8
);
point_transform_benches!(
    point_transform_translation_0064,
    point_transform_uniform_scale_0064,
    point_transform_general_affine_0064,
    64
);
point_transform_benches!(
    point_transform_translation_1024,
    point_transform_uniform_scale_1024,
    point_transform_general_affine_1024,
    1_024
);
