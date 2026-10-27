//! Chapter 13, why prime: what went wrong on ten marks. Two marks that
//! aren't zero, 2 and 5, multiply to ten, which lands on zero: the red
//! ten-mark face turns five and five again, round to 0 (13_1). So five
//! times each mark lands on 0 or 5, the arrows crowding five onto each
//! (13_2), and chapter 08's straight line on ten marks, 9, 3, 7, 1 and 5,
//! leaves trustee 5's card two possible secrets, 0 and 5 (13_3). The same
//! rows on eleven marks, every mark under the same mark, and the viewer is
//! asked how many marks come out before an arrow is drawn (13_4): all
//! eleven, five further round the dial each step, each mark reached once,
//! so on eleven marks trustee 5's card fits every secret (13_5). Ten splits
//! as 2 × 5 and eleven doesn't: it is prime (13_6). Primes split a number
//! one way only, so on eleven marks a product lands on zero only when a
//! factor is zero (13_7); so multiplying by any of 1 to 10 only shuffles
//! the eleven marks, while on ten marks only 1, 3, 7 and 9 do (13_8). The
//! card says the counting holds on any prime number of marks above the
//! number of trustees and the secret, as Shamir (1979) takes it, and claims
//! nothing more (13_9).
//!
//! Every row, count and candidate is the model's (`Dial::times_row`,
//! `Dial::shuffles`, `View::possible`); the scene stops unless ten marks
//! crowd onto two and eleven reach every mark. The ten-mark failure is
//! shown as it is, one card leaving two secrets, and no composite dial is
//! said to give every secret away. Emphasis never restyles or scales
//! typeset text: rings, outlines and arrows of their own point at it, and
//! only shapes (ticks, cells) change color.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::a_ten_mark_dial::{read_on, text_after};
use super::board::{now, ring};
use super::draw::{
    FRAME_W, Rail, STRIP_LEFT, card, fade_in_all, fail, frame_at, panel, pause, recolorings, say, strip_right, swaps,
    think, tick_at, wait_until,
};
use super::envelope::{PAD, frame_halos, outline};
use super::two_keys::{Face, glow, glowing, leaving, moving, text_halo};
use super::two_shares::{DROP, disc_ring, text_right};
use crate::facts::SHAMIR_YEAR;
use crate::model::{Dial, LINE_SLOPE, MARKS, Recipe, SECRET, TEN, TEN_HOLDER, TRUSTEES, View, zero_product};
use crate::palette;

