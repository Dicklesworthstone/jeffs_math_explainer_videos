//! The video's look, shared by its chapters (storyboard, color roles): a
//! picture's grid and its squares, its counts, switches drawn on it, small
//! copies of pictures for a gallery, the think-time countdown, and line
//! timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups; a thumbnail's squares are one mobject.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, v};
use fmn::geometry::QuadPath;
use fmn::prelude::*;

use crate::model::{Picture, Switch};
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

/// A straight stroke from `a` to `b`. Detached.
pub fn segment(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(vec![a, b]).style(line_style(color, width)).build())
}

/// An open stroke through `points` (a route along rows and columns).
/// Detached.
pub fn path_line(stage: &mut Stage<'_>, points: &[Vec3], color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points.to_vec()).style(line_style(color, width)).build())
}

/// A closed stroke through `corners`. Detached.
fn outline(stage: &mut Stage<'_>, corners: &[Vec3], color: Srgb, width: f64) -> Mob {
    let mut points = corners.to_vec();
    points.extend(corners.first().copied());
    stage
        .arena_mut()
        .add(polyline(points).style(line_style(color, width)).build())
}

// --------------------------------------------------------------- playback

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
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move several mobjects together by `(dx, dy)` over `rt` seconds.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(v(dx, dy))?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move each of several mobjects by its own `(dx, dy)`, together, over `rt`
/// seconds.
pub fn shift_each(stage: &mut Stage<'_>, moves: &[(Mob, f64, f64)], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, dx, dy) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .shift(v(dx, dy))?;
        anims.push(stage.prepare(a)?);
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
/// plus its share of that sentence's characters (the voice reads at a
/// nearly even rate inside a sentence; the pauses fall between them); the
/// narrated draft's cue sheets check the result.
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

    /// Wait (if need be) until the line ends.
    pub fn wait_to_end(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        wait_until(stage, self.start + self.len)
    }
}

// --------------------------------------------------------------- think time

/// How long the viewer gets to think after a ⟨predict⟩ question, the
/// narration silent (the owner's quiz rule: 4 to 6 s, with a visible
/// countdown and "pause for more time").
pub const THINK_S: f64 = 4.0;
const THINK_R: f64 = 0.62;
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

// ------------------------------------------------------------------- boards

/// How far a row's count stands right of the grid, and a column's below it.
pub const COUNT_GAP: f64 = 0.5;
/// The counts' type size.
pub const COUNT_SIZE: f64 = 30.0;
/// A filled square's side, as a fraction of the grid's pitch.
const FILL: f64 = 0.84;

/// A picture's grid on screen: square (`r`, `c`) is row `r` from the top and
/// column `c` from the left, `cell` scene units apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Board {
    pub centre: Vec3,
    pub cell: f64,
    pub rows: usize,
    pub cols: usize,
}

impl Board {
    pub fn new(at: [f64; 2], cell: f64, rows: usize, cols: usize) -> Self {
        Self {
            centre: v(at[0], at[1]),
            cell,
            rows,
            cols,
        }
    }

    /// The same board standing at `at`.
    pub fn moved(&self, at: [f64; 2]) -> Self {
        Self {
            centre: v(at[0], at[1]),
            ..*self
        }
    }

    pub fn left(&self) -> f64 {
        self.centre[0] - self.cell * self.cols as f64 / 2.0
    }

    pub fn right(&self) -> f64 {
        self.centre[0] + self.cell * self.cols as f64 / 2.0
    }

    pub fn top(&self) -> f64 {
        self.centre[1] + self.cell * self.rows as f64 / 2.0
    }

    pub fn bottom(&self) -> f64 {
        self.centre[1] - self.cell * self.rows as f64 / 2.0
    }

    /// The centre of square (`r`, `c`).
    pub fn at(&self, r: usize, c: usize) -> Vec3 {
        v(
            self.left() + self.cell * (c as f64 + 0.5),
            self.top() - self.cell * (r as f64 + 0.5),
        )
    }

    /// Where row `r`'s count stands.
    pub fn row_count_at(&self, r: usize) -> Vec3 {
        v(self.right() + COUNT_GAP, self.at(r, 0)[1])
    }

    /// Where column `c`'s count stands.
    pub fn col_count_at(&self, c: usize) -> Vec3 {
        v(self.at(0, c)[0], self.bottom() - COUNT_GAP)
    }

    /// The grid's lines, faint. Detached.
    pub fn lines(&self, stage: &mut Stage<'_>) -> Vec<Mob> {
        let mut out = Vec::with_capacity(self.rows + self.cols + 2);
        for r in 0..=self.rows {
            let y = self.top() - self.cell * r as f64;
            out.push(segment(
                stage,
                v(self.left(), y),
                v(self.right(), y),
                Palette::faint(),
                2.0,
            ));
        }
        for c in 0..=self.cols {
            let x = self.left() + self.cell * c as f64;
            out.push(segment(
                stage,
                v(x, self.top()),
                v(x, self.bottom()),
                Palette::faint(),
                2.0,
            ));
        }
        out
    }

    /// A filled square at (`r`, `c`). Detached.
    pub fn square(&self, stage: &mut Stage<'_>, r: usize, c: usize, color: Srgb) -> fmn::Result<Mob> {
        let side = self.cell * FILL;
        let square = Rectangle::new()
            .width(side)
            .height(side)
            .corner_radius(0.05)
            .style(flat(color, 1.0))
            .build()?
            .moved_to(self.at(r, c));
        Ok(stage.arena_mut().add(square))
    }

