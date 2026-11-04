//! Chapter 08, subgroups. The rack of eight moves from chapter 07 stays on
//! screen while subsets are tested: the rotations close up; {e, r, r²} leaks
//! r³ (the rigid tile performs r then r² and lands outside the box); the
//! closure machine then grows subgroups from generators, filling a copy of
//! the rack slot by slot, round by round; all ten subgroups drop onto size
//! shelves 1..8, leaving 3, 5, 6 and 7 empty; finally a two-slot test
//! machine is fed (h, h), (e, h) and (a, b⁻¹). Every set shown is computed
//! by `d4_group`; nothing is typed.
use crate::d4_group::{self as d4g, Elem, R, R2, R3, S};
use crate::{tile_actor, tile_model};
use explainer_kit::{Kit, Palette, Timed, play, rect, stroke_style, tween, v};
use fmn::prelude::*;

pub struct Subgroups {
    pub kit: Kit,
}

const RACK_X0: f64 = -5.55;
const RACK_DX: f64 = 1.35;
const BIN_X0: f64 = 1.25;
const BIN_DX: f64 = 1.5;
const ROW_Y: [f64; 2] = [0.95, -0.75];
const TILE_X: f64 = 3.6;
const TILE_Y: f64 = 0.35;

/// The three collections fed to the closure machine (08_8, 08_9, 08_10);
/// `facts` counts their passes from the same arrays.
pub(crate) const FLIP_ALONE: [Elem; 1] = [S];
pub(crate) const FLIP_AND_HALF_TURN: [Elem; 2] = [S, R2];
pub(crate) const QUARTER_TURN_AND_FLIP: [Elem; 2] = [R, S];

fn rack_at(g: Elem) -> (f64, f64) {
    (RACK_X0 + RACK_DX * f64::from(g.turns), ROW_Y[usize::from(g.flip)])
}

fn bin_at(g: Elem) -> (f64, f64) {
    (BIN_X0 + BIN_DX * f64::from(g.turns), ROW_Y[usize::from(g.flip)])
}

/// A frame around rack columns `from..=to` of the rotation row, names included.
fn row_box(stage: &mut Stage<'_>, from: u8, to: u8, color: Srgb) -> fmn::Result<Mob> {
    let left = RACK_X0 + RACK_DX * f64::from(from) - 0.65;
    let width = RACK_DX * f64::from(to - from) + 1.3;
    Ok(stage
        .arena_mut()
        .add(rect(v(left, ROW_Y[0] - 1.02), width, 1.7, stroke_style(color, 4.0))?))
}

/// TeX for a set of moves, built from the computed members.
fn set_tex(members: &[Elem]) -> String {
    let names: Vec<&str> = members.iter().map(|g| g.tex()).collect();
    format!(r"\{{{}\}}", names.join(","))
}

/// The closure machine fed `generators`: each round's new moves fly from the
/// rack into their own slots of the bin while that round's products are shown.
struct Machine {
    copies: Vec<Mob>,
    marks: Vec<Mob>,
}

impl Machine {
    fn clear(self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut all = self.copies;
        all.extend(self.marks);
        d4g::hide(stage, &all, 0.4)
    }
}

fn fly_in(stage: &mut Stage<'_>, kit: &Kit, moves: &[Elem], machine: &mut Machine) -> fmn::Result<()> {
    let mut copies = Vec::new();
    for &g in moves {
        let copy = d4g::thumb(stage, kit, g, rack_at(g), 1.0)?;
        copies.push(copy);
    }
    d4g::show(stage, &copies, 0.25)?;
    let targets: Vec<(Mob, (f64, f64))> = copies.iter().copied().zip(moves.iter().map(|&g| bin_at(g))).collect();
    d4g::glide_all(stage, &targets, 1.0)?;
    machine.copies.extend(copies);
    Ok(())
}

fn product_line(stage: &mut Stage<'_>, kit: &Kit, pairs: &[(Elem, Elem)], color: Srgb) -> fmn::Result<Mob> {
    let text: Vec<String> = pairs
        .iter()
        .map(|&(a, b)| format!("{}\\,{}={}", a.tex(), b.tex(), a.mul(b).tex()))
        .collect();
    let line = kit.tex(stage, &text.join(r",\quad "), 32.0, color, &[], v(3.55, -2.3))?;
    d4g::show(stage, &[line], 0.4)?;
    Ok(line)
}

