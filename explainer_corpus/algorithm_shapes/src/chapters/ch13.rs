//! Two distinct fusion laws, with an explicit reference for each comparison.
use super::draw::{dot, hide, prediction, show};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
pub struct Chapter13 {
    pub kit: Kit,
}
#[derive(Clone, Debug)]
enum Tree {
    Leaf(i64),
    Branch(Box<Tree>, Box<Tree>),
}
impl Tree {
    fn sample() -> Self {
        Self::Branch(
            Box::new(Self::Branch(Box::new(Self::Leaf(3)), Box::new(Self::Leaf(1)))),
            Box::new(Self::Leaf(5)),
        )
    }
    fn map(&self, f: fn(i64) -> i64) -> Self {
        match self {
            Self::Leaf(n) => Self::Leaf(f(*n)),
            Self::Branch(a, b) => Self::Branch(Box::new(a.map(f)), Box::new(b.map(f))),
        }
    }
    fn fold(&self, f: fn(i64) -> i64, g: fn(i64, i64) -> i64) -> i64 {
        match self {
            Self::Leaf(n) => f(*n),
            Self::Branch(a, b) => g(a.fold(f, g), b.fold(f, g)),
        }
    }
    fn nodes(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Branch(a, b) => 1 + a.nodes() + b.nodes(),
        }
    }
    fn leaves(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Branch(a, b) => a.leaves() + b.leaves(),
        }
    }
}
fn identity(n: i64) -> i64 {
    n
}
fn add(a: i64, b: i64) -> i64 {
    a + b
}
fn double(n: i64) -> i64 {
    n * 2
}
fn square(n: i64) -> i64 {
    n * n
}
fn plus_one(n: i64) -> i64 {
    n + 1
}
const POINTS: [[f64; 2]; 5] = [[0.0, 1.2], [-0.8, 0.0], [0.8, 0.0], [-1.3, -1.2], [-0.3, -1.2]];
const EDGES: [(usize, usize); 4] = [(0, 1), (0, 2), (1, 3), (1, 4)];
fn picture(kit: &Kit, stage: &mut Stage<'_>, shift: [f64; 2], scale: f64, f: fn(i64) -> i64) -> fmn::Result<Vec<Mob>> {
    let p = POINTS.map(|p| v(p[0] * scale + shift[0], p[1] * scale + shift[1]));
    let mut mobs = Vec::new();
    for (a, b) in EDGES {
        mobs.push(
            stage.arena_mut().add(
                Line::new(p[a], p[b])
                    .style(Style::default().stroke(Palette::dim(), 1.7, 1.0))
                    .build()?,
            ),
        );
    }
    for (i, point) in p.iter().enumerate() {
        mobs.push(dot(stage, *point, 0.07 * scale, Palette::teal()));
        if let Some(n) = match i {
            2 => Some(5),
            3 => Some(3),
            4 => Some(1),
            _ => None,
        } {
            mobs.push(kit.text(
                stage,
                &f(n).to_string(),
                28.0 * scale,
                Palette::gold(),
                v(point[0], point[1] - 0.24 * scale),
            )?);
        }
    }
    Ok(mobs)
}
fn label(kit: &Kit, stage: &mut Stage<'_>, s: &str, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 27.0, color, v(x, y))
}
fn frame(stage: &mut Stage<'_>, y: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(6.5)
            .height(1.55)
            .style(Style::default().stroke(color, 1.8, 1.0))
            .build()?
            .moved_to(v(2.8, y)),
    ))
}
fn rule(kit: &Kit, stage: &mut Stage<'_>, leaf: &str, branch: &str) -> fmn::Result<Mob> {
    label(
        kit,
        stage,
        &format!("leaf: {leaf}\nbranch: {branch}"),
        1.8,
        0.0,
        Palette::ink(),
    )
}
fn result(kit: &Kit, stage: &mut Stage<'_>, n: i64, lane: usize, color: Srgb) -> fmn::Result<Mob> {
    let m = kit.text(stage, &n.to_string(), 38.0, color, v(5.3, 2.1 - lane as f64 * 2.1))?;
    show(stage, &[m], 0.25)?;
    Ok(m)
}
fn rise(kit: &Kit, stage: &mut Stage<'_>, f: fn(i64) -> i64, g: fn(i64, i64) -> i64) -> fmn::Result<()> {
    let positions = POINTS.map(|p| v(p[0] - 4.5, p[1] + 0.5));
    let values = [g(g(f(3), f(1)), f(5)), g(f(3), f(1)), f(5), f(3), f(1)];
    let mut marks = Vec::new();
    for i in [3, 4, 1, 2, 0] {
        let m = kit.text(stage, &values[i].to_string(), 28.0, Palette::gold(), positions[i])?;
        show(stage, &[m], 0.15)?;
        if i != 0 {
            let parent = if i >= 3 { 1 } else { 0 };
            stage.play(
                m.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.35),
                        ..AnimateArgs::default()
                    })?
                    .move_to(positions[parent], ORIGIN)?,
            )?;
        }
        hide(stage, &[m], 0.1)?;
        marks.push(m);
    }
    Ok(())
}
impl SceneConstruct for Chapter13 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let tree = Tree::sample();
        let sum = tree.fold(identity, add);
        // The top branch's two children, whose totals the conditions in 13_5, 13_7 and 13_8 show.
        let Tree::Branch(left_tree, right_tree) = &tree else {
            unreachable!("the sample is a branch")
        };
        let (left, right) = (left_tree.fold(identity, add), right_tree.fold(identity, add));
        kit.say(stage, "13_1")?;
        let original = picture(kit, stage, [-4.5, 0.5], 1.0, identity)?;
        show(stage, &original, 0.4)?;
        let mut cards = Vec::new();
        for (i, name) in [
            "1 · Change leaves, then fold",
            "2 · Change the leaf rule",
            "3 · Fold, then change total",
        ]
        .iter()
        .enumerate()
        {
            let y = 2.1 - i as f64 * 2.1;
            cards.push(frame(stage, y, Palette::dim())?);
            cards.push(kit.text(
                stage,
                name,
                23.0,
                if i == 2 { Palette::dim() } else { Palette::ink() },
                v(2.5, y + 0.62),
            )?);
        }
        show(stage, &cards, 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "13_2")?;
        let mut new_tree = picture(kit, stage, [0.8, 2.05], 0.36, double)?;
        for &mob in &new_tree {
            show(stage, &[mob], 0.1)?;
        }
        // Each readout lands with the sentence that says it.
        kit.until_sentence(stage, "13_2", 1)?;
        let allocated = label(
            kit,
            stage,
            &format!("{} tree pieces built", tree.nodes()),
            3.0,
            1.65,
            Palette::dim(),
        )?;
        show(stage, &[allocated], 0.3)?;
        kit.until_sentence(stage, "13_2", 3)?;
        let mut answers = vec![result(
            kit,
            stage,
            tree.map(double).fold(identity, add),
            0,
            Palette::ink(),
        )?];
        kit.hold(stage)?;
        kit.say(stage, "13_3")?;
        let mut leaf_rule = rule(kit, stage, "2n", "a + b")?;
        show(stage, &[leaf_rule], 0.3)?;
        kit.until_sentence(stage, "13_3", 1)?;
        rise(kit, stage, double, add)?;
        kit.until_sentence(stage, "13_3", 2)?;
        answers.push(result(kit, stage, tree.fold(double, add), 1, Palette::teal())?);
        let no_tree = label(kit, stage, "No new tree", 3.0, -0.5, Palette::dim())?;
        show(stage, &[no_tree], 0.3)?;
        let mut reference = label(kit, stage, "Reference: lane 1", -4.2, -1.6, Palette::ink())?;
        let ref_frame = frame(stage, 2.1, Palette::gold())?;
        show(stage, &[reference, ref_frame], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "13_4")?;
        // The name lands with sentence 0, the squares with sentence 2, the reason with sentence 3.
        let fusion = label(
            kit,
            stage,
            "Fusion: remove the intermediate tree",
            0.0,
            -3.2,
            Palette::teal(),
        )?;
        show(stage, &[fusion], 0.3)?;
        kit.until_sentence(stage, "13_4", 2)?;
        hide(stage, &new_tree, 0.2)?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        hide(stage, &[leaf_rule], 0.2)?;
        new_tree = picture(kit, stage, [0.8, 2.05], 0.36, square)?;
        show(stage, &new_tree, 0.3)?;
        leaf_rule = rule(kit, stage, "n²", "a + b")?;
        show(stage, &[leaf_rule], 0.3)?;
        answers.push(result(
            kit,
            stage,
            tree.map(square).fold(identity, add),
            0,
            Palette::ink(),
        )?);
        rise(kit, stage, square, add)?;
        answers.push(result(kit, stage, tree.fold(square, add), 1, Palette::teal())?);
        kit.until_sentence(stage, "13_4", 3)?;
        let precisely_fusion = kit.text(
            stage,
            "Precisely, for every leaf change f:\nfold(leaf, branch) ∘ map f\n= fold(leaf ∘ f, branch)",
            24.0,
            Palette::dim(),
            v(-4.0, -2.45),
        )?;
        show(stage, &[precisely_fusion], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[fusion, precisely_fusion], 0.2)?;
        kit.say(stage, "13_5")?;
        hide(stage, &new_tree, 0.2)?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        hide(stage, &[leaf_rule], 0.2)?;
        new_tree = picture(kit, stage, [0.8, 2.05], 0.36, double)?;
        show(stage, &new_tree, 0.3)?;
        leaf_rule = rule(kit, stage, "2n", "a + b")?;
        show(stage, &[leaf_rule], 0.3)?;
        answers.push(result(
            kit,
            stage,
            tree.map(double).fold(identity, add),
            0,
            Palette::ink(),
        )?);
        answers.push(result(kit, stage, tree.fold(double, add), 1, Palette::teal())?);
        kit.until_sentence(stage, "13_5", 1)?;
        let mut post = label(kit, stage, &format!("{sum} → double"), 1.8, -2.1, Palette::ink())?;
        show(stage, &[post], 0.3)?;
        kit.until_sentence(stage, "13_5", 2)?;
        answers.push(result(kit, stage, double(sum), 2, Palette::teal())?);
        kit.until_sentence(stage, "13_5", 3)?;
        let mut condition = label(
            kit,
            stage,
            &format!(
                "2 × ({left} + {right}) = {} + {} = {}",
                double(left),
                double(right),
                double(left + right)
            ),
            0.0,
            -3.2,
            Palette::teal(),
        )?;
        show(stage, &[condition], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "13_6")?;
        hide(stage, &new_tree, 0.2)?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        hide(stage, &[leaf_rule, post, condition], 0.2)?;
        leaf_rule = rule(kit, stage, "n + 1", "a + b")?;
        show(stage, &[leaf_rule], 0.3)?;
        kit.until_sentence(stage, "13_6", 1)?;
        post = label(kit, stage, &format!("{sum} → add one"), 1.8, -2.1, Palette::ink())?;
        show(stage, &[post], 0.3)?;
        kit.until_sentence(stage, "13_6", 2)?;
        answers.push(result(kit, stage, plus_one(sum), 2, Palette::ink())?);
        prediction(kit, stage, 4, v(-4.2, -2.3))?;
        kit.say(stage, "13_7")?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        new_tree = picture(kit, stage, [0.8, 2.05], 0.36, plus_one)?;
        show(stage, &new_tree, 0.3)?;
        answers.push(result(
            kit,
            stage,
            tree.map(plus_one).fold(identity, add),
            0,
            Palette::ink(),
        )?);
        rise(kit, stage, plus_one, add)?;
        answers.push(result(kit, stage, tree.fold(plus_one, add), 1, Palette::teal())?);
        answers.push(result(kit, stage, plus_one(sum), 2, Palette::red())?);
        kit.until_sentence(stage, "13_7", 2)?;
        condition = label(
            kit,
            stage,
            &format!(
                "({left} + {right}) + 1 = {} ≠ {} = ({left} + 1) + ({right} + 1)",
                plus_one(left + right),
                plus_one(left) + plus_one(right)
            ),
            0.0,
            -3.2,
            Palette::red(),
        )?;
        show(stage, &[condition], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "13_8")?;
        hide(stage, &new_tree, 0.2)?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        hide(stage, &[leaf_rule, post, condition], 0.2)?;
        new_tree = picture(kit, stage, [0.8, 2.05], 0.36, square)?;
        show(stage, &new_tree, 0.3)?;
        leaf_rule = rule(kit, stage, "n²", "a + b")?;
        show(stage, &[leaf_rule], 0.3)?;
        post = label(kit, stage, &format!("{sum} → square"), 1.8, -2.1, Palette::ink())?;
        show(stage, &[post], 0.3)?;
        kit.until_sentence(stage, "13_8", 1)?;
        answers.push(result(
            kit,
            stage,
            tree.map(square).fold(identity, add),
            0,
            Palette::ink(),
        )?);
        answers.push(result(kit, stage, tree.fold(square, add), 1, Palette::teal())?);
        answers.push(result(kit, stage, square(sum), 2, Palette::red())?);
        condition = label(
            kit,
            stage,
            &format!(
                "({left} + {right})² = {} ≠ {} = {left}² + {right}²",
                square(left + right),
                square(left) + square(right)
            ),
            0.0,
            -3.2,
            Palette::red(),
        )?;
        show(stage, &[condition], 0.3)?;
        kit.until_sentence(stage, "13_8", 2)?;
        hide(stage, &new_tree, 0.3)?;
        hide(stage, &answers, 0.2)?;
        answers.clear();
        hide(
            stage,
            &[allocated, leaf_rule, post, condition, reference, ref_frame],
            0.3,
        )?;
        reference = label(kit, stage, "Now compare with lane 3", -4.0, -1.6, Palette::gold())?;
        let rescue_frame = frame(stage, -2.1, Palette::gold())?;
        show(stage, &[reference, rescue_frame], 0.3)?;
        leaf_rule = rule(kit, stage, "n + 1", "a + b − 1")?;
        show(stage, &[leaf_rule], 0.3)?;
        post = label(kit, stage, &format!("{sum} → add one"), 1.8, -2.1, Palette::ink())?;
        show(stage, &[post], 0.3)?;
        let repair = |a, b| a + b - 1;
        rise(kit, stage, plus_one, repair)?;
        answers.push(result(kit, stage, tree.fold(plus_one, repair), 1, Palette::teal())?);
        answers.push(result(kit, stage, plus_one(sum), 2, Palette::teal())?);
        kit.hold(stage)?;
        kit.say(stage, "13_9")?;
        hide(stage, &answers, 0.2)?;
        hide(stage, &[leaf_rule, post, reference, rescue_frame, no_tree], 0.2)?;
        let counts = [
            label(
                kit,
                stage,
                &format!("{} tree pieces built", tree.nodes()),
                2.8,
                2.1,
                Palette::ink(),
            )?,
            label(
                kit,
                stage,
                &format!("No new tree · {} doublings", tree.leaves()),
                2.8,
                0.0,
                Palette::teal(),
            )?,
            label(kit, stage, "No new tree · one doubling", 2.8, -2.1, Palette::gold())?,
        ];
        show(stage, &counts, 0.3)?;
        let caveat = label(
            kit,
            stage,
            "Counts by construction, not measured timings",
            0.0,
            -3.2,
            Palette::ink(),
        )?;
        show(stage, &[caveat], 0.3)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let tree = Tree::sample();
    facts.add("fusion.tree.nodes", tree.nodes())?;
    facts.add("fusion.tree.leaves", tree.leaves())?;
    facts.add("fusion.tree.sum", tree.fold(identity, add))?;
    for (name, change) in [
        ("double", double as fn(i64) -> i64),
        ("square", square as fn(i64) -> i64),
        ("plus1", plus_one as fn(i64) -> i64),
    ] {
        let mapped = tree.map(change);
        facts.add(&format!("fusion.{name}.mapped"), mapped.fold(identity, add))?;
        facts.add(&format!("fusion.{name}.fused"), tree.fold(change, add))?;
        facts.add(&format!("fusion.{name}.after"), change(tree.fold(identity, add)))?;
        facts.add(&format!("fusion.{name}.constructed"), mapped.nodes())?;
    }
    facts.add("fusion.plus1.repaired", tree.fold(plus_one, |a, b| a + b - 1))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_fusions_and_changed_reference_have_distinct_laws() {
        let t = Tree::sample();
        let s = t.fold(identity, add);
        assert_eq!((t.nodes(), t.leaves()), (5, 3));
        for f in [double as fn(i64) -> i64, square, plus_one] {
            assert_eq!(t.map(f).fold(identity, add), t.fold(f, add));
        }
        assert_eq!((t.fold(double, add), double(s)), (18, 18));
        assert_eq!((t.fold(square, add), square(s)), (35, 81));
        assert_eq!((t.fold(plus_one, add), plus_one(s)), (12, 10));
        assert_eq!(t.fold(plus_one, |a, b| a + b - 1), plus_one(s));
        for a in -3..=3 {
            for b in -3..=3 {
                assert_eq!(plus_one(a + b), plus_one(a) + plus_one(b) - 1);
            }
        }
    }
    #[test]
    fn conditions_use_the_top_branch_children() {
        let Tree::Branch(left, right) = Tree::sample() else {
            panic!("the sample is a branch")
        };
        let (l, r) = (left.fold(identity, add), right.fold(identity, add));
        assert_eq!((l, r), (4, 5));
        assert_eq!((double(l), double(r), double(l + r)), (8, 10, 18));
        assert_eq!((plus_one(l + r), plus_one(l) + plus_one(r)), (10, 11));
        assert_eq!((square(l + r), square(l) + square(r)), (81, 41));
    }
}
