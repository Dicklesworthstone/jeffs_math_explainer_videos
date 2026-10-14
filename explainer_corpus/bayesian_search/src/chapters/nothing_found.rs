//! Chapter 06, nothing found: Bayes' rule done as counting, before it has
//! a name. The search of B comes back empty (06_1); the twenty dots it
//! would have caught take rings, and go out (06_2, 06_3); the eighty that
//! are left drop into one row against a fixed ruler from 0 to 100, ten of
//! them B's (06_4); "your turn": ten out of eighty, what is B's chance now
//! (06_5); the row stretches to fill the ruler and B's ten span 12.5
//! (06_6); A did not change and its share did, because forty of eighty is
//! one half (06_7); and every cell we did not search grew by a quarter
//! (06_8).
//!
//! The row is made of copies of the lit dots, so the sea's own dots never
//! move ("in the sea A's dots have not moved") and the copies go back
//! where they came from.

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CARRY_HOLD_S, QuizFrame, SEA_DOT_R, Sea, SeaDot, Strip, fade_from_black, fade_in_all, fade_out_all, fail, find,
    flat, gold_tex, in_cell, line_style, map, pause, percent_tex, quote_card, say, sea_dots, sea_slot, sea_split,
    search, tween_linear, wait_until,
};
use crate::model::{Q, sea};

