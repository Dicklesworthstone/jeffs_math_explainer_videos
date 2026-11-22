//! The same road identities become layer edges, then matrix entries.
use super::draw::{Map, dot, hide, show};
use crate::model::{self, Arithmetic, Count, Error, PLACES, Quickest, ROADS};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter07 {
    pub kit: Kit,
}
type Matrix<V> = Vec<Vec<V>>;

fn adjacency<A: Arithmetic>() -> Result<Matrix<A::Value>, Error> {
    let mut a = vec![vec![A::zero(); 4]; 4];
    for &road in &ROADS {
        a[road.from][road.to] = A::combine(&a[road.from][road.to], &A::road(road))?;
    }
    Ok(a)
}
fn multiply<A: Arithmetic>(a: &Matrix<A::Value>, b: &Matrix<A::Value>) -> Result<Matrix<A::Value>, Error> {
    let mut c = vec![vec![A::zero(); 4]; 4];
    for (i, row) in c.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            for (k, arow) in a[i].iter().enumerate() {
                *cell = A::combine(cell, &A::extend(arow, &b[k][j])?)?;
            }
        }
    }
    Ok(c)
}
fn powers<A: Arithmetic>() -> Result<Vec<Matrix<A::Value>>, Error> {
    let a = adjacency::<A>()?;
    let mut result = vec![a.clone()];
    for _ in 1..4 {
        result.push(multiply::<A>(result.last().unwrap(), &a)?);
    }
    Ok(result)
}
fn number(n: Option<u64>) -> String {
    n.map_or_else(|| "·".into(), |n| n.to_string())
}
fn layer_point(k: usize, i: usize) -> Vec3 {
    v(-4.8 + k as f64 * 3.2, 1.8 - i as f64 * 1.2)
}
fn cell_point(i: usize, j: usize) -> Vec3 {
    v(-4.2 + j as f64 * 1.2, 1.8 - i as f64 * 1.2)
}
fn panel(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 27.0, color, v(3.0, y))
}
struct Grid {
    values: Vec<Mob>,
}
impl Grid {
    fn build(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Self> {
        for (i, name) in PLACES.iter().enumerate() {
            let row = kit.text(stage, name, 25.0, Palette::ink(), v(-5.7, cell_point(i, 0)[1]))?;
            let col = kit.text(stage, name, 25.0, Palette::ink(), v(cell_point(0, i)[0], 2.7))?;
            show(stage, &[row, col], 0.2)?;
        }
        let mut values = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                let p = cell_point(i, j);
                let frame = stage.arena_mut().add(
                    Rectangle::new()
                        .width(1.1)
                        .height(1.0)
                        .style(Style::default().stroke(Palette::dim(), 1.0, 1.0))
                        .build()?
                        .moved_to(p),
                );
                let label = kit.text(stage, "·", 30.0, Palette::dim(), p)?;
                show(stage, &[frame, label], 0.05)?;
                values.push(label);
            }
        }
        Ok(Self { values })
    }
    fn set(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        i: usize,
        j: usize,
        value: &str,
        color: Srgb,
    ) -> fmn::Result<()> {
        hide(stage, &[self.values[i * 4 + j]], 0.08)?;
        self.values[i * 4 + j] = kit.text(stage, value, 30.0, color, cell_point(i, j))?;
        show(stage, &[self.values[i * 4 + j]], 0.08)
    }
    fn fill(&mut self, kit: &Kit, stage: &mut Stage<'_>, matrix: &Matrix<Option<u64>>) -> fmn::Result<()> {
        self.replace(
            kit,
            stage,
            matrix.iter().flatten().map(|cell| {
                (
                    number(*cell),
                    if cell.is_some() {
                        Palette::gold()
                    } else {
                        Palette::dim()
                    },
                )
            }),
        )
    }
    fn replace(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        entries: impl IntoIterator<Item = (String, Srgb)>,
    ) -> fmn::Result<()> {
        // A completed matrix represents one road budget, never a hybrid of two.
        hide(stage, &self.values, 0.2)?;
        let mut next = Vec::new();
        for (index, (value, color)) in entries.into_iter().enumerate() {
            next.push(kit.text(stage, &value, 30.0, color, cell_point(index / 4, index % 4))?);
        }
        assert_eq!(next.len(), self.values.len());
        self.values = next;
        show(stage, &self.values, 0.25)
    }
}
impl SceneConstruct for Chapter07 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let layers = model::layers::<Quickest>(4, &ROADS, 0, 3).expect("bounded graph");
        let quick = powers::<Quickest>().expect("small matrix");
        let count = powers::<Count>().expect("small matrix");
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        kit.say(stage, "07_1")?;
        show(stage, &map.all(), 0.4)?;
        let question = panel(kit, stage, "At most TWO roads", 0.0, Palette::gold())?;
        show(stage, &[question], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "07_2")?;
        hide(stage, &[question], 0.25)?;
        hide(stage, &map.roads, 0.3)?;
        hide(stage, &map.costs, 0.3)?;
        hide(stage, &map.names, 0.3)?;
        let mut layer_mobs = Vec::new();
        for (i, &node) in map.nodes.iter().enumerate() {
            stage.play(
                node.animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.4),
                        ..AnimateArgs::default()
                    })?
                    .move_to(layer_point(0, i), ORIGIN)?,
            )?;
            layer_mobs.push(node);
            let name = kit.text(stage, PLACES[i], 25.0, Palette::ink(), v(-6.0, layer_point(0, i)[1]))?;
            show(stage, &[name], 0.15)?;
            layer_mobs.push(name);
        }
        let mut last_column = Vec::new();
        for k in 0..4 {
            let heading = kit.text(
                stage,
                &format!("{k} {}", if k == 1 { "road" } else { "roads" }),
                29.0,
                Palette::ink(),
                v(layer_point(k, 0)[0], 2.8),
            )?;
            show(stage, &[heading], 0.2)?;
            layer_mobs.push(heading);
            if k == 3 {
                last_column.push(heading);
            }
            if k > 0 {
                for road in ROADS {
                    let edge = stage.arena_mut().add(
                        Arrow::new(layer_point(k - 1, road.from), layer_point(k, road.to))
                            .buff(0.18)
                            .thickness(1.0)
                            .style(
                                Style::default()
                                    .fill(Palette::dim(), 0.55)
                                    .stroke(Palette::dim(), 1.0, 0.55),
                            )
                            .build()?,
                    );
                    show(stage, &[edge], 0.08)?;
                    layer_mobs.push(edge);
                    if k == 3 {
                        last_column.push(edge);
                    }
                }
                for i in 0..4 {
                    let node = dot(stage, layer_point(k, i), 0.09, Palette::ink());
                    show(stage, &[node], 0.08)?;
                    layer_mobs.push(node);
                    if k == 3 {
                        last_column.push(node);
                    }
                }
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "07_3")?;
        for (k, layer) in layers.iter().enumerate() {
            for (i, &value) in layer.iter().enumerate() {
                if let Some(value) = value {
                    let p = layer_point(k, i);
                    let label = kit.text(
                        stage,
                        &value.to_string(),
                        25.0,
                        Palette::gold(),
                        v(p[0] + 0.27, p[1] + 0.3),
                    )?;
                    show(stage, &[label], 0.2)?;
                    layer_mobs.push(label);
                    if k == 3 {
                        last_column.push(label);
                    }
                }
            }
        }
        kit.until_sentence(stage, "07_3", 3)?;
        hide(stage, &last_column, 0.5)?;
        // Already gone: a second fade at 07_4 would put the three-road column back on screen.
        layer_mobs.retain(|m| !last_column.contains(m));
        let best = layers[..=2]
            .iter()
            .filter_map(|row| row[3])
            .min()
            .expect("two-road route");
        let chosen = kit.text(
            stage,
            &format!("At most two roads: {best}"),
            32.0,
            Palette::teal(),
            v(0.0, -2.9),
        )?;
        show(stage, &[chosen], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "07_4")?;
        hide(stage, &layer_mobs, 0.4)?;
        hide(stage, &[chosen], 0.3)?;
        let mut grid = Grid::build(kit, stage)?;
        let mut title = kit.text(stage, "A: exactly one road", 30.0, Palette::ink(), v(-2.4, 3.35))?;
        show(stage, &[title], 0.3)?;
        for road in ROADS {
            let source = kit.text(
                stage,
                &format!("{} → {}: {}", PLACES[road.from], PLACES[road.to], road.value),
                27.0,
                Palette::gold(),
                v(3.0, 0.0),
            )?;
            show(stage, &[source], 0.15)?;
            let traveling = kit.text(stage, &road.value.to_string(), 30.0, Palette::gold(), v(3.0, -0.7))?;
            show(stage, &[traveling], 0.15)?;
            stage.play(
                traveling
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.55),
                        ..AnimateArgs::default()
                    })?
                    .move_to(cell_point(road.from, road.to), ORIGIN)?,
            )?;
            hide(stage, &[traveling], 0.05)?;
            grid.set(kit, stage, road.from, road.to, &road.value.to_string(), Palette::gold())?;
            hide(stage, &[source], 0.1)?;
        }
        let legend = panel(
            kit,
            stage,
            "Rows: leave\nColumns: reach\n· means no road",
            0.0,
            Palette::ink(),
        )?;
        show(stage, &[legend], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[legend], 0.3)?;
        kit.say(stage, "07_5")?;
        let row = stage.arena_mut().add(
            Rectangle::new()
                .width(4.7)
                .height(1.05)
                .style(Style::default().stroke(Palette::teal(), 2.5, 1.0))
                .build()?
                .moved_to(v(-2.4, 1.8)),
        );
        let col = stage.arena_mut().add(
            Rectangle::new()
                .width(1.15)
                .height(4.65)
                .style(Style::default().stroke(Palette::gold(), 2.5, 1.0))
                .build()?
                .moved_to(v(-0.6, 0.0)),
        );
        show(stage, &[row, col], 0.3)?;
        let mut offers = Vec::new();
        for k in 0..4 {
            let a = quick[0][0][k];
            let b = quick[0][k][3];
            let value = Quickest::extend(&a, &b).unwrap();
            let s = format!("{}: {} + {} → {}", PLACES[k], number(a), number(b), number(value));
            let label = panel(
                kit,
                stage,
                &s,
                1.7 - k as f64 * 0.8,
                if value.is_some() {
                    Palette::gold()
                } else {
                    Palette::dim()
                },
            )?;
            show(stage, &[label], 0.4)?;
            offers.push(label);
        }
        let answer = panel(
            kit,
            stage,
            &format!("Keep the smaller: {}", number(quick[1][0][3])),
            -2.0,
            Palette::teal(),
        )?;
        show(stage, &[answer], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "07_6")?;
        let programmer = kit.text(stage, "For programmers", 23.0, Palette::dim(), v(0.0, -2.9))?;
        let formula = kit.tex(
            stage,
            r"(A\otimes A)_{ij}=\bigoplus_k A_{ik}\otimes A_{kj}",
            29.0,
            Palette::ink(),
            &[],
            v(0.0, -3.4),
        )?;
        show(stage, &[programmer, formula], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &offers, 0.3)?;
        hide(stage, &[row, col, answer, programmer, formula], 0.3)?;
        kit.say(stage, "07_7")?;
        for (index, matrix) in quick.iter().enumerate().skip(2) {
            hide(stage, &[title], 0.2)?;
            grid.fill(kit, stage, matrix)?;
            title = kit.text(
                stage,
                &format!("Exactly {} roads", index + 1),
                30.0,
                Palette::ink(),
                v(-2.4, 3.35),
            )?;
            show(stage, &[title], 0.2)?;
            if index == 2 {
                kit.until_sentence(stage, "07_7", 1)?;
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "07_8")?;
        hide(stage, &[title], 0.2)?;
        let mut summaries = Vec::new();
        for (index, matrix) in count.iter().enumerate().take(3) {
            if index > 0 {
                hide(stage, &[title], 0.2)?;
            }
            grid.replace(
                kit,
                stage,
                matrix
                    .iter()
                    .flatten()
                    .map(|value| (value.to_string(), Palette::teal())),
            )?;
            title = kit.text(
                stage,
                &format!(
                    "Counts: exactly {} road{}",
                    index + 1,
                    if index == 0 { "" } else { "s" }
                ),
                30.0,
                Palette::teal(),
                v(-2.4, 3.35),
            )?;
            show(stage, &[title], 0.3)?;
            let s = panel(
                kit,
                stage,
                &format!("{} road(s): {}", index + 1, matrix[0][3]),
                1.4 - index as f64,
                Palette::teal(),
            )?;
            show(stage, &[s], 0.3)?;
            summaries.push(s);
        }
        let total: u64 = count.iter().take(3).map(|m| m[0][3]).sum();
        let sum = panel(kit, stage, &format!("Total: {total}"), -1.9, Palette::teal())?;
        show(stage, &[sum], 0.3)?;
        kit.hold(stage)?;
        kit.say(stage, "07_9")?;
        hide(stage, &summaries, 0.3)?;
        hide(stage, &[sum, title], 0.3)?;
        grid.fill(kit, stage, &quick[3])?;
        let final_title = kit.text(stage, "Four roads: no route", 30.0, Palette::ink(), v(-2.4, 3.35))?;
        show(stage, &[final_title], 0.3)?;
        kit.finish(stage)
    }
}

pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let quick = powers::<Quickest>().map_err(|e| format!("{e:?}"))?;
    let counts = powers::<Count>().map_err(|e| format!("{e:?}"))?;
    for (k, matrix) in quick.iter().enumerate() {
        for (i, row) in matrix.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                facts.add(&format!("matrix.quick.{}.{}.{}", k + 1, i, j), number(*value))?;
                facts.add(&format!("matrix.count.{}.{}.{}", k + 1, i, j), counts[k][i][j])?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_powers_agree_with_layers_from_every_start() {
        let quick = powers::<Quickest>().unwrap();
        let count = powers::<Count>().unwrap();
        for start in 0..4 {
            let q = model::layers::<Quickest>(4, &ROADS, start, 4).unwrap();
            let c = model::layers::<Count>(4, &ROADS, start, 4).unwrap();
            for k in 1..=4 {
                assert_eq!(quick[k - 1][start], q[k]);
                assert_eq!(count[k - 1][start], c[k]);
            }
        }
        assert_eq!(
            quick.iter().map(|m| m[0][3]).collect::<Vec<_>>(),
            [None, Some(8), Some(6), None]
        );
        assert_eq!(count.iter().map(|m| m[0][3]).collect::<Vec<_>>(), [0, 2, 1, 0]);
        assert!(quick[3].iter().flatten().all(Option::is_none));
    }
}
