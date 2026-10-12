//! Chapter 12, your CPU: the proof's second promise breaks over a chip, with
//! the measured failure beside it (12_1); the outbox x-ray (12_2), on which
//! the voiced order of the x86-TSO model plays step by step (12_3–12_4); the
//! fences F1 and F2 drain the outboxes and the same race lets one customer in
//! (12_5); a reordering outbox lets Green in on Blue's lowered flag, chapter
//! 05's crack again (12_6); the SeqCst panel, F1 and F2 marked (12_7).
//!
//! Every move drawn on the x-ray is a move of `model::tso`, replayed from the
//! voiced order the facts pin (`tso::VOICED`) and from this chapter's two
//! schedules ([`FENCED`], [`REORDERED`]), each replayed move by move.

use std::collections::VecDeque;

use explainer_kit::props::{CodePanel, lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::core::constants::FRAME_X_RADIUS;
use fmn::prelude::*;

use super::draw::{
    Flag, Slate, access_arrow, badge, board, chalk, chip_outline, customer, fail, flag, flag_amber, flat, group,
    hardware, line_style, play_timed, rgb, room, room_red, say, slate, wait_until,
};
use super::glyphs;
use super::tunes;
use crate::facts::{Arch, Measured, measured};
use crate::model::tso::{self, Move, Store, Tso, Var};

/// The lanes (Blue above, Green below), the outbox trays, and memory.
const LANE_Y: [f64; 2] = [2.3, -2.15];
const TRAY_Y: [f64; 2] = [1.2, -1.1];
const MEM_Y: f64 = 0.05;
/// Stations along a lane: start, flag up, read, ticket, flag down, check
/// flag, check slate.
const STATION_X: [f64; 7] = [-5.4, -4.6, -3.4, -2.2, -1.0, 0.6, 1.9];
const STATION_NAMES: [&str; 6] = [
    glyphs::STATION_FLAG_UP,
    glyphs::STATION_READ,
    glyphs::STATION_TICKET,
    glyphs::STATION_FLAG_DOWN,
    glyphs::STATION_CHECK_FLAG,
    glyphs::STATION_CHECK_SLATE,
];
/// The fences: F1 after raising the flag, F2 after lowering it.
const FENCE_X: [f64; 2] = [-4.0, -0.2];
/// Each tray's right end (the oldest letter waits there); slots run left.
const TRAY_RIGHT: [f64; 2] = [-0.6, 3.4];
const SLOTS: usize = 4;
const SLOT_W: f64 = 0.95;
/// Memory cells, per customer: (flag, slate).
const CELL_X: [[f64; 2]; 2] = [[-3.6, -2.0], [0.4, 2.0]];
/// The room.
const ROOM_AT: Vec3 = [5.65, 0.05, 0.0];
const ROOM_SEAT: [Vec3; 2] = [[5.2, 0.55, 0.0], [6.1, -0.45, 0.0]];

/// The code panel: its type size and width (the longest line fits), and its
/// left edge once it has slid in (`CodePanel::slide_in`).
const PANEL_PT: f64 = 18.0;
const PANEL_W: f64 = 8.9;
const PANEL_LEFT: f64 = FRAME_X_RADIUS - PANEL_W - 0.35;

/// 12_5's fix on the same race: both fences, so each outbox is delivered
/// before its customer reads; both choose at once, take 1, and the tie goes
/// to Blue while Green waits.
pub const FENCED: [(usize, &str); 22] = [
    (1, "store choosing:=1"),
    (2, "store choosing:=1"),
    (1, "flush choosing[1]=1"),
    (1, "mfence"),
    (2, "flush choosing[2]=1"),
    (2, "mfence"),
    (1, "load number[2]=0"),
    (2, "load number[1]=0"),
    (1, "store number:=1"),
    (2, "store number:=1"),
    (1, "store choosing:=0"),
    (2, "store choosing:=0"),
    (1, "flush number[1]=1"),
    (1, "flush choosing[1]=0"),
    (1, "mfence"),
    (2, "flush number[2]=1"),
    (2, "flush choosing[2]=0"),
    (2, "mfence"),
    (1, "L2 load choosing[2]=0, pass"),
    (1, "L3 load number[2]=1, pass -> ENTER CS"),
    (2, "L2 load choosing[1]=0, pass"),
    (2, "L3 load number[1]=1, wait"),
];

/// 12_6 on the reordering model, both fences in place: Blue's lowered flag
/// reaches memory before her ticket, Green sees no flag and no ticket and
/// walks in (05_2's crack), then Blue's ticket lands, wins the tie, and she
/// walks in too.
pub const REORDERED: [(usize, &str); 22] = [
    (1, "store choosing:=1"),
    (1, "flush choosing[1]=1"),
    (1, "mfence"),
    (1, "load number[2]=0"),
    (1, "store number:=1"),
    (1, "store choosing:=0"),
    (1, "flush choosing[1]=0 to memory (out of order)"),
    (2, "store choosing:=1"),
    (2, "flush choosing[2]=1"),
    (2, "mfence"),
    (2, "load number[1]=0"),
    (2, "store number:=1"),
    (2, "store choosing:=0"),
    (2, "flush number[2]=1"),
    (2, "flush choosing[2]=0"),
    (2, "mfence"),
    (2, "L2 load choosing[1]=0, pass"),
    (2, "L3 load number[1]=0, pass -> ENTER CS"),
    (1, "flush number[1]=1"),
    (1, "mfence"),
    (1, "L2 load choosing[2]=0, pass"),
    (1, "L3 load number[2]=1, pass -> ENTER CS"),
];

/// Short lines stacked under each other, centred at `x`, from `top` down.
fn stacked(kit: &Kit, stage: &mut Stage<'_>, lines: &[&str], size: f64, x: f64, top: f64) -> fmn::Result<Mob> {
    let step = size / 48.0 * 0.62 + 0.08;
    let parts: Vec<Mob> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| kit.text(stage, l, size, Palette::dim(), v(x, top - step * i as f64)))
        .collect::<fmn::Result<_>>()?;
    group(stage, &parts)
}

