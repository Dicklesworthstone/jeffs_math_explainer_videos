//! Chapter 03, why they cancel: from the pair A and B the opening question
//! asks about to one pulse paired with itself (03_1),
//! doubled (03_2) and doubled again into patterns A and B (03_3); at one
//! step every lined-up pair sits inside a half or reaches across the middle
//! (03_4): inside, B's products are A's (03_5); across, they flip sign
//! (03_6); added (03_7, a prediction), the crossing ones cancel in pairs and
//! the rest is twice the old pair's rows (03_8); the same at every shift and
//! every doubling, the identity on screen (03_9); without the flip the
//! crossing products add and the ghosts come back (03_10).
//!
//! Every pattern is `model::double`'s, from one pulse, except the opening's
//! A and B, the model's codes, which the facts check are the second
//! doubling; every product is
//! `facts::split` of the halves on screen; the column is checked against the
//! model as it settles. A pair across the middle is drawn in the across
//! role: a teal arc, and a teal ring on its block.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CHART_Y, Chart, LABEL_X, NAME_X, PULSE_H, Place, Row, STEP, Spoken, TICK_Y, X0, arc, block, block_across,
    fade_in_all, fade_out_all, fail, flash_all, line_style, pause, row, say, score_text, shift_all, swap, think,
    wait_until, x_of,
};
use crate::facts::{self, SHIFT_03};
use crate::model::{self, Scores, Span};
use crate::palette;

pub struct WhyTheyCancel {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIRST: &str = "first";
const OTHER: &str = "other";
const PATTERN_A: &str = "pattern A";
const PATTERN_B: &str = "pattern B";
const SCORE: &str = "score";
const SHIFT: &str = "shift";
const OLD_PAIR: &str = "the old pair, one step:";
const X: &str = "X";
const Y: &str = "Y";
const MINUS_Y: &str = "\u{2212}Y";

/// The first pattern's line and the other's.
const TOP_Y: f64 = 2.3;
const BOT_Y: f64 = 0.6;
/// How far under its row a sliding copy runs (03_4).
const COPY_DROP: f64 = 0.85;
/// An arc's rise over a one-step pair; longer pairs rise a little more.
const ARC_RISE: f64 = 0.3;
/// Where a crossing pair's two blocks meet when they cancel (03_9).
const MEET_Y: f64 = (TOP_Y + BOT_Y) / 2.0;
/// The countdown, right of the rows.
const THINK_AT: [f64; 2] = [4.8, 1.5];
/// The old pair's numbers (03_8), right of the column.
const NUMBERS_AT: [f64; 2] = [2.2, -1.15];

/// One shift's lined-up pairs drawn over a row: an arc and a product block
/// per pair, with where the pair sits.
struct Pairs {
    arcs: Vec<Mob>,
    blocks: Vec<(Mob, i64, Span)>,
}

impl Pairs {
    fn mobs(&self) -> Vec<Mob> {
        let mut all = self.arcs.clone();
        all.extend(self.blocks.iter().map(|&(m, _, _)| m));
        all
    }

    fn blocks(&self, want: Span) -> Vec<Mob> {
        self.blocks
            .iter()
            .filter(|&&(_, _, s)| s == want)
            .map(|&(m, _, _)| m)
            .collect()
    }

    fn arcs(&self, want: Span) -> Vec<Mob> {
        self.blocks
            .iter()
            .zip(&self.arcs)
            .filter(|&(&(_, _, s), _)| s == want)
            .map(|(_, &a)| a)
            .collect()
    }

