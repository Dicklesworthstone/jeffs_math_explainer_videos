//! Chapters 04 and 05's devices (storyboard-draft, "04_the_goal" and
//! "05_an_experiment"): the sealed envelope that stands for the dealer's
//! method, which both chapters use and deliberately do not explain
//! (chapters 07 to 09 derive it, and 06_8 opens the envelope); the two fair
//! spins that go into it; outlines for a card frame on the trustee rail and
//! for anything else pointed at; and the shares the method deals, from the
//! model.
//!
//! The big vault, the rail and the strip are `draw.rs`'s, and the glows and
//! moves `two_keys.rs`'s, unchanged. Builders return detached mobjects, as
//! there.
//!
//! As in chapters 02 and 03, emphasis never restyles or scales typeset text
//! (Mail 6866): a card, a mark, the envelope or the strip is pointed at by
//! an outline of its own that comes and goes, and only shapes swell or
//! change color. A number never turns into another at the same place (Mail
//! 6873): a new number comes in where there was none.

use explainer_kit::{Kit, Palette, Timed, v};
use fmn::prelude::*;

use super::draw::{
    DOOR_PULSE, FLASH, FRAME_H, FRAME_W, Vault, bundle, dial_angle, disc, frame_at, line_style, panel, recolor_all,
    stroke,
};
use crate::model::{Dial, Recipe, SPINS, View};
use crate::palette::{self, spin};

// ------------------------------------------------------------- the shares

/// The worked example's shares at `xs`, each with its position: what the
/// sealed method hands those trustees (chapter 09 derives it).
pub fn dealt(dial: Dial, xs: &[u32]) -> Vec<(u32, u32)> {
    let recipe = Recipe::example();
    xs.iter().map(|&x| (x, recipe.share(dial, x))).collect()
}

/// The strip's view for the trustees at `xs` holding what they were dealt:
/// the secrets that some pair of fair spins deals exactly those shares for.
pub fn holding(dial: Dial, xs: &[u32]) -> View {
    View::Shares {
        points: dealt(dial, xs),
        spins: SPINS.len(),
    }
}

/// The strip's caption for the pair's goal (as chapter 01's): what two
/// shares should leave, before anything shows what they do leave.
pub fn wanted_pair(pair: [u32; 2]) -> String {
    format!("wanted, for trustees {} and {}", pair[0], pair[1])
}

/// The strip's caption for what the pair's shares do leave.
pub fn for_pair(pair: [u32; 2]) -> String {
    format!("for: trustees {} and {}", pair[0], pair[1])
}

// ------------------------------------------------------------ the envelope

/// The envelope, over the middle of the strip, and how far its flap's point
/// dips below the top edge.
pub const ENV_X: f64 = 3.65;
pub const ENV_Y: f64 = 3.2;
pub const ENV_W: f64 = 3.0;
pub const ENV_H: f64 = 1.1;
const FLAP_DIP: f64 = 0.38;
const FLAP_IN: f64 = 0.06;
const SEAL_R: f64 = 0.09;
/// Its two lines of words, under the flap, from its centre, and their sizes.
const NAME_DY: f64 = -0.12;
const NOTE_DY: f64 = -0.38;
const NAME_SIZE: f64 = 20.0;
const NOTE_SIZE: f64 = 16.0;
/// How far inside its left end what goes in disappears.
const MOUTH_IN: f64 = 0.3;

/// The envelope's words: what it holds, and that it stays shut until the
/// chapters that derive the method.
pub const METHOD: &str = "the dealer's method";
pub const SEALED: &str = "sealed until chapters 07 to 09";

/// The method's stand-in: a shut envelope with its flap and seal, named, and
/// marked as explained only later.
pub struct Envelope {
    parts: Vec<Mob>,
}

impl Envelope {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let body = panel(stage, palette::ghost(), 0.9, v(ENV_X, ENV_Y), ENV_W, ENV_H)?;
        let top = ENV_Y + ENV_H / 2.0 - FLAP_IN;
        let (left, right) = (ENV_X - ENV_W / 2.0 + FLAP_IN, ENV_X + ENV_W / 2.0 - FLAP_IN);
        let tip = v(ENV_X, ENV_Y + ENV_H / 2.0 - FLAP_DIP);
        let flap = stroke(stage, vec![v(left, top), tip, v(right, top)], Palette::dim(), 2.0);
        let seal = disc(stage, Palette::dim(), tip, SEAL_R);
        let name = kit.text(stage, METHOD, NAME_SIZE, Palette::ink(), v(ENV_X, ENV_Y + NAME_DY))?;
        let note = kit.text(stage, SEALED, NOTE_SIZE, Palette::dim(), v(ENV_X, ENV_Y + NOTE_DY))?;
        Ok(Self {
            parts: vec![body, flap, seal, name, note],
        })
    }

    /// Everything the envelope is drawn with.
    pub fn parts(&self) -> &[Mob] {
        &self.parts
    }

    /// Where what goes in disappears: just inside its left end.
    pub fn mouth() -> Vec3 {
        v(ENV_X - ENV_W / 2.0 + MOUTH_IN, ENV_Y)
    }

    /// An outline round the envelope, in `color`: how it is pointed at.
    /// Detached.
    pub fn halo(stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        outline(stage, v(ENV_X, ENV_Y), ENV_W + 2.0 * PAD, ENV_H + 2.0 * PAD, color)
    }
}

