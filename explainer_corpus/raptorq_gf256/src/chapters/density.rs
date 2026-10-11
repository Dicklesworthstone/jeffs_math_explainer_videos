//! Chapter 14, the price of density: dense byte cards cost about K³/3 steps
//! (14_1); sparse cards become a graph of pieces and cards, with RFC 6330's
//! degrees (14_2); the five-piece toy peels C, A, B, D, E (14_3), and stalls
//! without its first card (14_4–14_5); the scene's own seeded runs of a
//! plain sparse code stall short of the picture, the hook's picture with
//! holes in it (14_6–14_7). (14_8, Reed–Solomon against a fountain for
//! programmers, is in the description: it kept the chapter over 85 s.)
//!
//! The toy graph is built here and reused by chapter 16 (`toy_graph`).

use std::cell::Cell;
use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{RowStrips, grow_histogram, histogram, lower_third, precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, hex, play, sound_file, thousands, tween, v};
use explainer_score::sonify::{Grain, cloud};
use fmn::prelude::*;

use super::draw::{OPLUS, Role, badge, card, flat, group_of, line_style};
use super::picture::{SIDE, cell};
use super::voice::{Spoken, board};
use crate::decoding::{
    Affine, DECODE_MS_8_BYTES, DECODE_MS_1280_BYTES, DECODE_PIECES, DEGREE_TABLE, DENSE_PIECES, PEEL_EXTRA_PERCENT,
    PEEL_PIECES, PeelRun, PeelStep, STEPS_PER_SECOND, TOY_CARDS, degree_mass, dense_steps, dense_time, mean_degree,
    peel_runs, peel_toy,
};

/// Recorded decode times (ms) as 14_1 prints them, in seconds to one digit:
/// "0.1", or "0.2–0.3" when the runs round apart.
pub fn seconds_shown(ms: &[f64]) -> String {
    let s = |x: f64| super::byte_lottery::sig_digits(x / 1000.0, 1);
    let lo = s(ms.iter().copied().fold(f64::INFINITY, f64::min));
    let hi = s(ms.iter().copied().fold(0.0, f64::max));
    if lo == hi { lo } else { format!("{lo}–{hi}") }
}
use crate::model::{pieces_of, result};

const KEY: &str = "14_the_price_of_density";

/// A dark cell: a piece the phone does not have.
fn hole() -> Srgb {
    hex("#0B0E13")
}

/// An animation of `m` to colour `c` over `rt` seconds.
fn recolor(m: Mob, c: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_color(c)?)
}

/// An animation of `m` to opacity `o` over `rt` seconds.
fn opacity(m: Mob, o: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(o)?)
}

/// A value the toy's peel has found, as math: `7`, or `2 ⊕ c` with C parked.
pub fn affine_tex(a: Affine) -> String {
    if a.with_c {
        format!(r"{} {OPLUS} c", a.constant)
    } else {
        a.constant.to_string()
    }
}

// ------------------------------------------------------------ the toy graph

/// The five-piece toy as a graph: the pieces' badges on top, one dot per
/// card below, a line from each card to each piece it mixes, and each card's
/// result under its dot (14_2–14_5, 16).
pub struct ToyGraph {
    pub group: Mob,
    pub badges: Vec<Mob>,
    pub piece_at: Vec<Vec3>,
    /// Indexed by card (`TOY_CARDS`); only the cards built are present.
    pub dots: Vec<Option<Mob>>,
    pub dot_at: Vec<Vec3>,
    pub results: Vec<Option<Mob>>,
    /// (card, piece, line).
    pub edges: Vec<(usize, usize, Mob)>,
    /// Its size: 1 is the graph beside 14_2's histogram.
    pub scale: f64,
}

/// Badge radius, dot radius and the rows' heights from the graph's centre.
const BADGE_R: f64 = 0.3;
const DOT_R: f64 = 0.15;
const PIECE_DY: f64 = 1.3;
const CARD_DY: f64 = -1.2;

/// The graph's size when it is the picture (14_3–14_5, 16_1).
pub const GRAPH_BIG: f64 = 1.45;

/// Where piece `i` and card `j` sit in a graph centred at `centre`, of
/// size `k`.
pub fn piece_pos(centre: Vec3, i: usize, k: f64) -> Vec3 {
    v(centre[0] + (i as f64 - 2.0) * 1.3 * k, centre[1] + PIECE_DY * k)
}
pub fn dot_pos(centre: Vec3, j: usize, k: f64) -> Vec3 {
    // left to right: A ⊕ B, A ⊕ C, C, B ⊕ C ⊕ D, D ⊕ E, B ⊕ E
    const SLOT: [usize; 6] = [2, 1, 3, 0, 4, 5];
    v(centre[0] + (SLOT[j] as f64 - 2.5) * 1.1 * k, centre[1] + CARD_DY * k)
}

