//! Chapter 07, one run: trace d replayed on five lanes (Pink stays home:
//! the trace has four customers), fast, slowing at the beats the voice
//! names. A step clock counts reads, writes and successful waits the 02_4
//! way and ends at the trace's counted steps (C-F6.d.steps_counted); 01's
//! scoreboard sits in the room and ticks once per visit. Purple and Green
//! choose together and Purple meets Green's raised flag (07_2); Teal reads
//! two ones and takes two (07_3–07_4); the tie goes to Green; Blue arrives
//! late and takes three, and the run holds on her 3 (07_5); Green is out
//! before Blue checks her, Blue waits for Purple then Teal, Green takes
//! four (07_7); the serving order is the pair order (07_6). Everything
//! shown is `Trace::D.steps()`, the replay the facts pin.

use explainer_kit::props::predict_pause;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    LaneLayout, LaneSpec, Slots, Spoken, TraceLanes, chalk, fade_in_all, fail, flat, group, order_gold, play_timed,
    room, room_red, say, scoreboard,
};
use super::glyphs;
use super::ties::{PairGrid, pair_token};
use crate::facts::waits_behind;
use crate::model::{SCORE_START, Step, Trace};

/// The lanes: Blue on top, Pink (idle) at the bottom.
const LANE_Y: [f64; 5] = [1.75, 0.65, -0.45, -1.55, -2.65];
/// How bright a glance stays once the run has moved past it: gone (the
/// renderer blends in linear light, so even a few percent reads as a
/// haze); the dots, bands and room bars keep the history.
const PAST: f64 = 0.0;
/// How dark a lane goes while the voice is about other customers.
const DIM: f64 = 0.72;
/// In-trace values, scaled up from TraceLanes' chips.
const CHIP_SCALE: f64 = 1.45;

/// The 1-based step at which customer `who`'s label first starts with `pat`.
fn step_of(steps: &[Step], who: usize, pat: &str) -> fmn::Result<usize> {
    steps
        .iter()
        .position(|s| s.customer == who && s.label.starts_with(pat))
        .map(|k| k + 1)
        .ok_or_else(|| fail(format!("07: trace d has no step {pat:?} by customer {who}")))
}

/// Counted steps (the 02_4 way) among the first `upto` steps.
fn counted_upto(steps: &[Step], upto: usize) -> usize {
    steps[..upto].iter().filter(|s| s.counted()).count()
}

pub struct OneRun {
    pub kit: Kit,
}

/// The run as it plays: the lanes, the step clock, the scoreboard, and how
/// far the picture has got.
struct Run<'a> {
    kit: &'a Kit,
    steps: Vec<Step>,
    lanes: TraceLanes,
    clock: Mob,
    board: super::draw::Scoreboard,
    served: u32,
    done: usize,
    /// Steps from here on are still drawn at full strength.
    bright: usize,
    /// Where the run has got to (scene x): the room bars grow to it.
    front: Mob,
    /// One dimmer per drawn lane, and the room bars, kept on top.
    dimmers: Vec<Mob>,
    bars: Vec<Mob>,
}

