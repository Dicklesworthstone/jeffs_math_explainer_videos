//! Uncompiled native FrankenManim scene draft. Requires square_symmetry.rs as
//! `crate::model`, exact cue binding, independent review, and picture checks.
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;
use crate::model::Symmetry;

pub struct Group { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob], duration: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

/// The labels travel with their original corners.
fn board(stage: &mut Stage<'_>, kit: &Kit, center: f64, symmetry: Symmetry) -> fmn::Result<Vec<Mob>> {
    let mut mobs = vec![stage.arena_mut().add(rect(
        v(center - 1.25, -1.1), 2.5, 2.5, stroke_style(Palette::dim(), 3.0),
    )?)];
    let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
    let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
    for (i, corner) in corners.iter().enumerate() {
        let (x, y) = symmetry.apply(*corner);
        let position = v(center + f64::from(x) * 1.0, 0.15 + f64::from(y) * 1.0);
        let dot = Circle::new().radius(0.23).style(ink_style(colors[i])).build().moved_to(position);
        mobs.push(stage.arena_mut().add(dot));
        mobs.push(kit.text(stage, &(i + 1).to_string(), 22.0, hex("#0E1116"), position)?);
    }
    Ok(mobs)
}

/// The endpoint of each move is a genuine square symmetry. During a flip,
/// the small sideways detour keeps crossing labels apart; that path is a
/// schematic visualization, not a claim about intermediate group elements.
fn animate_moves(stage: &mut Stage<'_>, left: &[Mob], right: &[Mob]) -> fmn::Result<()> {
    let corners = [(-1.0_f64, -1.0_f64), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
    let rotate = |(x, y): (f64, f64), t: f64| {
        let angle = std::f64::consts::FRAC_PI_2 * t;
        (x * angle.cos() - y * angle.sin(), x * angle.sin() + y * angle.cos())
    };
    let flip = |(x, y): (f64, f64), t: f64| {
        (x + 0.30 * (std::f64::consts::PI * t).sin() * y, y * (1.0 - 2.0 * t))
    };
    for phase in 0..2 {
        for step in 1..=8 {
            let t = f64::from(step) / 8.0;
            let mut animations: Vec<Box<dyn Animation>> = Vec::new();
            for (i, &p) in corners.iter().enumerate() {
                let l = if phase == 0 { rotate(p, t) } else { flip(rotate(p, 1.0), t) };
                let r = if phase == 0 { flip(p, t) } else { rotate(flip(p, 1.0), t) };
                for (items, center, point) in [(left, -3.1, l), (right, 3.1, r)] {
                    let destination = v(center + point.0, 0.15 + point.1);
                    for &mob in &items[1 + 2 * i..=2 + 2 * i] {
                        let animation = mob.animate().set_anim_args(AnimateArgs {
                            run_time: Some(0.15), ..AnimateArgs::default()
                        })?.move_to(destination, ORIGIN)?;
                        animations.push(stage.prepare(animation)?);
                    }
                }
            }
            stage.play_prepared(animations)?;
        }
    }
    Ok(())
}

impl SceneConstruct for Group {
    fn name(&self) -> &str { "02_group" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "02_1")?;
        let rules = [
            kit.text(stage, "What makes these moves a group?", 35.0, Palette::ink(), v(0.0, 2.7))?,
            kit.text(stage, "Closure: combine moves, get a move", 31.0, Palette::hero(), v(0.0, 1.5))?,
            kit.text(stage, "Identity: a do-nothing move", 31.0, Palette::blue(), v(0.0, 0.55))?,
            kit.text(stage, "Inverses: every move can be undone", 31.0, Palette::gold(), v(0.0, -0.4))?,
            kit.text(stage, "Associativity: regroup without reordering", 31.0, Palette::green(), v(0.0, -1.35))?,
        ];
        appear(stage, &rules[..1], 0.5)?;
        kit.until_sentence(stage, "02_1", 1)?;
        appear(stage, &rules[1..2], 0.4)?;
        kit.until_sentence(stage, "02_1", 2)?;
        appear(stage, &rules[2..3], 0.4)?;
        kit.until_sentence(stage, "02_1", 3)?;
        appear(stage, &rules[3..4], 0.4)?;
        kit.until_sentence(stage, "02_1", 4)?;
        appear(stage, &rules[4..5], 0.4)?;
        kit.say(stage, "02_2")?;
        disappear(stage, &rules)?;
        let headings = [
            kit.text(stage, "Turn, then flip", 26.0, Palette::blue(), v(-3.1, 2.4))?,
            kit.text(stage, "Flip, then turn", 26.0, Palette::gold(), v(3.1, 2.4))?,
            kit.text(stage, "Do the orders land in the same arrangement?", 25.0, Palette::hero(), v(0.0, -2.2))?,
        ];
        appear(stage, &headings, 0.5)?;
        let left = board(stage, &kit, -3.1, Symmetry::ID)?;
        let right = board(stage, &kit, 3.1, Symmetry::ID)?;
        let mut start = left.clone();
        start.extend(right.iter().copied());
        appear(stage, &start, 0.5)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        kit.say(stage, "02_3")?;
        let schematic = kit.text(stage, "Motion paths are schematic", 20.0, Palette::dim(), v(0.0, -2.65))?;
        appear(stage, &[schematic], 0.4)?;
        animate_moves(stage, &left, &right)?;
        let conclusion = kit.text(stage, "Different endpoints: order can matter", 25.0, Palette::green(), v(0.0, -3.0))?;
        appear(stage, &[conclusion], 0.4)?;
        kit.finish(stage)
    }
}