/// The graph of `cards` (indices into `TOY_CARDS`) at size `k`, detached.
pub fn toy_graph(kit: &Kit, stage: &mut Stage<'_>, centre: Vec3, k: f64, cards: &[usize]) -> fmn::Result<ToyGraph> {
    let piece_at: Vec<Vec3> = (0..5).map(|i| piece_pos(centre, i, k)).collect();
    let dot_at: Vec<Vec3> = (0..TOY_CARDS.len()).map(|j| dot_pos(centre, j, k)).collect();
    let mut all = Vec::new();
    let mut edges = Vec::new();
    for &j in cards {
        for p in pieces_of(TOY_CARDS[j]) {
            let a = v(dot_at[j][0], dot_at[j][1] + DOT_R * k);
            let b = v(piece_at[p][0], piece_at[p][1] - (BADGE_R + 0.08) * k);
            let line = Line::new(a, b)
                .style(
                    Style::default()
                        .fill(Role::card(), 0.0)
                        .stroke(Role::card(), 2.5 * k, 0.7),
                )
                .build()?;
            let m = stage.arena_mut().add(line);
            edges.push((j, p, m));
            all.push(m);
        }
    }
    let mut badges = Vec::new();
    for (i, &at) in piece_at.iter().enumerate() {
        let b = badge(kit, stage, i, BADGE_R * k, at)?;
        badges.push(b);
        all.push(b);
    }
    let (mut dots, mut results) = (vec![None; TOY_CARDS.len()], vec![None; TOY_CARDS.len()]);
    for &j in cards {
        let d = Circle::new()
            .radius(DOT_R * k)
            .arc_center(dot_at[j])
            .style(flat(Role::card(), 1.0))
            .build();
        let d = stage.arena_mut().add(d);
        let r = kit.tex(
            stage,
            &result(TOY_CARDS[j]).to_string(),
            34.0 * k,
            Palette::ink(),
            &[],
            v(dot_at[j][0], dot_at[j][1] - 0.45 * k),
        )?;
        dots[j] = Some(d);
        results[j] = Some(r);
        all.extend([d, r]);
    }
    let group = group_of(stage, &all)?;
    Ok(ToyGraph {
        group,
        badges,
        piece_at,
        dots,
        dot_at,
        results,
        edges,
        scale: k,
    })
}

/// A peel in progress on a [`ToyGraph`]: what each card still holds, the
/// rings that mark the ripple (cards with one unknown left), the values found.
pub struct Peeling {
    /// Per card: its remaining (piece, line).
    left: Vec<Vec<(usize, Mob)>>,
    /// Per card: its current right side.
    value: Vec<Affine>,
    /// Per card: the label of its right side on screen now.
    pub labels: Vec<Option<Mob>>,
    rings: Vec<Option<Mob>>,
    ring_shown: Vec<bool>,
    /// Per piece: its value label, once found.
    pub found: Vec<Option<Mob>>,
}

impl Peeling {
    /// Start on `g`, whose cards are `cards`, with `parked` moved to the
    /// right side as c.
    pub fn new(stage: &mut Stage<'_>, g: &ToyGraph, cards: &[usize], parked: Option<usize>) -> fmn::Result<Self> {
        let k = g.scale;
        let n = TOY_CARDS.len();
        let mut left = vec![Vec::new(); n];
        for &(j, p, m) in &g.edges {
            if Some(p) != parked {
                left[j].push((p, m));
            }
        }
        let mut value = vec![
            Affine {
                constant: 0,
                with_c: false
            };
            n
        ];
        let mut rings = vec![None; n];
        for &j in cards {
            let card = TOY_CARDS[j];
            value[j] = Affine {
                constant: result(card),
                with_c: parked.is_some_and(|p| card >> p & 1 == 1),
            };
            let ring = Circle::new()
                .radius((DOT_R + 0.13) * k)
                .arc_center(g.dot_at[j])
                .style(Style::default().fill(Role::lit(), 0.18).stroke(Role::lit(), 4.0, 1.0))
                .build();
            rings[j] = Some(stage.arena_mut().add(ring));
        }
        Ok(Self {
            left,
            value,
            labels: g.results.clone(),
            rings,
            ring_shown: vec![false; n],
            found: vec![None; 5],
        })
    }

