//! A useful new instance and an almost-instance that loses dependencies.
use super::draw::{Map, ROUTES, hide, prediction, route_color, show};
use crate::model::{self, Best, Count, Quickest, QuickestCount, ROADS, Reachable, Widest, Width};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use std::collections::BTreeSet;

pub struct Chapter15 {
    pub kit: Kit,
}
const JOBS: [&str; 5] = ["Chop", "Boil", "Fry", "Roast", "Simmer"];
/// The two jobs that end at Finish: Finish's two offers.
const ROAST: usize = 3;
const SOUP: usize = 4;
type Badge = BTreeSet<u32>;

pub fn schedule() -> ([u64; 4], [(u64, u64); 5]) {
    let mut badges = [0; 4];
    let mut bars = [(0, 0); 5];
    for place in model::topological_order(4, &ROADS).expect("the fixed map is acyclic") {
        for (i, road) in ROADS.iter().enumerate().filter(|(_, r)| r.from == place) {
            bars[i] = (badges[place], badges[place] + road.value);
            badges[road.to] = badges[road.to].max(bars[i].1);
        }
    }
    (badges, bars)
}
fn union(a: &Badge, b: &Badge) -> Badge {
    a.union(b).copied().collect()
}
fn extend(a: &Badge, b: &Badge) -> Badge {
    a.iter().flat_map(|x| b.iter().map(move |y| x | y)).collect()
}
pub fn road_sets() -> [Badge; 4] {
    let mut badges: [Badge; 4] = std::array::from_fn(|_| Badge::new());
    badges[0].insert(0);
    for place in model::topological_order(4, &ROADS).expect("fixed DAG") {
        for (i, road) in ROADS.iter().enumerate().filter(|(_, r)| r.from == place) {
            let offer = extend(&badges[place], &Badge::from([1 << i]));
            badges[road.to] = union(&badges[road.to], &offer);
        }
    }
    badges
}
fn open(mask: u32, sets: &Badge) -> bool {
    sets.iter().any(|&route| mask & route == route)
}
/// The chance that Finish is reachable, as a numerator over `10^ROADS.len()`.
pub fn reliability_numerator() -> u64 {
    let sets = road_sets();
    let (up, down) = (OPEN_TENTHS as u64, 10 - OPEN_TENTHS as u64);
    (0u32..1 << ROADS.len())
        .filter(|&m| open(m, &sets[3]))
        .map(|m| up.pow(m.count_ones()) * down.pow(ROADS.len() as u32 - m.count_ones()))
        .sum()
}
/// Each road is open this many times in ten, independently of the others.
const OPEN_TENTHS: u128 = 9;
fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}
/// An exact chance `num / 10^exp`; every chance in this chapter is a finite decimal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Chance {
    num: u128,
    exp: u32,
}
impl Chance {
    const ZERO: Self = Self { num: 0, exp: 0 };
    const ONE: Self = Self { num: 1, exp: 0 };
    /// One road is open.
    const ROAD: Self = Self {
        num: OPEN_TENTHS,
        exp: 1,
    };
    fn new(num: u128, exp: u32) -> Self {
        Self { num, exp }
    }
    /// Both of two independent events: extend along a road.
    fn then(self, other: Self) -> Self {
        Self::new(self.num * other.num, self.exp + other.exp)
    }
    /// The patched combine: at least one of two events, treated as independent.
    fn either(self, other: Self) -> Self {
        let (a, b) = (10u128.pow(self.exp), 10u128.pow(other.exp));
        Self::new(a * b - (a - self.num) * (b - other.num), self.exp + other.exp)
    }
    /// Plain addition of chances, which can pass one.
    fn plus(self, other: Self) -> Self {
        let exp = self.exp.max(other.exp);
        Self::new(
            self.num * 10u128.pow(exp - self.exp) + other.num * 10u128.pow(exp - other.exp),
            exp,
        )
    }
    fn pow(self, n: u32) -> Self {
        Self::new(self.num.pow(n), self.exp * n)
    }
    /// Lowest terms, as Python's `Fraction` prints it.
    fn fraction(self) -> String {
        let den = 10u128.pow(self.exp);
        let g = gcd(self.num, den);
        if den / g == 1 {
            (self.num / g).to_string()
        } else {
            format!("{}/{}", self.num / g, den / g)
        }
    }
    /// Rounded half up to `digits` decimal places, for a value marked "about".
    fn about(self, digits: u32) -> String {
        let scaled = if self.exp >= digits {
            let step = 10u128.pow(self.exp - digits);
            (self.num + step / 2) / step
        } else {
            self.num * 10u128.pow(digits - self.exp)
        };
        let unit = 10u128.pow(digits);
        format!("{}.{:0width$}", scaled / unit, scaled % unit, width = digits as usize)
    }
    fn value(self) -> f64 {
        self.num as f64 / 10f64.powi(self.exp as i32)
    }
}
fn patched_sweep_exact() -> [Chance; 4] {
    let mut p = [Chance::ONE, Chance::ZERO, Chance::ZERO, Chance::ZERO];
    for place in model::topological_order(4, &ROADS).expect("fixed DAG") {
        for road in ROADS.iter().filter(|r| r.from == place) {
            p[road.to] = p[road.to].either(p[place].then(Chance::ROAD));
        }
    }
    p
}
pub fn patched_sweep() -> [f64; 4] {
    patched_sweep_exact().map(Chance::value)
}
fn route_chances() -> Vec<Chance> {
    ROUTES.iter().map(|r| Chance::ROAD.pow(r.len() as u32)).collect()
}
/// One road followed by two side by side: combine first, or extend first.
fn law() -> (Chance, Chance) {
    let x = Chance::ROAD;
    (x.then(x.either(x)), x.then(x).either(x.then(x)))
}
/// The quickest dinner when at most `at_once` jobs run in any minute, and every finishing
/// minute of the soup among such quickest plans: a search over whole-minute start times.
fn capacity(at_once: usize) -> (u64, BTreeSet<u64>) {
    let horizon: u64 = ROADS.iter().map(|r| r.value).sum();
    let mut starts = [0u64; ROADS.len()];
    let mut best = horizon + 1;
    let mut soup = BTreeSet::new();
    loop {
        let ready = ROADS.iter().enumerate().all(|(i, road)| {
            ROADS
                .iter()
                .enumerate()
                .filter(|(_, r)| r.to == road.from)
                .all(|(k, r)| starts[k] + r.value <= starts[i])
        });
        if ready {
            let done = ROADS
                .iter()
                .enumerate()
                .map(|(i, r)| starts[i] + r.value)
                .max()
                .unwrap_or(0);
            let crowded = (0..done).any(|t| {
                ROADS
                    .iter()
                    .enumerate()
                    .filter(|&(i, r)| (starts[i]..starts[i] + r.value).contains(&t))
                    .count()
                    > at_once
            });
            if !crowded && done <= best {
                if done < best {
                    best = done;
                    soup.clear();
                }
                soup.insert(starts[SOUP] + ROADS[SOUP].value);
            }
        }
        let mut k = 0;
        loop {
            if k == starts.len() {
                return (best, soup);
            }
            starts[k] += 1;
            if starts[k] <= horizon {
                break;
            }
            starts[k] = 0;
            k += 1;
        }
    }
}
/// Exact values shown or spoken in 15_6, 15_8, 15_10 and 15_11, for the oracle to pin.
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add(
        "rel.sum.of.routes",
        route_chances().into_iter().fold(Chance::ZERO, Chance::plus).fraction(),
    )?;
    let (combine_first, extend_first) = law();
    facts.add(
        "rel.law",
        format!("{},{}", combine_first.fraction(), extend_first.fraction()),
    )?;
    facts.add(
        "rel.list.patched",
        route_chances()
            .into_iter()
            .fold(Chance::ZERO, Chance::either)
            .fraction(),
    )?;
    facts.add("rel.sweep", patched_sweep_exact()[3].fraction())?;
    let (best, soup) = capacity(2);
    facts.add("project.capacity.best.2", best)?;
    facts.add(
        "project.capacity.soup.ends.2",
        soup.iter().map(u64::to_string).collect::<Vec<_>>().join(","),
    )?;
    Ok(())
}
fn set_text(mask: u32) -> String {
    format!(
        "{{{}}}",
        (0..ROADS.len())
            .filter(|i| mask & (1 << i) != 0)
            .map(|i| ((b'a' + i as u8) as char).to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn txt(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, x: f64, y: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, size, c, v(x, y))
}
fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(a, b)
            .style(Style::default().stroke(color, 3.0, 1.0))
            .build()?,
    ))
}
fn shift(stage: &mut Stage<'_>, mob: Mob, at: Vec3) -> fmn::Result<()> {
    stage.play(
        mob.animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..Default::default()
            })?
            .move_to(at, ORIGIN)?,
    )?;
    Ok(())
}
fn bar(stage: &mut Stage<'_>, start: f64, end: f64, y: f64, scale: f64, c: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width((end - start) * scale)
            .height(0.26)
            .style(Style::default().fill(c, 0.8).stroke(c, 1.0, 1.0))
            .build()?
            .moved_to(v(-4.5 + (start + end) * scale / 2.0, y)),
    ))
}
impl SceneConstruct for Chapter15 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let (times, bars) = schedule();
        let sets = road_sets();
        let total: u64 = ROADS.iter().map(|r| r.value).sum();
        let truth = Chance::new(u128::from(reliability_numerator()), ROADS.len() as u32);
        let chances = route_chances();
        let sweep = patched_sweep_exact()[3];
        kit.say(stage, "15_1")?;
        let title = txt(kit, stage, "When is dinner ready?", 42.0, 0.0, 3.0, Palette::ink())?;
        show(stage, &[title], 0.3)?;
        // Fry sits between the rows: on the top row the Chop–Roast line would run through it and
        // read as one chain, Chop then Fry then Roast. Roast needs only the chopping. It also
        // sits right of the 15_4 map's Finish node, which fades in before the cards move.
        let positions = [(-4.2, 1.4), (-4.2, -1.3), (1.3, 0.05), (3.8, 1.4), (3.8, -1.3)];
        let mut cards = Vec::new();
        for (i, &(x, y)) in positions.iter().enumerate() {
            let c = txt(
                kit,
                stage,
                &format!("{}: {} min", JOBS[i], ROADS[i].value),
                28.0,
                x,
                y,
                Palette::gold(),
            )?;
            show(stage, &[c], 0.3)?;
            cards.push(c);
        }
        kit.hold(stage)?;
        kit.say(stage, "15_2")?;
        let mut dependencies = Vec::new();
        for (a, b) in [(0, 2), (0, 3), (1, 4), (2, 4)] {
            let (x, y) = positions[a];
            let (u, w) = positions[b];
            let edge = line(stage, v(x + 0.9, y - 0.3), v(u - 0.9, w - 0.3), Palette::dim())?;
            show(stage, &[edge], 0.25)?;
            dependencies.push(edge);
        }
        let assumption = txt(
            kit,
            stage,
            "Enough hands, burners and ovens: jobs may overlap",
            25.0,
            0.0,
            2.35,
            Palette::teal(),
        )?;
        show(stage, &[assumption], 0.3)?;
        prediction(kit, stage, 6, v(0.0, -2.3))?;
        kit.say(stage, "15_3")?;
        let rule = txt(
            kit,
            stage,
            "Along a chain: add     Waiting for both: max",
            29.0,
            0.0,
            -2.5,
            Palette::teal(),
        )?;
        show(stage, &[rule], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "15_4")?;
        hide(stage, &dependencies, 0.3)?;
        hide(stage, &[title, rule], 0.3)?;
        let map = Map::build(kit, stage, [1.1, 0.0], 0.95, true)?;
        // The map without its minutes. While the costs are hidden, hide this rather than
        // `map.all()`: fading an already-hidden mob brings it back for a moment.
        let plain: Vec<Mob> = map.nodes.iter().chain(&map.names).chain(&map.roads).copied().collect();
        show(stage, &map.all(), 0.4)?;
        // Keep task identities while moving each card onto its corresponding road.
        for (i, &card) in cards.iter().enumerate() {
            let r = ROADS[i];
            let a = map.points[r.from];
            let b = map.points[r.to];
            // The vertical road needs a side label; lower-right text must
            // clear the Lake node and the outgoing arrowhead.
            let offset = match i {
                2 => [1.0, 0.0],
                4 => [0.45, -0.9],
                1 => [0.0, -0.7],
                _ => [0.0, 0.7],
            };
            shift(
                stage,
                card,
                v((a[0] + b[0]) / 2.0 + offset[0], (a[1] + b[1]) / 2.0 + offset[1]),
            )?;
        }
        hide(stage, &map.costs, 0.3)?;
        // Each readout lands with the sentence that says it; Finish's two offers are the
        // finishing times of the soup (sentence 2) and the roast (sentence 3).
        let mut readouts = Vec::new();
        let rows = [
            (0, format!("{} ready: {}", model::PLACES[0], times[0])),
            (0, format!("{} ready: {}", model::PLACES[1], times[1])),
            (2, format!("{} ready: {}", model::PLACES[2], times[2])),
            (2, format!("{} done: {}", JOBS[SOUP], bars[SOUP].1)),
            (3, format!("{} done: {}", JOBS[ROAST], bars[ROAST].1)),
            (4, format!("{} ready: {}", model::PLACES[3], times[3])),
        ];
        for (k, (sentence, s)) in rows.iter().enumerate() {
            kit.until_sentence(stage, "15_4", *sentence)?;
            let m = txt(kit, stage, s, 28.0, 3.8, 1.5 - k as f64 * 0.6, Palette::teal())?;
            show(stage, &[m], 0.3)?;
            readouts.push(m);
        }
        kit.hold(stage)?;
        kit.say(stage, "15_5")?;
        map.travel(stage, ROUTES[0], route_color(0))?;
        let critical = txt(
            kit,
            stage,
            &format!("Critical path: Chop + Roast = {}", times[3]),
            30.0,
            0.0,
            -2.7,
            route_color(0),
        )?;
        show(stage, &[critical], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "15_6")?;
        hide(stage, &plain, 0.3)?;
        hide(stage, &cards, 0.3)?;
        hide(stage, &readouts, 0.3)?;
        hide(stage, &[critical], 0.3)?;
        let mut chart = Vec::new();
        for (i, &(start, end)) in bars.iter().enumerate() {
            let y = 1.45 - i as f64 * 0.68;
            let b = bar(stage, start as f64, end as f64, y, 0.85, route_color(i % 3))?;
            let label = txt(
                kit,
                stage,
                &format!("{}  {start}–{end}", JOBS[i]),
                23.0,
                4.5,
                y,
                Palette::ink(),
            )?;
            show(stage, &[b, label], 0.25)?;
            chart.extend([b, label]);
        }
        kit.until_sentence(stage, "15_6", 1)?;
        let overlap = bar(stage, 2.0, 3.0, 1.9, 0.85, Palette::gold())?;
        let overlap_label = txt(kit, stage, "3 at once", 24.0, -2.35, 2.15, Palette::gold())?;
        show(stage, &[overlap, overlap_label], 0.3)?;
        kit.until_sentence(stage, "15_6", 2)?;
        hide(stage, &chart, 0.3)?;
        hide(stage, &[overlap, overlap_label], 0.3)?;
        let mut serial = Vec::new();
        let mut end = 0;
        for (i, r) in ROADS.iter().enumerate() {
            let b = bar(stage, end as f64, (end + r.value) as f64, 0.0, 0.53, route_color(i % 3))?;
            show(stage, &[b], 0.25)?;
            serial.push(b);
            end += r.value;
        }
        let serial_note = txt(
            kit,
            stage,
            &format!("One job at a time: {total} minutes"),
            32.0,
            0.0,
            -1.0,
            Palette::gold(),
        )?;
        show(stage, &[serial_note], 0.3)?;
        kit.until_sentence(stage, "15_6", 3)?;
        let (best_two, soup_two) = capacity(2);
        let precisely_two = txt(
            kit,
            stage,
            &format!(
                "Precisely: with two jobs at a time, dinner can still be ready at {best_two},\nbut then the soup finishes at {}, not {}",
                soup_two.iter().map(u64::to_string).collect::<Vec<_>>().join(" or "),
                bars[SOUP].1
            ),
            24.0,
            0.0,
            -2.1,
            Palette::dim(),
        )?;
        show(stage, &[precisely_two], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &serial, 0.3)?;
        hide(stage, &[serial_note, assumption, precisely_two], 0.3)?;
        kit.say(stage, "15_7")?;
        // The reliability map shows no minutes: the dinner costs stay hidden.
        show(stage, &plain, 0.3)?;
        let heading = txt(
            kit,
            stage,
            &format!("Independent roads, each open with chance {}", Chance::ROAD.fraction()),
            29.0,
            0.0,
            3.0,
            Palette::ink(),
        )?;
        show(stage, &[heading], 0.3)?;
        for &road in &map.roads {
            hide(stage, &[road], 0.12)?;
            show(stage, &[road], 0.12)?;
        }
        prediction(kit, stage, 5, v(3.5, 0.0))?;
        kit.say(stage, "15_8")?;
        let mut calculations = Vec::new();
        // Exact fractions, as the voice says them ("eighty-one times in a hundred").
        kit.until_sentence(stage, "15_8", 1)?;
        for (i, chance) in chances.iter().enumerate() {
            map.travel(stage, ROUTES[i], route_color(i))?;
            let row = txt(
                kit,
                stage,
                &format!("Route {}: {}", i + 1, chance.fraction()),
                29.0,
                3.7,
                1.4 - i as f64 * 0.65,
                route_color(i),
            )?;
            show(stage, &[row], 0.25)?;
            calculations.push(row);
        }
        kit.until_sentence(stage, "15_8", 3)?;
        // The sum has three decimal places, so this is exact.
        let sum = txt(
            kit,
            stage,
            &format!(
                "Sum = {} > 1",
                chances.iter().copied().fold(Chance::ZERO, Chance::plus).about(3)
            ),
            32.0,
            3.5,
            -1.1,
            Palette::red(),
        )?;
        show(stage, &[sum], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "15_9")?;
        map.travel(stage, ROUTES[0], route_color(0))?;
        map.travel(stage, ROUTES[2], route_color(2))?;
        // Road letters first appear in 15_12, so name the shared road by its places.
        let common = ROUTES[0]
            .iter()
            .copied()
            .find(|r| ROUTES[2].contains(r))
            .expect("the first and third routes share a road");
        let shared = txt(
            kit,
            stage,
            &format!(
                "Both routes use {} → {}",
                model::PLACES[ROADS[common].from],
                model::PLACES[ROADS[common].to]
            ),
            27.0,
            0.0,
            -2.6,
            Palette::gold(),
        )?;
        show(stage, &[shared], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &calculations, 0.25)?;
        hide(stage, &[sum, shared], 0.25)?;
        kit.say(stage, "15_10")?;
        let patch = txt(
            kit,
            stage,
            "Combine: 1 − (1 − a)(1 − b)",
            29.0,
            2.5,
            1.8,
            Palette::gold(),
        )?;
        show(stage, &[patch], 0.3)?;
        // Rounded values say "about", as the voice does; the exact fraction sits under each.
        kit.until_sentence(stage, "15_10", 2)?;
        let estimate = txt(
            kit,
            stage,
            &format!("Sweep: about {}", sweep.about(3)),
            29.0,
            3.5,
            0.9,
            Palette::red(),
        )?;
        let estimate_exact = txt(
            kit,
            stage,
            &format!("exactly {}", sweep.fraction()),
            24.0,
            3.5,
            0.5,
            Palette::dim(),
        )?;
        show(stage, &[estimate, estimate_exact], 0.3)?;
        kit.until_sentence(stage, "15_10", 3)?;
        let mut patterns = Vec::new();
        for mask in 0u32..1 << ROADS.len() {
            let x = -5.5 + (mask % 16) as f64 * 0.72;
            let y = -2.65 - (mask / 16) as f64 * 0.65;
            let box_mob = stage.arena_mut().add(
                Rectangle::new()
                    .width(0.56)
                    .height(0.4)
                    .style(Style::default().fill(
                        if open(mask, &sets[3]) {
                            Palette::teal()
                        } else {
                            Palette::red()
                        },
                        0.65,
                    ))
                    .build()?
                    .moved_to(v(x, y)),
            );
            show(stage, &[box_mob], 0.06)?;
            patterns.push(box_mob);
        }
        let exact = txt(
            kit,
            stage,
            &format!("All patterns: about {}", truth.about(3)),
            29.0,
            3.5,
            0.0,
            Palette::teal(),
        )?;
        let exact_fraction = txt(
            kit,
            stage,
            &format!("exactly {}", truth.fraction()),
            24.0,
            3.5,
            -0.4,
            Palette::dim(),
        )?;
        // The boxes are equal in size but not in chance, so the teal share of boxes is not the
        // answer. The note lands with the readout, in the same fade.
        let unequal = txt(
            kit,
            stage,
            "Patterns are not equally likely",
            24.0,
            3.5,
            -0.85,
            Palette::dim(),
        )?;
        show(stage, &[exact, exact_fraction, unequal], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &patterns, 0.3)?;
        hide(
            stage,
            &[patch, estimate, estimate_exact, exact, exact_fraction, unequal],
            0.3,
        )?;
        kit.say(stage, "15_11")?;
        // Sentence 0 still points at the two offers meeting at Finish, so the map stays.
        kit.until_sentence(stage, "15_11", 1)?;
        hide(stage, &plain, 0.3)?;
        hide(stage, &[heading], 0.3)?;
        let (combine_first, extend_first) = law();
        let listed = chances.iter().copied().fold(Chance::ZERO, Chance::either);
        // Colour carries the verdict, as in 15_10: teal right, red wrong. On the small piece
        // combining first is right (sentence 4); in the whole-map row only the truth is.
        let wrong_sweep = format!("sweep {}", sweep.about(3));
        let wrong_list = format!("patched route list {}", listed.about(3));
        let right_truth = format!("truth {}", truth.about(3));
        let verdicts = [
            (wrong_sweep.as_str(), Palette::red()),
            (wrong_list.as_str(), Palette::red()),
            (right_truth.as_str(), Palette::teal()),
        ];
        let mut law_rows = Vec::new();
        for (i, (sentence, s)) in [
            (1, "One chance per badge forgets shared roads".to_string()),
            (
                2,
                format!(
                    "Combine, then extend: {0} × ({0} ⊕ {0}) = {1}",
                    Chance::ROAD.fraction(),
                    combine_first.fraction()
                ),
            ),
            (
                3,
                format!(
                    "Extend, then combine: {0} ⊕ {0} = {1}, about {2}",
                    Chance::ROAD.then(Chance::ROAD).fraction(),
                    extend_first.fraction(),
                    extend_first.about(3)
                ),
            ),
            (
                5,
                format!(
                    "Whole map, rounded: sweep {}  /  patched route list {}  /  truth {}",
                    sweep.about(3),
                    listed.about(3),
                    truth.about(3)
                ),
            ),
        ]
        .iter()
        .enumerate()
        {
            kit.until_sentence(stage, "15_11", *sentence)?;
            let y = 2.2 - i as f64 * 1.25;
            let m = if i == 3 {
                kit.text_t2c(stage, s, 24.0, Palette::ink(), &verdicts, v(0.0, y))?
            } else {
                let color = [Palette::ink(), Palette::teal(), Palette::red()][i];
                txt(kit, stage, s, 29.0, 0.0, y, color)?
            };
            show(stage, &[m], 0.3)?;
            law_rows.push(m);
        }
        kit.hold(stage)?;
        hide(stage, &law_rows, 0.3)?;
        kit.say(stage, "15_12")?;
        show(stage, &plain, 0.3)?;
        let mut letters = Vec::new();
        for (i, r) in ROADS.iter().enumerate() {
            let a = map.points[r.from];
            let b = map.points[r.to];
            let m = txt(
                kit,
                stage,
                &((b'a' + i as u8) as char).to_string(),
                27.0,
                (a[0] + b[0]) / 2.0 + 0.27,
                (a[1] + b[1]) / 2.0,
                Palette::gold(),
            )?;
            show(stage, &[m], 0.2)?;
            letters.push(m);
        }
        kit.until_sentence(stage, "15_12", 2)?;
        let mut stacks = Vec::new();
        for (i, collection) in sets.iter().enumerate().take(3) {
            let m = txt(
                kit,
                stage,
                &format!(
                    "{}: {}",
                    model::PLACES[i],
                    collection.iter().map(|&s| set_text(s)).collect::<Vec<_>>().join("  ")
                ),
                27.0,
                3.5,
                1.4 - i as f64 * 0.8,
                Palette::teal(),
            )?;
            show(stage, &[m], 0.3)?;
            stacks.push(m);
        }
        kit.until_sentence(stage, "15_12", 3)?;
        let precisely_sets = txt(
            kit,
            stage,
            "Precisely: combine is union, and extend joins each pair of sets.\nThe chance is worked out only at the end, outside the sweep.",
            24.0,
            0.0,
            3.0,
            Palette::dim(),
        )?;
        show(stage, &[precisely_sets], 0.3)?;
        kit.until_sentence(stage, "15_12", 4)?;
        let setlaw = txt(
            kit,
            stage,
            "Both orders: {x,y} and {x,z}",
            28.0,
            0.0,
            -2.6,
            Palette::teal(),
        )?;
        show(stage, &[setlaw], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "15_13")?;
        hide(stage, &stacks, 0.3)?;
        stacks.clear();
        hide(stage, &[setlaw, precisely_sets], 0.3)?;
        for (i, &s) in sets[3].iter().enumerate() {
            let m = txt(
                kit,
                stage,
                &set_text(s),
                31.0,
                3.5,
                1.5 - i as f64 * 0.65,
                route_color(i),
            )?;
            show(stage, &[m], 0.25)?;
            stacks.push(m);
        }
        let routes: Vec<u32> = sets[3].iter().copied().collect();
        let mut terms = Vec::new();
        for subset in 1u32..(1 << routes.len()) {
            let combined = routes
                .iter()
                .enumerate()
                .filter(|(i, _)| subset & (1 << i) != 0)
                .fold(0, |mask, (_, r)| mask | r);
            terms.push(format!(
                "{} {}",
                if subset.count_ones() % 2 == 1 { "+" } else { "−" },
                Chance::ROAD.pow(combined.count_ones()).about(5)
            ));
        }
        kit.until_sentence(stage, "15_13", 1)?;
        let terms_row = txt(kit, stage, &terms.join("  "), 21.0, 0.0, -1.7, Palette::ink())?;
        show(stage, &[terms_row], 0.4)?;
        let result = txt(
            kit,
            stage,
            &format!("Overlaps included: {}, about {}", truth.fraction(), truth.about(3)),
            30.0,
            0.0,
            -2.3,
            Palette::teal(),
        )?;
        show(stage, &[result], 0.3)?;
        kit.until_sentence(stage, "15_13", 2)?;
        // Road sets come from the set badge; routes come from the counting sweep.
        let routes_one = model::sweep::<Count>(4, &ROADS, 0).expect("fixed DAG").badges[3];
        let cost = txt(
            kit,
            stage,
            &format!(
                "Ten copies: {} road sets = {} routes",
                sets[3].len().pow(10),
                routes_one.pow(10)
            ),
            29.0,
            0.0,
            -3.0,
            Palette::gold(),
        )?;
        show(stage, &[cost], 0.3)?;
        kit.until_sentence(stage, "15_13", 3)?;
        let precisely_copies = txt(
            kit,
            stage,
            "Precisely: some maps have smaller exact methods;\nfor copies joined end to end, the chances multiply.",
            24.0,
            0.0,
            3.0,
            Palette::dim(),
        )?;
        show(stage, &[precisely_copies], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &plain, 0.3)?;
        hide(stage, &letters, 0.3)?;
        hide(stage, &stacks, 0.3)?;
        hide(stage, &[result, cost, terms_row, precisely_copies], 0.3)?;
        // Introduce the checklist with its question before showing any of its conditions.
        let bridge = txt(
            kit,
            stage,
            "When does the machine apply?",
            42.0,
            0.0,
            0.25,
            Palette::ink(),
        )?;
        stage.add_to_scene(bridge)?;
        kit.say(stage, "15_14")?;
        let mut checklist = Vec::new();
        for (i, (sentence, s)) in [
            (1, "A summary that keeps enough"),
            (1, "Two rules that obey the laws"),
            (2, "An order: incoming offers first"),
            (3, "Loops: a reason for finiteness, and another method"),
        ]
        .iter()
        .enumerate()
        {
            kit.until_sentence(stage, "15_14", *sentence)?;
            if i == 0 {
                stage.remove_from_scene(bridge);
            }
            let m = txt(kit, stage, s, 30.0, 0.0, 2.2 - i as f64 * 1.25, Palette::ink())?;
            show(stage, &[m], 0.3)?;
            checklist.push(m);
        }
        kit.hold(stage)?;
        hide(stage, &checklist, 0.3)?;
        kit.say(stage, "15_15")?;
        show(stage, &map.all(), 0.3)?;
        let recurrence = txt(
            kit,
            stage,
            "Badge = starting value ⊕ every (incoming badge ⊗ road)",
            27.0,
            0.0,
            3.0,
            Palette::ink(),
        )?;
        show(stage, &[recurrence], 0.3)?;
        let quickest = model::sweep::<Quickest>(4, &ROADS, 0).expect("fixed DAG").badges[3].unwrap();
        let count = model::sweep::<Count>(4, &ROADS, 0).expect("fixed DAG").badges[3];
        let reachable = model::sweep::<Reachable>(4, &ROADS, 0).expect("fixed DAG").badges[3];
        let Width::Finite(widest) = model::sweep::<Widest>(4, &ROADS, 0).expect("fixed DAG").badges[3] else {
            unreachable!("finite connected map")
        };
        let Best::Finite { time, count: ties } =
            model::sweep::<QuickestCount>(4, &ROADS, 0).expect("fixed DAG").badges[3]
        else {
            unreachable!("connected map")
        };
        // Each pair lands with its sentence and stays until the line ends.
        let jobs = [
            (2, format!("Quickest: {quickest}")),
            (2, format!("Routes: {count}")),
            (3, format!("Exists: {}", if reachable { "yes" } else { "no" })),
            (3, format!("Widest: {widest}")),
            (4, format!("Quickest and ties: ({time}, {ties})")),
            (4, format!("Dinner ready: {}", times[3])),
        ];
        let mut answers = Vec::new();
        for (k, (sentence, s)) in jobs.iter().enumerate() {
            kit.until_sentence(stage, "15_15", *sentence)?;
            let m = txt(kit, stage, s, 30.0, 3.6, 1.8 - k as f64 * 0.65, Palette::teal())?;
            show(stage, &[m], 0.3)?;
            answers.push(m);
        }
        kit.hold(stage)?;
        kit.say(stage, "15_16")?;
        hide(stage, &answers, 0.3)?;
        hide(stage, &[recurrence], 0.3)?;
        let ending = txt(
            kit,
            stage,
            "Different Problems, the Same Algorithm",
            38.0,
            0.0,
            3.0,
            Palette::ink(),
        )?;
        let qualifier = txt(
            kit,
            stage,
            "wherever these laws hold\nand the work follows a valid order",
            30.0,
            2.5,
            0.0,
            Palette::gold(),
        )?;
        show(stage, &[ending, qualifier], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &map.all(), 0.3)?;
        hide(stage, &[ending, qualifier], 0.3)?;
        let mut credits = Vec::new();
        for (i, s) in [
            "Sources",
            "Baril, Couceiro, Lagerkvist (2026)",
            "Meijer, Fokkinga, Paterson (1991) • Uustalu, Vene (1999)",
            "Kabanov, Vene (2006) • Hinze, Wu (2013)",
            "Mohri: Semiring frameworks and algorithms (2002)",
            "Related teaching: Tziavelis, Gatterbauer, Riedewald",
            "Toward Responsive DBMS, ICDE 2022, Part 5 slides",
        ]
        .iter()
        .enumerate()
        {
            let m = txt(
                kit,
                stage,
                s,
                if i == 0 { 34.0 } else { 25.0 },
                0.0,
                2.6 - i as f64 * 0.75,
                Palette::ink(),
            )?;
            credits.push(m);
        }
        show(stage, &credits, 0.5)?;
        stage.wait(8.0)?;
        kit.finish(stage)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schedule_respects_precedence_and_overlap() {
        let (b, s) = schedule();
        assert_eq!(b, [0, 2, 5, 9]);
        assert_eq!(s, [(0, 2), (0, 5), (2, 3), (2, 9), (5, 8)]);
        assert_eq!(s.iter().filter(|&&(a, b)| a <= 2 && b > 2).count(), 3);
    }
    #[test]
    fn event_sets_recover_shared_dependencies() {
        let sets = road_sets();
        assert_eq!(sets[3], Badge::from([9, 18, 21]));
        assert_eq!((0..32).filter(|&m| open(m, &sets[3])).count(), 15);
        assert_eq!(reliability_numerator(), 97119);
        assert!((patched_sweep()[3] - 0.977751).abs() < 1e-12);
        let a = Badge::from([1]);
        let b = Badge::from([2]);
        let c = Badge::from([4]);
        assert_eq!(extend(&a, &union(&b, &c)), union(&extend(&a, &b), &extend(&a, &c)));
    }
    #[test]
    fn exact_chances_match_the_research_fractions() {
        let sweep = patched_sweep_exact();
        assert_eq!(sweep[2].fraction(), "981/1000");
        assert_eq!(sweep[3].fraction(), "977751/1000000");
        assert_eq!(sweep[3].about(3), "0.978");
        let chances = route_chances();
        let fractions: Vec<String> = chances.iter().map(|c| c.fraction()).collect();
        assert_eq!(fractions, ["81/100", "81/100", "729/1000"]);
        let sum = chances.iter().copied().fold(Chance::ZERO, Chance::plus);
        assert_eq!(sum.fraction(), "2349/1000");
        assert_eq!(sum.about(3), "2.349");
        let listed = chances.iter().copied().fold(Chance::ZERO, Chance::either);
        assert_eq!(listed.fraction(), "9902169/10000000");
        assert_eq!(listed.about(3), "0.990");
        let (combine_first, extend_first) = law();
        assert_eq!(combine_first.fraction(), "891/1000");
        assert_eq!(extend_first.fraction(), "9639/10000");
        assert_eq!(extend_first.about(3), "0.964");
        let truth = Chance::new(u128::from(reliability_numerator()), ROADS.len() as u32);
        assert_eq!(truth.fraction(), "97119/100000");
        assert_eq!(truth.about(3), "0.971");
        assert_eq!(Chance::ROAD.fraction(), "9/10");
    }
    #[test]
    fn two_at_a_time_keeps_nine_but_moves_the_soup() {
        assert_eq!(JOBS[SOUP], "Simmer");
        assert_eq!(JOBS[ROAST], "Roast");
        assert_eq!(capacity(1).0, 18);
        assert_eq!(capacity(2), (9, BTreeSet::from([9])));
        let (best_three, soup_three) = capacity(3);
        assert_eq!(best_three, 9);
        assert!(soup_three.contains(&8));
    }
}
