//! Chapter 10, unequal wires: the general rule, each far end weighted by its
//! wire's conductance, the walker choosing wires in that proportion (10_1,
//! production chapter 02's 02_8, its picture and frozen read wait kept);
//! Doyle and Snell's unequal wires drawn on the same shape (10_2); the
//! prediction, the walker's likeliest first step (10_3); across with chance
//! one half, and the two equations (10_4); seven sixteenths and three
//! eighths, as voltages and as chances (10_5).

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_GAP, LABEL_SIZE, Picture, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all,
    link_spot, outside_link_spot, pause, picture, recolor, say, small_tex, tex, think, tokens_at, wait_until,
    walker_dot, weighted_picture,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

pub struct AnyWires {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const BOTH: &str = "voltage or chance";
const SAME: &str = "voltage = chance";
/// The note under the unequal wires: the conductances are Doyle and
/// Snell's (§1.3.1 to §1.3.2), the drawing this video's shape, not their
/// figure.
const SOURCE_NOTE: &str = "their conductances, drawn on this shape: Doyle and Snell, section 1.3.2";
const LEFT_NAME: &str = "L";
const RIGHT_NAME: &str = "R";

/// 10_1: the general rule in both pictures, and what its letters mean.
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

/// Where the picture stands, and the rows on the right (production
/// chapter 02's spacing).
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const PANEL_X: f64 = 3.2;
const ROW_Y: [f64; 6] = [3.0, 1.96, 1.13, 0.29, -0.81, -1.55];
const ROW_SIZE: f64 = 34.0;
/// The prediction's countdown, in the panel.
const THINK_AT: [f64; 2] = [3.2, 0.2];
/// How far a link's conductance sits from it (outward), the left's links'
/// labels in 10_1, the walker's size, how far along a link its step chance
/// is written, and where the left's total sits.
const LINK_GAP: f64 = 0.35;
const PAIR_GAP: f64 = -0.45;
const ACROSS_GAP: f64 = 0.3;
const DOWN_GAP: f64 = -0.35;
const WALKER_R: f64 = 0.11;
const STEP_T: f64 = 0.3;
const TOTAL_DY: f64 = -0.5;
/// The beat after the last line.
const END_BEAT: f64 = 0.6;

/// 10_1: each link's conductance over the junction's total, unreduced.
fn weights_row(conductances: &[Q], total: Q) -> String {
    let parts: Vec<String> = conductances
        .iter()
        .map(|&c| format!(r"\tfrac{{{}}}{{{}}}", tex(c), tex(total)))
        .collect();
    format!(r"\text{{weights: }} {}", parts.join(r",\ "))
}

/// 10_4: middle junction `x` of `net` as its first step's terms: the top's
/// and the bottom's values as numbers, the other middle junction by its
/// letter.
fn step_row(net: &Network, values: &[Q], x: usize) -> String {
    let to = facts::across_links(x);
    let steps = facts::steps(net, x, &to);
    let parts: Vec<String> = to
        .iter()
        .zip(&steps)
        .map(|(&y, &s)| {
            let far = match y {
                LEFT => LEFT_NAME.to_owned(),
                RIGHT => RIGHT_NAME.to_owned(),
                _ => tex(values[y]),
            };
            format!(r"{} \cdot {far}", small_tex(s))
        })
        .collect();
    let name = if x == LEFT { LEFT_NAME } else { RIGHT_NAME };
    format!("{name} = {}", parts.join(" + "))
}

/// A junction's label on `pic`. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    let gap = if x == TOP || x == BOTTOM { LABEL_GAP } else { END_LABEL_GAP };
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x, gap))
}

/// A row of the panel. Detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, i: usize) -> fmn::Result<Mob> {
    kit.tex(stage, src, ROW_SIZE, Palette::ink(), &[], v(PANEL_X, ROW_Y[i]))
}

