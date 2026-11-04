//! Two native FrankenManim quotient scenes; build and review pending.
//! The four source paragraphs bind, in order, to 06_1, 06_2, 07_1, 07_2.
//! Existing ElevenLabs takes remain whole; those cue boundaries are proposals.
//! `a.compose(b)` means do b first, then a; cosets below are left cosets gH.
use crate::model::{Symmetry, left_coset};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, stroke_style, v};
use fmn::prelude::*;

pub struct AmbiguousCosets { pub kit: Kit }
pub struct RotationQuotient { pub kit: Kit }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
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

fn symbol(action: Symmetry) -> String {
    let rotation = match action.turns {
        0 => "",
        1 => "r",
        2 => "r²",
        3 => "r³",
        _ => unreachable!("model normalizes turns"),
    };
    match (rotation, action.reflected) {
        ("", false) => "e".into(),
        ("", true) => "s".into(),
        (r, false) => r.into(),
        (r, true) => format!("{r}s"),
    }
}

fn coset_text(representative: Symmetry, subgroup: &[Symmetry]) -> String {
    let labels: Vec<String> = left_coset(representative, subgroup).into_iter().map(symbol).collect();
    format!("{{{}}}", labels.join(", "))
}

// These panels use the same colored, numbered corners as the other square scenes.
// `action` is computed by the existing exact D4 model.
fn marked_square(stage: &mut Stage<'_>, kit: &Kit, action: Symmetry, center: (f64, f64)) -> fmn::Result<Vec<Mob>> {
    let corners = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
    let colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
    let border = rect(v(center.0 - 1.0, center.1 - 1.0), 2.0, 2.0, stroke_style(Palette::dim(), 3.0))?;
    let mut panel = vec![stage.arena_mut().add(border)];
    for (i, corner) in corners.iter().enumerate() {
        let (x, y) = action.apply(*corner);
        let position = v(center.0 + f64::from(x) * 0.75, center.1 + f64::from(y) * 0.75);
        panel.push(stage.arena_mut().add(Circle::new().radius(0.20).style(ink_style(colors[i])).build().moved_to(position)));
        panel.push(kit.text(stage, &(i + 1).to_string(), 20.0, hex("#0E1116"), position)?);
    }
    Ok(panel)
}

// Endpoints are exact symmetries. The sideways detour avoids overlapping
// labels during the schematic, continuous drawing of a reflection.
fn reflection_path((x,y): (f64,f64), t: f64) -> (f64,f64) {
    (x + 0.30 * (std::f64::consts::PI*t).sin() * y, y * (1.0-2.0*t))
}

fn animate_reflection(stage: &mut Stage<'_>, panel: &[Mob], start: Symmetry, center: (f64,f64)) -> fmn::Result<()> {
    let corners = [(-1,-1),(1,-1),(1,1),(-1,1)];
    for step in 1..=8 {
        let t = f64::from(step)/8.0;
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for (i,corner) in corners.iter().enumerate() {
            let (x,y)=start.apply(*corner);
            let (x,y)=reflection_path((f64::from(x),f64::from(y)),t);
            let destination=v(center.0+0.75*x,center.1+0.75*y);
            for &mob in &panel[1+2*i..=2+2*i] {
                let animation=mob.animate().set_anim_args(AnimateArgs { run_time:Some(0.15), ..AnimateArgs::default() })?.move_to(destination,ORIGIN)?;
                animations.push(stage.prepare(animation)?);
            }
        }
        stage.play_prepared(animations)?;
    }
    Ok(())
}

