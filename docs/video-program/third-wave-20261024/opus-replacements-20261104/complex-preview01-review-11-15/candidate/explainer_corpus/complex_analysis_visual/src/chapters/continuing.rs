//! 14_continuation: what analytic continuation can and cannot do
//! (storyboard 14_1–14_8).
//!
//! - The bump's picture outside its recipe's disk.
//! - The geometric series against 1/(1 − z): partial sums settle at 1/2
//!   and grow at 1.2, where the function still has a value.
//! - Uniqueness on a connected region, then continuation along a path, as
//!   a chain of disks around the pole at 1.
//! - The divergent series 1 + 2 + 4 + … against the function's value −1,
//!   and ζ(−1).
//! - The square root carried once around 0 coming back as −1.
//! - A natural boundary.
//! - Γ revealed strip by strip from Γ(z) = Γ(z + 1)/z.

use std::f64::consts::TAU;

use explainer_kit::{Frame2, Kit, Palette, v};
use fmn::prelude::*;

use super::boundary::COLON;
use super::draw::{
    Painter, Plane, dot, fact, fact_f64, fact_list, fade_in_all, fade_out_all, faded_line, flat, ground, group, hero,
    input, lay_down, line_style, marks, names_now, output, pole, pole_mark, precisely_now, predict, reading_window,
    say, tween_linear, value, wait_until, warn,
};
use crate::model::{self, C, Func, ONE, ZERO};

pub const RECIPE_HERE: &str = "the recipe at 0 works here";
pub const EVERYWHERE: &str = "the bump: defined everywhere but ±i";
pub const GEOMETRIC_TEX: &str = r"1 + z + z^2 + z^3 + \cdots = \frac{1}{1-z} \qquad (|z| < 1)";
pub const SERIES_SAYS: [&str; 2] = ["z = ", " terms give "];
pub const VALUE_TEX: &str = r"\frac{1}{1 - 1.2} = ";
pub const AT_MOST_ONE: &str = "at most one extension to a connected region";
pub const REGION: &str = "the region here: every z except 1";
pub const STEP: &str = "step by step: re-centre inside the disk";
pub const FORCED: &str = "each step along the path is forced";
pub const NOT_ALWAYS: &str = "here every path gives 1/(1 − z); not always";
pub const PATH_EXACT: &str =
    "uniqueness is for one connected region; continuation along a path is unique for that path";
pub const ASK: [&str; 1] = ["1 + 2 + 4 + 8 + … = ?"];
pub const DOUBLING_TEX: &str = r"1 + 2 + 4 + 8 + \cdots";
pub const DIVERGES: &str = "the series diverges";
pub const AT_TWO_TEX: &str = r"\frac{1}{1-2} = ";
pub const EXTENDS: &str = "continuation extends the function, not the sum";
pub const NATURAL_SUM_TEX: &str = r"1 + 2 + 3 + 4 + \cdots";
pub const ZETA_TEX: &str = r"\zeta(-1) = -\frac{1}{12} \approx ";
pub const SQRT_SAYS: &str = "√ after one loop: ";
pub const LOG_SAYS: &str = "log gains 2πi each loop";
pub const LACUNARY_TEX: &str = r"z + z^2 + z^4 + z^8 + \cdots";
pub const DRAWN: [&str; 2] = ["drawn: ", " terms, inside |z| < "];
pub const NATURAL: &str = "natural boundary";
/// 14_7, on "a dense set of points": where the rim ticks are, and how far they are drawn.
pub const DENSE: [&str; 2] = [
    "it blows up at the 2nd, 4th, 8th, 16th, … roots of 1",
    "drawn: up to the 64th; all of them are dense on the circle",
];
pub const GAMMA_INT_TEX: &str = r"\Gamma(z) = \int_0^\infty t^{z-1} e^{-t}\,dt";
pub const RIGHT_ONLY: &str = "(for Re z > 0)";
pub const GAMMA_STEP_TEX: &str = r"\Gamma(z) = \frac{\Gamma(z+1)}{z}";
pub const NEW_POLES: &str = "a new pole at each step: 0, −1, −2, …";

/// The bump's picture (14_1).
pub fn bump_plane() -> Plane {
    Plane::new(v(-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5))
}

/// The continuation plane of 14_3: the pole at 1 and the chain around it.
pub fn chain_plane() -> Plane {
    Plane::new(v(-3.2, -0.2), 5.4, (-2.2, 3.2), (-2.4, 2.4))
}

/// The two planes of 14_6: z on the unit circle, and √z carried along.
pub fn sqrt_planes() -> (Plane, Plane) {
    (
        Plane::new(v(-4.3, -0.4), 4.2, (-1.4, 1.4), (-1.4, 1.4)),
        Plane::new(v(0.6, -0.4), 4.2, (-1.4, 1.4), (-1.4, 1.4)),
    )
}

