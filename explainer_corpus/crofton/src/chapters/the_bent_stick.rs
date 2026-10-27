//! Chapter 13, the bent stick. From 12's end (the coin, every curve's
//! average its length over π, the coast's 25.81 ÷ π ≈ 8.22) our 6 cm stick
//! comes back across the lines: its length over π, about 1.91 crossings,
//! and by 08's shadow rule its average shadow 12 ÷ π ≈ 3.82 cm, 64% of it,
//! the 2 ÷ π 09 left the coin to explain (13_1). Turned to lie along the
//! lines it crosses none; bent at its middle into two 3 cm arms at a right
//! angle, its length still 6 cm, it crosses one: will its average go up,
//! down, or stay? (13_2, a prediction.) The same, about 1.91: each arm is a
//! 3 cm stick and crossings add; the sheet turns half a turn about the
//! corner's line, the count running 1, 2, 3, 2, 1, 2, 1 (13_3). What
//! changes is the spread: straight, it misses every line 10.7% of the
//! time; bent, never, one arm always reaching more than a gap across; the
//! two spreads side by side, each averaging 1.91 (13_4). Bent once more
//! into a 4 that crosses itself, flat on the card, a line through the
//! crossing counts both pieces there; still 1.91 (13_5). Buffon's noodle
//! (13_6). And the coast is a long bent wire: seven at one placement, its
//! length over π on average, so π times the average count gives back its
//! length, 25.81 cm (13_7). The wire, its dots and its count are drawn
//! every frame from 12's [`Wire`] through this file's shape; the spreads
//! from the model's count shares. Starts where chapter 12 ends. Not
//! registered yet: root wires it in (`docs/candidate-12-13.md`).

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::{linear, smooth};
use fmn::prelude::*;

use super::a_crowd_of_sticks::{LEFT_X, LOW_ROW, TOP_ROW};
use super::draw::{card_point, fade_in_all, fail, flat, line_style, now, say, stand, think, together};
use super::placement::{self, Placement, Pose, at_once, glide};
use super::sheet::{self, THINK_AT};
use super::straight_sticks::{ACROSS, ALONG, HOME, LENGTH};
use super::the_shadow_rule::{ROW_SIZE, left_at, row};
use super::why_every_curve::{
    self, COIN_AT, Wire, bright_shore, closing_rows, coin_at_rest, count_at, counter, live_wire, live_wire_dots,
    placed, wire_count,
};
use crate::facts;
use crate::model::{self, Curve, GAP, P2};
use crate::palette;

pub struct TheBentStick {
    pub kit: Kit,
}

// ---------------------------------------------------------------- the wire

/// The 6 cm wire in its own frame, its middle at the origin, lying along
/// +x (turned to lie along the lines, +x runs left on the card): one 3 cm
/// arm on −x, the other as two 1.5 cm pieces on +x. Up to `u` = 1 the
/// second arm swings about the middle by a quarter turn (clockwise in its
/// own frame, so on the card it rises through the open upper left); from 1
/// to 2 it swings an eighth more while its outer piece turns three eighths
/// at its joint, into a 4 whose last piece crosses the first arm. Every
/// piece keeps its length at every `u`.
pub fn shape(u: f64) -> Vec<[P2; 2]> {
    let (bend, more) = (u.clamp(0.0, 1.0), (u - 1.0).clamp(0.0, 1.0));
    let first = -FRAC_PI_2 * bend - FRAC_PI_4 * more;
    let second = first - FOUR_TURN * more;
    let piece = LENGTH / 4.0;
    let joint = [piece * first.cos(), piece * first.sin()];
    let end = [joint[0] + piece * second.cos(), joint[1] + piece * second.sin()];
    vec![[[-LENGTH / 2.0, 0.0], [0.0, 0.0]], [[0.0, 0.0], joint], [joint, end]]
}

/// How far the outer piece turns at its joint to make the 4, radians.
pub const FOUR_TURN: f64 = 3.0 * FRAC_PI_4;

/// The morphs shown: straight, bent at a right angle, the 4.
pub const STRAIGHT: f64 = 0.0;
pub const BENT: f64 = 1.0;
pub const FOUR: f64 = 2.0;

/// The 4's crossing point on the card: its last piece meets its first arm.
pub fn four_meet() -> P2 {
    let w = placed(shape, HOME, ALONG, FOUR);
    let p = w.parts[2].seg.point(0.0);
    [p[0], HOME[1]]
}

/// The sheet at 13_5: turned an eighth about the 4's crossing, then slid
/// so a line runs through it.
pub fn four_move(first: Pose) -> placement::Move {
    let at = four_meet();
    let mut mv = placement::Move::turn(first, FRAC_PI_4, at);
    let turned = mv.end();
    let n = [-turned.beta.sin(), turned.beta.cos()];
    mv.by = (at[0] * n[0] + at[1] * n[1] - turned.slide).rem_euclid(GAP);
    mv
}

