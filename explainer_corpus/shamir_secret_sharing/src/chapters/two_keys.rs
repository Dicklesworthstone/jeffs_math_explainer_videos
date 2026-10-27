//! Chapters 02 and 03's devices (storyboard-draft, "02_cutting_it_up" and
//! "03_the_dial"): the two-trustee vault, a small door whose dial is the
//! vault's, with A and B under it, each with a card frame; a dial face that
//! can be drawn anywhere at any size (the small door's, the enlarged one,
//! the dealer's spare), whose needle turns one mark at a time; and the two
//! searches over what A could hold that say which secrets B's share leaves.
//!
//! The big vault, the trustee rail and the strip are `draw.rs`'s, unchanged.
//! Builders return detached mobjects, as there.
//!
//! What the strip shows for B is computed from the same `b` that B's card
//! prints: [`ordinary_view`] and [`dial_view`] give the strip's `View`, and
//! [`ordinary_secrets`] and [`if_a_held`] search A's holdings directly. The
//! scenes check that the strip lights exactly what the search finds.
//!
//! Emphasis never restyles or scales typeset text: root's native previews
//! of chapter 01 showed a label's glyphs doubled after an indicate and after
//! a recolor (Mail 6866). A mark, a card or a tag is pointed at by a ring or
//! an outline of its own that comes and goes ([`Face::halo`],
//! [`slot_halo`], [`text_halo`], [`glow`]); a changed card is a new card
//! crossfaded in; only shapes swell ([`swell_all`]).

use explainer_kit::{BACKGROUND, Kit, Palette, Timed, hex, tween, v};
use fmn::prelude::*;

use super::draw::{
    CARD_SIZE, DIAL_LINE, DIAL_R, DOOR_O, DOOR_PULSE, FLASH, FRAME_H, FRAME_W, FRAME_Y, HAND_LINE, HUB_R, MARK_LABEL_R,
    MARK_LABEL_SIZE, NEEDLE_R, RAIL_Y, Strip, TRUSTEE_R, bundle, disc, fail, frame, line_style, panel, stroke,
};
use crate::model::{Dial, View};
use crate::palette;

// ------------------------------------------------------------- B's share

