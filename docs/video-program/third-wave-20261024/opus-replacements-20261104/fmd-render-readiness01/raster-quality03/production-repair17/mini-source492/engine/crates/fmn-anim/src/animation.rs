//! The Animation contract (§9.1, fm-67a): the Reference's constructor
//! surface field-for-field, the `begin → interpolate(alpha) → finish`
//! lifecycle, per-submobject lag via `get_sub_alpha`, and the typed
//! `prepare_animation` boundary.
//!
//! Mirrors `manimlib/animation/animation.py` at the pinned commit
//! (`3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13`):
//!
//! - [`AnimConfig`] is the §1.3 constructor surface — `(mobject,
//!   run_time=1.0, time_span, lag_ratio=0, rate_func=smooth, name,
//!   remover=False, final_alpha_value=1.0, suspend_mobject_updating=False)`
//!   — with the mobject handle carried by [`AnimState`].
//! - The [`Animation`] trait provides the lifecycle verbatim: `begin`
//!   (time-span run-time widening → `set_animating_status(true)` → the
//!   starting-mobject copy → optional updater suspension → the zipped
//!   family table → `interpolate(0)`), `interpolate` →
//!   `interpolate_mobject` → per-pair `interpolate_submobject`, and
//!   `finish` (`interpolate(final_alpha_value)` →
//!   `set_animating_status(false)` → conditional `resume_updating`).
//!   Subclass extension happens through the [`Animation::setup`] hook (the
//!   Reference's `super().begin()` idiom, inverted so the canonical
//!   sequence is written once and cannot be reordered).
//! - **The one normalized-alpha pipeline (§9.4)**: raw segment alpha →
//!   [`time_spanned_alpha`] (the `time_span` re-window) → the lag
//!   distribution → [`RateFunc`] evaluation, exactly the Reference's
//!   `interpolate_mobject`/`get_sub_alpha` composition, implemented once in
//!   [`sub_alpha`] so every mechanism family (fm-cye) and composition
//!   operator (fm-hfe) shares it. [`RateFunc`] models `squish_rate_func`
//!   and `not_quite_there` as *composable data*, so a squish-of-squish is
//!   representable, testable, and journalable.
//! - [`prepare_animation`] accepts an animation or an `.animate` recording
//!   — [`IntoAnimation`] is implemented for `Animation` types, for
//!   [`AnimBuilder`], and for [`BuiltAnimate`], and for nothing else. The
//!   Reference raises `TypeError` on a bare bound method; here a bare
//!   method is unrepresentable at the type level, which is the typed form
//!   of the same rejection (one recording mechanism serves both front
//!   doors, per fm-yra's `IntoAnimate` seam).
//! - [`MethodAnimation`] is the concrete carrier for built `.animate`
//!   chains (the Reference's `_MethodAnimation`), delegated to the same
//!   Transform mechanism as explicit transforms: heterogeneous family
//!   alignment, curved point paths, numeric uniforms, matching-data locks,
//!   and host-view materialization all have one implementation.
//!
//! Deliberate divergences (D5): the Reference's `update_rate_info` uses
//! Python `or`, so an explicit `run_time=0` or `lag_ratio=0` is silently
//! ignored — [`Animation::update_rate_info`] takes `Option`s and `Some(0.0)`
//! *sets* zero (the truthiness trap is unrepresentable). A `time_span`
//! whose `end <= start` raises `ZeroDivisionError`-or-NaN in the Reference;
//! here it is [`AnimError::InvalidTimeSpan`] at `begin`.

use fmn_core::rate;
use fmn_mobject::animate::{AnimateArgs, AnimateError, IntoAnimate};
use fmn_mobject::{AnimBuilder, BuiltAnimate, Mob, Stage, StageError};

/// The Reference's `DEFAULT_ANIMATION_RUN_TIME`.
pub const DEFAULT_ANIMATION_RUN_TIME: f64 = 1.0;
/// The Reference's `DEFAULT_ANIMATION_LAG_RATIO`.
pub const DEFAULT_ANIMATION_LAG_RATIO: f64 = 0.0;

/// Linear interpolation of one contiguous f32 record column.
///
/// Record precision is f32 API surface, but the interpolation itself is f64
/// (§6.1). Four independent values share one `f64x4`; there is no horizontal
/// reduction and therefore no lane-width-dependent association.
#[cfg(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma"
    ),
    all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
))]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn interpolate_linear_column(a: &[f32], b: &[f32], alpha: f64) -> Vec<f32> {
    use std::simd::f64x4;

    debug_assert_eq!(a.len(), b.len());
    let common = a.len().min(b.len());
    let (a_chunks, a_tail) = a[..common].as_chunks::<4>();
    let (b_chunks, b_tail) = b[..common].as_chunks::<4>();
    let mut out = Vec::with_capacity(common);
    let left = f64x4::splat(1.0 - alpha);
    let right = f64x4::splat(alpha);
    for (a, b) in a_chunks.iter().zip(b_chunks) {
        let a = f64x4::from_array([
            f64::from(a[0]),
            f64::from(a[1]),
            f64::from(a[2]),
            f64::from(a[3]),
        ]);
        let b = f64x4::from_array([
            f64::from(b[0]),
            f64::from(b[1]),
            f64::from(b[2]),
            f64::from(b[3]),
        ]);
        out.extend((left * a + right * b).to_array().map(|value| value as f32));
    }
    out.extend(
        a_tail
            .iter()
            .zip(b_tail)
            .map(|(&a, &b)| ((1.0 - alpha) * f64::from(a) + alpha * f64::from(b)) as f32),
    );
    out
}

