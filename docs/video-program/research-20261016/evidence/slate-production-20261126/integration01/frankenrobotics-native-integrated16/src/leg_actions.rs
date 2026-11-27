//! Exact permutation actions on rectangular and radial leg layouts.

use std::f64::consts::TAU;

use explainer_kit::{Kit, Palette, Timed, add, fade_all, lerp, play, tween, v};
use fmn::prelude::*;

use crate::model::{QUAD_PERMS, hexapod_d6_elements, hexapod_d6_orbits, quad_stabilizer};
use crate::palette;

fn title(kit: &Kit, stage: &mut Stage<'_>, label: &str) -> fmn::Result<Mob> {
    kit.text(stage, label, 37.0, Palette::ink(), v(0.0, 3.4))
}

fn reveal(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut fades = Vec::new();
    for &mob in mobs {
        let fade = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(0.45);
        fades.push(stage.prepare(fade)?);
    }
    stage.play_prepared(fades)?;
    Ok(())
}

fn quad_foot(i: usize) -> Vec3 {
    v(
        if i < 2 { -1.6 } else { -4.8 },
        if i.is_multiple_of(2) { 1.0 } else { -1.0 },
    )
}

fn hex_foot(angle: f64) -> Vec3 {
    v(-3.2 + 2.1 * angle.cos(), 2.1 * angle.sin())
}

fn clear(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    fade_all(stage, 0.45)
}

fn cycles(permutation: &[usize]) -> u32 {
    let mut visited = vec![false; permutation.len()];
    let mut count = 0;
    for start in 0..permutation.len() {
        if visited[start] {
            continue;
        }
        count += 1;
        let mut current = start;
        while !visited[current] {
            visited[current] = true;
            current = permutation[current];
        }
    }
    count
}

