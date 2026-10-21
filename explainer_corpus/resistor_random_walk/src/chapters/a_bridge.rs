//! Chapter 03, a network with a bridge: identical wires join four
//! junctions, the top held at one volt and the bottom at zero (03_1);
//! current in equals current out, so each middle junction is the average of
//! its wires' far ends (03_2); the values previewed, the left junction's
//! four far ends sliding in to its value and the right's check without
//! words (03_3); the school shortcuts for wires (03_4, 03_5); the pairs
//! merged, and the prediction (03_6); none applies (03_7); the bridge blocks
//! both, so the voltages must come from the rule at both junctions at once
//! (03_8). 03_1 to 03_3 are production chapter 01's first three lines,
//! their pictures and frozen timing kept.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, EQUATION_SIZE, LABEL_SIZE, Picture, WIRE_W, battery, equation, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flash_all, flat, heavy_link, line_style, link_spot, pause, picture, recolor, say, slide_in,
    swap, tex, think, tokens, value_texts, volts_text, wait_until,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

pub struct ABridge {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HEADING: &str = "two school shortcuts";
const BRIDGE: &str = "bridge";
const SIDE_BY_SIDE: &str = r"\text{conductance} \times 2";
const IN_A_ROW: &str = r"\text{resistance} \times 2";
const SIDES: [&str; 2] = ["left", "right"];

/// 03_2's exact version (card E): wires side by side, the pair's far end
/// counted twice.
const CARD_E: &str = "wires side by side add their conductances, so the average counts the pair's far end twice.";

/// 03_5's exact version (card F): the two school shortcuts, as Doyle and
/// Snell state them (§1.3.4).
const CARD_F: &str = "side by side, conductances add; in a row, with nothing else at the junction between, resistances \
                      add (Doyle and Snell, section 1.3.4).";

/// Where the circuit stands (as production 01_1 had it, alone); the
/// shortcuts' panel on the right: its heading, its two small drawings and
/// their results, the counts of 03_7; the prediction's countdown.
const CIRCUIT_AT: [f64; 2] = [-0.5, 0.3];
const PANEL_X: f64 = 5.0;
const HEADING_Y: f64 = 2.9;
const DEMO_Y: [f64; 2] = [1.9, 0.5];
const DEMO_HALF: f64 = 0.9;
const DEMO_BOW: f64 = 0.14;
const DEMO_DOT_R: f64 = 0.1;
const DEMO_LABEL_DY: f64 = -0.45;
const COUNT_Y: [f64; 2] = [-0.5, -0.95];
const THINK_AT: [f64; 2] = [5.0, -1.0];
/// A lower-third's fade in and out, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// The fill of each junction before its value is shown: the top and the
/// bottom in their colors, the middle ones faint.
fn first_colors(net: &Network) -> Vec<Srgb> {
    (0..net.size())
        .map(|x| match x {
            TOP => palette::top(),
            BOTTOM => palette::bottom(),
            _ => palette::unknown(),
        })
        .collect()
}

/// A middle junction's value, written beside it. Detached.
fn value_label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, q: Q) -> fmn::Result<Mob> {
    kit.tex(
        stage,
        &tex(q),
        LABEL_SIZE,
        Palette::ink(),
        &[],
        pic.label_spot(x, END_LABEL_GAP),
    )
}

/// A junction of the shortcuts' drawings. Detached.
fn small_dot(stage: &mut Stage<'_>, x: f64, y: f64) -> Mob {
    let dot = Circle::new()
        .radius(DEMO_DOT_R)
        .style(flat(palette::unknown(), 1.0))
        .build()
        .moved_to(v(x, y));
    stage.arena_mut().add(dot)
}

/// A wire of the shortcuts' drawings through `path`, `width` wide. Detached.
fn demo_wire(stage: &mut Stage<'_>, path: Vec<Vec3>, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(path).style(line_style(palette::wire(), width)).build())
}

