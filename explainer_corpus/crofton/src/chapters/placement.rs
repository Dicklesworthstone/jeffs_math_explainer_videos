//! The sheet at any placement (chapters 04 and 05). A placement is a
//! direction and a slide ([`Pose`]). The sheet moves by sliding across its
//! lines and by turning about a point, as a rigid sheet does ([`Move`]).
//! The live sheet, its crossings, the count, the two controls and the halo
//! that picks out a line are redrawn every frame from the trackers of one
//! [`Placement`], so what is on screen is the model at the placement shown.
//! Card centimetres unless a name says frame; directions in radians, a
//! line's height along the normal [−sin β, cos β], as in
//! [`sheet`](super::sheet). Not registered yet: root wires it in
//! (`docs/candidate-04-05.md`).

use std::f64::consts::PI;
use std::rc::Rc;

use explainer_kit::{Kit, Palette, TextKind, Timed, ink_style, v};
use fmn::prelude::*;

use super::draw::{card_point, dashed, flat, line_style};
use super::sheet::{self, DOT_R, LINE_INSET, LINE_W};
use crate::model::{CARD_H, CARD_W, Curve, GAP, P2};
use crate::palette;

// ------------------------------------------------------------ the geometry

fn normal(beta: f64) -> P2 {
    [-beta.sin(), beta.cos()]
}

/// Where the sheet lies: its lines run at `beta` radians, line k at height
/// `slide + k × GAP` (`slide` cm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub beta: f64,
    pub slide: f64,
}

impl Pose {
    pub const fn new(beta: f64, slide: f64) -> Self {
        Self { beta, slide }
    }

    /// The same lines named as the controls show them: a direction from 0
    /// up to half a turn, and a slide from 0 up to one gap. Half a turn
    /// reverses the normal, so it negates the slide.
    pub fn canonical(self) -> Self {
        let turns = (self.beta / PI).floor();
        let slide = if (turns as i64).rem_euclid(2) == 1 {
            -self.slide
        } else {
            self.slide
        };
        Self {
            beta: self.beta - turns * PI,
            slide: slide.rem_euclid(GAP),
        }
    }

    /// Whether `other` puts every line where one of these lies (to `tol`).
    #[cfg(test)]
    pub fn same_lines(self, other: Self, tol: f64) -> bool {
        let (a, b) = (self.canonical(), other.canonical());
        let near = |x: f64| {
            let r = x.rem_euclid(GAP);
            r.min(GAP - r) < tol
        };
        let turned = a.beta - b.beta;
        if turned.abs() < tol {
            near(a.slide - b.slide)
        } else if PI - turned.abs() < tol {
            // one just short of half a turn, the other just past none
            near(a.slide + b.slide)
        } else {
            false
        }
    }
}

/// The sheet moving from `from`: turned by `turn` radians about `pin` (card
/// cm) and slid `by` cm across its lines, both in proportion as the move's
/// share `u` runs from 0 to 1. A turn about the card's middle keeps the
/// slide; about any other point every line's height moves by
/// (n(β) − n(β₀)) · pin, as a rigid sheet's lines do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub from: Pose,
    pub turn: f64,
    pub by: f64,
    pub pin: P2,
}

impl Move {
    pub const fn slide(from: Pose, by: f64) -> Self {
        Self {
            from,
            turn: 0.0,
            by,
            pin: [0.0, 0.0],
        }
    }

    pub const fn turn(from: Pose, turn: f64, pin: P2) -> Self {
        Self {
            from,
            turn,
            by: 0.0,
            pin,
        }
    }

    pub fn at(&self, u: f64) -> Pose {
        let beta = self.from.beta + self.turn * u;
        let (n, n0) = (normal(beta), normal(self.from.beta));
        Pose {
            beta,
            slide: self.from.slide + self.by * u + (n[0] - n0[0]) * self.pin[0] + (n[1] - n0[1]) * self.pin[1],
        }
    }

    pub fn end(&self) -> Pose {
        self.at(1.0)
    }
}

/// The point midway between two lines, or on one, at `pose`'s direction:
/// half a turn about it puts the lines back where they were. `offset` is
/// how far along the lines from the card's middle, cm.
pub fn half_turn_pin(pose: Pose, offset: f64) -> P2 {
    let (n, d) = (normal(pose.beta), [pose.beta.cos(), pose.beta.sin()]);
    // heights c with 2c ≡ 2 × slide (mod GAP): c = slide + k × GAP / 2
    let c = pose.slide - (pose.slide / (GAP / 2.0)).round() * (GAP / 2.0);
    [c * n[0] + offset * d[0], c * n[1] + offset * d[1]]
}