/// The secrets B's ordinary piece `b` leaves: A's piece is a whole number,
/// 0 or more, and the two add up to a mark of the dial.
pub fn ordinary_secrets(dial: Dial, b: u32) -> Vec<u32> {
    let mut found: Vec<u32> = dial.all().map(|a| a + b).filter(|&s| s < dial.marks()).collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Each mark A could hold, with the secret it would make with B's dial
/// share `b`: A's mark turned on by `b`.
pub fn if_a_held(dial: Dial, b: u32) -> Vec<(u32, u32)> {
    dial.all().map(|a| (a, dial.add(a, b))).collect()
}

/// The secrets B's dial share `b` leaves, from [`if_a_held`].
pub fn dial_secrets(dial: Dial, b: u32) -> Vec<u32> {
    let mut found: Vec<u32> = if_a_held(dial, b).into_iter().map(|(_, s)| s).collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// The strip's view for B holding the ordinary piece `b`.
pub fn ordinary_view(b: u32) -> View {
    View::OrdinaryPiece(b)
}

/// The strip's view for B holding the dial share `b`. B's share is the
/// secret take away A's fair spin, which is the secret plus a fair spin
/// (taking away a fair spin is turning on by another fair spin), so it is
/// the one-spin recipe's share at position 1: the same view, with no
/// two-of-two variant of its own.
pub fn dial_view(b: u32) -> View {
    View::Shares {
        points: vec![(1, b)],
        spins: 1,
    }
}

/// An error unless the strip lights exactly `want`, the search's answer
/// for the share on B's card.
pub fn check_strip(strip: &Strip, want: &[u32], whose: &str) -> fmn::Result<()> {
    if strip.lit() == want {
        Ok(())
    } else {
        Err(fail(format!(
            "the strip {whose} lights {:?}, but the search finds {want:?}",
            strip.lit()
        )))
    }
}

/// The next top of the dial forward from unwrapped mark `at`: where a
/// needle turned on to 0 points, unwrapped.
pub fn next_top(at: i64, marks: u32) -> i64 {
    let m = i64::from(marks);
    (at.div_euclid(m) + 1) * m
}

/// The marks a needle at unwrapped mark `from` lands on, one click at a
/// time, turning `steps` marks (forward when positive, back when negative).
pub fn passed(dial: Dial, from: i64, steps: i64) -> Vec<u32> {
    (1..=steps.abs())
        .map(|i| dial.wrap(from + steps.signum() * i))
        .collect()
}

// ------------------------------------------------------------ a dial face

/// Where a mark's tick starts, its label sits and the needle's tip reaches,
/// as fractions of the face's radius: the vault's proportions (draw.rs).
const TICK_IN: f64 = super::draw::TICK_IN / DIAL_R;
const LABEL_IN: f64 = MARK_LABEL_R / DIAL_R;
const NEEDLE_IN: f64 = NEEDLE_R / DIAL_R;
const HUB_IN: f64 = HUB_R / DIAL_R;
/// A mark's label size per unit of radius (the vault's).
const LABEL_PER_R: f64 = MARK_LABEL_SIZE / DIAL_R;
/// A mark's halo's radius, as a fraction of the face's, and its line width:
/// round the label, inside the ticks, clear of the needle's tip and of the
/// neighbouring labels, on the smallest face too.
const HALO_IN: f64 = 0.135;
const RING_LINE: f64 = 2.0;
/// An arc's distance from the centre (outside the rim) as a fraction of the
/// radius, and its label's.
pub const ARC_IN: f64 = 1.13;
pub const ARC_LABEL_IN: f64 = 1.42;

/// A dial of `dial`'s marks drawn at `centre` with radius `r`: rim, ticks
/// and labels as one group, so it can zoom; and a needle redrawn every frame
/// from one value tracker, the mark it points at, unwrapped (16 points where
/// 5 does on eleven marks), so turning on past the top keeps going forward.
pub struct Face {
    pub centre: Vec3,
    pub r: f64,
    dial: Dial,
    group: Mob,
    /// The tracker: the mark the needle points at.
    pub needle: Mob,
    hand: Mob,
}

impl Face {
    /// A face whose rim is `rim` and whose needle starts at mark `start`.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        dial: Dial,
        centre: Vec3,
        r: f64,
        rim: Srgb,
        start: u32,
    ) -> fmn::Result<Self> {
        let marks = dial.marks();
        let at = move |k: f64, d: f64| -> Vec3 { point_on(centre, marks, k, d) };
        let ring = Circle::new()
            .radius(r)
            .style(line_style(rim, DIAL_LINE))
            .build()
            .moved_to(centre);
        let ring = stage.arena_mut().add(ring);
        let mut members = vec![ring];
        for k in dial.all() {
            let kf = f64::from(k);
            members.push(stroke(
                stage,
                vec![at(kf, TICK_IN * r), at(kf, r)],
                Palette::dim(),
                DIAL_LINE,
            ));
            members.push(kit.text(
                stage,
                &k.to_string(),
                LABEL_PER_R * r,
                Palette::ink(),
                at(kf, LABEL_IN * r),
            )?);
        }
        members.push(disc(stage, palette::secret(), centre, HUB_IN * r));
        let group = bundle(stage, &members)?;
        let needle = stage.add_value_tracker(f64::from(start));
        let tip = NEEDLE_IN * r;
        let hand = stage.always_redraw(move |s| {
            let k = s.tracker_value(needle).unwrap_or(0.0);
            let h = polyline(vec![centre, point_on(centre, marks, k, tip)])
                .style(line_style(palette::secret(), HAND_LINE))
                .build();
            s.add(h)
        });
        Ok(Self {
            centre,
            r,
            dial,
            group,
            needle,
            hand,
        })
    }

    /// A point `d` from the centre at mark `k` (a fraction between marks).
    pub fn point(&self, k: f64, d: f64) -> Vec3 {
        point_on(self.centre, self.dial.marks(), k, d)
    }

    /// The face's group and needle, as they sit in the scene.
    pub fn parts(&self) -> [Mob; 2] {
        [self.group, self.hand]
    }

    /// A ring round mark `k`'s label, in `color`: how a mark is pointed
    /// at, since the labels are never restyled (nor handed out). Detached.
    pub fn halo(&self, stage: &mut Stage<'_>, k: u32, color: Srgb) -> Mob {
        let at = self.point(f64::from(self.dial.wrap(i64::from(k))), LABEL_IN * self.r);
        ring(stage, at, HALO_IN * self.r, color)
    }

    /// The mark the needle points at, unwrapped.
    pub fn at(&self, stage: &Stage<'_>) -> i64 {
        stage.tracker_value(self.needle).unwrap_or(0.0).round() as i64
    }

    /// Bring the face in (the needle once the face is in), over `rt`.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let a = fade_in(stage.arena_mut(), self.group, ORIGIN, 1.0)?;
        stage.play(a.rt(rt))?;
        stage.add_to_scene(self.hand)?;
        Ok(())
    }

    /// Turn the needle straight to unwrapped mark `k` over `rt` (setting a
    /// share on the dial, not adding).
    pub fn turn(&self, stage: &mut Stage<'_>, k: i64, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.needle, k as f64, rt)?)?;
        Ok(())
    }

    /// Turn `steps` marks one click at a time, `each` seconds a click: the
    /// needle moves one mark, `count` (a tracker, if any) reads the clicks
    /// so far, and a ring comes and goes round the mark landed on. Returns
    /// the marks landed on.
    pub fn click(&self, stage: &mut Stage<'_>, steps: i64, each: f64, count: Option<Mob>) -> fmn::Result<Vec<u32>> {
        let from = self.at(stage);
        let marks = passed(self.dial, from, steps);
        for (i, &k) in (1_i64..).zip(&marks) {
            let to = from + steps.signum() * i;
            let mut anims = vec![stage.prepare(tween(self.needle, to as f64, 0.6 * each)?)?];
            if let Some(c) = count {
                anims.push(stage.prepare(tween(c, i as f64, 0.6 * each)?)?);
            }
            let landed = self.halo(stage, k, palette::secret());
            anims.extend(glowing(stage, &[landed], each)?);
            stage.play_prepared(anims)?;
        }
        Ok(marks)
    }

    /// An arc just outside the rim from unwrapped mark `from` forward to
    /// `to`, in `color`, and its label `text` at the arc's middle. Detached.
    pub fn arc(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        from: f64,
        to: f64,
        text: &str,
        color: Srgb,
    ) -> fmn::Result<[Mob; 2]> {
        let n = (((to - from).abs() * 12.0).ceil() as usize).max(2);
        let points: Vec<Vec3> = (0..=n)
            .map(|i| self.point(from + (to - from) * i as f64 / n as f64, ARC_IN * self.r))
            .collect();
        let line = stroke(stage, points, color, 4.0);
        let label = kit.text(
            stage,
            text,
            LABEL_PER_R * self.r * 1.2,
            color,
            self.point((from + to) / 2.0, ARC_LABEL_IN * self.r),
        )?;
        Ok([line, label])
    }

    /// Zoom from this face onto `to`: this one grows or shrinks onto `to`'s
    /// place and size as it fades, `to` comes in from this one's place and
    /// size, `with` plays alongside, all over `rt`; `to`'s needle joins it
    /// once it is in. Both needles should point at the same mark.
    pub fn zoom(&self, stage: &mut Stage<'_>, to: &Face, rt: f64, with: Vec<Box<dyn Animation>>) -> fmn::Result<()> {
        let k = to.r / self.r;
        let d = minus(to.centre, self.centre);
        let mut anims = with;
        // the group scales about its centre, the dial's (the rim bounds it)
        let a = fade_out(stage.arena_mut(), self.group, d, k)?;
        anims.push(stage.prepare(a.rt(rt))?);
        // the needle scales about its own centre, so shift it as the
        // group's map would carry it
        stage.clear_updaters(self.hand, true);
        let p = stage.get_center(self.hand);
        let a = fade_out(stage.arena_mut(), self.hand, zoom_shift(p, self.centre, d, k), k)?;
        anims.push(stage.prepare(a.rt(rt))?);
        let a = fade_in(stage.arena_mut(), to.group, d, k)?;
        anims.push(stage.prepare(a.rt(rt))?);
        stage.play_prepared(anims)?;
        stage.add_to_scene(to.hand)?;
        Ok(())
    }
}