/// Portable-build fallback for a kernel whose explicit SIMD route only pays
/// back its conversion cost at the governed ISA tiers.
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma"
    ),
    all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn interpolate_linear_column(a: &[f32], b: &[f32], alpha: f64) -> Vec<f32> {
    a.iter()
        .zip(b)
        .map(|(&a, &b)| ((1.0 - alpha) * f64::from(a) + alpha * f64::from(b)) as f32)
        .collect()
}

// ---------------------------------------------------------------- RateFunc

/// A rate function as composable data: the named catalog entries
/// ([`fmn_core::rate`]) as the base case, with the Reference's two
/// combinators — `squish_rate_func` and `not_quite_there` — as explicit
/// nodes. Composition (squish of squish, scaled squish, …) stays
/// inspectable, which is what lets the replay journal and the API schema
/// name the rate curve of any animation.
///
/// Capturing closures (the Python front door's lambdas) join this type with
/// fmn-python's bridge work; every non-capturing `fn(f64) -> f64` — the
/// entire native catalog — fits [`RateFunc::Base`] today.
#[derive(Clone, Debug)]
pub enum RateFunc {
    /// A plain rate function (any `fn(f64) -> f64`, e.g. the fmn-core
    /// catalog).
    Base(fn(f64) -> f64),
    /// A pre-sampled rate curve on a uniform grid over `[0, 1]`, linearly
    /// interpolated between samples (at least two).
    ///
    /// This is the capturing-closure seam the type docs promise the Python
    /// front door: the portal evaluates a pure Python rate callable for
    /// every grid point BEFORE a segment runs, so the segment drivers stay
    /// free of mid-segment interpreter crossings. Sampled at a segment's
    /// frame count, the curve is exact at every unlagged capture boundary;
    /// lagged or time-spanned members evaluate between grid points and see
    /// the linear interpolant of the pure callable.
    Sampled(std::sync::Arc<[f64]>),
    /// The Reference's `squish_rate_func(func, a, b)`: run `inner`'s whole
    /// arc inside `[a, b]`, clamping outside; degenerate `a == b` returns
    /// `a` (kept exactly).
    Squish {
        /// The wrapped rate function.
        inner: Box<RateFunc>,
        /// Window start.
        a: f64,
        /// Window end.
        b: f64,
    },
    /// The Reference's `not_quite_there(func, proportion)`: scale `inner`
    /// to top out at `proportion`.
    Scaled {
        /// The wrapped rate function.
        inner: Box<RateFunc>,
        /// The output scale factor.
        proportion: f64,
    },
}

impl Default for RateFunc {
    /// The Reference's default: `smooth`.
    fn default() -> Self {
        Self::Base(rate::smooth)
    }
}

impl RateFunc {
    /// `smooth` — the constructor-surface default.
    #[must_use]
    pub fn smooth() -> Self {
        Self::Base(rate::smooth)
    }

    /// `linear`.
    #[must_use]
    pub fn linear() -> Self {
        Self::Base(rate::linear)
    }

    /// Wrap in `squish_rate_func(self, a, b)`.
    #[must_use]
    pub fn squish(self, a: f64, b: f64) -> Self {
        Self::Squish {
            inner: Box::new(self),
            a,
            b,
        }
    }

    /// Wrap in `not_quite_there(self, proportion)`.
    #[must_use]
    pub fn not_quite_there(self, proportion: f64) -> Self {
        Self::Scaled {
            inner: Box::new(self),
            proportion,
        }
    }

    /// Evaluate at `t`, with the Reference's exact combinator semantics.
    #[must_use]
    pub fn eval(&self, t: f64) -> f64 {
        match self {
            Self::Base(f) => f(t),
            Self::Sampled(samples) => match samples.len() {
                0 => t,
                1 => samples[0],
                len => {
                    #[allow(clippy::cast_precision_loss)]
                    let position = t.clamp(0.0, 1.0) * (len - 1) as f64;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let low = position.floor() as usize;
                    let high = low.min(len - 2) + 1;
                    #[allow(clippy::cast_precision_loss)]
                    let fraction = position - low as f64;
                    (1.0 - fraction) * samples[low.min(len - 1)] + fraction * samples[high]
                }
            },
            Self::Squish { inner, a, b } => {
                // The Reference's squish_rate_func body, branch for branch.
                if a == b {
                    *a
                } else if t < *a {
                    inner.eval(0.0)
                } else if t > *b {
                    inner.eval(1.0)
                } else {
                    inner.eval((t - a) / (b - a))
                }
            }
            Self::Scaled { inner, proportion } => proportion * inner.eval(t),
        }
    }
}

// --------------------------------------------- the normalized-alpha pipeline

/// The Reference's `clip` (`utils/simple_functions.py`): branch-for-branch,
/// with no panic on a hollow interval (unlike `f64::clamp`) — the pipeline
/// must stay total.
pub(crate) fn clip(value: f64, lower: f64, upper: f64) -> f64 {
    if value < lower {
        lower
    } else if value > upper {
        upper
    } else {
        value
    }
}

