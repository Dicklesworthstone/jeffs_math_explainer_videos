//! Chapter 07, Bayes' rule: the move of chapter 06 gets its name (07_1);
//! as a recipe, a table built a column a sentence: each cell's chance, times
//! the chance the search would come back empty if the wreck were there,
//! then divided by what is left, eighty (07_2); "your turn": where do you
//! search next (07_3); in dots nothing has changed for A or C, a search of
//! C would catch thirteen and a third (07_4); and the loop, search, empty,
//! update, choose again, turned twice with every dot split in three so that
//! whole dots go out (07_5).
//!
//! The numbers in the table are the model's, cell by cell, from the chance
//! the search would come back empty: one third for the cell searched and one
//! for every other.

use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CARRY_HOLD_S, GOLD_PT, QuizFrame, SEA_DOT_R, SEA_RING_R, Sea, SeaDot, Strip, arc_ring, fade_from_black,
    fade_in_all, fade_out_all, fail, find, fit_width, flat, frac_tex, gold_tex, in_cell, line_style, map, out, pause,
    percent_tex, say, sea_dots, sea_rect, sea_slot, sea_split, search, tween_linear, wait_until,
};
use super::glyphs;
use super::nothing_found::pct_text;
use crate::model::{Q, sea};

pub struct BayesRule {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["07_1", "07_2", "07_3", "07_4", "07_5"];

// the phrases the pictures land on (a test finds each in its line)
const P_RECIPE: &str = "As a recipe";
const P_MULTIPLY: &str = "Multiply each cell's chance";
const P_THIRD: &str = "One third for cell B";
const P_DIVIDE: &str = "Then divide by what is left";
const P_ONE_HUNDRED: &str = "so the map adds up";
const P_A_STILL: &str = "A search of A would still catch ten";
const P_C_THIRD: &str = "a search of C thirteen and a third";
const P_FALLEN: &str = "Only B has fallen";

// fixed words
const BAYES: &str = "Bayes\u{2019} rule";
const BAYES_NOTE: &str = "Thomas Bayes, a church minister, died";
const LAPLACE_NOTE: &str = "As a rule for equally likely cases, which is what the dots are: Laplace,";
const PRIOR: &str = "prior";
const PRIOR_WORDS: &str = "the map before a search. Posterior: the map after it";
const HEAD_CHANCE: &str = "chance";
const HEAD_EMPTY: &str = "× chance of empty";
const HEAD_PRODUCT: &str = "product";
/// The last column's head: what the products are divided by, and that the
/// quotient is read as a percent (40 ÷ 80 is a half, which is 50%; R2: M4,
/// D21, R10).
const HEAD_NEW_PERCENT: &str = ", as a percent";
const NOT_FOUND: &str = "a search of B cannot find a wreck that is somewhere else";
const SUM_LABEL: &str = "the chance this search comes back empty";
const OTHERS: &str = "every other cell: its own chance ÷ that same 1 − p d";
const UPDATE: &str = "update: redrawing the map by Bayes\u{2019} rule";
const MORE_DOTS: &str = "more dots, same map";
const CATCH_WORDS: &str = "dots a search would catch";
const LOOP_WORDS: [&str; 4] = ["Search", "Come back empty", "Update", "Choose again"];
const PROGRAMMERS_LINES: usize = 2;

/// The table's columns and rows (right half of the stage). The sum and its
/// label stand under the product column, the small note under the ones.
const COL_X: [f64; 5] = [0.45, 1.5, 2.95, 4.45, 5.85];
const ROW_Y: [f64; 4] = [2.7, 2.1, 1.5, 0.9];
const HEAD_Y: f64 = 3.35;
const NOTE_Y: f64 = 0.45;
const SUM_Y: f64 = -0.15;
/// The letters, the other cells' sentence and the name of the move, in the
/// hold (the sum, its label and the note have gone by then).
const HOLD_Y: [f64; 3] = [-0.4, -1.0, -1.5];
/// B's row worked in full stands right of the sum, under the last column,
/// in this much room.
const WORKED_AT: [f64; 2] = [5.75, SUM_Y];
const WORKED_ROOM: f64 = 1.9;
/// The code panel is lowered to this height (it is built on the middle line).
const PANEL_DROP: f64 = -2.55;
/// Where the "dots a search would catch" label stands.
const CATCH_AT: [f64; 2] = [-3.45, -2.45];
/// The loop's four words at the corners of a rectangle.
const LOOP: [[f64; 2]; 4] = [[2.1, 2.7], [5.0, 2.7], [5.0, 0.9], [2.1, 0.9]];
/// How far a side of the loop stands off a word's own edge, across.
const LOOP_GAP: f64 = 0.25;

/// What one search does to the map as the table works it: each cell's
/// chance (in dots of a hundred), its chance of coming back empty, the
/// product, and the new chance once divided by what is left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub chance: Q,
    pub empty: Q,
    pub product: Q,
    pub then: Q,
}