impl SceneConstruct for ABridge {
    fn name(&self) -> &str {
        "03_a_bridge"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let volt_ends = facts::far_values(&volts, LEFT).map_err(fail)?;

        // 03_1: the junctions, the wires, the doubled pairs, the battery
        let line = say(&kit, stage, "03_1")?;
        let circuit = picture(stage, &net, CIRCUIT_AT, &first_colors(&net))?;
        fade_in_lagged(stage, &circuit.junctions, 0.8, 0.3)?;
        let wires: Vec<Mob> = circuit.strands.iter().map(|s| s.mob).collect();
        fade_in_lagged(stage, &wires, 1.0, 0.15)?;
        line.wait_for(stage, "doubled")?;
        flash_all(stage, &circuit.doubled(), 0.6)?;
        line.wait_for(stage, "A battery")?;
        let mut cell = battery(stage, circuit.centre)?;
        let top_volts = kit.text(
            stage,
            &volts_text(volts[TOP]),
            24.0,
            Palette::ink(),
            circuit.label_spot(TOP, END_LABEL_GAP),
        )?;
        let bottom_volts = kit.text(
            stage,
            &volts_text(volts[BOTTOM]),
            24.0,
            Palette::ink(),
            circuit.label_spot(BOTTOM, END_LABEL_GAP),
        )?;
        cell.extend([top_volts, bottom_volts]);
        fade_in_all(stage, &cell, 0.6)?;

        // 03_2: each middle junction's four wires, and the exact version
        let line = say(&kit, stage, "03_2")?;
        flash_all(stage, &circuit.wires_at(LEFT), 0.6)?;
        flash_all(stage, &circuit.wires_at(RIGHT), 0.6)?;
        line.wait_for(stage, "the average")?;
        let pair_card = precisely(&kit, stage, CARD_E)?;
        fade_in_all(stage, &[pair_card], CARD_IN)?;
        let pair_read = stage.scene().time().to_f64() + read_s(CARD_E);

        // 03_3: the right junction's value; the left's four far ends slide in
        // and average to its value; then the right's check, without words
        wait_until(stage, pair_read)?;
        let line = say(&kit, stage, "03_3")?;
        let right_label = value_label(&kit, stage, &circuit, RIGHT, volts[RIGHT])?;
        swap(stage, &[pair_card], &[right_label], 0.3)?;
        recolor(stage, &[(circuit.junctions[RIGHT], palette::value(volts[RIGHT]))], 0.3)?;
        let arriving = tokens(&kit, stage, &circuit, LEFT, &value_texts(&volt_ends))?;
        line.wait_for(stage, "one, one")?;
        fade_in_lagged(stage, &arriving, 0.8, 0.4)?;
        slide_in(stage, &arriving, circuit.spot(LEFT), 0.7)?;
        line.wait_for(stage, "average to")?;
        let left_rule = kit.tex(
            stage,
            &equation(&volt_ends, volts[LEFT]),
            EQUATION_SIZE,
            Palette::ink(),
            &[],
            circuit.equation_spot(),
        )?;
        fade_in_all(stage, &[left_rule], 0.4)?;
        line.wait_for(stage, "to three fifths")?;
        let left_label = value_label(&kit, stage, &circuit, LEFT, volts[LEFT])?;
        fade_in_all(stage, &[left_label], 0.3)?;
        recolor(stage, &[(circuit.junctions[LEFT], palette::value(volts[LEFT]))], 0.3)?;
        let right_ends = facts::far_values(&volts, RIGHT).map_err(fail)?;
        let checking = tokens(&kit, stage, &circuit, RIGHT, &value_texts(&right_ends))?;
        fade_in_all(stage, &checking, 0.2)?;
        slide_in(stage, &checking, circuit.spot(RIGHT), 0.45)?;
        let right_rule = kit.tex(
            stage,
            &equation(&right_ends, volts[RIGHT]),
            EQUATION_SIZE,
            Palette::ink(),
            &[],
            circuit.equation_spot(),
        )?;
        fade_out_all(stage, &[left_rule], 0.125)?;
        fade_in_all(stage, &[right_rule], 0.125)?;
        flash_all(stage, &[right_label], 0.35)?;
        pause(stage, 3.0)?;

        // 03_4: the equation goes; the shortcuts' heading
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &[right_rule], 0.3)?;
        line.wait_for(stage, "Not from the school shortcuts")?;
        let heading = kit.text(stage, HEADING, 24.0, Palette::dim(), v(PANEL_X, HEADING_Y))?;
        fade_in_all(stage, &[heading], 0.4)?;

