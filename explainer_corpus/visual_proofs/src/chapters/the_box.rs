//! Chapter 08, the box: the top of Siu's block is exactly half full, and the
//! empty half is the step turned round (chapter 04's second staircase). Ten
//! cubes cannot cover twenty places, so every cube of the step is sliced in
//! half, the upper halves lifted, turned half way round and lowered into the
//! empty half: the top is flat, the block is a box four wide, five deep and
//! four and a half tall, ninety cubes, three stacks of thirty (08_1 to 08_3);
//! then the same numbers as letters, one stack is a third of the box (08_4),
//! and where the picture is from (08_5).
//!
//! The step is `model::stacks::top(4)` and the places it is turned onto are
//! `model::stacks::turned_half`, cell for cell; the half cubes are boxes a
//! cube wide and deep and half a cube tall (`iso::Block::sized`), and the
//! scene fails if the lowered halves are not where the model puts them.

use std::f64::consts::PI;

use explainer_kit::{Kit, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, dim, fact, fact_ints, fail, flat, hero, ink, line_style, lower_third, pause, question_card, question_pause,
    say, size, source_line, tex_row,
};
use super::iso::{Axis, Block, Iso, P3, Solid};
use super::kinetics::{Moves, arrive, hide, leave, play, show};
use super::siu::{VIEW, colour, edge_brace, level};
use crate::facts::shown;
use crate::model::stacks;

pub struct TheBox {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["08_1", "08_2", "08_3", "08_4", "08_5"];

// the chapter's fixed words
const PLAN_WORD: &str = "top face";
const SOURCE_VOLUME: &str = "Mathematics Magazine 57";
const SOURCE_NAME: &str = "Man-Keung Siu";
const PRECISELY: &str = "precisely:";
const TEXTBOOK: &str = "the textbook form is n (n + 1)(2n + 1) ÷ 6";
const WORD_WIDE: &str = "wide";
const WORD_DEEP: &str = "deep";
const WORD_TALL: &str = "tall";

// ---------------------------------------------------------------- geometry

/// The plan of the top face: the lower left corner of its first cell and
/// the size of a cell, to the right of the block.
pub const PLAN_AT: Vec3 = [1.8, -0.9, 0.0];
pub const PLAN_CELL: f64 = 0.55;
/// How high the upper halves lift (at least one cube clear of the lower
/// ones), and the half a cube they sink to reach the gap's level.
pub const LIFT: f64 = 1.5;
/// The half cube: a cube wide and deep, half a cube tall.
pub const HALF: P3 = [1.0, 1.0, 0.5];

/// The block of four levels as 08 begins: the levels of the model by stack,
/// and the step, as the solids `iso` draws. Everything is shown.
struct Block4 {
    full: Vec<Solid>,
    step: Solid,
}

fn build_block(iso: &Iso, stage: &mut Stage<'_>, n: i64) -> fmn::Result<Block4> {
    let mut full = Vec::new();
    for z in 0..n {
        for which in 0..3 {
            let cells = level(n, which, z);
            if cells.is_empty() {
                continue;
            }
            let s = iso.solid(stage, &cells, colour(which), [0.0; 3])?;
            iso.show(stage, s)?;
            full.push(s);
        }
    }
    let step = iso.solid(stage, &level(n, 2, n), colour(2), [0.0; 3])?;
    iso.show(stage, step)?;
    Ok(Block4 { full, step })
}

/// Half cubes: a solid of boxes `HALF` big, each over a column (x, y) at
/// half-height index `k` (so z = k / 2). Hidden until shown.
fn halves(iso: &Iso, stage: &mut Stage<'_>, columns: &[(i64, i64)], k: i64, c: Srgb) -> fmn::Result<Solid> {
    let blocks: Vec<Block> = columns.iter().map(|&(x, y)| Block::sized((x, y, k), HALF)).collect();
    iso.solid_of(stage, &blocks, c, [0.0; 3], true)
}

/// A mixed number from a fact's fraction as plain text: `9/2` is `4½`, `4` is
/// `4`.
pub fn half_number(s: &str) -> String {
    match s.split_once('/') {
        Some((a, b)) => match (a.parse::<i64>(), b.parse::<i64>()) {
            (Ok(a), Ok(2)) => format!("{}½", a / 2),
            _ => s.to_owned(),
        },
        None => s.to_owned(),
    }
}

/// The same as TeX: `9/2` is `4\frac{1}{2}`.
pub fn half_number_tex(s: &str) -> String {
    match s.split_once('/') {
        Some((a, b)) => match (a.parse::<i64>(), b.parse::<i64>()) {
            (Ok(a), Ok(2)) => format!("{}\\frac{{1}}{{2}}", a / 2),
            _ => s.to_owned(),
        },
        None => s.to_owned(),
    }
}

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    wide: i64,
    deep: i64,
    tall: String,
    volume: i64,
    three: i64,
    stack: i64,
    halves: i64,
    year: i64,
    page: i64,
}

