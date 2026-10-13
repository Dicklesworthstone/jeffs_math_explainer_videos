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

// the phrases the pictures land on (a test finds each in its line)
const P_NATURAL: &str = "The natural answer";
const P_MAP: &str = "So you need a map";
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
const CARD_Y: f64 = 1.65;
const TINY: (f64, f64) = (1.3, 0.85);
const TINY_Y: f64 = 1.65;

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
        let (x0, _, x1, y1) = sea_rect(cell);
        let mid = (x0 + x1) / 2.0;
        for (j, d) in mine.into_iter().enumerate() {
            on_card.push(v(centre[0] + pts[j].0, centre[1] + pts[j].1));
            hanging.push(v(mid + ((j % 5) as f64 - 2.0) * 0.22, y1 + 0.24 + 0.2 * (j / 5) as f64));
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
        // leaves, and the stage stands empty for a beat
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
        line.wait_for(stage, P_NATURAL)?;
        fade_out_all(stage, &on_stage, 0.7)?;
        line.wait_for(stage, P_MAP)?;
        line.wait_end(stage)?;
        pause(stage, 0.3)?;

        // 03_2: four cells draw themselves, two by two, tagged; two story
        // cards beside them
        let line = say(&kit, stage, LINES[1])?;
        let sea_map = Sea::build(&kit, stage, [0; 4])?;
        let cards = [story_card(&kit, stage, 0)?, story_card(&kit, stage, 1)?];
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
        let answer_label = kit.text(stage, &format!("{answer}%"), 40.0, map(), sea_slot(0, 1))?;
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
            sea_slot(0, 2),
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
        // fade cannot reach a picture redrawn each frame)
        for c in 0..4 {
            sea_map.percent_readout(&kit, stage, c, 30.0, map(), sea_slot(c, 1));
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
    use super::super::draw::SEA_CELL;
    use super::*;
    use crate::model::Q;
    use explainer_kit::Script;

    /// (line, phrase): every phrase a picture lands on.
    const PHRASES: [(&str, &str); 15] = [
        ("03_1", P_NATURAL),
        ("03_1", P_MAP),
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
        assert!(top + 0.24 + 0.2 + 0.1 < super::super::draw::QUIZ_FRAME.1);
        assert!(SEA_CELL.0 > 5.0 * 0.22, "five hanging dots span less than a cell");
    }
}
