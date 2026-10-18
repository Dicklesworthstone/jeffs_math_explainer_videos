//! Chapter 07, four points, five tiles: the lower bound. Our box is exactly
//! B = 2 + 1/√2 wide (about 2.70711, which is less than 2.71: the cutoff is
//! B itself, never a rounded number); suppose five tiles fit a narrower one
//! (07_1, the supposition, drawn as an empty box). Enlarge everything by
//! λ = B/s until the box is ours: every tile grows by λ, its width shown,
//! and none overlap (07_2, one real tile grown with the box). Keep a unit
//! core in each grown tile: does it cover a point? (07_3, a four-second
//! think.) It lies in our box, so by chapters 04 to 06 it covers one, at
//! worst on its edge (07_4). The grown
//! tile reaches (λ − 1)/2 further on every side, so the point is strictly
//! inside (07_5). Five tiles each holding a point strictly inside, no two
//! sharing one: four points are not enough (07_6, a schematic count). So no
//! narrower box holds five, and the turned packing fits 2 + 1/√2: it is the
//! smallest (07_7); there nothing grew, and its points sit on edges (07_8).
//!
//! The argument of `model.rs` § four points, five tiles. Nothing here draws
//! five tiles in a box narrower than B: the one tile grown is a real unit
//! tile (`model::scale_core`) shown in the supposed box and grown with it,
//! its depths computed; the count is a schematic and says so.

use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, corner, dim, dot, fact, fail, faint, flat, ink, line_style, path, polygon, reshape, retrace,
    ring, say, swap, touch, turned, warn,
};
use super::the_corner_lemma::{EDGE_BAND, LAYER_MARKS, LAYER_RING, POINT_R, RING_R_POINT};
use super::the_turned_tile::{BOARD, RING_AT, RING_R, THINK_S};
use crate::model::{self, MIDDLE, Point, Surd, Tile};

pub struct FourPointsFiveTiles {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 8] = ["07_1", "07_2", "07_3", "07_4", "07_5", "07_6", "07_7", "07_8"];

// the chapter's fixed words
pub const SUPPOSED: &str = "supposed, to be ruled out:";
pub const NARROWER: &str = "five tiles in a box narrower than B";
pub const OURS: &str = "our box, exactly";
pub const ONE_OF: &str = "one of the five";
pub const BOX: &str = "box width, about";
pub const SIDE: &str = "tile side, about";
pub const LAMBDA: &str = r"\lambda = \frac{B}{s} > 1";
pub const SCALED: &str = "every length times the same factor";
pub const CORE: &str = "a unit core, same centre and turn";
pub const COVERS: &str = "in our box, B wide: it covers a point";
pub const STRICT: &str = "strictly inside the grown tile";
pub const DEPTH: &str = r"\text{depth} \ge \frac{\lambda - 1}{2} > 0";
pub const SCHEMATIC: &str = "schematic: tiles and points, not a packing";
pub const COUNT: &str = "each tile needs its own point";
pub const SHARE: &str = "a shared point inside both: they overlap";
pub const B_IS: &str = r"B = 2 + \frac{1}{\sqrt{2}}";
pub const SMALLEST: &str = "fits; nothing narrower does";
pub const NO_GROWTH: &str = r"\lambda = 1:\ \text{depth } 0";
pub const SHARED: &str = "so points sit on edges, shared by two tiles";

// ---------------------------------------------------------------- layout

/// The close view of 07_3 to 07_5: our box, B wide, at 2.6 scene units a
/// tile, centred on the box's centre, near the core's.
pub const CLOSE: Board = Board {
    at: [-3.1, -0.25, 0.0],
    k: 2.6,
};
/// The left edge of the panel's text.
pub const TEXT_X: f64 = 1.2;
/// The panel's rows.
pub const ROWS: [f64; 8] = [3.1, 2.55, 1.9, 1.35, 0.8, 0.25, -0.3, -0.85];
/// Where the schematic count of 07_6 stands: the tiles' row and the points'
/// row (y), the first of each (x), the spacing, and an icon's width.
pub const COUNT_ROWS: (f64, f64) = (1.4, -1.2);
pub const COUNT_X: f64 = -5.6;
pub const COUNT_STEP: f64 = 1.05;
pub const ICON: f64 = 0.62;

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    side: f64,
    lambda: f64,
    small: f64,
    about: String,
    margin: String,
    tiles: usize,
    points: usize,
}

