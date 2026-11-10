//! Native staged uniqueness chapter. Stage05 source edits are not built or rendered.
//!
//! Each cue has one function, in film order, and each function begins with its own `say`.
//! The test y(x)E(-x) is built for any differentiable y, tried on functions that fail, then applied to the rule.
//! What it leans on is drawn as a one-way ledger, so the viewer can check that nothing here feeds back into the series.
use crate::limit_model::e_value;
use crate::phrase::until_phrase;
use crate::solution_model::{
    DEGREES, FAR_MULTIPLE, FAR_PIECE, FUNCTION_TOP, NEAR_PIECE, PANEL_RADIUS, PANEL_SAMPLES, PIECE_SAMPLES, PIECE_TOP,
    PRODUCT_TOP, RATE_END, RATE_SAMPLES, RATE_START, RATE_TOP, RATES, defect_tex, far_piece, imagined, near_piece,
    panel_x, polynomial, product, rate_label, rate_solution, rate_time,
};
use explainer_kit::{Kit, Palette, Timed, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct OnlySolution {
    pub kit: Kit,
}

const TITLE: &str = "Could another curve obey the rule?";
const LEDGER_TITLE: &str = "Is this a circle?";
const GAP_TITLE: &str = "The word interval is doing work";
const RATE_TITLE: &str = "Any constant rate";
const DOORS_TITLE: &str = "Three descriptions, one object";
const TITLE_Y: f64 = 3.35;
const RIGHT_X: f64 = 3.5;
const LEFT_X: f64 = -3.4;

const RULE: &str = r"y'=y,\quad y(0)=1";
const PROVED: &str = "Proved: the stepped approximations reach E";
const NOT_YET: &str = "Not yet: that every solution follows the steps";
const THIRD_DEBT: &str = "Why E is the only solution from height one";

const TEST: &str = r"y(x)E(-x)";
const DIFFERENTIATE: &str = r"\frac{d}{dx}\left(y(x)E(-x)\right)";
const FIRST_PART: &str = r"=y'(x)E(-x)";
const SECOND_PART: &str = r"+y(x)\frac{d}{dx}E(-x)";
const CHAIN_STEP: &str = r"\frac{d}{dx}E(-x)=-E'(-x)=-E(-x)";
const CHAIN_NOTE: &str = "chain rule, then E′ = E";
const TEST_RESULT: &str = r"\frac{d}{dx}\left(y(x)E(-x)\right)=\left(y'(x)-y(x)\right)E(-x)";
const NEVER_ZERO: &str = r"E(-x)\neq 0";
const LEVEL_WHEN: &str = r"\text{level}\iff y'=y";
const PRODUCT_NAME: &str = r"y(x)E(-x)";
const QUESTION: &str = "What must its product do?";

const PLUS_PART: &str = r"+y(x)E(-x)";
const MINUS_PART: &str = r"-y(x)E(-x)";
const CANCEL: &str = r"\frac{d}{dx}\left(y(x)E(-x)\right)=0";
const CONSTANT: &str = "zero slope on an interval: constant";
const AT_ZERO: &str = r"x=0:\quad y(0)E(0)=1\cdot 1=1";
const EQUALS_ONE: &str = r"y(x)E(-x)=1";

const TIMES_E: &str = r"y(x)E(-x)E(x)=E(x)";
const GRID_FACT: &str = r"E(x)E(-x)=1";
const GRID_NOTE: &str = "from the product grid";
const COLLAPSED: &str = r"y(x)=E(x)";
const BOTH_WAYS: &str = r"y'=y,\ y(0)=1\quad\iff\quad y=E";

const SERIES_BOX: &str = r"E(x)=\sum_{k=0}^{\infty}\frac{x^k}{k!}";
const DERIVATIVE_BOX: &str = r"E'=E,\ E(0)=1";
const GRID_BOX: &str = r"E(x)E(-x)=1";
const RESULT_BOX: &str = r"y'=y,\ y(0)=1\ \Rightarrow\ y=E";
const TERM_BY_TERM: &str = "differentiate term by term";
const MULTIPLY_SERIES: &str = "multiply series in the grid";
const TOOLS: &str = "added: product rule, chain rule, zero slope means constant";
const NOT_USED: [&str; 2] = ["uniqueness theorem", "logarithm"];
const NOT_USED_NOTE: &str = "not used:";

const NEEDS_INTERVAL: &str = "one unbroken interval containing the start";
const NEEDS_DERIVATIVE: &str = "a derivative at every point";
const BOTH_PIECES: &str = r"y'=y\ \text{on both pieces}";
const FAR_NAME: &str = r"3E(x)";
const NEAR_NAME: &str = r"E(x)";
const GAP_NAME: &str = "gap";

const RATE_RULE: &str = r"y'=ky";
const RATE_PRODUCT: &str = r"y(t)E(-kt)";
const RATE_TEST: &str = r"\frac{d}{dt}\left(y(t)E(-kt)\right)=\left(y'-ky\right)E(-kt)=0";
const RATE_RESULT: &str = r"y(t)=y(0)E(kt)";
const UNITS: &str = r"k\ \text{in}\ \frac{1}{\text{time}},\ \text{so}\ kt\ \text{is a pure number}";
const QUESTION_FOR_MODEL: &str = "Does a real quantity obey this? Ask each application.";

const DOOR_SERIES: &str = r"\sum_{k=0}^{\infty}\frac{x^k}{k!}";
const DOOR_LIMIT: &str = r"\lim_{n\to\infty}\left(1+\frac{x}{n}\right)^n";
const DOOR_RATE: &str = r"y'=y,\ y(0)=1";
const SAME_OBJECT: &str = r"E(x)=e^x";
const SAME_STRUCTURE: [&str; 3] = [r"E(a+b)=E(a)E(b)", r"E'=E", r"y'=ky"];
const DEBTS: [&str; 3] = [
    "Why adding inputs multiplies outputs",
    "Why the stepped approximations reach E",
    "Why E is the only solution from height one",
];

/// A pair of axes: where they stand, and what they show.
#[derive(Clone, Copy)]
struct Panel {
    left: f64,
    base: f64,
    width: f64,
    height: f64,
    x0: f64,
    x1: f64,
    top: f64,
}
impl Panel {
    fn at(self, x: f64, y: f64) -> Vec3 {
        v(
            self.left + self.width * (x - self.x0) / (self.x1 - self.x0),
            self.base + self.height * y / self.top,
        )
    }
}
const FUNCTION: Panel = Panel {
    left: -6.2,
    base: -2.6,
    width: 5.0,
    height: 3.6,
    x0: -PANEL_RADIUS,
    x1: PANEL_RADIUS,
    top: FUNCTION_TOP,
};
const PRODUCT: Panel = Panel {
    left: 1.0,
    base: -2.6,
    width: 5.0,
    height: 3.6,
    x0: -PANEL_RADIUS,
    x1: PANEL_RADIUS,
    top: PRODUCT_TOP,
};
const PIECES: Panel = Panel {
    left: -6.0,
    base: -2.6,
    width: 6.6,
    height: 3.9,
    x0: FAR_PIECE.0,
    x1: NEAR_PIECE.1,
    top: PIECE_TOP,
};
const RATE_PANEL: Panel = Panel {
    left: 0.8,
    base: -2.6,
    width: 3.9,
    height: 3.9,
    x0: 0.0,
    x1: RATE_END,
    top: RATE_TOP,
};

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
fn clear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
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
fn dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    let marker = Circle::new().radius(0.09).style(ink_style(color)).build().moved_to(at);
    stage.arena_mut().add(marker)
}
fn path(stage: &mut Stage<'_>, points: Vec<Vec3>, color: Srgb, width: f64) -> Mob {
    stage
        .arena_mut()
        .add(polyline(points).style(stroke_style(color, width)).build())
}
/// A filled arrow from a to b, ending on its points exactly.
fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let shape = Arrow::new(a, b)
        .buff(0.0)
        .thickness(3.5)
        .style(Style::default().fill(color, 1.0).stroke(color, 3.5, 1.0))
        .build()?;
    Ok(stage.arena_mut().add(shape))
}
/// y = f(x) sampled across the panel's inputs.
fn curve(stage: &mut Stage<'_>, panel: Panel, f: impl Fn(f64) -> f64, color: Srgb) -> Mob {
    let points: Vec<Vec3> = (0..=PANEL_SAMPLES)
        .map(|j| panel.at(panel_x(j), f(panel_x(j))))
        .collect();
    path(stage, points, color, 3.0)
}
/// Axes for a panel across -PANEL_RADIUS to PANEL_RADIUS, with its height scale, a mark at zero and the start.
fn axes(stage: &mut Stage<'_>, k: &Kit, panel: Panel) -> fmn::Result<Vec<Mob>> {
    let mut parts = vec![
        line(
            stage,
            panel.at(panel.x0, 0.0),
            panel.at(panel.x1, 0.0),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            panel.at(panel.x0, 0.0),
            panel.at(panel.x0, panel.top),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            panel.at(0.0, 0.0),
            panel.at(0.0, panel.top),
            Palette::faint(),
            1.5,
        )?,
        dot(stage, panel.at(0.0, 1.0), Palette::ink()),
        k.text(
            stage,
            "1",
            21.0,
            Palette::dim(),
            v(panel.left - 0.22, panel.at(0.0, 1.0)[1]),
        )?,
        k.text(
            stage,
            &panel.top.to_string(),
            21.0,
            Palette::dim(),
            v(panel.left - 0.22, panel.at(0.0, panel.top)[1]),
        )?,
    ];
    for x in [panel.x0, 0.0, panel.x1] {
        parts.push(k.tex(
            stage,
            &x.to_string(),
            21.0,
            Palette::dim(),
            &[],
            v(panel.at(x, 0.0)[0], panel.base - 0.28),
        )?);
    }
    Ok(parts)
}

