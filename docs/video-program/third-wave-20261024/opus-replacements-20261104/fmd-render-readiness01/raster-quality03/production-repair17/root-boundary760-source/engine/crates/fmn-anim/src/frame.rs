//! The six-step frame order + the FramePacket freeze (§9.3, §1.4, D-19,
//! fm-x79).
//!
//! The load-bearing order, exactly the Reference's
//! (`scene.py::progress_through_animations` + `update_frame`):
//!
//! 1. animation `update_mobjects(dt)` — per animation, interleaved with
//! 2. animation `interpolate(alpha)` — `alpha = t / run_time`, raw and
//!    **unclamped** (an animation shorter than its group's progression
//!    overshoots past 1; the §9.4 pipeline's clip absorbs it — that is the
//!    Reference's mechanism for unequal run_times in one `play`);
//! 3. time advances;
//! 4. **scene updaters run, observing post-interpolation state** —
//!    `always_redraw`-style scenes semantically depend on step 4 seeing
//!    step 2's output, and on time having already advanced (the
//!    Reference's `increment_time` precedes `update_mobjects`);
//! 5. capture — the serial-front-end boundary hook runs, then Rev 4's
//!    immutable [`FramePacket`] freezes. The hook is where Proscenium drains
//!    journaled input events; it is still part of capture preparation, never a
//!    concurrent or mid-capture mutation;
//! 6. emit — the packet leaves through the sink, in frame order.
//!
//! This order is a contract, not an implementation choice. The update-order
//! corpus (`tests/frame_order.rs`) locks it.
//!
//! **The FramePacket** carries the frame's index and alpha, the exact
//! rational capture time, the scene's RNG identity, and a CoW freeze of
//! the scene state. It is immutable by construction: no `&mut` access
//! exists, writes to the live stage after the freeze unshare instead of
//! leaking in (O(touched) cost, inherited from the §8.1 snapshot and
//! verified by test), and RNG state is *derived*, not carried — a keyed
//! per-frame fork is a pure function of `(seed, substream name, frame
//! index)` (§6.5), so [`FramePacket::rng_fork`] reproduces any
//! substream's frame stream from two words. Camera state and the
//! revisioned references into Lumen's compiled render IR join the packet
//! when those subsystems land (fm-gw7, fm-5xm) — a subset of the final
//! abstraction, never a substitute.
//!
//! Skip-mode semantics: the same frame-grid transitions run in the same order,
//! but capture and emit are disabled. This is what makes the terminal arena
//! state bit-identical even for nonlinear/stateful updaters; replacing those
//! transitions with one large `dt` would preserve only total elapsed time.
//! The post-play updater pass runs at `dt = 0` in **both** modes. The Reference
//! instead uses one large step and then applies `run_time` a second time — the
//! defect corrected and documented in BN-10.

use std::fmt;
use std::rc::Rc;

use fmn_core::rng::{Pcg64Dxsm, RngRoot};
use fmn_mobject::{Mob, Snapshot, Stage};

use crate::animation::{AnimError, Animation};
use crate::clock::{FrameSample, FrameSegment, RationalFrameClock, RationalTime};
use crate::purity::{
    ImpureEffect, Purity, SegmentKind, SegmentReport, classify_play, classify_wait,
};

/// The immutable frame boundary (§9.3 step 5): everything after capture
/// consumes only this. Cheap to hold several in flight (CoW snapshot
/// references, not deep copies — §17.4's 3–6 frame budget).
#[derive(Clone)]
pub struct FramePacket {
    frame_index: i64,
    segment_frame: i64,
    alpha: f64,
    time: RationalTime,
    rng_seed: u64,
    state: Rc<Snapshot>,
}

impl FramePacket {
    /// Freeze the current scene state. Called by the drivers at step 5;
    /// public because the Studio's scrubbing and the replay journal freeze
    /// at barriers too.
    #[must_use]
    pub fn freeze(
        stage: &Stage,
        clock: &RationalFrameClock,
        rng: &RngRoot,
        sample: &FrameSample,
    ) -> Self {
        Self {
            frame_index: clock.now().frames(),
            segment_frame: sample.frame,
            alpha: sample.alpha,
            time: clock.now(),
            rng_seed: rng.seed(),
            state: Rc::new(stage.snapshot()),
        }
    }

    /// Freeze the current state outside a sampled animation segment.
    ///
    /// `Scene::show`, presenter holds, Studio barriers, and replay
    /// checkpoints all need the same immutable boundary as an ordinary
    /// captured frame without inventing a zero-duration animation. A barrier
    /// packet therefore uses `segment_frame = 0` as an explicit sentinel and
    /// `alpha = 1`; its global frame index and exact time still come from the
    /// rational clock.
    #[must_use]
    pub fn freeze_barrier(stage: &Stage, clock: &RationalFrameClock, rng: &RngRoot) -> Self {
        Self {
            frame_index: clock.now().frames(),
            segment_frame: 0,
            alpha: 1.0,
            time: clock.now(),
            rng_seed: rng.seed(),
            state: Rc::new(stage.snapshot()),
        }
    }

