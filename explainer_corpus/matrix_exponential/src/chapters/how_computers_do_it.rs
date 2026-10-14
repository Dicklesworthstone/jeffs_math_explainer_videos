//! Chapter 13, how computers do it: surely a computer adds up the series.
//! Try the matrix from Moler and Van Loan's "Nineteen Dubious Ways to
//! Compute the Exponential of a Matrix" (13_1). Its terms swell past nine
//! million with alternating signs, and a computer that keeps about seven
//! digits throws away everything after the decimal point: can anything
//! survive (13_2)? No: every entry comes out wrong, one with the wrong sign
//! (13_3). The cure is tiny steps again: half the time, run twice, is the
//! whole time, so halve the matrix eight times, sum a short series, and
//! square the result eight times (13_4). Real programs do exactly that, with
//! a ratio of two polynomials for the short series; and when a matrix
//! untangles into separate rates, that is the eigenvalue video (13_5).
//!
//! Every number is a fact: the matrix `F-C.mvl.A`, its terms `F-C.mvl.term.*`,
//! the true entries `F-V.13_2.exact`, this video's own 32-bit run
//! `F-V.13_3.series` (stopped at `F-V.13_3.stopped_at_k`), the cure
//! `F-V.13_4.scaled` with `F-C.mvl.f32.scaled.*`, the sweep
//! `F-C.mvl.f32.halvings.*`, the Padé numbers `F-V.13_5.*`.

use explainer_kit::props::lower_third;
use explainer_kit::{Frame2, Kit, Palette, fade_all, thousands, v};
use fmn::prelude::*;

use super::draw::{
    MatrixGrid, chip, entries, fact, fade_in_all, fade_out_all, faded_line, flat, good, matrix, minus, music_hold,
    numbers, panel, predict_hold, result, rule, say, show, substance, turning, warn,
};
use super::flow::{fade_in_anim, fade_out_anim};
use super::walks::cell_outline;
use crate::facts::{cited, mvl, shown};
use crate::model::{M2f, Mat, factorial, pade_at_one, series};

// ------------------------------------------------------------------ choices

/// How many terms the bars of 13_2 show (k = 0 up to this, inclusive).
const BAR_TERMS: usize = 36;
/// The plot of the bars: its centre, size and the largest height it spans.
const PLOT_AT: Vec3 = [-1.4, 0.2, 0.0];
const PLOT_W: f64 = 8.6;
const PLOT_H: f64 = 3.5;
/// 13_5's closing pair, the tanks' two rates and the eigen formula under
/// them: the lower third of the citation stands beneath, its box topping out
/// at -3.14 (with the formula at -2.8 its foot stood 0.14 over it: the dense
/// read of the scored draft).
const CLOSER_Y: f64 = -0.8;
const RATES_Y: f64 = -1.6;
const LETTERS_Y: f64 = -2.55;

// ------------------------------------------------------------- the geometry

/// The largest entry of term k (by size, with its sign) of the series for
/// `a`, for k = 0..=kmax.
pub fn term_peaks(a: &Mat, kmax: u32) -> Vec<f64> {
    series(a, kmax)
        .iter()
        .map(|s| {
            s.term
                .entries()
                .iter()
                .copied()
                .max_by(|x, y| x.abs().total_cmp(&y.abs()))
                .unwrap_or(0.0)
        })
        .collect()
}

