//! Chapter 01, the hook: a million phones. The picture goes out to a field of
//! 1,000 phone glyphs (a seeded sample, one per 1,000 phones), each losing
//! its own packets (01_1); the field lights up and the secret is chalked
//! (01_2); the title (01_3); the picture cut into pieces and one lost (01_4);
//! requests swamping the tower, and a TV antenna that can't ask (01_5); the
//! carousel loop by loop, the stragglers, and the unluckiest of a million
//! (01_6); the sticker album and its count (01_7–01_8). Every count comes
//! from `model.rs`, the field from its seeded draw (oracle B-F24.field).

use std::f64::consts::TAU;

use explainer_kit::props::{Pose, RowStrips, Swarm, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, hex, play, thousands, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::draw::{Role, crossfade, flat, group_of, line_style, packet, phone, tower, tower_tip, waves};
use super::picture::{self, SIDE};
use crate::model::{
    GLYPHS, HOOK_PIECES, LOSS_ONE_IN, PHONES, SplitMix64, album_filled, album_mean, album_mean_floor,
    carousel_mean_caught, loss_flecks, phone_field, unluckiest_of_a_million,
};

/// The field's grid of glyphs.
const COLS: usize = 40;
const ROWS: usize = 25;
const _: () = assert!(COLS * ROWS == GLYPHS);
/// The field's extent.
const FIELD_X: (f64, f64) = (-3.3, 6.7);
const FIELD_Y: (f64, f64) = (-3.5, 2.9);
/// A phone glyph's height.
const PHONE_H: f64 = 0.19;
/// The tower: its base and height.
const TOWER_BASE: Vec3 = [-5.7, -3.0, 0.0];
const TOWER_H: f64 = 3.2;
/// The picture beside the tower, and the carousel round it.
const SMALL_PICTURE: (f64, Vec3) = (1.3, [-5.7, 2.35, 0.0]);
/// Packets shown streaming in 01_1, against which each phone's losses fall.
const SHOWN: usize = 300;

/// Where glyph `i` sits.
fn glyph_at(i: usize) -> Vec3 {
    let (col, row) = (i % COLS, i / COLS);
    let dx = (FIELD_X.1 - FIELD_X.0) / COLS as f64;
    let dy = (FIELD_Y.1 - FIELD_Y.0) / ROWS as f64;
    v(FIELD_X.0 + (col as f64 + 0.5) * dx, FIELD_Y.1 - (row as f64 + 0.5) * dy)
}

/// The phone field: one group per phone, and each phone's body (the part
/// that changes colour).
struct Field {
    group: Mob,
    bodies: Vec<Mob>,
}

fn field(stage: &mut Stage<'_>) -> Field {
    let phones: Vec<VMobject> = (0..GLYPHS)
        .map(|i| phone(Role::waiting(), PHONE_H).moved_to(glyph_at(i)))
        .collect();
    let group = stage.arena_mut().add(v_group(phones));
    let bodies = children(stage, group)
        .into_iter()
        .map(|p| children(stage, p)[0])
        .collect();
    Field { group, bodies }
}

/// Recolour `bodies` to `color` in a sweep from the field's left edge to its
/// right over `rt` seconds.
fn sweep(stage: &mut Stage<'_>, bodies: &[Mob], color: Srgb, rt: f64) -> fmn::Result<()> {
    let mut pending: Vec<(Mob, f64)> = bodies.iter().map(|&m| (m, stage.get_center(m)[0])).collect();
    let front = stage.add_value_tracker(FIELD_X.0 - 0.1);
    let driver = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(driver)?;
    stage.add_updater(
        driver,
        move |s, _| {
            let f = s.tracker_value(front).unwrap_or(f64::NEG_INFINITY);
            pending.retain(|&(m, x)| {
                let reached = x <= f;
                if reached {
                    s.set_fill(m, Some(color), None, None, false);
                }
                !reached
            });
        },
        false,
    )?;
    stage.play(tween(front, FIELD_X.1 + 0.1, rt)?)?;
    stage.clear_updaters(driver, false);
    stage.remove_from_scene(driver);
    Ok(())
}

/// A swarm member's straight flight: from `from` to `to`, starting at
/// `delay` s, taking `dur` s; `repeat` sends it again from the start.
#[derive(Clone, Copy)]
struct Flight {
    from: Vec3,
    to: Vec3,
    delay: f64,
    dur: f64,
    repeat: bool,
}

impl Flight {
    fn pose(&self, t: f64) -> Pose {
        let mut u = (t - self.delay) / self.dur;
        if self.repeat && u > 0.0 {
            u = u.fract();
        }
        let u = u.clamp(0.0, 1.0);
        let heading = (self.to[1] - self.from[1]).atan2(self.to[0] - self.from[0]);
        Pose {
            x: self.from[0] + (self.to[0] - self.from[0]) * u,
            y: self.from[1] + (self.to[1] - self.from[1]) * u,
            heading,
        }
    }
}

/// A swarm of copies of `template` flying `flights`, driven from now on.
fn launch(stage: &mut Stage<'_>, template: &VMobject, flights: Vec<Flight>) -> fmn::Result<Swarm> {
    let poses: Vec<Pose> = flights.iter().map(|f| f.pose(0.0)).collect();
    let swarm = Swarm::build(stage, template, &poses, None)?;
    stage.add_to_scene(swarm.group)?;
    let mut t = 0.0;
    swarm.drive(stage, poses, move |dt, ps| {
        t += dt;
        for (p, f) in ps.iter_mut().zip(&flights) {
            *p = f.pose(t);
        }
    })?;
    Ok(swarm)
}

/// A small arrow pointing along +x (a request).
fn arrow_glyph(color: Srgb) -> VMobject {
    Polygon::new([
        v(0.09, 0.0),
        v(0.0, 0.05),
        v(0.0, 0.018),
        v(-0.08, 0.018),
        v(-0.08, -0.018),
        v(0.0, -0.018),
        v(0.0, -0.05),
    ])
    .style(flat(color, 0.95))
    .build()
}

pub struct Hook {
    pub kit: Kit,
}

impl SceneConstruct for Hook {
    fn name(&self) -> &str {
        "01_hook"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let chalk = broadcast(&kit, stage)?;
        title(&kit, stage, chalk)?;
        pieces(&kit, stage)?;
        requests(&kit, stage)?;
        carousel(&kit, stage)?;
        album(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// The tower, its picture and the phone field, as 01_1, 01_5 and 01_6 show
/// them.
struct Stage1 {
    tower: Mob,
    picture: RowStrips,
    field: Field,
    key: Mob,
}

fn stage1(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Stage1> {
    let tower = stage.arena_mut().add(tower(TOWER_H, TOWER_BASE, Palette::dim())?);
    let picture = picture::picture(stage, SMALL_PICTURE.0, SMALL_PICTURE.1)?;
    let field = field(stage);
    let per = PHONES / GLYPHS as u64;
    let key = kit.text(
        stage,
        &format!("each phone drawn = {} phones", thousands(per)),
        24.0,
        Palette::dim(),
        v(4.6, 3.45),
    )?;
    Ok(Stage1 {
        tower,
        picture,
        field,
        key,
    })
}

/// 01_1–01_2: the picture to a million phones, each losing its own packets;
/// then every phone rebuilds it, and the secret is chalked (returned for the
/// title).
fn broadcast(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let s1 = stage1(kit, stage)?;
    // each phone's own losses among the packets shown: a red fleck at an angle
    // set by the packet's place in the stream, in ten batches by time
    let batches = 10;
    let mut flecks: Vec<Vec<VMobject>> = vec![Vec::new(); batches];
    for i in 0..GLYPHS {
        let c = glyph_at(i);
        for k in loss_flecks(i, SHOWN) {
            let a = TAU * k as f64 / SHOWN as f64;
            let p = v(c[0] + 0.115 * a.cos(), c[1] + 0.115 * a.sin());
            flecks[k * batches / SHOWN].push(Dot::new().point(p).radius(0.022).color(Role::lost()).build());
        }
    }
    let flecks: Vec<Mob> = flecks.into_iter().map(|b| stage.arena_mut().add(v_group(b))).collect();

    // "this picture": it opens big at the centre, then flies into the tower
    // as the phones appear
    let (big_size, big_at) = (3.6, v(0.0, 0.2));
    let big = picture::picture(stage, big_size, big_at)?;
    kit.say(stage, "01_1")?;
    play!(stage; fade_in(stage.arena_mut(), big.group, ORIGIN, 0.85)?.rt(0.7));
    stage.wait(0.9)?;
    let to_tower = v(SMALL_PICTURE.1[0] - big_at[0], SMALL_PICTURE.1[1] - big_at[1]);
    play!(stage;
        fade_out(stage.arena_mut(), big.group, to_tower, SMALL_PICTURE.0 / big_size)?.rt(0.9),
        fade_in(stage.arena_mut(), s1.picture.group, ORIGIN, 1.0)?.rt(0.9),
        show_creation(s1.tower).rt(0.9),
        fade_in(stage.arena_mut(), s1.field.group, ORIGIN, 1.0)?.rt(1.2).lag(0.0008),
        fade_in(stage.arena_mut(), s1.key, ORIGIN, 1.0)?.rt(0.8),
    );
    // packets stream from the tower to the phones; losses spark as they go
    let tip = tower_tip(TOWER_BASE, TOWER_H);
    let mut rng = SplitMix64::new(0x0101);
    let flights: Vec<Flight> = (0..48)
        .map(|k| Flight {
            from: tip,
            to: glyph_at((rng.next_u64() % GLYPHS as u64) as usize),
            delay: 0.06 * k as f64,
            dur: 1.1,
            repeat: true,
        })
        .collect();
    let stream = launch(stage, &packet(0.16, Role::card()), flights)?;
    let wave = stage.arena_mut().add(waves(tip, 0.3, 0.22, 3, Role::card())?);
    play!(stage; fade_in(stage.arena_mut(), wave, ORIGIN, 1.0)?.rt(0.6));
    for b in flecks.iter() {
        play!(stage; fade_in(stage.arena_mut(), *b, ORIGIN, 1.0)?.rt(0.32));
    }
    kit.hold(stage)?;

    // 01_2: two packets more than it needs, and every phone holds the picture
    kit.say(stage, "01_2")?;
    stage.clear_updaters(stream.group, true);
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in flecks.iter().copied().chain([stream.group, wave]) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.8),
            ..PlayOverrides::default()
        },
    )?;
    let caught = kit.text(
        stage,
        &format!(
            "{} pieces  ·  {} packets caught  →  picture rebuilt",
            thousands(HOOK_PIECES),
            thousands(HOOK_PIECES + 2)
        ),
        28.0,
        Palette::ink(),
        v(1.7, 3.45),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), s1.key, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), caught, ORIGIN, 1.0)?.rt(0.8),
    );
    sweep(stage, &s1.field.bodies, Role::done(), 1.8)?;
    stage.wait(0.8)?;
    // the secret, in chalk
    let gone: Vec<Mob> = vec![s1.field.group, s1.tower, s1.picture.group, caught];
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in gone {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.8),
            ..PlayOverrides::default()
        },
    )?;
    let chalk = kit.tex(
        stage,
        r"3 \times 7 = 9",
        150.0,
        hex("#F4F0E6"),
        &[("9", Role::byte())],
        v(0.0, -0.2),
    )?;
    play!(stage; write(stage.arena(), chalk).rt(1.6));
    // no hold: the title forms while the line names the arithmetic
    Ok(chalk)
}