fn matrix(permutation: [usize; 4]) -> String {
    let rows = (0..4)
        .map(|row| {
            permutation
                .iter()
                .map(|&destination| if destination == row { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join("&")
        })
        .collect::<Vec<_>>()
        .join(r"\\");
    format!(r"\begin{{pmatrix}}{rows}\end{{pmatrix}}")
}

pub fn construct(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "07_1")?;
    let header = title(kit, stage, "Four feet, four command channels")?;
    let body = Rectangle::new()
        .width(2.8)
        .height(1.42)
        .corner_radius(0.21)
        .style(
            Style::default()
                .fill(palette::panel(), 1.0)
                .stroke(palette::steel(), 2.6, 1.0),
        )
        .build()?
        .moved_to(v(-3.2, 0.0));
    let mut fixtures = vec![header, stage.arena_mut().add(body)];
    for (index, name) in ["FL", "FR", "BL", "BR"].into_iter().enumerate() {
        let foot = quad_foot(index);
        let label = kit.text(stage, name, 21.0, Palette::ink(), add(foot, v(0.0, -0.41)))?;
        fixtures.push(label);
    }
    reveal(stage, &fixtures)?;
    // Bar heights belong to input channels. Group elements send whole bars to output slots.
    let motion = stage.add_value_tracker(0.0);
    let colors = [
        palette::hero(),
        palette::rotation(),
        palette::translation(),
        palette::symmetry(),
    ];
    let bars = stage.always_redraw(move |scene| {
        let progress = scene.tracker_value(motion).unwrap_or(0.0).clamp(0.0, 2.0);
        let segment = usize::from(progress >= 1.0);
        let fraction = progress - segment as f64;
        let shapes: Vec<_> = colors
            .into_iter()
            .enumerate()
            .map(|(channel, color)| {
                let start = quad_foot(QUAD_PERMS[segment][channel]);
                let end = quad_foot(QUAD_PERMS[if segment == 0 { 1 } else { 3 }][channel]);
                let rise = 0.22 * (4 - channel) as f64;
                let bend = (std::f64::consts::PI * fraction).sin() * 0.23;
                let offset = if segment == 0 {
                    v(if channel.is_multiple_of(2) { bend } else { -bend }, rise * 0.5)
                } else {
                    v(0.0, rise * 0.5 + if channel < 2 { bend } else { -bend })
                };
                let anchor = v(lerp(start[0], end[0], fraction), lerp(start[1], end[1], fraction));
                let position = add(anchor, offset);
                Rectangle::new()
                    .width(0.30)
                    .height(rise)
                    .style(palette::flat(color, 1.0))
                    .build()
                    .expect("positive bar size")
                    .moved_to(position)
            })
            .collect();
        scene.add(v_group(shapes))
    });
    stage.add_to_scene(bars)?;

    kit.say(stage, "07_2")?;
    let swap_sides = kit.text(stage, "Mirror left and right", 28.0, palette::symmetry(), v(3.1, 1.3))?;
    reveal(stage, &[swap_sides])?;
    play!(stage; tween(motion, 1.0, 1.6)?);
    let swap_ends = kit.text(
        stage,
        "Then mirror front and back",
        26.0,
        palette::symmetry(),
        v(3.1, 0.58),
    )?;
    reveal(stage, &[swap_ends])?;
    play!(stage; tween(motion, 2.0, 1.6)?);

    kit.say(stage, "07_3")?;
    let product = format!(
        r"P_{{\mathrm{{front/back}}}}P_{{\mathrm{{left/right}}}}={}",
        matrix(QUAD_PERMS[3])
    );
    let combined = kit.tex(stage, &product, 26.0, palette::hero(), &[], v(2.9, -0.72))?;
    let group = kit.tex(stage, r"C_2\times C_2", 31.0, palette::symmetry(), &[], v(2.9, -2.29))?;
    reveal(stage, &[combined, group])?;

    kit.say(stage, "07_4")?;
    let counts = QUAD_PERMS.map(|permutation| 3_u32.pow(cycles(&permutation)));
    let raw = kit.tex(
        stage,
        r"3^4=81\quad\text{labeled patterns}",
        24.0,
        Palette::ink(),
        &[],
        v(-3.2, -2.0),
    )?;
    reveal(stage, &[raw])?;
    kit.until_sentence(stage, "07_4", 4)?;
    let [id, mirror_lr, mirror_fb, turn] = counts;
    let orbits = kit.tex(
        stage,
        &format!(
            r"\frac{{{id}+{mirror_lr}+{mirror_fb}+{turn}}}{{4}}={}",
            counts.iter().sum::<u32>() / 4
        ),
        27.0,
        palette::good(),
        &[],
        v(-3.2, -2.8),
    )?;
    reveal(stage, &[orbits])?;
    clear(kit, stage)?;

    kit.say(stage, "07_5")?;
    let header = title(kit, stage, "Which permutations survive?")?;
    reveal(stage, &[header])?;
    let board = [
        quad_stabilizer(false, false, false),
        quad_stabilizer(true, false, false),
        quad_stabilizer(true, true, false),
    ];
    let labels = ["Both mirrors + half turn", "Only left–right mirror", "Identity"];
    let bare = kit.text(stage, "Bare chassis", 27.0, Palette::ink(), v(-4.45, 2.18))?;
    let lidar = kit.text(stage, "Forward lidar", 27.0, palette::rotation(), v(0.0, 2.18))?;
    let payload = kit.text(stage, "Off-center load", 27.0, palette::warn(), v(4.45, 2.18))?;
    let mut backs = Vec::new();
    let mut counts = Vec::new();
    for (i, group) in board.iter().enumerate() {
        let x = (i as f64 - 1.0) * 4.45;
        let chassis = Rectangle::new()
            .width(2.0)
            .height(0.84)
            .corner_radius(0.15)
            .style(
                Style::default()
                    .fill(palette::panel(), 1.0)
                    .stroke(palette::steel(), 2.6, 1.0),
            )
            .build()?
            .moved_to(v(x, 1.16));
        let chassis = stage.arena_mut().add(chassis);
        let mut parts = vec![chassis];
        for dx in [-0.72, 0.72] {
            for dy in [-0.52, 0.52] {
                let shoe = Dot::new()
                    .point(v(x + dx, 1.16 + dy))
                    .radius(0.09)
                    .color(palette::steel())
                    .build();
                parts.push(stage.arena_mut().add(shoe));
            }
        }
        if i >= 1 {
            let nose = v(x + 1.0, 1.16);
            let cone = Polygon::new([nose, v(x + 1.52, 1.46), v(x + 1.52, 0.86)])
                .style(palette::flat(palette::rotation(), 0.7))
                .build();
            parts.push(stage.arena_mut().add(cone));
        }
        if i == 2 {
            let load = Rectangle::new()
                .width(0.52)
                .height(0.25)
                .style(palette::flat(palette::warn(), 1.0))
                .build()?
                .moved_to(v(x - 0.12, 1.37));
            parts.push(stage.arena_mut().add(load));
        }
        backs.push(parts);
        counts.push([
            kit.text(stage, &group.len().to_string(), 56.0, palette::symmetry(), v(x, 0.02))?,
            kit.text(stage, labels[i], 18.0, Palette::ink(), v(x, -0.88))?,
        ]);
    }
    reveal(stage, &[bare, lidar])?;
    reveal(stage, &backs[0])?;
    reveal(stage, &backs[1])?;

    kit.say(stage, "07_6")?;
    reveal(stage, &counts[0])?;
    reveal(stage, &counts[1])?;
    kit.say(stage, "07_7")?;
    reveal(stage, &[payload])?;
    reveal(stage, &backs[2])?;
    reveal(stage, &counts[2])?;
    let gear = kit.text(
        stage,
        "One different gearbox also breaks the mechanical symmetry",
        22.0,
        Palette::ink(),
        v(0.0, -2.0),
    )?;
    reveal(stage, &[gear])?;

    kit.say(stage, "07_8")?;
    let mirror = kit.tex(
        stage,
        r"(\mathrm{roll},\mathrm{pitch},\mathrm{yaw})\mapsto(-\mathrm{roll},\mathrm{pitch},-\mathrm{yaw})",
        27.0,
        palette::hero(),
        &[],
        v(0.0, -2.9),
    )?;
    reveal(stage, &[mirror])?;
    clear(kit, stage)?;

    kit.say(stage, "07_9")?;
    let header = title(kit, stage, "Six legs, twelve permutations")?;
    let mut fixtures = vec![header];
    for i in 0..6 {
        let angle = TAU * f64::from(i) / 6.0;
        let foot = hex_foot(angle);
        let spoke = polyline(vec![v(-3.2, 0.0), foot])
            .style(palette::faded_line(palette::steel(), 2.5, 0.48))
            .build();
        fixtures.push(stage.arena_mut().add(spoke));
        fixtures.push(kit.text(
            stage,
            &i.to_string(),
            24.0,
            Palette::ink(),
            add(foot, v(0.36 * angle.cos(), 0.36 * angle.sin())),
        )?);
    }
    let actions = kit.tex(
        stage,
        r"r(i)=i+1\pmod 6,\quad s(i)=-i\pmod 6",
        27.0,
        palette::symmetry(),
        &[],
        v(2.8, 1.05),
    )?;
    fixtures.push(actions);
    reveal(stage, &fixtures)?;

    kit.say(stage, "07_10")?;
    let elements = hexapod_d6_elements();
    let s = elements[1];
    let r = elements[2];
    let reflected = s[1];
    let turned = r[reflected];
    let last = s[turned];
    let visited = [1, reflected, turned, last];
    let phase = stage.add_value_tracker(0.0);
    let point = stage.always_redraw(move |scene| {
        let progress = scene.tracker_value(phase).unwrap_or(0.0).clamp(0.0, 3.0);
        let start = (progress.floor() as usize).min(2);
        let u = progress - start as f64;
        let a = visited[start] as f64;
        let b = visited[start + 1] as f64;
        let delta = (b - a + 3.0).rem_euclid(6.0) - 3.0;
        scene.add(
            Dot::new()
                .point(hex_foot(TAU * (a + u * delta) / 6.0))
                .radius(0.12)
                .color(palette::hero())
                .build(),
        )
    });
    stage.add_to_scene(point)?;
    let question = kit.text(stage, "Forward or backward?", 30.0, palette::quiz(), v(2.8, 0.18))?;
    reveal(stage, &[question])?;
    kit.hold(stage)?;
    kit.listen(stage, 5.0, "07_10_predict")?;

    kit.say(stage, "07_11")?;
    for (index, &slot) in visited.iter().enumerate().skip(1) {
        play!(stage; tween(phase, index as f64, 0.72)?);
        let step = kit.text(
            stage,
            &slot.to_string(),
            25.0,
            palette::hero(),
            v(1.4 + index as f64 * 0.72, -0.6),
        )?;
        reveal(stage, &[step])?;
    }
    let identity = kit.tex(stage, r"srs=r^{-1}", 32.0, palette::good(), &[], v(2.8, -1.43))?;
    reveal(stage, &[identity])?;

    kit.say(stage, "07_12")?;
    let total = kit.text(
        stage,
        &format!("D₆    {} distinct permutations", elements.len()),
        23.0,
        palette::symmetry(),
        v(2.8, -2.25),
    )?;
    reveal(stage, &[total])?;

    clear(kit, stage)?;
    kit.say(stage, "07_13")?;
    let header = title(kit, stage, "Ten choices for each leg")?;
    let raw = kit.tex(
        stage,
        r"10^6=1{,}000{,}000\quad\text{labeled builds}",
        31.0,
        Palette::ink(),
        &[],
        v(0.0, 1.9),
    )?;
    let same_module = kit.tex(
        stage,
        r"r=(0\ 1\ 2\ 3\ 4\ 5)\qquad 10\quad\text{fixed builds}",
        25.0,
        palette::translation(),
        &[],
        v(0.0, 0.94),
    )?;
    let fixed = |p: &[usize; 6]| 10_u64.pow(cycles(p));
    let rotations: u64 = elements.iter().step_by(2).map(fixed).sum();
    let reflections: u64 = elements.iter().skip(1).step_by(2).map(fixed).sum();
    let d6 = kit.tex(
        stage,
        &format!(
            r"\frac{{{rotations}+{reflections}}}{{12}}={}\quad\text{{all symmetries}}",
            hexapod_d6_orbits(10)
        ),
        31.0,
        palette::good(),
        &[],
        v(0.0, -0.35),
    )?;
    let gap_mirror = elements[3];
    let gap_fixed = fixed(&gap_mirror);
    let count_raw = 10_u64.pow(6);
    let two = (count_raw + gap_fixed) / 2;
    let one_mirror = kit.tex(
        stage,
        &format!(r"\frac{{{count_raw}+{gap_fixed}}}{{2}}={two}\quad\text{{one gap-axis mirror}}"),
        29.0,
        palette::symmetry(),
        &[],
        v(0.0, -1.92),
    )?;
    reveal(stage, &[header, raw])?;
    kit.until_sentence(stage, "07_13", 3)?;
    reveal(stage, &[same_module])?;
    kit.until_sentence(stage, "07_13", 5)?;
    reveal(stage, &[d6])?;
    kit.until_sentence(stage, "07_13", 7)?;
    reveal(stage, &[one_mirror])?;
    kit.finish(stage)
}