/// A letter waiting in an outbox: its chip and what it carries.
struct Letter {
    chip: Mob,
    store: Store,
}

/// The x-ray: two cores running Blue and Green, their outboxes, memory and
/// the room, with every piece the moves animate.
struct XRay {
    markers: [Mob; 2],
    at: [Vec3; 2],
    trays: [VecDeque<Letter>; 2],
    flags: [Flag; 2],
    slates: [Slate; 2],
    /// Everything static, for fades.
    frame: Vec<Mob>,
    fences: Vec<Mob>,
}

fn station(k: usize, s: usize) -> Vec3 {
    v(STATION_X[s], LANE_Y[k])
}

fn slot(k: usize, i: usize) -> Vec3 {
    v(TRAY_RIGHT[k] - SLOT_W * (i as f64 + 0.5), TRAY_Y[k])
}

/// A chip icon at the top right with a caption under it: which machine
/// the beat is about.
fn machine_badge(kit: &Kit, stage: &mut Stage<'_>, label: &str, caption: &str) -> fmn::Result<Mob> {
    let chip = chip_outline(kit, stage, v(5.65, 2.8), 0.7, label)?;
    let cap = kit.text(stage, caption, 20.0, Palette::dim(), v(5.65, 2.12))?;
    group(stage, &[chip, cap])
}

fn cell(var: Var, owner: usize) -> Vec3 {
    let x = CELL_X[owner][usize::from(var == Var::Slate)];
    v(x, MEM_Y)
}

/// A rounded frame: a faint fill, a hardware-grey edge.
fn frame_box(w: f64, h: f64, at: Vec3, fill_o: f64) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.14)
        .style(Style::default().fill(board(), fill_o).stroke(hardware(), 2.0, 0.9))
        .build()?
        .moved_to(at))
}

