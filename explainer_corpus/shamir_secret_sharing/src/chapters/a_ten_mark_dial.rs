//! Chapter 12, a ten-mark dial: why the vault's dial has eleven marks. The
//! rail holds the vault's cards, 8, 3, 1, 2 and 6, with each trustee's
//! height on paper over its frame, and the viewer is asked whether trustees
//! 4 and 5 would still learn nothing if the same secret and spins were
//! dealt on ten marks (12_1). There each height lands on its last digit, so
//! the pair would hold 5 and 5, not 2 and 6 (12_2). Chapter 05's tally, once
//! for each dial, one over the other on the strip's columns, shows what that
//! does before any word about why: on eleven marks every secret fits the
//! pair's cards one way; on ten marks only the secret 5 fits, ten ways
//! (12_3). The pair's heights are written in letters, the secret's, the
//! first spin's and the second spin's amounts in three columns (12_4); five
//! copies of one and four of the other hold the same amount of the first
//! spin, which cancels when one is taken from the other, leaving the secret
//! less twenty of the second spin (12_5). Twenty lands on zero on ten marks,
//! so 5 × 5 − 4 × 5 on the pair's cards is the secret itself (12_6); on
//! eleven marks −20 lands on 2, and the same sum, 8, is the secret plus
//! twice the second spin, which every secret makes with one second spin
//! (12_7). A table counts what every pair's two shares leave on ten marks:
//! nine pairs always cross some secrets off, two always find it, and
//! trustees 1 and 3 learn nothing (12_8).
//!
//! The tallies are `model::fitting_spins` counted for the pair's dealt
//! cards on each dial ([`tally`]); the scene stops unless the eleven-mark
//! tally is level and the ten-mark one lights the secret alone. The table
//! is `model::what_shares_leave` for every pair, every recipe dealt
//! ([`table`]); what is left always fits equally well, a likelihood, and the
//! table never calls it equally likely. Emphasis never restyles or scales
//! typeset text: rings, outlines and strokes of their own point at it, and
//! only shapes (bars, ticks) change color or swell.

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::board::{create_lagged, now, ring};
use super::draw::{
    CARD_SIZE, FRAME_H, FRAME_W, READ_CPS, Rail, STRIP_LEFT, TICK_W, card, fade_in_all, fail, frame, frame_at, panel,
    pause, read_by, recolorings, say, stroke, swaps, think, tick_at, wait_until,
};
use super::envelope::{PAD, dealt, frame_halos, outline, swelling};
use super::two_keys::{Face, TEXT_PAD, glow, glowing, leaving, text_halo};
use super::two_shares::{DROP, disc_ring, text_right};
use crate::model::{self, Dial, MARKS, PAIR, Recipe, SECRET, TEN, TRUSTEES, combinations};
use crate::palette;

