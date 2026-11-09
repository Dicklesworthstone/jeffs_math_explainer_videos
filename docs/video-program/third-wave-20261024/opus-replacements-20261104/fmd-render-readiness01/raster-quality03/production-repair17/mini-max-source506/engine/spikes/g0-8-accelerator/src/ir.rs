//! The prototype compiled render IR (§10.8), cut down to exactly what the
//! stroke-SDF + AA-resolve stage reads.
//!
//! This is the spike's primary deliverable. §20.1 spike 8 exists to answer
//! "does the IR map onto a GPU?" *before* W5 (fm-gw7) freezes the IR, so the
//! tables below are written to be a faithful subset of §10.8's design — the
//! same `PathTable` / `SegmentTable` / `StyleTable` / per-tile command lists,
//! carrying the same independent revisions — rather than a convenient fiction
//! that would prove nothing.
//!
//! ## The layout rule the spike is testing
//!
//! §10.8 says backend-specific layouts are *derived* from one backend-neutral
//! IR: "SoA/AoSoA sized to SIMD lanes on CPU; contiguous device arrays for
//! CUDA; packed shared/private buffers for Metal." [`RenderIr`] is that
//! backend-neutral form (host structs, `Vec`s of records); [`FlatIr`] is the
//! Metal-shaped derivation — one flat array per table, each array a single
//! scalar type, because a Metal buffer binding is a typed pointer and mixing
//! `u32` and `f32` in one buffer costs a bitcast in the kernel's inner loop.
//!
//! **That constraint is the finding.** It is cheap to honor if the IR is
//! designed for it and expensive to retrofit, which is precisely why this
//! spike runs before fm-gw7 rather than during it.
//!
//! ## Two spikes, one IR
//!
//! G0-8 (fm-ekx) needed the stroke stage and nothing else, and the IR was cut
//! to exactly that. G0-6 (fm-zn9) then needed *one nontrivial frame* carrying
//! gradient fills and glow falloff as well, through the same prototype
//! renderer — so [`DrawKind`] joined the path header rather than a second
//! renderer joining the tree.
//!
//! The stroke stage's device layout ([`FlatIr`]) is deliberately **unchanged**
//! by that addition. G0-8's Metal numbers were measured against those exact
//! bytes, and quietly reshaping the buffers would have invalidated a published
//! measurement to save a branch. The annex instead refuses a non-stroke IR by
//! name, which is also what D-18 requires: the frame that defines the certified
//! bits must never reach a GPU.
//!
//! ## What is still deliberately absent
//!
//! Images, 3D, glyph instancing, occlusion masks, and the second binning level.
//! Each is named in the G0-8 mapping report with its expected layout, so fm-gw7
//! inherits an assessment rather than a silence.

use fmn_geom::arclength;
use fmn_geom::quadpath::QuadPath;

/// A screen-space quadratic Bézier segment with its arc-length span.
///
/// The arc-length span `(s0, s1)` is normalized over the *whole path*, and it
/// is precomputed on the host by [`RenderIr::compile_path`] using fmn-geom's
/// exact closed-form arc length. This is a deliberate division of labour: §7.3
/// arc length is transcendental-dense and branchy — the wrong shape for a
/// per-pixel kernel — while §10.3's "width and colour interpolated by **arc
/// length**" only ever needs the answer, not the machinery. The GPU reads two
/// floats.
///
/// Because the span is a function of geometry alone, it is invalidated by the
/// *geometry* revision and survives a style change or a transform — matching
/// the retained-cache rule fmn-geom already implements in
/// `arclength::CachedArcLength`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Start anchor, screen space, pixels.
    pub p0: [f32; 2],
    /// Control handle.
    pub p1: [f32; 2],
    /// End anchor.
    pub p2: [f32; 2],
    /// Normalized arc length at `t = 0`, over the whole path.
    pub s0: f32,
    /// Normalized arc length at `t = 1`, over the whole path.
    pub s1: f32,
}

