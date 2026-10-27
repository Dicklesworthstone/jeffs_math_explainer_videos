//! The video's look, shared by its chapters (storyboard, "Devices"): the
//! vault with its dial, the trustee rail with its card frames, the strip of
//! possible secrets, card reading time, line timing and think time.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, short open strokes under moving things.
//!
//! The playback, line-timing and think-time helpers are randomized
//! response's (`explainer_corpus/randomized_response/src/chapters/draw.rs`,
//! after the Golay chapter's, which the orchestrator accepted on rendered
//! drafts); they live in each video until the shared kit takes them.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Kit, Palette, Script, Timed, hex, tween, v};
use fmn::prelude::*;

use crate::model::{Dial, View};
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

// ------------------------------------------------------------------ shapes

/// A flat disc of radius `r` in `color`, centred at `at`. Detached.
pub fn disc(stage: &mut Stage<'_>, color: Srgb, at: Vec3, r: f64) -> Mob {
    let d = Circle::new().radius(r).style(flat(color, 1.0)).build().moved_to(at);
    stage.arena_mut().add(d)
}

/// An open stroke through `points` in `color`, `w` wide. Detached.
pub fn stroke(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, w: f64) -> Mob {
    let l = polyline(points).style(line_style(color, w)).build();
    stage.arena_mut().add(l)
}

/// A flat rounded panel `w` × `h` in `color` at opacity `o`, centred at
/// `at`. Detached.
pub fn panel(stage: &mut Stage<'_>, color: Srgb, o: f64, at: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let p = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.08_f64.min(w / 4.0))
        .style(flat(color, o))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(p))
}

