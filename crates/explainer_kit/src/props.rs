//! Shared idea-carrying props (plan §3.3): each builder draws one recurring
//! idea the same way in every video, and follows the engine rules in
//! [`crate::kit`] (one flat fill per mobject, placement-only motion, no
//! long strokes under moving things, small live readouts only).
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like [`Kit::text`]: add them with `stage.add_to_scene`, or bring them in
//! with an animation. Video-specific props (space-time lanes, the danger
//! meter, the byte clock) live in their explainer crates unless a second
//! video needs them.

use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::mobject::ShapeTag;
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::animatic::wrap;
use crate::kit::Kit;
use crate::narration::{frames_covering, wait_frames};
use crate::palette::{Palette, hex, stroke_style};
use crate::play::{Timed, lagged};
use crate::plot::{Frame2, children, rect, v};

/// How long a prediction question stays on screen before its answer (plan
/// §5.1: 1.5–3 s).
pub const PREDICT_HOLD_S: f64 = 2.0;

/// The fill behind cards and lower-thirds: a shade above the background.
fn card_fill() -> Srgb {
    hex("#1A2130")
}

/// An empty parent in the arena, for grouping mobjects that are already
/// there (a group of `VMobject`s is built with `v_group` instead).
fn group_of(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// A rounded card of `w` × `h` centred at `at`, behind text.
fn card(w: f64, h: f64, at: Vec3) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(card_fill(), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at))
}

// ------------------------------------------------------------ proportion bar

/// A share of a whole: the marked share of a catch, the useless share of the
/// possible cards. The whole is a dim bar; the share is the same height, lit,
/// from the left edge.
pub struct ProportionBar {
    pub group: Mob,
    pub whole: Mob,
    pub part: Mob,
}

/// A proportion bar with its left edge's midpoint at `left`.
pub fn proportion_bar(
    stage: &mut Stage<'_>,
    left: Vec3,
    width: f64,
    height: f64,
    share: f64,
    on: Srgb,
    off: Srgb,
) -> fmn::Result<ProportionBar> {
    let dl = v(left[0], left[1] - height / 2.0);
    let flat = |c: Srgb, o: f64| Style::default().fill(c, o).stroke(c, 0.0, 0.0);
    let whole = rect(dl, width, height, flat(off, 0.35))?;
    let part = rect(dl, width * share.clamp(0.0, 1.0), height, flat(on, 0.9))?;
    let group = stage.arena_mut().add(v_group([whole, part]));
    let kids = children(stage, group);
    Ok(ProportionBar {
        group,
        whole: kids[0],
        part: kids[1],
    })
}

// ---------------------------------------------------------------- histogram

/// Counts that pile up (the caught-twice counts of Video A's 09; the spares
/// of Video B's 13): one flat bar per nonzero count, standing on the
/// window's bottom edge. `bars` are (x at the bar's centre, count) in data
/// units; `bar_width` is in data units too.
pub fn histogram(frame: &Frame2, bars: &[(f64, f64)], bar_width: f64, color: Srgb) -> fmn::Result<VMobject> {
    let scale_x = frame.width / (frame.x.1 - frame.x.0);
    let style = Style::default().fill(color, 0.85).stroke(color, 0.0, 0.0);
    let mut out = Vec::new();
    for &(x, count) in bars {
        if count <= 0.0 {
            continue;
        }
        let base = frame.p(x, frame.y.0);
        let top = frame.p(x, count);
        let w = bar_width * scale_x;
        out.push(rect(v(base[0] - w / 2.0, base[1]), w, top[1] - base[1], style)?);
    }
    Ok(v_group(out))
}

/// The histogram grows from the data: each bar rises from the axis
/// (`GrowFromEdge` from below), lagged left to right.
pub fn grow_histogram(stage: &mut Stage<'_>, group: Mob, lag: f64) -> fmn::Result<AnimationGroup> {
    let mut anims = Vec::new();
    for bar in children(stage, group) {
        let a = fmn::animation::grow_from_edge(stage.arena_mut(), bar, DOWN, None)?;
        anims.push(stage.prepare(a)?);
    }
    lagged(stage, anims, lag)
}

