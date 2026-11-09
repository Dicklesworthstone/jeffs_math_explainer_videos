//! The segment-purity classifier (§9.5, D-19, R20, fm-3xk): automatic,
//! conservative, journaled.
//!
//! Every `play()`/`wait()` segment is classified by its **effect
//! signature**. A segment is [`Purity::Pure`] when each of its frames is a
//! function of the begin-state CoW snapshot and its alpha alone: no
//! dt-updaters, no scene updaters (which subsumes `always_redraw`/
//! `f_always` — both bind as updaters), no stateful tracers (they arrive
//! with W7 as updater-bound classes and demote through the same probe),
//! no unclassified animations, no wait stop-conditions. Pure segments are
//! embarrassingly parallel across frames: worker *k* reconstructs frame
//! state from the snapshot plus α(k) and the keyed per-frame RNG fork
//! (§6.5), which [`reconstruct_pure_frame`] implements and the
//! equivalence tests hold to bit-identity against the serial path. The
//! actual multi-worker dispatch (render teams, the ordered emitter's
//! thread plumbing) is fmn-runtime's (fm-3df); the classifier, the
//! journal vocabulary, and the reconstruction contract live here.
//!
//! **The conservative rule (R20), load-bearing:** misclassification would
//! be a correctness bug, so the effect vocabulary is a closed allowlist
//! of provably-pure constructs and everything unrecognized demotes the
//! segment to stateful — unknown updater shapes, animations without a
//! declared [`AnimationSignature::Pure`], callbacks of any kind. When a
//! misclassification is ever found in the wild, its effect class is
//! demoted engine-wide until root-caused (R20's kill rule — enforced
//! operationally through the replay journal, fm-y7u, which records the
//! [`SegmentReport`] emitted here per segment).

use std::rc::Rc;

use fmn_core::rng::RngRoot;
use fmn_mobject::{Mob, Snapshot, Stage};

use crate::animation::{AnimError, Animation, AnimationSignature};
use crate::clock::{FrameSample, RationalFrameClock};
use crate::frame::FramePacket;

/// One reason a segment is stateful — the closed impurity vocabulary
/// (shared with the §13.4 effect model; the replay journal records these).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpureEffect {
    /// A dt-updater in a rooted or animation-owned family: frame state
    /// depends on accumulated time steps.
    DtUpdater,
    /// A non-dt updater in a rooted or animation-owned family (includes
    /// every `always_redraw`/`f_always` binding): frame state depends on
    /// per-frame re-execution order.
    SceneUpdater,
    /// An animation without a declared pure signature — the conservative
    /// default for custom `interpolate` implementations.
    UnclassifiedAnimation,
    /// A `wait_until`-style stop condition: an arbitrary callback reads
    /// per-frame state and decides the segment's length dynamically.
    StopCondition,
}

/// A segment's classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purity {
    /// Every frame is a function of (begin snapshot, alpha, keyed RNG
    /// fork) — eligible for frame-parallel rendering.
    Pure,
    /// At least one recorded effect requires serial front-end execution
    /// (the §17.4 pipeline still overlaps its back-end stages).
    Stateful(Vec<ImpureEffect>),
}

impl Purity {
    /// Whether the segment classified pure.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        matches!(self, Self::Pure)
    }
}

/// What kind of segment a report describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// A `play()` segment.
    Play,
    /// A `wait()`/`wait_until()` segment.
    Wait,
}

/// The per-segment record the drivers emit — §9.5's journal entry (the
/// replay journal, fm-y7u, persists these; the frame pipeline, fm-3df,
/// dispatches on them).
#[derive(Clone)]
pub struct SegmentReport {
    /// Play or wait.
    pub kind: SegmentKind,
    /// The classification, with its recorded reasons when stateful.
    pub purity: Purity,
    /// The begin-state CoW snapshot — the §9.5 worker input. `Some` only
    /// for pure, non-skipped segments.
    pub begin_state: Option<Rc<Snapshot>>,
    /// The clock's frame counter at segment start (frame *k* of the
    /// segment is global frame `base_frame + k`).
    pub base_frame: i64,
    /// Frames the segment covers on the grid.
    pub n_frames: i64,
    /// The segment's run time in seconds (the widened maximum for plays).
    pub run_time: f64,
}

