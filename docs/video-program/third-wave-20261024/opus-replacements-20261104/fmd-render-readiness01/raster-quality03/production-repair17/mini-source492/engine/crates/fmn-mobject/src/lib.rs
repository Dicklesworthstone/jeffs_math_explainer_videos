//! Marionette: the mobject engine — Stage arena, RecordBuffer, family tree, styles, updaters (§8).
//!
//! This crate implements the ownership model **as ratified by G0-1**
//! (D-11, `docs/g0/G0-1-object-model-ratification.md`): the [`Stage`]
//! arena with generational stage-scoped [`Mob`] handles, rooted lifetimes,
//! CoW [`Snapshot`]s, and the §8.2 record/view protocol core. The ten G0-1
//! lifetime scenarios are this crate's permanent regression suite
//! (`tests/scenarios.rs`).
//!
//! The family/positional API, bounding boxes, and the uniform inventory land
//! here (fm-jru): [`BoundingBox`] with automatic subtree-signature
//! invalidation, the positional surface on [`Stage`] (`shift`/`next_to`/
//! `move_to`/`align_to`/`arrange`/…), and the typed [`Uniforms`] inventory
//! (including the C-2/BN-07 and C-7 rulings).
//!
//! The dynamic-behavior surface (§8.6, fm-yra): insertion-ordered dt/non-dt
//! updaters with suspend/resume and the C-5 once-only `call` fix
//! ([`stage`]), ValueTrackers and `always_redraw`/`f_always` plus the C-6
//! group-addition correction ([`dynamics`]), and the `.animate` builder
//! with the Reference's real chaining rules ([`animate`]).
//!
//! Copy semantics (§8.3, fm-ncq) are complete: the [`stage::CopyMap`]
//! remap hook for the binding tier, the `generate_target` / `save_state` /
//! `restore_mobject` machinery with the Reference's exact link topology,
//! and `become` (over already-aligned families; `align_family` itself
//! lands with the Transform machinery, fm-cye).
#![forbid(unsafe_code)]

pub mod align;
pub mod animate;
pub mod bbox;
pub mod dynamics;
pub mod mobject;
pub mod numpy;
pub mod order;
pub mod persist;
pub mod placement;
pub mod positional;
pub mod record;
pub mod render_snapshot;
pub mod shape;
pub mod stage;
pub mod uniforms;

pub use align::MAX_ALIGNED_SUBMOBJECTS;
pub use animate::{AnimBuilder, AnimateArgs, AnimateError, BuiltAnimate, IntoAnimate};
pub use bbox::BoundingBox;
pub use dynamics::{Tracker, TrackerKind};
pub use mobject::{
    ImageColorSpace, ImagePixelFormat, ImageResource, ImageResourceError, ImageSampler, ImageWrap,
    Mobject, RenderPrimitive,
};
pub use numpy::{ExportError, LANE_BYTES, LANE_DTYPE, RecordArray, RecordArrayMut, RecordDType};
pub use order::{BatchKey, DrawItem, DrawPlan, PassOrder, ProgramKind};
pub use persist::{
    DecodedSceneState, DecodedSnapshot, PersistError, SCENE_STATE_SCHEMA, SNAPSHOT_SCHEMA,
    SceneState, SnapshotLimits, UpdaterIdentity, UpdaterKindTag, UpdaterManifest,
};
pub use placement::Placement;
pub use positional::PosTarget;
pub use record::{FieldSpec, MirrorSet, RecordBuffer, RecordError, RecordSchema, RecordView};
pub use render_snapshot::{RenderSnapshotError, RenderSnapshotLimits};
pub use shape::ShapeTag;
pub use stage::{
    CopyMap, Entry, FamilyCheckpoint, IdBuildHasher, IdHasher, Mob, Snapshot, Stage, UpdaterFn,
    UpdaterId, UpdaterSlot,
};
pub use uniforms::{JointType, Uniforms};

/// Errors from the mobject engine's ownership layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageError {
    /// The handle is stale (its entry was deleted), foreign (minted by
    /// another stage — the two-scene policy), or otherwise unresolvable.
    /// Always a defined state: a dead handle can never reach a recycled
    /// slot's data.
    StaleHandle,
    /// Attaching this edge would make the family graph cyclic (the
    /// Reference would recurse forever on such a graph; we refuse it).
    CycleDetected,
    /// `restore` without a prior `save_state` — the Reference raises
    /// "Trying to restore without having saved".
    NoSavedState,
    /// `become` between families the caller did not first reconcile through
    /// [`Stage::align_family`].
    FamilyShapeMismatch,
    /// `become` between records of different schemas — the Reference's
    /// `set_data` asserts dtype equality; this is the typed refusal.
    SchemaMismatch,
    /// A sampled surface has invalid topology, records, or an excessive UV grid.
    SurfaceGrid(&'static str),
    /// The geometry kernel refused a point run or bounded operation during
    /// alignment (alignment reads point runs as [`fmn_geom::QuadPath`]s).
    Geometry(fmn_geom::GeomError),
    /// Family alignment was asked to create more direct children than its
    /// fixed resource ceiling permits.
    SubmobjectBudgetExceeded {
        /// Requested final direct-child count.
        requested: usize,
        /// Maximum accepted final direct-child count.
        max: usize,
    },
    /// The monotonically assigned updater identity space is exhausted.
    /// Reuse would make durable replay manifests ambiguous.
    UpdaterIdExhausted,
    /// The proxy pin counter is exhausted. Wrapping it to zero would make a
    /// live proxy look unpinned and could finalize a deferred deletion.
    PinCountExhausted,
    /// Durable replay attempted to install updater callables against a
    /// different target/identity set than the decoded manifest named.
    UpdaterBindingMismatch,
    /// `put_start_and_end_on` on a family with no points, or whose first
    /// and last points coincide — the Reference's "Cannot position
    /// endpoints of closed loop". There is no rotation that separates two
    /// identical points, so the caller has to decide (`Line` rebuilds its
    /// path from the new ends instead).
    DegenerateEndpoints,
    /// A record sizing proof refused the operation (fm-vek.2) — the
    /// target record count and schema stride cannot be addressed. The
    /// buffer is untouched.
    Record(RecordError),
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StaleHandle => {
                write!(f, "stale, deleted, or foreign mobject handle")
            }
            Self::CycleDetected => {
                write!(f, "attachment would create a cycle in the family graph")
            }
            Self::NoSavedState => {
                write!(f, "trying to restore without having saved")
            }
            Self::FamilyShapeMismatch => {
                write!(
                    f,
                    "become between families of different shapes \
                     (family alignment lands with fm-cye)"
                )
            }
            Self::SchemaMismatch => {
                write!(f, "become between records of different schemas")
            }
            Self::SurfaceGrid(reason) => write!(f, "surface grid alignment refused: {reason}"),
            Self::Geometry(err) => {
                write!(f, "geometry operation refused during alignment: {err}")
            }
            Self::SubmobjectBudgetExceeded { requested, max } => {
                write!(
                    f,
                    "family alignment requested {requested} direct submobjects; maximum is {max}"
                )
            }
            Self::UpdaterIdExhausted => {
                write!(f, "updater identity space is exhausted")
            }
            Self::PinCountExhausted => {
                write!(f, "proxy pin count is exhausted")
            }
            Self::UpdaterBindingMismatch => {
                write!(f, "updater binding does not match the decoded manifest")
            }
            Self::DegenerateEndpoints => {
                write!(f, "cannot position endpoints of a closed or empty path")
            }
            Self::Record(err) => {
                write!(f, "record sizing refused: {err}")
            }
        }
    }
}

impl std::error::Error for StageError {}