pub struct NothingFound {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["06_1", "06_2", "06_3", "06_4", "06_5", "06_6", "06_7", "06_8"];

// the phrases the pictures land on (a test finds each in its line)
const P_EMPTY: &str = "comes back empty";
const P_LOOK: &str = "Look at the dots";
const P_THIRTY: &str = "Thirty are in B";
const P_TWO_IN_THREE: &str = "two out of every three";
const P_TWENTY: &str = "Twenty dots";
const P_GO_OUT: &str = "So they go out";
const P_STILL: &str = "Ten are still in B";
const P_ONE_IN_EIGHT: &str = "One in eight";
const P_HALF: &str = "But forty dots out of eighty";
const P_KNOW: &str = "What changed is what we know";
const P_FORTY: &str = "Forty to fifty";
const P_TWENTY_FIVE: &str = "Twenty to twenty-five";
const P_TWELVE: &str = "Ten to twelve and a half";

// fixed words
const EMPTY: &str = "empty";
const CAUGHT: &str = "would have caught";
const RULER_LO: &str = "0";
const RULER_HI: &str = "100";
const RULER_MID: &str = "50";
const DOTS_WORD: &str = "dots";
const DOTS_DIVIDED: &str = "÷";
const STONE: &str = "Stone and others, 2014";
const SUBJECTIVE: &str =
    "\u{201C}The probability distribution on which the search is based is therefore a subjective one.\u{201D}";

/// The row of lit dots: its left end, its width (the ruler, 0 to 100), its
/// height, and the dots' size against the sea's.
const ROW_X0: f64 = -6.0;
const ROW_W: f64 = 12.0;
const ROW_Y: f64 = -2.3;
const ROW_SCALE: f64 = 0.6;

/// Where dot `i` of the row stands when its groups are spread by `spread`
/// (1 as they drop, 100 / 80 once the row is stretched to fill the ruler).
pub fn row_x(i: usize, spread: f64) -> f64 {
    ROW_X0 + (i as f64 + 0.5) * (ROW_W / 100.0) * spread
}

/// A percent as the stage prints it: whole where it is whole (50), one
/// decimal where it is not (12.5).
pub fn pct_text(q: Q) -> String {
    let s = q.percent(1);
    format!("{}%", s.strip_suffix(".0").unwrap_or(&s))
}

/// A fraction above one as TeX: 5/4 is `1\frac{1}{4}`.
pub fn mixed_tex(q: Q) -> String {
    let s = q.to_string();
    match s.split_once('/') {
        Some((n, d)) => {
            let (n, d): (i128, i128) = (n.parse().unwrap_or(0), d.parse().unwrap_or(1));
            if n / d > 0 {
                format!("{}\\frac{{{}}}{{{d}}}", n / d, n % d)
            } else {
                format!("\\frac{{{n}}}{{{d}}}")
            }
        }
        None => s,
    }
}

/// The dots still lit after a search of cell `searched` has put out its
/// first `out_n` (in the cell's seeded order), as indices into the list
/// whose cells are `cells`, in row order: A's, then B's, C's and D's.
pub fn lit_indices(cells: &[usize], searched: usize, out_n: usize) -> Vec<usize> {
    let mut lit = Vec::new();
    for cell in 0..4 {
        let skip = if cell == searched { out_n } else { 0 };
        lit.extend(
            cells
                .iter()
                .enumerate()
                .filter(|&(_, &c)| c == cell)
                .map(|(i, _)| i)
                .skip(skip),
        );
    }
    lit
}

/// The sea's dots still lit after that search, in row order.
pub fn lit_after(dots: &[SeaDot], searched: usize, out_n: usize) -> Vec<SeaDot> {
    let cells: Vec<usize> = dots.iter().map(|d| d.cell).collect();
    lit_indices(&cells, searched, out_n)
        .into_iter()
        .map(|i| dots[i])
        .collect()
}

/// A pulse on `mobs`: up by `k` and back, `rt` seconds each way.
fn pulse(stage: &mut Stage<'_>, mobs: &[Mob], k: f64, rt: f64) -> fmn::Result<()> {
    for scale in [k, 1.0 / k] {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in mobs {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .scale(scale)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Move `moves` (mover, place) together by their centres, scaling each by
/// `scale` on the way, one after another by `lag`, in `rt` seconds.
fn fly_scaled(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], scale: f64, rt: f64, lag: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, to) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .scale(scale)?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

impl SceneConstruct for NothingFound {
    fn name(&self) -> &str {
        "06_nothing_found"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let prior = sea::prior();
        let seeing = sea::seeing();
        let finds = sea::find(&prior);
        let run = sea::run(sea::Rule::Best, 1);
        let b = run[0].cell;
        let after = run[0].after;
        let lit0: [u32; 4] = prior.map(|p| (p * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0));
        let out_n: usize = (prior[b] * seeing[b] * Q::int(sea::DOTS)).fixed(0).parse().unwrap_or(0);
        let left_n: usize = lit0.iter().map(|&n| n as usize).sum::<usize>() - out_n;

        // the sea as chapter 05 left it: blue chances, teal seeing, gold finds
        let sea_map = Sea::build(&kit, stage, lit0)?;
        let dots = sea_dots(stage, sea_split());
        for m in sea_map.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.add_to_scene(sea_map.tag)?;
        for d in &dots {
            stage.add_to_scene(d.dot)?;
        }
        let mut blue = Vec::new();
        let mut gold = Vec::new();
        for c in 0..4 {
            blue.push(kit.text(stage, &pct_text(prior[c]), 28.0, map(), sea_slot(c, 0))?);
            let (n, d) = {
                let s = seeing[c].to_string();
                let (n, d) = s.split_once('/').unwrap_or((s.as_str(), "1"));
                (n.to_owned(), d.to_owned())
            };
            let teal = kit.text(stage, &format!("{n} in {d}"), 26.0, search(), sea_slot(c, 1))?;
            stage.add_to_scene(teal)?;
            gold.push(gold_tex(&kit, stage, &percent_tex(finds[c]), find(), c)?);
        }
        for &m in blue.iter().chain(&gold) {
            stage.add_to_scene(m)?;
        }
        fade_from_black(stage, 0.5)?;

        // 06_1: so we search cell B, and the search comes back empty
        let line = say(&kit, stage, LINES[0])?;
        wait_until(stage, line.sentence(1)? - 1.0)?;
        kit.mark(stage, super::sound::SEARCH_MARK);
        let (x0, y0, x1, y1) = sea_map.rect(b);
        let strip = Strip::build(stage, (x0, y0, x1, y1), 0.1 * (x1 - x0))?;
        for m in strip.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.play(tween_linear(strip.phase, 1.0, 0.9)?)?;
        line.wait_for(stage, P_EMPTY)?;
        let empty = kit.text(stage, EMPTY, 40.0, Palette::ink(), v((x0 + x1) / 2.0, (y0 + y1) / 2.0))?;
        fade_in_all(stage, &[empty], 0.4)?;
        fade_out_all(stage, &[strip.wash, strip.bar], 0.4)?;
        line.wait_end(stage)?;

        // 06_2: that is not nothing; thirty dots are in B and the search
        // would have caught two out of every three: twenty
        let line = say(&kit, stage, LINES[1])?;
        let in_b = in_cell(&dots, b);
        line.wait_for(stage, P_LOOK)?;
        fade_out_all(stage, &[empty], 0.3)?;
        line.wait_for(stage, P_THIRTY)?;
        let all_rings: Vec<Mob> = in_b.iter().map(|d| d.ring).collect();
        fade_in_all(stage, &all_rings, 0.5)?;
        line.wait_for(stage, P_TWO_IN_THREE)?;
        // the ones the search would not have caught lose their ring
        let missed: Vec<Mob> = in_b.iter().skip(out_n).map(|d| d.ring).collect();
        fade_out_all(stage, &missed, 0.5)?;
        line.wait_for(stage, P_TWENTY)?;
        let caught = stage.add_value_tracker(0.0);
        let count_at = v(sea_map.centre(b)[0] + 3.0, sea_map.centre(b)[1] + 0.25);
        kit.live_number(stage, caught, 0, false, 52.0, search(), count_at, ORIGIN)?;
        let said = kit.text(stage, CAUGHT, 22.0, Palette::dim(), v(count_at[0], count_at[1] - 0.6))?;
        fade_in_all(stage, &[said], 0.3)?;
        stage.play(tween_linear(caught, out_n as f64, 0.8)?)?;
        line.wait_end(stage)?;

        // 06_3: those twenty go out, B cools and A, C and D deepen in the
        // same instant; the chances give way to counts of lit dots
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, P_GO_OUT)?;
        let went: Vec<SeaDot> = in_b.iter().take(out_n).copied().collect();
        let mut together: Vec<Box<dyn Animation>> = Vec::new();
        for d in &went {
            let twin = fade_in(stage.arena_mut(), d.twin, ORIGIN, 1.0)?;
            together.push(stage.prepare(twin.rt(0.7))?);
            let ring = fade_out(stage.arena_mut(), d.ring, ORIGIN, 1.0)?;
            together.push(stage.prepare(ring.rt(0.5))?);
        }
        let lit_now: Vec<f64> = (0..4)
            .map(|c| {
                if c == b {
                    (lit0[c] as usize - out_n) as f64
                } else {
                    f64::from(lit0[c])
                }
            })
            .collect();
        for c in 0..4 {
            together.push(stage.prepare(explainer_kit::tween(sea_map.lit[c], lit_now[c], 0.8)?)?);
        }
        stage.play_prepared(together)?;
        // the chances give way to counts of lit dots, and the old chance of
        // a find, which belongs to the old map, greys
        let mut counts = Vec::new();
        for (c, &n) in lit_now.iter().enumerate() {
            counts.push(kit.text(stage, &format!("{}", n as usize), 28.0, map(), sea_slot(c, 0))?);
        }
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &blue {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            swap.push(stage.prepare(a.rt(0.4))?);
        }
        for &m in &counts {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            swap.push(stage.prepare(a.rt(0.4))?);
        }
        for &g in &gold {
            let a = g
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.5),
                    ..AnimateArgs::default()
                })?
                .set_fill(Some(Palette::dim()), None)?;
            swap.push(stage.prepare(a)?);
        }
        stage.play_prepared(swap)?;
        line.wait_end(stage)?;

