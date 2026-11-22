//! One graph, three different questions. Every displayed offer comes from
//! the real generic traversal, including offers from a closed road.
use super::draw::{Map, ROUTES, hide, prediction, route_color, route_total, show};
use crate::model::{self, Arithmetic, Count, Offer, PLACES, ROADS, Reachable, Widest, Width};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter03 {
    pub kit: Kit,
}

fn position(i: usize) -> Vec3 {
    v(3.6, 1.7 - i as f64 * 0.8)
}
fn width(w: &Width) -> String {
    match w {
        Width::Unreachable => "no route".into(),
        Width::Unlimited => "no limit yet".into(),
        Width::Finite(n) => n.to_string(),
    }
}
fn yes(b: &bool) -> String {
    if *b { "yes" } else { "no" }.into()
}

fn replace(kit: &Kit, stage: &mut Stage<'_>, badges: &mut [Mob], values: &[String], color: Srgb) -> fmn::Result<()> {
    hide(stage, badges, 0.2)?;
    for (i, text) in values.iter().enumerate() {
        badges[i] = kit.text(stage, text, 28.0, color, position(i))?;
    }
    show(stage, badges, 0.25)
}

fn replay<V>(
    kit: &Kit,
    stage: &mut Stage<'_>,
    map: &Map,
    badges: &mut [Mob],
    offers: &[Offer<V>],
    format: fn(&V) -> String,
    color: Srgb,
) -> fmn::Result<()> {
    for event in offers {
        let to = ROADS[event.road_index].to;
        let incoming = kit.text(
            stage,
            &format!("offer: {}", format(&event.offered)),
            30.0,
            color,
            v(2.8, -1.8),
        )?;
        show(stage, &[incoming], 0.25)?;
        map.travel(stage, &[event.road_index], color)?;
        hide(stage, &[badges[to]], 0.15)?;
        badges[to] = kit.text(stage, &format(&event.combined), 28.0, color, position(to))?;
        show(stage, &[badges[to]], 0.2)?;
        hide(stage, &[incoming], 0.15)?;
    }
    Ok(())
}

