//! Chapter 05, a coin that never changes. From 04's question the coast dims
//! and the sheet stays at the first placement. A coin 2 cm across, exactly
//! as wide as the gap (05_1), lies on the sheet: its rim crosses one line
//! twice. Slid and turned anywhere, how many crossings? (05_2, a
//! prediction.) Always two: one line, and only one, runs through it, and a
//! circle turned looks the same (05_3); then it is set down at four more
//! places, passing every way between lines, the count 2 throughout. When
//! it fits exactly between two lines it touches each at a single point,
//! still two (05_4); nudged off that placement either way, one line runs
//! through it again. Its rim is π × 2 cm; rolled one full turn along a line
//! it covers 6.28 cm (05_5). Two crossings for 6.28 cm: each is worth π cm,
//! about 3.14 (05_6). A crossing is wherever the rim meets a line (02_4):
//! the coin's meeting points, the line through it and the count are
//! computed every frame from where it lies. Starts where chapter 04 ends.
//! Not registered yet: root wires it in (`docs/candidate-04-05.md`).

use std::f64::consts::{PI, TAU};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::core::rate::smooth;
use fmn::prelude::*;

use super::draw::{
    EVERY, STAND_UP, card_panel, card_point, dim_line, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now,
    path, pause, say, stand, think, together, wait_until,
};
use super::placement::{
    self, COUNT_RIGHT, CROSSINGS, HALO_OPACITY, HALO_W, QUESTION, QUESTION_AT, WORD_GAP, at_once, faded_line, glide,
    tracked,
};
use super::sheet::{self, COUNT_SIZE, DOT_R, LABEL_SIZE, LOW_Y, Sheet, THINK_AT, crossing_dot};
use crate::facts;
use crate::model::{self, COIN, GAP, P2};
use crate::palette;

pub struct ACoinThatNeverChanges {
    pub kit: Kit,
}

// ------------------------------------------------------------ the coin

/// The coin is [`COIN`] across (`coin.width`), exactly the gap, as the
/// model asserts.
const RADIUS: f64 = COIN / 2.0;
/// How near a line's distance from the centre must come to the radius to
/// count as touching, cm: far below any drawn difference, far above the
/// float error of a glide that lands on a touching placement.
const TOUCH: f64 = 1e-9;

/// A point where the rim meets a line of the sheet (line k as
/// [`sheet::crossings`] numbers it), and whether the line only touches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Meeting {
    pub at: P2,
    pub line: i64,
    pub touch: bool,
}

/// Where a coin centred at `c`, radius `r`, meets the lines of the sheet at
/// direction 0 and `slide`: two points on a line nearer the centre than `r`,
/// one where a line is `r` away (to [`TOUCH`]), none on a line further off.
/// Lowest line first, left to right.
pub fn meetings(c: P2, r: f64, slide: f64) -> Vec<Meeting> {
    let first = ((c[1] - r - TOUCH - slide) / GAP).floor() as i64;
    let last = ((c[1] + r + TOUCH - slide) / GAP).ceil() as i64;
    let mut out = Vec::new();
    for line in first..=last {
        let h = slide + line as f64 * GAP;
        let d = (h - c[1]).abs();
        if (d - r).abs() <= TOUCH {
            out.push(Meeting {
                at: [c[0], h],
                line,
                touch: true,
            });
        } else if d < r {
            let w = (r * r - d * d).sqrt();
            for x in [c[0] - w, c[0] + w] {
                out.push(Meeting {
                    at: [x, h],
                    line,
                    touch: false,
                });
            }
        }
    }
    out
}

/// The lines that run through the coin, not just touching it.
pub fn through(c: P2, r: f64, slide: f64) -> Vec<i64> {
    let mut lines: Vec<i64> = meetings(c, r, slide)
        .iter()
        .filter(|m| !m.touch)
        .map(|m| m.line)
        .collect();
    lines.dedup();
    lines
}

/// What a coin of radius `r` shows at every twentieth of a centimetre
/// across one gap of the sheet at `slide` (direction 0; the touching
/// heights are among them), each list sorted without repeats: how many
/// points it meets the lines at, how many lines run through it where it
/// touches none, and where it touches, how many lines and how many points
/// on each. `facts.rs` prints the coin's facts from it.
pub struct Survey {
    pub counts: Vec<usize>,
    pub through: Vec<usize>,
    pub touched: Vec<usize>,
    pub points: Vec<usize>,
}

impl Survey {
    pub fn across_a_gap(r: f64, slide: f64) -> Self {
        let mut s = Self {
            counts: Vec::new(),
            through: Vec::new(),
            touched: Vec::new(),
            points: Vec::new(),
        };
        let steps = (GAP * 20.0).round() as u32;
        for i in 0..=steps {
            let at = [0.0, slide + f64::from(i) / 20.0];
            let met = meetings(at, r, slide);
            s.counts.push(met.len());
            let mut lines: Vec<i64> = met.iter().filter(|m| m.touch).map(|m| m.line).collect();
            lines.dedup();
            if lines.is_empty() {
                s.through.push(through(at, r, slide).len());
            } else {
                s.touched.push(lines.len());
                for k in lines {
                    s.points.push(met.iter().filter(|m| m.line == k).count());
                }
            }
        }
        for v in [&mut s.counts, &mut s.through, &mut s.touched, &mut s.points] {
            v.sort_unstable();
            v.dedup();
        }
        s
    }
}