/// 01_3: the title, the chalk sliding into it, over the motif.
fn title(kit: &Kit, stage: &mut Stage<'_>, chalk: Mob) -> fmn::Result<()> {
    let words = kit.text_styled(stage, "The Arithmetic Where", 64.0, Palette::ink(), ORIGIN, true, false)?;
    // one line: the words, a space, the equation at title size
    let eq_scale = 0.46;
    let (ww, ew) = (stage.get_width(words), stage.get_width(chalk) * eq_scale);
    let gap = 0.35;
    let left = -(ww + gap + ew) / 2.0;
    let y = 0.3;
    stage.move_to(words, v(left, y), LEFT);
    let words_bottom = stage.get_bottom(words)[1];
    let eq_h = stage.get_height(chalk) * eq_scale;
    let to = v(left + ww + gap + ew / 2.0, words_bottom + eq_h / 2.0);
    let slide = chalk
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.4),
            ..AnimateArgs::default()
        })?
        .scale(eq_scale)?
        .move_to(to, ORIGIN)?;
    play!(stage; slide, write(stage.arena(), words).rt(1.3));
    kit.listen(stage, 3.0, "01_3")?;
    Ok(())
}

/// 01_4: the picture cut into pieces, a row of them packed into packets on
/// their way to a phone; one is lost, and the phone asks again.
fn pieces(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let (size, at) = (3.2, v(-4.4, 0.55));
    let big = picture::picture(stage, size, at)?;
    let cell = size / SIDE as f64;
    // the slicing: a faint line every tenth cell, then every cell in a corner
    let mut lines = Vec::new();
    for k in 0..=10 {
        let off = k as f64 * size / 10.0;
        let (l, t) = (at[0] - size / 2.0, at[1] + size / 2.0);
        lines.push(
            Line::new(v(l + off, t), v(l + off, t - size))
                .style(line_style(hex("#0E1116"), 1.5))
                .build()?,
        );
        lines.push(
            Line::new(v(l, t - off), v(l + size, t - off))
                .style(line_style(hex("#0E1116"), 1.5))
                .build()?,
        );
    }
    let grid = stage.arena_mut().add(v_group(lines));
    // the magnified block: 5 × 5 pieces where the sun meets sky and hill
    let (r0, c0, n) = (43usize, 51usize, 5usize);
    let corner = v(
        at[0] - size / 2.0 + c0 as f64 * cell,
        at[1] + size / 2.0 - r0 as f64 * cell,
    );
    let frame = Rectangle::new()
        .width(n as f64 * cell)
        .height(n as f64 * cell)
        .style(line_style(Palette::ink(), 2.5))
        .build()?
        .moved_to(v(corner[0] + n as f64 * cell / 2.0, corner[1] - n as f64 * cell / 2.0));
    let frame = stage.arena_mut().add(frame);
    let (side, gap) = (0.46, 0.08);
    let inset_c = v(-0.2, 0.55);
    let colors = picture::colors();
    let mut squares = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let p = v(
                inset_c[0] + (c as f64 - (n as f64 - 1.0) / 2.0) * (side + gap),
                inset_c[1] - (r as f64 - (n as f64 - 1.0) / 2.0) * (side + gap),
            );
            let sq = Rectangle::new()
                .width(side)
                .height(side)
                .corner_radius(0.04)
                .style(flat(colors[r0 + r][c0 + c], 1.0))
                .build()?
                .moved_to(p);
            squares.push(stage.arena_mut().add(sq));
        }
    }
    let inset = group_of(stage, &squares)?;
    let span = (n as f64) * (side + gap) / 2.0;
    let leader = stage.arena_mut().add(v_group([
        Line::new(
            v(corner[0] + n as f64 * cell, corner[1]),
            v(inset_c[0] - span, inset_c[1] + span),
        )
        .style(line_style(Palette::faint(), 2.0))
        .build()?,
        Line::new(
            v(corner[0] + n as f64 * cell, corner[1] - n as f64 * cell),
            v(inset_c[0] - span, inset_c[1] - span),
        )
        .style(line_style(Palette::faint(), 2.0))
        .build()?,
    ]));
    let label = kit.text(
        stage,
        &format!("{} pieces", thousands(HOOK_PIECES)),
        34.0,
        Palette::ink(),
        v(at[0], at[1] - size / 2.0 - 0.45),
    )?;

    kit.say(stage, "01_4")?;
    crossfade(stage, &[big.group], 0.9)?;
    play!(stage; show_creation(grid).rt(1.0));
    play!(stage;
        show_creation(frame).rt(0.6),
        show_creation(leader).rt(0.6),
        fade_in(stage.arena_mut(), inset, v(-0.3, 0.0), 1.0)?.rt(0.9).lag(0.04),
    );
    // the block's middle row (sun, sky and hill) goes out, one piece per
    // packet, to a phone
    let phone_at = v(5.9, 0.55);
    let receiver = stage.arena_mut().add(phone(Role::waiting(), 1.1).moved_to(phone_at));
    play!(stage; fade_in(stage.arena_mut(), receiver, ORIGIN, 1.0)?.rt(0.5));
    let row: Vec<Mob> = (0..n).map(|c| squares[(n / 2) * n + c]).collect();
    let mut cards = Vec::new();
    for k in 0..row.len() {
        let x = 1.9 + 0.82 * k as f64;
        let card = Rectangle::new()
            .width(0.7)
            .height(0.7)
            .corner_radius(0.08)
            .style(
                Style::default()
                    .fill(super::draw::card_fill(), 0.96)
                    .stroke(Role::card(), 3.0, 1.0),
            )
            .build()?
            .moved_to(v(x, -1.9));
        let card = stage.arena_mut().add(card);
        cards.push(card);
    }
    let mut pack: Vec<Box<dyn Animation>> = Vec::new();
    for (k, (&sq, &card)) in row.iter().zip(&cards).enumerate() {
        let to = stage.get_center(card);
        let a = fade_in(stage.arena_mut(), card, ORIGIN, 0.6)?;
        pack.push(stage.prepare(a)?);
        let m = sq
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0 + 0.1 * k as f64),
                ..AnimateArgs::default()
            })?
            .scale(0.8)?
            .move_to(to, ORIGIN)?;
        pack.push(stage.prepare(m)?);
    }
    stage.play_prepared(pack)?;
    play!(stage; fade_in(stage.arena_mut(), label, v(0.0, -0.1), 1.0)?.rt(0.8));
    // "Lose some": the loss waits for its sentence
    stage.wait(1.4)?;
    // the packets travel to the phone; the third is lost on the way
    let lost = 2;
    let mut travel: Vec<Box<dyn Animation>> = Vec::new();
    for (k, (&sq, &card)) in row.iter().zip(&cards).enumerate() {
        let dest = if k == lost {
            v(4.4, -2.9)
        } else {
            v(phone_at[0], phone_at[1] - 0.15)
        };
        for m in [sq, card] {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.4),
                    ..AnimateArgs::default()
                })?
                .move_to(dest, ORIGIN)?
                .scale(if k == lost { 1.0 } else { 0.3 })?;
            travel.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared(travel)?;
    // the lost one turns red and fades; the arrived ones vanish into the phone
    let mut settle: Vec<Box<dyn Animation>> = Vec::new();
    for (k, (&sq, &card)) in row.iter().zip(&cards).enumerate() {
        if k == lost {
            let a = card
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(Role::lost()), None, Some(0.6))?;
            settle.push(stage.prepare(a)?);
            let b = sq
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .set_opacity(0.35)?;
            settle.push(stage.prepare(b)?);
        } else {
            for m in [sq, card] {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                settle.push(stage.prepare(a)?);
            }
        }
    }
    stage.play_prepared(settle)?;
    // the usual fix: ask again
    let ask = Arrow::new(v(phone_at[0] - 0.3, phone_at[1] + 0.75), v(-2.55, 2.3))
        .thickness(4.0)
        .path_arc(0.5)
        .style(Style::default().fill(Role::lost(), 1.0).stroke(Role::lost(), 4.0, 1.0))
        .build()?;
    let ask = stage.arena_mut().add(ask);
    let again = kit.text(stage, "send it again?", 30.0, Role::lost(), v(1.6, 3.45))?;
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), ask)?.rt(0.9),
        fade_in(stage.arena_mut(), again, ORIGIN, 1.0)?.rt(0.9),
    );
    kit.hold(stage)?;
    Ok(())
}

