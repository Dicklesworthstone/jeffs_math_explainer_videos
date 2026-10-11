//! Chapter 02, bits and ex-or: a row of eight bits under their place values
//! (02_1), five as one, zero, one and a full byte as 255 (02_2), two pieces
//! mixed into one packet by ex-or (02_3), and ex-or as light switches: five
//! over seven, the prediction, the flips that give two, and the flips back
//! to five (02_4–02_6), with the byte-as-a-chord sonification. Plain binary
//! is lit in ink; ex-or work in the ex-or blue. Every number is the bits of
//! the example (oracle B-X4).

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::animation::{indicate, replacement_transform};
use fmn::prelude::*;

use super::draw::{OPLUS, Role, badge, card, group_of, line_style};
use super::register::tile;
use super::sound;
use crate::model::{A, B};

/// 02_2 and 02_4's examples: five, and seven to mix in.
const FIVE: u32 = 5;
const SEVEN: u32 = 7;

/// A row of `n` bit tiles (the top bit at the left), cells `cell` wide.
struct Row {
    tiles: Vec<Mob>,
    bits: Vec<bool>,
    centre: Vec3,
    cell: f64,
    lit: Srgb,
}

impl Row {
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        value: u32,
        n: usize,
        cell: f64,
        centre: Vec3,
        lit: Srgb,
    ) -> fmn::Result<Self> {
        let mut me = Self {
            tiles: Vec::with_capacity(n),
            bits: (0..n).map(|k| value >> (n - 1 - k) & 1 == 1).collect(),
            centre,
            cell,
            lit,
        };
        for k in 0..n {
            let t = tile(kit, stage, me.bits[k], cell, lit, me.at(k))?;
            me.tiles.push(t);
        }
        Ok(me)
    }

    /// The centre of position k (0 = the top bit).
    fn at(&self, k: usize) -> Vec3 {
        let n = self.bits.len() as f64;
        v(
            self.centre[0] + (k as f64 - (n - 1.0) / 2.0) * self.cell,
            self.centre[1],
        )
    }

    fn value(&self) -> u32 {
        self.bits.iter().fold(0, |acc, &b| acc << 1 | u32::from(b))
    }

    /// Set bit k to `one`: the old tile turns into the new one (prepared).
    fn set(&mut self, kit: &Kit, stage: &mut Stage<'_>, k: usize, one: bool) -> fmn::Result<Box<dyn Animation>> {
        let t = tile(kit, stage, one, self.cell, self.lit, self.at(k))?;
        let a = stage.prepare(replacement_transform(self.tiles[k], t))?;
        self.tiles[k] = t;
        self.bits[k] = one;
        Ok(a)
    }

    fn group(&self, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        group_of(stage, &self.tiles)
    }
}

pub struct Bits {
    pub kit: Kit,
}

