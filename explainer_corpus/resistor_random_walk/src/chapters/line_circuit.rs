//! Chapter 02, the same line with a battery: the top held at one volt and
//! the bottom at zero (02_1); current flows down, in equals out at every
//! middle junction (02_2), so one current in every wire and equal drops
//! (02_3); the prediction (02_4); a fifth of a volt, the walker's chance,
//! junction by junction (02_5); at one junction the drop above equals the
//! drop below (02_6), so its voltage is the average of its far ends', the
//! walker's rule (02_7); on a line that rule allows only equal steps, so
//! the two answers had to agree (02_8).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    LABEL_SIZE, LinePicture, TOKEN_SIZE, WIRE_W, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all,
    line_battery, line_picture, line_style, pause, recolor, say, small_tex, tex, think, volts_text, wait_until,
};
use crate::facts::{self, BATTERY};
use crate::model::{LINE_START, Q};
use crate::palette;

pub struct LineCircuit {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const VOLTAGE: &str = "voltage";
const CHANCE: &str = "chance of reaching the top first";
const UNKNOWN: &str = "?";

/// 02_3's exact version (card D): the only physics the video claims.
const CARD_D: &str = "ideal wires, steady currents, an ideal battery. Ohm's law: current = drop ÷ resistance. \
                      Kirchhoff's current law: the currents into a middle junction add to zero.";

/// Where the circuit's line stands, and the walk's beside it (02_5); the
/// row on the right (02_7); the countdown.
const LINE_AT: [f64; 2] = [-4.4, 0.45];
const WALK_AT: [f64; 2] = [-0.6, 0.45];
const PANEL_X: f64 = 3.6;
const ROW_Y: f64 = 1.6;
const THINK_AT: [f64; 2] = [3.6, -0.6];
/// The junction 02_6 and 02_7 single out, and how far right of a wire its
/// drop is written; a current's arrowhead.
const FOCUS: usize = 3;
const DROP_DX: f64 = 0.45;
const ARROW: f64 = 0.18;
/// A lower-third's fade in and out, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// 02_7: junction `x`'s voltage, the average of its far ends'.
fn halfway_row(volts: &[Q], x: usize) -> String {
    format!(
        r"{} = \frac{{{} + {}}}{{2}}",
        small_tex(volts[x]),
        small_tex(volts[x - 1]),
        small_tex(volts[x + 1])
    )
}

/// The current's arrowhead on a wire's middle, pointing down. Detached.
fn arrow_down(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let path = vec![
        v(at[0] - 0.6 * ARROW, at[1] + ARROW / 2.0),
        v(at[0], at[1] - ARROW / 2.0),
        v(at[0] + 0.6 * ARROW, at[1] + ARROW / 2.0),
    ];
    stage
        .arena_mut()
        .add(polyline(path).style(line_style(palette::current(), WIRE_W)).build())
}

/// Junction `x`'s value on `pic`, left of it. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &LinePicture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x))
}

impl SceneConstruct for LineCircuit {
    fn name(&self) -> &str {
        "02_the_line_as_a_circuit"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let row = facts::line().map_err(fail)?;
        let volts = row.kirchhoff(BATTERY[0], BATTERY[1]).map_err(fail)?;
        let chances = row.walk().map_err(fail)?.top;
        let (top, bottom) = (row.top(), row.bottom());
        let middle = row.middle();

        // 02_1: the line, its name; the battery and the ends' voltages
        let line = say(&kit, stage, "02_1")?;
        let colors: Vec<Srgb> = (0..row.size())
            .map(|x| if row.is_end(x) { palette::value(volts[x]) } else { palette::unknown() })
            .collect();
        let pic = line_picture(stage, &row, LINE_AT, &colors)?;
        let name = kit.text(stage, VOLTAGE, 26.0, Palette::dim(), pic.name_spot())?;
        let mut arriving = pic.mobs();
        arriving.push(name);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "A battery")?;
        let mut cell = line_battery(stage, &pic)?;
        for x in [top, bottom] {
            cell.push(kit.text(stage, &volts_text(volts[x]), 24.0, Palette::ink(), pic.label_spot(x))?);
        }
        fade_in_all(stage, &cell, 0.6)?;

