//! Chapter 11, the fair coin inside: a computer's noise source, read off as
//! ones and zeros with a few too many ones (11_1); the readings paired up,
//! the matching pairs thrown away, and the corrector Intel built on the
//! trick (11_2); then the picture of the video once more, the square at
//! four fifths with its mirror line, opening once into the loaded die's
//! thirty-six blocks, held for the resolved motif, and then smaller at the
//! left with the receipts beside it to the last frame (11_3). The noise is
//! the uniform
//! draws behind run 2 (`facts::trick_draws`), read against a threshold; the
//! pairs are filtered by `model::von_neumann`.

use super::draw::predict_pause;
use explainer_kit::{Kit, Palette, Timed, hex, lagged, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Block, COLUMN_MID, QuoteCard, SQUARE_AT, SQUARE_SIDE, Square, alice, bob, die_face, face_color, fade_in_all,
    fade_in_lagged, fade_out_all, faded_line, fail, flat, gold, group, line_style, pair_tile, panel, pause, rgb, say,
    tag, thrown, wait_until,
};
use crate::facts::{cited, facts, trick_draws};
use crate::model::{Face, LOADED, PairLaw, Q, bent, die, von_neumann};

pub struct TheFairCoinInside {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const PASSWORDS: &str = "passwords";
const CODES_CHIP: &str = "secret codes";
const READINGS: &str = "readings";
const LIKE_A_COIN: &str = "a one or a zero, like heads or tails";
const NEEDS: &str = "needs readings that don't affect each other (chapter 5)";
const EXAGGERATED: &str = "unfairness exaggerated";
const FAIR_TOSSES: &str = "fair tosses";
const CORRECTOR: &str = "corrector";
const GENERATOR: &str = "random number generator";
const INTEL_WORDS: &str = "a hardware corrector based on a concept proposed by John von Neumann";
const INTEL_SOURCE: &str = "Jun and Kocher, The Intel Random Number Generator,";
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const FIRST_TOSS: &str = "first toss";
const SECOND_TOSS: &str = "second toss";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const ROLLED_TWICE: &str = "a loaded die, rolled twice";
const COUNTS_CARRY: &str = "The counts carry the unfairness.";
const ORDER_FAIR: &str = "The order is perfectly fair.";
const RECEIPTS: &str = "Receipts";
const RECEIPT_COMPUTED: &str = "Every count and chance in this video was computed by code; quoted numbers are cited";
const RECEIPT_CODE: &str = "Code, data and every number:";
const RECEIPT_REPO: &str = "github.com/Dicklesworthstone/jeffs_math_explainer_videos";
const RECEIPT_SEEDS: &str = "Seeds";
const RECEIPT_TO: &str = "to";
const RECEIPT_GENERATOR: &str = "NumPy PCG64DXSM";
const RECEIPT_NAMES: &str = "Names to search:";
const VON_NEUMANN_TRICK: &str = "von Neumann's trick";
const SAMUELSON: &str = "Samuelson";
const HOEFFDING_SIMONS: &str = "Hoeffding and Simons";
const ELIAS: &str = "Elias";
const BLUM: &str = "Blum";
const DIJKSTRA: &str = "Dijkstra's fair roulette";
const PERES: &str = "Peres";
const JUELS: &str = "\"How to turn loaded dice into fair coins\"";

/// The dark ink of a pip, and of the cuts between blocks.
const DARK: &str = "#0E1116";

/// The share of ones the picture of raw noise shows: three in five, so the
/// eye can see it. An illustration's setting, labelled as exaggerated on
/// screen; the generator the narration names measured its raw readings
/// within about a hundredth of one half (Jun and Kocher).
fn shown_ones() -> Q {
    Q::new(3, 5)
}

/// How many readings the picture takes.
const READINGS_N: usize = 32;

// where things sit (11_1, 11_2)
const CHIP_AT: Vec3 = [-6.1, 2.3, 0.0];
const CHIP: f64 = 0.6;
/// The chip as the chapter opens: larger, in the middle.
const CHIP_OPEN_AT: Vec3 = [0.0, 0.9, 0.0];
const CHIP_OPEN: f64 = 2.4;
/// The trace: the first reading's place, the step between readings, the
/// band's foot and its height.
const TRACE_X: f64 = -4.95;
const TRACE_STEP: f64 = 0.365;
const TRACE_FOOT: f64 = 1.8;
const TRACE_H: f64 = 1.05;
const DIGITS_Y: f64 = 1.15;
const DIGIT_SIZE: f64 = 34.0;
const BAND_Y: f64 = 0.45;
/// The tag under the readings (11_1) and the one under the corrector (11_2).
const LIKE_Y: f64 = 0.5;
const NEEDS_Y: f64 = -0.05;
const OUT_Y: f64 = -0.95;
const OUT_STEP: f64 = 0.5;

/// The last picture (11_3, after the listen): the square, smaller, at the
/// left, and the receipts card beside it, both centred on this height.
const END_LEFT: f64 = -6.7;
const END_SIDE: f64 = 3.0;
const END_Y: f64 = 0.0;
/// The receipts card's right edge, its type size and line spacing.
const CARD_RIGHT: f64 = 6.85;
const CARD_TEXT: f64 = 22.0;
const CARD_LINE: f64 = 0.36;
/// How long the receipts hold beside the square, to the last frame.
const RECEIPTS_HOLD_S: f64 = 8.3;

fn args(rt: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    }
}

