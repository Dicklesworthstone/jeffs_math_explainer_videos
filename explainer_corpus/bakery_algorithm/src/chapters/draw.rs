//! Video C's look, shared by every chapter (storyboard §3, colour roles):
//! the five customers and their badges, slates with chalk tickets, the room,
//! flags, the scoreboard, space-time lanes with read and write arrows, the
//! progress square with its staircases, and the gold scanline of the order.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation, or `add_to_scene`.
//! Engine rules (`explainer_kit::kit`): one flat fill per mobject, no
//! outline on big filled shapes, motion by placement only, small groups.
//!
//! Three lanes build Video C's thirteen chapters from this one module at the
//! same time, so a prop may land before the chapter that first draws it.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::animatic::wrap;
use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, hex, v};
use explainer_score::rng::SplitMix64;
use fmn::animation::UpdateFromFunc;
use fmn::prelude::*;

use crate::model::{Order, Pc, State, Step};

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders), as the
/// kit's scratch voice estimates them.
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began and how long it runs, so a picture
/// can land on the phrase it illustrates. A phrase's moment is its share of
/// the line's characters (the voice reads at a nearly even rate); the cue
/// sheets of the narrated draft check the result.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
}

/// Speak line `id` now (after the previous line), as `Kit::say`.
pub fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Spoken> {
    kit.say(stage, id)?;
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?
        .to_owned();
    let words = text.split_whitespace().count() as f64;
    let len = kit
        .narrator
        .as_ref()
        .and_then(|n| n.length(id))
        .unwrap_or(words / SILENT_WPS + 0.4);
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
    })
}

impl Spoken {
    /// When `phrase` begins (scene seconds).
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let i = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        let before = self.text[..i].chars().count() as f64;
        let all = self.text.chars().count().max(1) as f64;
        Ok(self.start + self.len * before / all)
    }

    /// When the line ends.
    pub fn end(&self) -> f64 {
        self.start + self.len
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

// ------------------------------------------------------------- colour roles

/// The customers' names, spoken and shown (customer number = index + 1).
pub const NAMES: [&str; 5] = ["Blue", "Green", "Purple", "Teal", "Pink"];

/// The customers' hues, customer 1 first. Every pair, and every customer
/// against every role colour, is at least ΔE2000 20 apart (R1 creative C-3;
/// the test below).
const HUES: [&str; 5] = ["#5AA9FA", "#A6E05A", "#B48CFF", "#45DDB5", "#FF8FC8"];

/// Customer `k`'s hue (1-based).
///
/// # Panics
/// If `k` is not a customer number, 1 to 5.
pub fn customer(k: usize) -> Srgb {
    hex(HUES[k - 1])
}

/// The room (the critical section): warm red.
pub fn room_red() -> Srgb {
    hex("#E8604C")
}

/// A raised flag (choosing): amber.
pub fn flag_amber() -> Srgb {
    hex("#F0AC5F")
}

/// Tickets and slates: chalk white.
pub fn chalk() -> Srgb {
    Palette::ink()
}

/// The order (ticket, customer number): the gold scanline.
pub fn order_gold() -> Srgb {
    Palette::hero()
}

/// Store buffers ("outboxes") and hardware: cool grey.
pub fn hardware() -> Srgb {
    hex("#6F7886")
}

/// A slate's board, under the chalk.
pub fn board() -> Srgb {
    hex("#1C2530")
}

/// The figures' eyes and the badges' numerals: the background's ink.
fn dark() -> Srgb {
    hex("#0E1116")
}

// ------------------------------------------------------------------ styles

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill.
pub fn line_style(c: Srgb, w: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, 1.0)
}

/// A stroke at an opacity, and no fill.
pub fn faded_line(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// An empty parent in the arena holding mobjects already there.
pub fn group(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = stage.arena_mut().add(VMobject::new());
    for &m in members {
        stage.attach(g, m)?;
    }
    Ok(g)
}

// ---------------------------------------------------------------- customers

/// A customer: a friendly rounded figure in her colour, two eyes, and her
/// customer number on a chalk badge.
pub struct Figure {
    /// The whole figure (move this).
    pub group: Mob,
    pub body: Mob,
    pub pupils: [Mob; 2],
    pub badge: Mob,
    pub k: usize,
}

/// The figure's height at scale 1.
pub const FIGURE_H: f64 = 0.72;
/// The figure's width at scale 1, and its rounded corners.
const FIGURE_W: f64 = 0.56;
const FIGURE_CORNER: f64 = 0.26;
/// The figure's eyes: their height above its centre, and their radius.
const EYE_Y: f64 = 0.17;
const EYE_R: f64 = 0.075;
/// The belly badge's radius and centre height at scale 1.
const BADGE_R: f64 = 0.2;
const BADGE_Y: f64 = -0.14;

/// Customer `k` (1-based) standing at `at`, `scale` × the standard size.
pub fn figure(kit: &Kit, stage: &mut Stage<'_>, k: usize, at: Vec3, scale: f64) -> fmn::Result<Figure> {
    let s = scale;
    let p = |x: f64, y: f64| v(at[0] + x * s, at[1] + y * s);
    let body = Rectangle::new()
        .width(FIGURE_W * s)
        .height(FIGURE_H * s)
        .corner_radius(FIGURE_CORNER * s)
        .style(flat(customer(k), 1.0))
        .build()?
        .moved_to(at);
    let body = stage.arena_mut().add(body);
    let mut parts = vec![body];
    let mut pupils = Vec::new();
    for side in [-1.0, 1.0] {
        let white = Circle::new()
            .radius(EYE_R * s)
            .arc_center(p(0.11 * side, EYE_Y))
            .style(flat(chalk(), 1.0))
            .build();
        let pupil = Circle::new()
            .radius(0.037 * s)
            .arc_center(p(0.11 * side + 0.012, 0.165))
            .style(flat(dark(), 1.0))
            .build();
        parts.push(stage.arena_mut().add(white));
        let pm = stage.arena_mut().add(pupil);
        parts.push(pm);
        pupils.push(pm);
    }
    // the badge as big as the belly allows, so the number reads on a phone
    // (R3 maya-08): clear of the eyes above, inside the rounded body below
    let disc = Circle::new()
        .radius(BADGE_R * s)
        .arc_center(p(0.0, BADGE_Y))
        .style(flat(chalk(), 1.0))
        .build();
    let disc = stage.arena_mut().add(disc);
    let num = kit.text(stage, &k.to_string(), 160.0 * BADGE_R * s, dark(), p(0.0, BADGE_Y))?;
    let badge = group(stage, &[disc, num])?;
    parts.push(badge);
    let g = group(stage, &parts)?;
    Ok(Figure {
        group: g,
        body,
        pupils: [pupils[0], pupils[1]],
        badge,
        k,
    })
}

/// A customer's badge alone (her colour, her number): for lanes, grids and
/// pairs.
pub fn badge(kit: &Kit, stage: &mut Stage<'_>, k: usize, radius: f64, at: Vec3) -> fmn::Result<Mob> {
    let disc = Circle::new()
        .radius(radius)
        .arc_center(at)
        .style(flat(customer(k), 1.0))
        .build();
    let disc = stage.arena_mut().add(disc);
    let num = kit.text(stage, &k.to_string(), radius * 150.0, dark(), at)?;
    group(stage, &[disc, num])
}

// ------------------------------------------------------------------- slates

/// A slate: a small board with a chalk ticket on it.
pub struct Slate {
    pub board: Mob,
    /// The chalk numeral now on it (replace it with [`Slate::rewrite`]).
    pub value: Mob,
    pub center: Vec3,
    pub size: f64,
}

/// A slate `size` units tall (the board is 1.4× as wide) showing `value`.
pub fn slate(kit: &Kit, stage: &mut Stage<'_>, value: u32, at: Vec3, size: f64) -> fmn::Result<Slate> {
    let b = Rectangle::new()
        .width(1.4 * size)
        .height(size)
        .corner_radius(0.12 * size)
        .style(Style::default().fill(board(), 1.0).stroke(hardware(), 2.5, 1.0))
        .build()?
        .moved_to(at);
    let board = stage.arena_mut().add(b);
    let value = chalk_number(kit, stage, value, at, size)?;
    Ok(Slate {
        board,
        value,
        center: at,
        size,
    })
}

/// A chalk numeral sized for a slate `size` tall.
pub fn chalk_number(kit: &Kit, stage: &mut Stage<'_>, value: u32, at: Vec3, size: f64) -> fmn::Result<Mob> {
    kit.text(stage, &value.to_string(), 62.0 * size, chalk(), at)
}

impl Slate {
    /// Both parts, as one group (for fades and moves).
    pub fn parts(&self) -> [Mob; 2] {
        [self.board, self.value]
    }

    /// Wipe the slate and write `value` in chalk: the old numeral fades
    /// down, the new one is written. Returns the two animations, to play
    /// with others.
    pub fn rewrite(&mut self, kit: &Kit, stage: &mut Stage<'_>, value: u32) -> fmn::Result<[Box<dyn Animation>; 2]> {
        let old = self.value;
        let new = chalk_number(kit, stage, value, self.center, self.size)?;
        self.value = new;
        let out = fade_out(stage.arena_mut(), old, v(0.0, -0.08 * self.size), 1.0)?;
        let out = stage.prepare(out)?;
        let w = write(stage.arena(), new);
        let w = stage.prepare(w)?;
        Ok([out, w])
    }
}

/// Numerals stacked in one place, for a glance that catches a write: one
/// chalk numeral per value, all detached; [`flicker`] shows one at a time.
pub fn flicker_numerals(
    kit: &Kit,
    stage: &mut Stage<'_>,
    values: &[u32],
    at: Vec3,
    size: f64,
) -> fmn::Result<Vec<Mob>> {
    values.iter().map(|&x| chalk_number(kit, stage, x, at, size)).collect()
}

/// `secs` of flicker over `mobs` at `hz` changes a second, in a seeded
/// jittery order, ending on `mobs[last]` (the others hidden).
pub fn flicker(stage: &mut Stage<'_>, mobs: &[Mob], secs: f64, hz: f64, last: usize, seed: u64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    for &m in mobs {
        stage.set_opacity(m, 0.0, true);
        stage.add_to_scene(m)?;
    }
    let mut rng = SplitMix64::new(seed);
    let changes = (secs * hz).ceil().max(1.0) as usize;
    let mut order = Vec::with_capacity(changes);
    let mut prev = usize::MAX;
    for _ in 0..changes {
        let mut k = (rng.next_u64() % mobs.len() as u64) as usize;
        if k == prev {
            k = (k + 1) % mobs.len();
        }
        order.push(k);
        prev = k;
    }
    let shown: Vec<Mob> = mobs.to_vec();
    let anchor = stage.add(VMobject::new())?;
    let run = UpdateFromFunc::new_alpha(anchor, move |s, _, alpha| {
        let slot = ((alpha * changes as f64) as usize).min(changes - 1);
        let k = if alpha >= 1.0 { last } else { order[slot] };
        for (i, &m) in shown.iter().enumerate() {
            s.set_opacity(m, if i == k { 1.0 } else { 0.0 }, true);
        }
    });
    stage.play_prepared_with(
        vec![Box::new(run)],
        PlayOverrides {
            run_time: Some(secs),
            rate_func: Some(RateFunc::linear()),
            ..PlayOverrides::default()
        },
    )?;
    stage.remove_from_scene(anchor);
    Ok(())
}

