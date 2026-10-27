//! The video's look, shared by its chapters (storyboard, vocabulary and
//! color roles): the line of every number the company could have printed,
//! the game box, the think-time countdown, and line timing.
//!
//! Every position on the line comes from one map, [`x_at`]: number `n` on a
//! line shown from 1 to `end`. A scene binds its dots, boxes, ticks and
//! brackets to trackers (the line's `end` and the like) through that map,
//! so a zoom moves the whole picture by one rule. The binding moves and
//! reshapes mobjects that already exist, and only on a frame where a
//! tracker changed: nothing is redrawn every frame.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, short strokes.
//!
//! The playback and line-timing helpers are the resistor chapters' (accepted
//! centrally), kept to the ones this video uses.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, tween, v};
use fmn::prelude::*;

use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ----------------------------------------------------------------- the line

/// The strip the line is drawn on: its left end (number 1), its width and
/// its height on screen. Its right end is the last number shown.
pub const STRIP_X0: f64 = -6.3;
pub const STRIP_W: f64 = 12.45;
pub const STRIP_X1: f64 = STRIP_X0 + STRIP_W;
pub const STRIP_Y: f64 = -0.6;
/// A number's dot, and its opacity: even along the strip, brighter once it
/// is certain, faded at the strip's right end.
pub const DOT_R: f64 = 0.035;
pub const DOT_OPACITY: f64 = 0.55;
pub const LIT_OPACITY: f64 = 0.9;
pub const FADED: f64 = 0.06;
/// Where along the strip, as a share of its width, the dots begin to fade.
pub const FADE_FROM: f64 = 0.74;
/// The line beyond its dots: short pieces of a thin stroke.
pub const BASE_PIECES: usize = 16;
pub const BASE_W: f64 = 2.0;
/// A number's label under the line (or over it, for a crowded neighbour).
pub const LABEL_DY: f64 = 0.3;
pub const LABEL_SIZE: f64 = 20.0;
/// A bracket along the line, under its labels; its ends' ticks; a dashed
/// bracket's dashes and gaps.
pub const BRACKET_DY: f64 = 0.62;
pub const BRACKET_TICK: f64 = 0.1;
pub const BRACKET_W: f64 = 3.0;
pub const DASH: f64 = 0.16;
pub const DASH_GAP: f64 = 0.12;
/// Somewhere off the frame, for a piece not shown at the moment.
pub const OFF: [f64; 2] = [-20.0, -20.0];

/// Where number `n` sits on a line shown from 1 to `end`: 1 at the strip's
/// left end, `end` at its right.
pub fn x_at(n: f64, end: f64) -> f64 {
    STRIP_X0 + (n - 1.0) * STRIP_W / (end - 1.0)
}

/// A dot's opacity at `x`: even along the strip, fading out over its last
/// quarter, where nobody knows whether the line goes on.
pub fn dot_opacity(x: f64) -> f64 {
    let t = (x - STRIP_X0) / STRIP_W;
    if t <= FADE_FROM {
        DOT_OPACITY
    } else {
        DOT_OPACITY + (FADED - DOT_OPACITY) * ((t - FADE_FROM) / (1.0 - FADE_FROM)).min(1.0)
    }
}

/// Piece `k` of the line beyond number `last`, on a line shown to `end`:
/// its two ends, or `None` while no number past `last` is shown.
pub fn base_piece(k: usize, last: u64, end: f64) -> Option<(f64, f64)> {
    let from = x_at(last as f64 + 1.0, end);
    if from >= STRIP_X1 {
        return None;
    }
    let w = (STRIP_X1 - from) / BASE_PIECES as f64;
    Some((from + w * k as f64, from + w * (k + 1) as f64))
}

/// A bracket under the line from number `a` to number `b`, on a line shown
/// to `end`: a stroke with a short upward tick at each end.
pub fn bracket_points(a: f64, b: f64, end: f64) -> Vec<Vec3> {
    let y = STRIP_Y - BRACKET_DY;
    let (x0, x1) = (x_at(a, end), x_at(b, end));
    vec![v(x0, y + BRACKET_TICK), v(x0, y), v(x1, y), v(x1, y + BRACKET_TICK)]
}