/// A point `d` from `centre` at mark `k` of a `marks`-mark dial, clockwise
/// from the top (draw.rs's `dial_angle`).
fn point_on(centre: Vec3, marks: u32, k: f64, d: f64) -> Vec3 {
    let a = super::draw::dial_angle(k, marks);
    v(centre[0] + d * a.cos(), centre[1] + d * a.sin())
}

fn minus(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// The shift that carries a piece centred at `p` where a zoom about `c`
/// (shift `d`, then scale `k` about the shifted centre) carries it, when
/// the piece is scaled about its own centre instead.
fn zoom_shift(p: Vec3, c: Vec3, d: Vec3, k: f64) -> Vec3 {
    [
        d[0] + (k - 1.0) * (p[0] - c[0]),
        d[1] + (k - 1.0) * (p[1] - c[1]),
        d[2] + (k - 1.0) * (p[2] - c[2]),
    ]
}

// ------------------------------------------------------ the two-key vault

/// The small door, left, and its dial.
pub const SMALL_X: f64 = -5.2;
pub const SMALL_Y: f64 = 1.0;
pub const SMALL_W: f64 = 2.3;
pub const SMALL_H: f64 = 2.7;
pub const SMALL_DIAL_Y: f64 = 1.1;
pub const SMALL_DIAL_R: f64 = 0.8;
/// A and B, under the small door, each with a card frame over them, at the
/// rail's heights (draw.rs).
pub const HOLDERS: [&str; 2] = ["A", "B"];
pub const HOLDER_X: [f64; 2] = [-5.75, -4.65];
/// The column between the small door and the strip: equations, notes, the
/// dealer's spare dial, chapter 03's table.
pub const MID_X: f64 = -1.6;
/// The enlarged dial of 03_1 and 03_2, and the dealer's spare dial.
pub const BIG_X: f64 = -3.4;
pub const BIG_Y: f64 = 0.4;
pub const BIG_R: f64 = 2.2;
pub const SPARE_Y: f64 = 1.4;
pub const SPARE_R: f64 = 0.75;

/// The small door's dial's centre.
pub fn small_centre() -> Vec3 {
    v(SMALL_X, SMALL_DIAL_Y)
}

/// The enlarged dial's centre.
pub fn big_centre() -> Vec3 {
    v(BIG_X, BIG_Y)
}

/// Holder `i`'s (0 is A, 1 is B) disc centre.
pub fn holder_at(i: usize) -> Vec3 {
    v(HOLDER_X[i], RAIL_Y)
}

/// The centre of holder `i`'s card frame.
pub fn slot_at(i: usize) -> Vec3 {
    v(HOLDER_X[i], FRAME_Y)
}

/// The two-trustee vault: the small door with its face, and A and B.
pub struct TwoKeys {
    pub door: Mob,
    pub face: Face,
    discs: Vec<Mob>,
    holders: Vec<Mob>,
}

impl TwoKeys {
    /// The vault, its needle at mark `needle_at` (where the face it zooms
    /// from or to points).
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, dial: Dial, needle_at: u32) -> fmn::Result<Self> {
        let door = panel(stage, palette::ghost(), DOOR_O, v(SMALL_X, SMALL_Y), SMALL_W, SMALL_H)?;
        let face = Face::build(
            kit,
            stage,
            dial,
            small_centre(),
            SMALL_DIAL_R,
            Palette::dim(),
            needle_at,
        )?;
        let (mut discs, mut holders) = (Vec::new(), Vec::new());
        for (i, name) in HOLDERS.iter().enumerate() {
            let d = disc(stage, palette::share(), holder_at(i), TRUSTEE_R);
            let n = kit.text(stage, name, 24.0, hex(BACKGROUND), holder_at(i))?;
            let f = frame(stage, Palette::dim(), slot_at(i), FRAME_W, FRAME_H)?;
            holders.push(bundle(stage, &[d, n, f])?);
            discs.push(d);
        }
        Ok(Self {
            door,
            face,
            discs,
            holders,
        })
    }

    /// Holder `i`'s disc (0 is A, 1 is B).
    pub fn disc(&self, i: usize) -> Mob {
        self.discs[i]
    }

    /// The door, A and B (not the face).
    pub fn pieces(&self) -> Vec<Mob> {
        let mut all = vec![self.door];
        all.extend(&self.holders);
        all
    }

    /// The door, A and B fading in, prepared, over `rt` (the face comes
    /// with [`Face::show`] or a zoom).
    pub fn arriving(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in self.pieces() {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        Ok(anims)
    }

    /// The shut door pulses, over `rt` seconds, tinted its own shut color
    /// as the big door's (draw.rs's `Vault::pulse`: the indication default
    /// yellow read as a change in the preview).
    pub fn pulse(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let a = swelling(stage, self.door, DOOR_PULSE, palette::ghost(), rt)?;
        stage.play_prepared(vec![a])?;
        Ok(())
    }
}

/// Holder `i`'s card: `text` in `color`, in its frame. Detached.
pub fn holder_card(kit: &Kit, stage: &mut Stage<'_>, i: usize, text: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, CARD_SIZE, color, slot_at(i))
}

/// A card face down in holder `i`'s frame. Detached.
pub fn card_back(stage: &mut Stage<'_>, i: usize) -> fmn::Result<Mob> {
    panel(stage, Palette::dim(), 0.85, slot_at(i), FRAME_W - 0.14, FRAME_H - 0.12)
}

// --------------------------------------------------------------- emphasis

/// A card's or a tag's outline's line width (a frame's is 2).
const OUTLINE_LINE: f64 = 3.0;
/// How far a card's halo stands outside its frame, and a tag's outside its
/// words.
const SLOT_PAD: f64 = 0.07;
pub const TEXT_PAD: f64 = 0.12;

/// A ring of radius `r` round `at`, in `color`. Detached.
fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let c = Circle::new()
        .radius(r)
        .style(line_style(color, RING_LINE))
        .build()
        .moved_to(at);
    stage.arena_mut().add(c)
}

