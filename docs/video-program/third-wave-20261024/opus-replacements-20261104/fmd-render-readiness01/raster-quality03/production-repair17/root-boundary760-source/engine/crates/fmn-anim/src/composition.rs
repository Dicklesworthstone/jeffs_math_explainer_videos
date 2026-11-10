//! Composition — the interval algebra (§9.4, fm-hfe).
//!
//! Mirrors `manimlib/animation/composition.py` at the pinned commit
//! (`3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13`): one interval
//! model, three operators.
//!
//! **The interval model.** [`build_timings`] is the Reference's
//! `build_animations_with_timings`, formula for formula: member *k* occupies
//! `[start_k, start_k + run_time_k]` on the group's internal timeline, and
//! the next member starts at `interpolate(start_k, end_k, lag_ratio)` — so
//! `lag_ratio = 0` stacks every member at zero (simultaneous), `1` lays them
//! end to end (successive), and anything between overlaps them by a fixed
//! fraction of each member's own duration. The group's derived `run_time` is
//! the largest end time; a member's sub-alpha is its window position,
//! clipped to `[0, 1]`, which is what makes a short member hold its end
//! state while a long one finishes.
//!
//! **One normalized-alpha pipeline (§9.4).** A group's own alpha enters the
//! same pipeline every leaf animation uses — [`time_spanned_alpha`] then the
//! rate curve — before it becomes a position on the internal timeline. The
//! group's `lag_ratio` is *not* re-applied as a per-submobject lag: it is
//! already spent in the timings, and applying it twice is the one way to
//! double-count in this design.
//!
//! **Two deliberate divergences (D5, BN-11).**
//!
//! - *The Reference's group ignores its own `rate_func` and `time_span`*
//!   (`AnimationGroup.interpolate` overrides `Animation.interpolate` and
//!   consumes raw alpha), so `AnimationGroup(..., rate_func=there_and_back)`
//!   is silently inert. Here the pipeline runs, and a group's rate curve
//!   defaults to [`RateFunc::linear`] rather than `smooth` so the *default*
//!   composition is bit-for-bit the Reference's: members own their easing;
//!   the group's curve shapes the composition's timeline. Nested groups with
//!   independent rate functions compose exactly as written.
//! - *The Reference's `Succession` ignores its members' run times* — it
//!   picks the active member with `integer_interpolate(0, len, alpha)`, i.e.
//!   equal shares — while deriving its own `run_time` from those very run
//!   times, so `Succession(a(run_time=3), b(run_time=1))` runs 4 seconds and
//!   gives each member 2. And when one frame's alpha step crosses more than
//!   one member, the members in between are never begun or finished, so
//!   their effects vanish. [`Succession`] maps alpha through the same
//!   timings every other operator uses and walks the intermediate members in
//!   order.
//!
//! **What is *not* divergent:** each member still lands on its own
//! `final_alpha_value` at `finish` (a group's own `final_alpha_value` is not
//! a second landing point). That is load-bearing rather than incidental —
//! `FadeOut` finishes at alpha 0 precisely so a removed mobject is left in
//! its original state — and a group that overrode it would break every
//! remover it contains.

use fmn_mobject::{FamilyCheckpoint, Mob, Mobject, Stage};

use crate::animation::{
    AnimConfig, AnimError, AnimState, Animation, AnimationSignature, RateFunc, clip,
    time_spanned_alpha, validate_begin_state,
};

/// The Reference's `DEFAULT_LAGGED_START_LAG_RATIO`.
pub const DEFAULT_LAGGED_START_LAG_RATIO: f64 = 0.05;

// ---------------------------------------------------------------- intervals

/// One member's window on a composition's internal timeline, in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    /// When the member starts.
    pub start: f64,
    /// When the member reaches its own alpha 1.
    pub end: f64,
}

