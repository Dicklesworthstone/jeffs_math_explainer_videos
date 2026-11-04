//! Direct products: two clocks, a joint orbit and its limits.
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;
pub struct Products {
    pub kit: Kit,
}
fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &item in items {
        let a = if show {
            fade_in(stage.arena_mut(), item, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), item, ORIGIN, 1.0)?
        };
        anims.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}
fn cell(a: usize, b: usize) -> Vec3 {
    v(-4.5 + a as f64 * 1.5, 1.7 - b as f64 * 1.3)
}
fn dial_position(which: usize, value: f64, radius: f64) -> Vec3 {
    let modulus = if which == 0 { 4. } else { 3. };
    let angle = std::f64::consts::TAU * value / modulus;
    v(
        2.1 + which as f64 * 2.7 + radius * angle.sin(),
        1.3 + radius * angle.cos(),
    )
}
fn turn_pair(stage: &mut Stage<'_>, markers: &[Mob], from: [f64; 2], delta: [f64; 2]) -> fmn::Result<()> {
    for step in 1..=12 {
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &marker) in markers.iter().enumerate() {
            let value = from[i] + delta[i] * f64::from(step) / 12.;
            let motion = marker
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.07),
                    ..AnimateArgs::default()
                })?
                .move_to(dial_position(i, value, 0.73), ORIGIN)?;
            moves.push(stage.prepare(motion)?);
        }
        stage.play_prepared(moves)?;
    }
    Ok(())
}
impl SceneConstruct for Products {
    fn name(&self) -> &str {
        "15_products"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        k.say(stage, "15_1")?;
        let title = k.text(stage, "Two clocks, one state", 37.0, Palette::ink(), v(0.0, 3.0))?;
        let mut grid = vec![title];
        for b in 0..3 {
            for a in 0..4 {
                let p = cell(a, b);
                let disk = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.42)
                        .style(stroke_style(Palette::dim(), 2.))
                        .build()
                        .moved_to(p),
                );
                let label = k.text(stage, &format!("({a},{b})"), 26.0, Palette::ink(), p)?;
                grid.extend([disk, label]);
            }
        }
        fade(stage, &grid, true)?;
        let mut clock_markers = Vec::new();
        for (i, modulus) in [4, 3].into_iter().enumerate() {
            let color = if i == 0 { Palette::blue() } else { Palette::hero() };
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.73)
                    .style(stroke_style(color, 2.))
                    .build()
                    .moved_to(v(2.1 + i as f64 * 2.7, 1.3)),
            );
            let mut clock = vec![ring];
            for n in 0..modulus {
                clock.push(k.text(stage, &n.to_string(), 22., color, dial_position(i, f64::from(n), 1.03))?);
            }
            let marker = stage.arena_mut().add(
                Circle::new()
                    .radius(0.13)
                    .style(ink_style(color))
                    .build()
                    .moved_to(dial_position(i, 0., 0.73)),
            );
            clock.push(marker);
            clock_markers.push(marker);
            fade(stage, &clock, true)?;
            grid.extend(clock);
        }
        let count = k.text(stage, "4 × 3 = 12 states", 28.0, Palette::blue(), v(3.4, -2.1))?;
        fade(stage, &[count], true)?;
        grid.push(count);
        k.say(stage, "15_2")?;
        turn_pair(stage, &clock_markers, [0., 0.], [3., 2.])?;
        let calc = k.tex(stage, r"(3,2)+(2,2)=(1,1)", 28.0, Palette::hero(), &[], v(3.3, -0.3))?;
        fade(stage, &[calc], true)?;
        grid.push(calc);
        turn_pair(stage, &clock_markers, [3., 2.], [2., 2.])?;
        k.say(stage, "15_3")?;
        let rule = k.tex(
            stage,
            r"(a,b)+(c,d)=(a+c,b+d)",
            26.0,
            Palette::green(),
            &[],
            v(3.2, -1.1),
        )?;
        fade(stage, &[rule], true)?;
        grid.push(rule);
        k.say(stage, "15_4")?;
        fade(stage, &[calc, rule], false)?;
        grid.retain(|item| *item != calc && *item != rule);
        // Start a new orbit experiment with both dials explicitly reset to zero.
        turn_pair(stage, &clock_markers, [1., 1.], [3., 2.])?;
        let question = k.text(stage, "When do both return?", 27.0, Palette::hero(), v(3.2, -0.3))?;
        fade(stage, &[question], true)?;
        grid.push(question);
        // Begin the experiment with the answer sentence, not after the whole cue.
        // A real prediction pause must be bound in the narration plan before delivery.
        k.until_sentence(stage, "15_4", 4)?;
        let mut selected = stage.arena_mut().add(
            Circle::new()
                .radius(0.49)
                .style(stroke_style(Palette::gold(), 3.))
                .build()
                .moved_to(cell(0, 0)),
        );
        fade(stage, &[selected], true)?;
        grid.push(selected);
        for step in 1..=12 {
            let p = cell(step % 4, step % 3);
            let next = stage.arena_mut().add(
                Circle::new()
                    .radius(0.49)
                    .style(stroke_style(Palette::gold(), 3.))
                    .build()
                    .moved_to(p),
            );
            // Both clock hands and the discrete state settle on the same frame.
            // Crossfade fixed outlines; never sweep a filled marker over pair labels.
            for substep in 1..=12 {
                let mut moves: Vec<Box<dyn Animation>> = Vec::new();
                let value = (step - 1) as f64 + f64::from(substep) / 12.;
                for (i, &hand) in clock_markers.iter().enumerate() {
                    moves.push(
                        stage.prepare(
                            hand.animate()
                                .set_anim_args(AnimateArgs {
                                    run_time: Some(0.07),
                                    ..AnimateArgs::default()
                                })?
                                .move_to(dial_position(i, value, 0.73), ORIGIN)?,
                        )?,
                    );
                }
                if substep == 12 {
                    let old_out = fade_out(stage.arena_mut(), selected, ORIGIN, 1.)?.rt(0.07);
                    moves.push(stage.prepare(old_out)?);
                    let next_in = fade_in(stage.arena_mut(), next, ORIGIN, 1.)?.rt(0.07);
                    moves.push(stage.prepare(next_in)?);
                }
                stage.play_prepared(moves)?;
            }
            grid.retain(|item| *item != selected);
            selected = next;
            grid.push(next);
            if step < 12 {
                let dot = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.06)
                        .style(ink_style(Palette::gold()))
                        .build()
                        .moved_to(v(p[0], p[1] - 0.55)),
                );
                fade(stage, &[dot], true)?;
                grid.push(dot);
            }
        }
        let twelve = k.text(stage, "First shared return: 12", 28.0, Palette::green(), v(3.2, -1.0))?;
        fade(stage, &[twelve], true)?;
        grid.push(twelve);
        k.say(stage, "15_5")?;
        let order = k.tex(
            stage,
            r"\operatorname{ord}(a,b)=\operatorname{lcm}(\operatorname{ord}(a),\operatorname{ord}(b))",
            29.0,
            Palette::ink(),
            &[],
            v(0.0, -2.6),
        )?;
        fade(stage, &[order], true)?;
        grid.push(order);
        k.say(stage, "15_6")?;
        fade(stage, &grid, false)?;
        let contrast = k.tex(
            stage,
            r"C_2\times C_2\qquad\text{versus}\qquad C_4",
            38.0,
            Palette::ink(),
            &[],
            v(0.0, 2.9),
        )?;
        let mut contrast_objects = vec![contrast];
        let pair_position = |a: usize, b: usize| v(-4.6 + a as f64 * 1.7, 0.9 - b as f64 * 1.7);
        for b in 0..2 {
            for a in 0..2 {
                let p = pair_position(a, b);
                let disk = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.48)
                        .style(stroke_style(Palette::blue(), 2.))
                        .build()
                        .moved_to(p),
                );
                let label = k.text(stage, &format!("({a},{b})"), 27., Palette::ink(), p)?;
                contrast_objects.extend([disk, label]);
            }
        }
        for n in 0..4 {
            let t = std::f64::consts::TAU * n as f64 / 4.;
            let p = v(3. + 1.25 * t.sin(), 0.15 + 1.25 * t.cos());
            let label = k.text(stage, &n.to_string(), 30., Palette::ink(), p)?;
            contrast_objects.push(label);
        }
        fade(stage, &contrast_objects, true)?;
        k.until_sentence(stage, "15_6", 3)?;
        for (da, db) in [(1, 0), (0, 1), (1, 1)] {
            let caption = k.text(
                stage,
                &format!("Repeat ({da},{db})"),
                27.,
                Palette::blue(),
                v(-3.75, -1.8),
            )?;
            let first = stage.arena_mut().add(
                Circle::new()
                    .radius(0.57)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(pair_position(0, 0)),
            );
            fade(stage, &[caption, first], true)?;
            let next = stage.arena_mut().add(
                Circle::new()
                    .radius(0.57)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(pair_position(da, db)),
            );
            fade(stage, &[first], false)?;
            fade(stage, &[next], true)?;
            stage.wait(0.5)?;
            fade(stage, &[next], false)?;
            fade(stage, &[first], true)?;
            stage.wait(0.5)?;
            fade(stage, &[first, caption], false)?;
        }
        let left = k.text(
            stage,
            "All three choices return in 2",
            25.,
            Palette::blue(),
            v(-3.6, -2.5),
        )?;
        fade(stage, &[left], true)?;
        contrast_objects.push(left);
        let mut selected = None;
        for n in 0..=4 {
            let t = std::f64::consts::TAU * n as f64 / 4.;
            let p = v(3. + 1.25 * t.sin(), 0.15 + 1.25 * t.cos());
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.37)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(p),
            );
            if let Some(old) = selected {
                fade(stage, &[old], false)?;
            }
            fade(stage, &[ring], true)?;
            stage.wait(0.45)?;
            selected = Some(ring);
        }
        if let Some(ring) = selected {
            contrast_objects.push(ring);
        }
        let right = k.text(
            stage,
            "This choice first returns in 4",
            25.,
            Palette::hero(),
            v(3., -2.5),
        )?;
        fade(stage, &[right], true)?;
        contrast_objects.push(right);
        // Let the completed comparison be read before the following cue wipes it.
        stage.wait(2.5)?;
        k.say(stage, "15_7")?;
        fade(stage, &contrast_objects, false)?;
        let dictionary_title = k.text(
            stage,
            "Two descriptions of the same operation",
            32.,
            Palette::ink(),
            v(0., 3.),
        )?;
        let mut dictionary_objects = vec![dictionary_title];
        fade(stage, &dictionary_objects, true)?;
        let mut recalled_units = Vec::new();
        for (i, unit) in [1_usize, 5, 7, 11].into_iter().enumerate() {
            let x = -4.2 + i as f64 * 2.8;
            let number = k.text(stage, &unit.to_string(), 36., Palette::hero(), v(x, 0.2))?;
            fade(stage, &[number], true)?;
            recalled_units.push(number);
        }
        k.until_sentence(stage, "15_7", 1)?;
        for (i, unit) in [5_usize, 7, 11].into_iter().enumerate() {
            let x = -1.4 + i as f64 * 2.8;
            let square = k.tex(
                stage,
                &format!(r"{unit}^2\equiv{}\pmod{{12}}", unit * unit % 12),
                25.,
                Palette::green(),
                &[],
                v(x, -1.1),
            )?;
            fade(stage, &[square], true)?;
            recalled_units.push(square);
        }
        k.until_sentence(stage, "15_7", 2)?;
        fade(stage, &recalled_units, false)?;
        for (i, (a, b)) in [(0_u32, 0_u32), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
            let x = -4.2 + i as f64 * 2.8;
            let unit = (5_usize.pow(a) * 7_usize.pow(b)) % 12;
            let pair = k.text(stage, &format!("({a},{b})"), 33., Palette::blue(), v(x, 1.5))?;
            let arrow = k.tex(stage, r"\updownarrow", 30., Palette::dim(), &[], v(x, 0.85))?;
            let number = k.text(stage, &unit.to_string(), 36., Palette::hero(), v(x, 0.2))?;
            fade(stage, &[pair, arrow, number], true)?;
            dictionary_objects.extend([pair, arrow, number]);
            stage.wait(0.35)?;
        }
        k.until_sentence(stage, "15_7", 4)?;
        let mut input_rings = Vec::new();
        for x in [-1.4, 1.4] {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.43)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(v(x, 0.2)),
            );
            fade(stage, &[ring], true)?;
            input_rings.push(ring);
        }
        let pair_sum = k.tex(stage, r"(1,0)+(0,1)=(1,1)", 34., Palette::blue(), &[], v(0., -1.15))?;
        fade(stage, &[pair_sum], true)?;
        dictionary_objects.push(pair_sum);
        let product = 5 * 7;
        let check = k.tex(
            stage,
            &format!(r"5\cdot7={product}\equiv{}\pmod{{12}}", product % 12),
            34.,
            Palette::hero(),
            &[],
            v(0., -2.05),
        )?;
        fade(stage, &[check], true)?;
        dictionary_objects.push(check);
        stage.wait(1.5)?;
        fade(stage, &input_rings, false)?;
        let result_ring = stage.arena_mut().add(
            Circle::new()
                .radius(0.48)
                .style(stroke_style(Palette::green(), 3.))
                .build()
                .moved_to(v(4.2, 0.2)),
        );
        let result_pair = stage.arena_mut().add(
            Circle::new()
                .radius(0.48)
                .style(stroke_style(Palette::green(), 3.))
                .build()
                .moved_to(v(4.2, 1.5)),
        );
        fade(stage, &[result_ring, result_pair], true)?;
        dictionary_objects.extend([result_ring, result_pair]);
        stage.wait(2.0)?;
        k.say(stage, "15_8")?;
        fade(stage, &dictionary_objects, false)?;
        let crt = k.tex(
            stage,
            r"\mathbb{Z}/12\mathbb{Z}\longrightarrow\mathbb{Z}/4\mathbb{Z}\times\mathbb{Z}/3\mathbb{Z}",
            30.0,
            Palette::ink(),
            &[],
            v(0.0, 3.0),
        )?;
        fade(stage, &[crt], true)?;
        let mut remainder_objects = vec![crt];
        // Each of the twelve integers occupies exactly one computed remainder cell.
        for n in 0..12 {
            let a = n % 4;
            let b = n % 3;
            let p = v(-4.2 + a as f64 * 2.8, 1.5 - b as f64 * 1.45);
            let cell = stage.arena_mut().add(
                Polygon::new(vec![
                    v(p[0] - 1.1, p[1] + 0.55),
                    v(p[0] + 1.1, p[1] + 0.55),
                    v(p[0] + 1.1, p[1] - 0.55),
                    v(p[0] - 1.1, p[1] - 0.55),
                ])
                .style(stroke_style(Palette::dim(), 2.))
                .build(),
            );
            let pair = k.text(stage, &format!("({a},{b})"), 22., Palette::dim(), v(p[0], p[1] - 0.3))?;
            let number = k.text(
                stage,
                &n.to_string(),
                33.,
                if n == 7 { Palette::hero() } else { Palette::blue() },
                v(p[0], p[1] + 0.15),
            )?;
            fade(stage, &[cell, pair], true)?;
            if n == 7 {
                k.until_sentence(stage, "15_8", 2)?;
            }
            fade(stage, &[number], true)?;
            remainder_objects.extend([cell, pair, number]);
        }
        let seven = k.text(stage, "7 has remainders 3 and 1", 26., Palette::hero(), v(0., -2.7))?;
        fade(stage, &[seven], true)?;
        remainder_objects.push(seven);
        k.say(stage, "15_9")?;
        fade(stage, &[seven], false)?;
        remainder_objects.retain(|item| *item != seven);
        let coprime = k.text(
            stage,
            "Twelve integers. Twelve different pairs. No cell missing.",
            27.,
            Palette::green(),
            v(0., -2.7),
        )?;
        fade(stage, &[coprime], true)?;
        remainder_objects.push(coprime);
        k.until_sentence(stage, "15_9", 1)?;
        let mut orbit_ring = None;
        for n in 0..12 {
            let a = n % 4;
            let b = n % 3;
            let p = v(-4.2 + a as f64 * 2.8, 1.5 - b as f64 * 1.45);
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.34)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(v(p[0], p[1] + 0.15)),
            );
            if let Some(old) = orbit_ring {
                fade(stage, &[old], false)?;
            }
            fade(stage, &[ring], true)?;
            stage.wait(0.15)?;
            orbit_ring = Some(ring);
        }
        if let Some(ring) = orbit_ring {
            remainder_objects.push(ring);
        }
        k.until_sentence(stage, "15_9", 4)?;
        fade(stage, &remainder_objects, false)?;
        let common_title = k.text(
            stage,
            "Now try a four-clock and a two-clock",
            32.,
            Palette::ink(),
            v(0., 3.),
        )?;
        let mut common_objects = vec![common_title];
        for b in 0..2 {
            for a in 0..4 {
                let p = v(-4.2 + a as f64 * 2.8, 1. - b as f64 * 1.8);
                let reachable = a % 2 == b;
                let color = if reachable { Palette::blue() } else { Palette::dim() };
                let ring = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.62)
                        .style(stroke_style(color, 2.))
                        .build()
                        .moved_to(p),
                );
                let label = k.text(stage, &format!("({a},{b})"), 28., color, p)?;
                common_objects.extend([ring, label]);
            }
        }
        fade(stage, &common_objects, true)?;
        let mut last = None;
        for n in 0..=4 {
            let p = v(-4.2 + (n % 4) as f64 * 2.8, 1. - (n % 2) as f64 * 1.8);
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.73)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(p),
            );
            if let Some(old) = last {
                fade(stage, &[old], false)?;
            }
            fade(stage, &[ring], true)?;
            stage.wait(0.3)?;
            last = Some(ring);
        }
        if let Some(ring) = last {
            common_objects.push(ring);
        }
        let short_orbit = k.text(
            stage,
            "Back after 4 moves. Four pairs were never reached.",
            27.,
            Palette::hero(),
            v(0., -2.5),
        )?;
        fade(stage, &[short_orbit], true)?;
        common_objects.push(short_orbit);
        stage.wait(2.5)?;
        k.say(stage, "15_10")?;
        fade(stage, &common_objects, false)?;
        let internal = k.text(
            stage,
            "Find two independent parts inside the twelve-clock",
            32.,
            Palette::ink(),
            v(0., 3.),
        )?;
        fade(stage, &[internal], true)?;
        let h = [0, 3, 6, 9];
        let j = [0, 4, 8];
        let mut clock = Vec::new();
        for n in 0..12 {
            let angle = std::f64::consts::TAU * f64::from(n) / 12.;
            let position = v(1.9 * angle.sin(), 0.1 + 1.9 * angle.cos());
            let label = k.text(stage, &n.to_string(), 29., Palette::ink(), position)?;
            clock.push(label);
        }
        fade(stage, &clock, true)?;
        k.until_sentence(stage, "15_10", 2)?;
        let h_name = k.text(stage, "Steps of 3", 28., Palette::blue(), v(-4., 1.))?;
        let j_name = k.text(stage, "Steps of 4", 28., Palette::hero(), v(4., 1.))?;
        fade(stage, &[h_name], true)?;
        clock.push(h_name);
        for n in h {
            let angle = std::f64::consts::TAU * f64::from(n) / 12.;
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.35)
                    .style(stroke_style(Palette::blue(), 3.))
                    .build()
                    .moved_to(v(1.9 * angle.sin(), 0.1 + 1.9 * angle.cos())),
            );
            fade(stage, &[ring], true)?;
            clock.push(ring);
        }
        fade(stage, &[j_name], true)?;
        clock.push(j_name);
        for n in j {
            let angle = std::f64::consts::TAU * f64::from(n) / 12.;
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.44)
                    .style(stroke_style(Palette::hero(), 3.))
                    .build()
                    .moved_to(v(1.9 * angle.sin(), 0.1 + 1.9 * angle.cos())),
            );
            fade(stage, &[ring], true)?;
            clock.push(ring);
        }
        k.until_sentence(stage, "15_10", 3)?;
        fade(stage, &clock, false)?;
        let mut table = vec![internal];
        let plus = k.text(stage, "+", 30., Palette::ink(), v(-4.5, 1.8))?;
        table.push(plus);
        for (col, value) in h.iter().enumerate() {
            let label = k.text(
                stage,
                &value.to_string(),
                30.,
                Palette::blue(),
                v(-2.7 + col as f64 * 1.8, 1.8),
            )?;
            table.push(label);
        }
        for (row, value) in j.iter().enumerate() {
            let label = k.text(
                stage,
                &value.to_string(),
                30.,
                Palette::hero(),
                v(-4.5, 0.8 - row as f64),
            )?;
            table.push(label);
        }
        fade(stage, &table[1..], true)?;
        k.until_sentence(stage, "15_10", 4)?;
        for (row, b) in j.iter().enumerate() {
            for (col, a) in h.iter().enumerate() {
                let label = k.text(
                    stage,
                    &((a + b) % 12).to_string(),
                    31.,
                    Palette::ink(),
                    v(-2.7 + col as f64 * 1.8, 0.8 - row as f64),
                )?;
                fade(stage, &[label], true)?;
                table.push(label);
            }
        }
        k.until_sentence(stage, "15_10", 5)?;
        let coverage = k.text(
            stage,
            "Every residue appears once: a unique sum of two parts",
            26.,
            Palette::green(),
            v(0., -2.2),
        )?;
        fade(stage, &[coverage], true)?;
        table.push(coverage);
        // Read the unique description of seven directly from its row and column.
        let mut chosen = Vec::new();
        for (position, color) in [
            (v(-0.9, 1.8), Palette::blue()),
            (v(-4.5, -0.2), Palette::hero()),
            (v(-0.9, -0.2), Palette::green()),
        ] {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.33)
                    .style(stroke_style(color, 3.))
                    .build()
                    .moved_to(position),
            );
            fade(stage, &[ring], true)?;
            chosen.push(ring);
        }
        let sum = k.tex(stage, r"3+4=7", 30., Palette::green(), &[], v(0., -3.))?;
        fade(stage, &[sum], true)?;
        chosen.push(sum);
        stage.wait(1.0)?;
        k.until_sentence(stage, "15_10", 6)?;
        fade(stage, &chosen, false)?;
        let mut shared = Vec::new();
        for position in [v(-2.7, 1.8), v(-4.5, 0.8)] {
            let ring = stage.arena_mut().add(
                Circle::new()
                    .radius(0.33)
                    .style(stroke_style(Palette::green(), 3.))
                    .build()
                    .moved_to(position),
            );
            shared.push(ring);
        }
        let only = k.text(stage, "Only 0 belongs to both lists", 27., Palette::green(), v(0., -3.))?;
        shared.push(only);
        fade(stage, &shared, true)?;
        stage.wait(1.5)?;
        fade(stage, &shared, false)?;
        let commute = k.tex(stage, r"3+4=4+3=7", 32., Palette::hero(), &[], v(0., -3.))?;
        fade(stage, &[commute], true)?;
        table.push(commute);
        k.say(stage, "15_12")?;
        fade(stage, &table, false)?;
        let general = k.text(
            stage,
            "When do two internal parts form a direct product?",
            31.,
            Palette::ink(),
            v(0., 2.7),
        )?;
        fade(stage, &[general], true)?;
        for (line, y) in [
            ("Moves from different parts commute", 1.3),
            ("Their only shared element is the identity", 0.),
            ("Together they generate every element", -1.3),
        ] {
            let condition = k.text(stage, line, 30., Palette::blue(), v(0., y))?;
            fade(stage, &[condition], true)?;
            stage.wait(0.8)?;
        }
        k.say(stage, "15_11")?;
        k.finish(stage)
    }
}
