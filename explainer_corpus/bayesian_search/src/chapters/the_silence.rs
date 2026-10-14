//! Chapter 12, the silence: Air France 447 told as its paper tells it. The
//! flight, the date and the number aboard, set in ink on a cleared stage
//! (12_1); the circle the wreck had to lie in, and three underwater searches
//! that found nothing (12_2); a map built from three weighted maps, the
//! searches put out their share (12_3); one search that needed a decision,
//! the silence of the beacons (12_4, a question to carry); the map drawn
//! twice, once counting the silence and once leaving it out (12_5); and the
//! wreck at the same place on both, in water the listening ships had
//! already crossed (12_6).
//!
//! What the source supports and no more (fact sheet, finding 6): the
//! investigators planned the 2011 search from the first map, which already
//! allowed for dead beacons; both maps are dark near the centre; the paper
//! marks the wreck on the second only. So the ring is drawn at one place on
//! both discs, and nothing here says the first map hid the wreck. 75,000
//! particles cannot be drawn: 3,000 are, and the screen says so. The shapes
//! of the three maps and the areas searched are schematic and tagged so.

use std::collections::HashSet;
use std::f64::consts::TAU;

use explainer_kit::animatic::{HOLD_S, wrap};
use explainer_kit::props::lower_third;
use explainer_kit::stats::{Data, permutation};
use explainer_kit::{Facts, Kit, Palette, Timed, thousands, tween, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{
    CARRY_HOLD_S, QUIZ_LINES_Y, Spoken, fade_in_all, fade_in_lagged, fade_out_all, faded_line, fail, flat, group,
    last_known, line_style, map, out, pause, say, search, tag, true_place,
};
use super::sound::SEARCH_MARK;
use crate::facts::facts;
use crate::model::Q;

pub struct TheSilence {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 6] = ["12_1", "12_2", "12_3", "12_4", "12_5", "12_6"];

// the chapter's fixed words (numbers are built from facts)
const RIO: &str = "Rio de Janeiro";
const PARIS: &str = "Paris";
const PEOPLE: &str = "people";
const NAUTICAL_MILES: &str = "nautical miles";
const KM: &str = "km";
const MONTHS: &str = "months";
const UNDERWATER_SEARCHES: &str = "underwater searches";
const SCHEMATIC_AREAS: &str = "areas searched: schematic";
const METHODS: [&str; 3] = [
    "anywhere in the circle",
    "near the last position",
    "traced back from what was found afloat",
];
/// What a nautical mile is, under the radius (R2: M21), and the names of
/// 12_3's three stories and its count of empty searches (R2: M6, R13).
const NAUTICAL_IS_SEA: &str = "a nautical mile is a sea mile";
const STORY: &str = "story";
const EMPTY_SEARCHES: &str = "empty searches:";
const ON_THE_SURFACE: &str = "on the surface,";
const UNDER_WATER: &str = "under water";
const BY_JUDGMENT: &str = "weights chosen by judgment; shapes schematic";
const PARTICLES: &str = "particles; each dot here stands for";
const HEARD_NOTHING: &str = "heard nothing";
const SILENCE_COUNTS: &str = "the silence counts:";
const WOULD_HAVE_BEEN_HEARD: &str = "it would have been heard";
const SILENCE_LEFT_OUT: &str = "the silence left out";
const AUTHORS_WAY: &str = "the authors’ better way:";
const AUTHORS_TEXT: &str = "“a joint distribution on beacon failure and wreck location”";
const FROM_LAST: &str = "from the last position";
const ABOUT: &str = "about";
const FIRST_MAP: &str = "The investigators planned from the first map";

// ------------------------------------------------------------- the data

/// How many dots are drawn of the 75,000 particles (the paper's figure is
/// F-A.points; the screen says so and says what a dot stands for).
const DOTS_SHOWN: usize = 3_000;
/// Of the dots inside a searched area, the share a search puts out: the
/// paper's cap of 0.9 on any sensor's chance of seeing (S3 p. 76). A drawing
/// choice, not printed.
const SEARCHED_SHARE: Q = Q::lowest(9, 10);
/// The three maps' seeds, the three ways of putting a dot on the unit disc.
const SEEDS: [u64; 3] = [2009, 2010, 2011];
/// The spread of the second map (near the last position) and the third's
/// long and short axes and where its middle lies, in units of the circle's
/// radius: schematic.
const NEAR_SPREAD: f64 = 0.3;
const SMEAR: (f64, f64) = (0.38, 0.13);
const SMEAR_AT: (f64, f64) = (-0.15, 0.2);
const SMEAR_TURN: f64 = 2.2;
/// No dot is drawn nearer the circle than this.
const EDGE: f64 = 0.97;
/// The listening tracks: their heading, their spacing and the half width of
/// the band each one hears across (units of the circle's radius).
const TRACK_TURN: f64 = 0.21;
const TRACK_SPACING: f64 = 0.3;
const TRACK_HALF: f64 = 0.06;
/// Where the wreck lay from the last known position, as a bearing east of
/// north (north-north-east).
const BEARING_EAST_OF_NORTH: f64 = 22.5;
/// The seeds of the two choices of which dots go out.
const AREA_SEED: u64 = 1_800;
const BEACON_SEED: u64 = 1_801;

/// An area searched, drawn as a tilted ellipse (units of the circle's
/// radius): schematic, placed clear of where the wreck lay.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    at: (f64, f64),
    semi: (f64, f64),
    turn: f64,
}

/// The three areas of 2009, 2009 and 2010, in the order they were searched.
const AREAS: [Area; 3] = [
    Area {
        at: (-0.4, 0.18),
        semi: (0.3, 0.2),
        turn: 0.3,
    },
    Area {
        at: (0.38, -0.3),
        semi: (0.3, 0.22),
        turn: -0.4,
    },
    Area {
        at: (0.3, 0.42),
        semi: (0.2, 0.24),
        turn: 0.9,
    },
];

impl Area {
    /// Whether `p` is inside the ellipse grown by `grow` (1 for itself).
    fn contains(&self, p: (f64, f64), grow: f64) -> bool {
        let (dx, dy) = (p.0 - self.at.0, p.1 - self.at.1);
        let (c, s) = (self.turn.cos(), self.turn.sin());
        let (x, y) = (dx * c + dy * s, -dx * s + dy * c);
        (x / (self.semi.0 * grow)).powi(2) + (y / (self.semi.1 * grow)).powi(2) <= 1.0
    }

    /// The outline, `n` points round.
    fn outline(&self, n: usize) -> Vec<(f64, f64)> {
        let (c, s) = (self.turn.cos(), self.turn.sin());
        (0..n)
            .map(|k| {
                let t = TAU * k as f64 / n as f64;
                let (x, y) = (self.semi.0 * t.cos(), self.semi.1 * t.sin());
                (self.at.0 + x * c - y * s, self.at.1 + x * s + y * c)
            })
            .collect()
    }
}

/// One listening track: where it enters and leaves the circle, and how far
/// from the centre it runs (signed, along its normal).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Track {
    from: (f64, f64),
    to: (f64, f64),
    offset: f64,
}