/// The words the voice says the coin shows: its count (05_2, 05_3, 05_6,
/// `coin.crossings`), one line through it (05_3, `coin.lines_through`),
/// and at the touching placement two lines touched (05_4,
/// `coin.touching_lines`) at a single point each (`coin.touch_points`).
const SAID_COUNT: usize = 2;
const SAID_THROUGH: usize = 1;
const SAID_TOUCHED: usize = 2;
const SAID_POINTS: usize = 1;

// ------------------------------------------------------------------ layout

const COIN_FILL: f64 = 0.16;
const RIM_W: f64 = 4.0;
const TICK_W: f64 = 3.0;
/// The turn mark: a stretch of radius from this share of it to the rim.
const TICK_FROM: f64 = 0.35;
const TRAIL_W: f64 = 5.0;
const COAST_W: f64 = 4.0;
/// What the card and coast dim to while the coin is the subject.
const DIMMED: f64 = 0.25;
const BAR_W: f64 = 2.5;
const BAR_TICK: f64 = 0.15;
const RING_R: f64 = 0.17;
const EQUATION_SIZE: f64 = 28.0;

/// Where the coin lies on the card, cm: dropped in the band over the coast
/// across the line at 2.45 (05_1, 05_2); set down at four more places, the
/// last across the line at 2.45 above the touching placement (05_3); the
/// touching placement, between the lines at 0.45 and 2.45, over the
/// coast's low stretch on the right (05_4), nudged up and down; the roll,
/// on the line at 2.45, under the line at 4.45 (05_5).
const DROP: P2 = [1.5, 3.05];
const LANDINGS: [P2; 4] = [[2.6, 0.75], [-1.5, -3.2], [5.2, -2.3], [4.5, 1.85]];
/// How far each landing turns the coin, whole turns.
const LANDING_TURNS: [f64; 4] = [0.6, -0.9, 0.75, -0.5];
const TOUCH_X: f64 = 4.5;
const NUDGE: f64 = 0.2;
const ROLL_X: f64 = 0.0;
/// 05_1's bars, the coin's width and the gap, to its right, and the
/// width's label beyond them.
const WIDTH_BAR_X: f64 = DROP[0] + 1.3;
const GAP_BAR_X: f64 = DROP[0] + 1.7;
const WIDTH_LABEL_AT: P2 = [DROP[0] + 2.45, DROP[1]];
/// The roll's label over the middle of its trail, cm above the line.
const ROLL_LABEL_UP: f64 = 0.4;
/// 05_5's rim and 05_6's two rows, in the band under the coast, frame.
const RIM_AT: Vec3 = [1.8, LOW_Y, 0.0];
const ROWS_AT: [Vec3; 2] = [[1.8, LOW_Y + 0.22, 0.0], [1.8, LOW_Y - 0.22, 0.0]];

// ------------------------------------------------------------------ timing

const OPEN_S: f64 = 1.0;
const FADE_S: f64 = 0.4;
const SPIN_S: f64 = 2.4;
const LEG_S: f64 = 1.6;
const REST_S: f64 = 0.5;
const APPROACH_S: f64 = 2.5;
const NUDGE_S: f64 = 0.8;
const TO_ROLL_S: f64 = 1.6;
const ROLL_S: f64 = 3.5;

/// Every place the coin lies, from the sheet at `slide` (the touching
/// placement's and the roll's heights are its lines').
struct Plan {
    drop: P2,
    landings: [P2; 4],
    stop: P2,
    nudges: [P2; 4],
    roll_from: P2,
    roll_to: P2,
}

fn plan(slide: f64) -> Plan {
    let stop = [TOUCH_X, slide + RADIUS];
    let roll_from = [ROLL_X, slide + GAP + RADIUS];
    Plan {
        drop: DROP,
        landings: LANDINGS,
        stop,
        nudges: [[stop[0], stop[1] + NUDGE], stop, [stop[0], stop[1] - NUDGE], stop],
        roll_from,
        roll_to: [roll_from[0] + TAU * RADIUS, roll_from[1]],
    }
}

impl Plan {
    /// Every stretch the coin's centre moves along, in order (each glide
    /// is straight, its pace eased).
    fn moves(&self) -> Vec<[P2; 2]> {
        let mut stops = vec![self.drop];
        stops.extend(self.landings);
        stops.push(self.stop);
        stops.extend(self.nudges);
        stops.push(self.roll_from);
        stops.push(self.roll_to);
        stops.windows(2).map(|w| [w[0], w[1]]).collect()
    }
}

/// The chapter's words that carry numbers, each from the facts, refused
/// unless the coin is the gap's width, and the model gives what the voice
/// says: 2 crossings and one line through at the drop, 2 crossings on
/// every move, two lines touched at a single point each at the touching
/// placement, and a rim and worth the facts round the same way.
struct Words {
    width: String,
    rim_tex: String,
    rim: String,
    rows: [String; 2],
}

