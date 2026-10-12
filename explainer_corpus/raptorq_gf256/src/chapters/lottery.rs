//! Chapter 05, the ex-or lottery: the three steps of the cube as fractions
//! to fill (05_1), seven eighths times six eighths times four eighths, the
//! failure, and the many-piece limit of 71% (05_2); the danger meter on a
//! ten-slot bucket filled by a seeded run of random cards, flat near zero
//! and rising only at the end (05_3–05_4), and the rank chord in the listen
//! window: a tone for each useful card, a thud for each useless one.
//! Fractions and failures come from `model.rs` (oracle B-F4, B-F5); the
//! run is a picture, never a number on screen.

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, hex, play, v};
use explainer_score::Stereo;
use explainer_score::instr::{Instrument, Note, render};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{Role, crossfade, flat, group_of, line_style, packet};
use super::props::{BucketProp, DangerMeter};
use super::sound;
use crate::model::{Bucket, SplitMix64, p_fail, p_fail_limit, p_full, useless_chance};

/// The cube's pieces (05_1), and the bucket's (05_3).
const CUBE: u32 = 3;
const SLOTS: u32 = 10;
/// The run's seed: the hook's seed, this chapter's stream; never re-rolled.
const RUN_SEED: u64 = crate::model::FIELD_SEED ^ 0x0503;

/// One card of the seeded run: whether it was useful, and the bucket's
/// level when it arrived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    pub useful: bool,
    pub held: u32,
}

/// Uniform random ex-or cards over `pieces` pieces until they pin all of
/// them down: the run the bucket shows and the rank chord plays.
pub fn run(pieces: u32, seed: u64) -> Vec<Draw> {
    let mut rng = SplitMix64::new(seed);
    let mut bucket = Bucket::new();
    let mut out = Vec::new();
    while (bucket.len() as u32) < pieces {
        let held = bucket.len() as u32;
        let card = (rng.next_u64() % (1u64 << pieces)) as u32;
        out.push(Draw {
            useful: bucket.add(card),
            held,
        });
    }
    out
}

/// MIDI pitches of the rank chord's tones, lowest first: the video's E major
/// bit pitches, then two more up the scale for slots nine and ten.
const RANK_PITCH: [f64; 10] = [76.0, 78.0, 80.0, 83.0, 85.0, 88.0, 90.0, 92.0, 95.0, 97.0];

/// The rank chord: each useful card adds the next tone and lets it ring;
/// each useless one is a muted low thud. `step` seconds between cards.
pub fn rank_chord(draws: &[Draw], step: f64, seconds: f64) -> Stereo {
    let mut out = Stereo::silence(((seconds + 4.0) * explainer_score::FS) as usize);
    for (k, d) in draws.iter().enumerate() {
        let at = ((0.15 + step * k as f64) * explainer_score::FS) as usize;
        let note = if d.useful {
            let mut n = Note::new(RANK_PITCH[d.held as usize % 10], 0.55, seconds, k as u64 + 1);
            n.pan = (d.held as f64 - 4.5) / 4.5 * 0.4;
            render(Instrument::WarmGlass, &n)
        } else {
            render(Instrument::FeltPiano, &Note::new(40.0, 0.7, 0.12, k as u64 + 101))
        };
        out.mix_in(&note, at, 1.0);
    }
    sound::finish(out, seconds)
}

pub struct Lottery {
    pub kit: Kit,
}