        // 03_5: two wires side by side become one heavier wire; two in a
        // row become one long one; the exact version
        let line = say(&kit, stage, "03_5")?;
        let (x0, x1, y) = (PANEL_X - DEMO_HALF, PANEL_X + DEMO_HALF, DEMO_Y[0]);
        let r = DEMO_DOT_R;
        let pair_ends = [small_dot(stage, x0, y), small_dot(stage, x1, y)];
        let pair_wires = [
            demo_wire(stage, vec![v(x0 + r, y), v(PANEL_X, y + DEMO_BOW), v(x1 - r, y)], WIRE_W),
            demo_wire(stage, vec![v(x0 + r, y), v(PANEL_X, y - DEMO_BOW), v(x1 - r, y)], WIRE_W),
        ];
        let mut first = pair_ends.to_vec();
        first.extend(pair_wires);
        fade_in_all(stage, &first, 0.4)?;
        line.wait_for(stage, "act as one that carries")?;
        let merged_pair = demo_wire(stage, vec![v(x0 + r, y), v(x1 - r, y)], 2.0 * WIRE_W);
        let pair_result = kit.tex(
            stage,
            SIDE_BY_SIDE,
            26.0,
            Palette::ink(),
            &[],
            v(PANEL_X, y + DEMO_LABEL_DY),
        )?;
        fade_out_all(stage, &pair_wires, 0.2)?;
        fade_in_all(stage, &[merged_pair, pair_result], 0.3)?;
        line.wait_for(stage, "Two in a row")?;
        let y = DEMO_Y[1];
        let row_dots = [
            small_dot(stage, x0, y),
            small_dot(stage, PANEL_X, y),
            small_dot(stage, x1, y),
        ];
        let row_wires = [
            demo_wire(stage, vec![v(x0 + r, y), v(PANEL_X - r, y)], WIRE_W),
            demo_wire(stage, vec![v(PANEL_X + r, y), v(x1 - r, y)], WIRE_W),
        ];
        let shortcut_card = precisely(&kit, stage, CARD_F)?;
        let mut arriving = row_dots.to_vec();
        arriving.extend(row_wires);
        arriving.push(shortcut_card);
        fade_in_all(stage, &arriving, CARD_IN)?;
        let shortcut_read = stage.scene().time().to_f64() + read_s(CARD_F);
        line.wait_for(stage, "act as one twice as hard")?;
        let merged_row = demo_wire(stage, vec![v(x0 + r, y), v(x1 - r, y)], WIRE_W);
        let row_result = kit.tex(
            stage,
            IN_A_ROW,
            26.0,
            Palette::ink(),
            &[],
            v(PANEL_X, y + DEMO_LABEL_DY),
        )?;
        let mut going = row_wires.to_vec();
        going.push(row_dots[1]);
        fade_out_all(stage, &going, 0.2)?;
        fade_in_all(stage, &[merged_row, row_result], 0.3)?;
        // what stays of the drawings until 03_8
        let mut demo = pair_ends.to_vec();
        demo.extend([merged_pair, pair_result, row_dots[0], row_dots[2], merged_row, row_result]);

        // 03_6: each pair on the circuit becomes one heavier wire, five
        // links; the card stays while the question is asked and goes as it
        // ends; the countdown under the drawings
        let question = say(&kit, stage, "03_6")?;
        question.wait_for(stage, "Merge each pair")?;
        let mut links: Vec<(usize, usize, Mob)> = Vec::new();
        let mut heavy = Vec::new();
        for w in net.wires() {
            let mob = if w.conductance == Q::ONE {
                circuit.strand_between(w.a, w.b)?.mob
            } else {
                let m = heavy_link(stage, &circuit, w.a, w.b, WIRE_W * w.conductance.f64());
                heavy.push(m);
                m
            };
            links.push((w.a, w.b, mob));
        }
        let pairs = circuit.doubled();
        fade_out_all(stage, &pairs, 0.25)?;
        fade_in_all(stage, &heavy, 0.25)?;
        wait_until(stage, shortcut_read.max(question.start + question.len - CARD_OUT))?;
        fade_out_all(stage, &[shortcut_card], CARD_OUT)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 03_7: none; the five links one after another, none sharing both
        // ends with another; three meet at each middle junction
        let line = say(&kit, stage, "03_7")?;
        fade_out_all(stage, &[countdown], 0.2)?;
        for &(_, _, m) in &links {
            flash_all(stage, &[m], 0.3)?;
        }
        line.wait_for(stage, "three wires meet")?;
        let mut counts = Vec::new();
        for (side, x) in SIDES.iter().zip([LEFT, RIGHT]) {
            let at: Vec<Mob> = links
                .iter()
                .filter(|&&(a, b, _)| a == x || b == x)
                .map(|&(_, _, m)| m)
                .collect();
            if at.len() != net.neighbours(x).len() {
                return Err(fail(format!("the {side} junction meets {} links, not its neighbours'", at.len())));
            }
            let i = usize::from(x != LEFT);
            let count = kit.text(
                stage,
                &format!("{} wires meet at the {side}", at.len()),
                22.0,
                Palette::ink(),
                v(PANEL_X, COUNT_Y[i]),
            )?;
            fade_in_all(stage, &[count], 0.2)?;
            flash_all(stage, &at, 0.5)?;
            counts.push(count);
        }