// ------------------------------------------------------------------- quotes

/// A quote on a card: the words verbatim in italics, wrapped to `chars` a
/// line, and its source beneath in dim type. Detached, centred at `at`.
pub fn quote_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    words: &str,
    source: &str,
    at: Vec3,
    chars: usize,
) -> fmn::Result<Mob> {
    let lines = wrap(words, chars, 6);
    let line_h = 0.46;
    let h = 0.55 + line_h * lines.len() as f64 + 0.42;
    let mut mobs = Vec::new();
    let top = at[1] + h / 2.0 - 0.3 - line_h / 2.0;
    let mut widest: f64 = 0.0;
    for (i, l) in lines.iter().enumerate() {
        let m = kit.text_styled(
            stage,
            l,
            30.0,
            Palette::ink(),
            v(at[0], top - line_h * i as f64),
            false,
            true,
        )?;
        widest = widest.max(stage.get_width(m));
        mobs.push(m);
    }
    let src = kit.text(stage, source, 22.0, Palette::dim(), v(at[0], at[1] - h / 2.0 + 0.32))?;
    widest = widest.max(stage.get_width(src));
    let back = Rectangle::new()
        .width(widest + 0.8)
        .height(h)
        .corner_radius(0.14)
        .style(
            Style::default()
                .fill(hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let mut all = vec![back];
    all.extend(mobs);
    all.push(src);
    group(stage, &all)
}

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut v: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        v.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(v)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut v: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        v.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(v)?;
    Ok(())
}

// ----------------------------------------------------------------- the room

/// The room's tint. The renderer blends in linear light, so a fill this
/// faint already reads as a clear warm wash on the dark background.
pub const ROOM_TINT: f64 = 0.045;

/// The room: a warm red tinted space with a red outline (no outline under
/// moving things: the outline is a separate, still mobject).
pub fn room(w: f64, h: f64, at: Vec3) -> fmn::Result<[VMobject; 2]> {
    let tint = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.22)
        .style(flat(room_red(), ROOM_TINT))
        .build()?
        .moved_to(at);
    let edge = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.22)
        .style(line_style(room_red(), 3.5))
        .build()?
        .moved_to(at);
    Ok([tint, edge])
}

// -------------------------------------------------------------------- flags

/// A flag on a short pole: amber when raised, a dim stub when lowered.
pub struct Flag {
    pub pole: Mob,
    pub pennant: Mob,
    /// The pole's foot.
    pub foot: Vec3,
    pub height: f64,
    pub raised: bool,
}

/// A flag `height` tall with its pole's foot at `foot`.
pub fn flag(stage: &mut Stage<'_>, foot: Vec3, height: f64, raised: bool) -> fmn::Result<Flag> {
    let pole = Line::new(foot, v(foot[0], foot[1] + height))
        .style(line_style(Palette::dim(), 3.0))
        .build()?;
    let pole = stage.arena_mut().add(pole);
    let pennant = stage.arena_mut().add(pennant_shape(foot, height, raised));
    Ok(Flag {
        pole,
        pennant,
        foot,
        height,
        raised,
    })
}

fn pennant_shape(foot: Vec3, height: f64, raised: bool) -> VMobject {
    let (top, color, o) = if raised {
        (foot[1] + height, flag_amber(), 1.0)
    } else {
        (foot[1] + 0.36 * height, Palette::faint(), 1.0)
    };
    let d = 0.36 * height;
    Polygon::new([
        v(foot[0], top),
        v(foot[0] + 0.62 * height, top - d / 2.0),
        v(foot[0], top - d),
    ])
    .style(flat(color, o))
    .build()
}

impl Flag {
    /// Raise or lower the flag: the pennant slides along the pole and
    /// changes colour.
    pub fn set(&mut self, stage: &mut Stage<'_>, raised: bool) -> Transform {
        self.raised = raised;
        let target = stage.arena_mut().add(pennant_shape(self.foot, self.height, raised));
        Transform::new(self.pennant, target)
    }
}

// ------------------------------------------------------------ the door sign

