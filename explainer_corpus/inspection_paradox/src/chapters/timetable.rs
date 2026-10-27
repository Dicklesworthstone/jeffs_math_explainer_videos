//! The picture chapters 02 and 03 share, all from the one timetable
//! (storyboard, "Color roles"):
//! - the hour with both routes' markers, and chapter 01's sign;
//! - a bracket under each gap with its length;
//! - a tag over each bus, the gap before it;
//! - the hour's sixty passengers as dots below the line.
//!
//! The tags sit above the markers and the passengers below the line, so the
//! two counts never share a row. The gaps, the bus that ends each gap, the
//! tag on each bus and the gap each passenger stands in all come from
//! `model`, so a gap, a tag or a passenger is the same object in both
//! chapters. Builders return detached mobjects, like `draw.rs`.

use explainer_kit::{Facts, Kit, Palette, Timed, rect, v};
use fmn::prelude::*;

use super::draw::{
    HourLine, LINE_X0, LINE_X1, LINE_Y, PASSENGER_R, bus_marker, fact, fail, flat, hour_line, line_style, marker_top,
    x_of,
};
use crate::facts::short_and_long;
use crate::model::{self, HOUR, PERIOD, Q};
use crate::palette;

// ------------------------------------------------------------------ colors

/// A short gap's color, teal (storyboard, "Color roles"). `palette.rs` adds
/// a role when its first chapter draws it; until that file is open again,
/// the short gap's role lives here.
pub fn short_gap() -> Srgb {
    Palette::teal()
}

/// A gap's color by its length: gold for the example's long gap, teal for
/// its short gap, and dim for any other (chapter 03's even gaps, neither
/// short nor long).
pub fn gap_color(minutes: i64) -> Srgb {
    let (short, long) = short_and_long();
    if minutes == long {
        palette::long_gap()
    } else if minutes == short {
        short_gap()
    } else {
        Palette::dim()
    }
}

// -------------------------------------------------------------------- gaps

/// One gap of the hour: from a bus to the next, and the bus that ends it
/// (its place in `model::bus_times`). The hour's last gap ends at the next
/// hour's first bus, the bus at :00, so that bus carries its tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gap {
    pub from: i64,
    pub to: i64,
    pub ends: usize,
}

impl Gap {
    pub fn minutes(&self) -> i64 {
        self.to - self.from
    }

    /// Whether moment `t` falls in this gap: at or after its first bus,
    /// before the next.
    pub fn holds(&self, t: Q) -> bool {
        Q::int(self.from) <= t && t < Q::int(self.to)
    }
}

/// The hour's gaps in order, with the red bus `offset` minutes after the
/// blue.
pub fn gaps(offset: i64) -> Vec<Gap> {
    let times = model::bus_times(offset);
    let n = times.len();
    times
        .iter()
        .enumerate()
        .map(|(i, &from)| Gap {
            from,
            to: times.get(i + 1).copied().unwrap_or(HOUR),
            ends: (i + 1) % n,
        })
        .collect()
}

/// The gap each passenger stands in (its place in `gaps`), in passenger
/// order. A passenger in no gap is an error.
pub fn gap_of_each(gaps: &[Gap]) -> Result<Vec<usize>, String> {
    model::passengers()
        .into_iter()
        .map(|t| {
            gaps.iter()
                .position(|g| g.holds(t))
                .ok_or_else(|| format!("the passenger at {t} min stands in no gap"))
        })
        .collect()
}

/// Your moment in chapters 02 and 03: the passenger at 6½ minutes past, in
/// the middle of the hour's first long gap. You are one of the sixty
/// passengers, and you stand clear of both buses. Chapter 01's two mornings
/// sit half a minute from a bus, where your ring would cross its marker.
pub fn your_moment() -> Q {
    Q::new(13, 2)
}

