//! Chapter 03, a map of chances: the Navy's grid leaves and the stage
//! stands empty (03_1); four cells and two story cards (03_2); each story
//! shades its own tiny map (03_3); weights land on the cards and a hundred
//! dots leave them, sixty settling in the cells and forty hanging above
//! (03_4); the first "your turn", with thirty dots in A and forty hanging
//! (03_5); the forty fall, ten to a cell (03_6); and the cards go, leaving
//! the map: each cell's dots, its percent, its blue (03_7).
//!
//! Every count is a dot the viewer can see: the cells' counts are the
//! trackers the blue is recounted from, so a cell deepens as its dots land.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    CARRY_HOLD_S, Heat, NAVY_CENTRE, NAVY_ROW_H, NavyGrid, QuizFrame, Sea, SeaDot, fade_in_all, fade_in_lagged,
    fade_out_all, fail, fit_width, flat, fly_all, frac_tex, map, panel, pause, say, scatter, sea_dots, sea_rect,
    sea_slot, sea_split, tween_all,
};
use crate::model::sea;

pub struct AMapOfChances {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 7] = ["03_1", "03_2", "03_3", "03_4", "03_5", "03_6", "03_7"];

/// The bare stage between the Navy's grid leaving and the made-up sea coming, seconds.
const BEAT_S: f64 = 0.25;

// the phrases the pictures land on (a test finds each in its line)
const P_CELLS: &str = "Four cells";
const P_STORIES: &str = "two stories";
const P_A: &str = "most likely in cell A";
const P_B: &str = "maybe in B";
const P_C: &str = "less likely in C";
const P_STRAYED: &str = "In the second it strayed";
const P_WEIGH: &str = "Say sixty to forty";
const P_PICTURE: &str = "Then picture a hundred dots";
const P_FIRST: &str = "sixty from the first story";
const P_SECOND: &str = "forty from the second";
const P_FALL: &str = "A quarter of forty";
const P_SUM: &str = "Forty dots in a hundred";
const P_COUNT: &str = "Count every cell";

// fixed words
const KEPT: &str = "kept to its route";
const STRAYED: &str = "strayed";
const WEIGHT: &str = "weight";
const PRIOR: &str = "prior";
const PRIOR_WORDS: &str = "the map before a search";

/// The two cards' middles, and what is in one: its back, a tiny map of
/// four cells (two by two, as the sea), its weight above and its caption
/// below.
const CARD_X: [f64; 2] = [1.9, 5.25];
const CARD_SIZE: (f64, f64) = (3.0, 3.1);
/// The cards' height: their tops stand under the countdown ring of 03_5
/// (which sits in the frame's top right corner and crossed "weight 40" when
/// the cards were top-aligned with the sea at 1.65).
const CARD_Y: f64 = 0.9;
const TINY: (f64, f64) = (1.3, 0.85);
const TINY_Y: f64 = CARD_Y;

/// How far apart the three lines under a cell stand here: the count, the
/// percent, the sum. The sea's slots are 0.27 apart and a 30 pt count's
/// digits are as tall as that, so the percent at the second slot touched it
/// (the narrated draft's 03_6 tile).
const UNDER_STEP: [f64; 3] = [0.0, 0.31, 0.62];

/// Where line `k` (0 the count) under cell `cell` is centred.
fn under(cell: usize, k: usize) -> Vec3 {
    let top = sea_slot(cell, 0);
    v(top[0], top[1] - UNDER_STEP[k])
}

/// A story card on the stage.
struct Card {
    back: Mob,
    weight: Mob,
    caption: Mob,
    borders: [Mob; 4],
    /// The shade over each tiny cell, at no opacity until it is shaded.
    shades: [Mob; 4],
    /// The share in each tiny cell, as TeX, detached.
    labels: [Mob; 4],
    centres: [Vec3; 4],
}

impl Card {
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back, self.caption];
        out.extend(self.borders);
        out
    }
}