/// The sign on the room's door (02): "vacant" in chalk or "occupied" in
/// red, on a small board hung from a nail. The magic sign (02_7, 13_1) is
/// the same board with a gold edge.
pub struct Sign {
    pub board: Mob,
    pub hanger: Mob,
    pub text: Mob,
    pub center: Vec3,
    pub size: f64,
    pub occupied: bool,
}

/// The sign's words, by state.
pub const VACANT: &str = "vacant";
pub const OCCUPIED: &str = "occupied";

fn sign_text(kit: &Kit, stage: &mut Stage<'_>, occupied: bool, at: Vec3, size: f64) -> fmn::Result<Mob> {
    let (word, color) = if occupied {
        (OCCUPIED, room_red())
    } else {
        (VACANT, chalk())
    };
    kit.text(stage, word, 30.0 * size, color, at)
}

/// A door sign `size` × the standard size, centred at `at`; `magic` gives it
/// the gold edge of an atomic check-and-flip.
pub fn sign(kit: &Kit, stage: &mut Stage<'_>, occupied: bool, at: Vec3, size: f64, magic: bool) -> fmn::Result<Sign> {
    let (w, h) = (1.9 * size, 0.62 * size);
    let edge = if magic { order_gold() } else { hardware() };
    let b = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.1 * size)
        .style(
            Style::default()
                .fill(board(), 1.0)
                .stroke(edge, if magic { 4.0 } else { 2.5 }, 1.0),
        )
        .build()?
        .moved_to(at);
    let board = stage.arena_mut().add(b);
    let top = at[1] + h / 2.0;
    let nail = v(at[0], top + 0.3 * size);
    let mut parts = Vec::new();
    for dx in [-0.55, 0.55] {
        parts.push(
            Line::new(v(at[0] + dx * size, top), nail)
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
    }
    parts.push(
        Circle::new()
            .radius(0.05 * size)
            .arc_center(nail)
            .style(flat(Palette::dim(), 1.0))
            .build(),
    );
    let hanger = stage.arena_mut().add(v_group(parts));
    let text = sign_text(kit, stage, occupied, at, size)?;
    Ok(Sign {
        board,
        hanger,
        text,
        center: at,
        size,
        occupied,
    })
}

impl Sign {
    pub fn parts(&self) -> [Mob; 3] {
        [self.hanger, self.board, self.text]
    }

    /// Flip the sign to `occupied` (or back): the board turns edge-on, the
    /// words change, and it turns back, `rt` seconds in all.
    pub fn flip(&mut self, kit: &Kit, stage: &mut Stage<'_>, occupied: bool, rt: f64) -> fmn::Result<()> {
        let half = rt / 2.0;
        let narrow = |m: Mob| -> fmn::Result<AnimBuilder> {
            Ok(m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(half),
                    rate_func: Some(fmn::core::rate::rush_into),
                    ..AnimateArgs::default()
                })?
                .stretch(0.04, 0)?)
        };
        let (b, t) = (narrow(self.board)?, narrow(self.text)?);
        play_two(stage, b, t)?;
        let old = self.text;
        let new = sign_text(kit, stage, occupied, self.center, self.size)?;
        stage.stretch_about(new, 0.04, 0, Some(self.center), None);
        stage.remove_from_scene(old);
        stage.add_to_scene(new)?;
        self.text = new;
        self.occupied = occupied;
        let widen = |m: Mob, f: f64| -> fmn::Result<AnimBuilder> {
            Ok(m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(half),
                    rate_func: Some(fmn::core::rate::rush_from),
                    ..AnimateArgs::default()
                })?
                .stretch(f, 0)?)
        };
        let (b, t) = (widen(self.board, 25.0)?, widen(new, 25.0)?);
        play_two(stage, b, t)?;
        Ok(())
    }
}

fn play_two(stage: &mut Stage<'_>, a: AnimBuilder, b: AnimBuilder) -> fmn::Result<()> {
    let a = stage.prepare(a)?;
    let b = stage.prepare(b)?;
    stage.play_prepared(vec![a, b])?;
    Ok(())
}

// ------------------------------------------------- Dijkstra's four rules

/// The four rules (02_5–02_6) as numbered pictograms, so 08_8 can tick rule
/// 1 and 11_2 rule 4 on the same pictures:
/// 1. never two in the room: the progress square's red middle, walled off;
/// 2. someone stopped outside can't hold up anyone else: a dimmed, halted
///    lane while the other passes into the room;
/// 3. fast or slow, it still works: a sparse lane and a dense one, both in;
/// 4. everyone who wants in gets in eventually: each lane reaches the room.
pub fn pictogram(kit: &Kit, stage: &mut Stage<'_>, rule: usize, at: Vec3, size: f64) -> fmn::Result<Mob> {
    let u = size / 2.0;
    let p = |x: f64, y: f64| v(at[0] + x * u, at[1] + y * u);
    let frame = Rectangle::new()
        .width(size)
        .height(size)
        .corner_radius(0.12 * size)
        .style(
            Style::default()
                .fill(hex("#151B24"), 1.0)
                .stroke(Palette::faint(), 2.0, 1.0),
        )
        .build()?
        .moved_to(at);
    let mut shapes = vec![frame];
    let lane_w = (4.0 * size).max(1.5);
    let room_band = |x0: f64, x1: f64| -> fmn::Result<VMobject> {
        Ok(Rectangle::new()
            .width((x1 - x0) * u)
            .height(1.2 * u)
            .style(flat(room_red(), 0.35))
            .build()?
            .moved_to(p((x0 + x1) / 2.0, -0.05)))
    };
    let dots = |stage_shapes: &mut Vec<VMobject>, k: usize, y: f64, xs: &[f64]| {
        for &x in xs {
            stage_shapes.push(
                Dot::new()
                    .point(p(x, y))
                    .radius(0.035 * size)
                    .color(customer(k))
                    .build(),
            );
        }
    };
    match rule {
        1 => {
            let l = Lattice {
                center: p(0.05, -0.08),
                side: 1.15 * u,
                steps: 2,
            };
            for i in 0..=2 {
                let f = i as f64;
                shapes.push(
                    Line::new(l.p(f, 0.0), l.p(f, 2.0))
                        .style(line_style(Palette::faint(), 1.5))
                        .build()?,
                );
                shapes.push(
                    Line::new(l.p(0.0, f), l.p(2.0, f))
                        .style(line_style(Palette::faint(), 1.5))
                        .build()?,
                );
            }
            shapes.push(l.mark(1, 1, room_red()));
            shapes.push(
                polyline([l.p(0.0, 0.0), l.p(2.0, 0.0), l.p(2.0, 2.0)])
                    .style(line_style(chalk(), lane_w))
                    .build(),
            );
        }
        2 => {
            shapes.push(room_band(0.35, 0.75)?);
            shapes.push(
                Line::new(p(-0.8, 0.3), p(0.75, 0.3))
                    .style(line_style(customer(1), lane_w))
                    .build()?,
            );
            shapes.push(
                Line::new(p(-0.8, -0.35), p(-0.2, -0.35))
                    .style(faded_line(customer(2), lane_w, 0.35))
                    .build()?,
            );
            for dx in [-0.08, 0.04] {
                shapes.push(
                    Line::new(p(-0.1 + dx, -0.5), p(-0.1 + dx, -0.2))
                        .style(line_style(customer(2), lane_w))
                        .build()?,
                );
            }
        }
        3 => {
            shapes.push(room_band(0.35, 0.75)?);
            shapes.push(
                Line::new(p(-0.8, 0.3), p(0.6, 0.3))
                    .style(faded_line(customer(1), lane_w, 0.6))
                    .build()?,
            );
            shapes.push(
                Line::new(p(-0.8, -0.35), p(0.6, -0.35))
                    .style(faded_line(customer(2), lane_w, 0.6))
                    .build()?,
            );
            dots(&mut shapes, 1, 0.3, &[-0.7, -0.15, 0.45]);
            dots(
                &mut shapes,
                2,
                -0.35,
                &[-0.7, -0.52, -0.34, -0.16, 0.02, 0.2, 0.38, 0.56],
            );
        }
        _ => {
            for (k, y, x0) in [(1, 0.3, -0.55), (2, -0.35, 0.15)] {
                shapes.push(
                    Line::new(p(-0.8, y), p(0.8, y))
                        .style(faded_line(customer(k), lane_w, 0.6))
                        .build()?,
                );
                shapes.push(
                    Rectangle::new()
                        .width(0.42 * u)
                        .height(0.24 * u)
                        .style(flat(room_red(), 0.8))
                        .build()?
                        .moved_to(p(x0 + 0.21, y)),
                );
            }
        }
    }
    let body = stage.arena_mut().add(v_group(shapes));
    let num = kit.text(
        stage,
        &rule.to_string(),
        34.0 * size / 2.0,
        order_gold(),
        p(-0.78, 0.76),
    )?;
    group(stage, &[body, num])
}