/// The dashes of a dashed bracket from `from` to `to` (screen x), laid from
/// `to` leftwards so they keep their length and spacing while `from` moves;
/// the last one is cut at `from`.
pub fn dash_spans(from: f64, to: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut right = to;
    while right > from {
        out.push(((right - DASH).max(from), right));
        right -= DASH + DASH_GAP;
    }
    out
}

// ------------------------------------------------------------------ the box

/// A game box: a rounded body with a lid band across its top; the side
/// with the number has no lid band. On the line a box is [`SMALL`] of this.
pub const BOX_W: f64 = 1.15;
pub const BOX_H: f64 = 0.8;
pub const BOX_R: f64 = 0.1;
pub const LID_H: f64 = 0.22;
pub const BODY_O: f64 = 0.35;
pub const LID_O: f64 = 0.8;
pub const NUMBER_SIZE: f64 = 40.0;
pub const SMALL: f64 = 0.54;
/// A small box's centre above its number's dot, and the row it spreads into
/// when the line is shown too far out for five boxes to sit on their
/// numbers (the fan): its first centre and its step.
pub const BOX_RISE: f64 = 0.66;
pub const FAN_X0: f64 = -6.0;
pub const FAN_STEP: f64 = 0.72;
pub const TICK_W: f64 = 3.0;

/// A box's body and lid at `at`. Detached.
pub fn game_box(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let body = Rectangle::new()
        .width(BOX_W)
        .height(BOX_H)
        .corner_radius(BOX_R)
        .style(flat(palette::outline(), BODY_O))
        .build()?
        .moved_to(at);
    let lid = Rectangle::new()
        .width(BOX_W)
        .height(LID_H)
        .corner_radius(BOX_R)
        .style(flat(palette::outline(), LID_O))
        .build()?
        .moved_to(lid_spot(at));
    Ok([stage.arena_mut().add(body), stage.arena_mut().add(lid)])
}

/// The centre of the lid band of a box whose body is centred at `at`.
pub fn lid_spot(at: Vec3) -> Vec3 {
    v(at[0], at[1] + (BOX_H - LID_H) / 2.0)
}

/// Where the `i`th of the friends' boxes (number `n`) sits once it is small:
/// above its own number on a line shown to `end`, or, as `fan` goes from 0
/// to 1, in a row from the line's left end, so five numbers squeezed
/// together keep boxes that can still be read.
pub fn box_spot(i: usize, n: u64, end: f64, fan: f64) -> Vec3 {
    let on = x_at(n as f64, end);
    let spread = FAN_X0 + FAN_STEP * i as f64;
    v(on + (spread - on) * fan, STRIP_Y + BOX_RISE)
}

/// The middle of the bottom of a small box centred at `spot`, where its
/// tick meets it.
fn box_foot(spot: Vec3) -> Vec3 {
    v(spot[0], spot[1] - SMALL * BOX_H / 2.0)
}

/// The tick joining number `n`'s dot to its small box at `spot`, on a line
/// shown to `end`, grown to the share `rise` of its length: from the dot to
/// the middle of the box's bottom. Every small box sits at one height, above
/// the dots, so a tick stays below the boxes' common bottom and passes under
/// the other boxes however far they have spread; two ticks never cross,
/// since the boxes and their dots are in the same order. Ticks that met the
/// box where a line toward the dot leaves it ran out of its side and through
/// the neighbouring boxes while they spread (the opening preview, 73.5 s).
pub fn tick_ends(n: u64, end: f64, spot: Vec3, rise: f64) -> [Vec3; 2] {
    let dot = v(x_at(n as f64, end), STRIP_Y);
    let foot = box_foot(spot);
    let r = rise.clamp(0.0, 1.0);
    [dot, v(dot[0] + (foot[0] - dot[0]) * r, dot[1] + (foot[1] - dot[1]) * r)]
}

/// How far apart two boxes' outlines stay, at least, in every state a scene
/// passes through, not only where a move ends.
#[cfg(test)]
pub const BOX_CLEAR: f64 = 0.05;

