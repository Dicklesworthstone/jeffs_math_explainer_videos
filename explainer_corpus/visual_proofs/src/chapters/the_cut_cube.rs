//! Chapter 12, the cut cube: the owner's third picture. A square five on a
//! side is cut into a long part and a short part, three and two, and falls
//! into four pieces, a square wearing an L-shaped layer (12_1, 12_2); a cube
//! is cut the same way (12_3); the quiz, how many blocks and how many of
//! each kind (12_4, eight, and one, three, three, one); the blocks gather by
//! kind (12_5); their volumes add to five cubed and the cut slides anywhere
//! without a block appearing or vanishing (12_6); and the fourth power is
//! counted, not built (12_7).
//!
//! Every number is a fact or the model's: the cut `F-B.parts`, the pieces
//! `F-B.square.*`, the blocks `F-B.cube.*` and `F-B.fourth.*`, the quiz's
//! answer `F-Q.kinds.answer`, and Euclid's proposition `F-X.euclid`. The
//! solids are drawn by this chapter's own small renderer ([`Boxes`]): up to
//! sixteen boxes of any size, cut anywhere, projected by the video's fixed
//! viewpoint (`iso::ViewAt`) and painted back to front by the exact rule for
//! boxes that share axes; one updater rewrites every face from five
//! trackers, so a cut that slides is a tracker moving.

use explainer_kit::{Kit, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, CardRow, LENS_AT, ProofCard, QuizCard, QuizLook, Side, Words, breath, dim, fact, fact_ints, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, file_on_shelf, flat, formula_third, hero, ink, lens, lens_mark,
    lower_third, pause, say, size, slide, tex_row,
};
use super::iso::ViewAt;
use crate::model::Rect;

pub struct TheCutCube {
    pub kit: Kit,
}

// ------------------------------------------------------------------ choices

/// The flat board of 12_1 and 12_2: lower left corner and a tile's side.
const BOARD_AT: Vec3 = [-6.3, -1.2, 0.0];
const TILE: f64 = 0.8;
/// The right-hand panel's centre and width.
const PANEL_X: f64 = 2.9;
const PANEL_W: f64 = 7.0;
/// Where the legend chip's rows start (the top row's y) and their pitch.
const LEGEND_X: f64 = 0.55;
const LEGEND_Y: f64 = 2.75;
const LEGEND_PITCH: f64 = 0.52;
/// How far a block's faces stand in from its edges, lattice units: the
/// seams of a cut cube.
const INSET: f64 = 0.04;
/// How far apart the blocks drift, lattice units, at most.
const DRIFT: f64 = 0.7;
/// The gap between two copies of the cube, lattice units (the fourth side).
const PAIR_GAP: f64 = 2.0;
/// The gaps between blocks of the row of 12_5: inside a kind, between kinds.
const ROW_GAP: f64 = 0.55;
const ROW_KIND_GAP: f64 = 1.5;
/// Where each picture sits: the cube at the left, the row across the
/// frame, the two cubes at the left again but smaller.
const ROOM_CUBE: (f64, f64) = (5.6, 4.0);
const CENTRE_CUBE: (f64, f64) = (-3.7, 0.7);
const ROOM_ROW: (f64, f64) = (12.6, 2.8);
const CENTRE_ROW: (f64, f64) = (0.0, 0.6);
const ROOM_PAIR: (f64, f64) = (6.8, 3.0);
const CENTRE_PAIR: (f64, f64) = (-3.4, 0.7);
/// How bright a face is: the top, the front and the right side.
const SHADES: [f64; 3] = [1.0, 0.74, 0.52];
/// The slot this proof takes on the shelf, after the quartered square's.
const SLOT: usize = 6;

// the chapter's fixed words
const NO_SHORT: [&str; 5] = ["none short", "one short", "two short", "three short", "four short"];
const LONG: &str = "long";
const SHORT: &str = "short";
const LEGEND_HEAD: &str = "a block's sides that are short";
const SHORT_SIDES: &str = "short sides:";
const A_IS: &str = "a is the long part, b the short part";
const EUCLID: &str = "Euclid, Elements II.4, in words:";
const PASCAL: &str = "Pascal's triangle:";
const EACH_IS_SUM: &str = "each number is the sum of the two above it";
const ADDED_SHIFTED: &str = "the cube's row, added to itself one place over";
const TIMES: [&str; 2] = ["\u{d7} 3", "\u{d7} 2"];
const FOURTH_ROWS: [&str; 3] = ["square", "cube", "fourth power"];
const QUESTION: &str = "? blocks, and ? of each kind";
const FILL_EXACTLY: &str = "cutting each side once cuts the square and the cube into boxes that fill them exactly";
const SIZE_IS_PRODUCT: &str = "a box's size is the product of its sides";
const THE_FOURTH: &str = "the fourth power: sixteen blocks";
const LENGTHS: &str = "for any two lengths";

// ----------------------------------------------------------- the renderer

type P3 = [f64; 3];

/// The kind colour of a block with `short` short sides: violet for none,
/// then blue, green, copper and pink (storyboard §3, the size ramp).
pub fn kind_colour(short: usize) -> Srgb {
    size(5usize.saturating_sub(short).max(1))
}

fn shade(c: Srgb, f: f64) -> Srgb {
    Srgb {
        r: c.r * f,
        g: c.g * f,
        b: c.b * f,
    }
}

fn blend(a: Srgb, b: Srgb, t: f64) -> Srgb {
    let t = t.clamp(0.0, 1.0);
    Srgb {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// The boxes of the picture: for each of the sixteen (eight blocks of the
/// cube, and eight of a second copy for the fourth power), which of its
/// three sides are the short part.
pub fn sides_short() -> Vec<(usize, [bool; 3])> {
    (0..2)
        .flat_map(|set| (0..8usize).map(move |i| (set, [i & 1 == 1, i >> 1 & 1 == 1, i >> 2 & 1 == 1])))
        .collect()
}

/// What the five trackers say: the long part `a` of the side, how far the
/// blocks have drifted apart, how far they have gathered into the row of
/// kinds, how far the second copy has come in, and how far the colours of
/// the kinds have replaced the neutral grey.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vals {
    pub a: f64,
    pub drift: f64,
    pub row: f64,
    pub pair: f64,
    pub kinds: f64,
}

/// A box placed in the lattice and which kind it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub set: usize,
    pub lo: P3,
    pub hi: P3,
    pub short: usize,
}