fn tiny_centres(cx: f64) -> [Vec3; 4] {
    [
        v(cx - TINY.0 / 2.0, TINY_Y + TINY.1 / 2.0),
        v(cx + TINY.0 / 2.0, TINY_Y + TINY.1 / 2.0),
        v(cx - TINY.0 / 2.0, TINY_Y - TINY.1 / 2.0),
        v(cx + TINY.0 / 2.0, TINY_Y - TINY.1 / 2.0),
    ]
}

fn story_card(kit: &Kit, stage: &mut Stage<'_>, story: usize) -> fmn::Result<Card> {
    let s = &sea::stories()[story];
    let cx = CARD_X[story];
    let back = Rectangle::new()
        .width(CARD_SIZE.0)
        .height(CARD_SIZE.1)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(v(cx, CARD_Y));
    let back = stage.arena_mut().add(back);
    let centres = tiny_centres(cx);
    let mut borders = Vec::new();
    let mut shades = Vec::new();
    let mut labels = Vec::new();
    for (i, &c) in centres.iter().enumerate() {
        let shade = Rectangle::new()
            .width(TINY.0)
            .height(TINY.1)
            .style(flat(map(), 0.0))
            .build()?
            .moved_to(c);
        shades.push(stage.arena_mut().add(shade));
        let edge = Rectangle::new()
            .width(TINY.0)
            .height(TINY.1)
            .style(Style::default().fill(map(), 0.0).stroke(Palette::faint(), 1.6, 1.0))
            .build()?
            .moved_to(c);
        borders.push(stage.arena_mut().add(edge));
        labels.push(kit.tex(stage, &frac_tex(s.share[i]), 30.0, Palette::ink(), &[], c)?);
    }
    let weight = kit.text(
        stage,
        &format!("{WEIGHT} {}", s.weight.percent(0)),
        30.0,
        Palette::ink(),
        v(cx, CARD_Y + CARD_SIZE.1 / 2.0 - 0.38),
    )?;
    let caption = kit.text(
        stage,
        [KEPT, STRAYED][story],
        24.0,
        Palette::dim(),
        v(cx, CARD_Y - CARD_SIZE.1 / 2.0 + 0.4),
    )?;
    fit_width(stage, caption, CARD_SIZE.0 - 0.3);
    Ok(Card {
        back,
        weight,
        caption,
        borders: [borders[0], borders[1], borders[2], borders[3]],
        shades: [shades[0], shades[1], shades[2], shades[3]],
        labels: [labels[0], labels[1], labels[2], labels[3]],
        centres,
    })
}

/// Shade tiny cell `i` of `card` to its share, and bring its number in.
fn shade(stage: &mut Stage<'_>, card: &Card, story: usize, i: usize, rt: f64) -> fmn::Result<()> {
    let share = sea::stories()[story].share[i].f64();
    let a = card.shades[i]
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_fill(Some(map()), Some(Heat::SEA.opacity(share)))?;
    let shade_anim = stage.prepare(a)?;
    let label = fade_in(stage.arena_mut(), card.labels[i], ORIGIN, 1.0)?;
    let label = stage.prepare(label.rt(rt))?;
    stage.play_prepared(vec![shade_anim, label])?;
    Ok(())
}

/// The dots of one story, in cell order, with where each starts on its
/// card (inside the tiny cell of its own cell, seeded) and where it
/// hangs, waiting, above its cell.
struct Wave {
    dots: Vec<SeaDot>,
    on_card: Vec<Vec3>,
    hanging: Vec<Vec3>,
}

/// How the forty hang above a cell: ten dots in two rows of five over the
/// cell's OUTER half (the left half of A and C, the right half of B and D),
/// `HANG_SIDE` from its middle line, a dot every `HANG_STEP`, the first row
/// `HANG_RISE` over the cell's top and the second `HANG_ROW` over that. The
/// middle stays clear: the quiz's answer ("40%") lands on the line of cell
/// A's own percent, which is the height the bottom row's dots hang at (it
/// was crossed by them for two seconds, in the dots' own colour). The outer
/// side also keeps the top row's dots away from the sea's tag between them.
const HANG_SIDE: f64 = 0.85;
const HANG_STEP: f64 = 0.18;
const HANG_RISE: f64 = 0.24;
const HANG_ROW: f64 = 0.2;