/// Fades for several detached mobjects, to play beside other moves.
fn fading_in(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(rt))?);
    }
    Ok(out)
}

/// Fades out for several mobjects (updaters cleared first), to play beside
/// other moves.
fn fading_out(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(rt))?);
    }
    Ok(out)
}

/// One picture for another in one move: `ins` start when `outs` are half
/// gone, so the two are never both at full strength.
fn swap(stage: &mut Stage<'_>, outs: Vec<Box<dyn Animation>>, ins: Vec<Box<dyn Animation>>) -> fmn::Result<()> {
    let outs = lagged(stage, outs, 0.0)?;
    let ins = lagged(stage, ins, 0.0)?;
    let both = lagged(stage, vec![Box::new(outs), Box::new(ins)], 0.5)?;
    stage.play(both)?;
    Ok(())
}

// ---------------------------------------------------------------- the noise

/// A chip: a body with its legs, `size` across the body, centred at `at`.
/// Detached, one mobject.
fn chip(stage: &mut Stage<'_>, at: Vec3, size: f64) -> fmn::Result<Mob> {
    let mut parts = Vec::new();
    let (leg_w, leg_l) = (0.1 * size, 0.22 * size);
    let out = size / 2.0 + leg_l / 2.0 - 0.02 * size;
    for k in [-1.0, 0.0, 1.0] {
        let along = 0.28 * size * k;
        for (dx, dy, w, h) in [
            (along, out, leg_w, leg_l),
            (along, -out, leg_w, leg_l),
            (out, along, leg_l, leg_w),
            (-out, along, leg_l, leg_w),
        ] {
            let leg = Rectangle::new()
                .width(w)
                .height(h)
                .style(flat(Palette::dim(), 1.0))
                .build()?;
            parts.push(leg.moved_to(v(at[0] + dx, at[1] + dy)));
        }
    }
    let body = Rectangle::new()
        .width(size)
        .height(size)
        .corner_radius(0.12 * size)
        .style(Style::default().fill(panel(), 1.0).stroke(Palette::dim(), 2.5, 1.0))
        .build()?;
    parts.push(body.moved_to(at));
    let core = Rectangle::new()
        .width(0.44 * size)
        .height(0.44 * size)
        .corner_radius(0.05 * size)
        .style(flat(Palette::faint(), 1.0))
        .build()?;
    parts.push(core.moved_to(at));
    Ok(stage.arena_mut().add(v_group(parts)))
}

/// Raw noise as a picture (chapter 01's third small picture, drawn large):
/// the uniform draws behind run 2, in the order they were made, as heights
/// across a band, a draw near 0 at the top. Every other draw is a reading:
/// it reads as a one when the draw falls below [`shown_ones`], which is
/// when its point stands above the threshold line.
struct Noise {
    /// The trace's corners, left to right.
    line: Vec<Vec3>,
    /// Each reading: its point on the trace, and whether it reads as a one.
    readings: Vec<(Vec3, bool)>,
    threshold_y: f64,
}

