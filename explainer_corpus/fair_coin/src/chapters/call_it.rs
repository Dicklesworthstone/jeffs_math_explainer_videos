//! Chapter 01, call it: the coin alone, then tossed again and again, the
//! first results dropping into a row and the rest of its thousand tosses
//! pouring into a grid of ticks over a bar that leans far toward heads,
//! Alice and Bob either side, and under it the level bar of the game they
//! will play (01_1); the title (01_2); one toss, called, and the thousand
//! tosses' bar with the caller's wins marked (01_3); why a call is not fair
//! (01_4); what the model coin stands for (01_5); and the question chapter
//! 02 answers (01_6). Every toss shown is one of run 1
//! (`facts::call_it_run`): 01_1 shows the whole run in order, and 01_3's one
//! toss is the run's first toss again. The level bar is run 2's tally, as at
//! 02_9.

use std::cell::RefCell;
use std::rc::Rc;

use super::draw::predict_pause;
use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, lagged, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    CAPTION_Y, SplitBar, alice, bob, caption, die_face, disc, face_color, fade_in_all, fail, flat, group, heads,
    line_style, panel, say, tag, tails, thrown, wait_until,
};
use super::glyphs;
use crate::facts::{call_it_run, cited, trick_draws, trick_run};
use crate::model::{Face, bent, heads_first_wins};

pub struct CallIt {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const MODEL_COIN: &str = "a model coin (one we made up): heads";
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const SAME_COIN: &str = "the same coin";
const TOSSES: &str = "tosses";
const CALLER_WINS: &str = "the caller's wins";
const OTHER_KNOWS: &str = "what does the other one know?";
const REAL_COINS: &str = "real coins:";
const WEIGHTING: &str = "weighting can't make a tossed-and-caught coin favour a side by much";
const GELMAN_NOLAN: &str = "Gelman and Nolan";
const STANDS_FOR: &str = "Our coin stands for the others";
const SPUN_COIN: &str = "a spun coin";
const LOADED_DIE: &str = "a loaded die";
const NOISE_A: &str = "and, far more mildly,";
const NOISE_B: &str = "the noise inside a computer";
const QUESTION_A: &str = "How could they make the bet fair,";
const QUESTION_B: &str = "whatever either of them knows about the coin?";

/// The dark ink of a letter on a coloured disc (as `draw::lettered_disc`).
const DARK: &str = "#0E1116";

// where things sit
/// The opening shot: the coin alone, large and centred.
const OPEN_AT: Vec3 = [0.0, 0.25, 0.0];
const OPEN_R: f64 = 1.3;
/// Where the coin is tossed over its row and grid (01_1).
const TOSS_AT: Vec3 = [0.0, 2.75, 0.0];
const TOSS_R: f64 = 0.5;
/// On the title card.
const TITLE_AT: Vec3 = [0.0, 1.55, 0.0];
const TITLE_R: f64 = 0.62;
/// Above the three pictures of 01_5.
const TOP_AT: Vec3 = [0.0, 2.8, 0.0];
const TOP_R: f64 = 0.5;
/// Between the two players (01_3, 01_4, 01_6).
const MID_AT: Vec3 = [0.0, 0.6, 0.0];
const MID_R: f64 = 0.95;

/// The row the first results drop into: how many, and where (the grid's
/// first row, which they shrink into).
const ROW_N: usize = 20;
const ROW_Y: f64 = GRID_TOP;
const ROW_STEP: f64 = 0.4;
const DISC_R: f64 = 0.16;

/// Both bars: left end, width, height.
const BAR_LEFT: f64 = -4.0;
const BAR_W: f64 = 8.0;
const BAR_H: f64 = 0.4;
/// 01_1: the coin's own tally under the grid, its words and counts, the
/// count it settles at, and the game's bar under them.
const COIN_BAR_Y: f64 = -1.2;
const COUNT_Y: f64 = -2.18;
const GAME_BAR_Y: f64 = -2.85;
/// 01_3 and 01_4: the thousand tosses' bar under the coin.
const CALL_BAR_Y: f64 = -1.6;
/// Words and counts sit this far under their bar's middle.
const WORDS_BELOW: f64 = 0.52;

/// The grid of ticks, one for each toss of the run, read like a page.
const GRID_COLS: usize = 50;
const GRID_DX: f64 = 0.16;
const GRID_DY: f64 = 0.13;
const GRID_TOP: f64 = 1.75;
const TICK: (f64, f64) = (0.11, 0.09);
/// How many tosses each lit batch of the pour holds, and how long the pour
/// of the rest of the run takes.
const POUR_BATCH: usize = 25;
const POUR_S: f64 = 2.2;

/// The players stand this far either side of the middle.
const PLAYER_X: f64 = 5.75;
const HEAD_Y: f64 = 0.45;

fn other(f: Face) -> Face {
    match f {
        Face::H => Face::T,
        Face::T => Face::H,
    }
}

fn args(rt: f64) -> AnimateArgs {
    AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    }
}

/// Fades for several detached mobjects, to play beside other moves.
fn fading_in(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(rt))?);
    }
    Ok(out)
}

/// Fades out for several mobjects (updaters cleared first), to play beside
/// other moves.
fn fading_out(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a.rt(rt))?);
    }
    Ok(out)
}

/// One picture for another in one move: `ins` start when `outs` are half
/// gone, so the two are never both at full strength.
fn swap(stage: &mut Stage<'_>, outs: Vec<Box<dyn Animation>>, ins: Vec<Box<dyn Animation>>) -> fmn::Result<()> {
    let outs = lagged(stage, outs, 0.0)?;
    let ins = lagged(stage, ins, 0.0)?;
    let both = lagged(stage, vec![Box::new(outs), Box::new(ins)], 0.5)?;
    stage.play(both)?;
    Ok(())
}

// ----------------------------------------------------------------- the coin