    /// The global frame index (the clock's frame counter at capture) —
    /// also the key every per-frame RNG fork derives from.
    #[must_use]
    pub fn frame_index(&self) -> i64 {
        self.frame_index
    }

    /// The 1-based frame number within its segment, or zero for a
    /// [`FramePacket::freeze_barrier`] packet.
    #[must_use]
    pub fn segment_frame(&self) -> i64 {
        self.segment_frame
    }

    /// The segment alpha at capture (clamped to `[0, 1]`, §9.2).
    #[must_use]
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// The exact rational scene time at capture.
    #[must_use]
    pub fn time(&self) -> RationalTime {
        self.time
    }

    /// The frozen scene state (restore into a [`Stage`] to reconstruct the
    /// frame's front-end state — §9.5's frame-parallel workers do exactly
    /// this from the begin-state snapshot).
    #[must_use]
    pub fn state(&self) -> &Snapshot {
        &self.state
    }

    /// Materialize this packet's frozen Stage in an independent physical
    /// arena, synchronized to the packet's exact capture time.
    ///
    /// This is the Scene → Lumen/Reel adapter path: downstream work consumes
    /// only immutable packet state and never borrows the live scene. The
    /// materialized arena shares the packet's logical handle domain but its
    /// record writes are CoW-isolated.
    #[must_use]
    pub fn materialize_stage(&self) -> Stage {
        let mut stage = self.state.materialize();
        stage.set_time_from_clock(self.time.to_f64());
        stage
    }

    /// The keyed per-frame fork of a named substream (§6.5): a pure
    /// function of `(seed, name, frame index)`, identical from any thread
    /// in any order — the property that makes frame parallelism
    /// replay-identical by construction (D-18).
    #[must_use]
    pub fn rng_fork(&self, substream: &str) -> Pcg64Dxsm {
        RngRoot::from_seed(self.rng_seed)
            .substream(substream)
            .fork_frame(self.frame_index.cast_unsigned())
    }
}

/// Scene-updater pass without a time advance (the Reference's bare
/// `Scene.update_mobjects(dt)` — used by `finish_animations` and the top
/// of `wait`; `Stage::update` is steps 3–4 fused).
fn update_scene_mobjects(stage: &mut Stage, dt: f64) {
    for root in stage.roots().to_vec() {
        stage.update_mobject(root, dt);
    }
}

/// Whether `mob` is already in some rooted family (the Reference's
/// membership probe in `begin_animations`).
fn in_scene_family(stage: &Stage, mob: Mob) -> bool {
    stage
        .roots()
        .to_vec()
        .into_iter()
        .any(|root| stage.family(root).contains(&mob))
}