/// How many directions the spreads are found over (a midpoint rule over
/// direction, exact over the slide; the oracle's are in closed form).
pub const SHARE_DIRECTIONS: usize = 3600;

/// Each count's chance for the wire at `u` lying along at home, by count.
pub fn shares(u: f64) -> Vec<f64> {
    placed(shape, HOME, ALONG, u).count_shares(SHARE_DIRECTIONS, Some(GAP))
}

/// The straight wire lying across, as 07 and 12 show it.
fn across() -> Curve {
    placed(shape, HOME, ACROSS, STRAIGHT)
}

// ------------------------------------------------------------------ words

/// The chapter's words that carry numbers, from the facts. Refused unless
/// the model gives what the screen and the voice say: the stick's mean,
/// its mean shadow and share, as 08 and 09 found them, from 6 ÷ π; its
/// counts lying across, along and bent; the bent wire's length unchanged
/// and its mean the straight one's; the sweep's counts; each spread, its
/// counts and mean (none with a zero count when bent); the 4's mean and
/// its crossing counted for both pieces; and the coast's average times π
/// giving its length.
pub struct Words {
    pub mean: String,
    pub reach: String,
    pub share: String,
    /// 13_2: each arm's length beside it, and the two together.
    pub arm: String,
    pub arms: String,
    pub adds: String,
    /// 13_4: each spread's name, its counts and percents, its mean.
    pub names: [String; 2],
    pub counts: [Vec<String>; 2],
    pub percents: [Vec<String>; 2],
    pub heights: [Vec<f64>; 2],
    pub average: String,
    pub four: String,
    pub exact: String,
    pub source: String,
    pub coast: String,
}

const NAMES: [&str; 2] = ["straight", "bent"];
const SOURCE_LABEL: &str = "source:";
/// Ramaley's "Buffon's noodle problem", Amer. Math. Monthly 76 (1969)
/// (`brief.md`).
pub const NOODLE_YEAR: u32 = 1969;
const EXACT: &str = "the wire lies flat on the card and is counted piece by piece, so a line through its \
                     crossing counts twice. A wire bent out of the card is another question";