/// The natural-boundary picture (14_7).
pub fn lacunary_plane() -> Plane {
    Plane::new(v(-3.0, -0.35), 5.4, (-1.05, 1.05), (-1.05, 1.05))
}

/// Γ's picture (14_8).
pub fn gamma_plane() -> Plane {
    Plane::new(v(-2.6, -0.5), 7.5, (-4.8, 3.0), (-2.0, 2.0))
}

/// 14_5's chart top: the whole climb to 2047 fits, so no bar is clipped.
pub const DOUBLING_TOP: f64 = 2200.0;
/// The rim ticks of 14_7 run from the circle out to here, levels 0 to `RIM_LEVELS`.
const RIM_TICK: f64 = 1.05;
const RIM_LEVELS: u32 = 6;

/// The angles of the roots of 1 new at level `k`: of order 2^k, not of any lower order. At
/// each one the series' terms are all 1 from the k-th on, so its sum blows up on the way out.
pub fn rim_level(k: u32) -> Vec<f64> {
    if k == 0 {
        return vec![0.0];
    }
    let n = 1_u32 << k;
    (1..n).step_by(2).map(|j| TAU * f64::from(j) / f64::from(n)).collect()
}

/// The bar charts of 14_2 and 14_5: partial sums n = 0 … 10.
pub fn bars_frame(center_x: f64, top: f64) -> Frame2 {
    Frame2 {
        center: v(center_x, -0.9),
        width: 3.4,
        height: 3.6,
        x: (-0.5, f64::from(model::SERIES_TOP) + 0.5),
        y: (0.0, top),
    }
}

/// Where the ray at angle `a` from 0 leaves the square window `[-h, h]²`.
pub fn square_exit(h: f64, a: f64) -> f64 {
    h / a.cos().abs().max(a.sin().abs())
}

/// Dark quads covering the window of `p` (a square centred on 0) outside
/// `|z| = r_in`: the region beyond the recipe's disk in 14_1. The corners
/// fall on sector edges, so each quad's outer side lies on the window edge.
fn outside_mask(stage: &mut Stage<'_>, p: &Plane, r_in: f64) -> fmn::Result<Mob> {
    let h = p.frame.x.1;
    let n = 64;
    let mut quads = Vec::with_capacity(n);
    for k in 0..n {
        let (a, b) = (TAU * k as f64 / n as f64, TAU * (k + 1) as f64 / n as f64);
        let pts = vec![
            p.p(C::polar(r_in, a)),
            p.p(C::polar(square_exit(h, a), a)),
            p.p(C::polar(square_exit(h, b), b)),
            p.p(C::polar(r_in, b)),
        ];
        quads.push(
            stage
                .arena_mut()
                .add(Polygon::new(pts).style(flat(ground(), 0.93)).build()),
        );
    }
    group(stage, &quads)
}

/// Partial sums 1 + z + … + zⁿ for n = 0 … `SERIES_TOP`.
pub fn partial_sums(z: f64) -> Vec<f64> {
    (0..=model::SERIES_TOP)
        .map(|n| model::geometric_partial(z, n))
        .collect()
}

/// A chart's caption in 14_2: "z = 1/2: 11 terms give 1.999023".
fn series_says(at: &str, sum: &str) -> fmn::Result<String> {
    let terms = fact("series.terms")?;
    Ok(format!("{}{at}{COLON}{terms}{}{sum}", SERIES_SAYS[0], SERIES_SAYS[1]))
}

/// Bars of `values` in `g`, one per n, in `color`, clipped at the top.
fn bars(stage: &mut Stage<'_>, g: &Frame2, values: &[f64], color: Srgb) -> Vec<Mob> {
    values
        .iter()
        .enumerate()
        .map(|(n, &h)| {
            let x = n as f64;
            let top = h.min(g.y.1);
            let pts = vec![
                g.p(x - 0.35, 0.0),
                g.p(x + 0.35, 0.0),
                g.p(x + 0.35, top),
                g.p(x - 0.35, top),
            ];
            stage
                .arena_mut()
                .add(Polygon::new(pts).style(flat(color, 0.85)).build())
        })
        .collect()
}

/// The base line of a bar chart.
fn base(stage: &mut Stage<'_>, g: &Frame2) -> Mob {
    stage.arena_mut().add(
        polyline(vec![g.p(g.x.0, 0.0), g.p(g.x.1, 0.0)])
            .style(faded_line(Palette::dim(), 2.0, 0.9))
            .build(),
    )
}

/// A dashed level line across `g` at height `y`.
fn level(stage: &mut Stage<'_>, g: &Frame2, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let n = 14;
    let step = (g.x.1 - g.x.0) / f64::from(n);
    let mut dashes = Vec::new();
    for k in 0..n {
        let x0 = g.x.0 + step * f64::from(k);
        dashes.push(
            stage.arena_mut().add(
                polyline(vec![g.p(x0, y), g.p(x0 + step * 0.55, y)])
                    .style(line_style(color, 3.0))
                    .build(),
            ),
        );
    }
    group(stage, &dashes)
}

