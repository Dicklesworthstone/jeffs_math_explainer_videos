//! The `.animate` builder: deferred-command recording with the Reference's
//! REAL rules (§8.6, fm-yra; G0-1's ratified fluent-recording shape). This is
//! the native front door's builder; the Python portal records its own chains
//! and hands Choreo the resulting target.
//!
//! The recordable surface is the Reference's chainable mutation vocabulary
//! that a target copy can realize: placement (`shift`, `move_to`, `next_to`,
//! `align_to`, `to_edge`, `to_corner`, `center`, `set_x/y/z`, `match_*`,
//! `arrange`), transforms (`scale`, `stretch`, `rotate` about a point or axis,
//! `flip`, `apply_matrix`), size (`set_width`, `set_height`), style
//! (`set_color`, `set_fill`, `set_stroke`, `set_opacity`, `fade`), and value
//! trackers. Arbitrary callables (`apply_function`, `become`) are not
//! recordable as `Copy` commands; they are ordinary animations instead.
//!
//! `mob.animate()` returns a recorder; chained mutation calls append
//! commands; [`AnimBuilder::build`] realizes the recording against a target
//! copy generated **at build time** — the deferred-command model D-11
//! ratified (the borrow-checker-clean shape: the recorder owns only a
//! `Copy` handle). [`PosTarget::Mob`](crate::positional::PosTarget)
//! references inside commands also resolve at build time, so a `move_to`
//! recorded against a moving mobject uses its position when the play
//! happens, not when the chain was written.
//!
//! The Reference's builder rules, kept exactly (`_AnimationBuilder`):
//!
//! - **Animation arguments are set once per chain.** `animate(**kwargs)` /
//!   `set_anim_args` may be called only once; a second call raises. Here:
//!   [`AnimBuilder::set_anim_args`] errors with
//!   [`AnimateError::ArgsAlreadySet`].
//! - **Overridden animations don't chain.** A method carrying
//!   `_override_animate` may be neither preceded nor followed by another
//!   chained call. Here: [`AnimBuilder::with_override`] errors with
//!   [`AnimateError::OverrideNotChainable`] if anything was chained before
//!   it, and every recording method errors the same way after it. (The
//!   library's overridden methods arrive with W7; the rule and its
//!   enforcement live here.)
//! - **`prepare_animation` accepts an `Animation` or a builder, nothing
//!   else.** The Reference rejects bare bound methods with a `TypeError`;
//!   in Rust that contract is the [`IntoAnimate`] trait — implemented for
//!   [`AnimBuilder`] and [`BuiltAnimate`] only, so a "bare method" is
//!   unrepresentable at the type level. fmn-anim's `Animation` joins the
//!   same contract at fm-67a (§9.1 shares it).

use crate::placement::Placement;
use crate::positional::PosTarget;
use crate::stage::{Mob, Stage};
use fmn_core::color::Srgb;
use fmn_core::constants::{OUT, TAU};
use fmn_core::types::Vec3;
use fmn_geom::Mat3;

/// Transform-level arguments passed once per chain (the Reference's
/// `set_anim_args` surface: `run_time`, `rate_func`, `lag_ratio`,
/// `path_arc`, `time_span`). Consumed by Choreo when the built animation is
/// played (fm-67a).
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimateArgs {
    /// Seconds the transform runs; engine default when `None`.
    pub run_time: Option<f64>,
    /// The rate function (an fmn-core `rate` fn); engine default when `None`.
    pub rate_func: Option<fn(f64) -> f64>,
    /// Per-submobject stagger.
    pub lag_ratio: Option<f64>,
    /// Arc angle for the transform path.
    pub path_arc: Option<f64>,
    /// Axis for the arc path; defaults to OUT. A zero vector also means OUT.
    pub path_arc_axis: Option<Vec3>,
    /// `(start, end)` within the play window.
    pub time_span: Option<(f64, f64)>,
}

