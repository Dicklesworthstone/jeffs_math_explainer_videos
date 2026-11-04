//! 05_gallery: a tour of enhanced phase portraits (storyboard 05_1–05_9).
//!
//! - z³ − 1: three zeros, and a big loop whose image turns three times.
//! - (z − 1)/(z + 1)²: a simple zero, and a pole of order two whose small
//!   loop turns backwards twice.
//! - e^z: no zeros or poles, direction by height, size by how far right,
//!   one period 2π.
//! - sin z and tan z on one window: zeros on the real line, growth off it,
//!   tan settling toward ±i.
//! - Γ: the integral's half-plane, then the poles at 0, −1, −2, … marked as
//!   they are named.
//! - ζ: the series' half-plane, the pole at 1, the zeros at −2, −4, …, and
//!   a strip up the line Re s = ½ with the first four zeros, each found by
//!   Newton's method in the scene (the "precisely" of 05_8 is literal).
//! - The six pictures side by side, then one tile of z³ − 1 traced exactly
//!   (Newton along the band lines) to show it is nearly square.

use std::f64::consts::TAU;

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::counting::TURNS;
use super::draw::{
    Painter, Plane, Spoken, carded_lines, dashed, dot, fact, fact_list, fade_in_all, fade_out_all, fail, flat, ground,
    hero, line_style, marks, pole, pole_mark, precisely_now, say, wait_until, walker, zero_mark,
};
use crate::model::{self, C, Func, I, ONE, ZERO};

pub const TITLES_TEX: [&str; 7] = [
    r"f(z) = z^3 - 1",
    r"f(z) = \frac{z-1}{(z+1)^2}",
    r"f(z) = e^z",
    r"f(z) = \sin z",
    r"\tan z = \frac{\sin z}{\cos z}",
    r"f(z) = \Gamma(z)",
    r"\zeta(s) = 1 + \frac{1}{2^s} + \frac{1}{3^s} + \cdots",
];
pub const CUBIC_SAYS: &str = " = zeros inside";
pub const ZDP_SAYS: &str = " (backwards twice)";
pub const SIMPLE_ZERO: &str = "simple zero at 1";
pub const POLE2: &str = "pole of order 2";
pub const NO_ZEROS_POLES: &str = "no zeros, no poles";
pub const BY_HEIGHT: &str = "direction: set by height alone";
pub const BY_RIGHT: &str = "size: set by how far right alone";
pub const PERIOD_TEX: &str = r"2\pi \approx ";
pub const SIN_ZEROS: &str = "zeros: multiples of π, nowhere else";
pub const SIN_REAL: &str = "on the real line: −1 ≤ sin x ≤ 1";
pub const SIN_AT_TEX: [&str; 2] = [r"\sin(i) \approx ", r"\sin(3i) \approx "];
pub const ALTERNATE: &str = "zeros and poles alternate";
pub const TAN_LIMIT_TEX: [&str; 2] = [r"\tan z \to i", r"\tan z \to -i"];
pub const TAN_FAR_TEX: [&str; 2] = [r"\tan(0.4 + 12i) \approx ", r"\tan(0.4 - 12i) \approx "];
pub const FACTORIAL_TEX: &str = r"\Gamma(n+1) = n!";
pub const GAMMA_AT_TEX: [&str; 2] = [r"\Gamma(4) = 3! = ", r"\Gamma(5) = 4! = "];
pub const INTEGRAL_RIGHT: &str = "integral works to the right";
pub const NO_ZEROS: &str = "no zeros anywhere";
pub const CHAPTER_14: &str = "how: chapter 14";
pub const CONVERGES: &str = "series converges: Re s > 1";
pub const ONE_POLE: &str = "a single pole: s = 1";
pub const TRIVIAL: &str = "zeros at ";
pub const CRITICAL: &str = "more zeros on Re s = 1/2";
pub const SQUASHED: &str = "strip squashed: 0 ≤ Im s ≤ ";
pub const RH: [&str; 2] = ["Riemann hypothesis:", "open"];
pub const FOUR: &str = "this plot shows four zeros, located by Newton's method";
pub const ONE_TILE: &str = "one tile of z³ − 1, traced exactly";
pub const NEXT: &str = "next: what one complex derivative demands";

/// The walker's output plane (05_1, 05_2), and the radius outputs are
/// squashed into.
pub fn walk_plane() -> Plane {
    Plane::new(v(3.4, -0.6), 3.4, (-1.3, 1.3), (-1.3, 1.3))
}
pub const OUT_R: f64 = 1.1;
/// Where the turn count reads.
pub const COUNT_AT: (f64, f64) = (3.4, 1.75);
/// The titles' place.
pub const TITLE_AT: (f64, f64) = (3.4, 2.85);

/// The polynomial and rational window (05_1, 05_2).
pub fn square_plane() -> Plane {
    Plane::new(v(-3.4, -0.2), 5.6, (-2.0, 2.0), (-2.0, 2.0))
}
/// The cubic's big loop and the walker's squash.
pub const CUBIC_LOOP: (f64, f64) = (1.8, 3.0);
/// The small loop around the double pole at −1 and the walker's squash.
pub const ZDP_LOOP: (f64, f64) = (0.3, 3.0);

/// e^z over x ∈ [−3, 3], y ∈ [−4, 8] (05_3).
pub fn exp_plane() -> Plane {
    Plane::new(v(-2.6, -0.25), 3.2, (-3.0, 3.0), (-4.0, 8.0))
}