/// 01_5: one packet missed by about 10,000 phones; the requests swamp the
/// tower; a TV antenna has no way back.
fn requests(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let s1 = stage1(kit, stage)?;
    let tip = tower_tip(TOWER_BASE, TOWER_H);
    kit.say(stage, "01_5")?;
    crossfade(stage, &[s1.tower, s1.picture.group, s1.field.group, s1.key], 0.7)?;
    // thirty packets; for each, the glyphs that miss it (each with chance
    // 1 in 100), seeded
    let mut rng = SplitMix64::new(0x0105);
    let packets = 30;
    let missed: Vec<Vec<usize>> = (0..packets)
        .map(|_| {
            (0..GLYPHS)
                .filter(|_| rng.uniform() < 1.0 / f64::from(LOSS_ONE_IN))
                .collect()
        })
        .collect();
    // the first packet: its misses light red, about one glyph in a hundred
    let first: Vec<Mob> = missed[0].iter().map(|&i| s1.field.bodies[i]).collect();
    let flash = launch(
        stage,
        &packet(0.22, Role::card()),
        vec![Flight {
            from: tip,
            to: v(FIELD_X.1 + 0.6, tip[1]),
            delay: 0.0,
            dur: 1.2,
            repeat: false,
        }],
    )?;
    sweep(stage, &first, Role::lost(), 1.2)?;
    stage.clear_updaters(flash.group, true);
    stage.remove_from_scene(flash.group);
    let misses = kit.text(
        stage,
        &format!(
            "one packet: missed by ≈ {} phones",
            thousands(PHONES / u64::from(LOSS_ONE_IN))
        ),
        28.0,
        Role::lost(),
        v(1.7, 3.45),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), s1.key, ORIGIN, 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), misses, ORIGIN, 1.0)?.rt(0.6),
    );
    // every miss asks again: the requests pile onto the tower
    let mut flights = Vec::new();
    let mut pile = SplitMix64::new(0x0106);
    for (k, glyphs) in missed.iter().enumerate() {
        for &i in glyphs {
            let a = TAU * pile.uniform();
            let r = 0.25 + 0.55 * pile.uniform().sqrt();
            flights.push(Flight {
                from: glyph_at(i),
                to: v(tip[0] + r * a.cos() * 0.8, tip[1] + 0.3 + r * a.sin()),
                delay: 0.05 * k as f64,
                dur: 1.0,
                repeat: false,
            });
        }
    }
    flights.truncate(300);
    let swarm = launch(stage, &arrow_glyph(Role::lost()), flights)?;
    stage.wait(4.4)?;
    stage.clear_updaters(swarm.group, true);
    // the line's last sentence: a TV antenna cannot ask at all
    fade_all(stage, 0.5)?;
    let tower2 = stage.arena_mut().add(tower(2.8, v(-5.2, -2.6), Palette::dim())?);
    let tip2 = tower_tip(v(-5.2, -2.6), 2.8);
    let sky = stage.arena_mut().add(waves(tip2, 0.3, 0.25, 4, Role::card())?);
    let house = house(v(3.6, -2.6))?;
    let house = stage.arena_mut().add(house);
    let mast_top = v(3.6, 0.55);
    let back = Arrow::new(v(mast_top[0] - 0.4, mast_top[1] + 0.2), v(tip2[0] + 0.5, tip2[1] + 0.2))
        .thickness(4.0)
        .path_arc(0.35)
        .style(
            Style::default()
                .fill(Palette::dim(), 1.0)
                .stroke(Palette::dim(), 4.0, 0.8),
        )
        .build()?;
    let back = stage.arena_mut().add(back);
    let cross_at = v((mast_top[0] + tip2[0]) / 2.0, mast_top[1] + 1.15);
    let cross = stage.arena_mut().add(v_group([
        Line::new(
            v(cross_at[0] - 0.35, cross_at[1] - 0.35),
            v(cross_at[0] + 0.35, cross_at[1] + 0.35),
        )
        .style(line_style(Role::lost(), 9.0))
        .build()?,
        Line::new(
            v(cross_at[0] - 0.35, cross_at[1] + 0.35),
            v(cross_at[0] + 0.35, cross_at[1] - 0.35),
        )
        .style(line_style(Role::lost(), 9.0))
        .build()?,
    ]));
    let caption = kit.text(
        stage,
        "no return channel",
        34.0,
        Role::lost(),
        v(cross_at[0], cross_at[1] + 0.85),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), tower2, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), house, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), sky, ORIGIN, 0.6)?.rt(0.5),
    );
    play!(stage; fmn::animation::grow_arrow(stage.arena_mut(), back)?.rt(0.7));
    play!(stage;
        show_creation(cross).rt(0.5),
        fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.5),
    );
    kit.hold(stage)?;
    Ok(())
}

