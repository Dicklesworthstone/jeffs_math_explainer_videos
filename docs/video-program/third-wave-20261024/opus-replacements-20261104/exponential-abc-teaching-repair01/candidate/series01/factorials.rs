//! Native staged coefficient-matching chapter. Type-checked at the pin; never rendered.
//!
//! Each cue has one function, in film order, and each function begins with its own `say`.
//! The polynomial stays on screen from 02_1 to the end of 02_4: its letters become factorials,
//! its terms light up with their curves, and its dots are the ones 02_4 refuses to trust.
use crate::series_model::{
    BAR_TOP, INSET_TOP, ORDERS, PLOT_SEGMENTS, PLOT_TOP, RADIUS, TAIL_ORDERS, ZOOM, coefficient, column_tex,
    derivative_label, envelope, equation_label, factorial_tex, halving_from, letter, plot_x, ratio_label,
    running_product_tex, sum, tail_bound, tail_label, term_size, value_label,
};
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Timed, ink_style, lagged, rect, stroke_style, tween, v};
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

pub struct Factorials {
    pub kit: Kit,
}

const TITLE: &str = "Where the factorials come from";
const QUESTION: &str = "What must a be?";
const CURVES_NOTE: &str = "partial sums on fixed axes";
const LOST_POWER: &str = "A finite polynomial loses its highest power";
const NEEDS_PROOF: &str = "The infinite tail needs a proof";
const SUM_EXISTS: &str = "Does the sum exist?";
const TERM_BY_TERM: &str = "May we differentiate term by term?";
const UNIFORM: &str = "Uniform convergence on every bounded interval";
const FIRST_DEBT: &str = "Why adding inputs multiplies outputs";
const SECOND_DEBT: &str = "Why the stepped approximations reach E";
const THIRD_DEBT: &str = "Why E is the only solution from height one";
/// Word for word from 02_4 and 02_6: these two sentences carry more than one reveal.
const OBLIGATIONS_SENTENCE: &str =
    "We need to know that the sum exists and that differentiating it term by term is legitimate.";
const DEBTS_SENTENCE: &str = "We still owe three explanations: why adding inputs multiplies outputs, why the stepped approximations reach this curve, and why it is the only solution with that starting value.";
/// Each clause of DEBTS_SENTENCE, and the words it puts on screen.
const DEBTS: [(&str, &str); 3] = [
    ("why adding", FIRST_DEBT),
    ("why the stepped", SECOND_DEBT),
    ("why it is", THIRD_DEBT),
];

/// The drawing rule, as N9 writes it at the end of the rate chapter.
const RULE: &str = r"y'=y,\quad y(0)=1";
/// `u` and not `a`: the table's `a` is the coefficient of x, which would be `a_1` here.
const RECURRENCE: &str = r"u_{k+1}=\frac{u_k}{k+1},\quad u_0=1";
const RATIO_FIRST: &str = r"\frac{|x|}{k+1}\leq\frac{R}{k+1}";
const RATIO_ALL: &str = r"\frac{|x|}{k+1}\leq\frac{R}{k+1}\leq\frac12\quad(k+1\geq 2R)";
const TAIL_SUM: &str =
    r"\sum_{k=N+1}^{\infty}\frac{|x|^k}{k!}\leq\frac{R^{N+1}}{(N+1)!}\left(1+\frac12+\frac14+\cdots\right)";
const TAIL_TOTAL: &str = r"=\frac{2R^{N+1}}{(N+1)!}\quad(N+2\geq 2R)";
const SHIFTED: &str = r"\frac{d}{dx}\frac{x^k}{k!}=\frac{x^{k-1}}{(k-1)!}";
const DEFINITION: &str = r"E(x)=\sum_{k=0}^{\infty}\frac{x^k}{k!},\quad E'=E,\quad E(0)=1";
const SOLVES: &str = r"y=E\ \text{solves}\ y'=y,\ y(0)=1";
const DOTS: &str = r"+\cdots";

/// The letter polynomial, one term over each column.
const LETTER_TERMS: [&str; 5] = ["1", "+ax", "+bx^2", "+cx^3", "+dx^4"];
/// The column headers: TeX, because the bundled roman face has no ², ³ or ⁴.
const POWERS: [&str; 4] = ["x", "x^2", "x^3", "x^4"];

const COLUMN_0: f64 = -3.9;
const PITCH: f64 = 1.1;
const LABEL_X: f64 = -5.35;
const PREFIX_X: f64 = -4.8;
const RIGHT_X: f64 = 3.9;
const POLY_Y: f64 = 2.6;
const HEADER_Y: f64 = 2.2;
const FUNCTION_Y: f64 = 1.6;
const DERIVATIVE_Y: f64 = 0.8;
const EQUATION_Y: f64 = 0.05;
const CHAIN_Y: f64 = 0.95;
const RUNNING_Y: f64 = 0.2;
const DERIVED_Y: f64 = 1.4;
const CAPTION_Y: f64 = 0.35;

const INTERVAL_MID: f64 = -3.0;
const INTERVAL_HALF: f64 = 2.4;
const INTERVAL_Y: f64 = 2.35;
const BAR_X0: f64 = -5.7;
const BAR_PITCH: f64 = 0.66;
const BAR_WIDTH: f64 = 0.44;
const BAR_BASE: f64 = -2.05;
const BAR_UNIT: f64 = 1.9;
const RATIO_Y: f64 = 1.1;
const INSET_X0: f64 = 2.0;
const INSET_PITCH: f64 = 0.72;
const INSET_WIDTH: f64 = 0.5;
const INSET_BASE: f64 = -2.6;

const ROWS_E_Y: f64 = 2.1;
const ROWS_DERIVATIVE_Y: f64 = 1.0;

fn column(k: usize) -> f64 {
    COLUMN_0 + PITCH * k as f64
}
fn bar_x(k: usize) -> f64 {
    BAR_X0 + BAR_PITCH * k as f64
}
fn inset_x(k: usize) -> f64 {
    INSET_X0 + INSET_PITCH * (k - TAIL_ORDERS[0] - 1) as f64
}
fn inset_unit() -> f64 {
    BAR_UNIT * f64::from(ZOOM)
}
fn interval_x(x: f64) -> f64 {
    INTERVAL_MID + INTERVAL_HALF * x / RADIUS
}

