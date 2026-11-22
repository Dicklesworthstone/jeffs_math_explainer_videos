//! The structure stays fixed while its leaf and branch algebras change.
use super::draw::{Map, dot, hide, prediction, route_color, show};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter10 {
    pub kit: Kit,
}
#[derive(Clone)]
enum Tree {
    Leaf(i64),
    Branch(Box<Tree>, Box<Tree>),
}
fn example() -> Tree {
    Tree::Branch(
        Box::new(Tree::Branch(Box::new(Tree::Leaf(3)), Box::new(Tree::Leaf(1)))),
        Box::new(Tree::Leaf(5)),
    )
}
fn fold<T>(tree: &Tree, leaf: &impl Fn(i64) -> T, branch: &impl Fn(T, T) -> T) -> T {
    match tree {
        Tree::Leaf(n) => leaf(*n),
        Tree::Branch(a, b) => branch(fold(a, leaf, branch), fold(b, leaf, branch)),
    }
}
const POINTS: [Vec3; 5] = [
    [-3.0, 2.0, 0.0],
    [-4.3, 0.55, 0.0],
    [-1.5, 0.55, 0.0],
    [-5.3, -1.2, 0.0],
    [-3.4, -1.2, 0.0],
];
const PARENTS: [(usize, usize); 4] = [(1, 0), (2, 0), (3, 1), (4, 1)];
fn field(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64) -> fmn::Result<Mob> {
    kit.text(stage, s, 28.0, Palette::ink(), v(3.0, y))
}
struct Rule {
    fields: Vec<Mob>,
}
impl Rule {
    fn set(&mut self, kit: &Kit, stage: &mut Stage<'_>, leaf: &str, branch: &str) -> fmn::Result<()> {
        hide(stage, &self.fields, 0.2)?;
        self.fields = vec![
            field(kit, stage, &format!("Leaf: {leaf}"), 1.7)?,
            field(kit, stage, &format!("Branch: {branch}"), 0.7)?,
        ];
        show(stage, &self.fields, 0.3)
    }
}
/// `pace` scales a nominal 3.85 s; every step rounds up to whole frames: 0.6 takes 2.57 s at 30 fps, 2.42 s at 60.
fn pass(kit: &Kit, stage: &mut Stage<'_>, values: &[String; 5], color: Srgb, pace: f64) -> fmn::Result<Vec<Mob>> {
    let mut labels = Vec::new();
    for i in [3usize, 4, 1, 2, 0] {
        let p = POINTS[i];
        let s = kit.text(stage, &values[i], 28.0, color, v(p[0], p[1] - 0.37))?;
        show(stage, &[s], 0.25 * pace)?;
        labels.push(s);
        if let Some(&(_, parent)) = PARENTS.iter().find(|&&(child, _)| child == i) {
            let token = dot(stage, p, 0.08, color);
            show(stage, &[token], 0.1 * pace)?;
            stage.play(
                token
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.45 * pace),
                        ..Default::default()
                    })?
                    .move_to(POINTS[parent], ORIGIN)?,
            )?;
            hide(stage, &[token], 0.1 * pace)?;
        }
    }
    Ok(labels)
}
fn numeric_values(leaf: impl Fn(i64) -> i64, branch: impl Fn(i64, i64) -> i64) -> [String; 5] {
    let t = example();
    let Tree::Branch(left, right) = &t else { unreachable!() };
    [
        fold(&t, &leaf, &branch),
        fold(left, &leaf, &branch),
        fold(right, &leaf, &branch),
        leaf(3),
        leaf(1),
    ]
    .map(|n| n.to_string())
}
impl SceneConstruct for Chapter10 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let mut tree_mobs = Vec::new();
        kit.say(stage, "10_1")?;
        for (child, parent) in PARENTS {
            let line = stage.arena_mut().add(
                Line::new(POINTS[parent], POINTS[child])
                    .style(Style::default().stroke(Palette::dim(), 2.0, 1.0))
                    .build()?,
            );
            show(stage, &[line], 0.2)?;
            tree_mobs.push(line);
        }
        for (i, p) in POINTS.iter().enumerate() {
            let node = dot(stage, *p, 0.11, Palette::ink());
            show(stage, &[node], 0.15)?;
            tree_mobs.push(node);
            if let Some(n) = match i {
                2 => Some(5),
                3 => Some(3),
                4 => Some(1),
                _ => None,
            } {
                let name = kit.text(stage, &n.to_string(), 29.0, Palette::ink(), v(p[0], p[1] + 0.35))?;
                show(stage, &[name], 0.2)?;
                tree_mobs.push(name);
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "10_2")?;
        let frame = stage.arena_mut().add(
            Rectangle::new()
                .width(5.5)
                .height(2.4)
                .style(Style::default().stroke(Palette::dim(), 1.5, 1.0))
                .build()?
                .moved_to(v(3.0, 1.25)),
        );
        show(stage, &[frame], 0.3)?;
        let mut rule = Rule { fields: Vec::new() };
        rule.set(kit, stage, "?", "?")?;
        kit.hold(stage)?;
        kit.say(stage, "10_3")?;
        rule.set(kit, stage, "n", "a + b")?;
        let mut answers = pass(kit, stage, &numeric_values(|n| n, |a, b| a + b), Palette::gold(), 1.0)?;
        kit.hold(stage)?;
        kit.say(stage, "10_4")?;
        hide(stage, &answers, 0.3)?;
        rule.set(kit, stage, "1", "a + b")?;
        let ask = field(kit, stage, "What reaches the root?", -0.6)?;
        show(stage, &[ask], 0.3)?;
        prediction(kit, stage, 4, v(3.0, -1.7))?;
        hide(stage, &[ask], 0.2)?;
        kit.say(stage, "10_5")?;
        // Half pace so the leaf count ends before sentence 2 names the next rule.
        answers = pass(kit, stage, &numeric_values(|_| 1, |a, b| a + b), Palette::teal(), 0.5)?;
        kit.until_sentence(stage, "10_5", 2)?;
        hide(stage, &answers, 0.2)?;
        rule.set(kit, stage, "n", "max(a, b)")?;
        answers = pass(kit, stage, &numeric_values(|n| n, i64::max), route_color(1), 0.6)?;
        kit.until_sentence(stage, "10_5", 4)?;
        hide(stage, &answers, 0.2)?;
        rule.set(kit, stage, "write n", "write (a + b)")?;
        let printed = fold(&example(), &|n| n.to_string(), &|a, b| format!("({a} + {b})"));
        let expression = kit.text(stage, &printed, 37.0, route_color(2), v(-3.3, -2.5))?;
        show(stage, &[expression], 0.4)?;
        kit.hold(stage)?;
        kit.say(stage, "10_6")?;
        hide(stage, &rule.fields, 0.3)?;
        hide(stage, &[frame], 0.3)?;
        let summaries = [
            ("Sum", fold(&example(), &|n| n, &|a, b| a + b).to_string()),
            ("Leaves", fold(&example(), &|_| 1, &|a, b| a + b).to_string()),
            ("Largest", fold(&example(), &|n| n, &i64::max).to_string()),
            ("Expression", printed),
        ];
        let mut cards = Vec::new();
        for (i, (name, result)) in summaries.iter().enumerate() {
            let text = field(kit, stage, &format!("{name}: {result}"), 2.3 - i as f64 * 0.8)?;
            show(stage, &[text], 0.25)?;
            cards.push(text);
        }
        let relation = kit.text(
            stage,
            "One structure, different rules",
            28.0,
            Palette::teal(),
            v(0.0, -3.3),
        )?;
        show(stage, &[relation], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "10_7")?;
        let name = field(kit, stage, "Fold / catamorphism", -1.7)?;
        show(stage, &[name], 0.3)?;
        for i in [1usize, 0] {
            let children = PARENTS
                .iter()
                .filter(|&&(_, p)| p == i)
                .map(|&(c, _)| c)
                .collect::<Vec<_>>();
            let mut rings = Vec::new();
            for j in std::iter::once(i).chain(children) {
                let ring = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.26)
                        .style(Style::default().stroke(Palette::gold(), 2.0, 1.0))
                        .build()
                        .moved_to(POINTS[j]),
                );
                show(stage, &[ring], 0.2)?;
                rings.push(ring);
            }
            stage.wait(0.6)?;
            hide(stage, &rings, 0.3)?;
        }
        kit.hold(stage)?;
        kit.say(stage, "10_8")?;
        hide(stage, &tree_mobs, 0.4)?;
        hide(stage, &cards, 0.3)?;
        hide(stage, &[expression, relation, name], 0.3)?;
        let map = Map::build(kit, stage, [0.0, 0.0], 0.8, true)?;
        show(stage, &map.all(), 0.4)?;
        // Two visibly separate Hill questions grow from one shared map state.
        let tree_positions = [
            v(3.2, 2.3),
            v(1.5, 0.8),
            v(4.4, 0.8),
            v(1.5, -0.8),
            v(3.4, -0.8),
            v(5.4, -0.8),
            v(5.4, -2.3),
        ];
        let places = [3usize, 1, 2, 0, 0, 1, 0];
        let parents = [(1, 0), (2, 0), (3, 1), (4, 2), (5, 2), (6, 5)];
        let mut copies = Vec::new();
        for (i, &place) in places.iter().enumerate() {
            let copy = dot(
                stage,
                map.points[place],
                0.1,
                if place == 1 { Palette::gold() } else { Palette::ink() },
            );
            show(stage, &[copy], 0.1)?;
            stage.play(
                copy.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.4),
                        ..Default::default()
                    })?
                    .move_to(tree_positions[i], ORIGIN)?,
            )?;
            let name = kit.text(
                stage,
                crate::model::PLACES[place],
                23.0,
                Palette::ink(),
                if i == 0 {
                    v(tree_positions[i][0], tree_positions[i][1] + 0.3)
                } else {
                    v(tree_positions[i][0] - 0.55, tree_positions[i][1])
                },
            )?;
            show(stage, &[name], 0.2)?;
            copies.push(copy);
        }
        for (child, parent) in parents {
            let line = stage.arena_mut().add(
                Line::new(tree_positions[parent], tree_positions[child])
                    .style(Style::default().stroke(Palette::dim(), 1.2, 1.0))
                    .build()?,
            );
            show(stage, &[line], 0.15)?;
        }
        let limit = kit.text(
            stage,
            "Shared Hill on the map → repeated Hill in the tree",
            27.0,
            Palette::gold(),
            v(0.0, -3.25),
        )?;
        show(stage, &[limit], 0.3)?;
        for (i, &place) in places.iter().enumerate().rev() {
            let color = if place == 0 { route_color(2) } else { Palette::teal() };
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.2)
                    .style(Style::default().stroke(color, 2.0, 1.0))
                    .build()
                    .moved_to(tree_positions[i]),
            );
            show(stage, &[ring], 0.2)?;
            hide(stage, &[ring], 0.2)?;
        }
        kit.finish(stage)
    }
}
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    for (name, values) in [
        ("sum", numeric_values(|n| n, |a, b| a + b)),
        ("leaves", numeric_values(|_| 1, |a, b| a + b)),
        ("largest", numeric_values(|n| n, i64::max)),
    ] {
        for (i, value) in values.iter().enumerate() {
            facts.add(&format!("tree.{name}.{i}"), value)?;
        }
    }
    facts.add(
        "tree.printed",
        fold(&example(), &|n| n.to_string(), &|a, b| format!("({a} + {b})")),
    )?;
    facts.add("tree.anybig", fold(&example(), &|n| n > 4, &|a, b| a || b))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_tree_supports_distinct_algebras() {
        let t = example();
        assert_eq!(fold(&t, &|n| n, &|a, b| a + b), 9);
        assert_eq!(fold(&t, &|_| 1, &|a, b| a + b), 3);
        assert_eq!(fold(&t, &|n| n, &i64::max), 5);
        assert_eq!(
            fold(&t, &|n| n.to_string(), &|a, b| format!("({a} + {b})")),
            "((3 + 1) + 5)"
        );
        assert!(fold(&t, &|n| n > 4, &|a, b| a || b));
        assert_eq!(fold(&Tree::Leaf(-2), &|n| n, &i64::max), -2);
    }
}