/// An outlined rounded frame `w` × `h` in `color`, centred at `at`.
/// Detached.
pub fn frame(stage: &mut Stage<'_>, color: Srgb, at: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let f = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.08)
        .style(line_style(color, 2.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(f))
}

/// One parent for `members`, so they come and go together. Detached.
pub fn bundle(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

// --------------------------------------------------------------- the vault

/// The door, upper left, and the dial on it.
pub const DOOR_X: f64 = -3.9;
pub const DOOR_Y: f64 = 0.85;
pub const DOOR_W: f64 = 3.0;
pub const DOOR_H: f64 = 3.5;
/// The door's fill opacity, whatever its color.
pub const DOOR_O: f64 = 0.55;
pub const DIAL_Y: f64 = 1.05;
pub const DIAL_R: f64 = 1.0;
/// Where a mark's tick starts (it runs out to the rim), its label sits, and
/// the needle's tip reaches, from the dial's centre.
pub const TICK_IN: f64 = 0.86;
pub const MARK_LABEL_R: f64 = 0.68;
pub const NEEDLE_R: f64 = 0.48;
pub const HUB_R: f64 = 0.08;
/// A mark's label size, the rim's and ticks' line width, the needle's.
pub const MARK_LABEL_SIZE: f64 = 18.0;
pub const DIAL_LINE: f64 = 3.0;
pub const HAND_LINE: f64 = 6.0;
/// The alarm lamp over the door.
pub const LAMP_Y: f64 = 3.0;
pub const LAMP_R: f64 = 0.16;
/// How far the door grows when it pulses: little enough that it stays
/// clear of the card frames under it and the lamp over it.
pub const DOOR_PULSE: f64 = 1.08;

/// Where mark `k` (a fraction while the needle turns) sits on a dial of
/// `marks` marks: clockwise from the top, as radians.
pub fn dial_angle(k: f64, marks: u32) -> f64 {
    std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * k / f64::from(marks)
}

/// A point `r` from the dial's centre at mark `k`.
pub fn on_dial(k: f64, marks: u32, r: f64) -> Vec3 {
    let a = dial_angle(k, marks);
    v(DOOR_X + r * a.cos(), DIAL_Y + r * a.sin())
}

/// The vault: its door (recoloured to show it shut, open or opened for the
/// wrong person), its alarm lamp, and the needle, redrawn every frame from
/// one value tracker, the mark it points at.
pub struct Vault {
    pub door: Mob,
    pub lamp: Mob,
    /// The tracker: the mark the needle points at.
    pub needle: Mob,
    needle_mob: Mob,
    parts: Vec<Mob>,
}

impl Vault {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, dial: Dial) -> fmn::Result<Self> {
        let door = panel(stage, palette::ghost(), DOOR_O, v(DOOR_X, DOOR_Y), DOOR_W, DOOR_H)?;
        let rim = Circle::new()
            .radius(DIAL_R)
            .style(line_style(Palette::dim(), DIAL_LINE))
            .build()
            .moved_to(v(DOOR_X, DIAL_Y));
        let rim = stage.arena_mut().add(rim);
        let mut parts = vec![door, rim];
        let marks = dial.marks();
        for k in dial.all() {
            let at = f64::from(k);
            parts.push(stroke(
                stage,
                vec![on_dial(at, marks, TICK_IN), on_dial(at, marks, DIAL_R)],
                Palette::dim(),
                DIAL_LINE,
            ));
            parts.push(kit.text(
                stage,
                &k.to_string(),
                MARK_LABEL_SIZE,
                Palette::ink(),
                on_dial(at, marks, MARK_LABEL_R),
            )?);
        }
        parts.push(disc(stage, palette::secret(), v(DOOR_X, DIAL_Y), HUB_R));
        let lamp = disc(stage, palette::ghost(), v(DOOR_X, LAMP_Y), LAMP_R);
        parts.push(lamp);
        let needle = stage.add_value_tracker(0.0);
        let needle_mob = stage.always_redraw(move |s| {
            let k = s.tracker_value(needle).unwrap_or(0.0);
            let hand = polyline(vec![v(DOOR_X, DIAL_Y), on_dial(k, marks, NEEDLE_R)])
                .style(line_style(palette::secret(), HAND_LINE))
                .build();
            s.add(hand)
        });
        Ok(Self {
            door,
            lamp,
            needle,
            needle_mob,
            parts,
        })
    }

    /// Bring the vault in: door, dial and lamp fade in, then the needle.
    pub fn show(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_all(stage, &self.parts, rt)?;
        stage.add_to_scene(self.needle_mob)?;
        Ok(())
    }

    /// Turn the needle to mark `k` over `rt` seconds.
    pub fn turn(&self, stage: &mut Stage<'_>, k: u32, rt: f64) -> fmn::Result<()> {
        stage.play(tween(self.needle, f64::from(k), rt)?)?;
        Ok(())
    }

    /// Emphasise a closed door without turning it the indication default
    /// yellow: that looked like an opening/failure colour in the preview.
    pub fn pulse(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let c = palette::ghost();
        let tint = [c.r as f32, c.g as f32, c.b as f32];
        let a = fmn::animation::indicate(stage.arena_mut(), self.door, DOOR_PULSE, Some(tint))?;
        stage.play(a.rt(rt))?;
        Ok(())
    }

    /// The alarm: the lamp lights red, flashes, and goes dark again.
    pub fn alarm(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        recolor_all(stage, &[(self.lamp, palette::leak())], 0.15)?;
        flash_all(stage, &[self.lamp], 0.5)?;
        recolor_all(stage, &[(self.lamp, palette::ghost())], 0.3)
    }
}

// --------------------------------------------------------- the trustee rail

/// The rail along the bottom left: trustee 1's disc, the step between
/// discs, and the card frames over them.
pub const RAIL_X0: f64 = -5.7;
pub const RAIL_DX: f64 = 0.9;
pub const RAIL_Y: f64 = -2.2;
pub const TRUSTEE_R: f64 = 0.26;
pub const FRAME_Y: f64 = -1.45;
pub const FRAME_W: f64 = 0.62;
pub const FRAME_H: f64 = 0.44;
/// A card's digits, and a tag's words, in a frame.
pub const CARD_SIZE: f64 = 26.0;
const TAG_SIZE: f64 = 16.0;

/// Trustee `t`'s disc centre (trustees count from 1).
pub fn trustee_at(t: u32) -> Vec3 {
    v(RAIL_X0 + RAIL_DX * f64::from(t - 1), RAIL_Y)
}

/// The centre of trustee `t`'s card frame.
pub fn frame_at(t: u32) -> Vec3 {
    v(RAIL_X0 + RAIL_DX * f64::from(t - 1), FRAME_Y)
}

/// The five trustees, each a numbered disc with an empty card frame over it.
pub struct Rail {
    first: u32,
    discs: Vec<Mob>,
    pub frames: Vec<Mob>,
    trustees: Vec<Mob>,
}

impl Rail {
    /// The rail for trustees `first` to `first + count − 1`.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, trustees: &[u32]) -> fmn::Result<Self> {
        let first = *trustees.first().ok_or_else(|| fail("a rail with no trustees".into()))?;
        if trustees.iter().zip(first..).any(|(&t, want)| t != want) {
            return Err(fail(format!("trustees {trustees:?} are not numbered in a row")));
        }
        let (mut discs, mut frames, mut whole) = (Vec::new(), Vec::new(), Vec::new());
        for &t in trustees {
            let d = disc(stage, palette::share(), trustee_at(t), TRUSTEE_R);
            let n = kit.text(stage, &t.to_string(), 24.0, hex(BACKGROUND), trustee_at(t))?;
            let f = frame(stage, Palette::dim(), frame_at(t), FRAME_W, FRAME_H)?;
            whole.push(bundle(stage, &[d, n, f])?);
            discs.push(d);
            frames.push(f);
        }
        Ok(Self {
            first,
            discs,
            frames,
            trustees: whole,
        })
    }

    /// The complete trustee groups, including their card frames.
    pub fn parts(&self) -> &[Mob] {
        &self.trustees
    }

    /// Trustee `t`'s disc.
    pub fn disc(&self, t: u32) -> Mob {
        self.discs[(t - self.first) as usize]
    }

    /// The trustees come in one after another, the whole taking `rt`.
    pub fn reveal(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        fade_in_lagged(stage, &self.trustees, rt, 0.3)
    }

    /// `(disc, color)` for each of `group`, to recolor together.
    pub fn tint(&self, group: &[u32], color: Srgb) -> Vec<(Mob, Srgb)> {
        group.iter().map(|&t| (self.disc(t), color)).collect()
    }
}