/// The §10.8 primitive hint, cut to the shapes this stage can specialize.
///
/// The hint is carried through the IR unchanged from
/// `fmn_mobject::shape::ShapeTag` (which already implements the invalidation
/// rule: a write through `set_points` or a live view drops the hint back to
/// `General`). The spike's kernel *reads* it and takes the capsule fast path
/// for `Line`, which is what makes the hint's presence in the IR measurable
/// rather than decorative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PrimitiveHint {
    /// The general quadratic solver.
    General = 0,
    /// Every segment is exactly straight: distance is a point-to-capsule
    /// evaluation, no cubic root solve.
    Line = 1,
}

/// What a compiled path draws.
///
/// The stroke stage (G0-8) predates this: it had one kind and did not need to
/// say so. G0-6's determinism frame must exercise gradient fills and glow
/// falloff too (§20.1 spike 6), and a renderer that grew a second stage by
/// overloading the first would be the wrong thing to hash.
///
/// **The annex engine accepts `Stroke` only, and says so with a typed error.**
/// That is not a limitation to apologize for: D-18 refuses GPU work in the
/// certified path *permanently*, so the frame that defines the certified bits
/// must never reach a GPU. A kernel that silently skipped the fills would have
/// produced a plausible, wrong, and very hard to notice picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DrawKind {
    /// Stroke the path: distance to the curve against the width profile.
    Stroke = 0,
    /// Fill the path's interior under the nonzero winding rule.
    Fill = 1,
    /// A radial glow disc centred on the path's first anchor, radius
    /// [`Style::glow_radius`] — the Reference's `true_dot` profile.
    Glow = 2,
}

/// One renderable path: a segment range, a style, and a conservative slab.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathHeader {
    /// Index of this path's first segment in the [`SegmentTable`].
    pub first_segment: u32,
    /// How many segments the path owns.
    pub segment_count: u32,
    /// Index into the [`StyleTable`].
    pub style: u32,
    /// The primitive hint (§10.8).
    pub hint: PrimitiveHint,
    /// What this path draws.
    pub kind: DrawKind,
    /// Conservative screen-space stroke slab, `[min_x, min_y, max_x, max_y]`:
    /// the geometric bounds grown by `half_width + aa_width`. A pixel outside
    /// it is provably clear, so this is the kernel's per-pixel early-out and
    /// the host's binning key at once.
    pub slab: [f32; 4],
}

/// Stroke style. Widths are screen-space pixels; colour is linear light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Linear-light RGBA, straight (non-premultiplied) alpha.
    pub rgba: [f32; 4],
    /// Stroke width at arc-length 0.
    pub width_start: f32,
    /// Stroke width at arc-length 1. Equal to `width_start` for a uniform
    /// stroke; differing values are the per-point taper
    /// (`set_stroke(width=[0, 6, 0])`) that `VMobject::with_stroke_profile`
    /// already carries in fmn-library.
    pub width_end: f32,
    /// The AA band width in pixels — [`crate::sdf::ANTI_ALIAS_WIDTH_PX`]
    /// unless a mobject overrode it.
    pub aa_width: f32,
    /// Linear-light RGBA at the far end of the gradient. Equal to [`rgba`] for
    /// a flat style.
    ///
    /// For a stroke the gradient runs along **arc length**, sharing the
    /// coordinate the width taper already uses. For a fill it runs along the
    /// axis in [`gradient_axis`]. §10.2's real interior field (arc-length
    /// boundary interpolation with mean-value coordinates) is fm-5oi's; this
    /// is a *defined, deterministic stand-in* and the G0-6 note says so.
    ///
    /// [`rgba`]: Style::rgba
    /// [`gradient_axis`]: Style::gradient_axis
    pub rgba_end: [f32; 4],
    /// Screen-space gradient axis for fills: `[x0, y0, x1, y1]`. The colour at
    /// a pixel is [`rgba`] → [`rgba_end`] by its projection onto this segment,
    /// clamped. Ignored by strokes.
    ///
    /// [`rgba`]: Style::rgba
    /// [`rgba_end`]: Style::rgba_end
    pub gradient_axis: [f32; 4],
    /// Radius in pixels for [`DrawKind::Glow`]. Ignored otherwise.
    pub glow_radius: f32,
    /// The Reference's `glow_factor` uniform. `0` gives a hard-edged
    /// antialiased dot (`DotCloud`'s default); `2.0` is what `GlowDots` uses.
    /// Ignored unless the kind is [`DrawKind::Glow`].
    pub glow_factor: f32,
    /// How strongly the per-vertex joint angle widens the stroke at a joint.
    ///
    /// **A defined stand-in, not §10.3's join model.** A true-distance stroke
    /// already produces correct round joins from the distance field alone, so
    /// nothing in the stroke stage *needed* a joint angle. G0-6's frame does:
    /// §20.1 spike 6 requires it to exercise "per-vertex atan2 joints", and a
    /// joint angle that never reaches a pixel is not exercised by a frame hash.
    /// So this scales the Reference's own `miter_factor` (its
    /// `MITER_COS_ANGLE_THRESHOLD = -0.8` smoothstep) into a width gain near
    /// each joint. §10.3's real miter/bevel/auto joins with a miter limit are
    /// fm-oac's to build.
    ///
    /// **Zero disables it entirely**, which is what [`Style::flat`] sets and
    /// what every G0-8 stroke uses — so the published annex-equivalence numbers
    /// are unaffected by this field existing. The annex refuses a nonzero
    /// value by name rather than rendering something the shader does not
    /// implement.
    pub miter_gain: f32,
}