/// Wait for `clause` of sentence `j`: the sentence's real window, shared out by its characters. An estimate inside
/// a sentence, not a heard boundary. Nothing if that moment has passed, or without a narrator.
fn until_clause(k: &Kit, stage: &mut Stage<'_>, id: &str, j: usize, sentence: &str, clause: &str) -> fmn::Result<()> {
    let Some(narrator) = k.narrator.as_ref() else {
        return Ok(());
    };
    let Some(from) = narrator.sentence_at(id, j) else {
        return Ok(());
    };
    let end = narrator
        .sentence_at(id, 0)
        .zip(narrator.length(id))
        .map(|(began, length)| began + length);
    let Some(to) = narrator.sentence_at(id, j + 1).or(end) else {
        return Ok(());
    };
    let byte = sentence.find(clause).expect("the clause is in its sentence");
    let at = from + (to - from) * byte as f64 / sentence.len() as f64;
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        wait_frames(stage, frames_covering(at - now, fps))?;
    }
    Ok(())
}
fn reveal(stage: &mut Stage<'_>, mob: Mob) -> fmn::Result<()> {
    let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
    stage.play(a.rt(0.35))?;
    Ok(())
}
fn reveal_all(stage: &mut Stage<'_>, mobs: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(seconds))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
/// Fade these out. Updaters go first, so a live bar fades instead of being redrawn at full opacity.
fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        animations.push(stage.prepare(a.rt(0.3))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage
        .arena_mut()
        .add(Line::new(a, b).style(stroke_style(color, width)).build()?))
}
fn glide(stage: &mut Stage<'_>, mob: Mob, to: Vec3, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    stage.prepare(a)
}
fn recolor(stage: &mut Stage<'_>, mob: Mob, color: Srgb, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .set_color_with_opacity(color, 1.0)?;
    stage.prepare(a)
}
/// A filled bar standing on its base.
fn bar(stage: &mut Stage<'_>, x: f64, base: f64, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(v(x - width / 2.0, base), width, height, ink_style(color))?;
    Ok(stage.arena_mut().add(r))
}
/// An outlined ceiling standing on the same base.
fn ceiling(stage: &mut Stage<'_>, x: f64, base: f64, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(v(x - width / 2.0, base), width, height, stroke_style(color, 2.0))?;
    Ok(stage.arena_mut().add(r))
}
/// Where a live bar stands, and how tall one unit of the series draws.
#[derive(Clone, Copy)]
struct Slot {
    x: f64,
    base: f64,
    width: f64,
    unit: f64,
}
/// The bar of term k, redrawn each frame from the input the tracker holds.
fn live_bar(stage: &mut Stage<'_>, tracker: Mob, k: usize, slot: Slot, color: Srgb) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let input = s.tracker_value(tracker).unwrap_or(RADIUS);
        let height = (term_size(input, k) * slot.unit).max(0.004);
        let r = Rectangle::new()
            .width(slot.width)
            .height(height)
            .style(ink_style(color))
            .build()
            .expect("positive extents");
        s.add(r.moved_to(v(slot.x, slot.base + height / 2.0)))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// What 02_1 to 02_3 build, and who leaves when.
struct Table {
    /// The polynomial: `y=` and the terms over the columns.
    prefix: Mob,
    terms: Vec<Mob>,
    /// The right panel's equation.
    rule: Mob,
    /// Boxes, headers and the function row's name; these leave at 02_3 sentence 4.
    fixed: Vec<Mob>,
    /// What is in each box now: a letter, then its value.
    entries: Vec<Mob>,
    /// The derivative row's entries, columns 0 to 3.
    derived: Vec<Mob>,
    /// The name, the connectors, the equations and the frame; these leave at 02_3 sentence 1.
    working: Vec<Mob>,
    /// The highlight over the column being matched.
    frame: Mob,
}

fn frame_center(k: usize) -> Vec3 {
    v(column(k), 1.25)
}

/// 02_1: the rule, the polynomial, its coefficients falling into boxes, and differentiation moving them left.
fn opening(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Table> {
    k.say(stage, "02_1")?;
    let title = k.text(stage, TITLE, 35.0, Palette::ink(), v(0.0, 3.3))?;
    let rule = k.tex(stage, RULE, 30.0, Palette::ink(), &[], v(RIGHT_X, 2.2))?;
    reveal_all(stage, &[title, rule], 0.35)?;

    k.until_sentence(stage, "02_1", 1)?;
    let prefix = k.tex(stage, "y=", 27.0, Palette::blue(), &[], v(PREFIX_X, POLY_Y))?;
    let mut terms = Vec::new();
    for (i, term) in LETTER_TERMS.iter().enumerate() {
        terms.push(k.tex(stage, term, 27.0, Palette::blue(), &[], v(column(i), POLY_Y))?);
    }
    let mut polynomial = terms.clone();
    polynomial.push(prefix);
    reveal_all(stage, &polynomial, 0.35)?;

    // The boxes land on the sentence that names them, and each coefficient falls out of its term into its box.
    k.until_sentence(stage, "02_1", 2)?;
    let mut fixed = vec![k.text(stage, "function", 23.0, Palette::blue(), v(LABEL_X, FUNCTION_Y))?];
    for i in 0..5 {
        let x = column(i);
        let boxed = rect(v(x - 0.42, 1.3), 0.84, 0.6, stroke_style(Palette::dim(), 1.5))?;
        fixed.push(stage.arena_mut().add(boxed));
        fixed.push(if i == 0 {
            k.text(stage, "constant", 25.0, Palette::dim(), v(x, HEADER_Y))?
        } else {
            k.tex(stage, POWERS[i - 1], 25.0, Palette::dim(), &[], v(x, HEADER_Y))?
        });
    }
    reveal_all(stage, &fixed, 0.3)?;
    let mut entries = Vec::new();
    let mut falls: Vec<Box<dyn Animation>> = Vec::new();
    for i in 0..5 {
        let name = if i == 0 { "1" } else { letter(i) };
        let entry = k.text(stage, name, 25.0, Palette::blue(), v(column(i), FUNCTION_Y))?;
        let fall = fade_in(stage.arena_mut(), entry, v(0.0, FUNCTION_Y - POLY_Y), 1.0)?;
        falls.push(stage.prepare(fall.rt(0.6))?);
        entries.push(entry);
    }
    stage.play_prepared(falls)?;

    k.until_sentence(stage, "02_1", 3)?;
    let name = k.text(stage, "derivative", 23.0, Palette::gold(), v(LABEL_X, DERIVATIVE_Y))?;
    reveal(stage, name)?;
    let mut working = vec![name];
    let mut derived = Vec::new();
    for n in 1..=4 {
        // A copy of the term leaves its box, moves one column left and picks up its power.
        let moving = k.text(stage, letter(n), 25.0, Palette::gold(), v(column(n), FUNCTION_Y))?;
        reveal(stage, moving)?;
        let target = k.text(
            stage,
            &derivative_label(n),
            25.0,
            Palette::gold(),
            v(column(n - 1), DERIVATIVE_Y),
        )?;
        stage.play(Transform::new(moving, target).rt(0.6))?;
        derived.push(moving);
        let link = line(stage, v(column(n), 1.35), v(column(n - 1), 1.02), Palette::dim(), 2.0)?;
        reveal(stage, link)?;
        working.push(link);
    }

    // The frame points at the column the question is about. The answer stays hidden.
    k.until_sentence(stage, "02_1", 4)?;
    let highlight = rect(
        v(column(0) - 0.48, 0.52),
        0.96,
        1.46,
        stroke_style(Palette::yellow(), 3.0),
    )?;
    let frame = stage.arena_mut().add(highlight);
    let question = k.text(stage, QUESTION, 27.0, Palette::hero(), v(column(2), -0.75))?;
    reveal_all(stage, &[frame, question], 0.35)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, 4.0)?;
    clear(stage, &[question])?;
    Ok(Table {
        prefix,
        terms,
        rule,
        fixed,
        entries,
        derived,
        working,
        frame,
    })
}

/// Light the term about to be differentiated and what it leaves behind.
fn pulse(stage: &mut Stage<'_>, table: &Table, n: usize) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for m in [table.entries[n], table.derived[n - 1]] {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.3, None)?;
        animations.push(stage.prepare(a.rt(0.8))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
/// The frame moves to the column being matched, and that column's equation is printed under it.
fn print_equation(stage: &mut Stage<'_>, k: &Kit, table: &Table, n: usize) -> fmn::Result<Mob> {
    let at = v(column(n - 1), EQUATION_Y);
    let equation = k.text(stage, &equation_label(n), 25.0, Palette::gold(), at)?;
    let shown = fade_in(stage.arena_mut(), equation, ORIGIN, 1.0)?;
    let animations = vec![
        glide(stage, table.frame, frame_center(n - 1), 0.4)?,
        stage.prepare(shown.rt(0.4))?,
    ];
    stage.play_prepared(animations)?;
    Ok(equation)
}
/// The equation becomes its solution, and the value rises into the next box, where the next column will read it.
fn solve(stage: &mut Stage<'_>, k: &Kit, table: &mut Table, n: usize, equation: Mob) -> fmn::Result<()> {
    let at = v(column(n - 1), EQUATION_Y);
    let target = v(column(n), FUNCTION_Y);
    let arrived = k.text(stage, &coefficient(n), 25.0, Palette::green(), target)?;
    let rise = fade_in(stage.arena_mut(), arrived, v(target[0] - at[0], target[1] - at[1]), 1.0)?;
    let gone = fade_out(stage.arena_mut(), table.entries[n], ORIGIN, 1.0)?;
    let mut animations = vec![stage.prepare(rise.rt(0.5))?, stage.prepare(gone.rt(0.5))?];
    if equation_label(n) == value_label(n) {
        table.working.push(equation);
    } else {
        let solved = k.text(stage, &value_label(n), 25.0, Palette::green(), at)?;
        let old = fade_out(stage.arena_mut(), equation, ORIGIN, 1.0)?;
        let new = fade_in(stage.arena_mut(), solved, ORIGIN, 1.0)?;
        animations.push(stage.prepare(old.rt(0.5))?);
        animations.push(stage.prepare(new.rt(0.5))?);
        table.working.push(solved);
    }
    stage.play_prepared(animations)?;
    table.entries[n] = arrived;
    Ok(())
}

/// 02_2: the coefficients are matched, one column at a time, and each value feeds the next column.
fn matching(stage: &mut Stage<'_>, k: &Kit, table: &mut Table) -> fmn::Result<()> {
    k.say(stage, "02_2")?;
    let first = print_equation(stage, k, table, 1)?;
    solve(stage, k, table, 1, first)?;
    k.until_sentence(stage, "02_2", 1)?;
    pulse(stage, table, 2)?;
    k.until_sentence(stage, "02_2", 2)?;
    let second = print_equation(stage, k, table, 2)?;
    k.until_sentence(stage, "02_2", 3)?;
    solve(stage, k, table, 2, second)?;
    k.until_sentence(stage, "02_2", 4)?;
    pulse(stage, table, 3)?;
    k.until_sentence(stage, "02_2", 5)?;
    let third = print_equation(stage, k, table, 3)?;
    stage.wait(0.5)?;
    solve(stage, k, table, 3, third)?;
    Ok(())
}

/// 02_3 up to the factorials: the last column, the rule as a chain, and the divisions piling up in the denominators.
fn factorials_appear(stage: &mut Stage<'_>, k: &Kit, table: &mut Table) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "02_3")?;
    let fourth = print_equation(stage, k, table, 4)?;
    solve(stage, k, table, 4, fourth)?;

    // The derivative row has done its work. What remains is the row of values and the rule that made it.
    k.until_sentence(stage, "02_3", 1)?;
    let mut leaving = table.working.clone();
    leaving.extend(table.derived.iter().copied());
    leaving.push(table.frame);
    clear(stage, &leaving)?;
    let mut chain = vec![k.tex(stage, RECURRENCE, 28.0, Palette::ink(), &[], v(RIGHT_X, CHAIN_Y))?];
    for i in 0..5 {
        chain.push(k.tex(
            stage,
            &format!("u_{i}"),
            24.0,
            Palette::dim(),
            &[],
            v(column(i), CHAIN_Y),
        )?);
    }
    for i in 1..=4 {
        let middle = (column(i - 1) + column(i)) / 2.0;
        chain.push(k.tex(
            stage,
            &format!(r"\xrightarrow{{\div {i}}}"),
            24.0,
            Palette::gold(),
            &[],
            v(middle, CHAIN_Y),
        )?);
    }
    reveal_all(stage, &chain, 0.4)?;

    // As the voice reads the values, each one's divisions gather under it.
    k.until_sentence(stage, "02_3", 2)?;
    let name = k.text(stage, "denominator", 23.0, Palette::ink(), v(LABEL_X, RUNNING_Y))?;
    reveal_all(stage, &[name], 0.2)?;
    chain.push(name);
    let mut denominators = Vec::new();
    for (i, &entry) in table.entries.iter().enumerate() {
        let m = k.tex(
            stage,
            &running_product_tex(i),
            25.0,
            Palette::ink(),
            &[],
            v(column(i), RUNNING_Y),
        )?;
        let shown = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        let lit = fmn::animation::indicate(stage.arena_mut(), entry, 1.25, None)?;
        let animations = vec![stage.prepare(shown.rt(0.42))?, stage.prepare(lit.rt(0.42))?];
        stage.play_prepared(animations)?;
        denominators.push(m);
    }

    // The first factorial signs of the chapter, on the sentence that names them.
    k.until_sentence(stage, "02_3", 3)?;
    let mut morphs: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &m) in denominators.iter().enumerate() {
        let target = k.tex(
            stage,
            &factorial_tex(i),
            27.0,
            Palette::hero(),
            &[],
            v(column(i), RUNNING_Y),
        )?;
        morphs.push(stage.prepare(Transform::new(m, target).rt(0.6))?);
    }
    stage.play_prepared(morphs)?;
    Ok((chain, denominators))
}

/// 02_3 from sentence 4: the factorials rise into the polynomial, and its terms light up with their curves.
fn curves(
    stage: &mut Stage<'_>,
    k: &Kit,
    table: &mut Table,
    chain: &[Mob],
    denominators: &[Mob],
) -> fmn::Result<Vec<Mob>> {
    k.until_sentence(stage, "02_3", 4)?;
    let mut changes: Vec<Box<dyn Animation>> = Vec::new();
    let mut leaving = table.fixed.clone();
    leaving.extend(table.entries.iter().copied());
    leaving.extend(chain.iter().copied());
    leaving.extend(table.terms.iter().copied());
    leaving.push(table.rule);
    for m in leaving {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        changes.push(stage.prepare(a.rt(0.6))?);
    }
    for (i, &m) in denominators.iter().enumerate() {
        let target = k.tex(stage, &column_tex(i), 27.0, Palette::dim(), &[], v(column(i), POLY_Y))?;
        changes.push(stage.prepare(Transform::new(m, target).rt(0.6))?);
    }
    stage.play_prepared(changes)?;
    table.terms = denominators.to_vec();

    let map = |x: f64, y: f64| v(-4.8 + 6.4 * x, -2.0 + 0.8 * y);
    let mut graph = Vec::new();
    for (label, position) in [
        ("0".to_owned(), v(-5.05, -2.3)),
        (RADIUS.to_string(), v(4.8, -2.3)),
        (PLOT_TOP.to_string(), v(-5.1, 2.0)),
    ] {
        graph.push(k.text(stage, &label, 23.0, Palette::dim(), position)?);
    }
    for (a, b) in [(map(0.0, 0.0), map(RADIUS, 0.0)), (map(0.0, 0.0), map(0.0, PLOT_TOP))] {
        graph.push(line(stage, a, b, Palette::dim(), 2.0)?);
    }
    reveal_all(stage, &graph, 0.4)?;
    // One path per partial sum. The newest stays blue, the one before it dims, and its term lights above.
    let mut previous: Option<Mob> = None;
    for n in ORDERS {
        if n == 3 {
            k.until_sentence(stage, "02_3", 5)?;
        }
        let points: Vec<Vec3> = (0..=PLOT_SEGMENTS).map(|j| map(plot_x(j), sum(plot_x(j), n))).collect();
        let curve = stage
            .arena_mut()
            .add(polyline(points).style(stroke_style(Palette::blue(), 2.5)).build());
        let a = fade_in(stage.arena_mut(), curve, ORIGIN, 1.0)?;
        let mut animations = vec![
            stage.prepare(a.rt(0.5))?,
            recolor(stage, table.terms[n], Palette::blue(), 0.5)?,
        ];
        if let Some(old) = previous {
            // set_stroke, not set_color: a recolour also sets fill alpha and would fill the open path.
            let dimmed = old
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.5),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(Palette::dim()), None, None)?;
            animations.push(stage.prepare(dimmed)?);
        }
        if n == 3 {
            let note = k.text(stage, CURVES_NOTE, 23.0, Palette::dim(), v(0.0, -2.3))?;
            let shown = fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?;
            animations.push(stage.prepare(shown.rt(0.5))?);
            graph.push(note);
        }
        stage.play_prepared(animations)?;
        stage.wait(0.4)?;
        graph.push(curve);
        previous = Some(curve);
    }
    k.hold(stage)?;
    Ok(graph)
}