impl PartialEq for AnimateArgs {
    fn eq(&self, other: &Self) -> bool {
        // Manual impl: rate functions compare by address (`fn_addr_eq`, the
        // sanctioned form), everything else by value.
        self.run_time == other.run_time
            && self.lag_ratio == other.lag_ratio
            && self.path_arc == other.path_arc
            && self.path_arc_axis == other.path_arc_axis
            && self.time_span == other.time_span
            && match (self.rate_func, other.rate_func) {
                (None, None) => true,
                (Some(a), Some(b)) => std::ptr::fn_addr_eq(a, b),
                _ => false,
            }
    }
}

/// A recorded mutation, applied to the target copy at build.
/// Every variant maps 1:1 onto an existing Stage mutation surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimateCommand {
    /// `shift(vector)`.
    Shift(Vec3),
    /// `scale(factor)` about the bounding-box center.
    Scale(f64),
    /// `rotate(angle)` about the bounding-box center in the xy-plane.
    Rotate(f64),
    /// `set_opacity(opacity)` across the whole family.
    SetOpacity(f64),
    /// `stretch(factor, dim)`.
    Stretch(f64, usize),
    /// `center()`.
    Center,
    /// `move_to(target, aligned_edge)` — a Mob target resolves at build.
    MoveTo(PosTarget, Vec3),
    /// `next_to(target, direction, buff, aligned_edge)`.
    NextTo(PosTarget, Vec3, f64, Vec3),
    /// `align_to(target, direction)`.
    AlignTo(PosTarget, Vec3),
    /// `to_edge(edge, buff)`.
    ToEdge(Vec3, f64),
    /// `to_corner(corner, buff)`.
    ToCorner(Vec3, f64),
    /// `set_width(width, stretch)`.
    SetWidth(f64, bool),
    /// `set_height(height, stretch)`.
    SetHeight(f64, bool),
    /// Set a scalar tracker (plain or exponential) through its encoding.
    SetValue(f64),
    /// Increment a scalar tracker's decoded value at build time.
    IncrementValue(f64),
    /// Set both lanes of a complex tracker.
    SetComplexValue(f64, f64),
    /// Increment a complex tracker's real and imaginary components.
    IncrementComplexValue(f64, f64),
    /// `rotate(angle, axis, about_point)`; `None` pivots on the box center.
    RotateAbout(f64, Vec3, Option<Vec3>),
    /// `scale(factor, about_point, about_edge)`; both `None` is the center.
    ScaleAbout(f64, Option<Vec3>, Option<Vec3>),
    /// `flip(axis)`: a half turn about `axis` through the box center.
    Flip(Vec3),
    /// `apply_matrix(matrix)` about the origin, the Reference's default.
    ApplyMatrix(Mat3),
    /// `set_x` / `set_y` / `set_z` of the box center: `(dim, value)`.
    SetCoord(usize, f64),
    /// `match_x` / `match_y` / `match_z`: `(other, dim)`, read at build.
    MatchCoord(Mob, usize),
    /// `match_width` / `match_height` / `match_depth`: `(other, dim)`.
    MatchDimSize(Mob, usize),
    /// `arrange(direction, buff, center)` over the target's submobjects.
    Arrange(Vec3, f64, bool),
    /// `set_color(color, opacity)` across the family: fill and stroke (and
    /// a base mobject's `rgba`) take the colour; opacity only when given.
    SetColor(Srgb, Option<f64>),
    /// `set_fill(color, opacity)` across the family.
    SetFill(Option<Srgb>, Option<f64>),
    /// `set_stroke(color, width, opacity)` across the family.
    SetStroke(Option<Srgb>, Option<f64>, Option<f64>),
    /// `fade(darkness)`: the Reference's `set_opacity(1 - darkness)`.
    Fade(f64),
}

/// Marker for a W7 `override_animate` animation: the builder enforces the
/// no-chaining rule now; the actual overridden animations arrive with the
/// library classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverrideAnimation {
    /// The overriding animation's name (diagnostic identity until W7 binds
    /// real animation constructors here).
    pub name: &'static str,
}