impl SceneConstruct for AnyWires {
    fn name(&self) -> &str {
        "10_any_wires"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let unequal = facts::unequal().map_err(fail)?;
        let held = unequal.kirchhoff(facts::BATTERY[0], facts::BATTERY[1]).map_err(fail)?;
        let chances = unequal.walk().map_err(fail)?.top;
        if held != chances {
            return Err(fail(format!("the unequal wires' voltages {held:?} are not its chances {chances:?}")));
        }

        // the bridge network with its values, no battery: "voltage or
        // chance"
        let colors: Vec<Srgb> = volts.iter().map(|&q| palette::value(q)).collect();
        let pic = picture(stage, &net, PICTURE_AT, &colors)?;
        let name = kit.text(stage, BOTH, 26.0, Palette::dim(), pic.name_spot())?;
        let values: Vec<Mob> = (0..net.size())
            .map(|x| label(&kit, stage, &pic, x, &tex(volts[x]), Palette::ink()))
            .collect::<fmn::Result<_>>()?;
        let mut arriving = pic.mobs();
        arriving.push(name);
        arriving.extend(&values);
        fade_in_all(stage, &arriving, 0.6)?;

        // 10_1: conductances on the left's links, their weights, and the
        // rule for any positive conductances, in both pictures
        let line = say(&kit, stage, "10_1")?;
        let conductances = facts::left_conductances().map_err(fail)?;
        let mut links = Vec::new();
        for (&(y, gap), &c) in [(TOP, PAIR_GAP), (RIGHT, ACROSS_GAP), (BOTTOM, DOWN_GAP)]
            .iter()
            .zip(&conductances)
        {
            let (a, b) = if y == TOP { (TOP, LEFT) } else { (LEFT, y) };
            links.push(kit.tex(
                stage,
                &format!("c = {}", tex(c)),
                28.0,
                Palette::ink(),
                &[],
                link_spot(&pic, a, b, gap),
            )?);
        }
        fade_in_all(stage, &links, 0.3)?;
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

        // 10_2: the same shape with Doyle and Snell's unequal wires, each
        // link one wire, heavier for more conductance; the general rule
        // stays until read
        let line = say(&kit, stage, "10_2")?;
        let mut leaving = pic.mobs();
        leaving.extend(&values);
        leaving.extend(&links);
        leaving.extend([weights, name]);
        fade_out_all(stage, &leaving, 0.3)?;
        let first: Vec<Srgb> = (0..unequal.size())
            .map(|x| match x {
                TOP => palette::top(),
                BOTTOM => palette::bottom(),
                _ => palette::unknown(),
            })
            .collect();
        let shape = weighted_picture(stage, &unequal, PICTURE_AT, &first)?;
        let shape_name = kit.text(stage, BOTH, 26.0, Palette::dim(), shape.name_spot())?;
        let ends = [
            label(&kit, stage, &shape, TOP, &tex(held[TOP]), Palette::ink())?,
            label(&kit, stage, &shape, BOTTOM, &tex(held[BOTTOM]), Palette::ink())?,
        ];
        let mut numbers = Vec::new();
        for w in unequal.wires() {
            numbers.push(kit.tex(
                stage,
                &tex(w.conductance),
                28.0,
                Palette::ink(),
                &[],
                outside_link_spot(&shape, w.a, w.b, LINK_GAP),
            )?);
        }
        let source = kit.text(stage, SOURCE_NOTE, 18.0, Palette::dim(), shape.equation_spot())?;
        let mut arriving = shape.mobs();
        arriving.extend(ends);
        arriving.extend(&numbers);
        arriving.extend([shape_name, source]);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "At the left")?;
        let left_total = kit.text(
            stage,
            &format!("total {}", unequal.total(LEFT)),
            22.0,
            Palette::dim(),
            v(
                shape.label_spot(LEFT, END_LABEL_GAP)[0],
                shape.label_spot(LEFT, END_LABEL_GAP)[1] + TOTAL_DY,
            ),
        )?;
        fade_in_all(stage, &[left_total], 0.3)?;
        flash_all(stage, &shape.wires_at(LEFT), 0.6)?;

        // 10_3: the general rule has been read and goes; the walker at the
        // left; no step's chance shows through the countdown
        wait_until(stage, general_read_until)?;
        say(&kit, stage, "10_3")?;
        fade_out_all(stage, &general, 0.3)?;
        let dot = walker_dot(stage, shape.spot(LEFT), WALKER_R, 1.0);
        fade_in_all(stage, &[dot], 0.3)?;
        flash_all(stage, &shape.wires_at(LEFT), 0.6)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 10_4: across with chance one half; the left's equation; then the
        // right's steps and equation
        let line = say(&kit, stage, "10_4")?;
        let left_steps = facts::steps(&unequal, LEFT, &facts::across_links(LEFT));
        let left_texts: Vec<(String, Srgb)> = left_steps.iter().map(|&s| (small_tex(s), palette::walker())).collect();
        let left_tokens = tokens_at(&kit, stage, &shape, LEFT, &left_texts, STEP_T)?;
        fade_out_all(stage, &[countdown], 0.15)?;
        fade_in_all(stage, &left_tokens, 0.3)?;
        let left_row = row(&kit, stage, &step_row(&unequal, &held, LEFT), 0)?;
        fade_in_all(stage, &[left_row], 0.4)?;
        line.wait_for(stage, "At the right")?;
        let right_steps = facts::steps(&unequal, RIGHT, &facts::across_links(RIGHT));
        let right_texts: Vec<(String, Srgb)> = right_steps.iter().map(|&s| (small_tex(s), palette::walker())).collect();
        let right_tokens = tokens_at(&kit, stage, &shape, RIGHT, &right_texts, STEP_T)?;
        fade_out_all(stage, &left_tokens, 0.15)?;
        fade_in_all(stage, &right_tokens, 0.3)?;
        let right_row = row(&kit, stage, &step_row(&unequal, &held, RIGHT), 1)?;
        fade_in_all(stage, &[right_row], 0.4)?;