fn reads() -> fmn::Result<Reads> {
    let n = shown::STACK;
    let box_f = fact("F-S.box")?;
    let parts: Vec<&str> = box_f.split('x').collect();
    if parts.len() != 3 {
        return Err(fail(format!("F-S.box = {box_f:?} is not three sizes")));
    }
    let (wide, deep): (i64, i64) = (
        parts[0].parse().map_err(|_| fail("F-S.box width".into()))?,
        parts[1].parse().map_err(|_| fail("F-S.box depth".into()))?,
    );
    if (wide, deep) != (n, n + 1) {
        return Err(fail(format!(
            "F-S.box is {wide} by {deep}, the model's box {n} by {}",
            n + 1
        )));
    }
    let want = |key: &str, model: i64| -> fmn::Result<i64> {
        let got = super::draw::fact_int(key)?;
        if got == model {
            Ok(got)
        } else {
            Err(fail(format!("{key} is {got} but the model has {model}")))
        }
    };
    let top = stacks::top(n).len() as i64;
    let siu = fact_ints("F-X.siu")?;
    Ok(Reads {
        wide,
        deep,
        tall: parts[2].to_owned(),
        volume: want("F-S.box.volume", 90)?,
        three: want("F-S.three_stacks", 3 * stacks::standard(n).len() as i64)?,
        stack: want("F-S.stack", stacks::standard(n).len() as i64)?,
        halves: want("F-S.top.halves", 2 * top)?,
        year: siu[0],
        page: siu[1],
    })
}

/// How many cells of column `x` of the plan are step cells (the step is a
/// staircase: column `x` holds the cells with that first coordinate).
pub fn step_column(n: i64, x: i64) -> i64 {
    stacks::top(n).iter().filter(|c| c.0 == x).count() as i64
}