/// A house with a TV antenna on its roof, standing on `ground`.
fn house(ground: Vec3) -> fmn::Result<VMobject> {
    let (w, h) = (2.4, 1.6);
    let ink = Palette::dim();
    let body = Rectangle::new()
        .width(w)
        .height(h)
        .style(line_style(ink, 3.5))
        .build()?
        .moved_to(v(ground[0], ground[1] + h / 2.0));
    let roof = Polygon::new([
        v(ground[0] - w / 2.0 - 0.2, ground[1] + h),
        v(ground[0], ground[1] + h + 1.0),
        v(ground[0] + w / 2.0 + 0.2, ground[1] + h),
    ])
    .style(line_style(ink, 3.5))
    .build();
    let door = Rectangle::new()
        .width(0.45)
        .height(0.8)
        .style(line_style(ink, 3.0))
        .build()?
        .moved_to(v(ground[0] - 0.5, ground[1] + 0.4));
    let screen = Rectangle::new()
        .width(0.7)
        .height(0.5)
        .corner_radius(0.06)
        .style(flat(Role::card(), 0.85))
        .build()?
        .moved_to(v(ground[0] + 0.5, ground[1] + 0.85));
    // the antenna: a mast and three cross bars
    let base = v(ground[0], ground[1] + h + 1.0);
    let top = v(ground[0], base[1] + 1.15);
    let mut parts = vec![
        body,
        roof,
        door,
        screen,
        Line::new(base, top).style(line_style(ink, 3.5)).build()?,
    ];
    for (k, half) in [0.5, 0.38, 0.26].into_iter().enumerate() {
        let y = top[1] - 0.12 - 0.28 * k as f64;
        parts.push(
            Line::new(v(top[0] - half, y), v(top[0] + half, y))
                .style(line_style(ink, 3.0))
                .build()?,
        );
    }
    Ok(v_group(parts))
}