fn words(slide: f64, plan: &Plan) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let (width, gap) = (get("coin.width")?, get("sheet.gap")?);
    if width != facts::fixed(COIN, 0) || width != gap {
        return Err(format!(
            "the coin is drawn {COIN} cm across; the facts say {width}, the gap {gap}"
        ));
    }
    let said = [
        ("coin.crossings", SAID_COUNT),
        ("coin.lines_through", SAID_THROUGH),
        ("coin.touching_lines", SAID_TOUCHED),
        ("coin.touch_points", SAID_POINTS),
    ];
    for (key, n) in said {
        if get(key)? != n.to_string() {
            return Err(format!("the voice says {n} for {key}, the facts {}", get(key)?));
        }
    }
    let at_drop = meetings(plan.drop, RADIUS, slide);
    if at_drop.len() != SAID_COUNT || through(plan.drop, RADIUS, slide).len() != SAID_THROUGH {
        return Err(format!(
            "05_2 says the dropped coin crosses one line twice: {at_drop:?}"
        ));
    }
    for [a, b] in plan.moves() {
        for i in 0..=200 {
            let u = i as f64 / 200.0;
            let c = [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
            let n = meetings(c, RADIUS, slide).len();
            if n != SAID_COUNT {
                return Err(format!(
                    "05_3 says always two; at {c:?} the coin meets the lines {n} times"
                ));
            }
        }
    }
    let at_stop = meetings(plan.stop, RADIUS, slide);
    let touched = at_stop.iter().filter(|m| m.touch).count();
    if touched != SAID_TOUCHED || at_stop.len() != SAID_TOUCHED * SAID_POINTS {
        return Err(format!(
            "05_4 says two lines touched at a single point each: {at_stop:?}"
        ));
    }
    let (rim, worth) = (get("coin.rim")?, get("coin.worth")?);
    let rim_cm = PI * COIN;
    if rim != facts::fixed(rim_cm, 2) || worth != facts::fixed(rim_cm / SAID_COUNT as f64, 2) {
        return Err(format!(
            "the rim {rim_cm} over {SAID_COUNT} crossings; the facts say {rim} and {worth}"
        ));
    }
    Ok(Words {
        width: format!("{width} cm"),
        rim_tex: format!(r"\text{{rim}} = \pi \times {width}\text{{ cm}}"),
        rim: format!("{rim} cm"),
        rows: [
            format!(r"{SAID_COUNT}\text{{ crossings}} \leftrightarrow {rim}\text{{ cm}}"),
            format!(r"1\text{{ crossing}} \leftrightarrow \pi\text{{ cm}} \approx {worth}\text{{ cm}}"),
        ],
    })
}

// ---------------------------------------------------------- the live coin

/// The trackers the coin is drawn from: its centre (card cm) and how far
/// it has turned (radians, counterclockwise).
#[derive(Clone, Copy)]
struct Coin {
    cx: Mob,
    cy: Mob,
    spin: Mob,
}

impl Coin {
    fn new(stage: &mut Stage<'_>, at: P2) -> Self {
        Self {
            cx: stage.add_value_tracker(at[0]),
            cy: stage.add_value_tracker(at[1]),
            spin: stage.add_value_tracker(0.0),
        }
    }

    fn centre(&self, s: &fmn::mobject::Stage) -> P2 {
        [
            s.tracker_value(self.cx).unwrap_or(0.0),
            s.tracker_value(self.cy).unwrap_or(0.0),
        ]
    }

    fn turned(&self, s: &fmn::mobject::Stage) -> f64 {
        s.tracker_value(self.spin).unwrap_or(0.0)
    }

    /// Slide the coin to `to`, turning it `turn` more, `rt` seconds; it
    /// ends exactly there (a glide lands within float error of its stop).
    fn go(&self, stage: &mut Stage<'_>, to: P2, turn: f64, rt: f64) -> fmn::Result<()> {
        let spin = self.turned(stage) + turn;
        together(
            stage,
            vec![
                glide(self.cx, to[0], rt, smooth)?,
                glide(self.cy, to[1], rt, smooth)?,
                glide(self.spin, spin, rt, smooth)?,
            ],
        )?;
        for (m, x) in [(self.cx, to[0]), (self.cy, to[1]), (self.spin, spin)] {
            stage.set_tracker_value(m, x)?;
        }
        Ok(())
    }
}

