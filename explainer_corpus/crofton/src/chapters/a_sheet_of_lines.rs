//! Chapter 02, a sheet of lines. A string pressed into the bends would
//! work but is slow; it stops halfway round the headland and is never
//! straightened (02_1). Ruler, dividers and string all follow the coast
//! (02_2). A clear sheet ruled every 2 cm slides in and settles at the
//! shown placement (02_3). A crossing is where the coast meets a line: one
//! in the cove, one on the climb, each dotted and numbered (02_4). How many?
//! (02_5, a prediction, with time to count.) Seven: the other five light and
//! are numbered in the order the coast meets them (02_6). Only two lines
//! meet the coast, the lower four times and the upper three, each crossing
//! pulsing as it is named (02_7). The chapter ends on the card, the coast,
//! the sheet, the seven dots and "7 crossings", where chapter 03 begins.
//! Integrated by root from the preserved candidate; qualification evidence is external.

use std::f64::consts::{FRAC_PI_4, PI};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    EVERY, card_point, dim_line, dot, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, line_style, now, path,
    pause, say, stand, stop, swap, think, together, traced, wait_until,
};
use super::sheet::{
    self, COUNT_AT, COUNT_SIZE, Crossing, LABEL_SIZE, LINE_PICKED_W, LINE_W, Sheet, THINK_AT, TOP_Y, count_words,
    crossing_dot, gap_bracket, gap_label_at,
};
use crate::facts;
use crate::model::{self, CARD_W, GAP, HEADLAND, P2, PIECE_NAMES};
use crate::palette;

