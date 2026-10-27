//! The video's look, shared by its chapters (storyboard, color roles and
//! geometry): the card and what is drawn on it in card centimetres, curves
//! drawn by arc length from a tracker, the walking dividers, the zoom, the
//! think-time countdown, and line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Anything that moves
//! with a tracker is driven by an updater on an empty anchor, which must be
//! in the scene for the updater to run, and is stopped with [`stop`] before
//! the shape is moved or zoomed by animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, v};
use fmn::prelude::*;

use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Curve, P2};
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

/// Scene time now.
pub fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

// ---------------------------------------------------------------- the card

/// How far apart, along an arc, the points of a drawn curve are, cm.
pub const EVERY: f64 = 0.04;

/// A card point (cm, the card's centre at 0) in the frame.
pub fn card_point(p: P2) -> Vec3 {
    v(CARD_CENTRE[0] + CARD_UNIT * p[0], CARD_CENTRE[1] + CARD_UNIT * p[1])
}

/// The card: a panel a little lighter than the background. Detached.
pub fn card_panel(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(CARD_W * CARD_UNIT)
        .height(CARD_H * CARD_UNIT)
        .corner_radius(0.12)
        .style(flat(palette::card(), 0.55))
        .build()?
        .moved_to(card_point([0.0, 0.0]));
    Ok(stage.arena_mut().add(r))
}

/// Card points as one open polyline in `color`. Detached.
pub fn path(stage: &mut Stage<'_>, points: &[P2], color: Srgb, width: f64) -> Mob {
    let pts: Vec<Vec3> = points.iter().map(|&p| card_point(p)).collect();
    stage
        .arena_mut()
        .add(polyline(pts).style(line_style(color, width)).build())
}

/// A filled dot at a card point. Detached.
pub fn dot(stage: &mut Stage<'_>, at: P2, r: f64, color: Srgb) -> Mob {
    let d = Circle::new()
        .radius(r)
        .arc_center(card_point(at))
        .style(flat(color, 1.0))
        .build();
    stage.arena_mut().add(d)
}

/// A dashed line between two card points. Detached.
pub fn dashed(stage: &mut Stage<'_>, from: P2, to: P2, color: Srgb) -> fmn::Result<Mob> {
    let d = DashedLine::new(card_point(from), card_point(to))
        .dash_length(0.12)
        .style(line_style(color, 2.0))
        .build()?;
    Ok(stage.arena_mut().add(d))
}

// ------------------------------------------------ drawn and moved by trackers

/// `curve` drawn from its start to the share of its length `tracker` holds
/// (0 to 1), redrawn whenever the tracker moves: [anchor, line]. Detached.
pub fn traced(stage: &mut Stage<'_>, curve: &Curve, tracker: Mob, color: Srgb, width: f64) -> fmn::Result<[Mob; 2]> {
    let curve = curve.clone();
    let points = move |f: f64| -> Vec<Vec3> {
        curve
            .upto(f * curve.length, EVERY)
            .into_iter()
            .map(card_point)
            .collect()
    };
    let line = stage
        .arena_mut()
        .add(polyline(points(0.0)).style(line_style(color, width)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(tracker).unwrap_or(0.0);
            if last == Some(f) {
                return;
            }
            let _ = s.set_points(line, polyline(points(f)).build().points());
            last = Some(f);
        },
        true,
    )?;
    Ok([anchor, line])
}

/// A dot riding `curve` at the share of its length `tracker` holds:
/// [anchor, dot]. Detached.
pub fn rider(stage: &mut Stage<'_>, curve: &Curve, tracker: Mob, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let curve = curve.clone();
    let d = dot(stage, curve.at(0.0), 0.08, color);
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(tracker).unwrap_or(0.0);
            s.move_to(d, card_point(curve.at(f * curve.length)), ORIGIN);
        },
        true,
    )?;
    Ok([anchor, d])
}

/// Stop an anchor's updater and take it out of the scene; what it drew
/// stays as it was last drawn.
pub fn stop(stage: &mut Stage<'_>, anchor: Mob) {
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
}

// ------------------------------------------------------------ the dividers

/// A leg's length, as a share of the step the dividers walk.
pub const LEG_SHARE: f64 = 0.75;