/// Where dot `j` of the dots hanging above `cell` waits.
fn hang_at(cell: usize, j: usize) -> Vec3 {
    let (x0, _, x1, top) = sea_rect(cell);
    let side = if cell.is_multiple_of(2) { -1.0 } else { 1.0 };
    v(
        (x0 + x1) / 2.0 + side * HANG_SIDE + ((j % 5) as f64 - 2.0) * HANG_STEP,
        top + HANG_RISE + HANG_ROW * (j / 5) as f64,
    )
}

fn wave(all: &[SeaDot], story: usize, centres: &[Vec3; 4]) -> Wave {
    let mut dots = Vec::new();
    let mut on_card = Vec::new();
    let mut hanging = Vec::new();
    for (cell, centre) in centres.iter().enumerate() {
        let mine: Vec<SeaDot> = all
            .iter()
            .copied()
            .filter(|d| d.cell == cell && d.story == story)
            .collect();
        let pts = scatter(
            mine.len(),
            0.8 * TINY.0,
            0.78 * TINY.1,
            0xCA2D + (10 * story + cell) as u64,
        );
        for (j, d) in mine.into_iter().enumerate() {
            on_card.push(v(centre[0] + pts[j].0, centre[1] + pts[j].1));
            hanging.push(hang_at(cell, j));
            dots.push(d);
        }
    }
    Wave { dots, on_card, hanging }
}

impl SceneConstruct for AMapOfChances {
    fn name(&self) -> &str {
        "03_a_map_of_chances"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let answer = facts
            .get("F-Q.stories.answer")
            .ok_or_else(|| fail("no fact F-Q.stories.answer".into()))?
            .to_owned();

        // 03_1: chapter 02 ended on the Navy's grid, a quarter painted; it
        // stays under the whole line and leaves as 03_2 begins, and the
        // stage stands empty for a beat (BEAT_S) before the made-up sea's
        // tag (the grid left at "a map that gives every place its chance"
        // and the stage stood empty for 2.4 s over the finished read)
        let grid = NavyGrid::build(stage, NAVY_CENTRE, NAVY_ROW_H)?;
        // the cells the broad-area track had painted (`F-H.track.cells`)
        let track_cells: usize = facts
            .get("F-H.track.cells")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| fail("no fact F-H.track.cells".into()))?;
        let painted: Vec<Mob> = grid.washes.iter().take(track_cells).copied().collect();
        let mut on_stage: Vec<Mob> = grid.lines.clone();
        on_stage.extend(&painted);
        for &m in &on_stage {
            stage.add_to_scene(m)?;
        }
        let line = say(&kit, stage, LINES[0])?;
        line.wait_end(stage)?;

        // 03_2: four cells draw themselves, two by two, tagged; two story
        // cards beside them
        let line = say(&kit, stage, LINES[1])?;
        let sea_map = Sea::build(&kit, stage, [0; 4])?;
        let cards = [story_card(&kit, stage, 0)?, story_card(&kit, stage, 1)?];
        fade_out_all(stage, &on_stage, 0.3)?;
        pause(stage, BEAT_S)?;
        fade_in_all(stage, &[sea_map.tag], 0.4)?;
        line.wait_for(stage, P_CELLS)?;
        fade_in_lagged(stage, &sea_map.mobs(), 1.2, 0.5)?;
        line.wait_for(stage, P_STORIES)?;
        let mut cards_in: Vec<Mob> = Vec::new();
        for c in &cards {
            cards_in.extend(c.mobs());
        }
        fade_in_all(stage, &cards_in, 0.7)?;
        line.wait_end(stage)?;