/// The `time_span` re-window — the Reference's `time_spanned_alpha`,
/// formula for formula: with a span `(start, end)` inside a group interval,
/// `clip(alpha·run_time − start, 0, end − start) / (end − start)`; without
/// one, the identity.
///
/// `run_time` is the *widened* run time (`begin` raises it to at least
/// `end`). Callers validate `end > start` ([`AnimError::InvalidTimeSpan`])
/// before entering the pipeline.
#[must_use]
pub fn time_spanned_alpha(alpha: f64, run_time: f64, time_span: Option<(f64, f64)>) -> f64 {
    match time_span {
        Some((start, end)) => clip(alpha * run_time - start, 0.0, end - start) / (end - start),
        None => alpha,
    }
}

/// The lag distribution + rate evaluation — the Reference's
/// `get_sub_alpha`, formula for formula:
///
/// ```text
/// full_length = (num_submobjects − 1) · lag_ratio + 1
/// value       = alpha · full_length
/// lower       = index · lag_ratio
/// sub_alpha   = rate_func(clip(value − lower, 0, 1))
/// ```
///
/// `lag_ratio = 0` applies to all submobjects simultaneously; `1` strictly
/// successively; between, with lagged starts. This is the ONE place the
/// rate function enters the pipeline (§9.4) — segment alphas arrive raw.
#[must_use]
pub fn sub_alpha(
    alpha: f64,
    index: usize,
    num_submobjects: usize,
    lag_ratio: f64,
    rate_func: &RateFunc,
) -> f64 {
    let full_length = (num_submobjects as f64 - 1.0) * lag_ratio + 1.0;
    let value = alpha * full_length;
    let lower = index as f64 * lag_ratio;
    rate_func.eval(clip(value - lower, 0.0, 1.0))
}

// ------------------------------------------------------------------ errors

/// Errors from the animation contract.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimError {
    /// A mobject handle (the animated mobject, or a builder reference) is
    /// dead at begin/prepare time.
    StaleHandle(Mob),
    /// `time_span` with `end <= start` — the Reference divides by zero or
    /// produces NaN; we refuse it by name at `begin`.
    InvalidTimeSpan {
        /// The requested span start.
        start: f64,
        /// The requested span end.
        end: f64,
    },
    /// Legacy alignment refusal retained for source compatibility.
    /// Method animations now use Transform's heterogeneous alignment.
    UnalignedFamilies,
    /// Legacy arc-path refusal retained for source compatibility.
    /// Method animations now support Transform's native arc paths.
    PathArcUnsupported,
    /// The `.animate` build failed (stale handle, chaining-rule violation).
    Builder(AnimateError),
    /// The segment's run time was refused by the clock (non-finite, or
    /// beyond the frame counter's range).
    Clock(crate::clock::ClockError),
    /// Frame reconstruction was asked of a stateful (or skipped) segment —
    /// only pure segments carry a begin-state snapshot to reconstruct from
    /// (§9.5).
    SegmentNotPure,
    /// `MoveToTarget` on a mobject that never ran `generate_target` (the
    /// Reference's "MoveToTarget called on mobject without target").
    MissingTarget,
    /// `Restore` on a mobject that never ran `save_state`.
    MissingSavedState,
    /// An animation that needs a point to anchor on (`GrowArrow`'s
    /// `get_start`, `MoveAlongPath`'s path) got a pointless mobject — the
    /// Reference raises `IndexError`; this is the typed refusal.
    EmptyMobject,
    /// The Stage refused an alignment or copy operation (malformed point
    /// run, mixed schemas).
    Stage(StageError),
    /// A composition operator (§9.4) was handed no members. The Reference
    /// indexes `args[0]` and raises `IndexError`; `Succession.begin`
    /// asserts. An empty composition has no timeline to derive, so it is
    /// refused by name at construction.
    EmptyComposition,
    /// `AddTextWordByWord` was handed a grouped parent with no span-map
    /// word groups (fm-5wq.4.54) — an empty string mobject has no words
    /// to add, and revealing nothing must be named, never a silent no-op.
    NoWordGroups,
    /// `AddTextLetterByLetter` was handed a grouped parent with no
    /// span-map letter groups (fm-5wq.4.59) — a string mobject with no
    /// glyphs has no letters to add; named, never a silent no-op.
    NoLetterGroups,
    /// A `Timeline` seek named a frame outside the compiled schedule
    /// (§9.4). Frames are 1-based and on-grid — there is no off-grid seek,
    /// for the same reason there is no off-grid sample (D-18).
    SeekOutOfRange {
        /// The requested frame.
        frame: i64,
        /// Frames the schedule actually covers.
        total: i64,
    },
    /// A resumable frame was prepared twice or completed without its
    /// matching preparation. This names a front-end driver bug rather than
    /// allowing the clock and capture stream to drift apart.
    InvalidFramePhase(&'static str),
    /// A non-finite point-path parameter, rejected before Stage mutation.
    InvalidPath(&'static str),
    /// A succession's time moved backward through stateful interpolation or
    /// updater-owned history. Those effects cannot be undone by restoring
    /// mobject content; use Timeline seek's checkpoint-and-replay route.
    StatefulSuccessionRewind,
}

impl std::fmt::Display for AnimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleHandle(_) => write!(f, "animation target handle is stale"),
            Self::InvalidTimeSpan { start, end } => {
                write!(f, "time_span ({start}, {end}) must satisfy end > start")
            }
            Self::UnalignedFamilies => write!(
                f,
                "source and target families diverged since build; \
                 heterogeneous alignment (align_data) lands with the Transform family (fm-cye)"
            ),
            Self::PathArcUnsupported => write!(
                f,
                "path_arc is the Transform family's path_func mechanism (fm-cye); \
                 not yet available on built .animate chains"
            ),
            Self::Builder(e) => write!(f, "animate build failed: {e}"),
            Self::Clock(e) => write!(f, "segment refused by the clock: {e}"),
            Self::SegmentNotPure => write!(
                f,
                "frame reconstruction requires a pure, unskipped segment's begin state"
            ),
            Self::MissingTarget => write!(
                f,
                "MoveToTarget called on mobject without a generated target"
            ),
            Self::MissingSavedState => {
                write!(f, "Restore called on mobject without a saved state")
            }
            Self::EmptyMobject => {
                write!(f, "animation requires a mobject with points")
            }
            Self::Stage(e) => write!(f, "stage refused the operation: {e}"),
            Self::EmptyComposition => {
                write!(f, "a composition needs at least one animation")
            }
            Self::NoWordGroups => write!(
                f,
                "AddTextWordByWord requires span-map word groups; the string mobject produced none"
            ),
            Self::NoLetterGroups => write!(
                f,
                "AddTextLetterByLetter requires span-map letter groups; \
                 the string mobject produced none"
            ),
            Self::SeekOutOfRange { frame, total } => {
                write!(
                    f,
                    "seek to frame {frame} outside the schedule's 1..={total}"
                )
            }
            Self::InvalidPath(message) => write!(f, "invalid animation path: {message}"),
            Self::StatefulSuccessionRewind => write!(
                f,
                "Succession cannot reverse stateful or updater-driven members; \
                 use Timeline seek to replay from a checkpoint"
            ),
            Self::InvalidFramePhase(message) => {
                write!(f, "invalid resumable frame phase: {message}")
            }
        }
    }
}

