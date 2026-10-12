//! Video B's look, shared by every chapter: the colour roles of the
//! storyboard's table (one colour per idea, the whole video), the flat-fill
//! style rules (one flat fill per mobject, no outlines on big moving shapes;
//! `explainer_kit::kit`), the pieces' shape badges, cards (a packet's
//! equation), and the hook's phones, packets and tower.

use std::f64::consts::TAU;

use explainer_kit::{Kit, Palette, hex, v};
use fmn::prelude::*;

use crate::model::{PIECE_NAMES, pieces_of};

/// The storyboard's colour roles (§3 Color roles).
pub struct Role;

impl Role {
    /// Ex-or, GF(2), ex-or cards: cool blue.
    pub fn ex_or() -> Srgb {
        Palette::blue()
    }
    /// The byte field and byte cards: gold (the hero).
    pub fn byte() -> Srgb {
        Palette::hero()
    }
    /// Lost packets: red, shown dimmed.
    pub fn lost() -> Srgb {
        Palette::red()
    }
    /// The lit corners, "what we can already make": teal, translucent.
    pub fn lit() -> Srgb {
        Palette::teal()
    }
    /// The fold polynomial 0x11D: orange.
    pub fn fold() -> Srgb {
        Palette::gold()
    }
    /// Wrap-around (Z/256): desaturated purple.
    pub fn wrap() -> Srgb {
        Palette::purple()
    }
    /// Packets and cards: warm white.
    pub fn card() -> Srgb {
        hex("#F2E6CF")
    }
    /// A phone that holds the whole picture.
    pub fn done() -> Srgb {
        Palette::green()
    }
    /// A phone still waiting.
    pub fn waiting() -> Srgb {
        hex("#56607A")
    }
}