/// What 06_1 draws and 06_2 keeps: the function panel, with E and the curve in question.
struct FunctionPanel {
    axes: Vec<Mob>,
    e: Mob,
    e_name: Mob,
    other: Mob,
    other_name: Mob,
}
fn function_panel(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<FunctionPanel> {
    Ok(FunctionPanel {
        axes: axes(stage, k, FUNCTION)?,
        e: curve(stage, FUNCTION, e_value, Palette::blue()),
        e_name: k.tex(stage, "E(x)", 25.0, Palette::blue(), &[], v(-2.35, 0.2))?,
        other: curve(stage, FUNCTION, imagined, Palette::gold()),
        other_name: k.text(stage, "?", 29.0, Palette::gold(), v(-1.0, -0.65))?,
    })
}

/// 06_1: E obeys the rule. The limit does not rule out a second curve, so one is drawn as a question.
fn the_question(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<(FunctionPanel, Mob, Vec<Mob>)> {
    k.say(stage, "06_1")?;
    let title = k.text(stage, TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    let panel = function_panel(stage, k)?;
    let rule = k.tex(stage, RULE, 30.0, Palette::ink(), &[], v(RIGHT_X, 2.2))?;
    let mut shown = panel.axes.clone();
    shown.extend([title, panel.e, panel.e_name, rule]);
    reveal_all(stage, &shown, 0.4)?;

    // A curve from the same start. It breaks the rule on purpose: it stands for the question, and the "?" says so.
    k.until_sentence(stage, "06_1", 1)?;
    reveal_all(stage, &[panel.other, panel.other_name], 0.4)?;

    k.until_sentence(stage, "06_1", 2)?;
    let proved = k.text(stage, PROVED, 25.0, Palette::green(), v(RIGHT_X, 1.05))?;
    reveal(stage, proved)?;

    k.until_sentence(stage, "06_1", 3)?;
    let not_yet = k.text(stage, NOT_YET, 25.0, Palette::gold(), v(RIGHT_X, 0.35))?;
    let debt = k.text(stage, THIRD_DEBT, 25.0, Palette::gold(), v(RIGHT_X, -0.75))?;
    reveal_all(stage, &[not_yet, debt], 0.4)?;
    k.hold(stage)?;
    Ok((panel, title, vec![rule, proved, not_yet, debt]))
}

/// 06_2: the product rule on y(x)E(-x), for any differentiable y. Each part lands on the sentence that names it.
fn the_test(stage: &mut Stage<'_>, k: &Kit, leaving: &[Mob]) -> fmn::Result<(Mob, Vec<Mob>)> {
    k.say(stage, "06_2")?;
    clear(stage, leaving)?;
    k.until_sentence(stage, "06_2", 1)?;
    let test = k.tex(stage, TEST, 32.0, Palette::hero(), &[], v(RIGHT_X, 2.3))?;
    reveal(stage, test)?;

    k.until_sentence(stage, "06_2", 2)?;
    let derivative = k.tex(stage, DIFFERENTIATE, 28.0, Palette::ink(), &[], v(RIGHT_X, 1.35))?;
    reveal(stage, derivative)?;

    k.until_sentence(stage, "06_2", 3)?;
    let first = k.tex(stage, FIRST_PART, 27.0, Palette::blue(), &[], v(2.05, 0.45))?;
    reveal(stage, first)?;

    k.until_sentence(stage, "06_2", 4)?;
    let second = k.tex(stage, SECOND_PART, 27.0, Palette::gold(), &[], v(4.75, 0.45))?;
    let chain = k.tex(stage, CHAIN_STEP, 27.0, Palette::gold(), &[], v(RIGHT_X, -0.55))?;
    let note = k.text(stage, CHAIN_NOTE, 23.0, Palette::dim(), v(RIGHT_X, -1.2))?;
    reveal_all(stage, &[second, chain, note], 0.4)?;

    k.until_sentence(stage, "06_2", 5)?;
    let result = k.tex(stage, TEST_RESULT, 25.0, Palette::hero(), &[], v(RIGHT_X, -2.2))?;
    reveal(stage, result)?;
    k.hold(stage)?;
    Ok((result, vec![test, derivative, first, second, chain, note]))
}

/// The test's name above the product panel.
fn product_axes(stage: &mut Stage<'_>, k: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut parts = axes(stage, k, PRODUCT)?;
    parts.push(k.tex(
        stage,
        PRODUCT_NAME,
        25.0,
        Palette::dim(),
        &[],
        v(PRODUCT.at(0.9, 0.0)[0], PRODUCT.at(0.0, PRODUCT.top)[1] + 0.25),
    )?);
    Ok(parts)
}

/// What 06_3 leaves for 06_4: the left side and the two tilted products leave, and the product axes and the rule stay.
struct Failures {
    left: Vec<Mob>,
    products: Vec<Mob>,
    product_axes: Vec<Mob>,
    rule: Mob,
}

/// 06_3: two functions that fail, side by side with their products. The tilt is what the test reads.
fn test_on_failures(
    stage: &mut Stage<'_>,
    k: &Kit,
    panel: &FunctionPanel,
    result: Mob,
    leaving: &[Mob],
) -> fmn::Result<Failures> {
    k.say(stage, "06_3")?;
    let mut gone = leaving.to_vec();
    gone.extend([panel.e, panel.e_name, panel.other, panel.other_name]);
    clear(stage, &gone)?;
    let up = result
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.4),
            ..AnimateArgs::default()
        })?
        .move_to(v(0.0, 2.45), ORIGIN)?;
    let up = stage.prepare(up)?;
    stage.play_prepared(vec![up])?;
    let product_parts = product_axes(stage, k)?;
    reveal_all(stage, &product_parts, 0.4)?;

    k.until_sentence(stage, "06_3", 1)?;
    let low = curve(stage, FUNCTION, |x| polynomial(x, DEGREES[0]), Palette::gold());
    let low_name = k.text(stage, "degree 2", 23.0, Palette::gold(), v(-2.0, -1.7))?;
    reveal_all(stage, &[low, low_name], 0.4)?;

    k.until_sentence(stage, "06_3", 2)?;
    let low_product = curve(stage, PRODUCT, |x| product(x, DEGREES[0]), Palette::gold());
    let low_slope = k.tex(stage, &defect_tex(DEGREES[0]), 24.0, Palette::gold(), &[], v(2.3, -2.1))?;
    reveal_all(stage, &[low_product, low_slope], 0.4)?;

    k.until_sentence(stage, "06_3", 3)?;
    let high = curve(stage, FUNCTION, |x| polynomial(x, DEGREES[1]), Palette::teal());
    let high_name = k.text(stage, "degree 4", 23.0, Palette::teal(), v(-2.7, -0.35))?;
    let high_product = curve(stage, PRODUCT, |x| product(x, DEGREES[1]), Palette::teal());
    let high_slope = k.tex(stage, &defect_tex(DEGREES[1]), 24.0, Palette::teal(), &[], v(4.7, -2.1))?;
    reveal_all(stage, &[high, high_name, high_product, high_slope], 0.4)?;

    k.until_sentence(stage, "06_3", 4)?;
    let never = k.tex(stage, NEVER_ZERO, 25.0, Palette::ink(), &[], v(-4.8, 1.75))?;
    let level = k.tex(stage, LEVEL_WHEN, 25.0, Palette::ink(), &[], v(-2.2, 1.75))?;
    reveal_all(stage, &[never, level], 0.35)?;

    k.until_sentence(stage, "06_3", 5)?;
    let rule = k.tex(stage, RULE, 27.0, Palette::hero(), &[], v(3.5, 1.75))?;
    reveal(stage, rule)?;

    // The answer waits for the think gap.
    k.until_sentence(stage, "06_3", 6)?;
    let question = k.text(stage, QUESTION, 27.0, Palette::hero(), v(0.0, -3.5))?;
    reveal(stage, question)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, 4.0)?;
    clear(stage, &[question])?;
    let mut left = panel.axes.clone();
    left.extend([
        low, low_name, high, high_name, never, level, low_slope, high_slope, result,
    ]);
    Ok(Failures {
        left,
        products: vec![low_product, high_product],
        product_axes: product_parts,
        rule,
    })
}