impl Interval {
    /// The window's duration (`end - start`).
    #[must_use]
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }

    /// Where `time` sits inside the window, clipped to `[0, 1]` — the
    /// Reference's `clip((time - start_time) / anim_time, 0, 1)`, with the
    /// same `anim_time == 0 ⇒ 0` degenerate branch.
    #[must_use]
    pub fn sub_alpha(&self, time: f64) -> f64 {
        let duration = self.duration();
        if duration == 0.0 {
            0.0
        } else {
            clip((time - self.start) / duration, 0.0, 1.0)
        }
    }
}

/// The Reference's `build_animations_with_timings`: lay `run_times` out at
/// `lag_ratio`, each member starting where `interpolate(start, end,
/// lag_ratio)` puts it.
#[must_use]
pub fn build_timings(run_times: &[f64], lag_ratio: f64) -> Vec<Interval> {
    let mut out = Vec::with_capacity(run_times.len());
    let mut curr_time = 0.0;
    for &run_time in run_times {
        let start = curr_time;
        let end = start + run_time;
        out.push(Interval { start, end });
        // The Reference's `interpolate(start_time, end_time, lag_ratio)`.
        // Preserve its operation order: reassociation can add one frame
        // when max_end_time reaches the rational clock (fm-7l9k).
        curr_time = (1.0 - lag_ratio) * start + lag_ratio * end;
    }
    out
}

/// The largest end time in a timing table (the Reference's `max_end_time`,
/// `default=0`).
fn max_end_time(timings: &[Interval]) -> f64 {
    timings.iter().map(|i| i.end).fold(0.0, f64::max)
}

/// The shared construction step: the timing table, the derived run time, and
/// the group mobject the composition animates.
fn assemble(
    stage: &mut Stage,
    animations: &[Box<dyn Animation>],
    lag_ratio: f64,
    group: Option<Mob>,
) -> Result<(Vec<Interval>, f64, Mob), AnimError> {
    let run_times: Vec<f64> = animations.iter().map(|a| a.get_run_time()).collect();
    let timings = build_timings(&run_times, lag_ratio);
    let end = max_end_time(&timings);
    let group = match group {
        Some(existing) => {
            if !stage.contains(existing) {
                return Err(AnimError::StaleHandle(existing));
            }
            existing
        }
        None => {
            // The Reference's `VGroup(*remove_list_redundancies(mobs))` —
            // one fresh container over the distinct animated mobjects, in
            // first-appearance order.
            let container = stage.add(Mobject::new());
            let mut seen: Vec<Mob> = Vec::with_capacity(animations.len());
            for animation in animations {
                let mob = animation.state().mobject();
                if seen.contains(&mob) {
                    continue;
                }
                seen.push(mob);
                stage.attach(container, mob)?;
            }
            container
        }
    };
    Ok((timings, end, group))
}

/// The composition's own alpha → a position on its internal timeline: the
/// one normalized-alpha pipeline (`time_span` re-window, then the rate
/// curve), scaled by `max_end_time`.
fn timeline_position(config: &AnimConfig, alpha: f64, max_end_time: f64) -> f64 {
    let spanned = time_spanned_alpha(alpha, config.run_time, config.time_span);
    config.rate_func.eval(spanned) * max_end_time
}

/// The constructor-surface defaults every composition shares: the derived
/// run time and the identity rate curve (see the module note on BN-11).
fn group_config(name: &str, run_time: f64, lag_ratio: f64) -> AnimConfig {
    AnimConfig {
        run_time,
        lag_ratio,
        rate_func: RateFunc::linear(),
        name: name.to_owned(),
        ..AnimConfig::default()
    }
}

// ---------------------------------------------------------- AnimationGroup

/// `AnimationGroup` (composition.py:27): members progress together on one
/// internal timeline, spread by `lag_ratio`.
///
/// Every member is begun eagerly (the Reference's `begin`), so each member's
/// starting copy freezes the state the *composition* started from — which is
/// what makes simultaneous members independent. [`Succession`] is the
/// operator that needs the opposite (each member starting from the previous
/// one's result), and it begins just in time for exactly that reason.
pub struct AnimationGroup {
    state: AnimState,
    animations: Vec<Box<dyn Animation>>,
    timings: Vec<Interval>,
    max_end_time: f64,
}