/// A builder failure — each is one of the Reference's raise sites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimateError {
    /// `set_anim_args` called twice on one chain.
    ArgsAlreadySet,
    /// An overridden animation was mixed with chained calls.
    OverrideNotChainable,
    /// A handle (the source, or a command's Mob target) is dead at build.
    StaleHandle(Mob),
    /// A live source lacks the tracker encoding required by a command.
    TrackerKindMismatch {
        /// The live, but incompatible, source.
        source: Mob,
        /// The required decoded value type (scalar or complex).
        expected: &'static str,
    },
}

impl std::fmt::Display for AnimateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgsAlreadySet => write!(
                f,
                "animation arguments can only be passed by calling animate()/set_anim_args \
                 and can only be passed once"
            ),
            Self::OverrideNotChainable => write!(
                f,
                "method chaining is not supported for overridden animations"
            ),
            Self::StaleHandle(_) => write!(f, "animate target handle is stale"),
            Self::TrackerKindMismatch { expected, .. } => {
                write!(f, "animate command requires a {expected} value tracker")
            }
        }
    }
}

impl std::error::Error for AnimateError {}

/// The deferred-command recorder returned by [`Mob::animate`].
#[derive(Clone, Debug, PartialEq)]
pub struct AnimBuilder {
    source: Mob,
    commands: Vec<AnimateCommand>,
    args: Option<AnimateArgs>,
    overridden: Option<OverrideAnimation>,
    is_chaining: bool,
}

/// A realized recording: the source, the fully-positioned target copy, and
/// the animation arguments — what Choreo interpolates (fm-67a).
#[derive(Clone, Debug, PartialEq)]
pub struct BuiltAnimate {
    /// The animated mobject.
    pub source: Mob,
    /// The target copy the play interpolates toward (not in the scene).
    pub target: Mob,
    /// Transform-level arguments (defaults where unset).
    pub args: AnimateArgs,
    /// The overriding animation, if the chain recorded one.
    pub overridden: Option<OverrideAnimation>,
}

impl Mob {
    /// Start recording a deferred animation against this handle.
    #[must_use]
    pub fn animate(self) -> AnimBuilder {
        AnimBuilder {
            source: self,
            commands: Vec::new(),
            args: None,
            overridden: None,
            is_chaining: false,
        }
    }
}

impl AnimBuilder {
    /// The animated mobject.
    #[must_use]
    pub fn source(&self) -> Mob {
        self.source
    }

    /// The recorded commands, in order.
    #[must_use]
    pub fn commands(&self) -> &[AnimateCommand] {
        &self.commands
    }

    /// Pass the transform arguments — once per chain, as in the Reference.
    ///
    /// # Errors
    /// [`AnimateError::ArgsAlreadySet`] on a second call.
    pub fn set_anim_args(mut self, args: AnimateArgs) -> Result<Self, AnimateError> {
        if self.args.is_some() {
            return Err(AnimateError::ArgsAlreadySet);
        }
        self.args = Some(args);
        Ok(self)
    }

    /// Record an overridden animation. Neither preceded nor followed by
    /// chained calls, as in the Reference.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`].
    pub fn with_override(mut self, animation: OverrideAnimation) -> Result<Self, AnimateError> {
        if self.is_chaining || self.overridden.is_some() {
            return Err(AnimateError::OverrideNotChainable);
        }
        self.overridden = Some(animation);
        self.is_chaining = true;
        Ok(self)
    }

    fn push(mut self, command: AnimateCommand) -> Result<Self, AnimateError> {
        if self.overridden.is_some() {
            return Err(AnimateError::OverrideNotChainable);
        }
        self.is_chaining = true;
        self.commands.push(command);
        Ok(self)
    }