/// A rounded outline `w` × `h` round `at`, in `color`. Detached.
fn outline(stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    let o = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(line_style(color, OUTLINE_LINE))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(o))
}

/// An outline round holder `i`'s card frame, in the share role: how a card
/// is pointed at, face up or face down. Detached.
pub fn slot_halo(stage: &mut Stage<'_>, i: usize) -> fmn::Result<Mob> {
    outline(
        stage,
        slot_at(i),
        FRAME_W + 2.0 * SLOT_PAD,
        FRAME_H + 2.0 * SLOT_PAD,
        palette::share(),
    )
}

/// An outline round the words `m` as they stand, in `color`. Detached.
pub fn text_halo(stage: &mut Stage<'_>, m: Mob, color: Srgb) -> fmn::Result<Mob> {
    let (at, w, h) = (stage.get_center(m), stage.get_width(m), stage.get_height(m));
    outline(stage, at, w + 2.0 * TEXT_PAD, h + 2.0 * TEXT_PAD, color)
}

/// Each of the detached `halos` fading in and out again over `rt`, half
/// each way, prepared, to play alongside other moves. Each fade-out is a
/// remover, so no halo stays in the scene.
pub fn glowing(stage: &mut Stage<'_>, halos: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(halos.len());
    for &h in halos {
        let a = fade_in(stage.arena_mut(), h, ORIGIN, 1.0)?;
        let comes = stage.prepare(a.rt(rt / 2.0))?;
        let a = fade_out(stage.arena_mut(), h, ORIGIN, 1.0)?;
        let goes = stage.prepare(a.rt(rt / 2.0))?;
        let both = Succession::new(stage.arena_mut(), vec![comes, goes])?;
        anims.push(stage.prepare(both)?);
    }
    Ok(anims)
}