impl AnimationGroup {
    /// `AnimationGroup(*animations)` — simultaneous (`lag_ratio = 0`), run
    /// time derived from the members. An empty member list is valid: its
    /// timeline has length zero, and an explicit run-time override still
    /// participates in the ordinary scene clock and container lifecycle.
    ///
    /// # Errors
    /// [`AnimError::Stage`] if a member's mobject cannot join the container.
    pub fn new(stage: &mut Stage, animations: Vec<Box<dyn Animation>>) -> Result<Self, AnimError> {
        Self::with_lag_ratio(stage, animations, 0.0)
    }

    /// `AnimationGroup(*animations, lag_ratio=…)`.
    ///
    /// # Errors
    /// As [`AnimationGroup::new`].
    pub fn with_lag_ratio(
        stage: &mut Stage,
        animations: Vec<Box<dyn Animation>>,
        lag_ratio: f64,
    ) -> Result<Self, AnimError> {
        Self::in_group(stage, animations, lag_ratio, None)
    }

    /// The full constructor surface, including the Reference's `group=`
    /// parameter: an explicit container mobject to animate instead of a
    /// fresh one over the members' mobjects (this is how `LaggedStartMap`
    /// keeps the caller's group).
    ///
    /// # Errors
    /// As [`AnimationGroup::new`], plus [`AnimError::StaleHandle`] for a
    /// dead `group` handle.
    pub fn in_group(
        stage: &mut Stage,
        animations: Vec<Box<dyn Animation>>,
        lag_ratio: f64,
        group: Option<Mob>,
    ) -> Result<Self, AnimError> {
        let (timings, max_end_time, group) = assemble(stage, &animations, lag_ratio, group)?;
        Ok(Self {
            state: AnimState::new(
                group,
                group_config("AnimationGroup", max_end_time, lag_ratio),
            ),
            animations,
            timings,
            max_end_time,
        })
    }

    /// Override the derived run time (the Reference's explicit `run_time=`,
    /// which replaces its `-1` "sum of the inputs" sentinel). The internal
    /// timeline is unchanged — the composition simply plays over a
    /// different wall-clock duration.
    #[must_use]
    pub fn with_run_time(mut self, run_time: f64) -> Self {
        self.state.config.run_time = run_time;
        self
    }

    /// Rename the composition (the constructor surface's `name`).
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.state.config.name = name.to_owned();
        self
    }

    /// Replace the rate curve shaping the composition's timeline.
    #[must_use]
    pub fn with_rate_func(mut self, rate_func: RateFunc) -> Self {
        self.state.config.rate_func = rate_func;
        self
    }

    /// The composed members, in argument order.
    #[must_use]
    pub fn animations(&self) -> &[Box<dyn Animation>] {
        &self.animations
    }

    /// The interval table (`anims_with_timings`), member for member.
    #[must_use]
    pub fn timings(&self) -> &[Interval] {
        &self.timings
    }

    /// The internal timeline's length — the Reference's `max_end_time`.
    #[must_use]
    pub fn max_end_time(&self) -> f64 {
        self.max_end_time
    }

    /// The container mobject the composition animates.
    #[must_use]
    pub fn group(&self) -> Mob {
        self.state.mobject()
    }
}

