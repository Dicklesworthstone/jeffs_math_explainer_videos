//! Chapters 11 and 12: money, and the assumptions the formula quietly made. Every number is invented for teaching: this is
//! arithmetic, not a forecast and not advice. The capacity chart reuses the logistic picture of the earlier draft's
//! `applications.rs`. Author source, not built or rendered.
use super::ending_draw::{
    arrow, dot, fade_in_all, fade_out_all, flat, line_of_math, line_style, live, path, ring, run, seg, swap, title,
};
use crate::changing_rate_model as m;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct MoneyLimits {
    pub kit: Kit,
}

pub struct ChanceAndLimits {
    pub kit: Kit,
}

const CARD_X: f64 = 3.45;

// Think time after each question, in seconds. Review 721 M10 proposed three to four seconds for 11_2 and 12_1 and two
// to three for 12_3. The values are authored here and still have to be set against Frederick Surrey takes in
// audio-review.
const THINK_11_2: f64 = 4.0;
const THINK_12_1: f64 = 3.5;
const THINK_12_3: f64 = 2.5;

// The number line of 11_1. It starts at 112, not at zero, and says so.
const LINE_LEFT: f64 = -6.0;
const LINE_Y: f64 = 0.35;
const PER_UNIT: f64 = 7.2;
const NAME_X: f64 = 1.0;
const VALUE_X: f64 = 5.6;
const CREDIT_NAMES: [&str; 5] = ["once a year", "twice a year", "four times a year", "monthly", "daily"];

// The two-year chart of 11_2 and 11_3: height is proportional to the money.
const YEAR_X: [f64; 3] = [-5.7, -3.6, -1.5];
const AT_START_Y: f64 = 0.85;
const PER_MONEY: f64 = 0.06;
const WITH_MONEY: &str = r"y'=r(t)\,y+c(t)";
const SOLVED_FIRST: &str = r"y(T)=y(0)\,E\left(R(T)\right)";
const SOLVED_SECOND: &str = r"+\int_0^T c(s)\,E\left(R(T)-R(s)\right)ds";
/// In 11_3 the two withdrawals fall on the same year-end column, and their strokes would coincide where the amounts
/// overlap. Each is drawn this far to one side of the column (review 721 M9).
const DROP_OFFSET: f64 = 0.12;
/// Where 11_2's "90" stands, from its point: left of the column and a little down, off both withdrawal strokes.
const NINETY_AT: (f64, f64) = (-0.45, -0.12);

// The histogram of 12_2.
const RULER_X: f64 = -3.0;
const RULER_UNIT: f64 = 4.2;
const HIST_LEFT: f64 = -6.2;
const HIST_STEP: f64 = 0.125;
const HIST_BASE: f64 = -2.7;
const HIST_SCALE: f64 = 22.0;
/// The ruler of 12_2 measures distance as an input of E, not in the base-ten unit of 07's slide rule (review 721 H3).
const RULER_NAME: &str = "distance, measured as an input of E";
const MIDDLE_RUN: &str = "the middle run: exactly half heads";
/// One run with heads and tails in turn, under the 12_2 moral (review 721 L3): tosses across, distance down.
const RUN_LEFT: f64 = -6.2;
const RUN_RIGHT: f64 = -0.6;
const RUN_ZERO: f64 = 2.4;
const RUN_UNIT: f64 = 0.28;
const RUN_NAME: &str = "one run: heads and tails in turn";

// The capacity chart of 12_3 and 12_4.
const CHART_LEFT: f64 = -6.0;
const CHART_BASE: f64 = -2.7;
const CHART_W: f64 = 6.4;
const CHART_H: f64 = 4.4;
const CHART_END: f64 = 5.0;
/// The chart reaches a little above the capacity, so the unlimited curve visibly crosses it and runs on to the top
/// (review 721 L2). `cap.top` still records where it meets the capacity.
const CHART_TOP: f64 = 1.15;
/// Time marks, so "half a unit" and "one unit" can be read off the axis (review 721 L2).
const TIME_TICKS: [f64; 5] = [0.5, 1.0, 2.0, 3.0, 4.0];
const CURVE_SAMPLES: u32 = 120;
const EARLY: f64 = 0.5;
const LAW_ROOM: &str = r"y'=r\,y\left(1-\frac{y}{K}\right)";
const LAW_PLAIN: &str = r"y'=r\,y";
const RECIPROCAL: &str = r"\frac{1}{y(t)}-\frac{1}{K}=\left(\frac{1}{y(0)}-\frac{1}{K}\right)E(-r\,t)";

fn on_line(amount: f64) -> Vec3 {
    v(LINE_LEFT + PER_UNIT * (amount - 112.0), LINE_Y)
}
fn money_at(column: usize, amount: f64) -> Vec3 {
    v(YEAR_X[column], AT_START_Y + PER_MONEY * (amount - m::BALANCE))
}
fn chart(t: f64, y: f64) -> Vec3 {
    v(CHART_LEFT + CHART_W * t / CHART_END, CHART_BASE + CHART_H * y)
}
/// A point moved sideways, off a shared column.
fn beside(p: Vec3, dx: f64) -> Vec3 {
    v(p[0] + dx, p[1])
}
/// A point of the 12_2 alternating run: toss number across, distance in E's own input up and down.
fn run_point(toss: usize, distance: f64) -> Vec3 {
    let across = toss as f64 / f64::from(m::TOSSES);
    v(
        RUN_LEFT + (RUN_RIGHT - RUN_LEFT) * across,
        RUN_ZERO + RUN_UNIT * distance,
    )
}
fn swap_title(k: &Kit, stage: &mut Stage<'_>, old: Mob, words: &str) -> fmn::Result<Mob> {
    let new = title(k, stage, words)?;
    swap(stage, &[old], &[new], 0.5)?;
    Ok(new)
}
fn percent(fraction: f64) -> String {
    format!("{:.0}", fraction * 100.0)
}
/// A money amount beside its point on the chart.
fn node_label(
    k: &Kit,
    stage: &mut Stage<'_>,
    amount: f64,
    column: usize,
    dx: f64,
    dy: f64,
    c: Srgb,
) -> fmn::Result<Mob> {
    let at = money_at(column, amount);
    k.text(stage, &format!("{amount:.0}"), 25.0, c, v(at[0] + dx, at[1] + dy))
}