impl Style {
    /// A flat, ungraded style — the shape every G0-8 stroke used before
    /// gradients existed.
    pub fn flat(rgba: [f32; 4], width: f32, aa_width: f32) -> Style {
        Style {
            rgba,
            width_start: width,
            width_end: width,
            aa_width,
            rgba_end: rgba,
            gradient_axis: [0.0, 0.0, 1.0, 0.0],
            glow_radius: 0.0,
            glow_factor: 0.0,
            miter_gain: 0.0,
        }
    }
}

/// A uniform tile grid over the output surface.
///
/// One level, not §10.8's two (macrotile → fine tile). The spike's question is
/// whether *a* tiling maps to threadgroups, and it does; the macrotile level is
/// a host-side pruning optimization that changes no kernel contract. Named in
/// the report rather than implemented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileGrid {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Square tile edge in pixels.
    pub tile: u32,
}

impl TileGrid {
    /// Tiles across.
    pub fn cols(&self) -> u32 {
        self.width.div_ceil(self.tile)
    }
    /// Tiles down.
    pub fn rows(&self) -> u32 {
        self.height.div_ceil(self.tile)
    }
    /// Total tiles.
    pub fn count(&self) -> usize {
        self.cols() as usize * self.rows() as usize
    }
}

/// Per-tile command lists in painter order: CSR-style `offsets` into `draws`.
///
/// §10.8 requires "stable per-tile command runs" with "stable draw indices" and
/// forbids "unordered atomic appends" on the annex. The CSR shape enforces both
/// structurally — the list is *built* by a deterministic count/prefix/scatter on
/// the host and *read* in index order by the kernel, so no ordering decision
/// exists at dispatch time to get wrong.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandLists {
    /// `count() + 1` entries; tile `i` owns `draws[offsets[i]..offsets[i+1]]`.
    pub offsets: Vec<u32>,
    /// Path indices, ascending within a tile — which *is* painter order,
    /// because the path table is built in draw order.
    pub draws: Vec<u32>,
}

/// The backend-neutral compiled render IR for one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderIr {
    /// All segments of all paths, contiguous, grouped by path.
    pub segments: Vec<Segment>,
    /// Per-segment joint angles in radians, two per segment: the angle the
    /// path turns through at that segment's start anchor and at its end anchor.
    /// `0` where a segment has no neighbour (an open end).
    ///
    /// Computed on the host with `atan2` — deliberately, since §20.1 spike 6
    /// requires the determinism frame to exercise per-vertex `atan2`, and the
    /// host is where a branchy transcendental belongs (the same division of
    /// labour that put arc length on the host, G0-8 finding F3).
    ///
    /// **Not part of [`FlatIr`].** The Metal layout is byte-for-byte what G0-8
    /// measured; adding a table the shader does not read would have invalidated
    /// a published measurement for nothing.
    pub joint_angles: Vec<f32>,
    /// One header per path, in draw order.
    pub paths: Vec<PathHeader>,
    /// Deduplicated styles.
    pub styles: Vec<Style>,
    /// The tiling.
    pub grid: TileGrid,
    /// Per-tile painter-ordered command lists.
    pub tiles: CommandLists,
    /// Linear-light background the frame clears to.
    pub background: [f32; 4],
}

