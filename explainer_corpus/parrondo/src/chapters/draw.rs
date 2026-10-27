//! The video's look, shared by its chapters (storyboard, color roles and
//! stable objects): the player's token and dollar chips, coins, the game
//! cards, the coin badge, arrows, a run drawn along a data window, the
//! think-time countdown, and line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups. Value trackers, live numbers and bars come from the
//! kit (`tween`, `Kit::live_number`, `live_bar`); the line-timing and think
//! helpers follow the second-wave crates (`freivalds/src/chapters/draw.rs`),
//! where they are rendered and reviewed.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Frame2, Kit, Palette, Script, Timed, v};
use fmn::prelude::*;

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

/// Seconds a card needs on screen to be read: fifteen characters a second,
/// counted from its completed fade-in.
pub fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// Scene time now.
pub fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

/// A number as the screen writes it: a typographic minus, thousands
/// separated ("−1,062").
pub fn screen_int(x: i64) -> String {
    explainer_kit::signed_thousands(i128::from(x))
}

/// A fact's decimal as the screen writes it: the typographic minus.
pub fn screen(text: &str) -> String {
    text.replace('-', "\u{2212}")
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

/// Move a mobject so its centre is at `to`, `rt` seconds.
pub fn fly(mob: Mob, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// A mobject already in the scene brought to `opacity`, `rt` seconds.
pub fn fade_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(opacity)?)
}

/// A group of mobjects scaled by `factor` about `about` and moved by
/// `shift`, together, `rt` seconds: a picture shrinking to a corner while
/// every part keeps its place in it.
pub fn shrink_all(
    stage: &mut Stage<'_>,
    mobs: &[Mob],
    factor: f64,
    about: Vec3,
    shift: Vec3,
    rt: f64,
) -> fmn::Result<()> {
    let mut moves = Vec::with_capacity(mobs.len());
    for &m in mobs {
        moves.push(
            m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .scale_about(factor, Some(about), None)?
                .shift(shift)?,
        );
    }
    together(stage, moves)
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

/// How long a chapter's last reveal stands whole before the chapter fades,
/// seconds (the floor the corpus's chapters keep).
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
/// sentences start, so a picture can land on the phrase it illustrates (as
/// in the second-wave crates: a phrase's moment is its sentence's measured
/// start plus its share of that sentence's characters).
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
/// the narrator's `measured` onsets, or with no narrator each sentence's
/// share of `len` by characters. A narrator that splits the line
/// differently is an error, never a guess.
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

/// Refuse a picture whose `mob` would leave the band from `left` to `right`
/// (scene x): a label running into its neighbour or off the frame. Text
/// widths are only known once typeset, so the scene checks them as it goes.
pub fn keep_between(stage: &Stage<'_>, mob: Mob, left: f64, right: f64, what: &str) -> fmn::Result<()> {
    let (l, r) = (
        stage.get_bounding_box_point(mob, LEFT)[0],
        stage.get_bounding_box_point(mob, RIGHT)[0],
    );
    if l < left || r > right {
        return Err(fail(format!(
            "{what} spans x {l:.2} to {r:.2}, outside {left:.2} to {right:.2}"
        )));
    }
    Ok(())
}

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown and "pause for more time").
pub const THINK_S: f64 = 4.0;
const THINK_R: f64 = 0.62;
pub const PAUSE_NOTE: &str = "pause for more time";

/// The prediction pause: let the question finish, mark the moment for the
/// score, bring in a large "?" at `at` inside a hero ring that drains over
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

// ------------------------------------------------------------ the objects

/// The player's token: a teal disc, the same in every chapter. Detached.
pub const TOKEN_R: f64 = 0.3;

pub fn token(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(TOKEN_R)
            .arc_center(at)
            .style(flat(palette::player(), 1.0))
            .build(),
    )
}

/// One dollar chip, a small teal disc. Detached.
pub const CHIP_R: f64 = 0.11;

pub fn chip(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(CHIP_R)
            .arc_center(at)
            .style(flat(palette::player(), 0.85))
            .build(),
    )
}