pub struct ATenMarkDial {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const VAULT_TITLE: &str = "the vault: 11 marks";
const TEN_TITLE: &str = "10 marks";
const HEIGHTS_NAME: &str = "height on paper";
const TEN_NAME: &str = "on 10 marks";
const ELEVEN_NAME: &str = "on 11 marks";
const WAYS: &str = "ways";
const LEGEND: &str = "s: the secret   a: the 1st spin   b: the 2nd spin";
const TABLE_TITLE: &str = "on 10 marks, what each pair's two shares leave";
const TABLE_NOTE: &str = "whatever shares they hold; what is left fits equally well";

// ------------------------------------------------------------------ layout

/// The two faces, upper left (12_1 and 12_2), and their titles over them.
const FACE_R: f64 = 0.9;
const VAULT_FACE_X: f64 = -5.3;
const TEN_FACE_X: f64 = -2.7;
const FACE_Y: f64 = 2.05;
const FACE_TITLE_Y: f64 = 3.25;
const FACE_TITLE_SIZE: f64 = 20.0;
/// Over the rail's frames: each trustee's height on paper, and the red
/// frames for the cards it lands on with ten marks; the rows' names left
/// of the tallies.
const HEIGHTS_Y: f64 = -0.2;
const HEIGHT_SIZE: f64 = 22.0;
const TEN_FRAME_Y: f64 = -0.8;
const RED_FRAME_PAD: f64 = 0.06;
const NAME_X: f64 = -1.6;
const NAME_SIZE: f64 = 18.0;
/// The think ring, right (12_1).
const THINK_X: f64 = 3.8;
const THINK_Y: f64 = 1.2;

/// One tally's rows: its title, the bars' foot, its ticks, their labels,
/// and the lines under it.
#[derive(Clone, Copy, Debug)]
pub struct Rows {
    pub title: f64,
    pub base: f64,
    pub tick: f64,
    pub label: f64,
    pub lines: [f64; 3],
}

/// The eleven-mark tally over the ten-mark one, right, on the strip's
/// columns, so a mark stands in the same place on both.
pub const ELEVEN_ROWS: Rows = Rows {
    title: 3.45,
    base: 2.78,
    tick: 2.62,
    label: 2.32,
    lines: [1.95, 1.6, 1.25],
};
pub const TEN_ROWS: Rows = Rows {
    title: 0.3,
    base: -1.38,
    tick: -1.54,
    label: -1.84,
    lines: [-2.25, -2.62, -2.99],
};
/// A way's height on a bar, a bar's width, a tally tick's height, the
/// bars' counts over them, "ways" left of the bars, and the sizes.
pub const UNIT: f64 = 0.11;
const BAR_W: f64 = 0.3;
const TALLY_TICK_H: f64 = 0.2;
const COUNT_DY: f64 = 0.17;
const COUNT_SIZE: f64 = 18.0;
const WAYS_GAP: f64 = 0.15;
const WAYS_DY: f64 = 0.1;
const WAYS_SIZE: f64 = 16.0;
const TALLY_TITLE_SIZE: f64 = 20.0;
const TALLY_LABEL_SIZE: f64 = 18.0;
const FIT_SIZE: f64 = 20.0;
const SUM_SIZE: f64 = 22.0;
const TICK_RING: f64 = 0.22;
/// The derivation, upper left (12_4 to 12_7): the legend, the four rows and
/// the result under the rule, each row's name left of three columns.
const LEFT: f64 = -6.7;
const LEGEND_Y: f64 = 3.45;
const LEGEND_SIZE: f64 = 16.0;
pub const ROW_Y: [f64; 5] = [2.9, 2.45, 1.9, 1.45, 0.8];
const RULE_Y: f64 = 1.13;
pub const COL_X: [f64; 3] = [-4.4, -3.25, -2.0];
const MINUS_DX: f64 = 0.55;
const ROW_NAME_SIZE: f64 = 18.0;
const TERM_SIZE: f64 = 24.0;
const STRIKE_W: f64 = 3.0;
const RULE_W: f64 = 2.0;
/// A space between the pieces of one written line.
const PIECE_GAP: f64 = 0.14;
/// The table, upper left (12_8).
const TABLE_TITLE_Y: f64 = 3.45;
const TABLE_TITLE_SIZE: f64 = 20.0;
pub const TABLE_Y: [f64; 4] = [2.95, 2.55, 2.15, 1.75];
const TABLE_SIZE: f64 = 18.0;
const TABLE_NOTE_Y: f64 = 1.3;
const TABLE_NOTE_SIZE: f64 = 16.0;
const TABLE_ELEVEN_Y: f64 = 0.9;
/// The hold on the two tallies after 12_3, and after the table is read.
const HOLD_ON_TALLIES: f64 = 1.5;
const HOLD_AT_END: f64 = 0.8;

// -------------------------------------------------------------------- data

/// For each secret, how many spin pairs deal `held` (position, card) on
/// `dial` with it: chapter 05's tally.
pub fn tally(dial: Dial, held: &[(u32, u32)]) -> Vec<usize> {
    dial.all()
        .map(|s| model::fitting_spins(dial, held, s, 2).len())
        .collect()
}

/// A tally's title: "10 marks: trustees 4 and 5 hold 5 and 5".
pub fn tally_title(marks: u32, held: &[(u32, u32)]) -> String {
    format!(
        "{marks} marks: trustees {} and {} hold {} and {}",
        held[0].0, held[1].0, held[0].1, held[1].1
    )
}

/// What a tally says under it: every secret fitting the same number of
/// ways, or one secret alone; an error for any other tally, which this
/// chapter does not describe.
pub fn fits_line(ways: &[usize]) -> Result<String, String> {
    let lit: Vec<(usize, usize)> = ways.iter().copied().enumerate().filter(|&(_, w)| w > 0).collect();
    match lit.as_slice() {
        [(s, w)] if ways.len() > 1 => Ok(format!("only secret {s} fits: {w} ways")),
        [(_, w), ..] if lit.len() == ways.len() && lit.iter().all(|&(_, u)| u == *w) => Ok(format!(
            "every secret fits, {w} {} each",
            if *w == 1 { "way" } else { "ways" }
        )),
        _ => Err(format!("a tally of {ways:?}")),
    }
}

/// A height's amounts of the secret, the first spin and the second: the
/// secret plus x of the first plus x × x of the second.
pub fn amounts(x: u32) -> [i64; 3] {
    let x = i64::from(x);
    [1, x, x * x]
}

/// `c` copies of trustee `x`'s amounts.
pub fn scaled(c: i64, x: u32) -> [i64; 3] {
    amounts(x).map(|t| c * t)
}

/// How many copies of each of the pair's heights hold the same amount of
/// the first spin: the ten-mark shortcut's weights, (5, −4), as copies;
/// an error unless their first spins match.
pub fn copies(ten: Dial) -> Result<[i64; 2], String> {
    let (u, w) = model::pair_shortcut(ten, PAIR).ok_or("no shortcut for the pair on ten marks")?;
    let c = [u, -w];
    if c[0] * amounts(PAIR[0])[1] != c[1] * amounts(PAIR[1])[1] || c.iter().any(|&k| k <= 0) {
        return Err(format!("copies {c:?} do not hold the same first spin"));
    }
    Ok(c)
}

/// The first row of copies less the second: the secret's, the first
/// spin's and the second spin's amounts left.
pub fn difference(c: [i64; 2]) -> [i64; 3] {
    let (a, b) = (scaled(c[0], PAIR[0]), scaled(c[1], PAIR[1]));
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// One column's term: the secret's in the first ("s", "5s"), the others
/// signed ("+ 4a", "− 20b"), or "0" where nothing is left.
pub fn term(c: i64, letter: char, first: bool) -> String {
    let amount = if c.abs() == 1 {
        String::new()
    } else {
        c.abs().to_string()
    };
    match (first, c) {
        (_, 0) => "0".to_owned(),
        (true, 1..) => format!("{amount}{letter}"),
        (_, 1..) => format!("+ {amount}{letter}"),
        _ => format!("\u{2212} {amount}{letter}"),
    }
}

/// A row's three terms.
pub fn terms(t: [i64; 3]) -> [String; 3] {
    [term(t[0], 's', true), term(t[1], 'a', false), term(t[2], 'b', false)]
}

/// Where the result's second-spin term lands on `dial`, and what the sum
/// is then: "− 20b lands on 0:" and "s"; "− 20b lands on 2b:" and "s + 2b".
pub fn landed(dial: Dial, b: i64) -> [String; 2] {
    let w = dial.wrap(b);
    let lands = if w == 0 { "0".to_owned() } else { format!("{w}b") };
    let sum = if w == 0 { "s".to_owned() } else { format!("s + {w}b") };
    [format!("{} lands on {lands}:", term(b, 'b', false)), sum]
}

/// The pair's sum on their cards, in three pieces: the weighted cards, the
/// products (and their sum where it is off the dial), and where it lands.
pub fn sum_pieces(dial: Dial, c: [i64; 2], cards: [u32; 2]) -> [String; 3] {
    let [p, q] = [c[0] * i64::from(cards[0]), c[1] * i64::from(cards[1])];
    let total = p - q;
    let signed = |n: i64| {
        if n < 0 {
            format!("\u{2212}{}", -n)
        } else {
            n.to_string()
        }
    };
    let on_dial = (0..i64::from(dial.marks())).contains(&total);
    [
        format!("{} \u{d7} {} \u{2212} {} \u{d7} {}", c[0], cards[0], c[1], cards[1]),
        if on_dial {
            format!("= {p} \u{2212} {q}")
        } else {
            format!("= {p} \u{2212} {q} = {}", signed(total))
        },
        if on_dial {
            format!("= {total}")
        } else {
            format!("\u{2192} {}", dial.wrap(total))
        },
    ]
}

/// The table's rows on `dial`: for each number of secrets left, the pairs
/// of trustees whose two shares always leave that many, whatever shares
/// they hold, fewest secrets first; an error if a pair's shares leave
/// different numbers, or the secrets left fit unequally.
pub type TrusteePairGroups = Vec<(usize, Vec<[u32; 2]>)>;

pub fn table(dial: Dial) -> Result<TrusteePairGroups, String> {
    let mut rows: Vec<(usize, Vec<[u32; 2]>)> = Vec::new();
    for pair in combinations(&TRUSTEES, 2) {
        let leaves: Vec<(usize, bool)> = model::what_shares_leave(dial, &pair, 2).into_iter().collect();
        let [(n, true)] = leaves[..] else {
            return Err(format!("{} marks: {pair:?}'s shares leave {leaves:?}", dial.marks()));
        };
        match rows.iter().position(|r| r.0 == n) {
            Some(i) => rows[i].1.push([pair[0], pair[1]]),
            None => rows.push((n, vec![[pair[0], pair[1]]])),
        }
    }
    rows.sort_by_key(|r| r.0);
    Ok(rows)
}

/// A table row: "1 secret: 2 and 5, 4 and 5", or "all 10: 1 and 3".
pub fn table_row(n: usize, marks: u32, pairs: &[[u32; 2]]) -> String {
    let names: Vec<String> = pairs.iter().map(|p| format!("{} and {}", p[0], p[1])).collect();
    let left = match n {
        _ if n == marks as usize => format!("all {n}"),
        1 => "1 secret".to_owned(),
        _ => format!("{n} secrets"),
    };
    format!("{left}: {}", names.join(", "))
}

/// The line under the table: every pair, on the vault's dial.
pub fn eleven_row(rows: &[(usize, Vec<[u32; 2]>)]) -> Result<String, String> {
    match rows {
        [(n, pairs)] if pairs.len() == combinations(&TRUSTEES, 2).len() => {
            Ok(format!("on {MARKS} marks, every pair: all {n}"))
        }
        _ => Err(format!("on {MARKS} marks the pairs leave {rows:?}")),
    }
}

/// When a viewer still reading until `busy` has also read `text`, fully in
/// now (padlocks.rs's rule: one argument's lines are read one after
/// another).
pub fn read_on(stage: &Stage<'_>, busy: f64, text: &str) -> f64 {
    read_by(stage, text).max(busy + text.chars().count() as f64 / READ_CPS)
}

// ---------------------------------------------------------------- drawing

/// Text left-aligned a space after `prev`, on its line. Detached.
pub fn text_after(kit: &Kit, stage: &mut Stage<'_>, prev: Mob, s: &str, size: f64, color: Srgb) -> fmn::Result<Mob> {
    let (at, w) = (stage.get_center(prev), stage.get_width(prev));
    kit.text_left(stage, s, size, color, v(at[0] + w / 2.0 + PIECE_GAP, at[1]))
}

/// An outline round all of `mobs` as they stand, in `color`. Detached.
fn around(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb) -> fmn::Result<Mob> {
    if mobs.is_empty() {
        return Err(fail("an outline round nothing".into()));
    }
    let (mut l, mut r, mut b, mut t) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &m in mobs {
        let (at, w, h) = (stage.get_center(m), stage.get_width(m), stage.get_height(m));
        l = l.min(at[0] - w / 2.0);
        r = r.max(at[0] + w / 2.0);
        b = b.min(at[1] - h / 2.0);
        t = t.max(at[1] + h / 2.0);
    }
    let centre = v((l + r) / 2.0, (b + t) / 2.0);
    outline(stage, centre, r - l + 2.0 * TEXT_PAD, t - b + 2.0 * TEXT_PAD, color)
}

/// A stroke through the words `m`, in `color`. Detached.
fn strike(stage: &mut Stage<'_>, m: Mob, color: Srgb) -> Mob {
    let (at, w) = (stage.get_center(m), stage.get_width(m));
    stroke(
        stage,
        vec![v(at[0] - w / 2.0 - 0.06, at[1]), v(at[0] + w / 2.0 + 0.06, at[1])],
        color,
        STRIKE_W,
    )
}

/// An outline round the row of heights on paper. Detached.
fn heights_outline(stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
    let (x0, x1) = (frame_at(TRUSTEES[0])[0], frame_at(TRUSTEES[TRUSTEES.len() - 1])[0]);
    outline(
        stage,
        v((x0 + x1) / 2.0, HEIGHTS_Y),
        x1 - x0 + FRAME_W + 2.0 * PAD,
        HEIGHT_SIZE * 0.0065 + 2.0 * TEXT_PAD,
        color,
    )
}

/// Row `row` of the derivation: its name and its three terms, the first
/// spin's "0" in red. Detached.
fn write_row(kit: &Kit, stage: &mut Stage<'_>, row: usize, name: &str, t: [i64; 3]) -> fmn::Result<(Mob, [Mob; 3])> {
    let name = kit.text_left(stage, name, ROW_NAME_SIZE, Palette::dim(), v(LEFT, ROW_Y[row]))?;
    let mut mobs = [name; 3];
    for (j, w) in terms(t).iter().enumerate() {
        let color = if j == 1 && t[1] == 0 {
            palette::leak()
        } else {
            Palette::ink()
        };
        mobs[j] = kit.text(stage, w, TERM_SIZE, color, v(COL_X[j], ROW_Y[row]))?;
    }
    Ok((name, mobs))
}

/// An outline round trustee `t`'s red frame. Detached.
fn red_frame_halo(stage: &mut Stage<'_>, t: u32, color: Srgb) -> fmn::Result<Mob> {
    outline(
        stage,
        v(frame_at(t)[0], TEN_FRAME_Y),
        FRAME_W + 2.0 * RED_FRAME_PAD,
        FRAME_H + 2.0 * RED_FRAME_PAD,
        color,
    )
}

/// One dial's tally, right: its title, a tick for each secret with its
/// mark under it, and "ways" left of where the bars stand. Detached.
struct Tally {
    rows: Rows,
    marks: u32,
    title: Mob,
    ticks: Vec<Mob>,
    labels: Vec<Mob>,
    ways: Mob,
}

impl Tally {
    fn build(kit: &Kit, stage: &mut Stage<'_>, marks: u32, held: &[(u32, u32)], rows: Rows) -> fmn::Result<Self> {
        let title = kit.text_left(
            stage,
            &tally_title(marks, held),
            TALLY_TITLE_SIZE,
            Palette::ink(),
            v(STRIP_LEFT, rows.title),
        )?;
        let (mut ticks, mut labels) = (Vec::new(), Vec::new());
        for k in 0..marks {
            let x = tick_at(k)[0];
            ticks.push(panel(
                stage,
                Palette::dim(),
                1.0,
                v(x, rows.tick),
                TICK_W,
                TALLY_TICK_H,
            )?);
            labels.push(kit.text(
                stage,
                &k.to_string(),
                TALLY_LABEL_SIZE,
                Palette::ink(),
                v(x, rows.label),
            )?);
        }
        let ways = text_right(
            kit,
            stage,
            WAYS,
            WAYS_SIZE,
            palette::spin(),
            v(STRIP_LEFT - WAYS_GAP, rows.base + WAYS_DY),
        )?;
        Ok(Self {
            rows,
            marks,
            title,
            ticks,
            labels,
            ways,
        })
    }

    fn parts(&self) -> Vec<Mob> {
        let mut all = vec![self.title, self.ways];
        all.extend(&self.ticks);
        all.extend(&self.labels);
        all
    }

    /// Secret `s`'s bar, `ways` high, and its count over it. Detached.
    fn bar(&self, kit: &Kit, stage: &mut Stage<'_>, s: u32, ways: usize) -> fmn::Result<(Mob, Mob)> {
        let (x, h) = (tick_at(s)[0], UNIT * ways as f64);
        let bar = panel(stage, palette::spin(), 0.9, v(x, self.rows.base + h / 2.0), BAR_W, h)?;
        let count = kit.text(
            stage,
            &ways.to_string(),
            COUNT_SIZE,
            palette::spin(),
            v(x, self.rows.base + h + COUNT_DY),
        )?;
        Ok((bar, count))
    }

    /// Bars growing up from their foot with their counts, prepared.
    fn rising(stage: &mut Stage<'_>, bars: &[(Mob, Mob)], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut anims = Vec::with_capacity(2 * bars.len());
        for &(bar, count) in bars {
            let a = fmn::animation::grow_from_edge(stage.arena_mut(), bar, DOWN, None)?;
            anims.push(stage.prepare(a.rt(rt))?);
            anims.extend(swaps(stage, &[], &[count], rt)?);
        }
        Ok(anims)
    }

    /// A ring round secret `s`'s tick. Detached.
    fn tick_ring(&self, stage: &mut Stage<'_>, s: u32, color: Srgb) -> Mob {
        ring(stage, v(tick_at(s)[0], self.rows.tick), TICK_RING, color)
    }

    /// An outline round secret `s`'s tick. Detached.
    fn tick_outline(&self, stage: &mut Stage<'_>, s: u32, color: Srgb) -> fmn::Result<Mob> {
        outline(
            stage,
            v(tick_at(s)[0], self.rows.tick),
            TICK_W + 2.0 * PAD,
            TALLY_TICK_H + 2.0 * PAD,
            color,
        )
    }

    /// An outline round every bar one way high. Detached.
    fn level_outline(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        let (x0, x1) = (tick_at(0)[0], tick_at(self.marks - 1)[0]);
        outline(
            stage,
            v((x0 + x1) / 2.0, self.rows.base + UNIT / 2.0),
            x1 - x0 + BAR_W + 2.0 * PAD,
            UNIT + 2.0 * PAD,
            color,
        )
    }

    /// Line `i` under the tally, left-aligned at the strip's left. Detached.
    fn line(&self, kit: &Kit, stage: &mut Stage<'_>, i: usize, s: &str, size: f64, color: Srgb) -> fmn::Result<Mob> {
        kit.text_left(stage, s, size, color, v(STRIP_LEFT, self.rows.lines[i]))
    }
}

impl SceneConstruct for ATenMarkDial {
    fn name(&self) -> &str {
        "12_a_ten_mark_dial"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let ten = Dial::new(TEN);
        let recipe = Recipe::example();
        // the same recipe on both dials; the pair's cards and tallies, which
        // must be level on eleven marks and the secret alone on ten
        let vault_cards = recipe.deal(dial, &TRUSTEES);
        let ten_cards = recipe.deal(ten, &TRUSTEES);
        let held = dealt(dial, &PAIR);
        let ten_held: Vec<(u32, u32)> = PAIR.iter().map(|&x| (x, recipe.share(ten, x))).collect();
        let eleven_ways = tally(dial, &held);
        let ten_ways = tally(ten, &ten_held);
        let eleven_line = fits_line(&eleven_ways).map_err(fail)?;
        let ten_line = fits_line(&ten_ways).map_err(fail)?;
        let found: Vec<u32> = ten.all().filter(|&s| ten_ways[s as usize] > 0).collect();
        if found != [SECRET] || eleven_ways.iter().any(|&w| w != 1) {
            return Err(fail(format!("tallies {eleven_ways:?} and {ten_ways:?}")));
        }
        let heights: Vec<i64> = TRUSTEES.iter().map(|&x| recipe.height(x)).collect();
        // the derivation: copies holding the same first spin, the result, and
        // the pair's sum on each dial's cards
        let c = copies(ten).map_err(fail)?;
        let left = difference(c);
        if left[0] != 1 || left[1] != 0 {
            return Err(fail(format!("the copies leave {left:?}")));
        }
        let ten_sum = sum_pieces(ten, c, [ten_held[0].1, ten_held[1].1]);
        let eleven_sum = sum_pieces(dial, c, [held[0].1, held[1].1]);
        let ten_landed = landed(ten, left[2]);
        let eleven_landed = landed(dial, left[2]);
        // the table: every pair on ten marks, and on the vault's dial
        let rows = table(ten).map_err(fail)?;
        let eleven_table = eleven_row(&table(dial).map_err(fail)?).map_err(fail)?;
        let row_texts: Vec<String> = rows.iter().map(|(n, p)| table_row(*n, TEN, p)).collect();
        let leaking: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].0 < TEN as usize).collect();
        let finding: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].0 == 1).collect();
        let silent: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].0 == TEN as usize).collect();
        if row_texts.len() != TABLE_Y.len() || silent.len() != 1 || finding.is_empty() {
            return Err(fail(format!("the ten-mark table is {rows:?}")));
        }

        // the opening: the rail with the vault's cards face up, the pair's
        // discs purple; over the frames the heights on paper, named
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        let mut cards = Vec::with_capacity(TRUSTEES.len());
        for (&t, k) in TRUSTEES.iter().zip(&vault_cards) {
            cards.push(card(&kit, stage, t, &k.to_string(), palette::share())?);
        }
        let mut height_mobs = Vec::with_capacity(heights.len());
        for (&t, h) in TRUSTEES.iter().zip(&heights) {
            height_mobs.push(kit.text(
                stage,
                &h.to_string(),
                HEIGHT_SIZE,
                Palette::ink(),
                v(frame_at(t)[0], HEIGHTS_Y),
            )?);
        }
        let heights_name = kit.text_left(stage, HEIGHTS_NAME, NAME_SIZE, Palette::dim(), v(NAME_X, HEIGHTS_Y))?;
        let eleven_name = kit.text_left(stage, ELEVEN_NAME, NAME_SIZE, Palette::dim(), v(NAME_X, frame_at(1)[1]))?;
        let mut opening = cards.clone();
        opening.extend(&height_mobs);
        opening.extend([heights_name, eleven_name]);
        let mut anims = swaps(stage, &[], &opening, 0.6)?;
        anims.extend(recolorings(stage, &rail.tint(&PAIR, palette::pair()), 0.6)?);
        stage.play_prepared(anims)?;

        // 12_1: why eleven? the vault's face, a red ten-mark face; the same
        // recipe on ten marks, and the question
        let spoken = say(&kit, stage, "12_1")?;
        spoken.wait_for(stage, "Why does the dial have eleven marks")?;
        let vault_face = Face::build(
            &kit,
            stage,
            dial,
            v(VAULT_FACE_X, FACE_Y),
            FACE_R,
            Palette::dim(),
            SECRET,
        )?;
        let vault_title = kit.text(
            stage,
            VAULT_TITLE,
            FACE_TITLE_SIZE,
            Palette::ink(),
            v(VAULT_FACE_X, FACE_TITLE_Y),
        )?;
        fade_in_all(stage, &[vault_title], 0.4)?;
        vault_face.show(stage, 0.6)?;
        spoken.wait_for(stage, "ten would seem the natural choice")?;
        let ten_face = Face::build(&kit, stage, ten, v(TEN_FACE_X, FACE_Y), FACE_R, palette::leak(), SECRET)?;
        let ten_title = kit.text(
            stage,
            TEN_TITLE,
            FACE_TITLE_SIZE,
            palette::leak(),
            v(TEN_FACE_X, FACE_TITLE_Y),
        )?;
        fade_in_all(stage, &[ten_title], 0.4)?;
        ten_face.show(stage, 0.6)?;
        spoken.wait_for(stage, "Deal the same secret with the same two spins")?;
        let o = heights_outline(stage, palette::spin())?;
        glow(stage, &[o], 0.9)?;
        spoken.wait_for(stage, "on a ten-mark dial")?;
        let mut red_frames = Vec::with_capacity(TRUSTEES.len() + 1);
        for &t in &TRUSTEES {
            red_frames.push(frame(
                stage,
                palette::leak(),
                v(frame_at(t)[0], TEN_FRAME_Y),
                FRAME_W,
                FRAME_H,
            )?);
        }
        red_frames.push(kit.text_left(stage, TEN_NAME, NAME_SIZE, palette::leak(), v(NAME_X, TEN_FRAME_Y))?);
        fade_in_all(stage, &red_frames, 0.5)?;
        spoken.wait_for(stage, "Would trustees four and five")?;
        let mut halos: Vec<Mob> = PAIR.iter().map(|&t| disc_ring(stage, t, palette::pair())).collect();
        halos.extend(frame_halos(stage, &PAIR, palette::pair())?);
        for &t in &PAIR {
            halos.push(red_frame_halo(stage, t, palette::pair())?);
        }
        glow(stage, &halos, 0.9)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 12_2: each height lands on its last digit; the pair would hold 5
        // and 5, not 2 and 6
        let spoken = say(&kit, stage, "12_2")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "each height simply lands on its last digit")?;
        let o = heights_outline(stage, palette::spin())?;
        glow(stage, &[o], 0.9)?;
        spoken.wait_for(stage, "nine, seven, nine, five and five")?;
        let mut drops = Vec::with_capacity(TRUSTEES.len());
        for (&t, k) in TRUSTEES.iter().zip(&ten_cards) {
            let m = kit.text(
                stage,
                &k.to_string(),
                CARD_SIZE,
                palette::leak(),
                v(frame_at(t)[0], TEN_FRAME_Y),
            )?;
            let a = fade_in(stage.arena_mut(), m, v(0.0, -DROP), 1.0)?;
            drops.push(stage.prepare(a)?);
        }
        let rt = (spoken.at("So trustees four and five")? - now(stage)).clamp(1.2, 2.4);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), drops, 0.6)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "would hold five and five")?;
        let mut halos: Vec<Mob> = PAIR.iter().map(|&t| disc_ring(stage, t, palette::pair())).collect();
        for &t in &PAIR {
            halos.push(red_frame_halo(stage, t, palette::leak())?);
        }
        glow(stage, &halos, 0.9)?;
        spoken.wait_for(stage, "not two and six")?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        glow(stage, &halos, 0.9)?;
        spoken.wait_end(stage)?;

        // 12_3: the experiment again, a tally for each dial: level on eleven
        // marks, the secret alone on ten
        let spoken = say(&kit, stage, "12_3")?;
        let mut faces: Vec<Mob> = vault_face.parts().to_vec();
        faces.extend(ten_face.parts());
        faces.extend([vault_title, ten_title]);
        let anims = leaving(stage, &faces, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "run the experiment again")?;
        let upper = Tally::build(&kit, stage, MARKS, &held, ELEVEN_ROWS)?;
        let lower = Tally::build(&kit, stage, TEN, &ten_held, TEN_ROWS)?;
        let mut parts = upper.parts();
        parts.extend(lower.parts());
        fade_in_all(stage, &parts, 0.7)?;
        spoken.wait_for(stage, "On eleven marks")?;
        let mut upper_bars = Vec::with_capacity(eleven_ways.len());
        for (s, &w) in (0..).zip(&eleven_ways) {
            upper_bars.push(upper.bar(&kit, stage, s, w)?);
        }
        let mut anims = Tally::rising(stage, &upper_bars, 0.8)?;
        let lit: Vec<(Mob, Srgb)> = upper.ticks.iter().map(|&t| (t, palette::secret())).collect();
        anims.extend(recolorings(stage, &lit, 0.8)?);
        stage.play_prepared(anims)?;
        let upper_fit = upper.line(&kit, stage, 0, &eleven_line, FIT_SIZE, Palette::ink())?;
        fade_in_all(stage, &[upper_fit], 0.4)?;
        spoken.wait_for(stage, "On ten marks")?;
        // secret by secret: each tick tried, and the one bar rising at 5
        let mut each = Vec::with_capacity(ten_ways.len());
        let mut lower_bar = None;
        for (s, &w) in (0..).zip(&ten_ways) {
            let r = lower.tick_ring(stage, s, palette::spin());
            let mut step = glowing(stage, &[r], 1.0)?;
            if w > 0 {
                let bar = lower.bar(&kit, stage, s, w)?;
                step.extend(Tally::rising(stage, &[bar], 1.0)?);
                lower_bar = Some(bar);
            }
            let g = AnimationGroup::new(stage.arena_mut(), step)?;
            each.push(stage.prepare(g)?);
        }
        let (bar5, _) = lower_bar.ok_or_else(|| fail("no bar on ten marks".into()))?;
        let rt = (spoken.at("and every one of them")? - now(stage)).clamp(1.6, 3.2);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.5)?.with_run_time(rt);
        stage.play(g)?;
        let faint: Vec<(Mob, Srgb)> = (0..)
            .zip(&lower.ticks)
            .map(|(s, &t)| {
                (
                    t,
                    if found.contains(&s) {
                        palette::secret()
                    } else {
                        palette::ghost()
                    },
                )
            })
            .collect();
        let mut anims = recolorings(stage, &faint, 0.6)?;
        let lower_fit = lower.line(&kit, stage, 0, &ten_line, FIT_SIZE, Palette::ink())?;
        anims.extend(swaps(stage, &[], &[lower_fit], 0.6)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "has the secret five")?;
        let anims = vec![swelling(stage, bar5, 1.15, palette::spin(), 0.8)?];
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "These two could work out the secret alone")?;
        let mut halos: Vec<Mob> = PAIR.iter().map(|&t| disc_ring(stage, t, palette::pair())).collect();
        halos.push(lower.tick_outline(stage, SECRET, palette::leak())?);
        glow(stage, &halos, 1.0)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_TALLIES)?;

        // 12_4: the pair's heights in letters, like terms under like terms
        let spoken = say(&kit, stage, "12_4")?;
        spoken.wait_for(stage, "Write their heights out")?;
        let legend = kit.text_left(stage, LEGEND, LEGEND_SIZE, Palette::dim(), v(LEFT, LEGEND_Y))?;
        fade_in_all(stage, &[legend], 0.5)?;
        let mut derivation = vec![legend];
        let mut row_terms: Vec<[Mob; 3]> = Vec::with_capacity(ROW_Y.len());
        for (row, (&x, [own, first, second])) in PAIR
            .iter()
            .zip([
                [
                    "Trustee four's is",
                    "plus four of the first spin",
                    "plus sixteen of the second",
                ],
                [
                    "Trustee five's is",
                    "plus five of the first",
                    "plus twenty-five of the second",
                ],
            ])
            .enumerate()
        {
            spoken.wait_for(stage, own)?;
            let (name, mobs) = write_row(&kit, stage, row, &format!("trustee {x}:"), amounts(x))?;
            let i = TRUSTEES
                .iter()
                .position(|&t| t == x)
                .ok_or_else(|| fail(format!("no trustee {x}")))?;
            let halo = text_halo(stage, height_mobs[i], palette::pair())?;
            let mut anims = swaps(stage, &[], &[name, mobs[0]], 0.4)?;
            anims.extend(glowing(stage, &[halo], 0.9)?);
            stage.play_prepared(anims)?;
            spoken.wait_for(stage, first)?;
            fade_in_all(stage, &[mobs[1]], 0.4)?;
            spoken.wait_for(stage, second)?;
            fade_in_all(stage, &[mobs[2]], 0.4)?;
            derivation.push(name);
            derivation.extend(mobs);
            row_terms.push(mobs);
        }
        spoken.wait_end(stage)?;

        // 12_5: copies holding the same amount of the first spin; one taken
        // from the other, the first spin cancels
        let spoken = say(&kit, stage, "12_5")?;
        let mut amount_halos = Vec::with_capacity(PAIR.len());
        for (row, (&x, (&k, cue))) in PAIR
            .iter()
            .zip(
                c.iter()
                    .zip(["Five copies of trustee four's height", "four copies of trustee five's"]),
            )
            .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            let (name, mobs) = write_row(&kit, stage, row + 2, &format!("{k} \u{d7} trustee {x}:"), scaled(k, x))?;
            let rt = if row == 0 {
                (spoken.at("hold twenty")? - now(stage)).clamp(0.5, 1.2)
            } else {
                0.8
            };
            let mut all = vec![name];
            all.extend(mobs);
            let mut steps = Vec::with_capacity(all.len());
            for &m in &all {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                steps.push(stage.prepare(a)?);
            }
            let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), steps, 0.5)?.with_run_time(rt);
            stage.play(g)?;
            if row == 0 {
                spoken.wait_for(stage, "hold twenty")?;
            }
            let halo = text_halo(stage, mobs[1], palette::spin())?;
            fade_in_all(stage, &[halo], 0.4)?;
            amount_halos.push(halo);
            derivation.push(name);
            derivation.extend(mobs);
            row_terms.push(mobs);
        }
        spoken.wait_for(stage, "Take one from the other")?;
        let rule = stroke(
            stage,
            vec![v(COL_X[0] - MINUS_DX - 0.2, RULE_Y), v(COL_X[2] + 0.55, RULE_Y)],
            Palette::dim(),
            RULE_W,
        );
        let minus = kit.text(
            stage,
            "\u{2212}",
            TERM_SIZE,
            Palette::ink(),
            v(COL_X[0] - MINUS_DX, ROW_Y[3]),
        )?;
        create_lagged(stage, &[rule], 0.5, 0.0)?;
        fade_in_all(stage, &[minus], 0.3)?;
        derivation.extend([rule, minus]);
        spoken.wait_for(stage, "the first spin cancels")?;
        let strikes = [
            strike(stage, row_terms[2][1], palette::leak()),
            strike(stage, row_terms[3][1], palette::leak()),
        ];
        create_lagged(stage, &strikes, 0.6, 0.4)?;
        let (result_name, result) = write_row(&kit, stage, 4, "first \u{2212} second:", left)?;
        let mut anims = leaving(stage, &amount_halos, ORIGIN, 0.4)?;
        anims.extend(swaps(stage, &[], &[result[1]], 0.5)?);
        stage.play_prepared(anims)?;
        derivation.extend(strikes);
        spoken.wait_for(stage, "What's left is the secret")?;
        fade_in_all(stage, &[result_name, result[0]], 0.5)?;
        spoken.wait_for(stage, "minus twenty of the second spin")?;
        fade_in_all(stage, &[result[2]], 0.5)?;
        derivation.push(result_name);
        derivation.extend(result);
        spoken.wait_end(stage)?;

        // 12_6: on ten marks twenty lands on zero: the pair's sum is the secret
        let spoken = say(&kit, stage, "12_6")?;
        spoken.wait_for(stage, "twenty of anything lands on zero")?;
        let halo = text_halo(stage, result[2], palette::leak())?;
        let ten_a = lower.line(&kit, stage, 1, &ten_landed[0], SUM_SIZE, palette::leak())?;
        let mut anims = glowing(stage, &[halo], 1.0)?;
        anims.extend(swaps(stage, &[], &[ten_a], 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "leaves the secret bare")?;
        let bare = strike(stage, result[2], palette::leak());
        let ten_s = text_after(&kit, stage, ten_a, &ten_landed[1], SUM_SIZE, palette::leak())?;
        create_lagged(stage, &[bare], 0.5, 0.0)?;
        fade_in_all(stage, &[ten_s], 0.4)?;
        derivation.push(bare);
        spoken.wait_for(stage, "five times five")?;
        let ten_b = lower.line(&kit, stage, 2, &ten_sum[0], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[ten_b], 0.4)?;
        spoken.wait_for(stage, "is twenty-five minus twenty")?;
        let ten_c = text_after(&kit, stage, ten_b, &ten_sum[1], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[ten_c], 0.4)?;
        spoken.wait_for(stage, "which is five")?;
        let ten_d = text_after(&kit, stage, ten_c, &ten_sum[2], SUM_SIZE, Palette::ink())?;
        let r = lower.tick_ring(stage, SECRET, palette::leak());
        let mut anims = swaps(stage, &[], &[ten_d], 0.4)?;
        anims.push(swelling(stage, bar5, 1.15, palette::leak(), 0.8)?);
        anims.extend(glowing(stage, &[r], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;

        // 12_7: on eleven marks −20 lands on 2: the same sum hides the secret
        let spoken = say(&kit, stage, "12_7")?;
        spoken.wait_for(stage, "minus twenty lands on two")?;
        let halo = text_halo(stage, result[2], palette::pair())?;
        let eleven_a = upper.line(&kit, stage, 1, &eleven_landed[0], SUM_SIZE, Palette::ink())?;
        let mut anims = glowing(stage, &[halo], 1.0)?;
        anims.extend(swaps(stage, &[], &[eleven_a], 0.5)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "the secret plus twice the second spin")?;
        let eleven_s = text_after(&kit, stage, eleven_a, &eleven_landed[1], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[eleven_s], 0.4)?;
        spoken.wait_for(stage, "five times two")?;
        let eleven_b = upper.line(&kit, stage, 2, &eleven_sum[0], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[eleven_b], 0.4)?;
        spoken.wait_for(stage, "is minus fourteen")?;
        let eleven_c = text_after(&kit, stage, eleven_b, &eleven_sum[1], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[eleven_c], 0.4)?;
        spoken.wait_for(stage, "which lands on eight")?;
        let eleven_d = text_after(&kit, stage, eleven_c, &eleven_sum[2], SUM_SIZE, Palette::ink())?;
        fade_in_all(stage, &[eleven_d], 0.4)?;
        spoken.wait_for(stage, "Every secret makes eight")?;
        let level = upper.level_outline(stage, palette::spin())?;
        glow(stage, &[level], 1.1)?;
        spoken.wait_end(stage)?;

        // 12_8: every pair on ten marks, counted: the table, read in full
        let spoken = say(&kit, stage, "12_8")?;
        let anims = leaving(stage, &derivation, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "On ten marks")?;
        let title = kit.text_left(
            stage,
            TABLE_TITLE,
            TABLE_TITLE_SIZE,
            Palette::ink(),
            v(LEFT, TABLE_TITLE_Y),
        )?;
        let mut table_mobs = vec![title];
        for (text, &y) in row_texts.iter().zip(&TABLE_Y) {
            table_mobs.push(kit.text_left(stage, text, TABLE_SIZE, Palette::ink(), v(LEFT, y))?);
        }
        let note = kit.text_left(
            stage,
            TABLE_NOTE,
            TABLE_NOTE_SIZE,
            Palette::dim(),
            v(LEFT, TABLE_NOTE_Y),
        )?;
        let eleven = kit.text_left(
            stage,
            &eleven_table,
            TABLE_SIZE,
            Palette::ink(),
            v(LEFT, TABLE_ELEVEN_Y),
        )?;
        table_mobs.extend([note, eleven]);
        let rt = (spoken.at("nine of the ten pairs")? - now(stage)).clamp(1.2, 2.4);
        let mut steps = Vec::with_capacity(table_mobs.len());
        for &m in &table_mobs {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            steps.push(stage.prepare(a)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), steps, 0.4)?.with_run_time(rt);
        stage.play(g)?;
        // read one line after another from here, the title first
        let mut read = now(stage);
        for text in std::iter::once(TABLE_TITLE)
            .chain(row_texts.iter().map(String::as_str))
            .chain([TABLE_NOTE, eleven_table.as_str()])
        {
            read = read_on(stage, read, text);
        }
        let row_mobs = &table_mobs[1..=row_texts.len()];
        spoken.wait_for(stage, "nine of the ten pairs")?;
        let picked: Vec<Mob> = leaking.iter().map(|&i| row_mobs[i]).collect();
        let o = around(stage, &picked, palette::leak())?;
        glow(stage, &[o], 1.1)?;
        spoken.wait_for(stage, "two pairs always find it")?;
        let picked: Vec<Mob> = finding.iter().map(|&i| row_mobs[i]).collect();
        let o = around(stage, &picked, palette::leak())?;
        glow(stage, &[o], 1.0)?;
        spoken.wait_for(stage, "Only trustees one and three")?;
        let o = around(stage, &[row_mobs[silent[0]]], palette::secret())?;
        glow(stage, &[o], 1.1)?;
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

    use super::super::draw::{RAIL_DX, THINK_R, strip_right};
    use super::*;
    use crate::facts;
    use crate::model::csv;

    /// The production narration: registering chapter 12 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// A share as the definition writes it, the secret plus each spin
    /// times the position's power, wrapped onto `p` marks: apart from the
    /// model's Horner rule.
    fn share(s: u32, a: u32, b: u32, x: u32, p: u32) -> u32 {
        (s + a * x + b * x * x) % p
    }

    /// The pair's tally on `p` marks for their cards `y`, the spin pairs
    /// counted with the definition written out.
    fn counted(p: u32, y: [u32; 2]) -> Vec<usize> {
        (0..p)
            .map(|s| {
                (0..p)
                    .flat_map(|a| (0..p).map(move |b| (a, b)))
                    .filter(|&(a, b)| share(s, a, b, PAIR[0], p) == y[0] && share(s, a, b, PAIR[1], p) == y[1])
                    .count()
            })
            .collect()
    }

    /// Every phrase the scene waits for is in its line of the script, once,
    /// and they come in the order they are said.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let phrases = [
            ("12_1", "Why does the dial have eleven marks"),
            ("12_1", "ten would seem the natural choice"),
            ("12_1", "Deal the same secret with the same two spins"),
            ("12_1", "on a ten-mark dial"),
            ("12_1", "Would trustees four and five"),
            ("12_2", "each height simply lands on its last digit"),
            ("12_2", "nine, seven, nine, five and five"),
            ("12_2", "So trustees four and five"),
            ("12_2", "would hold five and five"),
            ("12_2", "not two and six"),
            ("12_3", "run the experiment again"),
            ("12_3", "On eleven marks"),
            ("12_3", "On ten marks"),
            ("12_3", "and every one of them"),
            ("12_3", "has the secret five"),
            ("12_3", "These two could work out the secret alone"),
            ("12_4", "Write their heights out"),
            ("12_4", "Trustee four's is"),
            ("12_4", "plus four of the first spin"),
            ("12_4", "plus sixteen of the second"),
            ("12_4", "Trustee five's is"),
            ("12_4", "plus five of the first"),
            ("12_4", "plus twenty-five of the second"),
            ("12_5", "Five copies of trustee four's height"),
            ("12_5", "hold twenty"),
            ("12_5", "four copies of trustee five's"),
            ("12_5", "Take one from the other"),
            ("12_5", "the first spin cancels"),
            ("12_5", "What's left is the secret"),
            ("12_5", "minus twenty of the second spin"),
            ("12_6", "twenty of anything lands on zero"),
            ("12_6", "leaves the secret bare"),
            ("12_6", "five times five"),
            ("12_6", "is twenty-five minus twenty"),
            ("12_6", "which is five"),
            ("12_7", "minus twenty lands on two"),
            ("12_7", "the secret plus twice the second spin"),
            ("12_7", "five times two"),
            ("12_7", "is minus fourteen"),
            ("12_7", "which lands on eight"),
            ("12_7", "Every secret makes eight"),
            ("12_8", "On ten marks"),
            ("12_8", "nine of the ten pairs"),
            ("12_8", "two pairs always find it"),
            ("12_8", "Only trustees one and three"),
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
    }

    /// What the scene draws and writes is the facts': the cards on both
    /// dials, the tallies and their lines, every row of the derivation, the
    /// pair's sums, and the table.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let get = |k: &str| f.get(k).unwrap_or_else(|| panic!("no fact {k}")).to_owned();
        let (dial, ten) = (Dial::vault(), Dial::new(TEN));
        let recipe = Recipe::example();
        assert_eq!(get("S-F6.shares"), csv(&recipe.deal(dial, &TRUSTEES)));
        assert_eq!(get("S-F8.ten.shares"), csv(&recipe.deal(ten, &TRUSTEES)));
        let heights: Vec<i64> = TRUSTEES.iter().map(|&x| recipe.height(x)).collect();
        assert_eq!(get("S-F6.heights"), csv(&heights));
        let held = dealt(dial, &PAIR);
        let ten_held: Vec<(u32, u32)> = PAIR.iter().map(|&x| (x, recipe.share(ten, x))).collect();
        assert_eq!(
            get("S-F8.ten.pair_shares"),
            format!("{},{}", ten_held[0].1, ten_held[1].1)
        );
        assert_eq!(get("S-F8.eleven.pair_ways"), csv(&tally(dial, &held)));
        assert_eq!(get("S-F8.ten.pair_ways"), csv(&tally(ten, &ten_held)));
        assert_eq!(
            get("S-F8.ten.pair_spin_pairs"),
            tally(ten, &ten_held).iter().sum::<usize>().to_string()
        );
        assert_eq!(tally_title(MARKS, &held), "11 marks: trustees 4 and 5 hold 2 and 6");
        assert_eq!(tally_title(TEN, &ten_held), "10 marks: trustees 4 and 5 hold 5 and 5");
        assert_eq!(fits_line(&tally(dial, &held)).unwrap(), "every secret fits, 1 way each");
        assert_eq!(
            fits_line(&tally(ten, &ten_held)).unwrap(),
            "only secret 5 fits: 10 ways"
        );
        // the derivation
        let c = copies(ten).unwrap();
        for (x, k) in PAIR.iter().zip(c) {
            assert_eq!(get(&format!("S-F8.pair_sum.height.{x}")), csv(&amounts(*x)));
            assert_eq!(get(&format!("S-F8.pair_sum.copies.{x}")), format!("{k},{x}"));
            assert_eq!(get(&format!("S-F8.pair_sum.scaled.{x}")), csv(&scaled(k, *x)));
        }
        assert_eq!(get("S-F8.pair_sum.first_spin"), scaled(c[0], PAIR[0])[1].to_string());
        let left = difference(c);
        assert_eq!(get("S-F8.pair_sum.first_spin_left"), left[1].to_string());
        assert_eq!(get("S-F8.pair_sum.spin_terms"), csv(&[left[2], left[1], left[0]]));
        assert_eq!(terms(amounts(4)), ["s", "+ 4a", "+ 16b"]);
        assert_eq!(terms(amounts(5)), ["s", "+ 5a", "+ 25b"]);
        assert_eq!(terms(scaled(5, 4)), ["5s", "+ 20a", "+ 80b"]);
        assert_eq!(terms(scaled(4, 5)), ["4s", "+ 20a", "+ 100b"]);
        assert_eq!(terms(left), ["s", "0", "\u{2212} 20b"]);
        // where −20b lands, and the sums on the cards
        assert_eq!(
            get("S-F8.ten.twenty_lands"),
            format!("{},{}", -left[2], ten.wrap(-left[2]))
        );
        assert_eq!(
            get("S-F8.eleven.minus_twenty_lands"),
            format!("{},{}", left[2], dial.wrap(left[2]))
        );
        assert_eq!(landed(ten, left[2]), ["\u{2212} 20b lands on 0:", "s"]);
        assert_eq!(landed(dial, left[2]), ["\u{2212} 20b lands on 2b:", "s + 2b"]);
        assert_eq!(get("S-F8.eleven.leftover"), dial.wrap(left[2]).to_string());
        assert_eq!(get("S-F8.ten.leftover"), ten.wrap(left[2]).to_string());
        assert_eq!(get("S-F8.ten.pair_formula_terms"), "5,5,-4,5");
        assert_eq!(
            sum_pieces(ten, c, [5, 5]),
            ["5 \u{d7} 5 \u{2212} 4 \u{d7} 5", "= 25 \u{2212} 20", "= 5"]
        );
        assert_eq!(get("S-F8.ten.pair_formula_products"), "25,-20");
        assert_eq!(get("S-F8.ten.pair_formula_value"), "5");
        assert_eq!(get("S-F8.eleven.pair_formula_terms"), "5,2,-4,6");
        assert_eq!(
            sum_pieces(dial, c, [2, 6]),
            [
                "5 \u{d7} 2 \u{2212} 4 \u{d7} 6",
                "= 10 \u{2212} 24 = \u{2212}14",
                "\u{2192} 8"
            ]
        );
        assert_eq!(get("S-F8.eleven.pair_formula_products"), "10,-24");
        assert_eq!(get("S-F8.eleven.pair_formula_unwrapped"), "-14");
        assert_eq!(get("S-F8.eleven.pair_formula_value"), "8");
        // the table
        let rows = table(ten).unwrap();
        for (n, pairs) in &rows {
            let names: Vec<String> = pairs.iter().map(|p| format!("{}-{}", p[0], p[1])).collect();
            assert_eq!(get(&format!("S-F8.ten.pairs_left.{n}")), names.join(","));
        }
        let texts: Vec<String> = rows.iter().map(|(n, p)| table_row(*n, TEN, p)).collect();
        assert_eq!(
            texts,
            [
                "1 secret: 2 and 5, 4 and 5",
                "2 secrets: 1 and 5, 3 and 5",
                "5 secrets: 1 and 2, 1 and 4, 2 and 3, 2 and 4, 3 and 4",
                "all 10: 1 and 3",
            ]
        );
        let leak = rows
            .iter()
            .filter(|r| r.0 < TEN as usize)
            .map(|r| r.1.len())
            .sum::<usize>();
        assert_eq!(get("S-F8.ten.pairs_that_leak"), leak.to_string());
        assert_eq!(get("S-F8.ten.pairs_that_find"), rows[0].1.len().to_string());
        assert_eq!(get("S-F8.ten.pair_that_learns_nothing"), csv(&rows[3].1[0]));
        assert_eq!(
            get("S-F7.all_pairs.pairs"),
            combinations(&TRUSTEES, 2).len().to_string()
        );
        assert_eq!(
            eleven_row(&table(dial).unwrap()).unwrap(),
            "on 11 marks, every pair: all 11"
        );
        assert_eq!(get("S-F7.all_pairs.two_leave"), "2,11");
    }

    /// The tallies counted again with the definition written out, apart
    /// from the model, and the shortcut checked on every recipe: on ten
    /// marks, whatever two cards the pair holds, the only secret is
    /// 5 × y4 − 4 × y5, ten ways, so a wrong card picks a different secret
    /// (the control: 6 and 5 find 0); on paper 5 × h4 − 4 × h5 is the secret
    /// less 20 of the second spin for every recipe. The wrong-modulus
    /// controls: twelve marks leave three secrets, which the scene's line
    /// refuses, and nine marks a level tally.
    #[test]
    fn the_tallies_and_the_shortcut_hold_for_every_card() {
        assert_eq!(counted(MARKS, [2, 6]), vec![1; 11]);
        assert_eq!(counted(TEN, [5, 5]), vec![0, 0, 0, 0, 0, 10, 0, 0, 0, 0]);
        let ten = Dial::new(TEN);
        assert_eq!(counted(TEN, [5, 5]), tally(ten, &[(4, 5), (5, 5)]));
        for y4 in 0..TEN {
            for y5 in 0..TEN {
                let want = (5 * i64::from(y4) - 4 * i64::from(y5)).rem_euclid(10) as usize;
                let ways = counted(TEN, [y4, y5]);
                assert_eq!(ways.iter().filter(|&&w| w > 0).count(), 1, "{y4},{y5}");
                assert_eq!(ways[want], 10, "{y4},{y5}");
            }
        }
        assert_eq!(fits_line(&counted(TEN, [6, 5])).unwrap(), "only secret 0 fits: 10 ways");
        for s in 0..MARKS {
            for a in 0..MARKS {
                for b in 0..MARKS {
                    let r = Recipe::new(s, &[a, b]);
                    let (h4, h5) = (r.height(4), r.height(5));
                    assert_eq!(5 * h4 - 4 * h5, i64::from(s) - 20 * i64::from(b));
                }
            }
        }
        let twelve = counted(12, [1, 11]);
        assert_eq!(twelve.iter().filter(|&&w| w > 0).count(), 3);
        assert!(fits_line(&twelve).is_err());
        assert_eq!(fits_line(&counted(9, [1, 8])).unwrap(), "every secret fits, 1 way each");
        assert_eq!(Dial::new(12).wrap(-20), 4);
        assert_eq!(landed(Dial::new(12), -20)[1], "s + 4b");
    }

    /// The table, counted: on ten marks every pair's shares always leave
    /// the same number of secrets, each fitting equally often, and only
    /// trustees 1 and 3 keep all ten; on eleven marks every pair keeps all
    /// eleven. The controls: twelve marks give another table, and a pair's
    /// shares counted by hand agree with the model's grouping.
    #[test]
    fn the_table_is_counted() {
        let ten = Dial::new(TEN);
        let rows = table(ten).unwrap();
        assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2, 5, 10]);
        assert_eq!(rows.iter().map(|r| r.1.len()).sum::<usize>(), 10);
        assert_eq!(rows[3].1, vec![[1, 3]]);
        let eleven = table(Dial::vault()).unwrap();
        assert_eq!(eleven.len(), 1);
        assert_eq!(eleven[0].0, 11);
        assert_ne!(table(Dial::new(12)), Ok(rows.clone()));
        // trustees 1 and 3 by hand: every pair of cards some recipe deals
        // leaves all ten secrets, ten ways each
        for y1 in 0..TEN {
            for y3 in 0..TEN {
                let ways: Vec<usize> = (0..TEN)
                    .map(|s| {
                        (0..TEN)
                            .flat_map(|a| (0..TEN).map(move |b| (a, b)))
                            .filter(|&(a, b)| share(s, a, b, 1, TEN) == y1 && share(s, a, b, 3, TEN) == y3)
                            .count()
                    })
                    .collect();
                assert!(ways.iter().all(|&w| w == ways[0]), "{y1},{y3}: {ways:?}");
            }
        }
        assert!(eleven_row(&rows).is_err());
    }

    /// The written pieces at their edges: a term of one is the letter
    /// alone, nothing left is "0", and a sum already on the dial is not
    /// wound.
    #[test]
    fn terms_and_sums_read_as_written() {
        assert_eq!(term(1, 'a', false), "+ a");
        assert_eq!(term(-1, 'b', false), "\u{2212} b");
        assert_eq!(term(0, 'a', false), "0");
        assert_eq!(term(3, 's', true), "3s");
        assert_eq!(sum_pieces(Dial::vault(), [5, 4], [5, 5])[2], "= 5");
        assert_eq!(sum_pieces(Dial::vault(), [5, 4], [9, 1])[2], "\u{2192} 8");
        assert!(fits_line(&[0, 0, 0]).is_err());
        assert!(fits_line(&[1, 2, 1]).is_err());
    }

    /// The pieces keep apart, inside the frame: the faces and the
    /// derivation upper left, the heights, the red frames and the rail's
    /// frames in rows, their names right of them, the two tallies right on
    /// the strip's columns, their bars clear of their titles, and the table
    /// in the derivation's place.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.3;
        let top = FRAME_Y_RADIUS - 0.3;
        // the faces: inside the frame, apart, over the heights
        const {
            assert!(VAULT_FACE_X - FACE_R > -FRAME_X_RADIUS + 0.3);
            assert!(TEN_FACE_X - VAULT_FACE_X > 2.0 * FACE_R + 0.5);
            assert!(FACE_Y - FACE_R > HEIGHTS_Y + 0.6);
            assert!(FACE_TITLE_Y + FACE_TITLE_SIZE * 0.0065 / 2.0 < FRAME_Y_RADIUS - 0.3);
            assert!(FACE_TITLE_Y - FACE_TITLE_SIZE * 0.0065 / 2.0 > FACE_Y + FACE_R + 0.1);
            assert!(TEN_FACE_X + FACE_R < STRIP_LEFT - 1.0);
        }
        // the rows over the rail: heights, red frames, the rail's frames
        let rail_frame_top = frame_at(1)[1] + FRAME_H / 2.0 + RED_FRAME_PAD;
        assert!(TEN_FRAME_Y - FRAME_H / 2.0 - RED_FRAME_PAD > rail_frame_top + 0.05);
        assert!(HEIGHTS_Y - text_h(HEIGHT_SIZE) / 2.0 - TEXT_PAD > TEN_FRAME_Y + FRAME_H / 2.0 + RED_FRAME_PAD);
        assert!(text_w("215", HEIGHT_SIZE) < RAIL_DX - 0.2);
        // the names right of the last frame, left of "ways"
        assert!(NAME_X > frame_at(5)[0] + FRAME_W / 2.0 + RED_FRAME_PAD + 0.1);
        for name in [HEIGHTS_NAME, TEN_NAME, ELEVEN_NAME] {
            assert!(NAME_X + text_w(name, NAME_SIZE) < STRIP_LEFT - WAYS_GAP - text_w(WAYS, WAYS_SIZE) - 0.1);
        }
        // the tallies: titles over the tallest bar's count, ticks under the
        // bars, labels and lines under the ticks, the upper over the lower
        for (rows, ways, marks) in [(ELEVEN_ROWS, 1.0, MARKS), (TEN_ROWS, 10.0, TEN)] {
            let count_top = rows.base + UNIT * ways + COUNT_DY + text_h(COUNT_SIZE) / 2.0;
            assert!(
                count_top < rows.title - text_h(TALLY_TITLE_SIZE) / 2.0 - 0.1,
                "{rows:?}"
            );
            assert!(rows.tick + TALLY_TICK_H / 2.0 < rows.base);
            assert!(rows.label + text_h(TALLY_LABEL_SIZE) / 2.0 < rows.tick - TALLY_TICK_H / 2.0);
            assert!(rows.lines[0] + text_h(FIT_SIZE) / 2.0 < rows.label - text_h(TALLY_LABEL_SIZE) / 2.0);
            for w in rows.lines.windows(2) {
                assert!(w[0] - w[1] > text_h(SUM_SIZE) + 0.15);
            }
            assert!(strip_right(marks) < edge);
            assert!(STRIP_LEFT + text_w(&tally_title(marks, &[(4, 5), (5, 5)]), TALLY_TITLE_SIZE) < edge);
        }
        assert!(ELEVEN_ROWS.title + text_h(TALLY_TITLE_SIZE) / 2.0 < top);
        assert!(ELEVEN_ROWS.lines[2] - text_h(SUM_SIZE) / 2.0 > TEN_ROWS.title + text_h(TALLY_TITLE_SIZE) / 2.0 + 0.1);
        assert!(TEN_ROWS.lines[2] - text_h(SUM_SIZE) / 2.0 > -FRAME_Y_RADIUS + 0.3);
        assert!(BAR_W < tick_at(1)[0] - tick_at(0)[0] - 0.1);
        // the longest lines under the tallies
        for line in [
            "\u{2212} 20b lands on 0: s",
            "5 \u{d7} 5 \u{2212} 4 \u{d7} 5 = 25 \u{2212} 20 = 5",
            "\u{2212} 20b lands on 2b: s + 2b",
            "5 \u{d7} 2 \u{2212} 4 \u{d7} 6 = 10 \u{2212} 24 = \u{2212}14 \u{2192} 8",
        ] {
            assert!(STRIP_LEFT + text_w(line, SUM_SIZE) + 3.0 * PIECE_GAP < edge, "{line}");
        }
        // the think ring right, clear of the faces and the rows' names
        const {
            assert!(THINK_X + THINK_R < FRAME_X_RADIUS - 0.3);
            assert!(THINK_X - THINK_R > STRIP_LEFT);
            assert!(THINK_Y - THINK_R - 0.3 > HEIGHTS_Y);
        }
        // the derivation: inside, its columns apart, over the heights, left
        // of the tallies' "ways"
        assert!(LEGEND_Y + text_h(LEGEND_SIZE) / 2.0 < top);
        assert!(LEFT + text_w(LEGEND, LEGEND_SIZE) < COL_X[2] + 0.5);
        assert!(LEGEND_Y - text_h(LEGEND_SIZE) / 2.0 > ROW_Y[0] + text_h(TERM_SIZE) / 2.0 + 0.1);
        let widest_name = text_w("first \u{2212} second:", ROW_NAME_SIZE);
        assert!(LEFT + widest_name < COL_X[0] - MINUS_DX - text_w("\u{2212}", TERM_SIZE) - 0.1);
        assert!(COL_X[0] + text_w("5s", TERM_SIZE) / 2.0 < COL_X[1] - text_w("+ 20a", TERM_SIZE) / 2.0 - 0.1);
        assert!(COL_X[1] + text_w("+ 20a", TERM_SIZE) / 2.0 < COL_X[2] - text_w("+ 100b", TERM_SIZE) / 2.0 - 0.1);
        assert!(COL_X[2] + text_w("+ 100b", TERM_SIZE) / 2.0 < STRIP_LEFT - WAYS_GAP - text_w(WAYS, WAYS_SIZE));
        for w in ROW_Y.windows(2) {
            assert!(w[0] - w[1] > text_h(TERM_SIZE) + 0.2);
        }
        assert!(ROW_Y[3] - text_h(TERM_SIZE) / 2.0 > RULE_Y + 0.1);
        assert!(ROW_Y[4] + text_h(TERM_SIZE) / 2.0 < RULE_Y - 0.1);
        assert!(ROW_Y[4] - text_h(TERM_SIZE) / 2.0 > HEIGHTS_Y + text_h(HEIGHT_SIZE) / 2.0 + 0.3);
        // the table in the derivation's place, its rows apart and inside
        assert!(TABLE_TITLE_Y + text_h(TABLE_TITLE_SIZE) / 2.0 < top);
        let rows = table(Dial::new(TEN)).unwrap();
        for (n, p) in &rows {
            let text = table_row(*n, TEN, p);
            assert!(
                LEFT + text_w(&text, TABLE_SIZE) < STRIP_LEFT - WAYS_GAP - text_w(WAYS, WAYS_SIZE),
                "{text}"
            );
        }
        assert!(LEFT + text_w(TABLE_TITLE, TABLE_TITLE_SIZE) < STRIP_LEFT - 0.2);
        assert!(LEFT + text_w(TABLE_NOTE, TABLE_NOTE_SIZE) < STRIP_LEFT - 0.2);
        let ys: Vec<f64> = std::iter::once(TABLE_TITLE_Y)
            .chain(TABLE_Y)
            .chain([TABLE_NOTE_Y, TABLE_ELEVEN_Y])
            .collect();
        for w in ys.windows(2) {
            assert!(w[0] - w[1] > text_h(TABLE_TITLE_SIZE) + 0.2);
        }
        assert!(TABLE_ELEVEN_Y - text_h(TABLE_SIZE) / 2.0 > HEIGHTS_Y + text_h(HEIGHT_SIZE) / 2.0 + 0.3);
    }
}
