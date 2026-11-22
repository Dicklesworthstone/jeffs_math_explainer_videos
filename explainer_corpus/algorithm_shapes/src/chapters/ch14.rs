//! A commuting square is a local obligation, not agreement on one output.
use super::draw::{Map, ROUTES, dot, hide, prediction, show};
use crate::model::ROADS;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter14 {
    pub kit: Kit,
}

#[derive(Clone, Debug)]
enum Tree {
    Leaf(u64),
    Branch(Box<Tree>, Box<Tree>),
}
fn example() -> Tree {
    Tree::Branch(
        Box::new(Tree::Branch(Box::new(Tree::Leaf(3)), Box::new(Tree::Leaf(1)))),
        Box::new(Tree::Leaf(5)),
    )
}
fn fold(t: &Tree) -> u64 {
    match t {
        Tree::Leaf(n) => *n,
        Tree::Branch(a, b) => fold(a) + fold(b),
    }
}
fn leaves(t: &Tree) -> u64 {
    match t {
        Tree::Leaf(_) => 1,
        Tree::Branch(a, b) => leaves(a) + leaves(b),
    }
}
fn layer(t: &Tree, h: &impl Fn(&Tree) -> u64) -> u64 {
    match t {
        Tree::Leaf(n) => *n,
        Tree::Branch(a, b) => h(a) + h(b),
    }
}
fn checks(t: &Tree, h: &impl Fn(&Tree) -> u64) -> Vec<(String, u64, u64)> {
    checks_below(t, h, true)
}
/// The tree's shape in the notation of the square's first corner, such as `((3,1),5)`.
fn shape(t: &Tree) -> String {
    match t {
        Tree::Leaf(n) => n.to_string(),
        Tree::Branch(a, b) => format!("({},{})", shape(a), shape(b)),
    }
}
/// Every layer's check, named so that the top and the inner branch differ on screen.
fn checks_below(t: &Tree, h: &impl Fn(&Tree) -> u64, top: bool) -> Vec<(String, u64, u64)> {
    let name = match t {
        Tree::Leaf(n) => format!("Leaf {n}"),
        Tree::Branch(_, _) if top => "Top".into(),
        Tree::Branch(_, _) => format!("Branch {}", shape(t)),
    };
    let mut out = vec![(name, h(t), layer(t, h))];
    if let Tree::Branch(a, b) = t {
        out.extend(checks_below(a, h, false));
        out.extend(checks_below(b, h, false));
    }
    out
}
fn hue(route: usize) -> Srgb {
    explainer_kit::palette::hex(if route == 0 { "#B9A3FF" } else { "#FFE29A" })
}
fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, p: Vec3, size: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, size, color, p)
}
fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(a, b)
            .buff(0.12)
            .thickness(1.8)
            .style(Style::default().fill(color, 1.0).stroke(color, 1.0, 1.0))
            .build()?,
    ))
}
fn travel(stage: &mut Stage<'_>, points: &[Vec3], color: Srgb) -> fmn::Result<()> {
    let token = dot(stage, points[0], 0.09, color);
    show(stage, &[token], 0.12)?;
    for &p in &points[1..] {
        stage.play(
            token
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.65),
                    ..Default::default()
                })?
                .move_to(p, ORIGIN)?,
        )?;
    }
    hide(stage, &[token], 0.12)
}
const CORNERS: [Vec3; 4] = [[0.8, 1.65, 0.0], [5.2, 1.65, 0.0], [0.8, -1.25, 0.0], [5.2, -1.25, 0.0]];
struct Square {
    all: Vec<Mob>,
    values: Vec<Mob>,
}
impl Square {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        let mut all = Vec::new();
        for (i, j) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
            let a = CORNERS[i];
            let b = CORNERS[j];
            let start = v(a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25);
            let end = v(a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75);
            all.push(arrow(stage, start, end, Palette::dim())?);
        }
        for (s, p) in [
            ("assemble", v(3.0, 2.1)),
            ("h", v(5.65, 0.2)),
            ("F h", v(0.15, 0.2)),
            ("layer rule", v(3.0, -1.7)),
        ] {
            all.push(text(kit, stage, s, p, 24.0, Palette::dim())?);
        }
        let mut square = Self {
            all,
            values: Vec::new(),
        };
        square.set(kit, stage, ["F Trees", "Trees", "F Numbers", "Numbers"], Palette::ink())?;
        show(stage, &square.all, 0.3)?;
        Ok(square)
    }
    fn set(&mut self, kit: &Kit, stage: &mut Stage<'_>, labels: [&str; 4], color: Srgb) -> fmn::Result<()> {
        hide(stage, &self.values, 0.12)?;
        self.values.clear();
        for (s, p) in labels.into_iter().zip(CORNERS) {
            self.values.push(text(kit, stage, s, p, 25.0, color)?);
        }
        show(stage, &self.values, 0.22)
    }
    fn clear(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        hide(stage, &self.values, 0.2)?;
        hide(stage, &self.all, 0.2)
    }
    /// Name the side arrows after the rule being checked: `rule` down the right, `F rule` down the left.
    /// `all[5]` and `all[6]` are the "h" and "F h" labels made in `build`.
    fn rename(&mut self, kit: &Kit, stage: &mut Stage<'_>, rule: &str) -> fmn::Result<()> {
        hide(stage, &self.all[5..7], 0.12)?;
        self.all[5] = text(kit, stage, rule, v(5.65, 0.2), 24.0, Palette::dim())?;
        self.all[6] = text(kit, stage, &format!("F {rule}"), v(0.15, 0.2), 24.0, Palette::dim())?;
        show(stage, &self.all[5..7], 0.22)
    }
}
fn miniature(kit: &Kit, stage: &mut Stage<'_>, x: f64, y: f64, scale: f64) -> fmn::Result<Vec<Mob>> {
    let points = [
        v(x, y),
        v(x - scale, y - scale),
        v(x + scale, y - scale),
        v(x - 1.5 * scale, y - 2.0 * scale),
        v(x - 0.5 * scale, y - 2.0 * scale),
    ];
    let mut all = Vec::new();
    for (a, b) in [(0, 1), (0, 2), (1, 3), (1, 4)] {
        all.push(
            stage.arena_mut().add(
                Line::new(points[a], points[b])
                    .style(Style::default().stroke(Palette::dim(), 1.6, 1.0))
                    .build()?,
            ),
        );
    }
    for p in points {
        all.push(dot(stage, p, 0.06, Palette::ink()));
    }
    for (i, value) in [(3, 3), (4, 1), (2, 5)] {
        all.push(text(
            kit,
            stage,
            &value.to_string(),
            v(points[i][0], points[i][1] - 0.25),
            23.0,
            Palette::ink(),
        )?);
    }
    Ok(all)
}
impl SceneConstruct for Chapter14 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let tree = example();
        kit.say(stage, "14_1")?;
        let title = text(
            kit,
            stage,
            "Category theory: a little",
            v(0.0, 2.5),
            39.0,
            Palette::ink(),
        )?;
        show(stage, &[title], 0.4)?;
        kit.hold(stage)?;
        kit.say(stage, "14_2")?;
        let mut intro = Vec::new();
        for (s, x) in [("Trees", -4.0), ("Numbers", 0.0), ("Numbers", 4.0)] {
            intro.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.9)
                        .style(Style::default().stroke(Palette::dim(), 1.5, 1.0))
                        .build()
                        .moved_to(v(x, 0.0)),
                ),
            );
            intro.push(text(kit, stage, s, v(x, 0.0), 30.0, Palette::ink())?);
            intro.push(text(
                kit,
                stage,
                "identity: leave unchanged",
                v(x, -1.4),
                19.0,
                Palette::dim(),
            )?);
        }
        for (a, b, s) in [(-2.95, -1.05, "fold"), (1.05, 2.95, "double")] {
            intro.push(arrow(stage, v(a, 0.0), v(b, 0.0), hue(0))?);
            intro.push(text(kit, stage, s, v((a + b) / 2.0, 0.45), 25.0, hue(0))?);
        }
        show(stage, &intro, 0.4)?;
        kit.until_sentence(stage, "14_2", 4)?;
        travel(stage, &[v(-4.0, 0.0), v(0.0, 0.0), v(4.0, 0.0)], hue(0))?;
        for x in [-4.0, 0.0] {
            // Return to exactly the same object: the identity does not change it.
            travel(
                stage,
                &[v(x, 0.0), v(x - 0.65, 0.65), v(x, 1.0), v(x + 0.65, 0.65), v(x, 0.0)],
                hue(1),
            )?;
        }
        kit.hold(stage)?;
        hide(stage, &intro, 0.3)?;
        hide(stage, &[title], 0.3)?;
        kit.say(stage, "14_3")?;
        let template = text(
            kit,
            stage,
            "One layer: leaf n  OR  branch( • , • )",
            v(0.0, 2.8),
            32.0,
            Palette::ink(),
        )?;
        show(stage, &[template], 0.3)?;
        let small = miniature(kit, stage, -3.3, 1.4, 0.55)?;
        show(stage, &small, 0.4)?;
        let mut layer_items = vec![
            text(kit, stage, "Trees in the slots", v(-3.3, -0.5), 25.0, hue(0))?,
            text(
                kit,
                stage,
                &match &tree {
                    Tree::Branch(a, b) => format!("branch({}, {})", fold(a), fold(b)),
                    Tree::Leaf(n) => format!("leaf {n}"),
                },
                v(3.0, 1.0),
                34.0,
                hue(1),
            )?,
            text(kit, stage, "Numbers in the slots", v(3.0, -0.5), 25.0, hue(1))?,
        ];
        show(stage, &layer_items, 0.4)?;
        kit.hold(stage)?;
        kit.say(stage, "14_4")?;
        let inside = arrow(stage, v(-1.5, 1.0), v(1.1, 1.0), hue(0))?;
        let functor = text(
            kit,
            stage,
            "F h: apply h inside each child slot",
            v(0.0, -1.5),
            29.0,
            Palette::ink(),
        )?;
        let laws = text(
            kit,
            stage,
            "F identity = identity\nF(g ∘ k) = Fg ∘ Fk",
            v(0.0, -2.5),
            23.0,
            Palette::dim(),
        )?;
        layer_items.extend([inside, functor, laws]);
        show(stage, &[inside, functor, laws], 0.3)?;
        travel(stage, &[v(-3.85, 0.85), v(2.5, 1.0)], hue(0))?;
        travel(stage, &[v(-2.75, 0.85), v(3.5, 1.0)], hue(1))?;
        kit.hold(stage)?;
        hide(stage, &layer_items, 0.3)?;
        hide(stage, &small, 0.3)?;
        hide(stage, &[template], 0.3)?;
        kit.say(stage, "14_5")?;
        let left_tree = miniature(kit, stage, -4.3, 1.7, 0.8)?;
        show(stage, &left_tree, 0.3)?;
        let mut square = Square::build(kit, stage)?;
        square.set(
            kit,
            stage,
            ["branch\n((3,1),5)", "whole tree", "children?", "answer?"],
            Palette::ink(),
        )?;
        kit.hold(stage)?;
        kit.say(stage, "14_6")?;
        travel(stage, &[CORNERS[0], CORNERS[1], CORNERS[3]], hue(0))?;
        let root = fold(&tree).to_string();
        square.set(
            kit,
            stage,
            ["branch\n((3,1),5)", "((3,1),5)", "children?", &root],
            hue(0),
        )?;
        kit.hold(stage)?;
        kit.say(stage, "14_7")?;
        let Tree::Branch(a, b) = &tree else { unreachable!() };
        let children = format!("branch({}, {})", fold(a), fold(b));
        travel(stage, &[CORNERS[0], CORNERS[2], CORNERS[3]], hue(1))?;
        square.set(
            kit,
            stage,
            ["branch\n((3,1),5)", "((3,1),5)", &children, &root],
            Palette::teal(),
        )?;
        kit.hold(stage)?;
        kit.say(stage, "14_8")?;
        let commuting = text(
            kit,
            stage,
            "Both orders agree: the square commutes",
            v(2.7, -2.5),
            25.0,
            Palette::teal(),
        )?;
        show(stage, &[commuting], 0.3)?;
        hide(stage, &left_tree, 0.3)?;
        let map = Map::build(kit, stage, [-2.0, 0.8], 0.55, false)?;
        show(stage, &map.all(), 0.3)?;
        let times: Vec<_> = ROUTES
            .iter()
            .map(|r| r.iter().map(|&e| ROADS[e].value).sum::<u64>())
            .collect();
        let choices = text(
            kit,
            stage,
            &format!(
                "Map routes: {}, {}, {}\nHere we must choose",
                times[0], times[1], times[2]
            ),
            v(-4.0, -1.3),
            24.0,
            Palette::dim(),
        )?;
        show(stage, &[choices], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &map.all(), 0.3)?;
        hide(stage, &[choices, commuting], 0.3)?;
        kit.say(stage, "14_9")?;
        show(stage, &left_tree, 0.3)?;
        let lawful = checks(&tree, &fold);
        let mut audit = Vec::new();
        for &i in &[2usize, 3, 1, 4, 0] {
            let (name, lhs, rhs) = &lawful[i];
            square.set(
                kit,
                stage,
                [name, "assemble, h", "h children", &format!("{lhs} = {rhs}")],
                Palette::teal(),
            )?;
            let row = text(
                kit,
                stage,
                &format!("{name}: {lhs} = {rhs}"),
                v(-4.0, -0.6 - audit.len() as f64 * 0.42),
                21.0,
                Palette::teal(),
            )?;
            show(stage, &[row], 0.25)?;
            audit.push(row);
        }
        kit.hold(stage)?;
        hide(stage, &audit, 0.25)?;
        kit.say(stage, "14_10")?;
        let fake = |t: &Tree| 3 * leaves(t);
        let challenge = text(
            kit,
            stage,
            &format!("g = 3 × leaves\ng(tree) = {}\nIs g the fold?", fake(&tree)),
            v(-4.0, -1.5),
            27.0,
            Palette::gold(),
        )?;
        show(stage, &[challenge], 0.3)?;
        kit.hold(stage)?;
        prediction(kit, stage, 5, v(3.0, -2.9))?;
        hide(stage, &[challenge], 0.2)?;
        kit.say(stage, "14_11")?;
        square.rename(kit, stage, "g")?;
        let false_checks = checks(&tree, &fake);
        let mut failures = Vec::new();
        for (i, (name, lhs, rhs)) in false_checks.iter().enumerate() {
            kit.until_sentence(stage, "14_11", i + 1)?;
            let color = if lhs == rhs {
                Palette::teal()
            } else {
                explainer_kit::palette::hex("#FF6B6B")
            };
            square.set(
                kit,
                stage,
                [name, "assemble, g", "g children", &format!("{lhs} vs {rhs}")],
                color,
            )?;
            let row = text(
                kit,
                stage,
                &format!("{name}: {lhs} vs {rhs}"),
                v(-4.0, -0.6 - i as f64 * 0.42),
                22.0,
                color,
            )?;
            show(stage, &[row], 0.3)?;
            failures.push(row);
        }
        kit.hold(stage)?;
        kit.say(stage, "14_12")?;
        hide(stage, &failures[..3], 0.3)?;
        let warning = text(
            kit,
            stage,
            "One matching result is not a law",
            v(2.8, -2.5),
            26.0,
            Palette::gold(),
        )?;
        show(stage, &[warning], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &failures[3..], 0.25)?;
        hide(stage, &[warning], 0.25)?;
        square.clear(stage)?;
        hide(stage, &left_tree, 0.25)?;
        kit.say(stage, "14_13")?;
        let theorem = text(kit, stage, "Initial algebra", v(0.0, 2.3), 40.0, Palette::ink())?;
        let bounds = text(
            kit,
            stage,
            "Finite trees + total functions\nOne layer rule → exactly one fold",
            v(0.0, 0.5),
            32.0,
            Palette::teal(),
        )?;
        let caveat = text(
            kit,
            stage,
            "Infinite trees or partial values need extra hypotheses",
            v(0.0, -1.4),
            25.0,
            Palette::dim(),
        )?;
        show(stage, &[theorem, bounds, caveat], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[theorem, bounds, caveat], 0.3)?;
        // Carry the square into the spoken bridge. Details still wait for their sentence.
        let bridge_points = [v(-1.4, 0.65), v(1.4, 0.65), v(-1.4, -1.15), v(1.4, -1.15)];
        let bridge = [
            text(kit, stage, "The same square", v(0.0, 2.25), 40.0, Palette::ink())?,
            arrow(stage, bridge_points[0], bridge_points[1], hue(0))?,
            arrow(stage, bridge_points[1], bridge_points[3], hue(0))?,
            arrow(stage, bridge_points[0], bridge_points[2], hue(1))?,
            arrow(stage, bridge_points[2], bridge_points[3], hue(1))?,
        ];
        for &object in &bridge {
            stage.add_to_scene(object)?;
        }
        kit.say(stage, "14_14")?;
        // The top branch's children, as in chapter 13's doubling condition.
        let fusion = (2 * (fold(a) + fold(b)), 2 * fold(a) + 2 * fold(b));
        let first = [1i64, 2, 3];
        let second = [10i64];
        let chunk = (
            (
                first.iter().chain(second.iter()).sum::<i64>(),
                first.len() + second.len(),
            ),
            (
                first.iter().sum::<i64>() + second.iter().sum::<i64>(),
                first.len() + second.len(),
            ),
        );
        let direct = ROADS[1].value;
        let via_hill = ROADS[0].value + ROADS[2].value;
        let onward = ROADS[4].value;
        let distribution = ((direct + onward).min(via_hill + onward), direct.min(via_hill) + onward);
        let values = [
            format!("{} = {}", fusion.0, fusion.1),
            format!("{:?} = {:?}", chunk.0, chunk.1),
            format!("{} = {}", distribution.0, distribution.1),
        ];
        for (i, (name, upper, lower)) in [
            ("Fusion", "add → double", "double → add"),
            ("Chunks", "join → summarize", "summarize → merge"),
            ("Distribution", "combine → extend", "extend → combine"),
        ]
        .iter()
        .enumerate()
        {
            kit.until_sentence(stage, "14_14", i + 1)?;
            if i == 0 {
                for &object in &bridge {
                    stage.remove_from_scene(object);
                }
            }
            let x = -4.3 + i as f64 * 4.3;
            let p = [v(x - 1.35, 1.2), v(x + 1.35, 1.2), v(x - 1.35, -0.5), v(x + 1.35, -0.5)];
            let mut objects = vec![
                text(kit, stage, name, v(x, 2.3), 30.0, Palette::ink())?,
                text(kit, stage, upper, v(x, 1.65), 19.0, hue(0))?,
                text(kit, stage, lower, v(x, -0.95), 19.0, hue(1))?,
                text(kit, stage, &values[i], v(x, -1.7), 25.0, Palette::teal())?,
            ];
            for (a, b) in [(0, 1), (1, 3), (0, 2), (2, 3)] {
                objects.push(arrow(stage, p[a], p[b], Palette::dim())?);
            }
            show(stage, &objects, 0.3)?;
            travel(stage, &[p[0], p[1], p[3]], hue(0))?;
            travel(stage, &[p[0], p[2], p[3]], hue(1))?;
        }
        kit.hold(stage)?;
        kit.finish(stage)
    }
}

pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let tree = example();
    for (name, rows) in [
        ("fold", checks(&tree, &fold)),
        ("three_times_leaves", checks(&tree, &|t| 3 * leaves(t))),
    ] {
        facts.add(&format!("square.{name}.layers"), rows.len())?;
        for (i, (_, across, down)) in rows.iter().enumerate() {
            facts.add(&format!("square.{name}.{i}"), format!("{across},{down}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_tree_agreement_does_not_replace_layer_checks() {
        let tree = example();
        assert!(checks(&tree, &fold).iter().all(|(_, a, b)| a == b));
        let fake = |t: &Tree| 3 * leaves(t);
        assert_eq!(fake(&tree), fold(&tree));
        assert_eq!(
            checks(&tree, &fake)
                .iter()
                .map(|(_, a, b)| (*a, *b))
                .collect::<Vec<_>>(),
            vec![(9, 9), (6, 6), (3, 3), (3, 1), (3, 5)]
        );
        for n in 0..=12 {
            assert_eq!(layer(&Tree::Leaf(n), &fake), n);
        }
    }
    #[test]
    fn layer_names_tell_the_two_branches_apart() {
        let tree = example();
        let names: Vec<String> = checks(&tree, &fold).into_iter().map(|(name, _, _)| name).collect();
        assert_eq!(names, ["Top", "Branch (3,1)", "Leaf 3", "Leaf 1", "Leaf 5"]);
        assert_eq!(shape(&tree), "((3,1),5)");
        let Tree::Branch(a, b) = &tree else { unreachable!() };
        assert_eq!((2 * (fold(a) + fold(b)), 2 * fold(a) + 2 * fold(b)), (18, 18));
    }
}
