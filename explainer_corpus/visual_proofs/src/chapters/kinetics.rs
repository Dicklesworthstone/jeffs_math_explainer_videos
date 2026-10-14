//! What the flat chapters (06 to 10) share beyond `draw`: several moves
//! played together or one after another, a rigid zoom about a point, and a
//! piece drawn from exact coordinates in a frame of the scene's choosing
//! (the board of `draw` takes whole cells; a halved square does not).

use explainer_kit::{Timed, lagged, v};
use fmn::prelude::*;

use super::draw::flat;

/// Moves ready to play.
pub type Moves = Vec<Box<dyn Animation>>;

/// `mob` fading in over `rt` seconds, ready to play with others.
pub fn arrive(stage: &mut Stage<'_>, mob: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// `mob` fading in while it slides `by` (it starts `by` short of its place:
/// a drop is `by` pointing down).
pub fn arrive_sliding(stage: &mut Stage<'_>, mob: Mob, by: Vec3, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_in(stage.arena_mut(), mob, by, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// `mob` fading out over `rt` seconds (its updaters cleared first).
pub fn leave(stage: &mut Stage<'_>, mob: Mob, rt: f64) -> fmn::Result<Box<dyn Animation>> {
    stage.clear_updaters(mob, true);
    let a = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.prepare(a.rt(rt))
}

/// `AnimBuilder`s prepared, to play with arrivals and departures.
pub fn prepared(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>) -> fmn::Result<Moves> {
    moves.into_iter().map(|m| stage.prepare(m)).collect()
}

/// Play `moves` together; with `lag > 0`, each starts `lag` of a move after
/// the one before. Nothing happens for an empty list.
pub fn play(stage: &mut Stage<'_>, moves: Moves, lag: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    if lag > 0.0 {
        let group = lagged(stage, moves, lag)?;
        stage.play(group)?;
    } else {
        stage.play_prepared(moves)?;
    }
    Ok(())
}

/// Bring `mobs` in together (lagged by `lag`), each fading in over `rt`.
pub fn show(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    let mut moves: Moves = Vec::with_capacity(mobs.len());
    for &m in mobs {
        moves.push(arrive(stage, m, rt)?);
    }
    play(stage, moves, lag)
}

/// Take `mobs` away together, each fading out over `rt`.
pub fn hide(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut moves: Moves = Vec::with_capacity(mobs.len());
    for &m in mobs {
        moves.push(leave(stage, m, rt)?);
    }
    play(stage, moves, 0.0)
}

/// Build moves from the stage and play them (`stage` cannot be passed to
/// [`play`] and used to build its moves in one expression).
pub fn play_with(
    stage: &mut Stage<'_>,
    lag: f64,
    build: impl FnOnce(&mut Stage<'_>) -> fmn::Result<Moves>,
) -> fmn::Result<()> {
    let moves = build(stage)?;
    play(stage, moves, lag)
}

/// Play `moves` (builders) together, or lagged: [`play`] for moves not yet
/// prepared.
pub fn together(stage: &mut Stage<'_>, moves: Vec<AnimBuilder>, lag: f64) -> fmn::Result<()> {
    let moves = prepared(stage, moves)?;
    play(stage, moves, lag)
}

/// Scale `mob` by `factor` about the scene point `point`, `rt` seconds. A
/// stroke keeps its width (a zoom is the picture growing, not its lines), so
/// use it on fills, on outlines whose width may stay, and never on a tick
/// whose length is a size (slide those by [`slid_by_zoom`]).
pub fn zoom_about(mob: Mob, factor: f64, point: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale_about(factor, Some(point), None)?)
}

/// Where a point goes when the picture is zoomed by `factor` about `about`.
pub fn zoomed(p: Vec3, about: Vec3, factor: f64) -> Vec3 {
    v(
        about[0] + factor * (p[0] - about[0]),
        about[1] + factor * (p[1] - about[1]),
    )
}

/// The displacement that carries a point-like mark from `p` to where a zoom
/// by `factor` about `about` takes it (the mark keeps its own size).
pub fn slid_by_zoom(p: Vec3, about: Vec3, factor: f64) -> Vec3 {
    let to = zoomed(p, about, factor);
    v(to[0] - p[0], to[1] - p[1])
}

/// A flat rectangle (a piece of a square, a band) with its lower left corner
/// at `at` and size `wide` by `tall`, inset by `inset` on every side so that
/// neighbours show a dark seam. Detached.
pub fn rectangle(stage: &mut Stage<'_>, at: Vec3, wide: f64, tall: f64, inset: f64, color: Srgb, opacity: f64) -> Mob {
    let (x0, y0) = (at[0] + inset, at[1] + inset);
    let (x1, y1) = (at[0] + wide - inset, at[1] + tall - inset);
    let quad = [v(x0, y0), v(x1, y0), v(x1, y1), v(x0, y1)];
    stage
        .arena_mut()
        .add(Polygon::new(quad).style(flat(color, opacity)).build())
}

/// Shrink `mob` about its centre until it is at most `wide` by `tall`.
pub fn fit_in(stage: &mut Stage<'_>, mob: Mob, wide: f64, tall: f64) {
    let (w, h) = (stage.get_width(mob), stage.get_height(mob));
    let k = (wide / w.max(1e-9)).min(tall / h.max(1e-9));
    if k < 1.0 {
        let at = stage.get_center(mob);
        stage.set_width(mob, w * k, false);
        stage.move_to(mob, at, ORIGIN);
    }
}