impl Run<'_> {
    /// Play steps `done + 1 ..= upto` over `run_time`: the lanes, the heads,
    /// the clock and the room bars. The stretch is cut at every room entry,
    /// so each visit lands at its own moment: the scoreboard ticks once per
    /// visit, and the score gets a `tick:` mark right on the entry (Codex
    /// round 6, P2: one mark per batch put two visits at one instant).
    fn to(&mut self, stage: &mut Stage<'_>, upto: usize, run_time: f64) -> fmn::Result<()> {
        if upto <= self.done {
            return Ok(());
        }
        let mut fades: Vec<Box<dyn Animation>> = Vec::new();
        // the last stretch's glances clear: dots, bands and room bars keep
        // the history, the new glances stand out
        for p in &self.lanes.parts[self.bright..self.done] {
            for (m, o) in [(p.arrow, PAST), (p.out, PAST), (p.static_fx, PAST), (p.chip, 0.0)] {
                let Some(m) = m else { continue };
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.5),
                        ..AnimateArgs::default()
                    })?
                    .set_opacity(o)?;
                fades.push(stage.prepare(a)?);
            }
        }
        self.bright = self.done;
        // segments ending at each entry, then the rest
        let mut ends: Vec<usize> = (self.done + 1..=upto).filter(|&k| self.steps[k - 1].enters()).collect();
        if ends.last() != Some(&upto) {
            ends.push(upto);
        }
        let per_step = run_time / (upto - self.done) as f64;
        for end in ends {
            let rt = per_step * (end - self.done) as f64;
            let mut anims: Vec<Box<dyn Animation>> = std::mem::take(&mut fades);
            if let Some(r) = self.lanes.reveal(stage, self.done + 1..=end, rt)? {
                anims.push(r);
            }
            let clock = tween(self.clock, counted_upto(&self.steps, end) as f64, rt)?;
            anims.push(stage.prepare(clock)?);
            if let Some(x) = self.lanes.x(end) {
                let front = tween(self.front, x + self.lanes.slot_w() / 2.0, rt)?;
                anims.push(stage.prepare(front)?);
            }
            stage.play_prepared(anims)?;
            // newly drawn steps go under the dimmers, the room bars over all
            stage.scene_mut().bring_to_front(&self.dimmers)?;
            stage.scene_mut().bring_to_front(&self.bars)?;
            let heads = self.lanes.heads_to(self.kit, stage, end)?;
            let enters = self.steps[end - 1].enters();
            if enters {
                self.kit.mark(stage, "tick:served");
            }
            if !heads.is_empty() {
                play_timed(stage, heads, 0.25)?;
            }
            if enters {
                self.served += 1;
                let tick = self.board.show(self.kit, stage, SCORE_START + self.served)?;
                play_timed(stage, tick.into(), 0.35)?;
            }
            self.done = end;
        }
        Ok(())
    }

    /// Clear the last stretch's glances too (the run is over).
    fn settle(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for p in &self.lanes.parts[self.bright..self.done] {
            for m in [p.arrow, p.out, p.static_fx, p.chip].into_iter().flatten() {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.6),
                        ..AnimateArgs::default()
                    })?
                    .set_opacity(PAST)?;
                anims.push(stage.prepare(a)?);
            }
        }
        self.bright = self.done;
        if !anims.is_empty() {
            stage.play_prepared(anims)?;
        }
        Ok(())
    }

    /// Light the lanes of `customers`, dim the rest (the voice's subject).
    fn focus(&self, stage: &mut Stage<'_>, customers: &[usize]) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (k, &m) in self.dimmers.iter().enumerate() {
            let o = if customers.contains(&(k + 1)) { 0.0 } else { DIM };
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.5),
                    ..AnimateArgs::default()
                })?
                .set_fill(None, Some(o))?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// Wait for `phrase` of `line`, then play up to `upto` over `run_time`.
    fn at(
        &mut self,
        stage: &mut Stage<'_>,
        line: &Spoken,
        phrase: &str,
        upto: usize,
        run_time: f64,
    ) -> fmn::Result<()> {
        line.wait_for(stage, phrase)?;
        self.to(stage, upto, run_time)
    }
}