impl std::error::Error for AnimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Builder(e) => Some(e),
            Self::Clock(e) => Some(e),
            _ => None,
        }
    }
}

impl From<StageError> for AnimError {
    fn from(e: StageError) -> Self {
        Self::Stage(e)
    }
}

impl From<AnimateError> for AnimError {
    fn from(e: AnimateError) -> Self {
        Self::Builder(e)
    }
}

// ----------------------------------------------------------------- config

/// The §1.3 constructor surface, field for field (minus the mobject, which
/// [`AnimState`] carries as the arena handle).
#[derive(Clone, Debug)]
pub struct AnimConfig {
    /// Seconds the animation runs (`begin` widens this to at least
    /// `time_span.1`). Reference default `1.0`.
    pub run_time: f64,
    /// `(start, end)` — the window inside a group interval the animation
    /// actually runs in ([`time_spanned_alpha`]).
    pub time_span: Option<(f64, f64)>,
    /// `0`: all submobjects simultaneously; `1`: strictly successively;
    /// between: lagged starts. Reference default `0`.
    pub lag_ratio: f64,
    /// The rate curve. Reference default `smooth`.
    pub rate_func: RateFunc,
    /// Diagnostic name (the Reference derives `ClassName + str(mobject)`
    /// when empty).
    pub name: String,
    /// Whether `finish`ing this animation removes the mobject from the
    /// scene (`clean_up_from_scene` — the scene runtime consumes
    /// [`Animation::is_remover`] at fm-5xm).
    pub remover: bool,
    /// The alpha `finish` leaves the mobject at. Reference default `1.0`.
    pub final_alpha_value: f64,
    /// Suspend the mobject's own updaters for the play (start/target
    /// copies keep updating — the Reference's documented asymmetry).
    pub suspend_mobject_updating: bool,
}

impl Default for AnimConfig {
    fn default() -> Self {
        Self {
            run_time: DEFAULT_ANIMATION_RUN_TIME,
            time_span: None,
            lag_ratio: DEFAULT_ANIMATION_LAG_RATIO,
            rate_func: RateFunc::default(),
            name: String::new(),
            remover: false,
            final_alpha_value: 1.0,
            suspend_mobject_updating: false,
        }
    }
}

impl AnimConfig {
    /// Map a built `.animate` chain's [`AnimateArgs`] onto the constructor
    /// surface (unset fields take the Reference defaults). `path_arc` is
    /// not part of this surface — [`MethodAnimation::new`] passes it to
    /// the shared Transform path mechanism, together with its axis.
    #[must_use]
    pub fn from_animate_args(args: &AnimateArgs) -> Self {
        Self {
            run_time: args.run_time.unwrap_or(DEFAULT_ANIMATION_RUN_TIME),
            time_span: args.time_span,
            lag_ratio: args.lag_ratio.unwrap_or(DEFAULT_ANIMATION_LAG_RATIO),
            rate_func: args
                .rate_func
                .map_or_else(RateFunc::default, RateFunc::Base),
            ..Self::default()
        }
    }
}

// ------------------------------------------------------------------ state