pub fn words() -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let fixed = facts::fixed;
    let agree = |key: &str, got: String| -> Result<String, String> {
        let want = get(key)?;
        if want == got {
            Ok(want)
        } else {
            Err(format!("{key}: the model gives {got}, the facts {want}"))
        }
    };
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let coast = model::coast();
    let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));

    // 13_1: 6 ÷ π, its shadow 2 × 6 ÷ π = 12 ÷ π, its share 2 ÷ π
    let stick = agree("stick.length", fixed(LENGTH, 0))?;
    let gap = agree("sheet.gap", fixed(GAP, 0))?;
    let twelve = fixed(GAP * LENGTH, 0);
    let mean = agree("stick.turned_mean", fixed(LENGTH / PI, 2))?;
    let reach = agree("stick.mean_reach", fixed(GAP * LENGTH / PI, 2))?;
    let share = agree("stick.mean_reach_share", fixed(2.0 / PI, 2))?;
    agree("stick.mean_reach_percent", fixed(200.0 / PI, 0))?;
    if fixed(number(&reach)? / LENGTH, 2) != share || (model::worth(GAP) - PI).abs() > 1e-12 {
        return Err(format!("13_1 shows {reach} ÷ {stick} ≈ {share} ≈ 2 ÷ π"));
    }

    // 13_2 and 13_3: across, three; along, none; bent, one; still 6 cm;
    // the sweep about the corner's line; the arms' means add
    agree("stick.home_across", count_at(&across(), first).to_string())?;
    let along = placed(shape, HOME, ALONG, STRAIGHT);
    agree("stick.home_along", count_at(&along, first).to_string())?;
    let bent = placed(shape, HOME, ALONG, BENT);
    agree("hook.home_count", count_at(&bent, first).to_string())?;
    if (bent.length - LENGTH).abs() > 1e-12 {
        let got = bent.length;
        return Err(format!("13_2 says the bent wire is still {stick} cm, not {got:.4}"));
    }
    let mut seen: Vec<usize> = Vec::new();
    for mv in sweep(first) {
        for i in 0..=3600 {
            let n = count_at(&bent, mv.at(f64::from(i) / 3600.0));
            if seen.last() != Some(&n) {
                seen.push(n);
            }
        }
    }
    let seen: Vec<String> = seen.iter().map(usize::to_string).collect();
    agree("hook.sweep", seen.join(","))?;
    let hook = agree("hook.mean", fixed(model::moments(&shares(BENT)).0, 2))?;
    if hook != mean || fixed(2.0 * (LENGTH / 2.0) / PI, 2) != hook {
        return Err(format!("13_3 shows 3 ÷ π + 3 ÷ π = 6 ÷ π ≈ {hook}"));
    }

    // 13_4: each spread, by count, to a tenth of a percent; straight it
    // misses 10.7% of the time, bent never
    let mut counts = [Vec::new(), Vec::new()];
    let mut percents = [Vec::new(), Vec::new()];
    let mut heights = [Vec::new(), Vec::new()];
    for (k, (u, key)) in [(STRAIGHT, "stick"), (BENT, "hook")].into_iter().enumerate() {
        let sh = shares(u);
        let low = sh.iter().position(|&p| p > 1e-9).unwrap_or(0);
        let shown: Vec<String> = sh[low..].iter().map(|&p| fixed(p * 100.0, 1)).collect();
        agree(&format!("{key}.shares_percent"), shown.join(","))?;
        let names: Vec<String> = (low..sh.len()).map(|c| c.to_string()).collect();
        let all = get(if k == 0 { "stick.all_counts" } else { "hook.counts" })?;
        if names.join(",") != all {
            return Err(format!("13_4 shows the counts {names:?}, the facts {all}"));
        }
        if fixed(model::moments(&sh).0, 2) != mean {
            return Err(format!("13_4 says the {key} averages {mean}"));
        }
        counts[k] = names;
        percents[k] = shown.iter().map(|p| format!("{p}%")).collect();
        heights[k] = sh[low..].to_vec();
    }
    if shares(BENT)[0] > 1e-12 || (LENGTH / 2.0) / 2f64.sqrt() <= GAP {
        return Err("13_4 says the bent wire never misses, one arm always reaching past a gap".into());
    }

    // 13_5: the 4, still 6 cm, crossing itself; its mean; the line through
    // its crossing counted for both pieces
    let four = placed(shape, HOME, ALONG, FOUR);
    if (four.length - LENGTH).abs() > 1e-12 {
        return Err("13_5 says the 4 is still 6 cm".into());
    }
    let four_mean = agree("four.mean", fixed(model::moments(&shares(FOUR)).0, 2))?;
    let at = four_move(first).end();
    let crossings = sheet::crossings(&four, at.beta, at.slide);
    agree("four.count", crossings.len().to_string())?;
    let meet = four_meet();
    let both = crossings
        .iter()
        .filter(|c| why_every_curve::dist(c.at, meet) < 1e-9)
        .count();
    agree("four.through", both.to_string())?;
    agree("noodle.year", NOODLE_YEAR.to_string())?;

    // 13_7: the coast's average times π, its length to two places
    agree("open.crossings", count_at(&coast, first).to_string())?;
    agree("crofton.mean_crossings", fixed(coast.length / PI, 2))?;
    let average = agree("crofton.mean_crossings_4dp", fixed(coast.length / PI, 4))?;
    let length = agree("coast.length_2dp", fixed(coast.length, 2))?;
    if fixed(number(&average)? * PI, 2) != length {
        return Err(format!("13_7 shows {average} × π ≈ {length}"));
    }

    let half = fixed(LENGTH / 2.0, 0);
    Ok(Words {
        mean: format!(r"{stick} \div \pi \approx {mean}"),
        reach: format!(r"{gap} \times {stick} \div \pi = {twelve} \div \pi \approx {reach}\text{{ cm}}"),
        share: format!(r"{reach} \div {stick} \approx {share} \approx 2 \div \pi"),
        arm: format!("{half} cm"),
        arms: format!(r"{half} + {half} = {stick}\text{{ cm}}"),
        adds: format!(r"{half} \div \pi + {half} \div \pi = {stick} \div \pi \approx {hook}"),
        names: NAMES.map(str::to_owned),
        counts,
        percents,
        heights,
        average: format!("average {mean}"),
        four: format!("average {four_mean}"),
        exact: EXACT.into(),
        source: format!("J. F. Ramaley, Buffon's noodle problem, Amer. Math. Monthly, {NOODLE_YEAR}"),
        coast: format!(r"{average} \times \pi \approx {length}\text{{ cm}}"),
    })
}

/// 13_3's half turn of the sheet about the point of the corner's line
/// under the corner, so its lines end where they began.
pub fn sweep(first: Pose) -> [placement::Move; 1] {
    let pin = placement::half_turn_pin(first, HOME[0]);
    [placement::Move::turn(first, PI, pin)]
}

// ------------------------------------------------------------------ layout

/// The arms' lengths beside them, card cm: over the arm along the lines,
/// left of the raised arm.
const ARM_LABELS: [P2; 2] = [[-3.0, 1.75], [-5.35, 2.7]];
const LABEL_SIZE: f64 = 22.0;
/// 13_4's spreads on the console: each name and mean at its left, its bars
/// a count apart, their percents over them and counts under them.
const NAME_X: [f64; 2] = [LEFT_X, 0.5];
const NAME_Y: f64 = -2.2;
const AVERAGE_Y: f64 = -2.65;
const BAR_X: [f64; 2] = [-3.4, 2.6];
const BAR_STEP: f64 = 0.8;
const BAR_W: f64 = 0.5;
const BAR_BASE: f64 = -2.95;
const BAR_K: f64 = 1.3;
const BAR_OPACITY: f64 = 0.85;
const PERCENT_GAP: f64 = 0.17;
const COUNT_Y: f64 = -3.17;
const SPREAD_SIZE: f64 = 18.0;
/// 13_5's ring round the 4's crossing, frame.
const RING_R: f64 = 0.17;