/// The zoomed number line and its scale.
fn number_line(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![seg(stage, on_line(112.0), on_line(112.8), Palette::dim(), 2.5)];
    for j in 0..=4 {
        let amount = 112.0 + 0.2 * f64::from(j);
        let at = on_line(amount);
        out.push(seg(
            stage,
            v(at[0], LINE_Y - 0.08),
            v(at[0], LINE_Y + 0.08),
            Palette::dim(),
            2.0,
        ));
        out.push(k.text(
            stage,
            &format!("{amount:.2}"),
            21.0,
            Palette::dim(),
            v(at[0], LINE_Y - 0.32),
        )?);
    }
    out.push(k.text(
        stage,
        "balance after one year; the line starts at 112",
        21.0,
        Palette::dim(),
        v(-3.1, LINE_Y - 0.82),
    )?);
    Ok(out)
}

/// 11_1: more credits raise the balance by less and less, toward the exponential.
fn credits(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Vec<Mob>)> {
    k.say(stage, "11_1")?;
    let heading = title(k, stage, "What a quoted rate means")?;
    fade_in_all(stage, &[heading], 0.5)?;

    k.until_sentence(stage, "11_1", 1)?;
    let mut shown = number_line(k, stage)?;
    let quoted = format!("start {:.0}, quoted {}% a year", m::BALANCE, percent(m::QUOTED));
    shown.push(k.text(stage, &quoted, 27.0, Palette::ink(), v(-3.1, 2.35))?);
    fade_in_all(stage, &shown, 0.5)?;

    let sentence_of_row = [2, 3, 3, 3, 4];
    for (i, (&n, name)) in m::CREDITS.iter().zip(CREDIT_NAMES).enumerate() {
        k.until_sentence(stage, "11_1", sentence_of_row[i])?;
        let y = 1.9 - 0.55 * i as f64;
        let amount = m::credited(n);
        let row = [
            k.text_left(stage, name, 26.0, Palette::ink(), v(NAME_X, y))?,
            k.text(stage, &format!("{amount:.2}"), 26.0, Palette::gold(), v(VALUE_X, y))?,
            dot(stage, on_line(amount), 0.08, Palette::gold()),
            k.text(
                stage,
                &n.to_string(),
                21.0,
                Palette::gold(),
                v(on_line(amount)[0], LINE_Y + 0.3),
            )?,
        ];
        fade_in_all(stage, &row, 0.35)?;
        shown.extend(row);
    }

    k.until_sentence(stage, "11_1", 5)?;
    let cells = line_of_math(
        k,
        stage,
        &format!(r"{:.0}\left(1+\frac{{{}}}{{n}}\right)^n", m::BALANCE, m::QUOTED),
        34.0,
        Palette::teal(),
        -3.1,
        -1.5,
    )?;
    fade_in_all(stage, &[cells], 0.5)?;
    k.until_sentence(stage, "11_1", 6)?;
    let limit = [
        line_of_math(
            k,
            stage,
            &format!(r"{:.0}\,E({})", m::BALANCE, m::QUOTED),
            27.0,
            Palette::hero(),
            NAME_X + 0.95,
            -0.95,
        )?,
        k.text(
            stage,
            &format!("{:.2}", m::continuous()),
            26.0,
            Palette::hero(),
            v(VALUE_X, -0.95),
        )?,
        ring(stage, on_line(m::continuous()), 0.17, Palette::hero(), 3.0),
    ];
    fade_in_all(stage, &limit, 0.5)?;
    k.hold(stage)?;
    shown.push(cells);
    shown.extend(limit);
    Ok((heading, shown))
}

/// 11_2: a percentage describes a multiplier; multipliers commute until money moves. 63 and 48 wait for 11_3.
fn multipliers(k: &Kit, stage: &mut Stage<'_>, heading: Mob, last: &[Mob]) -> fmn::Result<(Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "11_2")?;
    fade_out_all(stage, last, 0.4)?;
    swap_title(k, stage, heading, "Multipliers, not percentages")?;

    let (gain, loss, both) = (m::GOOD - 1.0, 1.0 - m::BAD, m::GOOD * m::BAD);
    let start = dot(stage, money_at(0, m::BALANCE), 0.09, Palette::ink());
    let start_name = node_label(k, stage, m::BALANCE, 0, -0.45, 0.0, Palette::ink())?;
    k.until_sentence(stage, "11_2", 1)?;
    let up = line_of_math(
        k,
        stage,
        &format!(r"\times {}:\quad +{}\%", m::GOOD, percent(gain)),
        30.0,
        Palette::green(),
        CARD_X,
        2.2,
    )?;
    fade_in_all(stage, &[start, start_name, up], 0.4)?;
    k.until_sentence(stage, "11_2", 2)?;
    let down = line_of_math(
        k,
        stage,
        &format!(r"\times {}:\quad -{}\%", m::BAD, percent(loss)),
        30.0,
        Palette::red(),
        CARD_X,
        1.6,
    )?;
    fade_in_all(stage, &[down], 0.4)?;
    k.until_sentence(stage, "11_2", 3)?;
    let naive = line_of_math(
        k,
        stage,
        &format!(
            r"+{}\%-{}\%\ \text{{suggests}}\ +{}\%",
            percent(gain),
            percent(loss),
            percent(gain - loss)
        ),
        27.0,
        Palette::dim(),
        CARD_X,
        1.0,
    )?;
    fade_in_all(stage, &[naive], 0.4)?;

    k.until_sentence(stage, "11_2", 4)?;
    let [a, b] = m::two_years(m::GOOD, m::BAD);
    let good_path = path(
        stage,
        vec![money_at(0, m::BALANCE), money_at(1, a), money_at(2, b)],
        Palette::gold(),
        3.5,
    );
    let good_labels = [
        node_label(k, stage, a, 1, 0.0, 0.3, Palette::gold())?,
        node_label(k, stage, b, 2, 0.45, 0.0, Palette::hero())?,
    ];
    let true_line = line_of_math(
        k,
        stage,
        &format!(
            r"{}\times {}={both:.2}:\quad +{}\%",
            m::GOOD,
            m::BAD,
            percent(both - 1.0)
        ),
        30.0,
        Palette::hero(),
        CARD_X,
        0.35,
    )?;
    fade_in_all(stage, &[good_path, good_labels[0], good_labels[1], true_line], 0.5)?;

    // Review 721 M6: each continuous rate is shown as the input of E that gives the year's multiplier. The natural
    // logarithm is not named before 15_2, so no logarithm is written here.
    let (good_rate, bad_rate, both_rate) = (
        m::continuous_rate(m::GOOD),
        m::continuous_rate(m::BAD),
        m::continuous_rate(both),
    );
    k.until_sentence(stage, "11_2", 6)?;
    let good_input = line_of_math(
        k,
        stage,
        &format!(r"E({good_rate:.5})={}", m::GOOD),
        27.0,
        Palette::teal(),
        CARD_X,
        -0.35,
    )?;
    fade_in_all(stage, &[good_input], 0.4)?;
    k.until_sentence(stage, "11_2", 8)?;
    let bad_input = line_of_math(
        k,
        stage,
        &format!(r"E({bad_rate:.5})={}", m::BAD),
        27.0,
        Palette::teal(),
        CARD_X,
        -0.9,
    )?;
    fade_in_all(stage, &[bad_input], 0.4)?;
    k.until_sentence(stage, "11_2", 9)?;
    let adding = [
        line_of_math(
            k,
            stage,
            &format!(r"{good_rate:.5}{bad_rate:.5}={both_rate:.5}"),
            25.0,
            Palette::teal(),
            CARD_X,
            -1.45,
        )?,
        line_of_math(
            k,
            stage,
            &format!(r"E({both_rate:.5})={both:.2}"),
            27.0,
            Palette::hero(),
            CARD_X,
            -2.0,
        )?,
    ];
    fade_in_all(stage, &adding, 0.5)?;

    k.until_sentence(stage, "11_2", 10)?;
    let [c, d] = m::two_years(m::BAD, m::GOOD);
    let bad_path = path(
        stage,
        vec![money_at(0, m::BALANCE), money_at(1, c), money_at(2, d)],
        Palette::teal(),
        3.5,
    );
    let bad_label = node_label(k, stage, c, 1, NINETY_AT.0, NINETY_AT.1, Palette::teal())?;
    fade_in_all(stage, &[bad_path, bad_label], 0.5)?;

    k.until_sentence(stage, "11_2", 13)?;
    // A mark, not to the chart's scale: it stands beside the year-end column and clear of every later path.
    let taken_x = YEAR_X[1] + 0.55;
    let taken = [
        arrow(stage, v(taken_x, 0.2), v(taken_x, -0.5), Palette::red(), 4.0),
        k.text(
            stage,
            &format!("\u{2212}{:.0}", m::TAKEN),
            27.0,
            Palette::red(),
            v(taken_x + 0.6, -0.15),
        )?,
    ];
    fade_in_all(stage, &taken, 0.4)?;
    k.until_sentence(stage, "11_2", 14)?;
    let question = k.text(
        stage,
        "does the order still not matter?",
        28.0,
        Palette::hero(),
        v(CARD_X, -2.6),
    )?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_11_2)?;

    let cards = vec![
        up, down, naive, true_line, good_input, bad_input, adding[0], adding[1], question,
    ];
    let chart_parts = vec![
        good_path,
        bad_path,
        good_labels[1],
        taken[0],
        taken[1],
        start,
        start_name,
        good_labels[0],
        bad_label,
    ];
    Ok((cards, chart_parts))
}

