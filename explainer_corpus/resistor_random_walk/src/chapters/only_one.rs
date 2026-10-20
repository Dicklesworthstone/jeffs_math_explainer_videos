//! Chapter 04, only one solution: the circuit beside a supposed second
//! solution with the same top and bottom (04_1); their difference, 0 at
//! both ends and keeping the rule (04_2); its largest value, which above
//! zero sits at a middle junction averaging far ends no larger (04_3); the
//! peak spreads along wires to the top, where the difference is 0 (04_4);
//! so it is 0, and the lowest value too, and the two solutions are the same
//! (04_5); an island of two junctions wired only to each other, and the
//! prediction (04_6); its peak crosses its one wire and stops, so any equal
//! pair keeps its rule (04_7); the hypothesis (04_8); a walker on the island
//! never reaches the top, a chance of 0, while the island's voltage is left
//! free (04_9).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, EQUATION_SIZE, JUNCTION_R, LABEL_GAP, LABEL_SIZE, Picture, SPOTS, TOKEN_SIZE, WIRE_W, battery,
    equation, fade_in_all, fade_out_all, fail, flash_all, flat, line_style, move_all, pause, picture, recolor, say,
    tex, think, tokens, volts_text, wait_until,
};
use crate::facts;
use crate::model::{BOTTOM, LEFT, Network, Q, RIGHT, TOP};
use crate::palette;