impl RenderIr {
    /// Start an empty IR over `grid` with `background`.
    pub fn new(grid: TileGrid, background: [f32; 4]) -> RenderIr {
        RenderIr {
            segments: Vec::new(),
            joint_angles: Vec::new(),
            paths: Vec::new(),
            styles: Vec::new(),
            grid,
            tiles: CommandLists::default(),
            background,
        }
    }

    /// Intern a style, returning its index. Deduplication is by exact bits,
    /// matching `fmn_mobject::order::BatchKey`'s bitwise float comparison — a
    /// style table that merged `0.1 + 0.2` with `0.3` would silently change
    /// batching, and batching is semantics (§8.5).
    pub fn intern_style(&mut self, style: Style) -> u32 {
        let key = style_bits(&style);
        for (i, s) in self.styles.iter().enumerate() {
            if style_bits(s) == key {
                return i as u32;
            }
        }
        self.styles.push(style);
        (self.styles.len() - 1) as u32
    }

    /// Compile one screen-space [`QuadPath`] into the tables, in draw order.
    ///
    /// This is the IR-synchronization step of §10.8 in miniature: the path's
    /// curves become segments, its exact arc length becomes the normalized
    /// span each segment carries, its bounds grown by `half_width + aa_width`
    /// become the conservative slab, and its shape decides the primitive hint.
    ///
    /// Returns the new path's index, or `None` for a path with no curves
    /// (which must produce no command, not an empty draw).
    pub fn compile_path(&mut self, path: &QuadPath, style: Style, kind: DrawKind) -> Option<u32> {
        let n = path.num_curves();
        if n == 0 {
            return None;
        }

        // Exact per-curve arc lengths, from fmn-geom's closed form. A null
        // curve (fmn-geom's subpath break) contributes zero length and no
        // segment: a break is not a stroked piece of the path.
        let mut lengths = Vec::with_capacity(n);
        let mut total = 0.0f64;
        for i in 0..n {
            let [a0, h, a1] = path.nth_curve_points(i)?;
            let len = if is_null_curve(a0, h, a1) {
                0.0
            } else {
                arclength::quadratic_arc_length(a0, h, a1)
            };
            lengths.push(len);
            total += len;
        }
        if total <= 0.0 {
            return None;
        }

        let first_segment = self.segments.len() as u32;
        let mut acc = 0.0f64;
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut all_straight = true;

        for (i, &len) in lengths.iter().enumerate() {
            if len <= 0.0 {
                continue;
            }
            let [a0, h, a1] = path.nth_curve_points(i)?;
            let s0 = acc / total;
            acc += len;
            let s1 = acc / total;

            if !is_straight(a0, h, a1) {
                all_straight = false;
            }
            for p in [a0, h, a1] {
                min[0] = min[0].min(p[0]);
                min[1] = min[1].min(p[1]);
                max[0] = max[0].max(p[0]);
                max[1] = max[1].max(p[1]);
            }
            self.segments.push(Segment {
                p0: [a0[0] as f32, a0[1] as f32],
                p1: [h[0] as f32, h[1] as f32],
                p2: [a1[0] as f32, a1[1] as f32],
                s0: s0 as f32,
                s1: s1 as f32,
            });
        }

        let segment_count = self.segments.len() as u32 - first_segment;
        if segment_count == 0 {
            return None;
        }

        // Per-vertex joint angles, by atan2 of the tangents either side of each
        // shared anchor. Consecutive segments in this path meet at anchors; an
        // open end has no turn, so it gets zero.
        let range = first_segment as usize..self.segments.len();
        for i in range.clone() {
            let seg = self.segments[i];
            let t_in = tangent_at_start(&seg);
            let start_angle = if i > range.start {
                let prev = self.segments[i - 1];
                turn_between(tangent_at_end(&prev), t_in)
            } else {
                0.0
            };
            let end_angle = if i + 1 < range.end {
                let next = self.segments[i + 1];
                turn_between(tangent_at_end(&seg), tangent_at_start(&next))
            } else {
                0.0
            };
            self.joint_angles.push(start_angle as f32);
            self.joint_angles.push(end_angle as f32);
        }

        // The slab must be conservative for *every* point the draw can touch,
        // and a quadratic's control polygon is its convex hull — so hull bounds
        // grown by the right margin are exact-or-generous, never tight-and-wrong.
        // The margin is per-kind, which is the whole reason the kind reaches
        // this function: a fill grown by half a stroke width would clip its own
        // antialiased edge, and a glow grown by a stroke width would clip most
        // of the glow.
        let grow = match kind {
            DrawKind::Stroke => {
                0.5 * style.width_start.max(style.width_end) as f64 + style.aa_width as f64
            }
            DrawKind::Fill => style.aa_width as f64,
            DrawKind::Glow => style.glow_radius as f64 + style.aa_width as f64,
        };
        let slab = [
            (min[0] - grow) as f32,
            (min[1] - grow) as f32,
            (max[0] + grow) as f32,
            (max[1] + grow) as f32,
        ];

        let style_index = self.intern_style(style);
        self.paths.push(PathHeader {
            first_segment,
            segment_count,
            style: style_index,
            hint: if all_straight {
                PrimitiveHint::Line
            } else {
                PrimitiveHint::General
            },
            kind,
            slab,
        });
        Some(self.paths.len() as u32 - 1)
    }