/// 11_3: the withdrawal makes the order matter. The answer starts only here.
fn withdrawal(k: &Kit, stage: &mut Stage<'_>, cards: &[Mob], chart_parts: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "11_3")?;
    fade_out_all(stage, cards, 0.4)?;
    let [a, b, c] = m::with_withdrawal(m::GOOD, m::BAD);
    let [p, q, r] = m::with_withdrawal(m::BAD, m::GOOD);

    // Review 721 M9: both 11_2 paths leave, and so does their "108", which no 11_3 path reaches. Each withdrawal is
    // drawn to one side of the shared column, so the two strokes do not coincide.
    k.until_sentence(stage, "11_3", 1)?;
    fade_out_all(stage, &chart_parts[..3], 0.3)?;
    let good_path = path(
        stage,
        vec![
            money_at(0, m::BALANCE),
            beside(money_at(1, a), -DROP_OFFSET),
            beside(money_at(1, b), -DROP_OFFSET),
            money_at(2, c),
        ],
        Palette::gold(),
        3.5,
    );
    let good_labels = [
        node_label(k, stage, b, 1, -0.45, 0.0, Palette::gold())?,
        node_label(k, stage, c, 2, 0.45, 0.0, Palette::gold())?,
    ];
    let good_name = k.text(stage, "good year first", 22.0, Palette::gold(), v(-2.45, 2.35))?;
    fade_in_all(stage, &[good_path, good_labels[0], good_labels[1], good_name], 0.5)?;

    k.until_sentence(stage, "11_3", 2)?;
    let bad_path = path(
        stage,
        vec![
            money_at(0, m::BALANCE),
            beside(money_at(1, p), DROP_OFFSET),
            beside(money_at(1, q), DROP_OFFSET),
            money_at(2, r),
        ],
        Palette::teal(),
        3.5,
    );
    let bad_labels = [
        node_label(k, stage, q, 1, -0.45, 0.0, Palette::teal())?,
        node_label(k, stage, r, 2, 0.45, 0.0, Palette::teal())?,
    ];
    let bad_name = k.text(stage, "bad year first", 22.0, Palette::teal(), v(-2.45, -3.1))?;
    fade_in_all(stage, &[bad_path, bad_labels[0], bad_labels[1], bad_name], 0.5)?;

    k.until_sentence(stage, "11_3", 3)?;
    let (top, bottom) = (money_at(2, c), money_at(2, r));
    let span = seg(
        stage,
        v(top[0] + 0.85, top[1]),
        v(bottom[0] + 0.85, bottom[1]),
        Palette::hero(),
        2.5,
    );
    let gap = line_of_math(
        k,
        stage,
        &format!(r"{c:.0}-{r:.0}={:.0}", c - r),
        30.0,
        Palette::hero(),
        CARD_X,
        2.2,
    )?;
    fade_in_all(stage, &[span, gap], 0.4)?;

    k.until_sentence(stage, "11_3", 4)?;
    let stays = k.text(
        stage,
        "each dollar is multiplied only while it stays",
        25.0,
        Palette::ink(),
        v(CARD_X, 1.5),
    )?;
    fade_in_all(stage, &[stays], 0.4)?;

    k.until_sentence(stage, "11_3", 5)?;
    let (undisturbed_good, undisturbed_bad) = (m::two_years(m::GOOD, m::BAD)[1], m::two_years(m::BAD, m::GOOD)[1]);
    let costs = [
        line_of_math(
            k,
            stage,
            &format!(r"{undisturbed_good:.0}-{:.0}={c:.0}", m::cost_of_withdrawal(m::BAD)),
            28.0,
            Palette::gold(),
            CARD_X,
            0.8,
        )?,
        line_of_math(
            k,
            stage,
            &format!(r"{undisturbed_bad:.0}-{:.0}={r:.0}", m::cost_of_withdrawal(m::GOOD)),
            28.0,
            Palette::teal(),
            CARD_X,
            0.25,
        )?,
    ];
    fade_in_all(stage, &costs, 0.4)?;

    k.until_sentence(stage, "11_3", 6)?;
    let law = line_of_math(k, stage, WITH_MONEY, 32.0, Palette::green(), CARD_X, -0.65)?;
    let term = k.text(
        stage,
        "c(t): money moving in or out",
        22.0,
        Palette::dim(),
        v(CARD_X, -1.2),
    )?;
    fade_in_all(stage, &[law, term], 0.5)?;
    k.until_sentence(stage, "11_3", 7)?;
    let solved = [
        line_of_math(k, stage, SOLVED_FIRST, 27.0, Palette::green(), CARD_X, -1.9)?,
        line_of_math(k, stage, SOLVED_SECOND, 27.0, Palette::green(), CARD_X, -2.7)?,
    ];
    fade_in_all(stage, &solved, 0.5)?;
    k.hold(stage)
}

