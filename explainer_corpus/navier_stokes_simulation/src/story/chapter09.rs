//! Chapter 09, "Pressure, a puzzle for the whole box" (navier-full-script03.md, cues 09_1 to 09_12).
//!
//! The four-cell box stays on the left at the 07_6 placement; the right-hand panel builds row A, the
//! 4 × 4 system, a face ledger (u*, push, u_new), the loop argument, the residual identity and the
//! continuum forms. Every number on screen is computed by `four_cell::BoxFacts` and checked by
//! `BoxFacts::check` before the first frame. Nothing here is typed.
//!
//! The scene opens on the chapter 08 end state (08_7, 08_8), rebuilt at t = 0 from `box_view`, the
//! module chapter 08 draws that state with: corrected arrows (1/4, −1/4, −1/4, 1/4) circulating
//! counterclockwise with their labels beside the faces, green 0 badges, the four step cards with
//! "correct" lit and the box-units tag at the box's upper left. It ends on the chapter 10 start
//! (10_1): the full static tunnel with its teaching grid and the open question card, nothing else.
//! The tunnel is the geometry of the retained startup snapshot the opening draws (`--tunnel DIR`,
//! bound in `story.rs`), drawn as `opening.rs` draws it.
//!
//! Voice: the kit narrator, as in chapters 07, 08 and 11. Each cue is `say`d before its pictures,
//! and pictures wait on its sentence starts (`until_sentence`); a scratch-voice draft times them
//! from estimated line lengths, a narrated render from the takes' own sentence starts.

use explainer_kit::{Facts, Kit, Palette, Timed, hex};
use fmn::animation::{grow_arrow, indicate};
use fmn::core::color::interpolate_color;
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

use crate::box_view::{
    self, BADGE_SIZE, BOX, CARD_WORD_SIZE, CARD_Y, CARDS, CORRECT, FLOW_LABEL_SIZE, FLOW_SCALE, LANE, LETTER_SIZE,
    SIDE, TAG_SIZE, WALL_SIZE, at_cell, badge_color, cell_center, face_mid, flow_color, on_face, p, push_color,
};
use crate::error;
use crate::field::TunnelGeometry;
use crate::four_cell::{BoxFacts, CELLS, FACES, LOOP_CELLS, PIN, Q, ROW_A, UNITS, V4, face_name};

/// Chapter 09's cues in spoken order and the number of sentences in each (the narrator's gates).
/// Sentences end at `.`, `?` or `!` before a space, as in the kit; the check recounts them.
pub const CUES: [(&str, usize); 12] = [
    ("09_1", 4),
    ("09_2", 1),
    ("09_3", 2),
    ("09_4", 3),
    ("09_5", 6),
    ("09_6", 3),
    ("09_7", 8),
    ("09_8", 2),
    ("09_9", 3),
    ("09_10", 3),
    ("09_11", 3),
    ("09_12", 4),
];
/// The script's chapter heading, shown at the start as chapters 07, 08 and 11 show theirs.
const TITLE: &str = "Pressure, a puzzle for the whole box";

/// Chapter 09's rows of `BoxFacts::fact_rows`, in that order (every row but the four chapter 08
/// registers), each pinned to the script line or source it supports.
const PINS: [(&str, &str, &str); 21] = [
    (
        "box.rowA.balance",
        "1 + 2pA - pB - pC = 0; 2pA - pB - pC = -1",
        "script 09_1 picture; FC2 line 31",
    ),
    ("box.rhs", "-1, 1, 0, 0", "script 09_2 picture"),
    (
        "box.matrix",
        "2 -1 -1 0; -1 2 0 -1; -1 0 2 -1; 0 -1 -1 2",
        "script 09_2 picture",
    ),
    ("box.eigenvalues", "0, 2, 2, 4", "script 09_2 precisely"),
    (
        "box.diagonal.open.neighbours",
        "2, 2, 2, 2",
        "script 09_3 \"a two on the diagonal\"",
    ),
    (
        "box.interior.diagonal.2d",
        "4",
        "script 09_3 \"a cell would have four\"",
    ),
    ("box.p.pinned", "0, 3/4, 1/4, 1/2", "script 09_5 picture"),
    (
        "box.ranking",
        "B, D, C, A",
        "script 09_5 \"B has the highest pressure\", \"A has the lowest\"",
    ),
    ("box.push.AB", "-3/4", "script 09_5 \"B pushes back into A\""),
    ("box.push.BD", "1/4", "script 09_5 \"and up into D\""),
    ("box.push.AC", "-1/4", "script 09_5 \"C pushes down into A\""),
    ("box.push.CD", "-1/4", "script 09_5 \"D pushes left into C\""),
    ("box.p.shift", "3/8", "script 09_6 picture"),
    ("box.p.meanzero", "-3/8, 3/8, -1/8, 1/8", "script 09_6 picture"),
    (
        "box.gauge.same.faces",
        "-3/4, -1/4, -1/4, 1/4",
        "script 09_6 \"none of the face corrections changes\"",
    ),
    ("box.loop.before", "1", "script 09_7 \"that total is one\""),
    ("box.loop.pressure", "0", "script 09_7 \"so they cancel\""),
    ("box.loop.after", "1", "script 09_7 \"so it stays one\""),
    ("box.loop.each", "1/4", "script 09_7 \"each carries a quarter\""),
    (
        "box.residual.identity",
        "p = -1/4, 1/4, 0, 0; r = -1/4, 1/4, -1/4, 1/4; div u_new = 1/4, -1/4, 1/4, -1/4",
        "script 09_8 picture; FC2 lines 58 to 66",
    ),
    (
        "units.projection",
        "h = 1, rho = 1, dt = 1; correction factor dt/rho = 1; matrix factor dt/(rho h^2) = 1",
        "script 09_9 precisely",
    ),
];

/// Chapter 09's numbers alone, checked (a preview's `facts`; chapter 08 owns the four shared rows).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 09's numbers to a film's registry after chapter 08's: every `fact_rows` row that
/// chapter 08 does not register, pinned as `PINS` says (the CLI's `self_check` compares each pin).
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let f = BoxFacts::solve();
    f.check()?;
    let rows: Vec<(String, String)> = f
        .fact_rows()
        .into_iter()
        .filter(|(key, _)| !super::chapter08::SHARED_FACTS.contains(&key.as_str()))
        .collect();
    let keys: Vec<&str> = rows.iter().map(|(key, _)| key.as_str()).collect();
    if keys != PINS.map(|(key, _, _)| key) {
        return Err(format!(
            "chapter 09 pins {:?}, but four_cell prints {keys:?}",
            PINS.map(|(key, _, _)| key)
        ));
    }
    for ((key, value), (_, expected, source)) in rows.iter().zip(PINS) {
        facts.pin(key, value, expected, source)?;
    }
    Ok(())
}

pub struct Chapter09 {
    pub kit: Kit,
    /// The tunnel the chapter ends on; `None` when the render was not given the retained snapshots.
    pub tunnel: Option<TunnelGeometry>,
}