        // 10_5: solved, seven sixteenths and three eighths, voltage and
        // chance alike
        let line = say(&kit, stage, "10_5")?;
        let solved = [
            label(&kit, stage, &shape, LEFT, &tex(held[LEFT]), Palette::ink())?,
            label(&kit, stage, &shape, RIGHT, &tex(held[RIGHT]), Palette::ink())?,
        ];
        let mut going = right_tokens.clone();
        going.extend([dot, left_total]);
        fade_out_all(stage, &going, 0.2)?;
        fade_in_all(stage, &solved, 0.2)?;
        recolor(
            stage,
            &[
                (shape.junctions[LEFT], palette::value(held[LEFT])),
                (shape.junctions[RIGHT], palette::value(held[RIGHT])),
            ],
            0.3,
        )?;
        line.wait_for(stage, "seven sixteenths")?;
        let both = row(
            &kit,
            stage,
            &format!(
                r"{LEFT_NAME} = {},\ {RIGHT_NAME} = {}",
                small_tex(held[LEFT]),
                small_tex(held[RIGHT])
            ),
            2,
        )?;
        let same = kit.text(stage, SAME, 22.0, Palette::dim(), v(PANEL_X, ROW_Y[3]))?;
        fade_in_all(stage, &[both, same], 0.4)?;
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
            ("10_1", "weight each far end"),
            ("10_1", "the walker chooses"),
            ("10_1", "The same rule"),
            ("10_2", "At the left"),
            ("10_4", "At the right"),
            ("10_5", "seven sixteenths"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The rows are the facts': the bridge's weights, and the unequal
    /// wires' first steps and values, Doyle and Snell's.
    #[test]
    fn the_rows_are_built_from_the_facts() {
        let net = facts::network().unwrap();
        assert_eq!(
            weights_row(&facts::left_conductances().unwrap(), net.total(LEFT)),
            r"\text{weights: } \tfrac{2}{4},\ \tfrac{1}{4},\ \tfrac{1}{4}"
        );
        let unequal = facts::unequal().unwrap();
        let held = unequal.kirchhoff(Q::ONE, Q::ZERO).unwrap();
        assert_eq!(
            step_row(&unequal, &held, LEFT),
            r"L = \tfrac{1}{4} \cdot 1 + \tfrac{1}{2} \cdot R + \tfrac{1}{4} \cdot 0"
        );
        assert_eq!(
            step_row(&unequal, &held, RIGHT),
            r"R = \tfrac{1}{5} \cdot 1 + \tfrac{2}{5} \cdot L + \tfrac{2}{5} \cdot 0"
        );
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F15.voltages"), Some("7/16,3/8"));
        assert_eq!(f.get("R-F15.most_likely"), Some("across"));
        assert_eq!(f.get("R-P1.voltages"), Some("7/16,3/8"));
        // the general note: each line set unwrapped, at most 72 characters,
        // and it states the rule's domain
        for text in GENERAL_NOTE {
            assert!(text.chars().count() <= 72, "{text}");
        }
        assert!(GENERAL_NOTE[0].contains("any middle junction"));
        assert!(GENERAL_NOTE[1].contains("each junction wired to x"));
        // the source note does not claim the drawing is their figure
        assert!(SOURCE_NOTE.starts_with("their conductances, drawn on this shape"));
    }

    /// The panel stays right of the picture and inside the title-safe area,
    /// and the countdown sits under its top row.
    #[test]
    fn the_panel_keeps_clear_of_the_picture() {
        use crate::chapters::draw::THINK_R;
        const { assert!(PANEL_X - 2.8 > PICTURE_AT[0] + 1.7 + END_LABEL_GAP + 0.3) };
        const { assert!(PANEL_X + 2.8 < FRAME_X_RADIUS - 0.25) };
        const { assert!(ROW_Y[0] + 0.45 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW_Y[5] - 2.0 * NOTE_LEADING - 0.15 > -FRAME_Y_RADIUS + 1.58) };
        const { assert!(THINK_AT[1] + THINK_R + 0.4 < ROW_Y[0]) };
    }
}