/// One count a move passes through and the share of the move it lasts,
/// `from` to `to` (0 to 1). Found at even steps of the move; a change lies
/// between two steps, so a run's ends are put midway.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Run {
    pub count: usize,
    pub from: f64,
    pub to: f64,
}

pub fn runs(curve: &Curve, mv: &Move, steps: usize) -> Vec<Run> {
    let half = 0.5 / steps as f64;
    let mut out: Vec<Run> = Vec::new();
    for i in 0..=steps {
        let u = i as f64 / steps as f64;
        let p = mv.at(u);
        let count = sheet::crossings(curve, p.beta, p.slide).len();
        match out.last_mut() {
            Some(r) if r.count == count => r.to = u,
            Some(r) => {
                r.to += half;
                out.push(Run {
                    count,
                    from: u - half,
                    to: u,
                });
            }
            None => out.push(Run {
                count,
                from: 0.0,
                to: u,
            }),
        }
    }
    out
}

/// The rectangle the lines are drawn in: the card less [`LINE_INSET`] at
/// its sides, as [`sheet::Sheet`] draws them at direction 0.
const CLIP: [f64; 2] = [CARD_W / 2.0 - LINE_INSET, CARD_H / 2.0];

/// The part inside [`CLIP`] of the line at height `h` across direction
/// `beta`, if any. At direction 0 a line is kept only strictly inside the
/// card's height, as [`sheet::lines_on_card`] keeps it.
pub fn clipped(beta: f64, h: f64) -> Option<[P2; 2]> {
    let (n, d) = (normal(beta), [beta.cos(), beta.sin()]);
    // the line's point nearest the card's middle, and how far along the
    // line, either way, it stays inside
    let o = [h * n[0], h * n[1]];
    let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
    for axis in 0..2 {
        let half = CLIP[axis];
        if d[axis].abs() < 1e-12 {
            if o[axis].abs() >= half {
                return None;
            }
        } else {
            let (a, b) = ((-half - o[axis]) / d[axis], (half - o[axis]) / d[axis]);
            lo = lo.max(a.min(b));
            hi = hi.min(a.max(b));
        }
    }
    (hi - lo > 1e-9).then(|| {
        [
            [o[0] + lo * d[0], o[1] + lo * d[1]],
            [o[0] + hi * d[0], o[1] + hi * d[1]],
        ]
    })
}

/// Every line of the sheet at `pose` that lies on the card, lowest first,
/// as (line, ends): line k as [`sheet::crossings`] numbers it.
pub fn lines_at(pose: Pose) -> Vec<(i64, [P2; 2])> {
    let reach = CLIP[0].hypot(CLIP[1]);
    let first = ((-reach - pose.slide) / GAP).floor() as i64;
    let last = ((reach - pose.slide) / GAP).ceil() as i64;
    (first..=last)
        .filter_map(|k| clipped(pose.beta, pose.slide + k as f64 * GAP).map(|ends| (k, ends)))
        .collect()
}

// ------------------------------------------------------- the live sheet

/// The trackers a live sheet reads: the move under way and how far along
/// it is.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    beta: Mob,
    slide: Mob,
    turn: Mob,
    by: Mob,
    pin_x: Mob,
    pin_y: Mob,
    along: Mob,
}

impl Placement {
    /// The sheet lying still at `pose`.
    pub fn new(stage: &mut Stage<'_>, pose: Pose) -> Self {
        let mut tracker = |x: f64| stage.add_value_tracker(x);
        Self {
            beta: tracker(pose.beta),
            slide: tracker(pose.slide),
            turn: tracker(0.0),
            by: tracker(0.0),
            pin_x: tracker(0.0),
            pin_y: tracker(0.0),
            along: tracker(0.0),
        }
    }

    /// The move under way, as the trackers hold it.
    pub fn current(&self, s: &fmn::mobject::Stage) -> Move {
        let get = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
        Move {
            from: Pose::new(get(self.beta), get(self.slide)),
            turn: get(self.turn),
            by: get(self.by),
            pin: [get(self.pin_x), get(self.pin_y)],
        }
    }

    /// The placement shown.
    pub fn pose(&self, s: &fmn::mobject::Stage) -> Pose {
        self.current(s).at(self.share(s))
    }

    /// How far along the move under way (0 to 1).
    pub fn share(&self, s: &fmn::mobject::Stage) -> f64 {
        s.tracker_value(self.along).unwrap_or(0.0)
    }

