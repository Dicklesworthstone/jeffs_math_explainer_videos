//! Chapter 06, Sprague's recentring: chapter 05's last picture goes, and so
//! does its interval. Chapter 05's square lived on [0, 1], at x = 1/2; now
//! x² is taken on all real numbers, at a = 2, where it is 4 (06_1).
//! Measured from the point, h = x − 2, the change is exactly
//! x² − 4 = h² + 4h (06_2), and on either side of 2 its size is at most
//! |h|² + 4|h| (06_3). The prediction: near 2 the curve is nearly straight,
//! with slope 4, so try δ = ε/4, one fortieth against one tenth; does every h
//! in that window pass (06_4)? No, but almost: h = 0.0249, inside it, moves x²
//! by 0.10022 (06_5). So both terms share the budget: adding 4 completes the
//! square, (|h| + 2)² < 4 + ε (06_6), and δ = √(4 + ε) − 2, positive for
//! every ε; every h with |h| < δ, on either side and h = 0 itself, keeps the
//! gap below ε (06_7). At one tenth δ ≈ 0.0248; the usual cap |h| < 1 keeps
//! |x + 2| < 5 and gives δ = min(1, ε/5) = 0.02 (06_8). Both work, and
//! Sprague's note solves the budget instead of capping it (06_9).
//!
//! Every number drawn comes from the facts registry; the windows are the
//! model's slope-only window and the continuation's two δ.

use explainer_kit::layout::TEXT_MARGIN;
use explainer_kit::{Facts, Frame2, Kit, Palette, rect, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, NUMBER_DROP, Plot, curve_color, fact, fade_in_all, fade_in_lagged, fade_out_all, fail, fails, fixed,
    flat, limit_color, line_style, moving, say, think_pause,
};
use super::hidden_search::end_picture;
use super::old_objection::{RIGHT_X, swap, tex_frac};
use super::shrinking_region::glow;
use crate::facts::facts;
use crate::model::{BASE, Frac, RECENTRE_AT, SPRAGUE_EPS, slope_window};

pub struct SpragueRecentring {
    pub kit: Kit,
}

const NEXT: &str = "next: a bound for any polynomial";

// what the pictures wait for, each in the line the test below holds it to
const REAL_AT: &str = "on all real numbers";
const FOUR_AT: &str = "where it is four";
const LET_AT: &str = "Let h be";
const EXPAND_AT: &str = "x squared minus four is";
const BOUND_AT: &str = "its size is at most";
const STRAIGHT_AT: &str = "nearly straight";
const TRY_AT: &str = "Try delta";
const TENTH_AT: &str = "against one tenth";
const ASK_AT: &str = "Does every h";
const ANSWER_AT: &str = "No, but almost";
const MOVES_AT: &str = "moves x squared by";
const BUDGET_AT: &str = "Budget both terms";
const SQUARE_AT: &str = "completes a square";
const DELTA_AT: &str = "So delta is the square root";
const POSITIVE_AT: &str = "positive for every epsilon";
const EVERY_AT: &str = "Every h within delta";
const VALUE_AT: &str = "about point zero two four eight";
const USUAL_AT: &str = "The usual way";
const SMALLER_AT: &str = "the smaller of one";
const WORK_AT: &str = "Both work";
const NOTE_AT: &str = "Sprague's note";
const NEXT_AT: &str = "Next, a bound";

// --- layout (scene units; the frame is 14.2 by 8)

