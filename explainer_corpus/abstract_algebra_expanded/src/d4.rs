//! Construct the eight square symmetries from the same marked square.
use explainer_kit::{Kit, Palette, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub(crate) struct Card {
    pub(crate) objects: Vec<Mob>,
    pub(crate) markers: Vec<(Mob, Mob)>,
}

const CORNERS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

/// The displayed word r^turns s^flip acts on the rightmost factor first.
pub fn apply(mut point: (i32, i32), turns: u8, flip: bool) -> (i32, i32) {
    if flip {
        point.1 = -point.1;
    }
    for _ in 0..turns {
        point = (-point.1, point.0);
    }
    point
}

pub(crate) fn show_move(
    stage: &mut Stage<'_>,
    markers: &[(Mob, Mob)],
    center: (f64, f64),
    turns: u8,
    flip: bool,
) -> fmn::Result<()> {
    if markers.len() != CORNERS.len() {
        return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(
            "four corners required",
        )));
    }
    if flip {
        for step in 1..=6 {
            let t = f64::from(step) / 6.0;
            let spread = 0.16 * (std::f64::consts::PI * t).sin();
            let mut animations: Vec<Box<dyn Animation>> = Vec::new();
            for (&(dot, label), &(x, y)) in markers.iter().zip(&CORNERS) {
                let px = f64::from(x) * 0.55 + spread * f64::from(y);
                let py = f64::from(y) * 0.55 * (1.0 - 2.0 * t);
                for mob in [dot, label] {
                    let animation = mob
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.12),
                            ..AnimateArgs::default()
                        })?
                        .move_to(v(center.0 + px, center.1 + py), ORIGIN)?;
                    animations.push(stage.prepare(animation)?);
                }
            }
            stage.play_prepared(animations)?;
        }
    }
    for step in 1..=u32::from(turns) * 4 {
        let angle = std::f64::consts::FRAC_PI_2 * f64::from(step) / 4.0;
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for (&(dot, label), &(x, y)) in markers.iter().zip(&CORNERS) {
            let x = f64::from(x) * 0.55;
            let y = f64::from(if flip { -y } else { y }) * 0.55;
            let at = v(
                center.0 + x * angle.cos() - y * angle.sin(),
                center.1 + x * angle.sin() + y * angle.cos(),
            );
            for mob in [dot, label] {
                let animation = mob
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.12),
                        ..AnimateArgs::default()
                    })?
                    .move_to(at, ORIGIN)?;
                animations.push(stage.prepare(animation)?);
            }
        }
        stage.play_prepared(animations)?;
    }
    Ok(())
}

pub(crate) fn card(stage: &mut Stage<'_>, kit: &Kit, center: (f64, f64), name: &str) -> fmn::Result<Card> {
    let outline = stage.arena_mut().add(rect(
        v(center.0 - 0.73, center.1 - 0.73),
        1.46,
        1.46,
        stroke_style(Palette::dim(), 2.5),
    )?);
    let label = kit.text(stage, name, 26.0, Palette::ink(), v(center.0, center.1 - 1.08))?;
    let mut objects = vec![outline, label];
    let mut markers = Vec::new();
    for (i, &(x, y)) in CORNERS.iter().enumerate() {
        let at = v(center.0 + f64::from(x) * 0.55, center.1 + f64::from(y) * 0.55);
        let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let dot = stage.arena_mut().add(
            Circle::new()
                .radius(0.12)
                .style(ink_style(colors[i]))
                .build()
                .moved_to(at),
        );
        let number = kit.text(stage, &(i + 1).to_string(), 17.0, hex("#0E1116"), at)?;
        objects.extend([dot, number]);
        markers.push((dot, number));
    }
    Ok(Card { objects, markers })
}
