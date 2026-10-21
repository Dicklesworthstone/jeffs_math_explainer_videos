//! Chapter 04, the same number twice: the circuit as chapter 03 leaves it
//! (alone, its battery, the middle junctions at their values); the same
//! wires again with a walker instead of a battery, and one route it might
//! take (04_1); its chance of reaching the top first is the same average,
//! one step ahead (04_2); the top and the bottom as one and zero, and the
//! prediction (04_3); the same values and the same equation, and why they
//! must agree: one rule with one solution (04_4); the walker is a model of
//! the equations, not of electrons (04_5). The lines are production
//! chapter 01's 01_4 to 01_8, their pictures and frozen timing kept; the
//! network's first three lines are chapter 03's.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, EQUATION_SIZE, LABEL_GAP, LABEL_SIZE, Picture, Strand, battery, equation, fade_in_all,
    fade_in_lagged, fade_out_all, fail, flash_all, flat, move_all, pause, picture, recolor, say, shift_all, slide_in,
    swap, tex, think, tokens, value_texts, volts_text, wait_until,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, ROUTE, TOP};
use crate::palette;

pub struct TheSameNumber {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const VOLTAGE: &str = "voltage";
const CHANCE: &str = "chance of reaching the top first";
const UNKNOWN: &str = "?";
/// The video's own scope statement (the slate's limit), not a quotation.
const MODEL_NOTE: &str = "a model of the equations, not of electrons";
/// The source of the theorem the chapter shows (Doyle and Snell §1.3.2,
/// "Interpretation of Voltage"); their remark on realism is §1.3.3's, about
/// current, and is not attributed on screen.
const SOURCE_NOTE: &str = "voltage equals chance: Doyle and Snell, section 1.3.2";

/// Where the circuit stands while it is alone, and where it moves when the
/// walk joins it; where the walk stands.
const CIRCUIT_ALONE: [f64; 2] = [-0.5, 0.3];
const CIRCUIT_BESIDE: [f64; 2] = [-4.0, 0.3];
const WALK_AT: [f64; 2] = [3.9, 0.3];
/// The prediction's countdown, between the two pictures.
const THINK_AT: [f64; 2] = [0.45, 0.6];
/// The walker's size, and how long one of its steps takes.
const WALKER_R: f64 = 0.11;
const STEP_S: f64 = 0.5;

/// "1, 2 and 3".
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// 04_2's exact version, built from the facts: the walk from the left is
/// still going after one, two and three steps with chances that are never
/// zero but shrink towards it, so it stops with chance one.
fn stops_card() -> Result<String, String> {
    let still = facts::still_walking()?;
    let steps: Vec<String> = (1..=still.len()).map(|n| n.to_string()).collect();
    let chances: Vec<String> = still.iter().map(Q::to_string).collect();
    let net = facts::network()?;
    let walk = net.walk()?;
    let stop = walk.top[LEFT] + walk.bottom[LEFT];
    if still.iter().any(|q| !q.is_positive()) || stop != Q::ONE {
        return Err(format!(
            "the walk from the left is still going with {still:?} and stops with {stop}"
        ));
    }
    Ok(format!(
        "the chance it is still walking after {} steps is {}. Never {}, but shrinking towards it, so the walk \
         stops with chance {stop}.",
        and_list(&steps),
        and_list(&chances),
        Q::ZERO
    ))
}

/// 04_4's exact version (cut X1 of the long form): the hypothesis that
/// makes the solution unique, stated for the drawn network only after the
/// model confirms it; the argument (the maximum principle, Doyle and Snell
/// §1.1.5) is chapter 09's. Two junctions wired only to each other
/// (`model::with_island`) have no unique solution.
fn unique_card() -> Result<String, String> {
    let cut = facts::network()?.cut_off();
    if !cut.is_empty() {
        return Err(format!(
            "junctions {cut:?} are wired to neither the top nor the bottom: the solution is not unique"
        ));
    }
    Ok(
        "here every middle junction is wired to the top or bottom, which is what makes the solution unique \
         (proved later in the video)."
            .to_owned(),
    )
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

/// The unknown "?" beside middle junction `x`. Detached.
fn unknown_label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize) -> fmn::Result<Mob> {
    kit.text(
        stage,
        UNKNOWN,
        LABEL_SIZE,
        Palette::hero(),
        pic.label_spot(x, END_LABEL_GAP),
    )
}

/// The walker as a dot at `at`. Detached.
fn walker(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let dot = Circle::new()
        .radius(WALKER_R)
        .style(flat(palette::walker(), 1.0))
        .build()
        .moved_to(at);
    stage.arena_mut().add(dot)
}

/// The walker takes `route` on `pic`, each step through its wire's middle.
fn walk_route(stage: &mut Stage<'_>, pic: &Picture, dot: Mob, route: &[usize]) -> fmn::Result<()> {
    for step in route.windows(2) {
        let s: &Strand = pic.strand_between(step[0], step[1])?;
        move_all(stage, &[(dot, pic.mid(s))], STEP_S / 2.0)?;
        move_all(stage, &[(dot, pic.spot(step[1]))], STEP_S / 2.0)?;
    }
    Ok(())
}

impl SceneConstruct for TheSameNumber {
    fn name(&self) -> &str {
        "04_the_same_number"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let chances = facts::chances().map_err(fail)?;
        let far_ends = net.far_ends(LEFT).map_err(fail)?;
        let (volt_ends, chance_ends) = (
            facts::far_values(&volts, LEFT).map_err(fail)?,
            facts::far_values(&chances, LEFT).map_err(fail)?,
        );

        // the circuit as chapter 03 leaves it: alone, its battery holding the
        // top and the bottom, the middle junctions at their values, coloured
        let colors: Vec<Srgb> = volts.iter().map(|&q| palette::value(q)).collect();
        let mut circuit = picture(stage, &net, CIRCUIT_ALONE, &colors)?;
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
        let left_label = value_label(&kit, stage, &circuit, LEFT, volts[LEFT])?;
        let right_label = value_label(&kit, stage, &circuit, RIGHT, volts[RIGHT])?;
        let mut arriving = circuit.mobs();
        arriving.extend(&cell);
        arriving.extend([left_label, right_label]);
        fade_in_all(stage, &arriving, 0.6)?;

        // 04_1: the circuit moves aside; the same wires again with a walker
        // instead of a battery, and one route it might take
        let line = say(&kit, stage, "04_1")?;
        let mut moving = circuit.mobs();
        moving.extend(&cell);
        moving.extend([left_label, right_label]);
        let (dx, dy) = (
            CIRCUIT_BESIDE[0] - CIRCUIT_ALONE[0],
            CIRCUIT_BESIDE[1] - CIRCUIT_ALONE[1],
        );
        shift_all(stage, &moving, dx, dy, 0.9)?;
        circuit.centre = CIRCUIT_BESIDE;
        let walk = picture(stage, &net, WALK_AT, &first_colors(&net))?;
        let circuit_name = kit.text(stage, VOLTAGE, 26.0, Palette::dim(), circuit.name_spot())?;
        let walk_name = kit.text(stage, CHANCE, 26.0, Palette::dim(), walk.name_spot())?;
        let mut arriving = walk.mobs();
        arriving.extend([circuit_name, walk_name]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "A walker")?;
        let dot = walker(stage, walk.spot(LEFT));
        fade_in_all(stage, &[dot], 0.3)?;
        line.wait_for(stage, "at random")?;
        flash_all(stage, &walk.wires_at(LEFT), 0.6)?;
        line.wait_for(stage, "walks on")?;
        walk_route(stage, &walk, dot, &ROUTE)?;
        let end = *ROUTE.last().ok_or_else(|| fail("the drawn route is empty".into()))?;
        flash_all(stage, &[walk.junctions[end]], 0.5)?;

        // 04_2: back at the left junction; the walk stops with chance one;
        // one step ahead, a "?" at each far end
        let line = say(&kit, stage, "04_2")?;
        let dot_back = walker(stage, walk.spot(LEFT));
        swap(stage, &[dot], &[dot_back], 0.4)?;
        let stops_text = stops_card().map_err(fail)?;
        let stops = precisely(&kit, stage, &stops_text)?;
        fade_in_all(stage, &[stops], 0.4)?;
        let stops_read_until = stage.scene().time().to_f64() + stops_text.chars().count() as f64 / 15.0;
        line.wait_for(stage, "look one step ahead")?;
        flash_all(stage, &walk.wires_at(LEFT), 0.6)?;
        let unknowns = vec![(UNKNOWN.to_owned(), Palette::hero()); far_ends.len()];
        let asked = tokens(&kit, stage, &walk, LEFT, &unknowns)?;
        line.wait_for(stage, "the average")?;
        fade_in_lagged(stage, &asked, 0.6, 0.3)?;

        // 04_3: the top's far ends become one and the bottom's zero; the
        // middle junctions carry "?"; the question, and the countdown
        wait_until(stage, stops_read_until)?;
        let line = say(&kit, stage, "04_3")?;
        fade_out_all(stage, &[stops], 0.3)?;
        let known = tokens(&kit, stage, &walk, LEFT, &value_texts(&chance_ends))?;
        for (end, phrase) in [(TOP, "From the top"), (BOTTOM, "from the bottom")] {
            line.wait_for(stage, phrase)?;
            let (old, new): (Vec<Mob>, Vec<Mob>) = far_ends
                .iter()
                .zip(asked.iter().zip(&known))
                .filter(|&(&y, _)| y == end)
                .map(|(_, (&a, &k))| (a, k))
                .unzip();
            let label = kit.tex(
                stage,
                &tex(chances[end]),
                LABEL_SIZE,
                Palette::ink(),
                &[],
                walk.label_spot(end, LABEL_GAP),
            )?;
            let mut arriving = new;
            arriving.push(label);
            fade_out_all(stage, &old, 0.2)?;
            fade_in_all(stage, &arriving, 0.2)?;
        }
        line.wait_for(stage, "So what is it")?;
        let left_q = unknown_label(&kit, stage, &walk, LEFT)?;
        let right_q = unknown_label(&kit, stage, &walk, RIGHT)?;
        fade_in_all(stage, &[left_q, right_q], 0.3)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 04_4: the answer; the same four values slide in and the same
        // equation forms; both pictures' equations side by side; then why
        // the two must agree
        let line = say(&kit, stage, "04_4")?;
        let walk_left = value_label(&kit, stage, &walk, LEFT, chances[LEFT])?;
        let walk_right = value_label(&kit, stage, &walk, RIGHT, chances[RIGHT])?;
        fade_out_all(stage, &[countdown, left_q, right_q], 0.15)?;
        fade_in_all(stage, &[walk_left, walk_right], 0.15)?;
        recolor(
            stage,
            &[
                (walk.junctions[LEFT], palette::value(chances[LEFT])),
                (walk.junctions[RIGHT], palette::value(chances[RIGHT])),
            ],
            0.3,
        )?;
        let right_asked: Vec<Mob> = far_ends
            .iter()
            .zip(&asked)
            .filter(|&(&y, _)| y == RIGHT)
            .map(|(_, &a)| a)
            .collect();
        let right_known: Vec<Mob> = far_ends
            .iter()
            .zip(&known)
            .filter(|&(&y, _)| y == RIGHT)
            .map(|(_, &k)| k)
            .collect();
        fade_out_all(stage, &right_asked, 0.15)?;
        fade_in_all(stage, &right_known, 0.15)?;
        slide_in(stage, &known, walk.spot(LEFT), 0.7)?;
        let walk_rule = kit.tex(
            stage,
            &equation(&chance_ends, chances[LEFT]),
            EQUATION_SIZE,
            Palette::ink(),
            &[],
            walk.equation_spot(),
        )?;
        fade_in_all(stage, &[walk_rule], 0.4)?;
        line.wait_for(stage, "One rule")?;
        let circuit_rule = kit.tex(
            stage,
            &equation(&volt_ends, volts[LEFT]),
            EQUATION_SIZE,
            Palette::ink(),
            &[],
            circuit.equation_spot(),
        )?;
        fade_in_all(stage, &[circuit_rule], 0.4)?;
        flash_all(stage, &[left_label, walk_left], 0.6)?;
        line.wait_for(stage, "only one solution")?;
        let unique_text = unique_card().map_err(fail)?;
        let unique = precisely(&kit, stage, &unique_text)?;
        // the card covers the equations' place: out, then in, never both
        fade_out_all(stage, &[circuit_rule, walk_rule], 0.2)?;
        fade_in_all(stage, &[unique], 0.2)?;
        let unique_read_until = stage.scene().time().to_f64() + unique_text.chars().count() as f64 / 15.0;
        line.wait_for(stage, "must agree")?;
        flash_all(stage, &[left_label, walk_left], 0.5)?;

        // 04_5: the walker is a model of the equations; it leaves, and the
        // two pictures stand with their equal numbers
        wait_until(stage, unique_read_until)?;
        let line = say(&kit, stage, "04_5")?;
        let at = walk.equation_spot();
        let model_note = kit.text(stage, MODEL_NOTE, 22.0, Palette::dim(), at)?;
        let source_note = kit.text(stage, SOURCE_NOTE, 18.0, Palette::dim(), v(at[0], at[1] - 0.4))?;
        // the notes stand where the card stood: out, then in
        fade_out_all(stage, &[unique, dot_back], 0.2)?;
        fade_in_all(stage, &[model_note, source_note], 0.2)?;
        line.wait_for(stage, "The two numbers agree")?;
        flash_all(stage, &[left_label, walk_left], 0.5)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, 0.6)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BATTERY_X, EQUATION_DY, JUNCTION_R, NAME_DY, SPOTS, THINK_R};

