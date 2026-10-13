//! Chapter 10, what is left: von Neumann's paragraph, verbatim, with its own
//! price for a fair coin (10_1); what our coin gives, a hundred seeded tosses
//! and the fair ones that drop out of them, beside a scale from nothing to a
//! hundred (10_2); the ceiling on that scale and why there is one (10_3); and
//! the two ways to climb toward it, deeper rows of the triangle and the trick
//! run again on its two leftovers, as bars that stop short of the ruled line
//! with the counts of a hundred thousand seeded tosses on them (10_4).

use std::ops::Range;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, children, choose, fade_all, hex, play, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    Block, QuoteCard, Square, SquarePose, alice, bob, column, disc, fade_in_all, fade_in_lagged, fade_out_all, fail,
    flat, gold, heads, lettered_disc, line_style, rgb, say, tails, thrown, thrown_lit, wait_until,
};
use crate::STORYBOARD;
use crate::facts::{ELIAS_BLOCKS, PERES_DEPTHS, cited, runs, squeeze_run};
use crate::model::{Face, PairLaw, Q, bent, elias, elias_rate, entropy, peres, peres_rate, von_neumann};

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
const ONE_FAIR_TOSS: &str = "one fair toss for every";
/// How many tosses a fair coin spends on each fair one, as the note says it
/// and as a number the model is held to.
const EVERY: (&str, i128) = ("four", 4);
const TOSSES_OF_OUR_COIN: &str = "tosses of our coin:";
const FAIR_OUT: &str = "fair tosses out of these";
const THE_TRICK: &str = "the trick, on average:";
const PER_WORD: &str = "per";
const MODEL: &str = "(model)";
const GUESS: [&str; 2] = ["guess heads: right", "times in"];
const HOLDS_LESS: &str = "so each toss holds less than one fair toss";
const THE_MOST: &str = "the most any rule can average:";
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
const WHICH_MATCHED: &str = "which pairs matched";
const WHAT_SHOWED: &str = "what the matching pairs showed";
const COUNTED_IN: &str = "counted in";
const TOSSES: &str = "tosses";
const THE_PRICE: &str = "the price: memory and waiting";

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
/// The bars: their width, the step between them, and where each panel's
/// first bar stands (the right panel's first is the trick's own bar).
const BAR_W: f64 = 0.46;
const BAR_STEP: f64 = 0.93;
const ELIAS_X: f64 = -6.1;
const PERES_X: f64 = 0.5;

/// The triangle of 10_4: its middle, its first row's height and width, and
/// the step down and out from row to row.
const TRIANGLE_AT: Vec3 = [-3.9, 3.3, 0.0];
const ROW_DOWN: f64 = 0.36;
const ROW_WIDTH: (f64, f64) = (0.5, 0.62);
/// The small square of 10_4 and the leftovers that drop out of it: the
/// square's corner and side (a whole stream of tosses is one side long),
/// then where each generation of leftovers starts and how tall its bars are.
const SMALL_AT: Vec3 = [0.9, 1.45, 0.0];
const SMALL_SIDE: f64 = 1.6;
const LEFTOVER_X: [f64; 3] = [3.0, 4.15, 4.85];
const LEFTOVER_H: [f64; 3] = [0.26, 0.12, 0.06];

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