    /// The blocks that sit `want`, with their products, for a chart.
    fn arriving(&self, want: Span) -> Vec<(Mob, i64)> {
        self.blocks
            .iter()
            .filter(|&&(_, _, s)| s == want)
            .map(|&(m, value, _)| (m, value))
            .collect()
    }
}

/// The pairs of the pattern `x then y` that line up at shift `k` (k > 0),
/// drawn over its row on the line `line_y`. Detached.
fn draw_pairs(stage: &mut Stage<'_>, x: &[i64], y: &[i64], k: i64, line_y: f64) -> fmn::Result<Pairs> {
    let base = line_y + PULSE_H + 0.06;
    let rise = ARC_RISE + 0.1 * (k - 1) as f64;
    let mut pairs = Pairs {
        arcs: Vec::new(),
        blocks: Vec::new(),
    };
    for (p, s) in facts::split(x, y, k) {
        let (left, right) = (x_of(p.copy as i64), x_of(p.recording as i64));
        let color = match s {
            Span::Inside => Palette::dim(),
            Span::Across => palette::across(),
        };
        pairs
            .arcs
            .push(stage.arena_mut().add(arc(v(left, base), v(right, base), rise, color)));
        let at = v((left + right) / 2.0, base + rise + 0.17);
        let b = match s {
            Span::Inside => block(stage, p.value, at, 1.0)?,
            Span::Across => block_across(stage, p.value, at)?,
        };
        pairs.blocks.push((b, p.value, s));
    }
    Ok(pairs)
}

/// The pair `pair` drawn as two rows from X0: the first on top, the other
/// below. Detached.
fn pair_rows(stage: &mut Stage<'_>, pair: &(Vec<i64>, Vec<i64>)) -> fmn::Result<(Row, Row)> {
    Ok((
        row(stage, &pair.0, X0, TOP_Y, palette::pattern_a())?,
        row(stage, &pair.1, X0, BOT_Y, palette::pattern_b())?,
    ))
}

/// Double the pair drawn as `rows`: a copy of the other rises to the right
/// of the first; the other slides right and a copy of the first drops in on
/// its left; the other flips. With `line`, each move waits for its words.
/// Ends on clean rows of `next`, the model's doubling of `pair`.
fn double_rows(
    stage: &mut Stage<'_>,
    line: Option<&Spoken>,
    rows: (Row, Row),
    pair: &(Vec<i64>, Vec<i64>),
    next: &(Vec<i64>, Vec<i64>),
    rt: f64,
) -> fmn::Result<(Row, Row)> {
    let (top, bottom) = rows;
    let n = pair.0.len() as f64;
    if let Some(l) = line {
        l.wait_for(stage, "then the other")?;
    }
    let up = row(stage, &pair.1, X0, BOT_Y, palette::pattern_b())?;
    fade_in_all(stage, &up.mobs(), 0.15)?;
    shift_all(stage, &up.mobs(), n * STEP, TOP_Y - BOT_Y, rt)?;
    if let Some(l) = line {
        l.wait_for(stage, "Then the first again")?;
    }
    shift_all(stage, &bottom.mobs(), n * STEP, 0.0, rt)?;
    let down = row(stage, &pair.0, X0, TOP_Y, palette::pattern_a())?;
    fade_in_all(stage, &down.mobs(), 0.15)?;
    shift_all(stage, &down.mobs(), 0.0, BOT_Y - TOP_Y, rt)?;
    if let Some(l) = line {
        l.wait_for(stage, "the other flipped")?;
    }
    let negated: Vec<i64> = pair.1.iter().map(|v| -v).collect();
    let flipped = row(stage, &negated, X0 + n * STEP, BOT_Y, palette::pattern_b())?;
    swap(stage, &bottom.mobs(), &flipped.mobs(), rt)?;
    let (new_top, new_bottom) = pair_rows(stage, next)?;
    let mut old = top.mobs();
    old.extend(up.mobs());
    old.extend(down.mobs());
    old.extend(flipped.mobs());
    let mut new = new_top.mobs();
    new.extend(new_bottom.mobs());
    swap(stage, &old, &new, 0.3)?;
    Ok((new_top, new_bottom))
}

/// A thin tick through a row at the middle of a pattern `len` pulses long.
fn middle_tick(stage: &mut Stage<'_>, len: usize, line_y: f64) -> Mob {
    let x = X0 + (len as f64 / 2.0 - 0.5) * STEP;
    let tick = polyline(vec![v(x, line_y - 0.5), v(x, line_y + 0.5)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    stage.arena_mut().add(tick)
}

/// The old pair's numbers at one step (03_8): factor(x + y) = total.
fn numbers_tex() -> fmn::Result<String> {
    let (factor, [x, y], total) = facts::doubling_numbers().map_err(fail)?;
    let y = if y < 0 { format!("({y})") } else { y.to_string() };
    Ok(format!("{factor}({x} + {y}) = {total}"))
}

/// The identity (03_9), its factor from the facts.
fn identity_tex() -> fmn::Result<String> {
    let (factor, _, _) = facts::doubling_numbers().map_err(fail)?;
    Ok(format!(r"S_{{X,Y}}(k) + S_{{X,-Y}}(k) = {factor}(S_X(k) + S_Y(k))"))
}

/// The exact scope of 03_9: the identity's reach and a pair it misses.
fn reach_claim() -> fmn::Result<String> {
    let lengths: Vec<String> = facts::doubling_lengths()
        .map_err(fail)?
        .iter()
        .take(4)
        .map(i64::to_string)
        .collect();
    let (a, b) = model::GOLAY_10;
    if !model::is_complementary(&a, &b) {
        return Err(fail("the length-10 pair does not cancel".into()));
    }
    Ok(format!(
        "for any halves X and Y of the same length, at every shift. Doubling from one pulse gives lengths {}, and on; but a pair of length {} cancels too",
        lengths.join(", "),
        a.len()
    ))
}

/// The assumptions every chapter keeps (03_10).
const STILL_SCENE: &str = "as before, the guarantee needs the same still scene for both sends, and receiver noise is scored too and is not guaranteed to cancel";

impl SceneConstruct for WhyTheyCancel {
    fn name(&self) -> &str {
        "03_why_they_cancel"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [seed, two, four] = facts::doublings();
        let name_at = |y: f64| v(NAME_X, y + 0.35);

        // 03_1: the pair the question asks about, patterns A and B as the
        // earlier chapters drew them, in before the line begins (the fade is
        // shorter than the lead-in, so the line keeps its start); then one
        // pulse, paired with itself
        let known = (model::CODE_A.to_vec(), model::CODE_B.to_vec());
        let asked_rows = pair_rows(stage, &known)?;
        let known_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(TOP_Y))?;
        let known_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(BOT_Y))?;
        let mut asked = asked_rows.0.mobs();
        asked.extend(asked_rows.1.mobs());
        asked.extend([known_a, known_b]);
        fade_in_all(stage, &asked, 0.5)?;
        let line = say(&kit, stage, "03_1")?;
        let rows = pair_rows(stage, &seed)?;
        let first = kit.text_left(stage, FIRST, 22.0, Palette::dim(), name_at(TOP_Y))?;
        let other = kit.text_left(stage, OTHER, 22.0, Palette::dim(), name_at(BOT_Y))?;
        line.wait_for(stage, "Start with one pulse")?;
        let mut all = rows.0.mobs();
        all.extend(rows.1.mobs());
        all.extend([first, other]);
        swap(stage, &asked, &all, 0.5)?;

        // 03_2: double it, move by move
        let line = say(&kit, stage, "03_2")?;
        let rows = double_rows(stage, Some(&line), rows, &seed, &two, 0.5)?;

        // 03_3: double again: patterns A and B
        let line = say(&kit, stage, "03_3")?;
        let (top, bottom) = double_rows(stage, None, rows, &two, &four, 0.35)?;
        line.wait_for(stage, "patterns A and B")?;
        let name_a = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(TOP_Y))?;
        let name_b = kit.text_left(stage, PATTERN_B, 22.0, palette::pattern_b(), name_at(BOT_Y))?;
        let ticks = [
            middle_tick(stage, four.0.len(), TOP_Y),
            middle_tick(stage, four.1.len(), BOT_Y),
        ];
        let mut new = vec![name_a, name_b];
        new.extend(ticks);
        swap(stage, &[first, other], &new, 0.4)?;

        // 03_4: each copy slides one step; the pairs that line up, as arcs
        let line = say(&kit, stage, "03_4")?;
        let (x, y) = (&two.0, &two.1);
        let flipped: Vec<i64> = y.iter().map(|v| -v).collect();
        line.wait_for(stage, "Slide each copy")?;
        let copy_a = row(stage, &four.0, X0, TOP_Y - COPY_DROP, Palette::dim())?;
        let copy_b = row(stage, &four.1, X0, BOT_Y - COPY_DROP, Palette::dim())?;
        let mut copies = copy_a.mobs();
        copies.extend(copy_b.mobs());
        fade_in_all(stage, &copies, 0.3)?;
        shift_all(stage, &copies, STEP * SHIFT_03 as f64, 0.0, 0.6)?;
        line.wait_for(stage, "the pairs that line up")?;
        let over_a = draw_pairs(stage, x, y, SHIFT_03, TOP_Y)?;
        let over_b = draw_pairs(stage, x, &flipped, SHIFT_03, BOT_Y)?;
        let mut drawn = over_a.mobs();
        drawn.extend(over_b.mobs());
        swap(stage, &copies, &drawn, 0.5)?;

        // 03_5: inside a half the products match
        let line = say(&kit, stage, "03_5")?;
        line.wait_for(stage, "both pulses flipped")?;
        let half = four.0.len() / 2;
        let mut seconds: Vec<Mob> = top.pulses[half..].to_vec();
        seconds.extend(&bottom.pulses[half..]);
        flash_all(stage, &seconds, 0.6)?;
        line.wait_for(stage, "Those products")?;
        let mut inside = over_a.blocks(Span::Inside);
        inside.extend(over_b.blocks(Span::Inside));
        flash_all(stage, &inside, 0.6)?;

        // 03_6: across the middle they change sign
        let line = say(&kit, stage, "03_6")?;
        line.wait_for(stage, "exactly one flipped pulse")?;
        let mut across = over_a.arcs(Span::Across);
        across.extend(over_b.arcs(Span::Across));
        across.extend(over_a.blocks(Span::Across));
        across.extend(over_b.blocks(Span::Across));
        flash_all(stage, &across, 0.7)?;

        // 03_7: the score line waits below; what is left?
        let line = say(&kit, stage, "03_7")?;
        let mut chart = Chart::new(stage, Place::main());
        let mut frame = vec![chart.line];
        for k in -(four.0.len() as i64 - 1)..four.0.len() as i64 {
            frame.push(kit.text(stage, &score_text(k), 20.0, Palette::dim(), v(x_of(k), TICK_Y))?);
        }
        frame.push(kit.text(stage, SCORE, 22.0, Palette::dim(), v(LABEL_X, CHART_Y))?);
        frame.push(kit.text(stage, SHIFT, 22.0, Palette::dim(), v(LABEL_X, TICK_Y))?);
        fade_in_all(stage, &frame, 0.4)?;
        line.wait_for(stage, "products here")?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 03_8: the crossing products cancel in pairs; the rest is twice the
        // old pair's rows
        let line = say(&kit, stage, "03_8")?;
        let mut arcs = over_a.arcs.clone();
        arcs.extend(&over_b.arcs);
        arcs.push(q);
        fade_out_all(stage, &arcs, 0.3)?;
        let mut crossing = over_a.arriving(Span::Across);
        crossing.extend(over_b.arriving(Span::Across));
        chart.settle_all(stage, &[(SHIFT_03, crossing)], 0.6, 0.3)?;
        line.wait_for(stage, "The rest")?;
        let mut rest = over_a.arriving(Span::Inside);
        rest.extend(over_b.arriving(Span::Inside));
        chart.settle_all(stage, &[(SHIFT_03, rest)], 0.6, 0.3)?;
        let at_shift = Scores {
            first: SHIFT_03,
            values: vec![facts::sum().at(SHIFT_03)],
        };
        chart.check(&at_shift, "A's and B's products at one step")?;
        let zero = kit.text(
            stage,
            &score_text(chart.height(SHIFT_03)),
            26.0,
            Palette::ink(),
            chart.place.past(SHIFT_03, chart.height(SHIFT_03), 0.25),
        )?;
        let old = kit.text(
            stage,
            OLD_PAIR,
            22.0,
            Palette::dim(),
            v(NUMBERS_AT[0], NUMBERS_AT[1] + 0.5),
        )?;
        let numbers = kit.tex(
            stage,
            &numbers_tex()?,
            36.0,
            Palette::ink(),
            &[],
            v(NUMBERS_AT[0], NUMBERS_AT[1]),
        )?;
        fade_in_all(stage, &[zero, old, numbers], 0.4)?;

        // 03_9: every shift, every doubling: the identity
        let line = say(&kit, stage, "03_9")?;
        fade_out_all(stage, &[old, numbers, zero], 0.3)?;
        for k in [SHIFT_03 + 1, SHIFT_03 + 2] {
            let over_a = draw_pairs(stage, x, y, k, TOP_Y)?;
            let over_b = draw_pairs(stage, x, &flipped, k, BOT_Y)?;
            let mut drawn = over_a.mobs();
            drawn.extend(over_b.mobs());
            fade_in_all(stage, &drawn, 0.3)?;
            // every pair here crosses: A's block and B's meet and vanish
            let mut meet: Vec<Box<dyn Animation>> = Vec::new();
            let mut gone = over_a.arcs.clone();
            gone.extend(&over_b.arcs);
            for (&(ma, va, _), &(mb, vb, _)) in over_a.blocks.iter().zip(&over_b.blocks) {
                if va != -vb {
                    return Err(fail(format!("at shift {k} the pair's products are {va} and {vb}")));
                }
                for m in [ma, mb] {
                    let to = v(stage.get_center(m)[0], MEET_Y);
                    let a = m
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.4),
                            ..AnimateArgs::default()
                        })?
                        .move_to(to, ORIGIN)?;
                    meet.push(stage.prepare(a)?);
                    gone.push(m);
                }
            }
            stage.play_prepared(meet)?;
            fade_out_all(stage, &gone, 0.25)?;
        }
        let labels = [
            (X, palette::pattern_a(), 0, TOP_Y),
            (Y, palette::pattern_a(), half, TOP_Y),
            (X, palette::pattern_b(), 0, BOT_Y),
            (MINUS_Y, palette::pattern_b(), half, BOT_Y),
        ];
        let mut names = Vec::new();
        for (words, color, start, line_y) in labels {
            let middle = X0 + (start as f64 + (half as f64 - 1.0) / 2.0) * STEP;
            names.push(kit.text(stage, words, 24.0, color, v(middle, line_y + 0.62))?);
        }
        fade_in_all(stage, &names, 0.3)?;
        line.wait_for(stage, "for every doubling")?;
        let identity = kit.tex(stage, &identity_tex()?, 40.0, Palette::ink(), &[], v(0.3, 3.45))?;
        fade_in_all(stage, &[identity], 0.5)?;
        line.wait_for(stage, "twice as long")?;
        let reach = precisely(&kit, stage, &reach_claim()?)?;
        fade_in_all(stage, &[reach], 0.4)?;

        // 03_10: leave out the flip: the crossing products add
        let line = say(&kit, stage, "03_10")?;
        let still = precisely(&kit, stage, STILL_SCENE)?;
        swap(stage, &[reach], &[still], 0.3)?;
        let unflipped = row(stage, &four.0, X0, BOT_Y, palette::pattern_a())?;
        let name_again = kit.text_left(stage, PATTERN_A, 22.0, palette::pattern_a(), name_at(BOT_Y))?;
        // the identity and the X, Y labels belong to the flipped pair: they
        // leave before the unflipped pair's arcs are drawn, so the identity
        // is never seen over the wrong pair (the 2026-10-18 draft's 03_10)
        let mut old = bottom.mobs();
        old.extend([name_b, identity]);
        old.extend(&names);
        let mut new = unflipped.mobs();
        new.extend([name_again]);
        swap(stage, &old, &new, 0.4)?;
        let far = 2 * x.len() as i64 - 1;
        let far_a = draw_pairs(stage, x, y, far, TOP_Y)?;
        let far_again = draw_pairs(stage, x, y, far, BOT_Y)?;
        let mut drawn = far_a.mobs();
        drawn.extend(far_again.mobs());
        fade_in_all(stage, &drawn, 0.3)?;
        line.wait_for(stage, "add instead")?;
        let mut adding = far_a.arriving(Span::Across);
        adding.extend(far_again.arriving(Span::Across));
        chart.settle_all(stage, &[(far, adding)], 0.6, 0.3)?;
        line.wait_for(stage, "The ghosts come back")?;
        let mut leaving = top.mobs();
        leaving.extend(unflipped.mobs());
        leaving.extend(far_a.arcs.iter().chain(&far_again.arcs));
        leaving.extend([name_a, name_again, ticks[0], ticks[1]]);
        fade_out_all(stage, &leaving, 0.3)?;
        let no_flip = model::autocorrelation(&four.0).scaled(2);
        let rest = no_flip.plus(&Scores {
            first: far,
            values: vec![-no_flip.at(far)],
        });
        chart.stand(stage, &rest, 0.4)?;
        chart.check(&no_flip, "pattern A paired with itself")?;
        let mut ghosts = Vec::new();
        for k in -far..=far {
            if k != 0 {
                ghosts.extend(chart.column(k));
            }
        }
        flash_all(stage, &ghosts, 0.5)?;
        let end = line.start + line.len;
        wait_until(stage, end)?;
        pause(stage, 0.5)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words and formulas on screen come from the facts.
    #[test]
    fn the_screen_text_is_built_from_the_facts() {
        assert_eq!(numbers_tex().unwrap(), "2(1 + (-1)) = 0");
        assert_eq!(
            identity_tex().unwrap(),
            r"S_{X,Y}(k) + S_{X,-Y}(k) = 2(S_X(k) + S_Y(k))"
        );
        let reach = reach_claim().unwrap();
        assert!(reach.contains("lengths 1, 2, 4, 8, and on"), "{reach}");
        assert!(reach.contains("a pair of length 10 cancels too"), "{reach}");
        assert!(STILL_SCENE.contains("same still scene for both sends"));
        assert!(STILL_SCENE.contains("not guaranteed to cancel"));
    }

    /// The pair shown during the opening question is the pair the doublings
    /// arrive at in 03_3.
    #[test]
    fn the_opening_pair_is_the_doubled_pair() {
        let [_, _, four] = facts::doublings();
        assert_eq!(four, (model::CODE_A.to_vec(), model::CODE_B.to_vec()));
    }

    /// Both "precisely" cards wrap to two lines, clear of the shift numbers.
    #[test]
    fn the_precisely_cards_are_two_lines() {
        for text in [reach_claim().unwrap(), STILL_SCENE.to_owned()] {
            let lines = explainer_kit::animatic::wrap(&text, 92, 3);
            assert!(lines.len() <= 2, "{} lines: {text}", lines.len());
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "Start with one pulse"),
            ("03_2", "then the other"),
            ("03_2", "Then the first again"),
            ("03_2", "the other flipped"),
            ("03_3", "patterns A and B"),
            ("03_4", "Slide each copy"),
            ("03_4", "the pairs that line up"),
            ("03_5", "both pulses flipped"),
            ("03_5", "Those products"),
            ("03_6", "exactly one flipped pulse"),
            ("03_7", "products here"),
            ("03_8", "The rest"),
            ("03_9", "for every doubling"),
            ("03_9", "twice as long"),
            ("03_10", "add instead"),
            ("03_10", "The ghosts come back"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart at the model's own sizes: the
    /// tallest arc and block clear the top, the bottom row's blocks clear the
    /// top row's pulses, and the countdown stands clear of every row.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::BLOCK;
        let [_, two, four] = facts::doublings();
        let far = 2 * two.0.len() as i64 - 1;
        let block_top = |line_y: f64, k: i64| line_y + PULSE_H + 0.06 + ARC_RISE + 0.1 * (k - 1) as f64 + 0.17 + 0.12;
        assert!(block_top(TOP_Y, far) < 3.85, "the far arc's block inside the safe area");
        assert!(
            block_top(BOT_Y, far) < TOP_Y - PULSE_H,
            "the bottom row's blocks under the top row"
        );
        const {
            assert!(
                TOP_Y - COPY_DROP - PULSE_H > BOT_Y + PULSE_H,
                "the top copy above the bottom row"
            )
        };
        let row_end = X0 + (four.0.len() as f64 - 1.0) * STEP + 0.35;
        assert!(THINK_AT[0] - 0.62 > row_end, "the countdown clear of the rows");
        // the no-flip row, standing after the rows clear, stays above the
        // shift numbers and under where the top row was
        let no_flip = model::autocorrelation(&four.0).scaled(2);
        let low = (no_flip.first..=no_flip.last())
            .map(|k| no_flip.at(k))
            .min()
            .unwrap_or(0);
        assert!(CHART_Y + low as f64 * BLOCK > TICK_Y + 0.15);
    }
}