/// The Reference's `begin_animations`: begin each animation, then root any
/// animated mobject not already on stage (begin first — its starting copy
/// must not be swept into the membership probe).
fn begin_animations(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
) -> Result<(), AnimError> {
    let before_begin = stage.snapshot();
    let result = (|| {
        for animation in animations.iter_mut() {
            animation.begin(stage)?;
            let mobject = animation.state().mobject();
            if !in_scene_family(stage, mobject) {
                stage
                    .add_to_scene(mobject)
                    .map_err(|_| AnimError::StaleHandle(mobject))?;
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(()),
        Err(error) => {
            stage.restore(&before_begin);
            Err(error)
        }
    }
}

/// One sample prepared through the animation and clock phases. Playback and
/// skip both prepare the same sample at `1/fps`; the completion call decides
/// whether to capture it.
#[derive(Debug, Clone, Copy)]
struct StepPlan {
    sample: FrameSample,
    dt: f64,
    updaters_completed: bool,
}

/// An external scene callback is not reconstructible from a begin snapshot.
/// Keep the journal conservative even when no native updater slot represents it.
fn note_external_updaters(report: &mut SegmentReport) {
    match &mut report.purity {
        Purity::Pure => report.purity = Purity::Stateful(vec![ImpureEffect::SceneUpdater]),
        Purity::Stateful(effects) => {
            if !effects.contains(&ImpureEffect::SceneUpdater) {
                effects.push(ImpureEffect::SceneUpdater);
            }
        }
    }
    report.begin_state = None;
}

fn mark_updaters_completed(prepared: &mut Option<StepPlan>) -> Result<(), AnimError> {
    let plan = prepared.as_mut().ok_or(AnimError::InvalidFramePhase(
        "prepare a frame before completing its external scene updaters",
    ))?;
    if plan.updaters_completed {
        return Err(AnimError::InvalidFramePhase(
            "the prepared frame's scene updaters were already completed",
        ));
    }
    plan.updaters_completed = true;
    Ok(())
}

/// The host-language release point inside step 4 of the six-step frame
/// contract.
///
/// Choreo has completed animation updates/interpolation and advanced the
/// rational clock, but has not yet run native scene updaters or frozen the
/// frame. A front end may release its Scene borrow, run its own ordered scene
/// updaters using [`Self::dt`], then resume this exact frame through
/// [`complete_play_frame`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUpdaterBoundary {
    /// The frame-local updater delta (`1 / fps`).
    pub dt: f64,
    /// Exact scene time after step 3.
    pub time: RationalTime,
    /// The sample that will be frozen after native scene updaters run.
    pub sample: FrameSample,
}

/// One completed step-2 animation interpolation in an open frame.
///
/// A host-language animation slot can use this yield to run its callback
/// after the slot's native placeholder and before the next animation in the
/// list. The Scene borrow is owned by the caller, so it can be dropped at this
/// exact ordering point without weakening Stage ownership.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationBoundary {
    /// Index in the play's top-level animation list.
    pub animation_index: usize,
    /// Raw `sample_time / animation_run_time`, matching Choreo's driver.
    pub alpha: f64,
    /// The frame-local delta used by `update_mobjects`.
    pub dt: f64,
    /// Current segment sample; the rational clock has not advanced yet.
    pub sample: FrameSample,
}

/// The Reference's `finish_animations`: finish each animation, apply
/// remover cleanup, then one scene-updater pass at `dt = 0` — in both
/// modes (the skip-mode `dt = run_time` double-application is the BN-10
/// correction).
fn finish_animations(stage: &mut Stage, animations: &mut [Box<dyn Animation>]) {
    for animation in animations.iter_mut() {
        animation.finish(stage);
        // `clean_up_from_scene` is the removal decision: leaves consult
        // their own `remover` flag, composition operators (§9.4) delegate to
        // their members — a remover composed into a group is still a
        // remover, and the container never is.
        animation.clean_up_from_scene(stage);
    }
    update_scene_mobjects(stage, 0.0);
}

/// What `begin_animations` + classification establish, before any sample
/// runs: the segment on the frame grid, its classification, the frame the
/// clock stood at, and the begin-state snapshot pure segments carry.
struct Prologue {
    segment: FrameSegment,
    purity: crate::purity::Purity,
    base_frame: i64,
    begin_state: Option<Rc<Snapshot>>,
    run_time: f64,
}

/// Validate a play segment without beginning an animation or changing the
/// stage or clock.
///
/// Proscenium uses this before emitting `PrePlay`; the Choreo drivers repeat
/// it at their public boundary so direct callers receive the same refusal.
/// Validation walks nested compositions through [`Animation::validate_begin`]
/// and asks the rational clock to prove the longest run time fits its frame
/// counter.
///
/// # Errors
/// A begin-time validation error, or [`AnimError::Clock`] when the longest
/// run time cannot be represented by `clock`.
pub fn validate_play(
    stage: &Stage,
    clock: &RationalFrameClock,
    animations: &[Box<dyn Animation>],
) -> Result<(), AnimError> {
    plan_play(stage, clock, animations).map(|_| ())
}

/// The read-only half of a play prologue: validate its closure and derive
/// the exact frame segment before `begin` can mutate the arena.
fn plan_play(
    stage: &Stage,
    clock: &RationalFrameClock,
    animations: &[Box<dyn Animation>],
) -> Result<(FrameSegment, f64), AnimError> {
    for animation in animations {
        animation.validate_begin(stage)?;
    }
    let run_time = animations
        .iter()
        .map(|animation| animation.get_run_time())
        .fold(f64::NEG_INFINITY, f64::max);
    let segment = clock.segment(run_time).map_err(AnimError::Clock)?;
    Ok((segment, run_time))
}

/// The opening of every play segment, whole or partial.
fn play_prologue(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    animations: &mut [Box<dyn Animation>],
    skip: bool,
) -> Result<Prologue, AnimError> {
    let (segment, run_time) = plan_play(stage, clock, animations)?;
    begin_animations(stage, animations)?;
    let purity = classify_play(stage, animations);
    let base_frame = clock.now().frames();
    let begin_state = (purity.is_pure() && !skip).then(|| Rc::new(stage.snapshot()));
    Ok(Prologue {
        segment,
        purity,
        base_frame,
        begin_state,
        run_time,
    })
}

impl Prologue {
    /// The journal record for the segment this prologue opened.
    fn report(self, kind: SegmentKind) -> SegmentReport {
        SegmentReport {
            kind,
            purity: self.purity,
            begin_state: self.begin_state,
            base_frame: self.base_frame,
            n_frames: self.segment.n_frames(),
            run_time: self.run_time,
        }
    }
}

/// A `play()` segment **opened but not finished** — the seek/scrub handle
/// (§9.4's `Timeline`, §13.5's scrubbing).
///
/// Opening runs the whole prologue and nothing else: the animations are
/// begun, the segment is classified, and a pure segment's begin-state
/// snapshot is taken. From there the caller either reconstructs any frame in
/// O(1) through [`reconstruct_pure_frame`](crate::purity::reconstruct_pure_frame)
/// (pure segments) or steps frames with [`advance_play`] (stateful ones) —
/// in both cases through this same driver, so a sought frame and a played
/// frame are the same frame by construction rather than by agreement.
pub struct OpenSegment {
    report: SegmentReport,
    segment: FrameSegment,
    stepped: i64,
    current_sample: Option<FrameSample>,
    next_animation: usize,
    prepared: Option<StepPlan>,
    animations_finished: bool,
    final_updaters_completed: bool,
}

impl OpenSegment {
    /// The segment's journal record (classification, begin state, frame
    /// range).
    #[must_use]
    pub fn report(&self) -> &SegmentReport {
        &self.report
    }

    /// The sampling plan the segment runs on.
    #[must_use]
    pub fn segment(&self) -> FrameSegment {
        self.segment
    }

    /// How many of its frames have been stepped so far.
    #[must_use]
    pub fn stepped(&self) -> i64 {
        self.stepped
    }

    /// Mark the prepared frame's whole scene-updater phase as completed by
    /// an unborrowed host. Completion still dispatches events and captures.
    ///
    /// # Errors
    /// No prepared frame exists, or this phase was already marked complete.
    pub fn mark_scene_updaters_complete(&mut self) -> Result<(), AnimError> {
        mark_updaters_completed(&mut self.prepared)?;
        note_external_updaters(&mut self.report);
        Ok(())
    }

    /// Mark the final zero-dt scene update complete after animation cleanup.
    ///
    /// # Errors
    /// Animation cleanup is not complete, a frame is pending, or the final
    /// updater phase was already marked complete.
    pub fn mark_final_scene_updaters_complete(&mut self) -> Result<(), AnimError> {
        if !self.animations_finished
            || self.prepared.is_some()
            || self.current_sample.is_some()
            || self.final_updaters_completed
        {
            return Err(AnimError::InvalidFramePhase(
                "finish animation lifecycles before completing final scene updaters once",
            ));
        }
        self.final_updaters_completed = true;
        note_external_updaters(&mut self.report);
        Ok(())
    }

    /// Consume the handle for its report (after finishing, or discarding).
    #[must_use]
    pub fn into_report(self) -> SegmentReport {
        self.report
    }
}

/// Open a `play()` segment: begin the animations, classify the segment, and
/// stop. Nothing is emitted and the clock does not move.
///
/// # Errors
/// As [`play_segment`].
pub fn open_play(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    animations: &mut [Box<dyn Animation>],
) -> Result<OpenSegment, AnimError> {
    open_play_with_mode(stage, clock, animations, false)
}

/// Open a play while carrying the caller's effective skip mode into purity
/// classification and begin-state retention.
///
/// # Errors
/// As [`open_play`].
pub fn open_play_with_mode(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    animations: &mut [Box<dyn Animation>],
    skip: bool,
) -> Result<OpenSegment, AnimError> {
    let prologue = play_prologue(stage, clock, animations, skip)?;
    let segment = prologue.segment;
    Ok(OpenSegment {
        report: prologue.report(SegmentKind::Play),
        segment,
        stepped: 0,
        current_sample: None,
        next_animation: 0,
        prepared: None,
        animations_finished: false,
        final_updaters_completed: false,
    })
}

/// Run one animation's interleaved `update_mobjects` + `interpolate` pair
/// and yield before the next animation.
///
/// Returning `Ok(None)` means either every animation for the current frame
/// has run (call [`prepare_play_frame`] to advance the clock) or the segment
/// has no frame left.
///
/// # Errors
/// A composition's deferred failure or an invalid caller phase.
pub fn prepare_play_animation(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    animations: &mut [Box<dyn Animation>],
    open: &mut OpenSegment,
) -> Result<Option<AnimationBoundary>, AnimError> {
    if open.animations_finished {
        return Err(AnimError::InvalidFramePhase(
            "cannot advance a play after animation cleanup",
        ));
    }
    if open.prepared.is_some() {
        return Err(AnimError::InvalidFramePhase(
            "complete the prepared frame before advancing another animation",
        ));
    }
    if let Some(error) = animations
        .iter()
        .find_map(|animation| animation.deferred_error())
    {
        return Err(error);
    }
    if open.current_sample.is_none() {
        if open.stepped >= open.segment.n_frames() {
            return Ok(None);
        }
        open.current_sample = open.segment.sample(open.stepped + 1);
        open.next_animation = 0;
    }
    if open.next_animation >= animations.len() {
        return Ok(None);
    }
    let sample = open.current_sample.ok_or(AnimError::InvalidFramePhase(
        "the open segment lost its current sample",
    ))?;
    let index = open.next_animation;
    let dt = clock.dt().to_f64();
    let animation = &mut animations[index];
    animation.update_mobjects(stage, dt);
    let alpha = sample.time.to_f64() / animation.state().config.run_time;
    animation.interpolate(stage, alpha);
    open.next_animation += 1;
    Ok(Some(AnimationBoundary {
        animation_index: index,
        alpha,
        dt,
        sample,
    }))
}

/// Prepare the next frame through steps 1–3, stopping immediately before
/// scene updaters.
///
/// This is the borrow-release half of Choreo's stepped front-door API. The
/// returned boundary must be paired with one [`complete_play_frame`] call
/// before another frame can be prepared.
///
/// # Errors
/// A composition's deferred failure, a clock failure, or an invalid caller
/// phase.
pub fn prepare_play_frame(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    animations: &mut [Box<dyn Animation>],
    open: &mut OpenSegment,
) -> Result<Option<SceneUpdaterBoundary>, AnimError> {
    if open.prepared.is_some() {
        return Err(AnimError::InvalidFramePhase(
            "complete the prepared frame before preparing another",
        ));
    }
    while prepare_play_animation(stage, clock, animations, open)?.is_some() {}
    let Some(sample) = open.current_sample else {
        return Ok(None);
    };
    let dt = clock.dt().to_f64();
    clock.advance_frames(1).map_err(AnimError::Clock)?;
    // Host-language updaters observe post-increment scene time, exactly as
    // the Reference's Scene.increment_time → update_mobjects order requires.
    stage.set_time_from_clock(clock.now().to_f64());
    open.prepared = Some(StepPlan {
        sample,
        dt,
        updaters_completed: false,
    });
    Ok(Some(SceneUpdaterBoundary {
        dt,
        time: clock.now(),
        sample,
    }))
}

/// Resume one prepared frame: finish step 4, run the serial capture
/// boundary, then freeze/emit steps 5–6 when `capture` is true.
///
/// # Errors
/// The caller did not first prepare a frame.
#[allow(clippy::too_many_arguments)]
pub fn complete_play_frame(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    rng: &RngRoot,
    open: &mut OpenSegment,
    capture: bool,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<(), AnimError> {
    let plan = open.prepared.take().ok_or(AnimError::InvalidFramePhase(
        "prepare a frame before completing it",
    ))?;
    if !plan.updaters_completed {
        stage.update_at_time(plan.dt, clock.now().to_f64());
    }
    boundary(stage, clock.now());
    if capture {
        emit(FramePacket::freeze(stage, clock, rng, &plan.sample));
    }
    open.stepped += 1;
    open.current_sample = None;
    open.next_animation = 0;
    Ok(())
}

/// Finish and consume an open play after all desired frames have completed.
///
/// This runs the same animation finish/remover/zero-dt updater epilogue as
/// [`play_segment`]. A prepared-but-uncompleted frame is refused so a front
/// end cannot silently capture a different state than it advanced.
///
/// # Errors
/// An invalid caller phase or a composition's deferred failure.
pub fn finish_open_play(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
    mut open: OpenSegment,
) -> Result<SegmentReport, AnimError> {
    finish_open_play_animations(stage, animations, &mut open)?;
    if !open.final_updaters_completed {
        update_scene_mobjects(stage, 0.0);
    }
    if let Some(error) = animations
        .iter()
        .find_map(|animation| animation.deferred_error())
    {
        return Err(error);
    }
    Ok(open.report)
}

/// Finish animation lifecycles without running the final scene-updater pass.
///
/// Host-language portals use this release point to resume animation-suspended
/// mobjects before dispatching their own zero-`dt` updaters.  They then call
/// [`finish_open_play`], which runs the native half of that same updater pass
/// and consumes the segment.  Ordinary Rust callers need only call
/// [`finish_open_play`], which performs both phases exactly once.
///
/// # Errors
/// A frame was prepared but not completed.
pub fn finish_open_play_animations(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
    open: &mut OpenSegment,
) -> Result<(), AnimError> {
    if open.prepared.is_some() || open.current_sample.is_some() {
        return Err(AnimError::InvalidFramePhase(
            "complete the prepared frame before finishing the segment",
        ));
    }
    if !open.animations_finished {
        for animation in animations.iter_mut() {
            animation.finish(stage);
            animation.clean_up_from_scene(stage);
        }
        open.animations_finished = true;
    }
    Ok(())
}

/// Abandon an open play without landing its endpoint.
///
/// This path is for host-language exceptions raised while a stepped frame is
/// between release boundaries. It consumes every open phase, releases each
/// animation's begin-time lifecycle state, and deliberately preserves the
/// partial scene state and clock position already observed by the host.
#[must_use]
pub fn abort_open_play(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
    open: OpenSegment,
) -> SegmentReport {
    for animation in animations {
        animation.abort(stage);
    }
    open.report
}

/// A declarative wait opened for borrow-released scene-updater stepping.
pub struct OpenWait {
    report: SegmentReport,
    segment: FrameSegment,
    stepped: i64,
    prepared: Option<StepPlan>,
}

impl OpenWait {
    /// Mark the prepared wait frame's whole scene-updater phase complete.
    ///
    /// # Errors
    /// No prepared frame exists, or this phase was already marked complete.
    pub fn mark_scene_updaters_complete(&mut self) -> Result<(), AnimError> {
        mark_updaters_completed(&mut self.prepared)?;
        note_external_updaters(&mut self.report);
        Ok(())
    }
}

/// Consume a wait abandoned at a host-language updater boundary.
#[must_use]
pub fn abort_open_wait(open: OpenWait) -> SegmentReport {
    open.report
}

impl fmt::Debug for OpenWait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenWait")
            .field("report", &self.report)
            .field("stepped", &self.stepped)
            .field("prepared", &self.prepared.is_some())
            .finish()
    }
}