/// A coin: a disc in `color` with `face` written on it. Detached.
pub fn coin(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb, face: &str) -> fmn::Result<[Mob; 2]> {
    let disc = stage
        .arena_mut()
        .add(Circle::new().radius(r).arc_center(at).style(flat(color, 1.0)).build());
    let face = kit.text(stage, face, 22.0, explainer_kit::hex(explainer_kit::BACKGROUND), at)?;
    Ok([disc, face])
}

/// A game's card: an outlined frame `w` by `h` about `centre` and its title
/// at the top in the game's color. Detached.
pub fn card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    title: &str,
    centre: Vec3,
    w: f64,
    h: f64,
    color: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let frame = explainer_kit::rect(
        v(centre[0] - w / 2.0, centre[1] - h / 2.0),
        w,
        h,
        line_style(Palette::dim(), 2.0),
    )?;
    let frame = stage.arena_mut().add(frame);
    let name = kit.text(stage, title, 30.0, color, v(centre[0], centre[1] + h / 2.0 - 0.35))?;
    Ok([frame, name])
}

/// A straight arrow from `from` to `to`: its shaft and a filled head.
/// Detached.
pub fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, width: f64) -> [Mob; 2] {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (ux, uy) = (dx / len, dy / len);
    let head = 0.18_f64.min(len / 2.0);
    let base = v(to[0] - ux * head, to[1] - uy * head);
    let shaft = stage
        .arena_mut()
        .add(polyline([from, base]).style(line_style(color, width)).build());
    let (px, py) = (-uy * head * 0.55, ux * head * 0.55);
    let tip = stage.arena_mut().add(
        Polygon::new([to, v(base[0] + px, base[1] + py), v(base[0] - px, base[1] - py)])
            .style(flat(color, 1.0))
            .build(),
    );
    [shaft, tip]
}

/// A curved arrow from `from` to `to` bowing `bow` to the left of the
/// direction of travel (upward for a left-to-right arrow). Detached.
pub fn curved_arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, bow: f64, color: Srgb, width: f64) -> [Mob; 2] {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    let (nx, ny) = (-dy / len, dx / len);
    let mid = v((from[0] + to[0]) / 2.0 + nx * bow, (from[1] + to[1]) / 2.0 + ny * bow);
    let at = |t: f64| -> Vec3 {
        let a = (1.0 - t) * (1.0 - t);
        let b = 2.0 * (1.0 - t) * t;
        let c = t * t;
        v(
            a * from[0] + b * mid[0] + c * to[0],
            a * from[1] + b * mid[1] + c * to[1],
        )
    };
    let points: Vec<Vec3> = (0..=32).map(|k| at(f64::from(k) / 32.0 * 0.94)).collect();
    let last = *points.last().expect("an arc has points");
    let shaft = stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build());
    let head = arrow(stage, last, to, color, width);
    // only the head of the short straight piece is kept
    [shaft, head[1]]
}

/// A dashed straight line, `dash` long dashes with equal gaps. Detached.
pub fn dashed(stage: &mut Stage<'_>, from: Vec3, to: Vec3, dash: f64, color: Srgb, width: f64) -> Vec<Mob> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    let n = (len / (2.0 * dash)).floor().max(1.0) as usize;
    (0..n)
        .map(|k| {
            let t0 = (2 * k) as f64 * dash / len;
            let t1 = (((2 * k + 1) as f64) * dash / len).min(1.0);
            let a = v(from[0] + dx * t0, from[1] + dy * t0);
            let b = v(from[0] + dx * t1, from[1] + dy * t1);
            stage
                .arena_mut()
                .add(polyline([a, b]).style(line_style(color, width)).build())
        })
        .collect()
}

/// The coin badge at the top right, naming the coins in play (storyboard,
/// "Coin badge"): `published` filled, the round-number coins outlined in
/// hero. Detached.
pub const BADGE_AT: Vec3 = [4.55, 3.6, 0.0];

pub fn badge(kit: &Kit, stage: &mut Stage<'_>, text: &str, published: bool) -> fmn::Result<[Mob; 2]> {
    let words = kit.text(
        stage,
        text,
        18.0,
        if published { Palette::ink() } else { Palette::hero() },
        BADGE_AT,
    )?;
    let w = stage.get_width(words) + 0.3;
    let h = 0.38;
    let style = if published {
        flat(Palette::faint(), 0.9)
    } else {
        line_style(Palette::hero(), 2.0)
    };
    let back = explainer_kit::rect(v(BADGE_AT[0] - w / 2.0, BADGE_AT[1] - h / 2.0), w, h, style)?;
    let back = stage.arena_mut().add(back);
    Ok([back, words])
}