impl Track {
    /// Whether a dot at `p` is under the band this track hears across.
    fn hears(&self, p: (f64, f64)) -> bool {
        let n = (-TRACK_TURN.sin(), TRACK_TURN.cos());
        (p.0 * n.0 + p.1 * n.1 - self.offset).abs() <= TRACK_HALF
    }
}

/// The listening tracks: five parallel ones, one of which runs through
/// `ring` (the ships had crossed the water the wreck lay in).
fn tracks(ring: (f64, f64)) -> Vec<Track> {
    let n = (-TRACK_TURN.sin(), TRACK_TURN.cos());
    let t = (TRACK_TURN.cos(), TRACK_TURN.sin());
    let through = ring.0 * n.0 + ring.1 * n.1;
    (-2..=2)
        .map(|j| {
            let d = through + TRACK_SPACING * f64::from(j);
            let half = (1.0 - d * d).max(0.0).sqrt();
            Track {
                from: (n.0 * d - t.0 * half, n.1 * d - t.1 * half),
                to: (n.0 * d + t.0 * half, n.1 * d + t.1 * half),
                offset: d,
            }
        })
        .collect()
}

/// Where the wreck lay: `found_nm` from the last position of a circle of
/// `circle_nm`, north-north-east, in units of the circle's radius.
fn ring_at(found_nm: f64, circle_nm: f64) -> (f64, f64) {
    let from_east = (90.0 - BEARING_EAST_OF_NORTH).to_radians();
    let r = found_nm / circle_nm;
    (r * from_east.cos(), r * from_east.sin())
}

/// `n` dots of one of the three maps, inside the circle: anywhere in it
/// (0), near the last position (1), traced back from what was found afloat
/// (2: a long smear).
fn component(kind: usize, n: usize) -> Vec<(f64, f64)> {
    let mut data = Data::new(SEEDS[kind]);
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let p = match kind {
            0 => {
                let (r, t) = (data.unit().sqrt(), TAU * data.unit());
                (r * t.cos(), r * t.sin())
            }
            1 => (NEAR_SPREAD * data.normal(), NEAR_SPREAD * data.normal()),
            _ => {
                let (a, b) = (SMEAR.0 * data.normal(), SMEAR.1 * data.normal());
                let (c, s) = (SMEAR_TURN.cos(), SMEAR_TURN.sin());
                (SMEAR_AT.0 + a * c - b * s, SMEAR_AT.1 + a * s + b * c)
            }
        };
        if p.0.hypot(p.1) <= EDGE {
            out.push(p);
        }
    }
    out
}

/// How many of `n` candidates a search that sees `share` of them puts out,
/// in whole dots, half up.
fn dots_out(n: usize, share: Q) -> usize {
    (share * Q::int(n as i128))
        .fixed(0)
        .parse()
        .expect("a share of a whole number of dots is a number")
}

/// The dots among `candidates` that go out when `share` of them do: the
/// first `dots_out` in a seeded order (never more than once each).
fn choose_out(candidates: &[usize], share: Q, seed: u64) -> Vec<usize> {
    let order = permutation(candidates.len(), &mut Data::new(seed));
    order
        .into_iter()
        .take(dots_out(candidates.len(), share))
        .map(|i| candidates[i])
        .collect()
}

/// One disc's cloud as data: where each dot is, whether the three searched
/// areas put it out, and whether the listening tracks do.
struct Dots {
    at: Vec<(f64, f64)>,
    by_area: Vec<bool>,
    by_beacon: Vec<bool>,
}

impl Dots {
    /// Dots at `at`; the areas put out `SEARCHED_SHARE` of those inside any
    /// of them; when `listening` is `Some(share)`, the tracks put out that
    /// share of the dots still lit under any track.
    fn new(at: Vec<(f64, f64)>, tracks: &[Track], listening: Option<Q>) -> Self {
        let n = at.len();
        let inside: Vec<usize> = (0..n)
            .filter(|&i| AREAS.iter().any(|a| a.contains(at[i], 1.0)))
            .collect();
        let mut by_area = vec![false; n];
        for i in choose_out(&inside, SEARCHED_SHARE, AREA_SEED) {
            by_area[i] = true;
        }
        let mut by_beacon = vec![false; n];
        if let Some(share) = listening {
            let heard: Vec<usize> = (0..n)
                .filter(|&i| !by_area[i] && tracks.iter().any(|t| t.hears(at[i])))
                .collect();
            for i in choose_out(&heard, share, BEACON_SEED) {
                by_beacon[i] = true;
            }
        }
        Dots { at, by_area, by_beacon }
    }
}

/// Everything the chapter draws from, computed once from the facts.
struct Plan {
    weights: [u32; 3],
    /// All the dots of the cloud, the first map's block, then the second's,
    /// then the third's.
    all: Vec<(f64, f64)>,
    /// Which map each dot belongs to.
    kind: Vec<usize>,
    ring: (f64, f64),
    tracks: Vec<Track>,
    /// The share of the dots under a listening track that the silence puts
    /// out: F-A.beacon.used.percent in hundredths.
    listening: Q,
}

fn number(f: &Facts, key: &str) -> Result<f64, String> {
    f.get(key)
        .ok_or_else(|| format!("no fact {key}"))?
        .parse()
        .map_err(|_| format!("fact {key} is not a number"))
}

fn plan(f: &Facts) -> Result<Plan, String> {
    let list = f.get("F-A.weights").ok_or("no fact F-A.weights")?;
    let weights: Vec<u32> = list
        .split(',')
        .map(|w| w.trim().parse().map_err(|_| format!("weight {w}")))
        .collect::<Result<_, _>>()?;
    let weights: [u32; 3] = weights
        .try_into()
        .map_err(|_| "F-A.weights is not three weights".to_string())?;
    let total: u32 = weights.iter().sum();
    let mut all = Vec::with_capacity(DOTS_SHOWN);
    let mut kind = Vec::with_capacity(DOTS_SHOWN);
    for (k, &w) in weights.iter().enumerate() {
        let n = DOTS_SHOWN * w as usize / total as usize;
        all.extend(component(k, n));
        kind.extend(std::iter::repeat_n(k, n));
    }
    let (circle_nm, found_nm) = (number(f, "F-A.circle.nm")?, number(f, "F-A.found.nm")?);
    let ring = ring_at(found_nm, circle_nm);
    let percent = number(f, "F-A.beacon.used.percent")?;
    Ok(Plan {
        weights,
        all,
        kind,
        ring,
        tracks: tracks(ring),
        listening: Q::new(percent as i128, 100),
    })
}

impl Plan {
    /// The cloud of 12_3 and 12_4: every dot, the searches' share out.
    fn whole(&self) -> Dots {
        Dots::new(self.all.clone(), &self.tracks, None)
    }

    /// One of the two discs of 12_5: every second dot (so the same places
    /// on both discs), the searches' share out; the left one also with the
    /// silence counted.
    fn disc(&self, counting_the_silence: bool) -> Dots {
        let half: Vec<(f64, f64)> = self.all.iter().copied().step_by(2).collect();
        Dots::new(half, &self.tracks, counting_the_silence.then_some(self.listening))
    }