/// A tick for a pictogram whose rule has been proved (08_8, 11_2), at its
/// upper-right corner.
pub fn rule_tick(at: Vec3, size: f64) -> VMobject {
    fmn::library::matchers::checkmark(Palette::green())
        .scaled_about(0.5 * size / 0.4, ORIGIN)
        .moved_to(v(at[0] + 0.36 * size, at[1] + 0.36 * size))
}

// --------------------------------------------------------------- scoreboard

/// The shared scoreboard of chapter 01: a dark panel with big digits.
pub struct Scoreboard {
    pub panel: Mob,
    pub value: Mob,
    pub center: Vec3,
    pub size: f64,
}

/// A scoreboard `size` units tall showing `value`.
pub fn scoreboard(kit: &Kit, stage: &mut Stage<'_>, value: u32, at: Vec3, size: f64) -> fmn::Result<Scoreboard> {
    let p = Rectangle::new()
        .width(1.7 * size)
        .height(size)
        .corner_radius(0.1 * size)
        .style(Style::default().fill(board(), 1.0).stroke(Palette::dim(), 3.0, 1.0))
        .build()?
        .moved_to(at);
    let panel = stage.arena_mut().add(p);
    let value = kit.text(stage, &value.to_string(), 80.0 * size, Palette::ink(), at)?;
    Ok(Scoreboard {
        panel,
        value,
        center: at,
        size,
    })
}

impl Scoreboard {
    /// Show `value` (the old digits fade, the new ones are written).
    pub fn show(&mut self, kit: &Kit, stage: &mut Stage<'_>, value: u32) -> fmn::Result<[Box<dyn Animation>; 2]> {
        let old = self.value;
        let new = kit.text(stage, &value.to_string(), 80.0 * self.size, Palette::ink(), self.center)?;
        self.value = new;
        let out = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
        let out = stage.prepare(out)?;
        let inn = fade_in(stage.arena_mut(), new, ORIGIN, 1.15)?;
        let inn = stage.prepare(inn)?;
        Ok([out, inn])
    }
}

// -------------------------------------------------------- space-time lanes

/// A customer's lane: time runs left to right along a line in her colour.
pub fn lane(x0: f64, x1: f64, y: f64, k: usize) -> fmn::Result<VMobject> {
    Ok(Line::new(v(x0, y), v(x1, y))
        .style(faded_line(customer(k), 4.0, 0.6))
        .build()?)
}

/// A read (an arrow in: from the memory to the reader's lane) or a write
/// (an arrow out: from the writer's lane to the memory), in the customer's
/// colour.
pub fn access_arrow(from: Vec3, to: Vec3, k: usize) -> fmn::Result<VMobject> {
    Ok(Arrow::new(from, to)
        .buff(0.08)
        .thickness(3.5)
        .style(Style::default().fill(customer(k), 1.0).stroke(customer(k), 3.5, 1.0))
        .build()?)
}

/// A step on a lane: a dot in the customer's colour.
pub fn step_dot(at: Vec3, k: usize) -> VMobject {
    Dot::new().point(at).radius(0.075).color(customer(k)).build()
}

// ----------------------------------------------------- the progress square

/// The progress square (chapters 01, 02, 05): the bottom side counts Blue's
/// steps, the left side Green's; every order is a staircase from the
/// lower-left corner to the upper-right.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lattice {
    pub center: Vec3,
    pub side: f64,
    /// Steps along each side.
    pub steps: usize,
}

impl Lattice {
    /// Lattice point (i, j): i of Blue's steps taken, j of Green's.
    pub fn p(&self, i: f64, j: f64) -> Vec3 {
        let n = self.steps as f64;
        v(
            self.center[0] + (i / n - 0.5) * self.side,
            self.center[1] + (j / n - 0.5) * self.side,
        )
    }

    pub fn cell(&self) -> f64 {
        self.side / self.steps as f64
    }

    /// The border and the inner grid lines, faint.
    pub fn grid(&self) -> fmn::Result<VMobject> {
        let n = self.steps as f64;
        let mut lines = Vec::new();
        for i in 0..=self.steps {
            let f = i as f64;
            let (w, c) = if i == 0 || i == self.steps {
                (2.5, Palette::dim())
            } else {
                (1.5, Palette::faint())
            };
            lines.push(
                Line::new(self.p(f, 0.0), self.p(f, n))
                    .style(line_style(c, w))
                    .build()?,
            );
            lines.push(
                Line::new(self.p(0.0, f), self.p(n, f))
                    .style(line_style(c, w))
                    .build()?,
            );
        }
        Ok(v_group(lines))
    }

    /// A square of the given colour around lattice point (i, j): the
    /// both-inside point, drawn red.
    pub fn mark(&self, i: usize, j: usize, color: Srgb) -> VMobject {
        let side = 0.46 * self.cell();
        Square::new()
            .side_length(side)
            .style(Style::default().fill(color, 0.35).stroke(color, 2.5, 1.0))
            .build()
            .moved_to(self.p(i as f64, j as f64))
    }

    /// An order's staircase, through its corners, pushed `offset` along the
    /// anti-diagonal so that a bundle of staircases fans out instead of
    /// collapsing onto shared edges (spike S-6). Draw [`Lattice::ends`] over
    /// the bundle so every staircase visibly leaves one corner and reaches
    /// the other.
    pub fn staircase(&self, order: &Order, offset: f64, style: Style) -> VMobject {
        let d = offset * std::f64::consts::FRAC_1_SQRT_2;
        polyline(order.corners().into_iter().map(|(i, j)| {
            let p = self.p(i as f64, j as f64);
            v(p[0] + d, p[1] - d)
        }))
        .style(style)
        .build()
    }

    /// The start and end of every order: a disc at each corner, wide enough
    /// to cover a bundle fanned over `fan_width` (see [`fan`]).
    pub fn ends(&self, fan_width: f64) -> [VMobject; 2] {
        let r = 0.5 * fan_width * std::f64::consts::FRAC_1_SQRT_2 + 0.1;
        let n = self.steps as f64;
        [(0.0, Palette::dim()), (n, Palette::dim())].map(|(c, color)| {
            Circle::new()
                .radius(r)
                .arc_center(self.p(c, c))
                .style(Style::default().fill(hex("#151B24"), 1.0).stroke(color, 2.5, 1.0))
                .build()
        })
    }
}

/// The bundle offset of staircase `r` of `n`, spread over `width`.
pub fn fan(r: usize, n: usize, width: f64) -> f64 {
    if n <= 1 {
        return 0.0;
    }
    (0.5 - (r as f64 + 0.5) / n as f64) * width
}

// --------------------------------------------------------- the gold order