/// 06_4: under the rule the two parts cancel, the product is level, and its height at zero is one.
fn it_stays_level(stage: &mut Stage<'_>, k: &Kit, failures: &Failures) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "06_4")?;
    let level_line = line(
        stage,
        PRODUCT.at(-PANEL_RADIUS, 1.0),
        PRODUCT.at(PANEL_RADIUS, 1.0),
        Palette::hero(),
        3.5,
    )?;
    let rule = failures.rule;
    let to_left = rule
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.4),
            ..AnimateArgs::default()
        })?
        .move_to(v(LEFT_X, 2.3), ORIGIN)?;
    let mut moves: Vec<Box<dyn Animation>> = vec![stage.prepare(to_left)?];
    for &m in failures.left.iter().chain(&failures.products) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        moves.push(stage.prepare(a.rt(0.4))?);
    }
    let shown = fade_in(stage.arena_mut(), level_line, ORIGIN, 1.0)?;
    moves.push(stage.prepare(shown.rt(0.4))?);
    stage.play_prepared(moves)?;

    // One contribution up, the other down, at one point of the level line. They have the same length.
    k.until_sentence(stage, "06_4", 1)?;
    // Left of the zero line, so the labels do not sit across it.
    let at = PRODUCT.at(-1.05, 1.0);
    let rise = 1.05;
    let plus = arrow(stage, at, v(at[0], at[1] + rise), Palette::green())?;
    let plus_name = k.tex(
        stage,
        PLUS_PART,
        24.0,
        Palette::green(),
        &[],
        v(at[0] + 0.85, at[1] + rise - 0.1),
    )?;
    let plus_left = k.tex(stage, PLUS_PART, 28.0, Palette::green(), &[], v(LEFT_X - 1.3, 1.2))?;
    reveal_all(stage, &[plus, plus_name, plus_left], 0.35)?;
    until_phrase(k, stage, "06_4", "and the other")?;
    let minus = arrow(stage, at, v(at[0], at[1] - rise), Palette::red())?;
    let minus_name = k.tex(
        stage,
        MINUS_PART,
        24.0,
        Palette::red(),
        &[],
        v(at[0] + 0.85, at[1] - rise + 0.1),
    )?;
    let minus_left = k.tex(stage, MINUS_PART, 28.0, Palette::red(), &[], v(LEFT_X + 1.3, 1.2))?;
    reveal_all(stage, &[minus, minus_name, minus_left], 0.35)?;

    k.until_sentence(stage, "06_4", 2)?;
    let cancel = k.tex(stage, CANCEL, 28.0, Palette::hero(), &[], v(LEFT_X, 0.2))?;
    let mut change: Vec<Box<dyn Animation>> = Vec::new();
    for m in [plus, minus, plus_name, minus_name] {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        change.push(stage.prepare(a.rt(0.5))?);
    }
    let shown = fade_in(stage.arena_mut(), cancel, ORIGIN, 1.0)?;
    change.push(stage.prepare(shown.rt(0.5))?);
    stage.play_prepared(change)?;

    k.until_sentence(stage, "06_4", 3)?;
    let constant = k.text(stage, CONSTANT, 25.0, Palette::ink(), v(LEFT_X, -0.6))?;
    reveal(stage, constant)?;
    until_phrase(k, stage, "06_4", "and at zero")?;
    let start = dot(stage, PRODUCT.at(0.0, 1.0), Palette::hero());
    let at_zero = k.tex(stage, AT_ZERO, 27.0, Palette::ink(), &[], v(LEFT_X, -1.3))?;
    reveal_all(stage, &[start, at_zero], 0.35)?;

    k.until_sentence(stage, "06_4", 4)?;
    let equals = k.tex(stage, EQUALS_ONE, 34.0, Palette::hero(), &[], v(LEFT_X, -2.4))?;
    reveal(stage, equals)?;
    k.hold(stage)?;
    let mut working = failures.product_axes.clone();
    working.extend([
        level_line, plus_left, minus_left, cancel, constant, start, at_zero, equals, rule,
    ]);
    Ok(working)
}

