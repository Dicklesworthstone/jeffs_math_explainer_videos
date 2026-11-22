//! Chapter 07. One face contributes the same flux with opposite signs.

use explainer_kit::{Facts, Kit, Palette, Timed, narration, v};
use fmn::prelude::*;

pub struct Chapter07 {
    pub kit: Kit,
}

// Positive means out of the cell. The unit-square example returns from chapter 05.
const CELL: [i64; 4] = [-2, 3, 0, -1]; // left, right, bottom, top
const PATCH_CENTER: [f64; 2] = [-3.25, 0.1];
const CELL_WIDTH: f64 = 1.38;
const SEALED_EDGES: [(usize, usize); 4] = [(0, 1), (2, 3), (0, 2), (1, 3)];
const RIGHT: f64 = 3.35;
/// Where 07_4 says the right, left and top rows' numbers: (word, sentence, seconds after that sentence
/// starts). Each is the later of the whisper large-v3-turbo onset on take a41d899b (ledger recognition
/// record 6) and the end of the pause it sits in on the placed WAV e5c6dd16, from
/// white-ledger-projection-sync01 `phrase-markers01`. Estimates, not heard boundaries.
const LEDGER_WORDS: [(&str, usize, f64); 3] = [("three", 2, 1.83), ("two", 2, 3.33), ("one", 2, 4.79)];
/// The same for the sum: the fourth sentence, "Zero."
const SUM_WORD: (&str, usize, f64) = ("zero", 3, 0.03);

fn draw_line(
    stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, width: f64,
) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Line::new(from, to)
            .style(Style::default().stroke(color, width, 1.0))
            .build()?,
    ))
}

fn panel(
    stage: &mut Stage<'_>, center: Vec3, width: f64, height: f64, rim: Srgb, opacity: f64,
) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Rectangle::new().width(width).height(height)
            .style(Style::default().stroke(rim, 2.0, 1.0).fill(rim, opacity))
            .build()?.moved_to(center),
    ))
}

fn arrow(stage: &mut Stage<'_>, from: Vec3, to: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(
        Arrow::new(from, to).buff(0.0).thickness(width)
            .style(Style::default().fill(color, 1.0).stroke(color, 1.0, 1.0))
            .build()?,
    ))
}

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], appear: bool, duration: f64) -> fmn::Result<()> {
    let mut changes = Vec::new();
    for &mob in mobs {
        let animation = if appear {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?
        };
        changes.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(changes)?;
    Ok(())
}

fn patch(stage: &mut Stage<'_>) -> fmn::Result<(Mob, Vec<Mob>)> {
    let [x, y] = PATCH_CENTER;
    let extent = 1.5 * CELL_WIDTH;
    let rim = panel(stage, v(x, y), extent * 2.0, extent * 2.0, Palette::dim(), 0.08)?;
    let mut faces = Vec::new();
    for offset in [-0.5, 0.5] {
        let d = offset * CELL_WIDTH;
        faces.push(draw_line(stage, v(x + d, y - extent), v(x + d, y + extent), Palette::dim(), 1.5)?);
        faces.push(draw_line(stage, v(x - extent, y + d), v(x + extent, y + d), Palette::dim(), 1.5)?);
    }
    Ok((rim, faces))
}

fn pressure_points(stage: &mut Stage<'_>) -> Vec<Mob> {
    let [x, y] = PATCH_CENTER;
    let color = Palette::gold();
    (-1..=1).flat_map(|i| (-1..=1).map(move |j| (i, j))).map(|(i, j)| {
        let point = v(x + f64::from(i) * CELL_WIDTH, y + f64::from(j) * CELL_WIDTH);
        stage.arena_mut().add(
            Dot::new().point(point).radius(0.089)
                .style(Style::default().fill(color, 1.0).stroke(color, 0.0, 1.0)).build(),
        )
    }).collect()
}

fn interior_face_samples(stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let [x, y] = PATCH_CENTER;
    let color = Palette::blue();
    let mut samples = Vec::new();
    for cross in [-0.5, 0.5] {
        for along in [-1.0, 0.0, 1.0] {
            let (d, c) = (cross * CELL_WIDTH, along * CELL_WIDTH);
            samples.push(arrow(stage, v(x + d - 0.2, y + c), v(x + d + 0.22, y + c), color, 1.55)?);
            samples.push(arrow(stage, v(x + c, y + d - 0.2), v(x + c, y + d + 0.22), color, 1.55)?);
        }
    }
    Ok(samples)
}

fn sealed_coefficients() -> [i64; 4] {
    let mut cells = [[0; 4]; 4];
    for (index, &(start, end)) in SEALED_EDGES.iter().enumerate() {
        cells[start][index] = 1;
        cells[end][index] = -1;
    }
    std::array::from_fn(|index| cells.iter().map(|cell| cell[index]).sum())
}

pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    facts.add("cell.out.right", CELL[1])?
        .add("cell.in.left", -CELL[0])?
        .add("cell.in.top", -CELL[3])?
        .add("cell.net", CELL.iter().sum::<i64>())?
        .add("box.sealed.sum", sealed_coefficients().iter().sum::<i64>())?;
    Ok(facts)
}