/// 02_4: differentiation leaves a hole, each new term fills the last hole and makes the next, and the dots arrive.
fn missing_power(stage: &mut Stage<'_>, k: &Kit, table: &Table, graph: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "02_4")?;
    clear(stage, graph)?;
    let derived_prefix = k.tex(stage, "y'=", 27.0, Palette::gold(), &[], v(PREFIX_X, DERIVED_Y))?;
    reveal_all(stage, &[derived_prefix], 0.3)?;
    let mut screen = vec![table.prefix, derived_prefix];
    screen.extend(table.terms.iter().copied());

    // Each term's derivative lands one column left, under the term it equals. The last column has nothing.
    k.until_sentence(stage, "02_4", 1)?;
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for (n, &term) in table.terms.iter().enumerate().skip(1) {
        let target = k.tex(
            stage,
            &column_tex(n - 1),
            27.0,
            Palette::gold(),
            &[],
            v(column(n - 1), DERIVED_Y),
        )?;
        let copy = fmn::animation::transform_from_copy(stage.arena_mut(), term, target)?;
        moves.push(stage.prepare(copy.rt(0.7))?);
        screen.push(target);
    }
    stage.play_prepared(moves)?;
    let empty = rect(
        v(column(4) - 0.42, DERIVED_Y - 0.33),
        0.84,
        0.66,
        stroke_style(Palette::hero(), 2.5),
    )?;
    let hole = stage.arena_mut().add(empty);
    let lost = k.text(stage, LOST_POWER, 25.0, Palette::gold(), v(0.0, CAPTION_Y))?;
    reveal_all(stage, &[hole, lost], 0.35)?;

    // One more term fills the hole and opens the next one. Twice is enough to see the pattern; then the dots.
    k.until_sentence(stage, "02_4", 2)?;
    for n in [5, 6] {
        let term = k.tex(stage, &column_tex(n), 27.0, Palette::hero(), &[], v(column(n), POLY_Y))?;
        reveal(stage, term)?;
        screen.push(term);
        let target = k.tex(
            stage,
            &column_tex(n - 1),
            27.0,
            Palette::gold(),
            &[],
            v(column(n - 1), DERIVED_Y),
        )?;
        let copy = fmn::animation::transform_from_copy(stage.arena_mut(), term, target)?;
        let animations = vec![
            stage.prepare(copy.rt(0.6))?,
            glide(stage, hole, v(column(n), DERIVED_Y), 0.6)?,
        ];
        stage.play_prepared(animations)?;
        screen.push(target);
    }
    let dots_above = k.tex(stage, DOTS, 27.0, Palette::hero(), &[], v(column(7), POLY_Y))?;
    let dots_below = k.tex(stage, DOTS, 27.0, Palette::hero(), &[], v(column(6), DERIVED_Y))?;
    let closed = fade_out(stage.arena_mut(), hole, ORIGIN, 1.0)?;
    let mut last = vec![stage.prepare(closed.rt(0.35))?];
    for m in [dots_above, dots_below] {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        last.push(stage.prepare(a.rt(0.35))?);
    }
    stage.play_prepared(last)?;
    screen.extend([dots_above, dots_below]);

    // These dots, on screen, are what the voice says are not a proof.
    k.until_sentence(stage, "02_4", 3)?;
    let needs = k.text(stage, NEEDS_PROOF, 25.0, Palette::gold(), v(0.0, CAPTION_Y))?;
    let out = fade_out(stage.arena_mut(), lost, ORIGIN, 1.0)?;
    let into = fade_in(stage.arena_mut(), needs, ORIGIN, 1.0)?;
    let pulse_above = fmn::animation::indicate(stage.arena_mut(), dots_above, 1.6, None)?;
    let pulse_below = fmn::animation::indicate(stage.arena_mut(), dots_below, 1.6, None)?;
    let animations = vec![
        stage.prepare(out.rt(0.6))?,
        stage.prepare(into.rt(0.6))?,
        stage.prepare(pulse_above.rt(0.6))?,
        stage.prepare(pulse_below.rt(0.6))?,
    ];
    stage.play_prepared(animations)?;
    screen.push(needs);

    k.until_sentence(stage, "02_4", 4)?;
    let exists = k.text(stage, SUM_EXISTS, 25.0, Palette::ink(), v(0.0, -0.75))?;
    reveal(stage, exists)?;
    until_clause(k, stage, "02_4", 4, OBLIGATIONS_SENTENCE, "and that differentiating")?;
    let allowed = k.text(stage, TERM_BY_TERM, 25.0, Palette::ink(), v(0.0, -1.45))?;
    reveal(stage, allowed)?;
    screen.extend([exists, allowed]);
    k.hold(stage)?;
    clear(stage, &screen)?;
    Ok(())
}