/// The composition-wide lifecycle pieces `AnimationGroup` and `Succession`
/// share verbatim.
macro_rules! composite_common {
    () => {
        fn state(&self) -> &AnimState {
            &self.state
        }

        fn state_mut(&mut self) -> &mut AnimState {
            &mut self.state
        }

        fn collect_resumed_updater_mobjects(&self, out: &mut Vec<Mob>) {
            for animation in &self.animations {
                animation.collect_resumed_updater_mobjects(out);
            }
        }

        /// A composition has no zipped family table: it interpolates
        /// *through its members*, each of which runs its own pipeline over
        /// its own families. This slot is unreachable for compositions and
        /// deliberately does nothing.
        fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {}

        /// The container plus every member's own mobject inventory. The
        /// Reference reports only the container; the purity classifier
        /// (§9.5) scans this list for updaters, and a member's starting or
        /// target copy is exactly the kind of animation-owned mobject that
        /// ticks every frame without being rooted — so hiding them here
        /// would be a misclassification risk (R20).
        fn all_mobjects(&self) -> Vec<Mob> {
            let mut out = vec![self.state.mobject()];
            for animation in &self.animations {
                for mob in animation.all_mobjects() {
                    if !out.contains(&mob) {
                        out.push(mob);
                    }
                }
            }
            out
        }

        /// The pre-begin closure is recursive and deliberately broader than
        /// the interpolation inventory: member targets and other auxiliary
        /// handles may not join the composition container.
        fn preflight_mobjects(&self) -> Vec<Mob> {
            let mut out = vec![self.state.mobject()];
            for animation in &self.animations {
                for mob in animation.preflight_mobjects() {
                    if !out.contains(&mob) {
                        out.push(mob);
                    }
                }
            }
            out
        }

        /// Validate the container and every nested member before the scene
        /// publishes its play lifecycle. This is especially load-bearing for
        /// `Succession`: later members begin just in time, but their static
        /// handles and timing contracts are knowable before frame zero.
        fn validate_begin(&self, stage: &Stage) -> Result<(), AnimError> {
            validate_begin_state(&self.state, stage, [self.state.mobject()])?;
            for animation in &self.animations {
                animation.validate_begin(stage)?;
            }
            Ok(())
        }

        fn can_replay_interpolation(&self) -> bool {
            self.animations
                .iter()
                .all(|animation| animation.can_replay_interpolation())
        }

        /// The Reference's `AnimationGroup.clean_up_from_scene`: removal is
        /// each member's own decision, never the container's.
        fn clean_up_from_scene(&mut self, stage: &mut Stage) {
            for animation in &mut self.animations {
                animation.clean_up_from_scene(stage);
            }
        }
    };
}

impl Animation for AnimationGroup {
    composite_common!();

    fn prepare_interpolation_replay(&mut self) {
        for animation in &mut self.animations {
            animation.prepare_interpolation_replay();
        }
    }

    /// Eagerly begun groups have every member's frozen inputs in the segment
    /// begin snapshot, so purity composes by conjunction.
    fn effect_signature(&self) -> AnimationSignature {
        if self
            .animations
            .iter()
            .all(|animation| matches!(animation.effect_signature(), AnimationSignature::Pure))
        {
            AnimationSignature::Pure
        } else {
            AnimationSignature::Unclassified
        }
    }

    /// The Reference's `AnimationGroup.begin`: mark the container animating,
    /// then begin every member. No starting copy and no zero-interpolate at
    /// this level — each member's own `begin` already lands it at its zero,
    /// which is where the composition's timeline starts it too.
    fn begin(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        begin_composite(&mut self.state, stage)?;
        for animation in &mut self.animations {
            animation.begin(stage)?;
        }
        Ok(())
    }