impl SceneConstruct for OneRun {
    fn name(&self) -> &str {
        "07_one_run"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let steps = Trace::D.steps();
        let n = steps.len();
        let layout = LaneLayout {
            head_x: -6.55,
            x0: -4.55,
            x1: 6.75,
            lanes: (1..=5)
                .map(|k| LaneSpec {
                    customer: k,
                    y: LANE_Y[k - 1],
                    idle: k == 5,
                })
                .collect(),
            slots: Slots::Counted,
            slate_size: 0.6,
        };
        // the lane heads are the customers themselves (R3, maya-02)
        let lanes = TraceLanes::build_with_faces(&kit, stage, layout, &steps, &[])?;
        for p in &lanes.parts {
            if let Some(c) = p.chip {
                stage.scale(c, CHIP_SCALE);
            }
        }
        // the room bars: each visit, from its entry to its exit, grown to
        // where the run has got; the brightest thing on the lanes
        let front = stage.add_value_tracker(lanes.layout.x0);
        let bars = room_bars(stage, &lanes, &steps, front)?;
        // a dimmer over each drawn lane (Blue to Teal), clear for now; from
        // the badge's left edge to the lane's end, inside the safe area
        let (dim_l, dim_r) = (lanes.layout.head_x - 0.4, lanes.layout.x1 + 0.2);
        let mut dimmers = Vec::new();
        for &y in &LANE_Y[..4] {
            let d = Rectangle::new()
                .width(dim_r - dim_l)
                .height(0.98)
                .style(flat(explainer_kit::hex(explainer_kit::BACKGROUND), 0.0))
                .build()?
                .moved_to(v((dim_l + dim_r) / 2.0, y));
            dimmers.push(stage.arena_mut().add(d));
        }
        let pink = kit.text_left(
            stage,
            glyphs::PINK_HOME,
            22.0,
            Palette::dim(),
            v(lanes.layout.x0 + 0.1, LANE_Y[4] + 0.34),
        )?;

        // the top band: the step clock, the room with 01's scoreboard
        let clock_label = kit.text_left(stage, glyphs::STEPS, 26.0, Palette::dim(), v(-6.75, 3.3))?;
        let clock = stage.add_value_tracker(0.0);
        let [tint, edge] = room(3.5, 1.25, v(4.95, 3.05))?;
        let (tint, edge) = (stage.arena_mut().add(tint), stage.arena_mut().add(edge));
        let room_label = kit.text(stage, glyphs::THE_ROOM, 22.0, room_red(), v(3.8, 3.05))?;
        let board = scoreboard(&kit, stage, SCORE_START, v(5.65, 3.05), 0.8)?;

        // 07_1: the run, fast
        let line = say(&kit, stage, "07_1")?;
        let mut base = lanes.base();
        base.extend([clock_label, tint, edge, room_label, board.panel, board.value, pink]);
        let mut show: Vec<Box<dyn Animation>> = Vec::new();
        for m in base {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.8);
            show.push(stage.prepare(a)?);
        }
        stage.play_prepared(show)?;
        for &m in dimmers.iter().chain(&bars) {
            stage.add_to_scene(m)?;
        }
        let readout = kit.live_number(stage, clock, 0, false, 30.0, chalk(), v(-5.65, 3.3), LEFT)?;
        let _ = line;
        let mut run = Run {
            kit: &kit,
            steps: steps.clone(),
            lanes,
            clock,
            board,
            served: 0,
            done: 0,
            bright: 0,
            front,
            dimmers,
            bars,
        };

