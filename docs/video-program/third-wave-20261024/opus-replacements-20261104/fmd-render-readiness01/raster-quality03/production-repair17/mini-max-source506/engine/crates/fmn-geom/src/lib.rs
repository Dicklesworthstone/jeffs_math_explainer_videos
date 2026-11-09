//! Chisel: the geometry kernel — quadratic-Bézier paths, true arc length, booleans, SVG processing (§7).
//!
//! This crate owns the shared-anchor `QuadPath` model (§7.1) — the shared
//! vocabulary of Chisel, Marionette's point data, Scribe's glyph outlines,
//! and Lumen's compiled paths — plus the Bézier/smoothing layers under it.
//! Object-space geometry computes in f64 (§6.1). Semantics are ported from
//! the pinned Reference (`3b1b/manim` @ `6199a00d4c1b1127ebe45cb629c3f22538b10e13`)
//! and locked by the fixtures in `fixtures/` and `tests/`.
//!
//! [`space_ops`] is the Reference's vector/rotation vocabulary ported
//! signature for signature (§7.5), over [`rotation`], which fixes scipy
//! `Rotation`'s quaternion and Euler conventions as our documented
//! semantics — those conventions are stated normatively in
//! `docs/ROTATION_CONVENTIONS.md`, not inferred from this code.
//!
//! The one error-bounded cubic→quadratic converter is the ingress for API and
//! smoothing cubics (fm-6cf); TrueType quadratics pass through losslessly.
//! [`boolean`] is the permanent certified flatten-and-clip path boolean
//! (§7.4). [`isolines`] is the adaptive quadtree isoline extractor
//! `ImplicitFunction` rides (§7.7), and [`earclip`] the deterministic
//! hole-support triangulator for booleans' flatten fallback and mesh export
//! (§7.8, fm-81u). [`svg`] is the hardened user-SVG document processor
//! (§7.6, fm-6nm) with an explicit accept/reject matrix and a UTF-8-safe
//! byte-admission facade before its tokenizer.
#![forbid(unsafe_code)]

pub mod arclength;
pub mod bezier;
pub mod boolean;
pub mod cubic;
pub mod distance;
pub mod earclip;
pub mod isolines;
pub mod quadpath;
pub mod rotation;
pub mod smoothing;
pub mod space_ops;
#[path = "svg_safe.rs"]
pub mod svg;
#[path = "svg.rs"]
mod svg_parser;

mod scalar;
mod vec;