/// A letter's chip: a small card edged in its writer's colour, carrying a
/// tiny flag (up or down) or a chalk ticket.
fn letter_chip(kit: &Kit, stage: &mut Stage<'_>, s: Store, at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(0.82)
        .height(0.44)
        .corner_radius(0.08)
        .style(
            Style::default()
                .fill(board(), 1.0)
                .stroke(customer(s.owner + 1), 2.0, 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let inner = match s.var {
        Var::Flag => {
            let f = flag(stage, v(at[0] - 0.12, at[1] - 0.15), 0.3, s.value != 0)?;
            group(stage, &[f.pole, f.pennant])?
        }
        Var::Slate => kit.text(stage, &s.value.to_string(), 24.0, chalk(), at)?,
    };
    group(stage, &[back, inner])
}

impl XRay {
    /// The x-ray, detached: frames, lanes, stations, trays, memory, room.
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut frame = Vec::new();
        for k in 0..2 {
            let core = frame_box(10.0, 0.9, v(-1.05, LANE_Y[k]), 0.35)?;
            frame.push(stage.arena_mut().add(core));
            frame.push(kit.text(stage, glyphs::CORE, 22.0, Palette::dim(), v(-6.55, LANE_Y[k]))?);
            let lane = Line::new(v(STATION_X[0], LANE_Y[k]), v(3.95, LANE_Y[k]))
                .style(
                    Style::default()
                        .fill(customer(k + 1), 0.0)
                        .stroke(customer(k + 1), 3.0, 0.5),
                )
                .build()?;
            frame.push(stage.arena_mut().add(lane));
            for &x in &STATION_X[1..] {
                let tick = Line::new(v(x, LANE_Y[k] - 0.12), v(x, LANE_Y[k] + 0.12))
                    .style(line_style(Palette::faint(), 2.5))
                    .build()?;
                frame.push(stage.arena_mut().add(tick));
            }
            let tray = frame_box(
                SLOT_W * SLOTS as f64 + 0.1,
                0.56,
                v(TRAY_RIGHT[k] - SLOT_W * 2.0, TRAY_Y[k]),
                0.5,
            )?;
            frame.push(stage.arena_mut().add(tray));
        }
        for (s, name) in STATION_NAMES.iter().enumerate() {
            // above the core's frame, clear of its edge
            frame.push(kit.text(stage, name, 20.0, Palette::dim(), v(STATION_X[s + 1], LANE_Y[0] + 0.66))?);
        }
        // each tray's label at its left end, clear of the room
        for (k, &right) in TRAY_RIGHT.iter().enumerate() {
            let left = right - SLOT_W * SLOTS as f64;
            let label = kit.text(stage, glyphs::OUTBOX, 22.0, Palette::dim(), v(left - 0.2, TRAY_Y[k]))?;
            stage.move_to(label, v(left - 0.2, TRAY_Y[k]), RIGHT);
            frame.push(label);
        }
        let mem = frame_box(8.6, 1.05, v(-0.8, MEM_Y), 0.6)?;
        frame.push(stage.arena_mut().add(mem));
        frame.push(kit.text(stage, glyphs::MEMORY, 24.0, Palette::dim(), v(-5.85, MEM_Y))?);
        let mut flags = Vec::new();
        let mut slates = Vec::new();
        for (k, cells) in CELL_X.iter().enumerate() {
            frame.push(badge(kit, stage, k + 1, 0.17, v(cells[0] - 0.85, MEM_Y))?);
            let f = flag(stage, v(cells[0] - 0.12, MEM_Y - 0.3), 0.6, false)?;
            frame.extend([f.pole, f.pennant]);
            flags.push(f);
            let s = slate(kit, stage, 0, cell(Var::Slate, k), 0.62)?;
            frame.extend(s.parts());
            slates.push(s);
        }
        let [tint, edge] = room(2.4, 3.2, ROOM_AT)?;
        frame.push(stage.arena_mut().add(tint));
        frame.push(stage.arena_mut().add(edge));
        frame.push(kit.text(
            stage,
            glyphs::THE_ROOM,
            26.0,
            room_red(),
            v(ROOM_AT[0], ROOM_AT[1] + 1.25),
        )?);
        let markers = [
            badge(kit, stage, 1, 0.22, station(0, 0))?,
            badge(kit, stage, 2, 0.22, station(1, 0))?,
        ];
        let [f0, f1]: [Flag; 2] = flags.try_into().map_err(|_| fail("two flags".into()))?;
        let [s0, s1]: [Slate; 2] = slates.try_into().map_err(|_| fail("two slates".into()))?;
        Ok(Self {
            markers,
            at: [station(0, 0), station(1, 0)],
            trays: [VecDeque::new(), VecDeque::new()],
            flags: [f0, f1],
            slates: [s0, s1],
            frame,
            fences: Vec::new(),
        })
    }

    /// Everything currently built, for a fade in.
    fn all(&self) -> Vec<Mob> {
        let mut m = self.frame.clone();
        m.extend(self.markers);
        m
    }

    /// The fence gates on both lanes, labelled F1 and F2.
    fn add_fences(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
        let mut out = Vec::new();
        for (k, &y) in LANE_Y.iter().enumerate() {
            for (&x, name) in FENCE_X.iter().zip([glyphs::F1, glyphs::F2]) {
                let gate = Rectangle::new()
                    .width(0.14)
                    .height(0.7)
                    .corner_radius(0.05)
                    .style(flat(flag_amber(), 0.9))
                    .build()?
                    .moved_to(v(x, y));
                let g = stage.arena_mut().add(gate);
                let dy = if k == 0 { -0.56 } else { 0.56 };
                let t = kit.text(stage, name, 22.0, flag_amber(), v(x, y + dy))?;
                out.extend([g, t]);
            }
        }
        self.fences.extend(&out);
        Ok(out)
    }

    /// Move a marker to `to`.
    fn walk(&mut self, k: usize, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
        self.at[k] = to;
        Ok(self.markers[k]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?)
    }

    /// Animate one model move by customer `k` (0-based), `rt` seconds.
    fn play_move(&mut self, kit: &Kit, stage: &mut Stage<'_>, k: usize, m: Move, rt: f64) -> fmn::Result<()> {
        match m {
            Move::Buffer(s) => {
                let to = match (s.var, s.value) {
                    (Var::Flag, 1) => station(k, 1),
                    (Var::Slate, 0) => self.at[k],
                    (Var::Slate, _) => station(k, 3),
                    (Var::Flag, _) => station(k, 4),
                };
                let w = self.walk(k, to, rt * 0.4)?;
                stage.play(w)?;
                let chip = letter_chip(kit, stage, s, to)?;
                let target = slot(k, self.trays[k].len());
                self.trays[k].push_back(Letter { chip, store: s });
                play!(stage;
                    fade_in(stage.arena_mut(), chip, ORIGIN, 0.6)?.rt(rt * 0.25),
                );
                let fly = chip
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt * 0.35),
                        ..AnimateArgs::default()
                    })?
                    .move_to(target, ORIGIN)?;
                stage.play(fly)?;
            }
            Move::Deliver(s) | Move::DeliverEarly(s) => {
                let idx = self.trays[k]
                    .iter()
                    .position(|l| l.store == s)
                    .ok_or_else(|| fail("a delivered letter waits in its tray".into()))?;
                let letter = self.trays[k].remove(idx).ok_or_else(|| fail("tray index".into()))?;
                let early = matches!(m, Move::DeliverEarly(_));
                let drop = letter
                    .chip
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(rt * 0.5),
                        path_arc: Some(if early { 1.2 } else { 0.0 }),
                        ..AnimateArgs::default()
                    })?
                    .move_to(cell(s.var, s.owner), ORIGIN)?;
                stage.play(drop)?;
                let mut anims: Vec<Box<dyn Animation>> = Vec::new();
                let gone = fade_out(stage.arena_mut(), letter.chip, ORIGIN, 0.6)?;
                anims.push(stage.prepare(gone.rt(rt * 0.3))?);
                match s.var {
                    Var::Flag => {
                        let t = self.flags[s.owner].set(stage, s.value != 0);
                        anims.push(stage.prepare(t.rt(rt * 0.3))?);
                    }
                    Var::Slate => {
                        anims.extend(self.slates[s.owner].rewrite(kit, stage, s.value)?);
                    }
                }
                for (i, l) in self.trays[k].iter().enumerate() {
                    let a = l
                        .chip
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(rt * 0.3),
                            ..AnimateArgs::default()
                        })?
                        .move_to(slot(k, i), ORIGIN)?;
                    anims.push(stage.prepare(a)?);
                }
                play_timed(stage, anims, rt * 0.45)?;
            }
            Move::Fence => {
                let gate = if self.at[k][0] < FENCE_X[0] + 0.5 { 0 } else { 1 };
                let w = self.walk(k, v(FENCE_X[gate], LANE_Y[k]), rt * 0.6)?;
                stage.play(w)?;
            }
            Move::ReadSlate { j, value } => {
                let w = self.walk(k, station(k, 2), rt * 0.35)?;
                stage.play(w)?;
                self.glance(kit, stage, k, cell(Var::Slate, j), &value.to_string(), rt * 0.65)?;
            }
            Move::CheckFlag { j, up } => {
                let w = self.walk(k, station(k, 5), rt * 0.35)?;
                stage.play(w)?;
                self.glance(
                    kit,
                    stage,
                    k,
                    cell(Var::Flag, j),
                    if up { glyphs::UP } else { glyphs::DOWN },
                    rt * 0.65,
                )?;
            }
            Move::CheckSlate { j, value, enter, .. } => {
                let w = self.walk(k, station(k, 6), rt * 0.35)?;
                stage.play(w)?;
                self.glance(kit, stage, k, cell(Var::Slate, j), &value.to_string(), rt * 0.45)?;
                if enter {
                    let w = self.walk(k, ROOM_SEAT[k], rt * 0.5)?;
                    stage.play(w)?;
                }
            }
        }
        Ok(())
    }

    /// A read: an arrow from the memory cell into the reader's marker, with
    /// what it saw beside the marker; both fade.
    fn glance(&self, kit: &Kit, stage: &mut Stage<'_>, k: usize, from: Vec3, seen: &str, rt: f64) -> fmn::Result<()> {
        let to = self.at[k];
        let start = v(from[0], from[1] + if to[1] > from[1] { 0.42 } else { -0.42 });
        let end = v(to[0], to[1] + if to[1] > from[1] { -0.24 } else { 0.24 });
        let arrow = stage.arena_mut().add(access_arrow(start, end, k + 1)?);
        let tag = kit.text(stage, seen, 26.0, chalk(), v(to[0] + 0.5, to[1]))?;
        play!(stage;
            fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(rt * 0.45),
            fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(rt * 0.45),
        );
        stage.wait(rt * 0.25)?;
        play!(stage;
            fade_out(stage.arena_mut(), arrow, ORIGIN, 1.0)?.rt(rt * 0.3),
            fade_out(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(rt * 0.3),
        );
        Ok(())
    }

    /// Back to the start: letters fade, markers return, memory reads zero.
    fn reset(&mut self, kit: &Kit, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for k in 0..2 {
            for l in self.trays[k].drain(..) {
                let a = fade_out(stage.arena_mut(), l.chip, ORIGIN, 0.6)?;
                anims.push(stage.prepare(a.rt(rt))?);
            }
            let w = self.walk(k, station(k, 0), rt)?;
            anims.push(stage.prepare(w)?);
            if self.flags[k].raised {
                let t = self.flags[k].set(stage, false);
                anims.push(stage.prepare(t.rt(rt))?);
            }
            anims.extend(self.slates[k].rewrite(kit, stage, 0)?);
        }
        play_timed(stage, anims, rt)?;
        Ok(())
    }

    /// Replay `schedule` on `model`, each move `rt` seconds (`slow` steps,
    /// 1-based, take `slow_rt`).
    #[allow(clippy::too_many_arguments)]
    fn replay(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        model: &Tso,
        schedule: &[(usize, &str)],
        range: std::ops::RangeInclusive<usize>,
        rt: f64,
        slow: &[usize],
        slow_rt: f64,
    ) -> fmn::Result<()> {
        let (_, moves) = model.replay(schedule).map_err(fail)?;
        for (n, (&(p, _), &m)) in schedule.iter().zip(&moves).enumerate() {
            let step = n + 1;
            if !range.contains(&step) {
                continue;
            }
            let t = if slow.contains(&step) { slow_rt } else { rt };
            self.play_move(kit, stage, p - 1, m, t)?;
        }
        Ok(())
    }
}