        // 06_4: eighty left, ten of them B's: the lit dots drop into one
        // row under the sea, grouped A, B, C, D, against a fixed ruler
        let line = say(&kit, stage, LINES[3])?;
        let ruler = ruler(&kit, stage)?;
        fade_in_all(stage, &ruler, 0.5)?;
        let lit = lit_after(&dots, b, out_n);
        if lit.len() != left_n {
            return Err(fail(format!("{} dots are left, not {left_n}", lit.len())));
        }
        // copies of the lit dots, at the dots, then dropping into the row
        let mut copies: Vec<Mob> = Vec::new();
        for d in &lit {
            let c = stage
                .arena_mut()
                .add(Dot::new().point(d.at).radius(SEA_DOT_R).style(flat(map(), 1.0)).build());
            stage.add_to_scene(c)?;
            copies.push(c);
        }
        let drop: Vec<(Mob, Vec3)> = copies
            .iter()
            .enumerate()
            .map(|(i, &c)| (c, v(row_x(i, 1.0), ROW_Y)))
            .collect();
        fly_scaled(stage, &drop, ROW_SCALE, 1.4, 0.6)?;
        line.wait_for(stage, P_STILL)?;
        let b_copies: Vec<Mob> = lit
            .iter()
            .zip(&copies)
            .filter(|(d, _)| d.cell == b)
            .map(|(_, &c)| c)
            .collect();
        pulse(stage, &b_copies, 1.8, 0.25)?;
        line.wait_end(stage)?;