/// 02_5: the terms as bars. Their ratios halve, the tail fits under a halving ceiling at any zoom,
/// the derivative series is the same bars, and the ceiling does not move when the input does.
fn tail(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "02_5")?;
    let top = BAR_TOP;
    let zoomed = TAIL_ORDERS[0];
    let mut card = vec![
        line(
            stage,
            v(interval_x(-RADIUS), INTERVAL_Y),
            v(interval_x(RADIUS), INTERVAL_Y),
            Palette::blue(),
            3.0,
        )?,
        k.tex(
            stage,
            "-R",
            25.0,
            Palette::blue(),
            &[],
            v(interval_x(-RADIUS), INTERVAL_Y - 0.3),
        )?,
        k.tex(
            stage,
            "R",
            25.0,
            Palette::blue(),
            &[],
            v(interval_x(RADIUS), INTERVAL_Y - 0.3),
        )?,
        k.text(
            stage,
            &format!("R = {RADIUS}"),
            23.0,
            Palette::dim(),
            v(INTERVAL_MID, INTERVAL_Y + 0.35),
        )?,
    ];
    let marker = Circle::new()
        .radius(0.09)
        .style(ink_style(Palette::gold()))
        .build()
        .moved_to(v(interval_x(RADIUS), INTERVAL_Y));
    let dot = stage.arena_mut().add(marker);
    let mut interval = card.clone();
    interval.push(dot);
    reveal_all(stage, &interval, 0.4)?;

    // Every term as a bar at the far end of the interval, where it is largest.
    k.until_sentence(stage, "02_5", 1)?;
    let mut bars = Vec::new();
    let mut grows: Vec<Box<dyn Animation>> = Vec::new();
    for i in 0..=top {
        let height = term_size(RADIUS, i) * BAR_UNIT;
        let b = bar(stage, bar_x(i), BAR_BASE, BAR_WIDTH, height, Palette::blue())?;
        let grow = fmn::animation::grow_from_edge(stage.arena_mut(), b, DOWN, None)?;
        grows.push(stage.prepare(grow)?);
        bars.push(b);
    }
    let group = lagged(stage, grows, 0.1)?;
    stage.play(group.rt(1.0))?;
    let mut labels = vec![
        k.tex(stage, r"\frac{|x|^k}{k!}", 26.0, Palette::blue(), &[], v(-2.0, 0.3))?,
        k.text(stage, "ratio", 23.0, Palette::dim(), v(bar_x(0), RATIO_Y))?,
        line(
            stage,
            v(bar_x(0) - 0.3, BAR_BASE),
            v(bar_x(top) + 0.3, BAR_BASE),
            Palette::dim(),
            2.0,
        )?,
    ];
    for i in 0..=top {
        labels.push(k.text(
            stage,
            &i.to_string(),
            21.0,
            Palette::dim(),
            v(bar_x(i), BAR_BASE - 0.25),
        )?);
    }
    let mut ratios = Vec::new();
    for i in 1..=top {
        ratios.push(k.text(stage, &ratio_label(i), 23.0, Palette::ink(), v(bar_x(i), RATIO_Y))?);
    }
    let (first, first_spans) = k.tex_spans(stage, RATIO_FIRST, 28.0, Palette::blue(), &[], v(RIGHT_X, 2.3))?;
    let mut shown = labels.clone();
    shown.extend(ratios.iter().copied());
    shown.push(first);
    reveal_all(stage, &shown, 0.35)?;
    card.extend(labels);
    card.extend(ratios.iter().copied());

    // "Eventually": the ratios that reach one half turn green, and the formula gains the half and its hypothesis.
    k.until_sentence(stage, "02_5", 2)?;
    let (all, all_spans) = k.tex_spans(stage, RATIO_ALL, 28.0, Palette::blue(), &[], v(RIGHT_X, 2.3))?;
    let morph = TransformMatchingTex::new(stage.arena_mut(), first, all, &first_spans, &all_spans)?;
    let mut animations = vec![stage.prepare(morph.rt(0.6))?];
    for (j, &m) in ratios.iter().enumerate() {
        let color = if j + 1 >= halving_from(RADIUS) {
            Palette::green()
        } else {
            Palette::dim()
        };
        animations.push(recolor(stage, m, color, 0.6)?);
    }
    stage.play_prepared(animations)?;
    card.push(all);

    // The tail is a few pixels here, so a picture at this scale proves nothing. Zoom in on it.
    k.until_sentence(stage, "02_5", 3)?;
    let mut inset = Vec::new();
    let mut zooms: Vec<Box<dyn Animation>> = Vec::new();
    for i in zoomed + 1..=INSET_TOP {
        let height = term_size(RADIUS, i) * inset_unit();
        let target = bar(stage, inset_x(i), INSET_BASE, INSET_WIDTH, height, Palette::hero())?;
        let enlarge = if let Some(&source) = bars.get(i) {
            let copy = fmn::animation::transform_from_copy(stage.arena_mut(), source, target)?;
            stage.prepare(copy.rt(0.6))?
        } else {
            let shown = fade_in(stage.arena_mut(), target, ORIGIN, 1.0)?;
            stage.prepare(shown.rt(0.6))?
        };
        zooms.push(enlarge);
        inset.push(target);
    }
    stage.play_prepared(zooms)?;
    let mut lids = Vec::new();
    let mut ceilings = vec![
        line(
            stage,
            v(inset_x(zoomed + 1) - 0.4, INSET_BASE),
            v(inset_x(INSET_TOP) + 0.4, INSET_BASE),
            Palette::dim(),
            2.0,
        )?,
        k.tex(
            stage,
            r"\cdots",
            26.0,
            Palette::hero(),
            &[],
            v(inset_x(INSET_TOP) + 0.75, INSET_BASE + 0.2),
        )?,
        k.text(stage, &format!("zoom ×{ZOOM}"), 23.0, Palette::dim(), v(5.4, -0.95))?,
    ];
    for i in zoomed + 1..=INSET_TOP {
        let height = envelope(RADIUS, zoomed, i) * inset_unit();
        let lid = ceiling(stage, inset_x(i), INSET_BASE, INSET_WIDTH, height, Palette::green())?;
        lids.push(lid);
        ceilings.push(lid);
        ceilings.push(k.text(
            stage,
            &i.to_string(),
            21.0,
            Palette::dim(),
            v(inset_x(i), INSET_BASE - 0.25),
        )?);
    }
    reveal_all(stage, &ceilings, 0.4)?;
    let sum_line = k.tex(stage, TAIL_SUM, 25.0, Palette::green(), &[], v(RIGHT_X, 1.3))?;
    let total = k.tex(stage, TAIL_TOTAL, 25.0, Palette::green(), &[], v(RIGHT_X, 0.5))?;
    let mut bounds = vec![sum_line, total];
    for (row, n) in TAIL_ORDERS.iter().copied().enumerate() {
        let printed = tail_label(tail_bound(RADIUS, n).expect("the drawn degrees meet the hypothesis"));
        let at = v(RIGHT_X, -0.05 - 0.36 * row as f64);
        bounds.push(k.text(
            stage,
            &format!("|x| ≤ {RADIUS}, degree {n}: tail ≤ {printed}"),
            23.0,
            Palette::ink(),
            at,
        )?);
    }
    reveal_all(stage, &bounds, 0.35)?;
    card.extend(ceilings);
    card.extend(bounds);

    // The derivative of each term is the previous term: the same bars, one column left, so the same bound.
    k.until_sentence(stage, "02_5", 4)?;
    let mut shifted = Vec::new();
    let mut shifts: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &source) in bars.iter().enumerate().skip(1) {
        let height = term_size(RADIUS, i - 1) * BAR_UNIT;
        let target = bar(stage, bar_x(i - 1), BAR_BASE, BAR_WIDTH, height, Palette::gold())?;
        let copy = fmn::animation::transform_from_copy(stage.arena_mut(), source, target)?;
        shifts.push(stage.prepare(copy.rt(0.7))?);
        shifted.push(target);
    }
    stage.play_prepared(shifts)?;
    let same = k.tex(stage, SHIFTED, 26.0, Palette::gold(), &[], v(-3.0, -3.0))?;
    reveal(stage, same)?;
    card.push(same);

    // Uniform: the input slides across the whole interval, and no bar ever passes its ceiling.
    k.until_sentence(stage, "02_5", 5)?;
    let uniform = k.text(stage, UNIFORM, 25.0, Palette::gold(), v(0.0, -3.55))?;
    let mut top_lines = Vec::new();
    for i in 0..=top {
        let height = term_size(RADIUS, i) * BAR_UNIT;
        top_lines.push(ceiling(stage, bar_x(i), BAR_BASE, BAR_WIDTH, height, Palette::green())?);
    }
    let mut change: Vec<Box<dyn Animation>> = Vec::new();
    let in_uniform = fade_in(stage.arena_mut(), uniform, ORIGIN, 1.0)?;
    change.push(stage.prepare(in_uniform.rt(0.35))?);
    for &m in &top_lines {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        change.push(stage.prepare(a.rt(0.35))?);
    }
    for &m in &shifted {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        change.push(stage.prepare(a.rt(0.35))?);
    }
    stage.play_prepared(change)?;
    card.push(uniform);
    card.extend(top_lines.iter().copied());
    lids.extend(top_lines);

    // Live from here: at the tracker's starting value each live bar is exactly the bar it replaces.
    let input = stage.add_value_tracker(RADIUS);
    for (i, &old) in bars.iter().enumerate() {
        let slot = Slot {
            x: bar_x(i),
            base: BAR_BASE,
            width: BAR_WIDTH,
            unit: BAR_UNIT,
        };
        card.push(live_bar(stage, input, i, slot, Palette::blue())?);
        stage.remove_from_scene(old);
    }
    for (j, &old) in inset.iter().enumerate() {
        let i = zoomed + 1 + j;
        let slot = Slot {
            x: inset_x(i),
            base: INSET_BASE,
            width: INSET_WIDTH,
            unit: inset_unit(),
        };
        card.push(live_bar(stage, input, i, slot, Palette::hero())?);
        stage.remove_from_scene(old);
    }
    let live_dot = stage.always_redraw(move |s| {
        let x = s.tracker_value(input).unwrap_or(RADIUS);
        let marker = Circle::new()
            .radius(0.09)
            .style(ink_style(Palette::gold()))
            .build()
            .moved_to(v(interval_x(x), INTERVAL_Y));
        s.add(marker)
    });
    stage.add_to_scene(live_dot)?;
    stage.remove_from_scene(dot);
    card.push(live_dot);
    // The live bars were added last. The ceilings go back on top, so the swap is not seen and each lid stays whole.
    for &lid in &lids {
        stage.add_to_scene(lid)?;
    }
    let sweep = tween(input, -RADIUS, 3.2)?;
    let prepared = stage.prepare(sweep)?;
    stage.play_prepared(vec![prepared])?;
    k.hold(stage)?;
    Ok(card)
}