/// Trustee `t`'s card: `text` (a copy of the secret, later a share) in
/// `color`, in its frame. Detached.
pub fn card(kit: &Kit, stage: &mut Stage<'_>, t: u32, text: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, CARD_SIZE, color, frame_at(t))
}

/// A small tag in trustee `t`'s frame ("lost", "away"). Detached.
pub fn tag(kit: &Kit, stage: &mut Stage<'_>, t: u32, text: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, TAG_SIZE, color, frame_at(t))
}

// ----------------------------------------------------------------- the strip

/// The strip at the right: mark 0's tick, the step between ticks, their
/// size, the labels under them, the title and count over them, and the
/// caption under the labels.
pub const STRIP_X0: f64 = 0.9;
pub const STRIP_DX: f64 = 0.55;
pub const STRIP_Y: f64 = 1.1;
pub const TICK_W: f64 = 0.34;
pub const TICK_H: f64 = 0.7;
pub const STRIP_LABEL_DY: f64 = 0.62;
pub const STRIP_TITLE_Y: f64 = 2.15;
pub const STRIP_CAPTION_Y: f64 = -0.05;
pub const STRIP_LEFT: f64 = STRIP_X0 - TICK_W / 2.0;
const STRIP_TITLE: &str = "possible secrets";
const STRIP_COUNT_SIZE: f64 = 40.0;
const STRIP_CAPTION_SIZE: f64 = 22.0;