    /// A tenth of one map's dots, for its small disc in 12_3.
    fn sketch(&self, k: usize) -> Vec<(f64, f64)> {
        let mine: Vec<(f64, f64)> = self
            .all
            .iter()
            .zip(&self.kind)
            .filter(|&(_, &m)| m == k)
            .map(|(&p, _)| p)
            .collect();
        mine.iter().copied().take(mine.len() / SKETCH_FRACTION).collect()
    }
}

/// A small disc shows one in this many of its map's dots.
const SKETCH_FRACTION: usize = 10;

// --------------------------------------------------------- words and dates

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// An ISO date (`2009-06-01`) as its year, month number and day.
fn parts(iso: &str) -> Result<(u32, usize, u32), String> {
    let mut it = iso.split('-').map(|p| p.parse::<u32>());
    match (it.next(), it.next(), it.next()) {
        (Some(Ok(y)), Some(Ok(m)), Some(Ok(d))) if (1..=12).contains(&m) => Ok((y, m as usize, d)),
        _ => Err(format!("{iso} is not a date")),
    }
}

/// "1 June 2009".
fn day_month_year(iso: &str) -> Result<String, String> {
    let (y, m, d) = parts(iso)?;
    Ok(format!("{d} {} {y}", MONTH_NAMES[m - 1]))
}

/// "April 2011".
fn month_year(iso: &str) -> Result<String, String> {
    let (y, m, _) = parts(iso)?;
    Ok(format!("{} {y}", MONTH_NAMES[m - 1]))
}

/// The radius label's line width in characters ("40 nautical miles" and
/// "(74 km)"), its size, and its distance right of the circle's rim.
const RADIUS_LINE_CHARS: usize = 18;
const RADIUS_PT: f64 = 20.0;
const RADIUS_GAP: f64 = 0.15;

/// The radius label's lines stand this far apart, the first this far over
/// the spoke; the gloss "a nautical mile is a sea mile" follows in two lines
/// of at most [`GLOSS_CHARS`] characters at [`GLOSS_PT`], from [`GLOSS_TOP`]
/// (relative to the circle's centre) in steps of [`GLOSS_STEP`].
const RADIUS_STEP: f64 = 0.35;
const RADIUS_RISE: f64 = 0.15;
const GLOSS_CHARS: usize = 15;
const GLOSS_PT: f64 = 18.0;
const GLOSS_TOP: f64 = -0.7;
const GLOSS_STEP: f64 = 0.3;

/// 12_3's three small discs: their heights, how far over a disc's middle the
/// story's name stands, the first line of its method and the step between
/// lines, and how far under the middle its weight stands (44 pt).
const SMALL_DISC_Y: [f64; 3] = [2.55, 0.5, -1.55];
const STORY_RISE: f64 = 0.72;
const METHOD_TOP: f64 = 0.42;
const METHOD_STEP: f64 = 0.3;
const WEIGHT_DROP: f64 = 0.35;
/// The count of empty searches in the left column once the small discs have
/// gone: the number's line and the breakdown's line.
const EMPTY_Y: [f64; 2] = [1.2, 0.55];

/// "40 nautical miles (74 km)".
fn radius_words(nm: &str, km: &str) -> String {
    format!("{nm} {NAUTICAL_MILES} ({km} {KM})")
}

/// The paper's empty searches as the screen counts them: "4 empty
/// searches:" and "1 on the surface, 3 under water" (the three under-water
/// searches are the facts', the fourth is the surface one).
fn empty_searches_words(empty: u32, under_water: u32) -> Result<[String; 2], String> {
    let surface = empty
        .checked_sub(under_water)
        .ok_or_else(|| format!("{under_water} under-water searches of {empty}"))?;
    Ok([
        format!("{empty} {EMPTY_SEARCHES}"),
        format!("{surface} {ON_THE_SURFACE} {under_water} {UNDER_WATER}"),
    ])
}

/// "75,000 particles; each dot here stands for 25".
fn particles_words(points: u64, shown: usize) -> Result<String, String> {
    if !points.is_multiple_of(shown as u64) {
        return Err(format!("{points} particles do not divide into {shown} dots"));
    }
    Ok(format!("{} {PARTICLES} {}", thousands(points), points / shown as u64))
}

// ----------------------------------------------------------------- drawing

/// The circle of 12_2 to 12_4, and the unit disc mapped onto it.
const CIRCLE_AT: Vec3 = [2.5, 0.15, 0.0];
const CIRCLE_R: f64 = 2.6;
/// The two discs of 12_5.
const DISC_AT: [Vec3; 2] = [[-3.5, 0.6, 0.0], [3.5, 0.6, 0.0]];
const DISC_R: f64 = 2.0;
/// A dot's radius against its disc's (no smaller than `MIN_DOT_R`, in scene
/// units), and how opaque it is when lit.
const DOT_FRACTION: f64 = 0.014;
const MIN_DOT_R: f64 = 0.012;
const DOT_OPACITY: f64 = 0.9;
/// The left column of 12_2 to 12_4.
const LEFT_X: f64 = -6.7;

/// A place on a disc of `radius` at `centre`.
fn on(centre: Vec3, radius: f64, p: (f64, f64)) -> Vec3 {
    v(centre[0] + radius * p.0, centre[1] + radius * p.1)
}

/// A cloud of dots on a disc. Each dot is one mobject whose colour and
/// opacity follow three trackers (how much of the cloud is on the stage,
/// how far the searched areas have put theirs out, how far the listening
/// tracks have), so nothing is animated as a group of thousands.
struct Cloud {
    group: Mob,
    shown: Mob,
    area: Mob,
    beacon: Mob,
    anchor: Mob,
}

impl Cloud {
    fn build(stage: &mut Stage<'_>, centre: Vec3, radius: f64, dots: &Dots) -> fmn::Result<Self> {
        let r = (DOT_FRACTION * radius).max(MIN_DOT_R);
        let mut members: Vec<(Mob, bool, bool)> = Vec::with_capacity(dots.at.len());
        for (i, &p) in dots.at.iter().enumerate() {
            let d = Dot::new()
                .point(on(centre, radius, p))
                .radius(r)
                .style(flat(map(), 0.0))
                .build();
            members.push((stage.arena_mut().add(d), dots.by_area[i], dots.by_beacon[i]));
        }
        let mobs: Vec<Mob> = members.iter().map(|m| m.0).collect();
        let group = group(stage, &mobs)?;
        let (shown, area, beacon) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = (f64::NAN, f64::NAN, f64::NAN);
        stage.add_updater(
            anchor,
            move |s, _| {
                let now = (
                    s.tracker_value(shown).unwrap_or(0.0),
                    s.tracker_value(area).unwrap_or(0.0),
                    s.tracker_value(beacon).unwrap_or(0.0),
                );
                if now == last {
                    return;
                }
                last = now;
                for &(m, by_area, by_beacon) in &members {
                    let gone = f64::max(if by_area { now.1 } else { 0.0 }, if by_beacon { now.2 } else { 0.0 });
                    let colour = interpolate_color(map(), out(), gone);
                    s.set_fill(m, Some(colour), Some(DOT_OPACITY * now.0), None, false);
                }
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        stage.add_to_scene(group)?;
        Ok(Cloud {
            group,
            shown,
            area,
            beacon,
            anchor,
        })
    }

    /// Take the cloud off the stage, its updater with it.
    fn remove(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.group);
        stage.remove_from_scene(self.anchor);
    }
}

/// A closed outline through `points`, detached.
fn outline(stage: &mut Stage<'_>, points: Vec<Vec3>, style: Style) -> Mob {
    let mut closed = points;
    if let Some(&first) = closed.first() {
        closed.push(first);
    }
    stage.arena_mut().add(polyline(closed).style(style).build())
}

/// An open diamond (the last known position) at `at`, detached.
fn diamond(stage: &mut Stage<'_>, at: Vec3, r: f64) -> Mob {
    outline(
        stage,
        vec![
            v(at[0], at[1] + r),
            v(at[0] + r, at[1]),
            v(at[0], at[1] - r),
            v(at[0] - r, at[1]),
        ],
        line_style(last_known(), 3.0),
    )
}

/// The circle's outline on a disc, detached.
fn circle(stage: &mut Stage<'_>, centre: Vec3, radius: f64) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .arc_center(centre)
            .style(line_style(Palette::dim(), 3.0))
            .build(),
    )
}