/// Where moment `t` (minutes past the hour, whole or not) sits on the line:
/// [`x_of`] for a marker in motion.
pub fn x_at(t: f64) -> f64 {
    LINE_X0 + (LINE_X1 - LINE_X0) * t / HOUR as f64
}

// ------------------------------------------------------------------ layout

/// The rows under the line: the clock labels (`draw.rs`, centred 0.32
/// below it), then each gap's bracket and its length, then the passengers.
const BRACKET_Y: f64 = 0.0;
const BRACKET_TICK: f64 = 0.1;
const BRACKET_INSET: f64 = 0.04;
const BRACKET_W: f64 = 3.0;
const BRACKET_LABEL_Y: f64 = -0.22;
const BRACKET_LABEL_SIZE: f64 = 20.0;
pub const DOT_Y: f64 = -0.6;
/// A gap drawn over the line, thick, in its color.
const SPAN_W: f64 = 7.0;
/// Over each marker, its tag.
const TAG_DY: f64 = 0.25;
const TAG_SIZE: f64 = 18.0;
/// The sign's card, lower left, exactly as chapter 01 draws it
/// (`the_stop.rs`).
const SIGN_AT: Vec3 = [-4.3, -1.55, 0.0];
const SIGN_W: f64 = 3.7;
const SIGN_H: f64 = 1.4;
const SIGN_YS: [f64; 3] = [-1.1, -1.55, -1.95];
const SIGN_SIZES: [f64; 2] = [22.0, 20.0];
/// The sign's fixed words, chapter 01's word for word.
const SIGN_HEAD: &str = "To the station";
const SIGN_BUSES: &str = "buses an hour";
const SIGN_EVERY: &str = "every";
const SIGN_MINUTES: &str = "minutes on average";

// ---------------------------------------------------------------- builders

/// A bracket under the gap from minute `from` to `to`: a line with a short
/// tick up at each end, inset so that neighbouring brackets never touch.
pub fn bracket_points(from: f64, to: f64) -> Vec<Vec3> {
    let (a, b) = (x_at(from) + BRACKET_INSET, x_at(to) - BRACKET_INSET);
    vec![
        v(a, BRACKET_Y + BRACKET_TICK),
        v(a, BRACKET_Y),
        v(b, BRACKET_Y),
        v(b, BRACKET_Y + BRACKET_TICK),
    ]
}

/// A gap's bracket and its length under it.
pub struct Bracket {
    pub line: Mob,
    pub label: Mob,
}

/// The bracket of `gap`, labelled `label` (a fact's printed value). A label
/// that is not the gap's length is refused.
pub fn bracket(kit: &Kit, stage: &mut Stage<'_>, gap: &Gap, label: &str, color: Srgb) -> fmn::Result<Bracket> {
    if label != gap.minutes().to_string() {
        return Err(fail(format!(
            "the gap from :{:02} is {} min, not {label}",
            gap.from,
            gap.minutes()
        )));
    }
    let line = stage.arena_mut().add(
        polyline(bracket_points(gap.from as f64, gap.to as f64))
            .style(line_style(color, BRACKET_W))
            .build(),
    );
    let middle = x_at((gap.from + gap.to) as f64 / 2.0);
    let label = kit.text(stage, label, BRACKET_LABEL_SIZE, color, v(middle, BRACKET_LABEL_Y))?;
    Ok(Bracket { line, label })
}

/// The ends of a gap drawn over the line, from minute `from` to `to`.
pub fn span_points(from: f64, to: f64) -> Vec<Vec3> {
    vec![v(x_at(from), LINE_Y), v(x_at(to), LINE_Y)]
}

/// A gap drawn over the line in `color`: the gap you stand in (02), the gap
/// that's growing (03).
pub fn span(stage: &mut Stage<'_>, from: f64, to: f64, color: Srgb) -> Mob {
    stage
        .arena_mut()
        .add(polyline(span_points(from, to)).style(line_style(color, SPAN_W)).build())
}

