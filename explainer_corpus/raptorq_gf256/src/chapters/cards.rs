//! Chapter 03, packets are equations: four packets drawn as cards (03_1),
//! the same cards as a grid (03_2), elimination: card one ex-or card three
//! cancels A (03_3) and card two then gives D, and the rest fill in (03_4);
//! the phone as a bucket filling one slot per useful card (03_5), a fifth
//! card that is not useful (03_6–03_7), the fountain and the shared deck
//! (03_8), and RaptorQ's name card (03_9). The cards and every result are
//! `model.rs`'s worked example (oracle B-F3); the deck's mixes are the toy
//! deck (B-S4.toy_deck).

use explainer_kit::props::{Pose, Swarm, lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, play, thousands, v};
use fmn::animation::{indicate, replacement_transform};
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

use super::draw::{Card, OPLUS, Role, badge, card, card_width, crossfade, group_of, line_style, packet, phone};
use super::props::{BucketProp, GridProp};
use crate::model::{CARDS, D, DECK_PACKETS, FIFTH_CARD, PIECES, deck_mix, pieces_of, result};

/// A number from the facts registry (every number on screen is computed
/// or sourced there and checked against the oracle).
fn fact(key: &str) -> fmn::Result<String> {
    let facts = crate::facts::facts()
        .map_err(|e| fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(e.into_boxed_str()))))?;
    facts
        .get(key)
        .map(str::to_owned)
        .ok_or(fmn::Error::Scene(SceneError::InvalidConfig(
            "a fact the chapter shows is not registered",
        )))
}

/// The cards' height and left edge, and their rows.
const CARD_H: f64 = 0.9;
const CARD_X0: f64 = -6.75;
/// One grid pitch between rows, so the grid's rows line up with the cards.
const PITCH: f64 = 1.1;
const ROWS_Y: [f64; 4] = [1.55, 1.55 - PITCH, 1.55 - 2.0 * PITCH, 1.55 - 3.0 * PITCH];

/// Where a card of `mask` sits with its left edge at `x0`, height `h`.
fn left_aligned(mask: u32, with_result: bool, h: f64, x0: f64, y: f64) -> Vec3 {
    let w = card_width(pieces_of(mask).len(), with_result, h);
    v(x0 + w / 2.0, y)
}

/// A prepared animation moving `m` to `to` over `rt` seconds.
fn move_to(stage: &mut Stage<'_>, m: Mob, to: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    stage.prepare(a)
}

pub struct Cards {
    pub kit: Kit,
}