// ------------------------------------------------------- cards and verdicts

/// A formula on a card: where a derivation lands. `t2c` colours substrings.
pub fn equation_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    src: &str,
    size: f64,
    t2c: &[(&str, Srgb)],
    at: Vec3,
) -> fmn::Result<Mob> {
    let tex = kit.tex(stage, src, size, Palette::ink(), t2c, at)?;
    let (w, h) = (stage.get_width(tex), stage.get_height(tex));
    let back = stage.arena_mut().add(card(w + 0.6, h + 0.45, at)?);
    group_of(stage, &[back, tex])
}

/// A verdict on a claim: "holds" in the good role, "fails" in the warn
/// role, inside an outline of the same colour.
pub fn verdict_box(kit: &Kit, stage: &mut Stage<'_>, text: &str, ok: bool, at: Vec3) -> fmn::Result<Mob> {
    let color = if ok { Palette::green() } else { Palette::red() };
    let t = kit.text(stage, text, 34.0, color, at)?;
    let (w, h) = (stage.get_width(t), stage.get_height(t));
    let outline = Rectangle::new()
        .width(w + 0.5)
        .height(h + 0.35)
        .corner_radius(0.1)
        .style(stroke_style(color, 3.0))
        .build()?
        .moved_to(at);
    let o = stage.arena_mut().add(outline);
    group_of(stage, &[o, t])
}

// ------------------------------------------------------------------ bit row

/// A byte as eight switches (Video B: bits, ex-or, bytes as polynomials):
/// cells left to right from the top bit, a lit cell for a one, and the place
/// values above.
pub struct BitRow {
    pub group: Mob,
    pub cells: Vec<Mob>,
    pub digits: Vec<Mob>,
    /// Empty when built without place values.
    pub places: Vec<Mob>,
}

/// The bits of `value`, cells `cell` wide, the row centred at `at`.
pub fn bit_row(kit: &Kit, stage: &mut Stage<'_>, value: u8, places: bool, cell: f64, at: Vec3) -> fmn::Result<BitRow> {
    let (mut cells, mut digits, mut labels) = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..8u32 {
        let bit = 7 - k;
        let one = value >> bit & 1 == 1;
        let c = v(at[0] + (f64::from(k) - 3.5) * cell, at[1]);
        let fill = if one { Palette::hero() } else { card_fill() };
        let r = Rectangle::new()
            .width(cell * 0.92)
            .height(cell * 0.92)
            .style(
                Style::default()
                    .fill(fill, if one { 0.3 } else { 0.9 })
                    .stroke(Palette::faint(), 1.5, 1.0),
            )
            .build()?
            .moved_to(c);
        cells.push(stage.arena_mut().add(r));
        let ink = if one { Palette::hero() } else { Palette::dim() };
        digits.push(kit.mono(stage, if one { "1" } else { "0" }, cell * 40.0, ink, c)?);
        if places {
            let p = (1u32 << bit).to_string();
            labels.push(kit.text(stage, &p, cell * 22.0, Palette::dim(), v(c[0], c[1] + 0.78 * cell))?);
        }
    }
    let all: Vec<Mob> = cells.iter().chain(&digits).chain(&labels).copied().collect();
    let group = group_of(stage, &all)?;
    Ok(BitRow {
        group,
        cells,
        digits,
        places: labels,
    })
}

// --------------------------------------------------------- prediction pause

/// The prediction pause (⟨predict⟩, plan §5.1): let the question finish,
/// mark the moment for the score (its fixed two-note sting answers
/// "predict"), bring in a large "?" at `at`, and hold it for
/// [`PREDICT_HOLD_S`]. Returns the "?" so the scene can clear it with the
/// answer.
pub fn predict_pause(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 90.0, Palette::hero(), at)?;
    let a = fade_in(stage.arena_mut(), q, ORIGIN, 1.0)?;
    stage.play(a.rt(0.3))?;
    wait_frames(stage, frames_covering(PREDICT_HOLD_S, stage.scene().time().fps()))?;
    Ok(q)
}

