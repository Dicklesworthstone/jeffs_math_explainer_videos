//! 09_boundary: Cauchy's integral formula as the payoff of loop sums, shown
//! as an average (storyboard 09_1–09_9). The bump's colour picture stays on
//! the left throughout.
//!
//! 09_1 earns the formula. f(z)/(z − a) splits into f(a)/(z − a), whose loop
//! integral 08_3 already gave, and the difference quotient g. g has a hole at a
//! and is never called holomorphic there: it tends to f′(a), so it is bounded
//! near a. A close-up of the disk around a then carries the proof. A small
//! circle around the hole; polygons in both circles, the ring between them cut
//! into triangles, each walked counterclockwise (08_5's triangle case: each
//! integral is 0); the inside edges cancel in visible pairs, leaving the outer
//! polygon counterclockwise and the inner one clockwise. With the small circle
//! held fixed the polygons gain sides and close in on the circles (an arc and
//! its chord share their ends, so only g's wobble times their length differs);
//! only then does the small circle shrink, its integral squeezed by 2πρ·M. No
//! Taylor series, residue or removable-singularity result is used.
//!
//! From 09_2 on: the samples on a circle appear twice, as dots on the circle
//! and as their values in a value plane on the right, where the average is
//! the visible centroid. On radius ½ the centroid sits on 1 (8 and 16
//! samples). On radius 2 the circle has swallowed both poles and the centroid
//! falls to 0. Turning each value back by its own angle (once, then twice)
//! moves the centroid to r·c₁ and r²·c₂, which reads the recipe's
//! coefficients off the boundary. The chapter ends on the opening puzzle
//! answered: the recipe at 0 works out to the poles, and recentred at 1 it
//! works on (−0.41, 2.41).

use std::f64::consts::{PI, TAU};

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, arrow_head_points, axis_names, card, carded, dot, fact, fact_f64, fade_in_all, fade_out_all,
    faded_line, flat, ground, group, hero, input, line_style, marks, names_now, output, pause, pole, precisely_now,
    predict, reading_window, say, tween_linear, value, wait_until, warn,
};
// the opening chapter's words for the same things
use super::the_bump::{POLE_I, POLE_MINUS_I, RADIUS, TITLE};
use crate::model::{self, C, Func, ZERO};

pub const QUOTIENT_TEX: &str = r"\frac{f(z)}{z-a}";
/// The two terms of the split's right side, each lit in its turn.
pub const SPLIT_TERMS: [&str; 2] = [r"\frac{f(a)}{z-a}", r"\frac{f(z)-f(a)}{z-a}"];
pub const BEYOND: &str = "a little beyond";
pub const BUMP_Z_TEX: &str = r"\mathrm{bump} = \frac{1}{1 + z^2}";
pub const R_NAMES: [&str; 2] = ["r = 1/2", "r = 2"];
pub const CENTRE_ZERO: &str = "a = 0";
/// Why 8 samples show as 4 dots, and 16 as 8: the bump takes one value at z and at −z.
pub const PAIR_UP: &str = "opposite samples share a value";
pub const THETA_TEX: &str = r"\theta = \mathrm{its}\;\mathrm{angle}";
pub const SLOPE: &str = "the slope at the centre";
pub const NEXT: &str = "the next coefficient";
pub const RECIPE_NAME: &str = "the recipe";
/// What the two check rows of 09_9 are compared with.
pub const MATCHES: &str = " = the bump";
pub const OUTSIDE: &str = " (outside: it blows up)";
pub const SPLIT_TEX: &str = r"\frac{f(z)}{z-a} = \frac{f(a)}{z-a} + \frac{f(z)-f(a)}{z-a}";
/// What 09_1 proves about the second piece. It is not called holomorphic: at a it is undefined.
pub const REST: &str = "the second piece: its loop integral is 0";
/// 09_1's first piece, from 08_3's 1/z on a circle centred at a.
pub const FIRST_TEX: &str = r"\oint \frac{f(a)}{z-a}\,dz = 2\pi i\, f(a)";
/// 09_1's second piece, named, with its hole.
pub const G_TEX: &str = r"g(z) = \frac{f(z)-f(a)}{z-a},\quad z \ne a";
pub const HOLE_NAME: &str = "g is undefined at a";
pub const LIMIT_TEX: &str = r"z \to a:\quad g(z) \to f'(a)";
/// Where g is bounded: inside the dashed circle about a (the hole itself left out).
pub const BOUND_TEX: &str = r"|g| \le M";
pub const INSIDE_DASHES: &str = "inside the dashes";
pub const RHO_TEX: &str = r"\rho";
pub const HOLOMORPHIC: [&str; 2] = ["g is holomorphic", "between the circles"];
pub const TRIANGLE_TEX: &str = r"\oint_{T} g\,dz = 0\quad \mathrm{around}\;\mathrm{each}\;\mathrm{triangle}\;T";
pub const CANCELS: [&str; 2] = ["inside edges cancel:", "walked both ways"];
/// The two polygons' names, outer and inner.
pub const POLYGON_NAMES: [&str; 2] = ["P", "p"];
pub const SURVIVE_TEX: &str = r"\oint_{P} g\,dz + \oint_{p\;\mathrm{clockwise}} g\,dz = 0";
pub const POLYGONS_TEX: &str = r"\oint_{P} g\,dz = \oint_{p} g\,dz";
pub const BOTH: [&str; 2] = ["both walked", "counterclockwise"];
pub const FIXED: &str = "small circle fixed";
pub const SAME_ENDS: &str = "same endpoints";
pub const CONSTANT_TEX: &str = r"\int_{\mathrm{arc}} c\,dz = c(B-A) = \int_{\mathrm{chord}} c\,dz";
pub const ARC_TEX: &str = r"\Big|\int_{\mathrm{arc}} g\,dz - \int_{\mathrm{chord}} g\,dz\,\Big| \le w \cdot L";
pub const WOBBLE: [&str; 2] = ["w: how much g varies", "L: arc + chord lengths"];
pub const SUM_TEX: &str = r"\Big|\oint_{\mathrm{circle}} g\,dz - \oint_{\mathrm{polygon}} g\,dz\,\Big| \le \max w \cdot \mathrm{total}\;L \to 0";
pub const CIRCLES_TEX: &str = r"\oint_{|z-a|=r} g\,dz = \oint_{|z-a|=\rho} g\,dz";
pub const EVERY_TEX: &str = r"\mathrm{for}\;\mathrm{every}\;\mathrm{small}\;\rho";
pub const SHRINK_TEX: &str = r"\Big|\oint_{|z-a|=\rho} g\,dz\,\Big| \le 2\pi\rho \cdot M \to 0";
pub const BAR_TEX: &str = r"2\pi\rho \cdot M";
pub const ZERO_TEX: &str = r"\oint_{|z-a|=r} g\,dz = 0";
pub const RESULT_TEX: &str = r"\oint \frac{f(z)}{z-a}\,dz = 2\pi i\, f(a)";
pub const FORMULA_TEX: &str = r"f(a) = \frac{1}{2\pi i}\oint \frac{f(z)}{z-a}\,dz";
pub const ON_CIRCLE_TEX: &str = r"z = a + r e^{i\theta}:\quad dz = i\,(z-a)\,d\theta";
pub const AVERAGE_TEX: &str = r"f(a) = \frac{1}{2\pi}\int_0^{2\pi} f(a + r e^{i\theta})\,d\theta";
pub const AVERAGE: &str = "the value at the centre = the average on the circle";
pub const VALUES: &str = "the samples' values";
pub const AVG_OF: &str = "average of ";
pub const COLON: &str = ": ";
pub const ASK: [&str; 1] = ["average = 1?"];
pub const LIMIT: &str = "limit: ";
pub const SHARES: &str = "bump at 0, then each pole: ";
pub const NEEDS: &str =
    "Cauchy's formula holds if f is holomorphic on an open set containing the closed disk: on the circle and inside it";
pub const COEF_TEX: &str =
    r"c_n = \frac{1}{r^n}\cdot \mathrm{average}\;\mathrm{of}\; f(a + r e^{i\theta})\, e^{-i n \theta}";
pub const TURNED: [&str; 2] = ["each turned back by its angle", "each turned back by twice its angle"];
pub const READ: [&str; 2] = [
    r"c_1 = \frac{\mathrm{average}}{r} \approx ",
    r"c_2 = \frac{\mathrm{average}}{r^2} \approx ",
];
pub const ASK_RECENTRE: [&str; 2] = ["recipe centred at 1:", "works from ? to ?"];
pub const WORKS: &str = "works on ";
pub const CHECK: &str = " terms at ";

/// "average of 8: 1.0039": how many samples, then what they average to.
fn average_says(samples: &str, mean: &str) -> String {
    format!("{AVG_OF}{samples}{COLON}{mean}")
}

/// The bump's picture, [−2.5, 2.5]², on the left.
pub fn picture() -> Plane {
    // its foot stops above where a two-line lower third starts
    Plane::new(v(-3.3, -0.2), 5.6, (-2.5, 2.5), (-2.5, 2.5))
}

/// Where the samples' values are drawn (09_3–09_5).
pub fn values_plane() -> Plane {
    Plane::new(v(4.0, 0.3), 5.0, (-0.55, 1.55), (-0.55, 0.55))
}

/// Where the turned values are drawn (09_6).
pub fn turned_plane() -> Plane {
    Plane::new(v(4.0, 0.15), 3.4, (-1.5, 1.5), (-1.5, 1.5))
}

/// 09_1's close-up of the disk around a, on the right: the plane near a, magnified (a is its
/// centre, so its coordinates are z − a). The loop is the hero circle of radius ½.
pub fn close_up() -> Plane {
    Plane::new(v(4.4, -0.95), 4.0, (-0.9, 0.9), (-0.9, 0.9))
}

/// 09_1's radii in the close-up: the loop; the dashed circle inside which |g| ≤ M is drawn; the
/// small circle while it is held fixed; where its shrinking pauses for the bound; where it stops.
pub const LOOP_R: f64 = 0.5;
pub const BOUND_R: f64 = 0.3;
pub const SMALL_R: f64 = 0.18;
pub const MID_R: f64 = 0.12;
/// Final review 995, L14: at 0.045 the shrunk circle's line (width 4) met the hole's ring
/// (width 3). Stopping here leaves a gap of 7.6 native pixels between the two lines.
pub const SHRUNK_R: f64 = 0.06;
/// The hole at a: a ring this wide (scene units), never crossed by the small circle.
pub const HOLE: f64 = 0.07;
/// The polygons as tiled into triangles, and the most sides they gain (both powers of two:
/// the sides double on the way).
pub const SIDES: usize = 1 << 3;
pub const FINEST: usize = 1 << 6;
/// The piece of the finest outer polygon whose arc and chord the inset magnifies.
pub const PIECE: usize = 24;
/// The spoke whose two walks are lit when the voice says they go in opposite directions.
pub const LIT_SPOKE: usize = 2;
/// How far each triangle's walk is drawn inside it: its corners pulled toward its centre.
pub const INSET: f64 = 0.7;
/// Labels in the close-up: a, ρ, the polygons P and p. P stands outside the outer polygon's
/// corner at 45°; p just outside the inner polygon's corner at 180°, between the small circle and
/// the dashes (beyond the outer polygon's corner at 225°, where it stood before, it read as a
/// second name for the outer polygon).
pub const A_AT: C = C::new(0.0, 0.09);
pub const RHO_AT: C = C::new(0.62, -0.25);
pub const POLYGON_NAMES_AT: [C; 2] = [C::new(0.445, 0.445), C::new(-0.245, 0.0)];
/// The heads on the polygons' sides (scene units). The inner ones are larger and drawn in
/// `inner_head_style`: green heads on the green polygon and circle read as notches.
pub const OUTER_HEAD: f64 = 0.12;
pub const INNER_HEAD: f64 = 0.16;
/// The width of the background rim round each inner head.
pub const INNER_HEAD_RIM: f64 = 2.5;
/// While the small circle shrinks, a's and ρ's names ride just outside its line, a above it and ρ
/// to its right. Held still, a name is crossed, since the circle passes every radius from SMALL_R
/// down to SHRUNK_R; ρ also jumped from RHO_AT to the updater's place. Each stands this far (scene
/// units) beyond the line's centre: half the line (width 4), a 0.045 gap, then half its ink on that
/// side (a 25 native pixels high, ρ 30 wide, at 270 per unit). At SHRUNK_R, a is 12 native pixels
/// above A_AT.
pub const A_RIDE: f64 = 0.02 + 0.045 + 12.5 / 270.0;
pub const RHO_RIDE: f64 = 0.02 + 0.045 + 15.0 / 270.0;
/// a's name on the picture, from its centre (09_1 on; 09_8's slide carries it with the centre).
pub const A_NAME_FROM_CENTRE: C = C::new(0.18, -0.2);
/// 09_7's disk, centred on 0, and 09_9's, centred on 1, each grow from radius 0.001 (out to 1 and
/// to √2) through a's name, which stands at the same place from each centre. The name keeps its
/// place and makes way: hidden while the disk's radius (plane units, its tracker's value) is in this
/// range, where its line (width 5) comes within 0.035 of the name's ink (35 x 38 native pixels at
/// 302 per plane unit, 0.18 to 0.35 from the centre), and fully back `A_NAME_EASE` either side.
pub const A_NAME_HIDDEN: [f64; 2] = [0.11, 0.43];
pub const A_NAME_EASE: f64 = 0.1;

