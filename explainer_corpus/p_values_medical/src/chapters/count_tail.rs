//! Chapter 04, count the whole tail: the study found fifteen, and which way
//! is better news was decided in advance (04_1); which bars are at least as
//! favourable (04_2, a prediction); fifteen and above (04_3); counted in
//! ways, not bars: one way to get twenty, shown as the actual assignment
//! (04_4), twenty ways to get nineteen, three of them shown (04_5), and the
//! rest of the ledger to a total of 21,700 (04_6); out of 1,048,576, about
//! two percent, and only then its name, the one-sided p-value (04_7); the
//! exact and the strict tails as other subsets and other numbers (04_8);
//! what was fixed in advance, the statistic and its direction, and what the
//! data set, the boundary at this result (04_9); and the world every one of
//! these ways lives in (04_10).
//!
//! Every lit or outlined set is `model::tail_members` of the tail said;
//! every count is the listed world's (`Study::null_world`).

use explainer_kit::props::{histogram, precisely};
use explainer_kit::{Frame2, Kit, Palette, Timed, stroke_style, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{fade_in_all, fade_in_lagged, fade_out_all, line_style, say, think_pause, treated, warn};
use super::five_percent::outline;
use super::imaginary_world::{Closing, SHOWN_Y, SMALL_AT, closing_frame, disc_at, lit_box, place_small, small};
use super::same_test::{pair_at, small_fraction};
use crate::model::{Study, Tail, study, tail_count, tail_members, tail_share};

pub struct CountTheWholeTail {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const OR_MORE: &str = "or more";
const ONE_WAY: &str = "these pairs: the one way to get";
const ONE_OF: &str = "these pairs: one of the";
const WAYS_TO_GET: &str = "ways to get";
const FAVOURING: &str = "favouring";
const OF: &str = "of";
const P_NAME: &str = "the one-sided p-value";
const PRECISE_A: &str = "one-sided p-value: under the sharp null,";
const PRECISE_B: &str = "of fair-coin assignments give";
const PRECISE_C: &str = "or more favouring. Not P(no effect).";
const EXACTLY: &str = "exactly";
const MORE_THAN: &str = "more than";
/// 04_9's two lines: what was chosen before the data, and what the data set
/// (the boundary is the observed count, never a cutoff chosen in advance).
const FIXED: &str = "fixed in advance: the statistic and its direction";
const SET_BY: &str = "set by the data: the tail starts at this result,";
const DENOMINATOR: &str = "every one of them in the world where the treatment changed no one's recovery";

/// The pairs 04_5 turns over, one at a time: three of the twenty single-pair
/// assignments that give nineteen.
const FLIP_PAIRS: [u32; 3] = [2, 9, 16];

// --- layout (scene units; the frame is 14.2 by 8)

/// The prediction's ring, under the small pairs.
const ASK_AT: Vec3 = [-3.9, -2.0, 0.0];
const ASK_R: f64 = 0.5;
/// The bracket over the tail: how far above bar 15, its ticks, its words.
const BRACKET_DY: f64 = 0.5;
const BRACKET_TICK: f64 = 0.1;
const BRACKET_WORDS_DY: f64 = 0.22;
/// The ledger: words left-aligned here, counts right-aligned there, one row
/// per count from twenty down, then a rule and the total.
const ROW_X: f64 = -6.6;
const NUM_X: f64 = -2.3;
const TOP_ROW_Y: f64 = -0.9;
const ROW_DY: f64 = 0.33;
const RULE_DY: f64 = 0.21;
const TOTAL_DY: f64 = 0.45;
/// How far the ledger rises when the small pairs leave (04_7).
const LIFT: f64 = 2.95;
/// The p-value's name and fraction (after the lift), and the lines under it:
/// the wrong tails (04_8), then what was fixed and what was set (04_9).
const P_NAME_Y: f64 = -0.6;
const P_Y: f64 = -1.45;
const WRONG_Y: [f64; 2] = [-2.35, -2.75];
const CARD_Y: [f64; 2] = [-2.3, -2.65];
/// The outline around the histogram's two-line title (04_10), half its
/// width, and the closing caption.
const TITLE_HALF_W: f64 = 2.9;
const TITLE_TOP: f64 = 2.45;
const TITLE_BOTTOM: f64 = 1.75;
const CAPTION_Y: f64 = -3.3;

/// The ledger row's height for the count `j` of a world of `pairs` pairs.
fn row_y(j: u32, pairs: u32) -> f64 {
    TOP_ROW_Y - ROW_DY * f64::from(pairs - j)
}

/// From the left edge of bar `first` to the right edge of bar `last`.
fn bar_span(frame: &Frame2, first: u32, last: u32) -> (f64, f64) {
    (
        frame.p(f64::from(first) - 0.45, 0.0)[0],
        frame.p(f64::from(last) + 0.45, 0.0)[0],
    )
}

/// The lower-third's exact version of the one-sided p-value, two lines
/// (three would cover the histogram's axis label).
fn precise_text(world: &[u64], k: u32) -> String {
    let total: u64 = world.iter().sum();
    format!(
        "{PRECISE_A} {} / {} ≈ {} {PRECISE_B} {k} {PRECISE_C}",
        thousands(tail_count(world, k, Tail::AtLeast)),
        thousands(total),
        tail_share(world, k, Tail::AtLeast).decimal(4)
    )
}

/// One ledger row, "17 favouring" and its count right-aligned, detached.
fn ledger_row(kit: &Kit, stage: &mut Stage<'_>, j: u32, n: u64, pairs: u32) -> fmn::Result<[Mob; 2]> {
    let y = row_y(j, pairs);
    let words = kit.text_left(stage, &format!("{j} {FAVOURING}"), 22.0, Palette::ink(), v(ROW_X, y))?;
    let num = kit.text(stage, &thousands(n), 22.0, treated(), v(NUM_X, y))?;
    stage.move_to(num, v(NUM_X, y), RIGHT);
    Ok([words, num])
}

/// The move of `m` so that its `edge` lands at `to`, prepared.
fn move_edge(stage: &mut Stage<'_>, m: Mob, to: Vec3, edge: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, edge)?;
    stage.prepare(a)
}

/// A red outline around the bars of `members` (consecutive counts), from
/// the axis to just over the tallest of them.
fn around_bars(stage: &mut Stage<'_>, frame: &Frame2, world: &[u64], members: &[u32]) -> fmn::Result<Mob> {
    let (first, last) = (members[0], members[members.len() - 1]);
    let top = members
        .iter()
        .map(|&j| frame.p(f64::from(j), world[j as usize] as f64)[1])
        .fold(frame.bottom(), f64::max)
        + 0.05;
    let bottom = frame.bottom() - 0.04;
    let (l, r) = bar_span(frame, first, last);
    let o = Rectangle::new()
        .width(r - l)
        .height(top - bottom)
        .style(stroke_style(warn(), 3.0))
        .build()?
        .moved_to(v((l + r) / 2.0, (top + bottom) / 2.0));
    Ok(stage.arena_mut().add(o))
}

/// The small pairs take the assignment `next`: each ring whose coin differs
/// jumps to its pair's other patient, the lit boxes follow the favouring
/// pairs; the finishing order never moves.
fn set_small(stage: &mut Stage<'_>, s: &Study, c: &mut Closing, next: u32, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for i in 0..s.pairs {
        let p = i as usize;
        if s.treated_is_a(next, i) != s.treated_is_a(c.assignment, i) {
            let to = small(disc_at(p, s.treated_is_a(next, i)));
            anims.push(move_edge(stage, c.rings[p], to, ORIGIN, rt)?);
        }
        match (c.lit[p], s.favours(next, i)) {
            (Some(m), false) => {
                let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(anim.rt(rt))?);
                c.lit[p] = None;
            }
            (None, true) => {
                let m = lit_box(stage, pair_at(p))?;
                place_small(stage, m, pair_at(p));
                let anim = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(anim.rt(rt))?);
                c.lit[p] = Some(m);
            }
            _ => {}
        }
    }
    stage.play_prepared(anims)?;
    // a newly lit box lands on top: keep the rings and the bars above it
    stage.scene_mut().bring_to_front(&c.rings)?;
    stage.scene_mut().bring_to_front(&c.bars)?;
    c.assignment = next;
    Ok(())
}