impl SceneConstruct for Cards {
    fn name(&self) -> &str {
        "03_cards"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let cards = equations(&kit, stage)?;
        eliminate(&kit, stage, &cards)?;
        bucket(&kit, stage)?;
        fountain(&kit, stage)?;
        name_card(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// 03_1–03_2: four packets as cards, then the grid beside them.
fn equations(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Card>> {
    let cards: Vec<Card> = CARDS
        .iter()
        .zip(ROWS_Y)
        .map(|(&c, y)| {
            card(
                kit,
                stage,
                c,
                Some(result(c)),
                CARD_H,
                left_aligned(c, true, CARD_H, CARD_X0, y),
            )
        })
        .collect::<fmn::Result<_>>()?;
    // a packet arrives and opens into the first card
    let first = &cards[0];
    let glyph = stage.arena_mut().add(packet(2.2, Role::card()).moved_to(v(0.5, 0.3)));
    let tag = kit.text(stage, "a packet", 40.0, Palette::ink(), v(0.5, 1.5))?;
    kit.say(stage, "03_1")?;
    play!(stage;
        fade_in(stage.arena_mut(), glyph, v(0.6, 0.0), 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.7),
    );
    stage.wait(0.8)?;
    let to = stage.get_center(first.back);
    let open = glyph
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.0),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?
        .set_width(first.width, true)?
        .set_height(first.height, true)?
        .set_opacity(0.0)?;
    play!(stage;
        open,
        fade_out(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), first.group, ORIGIN, 1.0)?.rt(1.0),
    );
    stage.remove_from_scene(glyph);
    stage.wait(1.2)?;
    let rest: Vec<Mob> = cards[1..].iter().map(|c| c.group).collect();
    let rest = group_of(stage, &rest)?;
    let keep = [first.group, rest];
    play!(stage; fade_in(stage.arena_mut(), rest, v(0.0, -0.2), 1.0)?.rt(1.4).lag(0.3));
    kit.hold(stage)?;

    // 03_2: the grid, a row per card
    let grid = GridProp::build(kit, stage, &CARDS, 4, v(1.6, ROWS_Y[0]), PITCH)?;
    // the headers and the rule before the results arrive first
    let in_rows = |m: &Mob| grid.rows.iter().any(|(d, r)| d.iter().any(|&(_, x)| x == *m) || r == m);
    let frame: Vec<Mob> = children(stage, grid.group)
        .into_iter()
        .filter(|m| !in_rows(m))
        .collect();
    kit.say(stage, "03_2")?;
    let heads = group_of(stage, &frame)?;
    play!(stage; fade_in(stage.arena_mut(), heads, v(0.0, -0.15), 1.0)?.rt(0.9).lag(0.1));
    for (r, (dots, res)) in grid.rows.iter().enumerate() {
        let mut members: Vec<Mob> = dots.iter().map(|&(_, d)| d).collect();
        members.push(*res);
        let row = group_of(stage, &members)?;
        play!(stage;
            indicate(stage.arena_mut(), cards[r].back, 1.04, Some([0.95, 0.9, 0.81]))?.rt(0.8),
            fade_in(stage.arena_mut(), row, v(-0.2, 0.0), 1.0)?.rt(0.8).lag(0.15),
        );
    }
    let chunk = precisely(
        kit,
        stage,
        "each piece is really a chunk, about a kilobyte; every operation runs across all its bytes at once",
    )?;
    play!(stage; fade_in(stage.arena_mut(), chunk, v(0.0, 0.2), 1.0)?.rt(0.6));
    kit.hold(stage)?;
    // the grid makes way for the elimination
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in stage.scene().mobjects().to_vec() {
        if keep.contains(&m) {
            continue;
        }
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.6),
            ..PlayOverrides::default()
        },
    )?;
    Ok(cards)
}

/// One badge of `card` (the piece's).
fn badge_of(card: &Card, piece: usize) -> Mob {
    card.badges
        .iter()
        .find(|&&(p, _)| p == piece)
        .expect("the card uses the piece")
        .1
}