pub struct WhyPrime {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const FROM_TITLE: &str = "mark";
const COUNTS_TITLE: &str = "arrows";

// ------------------------------------------------------------------ layout

/// The red ten-mark face, upper left (13_1 to 13_5), its title over it and
/// the product right of it.
const FACE_X: f64 = -5.5;
const FACE_Y: f64 = 2.45;
const FACE_R: f64 = 0.9;
const FACE_TITLE_Y: f64 = 3.55;
const FACE_TITLE_SIZE: f64 = 20.0;
const PRODUCT_X: f64 = -4.25;
const PRODUCT_SIZE: f64 = 28.0;
/// The secrets one card fits, left (13_3 to 13_5): the title, a tick for
/// each secret, and its mark under it.
const FITS_LEFT: f64 = -6.45;
const FITS_TITLE_Y: f64 = 1.25;
const FITS_TITLE_SIZE: f64 = 18.0;
const FIT_X0: f64 = -6.3;
const FIT_DX: f64 = 0.46;
const FIT_Y: f64 = 0.85;
const FIT_W: f64 = 0.3;
const FIT_H: f64 = 0.25;
const FIT_RING: f64 = 0.22;
const FIT_LABEL_Y: f64 = 0.55;
const FIT_LABEL_SIZE: f64 = 16.0;
/// Over the rail (13_3): trustee 5's rule, and the line's name under it.
const RULE_Y: f64 = -0.35;
const RULE_SIZE: f64 = 20.0;
const LINE_NAME_Y: f64 = -0.8;
const LINE_NAME_SIZE: f64 = 18.0;
/// The think ring (13_4), left of the eleven-mark rows.
const THINK_X: f64 = -0.85;
const THINK_Y: f64 = 0.05;

/// One dial's times rows, right, on the strip's columns: the title, the
/// marks, where five times each lands, the arrows' counts and the reach.
#[derive(Clone, Copy, Debug)]
pub struct Rows {
    pub title: f64,
    pub from: f64,
    pub to: f64,
    pub counts: f64,
    pub reach: f64,
}

/// Ten marks over eleven, so a mark stands in the same place on both.
pub const TEN_ROWS: Rows = Rows {
    title: 3.6,
    from: 3.15,
    to: 2.05,
    counts: 1.6,
    reach: 1.22,
};
pub const ELEVEN_ROWS: Rows = Rows {
    title: 0.88,
    from: 0.43,
    to: -0.62,
    counts: -1.07,
    reach: -1.47,
};
const ROWS_TITLE_SIZE: f64 = 20.0;
const ROW_SIZE: f64 = 22.0;
const ROW_TITLE_SIZE: f64 = 20.0;
const ROW_TITLE_GAP: f64 = 0.15;
const ARROW_GAP: f64 = 0.2;
const ARROW_W: f64 = 2.0;
const HIT_R: f64 = 0.2;
const COUNT_SIZE: f64 = 18.0;
const COUNTS_TITLE_SIZE: f64 = 16.0;
const REACH_SIZE: f64 = 20.0;
const NOTE_SIZE: f64 = 18.0;
/// The split cards (13_6), the proof under them (13_7), and in 13_8 the
/// proof's last line moved to the top with the derivation under it, all
/// left-aligned upper left.
const LEFT: f64 = -6.6;
pub const CARD_Y: [f64; 4] = [3.4, 2.95, 2.45, 2.0];
pub const PROOF_Y: [f64; 5] = [1.45, 1.05, 0.7, 0.25, -0.2];
const LINE_SIZE: f64 = 20.0;
const LEMMA_Y: f64 = 3.4;
pub const SAME_Y: [f64; 4] = [2.9, 2.5, 2.1, 1.7];

/// A reach grid's rows (13_8), on the strip's columns: its title, the
/// marks, and the first multiplier's row, the others under it.
#[derive(Clone, Copy, Debug)]
pub struct GridRows {
    pub title: f64,
    pub labels: f64,
    pub first: f64,
}

pub const TEN_GRID: GridRows = GridRows {
    title: 3.6,
    labels: 3.27,
    first: 2.97,
};
pub const ELEVEN_GRID: GridRows = GridRows {
    title: 0.6,
    labels: 0.27,
    first: -0.03,
};
const GRID_DY: f64 = 0.25;
const CELL_W: f64 = 0.4;
const CELL_H: f64 = 0.18;
const GRID_TITLE_SIZE: f64 = 20.0;
const GRID_LABEL_SIZE: f64 = 16.0;
/// The holds: on the ten-mark arrows (13_2), on both dials' rows (13_5),
/// on the grids (13_8), and after the card is read.
const HOLD_ON_TEN: f64 = 1.5;
const HOLD_ON_BOTH: f64 = 2.0;
const HOLD_ON_GRIDS: f64 = 1.5;
const HOLD_AT_END: f64 = 0.6;

// -------------------------------------------------------------------- data

/// `k` times each mark, as (mark, where it lands): the arrows.
pub fn arrows(dial: Dial, k: u32) -> Vec<(u32, u32)> {
    dial.all().zip(dial.times_row(k)).collect()
}

/// How many arrows land on each mark.
pub fn hits(dial: Dial, k: u32) -> Vec<usize> {
    let row = dial.times_row(k);
    dial.all().map(|m| row.iter().filter(|&&y| y == m).count()).collect()
}

/// The marks some arrow lands on.
pub fn reached(dial: Dial, k: u32) -> Vec<u32> {
    let h = hits(dial, k);
    dial.all().filter(|&m| h[m as usize] > 0).collect()
}

/// "reaches 2 of 10 marks".
pub fn reach_line(dial: Dial, k: u32) -> String {
    format!("reaches {} of {} marks", reached(dial, k).len(), dial.marks())
}

/// The lower row's title: "5 × mark".
pub fn times_title(k: u32) -> String {
    format!("{k} \u{d7} mark")
}

/// Where `k` × `x` lands past the top: "15 → 4"; an error below it.
pub fn wrap_note(dial: Dial, k: u32, x: u32) -> Result<String, String> {
    let n = k * x;
    if n < dial.marks() {
        return Err(format!("{k} \u{d7} {x} = {n} does not pass the top"));
    }
    Ok(format!("{n} \u{2192} {}", dial.mul(k, x)))
}

/// The secrets trustee `x`'s card `share` fits on `dial`, for a line's
/// one fair spin.
pub fn fitting(dial: Dial, x: u32, share: u32) -> Vec<u32> {
    View::Shares {
        points: vec![(x, share)],
        spins: 1,
    }
    .possible(dial)
}

/// The secrets' title: "secrets that fit trustee 5's 25 → 5, on 10 marks".
pub fn fits_title(x: u32, height: i64, dial: Dial) -> String {
    format!(
        "secrets that fit trustee {x}'s {height} \u{2192} {}, on {} marks",
        dial.wrap(height),
        dial.marks()
    )
}

/// The line's name over the rail: "the line 5 + 4 × position, on 10 marks".
pub fn line_name(dial: Dial) -> String {
    format!(
        "the line {SECRET} + {LINE_SLOPE} \u{d7} position, on {} marks",
        dial.marks()
    )
}

/// Trustee `x`'s rule on the line: "trustee 5: secret + 5 × spin".
pub fn holder_rule(x: u32) -> String {
    format!("trustee {x}: secret + {x} \u{d7} spin")
}

/// Two nonzero marks' product and where it lands: "2 × 5 = 10", "→ 0".
pub fn product_pieces(dial: Dial, (a, b): (u32, u32)) -> [String; 2] {
    [
        format!("{a} \u{d7} {b} = {}", a * b),
        format!("\u{2192} {}", dial.mul(a, b)),
    ]
}

/// "10 = 2 × 5".
pub fn split_line((a, b): (u32, u32)) -> String {
    format!("{} = {a} \u{d7} {b}", a * b)
}

/// The same product on `dial`: "on 11 marks, 2 × 5 = 10 lands on 10".
pub fn lands_line(dial: Dial, (a, b): (u32, u32)) -> String {
    format!(
        "on {} marks, {a} \u{d7} {b} = {} lands on {}",
        dial.marks(),
        a * b,
        dial.mul(a, b)
    )
}

/// "11 = 1 × 11: no two numbers from 2 to 10 multiply to 11"; an error
/// for a number of marks with other factors.
pub fn prime_line(dial: Dial) -> Result<String, String> {
    let p = dial.marks();
    let factors: Vec<u32> = (1..=p).filter(|&d| p.is_multiple_of(d)).collect();
    if factors != [1, p] {
        return Err(format!("{p} has the factors {factors:?}"));
    }
    Ok(format!(
        "{p} = 1 \u{d7} {p}: no two numbers from 2 to {} multiply to {p}",
        p - 1
    ))
}

/// 13_7's proof, line by line, its second sentence on two lines; only for
/// a prime number of marks.
pub fn proof_lines(dial: Dial) -> Result<[String; 5], String> {
    prime_line(dial)?;
    let p = dial.marks();
    Ok([
        format!("a \u{d7} b lands on 0 exactly when a \u{d7} b is a multiple of {p}"),
        format!("each number splits into primes one way only, so {p} is"),
        "among a \u{d7} b's primes only if it is among a's or b's".to_owned(),
        format!("a and b from 1 to {}: no {p} among their primes", p - 1),
        format!("so on {p} marks, a \u{d7} b lands on 0 only if a or b is 0"),
    ])
}

/// 13_8's derivation for the multiplier `k`.
pub fn same_mark_lines(k: u32) -> [String; 4] {
    [
        format!("{k} \u{d7} u and {k} \u{d7} w land on the same mark"),
        format!("so {k} \u{d7} (u \u{2212} w) lands on 0"),
        format!("{k} isn't 0, so u \u{2212} w lands on 0:"),
        "u and w are the same mark".to_owned(),
    ]
}

/// For each multiplier 1 to marks − 1, which marks it reaches.
pub fn reach_rows(dial: Dial) -> Vec<Vec<bool>> {
    (1..dial.marks())
        .map(|k| {
            let row = dial.times_row(k);
            dial.all().map(|m| row.contains(&m)).collect()
        })
        .collect()
}

/// A reach grid's title: "on 11 marks, the marks reached by × 1 to × 10".
pub fn grid_title(dial: Dial) -> String {
    let m = dial.marks();
    format!("on {m} marks, the marks reached by \u{d7} 1 to \u{d7} {}", m - 1)
}

/// 13_9's precisely card: the counting on any prime above the number of
/// trustees and the secret, `primes` the first few here, and Shamir's
/// paper, which takes it so; nothing about security beyond the counting.
pub fn card_text(primes: &[u32]) -> String {
    let list: Vec<String> = primes.iter().map(ToString::to_string).collect();
    format!(
        "the counting holds on any prime number of marks above both the number of trustees and \
         the secret (here: {} and on; Shamir {SHAMIR_YEAR})",
        list.join(", ")
    )
}

// ---------------------------------------------------------------- drawing

/// Where secret `s`'s tick stands, left.
fn fit_at(s: u32) -> Vec3 {
    v(FIT_X0 + FIT_DX * f64::from(s), FIT_Y)
}

/// The rail's left edge, where the words over it start.
fn rail_left() -> f64 {
    frame_at(TRUSTEES[0])[0] - FRAME_W / 2.0
}

/// One card's secrets, left: its title, and a faint tick for each secret
/// with its mark under it. Detached.
struct Fits {
    title: Mob,
    ticks: Vec<Mob>,
    labels: Vec<Mob>,
}

impl Fits {
    fn build(kit: &Kit, stage: &mut Stage<'_>, title: &str, marks: u32) -> fmn::Result<Self> {
        let title = kit.text_left(
            stage,
            title,
            FITS_TITLE_SIZE,
            Palette::ink(),
            v(FITS_LEFT, FITS_TITLE_Y),
        )?;
        let (mut ticks, mut labels) = (Vec::new(), Vec::new());
        for s in 0..marks {
            ticks.push(panel(stage, palette::ghost(), 1.0, fit_at(s), FIT_W, FIT_H)?);
            labels.push(kit.text(
                stage,
                &s.to_string(),
                FIT_LABEL_SIZE,
                Palette::ink(),
                v(fit_at(s)[0], FIT_LABEL_Y),
            )?);
        }
        Ok(Self { title, ticks, labels })
    }

    fn parts(&self) -> Vec<Mob> {
        let mut all = vec![self.title];
        all.extend(&self.ticks);
        all.extend(&self.labels);
        all
    }