pub struct OnlyOne {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIRST: &str = "a solution";
const SECOND: &str = "a second solution?";
const DIFFERENCE: &str = "their difference";
const SAME: &str = "the same solution";
const ISLAND: &str = "an island";
const NO_WIRE: &str = "no wire to the top or the bottom";
const CHANCE: &str = "chance of reaching the top first";
const VOLTAGE_NOTE: &str = "voltage: any equal pair c, c";
const UNKNOWN: &str = "?";
/// The difference's largest value, a part no larger, and the island's free
/// voltage, as TeX.
const PEAK: &str = "M";
const AT_MOST_PEAK: &str = r"\le M";
const FREE: &str = "c";

/// Where the circuit stands (as chapter 03 had it), and the second copy.
const CIRCUIT_AT: [f64; 2] = [-4.0, 0.3];
const COPY_AT: [f64; 2] = [3.9, 0.3];
const NAME_SIZE: f64 = 26.0;
const NOTE_SIZE: f64 = 22.0;
/// The island's two junctions, right of the battery; their values above
/// them, its name above those, its notes and rule beneath.
const ISLAND_AT: [[f64; 2]; 2] = [[2.4, 0.6], [5.4, 0.6]];
const ISLAND_LABEL_DY: f64 = JUNCTION_R + FRACTION_HALF + 0.18;
const ISLAND_NAME_DY: f64 = 1.6;
/// Half a stacked fraction's height at [`LABEL_SIZE`] (an island value of
/// 1/2), so a value above a junction clears its dot.
const FRACTION_HALF: f64 = 0.45;
/// The M that crosses the island's wire: how far above the wire it travels,
/// and how far from each junction's centre it sets out and stops.
const CROSS_DY: f64 = 0.3;
const CROSS_INSET: f64 = 0.45;
const NOTE_Y: [f64; 2] = [0.0, -0.85];
/// The prediction's countdown, under the island.
const THINK_AT: [f64; 2] = [3.9, -1.45];
/// The walker on the island: its size, a step, and how many steps it is
/// shown taking.
const WALKER_R: f64 = 0.11;
const STEP_S: f64 = 0.5;
const WALKER_STEPS: usize = 4;
/// Hold each unspoken equal-pair equation for at least two seconds once
/// visible, including the preceding recolor (04_7).
const LEVEL_S: f64 = 1.7;
/// A lower-third's fade in and out, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, a quick reader (the video can be paused). Each card stays up at
/// least this long from the moment it is fully in, over the lines after it.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// 04_2's exact version: why the difference keeps the rule, and its value
/// at the top and the bottom (each of the circuit's ends less itself).
fn difference_card() -> Result<String, String> {
    let v = facts::voltages()?;
    let (top, bottom) = facts::difference_ends()?;
    if top != bottom {
        return Err(format!("the difference is {top} at the top but {bottom} at the bottom"));
    }
    Ok(format!(
        "at every middle junction each solution is the conductance-weighted average of the same far ends, so their \
         difference is the average of the far ends' differences. At the top it is {} − {}, at the bottom {} − {}: \
         both {top}.",
        v[TOP], v[TOP], v[BOTTOM], v[BOTTOM]
    ))
}

/// 04_4's exact version: the maximum principle with positive weights; the
/// peak's path is this example's, here to the top, where the difference is
/// 0.
fn peak_card() -> Result<String, String> {
    let (top, _) = facts::difference_ends()?;
    Ok(format!(
        "with positive weights an average is at most its largest part, and equals it only if every part does. So a \
         peak above {top} at a middle junction passes to every far end, and on along wires; here, to the top, where it \
         is {top}. Likewise for the lowest value."
    ))
}

/// 04_8's exact version: the hypothesis the argument needs, and what the
/// island does without it. The maximum and minimum argument proves at most
/// one solution; the circuit's own voltages are one. Built only when the
/// model finds no path of wires from the island to either end, and every
/// equal pair shown keeping the island's rule.
fn hypothesis_card() -> Result<String, String> {
    let (net, cut) = facts::island()?;
    let reaches = cut
        .iter()
        .any(|&x| net.path(x, net.top()).is_some() || net.path(x, net.bottom()).is_some());
    if reaches {
        return Err(format!("junctions {cut:?} have a path of wires to an end: no island"));
    }
    for c in facts::island_levels() {
        if !facts::keeps_rule(&net, &facts::with_island_values(c, c)?) {
            return Err(format!("the equal pair {c}, {c} breaks the island's rule"));
        }
    }
    Ok(
        "with positive conductances, if every middle junction has a path of wires to the top or the bottom, the rule \
        with those two held has at most one solution. The island has no such path: any equal pair keeps its rule."
            .to_owned(),
    )
}

/// 04_9's note: the walker's chance, from the island, of reaching the top
/// first. No path of wires leads from the island to the top or the bottom,
/// and a walker moves only along wires, so it is never there after any
/// number of steps: that is the proof, for all time. The steps followed
/// (never at the top) are a witness that agrees, not the proof.
fn chance_note() -> Result<String, String> {
    let (net, cut) = facts::island()?;
    if let Some(&x) = cut
        .iter()
        .find(|&&x| net.path(x, net.top()).is_some() || net.path(x, net.bottom()).is_some())
    {
        return Err(format!("junction {x} of the island has a path of wires to an end"));
    }
    let steps = net.step_chances(cut[0], facts::ISLAND_STEPS)?;
    let most = steps.iter().map(|s| s[net.top()]).max().unwrap_or(Q::ONE);
    if !most.is_zero() {
        return Err(format!(
            "a walker from junction {} reaches the top with chance {most}",
            cut[0]
        ));
    }
    Ok(format!("{CHANCE}: {most}"))
}

/// The values beside a peak's wires at middle junction `x`: each far end's,
/// `end` at the top or the bottom (the difference there) and at most the
/// peak at a middle junction.
fn part_texts(net: &Network, x: usize, end: Q) -> Result<Vec<(String, Srgb)>, String> {
    Ok(net
        .far_ends(x)?
        .into_iter()
        .map(|y| {
            if net.is_end(y) {
                (tex(end), palette::value(end))
            } else {
                (AT_MOST_PEAK.to_owned(), Palette::hero())
            }
        })
        .collect())
}

/// The wires at a junction, kept one per far end: a peak travelling along a
/// pair of wires to one far end lands there as one mark. Indices into
/// `ends`, the kept and the rest.
fn one_per_end(ends: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let (mut kept, mut rest): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
    for (i, &y) in ends.iter().enumerate() {
        if kept.iter().any(|&k| ends[k] == y) {
            rest.push(i);
        } else {
            kept.push(i);
        }
    }
    (kept, rest)
}

/// Where a peak travelling to junction `y` lands, from its picture's
/// centre, clear of the junction's dot: beside a middle junction, where its
/// value goes; right of the top and the bottom (their 0 is on the left), a
/// little further from the middle.
fn landing(net: &Network, y: usize) -> [f64; 2] {
    let [x, h] = SPOTS[y];
    if net.is_end(y) {
        [x + LABEL_GAP, h + 0.05 * h.signum()]
    } else {
        [x + LABEL_GAP * x.signum(), h]
    }
}

/// Junction `x`'s value on `pic` as TeX in `color`, beside it. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x, END_LABEL_GAP))
}

/// The unknown "?" beside junction `x` of `pic`. Detached.
fn unknown(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize) -> fmn::Result<Mob> {
    kit.text(
        stage,
        UNKNOWN,
        LABEL_SIZE,
        Palette::hero(),
        pic.label_spot(x, END_LABEL_GAP),
    )
}

/// A junction's dot at `at` in `color`. Detached.
fn dot(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let circle = Circle::new().radius(r).style(flat(color, 1.0)).build().moved_to(at);
    stage.arena_mut().add(circle)
}

/// The island: its two junctions at [`ISLAND_AT`], faint, and the one wire
/// between them, joined to nothing else.
struct Island {
    junctions: [Mob; 2],
    wire: Mob,
}

impl Island {
    /// Junction `i`'s centre.
    fn spot(i: usize) -> Vec3 {
        v(ISLAND_AT[i][0], ISLAND_AT[i][1])
    }

    /// Where junction `i`'s value is written, above it.
    fn label_spot(i: usize) -> Vec3 {
        v(ISLAND_AT[i][0], ISLAND_AT[i][1] + ISLAND_LABEL_DY)
    }