/// 12_1: a coin chooses the multiplier. The average per toss is shown; the typical run waits for 12_2.
fn coin(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Mob, Vec<Mob>)> {
    k.say(stage, "12_1")?;
    let heading = title(k, stage, "When chance picks the multiplier")?;
    fade_in_all(stage, &[heading], 0.5)?;

    k.until_sentence(stage, "12_1", 2)?;
    let faces = [
        ring(stage, v(-4.3, 0.9), 1.0, Palette::green(), 4.0),
        k.text(stage, "heads", 30.0, Palette::green(), v(-4.3, 1.15))?,
        ring(stage, v(-1.4, 0.9), 1.0, Palette::red(), 4.0),
        k.text(stage, "tails", 30.0, Palette::red(), v(-1.4, 1.15))?,
    ];
    fade_in_all(stage, &faces, 0.5)?;
    k.until_sentence(stage, "12_1", 3)?;
    let multipliers = [
        line_of_math(
            k,
            stage,
            &format!(r"\times {}", m::HEADS),
            36.0,
            Palette::green(),
            -4.3,
            0.55,
        )?,
        line_of_math(
            k,
            stage,
            &format!(r"\times {}", m::TAILS),
            36.0,
            Palette::red(),
            -1.4,
            0.55,
        )?,
    ];
    fade_in_all(stage, &multipliers, 0.5)?;

    k.until_sentence(stage, "12_1", 4)?;
    let invented = k.text(stage, "invented numbers", 26.0, Palette::dim(), v(CARD_X, 1.5))?;
    fade_in_all(stage, &[invented], 0.4)?;
    k.until_sentence(stage, "12_1", 5)?;
    let no_market = k.text(stage, "no market, no investment", 26.0, Palette::dim(), v(CARD_X, 0.9))?;
    fade_in_all(stage, &[no_market], 0.4)?;

    k.until_sentence(stage, "12_1", 6)?;
    let mean = (m::HEADS + m::TAILS) / 2.0;
    let average = line_of_math(
        k,
        stage,
        &format!(
            r"\frac{{{}+{}}}{{2}}={mean:.2}:\quad +{}\%\ \text{{per toss}}",
            m::HEADS,
            m::TAILS,
            percent(mean - 1.0)
        ),
        32.0,
        Palette::gold(),
        -2.85,
        -1.2,
    )?;
    fade_in_all(stage, &[average], 0.5)?;
    k.until_sentence(stage, "12_1", 7)?;
    let tosses = k.text(
        stage,
        &format!("{} tosses", m::TOSSES),
        28.0,
        Palette::ink(),
        v(CARD_X, -0.3),
    )?;
    fade_in_all(stage, &[tosses], 0.4)?;
    k.until_sentence(stage, "12_1", 8)?;
    let question = k.text(
        stage,
        &format!("does a typical run grow {}% per toss?", percent(mean - 1.0)),
        28.0,
        Palette::hero(),
        v(0.0, -2.6),
    )?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_12_1)?;
    let mut shown = faces.to_vec();
    shown.extend(multipliers);
    shown.extend([invented, no_market, average, tosses, question]);
    Ok((heading, shown))
}

/// One bar of the head-count histogram.
fn head_bar(stage: &mut Stage<'_>, heads: u32, c: Srgb) -> Mob {
    let x = HIST_LEFT + HIST_STEP * f64::from(heads - m::SHOWN_HEADS.0);
    let top = HIST_BASE + HIST_SCALE * m::share(heads);
    stage.arena_mut().add(
        Polygon::new([
            v(x, HIST_BASE),
            v(x, top),
            v(x + HIST_STEP * 0.85, top),
            v(x + HIST_STEP * 0.85, HIST_BASE),
        ])
        .style(flat(c, 0.9))
        .build(),
    )
}