/// The three searched areas on a disc, as translucent teal shapes.
fn areas(stage: &mut Stage<'_>, centre: Vec3, radius: f64) -> Vec<Mob> {
    AREAS
        .iter()
        .map(|a| {
            let pts: Vec<Vec3> = a.outline(48).into_iter().map(|p| on(centre, radius, p)).collect();
            stage
                .arena_mut()
                .add(Polygon::new(pts).style(flat(search(), 0.32)).build())
        })
        .collect()
}

/// The listening tracks on a disc, in teal.
fn track_lines(stage: &mut Stage<'_>, centre: Vec3, radius: f64, tracks: &[Track], width: f64) -> Vec<Mob> {
    tracks
        .iter()
        .map(|t| {
            stage.arena_mut().add(
                polyline(vec![on(centre, radius, t.from), on(centre, radius, t.to)])
                    .style(line_style(search(), width))
                    .build(),
            )
        })
        .collect()
}

/// The hollow ring of the true place, at `ring` on a disc, detached.
fn ring_mark(stage: &mut Stage<'_>, centre: Vec3, radius: f64, ring: (f64, f64)) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(0.11)
            .arc_center(on(centre, radius, ring))
            .style(line_style(true_place(), 3.5))
            .build(),
    )
}

/// The last sentence of `line` as a question to carry: set in ink in the
/// caption band like the quiz's question, no ring. Detached.
fn carry_last(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<Mob> {
    let words = line
        .sentence_texts()
        .pop()
        .ok_or_else(|| fail(format!("{} has no sentences", line.id)))?;
    kit.text(stage, &words, 26.0, Palette::ink(), v(0.0, QUIZ_LINES_Y[0]))
}

/// The flight of 12_1: Rio at the lower left, Paris at the upper right,
/// and the points of the way there, as a quadratic curve across the stage.
fn flight(n: usize) -> Vec<Vec3> {
    let (a, c, b) = (v(-5.6, -1.7), v(-0.8, 2.7), v(5.5, 1.5));
    (0..=n)
        .map(|k| {
            let t = k as f64 / n as f64;
            let u = 1.0 - t;
            v(
                u * u * a[0] + 2.0 * u * t * c[0] + t * t * b[0],
                u * u * a[1] + 2.0 * u * t * c[1] + t * t * b[1],
            )
        })
        .collect()
}

/// How far along the flight the last reported position is.
const LAST_REPORT_AT: f64 = 0.62;

impl SceneConstruct for TheSilence {
    fn name(&self) -> &str {
        "12_the_silence"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let all = facts().map_err(fail)?;
        let fact = |key: &str| -> fmn::Result<String> {
            all.get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let plan = plan(&all).map_err(fail)?;

        // ------------------------------------------------------- 12_1
        // a flight and its last report, then a cleared stage: the date and
        // the number aboard, in ink, held on the last sentence
        let line = say(&kit, stage, LINES[0])?;
        let curve = flight(40);
        let last = (curve.len() as f64 * LAST_REPORT_AT) as usize;
        let way = stage.arena_mut().add(
            polyline(curve[..=last].to_vec())
                .style(line_style(Palette::dim(), 3.0))
                .build(),
        );
        let rest = stage.arena_mut().add(
            polyline(curve[last..].to_vec())
                .style(faded_line(Palette::dim(), 2.0, 0.35))
                .build(),
        );
        let from = kit.text(
            stage,
            RIO,
            24.0,
            Palette::dim(),
            v(curve[0][0] + 0.2, curve[0][1] - 0.4),
        )?;
        let to = kit.text(
            stage,
            PARIS,
            24.0,
            Palette::dim(),
            v(curve[curve.len() - 1][0] - 0.1, curve[curve.len() - 1][1] + 0.4),
        )?;
        let report = diamond(stage, curve[last], 0.16);
        fade_in_all(stage, &[rest, from, to], 0.5)?;
        line.wait_for(stage, "Air France flight")?;
        stage.play(show_creation(way).rt(3.2))?;
        fade_in_all(stage, &[report], 0.4)?;
        line.wait_for(stage, "All two hundred")?;
        fade_out_all(stage, &[way, rest, from, to, report], 0.6)?;
        let date = kit.text(
            stage,
            &day_month_year(&fact("F-A.lost")?).map_err(fail)?,
            60.0,
            Palette::ink(),
            v(0.0, 0.55),
        )?;
        let aboard = kit.text(
            stage,
            &format!("{} {PEOPLE}", fact("F-A.aboard")?),
            60.0,
            Palette::ink(),
            v(0.0, -0.45),
        )?;
        fade_in_all(stage, &[date, aboard], 0.6)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 12_2
        // the circle the wreck had to lie in, and three searches that
        // found nothing
        let line = say(&kit, stage, LINES[1])?;
        fade_out_all(stage, &[date, aboard], 0.5)?;
        let rim = circle(stage, CIRCLE_AT, CIRCLE_R);
        let centre_mark = diamond(stage, CIRCLE_AT, 0.16);
        let spoke = stage.arena_mut().add(
            polyline(vec![CIRCLE_AT, v(CIRCLE_AT[0] + CIRCLE_R, CIRCLE_AT[1])])
                .style(line_style(Palette::dim(), 2.0))
                .build(),
        );
        // two lines at the spoke's end, outside the rim: on one line over the
        // spoke the words were wider than it and touched the rim, and over
        // the spoke two lines stood on the 2010 area
        let radius_lines = wrap(
            &radius_words(&fact("F-A.circle.nm")?, &fact("F-A.circle.km")?),
            RADIUS_LINE_CHARS,
            2,
        );
        let mut radius_texts: Vec<Mob> = Vec::new();
        for (j, text) in radius_lines.iter().enumerate() {
            radius_texts.push(kit.text_left(
                stage,
                text,
                RADIUS_PT,
                Palette::ink(),
                v(
                    CIRCLE_AT[0] + CIRCLE_R + RADIUS_GAP,
                    CIRCLE_AT[1] + RADIUS_RISE - RADIUS_STEP * j as f64,
                ),
            )?);
        }
        // and what a nautical mile is, in the same column, under the radius
        for (j, text) in wrap(NAUTICAL_IS_SEA, GLOSS_CHARS, 2).iter().enumerate() {
            radius_texts.push(kit.text_left(
                stage,
                text,
                GLOSS_PT,
                Palette::dim(),
                v(
                    CIRCLE_AT[0] + CIRCLE_R + RADIUS_GAP,
                    CIRCLE_AT[1] + GLOSS_TOP - GLOSS_STEP * j as f64,
                ),
            )?);
        }
        let radius_label = group(stage, &radius_texts)?;
        line.wait_for(stage, "The wreck had to lie")?;
        stage.play(show_creation(rim).rt(1.4))?;
        fade_in_all(stage, &[centre_mark, spoke, radius_label], 0.5)?;
        line.wait_for(stage, "In nearly two years")?;
        let shaded = areas(stage, CIRCLE_AT, CIRCLE_R);
        let years_fact = fact("F-A.underwater_searches.years")?;
        let mut year_labels: Vec<Mob> = Vec::new();
        for (a, y) in AREAS.iter().zip(years_fact.split(',')) {
            year_labels.push(kit.text(stage, y.trim(), 24.0, Palette::ink(), on(CIRCLE_AT, CIRCLE_R, a.at))?);
        }
        fade_in_lagged(stage, &shaded, 1.6, 0.6)?;
        fade_in_all(stage, &year_labels, 0.4)?;
        let months = kit.text_left(
            stage,
            &format!("{} {MONTHS}", fact("F-A.months")?),
            40.0,
            Palette::ink(),
            v(LEFT_X, 1.2),
        )?;
        let searches = kit.text_left(
            stage,
            &format!("{} {UNDERWATER_SEARCHES}", fact("F-A.underwater_searches")?),
            26.0,
            Palette::dim(),
            v(LEFT_X, 0.55),
        )?;
        let schematic = tag(&kit, stage, SCHEMATIC_AREAS, v(LEFT_X + 1.7, -1.3), 20.0)?;
        fade_in_all(stage, &[months, searches, schematic], 0.5)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 12_3
        // three weighted maps become one cloud of 3,000 dots, and the
        // searched areas put out their share
        let line = say(&kit, stage, LINES[2])?;
        fade_out_all(stage, &[months, searches, schematic], 0.4)?;
        let whole = Cloud::build(stage, CIRCLE_AT, CIRCLE_R, &plan.whole())?;
        // the years stay readable when 3,000 dots come in over the areas
        stage.scene_mut().bring_to_front(&year_labels)?;
        let mut small_discs: Vec<Cloud> = Vec::new();
        let mut small_mobs: Vec<Mob> = Vec::new();
        let (small_x, small_r) = (-5.6, 0.75);
        for (k, name) in METHODS.iter().enumerate() {
            let at = v(small_x, SMALL_DISC_Y[k]);
            let rim = circle(stage, at, small_r);
            // a small disc is its map's sketch: none of it is put out
            let sketch = plan.sketch(k);
            let n = sketch.len();
            let dots = Dots {
                at: sketch,
                by_area: vec![false; n],
                by_beacon: vec![false; n],
            };
            let cloud = Cloud::build(stage, at, small_r, &dots)?;
            small_mobs.push(rim);
            // the story's name, so that the line's "stories" and the "three"
            // of the weights are the same three things (R2: M6)
            small_mobs.push(kit.text_left(
                stage,
                &format!("{STORY} {}", k + 1),
                24.0,
                Palette::ink(),
                v(small_x + 1.0, at[1] + STORY_RISE),
            )?);
            // three lines at most, so no word of a label is ever cut for an
            // ellipsis ("afloat" was, at two lines of twenty)
            for (j, text) in wrap(name, 26, 3).iter().enumerate() {
                small_mobs.push(kit.text_left(
                    stage,
                    text,
                    22.0,
                    Palette::dim(),
                    v(small_x + 1.0, at[1] + METHOD_TOP - METHOD_STEP * j as f64),
                )?);
            }
            small_discs.push(cloud);
        }
        fade_in_all(stage, &small_mobs, 0.6)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for c in &small_discs {
            anims.push(stage.prepare(tween(c.shown, 1.0, 0.8)?)?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "They weighted them")?;
        let mut weight_labels: Vec<Mob> = Vec::new();
        for (k, w) in plan.weights.iter().enumerate() {
            weight_labels.push(kit.text_left(
                stage,
                &w.to_string(),
                44.0,
                Palette::ink(),
                v(small_x + 1.0, SMALL_DISC_Y[k] - WEIGHT_DROP),
            )?);
        }
        let judgment = tag(&kit, stage, BY_JUDGMENT, v(LEFT_X + 2.4, -2.65), 20.0)?;
        fade_in_all(stage, &weight_labels, 0.5)?;
        fade_in_all(stage, &[judgment], 0.4)?;
        // the three come together as one cloud, and the searches' share goes out
        line.wait_for(stage, "And they updated")?;
        let note = kit.text_left(
            stage,
            &particles_words(
                fact("F-A.points")?
                    .parse()
                    .map_err(|_| fail("F-A.points is not a number".into()))?,
                DOTS_SHOWN,
            )
            .map_err(fail)?,
            22.0,
            Palette::dim(),
            v(LEFT_X, -3.15),
        )?;
        let mut together: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(whole.shown, 1.0, 1.2)?)?];
        for c in &small_discs {
            together.push(stage.prepare(tween(c.shown, 0.0, 1.2)?)?);
        }
        for &m in small_mobs.iter().chain(&weight_labels) {
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            together.push(stage.prepare(gone.rt(1.0))?);
        }
        let more = fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?;
        together.push(stage.prepare(more.rt(0.8))?);
        stage.play_prepared(together)?;
        for c in &small_discs {
            c.remove(stage);
        }
        stage.play(tween(whole.area, 1.0, 1.0)?)?;
        // the empty searches the map was updated for: four, one of them on
        // the surface and three under water (the three areas, with their
        // years; R2: R13). "Every empty search" is all four.
        let count =
            |key: &str| -> fmn::Result<u32> { fact(key)?.parse().map_err(|_| fail(format!("{key} is not a number"))) };
        let [empty_count, empty_split] =
            empty_searches_words(count("F-A.empty_searches")?, count("F-A.underwater_searches")?).map_err(fail)?;
        let empties = [
            kit.text_left(stage, &empty_count, 40.0, Palette::ink(), v(LEFT_X, EMPTY_Y[0]))?,
            kit.text_left(stage, &empty_split, 26.0, Palette::dim(), v(LEFT_X, EMPTY_Y[1]))?,
        ];
        line.wait_for(stage, "every empty search")?;
        fade_in_all(stage, &empties, 0.5)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 12_4
        // the question: how much should the silence count?
        let line = say(&kit, stage, LINES[3])?;
        fade_out_all(stage, &[judgment, empties[0], empties[1]], 0.3)?;
        line.wait_for(stage, "For a month")?;
        let lines = track_lines(stage, CIRCLE_AT, CIRCLE_R, &plan.tracks, 3.0);
        let heard = kit.text_left(stage, HEARD_NOTHING, 40.0, Palette::ink(), v(LEFT_X, 1.0))?;
        fade_in_lagged(stage, &lines, 1.8, 0.4)?;
        fade_in_all(stage, &[heard], 0.5)?;
        line.wait_last(stage)?;
        let question = carry_last(&kit, stage, &line)?;
        fade_in_all(stage, &[question], 0.4)?;
        // two seconds with nothing sounding: no sting under this question
        kit.hold(stage)?;
        pause(stage, CARRY_HOLD_S)?;
        fade_out_all(stage, &[question], 0.3)?;

        // ------------------------------------------------------- 12_5
        // the map drawn twice, once counting the silence and once leaving
        // it out
        let line = say(&kit, stage, LINES[4])?;
        let mut main = vec![rim, centre_mark, spoke, radius_label, heard, note];
        main.extend(&shaded);
        main.extend(&year_labels);
        main.extend(&lines);
        let mut left_over: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(whole.shown, 0.0, 0.7)?)?];
        for &m in &main {
            stage.clear_updaters(m, true);
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            left_over.push(stage.prepare(gone.rt(0.7))?);
        }
        stage.play_prepared(left_over)?;
        whole.remove(stage);

        let counted = Cloud::build(stage, DISC_AT[0], DISC_R, &plan.disc(true))?;
        let left_out = Cloud::build(stage, DISC_AT[1], DISC_R, &plan.disc(false))?;
        let mut pair: Vec<Mob> = Vec::new();
        for centre in DISC_AT {
            pair.push(circle(stage, centre, DISC_R));
            pair.push(diamond(stage, centre, 0.12));
            pair.extend(track_lines(stage, centre, DISC_R, &plan.tracks, 2.5));
        }
        let heard_words = wrap(
            &format!(
                "{SILENCE_COUNTS} {} in 100 {WOULD_HAVE_BEEN_HEARD}",
                fact("F-A.beacon.used.percent")?
            ),
            34,
            2,
        );
        let mut captions: Vec<Mob> = Vec::new();
        for (j, text) in heard_words.iter().enumerate() {
            captions.push(kit.text(
                stage,
                text,
                22.0,
                Palette::ink(),
                v(DISC_AT[0][0], DISC_AT[0][1] - DISC_R - 0.4 - 0.3 * j as f64),
            )?);
        }
        captions.push(kit.text(
            stage,
            SILENCE_LEFT_OUT,
            22.0,
            Palette::ink(),
            v(DISC_AT[1][0], DISC_AT[1][1] - DISC_R - 0.4),
        )?);
        fade_in_all(stage, &pair, 0.6)?;
        fade_in_all(stage, &captions, 0.5)?;
        // the four empty searches are in both maps from the first frame
        for c in [&counted, &left_out] {
            stage
                .arena_mut()
                .set_tracker_value(c.area, 1.0)
                .map_err(|e| fail(format!("a cloud's tracker: {e:?}")))?;
        }
        let both = vec![
            stage.prepare(tween(counted.shown, 1.0, 0.8)?)?,
            stage.prepare(tween(left_out.shown, 1.0, 0.8)?)?,
        ];
        stage.play_prepared(both)?;
        // the authors' own words stand under the two maps from the moment
        // they are drawn twice, so they can be read before the line is over
        let better = lower_third(&kit, stage, AUTHORS_WAY, AUTHORS_TEXT)?;
        line.wait_for(stage, "So the analysts drew")?;
        fade_in_all(stage, &[better], 0.5)?;
        // the left disc's dots under the listening tracks go out
        line.wait_for(stage, "once counting the silence")?;
        kit.mark(stage, SEARCH_MARK);
        stage.play(tween(counted.beacon, 1.0, 1.0)?)?;
        line.wait_end(stage)?;

        // ------------------------------------------------------- 12_6
        // both put their weight near the centre; the wreck lay at one place
        // on both, in water the listening ships had crossed
        let line = say(&kit, stage, LINES[5])?;
        fade_out_all(stage, &[better], 0.4)?;
        line.wait_for(stage, "The search resumed there")?;
        let rings: Vec<Mob> = DISC_AT
            .iter()
            .map(|&c| ring_mark(stage, c, DISC_R, plan.ring))
            .collect();
        let found_words = kit.text(
            stage,
            &format!(
                "{ABOUT} {} {NAUTICAL_MILES} ({} {KM}) {FROM_LAST}",
                fact("F-A.found.nm")?,
                fact("F-A.found.km")?
            ),
            24.0,
            Palette::ink(),
            v(0.0, DISC_AT[0][1] - DISC_R - 1.1),
        )?;
        fade_in_all(stage, &rings, 0.6)?;
        fade_in_all(stage, &[found_words], 0.5)?;
        line.wait_for(stage, "It lay in a high-chance part")?;
        // last, one listening track is drawn across the ring, on both
        let through = plan
            .tracks
            .iter()
            .copied()
            .find(|t| t.hears(plan.ring))
            .ok_or_else(|| fail("no listening track crosses the ring".into()))?;
        let mut crossing: Vec<Mob> = Vec::new();
        for centre in DISC_AT {
            crossing.extend(track_lines(stage, centre, DISC_R, &[through], 6.0));
        }
        let date_and_plan = kit.text(
            stage,
            &format!("{}. {FIRST_MAP}", month_year(&fact("F-A.found")?).map_err(fail)?),
            24.0,
            Palette::dim(),
            v(0.0, DISC_AT[0][1] - DISC_R - 1.6),
        )?;
        let mut drawing: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &crossing {
            drawing.push(stage.prepare(show_creation(m).rt(1.2))?);
        }
        stage.play_prepared(drawing)?;
        fade_in_all(stage, &[date_and_plan], 0.5)?;
        line.wait_end(stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, HOLD_S)?;

        // the clouds leave through their own trackers, the rest by fades
        let keep: HashSet<Mob> = [counted.group, counted.anchor, left_out.group, left_out.anchor]
            .into_iter()
            .collect();
        let mut leaving: Vec<Box<dyn Animation>> = vec![
            stage.prepare(tween(counted.shown, 0.0, 0.8)?)?,
            stage.prepare(tween(left_out.shown, 0.0, 0.8)?)?,
        ];
        let rest: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| !keep.contains(m))
            .collect();
        for m in rest {
            stage.clear_updaters(m, true);
            let gone = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            leaving.push(stage.prepare(gone.rt(0.8))?);
        }
        stage.play_prepared(leaving)?;
        counted.remove(stage);
        left_out.remove(stage);
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many dots of a disc are still lit.
    fn lit(d: &Dots) -> usize {
        (0..d.at.len()).filter(|&i| !d.by_area[i] && !d.by_beacon[i]).count()
    }

    /// The chapter speaks exactly the script's 12_ lines, in order.
    #[test]
    fn the_chapter_speaks_the_scripts_lines_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("12_"))
            .collect();
        assert_eq!(ids, LINES);
    }

    /// 12_1 and 12_2: the date, the number aboard, the circle and the
    /// years are the facts', worded as the storyboard words them.
    #[test]
    fn the_words_on_the_screen_are_the_facts() {
        let f = facts().unwrap();
        assert_eq!(day_month_year(f.get("F-A.lost").unwrap()).unwrap(), "1 June 2009");
        assert_eq!(f.get("F-A.aboard"), Some("228"));
        assert_eq!(
            radius_words(f.get("F-A.circle.nm").unwrap(), f.get("F-A.circle.km").unwrap()),
            "40 nautical miles (74 km)"
        );
        assert_eq!(f.get("F-A.months"), Some("22"));
        assert_eq!(f.get("F-A.underwater_searches"), Some("3"));
        assert_eq!(f.get("F-A.underwater_searches.years"), Some("2009,2009,2010"));
        assert_eq!(AREAS.len().to_string(), f.get("F-A.underwater_searches").unwrap());
        assert_eq!(month_year(f.get("F-A.found").unwrap()).unwrap(), "April 2011");
        assert_eq!(f.get("F-A.found.nm"), Some("6.5"));
        assert_eq!(f.get("F-A.found.km"), Some("12"));
        assert_eq!(f.get("F-A.beacon.used.percent"), Some("77"));
        assert!(parts("2011-13-01").is_err());
    }

    /// 12_2: the radius label is two lines at the spoke's end, right of the
    /// circle's rim (on one line over the spoke "km)" touched the rim; over
    /// it, two lines stood on the 2010 area), clear of the circle and of the
    /// frame's safe edge, and both lines keep every word.
    #[test]
    fn the_radius_label_stands_right_of_the_rim_and_in_the_frame() {
        use super::super::draw::CHAR_UNITS_PER_POINT;
        let f = facts().unwrap();
        let words = radius_words(f.get("F-A.circle.nm").unwrap(), f.get("F-A.circle.km").unwrap());
        let lines = wrap(&words, RADIUS_LINE_CHARS, 2);
        assert_eq!(lines, ["40 nautical miles", "(74 km)"]);
        assert_eq!(lines.join(" "), words);
        let widest = lines.iter().map(|l| l.chars().count()).max().unwrap() as f64;
        let end = CIRCLE_AT[0] + CIRCLE_R + RADIUS_GAP + widest * RADIUS_PT * CHAR_UNITS_PER_POINT;
        assert!(end <= FRAME_X_RADIUS - 0.15, "the label ends at {end}");
        // no area, however it is drawn, reaches the rim's right side
        for a in &AREAS {
            let reach = CIRCLE_AT[0] + (a.at.0 + a.semi.0.max(a.semi.1)) * CIRCLE_R;
            assert!(reach < CIRCLE_AT[0] + CIRCLE_R, "an area reaches {reach}");
        }
    }

    /// R2 (M21, M6, R13): the gloss under the radius ("a nautical mile is a
    /// sea mile"), each story's name over its method and the count of empty
    /// searches in the left column each stand in their own room, clear of
    /// their neighbours and the frame.
    #[test]
    fn the_gloss_the_story_names_and_the_empty_searches_stand_clear_of_their_neighbours() {
        use super::super::draw::CHAR_UNITS_PER_POINT as UNITS;
        let f = facts().unwrap();
        // 12_2: two lines, right of the rim, under the radius label's two
        let gloss = wrap(NAUTICAL_IS_SEA, GLOSS_CHARS, 2);
        assert_eq!(gloss, ["a nautical mile", "is a sea mile"]);
        assert_eq!(gloss.join(" "), NAUTICAL_IS_SEA);
        let widest = gloss.iter().map(|l| l.chars().count()).max().unwrap() as f64;
        let end = CIRCLE_AT[0] + CIRCLE_R + RADIUS_GAP + widest * GLOSS_PT * UNITS;
        assert!(end <= FRAME_X_RADIUS - 0.15, "the gloss ends at {end}");
        // the radius label's last line is 20 pt, the gloss's first 18 pt, and
        // a line is about 0.2 tall: half of each and a margin
        const {
            let last_radius = RADIUS_RISE - RADIUS_STEP;
            assert!(last_radius - GLOSS_TOP >= 0.1 + 0.09 + 0.15);
            assert!(GLOSS_STEP >= 2.0 * 0.09 + 0.08);
        }
        // 12_3: a story's name over its method (22 pt, at most two lines, so
        // the 44 pt weight under it touches nothing), and each weight's foot
        // clears the next disc's name
        for name in METHODS {
            let lines = wrap(name, 26, 3);
            assert!(lines.len() <= 2, "{name:?} wraps to {} lines", lines.len());
            let last = METHOD_TOP - METHOD_STEP * (lines.len() - 1) as f64;
            assert!(last - 0.1 > -WEIGHT_DROP + 0.2, "{name:?}");
        }
        const {
            assert!(STORY_RISE - METHOD_TOP >= METHOD_STEP - 0.02);
            let gap = SMALL_DISC_Y[0] - SMALL_DISC_Y[1];
            assert!(gap - WEIGHT_DROP - 0.2 > STORY_RISE + 0.12);
            let gap = SMALL_DISC_Y[1] - SMALL_DISC_Y[2];
            assert!(gap - WEIGHT_DROP - 0.2 > STORY_RISE + 0.12);
            // the first story's name is inside the frame
            assert!(SMALL_DISC_Y[0] + STORY_RISE + 0.12 < 3.65);
        }
        // the count of empty searches, two lines in the left column under
        // the circle's own left rim, and not into the weights' tag below
        let [count, split] = empty_searches_words(
            f.get("F-A.empty_searches").unwrap().parse().unwrap(),
            f.get("F-A.underwater_searches").unwrap().parse().unwrap(),
        )
        .unwrap();
        assert_eq!(
            format!("{count} {split}"),
            "4 empty searches: 1 on the surface, 3 under water"
        );
        let widths = [
            count.chars().count() as f64 * 40.0 * UNITS,
            split.chars().count() as f64 * 26.0 * UNITS,
        ];
        for w in widths {
            assert!(LEFT_X + w < CIRCLE_AT[0] - CIRCLE_R - 0.3, "{w}");
        }
        assert!(empty_searches_words(3, 4).is_err());
        // the 40 pt line's foot over the 26 pt line's head
        const {
            assert!(EMPTY_Y[0] - EMPTY_Y[1] >= 0.2 + 0.13 + 0.1);
            // and the weights' tag (at -2.65) and the particles' note
            // (at -3.15) are far below
            assert!(EMPTY_Y[1] - 0.15 > -2.65 + 0.3);
        }
    }

    /// 12_3: three maps weighted 35, 35 and 30 make 1,050, 1,050 and 900 of
    /// the 3,000 dots, and a dot stands for 25 of the 75,000 particles.
    #[test]
    fn the_cloud_is_the_weights_in_dots_and_a_dot_is_twenty_five() {
        let f = facts().unwrap();
        let p = plan(&f).unwrap();
        assert_eq!(p.weights, [35, 35, 30]);
        assert_eq!(p.weights.iter().sum::<u32>(), 100);
        assert_eq!(p.all.len(), DOTS_SHOWN);
        let count = |k: usize| p.kind.iter().filter(|&&m| m == k).count();
        assert_eq!([count(0), count(1), count(2)], [1050, 1050, 900]);
        assert_eq!(
            particles_words(75_000, DOTS_SHOWN).unwrap(),
            "75,000 particles; each dot here stands for 25"
        );
        assert_eq!(f.get("F-A.points"), Some("75000"));
        assert!(particles_words(75_001, DOTS_SHOWN).is_err());
        // a sketch is a tenth of its map
        assert_eq!(
            [p.sketch(0).len(), p.sketch(1).len(), p.sketch(2).len()],
            [105, 105, 90]
        );
    }

    /// Every dot is inside the circle, and every area and track is.
    #[test]
    fn everything_drawn_is_inside_the_circle() {
        let p = plan(&facts().unwrap()).unwrap();
        assert!(p.all.iter().all(|d| d.0.hypot(d.1) <= EDGE + 1e-12));
        for a in &AREAS {
            assert!(
                a.outline(96).iter().all(|q| q.0.hypot(q.1) < 0.95),
                "{a:?} reaches the circle's edge"
            );
        }
        for t in &p.tracks {
            assert!((t.from.0.hypot(t.from.1) - 1.0).abs() < 1e-9 && (t.to.0.hypot(t.to.1) - 1.0).abs() < 1e-9);
        }
        assert_eq!(p.tracks.len(), 5);
    }

    /// The wreck: the facts' 6.5 of 40 nautical miles from the centre,
    /// north-north-east; in none of the searched areas (three searches found
    /// nothing), and on a listening track (the ships had crossed that water).
    #[test]
    fn the_ring_is_clear_of_every_search_and_on_a_listening_track() {
        let f = facts().unwrap();
        let p = plan(&f).unwrap();
        assert_eq!(number(&f, "F-A.circle.nm").unwrap(), 40.0);
        assert_eq!(number(&f, "F-A.found.nm").unwrap(), 6.5);
        let r = p.ring.0.hypot(p.ring.1);
        assert!((r - 6.5 / 40.0).abs() < 1e-12, "{r}");
        let bearing = p.ring.0.atan2(p.ring.1).to_degrees();
        assert!((bearing - BEARING_EAST_OF_NORTH).abs() < 1e-9, "{bearing}");
        for a in &AREAS {
            assert!(!a.contains(p.ring, 1.25), "{a:?} holds the wreck");
        }
        let crossing = p.tracks.iter().filter(|t| t.hears(p.ring)).count();
        assert_eq!(crossing, 1, "exactly one track runs through it");
        // and the nearest dot-band of any other track is a clear spacing away
        let others = p.tracks.iter().filter(|t| !t.hears(p.ring));
        let n = (-TRACK_TURN.sin(), TRACK_TURN.cos());
        let d = p.ring.0 * n.0 + p.ring.1 * n.1;
        for t in others {
            assert!((t.offset - d).abs() >= TRACK_SPACING - 1e-9);
        }
    }

    /// 12_3: the searched areas put out nine in ten of the dots inside them,
    /// in whole dots, and no dot outside them.
    #[test]
    fn the_searches_put_out_nine_in_ten_of_what_is_inside() {
        let p = plan(&facts().unwrap()).unwrap();
        let d = p.whole();
        let inside: Vec<usize> = (0..d.at.len())
            .filter(|&i| AREAS.iter().any(|a| a.contains(d.at[i], 1.0)))
            .collect();
        assert!(inside.len() > 100, "{} dots inside", inside.len());
        let out = d.by_area.iter().filter(|&&b| b).count();
        assert_eq!(out, dots_out(inside.len(), Q::new(9, 10)));
        assert!((out as f64 - 0.9 * inside.len() as f64).abs() <= 0.5);
        assert!(d.by_area.iter().enumerate().all(|(i, &b)| !b || inside.contains(&i)));
        assert!(d.by_beacon.iter().all(|&b| !b), "the big cloud has no silence in it");
    }

    /// 12_5: 77 in 100 of the dots still lit under the listening tracks go
    /// out on the left disc, and nothing goes out on the right; the discs
    /// share their dots, and a dot is never put out twice.
    #[test]
    fn the_left_disc_counts_the_silence_and_the_right_leaves_it_out() {
        let f = facts().unwrap();
        let p = plan(&f).unwrap();
        let (left, right) = (p.disc(true), p.disc(false));
        assert_eq!(left.at, right.at, "the same places on both discs");
        assert_eq!(left.at.len(), DOTS_SHOWN / 2);
        assert_eq!(left.by_area, right.by_area, "the same four searches on both");
        assert!(right.by_beacon.iter().all(|&b| !b));
        let heard: Vec<usize> = (0..left.at.len())
            .filter(|&i| !left.by_area[i] && p.tracks.iter().any(|t| t.hears(left.at[i])))
            .collect();
        assert!(heard.len() > 100, "{} dots under the tracks", heard.len());
        let gone = left.by_beacon.iter().filter(|&&b| b).count();
        assert_eq!(gone, dots_out(heard.len(), Q::new(77, 100)));
        assert!((gone as f64 - 0.77 * heard.len() as f64).abs() <= 0.5);
        assert!(
            left.by_beacon
                .iter()
                .enumerate()
                .all(|(i, &b)| !b || heard.contains(&i))
        );
        assert!(
            left.by_area.iter().zip(&left.by_beacon).all(|(&a, &b)| !(a && b)),
            "no dot goes out twice"
        );
        assert_eq!(p.listening, Q::new(77, 100));
        assert_eq!(
            lit(&left) + left.by_area.iter().filter(|&&a| a).count() + gone,
            left.at.len()
        );
        assert!(lit(&left) < lit(&right), "counting the silence puts dots out");
    }

    /// The edges of "put out a share": none of nothing, all of everything,
    /// and half a dot rounds up.
    #[test]
    fn a_share_of_dots_goes_out_in_whole_dots_half_up() {
        assert_eq!(dots_out(0, Q::new(9, 10)), 0);
        assert_eq!(dots_out(10, Q::ZERO), 0);
        assert_eq!(dots_out(10, Q::ONE), 10);
        assert_eq!(dots_out(5, Q::new(1, 10)), 1, "half a dot is a dot");
        assert_eq!(dots_out(3, Q::new(1, 2)), 2);
        let chosen = choose_out(&[4, 8, 15, 16, 23, 42], Q::new(1, 2), 7);
        assert_eq!(chosen.len(), 3);
        let mut unique = chosen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3, "never the same dot twice");
    }

    /// The flight's last report is on the route, and the route runs from
    /// Rio's side of the stage to Paris's.
    #[test]
    fn the_flight_runs_rio_to_paris_and_the_last_report_is_on_it() {
        let route = flight(40);
        assert_eq!(route.len(), 41);
        assert!(route[0][0] < route[40][0] && route[0][1] < route[40][1]);
        let last = (route.len() as f64 * LAST_REPORT_AT) as usize;
        assert!(
            last > 0 && last < route.len() - 1,
            "the diamond is somewhere on the way"
        );
        assert!(route.iter().all(|p| p[0].abs() < 7.0 && p[1].abs() < 3.0));
    }
}