    /// The Reference's `AnimationGroup.interpolate`, with the group's own
    /// normalized-alpha pipeline in front of it (BN-11).
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        let time = timeline_position(&self.state.config, alpha, self.max_end_time);
        for (animation, interval) in self.animations.iter_mut().zip(&self.timings) {
            animation.interpolate(stage, interval.sub_alpha(time));
        }
    }

    /// The Reference's `AnimationGroup.update_mobjects`: every member ticks
    /// its own animation-owned mobjects.
    fn update_mobjects(&mut self, stage: &mut Stage, dt: f64) {
        for animation in &mut self.animations {
            animation.update_mobjects(stage, dt);
        }
    }

    /// The Reference's `AnimationGroup.finish`: every member lands on its
    /// own `final_alpha_value` (see the module note), then the container
    /// stops animating.
    fn finish(&mut self, stage: &mut Stage) {
        for animation in &mut self.animations {
            animation.finish(stage);
        }
        stage.set_animating_status(self.state.mobject(), false, true);
    }

    fn abort(&mut self, stage: &mut Stage) {
        for animation in &mut self.animations {
            animation.abort(stage);
        }
        stage.set_animating_status(self.state.mobject(), false, true);
    }

    /// A member's deferred failure (a nested [`Succession`], say) surfaces
    /// through the composition that contains it.
    fn deferred_error(&self) -> Option<AnimError> {
        self.animations.iter().find_map(|a| a.deferred_error())
    }
}

/// The composition-level opening of `begin`: the `time_span` widening the
/// canonical sequence performs, plus the container's animating status. The
/// leaf steps (starting copy, zipped families, zero-interpolate) have no
/// meaning for a container that owns no records.
fn begin_composite(state: &mut AnimState, stage: &mut Stage) -> Result<(), AnimError> {
    if let Some((start, end)) = state.config.time_span {
        if end <= start {
            return Err(AnimError::InvalidTimeSpan { start, end });
        }
        state.config.run_time = state.config.run_time.max(end);
    }
    let group = state.mobject();
    if !stage.contains(group) {
        return Err(AnimError::StaleHandle(group));
    }
    stage.set_animating_status(group, true, true);
    Ok(())
}

// --------------------------------------------------------------- Succession

/// `Succession` (composition.py:124): members run strictly one after
/// another (`lag_ratio = 1`), each begun **just in time**.
///
/// Just-in-time `begin` is the whole point of the operator: member *k*'s
/// starting copy must freeze the state member *k-1* left behind, so
/// `Succession(Transform(m, a), Transform(m, b))` walks `m → a → b` instead
/// of snapping back to `m`'s original records. It is also why `Succession`
/// carries a [`Animation::deferred_error`]: `interpolate` has no error
/// channel, and a member whose `begin` fails mid-composition records the
/// failure by name rather than continuing on stale state.
pub struct Succession {
    state: AnimState,
    animations: Vec<Box<dyn Animation>>,
    timings: Vec<Interval>,
    max_end_time: f64,
    active: usize,
    deferred: Option<AnimError>,
    playback: Vec<MemberPlayback>,
    replayable: bool,
    replaying: bool,
    last_time: Option<f64>,
}

/// At most three content checkpoints per initialized member, independent of
/// frame count. Lifecycle callbacks run once; revisiting a member restores its
/// already-prepared or already-finished content instead of calling them again.
struct MemberPlayback {
    before: Option<FamilyCheckpoint>,
    prepared: Option<FamilyCheckpoint>,
    completed: Option<FamilyCheckpoint>,
    finished: bool,
}

impl Succession {
    /// `Succession(*animations)` — `lag_ratio = 1`.
    ///
    /// # Errors
    /// [`AnimError::EmptyComposition`] for an empty member list;
    /// otherwise as [`AnimationGroup::new`].
    pub fn new(stage: &mut Stage, animations: Vec<Box<dyn Animation>>) -> Result<Self, AnimError> {
        Self::with_lag_ratio(stage, animations, 1.0)
    }

    /// `Succession(*animations, lag_ratio=…)` — the Reference exposes the
    /// knob; anything below `1` overlaps neighbours, and the just-in-time
    /// begin then applies to whichever member the timeline is inside.
    ///
    /// # Errors
    /// As [`Succession::new`].
    pub fn with_lag_ratio(
        stage: &mut Stage,
        animations: Vec<Box<dyn Animation>>,
        lag_ratio: f64,
    ) -> Result<Self, AnimError> {
        // Unlike a simultaneous group, a succession must have an active
        // member. Refuse before allocating a container or indexing member 0.
        if animations.is_empty() {
            return Err(AnimError::EmptyComposition);
        }
        let (timings, max_end_time, group) = assemble(stage, &animations, lag_ratio, None)?;
        Ok(Self {
            state: AnimState::new(group, group_config("Succession", max_end_time, lag_ratio)),
            animations,
            timings,
            max_end_time,
            active: 0,
            deferred: None,
            playback: Vec::new(),
            replayable: false,
            replaying: false,
            last_time: None,
        })
    }

