//! Chapter 04, by parts: twenty-three times forty-six, rounded first to
//! twenty times fifty, about a thousand (04_1); drawn as a rectangle whose
//! area is the answer (04_2), cut by the parts of its sides into a block,
//! two strips and a corner (04_3); how big the block is, asked (04_4) and
//! answered by its hundreds (04_5); the zeros left out while multiplying and
//! put back by size (04_6); the other
//! three pieces (04_7); and four numbers left to hold at once (04_8).
//!
//! The chapter ends on the picture chapter 05 starts from: the cut
//! rectangle, the four numbers wavering in the head, and the card left
//! waiting at 03_7 in its corner.

use explainer_kit::props::{lower_third, names_to_search};
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::animation::{grow_from_point, indicate};
use fmn::prelude::*;

use super::draw::{
    Board, CutRect, NOTE_PT, SIDE_PT, Side, dim, eq, fade_in_all, fade_out_all, fail, flat, fly, glyphs, in_your_head,
    middle_of, num, panel, paper_ink, quiz, repaint, rgb, say, slate, tag, waiting_card, waver, why_pause,
};
use crate::facts::shown;
use crate::model::{Piece, by_parts, estimate, lead_and_zeros};

pub struct ByParts {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ABOUT: &str = "about";
const DISTRIBUTIVE: &str = "the distributive law";
const FOR_PROGRAMMERS: &str = "for programmers:";
const BOTH_STRIPS: &str = "both strips at once, in tens.";
const MULTIPLICATIONS_NOT: &str = "multiplications, not";
const KARATSUBA: &str = "Karatsuba's algorithm";
/// Counts spelled out in a sentence, by their value.
const COUNT_WORDS: [&str; 11] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
];

/// The board of chapters 04 and 05: twenty-three by forty-six nearly fills
/// the figure's width (chapter 05 starts from it: `running_total.rs` keeps
/// the same numbers).
const UNIT: f64 = 0.14;
const BOARD_AT: Vec3 = [-6.1, -1.9, 0.0];
/// The faint grid of tens behind the rectangle, in units.
const GRID: (f64, f64) = (50.0, 30.0);
/// "in your head" at 04_8, in the column, and where the four numbers hang.
const HEAD_AT: Vec3 = [3.9, -0.35, 0.0];
const HEAD_SIZE: (f64, f64) = (5.6, 4.0);
const HELD_AT: [Vec3; 4] = [
    [2.55, 0.75, 0.0],
    [5.15, 0.55, 0.0],
    [2.85, -1.35, 0.0],
    [5.25, -1.15, 0.0],
];
const HELD_PT: f64 = 84.0;

/// The middle of the column beside this rectangle (it is wider than the
/// kit's figure, so the column starts right of its grid, at x = 1.1).
const COL_X: f64 = 4.05;
/// The problem and its estimate at 04_1, and the problem at the head of
/// the column after it.
const BIG_PT: f64 = 110.0;
const TAG_PT: f64 = 40.0;
const TAG_GAP: f64 = 0.35;
const PROBLEM_PT: f64 = 60.0;
const PROBLEM_AT: Vec3 = [COL_X, 2.2, 0.0];
/// The sums in the column, and the number on the block.
const SUM_PT: f64 = 56.0;
const BLOCK_PT: f64 = 64.0;
/// The three candidates of 04_4: their type and their cards.
const CANDIDATE_PT: f64 = 60.0;
const CANDIDATE: (f64, f64) = (1.7, 1.05);
const CANDIDATE_GAP: f64 = 1.88;
const CANDIDATES_X: f64 = 3.95;
const CANDIDATES_Y: f64 = 0.75;
/// Where "=" stands in the column's sums at 04_7, and their heights.
const EQUALS_X: f64 = COL_X + 0.25;
const SUM_YS: [f64; 4] = [1.15, 0.25, -0.65, -1.55];

/// How far the plate behind the block's number reaches beyond its figures.
const PLATE_PAD: f64 = 0.14;

