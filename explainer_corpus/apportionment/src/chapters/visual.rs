//! Small native scene helpers. Pending build and picture review.
use explainer_kit::Timed;
use fmn::prelude::*;

pub fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming { fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)? }
            else { fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)? };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}