impl SceneConstruct for Lottery {
    fn name(&self) -> &str {
        "05_the_ex_or_lottery"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        three(&kit, stage)?;
        meter(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// The label of the 3-piece card `c` (bit i = piece i): "∅", "A", "AB", …
fn label(c: u32) -> String {
    if c == 0 {
        return "∅".to_owned();
    }
    (0..CUBE as usize)
        .filter(|&i| c >> i & 1 == 1)
        .map(|i| crate::model::PIECE_NAMES[i])
        .collect()
}

/// 05_1–05_2: three steps as fractions; the product; the failure; the
/// many-piece limit.
fn three(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let total = 1u32 << CUBE;
    let cell = 1.0;
    let x0 = -6.5;
    let rows_y = [2.35, 0.85, -0.65];
    // after r useful cards (A, then B) the bucket can make 2^r of the cards:
    // those cells are lit, the others would help
    let mut strips = Vec::new();
    let mut fracs = Vec::new();
    let mut nums = Vec::new();
    for (r, &y) in rows_y.iter().enumerate() {
        let lit = 1u32 << r;
        let mut cells = Vec::new();
        for c in 0..total {
            let made = c < lit; // ∅, A, B, AB: the span of A and B in this order
            let at = v(x0 + cell * (c as f64 + 0.5), y);
            let sq = Rectangle::new()
                .width(cell * 0.9)
                .height(cell * 0.9)
                .corner_radius(0.06)
                .style(if made {
                    flat(Role::lit(), 0.9)
                } else {
                    Style::default()
                        .fill(Role::ex_or(), 0.12)
                        .stroke(Role::ex_or(), 2.0, 0.8)
                })
                .build()?
                .moved_to(at);
            cells.push(stage.arena_mut().add(sq));
            let ink = if made {
                hex(explainer_kit::BACKGROUND)
            } else {
                Palette::ink()
            };
            cells.push(kit.text(stage, &label(c), 26.0, ink, at)?);
        }
        let tag = kit.text_left(
            stage,
            &format!("after {r} useful"),
            22.0,
            Palette::dim(),
            v(x0, y + cell * 0.68),
        )?;
        cells.push(tag);
        strips.push(group_of(stage, &cells)?);
        let fx = x0 + cell * total as f64 + 0.9;
        let q = kit.tex(
            stage,
            &format!(r"\frac{{?}}{{{total}}}"),
            52.0,
            Palette::ink(),
            &[],
            v(fx, y),
        )?;
        fracs.push(q);
        let n = total - lit;
        nums.push(kit.tex(
            stage,
            &format!(r"\frac{{{n}}}{{{total}}}"),
            52.0,
            Role::ex_or(),
            &[],
            v(fx, y),
        )?);
    }
    kit.say(stage, "05_1")?;
    for (s, q) in strips.iter().zip(&fracs) {
        play!(stage;
            fade_in(stage.arena_mut(), *s, v(0.0, -0.15), 1.0)?.rt(0.7).lag(0.04),
            fade_in(stage.arena_mut(), *q, ORIGIN, 1.0)?.rt(0.7),
        );
    }
    let q = predict_pause(kit, stage, v(4.4, 0.85))?;

    // 05_2: seven eighths, six eighths, four eighths; one time in three
    kit.say(stage, "05_2")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    for (q, n) in fracs.iter().zip(&nums) {
        play!(stage; replacement_transform(*q, *n).rt(0.7));
        stage.wait(0.25)?;
    }
    let success = p_full(2, CUBE, 0);
    let fail = p_fail(2, CUBE, 0);
    let factors: Vec<String> = (0..CUBE)
        .map(|r| format!(r"\frac{{{}}}{{{total}}}", total - (1 << r)))
        .collect();
    let product = kit.tex(
        stage,
        &format!(r"{} = {:.3}", factors.join(r" \cdot "), success),
        48.0,
        Palette::ink(),
        &[],
        v(-2.5, -2.1),
    )?;
    let fails = kit.tex(
        stage,
        &format!(r"\text{{fails }} 1 - {:.3} = {:.3}", success, fail),
        48.0,
        Palette::ink(),
        &[],
        v(-2.5, -3.15),
    )?;
    play!(stage; write(stage.arena(), product).rt(1.2));
    play!(stage; write(stage.arena(), fails).rt(1.0));
    stage.wait(0.6)?;
    // the strips give way to the many-piece picture; the sums stay, smaller
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in strips.iter().chain(&nums) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    for (m, to) in [(product, v(-4.0, 3.2)), (fails, v(-4.0, 2.35))] {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .scale(0.75)?
            .move_to(to, ORIGIN)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared(out)?;
    // with many pieces: the failure climbs to its limit
    let frame = Frame2 {
        center: v(0.6, -0.25),
        width: 9.0,
        height: 3.2,
        x: (0.0, 13.0),
        y: (0.4, 0.8),
    };
    let limit = p_fail_limit(2, 0);
    let mut parts = vec![
        stage.arena_mut().add(
            Line::new(frame.p(0.0, 0.4), frame.p(13.0, 0.4))
                .style(line_style(Palette::dim(), 2.5))
                .build()?,
        ),
        stage.arena_mut().add(
            DashedLine::new(frame.p(0.0, limit), frame.p(13.0, limit))
                .dash_length(0.12)
                .style(line_style(Role::ex_or(), 2.5))
                .build()?,
        ),
    ];
    parts.push(kit.text(
        stage,
        "pieces",
        24.0,
        Palette::dim(),
        v(frame.center[0], frame.bottom() - 0.3),
    )?);
    parts.push(kit.text_left(
        stage,
        &format!("{:.0}% (model)", limit * 100.0),
        28.0,
        Role::ex_or(),
        v(frame.right() - 2.0, frame.p(0.0, limit)[1] + 0.3),
    )?);
    let axes = group_of(stage, &parts)?;
    let dots: Vec<VMobject> = (1..=12)
        .map(|k| {
            Dot::new()
                .point(frame.p(f64::from(k), p_fail(2, k, 0)))
                .radius(0.07)
                .color(Role::ex_or())
                .build()
        })
        .collect();
    let dots = stage.arena_mut().add(v_group(dots));
    let lottery = kit.text_styled(
        stage,
        "the ex-or lottery",
        44.0,
        Role::ex_or(),
        v(3.6, 2.9),
        true,
        false,
    )?;
    play!(stage; fade_in(stage.arena_mut(), axes, ORIGIN, 1.0)?.rt(0.6));
    play!(stage; fade_in(stage.arena_mut(), dots, v(0.0, -0.2), 1.0)?.rt(1.6).lag(0.15));
    let note = precisely(
        kit,
        stage,
        &format!(
            "with K pieces the cards all help with chance (1 − 1/2)(1 − 1/4)(1 − 1/8)…, K factors; it fails {:.6} \
             at K = 10 and {:.6} in the limit (model: each card uses each piece by a fair coin)",
            p_fail(2, 10, 0),
            limit
        ),
    )?;
    play!(stage;
        write(stage.arena(), lottery).rt(1.0),
        fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
    );
    kit.hold(stage)?;
    Ok(())
}

/// 05_3–05_4: the danger meter while a seeded run fills ten slots; the
/// chart of the chance per slot; the rank chord.
fn meter(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let b = BucketProp::build(stage, SLOTS as usize, v(-3.6, -1.0), 2.4, 4.6)?;
    let m = DangerMeter::on(kit, stage, &b, 0.0)?;
    // the chart: the chance a new card is useless, at each level of the fill
    let chart = Frame2 {
        center: v(3.4, -0.9),
        width: 5.4,
        height: 3.6,
        x: (0.0, f64::from(SLOTS)),
        y: (0.0, 0.5),
    };
    let base = stage.arena_mut().add(
        Line::new(v(chart.left(), chart.bottom()), v(chart.right(), chart.bottom()))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    );
    let x_label = kit.text(
        stage,
        "slots already filled",
        48.0,
        Palette::ink(),
        v(chart.center[0], chart.bottom() - 0.5),
    )?;
    let axis = group_of(stage, &[base, x_label])?;
    let bar_for = |stage: &mut Stage<'_>, held: u32| -> fmn::Result<Mob> {
        let p = useless_chance(held, SLOTS);
        let (bl, tr) = (chart.p(f64::from(held) + 0.12, 0.0), chart.p(f64::from(held) + 0.88, p));
        let r = Rectangle::new()
            .width(tr[0] - bl[0])
            .height((tr[1] - bl[1]).max(0.012))
            .style(flat(Role::lit(), 0.85))
            .build()?
            .moved_to(v((bl[0] + tr[0]) / 2.0, bl[1] + (tr[1] - bl[1]).max(0.012) / 2.0));
        Ok(stage.arena_mut().add(r))
    };
    let draws = run(SLOTS, RUN_SEED);
    kit.say(stage, "05_3")?;
    crossfade(stage, &[b.group, m.group, axis], 0.8)?;
    stage.add_to_scene(m.fill)?;
    // one card at a time: a useful card fills a slot, a useless one bounces
    let drop_from = v(b.centre[0] + 2.2, b.rim() + 1.6);
    let mut bars: Vec<Mob> = Vec::new();
    let mut waters: Vec<Mob> = Vec::new();
    let mut play_draw = |stage: &mut Stage<'_>, d: &Draw, fast: f64| -> fmn::Result<()> {
        if d.held as usize >= bars.len() {
            let bar = bar_for(stage, d.held)?;
            play!(stage;
                fmn::animation::grow_from_edge(stage.arena_mut(), bar, DOWN, None)?.rt(0.3 * fast),
                m.to(useless_chance(d.held, SLOTS), 0.3 * fast)?,
            );
            bars.push(bar);
        }
        let glyph = stage.arena_mut().add(packet(0.5, Role::card()).moved_to(drop_from));
        stage.add_to_scene(glyph)?;
        if d.useful {
            let water = b.water(stage, d.held as usize, Role::ex_or())?;
            waters.push(water);
            let fall = glyph
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4 * fast),
                    ..AnimateArgs::default()
                })?
                .move_to(b.slot_centre(d.held as usize), ORIGIN)?
                .set_opacity(0.0)?;
            play!(stage; fall);
            stage.remove_from_scene(glyph);
            play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), water, DOWN, None)?.rt(0.25 * fast));
        } else {
            let rim = v(b.centre[0], b.rim() + 0.25);
            let hit = glyph
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.3 * fast),
                    ..AnimateArgs::default()
                })?
                .move_to(rim, ORIGIN)?
                .set_color(Role::lit())?;
            play!(stage; hit);
            let away = glyph
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.4 * fast),
                    ..AnimateArgs::default()
                })?
                .move_to(v(rim[0] - 1.8, rim[1] + 0.9), ORIGIN)?
                .set_opacity(0.0)?;
            play!(stage; away);
            stage.remove_from_scene(glyph);
        }
        Ok(())
    };
    // 05_3: most of the fill, the meter flat near zero
    let first_part: Vec<Draw> = draws.iter().copied().filter(|d| d.held < SLOTS - 3).collect();
    for d in &first_part {
        play_draw(stage, d, 0.75)?;
    }
    kit.hold(stage)?;

    // 05_4: the last three slots, one in eight, four, two
    kit.say(stage, "05_4")?;
    let mut readout: Option<Mob> = None;
    let mut level = None;
    for d in draws.iter().filter(|d| d.held >= SLOTS - 3) {
        let new_level = level != Some(d.held);
        level = Some(d.held);
        play_draw(stage, d, 1.4)?;
        if new_level {
            let text = format!("1/{}", 1u32 << (SLOTS - d.held));
            let r = m.readout(kit, stage, &text)?;
            match readout.take() {
                Some(old) => play!(stage; replacement_transform(old, r).rt(0.4)),
                None => play!(stage; fade_in(stage.arena_mut(), r, ORIGIN, 1.0)?.rt(0.4)),
            }
            readout = Some(r);
        }
    }
    let either = kit.text(
        stage,
        &format!(
            "only the last few slots matter: it fails ≈ {:.0}% either way (model)",
            p_fail_limit(2, 0) * 100.0
        ),
        28.0,
        Role::ex_or(),
        v(0.0, 3.4),
    )?;
    play!(stage; fade_in(stage.arena_mut(), either, ORIGIN, 1.0)?.rt(0.7));
    // the listen window: the run again, as sound
    // every card sounds before the score's fade (the window's last 0.8 s)
    let step = 1.8 / draws.len() as f64;
    let wav = sound::wav("05_4_rank_chord", &rank_chord(&draws, step, 3.0))?;
    kit.sonify(stage, 3.0, "05_4", &wav)?;
    // the run again, in step with the sound: a slot glows with each tone,
    // the rim with each thud
    stage.wait(0.15)?;
    for d in &draws {
        let target = if d.useful { waters[d.held as usize] } else { m.track };
        let pulse = fmn::animation::indicate(stage.arena_mut(), target, 1.08, Some([1.0, 1.0, 1.0]))?;
        stage.play(pulse.rt(step))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_fills_every_slot_and_the_danger_lives_at_the_end() {
        let draws = run(SLOTS, RUN_SEED);
        let useful = draws.iter().filter(|d| d.useful).count();
        assert_eq!(useful, SLOTS as usize, "one useful card per slot");
        // each slot's level appears in order
        let mut level = 0;
        for d in &draws {
            assert_eq!(d.held, level);
            if d.useful {
                level += 1;
            }
        }
        // the brute-force chance of a useless card at each level is the meter's
        for held in 0..4u32 {
            let mut b = Bucket::new();
            for i in 0..held {
                b.add(1 << i);
            }
            let useless = (0..1u32 << 4).filter(|&c| b.can_make(c)).count();
            assert_eq!(useless as f64 / 16.0, useless_chance(held, 4));
        }
    }

    #[test]
    fn the_rank_chord_is_finite_and_audible() {
        let draws = run(SLOTS, RUN_SEED);
        let s = rank_chord(&draws, 0.2, 3.0);
        assert!(s.l.iter().chain(&s.r).all(|x| x.is_finite()));
        let peak = s.l.iter().chain(&s.r).fold(0.0f64, |p, x| p.max(x.abs()));
        assert!(peak > 0.01 && peak < 1.0, "{peak}");
    }
}
