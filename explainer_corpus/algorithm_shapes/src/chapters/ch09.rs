//! Backward questions become a tree, then shared states regain their identity.
use super::draw::{Map, dot, hide, prediction, route_color, show};
use crate::model::{self, PLACES, Quickest, ROADS};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter09 {
    pub kit: Kit,
}
#[derive(Clone)]
struct Question {
    place: usize,
    parent: Option<usize>,
    road: Option<usize>,
    point: Vec3,
}
fn questions() -> Vec<Question> {
    // A reverse DFS supplies structure and edge identity; positions only lay it out.
    fn grow(place: usize, parent: Option<usize>, road: Option<usize>, out: &mut Vec<Question>) {
        let index = out.len();
        out.push(Question {
            place,
            parent,
            road,
            point: ORIGIN,
        });
        for (edge, r) in ROADS.iter().enumerate().filter(|(_, r)| r.to == place) {
            grow(r.from, Some(index), Some(edge), out);
        }
    }
    let mut out = Vec::new();
    grow(3, None, None, &mut out);
    let positions = [
        v(-3.0, 2.5),
        v(-5.1, 1.0),
        v(-5.1, -0.6),
        v(-1.8, 1.0),
        v(-3.0, -0.6),
        v(-0.7, -0.6),
        v(-0.7, -2.2),
    ];
    assert_eq!(out.len(), positions.len());
    for (q, p) in out.iter_mut().zip(positions) {
        q.point = p;
    }
    out
}
fn answer(tree: &[Question], index: usize) -> u64 {
    if tree[index].place == 0 {
        return 0;
    }
    tree.iter()
        .enumerate()
        .filter(|(_, q)| q.parent == Some(index))
        .map(|(i, q)| answer(tree, i) + ROADS[q.road.unwrap()].value)
        .min()
        .unwrap()
}
fn call_count(copies: u32) -> u64 {
    (0..copies).fold(1, |n, _| 3 * n + 4)
}
fn seed_prefix(mut seed: i64, cap: usize) -> (Vec<i64>, bool) {
    let mut out = Vec::new();
    for _ in 0..cap {
        if seed == 0 {
            return (out, true);
        }
        out.push(seed);
        seed -= 2;
    }
    (out, seed == 0)
}
fn label(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 27.0, color, v(3.2, y))
}
impl SceneConstruct for Chapter09 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let tree = questions();
        let map = Map::build(kit, stage, [4.85, 1.5], 0.5, true)?;
        let mut nodes = Vec::new();
        let mut names = Vec::new();
        let mut branches = Vec::new();
        let mut costs = Vec::new();
        kit.say(stage, "09_1")?;
        show(stage, &map.all(), 0.4)?;
        for q in &tree {
            nodes.push(dot(stage, q.point, 0.12, route_color(1)));
            names.push(kit.text(
                stage,
                &format!("{}?", PLACES[q.place]),
                25.0,
                Palette::ink(),
                // Child names stay beside the node, clear of incoming arrows.
                if q.parent.is_some() {
                    v(q.point[0] - 0.65, q.point[1])
                } else {
                    v(q.point[0], q.point[1] + 0.35)
                },
            )?);
        }
        show(stage, &[nodes[0], names[0]], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "09_2")?;
        for (i, q) in tree.iter().enumerate().skip(1) {
            let parent = tree[q.parent.unwrap()].point;
            let edge = stage.arena_mut().add(
                Arrow::new(parent, q.point)
                    .buff(0.2)
                    .thickness(1.6)
                    .style(
                        Style::default()
                            .fill(Palette::dim(), 1.0)
                            .stroke(Palette::dim(), 1.0, 1.0),
                    )
                    .build()?,
            );
            let cost = kit.text(
                stage,
                &ROADS[q.road.unwrap()].value.to_string(),
                23.0,
                Palette::gold(),
                v((parent[0] + q.point[0]) / 2.0 + 0.2, (parent[1] + q.point[1]) / 2.0),
            )?;
            show(stage, &[edge, cost, nodes[i], names[i]], 0.4)?;
            branches.push(edge);
            costs.push(cost);
        }
        kit.hold(stage)?;
        kit.say(stage, "09_3")?;
        let unfold = label(kit, stage, "Unfold\nAnamorphism", -0.3, route_color(1))?;
        show(stage, &[unfold], 0.3)?;
        for i in [0usize, 1, 3, 2, 4, 5, 6] {
            let halo = stage.arena_mut().add(
                Circle::new()
                    .radius(0.24)
                    .style(Style::default().stroke(route_color(1), 2.0, 1.0))
                    .build()
                    .moved_to(tree[i].point),
            );
            show(stage, &[halo], 0.15)?;
            hide(stage, &[halo], 0.15)?;
        }
        kit.hold(stage)?;
        hide(stage, &[unfold], 0.2)?;
        kit.say(stage, "09_4")?;
        let fold = label(kit, stage, "Fold: answers return", -0.3, Palette::gold())?;
        show(stage, &[fold], 0.3)?;
        let mut values = Vec::new();
        for (i, q) in tree.iter().enumerate().rev() {
            let value = kit.text(
                stage,
                &answer(&tree, i).to_string(),
                27.0,
                Palette::gold(),
                v(q.point[0] + 0.4, q.point[1]),
            )?;
            show(stage, &[value], 0.25)?;
            values.push(value);
            if let Some(parent) = q.parent {
                let token = dot(stage, q.point, 0.08, Palette::gold());
                show(stage, &[token], 0.05)?;
                stage.play(
                    token
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.45),
                            ..Default::default()
                        })?
                        .move_to(tree[parent].point, ORIGIN)?,
                )?;
                hide(stage, &[token], 0.05)?;
            }
        }
        kit.hold(stage)?;
        hide(stage, &[fold], 0.2)?;
        kit.say(stage, "09_5")?;
        let hylo = label(kit, stage, "Unfold → Fold\nHylomorphism", -0.3, Palette::ink())?;
        let work = label(
            kit,
            stage,
            "Shape of the work\nNo stored tree required",
            -1.8,
            Palette::dim(),
        )?;
        show(stage, &[hylo, work], 0.3)?;
        // A moving marker follows call entry/return without claiming stored data.
        let cursor = dot(stage, tree[0].point, 0.08, route_color(0));
        show(stage, &[cursor], 0.1)?;
        for i in [1, 2, 1, 0, 3, 4, 3, 5, 6, 5, 3, 0] {
            stage.play(
                cursor
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.18),
                        ..Default::default()
                    })?
                    .move_to(tree[i].point, ORIGIN)?,
            )?;
        }
        hide(stage, &[cursor], 0.1)?;
        kit.hold(stage)?;
        hide(stage, &[hylo, work], 0.3)?;
        kit.say(stage, "09_6")?;
        let ask = label(kit, stage, "How many Hill questions?", -0.4, Palette::ink())?;
        show(stage, &[ask], 0.3)?;
        prediction(kit, stage, 4, v(3.0, -1.4))?;
        hide(stage, &[ask], 0.2)?;
        kit.say(stage, "09_7")?;
        let mut highlights = Vec::new();
        for q in tree.iter().filter(|q| q.place == 1) {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.24)
                    .style(Style::default().stroke(Palette::ink(), 2.0, 1.0))
                    .build()
                    .moved_to(q.point),
            );
            show(stage, &[ring], 0.25)?;
            highlights.push(ring);
        }
        for (q, color) in tree
            .iter()
            .filter(|q| q.place == 0)
            .zip([route_color(0), route_color(1), route_color(2)])
        {
            let leaf = dot(stage, q.point, 0.14, color);
            show(stage, &[leaf], 0.25)?;
            highlights.push(leaf);
        }
        let count = label(
            kit,
            stage,
            &format!(
                "Hill: {} questions\nStart: {} route leaves",
                tree.iter().filter(|q| q.place == 1).count(),
                tree.iter().filter(|q| q.place == 0).count()
            ),
            -0.4,
            Palette::teal(),
        )?;
        show(stage, &[count], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[count], 0.2)?;
        kit.say(stage, "09_8")?;
        let mut counters = Vec::new();
        for n in 1u32..=10 {
            hide(stage, &counters, 0.08)?;
            counters = vec![label(
                kit,
                stage,
                &format!(
                    "{n} map copies\n{} questions\n{} Start leaves\n{} shared places",
                    call_count(n),
                    3u64.pow(n),
                    3 * n + 1
                ),
                -0.7,
                Palette::gold(),
            )?];
            show(stage, &counters, 0.18)?;
        }
        kit.hold(stage)?;
        hide(stage, &counters, 0.2)?;
        kit.say(stage, "09_9")?;
        // The duplicated tree stays up while sentences 0-1 say nothing is shared yet.
        kit.until_sentence(stage, "09_9", 2)?;
        let unshared: Vec<Mob> = branches
            .iter()
            .chain(&costs)
            .chain(&values)
            .chain(&highlights)
            .copied()
            .chain(map.all())
            .collect();
        hide(stage, &unshared, 0.3)?;
        let merged = Map::build(kit, stage, [3.2, 0.0], 1.0, true)?;
        for (i, q) in tree.iter().enumerate() {
            let target = merged.points[q.place];
            let a = stage.prepare(
                nodes[i]
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.8),
                        ..Default::default()
                    })?
                    .move_to(target, ORIGIN)?,
            )?;
            let b = stage.prepare(
                names[i]
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.8),
                        ..Default::default()
                    })?
                    .move_to(v(target[0], target[1] + 0.35), ORIGIN)?,
            )?;
            stage.play_prepared(vec![a, b])?;
        }
        hide(stage, &nodes, 0.2)?;
        hide(stage, &names, 0.2)?;
        show(stage, &merged.all(), 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "09_10")?;
        let quick = model::sweep::<Quickest>(4, &ROADS, 0).expect("acyclic map");
        let mut badges = Vec::new();
        for &place in &quick.order {
            let p = merged.points[place];
            let halo = dot(stage, p, 0.14, Palette::teal());
            show(stage, &[halo], 0.2)?;
            let value = kit.text(
                stage,
                &quick.badges[place].unwrap().to_string(),
                29.0,
                Palette::teal(),
                match place {
                    0 => v(p[0] - 0.45, p[1] - 0.35),
                    1 => v(p[0] + 0.5, p[1] + 0.25),
                    2 => v(p[0] + 0.65, p[1] - 0.35),
                    _ => v(p[0] + 0.4, p[1] - 0.35),
                },
            )?;
            show(stage, &[value], 0.3)?;
            badges.push(value);
            hide(stage, &[halo], 0.1)?;
        }
        let dp = kit.text(
            stage,
            "Shared questions + safe order\nDynamic programming",
            29.0,
            Palette::ink(),
            v(0.0, -2.8),
        )?;
        show(stage, &[dp], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "09_11")?;
        hide(stage, &[dp], 0.3)?;
        hide(stage, &badges, 0.2)?;
        hide(stage, &merged.all(), 0.4)?;
        let layers = model::layers::<Quickest>(4, &ROADS, 0, 1).unwrap();
        let unrestricted = kit.text(
            stage,
            &format!("Lake\nBest time: {}", quick.badges[2].unwrap()),
            31.0,
            Palette::teal(),
            v(-3.0, 0.5),
        )?;
        let limited = kit.text(
            stage,
            &format!("Lake, at most one road\nBest time: {}", layers[1][2].unwrap()),
            31.0,
            Palette::gold(),
            v(2.3, 0.5),
        )?;
        let distinct = kit.text(
            stage,
            "Different questions: keep separate",
            28.0,
            Palette::ink(),
            v(0.0, -1.2),
        )?;
        show(stage, &[unrestricted, limited, distinct], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[unrestricted, limited, distinct], 0.3)?;
        kit.say(stage, "09_12")?;
        let stream = kit.text(stage, "Readings → readings → …", 27.0, Palette::dim(), v(0.0, 2.7))?;
        show(stage, &[stream], 0.3)?;
        kit.until_sentence(stage, "09_12", 1)?;
        hide(stage, &[stream], 0.3)?;
        let rule = kit.text(
            stage,
            "Integers: n → n − 2\nStop when n = 0",
            30.0,
            Palette::ink(),
            v(0.0, 2.4),
        )?;
        show(stage, &[rule], 0.3)?;
        kit.until_sentence(stage, "09_12", 3)?;
        for (seed, y, color) in [(6, 0.6, Palette::teal()), (7, -1.3, route_color(0))] {
            let (prefix, stops) = seed_prefix(seed, 5);
            let items = prefix
                .iter()
                .map(ToString::to_string)
                .chain(std::iter::once(if stops { "stop".into() } else { "…".into() }))
                .collect::<Vec<_>>();
            for (j, item) in items.iter().enumerate() {
                let x = -5.0 + j as f64 * 1.85;
                if j > 0 {
                    let arrow = stage.arena_mut().add(
                        Arrow::new(v(x - 1.45, y), v(x - 0.4, y))
                            .buff(0.05)
                            .thickness(1.5)
                            .build()?,
                    );
                    show(stage, &[arrow], 0.15)?;
                }
                let number = kit.text(stage, item, 32.0, color, v(x, y))?;
                show(stage, &[number], 0.3)?;
            }
            if seed == 6 {
                kit.until_sentence(stage, "09_12", 4)?;
            }
        }
        let parity = kit.text(
            stage,
            "Odd − even stays odd: zero is never reached",
            25.0,
            route_color(0),
            v(0.0, -2.8),
        )?;
        show(stage, &[parity], 0.3)?;
        kit.finish(stage)
    }
}
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let tree = questions();
    facts.add("backward.questions", tree.len())?;
    for (place, name) in PLACES.iter().enumerate() {
        facts.add(
            &format!("backward.asked.{name}"),
            tree.iter().filter(|q| q.place == place).count(),
        )?;
    }
    for (i, question) in tree.iter().enumerate() {
        facts.add(
            &format!("backward.node.{i}"),
            format!("{},{}", PLACES[question.place], answer(&tree, i)),
        )?;
    }
    for copies in 0..=10 {
        facts.add(&format!("backward.calls.{copies}"), call_count(copies))?;
    }
    for seed in [6, 7] {
        let (prefix, stopped) = seed_prefix(seed, 5);
        facts.add(
            &format!("unfold.{seed}.prefix"),
            prefix.iter().map(ToString::to_string).collect::<Vec<_>>().join(","),
        )?;
        facts.add(&format!("unfold.{seed}.stopped"), stopped)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_questions_preserve_route_and_state_counts() {
        let q = questions();
        assert_eq!(q.len(), 7);
        assert_eq!(q.iter().filter(|q| q.place == 0).count(), 3);
        assert_eq!(q.iter().filter(|q| q.place == 1).count(), 2);
        assert_eq!(answer(&q, 0), 6);
        assert_eq!(call_count(10), 177145);
        for n in 0..=10 {
            assert_eq!(call_count(n), 3u64.pow(n + 1) - 2);
        }
        assert_eq!(seed_prefix(6, 5), (vec![6, 4, 2], true));
        assert_eq!(seed_prefix(7, 5), (vec![7, 5, 3, 1, -1], false));
        for n in -100..100i64 {
            if n % 2 != 0 {
                assert_ne!(n - 2, 0);
            }
        }
    }
}