/// The number line of 06_1: the real numbers from −1 to 3 across the plot's
/// width, at the top left, its ends opened a little past them.
const LINE_RANGE: (f64, f64) = (-1.0, 3.0);
const LINE_LEFT: f64 = -5.8;
const LINE_WIDTH: f64 = 4.8;
const LINE_Y: f64 = 3.3;
/// How far the line runs past −1 and 3, in its own units.
const LINE_PAST: f64 = 0.15;
/// The chevrons that open the line's two ends.
const CHEVRON: (f64, f64) = (0.15, 0.12);
/// The square near (2, 4), magnified: under the number line, as wide as
/// chapter 05's plot. The windows at 1/10 fill about half its width.
const ZOOM: Plot = Plot {
    frame: Frame2 {
        center: [-3.4, 0.05, 0.0],
        width: 4.8,
        height: 4.0,
        x: (1.95, 2.05),
        y: (3.75, 4.25),
    },
};
/// How far the zoom's axes run past its edges.
const AXIS_PAST: f64 = 0.3;
/// The caption over the zoom, saying what it magnifies.
const CAPTION_Y: f64 = 2.55;
/// The band's label (data), inside the zoom's empty top left.
const BAND_LABEL: (f64, f64) = (1.953, 4.16);
/// The windows' labels (data): over the curve near the top, and under it
/// near the foot.
const LABEL_TOP_Y: f64 = 4.21;
const LABEL_FOOT_Y: f64 = 3.79;
/// The ring around the first input where the slope-only window loses.
const MISS_R: f64 = 0.09;
/// The right column's rows, top to bottom: the old and new settings, h, the
/// bound, the slope-only window and its loss, the completed square, δ, the
/// guarantee, δ at 1/10, the capped window.
const ROW_Y: [f64; 11] = [3.4, 2.92, 2.3, 1.7, 1.1, 0.55, -0.05, -0.7, -1.3, -1.9, -2.5];
/// The note's citation, under the zoom, and "next:", centred under all.
const NOTE_Y: f64 = -2.95;
const NEXT_Y: f64 = -3.45;
/// The think ring, in the right column's empty lower half while the rows
/// above it stop at the slope-only window.
const RING_AT: Vec3 = [3.6, -1.3, 0.0];
const RING_R: f64 = 0.45;

/// Whether every pair of neighbouring rows is at least `gap` apart.
const fn rows_apart(gap: f64) -> bool {
    let mut i = 0;
    while i + 1 < ROW_Y.len() {
        if ROW_Y[i] - ROW_Y[i + 1] < gap {
            return false;
        }
        i += 1;
    }
    true
}

// the number line and its labels stay title-safe and clear of the caption;
// the caption clears the zoom's axes; the rows stay apart, title-safe, and
// clear of "next:"; the note sits between the zoom's numbers and "next:";
// the ring and its note stay under the slope-only row and inside the frame
const _: () = assert!(LINE_Y + CHEVRON.1 + 0.1 < 4.0 - TEXT_MARGIN);
const _: () = assert!(LINE_Y - NUMBER_DROP - 0.15 > CAPTION_Y + 0.15);
const _: () = assert!(CAPTION_Y - 0.15 > ZOOM.frame.center[1] + ZOOM.frame.height / 2.0 + AXIS_PAST);
const _: () = assert!(rows_apart(0.45) && ROW_Y[0] + 0.15 < 4.0 - TEXT_MARGIN);
const _: () = assert!(ROW_Y[10] - 0.2 > NEXT_Y + 0.2 && NEXT_Y - 0.2 > -4.0 + TEXT_MARGIN);
const _: () = assert!(NOTE_Y + 0.15 < ZOOM.frame.center[1] - ZOOM.frame.height / 2.0 - NUMBER_DROP - 0.15);
const _: () = assert!(NOTE_Y - 0.15 > NEXT_Y + 0.2);
const _: () = assert!(RING_AT[1] + RING_R < ROW_Y[4] - 0.3 && RING_AT[0] + 0.85 < 7.1 - TEXT_MARGIN);

/// The number line's place for the real number t.
fn on_line(t: f64) -> Vec3 {
    let (lo, hi) = LINE_RANGE;
    v(LINE_LEFT + (t - lo) / (hi - lo) * LINE_WIDTH, LINE_Y)
}

/// A fact that is a decimal or a whole number, as a float for drawing.
fn number(f: &Facts, key: &str) -> fmn::Result<f64> {
    fact(f, key)?
        .parse()
        .map_err(|_| fail(format!("{key} is not a decimal")))
}

/// A row of the right column, left-aligned, detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    kit.tex_left(stage, src, size, color, &[], v(RIGHT_X, y))
}

