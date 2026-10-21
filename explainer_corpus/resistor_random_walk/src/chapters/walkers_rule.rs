//! Chapter 06, the walker's rule at both junctions: the same network as the
//! walk, both middle junctions unknown; the walker's first step, and the
//! prediction (06_1); one half, the pair being two of the four wires
//! (06_2); the left's first-step equation, a quarter for each wire, in
//! letters (06_3); the right's, its pair down counted as one half (06_4);
//! times four (06_5), the circuit's two balances letter for letter (06_6).
//! 06_1 to 06_3 are production chapter 02's 02_5 to 02_7, their pictures
//! kept but for 06_3's rows, now in letters (and its fresh-start card
//! dropped: chapter 01's card B gives it).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_GAP, LABEL_SIZE, Picture, fade_in_all, fade_out_all, fail, flash_all, link_spot, move_all,
    pause, picture, say, small_tex, swap, tex, think, tokens, value_texts, wait_until, walker_dot,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

pub struct WalkersRule {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const CHANCE: &str = "chance of reaching the top first";
const UNKNOWN: &str = "?";
/// The two middle junctions' chances, as letters (the circuit's voltages
/// are L and R in chapter 05).
const LEFT_CHANCE: &str = "p_L";
const RIGHT_CHANCE: &str = "p_R";
const LEFT_VOLTAGE: &str = "L";
const RIGHT_VOLTAGE: &str = "R";

/// Where the picture stands, and the rows on the right: the first three
/// across the panel, the last two in two columns, the walk's and the
/// circuit's side by side.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const PANEL_X: f64 = 3.2;
const COLUMN_X: [f64; 2] = [2.3, 5.3];
const ROW_Y: [f64; 5] = [3.0, 2.0, 1.0, -0.15, -1.05];
const ROW_SIZE: f64 = 34.0;
/// The prediction's countdown, in the panel.
const THINK_AT: [f64; 2] = [3.2, 0.2];
/// The walker's size, and how far the merged chances of a pair sit from it.
const WALKER_R: f64 = 0.11;
const PAIR_GAP: f64 = -0.45;
/// A lower-third's fade in, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// 06_2: the walker's first step up, wires counted.
fn up_row(up: usize, wires: usize, chance: Q) -> String {
    format!(
        r"\text{{first step up: }} \tfrac{{{up}}}{{{wires}}} = {}",
        small_tex(chance)
    )
}

/// A far end's chance in a first-step equation: the top's and the
/// bottom's as numbers, a middle junction's by its letter.
fn chance_symbol(chances: &[Q], y: usize) -> String {
    match y {
        LEFT => LEFT_CHANCE.to_owned(),
        RIGHT => RIGHT_CHANCE.to_owned(),
        _ => small_tex(chances[y]),
    }
}

/// 06_3: the left's first-step equation, one term per wire, each a
/// `1/wires` chance times the chance from its far end.
fn left_row(net: &Network, chances: &[Q]) -> Result<String, String> {
    let ends = net.far_ends(LEFT)?;
    let n = ends.len();
    let parts: Vec<String> = ends
        .iter()
        .map(|&y| format!(r"\tfrac{{1}}{{{n}}} \cdot {}", chance_symbol(chances, y)))
        .collect();
    Ok(format!("{LEFT_CHANCE} = {}", parts.join(" + ")))
}

/// 06_4: the right's, one term per far junction (its pair down as one
/// half), each that link's share of the junction's wires.
fn right_row(net: &Network, chances: &[Q]) -> String {
    let to = [TOP, LEFT, BOTTOM];
    let steps = facts::steps(net, RIGHT, &to);
    let parts: Vec<String> = to
        .iter()
        .zip(&steps)
        .map(|(&y, &s)| format!(r"{} \cdot {}", small_tex(s), chance_symbol(chances, y)))
        .collect();
    format!("{RIGHT_CHANCE} = {}", parts.join(" + "))
}

/// 06_5, 06_6: a balance times the junction's wires, `a x = c + x'`: the
/// fixed far ends added, the other middle junction's value once (its one
/// wire).
fn times_row(net: &Network, x: usize, name: &str, other: &str) -> String {
    let fixed = net.between(x, TOP) * facts::BATTERY[0] + net.between(x, BOTTOM) * facts::BATTERY[1];
    format!("{}{name} = {} + {other}", tex(net.total(x)), tex(fixed))
}

/// 06_2's exact version, from the facts: the pair counted as one wire is
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

/// A junction's label beside it. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str, color: Srgb, gap: f64) -> fmn::Result<Mob> {
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x, gap))
}

