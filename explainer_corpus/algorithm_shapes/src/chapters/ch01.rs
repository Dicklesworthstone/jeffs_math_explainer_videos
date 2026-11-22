//! Opening: solve the small route puzzle, then see enumeration grow.
//! All six narration cues and the authored five-second prediction remain.
use super::draw::{Map, ROUTES, hide, prediction, route_color, route_total, show};
use crate::model::ROADS;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter01 {
    pub kit: Kit,
}

impl SceneConstruct for Chapter01 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        let units = kit.text(stage, "One-way roads · minutes", 28.0, Palette::dim(), v(-3.2, 2.75))?;
        kit.say(stage, "01_1")?;
        show(stage, &[map.nodes[0], map.nodes[3], map.names[0], map.names[3]], 0.5)?;
        show(stage, &[map.nodes[1], map.nodes[2], map.names[1], map.names[2]], 0.5)?;
        for (road, cost) in map.roads.iter().zip(&map.costs) {
            show(stage, &[*road, *cost], 0.3)?;
        }
        show(stage, &[units], 0.3)?;
        kit.hold(stage)?;

        kit.say(stage, "01_2")?;
        let question = kit.text(
            stage,
            "Which route gets there first?",
            32.0,
            Palette::ink(),
            v(3.1, 1.6),
        )?;
        show(stage, &[question], 0.4)?;
        prediction(kit, stage, 5, v(3.1, 0.2))?;
        hide(stage, &[question], 0.3)?;

        kit.say(stage, "01_3")?;
        let mut strips = Vec::new();
        // Each new route is revealed at its spoken sentence, not at cue start.
        for (i, route) in ROUTES.iter().enumerate() {
            kit.until_sentence(stage, "01_3", [0, 2, 4][i])?;
            map.travel(stage, route, route_color(i))?;
            let terms = route
                .iter()
                .map(|&e| ROADS[e].value.to_string())
                .collect::<Vec<_>>()
                .join(" + ");
            let label = format!("{terms} = {} minutes", route_total(route));
            let strip = kit.text(stage, &label, 32.0, route_color(i), v(3.1, 1.25 - i as f64 * 0.9))?;
            show(stage, &[strip], 0.35)?;
            strips.push(strip);
        }
        kit.hold(stage)?;
        kit.say(stage, "01_4")?;
        map.travel(stage, ROUTES[2], route_color(2))?;
        kit.hold(stage)?;

        kit.say(stage, "01_5")?;
        // The one map stays while sentence 0 names it; the join starts at "Now join".
        kit.until_sentence(stage, "01_5", 1)?;
        hide(stage, &strips, 0.4)?;
        hide(stage, &map.all(), 0.5)?;
        hide(stage, &[units], 0.3)?;
        // Shared endpoints: every miniature Finish is the next Start. Ten
        // copies fit on one row; labels stay in the large readable counters.
        let mut copies = Vec::new();
        let scale = 0.25;
        // Short fades so all ten copies are in before sentence 3 says the total.
        let fade = 0.12;
        let mut counters = Vec::new();
        let base = ROUTES.len() as u64;
        for k in 1u32..=10 {
            let shift = [-5.7 + 5.3 * scale + (k - 1) as f64 * 4.2 * scale, 0.45];
            let copy = Map::build(kit, stage, shift, scale, false)?;
            show(stage, &copy.all(), fade)?;
            copies.extend(copy.all());
            if !counters.is_empty() {
                hide(stage, &counters, 0.1)?;
            }
            counters = vec![
                kit.text(
                    stage,
                    &format!("Routes: {base}^{k} = {}", base.pow(k)),
                    36.0,
                    Palette::teal(),
                    v(0.0, -1.05),
                )?,
                kit.text(
                    stage,
                    &format!("Roads: {} × {k} = {}", ROADS.len(), ROADS.len() as u32 * k),
                    36.0,
                    Palette::gold(),
                    v(0.0, -1.85),
                )?,
            ];
            show(stage, &counters, fade)?;
        }
        kit.hold(stage)?;
        hide(stage, &copies, 0.4)?;
        hide(stage, &counters, 0.3)?;
        show(stage, &map.all(), 0.5)?;
        show(stage, &strips, 0.4)?;
        kit.say(stage, "01_6")?;
        kit.hold(stage)?;
        let title = kit.text(
            stage,
            "Different Problems,\nthe Same Algorithm",
            44.0,
            Palette::ink(),
            v(2.7, 2.35),
        )?;
        show(stage, &[title], 0.5)?;
        stage.wait(3.0)?;
        kit.finish(stage)
    }
}
