//! Sequential continuity on a real domain, then the square root's window:
//! continuous at 0, yet its widest window is ε², so halving the tolerance
//! quarters the window and no fixed fraction of ε works for every tolerance.
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, fade_all, hex, rect, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, NUMBER_DROP, PLOT, curve_color, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, limit_color,
    line_style, moving, pieces, say, think_pause, wait_until,
};
use super::old_objection::{RIGHT_X, tex_frac};
use super::shrinking_region::swap_in_turn;
use crate::model::Frac;

pub struct Sequences {
    pub kit: Kit,
}
pub const STAGES: [i64; 4] = [1, 2, 4, 8];
/// The two tolerances the plot shows, as samples of [`root_sample`]: n = 2
/// is the tolerance 1/2 with its widest window 1/4; n = 4 is half that
/// tolerance, with a quarter of that window.
pub const PLOTTED: [i64; 2] = [2, 4];

// the root's fixed words (its numbers are built from the model)
const ROOT_TITLE: &str = "No fixed fraction of ε is enough";
const ROOT_RULE: &str = r"g(x)=\sqrt{x},\quad x\ge0,\quad a=0";
const ROOT_WINDOW: &str =
    r"\varepsilon>0,\ \ \delta=\varepsilon^2:\quad 0\le x<\delta\ \Longrightarrow\ \sqrt{x}<\varepsilon";
const NEXT: &str = "next: continuity through open sets";
// what the pictures wait for, each in the line the test below holds it to
const WIDEST_AT: &str = "widest window";
const HALVE_AT: &str = "Halve the tolerance";
const FRACTION_AT: &str = "no fixed fraction";
/// Samples of √x along a stretch of the plot (a multiple of the drawn
/// curves' piece length), denser near 0, where the root is steep.
const ROOT_SAMPLES: usize = 60;
/// The end marks' radius, as chapters 02 and 04 draw them.
const MARK_R: f64 = 0.07;
/// The right column's rows (scene units).
const ROW_Y: [f64; 3] = [2.5, 1.7, 0.75];

/// An exact countersequence for the opening limit at its endpoint.
pub fn approaching(n: i64) -> Result<Frac, String> {
    if n < 1 {
        return Err("sequence index must be positive".to_owned());
    }
    Frac::whole(1).minus(Frac::whole(1).over(Frac::whole(n))?)
}

/// x=1/n², sqrt(x)=1/n and their ratio; computed as exact fractions.
pub fn root_sample(n: i64) -> Result<(Frac, Frac, Frac), String> {
    if n < 1 {
        return Err("sequence index must be positive".to_owned());
    }
    let y = Frac::whole(1).over(Frac::whole(n))?;
    let x = y.times(y)?;
    Ok((x, y, y.over(x)?))
}

/// The square root's widest window at 0 on [0, ∞) for a tolerance ε > 0:
/// √x < ε exactly when 0 ≤ x < ε², so the window's radius is ε². A
/// tolerance that is not positive is refused.
pub fn widest_root_window(eps: Frac) -> Result<Frac, String> {
    if eps <= Frac::whole(0) {
        return Err(format!("the tolerance {eps} is not positive"));
    }
    eps.times(eps)
}

/// √x on the plot from 0 to `to`, sampled at x = to·(k/n)², detached.
fn root_curve(stage: &mut Stage<'_>, to: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
    let pts: Vec<Vec3> = (0..=ROOT_SAMPLES)
        .map(|k| {
            let s = k as f64 / ROOT_SAMPLES as f64;
            let x = to * s * s;
            PLOT.p(x, x.sqrt())
        })
        .collect();
    Ok(stage.arena_mut().add(pieces(&pts, color, width)))
}

/// What the plot shows for one tolerance ε, detached: the band 0 ≤ y < ε,
/// the curve's piece over the window, the window [0, δ) on the input axis
/// (its radius from a = 0; only x ≥ 0 exists) and its hollow end at δ = ε²,
/// where √x = ε exactly; the tags ε and δ; and the row "ε = …: δ = …". The
/// filled end at 0 belongs to every window and is drawn once.
struct RootWindow {
    shapes: [Mob; 4],
    tags: [Mob; 2],
    row: Mob,
}