/// The gold scanline of the order (ticket, customer number): a bright core
/// over a soft glow, vertical from `y0` to `y1` at `x`.
pub fn scanline(x: f64, y0: f64, y1: f64) -> fmn::Result<VMobject> {
    let glow = Line::new(v(x, y0), v(x, y1))
        .style(faded_line(order_gold(), 14.0, 0.18))
        .build()?;
    let core = Line::new(v(x, y0), v(x, y1))
        .style(line_style(order_gold(), 4.0))
        .build()?;
    Ok(v_group([glow, core]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_room_band_is_the_models_room() {
        use crate::model::Trace;
        // every trace in which the model never has two in the room: never two
        // red bands at one step, and a band wherever the model counts her in
        for t in [
            Trace::APrime,
            Trace::C,
            Trace::CThree,
            Trace::D,
            Trace::E1,
            Trace::E3,
            Trace::Leapfrog,
        ] {
            check(&t.model(), &t.steps(), &format!("{t:?}"));
        }
        // e2: the checker's scenario, E1 then a glance during Blue's exit write
        let m = Trace::E1.model();
        let mut e2 = Trace::E1.schedule();
        e2.extend(crate::model::SCHED_E2_TAIL.iter().map(|&(p, s)| (p, s.to_owned())));
        let e2 = crate::model::replay(&m, m.initial(), &e2).expect("e2 replays");
        check(&m, &e2, "E2");
        // e2 has the handoff the predicate must get right: Blue mid-way through
        // her exit write (out of the room) while Green is in
        let h = e2
            .iter()
            .find(|s| {
                let (b, g) = (s.after.local[0], s.after.local[1]);
                b.pc == Pc::Cs && b.wp && g.pc == Pc::Cs && !g.wp
            })
            .expect("trace e2 hands the room over mid-exit");
        assert!(!bands_at(&h.after, &h.after, 0).room);
        assert!(bands_at(&h.after, &h.after, 1).room);
    }

    /// At most one red band at every step of `steps`, and a band wherever
    /// the model counts a customer in the room.
    fn check(m: &crate::model::Bakery, steps: &[Step], name: &str) {
        for (i, s) in steps.iter().enumerate() {
            let red: Vec<usize> = (0..m.n).filter(|&c| bands_at(&s.before, &s.after, c).room).collect();
            assert!(red.len() <= 1, "{name} step {}: red bands for {red:?}", i + 1);
            for c in m.in_room(&s.after) {
                assert!(
                    red.contains(&c),
                    "{name} step {}: customer {} in the room, no band",
                    i + 1,
                    c + 1
                );
            }
        }
    }

    /// CIE L*a*b* (D65) of an sRGB colour.
    fn lab(c: Srgb) -> [f64; 3] {
        let h = c.to_hex();
        let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map(f64::from).expect("hex") / 255.0;
        let lin = |c: f64| {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(ch(1)), lin(ch(3)), lin(ch(5)));
        let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
        let z = 0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b;
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (24389.0 / 27.0 * t + 16.0) / 116.0
            }
        };
        let (fx, fy, fz) = (f(x / 0.950_47), f(y), f(z / 1.088_83));
        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }

    /// CIEDE2000 colour difference (Sharma, Wu and Dalal's formulation).
    fn de2000(a: [f64; 3], b: [f64; 3]) -> f64 {
        let deg = f64::to_radians;
        let [l1, a1, b1] = a;
        let [l2, a2, b2] = b;
        let cb = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
        let g = 0.5 * (1.0 - (cb.powi(7) / (cb.powi(7) + 25f64.powi(7))).sqrt());
        let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
        let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
        let hue = |b: f64, a: f64| b.atan2(a).to_degrees().rem_euclid(360.0);
        let (h1p, h2p) = (hue(b1, a1p), hue(b2, a2p));
        let dlp = l2 - l1;
        let dcp = c2p - c1p;
        let mut dhp = h2p - h1p;
        if c1p * c2p == 0.0 {
            dhp = 0.0;
        } else if dhp > 180.0 {
            dhp -= 360.0;
        } else if dhp < -180.0 {
            dhp += 360.0;
        }
        let dhh = 2.0 * (c1p * c2p).sqrt() * deg(dhp / 2.0).sin();
        let lbp = (l1 + l2) / 2.0;
        let cbp = (c1p + c2p) / 2.0;
        let hbp = if c1p * c2p == 0.0 {
            h1p + h2p
        } else if (h1p - h2p).abs() <= 180.0 {
            (h1p + h2p) / 2.0
        } else if h1p + h2p < 360.0 {
            (h1p + h2p + 360.0) / 2.0
        } else {
            (h1p + h2p - 360.0) / 2.0
        };
        let t = 1.0 - 0.17 * deg(hbp - 30.0).cos() + 0.24 * deg(2.0 * hbp).cos() + 0.32 * deg(3.0 * hbp + 6.0).cos()
            - 0.20 * deg(4.0 * hbp - 63.0).cos();
        let dth = 30.0 * (-((hbp - 275.0) / 25.0).powi(2)).exp();
        let rc = 2.0 * (cbp.powi(7) / (cbp.powi(7) + 25f64.powi(7))).sqrt();
        let sl = 1.0 + 0.015 * (lbp - 50.0).powi(2) / (20.0 + (lbp - 50.0).powi(2)).sqrt();
        let sc = 1.0 + 0.045 * cbp;
        let sh = 1.0 + 0.015 * cbp * t;
        let rt = -deg(2.0 * dth).sin() * rc;
        ((dlp / sl).powi(2) + (dcp / sc).powi(2) + (dhh / sh).powi(2) + rt * (dcp / sc) * (dhh / sh)).sqrt()
    }

    #[test]
    fn de2000_matches_sharma_wu_dalal() {
        // test pairs 1, 7 and 17 of Sharma, Wu and Dalal (2005), Table 1
        let cases = [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
            ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
        ];
        for (a, b, want) in cases {
            assert!((de2000(a, b) - want).abs() < 1e-3, "{a:?} {b:?}: {}", de2000(a, b));
        }
    }

    #[test]
    fn customers_stay_apart_from_each_other_and_from_every_role() {
        let roles = [room_red(), flag_amber(), chalk(), order_gold(), hardware()];
        let mut worst = f64::INFINITY;
        for k in 1..=5 {
            for j in k + 1..=5 {
                let d = de2000(lab(customer(k)), lab(customer(j)));
                assert!(d >= 20.0, "{} and {}: ΔE2000 {d:.1}", NAMES[k - 1], NAMES[j - 1]);
                worst = worst.min(d);
            }
            for r in roles {
                let d = de2000(lab(customer(k)), lab(r));
                assert!(d >= 20.0, "{} and role {}: ΔE2000 {d:.1}", NAMES[k - 1], r.to_hex());
                worst = worst.min(d);
            }
        }
        assert!(worst >= 20.0);
    }

    #[test]
    fn the_belly_badge_fits_the_figure() {
        // clear of the eyes above
        const { assert!(BADGE_Y + BADGE_R < EYE_Y - EYE_R) };
        // every point of the disc inside the rounded body
        let half_width = |d: f64| {
            let r = FIGURE_CORNER;
            if d >= r {
                FIGURE_W / 2.0
            } else {
                FIGURE_W / 2.0 - r + (r * r - (r - d) * (r - d)).max(0.0).sqrt()
            }
        };
        for i in 0..360 {
            let t = f64::from(i).to_radians();
            let (x, y) = (BADGE_R * t.cos(), BADGE_Y + BADGE_R * t.sin());
            let d = y + FIGURE_H / 2.0;
            assert!(
                d > 0.0 && x.abs() < half_width(d),
                "badge leaves the body at {i} degrees"
            );
        }
    }

    #[test]
    fn lane_head_faces_fit_between_lanes_and_beside_the_slate() {
        for spacing in [0.6, 0.9, 1.1, 1.35, 2.0] {
            let ys = [0.0, spacing, 2.0 * spacing];
            let s = face_scale(&ys);
            // neighbouring faces never touch
            assert!(s * FIGURE_H < spacing, "faces touch at spacing {spacing}");
            // the face ends before the slate begins (slates up to 0.6 tall)
            let face_right = s * FIGURE_W / 2.0;
            let slate_left = 0.72 + FACE_SHIFT - 0.7 * 0.6;
            assert!(
                face_right + 0.1 < slate_left,
                "face meets the slate at spacing {spacing}"
            );
        }
        // 07's lanes (1.1 apart) get the largest face
        assert!((face_scale(&[1.75, 0.65, -0.45, -1.55, -2.65]) - FACE_SCALE_MAX).abs() < 1e-12);
    }

    #[test]
    fn the_lattice_maps_corners_to_corners() {
        let l = Lattice {
            center: v(1.0, -0.5),
            side: 4.0,
            steps: 2,
        };
        assert_eq!(l.p(0.0, 0.0), v(-1.0, -2.5));
        assert_eq!(l.p(2.0, 2.0), v(3.0, 1.5));
        assert_eq!(l.p(1.0, 1.0), l.center);
        assert!((fan(0, 3, 0.6) + fan(2, 3, 0.6)).abs() < 1e-12);
        assert_eq!(fan(0, 1, 0.6), 0.0);
    }
}
// ------------------------------------------------------- traces on lanes