/// What the scene tells the coin's driver, and what the driver tells back.
struct Driver {
    /// The tosses the coin makes on its own clock: when each begins
    /// (seconds after the clock starts) and how long it flies.
    schedule: Vec<(f64, f64)>,
    clock: f64,
    running: bool,
    /// The tosses already made when the clock started.
    base: f64,
    /// Whether a landed result may drop into the row.
    row_open: bool,
    /// The share of heads among the tosses so far, eased.
    share: f64,
    /// The share trackers of the bars that follow it.
    bars: Vec<Mob>,
}

/// Tosses completed at `clock` (a toss in flight is the fraction), and
/// whether the schedule has run out.
fn scheduled(schedule: &[(f64, f64)], clock: f64) -> (f64, bool) {
    for (i, &(start, flight)) in schedule.iter().enumerate() {
        if clock < start {
            return (i as f64, false);
        }
        if clock < start + flight {
            return (i as f64 + (clock - start) / flight, false);
        }
    }
    (schedule.len() as f64, true)
}

/// How many tosses are over when the tracker reads `n`.
fn completed(n: f64, total: usize) -> usize {
    ((n + 1e-9).floor().max(0.0) as usize).min(total)
}

/// The coin: a large lettered disc that tosses itself, one result of the
/// seeded run after another. Everything it shows is a function of `n`, the
/// number of tosses so far: the disc turns over in flight and lands on the
/// run's next face, a landed result drops into the row beneath, and the
/// bars' shares follow the tally. One updater draws it; the scene moves it
/// by its trackers, and moves `n` either through the driver's schedule
/// (tosses that go on while other things are said) or by a tween.
struct Coin {
    /// Tosses so far.
    n: Mob,
    x: Mob,
    y: Mob,
    r: Mob,
    /// The row's discs, one for each of the run's first results.
    row: Vec<Mob>,
    /// The disc, its two letters and the mobject that carries the updater.
    own: Vec<Mob>,
    driver: Rc<RefCell<Driver>>,
}

/// How high a toss rises, in coin radii; how tall the letter is; how long a
/// result takes to drop into the row; how fast the bars follow the tally.
const LIFT: f64 = 0.8;
const LETTER: f64 = 0.85;
const DROP_S: f64 = 0.32;
const SHARE_EASE_S: f64 = 0.16;
/// Past this many tosses a second the single turns can't be seen: the coin
/// is drawn spinning steadily instead, at this many half-turns a second.
const BLUR_RATE: f64 = 14.0;
const BLUR_HALF_TURNS: f64 = 7.0;