/// Where every box is, for the side `total` and the trackers `vals`.
pub fn layout(total: f64, vals: Vals) -> Vec<Placed> {
    let list = sides_short();
    let b = total - vals.a;
    let extent = |bits: [bool; 3]| bits.map(|s| if s { b } else { vals.a });
    // the row: sorted by kind, then by their order, laid along x
    let mut order: Vec<usize> = (0..8).collect();
    order.sort_by_key(|&i| (list[i].1.iter().filter(|&&s| s).count(), i));
    let mut row_at = [[0.0; 3]; 8];
    let mut x = 0.0;
    for (n, &i) in order.iter().enumerate() {
        row_at[i] = [x, 0.0, 0.0];
        let e = extent(list[i].1);
        let next_kind = order.get(n + 1).map(|&j| list[j].1.iter().filter(|&&s| s).count());
        let this_kind = list[i].1.iter().filter(|&&s| s).count();
        x += e[0]
            + if next_kind == Some(this_kind) {
                ROW_GAP
            } else {
                ROW_KIND_GAP
            };
    }
    list.iter()
        .enumerate()
        .map(|(n, &(set, bits))| {
            let i = n % 8;
            let e = extent(bits);
            let cube_at = [0, 1, 2].map(|k| if bits[k] { vals.a + vals.drift * DRIFT } else { 0.0 });
            let at = [0, 1, 2].map(|k| lerp(cube_at[k], row_at[i][k], vals.row));
            let off = if set == 1 { total + PAIR_GAP } else { 0.0 };
            let lo = [at[0] + off, at[1], at[2]];
            Placed {
                set,
                lo,
                hi: [lo[0] + e[0], lo[1] + e[1], lo[2] + e[2]],
                short: bits.iter().filter(|&&s| s).count() + set,
            }
        })
        .collect()
}

/// The three viewpoints the picture moves between, fitted to their rooms:
/// the cube, the row of kinds and the two cubes.
pub fn views(total: f64, a: f64) -> [ViewAt; 3] {
    let base = ViewAt::classic((0.0, 0.0), 1.0);
    let cube = base.fitted(
        [0.0; 3],
        [total + DRIFT, total + DRIFT, total + DRIFT],
        CENTRE_CUBE,
        ROOM_CUBE,
    );
    let at_rest = Vals {
        a,
        drift: 0.0,
        row: 1.0,
        pair: 0.0,
        kinds: 1.0,
    };
    let placed: Vec<Placed> = layout(total, at_rest).into_iter().filter(|p| p.set == 0).collect();
    let hi = placed.iter().fold([0.0f64; 3], |m, p| {
        [m[0].max(p.hi[0]), m[1].max(p.hi[1]), m[2].max(p.hi[2])]
    });
    let row = base.fitted([0.0; 3], hi, CENTRE_ROW, ROOM_ROW);
    let pair = base.fitted([0.0; 3], [2.0 * total + PAIR_GAP, total, total], CENTRE_PAIR, ROOM_PAIR);
    [cube, row, pair]
}

/// The viewpoint for the trackers `vals`: the cube's, the row's and the
/// pair's, blended.
pub fn view_for(views: &[ViewAt; 3], vals: Vals) -> ViewAt {
    let [cube, row, pair] = *views;
    let mix = |x: ViewAt, y: ViewAt, t: f64| ViewAt {
        origin: (lerp(x.origin.0, y.origin.0, t), lerp(x.origin.1, y.origin.1, t)),
        scale: lerp(x.scale, y.scale, t),
        yaw: x.yaw,
        elevation: x.elevation,
    };
    mix(mix(cube, row, vals.row), pair, vals.pair)
}

/// The order to paint boxes in, farthest first, by the exact rule for boxes
/// that share axes: if a plane across one axis separates two boxes, the
/// one on the viewer's side of it is painted later. The viewer stands to
/// the right (+x), in front (−y) and above (+z).
pub fn paint_order(boxes: &[(P3, P3)]) -> Vec<usize> {
    // q is nearer than p, by a plane across one axis: p at the left of q, q
    // in front of p, or p below q
    let nearer =
        |p: (P3, P3), q: (P3, P3)| p.1[0] <= q.0[0] + 1e-9 || q.1[1] <= p.0[1] + 1e-9 || p.1[2] <= q.0[2] + 1e-9;
    // the bigger, the nearer: right, in front, above
    let depth = |b: (P3, P3)| (b.0[0] + b.1[0]) - (b.0[1] + b.1[1]) + (b.0[2] + b.1[2]);
    let n = boxes.len();
    let mut before = vec![Vec::new(); n];
    let mut waits = vec![0usize; n];
    for p in 0..n {
        for q in p + 1..n {
            let (pq, qp) = (nearer(boxes[p], boxes[q]), nearer(boxes[q], boxes[p]));
            // two planes that disagree: the boxes cannot overlap on screen
            let (first, second) = match (pq, qp) {
                (true, false) => (p, q),
                (false, true) => (q, p),
                (true, true) => continue,
                (false, false) if depth(boxes[p]) <= depth(boxes[q]) => (p, q),
                (false, false) => (q, p),
            };
            before[first].push(second);
            waits[second] += 1;
        }
    }
    let mut out = Vec::with_capacity(n);
    let mut ready: Vec<usize> = (0..n).filter(|&i| waits[i] == 0).collect();
    while let Some(i) = ready.first().copied() {
        ready.remove(0);
        out.push(i);
        for &j in &before[i] {
            waits[j] -= 1;
            if waits[j] == 0 {
                ready.push(j);
                ready.sort_unstable();
            }
        }
    }
    // a cycle (boxes that interlock: not in this chapter) is painted by depth
    for i in 0..n {
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

/// The three visible faces of a box on screen, top, front, right, each as
/// a closed run of corners (inset by [`INSET`]).
pub fn face_quads(view: &ViewAt, lo: P3, hi: P3) -> [[Vec3; 5]; 3] {
    let (l, h) = (lo.map(|x| x + INSET), hi.map(|x| x - INSET));
    let p = |x: f64, y: f64, z: f64| view.project([x, y, z]);
    let top = [
        p(l[0], l[1], h[2]),
        p(h[0], l[1], h[2]),
        p(h[0], h[1], h[2]),
        p(l[0], h[1], h[2]),
    ];
    let front = [
        p(l[0], l[1], l[2]),
        p(h[0], l[1], l[2]),
        p(h[0], l[1], h[2]),
        p(l[0], l[1], h[2]),
    ];
    let right = [
        p(h[0], l[1], l[2]),
        p(h[0], h[1], l[2]),
        p(h[0], h[1], h[2]),
        p(h[0], l[1], h[2]),
    ];
    [top, front, right].map(|q| [q[0], q[1], q[2], q[3], q[0]])
}

/// The picture's boxes on the stage: sixteen boxes of three faces each,
/// children of one group whose child order is the paint order, and the
/// trackers that move them.
pub struct Boxes {
    pub group: Mob,
    /// The long part, in lattice units.
    pub cut: Mob,
    pub drift: Mob,
    pub row: Mob,
    pub pair: Mob,
    pub kinds: Mob,
    /// How much of the whole picture shows, 0 to 1.
    pub on: Mob,
    pub anchor: Mob,
    pub faces: Vec<[Mob; 3]>,
}

impl Boxes {
    /// The boxes for a side of `total` cut at `a`, detached, at rest as a
    /// cube and not shown ([`Boxes::start`] puts them on the stage).
    pub fn build(stage: &mut Stage<'_>, total: f64, a: f64) -> fmn::Result<Self> {
        let list = sides_short();
        let mut faces: Vec<[Mob; 3]> = Vec::new();
        for _ in &list {
            let one = [0, 1, 2].map(|_| {
                stage.arena_mut().add(
                    Polygon::new([v(0.0, 0.0), v(1e-3, 0.0), v(0.0, 1e-3)])
                        .style(flat(dim(), 0.0))
                        .build(),
                )
            });
            faces.push(one);
        }
        let flat_faces: Vec<Mob> = faces.iter().flatten().copied().collect();
        let group = super::draw::group(stage, &flat_faces)?;
        let [cut, drift, row, pair, kinds, on] = [a, 0.0, 0.0, 0.0, 0.0, 0.0].map(|x| stage.add_value_tracker(x));
        let anchor = stage.arena_mut().add(VMobject::new());
        let all = views(total, a);
        let (face_ids, group_id) = (faces.clone(), group);
        let mut last = [f64::NAN; 6];
        let mut painted: Vec<Mob> = flat_faces;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let get = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let now = [get(cut), get(drift), get(row), get(pair), get(kinds), get(on)];
                if now.iter().zip(&last).all(|(x, y)| x.to_bits() == y.to_bits()) {
                    return;
                }
                last = now;
                let vals = Vals {
                    a: now[0],
                    drift: now[1],
                    row: now[2],
                    pair: now[3],
                    kinds: now[4],
                };
                let placed = layout(total, vals);
                let view = view_for(&all, vals);
                // paint each copy in its own order, the second after the first
                let mut order: Vec<usize> = Vec::new();
                for set in 0..2 {
                    let idx: Vec<usize> = (0..placed.len()).filter(|&i| placed[i].set == set).collect();
                    let boxes: Vec<(P3, P3)> = idx.iter().map(|&i| (placed[i].lo, placed[i].hi)).collect();
                    order.extend(paint_order(&boxes).into_iter().map(|k| idx[k]));
                }
                let mut kids: Vec<Mob> = Vec::with_capacity(order.len() * 3);
                for &i in &order {
                    let p = placed[i];
                    let quads = face_quads(&view, p.lo, p.hi);
                    let colour = blend(dim(), kind_colour(p.short), vals.kinds);
                    // the second copy comes in with the pair; all fade with `on`
                    let opacity = now[5] * if p.set == 1 { vals.pair } else { 1.0 };
                    for (f, quad) in quads.iter().enumerate() {
                        let m = face_ids[i][f];
                        let _ = s.set_points(m, &super::draw::corners(quad));
                        s.set_fill(m, Some(shade(colour, SHADES[f])), Some(opacity), None, false);
                        kids.push(m);
                    }
                }
                if kids != painted {
                    let _ = s.replace_children(group_id, &kids);
                    painted = kids;
                }
            },
            true,
        )?;
        Ok(Self {
            group,
            cut,
            drift,
            row,
            pair,
            kinds,
            on,
            anchor,
            faces,
        })
    }

    /// Put the boxes on the stage (shown when `on` rises).
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        stage.add_to_scene(self.group)?;
        Ok(())
    }

    /// Stop the updater and take the faces as plain mobjects.
    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// Every face (to move them as a picture).
    pub fn all_faces(&self) -> Vec<Mob> {
        self.faces.iter().flatten().copied().collect()
    }
}

