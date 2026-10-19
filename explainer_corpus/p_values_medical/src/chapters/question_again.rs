//! Chapter 06, the question again: back to the survey, its keyed answer as
//! Gao reprints it, with its attribution and the dispute (06_1); that item
//! was about p > 0.05, this study's p is 0.0207, and "chance alone" leaves
//! out what was counted (06_2); what did our two percent count (06_3, a
//! prediction)? The question with every part named: the assumed world, the
//! randomness, the statistic and how extreme, answered by the world's tail
//! (06_4); this study's one-sided p-value, one example, not a formula for
//! every test (06_5); what it does not say: how much sooner, whether that
//! matters, how uncertain (06_6); and the viewer's question (06_7).
//!
//! The survey's words and numbers are `facts::cited`; the answer is the
//! listed world's tail. No effect size, benefit or posterior is computed.

use explainer_kit::{Frame2, Kit, Palette, Timed, v};
use fmn::prelude::*;

use super::count_tail::{bar_overlays, bracket};
use super::does_not_say::last_picture;
use super::draw::{card, chip, fade_in_all, fade_out_all, line_style, say, survey, think_pause, treated, warn};
use super::five_percent::{COLLECTED, NO_CHOICE, PROGRAMS_TAG, PUBLISHED_IN, outline};
use super::imaginary_world::{hist_bars, study_tag};
use super::same_test::small_fraction;
use crate::facts::cited::{
    COLLECTED_MONTHS, COLLECTED_YEAR, GAO_YEAR, KEYED_CHOICE, KEYED_FOR, KEYED_PERMILLE, KEYED_TEXT, PROGRAMS,
    PUBLISHED_YEAR, RESPONDENTS,
};
use crate::model::{Q, Tail, study, tail_count, tail_members, tail_share};