    /// Bin every path into the tile command lists by its conservative slab.
    ///
    /// Deterministic count → prefix-sum → scatter, in that order and on the
    /// host: the shape §10.8 mandates for the annex, used here for both
    /// engines so the two consume byte-identical command lists and any
    /// divergence the comparison finds is provably in the kernel, not the
    /// binning.
    pub fn bin(&mut self) {
        let ntiles = self.grid.count();
        let mut counts = vec![0u32; ntiles];
        let tile = self.grid.tile as f32;
        let cols = self.grid.cols();
        let rows = self.grid.rows();

        let tile_span = |path: &PathHeader| -> (u32, u32, u32, u32) {
            let x0 = (path.slab[0] / tile).floor().max(0.0) as u32;
            let y0 = (path.slab[1] / tile).floor().max(0.0) as u32;
            let x1 = (path.slab[2] / tile).floor().max(0.0) as u32;
            let y1 = (path.slab[3] / tile).floor().max(0.0) as u32;
            (
                x0.min(cols.saturating_sub(1)),
                y0.min(rows.saturating_sub(1)),
                x1.min(cols.saturating_sub(1)),
                y1.min(rows.saturating_sub(1)),
            )
        };

        for path in &self.paths {
            if path.slab[2] < 0.0 || path.slab[3] < 0.0 {
                continue;
            }
            let (x0, y0, x1, y1) = tile_span(path);
            for ty in y0..=y1 {
                for tx in x0..=x1 {
                    counts[(ty * cols + tx) as usize] += 1;
                }
            }
        }

        let mut offsets = Vec::with_capacity(ntiles + 1);
        let mut running = 0u32;
        offsets.push(0);
        for c in &counts {
            running += c;
            offsets.push(running);
        }

        let mut cursor = offsets.clone();
        let mut draws = vec![0u32; running as usize];
        for (pi, path) in self.paths.iter().enumerate() {
            if path.slab[2] < 0.0 || path.slab[3] < 0.0 {
                continue;
            }
            let (x0, y0, x1, y1) = tile_span(path);
            for ty in y0..=y1 {
                for tx in x0..=x1 {
                    let t = (ty * cols + tx) as usize;
                    draws[cursor[t] as usize] = pi as u32;
                    cursor[t] += 1;
                }
            }
        }

        self.tiles = CommandLists { offsets, draws };
    }