/// 12_2: the ruler shows why the typical run shrinks; the exact binomial counts show who carries the average.
fn typical_run(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "12_2")?;
    fade_out_all(stage, last, 0.4)?;

    k.until_sentence(stage, "12_2", 1)?;
    let zero = v(RULER_X, 2.45);
    let spine = seg(stage, v(RULER_X, 2.6), v(RULER_X, 0.95), Palette::dim(), 2.5);
    let ruler_name = k.text(stage, RULER_NAME, 22.0, Palette::dim(), v(RULER_X, 2.8))?;
    fade_in_all(stage, &[spine, ruler_name], 0.4)?;

    k.until_sentence(stage, "12_2", 2)?;
    let heads_end = v(zero[0] + RULER_UNIT * m::HEADS.ln(), 2.2);
    let heads = [
        arrow(stage, v(zero[0], 2.2), heads_end, Palette::green(), 5.0),
        k.text(
            stage,
            &format!("heads +{:.3}", m::HEADS.ln()),
            23.0,
            Palette::green(),
            v(heads_end[0] + 0.95, 2.2),
        )?,
    ];
    fade_in_all(stage, &heads, 0.5)?;
    k.until_sentence(stage, "12_2", 3)?;
    let tails_end = v(zero[0] + RULER_UNIT * m::TAILS.ln(), 1.7);
    let tails = [
        arrow(stage, v(zero[0], 1.7), tails_end, Palette::red(), 5.0),
        k.text(
            stage,
            &format!("tails \u{2212}{:.3}", -m::TAILS.ln()),
            23.0,
            Palette::red(),
            v(zero[0] + 0.95, 1.7),
        )?,
    ];
    fade_in_all(stage, &tails, 0.5)?;
    k.until_sentence(stage, "12_2", 4)?;
    let pair_log = (m::HEADS * m::TAILS).ln();
    let pair_end = v(zero[0] + RULER_UNIT * pair_log, 1.2);
    let pair = [
        arrow(stage, v(zero[0], 1.2), pair_end, Palette::hero(), 5.0),
        k.text(
            stage,
            &format!("one of each: \u{00d7}{:.1}", m::HEADS * m::TAILS),
            23.0,
            Palette::hero(),
            v(zero[0] + 1.35, 1.2),
        )?,
    ];
    fade_in_all(stage, &pair, 0.5)?;

    k.until_sentence(stage, "12_2", 5)?;
    let typical = line_of_math(
        k,
        stage,
        &format!(
            r"{:.1}^{{{}}}\approx {:.5}",
            m::HEADS * m::TAILS,
            m::TOSSES / 2,
            m::typical_multiple()
        ),
        31.0,
        Palette::hero(),
        CARD_X,
        2.2,
    )?;
    fade_in_all(stage, &[typical], 0.5)?;
    // Review 721 M1: that run is the middle one, and five heads either way moves the result a hundredfold.
    k.until_sentence(stage, "12_2", 6)?;
    let middle = k.text(stage, MIDDLE_RUN, 23.0, Palette::hero(), v(CARD_X, 1.72))?;
    fade_in_all(stage, &[middle], 0.4)?;
    k.until_sentence(stage, "12_2", 7)?;
    let spread = line_of_math(
        k,
        stage,
        &format!(
            r"\text{{five heads more}}:\ \times {:.1}^{{{}}}\approx {:.2}",
            m::heads_for_tails(1),
            m::SPREAD_HEADS,
            m::heads_for_tails(m::SPREAD_HEADS)
        ),
        26.0,
        Palette::hero(),
        CARD_X,
        1.25,
    )?;
    fade_in_all(stage, &[spread], 0.4)?;

    k.until_sentence(stage, "12_2", 9)?;
    let mut bars = Vec::new();
    for h in m::SHOWN_HEADS.0..=m::SHOWN_HEADS.1 {
        let colour = if h < m::above_start() {
            Palette::red()
        } else {
            Palette::green()
        };
        bars.push(head_bar(stage, h, colour));
    }
    let mark = |h: u32| HIST_LEFT + HIST_STEP * (f64::from(h - m::SHOWN_HEADS.0) + 0.4);
    let scale = [
        k.text(stage, "50", 21.0, Palette::dim(), v(mark(50), HIST_BASE - 0.25))?,
        k.text(
            stage,
            &m::above_start().to_string(),
            21.0,
            Palette::green(),
            v(mark(m::above_start()), HIST_BASE - 0.25),
        )?,
        k.text(
            stage,
            &m::RARE_HEADS.to_string(),
            21.0,
            Palette::gold(),
            v(mark(m::RARE_HEADS), HIST_BASE - 0.25),
        )?,
        k.text(
            stage,
            &format!("heads in {} tosses", m::TOSSES),
            21.0,
            Palette::dim(),
            v(-3.7, HIST_BASE - 0.6),
        )?,
        k.text(stage, "finish above the start", 21.0, Palette::green(), v(-2.0, -1.5))?,
    ];
    let above = line_of_math(
        k,
        stage,
        &format!(
            r"\text{{above the start}}:\quad {:.1}\%",
            100.0 * m::share_from(m::above_start())
        ),
        27.0,
        Palette::green(),
        CARD_X,
        0.7,
    )?;
    fade_in_all(stage, &bars, 0.5)?;
    fade_in_all(stage, &scale, 0.4)?;
    fade_in_all(stage, &[above], 0.4)?;

    k.until_sentence(stage, "12_2", 10)?;
    let average = line_of_math(
        k,
        stage,
        &format!(r"\text{{average over all runs}}:\quad {:.2}", m::average_multiple()),
        27.0,
        Palette::gold(),
        CARD_X,
        0.1,
    )?;
    fade_in_all(stage, &[average], 0.4)?;

    k.until_sentence(stage, "12_2", 11)?;
    let mut rare_bars = Vec::new();
    for h in m::RARE_HEADS..=m::SHOWN_HEADS.1 {
        rare_bars.push(head_bar(stage, h, Palette::gold()));
    }
    let rare = [
        line_of_math(
            k,
            stage,
            &format!(
                r"{}\ \text{{heads or more}}:\quad {:.1}\%\ \text{{of runs}}",
                m::RARE_HEADS,
                100.0 * m::share_from(m::RARE_HEADS)
            ),
            26.0,
            Palette::gold(),
            CARD_X,
            -0.5,
        )?,
        line_of_math(
            k,
            stage,
            &format!(
                r"\text{{but}}\ {:.1}\%\ \text{{of the average}}",
                100.0 * m::average_share_from(m::RARE_HEADS)
            ),
            26.0,
            Palette::gold(),
            CARD_X,
            -1.05,
        )?,
    ];
    fade_in_all(stage, &rare_bars, 0.4)?;
    fade_in_all(stage, &rare, 0.4)?;

    // Review 721 L3: the moral names one run's path, so one exact run is drawn where the ruler stood. Heads and tails
    // in turn, no chance needed: it ends at fifty times the pair's distance, the middle run of the card above.
    k.until_sentence(stage, "12_2", 12)?;
    let distances = m::alternating_run();
    let finish = distances.last().copied().unwrap_or(0.0);
    let run_points: Vec<Vec3> = distances
        .iter()
        .enumerate()
        .map(|(toss, &distance)| run_point(toss, distance))
        .collect();
    let end = run_point(distances.len() - 1, finish);
    let one_run = [
        seg(
            stage,
            run_point(0, 0.0),
            run_point(distances.len() - 1, 0.0),
            Palette::faint(),
            2.0,
        ),
        path(stage, run_points, Palette::hero(), 2.5),
        k.text(
            stage,
            RUN_NAME,
            22.0,
            Palette::hero(),
            v((RUN_LEFT + RUN_RIGHT) / 2.0, 2.75),
        )?,
        line_of_math(
            k,
            stage,
            &format!(r"\text{{ends at}}\ E({finish:.3})"),
            23.0,
            Palette::hero(),
            end[0] - 0.6,
            end[1] - 0.3,
        )?,
    ];
    let mut ruler = vec![spine, ruler_name];
    ruler.extend(heads);
    ruler.extend(tails);
    ruler.extend(pair);
    swap(stage, &ruler, &one_run, 0.6)?;
    let moral = k.text(
        stage,
        "the average of many runs is not one run's path",
        24.0,
        Palette::hero(),
        v(CARD_X, -2.2),
    )?;
    fade_in_all(stage, &[moral], 0.5)?;
    k.hold(stage)?;

    let mut shown = vec![typical, middle, spread, above, average, moral];
    shown.extend(one_run);
    shown.extend(bars);
    shown.extend(scale);
    shown.extend(rare_bars);
    shown.extend(rare);
    Ok(shown)
}