/// Where walking dividers stand (card cm): the point that stays put, the
/// point that moves, and how many landings have been reached (the trail
/// runs through landings 0 to that). `open` (0 to 1) opens them from
/// closed at landing 0 to landings 0 and 1; `walk` (0 to `landings.len()`
/// − 2) then walks them: at a whole number k they stand on landings k and
/// k + 1, and in between they pivot on landing k + 1 while the other leg
/// swings, `step` from it, from landing k round to landing k + 2.
pub fn divider_pose(landings: &[P2], step: f64, open: f64, walk: f64) -> (P2, P2, usize) {
    let n = landings.len() - 1;
    if walk <= 0.0 {
        let (a, b) = (landings[0], landings[1]);
        let o = open.clamp(0.0, 1.0);
        return (
            a,
            [a[0] + o * (b[0] - a[0]), a[1] + o * (b[1] - a[1])],
            usize::from(o >= 1.0),
        );
    }
    let w = walk.min((n - 1) as f64);
    let k = (w.floor() as usize).min(n - 1);
    let f = w - k as f64;
    if f < 1e-9 {
        return (landings[k], landings[k + 1], k + 1);
    }
    let (behind, pivot, ahead) = (landings[k], landings[k + 1], landings[k + 2]);
    let a0 = (behind[1] - pivot[1]).atan2(behind[0] - pivot[0]);
    let a1 = (ahead[1] - pivot[1]).atan2(ahead[0] - pivot[0]);
    // the shorter way round, in (−π, π]
    let mut turn = (a1 - a0) % std::f64::consts::TAU;
    if turn > std::f64::consts::PI {
        turn -= std::f64::consts::TAU;
    } else if turn <= -std::f64::consts::PI {
        turn += std::f64::consts::TAU;
    }
    let a = a0 + f * turn;
    (pivot, [pivot[0] + step * a.cos(), pivot[1] + step * a.sin()], k + 1)
}

/// The hinge of dividers standing on `p` and `q`, legs `LEG_SHARE × step`
/// long, on the left of p → q (`toward` gives the side while they are
/// closed).
pub fn hinge(p: P2, q: P2, step: f64, toward: P2) -> P2 {
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let d = dx.hypot(dy);
    let (ux, uy) = if d > 1e-6 {
        (dx / d, dy / d)
    } else {
        (toward[0], toward[1])
    };
    let leg = LEG_SHARE * step;
    let rise = (leg * leg - d * d / 4.0).max(0.0).sqrt();
    [(p[0] + q[0]) / 2.0 - uy * rise, (p[1] + q[1]) / 2.0 + ux * rise]
}

/// Dividers walking `landings` (card cm), `step` apart: two trackers, the
/// legs and the trail of chords redrawn from them by one updater, and a
/// dot for each landing. Detached.
pub struct Dividers {
    pub open: Mob,
    pub walk: Mob,
    pub anchor: Mob,
    pub legs: Mob,
    pub trail: Mob,
    pub dots: Vec<Mob>,
    /// The swings that walk every landing (`landings.len()` − 2).
    pub swings: usize,
}

impl Dividers {
    pub fn new(stage: &mut Stage<'_>, landings: &[P2], step: f64) -> fmn::Result<Self> {
        if landings.len() < 2 {
            return Err(fail("dividers need two landings to stand on".into()));
        }
        let color = palette::dividers();
        let (a, b) = (landings[0], landings[1]);
        let d = (b[0] - a[0]).hypot(b[1] - a[1]);
        let toward = [(b[0] - a[0]) / d, (b[1] - a[1]) / d];
        let open = stage.add_value_tracker(0.0);
        let walk = stage.add_value_tracker(0.0);
        let shape = {
            let landings = landings.to_vec();
            move |o: f64, w: f64| -> (Vec<Vec3>, Vec<Vec3>) {
                let (p, q, reached) = divider_pose(&landings, step, o, w);
                let legs = vec![card_point(p), card_point(hinge(p, q, step, toward)), card_point(q)];
                let trail = if reached == 0 {
                    vec![card_point(landings[0]), card_point(q)]
                } else {
                    landings[..=reached].iter().map(|&l| card_point(l)).collect()
                };
                (legs, trail)
            }
        };
        let (legs0, trail0) = shape(0.0, 0.0);
        let legs = stage
            .arena_mut()
            .add(polyline(legs0).style(line_style(color, 3.5)).build());
        let trail = stage
            .arena_mut()
            .add(polyline(trail0).style(line_style(color, 2.5)).build());
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = (
                    s.tracker_value(open).unwrap_or(0.0),
                    s.tracker_value(walk).unwrap_or(0.0),
                );
                if last == Some(now) {
                    return;
                }
                let (l, t) = shape(now.0, now.1);
                let _ = s.set_points(legs, polyline(l).build().points());
                let _ = s.set_points(trail, polyline(t).build().points());
                last = Some(now);
            },
            true,
        )?;
        let dots = landings.iter().map(|&l| dot(stage, l, 0.05, color)).collect();
        Ok(Self {
            open,
            walk,
            anchor,
            legs,
            trail,
            dots,
            swings: landings.len() - 2,
        })
    }
}

