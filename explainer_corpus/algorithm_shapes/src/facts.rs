//! Computed facts for the graph and chapter-specific worked examples.
//! Export coverage does not replace a script/scene audit or audiovisual review.
use crate::model::{self, Best, Count, PLACES, Quickest, QuickestCount, ROADS, Reachable, Widest, Width};
use explainer_kit::Facts;

fn minutes(value: Option<u64>) -> String {
    value.map_or_else(|| "unreachable".into(), |n| n.to_string())
}

pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    let quick = model::sweep::<Quickest>(PLACES.len(), &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    let count = model::sweep::<Count>(PLACES.len(), &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    let wide = model::sweep::<Widest>(PLACES.len(), &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    let pairs = model::sweep::<QuickestCount>(PLACES.len(), &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    let layers = model::layers::<Quickest>(PLACES.len(), &ROADS, 0, 3).map_err(|e| format!("{e:?}"))?;
    facts.add("map.roads", ROADS.len())?;
    facts.add("map.routes", count.badges[3])?;
    facts.add(
        "map.order",
        quick.order.iter().map(|&i| PLACES[i]).collect::<Vec<_>>().join(","),
    )?;
    facts.add("copies10.roads", ROADS.len() * 10)?;
    facts.add(
        "copies10.routes",
        count.badges[3].checked_pow(10).ok_or("route count overflow")?,
    )?;
    for (i, road) in ROADS.iter().enumerate() {
        facts.add(&format!("road.{i}.value"), road.value)?;
    }
    for (i, place) in PLACES.iter().enumerate() {
        facts.add(&format!("quick.{place}"), minutes(quick.badges[i]))?;
        facts.add(&format!("count.{place}"), count.badges[i])?;
        let width = match wide.badges[i] {
            Width::Unreachable => "unreachable".to_owned(),
            Width::Unlimited => "unlimited".to_owned(),
            Width::Finite(n) => n.to_string(),
        };
        facts.add(&format!("wide.{place}"), width)?;
        let pair = match pairs.badges[i] {
            Best::Unreachable => "unreachable,0".to_owned(),
            Best::Finite { time, count } => format!("{time},{count}"),
        };
        facts.add(&format!("quick_count.{place}"), pair)?;
        for (roads, layer) in layers.iter().enumerate() {
            facts.add(&format!("exactly.{roads}.{place}"), minutes(layer[i]))?;
        }
    }
    for (mask, closure) in (0u32..1 << ROADS.len()).map(|mask| {
        let roads = std::array::from_fn::<_, 5, _>(|i| model::Road {
            open: mask & (1 << i) == 0,
            ..ROADS[i]
        });
        (mask, roads)
    }) {
        let reach = model::sweep::<Reachable>(PLACES.len(), &closure, 0).map_err(|e| format!("{e:?}"))?;
        facts.add(
            &format!("closure.{mask}.reachable"),
            reach
                .badges
                .iter()
                .map(|&b| if b { "yes" } else { "no" })
                .collect::<Vec<_>>()
                .join(","),
        )?;
    }
    crate::chapters::ch05::export_facts(&mut facts)?;
    crate::chapters::ch06::export_facts(&mut facts)?;
    crate::chapters::ch07::export_facts(&mut facts)?;
    crate::chapters::ch08::export_facts(&mut facts)?;
    crate::chapters::ch09::export_facts(&mut facts)?;
    crate::chapters::ch10::export_facts(&mut facts)?;
    crate::chapters::ch11::export_facts(&mut facts)?;
    crate::chapters::ch12::export_facts(&mut facts)?;
    crate::chapters::ch13::export_facts(&mut facts)?;
    crate::chapters::ch14::export_facts(&mut facts)?;
    crate::chapters::ch15::export_facts(&mut facts)?;
    let (project, schedule) = crate::chapters::ch15::schedule();
    let sets = crate::chapters::ch15::road_sets();
    let patched = crate::chapters::ch15::patched_sweep();
    for (i, place) in PLACES.iter().enumerate() {
        facts.add(&format!("project.{place}"), project[i])?;
        facts.add(
            &format!("rel.sets.{place}"),
            sets[i].iter().map(u32::to_string).collect::<Vec<_>>().join(","),
        )?;
        facts.add(&format!("rel.patched.{place}"), format!("{:.6}", patched[i]))?;
    }
    for (i, &(start, end)) in schedule.iter().enumerate() {
        facts.add(&format!("project.job.{i}"), format!("{start},{end}"))?;
    }
    let overlap = (0..project[3])
        .map(|t| schedule.iter().filter(|&&(start, end)| start <= t && t < end).count())
        .max()
        .unwrap_or(0);
    facts.add("project.max.overlap", overlap)?;
    facts.add("project.one.at.a.time", ROADS.iter().map(|r| r.value).sum::<u64>())?;
    facts.add("rel.truth.numerator", crate::chapters::ch15::reliability_numerator())?;
    facts.add("rel.truth.denominator", 10u64.pow(ROADS.len() as u32))?;
    facts.add(
        "rel.successful.patterns",
        (0u32..1 << ROADS.len())
            .filter(|&state| sets[3].iter().any(|&route| state & route == route))
            .count(),
    )?;
    facts.add("rel.ten.copies.sets", sets[3].len().pow(10))?;
    let routes: Vec<_> = sets[3].iter().copied().collect();
    for subset in 1u32..1 << routes.len() {
        let mask = routes
            .iter()
            .enumerate()
            .filter(|(i, _)| subset & (1 << i) != 0)
            .fold(0, |acc, (_, mask)| acc | mask);
        facts.add(
            &format!("rel.overlap.term.{subset}"),
            format!(
                "{},{},{:.5}",
                if subset.count_ones() % 2 == 1 { "+" } else { "-" },
                mask,
                0.9f64.powi(mask.count_ones() as i32)
            ),
        )?;
    }
    Ok(facts)
}