        // 03_3: each story shades its own tiny map: a half, a third, a
        // sixth, nothing; then the story that strayed, evenly
        let line = say(&kit, stage, LINES[2])?;
        line.wait_for(stage, P_A)?;
        shade(stage, &cards[0], 0, 0, 0.5)?;
        line.wait_for(stage, P_B)?;
        shade(stage, &cards[0], 0, 1, 0.5)?;
        line.wait_for(stage, P_C)?;
        shade(stage, &cards[0], 0, 2, 0.5)?;
        shade(stage, &cards[0], 0, 3, 0.4)?;
        line.wait_for(stage, P_STRAYED)?;
        for i in 0..4 {
            shade(stage, &cards[1], 1, i, 0.4)?;
        }
        line.wait_end(stage)?;

        // 03_4: the weights land; a hundred dots, sixty from the first
        // story and forty from the second; the sixty settle in the cells
        // and the forty hang above them
        let line = say(&kit, stage, LINES[3])?;
        line.wait_for(stage, P_WEIGH)?;
        fade_in_all(stage, &[cards[0].weight, cards[1].weight], 0.5)?;
        let dots = sea_dots(stage, sea_split());
        let first = wave(&dots, 0, &cards[0].centres);
        let second = wave(&dots, 1, &cards[1].centres);
        // every dot starts on its card
        for w in [&first, &second] {
            for (d, &at) in w.dots.iter().zip(&w.on_card) {
                stage.move_to(d.dot, at, ORIGIN);
            }
        }
        line.wait_for(stage, P_PICTURE)?;
        let first_mobs: Vec<Mob> = first.dots.iter().map(|d| d.dot).collect();
        let second_mobs: Vec<Mob> = second.dots.iter().map(|d| d.dot).collect();
        fade_in_all(stage, &first_mobs, 0.5)?;
        line.wait_for(stage, P_FIRST)?;
        // the sixty settle where they belong, and each cell counts them
        let settle: Vec<(Mob, Vec3)> = first.dots.iter().map(|d| (d.dot, d.at)).collect();
        let first_counts: Vec<(Mob, f64)> = (0..4)
            .map(|c| (sea_map.lit[c], first.dots.iter().filter(|d| d.cell == c).count() as f64))
            .collect();
        // a count under each cell, recounted from the trackers the blue
        // follows (it stays through 03_7: "its dots")
        for (c, &lit) in sea_map.lit.iter().enumerate() {
            kit.live_number(stage, lit, 0, false, 30.0, map(), sea_slot(c, 0), ORIGIN)?;
        }
        fly_all(stage, &settle, 1.4, 0.7)?;
        tween_all(stage, &first_counts, 0.4)?;
        line.wait_for(stage, P_SECOND)?;
        fade_in_all(stage, &second_mobs, 0.5)?;
        let hang: Vec<(Mob, Vec3)> = second
            .dots
            .iter()
            .zip(&second.hanging)
            .map(|(d, &p)| (d.dot, p))
            .collect();
        fly_all(stage, &hang, 1.2, 0.6)?;
        line.wait_end(stage)?;