    /// Show the rings of the cards with exactly one unknown left (and hide
    /// the others), over `rt` seconds.
    pub fn show_ripple(&mut self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for j in 0..self.left.len() {
            let Some(ring) = self.rings[j] else { continue };
            let want = self.left[j].len() == 1;
            if want && !self.ring_shown[j] {
                let a = fade_in(stage.arena_mut(), ring, ORIGIN, 0.6)?;
                anims.push(stage.prepare(a.rt(rt))?);
            } else if !want && self.ring_shown[j] {
                let a = fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
            }
            self.ring_shown[j] = want;
        }
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// Hide every ring.
    pub fn clear_ripple(&mut self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for j in 0..self.rings.len() {
            if let (Some(ring), true) = (self.rings[j], self.ring_shown[j]) {
                let a = fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(rt))?);
                self.ring_shown[j] = false;
            }
        }
        stage.play_prepared(anims)?;
        Ok(())
    }

    /// The card with one unknown left gives its piece away: the card and its
    /// line light, the piece lights and shows its value (`rt` seconds).
    pub fn give(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        g: &ToyGraph,
        step: &PeelStep,
        rt: f64,
    ) -> fmn::Result<()> {
        let (_, line) = self.left[step.card][0];
        let dot = g.dots[step.card].expect("a built card");
        let label = kit.tex(
            stage,
            &affine_tex(step.value),
            34.0 * g.scale,
            Role::lit(),
            &[],
            v(
                g.piece_at[step.piece][0],
                g.piece_at[step.piece][1] + (BADGE_R + 0.32) * g.scale,
            ),
        )?;
        self.found[step.piece] = Some(label);
        play!(stage;
            recolor(dot, Role::lit(), rt * 0.5)?,
            recolor(line, Role::lit(), rt * 0.5)?,
        );
        play!(stage;
            recolor(g.badges[step.piece], Role::lit(), rt * 0.5)?,
            fade_in(stage.arena_mut(), label, v(0.0, -0.15), 1.0)?.rt(rt * 0.5),
        );
        Ok(())
    }

    /// Remove the piece everywhere: its lines fade, and every card that used
    /// it moves it to the right side (the result changes); the spent card
    /// dims (`rt` seconds).
    pub fn remove(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        g: &ToyGraph,
        step: &PeelStep,
        rt: f64,
    ) -> fmn::Result<()> {
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for j in 0..self.left.len() {
            let Some(at) = self.left[j].iter().position(|&(p, _)| p == step.piece) else {
                continue;
            };
            let (_, line) = self.left[j].remove(at);
            let a = fade_out(stage.arena_mut(), line, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(rt))?);
            if j == step.card {
                continue;
            }
            let v_new = Affine {
                constant: self.value[j].constant ^ step.value.constant,
                with_c: self.value[j].with_c ^ step.value.with_c,
            };
            self.value[j] = v_new;
            let old = self.labels[j].expect("a built card");
            let at = stage.get_center(old);
            let new = kit.tex(stage, &affine_tex(v_new), 34.0 * g.scale, Palette::ink(), &[], at)?;
            let swap = fmn::animation::replacement_transform(old, new);
            anims.push(stage.prepare(swap.rt(rt))?);
            self.labels[j] = Some(new);
        }
        let dot = g.dots[step.card].expect("a built card");
        let a = opacity(dot, 0.3, rt)?;
        anims.push(stage.prepare(a)?);
        let r = self.labels[step.card].expect("a built card");
        let a = opacity(r, 0.3, rt)?;
        anims.push(stage.prepare(a)?);
        stage.play_prepared(anims)?;
        Ok(())
    }
}

// ------------------------------------------------------------- the pictures

/// 14_1's dense card deck: `n` × `n` gold cells, every card using every
/// piece with its own byte amount (a cell's strength is its amount).
fn dense_deck(n: usize, side: f64, centre: Vec3) -> fmn::Result<VMobject> {
    let step = side / n as f64;
    let mut cells = Vec::with_capacity(n * n);
    for r in 0..n {
        for c in 0..n {
            // an amount per cell, from the field itself: no two rows alike
            let amount = crate::model::mul(((r * n + c) % 255 + 1) as u8, 29, crate::model::FOLD);
            let o = 0.25 + 0.7 * f64::from(amount) / 255.0;
            let x = centre[0] - side / 2.0 + step * c as f64;
            let y = centre[1] + side / 2.0 - step * (r + 1) as f64;
            cells.push(explainer_kit::rect(
                v(x + 0.01, y + 0.01),
                step - 0.02,
                step - 0.02,
                flat(Role::byte(), o),
            )?);
        }
    }
    Ok(v_group(cells))
}