/// 02_6: the rows return with their dots, the shift is seen once more, and they match across.
fn rows_match(stage: &mut Stage<'_>, k: &Kit, card: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "02_6")?;
    clear(stage, card)?;
    let mut above = vec![k.tex(stage, "E=", 27.0, Palette::blue(), &[], v(PREFIX_X, ROWS_E_Y))?];
    let mut terms = Vec::new();
    for i in 0..=5 {
        terms.push(k.tex(
            stage,
            &column_tex(i),
            27.0,
            Palette::blue(),
            &[],
            v(column(i), ROWS_E_Y),
        )?);
    }
    let dots_above = k.tex(stage, DOTS, 27.0, Palette::blue(), &[], v(column(6), ROWS_E_Y))?;
    above.extend(terms.iter().copied());
    above.push(dots_above);
    reveal_all(stage, &above, 0.35)?;

    // The derivative of each term lands one column left, and the dots supply the last one.
    k.until_sentence(stage, "02_6", 1)?;
    let name = k.tex(stage, "E'=", 27.0, Palette::gold(), &[], v(PREFIX_X, ROWS_DERIVATIVE_Y))?;
    let dots_below = k.tex(stage, DOTS, 27.0, Palette::gold(), &[], v(column(6), ROWS_DERIVATIVE_Y))?;
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for shown in [name, dots_below] {
        let a = fade_in(stage.arena_mut(), shown, ORIGIN, 1.0)?;
        moves.push(stage.prepare(a.rt(0.7))?);
    }
    for i in 1..=6 {
        let source = terms.get(i).copied().unwrap_or(dots_above);
        let target = k.tex(
            stage,
            &column_tex(i - 1),
            27.0,
            Palette::gold(),
            &[],
            v(column(i - 1), ROWS_DERIVATIVE_Y),
        )?;
        let copy = fmn::animation::transform_from_copy(stage.arena_mut(), source, target)?;
        moves.push(stage.prepare(copy.rt(0.7))?);
    }
    stage.play_prepared(moves)?;
    // One frame walks the columns, so "all the way across" is something the eye does.
    let middle = (ROWS_E_Y + ROWS_DERIVATIVE_Y) / 2.0;
    let walker = rect(
        v(column(0) - 0.48, middle - 0.9),
        0.96,
        1.8,
        stroke_style(Palette::yellow(), 3.0),
    )?;
    let walker = stage.arena_mut().add(walker);
    reveal_all(stage, &[walker], 0.2)?;
    for i in 1..=6 {
        let step = glide(stage, walker, v(column(i), middle), 0.22)?;
        stage.play_prepared(vec![step])?;
    }
    clear(stage, &[walker])?;
    let definition = k.tex(stage, DEFINITION, 28.0, Palette::blue(), &[], v(0.0, 0.0))?;
    reveal(stage, definition)?;

    k.until_sentence(stage, "02_6", 2)?;
    let solves = k.tex(stage, SOLVES, 26.0, Palette::green(), &[], v(0.0, -0.85))?;
    reveal(stage, solves)?;

    // The three debts, in the order the next chapters pay them, each on its own clause.
    k.until_sentence(stage, "02_6", 3)?;
    let mut debts = Vec::new();
    for (row, (clause, debt)) in DEBTS.iter().enumerate() {
        until_clause(k, stage, "02_6", 3, DEBTS_SENTENCE, clause)?;
        let m = k.text(stage, debt, 25.0, Palette::gold(), v(0.0, -1.55 - 0.65 * row as f64))?;
        reveal(stage, m)?;
        debts.push(m);
    }
    k.until_sentence(stage, "02_6", 4)?;
    let mut lit: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &debts {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.08, None)?;
        lit.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(lit)?;
    Ok(())
}

