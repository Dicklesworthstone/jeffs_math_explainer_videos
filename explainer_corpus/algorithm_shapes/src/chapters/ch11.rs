//! History at input positions, shared states, and a constant-size summary.
use super::draw::{dot, hide, prediction, route_color, show};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter11 {
    pub kit: Kit,
}
const TOP: usize = 10;
fn ways(n: usize) -> Vec<u64> {
    let mut a = vec![1; n + 1];
    for i in 2..=n {
        a[i] = a[i - 1] + a[i - 2];
    }
    a
}
#[derive(Debug)]
struct Question {
    n: usize,
    parent: Option<usize>,
    depth: usize,
    point: Vec3,
}
fn tree(n: usize) -> Vec<Question> {
    fn grow(n: usize, parent: Option<usize>, depth: usize, left: f64, right: f64, out: &mut Vec<Question>) {
        let index = out.len();
        out.push(Question {
            n,
            parent,
            depth,
            point: v((left + right) / 2.0, 2.5 - depth as f64 * 0.52),
        });
        if n > 1 {
            let a = ways(n);
            let split = left + (right - left) * a[n - 1] as f64 / a[n] as f64;
            grow(n - 1, Some(index), depth + 1, left, split, out);
            grow(n - 2, Some(index), depth + 1, split, right, out);
        }
    }
    let mut out = Vec::new();
    grow(n, None, 0, -6.0, 2.4, &mut out);
    out
}
fn landing(i: usize) -> Vec3 {
    v(-5.7 + i as f64 * 1.1, -2.2 + i as f64 * 0.38)
}
fn border(stage: &mut Stage<'_>, center: Vec3, width: f64, height: f64, color: Srgb) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new()
            .width(width)
            .height(height)
            .style(Style::default().stroke(color, 2.0, 1.0))
            .build()?
            .moved_to(center),
    ))
}
fn step_table(a: &str, b: &str) -> Vec<Vec<usize>> {
    let a: Vec<_> = a.chars().collect();
    let b: Vec<_> = b.chars().collect();
    let mut d = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]));
        }
    }
    d
}
impl SceneConstruct for Chapter11 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let counts = ways(TOP);
        let questions = tree(TOP);
        kit.say(stage, "11_1")?;
        let heading = kit.text(stage, "One step or two?", 39.0, Palette::ink(), v(0.0, 3.0))?;
        show(stage, &[heading], 0.3)?;
        let mut stairs = Vec::new();
        for i in 0..=TOP {
            let p = landing(i);
            let node = dot(stage, p, 0.08, Palette::ink());
            let label = kit.text(stage, &i.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.85))?;
            stairs.extend([node, label]);
            if i > 0 {
                let last = landing(i - 1);
                let tread = stage
                    .arena_mut()
                    .add(Line::new(v(last[0], last[1] - 0.15), v(p[0], last[1] - 0.15)).build()?);
                let riser = stage
                    .arena_mut()
                    .add(Line::new(v(p[0], last[1] - 0.15), v(p[0], p[1] - 0.15)).build()?);
                stairs.extend([tread, riser]);
            }
        }
        show(stage, &stairs, 0.6)?;
        let walker = dot(stage, landing(0), 0.12, Palette::gold());
        show(stage, &[walker], 0.2)?;
        for i in [1, 3, 4, 6, 8, 10] {
            stage.play(
                walker
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.35),
                        path_arc: Some(-0.8),
                        ..AnimateArgs::default()
                    })?
                    .move_to(landing(i), ORIGIN)?,
            )?;
        }
        kit.hold(stage)?;
        hide(stage, &[walker], 0.2)?;
        prediction(kit, stage, 6, v(3.0, -1.5))?;
        kit.say(stage, "11_2")?;
        let mut badges = Vec::new();
        for (i, value) in counts.iter().enumerate() {
            // Bottom on sentence 2, first step on 3, the adding climb on 4.
            match i {
                0 => kit.until_sentence(stage, "11_2", 2)?,
                1 => kit.until_sentence(stage, "11_2", 3)?,
                2 => kit.until_sentence(stage, "11_2", 4)?,
                _ => {}
            }
            let p = landing(i);
            if i >= 2 {
                let mut tokens = Vec::new();
                let mut token_mobs = Vec::new();
                for j in [i - 2, i - 1] {
                    let token = dot(stage, landing(j), 0.07, Palette::teal());
                    show(stage, &[token], 0.1)?;
                    token_mobs.push(token);
                    tokens.push(
                        stage.prepare(
                            token
                                .animate()
                                .set_anim_args(AnimateArgs {
                                    run_time: Some(0.4),
                                    path_arc: Some(-0.5),
                                    ..AnimateArgs::default()
                                })?
                                .move_to(p, ORIGIN)?,
                        )?,
                    );
                }
                stage.play_prepared(tokens)?;
                hide(stage, &token_mobs, 0.1)?;
            }
            let label = kit.text(stage, &value.to_string(), 30.0, Palette::gold(), v(p[0], p[1] + 0.45))?;
            show(stage, &[label], 0.18)?;
            badges.push(label);
        }
        kit.hold(stage)?;
        kit.say(stage, "11_3")?;
        hide(stage, &[heading], 0.25)?;
        let name = kit.text(
            stage,
            "Fibonacci: add the previous two",
            36.0,
            Palette::teal(),
            v(0.0, 3.0),
        )?;
        show(stage, &[name], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &stairs, 0.3)?;
        hide(stage, &badges, 0.3)?;
        hide(stage, &[name], 0.2)?;
        kit.say(stage, "11_4")?;
        let mut tree_mobs = Vec::new();
        for depth in 0..=TOP {
            let mut level = Vec::new();
            for q in questions.iter().filter(|q| q.depth == depth) {
                if let Some(parent) = q.parent {
                    let line = stage.arena_mut().add(
                        Line::new(questions[parent].point, q.point)
                            .style(Style::default().stroke(Palette::dim(), 0.7, 1.0))
                            .build()?,
                    );
                    level.push(line);
                }
                level.push(dot(
                    stage,
                    q.point,
                    0.034,
                    if q.n == 8 { Palette::gold() } else { Palette::teal() },
                ));
                if depth <= 2 {
                    level.push(kit.text(
                        stage,
                        &q.n.to_string(),
                        20.0,
                        Palette::ink(),
                        v(q.point[0], q.point[1] + 0.16),
                    )?);
                }
            }
            if !level.is_empty() {
                show(stage, &level, 0.25)?;
                tree_mobs.extend(level);
            }
        }
        let mut summary = Vec::new();
        summary.push(kit.text(
            stage,
            &format!("{} questions\n{} ways", questions.len(), counts[TOP]),
            32.0,
            Palette::ink(),
            v(4.5, 2.0),
        )?);
        for (row, n) in [8, 7, 6].into_iter().enumerate() {
            let repeats = questions.iter().filter(|q| q.n == n).count();
            summary.push(kit.text(
                stage,
                &format!("Step {n}: asked {repeats} times"),
                25.0,
                Palette::gold(),
                v(4.4, 0.6 - row as f64 * 0.8),
            )?);
        }
        kit.until_sentence(stage, "11_4", 3)?;
        show(stage, &summary[..1], 0.3)?;
        kit.until_sentence(stage, "11_4", 4)?;
        show(stage, &summary[1..], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "11_5")?;
        hide(stage, &tree_mobs, 0.5)?;
        hide(stage, &summary, 0.3)?;
        show(stage, &stairs, 0.4)?;
        show(stage, &badges, 0.4)?;
        let once = kit.text(
            stage,
            &format!("{} landings · {} stored answers", counts.len(), counts.len()),
            35.0,
            Palette::teal(),
            v(0.0, 3.0),
        )?;
        show(stage, &[once], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "11_6")?;
        let mut histories = Vec::new();
        for i in 0..=TOP {
            let p = landing(i);
            histories.push(border(stage, v(p[0], p[1] + 0.45), 0.85, 0.65, Palette::teal())?);
        }
        show(stage, &histories, 0.5)?;
        let lookback = kit.text(
            stage,
            "History stays attached to each earlier landing",
            29.0,
            Palette::ink(),
            v(0.0, -3.2),
        )?;
        show(stage, &[lookback], 0.3)?;
        kit.until_sentence(stage, "11_6", 2)?;
        hide(stage, &[once], 0.2)?;
        let histo = kit.text(
            stage,
            "Histomorphism: a fold with history",
            34.0,
            Palette::teal(),
            v(0.0, 3.0),
        )?;
        show(stage, &[histo], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "11_7")?;
        let mut prefix = Vec::new();
        for end in [4, 7, 10] {
            hide(stage, &prefix, 0.2)?;
            prefix.clear();
            for i in 0..=end {
                prefix.push(dot(stage, landing(i), 0.12, route_color(1)));
            }
            show(stage, &prefix, 0.3)?;
            stage.wait(0.6)?;
        }
        kit.hold(stage)?;
        hide(stage, &prefix, 0.2)?;
        hide(stage, &histories, 0.2)?;
        hide(stage, &[lookback, histo], 0.2)?;
        kit.say(stage, "11_8")?;
        let pair_label = kit.text(stage, "Keep only the last two", 35.0, Palette::gold(), v(0.0, 3.0))?;
        show(stage, &[pair_label], 0.3)?;
        let mut pair_mobs = Vec::new();
        for i in 1..=TOP {
            // The pair starts sliding on sentence 2 and reaches the top on sentence 4.
            if i == 1 {
                kit.until_sentence(stage, "11_8", 2)?;
            } else if i == TOP {
                kit.until_sentence(stage, "11_8", 4)?;
            }
            hide(stage, &pair_mobs, 0.12)?;
            pair_mobs.clear();
            for j in [i - 1, i] {
                let p = landing(j);
                pair_mobs.push(border(stage, v(p[0], p[1] + 0.45), 0.85, 0.65, Palette::gold())?);
            }
            pair_mobs.push(kit.text(
                stage,
                &format!("(latest, previous) = ({}, {})", counts[i], counts[i - 1]),
                31.0,
                Palette::gold(),
                v(0.0, -3.2),
            )?);
            show(stage, &pair_mobs, 0.22)?;
        }
        kit.hold(stage)?;
        hide(stage, &stairs, 0.3)?;
        hide(stage, &badges, 0.3)?;
        hide(stage, &pair_mobs, 0.3)?;
        hide(stage, &[pair_label], 0.2)?;
        kit.say(stage, "11_9")?;
        let p1 = v(-3.0, 1.0);
        let p2 = v(3.0, 1.0);
        let mut duplicated = Vec::new();
        for (p, position) in [(p1, "Tree position A"), (p2, "Tree position B")] {
            duplicated.push(border(stage, p, 3.9, 2.2, Palette::teal())?);
            duplicated.push(kit.text(stage, position, 27.0, Palette::ink(), v(p[0], p[1] + 0.65))?);
            duplicated.push(kit.text(stage, "Hill", 38.0, Palette::gold(), p)?);
            duplicated.push(kit.text(stage, "its own history", 25.0, Palette::teal(), v(p[0], p[1] - 0.65))?);
        }
        duplicated.push(kit.text(
            stage,
            "Equal questions are still distinct positions",
            31.0,
            Palette::ink(),
            v(0.0, -1.7),
        )?);
        show(stage, &duplicated, 0.4)?;
        kit.hold(stage)?;
        hide(stage, &duplicated, 0.3)?;
        kit.say(stage, "11_10")?;
        let a = "cat";
        let b = "cut";
        let distances = step_table(a, b);
        let cell = |i: usize, j: usize| v(-3.0 + j as f64 * 1.4, 1.6 - i as f64 * 1.1);
        let mut grid = Vec::new();
        for i in 0..=a.len() {
            grid.push(kit.text(
                stage,
                if i == 0 { "empty" } else { &a[..i] },
                26.0,
                Palette::ink(),
                v(-4.4, cell(i, 0)[1]),
            )?);
        }
        for j in 0..=b.len() {
            grid.push(kit.text(
                stage,
                if j == 0 { "empty" } else { &b[..j] },
                26.0,
                Palette::ink(),
                v(cell(0, j)[0], 2.5),
            )?);
        }
        for (i, row) in distances.iter().enumerate() {
            for j in 0..row.len() {
                grid.push(border(stage, cell(i, j), 1.3, 1.0, Palette::dim())?);
            }
        }
        show(stage, &grid, 0.4)?;
        // Script08 11_10: question cells only; no answer is computed on screen.
        let illustration = kit.text(
            stage,
            "Illustration: the questions, not their answers",
            27.0,
            Palette::dim(),
            v(0.0, -3.0),
        )?;
        show(stage, &[illustration], 0.3)?;
        let count = distances.iter().map(Vec::len).sum::<usize>();
        kit.until_sentence(stage, "11_10", 2)?;
        let note = kit.text(
            stage,
            &format!("{count} prefix pairs\nNow order the questions"),
            29.0,
            Palette::teal(),
            v(4.2, 1.0),
        )?;
        show(stage, &[note], 0.3)?;
        kit.until_sentence(stage, "11_10", 3)?;
        // One cell asks its left, upper and diagonal neighbours.
        let (ci, cj) = (a.len(), b.len());
        let mut asks = Vec::new();
        for (di, dj) in [(0, 1), (1, 0), (1, 1)] {
            let arrow = stage.arena_mut().add(
                Arrow::new(cell(ci, cj), cell(ci - di, cj - dj))
                    .buff(0.3)
                    .thickness(2.0)
                    .style(
                        Style::default()
                            .fill(Palette::teal(), 1.0)
                            .stroke(Palette::teal(), 2.0, 1.0),
                    )
                    .build()?,
            );
            kit.allow_crossing(arrow);
            asks.push(arrow);
        }
        show(stage, &asks, 0.3)?;
        grid.extend(asks);
        // Row by row puts every neighbour before the cell that asks it.
        for i in 0..=a.len() {
            for j in 0..=b.len() {
                let mark = border(stage, cell(i, j), 1.3, 1.0, Palette::teal())?;
                show(stage, &[mark], 0.1)?;
                hide(stage, &[mark], 0.1)?;
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "11_11")?;
        hide(stage, &grid, 0.3)?;
        hide(stage, &[note, illustration], 0.3)?;
        let list_point = |index: usize| v(-5.4 + index as f64 * 0.72, 0.4);
        let mut history = Vec::new();
        for index in 0..count {
            history.push(border(stage, list_point(index), 0.66, 0.75, Palette::dim())?);
        }
        let list_title = kit.text(
            stage,
            &format!("The same {count} cells, stored as one linked list, row by row"),
            32.0,
            Palette::ink(),
            v(0.0, 2.3),
        )?;
        show(stage, &history, 0.4)?;
        show(stage, &[list_title], 0.3)?;
        let current = count - 1;
        let above = current - distances[0].len();
        // Name only the two cells the look-back joins.
        let ends = [
            kit.text(
                stage,
                &format!("{a}, {b}"),
                20.0,
                Palette::ink(),
                v(list_point(current)[0], 1.1),
            )?,
            kit.text(
                stage,
                &format!("{}, {b}", &a[..a.len() - 1]),
                20.0,
                Palette::ink(),
                v(list_point(above)[0], 1.1),
            )?,
        ];
        show(stage, &ends, 0.3)?;
        let pointer = dot(stage, v(list_point(current)[0], -0.3), 0.1, Palette::teal());
        show(stage, &[pointer], 0.2)?;
        for index in (above..current).rev() {
            stage.play(
                pointer
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.4),
                        ..AnimateArgs::default()
                    })?
                    .move_to(v(list_point(index)[0], -0.3), ORIGIN)?,
            )?;
        }
        let looked_back = kit.text(
            stage,
            "To reach the cell above, walk back through the list",
            27.0,
            Palette::teal(),
            v(0.0, -1.3),
        )?;
        show(stage, &[looked_back], 0.3)?;
        let limit = kit.text(
            stage,
            "Access cost depends on how history is stored",
            30.0,
            Palette::ink(),
            v(0.0, -3.1),
        )?;
        show(stage, &[limit], 0.3)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}
pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let answers = ways(TOP);
    let questions = tree(TOP);
    for (n, answer) in answers.iter().enumerate() {
        facts.add(&format!("history.ways.{n}"), answer)?;
        facts.add(
            &format!("history.asks.{n}"),
            questions.iter().filter(|q| q.n == n).count(),
        )?;
    }
    facts.add("history.calls", questions.len())?;
    facts.add("history.distinct", answers.len())?;
    let distances = step_table("cat", "cut");
    facts.add("history.grid.cells", distances.iter().map(Vec::len).sum::<usize>())?;
    for (i, row) in distances.iter().enumerate() {
        for (j, answer) in row.iter().enumerate() {
            facts.add(&format!("history.edit.{i}.{j}"), answer)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stairs_call_tree_and_prefix_grid_keep_different_identities() {
        let a = ways(TOP);
        let q = tree(TOP);
        assert_eq!(a, [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
        assert_eq!(q.len(), 177);
        for (n, repeats) in [(8, 2), (7, 3), (6, 5)] {
            assert_eq!(q.iter().filter(|x| x.n == n).count(), repeats);
        }
        let mut pair = (1, 1);
        for _ in 2..=TOP {
            pair = (pair.0 + pair.1, pair.0);
        }
        assert_eq!(pair, (a[TOP], a[TOP - 1]));
        for (i, x) in q.iter().enumerate() {
            if let Some(p) = x.parent {
                assert!(p < i);
                assert!(q[p].n > x.n);
            }
        }
        let d = step_table("cat", "cut");
        assert_eq!(d.iter().map(Vec::len).sum::<usize>(), 16);
        assert_eq!(d[3][3], 1);
        assert_eq!(step_table("", "cat")[0][3], 3);
        assert_eq!(step_table("cat", "cat")[3][3], 0);
    }
}