// ------------------------------------------------------------------ timing

const FADE_S: f64 = 0.4;
const TURN_S: f64 = 2.5;
const BEND_S: f64 = 3.0;
const SWEEP_S: f64 = 12.0;
const MOVE_S: f64 = 3.0;

// --------------------------------------------------------- what is drawn

/// One spread's bars, their percents and counts, and its name and mean.
/// Detached.
fn spread(kit: &Kit, stage: &mut Stage<'_>, words: &Words, k: usize) -> fmn::Result<Vec<Mob>> {
    let name_at = v(NAME_X[k], NAME_Y);
    let name = kit.text(stage, &words.names[k], LABEL_SIZE, palette::stick(), name_at)?;
    let average_at = v(NAME_X[k], AVERAGE_Y);
    let average = kit.text(stage, &words.average, SPREAD_SIZE, Palette::ink(), average_at)?;
    let mut out = vec![left_at(stage, name, name_at), left_at(stage, average, average_at)];
    for (i, &p) in words.heights[k].iter().enumerate() {
        let x = BAR_X[k] + BAR_STEP * i as f64;
        let h = (BAR_K * p).max(0.01);
        let bar = Rectangle::new()
            .width(BAR_W)
            .height(h)
            .style(flat(palette::stick(), BAR_OPACITY))
            .build()?
            .moved_to(v(x, BAR_BASE + h / 2.0));
        out.push(stage.arena_mut().add(bar));
        let top = v(x, BAR_BASE + h + PERCENT_GAP);
        out.push(kit.text(stage, &words.percents[k][i], SPREAD_SIZE, Palette::ink(), top)?);
        out.push(kit.text(stage, &words.counts[k][i], SPREAD_SIZE, Palette::dim(), v(x, COUNT_Y))?);
    }
    Ok(out)
}

/// A gold ring round a card point. Detached.
fn ring(stage: &mut Stage<'_>, at: P2) -> Mob {
    let r = Circle::new()
        .radius(RING_R)
        .arc_center(card_point(at))
        .style(line_style(palette::touch(), 3.0))
        .build();
    stage.arena_mut().add(r)
}

/// The sheet turned by `turn` about `pin` and slid `by` from where it
/// stands, `rt` seconds at `rate`, then named the controls' way.
fn move_sheet(
    stage: &mut Stage<'_>,
    place: Placement,
    (turn, by, pin): (f64, f64, P2),
    rt: f64,
    rate: fn(f64) -> f64,
) -> fmn::Result<()> {
    place.begin(stage, turn, by, pin)?;
    place.run(stage, &[(1.0, rt)], rate)?;
    place.settle(stage).map(|_| ())
}