/// The per-play runtime state every animation carries: the animated
/// mobject, the config, and what `begin` establishes (the starting copy,
/// the zipped family table, the updater-suspension memo).
#[derive(Debug, Clone)]
pub struct AnimState {
    mobject: Mob,
    /// The constructor surface (public: the Reference exposes these as
    /// plain attributes).
    pub config: AnimConfig,
    starting_mobject: Option<Mob>,
    families: Vec<Vec<Mob>>,
    mobject_was_updating: bool,
}

impl AnimState {
    /// A fresh state for `mobject` under `config`.
    #[must_use]
    pub fn new(mobject: Mob, config: AnimConfig) -> Self {
        Self {
            mobject,
            config,
            starting_mobject: None,
            families: Vec::new(),
            mobject_was_updating: false,
        }
    }

    /// The animated mobject.
    #[must_use]
    pub fn mobject(&self) -> Mob {
        self.mobject
    }

    /// The begin-time copy (`None` before `begin`).
    #[must_use]
    pub fn starting_mobject(&self) -> Option<Mob> {
        self.starting_mobject
    }

    /// The zipped family table `begin` established (rows follow
    /// [`Animation::all_mobjects`] order; truncated to the shortest family,
    /// as Python `zip` does).
    #[must_use]
    pub fn families(&self) -> &[Vec<Mob>] {
        &self.families
    }

    /// Whether `begin` suspended an updater-active mobject that `finish`
    /// must resume.
    #[must_use]
    pub fn mobject_was_updating(&self) -> bool {
        self.mobject_was_updating
    }
}

// -------------------------------------------------------------- signature

/// An animation's declared effect signature — §9.5's purity vocabulary
/// (shared with the §13.4 effect model). Membership in `Pure` is a closed
/// allowlist: only animations whose every interpolation provably writes
/// state as a function of the begin snapshot and alpha alone declare it.
/// The default is [`AnimationSignature::Unclassified`], and the
/// conservative rule (R20) demotes unclassified animations' segments to
/// stateful — misclassification would be a correctness bug, so unknown
/// always means impure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationSignature {
    /// Provably pure interpolation (allowlist member).
    Pure,
    /// No declaration — conservatively treated as effectful.
    Unclassified,
}

// ------------------------------------------------------------------ trait

/// The Animation lifecycle contract (§9.1). The canonical sequences are
/// written once as provided methods; concrete animations implement
/// [`Animation::interpolate_submobject`] (the Reference's "typically
/// implemented by subclass") and extend `begin` through the
/// [`Animation::setup`] hook rather than overriding the order.
pub trait Animation {
    /// The runtime state (mobject handle, config, begin-time products).
    fn state(&self) -> &AnimState;

    /// Mutable runtime state.
    fn state_mut(&mut self) -> &mut AnimState;

    /// Append mobjects whose Python/host updater layer must receive the
    /// `resume_updating(call_updater=True)` zero-`dt` callback after this
    /// animation's lifecycle finishes.
    fn collect_resumed_updater_mobjects(&self, out: &mut Vec<Mob>) {
        let state = self.state();
        if state.config.suspend_mobject_updating && state.mobject_was_updating() {
            let mobject = state.mobject();
            if !out.contains(&mobject) {
                out.push(mobject);
            }
        }
    }