impl SceneConstruct for Bits {
    fn name(&self) -> &str {
        "02_bits_and_ex_or"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        byte(&kit, stage)?;
        mixing(&kit, stage)?;
        switches(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// 02_1–02_2: eight bits, their place values, five, and the full byte.
fn byte(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let cell = 1.15;
    let mut row = Row::build(kit, stage, 0, 8, cell, v(0.0, 0.2), Palette::ink())?;
    let tiles = row.group(stage)?;
    // the place values, the top bit's at the left
    let places: Vec<Mob> = (0..8)
        .map(|k| {
            let at = row.at(k);
            kit.text(
                stage,
                &(1u32 << (7 - k)).to_string(),
                34.0,
                Palette::dim(),
                v(at[0], at[1] + 0.95),
            )
        })
        .collect::<fmn::Result<_>>()?;
    let doubles: Vec<Mob> = (0..7)
        .map(|k| {
            let (a, b) = (row.at(k), row.at(k + 1));
            kit.text(stage, "×2", 22.0, Palette::dim(), v((a[0] + b[0]) / 2.0, a[1] + 1.45))
        })
        .collect::<fmn::Result<_>>()?;

    kit.say(stage, "02_1")?;
    play!(stage; fade_in(stage.arena_mut(), tiles, v(0.0, -0.2), 1.0)?.rt(1.0).lag(0.08));
    let all_places = group_of(stage, &places)?;
    play!(stage; fade_in(stage.arena_mut(), all_places, v(0.0, -0.1), 1.0)?.rt(1.2).lag(0.1));
    let all_doubles = group_of(stage, &doubles)?;
    play!(stage; fade_in(stage.arena_mut(), all_doubles, ORIGIN, 1.0)?.rt(1.4).lag(0.12));
    stage.wait(1.8)?;
    // "One, two, four, eight": the four lowest places, from the right
    for k in (4..8).rev() {
        let a = indicate(stage.arena_mut(), places[k], 1.5, Some([1.0, 1.0, 1.0]))?;
        stage.play(a.rt(0.6))?;
    }
    kit.hold(stage)?;

    // 02_2: five is four plus one; a byte holds 0 … 255
    kit.say(stage, "02_2")?;
    let mut flips = Vec::new();
    for k in 0..8 {
        let one = FIVE >> (7 - k) & 1 == 1;
        if one {
            flips.push(row.set(kit, stage, k, true)?);
        }
    }
    stage.play_prepared_with(
        flips,
        PlayOverrides {
            run_time: Some(0.7),
            ..PlayOverrides::default()
        },
    )?;
    let sum = kit.tex(
        stage,
        &format!(r"4 + 1 = {}", row.value()),
        64.0,
        Palette::ink(),
        &[],
        v(0.0, -1.3),
    )?;
    play!(stage; write(stage.arena(), sum).rt(1.0));
    stage.wait(1.6)?;
    let mut flips = Vec::new();
    for k in 0..8 {
        if !row.bits[k] {
            flips.push(row.set(kit, stage, k, true)?);
        }
    }
    let full = AnimationGroup::with_lag_ratio(stage.arena_mut(), flips, 0.12)?;
    stage.play(full.rt(1.4))?;
    let max = row.value();
    let range = kit.tex(
        stage,
        &format!(r"\text{{a byte: }} 0 \text{{ to }} {max}"),
        64.0,
        Palette::ink(),
        &[],
        v(0.0, -1.3),
    )?;
    play!(stage; replacement_transform(sum, range).rt(0.9));
    kit.hold(stage)?;
    fade_all(stage, 0.6)?;
    Ok(())
}

/// 02_3: two pieces mixed into one packet; the mixing move is ex-or.
fn mixing(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let left = badge(kit, stage, 0, 0.7, v(-4.2, 0.5))?;
    let right = badge(kit, stage, 1, 0.7, v(4.2, 0.5))?;
    let packet = card(kit, stage, A | B, None, 2.4, v(0.0, 0.5))?;
    let [(_, ba), (_, bb)] = [packet.badges[0], packet.badges[1]];
    let name = kit.tex(stage, r"\oplus\ \text{ex-or}", 84.0, Role::ex_or(), &[], v(0.0, -1.9))?;
    kit.say(stage, "02_3")?;
    play!(stage;
        fade_in(stage.arena_mut(), left, v(-0.3, 0.0), 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), right, v(0.3, 0.0), 1.0)?.rt(0.8),
    );
    stage.wait(2.6)?;
    // the two pieces slide together onto one card, its paper and ⊕ arriving
    // under them; then the card takes over its own badges
    let (ta, tb) = (stage.get_center(ba), stage.get_center(bb));
    let op = packet.ops[0];
    play!(stage;
        left.animate().set_anim_args(AnimateArgs { run_time: Some(1.2), ..AnimateArgs::default() })?.move_to(ta, ORIGIN)?,
        right.animate().set_anim_args(AnimateArgs { run_time: Some(1.2), ..AnimateArgs::default() })?.move_to(tb, ORIGIN)?,
        fade_in(stage.arena_mut(), packet.back, ORIGIN, 1.0)?.rt(1.2),
        fade_in(stage.arena_mut(), op, ORIGIN, 1.0)?.rt(1.2),
    );
    for m in [left, right, packet.back, op] {
        stage.remove_from_scene(m);
    }
    stage.add_to_scene(packet.group)?;
    stage.wait(2.0)?;
    play!(stage; write(stage.arena(), name).rt(1.2));
    kit.hold(stage)?;
    fade_all(stage, 0.6)?;
    Ok(())
}

/// 02_4–02_6: ex-or as light switches, five over seven; the prediction;
/// the flips that give two, and flipping twice back to five.
fn switches(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let (n, cell) = (3usize, 1.45);
    let x = 0.4;
    let mut top = Row::build(kit, stage, FIVE, n, cell, v(x, 1.75), Role::ex_or())?;
    let bottom = Row::build(kit, stage, SEVEN, n, cell, v(x, -0.45), Role::ex_or())?;
    let rows = {
        let mut all = top.tiles.clone();
        all.extend(&bottom.tiles);
        group_of(stage, &all)?
    };
    let places: Vec<Mob> = (0..n)
        .map(|k| {
            let at = top.at(k);
            kit.text(
                stage,
                &(1u32 << (n - 1 - k)).to_string(),
                30.0,
                Palette::dim(),
                v(at[0], at[1] + 0.95),
            )
        })
        .collect::<fmn::Result<_>>()?;
    let label_x = top.at(0)[0] - 1.35;
    let five = kit.text(
        stage,
        &FIVE.to_string(),
        56.0,
        Palette::ink(),
        v(label_x, top.centre[1]),
    )?;
    let seven = kit.text(
        stage,
        &SEVEN.to_string(),
        56.0,
        Palette::ink(),
        v(label_x, bottom.centre[1]),
    )?;
    let op = kit.tex(
        stage,
        OPLUS,
        64.0,
        Role::ex_or(),
        &[],
        v(label_x - 0.75, bottom.centre[1]),
    )?;
    let labels = {
        let mut all = places.clone();
        all.extend([five, seven, op]);
        group_of(stage, &all)?
    };
    // the switches: from each 1 of the second row up to the bit it flips
    let mut arrows = Vec::new();
    for k in 0..n {
        if bottom.bits[k] {
            let (from, to) = (bottom.at(k), top.at(k));
            let half = cell * 0.42;
            let a = Arrow::new(v(from[0], from[1] + half + 0.06), v(to[0], to[1] - half - 0.06))
                .thickness(3.0)
                .buff(0.0)
                .style(
                    Style::default()
                        .fill(Role::ex_or(), 1.0)
                        .stroke(Role::ex_or(), 3.0, 1.0),
                )
                .build()?;
            arrows.push(stage.arena_mut().add(a));
        }
    }
    let switch_marks = group_of(stage, &arrows)?;

    kit.say(stage, "02_4")?;
    play!(stage;
        fade_in(stage.arena_mut(), rows, ORIGIN, 1.0)?.rt(1.0).lag(0.1),
        fade_in(stage.arena_mut(), labels, ORIGIN, 1.0)?.rt(1.0),
    );
    stage.wait(1.8)?;
    play!(stage; fade_in(stage.arena_mut(), switch_marks, v(0.0, -0.15), 1.0)?.rt(1.2).lag(0.25));
    kit.hold(stage)?;

    // 02_5: what's five ex-or seven?
    kit.say(stage, "02_5")?;
    let mut lights: Vec<Box<dyn Animation>> = Vec::new();
    for &t in top.tiles.iter().chain(&bottom.tiles) {
        let a = indicate(stage.arena_mut(), t, 1.12, Some([0.35, 0.77, 0.87]))?;
        lights.push(stage.prepare(a)?);
    }
    let sweep = AnimationGroup::with_lag_ratio(stage.arena_mut(), lights, 0.3)?;
    stage.play(sweep.rt(2.4))?;
    let q = predict_pause(kit, stage, v(5.4, 0.85))?;

    // 02_6: the flips; two; flip twice and five is back
    kit.say(stage, "02_6")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    let result_x = 4.9;
    let first = kit.tex(
        stage,
        &format!(r"{FIVE} \oplus {SEVEN} = {}", FIVE ^ SEVEN),
        60.0,
        Palette::ink(),
        &[(r"\oplus", Role::ex_or())],
        v(result_x, 1.25),
    )?;
    let second = kit.tex(
        stage,
        &format!(r"{} \oplus {SEVEN} = {FIVE}", FIVE ^ SEVEN),
        60.0,
        Palette::ink(),
        &[(r"\oplus", Role::ex_or())],
        v(result_x, -0.35),
    )?;
    // the sound of it: five, two, five again, each set bit a note
    let listen = 3.0;
    let wav = sound::wav(
        "02_6_ex_or_toggles",
        &sound::chords(
            &[
                (FIVE as u8, 0.0, 0.9),
                ((FIVE ^ SEVEN) as u8, 1.0, 0.9),
                (FIVE as u8, 2.0, 0.9),
            ],
            listen,
        ),
    )?;
    let flip_all = |top: &mut Row, stage: &mut Stage<'_>, rt: f64| -> fmn::Result<()> {
        let mut flips = Vec::new();
        for k in 0..n {
            if bottom.bits[k] {
                let now = top.bits[k];
                flips.push(top.set(kit, stage, k, !now)?);
            }
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), flips, 0.35)?;
        stage.play(g.rt(rt))?;
        Ok(())
    };
    flip_all(&mut top, stage, 1.2)?;
    let two = kit.text(
        stage,
        &top.value().to_string(),
        56.0,
        Role::ex_or(),
        v(label_x, top.centre[1]),
    )?;
    play!(stage;
        replacement_transform(five, two).rt(0.6),
        write(stage.arena(), first).rt(1.0),
    );
    stage.wait(0.8)?;
    // flip twice: the same switches again, while the line says so
    flip_all(&mut top, stage, 1.2)?;
    let back = kit.text(
        stage,
        &top.value().to_string(),
        56.0,
        Palette::ink(),
        v(label_x, top.centre[1]),
    )?;
    play!(stage;
        replacement_transform(two, back).rt(0.6),
        write(stage.arena(), second).rt(1.0),
    );
    let note = precisely(
        kit,
        stage,
        "ex-or of several bits is one exactly when an odd number of them are one",
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
    let rule = Line::new(v(result_x - 1.7, 0.45), v(result_x + 1.7, 0.45))
        .style(line_style(Palette::faint(), 1.5))
        .build()?;
    let rule = stage.arena_mut().add(rule);
    play!(stage; show_creation(rule).rt(0.4));
    // the listen window: five, two, five again as chords, the switches
    // flipping in step with them
    kit.sonify(stage, listen, "02_6", &wav)?;
    // chords at 0, 1 and 2 s: each flip lands as its chord strikes
    let mut value = back;
    for (wait, color) in [(0.7, Role::ex_or()), (0.5, Palette::ink())] {
        stage.wait(wait)?;
        flip_all(&mut top, stage, 0.3)?;
        let next = kit.text(stage, &top.value().to_string(), 56.0, color, v(label_x, top.centre[1]))?;
        play!(stage; replacement_transform(value, next).rt(0.2));
        value = next;
    }
    Ok(())
}