/// A plate in `color` just behind `parts` (a label's pieces): it hides what
/// is painted under it, here the ruled lines of a block, where the figures
/// stand. The layout lint takes a fill to hide a stroke over its middle
/// two thirds, so a label of `w` units across gets at least (`w` - 0.6) / 4
/// beyond its figures on each side (and `pad` at the least), for its outer
/// figures to stand inside that. Detached.
fn label_plate(stage: &mut Stage<'_>, parts: &[Mob], color: Srgb, pad: f64) -> fmn::Result<Mob> {
    let (mut left, mut right, mut bottom, mut top) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &m in parts {
        left = left.min(stage.get_left(m)[0]);
        right = right.max(stage.get_right(m)[0]);
        bottom = bottom.min(stage.get_bottom(m)[1]);
        top = top.max(stage.get_top(m)[1]);
    }
    if parts.is_empty() || left > right {
        return Err(fail("a label plate needs the figures it stands behind".into()));
    }
    let pad_x = pad.max((right - left - 0.6) / 4.0);
    let plate = Rectangle::new()
        .width(right - left + 2.0 * pad_x)
        .height(top - bottom + 2.0 * pad)
        .corner_radius(0.08)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(v((left + right) / 2.0, (bottom + top) / 2.0));
    Ok(stage.arena_mut().add(plate))
}

/// A count as a word, for a sentence on screen.
fn count_word(n: usize) -> fmn::Result<&'static str> {
    COUNT_WORDS
        .get(n)
        .copied()
        .ok_or_else(|| fail(format!("no word for the count {n}")))
}

fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// Text or mathematics, typeset on its own at the origin. Detached.
fn typeset(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, tex: bool) -> fmn::Result<Mob> {
    if tex {
        kit.tex(stage, src, size, color, &[], ORIGIN)
    } else {
        kit.text(stage, src, size, color, ORIGIN)
    }
}

/// `whole` (already in place, never shown) cut into `parts` that move on
/// their own: each part is typeset alone and laid over its own glyphs in the
/// whole, so the parts read as the whole until one of them moves. Detached.
fn cut_into(
    kit: &Kit,
    stage: &mut Stage<'_>,
    whole: Mob,
    parts: &[&str],
    size: f64,
    color: Srgb,
    tex: bool,
) -> fmn::Result<Vec<Mob>> {
    let all = glyphs(stage, whole);
    let mut out = Vec::new();
    let mut k = 0;
    for &part in parts {
        let m = typeset(kit, stage, part, size, color, tex)?;
        let n = glyphs(stage, m).len();
        if n == 0 || k + n > all.len() {
            return Err(fail(format!("{part:?} is not where it should be in {parts:?}")));
        }
        let at = middle_of(stage, &all[k..k + n])?;
        stage.move_to(m, at, ORIGIN);
        out.push(m);
        k += n;
    }
    if k != all.len() {
        return Err(fail(format!("{parts:?} leave glyphs over")));
    }
    Ok(out)
}