/// One row of Pascal's triangle as a strip of cells, one cell for each
/// group, brighter the bigger the group. Detached, one group.
fn triangle_row(stage: &mut Stage<'_>, n: u32, middle: Vec3, (w, h): (f64, f64)) -> fmn::Result<Mob> {
    let cell = w / f64::from(n + 1);
    let biggest = choose(u64::from(n), u64::from(n / 2)) as f64;
    let mut cells = Vec::new();
    for k in 0..=n {
        let size = choose(u64::from(n), u64::from(k)) as f64 / biggest;
        let left = middle[0] - w / 2.0 + cell * (f64::from(k) + 0.07);
        cells.push(rect(
            v(left, middle[1] - h / 2.0),
            cell * 0.86,
            h,
            flat(Palette::ink(), 0.16 + 0.84 * size.powf(0.3)),
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
        // the paragraph's price is the model's for a fair coin, and the note says it in tosses
        let fair_rate = Q::HALF * Q::HALF.other();
        if fair_rate.percent(0) != cited::EFFICIENCY_PERCENT.to_string() || Q::ONE / fair_rate != Q::int(EVERY.1) {
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
        let note_y = FAIR_AT[1] + FAIR_SIDE / 2.0;
        let note = [
            kit.text_left(stage, A_FAIR_COIN, 30.0, dim, v(NOTE_X, note_y + 0.3))?,
            kit.text_left(
                stage,
                &format!("{ONE_FAIR_TOSS} {}", EVERY.0),
                30.0,
                ink,
                v(NOTE_X, note_y - 0.3),
            )?,
        ];
        fade_in_all(stage, &note, 0.6)?;

        // 10_2: our coin gives less: a hundred tosses, and the fair ones that drop out
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
        fade_in_lagged(stage, &strip, 0.9, 0.02)?;
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
        line.wait_for(stage, "A hundred tosses")?;
        fade_in_all(stage, &[strip_note], 0.4)?;
        // a matching pair is thrown away; a pair that differs gives its first toss
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
        let fair_note = kit.text_left(
            stage,
            &format!("{} {FAIR_OUT} {PER}", kept.len()),
            22.0,
            dim,
            v(strip_left, FAIR_Y - 0.42),
        )?;
        // on the scale: what the trick gives on average
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
            fade_in(stage.arena_mut(), fair_note, ORIGIN, 1.0)?.rt(0.5),
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

        // 10_3: no; a toss you can mostly guess holds less than a fair one, and there is a ceiling
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
        let ruled = polyline(vec![v(SCALE_X - 0.5, ceiling_y), v(6.6, ceiling_y)])
            .style(line_style(ink, 3.5))
            .build();
        let ruled = stage.arena_mut().add(ruled);
        play!(stage; show_creation(ruled).rt(0.8));
        let ceiling_note = kit.text(
            stage,
            &format!("{THE_MOST} {ceiling:.1} {PER_WORD} {PER} {MODEL}"),
            22.0,
            ink,
            ORIGIN,
        )?;
        stage.move_to(ceiling_note, v(6.6, ceiling_y + 0.2), RIGHT);
        fade_in_all(stage, &[ceiling_note], 0.5)?;
        line.wait_for(stage, "That ceiling is called")?;
        let entropy_note = format!(
            "{} {PER}{} {ceiling:.1} {}",
            CEILING_NOTE[0], CEILING_NOTE[1], CEILING_NOTE[2]
        );
        let entropy_note = lower_third(&kit, stage, PRECISELY, &entropy_note)?;
        play!(stage; fade_in(stage.arena_mut(), entropy_note, ORIGIN, 1.0)?.rt(0.5));

        // 10_4: two ways closer, on two panels that share the scale
        let line = say(&kit, stage, "10_4")?;
        let mut gone = strip;
        gone.extend(drops.iter().map(|d| d.0));
        gone.extend([strip_note, fair_note, trick_note]);
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
            let width = ROW_WIDTH.0 + ROW_WIDTH.1 * k as f64;
            let row_at = v(TRIANGLE_AT[0], TRIANGLE_AT[1] - ROW_DOWN * k as f64);
            let row = triangle_row(stage, n, row_at, (width, 0.24))?;
            let row_name = kit.text(stage, &n.to_string(), 20.0, dim, ORIGIN)?;
            stage.move_to(row_name, v(row_at[0] - width / 2.0 - 0.14, row_at[1]), RIGHT);
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
                fade_in(stage.arena_mut(), row, v(0.0, -0.15), 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), row_name, ORIGIN, 1.0)?.rt(0.5),
                fmn::animation::grow_from_edge(stage.arena_mut(), column, DOWN, None)?.rt(0.5),
                fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), number, ORIGIN, 1.0)?.rt(0.5),
            );
        }
        kit.lint_here(stage, "10_4 the left panel");

        // the right panel: the trick again, on the two things it ignored
        line.wait_for(stage, "or run the trick again")?;
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
        fade_in_all(stage, &small.mobs(), 0.4)?;
        let pose = small.pose(stage);
        let whole = Leftover {
            len: 1.0,
            heads: p.f64(),
        };
        let two = whole.leaves();
        let decides = law.decides().f64();
        // which pairs matched: the grey part against the coloured part;
        // what the matching pairs showed: the two grey blocks
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
        let leftover_y = [SMALL_AT[1] + SMALL_SIDE - 0.2, SMALL_AT[1] + 0.6];
        let square_right = SMALL_AT[0] + SMALL_SIDE;
        for i in 0..2 {
            let y = leftover_y[i];
            let coin = stretches(
                stage,
                v(LEFTOVER_X[0], y),
                (SMALL_SIDE * two[i].len, LEFTOVER_H[0]),
                &split[i],
            )?;
            let word = kit.text_left(stage, words[i], 20.0, ink, v(LEFTOVER_X[0], y - 0.33))?;
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
            .map(|i| (two[i], v(LEFTOVER_X[0] + SMALL_SIDE * two[i].len, leftover_y[i])))
            .collect();
        for generation in 1..LEFTOVER_X.len() {
            let spread = [0.1, 0.045][generation - 1];
            let mut next = Vec::new();
            let mut grown = Vec::new();
            for &(parent, end) in &parents {
                for (child, dy) in parent.leaves().into_iter().zip([spread, -spread]) {
                    let left = v(LEFTOVER_X[generation], end[1] + dy);
                    let w = SMALL_SIDE * child.len;
                    grown.push(stretches(
                        stage,
                        left,
                        (w, LEFTOVER_H[generation]),
                        &[(1.0, Palette::dim())],
                    )?);
                    grown.push(link(stage, v(end[0] + 0.04, end[1]), v(left[0] - 0.04, left[1])));
                    next.push((child, v(left[0] + w, left[1])));
                }
            }
            fade_in_all(stage, &grown, 0.35)?;
            parents = next;
        }
        // the bars for more rounds
        let mut grows: Vec<Box<dyn Animation>> = Vec::new();
        let mut labels = Vec::new();
        for (k, &rounds) in PERES_DEPTHS.iter().enumerate().skip(1) {
            let rate = per * peres_rate(rounds, p.f64());
            let column = bar(stage, peres_x(k), rate);
            let a = fmn::animation::grow_from_edge(stage.arena_mut(), column, DOWN, None)?;
            grows.push(stage.prepare(a)?);
            labels.push(kit.text(stage, &rounds.to_string(), 20.0, dim, v(peres_x(k), floor_y - 0.25))?);
            labels.push(kit.text(
                stage,
                &format!("{rate:.0}"),
                20.0,
                ink,
                v(peres_x(k), number_over(rate)),
            )?);
        }
        let grows = AnimationGroup::with_lag_ratio(stage.arena_mut(), grows, 0.3)?.with_run_time(1.2);
        stage.play(grows)?;
        fade_in_all(stage, &labels, 0.3)?;
        kit.lint_here(stage, "10_4 the right panel");

        // both climb toward the ceiling: what 100,000 seeded tosses gave, on the bars
        line.wait_for(stage, "Both climb toward")?;
        let place = |list: &[u32], want: u32, what: &str| {
            list.iter()
                .position(|&x| x == want)
                .ok_or_else(|| fail(format!("no bar for {what} {want}")))
        };
        let counted = [
            (von_neumann(&run), peres_x(place(&PERES_DEPTHS, 1, "round")?)),
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
        dots.push(kit.text(stage, ALL, 20.0, dim, v(peres_x(PERES_DEPTHS.len()), floor_y - 0.25))?);
        fade_in_lagged(stage, &dots, 0.9, 0.1)?;
        let names = format!(
            "{} {}{} {}",
            ELIAS_PERES[0],
            cited::ELIAS_YEAR,
            ELIAS_PERES[1],
            cited::PERES_YEAR
        );
        let names = lower_third(&kit, stage, NAMES, &names)?;
        play!(stage;
            fade_out(stage.arena_mut(), entropy_note, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.5),
        );
        // neither ever reaches it
        line.wait_for(stage, "and neither ever")?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), ceiling_note, 1.1, Some(rgb(ink)))?.rt(0.8));
        wait_until(stage, line.end() - 0.6)?;
        let the_price = kit.text(stage, THE_PRICE, 26.0, ink, v(0.0, 3.55))?;
        fade_in_all(stage, &[the_price], 0.5)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