    /// Derive the Metal-shaped layout: one flat array per table, one scalar
    /// type per array.
    pub fn flatten(&self) -> FlatIr {
        let mut seg = Vec::with_capacity(self.segments.len() * 8);
        for s in &self.segments {
            seg.extend_from_slice(&[
                s.p0[0], s.p0[1], s.p1[0], s.p1[1], s.p2[0], s.p2[1], s.s0, s.s1,
            ]);
        }
        let mut path_u32 = Vec::with_capacity(self.paths.len() * 4);
        let mut path_f32 = Vec::with_capacity(self.paths.len() * 4);
        for p in &self.paths {
            path_u32.extend_from_slice(&[p.first_segment, p.segment_count, p.style, p.hint as u32]);
            path_f32.extend_from_slice(&p.slab);
        }
        let mut style = Vec::with_capacity(self.styles.len() * 8);
        for s in &self.styles {
            style.extend_from_slice(&[
                s.rgba[0],
                s.rgba[1],
                s.rgba[2],
                s.rgba[3],
                s.width_start,
                s.width_end,
                s.aa_width,
                0.0,
            ]);
        }
        FlatIr {
            params_u32: vec![
                self.grid.width,
                self.grid.height,
                self.grid.tile,
                self.grid.cols(),
            ],
            params_f32: self.background.to_vec(),
            segments: seg,
            path_u32,
            path_f32,
            styles: style,
            tile_offsets: self.tiles.offsets.clone(),
            // A tile with no draws still needs a valid binding; an empty Metal
            // buffer is an error, not an empty array.
            tile_draws: if self.tiles.draws.is_empty() {
                vec![0]
            } else {
                self.tiles.draws.clone()
            },
        }
    }
}

/// Stride, in scalars, of one [`Segment`] in [`FlatIr::segments`].
pub const SEGMENT_STRIDE: usize = 8;
/// Stride, in scalars, of one [`PathHeader`] in [`FlatIr::path_u32`].
pub const PATH_U32_STRIDE: usize = 4;
/// Stride, in scalars, of one path's slab in [`FlatIr::path_f32`].
pub const PATH_F32_STRIDE: usize = 4;
/// Stride, in scalars, of one [`Style`] in [`FlatIr::styles`].
pub const STYLE_STRIDE: usize = 8;

/// The Metal-shaped derivation of [`RenderIr`]: eight flat, single-typed
/// arrays, bound to buffer indices 0..8 in declaration order.
#[derive(Debug, Clone, PartialEq)]
pub struct FlatIr {
    /// `[width, height, tile, cols]`.
    pub params_u32: Vec<u32>,
    /// Background RGBA, linear.
    pub params_f32: Vec<f32>,
    /// [`SEGMENT_STRIDE`] scalars per segment.
    pub segments: Vec<f32>,
    /// [`PATH_U32_STRIDE`] scalars per path.
    pub path_u32: Vec<u32>,
    /// [`PATH_F32_STRIDE`] scalars per path.
    pub path_f32: Vec<f32>,
    /// [`STYLE_STRIDE`] scalars per style.
    pub styles: Vec<f32>,
    /// CSR offsets, `tiles + 1` entries.
    pub tile_offsets: Vec<u32>,
    /// CSR path indices.
    pub tile_draws: Vec<u32>,
}

impl FlatIr {
    /// Total bytes that cross the CPU↔GPU boundary for one frame — the number
    /// PG-A records as "bytes uploaded per frame" (§17.2).
    pub fn upload_bytes(&self) -> usize {
        4 * (self.params_u32.len()
            + self.params_f32.len()
            + self.segments.len()
            + self.path_u32.len()
            + self.path_f32.len()
            + self.styles.len()
            + self.tile_offsets.len()
            + self.tile_draws.len())
    }
}

fn style_bits(s: &Style) -> [u32; 18] {
    // 4 rgba + 4 rgba_end + 4 gradient_axis + 6 scalars. The array length is
    // asserted by construction: a field added without widening it panics here
    // in every test rather than silently merging two distinct styles.
    let mut b = [0u32; 18];
    for (i, v) in s
        .rgba
        .iter()
        .chain(s.rgba_end.iter())
        .chain(s.gradient_axis.iter())
        .chain(
            [
                s.width_start,
                s.width_end,
                s.aa_width,
                s.glow_radius,
                s.glow_factor,
                s.miter_gain,
            ]
            .iter(),
        )
        .enumerate()
    {
        b[i] = v.to_bits();
    }
    b
}