/// A curve from the origin of the chart to the clock, stopped at `stop`.
fn live_curve(stage: &mut Stage<'_>, clock: Mob, curve: fn(f64) -> f64, stop: f64, c: Srgb) -> fmn::Result<Mob> {
    live(stage, clock, move |s, now| {
        let end = now.clamp(1e-3, stop);
        let points: Vec<Vec3> = (0..=CURVE_SAMPLES)
            .map(|j| {
                let t = end * f64::from(j) / f64::from(CURVE_SAMPLES);
                chart(t, curve(t))
            })
            .collect();
        s.add(polyline(points).style(line_style(c, 4.0)).build())
    })
}

/// 12_3: the same start and rate, with and without a capacity. Only the early stretch is drawn before the question.
fn feedback(k: &Kit, stage: &mut Stage<'_>, heading: Mob, last: &[Mob]) -> fmn::Result<(Mob, Mob, Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "12_3")?;
    fade_out_all(stage, last, 0.4)?;
    let heading = swap_title(k, stage, heading, "One possible limit: capacity")?;

    k.until_sentence(stage, "12_3", 2)?;
    let asked = k.text(
        stage,
        "Must growth continue exponentially forever?",
        25.0,
        Palette::hero(),
        v(CARD_X, 2.25),
    )?;
    fade_in_all(stage, &[asked], 0.5)?;
    k.until_sentence(stage, "12_3", 3)?;
    let room = k.text(
        stage,
        "the rate can depend on the room left",
        23.0,
        Palette::dim(),
        v(CARD_X, 1.65),
    )?;
    let plain = line_of_math(k, stage, LAW_PLAIN, 31.0, Palette::blue(), CARD_X, 0.95)?;
    fade_in_all(stage, &[room, plain], 0.5)?;
    k.until_sentence(stage, "12_3", 4)?;
    let law = line_of_math(k, stage, LAW_ROOM, 31.0, Palette::gold(), CARD_X, 0.2)?;
    fade_in_all(stage, &[law], 0.5)?;

    k.until_sentence(stage, "12_3", 5)?;
    let mut chart_parts = vec![
        seg(stage, chart(0.0, 0.0), chart(CHART_END, 0.0), Palette::dim(), 2.5),
        seg(stage, chart(0.0, 0.0), chart(0.0, CHART_TOP), Palette::dim(), 2.5),
        seg(stage, chart(0.0, 1.0), chart(CHART_END, 1.0), Palette::faint(), 2.0),
        k.text(
            stage,
            "amount",
            21.0,
            Palette::dim(),
            v(CHART_LEFT + 0.5, chart(0.0, CHART_TOP)[1] + 0.2),
        )?,
        k.text(
            stage,
            "time \u{2192}",
            22.0,
            Palette::dim(),
            v(chart(CHART_END, 0.0)[0] - 0.45, CHART_BASE - 0.28),
        )?,
        k.text(
            stage,
            "capacity",
            20.0,
            Palette::dim(),
            v(chart(CHART_END, 1.0)[0] - 0.5, chart(0.0, 1.0)[1] - 0.2),
        )?,
        // The chart's empty upper left: the curves only reach this height far to the right.
        k.text(stage, "constant proportional rate", 22.0, Palette::blue(), v(-4.6, 1.3))?,
        k.text(stage, "capacity-limited example", 22.0, Palette::gold(), v(-4.6, 0.9))?,
    ];
    for t in TIME_TICKS {
        // Shortest form: 0.5, then 1, 2, 3, 4.
        let x = chart(t, 0.0)[0];
        chart_parts.push(seg(
            stage,
            v(x, CHART_BASE),
            v(x, CHART_BASE - 0.08),
            Palette::dim(),
            2.0,
        ));
        chart_parts.push(k.text(stage, &format!("{t}"), 20.0, Palette::dim(), v(x, CHART_BASE - 0.28))?);
    }
    fade_in_all(stage, &chart_parts, 0.5)?;
    let clock = stage.add_value_tracker(0.0);
    let curves = vec![
        live_curve(
            stage,
            clock,
            m::unlimited,
            m::unlimited_reaches(CHART_TOP),
            Palette::blue(),
        )?,
        live_curve(stage, clock, m::limited, CHART_END, Palette::gold())?,
    ];
    run(stage, clock, EARLY, 1.2)?;

    k.until_sentence(stage, "12_3", 6)?;
    let question = k.text(
        stage,
        "how long before you could tell them apart?",
        25.0,
        Palette::hero(),
        v(CARD_X, -1.85),
    )?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_12_3)?;

    let cards = vec![asked, room, plain, law, question];
    let mut keep = chart_parts;
    keep.extend(curves);
    Ok((heading, clock, cards, keep))
}

/// 12_4: the curves part slowly, the reciprocal decays exactly, and growth estimates inherit the caution.
fn parting(k: &Kit, stage: &mut Stage<'_>, clock: Mob, cards: &[Mob], kept: Vec<Mob>) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "12_4")?;
    fade_out_all(stage, cards, 0.4)?;

    k.until_sentence(stage, "12_4", 1)?;
    let ratio = |t: f64| m::unlimited(t) / m::limited(t);
    let first = line_of_math(
        k,
        stage,
        &format!(r"t=0.5:\quad \text{{ratio}}\ {:.2}", ratio(0.5)),
        28.0,
        Palette::ink(),
        CARD_X,
        2.25,
    )?;
    fade_in_all(stage, &[first], 0.4)?;
    k.until_sentence(stage, "12_4", 2)?;
    let second = line_of_math(
        k,
        stage,
        &format!(r"t=1:\quad \text{{ratio}}\ {:.2}", ratio(1.0)),
        28.0,
        Palette::ink(),
        CARD_X,
        1.65,
    )?;
    fade_in_all(stage, &[second], 0.4)?;
    run(stage, clock, 1.0, 0.8)?;
    k.until_sentence(stage, "12_4", 3)?;
    run(stage, clock, CHART_END, 3.2)?;

    k.until_sentence(stage, "12_4", 4)?;
    // Review 721 L1: exact data would separate them at once; noisy early data hardly can.
    let early = k.text(
        stage,
        "noisy early data can hardly tell them apart",
        24.0,
        Palette::hero(),
        v(CARD_X, 1.05),
    )?;
    fade_in_all(stage, &[early], 0.4)?;

    k.until_sentence(stage, "12_4", 6)?;
    let reciprocal = line_of_math(k, stage, RECIPROCAL, 25.0, Palette::green(), CARD_X, 0.15)?;
    let gaps: Vec<String> = m::CAP_TIMES
        .iter()
        .map(|&t| format!("{:.2}", m::reciprocal_gap(t)))
        .collect();
    let gap_row = line_of_math(
        k,
        stage,
        &format!(r"\text{{every half unit}}:\ {},\ \ldots", gaps[..3].join(r",\ ")),
        23.0,
        Palette::dim(),
        CARD_X,
        -0.75,
    )?;
    fade_in_all(stage, &[reciprocal, gap_row], 0.5)?;

    k.until_sentence(stage, "12_4", 7)?;
    let economy = k.text(
        stage,
        "an economy's growth rate:",
        24.0,
        Palette::ink(),
        v(CARD_X, -1.6),
    )?;
    fade_in_all(stage, &[economy], 0.4)?;
    k.until_sentence(stage, "12_4", 8)?;
    let after = k.text(
        stage,
        "measured after the fact; conditions can change",
        23.0,
        Palette::dim(),
        v(CARD_X, -2.15),
    )?;
    fade_in_all(stage, &[after], 0.4)?;
    k.hold(stage)?;

    let mut shown = kept;
    shown.extend([first, second, early, reciprocal, gap_row, economy, after]);
    Ok(shown)
}

