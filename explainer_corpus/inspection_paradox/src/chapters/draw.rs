//! The video's look, shared by its chapters (storyboard, color roles): the
//! hour as a line, the buses standing on it, you and the passengers as
//! dots, the think-time countdown, and line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups. The playback and timing helpers follow freivalds'
//! `draw.rs`, the corpus's reviewed pattern.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Facts, Kit, Palette, Script, Timed, rect, v};
use fmn::prelude::*;

use crate::model::{HOUR, Q};
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// A fact's printed value, refused when the facts lack the key.
pub fn fact(f: &Facts, key: &str) -> fmn::Result<String> {
    f.get(key)
        .map(str::to_owned)
        .ok_or_else(|| fail(format!("no fact {key}")))
}

/// "4:20" as the screen spells a duration out: "4 min 20 s", "3 min",
/// "30 s".
pub fn min_s(clock: &str) -> fmn::Result<String> {
    let (m, s) = clock
        .split_once(':')
        .ok_or_else(|| fail(format!("{clock} is not m:ss")))?;
    Ok(match (m, s) {
        ("0", "00") => "0 s".to_owned(),
        ("0", s) => format!("{} s", s.trim_start_matches('0')),
        (m, "00") => format!("{m} min"),
        (m, s) => format!("{m} min {} s", s.trim_start_matches('0')),
    })
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

/// Seconds a card needs on screen to be read: fifteen characters a second,
/// a quick reader (the video can be paused), counted from its completed
/// fade-in.
pub fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// Scene time now.
pub fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

// ---------------------------------------------------------------- the hour

/// The hour line: where it runs and how high it sits.
pub const LINE_X0: f64 = -6.0;
pub const LINE_X1: f64 = 6.0;
pub const LINE_Y: f64 = 0.6;
/// A bus marker's size: a short bar standing on the line.
const MARKER_W: f64 = 0.14;
const MARKER_H: f64 = 0.42;
/// The clock labels under the line, and their size.
const LABEL_DY: f64 = 0.32;
const LABEL_SIZE: f64 = 18.0;
const TICK_H: f64 = 0.12;
/// You: an ink dot with a ring in your color; a passenger is a smaller dot.
pub const YOU_R: f64 = 0.09;
const YOU_RING_R: f64 = 0.16;
pub const PASSENGER_R: f64 = 0.06;

/// Where moment `t` (minutes past the hour) sits on the line.
pub fn x_of(t: Q) -> f64 {
    LINE_X0 + (LINE_X1 - LINE_X0) * t.to_f64() / HOUR as f64
}

/// The hour line with a tick and a clock label at each of `marks` (minutes
/// past the hour, ":00" … ":60"). Detached: the axis first, then ticks and
/// labels.
pub struct HourLine {
    pub axis: Mob,
    pub ticks: Vec<Mob>,
    pub labels: Vec<Mob>,
}

impl HourLine {
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.axis];
        out.extend(&self.ticks);
        out.extend(&self.labels);
        out
    }
}