impl SceneConstruct for TheBentStick {
    fn name(&self) -> &str {
        "13_the_bent_stick"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let earlier = why_every_curve::words().map_err(fail)?;
        let words = words().map_err(fail)?;

        // where 12 ended: the shore at full strength, 09's panels down; the
        // coin at rest and 12's two rows
        let shore = bright_shore(&kit, stage, &coast, first, false)?;
        let place = shore.place;
        let coin = coin_at_rest(stage, COIN_AT);
        let rows = closing_rows(&kit, stage, &earlier)?;
        for &m in coin.iter().chain(&rows) {
            stage.add_to_scene(m)?;
        }

        // 13_1: the stick across, three; 6 ÷ π; its shadow, 64%, 2 ÷ π
        let line = say(&kit, stage, "13_1")?;
        line.wait_for(stage, "Back to our six centimetre stick")?;
        let wire = Wire::new(stage, HOME, ACROSS, shape);
        live_wire(stage, wire)?;
        let dots = stage.add_value_tracker(0.0);
        live_wire_dots(stage, wire, place, dots)?;
        let counts = stage.add_value_tracker(0.0);
        counter(&kit, stage, wire_count(wire, place, None), counts)?;
        let mut gone = coin.clone();
        gone.extend(rows);
        at_once(stage, &[], &gone, vec![], FADE_S)?;
        together(
            stage,
            vec![
                tween(wire.alpha, 1.0, FADE_S)?,
                tween(dots, 1.0, FADE_S)?,
                tween(counts, 1.0, FADE_S)?,
            ],
        )?;
        line.wait_for(stage, "about one point nine one crossings")?;
        let mean = row(&kit, stage, &words.mean, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[mean], FADE_S)?;
        line.wait_for(stage, "twelve over pi")?;
        let reach = row(&kit, stage, &words.reach, Palette::ink(), LOW_ROW)?;
        fade_in_all(stage, &[reach], FADE_S)?;
        line.wait_for(stage, "sixty-four percent")?;
        let share = kit.tex(stage, &words.share, ROW_SIZE, palette::length(), &[], LOW_ROW)?;
        stage.next_to(share, reach, RIGHT, 0.5, ORIGIN);
        fade_in_all(stage, &[share], FADE_S)?;

        // 13_2: turned to lie along, none; bent at its middle, one; the
        // arms' lengths; will its average change?
        let line = say(&kit, stage, "13_2")?;
        line.wait_for(stage, "Turn it to lie along the lines")?;
        let turn = wire.moves(HOME, ALONG, TURN_S)?;
        at_once(stage, &[], &[mean, reach, share], turn, FADE_S)?;
        line.wait_for(stage, "Now bend the same wire")?;
        stage.play(glide(wire.morph, BENT, BEND_S, smooth)?)?;
        line.wait_for(stage, "Its length hasn't changed")?;
        let mut labels = Vec::with_capacity(2);
        for at in ARM_LABELS {
            labels.push(kit.text(stage, &words.arm, LABEL_SIZE, palette::stick(), card_point(at))?);
        }
        let arms = row(&kit, stage, &words.arms, Palette::ink(), TOP_ROW)?;
        fade_in_all(stage, &[labels[0], labels[1], arms], FADE_S)?;
        line.wait_for(stage, "or stay the same")?;
        let q = think(&kit, stage, THINK_AT)?;

        // 13_3: the same; the arms' averages add; half a turn of the sheet
        // about the corner's line, the card's labels gone first so no line
        // sweeps through them
        let line = say(&kit, stage, "13_3")?;
        line.wait_for(stage, "It stays the same")?;
        let adds = row(&kit, stage, &words.adds, Palette::ink(), LOW_ROW)?;
        at_once(stage, &[adds], &[q], vec![], FADE_S)?;
        line.wait_for(stage, "Turn the sheet, and its count moves around")?;
        at_once(stage, &[], &labels, vec![], FADE_S)?;
        let [half_turn] = sweep(first);
        move_sheet(stage, place, (half_turn.turn, 0.0, half_turn.pin), SWEEP_S, linear)?;

        // 13_4: the spreads, straight and bent; the bent wire never misses
        let line = say(&kit, stage, "13_4")?;
        line.wait_for(stage, "What changes is how the count is spread")?;
        let straight = spread(&kit, stage, &words, 0)?;
        at_once(stage, &straight, &[arms, adds], vec![], FADE_S)?;
        line.wait_for(stage, "Bent, it never does")?;
        let bent = spread(&kit, stage, &words, 1)?;
        fade_in_all(stage, &bent, FADE_S)?;

        // 13_5: bent into the 4; the sheet turned and slid so a line runs
        // through its crossing; still 1.91
        let line = say(&kit, stage, "13_5")?;
        line.wait_for(stage, "Bend it once more")?;
        let mut gone = straight.clone();
        gone.extend(bent);
        at_once(stage, &[], &gone, vec![], FADE_S)?;
        stage.play(glide(wire.morph, FOUR, BEND_S, smooth)?)?;
        line.wait_for(stage, "Where its pieces cross")?;
        let mark = ring(stage, four_meet());
        fade_in_all(stage, &[mark], FADE_S)?;
        let mv = four_move(first);
        move_sheet(stage, place, (mv.turn, mv.by, mv.pin), MOVE_S, smooth)?;
        line.wait_for(stage, "counts both of them")?;
        let exact = precisely(&kit, stage, &words.exact)?;
        fade_in_all(stage, &[exact], FADE_S)?;
        line.wait_for(stage, "still one point nine one")?;
        let four = kit.text(stage, &words.four, LABEL_SIZE, Palette::ink(), TOP_ROW)?;
        let four = left_at(stage, four, TOP_ROW);
        fade_in_all(stage, &[four], FADE_S)?;

        // 13_6: Buffon's noodle
        let line = say(&kit, stage, "13_6")?;
        line.wait_for(stage, "Buffon's noodle")?;
        let source = lower_third(&kit, stage, SOURCE_LABEL, &words.source)?;
        at_once(stage, &[source], &[exact, mark], vec![], FADE_S)?;

        // 13_7: the coast, a long bent wire: seven here; its length over π
        // on average; π times that, its length
        let line = say(&kit, stage, "13_7")?;
        line.wait_for(stage, "our coast is just a long bent wire")?;
        at_once(
            stage,
            &[],
            &[source, four],
            vec![
                tween(wire.alpha, 0.0, FADE_S)?,
                tween(dots, 0.0, FADE_S)?,
                tween(counts, 0.0, FADE_S)?,
            ],
            FADE_S,
        )?;
        move_sheet(stage, place, (-mv.turn, -mv.by, mv.pin), MOVE_S, smooth)?;
        line.wait_for(stage, "seven times")?;
        together(
            stage,
            vec![
                tween(shore.dots_alpha, 1.0, FADE_S)?,
                tween(shore.count_alpha, 1.0, FADE_S)?,
            ],
        )?;
        line.wait_for(stage, "gives back its length")?;
        let back = row(&kit, stage, &words.coast, palette::length(), LOW_ROW)?;
        fade_in_all(stage, &[back], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::the_shadow_rule::CONSOLE;
    use super::*;
    use crate::model::{CARD_H, CARD_W, Part, Seg};

    fn first() -> Pose {
        Pose::new(0.0, sheet::shown_slide(&model::coast(), 0.0))
    }

    fn half_height(size: f64) -> f64 {
        size / 150.0
    }

    fn width(text: &str, size: f64) -> f64 {
        text.chars().count() as f64 * size / 150.0
    }

    /// A TeX row's shown characters, roughly: its commands dropped.
    fn shown(tex: &str) -> String {
        tex.replace(r"\text", "")
            .replace(r"\div", "÷")
            .replace(r"\times", "×")
            .replace(r"\approx", "≈")
            .replace(r"\pi", "π")
            .replace(['{', '}', '\\'], "")
    }

    fn bounds((at, w, h): (Vec3, f64, f64)) -> [f64; 4] {
        [at[0] - w / 2.0, at[0] + w / 2.0, at[1] - h / 2.0, at[1] + h / 2.0]
    }

    fn line(a: P2, b: P2) -> Curve {
        Curve {
            parts: vec![Part {
                seg: Seg::Line { from: a, to: b },
                start: 0.0,
            }],
            length: why_every_curve::dist(a, b),
        }
    }

    /// Straight segments end to end, as one curve.
    fn chain(points: &[P2]) -> Curve {
        let mut parts = Vec::new();
        let mut start = 0.0;
        for w in points.windows(2) {
            let seg = Seg::Line { from: w[0], to: w[1] };
            parts.push(Part { seg, start });
            start += seg.len();
        }
        Curve { parts, length: start }
    }

    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let w = words().unwrap();
        assert_eq!(w.mean, r"6 \div \pi \approx 1.91");
        assert_eq!(w.reach, r"2 \times 6 \div \pi = 12 \div \pi \approx 3.82\text{ cm}");
        assert_eq!(w.share, r"3.82 \div 6 \approx 0.64 \approx 2 \div \pi");
        assert_eq!(w.arms, r"3 + 3 = 6\text{ cm}");
        assert_eq!(w.adds, r"3 \div \pi + 3 \div \pi = 6 \div \pi \approx 1.91");
        assert_eq!(w.percents[0], ["10.7%", "22.9%", "31.2%", "35.3%"]);
        assert_eq!(w.percents[1], ["23.5%", "62.1%", "14.3%", "0.1%"]);
        assert_eq!(w.counts, [["0", "1", "2", "3"], ["1", "2", "3", "4"]]);
        assert_eq!(w.four, "average 1.91");
        assert_eq!(w.coast, r"8.2165 \times \pi \approx 25.81\text{ cm}");
    }

    /// Every bend keeps the wire 6 cm and each piece its length; a bend
    /// that lost a millimetre would average 1.88, not 1.91 (a negative
    /// control on length-changing bends).
    #[test]
    fn bending_keeps_the_length() {
        for i in 0..=200 {
            let u = FOUR * f64::from(i) / 200.0;
            let w = placed(shape, HOME, ALONG, u);
            assert!((w.length - LENGTH).abs() < 1e-12, "{u}");
            for (part, want) in w.parts.iter().zip([3.0, 1.5, 1.5]) {
                assert!((part.seg.len() - want).abs() < 1e-12, "{u}");
            }
            for pair in w.parts.windows(2) {
                assert!(why_every_curve::dist(pair[1].seg.point(0.0), pair[0].seg.point(1.0)) < 1e-12);
            }
        }
        let short = chain(&[[-2.9, 0.0], [0.0, 0.0], [0.0, 3.0]]);
        let m = model::moments(&short.count_shares(SHARE_DIRECTIONS, Some(GAP))).0;
        assert_eq!(facts::fixed(m, 2), "1.88");
    }

    /// Straight, bent and the 4 all average 6 ÷ π over 3,600 directions to
    /// within a millionth; their spreads differ; the bent wire never shows
    /// zero, its arms' shadows never both under a gap.
    #[test]
    fn the_bends_keep_the_average() {
        for u in [STRAIGHT, BENT, FOUR] {
            let sh = shares(u);
            assert!((sh.iter().sum::<f64>() - 1.0).abs() < 1e-9);
            assert!((model::moments(&sh).0 - LENGTH / PI).abs() < 1e-6, "{u}");
        }
        assert!(shares(BENT)[0] < 1e-12 && shares(STRAIGHT)[0] > 0.1);
        for i in 0..1800 {
            let phi = PI * (f64::from(i) + 0.5) / 1800.0;
            assert!(3.0 * phi.sin().abs().max(phi.cos().abs()) > GAP);
        }
        // the straight stick lying across counts as the one along does,
        // on average: where it lies does not matter
        let sh = across().count_shares(SHARE_DIRECTIONS, Some(GAP));
        assert!((model::moments(&sh).0 - LENGTH / PI).abs() < 1e-6);
    }

    /// A finite set of directions estimates, never gives exactly: over 36
    /// directions the bent wire's spread is off by more than a
    /// ten-thousandth somewhere, and four directions give 1.81 crossings,
    /// 5.69 cm times π, not 6.
    #[test]
    fn a_few_directions_only_estimate() {
        let bent = placed(shape, HOME, ALONG, BENT);
        let fine = bent.count_shares(SHARE_DIRECTIONS, Some(GAP));
        let coarse = bent.count_shares(36, Some(GAP));
        let off = fine.iter().zip(&coarse).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(off > 1e-4, "{off}");
        let four = bent.fixed_directions_estimate(4, GAP);
        assert_eq!(facts::fixed(four / PI, 2), "1.81");
        assert!((four - LENGTH).abs() > 0.1);
    }

    /// Folded double, the wire lies along 3 cm twice: counted piece by
    /// piece it averages 6 ÷ π; counted as points of the card (one layer),
    /// 3 ÷ π, as if 3 cm long (a negative control on lost multiplicity).
    #[test]
    fn folding_keeps_its_crossings_counted_twice() {
        let folded = chain(&[[-1.5, 0.0], [1.5, 0.0], [-1.5, 0.0]]);
        let both = model::moments(&folded.count_shares(SHARE_DIRECTIONS, Some(GAP))).0;
        let one = model::moments(&line([-1.5, 0.0], [1.5, 0.0]).count_shares(SHARE_DIRECTIONS, Some(GAP))).0;
        assert!((both - LENGTH / PI).abs() < 1e-6 && (one - LENGTH / 2.0 / PI).abs() < 1e-6);
        let pose = Pose::new(FRAC_PI_2, 0.3);
        let c = sheet::crossings(&folded, pose.beta, pose.slide);
        let mut points: Vec<P2> = c.iter().map(|x| x.at).collect();
        points.dedup_by(|a, b| why_every_curve::dist(*a, *b) < 1e-9);
        assert_eq!((c.len(), points.len()), (2, 1));
    }

    /// Lines another gap apart: the bent wire's mean changes, and times
    /// that sheet's worth it is still 6 cm; times this sheet's π it is not
    /// (a negative control on the spacing).
    #[test]
    fn the_spacing_sets_the_worth() {
        let bent = placed(shape, HOME, ALONG, BENT);
        for gap in [1.0, 4.0] {
            let m = model::moments(&bent.count_shares(SHARE_DIRECTIONS, Some(gap))).0;
            assert!((m * model::worth(gap) - LENGTH).abs() < 1e-5, "{gap}");
            assert!((m * PI - LENGTH).abs() > 1.0, "{gap}");
        }
    }

    /// The 4's last piece crosses its first arm at its meeting point; the
    /// line through it there counts both pieces, and no end or joint of the
    /// 4 lies within 0.1 cm of a line at 13_5's placement.
    #[test]
    fn the_four_counts_both_pieces_at_its_crossing() {
        let four = placed(shape, HOME, ALONG, FOUR);
        let meet = four_meet();
        let a = &four.parts[0].seg;
        let c = &four.parts[2].seg;
        let (x0, x1) = (a.point(0.0)[0], a.point(1.0)[0]);
        assert!(x0.min(x1) < meet[0] && meet[0] < x0.max(x1));
        assert!(c.point(1.0)[1] < meet[1] && meet[1] < c.point(0.0)[1]);
        let at = four_move(first()).end();
        let here = sheet::crossings(&four, at.beta, at.slide);
        let pieces: Vec<usize> = here
            .iter()
            .filter(|x| why_every_curve::dist(x.at, meet) < 1e-9)
            .map(|x| x.piece)
            .collect();
        assert_eq!(pieces, [0, 2]);
        let n = [-at.beta.sin(), at.beta.cos()];
        for part in &four.parts {
            for p in [part.seg.point(0.0), part.seg.point(1.0)] {
                let t = (p[0] * n[0] + p[1] * n[1] - at.slide) / GAP;
                assert!((t - t.round()).abs() * GAP > 0.1, "{p:?}");
            }
        }
    }

    /// The wire keeps 0.4 cm from the coast and inside the card through
    /// every turn and bend shown; its labels clear of it and the coast.
    #[test]
    fn the_wire_keeps_off_the_coast() {
        let coast = model::coast();
        let shore = coast.between(0.0, coast.length, 0.02);
        let near = |c: &Curve| {
            c.parts
                .iter()
                .flat_map(|p| (0..=60).map(move |k| p.seg.point(f64::from(k) / 60.0)))
                .map(|q| {
                    shore
                        .iter()
                        .map(|&s| why_every_curve::dist(s, q))
                        .fold(f64::MAX, f64::min)
                })
                .fold(f64::MAX, f64::min)
        };
        let inside = |c: &Curve| {
            c.parts.iter().all(|p| {
                [p.seg.point(0.0), p.seg.point(1.0)]
                    .iter()
                    .all(|q| q[0].abs() < CARD_W / 2.0 - 0.3 && q[1].abs() < CARD_H / 2.0 - 0.25)
            })
        };
        for i in 0..=40 {
            let t = f64::from(i) / 40.0;
            let turning = placed(shape, HOME, ACROSS + (ALONG - ACROSS) * t, STRAIGHT);
            let bending = placed(shape, HOME, ALONG, FOUR * t);
            for w in [&turning, &bending] {
                assert!(near(w) > 0.4 && inside(w), "{t}");
            }
        }
        for at in ARM_LABELS {
            assert!(shore.iter().all(|&s| why_every_curve::dist(s, at) > 0.8), "{at:?}");
        }
        // each label beside its arm's middle: the arm along the lines is one
        // piece, the raised arm two (its middle their joint)
        let bent = placed(shape, HOME, ALONG, BENT);
        let middles = [bent.parts[0].seg.point(0.5), bent.parts[1].seg.point(1.0)];
        for (at, mid) in ARM_LABELS.iter().zip(middles) {
            let d = why_every_curve::dist(*at, mid);
            assert!(d > 0.45 && d < 1.2, "{at:?}: {d}");
        }
        // the raised arm passes left of the count's digits, the 4's joint
        // under them
        let top = card_point(bent.parts[2].seg.point(1.0));
        assert!(top[0] + 0.1 < placement::COUNT_RIGHT[0] - width("4", sheet::COUNT_SIZE) - 0.2);
        let joint = card_point(placed(shape, HOME, ALONG, FOUR).parts[2].seg.point(0.0));
        assert!(joint[1] < placement::COUNT_RIGHT[1] - half_height(sheet::COUNT_SIZE) - 0.3);
    }

    /// The spreads fit the console: bars, percents and counts inside it
    /// and apart, names and means left of their bars.
    #[test]
    fn the_spreads_keep_their_room() {
        let w = words().unwrap();
        let console = bounds(CONSOLE);
        let rights = [NAME_X[1] - 0.3, console[1] - 0.2];
        for (k, heights) in w.heights.iter().enumerate() {
            let last = BAR_X[k] + BAR_STEP * (heights.len() - 1) as f64;
            let words_right = NAME_X[k] + width(&w.names[k], LABEL_SIZE).max(width(&w.average, SPREAD_SIZE));
            assert!(BAR_X[k] - BAR_W / 2.0 > words_right + 0.2, "{k}");
            assert!(last + BAR_W / 2.0 < rights[k], "{k}");
            for (&p, percent) in heights.iter().zip(&w.percents[k]) {
                let top = BAR_BASE + (BAR_K * p).max(0.01) + PERCENT_GAP + half_height(SPREAD_SIZE);
                assert!(top < console[3] - 0.05, "{k} {percent}");
                assert!(width(percent, SPREAD_SIZE) < BAR_STEP - 0.15);
            }
        }
        assert!(COUNT_Y - half_height(SPREAD_SIZE) > console[2] + 0.02);
        assert!(COUNT_Y + half_height(SPREAD_SIZE) < BAR_BASE - 0.02);
        assert!(NAME_Y - half_height(LABEL_SIZE) > AVERAGE_Y + half_height(SPREAD_SIZE) + 0.05);
        // the rows inside the console, 13_1's shadow and share side by side
        for tex in [&w.mean, &w.arms, &w.adds, &w.coast] {
            assert!(LEFT_X + width(&shown(tex), ROW_SIZE) < console[1] - 0.2, "{tex}");
        }
        let pair = width(&shown(&w.reach), ROW_SIZE) + 0.5 + width(&shown(&w.share), ROW_SIZE);
        assert!(LEFT_X + pair < console[1] - 0.2);
        assert!(w.exact.chars().count() <= 2 * 92 && w.source.chars().count() <= 92);
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("13_1", "Back to our six centimetre stick"),
            ("13_1", "about one point nine one crossings"),
            ("13_1", "twelve over pi"),
            ("13_1", "sixty-four percent"),
            ("13_2", "Turn it to lie along the lines"),
            ("13_2", "Now bend the same wire"),
            ("13_2", "Its length hasn't changed"),
            ("13_2", "or stay the same"),
            ("13_3", "It stays the same"),
            ("13_3", "Turn the sheet, and its count moves around"),
            ("13_4", "What changes is how the count is spread"),
            ("13_4", "Bent, it never does"),
            ("13_5", "Bend it once more"),
            ("13_5", "Where its pieces cross"),
            ("13_5", "counts both of them"),
            ("13_5", "still one point nine one"),
            ("13_6", "Buffon's noodle"),
            ("13_7", "our coast is just a long bent wire"),
            ("13_7", "seven times"),
            ("13_7", "gives back its length"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// 13_3's half turn about the corner's line ends where it began, and
    /// the counts it passes are the facts'.
    #[test]
    fn the_sweep_ends_where_it_began() {
        let [mv] = sweep(first());
        assert!(mv.end().same_lines(first(), 1e-9));
        let bent = placed(shape, HOME, ALONG, BENT);
        assert_eq!(count_at(&bent, mv.end()), count_at(&bent, first()));
    }
}