    fn set(&self, stage: &mut Stage<'_>, mv: Move) -> fmn::Result<()> {
        for (m, x) in [
            (self.beta, mv.from.beta),
            (self.slide, mv.from.slide),
            (self.turn, mv.turn),
            (self.by, mv.by),
            (self.pin_x, mv.pin[0]),
            (self.pin_y, mv.pin[1]),
            (self.along, 0.0),
        ] {
            stage.set_tracker_value(m, x)?;
        }
        Ok(())
    }

    /// Ready a move from the placement shown: turn by `turn` about `pin`
    /// and slide by `by`. Nothing moves until [`Placement::run`].
    pub fn begin(&self, stage: &mut Stage<'_>, turn: f64, by: f64, pin: P2) -> fmn::Result<Move> {
        let mv = Move {
            from: self.pose(stage),
            turn,
            by,
            pin,
        };
        self.set(stage, mv)?;
        Ok(mv)
    }

    /// Carry the move under way through `legs` in turn, each a share of
    /// the move to reach (0 to 1) and its seconds, at `rate`; at the end
    /// the move is exactly done.
    pub fn run(&self, stage: &mut Stage<'_>, legs: &[(f64, f64)], rate: fn(f64) -> f64) -> fmn::Result<()> {
        for &(to, seconds) in legs {
            stage.play(glide(self.along, to, seconds, rate)?)?;
        }
        stage.set_tracker_value(self.along, 1.0)?;
        Ok(())
    }

    /// Make `planned` from the placement shown, through `legs` at `rate`:
    /// refused unless the sheet stands where the plan starts it.
    pub fn make(
        &self,
        stage: &mut Stage<'_>,
        planned: &Move,
        legs: &[(f64, f64)],
        rate: fn(f64) -> f64,
    ) -> fmn::Result<()> {
        let mv = self.begin(stage, planned.turn, planned.by, planned.pin)?;
        if (mv.from.beta - planned.from.beta).abs() > 1e-9 || (mv.from.slide - planned.from.slide).abs() > 1e-9 {
            return Err(super::draw::fail(format!(
                "the sheet stands at {:?}, the plan starts at {:?}",
                mv.from, planned.from
            )));
        }
        self.run(stage, legs, rate)
    }

    /// Name the placement shown the controls' way ([`Pose::canonical`]),
    /// no move under way. The lines, crossings, count, knob and needle do
    /// not change; a halo on a numbered line would, so clear halos first.
    pub fn settle(&self, stage: &mut Stage<'_>) -> fmn::Result<Pose> {
        let pose = self.pose(stage).canonical();
        self.set(stage, Move::slide(pose, 0.0))?;
        Ok(pose)
    }
}

