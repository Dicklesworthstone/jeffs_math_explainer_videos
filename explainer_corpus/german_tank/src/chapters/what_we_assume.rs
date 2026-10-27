//! Chapter 04, "Saying it exactly": back on the friends' line from 1 to 84
//! with the badge in the corner, the question is pinned down: the count,
//! the true number of boxes, is unknown, and three values it might be come
//! up beside it (04_1). The friends' five are one handful; the one
//! assumption is the bin from before, and chapter 02's handful from the bin
//! lands as a row over the line with five more from the same draw, each as
//! likely as the friends' (04_2). A recipe turns a handful into an estimate
//! (04_3). Ours, the gap recipe: the sixty numbers from 1 to 60 under the
//! certain bracket, five of them the friends', the rest in the gap arcs,
//! sixty minus five (04_4); the same total for any five numbers, the recipe
//! in the largest number alone, and seventy-one for the friends, held to be
//! read (04_5); the prediction on the handful 1 to 5 (04_6); its estimate,
//! five, and why the recipe never guesses below the largest number it saw
//! (04_7); and why one handful can't judge a recipe, the rows coming back
//! (04_8).
//!
//! Highlights are halos: a separate rounded outline that comes and goes
//! around what the words point at, the thing itself untouched. Typeset text
//! is never scaled, indicated or recoloured in place (root saw duplicated
//! glyphs where another opening indicated text); a panel line that changes
//! is replaced, the old line fading as the new one comes in. Every number
//! on the screen is a fact. The halo, the replacement and the panel's
//! builders are shared with chapter 05.

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    BRACKET_W, LABEL_DY, LABEL_SIZE, STRIP_X0, STRIP_Y, TICK_W, bracket_points, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flash_lagged, line_style, pause, say, stroke, think, wait_until, x_at,
};
use super::friends_line::{
    ARC_H, ARC_W, BADGE_AT, FriendsLine, PANEL_SIZE, PANEL_X, PANEL_Y, RING_R, Start, THINK_AT, arc_points, at, badge,
    gap_spans, handfuls, is_handful, list, panel, ring, row_y, whole,
};
use crate::facts;
use crate::model::LINE_DOTS;
use crate::palette;

pub struct WhatWeAssume {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const COUNT: &str = "count = ?";
const HANDFUL: &str = "handful";
const RECIPE_WORD: &str = "recipe";
const ESTIMATE_WORD: &str = "estimate";
const ESTIMATOR: &str = "estimator";
const RECIPE: &str = "estimate = largest + average gap";
const NEVER: &str = "never below the largest";

/// A halo: how far it keeps clear of what it marks, its corner and its
/// stroke; how long it takes to come in (and to go), and how long it stays.
pub const HALO_PAD: f64 = 0.08;
pub const HALO_R: f64 = 0.1;
pub const HALO_W: f64 = 3.0;
pub const HALO_IN_S: f64 = 0.3;
pub const HALO_HOLD_S: f64 = 0.5;
/// The space between the pieces of a sum built piece by piece.
pub const SUM_GAP: f64 = 0.14;

/// The values the count might be (04_1), in a row after "count = ?".
const MIGHT_GAP: f64 = 0.4;
/// The rows of fair handfuls from the bin (04_2, 04_8): how many, and how
/// long one row takes to land.
const GHOST_ROWS: usize = 6;
const ROW_S: f64 = 0.3;
/// The recipe picture (04_3): the space either side of an arrow, an
/// arrow's length, stroke and head, the frame's padding round "recipe",
/// and "estimator" under it.
const FLOW_GAP: f64 = 0.15;
const FLOW_ARROW: f64 = 0.55;
const FLOW_ARROW_W: f64 = 3.0;
const FLOW_HEAD: f64 = 0.1;
const FLOW_SPREAD: f64 = 0.7;
const FLOW_PAD: [f64; 2] = [0.12, 0.08];
const FLOW_FRAME_W: f64 = 2.0;
const ESTIMATOR_DY: f64 = 0.42;
const ESTIMATOR_SIZE: f64 = 20.0;
/// An estimate's tick across the line, from just under it to above it,
/// and its number over the tick (as chapter 03 draws the first one).
const TICK_DOWN: f64 = 0.12;
const TICK_UP: f64 = 0.3;
const ESTIMATE_DY: f64 = 0.55;
/// "never below the largest", under the edge handful's label.
const NEVER_DY: f64 = 0.95;
/// The inspection hold after the friends' estimate (04_5), the beat after
/// the last line.
const HOLD_S: f64 = 2.5;
const END_BEAT: f64 = 0.6;
/// Which piece of a recipe sum is the gaps' total.
const TOTAL_PIECE: usize = 2;

/// What chapter 04 shows, from the facts: the friends' five (and so the
/// largest, the handful size and the line's first number), their gaps'
/// total and the estimate; the values the count might be; the bin and the
/// rows drawn from it; the edge handful, its gaps and its estimate.
#[derive(Clone, Debug)]
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    largest: u64,
    k: usize,
    unseen: u64,
    estimate: u64,
    /// What 04_1 says the count might be: sixty, seventy-one, five hundred.
    candidates: Vec<u64>,
    bin_dots: u64,
    bin_handful: Vec<u64>,
    ghosts: Vec<Vec<u64>>,
    edge: Vec<u64>,
    edge_gaps: Vec<u64>,
    edge_estimate: u64,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        numbers: list(&f, "G-F1.numbers")?,
        first: whole(&f, "G-F1.first")?,
        largest: whole(&f, "G-F1.largest")?,
        k: whole(&f, "G-F1.handful")? as usize,
        unseen: whole(&f, "G-F3.unseen")?,
        estimate: whole(&f, "G-F3.estimate")?,
        candidates: vec![
            whole(&f, "G-F1.largest")?,
            whole(&f, "G-F3.estimate")?,
            whole(&f, "G-F4.far_count")?,
        ],
        bin_dots: whole(&f, "G-F2.bin_dots")?,
        bin_handful: list(&f, "G-F2.bin_handful")?,
        ghosts: handfuls(&f, "G-F4.ghost_handfuls")?,
        edge: list(&f, "G-F4.edge_handful")?,
        edge_gaps: list(&f, "G-F4.edge_gaps")?,
        edge_estimate: whole(&f, "G-F4.edge_estimate")?,
    })
}

