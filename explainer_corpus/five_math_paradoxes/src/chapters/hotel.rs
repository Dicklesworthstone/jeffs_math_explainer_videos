//! Draft native fmn Hilbert hotel chapter; not installed or built yet.
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use five_math_paradoxes::moved_room;
use fmn::prelude::*;

pub struct Hotel {
    pub kit: Kit,
}

fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], duration: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], duration: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(duration))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Hotel {
    fn name(&self) -> &str {
        "05_hilbert_hotel"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(|_| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("Hotel facts failed"))
        })?;
        let count = facts.get("hotel.examples").and_then(|s| s.parse::<u64>().ok()).ok_or(
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("missing hotel example count"))
        )?;
        kit.say(stage, "05_1")?;
        let question = [
            kit.text(stage, "Every room has a guest", 37.0, Palette::ink(), v(0.0, 2.5))?,
            kit.text(stage, "A new guest arrives. Can everyone have a room?", 29.0, Palette::hero(), v(0.0, -1.0))?,
        ];
        fade_in_all(stage, &question, 0.5)?;
        let mut occupied = Vec::new();
        for n in 1..=count {
            let x = (n as f64 - (count + 1) as f64 / 2.0) * 2.6;
            occupied.push(kit.text(stage, &format!("room {n}: guest {n}"), 23.0, Palette::blue(), v(x, 0.8))?);
        }
        fade_in_all(stage, &occupied, 0.5)?;
        let ellipsis = kit.text(stage, "…", 38.0, Palette::dim(), v(0.0, -0.1))?;
        fade_in_all(stage, &[ellipsis], 0.4)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        let mut leaving = question.to_vec();
        leaving.extend(occupied);
        leaving.push(ellipsis);
        fade_out_all(stage, &leaving, 0.4)?;

        kit.say(stage, "05_2")?;
        let heading = kit.text(stage, "Give each original guest the next room", 34.0, Palette::ink(), v(0.0, 2.7))?;
        fade_in_all(stage, &[heading], 0.4)?;
        let center = |n: u64| (n as f64 - 3.0) * 2.1;
        let mut picture = Vec::new();
        let mut guests = Vec::new();
        let mut continuing_guest = Vec::new();
        for n in 1..=count + 1 {
            let x = center(n);
            picture.push(stage.arena_mut().add(rect(v(x - 0.78, -0.35), 1.56, 1.55, stroke_style(Palette::dim(), 2.5))?));
            picture.push(kit.text(stage, &format!("room {n}"), 21.0, Palette::ink(), v(x, 1.55))?);
            let pos = v(x, 0.4);
            let marker = stage.arena_mut().add(Circle::new().radius(0.23).style(ink_style(Palette::blue())).build().moved_to(pos));
            let label = kit.text(stage, &n.to_string(), 20.0, hex("#0E1116"), pos)?;
            guests.push((marker, label, n));
            if n == count + 1 {
                continuing_guest.extend([marker, label]);
            } else {
                picture.extend([marker, label]);
            }
        }
        let continuation = kit.text(stage, "...", 30.0, Palette::dim(), v(6.2, -0.6))?;
        let schematic = kit.text(stage, "A window on the assignment, not a physical schedule", 21.0, Palette::dim(), v(0.0, -2.3))?;
        picture.extend([continuation, schematic]);
        let mut initial_picture = picture.clone();
        initial_picture.extend(&continuing_guest);
        fade_in_all(stage, &initial_picture, 0.5)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for &(marker, label, n) in &guests {
            let destination = moved_room(n).ok_or(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("hotel destination overflow")))?;
            for mob in [marker, label] {
                let animation = mob.animate().set_anim_args(AnimateArgs { run_time: Some(2.0), ..AnimateArgs::default() })?.move_to(v(center(destination), 0.4), ORIGIN)?;
                moves.push(stage.prepare(animation)?);
            }
        }
        stage.play_prepared(moves)?;
        // The finite window ends here; this guest continues into the ellipsis.
        fade_out_all(stage, &continuing_guest, 0.35)?;
        kit.until_sentence(stage, "05_2", 1)?;
        let newcomer_dot = stage.arena_mut().add(Circle::new().radius(0.23).style(ink_style(Palette::green())).build().moved_to(v(center(1), 0.4)));
        let newcomer_label = kit.text(stage, "new", 17.0, hex("#0E1116"), v(center(1), 0.4))?;
        fade_in_all(stage, &[newcomer_dot, newcomer_label], 0.5)?;
        stage.wait(0.5)?;
        picture.extend([newcomer_dot, newcomer_label]);
        fade_out_all(stage, &picture, 0.4)?;
        let mut rows = Vec::new();
        for n in 1..=count {
            let destination = moved_room(n).ok_or(
                fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("hotel destination overflow"))
            )?;
            let x = if n % 2 == 1 { -2.8 } else { 2.8 };
            let y = 1.5 - ((n - 1) / 2) as f64 * 0.8;
            rows.push(kit.text(stage, &format!("guest {n} → room {destination}"), 25.0, Palette::blue(), v(x, y))?);
        }
        fade_in_all(stage, &rows, 0.6)?;
        let rule = kit.text(stage, "in general: guest n → room n + 1", 30.0, Palette::hero(), v(0.0, -0.7))?;
        let newcomer = kit.text(stage, &format!("new guest → room {}", facts.get("hotel.new_room").unwrap_or("?")), 28.0, Palette::green(), v(0.0, -1.5))?;
        fade_in_all(stage, &[rule, newcomer], 0.5)?;

        kit.say(stage, "05_3")?;
        let caution = kit.text(stage, "A mathematical assignment, not a physical moving plan", 22.0, Palette::dim(), v(0.0, -2.7))?;
        fade_in_all(stage, &[caution], 0.4)?;
        kit.finish(stage)
    }
}