/// Fade-outs, prepared, of `mobs`, each sliding into the envelope's mouth
/// as it goes, over `rt` (live pieces stop updating first).
pub fn going_in(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let to = Envelope::mouth();
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(mobs.len());
    for &m in mobs {
        stage.clear_updaters(m, true);
        let c = stage.get_center(m);
        let a = fade_out(stage.arena_mut(), m, v(to[0] - c[0], to[1] - c[1]), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

// --------------------------------------------------------------- the spins

/// A fair spin's dial: a rim, a short tick per mark and a needle, in the
/// spins' yellow, with no numbers (what a spin lands on is the dealer's
/// alone). Its radius, where its ticks start and where its needle reaches,
/// as fractions of it.
pub const SPINNER_R: f64 = 0.28;
const SPINNER_TICK_IN: f64 = 0.78;
const SPINNER_NEEDLE_IN: f64 = 0.62;

/// A point `d` from `centre` at mark `k` of a `marks`-mark dial, clockwise
/// from the top (draw.rs's `dial_angle`).
fn around(centre: Vec3, marks: u32, k: f64, d: f64) -> Vec3 {
    let a = dial_angle(k, marks);
    v(centre[0] + d * a.cos(), centre[1] + d * a.sin())
}

/// One fair spin: its rim and ticks as one group, and a needle redrawn every
/// frame from one value tracker, the mark it points at, unwrapped.
pub struct Spinner {
    dial: Dial,
    group: Mob,
    /// The tracker: the mark the needle points at.
    pub needle: Mob,
    hand: Mob,
}

impl Spinner {
    pub fn build(stage: &mut Stage<'_>, dial: Dial, centre: Vec3) -> fmn::Result<Self> {
        let marks = dial.marks();
        let rim = Circle::new()
            .radius(SPINNER_R)
            .style(line_style(spin(), 3.0))
            .build()
            .moved_to(centre);
        let rim = stage.arena_mut().add(rim);
        let mut members = vec![rim];
        for k in dial.all() {
            let kf = f64::from(k);
            members.push(stroke(
                stage,
                vec![
                    around(centre, marks, kf, SPINNER_TICK_IN * SPINNER_R),
                    around(centre, marks, kf, SPINNER_R),
                ],
                spin(),
                2.0,
            ));
        }
        let group = bundle(stage, &members)?;
        let needle = stage.add_value_tracker(0.0);
        let tip = SPINNER_NEEDLE_IN * SPINNER_R;
        let hand = stage.always_redraw(move |s| {
            let k = s.tracker_value(needle).unwrap_or(0.0);
            let h = polyline(vec![centre, around(centre, marks, k, tip)])
                .style(line_style(spin(), 4.0))
                .build();
            s.add(h)
        });
        Ok(Self {
            dial,
            group,
            needle,
            hand,
        })
    }

    /// The dial and its needle, as they sit in the scene.
    pub fn parts(&self) -> [Mob; 2] {
        [self.group, self.hand]
    }

    /// The needle turning `turns` whole turns and on to mark `k`, slowing as
    /// it stops, over `rt`, prepared.
    pub fn spinning(&self, stage: &mut Stage<'_>, k: u32, turns: u32, rt: f64) -> fmn::Result<Box<dyn Animation>> {
        let stop = f64::from(k + turns * self.dial.marks());
        let a = self
            .needle
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                rate_func: Some(fmn::core::rate::rush_from),
                ..AnimateArgs::default()
            })?
            .set_value(stop)?;
        stage.prepare(a)
    }
}

/// Bring the spins in together (each needle once its dial is in), over
/// `rt`.
pub fn show_spins(stage: &mut Stage<'_>, spins: &[Spinner], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(spins.len());
    for s in spins {
        let a = fade_in(stage.arena_mut(), s.group, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    for s in spins {
        stage.add_to_scene(s.hand)?;
    }
    Ok(())
}

// --------------------------------------------------------------- emphasis

/// How far an outline stands outside what it points at, a card frame's
/// outline outside its frame, and an outline's line width (a frame's is 2).
pub const PAD: f64 = 0.08;
const FRAME_PAD: f64 = 0.06;
const OUTLINE_LINE: f64 = 3.0;

/// A rounded outline `w` × `h` round `at`, in `color`. Detached.
pub fn outline(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let o = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(line_style(color, OUTLINE_LINE))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(o))
}

/// An outline round trustee `t`'s card frame on the rail, in `color`: how a
/// card there is pointed at. Detached.
pub fn frame_halo(stage: &mut Stage<'_>, t: u32, color: Srgb) -> fmn::Result<Mob> {
    outline(
        stage,
        frame_at(t),
        FRAME_W + 2.0 * FRAME_PAD,
        FRAME_H + 2.0 * FRAME_PAD,
        color,
    )
}

/// Outlines round each of `group`'s card frames, in `color`. Detached.
pub fn frame_halos(stage: &mut Stage<'_>, group: &[u32], color: Srgb) -> fmn::Result<Vec<Mob>> {
    group.iter().map(|&t| frame_halo(stage, t, color)).collect()
}

/// `color` as an indication tint.
fn tint(color: Srgb) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}

/// A shape (never typeset text) growing to `scale` and back over `rt`,
/// tinted `color`, prepared.
pub fn swelling(
    stage: &mut Stage<'_>,
    shape: Mob,
    scale: f64,
    color: Srgb,
    rt: f64,
) -> fmn::Result<Box<dyn Animation>> {
    let a = fmn::animation::indicate(stage.arena_mut(), shape, scale, Some(tint(color)))?;
    stage.prepare(a.rt(rt))
}

/// The vault's door glowing in `color`, growing only as far as its pulse
/// (draw.rs keeps that clear of the lamp and the card frames), over `rt`.
pub fn door_glow(stage: &mut Stage<'_>, vault: &Vault, color: Srgb, rt: f64) -> fmn::Result<()> {
    let a = swelling(stage, vault.door, DOOR_PULSE, color, rt)?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

/// The alarm, as `Vault::alarm` (the lamp lights red, flashes as far and
/// goes dark again), but the flash tinted its own red: the indication
/// default yellow is the spins' role.
pub fn alarm(stage: &mut Stage<'_>, vault: &Vault) -> fmn::Result<()> {
    recolor_all(stage, &[(vault.lamp, palette::leak())], 0.15)?;
    let a = swelling(stage, vault.lamp, FLASH, palette::leak(), 0.5)?;
    stage.play_prepared(vec![a])?;
    recolor_all(stage, &[(vault.lamp, palette::ghost())], 0.3)
}

/// Each shape's fill turning `color` at opacity `o`, together, prepared,
/// over `rt` (a trustee dimmed, still in its role's color).
pub fn fills(stage: &mut Stage<'_>, changes: &[(Mob, Srgb, f64)], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(changes.len());
    for &(m, c, o) in changes {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(c), Some(o))?;
        anims.push(stage.prepare(a)?);
    }
    Ok(anims)
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS, STROKE_WIDTH_CONVERSION};

    use super::super::draw::{
        DOOR_H, DOOR_W, DOOR_X, DOOR_Y, RAIL_DX, RAIL_Y, STRIP_LEFT, STRIP_TITLE_Y, TRUSTEE_R, strip_right,
    };
    use super::*;
    use crate::model::{MARKS, OPENING_TRIO, PAIR, SECRET, TRUSTEES};

    /// Generous text sizes: 0.0055 units a point a character wide (a
    /// quarter over the corpus's measured 0.0045) and 0.0065 a point tall.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// The method deals the worked example's shares, 8, 3, 1, 2 and 6; three
    /// of them leave the secret alone, and the pair's two leave every mark.
    #[test]
    fn the_sealed_method_deals_the_examples_shares() {
        let dial = Dial::vault();
        let shares: Vec<u32> = dealt(dial, &TRUSTEES).iter().map(|&(_, y)| y).collect();
        assert_eq!(shares, vec![8, 3, 1, 2, 6]);
        assert!(dealt(dial, &TRUSTEES).iter().zip(&TRUSTEES).all(|(&(x, _), &t)| x == t));
        assert_eq!(holding(dial, &OPENING_TRIO).possible(dial), vec![SECRET]);
        assert_eq!(holding(dial, &PAIR).possible(dial), dial.all().collect::<Vec<_>>());
        assert_eq!(wanted_pair(PAIR), "wanted, for trustees 4 and 5");
        assert_eq!(for_pair(PAIR), "for: trustees 4 and 5");
    }

    /// The envelope sits over the strip's title, inside the frame, right of
    /// the door even as it pulses; its words fit under the flap's seal and
    /// inside its edges; its outline stays inside the frame.
    #[test]
    fn the_envelope_fits() {
        const LEFT: f64 = ENV_X - ENV_W / 2.0;
        const RIGHT: f64 = ENV_X + ENV_W / 2.0;
        const TOP: f64 = ENV_Y + ENV_H / 2.0;
        const BOTTOM: f64 = ENV_Y - ENV_H / 2.0;
        const {
            assert!(TOP + PAD < FRAME_Y_RADIUS - 0.1, "its outline inside the top edge");
            assert!(
                BOTTOM - PAD > STRIP_TITLE_Y + 0.0065 * 40.0 / 2.0 + 0.15,
                "over the strip's title and count"
            );
            assert!(
                LEFT - PAD > DOOR_X + DOOR_PULSE * DOOR_W / 2.0 + 0.3,
                "right of the pulsing door"
            );
            assert!(MOUTH_IN < ENV_W / 2.0, "the mouth inside the envelope");
            assert!(RIGHT + PAD < FRAME_X_RADIUS - 0.25, "inside the right edge");
        }
        assert!(LEFT > STRIP_LEFT && RIGHT < strip_right(MARKS), "over the strip");
        // the words: inside the edges, the name under the seal, the note over
        // the bottom
        for (text, size) in [(METHOD, NAME_SIZE), (SEALED, NOTE_SIZE)] {
            assert!(text_w(text, size) < ENV_W - 0.3, "{text}");
        }
        let seal_bottom = TOP - FLAP_DIP - SEAL_R;
        assert!(ENV_Y + NAME_DY + text_h(NAME_SIZE) / 2.0 < seal_bottom - 0.08);
        assert!(ENV_Y + NOTE_DY - text_h(NOTE_SIZE) / 2.0 > BOTTOM + 0.05);
        assert!(
            (ENV_Y + NAME_DY - text_h(NAME_SIZE) / 2.0) - (ENV_Y + NOTE_DY + text_h(NOTE_SIZE) / 2.0) > 0.08,
            "the two lines apart"
        );
        // the flap's arms stay over the name where they cross its ends
        let half = text_w(METHOD, NAME_SIZE) / 2.0;
        let arm = |x: f64| {
            (TOP - FLAP_IN) - (TOP - FLAP_IN - (TOP - FLAP_DIP)) * (1.0 - (x - ENV_X).abs() / (ENV_W / 2.0 - FLAP_IN))
        };
        assert!(arm(ENV_X + half) > ENV_Y + NAME_DY + text_h(NAME_SIZE) / 2.0 + 0.08);
    }

    /// A spin's ticks run inside its rim, its needle (4 wide) inside its
    /// ticks (2 wide), and the spins it stops on are marks.
    #[test]
    fn a_spin_dial_holds_together() {
        const {
            assert!(SPINNER_TICK_IN < 1.0, "the ticks inside the rim");
            assert!(
                SPINNER_NEEDLE_IN * SPINNER_R + 4.0 * STROKE_WIDTH_CONVERSION / 2.0
                    < SPINNER_TICK_IN * SPINNER_R - 2.0 * STROKE_WIDTH_CONVERSION / 2.0,
                "the needle inside the ticks"
            );
        }
        assert!(SPINS.iter().all(|&s| s < MARKS), "the spins are marks");
    }

    /// A card frame's outline stands clear of its frame's line, of the next
    /// frame's outline, of its trustee's disc and of the pulsing door.
    #[test]
    fn the_frame_halos_keep_apart() {
        let half = |line: f64| line * STROKE_WIDTH_CONVERSION / 2.0;
        assert!(FRAME_PAD - half(OUTLINE_LINE) > half(2.0) + 0.01, "off its frame");
        assert!(
            RAIL_DX - (FRAME_W + 2.0 * FRAME_PAD) > 2.0 * half(OUTLINE_LINE) + 0.08,
            "neighbours apart"
        );
        let f = frame_at(1);
        assert!(
            f[1] - FRAME_H / 2.0 - FRAME_PAD > RAIL_Y + TRUSTEE_R + 0.15,
            "over the disc"
        );
        assert!(
            f[1] + FRAME_H / 2.0 + FRAME_PAD < DOOR_Y - DOOR_PULSE * DOOR_H / 2.0 - 0.05,
            "under the pulsing door"
        );
        assert!(frame_at(1)[0] - FRAME_W / 2.0 - FRAME_PAD > -FRAME_X_RADIUS + 0.25);
    }
}