/// The products that first produced each new move of `round`.
fn witnesses(have: &[Elem], round: &[Elem]) -> Vec<(Elem, Elem)> {
    round
        .iter()
        .map(|&target| {
            have.iter()
                .flat_map(|&a| have.iter().map(move |&b| (a, b)))
                .find(|&(a, b)| a.mul(b) == target)
                .expect("each new move is a product of moves already present")
        })
        .collect()
}

fn done_frame(stage: &mut Stage<'_>, machine: &mut Machine) -> fmn::Result<()> {
    let frame = stage
        .arena_mut()
        .add(rect(v(0.4, -1.75), 6.3, 3.5, stroke_style(Palette::green(), 5.0))?);
    play_creation(stage, frame)?;
    machine.marks.push(frame);
    Ok(())
}

fn play_creation(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let a = show_creation(mob).rt(0.6);
    let a = stage.prepare(a)?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

/// Run the machine on `generators`, pacing each round with sentence `cues`.
fn run_machine(
    stage: &mut Stage<'_>,
    kit: &Kit,
    cue: &str,
    generators: &[Elem],
    round_sentences: &[usize],
) -> fmn::Result<Machine> {
    let rounds = d4g::closure_rounds(generators);
    let mut machine = Machine {
        copies: Vec::new(),
        marks: Vec::new(),
    };
    let mut have: Vec<Elem> = Vec::new();
    for (index, round) in rounds.iter().enumerate() {
        if let Some(&sentence) = round_sentences.get(index) {
            kit.until_sentence(stage, cue, sentence)?;
        }
        if index > 0 {
            let line = product_line(stage, kit, &witnesses(&have, round), Palette::ink())?;
            machine.marks.push(line);
        }
        fly_in(stage, kit, round, &mut machine)?;
        have.extend(round.iter().copied());
        // Show the first witness as an action, not merely ss=e.
        if index == 0 && generators == FLIP_ALONE {
            kit.until_sentence(stage, cue, 1)?;
            let flip_progress = stage.add_value_tracker(0.0);
            let witness_tile =
                tile_actor::add_tile(stage, kit, flip_progress, 4.65, 0.3, 0.65, |t| d4g::two_step(S, S, t))?;
            d4g::show(stage, &witness_tile, 0.25)?;
            play!(stage; tween(flip_progress, 1.0, 0.8)?);
            stage.wait(0.2)?;
            play!(stage; tween(flip_progress, 2.0, 0.8)?);
            stage.wait(0.3)?;
            d4g::hide(stage, &witness_tile, 0.25)?;
        }
        if index > 0 {
            let old = machine.marks.pop().expect("product line just pushed");
            d4g::hide(stage, &[old], 0.3)?;
        }
    }
    Ok(machine)
}

fn token(stage: &mut Stage<'_>, kit: &Kit, tex: &str, color: Srgb, at: (f64, f64)) -> fmn::Result<Mob> {
    kit.tex(stage, tex, 40.0, color, &[], v(at.0, at.1))
}

/// The one-step test drawn as a machine: two inputs leave the pool, enter
/// the slots a and b, and the output a·b⁻¹ drops back into the pool.
struct Tester {
    body: Mob,
}

impl Tester {
    const SLOT_A: (f64, f64) = (-2.6, 0.95);
    const SLOT_B: (f64, f64) = (-2.6, 0.05);
    const OUT: (f64, f64) = (2.55, 0.5);

    fn feed(
        &self,
        stage: &mut Stage<'_>,
        kit: &Kit,
        inputs: [&str; 2],
        output: (&str, Srgb),
        spot: (f64, f64),
    ) -> fmn::Result<Vec<Mob>> {
        let a = token(stage, kit, inputs[0], Palette::ink(), (4.9, 0.7))?;
        let b = token(stage, kit, inputs[1], Palette::ink(), (5.6, 0.7))?;
        d4g::show(stage, &[a, b], 0.3)?;
        d4g::glide_all(stage, &[(a, Self::SLOT_A), (b, Self::SLOT_B)], 0.9)?;
        let pulse = fmn::animation::indicate(stage.arena_mut(), self.body, 1.08, None)?;
        play!(stage; pulse.rt(0.7));
        let out = token(stage, kit, output.0, output.1, Self::OUT)?;
        d4g::hide(stage, &[a, b], 0.3)?;
        d4g::show(stage, &[out], 0.4)?;
        stage.wait(0.5)?;
        d4g::glide(stage, out, spot, 0.9)?;
        Ok(vec![out])
    }
}

impl SceneConstruct for Subgroups {
    fn name(&self) -> &str {
        "08_subgroups"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 08_1: the rack from chapter 07; keep only the rotations.
        kit.say(stage, "08_1")?;
        let title = kit.text(stage, "A group inside a group", 36.0, Palette::ink(), v(0.0, 3.3))?;
        let mut rack = Vec::new();
        for g in d4g::ALL {
            let (x, y) = rack_at(g);
            rack.push(d4g::thumb(stage, &kit, g, (x, y), 1.0)?);
            rack.push(kit.text(stage, g.name(), 24.0, Palette::ink(), v(x, y - 0.68))?);
        }
        d4g::show(stage, &[title], 0.4)?;
        d4g::show(stage, &rack, 0.6)?;
        let rotations_box = row_box(stage, 0, 3, Palette::blue())?;
        play_creation(stage, rotations_box)?;
        // One rigid tile cycles through the rotations, then (08_3) does r, then r².
        let spin = stage.add_value_tracker(0.0);
        let hero = tile_actor::add_tile(stage, &kit, spin, TILE_X, TILE_Y, 1.0, |t| {
            if t <= 4.0 {
                tile_model::turn(t)
            } else {
                d4g::two_step(R, R2, t - 4.0)
            }
        })?;
        kit.until_sentence(stage, "08_1", 1)?;
        for quarter in 1..=4 {
            play!(stage; tween(spin, f64::from(quarter), 0.8)?);
            stage.wait(0.25)?;
        }
        kit.until_sentence(stage, "08_1", 2)?;
        let face = kit.text(
            stage,
            "Rotations never turn the tile over",
            28.0,
            Palette::blue(),
            v(TILE_X, -1.55),
        )?;
        let caption = kit.text(
            stage,
            "Same operation, smaller collection: a subgroup",
            28.0,
            Palette::ink(),
            v(0.0, -3.3),
        )?;
        d4g::show(stage, &[face], 0.4)?;
        kit.until_sentence(stage, "08_1", 3)?;
        d4g::show(stage, &[caption], 0.4)?;

        // 08_2: leave out r³. Prediction hold.
        kit.say(stage, "08_2")?;
        d4g::hide(stage, &[face, caption], 0.3)?;
        let three_box = row_box(stage, 0, 2, Palette::blue())?;
        play!(stage; fmn::animation::replacement_transform(rotations_box, three_box).rt(0.8));
        let left_out = d4g::slot_frame(stage, rack_at(R3), 1.0, Palette::gold())?;
        play_creation(stage, left_out)?;
        let question = kit.text(
            stage,
            "Closed under composition?",
            32.0,
            Palette::hero(),
            v(TILE_X, -1.55),
        )?;
        d4g::show(stage, &[question], 0.4)?;
        kit.hold(stage)?;
        stage.wait(2.5)?;

        // 08_3: quarter-turn, then half-turn, on the rigid tile.
        kit.say(stage, "08_3")?;
        d4g::hide(stage, &[question], 0.3)?;
        kit.until_sentence(stage, "08_3", 1)?;
        play!(stage; tween(spin, 5.0, 1.0)?);
        stage.wait(0.2)?;
        play!(stage; tween(spin, 6.0, 1.4)?);
        let leak = kit.tex(
            stage,
            r"r^2r=r^3\notin\{e,r,r^2\}",
            34.0,
            Palette::gold(),
            &[],
            v(TILE_X, -1.55),
        )?;
        d4g::show(stage, &[leak], 0.4)?;
        let flash = fmn::animation::indicate(stage.arena_mut(), rack[2 * R3.index()], 1.25, None)?;
        play!(stage; flash.rt(0.9));
        kit.until_sentence(stage, "08_3", 2)?;
        let not_subgroup = kit.text(
            stage,
            "One failed pair: not a subgroup",
            28.0,
            Palette::gold(),
            v(0.0, -3.3),
        )?;
        d4g::show(stage, &[not_subgroup], 0.4)?;

        // 08_7: repair by adding the missing product; build the machine.
        kit.say(stage, "08_7")?;
        kit.until_sentence(stage, "08_7", 2)?;
        let four_box = row_box(stage, 0, 3, Palette::blue())?;
        play!(stage; fmn::animation::replacement_transform(three_box, four_box).rt(0.8));
        d4g::hide(stage, &[left_out, leak, not_subgroup], 0.3)?;
        kit.until_sentence(stage, "08_7", 3)?;
        let mut retire = hero;
        retire.push(four_box);
        d4g::hide(stage, &retire, 0.4)?;
        let bin = stage
            .arena_mut()
            .add(rect(v(0.4, -1.75), 6.3, 3.5, stroke_style(Palette::dim(), 3.0))?);
        let bin_title = kit.text(stage, "Closure machine", 30.0, Palette::ink(), v(3.55, 2.2))?;
        d4g::show(stage, &[bin, bin_title], 0.5)?;
        kit.until_sentence(stage, "08_7", 5)?;
        let rule = kit.text(
            stage,
            "Combine every pair · add what is new · repeat",
            28.0,
            Palette::ink(),
            v(0.0, -3.3),
        )?;
        d4g::show(stage, &[rule], 0.4)?;

        // 08_8: feed {s}.
        kit.say(stage, "08_8")?;
        let fed = d4g::slot_frame(stage, rack_at(S), 1.0, Palette::hero())?;
        play_creation(stage, fed)?;
        let mut machine = run_machine(stage, &kit, "08_8", &FLIP_ALONE, &[0, 2])?;
        kit.until_sentence(stage, "08_8", 3)?;
        done_frame(stage, &mut machine)?;
        let grown = d4g::closure_rounds(&FLIP_ALONE).concat();
        let result = kit.tex(stage, &set_tex(&grown), 34.0, Palette::green(), &[], v(3.55, -2.3))?;
        d4g::show(stage, &[result], 0.4)?;
        machine.marks.push(result);
        machine.marks.push(fed);

        // 08_9: feed {s, r²}; the four-move subgroup K.
        kit.say(stage, "08_9")?;
        machine.clear(stage)?;
        let mut fed = Vec::new();
        for g in FLIP_AND_HALF_TURN {
            fed.push(d4g::slot_frame(stage, rack_at(g), 1.0, Palette::hero())?);
        }
        d4g::show(stage, &fed, 0.3)?;
        let mut machine = run_machine(stage, &kit, "08_9", &FLIP_AND_HALF_TURN, &[0, 1])?;
        kit.until_sentence(stage, "08_9", 2)?;
        let pass = kit.text(stage, "Second pass: nothing new", 26.0, Palette::green(), v(3.55, -2.3))?;
        d4g::show(stage, &[pass], 0.3)?;
        done_frame(stage, &mut machine)?;
        kit.until_sentence(stage, "08_9", 3)?;
        let grown = d4g::closure_rounds(&FLIP_AND_HALF_TURN).concat();
        let ordered: Vec<Elem> = d4g::ALL.into_iter().filter(|g| grown.contains(g)).collect();
        let k_label = kit.tex(stage, &set_tex(&ordered), 34.0, Palette::green(), &[], v(3.55, -2.3))?;
        play!(stage; fmn::animation::replacement_transform(pass, k_label).rt(0.6));
        machine.marks.push(k_label);
        machine.marks.extend(fed);
        let mut remembered = Vec::new();
        for &g in &grown {
            remembered.push(d4g::slot_frame(stage, rack_at(g), 1.0, Palette::green())?);
        }
        d4g::show(stage, &remembered, 0.4)?;

        // 08_10: feed {r, s}; the whole rack.
        kit.say(stage, "08_10")?;
        machine.clear(stage)?;
        d4g::hide(stage, &remembered, 0.3)?;
        let mut fed = Vec::new();
        for g in QUARTER_TURN_AND_FLIP {
            fed.push(d4g::slot_frame(stage, rack_at(g), 1.0, Palette::hero())?);
        }
        d4g::show(stage, &fed, 0.3)?;
        let mut machine = run_machine(stage, &kit, "08_10", &QUARTER_TURN_AND_FLIP, &[0, 1, 1])?;
        done_frame(stage, &mut machine)?;
        kit.until_sentence(stage, "08_10", 2)?;
        let whole = kit.text(
            stage,
            "r and s generate all eight",
            28.0,
            Palette::green(),
            v(3.55, -2.3),
        )?;
        d4g::show(stage, &[whole], 0.4)?;
        machine.marks.push(whole);
        machine.marks.extend(fed);

        // 08_11: all ten subgroups on shelves by size.
        kit.say(stage, "08_11")?;
        machine.clear(stage)?;
        let mut wipe = rack.clone();
        wipe.extend([bin, bin_title, rule, title]);
        d4g::hide(stage, &wipe, 0.5)?;
        let heading = kit.text(
            stage,
            "Every subgroup of the square, by size",
            34.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        let shelf_y = |size: usize| -2.85 + 0.75 * (size as f64 - 1.0);
        let mut shelves = vec![heading];
        let mut shelf_labels = Vec::new();
        for size in 1..=8 {
            let y = shelf_y(size);
            let line = stage.arena_mut().add(
                Line::new(v(-4.9, y - 0.3), v(6.7, y - 0.3))
                    .style(stroke_style(Palette::faint(), 2.0))
                    .build()?,
            );
            let label = kit.text(stage, &format!("size {size}"), 24.0, Palette::dim(), v(-5.9, y))?;
            shelves.extend([line, label]);
            shelf_labels.push(label);
        }
        d4g::show(stage, &shelves, 0.5)?;
        let all = d4g::subgroups();
        let mut nodes: Vec<Vec<Mob>> = vec![Vec::new(); 9];
        for size in [1_usize, 2, 4, 8] {
            let members: Vec<&Vec<Elem>> = all.iter().filter(|h| h.len() == size).collect();
            let spacing = if members.len() > 3 { 2.2 } else { 3.4 };
            let start = 0.9 - spacing * (members.len() as f64 - 1.0) / 2.0;
            for (index, h) in members.iter().enumerate() {
                let x = start + spacing * index as f64;
                let label = if size == 8 {
                    r"\text{all eight moves}".to_owned()
                } else {
                    set_tex(h)
                };
                let node = kit.tex(stage, &label, 26.0, Palette::blue(), &[], v(x, shelf_y(size) + 1.0))?;
                nodes[size].push(node);
            }
        }
        for (size, sentence) in [(1, 1), (2, 2), (4, 3), (8, 4)] {
            kit.until_sentence(stage, "08_11", sentence)?;
            let mut drops: Vec<Box<dyn Animation>> = Vec::new();
            for &node in &nodes[size] {
                let a = fade_in(stage.arena_mut(), node, v(0.0, 0.6), 1.0)?.rt(0.6);
                drops.push(stage.prepare(a)?);
            }
            let drops = explainer_kit::lagged(stage, drops, 0.25)?;
            play!(stage; drops);
            let mut settle: Vec<Box<dyn Animation>> = Vec::new();
            for &node in &nodes[size] {
                let x = stage.get_center(node)[0];
                let a = node
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.5),
                        ..AnimateArgs::default()
                    })?
                    .move_to(v(x, shelf_y(size)), ORIGIN)?;
                settle.push(stage.prepare(a)?);
            }
            stage.play_prepared(settle)?;
        }
        kit.until_sentence(stage, "08_11", 6)?;
        let mut lit: Vec<Box<dyn Animation>> = Vec::new();
        for size in [1_usize, 2, 4, 8] {
            lit.push(stage.prepare(shelf_labels[size - 1].animate().set_color(Palette::green())?)?);
        }
        stage.play_prepared(lit)?;
        kit.until_sentence(stage, "08_11", 7)?;
        let mut empty = Vec::new();
        for size in [3_usize, 5, 6, 7] {
            empty.push(kit.text(stage, "none", 24.0, Palette::gold(), v(0.9, shelf_y(size)))?);
        }
        d4g::show(stage, &empty, 0.5)?;
        let mut dim: Vec<Box<dyn Animation>> = Vec::new();
        for size in [3_usize, 5, 6, 7] {
            dim.push(stage.prepare(shelf_labels[size - 1].animate().set_color(Palette::gold())?)?);
        }
        stage.play_prepared(dim)?;

        // 08_4: the one-step test as a two-slot machine.
        kit.say(stage, "08_4")?;
        let mut old = shelves;
        old.extend(empty);
        for size in [1_usize, 2, 4, 8] {
            old.extend(nodes[size].iter().copied());
        }
        d4g::hide(stage, &old, 0.5)?;
        let statement = kit.tex(
            stage,
            r"a,b\in H\ \Longrightarrow\ ab^{-1}\in H",
            36.0,
            Palette::ink(),
            &[],
            v(0.0, 3.2),
        )?;
        // Keep the theorem domain visible: the empty set passes the pair test
        // vacuously, but is not a subgroup. This premise also stays with the iff.
        let premise = kit.text(stage, "Nonempty subset H of G", 26.0, Palette::ink(), v(0.0, 2.6))?;
        let body = stage.arena_mut().add(
            Rectangle::new()
                .width(3.4)
                .height(1.9)
                .corner_radius(0.2)
                .style(stroke_style(Palette::blue(), 4.0))
                .build()?
                .moved_to(v(0.0, 0.5)),
        );
        let recipe = kit.text(stage, "undo b, then do a", 26.0, Palette::blue(), v(0.0, 0.5))?;
        let slot_a = stage
            .arena_mut()
            .add(rect(v(-3.05, 0.55), 0.9, 0.8, stroke_style(Palette::dim(), 2.5))?);
        let slot_b = stage
            .arena_mut()
            .add(rect(v(-3.05, -0.35), 0.9, 0.8, stroke_style(Palette::dim(), 2.5))?);
        let tag_a = kit.text(stage, "a", 24.0, Palette::dim(), v(-3.45, 1.6))?;
        let tag_b = kit.text(stage, "b", 24.0, Palette::dim(), v(-3.45, -0.6))?;
        let pool = stage.arena_mut().add(
            Rectangle::new()
                .width(2.7)
                .height(2.9)
                .corner_radius(0.25)
                .style(stroke_style(Palette::green(), 4.0))
                .build()?
                .moved_to(v(5.25, 0.5)),
        );
        let pool_name = kit.tex(stage, "H", 36.0, Palette::green(), &[], v(5.25, 2.25))?;
        // Introduce the claim immediately; do not leave an empty screen
        // during the first two sentences of the one-step-test explanation.
        d4g::show(stage, &[statement, premise], 0.5)?;
        kit.until_sentence(stage, "08_4", 2)?;
        d4g::show(stage, &[pool, pool_name], 0.5)?;
        d4g::show(stage, &[body, recipe, slot_a, slot_b, tag_a, tag_b], 0.5)?;
        let tester = Tester { body };
        let pool_spots = [(4.65, 1.25), (5.85, 1.25), (4.65, 0.25), (5.85, 0.25), (5.25, -0.6)];
        kit.until_sentence(stage, "08_4", 3)?;
        let mut in_pool = tester.feed(stage, &kit, ["a", "b"], (r"ab^{-1}", Palette::hero()), pool_spots[4])?;
        kit.until_sentence(stage, "08_4", 5)?;
        let check = fmn::animation::indicate(stage.arena_mut(), pool, 1.06, None)?;
        play!(stage; check.rt(0.8));

        // 08_5: feed (h, h) -> identity.
        kit.say(stage, "08_5")?;
        d4g::hide(stage, &in_pool, 0.3)?;
        in_pool.clear();
        kit.until_sentence(stage, "08_5", 1)?;
        let member = token(stage, &kit, "h", Palette::hero(), pool_spots[0])?;
        d4g::show(stage, &[member], 0.4)?;
        kit.until_sentence(stage, "08_5", 2)?;
        in_pool.extend(tester.feed(stage, &kit, ["h", "h"], ("e", Palette::green()), pool_spots[1])?);
        kit.until_sentence(stage, "08_5", 4)?;
        let identity = kit.tex(stage, r"hh^{-1}=e", 32.0, Palette::green(), &[], v(-4.3, -2.3))?;
        let identity_name = kit.text(stage, "Identity", 26.0, Palette::green(), v(-4.3, -1.6))?;
        d4g::show(stage, &[identity_name, identity], 0.4)?;

        // 08_12: feed (e, h) -> h inverse.
        kit.say(stage, "08_12")?;
        in_pool.extend(tester.feed(stage, &kit, ["e", "h"], (r"h^{-1}", Palette::hero()), pool_spots[2])?);
        kit.until_sentence(stage, "08_12", 3)?;
        let inverse = kit.tex(stage, r"eh^{-1}=h^{-1}", 32.0, Palette::hero(), &[], v(0.0, -2.3))?;
        let inverse_name = kit.text(stage, "Inverses", 26.0, Palette::hero(), v(0.0, -1.6))?;
        d4g::show(stage, &[inverse_name, inverse], 0.4)?;

        // 08_13: feed (a, b inverse) -> a b.
        kit.say(stage, "08_13")?;
        kit.until_sentence(stage, "08_13", 1)?;
        in_pool.extend(tester.feed(stage, &kit, ["a", r"b^{-1}"], ("ab", Palette::gold()), pool_spots[3])?);
        kit.until_sentence(stage, "08_13", 3)?;
        let closure = kit.tex(stage, r"a(b^{-1})^{-1}=ab", 32.0, Palette::gold(), &[], v(4.3, -2.3))?;
        let closure_name = kit.text(stage, "Closure", 26.0, Palette::gold(), v(4.3, -1.6))?;
        d4g::show(stage, &[closure_name, closure], 0.4)?;

        // 08_6: the three results, associativity inherited, and the converse.
        kit.say(stage, "08_6")?;
        let mut sweep: Vec<Box<dyn Animation>> = Vec::new();
        for mob in [identity, inverse, closure] {
            let pulse = fmn::animation::indicate(stage.arena_mut(), mob, 1.15, None)?.rt(0.8);
            sweep.push(stage.prepare(pulse)?);
        }
        let sweep = explainer_kit::lagged(stage, sweep, 0.5)?;
        play!(stage; sweep);
        kit.until_sentence(stage, "08_6", 1)?;
        let inherited = kit.text(stage, "Associativity is inherited", 28.0, Palette::ink(), v(0.0, -3.25))?;
        d4g::show(stage, &[inherited], 0.4)?;
        kit.until_sentence(stage, "08_6", 2)?;
        let iff = kit.tex(
            stage,
            r"H\text{ is a subgroup}\iff\big(a,b\in H\Rightarrow ab^{-1}\in H\big)",
            34.0,
            Palette::ink(),
            &[],
            v(0.0, 3.2),
        )?;
        play!(stage; fmn::animation::replacement_transform(statement, iff).rt(0.9));
        kit.until_sentence(stage, "08_6", 3)?;
        let mut wipe = in_pool;
        wipe.extend([
            member, pool, pool_name, body, recipe, slot_a, slot_b, tag_a, tag_b, inherited,
        ]);
        wipe.extend([identity, identity_name, inverse, inverse_name, closure, closure_name]);
        d4g::hide(stage, &wipe, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witnesses_produce_each_new_move_from_moves_already_present() {
        for generators in [vec![S], vec![S, R2], vec![R, S]] {
            let rounds = d4g::closure_rounds(&generators);
            let mut have = rounds[0].clone();
            for round in &rounds[1..] {
                for (&(a, b), &target) in witnesses(&have, round).iter().zip(round) {
                    assert!(have.contains(&a) && have.contains(&b));
                    assert_eq!(a.mul(b), target);
                }
                have.extend(round.iter().copied());
            }
        }
    }

    #[test]
    fn the_leaking_product_is_r_cubed_and_the_tile_path_ends_there() {
        assert_eq!(R2.mul(R), R3);
        let end = d4g::two_step(R, R2, 2.0);
        let target = R3.matrix();
        for p in tile_model::CORNERS {
            let (a, b) = (tile_model::apply(end, p), tile_model::apply(target, p));
            assert!((0..3).all(|i| (a[i] - b[i]).abs() < 1e-9));
        }
    }

    #[test]
    fn two_flip_witness_visits_flip_then_identity_on_every_corner() {
        for (time, expected) in [(1.0, S.matrix()), (2.0, S.mul(S).matrix())] {
            let actual = d4g::two_step(S, S, time);
            for corner in tile_model::CORNERS {
                let a = tile_model::apply(actual, corner);
                let b = tile_model::apply(expected, corner);
                assert!((0..3).all(|i| (a[i] - b[i]).abs() < 1e-9));
            }
        }
    }

    #[test]
    fn shelf_labels_are_built_from_the_computed_subgroups() {
        let all = d4g::subgroups();
        assert_eq!(set_tex(&all[0]), r"\{e\}");
        assert!(all.iter().any(|h| set_tex(h) == r"\{e,r^2,s,r^2s\}"));
        assert_eq!(all.iter().filter(|h| h.len() == 2).count(), 5);
    }
}
