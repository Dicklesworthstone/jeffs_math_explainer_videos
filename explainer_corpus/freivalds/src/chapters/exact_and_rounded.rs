//! Chapter 04, exact and rounded. At a thousand rows twenty fresh checks
//! take twenty million multiplications, a fiftieth of the billion to redo
//! the product: a count of work, not of seconds (04_1). But many programs
//! compute with doubles: what is 10^16 + 1 in doubles? (04_2, a
//! prediction.) 10^16: the doubles there are two apart, so the one is lost
//! (04_3). So a right answer can be caught, its toss with both coins heads
//! losing two ones (04_4), and a wrong answer by one passes all four tosses
//! (04_5). A tolerance of the right one's largest gap in doubles lets it
//! through, but then a wrong C inside it passes every toss, with no half
//! guarantee (04_6). The promise is for exact arithmetic, on the opening
//! picture (04_7).

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{
    cell_box, coin_mobs, fade_in_all, fade_in_lagged, fade_out_all, fade_to, fail, flat, fly, grid_colored, line_style,
    matrix_grid, ring, say, stand, tex_grid, think, together, toss_coins, wait_until,
};
use crate::facts::{self, AGAIN, BIG_SIDE, RoundedToss};
use crate::model::{self, A, B, BIG, C, Matrix, ROUNDED_PASSES, ROUNDED_REJECTS, SIDE};
use crate::palette;