/// The 12_7 panel, verbatim from the storyboard's code block (which the
/// description repeats and `bakery_stress` runs).
pub fn panel_code() -> Vec<&'static str> {
    let board = crate::STORYBOARD;
    let from = board.find("## 12_your_cpu").expect("the storyboard has chapter 12");
    board[from..]
        .lines()
        .skip_while(|l| !l.starts_with("```rust"))
        .skip(1)
        .take_while(|l| !l.starts_with("```"))
        .collect()
}

/// The panel as shown: the storyboard's lock without its `use` line and its
/// comments (the description has the whole code), and `unlock`'s one-line
/// body on its own line, so the type can be larger (R3 dev-04, maya-12).
pub fn screen_code(code: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for l in code {
        let t = l.trim_start();
        if t.starts_with("use ") || t.starts_with("//") {
            continue;
        }
        match t.strip_prefix("pub fn unlock").and_then(|_| t.split_once("{ ")) {
            Some((head, body)) => {
                out.push(format!("{head}{{"));
                out.push(format!("    {}", body.trim_end_matches('}').trim_end()));
                out.push("}".to_owned());
            }
            None => out.push((*l).to_owned()),
        }
    }
    out
}

/// The panel's lines bracketed by a fence: F1 from raising the flag to the
/// max's loads, F2 from the ticket store to the first flag check (dev-04).
pub fn fence_lines(code: &[&str]) -> fmn::Result<[(usize, usize); 2]> {
    let find = |pat: &str| {
        code.iter()
            .position(|l| l.contains(pat))
            .ok_or_else(|| fail(format!("the 12_7 panel has no line with {pat:?}")))
    };
    Ok([
        (find("choosing[i].store(true")?, find(".max()")?),
        (find("number[i].store(t")?, find("while choosing[j].load")?),
    ])
}