/// 06_5: multiply through by E(x). The curve in question lands on E, which is the rule in both directions.
fn multiply_through(stage: &mut Stage<'_>, k: &Kit, leaving: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "06_5")?;
    clear(stage, leaving)?;
    let panel = function_panel(stage, k)?;
    let mut shown = panel.axes.clone();
    shown.extend([panel.e, panel.e_name, panel.other, panel.other_name]);
    let start = k.tex(stage, EQUALS_ONE, 30.0, Palette::hero(), &[], v(RIGHT_X, 2.3))?;
    let times = k.tex(stage, TIMES_E, 28.0, Palette::ink(), &[], v(RIGHT_X, 1.45))?;
    shown.extend([start, times]);
    reveal_all(stage, &shown, 0.4)?;

    k.until_sentence(stage, "06_5", 1)?;
    let grid = k.tex(stage, GRID_FACT, 28.0, Palette::green(), &[], v(RIGHT_X, 0.65))?;
    let note = k.text(stage, GRID_NOTE, 23.0, Palette::dim(), v(RIGHT_X, 0.15))?;
    reveal_all(stage, &[grid, note], 0.35)?;
    until_phrase(k, stage, "06_5", "so the left side")?;
    let collapsed = k.tex(stage, COLLAPSED, 32.0, Palette::hero(), &[], v(RIGHT_X, -0.65))?;
    reveal(stage, collapsed)?;

    // The curve in question lands on E. The "?" has nothing left to ask.
    k.until_sentence(stage, "06_5", 2)?;
    let on_e = curve(stage, FUNCTION, e_value, Palette::gold());
    let landed = Transform::new(panel.other, on_e);
    let asked = fade_out(stage.arena_mut(), panel.other_name, ORIGIN, 1.0)?;
    let moves = vec![stage.prepare(landed.rt(1.0))?, stage.prepare(asked.rt(1.0))?];
    stage.play_prepared(moves)?;

    k.until_sentence(stage, "06_5", 3)?;
    let both = k.tex(stage, BOTH_WAYS, 29.0, Palette::green(), &[], v(RIGHT_X, -1.75))?;
    reveal(stage, both)?;

    k.until_sentence(stage, "06_5", 4)?;
    let pulse = fmn::animation::indicate(stage.arena_mut(), both, 1.06, None)?;
    stage.play(pulse.rt(0.9))?;
    k.hold(stage)?;
    let mut screen = panel.axes;
    screen.extend([
        panel.e,
        panel.e_name,
        panel.other,
        start,
        times,
        grid,
        note,
        collapsed,
        both,
    ]);
    Ok(screen)
}