        // 02_2: current flows down every wire; at one junction in equals out
        let line = say(&kit, stage, "02_2")?;
        let arrows: Vec<Mob> = (0..pic.wires.len())
            .map(|i| arrow_down(stage, pic.wire_mid(i, 0.0)))
            .collect();
        fade_in_lagged(stage, &arrows, 0.6, 0.3)?;
        line.wait_for(stage, "the current in equals")?;
        flash_all(stage, &[arrows[FOCUS], arrows[FOCUS - 1]], 0.6)?;

        // 02_3: one current in every wire, so equal drops (card D)
        let line = say(&kit, stage, "02_3")?;
        line.wait_for(stage, "the same current")?;
        flash_all(stage, &arrows, 0.6)?;
        line.wait_for(stage, "equal drops")?;
        flash_all(stage, &pic.wires, 0.6)?;
        let card = precisely(&kit, stage, CARD_D)?;
        fade_in_all(stage, &[card], CARD_IN)?;
        let card_read = stage.scene().time().to_f64() + read_s(CARD_D);

        // 02_4: the question; the card stays while it is asked and goes as
        // it ends; no drop is shown with a number through the countdown
        let question = say(&kit, stage, "02_4")?;
        let asked = label(&kit, stage, &pic, LINE_START, UNKNOWN, Palette::hero())?;
        fade_in_all(stage, &[asked], 0.3)?;
        wait_until(stage, card_read.max(question.start + question.len - CARD_OUT))?;
        fade_out_all(stage, &[card], CARD_OUT)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 02_5: a fifth of a volt beside every wire, the voltages; the walk's
        // line beside it, the same numbers
        let line = say(&kit, stage, "02_5")?;
        let drops: Vec<Q> = row.wires().iter().map(|w| volts[w.b] - volts[w.a]).collect();
        let drop_labels: Vec<Mob> = drops
            .iter()
            .enumerate()
            .map(|(i, &d)| kit.tex(stage, &small_tex(d), TOKEN_SIZE, palette::current(), &[], pic.wire_mid(i, DROP_DX)))
            .collect::<fmn::Result<_>>()?;
        let mut values = Vec::new();
        let mut colours = Vec::new();
        for &x in &middle {
            values.push(label(&kit, stage, &pic, x, &tex(volts[x]), Palette::ink())?);
            colours.push((pic.junctions[x], palette::value(volts[x])));
        }
        fade_out_all(stage, &[countdown, asked], 0.15)?;
        let mut coming = drop_labels.clone();
        coming.extend(&values);
        fade_in_all(stage, &coming, 0.3)?;
        recolor(stage, &colours, 0.3)?;
        line.wait_for(stage, "Junction by junction")?;
        let walk_colors: Vec<Srgb> = chances.iter().map(|&q| palette::value(q)).collect();
        let walk = line_picture(stage, &row, WALK_AT, &walk_colors)?;
        let walk_name = kit.text(stage, CHANCE, 26.0, Palette::dim(), walk.name_spot())?;
        let mut walk_values = Vec::new();
        for x in 0..row.size() {
            walk_values.push(label(&kit, stage, &walk, x, &tex(chances[x]), Palette::ink())?);
        }
        let mut arriving = walk.mobs();
        arriving.push(walk_name);
        arriving.extend(&walk_values);
        fade_in_all(stage, &arriving, 0.5)?;
        if chances != volts {
            return Err(fail(format!("the line's chances {chances:?} are not its voltages {volts:?}")));
        }
        for (i, &x) in middle.iter().enumerate() {
            flash_all(stage, &[values[i], walk_values[x]], 0.3)?;
        }

        // 02_6: at one junction, the drop above against the drop below
        let line = say(&kit, stage, "02_6")?;
        line.wait_for(stage, "the drop above")?;
        flash_all(stage, &[drop_labels[FOCUS - 1], drop_labels[FOCUS]], 0.6)?;

