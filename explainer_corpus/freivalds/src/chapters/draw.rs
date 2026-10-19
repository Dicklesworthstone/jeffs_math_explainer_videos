//! The video's look, shared by its chapters (storyboard, color roles):
//! matrices and columns drawn as numbers between brackets, coins, outlines
//! that light a row, a column or an entry, the think-time countdown, and
//! line timing.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules
//! (`explainer_kit::kit`): one flat fill per mobject, no outline on filled
//! shapes, small groups. A coin's face is brought in after its disc, so the
//! digit is painted over it.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{BACKGROUND, Kit, Palette, Script, Timed, hex, signed_thousands, v};
use fmn::prelude::*;

use crate::model::{Column, Matrix};
use crate::palette;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ----------------------------------------------------------------- geometry

/// The spacing of a drawn matrix's entries, across and down.
pub const DX: f64 = 0.62;
pub const DY: f64 = 0.56;
/// The size an entry is set in.
pub const ENTRY_SIZE: f64 = 30.0;
/// How far a bracket stands outside the outer entries' cells, and its serifs.
const BRACKET_GAP: f64 = 0.05;
const SERIF: f64 = 0.12;
/// A coin's radius and the size of its face.
pub const COIN_R: f64 = 0.19;
const COIN_FACE: f64 = 20.0;