/// A line of mathematics in `parts`, its `edge` at `at`. Detached.
fn math_parts(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[&str],
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<Vec<Mob>> {
    let whole = kit.tex(stage, &parts.join(" "), size, color, &[], ORIGIN)?;
    stage.move_to(whole, at, edge);
    cut_into(kit, stage, whole, parts, size, color, true)
}

/// A side's label in parts ("2", "0"), where `Board::label` puts the whole
/// label. Detached.
fn side_label(
    kit: &Kit,
    stage: &mut Stage<'_>,
    board: &Board,
    parts: &[&str],
    r: [f64; 4],
    side: Side,
) -> fmn::Result<Vec<Mob>> {
    let whole = board.label(kit, stage, &parts.concat(), r, side, SIDE_PT, Palette::ink())?;
    cut_into(kit, stage, whole, parts, SIDE_PT, Palette::ink(), false)
}

/// A number's parts as its label shows them: the leading digits, then each
/// zero on its own (20 is "2", "0").
fn lead_parts(x: u64) -> Vec<String> {
    let (lead, zeros) = lead_and_zeros(x);
    let mut out = vec![num(lead)];
    out.extend((0..zeros).map(|_| "0".to_owned()));
    out
}

fn refs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

/// Fade `mob` in as if it slid out of `from`, `rt` seconds.
fn slide_in(stage: &mut Stage<'_>, mob: Mob, from: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let c = stage.get_center(mob);
    let a = fade_in(stage.arena_mut(), mob, v(c[0] - from[0], c[1] - from[1]), 1.0)?;
    stage.prepare(a.rt(rt))
}

/// Move `mob`'s centre to `to`, scaling it by `scale` and recolouring it on
/// the way, `rt` seconds.
fn fly_paint(mob: Mob, to: Vec3, scale: f64, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(scale)?
        .move_to(to, ORIGIN)?
        .set_color(color)?)
}

/// Prepared animations played one after another, overlapping by `lag`, the
/// whole taking `rt` seconds.
fn play_lagged(stage: &mut Stage<'_>, anims: Vec<Box<dyn Animation>>, rt: f64, lag: f64) -> fmn::Result<()> {
    if anims.is_empty() {
        return Ok(());
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// One of the three candidates of 04_4: a number on a card edged in the
/// question's colour. Detached: the card, then the number.
fn candidate(kit: &Kit, stage: &mut Stage<'_>, n: u64, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let number = eq(kit, stage, &num(n), CANDIDATE_PT, at)?;
    let back = Rectangle::new()
        .width(CANDIDATE.0)
        .height(CANDIDATE.1)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.96).stroke(quiz(), 2.5, 1.0))
        .build()?
        .moved_to(at);
    Ok([stage.arena_mut().add(back), number])
}

/// `h × w = area` in the column, the area in its piece's colour, its "=" at
/// `EQUALS_X`. Detached.
fn product_line(kit: &Kit, stage: &mut Stage<'_>, p: &Piece, color: Srgb, y: f64) -> fmn::Result<Mob> {
    let src = format!(r"{} \times {} = {}", p.height, p.width, p.area());
    let line = kit.tex(
        stage,
        &src,
        SUM_PT,
        Palette::ink(),
        &[(&num(p.area()), color)],
        v(EQUALS_X, y),
    )?;
    let all = glyphs(stage, line);
    let equals = num(p.height).len() + 1 + num(p.width).len();
    let at = all
        .get(equals)
        .map(|&g| stage.get_center(g))
        .ok_or_else(|| fail(format!("{src} has no equals sign where it should")))?;
    stage.shift(line, v(EQUALS_X - at[0], 0.0));
    Ok(line)
}

