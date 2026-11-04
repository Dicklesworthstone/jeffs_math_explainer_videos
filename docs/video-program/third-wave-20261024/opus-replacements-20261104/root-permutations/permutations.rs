//! Draft permutations scene; native and visual review remain open.
use explainer_kit::{Kit, Palette, Timed, ink_style, v};
use fmn::prelude::*;

pub struct Permutations {
    pub kit: Kit,
}
fn fade(stage: &mut Stage<'_>, items: &[Mob], on: bool) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in items {
        let a = if on {
            fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?
        };
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}
fn arrow(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(a, b)
            .buff(0.05)
            .thickness(3.0)
            .style(
                Style::default()
                    .fill(Palette::blue(), 0.9)
                    .stroke(Palette::blue(), 3.0, 0.9),
            )
            .build()?,
    ))
}

// Images of 1,2,3, in zero-based notation. These maps also drive the paths.
const SWAP_12: [usize; 3] = [1, 0, 2];
const CYCLE_123: [usize; 3] = [1, 2, 0];

fn route(origin: usize, first: [usize; 3], second: [usize; 3]) -> [usize; 3] {
    [origin, first[origin], second[first[origin]]]
}

fn lane(column: usize, row: usize) -> Vec3 {
    v(-4.2 + column as f64 * 4.2, 1.1 - row as f64 * 1.1)
}

fn route_board(stage: &mut Stage<'_>, k: &Kit, reverse: bool) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    let labels = if reverse {
        ["Start", "After (1 2 3)", "After (1 2)"]
    } else {
        ["Start", "After (1 2)", "After (1 2 3)"]
    };
    for (column, label) in labels.iter().enumerate() {
        mobs.push(k.text(stage, label, 27.0, Palette::ink(), v(lane(column, 0).x, 2.0))?);
        for row in 0..3 {
            mobs.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.31)
                        .style(Style::default().stroke(Palette::dim(), 2.0, 0.7))
                        .build()
                        .moved_to(lane(column, row)),
                ),
            );
            mobs.push(k.text(stage, &(row + 1).to_string(), 23.0, Palette::dim(), lane(column, row))?);
        }
    }
    fade(stage, &mobs, true)?;
    Ok(mobs)
}