impl RootWindow {
    fn new(kit: &Kit, stage: &mut Stage<'_>, eps: Frac) -> fmn::Result<Self> {
        let delta = widest_root_window(eps).map_err(fail)?;
        let (o, top, end) = (PLOT.p(0.0, 0.0), PLOT.p(1.0, eps.f64()), PLOT.p(delta.f64(), 0.0));
        let band = rect(o, top[0] - o[0], top[1] - o[1], flat(limit_color(), 0.22))?;
        let band = stage.arena_mut().add(band);
        let piece = root_curve(stage, delta.f64(), Palette::hero(), 7.0)?;
        let window = Line::new(o, end).style(line_style(moving(), 9.0)).build()?;
        let window = stage.arena_mut().add(window);
        let hollow = Circle::new()
            .radius(MARK_R)
            .arc_center(end)
            .style(Style::default().fill(hex(BACKGROUND), 1.0).stroke(moving(), 3.0, 1.0))
            .build();
        let hollow = stage.arena_mut().add(hollow);
        let eps_tag = kit.tex(stage, r"\varepsilon", 28.0, limit_color(), &[], v(top[0] + 0.3, top[1]))?;
        // under the axis's numbers, so the tag stays clear of them as δ shrinks
        let delta_tag = kit.tex(
            stage,
            r"\delta",
            28.0,
            moving(),
            &[],
            v(end[0], o[1] - 2.0 * NUMBER_DROP),
        )?;
        let row = kit.tex_left(
            stage,
            &format!(
                r"\varepsilon={}:\quad\delta={}",
                tex_frac(&eps.to_string()),
                tex_frac(&delta.to_string())
            ),
            30.0,
            moving(),
            &[],
            v(RIGHT_X, ROW_Y[2]),
        )?;
        Ok(Self {
            shapes: [band, piece, window, hollow],
            tags: [eps_tag, delta_tag],
            row,
        })
    }
}

fn tex(kit: &Kit, s: &mut Stage<'_>, text: &str, y: f64, size: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.tex(s, text, size, c, &[], v(0.0, y))
}
fn label(kit: &Kit, s: &mut Stage<'_>, text: &str, y: f64, size: f64) -> fmn::Result<Mob> {
    kit.text(s, text, size, Palette::ink(), v(0.0, y))
}