/// The small pairs' label changes with what they show.
fn relabel(kit: &Kit, stage: &mut Stage<'_>, c: &mut Closing, words: &str) -> fmn::Result<()> {
    let label = kit.text(stage, words, 20.0, Palette::dim(), v(SMALL_AT[0], SHOWN_Y))?;
    let out = fade_out(stage.arena_mut(), c.shown, ORIGIN, 1.0)?;
    let into = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
    let both: Vec<Box<dyn Animation>> = vec![stage.prepare(out.rt(0.4))?, stage.prepare(into.rt(0.4))?];
    stage.play_prepared(both)?;
    c.shown = label;
    Ok(())
}

impl SceneConstruct for CountTheWholeTail {
    fn name(&self) -> &str {
        "04_count_the_whole_tail"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = study();
        let world = s.null_world();
        let total: u64 = world.iter().sum();
        let (pairs, k) = (s.pairs, s.observed());
        let lit_tail = tail_members(pairs, k, Tail::AtLeast);
        let tail_sum = tail_count(&world, k, Tail::AtLeast);
        let (first, last) = (lit_tail[0], lit_tail[lit_tail.len() - 1]);
        let mut c = closing_frame(&kit, stage)?;
        let frame = c.frame;

        // 04_1: the study found fifteen; which way is better news was decided in advance
        say(&kit, stage, "04_1")?;
        let mut tosses = c.marks.toss_dots.clone();
        tosses.push(c.marks.toss_legend);
        fade_out_all(stage, &tosses, 0.5)?;

        // 04_2: which bars hold an outcome at least as favourable as fifteen?
        say(&kit, stage, "04_2")?;
        think_pause(&kit, stage, ASK_AT, ASK_R)?;

        // 04_3: fifteen itself and everything above it, up to twenty
        say(&kit, stage, "04_3")?;
        let mut overlays = Vec::new();
        for &j in &lit_tail {
            let bar = histogram(&frame, &[(f64::from(j), world[j as usize] as f64)], 0.8, treated())?;
            overlays.push(stage.arena_mut().add(bar));
        }
        fade_in_lagged(stage, &overlays, 1.0, 0.15)?;
        let y = frame.p(f64::from(first), world[first as usize] as f64)[1] + BRACKET_DY;
        let (xl, xr) = bar_span(&frame, first, last);
        let bracket = polyline(vec![
            v(xl, y - BRACKET_TICK),
            v(xl, y),
            v(xr, y),
            v(xr, y - BRACKET_TICK),
        ])
        .style(line_style(treated(), 3.0))
        .build();
        let bracket = stage.arena_mut().add(bracket);
        let bracket_words = kit.text(
            stage,
            &format!("{first} {OR_MORE}"),
            20.0,
            treated(),
            v((xl + xr) / 2.0, y + BRACKET_WORDS_DY),
        )?;
        fade_in_all(stage, &[bracket, bracket_words], 0.4)?;

        // 04_4: count ways, not bars: all twenty favouring happens one way
        let line = say(&kit, stage, "04_4")?;
        line.wait_for(stage, "only one way")?;
        set_small(stage, &s, &mut c, s.favouring_all_but(&[]), 0.8)?;
        relabel(&kit, stage, &mut c, &format!("{ONE_WAY} {pairs}"))?;
        let row = ledger_row(&kit, stage, pairs, world[pairs as usize], pairs)?;
        fade_in_all(stage, &row, 0.4)?;
        let mut rows = vec![(pairs, row)];

        // 04_5: nineteen: any one of the twenty pairs turned over
        let line = say(&kit, stage, "04_5")?;
        let nineteen = pairs - 1;
        line.wait_for(stage, "one pair goes the other way")?;
        set_small(stage, &s, &mut c, s.favouring_all_but(&[FLIP_PAIRS[0]]), 0.6)?;
        relabel(
            &kit,
            stage,
            &mut c,
            &format!("{ONE_OF} {} {WAYS_TO_GET} {nineteen}", world[nineteen as usize]),
        )?;
        line.wait_for(stage, "any of the twenty")?;
        for &j in &FLIP_PAIRS[1..] {
            set_small(stage, &s, &mut c, s.favouring_all_but(&[j]), 0.45)?;
        }
        line.wait_for(stage, "Twenty ways")?;
        let row = ledger_row(&kit, stage, nineteen, world[nineteen as usize], pairs)?;
        fade_in_all(stage, &row, 0.4)?;
        rows.push((nineteen, row));

        // 04_6: down to fifteen the counts grow; altogether 21,700 ways
        let line = say(&kit, stage, "04_6")?;
        let mut rest = Vec::new();
        for j in (first..nineteen).rev() {
            let row = ledger_row(&kit, stage, j, world[j as usize], pairs)?;
            rest.extend(row);
            rows.push((j, row));
        }
        fade_in_lagged(stage, &rest, 1.4, 0.1)?;
        line.wait_for(stage, "Altogether")?;
        let rule_y = row_y(first, pairs) - RULE_DY;
        let rule = polyline(vec![v(ROW_X, rule_y), v(NUM_X + 0.1, rule_y)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        let rule = stage.arena_mut().add(rule);
        let total_y = row_y(first, pairs) - TOTAL_DY;
        let total_words = kit.text_left(
            stage,
            &format!("{first} {OR_MORE}"),
            24.0,
            Palette::ink(),
            v(ROW_X, total_y),
        )?;
        let of_all = kit.text_left(
            stage,
            &format!("{OF} {}", thousands(total)),
            22.0,
            Palette::dim(),
            v(NUM_X + 0.2, total_y),
        )?;
        // the total's final words, still: they take over from the live count
        // when the ledger rises, and their width right-aligns the live count
        let still_total = kit.text(stage, &thousands(tail_sum), 24.0, treated(), v(NUM_X, total_y))?;
        stage.move_to(still_total, v(NUM_X, total_y), RIGHT);
        let half = stage.get_width(still_total) / 2.0;
        fade_in_all(stage, &[rule, total_words, of_all], 0.3)?;
        let so_far: u64 = world[nineteen as usize..].iter().sum();
        let running = stage.add_value_tracker(so_far as f64);
        let live_total = kit.live_count(stage, running, 24.0, treated(), v(NUM_X - half, total_y))?;
        let count_up: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(running, tail_sum as f64, 1.4)?)?];
        stage.play_prepared(count_up)?;

        // 04_7: about two percent, and only then its name: the one-sided p-value
        let line = say(&kit, stage, "04_7")?;
        let mut leaving: Vec<Mob> = c.boxes.clone();
        leaving.extend(&c.rings);
        leaving.extend(&c.bars);
        leaving.extend(c.lit.iter().flatten());
        leaving.extend(&c.legend);
        leaving.extend([c.shown, live_total]);
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for m in leaving {
            stage.clear_updaters(m, true);
            let anim = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            moves.push(stage.prepare(anim.rt(0.5))?);
        }
        for &(j, [words, num]) in &rows {
            let y = row_y(j, pairs) + LIFT;
            moves.push(move_edge(stage, words, v(ROW_X, y), LEFT, 0.8)?);
            moves.push(move_edge(stage, num, v(NUM_X, y), RIGHT, 0.8)?);
        }
        let to = v((ROW_X + NUM_X + 0.1) / 2.0, rule_y + LIFT);
        moves.push(move_edge(stage, rule, to, ORIGIN, 0.8)?);
        moves.push(move_edge(stage, total_words, v(ROW_X, total_y + LIFT), LEFT, 0.8)?);
        moves.push(move_edge(stage, of_all, v(NUM_X + 0.2, total_y + LIFT), LEFT, 0.8)?);
        // the live count cannot travel: its still copy arrives with the rows
        stage.move_to(still_total, v(NUM_X, total_y + LIFT), RIGHT);
        let anim = fade_in(stage.arena_mut(), still_total, ORIGIN, 1.0)?;
        moves.push(stage.prepare(anim.rt(0.8))?);
        stage.play_prepared(moves)?;
        line.wait_for(stage, "about two percent")?;
        let share = tail_share(&world, k, Tail::AtLeast);
        let p_value = small_fraction(
            &kit,
            stage,
            (tail_sum, total),
            treated(),
            &format!("≈ {}%", share.percent(0)),
            (SMALL_AT[0], P_Y),
        )?;
        fade_in_all(stage, &[p_value], 0.6)?;
        line.wait_for(stage, "one-sided p-value")?;
        let name = kit.text(stage, P_NAME, 26.0, treated(), v(SMALL_AT[0], P_NAME_Y))?;
        let lower = precisely(&kit, stage, &precise_text(&world, k))?;
        fade_in_all(stage, &[name, lower], 0.6)?;

        // 04_8: exactly fifteen, then more than fifteen: other sets, other numbers
        let line = say(&kit, stage, "04_8")?;
        line.wait_for(stage, "exactly fifteen")?;
        fade_out_all(stage, &[lower], 0.3)?;
        let mut wrong = Vec::new();
        for (n, (which, word, phrase)) in [
            (Tail::Exactly, EXACTLY, "exactly fifteen"),
            (Tail::MoreThan, MORE_THAN, "more than fifteen"),
        ]
        .into_iter()
        .enumerate()
        {
            line.wait_for(stage, phrase)?;
            let members = tail_members(pairs, k, which);
            let red = around_bars(stage, &frame, &world, &members)?;
            kit.allow_crossing(red);
            let words = format!(
                "{word} {k}: {} {OF} {} ≈ {}%",
                thousands(tail_count(&world, k, which)),
                thousands(total),
                tail_share(&world, k, which).percent(1)
            );
            let text = kit.text_left(stage, &words, 22.0, warn(), v(ROW_X, WRONG_Y[n]))?;
            fade_in_all(stage, &[red, text], 0.5)?;
            wrong.extend([red, text]);
        }

        // 04_9: neither is our test: the direction was fixed in advance; the
        // tail starts at this result
        let line = say(&kit, stage, "04_9")?;
        fade_out_all(stage, &wrong, 0.4)?;
        line.wait_for(stage, "fixed in advance")?;
        let fixed = kit.text_left(stage, FIXED, 20.0, Palette::ink(), v(ROW_X, CARD_Y[0]))?;
        fade_in_all(stage, &[fixed], 0.4)?;
        line.wait_for(stage, "this result")?;
        let set_by = kit.text_left(stage, &format!("{SET_BY} {k}"), 20.0, treated(), v(ROW_X, CARD_Y[1]))?;
        fade_in_all(stage, &[set_by], 0.4)?;
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for m in [bracket, p_value] {
            let anim = fmn::animation::indicate(stage.arena_mut(), m, 1.15, None)?;
            pulse.push(stage.prepare(anim.rt(0.6))?);
        }
        stage.play_prepared(pulse)?;

        // 04_10: every one of those ways lives in the world of no effect
        say(&kit, stage, "04_10")?;
        let title_box = outline(
            stage,
            (frame.center[0] - TITLE_HALF_W, TITLE_TOP),
            (frame.center[0] + TITLE_HALF_W, TITLE_BOTTOM),
        )?;
        kit.allow_crossing(title_box);
        let caption = kit.text(stage, DENOMINATOR, 22.0, Palette::ink(), v(0.0, CAPTION_Y))?;
        fade_in_all(stage, &[title_box, caption], 0.5)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        // chapter 05 opens on the tail, its p-value and the world it counts in
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::animatic::wrap;

    use super::super::imaginary_world::{SMALL, hist_frame};
    use super::super::same_test::{PAIR_H, STUDY_TAG_Y};
    use super::*;

    /// The ledger is the tail's bars counted in assignments and sums to the
    /// tail; the tail starts at this result; each flipped pair gives one of
    /// the twenty ways to get nineteen.
    #[test]
    fn the_ledger_is_the_tail_and_the_flips_give_nineteen() {
        let s = study();
        let world = s.null_world();
        let k = s.observed();
        let lit = tail_members(s.pairs, k, Tail::AtLeast);
        assert_eq!(lit.first(), Some(&k), "the tail starts at this result");
        assert_eq!(lit.last(), Some(&s.pairs));
        let ledger: u64 = lit.iter().map(|&j| world[j as usize]).sum();
        assert_eq!(ledger, tail_count(&world, k, Tail::AtLeast));
        assert_eq!(ledger, 21_700);
        assert_eq!(world[s.pairs as usize], 1);
        assert_eq!(world[s.pairs as usize - 1], u64::from(s.pairs));
        assert_eq!(s.favouring(s.favouring_all_but(&[])), s.pairs);
        for &j in &FLIP_PAIRS {
            assert!(j < s.pairs);
            assert_eq!(s.favouring(s.favouring_all_but(&[j])), s.pairs - 1);
        }
        let mut distinct = FLIP_PAIRS.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), FLIP_PAIRS.len());
    }

    /// The red outlines are the exact and the strict tails, each a different
    /// subset of the lavender one, and between them all of it.
    #[test]
    fn the_wrong_tails_are_other_subsets_of_the_tail() {
        let s = study();
        let k = s.observed();
        let at_least = tail_members(s.pairs, k, Tail::AtLeast);
        let exactly = tail_members(s.pairs, k, Tail::Exactly);
        let more = tail_members(s.pairs, k, Tail::MoreThan);
        assert_eq!(exactly, vec![k]);
        assert_eq!(more.first(), Some(&(k + 1)));
        assert!(exactly.iter().chain(&more).all(|j| at_least.contains(j)));
        let mut both = [exactly, more].concat();
        both.sort_unstable();
        assert_eq!(both, at_least);
    }

    /// The words: the exact version names the counts, the share and the
    /// assumptions, and fits two lines of the lower third, clear of the axis;
    /// 04_9's lines say the direction was fixed in advance and the boundary
    /// was set by the data, with no number in the fixed one.
    #[test]
    fn the_words_say_what_was_fixed_and_what_was_set() {
        let s = study();
        let world = s.null_world();
        let exact = precise_text(&world, s.observed());
        assert!(
            exact.contains("under the sharp null, 21,700 / 1,048,576 ≈ 0.0207"),
            "{exact}"
        );
        assert!(
            exact.contains("give 15 or more favouring. Not P(no effect)."),
            "{exact}"
        );
        // the kit wraps at 92 characters and narrows when a line runs long
        for width in (72..=92).step_by(4) {
            assert!(wrap(&exact, width, 9).len() <= 2, "{width}: {exact}");
        }
        assert!(FIXED.starts_with("fixed in advance") && FIXED.contains("direction"));
        assert!(
            !FIXED.chars().any(|c| c.is_ascii_digit()),
            "no cutoff was chosen in advance"
        );
        assert!(SET_BY.starts_with("set by the data") && SET_BY.contains("this result"));
    }

    /// The layout: the ledger starts under the small pairs and, lifted, stays
    /// under the study's tag and above the p-value's name; the fraction
    /// clears the lower third and the lines under it; every line keeps clear
    /// of the histogram and the frame's foot.
    #[test]
    fn the_left_half_stacks_without_collisions() {
        // a line of text's half height and width by size, generously
        let half = |size: f64| 0.0068 * size;
        let width = |s: &str, size: f64| s.chars().count() as f64 * 0.0058 * size;
        // a 36-point TeX fraction's half height
        const FRACTION_HALF: f64 = 0.45;
        let s = study();
        let (pairs, k) = (s.pairs, s.observed());
        let grid_bottom = small(pair_at(19))[1] - PAIR_H * SMALL / 2.0;
        assert!(row_y(pairs, pairs) + half(22.0) < grid_bottom - 0.05, "{grid_bottom}");
        let total_y = row_y(k, pairs) - TOTAL_DY;
        assert!(total_y - half(24.0) > -4.0 + 0.25);
        assert!(row_y(pairs, pairs) + LIFT + half(22.0) < STUDY_TAG_Y - half(22.0) - 0.1);
        assert!(total_y + LIFT - half(24.0) > P_NAME_Y + half(26.0) + 0.1);
        assert!(P_NAME_Y - half(26.0) > P_Y + FRACTION_HALF + 0.1);
        // the lower third's two lines stand under the fraction and under the
        // histogram's axis label
        let lower_top = -4.0 + 0.2 + 0.3 + 0.36 * 2.0;
        assert!(P_Y - FRACTION_HALF > lower_top + 0.05);
        let axis_label = hist_frame(&s.null_world()).bottom() - 0.62;
        assert!(axis_label - half(20.0) > lower_top + 0.05, "{axis_label}");
        assert!(P_Y - FRACTION_HALF > WRONG_Y[0] + half(22.0) + 0.1);
        assert!(P_Y - FRACTION_HALF > CARD_Y[0] + half(20.0) + 0.1);
        assert!(WRONG_Y[0] - WRONG_Y[1] > 2.0 * half(22.0) + 0.05);
        assert!(CARD_Y[0] - CARD_Y[1] > 2.0 * half(20.0) + 0.05);
        assert!(CARD_Y[1] - half(20.0) > CAPTION_Y + half(22.0) + 0.1);
        assert!(CAPTION_Y - half(22.0) > -4.0 + 0.15);
        // left of the histogram's window, inside the title-safe edge
        let hist_left = hist_frame(&s.null_world()).left();
        const {
            assert!(ROW_X > -7.1 + 0.25);
        }
        let rights = [
            ROW_X + width(FIXED, 20.0),
            ROW_X + width(&format!("{SET_BY} {k}"), 20.0),
            ROW_X + width("more than 15: 6,196 of 1,048,576 ≈ 0.6%", 22.0),
            NUM_X + 0.2 + width("of 1,048,576", 22.0),
        ];
        for right in rights {
            assert!(right < hist_left - 0.3, "{right} vs {hist_left}");
        }
        assert!(width(DENOMINATOR, 22.0) / 2.0 < 7.1 - 0.25);
    }
}