/// A tracker to `value` over `rt` seconds at `rate` (an fmn-core rate).
pub fn glide(tracker: Mob, value: f64, rt: f64, rate: fn(f64) -> f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(rate),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// Fade the detached `ins` in and `outs` out while `moves` (each with its
/// own run time) play, all at once, the fades `rt` seconds.
pub fn at_once(stage: &mut Stage<'_>, ins: &[Mob], outs: &[Mob], moves: Vec<AnimBuilder>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in outs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for &m in ins {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for m in moves {
        anims.push(stage.prepare(m)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// A stroke at an opacity, and no fill.
pub fn faded_line(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// The opacity tracker `alpha` holds, 0 to 1 (1 if it is not a tracker).
pub fn tracked(s: &fmn::mobject::Stage, alpha: Mob) -> f64 {
    s.tracker_value(alpha).unwrap_or(1.0).clamp(0.0, 1.0)
}

/// The sheet's lines at the placement shown, `dim` as [`sheet::Sheet`]
/// draws them, at the opacity `alpha` holds. In the scene.
pub fn live_lines(stage: &mut Stage<'_>, place: Placement, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let lines: Vec<VMobject> = lines_at(place.pose(s))
            .into_iter()
            .map(|(_, [a, b])| {
                polyline(vec![card_point(a), card_point(b)])
                    .style(faded_line(Palette::dim(), LINE_W, o))
                    .build()
            })
            .collect();
        s.add(v_group(lines))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// `curve`'s crossings at the placement shown, gold dots as
/// [`sheet::crossing_dot`] draws them, at the opacity `alpha` holds. In the
/// scene.
pub fn live_dots(stage: &mut Stage<'_>, place: Placement, curve: &Curve, alpha: Mob) -> fmn::Result<Mob> {
    let curve = curve.clone();
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let dots: Vec<VMobject> = sheet::crossings(&curve, p.beta, p.slide)
            .iter()
            .map(|c| {
                Circle::new()
                    .radius(DOT_R)
                    .arc_center(card_point(c.at))
                    .style(flat(palette::touch(), o))
                    .build()
            })
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A live count from `count`, gold digits with their right edge at `at`,
/// at the opacity `alpha` holds (so it can fade, unlike `Kit::live_value`).
/// In the scene.
pub fn live_count(
    kit: &Kit,
    stage: &mut Stage<'_>,
    count: impl Fn(&fmn::mobject::Stage) -> usize + 'static,
    alpha: Mob,
    size: f64,
    at: Vec3,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let d = DecimalNumber::new(count(s) as f64)
            .num_decimal_places(0)
            .include_sign(false)
            .font_size(size)
            .color(palette::touch())
            .build(&book)
            .expect("bundled digits always lay out");
        explainer_kit::glyphs::record(TextKind::Text, d.num_string());
        let m = s.add(d.into_vmob());
        s.move_to(m, at, RIGHT);
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// `word` redrawn every frame (so it fades with its count's `alpha`), its
/// left edge at `at`. In the scene.
pub fn live_word(
    kit: &Kit,
    stage: &mut Stage<'_>,
    word: &'static str,
    alpha: Mob,
    size: f64,
    at: Vec3,
) -> fmn::Result<Mob> {
    let book = Rc::clone(&kit.book);
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let t = Text::new(word)
            .font_size(size)
            .style(ink_style(palette::touch()))
            .build(&book)
            .expect("the bundled face sets every word on screen");
        explainer_kit::glyphs::record(TextKind::Text, word);
        let m = s.add(t);
        s.move_to(m, at, LEFT);
        s.set_family_opacity(m, o);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Stroked pieces widened (or narrowed) to `width`, their colour and fill
/// unchanged, `rt` seconds (as 03 picks out the coast).
pub fn widen(mobs: &[Mob], width: f64, rt: f64) -> fmn::Result<Vec<AnimBuilder>> {
    mobs.iter()
        .map(|&m| -> fmn::Result<AnimBuilder> {
            Ok(m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(rt),
                    ..AnimateArgs::default()
                })?
                .set_stroke(None, Some(width), None)?)
        })
        .collect()
}

/// A halo along line `k` of the sheet at the placement shown: a wide, faint
/// `ink` stroke that picks the line out without restyling it, at the
/// opacity `alpha` holds. In the scene: make it before the lines, so they
/// are drawn over it.
pub fn live_halo(stage: &mut Stage<'_>, place: Placement, k: i64, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, p) = (tracked(s, alpha), place.pose(s));
        let halo: Vec<VMobject> = clipped(p.beta, p.slide + k as f64 * GAP)
            .into_iter()
            .map(|[a, b]| {
                polyline(vec![card_point(a), card_point(b)])
                    .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY * o))
                    .build()
            })
            .collect();
        s.add(v_group(halo))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

pub const HALO_W: f64 = 14.0;
pub const HALO_OPACITY: f64 = 0.22;

/// A halo along frame points: the same wide faint stroke, still. Detached.
pub fn halo_along(stage: &mut Stage<'_>, points: Vec<Vec3>) -> Mob {
    stage.arena_mut().add(
        polyline(points)
            .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY))
            .build(),
    )
}

/// The point a turn is made about: a small `ink` ring. Detached.
pub fn pivot(stage: &mut Stage<'_>, at: P2) -> Mob {
    let ring = Circle::new()
        .radius(PIVOT_R)
        .arc_center(card_point(at))
        .style(line_style(Palette::ink(), 2.5))
        .build();
    stage.arena_mut().add(ring)
}

pub const PIVOT_R: f64 = 0.07;

/// Where the sheet's lines lie at `pose`, left behind as faint dashes, so
/// the viewer can see the lines come back onto them. Detached.
pub fn footprints(stage: &mut Stage<'_>, pose: Pose) -> fmn::Result<Vec<Mob>> {
    lines_at(pose)
        .into_iter()
        .map(|(_, [a, b])| dashed(stage, a, b, Palette::faint()))
        .collect()
}

// ------------------------------------------------------------ the controls

/// The two choices a placement is (04_5), in the band under the coast,
/// frame units: the slide, a track from 0 to one gap with a knob; the
/// direction, a half dial from 0° to 180° with a needle that runs the way
/// the lines run.
pub const SLIDER_Y: f64 = -2.5;
pub const SLIDER_X: [f64; 2] = [-5.4, -2.4];
const SLIDER_NAME_X: f64 = -5.95;
const KNOB_R: f64 = 0.09;
pub const DIAL_AT: Vec3 = [3.4, -2.6, 0.0];
pub const DIAL_R: f64 = 0.6;
const NEEDLE_R: f64 = 0.55;
const DIAL_NAME_X: f64 = 1.45;
const END_TICK: f64 = 0.1;
const CONTROL_LABEL: f64 = 20.0;
const CONTROL_NAME: f64 = 22.0;
const CONTROL_W: f64 = 2.5;
pub const SLIDE_NAME: &str = "slide";
pub const DIRECTION_NAME: &str = "direction";
const DIAL_ENDS: [&str; 2] = [r"0^\circ", r"180^\circ"];

/// Where the knob stands for `pose`: its slide from 0 to one gap, the
/// controls' way.
pub fn knob_x(pose: Pose) -> f64 {
    SLIDER_X[0] + (SLIDER_X[1] - SLIDER_X[0]) * pose.canonical().slide / GAP
}

/// The needle for `pose`: a diameter of the dial along the lines, so half
/// a turn leaves it where it was.
pub fn needle(pose: Pose) -> [Vec3; 2] {
    let (c, s) = (NEEDLE_R * pose.beta.cos(), NEEDLE_R * pose.beta.sin());
    [v(DIAL_AT[0] - c, DIAL_AT[1] - s), v(DIAL_AT[0] + c, DIAL_AT[1] + s)]
}

/// The dial's half circle, from 0° (right) over the top to 180° (left).
pub fn dial_arc() -> Vec<Vec3> {
    (0..=48)
        .map(|i| {
            let a = PI * i as f64 / 48.0;
            v(DIAL_AT[0] + DIAL_R * a.cos(), DIAL_AT[1] + DIAL_R * a.sin())
        })
        .collect()
}

pub fn slider_track() -> Vec<Vec3> {
    vec![v(SLIDER_X[0], SLIDER_Y), v(SLIDER_X[1], SLIDER_Y)]
}

pub struct Controls {
    /// The track, its ticks, the dial, its baseline and their words.
    pub fixed: Vec<Mob>,
    /// The knob's and the needle's opacity (they start hidden).
    pub knob_alpha: Mob,
    pub needle_alpha: Mob,
}

impl Controls {
    /// The fixed parts detached; the knob and the needle in the scene,
    /// bound to `place`, hidden until their trackers rise.
    pub fn new(kit: &Kit, stage: &mut Stage<'_>, place: Placement) -> fmn::Result<Self> {
        let mut fixed = Vec::new();
        let ink = |w: f64| line_style(Palette::ink(), w);
        fixed.push(
            stage
                .arena_mut()
                .add(polyline(slider_track()).style(ink(CONTROL_W)).build()),
        );
        for x in SLIDER_X {
            let tick = polyline(vec![v(x, SLIDER_Y - END_TICK), v(x, SLIDER_Y + END_TICK)])
                .style(ink(CONTROL_W))
                .build();
            fixed.push(stage.arena_mut().add(tick));
        }
        let ends = ["0".to_owned(), format!("{} cm", crate::facts::fixed(GAP, 0))];
        for (x, word) in SLIDER_X.iter().zip(&ends) {
            fixed.push(kit.text(stage, word, CONTROL_LABEL, Palette::dim(), v(*x, SLIDER_Y - 0.35))?);
        }
        fixed.push(kit.text(
            stage,
            SLIDE_NAME,
            CONTROL_NAME,
            Palette::ink(),
            v(SLIDER_NAME_X, SLIDER_Y),
        )?);
        fixed.push(
            stage
                .arena_mut()
                .add(polyline(dial_arc()).style(ink(CONTROL_W)).build()),
        );
        let base = polyline(vec![
            v(DIAL_AT[0] - DIAL_R, DIAL_AT[1]),
            v(DIAL_AT[0] + DIAL_R, DIAL_AT[1]),
        ])
        .style(line_style(Palette::faint(), 1.5))
        .build();
        fixed.push(stage.arena_mut().add(base));
        for (side, end) in [(1.0, DIAL_ENDS[0]), (-1.0, DIAL_ENDS[1])] {
            let at = v(DIAL_AT[0] + side * (DIAL_R + 0.32), DIAL_AT[1]);
            fixed.push(kit.tex(stage, end, CONTROL_LABEL, Palette::dim(), &[], at)?);
        }
        fixed.push(kit.text(
            stage,
            DIRECTION_NAME,
            CONTROL_NAME,
            Palette::ink(),
            v(DIAL_NAME_X, DIAL_AT[1]),
        )?);

        let knob_alpha = stage.add_value_tracker(0.0);
        let knob = stage.always_redraw(move |s| {
            let o = tracked(s, knob_alpha);
            let k = Circle::new()
                .radius(KNOB_R)
                .arc_center(v(knob_x(place.pose(s)), SLIDER_Y))
                .style(flat(Palette::ink(), o))
                .build();
            s.add(k)
        });
        stage.add_to_scene(knob)?;
        let needle_alpha = stage.add_value_tracker(0.0);
        let needle_mob = stage.always_redraw(move |s| {
            let o = tracked(s, needle_alpha);
            let [a, b] = needle(place.pose(s));
            s.add(polyline(vec![a, b]).style(faded_line(Palette::ink(), 3.0, o)).build())
        });
        stage.add_to_scene(needle_mob)?;
        Ok(Self {
            fixed,
            knob_alpha,
            needle_alpha,
        })
    }
}

// ------------------------------------------------------------ shared words

/// 04's last question, which 05 opens on, where the count was.
pub const QUESTION: &str = "average count = ?";
pub const QUESTION_AT: Vec3 = [-2.2, sheet::TOP_Y, 0.0];
/// The live count's digits end here; its word starts [`WORD_GAP`] after.
pub const COUNT_RIGHT: Vec3 = [-2.95, sheet::TOP_Y, 0.0];
pub const WORD_GAP: f64 = 0.15;

/// "crossings": every count chapters 04 and 05 show is more than one.
pub const CROSSINGS: &str = "crossings";

/// The sealed envelope as 03 left it, at the band over the coast's left: a
/// green body, its flap, "true length" under it. Detached. (A copy of 03's
/// private `envelope`; see the candidate's integration notes.)
pub const ENVELOPE_AT: Vec3 = [-5.6, 3.05, 0.0];
pub const ENVELOPE: [f64; 2] = [1.0, 0.62];
pub const SEALED_Y: f64 = 2.5;
pub const SEALED: &str = "true length";
const SEALED_SIZE: f64 = 20.0;

pub fn envelope(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let ([w, h], [x, y, _]) = (ENVELOPE, ENVELOPE_AT);
    let body = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.04)
        .style(flat(palette::length(), 0.35))
        .build()?
        .moved_to(ENVELOPE_AT);
    let body = stage.arena_mut().add(body);
    let flap = polyline(vec![
        v(x - w / 2.0, y + h / 2.0),
        v(x, y - 0.05),
        v(x + w / 2.0, y + h / 2.0),
    ])
    .style(line_style(palette::length(), 2.5))
    .build();
    let flap = stage.arena_mut().add(flap);
    let label = kit.text(stage, SEALED, SEALED_SIZE, palette::length(), v(x, SEALED_Y))?;
    Ok(vec![body, flap, label])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::fixed;
    use crate::model::{self, CARD_CENTRE, CARD_UNIT};

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    fn deg(d: f64) -> f64 {
        d.to_radians()
    }

    /// At direction 0 the live lines are [`sheet::Sheet`]'s: the same five
    /// heights and the same ends, to the bit.
    #[test]
    fn at_direction_zero_the_live_lines_are_the_sheets() {
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let live = lines_at(Pose::new(0.0, slide));
        let still = sheet::lines_on_card(slide);
        let x = CARD_W / 2.0 - LINE_INSET;
        assert_eq!(live.len(), still.len());
        for ((_, [a, b]), h) in live.iter().zip(&still) {
            assert_eq!((*a, *b), ([-x, *h], [x, *h]));
        }
        // and the line numbers are the crossings' own
        let lowest = sheet::crossings(&coast, 0.0, slide)
            .iter()
            .map(|c| c.line)
            .min()
            .unwrap();
        let on = live.iter().find(|(_, [a, _])| (a[1] - (-1.55)).abs() < 1e-12).unwrap();
        assert_eq!(on.0, lowest);
    }

    /// A drawn line's ends lie on it and on the rectangle's edge, at every
    /// direction; lines that miss the card are left out.
    #[test]
    fn drawn_lines_end_on_the_cards_edge() {
        for i in 0..36 {
            let beta = PI * i as f64 / 36.0;
            for (k, ends) in lines_at(Pose::new(beta, 0.37)) {
                let h = 0.37 + k as f64 * GAP;
                for p in ends {
                    let n = normal(beta);
                    assert!((p[0] * n[0] + p[1] * n[1] - h).abs() < 1e-9);
                    let edge = (p[0].abs() - CLIP[0]).abs().min((p[1].abs() - CLIP[1]).abs());
                    assert!(edge < 1e-9 && p[0].abs() <= CLIP[0] + 1e-9 && p[1].abs() <= CLIP[1] + 1e-9);
                }
            }
        }
        assert!(clipped(deg(45.0), 9.0).is_none());
        assert!(clipped(0.0, 4.5).is_none() && clipped(0.0, 4.45).is_some());
    }

    /// A turn about a point moves every line as a rigid sheet would: a
    /// point on line k, turned about the pin, lies on line k at the end.
    #[test]
    fn a_turn_about_a_point_is_rigid() {
        for (pin, turn) in [
            ([0.0, 0.0], deg(45.0)),
            ([1.3, -2.1], deg(170.0)),
            ([-4.0, 3.5], deg(-30.0)),
        ] {
            let mv = Move::turn(Pose::new(deg(20.0), 0.6), turn, pin);
            let (start, end) = (mv.from, mv.end());
            for k in -3..=3 {
                let h = start.slide + k as f64 * GAP;
                let (n, d) = (normal(start.beta), [start.beta.cos(), start.beta.sin()]);
                let p = [h * n[0] + 2.5 * d[0], h * n[1] + 2.5 * d[1]];
                let (c, s) = (turn.cos(), turn.sin());
                let q = [p[0] - pin[0], p[1] - pin[1]];
                let turned = [pin[0] + c * q[0] - s * q[1], pin[1] + s * q[0] + c * q[1]];
                let m = normal(end.beta);
                let height = turned[0] * m[0] + turned[1] * m[1];
                assert!((height - (end.slide + k as f64 * GAP)).abs() < 1e-9);
            }
        }
    }

    /// Repeats (04_6 to 04_9): one gap's slide, or half a turn about a
    /// point on a line or midway between two, puts every line where one
    /// was; half a turn about the card's middle, at 1.20 cm, does not, and
    /// neither does half a gap.
    #[test]
    fn a_gap_or_half_a_turn_about_the_right_point_repeats() {
        let at = Pose::new(deg(45.0), 1.2);
        assert!(Move::slide(at, GAP).end().same_lines(at, 1e-9));
        assert!(!Move::slide(at, GAP / 2.0).end().same_lines(at, 1e-3));
        for offset in [-3.0, 0.0, 2.0] {
            let pin = half_turn_pin(at, offset);
            let end = Move::turn(at, PI, pin).end();
            assert!(end.same_lines(at, 1e-9), "{pin:?} {end:?}");
        }
        let pin = half_turn_pin(at, 0.0);
        assert_eq!(
            (fixed(pin[0], 4), fixed(pin[1], 4)),
            ("-0.1414".into(), "0.1414".into())
        );
        assert!(!Move::turn(at, PI, [0.0, 0.0]).end().same_lines(at, 1e-3));
        // the first placement's own half-turn point lies on its middle line
        let first = Pose::new(0.0, 0.45);
        assert_eq!(half_turn_pin(first, 0.0), [0.0, 0.45]);
        assert!(Move::turn(first, PI, [0.0, 0.45]).end().same_lines(first, 1e-9));
    }

    /// Naming lines the controls' way: half a turn negates the slide, a gap
    /// wraps it, and a pose and its name put the lines in the same places.
    #[test]
    fn canonical_names_keep_the_lines() {
        let a = Pose::new(deg(225.0), 0.8).canonical();
        assert!((a.beta - deg(45.0)).abs() < 1e-12 && (a.slide - 1.2).abs() < 1e-12);
        let b = Pose::new(deg(180.0), -0.45).canonical();
        assert!(b.beta.abs() < 1e-12 && (b.slide - 0.45).abs() < 1e-12);
        for p in [
            Pose::new(deg(300.0), 5.1),
            Pose::new(deg(-60.0), -0.7),
            Pose::new(deg(45.0), 3.2),
        ] {
            let c = p.canonical();
            assert!(p.same_lines(c, 1e-9) && (0.0..PI).contains(&c.beta) && (0.0..GAP).contains(&c.slide));
            // the same segments on the card, perhaps drawn from the other end
            let (drawn, named) = (lines_at(p), lines_at(c));
            assert_eq!(drawn.len(), named.len());
            for (_, [a, b]) in &drawn {
                let found = named.iter().any(|(_, [x, y])| {
                    (dist(*a, *x) < 1e-9 && dist(*b, *y) < 1e-9) || (dist(*a, *y) < 1e-9 && dist(*b, *x) < 1e-9)
                });
                assert!(found, "{p:?}: {a:?} to {b:?}");
            }
        }
        // a pose that is not a repeat is told apart
        assert!(!Pose::new(deg(45.0), 1.2).same_lines(Pose::new(deg(45.0), 0.8), 1e-3));
        assert!(!Pose::new(deg(45.0), 1.2).same_lines(Pose::new(deg(135.0), 1.2), 1e-3));
        // the knob and the needle show a pose and its name the same way
        let (p, c) = (Pose::new(deg(225.0), 0.8), Pose::new(deg(45.0), 1.2));
        assert!((knob_x(p) - knob_x(c)).abs() < 1e-12);
        let (np, nc) = (needle(p), needle(c));
        assert!((np[0][0] - nc[1][0]).abs() < 1e-12 && (np[0][1] - nc[1][1]).abs() < 1e-12);
    }

    /// 04_1's slide, 1.5 cm up from the first placement (`slide.distance`),
    /// passes the counts 7, 9, 8, 7 (`slide.sequence`); the 8 is the
    /// briefest, from 0.99 to 1.05 cm past the start (`slide.shortest_from`,
    /// `slide.shortest_to`, the oracle's 15000 steps).
    #[test]
    fn the_slide_passes_seven_nine_eight_seven() {
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let r = runs(&coast, &Move::slide(first, 1.5), 15000);
        let counts: Vec<usize> = r.iter().map(|x| x.count).collect();
        assert_eq!(counts, [7, 9, 8, 7]);
        let brief = r
            .iter()
            .min_by(|a, b| (a.to - a.from).total_cmp(&(b.to - b.from)))
            .unwrap();
        assert_eq!(brief.count, 8);
        assert_eq!(
            (fixed(1.5 * brief.from, 2), fixed(1.5 * brief.to, 2)),
            ("0.99".into(), "1.05".into())
        );
        // the 8: the upper line has slipped off the coast's far end, the
        // lower line not yet off its near end
        let (start, end) = coast.ends();
        let step = 1.5 / 15000.0;
        assert!(((end[1] - first.slide).rem_euclid(GAP) - 1.5 * brief.from).abs() < step);
        assert!(((start[1] - first.slide).rem_euclid(GAP) - 1.5 * brief.to).abs() < step);
        assert!(((start[1] - first.slide).rem_euclid(GAP) - 1.05).abs() < 1e-9);
    }

    /// 04_3's eighth of a turn settles at the oracle's slide for 45°,
    /// 1.20 cm, with 8 crossings (`turn.45.slide`, `turn.45.crossings`);
    /// a gap later, and half a turn about its point later, still 8.
    #[test]
    fn an_eighth_of_a_turn_settles_at_eight() {
        let coast = model::coast();
        let beta = deg(45.0);
        let slide = sheet::shown_slide(&coast, beta);
        assert_eq!(fixed(slide, 2), "1.20");
        let count = |p: Pose| sheet::crossings(&coast, p.beta, p.slide).len();
        let at = Pose::new(beta, slide);
        assert_eq!(count(at), 8);
        assert_eq!(count(Move::slide(at, GAP).end()), 8);
        assert_eq!(count(Move::turn(at, PI, half_turn_pin(at, 0.0)).end()), 8);
    }

    /// The controls keep their rooms: the band under the coast and over
    /// the card's foot, apart from each other, inside the card.
    #[test]
    fn the_controls_keep_their_rooms() {
        let coast = model::coast();
        let low = coast
            .upto(coast.length, 0.02)
            .into_iter()
            .map(|p| card_point(p)[1])
            .fold(f64::INFINITY, f64::min);
        for top in [DIAL_AT[1] + DIAL_R, SLIDER_Y + 0.3] {
            assert!(top < low - 0.4, "{low}");
        }
        const {
            let card_foot = CARD_CENTRE[1] - CARD_UNIT * CARD_H / 2.0;
            let card_side = CARD_UNIT * CARD_W / 2.0;
            assert!(DIAL_AT[1] - NEEDLE_R > card_foot + 0.1);
            assert!(SLIDER_Y - 0.35 - 0.12 > card_foot + 0.1);
            assert!(SLIDER_NAME_X - 0.4 > -card_side && DIAL_AT[0] + DIAL_R + 0.6 < card_side);
            assert!(SLIDER_X[1] + 0.4 < DIAL_NAME_X - 0.6 && DIAL_NAME_X + 0.6 < DIAL_AT[0] - DIAL_R - 0.6);
            assert!(NEEDLE_R < DIAL_R);
            // the count's room and the question's, right of the envelope
            assert!(ENVELOPE_AT[0] + ENVELOPE[0] / 2.0 + 0.5 < COUNT_RIGHT[0] - 0.5);
            assert!(ENVELOPE_AT[0] + ENVELOPE[0] / 2.0 + 0.3 < QUESTION_AT[0] - 1.6);
        }
    }
}