/// A share of a day as a filled sector of radius `r` at `centre`, the
/// share read from `tracker` every frame (a small shape, so redrawn).
fn day_pie(stage: &mut Stage<'_>, tracker: Mob, centre: Vec3, r: f64) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let share = s.tracker_value(tracker).unwrap_or(0.0).clamp(0.0, 1.0);
        let n = 64;
        let mut pts = vec![centre];
        for k in 0..=n {
            // clockwise from twelve o'clock, like a clock face
            let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * share * k as f64 / n as f64;
            pts.push(v(centre[0] + r * a.cos(), centre[1] + r * a.sin()));
        }
        s.add(Polygon::new(pts).style(flat(Role::byte(), 0.85)).build())
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// The colours of the picture as a phone holds it after `got` peeling steps
/// of `run`: recovered pieces in their colours, the rest dark, and (with
/// `untouched`) the pieces no card mixed in red.
fn held(run: &PeelRun, rank: &[u32], got: usize, untouched: bool) -> Vec<Vec<Srgb>> {
    let mut colors: Vec<Vec<Srgb>> = (0..SIDE)
        .map(|r| {
            (0..SIDE)
                .map(|c| {
                    if (rank[r * SIDE + c] as usize) < got {
                        cell(r, c)
                    } else {
                        hole()
                    }
                })
                .collect()
        })
        .collect();
    if untouched {
        for &i in &run.untouched {
            let i = i as usize;
            colors[i / SIDE][i % SIDE] = Role::lost();
        }
    }
    colors
}

/// The pluck of each peeled piece in 14_3's listen window: one grain per
/// step, at `at` seconds, down the E pentatonic scale from `top` (MIDI).
fn plucks(at: &[f64], top: f64, seconds: f64) -> std::io::Result<std::path::PathBuf> {
    // E pentatonic: E, C♯, B, G♯, F♯ downwards (semitones below E)
    const DOWN: [f64; 5] = [0.0, 3.0, 5.0, 8.0, 10.0];
    let grains: Vec<Grain> = at
        .iter()
        .enumerate()
        .map(|(k, &t)| Grain {
            start: t,
            pitch: top - DOWN[k % DOWN.len()] - 12.0 * (k / DOWN.len()) as f64,
            pan: 0.0,
            amp: 0.35,
        })
        .collect();
    sound_file("14_3_peel_plucks", &cloud(&grains, seconds))
}

pub struct Density {
    pub kit: Kit,
}

impl SceneConstruct for Density {
    fn name(&self) -> &str {
        KEY
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let board = board(KEY)?;
        self.dense(&kit, stage, &board)?;
        self.peel(&kit, stage, &board)?;
        self.stall(&kit, stage, &board)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl Density {
    /// 14_1: K³/3 steps for dense byte cards, counted up; half a day.
    fn dense(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let k = DENSE_PIECES;
        let steps = dense_steps(k);
        let deck = stage.arena_mut().add(dense_deck(14, 3.4, v(-4.3, 0.55))?);
        let deck_label = kit.text(
            stage,
            &format!("{} cards × {} pieces, all byte amounts", thousands(k), thousands(k)),
            24.0,
            Palette::dim(),
            v(-4.3, -1.55),
        )?;
        let formula = kit.tex(
            stage,
            r"\text{steps} \approx \frac{K^3}{3}",
            60.0,
            Palette::ink(),
            &[("K", Role::byte())],
            v(2.4, 2.6),
        )?;
        let k_is = kit.tex(
            stage,
            &format!(r"K = {}", thousands(k).replace(',', "{,}")),
            40.0,
            Role::byte(),
            &[],
            v(2.4, 1.6),
        )?;
        let count = stage.add_value_tracker(0.0);
        let mantissa = steps / 10f64.powi(steps.log10().floor() as i32);
        let approx = kit.tex(
            stage,
            &format!(
                r"\approx {:.1} \times 10^{{{}}}\ \text{{steps}}",
                mantissa,
                steps.log10().floor() as i32
            ),
            40.0,
            Palette::dim(),
            &[],
            v(2.4, 0.2),
        )?;
        let hours = dense_time(k, STEPS_PER_SECOND, 3600);
        let days = dense_time(k, STEPS_PER_SECOND, 86_400);
        let pie_at = v(0.75, -1.3);
        let pie_r = 0.85;
        let rim = Circle::new()
            .radius(pie_r)
            .arc_center(pie_at)
            .style(line_style(Palette::dim(), 2.5))
            .build();
        let rim = stage.arena_mut().add(rim);
        let day_word = kit.text(
            stage,
            "one day",
            28.0,
            Palette::dim(),
            v(pie_at[0], pie_at[1] - pie_r - 0.3),
        )?;
        let at_speed = kit.tex_left(
            stage,
            &format!(
                r"\approx {:.1}\ \text{{h at}}\ 10^{{{}}}\ \text{{steps/s}}",
                hours,
                (STEPS_PER_SECOND as f64).log10().round() as i32
            ),
            46.0,
            Palette::ink(),
            &[],
            v(1.95, -1.3),
        )?;
        let note = lower_third(
            kit,
            stage,
            "for programmers:",
            &format!(
                "even ten times faster, still ≈ {:.1} h; the matrix alone is {:.1} GB; double the pieces, {}× the \
                 work. Measured: the raptorq 2.0.0 decoder rebuilds a {}-piece block in about {} s on one core \
                 ({} s with 1,280-byte pieces)",
                dense_time(k, STEPS_PER_SECOND * 10, 3600),
                (k * k) as f64 / 1e9,
                (2 * k).pow(3) / k.pow(3),
                thousands(u64::from(DECODE_PIECES)),
                seconds_shown(&DECODE_MS_8_BYTES),
                seconds_shown(&DECODE_MS_1280_BYTES),
            ),
        )?;

        let line = Spoken::say(kit, stage, board, "14_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), deck, ORIGIN, 0.9)?.rt(1.2).lag(0.02),
            fade_in(stage.arena_mut(), deck_label, ORIGIN, 1.0)?.rt(1.0),
        );
        line.at(stage, 1)?;
        play!(stage; write(stage.arena(), formula).rt(1.2));
        line.at(stage, 2)?;
        play!(stage; fade_in(stage.arena_mut(), k_is, v(0.0, -0.15), 1.0)?.rt(0.6));
        let live = kit.live_count(stage, count, 58.0, Role::byte(), v(2.4, 0.9))?;
        let up = count
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(2.4),
                rate_func: Some(fmn::core::rate::rush_into),
                ..AnimateArgs::default()
            })?
            .set_value(steps.round())?;
        stage.play(up)?;
        play!(stage;
            fade_in(stage.arena_mut(), approx, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        line.at(stage, 3)?;
        let share = stage.add_value_tracker(0.0);
        play!(stage;
            fade_in(stage.arena_mut(), rim, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), day_word, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), at_speed, ORIGIN, 1.0)?.rt(0.8),
        );
        let pie = day_pie(stage, share, pie_at, pie_r)?;
        stage.play(tween(share, days, 1.6)?)?;
        stage.clear_updaters(pie, true);
        stage.clear_updaters(live, true);
        Ok(())
    }
}

impl Density {
    /// 14_2–14_5: sparse cards as a graph, RFC 6330's degrees, the peel, and
    /// the stall without the first card.
    fn peel(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let all: Vec<usize> = (0..TOY_CARDS.len()).collect();
        // 14_2: the six cards, then each becomes a dot joined to its pieces
        let left = v(-3.35, 0.25);
        let mut cards = Vec::new();
        for (j, &c) in TOY_CARDS.iter().enumerate() {
            let at = v(-4.55 + 4.55 * (j % 3) as f64, if j < 3 { 1.2 } else { -0.4 });
            cards.push(card(kit, stage, c, Some(result(c)), 0.62, at)?);
        }
        let line = Spoken::say(kit, stage, board, "14_2")?;
        fade_all(stage, 0.6)?;
        // why not small dense blocks (R3 Dev D6)
        let small = lower_third(
            kit,
            stage,
            "for programmers:",
            "why not small dense blocks? Each recovered byte then costs a byte multiply-add per piece of its block, \
             and a phone must finish every block, so its unluckiest block sets the pace",
        )?;
        play!(stage; fade_in(stage.arena_mut(), small, v(0.0, 0.2), 1.0)?.rt(0.6));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for k in &cards {
            let a = fade_in(stage.arena_mut(), k.group, v(0.0, -0.2), 1.0)?;
            anims.push(stage.prepare(a.rt(0.8))?);
        }
        let group = explainer_kit::lagged(stage, anims, 0.12)?;
        stage.play(group)?;
        line.at(stage, 1)?;
        let g = toy_graph(kit, stage, left, 1.0, &all)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (j, k) in cards.iter().enumerate() {
            let to = g.dot_at[j];
            let a = k
                .group
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?
                .scale(0.12)?
                .set_opacity(0.0)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        for k in &cards {
            stage.scene_mut().remove(&[k.group]);
        }
        play!(stage; fade_in(stage.arena_mut(), g.group, ORIGIN, 1.0)?.rt(0.9));
        line.at(stage, 2)?;
        // RFC 6330's degrees: how many pieces a card mixes
        let frame = Frame2 {
            center: v(3.75, 0.35),
            width: 5.6,
            height: 3.0,
            x: (0.4, 30.6),
            y: (0.0, 0.55),
        };
        let bars: Vec<(f64, f64)> = (1..DEGREE_TABLE.len()).map(|d| (d as f64, degree_mass(d))).collect();
        let hist = stage.arena_mut().add(histogram(&frame, &bars, 0.72, Role::ex_or())?);
        let axis = Line::new(frame.p(0.4, 0.0), frame.p(30.6, 0.0))
            .style(line_style(Palette::dim(), 2.0))
            .build()?;
        let axis = stage.arena_mut().add(axis);
        let mut ticks = Vec::new();
        for d in [1usize, 10, 20, 30] {
            let p = frame.p(d as f64, 0.0);
            ticks.push(kit.text(stage, &d.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.25))?);
        }
        let title = kit.text(
            stage,
            "pieces a card mixes",
            26.0,
            Palette::dim(),
            v(frame.center[0], frame.bottom() - 0.7),
        )?;
        let top = frame.p(2.0, degree_mass(2));
        let half = kit.text(
            stage,
            &format!("{}% mix two", (degree_mass(2) * 100.0).round()),
            26.0,
            Role::ex_or(),
            v(top[0] + 1.25, top[1] - 0.05),
        )?;
        let mean = mean_degree();
        let mean_line = DashedLine::new(frame.p(mean, 0.0), frame.p(mean, 0.32))
            .dash_length(0.08)
            .style(line_style(Palette::ink(), 2.0))
            .build()?;
        let mean_line = stage.arena_mut().add(mean_line);
        let mean_at = frame.p(mean, 0.32);
        let mean_label = kit.text(
            stage,
            &format!("mean {mean:.2}"),
            24.0,
            Palette::ink(),
            v(mean_at[0] + 0.75, mean_at[1] + 0.12),
        )?;
        let ticks_g = group_of(stage, &ticks)?;
        play!(stage;
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), ticks_g, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.5),
        );
        stage.add_to_scene(hist)?;
        let grow = grow_histogram(stage, hist, 0.06)?;
        stage.play(grow.with_run_time(1.6))?;
        play!(stage; fade_in(stage.arena_mut(), half, v(-0.15, 0.0), 1.0)?.rt(0.5));
        play!(stage;
            show_creation(mean_line).rt(0.6),
            fade_in(stage.arena_mut(), mean_label, ORIGIN, 1.0)?.rt(0.6),
        );
        let last = DEGREE_TABLE.len() - 2;
        let note = precisely(
            kit,
            stage,
            &format!("RFC 6330's degrees: P(d) ≈ 1/(d(d−1)) for d = 2…{last}, the ideal soliton; mean {mean:.2}"),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), small, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );

        // 14_3: peeling, a chain reaction; the graph moves to the middle
        let line = Spoken::say(kit, stage, board, "14_3")?;
        let degree_mobs = [hist, axis, ticks_g, title, half, mean_line, mean_label, note];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in degree_mobs {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        // the graph becomes the picture: centred, and grown to GRAPH_BIG
        let centre = v(0.0, 0.2);
        let a = g
            .group
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .shift([centre[0] - left[0], centre[1] - left[1], 0.0])?
            .scale_about(GRAPH_BIG, Some(centre), None)?;
        stage.play(a)?;
        let g = ToyGraph {
            piece_at: (0..5).map(|i| piece_pos(centre, i, GRAPH_BIG)).collect(),
            dot_at: (0..TOY_CARDS.len()).map(|j| dot_pos(centre, j, GRAPH_BIG)).collect(),
            scale: GRAPH_BIG,
            ..g
        };
        let (steps, _) = peel_toy(&all, None);
        let mut peeling = Peeling::new(stage, &g, &all, None)?;
        // each dot is a card, and each line goes to a piece it uses (R3
        // Maya maya-17): the leftmost dot, B ⊕ C ⊕ D, and its piece B
        line.at(stage, 1)?;
        let (dot, piece) = (g.dot_at[3], g.piece_at[1]);
        let card_word = kit.text(stage, "a card", 28.0, Palette::ink(), ORIGIN)?;
        stage.move_to(card_word, v(dot[0] - 0.3, dot[1]), RIGHT);
        let piece_word = kit.text(stage, "a piece", 28.0, Palette::ink(), v(piece[0], piece[1] + 0.62))?;
        play!(stage; fade_in(stage.arena_mut(), card_word, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; fade_in(stage.arena_mut(), piece_word, ORIGIN, 1.0)?.rt(0.5));
        line.at(stage, 2)?;
        play!(stage;
            fade_out(stage.arena_mut(), card_word, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), piece_word, ORIGIN, 1.0)?.rt(0.4),
        );
        peeling.show_ripple(stage, 0.5)?;
        peeling.give(kit, stage, &g, &steps[0], 1.0)?;
        line.at(stage, 3)?;
        peeling.remove(kit, stage, &g, &steps[0], 1.0)?;
        peeling.show_ripple(stage, 0.5)?;
        line.at(stage, 4)?;
        let name = kit.text_styled(stage, "peeling", 48.0, Role::lit(), v(-5.3, 3.4), false, true)?;
        play!(stage; write(stage.arena(), name).rt(0.8));
        let note = lower_third(
            kit,
            stage,
            "for programmers:",
            "per piece, a list of the cards that use it; plus a queue of cards with one unknown left (the ripple)",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        // the listen window: the rest of the cascade, a pluck per piece
        let listen = board
            .beats
            .iter()
            .find(|b| b.id == "14_3")
            .and_then(explainer_kit::storyboard::Beat::listen_s)
            .unwrap_or(3.0);
        let per = listen / (steps.len() - 1) as f64;
        let starts: Vec<f64> = (0..steps.len() - 1).map(|k| per * k as f64 + 0.05).collect();
        // C's pluck was a mark under the voice; the window plucks the rest
        let wav = plucks(&starts, 73.0, listen).map_err(|e| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
                format!("14_3 plucks: {e}").into_boxed_str(),
            )))
        })?;
        kit.sonify(stage, listen, "14_3", &wav)?;
        for step in &steps[1..] {
            peeling.give(kit, stage, &g, step, per * 0.45)?;
            peeling.remove(kit, stage, &g, step, per * 0.35)?;
            peeling.show_ripple(stage, per * 0.2)?;
        }
        peeling.clear_ripple(stage, 0.3)?;

        // 14_4: start again, and lose the card that starts it
        let g = toy_graph(kit, stage, centre, GRAPH_BIG, &all)?;
        let first = steps[0].card;
        let first_dot = g.dots[first].expect("built");
        let first_label = g.results[first].expect("built");
        let first_edges: Vec<Mob> = g.edges.iter().filter(|e| e.0 == first).map(|e| e.2).collect();
        let line = Spoken::say(kit, stage, board, "14_4")?;
        fade_all(stage, 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), g.group, ORIGIN, 1.0)?.rt(0.4));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in first_edges.iter().chain([&first_dot, &first_label]) {
            let a = recolor(m, Role::lost(), 0.6)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        line.at_share(stage, 0.45)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in first_edges.iter().chain([&first_dot, &first_label]) {
            let a = fade_out(stage.arena_mut(), m, v(0.0, -0.3), 1.0)?;
            anims.push(stage.prepare(a.rt(0.8))?);
        }
        stage.play_prepared(anims)?;
        let q = predict_pause(kit, stage, v(5.6, -2.6))?;

        // 14_5: nothing moves; no card has one unknown left
        let rest: Vec<usize> = all.iter().copied().filter(|&j| j != first).collect();
        let (moves, _) = peel_toy(&rest, None);
        let verdict = if moves.is_empty() {
            "no card has one unknown left"
        } else {
            "a card still has one unknown left"
        };
        let stuck = kit.text(stage, verdict, 32.0, Palette::dim(), v(0.0, -3.1))?;
        // the music stops with the graph: an authored silence from "Nothing
        // moves." through the stall (the score fades it in the pause before)
        kit.mark(stage, "silence:stall");
        kit.say(stage, "14_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), stuck, ORIGIN, 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;
        // the frozen graph holds while the music stops (storyboard: 1.5 s)
        explainer_kit::narration::wait_frames(
            stage,
            explainer_kit::narration::frames_covering(1.5, stage.scene().time().fps()),
        )?;
        kit.mark(stage, "silence_end:stall");
        Ok(())
    }
}

