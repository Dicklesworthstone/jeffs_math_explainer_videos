//! The video's look, shared by its chapters (storyboard §3, colour roles):
//! the people of a clinic in two hues (the disease) and two brightnesses (the
//! result), the survey's teal, cards and chips, and the timing helpers that
//! land a picture on the phrase it illustrates.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no outline
//! on big filled shapes, small groups.

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, hex, v};
use fmn::prelude::*;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began and how long it runs, so a picture
/// can land on the phrase it illustrates. A phrase's moment is its share of
/// the line's characters (the voice reads at a nearly even rate); the cue
/// sheets of the narrated draft check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
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
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
    })
}

impl Spoken {
    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let i = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let before = self.text[..i].chars().count() as f64;
        let all = self.text.chars().count().max(1) as f64;
        Ok(self.start + self.len * before / all)
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }
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

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown; the kit's own pause is 2 s).
pub const THINK_S: f64 = 4.0;
const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause with think time: let the question finish, mark the
/// moment for the score, draw a gold ring of radius `r` around `at` (the
/// question on screen sits inside it) that drains over [`THINK_S`], with
/// "pause for more time" beside it; at zero the ring and the note go.
pub fn think_pause(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<()> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let note = kit.text_left(stage, PAUSE_NOTE, 18.0, Palette::dim(), v(at[0] + r + 0.2, at[1]))?;
    // the ring: what is left of the think time, clockwise from the top
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + r * a.cos(), at[1] + r * a.sin())
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
    fade_in_all(stage, &[anchor, ring, note], 0.3)?;
    let drain = left
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(THINK_S),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(0.0)?;
    stage.play(drain)?;
    fade_out_all(stage, &[anchor, ring, note], 0.25)
}

// ------------------------------------------------------------- colour roles

/// A person with the disease (a positive result, or not yet known).
pub fn disease() -> Srgb {
    hex("#F0AC5F")
}

/// A person with the disease and a negative result (or none shown yet).
pub fn disease_dark() -> Srgb {
    hex("#7A5A2C")
}

/// A person without the disease and a positive result: a false alarm.
pub fn healthy() -> Srgb {
    hex("#58C4DD")
}

/// A person without the disease and a negative result (or none shown yet).
pub fn healthy_dark() -> Srgb {
    hex("#23425A")
}

/// A person before anything about them is known.
pub fn person() -> Srgb {
    hex("#6B7280")
}

/// The place of a person who has left the array for the block of positives.
pub fn vacated() -> Srgb {
    hex("#1C222C")
}

/// The survey's measured numbers.
pub fn survey() -> Srgb {
    Palette::teal()
}

/// The treatment as the coin assigned it (chapter 03 on): the ring on a
/// treated patient, and a pair lit because its treated patient recovered
/// first.
pub fn treated() -> Srgb {
    hex("#C59BF2")
}

/// A wrong reading: the tempting answer, the disputed key.
pub fn warn() -> Srgb {
    Palette::red()
}

/// A card's back.
pub fn panel() -> Srgb {
    hex("#1A2130")
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

/// An empty parent in the arena holding mobjects already there.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring detached mobjects in one after another (a lagged reveal), the whole
/// taking `rt` seconds.
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

// ---------------------------------------------------------- cards and chips

/// A rounded back of `w` × `h` centred at `at`.
fn back(w: f64, h: f64, at: Vec3, edge: Srgb) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(edge, 1.5, 1.0))
        .build()?
        .moved_to(at))
}

/// Words on a card `width` wide centred at `at`: `head` small and dim on
/// top (if any), then `text` wrapped to `chars` a line in `color`, then
/// `foot` small and dim (if any). One group, detached.
#[allow(clippy::too_many_arguments)]
pub fn card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    head: Option<&str>,
    text: &str,
    foot: Option<&str>,
    (size, chars): (f64, usize),
    color: Srgb,
    width: f64,
    at: Vec3,
) -> fmn::Result<Mob> {
    let lines = wrap(text, chars, 4);
    let (line_h, small_h) = (size * 0.0145, 0.34);
    let h = 0.4
        + line_h * lines.len() as f64
        + if head.is_some() { small_h } else { 0.0 }
        + if foot.is_some() { small_h } else { 0.0 };
    let card_back = stage.arena_mut().add(back(width, h, at, Palette::faint())?);
    let left = at[0] - width / 2.0 + 0.25;
    let mut y = at[1] + h / 2.0 - 0.2;
    let mut all = vec![card_back];
    if let Some(head) = head {
        y -= small_h / 2.0;
        all.push(kit.text_left(stage, head, 20.0, Palette::dim(), v(left, y))?);
        y -= small_h / 2.0;
    }
    for l in &lines {
        y -= line_h / 2.0;
        all.push(kit.text_left(stage, l, size, color, v(left, y))?);
        y -= line_h / 2.0;
    }
    if let Some(foot) = foot {
        y -= small_h / 2.0;
        all.push(kit.text_left(stage, foot, 18.0, Palette::dim(), v(left, y))?);
    }
    group(stage, &all)
}

/// A few words on a dark chip edged in `color`, the words in `color` too:
/// a label that reads over anything. One group, detached.
pub fn chip(kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let t = kit.text(stage, words, size, color, at)?;
    let (w, h) = (stage.get_width(t), stage.get_height(t));
    let b = stage.arena_mut().add(back(w + 0.32, h + 0.24, at, color)?);
    group(stage, &[b, t])
}