/// A filled, outlined disk in `p`.
fn disk(stage: &mut Stage<'_>, p: &Plane, c: C, r: f64, color: Srgb, fill: f64) -> fmn::Result<Mob> {
    let pts: Vec<Vec3> = model::circle(c, r, 96).into_iter().map(|z| p.p(z)).collect();
    let f = stage
        .arena_mut()
        .add(Polygon::new(pts.clone()).style(flat(color, fill)).build());
    let mut closed = pts;
    closed.push(closed[0]);
    let rim = stage
        .arena_mut()
        .add(polyline(closed).style(line_style(color, 3.0)).build());
    group(stage, &[f, rim])
}

/// The closed outline covering Re z < `x` in `p` (Γ not yet continued
/// there).
pub fn strip_mask_points(p: &Plane, x: f64) -> Vec<Vec3> {
    let x = x.clamp(p.frame.x.0, p.frame.x.1);
    vec![
        p.p(C::new(p.frame.x.0, p.frame.y.0)),
        p.p(C::new(x, p.frame.y.0)),
        p.p(C::new(x, p.frame.y.1)),
        p.p(C::new(p.frame.x.0, p.frame.y.1)),
        p.p(C::new(p.frame.x.0, p.frame.y.0)),
    ]
}

pub struct Continuing {
    pub kit: Kit,
}

impl SceneConstruct for Continuing {
    fn name(&self) -> &str {
        "14_continuation"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let right = 4.1;

        // 14_1: the recipe stops at radius 1; the bump does not
        let line = say(&kit, stage, "14_1")?;
        let bp = bump_plane();
        let painter = Painter::build(stage, Some(Func::Bump), bp, 1.0)?;
        let border = bp.border(stage, Palette::faint())?;
        painter.show(stage, 1.0)?;
        let disk1 = bp.circle(stage, ZERO, fact_f64("bump.radius0")?, hero(), 5.0)?;
        fade_in_all(stage, &[border, disk1], 0.4)?;
        let here = kit.text(stage, RECIPE_HERE, 22.0, hero(), v(right, 2.2))?;
        fade_in_all(stage, &[here], 0.4)?;
        line.wait_sentence(stage, 1)?;
        // "stuck inside that circle?": only the recipe's disk shows
        let mask = outside_mask(stage, &bp, 1.03)?;
        let mask_edge = bp.border(stage, Palette::faint())?;
        fade_in_all(stage, &[mask, mask_edge], 0.6)?;
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &[mask, mask_edge], 1.0)?;
        line.wait_sentence(stage, 3)?;
        let bump_marks = marks(stage, Func::Bump, &bp);
        let everywhere = kit.text(stage, EVERYWHERE, 22.0, Palette::ink(), v(right, 1.5))?;
        fade_in_all(stage, &bump_marks, 0.3)?;
        fade_in_all(stage, &[everywhere], 0.4)?;
        line.wait_sentence(stage, 5)?;
        let search = names_now(&kit, stage, &["analytic continuation"])?;
        wait_until(stage, line.end())?;