fn trace_route(
    stage: &mut Stage<'_>,
    k: &Kit,
    origin: usize,
    first: [usize; 3],
    second: [usize; 3],
) -> fmn::Result<Vec<Mob>> {
    let rows = route(origin, first, second);
    let color = [Palette::blue(), Palette::hero(), Palette::green()][origin];
    let disk = stage.arena_mut().add(
        Circle::new()
            .radius(0.25)
            .style(ink_style(color))
            .build()
            .moved_to(lane(0, origin)),
    );
    // The moving card keeps its input label. The fixed slot number is its destination.
    let label = k.text(
        stage,
        &(origin + 1).to_string(),
        22.0,
        explainer_kit::hex("#0E1116"),
        lane(0, origin),
    )?;
    let mut mobs = vec![disk, label];
    fade(stage, &mobs, true)?;
    for column in 0..2 {
        let a = lane(column, rows[column]);
        let b = lane(column + 1, rows[column + 1]);
        let path = stage.arena_mut().add(
            Arrow::new(a, b)
                .buff(0.36)
                .thickness(2.0)
                .style(Style::default().fill(color, 0.65).stroke(color, 2.0, 0.65))
                .build()?,
        );
        fade(stage, &[path], true)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for mob in [disk, label] {
            let movement = mob
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.85),
                    ..AnimateArgs::default()
                })?
                .move_to(b, ORIGIN)?;
            moves.push(stage.prepare(movement)?);
        }
        stage.play_prepared(moves)?;
        mobs.push(path);
    }
    Ok(mobs)
}
impl SceneConstruct for Permutations {
    fn name(&self) -> &str {
        "11_permutations"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "11_1")?;
        let title = k.text(
            stage,
            "Permutations: follow the labels",
            36.0,
            Palette::ink(),
            v(0.0, 3.0),
        )?;
        let positions = [(-3.2, 1.3), (4.7, -1.1), (3.2, 1.3), (-3.2, -1.1)];
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let mut scene = vec![title];
        for (i, &(x, y)) in positions.iter().enumerate() {
            let disk = stage.arena_mut().add(
                Circle::new()
                    .radius(0.37)
                    .style(ink_style(colors[i]))
                    .build()
                    .moved_to(v(x, y)),
            );
            let label = k.text(
                stage,
                &(i + 1).to_string(),
                28.0,
                explainer_kit::hex("#0E1116"),
                v(x, y),
            )?;
            scene.extend([disk, label]);
        }
        fade(stage, &scene, true)?;
        k.say(stage, "11_2")?;
        let first = arrow(stage, v(-2.7, 1.3), v(2.7, 1.3))?;
        let second = arrow(stage, v(2.75, 1.0), v(-2.75, -0.8))?;
        let third = arrow(stage, v(-3.6, -0.65), v(-3.6, 0.85))?;
        for a in [first, second, third] {
            fade(stage, &[a], true)?;
            scene.push(a);
        }
        let cycle = k.tex(stage, r"(1\;3\;4)=(3\;4\;1)", 36.0, Palette::green(), &[], v(1.3, -1.7))?;
        fade(stage, &[cycle], true)?;
        scene.push(cycle);
        k.say(stage, "11_3")?;
        fade(stage, &scene, false)?;
        let heading = k.tex(
            stage,
            r"(1\;2\;3)(1\;2)\quad\text{rightmost first}",
            34.0,
            Palette::ink(),
            &[],
            v(0.0, 3.0),
        )?;
        fade(stage, &[heading], true)?;
        let mut composition = route_board(stage, &k, false)?;
        composition.push(heading);
        k.until_sentence(stage, "11_3", 3)?;
        composition.extend(trace_route(stage, &k, 0, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 4)?;
        composition.extend(trace_route(stage, &k, 2, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 5)?;
        composition.extend(trace_route(stage, &k, 1, SWAP_12, CYCLE_123)?);
        k.until_sentence(stage, "11_3", 6)?;
        let result = k.tex(
            stage,
            r"(1\;2\;3)(1\;2)=(1\;3)",
            34.0,
            Palette::green(),
            &[],
            v(0.0, -2.3),
        )?;
        fade(stage, &[result], true)?;
        composition.push(result);
        k.until_sentence(stage, "11_3", 7)?;
        fade(stage, &composition, false)?;
        composition = route_board(stage, &k, true)?;
        let reverse_title = k.text(stage, "Reverse the operations", 34.0, Palette::ink(), v(0.0, 3.0))?;
        fade(stage, &[reverse_title], true)?;
        composition.push(reverse_title);
        for origin in 0..3 {
            composition.extend(trace_route(stage, &k, origin, CYCLE_123, SWAP_12)?);
        }
        let reverse_result = k.tex(
            stage,
            r"(1\;2)(1\;2\;3)=(2\;3)",
            34.0,
            Palette::hero(),
            &[],
            v(0.0, -2.3),
        )?;
        fade(stage, &[reverse_result], true)?;
        composition.push(reverse_result);
        k.hold(stage)?;
        stage.wait(1.5)?;
        k.say(stage, "11_4")?;
        fade(stage, &composition, false)?;
        let inverse = k.tex(
            stage,
            r"(1\;3\;4)^{-1}=(1\;4\;3)",
            40.0,
            Palette::ink(),
            &[],
            v(0.0, 1.1),
        )?;
        let disjoint = k.text(
            stage,
            "Separate loops do not interfere",
            32.0,
            Palette::green(),
            v(0.0, -0.2),
        )?;
        fade(stage, &[inverse, disjoint], true)?;
        k.say(stage, "11_5")?;
        fade(stage, &[inverse, disjoint], false)?;
        let factor = k.tex(
            stage,
            r"(1\;3\;4)=(1\;4)(1\;3)",
            40.0,
            Palette::hero(),
            &[],
            v(0.0, 1.0),
        )?;
        let count = k.text(
            stage,
            "A length-m cycle can be built from m − 1 swaps",
            31.0,
            Palette::ink(),
            v(0.0, -0.3),
        )?;
        fade(stage, &[factor, count], true)?;
        k.say(stage, "11_6")?;
        let question = k.text(
            stage,
            "Can the same move be both even and odd?",
            31.0,
            Palette::blue(),
            v(0.0, -1.65),
        )?;
        fade(stage, &[question], true)?;
        k.hold(stage)?;
        stage.wait(3.0)?;
        k.say(stage, "11_7")?;
        fade(stage, &[factor, count, question], false)?;
        let product = k.tex(
            stage,
            r"\Delta(x)=\prod_{i<j}(x_j-x_i)",
            40.0,
            Palette::ink(),
            &[],
            v(0.0, 2.0),
        )?;
        let example = k.tex(
            stage,
            r"(x_2-x_1)(x_3-x_1)(x_3-x_2)",
            34.0,
            Palette::blue(),
            &[],
            v(0.0, 0.9),
        )?;
        let swapped = k.tex(
            stage,
            r"(x_1-x_2)(x_3-x_2)(x_3-x_1)",
            34.0,
            Palette::hero(),
            &[],
            v(0.0, -0.1),
        )?;
        let explanation = k.text(
            stage,
            "First factor reverses; the other two exchange",
            27.0,
            Palette::green(),
            v(0.0, -1.1),
        )?;
        let sign = k.tex(
            stage,
            r"\text{one swap}:\quad \Delta\mapsto-\Delta",
            38.0,
            Palette::hero(),
            &[],
            v(0.0, -2.0),
        )?;
        fade(stage, &[product], true)?;
        k.until_sentence(stage, "11_7", 2)?;
        fade(stage, &[example], true)?;
        k.until_sentence(stage, "11_7", 3)?;
        fade(stage, &[swapped], true)?;
        k.until_sentence(stage, "11_7", 4)?;
        fade(stage, &[explanation], true)?;
        fade(stage, &[sign], true)?;
        k.say(stage, "11_8")?;
        fade(stage, &[example, swapped, explanation, sign], false)?;
        let parity = k.tex(
            stage,
            r"\Delta_{\text{final}}=(-1)^N\Delta_{\text{initial}}",
            34.0,
            Palette::green(),
            &[],
            v(0.0, -1.2),
        )?;
        let swap_count = k.text(stage, "N = number of swaps", 26.0, Palette::green(), v(0.0, -2.15))?;
        fade(stage, &[parity, swap_count], true)?;
        k.say(stage, "11_9")?;
        fade(stage, &[product, parity, swap_count], false)?;
        let square = k.text(
            stage,
            "4 corners: 24 permutations, 8 square symmetries",
            32.0,
            Palette::ink(),
            v(0.0, 1.0),
        )?;
        let turn = k.text(stage, "Quarter-turn: 4-cycle, odd", 31.0, Palette::blue(), v(0.0, 0.0))?;
        let half = k.text(stage, "Half-turn: two swaps, even", 31.0, Palette::hero(), v(0.0, -1.0))?;
        fade(stage, &[square, turn, half], true)?;
        k.say(stage, "11_10")?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;

    #[test]
    fn displayed_compositions_have_different_endpoints_and_no_collisions() {
        let forward = (0..3).map(|i| route(i, SWAP_12, CYCLE_123)).collect::<Vec<_>>();
        let reverse = (0..3).map(|i| route(i, CYCLE_123, SWAP_12)).collect::<Vec<_>>();
        assert_eq!(forward, vec![[0, 1, 2], [1, 0, 1], [2, 2, 0]]);
        assert_eq!(reverse, vec![[0, 1, 0], [1, 2, 2], [2, 0, 1]]);
        for routes in [&forward, &reverse] {
            for column in 0..3 {
                let mut destinations = routes.iter().map(|r| r[column]).collect::<Vec<_>>();
                destinations.sort();
                assert_eq!(destinations, vec![0, 1, 2]);
            }
        }
    }
}