// Layout, scene units: the camera frame is 16 × 9 (frame_height 9 at 16:9; x within ±8, y within ±4.5).
// Widths are estimates (about CMR em × font size × 0.01); the right-hand notes keep ≥ 0.5 to the edge.
// The box and its face contract are `box_view`'s.
/// The push arrows' stroke (they sit `LANE` outside the face's line; the flow arrows sit on it).
const PUSH_THICKNESS: f64 = 3.2;
const CAPTION_Y: f64 = -3.4;
// The 4 × 4 system.
const MX: [f64; 4] = [1.3, 2.08, 2.86, 3.64];
const MY: [f64; 4] = [2.65, 2.05, 1.45, 0.85];
const HEAD_Y: f64 = 3.25;
const ROW_LABEL_X: f64 = 0.7;
const P_X: f64 = 4.6;
const EQ_X: f64 = 5.3;
const D_X: f64 = 5.95;
// The face ledger, under the system.
const LX: [f64; 4] = [2.2, 3.1, 4.0, 4.9];
const LEDGER_LABEL_X: f64 = 0.45;
const NOTE_X: f64 = 5.45;
const L_HEAD_CELLS: f64 = -0.3;
const L_P: f64 = -0.75;
const L_HEAD_FACES: f64 = -1.75;
const L_USTAR: f64 = -2.2;
const L_PUSH: f64 = -2.65;
const L_UNEW: f64 = -3.1;
// The tunnel, as `opening.rs` draws it.
const TUNNEL_WIDTH: f64 = 11.5;
const TUNNEL_CENTER_Y: f64 = 1.15;

fn pressure_color(value: Q, lo: Q, hi: Q) -> Srgb {
    interpolate_color(Palette::teal(), push_color(), ((value - lo) / (hi - lo)).to_f64())
}

