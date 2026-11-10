//! Topology-derived execution plans and the bounded frame pipeline (§17.4).
//!
//! This crate is deliberately a scheduling seam rather than a second scene or
//! renderer abstraction. Higher layers freeze their semantic state into an
//! owned job, then hand that job to [`FramePipeline`]. The runtime never
//! inspects a mobject, a render command, or a pixel; that keeps the crate below
//! `fmn-anim`, `fmn-render`, and `fmn-output` in the governed crate DAG.
//!
//! [`ExecutionPlan`] turns [`fmn_platform::topology::HardwareTopology`] into
//! advisory worker teams and a real memory-bounded in-flight budget. Certified
//! plans pin every bit-affecting choice (notably tile dimensions); standard
//! plans may consume a fingerprint-matched [`AutotuneCache`]. The scheduler is
//! free, but the caller-supplied jobs must obey §10.5: their result may not
//! depend on team assignment or completion order.
#![forbid(unsafe_code)]

mod pipeline;
mod plan;
mod stream;

pub use fmn_platform::profile::{
    PROFILE_SCHEMA, ProfileCounter, ProfileCounterRecord, ProfileLane, ProfileLaneRole,
    ProfilePath, ProfilePhase, ProfileRecord, ProfileRecorder, ProfileSnapshot, ProfileSpan,
    ProfileSpanRecord,
};
pub use pipeline::{
    BarrierContext, CancellationToken, FramePipeline, PipelineError, PipelineEvent,
    PipelineFailure, PipelineStage, PipelineStages, PipelineStats, StageUtilization,
};
pub use plan::{
    AutotuneCache, AutotuneProfile, Determinism, ExecutionEngine, ExecutionPlan, LocalityLane,
    OutputPixelFormat, PlanError, PlanRequest, RenderIntent, SurfaceSpec, TeamPlan, TeamRole,
    TopologyFingerprint, TuningSource,
};

pub use stream::{FramePermit, FrameStream, FrameStreamError};