/// The 32-bit product of two 2 by 2 matrices (each entry two products and a
/// sum, every operation rounded to 32 bits, as the model does).
fn mul32(a: &M2f, b: &M2f) -> M2f {
    let mut out = [[0.0_f32; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    out
}

/// Scaling and squaring in 32 bits, showing its work: the matrix after each
/// of the `halvings` squarings (the first is the short series of the
/// halved matrix, summed through `degree` terms; the last is the answer).
pub fn squaring_ladder(a: &M2f, halvings: u32, degree: u32) -> Vec<M2f> {
    let scale = 0.5_f32.powi(i32::try_from(halvings).expect("a small count"));
    let x: M2f = [[a[0][0] * scale, a[0][1] * scale], [a[1][0] * scale, a[1][1] * scale]];
    let mut term: M2f = [[1.0, 0.0], [0.0, 1.0]];
    let mut total = term;
    for k in 1..=degree {
        let product = mul32(&term, &x);
        for i in 0..2 {
            for j in 0..2 {
                term[i][j] = product[i][j] / k as f32;
                total[i][j] += term[i][j];
            }
        }
    }
    let mut ladder = vec![total];
    for _ in 0..halvings {
        total = mul32(&total, &total);
        ladder.push(total);
    }
    ladder
}

/// The entries of `wrong` whose sign is not the sign of the same entry of
/// `exact`: (row, column).
pub fn wrong_signs(wrong: &Mat, exact: &Mat) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for i in 0..wrong.n() {
        for j in 0..wrong.n() {
            if wrong.get(i, j).signum() != exact.get(i, j).signum() {
                out.push((i, j));
            }
        }
    }
    out
}

/// The coefficients (lowest power first) of the numerator of the [m/m] Padé
/// approximant of e^x, up to the common factor that makes them whole
/// numbers: (2m − j)! / (j! (m − j)!). The denominator is the same with
/// every odd power negated.
pub fn pade_numerator(m: u32) -> Vec<u128> {
    (0..=m)
        .map(|j| factorial(2 * m - j) / (factorial(j) * factorial(m - j)))
        .collect()
}

/// The numerator as TeX: `120+60x+12x^2+x^3`; `negate_odd` flips the odd
/// powers' signs (the denominator).
pub fn pade_tex(c: &[u128], negate_odd: bool) -> String {
    let mut s = String::new();
    for (j, &k) in c.iter().enumerate() {
        let minus = negate_odd && j % 2 == 1;
        let sign = match (s.is_empty(), minus) {
            (true, false) => "",
            (true, true) => "-",
            (false, false) => "+",
            (false, true) => "-",
        };
        let coeff = if k == 1 && j > 0 { String::new() } else { k.to_string() };
        let power = match j {
            0 => String::new(),
            1 => "x".to_owned(),
            _ => format!("x^{j}"),
        };
        s.push_str(&format!("{sign}{coeff}{power}"));
    }
    s
}

/// How many decimal digits of a value of size `scale` an answer that is off
/// by `error` still has right (rounded): the "two digits" of 13_4's strip.
pub fn digits_right(error: f64, scale: f64) -> u32 {
    (-(error / scale).log10()).round().max(0.0) as u32
}

fn mvl_f32() -> M2f {
    let a = cited::MVL_MATRIX;
    [[a[0][0] as f32, a[0][1] as f32], [a[1][0] as f32, a[1][1] as f32]]
}

// -------------------------------------------------------------- the scene

pub struct HowComputersDoIt {
    pub kit: Kit,
}

/// The largest entry as the card prints it: the fact's own digits, with
/// thousands separators in the whole part (`9303002.5` → `9,303,002.5`).
pub fn largest_entry_text(fact_value: &str) -> fmn::Result<String> {
    let (whole, frac) = fact_value.split_once('.').unwrap_or((fact_value, ""));
    let whole: u64 = whole
        .parse()
        .map_err(|e| super::draw::fail(format!("the largest term {fact_value:?}: {e}")))?;
    Ok(if frac.is_empty() {
        thousands(whole)
    } else {
        format!("{}.{frac}", thousands(whole))
    })
}

/// Rows of a matrix fact, as strings to `places` decimals.
fn rows3(key: &str) -> fmn::Result<Vec<Vec<String>>> {
    entries(key, Some(3))
}

/// 13_3's red box round each entry of the 32-bit run (`ours`) whose sign is
/// not the true entry's: `walks::cell_outline`, as wide as the number it
/// rings (a square cut through the digits of -0.839).
fn wrong_sign_rings(stage: &mut Stage<'_>, ours: &MatrixGrid) -> fmn::Result<Vec<Mob>> {
    let exact_m = matrix("F-V.13_2.exact")?;
    let ours_m = matrix("F-V.13_3.series")?;
    let wrong = wrong_signs(
        &Mat::from_rows(&ours_m.iter().map(|r| r.as_slice()).collect::<Vec<_>>()),
        &Mat::from_rows(&exact_m.iter().map(|r| r.as_slice()).collect::<Vec<_>>()),
    );
    let mut rings = Vec::new();
    for &(r, c) in &wrong {
        rings.push(cell_outline(stage, ours, r, c, warn())?);
    }
    Ok(rings)
}

/// 13_5's citation for the ratio of polynomials (a one-line lower third).
fn who_and_when(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    lower_third(
        kit,
        stage,
        "who and when:",
        &format!(
            "MATLAB and SciPy: Higham {}; Al-Mohy and Higham {}",
            cited::HIGHAM_YEAR,
            cited::AL_MOHY_HIGHAM_YEAR
        ),
    )
}

/// 13_5's pointer to the eigenvalue video: the tanks' two rates on a chip,
/// and the formula that untangles a matrix into separate rates under it.
fn eigen_pointer(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob)> {
    let rates = numbers("F-T.eigenvalues")?;
    let pointer = chip(
        kit,
        stage,
        &format!(
            "the tanks\u{2019} two rates: {} and {} a minute",
            super::draw::whole(rates[1]),
            minus(&show(rates[0], 1)?)
        ),
        28.0,
        substance(),
        v(0.0, RATES_Y),
    )?;
    let letters = kit.tex(
        stage,
        r"e^{tA}=V\,e^{t\Lambda}\,V^{-1}",
        50.0,
        Palette::ink(),
        &[("e", result()), ("tA", rule()), ("t\\Lambda", rule())],
        v(0.0, LETTERS_Y),
    )?;
    Ok((pointer, letters))
}

impl SceneConstruct for HowComputersDoIt {
    fn name(&self) -> &str {
        "13_how_computers_do_it"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, teal, gold) = (Palette::ink(), rule(), result());
        let a = mvl();

        // ---- 13_1: the paper and its matrix
        let line = say(&kit, stage, "13_1")?;
        let a_rows = entries("F-C.mvl.A", None)?;
        let a_grid = MatrixGrid::build(&kit, stage, &a_rows, 64.0, teal, v(-3.7, 0.5))?;
        let a_name = kit.tex(stage, "A=", 64.0, teal, &[], v(-6.0, 0.5))?;
        fade_in_all(stage, &[a_grid.group, a_name], 0.7)?;
        line.wait_for(stage, "from a paper by Moler and Van Loan")?;
        let card = Rectangle::new()
            .width(6.7)
            .height(3.1)
            .corner_radius(0.15)
            .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(v(3.3, 0.5));
        let card = stage.arena_mut().add(card);
        let title_a = kit.text(stage, "Nineteen Dubious Ways to", 34.0, ink, v(3.3, 1.35))?;
        let title_b = kit.text(stage, "Compute the Exponential of a Matrix", 34.0, ink, v(3.3, 0.8))?;
        let authors = kit.text(
            stage,
            "Cleve Moler and Charles Van Loan",
            28.0,
            Palette::dim(),
            v(3.3, 0.0),
        )?;
        let venue = kit.text(
            stage,
            &format!("SIAM Review, {}", cited::MVL_YEAR),
            28.0,
            Palette::dim(),
            v(3.3, -0.55),
        )?;
        fade_in_all(stage, &[card, title_a, title_b, authors, venue], 0.8)?;
        kit.hold(stage)?;

        // ---- 13_2: the terms swell; can anything after the point survive?
        let line = say(&kit, stage, "13_2")?;
        let mut gone = vec![a_grid.group, a_name, card, title_a, title_b, authors, venue];
        fade_out_all(stage, &gone, 0.5)?;
        gone.clear();
        let peaks = term_peaks(&a, BAR_TERMS as u32);
        let top = peaks.iter().fold(0.0_f64, |m, p| m.max(p.abs()));
        let frame = Frame2 {
            center: PLOT_AT,
            width: PLOT_W,
            height: PLOT_H,
            x: (-0.5, BAR_TERMS as f64 + 0.5),
            y: (0.0, top * 1.05),
        };
        let base = frame.bottom();
        let mut bars = Vec::new();
        for (k, &p) in peaks.iter().enumerate() {
            let x = frame.p(k as f64, 0.0)[0];
            let h = (frame.p(0.0, p.abs())[1] - base).max(0.01);
            let color = if p >= 0.0 { teal } else { turning() };
            let r = Rectangle::new()
                .width(PLOT_W / (BAR_TERMS as f64 + 1.0) * 0.8)
                .height(h)
                .style(flat(color, 0.9))
                .build()?
                .moved_to(v(x, base + h / 2.0));
            bars.push(stage.arena_mut().add(r));
        }
        let floor = polyline(vec![v(frame.left(), base), v(frame.right(), base)])
            .style(faded_line(Palette::dim(), 2.5, 1.0))
            .build();
        let floor = stage.arena_mut().add(floor);
        let mut ticks = vec![floor];
        for k in (0..=BAR_TERMS).step_by(6) {
            let x = frame.p(k as f64, 0.0)[0];
            ticks.push(kit.text(stage, &k.to_string(), 26.0, Palette::dim(), v(x, base - 0.35))?);
        }
        ticks.push(kit.text(
            stage,
            "k, the term",
            26.0,
            Palette::dim(),
            v(frame.right() - 0.9, base - 0.78),
        )?);
        fade_in_all(stage, &ticks, 0.5)?;
        line.wait_for(stage, "Its terms swell past nine million")?;
        let mut grow: Vec<Box<dyn Animation>> = Vec::new();
        for &b in &bars {
            let g = fmn::animation::grow_from_edge(stage.arena_mut(), b, DOWN, None)?;
            grow.push(stage.prepare(g)?);
        }
        let swell = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, 0.18)?.with_run_time(3.4);
        stage.play(swell)?;
        // the largest entry on one card, with the digits a computer keeps
        let largest = largest_entry_text(fact("F-C.mvl.term.largest")?)?;
        let digits = (f64::from(f32::MANTISSA_DIGITS) * 2.0_f64.log10()).round() as u32;
        let card2 = Rectangle::new()
            .width(3.6)
            .height(1.9)
            .corner_radius(0.15)
            .style(Style::default().fill(panel(), 0.96).stroke(Palette::faint(), 2.0, 1.0))
            .build()?
            .moved_to(v(5.0, 1.55));
        let card2 = stage.arena_mut().add(card2);
        let big = kit.text(stage, &largest, 44.0, gold, v(5.0, 1.9))?;
        let kept = kit.text(
            stage,
            &format!("about {digits} digits"),
            30.0,
            Palette::dim(),
            v(5.0, 1.1),
        )?;
        fade_in_all(stage, &[card2, big, kept], 0.6)?;
        line.wait_for(stage, "Can anything after the decimal point survive")?;
        // the true entries, far below every bar: a hairline and a grid
        let hair = polyline(vec![v(frame.left(), base + 0.02), v(frame.right(), base + 0.02)])
            .style(faded_line(gold, 2.5, 1.0))
            .build();
        let hair = stage.arena_mut().add(hair);
        let exact_grid = MatrixGrid::build(&kit, stage, &rows3("F-V.13_2.exact")?, 44.0, gold, v(5.0, -1.2))?;
        let exact_name = kit.text(stage, "the true entries", 26.0, Palette::dim(), v(5.0, -2.35))?;
        fade_in_all(stage, &[hair, exact_grid.group, exact_name], 0.6)?;
        predict_hold(&kit, stage)?;

        // ---- 13_3: no: every entry is wrong, one with the wrong sign
        let line = say(&kit, stage, "13_3")?;
        let mut leave = bars.clone();
        leave.extend(&ticks);
        leave.extend([card2, big, kept, hair, exact_grid.group, exact_name]);
        fade_out_all(stage, &leave, 0.5)?;
        let exact_rows = rows3("F-V.13_2.exact")?;
        let ours_rows = rows3("F-V.13_3.series")?;
        let truth = MatrixGrid::build(&kit, stage, &exact_rows, 60.0, gold, v(-3.4, 0.7))?;
        let ours = MatrixGrid::build(&kit, stage, &ours_rows, 60.0, warn(), v(3.4, 0.7))?;
        let truth_name = kit.text(stage, "the true answer", 28.0, Palette::dim(), v(-3.4, -0.6))?;
        let ours_name = kit.text(stage, "the series, in 32 bits", 28.0, Palette::dim(), v(3.4, -0.6))?;
        fade_in_all(stage, &[truth.group, truth_name], 0.5)?;
        line.wait_for(stage, "Every entry comes out wrong")?;
        fade_in_all(stage, &[ours.group, ours_name], 0.6)?;
        line.wait_for(stage, "one with the wrong sign")?;
        let rings = wrong_sign_rings(stage, &ours)?;
        fade_in_all(stage, &rings, 0.4)?;
        // the note comes with "The mathematics is fine", two sentences from
        // the end, so it has its seconds inside this line and is gone
        // before 13_4's picture (it came with the last sentence, stood
        // through 13_4's first and went at its second, stale under the new
        // picture)
        line.wait_for(stage, "The mathematics is fine")?;
        let stopped = fact("F-V.13_3.stopped_at_k")?;
        let wide = format!(
            "this video\u{2019}s own run: 32-bit, term \u{d7} A then \u{f7} k, stopped at k = {stopped}; \
             catastrophic cancellation. In 64-bit this matrix survives; doubled, it fails again"
        );
        let programmers = lower_third(&kit, stage, "for programmers:", &wide)?;
        #[cfg(test)]
        super::draw::watch_dwell(&kit, stage, programmers, "13_3:programmers")?;
        fade_in_all(stage, &[programmers], 0.5)?;
        kit.hold(stage)?;

        // ---- 13_4: halve, sum a short series, square back
        let line = say(&kit, stage, "13_4")?;
        // (the programmers' note of 13_3 goes with the pictures it belongs
        // to, before 13_4's own comes in)
        let mut clear = vec![truth.group, ours.group, truth_name, ours_name, programmers];
        clear.extend(&rings);
        fade_out_all(stage, &clear, 0.5)?;
        let halvings = fact("F-C.mvl.f32.scaled.halvings")?
            .parse::<u32>()
            .map_err(|e| super::draw::fail(format!("halvings: {e}")))?;
        let degree = fact("F-C.mvl.f32.scaled.terms")?
            .parse::<u32>()
            .map_err(|e| super::draw::fail(format!("terms: {e}")))?;
        // "Half the time, run twice, is the whole time": the rule of thumb is
        // the picture for those two sentences, large in the middle, and it
        // goes up to the top edge when the ladder of halvings comes in
        let rule_src = r"e^{A}=\left(e^{A/2}\right)^{2}";
        let rule_colors = [("e", gold), ("A", teal)];
        let big_rule = kit.tex(stage, rule_src, 90.0, ink, &rule_colors, v(0.0, 0.4))?;
        let rule_of_thumb = kit.tex(stage, rule_src, 54.0, ink, &rule_colors, v(0.0, 3.0))?;
        fade_in_all(stage, &[big_rule], 0.6)?;
        line.wait_for(stage, "So halve the matrix eight times")?;
        let to_top = vec![
            fade_out_anim(stage, big_rule, 0.4)?,
            fade_in_anim(stage, rule_of_thumb, 0.5)?,
        ];
        stage.play_prepared(to_top)?;
        // a ladder of boxes: A, A/2, ... A/256, each with its size
        let n_boxes = halvings as usize + 1;
        let pitch = 1.35;
        let left = -(n_boxes as f64 - 1.0) * pitch / 2.0;
        let size0: f64 = fact("F-C.mvl.size")?
            .parse()
            .map_err(|e| super::draw::fail(format!("F-C.mvl.size: {e}")))?;
        let mut boxes = Vec::new();
        let mut names = Vec::new();
        let mut sizes = Vec::new();
        for j in 0..n_boxes {
            let x = left + pitch * j as f64;
            let b = Rectangle::new()
                .width(pitch * 0.9)
                .height(0.9)
                .corner_radius(0.1)
                .style(Style::default().fill(panel(), 0.96).stroke(teal, 3.0, 1.0))
                .build()?
                .moved_to(v(x, 1.7));
            boxes.push(stage.arena_mut().add(b));
            let label = if j == 0 {
                "A".to_owned()
            } else {
                format!("A/{}", 1_u64 << j)
            };
            names.push(kit.text(stage, &label, 28.0, ink, v(x, 1.7))?);
            let size = size0 / f64::from(1_u32 << j);
            sizes.push(kit.text(stage, &format!("{:.2}", size), 24.0, Palette::dim(), v(x, 1.0))?);
        }
        let size_label = kit.text_left(
            stage,
            "its size, under each:",
            24.0,
            Palette::dim(),
            v(left - 0.6, 0.45),
        )?;
        let mut halving: Vec<Box<dyn Animation>> = Vec::new();
        for ((&b, &n), &s) in boxes.iter().zip(&names).zip(&sizes) {
            halving.push(fade_in_anim(stage, b, 0.5)?);
            halving.push(fade_in_anim(stage, n, 0.5)?);
            halving.push(fade_in_anim(stage, s, 0.5)?);
        }
        halving.push(fade_in_anim(stage, size_label, 0.5)?);
        let lagged = AnimationGroup::with_lag_ratio(stage.arena_mut(), halving, 0.12)?.with_run_time(2.6);
        stage.play(lagged)?;
        line.wait_for(stage, "until the series is short and tame")?;
        // the series of the small matrix: a few terms, and they are gone
        let small = Mat::from_rows(&[
            &[mvl().get(0, 0) / 256.0, mvl().get(0, 1) / 256.0],
            &[mvl().get(1, 0) / 256.0, mvl().get(1, 1) / 256.0],
        ]);
        let short = term_peaks(&small, degree);
        let short_top = short.iter().fold(0.0_f64, |m, p| m.max(p.abs()));
        let mut short_bars = Vec::new();
        for (k, &p) in short.iter().enumerate() {
            let h = 0.12 + 0.7 * (p.abs() / short_top).sqrt() * (if k == 0 { 1.0 } else { 0.7 });
            let r = Rectangle::new()
                .width(0.34)
                .height(h)
                .style(flat(teal, 0.85))
                .build()?
                .moved_to(v(left + 0.4 * k as f64, -0.1 + h / 2.0 - 0.35));
            short_bars.push(stage.arena_mut().add(r));
        }
        let short_note = kit.text(
            stage,
            &format!("a series through the {}th power", degree),
            26.0,
            Palette::dim(),
            v(left + 3.3, -0.8),
        )?;
        let mut short_in = short_bars.clone();
        short_in.push(short_note);
        fade_in_all(stage, &short_in, 0.5)?;
        line.wait_for(stage, "Then square the result, eight times")?;
        // squared back up: the top left entry after each squaring
        let ladder = squaring_ladder(&mvl_f32(), halvings, degree);
        let mut squared = Vec::new();
        for (j, m) in ladder.iter().enumerate() {
            let x = left + pitch * j as f64;
            let first = f64::from(m[0][0]);
            let c = if j + 1 == ladder.len() { good() } else { ink };
            squared.push(kit.text(stage, &minus(&show(first, 3)?), 28.0, c, v(x, -2.1))?);
        }
        let squared_label = kit.text(
            stage,
            "after each squaring, the top left entry:",
            26.0,
            Palette::dim(),
            v(0.0, -1.55),
        )?;
        let mut squares: Vec<Box<dyn Animation>> = vec![fade_in_anim(stage, squared_label, 0.4)?];
        for &s in &squared {
            squares.push(fade_in_anim(stage, s, 0.5)?);
        }
        let lagged = AnimationGroup::with_lag_ratio(stage.arena_mut(), squares, 0.25)?.with_run_time(2.8);
        stage.play(lagged)?;
        let done = kit.text(
            stage,
            &format!(
                "scaling and squaring: right to three places ({})",
                fact("F-V.13_4.right_to_three_places")?
            ),
            28.0,
            good(),
            v(0.0, -2.85),
        )?;
        fade_in_all(stage, &[done], 0.5)?;
        let sweep = *shown::HALVINGS_SWEEP.last().expect("a sweep");
        let worst: f64 = fact(&format!("F-C.mvl.f32.halvings.h{sweep}"))?
            .parse()
            .map_err(|e| super::draw::fail(format!("halvings sweep: {e}")))?;
        let digits_left = digits_right(worst, 1.5);
        let strip = lower_third(
            &kit,
            stage,
            "for programmers:",
            &format!("not too many halvings: {sweep} leave {digits_left} digits"),
        )?;
        fade_in_all(stage, &[strip], 0.5)?;
        kit.hold(stage)?;

        // ---- 13_5: a ratio of two polynomials; and eigenvalues point to K
        let line = say(&kit, stage, "13_5")?;
        let mut tidy = vec![rule_of_thumb, size_label, done, squared_label, strip];
        tidy.extend(&boxes);
        tidy.extend(&names);
        tidy.extend(&sizes);
        tidy.extend(&short_in);
        tidy.extend(&squared);
        let p = pade_numerator(3);
        let num = pade_tex(&p, false);
        let den = pade_tex(&p, true);
        // "Real programs do exactly that": the ladder is what they do, and
        // stays up for those words (the stage was cleared at the line's first
        // frame, so the sentence began over a black one)
        line.wait_for(stage, "with a ratio of two polynomials")?;
        fade_out_all(stage, &tidy, 0.5)?;
        let ratio = kit.tex(
            stage,
            &format!(r"e^{{x}}\approx\frac{{{num}}}{{{den}}}"),
            50.0,
            ink,
            &[("e", gold)],
            v(0.0, 2.3),
        )?;
        fade_in_all(stage, &[ratio], 0.6)?;
        let (pn, pd) = pade_at_one(3);
        let series_line = kit.text(
            stage,
            &format!(
                "series through the 6th power at x = 1: {}  (off by {})",
                fact("F-V.13_5.series")?,
                fact("F-C.series.k6.error")?
            ),
            30.0,
            ink,
            v(0.0, 0.7),
        )?;
        let pade_line = kit.text(
            stage,
            &format!(
                "ratio of two cubics: {pn}/{pd} = {}  (off by {})",
                fact("F-V.13_5.pade")?,
                fact("F-C.pade.m3.error")?
            ),
            30.0,
            gold,
            v(0.0, 0.0),
        )?;
        let closer = kit.text(
            stage,
            &format!("{} times closer", fact("F-V.13_5.times_closer")?),
            34.0,
            good(),
            v(0.0, CLOSER_Y),
        )?;
        fade_in_all(stage, &[series_line, pade_line, closer], 0.7)?;
        let who = who_and_when(&kit, stage)?;
        fade_in_all(stage, &[who], 0.5)?;
        line.wait_for(stage, "that's the eigenvalue video")?;
        let (pointer, letters) = eigen_pointer(&kit, stage)?;
        fade_in_all(stage, &[pointer, letters], 0.7)?;
        music_hold(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::super::draw::{quiz_log, testing};
    use super::*;
    use crate::model::{expm, pade_at_one, scale_square_f32, series_f32};

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

    /// 13_5 (the dense read of the scored draft): the eigen formula stands
    /// clear above the citation's lower third (it stood 0.14 over it), and
    /// clear of the chip of the two rates, which in turn is clear of the
    /// "times closer" line.
    #[test]
    fn the_eigen_formula_stands_clear_of_the_lower_third() {
        run(|stage| {
            let kit = Kit::new()?;
            let who = who_and_when(&kit, stage)?;
            let (pointer, letters) = eigen_pointer(&kit, stage)?;
            let closer = kit.text(stage, "8 times closer", 34.0, good(), v(0.0, CLOSER_Y))?;
            let band_top = stage.get_top(who)[1];
            let letters_foot = stage.get_bottom(letters)[1];
            assert!(
                letters_foot > band_top + 0.3,
                "the formula's foot {letters_foot:.2} against the lower third's top {band_top:.2}"
            );
            let (chip_foot, letters_top) = (stage.get_bottom(pointer)[1], stage.get_top(letters)[1]);
            assert!(
                chip_foot > letters_top + 0.2,
                "the chip's foot {chip_foot:.2} against the formula's top {letters_top:.2}"
            );
            let (closer_foot, chip_top) = (stage.get_bottom(closer)[1], stage.get_top(pointer)[1]);
            assert!(
                closer_foot > chip_top + 0.2,
                "the line's foot {closer_foot:.2} against the chip's top {chip_top:.2}"
            );
            // inside the frame sideways
            for m in [pointer, letters] {
                let (l, r) = (stage.get_left(m)[0], stage.get_right(m)[0]);
                assert!(l > -7.1 + 0.3 && r < 7.1 - 0.3, "{l:.2} to {r:.2}");
            }
            Ok(())
        });
    }

    /// 13_3 (the dense read of the scored draft): the red box on the one
    /// wrong-sign entry spans its whole number with room to spare (it was a
    /// square 0.9 of a row's pitch, 63 px on a number 115 px wide: its
    /// edges cut through the minus sign and the last digit), inside the
    /// grid's own brackets, and the box does ring the lower right entry.
    #[test]
    fn the_box_on_the_wrong_sign_spans_the_number() {
        run(|stage| {
            let kit = Kit::new()?;
            let ours = MatrixGrid::build(&kit, stage, &rows3("F-V.13_3.series")?, 60.0, warn(), v(3.4, 0.7))?;
            let rings = wrong_sign_rings(stage, &ours)?;
            assert_eq!(rings.len(), 1, "one wrong sign");
            let ring = rings[0];
            let cell = ours.cells[1][1];
            assert!(
                stage.get_center(cell)[1] < stage.get_center(ours.cells[0][1])[1],
                "the lower right entry"
            );
            let (cl, cr) = (stage.get_left(cell)[0], stage.get_right(cell)[0]);
            let (rl, rr) = (stage.get_left(ring)[0], stage.get_right(ring)[0]);
            assert!(
                rl < cl - 0.1 && rr > cr + 0.1,
                "the box spans {rl:.2} to {rr:.2}, the number {cl:.2} to {cr:.2}"
            );
            // the box stands inside the grid's brackets and clear of the
            // entry above and the one to the left
            let (bl, br) = (
                stage.get_left(ours.brackets[0])[0],
                stage.get_right(ours.brackets[1])[0],
            );
            assert!(
                rl > bl && rr < br,
                "the box {rl:.2} to {rr:.2} inside the brackets {bl:.2} to {br:.2}"
            );
            let left_cell = ours.cells[1][0];
            assert!(rl > stage.get_right(left_cell)[0], "clear of the entry to its left");
            let above = ours.cells[0][1];
            assert!(
                stage.get_top(ring)[1] <= stage.get_bottom(above)[1] + 0.05,
                "clear of the entry above"
            );
            Ok(())
        });
    }

    /// A 2 by 2 of 32-bit floats as a model matrix.
    fn widen(a: &M2f) -> Mat {
        Mat::from_rows(&[
            &[f64::from(a[0][0]), f64::from(a[0][1])],
            &[f64::from(a[1][0]), f64::from(a[1][1])],
        ])
    }

    fn rows_of(key: &str) -> Vec<Vec<f64>> {
        matrix(key).unwrap()
    }

    /// A polynomial with whole coefficients (lowest power first) at `x`.
    fn poly(c: &[u128], x: f64) -> f64 {
        c.iter().rev().fold(0.0, |acc, &k| acc * x + k as f64)
    }

    /// 13_2: the terms swell past nine million, with alternating signs, to
    /// the facts' largest entry at k = 16 and 17, then fall away.
    #[test]
    fn the_terms_swell_past_nine_million_with_alternating_signs() {
        let peaks = term_peaks(&mvl(), BAR_TERMS as u32);
        let largest: f64 = fact("F-C.mvl.term.largest").unwrap().parse().unwrap();
        let top = peaks.iter().fold(0.0_f64, |m, p| m.max(p.abs()));
        assert!((top - largest).abs() < 0.1, "{top} against {largest}");
        assert_eq!(
            (top / 1e6).floor() as u32,
            fact("F-V.13_2.millions").unwrap().parse::<u32>().unwrap()
        );
        let at_peak: Vec<usize> = peaks
            .iter()
            .enumerate()
            .filter(|(_, p)| (p.abs() - top).abs() < 1.0)
            .map(|(k, _)| k)
            .collect();
        assert_eq!(at_peak, [16, 17], "the largest entry is at k = 16 and 17");
        // from the first term the sign of the largest entry alternates
        assert!(peaks.windows(2).skip(1).all(|w| w[0].signum() != w[1].signum()));
        // and they fall away again: three orders down by the last bar
        assert!(peaks[BAR_TERMS].abs() < top / 1000.0);
        let far = term_peaks(&mvl(), 90);
        assert!(far[90].abs() < 1.0, "and far out they are tiny");
        // the true entries are all under two (the hairline's claim)
        let exact = expm(&mvl());
        assert!(exact.max_abs() < 2.0);
        assert_eq!(fact("F-V.13_2.all_smaller_than_two").unwrap(), "true");
        // a 32-bit float keeps about seven digits
        assert_eq!((f64::from(f32::MANTISSA_DIGITS) * 2.0_f64.log10()).round() as u32, 7);
    }

    /// 13_3: this video's own 32-bit run gets every entry wrong, one with
    /// the wrong sign, as the facts print it.
    #[test]
    fn the_series_in_thirty_two_bits_is_wrong_everywhere_and_once_in_sign() {
        let (wrong, stopped) = series_f32(&mvl_f32());
        let wrong = widen(&wrong);
        let exact = expm(&mvl());
        for (w, x) in wrong.entries().iter().zip(exact.entries()) {
            assert!((w - x).abs() > 0.05, "{w} against {x}");
        }
        let signs = wrong_signs(&wrong, &exact);
        assert_eq!(signs, [(1, 1)], "the one wrong sign is the lower right entry");
        assert_eq!(stopped.to_string(), fact("F-V.13_3.stopped_at_k").unwrap());
        let printed = rows_of("F-V.13_3.series");
        for (i, row) in printed.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                assert!((wrong.get(i, j) - x).abs() < 5e-4, "entry {i},{j}");
            }
        }
        // the programmers' strip: 64 bits survive, doubled fails again
        assert_eq!(fact("F-C.mvl.f64.right_to_six_places").unwrap(), "true");
        assert_eq!(fact("F-C.mvl.f64.doubled.entries_right_to_two_digits").unwrap(), "0");
    }

    /// 13_4: the ladder of squarings is the model's scaling and squaring,
    /// operation for operation, and ends right to three places in the same
    /// 32 bits; more halvings are not better.
    #[test]
    fn halving_and_squaring_in_the_same_bits_is_right_to_three_places() {
        let m = mvl_f32();
        let ladder = squaring_ladder(&m, 8, 8);
        assert_eq!(ladder.len(), 9, "the short series, then eight squarings");
        assert_eq!(*ladder.last().unwrap(), scale_square_f32(&m, 8, 8));
        let exact = expm(&mvl());
        let done = widen(ladder.last().unwrap());
        for (w, x) in done.entries().iter().zip(exact.entries()) {
            assert!((w - x).abs() < 5e-5, "{w} against {x}");
        }
        let printed = rows_of("F-V.13_4.scaled");
        for (i, row) in printed.iter().enumerate() {
            for (j, &x) in row.iter().enumerate() {
                assert!((done.get(i, j) - x).abs() < 5e-4);
            }
        }
        assert_eq!(fact("F-C.mvl.f32.scaled.divisor").unwrap(), "256");
        assert_eq!(1_u32 << 8, 256);
        // the starting matrix is small and its series short and tame
        assert!(mvl().norm_inf() / 256.0 < 0.5);
        // 4 and 8 halvings are best; more are worse (the sweep's facts)
        let errors: Vec<f64> = shown::HALVINGS_SWEEP
            .iter()
            .map(|h| fact(&format!("F-C.mvl.f32.halvings.h{h}")).unwrap().parse().unwrap())
            .collect();
        assert!(errors.windows(2).all(|w| w[1] > w[0]), "{errors:?}");
        // and twenty leave about two digits
        assert_eq!(digits_right(*errors.last().unwrap(), 1.5), 2);
    }

    /// 13_5: the ratio of two cubics is the facts' 193/71, agrees with e^x
    /// through the sixth power, and is eight times closer at x = 1 than the
    /// series through the sixth power.
    #[test]
    fn the_ratio_of_two_cubics_is_eight_times_closer() {
        let p = pade_numerator(3);
        assert_eq!(p, [120, 60, 12, 1]);
        assert_eq!(pade_tex(&p, false), "120+60x+12x^2+x^3");
        assert_eq!(pade_tex(&p, true), "120-60x+12x^2-x^3");
        let (pn, pd) = pade_at_one(3);
        assert_eq!((pn, pd), (193, 71));
        let (num, den) = (poly(&p, 1.0), poly(&p, -1.0));
        assert_eq!((num, den), (193.0, 71.0), "the denominator is the numerator at −x");
        assert_eq!(fact("F-C.pade.m3").unwrap(), "193/71");
        // p(x) − e^x p(−x) is of order x^7: tiny near zero
        for x in [0.05_f64, 0.1, 0.2] {
            let miss = poly(&p, x) - x.exp() * poly(&p, -x);
            assert!(miss.abs() < 1e-3 * x.powi(7) * 120.0 + 1e-15, "x = {x}: {miss}");
        }
        let e = std::f64::consts::E;
        let series_error = (crate::model::exp_partial(1.0, 6) - e).abs();
        let pade_error = (num / den - e).abs();
        assert_eq!((series_error / pade_error).round() as u32, 8);
        assert_eq!(fact("F-V.13_5.times_closer").unwrap(), "8");
        assert_eq!(show(num / den, 5).unwrap(), fact("F-V.13_5.pade").unwrap());
        assert_eq!(
            show(crate::model::exp_partial(1.0, 6), 5).unwrap(),
            fact("F-V.13_5.series").unwrap()
        );
        // the tanks' two rates, for the pointer to the eigenvalue video
        assert_eq!(fact("F-T.eigenvalues").unwrap(), "-0.200000,0.000000");
        assert_eq!(fact("F-C.diag.tanks.rebuilt").unwrap(), "true");
    }

    /// 13_3's programmers' note has time to be read and is gone before 13_4's
    /// own picture: it comes up inside 13_3 and stands fully shown for at
    /// least three seconds, and it goes as 13_4 begins, before the rule of
    /// thumb comes in (it stood through 13_4's first sentence, stale under the
    /// new picture, 2.8 s). Run on the scratch voice and on the finished
    /// read's own timing for 13_3 (`REAL_TIMING`).
    #[test]
    fn the_programmers_note_stands_three_seconds_and_is_gone_before_13_4s_picture() {
        for real in [false, true] {
            let voiced = if real {
                testing::run_voiced_real("13_how_computers_do_it")
            } else {
                testing::run_voiced("13_how_computers_do_it")
            }
            .unwrap();
            let events = quiz_log::take(&voiced.tag, "13_3:programmers");
            let when = |kind: &str| events.iter().find(|(k, _)| *k == kind).map(|e| e.1);
            let (up, down) = (
                when("up").expect("the note comes up"),
                when("down").expect("the note goes"),
            );
            let line_at = |id: &str| voiced.lines.iter().find(|(i, ..)| i == id).cloned().unwrap();
            let (_, start, len) = line_at("13_3");
            let (_, next, _) = line_at("13_4");
            assert!(
                up > start && up < start + len,
                "real {real}: up at {up:.1} s in a line from {start:.1} s to {:.1} s",
                start + len
            );
            assert!(
                down - up >= 3.0,
                "real {real}: it stands {:.1} s ({up:.1} to {down:.1})",
                down - up
            );
            // the clearing fade of 13_4 takes 0.5 s and the rule of thumb
            // comes in after it: the note is down by then
            assert!(
                down <= next + 0.6,
                "real {real}: it goes at {down:.1} s, 13_4 begins at {next:.1} s"
            );
        }
    }

    /// The matrix rows of a fact, as a model matrix.
    fn mat_of(key: &str) -> Mat {
        Mat::from_rows(&rows_of(key).iter().map(|r| r.as_slice()).collect::<Vec<_>>())
    }

    /// 13_3 against Moler and Van Loan (the mathematician audit): their IBM 370
    /// run, set against the e^A they print, has the wrong sign at two entries,
    /// the two of its right-hand column; this video's own 32-bit run has one.
    /// The description tells that story with the facts' own numbers around it
    /// (it once said three entries, and carried another run's digits).
    #[test]
    fn the_description_counts_the_papers_wrong_signs_and_prints_the_chapters_facts() {
        let paper = wrong_signs(&mat_of("F-B.mvl.ibm.result"), &mat_of("F-C.mvl.exact"));
        assert_eq!(
            paper,
            [(0, 1), (1, 1)],
            "the paper's run is wrong in sign in its right-hand column"
        );
        let ours = wrong_signs(&mat_of("F-C.mvl.f32.series"), &mat_of("F-C.mvl.exact"));
        assert_eq!(ours.len().to_string(), fact("F-C.mvl.f32.wrong_signs").unwrap());
        let count = |n: usize| ["no", "one", "two", "three", "four"][n];
        // in the facts' own digits, with the true minus, as the description sets a matrix
        let bracket = |key: &str| {
            format!(
                "[{}]",
                fact(key)
                    .unwrap()
                    .replace('-', "\u{2212}")
                    .replace(';', "; ")
                    .replace(',', ", ")
            )
        };
        let size: f64 = fact("F-C.mvl.size").unwrap().parse().unwrap();
        let want = [
            bracket("F-C.mvl.exact"),
            bracket("F-C.mvl.f32.series"),
            bracket("F-B.mvl.ibm.result"),
            bracket("F-C.mvl.f32.scaled"),
            format!("{} entries with the wrong sign", count(paper.len())),
            format!("{} entry with the wrong sign", count(ours.len())),
            format!("`k = {}`", fact("F-C.mvl.f32.terms").unwrap()),
            format!("`K = {}`", fact("F-B.mvl.ibm.terms").unwrap()),
            format!(
                "`{}`",
                largest_entry_text(fact("F-C.mvl.term.largest").unwrap()).unwrap()
            ),
            format!("worst error `{}`", fact("F-C.mvl.f32.scaled.worst_error").unwrap()),
            format!("`{}` times closer", fact("F-C.pade.m3.times_closer").unwrap()),
            format!("size `{:.2} < 1`", size / 256.0),
        ];
        // read the description on one line, so a re-wrapped paragraph still matches
        let text = include_str!("../../docs/description.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for w in &want {
            assert!(text.contains(w.as_str()), "the description never says {w:?}");
        }
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("13_how_computers_do_it").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        // 13_3 names the computation it shows: the run, its 32 bits, its loop and where it stopped
        let note = format!(
            "this video\u{2019}s own run: 32-bit, term \u{d7} A then \u{f7} k, stopped at k = {}",
            fact("F-V.13_3.stopped_at_k").unwrap()
        );
        for want in [
            note.as_str(),
            "the series, in 32 bits",
            "Nineteen Dubious Ways to",
            "9,303,002.5",
            "about 7 digits",
            "SIAM Review, 1978",
            "not too many halvings: 20 leave 2 digits",
            "e^{x}\\approx\\frac{120+60x+12x^2+x^3}{120-60x+12x^2-x^3}",
            "ratio of two cubics: 193/71 = 2.71831",
            "8 times closer",
            "MATLAB and SciPy: Higham 2005; Al-Mohy and Higham 2009",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        assert!(ran.seconds > 45.0 && ran.seconds < 85.0, "{} s", ran.seconds);
    }
}