        // 06_5: your turn: ten dots out of eighty, what is B's chance now?
        // the answer lands over B's ten, as the frame lifts
        let line = say(&kit, stage, LINES[4])?;
        let frame = QuizFrame::your_turn(&kit, stage, &line)?;
        let answer = facts
            .get("F-Q.after_empty.answer")
            .ok_or_else(|| fail("no fact F-Q.after_empty.answer".into()))?
            .to_owned();
        let b_first = lit.iter().position(|d| d.cell == b).unwrap_or(0);
        let mid_b = row_x(b_first + b_copies.len() / 2, 1.0);
        let answer_label = kit.text(stage, &format!("{answer}%"), 36.0, map(), v(mid_b, ROW_Y + 0.55))?;
        let mut anims = frame.lift(stage, 0.5)?;
        let a = fade_in(stage.arena_mut(), answer_label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.45))?);
        stage.play_prepared(anims)?;

        // 06_6: one in eight, twelve and a half percent, down from thirty:
        // the row stretches to fill the ruler, every group by the same amount
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, P_ONE_IN_EIGHT)?;
        let spread = 100.0 / left_n as f64;
        let stretch: Vec<(Mob, Vec3)> = copies
            .iter()
            .enumerate()
            .map(|(i, &c)| (c, v(row_x(i, spread), ROW_Y)))
            .collect();
        let new_mid = row_x(b_first + b_copies.len() / 2, spread);
        let mut together: Vec<Box<dyn Animation>> = Vec::new();
        for &(c, to) in &stretch {
            let a = c
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            together.push(stage.prepare(a)?);
        }
        let a = answer_label
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.2),
                ..AnimateArgs::default()
            })?
            .move_to(v(new_mid, ROW_Y + 0.55), ORIGIN)?;
        together.push(stage.prepare(a)?);
        stage.play_prepared(together)?;
        line.wait_end(stage)?;

        // 06_7: A did not change and its chance did: forty of eighty is one
        // half; the ruler marks 50 where A's dots end
        let line = say(&kit, stage, LINES[6])?;
        line.wait_for(stage, P_HALF)?;
        let mid = ROW_X0 + ROW_W / 2.0;
        let half = kit.text(stage, RULER_MID, 24.0, Palette::ink(), v(mid, ROW_Y + RULER_DROP_REACH))?;
        let tick = stage.arena_mut().add(
            polyline(vec![v(mid, ROW_Y - RULER_DROP - 0.1), v(mid, ROW_Y + 0.25)])
                .style(line_style(Palette::ink(), 3.0))
                .build(),
        );
        fade_in_all(stage, &[half, tick], 0.5)?;
        // what changed is what we know: the authors' sentence on it comes in
        // as that is said, so that fifteen words are on the stage for the
        // line's last two seconds and the two after it, not for two in all
        // (the quotation is one sentence of ninety characters: the kit's
        // lower-third leaves "one.”" alone on a second line, so it is set
        // in two balanced lines)
        line.wait_for(stage, P_KNOW)?;
        let quote = quote_card(&kit, stage, STONE, SUBJECTIVE)?;
        fade_in_all(stage, &[quote], 0.5)?;
        line.wait_end(stage)?;
        pause(stage, CARRY_HOLD_S)?;
        fade_out_all(stage, &[quote], 0.4)?;

        // 06_8: the dots return to the sea; the percents come back under
        // the cells, every cell we did not search grown by a quarter
        let line = say(&kit, stage, LINES[7])?;
        let back: Vec<(Mob, Vec3)> = lit.iter().zip(&copies).map(|(d, &c)| (c, d.at)).collect();
        fly_scaled(stage, &back, 1.0 / ROW_SCALE, 1.2, 0.5)?;
        let mut leaving: Vec<Mob> = copies.clone();
        leaving.extend(ruler.iter().copied());
        leaving.extend([answer_label, half, tick]);
        fade_out_all(stage, &leaving, 0.4)?;
        fade_out_all(stage, &counts, 0.3)?;
        line.wait_for(stage, P_FORTY)?;
        let mut shown = Vec::new();
        for c in [0usize, 2, 1, 3] {
            shown.push((c, kit.text(stage, &pct_text(after[c]), 28.0, map(), sea_slot(c, 0))?));
        }
        // A first, then C, then B and D, as the line reads them
        fade_in_all(stage, &[shown[0].1], 0.4)?;
        line.wait_for(stage, P_TWENTY_FIVE)?;
        fade_in_all(stage, &[shown[1].1], 0.4)?;
        line.wait_for(stage, P_TWELVE)?;
        fade_in_all(stage, &[shown[2].1, shown[3].1], 0.4)?;
        // × 1¼, and why: a hundred dots over the eighty that are left
        let rise = run[0].after[0] / prior[0];
        let times = kit.tex(
            stage,
            &format!("\\times {}", mixed_tex(rise)),
            44.0,
            Palette::ink(),
            &[],
            v(3.3, 1.5),
        )?;
        let why = kit.text(
            stage,
            &format!("({} {DOTS_WORD} {DOTS_DIVIDED} {left_n} {DOTS_WORD})", sea::DOTS),
            26.0,
            Palette::dim(),
            v(3.3, 0.8),
        )?;
        fade_in_all(stage, &[times, why], 0.6)?;
        line.wait_end(stage)?;
        pause(stage, 0.8)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