impl SceneConstruct for Factorials {
    fn name(&self) -> &str {
        "02_factorials"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let mut table = opening(stage, &k)?;
        matching(stage, &k, &mut table)?;
        let (chain, denominators) = factorials_appear(stage, &k, &mut table)?;
        let graph = curves(stage, &k, &mut table, &chain, &denominators)?;
        missing_power(stage, &k, &table, &graph)?;
        let card = tail(stage, &k)?;
        rows_match(stage, &k, &card)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("factorials.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("split yields a first part")
    }
    fn assert_in_order(needles: &[&str]) {
        let mut from = 0;
        for &needle in needles {
            let at = scene()[from..]
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} missing or out of order"));
            from += at + needle.len();
        }
    }
    #[test]
    fn the_think_gap_is_authored_and_the_answer_waits() {
        assert_in_order(&[
            "reveal_all(stage, &[frame, question], 0.35)?;",
            "k.hold(stage)?;",
            r#"k.mark(stage, "predict");"#,
            "k.music_breath(stage, 4.0)?;",
            "clear(stage, &[question])?;",
            r#"k.say(stage, "02_2")?;"#,
            "print_equation(stage, k, table, 1)?;",
        ]);
    }
    #[test]
    fn the_polynomial_lands_on_try_a_polynomial() {
        assert_in_order(&[
            r#"until_sentence(stage, "02_1", 1)"#,
            "LETTER_TERMS.iter()",
            r#"until_sentence(stage, "02_1", 2)"#,
            "stroke_style(Palette::dim(), 1.5)",
        ]);
    }
    #[test]
    fn each_equation_lands_on_its_sentence_and_is_then_solved() {
        assert_in_order(&[
            r#"until_sentence(stage, "02_2", 1)"#,
            "pulse(stage, table, 2)?;",
            r#"until_sentence(stage, "02_2", 2)"#,
            "print_equation(stage, k, table, 2)?;",
            r#"until_sentence(stage, "02_2", 3)"#,
            "solve(stage, k, table, 2, second)?;",
            r#"until_sentence(stage, "02_2", 4)"#,
            "pulse(stage, table, 3)?;",
            r#"until_sentence(stage, "02_2", 5)"#,
            "print_equation(stage, k, table, 3)?;",
            "solve(stage, k, table, 3, third)?;",
            r#"k.say(stage, "02_3")?;"#,
            "print_equation(stage, k, table, 4)?;",
        ]);
    }
    #[test]
    fn no_factorial_sign_appears_before_the_sentence_that_names_them() {
        let before = scene().split(r#"until_sentence(stage, "02_3", 3)"#).next().unwrap();
        assert!(!before.contains("factorial_tex(i)"), "factorial signs drawn early");
        let before_curves = scene().split("fn curves(").next().unwrap();
        assert!(!before_curves.contains("column_tex("), "factorial terms drawn early");
        assert_in_order(&[
            r#"until_sentence(stage, "02_3", 2)"#,
            "running_product_tex(i)",
            r#"until_sentence(stage, "02_3", 3)"#,
            "factorial_tex(i)",
            r#"until_sentence(stage, "02_3", 4)"#,
            "column_tex(i)",
        ]);
    }
    #[test]
    fn the_recurrence_does_not_reuse_the_tables_letter() {
        assert!(!RECURRENCE.contains("a_"));
        assert!(RECURRENCE.contains("u_{k+1}"));
        assert!(RECURRENCE.contains("u_0=1"));
    }
    #[test]
    fn the_dots_are_on_screen_before_they_are_refused() {
        assert_in_order(&[
            r#"until_sentence(stage, "02_4", 1)"#,
            "stroke_style(Palette::hero(), 2.5)",
            "LOST_POWER",
            r#"until_sentence(stage, "02_4", 2)"#,
            "for n in [5, 6]",
            "DOTS",
            r#"until_sentence(stage, "02_4", 3)"#,
            "NEEDS_PROOF",
            r#"until_sentence(stage, "02_4", 4)"#,
            "SUM_EXISTS",
            "TERM_BY_TERM",
        ]);
    }
    #[test]
    fn the_proof_steps_land_on_their_sentences() {
        assert_in_order(&[
            r#"until_sentence(stage, "02_5", 1)"#,
            "RATIO_FIRST",
            r#"until_sentence(stage, "02_5", 2)"#,
            "RATIO_ALL",
            r#"until_sentence(stage, "02_5", 3)"#,
            "envelope(RADIUS, zoomed, i)",
            "TAIL_SUM",
            "TAIL_TOTAL",
            r#"until_sentence(stage, "02_5", 4)"#,
            "SHIFTED",
            r#"until_sentence(stage, "02_5", 5)"#,
            "UNIFORM",
            "tween(input, -RADIUS, 3.2)",
        ]);
    }
    #[test]
    fn the_half_waits_for_eventually() {
        assert!(!RATIO_FIRST.contains(r"\frac12"));
        assert!(RATIO_ALL.starts_with(RATIO_FIRST));
    }
    #[test]
    fn the_hypotheses_are_printed_where_they_are_used() {
        assert!(RATIO_ALL.contains(r"(k+1\geq 2R)"));
        assert!(TAIL_TOTAL.contains(r"(N+2\geq 2R)"));
    }
    #[test]
    fn every_clause_is_in_its_sentence_in_spoken_order() {
        let at: Vec<usize> = DEBTS.iter().map(|(c, _)| DEBTS_SENTENCE.find(c).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]));
        assert!(OBLIGATIONS_SENTENCE.contains("and that differentiating"));
    }
    #[test]
    fn the_chapter_ends_with_all_three_debts_in_payment_order() {
        assert_in_order(&[
            r#"until_sentence(stage, "02_6", 1)"#,
            "DEFINITION",
            r#"until_sentence(stage, "02_6", 2)"#,
            "SOLVES",
            r#"until_sentence(stage, "02_6", 3)"#,
            "DEBTS.iter()",
            "until_clause(k, stage, \"02_6\", 3, DEBTS_SENTENCE, clause)?;",
            r#"until_sentence(stage, "02_6", 4)"#,
        ]);
    }
    #[test]
    fn live_bars_lose_their_updaters_before_they_fade() {
        assert_in_order(&["fn clear(", "stage.clear_updaters(m, true);", "fade_out("]);
    }
}