/// Where entry (`i`, `j`) of a `rows` × `cols` grid centred at `centre` sits.
pub fn cell(centre: Vec3, rows: usize, cols: usize, i: usize, j: usize) -> Vec3 {
    let x = (j as f64 - (cols as f64 - 1.0) / 2.0) * DX;
    let y = ((rows as f64 - 1.0) / 2.0 - i as f64) * DY;
    v(centre[0] + x, centre[1] + y)
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

/// A whole number as the screen writes it: "7", "1,000", "−2" (a
/// typographic minus).
pub fn num(x: i64) -> String {
    signed_thousands(i128::from(x))
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

/// Move several mobjects together by `(dx, dy)` over `rt` seconds.
pub fn shift_all(stage: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64, rt: f64) -> fmn::Result<()> {
    let mut moves = Vec::with_capacity(mobs.len());
    for &m in mobs {
        moves.push(
            m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .shift(v(dx, dy))?,
        );
    }
    together(stage, moves)
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

// ---------------------------------------------------- matrices and columns

/// A matrix or a column drawn as numbers between brackets: one text per
/// entry, so a row, a column or an entry can be lit, copied and moved.
pub struct Grid {
    pub centre: Vec3,
    pub rows: usize,
    pub cols: usize,
    /// The numbers drawn, row by row.
    pub values: Vec<Vec<i64>>,
    /// Their texts, row by row.
    pub entries: Vec<Vec<Mob>>,
    pub brackets: [Mob; 2],
}

impl Grid {
    /// Where entry (`i`, `j`) sits.
    pub fn at(&self, i: usize, j: usize) -> Vec3 {
        cell(self.centre, self.rows, self.cols, i, j)
    }

    /// The texts of column `j`, top to bottom.
    pub fn column(&self, j: usize) -> Vec<Mob> {
        self.entries.iter().map(|row| row[j]).collect()
    }

    /// The texts of row `i`, left to right.
    pub fn row(&self, i: usize) -> Vec<Mob> {
        self.entries[i].clone()
    }

    /// The brackets and every entry, to fade or move together.
    pub fn mobs(&self) -> Vec<Mob> {
        let mut all = self.brackets.to_vec();
        all.extend(self.entries.iter().flatten());
        all
    }

    /// From the centre to a bracket.
    pub fn half_width(&self) -> f64 {
        DX * self.cols as f64 / 2.0 + BRACKET_GAP
    }

    /// The same grid, its mobjects moved `dx` to the right.
    pub fn moved(self, dx: f64) -> Self {
        Self {
            centre: v(self.centre[0] + dx, self.centre[1]),
            ..self
        }
    }
}

/// One bracket of a `rows` × `cols` grid at `centre`, on the left (`side`
/// −1) or the right (+1): an open stroke, never filled.
fn bracket(centre: Vec3, rows: usize, cols: usize, side: f64) -> VMobject {
    let x = centre[0] + side * (DX * cols as f64 / 2.0 + BRACKET_GAP);
    let half = DY * rows as f64 / 2.0;
    let (top, bottom) = (centre[1] + half, centre[1] - half);
    let serif = -side * SERIF;
    polyline(vec![v(x + serif, top), v(x, top), v(x, bottom), v(x + serif, bottom)])
        .style(line_style(Palette::dim(), 2.5))
        .build()
}

/// The numbers `values`, row by row, as a grid centred at `centre` in
/// `color`, its brackets dim. Detached.
pub fn grid(kit: &Kit, stage: &mut Stage<'_>, values: Vec<Vec<i64>>, centre: Vec3, color: Srgb) -> fmn::Result<Grid> {
    grid_colored(kit, stage, values, centre, |_| color)
}

/// As [`grid`], each entry in the color `color_of` gives its value (a
/// mistake's zeros quiet, its wrong entries plain). Detached.
pub fn grid_colored(
    kit: &Kit,
    stage: &mut Stage<'_>,
    values: Vec<Vec<i64>>,
    centre: Vec3,
    color_of: impl Fn(i64) -> Srgb,
) -> fmn::Result<Grid> {
    grid_of(stage, values, centre, |stage, x, at| {
        kit.text(stage, &num(x), ENTRY_SIZE, color_of(x), at)
    })
}

/// As [`grid`], each entry typeset as the TeX `tex_of` gives its value, so
/// a grid can hold numbers too wide to print ("N" for 10^16). Detached.
pub fn tex_grid(
    kit: &Kit,
    stage: &mut Stage<'_>,
    values: Vec<Vec<i64>>,
    centre: Vec3,
    color: Srgb,
    tex_of: impl Fn(i64) -> String,
) -> fmn::Result<Grid> {
    grid_of(stage, values, centre, |stage, x, at| {
        kit.tex(stage, &tex_of(x), ENTRY_SIZE, color, &[], at)
    })
}

/// The grid of `values` centred at `centre`, each entry the mobject `entry`
/// makes of its value at its place, row by row, then the two brackets.
fn grid_of(
    stage: &mut Stage<'_>,
    values: Vec<Vec<i64>>,
    centre: Vec3,
    mut entry: impl FnMut(&mut Stage<'_>, i64, Vec3) -> fmn::Result<Mob>,
) -> fmn::Result<Grid> {
    let rows = values.len();
    let cols = values.first().map_or(0, Vec::len);
    let mut entries = Vec::with_capacity(rows);
    for (i, row) in values.iter().enumerate() {
        let mut texts = Vec::with_capacity(cols);
        for (j, &x) in row.iter().enumerate() {
            texts.push(entry(stage, x, cell(centre, rows, cols, i, j))?);
        }
        entries.push(texts);
    }
    let left = stage.arena_mut().add(bracket(centre, rows, cols, -1.0));
    let right = stage.arena_mut().add(bracket(centre, rows, cols, 1.0));
    Ok(Grid {
        centre,
        rows,
        cols,
        values,
        entries,
        brackets: [left, right],
    })
}

/// A matrix as a grid. Detached.
pub fn matrix_grid<const N: usize>(
    kit: &Kit,
    stage: &mut Stage<'_>,
    m: &Matrix<N>,
    centre: Vec3,
    color: Srgb,
) -> fmn::Result<Grid> {
    grid(kit, stage, m.iter().map(|row| row.to_vec()).collect(), centre, color)
}

/// A column as a grid of one column. Detached.
pub fn column_grid<const N: usize>(
    kit: &Kit,
    stage: &mut Stage<'_>,
    c: &Column<N>,
    centre: Vec3,
    color: Srgb,
) -> fmn::Result<Grid> {
    grid(kit, stage, c.iter().map(|&x| vec![x]).collect(), centre, color)
}

/// An outline of `w` × `h` centred at `at`, in `color`. Detached.
fn outline(stage: &mut Stage<'_>, w: f64, h: f64, at: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.08)
        .style(line_style(color, 2.5))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// An outline around column `j` of `g`. Detached.
pub fn column_box(stage: &mut Stage<'_>, g: &Grid, j: usize, color: Srgb) -> fmn::Result<Mob> {
    let at = v(g.at(0, j)[0], g.centre[1]);
    outline(stage, DX * 0.86, DY * g.rows as f64 + 0.06, at, color)
}

/// An outline around row `i` of `g`. Detached.
pub fn row_box(stage: &mut Stage<'_>, g: &Grid, i: usize, color: Srgb) -> fmn::Result<Mob> {
    let at = v(g.centre[0], g.at(i, 0)[1]);
    outline(stage, DX * g.cols as f64 + 0.06, DY * 0.86, at, color)
}

/// An outline around entry (`i`, `j`) of `g`. Detached.
pub fn cell_box(stage: &mut Stage<'_>, g: &Grid, i: usize, j: usize, color: Srgb) -> fmn::Result<Mob> {
    outline(stage, DX * 0.86, DY * 0.86, g.at(i, j), color)
}

// -------------------------------------------------------------------- coins

/// One coin: a disc and its face, 1 for heads (filled in the heads role)
/// and 0 for tails (faint).
pub struct Coin {
    pub at: Vec3,
    pub disc: Mob,
    pub face: Mob,
}

impl Coin {
    /// The disc, then the face painted over it.
    pub fn mobs(&self) -> [Mob; 2] {
        [self.disc, self.face]
    }
}

/// A coin at `at`, heads or tails. Detached.
pub fn coin(kit: &Kit, stage: &mut Stage<'_>, heads: bool, at: Vec3) -> fmn::Result<Coin> {
    let (fill, ink) = if heads {
        (palette::heads(), hex(BACKGROUND))
    } else {
        (Palette::faint(), Palette::dim())
    };
    let disc = Circle::new()
        .radius(COIN_R)
        .arc_center(at)
        .style(flat(fill, 1.0))
        .build();
    let disc = stage.arena_mut().add(disc);
    let face = kit.text(stage, &i64::from(heads).to_string(), COIN_FACE, ink, at)?;
    Ok(Coin { at, disc, face })
}

/// A toss as coins, one at each of `xs`, at height `y`. Detached.
pub fn toss_coins(kit: &Kit, stage: &mut Stage<'_>, toss: &[bool], xs: &[f64], y: f64) -> fmn::Result<Vec<Coin>> {
    toss.iter()
        .zip(xs)
        .map(|(&heads, &x)| coin(kit, stage, heads, v(x, y)))
        .collect()
}

/// Every disc and face of `coins`, in painting order.
pub fn coin_mobs(coins: &[Coin]) -> Vec<Mob> {
    coins.iter().flat_map(Coin::mobs).collect()
}

/// A ring of radius `r` about `at` (a coin picked out). Detached.
pub fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(r)
            .arc_center(at)
            .style(line_style(color, 3.0))
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A phrase that opens a sentence lands on the narrator's measured
    /// onset; inside a sentence it keeps its share of that sentence.
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

    /// A grid's cells are a step apart about its centre, and a column of
    /// three sits on the matrix's rows.
    #[test]
    fn cells_sit_a_step_apart_about_the_centre() {
        let close = |a: Vec3, b: Vec3| (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12;
        let centre = v(1.0, 2.0);
        assert!(close(cell(centre, 3, 3, 1, 1), centre));
        assert!(close(cell(centre, 3, 3, 0, 0), v(1.0 - DX, 2.0 + DY)));
        assert!(close(cell(centre, 3, 1, 2, 0), v(1.0, 2.0 - DY)));
        assert!(close(cell(centre, 3, 1, 0, 0), cell(centre, 3, 3, 0, 1)));
    }
}