    /// Record `shift`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn shift(self, vector: Vec3) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Shift(vector))
    }

    /// Record `scale`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn scale(self, factor: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Scale(factor))
    }

    /// Record `rotate` about the target's bounding-box center in the xy-plane.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn rotate(self, angle: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Rotate(angle))
    }

    /// Record `set_opacity` across the target family.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_opacity(self, opacity: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetOpacity(opacity))
    }

    /// Record `stretch`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn stretch(self, factor: f64, dim: usize) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Stretch(factor, dim))
    }

    /// Record `center`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn center(self) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Center)
    }

    /// Record `move_to`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn move_to(
        self,
        target: impl Into<PosTarget>,
        aligned_edge: Vec3,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MoveTo(target.into(), aligned_edge))
    }

    /// Record `next_to`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn next_to(
        self,
        target: impl Into<PosTarget>,
        direction: Vec3,
        buff: f64,
        aligned_edge: Vec3,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::NextTo(
            target.into(),
            direction,
            buff,
            aligned_edge,
        ))
    }

    /// Record `align_to`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn align_to(
        self,
        target: impl Into<PosTarget>,
        direction: Vec3,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::AlignTo(target.into(), direction))
    }

    /// Record `to_edge`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn to_edge(self, edge: Vec3, buff: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::ToEdge(edge, buff))
    }

    /// Record `to_corner`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn to_corner(self, corner: Vec3, buff: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::ToCorner(corner, buff))
    }

    /// Record `set_width`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_width(self, width: f64, stretch: bool) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetWidth(width, stretch))
    }

    /// Record `set_height`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_height(self, height: f64, stretch: bool) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetHeight(height, stretch))
    }

    /// Record `set_value` for a plain or exponential ValueTracker.
    /// The source is untouched; the target receives the decoded value at build.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override. The tracker
    /// kind is validated when the recording is built, before Stage mutation.
    pub fn set_value(self, value: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetValue(value))
    }

    /// Record a scalar increment, resolved against the target's decoded value
    /// at build time rather than the source's value when this call is recorded.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn increment_value(self, delta: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::IncrementValue(delta))
    }

    /// Record `set_value` for a complex tracker, as real and imaginary lanes.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_complex_value(self, real: f64, imaginary: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetComplexValue(real, imaginary))
    }

    /// Record an increment of a complex tracker's decoded components.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn increment_complex_value(self, real: f64, imaginary: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::IncrementComplexValue(real, imaginary))
    }

    /// Record `rotate(angle, axis, about_point)`. `about_point = None`
    /// rotates about the target's bounding-box center, as the Reference does.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn rotate_about(
        self,
        angle: f64,
        axis: Vec3,
        about_point: Option<Vec3>,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::RotateAbout(angle, axis, about_point))
    }

    /// Record `scale(factor, about_point=…, about_edge=…)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn scale_about(
        self,
        factor: f64,
        about_point: Option<Vec3>,
        about_edge: Option<Vec3>,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::ScaleAbout(factor, about_point, about_edge))
    }

    /// Record `flip(axis)` — the Reference's `rotate(TAU / 2, axis)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn flip(self, axis: Vec3) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Flip(axis))
    }

    /// Record `apply_matrix(matrix)` about the origin.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn apply_matrix(self, matrix: Mat3) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::ApplyMatrix(matrix))
    }

    /// Record `set_x`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_x(self, x: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetCoord(0, x))
    }

    /// Record `set_y`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_y(self, y: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetCoord(1, y))
    }

    /// Record `set_z`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_z(self, z: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetCoord(2, z))
    }

    /// Record `match_x(other)`; `other`'s position is read at build time.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_x(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchCoord(other, 0))
    }

    /// Record `match_y(other)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_y(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchCoord(other, 1))
    }

    /// Record `match_z(other)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_z(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchCoord(other, 2))
    }

    /// Record `match_width(other)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_width(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchDimSize(other, 0))
    }

    /// Record `match_height(other)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_height(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchDimSize(other, 1))
    }

    /// Record `match_depth(other)`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn match_depth(self, other: Mob) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::MatchDimSize(other, 2))
    }

    /// Record `arrange(direction, buff)`, re-centering the arranged group as
    /// the Reference's default `center=True` does.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn arrange(self, direction: Vec3, buff: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Arrange(direction, buff, true))
    }

    /// Record `set_color(color)` across the family; opacity is unchanged.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_color(self, color: Srgb) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetColor(color, None))
    }

    /// Record `set_color(color, opacity)` across the family.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_color_with_opacity(self, color: Srgb, opacity: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetColor(color, Some(opacity)))
    }

    /// Record `set_fill(color, opacity)`; `None` leaves that lane untouched.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_fill(self, color: Option<Srgb>, opacity: Option<f64>) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetFill(color, opacity))
    }

    /// Record `set_stroke(color, width, opacity)`; `None` leaves that aspect
    /// untouched.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn set_stroke(
        self,
        color: Option<Srgb>,
        width: Option<f64>,
        opacity: Option<f64>,
    ) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::SetStroke(color, width, opacity))
    }

    /// Record `fade(darkness)`: opacity becomes `1 - darkness`.
    ///
    /// # Errors
    /// [`AnimateError::OverrideNotChainable`] after an override.
    pub fn fade(self, darkness: f64) -> Result<Self, AnimateError> {
        self.push(AnimateCommand::Fade(darkness))
    }

    /// Realize the recording: generate the target copy NOW (dynamic target
    /// lookup at build time), apply every command to it, and hand back the
    /// [`BuiltAnimate`] Choreo interpolates. The target is arena-allocated
    /// but not in the scene.
    ///
    /// # Errors
    /// [`AnimateError::StaleHandle`] for a dead source or a dead Mob target
    /// inside a command; [`AnimateError::TrackerKindMismatch`] for a scalar
    /// command on a non-scalar source or a complex command on a non-complex one.
    pub fn build(self, stage: &mut Stage) -> Result<BuiltAnimate, AnimateError> {
        // Validate every Mob reference before mutating anything.
        if !stage.contains(self.source) {
            return Err(AnimateError::StaleHandle(self.source));
        }
        for command in &self.commands {
            let expected = match command {
                AnimateCommand::SetValue(_) | AnimateCommand::IncrementValue(_)
                    if stage.tracker_value(self.source).is_none() =>
                {
                    Some("scalar")
                }
                AnimateCommand::SetComplexValue(_, _)
                | AnimateCommand::IncrementComplexValue(_, _)
                    if stage.tracker_complex_value(self.source).is_none() =>
                {
                    Some("complex")
                }
                _ => None,
            };
            if let Some(expected) = expected {
                return Err(AnimateError::TrackerKindMismatch {
                    source: self.source,
                    expected,
                });
            }
            if let AnimateCommand::MoveTo(PosTarget::Mob(m), _)
            | AnimateCommand::NextTo(PosTarget::Mob(m), _, _, _)
            | AnimateCommand::AlignTo(PosTarget::Mob(m), _)
            | AnimateCommand::MatchCoord(m, _)
            | AnimateCommand::MatchDimSize(m, _) = command
                && !stage.contains(*m)
            {
                return Err(AnimateError::StaleHandle(*m));
            }
        }
        let target = stage
            .copy_family(self.source)
            .map_err(|_| AnimateError::StaleHandle(self.source))?;
        for command in &self.commands {
            apply(stage, target, *command)?;
        }
        Ok(BuiltAnimate {
            source: self.source,
            target,
            args: self.args.unwrap_or_default(),
            overridden: self.overridden,
        })
    }
}

