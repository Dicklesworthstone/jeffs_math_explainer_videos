//! Chapter 05, why each junction is an average: the circuit with the
//! preview's values, and the left junction's four wires (05_1); each wire's
//! drop, and which way its current flows (05_2); in equals out (05_3); both
//! middle voltages unknown, the left's balance gathered (05_4) into the
//! average of its far ends, the pair counted twice (05_5); the right
//! junction's wires, and the prediction (05_6); its balance, the average
//! of its far ends too (05_7). 05_1 to 05_3 are production chapter 02's
//! first three lines, their pictures kept (the Ohm card dropped, chapter
//! 02's card D gives Ohm's law; the Kirchhoff card cut to its last
//! sentence).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_SIZE, Picture, arrowheads, battery, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all,
    move_all, pause, picture, recolor, say, small_tex, tex, think, tokens_at, volts_text, wait_until,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

pub struct WhyAnAverage {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const VOLTAGE: &str = "voltage";
/// The two middle voltages' letters once they are hidden.
const LEFT_SYMBOL: &str = "L";
const RIGHT_SYMBOL: &str = "R";

/// 05_3's exact version (card G): why the drops balance as the currents do.
const CARD_G: &str = "each current is its drop over the same resistance, so the drops balance as the currents do.";

/// Where the picture stands (where chapter 04 leaves the circuit), and the
/// derivation's rows on the right, spaced for a stacked average beside a
/// sum: the averages stand about 0.9 tall, and each pair of neighbours
/// keeps about 0.2 between them.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const PANEL_X: f64 = 3.2;
const ROW_Y: [f64; 6] = [3.0, 1.96, 1.13, 0.29, -0.81, -1.55];
const ROW_SIZE: f64 = 34.0;
/// The prediction's countdown, in the panel under the left's average.
const THINK_AT: [f64; 2] = [3.2, 0.2];
/// How far along its wire a drop is written.
const DROP_T: f64 = 0.45;
/// A lower-third's fade in, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// "a + b + c" in TeX, each term inside a larger fraction.
fn terms(values: &[Q]) -> String {
    values.iter().map(|&q| small_tex(q)).collect::<Vec<_>>().join(" + ")
}

/// 05_3: the drops that flow in (or out), added, in volts. They are drops,
/// not currents: each current is its drop over the shared resistance.
fn flow_row(direction: &str, values: &[Q]) -> String {
    format!(
        r"\text{{drops {direction}: }} {} = {}\ \text{{V}}",
        terms(values),
        small_tex(facts::sum(values))
    )
}

/// The far ends of middle junction `x`'s wires in letters: the top's and the
/// bottom's voltages as numbers, a middle junction by its letter.
fn far_symbols(net: &Network, volts: &[Q], x: usize) -> Result<Vec<String>, String> {
    Ok(net
        .far_ends(x)?
        .into_iter()
        .map(|y| match y {
            LEFT => LEFT_SYMBOL.to_owned(),
            RIGHT => RIGHT_SYMBOL.to_owned(),
            _ => small_tex(volts[y]),
        })
        .collect())
}

/// 05_4: the balance at junction `name`, each wire's drop its far end less
/// the junction.
fn balance_row(name: &str, ends: &[String]) -> String {
    let drops: Vec<String> = ends.iter().map(|e| format!("({e} - {name})")).collect();
    format!("{} = 0", drops.join(" + "))
}

/// 05_4, 05_7: the unknown gathered, one from each wire.
fn gathered_row(name: &str, ends: &[String]) -> String {
    format!("{}{name} = {}", ends.len(), ends.join(" + "))
}

/// 05_5, 05_7: the junction as the average of its far ends.
fn average_row(name: &str, ends: &[String]) -> String {
    format!(r"{name} = \frac{{{}}}{{{}}}", ends.join(" + "), ends.len())
}

/// A middle junction's value written beside it. Detached.
fn value_label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str) -> fmn::Result<Mob> {
    kit.tex(
        stage,
        src,
        LABEL_SIZE,
        Palette::ink(),
        &[],
        pic.label_spot(x, END_LABEL_GAP),
    )
}

/// A row of the derivation. Detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, i: usize) -> fmn::Result<Mob> {
    kit.tex(stage, src, ROW_SIZE, Palette::ink(), &[], v(PANEL_X, ROW_Y[i]))
}