fn noise() -> fmn::Result<Noise> {
    let draws: Vec<f64> = trick_draws()
        .into_iter()
        .flatten()
        .flat_map(|(a, b)| [a, b])
        .take(2 * READINGS_N)
        .collect();
    if draws.len() < 2 * READINGS_N {
        return Err(fail(format!("run 2 has only {} draws", draws.len())));
    }
    let ones = shown_ones().f64();
    let threshold_y = TRACE_FOOT + TRACE_H * (1.0 - ones);
    let mut line = vec![v(TRACE_X - 0.6, threshold_y)];
    let mut readings = Vec::new();
    for (k, &u) in draws.iter().enumerate() {
        let p = v(TRACE_X + TRACE_STEP * k as f64 / 2.0, TRACE_FOOT + TRACE_H * (1.0 - u));
        line.push(p);
        if k % 2 == 0 {
            readings.push((p, u < ones));
        }
    }
    Ok(Noise {
        line,
        readings,
        threshold_y,
    })
}

/// A reading as the coin's face: a one is heads.
fn face(one: bool) -> Face {
    if one { Face::H } else { Face::T }
}

fn digit(f: Face) -> &'static str {
    match f {
        Face::H => "1",
        Face::T => "0",
    }
}

// ---------------------------------------------------- the die's square, once

/// The square cut six ways along each side by the loaded die's chances, the
/// biggest roll's stretch nearest the corner (chapter 08's grid): thirty-six
/// blocks, green above the diagonal, violet below, grey on it, drawn from
/// one tracker that slides every cut out from the coin's single cut (0: the
/// coin's four blocks, exactly as [`Square`] draws them; 1: the die).
struct DieSquare {
    open: Mob,
    blocks: Vec<Mob>,
    cuts: Vec<Mob>,
    frame: Mob,
    /// Carries the updater.
    anchor: Mob,
}

impl DieSquare {
    /// Detached. `coin` is where the cuts gather.
    fn build(stage: &mut Stage<'_>, corner: Vec3, side: f64, chances: &[Q], coin: f64) -> fmn::Result<Self> {
        let mut die_cuts = vec![0.0];
        let mut upto = Q::ZERO;
        for &chance in chances.iter().rev() {
            upto = upto + chance;
            die_cuts.push(upto.f64());
        }
        if upto != Q::ONE {
            return Err(fail("the die's chances do not add up to one".into()));
        }
        let n = chances.len();
        let at = move |u: f64, w: f64| v(corner[0] + side * u, corner[1] + side * w);
        let unit = [v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)];
        let mut blocks = Vec::new();
        for i in 0..n {
            for j in 0..n {
                // i < j: the first roll is the bigger, above the diagonal
                let color = match i.cmp(&j) {
                    std::cmp::Ordering::Less => alice(),
                    std::cmp::Ordering::Greater => bob(),
                    std::cmp::Ordering::Equal => thrown(),
                };
                blocks.push(
                    stage
                        .arena_mut()
                        .add(Polygon::new(unit).style(flat(color, 1.0)).build()),
                );
            }
        }
        let cut_style = line_style(hex(DARK), 3.0);
        let cuts: Vec<Mob> = (0..2 * (n - 1))
            .map(|_| {
                stage
                    .arena_mut()
                    .add(polyline(vec![v(0.0, 0.0), v(1.0, 0.0)]).style(cut_style).build())
            })
            .collect();
        let frame = stage.arena_mut().add(
            Polygon::new([at(0.0, 0.0), at(1.0, 0.0), at(1.0, 1.0), at(0.0, 1.0)])
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let open = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (block_mobs, cut_mobs) = (blocks.clone(), cuts.clone());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(open).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(t) {
                    return;
                }
                let c: Vec<f64> = die_cuts
                    .iter()
                    .enumerate()
                    .map(|(k, &d)| if k == 0 || k == n { d } else { coin + (d - coin) * t })
                    .collect();
                // a block with no room is not drawn (and never a hairline)
                let thin = 0.0015;
                for i in 0..n {
                    for j in 0..n {
                        let m = block_mobs[i * n + j];
                        let (u1, w1) = (c[i + 1].max(c[i] + thin), c[j + 1].max(c[j] + thin));
                        let q = [at(c[i], c[j]), at(u1, c[j]), at(u1, w1), at(c[i], w1)];
                        let _ = s.set_points(m, Polygon::new(q).build().points());
                        let room = c[i + 1] - c[i] > 1e-6 && c[j + 1] - c[j] > 1e-6;
                        s.set_fill(m, None, Some(if room { 1.0 } else { 0.0 }), None, false);
                    }
                }
                for k in 1..n {
                    let lines = [[at(c[k], 0.0), at(c[k], 1.0)], [at(0.0, c[k]), at(1.0, c[k])]];
                    for (line, &m) in lines.iter().zip(&cut_mobs[2 * (k - 1)..]) {
                        let _ = s.set_points(m, polyline(line.to_vec()).build().points());
                    }
                }
                last = Some(t);
            },
            true,
        )?;
        Ok(Self {
            open,
            blocks,
            cuts,
            frame,
            anchor,
        })
    }

    /// Everything to bring into the scene, back to front.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(&self.blocks);
        out.extend(&self.cuts);
        out.push(self.frame);
        out
    }
}