/// Two cards stacked at `at` ex-or into `target`: the shared badges flash
/// and cancel, the others slide into the target card, the results combine.
fn combine(stage: &mut Stage<'_>, top: &Card, bottom: &Card, target: &Card, rt: f64) -> fmn::Result<()> {
    let shared: Vec<usize> = top
        .badges
        .iter()
        .filter(|(p, _)| bottom.badges.iter().any(|(q, _)| q == p))
        .map(|&(p, _)| p)
        .collect();
    // the cancelling pairs meet in the middle and vanish
    let mut meet: Vec<Box<dyn Animation>> = Vec::new();
    for &p in &shared {
        let (a, b) = (badge_of(top, p), badge_of(bottom, p));
        let (ca, cb) = (stage.get_center(a), stage.get_center(b));
        let mid = v((ca[0] + cb[0]) / 2.0, (ca[1] + cb[1]) / 2.0);
        meet.push(move_to(stage, a, mid, rt * 0.5)?);
        meet.push(move_to(stage, b, mid, rt * 0.5)?);
    }
    stage.play_prepared(meet)?;
    let mut vanish: Vec<Box<dyn Animation>> = Vec::new();
    for &p in &shared {
        for m in [badge_of(top, p), badge_of(bottom, p)] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.6)?;
            vanish.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared_with(
        vanish,
        PlayOverrides {
            run_time: Some(rt * 0.4),
            ..PlayOverrides::default()
        },
    )?;
    // what is left becomes the target card
    let mut into: Vec<Box<dyn Animation>> = Vec::new();
    for &(p, m) in top.badges.iter().chain(&bottom.badges) {
        if shared.contains(&p) {
            continue;
        }
        into.push(stage.prepare(replacement_transform(m, badge_of(target, p)))?);
    }
    into.push(stage.prepare(replacement_transform(top.back, target.back))?);
    let a = fade_out(stage.arena_mut(), bottom.back, ORIGIN, 1.0)?;
    into.push(stage.prepare(a)?);
    for &o in top.ops.iter().chain(&bottom.ops) {
        let a = fade_out(stage.arena_mut(), o, ORIGIN, 1.0)?;
        into.push(stage.prepare(a)?);
    }
    for &o in &target.ops {
        let a = fade_in(stage.arena_mut(), o, ORIGIN, 1.0)?;
        into.push(stage.prepare(a)?);
    }
    if let (Some((te, tn)), Some((be, bn)), Some((ge, gn))) = (top.result, bottom.result, target.result) {
        into.push(stage.prepare(replacement_transform(te, ge))?);
        into.push(stage.prepare(replacement_transform(tn, gn))?);
        for m in [be, bn] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            into.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared_with(
        into,
        PlayOverrides {
            run_time: Some(rt * 0.6),
            ..PlayOverrides::default()
        },
    )?;
    // the target's parts are on stage one by one: one card again
    stage.add_to_scene(target.group)?;
    Ok(())
}

/// A copy of `c`'s card (same mask, same result) at `at`.
fn copy_of(kit: &Kit, stage: &mut Stage<'_>, c: u32, at: Vec3) -> fmn::Result<Card> {
    card(kit, stage, c, Some(result(c)), CARD_H, at)
}

/// 03_3–03_4: card one ex-or card three; card two then gives D; the rest.
fn eliminate(kit: &Kit, stage: &mut Stage<'_>, cards: &[Card]) -> fmn::Result<()> {
    let x0 = 0.9;
    let (y1, y2, y3) = (ROWS_Y[0], ROWS_Y[1], ROWS_Y[2] - 0.35);
    // copies of cards one and three, lifted from the column to the right
    let (p1, p3) = (stage_pos(&cards[0], stage), stage_pos(&cards[2], stage));
    let one = copy_of(kit, stage, CARDS[0], p1)?;
    let three = copy_of(kit, stage, CARDS[2], p3)?;
    let bc = card(
        kit,
        stage,
        FIFTH_CARD,
        Some(result(FIFTH_CARD)),
        CARD_H,
        left_aligned(FIFTH_CARD, true, CARD_H, x0, y3),
    )?;
    kit.say(stage, "03_3")?;
    // the copies lift off the originals
    stage.add_to_scene(one.group)?;
    stage.add_to_scene(three.group)?;
    let lifts = vec![
        move_to(stage, one.group, left_aligned(CARDS[0], true, CARD_H, x0, y1), 1.0)?,
        move_to(stage, three.group, left_aligned(CARDS[2], true, CARD_H, x0, y2), 1.0)?,
    ];
    stage.play_prepared(lifts)?;
    let plus = kit.tex(stage, OPLUS, 56.0, Role::ex_or(), &[], v(x0 - 0.5, y2))?;
    play!(stage; fade_in(stage.arena_mut(), plus, ORIGIN, 1.0)?.rt(0.4));
    stage.wait(0.6)?;
    combine(stage, &one, &three, &bc, 2.8)?;
    play!(stage; fade_out(stage.arena_mut(), plus, ORIGIN, 1.0)?.rt(0.3));
    // the algebra underneath: the A's cancel, two ex-or six is four
    let (r1, r3, r13) = (result(CARDS[0]), result(CARDS[2]), result(FIFTH_CARD));
    let raw = format!(r"A \oplus C \oplus A \oplus B = {r1} \oplus {r3}");
    let done = format!(r"B \oplus C = {r13}");
    let color = [(r"\oplus", Role::ex_or())];
    let (m_raw, s_raw) = kit.tex_spans(stage, &raw, 46.0, Palette::ink(), &color, v(3.7, -2.5))?;
    let (m_done, s_done) = kit.tex_spans(stage, &done, 46.0, Palette::ink(), &color, v(3.7, -2.5))?;
    play!(stage; write(stage.arena(), m_raw).rt(1.0));
    stage.wait(0.5)?;
    let morph = TransformMatchingTex::new(stage.arena_mut(), m_raw, m_done, &s_raw, &s_done)?;
    stage.play(morph.rt(1.6))?;
    kit.hold(stage)?;

    // 03_4: into card two: D is two; then everything else
    let p2 = stage_pos(&cards[1], stage);
    let two = copy_of(kit, stage, CARDS[1], p2)?;
    let d_card = card(
        kit,
        stage,
        D,
        Some(result(CARDS[1]) ^ result(FIFTH_CARD)),
        CARD_H,
        left_aligned(D, true, CARD_H, x0, y3),
    )?;
    kit.say(stage, "03_4")?;
    stage.add_to_scene(two.group)?;
    let lifts = vec![
        move_to(
            stage,
            two.group,
            left_aligned(CARDS[1], true, CARD_H, x0, y2 - 0.55),
            0.9,
        )?,
        move_to(
            stage,
            bc.group,
            left_aligned(FIFTH_CARD, true, CARD_H, x0, y3 - 0.35),
            0.9,
        )?,
    ];
    let gone = fade_out(stage.arena_mut(), m_done, ORIGIN, 1.0)?;
    let mut lifts = lifts;
    lifts.push(stage.prepare(gone)?);
    stage.play_prepared(lifts)?;
    combine(stage, &two, &bc, &d_card, 1.8)?;
    // and so on: A, B, C, D fill in
    let knowns: Vec<Mob> = (0..4)
        .map(|p| {
            let at = v(0.7 + 1.55 * p as f64, -3.3);
            let b = badge(kit, stage, p, 0.26, v(at[0] - 0.35, at[1]))?;
            let eq = kit.tex(
                stage,
                &format!("= {}", PIECES[p]),
                44.0,
                Palette::ink(),
                &[],
                v(at[0] + 0.3, at[1]),
            )?;
            group_of(stage, &[b, eq])
        })
        .collect::<fmn::Result<_>>()?;
    // D first (just found), then the rest fast
    let order = [3usize, 0, 2, 1];
    let mut fill: Vec<Box<dyn Animation>> = Vec::new();
    for p in order {
        let a = fade_in(stage.arena_mut(), knowns[p], v(0.0, 0.15), 1.0)?;
        fill.push(stage.prepare(a)?);
    }
    let all = AnimationGroup::with_lag_ratio(stage.arena_mut(), fill, 0.35)?;
    stage.play(all.rt(1.6))?;
    kit.hold(stage)?;
    Ok(())
}

/// Where a card on screen now sits (its back's centre).
fn stage_pos(c: &Card, stage: &Stage<'_>) -> Vec3 {
    stage.get_center(c.back)
}

/// 03_5–03_7: the bucket fills one slot per useful card; a fifth card
/// bounces off.
fn bucket(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let b = BucketProp::build(stage, CARDS.len(), v(-0.2, -0.75), 2.6, 4.0)?;
    let phone_mark = stage
        .arena_mut()
        .add(phone(Palette::dim(), 0.7).moved_to(v(-0.2, -3.25)));
    let label = kit.text(stage, "the phone", 28.0, Palette::dim(), v(0.75, -3.25))?;
    stage.move_to(label, v(0.25, -3.25), LEFT);
    // the cards arrive in another order: the phone doesn't care
    let arrival = [2usize, 0, 3, 1];
    let starts = [v(-5.0, 2.6), v(5.2, 2.4), v(-5.4, 0.8), v(5.0, 0.6)];
    let flying: Vec<Card> = arrival
        .iter()
        .zip(starts)
        .map(|(&k, at)| card(kit, stage, CARDS[k], Some(result(CARDS[k])), CARD_H, at))
        .collect::<fmn::Result<_>>()?;
    kit.say(stage, "03_5")?;
    crossfade(stage, &[b.group, phone_mark, label], 0.9)?;
    let flying_group: Vec<Mob> = flying.iter().map(|c| c.group).collect();
    let fg = group_of(stage, &flying_group)?;
    play!(stage; fade_in(stage.arena_mut(), fg, ORIGIN, 1.0)?.rt(0.8).lag(0.2));
    let k = CARDS.len();
    let mut count = kit.text(stage, &format!("useful: 0 of {k}"), 30.0, Role::ex_or(), v(3.6, -0.75))?;
    stage.add_to_scene(count)?;
    for (i, c) in flying.iter().enumerate() {
        let water = b.water(stage, i, Role::ex_or())?;
        let drop = c
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.9),
                ..AnimateArgs::default()
            })?
            .move_to(b.slot_centre(i), ORIGIN)?
            .scale(0.45)?
            .set_opacity(0.0)?;
        play!(stage; drop);
        stage.remove_from_scene(c.group);
        let next = kit.text(
            stage,
            &format!("useful: {} of {k}", i + 1),
            30.0,
            Role::ex_or(),
            v(3.6, -0.75),
        )?;
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), water, DOWN, None)?.rt(0.5),
            replacement_transform(count, next).rt(0.4),
        );
        count = next;
    }
    let useful = kit.text(
        stage,
        "useful: it tells you something you couldn't already work out",
        28.0,
        Palette::ink(),
        v(0.0, 3.3),
    )?;
    play!(stage; fade_in(stage.arena_mut(), useful, ORIGIN, 1.0)?.rt(0.8));
    kit.hold(stage)?;

    // 03_6: a fifth card over the full bucket
    let fifth = card(kit, stage, FIFTH_CARD, Some(result(FIFTH_CARD)), CARD_H, v(-0.2, 2.15))?;
    kit.say(stage, "03_6")?;
    play!(stage;
        fade_out(stage.arena_mut(), useful, ORIGIN, 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), fifth.group, v(0.0, 0.4), 1.0)?.rt(0.8),
    );
    let q = predict_pause(kit, stage, v(3.6, 2.15))?;

    // 03_7: no; cards one and three already make it
    kit.say(stage, "03_7")?;
    let rim = b.rim();
    let bounce = fifth
        .group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.45),
            rate_func: Some(fmn::core::rate::rush_into),
            ..AnimateArgs::default()
        })?
        .move_to(v(-0.2, rim + 0.45), ORIGIN)?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3), bounce);
    let off = fifth
        .group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.9),
            rate_func: Some(fmn::core::rate::rush_from),
            ..AnimateArgs::default()
        })?
        .move_to(v(-4.4, 2.6), ORIGIN)?
        .rotate(0.35)?;
    play!(stage; off);
    let made = kit.tex(
        stage,
        &format!(
            r"\text{{card 1}} \oplus \text{{card 3}}:\ {} \oplus {} = {}",
            result(CARDS[0]),
            result(CARDS[2]),
            result(FIFTH_CARD)
        ),
        44.0,
        Palette::ink(),
        &[(r"\oplus", Role::ex_or())],
        v(-4.0, 0.9),
    )?;
    play!(stage; write(stage.arena(), made).rt(1.0));
    let raid = lower_third(
        kit,
        stage,
        "for programmers:",
        "RAID-5 parity is one of these cards: the parity block is the ex-or of the data blocks",
    )?;
    play!(stage; fade_in(stage.arena_mut(), raid, v(0.0, 0.2), 1.0)?.rt(0.6));
    kit.hold(stage)?;
    Ok(())
}