impl Coin {
    fn build(kit: &Kit, stage: &mut Stage<'_>, run: &[Face], at: Vec3, r: f64, slots: &[Vec3]) -> fmn::Result<Self> {
        if run.len() < slots.len() {
            return Err(fail(format!(
                "the run has {} tosses, the row {}",
                run.len(),
                slots.len()
            )));
        }
        let total = run.len();
        let n = stage.add_value_tracker(0.0);
        let x = stage.add_value_tracker(at[0]);
        let y = stage.add_value_tracker(at[1]);
        let rad = stage.add_value_tracker(r);
        // the row's discs first, so each comes out from behind the coin
        let mut discs = Vec::new();
        for (&f, &slot) in run.iter().zip(slots) {
            let d = stage.add(disc(f, at, DISC_R).with_style(flat(face_color(f), 0.0)))?;
            discs.push((d, slot));
        }
        let body = Ellipse::new()
            .width(2.0 * r)
            .height(2.0 * r)
            .style(flat(heads(), 1.0))
            .build()
            .moved_to(at);
        let body = stage.add(body)?;
        let faces = [Face::H, Face::T];
        let mut letters = Vec::new();
        let mut aspect = Vec::new();
        for f in faces {
            let l = kit.text(stage, &f.letter().to_string(), 60.0, hex(DARK), at)?;
            aspect.push(stage.get_width(l) / stage.get_height(l).max(1e-6));
            stage.add_to_scene(l)?;
            letters.push(l);
        }
        let anchor = stage.add(VMobject::new())?;
        let driver = Rc::new(RefCell::new(Driver {
            schedule: Vec::new(),
            clock: 0.0,
            running: false,
            base: 0.0,
            row_open: false,
            share: 0.5,
            bars: Vec::new(),
        }));
        let run: Vec<Face> = run.to_vec();
        let prefix = heads_so_far(&run);
        let shared = Rc::clone(&driver);
        let row: Vec<Mob> = discs.iter().map(|&(d, _)| d).collect();
        let own = vec![body, letters[0], letters[1], anchor];
        let mut drop = vec![0.0_f64; discs.len()];
        let (mut last_n, mut spin) = (0.0_f64, 0.0_f64);
        let mut drawn: Option<(Vec3, f64, f64, Face, bool)> = None;
        stage.add_dt_updater(
            anchor,
            move |s, _, dt| {
                let mut d = shared.borrow_mut();
                if d.running {
                    d.clock += dt;
                    let (value, over) = scheduled(&d.schedule, d.clock);
                    let _ = s.set_tracker_value(n, d.base + value);
                    d.running = !over;
                }
                let read = |m: Mob, s: &fmn::mobject::Stage| s.tracker_value(m).unwrap_or(0.0);
                let now = read(n, s).clamp(0.0, total as f64);
                let (cx, cy, r) = (read(x, s), read(y, s), read(rad, s).max(0.01));
                let done = completed(now, total);
                let phase = (now - done as f64).max(0.0);
                let rate = if dt > 0.0 { (now - last_n).abs() / dt } else { 0.0 };
                last_n = now;
                let resting = |k: usize| if k == 0 { Face::H } else { run[k - 1] };
                // the face showing, how much of it (1 flat on, 0 edge on),
                // how high the coin is, and whether its letter can be read
                let (face, flat_on, up, lettered) = if rate > BLUR_RATE {
                    spin += dt * BLUR_HALF_TURNS;
                    let c = (PI * spin).cos();
                    (if c >= 0.0 { Face::H } else { Face::T }, c.abs(), 0.35, false)
                } else if phase > 1e-6 && done < total {
                    let (from, to) = (resting(done), run[done]);
                    let turns = if from == to { 4.0 } else { 3.0 };
                    let crossings = (turns * phase + 0.5).floor() as u32;
                    let face = if crossings.is_multiple_of(2) { from } else { other(from) };
                    (
                        face,
                        (PI * turns * phase).cos().abs(),
                        4.0 * phase * (1.0 - phase),
                        true,
                    )
                } else {
                    (resting(done), 1.0, 0.0, true)
                };
                let at = v(cx, cy + LIFT * r * up);
                let state = (at, r, flat_on, face, lettered);
                if drawn != Some(state) {
                    let thin = flat_on.max(0.06);
                    let shape = Ellipse::new()
                        .width(2.0 * r)
                        .height(2.0 * r * thin)
                        .build()
                        .moved_to(at);
                    let _ = s.set_points(body, shape.points());
                    s.set_fill(body, Some(face_color(face)), Some(1.0), None, false);
                    for (i, &l) in letters.iter().enumerate() {
                        let on = lettered && faces[i] == face;
                        let o = if on {
                            ((flat_on - 0.2) / 0.25).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        s.set_width(l, LETTER * r * aspect[i], true);
                        s.set_height(l, LETTER * r * thin, true);
                        s.move_to(l, at, ORIGIN);
                        s.set_opacity(l, o, true);
                    }
                    drawn = Some(state);
                }
                // a landed result comes out from behind the coin and drops
                // into its place in the row
                if d.row_open {
                    for (j, &(m, slot)) in discs.iter().enumerate() {
                        if j >= done || drop[j] >= 1.0 {
                            continue;
                        }
                        drop[j] = (drop[j] + dt / DROP_S).min(1.0);
                        let q = fmn::core::rate::smooth(drop[j]);
                        let p = v(cx + (slot[0] - cx) * q, cy + (slot[1] - cy) * q);
                        s.move_to(m, p, ORIGIN);
                        s.set_fill(m, None, Some((drop[j] * 4.0).min(1.0)), None, false);
                    }
                }
                // the tally the bars show
                if done > 0 {
                    let target = prefix[done] as f64 / done as f64;
                    let before = d.share;
                    d.share = if (target - before).abs() < 1e-4 {
                        target
                    } else {
                        before + (target - before) * (1.0 - (-dt / SHARE_EASE_S).exp())
                    };
                    if d.share != before {
                        for &b in &d.bars {
                            let _ = s.set_tracker_value(b, d.share);
                        }
                    }
                }
            },
            true,
        )?;
        Ok(Self {
            n,
            x,
            y,
            r: rad,
            row,
            own,
            driver,
        })
    }

    /// Start the driver's clock on `schedule`: more tosses, after those
    /// already made.
    fn start(&self, stage: &Stage<'_>, schedule: Vec<(f64, f64)>) {
        let mut d = self.driver.borrow_mut();
        d.base = (stage.tracker_value(self.n).unwrap_or(0.0) + 1e-9).floor();
        d.schedule = schedule;
        d.clock = 0.0;
        d.running = true;
    }

    /// Moves of the coin to `at` with radius `r`, to play beside others.
    fn to(&self, stage: &mut Stage<'_>, at: Vec3, r: f64, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for (t, value) in [(self.x, at[0]), (self.y, at[1]), (self.r, r)] {
            out.push(stage.prepare(tween(t, value, rt)?)?);
        }
        Ok(out)
    }

    /// The bar's share follows the tally from now on (call it once the bar
    /// has faded in: a fade and a moving split do not mix).
    fn feed(&self, stage: &mut Stage<'_>, bar: &SplitBar) -> fmn::Result<()> {
        let mut d = self.driver.borrow_mut();
        stage.set_tracker_value(bar.share, d.share)?;
        d.bars.push(bar.share);
        Ok(())
    }

    /// No bar follows the tally any more.
    fn stop_feeding(&self) {
        self.driver.borrow_mut().bars.clear();
    }

    fn share(&self) -> f64 {
        self.driver.borrow().share
    }
}

/// How many of the first k tosses were heads, for k = 0 to the run's length.
fn heads_so_far(run: &[Face]) -> Vec<usize> {
    let mut out = vec![0];
    for &f in run {
        out.push(out[out.len() - 1] + usize::from(f == Face::H));
    }
    out
}

/// The hook's first toss: one slow turn of the big coin (when it begins,
/// how long it flies).
const FIRST_TOSS: (f64, f64) = (0.15, 1.1);

/// The rest of the hook's tosses, once the coin has stepped up over its
/// row: one after another, quicker and quicker.
fn quick_tosses() -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let (mut t, mut flight) = (0.1, 0.44);
    for _ in 1..ROW_N {
        out.push((t, flight));
        t += flight;
        flight = (flight * 0.88_f64).max(0.23);
    }
    out
}

// -------------------------------------------------------------- the players

/// Alice or Bob: a head and shoulders in the player's colour, two eyes, a
/// name. The pupils and the eyes move on their own, so the parts are kept
/// apart (never one group).
struct Player {
    shapes: Vec<Mob>,
    eyes: [Mob; 2],
    pupils: [Mob; 2],
    /// +1 when the other player stands to the right.
    toward: f64,
}

impl Player {
    fn mobs(&self) -> Vec<Mob> {
        let mut out = self.shapes.clone();
        out.extend(self.eyes);
        out.extend(self.pupils);
        out
    }

    /// Narrow the eyes and turn them on the other player.
    fn eye_the_other(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        self.eyes_move(stage, EYE_NARROW, 1.0, rt)
    }

    /// Open the eyes again and look back at the coin (after
    /// [`Player::eye_the_other`]).
    fn look_at_the_coin(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        self.eyes_move(stage, 1.0 / EYE_NARROW, -1.0, rt)
    }