/// The receipts as a card that fits beside the square (the kit's
/// `receipts_card` spans the frame): "Receipts" in the hero role and the
/// lines left-aligned, its right edge at [`CARD_RIGHT`], centred on
/// [`END_Y`]. A card that would reach `room_left` is an error, never a
/// clipped line. Detached parts, the back first.
fn receipts_beside(kit: &Kit, stage: &mut Stage<'_>, lines: &[String], room_left: f64) -> fmn::Result<Vec<Mob>> {
    let mut texts = Vec::new();
    let mut widest: f64 = 0.0;
    for l in lines {
        let t = kit.text(stage, l, CARD_TEXT, Palette::ink(), ORIGIN)?;
        widest = widest.max(stage.get_width(t));
        texts.push(t);
    }
    let (w, h) = (widest + 0.8, 1.05 + CARD_LINE * lines.len() as f64);
    let (left, top) = (CARD_RIGHT - w, END_Y + h / 2.0);
    if left < room_left {
        return Err(fail(format!(
            "the receipts card is {w:.2} wide and would reach x = {left:.2}, past {room_left:.2}"
        )));
    }
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(v(left + w / 2.0, END_Y));
    let mut parts = vec![stage.arena_mut().add(back)];
    parts.push(kit.text(stage, RECEIPTS, 34.0, gold(), v(left + w / 2.0, top - 0.42))?);
    for (i, &t) in texts.iter().enumerate() {
        stage.move_to(t, v(left + 0.4, top - 0.95 - CARD_LINE * i as f64), LEFT);
        parts.push(t);
    }
    Ok(parts)
}

/// Two die faces side by side on a small card, as `draw::pair_tile` sets
/// two results. Detached, one group.
fn dice_tile(stage: &mut Stage<'_>, rolls: (usize, usize), at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(1.38)
        .height(0.82)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    for (value, dx) in [(rolls.0, -0.32), (rolls.1, 0.32)] {
        parts.push(die_face(
            stage,
            value,
            v(at[0] + dx, at[1]),
            0.5,
            Palette::ink(),
            hex(DARK),
        )?);
    }
    group(stage, &parts)
}