/// Where the tag over a marker centred at `x` sits.
pub fn tag_at(x: f64) -> Vec3 {
    v(x, marker_top(0)[1] + TAG_DY)
}

/// A tag, the gap before a bus: small and dim over its marker at `x`.
pub fn tag(kit: &Kit, stage: &mut Stage<'_>, x: f64, value: &str) -> fmn::Result<Mob> {
    kit.text(stage, value, TAG_SIZE, Palette::dim(), tag_at(x))
}

/// The passenger at moment `t`: a small dot under the line, in `color`.
pub fn passenger(stage: &mut Stage<'_>, t: Q, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Dot::new()
            .point(v(x_of(t), DOT_Y))
            .radius(PASSENGER_R)
            .style(flat(color, 1.0))
            .build(),
    )
}

/// The sign of chapter 01: its card, its head and its two promises.
pub struct Sign {
    pub card: Mob,
    pub head: Mob,
    pub buses: Mob,
    pub every: Mob,
}

impl Sign {
    pub fn mobs(&self) -> Vec<Mob> {
        vec![self.card, self.head, self.buses, self.every]
    }
}

pub fn sign(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Sign> {
    let card = rect(
        v(SIGN_AT[0] - SIGN_W / 2.0, SIGN_AT[1] - SIGN_H / 2.0),
        SIGN_W,
        SIGN_H,
        line_style(Palette::dim(), 2.0),
    )?;
    let card = stage.arena_mut().add(card);
    let head = kit.text(
        stage,
        SIGN_HEAD,
        SIGN_SIZES[0],
        Palette::ink(),
        v(SIGN_AT[0], SIGN_YS[0]),
    )?;
    let buses = kit.text(
        stage,
        &format!("{} {SIGN_BUSES}", fact(f, "F-T1.buses_per_hour")?),
        SIGN_SIZES[1],
        Palette::ink(),
        v(SIGN_AT[0], SIGN_YS[1]),
    )?;
    let every = kit.text(
        stage,
        &format!("{SIGN_EVERY} {} {SIGN_MINUTES}", fact(f, "F-T1.mean_gap")?),
        SIGN_SIZES[1],
        Palette::ink(),
        v(SIGN_AT[0], SIGN_YS[2]),
    )?;
    Ok(Sign {
        card,
        head,
        buses,
        every,
    })
}

/// Chapter 01's timetable: the hour line with a label every twelve minutes,
/// the blue markers, the red markers `offset` minutes after them, and the
/// sign.
pub struct Timetable {
    pub hour: HourLine,
    pub blue: Vec<Mob>,
    pub red: Vec<Mob>,
    pub sign: Sign,
}

impl Timetable {
    pub fn mobs(&self) -> Vec<Mob> {
        let mut out = self.hour.mobs();
        out.extend(&self.blue);
        out.extend(&self.red);
        out.extend(self.sign.mobs());
        out
    }
}

pub fn timetable(kit: &Kit, stage: &mut Stage<'_>, f: &Facts, offset: i64) -> fmn::Result<Timetable> {
    let marks: Vec<i64> = (0..=HOUR).step_by(PERIOD as usize).collect();
    let hour = hour_line(kit, stage, &marks)?;
    let blue: Vec<Mob> = model::route(0)
        .into_iter()
        .map(|t| bus_marker(stage, t, palette::blue_bus()))
        .collect::<fmn::Result<_>>()?;
    let red: Vec<Mob> = model::route(offset)
        .into_iter()
        .map(|t| bus_marker(stage, t, palette::red_bus()))
        .collect::<fmn::Result<_>>()?;
    let sign = sign(kit, stage, f)?;
    Ok(Timetable { hour, blue, red, sign })
}

// --------------------------------------------------------------- playback

/// Recolor a mobject, fill and stroke, its opacity kept, over `rt` seconds.
pub fn recolor(mob: Mob, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_color(color)?)
}

