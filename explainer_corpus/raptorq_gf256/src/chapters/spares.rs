//! Chapter 06, spares and rings: the risk ladder (06_1), each ex-or spare a
//! rung down, 71 → 42 → 23 → 12%, and a quarter of a million phones stuck
//! at two spares (06_2); twenty rungs to one in a million (06_3); the ring
//! of five whose letters cancel to nothing (06_4); amounts of only 0 or 1
//! against amounts that can be any byte (06_5); and what a byte times a
//! byte must do (06_6). Every chance is `model.rs`'s (oracle B-F5, B-F28),
//! labelled model; the ring is `model::RING` (B-F9).

use explainer_kit::props::{equation_card, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{Role, badge, crossfade, group_of};
use super::props::{BucketProp, DangerMeter, RingProp, RiskLadder};
use crate::model::{PHONES, RING, SplitMix64, p_fail_limit, p_fail_limit_times, rank_gf2, spares_for, useless_chance};

/// The examples of scaling a piece (06_5): three times A, a hundred times A.
const AMOUNTS: [u32; 2] = [3, 100];
/// Phones drawn for 06_2's quarter of a million (one per thousand).
const DRAWN: u32 = 1_000;
/// 06_3's target, "one in a million" (B-F28: the fewest spares for it).
const ONE_IN: u32 = 1_000_000;

pub struct Spares {
    pub kit: Kit,
}

impl SceneConstruct for Spares {
    fn name(&self) -> &str {
        "06_spares_and_rings"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        ladder(&kit, stage)?;
        ring(&kit, stage)?;
        amounts(&kit, stage)?;
        kit.hold(stage)?;
        // the end of act one: music alone for a moment
        kit.music_breath(stage, 2.5)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// A chance as a whole percentage ("71%").
fn pct(p: f64) -> String {
    format!("{:.0}%", p * 100.0)
}

/// 06_1–06_3: the ladder, a rung per spare.
fn ladder(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let l = RiskLadder::build(kit, stage, v(-3.5, -0.35), 6.3, 6)?;
    let rungs_to = spares_for(ONE_IN);
    let rungs: Vec<Mob> = (0..=rungs_to)
        .map(|h| l.rung(stage, p_fail_limit(2, h), Role::ex_or()))
        .collect::<fmn::Result<_>>()?;
    let none = l.marker(
        kit,
        stage,
        p_fail_limit(2, 0),
        &format!("no spares: {}", pct(p_fail_limit(2, 0))),
        Role::ex_or(),
    )?;
    let caption = kit.text(
        stage,
        "the chance the lottery fails",
        28.0,
        Palette::dim(),
        v(-3.5, 3.55),
    )?;
    kit.say(stage, "06_1")?;
    play!(stage;
        fade_in(stage.arena_mut(), l.group, ORIGIN, 1.0)?.rt(0.9),
        fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.9),
    );
    play!(stage;
        fade_in(stage.arena_mut(), rungs[0], ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), none, v(0.2, 0.0), 1.0)?.rt(0.6),
    );
    let q = predict_pause(kit, stage, v(1.6, 0.6))?;

    // 06_2: each spare a rung down; two spares, nearly one in four
    kit.say(stage, "06_2")?;
    // the answer first, as the line gives it: two spares, nearly one in four
    let p2 = p_fail_limit(2, 2);
    let answer = kit.text(
        stage,
        &format!("2 spares: {}, nearly 1 in {:.0}", pct(p2), 1.0 / p2),
        34.0,
        Role::ex_or(),
        v(3.9, 3.35),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
        fade_in(stage.arena_mut(), answer, v(0.0, -0.15), 1.0)?.rt(0.6),
    );
    stage.wait(0.9)?;
    let spares_label = |h: u32| {
        format!(
            "{} spare{}: {}",
            h,
            if h == 1 { "" } else { "s" },
            pct(p_fail_limit(2, h))
        )
    };
    let mut marker = none;
    for h in 1..=3u32 {
        let next = l.marker(kit, stage, p_fail_limit(2, h), &spares_label(h), Role::ex_or())?;
        play!(stage;
            fade_in(stage.arena_mut(), rungs[h as usize], ORIGIN, 1.0)?.rt(0.4),
            replacement_transform(marker, next).rt(0.8),
        );
        marker = next;
        stage.wait(0.5)?;
    }
    // back to two spares: the phone panel is that case, and the ladder
    // must say the same (Codex round 5)
    let two = l.marker(kit, stage, p2, &spares_label(2), Role::ex_or())?;
    play!(stage; replacement_transform(marker, two).rt(0.6));
    marker = two;
    // a million phones at two spares: about a quarter stuck (model)
    let stuck = p_fail_limit_times(2, 2, PHONES as u32);
    let drawn_stuck = (p_fail_limit(2, 2) * f64::from(DRAWN)).round() as usize;
    let mut rng = SplitMix64::new(crate::model::FIELD_SEED ^ 0x0602);
    let mut order: Vec<usize> = (0..DRAWN as usize).collect();
    for i in (1..order.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let (cols, cell) = (40usize, 0.13);
    let origin = v(0.9, 2.3);
    let mut red = vec![false; DRAWN as usize];
    for &i in &order[..drawn_stuck] {
        red[i] = true;
    }
    let mut ok_dots = Vec::new();
    let mut red_dots = Vec::new();
    for (i, &is_red) in red.iter().enumerate() {
        let p = v(
            origin[0] + cell * (i % cols) as f64,
            origin[1] - cell * (i / cols) as f64,
        );
        let d = Dot::new().point(p).radius(0.045);
        if is_red {
            red_dots.push(d.color(Role::lost()).build());
        } else {
            ok_dots.push(d.color(Role::done()).build());
        }
    }
    let ok_group = stage.arena_mut().add(v_group(ok_dots));
    let red_group = stage.arena_mut().add(v_group(red_dots));
    let field_note = kit.text(
        stage,
        &format!(
            "with 2 spares: ≈ {} of {} phones stuck (model)",
            thousands((stuck / 10_000.0).round() as u64 * 10_000),
            thousands(PHONES)
        ),
        26.0,
        Role::lost(),
        v(origin[0] + cell * cols as f64 / 2.0, origin[1] - cell * 25.0 - 0.35),
    )?;
    let key = kit.text(
        stage,
        &format!("each dot = {} phones", thousands(PHONES / u64::from(DRAWN))),
        22.0,
        Palette::dim(),
        v(origin[0] + cell * cols as f64 / 2.0, origin[1] + 0.35),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), ok_group, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.8),
    );
    play!(stage;
        fade_in(stage.arena_mut(), red_group, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), field_note, ORIGIN, 1.0)?.rt(0.8),
    );
    kit.hold(stage)?;

    // 06_3: twenty rungs to one in a million; ten halvings, twenty halvings
    kit.say(stage, "06_3")?;
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in [ok_group, red_group, field_note, key, answer] {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.5),
            ..PlayOverrides::default()
        },
    )?;
    let rest: Vec<Mob> = rungs[4..].to_vec();
    let rest = group_of(stage, &rest)?;
    let last = p_fail_limit(2, rungs_to);
    let end = l.marker(
        kit,
        stage,
        last,
        &format!("{rungs_to} spares: about 1 in {}", thousands(ONE_IN)),
        Role::ex_or(),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), rest, ORIGIN, 1.0)?.rt(1.8).lag(0.4),
        replacement_transform(marker, end).rt(1.8),
    );
    let tex_at = |y: f64| v(2.8, y);
    let half = rungs_to / 2;
    let ten = kit.tex(
        stage,
        &format!(r"2^{{{half}}} = {}", thousands(1u32 << half)),
        52.0,
        Palette::ink(),
        &[],
        tex_at(1.2),
    )?;
    let twenty = kit.tex(
        stage,
        &format!(r"2^{{{rungs_to}}} = {}", thousands(1u32 << rungs_to)),
        52.0,
        Palette::ink(),
        &[],
        tex_at(-0.2),
    )?;
    let tag = kit.text(
        stage,
        "each spare halves it, near enough",
        26.0,
        Palette::dim(),
        tex_at(2.3),
    )?;
    play!(stage; fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.6));
    play!(stage; write(stage.arena(), ten).rt(0.9));
    stage.wait(0.6)?;
    play!(stage; write(stage.arena(), twenty).rt(0.9));
    kit.hold(stage)?;
    Ok(())
}