/// White static with colour fringing around `at` (a glance that overlaps a
/// write, storyboard §3): short chalk strokes over two offset copies in
/// the fringe colours. Deterministic in `seed`.
pub fn static_burst(at: Vec3, size: f64, seed: u64) -> VMobject {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 10_000) as f64 / 10_000.0
    };
    let mut strokes = Vec::new();
    for _ in 0..9 {
        let (x, y) = ((next() - 0.5) * size, (next() - 0.5) * size * 0.8);
        let len = (0.25 + 0.5 * next()) * size * 0.45;
        let tilt = (next() - 0.5) * 0.5;
        strokes.push((
            v(at[0] + x - len / 2.0, at[1] + y - tilt * len / 2.0),
            v(at[0] + x + len / 2.0, at[1] + y + tilt * len / 2.0),
        ));
    }
    let layer = |dx: f64, color: Srgb, o: f64, w: f64| -> Vec<VMobject> {
        strokes
            .iter()
            .filter_map(|&(a, b)| {
                Line::new(v(a[0] + dx, a[1]), v(b[0] + dx, b[1]))
                    .style(faded_line(color, w, o))
                    .build()
                    .ok()
            })
            .collect()
    };
    let mut all = layer(-0.025 * size, hex("#FF5A6E"), 0.55, 2.5);
    all.extend(layer(0.025 * size, hex("#4FD6FF"), 0.55, 2.5));
    all.extend(layer(0.0, chalk(), 0.95, 2.5));
    v_group(all)
}

/// One lane on screen: whose, where, and whether it sits idle (dimmed,
/// never moving: Pink in 07, who stays home).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneSpec {
    /// 1-based.
    pub customer: usize,
    pub y: f64,
    pub idle: bool,
}

/// How steps take slots along the time axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slots {
    /// Every shown step has its own slot (step numbers match the replay's).
    EveryStep,
    /// Counted the 02_4 way: a blocked wait takes no slot and no dot (the
    /// waiting stretch shows it), so slots number the counted steps.
    Counted,
}

/// Where the lanes go.
#[derive(Clone, Debug, PartialEq)]
pub struct LaneLayout {
    /// The badge's x; the slate and the flag follow it.
    pub head_x: f64,
    /// The time axis: slot 1 starts at `x0`, the last slot ends at `x1`.
    pub x0: f64,
    pub x1: f64,
    pub lanes: Vec<LaneSpec>,
    pub slots: Slots,
    /// The slate's height at a lane's head.
    pub slate_size: f64,
}

/// A lane's head: badge, slate and flag, updated from the replayed states.
pub struct Head {
    pub badge: Mob,
    pub slate: Slate,
    pub flag: Flag,
    pub line: Mob,
}

/// What one step adds to the picture (all detached until revealed).
#[derive(Clone, Debug, Default)]
pub struct StepParts {
    /// The step's dot on the mover's lane (none for a blocked wait in
    /// `Slots::Counted`, or a hidden step).
    pub dot: Option<Mob>,
    /// A read: the arrow from the owner's lane into the reader's.
    pub arrow: Option<Mob>,
    /// What a read saw, or what a write wrote (`:=` value), in chalk.
    pub chip: Option<Mob>,
    /// The out-arrow of a ticket (or exit) write.
    pub out: Option<Mob>,
    /// White static on a read that overlaps the owner's write.
    pub static_fx: Option<Mob>,
    /// Band pieces at this step's slot: choosing (amber), the room (red),
    /// waiting (dim), a write in progress (chalk), on every lane that has one.
    pub bands: Vec<Mob>,
}

impl StepParts {
    /// Every mobject of the step, for highlights and recolouring.
    pub fn all(&self) -> Vec<Mob> {
        self.bands
            .iter()
            .copied()
            .chain(
                [self.dot, self.out, self.arrow, self.chip, self.static_fx]
                    .into_iter()
                    .flatten(),
            )
            .collect()
    }
}

/// A replayed trace (`model::Step`s) drawn on space-time lanes: time runs
/// left to right, reads are arrows in, writes arrows out; choosing is an
/// amber band, the room a red band, waiting a dim one. Built detached;
/// [`TraceLanes::reveal`] returns the animations that show a range of steps
/// (to play with the scene's own), and [`TraceLanes::heads_to`] those that
/// bring the slates and flags at the lane heads up to a step.
pub struct TraceLanes {
    pub layout: LaneLayout,
    pub steps: Vec<Step>,
    /// Each step's slot (1-based), or `None`: hidden, or a blocked wait in
    /// `Slots::Counted` (it shares the next drawn step's slot).
    pub slot: Vec<Option<usize>>,
    pub slot_count: usize,
    pub heads: Vec<Head>,
    pub parts: Vec<StepParts>,
    /// The step the heads show (0 = the start).
    shown: usize,
}

/// Who a step's label reads, and the value read: `read number[j]=v`,
/// `re-fetch number[j]=v`, `L2: choosing[j]=v`, `L3: number[j]=v`.
fn read_of(label: &str) -> Option<(usize, bool, u32)> {
    let flag = label.starts_with("L2: choosing[");
    let rest = label
        .strip_prefix("read number[")
        .or_else(|| label.strip_prefix("re-fetch number["))
        .or_else(|| label.strip_prefix("L2: choosing["))
        .or_else(|| label.strip_prefix("L3: number["))?;
    let (j, rest) = rest.split_once("]=")?;
    let v: String = rest.chars().take_while(char::is_ascii_digit).collect();
    Some((j.parse::<usize>().ok()?, flag, v.parse().ok()?))
}

/// A ticket (or exit) write that lands in this step: `number[i]:=t`,
/// `end number[i]:=t`, `number[i]:=0 (exit)`.
fn write_of(label: &str) -> Option<u32> {
    let rest = label.strip_prefix("end ").unwrap_or(label).strip_prefix("number[")?;
    let (_, value) = rest.split_once("]:=")?;
    let t: String = value.chars().take_while(char::is_ascii_digit).collect();
    t.parse().ok()
}

/// Customer `c` (0-based) is choosing in `st`: her flag is up, or being
/// raised.
fn choosing_in(st: &State, c: usize) -> bool {
    let l = st.local[c];
    st.choosing[c] == 1 || (l.pc == Pc::Ncs && l.wp)
}

