//! One loop, three different convergence questions; exact dyadic crowd shares.
use super::draw::{Map, dot, hide, prediction, route_color, show};
use crate::model::{self, Count, Error, Quickest, ROADS, Road};
use explainer_kit::{Kit, Palette, narration, v};
use fmn::prelude::*;

pub struct Chapter08 {
    pub kit: Kit,
}
fn roads() -> Vec<Road> {
    let mut r = ROADS.to_vec();
    r.push(Road {
        from: 2,
        to: 1,
        value: 4,
        open: true,
    });
    r
}
fn simple_routes() -> Vec<Vec<usize>> {
    fn visit(at: usize, seen: u8, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>, r: &[Road]) {
        if at == 3 {
            out.push(path.clone());
            return;
        }
        for (i, road) in r.iter().enumerate().filter(|(_, r)| r.from == at) {
            if seen & (1 << road.to) == 0 {
                path.push(i);
                visit(road.to, seen | (1 << road.to), path, out, r);
                path.pop();
            }
        }
    }
    let mut out = Vec::new();
    visit(0, 1, &mut Vec::new(), &mut out, &roads());
    out
}
/// Numerator over 2^budget from exact walk counts, reduced for display.
fn crowd_share(budget: usize) -> (u64, u64) {
    let count = model::layers::<Count>(4, &roads(), 0, budget).unwrap();
    let mut n = count
        .iter()
        .enumerate()
        .map(|(k, row)| row[3] << (budget - k))
        .sum::<u64>();
    let mut d = 1u64 << budget;
    while d > 1 && n % 2 == 0 {
        n /= 2;
        d /= 2;
    }
    (n, d)
}
fn fraction((n, d): (u64, u64)) -> String {
    if d == 1 { n.to_string() } else { format!("{n}/{d}") }
}
/// The toy payout of the road back in 08_13: its cost is minus this many units.
const PAYOUT: u64 = 3;
/// Road-label offsets for this chapter. The return arc passes through the shared anchors of the
/// two Start roads (draw.rs), so their labels move outward here only.
const CLEAR_OF_ARC: [[f64; 2]; 5] = [[-0.2, 0.65], [-0.2, -0.65], [0.3, 0.0], [0.2, 0.3], [0.2, -0.3]];
fn road_anchor(map: &Map, i: usize) -> Vec3 {
    let a = map.points[ROADS[i].from];
    let b = map.points[ROADS[i].to];
    v(
        (a[0] + b[0]) * 0.5 + CLEAR_OF_ARC[i][0],
        (a[1] + b[1]) * 0.5 + CLEAR_OF_ARC[i][1],
    )
}
fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}
/// A typeset minus sign, matching the payout label.
fn signed(n: i64) -> String {
    if n < 0 {
        format!("−{}", n.unsigned_abs())
    } else {
        n.to_string()
    }
}
/// Every loop number the chapter shows beyond the walk tables, computed once from the model so
/// the scene and its exported facts cannot drift apart.
struct LoopNumbers {
    /// Minutes on the added road back, Lake to Hill.
    added: u64,
    /// Minutes for one lap, Hill to Lake and back.
    lap: u64,
    /// Quickest badges with the loop present.
    hill: u64,
    lake: u64,
    /// The loop's offers: Lake's badge into Hill, and Hill's badge into Lake.
    hill_offer: u64,
    lake_offer: u64,
    /// Finish's quickest time once it settles.
    best: u64,
    /// Each road's share of the crowd, by road index (the road back is last).
    shares: Vec<(u64, u64)>,
    /// Start to Hill to Finish: the product of its two shares.
    quarter: (u64, u64),
    /// One lap when the road back pays out PAYOUT units.
    lap_total: i64,
    /// The cheapest cost after 0, 1, 2, 3 and 4 extra laps.
    laps: [i64; 5],
}
fn loop_numbers() -> LoopNumbers {
    let r = roads();
    let back = r[ROADS.len()];
    let forward = *r
        .iter()
        .find(|road| road.from == back.to && road.to == back.from)
        .expect("the road back reverses Hill to Lake");
    let quick = model::layers::<Quickest>(4, &r, 0, 6).unwrap();
    let best_at = |place: usize| quick.iter().filter_map(|row| row[place]).min().unwrap();
    let (hill, lake, best) = (best_at(1), best_at(2), best_at(3));
    let share = |road: &Road| (1u64, r.iter().filter(|other| other.from == road.from).count() as u64);
    let shares: Vec<_> = r.iter().map(share).collect();
    let (a, b) = (shares[0], shares[3]);
    let quarter = (a.0 * b.0, a.1 * b.1);
    let lap_total = forward.value as i64 - PAYOUT as i64;
    LoopNumbers {
        added: back.value,
        lap: forward.value + back.value,
        hill,
        lake,
        hill_offer: lake + back.value,
        lake_offer: hill + forward.value,
        best,
        shares,
        quarter,
        lap_total,
        laps: std::array::from_fn(|n| best as i64 + n as i64 * lap_total),
    }
}
/// "Then seven eighths, " is 20 of the 44 characters of 08_11's third sentence, so "then fifteen
/// sixteenths" begins about this far into it.
const FIFTEEN_SIXTEENTHS_AT: f64 = 20.0 / 44.0;
/// Wait until `fraction` of the way from sentence `j` of line `id` (said at `line_start`) to its
/// next sentence; nothing if that moment has passed, or without sentence starts (no narrator).
fn until_within_sentence(
    kit: &Kit,
    stage: &mut Stage<'_>,
    id: &str,
    j: usize,
    line_start: f64,
    fraction: f64,
) -> fmn::Result<()> {
    let starts = kit.sentence_starts(id);
    let (Some(a), Some(b)) = (starts.get(j), starts.get(j + 1)) else {
        return Ok(());
    };
    let at = line_start + a + (b - a) * fraction;
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        narration::wait_frames(stage, narration::frames_covering(at - now, fps))?;
    }
    Ok(())
}
fn panel(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 29.0, color, v(2.8, y))
}
fn lap(stage: &mut Stage<'_>, map: &Map, color: Srgb) -> fmn::Result<()> {
    let token = dot(stage, map.points[1], 0.14, color);
    show(stage, &[token], 0.15)?;
    stage.play(
        token
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.65),
                ..AnimateArgs::default()
            })?
            .move_to(map.points[2], ORIGIN)?,
    )?;
    stage.play(
        token
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.95),
                path_arc: Some(-std::f64::consts::PI),
                ..AnimateArgs::default()
            })?
            .move_to(map.points[1], ORIGIN)?,
    )?;
    hide(stage, &[token], 0.15)
}
impl SceneConstruct for Chapter08 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let loop_roads = roads();
        let quick = model::layers::<Quickest>(4, &loop_roads, 0, 6).unwrap();
        let counts = model::layers::<Count>(4, &loop_roads, 0, 6).unwrap();
        assert_eq!(model::sweep::<Quickest>(4, &loop_roads, 0), Err(Error::Cycle));
        let nums = loop_numbers();
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        // The labels are still detached here, so moving them shows nothing.
        for i in [0, 1] {
            stage.move_to(map.costs[i], road_anchor(&map, i), ORIGIN);
        }
        kit.say(stage, "08_1")?;
        show(stage, &map.all(), 0.4)?;
        let back = stage.arena_mut().add(curved_arrow(
            map.points[2],
            map.points[1],
            -std::f64::consts::PI,
            Style::default().stroke(Palette::gold(), 2.5, 1.0),
        )?);
        show(stage, &[back], 0.6)?;
        let added = kit.text(
            stage,
            &format!("{} · added", nums.added),
            28.0,
            Palette::gold(),
            v(-5.35, 0.65),
        )?;
        show(stage, &[added], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "08_2")?;
        let circular = panel(
            kit,
            stage,
            "Hill waits for Lake\nLake waits for Hill\nNo forward-only order",
            0.4,
            Palette::ink(),
        )?;
        show(stage, &[circular], 0.4)?;
        lap(stage, &map, Palette::gold())?;
        lap(stage, &map, Palette::gold())?;
        kit.hold(stage)?;
        hide(stage, &[circular], 0.3)?;
        kit.say(stage, "08_3")?;
        let question = panel(kit, stage, "Does the quickest time change?", 0.5, Palette::ink())?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 4, v(2.8, -1.3))?;
        hide(stage, &[question], 0.3)?;
        kit.say(stage, "08_4")?;
        let total = panel(
            kit,
            stage,
            &format!("Each lap adds {} minutes", nums.lap),
            2.3,
            Palette::gold(),
        )?;
        show(stage, &[total], 0.3)?;
        let mut labels = Vec::new();
        for k in 1..=5 {
            let best = quick[..=k].iter().filter_map(|row| row[3]).min();
            let s = format!(
                "Within {}: {}",
                counted(k, "road", "roads"),
                best.map_or_else(|| "no route".into(), |x| x.to_string())
            );
            let row = panel(kit, stage, &s, 1.4 - (k - 1) as f64 * 0.7, Palette::gold())?;
            show(stage, &[row], 0.3)?;
            labels.push(row);
        }
        kit.hold(stage)?;
        hide(stage, &labels, 0.3)?;
        hide(stage, &[total], 0.3)?;
        kit.say(stage, "08_5")?;
        let title = panel(kit, stage, "Walks may revisit a place", 2.3, Palette::ink())?;
        show(stage, &[title], 0.3)?;
        let mut readout = None;
        for k in 1..=6 {
            // Sentences 2, 3 and 4 speak the counts within 2, 3 and 4 roads; sentence 5 the growth.
            if (2..=5).contains(&k) {
                kit.until_sentence(stage, "08_5", k)?;
            }
            if let Some(old) = readout {
                hide(stage, &[old], 0.15)?;
            }
            let total: u64 = counts[..=k].iter().map(|row| row[3]).sum();
            let label = panel(
                kit,
                stage,
                &format!("At most {}\n{total} walks", counted(k, "road", "roads")),
                0.4,
                Palette::teal(),
            )?;
            show(stage, &[label], 0.3)?;
            readout = Some(label);
            if k > 2 {
                lap(stage, &map, Palette::teal())?;
            }
        }
        kit.hold(stage)?;
        hide(stage, &[title, readout.unwrap()], 0.3)?;
        kit.say(stage, "08_6")?;
        let paths = simple_routes();
        let mut strips = Vec::new();
        for (i, path) in paths.iter().enumerate() {
            let mut names = vec!["Start"];
            for &edge in path {
                names.push(crate::model::PLACES[loop_roads[edge].to]);
            }
            let label = kit.text(
                stage,
                &names.join(" → "),
                23.0,
                Palette::ink(),
                v(2.7, 1.7 - i as f64 * 0.8),
            )?;
            show(stage, &[label], 0.3)?;
            strips.push(label);
        }
        let distinct = panel(
            kit,
            stage,
            &format!("No repeated places: {}\nWalks allowing repeats: unbounded", paths.len()),
            -2.2,
            Palette::teal(),
        )?;
        show(stage, &[distinct], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &strips, 0.3)?;
        hide(stage, &[distinct], 0.3)?;
        kit.say(stage, "08_7")?;
        let equations = panel(kit, stage, "H = 1 + L\nL = 1 + H", 1.0, Palette::ink())?;
        show(stage, &[equations], 0.4)?;
        let contradiction = panel(
            kit,
            stage,
            "H = 2 + H\nNo finite count solves this",
            -1.0,
            route_color(0),
        )?;
        show(stage, &[contradiction], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[equations, contradiction], 0.3)?;
        kit.say(stage, "08_8")?;
        let (hill, lake) = (nums.hill, nums.lake);
        let fixed = panel(
            kit,
            stage,
            &format!(
                "H = min({}, L + {})\nL = min({}, H + {})\nH = {hill}    L = {lake}",
                ROADS[0].value, loop_roads[5].value, ROADS[1].value, ROADS[2].value
            ),
            0.7,
            Palette::gold(),
        )?;
        show(stage, &[fixed], 0.4)?;
        let checked = panel(
            kit,
            stage,
            &format!(
                "min({}, {}) = {hill}\nmin({}, {}) = {lake}",
                ROADS[0].value, nums.hill_offer, ROADS[1].value, nums.lake_offer
            ),
            -1.4,
            Palette::teal(),
        )?;
        show(stage, &[checked], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[fixed, checked], 0.3)?;
        kit.say(stage, "08_9")?;
        hide(stage, &map.costs, 0.3)?;
        hide(stage, &[added], 0.3)?;
        let share_title = panel(
            kit,
            stage,
            "Share of the crowd\nHalf takes each road out",
            2.0,
            Palette::teal(),
        )?;
        show(stage, &[share_title], 0.3)?;
        let mut shares = Vec::new();
        for (i, share) in nums.shares[..ROADS.len()].iter().enumerate() {
            let l = kit.text(stage, &fraction(*share), 26.0, Palette::teal(), road_anchor(&map, i))?;
            show(stage, &[l], 0.15)?;
            shares.push(l);
        }
        let half_back = kit.text(
            stage,
            &fraction(nums.shares[ROADS.len()]),
            26.0,
            Palette::teal(),
            v(-5.3, 0.65),
        )?;
        show(stage, &[half_back], 0.15)?;
        shares.push(half_back);
        let mut crowd = Vec::new();
        let mut places = [0usize; 32];
        for _ in 0..places.len() {
            crowd.push(dot(stage, map.points[0], 0.035, Palette::teal()));
        }
        show(stage, &crowd, 0.3)?;
        for _ in 0..2 {
            let mut seen = [0usize; 4];
            let mut arrived = [0usize; 4];
            let mut animations = Vec::new();
            for (i, place) in places.iter_mut().enumerate() {
                let outgoing: Vec<_> = loop_roads.iter().filter(|r| r.from == *place).collect();
                if !outgoing.is_empty() {
                    let old = *place;
                    *place = outgoing[seen[old] % outgoing.len()].to;
                    seen[old] += 1;
                }
                let q = arrived[*place];
                arrived[*place] += 1;
                let p = map.points[*place];
                let target = v(p[0] + (q % 4) as f64 * 0.07 - 0.1, p[1] + (q / 4) as f64 * 0.07 - 0.1);
                animations.push(
                    stage.prepare(
                        crowd[i]
                            .animate()
                            .set_anim_args(AnimateArgs {
                                run_time: Some(0.8),
                                ..AnimateArgs::default()
                            })?
                            .move_to(target, ORIGIN)?,
                    )?,
                );
            }
            stage.play_prepared(animations)?;
        }
        kit.hold(stage)?;
        kit.say(stage, "08_10")?;
        let quarter = panel(
            kit,
            stage,
            &format!(
                "{} × {} = {}",
                fraction(nums.shares[0]),
                fraction(nums.shares[3]),
                fraction(nums.quarter)
            ),
            0.4,
            Palette::teal(),
        )?;
        show(stage, &[quarter], 0.3)?;
        map.travel(stage, &[0, 3], Palette::teal())?;
        kit.hold(stage)?;
        hide(stage, &crowd, 0.3)?;
        hide(stage, &[quarter, share_title], 0.3)?;
        kit.say(stage, "08_11")?;
        // `say` returns at the line's first frame.
        let line_start = stage.scene().time().to_f64();
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(5.6)
                .height(0.6)
                .style(Style::default().stroke(Palette::ink(), 1.5, 1.0))
                .build()?
                .moved_to(v(2.8, 0.0)),
        );
        show(stage, &[frame], 0.3)?;
        let one = panel(kit, stage, "Arrived share · target 1", 1.8, Palette::ink())?;
        show(stage, &[one], 0.3)?;
        let mut last = None;
        for k in 2..=5 {
            // Each share lands with its words: sentence 0 says a half, 1 three quarters, and 2
            // seven eighths and then fifteen sixteenths.
            match k {
                3 | 4 => kit.until_sentence(stage, "08_11", k - 2)?,
                5 => until_within_sentence(kit, stage, "08_11", 2, line_start, FIFTEEN_SIXTEENTHS_AT)?,
                _ => {}
            }
            if let Some((bar, label)) = last {
                hide(stage, &[bar, label], 0.2)?;
            }
            let share = crowd_share(k);
            let width = 5.6 * share.0 as f64 / share.1 as f64;
            let bar = stage.arena_mut().add(
                Rectangle::new()
                    .width(width)
                    .height(0.52)
                    .style(Style::default().fill(Palette::teal(), 1.0))
                    .build()?
                    .moved_to(v(width / 2.0, 0.0)),
            );
            let label = panel(
                kit,
                stage,
                &format!("Within {k} roads: {}", fraction(share)),
                -1.0,
                Palette::teal(),
            )?;
            show(stage, &[bar, label], 0.5)?;
            last = Some((bar, label));
        }
        kit.hold(stage)?;
        kit.say(stage, "08_12")?;
        let (bar, label) = last.unwrap();
        hide(stage, &[bar, label], 0.3)?;
        let full = stage.arena_mut().add(
            Rectangle::new()
                .width(5.6)
                .height(0.52)
                .style(Style::default().fill(Palette::teal(), 1.0))
                .build()?
                .moved_to(v(2.8, 0.0)),
        );
        let limit = panel(kit, stage, "Limit: 1\nNot a finite sweep", -1.2, Palette::teal())?;
        show(stage, &[full, limit], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[frame, one, full, limit], 0.3)?;
        hide(stage, &shares, 0.3)?;
        kit.say(stage, "08_13")?;
        show(stage, &map.costs, 0.3)?;
        let paid = kit.text(stage, &signed(-(PAYOUT as i64)), 28.0, route_color(0), v(-5.3, 0.65))?;
        show(stage, &[paid], 0.3)?;
        let toy = panel(
            kit,
            stage,
            "Cost (toy units)\nEach lap pays out more than it costs",
            2.0,
            route_color(0),
        )?;
        show(stage, &[toy], 0.3)?;
        // Sentence 2 says a lap makes the trip cheaper: one lap shows it. Sentence 3 counts the costs
        // down, so they step at about the pace they are spoken.
        kit.until_sentence(stage, "08_13", 2)?;
        lap(stage, &map, route_color(0))?;
        kit.until_sentence(stage, "08_13", 3)?;
        let mut cost_label = None;
        for (laps, cost) in nums.laps.iter().enumerate() {
            if let Some(old) = cost_label {
                hide(stage, &[old], 0.15)?;
            }
            let label = panel(
                kit,
                stage,
                &format!("{}\nCost: {}", counted(laps, "extra lap", "extra laps"), signed(*cost)),
                0.0,
                route_color(0),
            )?;
            show(stage, &[label], 0.3)?;
            cost_label = Some(label);
            stage.wait(0.2)?;
        }
        kit.hold(stage)?;
        hide(stage, &[paid, toy, cost_label.unwrap()], 0.3)?;
        show(stage, &[added], 0.3)?;
        kit.say(stage, "08_14")?;
        let recap = panel(
            kit,
            stage,
            &format!(
                "Minutes: settles at {}\nWalk count: grows without bound\nCrowd share: approaches 1",
                nums.best
            ),
            0.6,
            Palette::ink(),
        )?;
        show(stage, &[recap], 0.4)?;
        kit.hold(stage)?;
        kit.say(stage, "08_15")?;
        let condition = panel(
            kit,
            stage,
            "Rules + order of work\nConvergence needs an argument",
            -1.8,
            Palette::teal(),
        )?;
        show(stage, &[condition], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[back, added], 0.4)?;
        kit.finish(stage)
    }
}

pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    facts.add("loop.simple.routes", simple_routes().len())?;
    let quick = model::layers::<Quickest>(4, &roads(), 0, 8).map_err(|e| format!("{e:?}"))?;
    let counts = model::layers::<Count>(4, &roads(), 0, 8).map_err(|e| format!("{e:?}"))?;
    for budget in 1..=8 {
        let best = quick[..=budget].iter().filter_map(|row| row[3]).min();
        facts.add(
            &format!("loop.quick.within.{budget}"),
            best.map_or_else(|| "unreachable".into(), |n| n.to_string()),
        )?;
        facts.add(
            &format!("loop.count.within.{budget}"),
            counts[..=budget].iter().map(|row| row[3]).sum::<u64>(),
        )?;
        facts.add(&format!("loop.crowd.within.{budget}"), fraction(crowd_share(budget)))?;
    }
    let n = loop_numbers();
    facts.add("loop.added.minutes", n.added)?;
    facts.add("loop.lap.minutes", n.lap)?;
    facts.add("loop.eq.quick.Hill", n.hill)?;
    facts.add("loop.eq.quick.Lake", n.lake)?;
    facts.add("loop.eq.quick.offer.Hill", n.hill_offer)?;
    facts.add("loop.eq.quick.offer.Lake", n.lake_offer)?;
    for (i, share) in n.shares.iter().enumerate() {
        facts.add(&format!("loop.crowd.share.{i}"), fraction(*share))?;
    }
    facts.add("loop.crowd.quarter", fraction(n.quarter))?;
    facts.add("loop.negative.payout", -(PAYOUT as i64))?;
    facts.add("loop.negative.lap.total", n.lap_total)?;
    for (laps, cost) in n.laps.iter().enumerate() {
        facts.add(&format!("loop.negative.laps.{laps}"), cost)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loop_claims_distinguish_finite_layers_simple_routes_and_limits() {
        let r = roads();
        assert_eq!(model::sweep::<Quickest>(4, &r, 0), Err(Error::Cycle));
        assert_eq!(simple_routes().len(), 4);
        let q = model::layers::<Quickest>(4, &r, 0, 8).unwrap();
        let c = model::layers::<Count>(4, &r, 0, 8).unwrap();
        for k in 2..=8 {
            assert_eq!(c[..=k].iter().map(|row| row[3]).sum::<u64>(), 2 * (k as u64 - 1));
            assert_eq!(
                q[..=k].iter().filter_map(|row| row[3]).min(),
                Some(if k == 2 { 8 } else { 6 })
            );
            assert_eq!(crowd_share(k), ((1u64 << (k - 1)) - 1, 1u64 << (k - 1)));
        }
        assert_eq!(crowd_share(0), (0, 1));
        assert_eq!(crowd_share(1), (0, 1));
        let n = loop_numbers();
        assert_eq!((n.added, n.lap, n.best), (4, 5, 6));
        assert_eq!((n.hill, n.lake, n.hill_offer, n.lake_offer), (2, 3, 7, 3));
        assert_eq!(n.shares, [(1, 2); 6]);
        assert_eq!(n.quarter, (1, 4));
        assert_eq!(n.lap_total, -2);
        assert_eq!(n.laps, [6, 4, 2, 0, -2]);
        assert_eq!(signed(-2), "−2");
        assert_eq!(counted(1, "road", "roads"), "1 road");
    }
}