/// The mobs at the positions where `ends` holds `y`.
fn where_end(ends: &[usize], mobs: &[Mob], y: usize) -> Vec<Mob> {
    ends.iter()
        .zip(mobs)
        .filter(|&(&e, _)| e == y)
        .map(|(_, &m)| m)
        .collect()
}

/// The wires of `pic` away from junction `x`.
fn away_from(pic: &Picture, x: usize) -> Vec<Mob> {
    pic.strands
        .iter()
        .filter(|s| s.a != x && s.b != x)
        .map(|s| s.mob)
        .collect()
}

impl SceneConstruct for WhyAnAverage {
    fn name(&self) -> &str {
        "05_why_an_average"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let ends = net.far_ends(LEFT).map_err(fail)?;
        let drops = facts::drops().map_err(fail)?;
        let left_ends = far_symbols(&net, &volts, LEFT).map_err(fail)?;
        let right_ends = far_symbols(&net, &volts, RIGHT).map_err(fail)?;

        // 05_1: the circuit as chapter 04 left it; the left junction's four
        // wires, the others faint
        say(&kit, stage, "05_1")?;
        let colors: Vec<Srgb> = volts.iter().map(|&q| palette::value(q)).collect();
        let pic = picture(stage, &net, PICTURE_AT, &colors)?;
        let mut cell = battery(stage, pic.centre)?;
        let top_volts = kit.text(
            stage,
            &volts_text(volts[TOP]),
            24.0,
            Palette::ink(),
            pic.label_spot(TOP, END_LABEL_GAP),
        )?;
        let bottom_volts = kit.text(
            stage,
            &volts_text(volts[BOTTOM]),
            24.0,
            Palette::ink(),
            pic.label_spot(BOTTOM, END_LABEL_GAP),
        )?;
        cell.extend([top_volts, bottom_volts]);
        let left_value = value_label(&kit, stage, &pic, LEFT, &tex(volts[LEFT]))?;
        let right_value = value_label(&kit, stage, &pic, RIGHT, &tex(volts[RIGHT]))?;
        let circuit_name = kit.text(stage, VOLTAGE, 26.0, Palette::dim(), pic.name_spot())?;
        let mut arriving = pic.mobs();
        arriving.extend(&cell);
        arriving.extend([left_value, right_value, circuit_name]);
        fade_in_all(stage, &arriving, 0.6)?;
        let left_wires = pic.wires_at(LEFT);
        let others = away_from(&pic, LEFT);
        let dimmed: Vec<(Mob, Srgb)> = others.iter().map(|&m| (m, Palette::faint())).collect();
        recolor(stage, &dimmed, 0.4)?;
        flash_all(stage, &left_wires, 0.6)?;

        // 05_2: each wire's drop, and the way its current flows: in down the
        // pair, out to the right and to the bottom
        let line = say(&kit, stage, "05_2")?;
        let sizes: Vec<(String, Srgb)> = drops
            .iter()
            .map(|&d| (small_tex(if d.is_positive() { d } else { -d }), palette::current()))
            .collect();
        let inward: Vec<bool> = drops.iter().map(|d| d.is_positive()).collect();
        let drop_labels = tokens_at(&kit, stage, &pic, LEFT, &sizes, DROP_T)?;
        let arrows = arrowheads(stage, &pic, LEFT, &inward, palette::current())?;
        let mut pair = where_end(&ends, &drop_labels, TOP);
        pair.extend(where_end(&ends, &arrows, TOP));
        fade_in_all(stage, &pair, 0.5)?;
        line.wait_for(stage, "Then one fifth")?;
        let mut onward = Vec::new();
        for y in [RIGHT, BOTTOM] {
            onward.extend(where_end(&ends, &drop_labels, y));
            onward.extend(where_end(&ends, &arrows, y));
        }
        fade_in_lagged(stage, &onward, 0.8, 0.5)?;

        // 05_3: in equals out, the drops added; why they balance (card G)
        let line = say(&kit, stage, "05_3")?;
        let into: Vec<Q> = drops.iter().copied().filter(|d| d.is_positive()).collect();
        let out: Vec<Q> = drops.iter().filter(|d| !d.is_positive()).map(|&d| -d).collect();
        let balance_card = precisely(&kit, stage, CARD_G)?;
        fade_in_all(stage, &[balance_card], CARD_IN)?;
        let card_read = stage.scene().time().to_f64() + read_s(CARD_G);
        line.wait_for(stage, "Two fifths and two fifths in")?;
        let in_row = row(&kit, stage, &flow_row("in", &into), 0)?;
        fade_in_all(stage, &[in_row], 0.4)?;
        line.wait_for(stage, "one fifth and three fifths out")?;
        let out_row = row(&kit, stage, &flow_row("out", &out), 1)?;
        fade_in_all(stage, &[out_row], 0.4)?;

        // 05_4: both middle voltages hidden; the left's balance, and the
        // unknown gathered
        wait_until(stage, card_read)?;
        let line = say(&kit, stage, "05_4")?;
        let left_unknown = value_label(&kit, stage, &pic, LEFT, LEFT_SYMBOL)?;
        let right_unknown = value_label(&kit, stage, &pic, RIGHT, RIGHT_SYMBOL)?;
        fade_out_all(stage, &[balance_card, left_value, right_value], 0.15)?;
        fade_in_all(stage, &[left_unknown, right_unknown], 0.15)?;
        recolor(
            stage,
            &[
                (pic.junctions[LEFT], palette::unknown()),
                (pic.junctions[RIGHT], palette::unknown()),
            ],
            0.3,
        )?;
        let balanced = row(&kit, stage, &balance_row(LEFT_SYMBOL, &left_ends), 2)?;
        fade_in_all(stage, &[balanced], 0.4)?;
        line.wait_for(stage, "four times the left")?;
        let gathered = row(&kit, stage, &gathered_row(LEFT_SYMBOL, &left_ends), 3)?;
        fade_in_all(stage, &[gathered], 0.4)?;

        // 05_5: the left is the average of its far ends, the pair's far end
        // twice; the working goes and the average rises to the top row
        let line = say(&kit, stage, "05_5")?;
        let left_average = row(&kit, stage, &average_row(LEFT_SYMBOL, &left_ends), 4)?;
        fade_in_all(stage, &[left_average], 0.4)?;
        line.wait_for(stage, "The pair is two wires")?;
        flash_all(stage, &where_end(&ends, &left_wires, TOP), 0.6)?;
        let mut working = vec![in_row, out_row, balanced, gathered];
        working.extend(&drop_labels);
        working.extend(&arrows);
        fade_out_all(stage, &working, 0.4)?;
        let restored: Vec<(Mob, Srgb)> = others.iter().map(|&m| (m, palette::wire())).collect();
        recolor(stage, &restored, 0.3)?;
        move_all(stage, &[(left_average, v(PANEL_X, ROW_Y[0]))], 0.6)?;

        // 05_6: the right junction's four wires, the others faint; no far
        // end's value is shown through the countdown
        let question = say(&kit, stage, "05_6")?;
        let right_others = away_from(&pic, RIGHT);
        let faint: Vec<(Mob, Srgb)> = right_others.iter().map(|&m| (m, Palette::faint())).collect();
        recolor(stage, &faint, 0.4)?;
        question.wait_for(stage, "One wire goes up")?;
        flash_all(stage, &pic.wires_at(RIGHT), 0.6)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 05_7: the right's balance gathered, then its average
        let line = say(&kit, stage, "05_7")?;
        let right_gathered = row(&kit, stage, &gathered_row(RIGHT_SYMBOL, &right_ends), 2)?;
        fade_out_all(stage, &[countdown], 0.15)?;
        fade_in_all(stage, &[right_gathered], 0.25)?;
        line.wait_for(stage, "So the right is the average")?;
        let right_average = row(&kit, stage, &average_row(RIGHT_SYMBOL, &right_ends), 1)?;
        fade_in_all(stage, &[right_average], 0.4)?;
        let back: Vec<(Mob, Srgb)> = right_others.iter().map(|&m| (m, palette::wire())).collect();
        recolor(stage, &back, 0.3)?;
        fade_out_all(stage, &[right_gathered], 0.3)?;
        flash_all(stage, &[left_average, right_average], 0.6)?;
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

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("05_2", "Then one fifth"),
            ("05_3", "Two fifths and two fifths in"),
            ("05_3", "one fifth and three fifths out"),
            ("05_4", "four times the left"),
            ("05_5", "The pair is two wires"),
            ("05_6", "One wire goes up"),
            ("05_7", "So the right is the average"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The derivation's rows are the facts': the drops in and out, then both
    /// balances in letters, each far end counted once per wire, the pair's
    /// twice.
    #[test]
    fn the_rows_are_built_from_the_facts() {
        let net = facts::network().unwrap();
        let volts = facts::voltages().unwrap();
        let drops = facts::drops().unwrap();
        let into: Vec<Q> = drops.iter().copied().filter(|d| d.is_positive()).collect();
        let out: Vec<Q> = drops.iter().filter(|d| !d.is_positive()).map(|&d| -d).collect();
        assert_eq!(
            flow_row("in", &into),
            r"\text{drops in: } \tfrac{2}{5} + \tfrac{2}{5} = \tfrac{4}{5}\ \text{V}"
        );
        assert_eq!(
            flow_row("out", &out),
            r"\text{drops out: } \tfrac{1}{5} + \tfrac{3}{5} = \tfrac{4}{5}\ \text{V}"
        );
        let left = far_symbols(&net, &volts, LEFT).unwrap();
        let right = far_symbols(&net, &volts, RIGHT).unwrap();
        assert_eq!(left, ["1", "1", "R", "0"]);
        assert_eq!(right, ["1", "L", "0", "0"]);
        assert_eq!(balance_row("L", &left), "(1 - L) + (1 - L) + (R - L) + (0 - L) = 0");
        assert_eq!(gathered_row("L", &left), "4L = 1 + 1 + R + 0");
        assert_eq!(average_row("L", &left), r"L = \frac{1 + 1 + R + 0}{4}");
        assert_eq!(gathered_row("R", &right), "4R = 1 + L + 0 + 0");
        assert_eq!(average_row("R", &right), r"R = \frac{1 + L + 0 + 0}{4}");
        // the spoken balances are the facts' far ends
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F9.left.far_ends"), Some("top,top,right,bottom"));
        assert_eq!(f.get("R-F13.right.far_ends"), Some("top,left,bottom,bottom"));
        // and the solved values keep both: 4 x 3/5 = 1 + 1 + 2/5 + 0
        assert_eq!(Q::int(4) * volts[LEFT], Q::int(2) + volts[RIGHT]);
        assert_eq!(Q::int(4) * volts[RIGHT], Q::ONE + volts[LEFT]);
    }

    /// Card G fits a lower-third of three lines and can be read from when it
    /// is in to the end of 05_3, by the scratch voice with this video's
    /// gaps.
    #[test]
    fn the_card_fits_and_can_be_read() {
        assert!(explainer_kit::animatic::wrap(CARD_G, 84, 10).len() <= 3);
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = voice.length("05_3").unwrap();
        assert!(read_s(CARD_G) <= len - CARD_IN + voice.line_gap("05_3", "05_4"));
    }

    /// The panel's rows stay right of the picture's battery, inside the
    /// title-safe area, and above a three-line lower-third, and its tall
    /// rows never touch.
    #[test]
    fn the_panel_keeps_clear_of_the_picture_and_the_card() {
        use crate::chapters::draw::{BATTERY_X, THINK_R};
        const { assert!(PANEL_X - 2.8 > PICTURE_AT[0] + BATTERY_X + 0.3) };
        const { assert!(PANEL_X + 2.8 < FRAME_X_RADIUS - 0.25) };
        const { assert!(ROW_Y[0] + 0.45 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW_Y[4] - 0.45 > -FRAME_Y_RADIUS + 1.58) };
        // the two averages (rows 0 and 1) each reach about 0.45 from their
        // centre: a gap stays between them
        const { assert!(ROW_Y[0] - ROW_Y[1] > 2.0 * 0.45 + 0.1) };
        // the countdown sits under the top row, right of the battery
        const { assert!(THINK_AT[1] + THINK_R + 0.4 < ROW_Y[0]) };
        const { assert!(THINK_AT[0] - THINK_R > PICTURE_AT[0] + BATTERY_X + 0.3) };
    }
}