pub struct TheQuestionAgain {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const PICKED: &str = "who answered picked it";
const THE_ITEM: &str = "the survey's item:";
const OUR_STUDY: &str = "this video's synthetic study:";
const OF_WHAT: &str = "“chance alone”: of what? counted how?";
/// 06_4: the question with its parts named, each tag beside its part.
const TAGS: [&str; 4] = ["assumed world", "randomness", "statistic", "as extreme or more"];
const WORLD: &str = "the treatment changed no patient's recovery";
const RANDOMNESS: &str = "the fair coins tossed again";
const STATISTIC: &str = "pairs favouring the treatment";
const OR_MORE: &str = "or more";
const HOW_OFTEN: &str = "how often";
const EVERY: &str = "every assignment, equally likely";
const AXIS: &str = "pairs favouring";
const NAME: &str = "this study's one-sided p-value";
const ONE_EXAMPLE: &str = "one example: other tests assume other worlds and count other things";
/// 06_6: what the p-value does not say.
const OPEN: [&str; 3] = [
    "how much sooner?",
    "does it matter to a patient?",
    "how uncertain is the difference?",
];
/// 06_7: the viewer's question.
const ASK: &str = "when you meet a p-value, ask";
const ASKS: [&str; 3] = [
    "what world did it assume?",
    "what did it count?",
    "how big is the difference?",
];

// --- layout (scene units; the frame is 14.2 by 8)

/// 06_1: the survey's tag lines (chapter 01's), the quote card, the share
/// that picked it and the dispute under it.
const SURVEY_TAG_Y: [f64; 2] = [3.55, 3.22];
const CARD_AT: Vec3 = [-3.4, 1.15, 0.0];
const CARD_W: f64 = 6.4;
const PICKED_Y: f64 = -0.25;
const DISPUTE_Y: f64 = -1.0;
/// 06_2–06_3: the right column, the two p-values and the question, then
/// the think ring.
const RIGHT_X: f64 = 3.6;
const P_ROWS_Y: [f64; 2] = [1.7, 1.15];
const OF_WHAT_Y: f64 = -0.3;
const ASK_AT: Vec3 = [3.6, -1.45, 0.0];
const ASK_R: f64 = 0.45;
/// 06_4–06_5: the named parts, tags left of them; the answer under them;
/// the small histogram right; the name and the scope under the answer.
const TAG_X: f64 = -6.6;
const PART_X: f64 = -3.75;
const PART_Y: [f64; 4] = [2.3, 1.7, 1.1, 0.5];
const ANSWER_Y: f64 = -0.45;
const ANSWER_X: f64 = -2.3;
const NAME_Y: f64 = -1.3;
const SCOPE_Y: f64 = -1.85;
const MINI: Frame2 = Frame2 {
    center: [4.65, 0.75, 0.0],
    width: 3.9,
    height: 2.0,
    x: (-0.5, 20.5),
    y: (0.0, 1.0),
};
const MINI_TITLE_Y: f64 = 2.1;
const MINI_BRACKET_DY: f64 = 0.3;
/// 06_6: the three open questions, left edges.
const OPEN_X: [f64; 3] = [-6.6, -2.0, 2.6];
const OPEN_Y: f64 = -2.6;
/// 06_7: the closing questions, centred.
const ASK_TITLE_Y: f64 = 1.2;
const ASKS_Y: [f64; 3] = [0.4, -0.3, -1.0];

/// The small histogram's window, scaled to the world's tallest bar.
fn mini_frame(world: &[u64]) -> Frame2 {
    let top = world.iter().copied().max().unwrap_or(1);
    Frame2 {
        y: (0.0, top as f64),
        ..MINI
    }
}

/// The quote card exactly as chapter 01 shows it: choice c verbatim as Gao
/// reprints it, the item it answers, the attribution; detached.
fn quote_card(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let head = format!("choice {KEYED_CHOICE}, the answer marked correct for {KEYED_FOR}:");
    let foot = format!("as reprinted by Gao {GAO_YEAR}");
    card(
        kit,
        stage,
        Some(head.as_str()),
        &format!("“{KEYED_TEXT}”"),
        Some(foot.as_str()),
        (22.0, 46),
        Palette::ink(),
        CARD_W,
        CARD_AT,
    )
}

/// The small histogram: its title, axis and ticks, the world's bars, the
/// lavender tail and its bracket; (furniture and bars, tail), detached.
fn mini_histogram(kit: &Kit, stage: &mut Stage<'_>, world: &[u64], k: u32) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    let frame = mini_frame(world);
    let pairs = world.len() as u32 - 1;
    let title = kit.text(stage, EVERY, 20.0, Palette::dim(), v(frame.center[0], MINI_TITLE_Y))?;
    let axis = polyline(vec![v(frame.left(), frame.bottom()), v(frame.right(), frame.bottom())])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut world_parts = vec![title, stage.arena_mut().add(axis)];
    for j in (0..=pairs).step_by(10) {
        let x = frame.p(f64::from(j), 0.0)[0];
        world_parts.push(kit.text(stage, &j.to_string(), 18.0, Palette::dim(), v(x, frame.bottom() - 0.22))?);
    }
    world_parts.push(kit.text(
        stage,
        AXIS,
        18.0,
        Palette::dim(),
        v(frame.center[0], frame.bottom() - 0.55),
    )?);
    world_parts.push(hist_bars(stage, &frame, world)?);
    let lit = tail_members(pairs, k, Tail::AtLeast);
    let mut tail = bar_overlays(stage, &frame, world, &lit, treated())?;
    let (first, last) = (lit[0], lit[lit.len() - 1]);
    let y = frame.p(f64::from(first), world[first as usize] as f64)[1] + MINI_BRACKET_DY;
    let words = format!("{first} {OR_MORE}");
    tail.extend(bracket(kit, stage, &frame, (first, last), y, &words, treated())?);
    Ok((world_parts, tail))
}

/// Scale every mobject in `mobs` up and back together, `rt` seconds.
fn pulse(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let anim = fmn::animation::indicate(stage.arena_mut(), m, scale, None)?;
        anims.push(stage.prepare(anim.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

impl SceneConstruct for TheQuestionAgain {
    fn name(&self) -> &str {
        "06_the_question_again"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let s = study();
        let world = s.null_world();
        let total: u64 = world.iter().sum();
        let k = s.observed();
        let p = tail_share(&world, k, Tail::AtLeast);
        let rest = last_picture(&kit, stage)?;

        // 06_1: back to the survey: the keyed answer, attributed; most who
        // answered picked it; a later paper disputed it
        let line = say(&kit, stage, "06_1")?;
        fade_out_all(stage, &rest, 0.6)?;
        let tag = [
            kit.text(
                stage,
                &format!("{PUBLISHED_IN} {PUBLISHED_YEAR} · {COLLECTED} {COLLECTED_MONTHS} {COLLECTED_YEAR}"),
                22.0,
                Palette::dim(),
                v(0.0, SURVEY_TAG_Y[0]),
            )?,
            kit.text(
                stage,
                &format!("{PROGRAMS} {PROGRAMS_TAG}"),
                22.0,
                Palette::dim(),
                v(0.0, SURVEY_TAG_Y[1]),
            )?,
        ];
        fade_in_all(stage, &tag, 0.4)?;
        line.wait_for(stage, "picked this answer")?;
        let quote = quote_card(&kit, stage)?;
        let picked = kit.text(
            stage,
            &format!(
                "{}% of the {RESPONDENTS} {PICKED}",
                Q::new(KEYED_PERMILLE, 10).decimal(1)
            ),
            22.0,
            survey(),
            v(CARD_AT[0], PICKED_Y),
        )?;
        fade_in_all(stage, &[quote, picked], 0.6)?;
        line.wait_for(stage, "A later paper")?;
        let dispute = chip(
            &kit,
            stage,
            &format!("Gao {GAO_YEAR}: {NO_CHOICE}"),
            22.0,
            warn(),
            v(CARD_AT[0], DISPUTE_Y),
        )?;
        fade_in_all(stage, &[dispute], 0.5)?;

        // 06_2: that item was about p > 0.05; ours is 0.0207; "chance alone"
        // leaves out of what, and counted how
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, "written for a p-value")?;
        let item = kit.text(
            stage,
            &format!("{THE_ITEM} {KEYED_FOR}"),
            22.0,
            Palette::ink(),
            v(RIGHT_X, P_ROWS_Y[0]),
        )?;
        let ours = kit.text(
            stage,
            &format!("{OUR_STUDY} p ≈ {}", p.decimal(4)),
            22.0,
            treated(),
            v(RIGHT_X, P_ROWS_Y[1]),
        )?;
        fade_in_all(stage, &[item, ours], 0.5)?;
        line.wait_for(stage, "chance alone")?;
        let of_what = kit.text(stage, OF_WHAT, 26.0, Palette::hero(), v(RIGHT_X, OF_WHAT_Y))?;
        fade_in_all(stage, &[of_what], 0.5)?;

        // 06_3: what did our two percent actually count?
        say(&kit, stage, "06_3")?;
        think_pause(&kit, stage, ASK_AT, ASK_R)?;

        // 06_4: the question with every part named, and its answer, the tail
        let line = say(&kit, stage, "06_4")?;
        let mut survey_parts = tag.to_vec();
        survey_parts.extend([quote, picked, dispute, item, ours, of_what]);
        fade_out_all(stage, &survey_parts, 0.5)?;
        let study_tag = study_tag(&kit, stage, s.pairs)?;
        let mut tags = Vec::new();
        for (name, y) in TAGS.iter().zip(PART_Y) {
            tags.push(kit.text_left(stage, name, 20.0, Palette::dim(), v(TAG_X, y))?);
        }
        let parts = [
            kit.text_left(stage, WORLD, 22.0, Palette::ink(), v(PART_X, PART_Y[0]))?,
            kit.text_left(stage, RANDOMNESS, 22.0, Palette::ink(), v(PART_X, PART_Y[1]))?,
            kit.text_left(stage, STATISTIC, 22.0, treated(), v(PART_X, PART_Y[2]))?,
            kit.text_left(stage, &format!("{k} {OR_MORE}"), 22.0, treated(), v(PART_X, PART_Y[3]))?,
        ];
        let w = stage.get_width(parts[0]);
        let world_box = outline(
            stage,
            (PART_X - 0.12, PART_Y[0] + 0.24),
            (PART_X + w + 0.12, PART_Y[0] - 0.24),
        )?;
        kit.allow_crossing(world_box);
        let (mini_world, mini_tail) = mini_histogram(&kit, stage, &world, k)?;
        let mut first_part = vec![study_tag, tags[0], parts[0], world_box];
        first_part.extend(&mini_world);
        fade_in_all(stage, &first_part, 0.5)?;
        line.wait_for(stage, "the coins were tossed again")?;
        fade_in_all(stage, &[tags[1], parts[1]], 0.4)?;
        line.wait_for(stage, "fifteen or more pairs would favour it")?;
        let mut tail_part = vec![tags[2], parts[2], tags[3], parts[3]];
        tail_part.extend(&mini_tail);
        fade_in_all(stage, &tail_part, 0.6)?;
        line.wait_for(stage, "about two percent")?;
        let how_often = kit.text_left(
            stage,
            &format!("{HOW_OFTEN}?"),
            22.0,
            Palette::ink(),
            v(TAG_X, ANSWER_Y),
        )?;
        let answer = small_fraction(
            &kit,
            stage,
            (tail_count(&world, k, Tail::AtLeast), total),
            treated(),
            &format!("≈ {}%", p.percent(2)),
            (ANSWER_X, ANSWER_Y),
        )?;
        fade_in_all(stage, &[how_often, answer], 0.6)?;

        // 06_5: this study's one-sided p-value: a world assumed, a tail counted
        let line = say(&kit, stage, "06_5")?;
        line.wait_for(stage, "one-sided p-value")?;
        let name = kit.text(stage, NAME, 24.0, treated(), v(ANSWER_X, NAME_Y))?;
        let scope = kit.text_left(stage, ONE_EXAMPLE, 20.0, Palette::dim(), v(TAG_X, SCOPE_Y))?;
        fade_in_all(stage, &[name, scope], 0.4)?;
        line.wait_for(stage, "A world assumed")?;
        pulse(stage, &[world_box, parts[0]], 1.06, 0.5)?;
        line.wait_for(stage, "a tail counted")?;
        pulse(stage, &mini_tail, 1.06, 0.5)?;

        // 06_6: what it does not say
        let line = say(&kit, stage, "06_6")?;
        let mut open = Vec::new();
        for ((words, x), phrase) in
            OPEN.iter()
                .zip(OPEN_X)
                .zip(["how much sooner", "whether that matters", "how uncertain"])
        {
            line.wait_for(stage, phrase)?;
            let m = kit.text_left(stage, words, 22.0, Palette::ink(), v(x, OPEN_Y))?;
            fade_in_all(stage, &[m], 0.4)?;
            open.push(m);
        }

        // 06_7: the viewer's question
        let line = say(&kit, stage, "06_7")?;
        let mut all = vec![study_tag, world_box, how_often, answer, name, scope];
        all.extend(tags);
        all.extend(parts);
        all.extend(&mini_world);
        all.extend(&mini_tail);
        all.extend(&open);
        fade_out_all(stage, &all, 0.5)?;
        let title = kit.text(stage, &format!("{ASK}:"), 28.0, Palette::hero(), v(0.0, ASK_TITLE_Y))?;
        fade_in_all(stage, &[title], 0.4)?;
        for ((words, y), phrase) in
            ASKS.iter()
                .zip(ASKS_Y)
                .zip(["what world it assumed", "what it counted", "how big the difference is"])
        {
            line.wait_for(stage, phrase)?;
            let m = kit.text(stage, words, 26.0, Palette::ink(), v(0.0, y))?;
            fade_in_all(stage, &[m], 0.4)?;
        }
        kit.hold(stage)?;
        kit.music_breath(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The question is conditioned the way the study was counted: the world
    /// and the randomness are the condition, the statistic and how extreme
    /// are what is counted, and the answer is the world's tail. The data are
    /// never the condition, and no part claims a probability of the world.
    #[test]
    fn the_question_names_its_world_randomness_statistic_and_tail() {
        let s = study();
        let world = s.null_world();
        let k = s.observed();
        assert_eq!(TAGS, ["assumed world", "randomness", "statistic", "as extreme or more"]);
        assert!(WORLD.contains("changed no patient's recovery"));
        assert!(RANDOMNESS.contains("coins tossed again"));
        assert_eq!(STATISTIC, "pairs favouring the treatment");
        assert_eq!(format!("{k} {OR_MORE}"), "15 or more");
        // the condition carries no data; the counted part carries no world
        for condition in [WORLD, RANDOMNESS] {
            assert!(!condition.chars().any(|c| c.is_ascii_digit()), "{condition}");
            assert!(!condition.contains("favour"), "{condition}");
        }
        for words in [WORLD, RANDOMNESS, STATISTIC, NAME, ONE_EXAMPLE] {
            for claim in ["given", "probability that", "chance the treatment", "is true", "works"] {
                assert!(!words.contains(claim), "{words:?} says {claim:?}");
            }
        }
        let p = tail_share(&world, k, Tail::AtLeast);
        assert_eq!(p, Q::ratio(21_700, 1 << 20));
        assert_eq!((p.percent(2), p.decimal(4)), ("2.07".to_owned(), "0.0207".to_owned()));
        assert!(
            ONE_EXAMPLE.starts_with("one example"),
            "not a formula for every p-value"
        );
    }

    /// The survey comes back as chapter 01 showed it: choice c verbatim as
    /// Gao reprints it, the item it answers (p > 0.05, not this study's
    /// 0.0207), the regional sample, most of whom picked the key, and the
    /// dispute; no word turns it into doctors in general.
    #[test]
    fn the_survey_is_quoted_and_attributed_as_reprinted() {
        assert!(KEYED_TEXT.ends_with("could occur by chance alone."));
        assert!(OF_WHAT.contains("chance alone"));
        assert_eq!(KEYED_FOR, "p > 0.05");
        assert_eq!((PROGRAMS, RESPONDENTS), (11, 277));
        const {
            assert!(COLLECTED_YEAR < PUBLISHED_YEAR);
            assert!(
                KEYED_PERMILLE > 500 && KEYED_PERMILLE < 1000,
                "most, not all, picked the key"
            );
        }
        let world = study().null_world();
        let ours = tail_share(&world, study().observed(), Tail::AtLeast);
        assert!(ours < Q::new(5, 100), "this study's p is not the item's p > 0.05");
        let words = [PICKED, THE_ITEM, OUR_STUDY, OF_WHAT, NAME, ONE_EXAMPLE, ASK]
            .iter()
            .chain(OPEN.iter())
            .chain(ASKS.iter())
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>();
        for w in &words {
            assert!(!w.contains("doctor") && !w.contains("wrong"), "{w}");
        }
        assert!(OUR_STUDY.contains("synthetic"));
    }

    /// The layout: the survey card, its share and the dispute stack on the
    /// left; the right column clears the card; the named parts, the answer,
    /// the name and the scope stack left of the small histogram; the open
    /// questions fit across the foot.
    #[test]
    fn the_pieces_stack_without_collisions() {
        let half = |size: f64| 0.0068 * size;
        let width = |s: &str, size: f64| s.chars().count() as f64 * 0.0058 * size;
        const FRACTION_HALF: f64 = 0.45;
        // the card: four lines at most of 22 points, a head and a foot
        let card_h = 0.4 + 0.319 * 3.0 + 0.68;
        assert!(CARD_AT[1] + card_h / 2.0 < SURVEY_TAG_Y[1] - half(22.0) - 0.1);
        assert!(CARD_AT[1] - card_h / 2.0 > PICKED_Y + half(22.0) + 0.05);
        assert!(PICKED_Y - half(22.0) > DISPUTE_Y + half(22.0) + 0.12 + 0.05);
        const {
            assert!(CARD_AT[0] - CARD_W / 2.0 > -7.1 + 0.15);
        }
        let card_right = CARD_AT[0] + CARD_W / 2.0;
        for (words, size) in [("this video's synthetic study: p ≈ 0.0207", 22.0), (OF_WHAT, 26.0)] {
            assert!(RIGHT_X - width(words, size) / 2.0 > card_right + 0.1, "{words}");
            assert!(RIGHT_X + width(words, size) / 2.0 < 7.1 - 0.25, "{words}");
        }
        assert!(ASK_AT[0] - ASK_R > card_right && ASK_AT[1] + ASK_R < OF_WHAT_Y - half(26.0));
        // the named parts and the small histogram
        let world = study().null_world();
        let mini = mini_frame(&world);
        assert!(PART_X + width(WORLD, 22.0) + 0.12 < mini.left() - 0.3);
        assert!(TAG_X + width("as extreme or more", 20.0) < PART_X - 0.15);
        for pair in PART_Y.windows(2) {
            assert!(pair[0] - pair[1] > 2.0 * half(22.0) + 0.1);
        }
        assert!(PART_Y[3] - half(22.0) > ANSWER_Y + FRACTION_HALF + 0.05);
        assert!(ANSWER_Y - FRACTION_HALF > NAME_Y + half(24.0) + 0.05);
        assert!(NAME_Y - half(24.0) > SCOPE_Y + half(20.0) + 0.05);
        assert!(SCOPE_Y - half(20.0) > OPEN_Y + half(22.0) + 0.1);
        assert!(TAG_X + width(ONE_EXAMPLE, 20.0) < mini.left() - 0.3);
        assert!(mini.right() < 7.1 - 0.25 && mini.top() < MINI_TITLE_Y - half(20.0));
        assert!(MINI_TITLE_Y + half(20.0) < 3.2 - half(22.0));
        assert!(mini.bottom() - 0.55 - half(18.0) > SCOPE_Y);
        for i in 0..2 {
            assert!(OPEN_X[i] + width(OPEN[i], 22.0) < OPEN_X[i + 1] - 0.2, "{i}");
        }
        assert!(OPEN_X[2] + width(OPEN[2], 22.0) < 7.1 - 0.25);
        assert!(OPEN_Y - half(22.0) > -4.0 + 0.25);
        assert!(width(ASK, 28.0) / 2.0 < 7.1 - 0.25);
        assert!(ASK_TITLE_Y - ASKS_Y[0] > half(28.0) + half(26.0));
    }
}