/// Customer `c` is writing her slate in `st` (a safe-slate write between
/// its begin and end).
fn writing_slate(st: &State, c: usize) -> bool {
    let l = st.local[c];
    l.wp && matches!(l.pc, Pc::Wn | Pc::Cs)
}

/// Customer `c` is writing her flag in `st`.
fn writing_flag(st: &State, c: usize) -> bool {
    let l = st.local[c];
    l.wp && matches!(l.pc, Pc::Ncs | Pc::Wc0)
}

/// The bands customer `c` (0-based) shows at a step from `before` to `after`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bands {
    /// Her flag is up, or going up or down.
    pub choosing: bool,
    /// She is in the room as the model counts it (`Bakery::in_room`): in
    /// the critical section and not yet writing her exit; once her exit
    /// write begins she has left, and the next customer may go in.
    pub room: bool,
    /// She waits in line through the whole step.
    pub waiting: bool,
    /// A safe-slate write of hers is in progress (a ticket or the exit).
    pub writing: bool,
}

pub fn bands_at(before: &State, after: &State, c: usize) -> Bands {
    let inside = |st: &State| st.local[c].pc == Pc::Cs && !st.local[c].wp;
    let waiting = |st: &State| matches!(st.local[c].pc, Pc::L2 | Pc::L3);
    Bands {
        choosing: choosing_in(before, c) || choosing_in(after, c),
        room: inside(before) || inside(after),
        waiting: waiting(before) && waiting(after),
        writing: writing_slate(before, c) || writing_slate(after, c),
    }
}

/// A lane head drawn as the customer herself ([`TraceLanes::build_with_faces`]):
/// her figure as tall as this share of the closest two lanes' spacing, at
/// most [`FACE_SCALE_MAX`], with the slate, flag and lane moved right by
/// [`FACE_SHIFT`] to make room.
const FACE_SHARE: f64 = 0.8;
const FACE_SCALE_MAX: f64 = 1.2;
const FACE_SHIFT: f64 = 0.2;

/// The figure scale of a lane head drawn as a face, for lanes at heights `ys`.
fn face_scale(ys: &[f64]) -> f64 {
    let mut sorted = ys.to_vec();
    sorted.sort_by(f64::total_cmp);
    let spacing = sorted.windows(2).map(|w| w[1] - w[0]).fold(f64::INFINITY, f64::min);
    (FACE_SHARE * spacing / FIGURE_H).min(FACE_SCALE_MAX)
}