/// Two boxes, each a centre and a scale (1 for a full-size box), at least
/// `BOX_CLEAR` apart across or up.
#[cfg(test)]
pub fn boxes_clear(a: (Vec3, f64), b: (Vec3, f64)) -> bool {
    let ((pa, sa), (pb, sb)) = (a, b);
    let across = (pa[0] - pb[0]).abs() - BOX_W * (sa + sb) / 2.0;
    let up = (pa[1] - pb[1]).abs() - BOX_H * (sa + sb) / 2.0;
    across >= BOX_CLEAR || up >= BOX_CLEAR
}

/// Every pair of `boxes` clear of each other.
#[cfg(test)]
pub fn all_clear(boxes: &[(Vec3, f64)]) -> bool {
    boxes
        .iter()
        .enumerate()
        .all(|(i, &a)| boxes[i + 1..].iter().all(|&b| boxes_clear(a, b)))
}

// ------------------------------------------------------------------ zooms

/// One leg of a change in what the line shows: the small boxes spreading
/// into their row or coming back onto their numbers (`Fan`, to its new
/// value) with the line's end fixed, or the end moving (`End`) with the
/// boxes where they are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Leg {
    Fan(f64),
    End(f64),
}

/// The legs from the view `from` to the view `to`, each `[end, fan]`. The
/// line's end moves only while the boxes are in their row (fan 1), where
/// their places do not depend on it: the boxes spread before the line
/// squeezes and come back after it stretches. Between two views at one
/// end, every box slides along its row at one pace (one tracker drives
/// them all), and boxes listed in the order of their numbers stay in that
/// order (over their numbers and in the row alike), so two boxes apart at
/// both ends of a leg are apart all along it. Moving the end and the fan
/// together squeezed the boxes into each other before they had spread (the
/// first opening preview, 01_8). The legs cannot help a view that is itself
/// crowded: a scene uses only views its tests show clear.
pub fn zoom_legs(from: [f64; 2], to: [f64; 2]) -> Vec<Leg> {
    let mut legs = Vec::new();
    if from[0].total_cmp(&to[0]).is_ne() {
        if from[1] < 1.0 {
            legs.push(Leg::Fan(1.0));
        }
        legs.push(Leg::End(to[0]));
        if to[1] < 1.0 {
            legs.push(Leg::Fan(to[1]));
        }
    } else if from[1].total_cmp(&to[1]).is_ne() {
        legs.push(Leg::Fan(to[1]));
    }
    legs
}

/// Every view the legs pass through from `from`, `steps` to a leg, each leg
/// ending exactly on its target as a tween does. A tween's rate function
/// changes when each view is reached, not which views are, since it stays
/// between 0 and 1.
#[cfg(test)]
pub fn leg_views(from: [f64; 2], legs: &[Leg], steps: usize) -> Vec<[f64; 2]> {
    let mut at = from;
    let mut out = vec![at];
    for &leg in legs {
        let (k, target) = match leg {
            Leg::End(e) => (0, e),
            Leg::Fan(f) => (1, f),
        };
        let start = at[k];
        for s in 1..steps {
            at[k] = start + (target - start) * s as f64 / steps as f64;
            out.push(at);
        }
        at[k] = target;
        out.push(at);
    }
    out
}

