//! Chapter 06, the bet. From 05's two rows the coin goes and the second row
//! stays as the bet: if every curve earns crossings at the coin's rate, a
//! crossing on any curve is worth π cm (06_1). The coast comes back with
//! its seven crossings at the first placement: what length does the bet
//! give? (06_2, a prediction.) 7 × π cm, about 22.0 cm (06_3). Only then
//! the envelope opens: 25.8 cm (06_4). The two lengths laid side by side,
//! to scale: the bet 3.8 cm short, about 15% (06_5). Slid 0.5 cm up, inside
//! 04's run of nine, the same coast gives 9 × π, about 28.3 cm, 2.5 cm too
//! long (06_6), and slid back, seven again (06_7). One count at one
//! placement was never the plan: the plan is π times the average count
//! over every slide and every direction (06_8), which the coast cannot give
//! yet, so the bet is tried on something simpler first (06_9). The sheet,
//! the coast's crossings, its count and the bet's bar are drawn from the
//! model at the placement shown, every frame. Starts where chapter 05
//! ends; integrated by root from `docs/candidate-06-07.md`.

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::smooth;
use fmn::prelude::*;

use super::a_coin_that_never_changes::meetings;
use super::draw::{
    EVERY, card_panel, card_point, dim_line, fade_in_all, fade_to, fail, flat, line_style, now, path, say, stand, stop,
    swap, think, together,
};
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, ENVELOPE, ENVELOPE_AT, Move, Placement, Pose, QUESTION, QUESTION_AT, WORD_GAP,
    at_once, faded_line, glide, tracked,
};
use super::sheet::{self, COUNT_SIZE, LABEL_SIZE, LOW_Y, Sheet, THINK_AT, crossing_dot};
use crate::facts;
use crate::model::{self, COIN, Curve, GAP, P2};
use crate::palette;

pub struct TheBet {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts);
// 03's arrow, as 04 copies it
const ARROW: &str = r"\rightarrow\ ?\text{ cm}";
const ASK: &str = "?";
const TAG: &str = "the bet";

/// What the voice says the model shows: the coin's two crossings (05's
/// rows, `coin.crossings`), the coast's first count (06_2, 06_3, 06_6,
/// `open.crossings`) and its count once the sheet has slid up (06_6,
/// `bet.nine_crossings`).
const SAID_COIN: usize = 2;
const SAID_FIRST: usize = 7;
const SAID_NINE: usize = 9;

/// How far 06_6 slides the sheet up from the first placement, cm: into
/// 04's run of nine (it lasts from about 0.40 to 0.99 cm of 04's slide),
/// passing no other count, to a slide whose lines keep 0.07 cm from every
/// turning height and 0.1 cm from every tangent point (tested), so no
/// crossing there sits on a near miss. `facts.rs` prints
/// `bet.nine_slide` from it.
pub const NINE_BY: f64 = 0.5;
/// Steps the slide is read at for its runs: one ten-thousandth of a
/// centimetre each.
const RUN_STEPS: usize = 5000;

/// One crossing's worth on the bet, cm: the coin's rim over its two
/// crossings, π × [`COIN`] / 2.
fn rate() -> f64 {
    PI * COIN / 2.0
}

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
/// What 05 dimmed the card and coast to, and the card's own opacity
/// (`card_panel`'s) it comes back to.
const DIMMED: f64 = 0.25;
const CARD_OPACITY: f64 = 0.55;
/// 05's coin at rest where its roll ended, its trail and label and its two
/// rows, copied from 05 (whose items are private) to rebuild its end.
const RADIUS: f64 = COIN / 2.0;
const COIN_FILL: f64 = 0.16;
const RIM_W: f64 = 4.0;
const TICK_W: f64 = 3.0;
const TICK_FROM: f64 = 0.35;
const TRAIL_W: f64 = 5.0;
const ROLL_X: f64 = 0.0;
const ROLL_LABEL_UP: f64 = 0.4;
const EQUATION_SIZE: f64 = 28.0;
const ROWS_AT: [Vec3; 2] = [[1.8, LOW_Y + 0.22, 0.0], [1.8, LOW_Y - 0.22, 0.0]];
/// "true length" under the envelope (`placement::envelope`'s size), and
/// the length it reveals beside it.
const SEALED_SIZE: f64 = 20.0;
const REVEAL_GAP: f64 = 0.2;
/// How far above the envelope's top edge its flap's point rises, open.
const FLAP_UP: f64 = 0.3;
/// 04's gap between the count's word and the arrow after it.
const ARROW_GAP: f64 = 0.25;

/// 06_5's chart, in the band under the coast, frame units: a bar for each
/// length from [`CHART_X`], [`CHART_K`] frame units to the centimetre, the
/// true length's row over the bet's; the names right-aligned to their
/// left; the values in a column right of the longest bar the chapter draws
/// (nine crossings' worth); the difference beyond, between the rows; and a
/// bracket under the bet's bar from its end to the true length's.
const CHART_X: f64 = -4.4;
const CHART_K: f64 = 0.2;
const ROW_Y: [f64; 2] = [-1.6, -1.9];
const CHART_SIZE: f64 = 18.0;
const NAME_RIGHT: f64 = CHART_X - 0.15;
const VALUE_X: f64 = 1.45;
const DIFF_X: f64 = 2.75;
const DIFF_Y: f64 = (ROW_Y[0] + ROW_Y[1]) / 2.0;
const DIFF_GAP: f64 = 0.2;
const BAR_W: f64 = 6.0;
const BRACKET_W: f64 = 2.0;
const BRACKET_DROP: f64 = 0.14;
const BRACKET_TICK: f64 = 0.07;
/// 06_8's equation, in the band under the coast.
const EQUATION_AT: Vec3 = [0.6, LOW_Y, 0.0];