/// sin and tan over x ∈ [−2π, 2π], y ∈ [−3.3, 3.3] (05_4, 05_5).
pub fn trig_plane() -> Plane {
    Plane::new(v(-2.4, -0.1), 8.4, (-TAU, TAU), (-3.3, 3.3))
}

/// Γ over x ∈ [−4.5, 4.5], y ∈ [−2.5, 2.5] (05_6).
pub fn gamma_plane() -> Plane {
    Plane::new(v(-2.4, -0.3), 8.4, (-4.5, 4.5), (-2.5, 2.5))
}

/// ζ over x ∈ [−7, 3], y ∈ [−3, 3] (05_7).
pub fn zeta_plane() -> Plane {
    Plane::new(v(-3.4, 0.2), 6.4, (-7.0, 3.0), (-3.0, 3.0))
}

/// The tall strip x ∈ [−1, 2], y ∈ [0, 32], squashed vertically (its units
/// are not square; the label says so).
pub fn strip_plane() -> Plane {
    Plane {
        frame: Frame2 {
            center: v(1.6, 0.35),
            width: 2.0,
            height: 5.9,
            x: (-1.0, 2.0),
            y: (0.0, model::ZETA_STRIP_TOP),
        },
    }
}

/// The right-hand column's x.
pub const COLUMN: f64 = 4.6;

/// The first four nontrivial zeros of ζ, each found by Newton's method from
/// its published height rounded to a whole number (as `facts` does).
pub fn zeta_zeros() -> Vec<C> {
    model::ZETA_ZERO_HEIGHTS
        .iter()
        .filter_map(|&t| model::zeta_zero_near(C::new(0.5, t.round())))
        .collect()
}

/// The mosaic's panels: 4.2 × 2.6 each, three across and two down.
pub const PANEL: (f64, f64) = (4.2, 2.6);

pub fn mosaic() -> [(Func, Plane); 6] {
    let panel = |cx: f64, cy: f64, x: (f64, f64), yc: f64| {
        let h = (x.1 - x.0) * PANEL.1 / PANEL.0;
        Plane::new(v(cx, cy), PANEL.0, x, (yc - h / 2.0, yc + h / 2.0))
    };
    [
        (Func::Cubic, panel(-4.5, 1.55, (-2.7, 2.7), 0.3)),
        (Func::ZeroDoublePole, panel(0.0, 1.55, (-2.7, 2.7), 0.0)),
        (Func::Exp, panel(4.5, 1.55, (-3.5, 3.5), 1.2)),
        (Func::Sin, panel(-4.5, -1.35, (-TAU, TAU), 0.0)),
        (Func::Gamma, panel(0.0, -1.35, (-4.5, 3.5), 0.0)),
        (Func::Zeta, panel(4.5, -1.35, (-7.0, 3.0), 0.0)),
    ]
}

/// Where the tile is sought: away from the cubic's zeros.
pub const TILE_SEED: C = C::new(0.9, 1.1);
/// Points traced along each side of the tile.
pub const TILE_STEPS: usize = 16;

/// z with z³ − 1 = e^w, by Newton's method from `guess`.
fn cubic_preimage(w: C, guess: C) -> C {
    let target = w.exp();
    let mut z = guess;
    for _ in 0..50 {
        let step = (z * z * z - ONE - target) / (z * z).scale(3.0);
        z = z - step;
        if step.abs() < 1e-15 {
            break;
        }
    }
    z
}

/// The exact outline of one tile of the enhanced portrait of z³ − 1: the
/// tile whose lower-left corner (in log size and angle) is the band corner
/// just below the seed's. Each side is a band line, traced point by point
/// by Newton's method from the straight-line guess, counterclockwise.
pub fn tile() -> Vec<C> {
    let s = model::band_step();
    let f0 = TILE_SEED * TILE_SEED * TILE_SEED - ONE;
    let g = (TILE_SEED * TILE_SEED).scale(3.0) / f0;
    let w0 = f0.ln();
    let corner = C::new(s * (w0.re / s).floor(), s * (w0.im / s).floor());
    let sides = [(ZERO, ONE), (ONE, ONE + I), (ONE + I, I), (I, ZERO)];
    let mut out = Vec::with_capacity(4 * TILE_STEPS + 1);
    for (a, b) in sides {
        for k in 0..TILE_STEPS {
            let c = a + (b - a).scale(k as f64 / TILE_STEPS as f64);
            let w = corner + c.scale(s);
            out.push(cubic_preimage(w, TILE_SEED + (w - w0) / g));
        }
    }
    out.push(out[0]);
    out
}

/// The tile's centre (the mean of its traced points).
pub fn tile_centre() -> C {
    let t = tile();
    let n = (t.len() - 1) as f64;
    t[..t.len() - 1].iter().fold(ZERO, |a, &z| a + z).scale(1.0 / n)
}

/// The zoom onto the tile (05_9).
pub fn zoom_plane() -> Plane {
    let c = tile_centre();
    Plane::new(
        v(2.6, -0.3),
        5.0,
        (c.re - 0.65, c.re + 0.65),
        (c.im - 0.65, c.im + 0.65),
    )
}