    /// The ticks of `lit` turning gold, as recolorings.
    fn lighting(&self, lit: &[u32]) -> Vec<(Mob, Srgb)> {
        lit.iter()
            .map(|&s| (self.ticks[s as usize], palette::secret()))
            .collect()
    }
}

/// One dial's times rows, right: its title, the marks over where five
/// times each lands, with their titles left of them, and the arrows
/// between, each with a ring on the mark it lands on if it is the first
/// to land there. Detached.
struct Times {
    rows: Rows,
    marks: u32,
    title: Mob,
    from_title: Mob,
    to_title: Mob,
    numbers: Vec<Mob>,
    arrows: Vec<(Mob, Option<Mob>)>,
}

impl Times {
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        dial: Dial,
        rows: Rows,
        title_color: Srgb,
        color: Srgb,
    ) -> fmn::Result<Self> {
        let marks = dial.marks();
        let title = kit.text_left(
            stage,
            &format!("on {marks} marks"),
            ROWS_TITLE_SIZE,
            title_color,
            v(STRIP_LEFT, rows.title),
        )?;
        let right = STRIP_LEFT - ROW_TITLE_GAP;
        let from_title = text_right(
            kit,
            stage,
            FROM_TITLE,
            ROW_TITLE_SIZE,
            palette::spin(),
            v(right, rows.from),
        )?;
        let to_title = text_right(
            kit,
            stage,
            &times_title(TEN_HOLDER),
            ROW_TITLE_SIZE,
            Palette::dim(),
            v(right, rows.to),
        )?;
        let mut numbers = Vec::with_capacity(2 * marks as usize);
        for k in dial.all() {
            let x = tick_at(k)[0];
            numbers.push(kit.text(stage, &k.to_string(), ROW_SIZE, palette::spin(), v(x, rows.from))?);
            numbers.push(kit.text(stage, &k.to_string(), ROW_SIZE, Palette::ink(), v(x, rows.to))?);
        }
        let mut landed = Vec::new();
        let mut drawn = Vec::with_capacity(marks as usize);
        for (a, d) in arrows(dial, TEN_HOLDER) {
            let start = v(tick_at(a)[0], rows.from - ARROW_GAP);
            let end = v(tick_at(d)[0], rows.to + ARROW_GAP);
            let arrow = Arrow::new(start, end)
                .buff(0.0)
                .thickness(ARROW_W)
                .color(color)
                .build()?;
            let arrow = stage.arena_mut().add(arrow);
            let hit = if landed.contains(&d) {
                None
            } else {
                landed.push(d);
                Some(ring(stage, v(tick_at(d)[0], rows.to), HIT_R, color))
            };
            drawn.push((arrow, hit));
        }
        Ok(Self {
            rows,
            marks,
            title,
            from_title,
            to_title,
            numbers,
            arrows: drawn,
        })
    }

    /// The title, the two rows and their titles.
    fn shown(&self) -> Vec<Mob> {
        let mut all = vec![self.title, self.from_title, self.to_title];
        all.extend(&self.numbers);
        all
    }

    /// The arrows and their rings.
    fn drawn(&self) -> Vec<Mob> {
        self.arrows
            .iter()
            .flat_map(|&(a, hit)| std::iter::once(a).chain(hit))
            .collect()
    }

    /// The arrows from the marks `from`, in that order.
    fn arrows_from_marks(&self, from: &[u32]) -> Vec<(Mob, Option<Mob>)> {
        from.iter().map(|&a| self.arrows[a as usize]).collect()
    }

    /// An outline round the lower row's mark `m`. Detached.
    fn mark_outline(&self, stage: &mut Stage<'_>, m: u32, color: Srgb) -> fmn::Result<Mob> {
        let side = 2.0 * (HIT_R + PAD);
        outline(stage, v(tick_at(m)[0], self.rows.to), side, side, color)
    }

    /// An outline round the whole lower row. Detached.
    fn row_outline(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        let (x0, x1) = (tick_at(0)[0], tick_at(self.marks - 1)[0]);
        outline(
            stage,
            v((x0 + x1) / 2.0, self.rows.to),
            x1 - x0 + 2.0 * (HIT_R + PAD),
            2.0 * (HIT_R + PAD),
            color,
        )
    }

    /// Under the lower row, how many arrows each mark gets, the ones that
    /// get some in `color`, and their title. Detached.
    fn counts(&self, kit: &Kit, stage: &mut Stage<'_>, hits: &[usize], color: Srgb) -> fmn::Result<Vec<Mob>> {
        let mut all = vec![text_right(
            kit,
            stage,
            COUNTS_TITLE,
            COUNTS_TITLE_SIZE,
            Palette::dim(),
            v(STRIP_LEFT - ROW_TITLE_GAP, self.rows.counts),
        )?];
        for (m, &h) in (0..).zip(hits) {
            let c = if h > 0 { color } else { Palette::dim() };
            all.push(kit.text(stage, &h.to_string(), COUNT_SIZE, c, v(tick_at(m)[0], self.rows.counts))?);
        }
        Ok(all)
    }

    /// The reach line under the counts, left. Detached.
    fn reach(&self, kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb) -> fmn::Result<Mob> {
        kit.text_left(stage, text, REACH_SIZE, color, v(STRIP_LEFT, self.rows.reach))
    }

    /// A grey note on the reach line's row, right. Detached.
    fn note(&self, kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
        text_right(
            kit,
            stage,
            text,
            NOTE_SIZE,
            Palette::dim(),
            v(strip_right(self.marks), self.rows.reach),
        )
    }
}