/// The detached `halos` coming and going together over `rt`.
pub fn glow(stage: &mut Stage<'_>, halos: &[Mob], rt: f64) -> fmn::Result<()> {
    if halos.is_empty() {
        return Ok(());
    }
    let anims = glowing(stage, halos, rt)?;
    stage.play_prepared(anims)?;
    Ok(())
}

/// `color` as an indication tint.
fn tint(color: Srgb) -> [f32; 3] {
    [color.r as f32, color.g as f32, color.b as f32]
}

/// A shape (never typeset text) growing to `scale` and back over `rt`,
/// tinted `color`, prepared.
fn swelling(stage: &mut Stage<'_>, shape: Mob, scale: f64, color: Srgb, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fmn::animation::indicate(stage.arena_mut(), shape, scale, Some(tint(color)))?;
    stage.prepare(a.rt(rt))
}

/// Shapes (never typeset text: dots, bars) swelling together over `rt`,
/// tinted `color`, their own role, not the indication default yellow,
/// which is the spins' role here.
pub fn swell_all(stage: &mut Stage<'_>, shapes: &[Mob], color: Srgb, rt: f64) -> fmn::Result<()> {
    if shapes.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(shapes.len());
    for &m in shapes {
        anims.push(swelling(stage, m, FLASH, color, rt)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

// --------------------------------------------------------------- playback

/// A move of `m`'s centre to `to` over `rt`, prepared.
pub fn moving(stage: &mut Stage<'_>, m: Mob, to: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    stage.prepare(a)
}

/// Move each mobject's centre to its own target, together, over `rt`.
pub fn move_all(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], rt: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims = Vec::with_capacity(moves.len());
    for &(m, to) in moves {
        anims.push(moving(stage, m, to, rt)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade-outs, prepared, of everything in the scene but `keep`, sliding
/// each by `shift` as it goes, over `rt` (live readouts stop updating
/// first, as in the kit's `fade_all`).
pub fn leaving_all_but(
    stage: &mut Stage<'_>,
    keep: &[Mob],
    shift: Vec3,
    rt: f64,
) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mobs: Vec<Mob> = stage.scene().mobjects().to_vec();
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for m in mobs {
        if keep.contains(&m) {
            continue;
        }
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, shift, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

/// Fade-outs, prepared, of `mobs`, each sliding by `shift`, over `rt`.
pub fn leaving(stage: &mut Stage<'_>, mobs: &[Mob], shift: Vec3, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, shift, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS, STROKE_WIDTH_CONVERSION};

    use super::super::draw::{STRIP_LEFT, STRIP_TITLE_Y, STRIP_Y, TICK_H};
    use super::*;
    use crate::model::{A_SHARE, MARKS, ORDINARY_PIECES, SECRET, any_spins, learns_nothing};

    /// An ordinary piece b leaves exactly the marks b and up, whatever b is:
    /// the strip's view and the search over A's piece agree, and what goes
    /// is exactly 0 to b − 1.
    #[test]
    fn a_cut_piece_rules_out_exactly_the_marks_below_it() {
        let dial = Dial::vault();
        for b in dial.all() {
            let found = ordinary_secrets(dial, b);
            assert_eq!(ordinary_view(b).possible(dial), found, "piece {b}");
            assert_eq!(found, (b..MARKS).collect::<Vec<_>>(), "piece {b}");
            let gone: Vec<u32> = dial.all().filter(|s| !found.contains(s)).collect();
            assert_eq!(gone, (0..b).collect::<Vec<_>>(), "piece {b}");
            assert_eq!(found.len() as u32, MARKS - b);
        }
        // the example: B's 3 leaves 8, A's piece is the rest of the secret
        let b = ORDINARY_PIECES[1];
        assert_eq!(ordinary_secrets(dial, b).len(), 8);
        assert_eq!(ORDINARY_PIECES[0] + b, SECRET);
    }

    /// A dial share b keeps every secret, and each one exactly once: the
    /// strip's view and the search over A's marks agree for every b.
    #[test]
    fn a_dial_share_keeps_every_secret_once() {
        let dial = Dial::vault();
        let all: Vec<u32> = dial.all().collect();
        for b in dial.all() {
            assert_eq!(dial_view(b).possible(dial), all, "share {b}");
            assert_eq!(dial_secrets(dial, b), all, "share {b}");
            let rows = if_a_held(dial, b);
            for s in dial.all() {
                assert_eq!(
                    rows.iter().filter(|&&(_, t)| t == s).count(),
                    1,
                    "share {b}, secret {s}"
                );
            }
            // and each row is A's mark turned on by b
            assert!(rows.iter().all(|&(a, s)| dial.sub(s, b) == a));
        }
        // the stand-in view is the model's own: a one-spin share at 1 fits
        // every secret equally, whatever it reads
        assert!(learns_nothing(dial, &[1], 1, any_spins));
        // the example: A's 9 and B's 7 make the secret
        let b = dial.sub(SECRET, A_SHARE);
        assert_eq!(b, 7);
        assert!(if_a_held(dial, b).contains(&(A_SHARE, SECRET)));
    }

    /// Seven clicks on from 9 pass 10 and 0 and stop at 5; seven back from
    /// 5 pass 0 and 10 and stop at 9.
    #[test]
    fn the_clicks_go_round_through_zero() {
        let dial = Dial::vault();
        assert_eq!(passed(dial, 9, 7), vec![10, 0, 1, 2, 3, 4, 5]);
        assert_eq!(passed(dial, 16, -7), vec![4, 3, 2, 1, 0, 10, 9]);
        assert_eq!(passed(dial, 5, -7), vec![4, 3, 2, 1, 0, 10, 9]);
        assert!(passed(dial, 3, 0).is_empty());
        for a in dial.all() {
            for b in dial.all() {
                let on = passed(dial, i64::from(a), i64::from(b));
                assert_eq!(on.last().copied().unwrap_or(a), dial.add(a, b));
            }
        }
    }

    /// Turning on to the top goes forward from wherever the needle is, and
    /// lands on 0: from 9 on to 11, from 16 (5) on to 22.
    #[test]
    fn the_needle_turns_on_to_the_top() {
        let dial = Dial::vault();
        assert_eq!(next_top(9, MARKS), 11);
        assert_eq!(next_top(16, MARKS), 22);
        for at in 0..40 {
            let top = next_top(at, MARKS);
            assert!(top > at && top - at <= i64::from(MARKS));
            assert_eq!(dial.wrap(top), 0);
        }
    }

    /// The needle's shift in a zoom carries every point of it where the
    /// group's zoom carries that point.
    #[test]
    fn the_needle_zooms_with_its_face() {
        let (c, d, k) = (
            small_centre(),
            minus(big_centre(), small_centre()),
            BIG_R / SMALL_DIAL_R,
        );
        for p in [v(-5.0, 1.4), c, v(-4.8, 0.9)] {
            let dp = zoom_shift(p, c, d, k);
            for x in [p, c, v(p[0] + 0.1, p[1] - 0.2)] {
                // the group's map, and the needle's (shift, then scale about
                // its shifted centre)
                let want = [c[0] + d[0] + k * (x[0] - c[0]), c[1] + d[1] + k * (x[1] - c[1])];
                let got = [p[0] + dp[0] + k * (x[0] - p[0]), p[1] + dp[1] + k * (x[1] - p[1])];
                assert!((want[0] - got[0]).abs() < 1e-12 && (want[1] - got[1]).abs() < 1e-12);
            }
        }
    }

    /// A mark's ring rings its label alone on every face drawn, the
    /// smallest (the spare's) included. Its line, half on each side of the
    /// circle, clears the widest label, "10" (0.0055 units a point a digit
    /// wide and 0.0065 tall, a quarter over the corpus's measured 0.0045 a
    /// character), the round-capped inner end of the label's tick, the
    /// round-capped tip of a needle pointing at it, and the next label. A
    /// card's halo stands clear of its frame's line and of the other card's
    /// halo, over its holder's disc and under the pulsing door.
    #[test]
    fn the_halos_ring_what_they_point_at_alone() {
        let half = |line: f64| line * STROKE_WIDTH_CONVERSION / 2.0;
        for r in [SPARE_R, SMALL_DIAL_R, BIG_R] {
            let (centre, ring) = (LABEL_IN * r, HALO_IN * r);
            let reach = (0.0055 * LABEL_PER_R * r).hypot(0.0065 * LABEL_PER_R * r / 2.0);
            assert!(ring - half(RING_LINE) > reach, "the ring clears the label, radius {r}");
            assert!(
                centre + ring + half(RING_LINE) < TICK_IN * r - half(DIAL_LINE) - 0.003,
                "the ring inside the ticks, radius {r}"
            );
            assert!(
                centre - ring - half(RING_LINE) > NEEDLE_IN * r + half(HAND_LINE) + 0.003,
                "the ring outside the needle's tip, radius {r}"
            );
            let next = 2.0 * centre * (std::f64::consts::PI / f64::from(MARKS)).sin();
            assert!(
                next - ring - half(RING_LINE) > reach + 0.05,
                "the ring clears the next label, radius {r}"
            );
        }
        assert!(
            SLOT_PAD - half(OUTLINE_LINE) > half(2.0) + 0.02,
            "a card halo off its frame"
        );
        const {
            assert!(
                HOLDER_X[1] - HOLDER_X[0] > FRAME_W + 2.0 * SLOT_PAD + 0.2,
                "the card halos apart"
            );
            assert!(
                FRAME_Y - FRAME_H / 2.0 - SLOT_PAD > RAIL_Y + TRUSTEE_R + 0.05,
                "a card halo over its holder's disc"
            );
            assert!(
                FRAME_Y + FRAME_H / 2.0 + SLOT_PAD < SMALL_Y - DOOR_PULSE * SMALL_H / 2.0 - 0.1,
                "a card halo under the pulsing door"
            );
            assert!(
                HOLDER_X[0] - FRAME_W / 2.0 - SLOT_PAD > -FRAME_X_RADIUS + 0.25,
                "A's card halo inside the left edge"
            );
        }
    }

    /// The small door, its dial and the arc stay inside the frame and apart
    /// from A and B under it, the middle column and the strip; the enlarged
    /// dial and its arcs stay inside the frame and left of the equations.
    #[test]
    fn the_two_key_vault_fits() {
        const DOOR_TOP: f64 = SMALL_Y + SMALL_H / 2.0;
        const DOOR_BOTTOM: f64 = SMALL_Y - SMALL_H / 2.0;
        const DOOR_RIGHT: f64 = SMALL_X + SMALL_W / 2.0;
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const {
            assert!(
                SMALL_X - SMALL_W / 2.0 > -FRAME_X_RADIUS + 0.25,
                "the door inside the left edge"
            );
            assert!(
                SMALL_DIAL_Y + SMALL_DIAL_R < DOOR_TOP - 0.2 && SMALL_DIAL_Y - SMALL_DIAL_R > DOOR_BOTTOM + 0.2,
                "the dial on the door"
            );
            assert!(SMALL_W / 2.0 > ARC_IN * SMALL_DIAL_R + 0.1, "the arc on the door");
            assert!(
                SMALL_Y - DOOR_PULSE * SMALL_H / 2.0 > FRAME_Y + FRAME_H / 2.0 + 0.1,
                "the pulsing door over the card frames"
            );
            assert!(
                SMALL_Y + DOOR_PULSE * SMALL_H / 2.0 < FRAME_Y_RADIUS - 0.25,
                "the pulsing door inside the top edge"
            );
            assert!(HOLDER_X[1] - HOLDER_X[0] > FRAME_W + 0.2, "A and B apart");
            assert!(
                HOLDER_X[0] - FRAME_W / 2.0 > SMALL_X - SMALL_W / 2.0 - 0.05,
                "A under the door"
            );
            assert!(HOLDER_X[1] + FRAME_W / 2.0 < DOOR_RIGHT + 0.05, "B under the door");
            assert!(RAIL_Y - TRUSTEE_R > BAND_TOP + 0.2, "A and B over the lower-third band");
            assert!(MID_X - 1.2 > DOOR_RIGHT + 0.2, "the middle column right of the door");
            assert!(MID_X + 1.2 < STRIP_LEFT - 0.2, "the middle column left of the strip");
            assert!(
                MID_X - SPARE_R * ARC_LABEL_IN > DOOR_RIGHT,
                "the spare dial right of the door"
            );
            assert!(
                MID_X + SPARE_R * ARC_LABEL_IN < STRIP_LEFT,
                "the spare dial left of the strip"
            );
            assert!(
                SPARE_Y + SPARE_R + 0.6 < FRAME_Y_RADIUS - 0.2,
                "the spare dial's title inside the top edge"
            );
            assert!(
                BIG_Y + ARC_LABEL_IN * BIG_R < FRAME_Y_RADIUS - 0.1,
                "the enlarged dial's arcs inside the top"
            );
            assert!(
                BIG_X - ARC_IN * BIG_R > -FRAME_X_RADIUS + 0.25,
                "the enlarged dial inside the left edge"
            );
            assert!(
                BIG_Y - BIG_R - 0.8 > -FRAME_Y_RADIUS + 0.25,
                "the click count under the enlarged dial"
            );
            // the strip's tick tops stay under the "met" dots of chapter
            // 03, which stay under its title
            assert!(
                STRIP_Y + TICK_H / 2.0 + 0.1 < STRIP_TITLE_Y - 0.3,
                "room for the dots over the ticks"
            );
        }
    }
}