    fn eyes_move(
        &self,
        stage: &mut Stage<'_>,
        stretch: f64,
        way: f64,
        rt: f64,
    ) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..2 {
            let lids = self.eyes[i].animate().set_anim_args(args(rt))?.stretch(stretch, 1)?;
            out.push(stage.prepare(lids)?);
            let look = self.pupils[i]
                .animate()
                .set_anim_args(args(rt))?
                .shift(v(way * 0.035 * self.toward, way * -0.03))?;
            out.push(stage.prepare(look)?);
        }
        Ok(out)
    }
}

/// How far a wary player's eyes close.
const EYE_NARROW: f64 = 0.55;

/// A player at `x`, looking up at the coin. Detached.
fn player(kit: &Kit, stage: &mut Stage<'_>, name: &str, color: Srgb, x: f64) -> fmn::Result<Player> {
    let toward = if x < 0.0 { 1.0 } else { -1.0 };
    let (head_r, shoulders_r) = (0.46, 0.74);
    let base = HEAD_Y - 1.18;
    let mut arc = Vec::new();
    for k in 0..=24 {
        let a = PI * f64::from(k) / 24.0;
        arc.push(v(x + shoulders_r * a.cos(), base + shoulders_r * a.sin()));
    }
    let shoulders = Polygon::new(arc).style(flat(color, 1.0)).build();
    let head = Circle::new()
        .radius(head_r)
        .style(flat(color, 1.0))
        .build()
        .moved_to(v(x, HEAD_Y));
    let name = kit.text(stage, name, 28.0, color, v(x, base - 0.38))?;
    let shapes = vec![stage.arena_mut().add(shoulders), stage.arena_mut().add(head), name];
    let mut eyes = Vec::new();
    let mut pupils = Vec::new();
    for side in [-1.0, 1.0] {
        let c = v(x + 0.17 * side, HEAD_Y + 0.06);
        let white = Ellipse::new()
            .width(0.2)
            .height(0.25)
            .style(flat(Palette::ink(), 1.0))
            .build()
            .moved_to(c);
        eyes.push(stage.arena_mut().add(white));
        let pupil = Dot::new()
            .point(v(c[0] + 0.035 * toward, c[1] + 0.03))
            .radius(0.055)
            .style(flat(hex(DARK), 1.0))
            .build();
        pupils.push(stage.arena_mut().add(pupil));
    }
    Ok(Player {
        shapes,
        eyes: [eyes[0], eyes[1]],
        pupils: [pupils[0], pupils[1]],
        toward,
    })
}

/// Alice on the left, Bob on the right.
fn players(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<[Player; 2]> {
    Ok([
        player(kit, stage, ALICE, alice(), -PLAYER_X)?,
        player(kit, stage, BOB, bob(), PLAYER_X)?,
    ])
}

// ----------------------------------------------------------- bars and ticks

/// A short mark across a bar's middle: where a level bar is cut.
fn half_mark(stage: &mut Stage<'_>, y: f64) -> Mob {
    let x = BAR_LEFT + BAR_W / 2.0;
    let mark = polyline(vec![v(x, y - 0.33), v(x, y + 0.33)])
        .style(line_style(Palette::ink(), 2.5))
        .build();
    stage.arena_mut().add(mark)
}

/// "heads" under the bar's left end and "tails" under its right.
fn bar_words(kit: &Kit, stage: &mut Stage<'_>, y: f64) -> fmn::Result<[Mob; 2]> {
    let h = kit.text_left(stage, HEADS, 26.0, heads(), v(BAR_LEFT, y))?;
    let t = kit.text(stage, TAILS, 26.0, tails(), v(BAR_LEFT + BAR_W, y))?;
    stage.move_to(t, v(BAR_LEFT + BAR_W, y), RIGHT);
    Ok([h, t])
}

/// Where toss `i` sits in the grid.
fn tick_at(i: usize) -> Vec3 {
    let left = -GRID_DX * (GRID_COLS - 1) as f64 / 2.0;
    v(
        left + GRID_DX * (i % GRID_COLS) as f64,
        GRID_TOP - GRID_DY * (i / GRID_COLS) as f64,
    )
}

/// The ticks of tosses `indices`, all one colour, as one mobject. Detached;
/// `None` when there are none.
fn ticks(stage: &mut Stage<'_>, indices: &[usize], color: Srgb) -> fmn::Result<Option<Mob>> {
    if indices.is_empty() {
        return Ok(None);
    }
    let mut marks = Vec::new();
    for &i in indices {
        let r = Rectangle::new()
            .width(TICK.0)
            .height(TICK.1)
            .style(flat(color, 1.0))
            .build()?;
        marks.push(r.moved_to(tick_at(i)));
    }
    Ok(Some(stage.arena_mut().add(v_group(marks))))
}

/// The lit ticks of tosses `from..to`: the heads, then the tails.
fn lit(stage: &mut Stage<'_>, run: &[Face], from: usize, to: usize) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for f in [Face::H, Face::T] {
        let indices: Vec<usize> = (from..to.min(run.len())).filter(|&i| run[i] == f).collect();
        out.extend(ticks(stage, &indices, face_color(f))?);
    }
    Ok(out)
}