/// Where 09_1's statements stand above the close-up: the main one, then its support.
pub const ROW_A: (f64, f64) = (4.3, 2.3);
pub const ROW_B: (f64, f64) = (4.3, 1.5);
/// The column between the picture and the close-up: its centre, and three heights.
pub const GAP_X: f64 = 1.15;
pub const GAP_TOP: f64 = 0.75;
pub const GAP_MID: f64 = -0.6;
pub const GAP_LOW: f64 = -2.0;
/// The arc-and-chord inset's card, in that column.
pub const BOX: (f64, f64) = (2.2, 1.7);
/// The 2πρ·M bar at the small circle's fixed radius (scene units).
pub const BAR_FULL: f64 = 2.0;

fn at((x, y): (f64, f64)) -> Vec3 {
    v(x, y)
}

/// The `n` corners of the regular polygon inscribed in |z| = r (z measured from a), the first
/// on the positive real axis.
pub fn polygon(r: f64, n: usize) -> Vec<C> {
    model::circle(ZERO, r, n)
}

/// The sides at doubling `m`: 2^⌊m⌋, from `SIDES` to `FINEST`.
pub fn sides_at(m: f64) -> usize {
    let k = m.floor().clamp(f64::from(SIDES.ilog2()), f64::from(FINEST.ilog2()));
    1 << (k as u32)
}

/// The ring between the polygons of `n` sides inscribed in |z| = `outer` and |z| = `inner`, cut
/// into triangles, each listed counterclockwise: the quadrilateral between neighbouring spokes
/// split along one diagonal. The hole inside the inner polygon is left empty.
pub fn ring_triangles(outer: f64, inner: f64, n: usize) -> Vec<[C; 3]> {
    let (o, i) = (polygon(outer, n), polygon(inner, n));
    let mut out = Vec::with_capacity(2 * n);
    for k in 0..n {
        let next = (k + 1) % n;
        out.push([i[k], o[k], o[next]]);
        out.push([i[k], o[next], i[next]]);
    }
    out
}

/// One edge of one triangle, walked the triangle's way round.
#[derive(Clone, Copy, Debug)]
pub struct Walk {
    /// The edge, from where it is walked to where.
    pub edge: [C; 2],
    /// The same walk as drawn: just inside its own triangle, so a shared edge's two walks show
    /// side by side.
    pub drawn: [C; 2],
    /// A spoke or a diagonal, shared with the neighbouring triangle; not a side of either polygon.
    pub inside: bool,
}

/// Every triangle's three walks, in the triangles' order.
pub fn ring_walks(triangles: &[[C; 3]]) -> Vec<Walk> {
    let mut out = Vec::with_capacity(3 * triangles.len());
    for t in triangles {
        let c = (t[0] + t[1] + t[2]).scale(1.0 / 3.0);
        let pull = |z: C| c + (z - c).scale(INSET);
        for j in 0..3 {
            let (a, b) = (t[j], t[(j + 1) % 3]);
            out.push(Walk {
                edge: [a, b],
                drawn: [pull(a), pull(b)],
                // a side of a polygon joins two corners of one radius; the others join the two
                inside: (a.abs() - b.abs()).abs() > 1e-9,
            });
        }
    }
    out
}

/// A solid arrowhead `size` long on the middle of the segment from `a` to `b`, pointing on to `b`.
pub fn mid_head(a: Vec3, b: Vec3, size: f64) -> Vec<Vec3> {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy).max(1e-9);
    let tip = v(
        (a[0] + b[0]) / 2.0 + dx / len * size / 2.0,
        (a[1] + b[1]) / 2.0 + dy / len * size / 2.0,
    );
    arrow_head_points(tip, a, size)
}

/// The circle |z| = r in `plane`, closed, as scene points.
pub fn round(plane: &Plane, r: f64) -> Vec<Vec3> {
    (0..=240)
        .map(|k| plane.p(C::polar(r, TAU * f64::from(k) / 240.0)))
        .collect()
}

/// The ring `inner` < |z| < `outer` as one outline: out along the outer circle, back the other
/// way along the inner one, the two joined by a cut of no width (so one flat fill shows a ring).
pub fn ring_outline(plane: &Plane, outer: f64, inner: f64) -> Vec<Vec3> {
    let mut pts = round(plane, outer);
    pts.extend(round(plane, inner).into_iter().rev());
    pts
}

/// The path of the point that closes in on the hole in 09_1, `s` from 0 to 1: a spiral from
/// near the loop to just outside the hole's ring, clear of a's name above it.
pub fn approach(s: f64) -> C {
    C::polar(0.42 + (0.07 - 0.42) * s, 0.6 + 2.5 * s)
}

/// a's and ρ's names in the close-up while the small circle has radius `r` (see [`A_RIDE`]).
pub fn shrink_names(r: f64) -> [C; 2] {
    let unit = close_up().unit();
    [C::new(0.0, r + A_RIDE / unit), C::new(r + RHO_RIDE / unit, 0.0)]
}

/// The ends of a's leader while the small circle has radius `r`. Riding the line, a's name is
/// always nearer the circle than the hole it names; this thin line ties it to the hole. It runs
/// on the vertical through a, from just outside the hole's ring up to just under a's ink, and
/// each end stops 0.02 short.
pub fn a_leader(r: f64) -> [C; 2] {
    let unit = close_up().unit();
    let foot = A_RIDE - 12.0 / 270.0;
    [
        C::new(0.0, (HOLE + 0.015 + 0.02) / unit),
        C::new(0.0, r + (foot - 0.02) / unit),
    ]
}

/// How much of a's name shows while a disk of radius `r` about its centre grows past it (see
/// [`A_NAME_HIDDEN`]).
pub fn a_name_shown(r: f64) -> f64 {
    let [lo, hi] = A_NAME_HIDDEN;
    ((lo - r).max(r - hi) / A_NAME_EASE).clamp(0.0, 1.0)
}

/// The two zoom lines of 09_1: from the "a little beyond" circle on the picture to the
/// close-up's left corners.
pub fn zoom_links(p: &Plane, cp: &Plane) -> [[Vec3; 2]; 2] {
    let f = cp.frame;
    [
        [p.p(C::polar(0.8, 0.7)), v(f.left(), f.top())],
        [p.p(C::polar(0.8, -0.7)), v(f.left(), f.bottom())],
    ]
}

/// The inset's arc (bulging up) and its chord's two ends, in scene units, centred in its card.
pub fn inset_arc() -> (Vec<Vec3>, [Vec3; 2]) {
    let (cx, cy, r, half) = (GAP_X, GAP_MID - 1.55, 1.85, 0.38);
    let point = |a: f64| v(cx + r * a.cos(), cy + r * a.sin());
    let arc = (0..=40)
        .map(|k| point(PI / 2.0 + half - 2.0 * half * f64::from(k) / 40.0))
        .collect();
    (arc, [point(PI / 2.0 + half), point(PI / 2.0 - half)])
}

/// The 2πρ·M bar for radius `rho`: from the column's left, as long as ρ is (to scale).
pub fn bar_points(rho: f64) -> Vec<Vec3> {
    let x0 = GAP_X - BAR_FULL / 2.0;
    vec![v(x0, GAP_MID - 0.1), v(x0 + BAR_FULL * rho / SMALL_R, GAP_MID - 0.1)]
}

/// The circle |z| = r in `plane`, a line of `color` at opacity `o`, closed, detached.
fn circle_in(stage: &mut Stage<'_>, plane: &Plane, r: f64, color: Srgb, width: f64, o: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(round(plane, r)).style(faded_line(color, width, o)).build())
}

/// The circle |z| = r in `plane`, dashed, as one group, detached.
fn dashed_circle(stage: &mut Stage<'_>, plane: &Plane, r: f64, color: Srgb) -> fmn::Result<Mob> {
    let n = 40;
    let mut dashes = Vec::with_capacity(n);
    for k in 0..n {
        let a0 = TAU * k as f64 / n as f64;
        let pts: Vec<Vec3> = (0..=6)
            .map(|j| plane.p(C::polar(r, a0 + TAU * 0.55 / n as f64 * f64::from(j) / 6.0)))
            .collect();
        dashes.push(
            stage
                .arena_mut()
                .add(polyline(pts).style(line_style(color, 3.0)).build()),
        );
    }
    group(stage, &dashes)
}

/// The closed polyline through `corners` in `plane`, detached.
fn outline(stage: &mut Stage<'_>, plane: &Plane, corners: &[C], color: Srgb, width: f64) -> Mob {
    let mut pts: Vec<Vec3> = corners.iter().map(|&z| plane.p(z)).collect();
    pts.push(pts[0]);
    stage
        .arena_mut()
        .add(polyline(pts).style(line_style(color, width)).build())
}

/// A head `size` long on the middle of every side of the polygon `corners`, pointing the way it
/// is walked, as scene points.
pub fn side_head_points(plane: &Plane, corners: &[C], counterclockwise: bool, size: f64) -> Vec<Vec<Vec3>> {
    let n = corners.len();
    (0..n)
        .map(|k| {
            let (a, b) = (plane.p(corners[k]), plane.p(corners[(k + 1) % n]));
            let (a, b) = if counterclockwise { (a, b) } else { (b, a) };
            mid_head(a, b, size)
        })
        .collect()
}

/// The inner polygon's heads: the yellow of the walks they stand for, each rimmed with the
/// background so that it is cut out of the green line it sits on.
pub fn inner_head_style() -> Style {
    Style::default().fill(hero(), 1.0).stroke(ground(), INNER_HEAD_RIM, 1.0)
}

/// Those heads in `style`, detached.
fn side_heads(
    stage: &mut Stage<'_>,
    plane: &Plane,
    corners: &[C],
    counterclockwise: bool,
    size: f64,
    style: Style,
) -> Vec<Mob> {
    side_head_points(plane, corners, counterclockwise, size)
        .into_iter()
        .map(|head| stage.arena_mut().add(Polygon::new(head).style(style).build()))
        .collect()
}

/// Four heads on the circle |z| = r, pointing counterclockwise.
fn circle_heads(stage: &mut Stage<'_>, plane: &Plane, r: f64, color: Srgb, size: f64) -> Vec<Mob> {
    let d = size / (2.0 * r * plane.unit());
    (0..4)
        .map(|k| {
            let th = PI / 4.0 + PI / 2.0 * f64::from(k);
            let head = arrow_head_points(plane.p(C::polar(r, th + d)), plane.p(C::polar(r, th - d)), size);
            stage
                .arena_mut()
                .add(Polygon::new(head).style(flat(color, 1.0)).build())
        })
        .collect()
}

/// Two lines of text, one under the other, centred at `at`, detached.
fn two_lines(
    kit: &Kit,
    stage: &mut Stage<'_>,
    lines: &[&str; 2],
    size: f64,
    color: Srgb,
    at: Vec3,
) -> fmn::Result<Vec<Mob>> {
    let pitch = size / 48.0 * 0.62 + 0.04;
    Ok(vec![
        kit.text(stage, lines[0], size, color, v(at[0], at[1] + pitch / 2.0))?,
        kit.text(stage, lines[1], size, color, v(at[0], at[1] - pitch / 2.0))?,
    ])
}