// ---------------------------------------------------- row strips (heat map)

/// A grid of coloured cells as one strip per row (spike S-3): each strip is
/// a polyline whose per-point stroke colours paint the cells, so the grid
/// is `rows` mobjects however many cells it has, and recolouring writes the
/// colour lane in place (never a rebuild, never `always_redraw`). Heat maps
/// over a lake, the byte table, region maps.
pub struct RowStrips {
    pub group: Mob,
    pub rows: Vec<Mob>,
    pub cols: usize,
}

/// Stroke colours for one strip, in the record layout [`strip_points`]
/// writes: a clear lead-in (3 records), 4 records per cell, a clear tail
/// (4 records), so the round caps at the ends paint nothing.
fn strip_rgba(colors: &[Srgb]) -> Vec<[f32; 4]> {
    let rgba = |c: Srgb, a: f32| [c.r as f32, c.g as f32, c.b as f32, a];
    let mut out = Vec::with_capacity(3 + 4 * colors.len() + 4);
    out.extend([rgba(colors[0], 0.0); 3]);
    for &c in colors {
        out.extend([rgba(c, 1.0); 4]);
    }
    out.extend([rgba(colors[colors.len() - 1], 0.0); 4]);
    out
}

/// The points of one strip: cells `cell` wide from `left`, along height `y`.
/// A null curve (a repeated anchor) before each cell makes the colour jump
/// at the boundary instead of blending. The clear lead-in and tail are each
/// half the stroke width (`thick`) long: the round caps' radius.
fn strip_points(left: f64, y: f64, cell: f64, thick: f64, n: usize) -> Vec<Vec3> {
    let q = thick / 4.0;
    let mut p = vec![v(left - 2.0 * q, y), v(left - q, y), v(left, y)];
    for k in 0..n {
        let (a0, a1) = (left + cell * k as f64, left + cell * (k + 1) as f64);
        let last = v(a0, y);
        p.extend([last, last, v((a0 + a1) / 2.0, y), v(a1, y)]);
    }
    let end = v(left + cell * n as f64, y);
    p.extend([end, end, v(end[0] + q, y), v(end[0] + 2.0 * q, y)]);
    p
}

impl RowStrips {
    /// Build the grid with its top-left corner at `top_left`, cells
    /// `(width, height)`, `colors[row][col]` (every row the same length).
    pub fn build(
        stage: &mut Stage<'_>,
        top_left: Vec3,
        (cell_w, cell_h): (f64, f64),
        colors: &[Vec<Srgb>],
    ) -> fmn::Result<Self> {
        let cols = colors.first().map_or(0, Vec::len);
        let bad = |m: &'static str| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(m));
        if cols == 0 || colors.iter().any(|r| r.len() != cols) {
            return Err(bad("RowStrips: every row needs the same, nonzero number of cells"));
        }
        let mut rows = Vec::with_capacity(colors.len());
        for (i, row) in colors.iter().enumerate() {
            let y = top_left[1] - cell_h * (i as f64 + 0.5);
            let points = strip_points(top_left[0], y, cell_w, cell_h, cols);
            let vertices = points.len().div_ceil(2);
            // stroke width in Reference units (0.01 scene units each): one cell
            let style = Style::default().fill(row[0], 0.0).stroke(row[0], cell_h / 0.01, 1.0);
            let vm = VMobject::from_points(points)
                .with_style(style)
                .with_shape(ShapeTag::Polyline {
                    vertices,
                    closed: false,
                });
            let mut m = Mobject::from(vm);
            let rgba = strip_rgba(row);
            if m.buffer.len() != rgba.len() || !m.buffer.write_range("stroke_rgba", 0, rgba.as_flattened()) {
                return Err(bad("RowStrips: the strip's colour lane does not match its points"));
            }
            rows.push(stage.arena_mut().add(m));
        }
        let group = group_of(stage, &rows)?;
        Ok(Self { group, rows, cols })
    }