impl SceneConstruct for ByParts {
    fn name(&self) -> &str {
        "04_by_parts"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (a, b) = shown::PARTS;
        let parts = by_parts(a, b);
        if parts.pieces.len() != 4 {
            return Err(fail(format!("{a} × {b} is drawn as four pieces: {parts:?}")));
        }
        let block = parts.pieces[0];
        let ink = Palette::ink();
        // the card left waiting at 03_7, already in its corner
        for m in waiting_card(&kit, stage, shown::FIVE_HARD)? {
            stage.add_to_scene(m)?;
        }

        // 04_1: round first: each number slides to its nearest ten, and
        // twenty times fifty is about a thousand
        let line = say(&kit, stage, "04_1")?;
        let (ra, rb, about) = estimate(a, b);
        let rough_src = [num(ra), r"\times".to_owned(), num(rb), "=".to_owned(), num(about)];
        let about_tag = tag(&kit, stage, ABOUT, ORIGIN, TAG_PT)?;
        // the two lines share their left edge, so each round number lands
        // under its own number; the estimate and its tag are centred
        let rough_w = {
            let whole = kit.tex(stage, &rough_src.join(" "), BIG_PT, ink, &[], ORIGIN)?;
            stage.get_width(whole)
        };
        let left_x = -0.4 - (rough_w + TAG_GAP + stage.get_width(about_tag)) / 2.0;
        let problem_src = [num(a), r"\times".to_owned(), num(b)];
        let problem = math_parts(&kit, stage, &refs(&problem_src), BIG_PT, ink, v(left_x, 1.45), LEFT)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &problem, 0.6)?;
        line.wait_sentence(stage, 2)?;
        let rough = math_parts(&kit, stage, &refs(&rough_src), BIG_PT, ink, v(left_x, -0.55), LEFT)?;
        // each number slides down to its nearest ten: a copy leaves the
        // problem and turns into the round number on the way
        let mut anims = Vec::new();
        for (from, to, x) in [(problem[0], rough[0], a), (problem[2], rough[2], b)] {
            let at = stage.get_center(from);
            let copy = eq(&kit, stage, &num(x), BIG_PT, at)?;
            stage.add_to_scene(copy)?;
            let goal = stage.get_center(to);
            let shift = v(goal[0] - at[0], goal[1] - at[1]);
            let gone = fade_out(stage.arena_mut(), copy, shift, 1.0)?;
            anims.push(stage.prepare(gone.rt(1.0))?);
            anims.push(slide_in(stage, to, at, 1.0)?);
        }
        let times_at = stage.get_center(problem[1]);
        anims.push(slide_in(stage, rough[1], times_at, 1.0)?);
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 3)?;
        fade_in_all(stage, &rough[3..], 0.5)?;
        line.wait_for(stage, "near a thousand")?;
        let tail = stage.get_right(rough[4]);
        stage.move_to(about_tag, v(tail[0] + TAG_GAP, tail[1]), LEFT);
        play!(stage; fade_in(stage.arena_mut(), about_tag, v(-0.2, 0.0), 1.0)?.rt(0.4));

        // 04_2: now draw it: a rectangle twenty-three by forty-six over the
        // faint grid of tens; the problem goes to the head of the column
        let line = say(&kit, stage, "04_2")?;
        let mut gone = rough.clone();
        gone.push(about_tag);
        fade_out_all(stage, &gone, 0.5)?;
        let centre = middle_of(stage, &problem)?;
        let k = PROBLEM_PT / BIG_PT;
        let mut anims = Vec::new();
        for &m in &problem {
            let c = stage.get_center(m);
            let to = v(
                PROBLEM_AT[0] + k * (c[0] - centre[0]),
                PROBLEM_AT[1] + k * (c[1] - centre[1]),
            );
            anims.push(stage.prepare(fly(m, to, k, 0.8)?)?);
        }
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 1)?;
        let board = Board::new(BOARD_AT, UNIT);
        let grid = board.tens(stage, GRID.0, GRID.1)?;
        let whole_r = [0.0, 0.0, b as f64, a as f64];
        let whole = board.rect(stage, whole_r, flat(slate(), 1.0));
        play!(stage; fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; grow_from_point(stage.arena_mut(), whole, board.p(0.0, 0.0), None)?.rt(0.9));
        let tall_label = board.label(&kit, stage, &num(a), whole_r, Side::Left, SIDE_PT, ink)?;
        let wide_label = board.label(&kit, stage, &num(b), whole_r, Side::Below, SIDE_PT, ink)?;
        fade_in_all(stage, &[tall_label, wide_label], 0.5)?;

        // 04_3: each number breaks into its parts; the rectangle falls into
        // four pieces, which take their colours one after another
        let line = say(&kit, stage, "04_3")?;
        let cut = CutRect::build(stage, board, a, b, slate());
        for &m in &cut.pieces {
            stage.add_to_scene(m)?;
        }
        stage.remove_from_scene(whole);
        // the side labels, as parts: the tens' zeros on their own
        let (a_tens, a_ones) = crate::model::parts(a);
        let (b_tens, b_ones) = crate::model::parts(b);
        let rect_of = |h: u64, w: u64| -> fmn::Result<[f64; 4]> {
            parts
                .pieces
                .iter()
                .position(|p| p.height == h && p.width == w)
                .map(|i| cut.rects[i])
                .ok_or_else(|| fail(format!("{a} × {b} has no piece {h} by {w}")))
        };
        let block_r = rect_of(a_tens, b_tens)?;
        let top_r = rect_of(a_ones, b_tens)?;
        let side_r = rect_of(a_tens, b_ones)?;
        let tens_a = lead_parts(a_tens);
        let tens_b = lead_parts(b_tens);
        let left_tens = side_label(&kit, stage, &board, &refs(&tens_a), block_r, Side::Left)?;
        let left_ones = side_label(&kit, stage, &board, &[&num(a_ones)], top_r, Side::Left)?;
        let below_tens = side_label(&kit, stage, &board, &refs(&tens_b), block_r, Side::Below)?;
        let below_ones = side_label(&kit, stage, &board, &[&num(b_ones)], side_r, Side::Below)?;
        line.wait_sentence(stage, 1)?;
        let from = stage.get_center(tall_label);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a_cut = fade_in(stage.arena_mut(), cut.cuts[0], ORIGIN, 1.0)?;
        anims.push(stage.prepare(a_cut.rt(0.6))?);
        let old = fade_out(stage.arena_mut(), tall_label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(old.rt(0.5))?);
        for &m in left_tens.iter().chain(&left_ones) {
            anims.push(slide_in(stage, m, from, 0.7)?);
        }
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 2)?;
        let from = stage.get_center(wide_label);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let b_cut = fade_in(stage.arena_mut(), cut.cuts[1], ORIGIN, 1.0)?;
        anims.push(stage.prepare(b_cut.rt(0.6))?);
        let old = fade_out(stage.arena_mut(), wide_label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(old.rt(0.5))?);
        for &m in below_tens.iter().chain(&below_ones) {
            anims.push(slide_in(stage, m, from, 0.7)?);
        }
        stage.play_prepared(anims)?;
        // the pieces take their roles' colours as they are named
        for (phrase, which) in [("big block", &[0][..]), ("two strips", &[1, 2]), ("small corner", &[3])] {
            line.wait_for(stage, phrase)?;
            let mut anims = Vec::new();
            for &i in which {
                anims.push(stage.prepare(repaint(cut.pieces[i], cut.role(i), 1.0, 0.5)?)?);
            }
            stage.play_prepared(anims)?;
        }

        // 04_4: the block lights; two times four is eight, but how big is
        // the block: eighty, eight hundred or eight thousand?
        let line = say(&kit, stage, "04_4")?;
        let others: Vec<Mob> = cut.pieces[1..].to_vec();
        let mut quiet = others.clone();
        quiet.extend(left_ones.iter().chain(&below_ones));
        let mut anims = Vec::new();
        for &m in &quiet {
            anims.push(stage.prepare(dim(m, 0.3, 0.6)?)?);
        }
        let lit = indicate(stage.arena_mut(), cut.pieces[0], 1.0, Some(rgb(ink)))?;
        anims.push(stage.prepare(lit.rt(0.8))?);
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 1)?;
        let (lead, zeros) = block.by_size();
        let (h_lead, _) = lead_and_zeros(block.height);
        let (w_lead, _) = lead_and_zeros(block.width);
        // the tens' zeros step back while the leading digits multiply
        let label_zeros: Vec<Mob> = left_tens[1..].iter().chain(&below_tens[1..]).copied().collect();
        if label_zeros.len() != zeros as usize {
            return Err(fail(format!(
                "{} zeros on the labels, but the block has {zeros}",
                label_zeros.len()
            )));
        }
        let small_src = [
            num(h_lead),
            r"\times".to_owned(),
            num(w_lead),
            "=".to_owned(),
            num(lead),
        ];
        let block_at = cut.centre(0);
        let small = math_parts(&kit, stage, &refs(&small_src), BLOCK_PT, paper_ink(), block_at, ORIGIN)?;
        let mut anims = Vec::new();
        for &m in &label_zeros {
            anims.push(stage.prepare(dim(m, 0.3, 0.5)?)?);
        }
        for &m in &small {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        // the three sizes it could be, as they are said
        let sizes = [block.area() / 10, block.area(), block.area() * 10];
        let mut cards = Vec::new();
        for (i, (&n, phrase)) in sizes
            .iter()
            .zip(["eighty,", "eight hundred,", "eight thousand"])
            .enumerate()
        {
            let x = CANDIDATES_X + CANDIDATE_GAP * (i as f64 - 1.0);
            let card = candidate(&kit, stage, n, v(x, CANDIDATES_Y))?;
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &card, 0.35)?;
            cards.push(card);
        }
        why_pause(&kit, stage)?;

        // 04_5: eight hundred: twenty times four is eighty already, and the
        // block's tens grid brightens into eight squares of a hundred
        let line = say(&kit, stage, "04_5")?;
        let wrong: Vec<Mob> = [cards[0], cards[2]].iter().flatten().copied().collect();
        let right_back = cards[1][0];
        let mut anims = Vec::new();
        for &m in &wrong {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let edge = right_back
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                ..AnimateArgs::default()
            })?
            .set_stroke(Some(ink), None, Some(1.0))?;
        anims.push(stage.prepare(edge)?);
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 1)?;
        let tens_by_ones = Piece {
            height: block.height,
            width: w_lead,
        };
        if tens_by_ones.area() * 10 != block.area() {
            return Err(fail(format!("{tens_by_ones:?} is not a tenth of the block")));
        }
        let tenth_src = format!(r"{} \times {} = {}", block.height, w_lead, tens_by_ones.area());
        let tenth = eq(&kit, stage, &tenth_src, SUM_PT, v(CANDIDATES_X, -0.75))?;
        play!(stage; fade_in(stage.arena_mut(), tenth, v(0.0, -0.12), 1.0)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        let tiles = board.hundreds(&kit, stage, block_r, false)?;
        // the block's number stands on a plate of the block's own colour,
        // painted over the ruled lines and under the figures, so that the
        // lines stop at its edge and do not run through "2 × 4 = 8" and
        // then "800" (the layout lint's `behind`, 04_6 and 04_7)
        let plate = label_plate(stage, &small, cut.role(0), PLATE_PAD)?;
        play!(stage;
            fade_in(stage.arena_mut(), tiles.lines, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), plate, ORIGIN, 1.0)?.rt(0.8),
        );
        stage.scene_mut().bring_to_front(&small)?;

        // 04_6: leave the zeros out, then put them back by size: 2 × 4 = 8
        // becomes 800
        let line = say(&kit, stage, "04_6")?;
        let mut gone = cards[1].to_vec();
        gone.push(tenth);
        fade_out_all(stage, &gone, 0.5)?;
        let block_number = eq(&kit, stage, &num(block.area()), BLOCK_PT, block_at)?;
        stage.set_color(block_number, paper_ink(), None, true);
        let block_src = [num(lead), "0".repeat(zeros as usize)];
        let block_parts = cut_into(
            &kit,
            stage,
            block_number,
            &refs(&block_src),
            BLOCK_PT,
            paper_ink(),
            true,
        )?;
        line.wait_for(stage, "put them back")?;
        let landing: Vec<Vec3> = glyphs(stage, block_parts[1])
            .iter()
            .map(|&g| stage.get_center(g))
            .collect();
        // each zero leaves its label (which shows it again once it has
        // gone) and lands in the number it joins
        let fliers = zeros_from(&kit, stage, &label_zeros, SIDE_PT)?;
        for &m in &label_zeros {
            stage.set_opacity(m, 0.0, true);
        }
        let mut anims = Vec::new();
        for &m in &small[..small.len() - 1] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let eight = small[small.len() - 1];
        anims.push(stage.prepare(fly(eight, stage.get_center(block_parts[0]), 1.0, 1.0)?)?);
        for (&m, &to) in fliers.iter().zip(&landing) {
            anims.push(stage.prepare(fly_paint(m, to, BLOCK_PT / SIDE_PT, paper_ink(), 1.0)?)?);
        }
        for &m in &label_zeros {
            anims.push(stage.prepare(dim(m, 1.0, 1.0)?)?);
        }
        stage.play_prepared(anims)?;
        stage.add_to_scene(block_number)?;
        stage.remove_from_scene(eight);
        for &m in &fliers {
            stage.remove_from_scene(m);
        }
        // 04_7: the other three pieces the same way, each with its sides and
        // its number; then the name of the rule
        let line = say(&kit, stage, "04_7")?;
        let mut anims = Vec::new();
        let grid_out = fade_out(stage.arena_mut(), tiles.lines, ORIGIN, 1.0)?;
        anims.push(stage.prepare(grid_out.rt(0.5))?);
        let plate_out = fade_out(stage.arena_mut(), plate, ORIGIN, 1.0)?;
        anims.push(stage.prepare(plate_out.rt(0.5))?);
        for &m in &quiet {
            anims.push(stage.prepare(dim(m, 1.0, 0.5)?)?);
        }
        stage.play_prepared(anims)?;
        let mut sums = vec![product_line(&kit, stage, &block, cut.role(0), SUM_YS[0])?];
        play!(stage; fade_in(stage.arena_mut(), sums[0], v(0.0, -0.1), 1.0)?.rt(0.5));
        let mut numbers = vec![block_number];
        for (i, piece) in parts.pieces.iter().enumerate().skip(1) {
            line.wait_sentence(stage, i)?;
            let sides: Vec<Mob> = [
                (piece.height, &left_tens, &left_ones, a_tens),
                (piece.width, &below_tens, &below_ones, b_tens),
            ]
            .iter()
            .flat_map(|&(x, tens, ones, t)| if x == t { tens.clone() } else { ones.clone() })
            .collect();
            let sum = product_line(&kit, stage, piece, cut.role(i), SUM_YS[i])?;
            let on_piece = eq(&kit, stage, &num(piece.area()), NOTE_PT, cut.centre(i))?;
            stage.set_color(on_piece, paper_ink(), None, true);
            let mut anims = Vec::new();
            for &m in &sides {
                let a = indicate(stage.arena_mut(), m, 1.35, Some(rgb(ink)))?;
                anims.push(stage.prepare(a.rt(0.7))?);
            }
            for m in [sum, on_piece] {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.6))?);
            }
            stage.play_prepared(anims)?;
            sums.push(sum);
            numbers.push(on_piece);
        }
        let names = names_to_search(&kit, stage, &[DISTRIBUTIVE])?;
        pause_then_show(stage, names, 0.4)?;

        // 04_8: four easy multiplications, and four numbers to hold while
        // you add them: they lift off the rectangle into the head and waver;
        // for programmers, three multiplications instead of four
        let line = say(&kit, stage, "04_8")?;
        // the name of the rule keeps the foot band until the next
        // lower-third takes it
        let mut gone = sums.clone();
        gone.extend(&problem);
        fade_out_all(stage, &gone, 0.5)?;
        let head = in_your_head(&kit, stage, HEAD_AT, HEAD_SIZE.0, HEAD_SIZE.1)?;
        fade_in_all(stage, &head, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let sizes = [BLOCK_PT, NOTE_PT, NOTE_PT, NOTE_PT];
        stage.scene_mut().bring_to_front(&numbers)?;
        let mut anims = Vec::new();
        for ((&m, &to), &size) in numbers.iter().zip(&HELD_AT).zip(&sizes) {
            anims.push(stage.prepare(fly_paint(m, to, HELD_PT / size, ink, 1.0)?)?);
        }
        play_lagged(stage, anims, 1.8, 0.35)?;
        // the numbers start to waver as the adding is named, and the
        // lower-third takes the foot band to itself through the hold
        let (a1, a0, b1, b0) = (a / 10, a % 10, b / 10, b % 10);
        let products = [a1 * b1, a0 * b0, (a1 + a0) * (b1 + b0)];
        let strips = products[2] - products[0] - products[1];
        if strips * 10 != parts.pieces[1].area() + parts.pieces[2].area() {
            return Err(fail(format!("{strips} tens is not the two strips of {parts:?}")));
        }
        let three = capitalised(count_word(products.len())?);
        let four = count_word(parts.pieces.len())?;
        let karatsuba = format!(
            "({a1} + {a0}) × ({b1} + {b0}) − {} − {} = {strips}: {BOTH_STRIPS} {three} {MULTIPLICATIONS_NOT} {four}: {KARATSUBA}",
            products[0], products[1]
        );
        let note = lower_third(&kit, stage, FOR_PROGRAMMERS, &karatsuba)?;
        line.wait_for(stage, "while you add them")?;
        let amount = stage.add_value_tracker(0.0);
        let anchor = waver(stage, &numbers, amount)?;
        play!(stage;
            tween(amount, 1.0, 1.0)?,
            fade_out(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), note, v(0.0, 0.15), 1.0)?.rt(0.5),
        );
        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        // chapter 05 starts from what is left: settle the numbers and clear
        // the foot band
        play!(stage;
            tween(amount, 0.0, 0.6)?,
            fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.6),
        );
        stage.clear_updaters(anchor, true);
        kit.finish(stage)
    }
}