// ------------------------------------------------------------------ timing

const OPEN_S: f64 = 1.0;
const FADE_S: f64 = 0.4;
const FLAP_S: f64 = 1.0;
const GROW_S: f64 = 1.2;
const SLIDE_S: f64 = 2.5;

/// The sheet's three placements: the first, 06_6's slide up into the run
/// of nine, and 06_7's slide back.
struct Plan {
    first: Pose,
    up: Move,
    down: Move,
}

fn plan(coast: &Curve, by: f64) -> Plan {
    let first = Pose::new(0.0, sheet::shown_slide(coast, 0.0));
    let up = Move::slide(first, by);
    let down = Move::slide(up.end(), -by);
    Plan { first, up, down }
}

/// Where 05's roll left the coin's centre: one full turn along the line
/// over the first placement's middle line, under the next.
fn coin_end(slide: f64) -> P2 {
    [ROLL_X + TAU * RADIUS, slide + GAP + RADIUS]
}

/// The coast's crossings at `pose`.
fn count_at(coast: &Curve, pose: Pose) -> usize {
    sheet::crossings(coast, pose.beta, pose.slide).len()
}

/// The coast's count at the placement shown, for the live counter.
fn live_counted(coast: &Curve, place: Placement) -> impl Fn(&fmn::mobject::Stage) -> usize + 'static {
    let coast = coast.clone();
    move |s| count_at(&coast, place.pose(s))
}

/// The bet's length at the placement shown, π cm for each crossing, for
/// the live bar and bracket.
fn live_bet(coast: &Curve, place: Placement) -> impl Fn(&fmn::mobject::Stage) -> f64 + 'static {
    let counted = live_counted(coast, place);
    move |s| counted(s) as f64 * rate()
}

/// Where a chart bar `cm` long ends, frame x.
fn bar_end(cm: f64) -> f64 {
    CHART_X + CHART_K * cm
}

/// The chapter's words that carry numbers, each from the facts, refused
/// unless the model gives what the voice says: 05's coin at rest crossing
/// twice, 7 at the first placement, 9 after the slide up, the slide up
/// passing only 7 then 9 and the slide back only 9 then 7; every length,
/// difference and the percentage rounded as the facts round them; and each
/// shown difference the difference of the shown lengths.
struct Words {
    rows: [String; 2],
    rim: String,
    estimates: [String; 2],
    length: String,
    bets: [String; 2],
    short: String,
    percent: String,
    long: String,
    equation: String,
}

fn words(coast: &Curve, plan: &Plan) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;

    // 05's rows, as 05 builds them, and its coin at rest
    let (coins, rim, worth) = (get("coin.crossings")?, get("coin.rim")?, get("coin.worth")?);
    let slide = plan.first.slide;
    let resting = meetings(coin_end(slide), RADIUS, slide).len();
    if coins != SAID_COIN.to_string() || resting != SAID_COIN {
        return Err(format!(
            "05's rows say {SAID_COIN} crossings; the facts say {coins}, the coin at rest meets the lines {resting} times"
        ));
    }
    if rim != fixed(PI * COIN, 2) || worth != fixed(rate(), 2) {
        return Err(format!("the coin's rim and worth are {rim} and {worth} in the facts"));
    }

    // the counts, and what the slides pass through
    let (first, nine) = (count_at(coast, plan.first), count_at(coast, plan.up.end()));
    for (key, said, got) in [
        ("open.crossings", SAID_FIRST, first),
        ("bet.nine_crossings", SAID_NINE, nine),
    ] {
        if get(key)? != said.to_string() || got != said {
            return Err(format!(
                "the voice says {said} for {key}; the facts say {}, the model {got}",
                get(key)?
            ));
        }
    }
    let seen = |mv: &Move| -> Vec<usize> { placement::runs(coast, mv, RUN_STEPS).iter().map(|r| r.count).collect() };
    let (up, down) = (seen(&plan.up), seen(&plan.down));
    if up != [SAID_FIRST, SAID_NINE] || down != [SAID_NINE, SAID_FIRST] {
        return Err(format!(
            "06_6 slides from seven to nine and 06_7 back; the slides pass {up:?} and {down:?}"
        ));
    }
    if get("bet.nine_slide")? != fixed(plan.up.end().slide, 2) {
        return Err(format!(
            "the sheet slides to {}; the facts say {}",
            plan.up.end().slide,
            get("bet.nine_slide")?
        ));
    }

    // the lengths, as the facts round them
    let length = coast.length;
    let (seven_cm, nine_cm) = (SAID_FIRST as f64 * rate(), SAID_NINE as f64 * rate());
    for (key, want) in [
        ("coast.length", fixed(length, 1)),
        ("open.estimate", fixed(seven_cm, 1)),
        ("bet.nine_estimate", fixed(nine_cm, 1)),
        ("open.short_by", fixed(length - seven_cm, 1)),
        ("bet.nine_long_by", fixed(nine_cm - length, 1)),
        ("open.error_percent", fixed((seven_cm - length) / length * 100.0, 0)),
    ] {
        if get(key)? != want {
            return Err(format!("{key}: the model gives {want}, the facts {}", get(key)?));
        }
    }
    // the shown differences follow from the shown lengths
    let shown = |key: &str| -> Result<f64, String> { get(key)?.parse::<f64>().map_err(|e| format!("{key}: {e}")) };
    let (l, s, n) = (
        shown("coast.length")?,
        shown("open.estimate")?,
        shown("bet.nine_estimate")?,
    );
    let (short, long) = (get("open.short_by")?, get("bet.nine_long_by")?);
    if s >= l || l >= n || fixed(l - s, 1) != short || fixed(n - l, 1) != long {
        return Err(format!(
            "06_5 and 06_6 show {s} and {n} against {l}, short by {short} and long by {long}"
        ));
    }
    let error = get("open.error_percent")?;
    let percent = error
        .strip_prefix('-')
        .ok_or_else(|| format!("06_5 says short; the error is {error}%"))?
        .to_owned();
    if fixed((l - s) / l * 100.0, 0) != percent {
        return Err(format!("06_5 shows {short} cm of {l} cm as about {percent}%"));
    }

    let (seven_tex, nine_tex) = (get("open.estimate")?, get("bet.nine_estimate")?);
    let length_cm = get("coast.length")?;
    Ok(Words {
        rows: [
            format!(r"{SAID_COIN}\text{{ crossings}} \leftrightarrow {rim}\text{{ cm}}"),
            format!(r"1\text{{ crossing}} \leftrightarrow \pi\text{{ cm}} \approx {worth}\text{{ cm}}"),
        ],
        rim: format!("{rim} cm"),
        estimates: [
            format!(r"\rightarrow\ {SAID_FIRST} \times \pi\text{{ cm}} \approx {seven_tex}\text{{ cm}}"),
            format!(r"\rightarrow\ {SAID_NINE} \times \pi\text{{ cm}} \approx {nine_tex}\text{{ cm}}"),
        ],
        length: format!("{length_cm} cm"),
        bets: [format!("{seven_tex} cm"), format!("{nine_tex} cm")],
        short: format!("{short} cm short"),
        percent: format!("about {percent}%"),
        long: format!("{long} cm too long"),
        equation: format!(r"\pi \times \text{{average count}} = {length_cm}\text{{ cm}}\ ?"),
    })
}