        // 03_8: the bridge blocks both; the pairs come back and the
        // shortcuts' panel goes
        let line = say(&kit, stage, "03_8")?;
        let bridge = circuit.strand_between(LEFT, RIGHT)?.mob;
        let bridge_name = kit.text(stage, BRIDGE, 22.0, Palette::dim(), link_spot(&circuit, LEFT, RIGHT, 0.35))?;
        fade_in_all(stage, &[bridge_name], 0.3)?;
        flash_all(stage, &[bridge], 0.6)?;
        line.wait_for(stage, "The voltages must come")?;
        let mut going = heavy.clone();
        going.extend(&demo);
        going.extend(&counts);
        going.push(heading);
        fade_out_all(stage, &going, 0.3)?;
        fade_in_all(stage, &pairs, 0.3)?;
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
    use crate::chapters::draw::{BATTERY_X, LABEL_GAP, SPOTS, THINK_R};

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "doubled"),
            ("03_1", "A battery"),
            ("03_2", "the average"),
            ("03_3", "one, one"),
            ("03_3", "average to"),
            ("03_3", "to three fifths"),
            ("03_4", "Not from the school shortcuts"),
            ("03_5", "act as one that carries"),
            ("03_5", "Two in a row"),
            ("03_5", "act as one twice as hard"),
            ("03_6", "Merge each pair"),
            ("03_7", "three wires meet"),
            ("03_8", "The voltages must come"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The merged picture is the facts': five links, none joining the same
    /// two junctions, three at each middle junction, the bridge between
    /// them; the two doubled links are the ones drawn heavier.
    #[test]
    fn the_merged_links_are_the_facts() {
        let net = facts::network().unwrap();
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F12.links"), Some(net.wires().len().to_string().as_str()));
        assert_eq!(f.get("R-F12.meeting"), Some("3,3"));
        assert_eq!(f.get("R-F12.bridge"), Some("left-right"));
        let doubled: Vec<(usize, usize)> = net
            .wires()
            .iter()
            .filter(|w| w.conductance != Q::ONE)
            .map(|w| (w.a, w.b))
            .collect();
        assert_eq!(doubled, [(TOP, LEFT), (RIGHT, BOTTOM)]);
        for x in [LEFT, RIGHT] {
            assert_eq!(net.neighbours(x).len(), 3);
        }
    }

    /// The exact versions fit a lower-third of three lines, and card F can
    /// be read before the countdown: in at "Two in a row", out as the 03_6
    /// question ends, by the scratch voice with this video's gaps, so the
    /// scene's guard adds no time to the draft (with a real voice that runs
    /// short it holds the countdown back until the card is read).
    #[test]
    fn the_cards_fit_and_can_be_read() {
        for text in [CARD_E, CARD_F] {
            let lines = explainer_kit::animatic::wrap(text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let after = |id: &str, phrase: &str| {
            let text = script.text(id).unwrap();
            let at = text[..text.find(phrase).unwrap()].chars().count() as f64;
            len(id) * (1.0 - at / text.chars().count() as f64)
        };
        let up = after("03_5", "Two in a row") - CARD_IN + voice.line_gap("03_5", "03_6") + len("03_6") - CARD_OUT;
        assert!(read_s(CARD_F) <= up, "{:.2} s to read, {up:.2} s up", read_s(CARD_F));
    }

    /// The shortcuts' panel stays right of the battery and inside the
    /// title-safe area, its rows apart, the countdown under the drawings
    /// and above the frame's safe bottom.
    #[test]
    fn the_panel_keeps_clear_of_the_circuit() {
        const { assert!(PANEL_X - DEMO_HALF - 0.4 > CIRCUIT_AT[0] + BATTERY_X + 0.3) };
        const { assert!(PANEL_X + DEMO_HALF + 0.3 < FRAME_X_RADIUS - 0.25) };
        const { assert!(HEADING_Y + 0.2 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(DEMO_Y[0] + DEMO_LABEL_DY - 0.25 > DEMO_Y[1] + DEMO_BOW + 0.2) };
        const { assert!(DEMO_Y[1] + DEMO_LABEL_DY - 0.25 > COUNT_Y[0] + 0.2) };
        const { assert!(THINK_AT[1] + THINK_R < DEMO_Y[1] + DEMO_LABEL_DY - 0.25) };
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 0.15) };
        // the circuit's right value clears the battery
        assert!(CIRCUIT_AT[0] + SPOTS[RIGHT][0] + LABEL_GAP + 0.3 < CIRCUIT_AT[0] + BATTERY_X);
    }
}