/// Arrows growing from their marks one after another, each with its ring
/// if it has one, the whole taking `rt`.
fn arrows_in(stage: &mut Stage<'_>, steps: &[(Mob, Option<Mob>)], rt: f64) -> fmn::Result<()> {
    if steps.is_empty() {
        return Ok(());
    }
    let mut each = Vec::with_capacity(steps.len());
    for &(arrow, hit) in steps {
        let a = fmn::animation::grow_arrow(stage.arena_mut(), arrow)?;
        let mut both = vec![stage.prepare(a)?];
        if let Some(h) = hit {
            both.extend(swaps(stage, &[], &[h], 1.0)?);
        }
        let g = AnimationGroup::new(stage.arena_mut(), both)?;
        each.push(stage.prepare(g)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.5)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// One dial's reach grid, right: its title and marks, and for each
/// multiplier its title and a cell at every mark, lit where it reaches,
/// `gap` where it doesn't. Detached.
struct ReachGrid {
    layout: GridRows,
    marks: u32,
    head: Vec<Mob>,
    rows: Vec<Vec<Mob>>,
}

impl ReachGrid {
    fn build(kit: &Kit, stage: &mut Stage<'_>, dial: Dial, layout: GridRows, gap: Srgb) -> fmn::Result<Self> {
        let marks = dial.marks();
        let mut head = vec![kit.text_left(
            stage,
            &grid_title(dial),
            GRID_TITLE_SIZE,
            Palette::ink(),
            v(STRIP_LEFT, layout.title),
        )?];
        for m in dial.all() {
            head.push(kit.text(
                stage,
                &m.to_string(),
                GRID_LABEL_SIZE,
                Palette::ink(),
                v(tick_at(m)[0], layout.labels),
            )?);
        }
        let mut rows = Vec::with_capacity(marks as usize - 1);
        for (i, (k, lit)) in (1u32..).zip(reach_rows(dial)).enumerate() {
            let y = grid_y(layout, i);
            let mut row = vec![text_right(
                kit,
                stage,
                &format!("\u{d7} {k}"),
                GRID_LABEL_SIZE,
                Palette::dim(),
                v(STRIP_LEFT - ROW_TITLE_GAP, y),
            )?];
            for (m, &on) in (0..).zip(&lit) {
                let (color, o) = if on { (palette::spin(), 0.9) } else { (gap, 0.45) };
                row.push(panel(stage, color, o, v(tick_at(m)[0], y), CELL_W, CELL_H)?);
            }
            rows.push(row);
        }
        Ok(Self {
            layout,
            marks,
            head,
            rows,
        })
    }

    /// The title and marks, then the rows one after another, the rows
    /// taking `rt`.
    fn appear(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &self.head, 0.4)?;
        let mut each = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let fades = swaps(stage, &[], row, 1.0)?;
            let g = AnimationGroup::new(stage.arena_mut(), fades)?;
            each.push(stage.prepare(g)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.3)?.with_run_time(rt);
        stage.play(g)?;
        Ok(())
    }

    /// An outline round rows `from` to `to` (0-based) of cells. Detached.
    fn outline_rows(&self, stage: &mut Stage<'_>, from: usize, to: usize, color: Srgb) -> fmn::Result<Mob> {
        let (x0, x1) = (tick_at(0)[0], tick_at(self.marks - 1)[0]);
        let (top, bottom) = (grid_y(self.layout, from), grid_y(self.layout, to));
        outline(
            stage,
            v((x0 + x1) / 2.0, (top + bottom) / 2.0),
            x1 - x0 + CELL_W + 2.0 * PAD,
            top - bottom + CELL_H + 2.0 * PAD,
            color,
        )
    }
}

/// Where row `i` (0-based: multiplier i + 1) of a reach grid stands.
fn grid_y(layout: GridRows, i: usize) -> f64 {
    layout.first - GRID_DY * i as f64
}

/// Lines `lines` of `texts`, each at its `ys`, left-aligned in its
/// `colors`, coming in together; returns them, and when a viewer still
/// reading until `busy` has read them too, one after another.
fn write_lines(
    kit: &Kit,
    stage: &mut Stage<'_>,
    texts: &[String],
    ys: &[f64],
    colors: &[Srgb],
    lines: std::ops::Range<usize>,
    busy: f64,
) -> fmn::Result<(Vec<Mob>, f64)> {
    let mut new = Vec::with_capacity(lines.len());
    for j in lines.clone() {
        new.push(kit.text_left(stage, &texts[j], LINE_SIZE, colors[j], v(LEFT, ys[j]))?);
    }
    fade_in_all(stage, &new, 0.5)?;
    let mut read = busy;
    for j in lines {
        read = read_on(stage, read, &texts[j]);
    }
    Ok((new, read))
}

impl SceneConstruct for WhyPrime {
    fn name(&self) -> &str {
        "13_why_prime"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let ten = Dial::new(TEN);
        let k = TEN_HOLDER;
        // ten's two nonzero marks with a zero product, and none on eleven
        let pair = zero_product(ten).ok_or_else(|| fail("ten marks: no zero product".into()))?;
        if let Some(p) = zero_product(dial) {
            return Err(fail(format!("eleven marks: {p:?} multiply to zero")));
        }
        let product = product_pieces(ten, pair);
        // the rows: ten crowd onto two marks, eleven reach every mark once
        let (ten_hits, eleven_hits) = (hits(ten, k), hits(dial, k));
        let ten_reached = reached(ten, k);
        if ten_reached.len() != 2 || eleven_hits.iter().any(|&h| h != 1) {
            return Err(fail(format!("times {k}: {ten_hits:?} and {eleven_hits:?}")));
        }
        // the line on ten marks, and what trustee 5's card fits on each dial
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let line_cards = line.deal(ten, &TRUSTEES);
        let height = line.height(k);
        let ten_found = fitting(ten, k, line.share(ten, k));
        let eleven_found = fitting(dial, k, line.share(dial, k));
        if ten_found != ten_reached || eleven_found.len() != MARKS as usize {
            return Err(fail(format!("trustee {k} fits {ten_found:?} and {eleven_found:?}")));
        }
        let rest: Vec<u32> = (4..MARKS).chain([0]).collect();
        let note = wrap_note(dial, k, 3).map_err(fail)?;
        // the split, the proof, the grids and the card
        let cards = [
            split_line(pair),
            lands_line(ten, pair),
            prime_line(dial).map_err(fail)?,
            lands_line(dial, pair),
        ];
        let proof = proof_lines(dial).map_err(fail)?;
        let same = same_mark_lines(k);
        let shufflers: Vec<usize> = (1..TEN).filter(|&m| ten.shuffles(m)).map(|m| m as usize - 1).collect();
        if (1..MARKS).any(|m| !dial.shuffles(m)) || shufflers.is_empty() {
            return Err(fail(format!("rows that shuffle: ten {shufflers:?}")));
        }
        let above = TRUSTEES.len().max(SECRET as usize) as u32;
        let primes: Vec<u32> = (above + 1..).filter(|&q| Dial::new(q).is_prime()).take(3).collect();
        let card_words = card_text(&primes);

        // the opening: the rail and its empty frames
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;

        // 13_1: on ten marks 2 × 5 is ten, which lands on zero
        let spoken = say(&kit, stage, "13_1")?;
        spoken.wait_for(stage, "What went wrong")?;
        let face = Face::build(&kit, stage, ten, v(FACE_X, FACE_Y), FACE_R, palette::leak(), 0)?;
        let face_title = kit.text(
            stage,
            &format!("{TEN} marks"),
            FACE_TITLE_SIZE,
            palette::leak(),
            v(FACE_X, FACE_TITLE_Y),
        )?;
        fade_in_all(stage, &[face_title], 0.4)?;
        face.show(stage, 0.6)?;
        spoken.wait_for(stage, "marks that aren't zero")?;
        let halos = [
            face.halo(stage, pair.0, palette::pair()),
            face.halo(stage, pair.1, palette::pair()),
        ];
        glow(stage, &halos, 1.0)?;
        spoken.wait_for(stage, "Two times five is ten")?;
        let mult = kit.text_left(stage, &product[0], PRODUCT_SIZE, Palette::ink(), v(PRODUCT_X, FACE_Y))?;
        fade_in_all(stage, &[mult], 0.4)?;
        // five, and five again: the needle round past the top to 10, mark 0
        face.turn(stage, i64::from(pair.1), 0.6)?;
        face.turn(stage, i64::from(pair.0 * pair.1), 0.6)?;
        spoken.wait_for(stage, "which lands on zero")?;
        let lands = text_after(&kit, stage, mult, &product[1], PRODUCT_SIZE, palette::leak())?;
        let zero = face.halo(stage, 0, palette::leak());
        let mut anims = swaps(stage, &[], &[lands], 0.4)?;
        anims.extend(glowing(stage, &[zero], 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;

        // 13_2: five times each of ten marks crowds onto 0 and 5
        let spoken = say(&kit, stage, "13_2")?;
        spoken.wait_for(stage, "multiplying by five")?;
        let upper = Times::build(&kit, stage, ten, TEN_ROWS, palette::leak(), palette::leak())?;
        fade_in_all(stage, &upper.shown(), 0.6)?;
        let rt = (spoken.at("lands on zero or five")? - now(stage)).clamp(2.0, 3.6);
        arrows_in(stage, &upper.arrows, rt)?;
        spoken.wait_for(stage, "lands on zero or five")?;
        let mut outlines = Vec::with_capacity(ten_reached.len());
        for &m in &ten_reached {
            outlines.push(upper.mark_outline(stage, m, palette::leak())?);
        }
        let ten_counts = upper.counts(&kit, stage, &ten_hits, palette::leak())?;
        let mut anims = glowing(stage, &outlines, 1.2)?;
        anims.extend(swaps(stage, &[], &ten_counts, 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "crowd onto two")?;
        let ten_reach = upper.reach(&kit, stage, &reach_line(ten, k), palette::leak())?;
        fade_in_all(stage, &[ten_reach], 0.4)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_TEN)?;

        // 13_3: the line on ten marks; trustee 5's card fits two secrets
        let spoken = say(&kit, stage, "13_3")?;
        spoken.wait_for(stage, "the straight line from before")?;
        let name = kit.text_left(
            stage,
            &line_name(ten),
            LINE_NAME_SIZE,
            palette::leak(),
            v(rail_left(), LINE_NAME_Y),
        )?;
        fade_in_all(stage, &[name], 0.5)?;
        spoken.wait_for(stage, "deals nine, three, seven, one and five")?;
        let mut drops = Vec::with_capacity(TRUSTEES.len());
        for (&t, c) in TRUSTEES.iter().zip(&line_cards) {
            let m = card(&kit, stage, t, &c.to_string(), palette::leak())?;
            let a = fade_in(stage.arena_mut(), m, v(0.0, -DROP), 1.0)?;
            drops.push(stage.prepare(a)?);
        }
        let rt = (spoken.at("Trustee five's share")? - now(stage)).clamp(1.2, 2.4);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), drops, 0.6)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "Trustee five's share")?;
        let rule = kit.text_left(
            stage,
            &holder_rule(k),
            RULE_SIZE,
            Palette::ink(),
            v(rail_left(), RULE_Y),
        )?;
        let halos = frame_halos(stage, &[k], palette::leak())?;
        let mut anims = swaps(stage, &[], &[rule], 0.5)?;
        anims.extend(glowing(stage, &halos, 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "five times the spin is zero or five")?;
        let mut outlines = Vec::with_capacity(ten_reached.len());
        for &m in &ten_reached {
            outlines.push(upper.mark_outline(stage, m, palette::leak())?);
        }
        glow(stage, &outlines, 1.1)?;
        spoken.wait_for(stage, "leaves two possible secrets")?;
        let ten_fits = Fits::build(&kit, stage, &fits_title(k, height, ten), TEN)?;
        fade_in_all(stage, &ten_fits.parts(), 0.6)?;
        spoken.wait_for(stage, "here zero and five")?;
        let rings: Vec<Mob> = ten_found
            .iter()
            .map(|&s| ring(stage, fit_at(s), FIT_RING, palette::secret()))
            .collect();
        let mut anims = recolorings(stage, &ten_fits.lighting(&ten_found), 0.6)?;
        anims.extend(glowing(stage, &rings, 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;

        // 13_4: the same rows on eleven marks, and the question
        let spoken = say(&kit, stage, "13_4")?;
        spoken.wait_for(stage, "on eleven marks")?;
        let lower = Times::build(&kit, stage, dial, ELEVEN_ROWS, Palette::ink(), palette::spin())?;
        fade_in_all(stage, &lower.shown(), 0.6)?;
        spoken.wait_for(stage, "Multiply every mark by five")?;
        let halo = text_halo(stage, lower.to_title, palette::spin())?;
        glow(stage, &[halo], 1.0)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 13_5: all eleven, each reached once; the card fits every secret
        let spoken = say(&kit, stage, "13_5")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "All eleven")?;
        let eleven_reach = lower.reach(&kit, stage, &reach_line(dial, k), Palette::ink())?;
        fade_in_all(stage, &[eleven_reach], 0.4)?;
        for (x, cue) in [(1, "One goes to five"), (2, "two to ten")] {
            spoken.wait_for(stage, cue)?;
            arrows_in(stage, &lower.arrows_from_marks(&[x]), 0.7)?;
        }
        spoken.wait_for(stage, "wraps round to four")?;
        let wrapped = lower.note(&kit, stage, &note)?;
        fade_in_all(stage, &[wrapped], 0.3)?;
        arrows_in(stage, &lower.arrows_from_marks(&[3]), 0.7)?;
        spoken.wait_for(stage, "Each step lands five further")?;
        let rt = (spoken.at("every mark is reached exactly once")? - now(stage)).clamp(2.4, 4.0);
        arrows_in(stage, &lower.arrows_from_marks(&rest), rt)?;
        spoken.wait_for(stage, "every mark is reached exactly once")?;
        let eleven_counts = lower.counts(&kit, stage, &eleven_hits, palette::spin())?;
        let o = lower.row_outline(stage, palette::spin())?;
        let mut anims = swaps(stage, &[], &eleven_counts, 0.5)?;
        anims.extend(glowing(stage, &[o], 1.2)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "trustee five's share would fit every secret")?;
        let eleven_fits = Fits::build(&kit, stage, &fits_title(k, height, dial), MARKS)?;
        let anims = swaps(stage, &ten_fits.parts(), &eleven_fits.parts(), 0.6)?;
        stage.play_prepared(anims)?;
        let anims = recolorings(stage, &eleven_fits.lighting(&eleven_found), 0.8)?;
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_BOTH)?;

        // 13_6: ten splits as 2 × 5; eleven doesn't, it's prime
        let spoken = say(&kit, stage, "13_6")?;
        let mut going: Vec<Mob> = face.parts().to_vec();
        going.extend([face_title, mult, lands, name, rule]);
        going.extend(eleven_fits.parts());
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        // ten's lines red, eleven's split gold and its product plain
        let card_colors = [palette::leak(), palette::leak(), palette::secret(), Palette::ink()];
        let mut read = now(stage);
        let mut card_mobs = Vec::with_capacity(cards.len());
        for (lines, cue) in [
            (0..1, "Ten is two times five"),
            (1..2, "their product lands on zero"),
            (2..4, "Eleven doesn't split like that"),
        ] {
            spoken.wait_for(stage, cue)?;
            let (new, by) = write_lines(&kit, stage, &cards, &CARD_Y, &card_colors, lines, read)?;
            card_mobs.extend(new);
            read = by;
        }
        spoken.wait_for(stage, "It's prime")?;
        let o = text_halo(stage, card_mobs[2], palette::secret())?;
        glow(stage, &[o], 1.2)?;
        spoken.wait_end(stage)?;

        // 13_7: primes split a number one way only, so on eleven marks a
        // product lands on zero only when a factor does
        let spoken = say(&kit, stage, "13_7")?;
        let plain = [Palette::ink(); 5];
        let mut proof_mobs = Vec::with_capacity(proof.len());
        for (lines, cue) in [
            (0..1, "Landing on zero means"),
            (1..2, "Every whole number splits into primes"),
            (2..3, "so a product of marks has eleven"),
            (3..4, "No mark from one to ten does"),
            (4..5, "So on eleven marks, a product lands on zero"),
        ] {
            spoken.wait_for(stage, cue)?;
            let (new, by) = write_lines(&kit, stage, &proof, &PROOF_Y, &plain, lines, read)?;
            proof_mobs.extend(new);
            read = by;
        }
        let lemma = proof_mobs[4];
        let o = text_halo(stage, lemma, palette::secret())?;
        glow(stage, &[o], 1.2)?;
        spoken.wait_end(stage)?;
        wait_until(stage, read)?;

        // 13_8: so every multiplier from 1 to 10 shuffles the eleven marks;
        // on ten marks only 1, 3, 7 and 9 do
        let spoken = say(&kit, stage, "13_8")?;
        let mut going = card_mobs.clone();
        going.extend(&proof_mobs[..4]);
        let mut anims = leaving(stage, &going, ORIGIN, 0.5)?;
        let at = stage.get_center(lemma);
        anims.push(moving(stage, lemma, v(at[0], LEMMA_Y), 0.6)?);
        stage.play_prepared(anims)?;
        let mut read = now(stage);
        for (lines, cue) in [
            (0..1, "if two marks gave the same answer"),
            (1..2, "five times their difference would land on zero"),
            (2..4, "unless they're the same mark"),
        ] {
            spoken.wait_for(stage, cue)?;
            let (_, by) = write_lines(&kit, stage, &same, &SAME_Y, &plain, lines, read)?;
            read = by;
        }
        spoken.wait_for(stage, "every multiplier from one to ten")?;
        let mut right = upper.shown();
        right.extend(upper.drawn());
        right.extend(lower.shown());
        right.extend(lower.drawn());
        right.extend(ten_counts);
        right.extend(eleven_counts);
        right.extend([ten_reach, eleven_reach, wrapped]);
        let anims = leaving(stage, &right, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let eleven_grid = ReachGrid::build(&kit, stage, dial, ELEVEN_GRID, palette::leak())?;
        let rt = (spoken.at("each one only shuffles the eleven marks")? - now(stage)).clamp(1.2, 2.2);
        eleven_grid.appear(stage, rt)?;
        read = read_on(stage, read, &grid_title(dial));
        spoken.wait_for(stage, "each one only shuffles the eleven marks")?;
        let all_rows = eleven_grid.outline_rows(stage, 0, eleven_grid.rows.len() - 1, palette::spin())?;
        glow(stage, &[all_rows], 1.2)?;
        spoken.wait_for(stage, "On ten marks")?;
        let ten_grid = ReachGrid::build(&kit, stage, ten, TEN_GRID, palette::leak())?;
        let rt = (spoken.at("only one, three, seven and nine do")? - now(stage)).clamp(0.8, 1.4);
        ten_grid.appear(stage, rt)?;
        read = read_on(stage, read, &grid_title(ten));
        spoken.wait_for(stage, "only one, three, seven and nine do")?;
        let mut full = Vec::with_capacity(shufflers.len());
        for &i in &shufflers {
            full.push(ten_grid.outline_rows(stage, i, i, palette::spin())?);
        }
        glow(stage, &full, 1.3)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_GRIDS)?;

        // 13_9: any prime above the trustees and the secret; the card
        let spoken = say(&kit, stage, "13_9")?;
        spoken.wait_for(stage, "Any prime number of marks works")?;
        let exact = precisely(&kit, stage, &card_words)?;
        let anims = swaps(stage, &[], &[exact], 0.5)?;
        stage.play_prepared(anims)?;
        let read = read_on(stage, read, &card_words);
        spoken.wait_for(stage, "each one has a position besides zero")?;
        let rings: Vec<Mob> = TRUSTEES
            .iter()
            .map(|&t| disc_ring(stage, t, palette::share()))
            .collect();
        glow(stage, &rings, 1.2)?;
        spoken.wait_end(stage)?;
        wait_until(stage, read)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{FRAME_H, THINK_R};
    use super::super::two_keys::TEXT_PAD;
    use super::*;
    use crate::facts;
    use crate::model::csv;

    /// The production narration: registering chapter 13 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// `k` times every mark of `p`, written out with ordinary remainders,
    /// apart from the model's dial.
    fn times(p: u32, k: u32) -> Vec<u32> {
        (0..p).map(|x| (k * x) % p).collect()
    }

    /// The secrets a line's card `y` at position `x` fits on `p` marks:
    /// every secret and spin tried.
    fn fits_by_hand(p: u32, x: u32, y: u32) -> Vec<u32> {
        (0..p).filter(|&s| (0..p).any(|a| (s + a * x) % p == y)).collect()
    }

    /// Every phrase the scene waits for is in its line of the script, once,
    /// and they come in the order they are said.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let phrases = [
            ("13_1", "What went wrong"),
            ("13_1", "marks that aren't zero"),
            ("13_1", "Two times five is ten"),
            ("13_1", "which lands on zero"),
            ("13_2", "multiplying by five"),
            ("13_2", "lands on zero or five"),
            ("13_2", "crowd onto two"),
            ("13_3", "the straight line from before"),
            ("13_3", "deals nine, three, seven, one and five"),
            ("13_3", "Trustee five's share"),
            ("13_3", "five times the spin is zero or five"),
            ("13_3", "leaves two possible secrets"),
            ("13_3", "here zero and five"),
            ("13_4", "on eleven marks"),
            ("13_4", "Multiply every mark by five"),
            ("13_5", "All eleven"),
            ("13_5", "One goes to five"),
            ("13_5", "two to ten"),
            ("13_5", "wraps round to four"),
            ("13_5", "Each step lands five further"),
            ("13_5", "every mark is reached exactly once"),
            ("13_5", "trustee five's share would fit every secret"),
            ("13_6", "Ten is two times five"),
            ("13_6", "their product lands on zero"),
            ("13_6", "Eleven doesn't split like that"),
            ("13_6", "It's prime"),
            ("13_7", "Landing on zero means"),
            ("13_7", "Every whole number splits into primes"),
            ("13_7", "so a product of marks has eleven"),
            ("13_7", "No mark from one to ten does"),
            ("13_7", "So on eleven marks, a product lands on zero"),
            ("13_8", "if two marks gave the same answer"),
            ("13_8", "five times their difference would land on zero"),
            ("13_8", "unless they're the same mark"),
            ("13_8", "every multiplier from one to ten"),
            ("13_8", "each one only shuffles the eleven marks"),
            ("13_8", "On ten marks"),
            ("13_8", "only one, three, seven and nine do"),
            ("13_9", "Any prime number of marks works"),
            ("13_9", "each one has a position besides zero"),
        ];
        let mut last: Option<(&str, usize)> = None;
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} in {text}");
            let at = text.find(phrase).unwrap();
            if let Some((prev_id, prev_at)) = last {
                assert!(
                    prev_id != id || prev_at < at,
                    "{id}: {phrase:?} is waited for out of order"
                );
            }
            last = Some((id, at));
        }
        // the question ends 13_4, so the think time follows the whole of it
        assert!(
            script
                .text("13_4")
                .unwrap()
                .ends_with("How many different marks come out?")
        );
    }

    /// What the scene draws and writes is the facts': the zero product,
    /// both dials' rows, counts and reach, the line's cards and what
    /// trustee 5's card fits, the split, the grids and the card.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}")).to_owned();
        let (dial, ten, k) = (Dial::vault(), Dial::new(TEN), TEN_HOLDER);
        let pair = zero_product(ten).unwrap();
        assert_eq!(get("S-F8.ten.marks"), TEN.to_string());
        assert_eq!(get("S-F8.ten.zero_product"), format!("{},{}", pair.0, pair.1));
        assert_eq!(
            get("S-F8.ten.zero_product_sum"),
            format!("{},{},{}", pair.0, pair.1, pair.0 * pair.1)
        );
        assert_eq!(get("S-F8.ten.zero_product_mark"), ten.mul(pair.0, pair.1).to_string());
        assert_eq!(product_pieces(ten, pair), ["2 \u{d7} 5 = 10", "\u{2192} 0"]);
        assert_eq!(get("S-F8.multiplier"), k.to_string());
        assert_eq!(times_title(k), "5 \u{d7} mark");
        // the rows
        assert_eq!(get("S-F8.ten.times5"), csv(&ten.times_row(k)));
        assert_eq!(get("S-F8.eleven.times5"), csv(&dial.times_row(k)));
        assert_eq!(get("S-F8.ten.times5_values"), csv(&reached(ten, k)));
        assert_eq!(get("S-F8.ten.times5_reached"), reached(ten, k).len().to_string());
        assert_eq!(get("S-F8.ten.times5_hits"), csv(&hits(ten, k)));
        assert_eq!(get("S-F8.eleven.times5_reached"), reached(dial, k).len().to_string());
        assert_eq!(get("S-F8.eleven.times5_hits"), csv(&hits(dial, k)));
        assert_eq!(get("S-F8.eleven.times5_each"), "1");
        assert_eq!(reach_line(ten, k), "reaches 2 of 10 marks");
        assert_eq!(reach_line(dial, k), "reaches 11 of 11 marks");
        assert_eq!(get("S-F8.eleven.times5_step.1"), "1,5");
        assert_eq!(get("S-F8.eleven.times5_step.2"), "2,10");
        assert_eq!(get("S-F8.eleven.times5_step.3"), "3,15,4");
        assert_eq!(wrap_note(dial, k, 3).unwrap(), "15 \u{2192} 4");
        // the line's cards on ten marks, and what trustee 5's card fits
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        assert_eq!(get("S-F8.line.shares"), csv(&line.deal(ten, &TRUSTEES)));
        assert_eq!(get("S-F8.line.holder"), k.to_string());
        assert_eq!(get("S-F8.line.holder_share"), line.share(ten, k).to_string());
        assert_eq!(get("S-F8.line.candidates"), csv(&fitting(ten, k, line.share(ten, k))));
        assert_eq!(get("S-F8.line.candidate_count"), "2");
        let heights: Vec<i64> = TRUSTEES.iter().map(|&x| line.height(x)).collect();
        assert_eq!(get("S-F4.heights"), csv(&heights));
        assert_eq!(
            get("S-F4.wrap.5"),
            format!("{},2,{}", line.height(k), line.share(dial, k))
        );
        assert_eq!(
            get("S-F8.eleven.line_candidates"),
            fitting(dial, k, line.share(dial, k)).len().to_string()
        );
        assert_eq!(line_name(ten), "the line 5 + 4 \u{d7} position, on 10 marks");
        assert_eq!(holder_rule(k), "trustee 5: secret + 5 \u{d7} spin");
        assert_eq!(
            fits_title(k, line.height(k), ten),
            "secrets that fit trustee 5's 25 \u{2192} 5, on 10 marks"
        );
        assert_eq!(
            fits_title(k, line.height(k), dial),
            "secrets that fit trustee 5's 25 \u{2192} 3, on 11 marks"
        );
        // the split, the factors, the product on eleven marks
        assert_eq!(get("S-F8.ten.split"), "10,2,5");
        assert_eq!(split_line(pair), "10 = 2 \u{d7} 5");
        assert_eq!(lands_line(ten, pair), "on 10 marks, 2 \u{d7} 5 = 10 lands on 0");
        assert_eq!(get("S-F8.eleven.factors"), "1,11");
        assert_eq!(
            prime_line(dial).unwrap(),
            "11 = 1 \u{d7} 11: no two numbers from 2 to 10 multiply to 11"
        );
        assert_eq!(get("S-F8.eleven.zero_product_check"), format!("2,5,{}", dial.mul(2, 5)));
        assert_eq!(lands_line(dial, pair), "on 11 marks, 2 \u{d7} 5 = 10 lands on 10");
        assert_eq!(get("S-F8.eleven.nonzero_range"), format!("1,{}", MARKS - 1));
        assert!(proof_lines(dial).unwrap()[3].starts_with("a and b from 1 to 10:"));
        // the grids
        let count = |d: Dial| -> Vec<usize> {
            reach_rows(d)
                .iter()
                .map(|r| r.iter().filter(|&&on| on).count())
                .collect()
        };
        assert_eq!(get("S-F8.ten.reach"), csv(&count(ten)));
        assert_eq!(get("S-F8.eleven.reach"), csv(&count(dial)));
        let full: Vec<u32> = (1..TEN).filter(|&m| ten.shuffles(m)).collect();
        assert_eq!(get("S-F8.ten.rows_that_shuffle"), csv(&full));
        assert_eq!(get("S-F8.eleven.rows_that_shuffle"), (MARKS - 1).to_string());
        assert_eq!(
            grid_title(ten),
            "on 10 marks, the marks reached by \u{d7} 1 to \u{d7} 9"
        );
        // the card: the first primes above both the trustees and the secret
        // are the facts' primes above the trustees, since the secret is 5
        let above = TRUSTEES.len().max(SECRET as usize) as u32;
        let primes: Vec<u32> = (above + 1..).filter(|&q| Dial::new(q).is_prime()).take(3).collect();
        assert_eq!(get("S-F8.primes_for_five"), csv(&primes));
        assert_eq!(get("S-F8.smallest_prime_for_five"), primes[0].to_string());
        assert_eq!(get("S-F1.trustees"), TRUSTEES.len().to_string());
        assert_eq!(get("S-X1.shamir.year"), SHAMIR_YEAR.to_string());
        assert_eq!(
            card_text(&[7, 11, 13]),
            "the counting holds on any prime number of marks above both the number of trustees and \
             the secret (here: 7, 11, 13 and on; Shamir 1979)"
        );
    }

    /// The rows counted by hand, apart from the model: on ten marks five
    /// times a mark is 0 or 5, five times each; on eleven every mark once;
    /// trustee 5's card fits 0 and 5 on ten marks, every secret on eleven.
    /// The wrong-modulus controls: twelve marks are not prime, yet five
    /// shuffles them (only two, three, four and the rest sharing a factor
    /// with twelve collapse), so the chapter never says a composite dial
    /// always fails; on nine marks three collapses; on 49 marks every
    /// multiplier up to the trustees shuffles. Prime is enough, not needed.
    #[test]
    fn the_rows_are_counted_by_hand() {
        let k = TEN_HOLDER;
        assert_eq!(times(TEN, k), vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5]);
        assert_eq!(times(TEN, k), Dial::new(TEN).times_row(k));
        assert_eq!(times(MARKS, k), vec![0, 5, 10, 4, 9, 3, 8, 2, 7, 1, 6]);
        assert_eq!(times(MARKS, k), Dial::vault().times_row(k));
        assert_eq!(hits(Dial::new(TEN), k), vec![5, 0, 0, 0, 0, 5, 0, 0, 0, 0]);
        assert_eq!(hits(Dial::vault(), k), vec![1; 11]);
        // each step of five round eleven marks lands five further on
        let row = times(MARKS, k);
        for x in 1..MARKS {
            assert_eq!(row[x as usize], (row[x as usize - 1] + k) % MARKS);
        }
        // the line's card
        assert_eq!(fits_by_hand(TEN, k, 5), vec![0, 5]);
        assert_eq!(fits_by_hand(TEN, k, 5), fitting(Dial::new(TEN), k, 5));
        assert_eq!(fits_by_hand(MARKS, k, 3), (0..MARKS).collect::<Vec<_>>());
        assert_eq!(fits_by_hand(MARKS, k, 3), fitting(Dial::vault(), k, 3));
        // a wrong card on ten marks fits a different pair
        assert_eq!(fits_by_hand(TEN, k, 6), vec![1, 6]);
        // wrong-modulus controls
        assert!(Dial::new(12).shuffles(5));
        assert_eq!(reached(Dial::new(12), 5).len(), 12);
        let twelve: Vec<u32> = (1..12).filter(|&m| !Dial::new(12).shuffles(m)).collect();
        assert_eq!(twelve, vec![2, 3, 4, 6, 8, 9, 10]);
        assert_eq!(reached(Dial::new(9), 3), vec![0, 3, 6]);
        assert!((1..=TRUSTEES.len() as u32 + 1).all(|m| Dial::new(49).shuffles(m)));
        assert!(!Dial::new(49).is_prime());
        assert!(wrap_note(Dial::vault(), k, 2).is_err());
    }

    /// The proof's claim, every product tried: on eleven marks two marks
    /// multiply to 0 only if one is 0, and multiplying by any of 1 to 10
    /// is a shuffle; on ten marks 2 × 5, 4 × 5, 5 × 6 and 5 × 8 land on 0,
    /// so ten's lines never claim the lemma. Unique splitting into primes,
    /// as the proof uses it: 11 divides a product of 1 to 10 never.
    #[test]
    fn the_proof_holds_for_every_product() {
        for a in 0..MARKS {
            for b in 0..MARKS {
                assert_eq!((a * b) % MARKS == 0, a == 0 || b == 0, "{a} \u{d7} {b}");
            }
        }
        for m in 1..MARKS {
            let mut row = times(MARKS, m);
            row.sort_unstable();
            assert_eq!(row, (0..MARKS).collect::<Vec<_>>(), "\u{d7} {m}");
        }
        let zero: Vec<(u32, u32)> = (1..TEN)
            .flat_map(|a| (a..TEN).map(move |b| (a, b)))
            .filter(|&(a, b)| (a * b) % TEN == 0)
            .collect();
        assert_eq!(zero, vec![(2, 5), (4, 5), (5, 6), (5, 8)]);
        assert!(proof_lines(Dial::new(TEN)).is_err());
        assert!(prime_line(Dial::new(TEN)).is_err());
        assert!(prime_line(Dial::new(7)).is_ok());
        // two marks with the same five times are the same mark
        for u in 0..MARKS {
            for w in 0..MARKS {
                if (TEN_HOLDER * u) % MARKS == (TEN_HOLDER * w) % MARKS {
                    assert_eq!(u, w);
                }
            }
        }
        assert_eq!(
            same_mark_lines(TEN_HOLDER),
            [
                "5 \u{d7} u and 5 \u{d7} w land on the same mark",
                "so 5 \u{d7} (u \u{2212} w) lands on 0",
                "5 isn't 0, so u \u{2212} w lands on 0:",
                "u and w are the same mark",
            ]
        );
        // the grids, row by row
        let rows = reach_rows(Dial::new(TEN));
        let full: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].iter().all(|&on| on)).collect();
        assert_eq!(full, vec![0, 2, 6, 8]);
        assert!(reach_rows(Dial::vault()).iter().all(|r| r.iter().all(|&on| on)));
        assert_eq!(reach_rows(Dial::vault()).len(), 10);
    }

    /// The precisely card stays two lines at every wrap the lower third can
    /// narrow to, and says the counting holds, never that a prime is
    /// needed.
    #[test]
    fn the_card_is_two_lines() {
        let text = card_text(&[7, 11, 13]);
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
        for word in ["needs", "only", "secure", "safe"] {
            assert!(!text.contains(word), "{word}");
        }
    }

    /// The pieces keep apart, inside the frame: the face and its product
    /// upper left, the secrets' ticks under it, the rule and the line's
    /// name over the rail, the think ring between them and the eleven
    /// rows, both dials' rows right on the strip's columns, the cards, the
    /// proof and the derivation left of the rows' titles, and the grids in
    /// the rows' place, over the precisely card.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.3;
        let top = FRAME_Y_RADIUS - 0.3;
        // the face, its title and the product
        const {
            assert!(FACE_X - FACE_R > -FRAME_X_RADIUS + 0.3);
            assert!(FACE_TITLE_Y + FACE_TITLE_SIZE * 0.0065 / 2.0 < FRAME_Y_RADIUS - 0.3);
            assert!(FACE_TITLE_Y - FACE_TITLE_SIZE * 0.0065 / 2.0 > FACE_Y + FACE_R + 0.1);
            assert!(PRODUCT_X > FACE_X + FACE_R + 0.2);
        }
        let product_end = PRODUCT_X + text_w("2 \u{d7} 5 = 10 \u{2192} 0", PRODUCT_SIZE) + 0.14;
        assert!(product_end < THINK_X - THINK_R - 0.3);
        // the secrets' ticks under the face, the rule under their marks
        let ten_title = "secrets that fit trustee 5's 25 \u{2192} 5, on 10 marks";
        assert!(FITS_TITLE_Y + text_h(FITS_TITLE_SIZE) / 2.0 < FACE_Y - FACE_R - 0.1);
        assert!(FITS_TITLE_Y - text_h(FITS_TITLE_SIZE) / 2.0 > FIT_Y + FIT_H / 2.0 + 0.1);
        assert!(FIT_LABEL_Y + text_h(FIT_LABEL_SIZE) / 2.0 < FIT_Y - FIT_H / 2.0);
        assert!(RULE_Y + text_h(RULE_SIZE) / 2.0 < FIT_LABEL_Y - text_h(FIT_LABEL_SIZE) / 2.0 - 0.3);
        assert!(FIT_RING * 2.0 < FIT_DX + 0.01 && FIT_RING > (FIT_W.hypot(FIT_H)) / 2.0);
        assert!(fit_at(0)[0] - FIT_W / 2.0 > -FRAME_X_RADIUS + 0.3);
        assert!(FITS_LEFT + text_w(ten_title, FITS_TITLE_SIZE) < THINK_X - THINK_R - 0.2);
        // the rule and the line's name over the rail's frames
        let frame_top = frame_at(1)[1] + FRAME_H / 2.0;
        assert!(LINE_NAME_Y - text_h(LINE_NAME_SIZE) / 2.0 > frame_top + 0.2);
        assert!(RULE_Y - LINE_NAME_Y > text_h(RULE_SIZE) + 0.2);
        // the think ring: right of the ten ticks and the words, left of the
        // eleven rows' titles, its note over the frames and under the ring
        let think_left = THINK_X - THINK_R;
        assert!(fit_at(TEN - 1)[0] + FIT_W / 2.0 < think_left - 0.2);
        let note_y = THINK_Y - THINK_R - 0.3;
        let note_w = text_w("pause for more time", 18.0);
        assert!(rail_left() + text_w(&line_name(Dial::new(TEN)), LINE_NAME_SIZE) < THINK_X - note_w / 2.0 - 0.2);
        assert!(note_y - text_h(18.0) / 2.0 > frame_top + 0.2);
        let title_right = STRIP_LEFT - ROW_TITLE_GAP;
        let times_w = text_w(&times_title(TEN_HOLDER), ROW_TITLE_SIZE);
        assert!(THINK_X + THINK_R < title_right - text_w(FROM_TITLE, ROW_TITLE_SIZE) - 0.2);
        // the ring's lowest point under the left of "5 × mark", and its
        // note under that title
        let dx = title_right - times_w - THINK_X;
        let low = THINK_Y - (THINK_R * THINK_R - dx * dx).max(0.0).sqrt();
        assert!(low > ELEVEN_ROWS.to + text_h(ROW_TITLE_SIZE) / 2.0 + 0.1, "{low}");
        assert!(note_y + text_h(18.0) / 2.0 < ELEVEN_ROWS.to - text_h(ROW_TITLE_SIZE) / 2.0 - 0.05);
        // the eleven ticks, after the ring has gone, left of the rows' titles
        assert!(fit_at(MARKS - 1)[0] + FIT_W / 2.0 < title_right - times_w - 0.2);
        // the rows, each dial's in order, the outlines clear of the counts,
        // the ten over the eleven, all inside
        for (rows, marks) in [(TEN_ROWS, TEN), (ELEVEN_ROWS, MARKS)] {
            assert!(rows.title - text_h(ROWS_TITLE_SIZE) / 2.0 > rows.from + text_h(ROW_SIZE) / 2.0 + 0.1);
            assert!(rows.from - rows.to > 2.0 * ARROW_GAP + 0.5, "{rows:?}");
            let outline_bottom = rows.to - HIT_R - PAD;
            assert!(outline_bottom > rows.counts + text_h(COUNT_SIZE) / 2.0 + 0.05);
            assert!(rows.counts - text_h(COUNT_SIZE) / 2.0 > rows.reach + text_h(REACH_SIZE) / 2.0 + 0.1);
            assert!(strip_right(marks) < edge);
            assert!(title_right - times_w > -FRAME_X_RADIUS);
            let reach_end = STRIP_LEFT + text_w(&format!("reaches {marks} of {marks} marks"), REACH_SIZE);
            assert!(reach_end < strip_right(marks) - text_w("15 \u{2192} 4", NOTE_SIZE) - 0.3);
        }
        assert!(TEN_ROWS.title + text_h(ROWS_TITLE_SIZE) / 2.0 < top + 0.25);
        assert!(TEN_ROWS.reach - text_h(REACH_SIZE) / 2.0 > ELEVEN_ROWS.title + text_h(ROWS_TITLE_SIZE) / 2.0 + 0.1);
        assert!(ELEVEN_ROWS.reach - text_h(REACH_SIZE) / 2.0 > -FRAME_Y_RADIUS + 0.3);
        assert!(2.0 * HIT_R < tick_at(1)[0] - tick_at(0)[0]);
        // the cards, the proof and the derivation: left of the rows' titles,
        // their lines apart, the proof over the rail
        let left_limit = title_right - times_w - 0.05;
        let dial = Dial::vault();
        let ten = Dial::new(TEN);
        let pair = zero_product(ten).unwrap();
        let cards = [
            split_line(pair),
            lands_line(ten, pair),
            prime_line(dial).unwrap(),
            lands_line(dial, pair),
        ];
        let proof = proof_lines(dial).unwrap();
        let same = same_mark_lines(TEN_HOLDER);
        for text in cards.iter().chain(&proof).chain(&same) {
            assert!(LEFT + text_w(text, LINE_SIZE) < left_limit, "{text}");
        }
        const {
            assert!(LEFT > -FRAME_X_RADIUS + 0.3);
        }
        let ys: Vec<f64> = CARD_Y.iter().chain(&PROOF_Y).copied().collect();
        for w in ys.windows(2) {
            assert!(w[0] - w[1] > text_h(LINE_SIZE) + 0.2, "{w:?}");
        }
        assert!(CARD_Y[0] + text_h(LINE_SIZE) / 2.0 < top);
        assert!(PROOF_Y[4] - text_h(LINE_SIZE) / 2.0 - TEXT_PAD > frame_top + 0.3);
        let ys: Vec<f64> = std::iter::once(LEMMA_Y).chain(SAME_Y).collect();
        for w in ys.windows(2) {
            assert!(w[0] - w[1] > text_h(LINE_SIZE) + 0.2, "{w:?}");
        }
        assert!(LEMMA_Y + text_h(LINE_SIZE) / 2.0 < top);
        // the grids: inside, the ten over the eleven, the cells apart, the
        // eleven over the precisely card's two lines
        for (layout, d) in [(TEN_GRID, ten), (ELEVEN_GRID, dial)] {
            assert!(layout.title - text_h(GRID_TITLE_SIZE) / 2.0 > layout.labels + text_h(GRID_LABEL_SIZE) / 2.0);
            assert!(layout.labels - text_h(GRID_LABEL_SIZE) / 2.0 > layout.first + CELL_H / 2.0);
            assert!(STRIP_LEFT + text_w(&grid_title(d), GRID_TITLE_SIZE) < edge);
        }
        assert!(TEN_GRID.title + text_h(GRID_TITLE_SIZE) / 2.0 < top + 0.25);
        let ten_bottom = grid_y(TEN_GRID, TEN as usize - 2) - CELL_H / 2.0;
        assert!(ten_bottom > ELEVEN_GRID.title + text_h(GRID_TITLE_SIZE) / 2.0 + 0.1);
        let eleven_bottom = grid_y(ELEVEN_GRID, MARKS as usize - 2) - CELL_H / 2.0 - PAD;
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        assert!(eleven_bottom > card_top + 0.1, "{eleven_bottom}");
        assert!(CELL_W < tick_at(1)[0] - tick_at(0)[0] - 0.1);
        // the cells' rows apart, and the outlines round rows 1 and 3 of the
        // ten grid too
        const {
            assert!(GRID_DY > CELL_H + 0.05);
            assert!(2.0 * GRID_DY > CELL_H + 2.0 * PAD + 0.1);
        }
    }
}
