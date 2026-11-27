//! A coupled foot problem separates into four irreducible sign patterns.

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use crate::{
    group_foundations::{dot, heading, reveal, stroke},
    model::{QUAD_COUPLING, QUAD_MODES, QUAD_PERMS, quad_mode_gains, reynolds_project_fl4},
    palette,
};

type Matrix = [[i64; 4]; 4];

const MODES: [&str; 4] = ["Together", "Front vs back", "Left vs right", "Diagonals"];

fn in_mode_basis(matrix: Matrix) -> Matrix {
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let value = matrix
                .iter()
                .enumerate()
                .map(|(r, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(c, value)| QUAD_MODES[i][r] * value * QUAD_MODES[j][c])
                        .sum::<i64>()
                })
                .sum::<i64>();
            assert_eq!(value % 4, 0);
            value / 4
        })
    })
}

fn math_cells<T: ToString, const R: usize, const C: usize>(matrix: [[T; C]; R]) -> String {
    let rows = matrix.map(|row| row.map(|element| element.to_string()).join("&"));
    format!(r"\begin{{pmatrix}}{}\end{{pmatrix}}", rows.join(r"\\"))
}

fn shape(stage: &mut Stage<'_>, centre_x: f64, commands: [i64; 4]) -> Mob {
    let coords = [
        v(centre_x + 0.64, 0.34),
        v(centre_x + 0.64, -0.48),
        v(centre_x - 0.64, 0.34),
        v(centre_x - 0.64, -0.48),
    ];
    let body = Polygon::new([coords[0], coords[1], coords[3], coords[2]])
        .style(palette::faded_line(palette::steel(), 3.0, 0.55))
        .build();
    let mut geometry = vec![body];
    for (point, sign) in coords.into_iter().zip(commands) {
        let color = if sign > 0 {
            palette::translation()
        } else {
            palette::symmetry()
        };
        geometry.push(dot(point, color, 0.128));
        let [x, y, _] = point;
        let stem = 0.38 * sign as f64;
        let toward = y + stem;
        geometry.push(stroke(vec![point, v(x, toward)], color, 3.5));
        geometry.push(stroke(
            vec![v(x - 0.10, y + stem * 0.68), v(x, toward), v(x + 0.10, y + stem * 0.68)],
            color,
            3.0,
        ));
    }
    stage.arena_mut().add(v_group(geometry))
}