pub struct ExactAndRounded {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const PROGRAMMERS: &str = "for programmers:";
const COST_NOTE: &str = "these count multiplications, not time. Adding up the heads columns of B and C still \
                         takes additions, and every entry is read from memory.";
const DOUBLES: &str = "doubles";
const EXACT: &str = "exact";
const MATCH: &str = "match";
const NO_MATCH: &str = "no match";
const NAMES: [&str; 3] = ["A", "B", "C"];
const MISTAKE_NAME: &str = r"C - AB";
const TIMES: &str = r"\times";
const EQUALS: &str = "=";
const RIGHT_LINE: &str = "a right C: every toss matches";
const WRONG_LINE: &str = "a wrong C: each fresh toss misses at most half the time";
const BOTH_FAIL: &str = "in doubles, both can fail";
/// The one added to N = 10^16, which doubles lose (04_2, 04_3).
const PLUS: i64 = 1;
/// The whole numbers around N on the number line, as offsets from N.
const OFFSETS: std::ops::RangeInclusive<i64> = -2..=2;

// ------------------------------------------------------------------ layout

/// 04_1: n at the top, then the two bars to scale, each under its count.
const COST_N_AT: Vec3 = [0.0, 3.05, 0.0];
const BAR_X: f64 = -5.0;
const BAR_W: f64 = 10.0;
const BAR_H: f64 = 0.32;
const REDO_LABEL_Y: f64 = 2.1;
const REDO_BAR_Y: f64 = 1.55;
const CHECK_LABEL_Y: f64 = 0.8;
const CHECK_BAR_Y: f64 = 0.25;
const RATIO_AT: Vec3 = [0.0, -0.95, 0.0];
/// 04_2, 04_3: N's name at the top left (until 04_6), the number line
/// around N, its doubles, and the question under it.
const N_NOTE_AT: Vec3 = [-5.6, 3.3, 0.0];
const LINE_Y: f64 = 1.6;
const LINE_DX: f64 = 2.4;
const LINE_HALF: f64 = 5.6;
const TICK_H: f64 = 0.24;
const LINE_LABEL_DY: f64 = -0.55;
const DOT_R: f64 = 0.11;
const GAP_LABEL_DY: f64 = 0.45;
const LEGEND_AT: Vec3 = [-5.0, 2.45, 0.0];
const QUESTION_AT: Vec3 = [-1.2, -0.6, 0.0];
const ANSWER_DY: f64 = -0.6;
const THINK_AT: Vec3 = [5.0, -0.6, 0.0];
/// 04_4 to 04_6: A, B, C and the mistake C − AB, two by two; the four
/// tosses of two coins; the verdict rows, exact and in doubles; the work.
const GRIDS_Y: f64 = 2.2;
const GRID_XS: [f64; 4] = [-5.3, -3.1, -0.9, 1.9];
const GRID_NAME_Y: f64 = 1.2;
const TOSS_XS: [f64; 4] = [-3.6, -1.2, 1.2, 3.6];
const COIN_Y: f64 = 0.3;
const COIN_PITCH: f64 = 0.5;
const SHOWN_R: f64 = 0.5;
const ROW_LABEL_X: f64 = -6.5;
const ROW_YS: [f64; 2] = [-0.45, -1.0];
const WORK_YS: [f64; 2] = [-1.7, -2.25];
/// The equality rows, dimmed while the tolerant test stands under them (04_6).
const EQUALITY_DIM: f64 = 0.3;
/// 04_7: the opening picture, as chapter 01 drew it, and the promise.
const OPEN_XS: [f64; 3] = [-5.0, -1.95, 1.1];
const OPEN_Y: f64 = 2.2;
const OPEN_NAME_Y: f64 = 0.95;
const OPEN_TIMES_X: f64 = -3.47;
const OPEN_EQUALS_X: f64 = -0.42;
const PROMISE_X: f64 = -1.0;
const PROMISE_YS: [f64; 4] = [-0.15, -0.7, -1.3, -1.95];

/// Where offset `k` from N stands on the number line.
fn line_x(k: i64) -> f64 {
    k as f64 * LINE_DX
}

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, a quick reader (the video can be paused).
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// Scene time now.
fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

/// A number with thousands separators, as TeX writes them.
fn tex_thousands(n: u64) -> String {
    thousands(n).replace(',', "{,}")
}

/// A ratio as `facts::ratio` prints it, "1/50", as TeX.
fn tex_of(ratio: &str) -> String {
    match ratio.split_once('/') {
        Some((top, bottom)) => format!(r"\frac{{{top}}}{{{bottom}}}"),
        None => ratio.to_owned(),
    }
}

/// A whole number as TeX, near N = 10^16 written from N: "N", "N + 1",
/// "-N + 1"; any other as itself.
fn n_tex(x: i64) -> String {
    let from = |base: i64, name: &str| match x - base {
        0 => Some(name.to_owned()),
        k @ 1..=9 => Some(format!("{name} + {k}")),
        k @ -9..=-1 => Some(format!("{name} - {}", -k)),
        _ => None,
    };
    from(BIG, "N")
        .or_else(|| from(-BIG, "-N"))
        .unwrap_or_else(|| x.to_string())
}

/// A column written across, as TeX: "(N + 1,\; -N + 1)".
fn tuple_tex(column: &[i64]) -> String {
    let entries: Vec<String> = column.iter().map(|&x| n_tex(x)).collect();
    format!("({})", entries.join(r",\; "))
}

/// The cost at a thousand rows (04_1), from the counted work.
struct Cost {
    n: String,
    redo: String,
    check: String,
    ratio: String,
    /// Twenty checks' multiplications as a share of the product's.
    share: f64,
}

fn cost() -> Result<Cost, String> {
    let redo = model::product_multiplications(BIG_SIDE);
    let again = u64::from(AGAIN) * model::check_multiplications(BIG_SIDE);
    let share = facts::ratio(again as i64, redo as i64)?;
    Ok(Cost {
        n: format!("n = {BIG_SIDE}"),
        redo: format!(
            r"\text{{redo the product (schoolbook): }} n^3 = {}",
            tex_thousands(redo)
        ),
        check: format!(
            r"\text{{{AGAIN} fresh checks: }} {AGAIN} \times n^2 = {}",
            tex_thousands(again)
        ),
        ratio: format!(
            r"\frac{{{}}}{{{}}} = {}",
            tex_thousands(again),
            tex_thousands(redo),
            tex_of(&share)
        ),
        share: again as f64 / redo as f64,
    })
}

/// N + 1 in doubles (04_2, 04_3), every number computed in doubles.
struct NearN {
    note: String,
    labels: Vec<String>,
    /// The offsets from N that are doubles.
    doubles: Vec<i64>,
    question: String,
    /// Where N + 1 lands in doubles, as an offset from N.
    landing: i64,
    gap: i64,
    gap_label: String,
    answer: [String; 2],
    card: String,
}

/// Refused unless N is a power of ten, N + 1 lands on the double N, and
/// N + 1 lies halfway between N and the next double up (the line says the
/// one is lost and the doubles are two apart).
fn near_n() -> Result<NearN, String> {
    let digits = BIG.to_string().len() - 1;
    if 10_i64.pow(digits as u32) != BIG {
        return Err(format!("{BIG} is not a power of ten"));
    }
    let near = BIG as f64;
    let doubles: Vec<i64> = OFFSETS
        .filter(|&k| facts::as_whole(&[(BIG + k) as f64])[0] == BIG + k)
        .collect();
    let landing = facts::as_whole(&[near + PLUS as f64])[0] - BIG;
    let gap = facts::as_whole(&[near.next_up() - near])[0];
    let left = facts::as_whole(&[(near + PLUS as f64) - near])[0];
    if landing != 0 || 2 * PLUS != gap || !doubles.contains(&landing) || !doubles.contains(&gap) {
        return Err("in doubles N + 1 must land on N, halfway to the next double".into());
    }
    let bits = f64::MANTISSA_DIGITS;
    let (plus, at, next) = (n_tex(BIG + PLUS), n_tex(BIG + landing), n_tex(BIG + gap));
    Ok(NearN {
        note: format!("N = 10^{{{digits}}}"),
        labels: OFFSETS.map(|k| n_tex(BIG + k)).collect(),
        doubles,
        question: format!("What is {plus} in doubles?"),
        landing,
        gap,
        gap_label: format!("{gap} apart"),
        answer: [
            format!(r"{plus} \to {at}"),
            format!(r"({plus}) - {} = {left} \text{{ in doubles}}", n_tex(BIG)),
        ],
        card: format!(
            "a double has {bits} significant bits, so above 2^{bits} doubles are at least {gap} apart. {plus} is \
             halfway between the doubles {at} and {next}; the tie goes to {at}."
        ),
    })
}

/// One rounded example (04_4, 04_5) on its four tosses.
struct Example {
    /// C is the exact product of A and B.
    right: bool,
    /// The toss the work is shown for: the first whose verdict doubles change.
    shown: usize,
    exact: Vec<bool>,
    doubled: Vec<bool>,
    work: [String; 2],
}

/// The example `m` = [A, B, C], every column computed both ways. Refused
/// unless doubles change some verdict the wrong way: a right C's columns
/// stop matching, or a wrong C's start to.
fn example(m: &[Matrix<2>; 3]) -> Result<Example, String> {
    let [a, b, c] = m;
    let right = model::product(a, b).0 == *c;
    let tosses: Vec<RoundedToss> = model::tosses::<2>()
        .iter()
        .map(|t| facts::rounded_toss(a, b, c, t))
        .collect();
    let exact: Vec<bool> = tosses.iter().map(|rt| rt.exact.passes()).collect();
    let doubled: Vec<bool> = tosses.iter().map(RoundedToss::passes).collect();
    let shown = (0..tosses.len())
        .find(|&k| exact[k] != doubled[k])
        .ok_or("doubles change no verdict")?;
    let rt = &tosses[shown];
    let work = if right {
        if rt.exact.a_br != rt.exact.cr || rt.a_br == rt.cr {
            return Err("the right C must match exactly and differ in doubles".into());
        }
        [
            format!(
                r"\text{{exact: }} B\,r = {},\;\; A(B\,r) = {} = C\,r",
                tuple_tex(&rt.exact.br),
                tuple_tex(&rt.exact.a_br)
            ),
            format!(
                r"\text{{doubles: }} B\,r = {},\;\; A(B\,r) = {} \neq {} = C\,r",
                tuple_tex(&rt.br),
                tuple_tex(&rt.a_br),
                tuple_tex(&rt.cr)
            ),
        ]
    } else {
        if rt.exact.a_br == rt.exact.cr || rt.a_br != rt.cr {
            return Err("the wrong C must differ exactly and match in doubles".into());
        }
        [
            format!(
                r"\text{{exact: }} A(B\,r) = {} \neq {} = C\,r",
                tuple_tex(&rt.exact.a_br),
                tuple_tex(&rt.exact.cr)
            ),
            format!(r"\text{{doubles: }} A(B\,r) = {} = C\,r", tuple_tex(&rt.a_br)),
        ]
    };
    Ok(Example {
        right,
        shown,
        exact,
        doubled,
        work,
    })
}

/// A verdict word and its color: plain where it is right ("match" for a
/// right C, "no match", caught, for a wrong one), red where it is wrong.
fn verdict(matches: bool, right: bool) -> (&'static str, Srgb) {
    match (matches, right) {
        (true, true) => (MATCH, Palette::ink()),
        (false, false) => (NO_MATCH, palette::caught()),
        (true, false) => (MATCH, palette::mistake()),
        (false, true) => (NO_MATCH, palette::mistake()),
    }
}

/// The tolerance (04_6): the test, what it does to the right C in doubles
/// and to the wrong C in exact sums, and the exact version.
struct Tolerance {
    check: String,
    result: String,
    card: String,
}

/// The tolerance is the smallest that lets the right C through in doubles
/// (`facts::right_tolerance`, its largest residual there). Refused unless
/// with it the right C passes every toss in doubles and the wrong C every
/// toss in exact sums (the line says both).
fn tolerance() -> Result<Tolerance, String> {
    let tol = facts::right_tolerance();
    let tosses = model::tosses::<2>().len();
    let within = |residuals: &[i64]| residuals.iter().filter(|&&r| r <= tol).count();
    let right = within(facts::right_residuals().as_slice());
    let wrong = within(facts::wrong_residuals().as_slice());
    let [a, b, c] = ROUNDED_PASSES;
    if tol <= 0 || right != tosses || wrong != tosses || model::product(&a, &b).0 == c {
        return Err("the tolerance must let the right C through in doubles and the wrong C in exact sums".into());
    }
    Ok(Tolerance {
        check: format!(r"\left|\, A(B\,r) - C\,r \,\right| \le {tol} \text{{ in every row}}"),
        result: format!(
            "the right C passes {right} of {tosses} in doubles; this wrong C, {wrong} of {tosses} in exact sums"
        ),
        card: format!(
            "tolerance {tol}, the right C's largest gap in doubles, lets it through, but then this wrong C passes all \
             {wrong} tosses in exact sums: no half bound."
        ),
    })
}

/// The promise (04_7) and its exact version, from the theorem's bound.
/// Refused unless the bound is one half (the line says half) and a right C
/// passes every toss.
fn promise() -> Result<([String; 4], String), String> {
    let (num, den) = facts::theorem_bound()?;
    let ab = facts::true_product();
    if (num, den) != (1, 2) || model::passing(&A, &B, &ab).len() != model::tosses::<SIDE>().len() {
        return Err("the promise needs half a toss and a right C passing every toss".into());
    }
    Ok((
        [
            RIGHT_LINE.to_owned(),
            WRONG_LINE.to_owned(),
            format!(r"k \text{{ fresh tosses all miss: }} \le \frac{{{num}}}{{{den}^k}}"),
            BOTH_FAIL.to_owned(),
        ],
        format!(
            "with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C with \
             chance at most {num}/{den}^k, in exact arithmetic."
        ),
    ))
}

/// A bar `w` long at height `y`, from the bars' left edge. Detached.
fn bar(stage: &mut Stage<'_>, y: f64, w: f64, style: Style) -> fmn::Result<Mob> {
    let r = rect(v(BAR_X, y - BAR_H / 2.0), w, BAR_H, style)?;
    Ok(stage.arena_mut().add(r))
}

/// A filled dot at `at`. Detached.
fn dot(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(DOT_R)
            .arc_center(at)
            .style(flat(color, 1.0))
            .build(),
    )
}