/// A card of left-aligned lines on the corpus's raised fill.
fn text_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    lines: &[(&str, f64, Srgb)],
    left: f64,
    top: f64,
    w: f64,
) -> fmn::Result<Mob> {
    let gap = 0.12;
    let heights: Vec<f64> = lines.iter().map(|(_, s, _)| s / 48.0 * 0.62 + gap).collect();
    let h: f64 = heights.iter().sum::<f64>() + 0.45;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(board(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(v(left + w / 2.0, top - h / 2.0));
    let mut parts = vec![stage.arena_mut().add(back)];
    let mut y = top - 0.25;
    for ((s, size, c), dh) in lines.iter().zip(&heights) {
        y -= dh / 2.0;
        parts.push(kit.text_left(stage, s, *size, *c, v(left + 0.3, y))?);
        y -= dh / 2.0;
    }
    group(stage, &parts)
}

/// 12_7's measured table: points lost by each kind of lock on both machines
/// (bead mx-e-c-7v0.17), with the sample size, the CPUs and Arm's longer
/// Release/Acquire run beneath. Detached, its top-left corner at
/// (`left`, `top`).
fn measured_table(
    kit: &Kit,
    stage: &mut Stage<'_>,
    arm: &Measured,
    x86: &Measured,
    left: f64,
    top: f64,
    w: f64,
) -> fmn::Result<Mob> {
    let rows = [
        (glyphs::RELAXED, "relaxed"),
        (glyphs::RELEASE_ACQUIRE, "relacq"),
        (glyphs::SEQCST, "seqcst"),
    ];
    let cols = [left + 0.6 * w, left + 0.84 * w];
    let row_h = 0.62;
    let visits = x86.mode("seqcst", 2).entries;
    let mut parts = Vec::new();
    let mut y = top - 0.45;
    parts.push(kit.text_left(
        stage,
        &format!(
            "{} {} {}, {} {}",
            glyphs::POINTS_LOST_IN,
            thousands(visits),
            glyphs::VISITS,
            x86.mode("seqcst", 2).threads,
            glyphs::CUSTOMERS
        ),
        26.0,
        Palette::dim(),
        v(left + 0.4, y),
    )?);
    y -= row_h;
    parts.push(kit.text(stage, glyphs::X86, 28.0, Palette::dim(), v(cols[0], y))?);
    parts.push(kit.text(stage, glyphs::ARM, 28.0, Palette::dim(), v(cols[1], y))?);
    for (name, mode) in rows {
        y -= row_h;
        parts.push(kit.text_left(stage, name, 30.0, Palette::ink(), v(left + 0.4, y))?);
        for (m, &x) in [x86, arm].into_iter().zip(&cols) {
            let r = m.mode(mode, 2);
            debug_assert_eq!(r.entries, visits, "every cell counts the same visits");
            // a zero is not a green light: only SeqCst's is promised by the
            // memory model (R3 dev-01, ruth-01), so zeros stay in plain ink
            let c = if r.lost > 0 { room_red() } else { Palette::ink() };
            parts.push(kit.text(stage, &thousands(r.lost), 34.0, c, v(x, y))?);
            if r.lost == 0 && mode != "seqcst" {
                parts.push(kit.text(stage, glyphs::NONE_SEEN, 17.0, Palette::dim(), v(x, y - 0.3))?);
            }
        }
    }
    let mut note = format!("{} {}. {} {}", glyphs::X86_COLON, x86.cpu, glyphs::ARM_COLON, arm.cpu);
    if let Some(long) = arm.long_relacq() {
        note.push_str(&format!(
            "{} {} {} {} {}",
            glyphs::LONGER_RUN_A,
            thousands(long.lost),
            glyphs::LONGER_RUN_B,
            thousands(long.entries),
            glyphs::LONGER_RUN_C
        ));
    }
    note.push_str(&format!(". {} {}", glyphs::ZERO_IS_NOT_PROOF, glyphs::X86_PLAIN_MOVES));
    y -= 0.3;
    for l in explainer_kit::animatic::wrap(&note, 76, 5) {
        y -= 0.38;
        parts.push(kit.text_left(stage, &l, 22.0, Palette::dim(), v(left + 0.4, y))?);
    }
    let h = top - y + 0.35;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(board(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(v(left + w / 2.0, top - h / 2.0));
    let mut all = vec![stage.arena_mut().add(back)];
    all.extend(parts);
    group(stage, &all)
}

pub struct YourCpu {
    pub kit: Kit,
}

impl SceneConstruct for YourCpu {
    fn name(&self) -> &str {
        "12_your_cpu"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (arm, x86) = (measured(Arch::Arm), measured(Arch::X86));

        // 12_1: the proof's promises return; the second breaks over a chip;
        // beside it, the failure measured on this machine
        let promises = lower_third(&kit, stage, glyphs::COUNTS_ON, glyphs::COUNTS_ON_TEXT)?;
        let chip = chip_outline(&kit, stage, v(-3.6, 0.2), 2.6, glyphs::CPU)?;
        let promise = kit.text(stage, glyphs::PROMISE, 32.0, Palette::ink(), v(-3.0, 2.55))?;
        let left = kit.text(stage, glyphs::PROMISE_LEFT, 32.0, Palette::ink(), ORIGIN)?;
        let right = kit.text(stage, glyphs::PROMISE_RIGHT, 32.0, Palette::ink(), ORIGIN)?;
        let (wl, wr) = (stage.get_width(left), stage.get_width(right));
        let gap = stage.get_width(promise) - wl - wr;
        let x0 = -3.0 - stage.get_width(promise) / 2.0;
        stage.move_to(left, v(x0 + wl / 2.0, 2.55), ORIGIN);
        stage.move_to(right, v(x0 + wl + gap + wr / 2.0, 2.55), ORIGIN);
        let (x86_relaxed, arm_relaxed) = (x86.mode("relaxed", 2), arm.mode("relaxed", 2));
        let card = text_card(
            &kit,
            stage,
            &[
                (glyphs::MEASURED_AS_WRITTEN, 26.0, Palette::dim()),
                (
                    &format!(
                        "{} {} {} {} {} {}",
                        arm_relaxed.threads,
                        glyphs::CUSTOMERS_DOT,
                        arm_relaxed.runs,
                        glyphs::RUNS_TIMES,
                        thousands(arm_relaxed.entries_per_run),
                        glyphs::VISITS
                    ),
                    24.0,
                    Palette::ink(),
                ),
                (&format!("{} {}", glyphs::X86_COLON, x86.cpu), 20.0, Palette::dim()),
                (
                    &format!("{} {}", thousands(x86_relaxed.lost), glyphs::POINTS_LOST),
                    34.0,
                    room_red(),
                ),
                (&format!("{} {}", glyphs::ARM_COLON, arm.cpu), 20.0, Palette::dim()),
                (
                    &format!("{} {}", thousands(arm_relaxed.lost), glyphs::POINTS_LOST),
                    34.0,
                    room_red(),
                ),
                (glyphs::RUST_RELAXED, 18.0, Palette::dim()),
            ],
            0.4,
            2.25,
            6.4,
        )?;
        kit.say(stage, "12_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), promises, v(0.0, 0.3), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), chip, ORIGIN, 0.9)?.rt(0.8),
        );
        play!(stage; fade_in(stage.arena_mut(), promise, v(0.0, -0.6), 1.0)?.rt(0.8));
        play!(stage; fade_in(stage.arena_mut(), card, v(0.3, 0.0), 1.0)?.rt(0.9));
        // the break lands on "isn't always seen"
        stage.wait(2.5)?;
        // flicker, then break: the halves part and tilt over a red crack
        for o in [0.25, 1.0, 0.35, 1.0] {
            let f = promise
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.12),
                    ..AnimateArgs::default()
                })?
                .set_opacity(o)?;
            stage.play(f)?;
        }
        stage.scene_mut().remove(&[promise]);
        stage.add_to_scene(left)?;
        stage.add_to_scene(right)?;
        let cx = x0 + wl + gap / 2.0;
        let crack = stage.arena_mut().add(
            polyline([
                v(cx - 0.05, 2.95),
                v(cx + 0.08, 2.72),
                v(cx - 0.07, 2.55),
                v(cx + 0.06, 2.36),
                v(cx - 0.04, 2.15),
            ])
            .style(line_style(room_red(), 4.0))
            .build(),
        );
        let split = |m: Mob, dx: f64, a: f64| -> fmn::Result<AnimBuilder> {
            Ok(m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.7),
                    ..AnimateArgs::default()
                })?
                .shift([dx, -0.12, 0.0])?
                .rotate(a)?
                .set_color(Palette::dim())?)
        };
        play!(stage;
            show_creation(crack).rt(0.35),
            split(left, -0.35, 0.05)?,
            split(right, 0.35, -0.05)?,
        );
        kit.hold(stage)?;

        // 12_2: cores, outboxes, memory
        let mut x = XRay::build(&kit, stage)?;
        let old = [promises, chip, left, right, crack, card];
        kit.say(stage, "12_2")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in old {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        let mut cores: Vec<Box<dyn Animation>> = Vec::new();
        for m in x.all() {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 0.95)?;
            cores.push(stage.prepare(a.rt(1.0))?);
        }
        stage.play_prepared(cores)?;
        let buffer_note = lower_third(&kit, stage, glyphs::STORE_BUFFER, glyphs::STORE_BUFFER_TEXT)?;
        play!(stage; fade_in(stage.arena_mut(), buffer_note, v(0.0, 0.2), 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 12_3: the voiced order, steps 1-8: everything lands in the outboxes
        let tso_model = Tso::new(2, 4, false, false);
        kit.say(stage, "12_3")?;
        play!(stage; fade_out(stage.arena_mut(), buffer_note, ORIGIN, 1.0)?.rt(0.5));
        x.replay(&kit, stage, &tso_model, &tso::VOICED, 1..=8, 0.75, &[], 0.75)?;
        let w0 = x.walk(0, station(0, 5), 0.5)?;
        let w1 = x.walk(1, station(1, 5), 0.5)?;
        play!(stage; w0, w1);
        let q = predict_pause(&kit, stage, v(5.65, 2.55))?;

        // 12_4: each sees no flag and no ticket; both walk in; memory still
        // reads zero everywhere
        kit.say(stage, "12_4")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        x.replay(&kit, stage, &tso_model, &tso::VOICED, 9..=12, 0.8, &[], 0.8)?;
        let (_, moves) = tso_model.replay(&tso::VOICED).map_err(fail)?;
        let steps = stacked(
            &kit,
            stage,
            &[
                glyphs::TSO_MODEL,
                &format!("{} {},", moves.len(), glyphs::MODEL_STEPS),
                glyphs::NOTHING_DELIVERED,
            ],
            22.0,
            ROOM_AT[0],
            ROOM_AT[1] - 1.9,
        )?;
        let mut zeros: Vec<Box<dyn Animation>> = Vec::new();
        for k in 0..2 {
            let a = fmn::animation::indicate(stage.arena_mut(), x.slates[k].board, 1.15, None)?;
            zeros.push(stage.prepare(a.rt(0.9))?);
            let b = fmn::animation::indicate(stage.arena_mut(), x.flags[k].pennant, 1.3, None)?;
            zeros.push(stage.prepare(b.rt(0.9))?);
        }
        let s = fade_in(stage.arena_mut(), steps, ORIGIN, 1.0)?;
        zeros.push(stage.prepare(s.rt(0.6))?);
        stage.play_prepared(zeros)?;
        // both in the room: their two tunes at once, the bug you can hear
        let [a, b] = in_room_after_voiced()?;
        let (wav, secs) = tunes::write("12_4", tunes::clash(a, b, CLASH_S)).map_err(|e| fail(e.to_string()))?;
        kit.sonify(stage, secs, "12_your_cpu:12_4", &wav)?;
        let t0 = stage.scene().time().to_f64();
        let mut both: Vec<Box<dyn Animation>> = Vec::new();
        for k in [a, b] {
            let m = x.markers[k - 1];
            let i = fmn::animation::indicate(stage.arena_mut(), m, 1.3, Some(rgb(room_red())))?;
            both.push(stage.prepare(i.rt(0.9))?);
        }
        stage.play_prepared(both)?;
        wait_until(stage, t0 + secs)?;
        kit.hold(stage)?;

        // 12_5: fences; the same race, both fenced, lets one in
        let x86_chip = machine_badge(&kit, stage, glyphs::X86, glyphs::INTEL_AMD)?;
        let fence_note = lower_third(
            &kit,
            stage,
            glyphs::X86_COLON,
            &format!(
                "{} {} {} ({}).",
                glyphs::FENCE_NOTE_A,
                crate::facts::TSO_CHECKED_CUSTOMERS,
                glyphs::FENCE_NOTE_B,
                crate::facts::cited::ABDULLA_TACAS
            ),
        )?;
        kit.say(stage, "12_5")?;
        play!(stage; fade_out(stage.arena_mut(), steps, ORIGIN, 1.0)?.rt(0.4));
        x.reset(&kit, stage, 0.7)?;
        let gates = x.add_fences(&kit, stage)?;
        let mut g: Vec<Box<dyn Animation>> = Vec::new();
        for m in gates.iter().copied().chain([x86_chip]) {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.1), 1.0)?;
            g.push(stage.prepare(a.rt(0.6))?);
        }
        let n = fade_in(stage.arena_mut(), fence_note, v(0.0, 0.2), 1.0)?;
        g.push(stage.prepare(n.rt(0.6))?);
        stage.play_prepared(g)?;
        let fenced = Tso::new(2, 4, true, true);
        x.replay(&kit, stage, &fenced, &FENCED, 1..=2, 0.42, &[], 0.6)?;
        // the first fence drains an outbox: the score answers with a whoosh
        kit.mark(stage, "whoosh:fence");
        x.replay(&kit, stage, &fenced, &FENCED, 3..=22, 0.42, &[3, 4, 13, 14, 15], 0.6)?;
        for f in 0..2 {
            let pulse: Vec<Box<dyn Animation>> = gates
                .chunks(2)
                .skip(f)
                .step_by(2)
                .map(|p| {
                    let a = fmn::animation::indicate(stage.arena_mut(), p[0], 1.6, None)?;
                    stage.prepare(a.rt(0.7))
                })
                .collect::<fmn::Result<_>>()?;
            stage.play_prepared(pulse)?;
        }
        kit.hold(stage)?;

        // 12_6: an outbox that may reorder: Blue's lowered flag lands before
        // her ticket, and Green walks in on it
        let arm_chip = machine_badge(&kit, stage, glyphs::ARM, glyphs::RECENT_MACS)?;
        let model_tag = stacked(
            &kit,
            stage,
            &[glyphs::REORDER_1, glyphs::REORDER_2, glyphs::REORDER_3],
            22.0,
            // its first line is wider than the room: kept inside the safe area
            ROOM_AT[0] - 0.15,
            ROOM_AT[1] - 1.9,
        )?;
        kit.say(stage, "12_6")?;
        play!(stage;
            fade_out(stage.arena_mut(), fence_note, ORIGIN, 1.0)?.rt(0.4),
            replacement_transform_anim(x86_chip, arm_chip),
        );
        x.reset(&kit, stage, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), model_tag, ORIGIN, 1.0)?.rt(0.5));
        let mut reorder = Tso::new(2, 4, true, true);
        reorder.reorder = true;
        x.replay(&kit, stage, &reorder, &REORDERED, 1..=22, 0.3, &[7, 17, 18], 0.85)?;
        kit.hold(stage)?;

        // 12_7: the SeqCst panel, F1 and F2 marked; what not to do; then the
        // panel slides away over an outbox draining
        // (the panel is built once the line starts: it waits off the frame's
        // right edge only for the instant before it slides in)
        let line = say(&kit, stage, "12_7")?;
        let shown = screen_code(&panel_code());
        let code: Vec<&str> = shown.iter().map(String::as_str).collect();
        let panel = CodePanel::build(&kit, stage, &code, PANEL_PT, PANEL_W)?;
        let [(f1a, f1b), (f2a, f2b)] = fence_lines(&code)?;
        let bx = PANEL_LEFT - 0.15;
        let mut marks = Vec::new();
        for (name, a, b) in [(glyphs::F1, f1a, f1b), (glyphs::F2, f2a, f2b)] {
            let (ya, yb) = (stage.get_center(panel.lines[a])[1], stage.get_center(panel.lines[b])[1]);
            let bracket = polyline([
                v(bx + 0.2, ya + 0.09),
                v(bx, ya + 0.09),
                v(bx, yb - 0.09),
                v(bx + 0.2, yb - 0.09),
            ])
            .style(line_style(flag_amber(), 3.5))
            .build();
            let bm = stage.arena_mut().add(bracket);
            let label = kit.text(stage, name, 24.0, flag_amber(), v(bx - 0.3, (ya + yb) / 2.0))?;
            marks.push((bm, label));
        }
        let ub = text_card(
            &kit,
            stage,
            &[
                (glyphs::UB_1, 22.0, Palette::ink()),
                (glyphs::UB_2, 22.0, Palette::ink()),
                (glyphs::UB_3, 22.0, Palette::ink()),
                (glyphs::UB_4, 22.0, Palette::ink()),
                (glyphs::UB_5, 22.0, Palette::ink()),
                (glyphs::UB_6, 22.0, Palette::ink()),
            ],
            -6.95,
            2.2,
            3.95,
        )?;
        // the shown panel leaves out the `use` line and the comments
        let full = {
            let at = v(FRAME_X_RADIUS - 0.4, stage.get_bottom(panel.group)[1] - 0.28);
            let m = kit.text(stage, glyphs::FULL_CODE, 22.0, Palette::dim(), at)?;
            stage.move_to(m, at, RIGHT);
            m
        };
        let table = measured_table(&kit, stage, &arm, &x86, -4.9, 2.6, 9.8)?;
        let cost_note = lower_third(&kit, stage, glyphs::FOR_PROGRAMMERS, glyphs::SEQCST_COST)?;
        // the x-ray steps back while the code is on screen
        let xray: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| *m != model_tag && *m != arm_chip)
            .collect();
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for &m in xray.iter().chain([&model_tag, &arm_chip]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            out.push(stage.prepare(a.rt(0.5))?);
        }
        stage.play_prepared(out)?;
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(1.0)?)?;
        let panel_in = stage.scene().time().to_f64();
        play!(stage; fade_in(stage.arena_mut(), full, ORIGIN, 1.0)?.rt(0.3));
        for (bm, label) in &marks {
            play!(stage;
                show_creation(*bm).rt(0.6),
                fade_in(stage.arena_mut(), *label, ORIGIN, 1.0)?.rt(0.6),
            );
            stage.wait(0.4)?;
        }
        play!(stage;
            fade_in(stage.arena_mut(), ub, v(0.2, 0.0), 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), cost_note, v(0.0, 0.2), 1.0)?.rt(0.7),
        );
        // the panel stays through its own sentence, six and a half seconds at
        // least; then, as the voice says why Acquire and release fall short,
        // the measured table takes the stage (the table belongs to that
        // sentence, and the chapter has no time for both after the line)
        line.wait_for(stage, "Acquire and release")?;
        wait_until(stage, panel_in + 6.5)?;
        // the panel slides a little way right as it fades, ending in frame
        let away = fade_out(stage.arena_mut(), panel.group, v(1.2, 0.0), 1.0)?;
        let mut out: Vec<Box<dyn Animation>> = vec![stage.prepare(away.rt(0.7))?];
        for m in [ub, full].into_iter().chain(marks.iter().flat_map(|(a, b)| [*a, *b])) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            out.push(stage.prepare(a.rt(0.5))?);
        }
        stage.play_prepared(out)?;
        play!(stage; fade_in(stage.arena_mut(), table, v(0.0, -0.2), 1.0)?.rt(0.7));
        // the chapter ends on what was measured (Codex R5: the closing drain
        // of 12_5's x-ray again cost the chapter its 85 s; 12_5 shows it)
        kit.hold(stage)?;
        stage.wait(0.2)?;
        fade_all(stage, 0.5)?;
        kit.finish(stage)
    }
}