/// The table for a search of `searched` that sees what is there with
/// chance `seeing`: its four rows and the sum they divide by.
pub fn recipe(searched: usize, seeing: Q) -> ([Row; 4], Q) {
    let p = sea::prior();
    let hundred = Q::int(sea::DOTS);
    let empty = |c: usize| if c == searched { Q::ONE - seeing } else { Q::ONE };
    let products: [Q; 4] = std::array::from_fn(|c| p[c] * hundred * empty(c));
    let left = products.iter().fold(Q::ZERO, |a, &b| a + b);
    let rows = std::array::from_fn(|c| Row {
        chance: p[c] * hundred,
        empty: empty(c),
        product: products[c],
        then: products[c] / left * hundred,
    });
    (rows, left)
}

/// A dot split in three: where its two new siblings stand (seeded around
/// it), each a blue dot and the slate twin that goes over it when a search
/// puts it out.
#[derive(Clone, Copy, Debug)]
struct Sibling {
    blue: Mob,
    twin: Mob,
}

/// One original dot with its two siblings.
#[derive(Clone, Copy, Debug)]
struct Triple {
    dot: SeaDot,
    sibs: [Sibling; 2],
    /// Whether the original was still lit when it split.
    lit: bool,
}

/// How far a sibling stands from its dot: further than a dot's width, so
/// the three of a triple never touch.
pub const SIBLING_GAP: f64 = 0.2;

/// Where the two siblings of a dot at `at` stand: either side of it,
/// [`SIBLING_GAP`] away, along a seeded direction.
pub fn sibling_spots(at: Vec3, seed: u64) -> [Vec3; 2] {
    let theta = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 40) as f64 / (1u64 << 24) as f64 * std::f64::consts::PI;
    let (dx, dy) = (SIBLING_GAP * theta.cos(), SIBLING_GAP * theta.sin());
    [v(at[0] + dx, at[1] + dy), v(at[0] - dx, at[1] - dy)]
}

/// Split every dot of the sea in three. `lit[i]` says whether dot `i` of
/// `dots` is still lit. Detached.
fn split(stage: &mut Stage<'_>, dots: &[SeaDot], lit: &[bool]) -> Vec<Triple> {
    let mut out_triples = Vec::new();
    for (i, d) in dots.iter().enumerate() {
        let spots = sibling_spots(d.at, 0x5111 + i as u64);
        let sibs = spots.map(|at| {
            let blue = stage
                .arena_mut()
                .add(Dot::new().point(at).radius(SEA_DOT_R).style(flat(map(), 1.0)).build());
            let twin = stage.arena_mut().add(
                Dot::new()
                    .point(at)
                    .radius(SEA_DOT_R * 1.1)
                    .style(flat(out(), 1.0))
                    .build(),
            );
            Sibling { blue, twin }
        });
        out_triples.push(Triple {
            dot: *d,
            sibs,
            lit: lit[i],
        });
    }
    out_triples
}

/// The slate twins of the first `n` lit dots of `cell`, original then its
/// two siblings, triple by triple: the dots a search of the cell puts out.
fn go_out(triples: &[Triple], cell: usize, n: usize) -> Vec<Mob> {
    triples
        .iter()
        .filter(|t| t.dot.cell == cell && t.lit)
        .flat_map(|t| [t.dot.twin, t.sibs[0].twin, t.sibs[1].twin])
        .take(n)
        .collect()
}

/// A count of the hundred dots as the table prints it: 40, 10, 80 (the
/// narrated draft printed `percent(0)` of these, 4000 and 8000: a count of
/// dots is already out of a hundred).
pub fn dots_text(q: Q) -> String {
    q.fixed(0)
}

/// The last column's head: "÷ 80, as a percent".
pub fn head_new(left: Q) -> String {
    format!("÷ {}{HEAD_NEW_PERCENT}", dots_text(left))
}

/// A row worked in full, so that the division is read as a percent:
/// "10 ÷ 80 = 1/8 = 12.5%".
pub fn worked_text(row: &Row, left: Q) -> String {
    let share = row.then / Q::int(sea::DOTS);
    format!(
        "{} ÷ {} = {} = {}",
        dots_text(row.product),
        dots_text(left),
        mixed_text(share),
        pct_text(share)
    )
}

/// A count that may carry a third, as plain text: "13 1/3", "6 2/3", "10".
pub fn mixed_text(q: Q) -> String {
    let s = q.to_string();
    match s.split_once('/') {
        Some((n, d)) => {
            let (n, d): (i128, i128) = (n.parse().unwrap_or(0), d.parse().unwrap_or(1));
            if n / d > 0 {
                format!("{} {}/{d}", n / d, n % d)
            } else {
                format!("{n}/{d}")
            }
        }
        None => s,
    }
}

/// A table cell's text, detached.
fn cell_text(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    kit.text(stage, words, size, color, at)
}