/// A dark cover over the part of `p` left of Re z = `x`.
fn cover_left(stage: &mut Stage<'_>, p: &Plane, x: f64) -> Mob {
    let (x0, y0, y1) = (p.frame.x.0, p.frame.y.0, p.frame.y.1);
    let pts = vec![
        p.p(C::new(x0, y0)),
        p.p(C::new(x, y0)),
        p.p(C::new(x, y1)),
        p.p(C::new(x0, y1)),
    ];
    stage
        .arena_mut()
        .add(Polygon::new(pts).style(flat(ground(), 0.88)).build())
}

/// The outline of the window `w` drawn inside the picture `p`.
fn window_box(stage: &mut Stage<'_>, p: &Plane, w: &Plane, color: Srgb) -> fmn::Result<Mob> {
    let (x, y) = (w.frame.x, w.frame.y);
    let zs = [
        C::new(x.0, y.0),
        C::new(x.1, y.0),
        C::new(x.1, y.1),
        C::new(x.0, y.1),
        C::new(x.0, y.0),
    ];
    p.path(stage, &zs, color, 4.0)
}

/// Part `k` of a comma-separated fact, as printed.
fn part(key: &str, k: usize) -> fmn::Result<&'static str> {
    fact(key)?
        .split(',')
        .nth(k)
        .ok_or_else(|| fail(format!("fact {key} has no part {k}")))
}

/// A printed number with a true minus sign.
fn minus(s: &str) -> String {
    s.replace('-', "−")
}

pub struct Gallery {
    pub kit: Kit,
}

impl Gallery {
    /// Walk the loop `|z − centre| = r` of `f` in `inp`, with its image in
    /// `out` and a live turn count, until `phrase` (or the line's end); then
    /// the count is replaced by `stated`. Returns everything it drew.
    #[allow(clippy::too_many_arguments)]
    fn walk_and_count(
        kit: &Kit,
        stage: &mut Stage<'_>,
        line: &Spoken,
        phrase: Option<&str>,
        f: Func,
        centre: C,
        (r, squash): (f64, f64),
        inp: Plane,
        stated: &str,
    ) -> fmn::Result<Vec<Mob>> {
        let out = walk_plane();
        let ring = inp.circle(stage, centre, r, hero(), 5.0)?;
        fade_in_all(stage, &[ring], 0.4)?;
        let w = walker(stage, f, centre, r, inp, out, squash, OUT_R)?;
        w.show(stage, 0.4)?;
        let at = v(COUNT_AT.0, COUNT_AT.1);
        let counter = w.counter(kit, stage, at)?;
        let rt = match phrase {
            Some(p) => line.until(stage, p, 2.5)?,
            None => (line.end() - stage.scene().time().to_f64()).max(2.5),
        };
        w.walk(stage, rt)?;
        stage.clear_updaters(counter, true);
        let shown = kit.text(stage, stated, 26.0, hero(), at)?;
        stage.remove_from_scene(counter);
        stage.add_to_scene(shown)?;
        stage.clear_updaters(w.anchor, true);
        let mut drawn = vec![ring, shown];
        drawn.extend(&w.mobs);
        Ok(drawn)
    }
}