/// Bring a dimmed text or filled shape back to full opacity in `color`,
/// over `rt` seconds (one animation, so the two changes never fight). Not
/// for stroked lines: see [`stroke_to`].
pub fn brighten(mob: Mob, color: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_opacity(1.0)?
        .set_color(color)?)
}

/// A stroked line's opacity to `opacity` over `rt` seconds, its fill left
/// alone: at the pinned engine `set_opacity` writes every alpha, so it would
/// fill a stroke-only shape.
pub fn stroke_to(mob: Mob, opacity: f64, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_stroke(None, None, Some(opacity))?)
}

/// Builder animations one after another, the whole taking `rt`.
pub fn play_lagged(stage: &mut Stage<'_>, builders: Vec<AnimBuilder>, rt: f64, lag: f64) -> fmn::Result<()> {
    if builders.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for b in builders {
        anims.push(stage.prepare(b)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Detached mobjects fade in while builder animations (made with the same
/// run time) play, all together over `rt` seconds.
pub fn fade_in_with(stage: &mut Stage<'_>, mobs: &[Mob], builders: Vec<AnimBuilder>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for b in builders {
        anims.push(stage.prepare(b)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Mobjects fade out while builder animations (made with the same run time)
/// play, all together over `rt` seconds.
pub fn fade_out_with(stage: &mut Stage<'_>, mobs: &[Mob], builders: Vec<AnimBuilder>, rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    for b in builders {
        anims.push(stage.prepare(b)?);
    }
    if !anims.is_empty() {
        stage.play_prepared(anims)?;
    }
    Ok(())
}

/// Bring detached mobjects in one after another, each dropping `drop` onto
/// its place, the whole taking `rt`.
pub fn drop_in_lagged(stage: &mut Stage<'_>, mobs: &[Mob], drop: f64, rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, v(0.0, -drop), 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// How many of `n` things a tracker running from 0 to `n` has reached.
pub fn reached(value: Option<f64>, n: usize) -> usize {
    (value.unwrap_or(0.0).max(0.0).floor() as usize).min(n)
}

/// An updater that gives the first ⌊tracker⌋ of `mobs` their `colors`, in
/// order, as the tracker runs up (02_5: each passenger takes its gap's
/// color as it lands). Returns its anchor, already in the scene; clear the
/// anchor's updaters when the run is over.
pub fn paint_in_order(stage: &mut Stage<'_>, mobs: &[Mob], colors: Vec<Srgb>, tracker: Mob) -> fmn::Result<Mob> {
    if colors.len() != mobs.len() {
        return Err(fail(format!("{} colors for {} mobjects", colors.len(), mobs.len())));
    }
    let anchor = stage.arena_mut().add(VMobject::new());
    let mobs = mobs.to_vec();
    let mut painted = 0;
    stage.add_updater(
        anchor,
        move |s, _| {
            let n = reached(s.tracker_value(tracker), mobs.len());
            while painted < n {
                s.set_color(mobs[painted], colors[painted], None, true);
                painted += 1;
            }
        },
        true,
    )?;
    stage.add_to_scene(anchor)?;
    Ok(anchor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EVEN_OFFSET, OFFSET};

    /// At every offset the chapters draw (our two minutes, and chapter 03's
    /// six down to one), the gaps tile the hour in order, and the tag on the
    /// bus that ends each gap is that gap's length: one tag per gap, from
    /// the bus that ends it (02_7).
    #[test]
    fn every_gap_is_tagged_on_the_bus_that_ends_it() {
        for offset in (1..=EVEN_OFFSET).chain([OFFSET]) {
            let gs = gaps(offset);
            let tags = model::gap_before_each_bus(offset);
            assert_eq!(gs.len(), tags.len());
            assert_eq!(gs[0].from, 0);
            assert_eq!(gs.last().unwrap().to, HOUR);
            for w in gs.windows(2) {
                assert_eq!(w[0].to, w[1].from);
            }
            let lengths: Vec<i64> = gs.iter().map(Gap::minutes).collect();
            assert_eq!(lengths, model::hour_gaps(offset));
            for g in &gs {
                assert_eq!(tags[g.ends], g.minutes(), "offset {offset}, gap from :{:02}", g.from);
            }
            let mut ends: Vec<usize> = gs.iter().map(|g| g.ends).collect();
            ends.sort_unstable();
            assert_eq!(
                ends,
                (0..tags.len()).collect::<Vec<_>>(),
                "every bus ends exactly one gap"
            );
        }
    }

    /// Every passenger stands in exactly one gap, and the long gaps hold
    /// five sixths of them, the time they fill.
    #[test]
    fn every_passenger_stands_in_one_gap() {
        let gs = gaps(OFFSET);
        let people = model::passengers();
        for &t in &people {
            assert_eq!(gs.iter().filter(|g| g.holds(t)).count(), 1, "{t}");
        }
        let (_, long) = short_and_long();
        let of = gap_of_each(&gs).unwrap();
        let in_long = of.iter().filter(|&&g| gs[g].minutes() == long).count() as i64;
        assert_eq!(
            Q::new(in_long, people.len() as i64),
            model::time_share(&model::hour_gaps(OFFSET), |g| g == long)
        );
    }

    /// You are one of the passengers, in a long gap, and at least two
    /// minutes from every bus, so your ring (radius 0.16) clears every
    /// marker (half width 0.07): two minutes are 0.4 on the line.
    #[test]
    fn you_stand_in_a_long_gap_clear_of_every_bus() {
        let at = your_moment();
        assert!(model::passengers().contains(&at));
        let (_, long) = short_and_long();
        assert_eq!(model::gap_at(OFFSET, at), long);
        for t in model::bus_times(OFFSET).into_iter().chain([HOUR]) {
            let apart = (at - Q::int(t)).to_f64().abs();
            assert!(apart >= 2.0, "{apart} min from the bus at :{t:02}");
        }
        assert!(x_at(2.0) - x_at(0.0) >= 0.16 + 0.07);
    }

    /// The line for a moving marker agrees with the line for whole and half
    /// minutes.
    #[test]
    fn the_moving_line_is_the_line() {
        for half in 0..=2 * HOUR {
            let t = Q::new(half, 2);
            assert!((x_at(t.to_f64()) - x_of(t)).abs() < 1e-12, "{t}");
        }
    }

    /// Each bracket stays inside its own gap, so the brackets of the short
    /// gap and of its long neighbours never touch, and its end ticks stay
    /// clear of the clock labels above (centred 0.32 under the line, 18 pt,
    /// about 0.06 tall each side of centre).
    #[test]
    fn brackets_keep_to_their_gaps() {
        for g in gaps(OFFSET) {
            let p = bracket_points(g.from as f64, g.to as f64);
            assert!(p[0][0] > x_at(g.from as f64) && p[3][0] < x_at(g.to as f64));
            assert!(p[3][0] - p[0][0] > 0.0);
        }
        const {
            assert!(BRACKET_Y + BRACKET_TICK < LINE_Y - 0.32 - 0.06);
            assert!(BRACKET_LABEL_Y > DOT_Y + PASSENGER_R);
        }
    }

    /// The sign's fixed words carry no digits: its numbers are facts.
    #[test]
    fn the_signs_fixed_words_carry_no_numbers() {
        for w in [SIGN_HEAD, SIGN_BUSES, SIGN_EVERY, SIGN_MINUTES] {
            assert!(!w.chars().any(|c| c.is_ascii_digit()), "{w}");
        }
    }

    #[test]
    fn a_tracker_reaches_whole_items_only() {
        assert_eq!(reached(None, 60), 0);
        assert_eq!(reached(Some(-1.0), 60), 0);
        assert_eq!(reached(Some(2.99), 60), 2);
        assert_eq!(reached(Some(60.0), 60), 60);
        assert_eq!(reached(Some(75.0), 60), 60);
    }
}