/// A card's paper: a warm dark panel, clearly lighter than the background
/// (the owner's note on 02–04: thin outlines on near-black read as plain).
pub fn card_fill() -> Srgb {
    hex("#2F2A24")
}

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// An empty parent holding mobjects that are already in the arena (a group
/// of detached `VMobject`s is built with `v_group` instead).
pub fn group_of(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// Fade out everything on screen while `incoming` fades in: one move in
/// place of a wipe and then a fade-in. The old picture is mostly gone
/// before the new one shows, so the two never read as one.
pub fn crossfade(stage: &mut Stage<'_>, incoming: &[Mob], rt: f64) -> fmn::Result<()> {
    let outgoing: Vec<Mob> = stage.scene().mobjects().to_vec();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for m in outgoing {
        stage.clear_updaters(m, true);
        let mut a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        a.update_rate_info(None, Some(RateFunc::smooth().squish(0.0, 0.6)), None);
        anims.push(stage.prepare(a)?);
    }
    for &m in incoming {
        let mut a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        a.update_rate_info(None, Some(RateFunc::smooth().squish(0.4, 1.0)), None);
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        anims,
        PlayOverrides {
            run_time: Some(rt),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}

/// The vertices of a regular `n`-gon of circumradius `r` round `at`, the
/// first at angle `start`.
fn ngon(n: usize, r: f64, start: f64, at: Vec3) -> Vec<Vec3> {
    (0..n)
        .map(|k| {
            let a = start + TAU * k as f64 / n as f64;
            v(at[0] + r * a.cos(), at[1] + r * a.sin())
        })
        .collect()
}

/// A piece's badge shape (no letter), `r` its size, centred at `at`:
/// circle A, triangle B, square C, diamond D, hexagon E. The shapes are
/// sized to look equally big, not to share a circumradius.
pub fn badge_shape(piece: usize, r: f64, style: Style, at: Vec3) -> VMobject {
    let q = TAU / 4.0;
    match piece {
        0 => Circle::new().radius(r).style(style).build().moved_to(at),
        1 => Polygon::new(ngon(3, r * 1.25, q, v(at[0], at[1] - r * 0.16)))
            .style(style)
            .build(),
        2 => Polygon::new(ngon(4, r * 1.2, q / 2.0, at)).style(style).build(),
        3 => Polygon::new(ngon(4, r * 1.22, q, at)).style(style).build(),
        4 => Polygon::new(ngon(6, r * 1.06, 0.0, at)).style(style).build(),
        _ => panic!("Video B has five pieces, A to E"),
    }
}

/// A piece's badge: its ink-white shape and its letter, centred at `at`.
pub fn badge(kit: &Kit, stage: &mut Stage<'_>, piece: usize, r: f64, at: Vec3) -> fmn::Result<Mob> {
    let ink = Palette::ink();
    let shape = badge_shape(
        piece,
        r,
        Style::default().fill(ink, 0.22).stroke(ink, (r * 9.0).max(1.5), 1.0),
        at,
    );
    let shape = stage.arena_mut().add(shape);
    let letter_at = if piece == 1 { v(at[0], at[1] - r * 0.12) } else { at };
    let letter = kit.text(stage, PIECE_NAMES[piece], r * 95.0, ink, letter_at)?;
    group_of(stage, &[shape, letter])
}

/// A card on screen: a packet's equation, its pieces' badges joined by ⊕,
/// then = and its result.
pub struct Card {
    pub group: Mob,
    pub back: Mob,
    /// One badge per piece the card uses, in piece order, with the piece.
    pub badges: Vec<(usize, Mob)>,
    /// The ⊕ signs between them.
    pub ops: Vec<Mob>,
    /// "=" and the result, when the card shows one.
    pub result: Option<(Mob, Mob)>,
    pub width: f64,
    pub height: f64,
}

/// The ⊕ in the ex-or role: math from the bundled faces (a heavier sans
/// look, accepted at the glyph probe).
pub const OPLUS: &str = r"\oplus";

/// Width of a card for `pieces` badges, with or without a result.
pub fn card_width(pieces: usize, with_result: bool, h: f64) -> f64 {
    let slot = h * 0.95;
    let n = pieces as f64;
    h * 0.5 + n * slot + (n - 1.0).max(0.0) * h * 0.55 + if with_result { h * 1.45 } else { 0.0 }
}

/// A card of height `h` centred at `at`: the badges of `card`'s pieces (bit
/// i = piece i) joined by ⊕, and `= result` when given.
pub fn card(kit: &Kit, stage: &mut Stage<'_>, card: u32, result: Option<u8>, h: f64, at: Vec3) -> fmn::Result<Card> {
    let pieces = pieces_of(card);
    let w = card_width(pieces.len(), result.is_some(), h);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(h * 0.16)
        .style(
            Style::default()
                .fill(card_fill(), 0.96)
                .stroke(Role::card(), (h * 6.0).max(2.0), 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let r = h * 0.27;
    let slot = h * 0.95;
    let gap = h * 0.55;
    let mut x = at[0] - w / 2.0 + h * 0.25 + slot / 2.0;
    let (mut badges, mut ops) = (Vec::new(), Vec::new());
    let mut all = vec![back];
    for (k, &p) in pieces.iter().enumerate() {
        if k > 0 {
            let o = kit.tex(
                stage,
                OPLUS,
                h * 52.0,
                Role::ex_or(),
                &[],
                v(x - slot / 2.0 - gap / 2.0, at[1]),
            )?;
            ops.push(o);
            all.push(o);
        }
        let b = badge(kit, stage, p, r, v(x, at[1]))?;
        badges.push((p, b));
        all.push(b);
        x += slot + gap;
    }
    let result = match result {
        Some(value) => {
            let eq_x = x - gap - slot / 2.0 + h * 0.42;
            let eq = kit.tex(stage, "=", h * 52.0, Palette::dim(), &[], v(eq_x, at[1]))?;
            let num = kit.text(
                stage,
                &value.to_string(),
                h * 64.0,
                Palette::ink(),
                v(eq_x + h * 0.62, at[1]),
            )?;
            all.extend([eq, num]);
            Some((eq, num))
        }
        None => None,
    };
    let group = group_of(stage, &all)?;
    Ok(Card {
        group,
        back,
        badges,
        ops,
        result,
        width: w,
        height: h,
    })
}

/// A phone glyph pointing up, `h` tall, one flat fill (its screen is cut
/// from the body as a second, darker flat shape).
pub fn phone(color: Srgb, h: f64) -> VMobject {
    let w = h * 0.56;
    let body = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(h * 0.14)
        .style(flat(color, 1.0))
        .build()
        .expect("positive extents");
    let screen = Rectangle::new()
        .width(w * 0.74)
        .height(h * 0.68)
        .style(flat(hex("#0E1116"), 0.55))
        .build()
        .expect("positive extents")
        .moved_to(v(0.0, h * 0.04));
    v_group([body, screen])
}

/// A packet glyph: a small warm-white card, flat.
pub fn packet(w: f64, color: Srgb) -> VMobject {
    Rectangle::new()
        .width(w)
        .height(w * 0.68)
        .corner_radius(w * 0.12)
        .style(flat(color, 0.95))
        .build()
        .expect("positive extents")
}

/// A broadcast tower `h` tall standing on `base`: two legs, cross braces, a
/// mast; its tip is at `tower_tip(base, h)`.
pub fn tower(h: f64, base: Vec3, color: Srgb) -> fmn::Result<VMobject> {
    let half = h * 0.22;
    let top = v(base[0], base[1] + h * 0.82);
    let (l, r) = (v(base[0] - half, base[1]), v(base[0] + half, base[1]));
    let s = line_style(color, 3.5);
    let mut parts = vec![
        Line::new(l, top).style(s).build()?,
        Line::new(r, top).style(s).build()?,
        Line::new(top, tower_tip(base, h)).style(s).build()?,
    ];
    // braces: zig-zag between the legs
    let rungs = 4;
    let at = |k: usize, side: f64| {
        let t = k as f64 / rungs as f64;
        v(base[0] + side * half * (1.0 - t), base[1] + t * h * 0.82)
    };
    for k in 0..rungs {
        parts.push(
            Line::new(at(k, -1.0), at(k + 1, 1.0))
                .style(line_style(color, 2.0))
                .build()?,
        );
        parts.push(
            Line::new(at(k, 1.0), at(k + 1, -1.0))
                .style(line_style(color, 2.0))
                .build()?,
        );
    }
    parts.push(
        Dot::new()
            .point(tower_tip(base, h))
            .radius(h * 0.03)
            .color(color)
            .build(),
    );
    Ok(v_group(parts))
}

/// The tip of [`tower`].
pub fn tower_tip(base: Vec3, h: f64) -> Vec3 {
    v(base[0], base[1] + h)
}

/// Radio waves leaving the tip: `n` concentric arcs opening right and left.
pub fn waves(tip: Vec3, r0: f64, step: f64, n: usize, color: Srgb) -> fmn::Result<VMobject> {
    let mut arcs = Vec::new();
    for k in 0..n {
        let r = r0 + step * k as f64;
        let opacity = 0.9 - 0.18 * k as f64;
        for start in [-0.6f64, std::f64::consts::PI - 0.6] {
            arcs.push(
                Arc::new()
                    .radius(r)
                    .start_angle(start)
                    .angle(1.2)
                    .arc_center(tip)
                    .style(Style::default().fill(color, 0.0).stroke(color, 3.0, opacity.max(0.2)))
                    .build()?,
            );
        }
    }
    Ok(v_group(arcs))
}

/// A byte card on screen: a [`Card`] whose pieces carry byte amounts.
pub struct ByteCard {
    pub card: Card,
    /// The amounts shown before their badges (an amount of 1 is left out),
    /// with their pieces.
    pub amounts: Vec<(usize, Mob)>,
}

/// A byte card of height `h` centred at `at`: each term's amount in byte
/// gold before its piece's badge (an amount of 1 is left out, as in
/// E ⊕ 2A), joined by ⊕, then `= result` when given; the paper's edge in
/// the byte role (13_1's random amounts, 16_4's E ⊕ 2A = 6).
pub fn card_with_amounts(
    kit: &Kit,
    stage: &mut Stage<'_>,
    terms: &[(usize, u8)],
    result: Option<u8>,
    h: f64,
    at: Vec3,
) -> fmn::Result<ByteCard> {
    let (r, slot, gap, pad) = (h * 0.27, h * 0.95, h * 0.55, h * 0.06);
    // the amounts first, to measure them
    let mut amounts = Vec::new();
    let mut widths = Vec::new();
    for &(p, a) in terms {
        if a == 1 {
            widths.push(0.0);
            continue;
        }
        let m = kit.text(stage, &a.to_string(), h * 64.0, Role::byte(), at)?;
        widths.push(stage.get_width(m) + pad);
        amounts.push((p, m));
    }
    // the result, measured too: "=" then the number, left-aligned
    let num = match result {
        Some(value) => Some(kit.text(stage, &value.to_string(), h * 64.0, Palette::ink(), at)?),
        None => None,
    };
    let num_w = num.map_or(0.0, |m| stage.get_width(m));
    let n = terms.len() as f64;
    let lead = h * 0.12;
    let w = h * 0.5
        + n * slot
        + (n - 1.0).max(0.0) * gap
        + widths
            .iter()
            .map(|&a| if a > 0.0 { a + lead } else { 0.0 })
            .sum::<f64>()
        + if result.is_some() { h * 0.9 + num_w } else { 0.0 };
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(h * 0.16)
        .style(
            Style::default()
                .fill(card_fill(), 0.96)
                .stroke(Role::byte(), (h * 6.0).max(2.0), 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let mut all = vec![back];
    let (mut badges, mut ops) = (Vec::new(), Vec::new());
    let mut x = at[0] - w / 2.0 + h * 0.25;
    let mut shown = amounts.iter();
    for (k, &(p, a)) in terms.iter().enumerate() {
        if k > 0 {
            let o = kit.tex(stage, OPLUS, h * 52.0, Role::ex_or(), &[], v(x + gap / 2.0, at[1]))?;
            ops.push(o);
            all.push(o);
            x += gap;
        }
        if a != 1 {
            let &(_, m) = shown.next().expect("one amount per term other than 1");
            x += lead;
            stage.move_to(m, v(x, at[1]), LEFT);
            all.push(m);
            x += widths[k];
        }
        let b = badge(kit, stage, p, r, v(x + slot / 2.0, at[1]))?;
        badges.push((p, b));
        all.push(b);
        x += slot;
    }
    let result = match num {
        Some(num) => {
            let eq_x = x + h * 0.32;
            let eq = kit.tex(stage, "=", h * 52.0, Palette::dim(), &[], v(eq_x, at[1]))?;
            stage.move_to(num, v(eq_x + h * 0.3, at[1]), LEFT);
            all.extend([eq, num]);
            Some((eq, num))
        }
        None => None,
    };
    let group = group_of(stage, &all)?;
    Ok(ByteCard {
        card: Card {
            group,
            back,
            badges,
            ops,
            result,
            width: w,
            height: h,
        },
        amounts,
    })
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;
    use crate::model::{CARDS, FIFTH_CARD, result};

    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    #[test]
    fn roles_are_distinct_and_cards_hold_their_pieces() {
        let roles = [
            Role::ex_or(),
            Role::byte(),
            Role::lost(),
            Role::lit(),
            Role::fold(),
            Role::wrap(),
            Role::card(),
            Role::done(),
            Role::waiting(),
        ];
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                assert_ne!(a, b);
            }
        }
        run(|stage| {
            let kit = Kit::new()?;
            for &c in CARDS.iter().chain([&FIFTH_CARD]) {
                let k = card(&kit, stage, c, Some(result(c)), 0.8, ORIGIN)?;
                let pieces: Vec<usize> = k.badges.iter().map(|&(p, _)| p).collect();
                assert_eq!(pieces, pieces_of(c));
                assert_eq!(k.ops.len(), pieces.len() - 1);
                // every badge and sign sits inside the card
                let (l, r) = (stage.get_left(k.back)[0], stage.get_right(k.back)[0]);
                for &m in k.badges.iter().map(|(_, m)| m).chain(&k.ops) {
                    assert!(stage.get_left(m)[0] > l && stage.get_right(m)[0] < r);
                }
                let (_, num) = k.result.expect("a result");
                assert!(stage.get_right(num)[0] < r, "the result fits");
                let w = stage.get_width(k.back);
                assert!((w - k.width).abs() < 0.02, "card width {w} vs {}", k.width);
            }
            for p in 0..5 {
                let b = badge(&kit, stage, p, 0.3, ORIGIN)?;
                assert!(stage.get_width(b) > 0.4 && stage.get_width(b) < 0.9, "badge {p}");
            }
            Ok(())
        });
    }

    #[test]
    #[should_panic(expected = "five pieces")]
    fn there_is_no_sixth_badge() {
        let _ = badge_shape(5, 0.3, flat(Palette::ink(), 1.0), ORIGIN);
    }
}