impl TraceLanes {
    /// The lanes for `steps`, with the steps in `hidden` (1-based) left out.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        layout: LaneLayout,
        steps: &[Step],
        hidden: &[usize],
    ) -> fmn::Result<Self> {
        Self::build_heads(kit, stage, layout, steps, hidden, false)
    }

    /// The same lanes with each head drawn as the customer's figure (her
    /// face and numbered badge, as in 02–05) instead of a bare badge (R3
    /// maya-02: the lanes keep the customers Maya knows).
    pub fn build_with_faces(
        kit: &Kit,
        stage: &mut Stage<'_>,
        layout: LaneLayout,
        steps: &[Step],
        hidden: &[usize],
    ) -> fmn::Result<Self> {
        Self::build_heads(kit, stage, layout, steps, hidden, true)
    }

    fn build_heads(
        kit: &Kit,
        stage: &mut Stage<'_>,
        layout: LaneLayout,
        steps: &[Step],
        hidden: &[usize],
        faces: bool,
    ) -> fmn::Result<Self> {
        let mut slot = Vec::with_capacity(steps.len());
        let mut n = 0;
        let mut pending = Vec::new();
        for (i, s) in steps.iter().enumerate() {
            let shown = !hidden.contains(&(i + 1));
            if shown && (layout.slots == Slots::EveryStep || !s.blocked()) {
                n += 1;
                for p in pending.drain(..) {
                    slot[p] = Some(n);
                }
                slot.push(Some(n));
            } else {
                if shown {
                    pending.push(i);
                }
                slot.push(None);
            }
        }
        // a trailing run of blocked waits takes one last slot
        if !pending.is_empty() {
            n += 1;
            for p in pending {
                slot[p] = Some(n);
            }
        }
        let start = steps.first().map(|s| s.before.clone());
        let mut heads = Vec::new();
        let ys: Vec<f64> = layout.lanes.iter().map(|l| l.y).collect();
        let (scale, dx) = if faces {
            (face_scale(&ys), FACE_SHIFT)
        } else {
            (0.0, 0.0)
        };
        for spec in &layout.lanes {
            let k = spec.customer;
            let dim = if spec.idle { 0.3 } else { 1.0 };
            let line = stage.arena_mut().add(
                Line::new(v(layout.head_x + 1.7 + dx, spec.y), v(layout.x1 + 0.1, spec.y))
                    .style(faded_line(customer(k), 4.0, 0.6 * dim))
                    .build()?,
            );
            let b = if faces {
                figure(kit, stage, k, v(layout.head_x, spec.y), scale)?.group
            } else {
                badge(kit, stage, k, 0.2, v(layout.head_x, spec.y))?
            };
            let value = start
                .as_ref()
                .map_or(0, |st| st.number.get(k - 1).copied().unwrap_or(0));
            let sl = slate(
                kit,
                stage,
                value,
                v(layout.head_x + 0.72 + dx, spec.y),
                layout.slate_size,
            )?;
            let raised = start.as_ref().is_some_and(|st| st.choosing.get(k - 1) == Some(&1));
            let fl = flag(stage, v(layout.head_x + 1.32 + dx, spec.y - 0.22), 0.44, raised)?;
            if spec.idle {
                for m in [b, sl.board, sl.value, fl.pole, fl.pennant] {
                    stage.set_opacity(m, 0.3, true);
                }
            }
            heads.push(Head {
                badge: b,
                slate: sl,
                flag: fl,
                line,
            });
        }
        let mut lanes = Self {
            layout,
            steps: steps.to_vec(),
            slot,
            slot_count: n,
            heads,
            parts: Vec::new(),
            shown: 0,
        };
        for i in 0..steps.len() {
            let p = lanes.build_step(kit, stage, i)?;
            lanes.parts.push(p);
        }
        Ok(lanes)
    }

    /// The width of one slot.
    pub fn slot_w(&self) -> f64 {
        (self.layout.x1 - self.layout.x0) / self.slot_count.max(1) as f64
    }

    /// The centre of slot `s` (1-based).
    pub fn slot_x(&self, s: usize) -> f64 {
        self.layout.x0 + (s as f64 - 0.5) * self.slot_w()
    }

    /// Step `step`'s time position (1-based), if it has a slot.
    pub fn x(&self, step: usize) -> Option<f64> {
        self.slot.get(step - 1).copied().flatten().map(|s| self.slot_x(s))
    }

    /// Customer `k`'s lane height (1-based), if she has a lane.
    pub fn y(&self, k: usize) -> Option<f64> {
        self.layout.lanes.iter().find(|l| l.customer == k).map(|l| l.y)
    }

    /// The step's mobjects (1-based), for highlights.
    pub fn step_mobs(&self, step: usize) -> Vec<Mob> {
        self.parts[step - 1].all()
    }

    /// The lanes and their heads, for a fade in.
    pub fn base(&self) -> Vec<Mob> {
        self.heads
            .iter()
            .flat_map(|h| {
                [
                    h.line,
                    h.badge,
                    h.slate.board,
                    h.slate.value,
                    h.flag.pole,
                    h.flag.pennant,
                ]
            })
            .collect()
    }

    fn build_step(&self, kit: &Kit, stage: &mut Stage<'_>, i: usize) -> fmn::Result<StepParts> {
        let mut p = StepParts::default();
        let Some(s) = self.slot[i] else {
            return Ok(p);
        };
        let x = self.slot_x(s);
        let w = self.slot_w();
        let step = &self.steps[i];
        // band pieces on every lane, from the states around the step
        for spec in self.layout.lanes.iter().filter(|l| !l.idle) {
            let c = spec.customer - 1;
            let (b, a) = (&step.before, &step.after);
            let mut piece = |color: Srgb, o: f64, h: f64| -> fmn::Result<()> {
                let r = Rectangle::new()
                    .width(w * 1.02)
                    .height(h)
                    .style(flat(color, o))
                    .build()?
                    .moved_to(v(x, spec.y));
                p.bands.push(stage.arena_mut().add(r));
                Ok(())
            };
            let on = bands_at(b, a, c);
            if on.choosing {
                piece(flag_amber(), 0.1, 0.34)?;
            }
            if on.room {
                piece(room_red(), 0.24, 0.34)?;
            }
            if on.waiting {
                piece(customer(c + 1), 0.08, 0.22)?;
            }
            if on.writing {
                piece(chalk(), 0.16, 0.14)?;
            }
        }
        let Some(y) = self.y(step.customer) else {
            return Ok(p);
        };
        let k = step.customer;
        if step.blocked() {
            let ring = Circle::new()
                .radius(0.075)
                .arc_center(v(x, y))
                .style(line_style(customer(k), 2.5))
                .build();
            p.dot = Some(stage.arena_mut().add(ring));
        } else {
            p.dot = Some(stage.arena_mut().add(step_dot(v(x, y), k)));
        }
        if let Some((j, is_flag, value)) = read_of(&step.label) {
            if let Some(yj) = self.y(j) {
                let dir = if yj > y { 1.0 } else { -1.0 };
                let arrow = access_arrow(v(x, yj - dir * 0.12), v(x, y + dir * 0.1), k)?;
                p.arrow = Some(stage.arena_mut().add(arrow));
                // what a slate read saw, just beyond the reader's lane (on the
                // far side from the arrow, clear of neighbouring chips); a flag
                // read needs none: a pass is a dot, a wait a ring
                if !is_flag {
                    let at = v(x, y - dir * 0.32);
                    p.chip = Some(kit.text(stage, &value.to_string(), 18.0, chalk(), at)?);
                }
                let overlapped = if is_flag {
                    writing_flag(&step.before, j - 1)
                } else {
                    writing_slate(&step.before, j - 1)
                };
                if overlapped {
                    let fx = static_burst(v(x, (y + yj) / 2.0), 0.5, i as u64 + 1);
                    p.static_fx = Some(stage.arena_mut().add(fx));
                }
            }
        } else if let Some(t) = write_of(&step.label) {
            let out = access_arrow(v(x, y + 0.08), v(x, y + 0.5), k)?;
            p.out = Some(stage.arena_mut().add(out));
            p.chip = Some(kit.text(stage, &format!(":={t}"), 18.0, chalk(), v(x, y + 0.64))?);
        }
        Ok(p)
    }

    /// The animations that show steps `range` (1-based) in order over
    /// `run_time` seconds, as one prepared lagged group (`None` if the
    /// range draws nothing). Play it alone or with the scene's own.
    pub fn reveal(
        &self,
        stage: &mut Stage<'_>,
        range: std::ops::RangeInclusive<usize>,
        run_time: f64,
    ) -> fmn::Result<Option<Box<dyn Animation>>> {
        let mut per_step: Vec<Box<dyn Animation>> = Vec::new();
        for step in range {
            let p = &self.parts[step - 1];
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &m in &p.bands {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a)?);
            }
            if let Some(d) = p.dot {
                let a = fade_in(stage.arena_mut(), d, ORIGIN, 0.4)?;
                anims.push(stage.prepare(a)?);
            }
            for m in [p.arrow, p.out].into_iter().flatten() {
                let a = fmn::animation::grow_arrow(stage.arena_mut(), m)?;
                anims.push(stage.prepare(a)?);
            }
            for m in [p.chip, p.static_fx].into_iter().flatten() {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a)?);
            }
            if !anims.is_empty() {
                per_step.push(Box::new(AnimationGroup::new(stage.arena_mut(), anims)?));
            }
        }
        if per_step.is_empty() {
            return Ok(None);
        }
        // lag 1: each step starts as the one before it finishes
        let mut g = AnimationGroup::with_lag_ratio(stage.arena_mut(), per_step, 1.0)?;
        g.update_rate_info(Some(run_time), None, None);
        Ok(Some(Box::new(g)))
    }

    /// The animations that bring every head's slate and flag to the state
    /// after `step` (0 = the start): only what changed since the last call.
    pub fn heads_to(&mut self, kit: &Kit, stage: &mut Stage<'_>, step: usize) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let st = if step == 0 {
            self.steps[0].before.clone()
        } else {
            self.steps[step - 1].after.clone()
        };
        let before = if self.shown == 0 {
            self.steps[0].before.clone()
        } else {
            self.steps[self.shown - 1].after.clone()
        };
        self.shown = step;
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for (h, spec) in self.heads.iter_mut().zip(&self.layout.lanes) {
            if spec.idle {
                continue;
            }
            let c = spec.customer - 1;
            if st.number[c] != before.number[c] {
                out.extend(h.slate.rewrite(kit, stage, st.number[c])?);
            }
            let raised = st.choosing[c] == 1;
            if raised != h.flag.raised {
                let t = h.flag.set(stage, raised);
                out.push(stage.prepare(t)?);
            }
        }
        Ok(out)
    }
}

// ------------------------------------------------------------- hardware

/// A chip outline with pins, `side` units square, `label` in its middle:
/// a processor (12), a core waking with its cache off (13).
pub fn chip_outline(kit: &Kit, stage: &mut Stage<'_>, at: Vec3, side: f64, label: &str) -> fmn::Result<Mob> {
    let body = stage.arena_mut().add(
        Rectangle::new()
            .width(side)
            .height(side)
            .corner_radius(0.12 * side)
            .style(Style::default().fill(board(), 0.9).stroke(hardware(), 3.0, 1.0))
            .build()?
            .moved_to(at),
    );
    let mut parts = vec![body];
    let pins = 5;
    for i in 0..pins {
        let t = (i as f64 + 0.5) / pins as f64 - 0.5;
        let d = side * t * 0.8;
        let e = side / 2.0;
        let l = 0.14 * side;
        for (a, b) in [
            (v(at[0] + d, at[1] + e), v(at[0] + d, at[1] + e + l)),
            (v(at[0] + d, at[1] - e), v(at[0] + d, at[1] - e - l)),
            (v(at[0] - e, at[1] + d), v(at[0] - e - l, at[1] + d)),
            (v(at[0] + e, at[1] + d), v(at[0] + e + l, at[1] + d)),
        ] {
            let pin = Line::new(a, b).style(line_style(hardware(), 3.0)).build()?;
            parts.push(stage.arena_mut().add(pin));
        }
    }
    // legible on a small chip too
    parts.push(kit.text(stage, label, (30.0 * side / 2.6).max(24.0), Palette::ink(), at)?);
    group(stage, &parts)
}

/// An `Srgb` as the `[f32; 3]` that `indicate` and its kin take.
pub fn rgb(c: Srgb) -> [f32; 3] {
    let h = c.to_hex();
    let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_or(0.0, |x| f32::from(x) / 255.0);
    [ch(1), ch(3), ch(5)]
}

/// Play prepared animations together, every one of them `run_time` seconds
/// (a slate's rewrite or a flag's turn otherwise keeps its own default).
pub fn play_timed(stage: &mut Stage<'_>, anims: Vec<Box<dyn Animation>>, run_time: f64) -> fmn::Result<()> {
    if anims.is_empty() {
        return Ok(());
    }
    stage.play_prepared_with(
        anims,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}