impl std::fmt::Debug for SegmentReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegmentReport")
            .field("kind", &self.kind)
            .field("purity", &self.purity)
            .field("has_begin_state", &self.begin_state.is_some())
            .field("base_frame", &self.base_frame)
            .field("n_frames", &self.n_frames)
            .field("run_time", &self.run_time)
            .finish()
    }
}

/// Push an effect at most once (presence, not multiplicity, is recorded).
fn note(effects: &mut Vec<ImpureEffect>, effect: ImpureEffect) {
    if !effects.contains(&effect) {
        effects.push(effect);
    }
}

/// Scan one family for updaters, demoting per kind.
fn scan_family(stage: &Stage, mob: Mob, effects: &mut Vec<ImpureEffect>) {
    let (non_dt, dt) = stage.family_updater_kinds(mob);
    if dt > 0 {
        note(effects, ImpureEffect::DtUpdater);
    }
    if non_dt > 0 {
        note(effects, ImpureEffect::SceneUpdater);
    }
}

/// Classify a `play()` segment (call after `begin` — the starting/target
/// copies the animations will tick must exist to be scanned).
#[must_use]
pub fn classify_play(stage: &Stage, animations: &[Box<dyn Animation>]) -> Purity {
    let mut effects = Vec::new();
    for animation in animations {
        if animation.effect_signature() != AnimationSignature::Pure {
            note(&mut effects, ImpureEffect::UnclassifiedAnimation);
        }
        // Animation-owned mobjects (starting/target) tick every frame
        // through update_mobjects even when unrooted.
        for mob in animation.all_mobjects() {
            scan_family(stage, mob, &mut effects);
        }
    }
    for root in stage.roots().to_vec() {
        scan_family(stage, root, &mut effects);
    }
    if effects.is_empty() {
        Purity::Pure
    } else {
        Purity::Stateful(effects)
    }
}

/// Classify a `wait()` segment.
#[must_use]
pub fn classify_wait(stage: &Stage, has_stop_condition: bool) -> Purity {
    let mut effects = Vec::new();
    if has_stop_condition {
        note(&mut effects, ImpureEffect::StopCondition);
    }
    for root in stage.roots().to_vec() {
        scan_family(stage, root, &mut effects);
    }
    if effects.is_empty() {
        Purity::Pure
    } else {
        Purity::Stateful(effects)
    }
}

/// Reconstruct one frame of a pure segment from its begin-state snapshot —
/// the §9.5 worker body: restore, interpolate at α(k), freeze. A pure
/// function of `(snapshot, sample, seed)`: callable for any frame in any
/// order, producing the bit-identical [`FramePacket`] the serial path
/// emitted for that frame (the equivalence tests hold it to `f32::to_bits`
/// identity).
///
/// # Errors
/// [`AnimError::SegmentNotPure`] when the report is stateful or carries no
/// begin state (skipped segments emit nothing to reconstruct).
pub fn reconstruct_pure_frame(
    stage: &mut Stage,
    animations: &mut [Box<dyn Animation>],
    report: &SegmentReport,
    rng: &RngRoot,
    sample: &FrameSample,
) -> Result<FramePacket, AnimError> {
    let Some(begin_state) = report
        .begin_state
        .as_ref()
        .filter(|_| report.purity.is_pure())
    else {
        return Err(AnimError::SegmentNotPure);
    };
    stage.restore(begin_state);
    // Steps 1–2 for this frame alone: update_mobjects is a no-op by
    // construction (pure ⇒ no updaters anywhere), and interpolation is
    // absolute in alpha — no per-frame accumulation exists to replay.
    for animation in animations.iter_mut() {
        let alpha = sample.time.to_f64() / animation.state().config.run_time;
        animation.interpolate(stage, alpha);
    }
    let mut clock =
        RationalFrameClock::new(sample.time.fps()).expect("sample fps is a live clock's");
    clock
        .advance_frames(report.base_frame)
        .and_then(|()| clock.advance_frames(sample.frame))
        .map_err(AnimError::Clock)?;
    stage.set_time_from_clock(clock.now().to_f64());
    Ok(FramePacket::freeze(stage, &clock, rng, sample))
}