/// How many numbers of a sorted handful's gaps below its largest the line
/// shows unseen: the stretches the arcs cover, counted.
pub fn unseen_below(hand: &[u64]) -> u64 {
    gap_spans(hand).iter().map(|&(a, b)| b - a + 1).sum()
}

/// The picture only if the scene can draw it truthfully: the friends' five
/// are a handful on the line from 1, the largest last; their gaps hold the
/// largest minus five, which five divides, and the estimate is the largest
/// plus that share; every value the count might be is at least the largest,
/// in order; the rows are fair handfuls from the bin of chapter 02, its
/// own handful first; the edge handful is 1 to five, with five empty gaps,
/// and its estimate is its largest.
fn checked(pic: Picture) -> Result<Picture, String> {
    let k = pic.k as u64;
    if !is_handful(&pic.numbers, pic.k, LINE_DOTS) || pic.numbers.last() != Some(&pic.largest) || pic.first != 1 {
        return Err(format!(
            "the friends' {:?} must be {} numbers in order on the line from 1, the largest {}",
            pic.numbers, pic.k, pic.largest
        ));
    }
    if pic.unseen + k != pic.largest || unseen_below(&pic.numbers) != pic.unseen {
        return Err(format!(
            "the gaps' {} must be the unseen numbers below {}, the largest minus {}",
            pic.unseen, pic.largest, pic.k
        ));
    }
    if !pic.unseen.is_multiple_of(k) || pic.estimate != pic.largest + pic.unseen / k {
        return Err(format!(
            "the estimate {} must be {} plus {} over {}, exactly",
            pic.estimate, pic.largest, pic.unseen, pic.k
        ));
    }
    if pic.candidates.len() != 3 || pic.candidates[0] < pic.largest || pic.candidates.windows(2).any(|w| w[0] >= w[1]) {
        return Err(format!(
            "the count might be {:?}: three values in order, each at least {}",
            pic.candidates, pic.largest
        ));
    }
    if pic.bin_dots != LINE_DOTS
        || pic.ghosts.len() != GHOST_ROWS
        || pic.ghosts.first() != Some(&pic.bin_handful)
        || !pic.ghosts.iter().all(|g| is_handful(g, pic.k, pic.bin_dots))
    {
        return Err(format!(
            "the rows {:?} must be {GHOST_ROWS} handfuls of {} from the bin's {}, chapter 02's {:?} first",
            pic.ghosts, pic.k, pic.bin_dots, pic.bin_handful
        ));
    }
    let edge: Vec<u64> = (pic.first..=k).collect();
    if pic.edge != edge || pic.edge_gaps != vec![0; pic.k] || pic.edge_estimate != k {
        return Err(format!(
            "the edge handful {:?} must be 1 to {}, its gaps {:?} all empty, its estimate {} its largest",
            pic.edge, pic.k, pic.edge_gaps, pic.edge_estimate
        ));
    }
    Ok(pic)
}

// ------------------------------------------------------------------ words

/// A number shown as a word of a panel line is coloured by its role only
/// where it stands once and nowhere inside another number: in "5 − 5 = 0"
/// the largest and the handful size are both "5", so neither is coloured.
/// Names are coloured wherever they stand.
pub fn roles(text: &str, wanted: &[(String, Srgb)]) -> Vec<(String, Srgb)> {
    wanted
        .iter()
        .filter(|(key, _)| {
            !key.chars().all(|c| c.is_ascii_digit() || c == '.')
                || (text.matches(key.as_str()).count() == 1
                    && text
                        .split_whitespace()
                        .filter(|w| w.trim_end_matches([',', ':']) == key.as_str())
                        .count()
                        == 1)
        })
        .cloned()
        .collect()
}

/// The recipe in words (04_4), its names in their roles' colours.
fn recipe_colors() -> Vec<(String, Srgb)> {
    vec![
        ("estimate".to_owned(), palette::estimate()),
        ("largest".to_owned(), palette::seen()),
        ("average gap".to_owned(), palette::unseen()),
    ]
}

/// The recipe in the largest number alone (04_5, and 05_3 for a pair), for
/// handfuls of `k`.
pub fn formula(k: usize) -> String {
    format!("estimate = largest + (largest − {k}) ÷ {k}")
}

pub fn formula_colors() -> Vec<(String, Srgb)> {
    vec![
        ("estimate".to_owned(), palette::estimate()),
        ("largest".to_owned(), palette::seen()),
    ]
}

/// The gaps' total for a handful whose largest is `m` (04_4, 05_3), and
/// with its rule written first (04_5, 04_7).
pub fn gaps_line(m: u64, k: usize, total: u64, with_rule: bool) -> (String, Vec<(String, Srgb)>) {
    let text = if with_rule {
        format!("gaps: largest − {k} = {m} − {k} = {total}")
    } else {
        format!("gaps: {m} − {k} = {total}")
    };
    let wanted = [
        ("gaps".to_owned(), palette::unseen()),
        ("largest".to_owned(), palette::seen()),
        (m.to_string(), palette::seen()),
        (total.to_string(), palette::unseen()),
    ];
    let colors = roles(&text, &wanted);
    (text, colors)
}

/// The recipe for one handful, piece by piece (04_5, 04_7, 05_4): its
/// largest, "+", the gaps' total, "÷", the handful size, "=", the estimate
/// as it is shown, each in its role's colour (pieces, so two equal numbers
/// can differ in colour).
pub fn instance(m: u64, total: u64, k: usize, estimate: &str) -> Vec<(String, Srgb)> {
    vec![
        (m.to_string(), palette::seen()),
        ("+".to_owned(), Palette::ink()),
        (total.to_string(), palette::unseen()),
        ("÷".to_owned(), Palette::ink()),
        (k.to_string(), Palette::ink()),
        ("=".to_owned(), Palette::ink()),
        (estimate.to_owned(), palette::estimate()),
    ]
}