/// Display math for a signed sum, `\tfrac{1}{4} + \tfrac{1}{4} - 1`.
fn sum_tex(terms: &[Q]) -> String {
    let mut out = String::new();
    for (k, &t) in terms.iter().enumerate() {
        match (k, t < Q::ZERO) {
            (0, _) => out.push_str(&t.tex()),
            (_, true) => out.push_str(&format!(" - {}", t.abs().tex())),
            (_, false) => out.push_str(&format!(" + {}", t.tex())),
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Flow,
    Push,
}

/// The chapter's state: the box's persistent mobjects, what is on screen, and a pending batch.
struct Ch9<'a> {
    k: &'a Kit,
    f: BoxFacts,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
    /// Pressure colour scale: every pressure the chapter shows lies in [lo, hi].
    lo: Q,
    hi: Q,
    fills: [Mob; 4],
    outline: Mob,
    faces: [Mob; 4],
    walls: Vec<Mob>,
    directions: Vec<Mob>,
    letters: [Mob; 4],
    dots: [Mob; 4],
    flow: [Option<Mob>; 4],
    push: [Option<Mob>; 4],
    flow_labels: [Mob; 4],
    badges: [Mob; 4],
    badge_values: [Mob; 4],
    pressures: Vec<Mob>,
    tag: Mob,
    cards: Vec<Mob>,
    caption: Option<Mob>,
}

impl Ch9<'_> {
    // Text and math through the kit, so the layout lint and the glyph check see every string.
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, at)
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, at)
    }

    fn tex_spans(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<(Mob, fmn::library::SpanMapData)> {
        self.k.tex_spans(stage, src, size, color, &[], at)
    }

    fn tex(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex(stage, src, size, color, &[], at)
    }

    fn tex_left(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex_left(stage, src, size, color, &[], at)
    }

    fn put(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                stage.add_to_scene(m)?;
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Remove at once (no fade); used only for objects about to be replaced in place.
    fn cut(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) {
        let gone: Vec<Mob> = mobs.iter().copied().filter(|m| self.on.contains(m)).collect();
        stage.scene_mut().remove(&gone);
        self.on.retain(|m| !gone.contains(m));
    }

    /// Bring mobjects already on screen to the front of the draw order.
    fn raise(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        let shown: Vec<Mob> = mobs.iter().copied().filter(|m| self.on.contains(m)).collect();
        stage.scene_mut().remove(&shown);
        for &m in &shown {
            stage.add_to_scene(m)?;
        }
        Ok(())
    }

    fn appear(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Fade out what is on screen; anything already gone is skipped (no second fade).
    fn vanish(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.retain(|&x| x != m);
            }
        }
        Ok(())
    }

    /// `from` (on screen) takes `to`'s shape and paint; `to` stays detached.
    fn morph(&mut self, stage: &mut Stage<'_>, from: Mob, to: Mob, secs: f64) -> fmn::Result<()> {
        if self.on.contains(&from) {
            self.pending.push(stage.prepare(Transform::new(from, to).rt(secs))?);
        }
        Ok(())
    }

    /// Tex to Tex by matching glyphs; the target replaces the source on screen.
    fn matching(
        &mut self,
        stage: &mut Stage<'_>,
        from: (Mob, &fmn::library::SpanMapData),
        to: (Mob, &fmn::library::SpanMapData),
        secs: f64,
    ) -> fmn::Result<Mob> {
        let a = TransformMatchingTex::new(stage.arena_mut(), from.0, to.0, from.1, to.1)?;
        self.pending.push(stage.prepare(a.rt(secs))?);
        self.on.retain(|&x| x != from.0);
        self.on.push(to.0);
        Ok(to.0)
    }

    fn grow(&mut self, stage: &mut Stage<'_>, arrow: Mob, secs: f64) -> fmn::Result<()> {
        if !self.on.contains(&arrow) {
            let a = grow_arrow(stage.arena_mut(), arrow)?.rt(secs);
            self.pending.push(stage.prepare(a)?);
            self.on.push(arrow);
        }
        Ok(())
    }

    fn pulse(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = indicate(stage.arena_mut(), m, 1.15, None)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
            }
        }
        Ok(())
    }

    fn refill(&mut self, stage: &mut Stage<'_>, m: Mob, color: Srgb, opacity: f64, secs: f64) -> fmn::Result<()> {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                ..AnimateArgs::default()
            })?
            .set_fill(Some(color), Some(opacity))?;
        self.pending.push(stage.prepare(a)?);
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    // The box's own builders (`box_view`), under the names the cues use.
    fn arrow(&self, stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, thickness: f64) -> fmn::Result<Mob> {
        box_view::arrow(stage, a, b, color, thickness)
    }

    fn line(&self, stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
        box_view::line(stage, a, b, color, width)
    }

    fn frame(&self, stage: &mut Stage<'_>, at: Vec3, w: f64, h: f64, color: Srgb, width: f64) -> fmn::Result<Mob> {
        box_view::frame(stage, at, w, h, color, width)
    }

    /// An arrow for `value` on face `f`: on the face's line (flow, `box_view`'s flow arrow) or its
    /// outer lane (push), from `start` along the face direction; `None` for an exact zero.
    fn face_arrow(
        &self,
        stage: &mut Stage<'_>,
        f: usize,
        value: Q,
        lane: Lane,
        start: f64,
    ) -> fmn::Result<Option<Mob>> {
        if lane == Lane::Flow {
            return box_view::flow_arrow(stage, f, value, start);
        }
        if value.is_zero() {
            return Ok(None);
        }
        let end = start + value.to_f64() * FLOW_SCALE;
        Ok(Some(self.arrow(
            stage,
            on_face(f, -LANE, start),
            on_face(f, -LANE, end),
            push_color(),
            PUSH_THICKNESS,
        )?))
    }

    fn centred(&self, stage: &mut Stage<'_>, f: usize, value: Q, lane: Lane) -> fmn::Result<Option<Mob>> {
        self.face_arrow(stage, f, value, lane, -value.to_f64() * FLOW_SCALE / 2.0)
    }

    /// Show `values` on every face, centred: morph, grow or fade each arrow as needed.
    fn set_arrows(&mut self, stage: &mut Stage<'_>, lane: Lane, values: &V4, secs: f64) -> fmn::Result<()> {
        for (f, &value) in values.iter().enumerate() {
            let current = match lane {
                Lane::Flow => self.flow[f],
                Lane::Push => self.push[f],
            };
            let next = match (current, self.centred(stage, f, value, lane)?) {
                (Some(m), Some(target)) => {
                    self.morph(stage, m, target, secs)?;
                    Some(m)
                }
                (Some(m), None) => {
                    self.vanish(stage, &[m], secs)?;
                    None
                }
                (None, Some(target)) => {
                    self.grow(stage, target, secs)?;
                    Some(target)
                }
                (None, None) => None,
            };
            match lane {
                Lane::Flow => self.flow[f] = next,
                Lane::Push => self.push[f] = next,
            }
        }
        Ok(())
    }

    fn set_flow_labels(&mut self, stage: &mut Stage<'_>, values: &V4, secs: f64) -> fmn::Result<()> {
        for (f, &value) in values.iter().enumerate() {
            let position = box_view::flow_label_at(f);
            let target = self.tex(stage, &value.tex(), FLOW_LABEL_SIZE, flow_color(), position)?;
            self.morph(stage, self.flow_labels[f], target, secs)?;
        }
        Ok(())
    }

    fn set_badges(&mut self, stage: &mut Stage<'_>, values: &V4, secs: f64) -> fmn::Result<()> {
        for (i, &value) in values.iter().enumerate() {
            let color = badge_color(value);
            let rim = box_view::badge_frame(stage, i, color)?;
            let number = self.tex(stage, &value.tex(), BADGE_SIZE, color, box_view::badge_at(i))?;
            self.morph(stage, self.badges[i], rim, secs)?;
            self.morph(stage, self.badge_values[i], number, secs)?;
        }
        Ok(())
    }

    /// Pressure labels and cell fills for `values`; the first call creates the labels.
    fn set_pressures(&mut self, stage: &mut Stage<'_>, values: &V4, secs: f64) -> fmn::Result<()> {
        let first = self.pressures.is_empty();
        for (i, &value) in values.iter().enumerate() {
            let label = self.tex(stage, &value.tex(), 30.0, Palette::ink(), at_cell(i, 0.56, 0.0))?;
            if first {
                self.pressures.push(label);
                self.appear(stage, &[label], secs)?;
            } else {
                self.morph(stage, self.pressures[i], label, secs)?;
            }
            self.refill(
                stage,
                self.fills[i],
                pressure_color(value, self.lo, self.hi),
                0.36,
                secs,
            )?;
        }
        Ok(())
    }

    fn set_caption(&mut self, stage: &mut Stage<'_>, s: &str, secs: f64) -> fmn::Result<()> {
        let next = self.text(stage, s, 24.0, Palette::dim(), p(BOX[0], CAPTION_Y))?;
        match self.caption {
            Some(old) => self.morph(stage, old, next, secs)?,
            None => {
                self.appear(stage, &[next], secs)?;
                self.caption = Some(next);
            }
        }
        Ok(())
    }

    /// Ledger row of four values at height `y`.
    fn ledger_row(&self, stage: &mut Stage<'_>, values: &V4, y: f64, color: Srgb) -> fmn::Result<[Mob; 4]> {
        let mut out = Vec::with_capacity(4);
        for (k, &value) in values.iter().enumerate() {
            out.push(self.tex(stage, &value.tex(), 30.0, color, p(LX[k], y))?);
        }
        Ok([out[0], out[1], out[2], out[3]])
    }

    fn morph_row(
        &mut self,
        stage: &mut Stage<'_>,
        row: &[Mob; 4],
        values: &V4,
        y: f64,
        color: Srgb,
        secs: f64,
    ) -> fmn::Result<()> {
        let targets = self.ledger_row(stage, values, y, color)?;
        for (&m, target) in row.iter().zip(targets) {
            self.morph(stage, m, target, secs)?;
        }
        Ok(())
    }

    /// Finish what is queued, then start line `id` (the kit waits for the previous line first).
    fn say(&mut self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        if !CUES.iter().any(|c| c.0 == id) {
            return Err(error(format!("chapter 09 has no cue {id}")));
        }
        self.play(stage)?;
        println!("Chapter 09 cue {id} at {:.6} s", stage.scene().time().to_f64());
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id` (nothing if it has begun).
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(error(format!("chapter 09 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }
}

/// The chapter 08 end state, all on screen at the first frame: every object built by the same
/// `box_view` call, at the same `box_view` place, that chapter 08 ends with.
fn chapter08_end<'a>(stage: &mut Stage<'_>, k: &'a Kit, f: BoxFacts) -> fmn::Result<Ch9<'a>> {
    let all = [f.pinned, f.mean_zero, f.unfinished];
    let lo = all.iter().flatten().copied().min().expect("pressures exist");
    let hi = all.iter().flatten().copied().max().expect("pressures exist");
    // Cell fills go first so that every later object draws over them; they start clear.
    let mut fills = Vec::new();
    for i in 0..4 {
        let [x, y] = cell_center(i);
        let r = Rectangle::new()
            .width(SIDE)
            .height(SIDE)
            .style(
                Style::default()
                    .fill(Palette::teal(), 0.0)
                    .stroke(Palette::teal(), 0.0, 0.0),
            )
            .build()?
            .moved_to(p(x, y));
        fills.push(stage.arena_mut().add(r));
    }
    let outline = box_view::outline(stage)?;
    let faces: Vec<Mob> = (0..FACES.len())
        .map(|g| box_view::dashed_face(stage, g))
        .collect::<fmn::Result<_>>()?;
    let mut s = Ch9 {
        k,
        f,
        on: Vec::new(),
        pending: Vec::new(),
        lo,
        hi,
        fills: [fills[0], fills[1], fills[2], fills[3]],
        outline,
        faces: [faces[0], faces[1], faces[2], faces[3]],
        walls: Vec::new(),
        directions: Vec::new(),
        letters: [outline; 4],
        dots: [outline; 4],
        flow: [None; 4],
        push: [None; 4],
        flow_labels: [outline; 4],
        badges: [outline; 4],
        badge_values: [outline; 4],
        pressures: Vec::new(),
        tag: outline,
        cards: Vec::new(),
        caption: None,
    };
    let wall_flux = Q::ZERO.tex();
    for w in 0..4 {
        let zero = s.tex(stage, &wall_flux, WALL_SIZE, Palette::dim(), box_view::wall_label_at(w))?;
        s.walls.push(zero);
    }
    for g in 0..FACES.len() {
        s.directions.push(box_view::direction_arrow(stage, g)?);
    }
    for (i, name) in CELLS.iter().enumerate() {
        s.letters[i] = s.text(stage, name, LETTER_SIZE, Palette::ink(), box_view::letter_at(i))?;
        s.dots[i] = box_view::dot(stage, i);
        let balance = s.f.corrected_div[i];
        s.badges[i] = box_view::badge_frame(stage, i, badge_color(balance))?;
        s.badge_values[i] = s.tex(
            stage,
            &balance.tex(),
            BADGE_SIZE,
            badge_color(balance),
            box_view::badge_at(i),
        )?;
    }
    let corrected = s.f.corrected;
    for (g, &value) in corrected.iter().enumerate() {
        s.flow[g] = box_view::centred_flow(stage, g, value)?;
        let position = box_view::flow_label_at(g);
        s.flow_labels[g] = s.tex(stage, &value.tex(), FLOW_LABEL_SIZE, flow_color(), position)?;
    }
    s.tag = s.tex_left(
        stage,
        &box_view::tag_tex(),
        TAG_SIZE,
        Palette::dim(),
        box_view::tag_at(),
    )?;
    for (k, name) in CARDS.iter().enumerate() {
        let (card, ink) = box_view::card(stage, k, k == CORRECT)?;
        let word = s.text(stage, name, CARD_WORD_SIZE, ink, p(box_view::card_x(k), CARD_Y))?;
        s.cards.extend([card, word]);
    }
    let mut first: Vec<Mob> = s.fills.to_vec();
    first.push(s.outline);
    first.extend(s.faces);
    first.extend(s.walls.iter().copied());
    first.extend(s.directions.iter().copied());
    first.extend(s.letters);
    first.extend(s.dots);
    first.extend(s.flow.iter().flatten().copied());
    first.extend(s.flow_labels);
    first.extend(s.badges);
    first.extend(s.badge_values);
    first.push(s.tag);
    first.extend(s.cards.iter().copied());
    s.put(stage, &first)?;
    println!(
        "Chapter 09 opens on the chapter 08 end state: {} objects, corrected faces {:?}",
        first.len(),
        corrected.map(|q| q.to_string())
    );
    Ok(s)
}

/// The static tunnel and its teaching grid, as `opening.rs` draws them, at any width and centre.
pub(super) fn tunnel(stage: &mut Stage<'_>, g: TunnelGeometry, width: f64, center: [f64; 2]) -> fmn::Result<Vec<Mob>> {
    let scale = width / g.length;
    let at = |x: f64, y: f64| {
        p(
            center[0] + scale * (x - g.length / 2.0),
            center[1] + scale * (y - g.height / 2.0),
        )
    };
    let [cx, cy, radius] = g.cylinder;
    let mut out = vec![
        stage.arena_mut().add(
            Rectangle::new()
                .width(width)
                .height(scale * g.height)
                .style(
                    Style::default()
                        .fill(hex("#17283C"), 1.0)
                        .stroke(Palette::dim(), 1.7, 1.0),
                )
                .build()?
                .moved_to(p(center[0], center[1])),
        ),
        stage.arena_mut().add(
            Circle::new()
                .radius(scale * radius)
                .style(
                    Style::default()
                        .fill(hex("#0A121F"), 1.0)
                        .stroke(Palette::ink(), 1.6, 1.0),
                )
                .build()
                .moved_to(at(cx, cy)),
        ),
    ];
    let cell = g.height / 3.0;
    let line = |stage: &mut Stage<'_>, a: Vec3, b: Vec3| -> fmn::Result<Mob> {
        Ok(stage.arena_mut().add(
            Line::new(a, b)
                .style(Style::default().stroke(Palette::faint(), 1.1, 1.0))
                .build()?,
        ))
    };
    for row in 1..3 {
        let y = f64::from(row) * cell;
        out.push(line(stage, at(0.0, y), at(g.length, y))?);
    }
    let middle = g.length / 2.0;
    let half_columns = (middle / cell).floor() as i32;
    for column in -half_columns..=half_columns {
        let x = middle + f64::from(column) * cell;
        out.push(line(stage, at(x, 0.0), at(x, g.height))?);
    }
    Ok(out)
}

impl SceneConstruct for Chapter09 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        // Both refusals come before the first frame.
        let geometry = self.tunnel.ok_or_else(|| {
            error("chapter 09 ends on the tunnel: render it with --tunnel and the retained startup snapshots".into())
        })?;
        let solved = BoxFacts::solve();
        solved.check().map_err(error)?;
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for line in facts().map_err(error)?.print().lines() {
            println!("Chapter 09 fact {line}");
        }
        let mut s = chapter08_end(stage, &self.kit, solved)?;
        let f = s.f.clone();
        let ink = Palette::ink();
        let dim = Palette::dim();
        let hero = Palette::hero();

        // 09_1: back to the provisional flow, then row A from its two faces.
        s.say(stage, "09_1")?;
        let cards = s.cards.clone();
        s.vanish(stage, &cards, 0.5)?;
        s.set_arrows(stage, Lane::Flow, &f.provisional, 0.7)?;
        s.set_flow_labels(stage, &f.provisional, 0.7)?;
        s.set_badges(stage, &f.imbalance, 0.7)?;
        s.set_caption(stage, "provisional flow u*", 0.5)?;
        // The chapter's title takes the cards' row while the first sentence plays, then leaves it
        // to row A's face updates.
        s.play(stage)?;
        let title = s.text(stage, TITLE, 45.0, ink, p(0.0, CARD_Y))?;
        s.appear(stage, &[title], 0.5)?;
        s.play(stage)?;
        stage.wait(0.9)?;
        s.vanish(stage, &[title], 0.4)?;
        s.until(stage, "09_1", 1)?;
        let row_faces: Vec<usize> = (0..4).filter(|&g| FACES[g].0 == ROW_A || FACES[g].1 == ROW_A).collect();
        let glow = |s: &Ch9<'_>, stage: &mut Stage<'_>, g: usize| -> fmn::Result<Mob> {
            s.line(
                stage,
                on_face(g, SIDE / 2.0, 0.0),
                on_face(g, -SIDE / 2.0, 0.0),
                hero,
                4.0,
            )
        };
        let first_face = glow(&s, stage, row_faces[0])?;
        let update_a = s.tex(stage, &f.face_update_tex(row_faces[0]), 36.0, ink, p(3.8, 3.3))?;
        s.appear(stage, &[first_face, update_a], 0.45)?;
        let (a, b) = FACES[row_faces[0]];
        s.pulse(stage, &[s.dots[a], s.dots[b]], 0.6)?;
        s.until(stage, "09_1", 2)?;
        let push_note = s.text(
            stage,
            "the push runs from higher p to lower p",
            22.0,
            push_color(),
            p(3.8, 2.85),
        )?;
        s.appear(stage, &[push_note], 0.4)?;
        s.until(stage, "09_1", 3)?;
        let second_face = glow(&s, stage, row_faces[1])?;
        let update_b = s.tex(stage, &f.face_update_tex(row_faces[1]), 36.0, ink, p(3.8, 2.35))?;
        s.appear(stage, &[second_face, update_b], 0.45)?;
        s.play(stage)?;
        stage.wait(0.6)?;
        let (balance, balance_spans) = s.tex_spans(stage, &f.balance_tex(ROW_A), 40.0, ink, p(3.8, 1.6))?;
        s.appear(stage, &[balance], 0.5)?;
        s.play(stage)?;
        stage.wait(0.9)?;
        let (row_a, row_a_spans) = s.tex_spans(stage, &f.equation_tex(ROW_A), 40.0, ink, p(3.8, 1.6))?;
        let row_a = s.matching(stage, (balance, &balance_spans), (row_a, &row_a_spans), 0.8)?;
        let minus_label = s.text(
            stage,
            "minus the provisional imbalance",
            22.0,
            Palette::red(),
            p(5.0, 1.05),
        )?;
        s.appear(stage, &[minus_label], 0.4)?;

        // 09_2: the same for every cell: the 4 × 4 system in order A, B, C, D.
        s.say(stage, "09_2")?;
        s.vanish(
            stage,
            &[
                first_face,
                second_face,
                update_a,
                update_b,
                push_note,
                minus_label,
                row_a,
            ],
            0.4,
        )?;
        let mut entries = [[s.outline; 4]; 4];
        let mut rows: Vec<Vec<Mob>> = Vec::new();
        let mut matrix_frame = Vec::new();
        for (j, name) in CELLS.iter().enumerate() {
            matrix_frame.push(s.text(stage, name, 24.0, dim, p(MX[j], HEAD_Y))?);
            matrix_frame.push(s.text(stage, name, 24.0, dim, p(ROW_LABEL_X, MY[j]))?);
        }
        for (i, row) in f.matrix.iter().enumerate() {
            let mut line = Vec::new();
            for (j, &value) in row.iter().enumerate() {
                entries[i][j] = s.tex(stage, &value.tex(), 34.0, ink, p(MX[j], MY[i]))?;
                line.push(entries[i][j]);
            }
            line.push(s.tex(stage, &format!("p_{}", CELLS[i]), 34.0, ink, p(P_X, MY[i]))?);
            line.push(s.tex(stage, &f.rhs[i].tex(), 34.0, Palette::red(), p(D_X, MY[i]))?);
            rows.push(line);
        }
        let (top, bottom) = (MY[0] + 0.35, MY[3] - 0.35);
        for (x, opens_right) in [
            (MX[0] - 0.35, true),
            (MX[3] + 0.36, false),
            (P_X - 0.32, true),
            (P_X + 0.32, false),
            (D_X - 0.35, true),
            (D_X + 0.35, false),
        ] {
            let tick = if opens_right { 0.12 } else { -0.12 };
            matrix_frame.push(s.line(stage, p(x, top), p(x, bottom), dim, 2.0)?);
            matrix_frame.push(s.line(stage, p(x, top), p(x + tick, top), dim, 2.0)?);
            matrix_frame.push(s.line(stage, p(x, bottom), p(x + tick, bottom), dim, 2.0)?);
        }
        let equals = s.tex(stage, "=", 34.0, ink, p(EQ_X, (top + bottom) / 2.0))?;
        matrix_frame.push(equals);
        s.appear(stage, &matrix_frame, 0.4)?;
        s.appear(stage, &rows[ROW_A], 0.4)?;
        s.until(stage, "09_2", 0)?;
        stage.wait(0.5)?;
        for (i, row) in rows.iter().enumerate() {
            if i != ROW_A {
                s.appear(stage, row, 0.35)?;
                s.play(stage)?;
                stage.wait(0.15)?;
            }
        }
        let mut matrix_all: Vec<Mob> = matrix_frame.clone();
        matrix_all.extend(rows.iter().flatten().copied());

        // 09_3: the corner diagonal is the count of crossable faces; an interior cell has four.
        s.say(stage, "09_3")?;
        let mut diagonal = Vec::new();
        for (&x, &y) in MX.iter().zip(&MY) {
            diagonal.push(s.frame(stage, p(x, y), 0.6, 0.5, ink, 2.4)?);
        }
        s.appear(stage, &diagonal, 0.45)?;
        let a_faces: Vec<Mob> = row_faces
            .iter()
            .map(|&g| glow(&s, stage, g))
            .collect::<fmn::Result<_>>()?;
        s.appear(stage, &a_faces, 0.4)?;
        s.until(stage, "09_3", 0)?;
        s.pulse(stage, &[s.walls[0], s.walls[2]], 0.8)?;
        s.until(stage, "09_3", 1)?;
        s.vanish(stage, &a_faces, 0.3)?;
        let inset_c = [2.6, -1.55];
        let mini = 0.45;
        let mut inset = Vec::new();
        for gx in -1..=1 {
            for gy in -1..=1 {
                let centre = gx == 0 && gy == 0;
                let at = p(inset_c[0] + f64::from(gx) * mini, inset_c[1] + f64::from(gy) * mini);
                let color = if centre { hero } else { dim };
                let cell = stage.arena_mut().add(
                    Rectangle::new()
                        .width(mini)
                        .height(mini)
                        .style(
                            Style::default()
                                .fill(color, if centre { 0.18 } else { 0.0 })
                                .stroke(color, 1.4, 1.0),
                        )
                        .build()?
                        .moved_to(at),
                );
                inset.push(cell);
            }
        }
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            let from = p(inset_c[0] + 0.12 * dx, inset_c[1] + 0.12 * dy);
            let to = p(inset_c[0] + 0.62 * dx, inset_c[1] + 0.62 * dy);
            inset.push(s.arrow(stage, from, to, dim, 1.6)?);
        }
        inset.push(s.tex(
            stage,
            &f.interior_open_faces.to_string(),
            30.0,
            hero,
            p(inset_c[0], inset_c[1]),
        )?);
        inset.push(s.text(
            stage,
            "deep inside a bigger grid",
            20.0,
            dim,
            p(inset_c[0], inset_c[1] - 1.0),
        )?);
        s.appear(stage, &inset, 0.5)?;

        // 09_4: each off-diagonal −1 is a shared face; the equations are linked.
        s.say(stage, "09_4")?;
        s.vanish(stage, &inset, 0.4)?;
        s.vanish(stage, &diagonal, 0.4)?;
        let mut links = Vec::new();
        for (g, &(a, b)) in FACES.iter().enumerate() {
            let face = glow(&s, stage, g)?;
            let rims = [
                s.frame(stage, p(MX[b], MY[a]), 0.62, 0.5, hero, 2.2)?,
                s.frame(stage, p(MX[a], MY[b]), 0.62, 0.5, hero, 2.2)?,
            ];
            let [mx, my] = face_mid(g);
            let wire = s.line(stage, p(mx, my), p(MX[b] - 0.3, MY[a]), hero, 1.4)?;
            links.push((face, rims, wire));
        }
        for (k, &(face, rims, wire)) in links.iter().enumerate() {
            if k == 2 {
                s.until(stage, "09_4", 1)?;
            }
            s.appear(stage, &[face, rims[0], rims[1], wire], 0.35)?;
            s.play(stage)?;
            stage.wait(0.45)?;
            s.vanish(stage, &[wire], 0.3)?;
        }
        s.pulse(stage, &[s.dots[ROW_A]], 0.6)?;
        s.play(stage)?;
        let neighbours: Vec<Mob> = row_faces
            .iter()
            .map(|&g| s.badges[FACES[g].0 + FACES[g].1 - ROW_A])
            .collect();
        s.pulse(stage, &neighbours, 0.7)?;
        s.until(stage, "09_4", 2)?;
        let off: Vec<Mob> = links.iter().flat_map(|(_, rims, _)| rims.iter().copied()).collect();
        s.pulse(stage, &off, 0.8)?;
        s.play(stage)?;
        let lit: Vec<Mob> = links
            .iter()
            .map(|&(face, _, _)| face)
            .chain(off.iter().copied())
            .collect();
        s.vanish(stage, &lit, 0.5)?;

        // 09_5: the pressures (A pinned at zero), the ranking, and the pushes in the spoken order.
        s.say(stage, "09_5")?;
        s.set_pressures(stage, &f.pinned, 0.6)?;
        let mut ledger_frame = Vec::new();
        for (k, name) in CELLS.iter().enumerate() {
            ledger_frame.push(s.text(stage, name, 22.0, dim, p(LX[k], L_HEAD_CELLS))?);
        }
        ledger_frame.push(s.tex_left(stage, "p", 30.0, ink, p(LEDGER_LABEL_X, L_P))?);
        let p_row = s.ledger_row(stage, &f.pinned, L_P, ink)?;
        let pin_note = format!("p_{} = {}", CELLS[PIN], f.pinned[PIN].tex());
        let note = s.tex_left(stage, &pin_note, 24.0, dim, p(NOTE_X, L_P))?;
        s.appear(stage, &ledger_frame, 0.4)?;
        s.appear(stage, &p_row, 0.4)?;
        s.appear(stage, &[note], 0.4)?;
        s.play(stage)?;
        s.pulse(stage, &[s.pressures[f.ranking[0]], p_row[f.ranking[0]]], 0.8)?;
        s.until(stage, "09_5", 1)?;
        let lowest = f.ranking[3];
        s.pulse(stage, &[s.pressures[lowest], p_row[lowest]], 0.8)?;
        let mut face_head = Vec::new();
        for (k, &(a, b)) in FACES.iter().enumerate() {
            face_head.push(s.tex(
                stage,
                &format!("{}\\to {}", CELLS[a], CELLS[b]),
                26.0,
                dim,
                p(LX[k], L_HEAD_FACES),
            )?);
        }
        let push_label = s.tex_left(
            stage,
            "-\\operatorname{grad} p",
            28.0,
            push_color(),
            p(LEDGER_LABEL_X, L_PUSH),
        )?;
        let push_row = s.ledger_row(stage, &f.push, L_PUSH, push_color())?;
        // Spoken order: B into A and B up into D, then C down into A, then D left into C.
        let spoken: [&[&str]; 3] = [&["AB", "BD"], &["AC"], &["CD"]];
        for (k, names) in spoken.iter().enumerate() {
            s.until(stage, "09_5", 2 + k)?;
            if k == 0 {
                s.appear(stage, &face_head, 0.3)?;
                s.appear(stage, &[push_label], 0.3)?;
            }
            for name in names.iter() {
                let g = (0..4).find(|&g| face_name(g) == *name).expect("spoken faces exist");
                let arrow = s.centred(stage, g, f.push[g], Lane::Push)?;
                if let Some(m) = arrow {
                    s.grow(stage, m, 0.6)?;
                }
                s.push[g] = arrow;
                s.appear(stage, &[push_row[g]], 0.4)?;
            }
        }
        // Nested: u* plus the push is the new flow, face by face.
        s.until(stage, "09_5", 5)?;
        let ustar_label = s.tex_left(stage, "u^*", 28.0, dim, p(LEDGER_LABEL_X, L_USTAR))?;
        let ustar_row = s.ledger_row(stage, &f.provisional, L_USTAR, dim)?;
        s.appear(stage, &[ustar_label], 0.3)?;
        s.appear(stage, &ustar_row, 0.3)?;
        // The push moves onto the face's line with its tail at the tip of u* (at the midpoint where u* is zero).
        for g in 0..4 {
            let tip = f.provisional[g].to_f64() * FLOW_SCALE / 2.0;
            if let Some(m) = s.push[g] {
                let end = tip + f.push[g].to_f64() * FLOW_SCALE;
                let orange = s.arrow(stage, on_face(g, 0.0, tip), on_face(g, 0.0, end), push_color(), 3.2)?;
                s.morph(stage, m, orange, 0.7)?;
            }
        }
        s.play(stage)?;
        stage.wait(0.35)?;
        let unew_label = s.tex_left(stage, "u_{\\text{new}}", 28.0, flow_color(), p(LEDGER_LABEL_X, L_UNEW))?;
        let unew_row = s.ledger_row(stage, &f.corrected, L_UNEW, flow_color())?;
        for g in 0..4 {
            let start = -f.provisional[g].to_f64() * FLOW_SCALE / 2.0;
            let Some(result) = s.face_arrow(stage, g, f.corrected[g], Lane::Flow, start)? else {
                continue;
            };
            match s.flow[g] {
                Some(m) => s.morph(stage, m, result, 0.7)?,
                None => {
                    s.grow(stage, result, 0.7)?;
                    s.flow[g] = Some(result);
                }
            }
        }
        s.appear(stage, &[unew_label], 0.4)?;
        s.appear(stage, &unew_row, 0.4)?;
        s.play(stage)?;
        stage.wait(0.45)?;
        s.set_arrows(stage, Lane::Flow, &f.corrected, 0.7)?;
        s.set_arrows(stage, Lane::Push, &f.push, 0.7)?;
        s.set_flow_labels(stage, &f.corrected, 0.7)?;
        s.set_badges(stage, &f.corrected_div, 0.7)?;
        s.set_caption(stage, "corrected flow", 0.5)?;

        // 09_6: one shared constant moves every pressure; no face correction changes.
        s.say(stage, "09_6")?;
        let pushes: Vec<Mob> = s.push.iter().flatten().copied().collect();
        s.pulse(stage, &pushes, 0.7)?;
        s.until(stage, "09_6", 1)?;
        s.set_pressures(stage, &f.mean_zero, 0.9)?;
        s.morph_row(stage, &p_row, &f.mean_zero, L_P, ink, 0.9)?;
        let shift_note = s.tex_left(
            stage,
            &format!("\\text{{each }} {}", (-f.shift).tex()),
            24.0,
            hero,
            p(NOTE_X, L_P),
        )?;
        s.morph(stage, note, shift_note, 0.6)?;
        s.play(stage)?;
        s.pulse(stage, &pushes, 0.8)?;
        s.pulse(stage, &push_row, 0.8)?;
        // The row already shows f.push_mean_zero: BoxFacts::check proved it equals f.push.
        let same = s.text_left(stage, "unchanged", 24.0, Palette::green(), p(NOTE_X, L_PUSH))?;
        s.appear(stage, &[same], 0.4)?;
        s.until(stage, "09_6", 2)?;
        let mean_note = s.text_left(stage, "mean zero", 22.0, dim, p(NOTE_X, L_P))?;
        s.morph(stage, note, mean_note, 0.5)?;

        // 09_7: the loop. The pressure differences cancel, so the loop total stays one, shared by four.
        s.say(stage, "09_7")?;
        s.vanish(stage, &matrix_all, 0.5)?;
        s.vanish(stage, &[same], 0.3)?;
        let mut ring = Vec::new();
        for pair in LOOP_CELLS.windows(2) {
            let ([x0, y0], [x1, y1]) = (cell_center(pair[0]), cell_center(pair[1]));
            let (dx, dy) = ((x1 - x0) / SIDE, (y1 - y0) / SIDE);
            ring.push(s.arrow(
                stage,
                p(x0 + 0.2 * dx, y0 + 0.2 * dy),
                p(x1 - 0.2 * dx, y1 - 0.2 * dy),
                ink,
                1.8,
            )?);
        }
        let route: Vec<&str> = LOOP_CELLS.iter().map(|&i| CELLS[i]).collect();
        let title = s.tex(
            stage,
            &format!("\\text{{around the loop }} {}", route.join("\\to ")),
            28.0,
            ink,
            p(3.9, 3.3),
        )?;
        s.appear(stage, &ring, 0.6)?;
        s.appear(stage, &[title], 0.5)?;
        s.play(stage)?;
        let flows: Vec<Mob> = s.flow.iter().flatten().copied().collect();
        s.raise(stage, &flows)?;
        let travel = |u: &V4| -> Vec<Q> { f.loop_faces.iter().map(|&(g, sign)| Q::int(sign) * u[g]).collect() };
        s.until(stage, "09_7", 1)?;
        let before_src = format!(
            "\\text{{before: }} {} = {}",
            sum_tex(&travel(&f.provisional)),
            f.loop_before.tex()
        );
        let before = s.tex(stage, &before_src, 30.0, dim, p(3.9, 2.75))?;
        s.appear(stage, &[before], 0.5)?;
        s.pulse(stage, &[ustar_row[0]], 0.8)?;
        s.until(stage, "09_7", 2)?;
        let differences: Vec<String> = LOOP_CELLS
            .windows(2)
            .map(|w| format!("(p_{} - p_{})", CELLS[w[1]], CELLS[w[0]]))
            .collect();
        let telescope = differences.join(" + ");
        let pressure_label = s.text(stage, "sum of pressure differences", 24.0, push_color(), p(3.9, 2.15))?;
        let (open, open_spans) = s.tex_spans(stage, &telescope, 28.0, ink, p(3.65, 1.6))?;
        s.appear(stage, &[pressure_label, open], 0.5)?;
        s.until(stage, "09_7", 3)?;
        s.pulse(stage, &[open], 0.8)?;
        s.play(stage)?;
        let closed_src = format!("{telescope} = {}", f.loop_pressure.tex());
        let (closed, closed_spans) = s.tex_spans(stage, &closed_src, 28.0, ink, p(3.65, 1.6))?;
        s.matching(stage, (open, &open_spans), (closed, &closed_spans), 0.7)?;
        s.until(stage, "09_7", 4)?;
        let unknown = vec!["?"; f.loop_faces.len()].join(" + ");
        let (after_q, after_q_spans) = s.tex_spans(
            stage,
            &format!("\\text{{after: }} {unknown} = {}", f.loop_after.tex()),
            30.0,
            flow_color(),
            p(3.9, 1.0),
        )?;
        s.appear(stage, &[after_q], 0.5)?;
        s.until(stage, "09_7", 5)?;
        s.pulse(stage, &flows, 0.9)?;
        let innocent: Vec<usize> = (0..4).filter(|&g| f.provisional[g].is_zero()).collect();
        let innocent_glow: Vec<Mob> = innocent
            .iter()
            .map(|&g| glow(&s, stage, g))
            .collect::<fmn::Result<_>>()?;
        s.appear(stage, &innocent_glow, 0.5)?;
        s.until(stage, "09_7", 6)?;
        let after_src = format!(
            "\\text{{after: }} {} = {}",
            sum_tex(&travel(&f.corrected)),
            f.loop_after.tex()
        );
        let (after, after_spans) = s.tex_spans(stage, &after_src, 30.0, flow_color(), p(3.9, 1.0))?;
        let after = s.matching(stage, (after_q, &after_q_spans), (after, &after_spans), 0.8)?;
        let each = s.tex(
            stage,
            &format!("\\text{{each face: }} {}", f.loop_each.tex()),
            30.0,
            flow_color(),
            p(2.2, 0.35),
        )?;
        s.appear(stage, &[each], 0.5)?;
        let innocent_ledger: Vec<Mob> = innocent.iter().flat_map(|&g| [ustar_row[g], unew_row[g]]).collect();
        s.pulse(stage, &innocent_ledger, 0.9)?;
        s.until(stage, "09_7", 7)?;
        let zero_pattern = travel(&[Q::ZERO; 4]);
        let not_zero = s.tex(
            stage,
            &format!("{} \\ne {}", sum_tex(&zero_pattern), f.loop_after.tex()),
            30.0,
            Palette::red(),
            p(5.7, 0.35),
        )?;
        s.appear(stage, &[not_zero], 0.5)?;
        s.pulse(stage, &[unew_row[0]], 0.8)?;

        // 09_8: an unfinished pressure; each cell's leftover imbalance is minus what its row missed by.
        s.say(stage, "09_8")?;
        let loop_panel = [title, before, pressure_label, closed, after, each, not_zero];
        s.vanish(stage, &loop_panel, 0.4)?;
        s.vanish(stage, &ring, 0.4)?;
        s.vanish(stage, &innocent_glow, 0.4)?;
        let identity_src = [
            "u_{\\text{new}} = u^* - \\operatorname{grad} p",
            "r = d - L p",
            "\\operatorname{div} u_{\\text{new}} = -r",
        ];
        let mut identity = Vec::new();
        for (k, src) in identity_src.iter().enumerate() {
            identity.push(s.tex_spans(stage, src, 34.0, ink, p(3.6, 2.9 - 0.65 * k as f64))?);
        }
        s.appear(stage, &[identity[0].0, identity[1].0], 0.5)?;
        s.set_pressures(stage, &f.unfinished, 0.8)?;
        s.morph_row(stage, &p_row, &f.unfinished, L_P, ink, 0.8)?;
        let unfinished_note = s.text_left(stage, "unfinished", 22.0, Palette::gold(), p(NOTE_X, L_P))?;
        s.morph(stage, note, unfinished_note, 0.5)?;
        s.set_arrows(stage, Lane::Push, &f.unfinished_push, 0.8)?;
        s.set_arrows(stage, Lane::Flow, &f.unfinished_flow, 0.8)?;
        s.set_flow_labels(stage, &f.unfinished_flow, 0.8)?;
        s.morph_row(stage, &push_row, &f.unfinished_push, L_PUSH, push_color(), 0.8)?;
        s.morph_row(stage, &unew_row, &f.unfinished_flow, L_UNEW, flow_color(), 0.8)?;
        s.set_caption(stage, "unfinished pressure", 0.5)?;
        s.until(stage, "09_8", 1)?;
        s.set_badges(stage, &f.unfinished_div, 0.7)?;
        let mut missed = Vec::new();
        for (i, &r) in f.unfinished_residual.iter().enumerate() {
            missed.push(s.tex(stage, &r.tex(), 26.0, Palette::gold(), at_cell(i, 0.0, 1.3))?);
        }
        let missed_label = s.text_left(
            stage,
            "missed by",
            22.0,
            Palette::gold(),
            p(-6.65, cell_center(0)[1] - 1.3),
        )?;
        s.appear(stage, &missed, 0.5)?;
        s.appear(stage, &[missed_label, identity[2].0], 0.5)?;
        s.play(stage)?;
        let pairs: Vec<(Mob, Mob)> = s.badge_values.iter().copied().zip(missed.iter().copied()).collect();
        for (badge, miss) in pairs {
            s.pulse(stage, &[badge, miss], 0.6)?;
            s.play(stage)?;
        }

        // 09_9: back to the exact pressures; the smooth-field forms beside the box's forms.
        s.say(stage, "09_9")?;
        s.vanish(stage, &missed, 0.4)?;
        s.vanish(stage, &[missed_label, identity[1].0, identity[2].0], 0.4)?;
        s.set_pressures(stage, &f.mean_zero, 0.7)?;
        s.morph_row(stage, &p_row, &f.mean_zero, L_P, ink, 0.7)?;
        let mean_again = s.text_left(stage, "mean zero", 22.0, dim, p(NOTE_X, L_P))?;
        s.morph(stage, note, mean_again, 0.5)?;
        s.set_arrows(stage, Lane::Push, &f.push, 0.7)?;
        s.set_arrows(stage, Lane::Flow, &f.corrected, 0.7)?;
        s.set_flow_labels(stage, &f.corrected, 0.7)?;
        s.set_badges(stage, &f.corrected_div, 0.7)?;
        s.morph_row(stage, &push_row, &f.push, L_PUSH, push_color(), 0.7)?;
        s.morph_row(stage, &unew_row, &f.corrected, L_UNEW, flow_color(), 0.7)?;
        s.set_caption(stage, "corrected flow", 0.5)?;
        let box_head = format!("\\text{{box: }} h = \\rho = \\Delta t = {}", UNITS.h);
        let heads = [
            s.tex(stage, &box_head, 26.0, dim, p(2.0, 3.3))?,
            s.tex(stage, "\\text{smooth fields}", 26.0, dim, p(5.6, 3.3))?,
        ];
        let box_poisson = s.tex(
            stage,
            "\\operatorname{div}\\operatorname{grad} p = \\operatorname{div} u^*",
            32.0,
            ink,
            p(2.0, 2.6),
        )?;
        let (box_update, box_update_spans) = s.tex_spans(stage, identity_src[0], 32.0, ink, p(2.0, 1.85))?;
        let box_update = s.matching(
            stage,
            (identity[0].0, &identity[0].1),
            (box_update, &box_update_spans),
            0.7,
        )?;
        s.appear(stage, &heads, 0.5)?;
        s.appear(stage, &[box_poisson], 0.5)?;
        s.until(stage, "09_9", 1)?;
        let smooth_poisson = s.tex(
            stage,
            "\\nabla^2 p = \\tfrac{\\rho}{\\Delta t}\\,\\nabla\\cdot u^*",
            32.0,
            ink,
            p(5.6, 2.6),
        )?;
        s.appear(stage, &[smooth_poisson], 0.6)?;
        s.until(stage, "09_9", 2)?;
        let smooth_update = s.tex(
            stage,
            "u_{\\text{new}} = u^* - \\tfrac{\\Delta t}{\\rho}\\,\\nabla p",
            32.0,
            ink,
            p(5.6, 1.85),
        )?;
        s.appear(stage, &[smooth_update], 0.6)?;
        s.play(stage)?;
        stage.wait(1.2)?;
        s.vanish(stage, &[s.tag], 0.6)?;

        // 09_10: not the barometer's pressure; only differences enter.
        s.say(stage, "09_10")?;
        let big_p = s.tex(stage, "p", 48.0, ink, p(2.3, 0.6))?;
        s.appear(stage, &[big_p], 0.4)?;
        s.pulse(stage, &p_row, 0.8)?;
        s.until(stage, "09_10", 1)?;
        let dial = [3.35, 0.6];
        let mut barometer = vec![
            stage.arena_mut().add(
                Circle::new()
                    .radius(0.34)
                    .style(Style::default().fill(ink, 0.0).stroke(ink, 2.0, 1.0))
                    .build()
                    .moved_to(p(dial[0], dial[1])),
            ),
        ];
        for k in 0..5 {
            let angle = std::f64::consts::PI * (0.9 - 0.2 * f64::from(k));
            let (c0, s0) = (angle.cos(), angle.sin());
            barometer.push(s.line(
                stage,
                p(dial[0] + 0.25 * c0, dial[1] + 0.25 * s0),
                p(dial[0] + 0.32 * c0, dial[1] + 0.32 * s0),
                ink,
                1.4,
            )?);
        }
        barometer.push(s.line(stage, p(dial[0], dial[1]), p(dial[0] + 0.17, dial[1] + 0.17), ink, 2.0)?);
        s.appear(stage, &barometer, 0.5)?;
        s.play(stage)?;
        stage.wait(0.5)?;
        let strike = s.line(
            stage,
            p(dial[0] - 0.45, dial[1] - 0.45),
            p(dial[0] + 0.45, dial[1] + 0.45),
            Palette::red(),
            4.0,
        )?;
        s.appear(stage, &[strike], 0.3)?;
        s.until(stage, "09_10", 2)?;
        s.pulse(stage, &push_row, 0.8)?;
        let pushes: Vec<Mob> = s.push.iter().flatten().copied().collect();
        s.pulse(stage, &pushes, 0.8)?;

        // 09_11: div and grad from the same faces; the walls shape the matrix.
        s.say(stage, "09_11")?;
        let mut forms = vec![
            heads[0],
            heads[1],
            box_poisson,
            box_update,
            smooth_poisson,
            smooth_update,
            big_p,
            strike,
        ];
        forms.extend(barometer.iter().copied());
        s.vanish(stage, &forms, 0.5)?;
        s.appear(stage, &matrix_all, 0.6)?;
        s.until(stage, "09_11", 1)?;
        let all_faces: Vec<Mob> = (0..4).map(|g| glow(&s, stage, g)).collect::<fmn::Result<_>>()?;
        let words = [
            s.tex(
                stage,
                "\\operatorname{div}",
                30.0,
                ink,
                p(BOX[0] - 1.85, BOX[1] + SIDE + 0.72),
            )?,
            s.text(
                stage,
                "same faces, same cells",
                20.0,
                hero,
                p(BOX[0], BOX[1] + SIDE + 0.72),
            )?,
            s.tex(
                stage,
                "\\operatorname{grad}",
                30.0,
                push_color(),
                p(BOX[0] + 1.85, BOX[1] + SIDE + 0.72),
            )?,
        ];
        s.appear(stage, &all_faces, 0.5)?;
        s.appear(stage, &words, 0.5)?;
        s.play(stage)?;
        let badges = s.badges;
        s.pulse(stage, &badges, 0.8)?;
        s.pulse(stage, &pushes, 0.8)?;
        s.until(stage, "09_11", 2)?;
        s.vanish(stage, &all_faces, 0.3)?;
        // Emphasize the boundary in place so the stroke clears the wall labels.
        let boundary = indicate(stage.arena_mut(), s.outline, 1.0, None)?.rt(0.9);
        s.pending.push(stage.prepare(boundary)?);
        let walls = s.walls.clone();
        s.pulse(stage, &walls, 0.9)?;
        let mut again = Vec::new();
        for (&x, &y) in MX.iter().zip(&MY) {
            again.push(s.frame(stage, p(x, y), 0.6, 0.5, ink, 2.4)?);
        }
        let corner = f.open_faces[ROW_A];
        let walls_note = format!(
            "\\text{{interior }} {} \\to \\text{{corner }} {corner}",
            f.interior_open_faces
        );
        let four_to_two = s.tex(stage, &walls_note, 28.0, hero, p(2.5, 0.15))?;
        s.appear(stage, &again, 0.5)?;
        s.appear(stage, &[four_to_two], 0.5)?;

        // 09_12: the tunnel has the same structure with far more cells; the question card glows once.
        s.say(stage, "09_12")?;
        let mut everything = s.on.clone();
        everything.retain(|m| !s.fills.contains(m));
        s.vanish(stage, &everything, 0.6)?;
        for m in s.fills {
            s.refill(stage, m, Palette::teal(), 0.0, 0.6)?;
        }
        let small = tunnel(stage, geometry, 2.0 * SIDE, BOX)?;
        let full = tunnel(stage, geometry, TUNNEL_WIDTH, [0.0, TUNNEL_CENTER_Y])?;
        s.appear(stage, &small, 0.6)?;
        s.play(stage)?;
        let fills = s.fills.to_vec();
        s.cut(stage, &fills);
        for (&from, &to) in small.iter().zip(&full) {
            s.morph(stage, from, to, 1.4)?;
        }
        let card_frame = stage.arena_mut().add(
            Rectangle::new()
                .width(8.2)
                .height(0.8)
                .corner_radius(0.12)
                .style(Style::default().fill(ink, 0.0).stroke(dim, 2.0, 1.0))
                .build()?
                .moved_to(p(0.0, -1.25)),
        );
        let card_words = s.text(
            stage,
            "Does halving the cells change the answer?",
            30.0,
            ink,
            p(0.0, -1.25),
        )?;
        s.appear(stage, &[card_frame, card_words], 0.6)?;
        s.until(stage, "09_12", 1)?;
        let grid: Vec<Mob> = small.iter().skip(2).copied().collect();
        s.pulse(stage, &grid, 0.9)?;
        s.until(stage, "09_12", 2)?;
        s.play(stage)?;
        stage.wait(1.0)?;
        s.pulse(stage, &[card_frame, card_words], 1.0)?;
        s.play(stage)?;
        println!(
            "Chapter 09 ends on the chapter 10 start: the full tunnel ({} objects) and the question card",
            small.len()
        );
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_run_in_order() {
        let ids: Vec<&str> = CUES.iter().map(|&(id, _)| id).collect();
        let expected: Vec<String> = (1..=12).map(|k| format!("09_{k}")).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        assert!(CUES.iter().all(|&(_, sentences)| sentences > 0));
    }

    #[test]
    fn chapters_08_and_09_register_every_box_row_once() {
        let mut film = Facts::new();
        super::super::chapter08::register(&mut film).expect("chapter 08 registers first");
        let before = film.len();
        register(&mut film).expect("chapter 09 joins without a clash");
        assert_eq!(film.len(), before + PINS.len());
        film.self_check().expect("every pin holds in the film registry");
        for (key, value) in BoxFacts::solve().fact_rows() {
            assert_eq!(film.get(&key), Some(value.as_str()), "{key} prints one way in the film");
        }
        let alone = facts().expect("chapter 09 alone");
        assert_eq!(alone.len(), PINS.len());
        assert!(
            super::super::chapter08::SHARED_FACTS
                .iter()
                .all(|key| alone.get(key).is_none())
        );
    }

    #[test]
    fn value_labels_use_the_shared_placement() {
        // The labels chapter 09 opens on are where chapter 08 ends them: beside the line, not on it.
        for g in 0..FACES.len() {
            let at = box_view::flow_label_at(g);
            assert_eq!(at, on_face(g, LANE, box_view::FLOW_LABEL_OFFSET));
            assert_ne!(at, on_face(g, LANE, 0.0));
        }
    }

    #[test]
    fn sums_print_their_signs() {
        assert_eq!(sum_tex(&[Q::ONE, Q::ZERO, Q::new(-1, 4)]), "1 + 0 - \\tfrac{1}{4}");
    }
}
