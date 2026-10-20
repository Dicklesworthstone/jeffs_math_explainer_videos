//! Chapter 02, why each junction is an average: the circuit where chapter
//! 01 left it, and the left junction's four wires (02_1); each wire's drop,
//! and which way its current flows (02_2); in equals out (02_3); the same
//! balance with the voltage unknown, gathered into the average (02_4); the
//! walker on the same wires, and the prediction (02_5); a first step up has
//! chance one half, the pair being two of the four wires (02_6); the
//! first-step equation, a quarter for each wire, the same average (02_7);
//! the general rule, each far end weighted by its wire's conductance, the
//! walker choosing wires in that proportion (02_8).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_GAP, LABEL_SIZE, Picture, arrowheads, battery, equation, fade_in_all, fade_in_lagged,
    fade_out_all, fail, flash_all, flat, link_spot, move_all, pause, picture, recolor, say, small_tex, swap, tex,
    think, tokens, tokens_at, value_texts, volts_text, wait_until,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Q, RIGHT, TOP};
use crate::palette;

pub struct WhyAnAverage {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const VOLTAGE: &str = "voltage";
const CHANCE: &str = "chance of reaching the top first";
const UNKNOWN: &str = "?";
const VOLTAGE_SYMBOL: &str = "v";

/// 02_1's exact version: Ohm's law and its scope.
const OHM: &str = "Ohm's law, for ideal resistors carrying steady currents: a wire's current is the voltage drop \
                   across it divided by its resistance. Identical wires share one resistance, so every current \
                   here is its drop over the same resistance.";

/// 02_3's exact version: Kirchhoff's current law, and why the drops
/// balance as the currents do.
const KIRCHHOFF: &str = "Kirchhoff's current law: with steady currents no charge builds up at a junction, so \
                         the currents into it add to zero. Each current is its drop over the same resistance, \
                         so the drops balance too.";

/// 02_7's exact version: why the walker's chance after one step is that
/// junction's own.
const FRESH: &str = "after its first step the walker starts afresh from where it landed, with no memory of how it \
                     got there, so its chance from there is that junction's own (the Markov property).";

/// 02_8: the general rule in both pictures, and what its letters mean.
const GENERAL_V: &str = r"v(x) = \sum_y \frac{c(x,y)}{c(x)}\, v(y)";
const GENERAL_P: &str = r"p(x) = \sum_y \frac{c(x,y)}{c(x)}\, p(y)";
/// The general rule's domain, three short lines under it (text does not
/// wrap): it holds at middle junctions only, summing over the junctions
/// wired to x, where every c(x, y) is positive. At the top the same
/// average would be `R-X5.top_average`, not the top's 1.
const GENERAL_NOTE: [&str; 3] = [
    "x: any middle junction (the top and the bottom stay fixed)",
    "y: each junction wired to x; c(x, y): their wires' conductance, positive",
    "(one over resistance, side-by-side wires added); c(x): their sum at x",
];
const NOTE_LEADING: f64 = 0.32;

/// Where the picture stands (where chapter 01 leaves the circuit), and the
/// derivation's rows on the right, spaced for what 02_8 stacks there: the
/// two averages and the two sums stand about 0.9 tall, the weights about
/// 0.45, and each pair of neighbours keeps about 0.2 between them.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const PANEL_X: f64 = 3.2;
const ROW_Y: [f64; 6] = [3.0, 1.96, 1.13, 0.29, -0.81, -1.55];
const ROW_SIZE: f64 = 34.0;
/// The prediction's countdown, in the panel under the voltage's row.
const THINK_AT: [f64; 2] = [3.2, 0.2];
/// The walker's size; how far along its wire a drop is written; how far
/// the pair's merged chance and the links' conductances sit from them.
const WALKER_R: f64 = 0.11;
const DROP_T: f64 = 0.45;
const PAIR_GAP: f64 = -0.45;
const ACROSS_GAP: f64 = 0.3;
const DOWN_GAP: f64 = -0.35;

/// "a + b + c" in TeX, each term inside a larger fraction.
fn terms(values: &[Q]) -> String {
    values.iter().map(|&q| small_tex(q)).collect::<Vec<_>>().join(" + ")
}

/// 02_3: the drops that flow in (or out), added, in volts. They are drops,
/// not currents: each current is its drop over the shared resistance.
fn flow_row(direction: &str, values: &[Q]) -> String {
    format!(
        r"\text{{drops {direction}: }} {} = {}\ \text{{V}}",
        terms(values),
        small_tex(facts::sum(values))
    )
}

/// 02_7: where the right junction's chance comes from, before the
/// first-step equation uses it.
fn known_row(right: Q) -> String {
    format!(r"p(\text{{right}}) = {}\ \text{{(from chapter 01)}}", small_tex(right))
}

/// 02_4: the balance with the voltage unknown, each wire's drop the far
/// end's voltage minus `v`.
fn balance_row(ends: &[Q]) -> String {
    let drops: Vec<String> = ends.iter().map(|&q| format!("({} - v)", small_tex(q))).collect();
    format!("{} = 0", drops.join(" + "))
}

/// 02_4: the unknown gathered, one `v` from each wire.
fn gathered_row(ends: &[Q]) -> String {
    format!("{}v = {}", ends.len(), terms(ends))
}

/// 02_6: the walker's first step up, wires counted.
fn up_row(up: usize, wires: usize, chance: Q) -> String {
    format!(
        r"\text{{first step up: }} \tfrac{{{up}}}{{{wires}}} = {}",
        small_tex(chance)
    )
}

/// 02_7: the first-step equation, a `1/wires` chance for each wire times
/// the chance from its far end.
fn expand_row(ends: &[Q]) -> String {
    let n = ends.len();
    let parts: Vec<String> = ends
        .iter()
        .map(|&q| format!(r"\tfrac{{1}}{{{n}}} \cdot {}", small_tex(q)))
        .collect();
    format!("p = {}", parts.join(" + "))
}

/// 02_8: each link's conductance over the junction's total, unreduced.
fn weights_row(conductances: &[Q], total: Q) -> String {
    let parts: Vec<String> = conductances
        .iter()
        .map(|&c| format!(r"\tfrac{{{}}}{{{}}}", tex(c), tex(total)))
        .collect();
    format!(r"\text{{weights: }} {}", parts.join(r",\ "))
}

/// 02_6's exact version, from the facts: the pair counted as one wire is
/// another network, the same in both pictures.
fn pair_card() -> Result<String, String> {
    let (up, voltages, chances) = facts::single_pair()?;
    if voltages != chances {
        return Err(format!(
            "the pair counted once gives voltages {voltages:?} and chances {chances:?}"
        ));
    }
    Ok(format!(
        "counting the pair as one wire makes another network: there the first step goes up with chance {up}, \
         and the left junction sits at {}, as a voltage and as a chance.",
        voltages[LEFT]
    ))
}

/// A middle junction's value written beside it. Detached.
fn value_label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x, END_LABEL_GAP))
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

