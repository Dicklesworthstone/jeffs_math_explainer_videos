//! Chapter 10, what is left: von Neumann's paragraph, verbatim, with its own
//! price for a fair coin (10_1); what our coin gives, a hundred seeded tosses
//! and the fair ones that drop out of them, against the average on a scale
//! from nothing to a hundred (10_2); the ceiling on that scale, why there is
//! one, and its name (10_3); and the two ways to climb toward it, deeper rows
//! of the triangle and the trick run again on the two things it ignored, as
//! bars that stop short of the ruled line with the counts of a hundred
//! thousand seeded tosses on them (10_4).

use std::ops::Range;

use super::draw::predict_pause;
use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::lower_third;
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, children, choose, fade_all, hex, play, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Block, QuoteCard, Square, SquarePose, alice, bob, column, disc, fade_in_all, fade_in_lagged, fade_out_all, fail,
    flat, gold, group, heads, lettered_disc, line_style, rgb, say, tails, thrown, thrown_lit, wait_until,
};
use crate::STORYBOARD;
use crate::facts::{ELIAS_BLOCKS, PERES_DEPTHS, cited, runs, squeeze_run};
use crate::model::{Face, PairLaw, Q, bent, elias, elias_rate, entropy, pascal_row, peres, peres_rate, von_neumann};

pub struct WhatIsLeft {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)

/// The printed summary's paragraph (Monte Carlo Method, 1951, p. 36), one
/// clause to an entry; the last clause is [`PRICE`] around its number.
const CLAUSES: [&str; 4] = [
    "If independence of successive tosses is assumed, we can reconstruct a 50–50 chance out of even a badly biased coin by tossing twice.",
    "If we get heads-heads or tails-tails, we reject the tosses and try again.",
    "If we get heads-tails (or tails-heads), we accept the result as heads (or tails).",
    "The resulting process is rigorously unbiased,",
];
const PRICE: [&str; 2] = [
    "although the amended process is at most",
    "percent as efficient as ordinary coin-tossing.",
];
/// The source line, around its two years and its page.
const SOURCE: [&str; 2] = [
    "John von Neumann, Various Techniques Used in Connection With Random Digits (summary written by George E. Forsythe). Monte Carlo Method,",
    "From a symposium held June 29 to July 1,",
];
const HEADS_TAILS: &str = "heads-tails";
const TAILS_HEADS: &str = "tails-heads";
const UNBIASED: &str = "rigorously unbiased";
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const A_FAIR_COIN: &str = "a fair coin:";
const FAIR_TOSSES_PER: &str = "fair tosses per";
const ON_AVERAGE: &str = "on average";
const TOSSES_OF_OUR_COIN: &str = "tosses of our coin:";
const FAIR_OUT: &str = "fair tosses out of these";
const WHOLE_RUN: &str = "the whole run of";
const FAIR_TOSSES: &str = "fair tosses,";
const THE_TRICK: &str = "the trick, on average:";
const PER_WORD: &str = "per";
const MODEL: &str = "(model)";
const GUESS: [&str; 2] = ["guess heads: right", "times in"];
const HOLDS_LESS: &str = "so each toss holds less than one fair toss";
const THE_MOST: &str = "the most any rule can average:";
const THE_CEILING: &str = "the ceiling (the coin's entropy):";
const ENTROPY: &str = "entropy";
const PRECISELY: &str = "precisely:";
const CEILING_NOTE: [&str; 3] = [
    "average fair tosses per",
    ", over a long run: at most",
    "(model). Name to search: entropy",
];
const NAMES: &str = "names to search:";
const ELIAS_PERES: [&str; 2] = ["deeper rows: Elias,", ". Reusing both leftovers: Peres,"];
const ROWS_TITLE: &str = "tosses in a row of the triangle";
const ROUNDS_TITLE: &str = "rounds of the trick";
const ALL: &str = "all";
const COUNTED_ONLY: &str = "counted only";
const WHICH_MATCHED: &str = "which pairs matched";
const WHAT_SHOWED: &str = "what the matching pairs showed";
const COUNTED_IN: &str = "counted in";
const TOSSES: &str = "tosses";
const THE_PRICE: &str = "the price: more tosses held at once, and a longer wait";

/// Rates are shown per this many tosses, and the strip holds this many.
const PER: usize = 100;
/// The quote card: characters to a line (the source line needs its two
/// lines) and the most lines the quote may take.
const QUOTE_CHARS: usize = 86;
const QUOTE_LINES: usize = 5;
const CARD_Y: f64 = 2.0;
/// The square at one half under the card, and the note beside it.
const FAIR_AT: Vec3 = [-3.4, -3.2, 0.0];
const FAIR_SIDE: f64 = 2.8;
const NOTE_X: f64 = 0.1;

/// The strip of tosses: pairs to a row, where its first pair sits, the step
/// from pair to pair and from row to row, and a disc's radius.
const ROW_PAIRS: usize = 10;
const STRIP_AT: Vec3 = [-6.35, 3.35, 0.0];
const PAIR_STEP: f64 = 0.52;
const ROW_STEP: f64 = 0.4;
const IN_PAIR: f64 = 0.125;
const DISC_R: f64 = 0.105;
/// The row the fair tosses drop to.
const FAIR_Y: f64 = 0.75;
const FAIR_STEP: f64 = 0.3;