    /// Subclass extension slot, run at the top of [`Animation::begin`]
    /// *before* the canonical sequence — where the Transform family will
    /// generate/align its target (fm-cye) and where [`MethodAnimation`]
    /// verifies pair alignment. Default: nothing.
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let _ = stage;
        Ok(())
    }

    /// The Reference's `create_starting_mobject`: a deep family copy of the
    /// animated mobject (updater callables shared by reference, per §8.3).
    fn create_starting_mobject(&self, stage: &mut Stage) -> Result<Mob, AnimError> {
        let mobject = self.state().mobject();
        stage
            .copy_family(mobject)
            .map_err(|_| AnimError::StaleHandle(mobject))
    }

    /// The Reference's `get_all_mobjects`: ordering **must** match the
    /// argument order [`Animation::interpolate_submobject`] receives its
    /// family row in. Default: `(mobject, starting_mobject)`.
    fn all_mobjects(&self) -> Vec<Mob> {
        let state = self.state();
        let mut mobs = vec![state.mobject()];
        if let Some(starting) = state.starting_mobject() {
            mobs.push(starting);
        }
        mobs
    }

    /// Every authored mobject the animation may read, copy, or publish.
    ///
    /// Unlike [`Animation::all_mobjects`], this inventory is not a zipped
    /// interpolation row and has no positional contract. It deliberately
    /// excludes begin-time products such as `starting_mobject`: timeline seek
    /// restores an earlier stage while reusing animation values, so products
    /// from the abandoned future are expected to be stale and will be rebuilt
    /// by the next `begin()`. Proscenium calls this before `begin()` so Scribe
    /// can preflight otherwise-unrooted static content. Animations with
    /// authored auxiliary handles (for example a Transform target or a path)
    /// must extend the default inventory.
    fn preflight_mobjects(&self) -> Vec<Mob> {
        vec![self.state().mobject()]
    }

    /// Refuse begin-time inputs that can be checked without mutating the
    /// animation or the stage.
    ///
    /// Choreo runs this before the play lifecycle becomes externally
    /// observable. The default covers the canonical animation state: every
    /// preflight handle must still be live, and a declared time span must be
    /// non-hollow. Composition operators extend the check recursively over
    /// their members. Concrete animations may override this when their
    /// [`Animation::setup`] hook has additional read-only refusals; an error
    /// that can only arise while setup mutates is still contained by the
    /// driver's stage transaction.
    ///
    /// # Errors
    /// [`AnimError::StaleHandle`] for a dead preflight handle, or
    /// [`AnimError::InvalidTimeSpan`] for `end <= start`.
    fn validate_begin(&self, stage: &Stage) -> Result<(), AnimError> {
        validate_begin_state(self.state(), stage, self.preflight_mobjects())
    }

    /// Interpolate one zipped family row (`mobs` follows
    /// [`Animation::all_mobjects`] order) at pipeline-final `sub_alpha`.
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64);

    /// The Reference's `begin`, sequence for sequence: called right as the
    /// play starts; as much initialization as possible lives here.
    ///
    /// # Errors
    /// [`AnimError::InvalidTimeSpan`], [`AnimError::StaleHandle`], and
    /// whatever [`Animation::setup`] reports.
    fn begin(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        self.setup(stage)?;
        // 1. time_span widens run_time to cover its end (the Reference
        //    mutates run_time here, and get_run_time repeats the max).
        if let Some((start, end)) = self.state().config.time_span {
            if end <= start {
                return Err(AnimError::InvalidTimeSpan { start, end });
            }
            let state = self.state_mut();
            state.config.run_time = state.config.run_time.max(end);
        }
        // 2. Mark the family (and its ancestors) animating.
        let mobject = self.state().mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        stage.set_animating_status(mobject, true, true);
        // 3. The starting copy.
        let starting = self.create_starting_mobject(stage)?;
        self.state_mut().starting_mobject = Some(starting);
        // 4. Optional updater suspension (remembering whether the mobject
        //    was updating, so finish only resumes what begin paused).
        if self.state().config.suspend_mobject_updating {
            let was_updating = !stage.is_updating_suspended(mobject);
            self.state_mut().mobject_was_updating = was_updating;
            stage.suspend_updating(mobject, true);
        }
        // 5. The zipped family table (Python zip: truncate to shortest).
        let all = self.all_mobjects();
        let families: Vec<Vec<Mob>> = all.iter().map(|&m| stage.family(m)).collect();
        let rows = families.iter().map(Vec::len).min().unwrap_or(0);
        self.state_mut().families = (0..rows)
            .map(|i| families.iter().map(|f| f[i]).collect())
            .collect();
        // 6. The zero interpolation — separate from emission (§9.2: no
        //    alpha-zero frame is ever emitted; this is where alpha zero
        //    happens instead).
        self.interpolate(stage, 0.0);
        // 7. Post-begin subclass hook — the Reference's Transform locks
        //    matching data *after* `super().begin()` (transform.py:54);
        //    this slot keeps that order with the canonical sequence
        //    written once.
        self.after_begin(stage);
        Ok(())
    }

    /// Subclass slot run at the end of [`Animation::begin`], after the
    /// zero interpolation (the Reference's post-`super().begin()` code —
    /// Transform's `lock_matching_data` call site). Default: nothing.
    fn after_begin(&mut self, stage: &mut Stage) {
        let _ = stage;
    }

    /// The Reference's `finish`: land on `final_alpha_value`, clear the
    /// animating status, resume updating if `begin` paused it. Scene
    /// removal for removers is the runtime's (`clean_up_from_scene`,
    /// fm-5xm) via [`Animation::is_remover`].
    fn finish(&mut self, stage: &mut Stage) {
        let final_alpha = self.state().config.final_alpha_value;
        self.interpolate(stage, final_alpha);
        let mobject = self.state().mobject();
        stage.set_animating_status(mobject, false, true);
        if self.state().config.suspend_mobject_updating && self.state().mobject_was_updating {
            // Reference resume_updating defaults: recurse and one
            // call_updater pass (which runs update(0) exactly once — C-5).
            stage.resume_updating(mobject, true, true);
        }
        // Subclass slot — the Reference's Transform unlocks data in its
        // `finish` override (transform.py:74).
        self.teardown(stage);
    }

    /// Abandon a begun animation after a host callback fails.
    ///
    /// Unlike [`Self::finish`], abort never interpolates the final endpoint,
    /// runs remover cleanup, or calls a resumed updater. It only releases the
    /// lifecycle state that `begin` acquired so the Scene is not left with a
    /// permanently animating or suspended family. The already-observed
    /// partial frame state and rational time remain intact, matching Python
    /// exception propagation rather than pretending the segment completed.
    fn abort(&mut self, stage: &mut Stage) {
        let mobject = self.state().mobject();
        if !stage.contains(mobject) {
            return;
        }
        stage.set_animating_status(mobject, false, true);
        if self.state().config.suspend_mobject_updating && self.state().mobject_was_updating {
            stage.resume_updating(mobject, true, false);
        }
        self.teardown(stage);
    }

    /// Subclass slot run at the end of [`Animation::finish`] (the
    /// Reference's `finish` overrides — Transform's `unlock_data` call
    /// site). Default: nothing.
    fn teardown(&mut self, stage: &mut Stage) {
        let _ = stage;
    }

    /// The mean of an animation: the Reference's `interpolate` (alias of
    /// `interpolate_mobject`). `alpha` is the raw segment alpha — the
    /// pipeline (`time_span` → lag → rate) applies inside, once.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        let state = self.state();
        let families = state.families.clone();
        let run_time = state.config.run_time;
        let time_span = state.config.time_span;
        let lag_ratio = state.config.lag_ratio;
        let rate_func = state.config.rate_func.clone();
        let num = families.len();
        let spanned = time_spanned_alpha(alpha, run_time, time_span);
        for (index, mobs) in families.iter().enumerate() {
            let sa = sub_alpha(spanned, index, num, lag_ratio, &rate_func);
            self.interpolate_submobject(stage, mobs, sa);
        }
    }

    /// The Reference's `update_mobjects(dt)`: tick the updaters of every
    /// animation-owned mobject (starting/target copies) — the scene handles
    /// `self.mobject` itself. Step one of the six-step frame order
    /// (fm-x79).
    fn update_mobjects(&mut self, stage: &mut Stage, dt: f64) {
        for mob in self.mobjects_to_update() {
            stage.update_mobject(mob, dt);
        }
    }

    /// The Reference's `get_all_mobjects_to_update`: everything in
    /// [`Animation::all_mobjects`] except the animated mobject itself,
    /// deduplicated, insertion order kept.
    fn mobjects_to_update(&self) -> Vec<Mob> {
        let mobject = self.state().mobject();
        let mut out: Vec<Mob> = Vec::new();
        for mob in self.all_mobjects() {
            if mob != mobject && !out.contains(&mob) {
                out.push(mob);
            }
        }
        out
    }

    /// The Reference's `get_run_time`: widened by `time_span` without
    /// mutating (begin performs the same max in place).
    fn get_run_time(&self) -> f64 {
        let config = &self.state().config;
        match config.time_span {
            Some((_, end)) => config.run_time.max(end),
            None => config.run_time,
        }
    }

    /// The Reference's `update_rate_info`: batch-update timing fields,
    /// `None` keeping the current value.
    ///
    /// Deliberate divergence (D5, module doc): the Reference's `or`-based
    /// merge silently ignores explicit zeros; here `Some(0.0)` sets zero.
    fn update_rate_info(
        &mut self,
        run_time: Option<f64>,
        rate_func: Option<RateFunc>,
        lag_ratio: Option<f64>,
    ) {
        let config = &mut self.state_mut().config;
        if let Some(rt) = run_time {
            config.run_time = rt;
        }
        if let Some(rf) = rate_func {
            config.rate_func = rf;
        }
        if let Some(lr) = lag_ratio {
            config.lag_ratio = lr;
        }
    }

    /// Whether finishing removes the mobject from the scene.
    fn is_remover(&self) -> bool {
        self.state().config.remover
    }

    /// The Reference's `clean_up_from_scene`: a remover leaves the scene
    /// once it has finished. Composition operators (§9.4) override this to
    /// delegate — a `FadeOut` inside a group is still a remover, and the
    /// container it was composed into is not.
    fn clean_up_from_scene(&mut self, stage: &mut Stage) {
        if self.is_remover() {
            stage.remove_from_scene(self.state().mobject());
        }
    }

    /// A failure recorded during `interpolate`, which has no error channel.
    /// Only the composition operators that begin members just in time
    /// (§9.4's `Succession`) can produce one; the drivers surface it at the
    /// end of the segment so a mid-play failure is named rather than
    /// silently frozen. `None` for every leaf animation.
    fn deferred_error(&self) -> Option<AnimError> {
        None
    }

    /// The declared effect signature (§9.5). Default: unclassified —
    /// which the segment-purity classifier conservatively demotes (R20).
    /// Override to [`AnimationSignature::Pure`] only for animations whose
    /// interpolation is provably a function of (begin snapshot, alpha).
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Unclassified
    }

    /// Whether interpolation can be sampled again after restoring this
    /// animation's prepared output family, while its existing begin-time
    /// copies remain live. The caller must also rule out updater-owned state.
    ///
    /// Pure leaves satisfy this narrower local contract. A just-in-time
    /// composition may support it through its own initialized-member cache
    /// while still requiring serial frame reconstruction: it is not enough
    /// to declare the whole segment [`AnimationSignature::Pure`]. No setup,
    /// teardown, or other authored lifecycle callback is replayed by this
    /// capability.
    fn can_replay_interpolation(&self) -> bool {
        matches!(self.effect_signature(), AnimationSignature::Pure)
    }

    /// Notify a locally replayable animation that its parent restored its
    /// prepared output family. Leaves keep their existing begin-time inputs;
    /// just-in-time compositions must restore their own initialized members
    /// before the next sample. This never repeats a lifecycle callback and
    /// does not make a stateful segment eligible for pure-frame reconstruction.
    fn prepare_interpolation_replay(&mut self) {}
}