        // 03_5: your turn: thirty dots in A, forty hanging above the sea
        let line = say(&kit, stage, LINES[4])?;
        let frame = QuizFrame::your_turn(&kit, stage, &line)?;
        // the answer lands where A's percent will stand, as the frame lifts
        let answer_label = kit.text(stage, &format!("{answer}%"), 32.0, map(), under(0, 1))?;
        let mut anims = frame.lift(stage, 0.5)?;
        let a = fade_in(stage.arena_mut(), answer_label, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.45))?);
        stage.play_prepared(anims)?;

        // 03_6: the forty fall, ten to a cell; in A, 30 + 10 = 40
        let line = say(&kit, stage, LINES[5])?;
        line.wait_for(stage, P_FALL)?;
        let fall: Vec<(Mob, Vec3)> = second.dots.iter().map(|d| (d.dot, d.at)).collect();
        let all_counts: Vec<(Mob, f64)> = (0..4)
            .map(|c| (sea_map.lit[c], dots.iter().filter(|d| d.cell == c).count() as f64))
            .collect();
        fly_all(stage, &fall, 1.2, 0.6)?;
        tween_all(stage, &all_counts, 0.4)?;
        line.wait_for(stage, P_SUM)?;
        let (from_first, from_second) = (sea_split()[0][0], sea_split()[1][0]);
        let working = kit.text(
            stage,
            &format!("{from_first} + {from_second} = {}", from_first + from_second),
            28.0,
            Palette::ink(),
            under(0, 2),
        )?;
        fade_in_all(stage, &[working], 0.5)?;
        line.wait_end(stage)?;

        // 03_7: the cards go; each cell shows its dots, its percent and its
        // blue: 40, 30, 20, 10
        let line = say(&kit, stage, LINES[6])?;
        line.wait_for(stage, P_COUNT)?;
        let mut leaving = cards_in.clone();
        for c in &cards {
            leaving.extend([c.weight]);
            leaving.extend(c.shades);
            leaving.extend(c.labels);
        }
        leaving.extend([answer_label, working]);
        fade_out_all(stage, &leaving, 0.7)?;
        // a percent is recounted every frame, so it comes in as it is (a
        // fade cannot reach a picture redrawn each frame); the readout is
        // detached until it is put on the stage (the narrated draft's 03_7
        // showed the counts alone: nothing had added the percents)
        for c in 0..4 {
            let readout = sea_map.percent_readout(&kit, stage, c, 28.0, map(), under(c, 1));
            stage.add_to_scene(readout)?;
        }
        line.wait_end(stage)?;
        kit.hold(stage)?;
        let third = lower_third(&kit, stage, PRIOR, PRIOR_WORDS)?;
        fade_in_all(stage, &[third], 0.5)?;
        pause(stage, CARRY_HOLD_S - 0.5)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::SEA_DOT_R;
    use super::*;
    use crate::model::Q;
    use explainer_kit::Script;

    /// (line, phrase): every phrase a picture lands on.
    const PHRASES: [(&str, &str); 13] = [
        ("03_2", P_CELLS),
        ("03_2", P_STORIES),
        ("03_3", P_A),
        ("03_3", P_B),
        ("03_3", P_C),
        ("03_3", P_STRAYED),
        ("03_4", P_WEIGH),
        ("03_4", P_PICTURE),
        ("03_4", P_FIRST),
        ("03_4", P_SECOND),
        ("03_6", P_FALL),
        ("03_6", P_SUM),
        ("03_7", P_COUNT),
    ];

    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("03_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// Every phrase a picture lands on is in the line it is keyed to
    /// (`Spoken::at` fails a render on a phrase that is not).
    #[test]
    fn every_phrase_the_pictures_land_on_is_in_its_line() {
        let script = Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in PHRASES {
            let text = script.text(id).unwrap();
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The numbers on the cards and in the cells are the facts: the
    /// weights, each story's shares and dots, the map, the quiz's answer.
    #[test]
    fn what_the_cards_and_cells_show_is_what_the_facts_say() {
        let f = crate::facts::facts().unwrap();
        let stories = sea::stories();
        assert_eq!(f.get("F-M.story1.weight"), Some(stories[0].weight.percent(0).as_str()));
        assert_eq!(f.get("F-M.story1.weight"), Some("60"));
        assert_eq!(f.get("F-M.story2.weight"), Some("40"));
        assert_eq!(f.get("F-M.story1.share"), Some("1/2,1/3,1/6,0"));
        assert_eq!(f.get("F-M.story2.share"), Some("1/4,1/4,1/4,1/4"));
        assert_eq!(f.get("F-M.story1.dots"), Some("30,20,10,0"));
        assert_eq!(f.get("F-M.story2.dots"), Some("10,10,10,10"));
        assert_eq!(f.get("F-M.prior.percent"), Some("40,30,20,10"));
        assert_eq!(f.get("F-Q.stories.answer"), Some("40"));
        // the dots the chapter flies are those counts
        let split = sea_split();
        let story: Vec<String> = split[0].iter().map(ToString::to_string).collect();
        assert_eq!(story.join(","), "30,20,10,0");
        assert_eq!(split[1], [10; 4]);
        // each card's number in a tiny cell is that story's share
        let labels: Vec<String> = stories[0].share.iter().map(|&q| frac_tex(q)).collect();
        assert_eq!(labels, ["\\frac{1}{2}", "\\frac{1}{3}", "\\frac{1}{6}", "0"]);
        let evenly: Vec<String> = stories[1].share.iter().map(|&q| frac_tex(q)).collect();
        assert_eq!(evenly, ["\\frac{1}{4}"; 4]);
    }

    /// 03_3: "most likely in cell A, maybe in B, less likely in C" is the
    /// first story's order, strictly, and the second story has no favourite.
    #[test]
    fn the_first_story_ranks_the_cells_as_the_line_says_and_the_second_does_not() {
        let [first, second] = sea::stories();
        assert!(first.share[0] > first.share[1] && first.share[1] > first.share[2] && first.share[2] > Q::ZERO);
        assert_eq!(first.share[3], Q::ZERO);
        assert!(second.share.iter().all(|&s| s == second.share[0]));
        // and each story's shares add to one whole sea
        for s in [first, second] {
            assert_eq!(s.share.iter().fold(Q::ZERO, |a, &b| a + b), Q::ONE);
        }
    }

    /// The tiny maps and the cards sit on the right of the stage, clear of
    /// the sea and inside the frame; hanging dots clear the tag.
    #[test]
    fn the_cards_sit_right_of_the_sea_inside_the_frame() {
        for &x in &CARD_X {
            assert!(x - CARD_SIZE.0 / 2.0 > sea_rect(1).2, "the card clears the sea");
            assert!(x + CARD_SIZE.0 / 2.0 < super::super::draw::QUIZ_FRAME.0);
        }
        assert!(
            CARD_X[0] + CARD_SIZE.0 / 2.0 < CARD_X[1] - CARD_SIZE.0 / 2.0,
            "the cards do not touch"
        );
        // the hanging rows (two of five dots) stay under the frame's top
        let (_, _, _, top) = sea_rect(0);
        assert!(top + HANG_RISE + HANG_ROW + 0.1 < super::super::draw::QUIZ_FRAME.1);
        // and over the cell's own outer half, inside the cell's width
        for cell in 0..4 {
            let (x0, _, x1, _) = sea_rect(cell);
            let mid = (x0 + x1) / 2.0;
            for j in 0..10 {
                let at = hang_at(cell, j);
                assert!(
                    at[0] > x0 + SEA_DOT_R && at[0] < x1 - SEA_DOT_R,
                    "dot {j} of {cell} at {}",
                    at[0]
                );
                let outer = if cell.is_multiple_of(2) {
                    at[0] < mid
                } else {
                    at[0] > mid
                };
                assert!(outer, "dot {j} of cell {cell} is not on the outer half");
            }
        }
    }

    /// 03_5: the quiz's answer "40%", landing on the line of A's percent,
    /// stands clear of every hanging dot of every cell (it stood on the
    /// bottom row's second row of five, in the dots' own colour: "40%" was
    /// not readable until they fell, two seconds on), and of A's count over
    /// it.
    #[test]
    fn the_answer_clears_the_hanging_dots() {
        use super::super::draw::CHAR_UNITS_PER_POINT as UNITS;
        let answer = format!("{}%", crate::facts::facts().unwrap().get("F-Q.stories.answer").unwrap());
        let at = under(0, 1);
        let (hw, hh) = (answer.chars().count() as f64 * 32.0 * UNITS / 2.0, 0.14);
        for cell in 0..4 {
            for j in 0..10 {
                let d = hang_at(cell, j);
                let dx = (d[0] - at[0]).abs() - hw;
                let dy = (d[1] - at[1]).abs() - hh;
                assert!(
                    dx.max(dy) > SEA_DOT_R + 0.04,
                    "dot {j} of cell {cell} at {d:?} is on the answer at {at:?}"
                );
            }
        }
        // the answer's line is under A's count line, not on it
        assert!(under(0, 0)[1] - hh > at[1] + hh, "the count touches the answer");
    }

    /// 03_5's countdown ring (the frame's top right corner, over whatever is
    /// there) is clear of both story cards and what they hold: it crossed
    /// "weight 40" ("weight 4" and a ring through the 0).
    #[test]
    fn the_countdown_ring_is_clear_of_the_story_cards() {
        use super::super::draw::{QUIZ_RING_AT, QUIZ_RING_R};
        for &x in &CARD_X {
            // the nearest point of the card to the ring's centre
            let (cx, cy) = (QUIZ_RING_AT[0], QUIZ_RING_AT[1]);
            let nx = cx.clamp(x - CARD_SIZE.0 / 2.0, x + CARD_SIZE.0 / 2.0);
            let ny = cy.clamp(CARD_Y - CARD_SIZE.1 / 2.0, CARD_Y + CARD_SIZE.1 / 2.0);
            let gap = (cx - nx).hypot(cy - ny) - QUIZ_RING_R;
            assert!(gap >= 0.08, "the ring is {gap} from the card at {x}");
        }
        // the cards still stand level with the sea's rows, not below them
        let (_, bottom, _, _) = sea_rect(2);
        assert!(
            CARD_Y - CARD_SIZE.1 / 2.0 > bottom - 0.3,
            "the cards hang under the sea"
        );
    }

    /// The count, the percent and the sum under a cell do not touch: a
    /// line's digits are about 0.27 tall at 30 pt, and the lowest line of the
    /// top row stays above the bottom row's cells.
    #[test]
    fn the_three_lines_under_a_cell_clear_each_other_and_the_cell_below() {
        let digit_h = 30.0 * 0.009;
        for pair in UNDER_STEP.windows(2) {
            assert!(pair[1] - pair[0] > digit_h + 0.03, "{pair:?}");
        }
        for cell in 0..2 {
            let lowest = under(cell, 2)[1] - 0.13;
            assert!(
                lowest > sea_rect(cell + 2).3,
                "line three of cell {cell} touches the cell below"
            );
        }
        // the bottom row's sums stay above the question's band
        assert!(under(2, 2)[1] - 0.13 > super::super::draw::QUIZ_LINES_Y[0] + 0.2);
    }

    /// 03_1 (blank_stretches on the scored draft: no picture for 2.4 s): the
    /// Navy's grid stays under the whole line and leaves as 03_2 begins, and
    /// the bare stage before the made-up sea's tag is the authored beat
    /// (BEAT_S), not the rest of the line and the gap after it. The line is
    /// placed on the finished read's onsets (4.19 s into a 7.47 s line the
    /// second sentence, "a map that gives every place its chance", begins).
    #[test]
    fn the_grid_leaves_with_the_next_line_and_the_bare_stage_is_a_beat() {
        use super::super::draw::watch;
        let w = watch::run_timed("03_a_map_of_chances", &[("03_1", &[0.0, 4.19], 7.47)]);
        // the first fade-out is the grid's and the first fade-in the made-up sea's tag
        let out = w.fades.iter().find(|f| !f.arriving).expect("the grid leaves");
        let tag = w
            .fades
            .iter()
            .find(|f| f.arriving)
            .expect("the made-up sea's tag comes");
        let next = w.line_start("03_2");
        let bare = tag.from - (out.from + out.seconds);
        eprintln!(
            "03_1: the grid leaves at {:.2} s, 03_2 starts {next:.2} s, bare stage {bare:.2} s",
            out.from
        );
        assert!(
            out.from >= next - 1e-6,
            "the grid leaves at {:.2} s, before 03_2 starts at {next:.2} s",
            out.from
        );
        assert!(
            (0.0..=0.5).contains(&bare),
            "{bare:.2} s of bare stage between the grid and the sea"
        );
    }
}