/// 03_8: endless fresh mixes, like water from a fountain; both sides own
/// the same deck, so a packet's number names its mix.
fn fountain(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    // the fountain: cards arcing out of a spout and falling away, endlessly
    let spout = v(-4.0, -2.1);
    let bowl = |dx: f64| {
        v(
            spout[0] + dx,
            spout[1] - 0.45 - 0.5 * (1.0 - (dx / 2.6).powi(2)).max(0.0).sqrt(),
        )
    };
    let basin = stage.arena_mut().add(v_group([
        Polygon::polyline((0..=24).map(|k| bowl(-2.6 + 5.2 * f64::from(k) / 24.0)))
            .style(line_style(Role::card(), 5.0))
            .build(),
        Line::new(v(spout[0] - 0.12, spout[1] - 0.95), v(spout[0] - 0.12, spout[1]))
            .style(line_style(Role::card(), 5.0))
            .build()?,
        Line::new(v(spout[0] + 0.12, spout[1] - 0.95), v(spout[0] + 0.12, spout[1]))
            .style(line_style(Role::card(), 5.0))
            .build()?,
    ]));
    let n = 28;
    let poses: Vec<Pose> = (0..n)
        .map(|_| Pose {
            x: spout[0],
            y: spout[1],
            heading: 0.0,
        })
        .collect();
    let jets = Swarm::build(stage, &packet(0.22, Role::card()), &poses, None)?;
    let caption = kit.text(
        stage,
        "fountain codes: endless fresh mixes",
        32.0,
        Palette::ink(),
        v(-4.0, 3.3),
    )?;
    kit.say(stage, "03_8")?;
    crossfade(stage, &[basin, caption], 0.8)?;
    stage.add_to_scene(jets.group)?;
    let mut t = 0.0;
    jets.drive(stage, poses, move |dt, ps| {
        t += dt;
        // each card rides its own arc, 2.1 s from the spout back to its
        // height, the cards evenly staggered round the cycle
        let (vy, g) = (4.6, 4.4);
        let period = 2.0 * vy / g;
        for (i, p) in ps.iter_mut().enumerate() {
            let a = (t / period + i as f64 / n as f64).fract() * period;
            let lean = ((i * 7 % 13) as f64 / 12.0 - 0.5) * 2.4;
            p.x = spout[0] + lean * a;
            p.y = spout[1] + vy * a - 0.5 * g * a * a;
            p.heading = 0.3 * (vy - g * a).atan2(lean);
        }
    })?;
    stage.wait(3.4)?;
    // the shared deck: the same recipe on both sides
    let sender_at = v(0.9, 1.5);
    let phone_at = v(4.7, 1.5);
    let sender = kit.text(stage, "sender", 30.0, Palette::dim(), v(sender_at[0], 2.75))?;
    let receiver = kit.text(stage, "phone", 30.0, Palette::dim(), v(phone_at[0], 2.75))?;
    let deck = |stage: &mut Stage<'_>, at: Vec3| -> fmn::Result<Mob> {
        let mut layers = Vec::new();
        for k in 0..5 {
            let r = Rectangle::new()
                .width(1.0)
                .height(0.7)
                .corner_radius(0.08)
                .style(
                    Style::default()
                        .fill(super::draw::card_fill(), 1.0)
                        .stroke(Role::card(), 2.5, 1.0),
                )
                .build()?
                .moved_to(v(at[0] + 0.05 * k as f64, at[1] + 0.05 * k as f64));
            layers.push(stage.arena_mut().add(r));
        }
        group_of(stage, &layers)
    };
    let d1 = deck(stage, sender_at)?;
    let d2 = deck(stage, phone_at)?;
    play!(stage;
        fade_in(stage.arena_mut(), sender, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), receiver, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), d1, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), d2, ORIGIN, 1.0)?.rt(0.6),
    );
    // three packet numbers, each naming the same mix on both sides
    for (k, &number) in DECK_PACKETS.iter().enumerate() {
        let y = 0.25 - 1.3 * k as f64;
        let mix = deck_mix(number);
        let tag = kit.text(
            stage,
            &format!("packet #{}", thousands(number)),
            28.0,
            Palette::ink(),
            v((sender_at[0] + phone_at[0]) / 2.0, y + 0.66),
        )?;
        let left = card(kit, stage, mix, None, 0.72, v(sender_at[0], y))?;
        let right = card(kit, stage, mix, None, 0.72, v(phone_at[0], y))?;
        play!(stage;
            fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), left.group, v(0.0, 0.3), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), right.group, v(0.0, 0.3), 1.0)?.rt(0.6),
        );
    }
    let note = lower_third(
        kit,
        stage,
        "precisely:",
        "the deck is a fixed recipe written into the standard (RFC 6330's packet number, the ESI, is the seed); \
         the block size travels in a header",
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
    kit.hold(stage)?;
    Ok(())
}

/// 03_9: RaptorQ, spelled once, and its standard.
fn name_card(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let name = kit.text_styled(stage, "RaptorQ", 120.0, Palette::ink(), v(0.0, 0.5), true, false)?;
    let line = kit.text(
        stage,
        &format!(
            "RFC {} · IETF Proposed Standard · August {}",
            fact("B-S1.sourced.rfc6330_number")?,
            fact("B-S1.sourced.rfc6330_year")?
        ),
        34.0,
        Palette::dim(),
        v(0.0, -0.75),
    )?;
    let rule = Line::new(v(-3.2, -0.15), v(3.2, -0.15))
        .style(line_style(Palette::faint(), 2.0))
        .build()?;
    let rule = stage.arena_mut().add(rule);
    kit.say(stage, "03_9")?;
    crossfade(stage, &[], 0.5)?;
    play!(stage; write(stage.arena(), name).rt(1.4), show_creation(rule).rt(1.0));
    play!(stage; fade_in(stage.arena_mut(), line, v(0.0, 0.15), 1.0)?.rt(0.8));
    Ok(())
}