/// The stall bars' layout: left edge, width, height, and the tops of the
/// two groups.
const BAR_X: f64 = -4.4;
const BAR_W: f64 = 8.6;
const BAR_H: f64 = 0.3;
const BAR_GAP: f64 = 0.43;

impl Density {
    /// 14_6–14_7: the scene's own seeded runs stall; the picture with holes,
    /// and the pieces no card mixed.
    fn stall(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let runs = peel_runs();
        let k = PEEL_PIECES;
        let groups = [(0usize, 2.75), (1usize, 0.0)];
        let caption = kit.text(
            stage,
            &format!(
                "{} pieces, the standard's degree table, {} seeded runs",
                thousands(k as u64),
                runs[0].len()
            ),
            24.0,
            Palette::dim(),
            v(0.0, -3.45),
        )?;
        let line = Spoken::say(kit, stage, board, "14_6")?;
        fade_all(stage, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.6));
        let mut keep = Vec::new();
        let mut extra_top = 0.0;
        for (g, top) in groups {
            if g == 1 {
                line.at(stage, 1)?;
                extra_top = top;
            }
            let pct = PEEL_EXTRA_PERCENT[g];
            let cards = runs[g][0].cards;
            let title = if pct == 0 {
                format!("exactly enough: {} cards", thousands(cards as u64))
            } else {
                format!("{pct}% extra: {} cards", thousands(cards as u64))
            };
            let head = kit.text_left(stage, &title, 28.0, Role::ex_or(), v(BAR_X, top))?;
            play!(stage; fade_in(stage.arena_mut(), head, ORIGIN, 1.0)?.rt(0.5));
            let t = stage.add_value_tracker(0.0);
            let mut live = Vec::new();
            for (i, run) in runs[g].iter().enumerate() {
                let y = top - 0.5 - BAR_GAP * i as f64;
                let back = explainer_kit::rect(v(BAR_X, y - BAR_H / 2.0), BAR_W, BAR_H, flat(Palette::faint(), 0.6))?;
                let back = stage.arena_mut().add(back);
                stage.add_to_scene(back)?;
                let got = run.recovered() as f64;
                let bar = stage.always_redraw(move |s| {
                    let done = s.tracker_value(t).unwrap_or(0.0).min(got);
                    let w = (BAR_W * done / k as f64).max(0.004);
                    let r = Rectangle::new()
                        .width(w)
                        .height(BAR_H)
                        .style(flat(Role::lit(), 0.9))
                        .build()
                        .expect("positive extents");
                    let m = s.add(r);
                    s.move_to(m, v(BAR_X, y), LEFT);
                    m
                });
                stage.add_to_scene(bar)?;
                let book = Rc::clone(&kit.book);
                let label = stage.always_redraw(move |s| {
                    let done = s.tracker_value(t).unwrap_or(0.0).min(got);
                    let d = DecimalNumber::new(100.0 * done / k as f64)
                        .num_decimal_places(1)
                        .unit("%")
                        .font_size(26.0)
                        .color(Palette::ink())
                        .build(&book)
                        .expect("bundled digits always lay out");
                    let m = s.add(d.into_vmob());
                    s.move_to(m, v(BAR_X + BAR_W + 0.2, y), LEFT);
                    m
                });
                stage.add_to_scene(label)?;
                live.extend([bar, label]);
                keep.push(back);
            }
            // every run peels at the same speed; a stalled one stops early
            let up = t
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(3.0),
                    rate_func: Some(fmn::core::rate::linear),
                    ..AnimateArgs::default()
                })?
                .set_value(k as f64)?;
            stage.play(up)?;
            for m in live {
                stage.clear_updaters(m, true);
                keep.push(m);
            }
            keep.push(head);
        }
        // the runs with 2% extra that did not collapse: how much is missing
        let missing: Vec<usize> = runs[1]
            .iter()
            .map(|r| k - r.recovered())
            .filter(|&m| m < k / 2)
            .collect();
        let (lo, hi) = (
            missing.iter().min().copied().unwrap_or(0),
            missing.iter().max().copied().unwrap_or(0),
        );
        line.at(stage, 2)?;
        // the range covers only the runs that kept going; a run that stalled
        // early is marked at its own bar (R3 Ruth R3-05)
        let gap = kit.text(
            stage,
            &format!(
                "{} of {} runs: {lo} to {hi} pieces still missing",
                missing.len(),
                runs[1].len()
            ),
            28.0,
            Role::lost(),
            v(BAR_X + BAR_W - 1.4, 0.0),
        )?;
        let mut marks = vec![gap];
        for (i, run) in runs[1].iter().enumerate() {
            if k - run.recovered() >= k / 2 {
                let y = extra_top - 0.5 - BAR_GAP * i as f64;
                marks.push(kit.text_left(stage, "stalled", 26.0, Role::lost(), v(BAR_X + BAR_W + 1.05, y))?);
            }
        }
        let gap = group_of(stage, &marks)?;
        play!(stage; fade_in(stage.arena_mut(), gap, ORIGIN, 1.0)?.rt(0.5));
        keep.push(gap);
        line.at_share(stage, 0.8)?;
        // the picture, as the most-missing run holds it: it develops in the
        // order peeling found the pieces, and stops
        let run = runs[1]
            .iter()
            .filter(|r| r.recovered() > k / 2)
            .min_by_key(|r| r.recovered())
            .expect("a run that did not collapse")
            .clone();
        let mut rank = vec![u32::MAX; k];
        for (n, &i) in run.order.iter().enumerate() {
            rank[i as usize] = n as u32;
        }
        let rank: Rc<Vec<u32>> = Rc::new(rank);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in keep {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        stage.play_prepared(anims)?;
        let size = 5.3;
        let at = v(0.0, 0.3);
        let strips = Rc::new(RowStrips::build(
            stage,
            v(at[0] - size / 2.0, at[1] + size / 2.0),
            (size / SIDE as f64, size / SIDE as f64),
            &held(&run, &rank, 0, false),
        )?);
        stage.add_to_scene(strips.group)?;
        let t = stage.add_value_tracker(0.0);
        let (s2, r2, run2) = (Rc::clone(&strips), Rc::clone(&rank), run.clone());
        let last = Rc::new(Cell::new(usize::MAX));
        let painted = Rc::clone(&last);
        stage.add_dt_updater(
            strips.group,
            move |s, _, _| {
                let got = s.tracker_value(t).unwrap_or(0.0).round() as usize;
                if got != painted.get() {
                    painted.set(got);
                    s2.recolor(s, &held(&run2, &r2, got, false));
                }
            },
            true,
        )?;
        let develop = t
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(2.6),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(run.recovered() as f64)?;
        stage.play(develop)?;
        stage.clear_updaters(strips.group, true);
        kit.hold(stage)?;

        // 14_7: the pieces in no card, red; no decoder can find them
        let avg = crate::decoding::expected_untouched();
        let caption2 = kit.text(
            stage,
            &format!(
                "{}% extra: on average ≈ {} of the {} pieces are in no card",
                PEEL_EXTRA_PERCENT[1],
                ((avg / 10.0).round() * 10.0) as u64,
                thousands(k as u64)
            ),
            24.0,
            Palette::dim(),
            v(0.0, -3.45),
        )?;
        let cell_w = size / SIDE as f64;
        let mut rings = Vec::new();
        for &i in &run.untouched {
            let (r, c) = (i as usize / SIDE, i as usize % SIDE);
            let p = v(
                at[0] - size / 2.0 + cell_w * (c as f64 + 0.5),
                at[1] + size / 2.0 - cell_w * (r as f64 + 0.5),
            );
            rings.push(
                Circle::new()
                    .radius(0.085)
                    .arc_center(p)
                    .style(line_style(Role::lost(), 2.5))
                    .build(),
            );
        }
        let rings = stage.arena_mut().add(v_group(rings));
        let line = Spoken::say(kit, stage, board, "14_7")?;
        strips.recolor(stage.arena_mut(), &held(&run, &rank, run.recovered(), true));
        play!(stage;
            fade_in(stage.arena_mut(), rings, ORIGIN, 1.6)?.rt(0.8),
            fmn::animation::replacement_transform(caption, caption2).rt(0.6),
        );
        line.at(stage, 1)?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), rings, 1.08, Some([0.99, 0.38, 0.33]))?.rt(1.2));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoding::PEEL_RUNS;

    /// Narration truth: every verdict 14's voice speaks, on the data the
    /// chapter draws.
    #[test]
    fn the_voice_of_14_matches_its_data() {
        // 14_1: "about forty-two trillion steps … At a billion a second, half a day"
        let steps = dense_steps(DENSE_PIECES);
        // the measured decode in 14_1's lower-third is of the same block size
        assert_eq!(u64::from(DECODE_PIECES), DENSE_PIECES);
        assert_eq!(seconds_shown(&DECODE_MS_8_BYTES), "0.1");
        assert_eq!(seconds_shown(&DECODE_MS_1280_BYTES), "0.2–0.3");
        assert_eq!((steps / 1e12).round(), 42.0);
        let days = dense_time(DENSE_PIECES, STEPS_PER_SECOND, 86_400);
        assert!((0.45..0.55).contains(&days), "half a day: {days}");
        // 14_2: "About half mix two, and a few mix many"
        assert_eq!(degree_mass(2), 0.5);
        assert!(degree_mass(30) > 0.0 && degree_mass(30) < 0.05, "a few mix many");
        // 14_3: the cascade runs to the end; 14_5: "Nothing moves."
        let all: Vec<usize> = (0..TOY_CARDS.len()).collect();
        assert_eq!(peel_toy(&all, None).0.len(), 5);
        assert!(peel_toy(&all[1..], None).0.is_empty());
        let runs = peel_runs();
        let share = |r: &PeelRun| r.recovered() as f64 / PEEL_PIECES as f64;
        // 14_6: "With exactly enough sparse cards, peeling usually gets stuck early"
        assert!(runs[0].iter().all(|r| share(r) < 0.5));
        // "Even with two percent extra, most runs get stuck around ninety-eight percent"
        let near_98 = runs[1].iter().filter(|r| (0.975..0.99).contains(&share(r))).count();
        assert!(2 * near_98 > PEEL_RUNS, "most runs: {near_98}");
        // "over a hundred pieces are missing"
        assert!(
            runs[1]
                .iter()
                .filter(|r| share(r) > 0.5)
                .all(|r| PEEL_PIECES - r.recovered() > 100)
        );
        // 14_7: "On average, about seventy of them aren't in any card"
        let avg = crate::decoding::expected_untouched();
        assert_eq!((avg / 10.0).round() * 10.0, 70.0, "{avg}");
        // the drawn run's own untouched pieces are never recovered (picture: red)
        for r in &runs[1] {
            assert!(r.untouched.iter().all(|i| !r.order.contains(i)));
        }
    }
}