    /// Where the crossing M sets out (junction 0) or stops (junction 1):
    /// above the wire, clear of the junction's dot, towards the other.
    fn along(i: usize) -> Vec3 {
        let inward = if i == 0 { CROSS_INSET } else { -CROSS_INSET };
        v(ISLAND_AT[i][0] + inward, ISLAND_AT[i][1] + CROSS_DY)
    }

    /// The point midway between the two junctions, at height `y`.
    fn under(y: f64) -> Vec3 {
        v((ISLAND_AT[0][0] + ISLAND_AT[1][0]) / 2.0, y)
    }

    /// Drawn, detached.
    fn new(stage: &mut Stage<'_>) -> Self {
        let (p, q) = (Island::spot(0), Island::spot(1));
        let path = vec![v(p[0] + JUNCTION_R, p[1]), v(q[0] - JUNCTION_R, q[1])];
        let wire = stage
            .arena_mut()
            .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build());
        let junctions = [
            dot(stage, p, JUNCTION_R, palette::unknown()),
            dot(stage, q, JUNCTION_R, palette::unknown()),
        ];
        Self { junctions, wire }
    }
}

/// The island's two values as TeX in `color`, above its junctions.
/// Detached.
fn island_labels(kit: &Kit, stage: &mut Stage<'_>, src: &str, color: Srgb) -> fmn::Result<[Mob; 2]> {
    Ok([
        kit.tex(stage, src, LABEL_SIZE, color, &[], Island::label_spot(0))?,
        kit.tex(stage, src, LABEL_SIZE, color, &[], Island::label_spot(1))?,
    ])
}