/// fmn-geom marks a subpath break by repeating the anchor into the handle.
fn is_null_curve(a0: [f64; 3], h: [f64; 3], a1: [f64; 3]) -> bool {
    let d = |p: [f64; 3], q: [f64; 3]| {
        (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs() < 1e-12
    };
    d(a0, h) && d(h, a1)
}

/// The unit-ish tangent at `t = 0`: `B'(0) = 2(p1 - p0)`.
fn tangent_at_start(s: &Segment) -> [f64; 2] {
    [
        2.0 * (s.p1[0] - s.p0[0]) as f64,
        2.0 * (s.p1[1] - s.p0[1]) as f64,
    ]
}

/// The tangent at `t = 1`: `B'(1) = 2(p2 - p1)`.
fn tangent_at_end(s: &Segment) -> [f64; 2] {
    [
        2.0 * (s.p2[0] - s.p1[0]) as f64,
        2.0 * (s.p2[1] - s.p1[1]) as f64,
    ]
}

/// The signed turn from `a` to `b`, wrapped to `(-π, π]`.
///
/// Two `atan2` calls rather than one `atan2` of a cross/dot pair, because
/// §20.1 spike 6 wants per-vertex `atan2` exercised and this is the form a
/// reader recognises. A degenerate (zero-length) tangent yields no turn rather
/// than `atan2(0, 0)`'s platform-defined answer — the kind of edge that is
/// exactly why the certified layer owns its own transcendentals.
fn turn_between(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (na, nb) = (a[0].abs() + a[1].abs(), b[0].abs() + b[1].abs());
    if na <= 0.0 || nb <= 0.0 {
        return 0.0;
    }
    let d = fmn_dmath::atan2(b[1], b[0]) - fmn_dmath::atan2(a[1], a[0]);
    let tau = std::f64::consts::TAU;
    let mut w = d % tau;
    if w > std::f64::consts::PI {
        w -= tau;
    } else if w <= -std::f64::consts::PI {
        w += tau;
    }
    w
}

/// True when the handle sits on the chord, so the quadratic term vanishes and
/// the capsule fast path is exact rather than approximate.
fn is_straight(a0: [f64; 3], h: [f64; 3], a1: [f64; 3]) -> bool {
    let cx = a0[0] - 2.0 * h[0] + a1[0];
    let cy = a0[1] - 2.0 * h[1] + a1[1];
    let scale = (a1[0] - a0[0]).abs() + (a1[1] - a0[1]).abs() + 1.0;
    (cx.abs() + cy.abs()) / scale < 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> Style {
        Style::flat([1.0, 0.0, 0.0, 1.0], 4.0, 1.5)
    }

    fn grid() -> TileGrid {
        TileGrid {
            width: 64,
            height: 64,
            tile: 16,
        }
    }

    #[test]
    fn arc_length_spans_partition_the_unit_interval() {
        let mut path = QuadPath::default();
        path.start_new_path([0.0, 0.0, 0.0]);
        path.add_line_to([10.0, 0.0, 0.0], false).unwrap();
        path.add_line_to([10.0, 30.0, 0.0], false).unwrap();
        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        ir.compile_path(&path, style(), DrawKind::Stroke)
            .expect("compiles");

        assert_eq!(ir.segments.len(), 2);
        assert_eq!(ir.segments[0].s0, 0.0);
        assert_eq!(ir.segments[1].s1, 1.0);
        assert_eq!(ir.segments[0].s1, ir.segments[1].s0);
        // Lengths 10 and 30: the split lands at exactly a quarter.
        assert!((ir.segments[0].s1 - 0.25).abs() < 1e-6, "{:?}", ir.segments);
    }

    #[test]
    fn a_straight_path_earns_the_line_hint_and_a_curved_one_does_not() {
        let mut straight = QuadPath::default();
        straight.start_new_path([0.0, 0.0, 0.0]);
        straight.add_line_to([10.0, 10.0, 0.0], false).unwrap();
        let mut curved = QuadPath::default();
        curved.start_new_path([0.0, 0.0, 0.0]);
        curved
            .add_quadratic_bezier_curve_to([5.0, 12.0, 0.0], [10.0, 0.0, 0.0], false)
            .unwrap();

        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        let a = ir
            .compile_path(&straight, style(), DrawKind::Stroke)
            .unwrap();
        let b = ir.compile_path(&curved, style(), DrawKind::Stroke).unwrap();
        assert_eq!(ir.paths[a as usize].hint, PrimitiveHint::Line);
        assert_eq!(ir.paths[b as usize].hint, PrimitiveHint::General);
    }

    #[test]
    fn the_slab_covers_the_stroke_and_its_whole_aa_band() {
        let mut path = QuadPath::default();
        path.start_new_path([20.0, 20.0, 0.0]);
        path.add_line_to([40.0, 20.0, 0.0], false).unwrap();
        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        ir.compile_path(&path, style(), DrawKind::Stroke).unwrap();
        let slab = ir.paths[0].slab;
        // half-width 2 + aa 1.5 = 3.5 of growth on every side.
        assert!((slab[0] - 16.5).abs() < 1e-5, "{slab:?}");
        assert!((slab[3] - 23.5).abs() < 1e-5, "{slab:?}");
    }

    #[test]
    fn styles_intern_by_bits_and_never_merge_distinct_ones() {
        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        let a = ir.intern_style(style());
        let b = ir.intern_style(style());
        assert_eq!(a, b);
        let mut other = style();
        other.width_end = 4.000001;
        assert_ne!(ir.intern_style(other), a);
        assert_eq!(ir.styles.len(), 2);
    }

    #[test]
    fn binning_is_a_valid_csr_in_painter_order() {
        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        for i in 0..4 {
            let mut p = QuadPath::default();
            let y = 8.0 + i as f64 * 16.0;
            p.start_new_path([2.0, y, 0.0]);
            p.add_line_to([60.0, y, 0.0], false).unwrap();
            ir.compile_path(&p, style(), DrawKind::Stroke).unwrap();
        }
        ir.bin();
        assert_eq!(ir.tiles.offsets.len(), ir.grid.count() + 1);
        assert_eq!(
            *ir.tiles.offsets.last().unwrap() as usize,
            ir.tiles.draws.len()
        );
        for t in 0..ir.grid.count() {
            let run =
                &ir.tiles.draws[ir.tiles.offsets[t] as usize..ir.tiles.offsets[t + 1] as usize];
            assert!(
                run.windows(2).all(|w| w[0] < w[1]),
                "tile {t} run out of painter order: {run:?}"
            );
        }
    }

    #[test]
    fn a_path_with_only_a_subpath_break_produces_no_command() {
        let mut path = QuadPath::default();
        path.start_new_path([1.0, 1.0, 0.0]);
        let mut ir = RenderIr::new(grid(), [0.0; 4]);
        assert!(ir.compile_path(&path, style(), DrawKind::Stroke).is_none());
        assert!(ir.paths.is_empty());
        assert!(ir.segments.is_empty());
    }

    #[test]
    fn flatten_preserves_every_table_at_its_declared_stride() {
        let mut path = QuadPath::default();
        path.start_new_path([0.0, 0.0, 0.0]);
        path.add_quadratic_bezier_curve_to([5.0, 12.0, 0.0], [10.0, 0.0, 0.0], false)
            .unwrap();
        let mut ir = RenderIr::new(grid(), [0.1, 0.2, 0.3, 1.0]);
        ir.compile_path(&path, style(), DrawKind::Stroke).unwrap();
        ir.bin();
        let flat = ir.flatten();
        assert_eq!(flat.segments.len(), ir.segments.len() * SEGMENT_STRIDE);
        assert_eq!(flat.path_u32.len(), ir.paths.len() * PATH_U32_STRIDE);
        assert_eq!(flat.path_f32.len(), ir.paths.len() * PATH_F32_STRIDE);
        assert_eq!(flat.styles.len(), ir.styles.len() * STYLE_STRIDE);
        assert_eq!(flat.tile_offsets.len(), ir.grid.count() + 1);
        assert_eq!(flat.params_f32, vec![0.1, 0.2, 0.3, 1.0]);
        assert!(flat.upload_bytes() > 0);
    }
}