/// The edge handful's numbers as its label says them.
fn spelled(hand: &[u64]) -> String {
    hand.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

// ------------------------------------------------------- shared builders

/// The rounded rectangle a halo draws around boxes `bounds` (each a lower
/// left and an upper right corner), `HALO_PAD` clear of every one: its
/// centre, width and height; `None` around nothing.
pub fn halo_frame(bounds: &[([f64; 2], [f64; 2])]) -> Option<([f64; 2], f64, f64)> {
    let (first, rest) = bounds.split_first()?;
    let (mut lo, mut hi) = *first;
    for &(a, b) in rest {
        lo = [lo[0].min(a[0]), lo[1].min(a[1])];
        hi = [hi[0].max(b[0]), hi[1].max(b[1])];
    }
    Some((
        [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0],
        hi[0] - lo[0] + 2.0 * HALO_PAD,
        hi[1] - lo[1] + 2.0 * HALO_PAD,
    ))
}

/// A halo in `color` around everything in `mobs`. Detached.
pub fn halo(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb) -> fmn::Result<Mob> {
    let bounds: Vec<([f64; 2], [f64; 2])> = mobs
        .iter()
        .map(|&m| {
            let b = stage.get_bounding_box(m);
            ([b.min[0], b.min[1]], [b.max[0], b.max[1]])
        })
        .collect();
    let (c, w, h) = halo_frame(&bounds).ok_or_else(|| fail("a halo around nothing".to_owned()))?;
    let frame = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(HALO_R)
        .style(line_style(color, HALO_W, 1.0))
        .build()?
        .moved_to(v(c[0], c[1]));
    Ok(stage.arena_mut().add(frame))
}

/// Point at `mobs`: a halo comes in around them, stays a moment and goes.
pub fn glow(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb) -> fmn::Result<()> {
    let h = halo(stage, mobs, color)?;
    fade_in_all(stage, &[h], HALO_IN_S)?;
    pause(stage, HALO_HOLD_S)?;
    fade_out_all(stage, &[h], HALO_IN_S)
}

/// Replace `out` with `into`, together, over `rt` seconds: the old fade as
/// the new come in (updaters on `out` cleared first, so a live number
/// fades as it stands).
pub fn replace(stage: &mut Stage<'_>, out: &[Mob], into: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in out {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in into {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if anims.is_empty() {
        return Ok(());
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// The left edges of pieces `widths` wide, laid left to right from `x`,
/// `gap` apart.
pub fn row_lefts(widths: &[f64], x: f64, gap: f64) -> Vec<f64> {
    let mut left = x;
    widths
        .iter()
        .map(|&w| {
            let at = left;
            left += w + gap;
            at
        })
        .collect()
}

/// Lay `mobs` left to right from `x`, centred on height `y`, `gap` apart.
pub fn place_row(stage: &mut Stage<'_>, mobs: &[Mob], x: f64, y: f64, gap: f64) {
    let widths: Vec<f64> = mobs.iter().map(|&m| stage.get_width(m)).collect();
    for ((&m, left), w) in mobs.iter().zip(row_lefts(&widths, x, gap)).zip(&widths) {
        stage.move_to(m, v(left + w / 2.0, y), ORIGIN);
    }
}

/// A sum built piece by piece, each piece its own text in its own colour,
/// laid out from `x` on height `y`. Detached.
pub fn sum_texts(kit: &Kit, stage: &mut Stage<'_>, pieces: &[(String, Srgb)], x: f64, y: f64) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for (s, c) in pieces {
        mobs.push(kit.text(stage, s, PANEL_SIZE, *c, v(x, y))?);
    }
    place_row(stage, &mobs, x, y, SUM_GAP);
    Ok(mobs)
}

/// A line of words with its left edge at `left`, the substrings `colors`
/// in theirs, the rest ink. Detached.
pub fn words_left(
    kit: &Kit,
    stage: &mut Stage<'_>,
    text: &str,
    colors: &[(String, Srgb)],
    left: Vec3,
) -> fmn::Result<Mob> {
    let t2c: Vec<(&str, Srgb)> = colors.iter().map(|(s, c)| (s.as_str(), *c)).collect();
    let m = kit.text_t2c(stage, text, PANEL_SIZE, Palette::ink(), &t2c, left)?;
    stage.move_to(m, left, LEFT);
    Ok(m)
}

// ------------------------------------------------------- chapter builders

/// Where the recipe picture (04_3) puts its parts on panel line 1, from the
/// widths of its first two words (handful, recipe): the arrow into the
/// recipe's frame, the frame's centre, the arrow out of it and the
/// estimate's left edge.
#[derive(Clone, Copy, Debug)]
struct FlowAt {
    into: [f64; 2],
    recipe: f64,
    out: [f64; 2],
    estimate: f64,
}

fn flow_at(w: [f64; 2]) -> FlowAt {
    let into = [PANEL_X + w[0] + FLOW_GAP, PANEL_X + w[0] + FLOW_GAP + FLOW_ARROW];
    let frame = w[1] + 2.0 * FLOW_PAD[0];
    let recipe = into[1] + FLOW_GAP + frame / 2.0;
    let out = [
        recipe + frame / 2.0 + FLOW_GAP,
        recipe + frame / 2.0 + FLOW_GAP + FLOW_ARROW,
    ];
    let estimate = out[1] + FLOW_GAP;
    FlowAt {
        into,
        recipe,
        out,
        estimate,
    }
}

/// An arrow along height `y` from `x0` to `x1`, its head at `x1`.
fn arrow_points(x0: f64, x1: f64, y: f64) -> Vec<Vec3> {
    let h = FLOW_HEAD.min((x1 - x0) / 2.0);
    vec![
        v(x0, y),
        v(x1, y),
        v(x1 - h, y + FLOW_SPREAD * h),
        v(x1, y),
        v(x1 - h, y - FLOW_SPREAD * h),
    ]
}

/// The recipe picture's parts, in the order they come in, and the name
/// that comes under it.
struct Flow {
    all: Vec<Mob>,
    name: Mob,
}

/// The recipe picture (04_3): "handful" in gold, an arrow into "recipe" in
/// its frame, an arrow out to "estimate" in green; "estimator" in dim under
/// the frame. Detached.
fn flow(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Flow> {
    let y = PANEL_Y[1];
    let handful = kit.text(stage, HANDFUL, PANEL_SIZE, palette::seen(), panel(1))?;
    let recipe = kit.text(stage, RECIPE_WORD, PANEL_SIZE, Palette::ink(), panel(1))?;
    let estimate = kit.text(stage, ESTIMATE_WORD, PANEL_SIZE, palette::estimate(), panel(1))?;
    let w = [
        stage.get_width(handful),
        stage.get_width(recipe),
        stage.get_width(estimate),
    ];
    let h = stage.get_height(recipe);
    let spot = flow_at([w[0], w[1]]);
    stage.move_to(handful, v(PANEL_X + w[0] / 2.0, y), ORIGIN);
    stage.move_to(recipe, v(spot.recipe, y), ORIGIN);
    stage.move_to(estimate, v(spot.estimate + w[2] / 2.0, y), ORIGIN);
    let frame = Rectangle::new()
        .width(w[1] + 2.0 * FLOW_PAD[0])
        .height(h + 2.0 * FLOW_PAD[1])
        .corner_radius(0.08)
        .style(line_style(palette::outline(), FLOW_FRAME_W, 1.0))
        .build()?
        .moved_to(v(spot.recipe, y));
    let frame = stage.arena_mut().add(frame);
    let into = stroke(
        stage,
        arrow_points(spot.into[0], spot.into[1], y),
        line_style(Palette::ink(), FLOW_ARROW_W, 1.0),
    );
    let out = stroke(
        stage,
        arrow_points(spot.out[0], spot.out[1], y),
        line_style(Palette::ink(), FLOW_ARROW_W, 1.0),
    );
    let name = kit.text(
        stage,
        ESTIMATOR,
        ESTIMATOR_SIZE,
        Palette::dim(),
        v(spot.recipe, y - ESTIMATOR_DY),
    )?;
    Ok(Flow {
        all: vec![handful, into, frame, recipe, out, estimate],
        name,
    })
}

/// Rows of computed handfuls over the line shown to `end`, row `j` at
/// `row_y(j)`, one ring per number. Detached.
fn ghost_rows(stage: &mut Stage<'_>, hands: &[Vec<u64>], end: f64) -> Vec<Vec<Mob>> {
    let mut rows = Vec::new();
    for (j, hand) in hands.iter().enumerate() {
        let mut row = Vec::new();
        for &n in hand {
            row.push(ring(stage, v(x_at(n as f64, end), row_y(j))));
        }
        rows.push(row);
    }
    rows
}

/// A handful's gap arcs below its largest, on a line shown to `end`, as
/// chapter 03 drew them. Detached.
fn gap_arcs(stage: &mut Stage<'_>, hand: &[u64], end: f64) -> Vec<Mob> {
    let mut arcs = Vec::new();
    for (a, b) in gap_spans(hand) {
        arcs.push(stroke(
            stage,
            arc_points(x_at(a as f64 - 0.5, end), x_at(b as f64 + 0.5, end), STRIP_Y, ARC_H),
            line_style(palette::unseen(), ARC_W, 1.0),
        ));
    }
    arcs
}

/// An estimate's green tick at number `n` on a line shown to `end`, and
/// its number over it. Detached.
fn estimate_tick(kit: &Kit, stage: &mut Stage<'_>, n: u64, end: f64) -> fmn::Result<[Mob; 2]> {
    let x = x_at(n as f64, end);
    let tick = stroke(
        stage,
        vec![v(x, STRIP_Y - TICK_DOWN), v(x, STRIP_Y + TICK_UP)],
        line_style(palette::estimate(), TICK_W, 1.0),
    );
    let label = kit.text(
        stage,
        &n.to_string(),
        PANEL_SIZE,
        palette::estimate(),
        v(x, STRIP_Y + ESTIMATE_DY),
    )?;
    Ok([tick, label])
}

impl SceneConstruct for WhatWeAssume {
    fn name(&self) -> &str {
        "04_what_we_assume"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let opening = LINE_DOTS as f64;

        // the friends' line as chapter 03 left it, and the badge
        let strip = FriendsLine::build(
            &kit,
            stage,
            pic.first,
            LINE_DOTS,
            &pic.numbers,
            &[],
            Start {
                end: opening,
                fan: 0.0,
                rise: 1.0,
                widest: opening,
            },
        )?;
        let corner = badge(&kit, stage, at(BADGE_AT))?;
        let mut opened = strip.opening();
        opened.push(corner);
        fade_in_all(stage, &opened, 0.8)?;

        // 04_1: the count, unknown; then three values it might be, each as
        // it is said
        let line = say(&kit, stage, "04_1")?;
        line.wait_for(stage, "Call the true number")?;
        let count = words_left(&kit, stage, COUNT, &[], panel(0))?;
        fade_in_all(stage, &[count], 0.5)?;
        let mut might = Vec::new();
        for n in &pic.candidates {
            might.push(kit.text(stage, &n.to_string(), PANEL_SIZE, palette::outline(), panel(0))?);
        }
        let after_count = PANEL_X + stage.get_width(count) + MIGHT_GAP;
        place_row(stage, &might, after_count, PANEL_Y[0], MIGHT_GAP);
        for (&m, phrase) in might.iter().zip(["sixty", "seventy-one", "five hundred"]) {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &[m], 0.3)?;
        }

        // 04_2: one handful, a halo round the five; the assumption, a halo
        // round the badge; then the bin's handful from chapter 02 and five
        // more from the same draw, row by row, every row alike
        let line = say(&kit, stage, "04_2")?;
        fade_out_all(stage, &might, 0.4)?;
        line.wait_for(stage, "a single handful")?;
        glow(stage, &strip.bodies, palette::seen())?;
        line.wait_for(stage, "We assume the bin from before")?;
        glow(stage, &[corner], Palette::ink())?;
        line.wait_for(stage, "every handful of five")?;
        let rows = ghost_rows(stage, &pic.ghosts, opening);
        for row in &rows {
            fade_in_all(stage, row, ROW_S)?;
        }

        // 04_3: the rows go; a handful into a recipe, out comes an
        // estimate; then its name
        let line = say(&kit, stage, "04_3")?;
        let landed: Vec<Mob> = rows.iter().flatten().copied().collect();
        fade_out_all(stage, &landed, 0.4)?;
        line.wait_for(stage, "turns a handful into an estimate")?;
        let machine = flow(&kit, stage)?;
        fade_in_lagged(stage, &machine.all, 1.2, 0.3)?;
        line.wait_for(stage, "Statisticians call it an estimator")?;
        fade_in_all(stage, &[machine.name], 0.4)?;

        // 04_4: ours, in words, in place of the picture; the certain
        // bracket from 1 to 60; a halo round the friends' five; the arcs
        // over the rest, and their total
        let line = say(&kit, stage, "04_4")?;
        line.wait_for(stage, "Ours, the gap recipe")?;
        let recipe = words_left(&kit, stage, RECIPE, &recipe_colors(), panel(1))?;
        let mut pictured = machine.all.clone();
        pictured.push(machine.name);
        replace(stage, &pictured, &[recipe], 0.5)?;
        line.wait_for(stage, "Of the sixty boxes")?;
        let certain = stroke(
            stage,
            bracket_points(pic.first as f64, pic.largest as f64, opening),
            line_style(palette::certain(), BRACKET_W, 1.0),
        );
        fade_in_all(stage, &[certain], 0.5)?;
        line.wait_for(stage, "five are the friends'")?;
        glow(stage, &strip.bodies, palette::seen())?;
        line.wait_for(stage, "so the gaps hold")?;
        let arcs = gap_arcs(stage, &pic.numbers, opening);
        fade_in_lagged(stage, &arcs, 0.8, 0.3)?;
        let (text, colors) = gaps_line(pic.largest, pic.k, pic.unseen, false);
        let total = words_left(&kit, stage, &text, &colors, panel(2))?;
        fade_in_all(stage, &[total], 0.5)?;

        // 04_5: the total's rule written before it, the arcs rippling; the
        // recipe in the largest number alone; the friends' sum piece by
        // piece, and the green tick at seventy-one; held to be read
        let line = say(&kit, stage, "04_5")?;
        line.wait_for(stage, "With any five numbers")?;
        let (text, colors) = gaps_line(pic.largest, pic.k, pic.unseen, true);
        let ruled = words_left(&kit, stage, &text, &colors, panel(2))?;
        replace(stage, &[total], &[ruled], 0.5)?;
        flash_lagged(stage, &arcs, 0.8, 0.2)?;
        line.wait_for(stage, "So the estimate is")?;
        let alone = words_left(&kit, stage, &formula(pic.k), &formula_colors(), panel(1))?;
        replace(stage, &[recipe], &[alone], 0.5)?;
        line.wait_for(stage, "For our friends")?;
        let sum = sum_texts(
            &kit,
            stage,
            &instance(pic.largest, pic.unseen, pic.k, &pic.estimate.to_string()),
            PANEL_X,
            PANEL_Y[3],
        )?;
        fade_in_lagged(stage, &sum, 1.2, 0.3)?;
        line.wait_for(stage, "seventy-one")?;
        let [tick, label] = estimate_tick(&kit, stage, pic.estimate, opening)?;
        fade_in_all(stage, &[tick, label], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, HOLD_S)?;

        // 04_6: the prediction: the friends' arcs, bracket and tick go, the
        // rings of 1 to 5 come in on the line with their label; the
        // countdown
        let line = say(&kit, stage, "04_6")?;
        line.wait_for(stage, "one, two, three, four and five")?;
        let mut friends_only = arcs.clone();
        friends_only.extend([certain, tick, label]);
        fade_out_all(stage, &friends_only, 0.4)?;
        let mut rings = Vec::new();
        for &n in &pic.edge {
            rings.push(ring(stage, v(x_at(n as f64, opening), STRIP_Y)));
        }
        let edge_label = kit.text_left(
            stage,
            &spelled(&pic.edge),
            LABEL_SIZE,
            palette::seen(),
            v(x_at(pic.first as f64, opening) - RING_R, STRIP_Y - LABEL_DY),
        )?;
        fade_in_lagged(stage, &rings, 0.8, 0.3)?;
        fade_in_all(stage, &[edge_label], 0.3)?;
        let q = think(&kit, stage, at(THINK_AT))?;

        // 04_7: five: the edge handful's total and sum in place of the
        // friends', the green tick at five; a halo round its empty total;
        // then what is certain about every estimate
        let line = say(&kit, stage, "04_7")?;
        let edge_largest = pic.edge.last().copied().unwrap_or(pic.first);
        let edge_total = unseen_below(&pic.edge);
        let (text, colors) = gaps_line(edge_largest, pic.k, edge_total, true);
        let edge_ruled = words_left(&kit, stage, &text, &colors, panel(2))?;
        let edge_sum = sum_texts(
            &kit,
            stage,
            &instance(edge_largest, edge_total, pic.k, &pic.edge_estimate.to_string()),
            PANEL_X,
            PANEL_Y[3],
        )?;
        let [edge_tick, edge_mark] = estimate_tick(&kit, stage, pic.edge_estimate, opening)?;
        let mut out = vec![q, ruled];
        out.extend(&sum);
        let mut into = vec![edge_ruled, edge_tick, edge_mark];
        into.extend(&edge_sum);
        replace(stage, &out, &into, 0.5)?;
        line.wait_for(stage, "There are no gaps")?;
        glow(stage, &[edge_sum[TOTAL_PIECE]], palette::unseen())?;
        line.wait_for(stage, "never guesses below")?;
        let never = kit.text_left(
            stage,
            NEVER,
            PANEL_SIZE,
            palette::certain(),
            v(STRIP_X0, STRIP_Y - NEVER_DY),
        )?;
        fade_in_all(stage, &[never], 0.5)?;

        // 04_8: one handful, a halo round it; the count it can't check, a
        // halo round that; the panel clears and the rows from the bin come
        // back, all of them at once
        let line = say(&kit, stage, "04_8")?;
        line.wait_for(stage, "A single handful")?;
        glow(stage, &rings, palette::seen())?;
        line.wait_for(stage, "can't tell whether")?;
        glow(stage, &[count], Palette::ink())?;
        line.wait_for(stage, "We'll judge a recipe")?;
        let mut done = vec![alone, edge_ruled, edge_label, edge_tick, edge_mark, never];
        done.extend(&edge_sum);
        done.extend(&rings);
        fade_out_all(stage, &done, 0.5)?;
        line.wait_for(stage, "every handful the company")?;
        let again: Vec<Mob> = ghost_rows(stage, &pic.ghosts, opening).into_iter().flatten().collect();
        fade_in_lagged(stage, &again, 1.2, 0.05)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_H, BOX_RISE, BOX_W, DOT_R, SMALL, THINK_R};
    use crate::chapters::friends_line::{BADGE, BADGE_PAD, BADGE_SIZE, est_width};
    use crate::facts::SILLY;
    use crate::model::{BIN_STREAM, FRIENDS, Q, all_handfuls, doubling_recipe, drawn_handfuls, gap_recipe, gaps};

    /// The rings of a row are this far apart at least, outline to outline
    /// (as chapter 02's rows).
    const RING_GAP: f64 = 0.02;
    /// A panel line's half height at its size, and the space kept between
    /// lines and pieces that must not touch.
    const HALF_LINE: f64 = 0.16;
    const CLEAR: f64 = 0.1;

    fn good() -> Picture {
        picture().unwrap()
    }

    fn joined(pieces: &[(String, Srgb)]) -> String {
        pieces.iter().map(|(s, _)| s.as_str()).collect::<Vec<_>>().join(" ")
    }

    /// The scene draws the facts: the friends' five and the estimate the
    /// model's recipe gives them; the rows are the bin's own draw, its
    /// first row chapter 02's handful; the edge handful's estimate is the
    /// model's; the panel's sums read as the words say them.
    #[test]
    fn the_picture_is_the_facts() {
        let pic = good();
        assert_eq!(pic.numbers, FRIENDS);
        assert_eq!(gap_recipe(&FRIENDS).whole(), Some(pic.estimate as i128));
        assert_eq!(pic.ghosts, drawn_handfuls(BIN_STREAM, LINE_DOTS, pic.k, GHOST_ROWS));
        assert_eq!(pic.ghosts[0], facts::bin_handful());
        assert_eq!(gap_recipe(&pic.edge).whole(), Some(pic.edge_estimate as i128));
        assert_eq!(gaps(&pic.edge), pic.edge_gaps);
        assert_eq!(pic.candidates, [pic.largest, pic.estimate, facts::FAR]);
        assert_eq!(
            joined(&instance(pic.largest, pic.unseen, pic.k, &pic.estimate.to_string())),
            "60 + 55 ÷ 5 = 71"
        );
        assert_eq!(
            joined(&instance(5, unseen_below(&pic.edge), pic.k, "5")),
            "5 + 0 ÷ 5 = 5"
        );
        assert_eq!(
            instance(5, 0, 5, "5")[TOTAL_PIECE].0,
            "0",
            "the halo goes round the gaps' total"
        );
        assert_eq!(gaps_line(60, 5, 55, false).0, "gaps: 60 − 5 = 55");
        assert_eq!(gaps_line(60, 5, 55, true).0, "gaps: largest − 5 = 60 − 5 = 55");
        assert_eq!(gaps_line(5, 5, 0, true).0, "gaps: largest − 5 = 5 − 5 = 0");
        assert_eq!(formula(pic.k), "estimate = largest + (largest − 5) ÷ 5");
        assert_eq!(spelled(&pic.edge), "1, 2, 3, 4, 5");
    }

    /// A number is coloured by its role only where it can't be taken for
    /// another: in the friends' line both 60 and 55 are; in the edge
    /// handful's line the three 5s are not, and its 0 is. The negative
    /// controls: a number standing inside another (5 in 55), or twice.
    #[test]
    fn a_number_is_coloured_only_where_it_is_unambiguous() {
        let keys =
            |line: (String, Vec<(String, Srgb)>)| -> Vec<String> { line.1.into_iter().map(|(k, _)| k).collect() };
        assert_eq!(keys(gaps_line(60, 5, 55, true)), ["gaps", "largest", "60", "55"]);
        assert_eq!(keys(gaps_line(5, 5, 0, true)), ["gaps", "largest", "0"]);
        let gold = palette::seen();
        assert!(
            roles("55 − 5", &[("5".to_owned(), gold)]).is_empty(),
            "5 stands inside 55"
        );
        assert!(
            roles("6 + 60", &[("6".to_owned(), gold)]).is_empty(),
            "6 stands inside 60"
        );
        assert!(roles("3 + 3", &[("3".to_owned(), gold)]).is_empty(), "3 stands twice");
        assert_eq!(roles("3 + 4", &[("3".to_owned(), gold)]).len(), 1);
        assert_eq!(roles("largest + largest", &[("largest".to_owned(), gold)]).len(), 1);
    }

    /// The negative controls: each picture the words or the scene would
    /// contradict is refused, one wrong fact at a time, for its reason.
    #[test]
    fn a_picture_the_words_contradict_is_refused() {
        let pic = good();
        let refused = |p: Picture, why: &str| {
            let e = checked(p).expect_err(why);
            assert!(e.contains(why), "refused for {e:?}, not {why:?}");
        };
        assert!(checked(pic.clone()).is_ok());
        refused(
            Picture {
                numbers: vec![14, 27, 33, 60, 51],
                ..pic.clone()
            },
            "in order",
        );
        refused(
            Picture {
                unseen: pic.unseen - 1,
                ..pic.clone()
            },
            "the largest minus",
        );
        // 54 unseen with the largest 59 is the rule, but not the friends'
        refused(
            Picture {
                largest: 59,
                unseen: 54,
                ..pic.clone()
            },
            "in order",
        );
        refused(
            Picture {
                estimate: pic.estimate + 1,
                ..pic.clone()
            },
            "exactly",
        );
        // the rival recipe, double the average (chapter 10), is not this one
        let doubled = doubling_recipe(&FRIENDS).nearest() as u64;
        assert_ne!(doubled, pic.estimate);
        refused(
            Picture {
                estimate: doubled,
                ..pic.clone()
            },
            "exactly",
        );
        // a count below the largest is impossible, and the three go in order
        refused(
            Picture {
                candidates: vec![55, 71, 500],
                ..pic.clone()
            },
            "at least",
        );
        refused(
            Picture {
                candidates: vec![71, 60, 500],
                ..pic.clone()
            },
            "at least",
        );
        // the rows: chapter 02's handful first, from the bin of 84
        let mut swapped = pic.ghosts.clone();
        swapped.swap(0, 1);
        refused(
            Picture {
                ghosts: swapped,
                ..pic.clone()
            },
            "first",
        );
        let mut beyond = pic.ghosts.clone();
        beyond[2] = vec![8, 19, 24, 52, 90];
        refused(
            Picture {
                ghosts: beyond,
                ..pic.clone()
            },
            "first",
        );
        refused(
            Picture {
                ghosts: pic.ghosts[..5].to_vec(),
                ..pic.clone()
            },
            "first",
        );
        // the edge handful: 1 to five, empty gaps, its largest as estimate
        refused(
            Picture {
                edge: vec![2, 3, 4, 5, 6],
                ..pic.clone()
            },
            "edge handful",
        );
        refused(
            Picture {
                edge_estimate: 6,
                ..pic.clone()
            },
            "edge handful",
        );
        refused(
            Picture {
                edge_gaps: vec![0, 0, 0, 0, 1],
                ..pic
            },
            "edge handful",
        );
    }

    /// 04_7's "never": for every handful of every size up to six from
    /// every count up to twelve, the gaps below the largest hold the
    /// largest minus the handful size (04_5), and the estimate is at least
    /// the largest, equal to it only for the handful 1 to k. The negative
    /// control: the doubling recipe guesses below the largest for the
    /// strange handful of chapter 10.
    #[test]
    fn the_recipe_never_guesses_below_the_largest() {
        let mut equal = 0;
        for k in 1..=6usize {
            for n in k as u64..=12 {
                for h in all_handfuls(n, k) {
                    let m = *h.last().unwrap();
                    assert_eq!(gaps(&h).iter().sum::<u64>(), m - k as u64, "{h:?}");
                    assert_eq!(unseen_below(&h), m - k as u64, "{h:?}");
                    let e = gap_recipe(&h);
                    assert!(e >= Q::int(m as i128), "{h:?}: {e}");
                    let edge: Vec<u64> = (1..=k as u64).collect();
                    assert_eq!(e == Q::int(m as i128), h == edge, "{h:?}");
                    if h == edge {
                        equal += 1;
                    }
                }
            }
        }
        // the edge handful is in every count's list once
        assert_eq!(equal, (1..=6u64).map(|k| 12 - k + 1).sum::<u64>());
        let m = *SILLY.last().unwrap();
        assert!(doubling_recipe(&SILLY) < Q::int(m as i128));
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once,
    /// and a line's phrases come in the order the scene waits for them.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let waits: &[(&str, &[&str])] = &[
            (
                "04_1",
                &["Call the true number", "sixty", "seventy-one", "five hundred"],
            ),
            (
                "04_2",
                &[
                    "a single handful",
                    "We assume the bin from before",
                    "every handful of five",
                ],
            ),
            (
                "04_3",
                &["turns a handful into an estimate", "Statisticians call it an estimator"],
            ),
            (
                "04_4",
                &[
                    "Ours, the gap recipe",
                    "Of the sixty boxes",
                    "five are the friends'",
                    "so the gaps hold",
                ],
            ),
            (
                "04_5",
                &[
                    "With any five numbers",
                    "So the estimate is",
                    "For our friends",
                    "seventy-one",
                ],
            ),
            ("04_6", &["one, two, three, four and five"]),
            ("04_7", &["There are no gaps", "never guesses below"]),
            (
                "04_8",
                &[
                    "A single handful",
                    "can't tell whether",
                    "We'll judge a recipe",
                    "every handful the company",
                ],
            ),
        ];
        for &(id, phrases) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            let mut last = 0;
            for p in phrases {
                assert_eq!(text.matches(p).count(), 1, "{id}: {p:?}");
                let i = text.find(p).unwrap();
                assert!(i >= last, "{id}: {p:?} comes after the phrase before it");
                last = i;
            }
        }
    }

    /// A halo's frame keeps `HALO_PAD` clear of everything it marks, and
    /// there is none around nothing.
    #[test]
    fn a_halo_keeps_clear_of_what_it_marks() {
        let bounds = [([-1.0, -0.5], [0.0, 0.5]), ([2.0, -0.2], [3.0, 0.1])];
        let (c, w, h) = halo_frame(&bounds).unwrap();
        for (lo, hi) in bounds {
            assert!(c[0] - w / 2.0 <= lo[0] - HALO_PAD + 1e-12 && hi[0] + HALO_PAD <= c[0] + w / 2.0 + 1e-12);
            assert!(c[1] - h / 2.0 <= lo[1] - HALO_PAD + 1e-12 && hi[1] + HALO_PAD <= c[1] + h / 2.0 + 1e-12);
        }
        assert!((w - (4.0 + 2.0 * HALO_PAD)).abs() < 1e-12 && (h - (1.0 + 2.0 * HALO_PAD)).abs() < 1e-12);
        assert!(halo_frame(&[]).is_none());
    }

    /// The halos the chapter draws stay on the frame: round the badge at
    /// the top left, round the five boxes (over the dots), round the edge
    /// handful's rings at the line's left end.
    #[test]
    fn the_halos_stay_on_the_frame() {
        let end = LINE_DOTS as f64;
        let (safe_x, top) = (FRAME_X_RADIUS - 0.25, FRAME_Y_RADIUS - 0.1);
        let half_badge = [est_width(BADGE, BADGE_SIZE) / 2.0 + BADGE_PAD[0], 0.15 + BADGE_PAD[1]];
        let b = [BADGE_AT[0], BADGE_AT[1]];
        let (c, w, h) = halo_frame(&[(
            [b[0] - half_badge[0], b[1] - half_badge[1]],
            [b[0] + half_badge[0], b[1] + half_badge[1]],
        )])
        .unwrap();
        assert!(c[1] + h / 2.0 < top && c[0] - w / 2.0 > -safe_x, "{c:?} {w} {h}");
        let half_box = [SMALL * BOX_W / 2.0, SMALL * BOX_H / 2.0];
        let boxes: Vec<([f64; 2], [f64; 2])> = FRIENDS
            .iter()
            .map(|&n| {
                let (x, y) = (x_at(n as f64, end), STRIP_Y + BOX_RISE);
                ([x - half_box[0], y - half_box[1]], [x + half_box[0], y + half_box[1]])
            })
            .collect();
        let (c, _, h) = halo_frame(&boxes).unwrap();
        assert!(
            c[1] - h / 2.0 > STRIP_Y + DOT_R,
            "the halo round the boxes stays over the dots"
        );
        let rings: Vec<([f64; 2], [f64; 2])> = (1..=5u64)
            .map(|n| {
                let x = x_at(n as f64, end);
                ([x - RING_R, STRIP_Y - RING_R], [x + RING_R, STRIP_Y + RING_R])
            })
            .collect();
        let (c, w, _) = halo_frame(&rings).unwrap();
        assert!(c[0] - w / 2.0 > -safe_x);
    }

    /// The panel's lines, the values the count might be and the recipe
    /// picture stay in the title-safe area; "estimator" sits under the
    /// recipe's frame, over the next line's place; the countdown's ring
    /// keeps clear of the sum over it.
    #[test]
    fn the_panel_fits_the_frame() {
        let pic = good();
        let safe_x = FRAME_X_RADIUS - 0.25;
        let widths: Vec<f64> = pic
            .candidates
            .iter()
            .map(|n| est_width(&n.to_string(), PANEL_SIZE))
            .collect();
        let lefts = row_lefts(&widths, PANEL_X + est_width(COUNT, PANEL_SIZE) + MIGHT_GAP, MIGHT_GAP);
        assert!(lefts[2] + widths[2] < safe_x);
        for (w, l) in widths.windows(2).zip(lefts.windows(2)) {
            assert!(l[0] + w[0] + MIGHT_GAP <= l[1] + 1e-12);
        }
        let spot = flow_at([est_width(HANDFUL, PANEL_SIZE), est_width(RECIPE_WORD, PANEL_SIZE)]);
        assert!(
            spot.estimate + est_width(ESTIMATE_WORD, PANEL_SIZE) < safe_x,
            "{spot:?}"
        );
        assert!(spot.into[0] < spot.into[1] && spot.into[1] < spot.recipe && spot.recipe < spot.out[0]);
        assert!(spot.out[1] < spot.estimate);
        assert!(est_width(ESTIMATOR, ESTIMATOR_SIZE) / 2.0 < spot.recipe - PANEL_X);
        const {
            assert!(
                PANEL_Y[1] - ESTIMATOR_DY - 0.12 > PANEL_Y[3] + HALF_LINE,
                "estimator over line 3"
            )
        };
        for text in [
            RECIPE.to_owned(),
            formula(pic.k),
            gaps_line(pic.largest, pic.k, pic.unseen, true).0,
            gaps_line(5, pic.k, 0, true).0,
        ] {
            assert!(PANEL_X + est_width(&text, PANEL_SIZE) < safe_x, "{text}");
        }
        let sum = instance(pic.largest, pic.unseen, pic.k, &pic.estimate.to_string());
        let sum_w: f64 =
            sum.iter().map(|(s, _)| est_width(s, PANEL_SIZE)).sum::<f64>() + SUM_GAP * (sum.len() - 1) as f64;
        assert!(
            PANEL_X + sum_w < THINK_AT[0] - THINK_R - 0.3,
            "the sum left of the countdown"
        );
        const {
            assert!(
                PANEL_Y[3] - HALF_LINE > THINK_AT[1] + THINK_R,
                "the sum over the countdown"
            )
        };
        const { assert!(PANEL_Y[0] + HALF_LINE < FRAME_Y_RADIUS - 0.15) };
    }

    /// The rows of fair handfuls sit between the boxes' tops and the
    /// panel's last line, their rings apart; the edge handful's rings stay
    /// apart on the line, its label under them and "never below the
    /// largest" under that, both on the frame and left of the friends'
    /// boxes; the two estimates' numbers clear the largest box. The
    /// negative control: neighbouring numbers on a line to a thousand would
    /// put their rings on top of each other.
    #[test]
    fn the_rows_and_the_rings_keep_apart() {
        fn apart(hand: &[u64], end: f64) -> bool {
            hand.windows(2)
                .all(|w| x_at(w[1] as f64, end) - x_at(w[0] as f64, end) >= 2.0 * RING_R + RING_GAP)
        }
        let pic = good();
        let end = LINE_DOTS as f64;
        assert!(pic.ghosts.iter().all(|h| apart(h, end)));
        assert!(apart(&pic.edge, end));
        assert!(!apart(&[43, 44], 1000.0), "neighbours on the line to a thousand");
        let box_top = STRIP_Y + BOX_RISE + SMALL * BOX_H / 2.0;
        assert!(row_y(0) - RING_R > box_top + CLEAR);
        assert!(row_y(GHOST_ROWS - 1) + RING_R < PANEL_Y[3] - HALF_LINE - CLEAR);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let label_left = x_at(1.0, end) - RING_R;
        assert!(label_left > -safe_x && STRIP_X0 > -safe_x);
        const {
            assert!(
                STRIP_Y - LABEL_DY + 0.12 < STRIP_Y - RING_R - 0.05,
                "the label under the rings"
            )
        };
        const {
            assert!(
                STRIP_Y - NEVER_DY + HALF_LINE < STRIP_Y - LABEL_DY - 0.12,
                "never under the label"
            )
        };
        assert!(STRIP_X0 + est_width(NEVER, PANEL_SIZE) < safe_x);
        // the friends' estimate's number clears the largest box; the edge
        // handful's clears the first box
        let box_half = SMALL * BOX_W / 2.0;
        let seventy_one = x_at(pic.estimate as f64, end) - est_width(&pic.estimate.to_string(), PANEL_SIZE) / 2.0;
        assert!(seventy_one > x_at(pic.largest as f64, end) + box_half + CLEAR);
        let five = x_at(pic.edge_estimate as f64, end) + est_width(&pic.edge_estimate.to_string(), PANEL_SIZE) / 2.0;
        assert!(five < x_at(pic.numbers[0] as f64, end) - box_half - CLEAR);
    }
}
