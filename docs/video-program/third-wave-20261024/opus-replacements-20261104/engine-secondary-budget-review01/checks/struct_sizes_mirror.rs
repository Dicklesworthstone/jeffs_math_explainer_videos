// A standalone MIRROR, not the crate. Each type below copies the field list of
// the engine type of the same name, read from the patched review copies
// (fmn-render table.rs, engine.rs, stroke.rs, fill.rs, arena.rs; fmn-mobject
// placement.rs, stage.rs, uniforms.rs, bbox.rs; fmn-geom arclength.rs, vec.rs;
// fmn-hash sha256.rs; fmn-core types.rs), with the `metal` feature off.
// rustc picks the layout of a non-repr(C) struct from its field types, so a
// same-toolchain mirror normally agrees with the crate; only a size_of inside
// the real crate proves it. Build: rustc +nightly-2026-08-31 -O this file.
#![allow(dead_code)]
use std::mem::size_of;
use std::sync::Arc;

type Semantic = f64;
type Vec3 = [Semantic; 3];
type Mat3 = [[f64; 3]; 3];

struct StrokeProfile; // only ever behind Arc
struct FillProfile;

#[derive(Clone, Copy)]
enum JointType { NoJoint, Auto, Bevel, Miter }

struct Segment { p0: Vec3, p1: Vec3, p2: Vec3, s0: f64, s1: f64 }

struct Style {
    stroke_rgba: [f32; 4],
    stroke_rgba_end: [f32; 4],
    stroke_width: f32,
    stroke_width_end: f32,
    stroke_profile: Option<Arc<StrokeProfile>>,
    fill_rgba: [f32; 4],
    fill_rgba_end: [f32; 4],
    fill_profile: Option<Arc<FillProfile>>,
    fill_border_width: f32,
    anti_alias_width: f32,
    joint_type: JointType,
    is_fixed_in_frame: f64,
    shading: Vec3,
    clip_planes: [[f64; 4]; 4],
    flat_stroke: bool,
    scale_stroke_with_zoom: bool,
    stroke_behind: bool,
    depth_test: bool,
}

struct Mob { stage_id: u64, index: u32, generation: u32 }
struct Placement { linear: Mat3, translation: Vec3 }
struct Instance {
    shape: u32,
    style: u32,
    mob: Mob,
    placement: Placement,
    order: u32,
    revisions: u64,
    volatile: bool,
    hint_unsafe: bool,
}

struct Digest([u8; 32]);
struct BoundingBox { min: Vec3, mid: Vec3, max: Vec3 }
enum Hint {
    General,
    Line,
    Polyline { closed: bool },
    Arc { center: [f64; 3], radius: f64, start_angle: f64, angle: f64 },
    Circle { center: [f64; 3], radius: f64 },
    Dot { center: [f64; 3], radius: f64 },
    Rect { center: [f64; 3], width: f64, height: f64 },
    RoundedRect { center: [f64; 3], width: f64, height: f64, corner_radius: f64 },
}
struct ArcLengthTable { curve_lengths: Vec<f64>, cumulative: Vec<f64> }
struct Shape {
    digest: Digest,
    first_segment: u32,
    segment_count: u32,
    bounds: BoundingBox,
    hint: Hint,
    arc_length: ArcLengthTable,
    subpath_starts: Vec<u32>,
}

struct MonoPiece { p0: [f64; 2], p1: [f64; 2], p2: [f64; 2] }
struct JoinWedge {
    anchor: [f64; 2],
    t_in: [f64; 2],
    t_out: [f64; 2],
    half_width: f64,
    bisector: [f64; 2],
    n_in: [f64; 2],
    n_out: [f64; 2],
}
struct ChordStrip { origin: [f64; 2], normal: [f64; 2], half_width: f64 }
struct PreparedSegment {
    slab: [f64; 4],
    total_arc_length: f64,
    line: bool,
    station: bool,
    cull_distance_squared: Option<f64>,
    strip: Option<ChordStrip>,
}

#[derive(Clone, Copy)]
struct PoolRange { start: u32, len: u32 }
struct StrokeRef { segments: PoolRange, slab: [f64; 4] }
struct FieldRef { points: PoolRange, params: PoolRange, next: PoolRange, edge_params: PoolRange }
enum FillKernel {
    General,
    Rect { rect: [f64; 4] },
    Disc { center: [f64; 2], radius: f64 },
    RoundedRect { rect: [f64; 4], radius: f64 },
}
struct Draw {
    first_segment: u32,
    segment_count: u32,
    shape: u32,
    translate: [f64; 2],
    transformed_segments: Option<PoolRange>,
    transformed_pieces: Option<PoolRange>,
    style: Style,
    kernel: FillKernel,
    joins: PoolRange,
    stroke: Option<StrokeRef>,
    field: Option<FieldRef>,
    flat_fill: Option<[f32; 4]>,
    fill_slab: [f64; 4],
    draws_fill: bool,
    draws_stroke: bool,
}

fn main() {
    let rows: [(&str, usize); 21] = [
        ("Segment", size_of::<Segment>()),
        ("Style", size_of::<Style>()),
        ("Mob", size_of::<Mob>()),
        ("Placement", size_of::<Placement>()),
        ("Instance", size_of::<Instance>()),
        ("Digest", size_of::<Digest>()),
        ("BoundingBox", size_of::<BoundingBox>()),
        ("Hint", size_of::<Hint>()),
        ("ArcLengthTable", size_of::<ArcLengthTable>()),
        ("Shape", size_of::<Shape>()),
        ("MonoPiece", size_of::<MonoPiece>()),
        ("JoinWedge", size_of::<JoinWedge>()),
        ("ChordStrip", size_of::<ChordStrip>()),
        ("PreparedSegment", size_of::<PreparedSegment>()),
        ("PoolRange", size_of::<PoolRange>()),
        ("StrokeRef", size_of::<StrokeRef>()),
        ("FieldRef", size_of::<FieldRef>()),
        ("FillKernel", size_of::<FillKernel>()),
        ("Draw", size_of::<Draw>()),
        ("OptionDraw", size_of::<Option<Draw>>()),
        ("JoinPair", size_of::<(usize, usize)>()),
    ];
    for (name, size) in rows {
        println!("{name} {size}");
    }
    println!("PieceCurve {}", size_of::<[[f64; 2]; 3]>());
    // three_d.rs ProjectedCurvePiece { screen: Segment, world: [Vec3; 3] }.
    println!("ProjectedCurvePiece {}", size_of::<(Segment, [Vec3; 3])>());
}