pub use arclength::{ArcLengthTable, CachedArcLength};
pub use boolean::{
    BooleanError, BooleanLimits, BooleanOperand, BooleanOperation, BooleanOptions, BooleanPhase,
    BooleanResult, BooleanRoute, BooleanStats, FillRule, path_boolean, path_boolean_flattened,
};
pub use earclip::{
    EarClipError, EarClipOptions, MAX_EARCLIP_VERTICES, RingRole, triangulate,
    triangulate_with_options,
};
pub use isolines::{
    IsolineConfig, IsolineError, IsolineStats, MAX_ISOLINE_EVALUATIONS, MAX_ISOLINE_LEAVES,
    plot_isoline, plot_isoline_with_stats, try_plot_isoline, try_plot_isoline_with_stats,
};
pub use quadpath::{
    AnchorMode, DEFAULT_TOLERANCE_FOR_POINT_EQUALITY, MAX_SUBDIVIDED_CURVES, QuadPath,
};
pub use rotation::{EulerAngles, EulerSeq, Quat};
pub use smoothing::{MAX_CLOSED_SMOOTHING_DIMENSION, MAX_CLOSED_SMOOTHING_MATRIX_CELLS};
pub use space_ops::{
    MAX_COMPASS_DIRECTIONS, MAX_THICK_DIAGONAL_CELLS, MAX_THICK_DIAGONAL_DIMENSION, SpaceOpsError,
    rotation_matrix,
};
pub use svg::{
    DEFAULT_SVG_TOLERANCE, LineCap, LineJoin, Paint, SvgDocument, SvgError, SvgLimits, SvgShape,
    SvgStyle, emit_path_data, emit_svg, emit_svg_document,
};
pub use vec::Mat3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomError {
    /// A point run that would leave the shared-anchor layout with a nonzero
    /// even length (the invariant requires 0 or odd).
    EvenPointCount {
        /// The offending length.
        len: usize,
    },
    /// An operation that needs an existing path end was called on an empty
    /// path.
    EmptyPath,
    /// `set_anchors_and_handles` requires exactly one more anchor than
    /// handles.
    MismatchedAnchorsAndHandles {
        /// Number of anchors supplied.
        anchors: usize,
        /// Number of handles supplied.
        handles: usize,
    },
    /// A smoothing solve hit a singular system.
    SingularSystem,
    /// The smoothing solver refused the system for a reason other than
    /// singularity (for example, non-finite anchor coordinates).
    SolverRefused,
    /// Sizing a smoothing system from the supplied anchor count overflowed
    /// `usize` before allocation.
    SmoothingSizeOverflow {
        /// Number of anchors supplied to the smoothing operation.
        anchors: usize,
    },
    /// A closed smoothing solve would exceed the fixed dense-workspace
    /// budget.
    ClosedSmoothingBudgetExceeded {
        /// Width and height of the requested dense system.
        dimension: usize,
        /// Number of `f64` cells in the requested dense system.
        cells: usize,
    },
    /// `subdivide_sharp_curves` requires a positive, finite angle threshold.
    InvalidSubdivisionThreshold,
    /// A requested subdivision or curve insertion would exceed the bounded
    /// output-curve budget.
    SubdivisionBudgetExceeded {
        /// Total output curves requested, or `usize::MAX` when the count
        /// arithmetic itself overflowed.
        requested: usize,
        /// Maximum output curves admitted by the operation.
        max: usize,
    },
    /// A conversion tolerance that was not a positive, finite number.
    ///
    /// This and [`GeomError::ToleranceUnreachable`] are the converter's
    /// explicit failure modes.
    InvalidTolerance,
    /// Holding the requested tolerance would need more than
    /// [`cubic::MAX_SEGMENTS`] quadratics.
    ///
    /// This is a resource/representation guard against sizing an allocation
    /// from arithmetic or silently emitting a coarser curve.
    ToleranceUnreachable {
        /// The piece count the tolerance would have required.
        needed: usize,
    },
    /// An arc was requested with zero quadratic components — the
    /// degenerate case the caller must refuse rather than emit a one-point
    /// pseudo-arc (fm-4tb.1).
    ZeroArcComponents,
    /// An arc was requested with a non-finite angle (NaN or infinite) —
    /// the component count cannot be computed (fm-4tb.1).
    NonFiniteArcAngle,
    /// An arc's point-count arithmetic (`2·n + 1`) overflowed `usize`
    /// before allocation (fm-4tb.1).
    ArcComponentOverflow {
        /// The offending component count.
        count: usize,
    },
    /// An arc's component count exceeds the declared
    /// [`bezier::MAX_ARC_COMPONENTS`] budget (fm-4tb.1).
    ArcComponentsAboveBudget {
        /// The offending component count.
        count: usize,
        /// The declared budget.
        budget: usize,
    },
    /// A rectangle-like builder received a width or height that was not a
    /// positive, finite number — the degenerate or garbage extent would
    /// silently emit an invisible or unbounded shape.
    InvalidExtent,
}

impl std::fmt::Display for GeomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EvenPointCount { len } => write!(
                f,
                "shared-anchor point runs must have odd length when nonempty (got {len})"
            ),
            Self::EmptyPath => write!(f, "operation requires a path with at least one point"),
            Self::MismatchedAnchorsAndHandles { anchors, handles } => write!(
                f,
                "need exactly one more anchor than handles (got {anchors} anchors, {handles} handles)"
            ),
            Self::SingularSystem => write!(f, "smoothing solve hit a singular linear system"),
            Self::SolverRefused => {
                write!(f, "smoothing solver refused the linear system")
            }
            Self::SmoothingSizeOverflow { anchors } => write!(
                f,
                "smoothing system dimensions overflow usize for {anchors} anchors"
            ),
            Self::ClosedSmoothingBudgetExceeded { dimension, cells } => write!(
                f,
                "closed smoothing needs a {dimension} by {dimension} system ({cells} cells), above the {}-dimension/{}-cell budget",
                smoothing::MAX_CLOSED_SMOOTHING_DIMENSION,
                smoothing::MAX_CLOSED_SMOOTHING_MATRIX_CELLS
            ),
            Self::InvalidSubdivisionThreshold => {
                write!(
                    f,
                    "curve-subdivision angle threshold must be positive and finite"
                )
            }
            Self::SubdivisionBudgetExceeded { requested, max } => write!(
                f,
                "curve operation requests {requested} output curves, above the {max} cap"
            ),
            Self::InvalidTolerance => {
                write!(f, "conversion tolerance must be a positive, finite number")
            }
            Self::ToleranceUnreachable { needed } => write!(
                f,
                "tolerance would need {needed} quadratics, above the {} cap",
                cubic::MAX_SEGMENTS
            ),
            Self::ZeroArcComponents => {
                write!(f, "an arc needs at least one quadratic component")
            }
            Self::NonFiniteArcAngle => write!(f, "arc angle must be finite"),
            Self::ArcComponentOverflow { count } => write!(
                f,
                "arc component count {count} overflows point-count arithmetic"
            ),
            Self::ArcComponentsAboveBudget { count, budget } => {
                write!(f, "arc component count {count} exceeds the {budget} budget")
            }
            Self::InvalidExtent => {
                write!(f, "rectangle extent must be positive and finite")
            }
        }
    }
}

impl std::error::Error for GeomError {}