/// 01_6: the carousel, loop by loop; the stragglers after loop 3; the
/// unluckiest of a million through loops 5 and 6.
fn carousel(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let s1 = stage1(kit, stage)?;
    let loops = phone_field();
    let worst = unluckiest_of_a_million();
    // the carousel: the file's packets on a ring round the picture
    let (ring_r, centre) = (1.0, SMALL_PICTURE.1);
    let ring: Vec<VMobject> = (0..20)
        .map(|k| {
            let a = TAU * k as f64 / 20.0;
            packet(0.2, Role::card()).moved_to(v(centre[0] + ring_r * a.cos(), centre[1] + ring_r * a.sin()))
        })
        .collect();
    let ring = stage.arena_mut().add(v_group(ring));
    // the loop counter: a word and a live number beside it
    let loop_n = stage.add_value_tracker(1.0);
    let word = kit.text(
        stage,
        "loop",
        34.0,
        Palette::ink(),
        v(TOWER_BASE[0] - 0.25, TOWER_BASE[1] - 0.5),
    )?;
    let number_at = v(stage.get_right(word)[0] + 0.15, TOWER_BASE[1] - 0.5);
    kit.say(stage, "01_6")?;
    crossfade(stage, &[s1.tower, s1.picture.group, s1.field.group, s1.key, ring], 0.7)?;
    let turn = |stage: &mut Stage<'_>| -> fmn::Result<()> {
        let r = Rotating::new(ring).with_angle(-TAU).with_about_point(centre);
        stage.play(r.rt(1.05))?;
        Ok(())
    };
    stage.add_to_scene(word)?;
    kit.live_number(stage, loop_n, 0, false, 34.0, Palette::ink(), number_at, LEFT)?;
    for m in 1..=4u32 {
        if m > 1 {
            stage.set_tracker_value(loop_n, f64::from(m))?;
        }
        turn(stage)?;
        let done: Vec<Mob> = (0..GLYPHS)
            .filter(|&i| loops[i] == m)
            .map(|i| s1.field.bodies[i])
            .collect();
        sweep(stage, &done, Role::done(), 0.5)?;
        if m == 3 {
            // the stragglers: still waiting after three loops
            let late: Vec<Mob> = (0..GLYPHS)
                .filter(|&i| loops[i] > 3)
                .map(|i| s1.field.bodies[i])
                .collect();
            for &b in &late {
                stage.set_fill(b, Some(Role::lost()), None, None, false);
            }
            let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
            for &b in &late {
                let a = indicate(stage.arena_mut(), b, 2.2, Some([0.99, 0.38, 0.33]))?;
                pulse.push(stage.prepare(a)?);
            }
            if !pulse.is_empty() {
                stage.play_prepared_with(
                    pulse,
                    PlayOverrides {
                        run_time: Some(0.9),
                        ..PlayOverrides::default()
                    },
                )?;
            }
        }
    }
    // the unluckiest of a million: one phone, loop after loop
    let big_at = v(1.7, -0.6);
    let backdrop = stage.arena_mut().add(
        Rectangle::new()
            .width(5.6)
            .height(4.6)
            .corner_radius(0.2)
            .style(flat(hex(explainer_kit::BACKGROUND), 0.94))
            .build()?
            .moved_to(v(big_at[0], big_at[1] + 0.35)),
    );
    let big = stage.arena_mut().add(phone(Role::waiting(), 2.6).moved_to(big_at));
    let big_body = children(stage, big)[0];
    let who = kit.text(
        stage,
        &format!("the unluckiest of {} phones", thousands(PHONES)),
        34.0,
        Palette::ink(),
        v(big_at[0], big_at[1] + 1.95),
    )?;
    let dim = s1
        .field
        .group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.6),
            ..AnimateArgs::default()
        })?
        .set_opacity(0.12)?;
    play!(stage;
        dim,
        fade_in(stage.arena_mut(), backdrop, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), big, ORIGIN, 0.4)?.rt(0.8),
        fade_in(stage.arena_mut(), who, ORIGIN, 1.0)?.rt(0.8),
    );
    for m in 5..=worst {
        stage.set_tracker_value(loop_n, f64::from(m))?;
        turn(stage)?;
    }
    // done at last: green, the picture on its screen
    let shown = picture::picture(stage, 1.05, v(big_at[0], big_at[1] + 0.1))?;
    play!(stage;
        big_body.animate().set_anim_args(AnimateArgs { run_time: Some(0.6), ..AnimateArgs::default() })?
            .set_fill(Some(Role::done()), None)?,
        fade_in(stage.arena_mut(), shown.group, ORIGIN, 1.0)?.rt(0.8),
    );
    kit.hold(stage)?;
    Ok(())
}