/// 06_4: five cards in a ring; every letter twice, so together nothing;
/// the five are worth only four.
fn ring(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let centre = v(-2.2, 0.1);
    let r = RingProp::build(kit, stage, centre, 2.45, 0.85, false)?;
    let groups: Vec<Mob> = r.cards.iter().map(|c| c.group).collect();
    let caption = kit.text(stage, "structure, not luck", 32.0, Palette::dim(), v(-2.2, 3.55))?;
    kit.say(stage, "06_4")?;
    let all = group_of(stage, &groups)?;
    crossfade(stage, &[all, caption], 0.9)?;
    stage.wait(2.4)?;
    // a copy of every badge flies to the middle, the ring staying whole;
    // the pairs meet and vanish
    let mut fly: Vec<Box<dyn Animation>> = Vec::new();
    let mut by_piece: Vec<Vec<Mob>> = vec![Vec::new(); 5];
    for c in &r.cards {
        for &(p, m) in &c.badges {
            let copy = stage.copy_family(m)?;
            stage.add_to_scene(copy)?;
            by_piece[p].push(copy);
        }
    }
    for (p, ms) in by_piece.iter().enumerate() {
        let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * p as f64 / 5.0;
        let to = v(centre[0] + 0.75 * a.cos(), centre[1] + 0.75 * a.sin());
        for &m in ms {
            let anim = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.4),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            fly.push(stage.prepare(anim)?);
        }
    }
    stage.play_prepared(fly)?;
    let mut gone: Vec<Box<dyn Animation>> = Vec::new();
    for ms in &by_piece {
        for &m in ms {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.8)?;
            gone.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared_with(
        gone,
        PlayOverrides {
            run_time: Some(0.9),
            ..PlayOverrides::default()
        },
    )?;
    let nothing = kit.text(stage, "nothing", 40.0, Role::ex_or(), centre)?;
    play!(stage; fade_in(stage.arena_mut(), nothing, ORIGIN, 0.6)?.rt(0.6));
    let worth = kit.text(
        stage,
        &format!("{} cards, worth only {}", RING.len(), rank_gf2(&RING)),
        40.0,
        Palette::ink(),
        v(3.9, 0.1),
    )?;
    play!(stage; write(stage.arena(), worth).rt(1.0));
    kit.hold(stage)?;
    Ok(())
}