/// How far under the row the ruler's line runs, how far above it the 50
/// mark reaches, and how far beside the ruler's ends their names stand.
const RULER_DROP: f64 = 0.2;
const RULER_DROP_REACH: f64 = 0.55;
const END_LABEL_GAP: f64 = 0.35;

/// The ruler under the row: a line from 0 to 100 with its two ends named
/// beside them, so nothing hangs under it into the question's band.
fn ruler(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let y = ROW_Y - RULER_DROP;
    let line = stage.arena_mut().add(
        polyline(vec![v(ROW_X0, y), v(ROW_X0 + ROW_W, y)])
            .style(line_style(Palette::dim(), 2.5))
            .build(),
    );
    let mut out = vec![line];
    for (x, words, side) in [(ROW_X0, RULER_LO, -1.0), (ROW_X0 + ROW_W, RULER_HI, 1.0)] {
        let tick = stage.arena_mut().add(
            polyline(vec![v(x, y + 0.08), v(x, y - 0.08)])
                .style(line_style(Palette::dim(), 2.5))
                .build(),
        );
        out.push(tick);
        out.push(kit.text(stage, words, 22.0, Palette::dim(), v(x + side * END_LABEL_GAP, y))?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::draw::balanced_lines;
    use super::*;

    /// (line, phrase): every phrase a picture lands on.
    const PHRASES: [(&str, &str); 13] = [
        ("06_1", P_EMPTY),
        ("06_2", P_LOOK),
        ("06_2", P_THIRTY),
        ("06_2", P_TWO_IN_THREE),
        ("06_2", P_TWENTY),
        ("06_3", P_GO_OUT),
        ("06_4", P_STILL),
        ("06_6", P_ONE_IN_EIGHT),
        ("06_7", P_HALF),
        ("06_7", P_KNOW),
        ("06_8", P_FORTY),
        ("06_8", P_TWENTY_FIVE),
        ("06_8", P_TWELVE),
    ];
    use crate::chapters::draw::{QUIZ_FRAME, QUIZ_LINES_Y, sea_rect};
    use explainer_kit::Script;

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("06_"))
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
    }

    /// 06_2 to 06_4: of B's thirty dots a search that sees two in three
    /// would have caught twenty, ten are still lit, and eighty are left in
    /// the sea; each is a fact.
    #[test]
    fn the_dots_are_counted_as_the_facts_count_them() {
        let f = crate::facts::facts().unwrap();
        let prior = sea::prior();
        let b = 1usize;
        let hundred = Q::int(sea::DOTS);
        let in_b = prior[b] * hundred;
        let out_n = in_b * sea::seeing()[b];
        assert_eq!(in_b, Q::int(30));
        assert_eq!(f.get("F-M.empty1.dots_out"), Some(out_n.to_string().as_str()));
        assert_eq!(f.get("F-M.empty1.dots_out"), Some("20"));
        assert_eq!(f.get("F-M.empty1.dots_left"), Some("80"));
        assert_eq!(f.get("F-M.empty1.cell_dots_left"), Some("10"));
        assert_eq!(in_b - out_n, Q::int(10));
        assert_eq!(f.get("F-Q.after_empty.answer"), Some("12.5"));
    }

    /// The row: eighty dots, A's forty then B's ten, C's twenty and D's ten,
    /// no two at one place, inside the ruler; stretched by 100 / 80 the row
    /// fills it and B's ten span 12.5, which is the quiz's answer.
    #[test]
    fn the_row_is_eighty_dots_that_fill_the_ruler_when_stretched() {
        let spread = 100.0 / 80.0;
        assert!((row_x(79, 1.0) + ROW_W / 100.0 / 2.0 - (ROW_X0 + 80.0 * ROW_W / 100.0)).abs() < 1e-9);
        for i in 0..80 {
            assert!(row_x(i, 1.0) >= ROW_X0 && row_x(i, 1.0) <= ROW_X0 + ROW_W);
            assert!(row_x(i, spread) <= ROW_X0 + ROW_W, "dot {i} past the ruler");
            if i > 0 {
                let gap = row_x(i, spread) - row_x(i - 1, spread);
                assert!(gap >= 2.0 * SEA_DOT_R * ROW_SCALE, "dots touch at {i}: {gap}");
            }
        }
        // the last dot of the stretched row ends at the ruler's end
        let end = row_x(79, spread) + ROW_W / 100.0 * spread / 2.0;
        assert!((end - (ROW_X0 + ROW_W)).abs() < 1e-9);
        // B's ten span 12.5 units of the ruler: a tenth of the row is an
        // eighth of the hundred
        let span = 10.0 * spread;
        assert!((span - 12.5).abs() < 1e-9);
        // A's forty end at 50
        assert!((40.0 * spread - 50.0).abs() < 1e-9);
        let facts = crate::facts::facts().unwrap();
        assert_eq!(facts.get("F-Q.after_empty.answer"), Some("12.5"));
    }

    /// 06_8: "grew by a quarter", and the three stretches it names.
    #[test]
    fn every_cell_we_did_not_search_grew_by_a_quarter() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("F-M.empty1.rise"), Some("5/4"));
        assert_eq!(f.get("F-M.empty1.after.percent"), Some("50.0,12.5,25.0,12.5"));
        let (p, after) = (sea::prior(), sea::run(sea::Rule::Best, 1)[0].after);
        let b = 1;
        for c in [0usize, 2, 3] {
            assert_eq!(after[c] / p[c], Q::lowest(5, 4), "cell {c}");
        }
        assert!(after[b] < p[b], "the searched cell fell");
        let shown: Vec<String> = after.iter().map(|&q| pct_text(q)).collect();
        assert_eq!(shown, ["50%", "12.5%", "25%", "12.5%"]);
        assert_eq!(mixed_tex(Q::lowest(5, 4)), "1\\frac{1}{4}");
        assert_eq!(mixed_tex(Q::lowest(1, 4)), "\\frac{1}{4}");
        // the pairs the line says, in order: 40 to 50, 20 to 25, 10 to 12.5
        let before: Vec<String> = [0usize, 2, 3].iter().map(|&c| p[c].percent(0)).collect();
        assert_eq!(before, ["40", "20", "10"]);
    }

    /// The edges of "a cell not searched grows": a search that sees
    /// nothing changes no cell, and one that sees everything empties its
    /// cell and scales the rest by one over what is left.
    #[test]
    fn a_blind_search_changes_nothing_and_a_sure_one_empties_its_cell() {
        let p = sea::prior();
        assert_eq!(sea::bayes(&p, 1, Q::ZERO), p);
        let sure = sea::bayes(&p, 1, Q::ONE);
        assert_eq!(sure[1], Q::ZERO);
        let left = Q::ONE - p[1];
        for c in [0usize, 2, 3] {
            assert_eq!(sure[c], p[c] / left);
        }
    }

    /// The lit dots after B's search, in row order: A's forty, B's last ten
    /// (its first twenty are out), C's twenty and D's ten, each cell's in
    /// its own order.
    #[test]
    fn the_lit_dots_go_into_the_row_cell_by_cell() {
        // the sea's dots by cell, as `sea_dots` lists them: 40, 30, 20, 10
        let cells: Vec<usize> = [40usize, 30, 20, 10]
            .iter()
            .enumerate()
            .flat_map(|(c, &n)| std::iter::repeat_n(c, n))
            .collect();
        let lit = lit_indices(&cells, 1, 20);
        assert_eq!(lit.len(), 80);
        let by_cell: Vec<usize> = (0..4).map(|c| lit.iter().filter(|&&i| cells[i] == c).count()).collect();
        assert_eq!(by_cell, [40, 10, 20, 10]);
        // row order is cell order, and B's are its last ten in its order
        assert!(lit.windows(2).all(|w| cells[w[0]] <= cells[w[1]]));
        let b: Vec<usize> = lit.iter().copied().filter(|&i| cells[i] == 1).collect();
        assert_eq!(b, (60..70).collect::<Vec<_>>());
        // a search that puts out nothing leaves all hundred
        assert_eq!(lit_indices(&cells, 1, 0).len(), 100);
        // one that puts out the whole cell leaves none of it
        assert!(lit_indices(&cells, 1, 30).iter().all(|&i| cells[i] != 1));
        assert_eq!(pct_text(Q::lowest(1, 8)), "12.5%");
        assert_eq!(pct_text(Q::lowest(1, 2)), "50%");
    }

    /// The kit's lower-third left "one.”" alone on a second line of the
    /// quotation (the label "Stone and others, 2014" leaves about 86
    /// characters a line); the card now breaks it in two balanced lines.
    #[test]
    fn the_quotation_is_two_balanced_lines_not_a_line_and_an_orphan() {
        let lines = balanced_lines(SUBJECTIVE, 86);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(lines.join(" "), SUBJECTIVE);
        let (a, b) = (lines[0].chars().count(), lines[1].chars().count());
        assert!(a.abs_diff(b) <= 20, "{a} and {b}");
        assert!(b >= 30, "the second line is {lines:?}");
    }

    #[test]
    fn the_row_the_ruler_and_the_question_do_not_touch() {
        // the sea's lowest sums stay above the row of dots (a mixed fraction
        // is the lowest: 13⅓% stands TALL_DROP under its slot and reaches
        // TALL_HALF_H either side of that); the ruler's line and the 50
        // mark stay above the question's first line
        use super::super::draw::{TALL_DROP, TALL_HALF_H};
        let slot_low = super::super::draw::sea_slot(2, 2)[1] - TALL_DROP - TALL_HALF_H;
        assert!(
            slot_low > ROW_Y + SEA_DOT_R * ROW_SCALE + 0.05,
            "the sums clear the row's dots"
        );
        // the 50 mark reaches up beside the sea, not into it
        assert!(
            ROW_X0 + ROW_W / 2.0 - 0.3 > sea_rect(1).2,
            "the 50 stands right of the sea"
        );
        // the ruler's line clears the question's first line (constants,
        // checked as the crate builds)
        const {
            assert!(ROW_Y - RULER_DROP - 0.1 > QUIZ_LINES_Y[0] + 0.2);
        }
        assert!(ROW_X0 - END_LABEL_GAP > -QUIZ_FRAME.0 && ROW_X0 + ROW_W + END_LABEL_GAP + 0.3 < QUIZ_FRAME.0);
        assert!(sea_rect(2).1 > ROW_Y);
    }
}