// --------------------------------------------------------- what is drawn

/// 05's coin at rest at `c`: its faint disc, its rim and its turn mark,
/// pointing down at the line it rolled along. Detached.
fn coin_at_rest(stage: &mut Stage<'_>, c: P2) -> Vec<Mob> {
    let (at, r) = (card_point(c), model::CARD_UNIT * RADIUS);
    let disc = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(flat(palette::coin(), COIN_FILL))
        .build();
    let rim = Circle::new()
        .radius(r)
        .arc_center(at)
        .style(line_style(palette::coin(), RIM_W))
        .build();
    let mark = polyline(vec![
        card_point([c[0], c[1] - TICK_FROM * RADIUS]),
        card_point([c[0], c[1] - RADIUS]),
    ])
    .style(line_style(Palette::ink(), TICK_W))
    .build();
    [disc, rim, mark].map(|m| stage.arena_mut().add(m)).to_vec()
}

/// The envelope's flap, `open` of the way from shut (its point just under
/// the body's top edge, as `placement::envelope` draws it) to open (its
/// point [`FLAP_UP`] over that edge).
fn flap_points(open: f64) -> Vec<Vec3> {
    let ([w, h], [x, y, _]) = (ENVELOPE, ENVELOPE_AT);
    let (shut, up) = (y - 0.05, y + h / 2.0 + FLAP_UP);
    vec![
        v(x - w / 2.0, y + h / 2.0),
        v(x, shut + (up - shut) * open.clamp(0.0, 1.0)),
        v(x + w / 2.0, y + h / 2.0),
    ]
}