fn reads() -> fmn::Result<Reads> {
    for (key, want) in [
        ("scale.grown.held", "inside,inside,inside,inside"),
        ("scale.margin.at1", "0|edge"),
        ("scale.grown.walls", "inside the walls"),
        ("scale.small.walls", "inside the walls"),
        ("scale.shared", "touch|overlap"),
        ("turned.check", "fits"),
        ("middle.packing.inside", "none"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    if !fact("scale.margin.below")?.ends_with("|outside") {
        return Err(fail("scale.margin.below: shrinking does not lose the point".into()));
    }
    if !fact("scale.core.held")?.starts_with("edge,") {
        return Err(fail("scale.core.held: the core's point is not on its edge".into()));
    }
    let count = fact("scale.count")?;
    let (tiles, points) = count
        .split_once('|')
        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        .ok_or_else(|| fail(format!("scale.count is {count:?}")))?;
    let lambda = Surd::ratio(model::GROWTH.0, model::GROWTH.1);
    // B exactly, and its decimal only ever as "about"
    if fact("turned.side")? != model::turned_side().to_string() {
        return Err(fail("turned.side is not the model's B".into()));
    }
    let about = format!("B is about {}", fact("turned.side.5dp")?);
    let margin = fact("scale.margin.2dp")?;
    if margin
        != model::depth_sized(
            &model::scale_core(),
            lambda,
            model::corner_points(model::turned_side())[0],
        )
        .decimals(2)
    {
        return Err(fail("scale.margin.2dp is not the model's".into()));
    }
    Ok(Reads {
        side: model::turned_side().to_f64(),
        lambda: lambda.to_f64(),
        small: (model::turned_side() * lambda.recip()).to_f64(),
        about,
        margin,
        tiles,
        points,
    })
}

/// An exact tile as drawn.
fn as_f64(t: &Tile<Surd>) -> Tile<f64> {
    Tile {
        centre: (t.centre.0.to_f64(), t.centre.1.to_f64()),
        cos: t.cos.to_f64(),
        sin: t.sin.to_f64(),
    }
}

/// The supposed box `small` wide and the shown tile, both grown by `lambda`
/// about the box's corner: the box's outline and the tile's corners (λ
/// wide) in box coordinates. At λ = 1 the tile is the core shrunk with our
/// box, B wide, to the supposed box, a unit tile in it; at λ = B / small it
/// is the core's own place, λ wide.
fn growth_frame(small: f64, core: &Tile<f64>, lambda: f64) -> ([Point<f64>; 5], [Point<f64>; 4]) {
    let w = small * lambda;
    let start = model::grown(core, small / model::turned_side().to_f64());
    let tile = model::grown(&start, lambda);
    (
        [(0.0, 0.0), (w, 0.0), (w, w), (0.0, w), (0.0, 0.0)],
        tile.corners_sized(lambda),
    )
}

/// The rings' colours for depths: yellow inside, green on the edge, faint
/// outside.
fn ring_colour(depth: f64) -> Srgb {
    if depth > EDGE_BAND {
        Palette::yellow()
    } else if depth >= -EDGE_BAND {
        touch()
    } else {
        faint()
    }
}

impl SceneConstruct for FourPointsFiveTiles {
    fn name(&self) -> &str {
        "07_four_points_five_tiles"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let side = r.side;
        let at_box = |q: Point<f64>| BOARD.p(side, q);
        let at_close = |q: Point<f64>| CLOSE.p(side, q);
        let core = as_f64(&model::scale_core());
        let pts = model::corner_points(model::turned_side()).map(|p| (p.0.to_f64(), p.1.to_f64()));
        let caption = |stage: &mut Stage<'_>, s: &str, c: Srgb, row: usize| {
            kit.text_left(stage, s, 24.0, c, v(TEXT_X, ROWS[row]))
        };

        // ---------------------------------------------------------- 07_1
        // our box, B wide exactly (its decimal only ever "about"); the
        // supposition: an empty box narrower than B inside its outline
        let line_1 = say(&kit, stage, LINES[0])?;
        let big_outline = path(stage, BOARD.outline(side), line_style(faint(), 3.0));
        let b_def = kit.tex(stage, B_IS, 34.0, ink(), &[], v(TEXT_X + 1.6, ROWS[0]))?;
        let ours = caption(stage, OURS, dim(), 1)?;
        let about = kit.text_left(stage, &r.about, 24.0, dim(), v(TEXT_X + 2.6, ROWS[1]))?;
        swap(stage, &[], &[big_outline, b_def, ours, about], 0.6)?;
        line_1.wait_for(stage, "Suppose five tiles")?;
        let (outline0, tile0) = growth_frame(r.small, &core, 1.0);
        let small_box = path(
            stage,
            outline0.iter().map(|&q| at_box(q)).collect(),
            line_style(ink(), 4.0),
        );
        let supposed = caption(stage, SUPPOSED, warn(), 2)?;
        let narrower = caption(stage, NARROWER, ink(), 3)?;
        swap(stage, &[], &[small_box, supposed, narrower], 0.6)?;

        // ---------------------------------------------------------- 07_2
        // everything grows by λ = B/s about the box's corner (B still on
        // screen): the box to ours, one tile (a real unit tile placed in the
        // supposed box) to λ wide; its width and the box's read live, about
        let line_2 = say(&kit, stage, LINES[1])?;
        let shown = polygon(stage, tile0.iter().map(|&q| at_box(q)).collect(), flat(corner(), 0.85));
        let one_of = kit.text(stage, ONE_OF, 22.0, dim(), at_box((tile0[0].0, tile0[0].1 - 0.25)))?;
        let lambda_tex = kit.tex(stage, LAMBDA, 32.0, ink(), &[], v(TEXT_X + 1.4, ROWS[4]))?;
        swap(stage, &[], &[shown, one_of, lambda_tex], 0.5)?;
        let growth = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let small = r.small;
        let core_for = core;
        stage.add_updater(
            anchor,
            move |s: &mut fmn::mobject::Stage, _| {
                let l = s.tracker_value(growth).unwrap_or(1.0);
                let (outline, tile) = growth_frame(small, &core_for, l);
                retrace(s, small_box, outline.iter().map(|&q| BOARD.p(side, q)).collect());
                reshape(s, shown, tile.iter().map(|&q| BOARD.p(side, q)).collect());
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let label =
            |stage: &mut Stage<'_>, s: &str, row: usize| kit.text_left(stage, s, 28.0, dim(), v(TEXT_X, ROWS[row]));
        let box_label = label(stage, BOX, 5)?;
        let side_label = label(stage, SIDE, 6)?;
        let readout = |stage: &mut Stage<'_>, scale: f64, row: usize| {
            kit.live_value(
                stage,
                move |s: &fmn::mobject::Stage| s.tracker_value(growth).unwrap_or(1.0) * scale,
                2,
                false,
                32.0,
                ink(),
                v(TEXT_X + 3.4, ROWS[row]),
                RIGHT,
            )
        };
        let box_value = readout(stage, small, 5)?;
        let side_value = readout(stage, 1.0, 6)?;
        let scaled = caption(stage, SCALED, dim(), 7)?;
        swap(
            stage,
            &[one_of],
            &[box_label, side_label, box_value, side_value, scaled],
            0.4,
        )?;
        let rt = (line_2.left(stage, 2.0) - 0.2).max(1.5);
        stage.play(tween(growth, side / small, rt)?)?;
        stage.clear_updaters(anchor, true);

        // ---------------------------------------------------------- 07_3
        // close up: the grown tile and its unit core; the question; four
        // seconds to think
        let line_3 = say(&kit, stage, LINES[2])?;
        let lambda = r.lambda;
        let grown_tile = polygon(
            stage,
            core.corners_sized(lambda).iter().map(|&q| at_close(q)).collect(),
            flat(corner(), 0.85),
        );
        let core_fill = polygon(
            stage,
            core.corners().iter().map(|&q| at_close(q)).collect(),
            flat(ink(), 0.16),
        );
        let mut core_edge: Vec<Vec3> = core.corners().iter().map(|&q| at_close(q)).collect();
        core_edge.push(core_edge[0]);
        let core_line = path(stage, core_edge, line_style(ink(), 3.0));
        stage.set_z_index(core_line, LAYER_MARKS, true);
        let core_caption = caption(stage, CORE, ink(), 2)?;
        let mut leaving = vec![big_outline, small_box, shown, supposed, narrower, lambda_tex, scaled];
        leaving.extend([box_label, side_label, box_value, side_value, b_def, ours, about]);
        swap(stage, &leaving, &[grown_tile], 0.6)?;
        line_3.wait_for(stage, "keep a unit tile")?;
        swap(stage, &[], &[core_fill, core_line, core_caption], 0.5)?;
        line_3.wait_for(stage, "Does it cover")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        kit.hold(stage)?;
        let mut elapsed = 0.0;
        stage.add_dt_updater(
            arc,
            move |s: &mut fmn::mobject::Stage, m, dt| {
                elapsed += dt;
                retrace(s, m, arc_points(RING_AT, RING_R, 1.0 - elapsed / THINK_S));
            },
            false,
        )?;
        kit.listen(stage, THINK_S, LINES[2])?;

        // ---------------------------------------------------------- 07_4
        // the four points, ringed by what the core covers: one on its edge
        let line_4 = say(&kit, stage, LINES[3])?;
        let dots: Vec<Mob> = pts
            .iter()
            .map(|&p| dot(stage, at_close(p), POINT_R, Palette::purple()))
            .collect();
        let core_rings: Vec<Mob> = pts
            .iter()
            .map(|&p| {
                let c = ring_colour(model::depth_inside(&core, p));
                ring(stage, at_close(p), RING_R_POINT, c, 5.0)
            })
            .collect();
        for &m in dots.iter().chain(&core_rings) {
            stage.set_z_index(m, LAYER_RING, true);
        }
        let covers = caption(stage, COVERS, touch(), 3)?;
        let mut ins = dots.clone();
        ins.extend(core_rings.iter().copied());
        swap(stage, &[track, arc], &ins, 0.5)?;
        line_4.wait_for(stage, "it covers a point")?;
        swap(stage, &[], &[covers], 0.4)?;

        // ---------------------------------------------------------- 07_5
        // the grown tile reaches (λ − 1)/2 further: the point on the core's
        // edge strictly inside it, the margin drawn and read
        let line_5 = say(&kit, stage, LINES[4])?;
        line_5.wait_for(stage, "strictly inside")?;
        let grown_rings: Vec<Mob> = pts
            .iter()
            .map(|&p| {
                let c = ring_colour(model::depth_sized(&core, lambda, p));
                ring(stage, at_close(p), RING_R_POINT, c, 5.0)
            })
            .collect();
        for &m in &grown_rings {
            stage.set_z_index(m, LAYER_RING, true);
        }
        // from the point on the core's edge straight out to the grown edge
        let edge_point = pts
            .iter()
            .copied()
            .find(|&p| model::depth_inside(&core, p).abs() < EDGE_BAND)
            .ok_or_else(|| fail("the core has no point on its edge".into()))?;
        let (u, w) = model::tile_frame(&core, edge_point);
        let [e1, e2] = core.axes();
        let out = if u.abs() >= w.abs() {
            (u.signum() * e1.0, u.signum() * e1.1)
        } else {
            (w.signum() * e2.0, w.signum() * e2.1)
        };
        let reach = 0.5 * (lambda - 1.0);
        let bar_end = (edge_point.0 + reach * out.0, edge_point.1 + reach * out.1);
        let bar = path(
            stage,
            vec![at_close(edge_point), at_close(bar_end)],
            line_style(warn(), 6.0),
        );
        stage.set_z_index(bar, LAYER_MARKS, true);
        let margin = kit.text(
            stage,
            &r.margin,
            26.0,
            warn(),
            at_close((bar_end.0 + 0.2 * out.0, bar_end.1 + 0.2 * out.1)),
        )?;
        let strict = caption(stage, STRICT, Palette::yellow(), 4)?;
        let depth_tex = kit.tex(stage, DEPTH, 30.0, ink(), &[], v(TEXT_X + 2.2, ROWS[5]))?;
        swap(
            stage,
            &core_rings,
            &[
                grown_rings[0],
                grown_rings[1],
                grown_rings[2],
                grown_rings[3],
                bar,
                margin,
                strict,
                depth_tex,
            ],
            0.5,
        )?;

        // ---------------------------------------------------------- 07_6
        // a schematic count: five tiles, four points, each tile pointing at a
        // point strictly inside it; two must share one, and would overlap
        let line_6 = say(&kit, stage, LINES[5])?;
        let mut close_out = vec![
            grown_tile,
            core_fill,
            core_line,
            bar,
            margin,
            core_caption,
            covers,
            strict,
            depth_tex,
        ];
        close_out.extend(dots.iter().copied());
        close_out.extend(grown_rings.iter().copied());
        let schematic = caption(stage, SCHEMATIC, warn(), 0)?;
        swap(stage, &close_out, &[schematic], 0.5)?;
        let (ty, py) = COUNT_ROWS;
        let icons: Vec<Mob> = (0..r.tiles)
            .map(|i| {
                let x = COUNT_X + COUNT_STEP * i as f64;
                let h = ICON / 2.0;
                polygon(
                    stage,
                    vec![v(x - h, ty - h), v(x + h, ty - h), v(x + h, ty + h), v(x - h, ty + h)],
                    flat(corner(), 0.85),
                )
            })
            .collect();
        let point_x = |j: usize| COUNT_X + COUNT_STEP * (j as f64 + 0.5);
        let marks: Vec<Mob> = (0..r.points)
            .map(|j| dot(stage, v(point_x(j), py), POINT_R * 1.4, Palette::purple()))
            .collect();
        let count = caption(stage, COUNT, ink(), 1)?;
        let mut first = icons.clone();
        first.extend(marks.iter().copied());
        first.push(count);
        swap(stage, &[], &first, 0.5)?;
        line_6.wait_for(stage, "no two can share")?;
        // tile i points at point i, the last tile at the last point too
        let arrows: Vec<Mob> = (0..r.tiles)
            .map(|i| {
                let j = i.min(r.points - 1);
                let x = COUNT_X + COUNT_STEP * i as f64;
                let c = if i >= r.points { warn() } else { dim() };
                path(
                    stage,
                    vec![v(x, ty - ICON / 2.0 - 0.05), v(point_x(j), py + 0.15)],
                    line_style(c, 3.0),
                )
            })
            .collect();
        let crowded = ring(stage, v(point_x(r.points - 1), py), RING_R_POINT * 1.3, warn(), 5.0);
        let share = caption(stage, SHARE, warn(), 2)?;
        let mut second = arrows.clone();
        second.extend([crowded, share]);
        swap(stage, &[], &second, 0.5)?;

        // ---------------------------------------------------------- 07_7
        // the real packing in the box of 2 + 1/√2: it fits, and nothing
        // narrower does
        let line_7 = say(&kit, stage, LINES[6])?;
        let mut count_out = vec![schematic, count, crowded, share];
        count_out.extend(icons);
        count_out.extend(marks);
        count_out.extend(arrows);
        swap(stage, &count_out, &[], 0.5)?;
        let five = model::five_turned();
        let mut packing: Vec<Mob> = five
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let c = if i == MIDDLE { turned() } else { corner() };
                polygon(stage, BOARD.tile(side, &as_f64(t)), flat(c, 0.85))
            })
            .collect();
        packing.push(path(stage, BOARD.outline(side), line_style(ink(), 4.0)));
        let b_is = kit.tex(stage, B_IS, 34.0, ink(), &[], v(TEXT_X + 1.6, ROWS[1]))?;
        swap(stage, &[], &packing, 0.6)?;
        line_7.wait_for(stage, "It is the smallest")?;
        let smallest = caption(stage, SMALLEST, touch(), 3)?;
        swap(stage, &[], &[b_is, smallest], 0.5)?;

        // ---------------------------------------------------------- 07_8
        // nothing grew here: depth 0, each point on two tiles' edges
        let line_8 = say(&kit, stage, LINES[7])?;
        let edge_rings: Vec<Mob> = pts
            .iter()
            .map(|&p| {
                let on = five
                    .tiles
                    .iter()
                    .map(as_f64)
                    .filter(|t| model::depth_inside(t, p).abs() < EDGE_BAND)
                    .count();
                let c = if on == 2 { touch() } else { warn() };
                ring(stage, at_box(p), RING_R_POINT * 0.8, c, 4.0)
            })
            .collect();
        for &m in &edge_rings {
            stage.set_z_index(m, LAYER_RING, true);
        }
        let no_growth = kit.tex(stage, NO_GROWTH, 30.0, ink(), &[], v(TEXT_X + 1.6, ROWS[4]))?;
        swap(stage, &[], &edge_rings, 0.4)?;
        line_8.wait_for(stage, "shared by two tiles")?;
        let shared = caption(stage, SHARED, touch(), 5)?;
        swap(stage, &[], &[no_growth, shared], 0.5)?;

        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chapter's lines are its script's, and every phrase a beat waits
    /// for or times a move by is in the line that beat speaks, exactly (case
    /// included), in the order the beat uses them.
    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("07_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
        let source = include_str!("four_points_five_tiles.rs");
        let mut from = [0usize; LINES.len()];
        let mut cues = 0;
        for (k, _) in source.match_indices("line_") {
            let rest = &source[k + "line_".len()..];
            let Some(n) = rest.chars().next().and_then(|c| c.to_digit(10)) else {
                continue;
            };
            let call = &rest[1..];
            let Some(phrase) = [".wait_for(stage, \"", ".at(\""]
                .iter()
                .find_map(|p| call.strip_prefix(p))
                .and_then(|s| s.split('"').next())
            else {
                continue;
            };
            let i = n as usize - 1;
            let text = script.text(LINES[i]).unwrap();
            let at = text[from[i]..]
                .find(phrase)
                .unwrap_or_else(|| panic!("{} has no {phrase:?} at or after its earlier cues", LINES[i]));
            from[i] += at;
            cues += 1;
        }
        assert!(cues >= 7, "only {cues} cues found");
    }

    /// The growth keeps units honest: the shown tile starts a unit tile in
    /// the supposed box, ends λ wide at the core's own place in our box, B
    /// wide, and both fit their boxes; the box's outline ends B wide.
    #[test]
    fn the_growth_scales_the_tile_with_the_box() {
        let side = model::turned_side().to_f64();
        let lambda = Surd::ratio(model::GROWTH.0, model::GROWTH.1).to_f64();
        let small = side / lambda;
        let core = as_f64(&model::scale_core());
        let (outline, start) = growth_frame(small, &core, 1.0);
        let width = |c: &[Point<f64>; 4]| ((c[1].0 - c[0].0).powi(2) + (c[1].1 - c[0].1).powi(2)).sqrt();
        assert!((width(&start) - 1.0).abs() < 1e-12, "a unit tile in the supposed box");
        assert!((outline[2].0 - small).abs() < 1e-12);
        let fits = |c: &[Point<f64>], w: f64| {
            c.iter()
                .all(|q| (-1e-12..=w + 1e-12).contains(&q.0) && (-1e-12..=w + 1e-12).contains(&q.1))
        };
        assert!(fits(&start, small));
        let end = side / small;
        assert!((end - lambda).abs() < 1e-12);
        let (outline, last) = growth_frame(small, &core, end);
        assert!((outline[2].0 - side).abs() < 1e-12, "the box ends B wide");
        assert!(
            (width(&last) - lambda).abs() < 1e-12,
            "the tile ends λ wide, not a unit"
        );
        let want = core.corners_sized(lambda);
        for (a, b) in last.iter().zip(want) {
            assert!((a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12);
        }
        assert!(fits(&last, side));
    }

    /// The close view, the panel and the schematic stay in the frame.
    #[test]
    fn the_layout_fits_the_frame() {
        let side = model::turned_side().to_f64();
        let lambda = Surd::ratio(model::GROWTH.0, model::GROWTH.1).to_f64();
        let core = as_f64(&model::scale_core());
        let mut pts: Vec<Point<f64>> = core.corners_sized(lambda).to_vec();
        pts.extend(model::corner_points(model::turned_side()).map(|p| (p.0.to_f64(), p.1.to_f64())));
        for q in pts {
            let p = CLOSE.p(side, q);
            assert!(p[0] > -FRAME_X_RADIUS + 0.3 && p[0] < TEXT_X - 0.4, "{q:?} at {p:?}");
            assert!(p[1].abs() < FRAME_Y_RADIUS - 0.3, "{q:?} at {p:?}");
        }
        assert!(
            BOARD.p(side, (side, side))[0] < TEXT_X - 0.4,
            "the box clears the panel"
        );
        let last_icon = COUNT_X + COUNT_STEP * 4.0 + ICON / 2.0;
        assert!(COUNT_X - ICON / 2.0 > -FRAME_X_RADIUS + 0.3 && last_icon < TEXT_X - 0.3);
        assert!(ROWS.windows(2).all(|r| r[0] > r[1]));
    }

    /// The rings read the depths: the core's point on its edge is green,
    /// strictly inside the grown tile it is yellow.
    #[test]
    fn the_rings_follow_the_depths() {
        let lambda = Surd::ratio(model::GROWTH.0, model::GROWTH.1).to_f64();
        let core = as_f64(&model::scale_core());
        let p = model::corner_points(model::turned_side())[0];
        let p = (p.0.to_f64(), p.1.to_f64());
        assert!(model::depth_inside(&core, p).abs() < EDGE_BAND);
        assert!(model::depth_sized(&core, lambda, p) > EDGE_BAND);
        assert!(model::depth_sized(&core, 0.9, p) < -EDGE_BAND);
    }
}