/// The right edge of the strip of a `marks`-mark dial (the count's right
/// edge too).
pub fn strip_right(marks: u32) -> f64 {
    STRIP_X0 + STRIP_DX * f64::from(marks - 1) + TICK_W / 2.0
}

/// Mark `k`'s tick centre.
pub fn tick_at(k: u32) -> Vec3 {
    v(STRIP_X0 + STRIP_DX * f64::from(k), STRIP_Y)
}

/// The strip of possible secrets: one tick per mark, gold while that mark
/// still fits what the person watched holds, and a live count of the lit
/// ticks. Ticks and count both come from [`View::possible`]; the caption
/// says whose view it is.
pub struct Strip {
    dial: Dial,
    ticks: Vec<Mob>,
    labels: Vec<Mob>,
    title: Mob,
    /// The tracker the count reads.
    count: Mob,
    caption: Option<Mob>,
    lit: Vec<u32>,
}

impl Strip {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, dial: Dial) -> fmn::Result<Self> {
        let (mut ticks, mut labels) = (Vec::new(), Vec::new());
        for k in dial.all() {
            let at = tick_at(k);
            ticks.push(panel(stage, palette::secret(), 0.9, at, TICK_W, TICK_H)?);
            labels.push(kit.text(
                stage,
                &k.to_string(),
                20.0,
                Palette::dim(),
                v(at[0], at[1] - STRIP_LABEL_DY),
            )?);
        }
        let title = kit.text_left(stage, STRIP_TITLE, 24.0, Palette::dim(), v(STRIP_LEFT, STRIP_TITLE_Y))?;
        let count = stage.add_value_tracker(0.0);
        Ok(Self {
            dial,
            ticks,
            labels,
            title,
            count,
            caption: None,
            lit: Vec::new(),
        })
    }

    /// The marks lit now.
    pub fn lit(&self) -> &[u32] {
        &self.lit
    }

    /// Bring the strip in as an outsider sees it: every tick lit, one after
    /// another, while the count climbs to match, the whole taking `rt`.
    pub fn reveal(&mut self, kit: &Kit, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let lit = View::Outsider.possible(self.dial);
        if lit.len() != self.ticks.len() {
            return Err(fail(format!(
                "an outsider's strip lights {} of {} ticks",
                lit.len(),
                self.ticks.len()
            )));
        }
        let mut columns = Vec::with_capacity(self.ticks.len());
        for (&t, &l) in self.ticks.iter().zip(&self.labels) {
            let both = bundle(stage, &[t, l])?;
            let a = fade_in(stage.arena_mut(), both, ORIGIN, 1.0)?;
            columns.push(stage.prepare(a)?);
        }
        let columns = AnimationGroup::with_lag_ratio(stage.arena_mut(), columns, 0.15)?.with_run_time(rt);
        let title = fade_in(stage.arena_mut(), self.title, ORIGIN, 1.0)?;
        kit.live_number(
            stage,
            self.count,
            0,
            false,
            STRIP_COUNT_SIZE,
            palette::secret(),
            v(strip_right(self.dial.marks()), STRIP_TITLE_Y),
            RIGHT,
        )?;
        let anims = vec![
            stage.prepare(columns)?,
            stage.prepare(title.rt(rt.min(0.5)))?,
            stage.prepare(tween(self.count, lit.len() as f64, rt)?)?,
        ];
        stage.play_prepared(anims)?;
        self.lit = lit;
        Ok(())
    }

    /// Show what `view` leaves possible, captioned `caption`: ticks dim or
    /// light, the count follows and the caption changes, together over `rt`.
    pub fn show(&mut self, kit: &Kit, stage: &mut Stage<'_>, view: &View, caption: &str, rt: f64) -> fmn::Result<()> {
        self.show_with(kit, stage, view, caption, rt, Vec::new())
    }

    /// [`Strip::show`], with `with` (prepared over the same `rt`) playing
    /// alongside, so the strip changes in the same move as the picture.
    pub fn show_with(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        view: &View,
        caption: &str,
        rt: f64,
        with: Vec<Box<dyn Animation>>,
    ) -> fmn::Result<()> {
        let lit = view.possible(self.dial);
        let mut anims = with;
        for (k, &m) in self.dial.all().zip(&self.ticks) {
            let c = if lit.contains(&k) {
                palette::secret()
            } else {
                palette::ghost()
            };
            anims.push(stage.prepare(recolor(m, c, rt)?)?);
        }
        anims.push(stage.prepare(tween(self.count, lit.len() as f64, rt)?)?);
        let new = kit.text_left(
            stage,
            caption,
            STRIP_CAPTION_SIZE,
            Palette::dim(),
            v(STRIP_LEFT, STRIP_CAPTION_Y),
        )?;
        if let Some(old) = self.caption.replace(new) {
            let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
        }
        let a = fade_in(stage.arena_mut(), new, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
        stage.play_prepared(anims)?;
        self.lit = lit;
        Ok(())
    }
}