/// A chart bar on `row`, `cm(s)` long as far as `grown` holds (0 to 1), at
/// the opacity `alpha` holds. In the scene.
fn live_bar(
    stage: &mut Stage<'_>,
    row: f64,
    color: Srgb,
    cm: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    grown: Mob,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, length) = (tracked(s, alpha), cm(s) * tracked(s, grown));
        let bar: Vec<VMobject> = (length > 1e-6)
            .then(|| {
                polyline(vec![v(CHART_X, row), v(bar_end(length), row)])
                    .style(faded_line(color, BAR_W, o))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(bar))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// Where the bet's bar (`bet` cm) and the true length's (`length` cm) end,
/// frame x, the bet's first.
fn gap_ends(length: f64, bet: f64) -> [f64; 2] {
    [bar_end(bet), bar_end(length)]
}

/// The chart's difference: an upright at the true length's end, from its
/// bar down past the bet's, and a bracket just under the bet's bar from
/// the bet's end (`bet(s)` cm) to the true length's (`length` cm), at the
/// opacity `alpha` holds. In the scene.
fn live_gap(
    stage: &mut Stage<'_>,
    length: f64,
    bet: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
    alpha: Mob,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let [a, b] = gap_ends(length, bet(s));
        let y = ROW_Y[1] - BRACKET_DROP;
        let parts: Vec<VMobject> = [
            vec![v(b, ROW_Y[0]), v(b, y)],
            vec![v(a, y), v(b, y)],
            vec![v(a, y), v(a, y + BRACKET_TICK)],
        ]
        .into_iter()
        .map(|pts| polyline(pts).style(faded_line(Palette::ink(), BRACKET_W, o)).build())
        .collect();
        s.add(v_group(parts))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A text moved (and scaled by `scale`) so its `edge` lands on `to`, `rt`
/// seconds.
fn carry(m: Mob, scale: f64, to: Vec3, edge: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .scale(scale)?
        .move_to(to, edge)?)
}

impl SceneConstruct for TheBet {
    fn name(&self) -> &str {
        "06_the_bet"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let plan = plan(&coast, NINE_BY);
        let words = words(&coast, &plan).map_err(fail)?;
        let slide = plan.first.slide;

        // where 05 ended: the dimmed card and coast, the sheet at the first
        // placement, the envelope, the coin at rest after its roll with its
        // two touching points and count, the rim laid out and labelled, and
        // the two rows
        let card = card_panel(stage)?;
        stage.set_family_opacity(card, DIMMED);
        stage.add_to_scene(card)?;
        let pieces: Vec<Mob> = coast
            .parts
            .iter()
            .map(|p| {
                path(
                    stage,
                    &coast.between(p.start, p.end(), EVERY),
                    palette::coast(),
                    COAST_W,
                )
            })
            .collect();
        for &m in &pieces {
            stage.set_color_lanes(m, "stroke_rgba", None, Some(DIMMED), true);
            stage.add_to_scene(m)?;
        }
        let panel = Sheet::new(stage, slide, 0.0)?.panel;
        stage.add_to_scene(panel)?;
        let place = Placement::new(stage, plan.first);
        let lines_alpha = stage.add_value_tracker(1.0);
        placement::live_lines(stage, place, lines_alpha)?;
        let dots_alpha = stage.add_value_tracker(0.0);
        placement::live_dots(stage, place, &coast, dots_alpha)?;
        let envelope = placement::envelope(&kit, stage)?;
        for &m in &envelope {
            stage.add_to_scene(m)?;
        }
        let &[body, flap, sealed] = envelope.as_slice() else {
            return Err(fail("the envelope is a body, a flap and a label".into()));
        };
        let rest = coin_end(slide);
        let rolled_from = [ROLL_X, rest[1] - RADIUS];
        let trail = path(
            stage,
            &[rolled_from, [rest[0], rolled_from[1]]],
            palette::coin(),
            TRAIL_W,
        );
        stage.add_to_scene(trail)?;
        let mut coin = coin_at_rest(stage, rest);
        for m in meetings(rest, RADIUS, slide) {
            coin.push(crossing_dot(stage, m.at));
        }
        for &m in &coin {
            stage.add_to_scene(m)?;
        }
        let word_at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
        let coin_alpha = stage.add_value_tracker(1.0);
        let resting = meetings(rest, RADIUS, slide).len();
        placement::live_count(&kit, stage, move |_| resting, coin_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        placement::live_word(&kit, stage, CROSSINGS, coin_alpha, COUNT_SIZE, word_at)?;
        let label_at = [(ROLL_X + rest[0]) / 2.0, rolled_from[1] + ROLL_LABEL_UP];
        let rim = kit.text(stage, &words.rim, LABEL_SIZE, palette::coin(), card_point(label_at))?;
        stage.add_to_scene(rim)?;
        let rows = [
            kit.tex(stage, &words.rows[0], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[0])?,
            kit.tex(stage, &words.rows[1], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[1])?,
        ];
        for m in rows {
            stage.add_to_scene(m)?;
        }
        // the word the coast's count will show, never drawn: where it ends
        // places the arrow after it
        let probe = kit.text(stage, CROSSINGS, COUNT_SIZE, palette::touch(), word_at)?;
        stage.move_to(probe, word_at, LEFT);

        // 06_1: the coin, its count, its rim and the first row go; the
        // second stays, the bet. On "every curve" the card and coast come
        // back
        let line = say(&kit, stage, "06_1")?;
        line.wait_for(stage, "Now a bet")?;
        let mut gone = coin;
        gone.extend([trail, rim, rows[0]]);
        at_once(stage, &[], &gone, vec![tween(coin_alpha, 0.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "every curve")?;
        let mut brighten = vec![fade_to(card, CARD_OPACITY, OPEN_S)?];
        for &m in &pieces {
            brighten.push(dim_line(m, 1.0, OPEN_S)?);
        }
        together(stage, brighten)?;
        line.wait_for(stage, "a crossing on any curve")?;
        let tag = kit.text(stage, TAG, CHART_SIZE, Palette::ink(), ROWS_AT[1])?;
        stage.next_to(tag, rows[1], LEFT, 0.3, ORIGIN);
        fade_in_all(stage, &[tag], FADE_S)?;

        // 06_2: the coast's seven crossings and their count; what length?
        let line = say(&kit, stage, "06_2")?;
        line.wait_for(stage, "seven crossings")?;
        let count_alpha = stage.add_value_tracker(0.0);
        let counted = live_counted(&coast, place);
        placement::live_count(&kit, stage, counted, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        placement::live_word(&kit, stage, CROSSINGS, count_alpha, COUNT_SIZE, word_at)?;
        together(
            stage,
            vec![tween(dots_alpha, 1.0, FADE_S)?, tween(count_alpha, 1.0, FADE_S)?],
        )?;
        line.wait_for(stage, "What length")?;
        let arrow = kit.tex(
            stage,
            ARROW,
            COUNT_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            word_at,
        )?;
        stage.next_to(arrow, probe, RIGHT, ARROW_GAP, ORIGIN);
        fade_in_all(stage, &[arrow], FADE_S)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 06_3: seven times π, about 22.0 cm; the envelope stays shut
        let line = say(&kit, stage, "06_3")?;
        line.wait_for(stage, "Seven times pi")?;
        let seven = kit.tex(stage, &words.estimates[0], COUNT_SIZE, Palette::ink(), &[], word_at)?;
        stage.next_to(seven, probe, RIGHT, ARROW_GAP, ORIGIN);
        swap(stage, &[q, arrow], &[seven], FADE_S)?;

        // 06_4: the envelope opens; the true length beside its label
        let line = say(&kit, stage, "06_4")?;
        line.wait_for(stage, "open the envelope")?;
        let opened = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |s, _| {
                let f = s.tracker_value(opened).unwrap_or(0.0);
                let _ = s.set_points(flap, polyline(flap_points(f)).build().points());
            },
            true,
        )?;
        stage.play(glide(opened, 1.0, FLAP_S, smooth)?)?;
        stop(stage, anchor);
        line.wait_for(stage, "twenty-five point eight")?;
        let length = kit.text(stage, &words.length, SEALED_SIZE, palette::length(), ENVELOPE_AT)?;
        stage.next_to(length, sealed, RIGHT, REVEAL_GAP, ORIGIN);
        fade_in_all(stage, &[length], FADE_S)?;

        // 06_5: side by side, to scale. The envelope's label and length
        // move into the chart before the bars grow. Dim the geometric
        // experiment while the labels cross it, then restore it unchanged.
        let line = say(&kit, stage, "06_5")?;
        line.wait_for(stage, "Side by side")?;
        let grown = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let chart_alpha = stage.add_value_tracker(1.0);
        let true_cm = coast.length;
        live_bar(
            stage,
            ROW_Y[0],
            palette::length(),
            move |_| true_cm,
            grown[0],
            chart_alpha,
        )?;
        live_bar(
            stage,
            ROW_Y[1],
            palette::touch(),
            live_bet(&coast, place),
            grown[1],
            chart_alpha,
        )?;
        let gap_alpha = stage.add_value_tracker(0.0);
        live_gap(stage, true_cm, live_bet(&coast, place), gap_alpha)?;
        let shrink = CHART_SIZE / SEALED_SIZE;
        let mut quiet = vec![tween(lines_alpha, 0.12, FADE_S)?, tween(dots_alpha, 0.12, FADE_S)?];
        for &m in &pieces {
            quiet.push(dim_line(m, 0.12, FADE_S)?);
        }
        together(stage, quiet)?;
        at_once(
            stage,
            &[],
            &[body, flap, rows[1]],
            vec![
                carry(sealed, shrink, v(NAME_RIGHT, ROW_Y[0]), RIGHT, GROW_S)?,
                carry(length, shrink, v(VALUE_X, ROW_Y[0]), LEFT, GROW_S)?,
            ],
            FADE_S,
        )?;
        let seven_cm = kit.text(
            stage,
            &words.bets[0],
            CHART_SIZE,
            palette::touch(),
            v(VALUE_X, ROW_Y[1]),
        )?;
        stage.move_to(seven_cm, v(VALUE_X, ROW_Y[1]), LEFT);
        at_once(
            stage,
            &[seven_cm],
            &[],
            vec![carry(tag, 1.0, v(NAME_RIGHT, ROW_Y[1]), RIGHT, GROW_S)?],
            GROW_S,
        )?;
        let mut restore = vec![
            tween(lines_alpha, 1.0, FADE_S)?,
            tween(dots_alpha, 1.0, FADE_S)?,
            glide(grown[0], 1.0, GROW_S, smooth)?,
            glide(grown[1], 1.0, GROW_S, smooth)?,
        ];
        for &m in &pieces {
            restore.push(dim_line(m, 1.0, FADE_S)?);
        }
        together(stage, restore)?;
        line.wait_for(stage, "three point eight centimetres short")?;
        let short = kit.text(stage, &words.short, CHART_SIZE, Palette::ink(), v(DIFF_X, DIFF_Y))?;
        stage.move_to(short, v(DIFF_X, DIFF_Y), LEFT);
        at_once(stage, &[short], &[], vec![tween(gap_alpha, 1.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "about fifteen percent")?;
        let percent = kit.text(stage, &words.percent, CHART_SIZE, Palette::ink(), v(DIFF_X, DIFF_Y))?;
        stage.next_to(percent, short, RIGHT, DIFF_GAP, ORIGIN);
        fade_in_all(stage, &[percent], FADE_S)?;

        // 06_6: one count at one placement. The estimate and the bet's
        // value and difference go; the sheet slides up into the run of
        // nine, the count and the gold bar following it; then the new
        // estimate, value and difference
        let line = say(&kit, stage, "06_6")?;
        line.wait_for(stage, "But seven")?;
        at_once(
            stage,
            &[],
            &[seven, seven_cm, short, percent],
            vec![tween(gap_alpha, 0.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "Slide the sheet up")?;
        place.make(stage, &plan.up, &[(1.0, SLIDE_S)], smooth)?;
        line.wait_for(stage, "Nine times pi")?;
        let nine = kit.tex(stage, &words.estimates[1], COUNT_SIZE, Palette::ink(), &[], word_at)?;
        stage.next_to(nine, probe, RIGHT, ARROW_GAP, ORIGIN);
        let nine_cm = kit.text(
            stage,
            &words.bets[1],
            CHART_SIZE,
            palette::touch(),
            v(VALUE_X, ROW_Y[1]),
        )?;
        stage.move_to(nine_cm, v(VALUE_X, ROW_Y[1]), LEFT);
        fade_in_all(stage, &[nine, nine_cm], FADE_S)?;
        line.wait_for(stage, "two point five centimetres too long")?;
        let long = kit.text(stage, &words.long, CHART_SIZE, Palette::ink(), v(DIFF_X, DIFF_Y))?;
        stage.move_to(long, v(DIFF_X, DIFF_Y), LEFT);
        at_once(stage, &[long], &[], vec![tween(gap_alpha, 1.0, FADE_S)?], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;

        // 06_7: the same coast and bet, back at the first placement: short
        // again
        let line = say(&kit, stage, "06_7")?;
        line.wait_for(stage, "Same coast")?;
        at_once(
            stage,
            &[],
            &[nine, nine_cm, long],
            vec![tween(gap_alpha, 0.0, FADE_S)?],
            FADE_S,
        )?;
        place.make(stage, &plan.down, &[(1.0, SLIDE_S)], smooth)?;
        line.wait_for(stage, "short or long")?;
        let again = kit.text(
            stage,
            &words.bets[0],
            CHART_SIZE,
            palette::touch(),
            v(VALUE_X, ROW_Y[1]),
        )?;
        stage.move_to(again, v(VALUE_X, ROW_Y[1]), LEFT);
        let short = kit.text(stage, &words.short, CHART_SIZE, Palette::ink(), v(DIFF_X, DIFF_Y))?;
        stage.move_to(short, v(DIFF_X, DIFF_Y), LEFT);
        at_once(
            stage,
            &[again, short],
            &[],
            vec![tween(gap_alpha, 1.0, FADE_S)?],
            FADE_S,
        )?;
        let up = now(stage);
        stand(&kit, stage, up)?;

        // 06_8: never one count. The chart goes; the count becomes the
        // average's question; then what the bet must mean
        let line = say(&kit, stage, "06_8")?;
        line.wait_for(stage, "never one count")?;
        at_once(
            stage,
            &[],
            &[sealed, length, tag, again, short],
            vec![tween(chart_alpha, 0.0, FADE_S)?, tween(gap_alpha, 0.0, FADE_S)?],
            FADE_S,
        )?;
        line.wait_for(stage, "the average count")?;
        let question = kit.text_t2c(
            stage,
            QUESTION,
            COUNT_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            QUESTION_AT,
        )?;
        at_once(stage, &[question], &[], vec![tween(count_alpha, 0.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "whether pi times")?;
        let equation = kit.tex(
            stage,
            &words.equation,
            EQUATION_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            EQUATION_AT,
        )?;
        fade_in_all(stage, &[equation], FADE_S)?;

        // 06_9: not yet for the coast; something simpler first. The card
        // and coast step back, the sheet stays
        let line = say(&kit, stage, "06_9")?;
        line.wait_for(stage, "So try the bet first")?;
        let mut dim = vec![fade_to(card, DIMMED, OPEN_S)?, tween(dots_alpha, 0.0, OPEN_S)?];
        for &m in &pieces {
            dim.push(dim_line(m, DIMMED, OPEN_S)?);
        }
        at_once(stage, &[], &[question, equation], dim, OPEN_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::sheet::TOP_Y;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Seg};

    fn shown() -> (Curve, Plan) {
        let coast = model::coast();
        let plan = plan(&coast, NINE_BY);
        (coast, plan)
    }

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The coast at every 0.02 cm of its length, straight runs included.
    fn along(coast: &Curve) -> Vec<P2> {
        let n = (coast.length / 0.02).ceil() as usize;
        (0..=n).map(|i| coast.at(coast.length * i as f64 / n as f64)).collect()
    }

    /// A text `size` points tall is taken as at most size/75 frame units
    /// high, size/150 a character wide (30 points: 0.4 high, "crossings"
    /// 1.8 wide), as the other chapters' layout tests take it.
    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    fn frame_y(card_y: f64) -> f64 {
        CARD_CENTRE[1] + CARD_UNIT * card_y
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let (coast, plan) = shown();
        let w = words(&coast, &plan).unwrap();
        assert_eq!(w.rows[0], r"2\text{ crossings} \leftrightarrow 6.28\text{ cm}");
        assert_eq!(
            w.rows[1],
            r"1\text{ crossing} \leftrightarrow \pi\text{ cm} \approx 3.14\text{ cm}"
        );
        assert_eq!(w.rim, "6.28 cm");
        assert_eq!(
            w.estimates,
            [
                r"\rightarrow\ 7 \times \pi\text{ cm} \approx 22.0\text{ cm}",
                r"\rightarrow\ 9 \times \pi\text{ cm} \approx 28.3\text{ cm}",
            ]
        );
        assert_eq!(w.length, "25.8 cm");
        assert_eq!(w.bets, ["22.0 cm", "28.3 cm"]);
        assert_eq!(w.short, "3.8 cm short");
        assert_eq!(w.percent, "about 15%");
        assert_eq!(w.long, "2.5 cm too long");
        assert_eq!(w.equation, r"\pi \times \text{average count} = 25.8\text{ cm}\ ?");
    }

    /// The first placement shows seven and the slid one nine; the slide up
    /// passes 7 then 9 only, with no brief count between, and back 9 then
    /// 7; 06_6's slide lies inside 04's run of nine, which 04's whole slide
    /// reads as 7, 9, 8, 7.
    #[test]
    fn the_slide_goes_from_seven_to_nine_and_back() {
        let (coast, plan) = shown();
        assert_eq!(facts::fixed(plan.first.slide, 2), "0.45");
        assert_eq!(facts::fixed(plan.up.end().slide, 2), "0.95");
        assert_eq!(count_at(&coast, plan.first), 7);
        assert_eq!(count_at(&coast, plan.up.end()), 9);
        assert!((plan.down.end().slide - plan.first.slide).abs() < 1e-12);
        let read =
            |mv: &Move, steps| -> Vec<usize> { placement::runs(&coast, mv, steps).iter().map(|r| r.count).collect() };
        assert_eq!(read(&plan.up, RUN_STEPS), [7, 9]);
        assert_eq!(read(&plan.down, RUN_STEPS), [9, 7]);
        // twice as fine, the same
        assert_eq!(read(&plan.up, 2 * RUN_STEPS), [7, 9]);
        let whole = Move::slide(plan.first, 1.5);
        let runs = placement::runs(&coast, &whole, 15000);
        assert_eq!(runs.iter().map(|r| r.count).collect::<Vec<_>>(), [7, 9, 8, 7]);
        let share = NINE_BY / 1.5;
        assert!(runs[1].from + 0.02 < share && share < runs[1].to - 0.02, "{runs:?}");
    }

    /// At the slid placement every line keeps 0.07 cm from every turning
    /// height, and 0.1 cm from every tangent point (an arc's highest or
    /// lowest point, where two crossings are born together), so no crossing
    /// shown sits on a near miss; its nine dots lie at least 0.5 cm apart.
    #[test]
    fn the_nine_placement_has_no_near_misses() {
        let (coast, plan) = shown();
        let s = plan.up.end().slide;
        assert!(
            sheet::clearance(&coast, 0.0, s) >= 0.07,
            "{}",
            sheet::clearance(&coast, 0.0, s)
        );
        let off = |h: f64| {
            let t = (h - s) / GAP;
            (t - t.round()).abs() * GAP
        };
        let mut tangents = 0;
        for part in &coast.parts {
            if let Seg::Arc {
                centre,
                radius,
                from,
                to,
            } = part.seg
            {
                let (lo, span) = (from.min(to), (to - from).abs());
                for (angle, h) in [(PI / 2.0, centre[1] + radius), (1.5 * PI, centre[1] - radius)] {
                    let u = (angle - lo).rem_euclid(TAU);
                    if u > 0.0 && u < span {
                        tangents += 1;
                        assert!(off(h) >= 0.1, "a tangent point at height {h} is {} from a line", off(h));
                    }
                }
            }
        }
        assert!(tangents > 0);
        let dots: Vec<P2> = sheet::crossings(&coast, 0.0, s).iter().map(|c| c.at).collect();
        for (i, a) in dots.iter().enumerate() {
            for b in &dots[i + 1..] {
                assert!(dist(*a, *b) >= 0.5, "{a:?} {b:?}");
            }
        }
    }

    /// Slides the voice does not describe are refused, each by its own
    /// check: 0.3 cm up stays at seven; 1.1 cm down ends at nine but passes
    /// nine, seven and eight on the way; 0.6 cm up passes only seven and
    /// nine but is not the slide the facts print.
    #[test]
    fn a_slide_the_voice_does_not_describe_is_refused() {
        let coast = model::coast();
        for (by, why) in [
            (0.3, "bet.nine_crossings"),
            (-1.1, "the slides pass"),
            (0.6, "the sheet slides to"),
        ] {
            let p = plan(&coast, by);
            let err = words(&coast, &p).err().unwrap_or_default();
            assert!(err.contains(why), "{by}: {err}");
        }
    }

    /// The chart is to scale: every bar is CHART_K frame units a
    /// centimetre from the same start, so the gap between the bet's end and
    /// the true length's is the difference itself, shown as the facts
    /// round it; short at seven, long at nine.
    #[test]
    fn the_chart_is_to_scale() {
        let (coast, _) = shown();
        let l = coast.length;
        let (seven, nine) = (7.0 * rate(), 9.0 * rate());
        assert!(seven < l && l < nine);
        let [a, b] = gap_ends(l, seven);
        assert!(a < b);
        assert!(((b - a) / CHART_K - (l - seven)).abs() < 1e-12);
        assert_eq!(facts::fixed((b - a) / CHART_K, 1), "3.8");
        let [a, b] = gap_ends(l, nine);
        assert!(a > b);
        assert_eq!(facts::fixed((a - b) / CHART_K, 1), "2.5");
        for cm in [l, seven, nine] {
            assert!(((bar_end(cm) - CHART_X) / CHART_K - cm).abs() < 1e-12);
        }
    }

    /// The true length stays sealed until the envelope opens: nothing on
    /// screen before 06_4 gives it away.
    #[test]
    fn the_true_length_is_sealed_until_the_envelope_opens() {
        let f = facts::facts().unwrap();
        let (coast, plan) = shown();
        let w = words(&coast, &plan).unwrap();
        let before = [
            w.rows[0].clone(),
            w.rows[1].clone(),
            w.rim.clone(),
            w.estimates[0].clone(),
            ARROW.into(),
            TAG.into(),
            CROSSINGS.into(),
            placement::SEALED.into(),
        ];
        for key in ["coast.length", "coast.length_2dp", "coast.length_3dp"] {
            let length = f.get(key).unwrap();
            assert!(before.iter().all(|t| !t.contains(length)), "{length}");
        }
        assert!(w.length.contains(f.get("coast.length").unwrap()));
    }

    /// 05's coin rebuilt where its roll ended: on the card, touching the
    /// lines over and under it at a single point each, so its count is two;
    /// its trail one rim long; the rim's label clear of the coin.
    #[test]
    fn the_coin_rests_where_05_left_it() {
        let (_, plan) = shown();
        let s = plan.first.slide;
        let c = coin_end(s);
        let m = meetings(c, RADIUS, s);
        assert_eq!(m.len(), 2);
        assert!(m.iter().all(|x| x.touch));
        assert_eq!(facts::fixed(c[0] - ROLL_X, 2), "6.28");
        assert!(c[0] + RADIUS < CARD_W / 2.0 - sheet::LINE_INSET && c[1] + RADIUS < CARD_H / 2.0);
        let label = [(ROLL_X + c[0]) / 2.0, c[1] - RADIUS + ROLL_LABEL_UP];
        assert!(label[0] + 0.8 < c[0] - RADIUS);
    }

    /// The chart's rows clear the coast's lowest point and the sheet's
    /// lowest line at every moment of the slide up and back; no line passes
    /// through the chart; the names, values and differences keep to the
    /// card and to their columns; the flap opens under the top line; the
    /// revealed length clears the count; the equation and the bet's row sit
    /// in the band under the coast.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let (coast, plan) = shown();
        let pts = along(&coast);
        let lowest = pts.iter().map(|&p| card_point(p)[1]).fold(f64::INFINITY, f64::min);
        let hh = half_height(CHART_SIZE);
        let top = ROW_Y[0] + hh;
        let bottom = (ROW_Y[1] - BRACKET_DROP).min(ROW_Y[1] - hh);
        assert!(top < lowest - 0.1, "{top} under {lowest}");
        for i in 0..=100 {
            let u = f64::from(i) / 100.0;
            for (_, [a, _]) in placement::lines_at(plan.up.at(u)) {
                let y = card_point(a)[1];
                assert!(
                    y < bottom - 0.1 || y > top + 0.1,
                    "a line at {y} meets the chart at {u}"
                );
            }
        }
        // the columns, frame x
        let card_left = CARD_CENTRE[0] - CARD_W * CARD_UNIT / 2.0;
        let card_right = CARD_CENTRE[0] + CARD_W * CARD_UNIT / 2.0;
        assert!(NAME_RIGHT - width(placement::SEALED, CHART_SIZE) > card_left + 0.2);
        assert!(bar_end(9.0 * rate()) + 0.15 < VALUE_X);
        for value in ["25.8 cm", "22.0 cm", "28.3 cm"] {
            assert!(VALUE_X + width(value, CHART_SIZE) < DIFF_X - 0.2);
        }
        let short = width("3.8 cm short", CHART_SIZE) + DIFF_GAP + width("about 15%", CHART_SIZE);
        assert!(DIFF_X + short.max(width("2.5 cm too long", CHART_SIZE)) < card_right - 0.3);
        // the flap opens under the line over the envelope, at the first
        // placement (the sheet slides only after the envelope has gone)
        let top_line = frame_y(plan.first.slide + 2.0 * GAP);
        assert!(flap_points(1.0)[1][1] < top_line - 0.1);
        assert_eq!(flap_points(0.0)[1][1], ENVELOPE_AT[1] - 0.05);
        // the revealed length, beside "true length", clears the count's
        // digits (two at most, 30 points)
        let revealed =
            ENVELOPE_AT[0] + width(placement::SEALED, SEALED_SIZE) / 2.0 + REVEAL_GAP + width("25.8 cm", SEALED_SIZE);
        assert!(revealed < COUNT_RIGHT[0] - 2.0 * COUNT_SIZE / 150.0 - 0.2);
        // the bet's row, its tag and the equation, between the coast and
        // the first placement's lowest line
        let line = frame_y(plan.first.slide - 2.0 * GAP);
        for at in [ROWS_AT[1], EQUATION_AT] {
            assert!(at[1] < lowest - 0.35 && at[1] > line + 0.25, "{at:?}");
        }
        assert!(TOP_Y > frame_y(plan.up.end().slide + GAP) + half_height(COUNT_SIZE));
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("06_1", "Now a bet"),
            ("06_1", "every curve"),
            ("06_1", "a crossing on any curve"),
            ("06_2", "seven crossings"),
            ("06_2", "What length"),
            ("06_3", "Seven times pi"),
            ("06_4", "open the envelope"),
            ("06_4", "twenty-five point eight"),
            ("06_5", "Side by side"),
            ("06_5", "three point eight centimetres short"),
            ("06_5", "about fifteen percent"),
            ("06_6", "But seven"),
            ("06_6", "Slide the sheet up"),
            ("06_6", "Nine times pi"),
            ("06_6", "two point five centimetres too long"),
            ("06_7", "Same coast"),
            ("06_7", "short or long"),
            ("06_8", "never one count"),
            ("06_8", "the average count"),
            ("06_8", "whether pi times"),
            ("06_9", "So try the bet first"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