/// 12_5: each assumption and its replacement.
fn what_each_buys(k: &Kit, stage: &mut Stage<'_>, heading: Mob, last: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "12_5")?;
    fade_out_all(stage, last, 0.4)?;
    swap_title(k, stage, heading, "What each assumption buys")?;

    // Review 721 M2: chance splits into two rows. Averaging the multipliers is right for the average over runs, and
    // averaging the distances is right for a typical run.
    let rows: [(usize, &str, &str, Srgb); 6] = [
        (1, "constant rate", r"E(k\,t)", Palette::ink()),
        (
            2,
            "rate moves with the clock",
            r"E\left(\int_0^t r(s)\,ds\right)",
            Palette::green(),
        ),
        (
            3,
            "money moves in or out",
            r"\text{one exponential per payment}",
            Palette::gold(),
        ),
        (
            5,
            "chance: the average over runs",
            r"\text{average the multipliers}",
            Palette::teal(),
        ),
        (
            6,
            "chance: a typical run",
            r"\text{average the distances on the ruler}",
            Palette::teal(),
        ),
        (
            7,
            "the amount feeds back",
            r"\text{a different equation}",
            Palette::blue(),
        ),
    ];
    for (i, (sentence, assumption, replacement, colour)) in rows.into_iter().enumerate() {
        k.until_sentence(stage, "12_5", sentence)?;
        let y = 2.25 - 0.78 * i as f64;
        let row = [
            k.text_left(stage, assumption, 27.0, colour, v(-6.3, y))?,
            line_of_math(k, stage, replacement, 28.0, colour, 3.4, y)?,
        ];
        fade_in_all(stage, &row, 0.4)?;
    }
    k.until_sentence(stage, "12_5", 8)?;
    let arithmetic = k.text(
        stage,
        "ways of doing the arithmetic",
        27.0,
        Palette::ink(),
        v(0.0, -2.45),
    )?;
    fade_in_all(stage, &[arithmetic], 0.4)?;
    k.until_sentence(stage, "12_5", 9)?;
    let note = k.text(
        stage,
        "not forecasts, and not advice about money",
        27.0,
        Palette::hero(),
        v(0.0, -3.05),
    )?;
    fade_in_all(stage, &[note], 0.4)?;
    Ok(())
}

impl SceneConstruct for MoneyLimits {
    fn name(&self) -> &str {
        "11_money_limits"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (heading, shown) = credits(&k, stage)?;
        let (cards, chart_parts) = multipliers(&k, stage, heading, &shown)?;
        withdrawal(&k, stage, &cards, &chart_parts)?;
        k.finish(stage)
    }
}