    /// Override the derived run time.
    #[must_use]
    pub fn with_run_time(mut self, run_time: f64) -> Self {
        self.state.config.run_time = run_time;
        self
    }

    /// Replace the rate curve shaping the composition's timeline.
    #[must_use]
    pub fn with_rate_func(mut self, rate_func: RateFunc) -> Self {
        self.state.config.rate_func = rate_func;
        self
    }

    /// The composed members, in argument order.
    #[must_use]
    pub fn animations(&self) -> &[Box<dyn Animation>] {
        &self.animations
    }

    /// The interval table, member for member.
    #[must_use]
    pub fn timings(&self) -> &[Interval] {
        &self.timings
    }

    /// The internal timeline's length.
    #[must_use]
    pub fn max_end_time(&self) -> f64 {
        self.max_end_time
    }

    /// The index of the member the timeline is currently inside.
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// The container mobject.
    #[must_use]
    pub fn group(&self) -> Mob {
        self.state.mobject()
    }

    /// The member whose window contains `time`: the last member that has
    /// started. Overlapping windows (`lag_ratio < 1`) resolve to the latest
    /// one started, which is the member the Reference's index arithmetic
    /// would also be inside.
    fn member_at(&self, time: f64) -> usize {
        let mut index = 0;
        for (i, interval) in self.timings.iter().enumerate() {
            if time >= interval.start {
                index = i;
            }
        }
        index
    }

    fn replay_sources_are_static(&self, stage: &Stage) -> bool {
        self.can_replay_interpolation()
            && self.animations.iter().all(|animation| {
                animation
                    .preflight_mobjects()
                    .into_iter()
                    .chain(animation.all_mobjects())
                    .all(|mob| !stage.has_updaters_in_family(mob))
            })
    }

    fn begin_member(&mut self, stage: &mut Stage, index: usize) -> Result<(), AnimError> {
        let mob = self.animations[index].state().mobject();
        let before = self
            .replayable
            .then(|| stage.checkpoint_family(mob))
            .transpose()?;
        // Own the attempted lifetime before calling user code. A canonical
        // begin may already have suspended the family when its zero sample
        // reports a deferred error; the segment's abort must still visit it.
        self.playback.push(MemberPlayback {
            before,
            prepared: None,
            completed: None,
            finished: false,
        });
        self.animations[index].begin(stage)?;
        if let Some(error) = self.animations[index].deferred_error() {
            return Err(error);
        }
        // A setup hook may have installed updaters on an auxiliary object.
        // Such state must never be silently reset by the content cache.
        self.replayable &= self.replay_sources_are_static(stage);
        self.playback[index].prepared = self
            .replayable
            .then(|| stage.checkpoint_family(mob))
            .transpose()?;
        Ok(())
    }

    /// Rebuild a visited member's local context. Undo later members in reverse
    /// order, then apply finished predecessors in author order. This handles
    /// both disjoint mobjects and chains that mutate the same family, including
    /// alignments that changed its child topology or record schema.
    fn restore_member(&self, stage: &mut Stage, target: usize) -> Result<(), AnimError> {
        for member in self.playback.iter().rev() {
            if let Some(before) = &member.before {
                stage.restore_family_checkpoint(before)?;
            }
        }
        for member in self.playback.iter().take(target) {
            if let Some(completed) = &member.completed {
                stage.restore_family_checkpoint(completed)?;
            }
        }
        if let Some(prepared) = &self.playback[target].prepared {
            stage.restore_family_checkpoint(prepared)?;
        }
        Ok(())
    }