impl SceneConstruct for BayesRule {
    fn name(&self) -> &str {
        "07_bayes_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let prior = sea::prior();
        let seeing = sea::seeing();
        let finds = sea::find(&prior);
        let run = sea::run(sea::Rule::Best, 3);
        let b = run[0].cell;
        let (rows, left) = recipe(b, seeing[b]);
        let lit0: [u32; 4] = prior.map(|p| (p * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0));
        let out_n: usize = (prior[b] * seeing[b] * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0);

        // the made-up sea as it stood before the search, on the left
        let sea_map = Sea::build(&kit, stage, lit0)?;
        let dots = sea_dots(stage, sea_split());
        for m in sea_map.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.add_to_scene(sea_map.tag)?;
        for d in &dots {
            stage.add_to_scene(d.dot)?;
        }
        let mut percents = Vec::new();
        let mut seen = Vec::new();
        for c in 0..4 {
            let p = cell_text(&kit, stage, &pct_text(prior[c]), 28.0, map(), sea_slot(c, 0))?;
            stage.add_to_scene(p)?;
            percents.push(p);
            let s = seeing[c].to_string();
            let (n, d) = s.split_once('/').unwrap_or((s.as_str(), "1"));
            let t = cell_text(&kit, stage, &format!("{n} in {d}"), 26.0, search(), sea_slot(c, 1))?;
            stage.add_to_scene(t)?;
            seen.push(t);
        }
        fade_from_black(stage, 0.5)?;

        // 07_1: that move has a name
        let line = say(&kit, stage, LINES[0])?;
        let died = facts
            .get("F-N.bayes.died")
            .ok_or_else(|| fail("no fact F-N.bayes.died".into()))?;
        let laplace = facts
            .get("F-N.laplace")
            .ok_or_else(|| fail("no fact F-N.laplace".into()))?;
        let name = kit.text(stage, BAYES, 56.0, Palette::ink(), v(3.4, 2.4))?;
        let note = kit.text(
            stage,
            &format!("{BAYES_NOTE} {died}"),
            30.0,
            Palette::dim(),
            v(3.4, 1.55),
        )?;
        let note2 = kit.text(
            stage,
            &format!("{LAPLACE_NOTE} {laplace}"),
            22.0,
            Palette::dim(),
            v(3.4, 0.95),
        )?;
        super::draw::fit_width(stage, note2, 6.6);
        fade_in_all(stage, &[name, note, note2], 0.7)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        let names = lower_third(&kit, stage, PRIOR, PRIOR_WORDS)?;
        fade_in_all(stage, &[names], 0.5)?;
        pause(stage, CARRY_HOLD_S)?;
        fade_out_all(stage, &[names, name, note, note2], 0.5)?;

        // 07_2: as a recipe, a column a sentence
        let line = say(&kit, stage, LINES[1])?;
        line.wait_for(stage, P_RECIPE)?;
        let heads = [
            cell_text(&kit, stage, HEAD_CHANCE, 22.0, Palette::dim(), v(COL_X[1], HEAD_Y))?,
            cell_text(&kit, stage, HEAD_EMPTY, 22.0, Palette::dim(), v(COL_X[2], HEAD_Y))?,
            cell_text(&kit, stage, HEAD_PRODUCT, 22.0, Palette::dim(), v(COL_X[3], HEAD_Y))?,
            cell_text(&kit, stage, &head_new(left), 22.0, Palette::dim(), v(COL_X[4], HEAD_Y))?,
        ];
        for h in &heads[1..] {
            fit_width(stage, *h, 2.0);
        }
        let mut letters = Vec::new();
        let mut chances = Vec::new();
        for c in 0..4 {
            letters.push(cell_text(
                &kit,
                stage,
                sea::CELLS[c],
                28.0,
                Palette::dim(),
                v(COL_X[0], ROW_Y[c]),
            )?);
            chances.push(cell_text(
                &kit,
                stage,
                &dots_text(rows[c].chance),
                30.0,
                map(),
                v(COL_X[1], ROW_Y[c]),
            )?);
        }
        let mut first: Vec<Mob> = vec![heads[0]];
        first.extend(&letters);
        fade_in_all(stage, &first, 0.4)?;
        // the percents fly in from the map
        let mut flights: Vec<Box<dyn Animation>> = Vec::new();
        for c in 0..4 {
            let at = sea_slot(c, 0);
            stage.move_to(chances[c], at, ORIGIN);
            stage.add_to_scene(chances[c])?;
            let a = chances[c]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.9),
                    ..AnimateArgs::default()
                })?
                .move_to(v(COL_X[1], ROW_Y[c]), ORIGIN)?;
            flights.push(stage.prepare(a)?);
        }
        stage.play_prepared(flights)?;

        line.wait_for(stage, P_MULTIPLY)?;
        fade_in_all(stage, &[heads[1]], 0.4)?;
        line.wait_for(stage, P_THIRD)?;
        let mut mults = Vec::new();
        for c in 0..4 {
            mults.push(kit.tex(
                stage,
                &frac_tex(rows[c].empty),
                34.0,
                search(),
                &[],
                v(COL_X[2], ROW_Y[c]),
            )?);
        }
        fade_in_all(stage, &mults, 0.5)?;
        let why = cell_text(&kit, stage, NOT_FOUND, 20.0, Palette::dim(), v(COL_X[2] + 0.9, NOTE_Y))?;
        fit_width(stage, why, 6.2);
        fade_in_all(stage, &[why, heads[2]], 0.5)?;
        let mut products = Vec::new();
        for c in 0..4 {
            products.push(cell_text(
                &kit,
                stage,
                &dots_text(rows[c].product),
                30.0,
                Palette::ink(),
                v(COL_X[3], ROW_Y[c]),
            )?);
        }
        fade_in_all(stage, &products, 0.5)?;
        line.wait_for(stage, P_DIVIDE)?;
        let total = cell_text(&kit, stage, &dots_text(left), 36.0, search(), v(COL_X[3], SUM_Y))?;
        let total_label = cell_text(&kit, stage, SUM_LABEL, 20.0, search(), v(COL_X[3] - 0.4, SUM_Y - 0.5))?;
        fit_width(stage, total_label, 5.6);
        fade_in_all(stage, &[total, total_label], 0.6)?;
        line.wait_for(stage, P_ONE_HUNDRED)?;
        fade_in_all(stage, &[heads[3]], 0.4)?;
        let mut thens = Vec::new();
        for c in 0..4 {
            thens.push(cell_text(
                &kit,
                stage,
                &pct_text(rows[c].then / Q::int(100)),
                30.0,
                map(),
                v(COL_X[4], ROW_Y[c]),
            )?);
        }
        fade_in_all(stage, &thens, 0.6)?;
        // B's row worked in full (10 ÷ 80 = 1/8 = 12.5%): the division is
        // read as a percent, and the 12.5 is B's own
        let worked = cell_text(
            &kit,
            stage,
            &worked_text(&rows[b], left),
            20.0,
            Palette::dim(),
            v(WORKED_AT[0], WORKED_AT[1]),
        )?;
        fit_width(stage, worked, WORKED_ROOM);
        fade_in_all(stage, &[worked], 0.5)?;
        line.wait_end(stage)?;

        // the hold: the letters land on B's row, the other cells' sentence,
        // the name of the move, and the first two lines of the code panel
        kit.hold(stage)?;
        // the sum, its label and the note have said their piece
        fade_out_all(stage, &[why, total, total_label, worked], 0.4)?;
        let formula = kit.tex(
            stage,
            "\\frac{p(1-d)}{1-pd}",
            44.0,
            Palette::ink(),
            &[("p", map()), ("d", search())],
            v(3.3, HOLD_Y[0]),
        )?;
        let others = cell_text(&kit, stage, OTHERS, 22.0, Palette::dim(), v(3.3, HOLD_Y[1]))?;
        fit_width(stage, others, 6.4);
        let named = cell_text(&kit, stage, UPDATE, 24.0, Palette::ink(), v(3.3, HOLD_Y[2]))?;
        fade_in_all(stage, &[formula, others, named], 0.7)?;
        let panel = CodePanel::build(&kit, stage, &glyphs::CODE[..PROGRAMMERS_LINES], 16.0, 10.0)?;
        // the panel is built at the frame's right edge on the middle line;
        // lower it to the bottom band, clear of the table
        stage.shift(panel.group, [0.0, PANEL_DROP, 0.0]);
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.8)?)?;
        pause(stage, 1.2)?;

        // the sea is redrawn: B's twenty go out, B cools, the others deepen
        let mut update: Vec<Box<dyn Animation>> = Vec::new();
        let in_b = in_cell(&dots, b);
        for d in in_b.iter().take(out_n) {
            let twin = fade_in(stage.arena_mut(), d.twin, ORIGIN, 1.0)?;
            update.push(stage.prepare(twin.rt(0.7))?);
        }
        let lit_after: [f64; 4] = std::array::from_fn(|c| {
            if c == b {
                (lit0[c] as usize - out_n) as f64
            } else {
                f64::from(lit0[c])
            }
        });
        for c in 0..4 {
            update.push(stage.prepare(explainer_kit::tween(sea_map.lit[c], lit_after[c], 0.8)?)?);
        }
        for &m in &percents {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            update.push(stage.prepare(a.rt(0.4))?);
        }
        let mut after_labels = Vec::new();
        for (c, &q) in run[0].after.iter().enumerate() {
            let m = cell_text(&kit, stage, &pct_text(q), 28.0, map(), sea_slot(c, 0))?;
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            update.push(stage.prepare(a.rt(0.5))?);
            after_labels.push(m);
        }
        stage.play_prepared(update)?;

        // 07_3: your turn: here is the new map; A is the canyon; where next?
        // the table and the panel clear; the first round's gold numbers are
        // greyed; the answer is the cell whose border turns gold
        let mut table: Vec<Mob> = Vec::new();
        table.extend(heads);
        table.extend(&letters);
        table.extend(&chances);
        table.extend(&mults);
        table.extend(&products);
        table.extend(&thens);
        table.extend([formula, others, named, panel.group]);
        fade_out_all(stage, &table, 0.6)?;
        let mut greyed = Vec::new();
        for (c, &q) in finds.iter().enumerate() {
            greyed.push(gold_tex(&kit, stage, &percent_tex(q), Palette::dim(), c)?);
        }
        fade_in_all(stage, &greyed, 0.4)?;
        let line = say(&kit, stage, LINES[2])?;
        let frame = QuizFrame::your_turn(&kit, stage, &line)?;
        let answer = facts
            .get("F-Q.next_search.answer")
            .ok_or_else(|| fail("no fact F-Q.next_search.answer".into()))?;
        let next = sea::CELLS
            .iter()
            .position(|c| *c == answer)
            .ok_or_else(|| fail(format!("the quiz's answer {answer} is not a cell")))?;
        let mut anims = frame.lift(stage, 0.5)?;
        let gold = sea_map.borders[next]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.45),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(find()), Some(5.0), Some(1.0))?;
        anims.push(stage.prepare(gold)?);
        stage.play_prepared(anims)?;

        // 07_4: in dots nothing has changed for A or C: each search's rings
        // (a third of a ring where a third of a dot is), the new numbers
        let line = say(&kit, stage, LINES[3])?;
        let hundred = Q::int(sea::DOTS);
        // what a search of each cell would catch now, in dots of the first
        // hundred: its next slice (`F-M.second.find.dots`)
        let catch: [Q; 4] = std::array::from_fn(|c| {
            let searched = u32::from(c == b);
            sea::slices(c, searched + 1)[searched as usize] * hundred
        });
        // the rings: whole ones on the cell's lit dots in the order they
        // would be caught, and a part of a ring on the next for a fraction
        let mut ring_sets: Vec<Vec<Mob>> = Vec::new();
        for (c, q) in catch.iter().enumerate() {
            let lit_here: Vec<SeaDot> = in_cell(&dots, c)
                .into_iter()
                .skip(if c == b { out_n } else { 0 })
                .collect();
            let value = q.f64();
            let whole = (value + 1e-9).floor() as usize;
            let part = value - whole as f64;
            let mut rings: Vec<Mob> = lit_here.iter().take(whole).map(|d| d.ring).collect();
            if part > 1e-9
                && let Some(d) = lit_here.get(whole)
            {
                rings.push(arc_ring(stage, d.at, SEA_RING_R, part, search(), 2.5));
            }
            ring_sets.push(rings);
        }
        let mut numbers = Vec::new();
        for (c, q) in catch.iter().enumerate() {
            // text, not a stacked fraction: B's sits under the top row,
            // where the cell below starts 0.24 under the slot
            numbers.push(kit.text(stage, &mixed_text(*q), GOLD_PT, find(), sea_slot(c, 2))?);
        }
        let catch_label = kit.text(stage, CATCH_WORDS, 24.0, find(), v(CATCH_AT[0], CATCH_AT[1]))?;
        // A's ten as "a search of A would still catch ten", then C's
        // thirteen and a third, then B's and D's six and two thirds as the
        // numbers of the first round give way to these
        line.wait_for(stage, P_A_STILL)?;
        fade_in_all(stage, &ring_sets[0], 0.5)?;
        line.wait_for(stage, P_C_THIRD)?;
        fade_in_all(stage, &ring_sets[next], 0.5)?;
        line.wait_for(stage, P_FALLEN)?;
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for (c, rings) in ring_sets.iter().enumerate() {
            if c != 0 && c != next {
                for &r in rings {
                    let a = fade_in(stage.arena_mut(), r, ORIGIN, 1.0)?;
                    swap.push(stage.prepare(a.rt(0.5))?);
                }
            }
        }
        for (&g, &n) in greyed.iter().zip(&numbers) {
            let gone = fade_out(stage.arena_mut(), g, ORIGIN, 1.0)?;
            swap.push(stage.prepare(gone.rt(0.4))?);
            let a = fade_in(stage.arena_mut(), n, ORIGIN, 1.0)?;
            swap.push(stage.prepare(a.rt(0.5))?);
        }
        let a = fade_in(stage.arena_mut(), catch_label, ORIGIN, 1.0)?;
        swap.push(stage.prepare(a.rt(0.5))?);
        stage.play_prepared(swap)?;
        line.wait_end(stage)?;

        // 07_5: search, come back empty, update, choose again: every dot
        // splits in three, then the loop turns, and once more by itself.
        // The preparation is silent, so the picture is ready as the first
        // word sounds.
        let mut leaving: Vec<Mob> = after_labels.clone();
        leaving.extend(seen.iter().copied());
        leaving.extend(&numbers);
        leaving.push(catch_label);
        for rings in &ring_sets {
            leaving.extend(rings);
        }
        fade_out_all(stage, &leaving, 0.4)?;
        let went: Vec<Mob> = in_cell(&dots, b).iter().take(out_n).map(|d| d.dot).collect();
        let lit_flags: Vec<bool> = dots.iter().map(|d| !went.contains(&d.dot)).collect();
        let triples = split(stage, &dots, &lit_flags);
        // the siblings of lit dots are blue; of dots already out, slate
        let mut ins: Vec<Mob> = Vec::new();
        for t in &triples {
            for s in &t.sibs {
                ins.push(if t.lit { s.blue } else { s.twin });
            }
        }
        let more = cell_text(&kit, stage, MORE_DOTS, 28.0, Palette::dim(), v(3.5, -0.3))?;
        fade_in_all(stage, &ins, 0.7)?;
        fade_in_all(stage, &[more], 0.4)?;
        let third = |n: usize| (n * crate::facts::shown::SPLIT as usize) as f64;
        let lit3: [f64; 4] = lit_after.map(|n| third(n as usize));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for c in 0..4 {
            anims.push(stage.prepare(explainer_kit::tween(sea_map.lit[c], lit3[c], 0.3)?)?);
        }
        stage.play_prepared(anims)?;

        // the loop's four words, at the corners of a rectangle
        let mut words = Vec::new();
        for (i, w) in LOOP_WORDS.iter().enumerate() {
            words.push(cell_text(
                &kit,
                stage,
                w,
                34.0,
                Palette::dim(),
                v(LOOP[i][0], LOOP[i][1]),
            )?);
        }
        let mut sides = Vec::new();
        for i in 0..4 {
            let (a, c) = (LOOP[i], LOOP[(i + 1) % 4]);
            let (dx, dy) = (c[0] - a[0], c[1] - a[1]);
            let n = (dx * dx + dy * dy).sqrt();
            // from just off one word to just off the next: across, a gap
            // from each word's own edge (the narrated draft's dash touched
            // "Come back empty"); down, clear of a line of type
            let (ux, uy) = (dx / n, dy / n);
            let (wa, wc) = (stage.get_width(words[i]), stage.get_width(words[(i + 1) % 4]));
            let (pa, pc) = if dx.abs() > dy.abs() {
                (wa / 2.0 + LOOP_GAP, wc / 2.0 + LOOP_GAP)
            } else {
                (0.5, 0.5)
            };
            sides.push(
                stage.arena_mut().add(
                    polyline(vec![
                        v(a[0] + ux * pa, a[1] + uy * pa),
                        v(c[0] - ux * pc, c[1] - uy * pc),
                    ])
                    .style(line_style(Palette::faint(), 2.5))
                    .build(),
                ),
            );
        }
        let mut ring_ins = words.clone();
        ring_ins.extend(&sides);
        fade_in_all(stage, &ring_ins, 0.5)?;

        let line = say(&kit, stage, LINES[4])?;
        let c_cell = run[1].cell;
        let a_cell = run[2].cell;
        // "Search": the strip crosses C
        line.wait_sentence(stage, 0)?;
        highlight(stage, &words, 0)?;
        kit.mark(stage, super::sound::SEARCH_MARK);
        let (x0, y0, x1, y1) = sea_rect(c_cell);
        let strip = Strip::build(stage, (x0, y0, x1, y1), 0.1 * (x1 - x0))?;
        for m in strip.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(strip.phase, 1.0, 0.7)?)?;
        // "Come back empty": nothing answers
        line.wait_sentence(stage, 1)?;
        highlight(stage, &words, 1)?;
        // "Update": forty of C's sixty go out and the heat shifts
        line.wait_sentence(stage, 2)?;
        highlight(stage, &words, 2)?;
        let gone_c = go_out(&triples, c_cell, 40);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &gone_c {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        let lit4: [f64; 4] = std::array::from_fn(|c| if c == c_cell { lit3[c] - 40.0 } else { lit3[c] });
        for c in 0..4 {
            anims.push(stage.prepare(explainer_kit::tween(sea_map.lit[c], lit4[c], 0.7)?)?);
        }
        for m in [strip.wash, strip.bar] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        // "Choose again": the gold border jumps to A
        line.wait_sentence(stage, 3)?;
        highlight(stage, &words, 3)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (cell, color, width) in [(next, Palette::faint(), 2.0), (a_cell, find(), 5.0)] {
            let a = sea_map.borders[cell]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(color), Some(width), Some(1.0))?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        line.wait_end(stage)?;

        // three seconds of the loop turning once more by itself: A is
        // searched, comes back empty, thirty dots go out, the gold stays on A
        kit.hold(stage)?;
        let t0 = stage.scene().time().to_f64();
        highlight(stage, &words, 0)?;
        kit.mark(stage, super::sound::SEARCH_MARK);
        let (x0, y0, x1, y1) = sea_rect(a_cell);
        let strip = Strip::build(stage, (x0, y0, x1, y1), 0.1 * (x1 - x0))?;
        for m in strip.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(strip.phase, 1.0, 0.7)?)?;
        highlight(stage, &words, 1)?;
        pause(stage, 0.3)?;
        highlight(stage, &words, 2)?;
        let gone_a = go_out(&triples, a_cell, 30);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &gone_a {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let lit5: [f64; 4] = std::array::from_fn(|c| if c == a_cell { lit4[c] - 30.0 } else { lit4[c] });
        for c in 0..4 {
            anims.push(stage.prepare(explainer_kit::tween(sea_map.lit[c], lit5[c], 0.6)?)?);
        }
        stage.play_prepared(anims)?;
        highlight(stage, &words, 3)?;
        wait_until(stage, t0 + 3.0)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// Light word `i` of the loop in ink and put the others back to dim.
fn highlight(stage: &mut Stage<'_>, words: &[Mob], i: usize) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for (k, &w) in words.iter().enumerate() {
        let color = if k == i { Palette::ink() } else { Palette::dim() };
        let a = w
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.2),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(color), None)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::Script;

    /// (line, phrase): every phrase a picture lands on.
    const PHRASES: [(&str, &str); 9] = [
        ("07_2", P_RECIPE),
        ("07_2", P_MULTIPLY),
        ("07_2", P_THIRD),
        ("07_2", P_DIVIDE),
        ("07_2", P_ONE_HUNDRED),
        ("07_4", "Cell C"),
        ("07_4", P_A_STILL),
        ("07_4", P_C_THIRD),
        ("07_4", P_FALLEN),
    ];

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("07_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    #[test]
    fn every_phrase_the_pictures_land_on_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in PHRASES {
            let text = script.text(id).unwrap();
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // 07_5 is keyed by sentence, and has the four words of the loop in
        // order plus the verdict
        let loop_line = script.text("07_5").unwrap();
        assert_eq!(loop_line.matches(". ").count() + 1, 5);
        let mut at = 0;
        for w in ["Search", "Come back empty", "Update", "Choose again"] {
            let i = loop_line[at..].find(w).expect(w) + at;
            at = i + w.len();
        }
    }

    /// The table of 07_2 as the model works it: 40, 30, 20, 10; times 1,
    /// one third, 1, 1; the products 40, 10, 20, 10; their sum 80; and each
    /// divided by it, 50, 12.5, 25, 12.5. Each number is a fact.
    #[test]
    fn the_table_is_the_models_and_the_facts() {
        let b = 1;
        let (rows, left) = recipe(b, sea::seeing()[b]);
        let f = crate::facts::facts().unwrap();
        let col = |g: fn(&Row) -> Q, places: u32| -> String {
            rows.iter().map(|r| g(r).fixed(places)).collect::<Vec<_>>().join(",")
        };
        assert_eq!(col(|r| r.chance, 0), "40,30,20,10");
        assert_eq!(col(|r| r.product, 0), "40,10,20,10");
        assert_eq!(left, Q::int(80));
        assert_eq!(f.get("F-M.empty1.dots_left"), Some("80"));
        assert_eq!(col(|r| r.then, 1), "50.0,12.5,25.0,12.5");
        assert_eq!(f.get("F-M.empty1.after.percent"), Some("50.0,12.5,25.0,12.5"));
        assert_eq!(
            rows.iter().map(|r| frac_tex(r.empty)).collect::<Vec<_>>(),
            ["1", "\\frac{1}{3}", "1", "1"]
        );
        // the products add to what is left, and the new chances to a hundred
        assert_eq!(rows.iter().fold(Q::ZERO, |a, r| a + r.product), left);
        assert_eq!(rows.iter().fold(Q::ZERO, |a, r| a + r.then), Q::int(100));
        // the letters' formula for B's row, p (1 - d) over (1 - p d), is the
        // same number: with p = 3/10 and d = 2/3, 1/10 over 4/5 is 1/8
        let (p, d) = (sea::prior()[b], sea::seeing()[b]);
        assert_eq!(p * (Q::ONE - d) / (Q::ONE - p * d), Q::lowest(1, 8));
        assert_eq!(f.get("F-M.empty1.after.fraction"), Some("1/2,1/8,1/4,1/8"));
        // and what the stage prints is those counts, not their percents (the
        // narrated draft showed 4000, 1000 and 8000: `percent(0)` of a count
        // of dots out of a hundred)
        let printed = |g: fn(&Row) -> Q| -> Vec<String> { rows.iter().map(|r| dots_text(g(r))).collect() };
        assert_eq!(printed(|r| r.chance), ["40", "30", "20", "10"]);
        assert_eq!(printed(|r| r.product), ["40", "10", "20", "10"]);
        assert_eq!(dots_text(left), "80");
        let new_chances: Vec<String> = rows.iter().map(|r| pct_text(r.then / Q::int(100))).collect();
        assert_eq!(new_chances, ["50%", "12.5%", "25%", "12.5%"]);
        // R2 (M4, D21, R10): the column's head says the quotient is a
        // percent, and B's row is worked in full, as 40 ÷ 80 is a half and
        // not 50 until it is read as a percent
        assert_eq!(head_new(left), "÷ 80, as a percent");
        assert_eq!(worked_text(&rows[b], left), "10 ÷ 80 = 1/8 = 12.5%");
        assert_eq!(worked_text(&rows[0], left), "40 ÷ 80 = 1/2 = 50%");
        // the head and the worked row fit the column's room at their size
        let units = super::super::draw::CHAR_UNITS_PER_POINT;
        assert!(head_new(left).chars().count() as f64 * 22.0 * units <= 2.0);
        assert!(worked_text(&rows[b], left).chars().count() as f64 * 20.0 * units <= WORKED_ROOM);
        // it stands right of the sum and inside the frame
        const {
            assert!(WORKED_AT[0] - WORKED_ROOM / 2.0 > COL_X[3] + 0.3);
            assert!(WORKED_AT[0] + WORKED_ROOM / 2.0 < super::super::draw::QUIZ_FRAME.0 - 0.15);
        }
    }

    /// R2 (M22): Bayes was a church minister, said on the card as well as
    /// in the line.
    #[test]
    fn the_card_says_what_kind_of_minister_bayes_was() {
        assert_eq!(BAYES_NOTE, "Thomas Bayes, a church minister, died");
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("F-N.bayes.died"), Some("1761"));
    }

    /// 07_4's numbers are counts of dots with a third in them: set as text,
    /// the way the line reads them.
    #[test]
    fn a_count_with_a_third_in_it_is_set_as_the_line_reads_it() {
        assert_eq!(mixed_text(Q::lowest(40, 3)), "13 1/3");
        assert_eq!(mixed_text(Q::lowest(20, 3)), "6 2/3");
        assert_eq!(mixed_text(Q::int(10)), "10");
        assert_eq!(mixed_text(Q::lowest(1, 3)), "1/3");
    }

    /// The edges of the recipe: a search that cannot see leaves the map as
    /// it was, and one that sees everything takes its cell to nothing; in
    /// either case the new chances still add to a hundred.
    #[test]
    fn the_recipe_at_its_edges() {
        let blind = recipe(1, Q::ZERO).0;
        assert_eq!(blind.map(|r| r.then), blind.map(|r| r.chance));
        let sure = recipe(1, Q::ONE).0;
        assert_eq!(sure[1].then, Q::ZERO);
        assert_eq!(sure.iter().fold(Q::ZERO, |a, r| a + r.then), Q::int(100));
    }

    /// 07_3 and 07_4: after the update C is the cell to search (A would be
    /// the likeliest), what each search would catch in dots is the fact,
    /// and "nothing has changed for A or C" in dots.
    #[test]
    fn the_second_choice_is_c_and_in_dots_a_and_c_have_not_changed() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("F-Q.next_search.answer"), Some("C"));
        assert_eq!(f.get("F-M.second.find.dots"), Some("10,20/3,40/3,20/3"));
        let run = sea::run(sea::Rule::Best, 3);
        assert_eq!(sea::CELLS[run[1].cell], "C");
        assert_eq!(sea::CELLS[run[2].cell], "A");
        let after = run[0].after;
        // A is now the likeliest cell and C is not the choice by chance
        assert!(after[0] > after[2]);
        // what a search catches, in dots of the first hundred
        let hundred = Q::int(sea::DOTS);
        let before: Vec<Q> = (0..4).map(|c| sea::slices(c, 1)[0] * hundred).collect();
        let now: Vec<Q> = (0..4)
            .map(|c| {
                let k = usize::from(c == 1);
                sea::slices(c, k as u32 + 1)[k] * hundred
            })
            .collect();
        assert_eq!(before[0], now[0], "A: unchanged in dots");
        assert_eq!(before[2], now[2], "C: unchanged in dots");
        assert!(now[1] < before[1], "only B has fallen");
        assert_eq!(now[2], Q::lowest(40, 3), "thirteen and a third");
    }

    /// 07_5: split in three, 80 lit dots become 240; C's sixty lose forty
    /// (A, B, C, D go 120, 30, 60, 30 to 120, 30, 20, 30, which is 60, 15,
    /// 10, 15 percent); A's 120 then lose thirty.
    #[test]
    fn splitting_in_three_makes_whole_dots_and_the_facts_follow() {
        let f = crate::facts::facts().unwrap();
        let k = crate::facts::shown::SPLIT as usize;
        assert_eq!(k, 3);
        let after: [usize; 4] = [40, 10, 20, 10];
        let split: Vec<usize> = after.iter().map(|n| n * k).collect();
        assert_eq!(split, [120, 30, 60, 30]);
        assert_eq!(split.iter().sum::<usize>(), 240);
        assert_eq!(f.get("F-M.split.lit"), Some("240"));
        assert_eq!(f.get("F-M.split.lit.cells"), Some("120,30,60,30"));
        // the second search: two thirds of C's sixty
        let second = sea::seeing()[2] * Q::int(split[2] as i128);
        assert_eq!(second, Q::int(40));
        assert_eq!(f.get("F-M.split.second.dots_out"), Some("40"));
        assert_eq!(f.get("F-M.split.second.lit.cells"), Some("120,30,20,30"));
        assert_eq!(f.get("F-M.empty2.after.percent"), Some("60,15,10,15"));
        // the third: a quarter of A's hundred and twenty
        let third = sea::seeing()[0] * Q::int(120);
        assert_eq!(third, Q::int(30));
        assert_eq!(f.get("F-M.third.dots_out"), Some("30"));
    }

    #[test]
    fn a_dots_two_siblings_stand_either_side_of_it_and_apart() {
        for seed in [0u64, 1, 7, 99, 12345] {
            let at = v(1.5, -0.5);
            let [a, b] = sibling_spots(at, seed);
            let (da, db) = ((a[0] - at[0]).hypot(a[1] - at[1]), (b[0] - at[0]).hypot(b[1] - at[1]));
            assert!((da - SIBLING_GAP).abs() < 1e-9 && (db - SIBLING_GAP).abs() < 1e-9);
            // opposite sides of the dot
            assert!(((a[0] + b[0]) / 2.0 - at[0]).abs() < 1e-9);
            assert!(((a[1] + b[1]) / 2.0 - at[1]).abs() < 1e-9);
        }
        // and a dot's width fits between a sibling and its dot (constants,
        // checked as the crate builds)
        const {
            assert!(SIBLING_GAP >= 2.0 * SEA_DOT_R);
        }
    }

    /// The table and the loop sit in the right half, clear of the sea.
    #[test]
    fn the_table_and_the_loop_sit_right_of_the_sea() {
        use crate::chapters::draw::QUIZ_FRAME;
        let sea_right = sea_rect(1).2;
        assert!(COL_X[0] - 0.3 > sea_right);
        assert!(COL_X[4] + 0.5 < QUIZ_FRAME.0 + 0.2);
        assert!(COL_X.windows(2).all(|w| w[1] - w[0] >= 1.0), "columns do not touch");
        assert!(ROW_Y.windows(2).all(|w| w[0] - w[1] >= 0.55), "rows do not touch");
        for p in LOOP {
            assert!(p[0] - 1.3 > sea_right && p[0] + 1.3 < QUIZ_FRAME.0 + 0.4, "{p:?}");
        }
    }
}