impl SceneConstruct for OnlyOne {
    fn name(&self) -> &str {
        "04_only_one"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let (top_gap, bottom_gap) = facts::difference_ends().map_err(fail)?;
        // the only solution with the difference's ends: what the argument
        // leaves the difference
        let difference = net.kirchhoff(top_gap, bottom_gap).map_err(fail)?;

        // 04_1: the circuit and its values; a second copy with the same top
        // and bottom and unknown middle junctions
        let line = say(&kit, stage, "04_1")?;
        let colors: Vec<Srgb> = volts.iter().map(|&q| palette::value(q)).collect();
        let circuit = picture(stage, &net, CIRCUIT_AT, &colors)?;
        let mut held = battery(stage, circuit.centre)?;
        let ends_text = [
            kit.text(
                stage,
                &volts_text(volts[TOP]),
                24.0,
                Palette::ink(),
                circuit.label_spot(TOP, END_LABEL_GAP),
            )?,
            kit.text(
                stage,
                &volts_text(volts[BOTTOM]),
                24.0,
                Palette::ink(),
                circuit.label_spot(BOTTOM, END_LABEL_GAP),
            )?,
        ];
        held.extend(ends_text);
        let circuit_labels = [
            label(&kit, stage, &circuit, LEFT, &tex(volts[LEFT]), Palette::ink())?,
            label(&kit, stage, &circuit, RIGHT, &tex(volts[RIGHT]), Palette::ink())?,
        ];
        let circuit_name = kit.text(stage, FIRST, NAME_SIZE, Palette::dim(), circuit.name_spot())?;
        let mut arriving = circuit.mobs();
        arriving.extend(&held);
        arriving.extend(circuit_labels);
        arriving.push(circuit_name);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Suppose another")?;
        let copy_colors: Vec<Srgb> = (0..net.size())
            .map(|x| {
                if net.is_end(x) {
                    palette::value(volts[x])
                } else {
                    palette::unknown()
                }
            })
            .collect();
        let copy = picture(stage, &net, COPY_AT, &copy_colors)?;
        let mut copy_labels = [
            label(&kit, stage, &copy, TOP, &tex(volts[TOP]), Palette::ink())?,
            unknown(&kit, stage, &copy, LEFT)?,
            unknown(&kit, stage, &copy, RIGHT)?,
            label(&kit, stage, &copy, BOTTOM, &tex(volts[BOTTOM]), Palette::ink())?,
        ];
        let copy_name = kit.text(stage, SECOND, NAME_SIZE, Palette::dim(), copy.name_spot())?;
        let mut arriving = copy.mobs();
        arriving.extend(copy_labels);
        arriving.push(copy_name);
        fade_in_all(stage, &arriving, 0.5)?;
        line.wait_for(stage, "the same top and bottom")?;
        flash_all(
            stage,
            &[ends_text[0], ends_text[1], copy_labels[TOP], copy_labels[BOTTOM]],
            0.6,
        )?;

        // 04_2: the copy becomes the difference, 0 at both ends; its card
        let line = say(&kit, stage, "04_2")?;
        let diff_name = kit.text(stage, DIFFERENCE, NAME_SIZE, Palette::dim(), copy.name_spot())?;
        let zero_ends = [
            label(&kit, stage, &copy, TOP, &tex(top_gap), Palette::ink())?,
            label(&kit, stage, &copy, BOTTOM, &tex(bottom_gap), Palette::ink())?,
        ];
        fade_out_all(stage, &[copy_name, copy_labels[TOP], copy_labels[BOTTOM]], 0.25)?;
        fade_in_all(stage, &[diff_name, zero_ends[0], zero_ends[1]], 0.25)?;
        [copy_labels[TOP], copy_labels[BOTTOM]] = zero_ends;
        recolor(
            stage,
            &[
                (copy.junctions[TOP], palette::value(top_gap)),
                (copy.junctions[BOTTOM], palette::value(bottom_gap)),
            ],
            0.4,
        )?;
        line.wait_for(stage, "The difference is zero")?;
        flash_all(stage, &zero_ends, 0.6)?;
        let rule_text = difference_card().map_err(fail)?;
        let rule_card = precisely(&kit, stage, &rule_text)?;
        fade_in_all(stage, &[rule_card], CARD_IN)?;
        let rule_read = stage.scene().time().to_f64() + read_s(&rule_text);
        line.wait_for(stage, "keeps the rule too")?;
        let copy_wires: Vec<Mob> = copy.strands.iter().map(|s| s.mob).collect();
        flash_all(stage, &copy_wires, 0.6)?;

        // 04_3: its largest value, at the left junction; the ends cannot hold
        // it; the far ends it averages, none larger
        let line = say(&kit, stage, "04_3")?;
        let peak = label(&kit, stage, &copy, LEFT, PEAK, Palette::hero())?;
        fade_out_all(stage, &[copy_labels[LEFT]], 0.2)?;
        fade_in_all(stage, &[peak], 0.2)?;
        copy_labels[LEFT] = peak;
        recolor(stage, &[(copy.junctions[LEFT], Palette::hero())], 0.4)?;
        line.wait_for(stage, "If above zero")?;
        flash_all(stage, &zero_ends, 0.6)?;
        line.wait_for(stage, "an average of far ends")?;
        let parts = tokens(
            &kit,
            stage,
            &copy,
            LEFT,
            &part_texts(&net, LEFT, top_gap).map_err(fail)?,
        )?;
        fade_in_all(stage, &parts, 0.4)?;

        // 04_4: the maximum principle's card; every part must be the peak,
        // which travels along every wire, and along the path to the top
        wait_until(stage, rule_read)?;
        let line = say(&kit, stage, "04_4")?;
        let peak_text = peak_card().map_err(fail)?;
        let max_card = precisely(&kit, stage, &peak_text)?;
        fade_out_all(stage, &[rule_card], CARD_OUT / 2.0)?;
        fade_in_all(stage, &[max_card], CARD_OUT / 2.0)?;
        let peak_read = stage.scene().time().to_f64() + read_s(&peak_text);
        line.wait_for(stage, "every part equals it")?;
        let forced_texts: Vec<(String, Srgb)> = parts.iter().map(|_| (PEAK.to_owned(), Palette::hero())).collect();
        let forced = tokens(&kit, stage, &copy, LEFT, &forced_texts)?;
        // one after the other: "≤ M" and "M" never share a place
        fade_out_all(stage, &parts, 0.2)?;
        fade_in_all(stage, &forced, 0.3)?;
        line.wait_for(stage, "So the peak spreads")?;
        // one M per far end, landing clear of its dot: a pair's second M
        // goes as the first sets out, and a middle junction's "?" goes for
        // the M it receives, which then becomes its value (one after the
        // other, never two M at one place)
        let ends = net.far_ends(LEFT).map_err(fail)?;
        let (kept, rest) = one_per_end(&ends);
        let mut going: Vec<Mob> = rest.iter().map(|&i| forced[i]).collect();
        going.extend(
            kept.iter()
                .filter(|&&i| !net.is_end(ends[i]))
                .map(|&i| copy_labels[ends[i]]),
        );
        fade_out_all(stage, &going, 0.2)?;
        let travel: Vec<(Mob, Vec3)> = kept
            .iter()
            .map(|&i| {
                let [dx, dy] = landing(&net, ends[i]);
                (forced[i], v(copy.centre[0] + dx, copy.centre[1] + dy))
            })
            .collect();
        move_all(stage, &travel, 0.8)?;
        let (at_ends, at_middle): (Vec<usize>, Vec<usize>) = kept.iter().partition(|&&i| net.is_end(ends[i]));
        let arrived: Vec<Mob> = at_middle.iter().map(|&i| forced[i]).collect();
        fade_out_all(stage, &arrived, 0.15)?;
        let (mut values, mut colours) = (Vec::new(), Vec::new());
        for &i in &at_middle {
            let y = ends[i];
            copy_labels[y] = label(&kit, stage, &copy, y, PEAK, Palette::hero())?;
            values.push(copy_labels[y]);
            colours.push((copy.junctions[y], Palette::hero()));
        }
        fade_in_all(stage, &values, 0.3)?;
        recolor(stage, &colours, 0.3)?;
        let path = net
            .path(LEFT, TOP)
            .ok_or_else(|| fail("no path of wires from the left junction to the top".to_owned()))?;
        let mut along = Vec::new();
        for step in path.windows(2) {
            along.push(copy.strand_between(step[0], step[1])?.mob);
        }
        flash_all(stage, &along, 0.5)?;
        line.wait_for(stage, "which is zero")?;
        // each end's 0 against the M that reached it
        let marks: Vec<Mob> = at_ends.iter().map(|&i| forced[i]).collect();
        let mut clash = marks.clone();
        clash.extend(at_ends.iter().map(|&i| copy_labels[ends[i]]));
        flash_all(stage, &clash, 0.6)?;
        fade_out_all(stage, &marks, 0.3)?;

        // 04_5: the peak is 0, the lowest value too: the difference is 0
        // everywhere, and the second solution is the first
        let line = say(&kit, stage, "04_5")?;
        let zeros = [
            label(&kit, stage, &copy, LEFT, &tex(difference[LEFT]), Palette::ink())?,
            label(&kit, stage, &copy, RIGHT, &tex(difference[RIGHT]), Palette::ink())?,
        ];
        fade_out_all(stage, &[copy_labels[LEFT], copy_labels[RIGHT]], 0.2)?;
        fade_in_all(stage, &zeros, 0.2)?;
        [copy_labels[LEFT], copy_labels[RIGHT]] = zeros;
        recolor(
            stage,
            &[
                (copy.junctions[LEFT], palette::value(difference[LEFT])),
                (copy.junctions[RIGHT], palette::value(difference[RIGHT])),
            ],
            0.4,
        )?;
        line.wait_for(stage, "likewise the lowest")?;
        flash_all(stage, &zeros, 0.6)?;
        line.wait_for(stage, "Both solutions are the same")?;
        let same_name = kit.text(stage, SAME, NAME_SIZE, Palette::dim(), copy.name_spot())?;
        let same_labels = [
            label(&kit, stage, &copy, TOP, &tex(volts[TOP]), Palette::ink())?,
            label(&kit, stage, &copy, LEFT, &tex(volts[LEFT]), Palette::ink())?,
            label(&kit, stage, &copy, RIGHT, &tex(volts[RIGHT]), Palette::ink())?,
            label(&kit, stage, &copy, BOTTOM, &tex(volts[BOTTOM]), Palette::ink())?,
        ];
        let mut old = vec![diff_name];
        old.extend(copy_labels);
        let mut new = vec![same_name];
        new.extend(same_labels);
        fade_out_all(stage, &old, 0.25)?;
        fade_in_all(stage, &new, 0.25)?;
        let back: Vec<(Mob, Srgb)> = (0..net.size())
            .map(|x| (copy.junctions[x], palette::value(volts[x])))
            .collect();
        recolor(stage, &back, 0.5)?;
        flash_all(
            stage,
            &[
                circuit_labels[0],
                circuit_labels[1],
                same_labels[LEFT],
                same_labels[RIGHT],
            ],
            0.6,
        )?;

        // 04_6: the copy goes; an island beside the circuit; the card stays
        // while the question is asked and goes as it ends, so the island's
        // "?" and the countdown are all that change
        let question = say(&kit, stage, "04_6")?;
        let mut going = copy.mobs();
        going.extend(same_labels);
        going.push(same_name);
        fade_out_all(stage, &going, 0.5)?;
        question.wait_for(stage, "two junctions wired")?;
        let island = Island::new(stage);
        let mut island_values = [
            kit.text(stage, UNKNOWN, LABEL_SIZE, Palette::hero(), Island::label_spot(0))?,
            kit.text(stage, UNKNOWN, LABEL_SIZE, Palette::hero(), Island::label_spot(1))?,
        ];
        let island_name = kit.text(
            stage,
            ISLAND,
            NAME_SIZE,
            Palette::dim(),
            Island::under(ISLAND_AT[0][1] + ISLAND_NAME_DY),
        )?;
        let mut arriving = vec![island.wire];
        arriving.extend(island.junctions);
        arriving.extend(island_values);
        arriving.push(island_name);
        fade_in_all(stage, &arriving, 0.5)?;
        wait_until(stage, peak_read.max(question.start + question.len - CARD_OUT))?;
        fade_out_all(stage, &[max_card], CARD_OUT)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 04_7: no; the island's peak crosses its one wire and stops; any
        // equal pair keeps its rule, the circuit unchanged
        let line = say(&kit, stage, "04_7")?;
        let first_peak = kit.tex(stage, PEAK, LABEL_SIZE, Palette::hero(), &[], Island::label_spot(0))?;
        fade_out_all(stage, &[countdown, island_values[0]], 0.15)?;
        fade_in_all(stage, &[first_peak], 0.15)?;
        island_values[0] = first_peak;
        recolor(stage, &[(island.junctions[0], Palette::hero())], 0.3)?;
        line.wait_for(stage, "crosses its one wire")?;
        // the M crosses above the wire, clear of both dots, and hands over
        // to the second junction's value: one M there at a time
        let crossing = kit.tex(stage, PEAK, TOKEN_SIZE, Palette::hero(), &[], Island::along(0))?;
        fade_in_all(stage, &[crossing], 0.2)?;
        move_all(stage, &[(crossing, Island::along(1))], 0.6)?;
        fade_out_all(stage, &[crossing, island_values[1]], 0.2)?;
        let second_peak = kit.tex(stage, PEAK, LABEL_SIZE, Palette::hero(), &[], Island::label_spot(1))?;
        fade_in_all(stage, &[second_peak], 0.3)?;
        island_values[1] = second_peak;
        recolor(stage, &[(island.junctions[1], Palette::hero())], 0.3)?;
        line.wait_for(stage, "never reaching")?;
        let no_wire = kit.text(stage, NO_WIRE, NOTE_SIZE, Palette::dim(), Island::under(NOTE_Y[0]))?;
        fade_in_all(stage, &[no_wire], 0.4)?;
        line.wait_for(stage, "Any equal pair")?;
        let (island_net, cut) = facts::island().map_err(fail)?;
        let mut rule_row: Vec<Mob> = Vec::new();
        for c in facts::island_levels() {
            let values = facts::with_island_values(c, c).map_err(fail)?;
            if !facts::keeps_rule(&island_net, &values) {
                return Err(fail(format!("the equal pair {c}, {c} breaks the island's rule")));
            }
            let next = island_labels(&kit, stage, &tex(c), Palette::ink())?;
            let row = kit.tex(
                stage,
                &equation(&[values[cut[1]]], values[cut[0]]),
                EQUATION_SIZE,
                Palette::ink(),
                &[],
                Island::under(NOTE_Y[1]),
            )?;
            let mut old = island_values.to_vec();
            old.extend(&rule_row);
            fade_out_all(stage, &old, 0.15)?;
            fade_in_all(stage, &[next[0], next[1], row], 0.15)?;
            recolor(
                stage,
                &[
                    (island.junctions[0], palette::value(values[cut[0]])),
                    (island.junctions[1], palette::value(values[cut[1]])),
                ],
                0.3,
            )?;
            pause(stage, LEVEL_S)?;
            island_values = next;
            rule_row = vec![row];
        }

        // 04_8: the hypothesis' card; the circuit's wires reach the top and
        // the bottom, the island's do not
        let line = say(&kit, stage, "04_8")?;
        let hypothesis_text = hypothesis_card().map_err(fail)?;
        let hypothesis = precisely(&kit, stage, &hypothesis_text)?;
        fade_in_all(stage, &[hypothesis], CARD_IN)?;
        let hypothesis_read = stage.scene().time().to_f64() + read_s(&hypothesis_text);
        line.wait_for(stage, "along wires")?;
        let circuit_wires: Vec<Mob> = circuit.strands.iter().map(|s| s.mob).collect();
        flash_all(stage, &circuit_wires, 0.6)?;
        flash_all(stage, &[island.wire], 0.6)?;

        // 04_9: a walker on the island, never at an end; its chance of
        // reaching the top first is 0, while the island's voltage is free
        let line = say(&kit, stage, "04_9")?;
        let walker = dot(stage, Island::spot(0), WALKER_R, palette::walker());
        fade_in_all(stage, &[walker], 0.3)?;
        for k in 1..=WALKER_STEPS {
            move_all(stage, &[(walker, Island::spot(k % 2))], STEP_S)?;
        }
        line.wait_for(stage, "Its chance is zero")?;
        let chance = kit.text(
            stage,
            &chance_note().map_err(fail)?,
            NOTE_SIZE,
            Palette::ink(),
            Island::under(NOTE_Y[0]),
        )?;
        let mut old = vec![no_wire];
        old.extend(&rule_row);
        fade_out_all(stage, &old, 0.2)?;
        fade_in_all(stage, &[chance], 0.2)?;
        line.wait_for(stage, "voltage is left free")?;
        let free = island_labels(&kit, stage, FREE, Palette::ink())?;
        let voltage = kit.text(stage, VOLTAGE_NOTE, NOTE_SIZE, Palette::dim(), Island::under(NOTE_Y[1]))?;
        fade_out_all(stage, &island_values, 0.2)?;
        fade_in_all(stage, &[free[0], free[1], voltage], 0.2)?;
        recolor(
            stage,
            &[
                (island.junctions[0], palette::unknown()),
                (island.junctions[1], palette::unknown()),
            ],
            0.4,
        )?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        wait_until(stage, hypothesis_read)?;

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
            ("04_1", "Suppose another"),
            ("04_1", "the same top and bottom"),
            ("04_2", "The difference is zero"),
            ("04_2", "keeps the rule too"),
            ("04_3", "If above zero"),
            ("04_3", "an average of far ends"),
            ("04_4", "every part equals it"),
            ("04_4", "So the peak spreads"),
            ("04_4", "which is zero"),
            ("04_5", "likewise the lowest"),
            ("04_5", "Both solutions are the same"),
            ("04_6", "two junctions wired"),
            ("04_7", "crosses its one wire"),
            ("04_7", "never reaching"),
            ("04_7", "Any equal pair"),
            ("04_8", "along wires"),
            ("04_9", "Its chance is zero"),
            ("04_9", "voltage is left free"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The values beside the peak's wires are its far ends': 0 from the
    /// pair to the top and from the bottom, at most the peak from the right;
    /// the peak's path to the top is one wire; the difference the argument
    /// leaves is 0 everywhere.
    #[test]
    fn the_difference_is_the_facts() {
        let net = facts::network().unwrap();
        let (top, bottom) = facts::difference_ends().unwrap();
        assert_eq!((top, bottom), (Q::ZERO, Q::ZERO));
        let parts: Vec<String> = part_texts(&net, LEFT, top)
            .unwrap()
            .into_iter()
            .map(|(src, _)| src)
            .collect();
        assert_eq!(parts, ["0", "0", AT_MOST_PEAK, "0"]);
        assert_eq!(net.path(LEFT, TOP), Some(vec![LEFT, TOP]));
        let difference = net.kirchhoff(top, bottom).unwrap();
        assert!(difference.iter().all(|q| q.is_zero()));
        // the island shown is the model's: two junctions, one wire, cut off
        let (island, cut) = facts::island().unwrap();
        assert_eq!(cut.len(), ISLAND_AT.len());
        let inside: Vec<_> = island
            .wires()
            .iter()
            .filter(|w| cut.contains(&w.a) && cut.contains(&w.b))
            .collect();
        assert_eq!(inside.len(), 1);
        assert_eq!(inside[0].conductance, Q::ONE);
        // an unequal pair breaks the island's rule; the equal ones keep it
        assert!(!facts::keeps_rule(
            &island,
            &facts::with_island_values(Q::ZERO, Q::ONE).unwrap()
        ));
        for c in facts::island_levels() {
            assert!(facts::keeps_rule(&island, &facts::with_island_values(c, c).unwrap()));
        }
    }

    /// The peak reaches each far end of the left junction as one mark: the
    /// pair to the top gives one, not two. Each lands clear of its dot and of
    /// every wire (allowing for a pair's bow): right of the top and the
    /// bottom, whose 0 is on the left, and at the right junction where its
    /// value goes.
    #[test]
    fn the_peak_lands_once_per_far_end_and_clear() {
        use crate::chapters::draw::SPREAD;
        let net = facts::network().unwrap();
        let ends = net.far_ends(LEFT).unwrap();
        assert_eq!(ends, [TOP, TOP, RIGHT, BOTTOM]);
        let (kept, rest) = one_per_end(&ends);
        assert_eq!((kept.clone(), rest), (vec![0, 2, 3], vec![1]));
        let distinct: Vec<usize> = kept.iter().map(|&i| ends[i]).collect();
        assert_eq!(distinct, [TOP, RIGHT, BOTTOM]);
        // from a point to the segment between two junctions
        let to_wire = |p: [f64; 2], a: [f64; 2], b: [f64; 2]| {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
            (p[0] - a[0] - t * dx).hypot(p[1] - a[1] - t * dy)
        };
        for y in [TOP, BOTTOM] {
            let at = landing(&net, y);
            assert!(at[0] > SPOTS[y][0], "the M lands right of junction {y}, opposite its 0");
            assert!((at[0] - SPOTS[y][0]).hypot(at[1] - SPOTS[y][1]) > JUNCTION_R + 0.3);
            for w in net.wires() {
                let gap = to_wire(at, SPOTS[w.a], SPOTS[w.b]) - SPREAD / 2.0;
                assert!(gap > 0.25, "junction {y}'s M is {gap:.2} from the wire {}-{}", w.a, w.b);
            }
        }
        // at a middle junction it lands where the value goes
        let at = landing(&net, RIGHT);
        assert!((at[0] - (SPOTS[RIGHT][0] + LABEL_GAP)).abs() < 1e-12 && (at[1] - SPOTS[RIGHT][1]).abs() < 1e-12);
    }

    /// The exact versions say what the facts say, and fit a lower-third of
    /// three lines; the walker's note gives the chance from the facts.
    #[test]
    fn the_exact_versions_are_built_from_the_facts_and_fit() {
        let difference = difference_card().unwrap();
        assert!(
            difference.contains("so their difference is the average of the far ends' differences"),
            "{difference}"
        );
        assert!(
            difference.ends_with("At the top it is 1 − 1, at the bottom 0 − 0: both 0."),
            "{difference}"
        );
        let peak = peak_card().unwrap();
        assert!(
            peak.starts_with("with positive weights an average is at most its largest part"),
            "{peak}"
        );
        // the route is this example's: here, to the top
        assert!(
            peak.contains("and on along wires; here, to the top, where it is 0. Likewise for the lowest value."),
            "{peak}"
        );
        let hypothesis = hypothesis_card().unwrap();
        assert!(
            hypothesis.contains("if every middle junction has a path of wires to the top or the bottom"),
            "{hypothesis}"
        );
        // the maximum and minimum argument proves at most one solution
        assert!(hypothesis.contains("has at most one solution"), "{hypothesis}");
        assert!(!hypothesis.contains("exactly"), "{hypothesis}");
        assert!(hypothesis.starts_with("with positive conductances,"), "{hypothesis}");
        assert!(hypothesis.ends_with("any equal pair keeps its rule."), "{hypothesis}");
        assert_eq!(chance_note().unwrap(), "chance of reaching the top first: 0");
        for text in [difference, peak, hypothesis] {
            let lines = explainer_kit::animatic::wrap(&text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
    }

    /// Every card can be read at fifteen characters a second in the time it
    /// stays up, by the scratch voice's timing of the lines it stays over
    /// (with this video's thought-following gaps), so the guards add no time
    /// to the draft; with a real voice they hold a card longer.
    #[test]
    fn every_card_stays_up_long_enough_to_read() {
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let gap = |a: &str, b: &str| voice.line_gap(a, b);
        // what is left of a line once `phrase` begins, read evenly
        let after = |id: &str, phrase: &str| {
            let text = script.text(id).unwrap();
            let at = text[..text.find(phrase).unwrap()].chars().count() as f64;
            len(id) * (1.0 - at / text.chars().count() as f64)
        };
        let windows = [
            // in at "The difference is zero", up over 04_3, out as 04_4 begins
            (
                difference_card().unwrap(),
                after("04_2", "The difference is zero") - CARD_IN
                    + gap("04_2", "04_3")
                    + len("04_3")
                    + gap("04_3", "04_4"),
            ),
            // in as 04_4 begins, out as the 04_6 question ends
            (
                peak_card().unwrap(),
                len("04_4") - CARD_OUT + gap("04_4", "04_5") + len("04_5") + gap("04_5", "04_6") + len("04_6")
                    - CARD_OUT,
            ),
            // in as 04_8 begins, up to the beat after 04_9
            (
                hypothesis_card().unwrap(),
                len("04_8") - CARD_IN + gap("04_8", "04_9") + len("04_9") + END_BEAT,
            ),
        ];
        for (text, up) in windows {
            let need = read_s(&text);
            assert!(need <= up, "{need:.2} s to read, {up:.2} s up: {text}");
        }
    }

    /// The copy and the island stay right of the battery and inside the
    /// title-safe area; the island's notes sit above a three-line
    /// lower-third, and the countdown under them, its note above the frame's
    /// safe bottom.
    #[test]
    fn the_pictures_keep_clear_of_each_other_and_the_card() {
        use crate::chapters::draw::{BATTERY_X, NAME_DY, THINK_R};
        // the copy's right junction's value, inside the frame
        const { assert!(COPY_AT[0] + SPOTS[RIGHT][0] + LABEL_GAP + 0.5 < FRAME_X_RADIUS - 0.25) };
        // the copy's top value, right of the battery
        const { assert!(COPY_AT[0] - END_LABEL_GAP - 0.4 > CIRCUIT_AT[0] + BATTERY_X + 0.5) };
        // the island: right of the battery, inside the frame, its name below
        // the pictures' names
        const { assert!(ISLAND_AT[0][0] - JUNCTION_R - 0.3 > CIRCUIT_AT[0] + BATTERY_X + 0.5) };
        const { assert!(ISLAND_AT[1][0] + JUNCTION_R + 0.3 < FRAME_X_RADIUS - 0.25) };
        const { assert!(ISLAND_AT[0][1] + ISLAND_NAME_DY + 0.3 < CIRCUIT_AT[1] + NAME_DY - 0.3) };
        // an island value, even a fraction, clears its dot below and the
        // name above
        const { assert!(ISLAND_LABEL_DY - FRACTION_HALF > JUNCTION_R + 0.1) };
        const { assert!(ISLAND_LABEL_DY + FRACTION_HALF + 0.2 < ISLAND_NAME_DY) };
        // the crossing M (half a glyph, 0.15) clears the wire below and the
        // M values above it (the fractions come after it has gone), and
        // sets out and stops clear of both dots
        const { assert!(CROSS_DY - 0.15 > 0.1 && CROSS_DY + 0.15 + 0.1 < ISLAND_LABEL_DY - 0.2) };
        const { assert!(CROSS_INSET > JUNCTION_R + 0.25) };
        // its notes above a three-line lower-third, the countdown below them
        const { assert!(NOTE_Y[1] - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        const { assert!(THINK_AT[1] + THINK_R < NOTE_Y[0] - 0.25) };
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 0.15) };
        // the island's wire is level: its two junctions at one height
        assert!((ISLAND_AT[0][1] - ISLAND_AT[1][1]).abs() < 1e-12);
    }
}