    /// A square at (`r`, `c`) shaded `color` at `opacity` (0 to 1): how
    /// often a square is filled, drawn as how strongly it is tinted.
    /// Detached.
    pub fn shade(&self, stage: &mut Stage<'_>, r: usize, c: usize, color: Srgb, opacity: f64) -> fmn::Result<Mob> {
        let side = self.cell * FILL;
        let square = Rectangle::new()
            .width(side)
            .height(side)
            .corner_radius(0.05)
            .style(flat(color, opacity.clamp(0.0, 1.0)))
            .build()?
            .moved_to(self.at(r, c));
        Ok(stage.arena_mut().add(square))
    }

    /// A small dot in square (`r`, `c`): a square reasoning has settled
    /// empty. Detached.
    pub fn dot(&self, stage: &mut Stage<'_>, r: usize, c: usize, color: Srgb) -> Mob {
        let dot = Circle::new()
            .radius(self.cell * 0.08)
            .style(flat(color, 1.0))
            .build()
            .moved_to(self.at(r, c));
        stage.arena_mut().add(dot)
    }

    /// A ring around square (`r`, `c`): a closed stroke just outside a
    /// filled square's edge. Detached.
    pub fn ring(&self, stage: &mut Stage<'_>, r: usize, c: usize, color: Srgb) -> Mob {
        let h = self.cell * (FILL + 0.1) / 2.0;
        let p = self.at(r, c);
        let corners = [
            v(p[0] - h, p[1] + h),
            v(p[0] + h, p[1] + h),
            v(p[0] + h, p[1] - h),
            v(p[0] - h, p[1] - h),
        ];
        outline(stage, &corners, color, 4.0)
    }

    /// The rectangle a switch spans, through its four corners' centres.
    /// Detached.
    pub fn rectangle(&self, stage: &mut Stage<'_>, s: Switch, color: Srgb) -> Mob {
        let corners = s.corners().map(|(r, c)| self.at(r, c));
        outline(stage, &corners, color, 3.0)
    }

    /// A cross over square (`r`, `c`): two strokes, corner to corner.
    /// Detached.
    pub fn cross(&self, stage: &mut Stage<'_>, r: usize, c: usize, color: Srgb) -> [Mob; 2] {
        let h = self.cell * FILL / 2.0;
        let p = self.at(r, c);
        [
            segment(stage, v(p[0] - h, p[1] + h), v(p[0] + h, p[1] - h), color, 6.0),
            segment(stage, v(p[0] - h, p[1] - h), v(p[0] + h, p[1] + h), color, 6.0),
        ]
    }
}

/// A small copy of picture `p` centred at `at`, `cell` scene units a square:
/// its frame (a stroke in `frame`) and its filled squares as one mobject.
/// Detached.
pub fn thumb(stage: &mut Stage<'_>, p: &Picture, at: Vec3, cell: f64, frame: Srgb) -> fmn::Result<Vec<Mob>> {
    let (w, h) = (cell * p.cols() as f64, cell * p.rows() as f64);
    let (left, top) = (at[0] - w / 2.0, at[1] + h / 2.0);
    let edge = 0.03;
    let corners = [
        v(left - edge, top + edge),
        v(left + w + edge, top + edge),
        v(left + w + edge, top - h - edge),
        v(left - edge, top - h - edge),
    ];
    let mut out = vec![outline(stage, &corners, frame, 1.5)];
    let filled = p.filled();
    if !filled.is_empty() {
        let g = cell * (1.0 - FILL) / 2.0;
        let mut path = QuadPath::new();
        for (r, c) in filled {
            let (x0, y1) = (left + cell * c as f64, top - cell * r as f64);
            let (x1, y0) = (x0 + cell, y1 - cell);
            let quad = [
                v(x0 + g, y0 + g),
                v(x1 - g, y0 + g),
                v(x1 - g, y1 - g),
                v(x0 + g, y1 - g),
            ];
            path.add_subpath(Polygon::new(quad).build().points())
                .map_err(|e| fail(format!("a thumbnail's square: {e:?}")))?;
        }
        out.push(
            stage
                .arena_mut()
                .add(VMobject::from_path(&path).with_style(flat(palette::filled(), 1.0))),
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_start_after_their_stops() {
        let text = "Hide the picture and keep the shadows. Could a different picture cast the same shadows?";
        let offsets = sentence_offsets(text);
        assert_eq!(offsets.len(), 2);
        assert!(text[offsets[1]..].starts_with("Could"));
    }

    /// A board's squares sit inside its lines, and its counts outside them.
    #[test]
    fn a_board_keeps_its_squares_inside_and_its_counts_outside() {
        let b = Board::new([-0.6, 0.35], 0.62, 5, 5);
        let (tl, br) = (b.at(0, 0), b.at(4, 4));
        assert!(tl[0] > b.left() && tl[1] < b.top());
        assert!(br[0] < b.right() && br[1] > b.bottom());
        assert!(b.row_count_at(2)[0] > b.right());
        assert!(b.col_count_at(2)[1] < b.bottom());
        assert_eq!(b.moved([-0.6, 0.35]), b);
    }
}