impl SceneConstruct for ChanceAndLimits {
    fn name(&self) -> &str {
        "12_chance_and_limits"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (heading, shown) = coin(&k, stage)?;
        let shown = typical_run(&k, stage, &shown)?;
        let (heading, clock, cards, kept) = feedback(&k, stage, heading, &shown)?;
        let shown = parting(&k, stage, clock, &cards, kept)?;
        what_each_buys(&k, stage, heading, &shown)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ending_draw::{assert_in_order, scene_source};
    const SOURCE: &str = include_str!("money_limits.rs");

    #[test]
    fn the_cues_run_in_film_order() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.say(stage, "11_1")?;"#,
                r#"k.say(stage, "11_2")?;"#,
                r#"k.say(stage, "11_3")?;"#,
                r#"k.say(stage, "12_1")?;"#,
                r#"k.say(stage, "12_2")?;"#,
                r#"k.say(stage, "12_3")?;"#,
                r#"k.say(stage, "12_4")?;"#,
                r#"k.say(stage, "12_5")?;"#,
            ],
        );
    }
    use super::{
        CHART_TOP, DROP_OFFSET, NINETY_AT, RULER_NAME, THINK_11_2, THINK_12_1, THINK_12_3, TIME_TICKS, YEAR_X, chart,
        money_at, run_point,
    };
    use crate::changing_rate_model as m;

    #[test]
    fn each_prediction_is_held_before_its_answer() {
        assert_in_order(
            SOURCE,
            &[
                "does the order still not matter?",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_11_2)?;",
                r#"k.say(stage, "11_3")?;"#,
                "m::with_withdrawal(m::GOOD, m::BAD)",
                "does a typical run grow",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_12_1)?;",
                r#"k.say(stage, "12_2")?;"#,
                "m::typical_multiple()",
                "how long before you could tell them apart?",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_12_3)?;",
                r#"k.say(stage, "12_4")?;"#,
                "run(stage, clock, CHART_END, 3.2)?;",
            ],
        );
        // Review 721 M10 proposed three to four seconds for 11_2 and 12_1, two to three for 12_3. Audio-review decides.
        const { assert!(THINK_11_2 >= 3.0) };
        const { assert!(THINK_11_2 <= 4.0) };
        const { assert!(THINK_12_1 >= 3.0) };
        const { assert!(THINK_12_1 <= 4.0) };
        const { assert!(THINK_12_3 >= 2.0) };
        const { assert!(THINK_12_3 <= 3.0) };
    }
    #[test]
    fn each_continuous_rate_is_shown_as_an_input_of_e_and_no_logarithm_is_written() {
        // Review 721 M6: the natural logarithm is first named in 15_2.
        assert!(!scene_source(SOURCE).contains(r"\ln"));
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "11_2", 6)?;"#,
                r#"&format!(r"E({good_rate:.5})={}", m::GOOD)"#,
                r#"k.until_sentence(stage, "11_2", 8)?;"#,
                r#"&format!(r"E({bad_rate:.5})={}", m::BAD)"#,
                r#"k.until_sentence(stage, "11_2", 9)?;"#,
                r#"&format!(r"{good_rate:.5}{bad_rate:.5}={both_rate:.5}")"#,
                r#"&format!(r"E({both_rate:.5})={both:.2}")"#,
                r#"k.until_sentence(stage, "11_2", 10)?;"#,
            ],
        );
        let (good, bad) = (m::continuous_rate(m::GOOD), m::continuous_rate(m::BAD));
        assert_eq!(format!("{good:.5} {bad:.5}"), "0.18232 -0.10536");
        assert!((good + bad - m::continuous_rate(m::GOOD * m::BAD)).abs() < 1e-15);
    }
    #[test]
    fn the_two_withdrawals_stand_apart_and_the_ninety_clears_them() {
        // Review 721 M9.
        assert_in_order(
            SOURCE,
            &[
                "node_label(k, stage, c, 1, NINETY_AT.0, NINETY_AT.1, Palette::teal())",
                r#"k.say(stage, "11_3")?;"#,
                "fade_out_all(stage, &chart_parts[..3], 0.3)?;",
                "beside(money_at(1, a), -DROP_OFFSET)",
                "beside(money_at(1, b), -DROP_OFFSET)",
                "beside(money_at(1, p), DROP_OFFSET)",
                "beside(money_at(1, q), DROP_OFFSET)",
            ],
        );
        let [a, b, _] = m::with_withdrawal(m::GOOD, m::BAD);
        let [p, q, _] = m::with_withdrawal(m::BAD, m::GOOD);
        let (good_low, bad_high) = (money_at(1, b)[1], money_at(1, p)[1]);
        // The drops share a stretch of the column, so without the offset their strokes would coincide.
        assert!(good_low < bad_high && money_at(1, q)[1] < good_low && bad_high < money_at(1, a)[1]);
        let ninety = money_at(1, m::BALANCE * m::BAD)[0] + NINETY_AT.0;
        for drop_x in [YEAR_X[1] - DROP_OFFSET, YEAR_X[1] + DROP_OFFSET] {
            // About half the width of "90" at 25 points, and room to spare.
            assert!((ninety - drop_x).abs() > 0.12 + 0.1, "{drop_x}");
        }
        // The 11_2 list keeps "108" third, so the 11_3 fade takes it with both paths.
        assert_in_order(
            SOURCE,
            &[
                "let chart_parts = vec![\n        good_path,\n        bad_path,\n        good_labels[1],\n        taken[0],",
            ],
        );
    }
    #[test]
    fn the_middle_run_and_its_spread_follow_their_sentences() {
        // Review 721 M1 and W2.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "12_2", 5)?;"#,
                "m::typical_multiple()",
                r#"k.until_sentence(stage, "12_2", 6)?;"#,
                "MIDDLE_RUN",
                r#"k.until_sentence(stage, "12_2", 7)?;"#,
                "m::heads_for_tails(m::SPREAD_HEADS)",
                r#"k.until_sentence(stage, "12_2", 9)?;"#,
                r#"k.until_sentence(stage, "12_2", 10)?;"#,
                "m::average_multiple()",
                r#"k.until_sentence(stage, "12_2", 11)?;"#,
                r#"k.until_sentence(stage, "12_2", 12)?;"#,
            ],
        );
        assert_eq!(format!("{:.2}", m::average_multiple()), "131.50");
        assert_eq!(format!("{:.2}", m::heads_for_tails(m::SPREAD_HEADS)), "97.66");
    }
    #[test]
    fn the_ruler_is_named_in_e_units_and_one_alternating_run_is_drawn_under_the_moral() {
        // Review 721 H3 and L3.
        assert!(RULER_NAME.contains("input of E"));
        assert!(!scene_source(SOURCE).contains("distance on the ruler\""));
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "12_2", 12)?;"#,
                "m::alternating_run()",
                "swap(stage, &ruler, &one_run, 0.6)?;",
                "the average of many runs is not one run's path",
            ],
        );
        let run = m::alternating_run();
        let finish = *run.last().unwrap();
        assert_eq!(run.len(), m::TOSSES as usize + 1);
        assert!((finish - m::typical_multiple().ln()).abs() < 1e-12);
        assert_eq!(format!("{finish:.3}"), "-5.268");
        // The whole run stays between the title and the histogram, left of the cards.
        for (toss, &distance) in run.iter().enumerate() {
            let p = run_point(toss, distance);
            assert!(p[1] < 2.6 && p[1] > 0.0 && p[0] < 0.0, "{toss}");
        }
    }
    #[test]
    fn the_capacity_chart_has_time_marks_and_room_above_the_capacity() {
        // Review 721 L2.
        assert!(TIME_TICKS.contains(&0.5) && TIME_TICKS.contains(&1.0));
        assert!(m::unlimited_reaches(CHART_TOP) > m::unlimited_top());
        assert!((m::unlimited(m::unlimited_reaches(CHART_TOP)) - CHART_TOP).abs() < 1e-12);
        assert!(chart(0.0, CHART_TOP)[1] + 0.2 < 3.0);
        assert!(scene_source(SOURCE).contains("m::unlimited_reaches(CHART_TOP)"));
        assert!(scene_source(SOURCE).contains("noisy early data can hardly tell them apart"));
    }
    #[test]
    fn the_assumption_table_splits_chance_in_two() {
        // Review 721 M2.
        assert_in_order(
            SOURCE,
            &[
                "(1, \"constant rate\"",
                "3,\n            \"money moves in or out\"",
                "5,\n            \"chance: the average over runs\"",
                r#"r"\text{average the multipliers}""#,
                "6,\n            \"chance: a typical run\"",
                r#"r"\text{average the distances on the ruler}""#,
                "7,\n            \"the amount feeds back\"",
                r#"k.until_sentence(stage, "12_5", 8)?;"#,
                r#"k.until_sentence(stage, "12_5", 9)?;"#,
            ],
        );
    }
    #[test]
    fn only_the_early_stretch_is_drawn_before_the_question() {
        let feedback = scene_source(SOURCE)
            .split("fn feedback")
            .nth(1)
            .unwrap()
            .split("fn parting")
            .next()
            .unwrap();
        assert!(feedback.contains("run(stage, clock, EARLY, 1.2)?;"));
        assert!(!feedback.contains("CHART_END, 3.2"));
    }
    #[test]
    fn the_film_says_this_is_not_advice_on_screen() {
        assert!(scene_source(SOURCE).contains("not forecasts, and not advice about money"));
        assert!(scene_source(SOURCE).contains("invented numbers"));
    }
}
