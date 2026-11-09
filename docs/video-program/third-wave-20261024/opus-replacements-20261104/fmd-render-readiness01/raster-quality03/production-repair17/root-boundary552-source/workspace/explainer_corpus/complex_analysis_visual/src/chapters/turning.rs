//! 02_turning_numbers: complex numbers as arrows that add tip to tail and
//! multiply by stretching and turning (storyboard 02_1–02_6).
//!
//! - 3 + 2i as a point, its two parts as guides, then as an arrow.
//! - 1 − i added tip to tail.
//! - A grid with an F-shaped glyph turned a quarter turn by i, with a live
//!   angle; 1 → i → −1 as two quarter turns.
//! - Multiplying by 1 + i stretches by √2 and turns 45°; the prediction
//!   asks for (1 + i)², and the second multiplication answers it.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Plane, dashed, derivative, dot, fact, fade_in_all, fade_out_all, hero, input, line_style, output, predict, say,
    shift_all, value, wait_until,
};
use crate::model::{C, I, ONE, TURN_ADDED, TURN_BY, TURN_POINT, ZERO};

pub const AXES: [&str; 2] = ["real part", "imaginary part"];
pub const WRITE_TEX: &str = r"z = x + iy";
pub const ADDING: &str = "adding: tip to tail";
pub const TIMES_I: &str = "every point times i";
pub const TURNED: &str = "turned:";
pub const DEGREE: &str = "°";
pub const I_I_TEX: &str = r"i \cdot i = -1";
pub const STRETCH_TURN: [&str; 2] = ["multiplying by w: stretch by |w|,", "turn by the angle of w"];
pub const LENGTH_TEX: &str = r"|1 + i| = \sqrt{2} \approx ";
pub const ANGLE: &str = "angle: ";
pub const NAMES: [&str; 2] = ["names:", "length = modulus |w|, angle = argument arg w"];
pub const ASK: [&str; 1] = ["(1 + i)² = ?"];
pub const TOTAL: [&str; 2] = ["total turn: ", "total stretch: "];
pub const SQUARE_TEX: &str = r"(1 + i)^2 = ";
pub const RULE: &str = "lengths multiply, angles add";

/// Where "i" and "−1" are written in 02_3: off the axis each point lies on, and outside the
/// unit circle the walker draws.
pub const TURN_NAMES_AT: [C; 2] = [C::new(-0.3, 1.25), C::new(-1.35, 0.35)];
/// Where "1 + i" is written: past the arrow's tip, clear of the grid's lines whenever the grid
/// rests (the lines pass over it only while the second multiplication runs).
pub const W_NAME_AT: C = C::new(1.5, 1.5);
/// Where "2i" is written: over the grid's top edge, left of the axis.
pub const W2_NAME_AT: C = C::new(-0.4, 2.3);

/// The one plane of the chapter: a unit is one scene unit.
pub fn turn_plane() -> Plane {
    Plane::new(v(-2.0, -0.1), 8.4, (-3.4, 5.0), (-2.6, 2.9))
}

/// The right-hand column's x.
pub const COLUMN: f64 = 4.6;
/// The radius of the arc that marks 1 + i's angle (02_4 – 02_6), in plane units.
pub const ANGLE_ARC_R: f64 = 0.45;
/// How long a gold mark takes to come and to go (02_5, 02_6), seconds.
pub const PULSE: (f64, f64) = (0.25, 0.4);
/// How long the recalled eighth of a turn takes to sweep (02_5), seconds.
pub const SWEEP_S: f64 = 0.7;
/// How far under a readout its gold mark runs, scene units.
pub const UNDERLINE_GAP: f64 = 0.06;

/// The grid's lines (x, y in [−1, 1], step ½) and the F glyph's two
/// strokes, as plane points; the glyph is asymmetric so a turn shows.
pub fn sheet() -> (Vec<[C; 2]>, Vec<Vec<C>>) {
    let mut lines = Vec::new();
    for k in -2..=2 {
        let t = f64::from(k) / 2.0;
        lines.push([C::new(t, -1.0), C::new(t, 1.0)]);
        lines.push([C::new(-1.0, t), C::new(1.0, t)]);
    }
    let glyph = vec![
        vec![C::new(0.15, 0.1), C::new(0.15, 0.9), C::new(0.7, 0.9)],
        vec![C::new(0.15, 0.5), C::new(0.5, 0.5)],
    ];
    (lines, glyph)
}

