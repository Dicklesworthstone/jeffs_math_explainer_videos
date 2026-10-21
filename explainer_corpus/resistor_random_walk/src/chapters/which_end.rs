//! Chapter 01, which end first: five identical wires in a line and a walker
//! one wire above the bottom (01_1); it picks one of its two wires at
//! random and stops at an end (01_2); one route, and without words a ghost
//! whose first step ends it at the bottom (01_3); its chance is the
//! fraction of very many walkers (01_4); its routes never run out, so one
//! step ahead, half the time up and half down (01_5); so the average of the
//! two far ends' chances, halfway between them (01_6); the prediction
//! (01_7); one fifth, equal steps (01_8); its chance of walking for ever
//! keeps the same rule with zero at both ends, so it is zero (01_9).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    LABEL_SIZE, LinePicture, TOKEN_SIZE, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, line_picture,
    line_style, move_all, pause, recolor, say, small_tex, tex, think, wait_until, walker_dot,
};
use crate::facts;
use crate::model::{LINE_GHOST, LINE_ROUTE, LINE_START, Q};
use crate::palette;

pub struct WhichEnd {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const CHANCE: &str = "chance of reaching the top first";
const FOR_EVER: &str = "chance of walking for ever";
const HALFWAY: &str = "halfway between its neighbours";
const UNKNOWN: &str = "?";

/// 01_4's exact version (card A): what a chance is.
const CARD_A: &str = "walkers step independently, and as their number grows, the fraction that reach the top first \
                      tends to this chance (the law of large numbers).";
/// 01_5's exact version (card B): the walker starts afresh (the Markov
/// property).
const CARD_B: &str = "its next steps do not depend on how it got here, so its chance from where it lands is that \
                      junction's own chance.";

/// Where the line stands; the panel's row and note, and the countdown, on
/// the right; the walkers beside junction 1 (01_4); the step chances
/// beside its wires (01_5); the bars of the ramp (01_8).
const LINE_AT: [f64; 2] = [-3.0, 0.45];
const PANEL_X: f64 = 2.6;
const ROW_Y: f64 = 1.6;
const NOTE_Y: f64 = 0.85;
const THINK_AT: [f64; 2] = [2.6, -0.5];
const CROWD: usize = 10;
const CROWD_X0: f64 = 0.45;
const CROWD_STEP: f64 = 0.2;
const CROWD_R: f64 = 0.06;
const STEP_DX: f64 = 0.45;
const BAR_X0: f64 = 0.45;
const BAR_LEN: f64 = 2.5;
const BAR_W: f64 = 10.0;
/// The walker's size and how long one of its steps takes; a ghost's
/// opacity.
const WALKER_R: f64 = 0.11;
const STEP_S: f64 = 0.4;
const GHOST: f64 = 0.4;
/// A lower-third's fade in and out, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// 01_9's exact version (card C), from the facts: the walk from one wire
/// above the bottom stops with chance one, though endless walks exist, and
/// on a line the chance climbs as x over N.
fn stops_card() -> Result<String, String> {
    let row = facts::line()?;
    let walk = row.walk()?;
    let never = row.kirchhoff(Q::ZERO, Q::ZERO)?[LINE_START];
    let stops = walk.top[LINE_START] + walk.bottom[LINE_START];
    if !never.is_zero() || stops != Q::ONE {
        return Err(format!("the line's walk goes on for ever with chance {never} and stops with {stops}"));
    }
    Ok(format!(
        "endless walks exist, but together they have chance {never}, so it stops with chance {stops}. On a line \
         of N wires, x wires up, the chance is x/N."
    ))
}

/// 01_6: junction `x`'s chance, the average of its two far ends'.
fn halfway_row(x: usize, wires: usize) -> String {
    format!(r"p({x}) = \frac{{p({}) + p({})}}{{{wires}}}", x - 1, x + 1)
}

/// Junction `x`'s value on the line, left of it. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &LinePicture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x))
}

/// The walker takes `route` along the line, a step a wire.
fn walk_line(stage: &mut Stage<'_>, pic: &LinePicture, dot: Mob, route: &[usize]) -> fmn::Result<()> {
    for step in route.windows(2) {
        move_all(stage, &[(dot, pic.spot(step[1]))], STEP_S)?;
    }
    Ok(())
}