fn apply(stage: &mut Stage, target: Mob, command: AnimateCommand) -> Result<(), AnimateError> {
    match command {
        AnimateCommand::Shift(v) => {
            stage.shift(target, v);
        }
        AnimateCommand::Scale(f) => {
            stage.scale(target, f);
        }
        AnimateCommand::Rotate(angle) => {
            stage.rotate(target, angle, OUT, None, None);
        }
        AnimateCommand::SetOpacity(opacity) => {
            stage.set_family_opacity(target, opacity);
        }
        AnimateCommand::Stretch(f, dim) => {
            stage.stretch(target, f, dim);
        }
        AnimateCommand::Center => {
            stage.center(target);
        }
        AnimateCommand::MoveTo(t, aligned_edge) => {
            stage.move_to(target, t, aligned_edge);
        }
        AnimateCommand::NextTo(t, direction, buff, aligned_edge) => {
            stage.next_to(target, t, direction, buff, aligned_edge);
        }
        AnimateCommand::AlignTo(t, direction) => {
            stage.align_to(target, t, direction);
        }
        AnimateCommand::ToEdge(edge, buff) => {
            stage.to_edge(target, edge, buff);
        }
        AnimateCommand::ToCorner(corner, buff) => {
            stage.to_corner(target, corner, buff);
        }
        AnimateCommand::SetWidth(w, s) => {
            stage.set_width(target, w, s);
        }
        AnimateCommand::SetHeight(h, s) => {
            stage.set_height(target, h, s);
        }
        AnimateCommand::SetValue(value) => {
            stage
                .set_tracker_value(target, value)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::IncrementValue(delta) => {
            stage
                .increment_tracker_value(target, delta)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::SetComplexValue(real, imaginary) => {
            stage
                .set_tracker_complex_value(target, real, imaginary)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::IncrementComplexValue(real, imaginary) => {
            stage
                .increment_tracker_complex_value(target, real, imaginary)
                .map_err(|_| AnimateError::StaleHandle(target))?;
        }
        AnimateCommand::RotateAbout(angle, axis, about_point) => {
            stage.rotate(target, angle, axis, about_point, None);
        }
        AnimateCommand::ScaleAbout(factor, about_point, about_edge) => {
            stage.scale_about(target, factor, about_point, about_edge);
        }
        AnimateCommand::Flip(axis) => {
            stage.rotate(target, TAU / 2.0, axis, None, None);
        }
        AnimateCommand::ApplyMatrix(matrix) => {
            stage.apply_affine(target, Placement::new(matrix, [0.0; 3]));
        }
        AnimateCommand::SetCoord(dim, value) => {
            stage.set_coord(target, value, dim, fmn_core::constants::ORIGIN);
        }
        AnimateCommand::MatchCoord(other, dim) => {
            stage.match_coord(target, other, dim, fmn_core::constants::ORIGIN);
        }
        AnimateCommand::MatchDimSize(other, dim) => {
            stage.match_dim_size(target, other, dim);
        }
        AnimateCommand::Arrange(direction, buff, center) => {
            stage.arrange(target, direction, buff, center);
        }
        AnimateCommand::SetColor(color, opacity) => {
            let rgb = Some([color.r, color.g, color.b]);
            for field in ["fill_rgba", "stroke_rgba", "rgba"] {
                stage.set_color_lanes(target, field, rgb, opacity, true);
            }
        }
        AnimateCommand::SetFill(color, opacity) => {
            let rgb = color.map(|c| [c.r, c.g, c.b]);
            stage.set_color_lanes(target, "fill_rgba", rgb, opacity, true);
        }
        AnimateCommand::SetStroke(color, width, opacity) => {
            let rgb = color.map(|c| [c.r, c.g, c.b]);
            stage.set_color_lanes(target, "stroke_rgba", rgb, opacity, true);
            if let Some(width) = width {
                stage.set_scalar_lanes(target, "stroke_width", width, true);
            }
        }
        AnimateCommand::Fade(darkness) => {
            stage.set_family_opacity(target, 1.0 - darkness);
        }
    }
    Ok(())
}

/// The `prepare_animation` contract (§9.1, shared with fmn-anim): a play
/// call accepts an animation or a builder — nothing else. Implemented for
/// [`AnimBuilder`] (which builds) and [`BuiltAnimate`] (identity); a bare
/// method reference is unrepresentable, which is the typed form of the
/// Reference's `TypeError`.
pub trait IntoAnimate {
    /// Convert into a realized animation input.
    ///
    /// # Errors
    /// [`AnimateError`] from the build.
    fn prepare(self, stage: &mut Stage) -> Result<BuiltAnimate, AnimateError>;
}

impl IntoAnimate for AnimBuilder {
    fn prepare(self, stage: &mut Stage) -> Result<BuiltAnimate, AnimateError> {
        self.build(stage)
    }
}

impl IntoAnimate for BuiltAnimate {
    fn prepare(self, stage: &mut Stage) -> Result<BuiltAnimate, AnimateError> {
        if stage.contains(self.source) && stage.contains(self.target) {
            Ok(self)
        } else {
            Err(AnimateError::StaleHandle(self.source))
        }
    }
}