impl SceneConstruct for TheBox {
    fn name(&self) -> &str {
        "08_the_box"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let n = shown::STACK;
        let iso = Iso::new(stage, VIEW)?;
        let block = build_block(&iso, stage, n)?;
        let step_cells = stacks::top(n);
        let empty_cells: Vec<(i64, i64)> = step_cells.iter().map(|&c| stacks::turned_half(n, c)).collect();
        if 2 * step_cells.len() as i64 != r.halves {
            return Err(fail(format!(
                "F-S.top.halves is {} but the step's {} cubes make {} halves",
                r.halves,
                step_cells.len(),
                2 * step_cells.len()
            )));
        }

        // ---------------------------------------------------------- 08_1
        // the block as 07 left it; the step in gold; the empty half outlined
        let line = say(&kit, stage, LINES[0])?;
        let mut shown_now: Vec<AnimBuilder> = Vec::new();
        for &s in block.full.iter().chain([&block.step]) {
            shown_now.push(iso.fade(s, 1.0, 0.4)?);
        }
        iso.play(stage, shown_now, 0.0)?;
        line.wait_for(stage, "exactly half full")?;
        let gold = iso.glow(block.step, hero(), 1.0, 0.5)?;
        iso.play(stage, vec![gold], 0.0)?;
        let outline = flat_cells(&iso, stage, &empty_cells, n as f64, (0.0, 0.0), dim())?;
        show(stage, &[outline], 0.5, 0.0)?;
        line.wait_last(stage)?;
        let words = line
            .sentence_texts()
            .pop()
            .ok_or_else(|| fail("08_1 has no sentences".into()))?;
        let question = question_card(&kit, stage, &words, v(3.2, 1.8))?;
        show(stage, &[question], 0.5, 0.0)?;
        question_pause(&kit, stage)?;
        hide(stage, &[question], 0.4)?;

        // ---------------------------------------------------------- 08_2
        let line = say(&kit, stage, LINES[1])?;
        // a flat plan of the top face beside the block: the step's ten cells
        // in gold, the ten empty places outlined, a gold dot at the middle
        let board = Board::new(PLAN_AT, PLAN_CELL);
        let step_plan = board.tiles(stage, &step_cells, hero())?;
        let empty_plan = board.tiles_styled(stage, &empty_cells, line_style(dim(), 2.5))?;
        let dot = stage.arena_mut().add(
            Circle::new()
                .radius(0.09)
                .arc_center(board.p(n as f64 / 2.0, (n + 1) as f64 / 2.0))
                .style(flat(hero(), 1.0))
                .build(),
        );
        let plan_word = kit.text(
            stage,
            PLAN_WORD,
            26.0,
            dim(),
            board.p(n as f64 / 2.0, (n + 1) as f64 + 0.45),
        )?;
        let mut pairs: Vec<Mob> = Vec::new();
        for x in 0..n {
            let a = step_column(n, x);
            let b = (n + 1) - a;
            let t = kit.text(stage, &format!("{a}+{b}"), 20.0, ink(), board.p(x as f64 + 0.5, -0.5))?;
            pairs.push(t);
        }
        let total = kit.text(
            stage,
            &format!("= {}", n + 1),
            24.0,
            ink(),
            board.p(n as f64 + 1.0, -0.5),
        )?;
        let mut intro: Moves = vec![
            arrive(stage, step_plan, 0.5)?,
            arrive(stage, empty_plan, 0.5)?,
            arrive(stage, dot, 0.5)?,
            arrive(stage, plan_word, 0.5)?,
        ];
        for &p in pairs.iter().chain([&total]) {
            intro.push(arrive(stage, p, 0.5)?);
        }
        play(stage, intro, 0.12)?;
        // the step turns half way round the dot onto the empty cells
        line.wait_for(stage, "turned round")?;
        let centre = board.p(n as f64 / 2.0, (n + 1) as f64 / 2.0);
        let spin = Rotating::new(step_plan).with_angle(PI).with_about_point(centre).rt(1.4);
        let spin = stage.prepare(spin)?;
        play(stage, vec![spin], 0.0)?;

        // ten cubes cannot cover twenty places: every one is sliced in half
        line.wait_sentence(stage, 1)?;
        let lower = halves(&iso, stage, &step_cells, 2 * n, colour(2))?;
        let upper = halves(&iso, stage, &step_cells, 2 * n + 1, colour(2))?;
        iso.show(stage, lower)?;
        iso.show(stage, upper)?;
        let gold_off = iso.glow(block.step, hero(), 0.0, 0.01)?;
        let vanish = iso.fade(block.step, 0.0, 0.05)?;
        iso.play(stage, vec![gold_off, vanish], 0.0)?;
        let outline_out = leave(stage, outline, 0.3)?;
        play(stage, vec![outline_out], 0.0)?;
        pause(stage, 0.6)?;
        // lift the upper halves, turn them half way round, lower them in
        line.wait_sentence(stage, 2)?;
        let up = iso.slide_by(stage, upper, [0.0, 0.0, LIFT], 0.6)?;
        iso.play(stage, vec![up], 0.0)?;
        let pivot = [n as f64 / 2.0, (n + 1) as f64 / 2.0, n as f64 + 0.5 + LIFT];
        let turn = iso.turn(stage, upper, Axis::Z, 2, pivot, 1.0)?;
        iso.play(stage, vec![turn], 0.0)?;
        let down = iso.slide_by(stage, upper, [0.0, 0.0, -(LIFT + 0.5)], 0.6)?;
        iso.play(stage, vec![down], 0.0)?;
        // the model's flat top: the lower halves where the step was, the
        // upper halves where it turns to; each is half a cube tall, level
        let (lo, hi) = iso.bounds(stage, upper);
        let (e_lo, e_hi) = cell_bounds(&empty_cells);
        let ok = (lo[0] - e_lo.0).abs() < 1e-9
            && (lo[1] - e_lo.1).abs() < 1e-9
            && (hi[0] - e_hi.0).abs() < 1e-9
            && (hi[1] - e_hi.1).abs() < 1e-9
            && (lo[2] - n as f64).abs() < 1e-9
            && (hi[2] - (n as f64 + 0.5)).abs() < 1e-9;
        if !ok {
            return Err(fail(format!(
                "the turned halves are not on the empty cells: {lo:?} {hi:?}"
            )));
        }
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 08_3
        let line = say(&kit, stage, LINES[2])?;
        let mut plan_out: Moves = vec![
            leave(stage, step_plan, 0.4)?,
            leave(stage, empty_plan, 0.4)?,
            leave(stage, dot, 0.4)?,
            leave(stage, plan_word, 0.4)?,
        ];
        for &p in pairs.iter().chain([&total]) {
            plan_out.push(leave(stage, p, 0.4)?);
        }
        play(stage, plan_out, 0.0)?;
        // the level top is held a moment; then the braces
        pause(stage, 1.5)?;
        let (w, d) = (r.wide as f64, r.deep as f64);
        let h = n as f64 + 0.5;
        let tall_text = half_number(&r.tall);
        let edges = [
            (
                [iso.project(stage, [0.0, 0.0, 0.0]), iso.project(stage, [w, 0.0, 0.0])],
                v(0.0, -1.0),
                r.wide.to_string(),
            ),
            (
                [iso.project(stage, [w, 0.0, 0.0]), iso.project(stage, [w, d, 0.0])],
                v(1.0, -0.6),
                r.deep.to_string(),
            ),
            (
                [iso.project(stage, [0.0, 0.0, h]), iso.project(stage, [0.0, 0.0, 0.0])],
                v(-1.0, 0.0),
                tall_text.clone(),
            ),
        ];
        let mut braces: Vec<Mob> = Vec::new();
        for (edge, out, word) in &edges {
            let (b, reach) = edge_brace(stage, edge[0], edge[1], *out, ink())?;
            let mid = v((edge[0][0] + edge[1][0]) / 2.0, (edge[0][1] + edge[1][1]) / 2.0);
            let len = (out[0] * out[0] + out[1] * out[1]).sqrt();
            let label = kit.text(
                stage,
                word,
                36.0,
                ink(),
                v(
                    mid[0] + (reach + 0.32) * out[0] / len,
                    mid[1] + (reach + 0.32) * out[1] / len,
                ),
            )?;
            braces.extend([b, label]);
        }
        line.wait_for(stage, "Four wide")?;
        let mut braces_in: Moves = Vec::new();
        for &m in &braces {
            braces_in.push(arrive(stage, m, 0.4)?);
        }
        play(stage, braces_in, 0.25)?;
        line.wait_for(stage, "That makes ninety")?;
        let ninety = kit.tex(
            stage,
            &format!(
                "{} \\times {} \\times {} = {} = {} \\times {}",
                r.wide,
                r.deep,
                half_number_tex(&r.tall),
                r.volume,
                r.three / r.stack,
                r.stack
            ),
            52.0,
            ink(),
            &[],
            v(3.2, 2.4),
        )?;
        show(stage, &[ninety], 0.6, 0.0)?;
        line.wait_end(stage)?;

        // ---------------------------------------------------------- 08_4
        let line = say(&kit, stage, LINES[3])?;
        // numbers first: a third of the box is one stack
        let third = kit.tex(
            stage,
            &format!(
                "{} = \\frac{{1}}{{3}} \\times {} \\times {} \\times {}",
                r.stack,
                r.wide,
                r.deep,
                half_number_tex(&r.tall)
            ),
            48.0,
            ink(),
            &[],
            v(3.2, 1.2),
        )?;
        show(stage, &[third], 0.6, 0.0)?;
        // then the letters, each factor named in turn as it is said
        let parts: [(&str, Srgb); 5] = [
            ("1^{2} + 2^{2} + \\cdots + n^{2} =", ink()),
            ("\\frac{1}{3}", ink()),
            ("n", size(4)),
            ("(n + 1)", size(3)),
            ("(n + \\frac{1}{2})", size(2)),
        ];
        let factors = tex_row(&kit, stage, &parts, 46.0, v(3.2, -0.3), 0.3, 6.6)?;
        let mut labels: Vec<Mob> = Vec::new();
        for (m, word) in factors[2..].iter().zip([WORD_WIDE, WORD_DEEP, WORD_TALL]) {
            let at = stage.get_center(*m);
            labels.push(kit.text(stage, word, 26.0, dim(), v(at[0], at[1] - 0.75))?);
        }
        line.wait_sentence(stage, 1)?;
        let mut letters: Moves = Vec::new();
        for &m in &factors {
            letters.push(arrive(stage, m, 0.5)?);
        }
        play(stage, letters, 0.1)?;
        let mut words_in: Moves = Vec::new();
        for &l in &labels {
            words_in.push(arrive(stage, l, 0.4)?);
        }
        play(stage, words_in, 0.5)?;
        let textbook = lower_third(&kit, stage, PRECISELY, TEXTBOOK)?;
        show(stage, &[textbook], 0.5, 0.0)?;
        line.wait_end(stage)?;
        pause(stage, 1.2)?;

        // ---------------------------------------------------------- 08_5
        let line = say(&kit, stage, LINES[4])?;
        hide(stage, &[textbook], 0.4)?;
        let source = source_line(
            &kit,
            stage,
            &format!("{SOURCE_NAME}, {SOURCE_VOLUME} ({}), p. {}", r.year, r.page),
            false,
        )?;
        show(stage, &[source], 0.6, 0.0)?;
        line.wait_end(stage)?;
        pause(stage, 0.8)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// The polygon of a set of columns on the plane `z`, shifted, in `color`,
/// outline only (an empty half; a ghost that asks a question). Detached.
fn flat_cells(
    iso: &Iso,
    stage: &mut Stage<'_>,
    cells: &[(i64, i64)],
    z: f64,
    shift: (f64, f64),
    color: Srgb,
) -> fmn::Result<Mob> {
    let loops = super::draw::boundary(cells);
    let corners: Vec<Vec3> = loops
        .first()
        .ok_or_else(|| fail("no cells to outline".into()))?
        .iter()
        .map(|&(x, y)| iso.project(stage, [x as f64 + shift.0, y as f64 + shift.1, z]))
        .collect();
    Ok(stage
        .arena_mut()
        .add(Polygon::new(corners).style(line_style(color, 3.0)).build()))
}

/// The least and greatest corner of a set of columns: ((x, y), (x + 1, y + 1)).
fn cell_bounds(cells: &[(i64, i64)]) -> ((f64, f64), (f64, f64)) {
    let min = |f: fn(&(i64, i64)) -> i64| cells.iter().map(f).min().unwrap_or(0) as f64;
    let max = |f: fn(&(i64, i64)) -> i64| cells.iter().map(f).max().unwrap_or(0) as f64 + 1.0;
    ((min(|c| c.0), min(|c| c.1)), (max(|c| c.0), max(|c| c.1)))
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::*;

    /// The chapter speaks exactly the script's 08_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("08_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// The facts are the model's: the box is 4 by 5 by 4½, 90 cubes, three
    /// stacks of 30; the step has ten cubes and twenty halves; Siu, 1984, p.
    /// 92.
    #[test]
    fn the_facts_are_the_models() {
        let r = reads().unwrap();
        assert_eq!((r.wide, r.deep, r.tall.as_str()), (4, 5, "9/2"));
        assert_eq!((r.volume, r.three, r.stack, r.halves), (90, 90, 30, 20));
        assert_eq!((r.year, r.page), (1984, 92));
        assert_eq!(half_number("9/2"), "4½");
        assert_eq!(half_number_tex("9/2"), "4\\frac{1}{2}");
        assert_eq!(half_number("5"), "5");
    }

    /// The empty half is the step turned round: the half turn about the
    /// middle of the top face takes the ten step columns onto the ten empty
    /// ones, the pairs of a column add to five, and together they fill the
    /// 4 by 5 top.
    #[test]
    fn the_empty_half_is_the_step_turned_round() {
        let n = 4;
        let step = stacks::top(n);
        assert_eq!(step.len(), 10);
        let mut empty: Vec<(i64, i64)> = step.iter().map(|&c| stacks::turned_half(n, c)).collect();
        // the half turn about (n/2, (n + 1)/2): x to n − 1 − x, y to n − y
        for (&(x, y), &(tx, ty)) in step.iter().zip(&empty) {
            assert_eq!((tx, ty), (n - 1 - x, n - y));
        }
        let mut all: Vec<(i64, i64)> = step.clone();
        all.append(&mut empty);
        all.sort_unstable();
        let mut want: Vec<(i64, i64)> = (0..n).flat_map(|x| (0..=n).map(move |y| (x, y))).collect();
        want.sort_unstable();
        assert_eq!(all, want, "ten and ten fill the top");
        // columns: 1, 2, 3, 4 step cells, and 4, 3, 2, 1 empty ones
        let cols: Vec<i64> = (0..n).map(|x| step_column(n, x)).collect();
        assert_eq!(cols, [1, 2, 3, 4]);
        for c in cols {
            assert!(c >= 1 && n + 1 - c >= 1);
        }
    }

    /// The model's flat top: every cube of the three stacks below the top
    /// is whole, and the top is the lower halves of the step and its upper
    /// halves turned onto the empty half, 20 half cubes, filling the box
    /// 4 by 5 by 4½ with nothing over.
    #[test]
    fn the_sliced_step_fills_the_box() {
        stacks::fills_the_box(4).unwrap();
        let all = stacks::half_cells(4);
        assert_eq!(all.len(), 2 * 90);
        let at_top = all.iter().filter(|c| c.2 == 2 * 4).count();
        assert_eq!(at_top, 20, "twenty half cubes level");
    }

    /// The scene constructs end to end: the lowered halves end on the model's
    /// empty cells (the scene fails otherwise), and the layout lint finds
    /// nothing.
    #[test]
    fn the_chapter_runs_and_the_layout_lint_is_clean() {
        let ran = testing::run("08_the_box").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        assert!(ran.seconds > 35.0 && ran.seconds < 80.0, "{} s", ran.seconds);
    }
}