impl SceneConstruct for TheFairCoinInside {
    fn name(&self) -> &str {
        "11_the_fair_coin_inside"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = bent().f64();

        // 11_1: computers need fair tosses too; many get them from noise,
        // and raw noise comes out a little unfair
        let line = say(&kit, stage, "11_1")?;
        let source = chip(stage, CHIP_AT, CHIP)?;
        stage.scale(source, CHIP_OPEN / CHIP);
        stage.move_to(source, CHIP_OPEN_AT, ORIGIN);
        play!(stage; fade_in(stage.arena_mut(), source, ORIGIN, 1.0)?.rt(0.5));
        let mut uses = Vec::new();
        for (words, phrase, x) in [(PASSWORDS, "for passwords", -2.0), (CODES_CHIP, "secret codes", 2.0)] {
            line.wait_for(stage, phrase)?;
            let t = tag(&kit, stage, words, v(x, CHIP_OPEN_AT[1] - 2.3), 28.0)?;
            play!(stage; fade_in(stage.arena_mut(), t, v(0.0, 0.2), 1.0)?.rt(0.4));
            uses.push(t);
        }
        // the chip steps aside and its noise runs out across the frame
        line.wait_for(stage, "Many get them from electrical noise")?;
        let aside = source
            .animate()
            .set_anim_args(args(0.7))?
            .scale(CHIP / CHIP_OPEN)?
            .move_to(CHIP_AT, ORIGIN)?;
        let mut step = fading_out(stage, &uses, 0.4)?;
        step.push(stage.prepare(aside)?);
        stage.play_prepared(step)?;
        let noise = noise()?;
        let trace = polyline(noise.line.clone())
            .style(line_style(Palette::ink(), 2.5))
            .build();
        let trace = stage.arena_mut().add(trace);
        let right = TRACE_X + TRACE_STEP * READINGS_N as f64;
        let threshold = DashedLine::new(v(TRACE_X - 0.6, noise.threshold_y), v(right, noise.threshold_y))
            .dash_length(0.12)
            .style(faded_line(Palette::dim(), 2.0, 0.8))
            .build()?;
        let threshold = stage.arena_mut().add(threshold);
        let mut draw = show_creation(trace);
        draw.update_rate_info(Some(1.5), Some(RateFunc::linear()), None);
        play!(stage; draw, fade_in(stage.arena_mut(), threshold, ORIGIN, 1.0)?.rt(1.5));
        // the readings: a one above the line, a zero below, each under its
        // own point of the trace
        line.wait_for(stage, "raw noise comes out")?;
        let mut dots = Vec::new();
        let mut digits = Vec::new();
        let mut landing = Vec::new();
        for &(at, one) in &noise.readings {
            let color = face_color(face(one));
            let dot = Dot::new().point(at).radius(0.055).style(flat(color, 1.0)).build();
            let dot = stage.arena_mut().add(dot);
            let d = kit.text(stage, digit(face(one)), DIGIT_SIZE, color, v(at[0], DIGITS_Y))?;
            dots.push(dot);
            digits.push(d);
            landing.extend([dot, d]);
        }
        let readings = kit.text(stage, READINGS, 22.0, Palette::dim(), v(CHIP_AT[0], DIGITS_Y))?;
        fade_in_all(stage, &[readings], 0.3)?;
        fade_in_lagged(stage, &landing, 2.0, 0.25)?;
        let exaggerated = kit.text(stage, EXAGGERATED, 22.0, Palette::dim(), ORIGIN)?;
        stage.move_to(exaggerated, v(right, 3.38), RIGHT);
        // what a reading is, in the video's words
        let pairs_mid = TRACE_X + TRACE_STEP * (READINGS_N - 1) as f64 / 2.0;
        let like = tag(&kit, stage, LIKE_A_COIN, v(pairs_mid, LIKE_Y), 22.0)?;
        fade_in_all(stage, &[exaggerated, like], 0.4)?;
        let q = predict_pause(&kit, stage, v(0.6, OUT_Y))?;

        // 11_2: pair the readings up, throw away the pairs that match; in
        // 1999 Intel built a corrector on the trick
        let line = say(&kit, stage, "11_2")?;
        let faces: Vec<Face> = noise.readings.iter().map(|&(_, one)| face(one)).collect();
        let kept = von_neumann(&faces);
        let mut outlines = Vec::new();
        for pair in noise.readings.as_chunks::<2>().0 {
            let x = (pair[0].0[0] + pair[1].0[0]) / 2.0;
            let o = Rectangle::new()
                .width(TRACE_STEP + 0.26)
                .height(0.52)
                .corner_radius(0.1)
                .style(faded_line(Palette::dim(), 1.6, 0.9))
                .build()?
                .moved_to(v(x, DIGITS_Y));
            outlines.push(stage.arena_mut().add(o));
        }
        fade_out_all(stage, &[q], 0.3)?;
        fade_in_lagged(stage, &outlines, 0.8, 0.2)?;
        // a matching pair goes grey; a pair that differs keeps its colours
        line.wait_for(stage, "and throw away")?;
        let mut greying: Vec<Box<dyn Animation>> = Vec::new();
        let mut deciding = Vec::new();
        for (k, pair) in faces.as_chunks::<2>().0.iter().enumerate() {
            if pair[0] != pair[1] {
                deciding.push(k);
                continue;
            }
            for &d in &digits[2 * k..2 * k + 2] {
                let grey = d.animate().set_anim_args(args(0.5))?.set_color(thrown())?;
                greying.push(stage.prepare(grey)?);
            }
            let a = fade_out(stage.arena_mut(), outlines[k], ORIGIN, 1.0)?;
            greying.push(stage.prepare(a.rt(0.5))?);
        }
        let greying = lagged(stage, greying, 0.12)?.with_run_time(1.0);
        // the tag under the readings makes way for the fair tosses' fall
        let mut making_way = fading_out(stage, &[like], 0.4)?;
        making_way.push(Box::new(greying));
        stage.play_prepared(making_way)?;
        if deciding.len() != kept.len() {
            return Err(fail(format!(
                "{} pairs differ but the rule kept {}",
                deciding.len(),
                kept.len()
            )));
        }
        // the first reading of each pair that differs comes out below
        let out_left = 0.6 - OUT_STEP * (kept.len().max(1) - 1) as f64 / 2.0;
        let mut outs = Vec::new();
        let mut dropping: Vec<Box<dyn Animation>> = Vec::new();
        for (i, (&k, &f)) in deciding.iter().zip(&kept).enumerate() {
            let from = v(noise.readings[2 * k].0[0], DIGITS_Y);
            let d = kit.text(stage, digit(f), DIGIT_SIZE, face_color(f), from)?;
            stage.add_to_scene(d)?;
            let to = v(out_left + OUT_STEP * i as f64, OUT_Y);
            let fall = d.animate().set_anim_args(args(0.7))?.move_to(to, ORIGIN)?;
            dropping.push(stage.prepare(fall)?);
            outs.push(d);
        }
        let fair = kit.text(stage, FAIR_TOSSES, 26.0, Palette::ink(), ORIGIN)?;
        stage.move_to(fair, v(out_left - 0.5, OUT_Y), RIGHT);
        let dropping = lagged(stage, dropping, 0.25)?.with_run_time(1.2);
        stage.play(dropping)?;
        fade_in_all(stage, &[fair], 0.4)?;
        // the card: what the source says, and its date
        line.wait_for(stage, "In nineteen ninety-nine")?;
        let card = QuoteCard::build(
            &kit,
            stage,
            INTEL_WORDS,
            &format!("{INTEL_SOURCE} {}", cited::INTEL_YEAR),
            v(0.0, -2.45),
            72,
            2,
        )?;
        let mut ins = vec![card.back];
        ins.extend(&card.lines);
        ins.extend(&card.source);
        fade_in_all(stage, &ins, 0.6)?;
        // a corrector, built into the generator
        line.wait_for(stage, "built a corrector")?;
        // as wide as the row of pairs it stands under, with what it needs
        // under it
        let band = Rectangle::new()
            .width(TRACE_STEP * READINGS_N as f64 - 0.1)
            .height(0.42)
            .corner_radius(0.1)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::dim(), 1.6, 1.0))
            .build()?
            .moved_to(v(pairs_mid, BAND_Y));
        let band = stage.arena_mut().add(band);
        let name = kit.text(stage, CORRECTOR, 22.0, Palette::ink(), v(pairs_mid, BAND_Y))?;
        let needs = tag(&kit, stage, NEEDS, v(pairs_mid, NEEDS_Y), 20.0)?;
        fade_in_all(stage, &[band, name, needs], 0.5)?;
        line.wait_for(stage, "into its random number generator")?;
        // the generator holds the corrector and its condition; the fair
        // tosses come out under it
        let (top, bottom) = (3.72, NEEDS_Y - 0.36);
        let walls = Rectangle::new()
            .width(13.7)
            .height(top - bottom)
            .corner_radius(0.2)
            .style(line_style(Palette::dim(), 2.0))
            .build()?
            .moved_to(v(0.0, (top + bottom) / 2.0));
        let walls = stage.arena_mut().add(walls);
        let generator = kit.text_left(stage, GENERATOR, 22.0, Palette::ink(), v(-6.6, 3.38))?;
        fade_in_all(stage, &[walls, generator], 0.6)?;

        // 11_3: the fair coin inside the unfair one; set aside how many
        // heads, look only at the order
        let line = say(&kit, stage, "11_3")?;
        let square = Square::build(stage, SQUARE_AT, SQUARE_SIDE, p)?;
        square.paint(stage, Block::HT, alice(), 1.0);
        square.paint(stage, Block::TH, bob(), 1.0);
        square.paint(stage, Block::HH, thrown(), 1.0);
        square.paint(stage, Block::TT, thrown(), 1.0);
        let edges = square.edges(&kit, stage, HEADS, TAILS)?;
        let mid_x = SQUARE_AT[0] + SQUARE_SIDE / 2.0;
        let first_toss = kit.text(stage, FIRST_TOSS, 22.0, Palette::dim(), v(mid_x, SQUARE_AT[1] - 0.78))?;
        let second_toss = kit.text_left(
            stage,
            SECOND_TOSS,
            22.0,
            Palette::dim(),
            v(SQUARE_AT[0] - 0.95, SQUARE_AT[1] + SQUARE_SIDE + 0.32),
        )?;
        let diagonal = square.diagonal(stage)?;
        let mut picture = square.mobs();
        picture.extend(edges.mobs());
        picture.extend([first_toss, second_toss, diagonal]);
        let leaving: Vec<Mob> = stage.scene().mobjects().to_vec();
        let outs_now = fading_out(stage, &leaving, 0.5)?;
        let ins = fading_in(stage, &picture, 0.8)?;
        swap(stage, outs_now, ins)?;
        // what stays on top of the blocks when one of them is animated
        let mut over = square.cuts.to_vec();
        over.extend([square.frame, diagonal]);
        // the two pairs that decide, equally likely: the fair coin inside
        line.wait_for(stage, "still has a fair one inside it")?;
        let law = PairLaw::same_coin(bent());
        let (left_x, right_x, pairs_y) = (COLUMN_MID - 1.25, COLUMN_MID + 1.25, 2.1);
        let mut column = Vec::new();
        for (pair, x, name, color) in [
            ((Face::H, Face::T), left_x, ALICE, alice()),
            ((Face::T, Face::H), right_x, BOB, bob()),
        ] {
            let parts = pair_tile(&kit, stage, pair, v(x, pairs_y), 1.0)?;
            column.push(group(stage, &parts)?);
            column.push(kit.text(stage, name, 26.0, color, v(x, pairs_y - 0.68))?);
            let share = format!("{}%", law.prob(pair.0, pair.1).percent(0));
            column.push(kit.text(stage, &share, 26.0, Palette::ink(), v(x, pairs_y - 1.08))?);
        }
        column.push(kit.text(stage, "=", 44.0, gold(), v(COLUMN_MID, pairs_y))?);
        fade_in_all(stage, &column, 0.6)?;
        let pulse = |stage: &mut Stage<'_>| -> fmn::Result<()> {
            play!(stage;
                fmn::animation::indicate(stage.arena_mut(), square.block(Block::HT), 1.07, Some(rgb(alice())))?.rt(0.8),
                fmn::animation::indicate(stage.arena_mut(), square.block(Block::TH), 1.07, Some(rgb(bob())))?.rt(0.8),
            );
            stage.scene_mut().bring_to_front(&over)?;
            Ok(())
        };
        pulse(stage)?;
        line.wait_for(stage, "as long as it can land both ways")?;
        pulse(stage)?;
        // once: the same square for the loaded die, and back
        line.wait_for(stage, "You find it")?;
        let chances = die(&LOADED);
        let grid = DieSquare::build(stage, SQUARE_AT, SQUARE_SIDE, &chances, p)?;
        for m in grid.mobs() {
            stage.add_to_scene(m)?;
        }
        stage.scene_mut().bring_to_front(&[diagonal])?;
        // (the edges' anchor stays: a fade would clear the updater that
        // keeps the stretches on the square when it moves at the end)
        let mut coin_words = edges.stretches.to_vec();
        coin_words.extend(edges.words);
        coin_words.extend([first_toss, second_toss]);
        let rolled = kit.text(stage, ROLLED_TWICE, 22.0, Palette::dim(), v(mid_x, SQUARE_AT[1] - 0.45))?;
        let (six, five) = (chances.len(), chances.len() - 1);
        let dice_y = pairs_y - 2.25;
        let mut dice = vec![
            dice_tile(stage, (six, five), v(left_x, dice_y))?,
            dice_tile(stage, (five, six), v(right_x, dice_y))?,
            kit.text(stage, "=", 44.0, gold(), v(COLUMN_MID, dice_y))?,
        ];
        let each = format!("{}%", (chances[six - 1] * chances[five - 1]).percent(0));
        for x in [left_x, right_x] {
            dice.push(kit.text(stage, &each, 26.0, Palette::ink(), v(x, dice_y - 0.68))?);
        }
        // (the coin's words leave before the die's caption comes, and the
        // other way round: two captions never share the place)
        let mut opening = fading_out(stage, &coin_words, 0.4)?;
        opening.push(stage.prepare(tween(grid.open, 1.0, 1.0)?)?);
        opening.extend(fading_in(stage, &dice, 1.0)?);
        stage.play_prepared(opening)?;
        fade_in_all(stage, &[rolled], 0.3)?;
        line.wait_for(stage, "setting aside how many heads")?;
        let words_y = dice_y - 1.5;
        let counts = kit.text(stage, COUNTS_CARRY, 28.0, Palette::ink(), v(COLUMN_MID, words_y))?;
        fade_in_all(stage, &[counts], 0.5)?;
        // closed again in time for the order
        wait_until(stage, line.at("and looking only at the order")? - 1.5)?;
        fade_out_all(stage, &[rolled], 0.3)?;
        let mut closing = fading_in(stage, &coin_words, 1.0)?;
        closing.push(stage.prepare(tween(grid.open, 0.0, 1.0)?)?);
        stage.play_prepared(closing)?;
        stage.scene_mut().remove(&grid.mobs());
        // looking only at the order
        line.wait_for(stage, "and looking only at the order")?;
        let order = kit.text(stage, ORDER_FAIR, 28.0, gold(), v(COLUMN_MID, words_y - 0.55))?;
        fade_in_all(stage, &[order], 0.5)?;
        pulse(stage)?;
        // the motif resolves over the held square, with nothing arriving
        kit.listen(stage, 4.0, "11_the_fair_coin_inside:11_3")?;

        // the receipts
        let all = facts().map_err(fail)?;
        let seeds = all
            .get("E-S0.seeds")
            .and_then(|s| s.split_once('-'))
            .ok_or_else(|| fail("the facts have no seeds".into()))?;
        let lines = [
            RECEIPT_COMPUTED.to_owned(),
            format!("{RECEIPT_CODE} {RECEIPT_REPO}"),
            format!(
                "{RECEIPT_SEEDS} {} {RECEIPT_TO} {}, {RECEIPT_GENERATOR}",
                seeds.0, seeds.1
            ),
            RECEIPT_NAMES.to_owned(),
            format!(
                "{VON_NEUMANN_TRICK}; {SAMUELSON}, {}; {HOEFFDING_SIMONS}, {}; {ELIAS}, {}; {BLUM}, {};",
                cited::SAMUELSON_YEAR,
                cited::HOEFFDING_SIMONS_YEAR,
                cited::ELIAS_YEAR,
                cited::BLUM_FOCS_YEAR
            ),
            format!(
                "{DIJKSTRA}, {}; {PERES}, {}; {JUELS}, {}",
                cited::DIJKSTRA_YEAR,
                cited::PERES_YEAR,
                cited::JUELS_YEAR
            ),
        ];
        let receipts = receipts_beside(&kit, stage, &lines, END_LEFT + END_SIDE + 0.3)?;
        // the square slides to the left, smaller, with its gold line; its
        // words and the tiles beside it go, and the receipts come in beside
        // it. The square stays to the last frame.
        let mut keep = square.mobs();
        keep.push(edges.anchor);
        keep.extend(edges.stretches);
        keep.push(diagonal);
        let leaving: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| !keep.contains(m))
            .collect();
        let pose = square.pose(stage);
        let end_bottom = END_Y - END_SIDE / 2.0;
        let shrink = END_SIDE / pose.side;
        let follow = diagonal
            .animate()
            .set_anim_args(args(1.0))?
            .scale_about(shrink, Some(v(pose.left, pose.bottom)), None)?
            .shift(v(END_LEFT - pose.left, end_bottom - pose.bottom))?;
        let mut slide = fading_out(stage, &leaving, 0.4)?;
        for (t, value) in [
            (square.view.left, END_LEFT),
            (square.view.bottom, end_bottom),
            (square.view.side, END_SIDE),
        ] {
            slide.push(stage.prepare(tween(t, value, 1.0)?)?);
        }
        slide.push(stage.prepare(follow)?);
        let ins = fading_in(stage, &receipts, 0.7)?;
        swap(stage, slide, ins)?;
        pause(stage, RECEIPTS_HOLD_S)?;
        kit.finish(stage)
    }
}