    /// Recolour every cell in place (callable from an updater). `false` if
    /// the colours do not fit the grid or a row is gone.
    pub fn recolor(&self, arena: &mut Arena, colors: &[Vec<Srgb>]) -> bool {
        if colors.len() != self.rows.len() || colors.iter().any(|r| r.len() != self.cols) {
            return false;
        }
        self.rows.iter().zip(colors).all(|(&row, cs)| {
            arena
                .get_mut(row)
                .is_some_and(|e| e.buffer.write_range("stroke_rgba", 0, strip_rgba(cs).as_flattened()))
        })
    }
}

// ------------------------------------------------------------------- swarm

/// Where one swarm member is and which way it faces (radians from +x).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
}

/// Many small things moving together (fish, packets, phones): one group,
/// each member a copy of one template built at the origin (pointing along
/// +x) and moved only by placement, never by rewriting points, driven by
/// one dt updater on the group.
pub struct Swarm {
    pub group: Mob,
    pub members: Vec<Mob>,
}

impl Swarm {
    /// One member per pose, coloured per member (`colors`), or keeping the
    /// template's own colours (`None`: a gold fish with its translucent
    /// halo).
    ///
    /// # Panics
    /// If `colors` and `poses` differ in length.
    pub fn build(
        stage: &mut Stage<'_>,
        template: &VMobject,
        poses: &[Pose],
        colors: Option<&[Srgb]>,
    ) -> fmn::Result<Self> {
        let copies: Vec<VMobject> = match colors {
            Some(colors) => {
                assert_eq!(poses.len(), colors.len(), "Swarm: one colour per member");
                colors
                    .iter()
                    .map(|&c| {
                        template.clone().map_style_deep(move |s| {
                            s.fill(c, s.fill_opacity).stroke(c, s.stroke_width, s.stroke_opacity)
                        })
                    })
                    .collect()
            }
            None => vec![template.clone(); poses.len()],
        };
        let group = stage.arena_mut().add(v_group(copies));
        let members = children(stage, group);
        for (&m, p) in members.iter().zip(poses) {
            stage.rotate(m, p.heading, OUT, Some(ORIGIN), None);
            stage.shift(m, [p.x, p.y, 0.0]);
        }
        Ok(Self { group, members })
    }

    /// Move the swarm every frame: `step(dt, poses)` advances the poses (it
    /// owns the motion model), and each member is turned about where it was
    /// and shifted to where it is now. `poses` must be the poses it was
    /// built with.
    pub fn drive(
        &self,
        stage: &mut Stage<'_>,
        mut poses: Vec<Pose>,
        mut step: impl FnMut(f64, &mut [Pose]) + 'static,
    ) -> fmn::Result<()> {
        let members = self.members.clone();
        stage.add_dt_updater(
            self.group,
            move |s, _, dt| {
                if dt <= 0.0 {
                    return;
                }
                let before = poses.clone();
                step(dt, &mut poses);
                for ((&m, o), n) in members.iter().zip(&before).zip(&poses) {
                    if n.heading != o.heading {
                        s.rotate(m, n.heading - o.heading, OUT, Some([o.x, o.y, 0.0]), None);
                    }
                    if n.x != o.x || n.y != o.y {
                        s.shift(m, [n.x - o.x, n.y - o.y, 0.0]);
                    }
                }
            },
            false,
        )?;
        Ok(())
    }
}

// --------------------------------------------------------------------- glow

/// A soft halo around `source` (the marked fish, the winning card): `layers`
/// widening strokes (8–12 reads as a glow) from a faint `color`, adding up
/// to `spread` Reference stroke units. Draw the bright core on top.
pub fn glow(source: &VMobject, color: Srgb, layers: usize, spread: f64) -> VMobject {
    let faint = source
        .clone()
        .map_style_deep(move |s| s.stroke(color, s.stroke_width.max(1.0), 0.06));
    v_highlight(&faint, layers, spread, [color, color])
}