/// A box around a formula.
fn boxed(stage: &mut Stage<'_>, at: Vec3, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    let r = rect(
        v(at[0] - width / 2.0, at[1] - height / 2.0),
        width,
        height,
        stroke_style(color, 2.0),
    )?;
    Ok(stage.arena_mut().add(r))
}

/// 06_6: the ledger. Every arrow runs one way, and the two things the chapter did not use are struck out.
fn no_circle(stage: &mut Stage<'_>, k: &Kit, leaving: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "06_6")?;
    clear(stage, leaving)?;
    let title = k.text(stage, LEDGER_TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    reveal(stage, title)?;
    let series_at = v(-4.9, 0.4);
    let derivative_at = v(-0.9, 1.55);
    let grid_at = v(-0.9, -0.75);
    let result_at = v(4.0, 0.4);
    k.until_sentence(stage, "06_6", 1)?;
    let series_box = boxed(stage, series_at, 2.9, 1.2, Palette::blue())?;
    let series = k.tex(stage, SERIES_BOX, 25.0, Palette::blue(), &[], series_at)?;
    reveal_all(stage, &[series_box, series], 0.4)?;

    k.until_sentence(stage, "06_6", 2)?;
    let derivative_box = boxed(stage, derivative_at, 2.9, 0.9, Palette::gold())?;
    let derivative = k.tex(stage, DERIVATIVE_BOX, 25.0, Palette::gold(), &[], derivative_at)?;
    let first = arrow(stage, v(-3.45, 0.75), v(-2.35, 1.3), Palette::gold())?;
    let first_note = k.text(stage, TERM_BY_TERM, 21.0, Palette::dim(), v(-4.3, 1.75))?;
    reveal_all(stage, &[derivative_box, derivative, first, first_note], 0.4)?;

    k.until_sentence(stage, "06_6", 3)?;
    let grid_box = boxed(stage, grid_at, 2.9, 0.9, Palette::green())?;
    let grid = k.tex(stage, GRID_BOX, 25.0, Palette::green(), &[], grid_at)?;
    let second = arrow(stage, v(-3.45, 0.05), v(-2.35, -0.5), Palette::green())?;
    let second_note = k.text(stage, MULTIPLY_SERIES, 21.0, Palette::dim(), v(-4.3, -0.95))?;
    reveal_all(stage, &[grid_box, grid, second, second_note], 0.4)?;

    // The result joins the ledger only here, at the far end. Nothing runs back from it.
    k.until_sentence(stage, "06_6", 4)?;
    let result_box = boxed(stage, result_at, 4.0, 0.9, Palette::hero())?;
    let result = k.tex(stage, RESULT_BOX, 25.0, Palette::hero(), &[], result_at)?;
    reveal_all(stage, &[result_box, result], 0.4)?;

    k.until_sentence(stage, "06_6", 5)?;
    let into_top = arrow(stage, v(0.55, 1.3), v(2.0, 0.75), Palette::ink())?;
    let into_low = arrow(stage, v(0.55, -0.5), v(2.0, 0.05), Palette::ink())?;
    let tools = k.text(stage, TOOLS, 23.0, Palette::ink(), v(0.0, -2.0))?;
    reveal_all(stage, &[into_top, into_low, tools], 0.4)?;

    k.until_sentence(stage, "06_6", 6)?;
    let note = k.text(stage, NOT_USED_NOTE, 23.0, Palette::dim(), v(-4.1, -2.7))?;
    let mut struck = vec![note];
    for (j, name) in NOT_USED.iter().enumerate() {
        let x = -1.9 + 3.9 * j as f64;
        struck.push(k.text(stage, name, 25.0, Palette::dim(), v(x, -2.7))?);
        let half = if j == 0 { 1.15 } else { 0.62 };
        let strike = line(stage, v(x - half, -2.7), v(x + half, -2.7), Palette::red(), 2.5)?;
        k.allow_crossing(strike);
        struck.push(strike);
    }
    reveal_all(stage, &struck, 0.4)?;

    k.until_sentence(stage, "06_6", 7)?;
    let arrows = [first, second, into_top, into_low];
    let mut lit: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &arrows {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.1, None)?;
        lit.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(lit)?;
    k.hold(stage)?;
    let mut screen = vec![
        title,
        series_box,
        series,
        derivative_box,
        derivative,
        first,
        first_note,
        grid_box,
        grid,
        second,
        second_note,
        result_box,
        result,
        into_top,
        into_low,
        tools,
    ];
    screen.extend(struck);
    Ok(screen)
}

/// 06_7: why the word "interval" is in the theorem. A gap lets a second multiple of E through.
fn the_gap(stage: &mut Stage<'_>, k: &Kit, ledger: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "06_7")?;
    clear(stage, ledger)?;
    let title = k.text(stage, GAP_TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    let mut parts = vec![
        title,
        line(
            stage,
            PIECES.at(PIECES.x0, 0.0),
            PIECES.at(PIECES.x1, 0.0),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            PIECES.at(PIECES.x0, 0.0),
            PIECES.at(PIECES.x0, PIECES.top),
            Palette::dim(),
            2.0,
        )?,
        k.text(
            stage,
            &PIECES.top.to_string(),
            21.0,
            Palette::dim(),
            v(PIECES.left - 0.22, PIECES.at(0.0, PIECES.top)[1]),
        )?,
    ];
    for x in [PIECES.x0, -1.0, 0.0, PIECES.x1] {
        parts.push(k.tex(
            stage,
            &x.to_string(),
            21.0,
            Palette::dim(),
            &[],
            v(PIECES.at(x, 0.0)[0], PIECES.base - 0.28),
        )?);
    }
    let near_points: Vec<Vec3> = (0..=PIECE_SAMPLES)
        .map(|j| {
            let (x, y) = near_piece(j);
            PIECES.at(x, y)
        })
        .collect();
    let near = path(stage, near_points, Palette::blue(), 3.5);
    let near_name = k.tex(
        stage,
        NEAR_NAME,
        25.0,
        Palette::blue(),
        &[],
        v(PIECES.at(0.55, 0.0)[0], PIECES.at(0.0, 0.9)[1]),
    )?;
    let start = dot(stage, PIECES.at(0.0, 1.0), Palette::ink());
    let mut shown = parts.clone();
    shown.extend([near, near_name, start]);
    reveal_all(stage, &shown, 0.4)?;

    k.until_sentence(stage, "06_7", 1)?;
    let far_points: Vec<Vec3> = (0..=PIECE_SAMPLES)
        .map(|j| {
            let (x, y) = far_piece(j);
            PIECES.at(x, y)
        })
        .collect();
    let far = path(stage, far_points, Palette::gold(), 3.5);
    let far_name = k.tex(
        stage,
        FAR_NAME,
        25.0,
        Palette::gold(),
        &[],
        v(PIECES.at(-1.3, 0.0)[0], -0.9),
    )?;
    let band = rect(
        v(PIECES.at(FAR_PIECE.1, 0.0)[0], PIECES.base),
        PIECES.at(NEAR_PIECE.0, 0.0)[0] - PIECES.at(FAR_PIECE.1, 0.0)[0],
        PIECES.height,
        Style::default()
            .fill(Palette::faint(), 0.35)
            .stroke(Palette::faint(), 0.0, 1.0),
    )?;
    let band = stage.arena_mut().add(band);
    k.allow_crossing(band);
    let gap_name = k.text(
        stage,
        GAP_NAME,
        21.0,
        Palette::dim(),
        v(
            (PIECES.at(FAR_PIECE.1, 0.0)[0] + PIECES.at(NEAR_PIECE.0, 0.0)[0]) / 2.0,
            PIECES.base + PIECES.height + 0.2,
        ),
    )?;
    reveal_all(stage, &[band, far, far_name, gap_name], 0.4)?;

    // The test on each piece: level, at two different heights. Same rule, and no single constant.
    k.until_sentence(stage, "06_7", 2)?;
    let obeys = k.tex(stage, BOTH_PIECES, 26.0, Palette::ink(), &[], v(RIGHT_X, 2.3))?;
    let far_read = k.tex(
        stage,
        &format!(r"3E(x)\cdot E(-x)={FAR_MULTIPLE}"),
        26.0,
        Palette::gold(),
        &[],
        v(RIGHT_X, 1.6),
    )?;
    let near_read = k.tex(
        stage,
        r"E(x)\cdot E(-x)=1",
        26.0,
        Palette::blue(),
        &[],
        v(RIGHT_X, 0.85),
    )?;
    reveal_all(stage, &[obeys, far_read, near_read], 0.35)?;

    k.until_sentence(stage, "06_7", 3)?;
    let needs_interval = k.text(stage, NEEDS_INTERVAL, 25.0, Palette::ink(), v(RIGHT_X, -0.35))?;
    reveal(stage, needs_interval)?;
    until_phrase(k, stage, "06_7", "with a derivative")?;
    let needs_derivative = k.text(stage, NEEDS_DERIVATIVE, 25.0, Palette::ink(), v(RIGHT_X, -1.0))?;
    reveal(stage, needs_derivative)?;
    k.hold(stage)?;
    let mut screen = parts;
    screen.extend([
        near, near_name, start, far, far_name, band, gap_name, obeys, far_read, near_read,
    ]);
    screen.extend([needs_interval, needs_derivative]);
    Ok(screen)
}

/// 06_8: the same test with a constant rate. Growth, decay, the flat line and the zero start are all one formula.
fn any_constant_rate(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "06_8")?;
    clear(stage, screen)?;
    let title = k.text(stage, RATE_TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    reveal(stage, title)?;
    k.until_sentence(stage, "06_8", 1)?;
    let rule = k.tex(stage, RATE_RULE, 32.0, Palette::ink(), &[], v(LEFT_X, 2.3))?;
    reveal(stage, rule)?;

    let multiplied = k.tex(stage, RATE_PRODUCT, 28.0, Palette::ink(), &[], v(LEFT_X, 1.65))?;
    reveal(stage, multiplied)?;

    // The cancellation is what this sentence says, so the zero waits for it.
    k.until_sentence(stage, "06_8", 2)?;
    let test = k.tex(stage, RATE_TEST, 24.0, Palette::gold(), &[], v(LEFT_X - 0.1, 0.95))?;
    reveal(stage, test)?;
    until_phrase(k, stage, "06_8", "so y of t")?;
    let result = k.tex(stage, RATE_RESULT, 32.0, Palette::hero(), &[], v(LEFT_X, 0.1))?;
    reveal(stage, result)?;

    k.until_sentence(stage, "06_8", 3)?;
    let mut parts = vec![
        line(
            stage,
            RATE_PANEL.at(0.0, 0.0),
            RATE_PANEL.at(RATE_END, 0.0),
            Palette::dim(),
            2.0,
        )?,
        line(
            stage,
            RATE_PANEL.at(0.0, 0.0),
            RATE_PANEL.at(0.0, RATE_PANEL.top),
            Palette::dim(),
            2.0,
        )?,
        k.text(
            stage,
            &RATE_START.to_string(),
            21.0,
            Palette::dim(),
            v(RATE_PANEL.left - 0.22, RATE_PANEL.at(0.0, RATE_START)[1]),
        )?,
        k.tex(
            stage,
            &format!(r"t={RATE_END}"),
            21.0,
            Palette::dim(),
            &[],
            v(RATE_PANEL.at(RATE_END, 0.0)[0], RATE_PANEL.base - 0.28),
        )?,
        dot(stage, RATE_PANEL.at(0.0, RATE_START), Palette::ink()),
    ];
    let colors = [Palette::green(), Palette::ink(), Palette::red()];
    let mut curves = Vec::new();
    for (&(rate, name), color) in RATES.iter().zip(colors) {
        let points: Vec<Vec3> = (0..=RATE_SAMPLES)
            .map(|j| RATE_PANEL.at(rate_time(j), rate_solution(rate, RATE_START, rate_time(j))))
            .collect();
        curves.push(path(stage, points, color, 3.0));
        let end = RATE_PANEL.at(RATE_END, rate_solution(rate, RATE_START, RATE_END));
        parts.push(k.tex(
            stage,
            &format!(r"k={name}:\ {}", rate_label(rate)),
            22.0,
            color,
            &[],
            v(end[0] + 1.15, end[1]),
        )?);
    }
    parts.push(line(
        stage,
        RATE_PANEL.at(0.0, 0.0),
        RATE_PANEL.at(RATE_END, 0.0),
        Palette::dim(),
        4.0,
    )?);
    parts.push(k.tex(
        stage,
        r"y(0)=0:\ y=0",
        23.0,
        Palette::dim(),
        &[],
        v(RATE_PANEL.at(1.0, 0.0)[0], RATE_PANEL.base + 0.22),
    )?);
    let mut grown: Vec<Box<dyn Animation>> = Vec::new();
    for &m in &curves {
        grown.push(stage.prepare(show_creation(m).rt(0.9))?);
    }
    for &m in &parts {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        grown.push(stage.prepare(a.rt(0.9))?);
    }
    stage.play_prepared(grown)?;

    k.until_sentence(stage, "06_8", 4)?;
    let units = k.tex(stage, UNITS, 25.0, Palette::ink(), &[], v(LEFT_X, -1.05))?;
    reveal(stage, units)?;

    k.until_sentence(stage, "06_8", 5)?;
    let model = k.text(stage, QUESTION_FOR_MODEL, 25.0, Palette::gold(), v(0.0, -3.5))?;
    reveal(stage, model)?;
    k.hold(stage)?;
    let mut all = parts;
    all.extend(curves);
    all.extend([title, rule, multiplied, test, result, units, model]);
    Ok(all)
}

/// 06_9: three doors, one object, and the promises from the series chapter in the words they were made in.
fn three_doors(stage: &mut Stage<'_>, k: &Kit, screen: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "06_9")?;
    clear(stage, screen)?;
    let title = k.text(stage, DOORS_TITLE, 35.0, Palette::ink(), v(0.0, TITLE_Y))?;
    reveal(stage, title)?;
    let middle = v(0.0, -0.1);
    let doors = [
        (DOOR_SERIES, -4.6, Palette::blue()),
        (DOOR_LIMIT, 0.0, Palette::gold()),
        (DOOR_RATE, 4.6, Palette::teal()),
    ];
    let mut parts = Vec::new();
    let mut arrows = Vec::new();
    for (tex, x, color) in doors {
        let at = v(x, 1.75);
        parts.push(boxed(stage, at, 3.5, 1.2, color)?);
        parts.push(k.tex(stage, tex, 25.0, color, &[], at)?);
        arrows.push(arrow(stage, v(x * 0.92, 1.1), v(x * 0.2, 0.3), color)?);
    }
    reveal_all(stage, &parts, 0.5)?;
    reveal_all(stage, &arrows, 0.5)?;
    let same = k.tex(stage, SAME_OBJECT, 42.0, Palette::hero(), &[], middle)?;
    reveal(stage, same)?;

    // Powers, derivatives and the growth rule, in the order the sentence names them.
    k.until_sentence(stage, "06_9", 1)?;
    let mut fits = Vec::new();
    for (tex, x) in SAME_STRUCTURE.into_iter().zip([-4.2, 0.0, 4.2]) {
        fits.push(k.tex(stage, tex, 25.0, Palette::ink(), &[], v(x, -0.85))?);
    }
    reveal_all(stage, &fits, 0.5)?;

    k.until_sentence(stage, "06_9", 2)?;
    let mut debts = Vec::new();
    for (row, debt) in DEBTS.iter().enumerate() {
        debts.push(k.text(stage, debt, 25.0, Palette::green(), v(0.0, -1.6 - 0.6 * row as f64))?);
    }
    reveal_all(stage, &debts, 0.5)?;
    Ok(())
}

impl SceneConstruct for OnlySolution {
    fn name(&self) -> &str {
        "06_only_solution"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (panel, title, opening) = the_question(stage, &k)?;
        let (result, derivation) = the_test(stage, &k, &opening)?;
        let failures = test_on_failures(stage, &k, &panel, result, &derivation)?;
        let working = it_stays_level(stage, &k, &failures)?;
        let mut screen = multiply_through(stage, &k, &working)?;
        screen.push(title);
        let ledger = no_circle(stage, &k, &screen)?;
        let gap = the_gap(stage, &k, &ledger)?;
        let rate = any_constant_rate(stage, &k, &gap)?;
        three_doors(stage, &k, &rate)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// The scene's code, without this test module.
    fn scene() -> &'static str {
        include_str!("only_solution.rs")
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
    fn the_cues_run_in_film_order() {
        assert_in_order(&[
            r#"k.say(stage, "06_1")?;"#,
            r#"k.say(stage, "06_2")?;"#,
            r#"k.say(stage, "06_3")?;"#,
            r#"k.say(stage, "06_4")?;"#,
            r#"k.say(stage, "06_5")?;"#,
            r#"k.say(stage, "06_6")?;"#,
            r#"k.say(stage, "06_7")?;"#,
            r#"k.say(stage, "06_8")?;"#,
            r#"k.say(stage, "06_9")?;"#,
        ]);
    }
    /// A cue's words, from the script the phrase hook reads at run time.
    fn spoken(cue: &str) -> &'static str {
        let prefix = format!("{cue}\t");
        crate::phrase::SCRIPT
            .lines()
            .find_map(|l| l.strip_prefix(prefix.as_str()))
            .unwrap_or_else(|| panic!("{cue} is not in the script"))
    }
    #[test]
    fn every_phrase_hook_is_in_its_own_line() {
        for (cue, phrase) in [
            ("06_4", "and the other"),
            ("06_4", "and at zero"),
            ("06_5", "so the left side"),
            ("06_7", "with a derivative"),
            ("06_8", "so y of t"),
        ] {
            assert!(spoken(cue).contains(phrase), "{cue}: {phrase}");
            assert!(scene().contains(&format!(r#"until_phrase(k, stage, "{cue}", "{phrase}")?;"#)));
        }
    }
    #[test]
    fn the_test_is_built_for_any_y_before_the_rule_is_applied() {
        // The rule comes off the screen at 06_2 and returns at 06_3 sentence 5, after the failures.
        assert_in_order(&[
            r#"k.say(stage, "06_2")?;"#,
            "clear(stage, leaving)?;",
            "k.tex(stage, TEST,",
            "k.tex(stage, TEST_RESULT",
            r#"k.say(stage, "06_3")?;"#,
            r#"until_sentence(stage, "06_3", 1)"#,
            "defect_tex(DEGREES[0])",
            "defect_tex(DEGREES[1])",
            r#"until_sentence(stage, "06_3", 5)"#,
            "k.tex(stage, RULE",
        ]);
    }
    #[test]
    fn the_think_gap_is_authored_and_the_answer_waits() {
        assert_in_order(&[
            "k.text(stage, QUESTION,",
            "reveal(stage, question)?;",
            "k.hold(stage)?;",
            r#"k.mark(stage, "predict");"#,
            "k.music_breath(stage, 4.0)?;",
            "clear(stage, &[question])?;",
            r#"k.say(stage, "06_4")?;"#,
            "Palette::hero(),\n        3.5,",
        ]);
    }
    #[test]
    fn the_cancellation_is_shown_before_it_is_used() {
        assert_in_order(&[
            r#"k.say(stage, "06_4")?;"#,
            r#"until_sentence(stage, "06_4", 1)"#,
            "PLUS_PART",
            r#"until_phrase(k, stage, "06_4", "and the other")?;"#,
            "MINUS_PART",
            r#"until_sentence(stage, "06_4", 2)"#,
            "k.tex(stage, CANCEL",
            r#"until_sentence(stage, "06_4", 3)"#,
            "k.text(stage, CONSTANT",
            r#"until_phrase(k, stage, "06_4", "and at zero")?;"#,
            "k.tex(stage, AT_ZERO",
            r#"until_sentence(stage, "06_4", 4)"#,
            "k.tex(stage, EQUALS_ONE",
            r#"k.say(stage, "06_5")?;"#,
            "k.tex(stage, TIMES_E",
            r#"until_sentence(stage, "06_5", 1)"#,
            "k.tex(stage, GRID_FACT",
            r#"until_phrase(k, stage, "06_5", "so the left side")?;"#,
            "k.tex(stage, COLLAPSED",
        ]);
    }
    #[test]
    fn the_result_enters_the_ledger_last_and_nothing_runs_back_from_it() {
        assert_in_order(&[
            r#"k.say(stage, "06_6")?;"#,
            "SERIES_BOX",
            "DERIVATIVE_BOX",
            "GRID_BOX",
            r#"until_sentence(stage, "06_6", 4)"#,
            "RESULT_BOX",
            r#"until_sentence(stage, "06_6", 5)"#,
            "TOOLS",
            r#"until_sentence(stage, "06_6", 6)"#,
            "NOT_USED.iter()",
        ]);
        assert_eq!(NOT_USED, ["uniqueness theorem", "logarithm"]);
    }
    #[test]
    fn the_debts_keep_the_words_the_series_chapter_wrote() {
        assert_eq!(DEBTS[0], "Why adding inputs multiplies outputs");
        assert_eq!(DEBTS[1], "Why the stepped approximations reach E");
        assert_eq!(DEBTS[2], THIRD_DEBT);
    }
    #[test]
    fn the_assumptions_are_on_screen_and_the_model_question_stays_open() {
        assert_in_order(&[
            "NEEDS_INTERVAL",
            "NEEDS_DERIVATIVE",
            "RATE_RESULT",
            "UNITS",
            "QUESTION_FOR_MODEL",
        ]);
    }
}