/// Copies of the zeros in `zeros` (glyphs of labels or of a line), typeset
/// at `size`, each over its zero, in the scene. They fly where a number
/// needs its zeros back.
fn zeros_from(kit: &Kit, stage: &mut Stage<'_>, zeros: &[Mob], size: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for &z in zeros {
        let at = stage.get_center(z);
        let copy = eq(kit, stage, "0", size, at)?;
        stage.add_to_scene(copy)?;
        out.push(copy);
    }
    Ok(out)
}

/// A short breath, then `mob` fades in: a lower-third once the beat's
/// picture has landed.
fn pause_then_show(stage: &mut Stage<'_>, mob: Mob, wait: f64) -> fmn::Result<()> {
    super::draw::pause(stage, wait)?;
    play!(stage; fade_in(stage.arena_mut(), mob, v(0.0, 0.15), 1.0)?.rt(0.5));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::draw::{END_HOLD_S, testing};

    /// The numbers held in the head waver as the adding is named, and keep
    /// wavering through the music breath that ends the chapter, up to the
    /// settle that clears them (bead mx-e-a-f-ms4.10).
    #[test]
    fn the_held_numbers_keep_moving_through_the_chapters_last_hold() {
        let run = testing::run_voiced("04_by_parts").unwrap();
        let frames = testing::frames_of(&run.frames_dir).unwrap();
        let (start, seconds) = *run.breaths.last().expect("04 ends on a music breath");
        assert!((seconds - END_HOLD_S).abs() < 1e-9, "the hold is {seconds} s");
        let fps = f64::from(testing::FPS);
        let (from, to) = (
            (start * fps).round() as usize,
            ((start + seconds) * fps).round() as usize,
        );
        let held = to - from;
        let moving = testing::changes(&frames, from, to);
        assert!(
            moving + 1 >= held,
            "{moving} of the {held} frames of the last hold differ from the frame before: the numbers froze"
        );
        // the same check sees a still picture where there is one: this chapter
        // stands still for whole seconds elsewhere
        let (_, still) = testing::longest_still(&frames);
        assert!(
            still >= 10,
            "the longest still stretch of the chapter is {still} frames, so the check proves little"
        );
    }
}