/// The run's tosses from `from` on, lit in the grid as the toss count `n`
/// passes them: a batch of [`POUR_BATCH`] shows the share of its tosses
/// already made, so the grid and the tallies always agree. The batches go
/// into the scene unlit; returns them and the anchor that carries the
/// updater (also in the scene).
fn pour(stage: &mut Stage<'_>, run: &[Face], from: usize, n: Mob) -> fmn::Result<(Vec<Mob>, Mob)> {
    let total = run.len();
    let mut batches: Vec<(Mob, f64, f64)> = Vec::new();
    let mut start = from;
    while start < total {
        let end = ((start / POUR_BATCH + 1) * POUR_BATCH).min(total);
        for m in lit(stage, run, start, end)? {
            stage.set_fill(m, None, Some(0.0), None, true);
            stage.add_to_scene(m)?;
            batches.push((m, start as f64, end as f64));
        }
        start = end;
    }
    let anchor = stage.arena_mut().add(VMobject::new());
    let lighting = batches.clone();
    let mut shown = vec![0.0_f64; lighting.len()];
    stage.add_updater(
        anchor,
        move |s, _| {
            let now = s.tracker_value(n).unwrap_or(0.0);
            for (k, &(m, a, b)) in lighting.iter().enumerate() {
                let o = ((now - a) / (b - a)).clamp(0.0, 1.0);
                if o != shown[k] {
                    s.set_fill(m, None, Some(o), None, true);
                    shown[k] = o;
                }
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok((batches.iter().map(|b| b.0).collect(), anchor))
}

/// The two tallies beside a bar's words: the heads and the tails among the
/// tosses made so far (live, so they appear at once). `words` are the
/// bar's "heads" and "tails".
fn tallies(kit: &Kit, stage: &mut Stage<'_>, coin: &Coin, prefix: &[usize], words: [Mob; 2]) -> fmn::Result<[Mob; 2]> {
    let (n, total) = (coin.n, prefix.len() - 1);
    let y = stage.get_center(words[0])[1];
    let counts = prefix.to_vec();
    let heads_x = BAR_LEFT + stage.get_width(words[0]) + 0.2;
    let heads_count = kit.live_value(
        stage,
        move |s| counts[completed(s.tracker_value(n).unwrap_or(0.0), total)] as f64,
        0,
        false,
        30.0,
        Palette::ink(),
        v(heads_x, y),
        LEFT,
    )?;
    let counts = prefix.to_vec();
    let tails_x = BAR_LEFT + BAR_W - stage.get_width(words[1]) - 0.2;
    let tails_count = kit.live_value(
        stage,
        move |s| {
            let done = completed(s.tracker_value(n).unwrap_or(0.0), total);
            (done - counts[done]) as f64
        },
        0,
        false,
        30.0,
        Palette::ink(),
        v(tails_x, y),
        RIGHT,
    )?;
    Ok([heads_count, tails_count])
}

/// The two counts beside a bar's words, as still numbers.
fn counts(kit: &Kit, stage: &mut Stage<'_>, (h, t): (usize, usize), words: [Mob; 2]) -> fmn::Result<[Mob; 2]> {
    let y = stage.get_center(words[0])[1];
    let heads_x = BAR_LEFT + stage.get_width(words[0]) + 0.2;
    let heads_count = kit.text_left(stage, &h.to_string(), 30.0, Palette::ink(), v(heads_x, y))?;
    let tails_x = BAR_LEFT + BAR_W - stage.get_width(words[1]) - 0.2;
    let tails_count = kit.text(stage, &t.to_string(), 30.0, Palette::ink(), v(tails_x, y))?;
    stage.move_to(tails_count, v(tails_x, y), RIGHT);
    Ok([heads_count, tails_count])
}

/// A player's call over their head: the side, in its colour, on a small
/// back. Detached, one group.
fn call_tag(kit: &Kit, stage: &mut Stage<'_>, f: Face, at: Vec3) -> fmn::Result<Mob> {
    let word = match f {
        Face::H => HEADS,
        Face::T => TAILS,
    };
    let t = kit.text(stage, word, 28.0, face_color(f), at)?;
    let (w, h) = (stage.get_width(t), stage.get_height(t));
    let back = Rectangle::new()
        .width(w + 0.34)
        .height(h + 0.26)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(face_color(f), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group(stage, &[back, t])
}

// ------------------------------------------------------- the three pictures

/// A small picture's card, centred at `at`.
fn picture_card(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(4.1)
        .height(2.9)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(back))
}

/// A coin spinning on its edge: the disc, the table under it, and the
/// mobject whose updater turns it (add that one to the scene once the disc
/// has faded in). Detached.
fn spun_coin(stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<[Mob; 3]> {
    let body = Ellipse::new()
        .width(2.0 * r)
        .height(2.0 * r)
        .style(flat(heads(), 1.0))
        .build()
        .moved_to(at);
    let body = stage.arena_mut().add(body);
    let table = polyline(vec![v(at[0] - 1.2, at[1] - r - 0.04), v(at[0] + 1.2, at[1] - r - 0.04)])
        .style(line_style(Palette::dim(), 3.0))
        .build();
    let table = stage.arena_mut().add(table);
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut t = 0.0_f64;
    stage.add_dt_updater(
        anchor,
        move |s, _, dt| {
            t += dt;
            // a turn and a half a second, slowing a little as it goes
            let c = (PI * 3.0 * t / (1.0 + 0.04 * t)).cos();
            let shape = Ellipse::new()
                .width(2.0 * r * c.abs().max(0.07))
                .height(2.0 * r)
                .build()
                .moved_to(at);
            let _ = s.set_points(body, shape.points());
            let face = if c >= 0.0 { Face::H } else { Face::T };
            s.set_fill(body, Some(face_color(face)), None, None, false);
        },
        false,
    )?;
    Ok([body, table, anchor])
}

/// Raw noise as a picture: the uniform draws behind run 2, in the order
/// they were made, as heights across a band `size` centred at `at` (chapter
/// 11 reads the same draws off as ones and zeros). Detached.
fn noise_trace(stage: &mut Stage<'_>, at: Vec3, size: (f64, f64), points: usize) -> Mob {
    let draws: Vec<f64> = trick_draws()
        .into_iter()
        .flatten()
        .flat_map(|(a, b)| [a, b])
        .take(points)
        .collect();
    let step = size.0 / (draws.len().max(2) - 1) as f64;
    let line: Vec<Vec3> = draws
        .iter()
        .enumerate()
        .map(|(i, u)| v(at[0] - size.0 / 2.0 + step * i as f64, at[1] + size.1 * (0.5 - u)))
        .collect();
    let trace = polyline(line).style(line_style(Palette::ink(), 2.5)).build();
    stage.arena_mut().add(trace)
}

/// The prediction question on a card centred at `at`: the card with its
/// first line (one group), and its second line. Detached.
fn question_card(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<[Mob; 2]> {
    let a = kit.text(stage, QUESTION_A, 32.0, Palette::ink(), v(at[0], at[1] + 0.26))?;
    let b = kit.text(stage, QUESTION_B, 32.0, Palette::ink(), v(at[0], at[1] - 0.26))?;
    let w = stage.get_width(a).max(stage.get_width(b));
    let back = Rectangle::new()
        .width(w + 0.8)
        .height(1.4)
        .corner_radius(0.14)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    Ok([group(stage, &[back, a])?, b])
}

impl SceneConstruct for CallIt {
    fn name(&self) -> &str {
        "01_call_it"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let run = call_it_run();
        let total = run.len();
        let prefix = heads_so_far(&run);
        let row_left = -ROW_STEP * (ROW_N - 1) as f64 / 2.0;
        let slots: Vec<Vec3> = (0..ROW_N).map(|i| v(row_left + ROW_STEP * i as f64, ROW_Y)).collect();

        // the opening shot: the coin alone, large and centred, from the
        // first frame
        let coin = Coin::build(&kit, stage, &run, OPEN_AT, OPEN_R, &slots)?;

        // 01_1: it isn't fair, and the whole run shows it; Alice and Bob;
        // and yet, with the same coin
        let line = say(&kit, stage, "01_1")?;
        coin.start(stage, vec![FIRST_TOSS]);
        let model = format!("{MODEL_COIN} {}%", bent().percent(0));
        let label = tag(&kit, stage, &model, ORIGIN, 24.0)?;
        let beside = |stage: &mut Stage<'_>, at: Vec3, r: f64| {
            let w = stage.get_width(label);
            stage.move_to(label, v(at[0] + r + 0.3 + w / 2.0, at[1]), ORIGIN);
        };
        beside(stage, TOSS_AT, TOSS_R);
        // it steps up over its row, and says what it is
        line.wait_for(stage, "It lands heads")?;
        wait_until(stage, line.start + FIRST_TOSS.0 + FIRST_TOSS.1 + 0.05)?;
        let mut step = coin.to(stage, TOSS_AT, TOSS_R, 0.6)?;
        step.extend(fading_in(stage, &[label], 0.6)?);
        stage.play_prepared(step)?;
        // and is tossed again and again, each result dropping into the row
        coin.driver.borrow_mut().row_open = true;
        let quick = quick_tosses();
        let few = quick[2].0 + quick[2].1 + 0.05;
        let row_done = quick.last().map_or(0.0, |&(t, f)| t + f) + DROP_S + 0.05;
        let since = stage.scene().time().to_f64();
        coin.start(stage, quick);
        // the tally, once there are a few results to count: the bar, and
        // the heads and tails that have landed so far
        wait_until(stage, since + few)?;
        let coin_bar = SplitBar::build(
            stage,
            v(BAR_LEFT, COIN_BAR_Y),
            (BAR_W, BAR_H),
            (heads(), tails()),
            coin.share(),
        )?;
        let mark = half_mark(stage, COIN_BAR_Y);
        let words = bar_words(&kit, stage, COIN_BAR_Y - WORDS_BELOW)?;
        let mut ins = coin_bar.mobs().to_vec();
        ins.push(mark);
        ins.extend(words);
        fade_in_all(stage, &ins, 0.4)?;
        coin.feed(stage, &coin_bar)?;
        tallies(&kit, stage, &coin, &prefix, words)?;
        // Alice and Bob, either side
        let [a, b] = players(&kit, stage)?;
        line.wait_for(stage, "Alice and Bob")?;
        fade_in_all(stage, &a.mobs(), 0.4)?;
        fade_in_all(stage, &b.mobs(), 0.4)?;
        // each eyeing the other
        line.wait_for(stage, "and each suspects")?;
        let mut eyeing = a.eye_the_other(stage, 0.4)?;
        eyeing.extend(b.eye_the_other(stage, 0.4)?);
        stage.play_prepared(eyeing)?;
        // the row shrinks into the first ticks of a grid for the whole run
        wait_until(stage, since + row_done)?;
        let mut empty = Vec::new();
        for row in 0..total.div_ceil(GRID_COLS) {
            let indices: Vec<usize> = (row * GRID_COLS..((row + 1) * GRID_COLS).min(total))
                .filter(|&i| i >= ROW_N)
                .collect();
            empty.extend(ticks(stage, &indices, thrown())?);
        }
        stage.scene_mut().bring_to_front(&coin.row)?;
        let mut shrink = fading_in(stage, &empty, 0.5)?;
        for (i, &d) in coin.row.iter().enumerate() {
            let tick = Rectangle::new()
                .width(TICK.0)
                .height(TICK.1)
                .style(flat(face_color(run[i]), 1.0))
                .build()?
                .moved_to(tick_at(i));
            let tick = stage.arena_mut().add(tick);
            shrink.push(stage.prepare(Transform::new(d, tick).rt(0.5))?);
        }
        stage.play_prepared(shrink)?;
        // the rest of the thousand pour in; the tallies count what has landed
        pour(stage, &run, ROW_N, coin.n)?;
        let pouring = coin
            .n
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(POUR_S),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(total as f64)?;
        play!(stage; pouring);
        // what the bar has settled at, said in words
        let settled = format!("{} {TOSSES}: {} {HEADS}", thousands(total as u64), prefix[total]);
        let settled = kit.text(stage, &settled, 26.0, Palette::dim(), v(0.0, COUNT_Y))?;
        fade_in_all(stage, &[settled], 0.4)?;
        // the payoff as a paradox: a second bar, level, in their two colours
        line.wait_for(stage, "They can still do it")?;
        let games = trick_run();
        let level = heads_first_wins(&games) as f64 / games.len().max(1) as f64;
        let game_bar = SplitBar::build(stage, v(BAR_LEFT, GAME_BAR_Y), (BAR_W, BAR_H), (alice(), bob()), level)?;
        let game_mark = half_mark(stage, GAME_BAR_Y);
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), game_bar.parts[0], RIGHT, None)?.rt(1.0),
            fmn::animation::grow_from_edge(stage.arena_mut(), game_bar.parts[1], LEFT, None)?.rt(1.0),
            fade_in(stage.arena_mut(), game_mark, ORIGIN, 1.0)?.rt(1.0),
        );
        line.wait_for(stage, "using nothing but this coin")?;
        let same = kit.text(stage, SAME_COIN, 28.0, Palette::ink(), v(0.0, GAME_BAR_Y - 0.55))?;
        fade_in_all(stage, &[same], 0.5)?;
        wait_until(stage, line.end() + 0.1)?;

        // 01_2: the title; the coin stays, everything else leaves
        let (top, rest) = glyphs::TITLE
            .split_once(" Out ")
            .ok_or_else(|| fail("the title has no \" Out \" to break at".into()))?;
        let rest = format!("Out {rest}");
        let mut title = Vec::new();
        for (words, y) in [(top, 0.0), (rest.as_str(), -1.0)] {
            title.push(kit.text_styled(stage, words, 70.0, Palette::hero(), v(0.0, y), true, false)?);
        }
        let leaving: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| !coin.own.contains(m))
            .collect();
        let mut wipe = fading_out(stage, &leaving, 0.5)?;
        wipe.extend(coin.to(stage, TITLE_AT, TITLE_R, 0.6)?);
        let mut writing: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &title {
            let w = write(stage.arena(), t);
            writing.push(stage.prepare(w.rt(0.8))?);
        }
        swap(stage, wipe, writing)?;
        coin.stop_feeding();
        kit.listen(stage, 3.0, "01_call_it:01_2")?;

        // 01_3: call it; one toss, called; whoever gets heads wins four bets
        // in five
        let line = say(&kit, stage, "01_3")?;
        let [a, b] = players(&kit, stage)?;
        let mut ins = a.mobs();
        ins.extend(b.mobs());
        let leave = fading_out(stage, &title, 0.4)?;
        let mut back = coin.to(stage, MID_AT, MID_R, 0.7)?;
        back.extend(fading_in(stage, &ins, 0.7)?);
        swap(stage, leave, back)?;
        // Alice calls heads
        line.wait_for(stage, "call it")?;
        let call = call_tag(&kit, stage, Face::H, v(-PLAYER_X, HEAD_Y + 0.95))?;
        play!(stage; fade_in(stage.arena_mut(), call, v(0.0, -0.15), 1.0)?.rt(0.4));
        // one toss: the run's first toss again (the coin rests on heads
        // before it, and the run's last toss left it on heads too, so
        // going back to the start shows nothing)
        line.wait_for(stage, "One toss")?;
        if run.last() != Some(&Face::H) {
            return Err(fail("run 1 no longer ends on heads: 01_3's toss would jump".into()));
        }
        stage.set_tracker_value(coin.n, 0.0)?;
        play!(stage; tween(coin.n, 1.0, FIRST_TOSS.1)?);
        // whoever gets heads: the thousand tosses' bar, its two counts, and
        // the heads end marked as the caller's wins
        line.wait_for(stage, "But whoever gets heads")?;
        let heads_n = prefix[total];
        let share = heads_n as f64 / total as f64;
        let bar = SplitBar::build(
            stage,
            v(BAR_LEFT, CALL_BAR_Y),
            (BAR_W, BAR_H),
            (heads(), tails()),
            share,
        )?;
        let mark = half_mark(stage, CALL_BAR_Y);
        let words = bar_words(&kit, stage, CALL_BAR_Y - WORDS_BELOW)?;
        let numbers = counts(&kit, stage, (heads_n, total - heads_n), words)?;
        let cap = caption(&kit, stage, &format!("{} {TOSSES}", thousands(total as u64)))?;
        let mut ins = bar.mobs().to_vec();
        ins.push(mark);
        ins.extend(words);
        ins.extend(numbers);
        ins.push(cap);
        fade_in_all(stage, &ins, 0.5)?;
        let (y0, cut) = (CALL_BAR_Y + BAR_H / 2.0 + 0.28, BAR_LEFT + BAR_W * share);
        let bracket = polyline(vec![
            v(BAR_LEFT, y0 - 0.12),
            v(BAR_LEFT, y0),
            v(cut, y0),
            v(cut, y0 - 0.12),
        ])
        .style(line_style(Palette::ink(), 2.5))
        .build();
        let bracket = stage.arena_mut().add(bracket);
        let wins = kit.text(
            stage,
            CALLER_WINS,
            26.0,
            Palette::ink(),
            v((BAR_LEFT + cut) / 2.0, y0 + 0.26),
        )?;
        fade_in_all(stage, &[bracket, wins], 0.4)?;

        // 01_4: a call is fair only if the caller knows nothing; and neither
        // can check what the other knows
        let line = say(&kit, stage, "01_4")?;
        // who ends up with the winning end? it flickers between their colours
        line.wait_for(stage, "has no idea which side")?;
        for color in [alice(), bob(), alice(), bob(), heads()] {
            let turn = bar.parts[0]
                .animate()
                .set_anim_args(args(0.42))?
                .set_fill(Some(color), Some(1.0))?;
            play!(stage; turn);
        }
        // each looks at the other: what does the other one know?
        line.wait_for(stage, "And neither of them can check")?;
        let other = kit.text(
            stage,
            OTHER_KNOWS,
            28.0,
            Palette::ink(),
            v(0.0, MID_AT[1] + MID_R + 0.55),
        )?;
        let mut asks = Vec::new();
        for p in [&a, &b] {
            let x = if p.toward > 0.0 { -PLAYER_X } else { PLAYER_X };
            asks.push(kit.text(stage, "?", 48.0, Palette::hero(), v(x + 0.5 * p.toward, HEAD_Y + 0.95))?);
        }
        let mut eyeing = a.eye_the_other(stage, 0.4)?;
        eyeing.extend(b.eye_the_other(stage, 0.4)?);
        eyeing.extend(fading_out(stage, &[call], 0.3)?);
        eyeing.extend(fading_in(stage, &asks, 0.4)?);
        eyeing.extend(fading_in(stage, &[other], 0.4)?);
        stage.play_prepared(eyeing)?;

        // 01_5: what our coin stands for
        let line = say(&kit, stage, "01_5")?;
        let mut clear = a.mobs();
        clear.extend(b.mobs());
        clear.extend(bar.mobs());
        clear.extend(words);
        clear.extend(numbers);
        clear.extend(asks);
        clear.extend([mark, other, cap, bracket, wins]);
        // the coin alone, with what it is
        stage.move_to(label, v(MID_AT[0], MID_AT[1] - MID_R - 0.55), ORIGIN);
        let mut alone = fading_out(stage, &clear, 0.5)?;
        alone.extend(fading_in(stage, &[label], 0.5)?);
        stage.play_prepared(alone)?;
        line.wait_for(stage, "can't be weighted")?;
        let fine = format!(
            "{WEIGHTING} ({GELMAN_NOLAN}, {}). {STANDS_FOR}",
            cited::GELMAN_NOLAN_YEAR
        );
        let third = lower_third(&kit, stage, REAL_COINS, &fine)?;
        fade_in_all(stage, &[third], 0.5)?;
        // it steps up, to stand over the three it stands for
        wait_until(stage, line.at("But a spun coin")? - 0.8)?;
        let mut to_top = coin.to(stage, TOP_AT, TOP_R, 0.8)?;
        to_top.extend(fading_out(stage, &[label], 0.4)?);
        stage.play_prepared(to_top)?;
        beside(stage, TOP_AT, TOP_R);
        let (card_y, picture_y, words_y) = (0.3, 0.72, -0.72);
        let mut pictures: Vec<Mob> = Vec::new();
        // a coin spinning on its edge
        line.wait_for(stage, "But a spun coin")?;
        let x = -4.6;
        let card = picture_card(stage, v(x, card_y))?;
        let [spun, table, spinner] = spun_coin(stage, v(x, picture_y), 0.62)?;
        let name = kit.text(stage, SPUN_COIN, 26.0, Palette::ink(), v(x, words_y))?;
        fade_in_all(stage, &[card, table, spun, name, label], 0.5)?;
        stage.add_to_scene(spinner)?;
        pictures.extend([card, table, spun, name, spinner]);
        // a loaded die, and with it (far more mildly) a computer's noise:
        // both come in together, so the third card is up as long as the
        // other two
        line.wait_for(stage, "and a loaded die")?;
        let card = picture_card(stage, v(0.0, card_y))?;
        let die = die_face(stage, 6, v(0.0, picture_y), 1.2, Palette::ink(), hex(DARK))?;
        stage.rotate(die, -0.2, OUT, None, None);
        let name = kit.text(stage, LOADED_DIE, 26.0, Palette::ink(), v(0.0, words_y))?;
        let x = 4.6;
        let noise_card = picture_card(stage, v(x, card_y))?;
        let trace = noise_trace(stage, v(x, picture_y + 0.1), (3.3, 1.0), 44);
        let name_a = kit.text(stage, NOISE_A, 22.0, Palette::ink(), v(x, words_y + 0.14))?;
        let name_b = kit.text(stage, NOISE_B, 22.0, Palette::ink(), v(x, words_y - 0.18))?;
        let mut both = fading_in(stage, &[card, die, name, noise_card, name_a, name_b], 0.5)?;
        both.push(stage.prepare(show_creation(trace).rt(0.9))?);
        stage.play_prepared(both)?;
        pictures.extend([card, die, name, noise_card, trace, name_a, name_b]);

        // 01_6: the question chapter 02 answers; the three pictures stay
        // until its card comes
        let line = say(&kit, stage, "01_6")?;
        wait_until(stage, line.start + 0.9)?;
        pictures.extend([third, label]);
        let [a, b] = players(&kit, stage)?;
        let [question, whatever] = question_card(&kit, stage, v(0.0, CAPTION_Y + 1.15))?;
        let mut ins = a.mobs();
        ins.extend(b.mobs());
        let leave = fading_out(stage, &pictures, 0.4)?;
        let mut to_mid = coin.to(stage, MID_AT, MID_R, 0.7)?;
        to_mid.extend(fading_in(stage, &ins, 0.7)?);
        let card_in = fade_in(stage.arena_mut(), question, v(0.0, 0.15), 1.0)?;
        to_mid.push(stage.prepare(card_in.rt(0.7))?);
        swap(stage, leave, to_mid)?;
        // whatever either of them knows: each eyes the other again
        line.wait_for(stage, "whatever either of them knows")?;
        let mut eyeing = a.eye_the_other(stage, 0.4)?;
        eyeing.extend(b.eye_the_other(stage, 0.4)?);
        eyeing.extend(fading_in(stage, &[whatever], 0.4)?);
        stage.play_prepared(eyeing)?;
        predict_pause(&kit, stage, v(0.0, MID_AT[1] + MID_R + 0.75))?;
        // they turn back to the coin; the question hangs
        let mut back = a.look_at_the_coin(stage, 0.5)?;
        back.extend(b.look_at_the_coin(stage, 0.5)?);
        stage.play_prepared(back)?;
        // a short breath: the prediction pause has already held the picture
        kit.music_breath(stage, 1.0)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