fn text(kit: &Kit, stage: &mut Stage<'_>, words: &str, x: f64, y: f64) -> fmn::Result<()> {
    let label = kit.text(stage, words, 25.0, Palette::ink(), v(x, y))?;
    reveal(stage, &[label])
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "08_1")?;
    heading(kit, stage, "A different basis uncouples the legs")?;
    let positions = [v(-2.48, 1.14), v(-2.48, -1.06), v(-5.0, 1.14), v(-5.0, -1.06)];
    for (other, value) in positions[1..].iter().zip(QUAD_COUPLING[0][1..].iter().copied()) {
        let edge = stroke(
            vec![positions[0], *other],
            if value == 0 {
                palette::panel()
            } else {
                palette::rotation()
            },
            (value * 2 + 2) as f64,
        );
        let mob = stage.arena_mut().add(edge);
        reveal(stage, &[mob])?;
    }
    for (i, loc) in positions.into_iter().enumerate() {
        let foot = stage
            .arena_mut()
            .add(dot(loc, if i == 0 { palette::hero() } else { palette::steel() }, 0.13));
        reveal(stage, &[foot])?;
    }

    kit.say(stage, "08_2")?;
    let row = kit.tex(
        stage,
        &format!(r"\text{{front-left:}}\quad {}", math_cells([QUAD_COUPLING[0]])),
        28.0,
        palette::rotation(),
        &[],
        v(2.73, 1.55),
    )?;
    reveal(stage, &[row])?;
    text(kit, stage, "A teaching model of coupled feet", 2.70, 0.78)?;

    kit.say(stage, "08_3")?;
    let coupling = kit.tex(
        stage,
        &format!("K={}", math_cells(QUAD_COUPLING)),
        31.0,
        palette::hero(),
        &[],
        v(2.73, -0.53),
    )?;
    let commuting = kit.tex(
        stage,
        r"P_g K=K P_g\qquad (g\in C_2\times C_2)",
        30.0,
        palette::symmetry(),
        &[],
        v(0.0, -2.59),
    )?;
    reveal(stage, &[coupling, commuting])?;
    kit.hold(stage)?;
    fade_all(stage, 0.41)?;

    kit.say(stage, "08_4")?;
    heading(kit, stage, "Four independent sign patterns")?;
    let centers = [-4.65, -1.55, 1.55, 4.65];
    for ((x, pattern), name) in centers.into_iter().zip(QUAD_MODES).zip(MODES) {
        let silhouette = shape(stage, x, pattern);
        reveal(stage, &[silhouette])?;
        text(kit, stage, name, x, 1.65)?;
        let signs = kit.tex(stage, &math_cells([pattern]), 24.0, palette::steel(), &[], v(x, -1.32))?;
        reveal(stage, &[signs])?;
    }

    kit.say(stage, "08_5")?;
    let prompt = kit.tex(
        stage,
        &format!(r"{}\,\cdot\,\text{{each pattern}}\ ? ", math_cells([QUAD_COUPLING[0]])),
        30.0,
        palette::quiz(),
        &[],
        v(0.0, -2.59),
    )?;
    reveal(stage, &[prompt])?;
    kit.listen(stage, 5.0, "four_mode_gains")?;

    kit.say(stage, "08_6")?;
    let dismiss = fade_out(stage.arena_mut(), prompt, ORIGIN, 1.0)?.rt(0.25);
    stage.play(dismiss)?;
    for (x, gain) in centers.into_iter().zip(quad_mode_gains()) {
        let result = kit.tex(
            stage,
            &format!(r"\lambda={gain}"),
            27.0,
            palette::hero(),
            &[],
            v(x, -2.07),
        )?;
        reveal(stage, &[result])?;
    }
    kit.hold(stage)?;

    kit.say(stage, "08_7")?;
    fade_all(stage, 0.42)?;
    heading(kit, stage, "The matrix in the new coordinates")?;
    let columns: [[i64; 4]; 4] = std::array::from_fn(|row| std::array::from_fn(|col| QUAD_MODES[col][row]));
    let basis = kit.tex(
        stage,
        &format!(r"U=\frac{{1}}{{2}}{}", math_cells(columns)),
        32.0,
        palette::rotation(),
        &[],
        v(-3.33, 0.82),
    )?;
    reveal(stage, &[basis])?;
    let diagonal = kit.tex(
        stage,
        &format!(r"U^\mathsf{{T}} K U={}", math_cells(in_mode_basis(QUAD_COUPLING))),
        36.0,
        palette::hero(),
        &[],
        v(3.33, 0.82),
    )?;
    kit.until_sentence(stage, "08_7", 3)?;
    reveal(stage, &[diagonal])?;

    kit.say(stage, "08_8")?;
    let same_sign = kit.tex(
        stage,
        r"P_gv=\chi(g)v,\qquad P_g(Kv)=K(P_gv)=\chi(g)Kv",
        27.0,
        palette::good(),
        &[],
        v(0.0, -1.30),
    )?;
    kit.until_sentence(stage, "08_8", 2)?;
    reveal(stage, &[same_sign])?;
    text(kit, stage, "The same two mirror signs pick out one line", 0.0, -2.26)?;
    kit.hold(stage)?;

    kit.say(stage, "08_9")?;
    fade_all(stage, 0.42)?;
    heading(kit, stage, "Change just one foot")?;
    let mut perturbation = [[0; 4]; 4];
    perturbation[0][0] = 4;
    let source = kit.tex(
        stage,
        &format!(r"\Delta K={}", math_cells(perturbation)),
        33.0,
        palette::rotation(),
        &[],
        v(-3.31, 0.88),
    )?;
    reveal(stage, &[source])?;

    kit.say(stage, "08_10")?;
    let mixed = kit.tex(
        stage,
        &format!(r"U^\mathsf{{T}}\Delta K U={}", math_cells(in_mode_basis(perturbation))),
        34.0,
        palette::symmetry(),
        &[],
        v(3.31, 0.88),
    )?;
    reveal(stage, &[mixed])?;
    text(kit, stage, "The off-diagonal entries mix the modes", 0.0, -1.20)?;

    kit.say(stage, "08_11")?;
    fade_all(stage, 0.42)?;
    heading(kit, stage, "Average over the symmetries that survive")?;
    let projection = kit.tex(
        stage,
        r"\Delta K\quad\longmapsto\quad \frac{1}{|G|}\sum_{g\in G}P_g\Delta K P_g^{-1}",
        35.0,
        palette::hero(),
        &[],
        v(0.0, 1.66),
    )?;
    reveal(stage, &[projection])?;

    kit.say(stage, "08_12")?;
    let full: Vec<_> = (0..QUAD_PERMS.len()).collect();
    let cases = [
        (full.as_slice(), "Both mirrors"),
        (&[0, 1][..], "Only left-right"),
        (&[0][..], "Identity alone"),
    ];
    for (i, (group, description)) in cases.into_iter().enumerate() {
        let y = 0.15 - 1.05 * i as f64;
        text(kit, stage, description, -3.27, y)?;
        let pattern = kit.tex(
            stage,
            &math_cells([reynolds_project_fl4(group)]),
            31.0,
            palette::symmetry(),
            &[],
            v(2.61, y),
        )?;
        reveal(stage, &[pattern])?;
    }
    kit.finish(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_irreducible_lines_diagonalize_the_rectangular_coupling() {
        let diagonal = in_mode_basis(QUAD_COUPLING);
        assert_eq!(diagonal, [[6, 0, 0, 0], [0, 2, 0, 0], [0, 0, 4, 0], [0, 0, 0, 0]]);
        let mut local = [[0; 4]; 4];
        local[0][0] = 4;
        assert_eq!(in_mode_basis(local), [[1; 4]; 4]);
    }
}