impl SceneConstruct for Chapter03 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let counts = model::sweep::<Count>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let mut closed = ROADS;
        closed[0].open = false;
        let reach = model::sweep::<Reachable>(PLACES.len(), &closed, 0).expect("tested DAG");
        let wide = model::sweep::<Widest>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        show(stage, &map.all(), 0.5)?;
        kit.say(stage, "03_1")?;
        hide(stage, &map.costs, 0.4)?;
        let mut badges = Vec::new();
        let mut names = Vec::new();
        let mut frames = Vec::new();
        for (i, name) in PLACES.iter().enumerate() {
            let p = position(i);
            let frame = stage.arena_mut().add(
                Rectangle::new()
                    .width(2.25)
                    .height(0.62)
                    .style(Style::default().stroke(Palette::dim(), 1.5, 1.0))
                    .build()?
                    .moved_to(p),
            );
            let label = kit.text(stage, name, 27.0, Palette::ink(), v(1.15, p[1]))?;
            let badge = kit.text(stage, if i == 0 { "1" } else { "0" }, 28.0, Palette::teal(), p)?;
            show(stage, &[frame, label, badge], 0.25)?;
            frames.push(frame);
            names.push(label);
            badges.push(badge);
        }
        let count_rule = kit.text(
            stage,
            "Count routes\nextend: ×1   combine: +",
            29.0,
            Palette::teal(),
            v(0.0, 3.05),
        )?;
        show(stage, &[count_rule], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "03_2")?;
        // Each group of offers starts with the sentence that names it: Hill at
        // "Hill receives one", both Lake offers at "Lake receives one directly,
        // and another through Hill", so Lake reaches 2 by "Two routes reach Lake".
        for (offers, sentence) in [(0..1, 3), (1..3, 4)] {
            kit.until_sentence(stage, "03_2", sentence)?;
            replay(
                kit,
                stage,
                &map,
                &mut badges,
                &counts.offers[offers],
                |n| n.to_string(),
                Palette::teal(),
            )?;
        }
        kit.hold(stage)?;
        kit.say(stage, "03_3")?;
        // Both incoming contributions are available now. The narration
        // presents Lake's two routes first, then Hill's one; recompute the
        // displayed partial sum in that order rather than showing 3 early.
        let mut finish_offers: Vec<_> = counts
            .offers
            .iter()
            .filter(|event| ROADS[event.road_index].to == 3)
            .rev()
            .cloned()
            .collect();
        let mut total = Count::zero();
        for event in &mut finish_offers {
            event.previous = total;
            total = Count::combine(&total, &event.offered).expect("tiny checked route count");
            event.combined = total;
        }
        replay(
            kit,
            stage,
            &map,
            &mut badges,
            &finish_offers,
            |n| n.to_string(),
            Palette::teal(),
        )?;
        for (i, route) in ROUTES.iter().enumerate() {
            map.travel(stage, route, route_color(i))?;
        }
        kit.hold(stage)?;
        kit.say(stage, "03_4")?;
        let order = counts.order.iter().map(|&i| PLACES[i]).collect::<Vec<_>>().join(" → ");
        let order_label = kit.text(stage, &order, 27.0, Palette::ink(), v(2.6, -2.0))?;
        show(stage, &[order_label], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[order_label, count_rule], 0.3)?;
        kit.say(stage, "03_5")?;
        replace(
            kit,
            stage,
            &mut badges,
            &["yes".into(), "no".into(), "no".into(), "no".into()],
            route_color(1),
        )?;
        let reach_rule = kit.text(
            stage,
            "Reachable?\nextend: AND   combine: OR",
            29.0,
            route_color(1),
            v(0.0, 3.05),
        )?;
        show(stage, &[reach_rule], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "03_6")?;
        let barrier = kit.text(stage, "×", 55.0, route_color(0), v(-4.25, 0.775))?;
        let question = kit.text(
            stage,
            "Start → Hill is closed.\nCan we still reach Finish?",
            29.0,
            Palette::ink(),
            v(2.7, -1.85),
        )?;
        show(stage, &[barrier, question], 0.3)?;
        prediction(kit, stage, 4, v(2.7, -3.0))?;
        hide(stage, &[question], 0.2)?;
        kit.say(stage, "03_7")?;
        replay(kit, stage, &map, &mut badges, &reach.offers, yes, route_color(1))?;
        kit.hold(stage)?;
        hide(stage, &[barrier, reach_rule], 0.3)?;
        kit.say(stage, "03_8")?;
        show(stage, &map.costs, 0.3)?;
        hide(stage, &badges, 0.3)?;
        hide(stage, &names, 0.3)?;
        hide(stage, &frames, 0.3)?;
        let capacity_rule = kit.text(
            stage,
            "Width limit · toy units\nextend: min   combine: max",
            29.0,
            route_color(2),
            v(0.0, 3.05),
        )?;
        show(stage, &[capacity_rule], 0.3)?;
        // A rigid block cannot shrink through the narrower gap. This is a
        // diagram of capacity, not a claim about the drawn road's scale.
        let scale = 0.35;
        let gap = ROADS[2].value as f64 * scale;
        let vehicle = ROADS[0].value as f64 * scale;
        let mut gates = Vec::new();
        for direction in [-1.0, 1.0] {
            let rail = stage.arena_mut().add(
                Rectangle::new()
                    .width(1.0)
                    .height(0.10)
                    .style(Style::default().fill(Palette::dim(), 1.0))
                    .build()?
                    .moved_to(v(2.7 + direction * (gap / 2.0 + 0.5), 0.0)),
            );
            gates.push(rail);
        }
        let block = stage.arena_mut().add(
            Rectangle::new()
                .width(vehicle)
                .height(0.65)
                .style(Style::default().fill(route_color(1), 1.0))
                .build()?
                .moved_to(v(2.7, -1.7)),
        );
        let gate_label = kit.text(
            stage,
            &format!(
                "Hill → Lake: width {}\nVehicle width {}",
                ROADS[2].value, ROADS[0].value
            ),
            28.0,
            Palette::ink(),
            v(2.7, 1.0),
        )?;
        show(stage, &gates, 0.3)?;
        show(stage, &[block, gate_label], 0.3)?;
        stage.play(block.animate().move_to(v(2.7, -0.40), ORIGIN)?)?;
        kit.hold(stage)?;
        hide(stage, &gates, 0.3)?;
        hide(stage, &[block, gate_label], 0.3)?;
        kit.say(stage, "03_9")?;
        let question = kit.text(
            stage,
            "Which route fits the widest vehicle?",
            29.0,
            Palette::ink(),
            v(2.7, 1.5),
        )?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 5, v(2.7, 0.1))?;
        hide(stage, &[question], 0.3)?;
        kit.say(stage, "03_10")?;
        show(stage, &names, 0.3)?;
        show(stage, &frames, 0.3)?;
        replace(
            kit,
            stage,
            &mut badges,
            &[
                "no limit yet".into(),
                "no route".into(),
                "no route".into(),
                "no route".into(),
            ],
            route_color(2),
        )?;
        // "Hill gets two", "Lake receives five ... only one through Hill", then
        // both Finish offers at "Finish receives two through Hill and three
        // through Lake", so Finish shows 3 at "Keep three".
        for (offers, sentence) in [(0..1, 2), (1..3, 3), (3..5, 5)] {
            kit.until_sentence(stage, "03_10", sentence)?;
            replay(
                kit,
                stage,
                &map,
                &mut badges,
                &wide.offers[offers],
                width,
                route_color(2),
            )?;
        }
        kit.hold(stage)?;
        kit.say(stage, "03_11")?;
        let quick = model::sweep::<model::Quickest>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let contrast = kit.text(
            stage,
            &format!(
                "At Lake\nquickest: {} via Hill\nwidest: {} direct",
                quick.badges[2].unwrap(),
                width(&wide.badges[2])
            ),
            28.0,
            Palette::ink(),
            v(2.7, -2.1),
        )?;
        show(stage, &[contrast], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[contrast], 0.3)?;
        kit.say(stage, "03_12")?;
        // One persistent row per route, each at the sentence that names it, so
        // the closing comparison has all three results on screen.
        let bottleneck = |route: &[usize]| route.iter().map(|&e| ROADS[e].value).min().expect("nonempty route");
        let order = [(2usize, 1), (0, 2), (1, 3)];
        let row_y = |row: usize| -1.75 - row as f64 * 0.7;
        for (row, &(index, sentence)) in order.iter().enumerate() {
            let route = ROUTES[index];
            kit.until_sentence(stage, "03_12", sentence)?;
            let result = kit.text(
                stage,
                &format!(
                    "Width limit: {} · Travel time: {} minutes",
                    bottleneck(route),
                    route_total(route)
                ),
                27.0,
                route_color(index),
                v(2.7, row_y(row)),
            )?;
            show(stage, &[result], 0.3)?;
            map.travel(stage, route, route_color(index))?;
        }
        // "This time, the route through Lake wins": frame the widest route's row.
        let (winner_row, &(winner, _)) = order
            .iter()
            .enumerate()
            .max_by_key(|(_, (index, _))| bottleneck(ROUTES[*index]))
            .expect("three routes");
        kit.until_sentence(stage, "03_12", 4)?;
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(5.6)
                .height(0.58)
                .style(Style::default().stroke(route_color(winner), 1.5, 1.0))
                .build()?
                .moved_to(v(2.7, row_y(winner_row))),
        );
        show(stage, &[frame], 0.3)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}