        // 02_7: so its voltage is the average of its far ends'; the walker's
        // rule
        let line = say(&kit, stage, "02_7")?;
        let rule = kit.tex(
            stage,
            &halfway_row(&volts, FOCUS),
            36.0,
            Palette::ink(),
            &[],
            v(PANEL_X, ROW_Y),
        )?;
        fade_in_all(stage, &[rule], 0.4)?;
        line.wait_for(stage, "the walker's rule")?;
        flash_all(stage, &[pic.junctions[FOCUS], walk.junctions[FOCUS]], 0.6)?;

        // 02_8: the same rule, the same top and bottom: the two answers
        let line = say(&kit, stage, "02_8")?;
        line.wait_for(stage, "the same top and bottom")?;
        flash_all(
            stage,
            &[
                pic.junctions[top],
                pic.junctions[bottom],
                walk.junctions[top],
                walk.junctions[bottom],
            ],
            0.6,
        )?;
        line.wait_for(stage, "had to agree")?;
        let mut all = values.clone();
        all.extend(middle.iter().map(|&x| walk_values[x]));
        flash_all(stage, &all, 0.6)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{LINE_BATTERY_X, LINE_LABEL_GAP, THINK_R};

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "A battery"),
            ("02_2", "the current in equals"),
            ("02_3", "the same current"),
            ("02_3", "equal drops"),
            ("02_5", "Junction by junction"),
            ("02_6", "the drop above"),
            ("02_7", "the walker's rule"),
            ("02_8", "the same top and bottom"),
            ("02_8", "had to agree"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The circuit's numbers are the facts': equal drops of a fifth, the
    /// voltages the walker's chances, the halfway row at the junction shown.
    #[test]
    fn the_line_circuit_is_the_facts() {
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F11.drops"), Some("1/5,1/5,1/5,1/5,1/5"));
        assert_eq!(f.get("R-F11.answer"), Some("1/5"));
        assert_eq!(f.get("R-F11.same"), Some("true"));
        let row = facts::line().unwrap();
        let volts = row.kirchhoff(BATTERY[0], BATTERY[1]).unwrap();
        assert_eq!(
            halfway_row(&volts, FOCUS),
            r"\tfrac{3}{5} = \frac{\tfrac{2}{5} + \tfrac{4}{5}}{2}"
        );
        assert!(row.middle().contains(&FOCUS));
        assert!(explainer_kit::animatic::wrap(CARD_D, 84, 10).len() <= 3);
    }

    /// Card D can be read from "equal drops" to the end of the 02_4
    /// question, by the scratch voice with this video's gaps.
    #[test]
    fn the_card_stays_up_long_enough_to_read() {
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let text = script.text("02_3").unwrap();
        let at = text[..text.find("equal drops").unwrap()].chars().count() as f64 / text.chars().count() as f64;
        let up = len("02_3") * (1.0 - at) - CARD_IN + voice.line_gap("02_3", "02_4") + len("02_4") - CARD_OUT;
        assert!(read_s(CARD_D) <= up, "{:.2} s to read, {up:.2} s up", read_s(CARD_D));
    }

    /// The two lines, the battery between them and the panel stay apart and
    /// inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const { assert!(LINE_AT[0] - LINE_LABEL_GAP - 0.5 > -(FRAME_X_RADIUS - 0.25)) };
        const { assert!(LINE_AT[0] + LINE_BATTERY_X + 0.3 < WALK_AT[0] - LINE_LABEL_GAP - 0.4) };
        const { assert!(LINE_AT[0] + DROP_DX + 0.4 < LINE_AT[0] + LINE_BATTERY_X - 0.1) };
        const { assert!(WALK_AT[0] + 0.5 < PANEL_X - 1.6) };
        const { assert!(PANEL_X + 1.6 < FRAME_X_RADIUS - 0.25) };
        const { assert!(THINK_AT[1] + THINK_R + 0.3 < ROW_Y - 0.4) };
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 1.58) };
    }
}