/// 12_4's two tunes at once (s): the music lane's 3.0 s window, which holds
/// `tunes::clash`'s second tune (half a beat late) to its last note before
/// the score's 0.8 s fade.
const CLASH_S: f64 = 3.0;

/// The customers (1-based) in the room once the voiced order has run: both.
fn in_room_after_voiced() -> fmn::Result<[usize; 2]> {
    let m = Tso::new(2, 4, false, false);
    let (states, _) = m.replay(&tso::VOICED).map_err(fail)?;
    let last = states
        .last()
        .ok_or_else(|| fail("the voiced order is empty".to_owned()))?;
    let inside: Vec<usize> = (0..2)
        .filter(|&i| last.local[i].pc == tso::Pc::Room)
        .map(|i| i + 1)
        .collect();
    inside
        .try_into()
        .map_err(|v: Vec<usize>| fail(format!("12_4 needs both in the room, the model has {v:?}")))
}

/// `replacement_transform` as a value for `play!`.
fn replacement_transform_anim(a: Mob, b: Mob) -> Transform {
    fmn::animation::replacement_transform(a, b).rt(0.6)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_voiced_order_puts_both_in_the_room_for_12_4s_clash() {
        assert_eq!(super::in_room_after_voiced().unwrap(), [1, 2]);
    }

    use super::*;

    #[test]
    fn the_panel_is_the_storyboards_and_its_fences_bracket_the_right_lines() {
        let code = panel_code();
        assert!(code[0].starts_with("use std::sync::atomic"));
        assert!(code.iter().any(|l| l.contains("// each slate read once")));
        let [(a1, b1), (a2, b2)] = fence_lines(&code).unwrap();
        assert!(a1 < b1 && b1 < a2 && a2 < b2);
        assert!(code[a1 + 1].trim() == "// each slate read once");
        assert!(code[a2 + 1].contains("choosing[i].store(false"));
    }

    #[test]
    fn the_shown_panel_is_the_lock_without_its_use_line_and_comments() {
        let code = panel_code();
        let shown = screen_code(&code);
        let tokens =
            |ls: &mut dyn Iterator<Item = &str>| ls.flat_map(str::split_whitespace).collect::<Vec<_>>().join(" ");
        let kept = tokens(&mut code.iter().copied().filter(|l| {
            let t = l.trim_start();
            !t.starts_with("use ") && !t.starts_with("//")
        }));
        assert_eq!(tokens(&mut shown.iter().map(String::as_str)), kept);
        assert_eq!(shown.len(), code.len() - 4 + 2, "four lines dropped; unlock on three");
        assert!(shown.iter().all(|l| !l.contains("//") && !l.starts_with("use ")));
        // the fences bracket the same code lines
        let refs: Vec<&str> = shown.iter().map(String::as_str).collect();
        let [(a1, b1), (a2, b2)] = fence_lines(&refs).unwrap();
        assert_eq!((b1, a2), (a1 + 1, b1 + 1));
        assert!(refs[b2].contains("while choosing[j].load"));
        // unlock keeps its indentation
        let u = refs.iter().position(|l| l.starts_with("pub fn unlock")).unwrap();
        assert_eq!((refs[u + 1].trim(), refs[u + 2]), ("number[i].store(0, SeqCst)", "}"));
        assert!(refs[u + 1].starts_with("    "));
    }

    #[test]
    fn with_both_fences_the_same_race_lets_one_in() {
        let m = Tso::new(2, 4, true, true);
        let (states, _) = m.replay(&FENCED).unwrap();
        let end = states.last().unwrap();
        assert_eq!(m.in_room(end), 1);
        assert_eq!(end.local[0].pc, tso::Pc::Room);
        assert_eq!(end.local[1].pc, tso::Pc::WaitTicket);
        assert!(states.iter().all(|s| m.in_room(s) <= 1));
    }

    #[test]
    fn a_reordering_outbox_lets_green_in_on_blues_lowered_flag() {
        let mut ooo = Tso::new(2, 4, true, true);
        ooo.reorder = true;
        let (states, moves) = ooo.replay(&REORDERED).unwrap();
        assert_eq!(moves.len(), 22);
        assert_eq!(ooo.in_room(states.last().unwrap()), 2);
        // Green walks in while Blue's ticket is still in her outbox: no flag,
        // no ticket in memory (05_2's crack)
        let green_in = states.iter().position(|s| s.local[1].pc == tso::Pc::Room).unwrap();
        let s = &states[green_in];
        assert_eq!((s.flags[0], s.slates[0]), (0, 0));
        assert!(s.outbox[0].iter().any(|e| e.var == Var::Slate && e.value == 1));
        // the in-order outbox refuses the overtaking delivery
        assert!(Tso::new(2, 4, true, true).replay(&REORDERED).is_err());
    }

    #[test]
    fn every_station_and_slot_fits_the_frame() {
        for (k, &right) in TRAY_RIGHT.iter().enumerate() {
            for s in 0..7 {
                let p = station(k, s);
                assert!(p[0].abs() < 6.9 && p[1].abs() < 3.0);
            }
            for i in 0..SLOTS {
                assert!(slot(k, i)[0] > right - SLOT_W * SLOTS as f64);
            }
        }
        // read arrows never cross the reader's own tray: each customer's tray
        // sits over her own memory cells, away from the other's
        for (k, &right) in TRAY_RIGHT.iter().enumerate() {
            let other = 1 - k;
            let (lo, hi) = (right - SLOT_W * SLOTS as f64, right);
            for x in CELL_X[other] {
                assert!(x < lo || x > hi, "customer {k}'s tray covers x = {x}");
            }
        }
    }
}