impl SceneConstruct for WhyAnAverage {
    fn name(&self) -> &str {
        "02_why_an_average"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let chances = facts::chances().map_err(fail)?;
        let ends = net.far_ends(LEFT).map_err(fail)?;
        let volt_ends = facts::far_values(&volts, LEFT).map_err(fail)?;
        let chance_ends = facts::far_values(&chances, LEFT).map_err(fail)?;
        let drops = facts::drops().map_err(fail)?;
        let (up, wires) = facts::first_step_up().map_err(fail)?;
        let to_top = Q::int(up as i128) / Q::int(wires as i128);

        // 02_1: the circuit as chapter 01 left it; the left junction's four
        // wires, the others faint; Ohm's law
        let line = say(&kit, stage, "02_1")?;
        let colors: Vec<Srgb> = (0..net.size())
            .map(|x| match x {
                TOP => palette::top(),
                BOTTOM => palette::bottom(),
                _ => palette::value(volts[x]),
            })
            .collect();
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
        let left_value = value_label(&kit, stage, &pic, LEFT, &tex(volts[LEFT]), Palette::ink())?;
        let right_value = value_label(&kit, stage, &pic, RIGHT, &tex(volts[RIGHT]), Palette::ink())?;
        let circuit_name = kit.text(stage, VOLTAGE, 26.0, Palette::dim(), pic.name_spot())?;
        let mut arriving = pic.mobs();
        arriving.extend(&cell);
        arriving.extend([left_value, right_value, circuit_name]);
        fade_in_all(stage, &arriving, 0.6)?;
        let left_wires = pic.wires_at(LEFT);
        let others: Vec<Mob> = pic
            .strands
            .iter()
            .filter(|s| s.a != LEFT && s.b != LEFT)
            .map(|s| s.mob)
            .collect();
        let dimmed: Vec<(Mob, Srgb)> = others.iter().map(|&m| (m, Palette::faint())).collect();
        recolor(stage, &dimmed, 0.4)?;
        flash_all(stage, &left_wires, 0.6)?;
        line.wait_for(stage, "shared resistance")?;
        let ohm = precisely(&kit, stage, OHM)?;
        fade_in_all(stage, &[ohm], 0.4)?;
        let ohm_read_until = stage.scene().time().to_f64() + OHM.chars().count() as f64 / 15.0;

        // 02_2: each wire's drop, and the way its current flows: in down the
        // pair, out to the right and to the bottom
        wait_until(stage, ohm_read_until)?;
        let line = say(&kit, stage, "02_2")?;
        fade_out_all(stage, &[ohm], 0.3)?;
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

        // 02_3: in equals out, the drops added; Kirchhoff's current law
        let line = say(&kit, stage, "02_3")?;
        let into: Vec<Q> = drops.iter().copied().filter(|d| d.is_positive()).collect();
        let out: Vec<Q> = drops.iter().filter(|d| !d.is_positive()).map(|&d| -d).collect();
        let kirchhoff = precisely(&kit, stage, KIRCHHOFF)?;
        fade_in_all(stage, &[kirchhoff], 0.4)?;
        let kirchhoff_read_until = stage.scene().time().to_f64() + KIRCHHOFF.chars().count() as f64 / 15.0;
        line.wait_for(stage, "Two fifths and two fifths in")?;
        let in_row = row(&kit, stage, &flow_row("in", &into), 0)?;
        fade_in_all(stage, &[in_row], 0.4)?;
        line.wait_for(stage, "one fifth and three fifths out")?;
        let out_row = row(&kit, stage, &flow_row("out", &out), 1)?;
        fade_in_all(stage, &[out_row], 0.4)?;

        // 02_4: the voltage unknown; the balance, the unknown gathered, the
        // average; the working goes and v's equation rises to the top row
        wait_until(stage, kirchhoff_read_until)?;
        let line = say(&kit, stage, "02_4")?;
        let unknown_v = value_label(&kit, stage, &pic, LEFT, VOLTAGE_SYMBOL, Palette::ink())?;
        fade_out_all(stage, &[kirchhoff, left_value], 0.15)?;
        fade_in_all(stage, &[unknown_v], 0.15)?;
        let balanced = row(&kit, stage, &balance_row(&volt_ends), 2)?;
        fade_in_all(stage, &[balanced], 0.4)?;
        line.wait_for(stage, "four times it")?;
        let gathered = row(&kit, stage, &gathered_row(&volt_ends), 3)?;
        fade_in_all(stage, &[gathered], 0.4)?;
        line.wait_for(stage, "their average")?;
        let solved = row(&kit, stage, &format!("v = {}", equation(&volt_ends, volts[LEFT])), 4)?;
        fade_in_all(stage, &[solved], 0.4)?;
        let mut working = vec![in_row, out_row, balanced, gathered];
        working.extend(&drop_labels);
        working.extend(&arrows);
        fade_out_all(stage, &working, 0.4)?;
        let restored: Vec<(Mob, Srgb)> = others.iter().map(|&m| (m, palette::wire())).collect();
        recolor(stage, &restored, 0.3)?;
        move_all(stage, &[(solved, v(PANEL_X, ROW_Y[0]))], 0.6)?;

        // 02_5: the same picture as the walk; the walker and its four wires;
        // nothing shows a step's chance through the countdown
        let line = say(&kit, stage, "02_5")?;
        let walk_name = kit.text(stage, CHANCE, 26.0, Palette::dim(), pic.name_spot())?;
        let top_chance = kit.tex(
            stage,
            &tex(chances[TOP]),
            LABEL_SIZE,
            Palette::ink(),
            &[],
            pic.label_spot(TOP, LABEL_GAP),
        )?;
        let bottom_chance = kit.tex(
            stage,
            &tex(chances[BOTTOM]),
            LABEL_SIZE,
            Palette::ink(),
            &[],
            pic.label_spot(BOTTOM, LABEL_GAP),
        )?;
        let left_q = value_label(&kit, stage, &pic, LEFT, UNKNOWN, Palette::hero())?;
        let mut leaving = cell;
        leaving.extend([circuit_name, unknown_v]);
        fade_out_all(stage, &leaving, 0.25)?;
        fade_in_all(stage, &[walk_name, top_chance, bottom_chance, left_q], 0.25)?;
        recolor(stage, &[(pic.junctions[LEFT], palette::unknown())], 0.3)?;
        let dot = Circle::new()
            .radius(WALKER_R)
            .style(flat(palette::walker(), 1.0))
            .build()
            .moved_to(pic.spot(LEFT));
        let dot = stage.arena_mut().add(dot);
        fade_in_all(stage, &[dot], 0.3)?;
        line.wait_for(stage, "each equally likely")?;
        flash_all(stage, &left_wires, 0.6)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 02_6: a quarter for each wire; the pair's two quarters make one
        // half; the pair counted once is another network
        let line = say(&kit, stage, "02_6")?;
        let each = vec![(small_tex(Q::ONE / Q::int(wires as i128)), palette::walker()); wires];
        let quarters = tokens(&kit, stage, &pic, LEFT, &each)?;
        swap(stage, &[countdown], &quarters, 0.3)?;
        line.wait_for(stage, "The pair is two")?;
        let pair_spot = link_spot(&pic, TOP, LEFT, PAIR_GAP);
        let pair_quarters = where_end(&ends, &quarters, TOP);
        let moves: Vec<(Mob, Vec3)> = pair_quarters.iter().map(|&m| (m, pair_spot)).collect();
        move_all(stage, &moves, 0.5)?;
        let half = kit.tex(stage, &small_tex(to_top), 30.0, palette::walker(), &[], pair_spot)?;
        let up_line = row(&kit, stage, &up_row(up, wires, to_top), 1)?;
        fade_out_all(stage, &pair_quarters, 0.15)?;
        fade_in_all(stage, &[half, up_line], 0.15)?;
        line.wait_for(stage, "like its two currents")?;
        let pair_text = pair_card().map_err(fail)?;
        let pair_note = precisely(&kit, stage, &pair_text)?;
        fade_in_all(stage, &[pair_note], 0.4)?;
        let pair_read_until = stage.scene().time().to_f64() + pair_text.chars().count() as f64 / 15.0;

        // 02_7: the far ends' chances; the walker starts afresh; a quarter of
        // each, added, the same average as v's
        wait_until(stage, pair_read_until)?;
        let line = say(&kit, stage, "02_7")?;
        let far_chances = tokens(&kit, stage, &pic, LEFT, &value_texts(&chance_ends))?;
        let mut gone = vec![pair_note, half];
        for y in [RIGHT, BOTTOM] {
            gone.extend(where_end(&ends, &quarters, y));
        }
        fade_out_all(stage, &gone, 0.2)?;
        fade_in_all(stage, &far_chances, 0.2)?;
        line.wait_for(stage, "Right, its chance")?;
        let fresh = precisely(&kit, stage, FRESH)?;
        let known = row(&kit, stage, &known_row(chances[RIGHT]), 2)?;
        fade_in_all(stage, &[fresh, known], 0.4)?;
        let fresh_read_until = stage.scene().time().to_f64() + FRESH.chars().count() as f64 / 15.0;
        flash_all(stage, &[right_value], 0.5)?;
        line.wait_for(stage, "A quarter of each")?;
        let expanded = row(&kit, stage, &expand_row(&chance_ends), 3)?;
        fade_in_all(stage, &[expanded], 0.4)?;
        let p_line = row(
            &kit,
            stage,
            &format!("p = {}", equation(&chance_ends, chances[LEFT])),
            4,
        )?;
        let left_p = value_label(&kit, stage, &pic, LEFT, &tex(chances[LEFT]), Palette::ink())?;
        fade_out_all(stage, &[left_q], 0.2)?;
        fade_in_all(stage, &[p_line, left_p], 0.2)?;
        recolor(stage, &[(pic.junctions[LEFT], palette::value(chances[LEFT]))], 0.3)?;
        flash_all(stage, &[solved, p_line], 0.6)?;

        // 02_8: conductances on the left's links, their weights, and the rule
        // for any positive conductances, in both pictures
        wait_until(stage, fresh_read_until)?;
        let line = say(&kit, stage, "02_8")?;
        let conductances = facts::left_conductances().map_err(fail)?;
        let mut labels = Vec::new();
        for (&(y, gap), &c) in [(TOP, PAIR_GAP), (RIGHT, ACROSS_GAP), (BOTTOM, DOWN_GAP)]
            .iter()
            .zip(&conductances)
        {
            let (a, b) = if y == TOP { (TOP, LEFT) } else { (LEFT, y) };
            labels.push(kit.tex(
                stage,
                &format!("c = {}", tex(c)),
                28.0,
                Palette::ink(),
                &[],
                link_spot(&pic, a, b, gap),
            )?);
        }
        let mut clearing = vec![fresh, up_line, known, expanded];
        clearing.extend(&far_chances);
        clearing.push(dot);
        fade_out_all(stage, &clearing, 0.2)?;
        fade_in_all(stage, &labels, 0.2)?;
        move_all(stage, &[(p_line, v(PANEL_X, ROW_Y[1]))], 0.5)?;
        let weights = row(&kit, stage, &weights_row(&conductances, net.total(LEFT)), 2)?;
        let general_v = row(&kit, stage, GENERAL_V, 3)?;
        let general_p = row(&kit, stage, GENERAL_P, 4)?;
        let mut general = vec![general_v, general_p];
        for (i, text) in GENERAL_NOTE.iter().enumerate() {
            let y = ROW_Y[5] - NOTE_LEADING * i as f64;
            general.push(kit.text(stage, text, 18.0, Palette::dim(), v(PANEL_X, y))?);
        }
        line.wait_for(stage, "weight each far end")?;
        fade_in_all(stage, &[weights], 0.4)?;
        line.wait_for(stage, "the walker chooses")?;
        fade_in_lagged(stage, &general, 0.9, 0.4)?;
        let general_read_until = stage.scene().time().to_f64()
            + GENERAL_NOTE.iter().map(|text| text.chars().count()).sum::<usize>() as f64 / 15.0;
        line.wait_for(stage, "The same rule")?;
        flash_all(stage, &[general_v, general_p], 0.6)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, 0.6)?;
        wait_until(stage, general_read_until)?;

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
            ("02_1", "shared resistance"),
            ("02_2", "Then one fifth"),
            ("02_3", "Two fifths and two fifths in"),
            ("02_3", "one fifth and three fifths out"),
            ("02_4", "four times it"),
            ("02_4", "their average"),
            ("02_5", "each equally likely"),
            ("02_6", "The pair is two"),
            ("02_6", "like its two currents"),
            ("02_7", "Right, its chance"),
            ("02_7", "A quarter of each"),
            ("02_8", "weight each far end"),
            ("02_8", "the walker chooses"),
            ("02_8", "The same rule"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The derivation's rows are the facts' numbers: the drops in and out,
    /// the balance with the unknown, the gathered unknown, the walker's
    /// step and terms, the weights; signs as the currents flow.
    #[test]
    fn the_rows_are_built_from_the_facts() {
        let volts = facts::voltages().unwrap();
        let chances = facts::chances().unwrap();
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
        // the right's chance is named as chapter 01's before it is used
        assert_eq!(
            known_row(chances[RIGHT]),
            r"p(\text{right}) = \tfrac{2}{5}\ \text{(from chapter 01)}"
        );
        let ends = facts::far_values(&volts, LEFT).unwrap();
        assert_eq!(
            balance_row(&ends),
            r"(1 - v) + (1 - v) + (\tfrac{2}{5} - v) + (0 - v) = 0"
        );
        assert_eq!(gathered_row(&ends), r"4v = 1 + 1 + \tfrac{2}{5} + 0");
        let (up, wires) = facts::first_step_up().unwrap();
        assert_eq!(
            up_row(up, wires, Q::new(1, 2)),
            r"\text{first step up: } \tfrac{2}{4} = \tfrac{1}{2}"
        );
        let chance_ends = facts::far_values(&chances, LEFT).unwrap();
        assert_eq!(
            expand_row(&chance_ends),
            r"p = \tfrac{1}{4} \cdot 1 + \tfrac{1}{4} \cdot 1 + \tfrac{1}{4} \cdot \tfrac{2}{5} + \tfrac{1}{4} \cdot 0"
        );
        let net = facts::network().unwrap();
        assert_eq!(
            weights_row(&facts::left_conductances().unwrap(), net.total(LEFT)),
            r"\text{weights: } \tfrac{2}{4},\ \tfrac{1}{4},\ \tfrac{1}{4}"
        );
        // the two pictures' final equations are one fraction
        assert_eq!(equation(&ends, volts[LEFT]), equation(&chance_ends, chances[LEFT]));
    }

    /// The exact versions fit a lower-third of three lines, and the pair's
    /// is built from the facts.
    #[test]
    fn the_exact_versions_are_built_from_the_facts_and_fit() {
        let pair = pair_card().unwrap();
        assert!(pair.contains("goes up with chance 1/3"), "{pair}");
        assert!(pair.contains("sits at 5/11"), "{pair}");
        for text in [OHM.to_string(), KIRCHHOFF.to_string(), FRESH.to_string(), pair] {
            let lines = explainer_kit::animatic::wrap(&text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
        // the dim note under the general rule is set line by line, unwrapped:
        // each line at most 72 characters, what the panel holds at 18 pt
        for text in GENERAL_NOTE {
            assert!(text.chars().count() <= 72, "{text}");
        }
        // and it states the rule's domain: middle junctions, wired neighbours
        assert!(GENERAL_NOTE[0].contains("any middle junction"));
        assert!(GENERAL_NOTE[1].contains("each junction wired to x"));
    }

    /// The general rule is an average at middle junctions only: at the top
    /// the same weighted average is not the top's value.
    #[test]
    fn the_general_rule_does_not_hold_at_the_top() {
        let net = facts::network().unwrap();
        let volts = facts::voltages().unwrap();
        assert_eq!(net.average(&volts, TOP), Some(Q::new(8, 15)));
        assert_ne!(net.average(&volts, TOP), Some(volts[TOP]));
        for x in net.middle() {
            assert_eq!(net.average(&volts, x), Some(volts[x]));
        }
    }

    /// The panel's rows stay right of the picture's battery, inside the
    /// title-safe area, and above a three-line lower-third, and its tall
    /// rows never touch.
    #[test]
    fn the_panel_keeps_clear_of_the_picture_and_the_card() {
        use crate::chapters::draw::{BATTERY_X, THINK_R};
        // the widest row is about 5.6 wide at 34 pt: it stays right of the
        // battery and inside the title-safe edge
        const { assert!(PANEL_X - 2.8 > PICTURE_AT[0] + BATTERY_X + 0.3) };
        const { assert!(PANEL_X + 2.8 < FRAME_X_RADIUS - 0.25) };
        // the top row under the frame's safe top; the note's third line
        // above a three-line lower-third (1.58 above the frame's bottom),
        // though no lower-third is shown while the note is
        const { assert!(ROW_Y[0] + 0.45 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW_Y[5] - 2.0 * NOTE_LEADING - 0.15 > -FRAME_Y_RADIUS + 1.58) };
        // the two averages (rows 0 and 1) and the two sums (rows 3 and 4)
        // each reach about 0.45 from their centre: a gap stays between them
        const { assert!(ROW_Y[0] - ROW_Y[1] > 2.0 * 0.45 + 0.1) };
        const { assert!(ROW_Y[3] - ROW_Y[4] > 2.0 * 0.45 + 0.1) };
        // the countdown sits under the top row, right of the battery
        const { assert!(THINK_AT[1] + THINK_R + 0.4 < ROW_Y[0]) };
        const { assert!(THINK_AT[0] - THINK_R > PICTURE_AT[0] + BATTERY_X + 0.3) };
    }
}