pub struct ASheetOfLines {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const TOOLS: [&str; 3] = ["ruler", "dividers", "string"];
const DEFINITION: &str = "crossing: where the coast meets a line";
const DEFINED: &str = "crossing";
const QUESTION: &str = "How many crossings?";
/// 02_4 marks the first two crossings the coast meets on these words, and
/// says which pieces they are on.
const FIRST_TWO: [(&str, &str); 2] = [("Here's one", "cove"), ("here's another", "climb")];
/// 02_7 names each line's crossings, in the order the coast meets them, by
/// these phrases and the pieces they are on: the lower line, then the upper.
const LOWER: [(&str, &[&str]); 3] = [
    ("in the cove", &["cove"]),
    ("on the climb", &["climb"]),
    ("twice round the bay", &["bay", "bay"]),
];
const UPPER: [(&str, &[&str]); 2] = [
    ("either side of the headland", &["rise", "fall"]),
    ("once near the end", &["end"]),
];

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
/// The string pressed into the bends, thinner than the coast so the coast
/// shows on either side of it.
const STRING_W: f64 = 2.5;
/// 02_2's three tools, small, in the empty band over the coast, frame
/// units, with their names under them.
const ICON_Y: f64 = 3.15;
const ICON_XS: [f64; 3] = [-2.4, 0.0, 2.4];
const ICON_HALF: f64 = 0.55;
const ICON_NAME_Y: f64 = 2.55;
const ICON_NAME_SIZE: f64 = 20.0;
/// The beads that follow the coast in 02_2, cm apart along it, and their
/// size (frame units).
const BEAD_EVERY: f64 = 0.5;
const BEAD_R: f64 = 0.035;
/// The sheet arrives from beyond the frame's right edge, frame units.
const SLIDE_IN_SHIFT: f64 = 14.0;
const DEFINITION_SIZE: f64 = 26.0;
/// A crossing's number: its size, and how far from its dot, cm.
const NUMBER_SIZE: f64 = 22.0;
const NUMBER_OFF: f64 = 0.5;
/// How far past a line's right end its tally stands, cm.
const TALLY_OFF: f64 = 0.45;
const TALLY_SIZE: f64 = 30.0;

// ------------------------------------------------------------------ timing

const CARD_IN: f64 = 0.4;
/// The string stops at each joint for this long: bend by bend.
const BEND_BEAT: f64 = 0.25;
/// The string's pressing never runs faster than this in all, seconds.
const MIN_PRESS: f64 = 2.0;
const FOLLOW_S: f64 = 2.4;
const SLIDE_IN_S: f64 = 1.6;
/// A crossing's dot grows and shrinks back once, this long in all.
const PULSE: f64 = 1.8;
const PULSE_S: f64 = 0.4;
/// Each of 02_6's five crossings has this long to itself.
const REVEAL_S: f64 = 0.75;
/// The lines that meet nothing, while 02_7 counts the two that do.
const UNUSED_LINES: f64 = 0.35;

/// The chapter's words that carry numbers, each from the model or the
/// facts: the gap, the count, each crossing's number, and each crossed
/// line's tally, lowest first. Refused unless the sheet's gap is the facts'.
struct Words {
    gap: String,
    count: String,
    numbers: Vec<String>,
    tallies: Vec<String>,
}

fn words(found: &[Crossing]) -> Result<Words, String> {
    let f = facts::facts()?;
    let gap = f.get("sheet.gap").ok_or("no fact sheet.gap")?.to_owned();
    if gap != facts::fixed(GAP, 0) {
        return Err(format!("the sheet is drawn {GAP} cm apart, the facts say {gap}"));
    }
    Ok(Words {
        gap: format!("{gap} cm"),
        count: count_words(found.len()),
        numbers: (1..=found.len()).map(|k| k.to_string()).collect(),
        tallies: sheet::per_line(found).into_iter().map(|(_, n)| n.to_string()).collect(),
    })
}

/// The shown sheet's crossings, refused unless they are the ones the
/// chapter's words name: the first two on the cove and the climb, and two
/// lines crossed, holding the pieces 02_7 names in the order it names them.
fn named_crossings(coast: &model::Curve, slide: f64) -> Result<Vec<Crossing>, String> {
    let found = sheet::crossings(coast, 0.0, slide);
    for ((_, piece), c) in FIRST_TWO.iter().zip(&found) {
        if PIECE_NAMES[c.piece] != *piece {
            return Err(format!(
                "02_4 names the {piece}, the crossing is on the {}",
                PIECE_NAMES[c.piece]
            ));
        }
    }
    let lines = sheet::per_line(&found);
    if lines.len() != 2 {
        return Err(format!("02_7 names two lines, the coast meets {}", lines.len()));
    }
    for ((line, _), named) in lines.iter().zip([&LOWER[..], &UPPER[..]]) {
        let on: Vec<&str> = found
            .iter()
            .filter(|c| c.line == *line)
            .map(|c| PIECE_NAMES[c.piece])
            .collect();
        let said: Vec<&str> = named.iter().flat_map(|(_, pieces)| pieces.iter().copied()).collect();
        if on != said {
            return Err(format!("02_7 names {said:?} on a line that holds {on:?}"));
        }
    }
    Ok(found)
}

fn dist(a: P2, b: P2) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// How far `p` is from the polyline through `points`.
fn to_chords(points: &[P2], p: P2) -> f64 {
    points
        .windows(2)
        .map(|w| {
            let (a, d) = (w[0], [w[1][0] - w[0][0], w[1][1] - w[0][1]]);
            let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / (d[0] * d[0] + d[1] * d[1])).clamp(0.0, 1.0);
            dist([a[0] + t * d[0], a[1] + t * d[1]], p)
        })
        .fold(f64::INFINITY, f64::min)
}

/// How much room a number at `p` has: its distance from the coast or the
/// nearest line, whichever is less, cm.
fn room(coast: &[P2], lines: &[f64], p: P2) -> f64 {
    let to_lines = lines.iter().map(|&h| (p[1] - h).abs()).fold(f64::INFINITY, f64::min);
    to_chords(coast, p).min(to_lines)
}

/// Where a crossing's number goes, cm: of eight places [`NUMBER_OFF`]
/// round its dot, the one with the most room (the last of equals).
fn number_at(coast: &[P2], lines: &[f64], at: P2) -> P2 {
    (0..8)
        .map(|k| {
            let a = FRAC_PI_4 * k as f64;
            [at[0] + NUMBER_OFF * a.cos(), at[1] + NUMBER_OFF * a.sin()]
        })
        .max_by(|p, q| room(coast, lines, *p).total_cmp(&room(coast, lines, *q)))
        .unwrap_or(at)
}

/// Where 02_1's string stops, cm along the coast: halfway round the
/// headland. It is pressed piece by piece up to there.
fn string_stops(coast: &model::Curve) -> Vec<f64> {
    let head = coast.parts[HEADLAND];
    let stop_at = head.start + head.seg.len() / 2.0;
    coast.parts[..=HEADLAND].iter().map(|p| p.end().min(stop_at)).collect()
}