// --------------------------------------------------------------- playback

/// Recolor `m` to `color` (its opacity unchanged) over `rt` seconds.
pub fn recolor(m: Mob, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_color(color)?)
}

/// [`recolor_all`]'s animations, prepared but not played, to play with
/// others.
pub fn recolorings(stage: &mut Stage<'_>, changes: &[(Mob, Srgb)], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, c) in changes {
        anims.push(stage.prepare(recolor(m, c, rt)?)?);
    }
    Ok(anims)
}

/// Recolor each mobject to its own color, all together, over `rt` seconds.
pub fn recolor_all(stage: &mut Stage<'_>, changes: &[(Mob, Srgb)], rt: f64) -> fmn::Result<()> {
    if changes.is_empty() {
        return Ok(());
    }
    let anims = recolorings(stage, changes, rt)?;
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring detached mobjects in one after another, the whole taking `rt`.
pub fn fade_in_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// How far a flash grows what it highlights.
pub const FLASH: f64 = 1.25;

/// Flash several mobjects (the highlight role), together.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    pulse_all(stage, mobs, FLASH, rt)
}

/// Flash several mobjects together, each growing to `scale` and back.
pub fn pulse_all(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64, rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, scale, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// [`swap`]'s animations, prepared but not played, to play with others.
pub fn swaps(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in old {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in new {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

/// Fade `old` out and the detached `new` in, in one move.
pub fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<()> {
    if old.is_empty() && new.is_empty() {
        return Ok(());
    }
    let anims = swaps(stage, old, new, rt)?;
    stage.play_prepared(anims)?;
    Ok(())
}

/// Wait `seconds` (a beat of stillness after a reveal).
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

/// Characters a second a lower-third is read at (the video can be paused):
/// every card stays up at least its characters over this, from the moment it
/// is fully in.
pub const READ_CPS: f64 = 15.0;

/// When a card whose text is `text`, fully in now, has been read.
pub fn read_by(stage: &Stage<'_>, text: &str) -> f64 {
    stage.scene().time().to_f64() + text.chars().count() as f64 / READ_CPS
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the phrase it illustrates. A
/// phrase's moment is its sentence's start, as the narrator measured it,
/// plus its share of that sentence's characters; the narrated draft's cue
/// sheets check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    /// Each sentence's first character and its start, seconds into the line.
    sentences: Vec<(usize, f64)>,
}

/// The character each sentence of `text` starts at: after a `.`, `?` or
/// `!` and its space, as the narrator splits a line.
fn sentence_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![0];
    for (i, pair) in chars.windows(2).enumerate() {
        if matches!(pair[0], '.' | '?' | '!') && pair[1] == ' ' && i + 2 < chars.len() {
            out.push(i + 2);
        }
    }
    out
}

/// Line `id`'s sentences, each with its start in seconds into the line:
/// the narrator's `measured` onsets, one per sentence, or with no narrator
/// (an empty list) each sentence's share of `len` by characters. A
/// narrator that splits the line differently is an error, never a guess.
fn timed_sentences(id: &str, text: &str, len: f64, measured: &[f64]) -> fmn::Result<Vec<(usize, f64)>> {
    let offsets = sentence_offsets(text);
    if measured.is_empty() {
        let all = text.chars().count().max(1) as f64;
        return Ok(offsets.into_iter().map(|at| (at, len * at as f64 / all)).collect());
    }
    if measured.len() != offsets.len() {
        return Err(fail(format!(
            "line {id} has {} sentences, but the narrator timed {}",
            offsets.len(),
            measured.len()
        )));
    }
    Ok(offsets.into_iter().zip(measured.iter().copied()).collect())
}

/// Speak line `id` now (after the previous line), as `Kit::say`.
pub fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Spoken> {
    kit.say(stage, id)?;
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?
        .to_owned();
    let words = text.split_whitespace().count() as f64;
    let len = kit
        .narrator
        .as_ref()
        .and_then(|n| n.length(id))
        .unwrap_or(words / SILENT_WPS + 0.4);
    let sentences = timed_sentences(id, &text, len, &kit.sentence_starts(id))?;
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
        sentences,
    })
}

impl Spoken {
    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let byte = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let i = self.text[..byte].chars().count();
        let all = self.text.chars().count().max(1);
        let j = self.sentences.iter().rposition(|&(at, _)| at <= i).unwrap_or(0);
        let (from, t0) = self.sentences[j];
        let (to, t1) = self.sentences.get(j + 1).copied().unwrap_or((all, self.len));
        let share = (i - from) as f64 / (to - from).max(1) as f64;
        Ok(self.start + t0 + (t1 - t0) * share)
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }

    /// Wait (if need be) until the line has been spoken.
    pub fn wait_end(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        wait_until(stage, self.start + self.len)
    }
}

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown and "pause for more time").
pub const THINK_S: f64 = 4.0;
pub const THINK_R: f64 = 0.62;
pub const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause: let the question finish, mark the moment for the
/// score, bring in a large "?" at `at` inside a gold ring that drains over
/// [`THINK_S`] with "pause for more time" under it; at zero the ring and the
/// note go. Returns the "?", which the scene clears with the answer.
pub fn think(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 80.0, Palette::hero(), at)?;
    let note = kit.text(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0], at[1] - THINK_R - 0.3))?;
    // the ring: what is left of the think time, clockwise from the top
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + THINK_R * a.cos(), at[1] + THINK_R * a.sin())
            })
            .collect()
    };
    let left = stage.add_value_tracker(1.0);
    let ring = stage
        .arena_mut()
        .add(polyline(arc(1.0)).style(line_style(Palette::hero(), 4.0)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(left).unwrap_or(0.0);
            if last == Some(f) {
                return;
            }
            let _ = s.set_points(ring, polyline(arc(f)).build().points());
            last = Some(f);
        },
        true,
    )?;
    fade_in_all(stage, &[anchor, ring, q, note], 0.2)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.2)?;
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MARKS, SECRET, TRUSTEES};

    #[test]
    fn sentences_start_after_their_stops() {
        assert_eq!(sentence_offsets("Five trustees. One vault."), vec![0, 15]);
        assert_eq!(sentence_offsets("Three."), vec![0]);
        let timed = timed_sentences("x", "Ab. Cd.", 7.0, &[]).unwrap();
        assert_eq!(timed, vec![(0, 0.0), (4, 4.0)]);
        assert!(timed_sentences("x", "Ab. Cd.", 7.0, &[0.0]).is_err());
    }

    /// Mark 0 sits at the top, marks run clockwise, and a whole turn of
    /// eleven marks comes back to the top.
    #[test]
    fn the_dial_runs_clockwise_from_the_top() {
        let top = on_dial(0.0, MARKS, DIAL_R);
        assert!((top[0] - DOOR_X).abs() < 1e-12 && (top[1] - (DIAL_Y + DIAL_R)).abs() < 1e-12);
        let quarter = on_dial(f64::from(MARKS) / 4.0, MARKS, DIAL_R);
        assert!(
            (quarter[0] - (DOOR_X + DIAL_R)).abs() < 1e-9,
            "a quarter turn is to the right"
        );
        let round = on_dial(f64::from(MARKS), MARKS, DIAL_R);
        assert!((round[0] - top[0]).abs() < 1e-9 && (round[1] - top[1]).abs() < 1e-9);
        // the secret's mark is right of the top, below the middle (5 of 11)
        let s = on_dial(f64::from(SECRET), MARKS, DIAL_R);
        assert!(s[0] > DOOR_X && s[1] < DIAL_Y);
    }

    /// The devices keep apart and inside the frame: the dial on the door,
    /// the lamp over it, the frames between the door and the rail, the strip
    /// right of the rail and inside the right edge.
    #[test]
    fn the_devices_fit() {
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const DOOR_TOP: f64 = DOOR_Y + DOOR_H / 2.0;
        const DOOR_BOTTOM: f64 = DOOR_Y - DOOR_H / 2.0;
        const {
            assert!(
                DIAL_Y + DIAL_R < DOOR_TOP - 0.2 && DIAL_Y - DIAL_R > DOOR_BOTTOM + 0.2,
                "the dial on the door"
            );
            assert!(DOOR_W / 2.0 > DIAL_R + 0.2, "the dial inside the door's sides");
            assert!(LAMP_Y - LAMP_R > DOOR_TOP + 0.1, "the lamp over the door");
            assert!(LAMP_Y + LAMP_R < FRAME_Y_RADIUS - 0.25, "the lamp inside the top edge");
            assert!(
                MARK_LABEL_R + 0.12 < TICK_IN && NEEDLE_R < MARK_LABEL_R - 0.12,
                "labels between ticks and needle"
            );
            assert!(FRAME_Y + FRAME_H / 2.0 < DOOR_BOTTOM - 0.2, "the frames under the door");
            assert!(
                FRAME_Y - FRAME_H / 2.0 > RAIL_Y + TRUSTEE_R + 0.2,
                "the frames over the discs"
            );
            assert!(
                RAIL_Y - TRUSTEE_R > BAND_TOP + 0.2,
                "the rail over the lower-third band"
            );
            assert!(
                RAIL_DX > FRAME_W + 0.2 && RAIL_DX > 2.0 * TRUSTEE_R + 0.2,
                "trustees apart"
            );
            assert!(
                DOOR_X - DOOR_W / 2.0 > -FRAME_X_RADIUS + 0.25,
                "the door inside the left edge"
            );
            assert!(DOOR_X + DOOR_W / 2.0 < STRIP_LEFT - 0.5, "the door left of the strip");
            assert!(STRIP_DX > TICK_W + 0.15, "ticks apart");
            assert!(
                STRIP_TITLE_Y - 0.25 > STRIP_Y + TICK_H / 2.0,
                "the title over the ticks"
            );
            assert!(
                STRIP_TITLE_Y + 0.25 < FRAME_Y_RADIUS - 0.25,
                "the title inside the top edge"
            );
            assert!(
                STRIP_CAPTION_Y + 0.2 < STRIP_Y - STRIP_LABEL_DY - 0.15,
                "the caption under the labels"
            );
        }
        let (left, right) = (trustee_at(1), trustee_at(TRUSTEES.len() as u32));
        assert!(left[0] - TRUSTEE_R > -FRAME_X_RADIUS + 0.25);
        assert!(right[0] + FRAME_W / 2.0 < STRIP_LEFT - 1.0);
        assert!(strip_right(MARKS) < FRAME_X_RADIUS - 0.25);
    }
}