/// A row of the panel at `x`, height `ROW_Y[i]`. Detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, x: f64, i: usize) -> fmn::Result<Mob> {
    kit.tex(stage, src, ROW_SIZE, Palette::ink(), &[], v(x, ROW_Y[i]))
}

/// The mobs at the positions where `ends` holds `y`.
fn where_end(ends: &[usize], mobs: &[Mob], y: usize) -> Vec<Mob> {
    ends.iter()
        .zip(mobs)
        .filter(|&(&e, _)| e == y)
        .map(|(_, &m)| m)
        .collect()
}

impl SceneConstruct for WalkersRule {
    fn name(&self) -> &str {
        "06_the_walkers_rule"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let chances = facts::chances().map_err(fail)?;
        let ends = net.far_ends(LEFT).map_err(fail)?;
        let right_ends = net.far_ends(RIGHT).map_err(fail)?;
        let (up, wires) = facts::first_step_up().map_err(fail)?;
        let to_top = Q::int(up as i128) / Q::int(wires as i128);

        // the network as the walk: the top 1, the bottom 0, both middle
        // junctions unknown
        let colors: Vec<Srgb> = (0..net.size())
            .map(|x| match x {
                TOP => palette::top(),
                BOTTOM => palette::bottom(),
                _ => palette::unknown(),
            })
            .collect();
        let pic = picture(stage, &net, PICTURE_AT, &colors)?;
        let name = kit.text(stage, CHANCE, 26.0, Palette::dim(), pic.name_spot())?;
        let top_chance = label(&kit, stage, &pic, TOP, &tex(chances[TOP]), Palette::ink(), LABEL_GAP)?;
        let bottom_chance = label(&kit, stage, &pic, BOTTOM, &tex(chances[BOTTOM]), Palette::ink(), LABEL_GAP)?;
        let left_q = label(&kit, stage, &pic, LEFT, UNKNOWN, Palette::hero(), END_LABEL_GAP)?;
        let right_q = label(&kit, stage, &pic, RIGHT, UNKNOWN, Palette::hero(), END_LABEL_GAP)?;
        let mut arriving = pic.mobs();
        arriving.extend([name, top_chance, bottom_chance, left_q, right_q]);
        fade_in_all(stage, &arriving, 0.6)?;

        // 06_1: the walker and its four wires; nothing shows a step's chance
        // through the countdown
        let line = say(&kit, stage, "06_1")?;
        let dot = walker_dot(stage, pic.spot(LEFT), WALKER_R, 1.0);
        fade_in_all(stage, &[dot], 0.3)?;
        line.wait_for(stage, "each equally likely")?;
        let left_wires = pic.wires_at(LEFT);
        flash_all(stage, &left_wires, 0.6)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 06_2: a quarter for each wire; the pair's two quarters make one
        // half; the pair counted once is another network
        let line = say(&kit, stage, "06_2")?;
        let each = vec![(small_tex(Q::ONE / Q::int(wires as i128)), palette::walker()); wires];
        let quarters = tokens(&kit, stage, &pic, LEFT, &each)?;
        swap(stage, &[countdown], &quarters, 0.3)?;
        line.wait_for(stage, "The pair is two")?;
        let pair_spot = link_spot(&pic, TOP, LEFT, PAIR_GAP);
        let pair_quarters = where_end(&ends, &quarters, TOP);
        let moves: Vec<(Mob, Vec3)> = pair_quarters.iter().map(|&m| (m, pair_spot)).collect();
        move_all(stage, &moves, 0.5)?;
        let half = kit.tex(stage, &small_tex(to_top), 30.0, palette::walker(), &[], pair_spot)?;
        let up_line = row(&kit, stage, &up_row(up, wires, to_top), PANEL_X, 0)?;
        fade_out_all(stage, &pair_quarters, 0.15)?;
        fade_in_all(stage, &[half, up_line], 0.15)?;
        line.wait_for(stage, "like its two currents")?;
        let pair_text = pair_card().map_err(fail)?;
        let pair_note = precisely(&kit, stage, &pair_text)?;
        fade_in_all(stage, &[pair_note], CARD_IN)?;
        let pair_read = stage.scene().time().to_f64() + read_s(&pair_text);

        // 06_3: the far ends' chances, the right's a letter; a quarter of
        // each, added, the left's first-step equation
        wait_until(stage, pair_read)?;
        let line = say(&kit, stage, "06_3")?;
        let far: Vec<(String, Srgb)> = ends
            .iter()
            .map(|&y| {
                if net.is_end(y) {
                    value_texts(&[chances[y]])[0].clone()
                } else {
                    (chance_symbol(&chances, y), Palette::ink())
                }
            })
            .collect();
        let far_chances = tokens(&kit, stage, &pic, LEFT, &far)?;
        let mut gone = vec![pair_note, half];
        for y in [RIGHT, BOTTOM] {
            gone.extend(where_end(&ends, &quarters, y));
        }
        fade_out_all(stage, &gone, 0.2)?;
        fade_in_all(stage, &far_chances, 0.2)?;
        line.wait_for(stage, "Right, its chance")?;
        flash_all(stage, &[right_q], 0.5)?;
        line.wait_for(stage, "A quarter of each")?;
        let left_equation = row(&kit, stage, &left_row(&net, &chances).map_err(fail)?, PANEL_X, 1)?;
        fade_in_all(stage, &[left_equation], 0.4)?;

        // 06_4: from the right a quarter up, a quarter to the left, and the
        // pair down's two quarters one half; the right's equation
        let line = say(&kit, stage, "06_4")?;
        let dot_right = walker_dot(stage, pic.spot(RIGHT), WALKER_R, 1.0);
        let right_quarters = tokens(&kit, stage, &pic, RIGHT, &each)?;
        let mut leaving = far_chances.clone();
        leaving.push(dot);
        fade_out_all(stage, &leaving, 0.2)?;
        let mut coming = right_quarters.clone();
        coming.push(dot_right);
        fade_in_all(stage, &coming, 0.3)?;
        line.wait_for(stage, "down with a half")?;
        let down_spot = link_spot(&pic, RIGHT, BOTTOM, -PAIR_GAP);
        let down_quarters = where_end(&right_ends, &right_quarters, BOTTOM);
        let moves: Vec<(Mob, Vec3)> = down_quarters.iter().map(|&m| (m, down_spot)).collect();
        move_all(stage, &moves, 0.5)?;
        let down_half = kit.tex(
            stage,
            &small_tex(facts::steps(&net, RIGHT, &[BOTTOM])[0]),
            30.0,
            palette::walker(),
            &[],
            down_spot,
        )?;
        fade_out_all(stage, &down_quarters, 0.15)?;
        fade_in_all(stage, &[down_half], 0.15)?;
        line.wait_for(stage, "So its chance")?;
        let right_equation = row(&kit, stage, &right_row(&net, &chances), PANEL_X, 2)?;
        fade_in_all(stage, &[right_equation], 0.4)?;

        // 06_5: times four, both equations
        let line = say(&kit, stage, "06_5")?;
        line.wait_for(stage, "four times the left is")?;
        let walk_left = row(
            &kit,
            stage,
            &times_row(&net, LEFT, LEFT_CHANCE, RIGHT_CHANCE),
            COLUMN_X[0],
            3,
        )?;
        fade_in_all(stage, &[walk_left], 0.4)?;
        line.wait_for(stage, "four times the right is")?;
        let walk_right = row(
            &kit,
            stage,
            &times_row(&net, RIGHT, RIGHT_CHANCE, LEFT_CHANCE),
            COLUMN_X[0],
            4,
        )?;
        fade_in_all(stage, &[walk_right], 0.4)?;

        // 06_6: the circuit's two balances beside them, letter for letter
        let line = say(&kit, stage, "06_6")?;
        let circuit_left = row(
            &kit,
            stage,
            &times_row(&net, LEFT, LEFT_VOLTAGE, RIGHT_VOLTAGE),
            COLUMN_X[1],
            3,
        )?;
        let circuit_right = row(
            &kit,
            stage,
            &times_row(&net, RIGHT, RIGHT_VOLTAGE, LEFT_VOLTAGE),
            COLUMN_X[1],
            4,
        )?;
        fade_in_all(stage, &[circuit_left, circuit_right], 0.4)?;
        line.wait_for(stage, "letter for letter")?;
        flash_all(stage, &[walk_left, circuit_left], 0.5)?;
        flash_all(stage, &[walk_right, circuit_right], 0.5)?;
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
            ("06_1", "each equally likely"),
            ("06_2", "The pair is two"),
            ("06_2", "like its two currents"),
            ("06_3", "Right, its chance"),
            ("06_3", "A quarter of each"),
            ("06_4", "down with a half"),
            ("06_4", "So its chance"),
            ("06_5", "four times the left is"),
            ("06_5", "four times the right is"),
            ("06_6", "letter for letter"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The rows are the facts': the first step up, both first-step
    /// equations in letters, and times four the circuit's balances.
    #[test]
    fn the_rows_are_built_from_the_facts() {
        let net = facts::network().unwrap();
        let chances = facts::chances().unwrap();
        let (up, wires) = facts::first_step_up().unwrap();
        assert_eq!(
            up_row(up, wires, Q::new(1, 2)),
            r"\text{first step up: } \tfrac{2}{4} = \tfrac{1}{2}"
        );
        assert_eq!(
            left_row(&net, &chances).unwrap(),
            r"p_L = \tfrac{1}{4} \cdot 1 + \tfrac{1}{4} \cdot 1 + \tfrac{1}{4} \cdot p_R + \tfrac{1}{4} \cdot 0"
        );
        assert_eq!(
            right_row(&net, &chances),
            r"p_R = \tfrac{1}{4} \cdot 1 + \tfrac{1}{4} \cdot p_L + \tfrac{1}{2} \cdot 0"
        );
        assert_eq!(times_row(&net, LEFT, LEFT_CHANCE, RIGHT_CHANCE), "4p_L = 2 + p_R");
        assert_eq!(times_row(&net, RIGHT, RIGHT_CHANCE, LEFT_CHANCE), "4p_R = 1 + p_L");
        assert_eq!(times_row(&net, LEFT, LEFT_VOLTAGE, RIGHT_VOLTAGE), "4L = 2 + R");
        assert_eq!(times_row(&net, RIGHT, RIGHT_VOLTAGE, LEFT_VOLTAGE), "4R = 1 + L");
        // the spoken numbers: the right's steps, and the walker's equations
        // the circuit's
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F13.right.steps"), Some("1/4,1/4,1/2"));
        assert_eq!(f.get("R-F13.walk_same"), Some("true"));
        // the right's chances keep its equation
        assert_eq!(
            Q::int(4) * chances[RIGHT],
            Q::ONE + chances[LEFT],
            "4 p_R = 1 + p_L at the walk's values"
        );
    }

    /// The pair's exact version is built from the facts and fits a
    /// lower-third of three lines.
    #[test]
    fn the_exact_version_is_built_from_the_facts_and_fits() {
        let pair = pair_card().unwrap();
        assert!(pair.contains("goes up with chance 1/3"), "{pair}");
        assert!(pair.contains("sits at 5/11"), "{pair}");
        assert!(explainer_kit::animatic::wrap(&pair, 84, 10).len() <= 3);
    }

    /// The panel's rows stay right of the picture and inside the title-safe
    /// area, above a three-line lower-third; the two columns stay apart.
    #[test]
    fn the_panel_keeps_clear_of_the_picture() {
        use crate::chapters::draw::{SPOTS, THINK_R};
        const { assert!(COLUMN_X[0] - 1.3 > PICTURE_AT[0] + 1.7 + LABEL_GAP + 0.5) };
        const { assert!(COLUMN_X[1] + 1.3 < FRAME_X_RADIUS - 0.25) };
        const { assert!(COLUMN_X[1] - COLUMN_X[0] > 2.7) };
        const { assert!(ROW_Y[0] + 0.45 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW_Y[4] - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        const { assert!(THINK_AT[1] + THINK_R + 0.4 < ROW_Y[0]) };
        assert!(PICTURE_AT[0] + SPOTS[RIGHT][0] + END_LABEL_GAP < COLUMN_X[0] - 1.3);
    }
}