/// Scale `mobs` by `factor` about `about` (a zoom: everything on the card
/// together), `rt` seconds. Stop any updater drawing them first.
pub fn zoom(stage: &mut Stage<'_>, mobs: &[Mob], factor: f64, about: Vec3, rt: f64) -> fmn::Result<()> {
    let mut moves = Vec::with_capacity(mobs.len());
    for &m in mobs {
        moves.push(
            m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .scale_about(factor, Some(about), None)?,
        );
    }
    together(stage, moves)
}

/// Where a card point lands after [`zoom`] by `factor` about `about`.
pub fn zoomed(p: P2, factor: f64, about: Vec3) -> Vec3 {
    let q = card_point(p);
    v(
        about[0] + factor * (q[0] - about[0]),
        about[1] + factor * (q[1] - about[1]),
    )
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

/// Fade `old` out and the detached `new` in, in one move.
pub fn swap(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<()> {
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
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// A filled shape already in the scene brought to `opacity`, `rt` seconds.
/// Every colour of the family takes it, the fill's too, so a stroked line
/// would fill in: dim lines with [`dim_line`].
pub fn fade_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

/// A stroked line already in the scene brought to `opacity`, its fill left
/// empty, `rt` seconds.
pub fn dim_line(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_stroke(None, None, Some(opacity))?)
}

/// Builders played together, `rt` seconds each (as set on them).
pub fn together(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(moves.len());
    for m in moves {
        anims.push(stage.prepare(m)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
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

/// How long a chapter's last reveal stands whole before the chapter ends,
/// seconds (the floor the corpus's chapters keep).
pub const STAND_UP: f64 = 2.6;

/// Before a chapter's end: the last line and its breath (`Kit::hold`), and
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
    use crate::model::{self, OPENING_STEP};

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// Walking dividers stand on two landings at every whole step, keep
    /// their points one step apart through every swing, and open from
    /// closed.
    #[test]
    fn the_dividers_stand_on_landings_and_keep_their_step() {
        let walk = model::coast().walk(OPENING_STEP);
        let landings: Vec<P2> = walk.landings.iter().map(|&(p, _)| p).collect();
        let (p, q, reached) = divider_pose(&landings, OPENING_STEP, 0.0, 0.0);
        assert!(dist(p, q) < 1e-12 && reached == 0, "closed at the start");
        let (_, q, reached) = divider_pose(&landings, OPENING_STEP, 1.0, 0.0);
        assert!(dist(q, landings[1]) < 1e-12 && reached == 1, "open on the first step");
        let swings = landings.len() - 2;
        for k in 0..=swings {
            let (p, q, reached) = divider_pose(&landings, OPENING_STEP, 1.0, k as f64);
            assert!(
                dist(p, landings[k]) < 1e-12 && dist(q, landings[k + 1]) < 1e-9,
                "step {k}"
            );
            assert_eq!(reached, k + 1);
        }
        for i in 1..100 * swings {
            let w = i as f64 / 100.0;
            let (p, q, _) = divider_pose(&landings, OPENING_STEP, 1.0, w);
            assert!((dist(p, q) - OPENING_STEP).abs() < 1e-9, "at {w}");
        }
        // past the last landing they stay on it
        let (_, q, reached) = divider_pose(&landings, OPENING_STEP, 1.0, swings as f64 + 3.0);
        assert!(dist(q, landings[swings + 1]) < 1e-9 && reached == swings + 1);
    }

    #[test]
    fn the_hinge_stands_a_leg_from_each_point() {
        let (p, q) = ([0.0, 0.0], [2.0, 0.0]);
        let h = hinge(p, q, 2.0, [1.0, 0.0]);
        assert!((dist(h, p) - 1.5).abs() < 1e-12 && (dist(h, q) - 1.5).abs() < 1e-12);
        assert!(h[1] > 0.0, "on the left of p to q");
        let closed = hinge(p, p, 2.0, [0.0, -1.0]);
        assert!((dist(closed, p) - 1.5).abs() < 1e-12 && closed[0] > 0.0, "{closed:?}");
    }

    #[test]
    fn zoomed_points_follow_scaling_about_the_focus() {
        let about = card_point([1.0, 1.0]);
        assert_eq!(zoomed([1.0, 1.0], 2.2, about), about);
        let z = zoomed([2.0, 1.0], 2.0, about);
        assert!((z[0] - about[0] - 2.0 * CARD_UNIT).abs() < 1e-12 && (z[1] - about[1]).abs() < 1e-12);
    }
}