pub fn hour_line(kit: &Kit, stage: &mut Stage<'_>, marks: &[i64]) -> fmn::Result<HourLine> {
    let axis = stage.arena_mut().add(
        Line::new(v(LINE_X0, LINE_Y), v(LINE_X1, LINE_Y))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let mut ticks = Vec::with_capacity(marks.len());
    let mut labels = Vec::with_capacity(marks.len());
    for &m in marks {
        let x = x_of(Q::int(m));
        ticks.push(
            stage.arena_mut().add(
                Line::new(v(x, LINE_Y - TICK_H / 2.0), v(x, LINE_Y + TICK_H / 2.0))
                    .style(line_style(Palette::dim(), 2.0))
                    .build()?,
            ),
        );
        labels.push(kit.text(
            stage,
            &format!(":{m:02}"),
            LABEL_SIZE,
            Palette::dim(),
            v(x, LINE_Y - LABEL_DY),
        )?);
    }
    Ok(HourLine { axis, ticks, labels })
}

/// A bus at minute `t`: a short bar in its route's color standing on the line.
pub fn bus_marker(stage: &mut Stage<'_>, t: i64, color: Srgb) -> fmn::Result<Mob> {
    let x = x_of(Q::int(t));
    let r = rect(
        v(x - MARKER_W / 2.0, LINE_Y + 0.02),
        MARKER_W,
        MARKER_H,
        flat(color, 1.0),
    )?;
    Ok(stage.arena_mut().add(r))
}

/// The top of a bus marker at minute `t` (where its tag or tick sits).
pub fn marker_top(t: i64) -> Vec3 {
    v(x_of(Q::int(t)), LINE_Y + 0.02 + MARKER_H)
}

/// You at `at`: an ink dot inside a ring in your color, as one group.
pub fn you(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let dot = stage.arena_mut().add(
        Dot::new()
            .point(at)
            .radius(YOU_R)
            .style(flat(Palette::ink(), 1.0))
            .build(),
    );
    let ring = stage.arena_mut().add(
        Circle::new()
            .radius(YOU_RING_R)
            .arc_center(at)
            .style(line_style(palette::you(), 3.0))
            .build(),
    );
    group(stage, &[dot, ring])
}

/// An empty parent in the arena holding mobjects already there (the
/// bakery_algorithm helper): move or fade the parent, and the members go
/// with it.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

// --------------------------------------------------------------- playback

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

/// Indicate (flash) several mobjects together, `rt` seconds.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Indicate mobjects one after another, the whole taking `rt`.
pub fn flash_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Move a mobject so its centre lands on `to`, over `rt` seconds.
pub fn fly(mob: Mob, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// Scale a mobject by `k` about its centre while its centre moves to `to`.
pub fn shrink_to(mob: Mob, k: f64, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(k)?
        .move_to(to, ORIGIN)?)
}

/// Fade a mobject to `opacity` over `rt` seconds.
pub fn fade_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

/// A tracker run to `value` at a steady rate over `rt` seconds (a
/// countdown: the wait shrinks at one minute a minute, sped up). Not
/// `explainer_kit::tween`: that sets its own smooth rate and a chain takes
/// its arguments once, so it cannot be made linear, and a wait that eases in
/// and out would teach the wrong slope (chapter 05's triangles are this
/// steady fall). Smooth tracker moves use `tween`.
pub fn steady(tracker: Mob, value: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// A silent hold of `seconds`, whole frames.
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

/// Hold until scene time `t` (nothing if it has passed).
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

/// The least a finished picture stands before a chapter's fade.
pub const STAND_UP: f64 = 2.6;

/// Before a chapter's fade: the last line and its breath (`Kit::hold`), and
/// at least [`STAND_UP`] since `up`, when its last reveal became whole.
pub fn stand(kit: &Kit, stage: &mut Stage<'_>, up: f64) -> fmn::Result<()> {
    kit.hold(stage)?;
    wait_until(stage, up + STAND_UP)
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the phrase it illustrates. A
/// phrase's moment is its sentence's start, as the narrator measured it,
/// plus its share of that sentence's characters.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    /// Each sentence's first character and its start, seconds into the line.
    sentences: Vec<(usize, f64)>,
}

/// Where each sentence of `text` starts, by character.
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

/// Speak line `id` now (`Kit::say`) and return when its phrases fall.
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
}

// ------------------------------------------------------------- think time

/// How long the countdown ring runs before the answer, and its radius.
pub const THINK_S: f64 = 4.0;
const THINK_R: f64 = 0.62;
pub const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause: after the question's line, a "?" in your color
/// inside a ring that drains clockwise over [`THINK_S`], with "pause for
/// more time" under it. Returns the "?" (still on screen); the ring and the
/// note are gone.
pub fn think(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 80.0, palette::you(), at)?;
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
        .add(polyline(arc(1.0)).style(line_style(palette::you(), 4.0)).build());
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
    stage.play(steady(left, 0.0, THINK_S)?)?;
    fade_out_all(stage, &[anchor, ring, note], 0.2)?;
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_spell_out_as_the_screen_writes_them() {
        assert_eq!(min_s("4:20").unwrap(), "4 min 20 s");
        assert_eq!(min_s("3:00").unwrap(), "3 min");
        assert_eq!(min_s("0:30").unwrap(), "30 s");
        assert_eq!(min_s("8:40").unwrap(), "8 min 40 s");
        assert_eq!(min_s("1:05").unwrap(), "1 min 5 s");
        assert!(min_s("4.33").is_err());
    }

    #[test]
    fn the_hour_line_runs_its_full_width() {
        assert_eq!(x_of(Q::int(0)), LINE_X0);
        assert_eq!(x_of(Q::int(HOUR)), LINE_X1);
        assert!(x_of(Q::new(23, 2)) < x_of(Q::int(12)));
    }

    #[test]
    fn sentences_split_where_the_narrator_splits_them() {
        assert_eq!(sentence_offsets("One. Two? Three."), vec![0, 5, 10]);
    }
}