/// Where on a row to put a glow: `share` of the way along it.
fn along(stage: &Stage<'_>, m: Mob, x0: f64, y: f64, share: f64) -> Vec3 {
    v(x0 + stage.get_width(m) * share, y)
}

/// The number line with its ticks and numbers at 0, 1 and 2, detached.
fn number_line(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Vec<Mob>> {
    let dim = Palette::dim();
    let (lo, hi) = LINE_RANGE;
    let base = Line::new(on_line(lo - LINE_PAST), on_line(hi + LINE_PAST))
        .style(line_style(dim, 2.5))
        .build()?;
    let mut out = vec![stage.arena_mut().add(base)];
    for key in ["C1.domain.left", "C1.domain.right", "C6.a"] {
        let label = fact(f, key)?;
        let at = on_line(number(f, key)?);
        let tick = Line::new(v(at[0], at[1] - 0.09), v(at[0], at[1] + 0.09))
            .style(line_style(dim, 2.5))
            .build()?;
        out.push(stage.arena_mut().add(tick));
        out.push(kit.text(stage, &label, 22.0, dim, v(at[0], at[1] - NUMBER_DROP))?);
    }
    Ok(out)
}

/// A stretch of the number line highlighted, as the curve's domain, detached.
fn highlight(stage: &mut Stage<'_>, from: f64, to: f64) -> fmn::Result<Mob> {
    let l = Line::new(on_line(from), on_line(to))
        .style(line_style(curve_color(), 6.0))
        .build()?;
    Ok(stage.arena_mut().add(l))
}

/// The whole line highlighted, both ends opened by chevrons: every real
/// number, detached.
fn whole_line(stage: &mut Stage<'_>) -> fmn::Result<[Mob; 3]> {
    let (lo, hi) = LINE_RANGE;
    let line = highlight(stage, lo - LINE_PAST, hi + LINE_PAST)?;
    let (w, h) = CHEVRON;
    let mut chevron = |tip: Vec3, back: f64| {
        let c = polyline([v(tip[0] + back, tip[1] + h), tip, v(tip[0] + back, tip[1] - h)])
            .style(line_style(curve_color(), 6.0))
            .build();
        stage.arena_mut().add(c)
    };
    let left = chevron(on_line(lo - LINE_PAST), w);
    let right = chevron(on_line(hi + LINE_PAST), -w);
    Ok([line, left, right])
}

/// The point of the number line chosen for the square, detached.
fn line_dot(stage: &mut Stage<'_>, t: f64) -> Mob {
    let d = Dot::new()
        .point(on_line(t))
        .radius(0.07)
        .style(flat(fixed(), 1.0))
        .build();
    stage.arena_mut().add(d)
}

/// The zoom's axes, a tick on each at x = 2 and y = 4 with its number, and
/// the axes' letters, detached.
fn zoom_axes(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Vec<Mob>> {
    let (x0, x1) = ZOOM.frame.x;
    let (y0, y1) = ZOOM.frame.y;
    let (o, xe, ye) = (ZOOM.p(x0, y0), ZOOM.p(x1, y0), ZOOM.p(x0, y1));
    let (ta, tv) = (ZOOM.p(number(f, "C6.a")?, y0), ZOOM.p(x0, number(f, "C6.value")?));
    let dim = Palette::dim();
    let mut out = Vec::new();
    for (a, b) in [
        (v(o[0] - 0.2, o[1]), v(xe[0] + AXIS_PAST, o[1])),
        (v(o[0], o[1] - 0.2), v(o[0], ye[1] + AXIS_PAST)),
        (v(ta[0], o[1] - 0.09), v(ta[0], o[1] + 0.09)),
        (v(o[0] - 0.09, tv[1]), v(o[0] + 0.09, tv[1])),
    ] {
        let l = Line::new(a, b).style(line_style(dim, 2.5)).build()?;
        out.push(stage.arena_mut().add(l));
    }
    out.push(kit.text(stage, &fact(f, "C6.a")?, 24.0, dim, v(ta[0], o[1] - NUMBER_DROP))?);
    out.push(kit.text(stage, &fact(f, "C6.value")?, 24.0, dim, v(o[0] - 0.3, tv[1]))?);
    out.push(kit.tex(stage, "x", 30.0, dim, &[], v(xe[0] + AXIS_PAST + 0.22, o[1]))?);
    out.push(kit.tex(stage, "y", 30.0, dim, &[], v(o[0] - 0.3, ye[1] + AXIS_PAST))?);
    Ok(out)
}

/// The square across the zoom's x range, one smooth stroke, detached.
fn zoom_curve(stage: &mut Stage<'_>) -> Mob {
    const SAMPLES: u32 = 64;
    let (lo, hi) = ZOOM.frame.x;
    let pts: Vec<Vec3> = (0..=SAMPLES)
        .map(|k| {
            let x = lo + (hi - lo) * f64::from(k) / f64::from(SAMPLES);
            ZOOM.p(x, x * x)
        })
        .collect();
    let c = polyline(pts).style(line_style(curve_color(), BRIGHT_W)).build();
    stage.arena_mut().add(c)
}

/// The full-height window |x − 2| < d on the zoom, in `style`, detached.
fn zoom_window(stage: &mut Stage<'_>, d: f64, style: Style) -> fmn::Result<Mob> {
    let a = RECENTRE_AT as f64;
    let (lo, hi) = (ZOOM.p(a - d, ZOOM.frame.y.0), ZOOM.p(a + d, ZOOM.frame.y.1));
    let r = rect(lo, hi[0] - lo[0], hi[1] - lo[1], style)?;
    Ok(stage.arena_mut().add(r))
}

/// The right column's TeX and the note's citation, built from the facts.
struct Rows {
    was: String,
    now: String,
    h: String,
    expanded: String,
    bound: String,
    slope_only: String,
    miss: String,
    square: String,
    delta: String,
    guarantee: String,
    value: String,
    capped: String,
    note: String,
}

impl Rows {
    fn new(f: &Facts) -> fmn::Result<Self> {
        let n = fact(f, "C3.power")?;
        let a = fact(f, "C6.a")?;
        let value = fact(f, "C6.value")?;
        let slope = fact(f, "C6.slope")?;
        let half_slope = fact(f, "C6.half_slope")?;
        let added = fact(f, "C6.added")?;
        let zero = fact(f, "C6.h.at_a")?;
        let tenth = tex_frac(&fact(f, "S.eps")?);
        let cap = fact(f, "C6.cap")?;
        let factor = fact(f, "C6.cap.factor")?;
        Ok(Self {
            was: format!(
                r"\text{{was: }} x \in [{}, {}],\ a = {}",
                fact(f, "C1.domain.left")?,
                fact(f, "C1.domain.right")?,
                tex_frac(&fact(f, "C5.a")?)
            ),
            now: format!(r"\text{{now: }} x \in \mathbb{{R}},\ a = {a},\ a^{{{n}}} = {value}"),
            h: format!(r"h = x - {a}"),
            expanded: format!(r"x^{{{n}}} - {value} = h^{{{n}}} + {slope}h"),
            bound: format!(r"|x^{{{n}}} - {value}| = |h^{{{n}}} + {slope}h| \le |h|^{{{n}}} + {slope}|h|"),
            slope_only: format!(
                r"\text{{slope only: }} \delta = \tfrac{{\varepsilon}}{{{slope}}} = {}",
                tex_frac(&fact(f, "C6.naive.delta")?)
            ),
            miss: format!(
                r"h = {}:\ |x^{{{n}}} - {value}| \approx {} > {tenth}",
                fact(f, "C6.naive.h")?,
                fact(f, "C6.naive.change.5dp")?
            ),
            square: format!(
                r"|h|^{{{n}}} + {slope}|h| + {added} = (|h| + {half_slope})^{{{n}}} < {added} + \varepsilon"
            ),
            delta: format!(
                r"\delta = \sqrt{{{added} + \varepsilon}} - {half_slope} > {zero} \text{{ for every }} \varepsilon > {zero}"
            ),
            guarantee: format!(
                r"{zero} \le |h| < \delta \Rightarrow |x^{{{n}}} - {value}| \le |h|^{{{n}}} + {slope}|h| < \delta^{{{n}}} + {slope}\delta = \varepsilon"
            ),
            value: format!(
                r"\varepsilon = {tenth}:\ \delta \approx {}",
                fact(f, "S.delta_sprague.10dp")?
            ),
            capped: format!(
                r"|h| < {cap} \Rightarrow |x + {a}| < {factor}:\ \delta = \min({cap}, \tfrac{{\varepsilon}}{{{factor}}}) = {}",
                fact(f, "S.delta_simple.decimal")?
            ),
            note: format!(
                r"\text{{A. H. Sprague, A Note on }} \delta \text{{ and }} \varepsilon\text{{, Amer. Math. Monthly {} ({}), p. {}}}",
                fact(f, "C6.note.volume")?,
                fact(f, "C6.note.year")?,
                fact(f, "C6.note.page")?
            ),
        })
    }
}

impl SceneConstruct for SpragueRecentring {
    fn name(&self) -> &str {
        "06_sprague_recentring"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let end = end_picture(&kit, stage, &f)?;
        let rows = Rows::new(&f)?;
        let ink = Palette::ink();
        let a = number(&f, "C6.a")?;
        let value = number(&f, "C6.value")?;
        let eps = SPRAGUE_EPS.f64();
        let tolerance = Frac::whole(i64::from(SPRAGUE_EPS.num))
            .over(Frac::whole(i64::from(SPRAGUE_EPS.den)))
            .map_err(fail)?;
        let naive = slope_window(RECENTRE_AT, tolerance).map_err(fail)?.f64();
        let sprague = number(&f, "S.delta_sprague.10dp")?;
        let capped = number(&f, "S.delta_simple.decimal")?;

        // 06_1: chapter 05's picture goes; its domain [0, 1] and point 1/2 on
        // the number line, then every real number and the point 2; the square
        // near (2, 4), magnified
        let line = say(&kit, stage, "06_1")?;
        fade_out_all(stage, &end.clear, 0.6)?;
        let mut setting = number_line(&kit, stage, &f)?;
        let old = highlight(stage, number(&f, "C1.domain.left")?, number(&f, "C1.domain.right")?)?;
        let old_point = line_dot(stage, BASE.f64());
        let was = row(&kit, stage, &rows.was, 22.0, Palette::dim(), ROW_Y[0])?;
        setting.extend([old, old_point, was]);
        fade_in_lagged(stage, &setting, 1.0, 0.1)?;
        line.wait_for(stage, REAL_AT)?;
        let whole = whole_line(stage)?;
        let new_point = line_dot(stage, a);
        let now = row(&kit, stage, &rows.now, 24.0, ink, ROW_Y[1])?;
        let mut anims = swap(stage, &[old], &whole, 0.6)?;
        anims.extend(swap(stage, &[old_point], &[new_point], 0.6)?);
        anims.extend(swap(stage, &[], &[now], 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, FOUR_AT)?;
        let caption = kit.tex(
            stage,
            &format!(r"\text{{magnified near }} x = {}", fact(&f, "C6.a")?),
            20.0,
            Palette::dim(),
            &[],
            v(ZOOM.frame.center[0], CAPTION_Y),
        )?;
        let mut zoom = vec![caption];
        zoom.extend(zoom_axes(&kit, stage, &f)?);
        zoom.push(zoom_curve(stage));
        let point = Dot::new()
            .point(ZOOM.p(a, value))
            .radius(0.07)
            .style(flat(fixed(), 1.0))
            .build();
        let point = stage.arena_mut().add(point);
        zoom.push(point);
        fade_in_lagged(stage, &zoom, 1.0, 0.1)?;

        // 06_2: measure from the point: h = x − 2, and the change in powers
        // of h
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, LET_AT)?;
        let h_row = row(&kit, stage, &rows.h, 26.0, ink, ROW_Y[2])?;
        fade_in_all(stage, &[h_row], 0.5)?;
        line.wait_for(stage, EXPAND_AT)?;
        let expanded_x = RIGHT_X + stage.get_width(h_row) + 0.4;
        let expanded = kit.tex_left(stage, &rows.expanded, 26.0, ink, &[], v(expanded_x, ROW_Y[2]))?;
        fade_in_all(stage, &[expanded], 0.6)?;

        // 06_3: either sign of h, the size is at most |h|² + 4|h|
        let line = say(&kit, stage, "06_3")?;
        line.wait_for(stage, BOUND_AT)?;
        let bound = row(&kit, stage, &rows.bound, 26.0, ink, ROW_Y[3])?;
        fade_in_all(stage, &[bound], 0.6)?;

        // 06_4: the prediction: the slope alone says δ = ε/4, 1/40 against
        // 1/10; no input of it is shown losing until the ring has drained
        let line = say(&kit, stage, "06_4")?;
        line.wait_for(stage, STRAIGHT_AT)?;
        glow(&kit, stage, ZOOM.p(a, value), 0.35)?;
        line.wait_for(stage, TRY_AT)?;
        let (y0, y1) = ZOOM.frame.y;
        let naive_edges = [
            ZOOM.dashed(stage, (a - naive, y0), (a - naive, y1), moving())?,
            ZOOM.dashed(stage, (a + naive, y0), (a + naive, y1), moving())?,
        ];
        let naive_label = kit.tex(
            stage,
            &format!(r"\tfrac{{\varepsilon}}{{{}}}", fact(&f, "C6.slope")?),
            24.0,
            moving(),
            &[],
            ZOOM.p(a, LABEL_TOP_Y),
        )?;
        fade_in_all(stage, &[naive_edges[0], naive_edges[1], naive_label], 0.5)?;
        line.wait_for(stage, TENTH_AT)?;
        let band = rect(
            ZOOM.p(ZOOM.frame.x.0, value - eps),
            ZOOM.frame.width,
            ZOOM.p(a, value + eps)[1] - ZOOM.p(a, value - eps)[1],
            flat(limit_color(), 0.22),
        )?;
        let band = stage.arena_mut().add(band);
        let band_label = kit.tex_left(
            stage,
            &format!(r"\varepsilon = {}", tex_frac(&fact(&f, "S.eps")?)),
            22.0,
            limit_color(),
            &[],
            ZOOM.p(BAND_LABEL.0, BAND_LABEL.1),
        )?;
        let slope_only = row(&kit, stage, &rows.slope_only, 26.0, moving(), ROW_Y[4])?;
        fade_in_lagged(stage, &[band, band_label, slope_only], 0.8, 0.2)?;
        stage.scene_mut().bring_to_front(&[point])?;
        line.wait_for(stage, ASK_AT)?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), RING_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, RING_AT, RING_R)?;

        // 06_5: the answer: 0.0249, inside 1/40, leaves the band; ringed where
        // the curve crosses the band's top
        let line = say(&kit, stage, "06_5")?;
        fade_out_all(stage, &[ask], 0.3)?;
        line.wait_for(stage, ANSWER_AT)?;
        let miss_at = ZOOM.p(
            a + number(&f, "C6.naive.h")?,
            value + number(&f, "C6.naive.change.5dp")?,
        );
        let miss_ring = Circle::new()
            .radius(MISS_R)
            .arc_center(miss_at)
            .style(line_style(fails(), 3.5))
            .build();
        let miss_ring = stage.arena_mut().add(miss_ring);
        fade_in_all(stage, &[miss_ring], 0.3)?;
        glow(&kit, stage, miss_at, 0.25)?;
        line.wait_for(stage, MOVES_AT)?;
        let miss = row(&kit, stage, &rows.miss, 24.0, fails(), ROW_Y[5])?;
        fade_in_all(stage, &[miss], 0.5)?;

        // 06_6: budget both terms; adding 4 completes a square
        let line = say(&kit, stage, "06_6")?;
        line.wait_for(stage, BUDGET_AT)?;
        let at = along(stage, bound, RIGHT_X, ROW_Y[3], 0.82);
        glow(&kit, stage, at, 0.3)?;
        line.wait_for(stage, SQUARE_AT)?;
        let square = row(&kit, stage, &rows.square, 26.0, ink, ROW_Y[6])?;
        fade_in_all(stage, &[square], 0.6)?;

        // 06_7: δ = √(4 + ε) − 2 > 0 for every ε; every h inside it, either
        // sign or zero, passes; its window replaces the slope's
        let line = say(&kit, stage, "06_7")?;
        line.wait_for(stage, DELTA_AT)?;
        let delta = row(&kit, stage, &rows.delta, 30.0, moving(), ROW_Y[7])?;
        fade_in_all(stage, &[delta], 0.6)?;
        line.wait_for(stage, POSITIVE_AT)?;
        let at = along(stage, delta, RIGHT_X, ROW_Y[7], 0.5);
        glow(&kit, stage, at, 0.3)?;
        line.wait_for(stage, EVERY_AT)?;
        let guarantee = row(&kit, stage, &rows.guarantee, 24.0, ink, ROW_Y[8])?;
        let window = zoom_window(stage, sprague, flat(moving(), 0.22))?;
        let window_label = kit.tex(stage, r"\delta", 28.0, moving(), &[], ZOOM.p(a, LABEL_TOP_Y))?;
        let mut anims = swap(
            stage,
            &[naive_edges[0], naive_edges[1], naive_label],
            &[window, window_label],
            0.6,
        )?;
        anims.extend(swap(stage, &[], &[guarantee], 0.6)?);
        stage.play_prepared(anims)?;
        stage.scene_mut().bring_to_front(&[point, miss_ring])?;

        // 06_8: δ at 1/10; the usual cap |h| < 1 and its window, min(1, ε/5)
        let line = say(&kit, stage, "06_8")?;
        line.wait_for(stage, VALUE_AT)?;
        let value_row = row(&kit, stage, &rows.value, 26.0, moving(), ROW_Y[9])?;
        fade_in_all(stage, &[value_row], 0.5)?;
        line.wait_for(stage, USUAL_AT)?;
        let capped_row = row(&kit, stage, &rows.capped, 24.0, ink, ROW_Y[10])?;
        let capped_window = zoom_window(stage, capped, line_style(moving(), 2.5))?;
        let capped_label = kit.tex(
            stage,
            &format!(
                r"\min({}, \tfrac{{\varepsilon}}{{{}}})",
                fact(&f, "C6.cap")?,
                fact(&f, "C6.cap.factor")?
            ),
            22.0,
            moving(),
            &[],
            ZOOM.p(a, LABEL_FOOT_Y),
        )?;
        fade_in_lagged(stage, &[capped_row, capped_window, capped_label], 0.9, 0.2)?;
        line.wait_for(stage, SMALLER_AT)?;
        let at = along(stage, capped_row, RIGHT_X, ROW_Y[10], 0.8);
        glow(&kit, stage, at, 0.3)?;

        // 06_9: both work; the note solves the budget instead of capping it
        let line = say(&kit, stage, "06_9")?;
        line.wait_for(stage, WORK_AT)?;
        for (m, y) in [(value_row, ROW_Y[9]), (capped_row, ROW_Y[10])] {
            let at = along(stage, m, RIGHT_X, y, 0.9);
            glow(&kit, stage, at, 0.25)?;
        }
        line.wait_for(stage, NOTE_AT)?;
        let note = kit.tex_left(stage, &rows.note, 18.0, Palette::dim(), &[], v(LINE_LEFT, NOTE_Y))?;
        fade_in_all(stage, &[note], 0.6)?;
        line.wait_for(stage, NEXT_AT)?;
        let next = kit.text(stage, NEXT, 22.0, Palette::dim(), v(0.0, NEXT_Y))?;
        fade_in_all(stage, &[next], 0.6)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SQUARE_POLY, recenter, slope_window_miss};

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, in the order the pictures take them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("06_1", REAL_AT),
            ("06_1", FOUR_AT),
            ("06_2", LET_AT),
            ("06_2", EXPAND_AT),
            ("06_3", BOUND_AT),
            ("06_4", STRAIGHT_AT),
            ("06_4", TRY_AT),
            ("06_4", TENTH_AT),
            ("06_4", ASK_AT),
            ("06_5", ANSWER_AT),
            ("06_5", MOVES_AT),
            ("06_6", BUDGET_AT),
            ("06_6", SQUARE_AT),
            ("06_7", DELTA_AT),
            ("06_7", POSITIVE_AT),
            ("06_7", EVERY_AT),
            ("06_8", VALUE_AT),
            ("06_8", USUAL_AT),
            ("06_8", SMALLER_AT),
            ("06_9", WORK_AT),
            ("06_9", NOTE_AT),
            ("06_9", NEXT_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        for pair in waits.windows(2).filter(|w| w[0].0 == w[1].0) {
            let line = text(pair[0].0);
            assert!(line.find(pair[0].1) < line.find(pair[1].1), "{pair:?}");
        }
    }

    /// The setting changes before anything is computed; the question comes
    /// before its answer, and no line before the answer says where the
    /// slope's window loses; the answer opens 06_5. Sprague's δ is said to be
    /// positive for every tolerance and to hold on either side, and the
    /// usual cap comes after it.
    #[test]
    fn the_slope_window_is_asked_before_its_loss_is_shown() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        assert!(text("06_1").contains("on all real numbers"));
        assert!(text("06_4").ends_with('?') && text("06_4").contains("epsilon over four"));
        for id in ["06_1", "06_2", "06_3", "06_4"] {
            assert!(
                !text(id).contains("point zero two four nine"),
                "{id} says the answer early"
            );
            assert!(!text(id).contains("point one zero zero"), "{id} says the answer early");
        }
        assert!(text("06_5").starts_with(ANSWER_AT));
        assert!(text("06_7").contains(POSITIVE_AT) && text("06_7").contains("on either side"));
        assert!(text("06_8").contains("The usual way first keeps h within one"));
        assert!(text("06_9").starts_with(WORK_AT) && text("06_9").contains(NOTE_AT));
    }

    /// The zoom is centred on the point and holds the curve and every
    /// window it draws; the slope's window is wider than Sprague's δ, which
    /// is wider than the capped one; the loss is inside the slope's window
    /// and outside Sprague's; the labels clear the curve, and the band's
    /// label clears every window.
    #[test]
    fn the_zoom_holds_every_window() {
        let f = facts().unwrap();
        let num = |k: &str| number(&f, k).unwrap();
        let (a, value) = (num("C6.a"), num("C6.value"));
        let (x0, x1) = ZOOM.frame.x;
        let (y0, y1) = ZOOM.frame.y;
        assert!(((x0 + x1) / 2.0 - a).abs() < 1e-12 && x0 * x0 > y0 && x1 * x1 < y1);
        let eps = SPRAGUE_EPS.f64();
        assert!(value - eps > y0 && value + eps < y1);
        let tolerance = Frac::whole(1).over(Frac::whole(10)).unwrap();
        let naive = slope_window(RECENTRE_AT, tolerance).unwrap().f64();
        let (sprague, capped) = (num("S.delta_sprague.10dp"), num("S.delta_simple.decimal"));
        assert!(capped < sprague && sprague < naive && a + naive < x1);
        let (h, _) = slope_window_miss(RECENTRE_AT, tolerance, 4).unwrap();
        assert!(sprague < h.f64() && h.f64() < naive);
        // the band's label, "ε = 1/10" at 22 pt (under 0.8 scene units wide,
        // so under 0.017 of the zoom's x), sits above the band, ends left of
        // every window, and clears the curve under it
        let label_end = BAND_LABEL.0 + 0.8 / ZOOM.frame.width * (x1 - x0);
        assert!(BAND_LABEL.1 > value + eps && label_end < a - naive && label_end * label_end < BAND_LABEL.1 - 0.15);
        // the windows' labels stay inside the zoom and clear the curve within
        // their reach, about 0.02 either side of 2
        let near = |x: f64| x * x;
        assert!(
            LABEL_TOP_Y > near(a + 0.02) + 0.1
                && LABEL_FOOT_Y < near(a - 0.02) - 0.1
                && LABEL_TOP_Y < y1
                && LABEL_FOOT_Y > y0
        );
        assert_eq!(recenter(&SQUARE_POLY, RECENTRE_AT).unwrap()[0], value as i128);
    }
}