/// 02_2's three tools, small: the ruler, the dividers' legs, a loose
/// string. Detached.
fn tool_icons(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let [rx, dx, sx] = ICON_XS;
    let ruler = Rectangle::new()
        .width(2.0 * ICON_HALF)
        .height(0.22)
        .corner_radius(0.03)
        .style(flat(Palette::ink(), 0.8))
        .build()?
        .moved_to(v(rx, ICON_Y));
    let ruler = stage.arena_mut().add(ruler);
    let legs = polyline(vec![
        v(dx - 0.32, ICON_Y - 0.3),
        v(dx, ICON_Y + 0.3),
        v(dx + 0.32, ICON_Y - 0.3),
    ])
    .style(line_style(palette::dividers(), 3.5))
    .build();
    let legs = stage.arena_mut().add(legs);
    let wave: Vec<Vec3> = (0..=32)
        .map(|i| {
            let u = f64::from(i) / 32.0;
            v(
                sx - ICON_HALF + 2.0 * ICON_HALF * u,
                ICON_Y + 0.12 * (3.0 * PI * u).sin(),
            )
        })
        .collect();
    let string = stage
        .arena_mut()
        .add(polyline(wave).style(line_style(Palette::ink(), STRING_W)).build());
    Ok(vec![ruler, legs, string])
}

/// Where a line's tally stands: past its right end, cm.
fn tally_at(height: f64) -> P2 {
    [CARD_W / 2.0 - sheet::LINE_INSET + TALLY_OFF, height]
}

/// Dots grown and shrunk back together, once.
fn pulse(stage: &mut Stage<'_>, dots: &[Mob]) -> fmn::Result<()> {
    for factor in [PULSE, 1.0 / PULSE] {
        let mut moves = Vec::with_capacity(dots.len());
        for &d in dots {
            moves.push(
                d.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(PULSE_S / 2.0),
                        ..AnimateArgs::default()
                    })?
                    .scale(factor)?,
            );
        }
        together(stage, moves)?;
    }
    Ok(())
}

/// A sheet line restroked (its fill stays empty), `rt` seconds.
fn restroke(line: Mob, color: Srgb, width: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(line
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_stroke(Some(color), Some(width), Some(1.0))?)
}