// Kit text is registered but detached until the scene admits it.
fn visible_text(
    kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3,
) -> fmn::Result<Mob> {
    let text = kit.text(stage, words, size, color, at)?;
    stage.scene_mut().add(&[text])?;
    Ok(text)
}

/// Wait until `offset` seconds after sentence `j` of line `id` begins, so a picture lands on a word inside
/// that sentence. It is `Kit::until_sentence` with an offset, on the same clock arithmetic (offset 0 is that
/// gate exactly): nothing if the moment has passed or without a narrator, and never past sentence `j + 1`.
fn wait_for_word(kit: &Kit, stage: &mut Stage<'_>, id: &str, j: usize, offset: f64) -> fmn::Result<()> {
    let Some(narrator) = kit.narrator.as_ref() else {
        return Ok(());
    };
    let Some(start) = narrator.sentence_at(id, j) else {
        return Ok(());
    };
    let at = narrator
        .sentence_at(id, j + 1)
        .map_or(start + offset, |next| next.min(start + offset));
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        narration::wait_frames(stage, narration::frames_covering(at - now, fps))?;
    }
    Ok(())
}

impl SceneConstruct for Chapter07 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = &self.kit;
        let [x, y] = PATCH_CENTER;
        let ink = Palette::ink();
        let blue = Palette::blue();
        let dim = Palette::dim();
        let gold = Palette::gold();
        let pressure = Palette::gold();
        k.say(stage, "07_1")?;
        visible_text(k, stage, "A ledger of cells", 45.0, ink, v(0.0, 3.45))?;
        let (rim, faces) = patch(stage)?;
        let mut grid = vec![rim];
        grid.extend_from_slice(&faces);
        fade(stage, &grid, true, 0.65)?;
        let note = visible_text(k, stage, "A patch of the teaching grid", 26.0, dim, v(x, -2.64))?;

        k.say(stage, "07_2")?;
        let pressure_points = pressure_points(stage);
        fade(stage, &pressure_points, true, 0.44)?;
        let pressure_label = visible_text(k, stage, "Pressure at the centers", 30.0, pressure, v(RIGHT, 1.0))?;
        k.until_sentence(stage, "07_2", 2)?;
        let samples = interior_face_samples(stage)?;
        fade(stage, &samples, true, 0.42)?;
        let face_label = visible_text(k, stage, "Velocity across the faces", 30.0, blue, v(RIGHT, -0.05))?;

        k.say(stage, "07_3")?;
        let name = visible_text(k, stage, "Staggered (MAC) grid", 26.0, ink, v(RIGHT, -1.18))?;

        k.say(stage, "07_4")?;
        fade(stage, &samples, false, 0.27)?;
        let reading = panel(stage, v(x, y), CELL_WIDTH, CELL_WIDTH, gold, 0.12)?;
        fade(stage, &[reading], true, 0.33)?;
        stage.scene_mut().remove(&[pressure_label, face_label, name]);
        let net_title = visible_text(k, stage, "Net outflow", 33.0, ink, v(RIGHT, 1.60))?;
        let d = CELL_WIDTH / 2.0;
        let flows = [
            arrow(stage, v(x + d - 0.24, y), v(x + d + 0.44, y), blue, 1.85)?,
            arrow(stage, v(x - d - 0.36, y), v(x - d + 0.20, y), blue, 1.85)?,
            arrow(stage, v(x, y + d + 0.30), v(x, y + d - 0.16), blue, 1.85)?,
        ];
        let items = [
            ("right", CELL[1], 0.78),
            ("left", CELL[0], 0.18),
            ("top", CELL[3], -0.42),
            ("bottom", CELL[2], -1.02),
        ];
        let mut ledger = vec![net_title];
        for (index, (side, value, row)) in items.into_iter().enumerate() {
            // A spoken row waits for its number; the unspoken sealed bottom row follows the top one.
            if let Some(&flow) = flows.get(index) {
                let (_, sentence, offset) = LEDGER_WORDS[index];
                wait_for_word(k, stage, "07_4", sentence, offset)?;
                fade(stage, &[flow], true, 0.28)?;
            }
            ledger.push(visible_text(k, stage, side, 24.0, dim, v(RIGHT - 0.80, row))?);
            ledger.push(visible_text(k, stage, &format!("{value:+}").replace('-', "−"), 29.0,
                               if value == 0 { dim } else { blue }, v(RIGHT + 0.84, row))?);
            stage.wait(0.27)?;
        }
        let rule = draw_line(stage, v(RIGHT - 1.22, -1.40), v(RIGHT + 1.22, -1.40), dim, 1.3)?;
        fade(stage, &[rule], true, 0.2)?;
        wait_for_word(k, stage, "07_4", SUM_WORD.1, SUM_WORD.2)?;
        ledger.push(visible_text(k, stage, &CELL.iter().sum::<i64>().to_string(),
                           39.0, Palette::green(), v(RIGHT + 0.84, -1.85))?);
        ledger.push(rule);

        k.say(stage, "07_5")?;
        stage.scene_mut().remove(&ledger);
        fade(stage, &[flows[1], flows[2]], false, 0.24)?;
        let neighbor = panel(stage, v(x + CELL_WIDTH, y), CELL_WIDTH, CELL_WIDTH, blue, 0.10)?;
        fade(stage, &[neighbor], true, 0.34)?;
        let owner = visible_text(k, stage, &format!("Left cell  +{}", CELL[1]), 28.0, blue, v(RIGHT, 0.68))?;
        let receiver = visible_text(k, stage, &format!("Right cell  −{}", CELL[1]), 28.0, blue, v(RIGHT, -0.08))?;
        k.until_sentence(stage, "07_5", 2)?;
        let cancel = visible_text(k, stage, "Same face, opposite signs", 22.0, dim, v(RIGHT, -1.11))?;
        stage.wait(0.65)?;
        fade(stage, &faces, false, 0.35)?;
        fade(stage, &[flows[0], neighbor, reading], false, 0.30)?;
        stage.scene_mut().remove(&[owner, receiver, cancel, note]);
        fade(stage, &pressure_points, false, 0.25)?;
        let edge_note = visible_text(k, stage, "Only the outer boundary remains", 27.0, gold, v(RIGHT, -0.35))?;

        k.say(stage, "07_6")?;
        stage.scene_mut().remove(&[rim, edge_note]);
        let (x, y) = (-3.5, -0.3);
        let extent = 2.08;
        let sealed = panel(stage, v(x, y), 2.0 * extent, 2.0 * extent, gold, 0.08)?;
        let mut box_mobs = vec![sealed];
        for part in 0..11 {
            let a = -extent + f64::from(part) * (2.0 * extent / 11.0) + 0.065;
            let b = a + 0.23;
            box_mobs.push(draw_line(stage, v(x, y + a), v(x, y + b), dim, 1.7)?);
            box_mobs.push(draw_line(stage, v(x + a, y), v(x + b, y), dim, 1.7)?);
        }
        fade(stage, &box_mobs, true, 0.44)?;
        for (label, dx, dy) in [("A", -0.99, -0.99), ("B", 0.99, -0.99),
                                ("C", -0.99, 0.99), ("D", 0.99, 0.99)] {
            visible_text(k, stage, label, 34.0, ink, v(x + dx, y + dy))?;
        }
        visible_text(k, stage, "Solid walls", 28.0, gold, v(x, -3.05))?;
        visible_text(k, stage, "No flow through a wall", 24.0, dim, v(x, -3.46))?;
        for at in [v(x - 2.38, y), v(x + 2.38, y), v(x, y + 2.40), v(x, y - 2.38)] {
            visible_text(k, stage, "0", 24.0, gold, at)?;
        }
        let mut signs = Vec::new();
        for offset in [-1.02, 1.02] {
            signs.push(arrow(stage, v(x - 0.22, y + offset), v(x + 0.22, y + offset), dim, 1.3)?);
            signs.push(arrow(stage, v(x + offset, y - 0.22), v(x + offset, y + 0.22), dim, 1.3)?);
        }
        fade(stage, &signs, true, 0.36)?;

        k.say(stage, "07_7")?;
        let question = visible_text(k, stage, "Total imbalance?", 36.0, ink, v(RIGHT, 0.73))?;
        let expression = visible_text(k, stage, "A + B + C + D", 30.0, dim, v(RIGHT, -0.16))?;
        k.hold(stage)?;
        k.mark(stage, "predict");
        for second in (1..=5).rev() {
            let clock = visible_text(k, stage, &second.to_string(), 34.0, gold, v(RIGHT, -1.31))?;
            stage.wait(1.0)?;
            stage.scene_mut().remove(&[clock]);
        }

        k.say(stage, "07_8")?;
        stage.scene_mut().remove(&[question, expression]);
        let weights = sealed_coefficients();
        debug_assert_eq!(weights, [0; 4]);
        visible_text(k, stage, &format!("A + B + C + D = {}", weights.iter().sum::<i64>()),
               36.0, Palette::green(), v(RIGHT, 0.63))?;
        visible_text(k, stage, "Each inner face cancels", 26.0, ink, v(RIGHT, -0.26))?;
        visible_text(k, stage, "Every wall contributes zero", 24.0, dim, v(RIGHT, -1.03))?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unit_square_and_sealed_patch_balance() {
        assert_eq!(CELL.iter().sum::<i64>(), 0);
        assert_eq!(sealed_coefficients(), [0; 4]);
    }

    #[test]
    fn the_ledger_waits_for_the_rows_in_spoken_order() {
        const NAMES: [&str; 4] = ["zero", "one", "two", "three"];
        let script = include_str!("../../script.tsv");
        let text = script
            .lines()
            .find_map(|l| l.strip_prefix("07_4\t"))
            .expect("07_4 in the script");
        let mut rest = text;
        let mut last = (0, 0.0);
        // The ledger reveals right, left, top: the order 07_4 reads them, out for positive and in for negative.
        for (&(word, sentence, offset), value) in LEDGER_WORDS.iter().zip([CELL[1], CELL[0], CELL[3]]) {
            assert_eq!(word, NAMES[value.unsigned_abs() as usize]);
            let spoken = format!("{word} {} ", if value > 0 { "out" } else { "in" });
            let at = rest
                .find(&spoken)
                .unwrap_or_else(|| panic!("07_4 says {spoken:?} next"));
            rest = &rest[at + spoken.len()..];
            assert!((sentence, offset) > last, "the gates follow the words");
            last = (sentence, offset);
        }
        assert_eq!(SUM_WORD.0, NAMES[CELL.iter().sum::<i64>().unsigned_abs() as usize]);
        assert!(text.ends_with(" Zero.") && (SUM_WORD.1, SUM_WORD.2) > last);
    }
}