/// Open a wait after the Reference's initial zero-dt native updater pass.
///
/// Host-language front ends run their own zero-dt updater pass immediately
/// before calling this function, while no Scene borrow is held.
///
/// # Errors
/// [`AnimError::Clock`] for an invalid duration.
pub fn open_wait(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    duration: f64,
    skip: bool,
) -> Result<OpenWait, AnimError> {
    update_scene_mobjects(stage, 0.0);
    plan_open_wait(stage, clock, duration, skip, false)
}

/// Open a wait after a host has completed BOTH halves of its initial zero-dt
/// scene-updater phase. This does not repeat native updater slots.
///
/// # Errors
/// [`AnimError::Clock`] for an invalid duration.
pub fn open_wait_after_updaters(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    duration: f64,
    skip: bool,
) -> Result<OpenWait, AnimError> {
    plan_open_wait(stage, clock, duration, skip, true)
}

fn plan_open_wait(
    stage: &Stage,
    clock: &RationalFrameClock,
    duration: f64,
    skip: bool,
    external_updaters: bool,
) -> Result<OpenWait, AnimError> {
    let segment = clock.segment(duration).map_err(AnimError::Clock)?;
    let purity = classify_wait(stage, false);
    let mut report = SegmentReport {
        kind: SegmentKind::Wait,
        purity: purity.clone(),
        begin_state: (purity.is_pure() && !skip).then(|| Rc::new(stage.snapshot())),
        base_frame: clock.now().frames(),
        n_frames: segment.n_frames(),
        run_time: duration,
    };
    if external_updaters {
        note_external_updaters(&mut report);
    }
    Ok(OpenWait {
        report,
        segment,
        stepped: 0,
        prepared: None,
    })
}