impl SceneConstruct for WhichEnd {
    fn name(&self) -> &str {
        "01_which_end"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let row = facts::line().map_err(fail)?;
        let walk = row.walk().map_err(fail)?;
        let chances = walk.top.clone();
        let (top, bottom, start) = (row.top(), row.bottom(), LINE_START);
        let wires = row.far_ends(start).map_err(fail)?.len();

        // 01_1: the junctions from the bottom up, the wires, the name; the
        // walker one wire above the bottom
        let line = say(&kit, stage, "01_1")?;
        let colors: Vec<Srgb> = (0..row.size())
            .map(|x| {
                if x == top {
                    palette::top()
                } else if x == bottom {
                    palette::bottom()
                } else {
                    palette::unknown()
                }
            })
            .collect();
        let pic = line_picture(stage, &row, LINE_AT, &colors)?;
        fade_in_lagged(stage, &pic.junctions, 0.8, 0.3)?;
        let mut arriving = pic.wires.clone();
        let mut name = kit.text(stage, CHANCE, 26.0, Palette::dim(), pic.name_spot())?;
        arriving.push(name);
        fade_in_lagged(stage, &arriving, 0.9, 0.2)?;
        line.wait_for(stage, "A walker")?;
        let mut dot = walker_dot(stage, pic.spot(start), WALKER_R, 1.0);
        fade_in_all(stage, &[dot], 0.3)?;

        // 01_2: its two wires; the two ends where it stops
        let line = say(&kit, stage, "01_2")?;
        line.wait_for(stage, "one of its two wires")?;
        flash_all(stage, &pic.wires_at(start), 0.6)?;
        line.wait_for(stage, "it stops")?;
        flash_all(stage, &[pic.junctions[top], pic.junctions[bottom]], 0.6)?;

        // 01_3: one route to the top; then without words a ghost whose first
        // step ends it at the bottom; the walker back where it started
        let line = say(&kit, stage, "01_3")?;
        line.wait_for(stage, "Up, down")?;
        walk_line(stage, &pic, dot, &LINE_ROUTE)?;
        flash_all(stage, &[pic.junctions[top]], 0.5)?;
        let ghost = walker_dot(stage, pic.spot(LINE_GHOST[0]), WALKER_R, GHOST);
        fade_in_all(stage, &[ghost], 0.3)?;
        walk_line(stage, &pic, ghost, &LINE_GHOST)?;
        flash_all(stage, &[pic.junctions[bottom]], 0.5)?;
        let back = walker_dot(stage, pic.spot(start), WALKER_R, 1.0);
        fade_out_all(stage, &[ghost, dot], 0.25)?;
        fade_in_all(stage, &[back], 0.25)?;
        dot = back;

        // 01_4: its chance, a "?" beside it; very many walkers, no count;
        // what a chance is (card A)
        let line = say(&kit, stage, "01_4")?;
        let mut labels: Vec<Option<Mob>> = vec![None; row.size()];
        let asked = label(&kit, stage, &pic, start, UNKNOWN, Palette::hero())?;
        fade_in_all(stage, &[asked], 0.3)?;
        labels[start] = Some(asked);
        line.wait_for(stage, "very many walkers")?;
        let at = pic.spot(start);
        let crowd: Vec<Mob> = (0..CROWD)
            .map(|i| {
                let x = at[0] + CROWD_X0 + CROWD_STEP * i as f64;
                walker_dot(stage, v(x, at[1]), CROWD_R, GHOST)
            })
            .collect();
        fade_in_lagged(stage, &crowd, 0.6, 0.3)?;
        let card_a = precisely(&kit, stage, CARD_A)?;
        fade_in_all(stage, &[card_a], CARD_IN)?;
        let card_a_read = stage.scene().time().to_f64() + read_s(CARD_A);

        // 01_5: one step ahead: a "?" at both far ends, a half beside each
        // wire; from there it starts afresh (card B)
        let line = say(&kit, stage, "01_5")?;
        fade_out_all(stage, &crowd, 0.3)?;
        line.wait_for(stage, "look one step ahead")?;
        let mut ahead = Vec::new();
        for y in [start - 1, start + 1] {
            ahead.push(label(&kit, stage, &pic, y, UNKNOWN, Palette::hero())?);
        }
        let half = small_tex(Q::ONE / Q::int(wires as i128));
        for i in [start - 1, start] {
            ahead.push(kit.tex(stage, &half, TOKEN_SIZE, palette::walker(), &[], pic.wire_mid(i, STEP_DX))?);
        }
        fade_in_lagged(stage, &ahead, 0.6, 0.3)?;
        line.wait_for(stage, "starts afresh")?;
        flash_all(stage, &[pic.junctions[start + 1]], 0.5)?;
        wait_until(stage, card_a_read)?;
        let card_b = precisely(&kit, stage, CARD_B)?;
        fade_out_all(stage, &[card_a], CARD_OUT / 2.0)?;
        fade_in_all(stage, &[card_b], CARD_OUT / 2.0)?;
        let card_b_read = stage.scene().time().to_f64() + read_s(CARD_B);

        // 01_6: the average of the two far ends' chances, halfway between
        let line = say(&kit, stage, "01_6")?;
        let rule = kit.tex(
            stage,
            &halfway_row(start, wires),
            36.0,
            Palette::ink(),
            &[],
            v(PANEL_X, ROW_Y),
        )?;
        fade_out_all(stage, &ahead, 0.2)?;
        fade_in_all(stage, &[rule], 0.3)?;
        line.wait_for(stage, "halfway")?;
        let note = kit.text(stage, HALFWAY, 22.0, Palette::dim(), v(PANEL_X, NOTE_Y))?;
        fade_in_all(stage, &[note], 0.4)?;

        // 01_7: the top's 1 and the bottom's 0; the other middle junctions
        // "?"; the question, and the countdown
        wait_until(stage, card_b_read)?;
        let line = say(&kit, stage, "01_7")?;
        fade_out_all(stage, &[card_b], CARD_OUT)?;
        let mut known = Vec::new();
        for x in [top, bottom] {
            let m = label(&kit, stage, &pic, x, &tex(chances[x]), Palette::ink())?;
            labels[x] = Some(m);
            known.push(m);
        }
        for x in row.middle().into_iter().filter(|&x| x != start) {
            let m = label(&kit, stage, &pic, x, UNKNOWN, Palette::dim())?;
            labels[x] = Some(m);
            known.push(m);
        }
        fade_in_lagged(stage, &known, 0.6, 0.3)?;
        line.wait_for(stage, "So what is it")?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 01_8: one fifth, and every middle junction's chance; the bars of
        // a straight ramp
        let line = say(&kit, stage, "01_8")?;
        let mut old = vec![countdown];
        let mut new = Vec::new();
        let mut colours = Vec::new();
        for x in row.middle() {
            if let Some(m) = labels[x] {
                old.push(m);
            }
            let m = label(&kit, stage, &pic, x, &tex(chances[x]), Palette::ink())?;
            labels[x] = Some(m);
            new.push(m);
            colours.push((pic.junctions[x], palette::value(chances[x])));
        }
        fade_out_all(stage, &old, 0.15)?;
        fade_in_all(stage, &new, 0.15)?;
        recolor(stage, &colours, 0.3)?;
        line.wait_for(stage, "means equal steps")?;
        let mut bars = Vec::new();
        for x in (0..row.size()).filter(|&x| chances[x].is_positive()) {
            let p = pic.spot(x);
            let x0 = p[0] + BAR_X0;
            let path = vec![v(x0, p[1]), v(x0 + BAR_LEN * chances[x].f64(), p[1])];
            let bar = polyline(path)
                .style(line_style(palette::value(chances[x]), BAR_W))
                .build();
            bars.push(stage.arena_mut().add(bar));
        }
        fade_in_lagged(stage, &bars, 0.9, 0.3)?;

        // 01_9: the chance of walking for ever: the same rule, zero at both
        // ends, so zero everywhere (card C)
        let line = say(&kit, stage, "01_9")?;
        let never = row.kirchhoff(Q::ZERO, Q::ZERO).map_err(fail)?;
        let for_ever = kit.text(stage, FOR_EVER, 26.0, Palette::dim(), pic.name_spot())?;
        fade_out_all(stage, &[name, rule, note], 0.2)?;
        fade_in_all(stage, &[for_ever], 0.2)?;
        name = for_ever;
        line.wait_for(stage, "with zero at both ends")?;
        let top_zero = label(&kit, stage, &pic, top, &tex(never[top]), Palette::ink())?;
        let mut going: Vec<Mob> = labels[top].into_iter().collect();
        going.push(dot);
        fade_out_all(stage, &going, 0.2)?;
        fade_in_all(stage, &[top_zero], 0.2)?;
        flash_all(stage, &[top_zero, labels[bottom].ok_or_else(|| fail("no bottom label".into()))?], 0.5)?;
        line.wait_for(stage, "equal steps from zero")?;
        let mut old: Vec<Mob> = bars.clone();
        let mut zeros = Vec::new();
        let mut colours = Vec::new();
        for x in row.middle() {
            if let Some(m) = labels[x] {
                old.push(m);
            }
            zeros.push(label(&kit, stage, &pic, x, &tex(never[x]), Palette::ink())?);
            colours.push((pic.junctions[x], palette::value(never[x])));
        }
        fade_out_all(stage, &old, 0.3)?;
        fade_in_all(stage, &zeros, 0.2)?;
        recolor(stage, &colours, 0.3)?;
        let card_text = stops_card().map_err(fail)?;
        let card_c = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[card_c], CARD_IN)?;
        let card_c_read = stage.scene().time().to_f64() + read_s(&card_text);
        flash_all(stage, &[name], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        wait_until(stage, card_c_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{JUNCTION_R, LINE_LABEL_GAP, THINK_R, line_spot};
    use crate::model::LINE_WIRES;

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "A walker"),
            ("01_2", "one of its two wires"),
            ("01_2", "it stops"),
            ("01_3", "Up, down"),
            ("01_4", "very many walkers"),
            ("01_5", "look one step ahead"),
            ("01_5", "starts afresh"),
            ("01_6", "halfway"),
            ("01_7", "So what is it"),
            ("01_8", "means equal steps"),
            ("01_9", "with zero at both ends"),
            ("01_9", "equal steps from zero"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The pictures are the facts': the route is "up, down, up, up, up and
    /// up" to the top, the ghost's first step ends at the bottom, the
    /// halfway row is the rule at the walker's junction, and the cards say
    /// what the facts say.
    #[test]
    fn the_line_is_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(facts::moves(&LINE_ROUTE).join(", "), "up, down, up, up, up, up");
        assert_eq!(f.get("R-F10.moves"), Some("up,down,up,up,up,up"));
        let row = facts::line().unwrap();
        assert_eq!(LINE_GHOST.last(), Some(&row.bottom()));
        assert_eq!(LINE_ROUTE.last(), Some(&row.top()));
        assert_eq!(halfway_row(LINE_START, 2), r"p(1) = \frac{p(0) + p(2)}{2}");
        assert_eq!(f.get("R-F10.answer"), Some("1/5"));
        let card = stops_card().unwrap();
        assert!(card.starts_with("endless walks exist, but together they have chance 0, so it stops with chance 1."));
        for text in [CARD_A.to_string(), CARD_B.to_string(), card] {
            let lines = explainer_kit::animatic::wrap(&text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
    }

    /// Cards A and B can be read in the time they stay up, by the scratch
    /// voice with this video's gaps: A from "very many walkers" until card
    /// B replaces it at "starts afresh", B from there to 01_7's start.
    #[test]
    fn the_cards_stay_up_long_enough_to_read() {
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let share = |id: &str, phrase: &str| {
            let text = script.text(id).unwrap();
            text[..text.find(phrase).unwrap()].chars().count() as f64 / text.chars().count() as f64
        };
        let a_up = len("01_4") * (1.0 - share("01_4", "very many walkers")) - CARD_IN
            + voice.line_gap("01_4", "01_5")
            + len("01_5") * share("01_5", "starts afresh");
        let b_up = len("01_5") * (1.0 - share("01_5", "starts afresh")) + voice.line_gap("01_5", "01_6") + len("01_6");
        assert!(read_s(CARD_A) <= a_up, "{:.2} s to read, {a_up:.2} s up", read_s(CARD_A));
        assert!(read_s(CARD_B) <= b_up, "{:.2} s to read, {b_up:.2} s up", read_s(CARD_B));
    }

    /// The line, its name, its labels and its bars stay inside the
    /// title-safe area and above a three-line lower-third; the panel and the
    /// countdown stay right of the bars.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let n = LINE_WIRES + 1;
        let (bottom, top) = (line_spot(LINE_AT, n, 0), line_spot(LINE_AT, n, n - 1));
        assert!(top[1] + 0.6 + 0.2 < FRAME_Y_RADIUS - 0.15, "the name inside the top");
        assert!(bottom[1] - JUNCTION_R - 0.15 > -FRAME_Y_RADIUS + 1.58, "above a lower-third");
        assert!(LINE_AT[0] - LINE_LABEL_GAP - 0.4 > -(FRAME_X_RADIUS - 0.25));
        let bars_end = LINE_AT[0] + BAR_X0 + BAR_LEN;
        let crowd_end = LINE_AT[0] + CROWD_X0 + CROWD_STEP * CROWD as f64;
        assert!(bars_end + 0.4 < PANEL_X - 1.6 && crowd_end + 0.4 < PANEL_X - 1.6);
        assert!(THINK_AT[0] - THINK_R > bars_end + 0.3);
        assert!(THINK_AT[1] + THINK_R + 0.3 < NOTE_Y - 0.2);
        assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 1.58);
        assert!(PANEL_X + 1.6 < FRAME_X_RADIUS - 0.25);
    }
}
