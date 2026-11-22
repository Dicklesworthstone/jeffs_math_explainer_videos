//! A mergeable summary keeps information that a finished average discards.
use super::draw::{Map, hide, prediction, show};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter12 {
    pub kit: Kit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Summary {
    sum: i64,
    count: usize,
}
impl Summary {
    fn of(values: &[i64]) -> Self {
        Self {
            sum: values.iter().sum(),
            count: values.len(),
        }
    }
    fn merge(self, other: Self) -> Self {
        Self {
            sum: self.sum + other.sum,
            count: self.count + other.count,
        }
    }
    fn pair(self) -> String {
        format!("({}, {})", self.sum, self.count)
    }
    fn mean(self) -> Option<f64> {
        (self.count > 0).then(|| self.sum as f64 / self.count as f64)
    }
    fn fraction(self) -> String {
        if self.count == 0 {
            return "undefined".into();
        }
        let mut a = self.sum.unsigned_abs();
        let mut b = self.count as u64;
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let den = self.count as i64 / a as i64;
        let num = self.sum / a as i64;
        if den == 1 {
            num.to_string()
        } else {
            format!("{num}/{den}")
        }
    }
}
fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 31.0, color, v(x, y))
}
fn box_at(stage: &mut Stage<'_>, x: f64, y: f64, w: f64, h: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(w)
            .height(h)
            .style(Style::default().stroke(color, 2.0, 1.0))
            .build()?
            .moved_to(v(x, y)),
    ))
}
fn move_to(stage: &mut Stage<'_>, mob: Mob, at: Vec3) -> fmn::Result<()> {
    stage.play(
        mob.animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.5),
                ..AnimateArgs::default()
            })?
            .move_to(at, ORIGIN)?,
    )?;
    Ok(())
}
impl SceneConstruct for Chapter12 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let first = [1, 2, 3];
        let second = [10];
        let values = [1, 2, 3, 10];
        let a = Summary::of(&first);
        let b = Summary::of(&second);
        let total = a.merge(b);
        kit.say(stage, "12_1")?;
        let left = box_at(stage, -3.2, 1.0, 5.1, 2.4, Palette::blue())?;
        let right = box_at(stage, 3.2, 1.0, 5.1, 2.4, Palette::teal())?;
        let title = text(kit, stage, "Can we combine two averages?", 0.0, 3.0, Palette::ink())?;
        show(stage, &[left, right, title], 0.4)?;
        let mut tiles = Vec::new();
        for (i, n) in values.iter().enumerate() {
            let label = text(kit, stage, &n.to_string(), -2.7 + i as f64 * 1.8, -1.7, Palette::ink())?;
            show(stage, &[label], 0.2)?;
            move_to(
                stage,
                label,
                if i < 3 {
                    v(-4.5 + i as f64 * 1.3, 1.3)
                } else {
                    v(3.2, 1.3)
                },
            )?;
            tiles.push(label);
        }
        kit.hold(stage)?;
        kit.say(stage, "12_2")?;
        let means = [
            text(
                kit,
                stage,
                &format!("Average: {}", a.fraction()),
                -3.2,
                0.4,
                Palette::gold(),
            )?,
            text(
                kit,
                stage,
                &format!("Average: {}", b.fraction()),
                3.2,
                0.4,
                Palette::gold(),
            )?,
        ];
        show(stage, &means, 0.3)?;
        prediction(kit, stage, 5, v(0.0, -1.3))?;
        kit.say(stage, "12_3")?;
        let bad = (a.mean().unwrap() + b.mean().unwrap()) / 2.0;
        let wrong = text(
            kit,
            stage,
            &format!("({} + {}) ÷ 2 = {bad}", a.fraction(), b.fraction()),
            0.0,
            -1.1,
            Palette::red(),
        )?;
        show(stage, &[wrong], 0.3)?;
        kit.until_sentence(stage, "12_3", 2)?;
        let correct = text(
            kit,
            stage,
            &format!("{} ÷ {} = {}", total.sum, total.count, total.fraction()),
            0.0,
            -2.1,
            Palette::teal(),
        )?;
        show(stage, &[correct], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &tiles, 0.25)?;
        hide(stage, &means, 0.25)?;
        // The question heading stays up while 12_4 asks it again.
        hide(stage, &[left, right, wrong, correct], 0.3)?;
        kit.say(stage, "12_4")?;
        let mut collisions = Vec::new();
        for (row, part) in [&[2][..], &[1, 3][..]].into_iter().enumerate() {
            let zero = Summary::of(&[0]);
            let p = Summary::of(part);
            let joined = zero.merge(p);
            let y = 1.8 - row as f64 * 2.0;
            let originals = text(kit, stage, &format!("[0] + {part:?}"), -4.0, y, Palette::ink())?;
            let summary = text(
                kit,
                stage,
                &format!("Means: ({}, {})", zero.fraction(), p.fraction()),
                0.0,
                y,
                Palette::gold(),
            )?;
            let answer = text(
                kit,
                stage,
                &format!("Together: {}", joined.fraction()),
                4.0,
                y,
                Palette::red(),
            )?;
            // Row 0 on sentences 2 (means) and 3 (result); row 1 on sentence 4.
            if row == 0 {
                kit.until_sentence(stage, "12_4", 2)?;
            } else {
                kit.until_sentence(stage, "12_4", 4)?;
            }
            show(stage, &[originals], 0.3)?;
            show(stage, &[summary], 0.3)?;
            if row == 0 {
                kit.until_sentence(stage, "12_4", 3)?;
            }
            show(stage, &[answer], 0.3)?;
            collisions.extend([originals, summary, answer]);
        }
        collisions.push(text(
            kit,
            stage,
            "Same inputs to the merge · different required answers",
            0.0,
            -2.3,
            Palette::ink(),
        )?);
        kit.until_sentence(stage, "12_4", 5)?;
        show(stage, &collisions[6..], 0.3)?;
        kit.hold(stage)?;
        collisions.push(title);
        hide(stage, &collisions, 0.3)?;
        kit.say(stage, "12_5")?;
        show(stage, &[left, right], 0.3)?;
        let role = text(kit, stage, "(sum, count)", 0.0, 3.0, Palette::ink())?;
        let pa = text(kit, stage, &a.pair(), -3.2, 1.0, Palette::blue())?;
        let pb = text(kit, stage, &b.pair(), 3.2, 1.0, Palette::teal())?;
        show(stage, &[role], 0.3)?;
        kit.until_sentence(stage, "12_5", 1)?;
        show(stage, &[pa], 0.3)?;
        kit.until_sentence(stage, "12_5", 2)?;
        show(stage, &[pb], 0.3)?;
        kit.until_sentence(stage, "12_5", 3)?;
        move_to(stage, pa, v(-1.4, -0.7))?;
        move_to(stage, pb, v(1.4, -0.7))?;
        let joined = text(
            kit,
            stage,
            &format!("{} → {}", total.pair(), total.fraction()),
            0.0,
            -2.0,
            Palette::gold(),
        )?;
        kit.until_sentence(stage, "12_5", 4)?;
        show(stage, &[joined], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[left, right, pa, pb, joined], 0.3)?;
        kit.say(stage, "12_6")?;
        let empty = Summary::of(&[]);
        let identity = text(
            kit,
            stage,
            &format!("{} + {} = {}", a.pair(), empty.pair(), a.merge(empty).pair()),
            0.0,
            1.2,
            Palette::teal(),
        )?;
        let undefined = text(
            kit,
            stage,
            &format!("{} ÷ {}: {}", empty.sum, empty.count, empty.fraction()),
            0.0,
            -0.5,
            Palette::red(),
        )?;
        let outside = text(
            kit,
            stage,
            "Merge first. Divide only if count > 0.",
            0.0,
            -2.1,
            Palette::ink(),
        )?;
        show(stage, &[identity, undefined, outside], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[identity, undefined, outside, role], 0.3)?;
        kit.say(stage, "12_7")?;
        let full = text(kit, stage, &format!("{values:?}"), -3.5, 2.0, Palette::ink())?;
        let split = text(kit, stage, &format!("{first:?} | {second:?}"), 3.5, 2.0, Palette::ink())?;
        let whole = text(kit, stage, &total.pair(), -3.5, -0.3, Palette::gold())?;
        let merged = text(
            kit,
            stage,
            &format!("{} + {}", a.pair(), b.pair()),
            3.5,
            -0.3,
            Palette::teal(),
        )?;
        show(stage, &[full, split], 0.3)?;
        let mut arrows = Vec::new();
        for x in [-3.5, 3.5] {
            arrows.push(
                stage
                    .arena_mut()
                    .add(Arrow::new(v(x, 1.5), v(x, 0.3)).buff(0.1).thickness(2.0).build()?),
            );
        }
        show(stage, &arrows, 0.3)?;
        show(stage, &[whole, merged], 0.3)?;
        let law = kit.tex(
            stage,
            // Two braced (Ord) pluses read as one ++, with \: as binary spacing;
            // the locked fmd-math (126a530) has no \mathbin.
            r"h(x \:{+}{+}\: y)=h(x)\odot h(y)",
            32.0,
            Palette::ink(),
            &[],
            v(0.0, -2.0),
        )?;
        show(stage, &[law], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &arrows, 0.3)?;
        hide(stage, &[full, split, whole, merged, law], 0.3)?;
        kit.say(stage, "12_8")?;
        let leaf: Vec<_> = values.iter().map(|n| Summary::of(&[*n])).collect();
        let mid = [leaf[0].merge(leaf[1]), leaf[2].merge(leaf[3])];
        let top = mid[0].merge(mid[1]);
        let mut tree_mobs = Vec::new();
        for (i, s) in leaf.iter().enumerate() {
            tree_mobs.push(text(
                kit,
                stage,
                &s.pair(),
                -4.5 + i as f64 * 3.0,
                2.0,
                Palette::blue(),
            )?);
        }
        show(stage, &tree_mobs, 0.3)?;
        for (i, s) in mid.iter().enumerate() {
            // "Three and two." is sentence 1, "Thirteen and two." sentence 2.
            if i == 0 {
                kit.until_sentence(stage, "12_8", 1)?;
            } else {
                kit.until_sentence(stage, "12_8", 2)?;
            }
            let x = -3.0 + i as f64 * 6.0;
            for j in 0..2 {
                let line = stage.arena_mut().add(
                    Arrow::new(v(-4.5 + (i * 2 + j) as f64 * 3.0, 1.6), v(x, 0.5))
                        .buff(0.1)
                        .thickness(2.0)
                        .build()?,
                );
                show(stage, &[line], 0.3)?;
                tree_mobs.push(line);
            }
            let t = text(kit, stage, &s.pair(), x, 0.1, Palette::teal())?;
            show(stage, &[t], 0.3)?;
            tree_mobs.push(t);
        }
        kit.until_sentence(stage, "12_8", 3)?;
        for x in [-3.0, 3.0] {
            let line = stage
                .arena_mut()
                .add(Arrow::new(v(x, -0.3), v(0.0, -1.3)).buff(0.1).thickness(2.0).build()?);
            show(stage, &[line], 0.3)?;
            tree_mobs.push(line);
        }
        let root = text(kit, stage, &top.pair(), 0.0, -1.8, Palette::gold())?;
        show(stage, &[root], 0.3)?;
        tree_mobs.push(root);
        kit.hold(stage)?;
        hide(stage, &tree_mobs, 0.3)?;
        kit.say(stage, "12_9")?;
        let log_a = "ab";
        let log_b = "cd";
        let al = text(kit, stage, log_a, -2.0, 1.5, Palette::blue())?;
        let bl = text(kit, stage, log_b, 2.0, 1.5, Palette::teal())?;
        let ordered = text(kit, stage, &format!("{log_a}{log_b}"), 0.0, 0.0, Palette::ink())?;
        let grouping = text(kit, stage, "(a · b) · cd = a · (b · cd)", 0.0, -2.5, Palette::teal())?;
        // The law shows with "Grouping is free."; the swap and its verdict wait for sentence 1, "Reordering is not".
        show(stage, &[al, bl, ordered, grouping], 0.3)?;
        kit.until_sentence(stage, "12_9", 1)?;
        stage.play(
            al.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    path_arc: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .move_to(v(2.0, 1.5), ORIGIN)?,
        )?;
        stage.play(
            bl.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    path_arc: Some(-1.0),
                    ..AnimateArgs::default()
                })?
                .move_to(v(-2.0, 1.5), ORIGIN)?,
        )?;
        let reordered = text(
            kit,
            stage,
            &format!("{log_b}{log_a} ≠ {log_a}{log_b}"),
            0.0,
            -1.3,
            Palette::red(),
        )?;
        show(stage, &[reordered], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[al, bl, ordered, reordered, grouping], 0.3)?;
        kit.say(stage, "12_10")?;
        let programmer = text(
            kit,
            stage,
            "For programmers · IEEE 754 doubles",
            0.0,
            2.5,
            Palette::dim(),
        )?;
        let (x, y, z) = (
            std::hint::black_box(0.1_f64),
            std::hint::black_box(0.2_f64),
            std::hint::black_box(0.3_f64),
        );
        let l = (x + y) + z;
        let r = x + (y + z);
        let fl = text(
            kit,
            stage,
            &format!("({x} + {y}) + {z} = {l:?}"),
            0.0,
            1.0,
            Palette::gold(),
        )?;
        let fr = text(
            kit,
            stage,
            &format!("{x} + ({y} + {z}) = {r:?}"),
            0.0,
            -0.3,
            Palette::teal(),
        )?;
        let caveat = text(
            kit,
            stage,
            "Grouping can change the last bits",
            0.0,
            -2.0,
            Palette::ink(),
        )?;
        show(stage, &[programmer, fl], 0.3)?;
        show(stage, &[fr, caveat], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[programmer, fl, fr, caveat], 0.3)?;
        kit.say(stage, "12_11")?;
        show(stage, &tree_mobs, 0.4)?;
        let monoid = text(
            kit,
            stage,
            "Monoid: associative merge + identity",
            0.0,
            -3.0,
            Palette::ink(),
        )?;
        show(stage, &[monoid], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &tree_mobs, 0.3)?;
        hide(stage, &[monoid], 0.3)?;
        kit.say(stage, "12_12")?;
        let map = Map::build(kit, stage, [0.0, 0.0], 0.8, true)?;
        show(stage, &map.all(), 0.4)?;
        let rich = text(
            kit,
            stage,
            &format!("Sum and count: {}", total.pair()),
            3.0,
            1.8,
            Palette::teal(),
        )?;
        let callback = text(
            kit,
            stage,
            "Time and count\nKeep what later work needs",
            3.0,
            0.0,
            Palette::gold(),
        )?;
        let trap = text(
            kit,
            stage,
            "Unreachable is not an ordinary time",
            3.0,
            -1.8,
            Palette::red(),
        )?;
        show(stage, &[rich, callback, trap], 0.4)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let a = Summary::of(&[1, 2, 3]);
    let b = Summary::of(&[10]);
    for (name, summary) in [
        ("a", a),
        ("b", b),
        ("merged", a.merge(b)),
        ("empty", Summary::of(&[])),
        ("zero", Summary::of(&[0])),
        ("two", Summary::of(&[2])),
        ("one_three", Summary::of(&[1, 3])),
        ("zero_two", Summary::of(&[0, 2])),
        ("zero_one_three", Summary::of(&[0, 1, 3])),
        ("tree_left", Summary::of(&[1]).merge(Summary::of(&[2]))),
        ("tree_right", Summary::of(&[3]).merge(Summary::of(&[10]))),
    ] {
        facts.add(&format!("chunk.{name}.pair"), summary.pair())?;
        facts.add(&format!("chunk.{name}.mean"), summary.fraction())?;
    }
    facts.add(
        "chunk.wrong_mean",
        (a.mean().ok_or("empty a")? + b.mean().ok_or("empty b")?) / 2.0,
    )?;
    let x = std::hint::black_box(0.1_f64);
    let y = std::hint::black_box(0.2_f64);
    let z = std::hint::black_box(0.3_f64);
    facts.add("chunk.float.left.bits", ((x + y) + z).to_bits())?;
    facts.add("chunk.float.right.bits", (x + (y + z)).to_bits())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summaries_preserve_every_split_and_means_do_not() {
        let values = [1, 2, 3, 10];
        let all = Summary::of(&values);
        for i in 0..=values.len() {
            for j in i..=values.len() {
                let a = Summary::of(&values[..i]);
                let b = Summary::of(&values[i..j]);
                let c = Summary::of(&values[j..]);
                assert_eq!(a.merge(b).merge(c), all);
                assert_eq!(a.merge(b.merge(c)), all);
            }
        }
        assert_eq!(all.fraction(), "4");
        assert_eq!(Summary::of(&[]).mean(), None);
        assert_eq!(Summary::of(&[0]).fraction(), "0");
        assert_eq!(Summary::of(&[2]).mean(), Summary::of(&[1, 3]).mean());
        assert_ne!(Summary::of(&[0, 2]).mean(), Summary::of(&[0, 1, 3]).mean());
        let (a, b, c) = (
            std::hint::black_box(0.1_f64),
            std::hint::black_box(0.2_f64),
            std::hint::black_box(0.3_f64),
        );
        assert_ne!(((a + b) + c).to_bits(), (a + (b + c)).to_bits());
    }
}