/// `by` raised to the share `s` of the way: stretch |by|ˢ, turn s·arg(by)
/// (the path a multiplication is animated along).
pub fn partway(by: C, s: f64) -> C {
    C::polar(by.abs().powf(s), by.arg() * s)
}

/// "a + bi" as TeX source, for whole-number parts (TeX sets the hyphen as a
/// minus sign).
pub fn written(z: C) -> String {
    let (re, im) = (z.re.round() as i64, z.im.round() as i64);
    let imag = match im.abs() {
        0 => String::new(),
        1 => "i".to_owned(),
        n => format!("{n}i"),
    };
    match (re, im) {
        (_, 0) => format!("{re}"),
        (0, _) => format!("{}{imag}", if im < 0 { "-" } else { "" }),
        _ => format!("{re} {} {imag}", if im < 0 { "-" } else { "+" }),
    }
}

/// A complex number printed in facts as "re,im".
fn fact_c(key: &str) -> fmn::Result<C> {
    let parts: Vec<f64> = super::draw::fact_list(key)?;
    Ok(C::new(parts[0], parts[1]))
}

/// An arrow that reaches both its points: the voice says "an arrow from zero" and "tip to
/// tail". `Arrow::new` alone trims `MED_SMALL_BUFF` (0.25) off each end.
fn reaching_arrow(a: Vec3, b: Vec3, color: Srgb) -> fmn::Result<VMobject> {
    Ok(Arrow::new(a, b).buff(0.0).thickness(5.0).color(color).build()?)
}

/// The arc of radius `r` about 0 from angle `a0` to `a1`.
fn arc(r: f64, a0: f64, a1: f64) -> Vec<C> {
    (0..=48)
        .map(|k| C::polar(r, a0 + (a1 - a0) * f64::from(k) / 48.0))
        .collect()
}

/// A gold underline under each of `rows`, in and out again: the readouts the voice is on.
fn underline_pulse(stage: &mut Stage<'_>, rows: &[Mob]) -> fmn::Result<()> {
    let mut lines = Vec::with_capacity(rows.len());
    for &m in rows {
        let (at, w, h) = (stage.get_center(m), stage.get_width(m), stage.get_height(m));
        let y = at[1] - h / 2.0 - UNDERLINE_GAP;
        let line = polyline(vec![v(at[0] - w / 2.0, y), v(at[0] + w / 2.0, y)])
            .style(line_style(hero(), 4.0))
            .build();
        lines.push(stage.arena_mut().add(line));
    }
    fade_in_all(stage, &lines, PULSE.0)?;
    fade_out_all(stage, &lines, PULSE.1)
}

/// A gold arc of radius `r` about 0 swept from angle 0 to `to` in the plane `p`, then gone: the
/// turn acted out again, leaving what was on screen as it was.
fn sweep_arc(stage: &mut Stage<'_>, p: Plane, r: f64, to: f64, rt: f64) -> fmn::Result<()> {
    let share = stage.add_value_tracker(0.0);
    let swept = p.path(stage, &arc(r, 0.0, to * 1e-3), hero(), 7.0)?;
    stage.add_to_scene(swept)?;
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let th = to * value(a, share).max(1e-3);
            let pts: Vec<Vec3> = arc(r, 0.0, th).into_iter().map(|z| p.p(z)).collect();
            let _ = a.set_points(swept, polyline(pts).build().points());
        },
        true,
    )?;
    stage.play(tween(share, 1.0, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    fade_out_all(stage, &[swept], PULSE.1)
}

/// The grid and glyph on screen: each mob with the plane points it was
/// drawn from.
pub struct Sheet {
    pub items: Vec<(Mob, Vec<C>)>,
}

impl Sheet {
    fn build(stage: &mut Stage<'_>, p: &Plane) -> Self {
        let (lines, glyph) = sheet();
        let mut items = Vec::new();
        for l in lines {
            let m = stage.arena_mut().add(
                polyline(vec![p.p(l[0]), p.p(l[1])])
                    .style(line_style(input(), 2.5))
                    .build(),
            );
            items.push((m, l.to_vec()));
        }
        for g in glyph {
            let pts: Vec<Vec3> = g.iter().map(|&z| p.p(z)).collect();
            let m = stage
                .arena_mut()
                .add(polyline(pts).style(line_style(hero(), 7.0)).build());
            items.push((m, g));
        }
        Self { items }
    }

    fn mobs(&self) -> Vec<Mob> {
        self.items.iter().map(|(m, _)| *m).collect()
    }

    /// Multiply the sheet, already multiplied by `from`, by `by`, over `rt`
    /// seconds, along `partway`; tracker `share` (0 → 1) drives it.
    fn multiply(&self, stage: &mut Stage<'_>, p: Plane, from: C, by: C, share: Mob, rt: f64) -> fmn::Result<()> {
        let items = self.items.clone();
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.set_tracker_value(share, 0.0)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let m = from * partway(by, value(a, share));
                for (mob, zs) in &items {
                    let pts: Vec<Vec3> = zs.iter().map(|&z| p.p(z * m)).collect();
                    let _ = a.set_points(*mob, polyline(pts).build().points());
                }
            },
            true,
        )?;
        stage.play(tween(share, 1.0, rt)?)?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        Ok(())
    }
}