        // 07_2: Purple and Green choose together; Purple meets Green's flag
        let stopped = step_of(&steps, 3, "L2: choosing[2]=1, wait")?;
        let green_writes = step_of(&steps, 2, "number[2]:=1")?;
        let green_lowers = step_of(&steps, 2, "choosing:=0")?;
        let line = say(&kit, stage, "07_2")?;
        run.focus(stage, &[2, 3])?;
        run.to(stage, stopped - 1, 2.2)?;
        line.wait_for(stage, "finds Green's flag")?;
        run.to(stage, stopped, 0.6)?;
        let ring = highlight(&run.lanes, stage, stopped)?;
        // without the flag: Green's slate still says zero (her ticket unwritten)
        line.wait_for(stage, "Without the flag")?;
        let green_slate = run.lanes.heads[1].slate.parts();
        let unwritten = kit.text(stage, glyphs::UNWRITTEN, 20.0, room_red(), v(-5.83, LANE_Y[1] - 0.42))?;
        play!(stage;
            indicate(stage.arena_mut(), green_slate[1], 1.4, Some(rgb(room_red())))?.rt(1.0),
            fade_in(stage.arena_mut(), unwritten, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "unwritten ticket")?;
        play!(stage;
            fade_out(stage.arena_mut(), unwritten, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.5),
        );
        run.to(stage, green_lowers.max(green_writes), 1.0)?;
        kit.hold(stage)?;

        // 07_3: Teal reads Blue's 0, Green's 1, Purple's 1; what does she take?
        let teal_reads_done = step_of(&steps, 4, "read number[3]=")?;
        let teal_writes = step_of(&steps, 4, "number[4]:=")?;
        let line = say(&kit, stage, "07_3")?;
        run.focus(stage, &[2, 3, 4])?;
        run.at(stage, &line, "Teal sees", teal_reads_done, 1.6)?;
        let q = predict_pause(&kit, stage, v(1.6, 3.0))?;

        // 07_4: two; the tie goes to Green, customer two
        let line = say(&kit, stage, "07_4")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        run.to(stage, teal_writes + 1, 0.8)?;
        line.wait_for(stage, "the tie")?;
        let tie = tie_card(&kit, stage, &steps)?;
        play!(stage; fade_in(stage.arena_mut(), tie, v(0.0, -0.15), 1.0)?.rt(0.7));
        let purple_waits = step_of(&steps, 3, "L3: number[2]=1, wait")?;
        run.to(stage, purple_waits, 0.8)?;
        kit.hold(stage)?;

        // 07_5: Green in first; Blue late with three, and the run holds on
        // her 3 (R3, maya-02: four events in one line was too many)
        let green_in = step_of(&steps, 2, "L3: number[4]=2, pass")?;
        let blue_writes = step_of(&steps, 1, "number[1]:=")?;
        let line = say(&kit, stage, "07_5")?;
        play!(stage; fade_out(stage.arena_mut(), tie, ORIGIN, 1.0)?.rt(0.4));
        run.focus(stage, &[1, 2])?;
        run.to(stage, green_in, 1.4)?;
        run.at(stage, &line, "arrives late", blue_writes, 1.4)?;
        kit.hold(stage)?;
        // a gold frame round her slate, and the 3 pulses (gold on the dark
        // board: an indicated board would bury a gold numeral)
        let (board, three) = (run.lanes.heads[0].slate.board, run.lanes.heads[0].slate.value);
        let frame = SurroundingRectangle::from_extent(Some((stage.get_corner(board, DL), stage.get_corner(board, UR))))
            .buff(0.08)
            .color(order_gold())
            .build();
        let frame = stage.arena_mut().add(frame);
        play!(stage;
            show_creation(frame).rt(0.5),
            indicate(stage.arena_mut(), three, 1.5, Some(rgb(order_gold())))?.rt(1.2),
        );
        stage.wait(0.3)?;
        play!(stage; fade_out(stage.arena_mut(), frame, ORIGIN, 1.0)?.rt(0.3));

        // 07_7: Green out before Blue checks her (her slate is back at 0);
        // Blue waits for Purple, then Teal; Green takes four
        let green_out = step_of(&steps, 2, "number[2]:=0")?;
        let blue_checks_green = step_of(&steps, 1, "L3: number[2]")?;
        let behind = waits_behind(&steps, 1);
        let wait_purple = steps
            .iter()
            .position(|s| s.customer == 1 && s.blocked() && s.label.starts_with("L3: number[3]"))
            .map_or(0, |k| k + 1);
        let wait_teal = steps
            .iter()
            .position(|s| s.customer == 1 && s.blocked() && s.label.starts_with("L3: number[4]"))
            .map_or(0, |k| k + 1);
        if behind.len() != 2 || wait_purple == 0 || wait_teal == 0 {
            return Err(fail(format!(
                "07: Blue should wait behind Purple, then Teal: {behind:?}"
            )));
        }
        let green_again = step_of(&steps[green_out..], 2, "number[2]:=")? + green_out;
        let line = say(&kit, stage, "07_7")?;
        run.focus(stage, &[1, 2, 3, 4])?;
        run.to(stage, green_out, 0.5)?;
        run.at(stage, &line, "before Blue checks", blue_checks_green, 0.9)?;
        let look = highlight(&run.lanes, stage, blue_checks_green)?;
        let green = run.lanes.heads[1].slate.value;
        play!(stage; indicate(stage.arena_mut(), green, 1.5, Some(rgb(order_gold())))?.rt(1.0));
        run.at(stage, &line, "waits for Purple", wait_purple, 0.9)?;
        run.at(stage, &line, "then Teal", wait_teal, 0.9)?;
        run.at(stage, &line, "Green gets back", green_again, 1.0)?;
        play!(stage; fade_out(stage.arena_mut(), look, ORIGIN, 1.0)?.rt(0.4));
        run.to(stage, n, 2.2)?;
        kit.hold(stage)?;

        // 07_6: five visits, five points, none lost; the order is the grid's
        let line = say(&kit, stage, "07_6")?;
        run.settle(stage)?;
        let board = run.board.panel;
        play!(stage; indicate(stage.arena_mut(), board, 1.12, None)?.rt(0.9));
        line.wait_for(stage, "Who gets served")?;
        served_order(&kit, stage, &steps, &line)?;
        kit.hold(stage)?;
        stage.clear_updaters(readout, true);
        explainer_kit::fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

use fmn::animation::indicate;

/// The room's visits on the lanes: one bright red bar per visit, from the
/// left of its entry's slot to the right of its exit's (or the lanes' end),
/// redrawn so it reaches only as far as the run has got (`front`).
fn room_bars(stage: &mut Stage<'_>, lanes: &TraceLanes, steps: &[Step], front: Mob) -> fmn::Result<Vec<Mob>> {
    let w = lanes.slot_w();
    let mut out = Vec::new();
    for (e, s) in steps.iter().enumerate().filter(|(_, s)| s.enters()) {
        let c = s.customer;
        let exit = steps[e + 1..]
            .iter()
            .position(|t| t.customer == c && t.label.starts_with(&format!("number[{c}]:=0")))
            .map(|k| e + 1 + k + 1);
        let (Some(x0), Some(y)) = (lanes.x(e + 1), lanes.y(c)) else {
            continue;
        };
        let x0 = x0 - w / 2.0;
        let x1 = exit.and_then(|x| lanes.x(x)).map_or(lanes.layout.x1, |x| x + w / 2.0);
        let bar = stage.always_redraw(move |st| {
            let f = st.tracker_value(front).unwrap_or(x0);
            let right = f.min(x1);
            let piece = if right > x0 + 1e-6 {
                Rectangle::new()
                    .width(right - x0)
                    .height(0.3)
                    .corner_radius(0.06)
                    .style(flat(room_red(), 0.9))
                    .build()
                    .expect("positive extents")
                    .moved_to(v((x0 + right) / 2.0, y))
            } else {
                VMobject::new()
            };
            st.add(piece)
        });
        stage.remove_from_scene(bar);
        out.push(bar);
    }
    Ok(out)
}

/// An `Srgb` as `indicate` takes it.
fn rgb(c: Srgb) -> [f32; 3] {
    super::draw::rgb(c)
}

/// A gold ring round step `step`'s dot (a moment the voice names).
fn highlight(lanes: &TraceLanes, stage: &mut Stage<'_>, step: usize) -> fmn::Result<Mob> {
    let s = &lanes.steps[step - 1];
    let x = lanes
        .x(step)
        .ok_or_else(|| fail(format!("07: step {step} has no slot")))?;
    let y = lanes
        .y(s.customer)
        .ok_or_else(|| fail(format!("07: no lane for customer {}", s.customer)))?;
    let ring = Circle::new()
        .radius(0.2)
        .arc_center(v(x, y))
        .style(super::draw::line_style(order_gold(), 3.0))
        .build();
    let ring = stage.arena_mut().add(ring);
    play!(stage; fmn::animation::grow_from_center(stage.arena_mut(), ring, None)?.rt(0.4));
    Ok(ring)
}

/// 07_4's card: the tie between Purple and Green, settled by the customer
/// number, from the tickets they hold at Purple's wait.
fn tie_card(kit: &Kit, stage: &mut Stage<'_>, steps: &[Step]) -> fmn::Result<Mob> {
    let wait = step_of(steps, 3, "L3: number[2]=")?;
    let st = &steps[wait - 1].before;
    let (g, p) = (st.number[1], st.number[2]);
    if g != p || g == 0 {
        return Err(fail(format!("07_4: Green and Purple should tie, hold {g} and {p}")));
    }
    let at = v(0.0, 3.05);
    let first = pair_token(kit, stage, g, 2, v(at[0] - 2.0, at[1]), 0.8)?;
    let before = kit.text(stage, glyphs::COMES_BEFORE, 30.0, chalk(), at)?;
    let second = pair_token(kit, stage, p, 3, v(at[0] + 2.0, at[1]), 0.8)?;
    group(stage, &[first, before, second])
}

/// 07_6: the five visits as pairs in the order they happened (across the
/// top), then 04's grid over the finished run: its scanline sweeps column by
/// column, and each pair it reaches lights the next visit, left to right.
fn served_order(kit: &Kit, stage: &mut Stage<'_>, steps: &[Step], line: &Spoken) -> fmn::Result<()> {
    let entries: Vec<(u32, usize)> = steps
        .iter()
        .filter(|s| s.enters())
        .map(|s| (s.after.number[s.customer - 1], s.customer))
        .collect();
    let mut sweep = entries.clone();
    sweep.sort_unstable();
    if sweep != entries {
        return Err(fail("07_6: the serving order is not the pair order".to_owned()));
    }
    // the visits as they happened
    let (y, x0, dx) = (3.05, -3.6, 1.45);
    let mut tokens = Vec::new();
    for (i, &(t, k)) in entries.iter().enumerate() {
        tokens.push(pair_token(kit, stage, t, k, v(x0 + dx * i as f64, y), 0.75)?);
    }
    let mut ins: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &m) in tokens.iter().enumerate() {
        let a = fade_in(stage.arena_mut(), m, v(0.0, -0.1), 1.0)?.rt(0.3 + 0.12 * i as f64);
        ins.push(stage.prepare(a)?);
    }
    stage.play_prepared(ins)?;
    // 04's grid, on a card over the run
    line.wait_for(stage, "follows the grid")?;
    let top_ticket = entries.iter().map(|p| p.0).max().unwrap_or(1);
    let top_customer = entries.iter().map(|p| p.1).max().unwrap_or(1);
    let grid = PairGrid {
        center: v(3.35, -0.55),
        cell: 0.8,
        tickets: top_ticket,
        customers: top_customer,
    };
    let card = Rectangle::new()
        .width(0.8 * f64::from(top_ticket) + 2.3)
        .height(0.8 * top_customer as f64 + 1.9)
        .corner_radius(0.15)
        .style(
            Style::default()
                .fill(explainer_kit::hex("#0E1116"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(v(grid.center[0] - 0.35, grid.center[1] - 0.25));
    let card = stage.arena_mut().add(card);
    let lines = stage.arena_mut().add(grid.lines()?);
    let labels = grid.labels(kit, stage, glyphs::TICKET_ACROSS, glyphs::CUSTOMER_NUMBER_UP)?;
    let cells: Vec<Mob> = entries
        .iter()
        .map(|&(t, k)| grid.pair(kit, stage, t, k))
        .collect::<fmn::Result<_>>()?;
    let mut ins = vec![card, lines];
    ins.extend(&labels);
    ins.extend(&cells);
    fade_in_all(stage, &ins, 0.5)?;
    let scan = stage.arena_mut().add(grid.scanline(1.0)?);
    play!(stage; fade_in(stage.arena_mut(), scan, ORIGIN, 1.0)?.rt(0.3));
    let mut x_now = grid.before(1.0);
    for (n, &(t, k)) in sweep.iter().enumerate() {
        let x = grid.before(f64::from(t) + 1.0);
        if x > x_now + 1e-9 {
            let a = scan
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.35),
                    ..AnimateArgs::default()
                })?
                .shift(v(x - x_now, 0.0))?;
            stage.play(a)?;
            x_now = x;
        }
        let cell = cells[entries.iter().position(|&e| e == (t, k)).unwrap_or(n)];
        let gold = rgb(order_gold());
        play!(stage;
            indicate(stage.arena_mut(), cell, 1.35, Some(gold))?.rt(0.45),
            indicate(stage.arena_mut(), tokens[n], 1.2, Some(gold))?.rt(0.45),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_07_says_is_what_trace_d_does() {
        let d = Trace::D.steps();
        // 07_1: the step clock ends at the counted steps
        assert_eq!(counted_upto(&d, d.len()), 64);
        // 07_2: both choose ticket one; Purple meets Green's raised flag
        let tickets: Vec<(usize, u32)> = d.iter().filter_map(Step::ticket_written).collect();
        assert_eq!(&tickets[..2], &[(3, 1), (2, 1)]);
        let stopped = step_of(&d, 3, "L2: choosing[2]=1, wait").expect("stopped");
        assert_eq!(
            d[stopped - 1].before.number[1],
            0,
            "Green's ticket unwritten when Purple looks"
        );
        // 07_3-07_4: Teal sees two ones and takes two
        assert_eq!(crate::model::reads_by(&d, 4), [0, 1, 1]);
        assert_eq!(tickets[2], (4, 2));
        // 07_5, 07_7: Blue takes three, Green four; Blue waits behind
        // Purple, then Teal
        assert_eq!(tickets[3..], [(1, 3), (2, 4)]);
        assert_eq!(waits_behind(&d, 1), [(1, 3), (2, 4)]);
        // 07_7: Green is out before Blue checks her, so the ringed read
        // finds her slate back at 0
        let out = step_of(&d, 2, "number[2]:=0").expect("out");
        let check = step_of(&d, 1, "L2: choosing[2]").expect("check");
        let read = step_of(&d, 1, "L3: number[2]").expect("read");
        assert!(out < check && check < read, "Green is out before Blue checks her");
        assert_eq!(d[read - 1].before.number[1], 0, "Blue reads Green's 0");
        // 07_6: five visits, five points (41 to 46), never two inside
        let entries = d.iter().filter(|s| s.enters()).count() as u32;
        assert_eq!((entries, SCORE_START + entries), (5, 46));
        let model = Trace::D.model();
        assert!(d.iter().all(|s| model.in_room(&s.after).len() <= 1));
    }
}