impl SceneConstruct for AmbiguousCosets {
    fn name(&self) -> &str { "06_ambiguous_cosets" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (e, r, s) = (Symmetry::ID, Symmetry::R, Symmetry::S);
        let h = [e, s];
        let r_h = left_coset(r, &h);
        let left_result = e.compose(r);
        let right_result = s.compose(r);
        let right_class = left_coset(right_result, &h);
        if !r_h.contains(&left_result) || r_h.contains(&right_result) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("coset example is not ambiguous")));
        }
        kit.say(stage, "06_1")?;
        let setup = [
            kit.text(stage, "Can we multiply these classes?", 36.0, Palette::ink(), v(0.0, 2.9))?,
            kit.text(stage, &format!("H = {}", coset_text(e, &h)), 29.0, Palette::blue(), v(-2.7, 1.8))?,
            kit.text(stage, &format!("rH = {}", coset_text(r, &h)), 29.0, Palette::gold(), v(2.7, 1.8))?,
            kit.text(stage, "Choose e or s from the same H; choose r from rH.", 25.0, Palette::hero(), v(0.0, 0.8))?,
            kit.text(stage, "Will the resulting class depend on the choice?", 25.0, Palette::ink(), v(0.0, -0.2))?,
        ];
        appear(stage, &setup)?;
        // Show the two choices themselves during the question, without the
        // products or their resulting classes.
        let mut choices = marked_square(stage, &kit, e, (-3.0, -1.9))?;
        choices.push(kit.text(stage, "choose e", 24.0, Palette::blue(), v(-3.0, -0.65))?);
        choices.extend(marked_square(stage, &kit, s, (3.0, -1.9))?);
        choices.push(kit.text(stage, "choose s", 24.0, Palette::gold(), v(3.0, -0.65))?);
        appear(stage, &choices)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;
        disappear(stage, &choices)?;

        kit.say(stage, "06_2")?;
        if right_class.contains(&left_result) {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("different classes overlap")));
        }
        kit.until_sentence(stage, "06_2", 2)?;
        let mut first = marked_square(stage, &kit, left_result, (-3.0, -1.9))?;
        first.push(kit.text(stage, "r then e", 24.0, Palette::blue(), v(-3.0, -0.65))?);
        first.push(kit.text(stage, &format!("e · r = {}  ∈  {}", symbol(left_result), coset_text(r, &h)), 27.0, Palette::blue(), v(-3.0, -3.25))?);
        appear(stage, &first)?;
        kit.until_sentence(stage, "06_2", 3)?;
        let mut second = marked_square(stage, &kit, r, (3.0, -1.9))?;
        second.push(kit.text(stage, "r then s (schematic)", 22.0, Palette::gold(), v(3.0, -0.65))?);
        appear(stage, &second)?;
        animate_reflection(stage, &second, r, (3.0, -1.9))?;
        let second_result=kit.text(stage, &format!("s · r = {}  ∈  {}", symbol(right_result), coset_text(right_result, &h)), 27.0, Palette::gold(), v(3.0, -3.25))?;
        appear(stage, &[second_result])?;
        kit.finish(stage)
    }
}

impl SceneConstruct for RotationQuotient {
    fn name(&self) -> &str { "07_rotation_quotient" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (e, s) = (Symmetry::ID, Symmetry::S);
        let rotations = [e, Symmetry::R, Symmetry::new(2, false), Symmetry::new(3, false)];
        let classes = [left_coset(e, &rotations), left_coset(s, &rotations)];
        // Check every representative pair before showing the class table.
        for left in &classes {
            for right in &classes {
                let expected = left[0].compose(right[0]).reflected;
                if left.iter().any(|a| right.iter().any(|b| a.compose(*b).reflected != expected)) {
                    return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("class product depends on representative")));
                }
            }
        }
        kit.say(stage, "07_1")?;
        let heading = [
            kit.text(stage, "Ignore rotations, keep reflection", 36.0, Palette::ink(), v(0.0, 2.9))?,
            kit.text(stage, &format!("Rotations R = {}", coset_text(e, &rotations)), 24.0, Palette::blue(), v(0.0, 1.9))?,
            kit.text(stage, &format!("Reflections F = {}", coset_text(s, &rotations)), 24.0, Palette::gold(), v(0.0, 1.2))?,
        ];
        appear(stage, &heading)?;
        let mut table = Vec::new();
        table.push(kit.text(stage, "then combine", 22.0, Palette::dim(), v(-2.8, 0.15))?);
        for (index, name) in ["R", "F"].iter().enumerate() {
            let offset = index as f64 * 1.8;
            table.push(kit.text(stage, name, 26.0, Palette::ink(), v(-0.3 + offset, 0.15))?);
            table.push(kit.text(stage, name, 26.0, Palette::ink(), v(-1.7, -0.7 - offset))?);
        }
        for (row, left) in classes.iter().enumerate() {
            for (column, right) in classes.iter().enumerate() {
                let reflected = left[0].compose(right[0]).reflected;
                table.push(kit.text(stage, if reflected { "F" } else { "R" }, 28.0,
                    if reflected { Palette::gold() } else { Palette::blue() },
                    v(-0.3 + column as f64 * 1.8, -0.7 - row as f64 * 1.8))?);
            }
        }
        kit.until_sentence(stage, "07_1", 3)?;
        appear(stage, &table)?;
        kit.say(stage, "07_2")?;
        kit.until_sentence(stage, "07_2", 2)?;
        let note = kit.text(stage, "Normality keeps every representative choice consistent", 21.0, Palette::dim(), v(0.0, -3.2))?;
        appear(stage, &[note])?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::reflection_path;
    use crate::model::Symmetry;
    #[test]
    fn schematic_reflection_ends_at_the_exact_composite() {
        let r=Symmetry::R;let s=Symmetry::S;
        for corner in [(-1,-1),(1,-1),(1,1),(-1,1)] {
            let (x,y)=r.apply(corner);let (u,v)=reflection_path((f64::from(x),f64::from(y)),1.0);
            let (want_x,want_y)=s.compose(r).apply(corner);
            assert!((u-f64::from(want_x)).abs()<1e-12 && (v-f64::from(want_y)).abs()<1e-12);
        }
    }
}