    /// Every phrase the scene waits for is in its line of script.tsv (a
    /// changed line fails here, not in the middle of a render).
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "A walker"),
            ("04_1", "at random"),
            ("04_1", "walks on"),
            ("04_2", "look one step ahead"),
            ("04_2", "the average"),
            ("04_3", "From the top"),
            ("04_3", "from the bottom"),
            ("04_3", "So what is it"),
            ("04_4", "One rule"),
            ("04_4", "only one solution"),
            ("04_4", "must agree"),
            ("04_5", "The two numbers agree"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The exact versions say what the facts say, and fit a lower-third of
    /// three lines (the kit's band wraps at about 92 characters; 84 leaves
    /// room for the "precisely:" label).
    #[test]
    fn the_exact_versions_are_built_from_the_facts_and_fit() {
        let stops = stops_card().unwrap();
        assert!(
            stops.contains("after 1, 2 and 3 steps is 1/4, 1/16 and 1/64."),
            "{stops}"
        );
        assert!(stops.contains("stops with chance 1."), "{stops}");
        let unique = unique_card().unwrap();
        // the hypothesis, and that it is what makes the answer unique; the
        // argument is chapter 09's
        assert!(
            unique.starts_with("here every middle junction is wired to the top or bottom, which is what makes"),
            "{unique}"
        );
        assert!(unique.ends_with("(proved later in the video)."), "{unique}");
        for text in [stops, unique] {
            let lines = explainer_kit::animatic::wrap(&text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
        assert_eq!(and_list(&["a".into()]), "a");
        assert_eq!(and_list(&["a".into(), "b".into(), "c".into()]), "a, b and c");
    }

    /// The equations on screen are the facts' far ends over their count.
    #[test]
    fn the_equations_are_the_facts() {
        let volts = facts::voltages().unwrap();
        let chances = facts::chances().unwrap();
        let want = r"\frac{1 + 1 + \tfrac{2}{5} + 0}{4} = \frac{3}{5}";
        assert_eq!(equation(&facts::far_values(&volts, LEFT).unwrap(), volts[LEFT]), want);
        assert_eq!(
            equation(&facts::far_values(&chances, LEFT).unwrap(), chances[LEFT]),
            want
        );
        assert_eq!(
            equation(&facts::far_values(&volts, RIGHT).unwrap(), volts[RIGHT]),
            r"\frac{1 + \tfrac{3}{5} + 0 + 0}{4} = \frac{2}{5}"
        );
        // the route the walker is drawn taking is a walk on these wires
        assert!(facts::network().unwrap().is_route(&ROUTE));
    }

    /// Both pictures, their battery, names, labels and equations stay inside
    /// the title-safe area, and the countdown sits between them.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.15;
        for c in [CIRCUIT_ALONE, CIRCUIT_BESIDE, WALK_AT] {
            assert!(c[1] + NAME_DY + 0.2 < safe_y, "the name inside the top");
            assert!(c[1] + EQUATION_DY - 0.5 > -safe_y, "the equation inside the bottom");
            for s in SPOTS {
                assert!((c[0] + s[0]).abs() + LABEL_GAP + 0.3 < safe_x);
            }
        }
        // a three-line lower-third reaches 1.58 above the frame's bottom
        // edge (the kit's band: 0.2 margin, 0.3 padding, 0.36 a line); it
        // clears the bottom junctions and their labels
        for c in [CIRCUIT_ALONE, WALK_AT] {
            assert!(c[1] + SPOTS[BOTTOM][1] - JUNCTION_R - 0.15 > -FRAME_Y_RADIUS + 1.58);
        }
        // the battery stays inside, alone and beside the walk
        assert!(CIRCUIT_ALONE[0] + BATTERY_X + 0.3 < safe_x);
        // the countdown clears the circuit's battery and the walk's left label
        let battery = CIRCUIT_BESIDE[0] + BATTERY_X + 0.3;
        let walk_label = WALK_AT[0] + SPOTS[LEFT][0] - LABEL_GAP - 0.25;
        assert!(THINK_AT[0] - THINK_R > battery);
        assert!(THINK_AT[0] + THINK_R < walk_label);
        // the two pictures do not overlap: the circuit's battery is left of
        // the walk's left value
        assert!(battery < walk_label);
    }
}