/// 06_5–06_6: amounts of 0 or 1, the last slot at one in two and the stuck
/// ring; amounts that scale a piece; any byte as the amount; what a byte
/// times a byte must do.
fn amounts(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    // the last slot of a four-slot bucket, its meter at one in two
    let b = BucketProp::build(stage, 4, v(-4.7, -1.5), 1.6, 2.8)?;
    let m = DangerMeter::on(kit, stage, &b, useless_chance(3, 4))?;
    let filled: Vec<Mob> = (0..3)
        .map(|i| b.water(stage, i, Role::ex_or()))
        .collect::<fmn::Result<_>>()?;
    let reading = m.readout(kit, stage, &format!("1/{:.0}", 1.0 / useless_chance(3, 4)))?;
    let mini = RingProp::build(kit, stage, v(-2.4, 2.15), 1.0, 0.4, false)?;
    let mini_groups: Vec<Mob> = mini.cards.iter().map(|c| c.group).collect();
    let mini = group_of(stage, &mini_groups)?;
    let set01 = kit.tex(
        stage,
        r"\text{amount} \in \{0, 1\}",
        64.0,
        Role::ex_or(),
        &[],
        v(2.2, 2.0),
    )?;
    let mut bucket_parts = vec![b.group, m.group, reading];
    bucket_parts.extend(&filled);
    let bucket_all = group_of(stage, &bucket_parts)?;
    kit.say(stage, "06_5")?;
    crossfade(stage, &[bucket_all, mini, set01], 0.9)?;
    stage.add_to_scene(m.fill)?;
    stage.wait(3.4)?;
    // scaling a piece: three times A, a hundred times A
    let examples: Vec<Mob> = AMOUNTS
        .iter()
        .enumerate()
        .map(|(k, &amount)| {
            let at = v(0.6 + 3.0 * k as f64, 0.4);
            let num = kit.text(
                stage,
                &format!("{amount} ×"),
                48.0,
                Role::byte(),
                v(at[0] - 0.55, at[1]),
            )?;
            stage.move_to(num, v(at[0] - 0.05, at[1]), RIGHT);
            let a = badge(kit, stage, 0, 0.36, v(at[0] + 0.45, at[1]))?;
            group_of(stage, &[num, a])
        })
        .collect::<fmn::Result<_>>()?;
    for &e in &examples {
        play!(stage; fade_in(stage.arena_mut(), e, v(0.0, 0.2), 1.0)?.rt(0.6));
        stage.wait(0.5)?;
    }
    // any byte: 256 amounts, one dot each
    let bytes = 1u32 << 8;
    let (side, pitch) = (16usize, 0.16);
    let grid_c = v(2.4, -2.0);
    let dots: Vec<VMobject> = (0..bytes as usize)
        .map(|i| {
            let p = v(
                grid_c[0] + pitch * (i % side) as f64 - pitch * (side as f64 - 1.0) / 2.0,
                grid_c[1] + pitch * (side as f64 - 1.0) / 2.0 - pitch * (i / side) as f64,
            );
            Dot::new().point(p).radius(0.05).color(Role::byte()).build()
        })
        .collect();
    let dots = stage.arena_mut().add(v_group(dots));
    let set256 = kit.tex(
        stage,
        &format!(r"\text{{amount: any byte, }} {bytes} \text{{ choices}}"),
        52.0,
        Role::byte(),
        &[],
        v(2.2, 2.0),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), set01, v(0.0, 0.3), 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), set256, v(0.0, -0.3), 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), dots, ORIGIN, 1.0)?.rt(1.6).lag(0.01),
    );
    kit.hold(stage)?;

    // 06_6: what the arithmetic must do
    let need = equation_card(
        kit,
        stage,
        r"\text{byte} \times \text{byte} \to \text{byte}, \text{ and undo it}",
        60.0,
        &[(r"\times", Role::byte())],
        v(0.0, 0.0),
    )?;
    kit.say(stage, "06_6")?;
    crossfade(stage, &[need], 0.9)?;
    Ok(())
}