impl SceneConstruct for Gallery {
    fn name(&self) -> &str {
        "05_gallery"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title_at = v(TITLE_AT.0, TITLE_AT.1);

        // 05_1: z³ − 1: three zeros on the unit circle; a big loop turns three times
        let line = say(&kit, stage, "05_1")?;
        let sp = square_plane();
        let painter = Painter::build(stage, Some(Func::Cubic), sp, 1.0)?;
        let border = sp.border(stage, Palette::faint())?;
        painter.show(stage, 1.0)?;
        let title = kit.tex(stage, TITLES_TEX[0], 34.0, Palette::ink(), &[], title_at)?;
        fade_in_all(stage, &[border, title], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let unit = sp.circle(stage, ZERO, 1.0, Palette::faint(), 2.0)?;
        let cubic_marks = marks(stage, Func::Cubic, &sp);
        fade_in_all(stage, &[unit], 0.3)?;
        fade_in_all(stage, &cubic_marks, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let out = walk_plane();
        let mut out_frame = vec![out.border(stage, Palette::faint())?];
        out_frame.extend(out.axes(stage)?);
        fade_in_all(stage, &out_frame, 0.3)?;
        let stated = format!("{TURNS}{}{CUBIC_SAYS}", fact("gallery.cubic.turns_r1_8")?);
        let walked = Self::walk_and_count(
            &kit,
            stage,
            &line,
            Some("Three zeros inside"),
            Func::Cubic,
            ZERO,
            CUBIC_LOOP,
            sp,
            &stated,
        )?;
        wait_until(stage, line.end())?;

        // 05_2: (z − 1)/(z + 1)²: a simple zero; a double pole turns backwards twice
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_2")?;
        let mut gone = vec![unit, title];
        gone.extend(cubic_marks);
        gone.extend(walked);
        fade_out_all(stage, &gone, 0.4)?;
        painter.to(stage, Some(Func::ZeroDoublePole), 1.2)?;
        let title = kit.tex(stage, TITLES_TEX[1], 34.0, Palette::ink(), &[], title_at)?;
        fade_in_all(stage, &[title], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let zdp_marks = marks(stage, Func::ZeroDoublePole, &sp);
        fade_in_all(stage, &zdp_marks, 0.4)?;
        let mut labels = carded_lines(
            &kit,
            stage,
            &[SIMPLE_ZERO],
            20.0,
            Palette::ink(),
            sp.p(C::new(1.0, -0.42)),
            Palette::faint(),
        )?;
        labels.extend(carded_lines(
            &kit,
            stage,
            &[POLE2],
            20.0,
            pole(),
            sp.p(C::new(-1.0, 0.62)),
            pole(),
        )?);
        fade_in_all(stage, &labels, 0.3)?;
        line.wait_for(stage, "where the colours run backwards")?;
        let stated = format!("{TURNS}{}{ZDP_SAYS}", minus(fact("gallery.zdp.turns_at_minus1")?));
        let walked = Self::walk_and_count(
            &kit,
            stage,
            &line,
            None,
            Func::ZeroDoublePole,
            -ONE,
            ZDP_LOOP,
            sp,
            &stated,
        )?;
        wait_until(stage, line.end())?;

        // 05_3: e^z: no zeros or poles; direction by height, size by how far right
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_3")?;
        let mut gone = vec![title, border];
        gone.extend(zdp_marks);
        gone.extend(labels);
        gone.extend(walked);
        gone.extend(out_frame);
        fade_out_all(stage, &gone, 0.4)?;
        painter.leave(stage, 0.5)?;
        let ep = exp_plane();
        let exp = Painter::build(stage, Some(Func::Exp), ep, 1.0)?;
        let eborder = ep.border(stage, Palette::faint())?;
        exp.show(stage, 0.8)?;
        let col = 3.6;
        let title = kit.tex(stage, TITLES_TEX[2], 34.0, Palette::ink(), &[], v(col, 2.6))?;
        let none = kit.text(stage, NO_ZEROS_POLES, 22.0, Palette::ink(), v(col, 1.8))?;
        fade_in_all(stage, &[eborder, title, none], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let up = Arrow::new(ep.p(C::new(-2.55, -3.5)), ep.p(C::new(-2.55, 7.5)))
            .thickness(4.0)
            .color(Palette::ink())
            .build()?;
        let up = stage.arena_mut().add(up);
        let height_says = kit.text(stage, BY_HEIGHT, 22.0, Palette::ink(), v(col, 1.0))?;
        fade_in_all(stage, &[up, height_says], 0.4)?;
        line.wait_for(stage, "and its size")?;
        let across = Arrow::new(ep.p(C::new(-2.55, -3.5)), ep.p(C::new(2.6, -3.5)))
            .thickness(4.0)
            .color(Palette::ink())
            .build()?;
        let across = stage.arena_mut().add(across);
        let right_says = kit.text(stage, BY_RIGHT, 22.0, Palette::ink(), v(col, 0.4))?;
        fade_in_all(stage, &[across, right_says], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let (b0, b1) = (ep.p(C::new(3.0, 0.0)), ep.p(C::new(3.0, TAU)));
        let bracket = stage.arena_mut().add(
            polyline(vec![
                v(b0[0] + 0.08, b0[1]),
                v(b0[0] + 0.25, b0[1]),
                v(b1[0] + 0.25, b1[1]),
                v(b1[0] + 0.08, b1[1]),
            ])
            .style(line_style(hero(), 4.0))
            .build(),
        );
        let period = kit.tex(
            stage,
            &format!("{PERIOD_TEX}{}", fact("gallery.exp.period")?),
            30.0,
            hero(),
            &[],
            v(b0[0] + 1.45, (b0[1] + b1[1]) / 2.0),
        )?;
        fade_in_all(stage, &[bracket, period], 0.5)?;
        wait_until(stage, line.end())?;

        // 05_4: sin z: zeros at kπ only; bounded on ℝ, fast growth off it
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_4")?;
        fade_out_all(
            stage,
            &[
                eborder,
                title,
                none,
                up,
                height_says,
                across,
                right_says,
                bracket,
                period,
            ],
            0.4,
        )?;
        exp.leave(stage, 0.5)?;
        let tp = trig_plane();
        let trig = Painter::build(stage, Some(Func::Sin), tp, 1.0)?;
        let tborder = tp.border(stage, Palette::faint())?;
        trig.show(stage, 0.8)?;
        let title = kit.tex(stage, TITLES_TEX[3], 34.0, Palette::ink(), &[], v(COLUMN, 2.6))?;
        fade_in_all(stage, &[tborder, title], 0.4)?;
        let sin_marks = marks(stage, Func::Sin, &tp);
        let zeros_say = kit.text(stage, SIN_ZEROS, 20.0, Palette::ink(), v(COLUMN, 1.8))?;
        fade_in_all(stage, &sin_marks, 0.4)?;
        fade_in_all(stage, &[zeros_say], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let real = tp.path(stage, &[C::real(-TAU), C::real(TAU)], hero(), 5.0)?;
        let real_says = kit.text(stage, SIN_REAL, 20.0, hero(), v(COLUMN, 1.15))?;
        fade_in_all(stage, &[real, real_says], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let climb = Arrow::new(tp.p(ZERO), tp.p(C::new(0.0, 3.0)))
            .thickness(4.0)
            .color(Palette::ink())
            .build()?;
        let climb = stage.arena_mut().add(climb);
        fade_in_all(stage, &[climb], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let mut readouts = Vec::new();
        for (k, (z, key)) in [(I, "gallery.sin.at_i"), (C::new(0.0, 3.0), "gallery.sin.at_3i")]
            .into_iter()
            .enumerate()
        {
            let at = tp.p(z);
            let d = dot(stage, at, 0.08, Palette::ink());
            let card = explainer_kit::props::equation_card(
                &kit,
                stage,
                &format!(r"{}{}\,i", SIN_AT_TEX[k], part(key, 1)?),
                26.0,
                &[],
                v(at[0] + 1.9, at[1]),
            )?;
            let left = at[0] + 1.9 - stage.get_width(card) / 2.0;
            let arrow = Arrow::new(v(left - 0.05, at[1]), v(at[0] + 0.14, at[1]))
                .thickness(3.0)
                .color(Palette::ink())
                .build()?;
            let arrow = stage.arena_mut().add(arrow);
            fade_in_all(stage, &[d, card, arrow], 0.4)?;
            readouts.extend([d, card, arrow]);
            if k == 0 {
                line.wait_for(stage, "and sine of three i")?;
            }
        }
        wait_until(stage, line.end())?;

        // 05_5: tan z on the same window: alternating zeros and poles; ±i far off
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_5")?;
        let mut gone = vec![title, zeros_say, real, real_says, climb];
        gone.extend(sin_marks);
        gone.extend(readouts);
        fade_out_all(stage, &gone, 0.4)?;
        trig.to(stage, Some(Func::Tan), 1.2)?;
        let title = kit.tex(stage, TITLES_TEX[4], 32.0, Palette::ink(), &[], v(COLUMN, 2.6))?;
        fade_in_all(stage, &[title], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let tan_marks = marks(stage, Func::Tan, &tp);
        let alternate = kit.text(stage, ALTERNATE, 20.0, Palette::ink(), v(COLUMN, 1.7))?;
        fade_in_all(stage, &tan_marks, 0.5)?;
        fade_in_all(stage, &[alternate], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let up_card = explainer_kit::props::equation_card(
            &kit,
            stage,
            TAN_LIMIT_TEX[0],
            26.0,
            &[],
            tp.p(C::new(std::f64::consts::PI, 2.4)),
        )?;
        let far_up = kit.tex(
            stage,
            &format!(r"{}{}\,i", TAN_FAR_TEX[0], part("gallery.tan.far", 1)?),
            24.0,
            Palette::ink(),
            &[],
            v(COLUMN, 0.9),
        )?;
        fade_in_all(stage, &[up_card, far_up], 0.4)?;
        line.wait_for(stage, "and far below")?;
        let down_card = explainer_kit::props::equation_card(
            &kit,
            stage,
            TAN_LIMIT_TEX[1],
            26.0,
            &[],
            tp.p(C::new(std::f64::consts::PI, -2.4)),
        )?;
        let far_down = kit.tex(
            stage,
            &format!(r"{}{}\,i", TAN_FAR_TEX[1], part("gallery.tan.far", 3)?),
            24.0,
            Palette::ink(),
            &[],
            v(COLUMN, 0.1),
        )?;
        fade_in_all(stage, &[down_card, far_down], 0.4)?;
        wait_until(stage, line.end())?;

        // 05_6: Γ: the integral's half-plane, then the poles as they are named
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_6")?;
        let mut gone = vec![tborder, title, alternate, up_card, far_up, down_card, far_down];
        gone.extend(tan_marks);
        fade_out_all(stage, &gone, 0.4)?;
        trig.leave(stage, 0.5)?;
        let gp = gamma_plane();
        let gamma = Painter::build(stage, Some(Func::Gamma), gp, 1.0)?;
        let gborder = gp.border(stage, Palette::faint())?;
        gamma.show(stage, 0.8)?;
        let title = kit.tex(stage, TITLES_TEX[5], 34.0, Palette::ink(), &[], v(COLUMN, 2.6))?;
        fade_in_all(stage, &[gborder, title], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let factorial = kit.tex(stage, FACTORIAL_TEX, 30.0, hero(), &[], v(COLUMN, 1.75))?;
        let at4 = kit.tex(
            stage,
            &format!("{}{}", GAMMA_AT_TEX[0], fact("gallery.gamma.at4")?),
            26.0,
            Palette::ink(),
            &[],
            v(COLUMN, 0.95),
        )?;
        let at5 = kit.tex(
            stage,
            &format!("{}{}", GAMMA_AT_TEX[1], fact("gallery.gamma.at5")?),
            26.0,
            Palette::ink(),
            &[],
            v(COLUMN, 0.3),
        )?;
        fade_in_all(stage, &[factorial], 0.4)?;
        fade_in_all(stage, &[at4, at5], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let cover = cover_left(stage, &gp, 0.0);
        let edge = dashed(stage, gp.p(C::new(0.0, -2.5)), gp.p(C::new(0.0, 2.5)), Palette::ink())?;
        let mut right_card = carded_lines(
            &kit,
            stage,
            &[INTEGRAL_RIGHT],
            20.0,
            Palette::ink(),
            gp.p(C::new(2.25, 2.0)),
            Palette::faint(),
        )?;
        fade_in_all(stage, &[cover, edge], 0.5)?;
        fade_in_all(stage, &right_card, 0.3)?;
        line.wait_sentence(stage, 3)?;
        fade_out_all(stage, &[cover], 0.8)?;
        let mut gamma_poles = Vec::new();
        let named = ["a pole at zero", "at minus one", "at minus two"];
        for (k, &p0) in fact_list("gallery.gamma.poles")?.iter().enumerate() {
            if let Some(phrase) = named.get(k) {
                line.wait_for(stage, phrase)?;
            }
            let pm = pole_mark(stage, gp.p(C::real(p0)));
            fade_in_all(stage, &pm, 0.25)?;
            gamma_poles.extend(pm);
        }
        line.wait_for(stage, "and no zeros anywhere")?;
        let no_zeros = kit.text(stage, NO_ZEROS, 22.0, Palette::ink(), v(COLUMN, -0.6))?;
        fade_in_all(stage, &[no_zeros], 0.4)?;
        line.wait_sentence(stage, 4)?;
        let later = kit.text(stage, CHAPTER_14, 22.0, Palette::dim(), v(COLUMN, -1.3))?;
        fade_in_all(stage, &[later], 0.4)?;
        wait_until(stage, line.end())?;

        // 05_7: ζ: the series' half-plane; a pole at 1; zeros at −2, −4, …; the strip
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_7")?;
        let mut gone = vec![gborder, title, factorial, at4, at5, edge, no_zeros, later];
        gone.append(&mut right_card);
        gone.extend(gamma_poles);
        fade_out_all(stage, &gone, 0.4)?;
        gamma.leave(stage, 0.5)?;
        let zp = zeta_plane();
        let zeta = Painter::build(stage, Some(Func::Zeta), zp, 1.0)?;
        let zborder = zp.border(stage, Palette::faint())?;
        let series = kit.tex(stage, TITLES_TEX[6], 30.0, Palette::ink(), &[], v(-3.4, 2.85))?;
        fade_in_all(stage, &[series], 0.5)?;
        line.wait_sentence(stage, 1)?;
        let zcover = cover_left(stage, &zp, 1.0);
        zeta.show(stage, 0.8)?;
        stage.add_to_scene(zcover)?;
        let zedge = dashed(stage, zp.p(C::new(1.0, -3.0)), zp.p(C::new(1.0, 3.0)), Palette::ink())?;
        // the column sits right of the strip's height labels
        let zcol = COLUMN + 0.4;
        let converges = kit.text(stage, CONVERGES, 22.0, Palette::ink(), v(zcol, 2.5))?;
        fade_in_all(stage, &[zborder, zedge, converges], 0.4)?;
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &[zcover], 0.8)?;
        line.wait_sentence(stage, 3)?;
        let pole1 = pole_mark(stage, zp.p(ONE));
        let pole_says = kit.text(stage, ONE_POLE, 22.0, pole(), v(zcol, 1.85))?;
        fade_in_all(stage, &pole1, 0.3)?;
        fade_in_all(stage, &[pole_says], 0.3)?;
        line.wait_sentence(stage, 4)?;
        let mut trivial = Vec::new();
        for &x in &fact_list("gallery.zeta.trivial")? {
            trivial.extend(zero_mark(stage, zp.p(C::real(x))));
        }
        let trivial_says = kit.text(
            stage,
            &format!(
                "{TRIVIAL}{}, …",
                minus(fact("gallery.zeta.trivial")?).replace(',', ", ")
            ),
            22.0,
            Palette::ink(),
            v(zcol, 1.2),
        )?;
        fade_in_all(stage, &trivial, 0.4)?;
        fade_in_all(stage, &[trivial_says], 0.3)?;
        line.wait_sentence(stage, 5)?;
        // the strip starts where the wide picture's [−1, 2] × [0, 3] is
        let st = strip_plane();
        let start_box = zp.path(
            stage,
            &[
                C::new(-1.0, 0.0),
                C::new(2.0, 0.0),
                C::new(2.0, 3.0),
                C::new(-1.0, 3.0),
                C::new(-1.0, 0.0),
            ],
            hero(),
            3.0,
        )?;
        let link = stage.arena_mut().add(
            polyline(vec![zp.p(C::new(2.0, 3.0)), st.p(C::new(-1.0, 0.0))])
                .style(line_style(hero(), 2.0))
                .build(),
        );
        let strip = Painter::build(stage, Some(Func::Zeta), st, 1.0)?;
        let sborder = st.border(stage, hero())?;
        strip.show(stage, 0.8)?;
        let half = st.path(
            stage,
            &[C::new(0.5, 0.0), C::new(0.5, model::ZETA_STRIP_TOP)],
            Palette::ink(),
            2.0,
        )?;
        let squashed = kit.text(
            stage,
            &format!("{SQUASHED}{}", fact("gallery.zeta.strip_top")?),
            18.0,
            Palette::dim(),
            v(st.frame.center[0], st.frame.bottom() - 0.25),
        )?;
        let critical = kit.text(stage, CRITICAL, 22.0, Palette::ink(), v(zcol, 0.5))?;
        fade_in_all(stage, &[start_box, link, sborder, half, squashed, critical], 0.4)?;
        let heights: Vec<&str> = fact("gallery.zeta.zeros")?.split(',').collect();
        let mut climbing = Vec::new();
        for (z, h) in zeta_zeros().into_iter().zip(heights) {
            let zm = zero_mark(stage, st.p(z));
            let label = kit.text(
                stage,
                &format!("{h}i"),
                20.0,
                Palette::ink(),
                v(st.frame.right() + 0.5, st.p(z)[1]),
            )?;
            fade_in_all(stage, &zm, 0.25)?;
            fade_in_all(stage, &[label], 0.2)?;
            climbing.extend(zm);
            climbing.push(label);
        }
        wait_until(stage, line.end())?;

        // 05_8: none found off the line; the Riemann hypothesis is open
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_8")?;
        let rings: Vec<Mob> = zeta_zeros()
            .into_iter()
            .map(|z| {
                let ring = Circle::new()
                    .radius(0.2)
                    .style(line_style(hero(), 5.0))
                    .build()
                    .moved_to(st.p(z));
                stage.arena_mut().add(ring)
            })
            .collect();
        fade_in_all(stage, &rings, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let rh = carded_lines(&kit, stage, &RH, 26.0, hero(), v(zcol, -1.0), hero())?;
        fade_in_all(stage, &rh, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let four = precisely_now(&kit, stage, FOUR)?;
        wait_until(stage, line.end())?;

        // 05_9: the six side by side; one tile of z³ − 1 traced exactly
        kit.hold(stage)?;
        let line = say(&kit, stage, "05_9")?;
        let mut gone = vec![
            zborder,
            series,
            zedge,
            converges,
            pole_says,
            trivial_says,
            start_box,
            link,
            sborder,
            half,
            squashed,
            critical,
            four,
        ];
        gone.extend(pole1);
        gone.extend(trivial);
        gone.extend(climbing);
        gone.extend(rings);
        gone.extend(rh);
        fade_out_all(stage, &gone, 0.4)?;
        zeta.leave(stage, 0.4)?;
        strip.leave(stage, 0.4)?;
        let panels = mosaic();
        let mut painters = Vec::with_capacity(panels.len());
        let mut frames = Vec::new();
        for &(f, p) in &panels {
            let painter = Painter::build(stage, Some(f), p, 1.0)?;
            painter.show(stage, 0.25)?;
            frames.push(p.border(stage, Palette::faint())?);
            painters.push(painter);
        }
        fade_in_all(stage, &frames, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let panel_marks: Vec<Vec<Mob>> = panels.iter().map(|&(f, p)| marks(stage, f, &p)).collect();
        fade_in_all(stage, &panel_marks.concat(), 0.5)?;
        line.wait_sentence(stage, 2)?;
        let (_, cubic) = panels[0];
        let zp9 = zoom_plane();
        let boxed = window_box(stage, &cubic, &zp9, hero())?;
        fade_in_all(stage, &[boxed], 0.4)?;
        // keep the cubic's panel; the others make room for the zoom
        let mut leaving: Vec<Mob> = frames[1..].to_vec();
        leaving.extend(panel_marks[1..].concat());
        fade_out_all(stage, &leaving, 0.4)?;
        for p in &painters[1..] {
            p.leave(stage, 0.4)?;
        }
        let zoom = Painter::build(stage, Some(Func::Cubic), zp9, 1.0)?;
        let zborder9 = zp9.border(stage, hero())?;
        let corner = |p: &Plane, x: usize, y: usize| {
            let xs = [p.frame.x.0, p.frame.x.1];
            let ys = [p.frame.y.0, p.frame.y.1];
            C::new(xs[x], ys[y])
        };
        let links: Vec<Mob> = [(1, 1), (1, 0)]
            .into_iter()
            .map(|(x, y)| {
                let a = cubic.p(corner(&zp9, x, y));
                let b = zp9.p(corner(&zp9, 0, y));
                stage
                    .arena_mut()
                    .add(polyline(vec![a, b]).style(line_style(hero(), 2.0)).build())
            })
            .collect();
        zoom.show(stage, 0.8)?;
        fade_in_all(stage, &[zborder9], 0.3)?;
        fade_in_all(stage, &links, 0.3)?;
        let outline = zp9.path(stage, &tile(), hero(), 6.0)?;
        let tile_says = kit.text(stage, ONE_TILE, 20.0, hero(), v(-4.5, -0.6))?;
        fade_in_all(stage, &[outline, tile_says], 0.5)?;
        line.wait_sentence(stage, 4)?;
        let next = kit.text(stage, NEXT, 20.0, Palette::ink(), v(-4.5, -1.4))?;
        fade_in_all(stage, &[next], 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![frames[0], boxed, zborder9, outline, tile_says, next];
        all.extend(panel_marks[0].iter().copied());
        all.extend(links);
        fade_out_all(stage, &all, 0.5)?;
        painters[0].leave(stage, 0.5)?;
        zoom.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::draw::Walker;

    fn get(key: &str) -> &'static str {
        fact(key).unwrap()
    }

    #[test]
    fn the_walks_count_what_the_facts_say() {
        let (r, _) = CUBIC_LOOP;
        let cubic = Walker::turns_so_far(Func::Cubic, ZERO, r, 1.0);
        assert!((cubic - get("gallery.cubic.turns_r1_8").parse::<f64>().unwrap()).abs() < 1e-9);
        let (r, _) = ZDP_LOOP;
        let zdp = Walker::turns_so_far(Func::ZeroDoublePole, -ONE, r, 1.0);
        assert!((zdp - get("gallery.zdp.turns_at_minus1").parse::<f64>().unwrap()).abs() < 1e-9);
        // both loops lie in the picture, and the double pole's loop holds nothing else
        let sp = square_plane();
        for z in model::circle(ZERO, CUBIC_LOOP.0, 64)
            .into_iter()
            .chain(model::circle(-ONE, ZDP_LOOP.0, 64))
        {
            assert!(sp.contains(z));
        }
        assert!((ONE - (-ONE)).abs() > ZDP_LOOP.0);
        // the cubic's three zeros are inside its loop
        assert!(
            Func::Cubic
                .zeros((-2.0, 2.0), (-2.0, 2.0))
                .iter()
                .all(|z| z.abs() < CUBIC_LOOP.0)
        );
    }

    #[test]
    fn the_readouts_are_the_facts() {
        assert_eq!(get("gallery.sin.at_i").split(',').nth(1), Some("1.18"));
        assert_eq!(get("gallery.sin.at_3i").split(',').nth(1), Some("10.0"));
        let far: Vec<&str> = get("gallery.tan.far").split(',').collect();
        assert_eq!((far[1], far[3]), ("1.0000", "-1.0000"));
        // the cards type the point: it has to be the point the facts read
        let p = model::TAN_FAR;
        assert!(TAN_FAR_TEX[0].contains(&format!("{} + {}i", p.re, p.im)));
        assert!(TAN_FAR_TEX[1].contains(&format!("{} - {}i", p.re, p.im)));
        // the strip's stated top is above every zero marked on it
        assert_eq!(get("gallery.zeta.strip_top"), "32");
        assert!(model::ZETA_ZERO_HEIGHTS.iter().all(|&t| t < model::ZETA_STRIP_TOP));
        assert_eq!(get("gallery.exp.period"), "6.28");
        assert_eq!((get("gallery.gamma.at4"), get("gallery.gamma.at5")), ("6", "24"));
        // sin and tan on the trig window: five zeros of each, four poles of tan
        let tp = trig_plane();
        let (x, y) = (tp.frame.x, tp.frame.y);
        assert_eq!(Func::Sin.zeros(x, y).len(), 5);
        assert_eq!(Func::Tan.zeros(x, y).len(), 5);
        assert_eq!(Func::Tan.poles(x, y).len(), 4);
        assert!(tp.contains(C::new(0.0, 3.0)));
        // Γ's window holds the five named poles
        let gp = gamma_plane();
        let poles: Vec<f64> = get("gallery.gamma.poles")
            .split(',')
            .map(|p| p.parse().unwrap())
            .collect();
        assert_eq!(poles.len(), 5);
        assert!(poles.iter().all(|&p| gp.contains(C::real(p))));
    }

    #[test]
    fn the_strip_zeros_are_newtons_and_match_the_facts() {
        let zs = zeta_zeros();
        assert_eq!(zs.len(), 4);
        let st = strip_plane();
        let printed: Vec<String> = zs.iter().map(|z| format!("{:.2}", z.im)).collect();
        assert_eq!(printed.join(","), get("gallery.zeta.zeros"));
        for z in &zs {
            assert!((z.re - 0.5).abs() < 1e-9 && st.contains(*z));
            assert!(model::zeta(*z).unwrap().abs() < 1e-9);
        }
        // the wide picture holds the pole and the three trivial zeros it marks
        let zp = zeta_plane();
        assert!(zp.contains(ONE));
        for x in get("gallery.zeta.trivial").split(',') {
            assert!(zp.contains(C::real(x.parse().unwrap())));
        }
        // the strip stays above the lower third and inside the frame
        assert!(st.frame.bottom() > -2.78 && st.frame.top() < 3.85);
    }

    #[test]
    fn the_tile_is_traced_exactly_and_is_nearly_square() {
        let s = model::band_step();
        let t = tile();
        assert_eq!(t.len(), 4 * TILE_STEPS + 1);
        for &z in &t {
            let w = (z * z * z - ONE).ln();
            let on = |x: f64| (x / s - (x / s).round()).abs() < 1e-9;
            assert!(on(w.re) || on(w.im), "{z:?}");
        }
        let corners: Vec<C> = (0..4).map(|k| t[k * TILE_STEPS]).collect();
        let sides: Vec<f64> = (0..4).map(|k| (corners[(k + 1) % 4] - corners[k]).abs()).collect();
        let (lo, hi) = sides
            .iter()
            .fold((f64::MAX, 0.0_f64), |(a, b), &x| (a.min(x), b.max(x)));
        assert!(hi / lo < 1.1, "{sides:?}");
        // the tile sits in the zoom, the zoom in the cubic's mosaic panel, with no zero in the zoom
        let zp = zoom_plane();
        assert!(t.iter().all(|&z| zp.contains(z)));
        let (_, cubic) = mosaic()[0];
        let (x, y) = (zp.frame.x, zp.frame.y);
        for z in [C::new(x.0, y.0), C::new(x.1, y.1)] {
            assert!(cubic.contains(z));
        }
        assert!(Func::Cubic.zeros(x, y).is_empty());
    }

    #[test]
    fn the_mosaic_fits_the_frame() {
        let panels = mosaic();
        for (k, (_, p)) in panels.iter().enumerate() {
            assert!((p.frame.height - PANEL.1).abs() < 1e-9, "{k}");
            assert!(p.frame.left() > -6.96 && p.frame.right() < 6.96);
            assert!(p.frame.bottom() > -3.85 && p.frame.top() < 3.85);
            for (_, q) in &panels[k + 1..] {
                let apart = p.frame.right() <= q.frame.left()
                    || q.frame.right() <= p.frame.left()
                    || p.frame.top() <= q.frame.bottom()
                    || q.frame.top() <= p.frame.bottom();
                assert!(apart);
            }
        }
        // the zoom stays clear of the cubic's panel
        let (_, cubic) = panels[0];
        assert!(zoom_plane().frame.left() > cubic.frame.right());
    }
}