/// The canonical, non-mutating begin checks shared by leaf animations and
/// composition containers.
pub(crate) fn validate_begin_state(
    state: &AnimState,
    stage: &Stage,
    mobjects: impl IntoIterator<Item = Mob>,
) -> Result<(), AnimError> {
    for mobject in mobjects {
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
    }
    if let Some((start, end)) = state.config.time_span
        && end <= start
    {
        return Err(AnimError::InvalidTimeSpan { start, end });
    }
    Ok(())
}

// ------------------------------------------------------- MethodAnimation

#[path = "method_animation.rs"]
mod method_animation;
pub use method_animation::MethodAnimation;

// ----------------------------------------------------- prepare_animation

/// The typed `prepare_animation` contract (§9.1): an animation, or an
/// `.animate` recording — nothing else. The Reference's runtime
/// `TypeError` on bare bound methods is a compile error here: no other
/// type implements this trait.
pub trait IntoAnimation {
    /// Realize into a boxed [`Animation`].
    ///
    /// # Errors
    /// [`AnimError`] from the `.animate` build or carrier construction.
    fn into_animation(self, stage: &mut Stage) -> Result<Box<dyn Animation>, AnimError>;
}

impl<A: Animation + 'static> IntoAnimation for A {
    fn into_animation(self, stage: &mut Stage) -> Result<Box<dyn Animation>, AnimError> {
        let _ = stage;
        Ok(Box::new(self))
    }
}

