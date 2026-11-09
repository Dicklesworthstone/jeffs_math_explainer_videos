//! Native executable scene programs for the isolated Studio worker.
//!
//! These owners use Proscenium's stepped animation/wait drivers. They retain
//! executable callbacks while paused rather than pretending a durable snapshot
//! can recreate them. Deferred authoring builds later objects and animations
//! from the preceding segment's completed state.

mod build;
mod camera;
mod edits;
mod program;
mod raster;
mod service;

pub use build::{NativeBuild, NativeBuildContext};
pub use camera::camera_capture_backend;
pub use program::{MAX_NATIVE_SEGMENTS, NativeSceneProgram, NativeSegment};
pub use service::{NativeReplayPolicy, NativeSceneWorker, NativeWorkerConfig};
