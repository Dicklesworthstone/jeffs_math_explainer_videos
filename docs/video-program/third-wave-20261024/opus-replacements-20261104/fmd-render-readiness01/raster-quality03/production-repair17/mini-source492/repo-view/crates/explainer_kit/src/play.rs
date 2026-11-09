//! Playback: fluent timing on any animation, a `play!` macro for several at
//! once, lagged groups, tracker tweens, and a fade-out of everything.

use fmn::prelude::*;

/// Set run time or lag on any animation, fluently: `write(..).rt(1.5)`.
pub trait Timed: Sized {
    fn rt(self, run_time: f64) -> Self;
    fn lag(self, lag_ratio: f64) -> Self;
}

impl<A: Animation> Timed for A {
    fn rt(mut self, run_time: f64) -> Self {
        self.update_rate_info(Some(run_time), None, None);
        self
    }
    fn lag(mut self, lag_ratio: f64) -> Self {
        self.update_rate_info(None, None, Some(lag_ratio));
        self
    }
}

/// `play!(stage; a, b, c)`: simultaneous animations, each built just before
/// it is prepared so builders may borrow the stage. `play!(stage, 2.0; ...)`
/// overrides every member's run time. (`stage.play` takes at most a 2-tuple.)
#[macro_export]
macro_rules! play {
    ($stage:ident; $($a:expr),+ $(,)?) => {{
        let mut v: Vec<Box<dyn fmn::prelude::Animation>> = Vec::new();
        $( { let anim = $a; v.push($stage.prepare(anim)?); } )+
        $stage.play_prepared(v)?;
    }};
    ($stage:ident, $rt:expr; $($a:expr),+ $(,)?) => {{
        let mut v: Vec<Box<dyn fmn::prelude::Animation>> = Vec::new();
        $( { let anim = $a; v.push($stage.prepare(anim)?); } )+
        $stage.play_prepared_with(v, fmn::prelude::PlayOverrides {
            run_time: Some($rt),
            ..fmn::prelude::PlayOverrides::default()
        })?;
    }};
}

/// Prepared animations as one lagged group.
pub fn lagged(stage: &mut Stage<'_>, anims: Vec<Box<dyn Animation>>, lag: f64) -> fmn::Result<AnimationGroup> {
    Ok(AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?)
}

/// Animate a value tracker to `value` over `run_time` seconds.
pub fn tween(tracker: Mob, value: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// Fade every top-level mobject out together. Updaters are cleared first, so
/// live readouts fade instead of being redrawn at full opacity each frame.
pub fn fade_all(stage: &mut Stage<'_>, run_time: f64) -> fmn::Result<()> {
    let mobs: Vec<Mob> = stage.scene().mobjects().to_vec();
    if mobs.is_empty() {
        return Ok(());
    }
    let mut v: Vec<Box<dyn Animation>> = Vec::new();
    for m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        v.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        v,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}