/// The coin where its trackers put it: a faint teal disc, its rim, and a
/// mark that shows it turning, at the opacity `alpha` holds. In the scene.
fn live_coin(stage: &mut Stage<'_>, coin: Coin, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let (o, c, a) = (tracked(s, alpha), coin.centre(s), coin.turned(s) - PI / 2.0);
        let at = card_point(c);
        let r = model::CARD_UNIT * RADIUS;
        let disc = Circle::new()
            .radius(r)
            .arc_center(at)
            .style(flat(palette::coin(), COIN_FILL * o))
            .build();
        let rim = Circle::new()
            .radius(r)
            .arc_center(at)
            .style(faded_line(palette::coin(), RIM_W, o))
            .build();
        let mark = polyline(vec![
            card_point([c[0] + TICK_FROM * RADIUS * a.cos(), c[1] + TICK_FROM * RADIUS * a.sin()]),
            card_point([c[0] + RADIUS * a.cos(), c[1] + RADIUS * a.sin()]),
        ])
        .style(faded_line(Palette::ink(), TICK_W, o))
        .build();
        s.add(v_group(vec![disc, rim, mark]))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The coin's meeting points with the sheet at `slide`, gold dots as
/// [`sheet::crossing_dot`] draws them, at the opacity `alpha` holds. In the
/// scene.
fn live_meetings(stage: &mut Stage<'_>, coin: Coin, slide: f64, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let dots: Vec<VMobject> = meetings(coin.centre(s), RADIUS, slide)
            .iter()
            .map(|m| {
                Circle::new()
                    .radius(DOT_R)
                    .arc_center(card_point(m.at))
                    .style(flat(palette::touch(), o))
                    .build()
            })
            .collect();
        s.add(v_group(dots))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A halo along each line that runs through the coin (not one it only
/// touches), at the opacity `alpha` holds. In the scene: make it before
/// the lines, so they are drawn over it.
fn live_through(stage: &mut Stage<'_>, coin: Coin, slide: f64, alpha: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let o = tracked(s, alpha);
        let halos: Vec<VMobject> = through(coin.centre(s), RADIUS, slide)
            .into_iter()
            .filter_map(|k| placement::clipped(0.0, slide + k as f64 * GAP))
            .map(|[a, b]| {
                polyline(vec![card_point(a), card_point(b)])
                    .style(faded_line(Palette::ink(), HALO_W, HALO_OPACITY * o))
                    .build()
            })
            .collect();
        s.add(v_group(halos))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The rim laid along the line under the roll: from `from` (card cm), as
/// far as `rolled` holds. In the scene.
fn live_trail(stage: &mut Stage<'_>, from: P2, rolled: Mob) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let length = s.tracker_value(rolled).unwrap_or(0.0);
        let trail: Vec<VMobject> = (length > 1e-6)
            .then(|| {
                polyline(vec![card_point(from), card_point([from[0] + length, from[1]])])
                    .style(line_style(palette::coin(), TRAIL_W))
                    .build()
            })
            .into_iter()
            .collect();
        s.add(v_group(trail))
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// An upright bar from `lo` to `hi` at `x` (card cm), with end ticks.
/// Detached.
fn bar(stage: &mut Stage<'_>, x: f64, [lo, hi]: [f64; 2], color: Srgb) -> Vec<Mob> {
    let mut out = vec![path(stage, &[[x, lo], [x, hi]], color, BAR_W)];
    for y in [lo, hi] {
        out.push(path(stage, &[[x - BAR_TICK, y], [x + BAR_TICK, y]], color, BAR_W));
    }
    out
}

/// A gold ring round a card point. Detached.
fn ring(stage: &mut Stage<'_>, at: P2) -> Mob {
    let ring = Circle::new()
        .radius(RING_R)
        .arc_center(card_point(at))
        .style(line_style(palette::touch(), 2.5))
        .build();
    stage.arena_mut().add(ring)
}

impl SceneConstruct for ACoinThatNeverChanges {
    fn name(&self) -> &str {
        "05_a_coin_that_never_changes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let plan = plan(slide);
        let words = words(slide, &plan).map_err(fail)?;

        // where 04 ended: the card, the coast and the sheet at the first
        // placement with its seven crossings, the question, the envelope;
        // the coin's halo under the lines, its trail and the coin over them
        let card = card_panel(stage)?;
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
            stage.add_to_scene(m)?;
        }
        let ruled = Sheet::new(stage, slide, 0.0)?;
        stage.add_to_scene(ruled.panel)?;
        let coin = Coin::new(stage, plan.drop);
        let halo_alpha = stage.add_value_tracker(0.0);
        live_through(stage, coin, slide, halo_alpha)?;
        for &m in &ruled.lines {
            stage.add_to_scene(m)?;
        }
        let dots: Vec<Mob> = sheet::crossings(&coast, 0.0, slide)
            .iter()
            .map(|c| crossing_dot(stage, c.at))
            .collect();
        for &m in &dots {
            stage.add_to_scene(m)?;
        }
        let question = kit.text_t2c(
            stage,
            QUESTION,
            COUNT_SIZE,
            Palette::ink(),
            &[("?", palette::touch())],
            QUESTION_AT,
        )?;
        stage.add_to_scene(question)?;
        for m in placement::envelope(&kit, stage)? {
            stage.add_to_scene(m)?;
        }
        let rolled = stage.add_value_tracker(0.0);
        live_trail(stage, [plan.roll_from[0], plan.roll_from[1] - RADIUS], rolled)?;
        let coin_alpha = stage.add_value_tracker(0.0);
        live_coin(stage, coin, coin_alpha)?;
        let dots_alpha = stage.add_value_tracker(0.0);
        live_meetings(stage, coin, slide, dots_alpha)?;

        // 05_1: the coast steps back; a coin of known width, as wide as
        // the gap
        let line = say(&kit, stage, "05_1")?;
        let mut dim = vec![fade_to(card, DIMMED, OPEN_S)?];
        for &m in &pieces {
            dim.push(dim_line(m, DIMMED, OPEN_S)?);
        }
        let mut gone = dots.clone();
        gone.push(question);
        at_once(stage, &[], &gone, dim, OPEN_S)?;
        line.wait_for(stage, "Take a coin")?;
        stage.play(tween(coin_alpha, 1.0, 0.6)?)?;
        line.wait_for(stage, "two centimetres across")?;
        let mut bars = bar(
            stage,
            WIDTH_BAR_X,
            [plan.drop[1] - RADIUS, plan.drop[1] + RADIUS],
            palette::coin(),
        );
        bars.push(kit.text(
            stage,
            &words.width,
            LABEL_SIZE,
            palette::coin(),
            card_point(WIDTH_LABEL_AT),
        )?);
        fade_in_all(stage, &bars, FADE_S)?;
        line.wait_for(stage, "exactly as wide as the gap")?;
        let gap = bar(stage, GAP_BAR_X, [slide + GAP, slide + 2.0 * GAP], Palette::ink());
        fade_in_all(stage, &gap, FADE_S)?;
        bars.extend(gap);

        // 05_2: on the sheet its rim crosses this line twice; anywhere?
        let line = say(&kit, stage, "05_2")?;
        line.wait_for(stage, "Drop it on the sheet")?;
        at_once(stage, &[], &bars, vec![tween(dots_alpha, 1.0, FADE_S)?], FADE_S)?;
        line.wait_for(stage, "crosses this line twice")?;
        stage.play(tween(halo_alpha, 1.0, FADE_S)?)?;
        let q = think(&kit, stage, THINK_AT)?;
        fade_out_all(stage, &[q], 0.3)?;

        // 05_3: always two; the count, then a turn on the spot, then four
        // more places in silence, each one line through, the count 2
        let line = say(&kit, stage, "05_3")?;
        line.wait_for(stage, "Always two")?;
        let count_alpha = stage.add_value_tracker(0.0);
        let count_now = move |s: &fmn::mobject::Stage| meetings(coin.centre(s), RADIUS, slide).len();
        placement::live_count(&kit, stage, count_now, count_alpha, COUNT_SIZE, COUNT_RIGHT)?;
        let word_at = v(COUNT_RIGHT[0] + WORD_GAP, COUNT_RIGHT[1]);
        placement::live_word(&kit, stage, CROSSINGS, count_alpha, COUNT_SIZE, word_at)?;
        stage.play(tween(count_alpha, 1.0, FADE_S)?)?;
        line.wait_for(stage, "Turning changes nothing")?;
        coin.go(stage, plan.drop, TAU, SPIN_S)?;
        kit.hold(stage)?;
        for (&to, turns) in plan.landings.iter().zip(LANDING_TURNS) {
            coin.go(stage, to, turns * TAU, LEG_S)?;
            pause(stage, REST_S)?;
        }

        // 05_4: exactly between two lines, touching both, a single point
        // each; nudged off it either way, one line runs through again
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, "when the coin fits exactly")?;
        coin.go(stage, plan.stop, 0.0, APPROACH_S)?;
        line.wait_for(stage, "a single point")?;
        let touches: Vec<Mob> = meetings(plan.stop, RADIUS, slide)
            .iter()
            .map(|m| ring(stage, m.at))
            .collect();
        fade_in_all(stage, &touches, 0.3)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        fade_out_all(stage, &touches, 0.3)?;
        for to in plan.nudges {
            coin.go(stage, to, 0.0, NUDGE_S)?;
        }
        let still = now(stage);
        wait_until(stage, still + STAND_UP)?;

        // 05_5: its rim is π times its width; rolled one full turn along
        // the line under it, it lays out 6.28 cm
        let line = say(&kit, stage, "05_5")?;
        let rim_tex = kit.tex(stage, &words.rim_tex, EQUATION_SIZE, Palette::ink(), &[], RIM_AT)?;
        fade_in_all(stage, &[rim_tex], FADE_S)?;
        // the mark points down, at the line it will roll along
        let spun = coin.turned(stage);
        coin.go(stage, plan.roll_from, (spun / TAU).round() * TAU - spun, TO_ROLL_S)?;
        line.wait_for(stage, "Roll the coin one full turn")?;
        let (length, spin) = (plan.roll_to[0] - plan.roll_from[0], coin.turned(stage));
        together(
            stage,
            vec![
                glide(coin.cx, plan.roll_to[0], ROLL_S, smooth)?,
                glide(coin.spin, spin - length / RADIUS, ROLL_S, smooth)?,
                glide(rolled, length, ROLL_S, smooth)?,
            ],
        )?;
        for (m, x) in [
            (coin.cx, plan.roll_to[0]),
            (coin.spin, spin - length / RADIUS),
            (rolled, length),
        ] {
            stage.set_tracker_value(m, x)?;
        }
        line.wait_for(stage, "six point two eight")?;
        let label_at = [
            (plan.roll_from[0] + plan.roll_to[0]) / 2.0,
            plan.roll_from[1] - RADIUS + ROLL_LABEL_UP,
        ];
        let label = kit.text(stage, &words.rim, LABEL_SIZE, palette::coin(), card_point(label_at))?;
        fade_in_all(stage, &[label], FADE_S)?;

        // 05_6: two crossings for 6.28 cm, so one is worth π cm
        let line = say(&kit, stage, "05_6")?;
        fade_out_all(stage, &[rim_tex], 0.3)?;
        let first = kit.tex(stage, &words.rows[0], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[0])?;
        fade_in_all(stage, &[first], FADE_S)?;
        line.wait_for(stage, "each crossing is worth")?;
        let second = kit.tex(stage, &words.rows[1], EQUATION_SIZE, Palette::ink(), &[], ROWS_AT[1])?;
        fade_in_all(stage, &[second], FADE_S)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::THINK_R;
    use super::super::sheet::TOP_Y;
    use super::*;
    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W, Curve};

    fn shown() -> (Curve, f64, Plan) {
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        (coast, slide, plan(slide))
    }

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The coast at every 0.02 cm of its length, straight runs included.
    fn along(coast: &Curve) -> Vec<P2> {
        let n = (coast.length / 0.02).ceil() as usize;
        (0..=n).map(|i| coast.at(coast.length * i as f64 / n as f64)).collect()
    }

    /// Points on the coin's moves, every 0.01 cm or less of each stretch.
    fn on_the_way(plan: &Plan) -> Vec<P2> {
        let mut out = Vec::new();
        for [a, b] in plan.moves() {
            let n = (dist(a, b) / 0.01).ceil().max(1.0) as usize;
            out.extend((0..=n).map(|i| {
                let u = i as f64 / n as f64;
                [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u]
            }));
        }
        out
    }

    /// The meeting rule at every height a twentieth of a centimetre apart,
    /// against exact integer arithmetic in twentieths (lines at 9 + 40k,
    /// the first placement's 0.45 + 2k cm; a radius r is 20r): a line d
    /// twentieths from the centre is met twice when 50d < 1000r and once
    /// when 50d = 1000r. Radius 1 always meets the lines twice; radii 0.999
    /// and 1.001, never touching exactly, meet them 0 or 2 and 2 or 4 times.
    #[test]
    fn the_meeting_rule_is_exact_at_every_twentieth() {
        let (_, slide, _) = shown();
        assert_eq!(facts::fixed(slide, 2), "0.45");
        let cases: [(f64, i64, &[usize]); 3] = [(1.0, 1000, &[2]), (0.999, 999, &[0, 2]), (1.001, 1001, &[2, 4])];
        for (r, thousandths, want) in cases {
            let mut seen = std::collections::BTreeSet::new();
            for c20 in -70..=70i64 {
                let away: Vec<i64> = (-4..=4i64).map(|k| (9 + 40 * k - c20).abs()).collect();
                let crossed = away.iter().filter(|&&d| 50 * d < thousandths).count();
                let touched = away.iter().filter(|&&d| 50 * d == thousandths).count();
                let got = meetings([0.3, c20 as f64 / 20.0], r, slide);
                assert_eq!(got.len(), 2 * crossed + touched, "radius {r} at {c20}/20 cm");
                assert_eq!(
                    got.iter().filter(|m| m.touch).count(),
                    touched,
                    "radius {r} at {c20}/20 cm"
                );
                seen.insert(got.len());
            }
            assert_eq!(seen.into_iter().collect::<Vec<_>>(), want, "radius {r}");
        }
    }

    /// Across one gap the coin always meets the lines twice, one line runs
    /// through it wherever it touches none, and where it touches, it
    /// touches two lines at one point each: what `facts.rs` prints. Coins
    /// a thousandth narrower and wider give more than one answer, never
    /// touching.
    #[test]
    fn the_survey_across_a_gap_is_one_answer_for_this_coin_only() {
        let (_, slide, _) = shown();
        let s = Survey::across_a_gap(RADIUS, slide);
        assert_eq!(
            (s.counts, s.through, s.touched, s.points),
            (vec![2], vec![1], vec![2], vec![1])
        );
        let narrow = Survey::across_a_gap(0.999, slide);
        assert_eq!(
            (narrow.counts, narrow.through, narrow.touched),
            (vec![0, 2], vec![0, 1], vec![])
        );
        let wide = Survey::across_a_gap(1.001, slide);
        assert_eq!(
            (wide.counts, wide.through, wide.touched),
            (vec![2, 4], vec![1, 2], vec![])
        );
    }

    /// Always two, wherever the coin lies on its way, at every 0.01 cm of
    /// every move, one line through it except where it touches two; and
    /// the moves pass between lines, through touching placements, at
    /// least three times.
    #[test]
    fn the_coin_meets_the_lines_twice_all_the_way() {
        let (_, slide, plan) = shown();
        for c in on_the_way(&plan) {
            let m = meetings(c, RADIUS, slide);
            assert_eq!(m.len(), 2, "{c:?}: {m:?}");
            let lines = through(c, RADIUS, slide).len();
            let touched = m.iter().filter(|x| x.touch).count();
            assert!((lines, touched) == (1, 0) || (lines, touched) == (0, 2), "{c:?}: {m:?}");
        }
        // heights where the coin touches two lines, crossed by a move
        let passes = plan
            .moves()
            .iter()
            .filter(|[a, b]| {
                let (lo, hi) = (a[1].min(b[1]), a[1].max(b[1]));
                (-3..=3).any(|k| {
                    let h = slide + RADIUS + k as f64 * GAP;
                    lo < h && h < hi
                })
            })
            .count();
        assert!(passes >= 3, "{passes}");
        // and exactly at the crossing of one such height, two touches
        let [a, b] = plan.moves()[0];
        let h = slide + RADIUS + GAP * ((a[1].min(b[1]) - slide - RADIUS) / GAP).ceil();
        let u = (h - a[1]) / (b[1] - a[1]);
        let at = [a[0] + (b[0] - a[0]) * u, h];
        let m = meetings(at, RADIUS, slide);
        assert_eq!(m.iter().filter(|x| x.touch).count(), 2, "{at:?}");
    }

    /// The drop: one line through, crossed twice; the touching placement:
    /// two lines, a single point each, none through; nudged either way,
    /// one line through again; and coins a thousandth of a centimetre
    /// narrower or wider meet those two lines 0 or 4 times there.
    #[test]
    fn the_drop_and_the_touching_placement() {
        let (_, slide, plan) = shown();
        assert_eq!(through(plan.drop, RADIUS, slide), [1]);
        let stop = meetings(plan.stop, RADIUS, slide);
        assert_eq!(
            stop.iter().map(|m| (m.line, m.touch)).collect::<Vec<_>>(),
            [(0, true), (1, true)]
        );
        assert!(through(plan.stop, RADIUS, slide).is_empty());
        assert_eq!(through(plan.nudges[0], RADIUS, slide), [1]);
        assert_eq!(through(plan.nudges[2], RADIUS, slide), [0]);
        assert_eq!(plan.nudges[3], plan.stop);
        assert_eq!(meetings(plan.stop, 0.999, slide).len(), 0);
        assert_eq!(meetings(plan.stop, 1.001, slide).len(), 4);
        // a glide's last frame, a few ulps off the stop, still touches
        let off = [plan.stop[0], plan.stop[1] + 4.0 * f64::EPSILON];
        assert_eq!(meetings(off, RADIUS, slide).iter().filter(|m| m.touch).count(), 2);
    }

    /// The roll: along the line at 2.45 under the line at 4.45, touching
    /// both all the way, one full turn of the rim, 6.28 cm, inside the
    /// card's lines.
    #[test]
    fn the_roll_lays_the_rim_along_a_line() {
        let (_, slide, plan) = shown();
        let [a, b] = [plan.roll_from, plan.roll_to];
        assert_eq!(a[1], b[1]);
        assert!((a[1] - RADIUS - (slide + GAP)).abs() < 1e-12 && (a[1] + RADIUS - (slide + 2.0 * GAP)).abs() < 1e-12);
        assert_eq!(facts::fixed(b[0] - a[0], 2), "6.28");
        assert!(((b[0] - a[0]) / RADIUS - TAU).abs() < 1e-12);
        for i in 0..=100 {
            let c = [a[0] + (b[0] - a[0]) * i as f64 / 100.0, a[1]];
            assert!(meetings(c, RADIUS, slide).iter().all(|m| m.touch));
        }
        let edge = model::CARD_W / 2.0 - sheet::LINE_INSET;
        assert!(a[0] - RADIUS > -edge && b[0] + RADIUS < edge);
    }

    /// The words on screen say what the facts and the model say.
    #[test]
    fn the_screen_words_are_built_from_the_facts() {
        let (_, slide, plan) = shown();
        let w = words(slide, &plan).unwrap();
        assert_eq!(w.width, "2 cm");
        assert_eq!(w.rim, "6.28 cm");
        assert_eq!(w.rim_tex, r"\text{rim} = \pi \times 2\text{ cm}");
        assert_eq!(w.rows[0], r"2\text{ crossings} \leftrightarrow 6.28\text{ cm}");
        assert_eq!(
            w.rows[1],
            r"1\text{ crossing} \leftrightarrow \pi\text{ cm} \approx 3.14\text{ cm}"
        );
    }

    /// A plan the voice does not describe is refused: a drop on a touching
    /// placement (05_2 says one line, crossed twice), and a touching
    /// placement 0.05 cm off (05_4 says two lines, a single point each).
    /// No placement of this coin breaks "always two"; the twentieths test
    /// shows coins a thousandth narrower and wider that do.
    #[test]
    fn a_plan_the_voice_does_not_describe_is_refused() {
        let (_, slide, mut plan) = shown();
        plan.drop = plan.stop;
        assert!(words(slide, &plan).is_err());
        let (_, slide, mut plan) = shown();
        plan.stop = [plan.stop[0], plan.stop[1] + 0.05];
        assert!(words(slide, &plan).is_err());
    }

    /// The true length stays sealed: no word on screen gives it away.
    #[test]
    fn the_true_length_is_not_on_screen() {
        let f = facts::facts().unwrap();
        let (_, slide, plan) = shown();
        let w = words(slide, &plan).unwrap();
        let on_screen = [
            w.width,
            w.rim_tex,
            w.rim,
            w.rows[0].clone(),
            w.rows[1].clone(),
            CROSSINGS.into(),
            QUESTION.into(),
            placement::SEALED.into(),
        ];
        for key in ["coast.length", "coast.length_2dp", "coast.length_3dp"] {
            let length = f.get(key).unwrap();
            assert!(on_screen.iter().all(|t| !t.contains(length)), "{length}");
        }
    }

    /// The coin keeps to the card and its meeting points to the drawn
    /// lines all the way; it stays clear of the count, the envelope and
    /// (at the drop) the think ring; where it stops it is clear of the
    /// coast; the roll's label clears the rolled coin; the rows sit in the
    /// band under the coast.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let (coast, slide, plan) = shown();
        let edge = [CARD_W / 2.0 - sheet::LINE_INSET, CARD_H / 2.0];
        // the count's room ("crossings" at 30 points taken as at most 1.8
        // wide) and the envelope's ("true length" under it at most 1.6),
        // frame
        let count_box = [
            COUNT_RIGHT[0] - 0.5,
            COUNT_RIGHT[0] + WORD_GAP + 1.8,
            TOP_Y - 0.3,
            TOP_Y + 0.3,
        ];
        let envelope_right = placement::ENVELOPE_AT[0] + 0.8_f64.max(placement::ENVELOPE[0] / 2.0);
        for c in on_the_way(&plan) {
            assert!(c[0].abs() + RADIUS < edge[0] && c[1].abs() + RADIUS < edge[1], "{c:?}");
            for m in meetings(c, RADIUS, slide) {
                assert!(m.at[0].abs() < edge[0] && m.at[1].abs() < edge[1]);
            }
            let f = card_point(c);
            let r = CARD_UNIT * RADIUS;
            let gap_x = (count_box[0] - f[0]).max(f[0] - count_box[1]).max(0.0);
            let gap_y = (count_box[2] - f[1]).max(f[1] - count_box[3]).max(0.0);
            assert!(gap_x.hypot(gap_y) > r + 0.1, "the coin at {c:?} meets the count");
            assert!(
                f[0] - r > envelope_right + 0.2 || f[1] + r < placement::SEALED_Y - 0.2,
                "{c:?}"
            );
        }
        let d = card_point(plan.drop);
        assert!(dist([d[0], d[1]], [THINK_AT[0], THINK_AT[1]]) > CARD_UNIT * RADIUS + THINK_R + 0.3);
        // "pause for more time" under the ring, at 18 points taken as at
        // most 2.4 wide, is near the line the dropped coin's points are on
        for m in meetings(plan.drop, RADIUS, slide) {
            assert!(
                card_point(m.at)[0] < THINK_AT[0] - 1.2 - 0.3,
                "{m:?} meets the think note"
            );
        }
        let pts = along(&coast);
        let mut stops = vec![
            plan.drop,
            plan.stop,
            plan.nudges[0],
            plan.nudges[2],
            plan.roll_from,
            plan.roll_to,
        ];
        stops.extend(plan.landings);
        for c in stops {
            let near = pts.iter().map(|&p| dist(p, c)).fold(f64::INFINITY, f64::min);
            assert!(
                near - RADIUS > 0.2,
                "the coin at {c:?} comes {} cm from the coast",
                near - RADIUS
            );
        }
        let label = [
            (plan.roll_from[0] + plan.roll_to[0]) / 2.0,
            plan.roll_from[1] - RADIUS + ROLL_LABEL_UP,
        ];
        assert!(label[0] + 0.8 < plan.roll_to[0] - RADIUS && label[1] < slide + 2.0 * GAP - 0.3);
        let lowest = pts.iter().map(|&p| card_point(p)[1]).fold(f64::INFINITY, f64::min);
        let line = CARD_CENTRE[1] + CARD_UNIT * (slide - 2.0 * GAP);
        for at in [RIM_AT, ROWS_AT[0], ROWS_AT[1]] {
            assert!(
                at[1] < lowest - 0.35 && at[1] > line + 0.25,
                "{at:?} between {line} and {lowest}"
            );
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("05_1", "Take a coin"),
            ("05_1", "two centimetres across"),
            ("05_1", "exactly as wide as the gap"),
            ("05_2", "Drop it on the sheet"),
            ("05_2", "crosses this line twice"),
            ("05_3", "Always two"),
            ("05_3", "Turning changes nothing"),
            ("05_4", "when the coin fits exactly"),
            ("05_4", "a single point"),
            ("05_5", "Roll the coin one full turn"),
            ("05_5", "six point two eight"),
            ("05_6", "each crossing is worth"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }
}