        // 14_2: 1 + z + z² + … settles at 1/2 and grows at 1.2; 1/(1 − z) has
        // a value at 1.2 all the same
        kit.hold(stage)?;
        let line = say(&kit, stage, "14_2")?;
        let mut gone = vec![border, disk1, here, everywhere, search];
        gone.extend(bump_marks);
        fade_out_all(stage, &gone, 0.4)?;
        painter.leave(stage, 0.5)?;
        let geometric = kit.tex(stage, GEOMETRIC_TEX, 32.0, Palette::ink(), &[], v(0.0, 3.1))?;
        fade_in_all(stage, &[geometric], 0.5)?;
        let half = bars_frame(-3.4, 2.4);
        let big = bars_frame(1.0, 36.0);
        let frames = vec![base(stage, &half), base(stage, &big)];
        fade_in_all(stage, &frames, 0.3)?;
        line.wait_sentence(stage, 2)?;
        let limit = fact_f64("series.half.function")?;
        let two = level(stage, &half, limit, input())?;
        let two_label = kit.text(
            stage,
            fact("series.half.function")?,
            22.0,
            input(),
            v(half.left() - 0.25, half.p(0.0, limit)[1]),
        )?;
        fade_in_all(stage, &[two, two_label], 0.3)?;
        let half_bars = bars(stage, &half, &partial_sums(model::SERIES_INSIDE), input());
        lay_down(stage, &half_bars, 0.12)?;
        let half_says = kit.text(
            stage,
            &series_says(fact("series.half.at")?, fact("series.half.n10")?)?,
            22.0,
            input(),
            v(half.center[0], half.bottom() - 0.4),
        )?;
        fade_in_all(stage, &[half_says], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let big_bars = bars(stage, &big, &partial_sums(model::SERIES_OUTSIDE), warn());
        lay_down(stage, &big_bars, 0.12)?;
        let big_says = kit.text(
            stage,
            &series_says(fact("series.one_two.at")?, fact("series.one_two.n10")?)?,
            22.0,
            warn(),
            v(big.center[0], big.bottom() - 0.4),
        )?;
        // the charts have their own scales: the blue chart's 2 is drawn on this one as well
        let two_here = level(stage, &big, limit, input())?;
        let two_here_label = kit.text(
            stage,
            fact("series.half.function")?,
            22.0,
            input(),
            v(big.left() - 0.25, big.p(0.0, limit)[1]),
        )?;
        fade_in_all(stage, &[big_says, two_here, two_here_label], 0.4)?;
        line.wait_sentence(stage, 4)?;
        let value_tex = kit.tex(stage, VALUE_TEX, 34.0, hero(), &[], v(5.0, 0.6))?;
        fade_in_all(stage, &[value_tex], 0.5)?;
        // "It equals minus five": the value waits for the voice
        line.wait_sentence(stage, 5)?;
        let value_edge = stage.get_center(value_tex)[0] + stage.get_width(value_tex) / 2.0;
        let value_is = kit.tex(
            stage,
            fact("series.one_two.function")?,
            34.0,
            hero(),
            &[],
            v(value_edge + 0.4, 0.6),
        )?;
        stage.move_to(value_is, v(value_edge + 0.14, 0.6), LEFT);
        fade_in_all(stage, &[value_is], 0.3)?;
        wait_until(stage, line.end())?;

        // 14_3: one extension per connected region; continuation along a path
        kit.hold(stage)?;
        let line = say(&kit, stage, "14_3")?;
        let mut gone = vec![
            geometric,
            two,
            two_label,
            half_says,
            big_says,
            two_here,
            two_here_label,
            value_tex,
            value_is,
        ];
        gone.extend(frames);
        gone.extend(half_bars);
        gone.extend(big_bars);
        fade_out_all(stage, &gone, 0.4)?;
        let cp = chain_plane();
        let mut cframe = vec![cp.border(stage, Palette::faint())?];
        cframe.extend(cp.axes(stage)?);
        let unit = disk(stage, &cp, ZERO, 1.0, input(), 0.18)?;
        let pole1 = pole_mark(stage, cp.p(ONE));
        fade_in_all(stage, &cframe, 0.3)?;
        fade_in_all(stage, &[unit], 0.4)?;
        fade_in_all(stage, &pole1, 0.3)?;
        let at_most = kit.text(stage, AT_MOST_ONE, 20.0, Palette::ink(), v(right, 2.5))?;
        fade_in_all(stage, &[at_most], 0.4)?;
        line.wait_sentence(stage, 1)?;
        // the connected region: the whole window but the pole
        let region = stage.arena_mut().add(
            Polygon::new(vec![
                cp.p(C::new(cp.frame.x.0, cp.frame.y.0)),
                cp.p(C::new(cp.frame.x.1, cp.frame.y.0)),
                cp.p(C::new(cp.frame.x.1, cp.frame.y.1)),
                cp.p(C::new(cp.frame.x.0, cp.frame.y.1)),
            ])
            .style(flat(input(), 0.08))
            .build(),
        );
        let region_says = kit.text(stage, REGION, 20.0, input(), v(right, 1.9))?;
        fade_in_all(stage, &[region, region_says], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let step_says = kit.text(stage, STEP, 20.0, Palette::ink(), v(right, 1.1))?;
        let first = disk(
            stage,
            &cp,
            C::real(-0.5),
            fact_f64("continuation.first_disk.radius")?,
            hero(),
            0.12,
        )?;
        let first_c = dot(stage, cp.p(C::real(-0.5)), 0.06, hero());
        fade_in_all(stage, &[step_says, first_c], 0.3)?;
        fade_in_all(stage, &[first], 0.6)?;
        line.wait_sentence(stage, 4)?;
        fade_out_all(stage, &[first, first_c, region], 0.3)?;
        let centres = model::chain_centres(8);
        let walker = dot(stage, cp.p(centres[0]), 0.08, Palette::ink());
        fade_in_all(stage, &[walker], 0.2)?;
        let mut chain = Vec::new();
        for (k, &c) in centres.iter().enumerate().skip(1) {
            let d = disk(stage, &cp, c, (c - ONE).abs(), output(), 0.10)?;
            fade_in_all(stage, &[d], 0.3)?;
            stage.play(
                walker
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.35),
                        ..AnimateArgs::default()
                    })?
                    .move_to(cp.p(c), ORIGIN)?,
            )?;
            chain.push(d);
            if k == 4 {
                line.wait_sentence(stage, 5)?;
                let forced = kit.text(stage, FORCED, 20.0, Palette::ink(), v(right, 0.4))?;
                fade_in_all(stage, &[forced], 0.3)?;
                chain.push(forced);
            }
        }
        // the eight rims all meet at the pole: its mark goes back on top of them
        let pole_top = pole_mark(stage, cp.p(ONE));
        fade_in_all(stage, &pole_top, 0.2)?;
        line.wait_sentence(stage, 6)?;
        let same = kit.text(stage, NOT_ALWAYS, 20.0, hero(), v(right, -0.3))?;
        fade_in_all(stage, &[same], 0.4)?;
        let exact = precisely_now(&kit, stage, PATH_EXACT)?;
        wait_until(stage, line.end())?;

        // 14_4: 1 + 2 + 4 + 8 + …?
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "14_4")?;
        let mut gone = vec![unit, at_most, region_says, step_says, walker, same, exact];
        gone.extend(cframe);
        gone.extend(pole1);
        gone.extend(pole_top);
        gone.extend(chain);
        fade_out_all(stage, &gone, 0.4)?;
        let doubling = kit.tex(stage, DOUBLING_TEX, 36.0, Palette::ink(), &[], v(-3.4, 3.0))?;
        fade_in_all(stage, &[doubling], 0.4)?;
        let asked = predict(&kit, stage, &line, &ASK, v(0.0, 1.4))?;

        // 14_5: no sum; −1 is the function's value; ζ(−1) likewise
        let line = say(&kit, stage, "14_5")?;
        fade_out_all(stage, &asked, 0.3)?;
        let climb = bars_frame(-3.4, DOUBLING_TOP);
        let axis = base(stage, &climb);
        fade_in_all(stage, &[axis], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let red = bars(stage, &climb, &partial_sums(2.0), warn());
        lay_down(stage, &red, 0.12)?;
        let climb_says = kit.text(
            stage,
            &series_says(fact("series.doubling.at")?, fact("series.doubling.n10")?)?,
            22.0,
            warn(),
            v(climb.center[0], climb.top() + 0.35),
        )?;
        let diverges = kit.text(stage, DIVERGES, 24.0, warn(), v(-3.4, -3.15))?;
        fade_in_all(stage, &[climb_says, diverges], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let divider = stage.arena_mut().add(
            polyline(vec![v(0.0, 2.3), v(0.0, -2.6)])
                .style(line_style(Palette::faint(), 3.0))
                .build(),
        );
        let at2 = kit.tex(
            stage,
            &format!("{AT_TWO_TEX}{}", fact("continuation.two")?),
            34.0,
            hero(),
            &[],
            v(right, 1.4),
        )?;
        fade_in_all(stage, &[divider, at2], 0.5)?;
        line.wait_sentence(stage, 3)?;
        let extends = kit.text(stage, EXTENDS, 20.0, Palette::ink(), v(right, 0.5))?;
        fade_in_all(stage, &[extends], 0.4)?;
        line.wait_sentence(stage, 5)?;
        let naturals = kit.tex(stage, NATURAL_SUM_TEX, 30.0, warn(), &[], v(right, -0.6))?;
        fade_in_all(stage, &[naturals], 0.4)?;
        line.wait_sentence(stage, 6)?;
        let zeta = kit.tex(
            stage,
            &format!("{ZETA_TEX}{}", fact("continuation.zeta_minus1")?),
            30.0,
            hero(),
            &[],
            v(right, -2.2),
        )?;
        fade_in_all(stage, &[zeta], 0.5)?;
        line.wait_sentence(stage, 7)?;
        let n_diverges = kit.text(stage, DIVERGES, 20.0, warn(), v(right, -1.25))?;
        fade_in_all(stage, &[n_diverges], 0.4)?;
        wait_until(stage, line.end())?;

        // 14_6: √z carried once around 0 comes back as −1
        kit.hold(stage)?;
        let line = say(&kit, stage, "14_6")?;
        let mut gone = vec![
            doubling, axis, climb_says, diverges, divider, at2, extends, naturals, n_diverges, zeta,
        ];
        gone.extend(red);
        fade_out_all(stage, &gone, 0.4)?;
        let (zi, wo) = sqrt_planes();
        let mut sframe = vec![zi.border(stage, input())?, wo.border(stage, output())?];
        sframe.extend(zi.axes(stage)?);
        sframe.extend(wo.axes(stage)?);
        sframe.push(zi.circle(stage, ZERO, 1.0, Palette::faint(), 2.0)?);
        sframe.push(wo.circle(stage, ZERO, 1.0, Palette::faint(), 2.0)?);
        sframe.push(kit.tex(
            stage,
            "z",
            28.0,
            input(),
            &[],
            v(zi.frame.center[0], zi.frame.top() + 0.3),
        )?);
        sframe.push(kit.tex(
            stage,
            r"\sqrt{z}",
            28.0,
            output(),
            &[],
            v(wo.frame.center[0], wo.frame.top() + 0.35),
        )?);
        fade_in_all(stage, &sframe, 0.4)?;
        line.wait_sentence(stage, 2)?;
        let theta = stage.add_value_tracker(0.0);
        let zdot = dot(stage, zi.p(ONE), 0.09, input());
        let warrow = stage.arena_mut().add(
            polyline(vec![wo.p(ZERO), wo.p(ONE)])
                .style(line_style(output(), 5.0))
                .build(),
        );
        let wdot = dot(stage, wo.p(ONE), 0.09, output());
        let ztrail = stage
            .arena_mut()
            .add(polyline(vec![zi.p(ONE); 2]).style(line_style(input(), 3.0)).build());
        let wtrail = stage
            .arena_mut()
            .add(polyline(vec![wo.p(ONE); 2]).style(line_style(output(), 3.0)).build());
        fade_in_all(stage, &[zdot, warrow, wdot, ztrail, wtrail], 0.3)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let th = value(a, theta).max(1e-3);
                let w = model::sqrt_carried(th);
                a.move_to(zdot, zi.p(C::polar(1.0, th)), ORIGIN);
                a.move_to(wdot, wo.p(w), ORIGIN);
                let _ = a.set_points(warrow, polyline(vec![wo.p(ZERO), wo.p(w)]).build().points());
                let n = 90;
                let zs: Vec<Vec3> = (0..=n)
                    .map(|k| zi.p(C::polar(1.0, th * f64::from(k) / f64::from(n))))
                    .collect();
                let ws: Vec<Vec3> = (0..=n)
                    .map(|k| wo.p(model::sqrt_carried(th * f64::from(k) / f64::from(n))))
                    .collect();
                let _ = a.set_points(ztrail, polyline(zs).build().points());
                let _ = a.set_points(wtrail, polyline(ws).build().points());
            },
            true,
        )?;
        line.wait_sentence(stage, 3)?;
        let rt = line.until(stage, "When you come back", 2.5)?;
        stage.play(tween_linear(theta, TAU, rt)?)?;
        stage.clear_updaters(anchor, true);
        let end = fact_list("continuation.sqrt.end")?;
        let sqrt_says = kit.text(
            stage,
            &format!("{SQRT_SAYS}{}", end[0]).replace('-', "−"),
            24.0,
            output(),
            v(4.9, 1.4),
        )?;
        fade_in_all(stage, &[sqrt_says], 0.4)?;
        line.wait_sentence(stage, 5)?;
        let log_says = kit.text(stage, LOG_SAYS, 20.0, Palette::ink(), v(4.9, 0.6))?;
        fade_in_all(stage, &[log_says], 0.4)?;
        wait_until(stage, line.end())?;

        // 14_7: no way across: a natural boundary
        kit.hold(stage)?;
        let line = say(&kit, stage, "14_7")?;
        let mut gone = vec![zdot, warrow, wdot, ztrail, wtrail, sqrt_says, log_says];
        gone.extend(sframe);
        fade_out_all(stage, &gone, 0.4)?;
        let lp = lacunary_plane();
        let lac = Painter::build(stage, Some(Func::Lacunary), lp, 1.0)?;
        let lborder = lp.border(stage, Palette::faint())?;
        lac.show(stage, 1.0)?;
        fade_in_all(stage, &[lborder], 0.3)?;
        line.wait_sentence(stage, 2)?;
        let lac_tex = kit.tex(stage, LACUNARY_TEX, 32.0, Palette::ink(), &[], v(right, 2.4))?;
        let drawn = kit.text(
            stage,
            &format!(
                "{}{}{}{}",
                DRAWN[0],
                fact("lacunary.terms")?,
                DRAWN[1],
                fact("lacunary.limit")?
            ),
            20.0,
            Palette::dim(),
            v(right, 1.7),
        )?;
        fade_in_all(stage, &[lac_tex, drawn], 0.4)?;
        // "a dense set of points on the edge": the rim ticks double, level by level
        line.wait_for(stage, "near a dense set")?;
        let dense = [
            kit.text(stage, DENSE[0], 20.0, Palette::ink(), v(right, 1.2))?,
            kit.text(stage, DENSE[1], 20.0, Palette::dim(), v(right, 0.85))?,
        ];
        fade_in_all(stage, &dense, 0.3)?;
        let mut rim = Vec::new();
        for k in 0..=RIM_LEVELS {
            let ticks: Vec<Mob> = rim_level(k)
                .into_iter()
                .map(|a| {
                    let tick = polyline(vec![lp.p(C::polar(1.0, a)), lp.p(C::polar(RIM_TICK, a))])
                        .style(line_style(warn(), 3.0))
                        .build();
                    stage.arena_mut().add(tick)
                })
                .collect();
            fade_in_all(stage, &ticks, 0.2)?;
            rim.extend(ticks);
        }
        line.wait_sentence(stage, 4)?;
        let edge = lp.circle(stage, ZERO, 1.0, warn(), 6.0)?;
        let natural = kit.text(stage, NATURAL, 26.0, warn(), v(right, 0.1))?;
        fade_in_all(stage, &[edge, natural], 0.5)?;
        wait_until(stage, line.end())?;

        // 14_8: Γ continued strip by strip with Γ(z) = Γ(z + 1)/z
        kit.hold(stage)?;
        let line = say(&kit, stage, "14_8")?;
        let mut gone = vec![lborder, lac_tex, drawn, edge, natural];
        gone.extend(dense);
        gone.extend(rim);
        fade_out_all(stage, &gone, 0.4)?;
        lac.leave(stage, 0.4)?;
        let gp = gamma_plane();
        // the plane comes up blank and the cover goes on before any colour can show
        let gamma = Painter::build(stage, None, gp, 1.0)?;
        let gborder = gp.border(stage, Palette::faint())?;
        let reveal = stage.add_value_tracker(0.0);
        let cover = stage
            .arena_mut()
            .add(polyline(strip_mask_points(&gp, 0.0)).style(flat(ground(), 1.0)).build());
        gamma.show(stage, 0.1)?;
        stage.add_to_scene(cover)?;
        gamma.to(stage, Some(Func::Gamma), 0.7)?;
        fade_in_all(stage, &[gborder], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let gamma_int = kit.tex(stage, GAMMA_INT_TEX, 28.0, Palette::ink(), &[], v(-2.6, 2.3))?;
        let right_only = kit.text(stage, RIGHT_ONLY, 20.0, Palette::dim(), v(1.6, 2.3))?;
        fade_in_all(stage, &[gamma_int, right_only], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let step_tex = kit.tex(stage, GAMMA_STEP_TEX, 32.0, hero(), &[], v(4.3, 0.6))?;
        fade_in_all(stage, &[step_tex], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let x = value(a, reveal);
                let _ = a.set_points(cover, polyline(strip_mask_points(&gp, x)).build().points());
            },
            true,
        )?;
        let mut new_poles = Vec::new();
        for &p0 in &fact_list("gallery.gamma.poles")? {
            // the values one to the right give the new strip
            let hop = Arrow::new(gp.p(C::new(p0 + 0.6, 1.2)), gp.p(C::new(p0 - 0.6, 1.2)))
                .thickness(3.0)
                .color(hero())
                .build()?;
            let hop = stage.arena_mut().add(hop);
            fade_in_all(stage, &[hop], 0.2)?;
            stage.play(tween_linear(reveal, p0 - 1.0, 0.7)?)?;
            let pm = pole_mark(stage, gp.p(C::real(p0)));
            fade_in_all(stage, &pm, 0.2)?;
            fade_out_all(stage, &[hop], 0.2)?;
            new_poles.extend(pm);
        }
        stage.clear_updaters(anchor, true);
        let pole_says = kit.text(stage, NEW_POLES, 22.0, pole(), v(-2.6, -3.0))?;
        fade_in_all(stage, &[pole_says], 0.4)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![gborder, gamma_int, right_only, step_tex, cover, pole_says];
        all.extend(new_poles);
        fade_out_all(stage, &all, 0.5)?;
        gamma.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(key: &str) -> f64 {
        crate::facts::facts().unwrap().get(key).unwrap().parse().unwrap()
    }

    #[test]
    fn the_bars_show_the_facts() {
        let (inside, outside) = (model::SERIES_INSIDE, model::SERIES_OUTSIDE);
        let half = partial_sums(inside);
        assert_eq!(half.len(), num("series.terms") as usize);
        assert!((half[10] - num("series.half.n10")).abs() < 1e-6);
        let limit = num("series.half.function");
        assert!((1.0 / (1.0 - inside) - limit).abs() < 1e-12);
        assert!(half.iter().all(|&s| s < limit && s < bars_frame(-3.4, 2.4).y.1));
        let big = partial_sums(outside);
        assert!((big[10] - num("series.one_two.n10")).abs() < 0.005);
        assert!(big.windows(2).all(|w| w[1] > w[0]) && big[10] < bars_frame(1.0, 36.0).y.1);
        assert!((1.0 / (1.0 - outside) - num("series.one_two.function")).abs() < 1e-9);
        // the fraction on screen is typed: it has to name the same point
        assert!(VALUE_TEX.contains(&num("series.one_two.at").to_string()));
        assert_eq!(
            series_says("1/2", "1.999023").unwrap(),
            "z = 1/2: 11 terms give 1.999023"
        );
        assert!((1.0 / (1.0 - 2.0) - num("continuation.two")).abs() < 1e-12);
        // the doubling chart holds its tallest bar whole, so nothing is clipped, and its caption
        // prints that bar
        let doubling = partial_sums(2.0);
        assert!(doubling[10] < bars_frame(-3.4, DOUBLING_TOP).y.1);
        assert!((doubling[10] - num("series.doubling.n10")).abs() < 1e-9);
        assert!((num("series.doubling.at") - 2.0).abs() < 1e-12);
        assert!((num("continuation.zeta_minus1") + 1.0 / 12.0).abs() < 1e-6);
    }

    #[test]
    fn the_rim_ticks_mark_where_the_series_blows_up() {
        let mut count = 0;
        for k in 0..=RIM_LEVELS {
            let angles = rim_level(k);
            count += angles.len();
            for a in angles {
                let zeta = C::polar(1.0, a);
                // a root of 1 of order 2^k
                assert!((zeta.powi(1 << k) - ONE).abs() < 1e-9, "{k} {a}");
                // the sum keeps growing as z goes out along that ray
                let sum = |r: f64| {
                    let (mut total, mut w) = (ZERO, zeta.scale(r));
                    for _ in 0..64 {
                        total = total + w;
                        w = w * w;
                    }
                    total.abs()
                };
                assert!(sum(0.999_999_999) > sum(0.999) + 10.0, "{k} {a}");
            }
        }
        assert_eq!(count, 1 << RIM_LEVELS);
        // the ticks stay inside the window
        let lp = lacunary_plane();
        assert!(lp.contains(C::real(RIM_TICK)) && lp.contains(C::real(-RIM_TICK)));
    }

    #[test]
    fn the_mask_meets_the_window_edge() {
        let p = bump_plane();
        let h = p.frame.x.1;
        assert_eq!(p.frame.x, p.frame.y);
        for k in 0..64 {
            let a = TAU * f64::from(k) / 64.0;
            let z = C::polar(square_exit(h, a), a);
            // on the edge of the window, never beyond it
            assert!((z.re.abs().max(z.im.abs()) - h).abs() < 1e-9, "{k}");
        }
        // the corners are sector edges
        for k in [8, 24, 40, 56] {
            let a = TAU * f64::from(k) / 64.0;
            assert!((square_exit(h, a) - h * std::f64::consts::SQRT_2).abs() < 1e-9);
        }
        assert!((num("bump.radius0") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn the_chain_of_disks_goes_round_the_pole_without_covering_it() {
        let centres = model::chain_centres(8);
        assert!(centres[0].abs() < 1e-12 && centres[8].abs() < 1e-12);
        let cp = chain_plane();
        for w in centres.windows(2) {
            let r = (w[0] - ONE).abs();
            // each disk reaches exactly to the pole, and the next centre lies inside it
            assert!((r - 1.0).abs() < 1e-12 && (w[1] - w[0]).abs() < r);
            for z in model::circle(w[1], 1.0, 64) {
                assert!(cp.contains(z), "{z:?}");
            }
        }
        // the first re-centred disk reaches to the pole and pokes outside the unit disk
        let r1 = num("continuation.first_disk.radius");
        assert!((r1 - (ONE - C::real(-0.5)).abs()).abs() < 1e-12);
        assert!(-0.5 - r1 < -1.0 && cp.contains(C::real(-0.5 - r1)));
        // the window stays above the lower third
        assert!(cp.frame.bottom() > -2.78);
    }

    #[test]
    fn the_square_root_comes_back_as_minus_one() {
        let w = model::sqrt_carried(TAU);
        assert!((w - C::real(-1.0)).abs() < 1e-12);
        assert_eq!(
            crate::facts::facts().unwrap().get("continuation.sqrt.end"),
            Some("-1,0")
        );
        // it really is a square root all the way round
        for k in 0..=16 {
            let th = TAU * f64::from(k) / 16.0;
            let s = model::sqrt_carried(th);
            assert!((s * s - C::polar(1.0, th)).abs() < 1e-12);
        }
    }

    #[test]
    fn gamma_is_revealed_pole_by_pole() {
        assert_eq!(
            crate::facts::facts().unwrap().get("gallery.gamma.poles"),
            Some("0,-1,-2,-3,-4")
        );
        let gp = gamma_plane();
        for k in 0..5 {
            let p0 = -f64::from(k);
            assert!(gp.contains(C::real(p0)));
            // the hop arrow sits inside the window
            assert!(gp.contains(C::new(p0 - 0.6, 1.2)) && gp.contains(C::new(p0 + 0.6, 1.2)));
        }
        // the last reveal edge (−5) clamps to the window's left side
        assert_eq!(
            strip_mask_points(&gp, -5.0)[1],
            gp.p(C::new(gp.frame.x.0, gp.frame.y.0))
        );
        // Γ(z) = Γ(z + 1)/z off the poles
        let z = C::new(-1.5, 0.7);
        let (a, b) = (model::gamma(z).unwrap(), model::gamma(z + ONE).unwrap() / z);
        assert!((a - b).abs() < 1e-9 * a.abs());
    }
}