impl SceneConstruct for Sequences {
    fn name(&self) -> &str {
        "10_approaching_along_sequences"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let _line = say(&kit, stage, "10_1")?;
        let title = label(&kit, stage, "Approach along sequences", 3.45, 34.0)?;
        let rule = tex(
            &kit,
            stage,
            r"x_n\in D,\quad x_n\to a\quad\Longrightarrow\quad f(x_n)\to f(a)",
            2.15,
            34.0,
            Palette::teal(),
        )?;
        let every = label(
            &kit,
            stage,
            "continuity requires this for every approaching sequence",
            1.3,
            25.0,
        )?;
        fade_in_lagged(stage, &[title, rule, every], 0.7, 0.1)?;

        let _line = say(&kit, stage, "10_2")?;
        let seq = tex(&kit, stage, r"x_n=1-1/n\to1,\qquad n\ge1", 0.3, 34.0, Palette::blue())?;
        fade_in_all(stage, &[seq], 0.4)?;
        let axis = stage.arena_mut().add(
            Line::new(v(-5.0, -1.0), v(5.0, -1.0))
                .style(line_style(Palette::dim(), 3.0))
                .build()?,
        );
        let zero = kit.text(stage, "0", 25.0, Palette::ink(), v(-5.0, -1.6))?;
        let one = kit.text(stage, "1", 25.0, Palette::ink(), v(5.0, -1.6))?;
        fade_in_all(stage, &[axis, zero, one], 0.4)?;
        let mut old = None;
        for n in STAGES {
            let x = approaching(n).map_err(fail)?;
            let dot = stage.arena_mut().add(
                Dot::new()
                    .point(v(-5.0 + 10.0 * x.f64(), -1.0))
                    .radius(0.09)
                    .style(flat(Palette::blue(), 1.0))
                    .build(),
            );
            let tag = tex(
                &kit,
                stage,
                &format!(r"n={n}:\quad x_n={x}"),
                -2.3,
                29.0,
                Palette::blue(),
            )?;
            if let Some(o) = old {
                fade_out_all(stage, &[o], 0.15)?;
            }
            fade_in_all(stage, &[dot, tag], 0.35)?;
            stage.wait(0.65)?;
            old = Some(tag);
        }
        let _line = say(&kit, stage, "10_3")?;
        let fail_line = tex(&kit, stage, r"f(x_n)=0\ne1=f(1)", -3.15, 34.0, Palette::gold())?;
        fade_in_all(stage, &[fail_line], 0.5)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _question = say(&kit, stage, "10_4")?;
        let question = label(&kit, stage, "Can checking several good sequences settle it?", 2.7, 30.0)?;
        let mark = label(&kit, stage, "?", 0.0, 46.0)?;
        fade_in_all(stage, &[question, mark], 0.5)?;
        think_pause(&kit, stage, v(0.0, 0.0), 0.5)?;
        fade_out_all(stage, &[mark], 0.2)?;
        let _line = say(&kit, stage, "10_5")?;
        let answer = label(&kit, stage, "No. Every approaching sequence must work.", 1.75, 29.0)?;
        let failure = tex(
            &kit,
            stage,
            r"\exists\varepsilon_0>0\quad\forall\delta>0\quad\exists x\in D",
            0.6,
            32.0,
            Palette::ink(),
        )?;
        let bad = tex(
            &kit,
            stage,
            r"|x-a|<\delta,\qquad |f(x)-f(a)|\ge\varepsilon_0",
            -0.15,
            32.0,
            Palette::gold(),
        )?;
        fade_in_lagged(stage, &[answer, failure, bad], 0.6, 0.1)?;
        let _line = say(&kit, stage, "10_6")?;
        let choice = tex(
            &kit,
            stage,
            r"\delta=1/n:\quad |x_n-a|<1/n\to0",
            -1.3,
            32.0,
            Palette::blue(),
        )?;
        let miss = tex(
            &kit,
            stage,
            r"|f(x_n)-f(a)|\ge\varepsilon_0\quad\text{for every }n",
            -2.15,
            31.0,
            Palette::gold(),
        )?;
        let conclusion = label(
            &kit,
            stage,
            "failure of continuity produces a failing sequence",
            -3.0,
            26.0,
        )?;
        fade_in_lagged(stage, &[choice, miss, conclusion], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 10.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        // 10_7: the square root on [0, ∞) at 0, its widest window for 1/2
        let line = say(&kit, stage, "10_7")?;
        let title2 = label(&kit, stage, ROOT_TITLE, 3.45, 33.0)?;
        let rule = kit.tex_left(stage, ROOT_RULE, 32.0, Palette::teal(), &[], v(RIGHT_X, ROW_Y[0]))?;
        let window_rule = kit.tex_left(stage, ROOT_WINDOW, 26.0, Palette::ink(), &[], v(RIGHT_X, ROW_Y[1]))?;
        let mut picture = vec![title2, rule, window_rule];
        picture.extend(PLOT.axes(&kit, stage)?);
        picture.push(root_curve(stage, 1.0, curve_color(), BRIGHT_W)?);
        fade_in_lagged(stage, &picture, 0.9, 0.05)?;
        let tolerances = PLOTTED
            .iter()
            .map(|&n| root_sample(n).map(|(_, eps, _)| eps).map_err(fail))
            .collect::<fmn::Result<Vec<Frac>>>()?;
        line.wait_for(stage, WIDEST_AT)?;
        let wide = RootWindow::new(&kit, stage, tolerances[0])?;
        let zero = Dot::new()
            .point(PLOT.p(0.0, 0.0))
            .radius(MARK_R)
            .style(flat(moving(), 1.0))
            .build();
        let mut first = wide.shapes.to_vec();
        first.push(stage.arena_mut().add(zero));
        first.extend(wide.tags);
        first.push(wide.row);
        fade_in_all(stage, &first, 0.6)?;

        // 10_8: half the tolerance, a quarter of the window; the shapes and
        // tags morph, the row is replaced in turn
        let line = say(&kit, stage, "10_8")?;
        line.wait_for(stage, HALVE_AT)?;
        let narrow = RootWindow::new(&kit, stage, tolerances[1])?;
        let mut anims = Vec::new();
        for (&from, &to) in wide
            .shapes
            .iter()
            .chain(&wide.tags)
            .zip(narrow.shapes.iter().chain(&narrow.tags))
        {
            anims.push(stage.prepare(fmn::animation::replacement_transform(from, to).rt(0.8))?);
        }
        anims.push(swap_in_turn(stage, &[wide.row], &[narrow.row], 0.8)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, FRACTION_AT)?;
        let next = label(&kit, stage, NEXT, -3.5, 24.0)?;
        fade_in_all(stage, &[next], 0.4)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approaching_stays_in_domain_and_closes_exact_distance() {
        for n in 1..=256 {
            let x = approaching(n).unwrap();
            assert!(x >= Frac::whole(0) && x < Frac::whole(1));
            assert_eq!(
                Frac::whole(1).minus(x).unwrap().times(Frac::whole(n)).unwrap(),
                Frac::whole(1)
            );
            assert_eq!(crate::model::limit(x.f64()).unwrap(), 0.0);
        }
        assert!(approaching(0).is_err());
    }
    #[test]
    fn root_ratio_diverges_while_both_errors_shrink() {
        let mut last = Frac::whole(2);
        for n in 1..=256 {
            let (x, y, r) = root_sample(n).unwrap();
            assert_eq!(y.times(y).unwrap(), x);
            assert_eq!(r, Frac::whole(n));
            assert!(y < last);
            last = y;
        }
        assert!(root_sample(0).is_err());
    }
    /// The widest window is the tolerance squared: halving the tolerance
    /// quarters it, and a fixed fraction c of the tolerance is too wide once
    /// the tolerance is below c (its window holds ε², whose root is ε).
    #[test]
    fn the_widest_root_window_is_the_tolerance_squared() {
        for k in 1..=64 {
            for j in [1, 2, 3, 5] {
                let eps = Frac::new(j, k).unwrap();
                let w = widest_root_window(eps).unwrap();
                assert_eq!(w, eps.times(eps).unwrap());
                let half = widest_root_window(eps.over(Frac::whole(2)).unwrap()).unwrap();
                assert_eq!(half.times(Frac::whole(4)).unwrap(), w);
            }
        }
        for c in [Frac::whole(1), Frac::new(1, 10).unwrap(), Frac::new(1, 22).unwrap()] {
            let eps = c.over(Frac::whole(2)).unwrap();
            assert!(widest_root_window(eps).unwrap() < c.times(eps).unwrap());
        }
        assert!(widest_root_window(Frac::whole(0)).is_err());
        assert!(widest_root_window(Frac::whole(-1)).is_err());
        let (x2, y2, _) = root_sample(PLOTTED[0]).unwrap();
        let (x4, y4, _) = root_sample(PLOTTED[1]).unwrap();
        assert_eq!(
            (widest_root_window(y2).unwrap(), widest_root_window(y4).unwrap()),
            (x2, x4)
        );
    }
    /// Every phrase a picture waits for is said once, in its line; the
    /// table's lines are gone.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [("10_7", WIDEST_AT), ("10_8", HALVE_AT), ("10_8", FRACTION_AT)] {
            assert_eq!(script.text(id).unwrap().matches(phrase).count(), 1, "{id}: {phrase}");
        }
        assert!(script.text("10_9").is_none() && script.text("10_10").is_none());
    }
}