/// The number line around N: its base, a tick at each whole number, and
/// their `labels`. Detached.
fn number_line(kit: &Kit, stage: &mut Stage<'_>, labels: &[String]) -> fmn::Result<Vec<Mob>> {
    let base = polyline(vec![v(-LINE_HALF, LINE_Y), v(LINE_HALF, LINE_Y)])
        .style(line_style(Palette::dim(), 2.0))
        .build();
    let mut all = vec![stage.arena_mut().add(base)];
    for (k, label) in OFFSETS.zip(labels) {
        let x = line_x(k);
        let tick = polyline(vec![v(x, LINE_Y - TICK_H / 2.0), v(x, LINE_Y + TICK_H / 2.0)])
            .style(line_style(Palette::dim(), 2.0))
            .build();
        all.push(stage.arena_mut().add(tick));
        all.push(kit.tex(stage, label, 26.0, Palette::ink(), &[], v(x, LINE_Y + LINE_LABEL_DY))?);
    }
    Ok(all)
}

/// Example `m`'s grids, A, B and C with N in their entries and the mistake
/// C − AB (zeros dim, wrong entries outlined), all detached.
fn example_grids(kit: &Kit, stage: &mut Stage<'_>, m: &[Matrix<2>; 3]) -> fmn::Result<Vec<Mob>> {
    let [a, b, c] = m;
    let mut all = Vec::new();
    for (matrix, &x) in [a, b, c].into_iter().zip(&GRID_XS) {
        let values = matrix.iter().map(|row| row.to_vec()).collect();
        all.extend(tex_grid(kit, stage, values, v(x, GRIDS_Y), Palette::ink(), n_tex)?.mobs());
    }
    let d = model::mistake(c, &model::product(a, b).0);
    let values = d.iter().map(|row| row.to_vec()).collect();
    let g = grid_colored(kit, stage, values, v(GRID_XS[3], GRIDS_Y), |x| {
        if x == 0 { Palette::dim() } else { Palette::ink() }
    })?;
    all.extend(g.mobs());
    for (i, j) in model::wrong_entries(&d) {
        all.push(cell_box(stage, &g, i, j, palette::mistake())?);
    }
    Ok(all)
}