/// The scale: its axis, the height of nothing, and the height of one fair
/// toss per hundred.
const SCALE_X: f64 = 0.0;
const SCALE_FOOT: f64 = -2.35;
const UNIT: f64 = 0.044;
/// Where the ruled line's words end.
const CEILING_RIGHT: f64 = 6.6;
/// The bars: their width, the step between them, and where each panel's
/// first bar stands (the right panel's first is the trick's own bar).
const BAR_W: f64 = 0.46;
const BAR_STEP: f64 = 0.93;
const ELIAS_X: f64 = -6.1;
const PERES_X: f64 = 0.5;

/// The triangle of 10_4, chapter 07's number rows grown downward: its
/// middle, its first row's height and the step down to the next. The first
/// rows are numbers ((size, step from entry to entry), shrinking), the
/// deeper ones strips of cells (their widths).
const TRIANGLE_X: f64 = -3.9;
const TRIANGLE_TOP: f64 = 3.12;
const TRIANGLE_DOWN: f64 = 0.34;
const NUMBER_ROWS: [(f64, f64); 3] = [(28.0, 0.55), (24.0, 0.48), (20.0, 0.4)];
const CELL_ROWS: [f64; 3] = [3.4, 4.1, 4.8];
/// Where the rows' names end: one column left of every row, so a name never
/// reads as the row's first number.
const TRIANGLE_NAMES_X: f64 = -6.5;
/// The left panel's dimmer while the right one builds: its centre and size.
const DIM_AT: Vec3 = [-3.78, 0.2, 0.0];
const DIM_SIZE: (f64, f64) = (6.3, 6.4);
/// The small square of 10_4 and the coins that drop out of it: the square's
/// corner and side (a whole stream of tosses is one side long), the height
/// of each new coin, then where each generation of leftovers starts and how
/// tall its bars are.
const SMALL_AT: Vec3 = [0.55, 1.75, 0.0];
const SMALL_SIDE: f64 = 1.5;
const COIN_Y: [f64; 2] = [3.05, 2.15];
const LEFTOVER_X: [f64; 3] = [2.55, 3.6, 4.3];
const LEFTOVER_H: [f64; 3] = [0.24, 0.12, 0.06];
/// The caption over everything at the end.
const PRICE_AT: Vec3 = [-1.2, 3.62, 0.0];

/// The height of `per_hundred` fair tosses per hundred on the scale.
fn rate_y(per_hundred: f64) -> f64 {
    SCALE_FOOT + UNIT * per_hundred
}