/// 01_7–01_8: the sticker album; random pieces until all 10,000 are seen,
/// counted on average; then the three counts side by side.
fn album(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let k = HOOK_PIECES as usize;
    let (size, at) = (5.6, v(-3.4, -0.35));
    let empty = hex("#1A2130");
    let art = picture::colors();
    // the order the slots fill in: a seeded shuffle
    let mut rng = SplitMix64::new(0x0107);
    let mut order: Vec<usize> = (0..k).collect();
    for i in (1..k).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let mut rank = vec![0usize; k];
    for (r, &slot) in order.iter().enumerate() {
        rank[slot] = r;
    }
    let colors_for = move |lit: usize| -> Vec<Vec<Srgb>> {
        (0..SIDE)
            .map(|r| {
                (0..SIDE)
                    .map(|c| if rank[r * SIDE + c] < lit { art[r][c] } else { empty })
                    .collect()
            })
            .collect()
    };
    let cell = size / SIDE as f64;
    let slots = RowStrips::build(
        stage,
        v(at[0] - size / 2.0, at[1] + size / 2.0),
        (cell, cell),
        &colors_for(0),
    )?;
    let cover = stage.arena_mut().add(
        Rectangle::new()
            .width(size + 0.3)
            .height(size + 0.3)
            .corner_radius(0.12)
            .style(line_style(Role::card(), 4.0))
            .build()?
            .moved_to(at),
    );
    let heading = kit.text(
        stage,
        &format!("sticker album: {} slots", thousands(HOOK_PIECES)),
        32.0,
        Palette::ink(),
        v(at[0], at[1] + size / 2.0 + 0.5),
    )?;

    kit.say(stage, "01_7")?;
    crossfade(stage, &[slots.group, cover, heading], 0.8)?;
    // random stickers trickle in as the question is asked; the album fills
    // on the average curve, k(1 − (1 − 1/k)^draws), all of it at the average
    let total = album_mean_floor(HOOK_PIECES);
    let n = stage.add_value_tracker(0.0);
    let driver = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(driver)?;
    let mut shown_lit = 0;
    let strips = slots;
    stage.add_updater(
        driver,
        move |s, _| {
            let drawn = s.tracker_value(n).unwrap_or(0.0);
            let lit = if drawn >= total as f64 {
                k
            } else {
                (album_filled(HOOK_PIECES, drawn).floor() as usize).min(k - 1)
            };
            if lit != shown_lit {
                strips.recolor(s, &colors_for(lit));
                shown_lit = lit;
            }
        },
        false,
    )?;
    stage.play(tween(n, 400.0, 4.2)?)?;
    let q = predict_pause(kit, stage, v(3.6, 0.6))?;

    // 01_8: the count, on average
    let caption = kit.text(stage, "random pieces drawn", 28.0, Palette::dim(), v(3.9, 3.15))?;
    kit.live_count(stage, n, 64.0, Palette::ink(), v(3.9, 2.35))?;
    let average = kit.text_styled(stage, "on average", 28.0, Palette::dim(), v(3.9, 1.65), false, true)?;
    kit.say(stage, "01_8")?;
    play!(stage;
        fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
        fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.5),
    );
    let run = n
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(2.6),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(total as f64)?;
    stage.play(run)?;
    stage.clear_updaters(driver, false);
    play!(stage; fade_in(stage.arena_mut(), average, ORIGIN, 1.0)?.rt(0.4));
    // three counts side by side: packets a phone catches, on average
    let rows = [
        (
            "random pieces",
            format!(
                "≈ {}",
                thousands((album_mean(HOOK_PIECES) / 1000.0).round() as u64 * 1000)
            ),
            album_mean(HOOK_PIECES),
            Palette::dim(),
        ),
        (
            "carousel",
            format!("≈ {}", thousands((carousel_mean_caught() / 100.0).round() as u64 * 100)),
            carousel_mean_caught(),
            Palette::dim(),
        ),
        (
            "two extra packets",
            thousands(HOOK_PIECES + 2),
            f64::from(HOOK_PIECES + 2),
            Role::done(),
        ),
    ];
    let (x0, unit) = (1.0, 5.6 / album_mean(HOOK_PIECES));
    let header = kit.text_left(
        stage,
        "packets a phone catches, on average (model)",
        24.0,
        Palette::dim(),
        v(x0, 0.8),
    )?;
    let mut bars = vec![header];
    for (r, (name, value, amount, color)) in rows.iter().enumerate() {
        let y = 0.05 - 1.15 * r as f64;
        let bar = Rectangle::new()
            .width(amount * unit)
            .height(0.42)
            .style(flat(*color, 0.85))
            .build()?
            .moved_to(v(x0 + amount * unit / 2.0, y - 0.32));
        let bar = stage.arena_mut().add(bar);
        let tag = kit.text_left(stage, name, 28.0, Palette::ink(), v(x0, y + 0.2))?;
        let num = kit.text_left(stage, value, 28.0, *color, v(stage.get_right(tag)[0] + 0.3, y + 0.2))?;
        bars.extend([bar, tag, num]);
    }
    let panel = group_of(stage, &bars)?;
    play!(stage; fade_in(stage.arena_mut(), panel, ORIGIN, 1.0)?.rt(1.2).lag(0.15));
    // the storyboard's hold: let the three counts land
    stage.wait(2.6)?;
    Ok(())
}