/// Keep each proof statement inside the right-hand column; actual glyph/pixel
/// fit is still checked by the orchestrator's native preview.
fn proof_tex(kit: &Kit, stage: &mut Stage<'_>, tex: &str, size: f64, color: Srgb, pos: (f64, f64)) -> fmn::Result<Mob> {
    let m = kit.tex(stage, tex, size, color, &[], at(pos))?;
    if stage.get_width(m) > 5.1 {
        stage.set_width(m, 5.1, false);
    }
    Ok(m)
}

/// Where 09_3's "a = 0" card stands on the picture, and where each share card of 09_5 stands
/// from its pole. The three are on screen together through 09_5.
const CENTRE_CARD_AT: C = C::new(1.05, -0.4);
const SHARE_CARD_FROM_POLE: C = C::new(0.72, 0.0);

/// The real graph of 09_7–09_9.
pub fn graph() -> Frame2 {
    Frame2 {
        center: v(3.9, -0.6),
        width: 5.6,
        height: 3.0,
        x: (-1.6, 3.0),
        y: (-0.5, 1.5),
    }
}

/// The bump at a complex point.
pub fn bump_at(z: C) -> C {
    Func::Bump.eval(z).unwrap_or(ZERO)
}

/// The `n` sample points on the circle of radius `r` about 0.
pub fn samples(r: f64, n: usize) -> Vec<C> {
    model::circle(ZERO, r, n)
}

/// The mean of some points.
pub fn centroid(ws: &[C]) -> C {
    ws.iter().fold(ZERO, |a, &b| a + b).scale(1.0 / ws.len().max(1) as f64)
}

/// The values on the circle of radius `r`, each turned back by `s` times its
/// own angle (`s` = 1 reads r·c₁, `s` = 2 reads r²·c₂).
pub fn turned_values(r: f64, n: usize, s: f64) -> Vec<C> {
    (0..n)
        .map(|k| {
            let th = TAU * k as f64 / n as f64;
            bump_at(C::polar(r, th)) * C::polar(1.0, -s * th)
        })
        .collect()
}

/// Steps across a graph's window: 0.005 each on [−1.6, 3], so that 0 and ±1 are on the grid
/// and the recipe's two spikes at ±1 draw alike.
pub const CURVE_STEPS: i32 = 920;

