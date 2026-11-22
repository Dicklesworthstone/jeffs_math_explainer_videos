//! The computed offer trace drives the picture; the road-budget example then
//! demonstrates exactly which information the single badge lost.
use super::draw::{Map, ROUTES, hide, prediction, route_color, route_total, show};
use crate::model::{self, Offer, PLACES, Quickest, ROADS};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter02 {
    pub kit: Kit,
}

fn value(n: Option<u64>) -> String {
    n.map_or_else(|| "no route yet".to_owned(), |x| x.to_string())
}

fn badge_position(index: usize) -> Vec3 {
    v(3.8, 1.7 - index as f64 * 0.8)
}

fn panel(kit: &Kit, stage: &mut Stage<'_>, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, 28.0, Palette::ink(), v(2.7, -2.15))
}

/// Whether an offer is made for the first time or replayed afterwards.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    /// The badge takes the combined value.
    Live,
    /// The badges already hold their final values and are left alone; a first
    /// arrival shows only its extend line, so no stale combine is displayed.
    Replay,
}

fn offer(
    kit: &Kit,
    stage: &mut Stage<'_>,
    map: &Map,
    event: &Offer<Option<u64>>,
    badges: &mut [Mob],
    shown: &mut Vec<Mob>,
    pass: Pass,
) -> fmn::Result<()> {
    if shown.is_empty() && pass == Pass::Replay {
        // Preserve replay timing without fading an already hidden formula.
        stage.wait(0.2)?;
    } else {
        hide(stage, shown, 0.2)?;
    }
    let road = ROADS[event.road_index];
    let extend = format!(
        "extend: {} + {} = {}",
        value(event.source),
        value(event.edge),
        value(event.offered)
    );
    let label = if pass == Pass::Replay && event.previous.is_none() {
        extend
    } else {
        format!(
            "{extend}\ncombine: min({}, {}) = {}",
            value(event.previous),
            value(event.offered),
            value(event.combined)
        )
    };
    let explanation = panel(kit, stage, &label)?;
    show(stage, &[explanation], 0.25)?;
    map.travel(stage, &[event.road_index], Palette::gold())?;
    if pass == Pass::Live {
        hide(stage, &[badges[road.to]], 0.15)?;
        badges[road.to] = kit.text(
            stage,
            &value(event.combined),
            29.0,
            Palette::gold(),
            badge_position(road.to),
        )?;
        show(stage, &[badges[road.to]], 0.2)?;
    }
    *shown = vec![explanation];
    Ok(())
}