// --------------------------------------------------------- small drawings

/// A `w` by `h` rectangle centred at `at`, detached.
fn block(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, style: Style) -> fmn::Result<Mob> {
    let r = Rectangle::new().width(w).height(h).style(style).build()?.moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// The legend chip: five swatches and their words, a block's colour saying
/// how many of its sides are short. Detached; returns every mobject.
fn legend(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    out.push(kit.text_left(stage, LEGEND_HEAD, 24.0, dim(), v(LEGEND_X - 0.1, LEGEND_Y + 0.55))?);
    for (k, words) in NO_SHORT.iter().enumerate() {
        let y = LEGEND_Y - LEGEND_PITCH * k as f64;
        out.push(block(
            stage,
            v(LEGEND_X + 0.2, y),
            0.36,
            0.36,
            flat(kind_colour(k), 1.0),
        )?);
        out.push(kit.text_left(stage, words, 26.0, ink(), v(LEGEND_X + 0.62, y))?);
    }
    Ok(out)
}

/// The same chip in one row across the top, for the picture that fills the
/// frame (12_5). Detached; returns every mobject.
fn legend_row(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let y = 3.4;
    let mut out = vec![kit.text_left(stage, SHORT_SIDES, 24.0, dim(), v(-6.7, y))?];
    for (k, words) in ["none", "one", "two", "three", "four"].iter().enumerate() {
        let x = -3.9 + 2.0 * k as f64;
        out.push(block(stage, v(x, y), 0.32, 0.32, flat(kind_colour(k), 1.0))?);
        out.push(kit.text_left(stage, words, 26.0, ink(), v(x + 0.3, y))?);
    }
    Ok(out)
}

/// A fact that is a list of whole numbers as TeX terms, repeating each
/// value as many times as its kind occurs (`9,6,4` with `1,2,1` is
/// `9 + 6 + 6 + 4`), each term in its kind's colour, then `= total`.
fn sum_parts(each: &[i64], counts: &[i64], total: i64) -> Vec<(String, Srgb)> {
    let mut parts = Vec::new();
    for (k, (&value, &n)) in each.iter().zip(counts).enumerate() {
        for _ in 0..n {
            if !parts.is_empty() {
                parts.push(("+".to_owned(), ink()));
            }
            parts.push((value.to_string(), kind_colour(k)));
        }
    }
    parts.push((format!("= {total}"), ink()));
    parts
}

impl SceneConstruct for TheCutCube {
    fn name(&self) -> &str {
        "12_the_cut_cube"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let parts = fact_ints("F-B.parts")?;
        let (long, short) = (parts[0], parts[1]);
        let total = long + short;
        let side = total as f64;
        let board = Board::new(BOARD_AT, TILE);
        let frame = Rect::new(0, 0, total, total);
        // the four pieces of the square: none short, one, one, two
        let big = Rect::new(0, 0, long, long);
        let right = Rect::new(long, 0, short, long);
        let top = Rect::new(0, long, long, short);
        let small = Rect::new(long, long, short, short);

        // ------------------------------------------------------------ 12_1
        // a square, five on a side, each side cut into three and two
        let line = say(&kit, stage, "12_1")?;
        let cells: Vec<(i64, i64)> = frame.cells().collect();
        let whole = board.tiles(stage, &cells, dim())?;
        let cut_across = stage.arena_mut().add(
            polyline(vec![board.p(0.0, long as f64), board.p(side, long as f64)])
                .style(faded_line(ink(), 4.0, 1.0))
                .build(),
        );
        let cut_down = stage.arena_mut().add(
            polyline(vec![board.p(long as f64, 0.0), board.p(long as f64, side)])
                .style(faded_line(ink(), 4.0, 1.0))
                .build(),
        );
        let chip = legend(&kit, stage)?;
        let width_long = board.dimension(
            &kit,
            stage,
            big,
            Side::Below,
            Words::Text(&format!("{long} {LONG}")),
            28.0,
            ink(),
        )?;
        let width_short = board.dimension(
            &kit,
            stage,
            right,
            Side::Below,
            Words::Text(&format!("{short} {SHORT}")),
            28.0,
            ink(),
        )?;
        fade_in_all(stage, &[whole], 0.8)?;
        line.wait_for(stage, "cut each side")?;
        fade_in_all(stage, &[cut_across, cut_down], 0.7)?;
        fade_in_all(stage, &chip, 0.6)?;
        line.wait_for(stage, "Three and two")?;
        let mut dims = width_long.mobs().to_vec();
        dims.extend(width_short.mobs());
        fade_in_all(stage, &dims, 0.6)?;

        // ------------------------------------------------------------ 12_2
        // four pieces; nine, six, six and four make twenty-five
        let line = say(&kit, stage, "12_2")?;
        let pieces = [
            board.rect_tiles(stage, big, kind_colour(0))?,
            board.rect_tiles(stage, right, kind_colour(1))?,
            board.rect_tiles(stage, top, kind_colour(1))?,
            board.rect_tiles(stage, small, kind_colour(2))?,
        ];
        line.wait_for(stage, "A big square")?;
        fade_in_all(stage, &pieces, 0.8)?;
        fade_out_all(stage, &[whole], 0.3)?;
        let each = fact_ints("F-B.square.each")?;
        let counts = fact_ints("F-B.square.kinds")?;
        let square_total: i64 = fact("F-B.square.total")?
            .parse()
            .map_err(|_| fail("F-B.square.total is not a whole number".into()))?;
        let terms = sum_parts(&each, &counts, square_total);
        let term_refs: Vec<(&str, Srgb)> = terms.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let sum = tex_row(&kit, stage, &term_refs, 48.0, v(PANEL_X + 0.4, 0.2), 0.2, PANEL_W - 1.0)?;
        line.wait_for(stage, "Nine, six, six and four")?;
        fade_in_lagged(stage, &sum, 1.4, 0.15)?;
        // a square wearing an L-shaped layer: the L pulls away and returns
        line.wait_for(stage, "A square wearing")?;
        let layer = [pieces[1], pieces[2], pieces[3]];
        let step = board.step(0.55, 0.55);
        let mut out_anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &layer {
            out_anims.push(stage.prepare(slide(m, step, 0.8)?)?);
        }
        stage.play_prepared(out_anims)?;
        pause(stage, 0.3)?;
        let mut back_anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &layer {
            back_anims.push(stage.prepare(slide(m, v(-step[0], -step[1]), 0.8)?)?);
        }
        stage.play_prepared(back_anims)?;
        let euclid = formula_third(
            &kit,
            stage,
            EUCLID,
            &[Words::Tex("(a+b)^{2} = a^{2} + 2ab + b^{2}"), Words::Text(A_IS)],
        )?;
        line.wait_end(stage)?;
        fade_in_all(stage, &[euclid], 0.5)?;
        pause(stage, 0.6)?;

        // ------------------------------------------------------------ 12_3
        // the same to a cube: three cuts, as seams
        let line = say(&kit, stage, "12_3")?;
        let mut flat_leaving: Vec<Mob> = pieces.to_vec();
        flat_leaving.extend(&dims);
        flat_leaving.extend(&sum);
        flat_leaving.extend([cut_across, cut_down, euclid]);
        flat_leaving.extend(&chip);
        fade_out_all(stage, &flat_leaving, 0.6)?;
        let cube = Boxes::build(stage, side, long as f64)?;
        cube.start(stage)?;
        stage.play(tween(cube.on, 1.0, 0.8)?)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 12_4
        // your turn: how many blocks, and how many of each kind?
        let answer_kinds = fact("F-Q.kinds.answer")?;
        let blocks_n = fact("F-B.cube.blocks")?;
        let answer_text = format!("{blocks_n} blocks: {}", answer_kinds.replace(',', ", "));
        let line = say(&kit, stage, "12_4")?;
        let card = QuizCard::build(
            &kit,
            stage,
            "kinds",
            Words::Text(QUESTION),
            Words::Text(&answer_text),
            QuizLook::BESIDE,
        )?;
        line.wait_sentence(stage, 2)?;
        card.show(stage, 0.5)?;
        card.count(&kit, stage)?;

        // ------------------------------------------------------------ 12_5
        // eight blocks, in four kinds: one, three, three, one
        let line = say(&kit, stage, "12_5")?;
        fade_out_all(stage, &card.mobs(), 0.4)?;
        let counts_cube = fact_ints("F-B.cube.kinds")?;
        line.wait_for(stage, "Eight blocks")?;
        play_all(stage, &[(cube.kinds, 1.0, 0.8), (cube.drift, 1.0, 0.9)])?;
        play_all(stage, &[(cube.row, 1.0, 1.8)])?;
        // each cluster named by how many blocks it holds
        let all = views(side, long as f64);
        let at_row = Vals {
            a: long as f64,
            drift: 1.0,
            row: 1.0,
            pair: 0.0,
            kinds: 1.0,
        };
        let placed = layout(side, at_row);
        let view = view_for(&all, at_row);
        let mut tags = Vec::new();
        for (k, &n) in counts_cube.iter().enumerate() {
            let mine: Vec<&Placed> = placed.iter().filter(|p| p.set == 0 && p.short == k).collect();
            let (x0, x1) = mine.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(l, r), p| {
                (
                    l.min(view.project(p.lo)[0]),
                    r.max(view.project([p.hi[0], p.lo[1], p.lo[2]])[0]),
                )
            });
            let base = view.project([0.0, 0.0, 0.0])[1];
            tags.push(kit.text(
                stage,
                &n.to_string(),
                52.0,
                kind_colour(k),
                v((x0 + x1) / 2.0, base - 0.75),
            )?);
        }
        line.wait_for(stage, "One, three, three, one")?;
        fade_in_lagged(stage, &tags, 1.2, 0.3)?;
        let chip_again = legend_row(&kit, stage)?;
        fade_in_all(stage, &chip_again, 0.5)?;
        pause(stage, 0.6)?;

        // ------------------------------------------------------------ 12_6
        // the blocks add up to five cubed, and the cut slides anywhere
        let line = say(&kit, stage, "12_6")?;
        fade_out_all(stage, &chip_again, 0.4)?;
        let each_cube = fact_ints("F-B.cube.each")?;
        let by_kind = fact_ints("F-B.cube.by_kind")?;
        let cube_total = fact("F-B.cube.total")?;
        let volumes: Vec<Mob> = {
            let mut out = Vec::new();
            for (k, &vol) in each_cube.iter().enumerate() {
                let at = tags[k];
                let c = stage.get_center(at);
                out.push(kit.text(stage, &vol.to_string(), 36.0, kind_colour(k), v(c[0], c[1] - 0.6))?);
            }
            out
        };
        line.wait_for(stage, "The blocks add up")?;
        fade_out_all(stage, &tags, 0.3)?;
        fade_in_all(stage, &volumes, 0.6)?;
        let by_parts: Vec<(String, Srgb)> = {
            let mut p = Vec::new();
            for (k, &x) in by_kind.iter().enumerate() {
                if k > 0 {
                    p.push(("+".to_owned(), ink()));
                }
                p.push((x.to_string(), kind_colour(k)));
            }
            p.push((format!("= {cube_total}"), ink()));
            p
        };
        let by_refs: Vec<(&str, Srgb)> = by_parts.iter().map(|(s, c)| (s.as_str(), *c)).collect();
        let by_row = tex_row(&kit, stage, &by_refs, 46.0, v(0.0, -3.1), 0.22, 9.0)?;
        fade_in_all(stage, &by_row, 0.7)?;
        line.wait_for(stage, "Five cubed")?;
        flash_text(stage, &by_row)?;
        // back into the cube, and the cut slides
        line.wait_for(stage, "Slide the cut")?;
        let mut leaving = volumes.clone();
        leaving.extend(&by_row);
        fade_out_all(stage, &leaving, 0.4)?;
        play_all(stage, &[(cube.row, 0.0, 1.4), (cube.drift, 0.0, 1.0)])?;
        let formula = tex_row(
            &kit,
            stage,
            &[
                ("(a+b)^{3} =", ink()),
                ("a^{3}", kind_colour(0)),
                ("+", ink()),
                ("3a^{2}b", kind_colour(1)),
                ("+", ink()),
                ("3ab^{2}", kind_colour(2)),
                ("+", ink()),
                ("b^{3}", kind_colour(3)),
            ],
            44.0,
            v(PANEL_X, 2.5),
            0.2,
            PANEL_W,
        )?;
        let words = kit.text(stage, A_IS, 24.0, dim(), v(PANEL_X, 1.75))?;
        fade_in_all(stage, &formula, 0.6)?;
        fade_in_all(stage, &[words], 0.4)?;
        // live from the cut: the two parts, and the volume of each kind
        let cut = cube.cut;
        let whole = side;
        let a_of = move |s: &fmn::mobject::Stage| s.tracker_value(cut).unwrap_or(0.0);
        let mut live: Vec<Mob> = Vec::new();
        let lengths = [("a =", 0.5, kind_colour(0)), ("b =", 3.4, kind_colour(3))];
        for (k, (name, x, colour)) in lengths.into_iter().enumerate() {
            live.push(kit.text_left(stage, name, 30.0, colour, v(x, 0.95))?);
            live.push(kit.live_value(
                stage,
                move |s| if k == 0 { a_of(s) } else { whole - a_of(s) },
                1,
                false,
                40.0,
                colour,
                v(x + 0.8, 0.95),
                LEFT,
            )?);
        }
        let volume_of = |k: usize| {
            move |s: &fmn::mobject::Stage| {
                let (a, b) = (a_of(s), whole - a_of(s));
                [a * a * a, 3.0 * a * a * b, 3.0 * a * b * b, b * b * b][k]
            }
        };
        for k in 0..4 {
            live.push(kit.live_value(
                stage,
                volume_of(k),
                1,
                false,
                40.0,
                kind_colour(k),
                v(0.5 + 1.6 * k as f64, 0.1),
                LEFT,
            )?);
        }
        let slide_to = (long + 1) as f64;
        play_all(stage, &[(cube.cut, slide_to, 1.6)])?;
        play_all(stage, &[(cube.cut, long as f64, 1.6)])?;
        line.wait_end(stage)?;

        // ------------------------------------------------------------ 12_7
        // the fourth power: counted, not built
        let line = say(&kit, stage, "12_7")?;
        let mut done = formula.clone();
        done.push(words);
        done.extend(&live);
        fade_out_all(stage, &done, 0.4)?;
        line.wait_for(stage, "Every block of the cube appears twice")?;
        play_all(stage, &[(cube.pair, 1.0, 1.4)])?;
        let pair_view = views(side, long as f64)[2];
        let blocks_four = fact("F-B.fourth.blocks")?;
        let mut tags = Vec::new();
        for (set, words) in TIMES.iter().enumerate() {
            let off = if set == 0 { 0.0 } else { side + PAIR_GAP };
            let x0 = pair_view.project([off, 0.0, 0.0]);
            let x1 = pair_view.project([off + side, 0.0, 0.0]);
            tags.push(kit.text(stage, words, 40.0, hero(), v((x0[0] + x1[0]) / 2.0, x0[1] - 0.7))?);
        }
        let dashed = {
            let a = pair_view.project([side, side / 2.0, side / 2.0]);
            let b = pair_view.project([side + PAIR_GAP, side / 2.0, side / 2.0]);
            let d = DashedLine::new(a, b)
                .dash_length(0.12)
                .style(faded_line(dim(), 3.0, 1.0))
                .build()?;
            stage.arena_mut().add(d)
        };
        let count = kit.text(stage, &blocks_four, 56.0, hero(), v(-3.4, 3.1))?;
        fade_in_all(stage, &tags, 0.5)?;
        fade_in_all(stage, &[dashed, count], 0.5)?;
        // rows 1 2 1, 1 3 3 1, 1 4 6 4 1; each the sum of the two above it
        line.wait_for(stage, "One, four, six, four, one")?;
        let rows: Vec<Vec<i64>> = ["F-B.square.kinds", "F-B.cube.kinds", "F-B.fourth.kinds"]
            .iter()
            .map(|k| fact_ints(k))
            .collect::<fmn::Result<_>>()?;
        let mut pascal: Vec<Mob> = Vec::new();
        for (r, row) in rows.iter().enumerate() {
            let y = 2.3 - 0.9 * r as f64;
            pascal.push(kit.text_left(stage, FOURTH_ROWS[r], 24.0, dim(), v(0.4, y))?);
            for (i, &n) in row.iter().enumerate() {
                let x = 4.6 + 0.95 * (i as f64 - (row.len() as f64 - 1.0) / 2.0);
                pascal.push(kit.text(stage, &n.to_string(), 44.0, kind_colour(i), v(x, y))?);
            }
        }
        fade_in_lagged(stage, &pascal, 2.2, 0.08)?;
        // the middle three numbers are the cube's row added to itself, one
        // place over (1+3, 3+3, 3+1)
        let sums = fact("F-B.fourth.from_cube")?;
        let sum_row = kit.tex(stage, &sums.replace(',', "\\quad "), 36.0, dim(), &[], v(4.6, -0.45))?;
        let from = kit.text(stage, ADDED_SHIFTED, 21.0, dim(), v(3.9, -1.05))?;
        fade_in_all(stage, &[sum_row, from], 0.5)?;
        let name = lower_third(&kit, stage, PASCAL, EACH_IS_SUM)?;
        fade_in_all(stage, &[name], 0.5)?;
        // the card: the square and the cube are pictures; the fourth power
        // is counted, not built
        line.wait_for(stage, "We can count its blocks")?;
        fade_out_all(stage, &[name], 0.3)?;
        let shown = format!(
            "\\text{{{LENGTHS} }} a \\text{{ and }} b\\text{{: }} (a+b)^{{2}}=a^{{2}}+2ab+b^{{2}} \\text{{ and }} (a+b)^{{3}}=a^{{3}}+3a^{{2}}b+3ab^{{2}}+b^{{3}}\\text{{, each block counted once}}"
        );
        let mut proof = ProofCard::build(
            &kit,
            stage,
            &[
                CardRow::Shown(Words::Tex(&shown)),
                CardRow::Assumed(Words::Text(FILL_EXACTLY)),
                CardRow::Counted(Words::Text(THE_FOURTH)),
            ],
        )?;
        proof.show(stage, 0.6)?;
        let ring = lens(stage);
        let unit = 0.55;
        let mut close = vec![ring];
        for (dx, dy, w, h, k) in [
            (-0.5, -0.5, 1.0, 1.0, 0),
            (0.45, -0.5, 0.7, 1.0, 1),
            (-0.5, 0.45, 1.0, 0.7, 1),
            (0.45, 0.45, 0.7, 0.7, 2),
        ] {
            close.push(block(
                stage,
                v(LENS_AT[0] + dx * unit * 2.0, LENS_AT[1] + dy * unit * 2.0),
                w * unit * 1.9,
                h * unit * 1.9,
                flat(kind_colour(k), 1.0),
            )?);
        }
        fade_in_all(stage, &close, 0.5)?;
        proof.tick(&kit, stage, 1, SIZE_IS_PRODUCT)?;
        let tick = lens_mark(stage, LENS_AT, 0.7, true);
        fade_in_all(stage, &[tick], 0.3)?;
        line.wait_end(stage)?;
        pause(stage, 0.4)?;
        // filed: the cube and its copy go to the shelf
        let mut leaving = close.clone();
        leaving.push(tick);
        leaving.extend(proof.mobs());
        leaving.extend(&pascal);
        leaving.extend([sum_row, from]);
        fade_out_all(stage, &leaving, 0.4)?;
        cube.stop(stage);
        let mut shelved = cube.all_faces();
        shelved.extend(&tags);
        shelved.extend([dashed, count]);
        let mid = v(CENTRE_PAIR.0, CENTRE_PAIR.1);
        file_on_shelf(stage, &shelved, mid, ROOM_PAIR.0, SLOT, 1.0)?;
        breath(&kit, stage)?;

        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// Several trackers to values, each at its own length, all at once.
fn play_all(stage: &mut Stage<'_>, moves: &[(Mob, f64, f64)]) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(tracker, value, rt) in moves {
        anims.push(stage.prepare(tween(tracker, value, rt)?)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Flash a row of terms (all of them, together).
fn flash_text(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    super::draw::flash(stage, mobs, hero(), 1.1, 0.7)
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;
    use crate::model::blocks;

    fn parts() -> (f64, f64) {
        let p = fact_ints("F-B.parts").unwrap();
        (p[0] as f64, (p[0] + p[1]) as f64)
    }

    fn volume(p: &Placed) -> f64 {
        (0..3).map(|k| p.hi[k] - p.lo[k]).product()
    }

    /// 12_1 to 12_6: the pieces, the kinds, the volumes and the totals are
    /// the facts, and the model's blocks are the ones the renderer places.
    #[test]
    fn the_counts_and_volumes_are_the_facts_and_the_models_blocks() {
        let (a, total) = parts();
        assert_eq!(fact("F-B.parts").unwrap(), "3,2");
        for (name, dim) in [("square", 2usize), ("cube", 3), ("fourth", 4)] {
            let cut = blocks::cut(3, 2, dim);
            assert_eq!(fact(&format!("F-B.{name}.blocks")).unwrap(), cut.len().to_string());
            assert_eq!(
                fact(&format!("F-B.{name}.total")).unwrap(),
                cut.iter().map(blocks::Block::volume).sum::<i64>().to_string()
            );
            assert_eq!(
                fact(&format!("F-B.{name}.power")).unwrap(),
                5i64.pow(dim as u32).to_string()
            );
        }
        assert_eq!(fact("F-B.square.each").unwrap(), "9,6,4");
        assert_eq!(fact("F-B.square.kinds").unwrap(), "1,2,1");
        assert_eq!(fact("F-B.cube.each").unwrap(), "27,18,12,8");
        assert_eq!(fact("F-B.cube.kinds").unwrap(), "1,3,3,1");
        assert_eq!(fact("F-B.cube.by_kind").unwrap(), "27,54,36,8");
        assert_eq!(fact("F-B.cube.total").unwrap(), "125");
        assert_eq!(fact("F-B.fourth.kinds").unwrap(), "1,4,6,4,1");
        assert_eq!(fact("F-B.fourth.from_cube").unwrap(), "1+3,3+3,3+1");
        assert_eq!(fact("F-Q.kinds.answer").unwrap(), "1,3,3,1");
        assert_eq!(fact("F-X.euclid").unwrap(), "II.4");
        // the sum of 12_2 is built from the facts
        let terms = sum_parts(
            &fact_ints("F-B.square.each").unwrap(),
            &fact_ints("F-B.square.kinds").unwrap(),
            25,
        );
        let text: Vec<&str> = terms.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(text, ["9", "+", "6", "+", "6", "+", "4", "= 25"]);
        // the renderer's blocks at rest are the cut of the cube: the model's
        // volumes, one of each kind by number of short sides
        let at_rest = Vals {
            a,
            drift: 0.0,
            row: 0.0,
            pair: 0.0,
            kinds: 1.0,
        };
        let placed: Vec<Placed> = layout(total, at_rest).into_iter().filter(|p| p.set == 0).collect();
        assert_eq!(placed.len(), 8);
        let mut by_kind = [0.0; 4];
        let mut count = [0usize; 4];
        for p in &placed {
            by_kind[p.short] += volume(p);
            count[p.short] += 1;
        }
        assert_eq!(count, [1, 3, 3, 1]);
        assert_eq!(by_kind, [27.0, 54.0, 36.0, 8.0]);
        assert_eq!(by_kind.iter().sum::<f64>(), total.powi(3));
        // the model's blocks list the same kinds
        let model = blocks::cut(3, 2, 3);
        for (k, &n) in count.iter().enumerate() {
            assert_eq!(model.iter().filter(|b| b.bs == k).count(), n);
        }
    }

    /// The renderer: at rest the eight blocks fill the cube exactly (no two
    /// overlap), and for any cut the volumes by kind are a³, 3a²b, 3ab² and
    /// b³, which add to (a + b)³.
    #[test]
    fn the_blocks_fill_the_cube_for_every_cut_and_add_to_the_whole() {
        let total = 5.0;
        for step in 0..=20 {
            let a = 1.0 + 3.0 * f64::from(step) / 20.0;
            let b = total - a;
            let placed: Vec<Placed> = layout(
                total,
                Vals {
                    a,
                    drift: 0.0,
                    row: 0.0,
                    pair: 0.0,
                    kinds: 0.0,
                },
            )
            .into_iter()
            .filter(|p| p.set == 0)
            .collect();
            let sum: f64 = placed.iter().map(volume).sum();
            assert!((sum - total.powi(3)).abs() < 1e-9, "a = {a}: {sum}");
            for (i, p) in placed.iter().enumerate() {
                for q in &placed[i + 1..] {
                    let apart = (0..3).any(|k| p.hi[k] <= q.lo[k] + 1e-9 || q.hi[k] <= p.lo[k] + 1e-9);
                    assert!(apart, "a = {a}: two blocks overlap");
                }
            }
            let kind_volume = |k: usize| placed.iter().filter(|p| p.short == k).map(volume).sum::<f64>();
            let want = [a.powi(3), 3.0 * a * a * b, 3.0 * a * b * b, b.powi(3)];
            for (k, &w) in want.iter().enumerate() {
                assert!((kind_volume(k) - w).abs() < 1e-9, "a = {a}, kind {k}");
            }
        }
    }

    /// The painter's order is right: for sample points over the picture, the
    /// block painted last over the point is the block a ray from the viewer
    /// meets first. Checked on the cube, drifted apart, in the row, midway,
    /// cut at 3.5 and as two copies.
    #[test]
    fn the_paint_order_shows_the_nearest_block_everywhere() {
        let total = 5.0;
        let all = views(total, 3.0);
        let cases = [
            Vals {
                a: 3.0,
                drift: 0.0,
                row: 0.0,
                pair: 0.0,
                kinds: 1.0,
            },
            Vals {
                a: 3.0,
                drift: 1.0,
                row: 0.0,
                pair: 0.0,
                kinds: 1.0,
            },
            Vals {
                a: 3.0,
                drift: 1.0,
                row: 1.0,
                pair: 0.0,
                kinds: 1.0,
            },
            Vals {
                a: 3.5,
                drift: 0.0,
                row: 0.0,
                pair: 0.0,
                kinds: 1.0,
            },
            Vals {
                a: 3.0,
                drift: 0.0,
                row: 0.0,
                pair: 1.0,
                kinds: 1.0,
            },
        ];
        for vals in cases {
            let placed = layout(total, vals);
            let view = view_for(&all, vals);
            let mut checked = 0;
            for set in 0..2 {
                let idx: Vec<usize> = (0..placed.len()).filter(|&i| placed[i].set == set).collect();
                let boxes: Vec<(P3, P3)> = idx.iter().map(|&i| (placed[i].lo, placed[i].hi)).collect();
                let order = paint_order(&boxes);
                assert_eq!(order.len(), boxes.len());
                // screen sample points over the set's bounding box
                let quads: Vec<[[Vec3; 5]; 3]> = boxes.iter().map(|&(lo, hi)| face_quads(&view, lo, hi)).collect();
                let (mut lo_s, mut hi_s) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
                for q in quads.iter().flatten().flatten() {
                    lo_s = [lo_s[0].min(q[0]), lo_s[1].min(q[1])];
                    hi_s = [hi_s[0].max(q[0]), hi_s[1].max(q[1])];
                }
                let toward = view.toward_viewer();
                let (right, up) = screen_axes(&view);
                for gx in 0..60 {
                    for gy in 0..40 {
                        let sx = lo_s[0] + (hi_s[0] - lo_s[0]) * (f64::from(gx) + 0.37) / 60.0;
                        let sy = lo_s[1] + (hi_s[1] - lo_s[1]) * (f64::from(gy) + 0.61) / 40.0;
                        // painted: the last block in the order with a face over the point
                        let mut painted = None;
                        for &k in &order {
                            if quads[k].iter().any(|q| inside(q, sx, sy)) {
                                painted = Some(k);
                            }
                        }
                        // geometry: the box the ray from the viewer meets first
                        let u = (sx - view.origin.0) / view.scale;
                        let w = (sy - view.origin.1) / view.scale;
                        let p0 = [
                            u * right[0] + w * up[0],
                            u * right[1] + w * up[1],
                            u * right[2] + w * up[2],
                        ];
                        let mut nearest: Option<(usize, f64)> = None;
                        for (k, &(lo, hi)) in boxes.iter().enumerate() {
                            let (lo, hi) = (lo.map(|x| x + INSET), hi.map(|x| x - INSET));
                            let (mut t0, mut t1) = (f64::NEG_INFINITY, f64::INFINITY);
                            let mut miss = false;
                            for ax in 0..3 {
                                if toward[ax].abs() < 1e-12 {
                                    if p0[ax] < lo[ax] || p0[ax] > hi[ax] {
                                        miss = true;
                                    }
                                } else {
                                    let (a, b) = ((lo[ax] - p0[ax]) / toward[ax], (hi[ax] - p0[ax]) / toward[ax]);
                                    t0 = t0.max(a.min(b));
                                    t1 = t1.min(a.max(b));
                                }
                            }
                            if !miss && t0 < t1 - 1e-9 && nearest.is_none_or(|(_, t)| t1 > t) {
                                nearest = Some((k, t1));
                            }
                        }
                        // points on a silhouette or in a seam are skipped by
                        // the margin (the sample grid is off-lattice)
                        assert_eq!(
                            painted,
                            nearest.map(|(k, _)| k),
                            "{vals:?} set {set} at ({sx:.2}, {sy:.2})"
                        );
                        checked += 1;
                    }
                }
            }
            assert!(checked > 0);
        }
    }

    /// The lattice's screen right and up (the view's own axes).
    fn screen_axes(view: &ViewAt) -> (P3, P3) {
        let o = view.project([0.0, 0.0, 0.0]);
        let to = |p: P3| {
            let s = view.project(p);
            [(s[0] - o[0]) / view.scale, (s[1] - o[1]) / view.scale]
        };
        // right: the lattice point whose screen image is (1, 0) and which
        // is square to the viewer; up likewise
        let (ex, ey, ez) = (to([1.0, 0.0, 0.0]), to([0.0, 1.0, 0.0]), to([0.0, 0.0, 1.0]));
        // the viewer's frame: right = (ex.x, ey.x, ez.x), up = (ex.y, ey.y, ez.y)
        ([ex[0], ey[0], ez[0]], [ex[1], ey[1], ez[1]])
    }

    /// Whether the screen point is inside a closed convex quad.
    fn inside(q: &[Vec3; 5], x: f64, y: f64) -> bool {
        let mut sign = 0.0;
        for i in 0..4 {
            let (a, b) = (q[i], q[i + 1]);
            let cross = (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]);
            if cross.abs() < 1e-12 {
                continue;
            }
            if sign == 0.0 {
                sign = cross.signum();
            } else if cross.signum() != sign {
                return false;
            }
        }
        true
    }

    /// The row of kinds fits its room and its clusters are one, three,
    /// three and one blocks, in order; the cube and the pair fit theirs.
    #[test]
    fn the_row_the_cube_and_the_pair_fit_their_rooms() {
        let (a, total) = parts();
        let all = views(total, a);
        let inside_room = |view: &ViewAt, placed: &[Placed], centre: (f64, f64), room: (f64, f64)| {
            let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
            for p in placed {
                for m in 0..8usize {
                    let c = [0, 1, 2].map(|k| if m >> k & 1 == 1 { p.hi[k] } else { p.lo[k] });
                    let s = view.project(c);
                    lo = [lo[0].min(s[0]), lo[1].min(s[1])];
                    hi = [hi[0].max(s[0]), hi[1].max(s[1])];
                }
            }
            assert!(
                hi[0] - lo[0] <= room.0 + 1e-6 && hi[1] - lo[1] <= room.1 + 1e-6,
                "{lo:?} {hi:?}"
            );
            assert!(
                (lo[0] + hi[0]) / 2.0 - centre.0 <= room.0 / 2.0 && (lo[1] + hi[1]) / 2.0 - centre.1 <= room.1 / 2.0
            );
            assert!(
                lo[0] > -FRAME_X_RADIUS + 0.15 && hi[0] < FRAME_X_RADIUS - 0.15,
                "{lo:?} {hi:?}"
            );
            assert!(
                lo[1] > -FRAME_Y_RADIUS + 0.15 && hi[1] < FRAME_Y_RADIUS - 0.15,
                "{lo:?} {hi:?}"
            );
        };
        let row = Vals {
            a,
            drift: 0.0,
            row: 1.0,
            pair: 0.0,
            kinds: 1.0,
        };
        let placed: Vec<Placed> = layout(total, row).into_iter().filter(|p| p.set == 0).collect();
        inside_room(&all[1], &placed, CENTRE_ROW, ROOM_ROW);
        // clusters in order: kinds 0, 1, 1, 1, 2, 2, 2, 3 left to right
        let mut by_x: Vec<&Placed> = placed.iter().collect();
        by_x.sort_by(|p, q| p.lo[0].total_cmp(&q.lo[0]));
        assert_eq!(
            by_x.iter().map(|p| p.short).collect::<Vec<_>>(),
            [0, 1, 1, 1, 2, 2, 2, 3]
        );
        // no two overlap along the row
        assert!(by_x.windows(2).all(|w| w[0].hi[0] < w[1].lo[0]));
        let cube = Vals {
            a,
            drift: 1.0,
            row: 0.0,
            pair: 0.0,
            kinds: 1.0,
        };
        let placed: Vec<Placed> = layout(total, cube).into_iter().filter(|p| p.set == 0).collect();
        inside_room(&all[0], &placed, CENTRE_CUBE, ROOM_CUBE);
        let pair = Vals {
            a,
            drift: 0.0,
            row: 0.0,
            pair: 1.0,
            kinds: 1.0,
        };
        inside_room(&all[2], &layout(total, pair), CENTRE_PAIR, ROOM_PAIR);
        // the view's ends are the fitted views
        assert_eq!(
            view_for(
                &all,
                Vals {
                    a,
                    drift: 0.0,
                    row: 0.0,
                    pair: 0.0,
                    kinds: 0.0
                }
            ),
            all[0]
        );
        assert_eq!(
            view_for(
                &all,
                Vals {
                    a,
                    drift: 0.0,
                    row: 1.0,
                    pair: 0.0,
                    kinds: 0.0
                }
            ),
            all[1]
        );
        assert_eq!(
            view_for(
                &all,
                Vals {
                    a,
                    drift: 0.0,
                    row: 0.0,
                    pair: 1.0,
                    kinds: 0.0
                }
            ),
            all[2]
        );
    }

    /// The kind colours are the storyboard's ramp, and a block's kind in the
    /// second copy is one more than in the first.
    #[test]
    fn the_kinds_wear_the_ramp_and_the_second_copy_is_one_short_more() {
        let want = [size(5), size(4), size(3), size(2), size(1)];
        for (k, w) in want.iter().enumerate() {
            assert_eq!(kind_colour(k), *w, "kind {k}");
        }
        let (a, total) = parts();
        let both = layout(
            total,
            Vals {
                a,
                drift: 0.0,
                row: 0.0,
                pair: 1.0,
                kinds: 1.0,
            },
        );
        let count = |set: usize, k: usize| both.iter().filter(|p| p.set == set && p.short == k).count();
        assert_eq!((0..4).map(|k| count(0, k)).collect::<Vec<_>>(), [1, 3, 3, 1]);
        assert_eq!((0..5).map(|k| count(1, k)).collect::<Vec<_>>(), [0, 1, 3, 3, 1]);
        // together: 1, 4, 6, 4, 1, the fourth power's kinds
        let together: Vec<usize> = (0..5).map(|k| count(0, k) + count(1, k)).collect();
        assert_eq!(together, [1, 4, 6, 4, 1]);
        assert_eq!(fact("F-B.fourth.kinds").unwrap(), "1,4,6,4,1");
        assert_eq!(fact("F-B.fourth.blocks").unwrap(), "16");
    }

    /// The picture never keeps the voice waiting: with the scratch voice
    /// placed, each line starts within two seconds of the end of the one
    /// before, except after the quiz, whose countdown and landed answer
    /// fill the gap (shown with `--nocapture`).
    #[test]
    fn no_line_waits_long_for_its_picture() {
        let v = testing::run_voiced("12_the_cut_cube").unwrap();
        let mut lines = v.lines.clone();
        lines.sort_by(|a, b| a.1.total_cmp(&b.1));
        let quiz = super::super::draw::quiz_named("kinds").unwrap();
        let held = f64::from(quiz.seconds) + super::super::draw::LAND_S;
        let mut prev: Option<(String, f64)> = None;
        for (id, at, len) in &lines {
            if let Some((before, end)) = &prev {
                let gap = at - end;
                eprintln!("{id}: starts {at:.1} after a gap of {gap:.1} (since {before})");
                let allowed = if before == quiz.line { held + 2.0 } else { 2.0 };
                assert!(gap <= allowed, "{id} starts {gap:.1} s after {before} ends");
            }
            prev = Some((id.clone(), at + len));
        }
        eprintln!("chapter {:.1} s", v.ran.seconds);
        assert!(prev.is_some_and(|(_, end)| end < v.ran.seconds));
    }

    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("12_the_cut_cube").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "3 long",
            "2 short",
            "none short",
            "four short",
            "(a+b)^{2} = a^{2} + 2ab + b^{2}",
            "(a+b)^{3} =",
            "? blocks, and ? of each kind",
            "8 blocks: 1, 3, 3, 1",
            "Pascal's triangle",
            "\u{d7} 3",
            "\u{d7} 2",
            "= 125",
            "16",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 40.0 && ran.seconds < 100.0, "{} s", ran.seconds);
    }
}