/// Advance a wait's clock and yield before its scene updater phase.
///
/// # Errors
/// A clock failure or invalid caller phase.
pub fn prepare_wait_frame(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    open: &mut OpenWait,
) -> Result<Option<SceneUpdaterBoundary>, AnimError> {
    if open.prepared.is_some() {
        return Err(AnimError::InvalidFramePhase(
            "complete the prepared wait frame before preparing another",
        ));
    }
    if open.stepped >= open.segment.n_frames() {
        return Ok(None);
    }
    let sample = open
        .segment
        .sample(open.stepped + 1)
        .ok_or(AnimError::InvalidFramePhase(
            "the open wait lost its next frame sample",
        ))?;
    let dt = clock.dt().to_f64();
    clock.advance_frames(1).map_err(AnimError::Clock)?;
    stage.set_time_from_clock(clock.now().to_f64());
    open.prepared = Some(StepPlan {
        sample,
        dt,
        updaters_completed: false,
    });
    Ok(Some(SceneUpdaterBoundary {
        dt,
        time: clock.now(),
        sample,
    }))
}

/// Complete a prepared wait frame through native scene updaters and capture.
///
/// # Errors
/// The caller did not first prepare a wait frame.
#[allow(clippy::too_many_arguments)]
pub fn complete_wait_frame(
    stage: &mut Stage,
    clock: &RationalFrameClock,
    rng: &RngRoot,
    open: &mut OpenWait,
    capture: bool,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<(), AnimError> {
    let plan = open.prepared.take().ok_or(AnimError::InvalidFramePhase(
        "prepare a wait frame before completing it",
    ))?;
    if !plan.updaters_completed {
        stage.update_at_time(plan.dt, clock.now().to_f64());
    }
    boundary(stage, clock.now());
    if capture {
        emit(FramePacket::freeze(stage, clock, rng, &plan.sample));
    }
    open.stepped += 1;
    Ok(())
}

/// Consume a completed or deliberately shortened open wait.
///
/// # Errors
/// A frame is still prepared but not completed.
pub fn finish_open_wait(mut open: OpenWait) -> Result<SegmentReport, AnimError> {
    if open.prepared.is_some() {
        return Err(AnimError::InvalidFramePhase(
            "complete the prepared wait frame before finishing the segment",
        ));
    }
    open.report.n_frames = open.stepped;
    Ok(open.report)
}

/// Step an open segment forward until `upto` of its frames have run,
/// emitting each. Idempotent past the end and a no-op when already there.
///
/// # Errors
/// A composition's deferred failure ([`Animation::deferred_error`]).
pub fn advance_play(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    animations: &mut [Box<dyn Animation>],
    open: &mut OpenSegment,
    upto: i64,
    emit: &mut dyn FnMut(FramePacket),
) -> Result<(), AnimError> {
    advance_play_with_boundary(
        stage,
        clock,
        rng,
        animations,
        open,
        upto,
        &mut |_, _| {},
        emit,
    )
}

/// As [`advance_play`], with one serial-front-end callback after scene
/// updaters and immediately before each immutable frame freeze.
///
/// The callback also runs for stepped frames whose eventual caller suppresses
/// capture. It must not retain the borrowed Stage; downstream capture consumes
/// only the packet frozen after it returns.
///
/// # Errors
/// As [`advance_play`].
#[allow(clippy::too_many_arguments)]
pub fn advance_play_with_boundary(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    animations: &mut [Box<dyn Animation>],
    open: &mut OpenSegment,
    upto: i64,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<(), AnimError> {
    advance_play_with_boundary_mode(
        stage, animations, clock, rng, open, upto, true, boundary, emit,
    )
}

#[allow(clippy::too_many_arguments)]
fn advance_play_with_boundary_mode(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    open: &mut OpenSegment,
    upto: i64,
    capture: bool,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<(), AnimError> {
    let target = upto.clamp(open.stepped, open.segment.n_frames());
    while open.stepped < target {
        prepare_play_frame(stage, clock, animations, open)?.ok_or(AnimError::InvalidFramePhase(
            "the segment ended before the requested target",
        ))?;
        complete_play_frame(stage, clock, rng, open, capture, boundary, emit)?;
    }
    match animations.iter().find_map(|a| a.deferred_error()) {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// A `wait()` segment run partway — the wait-side seek primitive, matching
/// the [`open_play`] / [`advance_play`] pair. No stop condition: a declarative
/// timeline has no callbacks to consult.
///
/// # Errors
/// [`AnimError::Clock`] for a non-finite or oversized duration.
pub fn wait_segment_upto(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    duration: f64,
    upto: i64,
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    update_scene_mobjects(stage, 0.0);
    let segment = clock.segment(duration).map_err(AnimError::Clock)?;
    let purity = classify_wait(stage, false);
    let base_frame = clock.now().frames();
    let begin_state = purity.is_pure().then(|| Rc::new(stage.snapshot()));
    let dt = clock.dt().to_f64();
    for sample in segment
        .samples()
        .take(usize::try_from(upto).unwrap_or(usize::MAX))
    {
        clock.advance_frames(1).map_err(AnimError::Clock)?;
        stage.update_at_time(dt, clock.now().to_f64());
        emit(FramePacket::freeze(stage, clock, rng, &sample));
    }
    Ok(SegmentReport {
        kind: SegmentKind::Wait,
        purity,
        begin_state,
        base_frame,
        n_frames: segment.n_frames(),
        run_time: duration,
    })
}

/// One `play()` segment under the six-step frame order: begin → automatic
/// purity classification (§9.5) → the per-sample steps over the §9.2
/// progression → finish. Emits one [`FramePacket`] per sample in frame order
/// (nothing under skip) and
/// returns the segment's [`SegmentReport`] — the journal record, carrying
/// the begin-state snapshot when the segment classified pure.
///
/// The play-level `run_time`/`rate_func`/`lag_ratio` overrides and the
/// pre/post-play scaffolding (writer, window) are the scene runtime's
/// (fm-5xm) — it calls `update_rate_info` before handing animations here.
/// Frame-parallel dispatch of pure segments is fmn-runtime's (fm-3df);
/// this driver always executes serially and the report is what makes the
/// parallel path legal.
///
/// # Errors
/// [`AnimError`] from `begin` (stale handles, hollow `time_span`) or a
/// non-finite/oversized run time ([`AnimError::Clock`]).
pub fn play_segment(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    animations: &mut [Box<dyn Animation>],
    skip: bool,
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    play_segment_with_boundary(stage, clock, rng, animations, skip, &mut |_, _| {}, emit)
}

/// As [`play_segment`], with a deterministic serial-front-end callback after
/// scene updaters and before every possible frame freeze.
///
/// Proscenium uses this one seam for input events. Keeping it inside Choreo's
/// driver proves that a listener can mutate neither an already-frozen packet
/// nor a capture in progress.
///
/// # Errors
/// As [`play_segment`].
pub fn play_segment_with_boundary(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    animations: &mut [Box<dyn Animation>],
    skip: bool,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    let prologue = play_prologue(stage, clock, animations, skip)?;
    let segment = prologue.segment;
    let report = prologue.report(SegmentKind::Play);
    // Once the prologue succeeds, the finish/cleanup phase runs EXACTLY once
    // in both modes, error or not (fm-tqy4): the segment body's earliest
    // driver/deferred error is captured — never `?`-propagated past
    // `finish_animations` — so a failing composition is still unmarked from
    // animating, removers apply, and the final zero-dt updater pass runs
    // before the same error surfaces that skip mode reports.
    let mut open = OpenSegment {
        report: report.clone(),
        segment,
        stepped: 0,
        current_sample: None,
        next_animation: 0,
        prepared: None,
        animations_finished: false,
        final_updaters_completed: false,
    };
    // Both modes use the exact same stepped driver. Skip changes one thing:
    // `complete_play_frame` suppresses capture/emission. In particular, a
    // composition's deferred error is observed before the same clock advance
    // in both modes rather than being delayed until skip reaches the endpoint.
    let body = advance_play_with_boundary_mode(
        stage,
        animations,
        clock,
        rng,
        &mut open,
        segment.n_frames(),
        !skip,
        boundary,
        emit,
    );
    finish_animations(stage, animations);
    body?;
    // A composition operator that begins members just in time has no error
    // channel inside `interpolate` (§9.4); the segment surfaces what it
    // recorded, by name, rather than leaving a frozen composition unexplained.
    if let Some(err) = animations.iter().find_map(|a| a.deferred_error()) {
        return Err(err);
    }
    Ok(report)
}

/// The Reference's `wait` / `wait_until`: an initial scene-updater pass at
/// `dt = 0` (no time advance), then per-frame steps 3–6. A stop condition
/// is checked **after** each frame's emit (the frame where it turns true
/// is emitted, then the wait ends) and forces per-frame stepping even
/// under skip (the Reference's `override_skip_animations`) — capture stays
/// off while skipping. Returns the segment's [`SegmentReport`]; a stop
/// condition demotes to stateful by vocabulary (its callback reads
/// per-frame state and decides the segment's length).
///
/// # Errors
/// [`AnimError::Clock`] for a non-finite or oversized duration.
pub fn wait_segment(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    duration: f64,
    stop_condition: Option<&mut dyn FnMut(&Stage) -> bool>,
    skip: bool,
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    wait_segment_with_boundary(
        stage,
        clock,
        rng,
        duration,
        stop_condition,
        skip,
        &mut |_, _| {},
        emit,
    )
}

/// As [`wait_segment`], with a deterministic serial-front-end callback after
/// scene updaters and before each possible frame freeze. The stop condition
/// observes the callback's mutation for that frame.
///
/// # Errors
/// As [`wait_segment`].
#[allow(clippy::too_many_arguments)]
pub fn wait_segment_with_boundary(
    stage: &mut Stage,
    clock: &mut RationalFrameClock,
    rng: &RngRoot,
    duration: f64,
    mut stop_condition: Option<&mut dyn FnMut(&Stage) -> bool>,
    skip: bool,
    boundary: &mut dyn FnMut(&mut Stage, RationalTime),
    emit: &mut dyn FnMut(FramePacket),
) -> Result<SegmentReport, AnimError> {
    update_scene_mobjects(stage, 0.0);
    let segment = clock.segment(duration).map_err(AnimError::Clock)?;
    let purity = classify_wait(stage, stop_condition.is_some());
    let base_frame = clock.now().frames();
    let begin_state = (purity.is_pure() && !skip).then(|| Rc::new(stage.snapshot()));
    let mut report = SegmentReport {
        kind: SegmentKind::Wait,
        purity,
        begin_state,
        base_frame,
        n_frames: segment.n_frames(),
        run_time: duration,
    };
    let dt = clock.dt().to_f64();
    let mut stepped = 0;
    for sample in segment.samples() {
        clock.advance_frames(1).map_err(AnimError::Clock)?;
        stage.update_at_time(dt, clock.now().to_f64());
        stepped += 1;
        boundary(stage, clock.now());
        if !skip {
            emit(FramePacket::freeze(stage, clock, rng, &sample));
        }
        if let Some(condition) = stop_condition.as_deref_mut()
            && condition(stage)
        {
            break;
        }
    }
    report.n_frames = stepped;
    Ok(report)
}