impl SceneConstruct for ASheetOfLines {
    fn name(&self) -> &str {
        "02_a_sheet_of_lines"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let found = named_crossings(&coast, slide).map_err(fail)?;
        let words = words(&found).map_err(fail)?;
        let heights = sheet::lines_on_card(slide);
        let drawn = coast.upto(coast.length, EVERY);
        let stops = string_stops(&coast);

        // where chapter 01 ended: the card and the coast
        let card = super::draw::card_panel(stage)?;
        let shore = path(stage, &drawn, palette::coast(), COAST_W);
        for m in [card, shore] {
            stage.add_to_scene(m)?;
        }

        // 02_1: a string pressed into the bends, piece by piece, stopping
        // halfway round the headland; never straightened
        let line = say(&kit, stage, "02_1")?;
        let (t0, t1) = (line.at("Press it")?, line.at("slow and fiddly")?);
        line.wait_for(stage, "Press it")?;
        let pressed = stage.add_value_tracker(0.0);
        let [pen, string] = traced(stage, &coast, pressed, Palette::ink(), STRING_W)?;
        fade_in_all(stage, &[pen, string], 0.1)?;
        let reach = stops[stops.len() - 1];
        let press = ((t1 - t0) - BEND_BEAT * (stops.len() - 1) as f64 - 0.1).max(MIN_PRESS);
        let mut done = 0.0;
        for (i, &end) in stops.iter().enumerate() {
            stage.play(tween(pressed, end / coast.length, press * (end - done) / reach)?)?;
            done = end;
            if i + 1 < stops.len() {
                pause(stage, BEND_BEAT)?;
            }
        }
        line.wait_for(stage, "slow and fiddly")?;
        stop(stage, pen);
        fade_out_all(stage, &[string], 0.5)?;

        // 02_2: the three tools, and beads following the coast from its
        // start to its end; all go on "without following it at all"
        let line = say(&kit, stage, "02_2")?;
        let mut tools = tool_icons(stage)?;
        for (name, x) in TOOLS.iter().zip(ICON_XS) {
            tools.push(kit.text(stage, name, ICON_NAME_SIZE, Palette::dim(), v(x, ICON_NAME_Y))?);
        }
        fade_in_all(stage, &tools, CARD_IN)?;
        line.wait_for(stage, "follow the coast")?;
        let beads: Vec<Mob> = (0..(coast.length / BEAD_EVERY).floor() as usize)
            .map(|k| dot(stage, coast.at((k as f64 + 0.5) * BEAD_EVERY), BEAD_R, Palette::ink()))
            .collect();
        fade_in_lagged(stage, &beads, FOLLOW_S, 0.15)?;
        line.wait_for(stage, "without following it at all")?;
        tools.extend(&beads);
        fade_out_all(stage, &tools, 0.5)?;

        // 02_3: the clear sheet slides in from the right and settles at the
        // shown placement; the 2 cm bracket
        let line = say(&kit, stage, "02_3")?;
        let ruled = Sheet::new(stage, slide, SLIDE_IN_SHIFT)?;
        line.wait_for(stage, "Take a clear sheet")?;
        let mut moves = Vec::new();
        for m in ruled.all() {
            stage.add_to_scene(m)?;
            moves.push(
                m.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(SLIDE_IN_S),
                        ..AnimateArgs::default()
                    })?
                    .shift(v(-SLIDE_IN_SHIFT, 0.0))?,
            );
        }
        together(stage, moves)?;
        line.wait_for(stage, "two centimetres apart")?;
        let tick = path(stage, &gap_bracket(&heights), Palette::ink(), 2.0);
        let gap = kit.text(
            stage,
            &words.gap,
            LABEL_SIZE,
            Palette::dim(),
            card_point(gap_label_at(&heights)),
        )?;
        fade_in_all(stage, &[tick, gap], 0.4)?;

        // 02_4: what a crossing is, on the first two the coast meets
        let line = say(&kit, stage, "02_4")?;
        fade_out_all(stage, &[tick, gap], 0.3)?;
        line.wait_for(stage, "call that a crossing")?;
        let definition = kit.text_t2c(
            stage,
            DEFINITION,
            DEFINITION_SIZE,
            Palette::ink(),
            &[(DEFINED, palette::touch())],
            v(0.0, TOP_Y),
        )?;
        fade_in_all(stage, &[definition], 0.4)?;
        let mut dots = Vec::with_capacity(found.len());
        let mut numbers = Vec::with_capacity(found.len());
        for c in &found {
            dots.push(crossing_dot(stage, c.at));
            let at = number_at(&drawn, &heights, c.at);
            numbers.push(kit.text(
                stage,
                &words.numbers[numbers.len()],
                NUMBER_SIZE,
                palette::touch(),
                card_point(at),
            )?);
        }
        for (k, (phrase, _)) in FIRST_TWO.iter().enumerate() {
            line.wait_for(stage, phrase)?;
            fade_in_all(stage, &[dots[k], numbers[k]], 0.3)?;
            pulse(stage, &dots[k..=k])?;
        }

        // 02_5: count them; the question, and time to count
        let line = say(&kit, stage, "02_5")?;
        line.wait_for(stage, "How many crossings")?;
        let question = kit.text(stage, QUESTION, COUNT_SIZE, Palette::ink(), COUNT_AT)?;
        swap(stage, &[definition], &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 02_6: seven; the other five light and are numbered in the order
        // the coast meets them, each given its own moment
        let line = say(&kit, stage, "02_6")?;
        let count = kit.text(stage, &words.count, COUNT_SIZE, palette::touch(), COUNT_AT)?;
        swap(stage, &[q, question], &[count], 0.3)?;
        line.wait_for(stage, "Here they are")?;
        for (&d, &n) in dots.iter().zip(&numbers).skip(FIRST_TWO.len()) {
            let slot = now(stage);
            fade_in_all(stage, &[d, n], 0.3)?;
            pulse(stage, &[d])?;
            wait_until(stage, slot + REVEAL_S)?;
        }

        // 02_7: only two lines meet the coast; each is picked out with its
        // tally, and its crossings pulse as they are named
        let line = say(&kit, stage, "02_7")?;
        fade_out_all(stage, &numbers, 0.4)?;
        let crossed = sheet::per_line(&found);
        let picked: Vec<Mob> = crossed
            .iter()
            .map(|&(k, _)| {
                found
                    .iter()
                    .find(|c| c.line == k)
                    .and_then(|c| ruled.line(c))
                    .ok_or_else(|| fail(format!("no drawn line {k}")))
            })
            .collect::<fmn::Result<_>>()?;
        let unused: Vec<Mob> = ruled.lines.iter().copied().filter(|m| !picked.contains(m)).collect();
        line.wait_for(stage, "Only two of the lines")?;
        together(
            stage,
            unused
                .iter()
                .map(|&m| dim_line(m, UNUSED_LINES, 0.5))
                .collect::<fmn::Result<_>>()?,
        )?;
        let mut tallies = Vec::with_capacity(crossed.len());
        for (i, ((k, _), (start, tally, named))) in crossed
            .iter()
            .zip([
                ("The lower one", "four times", &LOWER[..]),
                ("The upper one", "three times", &UPPER[..]),
            ])
            .enumerate()
        {
            line.wait_for(stage, start)?;
            let mut moves = vec![restroke(picked[i], Palette::ink(), LINE_PICKED_W, 0.4)?];
            if i > 0 {
                moves.push(restroke(picked[i - 1], Palette::dim(), LINE_W, 0.4)?);
            }
            together(stage, moves)?;
            line.wait_for(stage, tally)?;
            let height = slide + *k as f64 * GAP;
            let t = kit.text(
                stage,
                &words.tallies[i],
                TALLY_SIZE,
                palette::touch(),
                card_point(tally_at(height)),
            )?;
            fade_in_all(stage, &[t], 0.3)?;
            tallies.push(t);
            let mut on_line = found.iter().enumerate().filter(|(_, c)| c.line == *k).map(|(j, _)| j);
            for (phrase, pieces) in named {
                line.wait_for(stage, phrase)?;
                let these: Vec<Mob> = on_line.by_ref().take(pieces.len()).map(|j| dots[j]).collect();
                pulse(stage, &these)?;
            }
        }
        kit.hold(stage)?;
        let mut back = vec![restroke(picked[picked.len() - 1], Palette::dim(), LINE_W, 0.5)?];
        for &m in &unused {
            back.push(dim_line(m, 1.0, 0.5)?);
        }
        together(stage, back)?;
        fade_out_all(stage, &tallies, 0.4)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT};

    fn shown() -> (model::Curve, f64, Vec<Crossing>) {
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let found = named_crossings(&coast, slide).unwrap();
        (coast, slide, found)
    }

    /// The words on screen say what the model and the facts say: the
    /// oracle's `sheet.gap`, `open.crossings` and `open.per_line`.
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        let (_, _, found) = shown();
        let w = words(&found).unwrap();
        assert_eq!(w.gap, "2 cm");
        assert_eq!(w.count, "7 crossings");
        assert_eq!(w.numbers, ["1", "2", "3", "4", "5", "6", "7"]);
        assert_eq!(w.tallies, ["4", "3"]);
        assert_eq!(count_words(1), "1 crossing");
    }

    /// The true length stays sealed: nothing on screen gives it away, and
    /// the string never covers the coast.
    #[test]
    fn the_true_length_is_not_on_screen() {
        let f = facts::facts().unwrap();
        let (coast, _, found) = shown();
        let w = words(&found).unwrap();
        let mut shown_words = vec![
            w.gap.clone(),
            w.count.clone(),
            DEFINITION.into(),
            QUESTION.into(),
            super::super::draw::PAUSE_NOTE.into(),
        ];
        shown_words.extend(w.numbers.iter().cloned());
        shown_words.extend(w.tallies.iter().cloned());
        shown_words.extend(TOOLS.iter().map(|&t| t.to_owned()));
        for key in ["coast.length", "coast.length_2dp", "coast.length_3dp"] {
            let length = f.get(key).unwrap();
            for text in &shown_words {
                assert!(!text.contains(length), "{text} shows {key}");
            }
        }
        let stops = string_stops(&coast);
        assert!(stops[stops.len() - 1] < coast.length / 2.0);
    }

    /// The string is pressed piece by piece, cove to headland, and stops
    /// halfway round the headland.
    #[test]
    fn the_string_stops_halfway_round_the_headland() {
        let coast = model::coast();
        let stops = string_stops(&coast);
        assert_eq!(stops.len(), HEADLAND + 1);
        for w in stops.windows(2) {
            assert!(w[1] > w[0]);
        }
        let head = coast.parts[HEADLAND];
        assert!((stops[HEADLAND] - (head.start + head.end()) / 2.0).abs() < 1e-12);
        for (i, &s) in stops[..HEADLAND].iter().enumerate() {
            assert!((s - coast.parts[i].end()).abs() < 1e-12);
        }
    }

    /// The crossings the words name are the crossings: 02_4's first two,
    /// and 02_7's, line by line in the order the coast meets them; a wrong
    /// name is refused.
    #[test]
    fn the_named_crossings_are_the_lines_crossings() {
        let (coast, slide, found) = shown();
        assert_eq!(found.len(), 7);
        let names: Vec<&str> = found.iter().map(|c| PIECE_NAMES[c.piece]).collect();
        assert_eq!(names[..2], [FIRST_TWO[0].1, FIRST_TWO[1].1]);
        assert_eq!(
            (PIECE_NAMES[HEADLAND - 1], PIECE_NAMES[HEADLAND + 1]),
            (UPPER[0].1[0], UPPER[0].1[1]),
            "either side of the headland"
        );
        let said: usize = LOWER.iter().chain(&UPPER).map(|(_, p)| p.len()).sum();
        assert_eq!(said, found.len());
        // the same check refuses a placement whose crossings differ
        assert!(named_crossings(&coast, slide + 0.6).is_err());
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let mut waits: Vec<(&str, &str)> = vec![
            ("02_1", "Press it"),
            ("02_1", "slow and fiddly"),
            ("02_2", "follow the coast"),
            ("02_2", "without following it at all"),
            ("02_3", "Take a clear sheet"),
            ("02_3", "two centimetres apart"),
            ("02_4", "call that a crossing"),
            ("02_5", "How many crossings"),
            ("02_6", "Here they are"),
            ("02_7", "Only two of the lines"),
            ("02_7", "The lower one"),
            ("02_7", "four times"),
            ("02_7", "The upper one"),
            ("02_7", "three times"),
        ];
        waits.extend(FIRST_TWO.iter().map(|&(p, _)| ("02_4", p)));
        waits.extend(LOWER.iter().chain(&UPPER).map(|&(p, _)| ("02_7", p)));
        for (id, phrase) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // 02_7's phrases come in the order the coast meets their crossings
        let text = script.text("02_7").unwrap();
        let at: Vec<usize> = LOWER.iter().chain(&UPPER).map(|(p, _)| text.find(p).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[1] > w[0]), "{at:?}");
    }

    /// Each crossing's number keeps clear of the coast, of every line, of
    /// the other dots and of the other numbers, and stays on the card.
    #[test]
    fn the_numbers_keep_clear() {
        let (coast, slide, found) = shown();
        let drawn = coast.upto(coast.length, EVERY);
        let heights = sheet::lines_on_card(slide);
        let at: Vec<P2> = found.iter().map(|c| number_at(&drawn, &heights, c.at)).collect();
        for (i, &p) in at.iter().enumerate() {
            assert!(
                to_chords(&drawn, p) > 0.3,
                "number {} is {:.3} cm from the coast",
                i + 1,
                to_chords(&drawn, p)
            );
            assert!(room(&drawn, &heights, p) > 0.25, "number {}", i + 1);
            for (j, c) in found.iter().enumerate() {
                if j != i {
                    assert!(dist(c.at, p) > 0.45, "number {} near dot {}", i + 1, j + 1);
                    assert!(dist(at[j], p) > 0.45, "numbers {} and {}", i + 1, j + 1);
                }
            }
            assert!(p[0].abs() < CARD_W / 2.0 - 0.3 && p[1].abs() < CARD_H / 2.0 - 0.3);
        }
    }

    /// The layout keeps its rooms: the tools over the coast and under the
    /// card's top (the bracket's room is `sheet`'s test), the tallies
    /// inside the frame and clear of the coast.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let (coast, slide, found) = shown();
        let drawn = coast.upto(coast.length, EVERY);
        let card_top = CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0;
        let coast_top = drawn
            .iter()
            .map(|p| card_point(*p)[1])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(ICON_Y + 0.32 < card_top - 0.1 && ICON_NAME_Y - 0.15 > coast_top + 0.3);
        assert!(ICON_XS.windows(2).all(|w| w[1] - w[0] > 2.0 * ICON_HALF + 0.6));
        for (k, _) in sheet::per_line(&found) {
            let t = tally_at(slide + k as f64 * GAP);
            assert!(to_chords(&drawn, t) > 0.6, "tally at {t:?}");
            assert!(card_point(t)[0] + 0.2 < FRAME_X_RADIUS);
        }
        const {
            assert!(COUNT_AT[1] + 0.2 < FRAME_Y_RADIUS);
        }
    }
}