impl IntoAnimation for AnimBuilder {
    fn into_animation(self, stage: &mut Stage) -> Result<Box<dyn Animation>, AnimError> {
        let built = self.prepare(stage)?;
        Ok(Box::new(MethodAnimation::new(built)?))
    }
}

impl IntoAnimation for BuiltAnimate {
    fn into_animation(self, stage: &mut Stage) -> Result<Box<dyn Animation>, AnimError> {
        let built = self.prepare(stage)?; // revalidates handles, as IntoAnimate does
        Ok(Box::new(MethodAnimation::new(built)?))
    }
}

/// The Reference's `prepare_animation` entry point, shared by both front
/// doors: builders build (dynamic target lookup happens NOW, at play
/// time), animations pass through.
///
/// # Errors
/// [`AnimError`] from the build or carrier construction.
pub fn prepare_animation(
    input: impl IntoAnimation,
    stage: &mut Stage,
) -> Result<Box<dyn Animation>, AnimError> {
    input.into_animation(stage)
}

/// Anything accepted by the high-level `play` surface: one animation-like
/// value or a pair that should start in the same segment.
///
/// The pair implementation prepares both values into the same vector passed
/// to Choreo's existing segment driver. It is therefore simultaneous play,
/// not an [`AnimationGroup`](crate::composition::AnimationGroup) wrapper and
/// not a second scheduling mechanism.
pub trait IntoAnimations {
    /// Realize the input into one non-empty play list.
    ///
    /// # Errors
    /// [`AnimError`] from any animation or `.animate` recording.
    fn into_animations(self, stage: &mut Stage) -> Result<Vec<Box<dyn Animation>>, AnimError>;
}

impl<A: Animation + 'static> IntoAnimations for A {
    fn into_animations(self, _stage: &mut Stage) -> Result<Vec<Box<dyn Animation>>, AnimError> {
        Ok(vec![Box::new(self)])
    }
}

impl IntoAnimations for AnimBuilder {
    fn into_animations(self, stage: &mut Stage) -> Result<Vec<Box<dyn Animation>>, AnimError> {
        prepare_animation(self, stage).map(|animation| vec![animation])
    }
}

impl IntoAnimations for BuiltAnimate {
    fn into_animations(self, stage: &mut Stage) -> Result<Vec<Box<dyn Animation>>, AnimError> {
        prepare_animation(self, stage).map(|animation| vec![animation])
    }
}

impl<A, B> IntoAnimations for (A, B)
where
    A: IntoAnimation,
    B: IntoAnimation,
{
    fn into_animations(self, stage: &mut Stage) -> Result<Vec<Box<dyn Animation>>, AnimError> {
        let first = prepare_animation(self.0, stage)?;
        let second = prepare_animation(self.1, stage)?;
        Ok(vec![first, second])
    }
}

/// Prepare one high-level play input without changing segment semantics.
///
/// # Errors
/// [`AnimError`] from any animation or `.animate` recording.
pub fn prepare_animations(
    input: impl IntoAnimations,
    stage: &mut Stage,
) -> Result<Vec<Box<dyn Animation>>, AnimError> {
    input.into_animations(stage)
}

#[cfg(test)]
mod simd_tests {
    use super::interpolate_linear_column;

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn linear_field_simd_is_the_scalar_f64_definition_lane_for_lane() {
        let a = [
            -0.0,
            0.0,
            1.0,
            -1.0,
            f32::MIN_POSITIVE,
            -f32::MIN_POSITIVE,
            f32::EPSILON,
            12_345.5,
            -98_765.25,
            f32::MAX,
            -f32::MAX,
        ];
        let b = [
            9.0,
            -7.0,
            -3.0,
            5.0,
            f32::EPSILON,
            -f32::EPSILON,
            f32::MIN_POSITIVE,
            -54_321.25,
            87_654.5,
            -f32::MAX,
            f32::MAX,
        ];
        for alpha in [-0.25, 0.0, 0.314_159_265_358_979_3, 1.0, 1.5] {
            let simd = interpolate_linear_column(&a, &b, alpha);
            let scalar: Vec<f32> = a
                .iter()
                .zip(b)
                .map(|(&a, b)| ((1.0 - alpha) * f64::from(a) + alpha * f64::from(b)) as f32)
                .collect();
            assert_eq!(
                simd.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
                scalar
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                "alpha={alpha}"
            );
        }
    }
}