    fn finish_member(&mut self, stage: &mut Stage, index: usize) -> Result<(), AnimError> {
        if self.playback[index].finished {
            if let Some(completed) = &self.playback[index].completed {
                stage.restore_family_checkpoint(completed)?;
            }
            return Ok(());
        }
        self.animations[index].finish(stage);
        self.playback[index].finished = true;
        if let Some(error) = self.animations[index].deferred_error() {
            return Err(error);
        }
        if self.replayable {
            self.playback[index].completed =
                Some(stage.checkpoint_family(self.animations[index].state().mobject())?);
        }
        Ok(())
    }

    /// First visits still walk and run every intermediate member. Revisits
    /// never begin an animation a second time, so setup/teardown effects and
    /// native starting copies retain their original lifetime.
    fn advance_to(&mut self, stage: &mut Stage, target: usize) {
        while self.playback.len() <= target && self.deferred.is_none() {
            let previous = self.playback.len() - 1;
            let result = (|| {
                if self.replayable && self.replaying {
                    self.restore_member(stage, previous)?;
                    self.animations[previous].prepare_interpolation_replay();
                }
                self.finish_member(stage, previous)?;
                self.active = previous + 1;
                self.begin_member(stage, self.active)
            })();
            if let Err(error) = result {
                self.deferred = Some(error);
            }
        }
    }
}

impl Animation for Succession {
    composite_common!();

    fn prepare_interpolation_replay(&mut self) {
        self.replaying = true;
    }

    /// Later members' starting copies do not exist in the segment's begin
    /// snapshot. Restoring that snapshot while retaining an advanced member
    /// index would leave stale inputs, even when every leaf is pure. Timeline
    /// seek therefore uses its existing checkpoint-and-serial-replay route.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Unclassified
    }

    /// The Reference's `Succession.begin`: only the first member begins —
    /// the rest begin as the timeline reaches them.
    fn begin(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        begin_composite(&mut self.state, stage)?;
        self.active = 0;
        self.deferred = None;
        self.playback.clear();
        self.replaying = false;
        self.last_time = None;
        self.replayable = self.replay_sources_are_static(stage);
        self.begin_member(stage, 0)
    }

    /// Locate the member the timeline is inside, walking (and running) any
    /// members passed on the way, then interpolate it at its window
    /// position.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        if self.deferred.is_some() || self.playback.is_empty() {
            return;
        }
        // The rational clock may round the final raw alpha above one. Its
        // clipped endpoint and finish(1) are the same timeline position, not
        // a backwards traversal of stateful content.
        let time = clip(
            timeline_position(&self.state.config, alpha, self.max_end_time),
            0.0,
            self.max_end_time,
        );
        self.replayable &= self.replay_sources_are_static(stage);
        if self.last_time.is_some_and(|last| time < last) {
            if !self.replayable {
                self.deferred = Some(AnimError::StatefulSuccessionRewind);
                return;
            }
            self.replaying = true;
        }
        let target = self.member_at(time);
        self.advance_to(stage, target);
        if self.deferred.is_some() {
            return;
        }
        if self.replayable && self.replaying {
            if let Err(error) = self.restore_member(stage, target) {
                self.deferred = Some(error);
                return;
            }
            self.animations[target].prepare_interpolation_replay();
        }
        self.active = target;
        let sub_alpha = self.timings[self.active].sub_alpha(time);
        self.animations[self.active].interpolate(stage, sub_alpha);
        self.last_time = Some(time);
    }

    /// The Reference's `Succession.update_mobjects`: only the active member
    /// ticks — the others own no live state yet, or no longer do.
    fn update_mobjects(&mut self, stage: &mut Stage, dt: f64) {
        if self.deferred.is_none() {
            self.animations[self.active].update_mobjects(stage, dt);
        }
    }

    /// Land the timeline on `final_alpha_value` (which walks the remaining
    /// members through `interpolate`), finish the member left active, and
    /// stop the container animating.
    fn finish(&mut self, stage: &mut Stage) {
        // A reverse curve may leave the most recently begun member paused
        // behind the current view. Balance that lifetime once before landing
        // on the composition's final member; restoring checkpoints then puts
        // the final view back without repeating any lifecycle callback.
        if self.deferred.is_none()
            && let Some(last) = self.playback.len().checked_sub(1)
            && last != self.active
            && !self.playback[last].finished
        {
            let result = (|| {
                self.restore_member(stage, last)?;
                self.animations[last].prepare_interpolation_replay();
                self.finish_member(stage, last)
            })();
            if let Err(error) = result {
                self.deferred = Some(error);
            }
        }
        let final_alpha = self.state.config.final_alpha_value;
        self.interpolate(stage, final_alpha);
        if self.deferred.is_none() {
            if let Err(error) = self.finish_member(stage, self.active) {
                self.deferred = Some(error);
            }
        } else {
            // A named reverse refusal must not leave the active family's
            // updaters suspended after the segment driver reports the error.
            self.abort(stage);
        }
        stage.set_animating_status(self.state.mobject(), false, true);
    }

    fn abort(&mut self, stage: &mut Stage) {
        for (animation, playback) in self.animations.iter_mut().zip(&mut self.playback) {
            if !playback.finished {
                animation.abort(stage);
                playback.finished = true;
            }
        }
        stage.set_animating_status(self.state.mobject(), false, true);
    }

    /// A member's just-in-time `begin` failure, recorded by name.
    fn deferred_error(&self) -> Option<AnimError> {
        self.deferred.clone().or_else(|| {
            self.animations
                .iter()
                .take(self.active + 1)
                .find_map(|a| a.deferred_error())
        })
    }
}