/// Play `legs` one after another on the trackers `end` and `fan`: a fan
/// leg in `fan_s` seconds, an end leg in `end_s`.
pub fn play_legs(stage: &mut Stage<'_>, end: Mob, fan: Mob, legs: &[Leg], fan_s: f64, end_s: f64) -> fmn::Result<()> {
    for &leg in legs {
        let a = match leg {
            Leg::End(e) => tween(end, e, end_s)?,
            Leg::Fan(f) => tween(fan, f, fan_s)?,
        };
        stage.play(a)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill, at opacity `o`.
pub fn line_style(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// A stroke through `points`, detached.
pub fn stroke(stage: &mut Stage<'_>, points: Vec<Vec3>, style: Style) -> Mob {
    stage.arena_mut().add(polyline(points).style(style).build())
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

/// Flash several mobjects (the highlight role), together.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Flash mobjects one after another, the whole taking `rt` (a ripple along
/// the line).
pub fn flash_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.6, None)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Scale each mobject by `factor` and move it to its point, together, over
/// `rt` seconds (`factor` 1 just moves it).
pub fn move_all(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], factor: f64, rt: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, to) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .scale(factor)?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Stretch each mobject's height by `factor` about its centre, together,
/// over `rt` seconds (half of a box turning over).
pub fn squash_all(stage: &mut Stage<'_>, mobs: &[Mob], factor: f64, rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .stretch(factor, 1)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Turn each mobject to its color and opacity, together, over `rt` seconds.
pub fn refill(stage: &mut Stage<'_>, mobs: &[(Mob, Srgb, f64)], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, color, opacity) in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_color_with_opacity(color, opacity)?;
        anims.push(stage.prepare(a)?);
    }
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

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the phrase it illustrates. A
/// phrase's moment is its sentence's start, as the narrator measured it,
/// plus its share of that sentence's characters. That share is an
/// approximate draft cue, not measured word timing: before the narrated
/// version claims a number flip or reveal (01_4's five numbers) lands on its
/// word, those moments must come from the qualified FrankenWhisper word
/// alignment.
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
    /// When `phrase` begins (scene seconds), interpolated by character
    /// position inside its sentence: a draft cue, not a measured word onset.
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
        .add(polyline(arc(1.0)).style(line_style(Palette::hero(), 4.0, 1.0)).build());
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
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::model::{BIG, FRIENDS, LINE_DOTS};

    const NEAR: f64 = 1e-12;

    /// Number 1 sits at the strip's left end and the last number shown at
    /// its right, whatever the line shows; numbers keep their order; the
    /// opening line's step is the storyboard's 0.15.
    #[test]
    fn the_line_maps_every_number_by_one_rule() {
        for end in [LINE_DOTS as f64, BIG as f64] {
            assert!((x_at(1.0, end) - STRIP_X0).abs() < NEAR);
            assert!((x_at(end, end) - STRIP_X1).abs() < NEAR);
            for n in 1..LINE_DOTS {
                assert!(x_at(n as f64, end) < x_at(n as f64 + 1.0, end));
            }
        }
        assert!((x_at(2.0, LINE_DOTS as f64) - x_at(1.0, LINE_DOTS as f64) - 0.15).abs() < 1e-9);
        const { assert!(STRIP_X0 > -(FRAME_X_RADIUS - 0.25) && STRIP_X1 < FRAME_X_RADIUS - 0.25) };
    }

    /// The dots are even up to number 62 of the opening 84 and fade to
    /// almost nothing at 84, never brightening on the way.
    #[test]
    fn the_dots_fade_only_at_the_right_end() {
        let end = LINE_DOTS as f64;
        let o = |n: u64| dot_opacity(x_at(n as f64, end));
        for n in 1..=62 {
            assert_eq!(o(n), DOT_OPACITY, "number {n}");
        }
        assert!((o(LINE_DOTS) - FADED).abs() < NEAR);
        for n in 1..LINE_DOTS {
            assert!(o(n + 1) <= o(n));
        }
        // shown to a thousand, the opening's dots are all far from the end
        assert!((1..=LINE_DOTS).all(|n| dot_opacity(x_at(n as f64, BIG as f64)) == DOT_OPACITY));
    }

    /// The line past the dots covers the rest of the strip in pieces end to
    /// end, and is not there while nothing past the dots is shown.
    #[test]
    fn the_line_past_the_dots_fills_the_strip() {
        assert_eq!(base_piece(0, LINE_DOTS, LINE_DOTS as f64), None);
        let end = BIG as f64;
        let pieces: Vec<(f64, f64)> = (0..BASE_PIECES)
            .map(|k| base_piece(k, LINE_DOTS, end).unwrap())
            .collect();
        assert!((pieces[0].0 - x_at(LINE_DOTS as f64 + 1.0, end)).abs() < NEAR);
        assert!((pieces[BASE_PIECES - 1].1 - STRIP_X1).abs() < 1e-9);
        for w in pieces.windows(2) {
            assert!((w[0].1 - w[1].0).abs() < 1e-9 && w[0].0 < w[0].1);
        }
    }

    /// A dashed bracket's dashes stay inside it, in order, apart, and the
    /// same length except the one cut at its left end; it has more dashes
    /// the longer it is, and none when it is empty.
    #[test]
    fn dashes_keep_their_size_as_the_bracket_grows() {
        let short = dash_spans(x_at(61.0, LINE_DOTS as f64), STRIP_X1);
        let long = dash_spans(x_at(61.0, BIG as f64), STRIP_X1);
        assert!(short.len() < long.len());
        assert!(dash_spans(STRIP_X1, STRIP_X1).is_empty());
        for spans in [&short, &long] {
            assert!((spans[0].1 - STRIP_X1).abs() < NEAR);
            for (i, &(a, b)) in spans.iter().enumerate() {
                assert!(a < b && b - a <= DASH + NEAR);
                if i + 1 < spans.len() {
                    assert!((b - a - DASH).abs() < 1e-9);
                    assert!((a - spans[i + 1].1 - DASH_GAP).abs() < 1e-9);
                }
            }
        }
        // the bracket starts where its first number is, never further left
        let from = x_at(61.0, BIG as f64);
        assert!(long.last().unwrap().0 >= from - NEAR);
    }

    /// A tick runs from its dot to the middle of its box's bottom: straight
    /// up while the box is over its number, slanting once the box has moved
    /// aside; half grown, it is half as long.
    #[test]
    fn a_tick_joins_its_dot_to_its_box() {
        let end = LINE_DOTS as f64;
        let bottom = |spot: Vec3| spot[1] - SMALL * BOX_H / 2.0;
        for (i, &n) in FRIENDS.iter().enumerate() {
            let spot = box_spot(i, n, end, 0.0);
            let [dot, top] = tick_ends(n, end, spot, 1.0);
            assert!((dot[0] - top[0]).abs() < NEAR, "straight up");
            assert!((top[1] - bottom(spot)).abs() < NEAR, "to the box's bottom");
            let [_, mid] = tick_ends(n, end, spot, 0.5);
            assert!((mid[1] - (dot[1] + top[1]) / 2.0).abs() < NEAR);
            // spread out over a line shown to a thousand: from its own dot to
            // the middle of the box's bottom
            let spot = box_spot(i, n, BIG as f64, 1.0);
            let [dot, foot] = tick_ends(n, BIG as f64, spot, 1.0);
            assert!((dot[0] - x_at(n as f64, BIG as f64)).abs() < NEAR && dot[1] == STRIP_Y);
            assert!((foot[0] - spot[0]).abs() < NEAR && (foot[1] - bottom(spot)).abs() < NEAR);
        }
    }

    /// Whether the segment from `a` to `b` passes through the inside of the
    /// small box centred at `c` (touching its outline does not count):
    /// Liang–Barsky clipping against the box shrunk by `NEAR`.
    fn enters_box(a: Vec3, b: Vec3, c: Vec3) -> bool {
        let half = [SMALL * BOX_W / 2.0 - NEAR, SMALL * BOX_H / 2.0 - NEAR];
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
        for (p, q) in [
            (-dx, a[0] - (c[0] - half[0])),
            (dx, c[0] + half[0] - a[0]),
            (-dy, a[1] - (c[1] - half[1])),
            (dy, c[1] + half[1] - a[1]),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else if p < 0.0 {
                t0 = t0.max(q / p);
            } else {
                t1 = t1.min(q / p);
            }
        }
        t0 < t1
    }

    /// The old tick's end, kept as the negative control: where a line from
    /// the box's centre toward the dot leaves the small box.
    fn side_exit(centre: Vec3, toward: Vec3) -> Vec3 {
        let half = [SMALL * BOX_W / 2.0, SMALL * BOX_H / 2.0];
        let (dx, dy) = (toward[0] - centre[0], toward[1] - centre[1]);
        let reach = |d: f64, h: f64| if d.abs() > 1e-12 { h / d.abs() } else { f64::INFINITY };
        let s = reach(dx, half[0]).min(reach(dy, half[1])).min(1.0);
        v(centre[0] + dx * s, centre[1] + dy * s)
    }

    /// The clipping test at its edges: through a box, beside it, touching
    /// its outline, ending inside it.
    #[test]
    fn a_segment_enters_a_box_only_through_its_inside() {
        let c = v(0.0, 0.0);
        let (w, h) = (SMALL * BOX_W / 2.0, SMALL * BOX_H / 2.0);
        assert!(enters_box(v(-1.0, 0.0), v(1.0, 0.0), c), "straight through");
        assert!(!enters_box(v(-1.0, h + 0.01), v(1.0, h + 0.01), c), "passing above");
        assert!(!enters_box(v(0.0, -1.0), v(0.0, -h), c), "up to its bottom");
        assert!(!enters_box(v(-1.0, -h), v(1.0, -h), c), "along its bottom");
        assert!(enters_box(v(0.0, -1.0), v(0.0, 0.0), c), "into its middle");
        assert!(enters_box(v(-w - 0.5, -h - 0.1), v(0.0, h), c), "slanting in");
    }

    /// Ticks pass under every box in every state the line goes through: the
    /// legs between the opening view and the view of a thousand, both ways,
    /// and the ticks growing on the opening line. No tick enters any box, its
    /// own included, and no two ticks cross. The negative control is the old
    /// tick, which met its box where a line toward the dot leaves it: part way
    /// through the spread it runs through a neighbour's body (the opening
    /// preview, 73.5 s: 51's tick through 60).
    #[test]
    fn ticks_pass_under_every_box() {
        let (near, far) = (LINE_DOTS as f64, BIG as f64);
        let steps = 400;
        let mut states: Vec<([f64; 2], f64)> = Vec::new();
        for (from, to) in [([near, 0.0], [far, 1.0]), ([far, 1.0], [near, 0.0])] {
            states.extend(
                leg_views(from, &zoom_legs(from, to), steps)
                    .into_iter()
                    .map(|view| (view, 1.0)),
            );
        }
        states.extend((0..=steps).map(|s| ([near, 0.0], s as f64 / steps as f64)));
        let mut old_crossed = false;
        for &([end, fan], rise) in &states {
            let spots: Vec<Vec3> = FRIENDS
                .iter()
                .enumerate()
                .map(|(i, &n)| box_spot(i, n, end, fan))
                .collect();
            let ticks: Vec<[Vec3; 2]> = FRIENDS
                .iter()
                .zip(&spots)
                .map(|(&n, &spot)| tick_ends(n, end, spot, rise))
                .collect();
            for (i, &[dot, tip]) in ticks.iter().enumerate() {
                for (j, &c) in spots.iter().enumerate() {
                    assert!(
                        !enters_box(dot, tip, c),
                        "tick {} through box {} at end {end}, fan {fan}, rise {rise}",
                        FRIENDS[i],
                        FRIENDS[j]
                    );
                }
                let old = side_exit(spots[i], dot);
                old_crossed |= spots
                    .iter()
                    .enumerate()
                    .any(|(j, &c)| j != i && enters_box(dot, old, c));
            }
            // in the same order at the dots and at the boxes: no two cross
            for w in ticks.windows(2) {
                assert!(
                    w[0][0][0] < w[1][0][0] && w[0][1][0] < w[1][1][0],
                    "{end}, {fan}, {rise}"
                );
            }
        }
        assert!(old_crossed, "the preview's ticks ran through a neighbouring box");
    }

    /// Small boxes on their numbers clear each other on the opening line,
    /// and so do the fanned boxes; both stay in the title-safe area, above
    /// a three-line lower-third, and the fan starts at the line's left.
    #[test]
    fn the_boxes_never_overlap() {
        let w = SMALL * BOX_W;
        let end = LINE_DOTS as f64;
        for fan in [0.0, 1.0] {
            let end = if fan == 0.0 { end } else { BIG as f64 };
            let spots: Vec<Vec3> = FRIENDS
                .iter()
                .enumerate()
                .map(|(i, &n)| box_spot(i, n, end, fan))
                .collect();
            for p in spots.windows(2) {
                assert!(p[1][0] - p[0][0] > w + 0.05, "{fan}: {p:?}");
            }
            assert!(spots[0][0] - w / 2.0 > -(FRAME_X_RADIUS - 0.25));
        }
        const { assert!(STRIP_Y + BOX_RISE - SMALL * BOX_H / 2.0 > STRIP_Y + DOT_R + 0.3) };
        assert!((FAN_X0 - STRIP_X0).abs() < 0.5);
        let lowest = STRIP_Y - BRACKET_DY - 0.35 - 0.2;
        assert!(lowest > -FRAME_Y_RADIUS + 1.58, "above a lower-third");
    }

    /// Two boxes are clear when their outlines are `BOX_CLEAR` apart across
    /// or up, and not when they come closer both ways; scale counts.
    #[test]
    fn boxes_are_clear_only_with_room_between_them() {
        let (w, h) = (BOX_W * SMALL, BOX_H * SMALL);
        let at = |x: f64, y: f64| (v(x, y), SMALL);
        assert!(boxes_clear(at(0.0, 0.0), at(w + BOX_CLEAR + NEAR, 0.0)));
        assert!(!boxes_clear(at(0.0, 0.0), at(w + BOX_CLEAR - 0.01, 0.0)));
        assert!(boxes_clear(at(0.0, 0.0), at(0.0, h + BOX_CLEAR + NEAR)));
        // diagonal neighbours whose corners overlap both ways
        assert!(!boxes_clear(at(0.0, 0.0), at(w - 0.1, h - 0.1)));
        // a full-size box needs more room than a small one
        assert!(!boxes_clear((v(0.0, 0.0), 1.0), at(w + BOX_CLEAR, 0.0)));
        assert!(all_clear(&[at(0.0, 0.0), at(1.0, 0.0), at(2.0, 0.0)]));
        assert!(
            !all_clear(&[at(0.0, 0.0), at(1.0, 0.0), at(0.1, 0.0)]),
            "the first and third"
        );
    }

    /// The end moves only while the boxes are in their row: out, the boxes
    /// spread first; back in, they come home last; a change of one tracker
    /// is one leg; no change, no legs. Every leg ends where the next begins,
    /// and the last view is the target.
    #[test]
    fn the_line_moves_only_under_spread_boxes() {
        let (near, far) = (LINE_DOTS as f64, BIG as f64);
        assert_eq!(zoom_legs([near, 0.0], [far, 1.0]), [Leg::Fan(1.0), Leg::End(far)]);
        assert_eq!(zoom_legs([far, 1.0], [near, 0.0]), [Leg::End(near), Leg::Fan(0.0)]);
        assert_eq!(zoom_legs([near, 0.0], [near, 1.0]), [Leg::Fan(1.0)]);
        assert_eq!(zoom_legs([far, 1.0], [near, 1.0]), [Leg::End(near)]);
        assert!(zoom_legs([near, 0.0], [near, 0.0]).is_empty());
        for (from, to) in [
            ([near, 0.0], [far, 1.0]),
            ([far, 1.0], [near, 0.0]),
            ([near, 0.3], [far, 0.6]),
        ] {
            let views = leg_views(from, &zoom_legs(from, to), 50);
            assert_eq!(views[0], from);
            assert_eq!(*views.last().unwrap(), to);
            // wherever the end is not at either of its ends, the boxes are spread
            for &[end, fan] in &views {
                if end != from[0] && end != to[0] {
                    assert_eq!(fan, 1.0, "{from:?} -> {to:?} at {end}");
                }
            }
        }
    }

    #[test]
    fn sentences_split_like_the_narrator() {
        assert_eq!(sentence_offsets("One. Two? Three."), [0, 5, 10]);
        let timed = timed_sentences("x", "Ab. Cd.", 7.0, &[]).unwrap();
        assert_eq!(timed[0], (0, 0.0));
        assert!(timed_sentences("x", "Ab. Cd.", 7.0, &[0.0]).is_err());
    }
}