// -------------------------------------------------- panels and lower-thirds

/// The "for programmers" code panel, built just off the right edge of the
/// frame; [`CodePanel::slide_in`] brings it on.
pub struct CodePanel {
    pub group: Mob,
    pub lines: Vec<Mob>,
    /// How far it slides in.
    pub width: f64,
}

impl CodePanel {
    /// The panel: a "for programmers" header and the code lines in the
    /// bundled typewriter face, `size` points, left-aligned.
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, code: &[&str], size: f64, width: f64) -> fmn::Result<Self> {
        let line_h = size / 48.0 * 0.5;
        let h = 0.9 + line_h * code.len() as f64;
        let centre = v(FRAME_X_RADIUS + width / 2.0 + 0.1, 0.0);
        let back = stage.arena_mut().add(card(width, h, centre)?);
        let left = centre[0] - width / 2.0 + 0.3;
        let top = centre[1] + h / 2.0 - 0.35;
        let header = kit.text_left(stage, "for programmers", 24.0, Palette::dim(), v(left, top))?;
        let mut lines = Vec::with_capacity(code.len());
        for (i, line) in code.iter().enumerate() {
            let y = top - 0.5 - line_h * i as f64;
            let m = kit.mono(stage, line, size, Palette::ink(), v(left, y))?;
            stage.move_to(m, v(left, y), LEFT);
            lines.push(m);
        }
        let mut all = vec![back, header];
        all.extend(&lines);
        let group = group_of(stage, &all)?;
        Ok(Self { group, lines, width })
    }

    /// The slide onto the screen's right side, `run_time` seconds.
    pub fn slide_in(&self, run_time: f64) -> fmn::Result<AnimBuilder> {
        Ok(self
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(run_time),
                ..AnimateArgs::default()
            })?
            .shift([-(self.width + 0.45), 0.0, 0.0])?)
    }
}

/// Characters per lower-third line at 26 pt across the band.
const LOWER_THIRD_CHARS: usize = 92;
/// The most lines a lower-third shows; longer text belongs in the description.
const LOWER_THIRD_LINES: usize = 3;

/// A lower-third band at the foot of the frame: a `label` in the hero role
/// ("precisely:", "names to search:") and the wrapped `text` beside it.
pub fn lower_third(kit: &Kit, stage: &mut Stage<'_>, label: &str, text: &str) -> fmn::Result<Mob> {
    let lines = wrap(text, LOWER_THIRD_CHARS, LOWER_THIRD_LINES);
    let line_h = 0.36;
    let h = 0.3 + line_h * lines.len().max(1) as f64;
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let back = stage.arena_mut().add(card(w, h, centre)?);
    let left = -w / 2.0 + 0.25;
    let top = centre[1] + h / 2.0 - 0.15 - line_h / 2.0;
    let tag = kit.text_left(stage, label, 26.0, Palette::hero(), v(left, top))?;
    let indent = left + stage.get_width(tag) + 0.2;
    let mut all = vec![back, tag];
    for (i, l) in lines.iter().enumerate() {
        all.push(kit.text_left(stage, l, 26.0, Palette::ink(), v(indent, top - line_h * i as f64))?);
    }
    group_of(stage, &all)
}

/// "precisely:" — the exact version of a simplification (plan §5.1 rule 11).
pub fn precisely(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    lower_third(kit, stage, "precisely:", text)
}

/// "names to search:" — the terms a viewer would type (plan §5.1 rule 26).
pub fn names_to_search(kit: &Kit, stage: &mut Stage<'_>, names: &[&str]) -> fmn::Result<Mob> {
    lower_third(kit, stage, "names to search:", &names.join(" · "))
}