impl SceneConstruct for Chapter02 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let trace =
            model::sweep::<Quickest>(PLACES.len(), &ROADS, 0).expect("the film's tested fixed graph is acyclic");
        let layers = model::layers::<Quickest>(PLACES.len(), &ROADS, 0, 2)
            .expect("the film's tested fixed graph has valid endpoints");
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        show(stage, &map.all(), 0.5)?;
        let mut badges = Vec::new();
        kit.say(stage, "02_1")?;
        for (i, place) in PLACES.iter().enumerate() {
            let p = badge_position(i);
            let frame = stage.arena_mut().add(
                Rectangle::new()
                    .width(2.25)
                    .height(0.62)
                    .style(Style::default().stroke(Palette::dim(), 1.5, 1.0))
                    .build()?
                    .moved_to(p),
            );
            let name = kit.text(stage, place, 27.0, Palette::ink(), v(1.35, p[1]))?;
            let initial = if i == 0 { Some(0) } else { None };
            let text = kit.text(stage, &value(initial), 29.0, Palette::gold(), p)?;
            show(stage, &[frame, name, text], 0.25)?;
            badges.push(text);
        }
        kit.hold(stage)?;
        let mut shown = Vec::new();
        kit.say(stage, "02_2")?;
        // Hill at "Hill gets a two", the direct road at "Lake first receives a
        // five", and the road through Hill at "The two minutes we already
        // spent, plus one more" (sentence 2 only announces the possibility).
        for (event, sentence) in trace.offers[..3].iter().zip([0, 1, 3]) {
            kit.until_sentence(stage, "02_2", sentence)?;
            offer(kit, stage, &map, event, &mut badges, &mut shown, Pass::Live)?;
        }
        kit.hold(stage)?;
        kit.say(stage, "02_3")?;
        for (i, event) in trace.offers[3..].iter().enumerate() {
            kit.until_sentence(stage, "02_3", i + 1)?;
            offer(kit, stage, &map, event, &mut badges, &mut shown, Pass::Live)?;
        }
        kit.hold(stage)?;
        hide(stage, &shown, 0.2)?;
        shown.clear();
        kit.say(stage, "02_4")?;
        let order = trace.order.iter().map(|&i| PLACES[i]).collect::<Vec<_>>().join(" → ");
        let schedule = kit.text(stage, &order, 28.0, Palette::teal(), v(0.0, 3.05))?;
        show(stage, &[schedule], 0.3)?;
        for event in &trace.offers {
            map.travel(stage, &[event.road_index], Palette::teal())?;
        }
        let comparisons = panel(
            kit,
            stage,
            &format!(
                "{} road contributions\n{} possible routes\nContributions ≠ total running time",
                ROADS.len() * 10,
                (ROUTES.len() as u64).pow(10)
            ),
        )?;
        show(stage, &[comparisons], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[comparisons, schedule], 0.25)?;
        kit.say(stage, "02_5")?;
        // Replaying offers does not recompute a different traversal, and the
        // badges keep their final values (Lake stays 3 beside Finish 6). The
        // direct road illustrates "we extend", the road through Hill "we combine".
        let lake_offers = trace.offers.iter().filter(|e| ROADS[e.road_index].to == 2);
        for (event, sentence) in lake_offers.zip([1, 2]) {
            kit.until_sentence(stage, "02_5", sentence)?;
            offer(kit, stage, &map, event, &mut badges, &mut shown, Pass::Replay)?;
        }
        kit.hold(stage)?;
        hide(stage, &shown, 0.2)?;
        kit.say(stage, "02_6")?;
        let mut winning = vec![None; PLACES.len()];
        for event in &trace.offers {
            if event.combined != event.previous {
                winning[ROADS[event.road_index].to] = Some(event.road_index);
            }
        }
        let mut backward = Vec::new();
        let mut cursor = PLACES.len() - 1;
        while let Some(edge) = winning[cursor] {
            backward.push(edge);
            cursor = ROADS[edge].from;
        }
        let witness = backward
            .iter()
            .map(|&e| format!("{} ← {}", PLACES[ROADS[e].to], PLACES[ROADS[e].from]))
            .collect::<Vec<_>>()
            .join("\n");
        let witness_label = panel(kit, stage, &format!("For programmers: remember the winner\n{witness}"))?;
        show(stage, &[witness_label], 0.3)?;
        backward.reverse();
        map.travel(stage, &backward, route_color(2))?;
        kit.hold(stage)?;
        hide(stage, &[witness_label], 0.2)?;
        kit.say(stage, "02_7")?;
        let arrivals = [layers[1][2].unwrap(), layers[2][2].unwrap()];
        let remaining = ROADS[4].value;
        let preservation = panel(
            kit,
            stage,
            &format!(
                "Same remaining road: +{remaining}\n{} + {remaining} = {}\n{} + {remaining} = {}",
                arrivals[0],
                arrivals[0] + remaining,
                arrivals[1],
                arrivals[1] + remaining
            ),
        )?;
        show(stage, &[preservation], 0.3)?;
        map.travel(stage, &[4], route_color(1))?;
        map.travel(stage, &[4], route_color(2))?;
        kit.hold(stage)?;
        hide(stage, &[preservation], 0.2)?;
        kit.say(stage, "02_8")?;
        let clock = kit.text(
            stage,
            "Arrival time?\nWould the roads change?",
            27.0,
            Palette::gold(),
            v(-3.2, -2.85),
        )?;
        show(stage, &[clock], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[clock], 0.2)?;
        kit.say(stage, "02_9")?;
        hide(stage, &[badges[3]], 0.2)?;
        let unknown = kit.text(stage, "?", 29.0, Palette::gold(), badge_position(3))?;
        show(stage, &[unknown], 0.2)?;
        let question = panel(kit, stage, "At most two roads\nWhich allowed route is quickest?")?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 5, v(2.7, -3.1))?;
        hide(stage, &[question], 0.2)?;
        kit.say(stage, "02_10")?;
        map.travel(stage, ROUTES[1], route_color(1))?;
        hide(stage, &[unknown], 0.15)?;
        let allowed = kit.text(
            stage,
            &minutes_for_budget(&layers),
            29.0,
            Palette::teal(),
            badge_position(3),
        )?;
        show(stage, &[allowed], 0.2)?;
        let budget = panel(
            kit,
            stage,
            &format!(
                "Allowed: {} minutes, {} roads\nRejected: {} minutes, {} roads",
                layers[2][3].unwrap(),
                ROUTES[1].len(),
                route_total(ROUTES[2]),
                ROUTES[2].len()
            ),
        )?;
        show(stage, &[budget], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[budget, badges[2]], 0.2)?;
        kit.say(stage, "02_11")?;
        let state_label = kit.text(stage, "two states", 27.0, Palette::gold(), badge_position(2))?;
        show(stage, &[state_label], 0.2)?;
        let mut split = Vec::new();
        for (row, roads) in [1usize, 2].into_iter().enumerate() {
            let p = v(2.7, -1.7 - row as f64 * 0.95);
            let color = route_color(roads);
            let frame = stage.arena_mut().add(
                Rectangle::new()
                    .width(5.5)
                    .height(0.85)
                    .style(Style::default().stroke(color, 1.5, 1.0))
                    .build()?
                    .moved_to(p),
            );
            let outcome = if roads == 1 {
                format!("Finish: {}", layers[2][3].unwrap())
            } else {
                "no roads left".to_owned()
            };
            let text = kit.text(
                stage,
                &format!(
                    "Lake · {roads} road{} · {} min\n→ {outcome}",
                    if roads == 1 { "" } else { "s" },
                    layers[roads][2].unwrap()
                ),
                26.0,
                color,
                p,
            )?;
            show(stage, &[frame, text], 0.3)?;
            split.extend([frame, text]);
        }
        map.travel(stage, &[4], route_color(1))?;
        kit.hold(stage)?;
        hide(stage, &split, 0.3)?;
        hide(stage, &[state_label, allowed], 0.2)?;
        let restored = panel(kit, stage, "Returning to unrestricted routes")?;
        show(stage, &[restored, badges[2], badges[3]], 0.25)?;
        stage.wait(1.0)?;
        hide(stage, &[restored], 0.2)?;
        kit.finish(stage)
    }
}

fn minutes_for_budget(layers: &[Vec<Option<u64>>]) -> String {
    layers
        .iter()
        .take(3)
        .filter_map(|layer| layer[3])
        .min()
        .expect("the tested map has a route using at most two roads")
        .to_string()
}