// -------------------------------------------------------------- LaggedStart

/// `LaggedStart(*animations)` (composition.py:156): an [`AnimationGroup`] at
/// the Reference's 5 % lag.
///
/// # Errors
/// As [`AnimationGroup::new`].
pub fn lagged_start(
    stage: &mut Stage,
    animations: Vec<Box<dyn Animation>>,
) -> Result<AnimationGroup, AnimError> {
    Ok(
        AnimationGroup::with_lag_ratio(stage, animations, DEFAULT_LAGGED_START_LAG_RATIO)?
            .with_name("LaggedStart"),
    )
}

/// `LaggedStartMap(anim_func, group)` (composition.py:166): build one
/// animation per submobject of `group`, lag them by 5 %, and animate the
/// caller's group itself (`run_time = 2.0`, the Reference's default).
///
/// `anim_func` is the Reference's `anim_func(submob, **kwargs)`; it receives
/// the stage because most constructors need it (a target copy, a path
/// query), and it may fail by name. A childless group is valid: the factory
/// is never called, the caller's group is retained, and the default two-second
/// duration still uses the ordinary animation lifecycle.
///
/// # Errors
/// [`AnimError::StaleHandle`] for a dead `group`, and whatever `anim_func`
/// reports.
pub fn lagged_start_map<F>(
    stage: &mut Stage,
    group: Mob,
    mut anim_func: F,
) -> Result<AnimationGroup, AnimError>
where
    F: FnMut(&mut Stage, Mob) -> Result<Box<dyn Animation>, AnimError>,
{
    let submobjects = stage
        .get(group)
        .map(|entry| entry.submobjects().to_vec())
        .ok_or(AnimError::StaleHandle(group))?;
    let mut animations = Vec::with_capacity(submobjects.len());
    for submob in submobjects {
        animations.push(anim_func(stage, submob)?);
    }
    Ok(AnimationGroup::in_group(
        stage,
        animations,
        DEFAULT_LAGGED_START_LAG_RATIO,
        Some(group),
    )?
    .with_run_time(2.0)
    .with_name("LaggedStartMap"))
}