/// How many glyphs `text` draws: its characters that are not spaces.
fn inked(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

/// Which of the quote's glyphs (in reading order, spaces not counted) spell
/// `part`.
fn span(quote: &str, part: &str) -> fmn::Result<Range<usize>> {
    let i = quote
        .find(part)
        .ok_or_else(|| fail(format!("the quote has no {part:?}")))?;
    let start = inked(&quote[..i]);
    Ok(start..start + inked(part))
}

/// Put `text`'s left edge at `at` with its baseline on `at`'s height (its
/// first glyph stands on the line).
fn on_baseline(stage: &mut Stage<'_>, text: Mob, at: Vec3) {
    let first = children(stage, text).first().copied().unwrap_or(text);
    let dy = at[1] - stage.get_bottom(first)[1];
    let dx = at[0] - stage.get_left(text)[0];
    stage.shift(text, v(dx, dy));
}

/// A bar of the scale: `per_hundred` tall, standing at `x`. Detached.
fn bar(stage: &mut Stage<'_>, x: f64, per_hundred: f64) -> Mob {
    let b = column(v(x, rate_y(0.0)), BAR_W, UNIT * per_hundred, Palette::ink()).with_style(flat(Palette::ink(), 0.3));
    stage.arena_mut().add(b)
}

/// A flat bar in coloured stretches, `(share of the width, colour)` each,
/// its left end's middle at `left`. Detached, one group.
fn stretches(stage: &mut Stage<'_>, left: Vec3, (w, h): (f64, f64), parts: &[(f64, Srgb)]) -> fmn::Result<Mob> {
    let mut x = left[0];
    let mut rects = Vec::new();
    for &(share, color) in parts {
        rects.push(rect(v(x, left[1] - h / 2.0), w * share, h, flat(color, 1.0))?);
        x += w * share;
    }
    Ok(stage.arena_mut().add(v_group(rects)))
}

/// Row `k` of the triangle (for `n` tosses), centred on the triangle's
/// middle: its group sizes as numbers in ink while the row is one of the
/// first ones (as chapter 07 ends on them), else a strip of cells, one for
/// each group, brighter the bigger the group. Returns the row as one group.
/// Detached.
fn triangle_row(kit: &Kit, stage: &mut Stage<'_>, k: usize, n: u32) -> fmn::Result<Mob> {
    let y = TRIANGLE_TOP - TRIANGLE_DOWN * k as f64;
    let sizes = pascal_row(n);
    if let Some(&(size, step)) = NUMBER_ROWS.get(k) {
        let mut entries = Vec::new();
        for (i, s) in sizes.iter().enumerate() {
            let x = TRIANGLE_X + step * (i as f64 - f64::from(n) / 2.0);
            entries.push(kit.text(stage, &s.to_string(), size, Palette::ink(), v(x, y))?);
        }
        return group(stage, &entries);
    }
    let w = CELL_ROWS
        .get(k - NUMBER_ROWS.len())
        .copied()
        .ok_or_else(|| fail(format!("the triangle has no width for its row of {n}")))?;
    let h = 0.2;
    let cell = w / f64::from(n + 1);
    let biggest = choose(u64::from(n), u64::from(n / 2)) as f64;
    let mut cells = Vec::new();
    for (i, &s) in sizes.iter().enumerate() {
        let share = s as f64 / biggest;
        let left = TRIANGLE_X - w / 2.0 + cell * (i as f64 + 0.07);
        cells.push(rect(
            v(left, y - h / 2.0),
            cell * 0.86,
            h,
            flat(Palette::ink(), 0.16 + 0.84 * share.powf(0.3)),
        )?);
    }
    Ok(stage.arena_mut().add(v_group(cells)))
}

/// What a round of the trick leaves of a stream of tosses: how long the
/// leftover is (the stream it came from being 1) and its chance of heads.
#[derive(Clone, Copy)]
struct Leftover {
    len: f64,
    heads: f64,
}

impl Leftover {
    /// The two things the trick ignores: which pairs matched (a toss for
    /// every pair; heads when the pair differs), and what the matching
    /// pairs showed (a toss for every matching pair).
    fn leaves(self) -> [Leftover; 2] {
        let (h, t) = (self.heads, 1.0 - self.heads);
        let matched = h * h + t * t;
        [
            Leftover {
                len: self.len / 2.0,
                heads: 1.0 - matched,
            },
            Leftover {
                len: self.len / 2.0 * matched,
                heads: h * h / matched,
            },
        ]
    }
}

/// A white outline around block `b` of a square, detached.
fn outline(stage: &mut Stage<'_>, pose: &SquarePose, b: Block) -> Mob {
    let (u0, w0, u1, w1) = pose.unit_rect(b);
    let quad = [
        pose.point(u0, w0),
        pose.point(u1, w0),
        pose.point(u1, w1),
        pose.point(u0, w1),
    ];
    stage
        .arena_mut()
        .add(Polygon::new(quad).style(line_style(Palette::ink(), 4.0)).build())
}

/// A thin line from `a` to `b`, detached.
fn link(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> Mob {
    stage
        .arena_mut()
        .add(polyline(vec![a, b]).style(line_style(Palette::dim(), 1.5)).build())
}

/// A break in an axis at `x` (the columns either side are not one step
/// apart): a gap in the floor line and two slanted strokes. Detached.
fn axis_break(stage: &mut Stage<'_>, x: f64, y: f64) -> fmn::Result<Vec<Mob>> {
    let gap = Rectangle::new()
        .width(0.09)
        .height(0.06)
        .style(flat(hex(BACKGROUND), 1.0))
        .build()?
        .moved_to(v(x, y));
    let mut out = vec![stage.arena_mut().add(gap)];
    for o in [-0.045, 0.045] {
        let stroke = polyline(vec![v(x + o - 0.035, y - 0.09), v(x + o + 0.035, y + 0.09)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        out.push(stage.arena_mut().add(stroke));
    }
    Ok(out)
}

impl SceneConstruct for WhatIsLeft {
    fn name(&self) -> &str {
        "10_what_is_left"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let per = PER as f64;

        // 10_1: the paragraph, verbatim, clause by clause
        let price = format!("{} {} {}", PRICE[0], cited::EFFICIENCY_PERCENT, PRICE[1]);
        let clauses = [CLAUSES[0], CLAUSES[1], CLAUSES[2], CLAUSES[3], &price];
        let quote = clauses.join(" ");
        let page = cited::PAGES.split('-').next().unwrap_or(cited::PAGES);
        let source = format!(
            "{} {}, p. {page}. {} {}",
            SOURCE[0],
            cited::PRINTED_YEAR,
            SOURCE[1],
            cited::TALK_YEAR
        );
        if !STORYBOARD.contains(&quote) || !STORYBOARD.contains(&source) {
            return Err(fail(
                "the quote card's words are not the storyboard's, character for character".into(),
            ));
        }
        // the paragraph's price is the model's for a fair coin, per hundred tosses
        let fair_rate = Q::HALF * Q::HALF.other();
        let fair_per_hundred = fair_rate.percent(0);
        if fair_per_hundred != cited::EFFICIENCY_PERCENT.to_string() {
            return Err(fail("a fair coin's rate is not the paragraph's price".into()));
        }
        let line = say(&kit, stage, "10_1")?;
        let card = QuoteCard::build(&kit, stage, &quote, &source, v(0.0, CARD_Y), QUOTE_CHARS, QUOTE_LINES)?;
        // the quote's glyphs in reading order, and the line each sits on
        let mut glyphs: Vec<Mob> = Vec::new();
        let mut line_of: Vec<usize> = Vec::new();
        for (i, &l) in card.lines.iter().enumerate() {
            let on_line = children(stage, l);
            line_of.extend(std::iter::repeat_n(i, on_line.len()));
            glyphs.extend(on_line);
        }
        if glyphs.len() != inked(&quote) {
            return Err(fail(format!(
                "the quote card drew {} glyphs for {} characters",
                glyphs.len(),
                inked(&quote)
            )));
        }
        for (words, color) in [(HEADS_TAILS, alice()), (TAILS_HEADS, bob()), (UNBIASED, gold())] {
            for &g in &glyphs[span(&quote, words)?] {
                stage.set_color(g, color, None, false);
            }
        }
        let mut paragraph = vec![card.back];
        paragraph.extend(&card.source);
        fade_in_all(stage, &paragraph, 0.5)?;
        let spans: Vec<Range<usize>> = clauses.iter().map(|c| span(&quote, c)).collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &glyphs[spans[0].clone()], 0.6)?;
        wait_until(stage, line.start + 2.5)?;
        fade_in_all(stage, &glyphs[spans[1].clone()], 0.6)?;
        wait_until(stage, line.start + 4.2)?;
        fade_in_all(stage, &glyphs[spans[2].clone()], 0.6)?;
        line.wait_for(stage, "The printed summary")?;
        fade_in_all(stage, &glyphs[spans[3].clone()], 0.6)?;
        // "which is to say exactly fair": a gold stroke under the two words
        line.wait_for(stage, "which is to say exactly fair")?;
        let under = span(&quote, UNBIASED)?;
        let mut strokes: Vec<Mob> = Vec::new();
        for i in 0..card.lines.len() {
            let on_line: Vec<Mob> = under.clone().filter(|&g| line_of[g] == i).map(|g| glyphs[g]).collect();
            let (Some(&first), Some(&last)) = (on_line.first(), on_line.last()) else {
                continue;
            };
            let low = on_line
                .iter()
                .map(|&g| stage.get_bottom(g)[1])
                .fold(f64::INFINITY, f64::min);
            let (left, right) = (stage.get_left(first)[0], stage.get_right(last)[0]);
            let stroke = polyline(vec![v(left, low - 0.06), v(right, low - 0.06)])
                .style(line_style(gold(), 3.0))
                .build();
            strokes.push(stage.arena_mut().add(stroke));
        }
        for &s in &strokes {
            play!(stage; show_creation(s).rt(0.5));
        }
        // "It also names the price": the square of a fair coin
        line.wait_for(stage, "It also names the price")?;
        let fair = Square::build(stage, FAIR_AT, FAIR_SIDE, Q::HALF.f64())?;
        let paints = [
            (Block::HH, thrown()),
            (Block::HT, alice()),
            (Block::TH, bob()),
            (Block::TT, thrown()),
        ];
        for (b, c) in paints {
            fair.paint(stage, b, c, 1.0);
        }
        let fair_edges = fair.edges(&kit, stage, HEADS, TAILS)?;
        let mut fair_mobs = fair.mobs();
        fair_mobs.push(fair_edges.anchor);
        fair_mobs.extend(fair_edges.stretches);
        fade_in_all(stage, &fair_mobs, 0.6)?;
        line.wait_for(stage, "At most twenty-five")?;
        fade_in_all(stage, &glyphs[spans[4].clone()], 0.6)?;
        wait_until(stage, line.at("as efficient")? + 0.3)?;
        // the price per hundred tosses, as 10_2 and 10_3 count theirs
        let note_y = FAIR_AT[1] + FAIR_SIDE / 2.0;
        let note = [
            kit.text_left(stage, A_FAIR_COIN, 30.0, dim, v(NOTE_X, note_y + 0.3))?,
            kit.text_left(
                stage,
                &format!("{fair_per_hundred} {FAIR_TOSSES_PER} {PER}, {ON_AVERAGE}"),
                30.0,
                ink,
                v(NOTE_X, note_y - 0.3),
            )?,
        ];
        fade_in_all(stage, &note, 0.6)?;

        // 10_2: our coin gives less: these hundred tosses made fifteen fair
        // ones, and the average is sixteen
        let line = say(&kit, stage, "10_2")?;
        let mut gone = paragraph;
        gone.extend(&glyphs);
        gone.extend(&strokes);
        gone.extend(&fair_mobs);
        gone.extend(note);
        fade_out_all(stage, &gone, 0.4)?;
        // the scale: an axis, a tick every ten, numbers at the ends and the middle
        let axis = polyline(vec![v(SCALE_X, rate_y(0.0)), v(SCALE_X, rate_y(per))])
            .style(line_style(dim, 2.5))
            .build();
        let mut scale = vec![stage.arena_mut().add(axis)];
        let mut ticks = Vec::new();
        for t in (0..=PER).step_by(PER / 10) {
            let y = rate_y(t as f64);
            let named = t % (PER / 2) == 0;
            let half = if named { 0.1 } else { 0.05 };
            ticks.push(
                polyline(vec![v(SCALE_X - half, y), v(SCALE_X + half, y)])
                    .style(line_style(dim, 2.0))
                    .build(),
            );
            if named {
                let number = kit.text(stage, &t.to_string(), 20.0, dim, ORIGIN)?;
                stage.move_to(number, v(SCALE_X - 0.18, y), RIGHT);
                scale.push(number);
            }
        }
        scale.push(stage.arena_mut().add(v_group(ticks)));
        fade_in_all(stage, &scale, 0.4)?;
        // the first hundred tosses of the seeded run, two to a pair
        let run = squeeze_run();
        let first = run
            .get(..PER)
            .ok_or_else(|| fail("run 9 is shorter than the strip".into()))?;
        let mut spots: Vec<Vec3> = Vec::new();
        let mut strip: Vec<Mob> = Vec::new();
        for (i, &f) in first.iter().enumerate() {
            let (pair, second) = (i / 2, i % 2 == 1);
            let x = STRIP_AT[0] + PAIR_STEP * (pair % ROW_PAIRS) as f64 + if second { IN_PAIR } else { -IN_PAIR };
            let at = v(x, STRIP_AT[1] - ROW_STEP * (pair / ROW_PAIRS) as f64);
            spots.push(at);
            strip.push(stage.arena_mut().add(disc(f, at, DISC_R)));
        }
        let rows = PER.div_ceil(2 * ROW_PAIRS);
        let strip_left = STRIP_AT[0] - IN_PAIR - DISC_R;
        let strip_note = format!(
            "{PER} {TOSSES_OF_OUR_COIN} {} {HEADS}",
            first.iter().filter(|&&f| f == Face::H).count()
        );
        let strip_note = kit.text_left(
            stage,
            &strip_note,
            22.0,
            dim,
            v(strip_left, STRIP_AT[1] - ROW_STEP * rows as f64 - 0.02),
        )?;
        fade_in_lagged(stage, &strip, 0.9, 0.02)?;
        fade_in_all(stage, &[strip_note], 0.4)?;
        // a matching pair is thrown away; a pair that differs gives its first toss
        line.wait_for(stage, "These hundred tosses")?;
        let mut fades: Vec<Box<dyn Animation>> = Vec::new();
        let mut drops: Vec<(Mob, Vec3)> = Vec::new();
        for (pair, toss) in first.as_chunks::<2>().0.iter().enumerate() {
            if toss[0] == toss[1] {
                for &m in &strip[2 * pair..2 * pair + 2] {
                    let a = m
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.5),
                            ..AnimateArgs::default()
                        })?
                        .set_fill(None, Some(0.28))?;
                    fades.push(stage.prepare(a)?);
                }
            } else {
                let mover = stage.arena_mut().add(disc(toss[0], spots[2 * pair], DISC_R));
                let to = v(strip_left + DISC_R + FAIR_STEP * drops.len() as f64, FAIR_Y);
                drops.push((mover, to));
            }
        }
        let kept = von_neumann(first);
        if kept.len() != drops.len() {
            return Err(fail(
                "the strip's deciding pairs are not the fair tosses the rule gives".into(),
            ));
        }
        stage.play_prepared(fades)?;
        let mut falls: Vec<Box<dyn Animation>> = Vec::new();
        for &(mover, to) in &drops {
            let a = mover
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.8),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            falls.push(stage.prepare(a)?);
        }
        let falls = AnimationGroup::with_lag_ratio(stage.arena_mut(), falls, 0.12)?.with_run_time(1.3);
        stage.play(falls)?;
        // the strip's own count, as it is said
        line.wait_for(stage, "fifteen fair ones")?;
        let fair_note = kit.text_left(
            stage,
            &format!("{} {FAIR_OUT} {PER}", kept.len()),
            22.0,
            dim,
            v(strip_left, FAIR_Y - 0.42),
        )?;
        fade_in_all(stage, &[fair_note], 0.4)?;
        // and the whole run's tally beneath it: fair tosses, about half heads
        let all_fair = von_neumann(&run);
        let all_heads = all_fair.iter().filter(|&&f| f == Face::H).count();
        let whole = [
            format!("{WHOLE_RUN} {}:", thousands(run.len() as u64)),
            format!(
                "{} {FAIR_TOSSES} {} {HEADS}",
                thousands(all_fair.len() as u64),
                thousands(all_heads as u64)
            ),
        ];
        let mut whole_run = Vec::new();
        for (i, words) in whole.iter().enumerate() {
            whole_run.push(kit.text_left(stage, words, 22.0, dim, v(strip_left, FAIR_Y - 0.86 - 0.32 * i as f64))?);
        }
        fade_in_all(stage, &whole_run, 0.4)?;
        // on the scale: what the trick gives on average, as it is said
        line.wait_for(stage, "the average is sixteen")?;
        let law = PairLaw::same_coin(p);
        let trick_rate = law.prob(Face::H, Face::T);
        let trick = bar(stage, PERES_X, per * trick_rate.f64());
        let trick_note = kit.text_left(
            stage,
            &format!("{THE_TRICK} {} {PER_WORD} {PER} {MODEL}", trick_rate.percent(0)),
            24.0,
            ink,
            v(PERES_X + 0.45, rate_y(per * trick_rate.f64()) - 0.02),
        )?;
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), trick, DOWN, None)?.rt(0.6),
            fade_in(stage.arena_mut(), trick_note, ORIGIN, 1.0)?.rt(0.6),
        );
        // could a better rule get all hundred?
        line.wait_for(stage, "Could a better rule")?;
        let reach_from = rate_y(per * trick_rate.f64()) + 0.3;
        let reach = DashedLine::new(v(PERES_X, reach_from), v(PERES_X, rate_y(per) - 0.14))
            .dash_length(0.12)
            .style(line_style(dim, 2.5))
            .build()
            .map_err(|e| fail(format!("the reach to a hundred: {e}")))?;
        let reach = stage.arena_mut().add(reach);
        let tip = Polygon::new([
            v(PERES_X - 0.1, rate_y(per) - 0.17),
            v(PERES_X + 0.1, rate_y(per) - 0.17),
            v(PERES_X, rate_y(per)),
        ])
        .style(flat(dim, 1.0))
        .build();
        let tip = stage.arena_mut().add(tip);
        fade_in_all(stage, &[reach, tip], 0.5)?;
        let asked = predict_pause(&kit, stage, v(PERES_X + 1.0, rate_y(per / 2.0)))?;

        // 10_3: no; a toss you can mostly guess holds less than a fair one,
        // and there is a ceiling, named as it is said
        let line = say(&kit, stage, "10_3")?;
        fade_out_all(stage, &[reach, tip, asked], 0.3)?;
        line.wait_for(stage, "Guess heads")?;
        // heads so many times in so many: the coin's chance as a fraction
        let chance = p.exact();
        let (right, times) = chance
            .split_once('/')
            .and_then(|(n, d)| Some((n.parse::<usize>().ok()?, d.parse::<usize>().ok()?)))
            .ok_or_else(|| fail(format!("the coin's chance {chance} is not a fraction")))?;
        let guess_at = v(4.3, 3.15);
        let mut guesses: Vec<Mob> = Vec::new();
        for i in 0..times {
            let f = if i < right { Face::H } else { Face::T };
            let x = guess_at[0] + 0.7 * (i as f64 - (times as f64 - 1.0) / 2.0);
            let [d, l] = lettered_disc(&kit, stage, f, v(x, guess_at[1]), 0.26, 30.0)?;
            play!(stage;
                fade_in(stage.arena_mut(), d, ORIGIN, 1.0)?.rt(0.25),
                fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?.rt(0.25),
            );
            guesses.extend([d, l]);
        }
        let guess_note = kit.text(
            stage,
            &format!("{} {right} {} {times}", GUESS[0], GUESS[1]),
            26.0,
            ink,
            v(guess_at[0], guess_at[1] - 0.62),
        )?;
        fade_in_all(stage, &[guess_note], 0.5)?;
        line.wait_for(stage, "so each toss")?;
        let holds_less = kit.text(stage, HOLDS_LESS, 24.0, ink, v(guess_at[0], guess_at[1] - 1.08))?;
        fade_in_all(stage, &[holds_less], 0.5)?;
        guesses.extend([guess_note, holds_less]);
        // the ceiling: a ruled line, from the scale across the right half
        line.wait_for(stage, "The most any rule")?;
        let ceiling = per * entropy(p.f64());
        let ceiling_y = rate_y(ceiling);
        let ruled = polyline(vec![v(SCALE_X - 0.5, ceiling_y), v(CEILING_RIGHT, ceiling_y)])
            .style(line_style(ink, 3.5))
            .build();
        let ruled = stage.arena_mut().add(ruled);
        play!(stage; show_creation(ruled).rt(0.8));
        // its words: a fixed value, and in front of it what the line is
        let words_base = ceiling_y + 0.13;
        let value = kit.text(
            stage,
            &format!("{ceiling:.1} {PER_WORD} {PER} {MODEL}"),
            22.0,
            ink,
            ORIGIN,
        )?;
        let value_left = CEILING_RIGHT - stage.get_width(value);
        on_baseline(stage, value, v(value_left, words_base));
        let most = kit.text(stage, THE_MOST, 22.0, ink, ORIGIN)?;
        let most_w = stage.get_width(most);
        on_baseline(stage, most, v(value_left - 0.18 - most_w, words_base));
        fade_in_all(stage, &[most, value], 0.5)?;
        // the word, in gold on the line, as it is spoken
        line.wait_for(stage, "called the coin's entropy")?;
        let named = kit.text_t2c(stage, THE_CEILING, 22.0, ink, &[(ENTROPY, gold())], ORIGIN)?;
        let named_w = stage.get_width(named);
        on_baseline(stage, named, v(value_left - 0.18 - named_w, words_base));
        play!(stage;
            fade_out(stage.arena_mut(), most, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), named, ORIGIN, 1.0)?.rt(0.4),
        );
        let entropy_note = format!(
            "{} {PER}{} {ceiling:.1} {}",
            CEILING_NOTE[0], CEILING_NOTE[1], CEILING_NOTE[2]
        );
        let entropy_note = lower_third(&kit, stage, PRECISELY, &entropy_note)?;
        play!(stage; fade_in(stage.arena_mut(), entropy_note, ORIGIN, 1.0)?.rt(0.5));

        // 10_4: two ways closer, on two panels that share the scale; the left
        // one first, then dimmed while the right one builds
        let line = say(&kit, stage, "10_4")?;
        let mut gone = strip;
        gone.extend(drops.iter().map(|d| d.0));
        gone.extend([strip_note, fair_note, trick_note]);
        gone.extend(&whole_run);
        gone.extend(&guesses);
        fade_out_all(stage, &gone, 0.4)?;
        let ruled_left = polyline(vec![v(SCALE_X - 0.5, ceiling_y), v(-6.6, ceiling_y)])
            .style(line_style(ink, 3.5))
            .build();
        let ruled_left = stage.arena_mut().add(ruled_left);
        play!(stage; show_creation(ruled_left).rt(0.6));
        // each panel's floor, what its bars stand for, and the names under them
        let elias_x = |k: usize| ELIAS_X + BAR_STEP * k as f64;
        let peres_x = |k: usize| PERES_X + BAR_STEP * k as f64;
        let floor_y = rate_y(0.0);
        let mut floors = Vec::new();
        for (from, to) in [
            (elias_x(0) - 0.45, elias_x(ELIAS_BLOCKS.len() - 1) + 0.45),
            (peres_x(0) - 0.3, peres_x(PERES_DEPTHS.len()) + 0.45),
        ] {
            floors.push(link(stage, v(from, floor_y), v(to, floor_y)));
        }
        let panel_mid = |from: f64, to: f64| (from + to) / 2.0;
        floors.push(kit.text(
            stage,
            ROWS_TITLE,
            20.0,
            dim,
            v(panel_mid(elias_x(0), elias_x(ELIAS_BLOCKS.len() - 1)), floor_y - 0.57),
        )?);
        floors.push(kit.text(
            stage,
            ROUNDS_TITLE,
            20.0,
            dim,
            v(panel_mid(peres_x(0), peres_x(PERES_DEPTHS.len())), floor_y - 0.57),
        )?);
        // the trick's bar is the right panel's first: one round
        let number_over = |per_hundred: f64| rate_y(per_hundred) + 0.2;
        let first_round = per * peres_rate(PERES_DEPTHS[0], p.f64());
        floors.push(kit.text(
            stage,
            &PERES_DEPTHS[0].to_string(),
            20.0,
            dim,
            v(peres_x(0), floor_y - 0.25),
        )?);
        floors.push(kit.text(
            stage,
            &format!("{first_round:.0}"),
            20.0,
            ink,
            v(peres_x(0), number_over(first_round)),
        )?);
        fade_in_all(stage, &floors, 0.5)?;

        // the left panel: deeper rows of the triangle, bigger groups
        line.wait_for(stage, "Use the bigger groups")?;
        for (k, &n) in ELIAS_BLOCKS.iter().enumerate() {
            let rate = per * elias_rate(n, p.f64());
            let row = triangle_row(&kit, stage, k, n)?;
            let row_y = TRIANGLE_TOP - TRIANGLE_DOWN * k as f64;
            let row_name = kit.text(stage, &n.to_string(), 20.0, dim, ORIGIN)?;
            stage.move_to(row_name, v(TRIANGLE_NAMES_X, row_y), RIGHT);
            let column = bar(stage, elias_x(k), rate);
            let name = kit.text(stage, &n.to_string(), 20.0, dim, v(elias_x(k), floor_y - 0.25))?;
            let number = kit.text(
                stage,
                &format!("{rate:.0}"),
                20.0,
                ink,
                v(elias_x(k), number_over(rate)),
            )?;
            play!(stage;
                fade_in(stage.arena_mut(), row, v(0.0, 0.12), 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), row_name, ORIGIN, 1.0)?.rt(0.5),
                fmn::animation::grow_from_edge(stage.arena_mut(), column, DOWN, None)?.rt(0.5),
                fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), number, ORIGIN, 1.0)?.rt(0.5),
            );
        }
        kit.lint_here(stage, "10_4 the left panel");

        // the right panel: the trick again, on the two things it ignored;
        // the left panel stands back while it builds
        line.wait_for(stage, "Or run the trick again")?;
        let dimmer = Rectangle::new()
            .width(DIM_SIZE.0)
            .height(DIM_SIZE.1)
            .style(flat(hex(BACKGROUND), 0.75))
            .build()?
            .moved_to(DIM_AT);
        let dimmer = stage.arena_mut().add(dimmer);
        let small = Square::build(stage, SMALL_AT, SMALL_SIDE, p.f64())?;
        let paints = [
            (Block::HH, thrown_lit()),
            (Block::HT, alice()),
            (Block::TH, bob()),
            (Block::TT, thrown_lit()),
        ];
        for (b, c) in paints {
            small.paint(stage, b, c, 1.0);
        }
        let mut ins = vec![dimmer];
        ins.extend(small.mobs());
        fade_in_all(stage, &ins, 0.4)?;
        let pose = small.pose(stage);
        let whole_stream = Leftover {
            len: 1.0,
            heads: p.f64(),
        };
        let two = whole_stream.leaves();
        let decides = law.decides().f64();
        // which pairs matched: the grey part against the coloured part;
        // what the matching pairs showed: the two grey blocks. Each new coin
        // drops out of the square, named large, as it is said
        let split = [
            vec![
                (1.0 - decides, thrown_lit()),
                (decides / 2.0, alice()),
                (decides / 2.0, bob()),
            ],
            vec![(two[1].heads, heads()), (1.0 - two[1].heads, tails())],
        ];
        let from_blocks = [[Block::HT, Block::TH], [Block::HH, Block::TT]];
        let words = [WHICH_MATCHED, WHAT_SHOWED];
        let square_right = SMALL_AT[0] + SMALL_SIDE;
        for i in 0..2 {
            line.wait_for(stage, words[i])?;
            let y = COIN_Y[i];
            let coin = stretches(
                stage,
                v(LEFTOVER_X[0], y),
                (SMALL_SIDE * two[i].len, LEFTOVER_H[0]),
                &split[i],
            )?;
            let word = kit.text_left(stage, words[i], 28.0, ink, v(LEFTOVER_X[0], y - 0.42))?;
            let from = link(stage, v(square_right + 0.06, y), v(LEFTOVER_X[0] - 0.06, y));
            let rings = from_blocks[i].map(|b| outline(stage, &pose, b));
            play!(stage;
                fade_in(stage.arena_mut(), rings[0], ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), rings[1], ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), from, ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), coin, v(0.35, 0.0), 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), word, ORIGIN, 1.0)?.rt(0.5),
            );
            fade_out_all(stage, &rings, 0.3)?;
        }
        // and on what those leave, round after round: shorter each time
        let mut parents: Vec<(Leftover, Vec3)> = (0..2)
            .map(|i| (two[i], v(LEFTOVER_X[0] + SMALL_SIDE * two[i].len, COIN_Y[i])))
            .collect();
        let mut grown = Vec::new();
        for generation in 1..LEFTOVER_X.len() {
            let spread = [0.1, 0.045][generation - 1];
            let mut next = Vec::new();
            for &(parent, end) in &parents {
                for (child, dy) in parent.leaves().into_iter().zip([spread, -spread]) {
                    let left = v(LEFTOVER_X[generation], end[1] + dy);
                    let w = SMALL_SIDE * child.len;
                    grown.push(link(stage, v(end[0] + 0.04, end[1]), v(left[0] - 0.04, left[1])));
                    grown.push(stretches(
                        stage,
                        left,
                        (w, LEFTOVER_H[generation]),
                        &[(1.0, Palette::dim())],
                    )?);
                    next.push((child, v(left[0] + w, left[1])));
                }
            }
            parents = next;
        }
        fade_in_lagged(stage, &grown, 0.6, 0.15)?;
        // the bars for more rounds, each with its rounds and its height
        let mut grows: Vec<Box<dyn Animation>> = Vec::new();
        for (k, &rounds) in PERES_DEPTHS.iter().enumerate().skip(1) {
            let rate = per * peres_rate(rounds, p.f64());
            let column = bar(stage, peres_x(k), rate);
            let a = fmn::animation::grow_from_edge(stage.arena_mut(), column, DOWN, None)?;
            grows.push(stage.prepare(a)?);
            for (words, color, at) in [
                (rounds.to_string(), dim, v(peres_x(k), floor_y - 0.25)),
                (format!("{rate:.0}"), ink, v(peres_x(k), number_over(rate))),
            ] {
                let label = kit.text(stage, &words, 20.0, color, at)?;
                let a = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
                grows.push(stage.prepare(a)?);
            }
        }
        // the rounds skip a step after 4 and after 6: the axis says so
        let mut breaks = Vec::new();
        for (k, pair) in PERES_DEPTHS.windows(2).enumerate() {
            if pair[1] - pair[0] > 1 {
                breaks.extend(axis_break(stage, (peres_x(k) + peres_x(k + 1)) / 2.0, floor_y)?);
            }
        }
        for &m in &breaks {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            grows.push(stage.prepare(a)?);
        }
        let grows = AnimationGroup::with_lag_ratio(stage.arena_mut(), grows, 0.12)?.with_run_time(1.1);
        stage.play(grows)?;
        kit.lint_here(stage, "10_4 the right panel");

        // both climb toward the ceiling: both panels up, and what 100,000
        // seeded tosses gave, on the bars
        line.wait_for(stage, "Both climb toward")?;
        let names = format!(
            "{} {}{} {}",
            ELIAS_PERES[0],
            cited::ELIAS_YEAR,
            ELIAS_PERES[1],
            cited::PERES_YEAR
        );
        let names = lower_third(&kit, stage, NAMES, &names)?;
        play!(stage;
            fade_out(stage.arena_mut(), dimmer, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), entropy_note, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.5),
        );
        let place = |list: &[u32], want: u32, what: &str| {
            list.iter()
                .position(|&x| x == want)
                .ok_or_else(|| fail(format!("no bar for {what} {want}")))
        };
        let counted = [
            (all_fair, peres_x(place(&PERES_DEPTHS, 1, "round")?)),
            (peres(&run, 2), peres_x(place(&PERES_DEPTHS, 2, "rounds")?)),
            (peres(&run, 3), peres_x(place(&PERES_DEPTHS, 3, "rounds")?)),
            (peres(&run, runs::PERES_ALL), peres_x(PERES_DEPTHS.len())),
            (
                elias(&run, runs::ELIAS_BLOCK),
                elias_x(place(&ELIAS_BLOCKS, runs::ELIAS_BLOCK as u32, "a row of")?),
            ),
        ];
        let dot_style = Style::default().fill(dim, 1.0).stroke(hex(BACKGROUND), 1.5, 1.0);
        let mut dots: Vec<Mob> = Vec::new();
        for (out, x) in &counted {
            let got = Q::new((PER * out.len()) as i128, run.len() as i128);
            let y = rate_y(got.f64());
            let dot = Dot::new().point(v(*x, y)).radius(0.065).style(dot_style).build();
            dots.push(stage.arena_mut().add(dot));
            dots.push(kit.text_left(stage, &got.decimal(1), 20.0, dim, v(*x + 0.27, y - 0.02))?);
        }
        let key_at = v(PERES_X - 0.1, rate_y(ceiling) - 0.42);
        let key_dot = Dot::new().point(key_at).radius(0.065).style(dot_style).build();
        dots.push(stage.arena_mut().add(key_dot));
        dots.push(kit.text_left(
            stage,
            &format!("{COUNTED_IN} {} {TOSSES}", thousands(run.len() as u64)),
            20.0,
            dim,
            v(key_at[0] + 0.18, key_at[1]),
        )?);
        // the last column has no model bar: all the rounds there are, counted only
        let last_x = peres_x(PERES_DEPTHS.len());
        dots.push(kit.text(stage, ALL, 20.0, dim, v(last_x, floor_y - 0.25))?);
        dots.push(kit.text(stage, COUNTED_ONLY, 20.0, dim, v(last_x, floor_y - 0.53))?);
        fade_in_lagged(stage, &dots, 0.9, 0.1)?;
        // and neither ever reaches it
        line.wait_for(stage, "and neither ever reaches it")?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), value, 1.08, Some(rgb(ink)))?.rt(0.8));
        wait_until(stage, line.end() - 0.6)?;
        let the_price = kit.text(stage, THE_PRICE, 24.0, ink, PRICE_AT)?;
        fade_in_all(stage, &[the_price], 0.5)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