// --------------------------------------------------------------- the runs

/// Every `step`th point of a run as data (round, dollars), the last always
/// kept, so a long run draws as a few hundred segments.
pub fn thinned(dollars: &[i64], step: usize) -> Vec<(f64, f64)> {
    let last = dollars.len() - 1;
    (0..=last)
        .step_by(step.max(1))
        .chain((!last.is_multiple_of(step.max(1))).then_some(last))
        .map(|k| (k as f64, dollars[k] as f64))
        .collect()
}

/// A run drawn along `frame` as a tracker goes from 0 to 1: the polyline
/// grows from its first point, and `rider` (the player's token, if given)
/// rides its tip. Returns the line, an invisible anchor holding the
/// updater, and the tracker; play `tween(tracker, 1.0, rt)` to draw it.
pub fn growing_run(
    stage: &mut Stage<'_>,
    frame: &Frame2,
    points: &[(f64, f64)],
    color: Srgb,
    rider: Option<Mob>,
) -> fmn::Result<(Mob, Mob, Mob)> {
    let scene: Vec<Vec3> = points.iter().map(|&(x, y)| frame.p(x, y)).collect();
    let progress = stage.add_value_tracker(0.0);
    let first = scene[0];
    let line = stage
        .arena_mut()
        .add(polyline([first, first]).style(line_style(color, 3.0)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(progress).unwrap_or(0.0).clamp(0.0, 1.0);
            let k = ((scene.len() - 1) as f64 * f).round() as usize;
            if last == Some(k) {
                return;
            }
            let shown: Vec<Vec3> = if k == 0 {
                vec![first, first]
            } else {
                scene[..=k].to_vec()
            };
            let _ = s.set_points(line, polyline(shown).build().points());
            if let Some(r) = rider {
                s.move_to(r, scene[k], ORIGIN);
            }
            last = Some(k);
        },
        true,
    )?;
    Ok((line, anchor, progress))
}

/// Draw a run made by [`growing_run`] over `rt` seconds, at an even pace
/// (the kit's `tween` eases, which would hide where the run is slow).
pub fn draw_run(stage: &mut Stage<'_>, progress: Mob, rt: f64) -> fmn::Result<()> {
    let grow = progress
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(1.0)?;
    stage.play(grow)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phrase_lands_with_its_measured_sentence() {
        let text = "One two. Three four.";
        let sentences = timed_sentences("x", text, 6.0, &[0.0, 4.0]).unwrap();
        assert_eq!(sentences.iter().map(|&(at, _)| at).collect::<Vec<_>>(), [0, 9]);
        let line = Spoken {
            id: "x".to_owned(),
            start: 10.0,
            len: 6.0,
            text: text.to_owned(),
            sentences,
        };
        let close = |phrase: &str, t: f64| (line.at(phrase).unwrap() - t).abs() < 1e-9;
        assert!(close("Three", 14.0));
        assert!(close("two", 10.0 + 4.0 * 4.0 / 9.0));
        assert!(line.at("five").is_err());
        assert!(timed_sentences("x", text, 6.0, &[0.0]).is_err());
    }

    /// A thinned run keeps its first and last rounds and every step between.
    #[test]
    fn thinning_keeps_both_ends() {
        let run: Vec<i64> = (0..=10).collect();
        let t = thinned(&run, 4);
        assert_eq!(t.iter().map(|p| p.0 as usize).collect::<Vec<_>>(), [0, 4, 8, 10]);
        let t = thinned(&run, 5);
        assert_eq!(t.iter().map(|p| p.0 as usize).collect::<Vec<_>>(), [0, 5, 10]);
    }

    #[test]
    fn screen_numbers_use_the_typographic_minus() {
        assert_eq!(screen_int(-1062), "\u{2212}1,062");
        assert_eq!(screen_int(1748), "1,748");
        assert_eq!(screen("-0.87"), "\u{2212}0.87");
    }
}