pub struct Turning {
    pub kit: Kit,
}

impl SceneConstruct for Turning {
    fn name(&self) -> &str {
        "02_turning_numbers"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let tp = turn_plane();
        let arrow = |stage: &mut Stage<'_>, a: C, b: C, color: Srgb| -> fmn::Result<Mob> {
            let m = reaching_arrow(tp.p(a), tp.p(b), color)?;
            Ok(stage.arena_mut().add(m))
        };

        // 02_1: a point in a plane; its two parts; the arrow from 0
        let line = say(&kit, stage, "02_1")?;
        let mut frame = vec![tp.border(stage, Palette::faint())?];
        frame.extend(tp.axes(stage)?);
        frame.push(kit.text(stage, AXES[0], 20.0, Palette::dim(), tp.p(C::new(4.1, -0.3)))?);
        frame.push(kit.text(stage, AXES[1], 20.0, Palette::dim(), tp.p(C::new(1.05, 2.6)))?);
        fade_in_all(stage, &frame, 0.5)?;
        let a_dot = dot(stage, tp.p(TURN_POINT), 0.09, input());
        fade_in_all(stage, &[a_dot], 0.3)?;
        line.wait_for(stage, "how far right")?;
        let right = dashed(stage, tp.p(TURN_POINT), tp.p(C::real(TURN_POINT.re)), input())?;
        let right_label = kit.tex(
            stage,
            &written(C::real(TURN_POINT.re)),
            26.0,
            input(),
            &[],
            tp.p(C::new(TURN_POINT.re, -0.35)),
        )?;
        fade_in_all(stage, &[right, right_label], 0.4)?;
        line.wait_for(stage, "how far up")?;
        let up = dashed(stage, tp.p(TURN_POINT), tp.p(C::new(0.0, TURN_POINT.im)), input())?;
        let up_label = kit.tex(
            stage,
            &written(C::new(0.0, TURN_POINT.im)),
            26.0,
            input(),
            &[],
            tp.p(C::new(-0.4, TURN_POINT.im)),
        )?;
        fade_in_all(stage, &[up, up_label], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let write = kit.tex(stage, WRITE_TEX, 32.0, Palette::ink(), &[], v(COLUMN, 2.2))?;
        let a_arrow = arrow(stage, ZERO, TURN_POINT, input())?;
        let a_label = kit.tex(
            stage,
            &written(TURN_POINT),
            28.0,
            input(),
            &[],
            tp.p(TURN_POINT + C::new(0.55, 0.3)),
        )?;
        fade_in_all(stage, &[write], 0.4)?;
        fade_in_all(stage, &[a_arrow, a_label], 0.5)?;
        wait_until(stage, line.end())?;

        // 02_2: adding is tip to tail; multiplying every point by i turns the plane
        kit.hold(stage)?;
        let line = say(&kit, stage, "02_2")?;
        fade_out_all(stage, &[right, right_label, up, up_label, a_dot], 0.3)?;
        let b_arrow = arrow(stage, ZERO, TURN_ADDED, output())?;
        let b_label = kit.tex(
            stage,
            &written(TURN_ADDED),
            28.0,
            output(),
            &[],
            tp.p(TURN_ADDED + C::new(0.6, 0.15)),
        )?;
        fade_in_all(stage, &[b_arrow, b_label], 0.4)?;
        let adding = kit.text(stage, ADDING, 22.0, Palette::ink(), v(COLUMN, 1.4))?;
        fade_in_all(stage, &[adding], 0.3)?;
        let shift = tp.unit();
        shift_all(
            stage,
            &[b_arrow, b_label],
            shift * TURN_POINT.re,
            shift * TURN_POINT.im,
            1.0,
        )?;
        let sum = fact_c("turn.sum")?;
        let s_arrow = arrow(stage, ZERO, sum, Palette::ink())?;
        let s_label = kit.tex(
            stage,
            &written(sum),
            28.0,
            Palette::ink(),
            &[],
            tp.p(sum + C::new(0.3, -0.45)),
        )?;
        fade_in_all(stage, &[s_arrow, s_label], 0.5)?;
        line.wait_sentence(stage, 1)?;
        fade_out_all(
            stage,
            &[a_arrow, a_label, b_arrow, b_label, s_arrow, s_label, adding, write],
            0.4,
        )?;
        let grid = Sheet::build(stage, &tp);
        fade_in_all(stage, &grid.mobs(), 0.6)?;
        line.wait_sentence(stage, 2)?;
        let times_i = kit.text(stage, TIMES_I, 22.0, Palette::ink(), v(COLUMN, 1.4))?;
        let turned = kit.text(stage, TURNED, 24.0, derivative(), v(COLUMN - 0.8, 0.6))?;
        let share = stage.add_value_tracker(0.0);
        let quarter = (I.arg() / TAU) * 360.0;
        let degrees = kit.live_value(
            stage,
            move |s| value(s, share) * quarter,
            0,
            false,
            30.0,
            derivative(),
            v(COLUMN + 0.35, 0.6),
            ORIGIN,
        )?;
        let degree_mark = kit.text(stage, DEGREE, 24.0, derivative(), v(COLUMN + 0.6, 0.68))?;
        fade_in_all(stage, &[times_i, turned, degree_mark], 0.3)?;
        line.wait_for(stage, "and the whole plane turns")?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(2.0);
        grid.multiply(stage, tp, ONE, I, share, rt)?;
        wait_until(stage, line.end())?;

        // 02_3: i · i is a half turn: 1 → i → −1
        kit.hold(stage)?;
        let line = say(&kit, stage, "02_3")?;
        stage.clear_updaters(degrees, true);
        let mut gone = vec![times_i, turned, degree_mark, degrees];
        gone.extend(grid.mobs());
        fade_out_all(stage, &gone, 0.4)?;
        let walker = dot(stage, tp.p(ONE), 0.1, hero());
        let one = kit.tex(stage, "1", 26.0, Palette::ink(), &[], tp.p(C::new(1.0, -0.35)))?;
        fade_in_all(stage, &[walker, one], 0.3)?;
        let angle = stage.add_value_tracker(0.0);
        let trail = stage
            .arena_mut()
            .add(polyline(vec![tp.p(ONE); 2]).style(line_style(hero(), 4.0)).build());
        stage.add_to_scene(trail)?;
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let th = value(a, angle).max(1e-3);
                a.move_to(walker, tp.p(C::polar(1.0, th)), ORIGIN);
                let pts: Vec<Vec3> = arc(1.0, 0.0, th).into_iter().map(|z| tp.p(z)).collect();
                let _ = a.set_points(trail, polyline(pts).build().points());
            },
            true,
        )?;
        let mut labels = Vec::new();
        for (k, name) in ["i", "-1"].into_iter().enumerate() {
            stage.play(tween(angle, PI / 2.0 * (k as f64 + 1.0), 0.9)?)?;
            let l = kit.tex(stage, name, 26.0, Palette::ink(), &[], tp.p(TURN_NAMES_AT[k]))?;
            fade_in_all(stage, &[l], 0.2)?;
            labels.push(l);
        }
        stage.clear_updaters(anchor, true);
        line.wait_sentence(stage, 2)?;
        let ii = explainer_kit::props::equation_card(&kit, stage, I_I_TEX, 32.0, &[], v(COLUMN, 1.4))?;
        fade_in_all(stage, &[ii], 0.4)?;
        wait_until(stage, line.end())?;

        // 02_4: multiplying by 1 + i: stretch √2, turn 45°
        kit.hold(stage)?;
        let line = say(&kit, stage, "02_4")?;
        let mut gone = vec![walker, one, trail, ii];
        gone.extend(labels);
        fade_out_all(stage, &gone, 0.4)?;
        let grid = Sheet::build(stage, &tp);
        fade_in_all(stage, &grid.mobs(), 0.5)?;
        let stretch_turn: Vec<Mob> = STRETCH_TURN
            .iter()
            .enumerate()
            .map(|(k, s)| kit.text(stage, s, 20.0, Palette::ink(), v(COLUMN, 2.4 - 0.45 * k as f64)))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &stretch_turn, 0.4)?;
        line.wait_sentence(stage, 1)?;
        // the multiplier wears the stretch-and-turn's teal: the F it acts on is gold
        let w_arrow = arrow(stage, ZERO, TURN_BY, derivative())?;
        let w_label = kit.tex(stage, &written(TURN_BY), 26.0, derivative(), &[], tp.p(W_NAME_AT))?;
        let length = kit.tex(
            stage,
            &format!("{LENGTH_TEX}{}", fact("turn.one_plus_i.length")?),
            26.0,
            derivative(),
            &[],
            v(COLUMN, 1.2),
        )?;
        let angle_arc = tp.path(stage, &arc(ANGLE_ARC_R, 0.0, TURN_BY.arg()), derivative(), 3.0)?;
        let angle_says = kit.text(
            stage,
            &format!("{ANGLE}{}{DEGREE}", fact("turn.one_plus_i.angle_deg")?),
            22.0,
            derivative(),
            v(COLUMN, 0.55),
        )?;
        fade_in_all(stage, &[w_arrow, w_label], 0.4)?;
        fade_in_all(stage, &[length, angle_arc, angle_says], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let names = explainer_kit::props::lower_third(&kit, stage, NAMES[0], NAMES[1])?;
        fade_in_all(stage, &[names], 0.4)?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(2.0);
        grid.multiply(stage, tp, ONE, TURN_BY, share, rt)?;
        wait_until(stage, line.end())?;

        // 02_5: what does a second multiplication by 1 + i make?
        kit.hold(stage)?;
        let line = say(&kit, stage, "02_5")?;
        fade_out_all(stage, &[names], 0.3)?;
        // the voice recalls the turn just made: its angle is swept again, on what is already
        // on screen, so nothing here answers the question that follows
        line.wait_for(stage, "an eighth of a turn")?;
        sweep_arc(stage, tp, ANGLE_ARC_R, TURN_BY.arg(), SWEEP_S)?;
        underline_pulse(stage, &[angle_says])?;
        let asked = predict(&kit, stage, &line, &ASK, v(COLUMN, -1.0))?;

        // 02_6: a quarter turn and a stretch of 2: (1 + i)² = 2i
        let line = say(&kit, stage, "02_6")?;
        // the question goes, and with it the caption up since 02_4: the voice now uses the rows
        let mut gone = asked;
        gone.extend(stretch_turn);
        fade_out_all(stage, &gone, 0.3)?;
        let rt = line.until(stage, "So one plus i", 2.5)?;
        grid.multiply(stage, tp, TURN_BY, TURN_BY, share, rt)?;
        let total_turn = kit.text(
            stage,
            &format!("{}{}{DEGREE}", TOTAL[0], fact("turn.square.angle_deg")?),
            22.0,
            derivative(),
            v(COLUMN, -0.25),
        )?;
        let total_stretch = kit.text(
            stage,
            &format!("{}{}", TOTAL[1], fact("turn.square.length")?),
            22.0,
            derivative(),
            v(COLUMN, -0.8),
        )?;
        fade_in_all(stage, &[total_turn, total_stretch], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let square = fact_c("turn.square.value")?;
        let w2_arrow = arrow(stage, ZERO, square, derivative())?;
        let w2_label = kit.tex(stage, &written(square), 26.0, derivative(), &[], tp.p(W2_NAME_AT))?;
        let boxed = explainer_kit::props::equation_card(
            &kit,
            stage,
            &format!("{SQUARE_TEX}{}", written(square)),
            34.0,
            &[],
            v(COLUMN, -1.8),
        )?;
        fade_in_all(stage, &[w2_arrow, w2_label, boxed], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let rule = kit.text(stage, RULE, 24.0, hero(), v(COLUMN, -2.7))?;
        fade_in_all(stage, &[rule], 0.4)?;
        // the rule read off the rows: the two lengths, then the two angles
        underline_pulse(stage, &[length, total_stretch])?;
        line.wait_for(stage, "and angles add")?;
        underline_pulse(stage, &[angle_says, total_turn])?;
        // the moves the voice now names are the arrows: the grid and the F make room for them
        line.wait_for(stage, "Every complex derivative")?;
        fade_out_all(stage, &grid.mobs(), 0.5)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![
            w_arrow,
            w_label,
            length,
            angle_arc,
            angle_says,
            total_turn,
            total_stretch,
            w2_arrow,
            w2_label,
            boxed,
            rule,
        ];
        all.extend(frame);
        fade_out_all(stage, &all, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(key: &str) -> &'static str {
        fact(key).unwrap()
    }

    #[test]
    fn the_labels_are_written_from_the_numbers() {
        assert_eq!(written(TURN_POINT), "3 + 2i");
        assert_eq!(written(TURN_ADDED), "1 - i");
        assert_eq!(written(TURN_BY), "1 + i");
        assert_eq!(written(C::new(0.0, 2.0)), "2i");
        assert_eq!(written(-ONE), "-1");
        assert_eq!(written(C::real(TURN_POINT.re)), "3");
        let sum: Vec<f64> = get("turn.sum").split(',').map(|x| x.parse().unwrap()).collect();
        assert_eq!(written(C::new(sum[0], sum[1])), "4 + i");
        assert_eq!(get("turn.square.value"), "0,2");
    }

    #[test]
    fn the_multiplications_end_where_the_facts_say() {
        assert!((partway(I, 1.0) - I).abs() < 1e-15);
        assert!((partway(TURN_BY, 0.0) - ONE).abs() < 1e-15);
        let twice = TURN_BY * partway(TURN_BY, 1.0);
        assert!((twice - C::new(0.0, 2.0)).abs() < 1e-14);
        assert!((TURN_BY.abs() - get("turn.one_plus_i.length").parse::<f64>().unwrap()).abs() < 5e-4);
        assert_eq!(get("turn.one_plus_i.angle_deg"), "45");
        assert_eq!((get("turn.square.angle_deg"), get("turn.square.length")), ("90", "2"));
        // halfway along, the stretch and the turn are both half done
        let half = partway(TURN_BY, 0.5);
        assert!((half.abs() - 2f64.sqrt().sqrt()).abs() < 1e-12 && (half.arg() - PI / 8.0).abs() < 1e-12);
    }

    #[test]
    fn everything_stays_in_the_plane_and_above_the_lower_third() {
        let tp = turn_plane();
        let (lines, glyph) = sheet();
        let all: Vec<C> = lines.iter().flatten().chain(glyph.iter().flatten()).copied().collect();
        for m in [ONE, I, TURN_BY, TURN_BY * TURN_BY] {
            for &z in &all {
                assert!(tp.contains(z * m), "{z:?} × {m:?}");
            }
        }
        for z in [TURN_POINT, TURN_ADDED, TURN_POINT + TURN_ADDED] {
            assert!(tp.contains(z));
        }
        assert!(tp.frame.bottom() > (-4.0 + 0.2 + 0.66) && tp.frame.left() > -6.96);
        assert!((tp.unit() - 1.0).abs() < 1e-12);
    }

    /// The whole outline runs from one point to the other, so "from zero" and "tip to tail"
    /// are what is drawn. Read from the real arrow's outline.
    #[test]
    fn the_arrows_reach_their_points() {
        let tp = turn_plane();
        for (a, b) in [
            (ZERO, TURN_POINT),
            (TURN_POINT, TURN_POINT + TURN_ADDED),
            (ZERO, TURN_BY),
            (ZERO, TURN_BY * TURN_BY),
        ] {
            let (pa, pb) = (tp.p(a), tp.p(b));
            let (dx, dy) = (pb[0] - pa[0], pb[1] - pa[1]);
            let len = (dx * dx + dy * dy).sqrt();
            let m = reaching_arrow(pa, pb, input()).unwrap();
            let along: Vec<f64> = m
                .points()
                .iter()
                .map(|q| ((q[0] - pa[0]) * dx + (q[1] - pa[1]) * dy) / len)
                .collect();
            let lo = along.iter().copied().fold(f64::MAX, f64::min);
            let hi = along.iter().copied().fold(f64::MIN, f64::max);
            assert!(
                lo.abs() < 1e-6 && (hi - len).abs() < 1e-6,
                "{a:?} to {b:?}: {lo} to {hi} of {len}"
            );
        }
    }

    /// "i" and "−1" stand off the axes their points lie on and off the walker's circle.
    #[test]
    fn the_names_of_i_and_minus_one_stand_off_the_axes() {
        let tp = turn_plane();
        const { assert!(TURN_NAMES_AT[0].re.abs() > 0.2 && TURN_NAMES_AT[1].im.abs() > 0.2) };
        for z in TURN_NAMES_AT {
            assert!(z.abs() > 1.2 && tp.contains(z), "{z:?}");
        }
    }

    /// Whenever the grid rests, none of its lines or the glyph's strokes run through the two
    /// names. The half sizes are CMR10 widths at 26, an estimate: the pictures decide.
    #[test]
    fn the_arrow_names_are_clear_of_the_grid_at_rest() {
        let (lines, glyph) = sheet();
        let mut strokes: Vec<(C, C)> = lines.iter().map(|l| (l[0], l[1])).collect();
        for g in &glyph {
            strokes.extend(g.windows(2).map(|w| (w[0], w[1])));
        }
        let clear = |at: C, m: C| {
            strokes.iter().all(|&(a, b)| {
                (0..=200).all(|k| {
                    let z = (a + (b - a).scale(f64::from(k) / 200.0)) * m;
                    (z.re - at.re).abs() > 0.3 || (z.im - at.im).abs() > 0.16
                })
            })
        };
        for m in [ONE, TURN_BY, TURN_BY * TURN_BY] {
            assert!(clear(W_NAME_AT, m), "1 + i at {m:?}");
        }
        assert!(clear(W2_NAME_AT, TURN_BY * TURN_BY));
        assert!(turn_plane().contains(W2_NAME_AT));
    }

    /// The source between two markers, with the tests left off. Read from the source because
    /// no render can run here; what it does to the pixels is for the pictures to show.
    fn between<'a>(source: &'a str, from: &str, to: &str) -> &'a str {
        let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
        let start = code.find(from).unwrap_or_else(|| panic!("{from}"));
        let end = start + code[start..].find(to).unwrap_or_else(|| panic!("{to}"));
        &code[start..end]
    }

    /// 02_5 acts out the eighth of a turn the voice recalls, using only the angle already on
    /// screen: the answer to its question stays hidden until the question's window is over.
    #[test]
    fn the_recalled_turn_moves_and_does_not_answer_the_question() {
        let beat = between(
            include_str!("turning.rs"),
            "say(&kit, stage, \"02_5\")",
            "let asked = predict(",
        );
        let hook = beat.find("line.wait_for(stage, \"an eighth of a turn\")?;").unwrap();
        let sweep = beat
            .find("sweep_arc(stage, tp, ANGLE_ARC_R, TURN_BY.arg(), SWEEP_S)?;")
            .unwrap();
        assert!(hook < sweep);
        for answer in ["TURN_BY * TURN_BY", "grid.multiply", "w2_arrow", "total_", "SQUARE_TEX"] {
            assert!(!beat.contains(answer), "{answer} answers the question early");
        }
        // the arc swept is the arc drawn, and it is the 45 degrees the facts state
        assert_eq!(get("turn.one_plus_i.angle_deg"), "45");
        assert!((TURN_BY.arg() - PI / 4.0).abs() < 1e-12);
        assert!(turn_plane().contains(C::polar(ANGLE_ARC_R, TURN_BY.arg())));
        // a beat, not a scene: the fixed part stays short so a brisk take still reaches the
        // question's card on its words (`predict` waits for the last sentence, not for this)
        const { assert!(SWEEP_S + PULSE.1 + PULSE.0 + PULSE.1 < 2.0) };
    }

    /// 02_6 stops being a still picture: the rule is read off the rows it names, and the two
    /// arrows stand alone for the last sentence. Nothing that leaves here fades twice.
    #[test]
    fn the_last_beat_reads_the_rule_off_the_rows_and_ends_on_the_arrows() {
        let source = include_str!("turning.rs");
        let beat = between(source, "say(&kit, stage, \"02_6\")", "kit.finish(stage)");
        let rule = beat.find("fade_in_all(stage, &[rule], 0.4)?;").unwrap();
        let lengths = beat.find("underline_pulse(stage, &[length, total_stretch])?;").unwrap();
        let hook = beat.find("line.wait_for(stage, \"and angles add\")?;").unwrap();
        let angles = beat
            .find("underline_pulse(stage, &[angle_says, total_turn])?;")
            .unwrap();
        let last = beat
            .find("line.wait_for(stage, \"Every complex derivative\")?;")
            .unwrap();
        let cleared = beat.find("fade_out_all(stage, &grid.mobs(), 0.5)?;").unwrap();
        let closing = beat.find("kit.music_breath(stage, 2.0)?;").unwrap();
        assert!(rule < lengths && lengths < hook && hook < angles && angles < last);
        assert!(last < cleared && cleared < closing);
        let ending = &beat[closing..];
        for twice in ["grid.mobs()", "stretch_turn"] {
            assert!(!ending.contains(twice), "{twice} fades twice");
        }
        assert!(ending.contains("w_arrow") && ending.contains("w2_arrow"));
        // the caption from 02_4 goes as the answer starts
        let head = &beat[..beat
            .find("let rt = line.until(stage, \"So one plus i\", 2.5)?;")
            .unwrap()];
        assert!(head.contains("gone.extend(stretch_turn);") && head.contains("fade_out_all(stage, &gone, 0.3)?;"));
    }

    /// The sentence each hook names begins at this byte of the cue.
    fn sentence_bytes(text: &str) -> Vec<usize> {
        let mut out = vec![0];
        for (i, c) in text.char_indices() {
            if matches!(c, '.' | '?' | '!') && text[i + 1..].starts_with(' ') {
                out.push(i + 2);
            }
        }
        out
    }

    /// Every phrase and sentence hook in chapters 02 to 05 is one the voice really says, and
    /// they run in the order it says them. A phrase the cue lacks fails the render; hooks out
    /// of order land a picture on words already gone. Hooks read through a variable are left
    /// to the tests beside them. No hook is a time: every moment is read from the words.
    #[test]
    fn every_hook_is_said_and_they_run_in_the_voices_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for source in [
            include_str!("turning.rs"),
            include_str!("two_planes.rs"),
            include_str!("reading_color.rs"),
            include_str!("gallery.rs"),
        ] {
            let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
            let says: Vec<usize> = code
                .match_indices("let line = say(&kit, stage, \"")
                .map(|(at, _)| at)
                .collect();
            assert!(!says.is_empty());
            for (k, &at) in says.iter().enumerate() {
                let end = says.get(k + 1).copied().unwrap_or(code.len());
                let body = &code[at..end];
                let id = body.split('"').nth(1).unwrap();
                let text = script.text(id).unwrap_or_else(|| panic!("{id}"));
                let starts = sentence_bytes(text);
                let mut hooks: Vec<(usize, usize)> = Vec::new();
                for form in ["line.wait_for(stage, \"", "line.until(stage, \""] {
                    for (place, _) in body.match_indices(form) {
                        let phrase = body[place + form.len()..].split('"').next().unwrap();
                        let byte = text.find(phrase).unwrap_or_else(|| panic!("{id}: {phrase}"));
                        hooks.push((place, byte));
                    }
                }
                let form = "line.wait_sentence(stage, ";
                for (place, _) in body.match_indices(form) {
                    let n = body[place + form.len()..].split(')').next().unwrap();
                    if let Ok(n) = n.parse::<usize>() {
                        let byte = *starts.get(n).unwrap_or_else(|| panic!("{id}: sentence {n}"));
                        hooks.push((place, byte));
                    }
                }
                hooks.sort_unstable();
                for pair in hooks.windows(2) {
                    assert!(pair[0].1 <= pair[1].1, "{id}: hooks out of the voice's order");
                }
            }
        }
    }
}