/// A function of x drawn in `g`, cut into pieces wherever it leaves the window.
pub fn clipped(g: &Frame2, f: impl Fn(f64) -> f64) -> Vec<Vec<Vec3>> {
    let n = CURVE_STEPS;
    let mut pieces: Vec<Vec<Vec3>> = Vec::new();
    let mut current: Vec<Vec3> = Vec::new();
    for k in 0..=n {
        let x = g.x.0 + (g.x.1 - g.x.0) * f64::from(k) / f64::from(n);
        let y = f(x);
        if y.is_finite() && y >= g.y.0 && y <= g.y.1 {
            current.push(g.p(x, y));
        } else if !current.is_empty() {
            pieces.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        pieces.push(current);
    }
    pieces.into_iter().filter(|p| p.len() >= 2).collect()
}

fn signed(s: &str) -> String {
    s.replace('-', "−")
}

pub struct Boundary {
    pub kit: Kit,
}

impl SceneConstruct for Boundary {
    fn name(&self) -> &str {
        "09_boundary"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = picture();
        let right = 4.0;

        // 09_1: divide by z − a; split; the rest integrates to zero
        let line = say(&kit, stage, "09_1")?;
        let painter = Painter::build(stage, Some(Func::Bump), p, 1.0)?;
        let mut p_frame = axis_names(&kit, stage, &p)?;
        p_frame.push(p.border(stage, Palette::faint())?);
        painter.show(stage, 1.0)?;
        let poles = marks(stage, Func::Bump, &p);
        fade_in_all(stage, &p_frame, 0.3)?;
        fade_in_all(stage, &poles, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let centre = dot(stage, p.p(ZERO), 0.08, Palette::ink());
        let a_name = kit.tex(stage, "a", 28.0, Palette::ink(), &[], p.p(A_NAME_FROM_CENTRE))?;
        let half = p.circle(stage, ZERO, 0.5, hero(), 5.0)?;
        let beyond = p.circle(stage, ZERO, 0.8, Palette::faint(), 2.0)?;
        let beyond_name = carded(
            &kit,
            stage,
            BEYOND,
            18.0,
            Palette::ink(),
            p.p(C::new(1.7, -0.3)),
            Palette::faint(),
        )?;
        fade_in_all(
            stage,
            &[centre, a_name, half, beyond, beyond_name[0], beyond_name[1]],
            0.5,
        )?;
        // The approved 09_1 is a complete proof, not a promise of one. Every
        // reveal below is tied to its words; measured phrase marks take priority.
        line.wait_for(stage, "divide it by")?;
        let quotient = kit.tex(stage, QUOTIENT_TEX, 34.0, Palette::ink(), &[], v(2.0, 3.2))?;
        fade_in_all(stage, &[quotient], 0.3)?;
        line.wait_for(stage, "Split that quotient")?;
        let split = kit.tex(
            stage,
            SPLIT_TEX,
            34.0,
            Palette::ink(),
            &[(SPLIT_TERMS[0], hero())],
            v(2.0, 3.2),
        )?;
        fade_out_all(stage, &[quotient], 0.2)?;
        fade_in_all(stage, &[split], 0.3)?;
        line.wait_for(stage, "The first is")?;
        let first = proof_tex(&kit, stage, FIRST_TEX, 28.0, hero(), ROW_A)?;
        fade_in_all(stage, &[first], 0.3)?;
        line.wait_for(stage, "The second is")?;
        let split_rest = kit.tex(
            stage,
            SPLIT_TEX,
            34.0,
            Palette::ink(),
            &[(SPLIT_TERMS[1], output())],
            v(2.0, 3.2),
        )?;
        let g_name = proof_tex(&kit, stage, G_TEX, 27.0, output(), ROW_A)?;
        fade_out_all(stage, &[split, first], 0.2)?;
        fade_in_all(stage, &[split_rest, g_name], 0.3)?;

        // The empty centre is a domain exclusion, not a removable extension.
        line.wait_for(stage, "It is undefined at a itself")?;
        let cp = close_up();
        let outer = circle_in(stage, &cp, LOOP_R, hero(), 4.0, 1.0);
        let hole = stage.arena_mut().add(
            Circle::new()
                .radius(HOLE)
                .style(line_style(warn(), 3.0))
                .build()
                .moved_to(cp.p(ZERO)),
        );
        let ca = kit.tex(stage, "a", 20.0, Palette::ink(), &[], cp.p(A_AT))?;
        let hole_name = kit.text(stage, HOLE_NAME, 23.0, warn(), at(ROW_B))?;
        let mut links = Vec::new();
        for ends in zoom_links(&p, &cp) {
            links.push(
                stage.arena_mut().add(
                    polyline(ends.to_vec())
                        .style(faded_line(Palette::dim(), 1.5, 0.45))
                        .build(),
                ),
            );
        }
        fade_in_all(stage, &links, 0.3)?;
        fade_in_all(stage, &[outer, hole, ca, hole_name], 0.3)?;
        line.wait_for(stage, "But as z approaches a")?;
        let limit = proof_tex(&kit, stage, LIMIT_TEX, 26.0, output(), ROW_B)?;
        // The zoom lines have tied the close-up to the picture by now. Held longer they would
        // run through the column's |g| <= M and cancellation lines, so they go with the hole's
        // name, in the same 0.2 s fade: no time is added.
        let mut named = vec![hole_name];
        named.extend(&links);
        fade_out_all(stage, &named, 0.2)?;
        fade_in_all(stage, &[limit], 0.3)?;
        let z = dot(stage, cp.p(approach(0.0)), 0.045, output());
        fade_in_all(stage, &[z], 0.2)?;
        let travel = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                a.move_to(z, cp.p(approach(value(a, travel))), ORIGIN);
            },
            true,
        )?;
        let rt = line.until(stage, "so near a", 0.3)?.min(2.5);
        stage.play(tween_linear(travel, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_for(stage, "so near a")?;
        let dashes = dashed_circle(stage, &cp, BOUND_R, Palette::dim())?;
        let bound = kit.tex(stage, BOUND_TEX, 25.0, output(), &[], v(GAP_X, GAP_TOP))?;
        let inside = kit.text(stage, INSIDE_DASHES, 17.0, Palette::dim(), v(GAP_X, GAP_TOP - 0.4))?;
        fade_in_all(stage, &[dashes, bound, inside], 0.3)?;
        fade_out_all(stage, &[z], 0.2)?;
        line.wait_for(stage, "Draw a small circle")?;
        let small = circle_in(stage, &cp, SMALL_R, output(), 4.0, 1.0);
        let rho = kit.tex(stage, RHO_TEX, 22.0, output(), &[], cp.p(RHO_AT))?;
        fade_in_all(stage, &[small, rho], 0.3)?;
        line.wait_for(stage, "Between the two circles")?;
        fade_out_all(stage, &[limit], 0.2)?;
        let holo = two_lines(&kit, stage, &HOLOMORPHIC, 21.0, Palette::ink(), at(ROW_B))?;
        fade_in_all(stage, &holo, 0.3)?;
        let annulus = stage.arena_mut().add(
            Polygon::new(ring_outline(&cp, LOOP_R, SMALL_R))
                .style(flat(input(), 0.09))
                .build(),
        );
        fade_in_all(stage, &[annulus], 0.3)?;

        // Native filled triangles leave the inner polygon entirely empty.
        line.wait_for(stage, "Trace each circle with a polygon")?;
        fade_out_all(stage, &[annulus], 0.2)?;
        let mut outer_poly = outline(stage, &cp, &polygon(LOOP_R, SIDES), hero(), 4.0);
        let mut inner_poly = outline(stage, &cp, &polygon(SMALL_R, SIDES), output(), 4.0);
        let triangles = ring_triangles(LOOP_R, SMALL_R, SIDES);
        let mut tiles = Vec::new();
        for (k, triangle) in triangles.iter().enumerate() {
            let color = if k % 2 == 0 { input() } else { output() };
            tiles.push(
                stage.arena_mut().add(
                    Polygon::new(triangle.iter().map(|&z| cp.p(z)).collect::<Vec<_>>())
                        .style(Style::default().fill(color, 0.13).stroke(Palette::dim(), 1.5, 0.65))
                        .build(),
                ),
            );
        }
        fade_in_all(stage, &[outer_poly, inner_poly], 0.3)?;
        fade_in_all(stage, &tiles, 0.6)?;
        line.wait_for(stage, "We saw that the shrinking argument works for triangles")?;
        fade_out_all(stage, &holo, 0.2)?;
        let triangle_rule = proof_tex(&kit, stage, TRIANGLE_TEX, 23.0, Palette::ink(), ROW_B)?;
        fade_in_all(stage, &[triangle_rule], 0.3)?;
        let walks = ring_walks(&triangles);
        let mut internal_walks = Vec::new();
        let mut boundary_walks = Vec::new();
        for walk in &walks {
            let ends = walk.drawn.map(|z| cp.p(z));
            let color = if walk.inside { Palette::ink() } else { hero() };
            let edge = stage
                .arena_mut()
                .add(polyline(ends.to_vec()).style(faded_line(color, 2.0, 0.85)).build());
            let head = stage.arena_mut().add(
                Polygon::new(mid_head(ends[0], ends[1], 0.065))
                    .style(flat(color, 1.0))
                    .build(),
            );
            if walk.inside {
                internal_walks.extend([edge, head]);
            } else {
                boundary_walks.extend([edge, head]);
            }
        }
        fade_in_all(stage, &boundary_walks, 0.4)?;
        fade_in_all(stage, &internal_walks, 0.4)?;
        line.wait_for(stage, "Now add up all the triangles")?;
        let cancellation = two_lines(&kit, stage, &CANCELS, 19.0, Palette::ink(), v(GAP_X, GAP_LOW))?;
        fade_in_all(stage, &cancellation, 0.3)?;
        line.wait_for(stage, "Every inside edge is walked twice")?;
        // One actual shared spoke is singled out, with opposite arrows offset
        // into its two incident triangles. Then all shared walks disappear.
        let spoke = polygon(LOOP_R, SIDES)[LIT_SPOKE];
        let small_spoke = polygon(SMALL_R, SIDES)[LIT_SPOKE];
        let mut pair = Vec::new();
        for walk in &walks {
            if ((walk.edge[0] - spoke).abs() < 1e-9 && (walk.edge[1] - small_spoke).abs() < 1e-9)
                || ((walk.edge[1] - spoke).abs() < 1e-9 && (walk.edge[0] - small_spoke).abs() < 1e-9)
            {
                let e = walk.drawn.map(|z| cp.p(z));
                pair.push(
                    stage
                        .arena_mut()
                        .add(polyline(e.to_vec()).style(line_style(warn(), 5.0)).build()),
                );
                pair.push(
                    stage.arena_mut().add(
                        Polygon::new(mid_head(e[0], e[1], 0.12))
                            .style(flat(warn(), 1.0))
                            .build(),
                    ),
                );
            }
        }
        fade_in_all(stage, &pair, 0.3)?;
        line.wait_for(stage, "so those parts cancel")?;
        fade_out_all(stage, &pair, 0.35)?;
        fade_out_all(stage, &internal_walks, 0.5)?;
        fade_out_all(stage, &tiles, 0.3)?;
        line.wait_for(stage, "What survives is the outer polygon")?;
        fade_out_all(stage, &boundary_walks, 0.2)?;
        fade_out_all(stage, &[triangle_rule], 0.2)?;
        let survived = proof_tex(&kit, stage, SURVIVE_TEX, 25.0, Palette::ink(), ROW_B)?;
        let outer_arrows = side_heads(stage, &cp, &polygon(LOOP_R, SIDES), true, OUTER_HEAD, flat(hero(), 1.0));
        let inner_cw = side_heads(
            stage,
            &cp,
            &polygon(SMALL_R, SIDES),
            false,
            INNER_HEAD,
            inner_head_style(),
        );
        let mut polygon_names = Vec::new();
        for (name, position) in POLYGON_NAMES.into_iter().zip(POLYGON_NAMES_AT) {
            polygon_names.push(kit.tex(stage, name, 20.0, Palette::ink(), &[], cp.p(position))?);
        }
        fade_in_all(stage, &[survived], 0.3)?;
        fade_in_all(stage, &outer_arrows, 0.3)?;
        fade_in_all(stage, &inner_cw, 0.3)?;
        fade_in_all(stage, &polygon_names, 0.3)?;
        line.wait_for(stage, "So the two polygons, both walked counterclockwise")?;
        let polygons_equal = proof_tex(&kit, stage, POLYGONS_TEX, 28.0, Palette::ink(), ROW_B)?;
        let inner_ccw = side_heads(
            stage,
            &cp,
            &polygon(SMALL_R, SIDES),
            true,
            INNER_HEAD,
            inner_head_style(),
        );
        fade_out_all(stage, &[survived], 0.2)?;
        fade_out_all(stage, &inner_cw, 0.2)?;
        fade_out_all(stage, &cancellation, 0.2)?;
        fade_in_all(stage, &[polygons_equal], 0.3)?;
        fade_in_all(stage, &inner_ccw, 0.3)?;
        let both = two_lines(&kit, stage, &BOTH, 18.0, Palette::ink(), v(GAP_X, GAP_LOW))?;
        fade_in_all(stage, &both, 0.3)?;

        // First limit: n increases. Both radii stay exactly fixed throughout.
        line.wait_for(stage, "Now keep the small circle fixed")?;
        let fixed = kit.text(stage, FIXED, 19.0, output(), v(4.4, -2.6))?;
        fade_in_all(stage, &[fixed], 0.3)?;
        fade_out_all(stage, &outer_arrows, 0.2)?;
        fade_out_all(stage, &inner_ccw, 0.2)?;
        fade_out_all(stage, &polygon_names, 0.2)?;
        fade_out_all(stage, &both, 0.2)?;
        line.wait_for(stage, "give the polygons more and more sides")?;
        for power in SIDES.ilog2() + 1..=FINEST.ilog2() {
            let n = sides_at(f64::from(power));
            let next_outer = outline(stage, &cp, &polygon(LOOP_R, n), hero(), 4.0);
            let next_inner = outline(stage, &cp, &polygon(SMALL_R, n), output(), 4.0);
            fade_out_all(stage, &[outer_poly, inner_poly], 0.15)?;
            fade_in_all(stage, &[next_outer, next_inner], 0.25)?;
            outer_poly = next_outer;
            inner_poly = next_inner;
            pause(stage, 0.25)?;
        }
        line.wait_for(stage, "Each short chord and its arc share their endpoints")?;
        let inset_back = card(stage, BOX.0, BOX.1, v(GAP_X, GAP_MID), Palette::dim())?;
        let (arc_pts, ends) = inset_arc();
        let arc = stage
            .arena_mut()
            .add(polyline(arc_pts).style(line_style(hero(), 4.0)).build());
        let chord = stage
            .arena_mut()
            .add(polyline(ends.to_vec()).style(line_style(output(), 4.0)).build());
        let endpoint_dots = [
            dot(stage, ends[0], 0.055, Palette::ink()),
            dot(stage, ends[1], 0.055, Palette::ink()),
        ];
        let same_ends = kit.text(stage, SAME_ENDS, 17.0, Palette::ink(), v(GAP_X, GAP_MID - 0.55))?;
        let mut inset = vec![inset_back, arc, chord, endpoint_dots[0], endpoint_dots[1], same_ends];
        fade_in_all(stage, &inset, 0.3)?;
        // Identify the magnified piece on the actual outer polygon.
        let theta = TAU * PIECE as f64 / FINEST as f64;
        let piece = cp.path(
            stage,
            &[C::polar(LOOP_R, theta), C::polar(LOOP_R, theta + TAU / FINEST as f64)],
            warn(),
            7.0,
        )?;
        fade_in_all(stage, &[piece], 0.3)?;
        line.wait_for(stage, "so a constant has the same integral along both")?;
        let constant = proof_tex(&kit, stage, CONSTANT_TEX, 23.0, Palette::ink(), ROW_B)?;
        fade_out_all(stage, &[polygons_equal], 0.2)?;
        fade_in_all(stage, &[constant], 0.3)?;
        line.wait_for(stage, "The difference comes only from how much the quotient wobbles")?;
        let arc_bound = proof_tex(&kit, stage, ARC_TEX, 24.0, Palette::ink(), ROW_B)?;
        fade_out_all(stage, &[constant], 0.2)?;
        fade_in_all(stage, &[arc_bound], 0.3)?;
        let wobble = two_lines(&kit, stage, &WOBBLE, 16.0, Palette::ink(), v(GAP_X, GAP_LOW))?;
        fade_in_all(stage, &wobble, 0.3)?;
        line.wait_for(stage, "Continuity makes every wobble small at once")?;
        let summed = proof_tex(&kit, stage, SUM_TEX, 23.0, Palette::ink(), ROW_B)?;
        fade_out_all(stage, &[arc_bound], 0.2)?;
        fade_in_all(stage, &[summed], 0.3)?;
        line.wait_for(
            stage,
            "So in the limit, the big circle's integral equals the small circle's",
        )?;
        let circles_equal = proof_tex(&kit, stage, CIRCLES_TEX, 27.0, Palette::ink(), ROW_B)?;
        fade_out_all(stage, &[summed, outer_poly, inner_poly, piece], 0.3)?;
        inset.extend(wobble);
        fade_out_all(stage, &inset, 0.3)?;
        fade_in_all(stage, &[circles_equal], 0.3)?;
        let circle_arrows = circle_heads(stage, &cp, LOOP_R, hero(), 0.14);
        let small_arrows = circle_heads(stage, &cp, SMALL_R, output(), 0.10);
        fade_in_all(stage, &circle_arrows, 0.3)?;
        fade_in_all(stage, &small_arrows, 0.3)?;
        line.wait_for(stage, "That holds for every small radius")?;
        let every = kit.tex(stage, EVERY_TEX, 20.0, Palette::ink(), &[], v(GAP_X, GAP_LOW))?;
        stage.set_width(every, 2.1, false);
        fade_in_all(stage, &[every], 0.3)?;

        // Second limit: now, and only now, rho decreases. M and outer stay fixed.
        line.wait_for(stage, "Only now do we shrink the small circle")?;
        fade_out_all(stage, &[fixed, every], 0.2)?;
        // a's and ρ's names leave with the small circle's heads and come back with the bar,
        // beside the circle's line where the shrink first puts them. Both go in fades already
        // here, so no time is added.
        let [a_from, rho_from] = shrink_names(SMALL_R);
        let ca_out = kit.tex(stage, "a", 20.0, Palette::ink(), &[], cp.p(a_from))?;
        let rho_out = kit.tex(stage, RHO_TEX, 22.0, output(), &[], cp.p(rho_from))?;
        let [low, high] = a_leader(SMALL_R);
        let leader = stage.arena_mut().add(
            polyline(vec![cp.p(low), cp.p(high)])
                .style(faded_line(Palette::ink(), 1.5, 0.6))
                .build(),
        );
        let mut leaving = small_arrows;
        leaving.extend([ca, rho]);
        fade_out_all(stage, &leaving, 0.2)?;
        let radius = stage.add_value_tracker(SMALL_R);
        let bar = stage
            .arena_mut()
            .add(polyline(bar_points(SMALL_R)).style(line_style(output(), 12.0)).build());
        let bar_name = kit.tex(stage, BAR_TEX, 24.0, output(), &[], v(GAP_X, GAP_MID + 0.35))?;
        fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, radius);
                let _ = a.set_points(small, polyline(round(&cp, r)).build().points());
                let _ = a.set_points(bar, polyline(bar_points(r)).build().points());
                let [a_at, rho_at] = shrink_names(r);
                a.move_to(ca_out, cp.p(a_at), ORIGIN);
                a.move_to(rho_out, cp.p(rho_at), ORIGIN);
                let [low, high] = a_leader(r);
                let _ = a.set_points(leader, polyline(vec![cp.p(low), cp.p(high)]).build().points());
            },
            true,
        )?;
        stage.play(tween_linear(radius, MID_R, 0.7)?)?;
        line.wait_for(stage, "The magnitude of its integral is at most")?;
        let shrink_bound = proof_tex(&kit, stage, SHRINK_TEX, 26.0, output(), ROW_B)?;
        fade_out_all(stage, &[circles_equal], 0.2)?;
        fade_in_all(stage, &[shrink_bound], 0.3)?;
        line.wait_for(stage, "and that goes to zero")?;
        stage.play(tween_linear(radius, SHRUNK_R, 1.0)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        // A finite picture never claims rho reached zero or filled the hole.
        line.wait_for(stage, "But the big circle's integral never changed")?;
        let zero = proof_tex(&kit, stage, ZERO_TEX, 28.0, hero(), ROW_B)?;
        fade_out_all(stage, &[shrink_bound], 0.2)?;
        fade_in_all(stage, &[zero], 0.3)?;
        line.wait_for(stage, "The second piece contributes nothing")?;
        let mut proof_gone = vec![
            outer, small, hole, ca_out, rho_out, leader, g_name, zero, dashes, bound, inside, bar, bar_name,
        ];
        proof_gone.extend(circle_arrows);
        fade_out_all(stage, &proof_gone, 0.4)?;
        let rest = kit.text(stage, REST, 20.0, output(), v(right, 2.2))?;
        let result = kit.tex(stage, RESULT_TEX, 36.0, hero(), &[], v(right, 1.1))?;
        fade_in_all(stage, &[rest, result], 0.4)?;
        wait_until(stage, line.end())?;

        // 09_2: on a circle the formula is an average
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_2")?;
        fade_out_all(stage, &[split_rest, rest, beyond, beyond_name[0], beyond_name[1]], 0.3)?;
        let formula = kit.tex(stage, FORMULA_TEX, 34.0, Palette::ink(), &[], v(2.0, 3.2))?;
        let on_circle = kit.tex(stage, ON_CIRCLE_TEX, 28.0, Palette::dim(), &[], v(right, 0.0))?;
        fade_in_all(stage, &[formula, on_circle], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let average_tex = kit.tex(stage, AVERAGE_TEX, 32.0, hero(), &[], v(right, -1.1))?;
        let average = kit.text(stage, AVERAGE, 22.0, Palette::ink(), v(right, -2.0))?;
        fade_in_all(stage, &[average_tex, average], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["Cauchy's integral formula"])?;

        // 09_3: the bump on radius ½: 8 samples, then 16
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_3")?;
        fade_out_all(stage, &[result, on_circle, average_tex, average, search], 0.4)?;
        let vp = values_plane();
        let mut vp_frame = vec![vp.border(stage, output())?];
        vp_frame.extend(vp.axes(stage)?);
        vp_frame.push(kit.text(
            stage,
            VALUES,
            22.0,
            output(),
            v(vp.frame.center[0], vp.frame.top() + 0.25),
        )?);
        let peak = fact_f64("bump.peak")?;
        vp_frame.push(dot(stage, vp.p(C::real(peak)), 0.05, Palette::dim()));
        vp_frame.push(kit.text(
            stage,
            fact("bump.peak")?,
            20.0,
            Palette::dim(),
            vp.p(C::new(peak, -0.12)),
        )?);
        fade_in_all(stage, &vp_frame, 0.4)?;
        // what the voice names: the function, the radius, the centre
        let bump_name = kit.tex(stage, BUMP_Z_TEX, 28.0, Palette::ink(), &[], v(-5.0, 3.0))?;
        let r_half = carded(
            &kit,
            stage,
            R_NAMES[0],
            20.0,
            Palette::ink(),
            p.p(C::new(-1.05, 0.3)),
            hero(),
        )?;
        let a_is = carded(
            &kit,
            stage,
            CENTRE_ZERO,
            20.0,
            Palette::ink(),
            p.p(CENTRE_CARD_AT),
            Palette::faint(),
        )?;
        fade_in_all(stage, &[bump_name, r_half[0], r_half[1], a_is[0], a_is[1]], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let [few, many] = model::AVERAGE_SAMPLES;
        let counts: Vec<&str> = fact("cauchy.samples")?.split(',').collect();
        let (dots8, mean8) = sample_round(stage, &p, &vp, 0.5, few)?;
        let avg8 = kit.text(
            stage,
            &average_says(counts[0], fact("cauchy.r_half.n8")?),
            24.0,
            hero(),
            v(right, -1.4),
        )?;
        fade_in_all(stage, &[avg8], 0.4)?;
        let pair_up = carded(
            &kit,
            stage,
            PAIR_UP,
            18.0,
            Palette::dim(),
            v(right, -0.75),
            Palette::faint(),
        )?;
        fade_in_all(stage, &pair_up, 0.3)?;
        vp_frame.extend(pair_up);
        line.wait_sentence(stage, 2)?;
        let mut gone = dots8;
        gone.push(mean8);
        fade_out_all(stage, &gone, 0.3)?;
        let (dots16, mean16) = sample_round(stage, &p, &vp, 0.5, many)?;
        let avg16 = kit.text(
            stage,
            &average_says(counts[1], fact("cauchy.r_half.n16")?),
            24.0,
            hero(),
            v(right, -1.95),
        )?;
        fade_in_all(stage, &[avg16], 0.4)?;

        // 09_4: radius 2: is the average still 1?
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_4")?;
        let mut gone = dots16;
        // one ring at a time: the small one comes back with its samples in 09_6
        gone.extend([mean16, avg8, avg16, half, r_half[0], r_half[1]]);
        fade_out_all(stage, &gone, 0.3)?;
        line.wait_sentence(stage, 0)?;
        let two = p.circle(stage, ZERO, 2.0, hero(), 5.0)?;
        let r_two = carded(
            &kit,
            stage,
            R_NAMES[1],
            20.0,
            Palette::ink(),
            p.p(C::new(-2.0, 1.85)),
            hero(),
        )?;
        fade_in_all(stage, &[two, r_two[0], r_two[1]], 0.5)?;
        let (dots2, mean2) = sample_round_hidden(stage, &p, &vp, 2.0, many)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right, -2.0))?;

        // 09_5: zero: each swallowed pole pulls the average down by one half
        let line = say(&kit, stage, "09_5")?;
        fade_out_all(stage, &asked, 0.3)?;
        fade_in_all(stage, &[mean2], 0.4)?;
        let avg2 = kit.text(
            stage,
            &average_says(counts[1], &signed(fact("cauchy.r_two.n16")?)),
            24.0,
            hero(),
            v(right, -1.35),
        )?;
        fade_in_all(stage, &[avg2], 0.4)?;
        let limit = kit.text(
            stage,
            &format!("{LIMIT}{}", fact("cauchy.r_two.limit")?),
            24.0,
            hero(),
            v(right, -1.8),
        )?;
        fade_in_all(stage, &[limit], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let share = signed(fact("cauchy.pole_share")?);
        // the poles ringed on "swallowed", each one's share on "Each pole pulls", on a card
        // because pole-coloured words do not read on the plate around a pole
        let mut flash = Vec::new();
        for z in [model::I, -model::I] {
            flash.push(p.circle(stage, z, 0.22, pole(), 5.0)?);
        }
        fade_in_all(stage, &flash, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let mut shares = Vec::new();
        for z in [model::I, -model::I] {
            shares.extend(carded(
                &kit,
                stage,
                &share,
                24.0,
                Palette::ink(),
                p.p(z + SHARE_CARD_FROM_POLE),
                pole(),
            )?);
        }
        fade_in_all(stage, &shares, 0.3)?;
        flash.extend(shares);
        let tally = kit.text(
            stage,
            &format!(
                "{SHARES}{} {} {} = {}",
                fact("bump.peak")?,
                share.replace('−', "− "),
                share.replace('−', "− "),
                fact("cauchy.r_two.limit")?
            ),
            22.0,
            Palette::ink(),
            v(right, -2.25),
        )?;
        fade_in_all(stage, &[tally], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let needs = precisely_now(&kit, stage, NEEDS)?;
        wait_until(stage, line.end())?;

        // 09_6: turn the samples back by their angles: the coefficients
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "09_6")?;
        let mut gone = vec![two, r_two[0], r_two[1], mean2, avg2, limit, tally, needs, formula];
        gone.extend(dots2);
        gone.extend(flash);
        gone.extend(vp_frame);
        fade_out_all(stage, &gone, 0.4)?;
        let coef_tex = kit.tex(stage, COEF_TEX, 32.0, Palette::ink(), &[], v(2.0, 3.25))?;
        let tp = turned_plane();
        let mut tp_frame = vec![tp.border(stage, output())?];
        tp_frame.extend(tp.axes(stage)?);
        tp_frame.push(kit.text(
            stage,
            VALUES,
            22.0,
            output(),
            v(tp.frame.center[0], tp.frame.top() + 0.25),
        )?);
        tp_frame.push(dot(stage, tp.p(C::real(peak)), 0.05, Palette::dim()));
        tp_frame.push(kit.text(
            stage,
            fact("bump.peak")?,
            20.0,
            Palette::dim(),
            tp.p(C::new(peak, -0.2)),
        )?);
        tp_frame.push(kit.tex(stage, THETA_TEX, 24.0, Palette::dim(), &[], v(right, 2.55))?);
        fade_in_all(stage, &[coef_tex], 0.4)?;
        fade_in_all(stage, &tp_frame, 0.3)?;
        let n = many;
        let on_circle: Vec<Mob> = samples(0.5, n)
            .iter()
            .map(|&z| dot(stage, p.p(z), 0.05, input()))
            .collect();
        let vals: Vec<Mob> = turned_values(0.5, n, 0.0)
            .iter()
            .map(|&w| dot(stage, tp.p(w), 0.06, output()))
            .collect();
        let mean = stage.arena_mut().add(
            Circle::new()
                .radius(0.12)
                .style(line_style(hero(), 4.0))
                .build()
                .moved_to(tp.p(centroid(&turned_values(0.5, n, 0.0)))),
        );
        let r_again = carded(
            &kit,
            stage,
            R_NAMES[0],
            20.0,
            Palette::ink(),
            p.p(C::new(-1.05, 0.3)),
            hero(),
        )?;
        let mut small_ring = on_circle.clone();
        small_ring.extend([half, r_again[0], r_again[1]]);
        fade_in_all(stage, &small_ring, 0.3)?;
        fade_in_all(stage, &vals, 0.3)?;
        fade_in_all(stage, &[mean], 0.3)?;
        let s = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let moving = vals.clone();
        stage.add_updater(
            anchor,
            move |a, _| {
                let sv = value(a, s);
                let ws = turned_values(0.5, n, sv);
                for (m, w) in moving.iter().zip(&ws) {
                    a.move_to(*m, tp.p(*w), ORIGIN);
                }
                a.move_to(mean, tp.p(centroid(&ws)), ORIGIN);
            },
            true,
        )?;
        line.wait_sentence(stage, 1)?;
        let c: Vec<&str> = fact("cauchy.coef")?.split(',').collect();
        let turned_once = kit.text(stage, TURNED[0], 20.0, Palette::dim(), v(right, -1.95))?;
        fade_in_all(stage, &[turned_once], 0.3)?;
        stage.play(tween_linear(s, 1.0, 2.2)?)?;
        line.wait_sentence(stage, 2)?;
        let read1 = kit.tex(
            stage,
            &format!("{}{}", READ[0], c[1]),
            26.0,
            hero(),
            &[],
            v(right, -2.55),
        )?;
        let slope = kit.text(stage, SLOPE, 18.0, hero(), v(1.35, -2.55))?;
        fade_in_all(stage, &[read1, slope], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let turned_twice = kit.text(stage, TURNED[1], 20.0, Palette::dim(), v(right, -1.95))?;
        fade_out_all(stage, &[turned_once], 0.2)?;
        fade_in_all(stage, &[turned_twice], 0.3)?;
        stage.play(tween_linear(s, 2.0, 2.2)?)?;
        let read2 = kit.tex(
            stage,
            &format!("{}{}", READ[1], c[2]),
            26.0,
            hero(),
            &[],
            v(right, -3.4),
        )?;
        let next = kit.text(stage, NEXT, 18.0, hero(), v(1.5, -3.4))?;
        fade_in_all(stage, &[read2, next], 0.4)?;
        stage.clear_updaters(anchor, true);
        line.wait_sentence(stage, 4)?;
        let list = kit.tex(
            stage,
            &format!(
                r"c_0 \approx {},\; c_1 \approx {},\; c_2 \approx {},\; c_3 \approx {},\; c_4 \approx {}",
                c[0], c[1], c[2], c[3], c[4]
            ),
            26.0,
            Palette::ink(),
            &[],
            v(-3.3, -3.45),
        )?;
        fade_in_all(stage, &[list], 0.5)?;
        // "hidden in the boundary values": the ring they came from lights
        let lit = p.circle(stage, ZERO, 0.5, hero(), 10.0)?;
        fade_in_all(stage, &[lit], 0.3)?;
        fade_out_all(stage, &[lit], 0.5)?;
        wait_until(stage, line.end())?;

        // 09_7: the opening puzzle answered: holomorphic out to the poles, radius 1
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_7")?;
        let mut gone = vec![
            coef_tex,
            mean,
            read1,
            read2,
            slope,
            next,
            turned_twice,
            list,
            half,
            r_again[0],
            r_again[1],
            a_is[0],
            a_is[1],
        ];
        gone.extend(on_circle);
        gone.extend(vals);
        gone.extend(tp_frame);
        fade_out_all(stage, &gone, 0.4)?;
        let g = graph();
        let mut axes_g = vec![
            stage.arena_mut().add(
                polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
                    .style(faded_line(Palette::dim(), 2.0, 0.9))
                    .build(),
            ),
            stage.arena_mut().add(
                polyline(vec![g.p(0.0, g.y.0), g.p(0.0, g.y.1)])
                    .style(faded_line(Palette::dim(), 2.0, 0.9))
                    .build(),
            ),
        ];
        for piece in clipped(&g, model::bump) {
            axes_g.push(
                stage
                    .arena_mut()
                    .add(polyline(piece).style(line_style(Palette::ink(), 4.0)).build()),
            );
        }
        let tick = |x: f64| {
            let q = g.p(x, 0.0);
            v(q[0], q[1] - 0.24)
        };
        axes_g.push(kit.text(stage, "−1", 20.0, Palette::dim(), tick(-1.0))?);
        axes_g.push(kit.text(stage, "1", 20.0, Palette::dim(), tick(1.0))?);
        let axis_end = g.p(g.x.1, 0.0);
        axes_g.push(kit.text(stage, "x", 22.0, Palette::dim(), v(axis_end[0] + 0.18, axis_end[1]))?);
        axes_g.push(kit.text(stage, TITLE, 20.0, Palette::ink(), g.p(2.0, 0.45))?);
        fade_in_all(stage, &axes_g, 0.5)?;
        let recipe0: Vec<Mob> = clipped(&g, |x| model::bump_recipe0(x, 40))
            .into_iter()
            .map(|pc| {
                stage
                    .arena_mut()
                    .add(polyline(pc).style(line_style(hero(), 3.0)).build())
            })
            .collect();
        let recipe_name = kit.text(stage, RECIPE_NAME, 20.0, hero(), g.p(-0.55, 1.2))?;
        let mut recipe_shown = recipe0.clone();
        recipe_shown.push(recipe_name);
        fade_in_all(stage, &recipe_shown, 0.5)?;
        // the largest disk: it grows from the centre until it hits the poles; a's name, which it
        // passes, makes way while its line is near
        line.wait_for(stage, "the largest disk")?;
        let unit = p.circle(stage, ZERO, 1e-3, hero(), 5.0)?;
        fade_in_all(stage, &[unit], 0.1)?;
        let disk = stage.add_value_tracker(1e-3);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, disk).max(1e-3);
                let pts: Vec<Vec3> = (0..=120)
                    .map(|k| p.p(C::polar(r, TAU * f64::from(k) / 120.0)))
                    .collect();
                let _ = a.set_points(unit, polyline(pts).build().points());
                a.set_opacity(a_name, a_name_shown(r), true);
            },
            true,
        )?;
        let rt = line.until(stage, "That answers", 1.6)?.min(3.0);
        stage.play(tween_linear(disk, fact_f64("bump.radius0")?, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        line.wait_sentence(stage, 2)?;
        let mut pole_names = Vec::new();
        for (z, s, dy) in [(model::I, POLE_I, 0.3), (-model::I, POLE_MINUS_I, -0.3)] {
            pole_names.extend(carded(
                &kit,
                stage,
                s,
                22.0,
                Palette::ink(),
                p.p(z + C::new(-0.42, dy)),
                pole(),
            )?);
        }
        fade_in_all(stage, &pole_names, 0.4)?;
        line.wait_sentence(stage, 3)?;
        let reach0 = p.path(stage, &[ZERO, model::I], hero(), 4.0)?;
        let radius_name = carded(
            &kit,
            stage,
            &format!("{RADIUS}{}", fact("bump.radius0")?),
            20.0,
            Palette::ink(),
            p.p(C::new(1.5, 0.7)),
            hero(),
        )?;
        let glow0 = stage.arena_mut().add(
            polyline(vec![g.p(-1.0, 0.0), g.p(1.0, 0.0)])
                .style(faded_line(hero(), 12.0, 0.55))
                .build(),
        );
        fade_in_all(stage, &[reach0, radius_name[0], radius_name[1], glow0], 0.5)?;
        // "no further": the pole that stops it
        line.wait_for(stage, "no further")?;
        let stops = p.circle(stage, model::I, 0.2, pole(), 5.0)?;
        fade_in_all(stage, &[stops], 0.3)?;

        // 09_8: recentre at 1: how far now?
        kit.hold(stage)?;
        let line = say(&kit, stage, "09_8")?;
        // the old disk goes as its centre leaves: the size of the new one is the question
        let mut gone = recipe0;
        gone.extend([glow0, recipe_name, reach0, radius_name[0], radius_name[1], stops, unit]);
        fade_out_all(stage, &gone, 0.3)?;
        let slide = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let c = C::real(value(a, slide));
                a.move_to(centre, p.p(c), ORIGIN);
                a.move_to(a_name, p.p(c + A_NAME_FROM_CENTRE), ORIGIN);
            },
            true,
        )?;
        stage.play(tween_linear(slide, 1.0, 1.8)?)?;
        stage.clear_updaters(anchor, true);
        let one_name = carded(
            &kit,
            stage,
            "1",
            22.0,
            Palette::ink(),
            p.p(C::new(1.0, -0.62)),
            Palette::faint(),
        )?;
        fade_in_all(stage, &one_name, 0.2)?;
        let asked = predict(&kit, stage, &line, &ASK_RECENTRE, v(3.9, 2.2))?;

        // 09_9: radius √2 through ±i; the recipe at 1 hugs the bump on (−0.41, 2.41)
        let line = say(&kit, stage, "09_9")?;
        fade_out_all(stage, &asked, 0.3)?;
        let radius = fact_f64("recentre1.radius")?;
        // the new distance, drawn and named while the voice says it, for the disk to grow to
        let unit = p.circle(stage, C::real(1.0), 1e-3, hero(), 5.0)?;
        let reach1 = p.path(stage, &[C::real(1.0), model::I], hero(), 4.0)?;
        let reach_name = carded(
            &kit,
            stage,
            &format!("√2 ≈ {}", fact("recentre1.radius")?),
            20.0,
            Palette::ink(),
            p.p(C::new(0.95, 0.95)),
            hero(),
        )?;
        fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;
        // The disk also grows through the "1" card and a's name. Rooted again above the disk, the
        // card covers its line as the √2 card does; a makes way as in 09_7.
        for m in one_name {
            stage.add_to_scene(m)?;
        }
        let grow = stage.add_value_tracker(1e-3);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let r = value(a, grow).max(1e-3);
                let pts: Vec<Vec3> = (0..=160)
                    .map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))
                    .collect();
                let _ = a.set_points(unit, polyline(pts).build().points());
                a.set_opacity(a_name, a_name_shown(r), true);
            },
            true,
        )?;
        let rt = line.until(stage, "So the new recipe", 1.6)?;
        stage.play(tween_linear(grow, radius, rt)?)?;
        stage.clear_updaters(anchor, true);
        let (left, right_end) = (fact_f64("recentre1.left")?, fact_f64("recentre1.right")?);
        let glow1 = stage.arena_mut().add(
            polyline(vec![g.p(left, 0.0), g.p(right_end, 0.0)])
                .style(faded_line(hero(), 12.0, 0.55))
                .build(),
        );
        let works = kit.text(
            stage,
            &format!(
                "{WORKS}({}, {})",
                signed(fact("recentre1.left")?),
                fact("recentre1.right")?
            ),
            24.0,
            hero(),
            v(3.9, 1.45),
        )?;
        let recipe1: Vec<Mob> = clipped(&g, |x| model::bump_recipe(1.0, x, model::RECENTRE_CHECK_TERMS))
            .into_iter()
            .map(|pc| {
                stage
                    .arena_mut()
                    .add(polyline(pc).style(line_style(hero(), 3.0)).build())
            })
            .collect();
        let recipe_again = kit.text(stage, RECIPE_NAME, 20.0, hero(), g.p(1.0, 0.95))?;
        let mut recipe_shown = recipe1.clone();
        recipe_shown.push(recipe_again);
        fade_in_all(stage, &recipe_shown, 0.6)?;
        fade_in_all(stage, &[glow1, works], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let terms = fact("recentre1.check.terms")?;
        let points: Vec<&str> = fact("recentre1.check.at")?.split(',').collect();
        let check = [
            kit.text(
                stage,
                &format!(
                    "{terms}{CHECK}{}{COLON}{}{MATCHES}",
                    points[0],
                    fact("recentre1.check.at2")?
                ),
                22.0,
                Palette::ink(),
                v(3.9, 2.55),
            )?,
            kit.text(
                stage,
                &format!(
                    "{terms}{CHECK}{}{COLON}{}{OUTSIDE}",
                    points[1],
                    fact("recentre1.check.at2_6")?
                ),
                22.0,
                Palette::ink(),
                v(3.9, 2.0),
            )?,
        ];
        fade_in_all(stage, &check, 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            centre,
            a_name,
            unit,
            reach1,
            glow1,
            works,
            check[0],
            check[1],
            bump_name,
            recipe_again,
        ];
        all.extend(reach_name);
        all.extend(one_name);
        all.extend(pole_names);
        all.extend(p_frame);
        all.extend(poles);
        all.extend(axes_g);
        all.extend(recipe1);
        fade_out_all(stage, &all, 0.5)?;
        painter.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

/// `n` samples on the circle of radius `r`: dots on the picture, their values
/// in `vp`, and the centroid ring. Returns the dots and the ring.
fn sample_round(stage: &mut Stage<'_>, p: &Plane, vp: &Plane, r: f64, n: usize) -> fmn::Result<(Vec<Mob>, Mob)> {
    let (dots, mean) = sample_round_hidden(stage, p, vp, r, n)?;
    pause(stage, 0.3)?;
    fade_in_all(stage, &[mean], 0.4)?;
    Ok((dots, mean))
}

/// As [`sample_round`], but the centroid ring is built and left hidden.
fn sample_round_hidden(stage: &mut Stage<'_>, p: &Plane, vp: &Plane, r: f64, n: usize) -> fmn::Result<(Vec<Mob>, Mob)> {
    let zs = samples(r, n);
    let ws: Vec<C> = zs.iter().map(|&z| bump_at(z)).collect();
    let mut dots: Vec<Mob> = zs.iter().map(|&z| dot(stage, p.p(z), 0.06, input())).collect();
    let shown: Vec<Mob> = ws.iter().map(|&w| dot(stage, vp.p(w), 0.06, output())).collect();
    fade_in_all(stage, &dots, 0.4)?;
    fade_in_all(stage, &shown, 0.4)?;
    dots.extend(shown);
    let mean = stage.arena_mut().add(
        Circle::new()
            .radius(0.12)
            .style(line_style(hero(), 4.0))
            .build()
            .moved_to(vp.p(centroid(&ws))),
    );
    Ok((dots, mean))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(key: &str) -> f64 {
        crate::facts::facts().unwrap().get(key).unwrap().parse().unwrap()
    }

    #[test]
    fn the_centroids_drawn_are_the_averages_said() {
        let at = |r: f64, n: usize| centroid(&samples(r, n).iter().map(|&z| bump_at(z)).collect::<Vec<_>>());
        let [few, many] = model::AVERAGE_SAMPLES;
        assert!((at(0.5, few).re - num("cauchy.r_half.n8")).abs() < 5e-5);
        assert!((at(0.5, many).re - num("cauchy.r_half.n16")).abs() < 5e-7);
        assert!((at(2.0, many).re - num("cauchy.r_two.n16")).abs() < 5e-7);
        assert!(at(0.5, few).im.abs() < 1e-12 && at(2.0, many).im.abs() < 1e-12);
        // the limit said is what more samples close in on
        assert!((at(2.0, 256).re - num("cauchy.r_two.limit")).abs() < 1e-9);
    }

    #[test]
    fn the_captions_keep_their_words_around_the_facts() {
        assert_eq!(average_says("8", "1.0039"), "average of 8: 1.0039");
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("cauchy.samples"), Some("8,16"));
        assert_eq!(f.get("recentre1.check.terms"), Some("60"));
        assert_eq!(f.get("recentre1.check.at"), Some("2,2.6"));
        assert_eq!(f.get("cauchy.r_two.limit"), Some("0"));
    }

    #[test]
    fn turning_back_reads_the_coefficients() {
        let c: Vec<f64> = crate::facts::facts()
            .unwrap()
            .get("cauchy.coef")
            .unwrap()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        for (s, k) in [(0.0, 0usize), (1.0, 1), (2.0, 2)] {
            let mean = centroid(&turned_values(0.5, model::AVERAGE_SAMPLES[1], s));
            assert!((mean.re / 0.5f64.powi(k as i32) - c[k]).abs() < 1e-3, "c{k}");
        }
    }

    #[test]
    fn opposite_samples_share_a_value() {
        for n in model::AVERAGE_SAMPLES {
            let zs = samples(0.5, n);
            for k in 0..n / 2 {
                assert!((bump_at(zs[k]) - bump_at(zs[k + n / 2])).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn a_curve_puts_zero_and_both_ones_on_its_grid() {
        let g = graph();
        let step = (g.x.1 - g.x.0) / f64::from(CURVE_STEPS);
        for x in [-1.0, 0.0, 1.0] {
            let k = (x - g.x.0) / step;
            assert!((k - k.round()).abs() < 1e-9, "{x} is off the grid");
        }
    }

    /// Picture review 09-N2: the lower share card covered the bottom of "a = 0" for 14 s. Half
    /// sizes in scene units, measured at 1280 × 720 on frame 240 of 09.
    #[test]
    fn the_centre_card_is_clear_of_what_joins_it_on_the_picture() {
        let p = picture();
        let (at, centre) = (p.p(CENTRE_CARD_AT), p.p(ZERO));
        let (half_w, half_h) = (44.0 / 90.0, 22.5 / 90.0);
        let (share_w, share_h) = (41.5 / 90.0, 23.5 / 90.0);
        for z in [model::I, -model::I] {
            let share = p.p(z + SHARE_CARD_FROM_POLE);
            let apart_x = (at[0] - share[0]).abs() - half_w - share_w;
            let apart_y = (at[1] - share[1]).abs() - half_h - share_h;
            assert!(apart_x.max(apart_y) > 0.1, "{z:?}: {apart_x}, {apart_y}");
        }
        // between the small ring of 09_3 and 09_6, with its dots, and the large ring of 09_4
        let (dx, dy) = ((at[0] - centre[0]).abs(), (at[1] - centre[1]).abs());
        assert!(dx > half_w && dy > half_h);
        let nearest = (dx - half_w).hypot(dy - half_h);
        let farthest = (dx + half_w).hypot(dy + half_h);
        assert!(nearest > 0.5 * p.unit() + 0.1, "{nearest}");
        assert!(farthest < 2.0 * p.unit() - 0.1, "{farthest}");
    }

    #[test]
    fn every_point_drawn_stays_in_its_window() {
        let (p, vp, tp) = (picture(), values_plane(), turned_plane());
        for r in [0.5, 2.0] {
            for z in samples(r, model::AVERAGE_SAMPLES[1]) {
                assert!(p.contains(z) && vp.contains(bump_at(z)), "radius {r}");
            }
        }
        for k in 0..=20 {
            for w in turned_values(0.5, model::AVERAGE_SAMPLES[1], f64::from(k) / 10.0) {
                assert!(tp.contains(w));
            }
        }
        // the recentred circle fits the picture and the graph shows its interval
        let r1 = num("recentre1.radius");
        assert!(1.0 + r1 < p.frame.x.1 && 1.0 - r1 > p.frame.x.0);
        let g = graph();
        assert!(g.x.0 < num("recentre1.left") && g.x.1 > num("recentre1.right"));
        for piece in clipped(&g, |x| model::bump_recipe(1.0, x, model::RECENTRE_CHECK_TERMS)) {
            for q in piece {
                assert!(q[1] >= g.bottom() - 1e-9 && q[1] <= g.top() + 1e-9);
            }
        }
        // the right column clears the picture
        assert!(p.frame.right() < vp.frame.left() && p.frame.right() < tp.frame.left());
        assert!(p.frame.right() < g.left());
    }

    #[test]
    fn the_ring_has_ccw_triangles_and_only_opposite_internal_walks() {
        for n in [SIDES, SIDES * 2, SIDES * 4, FINEST] {
            let triangles = ring_triangles(LOOP_R, SMALL_R, n);
            assert_eq!(triangles.len(), 2 * n);
            for t in &triangles {
                let (u, w) = (t[1] - t[0], t[2] - t[0]);
                assert!(u.re * w.im - u.im * w.re > 0.0);
                // Every edge, including the inner chord, stays strictly away from a.
                for k in 0..3 {
                    let (a, d) = (t[k], t[(k + 1) % 3] - t[k]);
                    let along = (-(a.re * d.re + a.im * d.im) / (d.re * d.re + d.im * d.im)).clamp(0.0, 1.0);
                    assert!((a + d.scale(along)).abs() >= SMALL_R * (PI / n as f64).cos() - 1e-12);
                }
            }
            let walks = ring_walks(&triangles);
            let same = |a: C, b: C| (a - b).abs() < 1e-12;
            for walk in &walks {
                let reverse = walks
                    .iter()
                    .filter(|w| same(w.edge[0], walk.edge[1]) && same(w.edge[1], walk.edge[0]))
                    .count();
                assert_eq!(reverse, usize::from(walk.inside));
                if !walk.inside {
                    let cross = walk.edge[0].re * walk.edge[1].im - walk.edge[0].im * walk.edge[1].re;
                    assert_eq!(cross > 0.0, (walk.edge[0].abs() - LOOP_R).abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn fixed_radius_refinement_has_vanishing_gap_and_bounded_total_length() {
        for radius in [LOOP_R, SMALL_R] {
            let mut last_gap = radius;
            for power in SIDES.ilog2()..=FINEST.ilog2() {
                let n = sides_at(f64::from(power));
                let corners = polygon(radius, n);
                assert!(corners.iter().all(|z| (z.abs() - radius).abs() < 1e-12));
                let perimeter: f64 = (0..n).map(|k| (corners[(k + 1) % n] - corners[k]).abs()).sum();
                let circle_length = TAU * radius;
                assert!(perimeter < circle_length);
                assert!(perimeter + circle_length < 2.0 * circle_length);
                let gap = radius * (1.0 - (PI / n as f64).cos());
                assert!(gap < last_gap);
                last_gap = gap;
            }
        }
    }

    #[test]
    fn shrinking_bound_and_picture_keep_the_excluded_point_empty() {
        let radii = [SHRUNK_R, MID_R, SMALL_R, BOUND_R, LOOP_R];
        assert!(radii.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(SHRUNK_R * close_up().unit() > HOLE);
        let length = |r| {
            let p = bar_points(r);
            p[1][0] - p[0][0]
        };
        assert!((length(MID_R) / length(SMALL_R) - MID_R / SMALL_R).abs() < 1e-12);
        assert!((length(SHRUNK_R) / length(SMALL_R) - SHRUNK_R / SMALL_R).abs() < 1e-12);
        assert_eq!(length(0.0), 0.0);
    }

    #[test]
    fn the_repaired_proof_waits_follow_the_recorded_words() {
        let spoken = crate::SCRIPT
            .lines()
            .find_map(|line| line.strip_prefix("09_1\t"))
            .unwrap();
        let source = include_str!("boundary.rs");
        // Restrict to construct, so this test's own strings cannot satisfy it.
        let body = source
            .split("// 09_1: divide by")
            .nth(1)
            .unwrap()
            .split("// 09_2: on a circle")
            .next()
            .unwrap();
        let mut previous = 0;
        let mut count = 0;
        for call in body.split("line.wait_for(").skip(1) {
            let phrase = call.split('"').nth(1).unwrap();
            let position = spoken
                .find(phrase)
                .unwrap_or_else(|| panic!("missing phrase: {phrase}"));
            assert!(position >= previous, "out-of-order phrase: {phrase}");
            previous = position;
            count += 1;
        }
        assert!(count >= 25);
        let fixed = body.find("Now keep the small circle fixed").unwrap();
        let equality = body
            .find("So in the limit, the big circle's integral equals the small circle's")
            .unwrap();
        let shrink = body.find("Only now do we shrink the small circle").unwrap();
        let result = body.find("let result =").unwrap();
        assert!(fixed < equality && equality < shrink && shrink < result);
    }

    /// Native 4K pixels per scene unit; the ink boxes' half sizes of a and p (size 20) at that
    /// scale, measured in the final 09 review frame at 72 s; the least gap the repaired heads and
    /// name keep from their neighbours (about 9.5 native pixels). A line of width w is w/100
    /// scene units wide (the width 3 dashes measure 8 native pixels).
    const PX: f64 = 270.0;
    const A_HALF: (f64, f64) = (12.0 / PX, 12.5 / PX);
    const P_HALF: (f64, f64) = (13.5 / PX, 17.5 / PX);
    const CLEAR: f64 = 0.035;

    fn luminance(c: Srgb) -> f64 {
        let linear = |u: f64| {
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    }

    /// The contrast ratio of two colours, 1 to 21.
    fn contrast(a: Srgb, b: Srgb) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    fn cross(o: Vec3, p: Vec3, q: Vec3) -> f64 {
        (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
    }

    fn apart(p: Vec3, q: Vec3) -> f64 {
        (p[0] - q[0]).hypot(p[1] - q[1])
    }

    fn to_segment(q: Vec3, a: Vec3, b: Vec3) -> f64 {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let t = (((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        (q[0] - a[0] - t * dx).hypot(q[1] - a[1] - t * dy)
    }

    fn edges(p: &[Vec3]) -> Vec<(Vec3, Vec3)> {
        (0..p.len()).map(|k| (p[k], p[(k + 1) % p.len()])).collect()
    }

    fn holds(edges: &[(Vec3, Vec3)], q: Vec3) -> bool {
        edges.iter().all(|&(u, w)| cross(u, w, q) >= 0.0) || edges.iter().all(|&(u, w)| cross(u, w, q) <= 0.0)
    }

    fn nearest(points: &[Vec3], edges: &[(Vec3, Vec3)]) -> f64 {
        let mut best = f64::INFINITY;
        for &q in points {
            for &(u, w) in edges {
                best = best.min(to_segment(q, u, w));
            }
        }
        best
    }

    /// The gap between two convex outlines: 0 if they touch, cross or one holds the other.
    fn gap(a: &[Vec3], b: &[Vec3]) -> f64 {
        let (ea, eb) = (edges(a), edges(b));
        let crossing = ea.iter().any(|&(p, q)| {
            eb.iter()
                .any(|&(r, s)| cross(p, q, r) * cross(p, q, s) < 0.0 && cross(r, s, p) * cross(r, s, q) < 0.0)
        });
        if crossing || a.iter().any(|&q| holds(&eb, q)) || b.iter().any(|&q| holds(&ea, q)) {
            return 0.0;
        }
        nearest(a, &eb).min(nearest(b, &ea))
    }

    fn label_box(at: Vec3, (w, h): (f64, f64)) -> Vec<Vec3> {
        vec![
            v(at[0] - w, at[1] - h),
            v(at[0] + w, at[1] - h),
            v(at[0] + w, at[1] + h),
            v(at[0] - w, at[1] + h),
        ]
    }

    /// Picture review 747, B1: the inner polygon's heads were green on its green line and read as
    /// notches, so neither its clockwise walk nor its turn to counterclockwise could be seen. The
    /// heads drawn now: the walks' yellow inside a rim of the background, larger than the outer
    /// heads, one on the middle of each side, pointing the way that side is walked, and clear of
    /// a's name, the hole's ring and the dashes.
    #[test]
    fn the_inner_heads_stand_out_and_point_the_way_each_side_is_walked() {
        let style = inner_head_style();
        assert_eq!(style.fill_color, hero());
        assert_eq!(style.stroke_color, ground());
        assert!(style.fill_opacity >= 1.0 && style.stroke_opacity >= 1.0);
        assert!((style.stroke_width - INNER_HEAD_RIM).abs() < 1e-12);
        assert!(
            contrast(style.stroke_color, output()) > 4.5,
            "rim against the green line"
        );
        assert!(
            contrast(style.fill_color, style.stroke_color) > 4.5,
            "fill against its rim"
        );
        // over 40 native pixels long and wide; the rim's outer half over 3
        assert!(INNER_HEAD > OUTER_HEAD && INNER_HEAD * PX > 40.0);
        let rim = INNER_HEAD_RIM / 100.0 / 2.0;
        assert!(rim * PX > 3.0);

        let cp = close_up();
        let (centre, unit) = (cp.p(ZERO), cp.unit());
        let corners = polygon(SMALL_R, SIDES);
        let name_a = label_box(cp.p(A_AT), A_HALF);
        for counterclockwise in [false, true] {
            let heads = side_head_points(&cp, &corners, counterclockwise, INNER_HEAD);
            assert_eq!(heads.len(), SIDES);
            for (k, head) in heads.iter().enumerate() {
                let (a, b) = (cp.p(corners[k]), cp.p(corners[(k + 1) % SIDES]));
                let middle = v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0);
                let base = v((head[1][0] + head[2][0]) / 2.0, (head[1][1] + head[2][1]) / 2.0);
                let half_way = v((head[0][0] + base[0]) / 2.0, (head[0][1] + base[1]) / 2.0);
                assert!(apart(half_way, middle) < 1e-9, "side {k} off its middle");
                assert!((apart(head[0], base) - INNER_HEAD).abs() < 1e-9);
                assert!((apart(head[1], head[2]) - INNER_HEAD).abs() < 1e-9);
                // counterclockwise about a: the tip lies to the left of a's line to the middle
                assert_eq!(cross(centre, middle, head[0]) > 0.0, counterclockwise, "side {k}");
                assert!(gap(head, &name_a) - rim > CLEAR, "side {k}: a's name");
                let inmost = (0..3)
                    .map(|j| to_segment(centre, head[j], head[(j + 1) % 3]))
                    .fold(f64::INFINITY, f64::min);
                let outmost = head.iter().map(|&q| apart(q, centre)).fold(0.0, f64::max);
                assert!(inmost - rim > HOLE + 0.015 + CLEAR, "side {k}: the hole");
                assert!(outmost + rim < BOUND_R * unit - 0.015 - CLEAR, "side {k}: the dashes");
            }
        }
    }

    /// Picture review 747, B1: p, the inner polygon's name, stood beyond the outer polygon's
    /// corner, about 0.45 from the inner polygon. It now stands by the inner polygon, in the ring
    /// between the small circle's line and the dashes, clear of the inner heads both ways round;
    /// each name is far nearer its own polygon than the other one.
    #[test]
    fn each_polygon_name_stands_by_its_own_polygon() {
        let cp = close_up();
        let (centre, unit) = (cp.p(ZERO), cp.unit());
        let rim = INNER_HEAD_RIM / 100.0 / 2.0;
        let name = label_box(cp.p(POLYGON_NAMES_AT[1]), P_HALF);
        let inmost = (0..4)
            .map(|j| to_segment(centre, name[j], name[(j + 1) % 4]))
            .fold(f64::INFINITY, f64::min);
        let outmost = name.iter().map(|&q| apart(q, centre)).fold(0.0, f64::max);
        // the small circle's line is width 4, the dashes width 3
        assert!(inmost > SMALL_R * unit + 0.02 + CLEAR, "{inmost}");
        assert!(outmost < BOUND_R * unit - 0.015 - CLEAR, "{outmost}");
        for counterclockwise in [false, true] {
            for head in side_head_points(&cp, &polygon(SMALL_R, SIDES), counterclockwise, INNER_HEAD) {
                assert!(gap(&head, &name) - rim > CLEAR);
            }
        }
        let to_polygon = |at: Vec3, r: f64| {
            let corners: Vec<Vec3> = polygon(r, SIDES).into_iter().map(|z| cp.p(z)).collect();
            (0..SIDES)
                .map(|k| to_segment(at, corners[k], corners[(k + 1) % SIDES]))
                .fold(f64::INFINITY, f64::min)
        };
        let (outer_name, inner_name) = (cp.p(POLYGON_NAMES_AT[0]), cp.p(POLYGON_NAMES_AT[1]));
        assert!(3.0 * to_polygon(inner_name, SMALL_R) < to_polygon(inner_name, LOOP_R));
        assert!(3.0 * to_polygon(outer_name, LOOP_R) < to_polygon(outer_name, SMALL_R));
    }

    /// The least and greatest distance from `centre` of a label box (four corners, in order).
    fn reach(centre: Vec3, name: &[Vec3]) -> (f64, f64) {
        let inmost = (0..4)
            .map(|j| to_segment(centre, name[j], name[(j + 1) % 4]))
            .fold(f64::INFINITY, f64::min);
        let outmost = name.iter().map(|&q| apart(q, centre)).fold(0.0, f64::max);
        (inmost, outmost)
    }

    /// Final review 995, M1: each bound's closing \Big| touched the z of dz. A thin space now
    /// stands between them.
    #[test]
    fn the_closing_bars_keep_clear_of_dz() {
        for tex in [ARC_TEX, SUM_TEX, SHRINK_TEX] {
            let (before, after) = tex.rsplit_once(r"\Big|").unwrap();
            assert!(before.ends_with(r"g\,dz\,"), "{tex}");
            assert!(after.starts_with(r" \le "), "{tex}");
        }
    }

    /// Final review 995, M2 and M3: ρ's name jumped from RHO_AT onto the dashes as the shrink
    /// began, and a's name, held at A_AT, was crossed by the shrinking circle. Both now ride
    /// outside the circle's line at every radius it passes, clear of it, the dashes, the hole's
    /// ring and each other; a comes to rest straight above the hole, 12 native pixels above A_AT.
    #[test]
    fn the_shrinking_circle_never_crosses_a_or_rho() {
        // ρ's ink at size 22: 25 x 42 native pixels, widened to 30 for the overview's reading
        const RHO_HALF: (f64, f64) = (15.0 / PX, 21.0 / PX);
        let cp = close_up();
        let (centre, unit) = (cp.p(ZERO), cp.unit());
        for k in 0..=1000 {
            let r = SHRUNK_R + (SMALL_R - SHRUNK_R) * f64::from(k) / 1000.0;
            let [a_at, rho_at] = shrink_names(r);
            let names = [label_box(cp.p(a_at), A_HALF), label_box(cp.p(rho_at), RHO_HALF)];
            for name in &names {
                let (inmost, outmost) = reach(centre, name);
                // the small circle's line is width 4, the dashes and the hole's ring width 3
                assert!(inmost > r * unit + 0.02 + CLEAR, "{r}: the circle");
                assert!(outmost < BOUND_R * unit - 0.015 - CLEAR, "{r}: the dashes");
                assert!(inmost > HOLE + 0.015 + CLEAR, "{r}: the hole");
            }
            assert!(gap(&names[0], &names[1]) > CLEAR, "{r}: a and ρ");
        }
        let [a_end, _] = shrink_names(SHRUNK_R);
        assert!(apart(cp.p(a_end), cp.p(A_AT)) * PX < 12.5);
    }

    /// Final review 995, L14: at 0.045 the shrunk circle's line (width 4) met the hole's ring
    /// (width 3). Where the shrink stops now, a drawn gap of more than 7 native pixels stays
    /// between them.
    #[test]
    fn the_shrunk_circle_keeps_apart_from_the_hole_ring() {
        let drawn_gap = (SHRUNK_R * close_up().unit() - 0.02) - (HOLE + 0.015);
        assert!(drawn_gap * PX > 7.0, "{}", drawn_gap * PX);
    }

    /// The shrink's names are made where its first frame puts them, and they leave and come back
    /// in fades that were already there, so they neither jump nor add time.
    #[test]
    fn the_shrinking_names_start_where_the_shrink_puts_them() {
        let source = include_str!("boundary.rs");
        let shrink = source
            .split("// Second limit:")
            .nth(1)
            .unwrap()
            .split("// A finite picture")
            .next()
            .unwrap();
        for wanted in [
            "let [a_from, rho_from] = shrink_names(SMALL_R);",
            "fade_out_all(stage, &leaving, 0.2)?;",
            "fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;",
            "let [a_at, rho_at] = shrink_names(r);",
        ] {
            assert_eq!(shrink.matches(wanted).count(), 1, "{wanted}");
        }
        assert!(!shrink.contains("r + 0.06"));
    }

    /// Final review 995, M4: 09_7's disk, centred on 0, and 09_9's, centred on 1, each grow from
    /// radius 0.001 (to 1 and to √2) through a's name, which stands at the same place from each
    /// centre. It keeps its place and makes way: hidden whenever a disk's line comes within CLEAR
    /// of its ink, fully shown as each disk starts and stops.
    #[test]
    fn the_growing_disks_never_draw_through_a() {
        // a's ink at size 28: a's 24 x 25 native pixels at 20, scaled, then widened to the
        // overview's reading
        const A_NAME_HALF: (f64, f64) = (17.5 / PX, 19.0 / PX);
        let p = picture();
        let unit = p.unit();
        for (at, end) in [(ZERO, num("bump.radius0")), (C::real(1.0), num("recentre1.radius"))] {
            let name = label_box(p.p(at + A_NAME_FROM_CENTRE), A_NAME_HALF);
            let (inmost, outmost) = reach(p.p(at), &name);
            let mut hidden = 0;
            for k in 0..=4000 {
                let r = 1e-3 + (end - 1e-3) * f64::from(k) / 4000.0;
                // the disks' lines are width 5
                if r * unit + 0.025 + CLEAR > inmost && r * unit - 0.025 - CLEAR < outmost {
                    assert!(a_name_shown(r) < 1e-12, "{r}");
                    hidden += 1;
                }
            }
            assert!(hidden > 0, "{end}");
            for r in [1e-3, end] {
                assert!(a_name_shown(r) > 1.0 - 1e-12, "{r}");
            }
        }
    }

    /// Final review 995 (09_9, from the source): the disk about 1 also grows through the "1" card
    /// below its centre. The card is rooted again after the disk fades in, so it lies above the
    /// disk's line as the √2 card does, and both growths fade a's name by the disk's radius.
    #[test]
    fn the_growing_disks_pass_under_the_cards_and_round_a() {
        let source = include_str!("boundary.rs");
        let film = source.split("\n#[cfg(test)]\nmod tests {").next().unwrap();
        assert_eq!(film.matches("a.set_opacity(a_name, a_name_shown(r), true);").count(), 2);
        // what the geometry test above assumes: each disk starts at radius 0.001 about its centre
        // and grows to its fact, and 09_8's slide leaves a's name at 1 + A_NAME_FROM_CENTRE
        for wanted in [
            "let disk = stage.add_value_tracker(1e-3);",
            ".map(|k| p.p(C::polar(r, TAU * f64::from(k) / 120.0)))",
            "stage.play(tween_linear(disk, fact_f64(\"bump.radius0\")?, rt)?)?;",
            "a.move_to(a_name, p.p(c + A_NAME_FROM_CENTRE), ORIGIN);",
            "stage.play(tween_linear(slide, 1.0, 1.8)?)?;",
            "let unit = p.circle(stage, C::real(1.0), 1e-3, hero(), 5.0)?;",
            "let grow = stage.add_value_tracker(1e-3);",
            ".map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))",
            "let radius = fact_f64(\"recentre1.radius\")?;",
            "stage.play(tween_linear(grow, radius, rt)?)?;",
        ] {
            assert_eq!(film.matches(wanted).count(), 1, "{wanted}");
        }
        let disk_in = film
            .find("fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;")
            .unwrap();
        let card_up = film
            .find("for m in one_name {\n            stage.add_to_scene(m)?;")
            .unwrap();
        let growth = film.find("let grow = stage.add_value_tracker(1e-3);").unwrap();
        assert!(disk_in < card_up && card_up < growth);
    }

    /// CC1's review of candidate01 (Medium): riding the circle, a's name could read as a moving
    /// centre. The leader ties it to the fixed hole it names: at every radius of the
    /// shrink it stops short of the hole's ring and of a's ink, crosses the circle's line once, and
    /// stays clear of ρ.
    #[test]
    fn the_leader_runs_from_the_hole_up_to_a() {
        const RHO_HALF: (f64, f64) = (15.0 / PX, 21.0 / PX);
        let cp = close_up();
        let (centre, unit) = (cp.p(ZERO), cp.unit());
        for k in 0..=1000 {
            let r = SHRUNK_R + (SMALL_R - SHRUNK_R) * f64::from(k) / 1000.0;
            let [low, high] = a_leader(r);
            let [a_at, rho_at] = shrink_names(r);
            let (low, high) = (apart(cp.p(low), centre), apart(cp.p(high), centre));
            // the leader is width 1.5; the circle's line width 4, the hole's ring width 3
            assert!(low - 0.0075 > HOLE + 0.015, "{r}: the hole");
            assert!(high + 0.0075 < apart(cp.p(a_at), centre) - A_HALF.1, "{r}: a");
            assert!(low < r * unit - 0.02 && high > r * unit + 0.02, "{r}: the circle");
            let rho_left = label_box(cp.p(rho_at), RHO_HALF)
                .iter()
                .map(|q| q[0])
                .fold(f64::INFINITY, f64::min);
            assert!(rho_left - centre[0] - 0.0075 > CLEAR, "{r}: ρ");
        }
    }
}