/// One verdict row for `matches` of a right or wrong C, at height `y`.
/// Detached.
fn verdict_row(kit: &Kit, stage: &mut Stage<'_>, matches: &[bool], right: bool, y: f64) -> fmn::Result<Vec<Mob>> {
    matches
        .iter()
        .zip(&TOSS_XS)
        .map(|(&m, &x)| {
            let (word, color) = verdict(m, right);
            kit.text(stage, word, 22.0, color, v(x, y))
        })
        .collect()
}

impl SceneConstruct for ExactAndRounded {
    fn name(&self) -> &str {
        "04_exact_and_rounded"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let cost = cost().map_err(fail)?;
        let near = near_n().map_err(fail)?;
        let reject = example(&ROUNDED_REJECTS).map_err(fail)?;
        let pass = example(&ROUNDED_PASSES).map_err(fail)?;
        let tol = tolerance().map_err(fail)?;
        let (lines, last_card) = promise().map_err(fail)?;
        if !reject.right || pass.right {
            return Err(fail("the first example must be right and the second wrong".into()));
        }

        // 04_1: the cost at a thousand rows, counted in multiplications
        let line = say(&kit, stage, "04_1")?;
        let n_label = kit.tex(stage, &cost.n, 34.0, Palette::ink(), &[], COST_N_AT)?;
        let redo_label = kit.tex_left(stage, &cost.redo, 28.0, Palette::ink(), &[], v(BAR_X, REDO_LABEL_Y))?;
        let redo_track = bar(stage, REDO_BAR_Y, BAR_W, flat(Palette::faint(), 0.6))?;
        let redo_fill = bar(stage, REDO_BAR_Y, BAR_W, flat(Palette::dim(), 0.9))?;
        fade_in_all(stage, &[n_label, redo_label, redo_track, redo_fill], 0.5)?;
        line.wait_for(stage, "twenty fresh checks")?;
        let check_label = kit.tex_left(stage, &cost.check, 28.0, Palette::ink(), &[], v(BAR_X, CHECK_LABEL_Y))?;
        let check_track = bar(stage, CHECK_BAR_Y, BAR_W, flat(Palette::faint(), 0.6))?;
        let check_fill = bar(stage, CHECK_BAR_Y, BAR_W * cost.share, flat(palette::matrix_a(), 0.95))?;
        fade_in_all(stage, &[check_label, check_track, check_fill], 0.5)?;
        line.wait_for(stage, "a fiftieth")?;
        let ratio = kit.tex(stage, &cost.ratio, 32.0, Palette::ink(), &[], RATIO_AT)?;
        fade_in_all(stage, &[ratio], 0.4)?;
        line.wait_for(stage, "That counts work")?;
        let cost_card = lower_third(&kit, stage, PROGRAMMERS, COST_NOTE)?;
        fade_in_all(stage, &[cost_card], 0.4)?;
        let cost_up = now(stage);

        // 04_2: doubles skip whole numbers; what is N + 1 in doubles? The
        // think time, with no double marked
        let line = say(&kit, stage, "04_2")?;
        let leaving = [
            n_label,
            redo_label,
            redo_track,
            redo_fill,
            check_label,
            check_track,
            check_fill,
            ratio,
        ];
        fade_out_all(stage, &leaving, 0.5)?;
        let n_note = kit.tex(stage, &near.note, 28.0, Palette::ink(), &[], N_NOTE_AT)?;
        let the_line = number_line(&kit, stage, &near.labels)?;
        let mut arriving = vec![n_note];
        arriving.extend(&the_line);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "What is ten to the sixteen")?;
        let question = kit.text(stage, &near.question, 30.0, Palette::ink(), QUESTION_AT)?;
        fade_in_all(stage, &[question], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 04_3: N, written as it is said, so the answer stands through the
        // line; then the doubles there two apart, and N + 1 landing on N
        let line = say(&kit, stage, "04_3")?;
        let answer = [
            kit.tex(stage, &near.answer[0], 30.0, palette::mistake(), &[], QUESTION_AT)?,
            kit.tex(
                stage,
                &near.answer[1],
                26.0,
                Palette::ink(),
                &[],
                v(QUESTION_AT[0], QUESTION_AT[1] + ANSWER_DY),
            )?,
        ];
        fade_out_all(stage, &[q], 0.2)?;
        fade_out_all(stage, &[question], 0.2)?;
        fade_in_all(stage, &answer, 0.2)?;
        line.wait_for(stage, "Doubles there are two apart")?;
        let mut marks: Vec<Mob> = near
            .doubles
            .iter()
            .map(|&k| dot(stage, v(line_x(k), LINE_Y), palette::heads()))
            .collect();
        marks.push(kit.text_left(stage, DOUBLES, 22.0, palette::heads(), LEGEND_AT)?);
        marks.push(kit.text(
            stage,
            &near.gap_label,
            22.0,
            palette::heads(),
            v(line_x(near.gap) / 2.0, LINE_Y + GAP_LABEL_DY),
        )?);
        fade_in_all(stage, &marks, 0.5)?;
        wait_until(stage, cost_up + read_s(COST_NOTE))?;
        let gap_card = precisely(&kit, stage, &near.card)?;
        fade_out_all(stage, &[cost_card], 0.2)?;
        fade_in_all(stage, &[gap_card], 0.2)?;
        let gap_up = now(stage);
        line.wait_for(stage, "so the one is lost")?;
        let lost = dot(stage, v(line_x(PLUS), LINE_Y), palette::mistake());
        fade_in_all(stage, &[lost], 0.3)?;
        together(stage, vec![fly(lost, v(line_x(near.landing), LINE_Y), 0.8)?])?;

        // 04_4: the right C, caught on the toss doubles get wrong
        let line = say(&kit, stage, "04_4")?;
        let mut leaving = the_line;
        leaving.extend(marks);
        leaving.push(lost);
        leaving.extend(answer);
        fade_out_all(stage, &leaving, 0.5)?;
        let grids = example_grids(&kit, stage, &ROUNDED_REJECTS)?;
        let mut names = Vec::with_capacity(GRID_XS.len());
        let colors = [palette::matrix_a(), palette::matrix_b(), palette::claim()];
        for ((name, color), &x) in NAMES.iter().zip(colors).zip(&GRID_XS) {
            names.push(kit.tex(stage, name, 30.0, color, &[], v(x, GRID_NAME_Y))?);
        }
        names.push(kit.tex(
            stage,
            MISTAKE_NAME,
            30.0,
            palette::mistake(),
            &[],
            v(GRID_XS[3], GRID_NAME_Y),
        )?);
        let mut coins = Vec::new();
        for (t, &x) in model::tosses::<2>().iter().zip(&TOSS_XS) {
            let xs = [x - COIN_PITCH / 2.0, x + COIN_PITCH / 2.0];
            coins.extend(coin_mobs(&toss_coins(&kit, stage, t, &xs, COIN_Y)?));
        }
        let row_labels = [
            kit.text_left(stage, EXACT, 22.0, Palette::dim(), v(ROW_LABEL_X, ROW_YS[0]))?,
            kit.text_left(stage, DOUBLES, 22.0, Palette::dim(), v(ROW_LABEL_X, ROW_YS[1]))?,
        ];
        let mut arriving = grids.clone();
        arriving.extend(&names);
        arriving.extend(&coins);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "This C is A times B")?;
        let mut exact_row = verdict_row(&kit, stage, &reject.exact, reject.right, ROW_YS[0])?;
        exact_row.insert(0, row_labels[0]);
        fade_in_lagged(stage, &exact_row, 0.8, 0.2)?;
        line.wait_for(stage, "with both coins heads")?;
        let mut doubles_row = verdict_row(&kit, stage, &reject.doubled, reject.right, ROW_YS[1])?;
        doubles_row.insert(0, row_labels[1]);
        fade_in_lagged(stage, &doubles_row, 0.8, 0.2)?;
        let shown = ring(stage, v(TOSS_XS[reject.shown], COIN_Y), SHOWN_R, Palette::hero());
        let work = [
            kit.tex(stage, &reject.work[0], 26.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.tex(
                stage,
                &reject.work[1],
                26.0,
                palette::mistake(),
                &[],
                v(0.0, WORK_YS[1]),
            )?,
        ];
        fade_in_all(stage, &[shown], 0.3)?;
        fade_in_lagged(stage, &work, 0.8, 0.4)?;

        // 04_5: the wrong C by one, passing every toss in doubles
        let line = say(&kit, stage, "04_5")?;
        wait_until(stage, gap_up + read_s(&near.card))?;
        let next_grids = example_grids(&kit, stage, &ROUNDED_PASSES)?;
        let mut old = grids;
        old.extend(exact_row.iter().skip(1));
        old.extend(doubles_row.iter().skip(1));
        old.push(shown);
        old.extend(work);
        old.push(gap_card);
        fade_out_all(stage, &old, 0.25)?;
        fade_in_all(stage, &next_grids, 0.25)?;
        line.wait_for(stage, "This C is too small by one")?;
        let exact_row = verdict_row(&kit, stage, &pass.exact, pass.right, ROW_YS[0])?;
        fade_in_lagged(stage, &exact_row, 0.8, 0.2)?;
        line.wait_for(stage, "all four tosses miss it")?;
        let doubles_row = verdict_row(&kit, stage, &pass.doubled, pass.right, ROW_YS[1])?;
        fade_in_lagged(stage, &doubles_row, 0.8, 0.2)?;
        let shown = ring(stage, v(TOSS_XS[pass.shown], COIN_Y), SHOWN_R, Palette::hero());
        let work = [
            kit.tex(stage, &pass.work[0], 26.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.tex(stage, &pass.work[1], 26.0, palette::mistake(), &[], v(0.0, WORK_YS[1]))?,
        ];
        fade_in_all(stage, &[shown], 0.3)?;
        fade_in_lagged(stage, &work, 0.8, 0.4)?;

        // 04_6: a tolerance, and the wrong C inside it
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &work, 0.3)?;
        line.wait_for(stage, "but then a wrong C inside it")?;
        // the rows above are the test of equality; dimmed, they step back
        // for the tolerant test, whose verdicts differ from theirs
        let mut dims = Vec::new();
        for &m in exact_row.iter().chain(&doubles_row).chain(&row_labels) {
            dims.push(fade_to(m, EQUALITY_DIM, 0.4)?);
        }
        together(stage, dims)?;
        let tol_lines = [
            kit.tex(stage, &tol.check, 28.0, Palette::ink(), &[], v(0.0, WORK_YS[0]))?,
            kit.text(stage, &tol.result, 22.0, Palette::ink(), v(0.0, WORK_YS[1]))?,
        ];
        let tol_card = precisely(&kit, stage, &tol.card)?;
        let mut arriving = tol_lines.to_vec();
        arriving.push(tol_card);
        fade_in_all(stage, &arriving, 0.4)?;
        let tol_up = now(stage);

        // 04_7: the promise, on the opening picture
        let line = say(&kit, stage, "04_7")?;
        let mut leaving = vec![n_note, shown];
        leaving.extend(next_grids);
        leaving.extend(names);
        leaving.extend(coins);
        leaving.extend(row_labels);
        leaving.extend(exact_row);
        leaving.extend(doubles_row);
        leaving.extend(tol_lines);
        fade_out_all(stage, &leaving, 0.5)?;
        let mut opening = Vec::new();
        let colors = [palette::matrix_a(), palette::matrix_b(), palette::claim()];
        let mut grids = Vec::with_capacity(OPEN_XS.len());
        for (((m, name), color), &x) in [A, B, C].iter().zip(NAMES).zip(colors).zip(&OPEN_XS) {
            let g = matrix_grid(&kit, stage, m, v(x, OPEN_Y), Palette::ink())?;
            opening.extend(g.mobs());
            opening.push(kit.tex(stage, name, 34.0, color, &[], v(x, OPEN_NAME_Y))?);
            grids.push(g);
        }
        opening.push(kit.tex(stage, TIMES, 36.0, Palette::dim(), &[], v(OPEN_TIMES_X, OPEN_Y))?);
        opening.push(kit.tex(stage, EQUALS, 36.0, Palette::dim(), &[], v(OPEN_EQUALS_X, OPEN_Y))?);
        let (wrong_row, wrong_col) = facts::wrong_entry().map_err(fail)?;
        opening.push(cell_box(stage, &grids[2], wrong_row, wrong_col, palette::mistake())?);
        fade_in_all(stage, &opening, 0.6)?;
        line.wait_for(stage, "A right C matches every toss")?;
        let first = kit.text(stage, &lines[0], 24.0, Palette::ink(), v(PROMISE_X, PROMISE_YS[0]))?;
        fade_in_all(stage, &[first], 0.4)?;
        line.wait_for(stage, "a wrong one gets past")?;
        let rest = [
            kit.text(stage, &lines[1], 24.0, Palette::ink(), v(PROMISE_X, PROMISE_YS[1]))?,
            kit.tex(stage, &lines[2], 28.0, Palette::ink(), &[], v(PROMISE_X, PROMISE_YS[2]))?,
            kit.text(stage, &lines[3], 22.0, Palette::dim(), v(PROMISE_X, PROMISE_YS[3]))?,
        ];
        fade_in_lagged(stage, &rest, 1.0, 0.4)?;
        wait_until(stage, tol_up + read_s(&tol.card))?;
        let last = precisely(&kit, stage, &last_card)?;
        fade_out_all(stage, &[tol_card], 0.2)?;
        fade_in_all(stage, &[last], 0.2)?;
        let card_up = now(stage);
        stand(&kit, stage, card_up)?;
        wait_until(stage, card_up + read_s(&last_card))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cost is the counted work, and the bars stand to scale.
    #[test]
    fn the_cost_is_counted_work() {
        let c = cost().unwrap();
        assert_eq!(c.n, "n = 1000");
        assert_eq!(
            c.redo,
            r"\text{redo the product (schoolbook): } n^3 = 1{,}000{,}000{,}000"
        );
        assert_eq!(c.check, r"\text{20 fresh checks: } 20 \times n^2 = 20{,}000{,}000");
        assert_eq!(c.ratio, r"\frac{20{,}000{,}000}{1{,}000{,}000{,}000} = \frac{1}{50}");
        assert!((c.share * 50.0 - 1.0).abs() < 1e-12, "{}", c.share);
        // the check's bar is narrow but there: about a fiftieth of the track
        assert!(BAR_W * c.share > 0.1);
    }

    /// N + 1 lands on N in doubles, the doubles near N two apart, all
    /// computed.
    #[test]
    fn n_plus_one_is_lost_in_doubles() {
        let n = near_n().unwrap();
        assert_eq!(n.note, "N = 10^{16}");
        assert_eq!(n.labels, ["N - 2", "N - 1", "N", "N + 1", "N + 2"]);
        assert_eq!(n.doubles, [-2, 0, 2]);
        assert_eq!(n.question, "What is N + 1 in doubles?");
        assert_eq!((n.landing, n.gap), (0, 2));
        assert_eq!(n.gap_label, "2 apart");
        assert_eq!(n.answer, [r"N + 1 \to N", r"(N + 1) - N = 0 \text{ in doubles}"]);
        assert_eq!(n_tex(1 - BIG), "-N + 1");
        assert_eq!(n_tex(2), "2");
        assert_eq!(tuple_tex(&[BIG + 1, 1 - BIG]), r"(N + 1,\; -N + 1)");
    }

    /// In doubles the right C is caught on toss 11 and the wrong C passes
    /// every toss; exact sums keep both promises.
    #[test]
    fn the_examples_break_both_promises_in_doubles() {
        let r = example(&ROUNDED_REJECTS).unwrap();
        assert!(r.right);
        assert_eq!(r.shown, 3);
        assert_eq!(r.exact, [true; 4]);
        assert_eq!(r.doubled, [true, true, true, false]);
        assert_eq!(
            r.work,
            [
                r"\text{exact: } B\,r = (N + 1,\; -N + 1),\;\; A(B\,r) = (2,\; 0) = C\,r",
                r"\text{doubles: } B\,r = (N,\; -N),\;\; A(B\,r) = (0,\; 0) \neq (2,\; 0) = C\,r",
            ]
        );
        let p = example(&ROUNDED_PASSES).unwrap();
        assert!(!p.right);
        assert_eq!(p.shown, 2);
        assert_eq!(p.exact, [true, true, false, false]);
        assert_eq!(p.doubled, [true; 4]);
        assert_eq!(
            p.work,
            [
                r"\text{exact: } A(B\,r) = (N + 1,\; 0) \neq (N,\; 0) = C\,r",
                r"\text{doubles: } A(B\,r) = (N,\; 0) = C\,r",
            ]
        );
        // a verdict is red exactly where it is wrong
        assert_eq!(verdict(false, true).1, palette::mistake());
        assert_eq!(verdict(true, false).1, palette::mistake());
        assert_eq!(verdict(false, false), (NO_MATCH, palette::caught()));
    }

    /// The tolerance is the right C's largest gap in doubles: it lets the
    /// right C through on every toss in doubles, and the wrong C on every
    /// toss even in exact sums. A tolerance of 1 would still catch the right
    /// C on toss 11.
    #[test]
    fn a_tolerance_passes_the_wrong_c() {
        assert_eq!(facts::right_residuals(), [0, 0, 0, 2]);
        assert_eq!(facts::wrong_residuals(), [0, 0, 1, 1]);
        assert_eq!(facts::right_tolerance(), 2);
        let smaller = facts::right_tolerance() - 1;
        assert!(
            facts::right_residuals().iter().any(|&r| r > smaller),
            "tolerance {smaller} would still catch the right C"
        );
        let t = tolerance().unwrap();
        assert_eq!(t.check, r"\left|\, A(B\,r) - C\,r \,\right| \le 2 \text{ in every row}");
        assert_eq!(
            t.result,
            "the right C passes 4 of 4 in doubles; this wrong C, 4 of 4 in exact sums"
        );
    }

    /// The promise is the theorem's, and every card keeps its conditions,
    /// word for word, in two lines with room to spare.
    #[test]
    fn the_cards_keep_their_conditions() {
        let (lines, last) = promise().unwrap();
        assert_eq!(lines[2], r"k \text{ fresh tosses all miss: } \le \frac{1}{2^k}");
        assert_eq!(
            last,
            "with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C \
             with chance at most 1/2^k, in exact arithmetic."
        );
        let n = near_n().unwrap();
        assert_eq!(
            n.card,
            "a double has 53 significant bits, so above 2^53 doubles are at least 2 apart. N + 1 is halfway \
             between the doubles N and N + 2; the tie goes to N."
        );
        let t = tolerance().unwrap();
        assert_eq!(
            t.card,
            "tolerance 2, the right C's largest gap in doubles, lets it through, but then this wrong C passes all 4 \
             tosses in exact sums: no half bound."
        );
        assert!(COST_NOTE.contains("not time"), "{COST_NOTE}");
        for text in [COST_NOTE, n.card.as_str(), t.card.as_str(), last.as_str()] {
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "twenty fresh checks"),
            ("04_1", "a fiftieth"),
            ("04_1", "That counts work"),
            ("04_2", "What is ten to the sixteen"),
            ("04_3", "Doubles there are two apart"),
            ("04_3", "so the one is lost"),
            ("04_4", "This C is A times B"),
            ("04_4", "with both coins heads"),
            ("04_5", "This C is too small by one"),
            ("04_5", "all four tosses miss it"),
            ("04_6", "but then a wrong C inside it"),
            ("04_7", "A right C matches every toss"),
            ("04_7", "a wrong one gets past"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::{COIN_R, DX, DY};
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const CARD_2: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // 04_1: the bars inside the frame, each under its label, the ratio
        // over a card
        const {
            assert!(
                BAR_X > -FRAME_X_RADIUS + 0.15 && BAR_X + BAR_W < FRAME_X_RADIUS - 0.15,
                "bars inside"
            )
        };
        const { assert!(COST_N_AT[1] - 0.25 > REDO_LABEL_Y + 0.2, "n over the first label") };
        const { assert!(REDO_LABEL_Y - 0.2 > REDO_BAR_Y + BAR_H / 2.0, "label over its bar") };
        const { assert!(REDO_BAR_Y - BAR_H / 2.0 - 0.2 > CHECK_LABEL_Y + 0.2, "bars apart") };
        const { assert!(CHECK_LABEL_Y - 0.2 > CHECK_BAR_Y + BAR_H / 2.0, "label over its bar") };
        const { assert!(RATIO_AT[1] - 0.45 > CARD_2, "ratio over the card") };
        // 04_2, 04_3: the line, its labels and dots inside, the question
        // and the think ring clear of them and of each other
        const { assert!(LINE_HALF < FRAME_X_RADIUS - 0.15, "line inside") };
        const { assert!(2.0 * LINE_DX + 0.6 < LINE_HALF + 0.3, "outer labels on the line") };
        const { assert!(N_NOTE_AT[1] - 0.2 > LEGEND_AT[1] + 0.2, "N over the legend") };
        const {
            assert!(
                LEGEND_AT[1] - 0.15 > LINE_Y + GAP_LABEL_DY + 0.15,
                "legend over the gap label"
            )
        };
        const {
            assert!(
                LINE_Y + LINE_LABEL_DY - 0.2 > QUESTION_AT[1] + 0.25,
                "labels over the question"
            )
        };
        const {
            assert!(
                THINK_AT[1] + 0.62 < LINE_Y + LINE_LABEL_DY - 0.2,
                "ring under the labels"
            )
        };
        const { assert!(QUESTION_AT[1] + ANSWER_DY - 0.2 > CARD_2, "answer over the card") };
        // 04_4 to 04_6: grids apart and inside, names, coins, rows and work
        // in order down to a card
        const { assert!(GRID_XS[0] - DX - 0.2 > -FRAME_X_RADIUS + 0.15, "grids inside") };
        const { assert!(GRID_XS[1] - GRID_XS[0] > 2.0 * (DX + 0.05) + 0.3, "grids apart") };
        const { assert!(GRIDS_Y + DY + 0.2 < N_NOTE_AT[1] - 0.2, "grids under N") };
        const { assert!(GRIDS_Y - DY - 0.05 > GRID_NAME_Y + 0.2, "names under the grids") };
        const { assert!(GRID_NAME_Y - 0.2 > COIN_Y + SHOWN_R + 0.05, "coins under the names") };
        const { assert!(COIN_Y - SHOWN_R > ROW_YS[0] + 0.15, "rows under the ring") };
        const { assert!(ROW_YS[0] - ROW_YS[1] > 0.4, "rows apart") };
        const { assert!(ROW_YS[1] - 0.15 > WORK_YS[0] + 0.2, "work under the rows") };
        const { assert!(WORK_YS[1] - 0.2 > CARD_2, "work over the card") };
        const { assert!(SHOWN_R > COIN_PITCH / 2.0 + COIN_R, "the ring around both coins") };
        const { assert!(TOSS_XS[1] - TOSS_XS[0] > 2.0 * SHOWN_R + 0.3, "tosses apart") };
        const { assert!(ROW_LABEL_X + 1.0 < TOSS_XS[0] - 0.6, "row labels left of the verdicts") };
        // 04_7: the promise under the opening picture's names, over the card
        const { assert!(OPEN_NAME_Y - 0.2 > PROMISE_YS[0] + 0.2, "promise under the names") };
        const { assert!(PROMISE_YS[3] - 0.15 > CARD_2, "promise over the card") };
    }
}