/// The receipts card (plan §5.1 rule 24): where every number came from, one
/// line each, centred.
pub fn receipts_card(kit: &Kit, stage: &mut Stage<'_>, lines: &[&str]) -> fmn::Result<Mob> {
    let line_h = 0.42;
    let h = 1.1 + line_h * lines.len() as f64;
    let w = 2.0 * FRAME_X_RADIUS - 2.0;
    let back = stage.arena_mut().add(card(w, h, ORIGIN)?);
    let title = kit.text(stage, "Receipts", 40.0, Palette::hero(), v(0.0, h / 2.0 - 0.45))?;
    let left = -w / 2.0 + 0.4;
    let mut all = vec![back, title];
    for (i, l) in lines.iter().enumerate() {
        let y = h / 2.0 - 1.05 - line_h * i as f64;
        all.push(kit.text_left(stage, l, 26.0, Palette::ink(), v(left, y))?);
    }
    group_of(stage, &all)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;

    /// Run `f` as a scene's `construct`, headless, at 10 fps.
    fn run(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("the probe scene runs");
    }

    /// Mobjects drawn: every root's family, counted.
    fn drawn(stage: &Stage<'_>) -> usize {
        stage.roots().iter().map(|&r| stage.family(r).len()).sum()
    }

    #[test]
    fn builders_make_the_families_they_promise() {
        run(|stage| {
            let kit = Kit::new()?;
            let row = bit_row(&kit, stage, 0b1010_0101, true, 0.6, ORIGIN)?;
            assert_eq!((row.cells.len(), row.digits.len(), row.places.len()), (8, 8, 8));
            let plain = bit_row(&kit, stage, 0, false, 0.6, ORIGIN)?;
            assert!(plain.places.is_empty());
            let bar = proportion_bar(stage, v(-3.0, 0.0), 6.0, 0.4, 0.1, Palette::hero(), Palette::dim())?;
            let (pw, ww) = (stage.get_width(bar.part), stage.get_width(bar.whole));
            assert!((pw - 0.6).abs() < 1e-6, "a tenth of 6 wide: part {pw}, whole {ww}");
            assert!((ww - 6.0).abs() < 1e-6, "whole {ww}");
            let frame = Frame2 {
                center: ORIGIN,
                width: 8.0,
                height: 4.0,
                x: (0.0, 10.0),
                y: (0.0, 50.0),
            };
            let h = histogram(&frame, &[(1.0, 10.0), (2.0, 0.0), (3.0, 50.0)], 0.8, Palette::blue())?;
            let hm = stage.arena_mut().add(h);
            assert_eq!(children(stage, hm).len(), 2, "zero counts draw no bar");
            let tall = children(stage, hm)[1];
            assert!(
                (stage.get_height(tall) - 4.0).abs() < 1e-9,
                "the top count spans the window"
            );
            stage.add_to_scene(hm)?;
            let grow = grow_histogram(stage, hm, 0.2)?;
            stage.play(grow)?;
            let card = equation_card(&kit, stage, r"\hat N = \frac{Mn}{k}", 48.0, &[], ORIGIN)?;
            assert_eq!(children(stage, card).len(), 2);
            let verdict = verdict_box(&kit, stage, "holds", true, ORIGIN)?;
            assert_eq!(children(stage, verdict).len(), 2);
            let poses: Vec<Pose> = (0..50)
                .map(|i| Pose {
                    x: f64::from(i) * 0.1,
                    y: 0.0,
                    heading: 0.0,
                })
                .collect();
            let swarm = Swarm::build(stage, &Circle::new().radius(0.05).build(), &poses, None)?;
            assert_eq!(swarm.members.len(), 50);
            Ok(())
        });
    }

    #[test]
    fn row_strips_recolour_in_place_and_a_rebuild_would_grow_the_arena() {
        let counts = Rc::new(RefCell::new(Vec::new()));
        let seen = Rc::clone(&counts);
        run(move |stage| {
            let colors = |shade: f64| -> Vec<Vec<Srgb>> {
                (0..6)
                    .map(|r| {
                        (0..9)
                            .map(|c| {
                                fmn::core::color::interpolate_color(
                                    Palette::blue(),
                                    Palette::hero(),
                                    (shade + 0.01 * f64::from(r * c)).min(1.0),
                                )
                            })
                            .collect()
                    })
                    .collect()
            };
            let grid = RowStrips::build(stage, v(-2.0, 1.5), (0.3, 0.3), &colors(0.0))?;
            assert_eq!(grid.rows.len(), 6);
            stage.add_to_scene(grid.group)?;
            let before: Vec<f32> = stage
                .get(grid.rows[0])
                .unwrap()
                .buffer
                .read_column("stroke_rgba")
                .unwrap();
            let rows = grid.rows.clone();
            let shared = Rc::new(grid);
            let g = Rc::clone(&shared);
            let tick = Rc::new(RefCell::new(0.0));
            let t = Rc::clone(&tick);
            let log = Rc::clone(&seen);
            let logging = Rc::new(Cell::new(true));
            let on = Rc::clone(&logging);
            stage.add_dt_updater(
                rows[0],
                move |s, _, dt| {
                    *t.borrow_mut() += dt;
                    let shade = (*t.borrow() / 10.0).min(1.0);
                    assert!(g.recolor(s, &colors(shade)));
                    let n: usize = s.roots().iter().map(|&r| s.family(r).len()).sum();
                    if on.get() {
                        log.borrow_mut().push(n);
                    }
                },
                false,
            )?;
            wait_frames(stage, 100)?;
            let after: Vec<f32> = stage.get(rows[0]).unwrap().buffer.read_column("stroke_rgba").unwrap();
            assert_ne!(before, after, "the colours changed");
            assert!(
                !shared.recolor(stage.arena_mut(), &[vec![Palette::blue(); 9]]),
                "a mismatched grid is refused"
            );
            // planted negative: adding a mobject every tick (a rebuild without
            // a removal) grows the scene, and the counter sees it
            logging.set(false);
            let start = drawn(stage);
            let lone = stage.arena_mut().add(Circle::new().radius(0.1).build());
            let driver = stage.arena_mut().add(VMobject::new());
            stage.add_to_scene(driver)?;
            stage.add_dt_updater(
                driver,
                move |s, _, _| {
                    let copy = s.copy_family(lone).expect("the template copies");
                    s.add_to_scene(copy).expect("the copy is added");
                },
                false,
            )?;
            wait_frames(stage, 10)?;
            assert!(drawn(stage) >= start + 10, "the counter sees a rebuild leak");
            Ok(())
        });
        let counts = counts.borrow();
        assert!(counts.len() >= 100, "the recolouring updater ran every frame");
        assert!(
            counts.iter().all(|&n| n == counts[0]),
            "recolouring never changes the mobject count: {counts:?}"
        );
    }

    #[test]
    fn a_swarm_moves_by_placement_only() {
        run(|stage| {
            let fish = Polygon::new(vec![v(0.12, 0.0), v(-0.08, 0.05), v(-0.08, -0.05)]).build();
            let poses: Vec<Pose> = (0..40)
                .map(|i| Pose {
                    x: -2.0 + 0.1 * f64::from(i),
                    y: 0.0,
                    heading: 0.0,
                })
                .collect();
            let swarm = Swarm::build(stage, &fish, &poses, Some(&vec![Palette::gold(); 40]))?;
            stage.add_to_scene(swarm.group)?;
            let points = |s: &Stage<'_>, m: Mob| s.get(m).unwrap().buffer.read_column("point").unwrap();
            let first = swarm.members[0];
            let (p0, c0) = (points(stage, first), stage.get_center(first));
            // every member swims up at 1 unit/s, turning a quarter turn a second
            swarm.drive(stage, poses, |dt, ps| {
                for p in ps {
                    p.y += dt;
                    p.heading += std::f64::consts::FRAC_PI_2 * dt;
                }
            })?;
            wait_frames(stage, 20)?;
            let c1 = stage.get_center(first);
            assert!(
                (c1[1] - c0[1] - 2.0).abs() < 0.05,
                "two seconds at one unit a second: {c0:?} -> {c1:?}"
            );
            assert!((c1[0] - c0[0]).abs() < 0.05);
            assert_eq!(points(stage, first), p0, "object-space points are never rewritten");
            Ok(())
        });
    }

    #[test]
    fn lower_thirds_and_panels_stay_in_their_places() {
        run(|stage| {
            let kit = Kit::new()?;
            let long = "after mixing, being marked is independent of position; equal catchability is a separate \
                        assumption, and the estimate aims at the lake on marking day, so deaths leave it unchanged";
            let lt = precisely(&kit, stage, long)?;
            let bottom = stage.get_bounding_box_point(lt, DOWN)[1];
            let top = stage.get_bounding_box_point(lt, UP)[1];
            assert!(
                bottom >= -FRAME_Y_RADIUS && top < -FRAME_Y_RADIUS + 2.0,
                "a band at the foot: {bottom}..{top}"
            );
            assert!(stage.get_width(lt) <= 2.0 * FRAME_X_RADIUS, "inside the frame");
            let names = names_to_search(
                &kit,
                stage,
                &["capture-recapture", "Lincoln–Petersen", "Chapman estimator"],
            )?;
            assert!(children(stage, names).len() >= 3);
            let panel = CodePanel::build(&kit, stage, &["k = rng.choice(N, n)", "est = M * n / k"], 30.0, 5.0)?;
            assert!(
                stage.get_bounding_box_point(panel.group, LEFT)[0] > FRAME_X_RADIUS,
                "built off screen"
            );
            stage.add_to_scene(panel.group)?;
            let slide = panel.slide_in(0.5)?;
            stage.play(slide)?;
            let right = stage.get_bounding_box_point(panel.group, RIGHT)[0];
            assert!(
                right < FRAME_X_RADIUS && right > FRAME_X_RADIUS - 1.0,
                "on screen at the right edge: {right}"
            );
            let receipts = receipts_card(&kit, stage, &["every number: facts.rs, checked by oracle.py"])?;
            assert_eq!(children(stage, receipts).len(), 3);
            Ok(())
        });
    }

    #[test]
    fn the_prediction_pause_holds_its_question() {
        run(|stage| {
            let kit = Kit::new()?;
            let t0 = stage.scene().time().to_f64();
            let q = predict_pause(&kit, stage, v(5.6, -2.6))?;
            let held = stage.scene().time().to_f64() - t0;
            assert!(
                (PREDICT_HOLD_S + 0.29..PREDICT_HOLD_S + 0.5).contains(&held),
                "held {held} s"
            );
            assert!(stage.scene().mobjects().contains(&q), "the ? is on screen");
            Ok(())
        });
    }

    #[test]
    fn hud_text_and_numbers_are_fixed_in_frame_and_plain_ones_are_not() {
        run(|stage| {
            let kit = Kit::new()?;
            let fixed = |s: &Stage<'_>, m: Mob| {
                s.family(m)
                    .iter()
                    .any(|&f| s.get(f).unwrap().uniforms().is_fixed_in_frame == 1.0)
            };
            let label = kit.text_hud(stage, "estimated lake size", 30.0, Palette::ink(), v(0.0, 3.0))?;
            assert!(fixed(stage, label));
            let t = stage.add_value_tracker(10_000.0);
            let hud = kit.live_number_hud(stage, t, 0, false, 30.0, Palette::ink(), v(0.0, 2.5), ORIGIN)?;
            let plain = kit.live_number(stage, t, 0, false, 30.0, Palette::ink(), v(0.0, 2.0), ORIGIN)?;
            wait_frames(stage, 3)?;
            assert!(fixed(stage, hud), "the rebuilt number stays pinned");
            assert!(!fixed(stage, plain), "a plain readout moves with the camera");
            Ok(())
        });
    }

    #[test]
    fn a_glow_is_a_stack_of_faint_layers() {
        let dot = Circle::new().radius(0.1).build();
        let g = glow(&dot, Palette::hero(), 10, 12.0);
        assert_eq!(g.children().len(), 10);
    }
}
