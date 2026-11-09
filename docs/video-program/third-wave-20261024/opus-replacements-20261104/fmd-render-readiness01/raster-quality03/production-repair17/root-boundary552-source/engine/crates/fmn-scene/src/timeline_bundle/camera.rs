//! Additive FMTL/1 minor 1: one exact camera capture per recorded output frame.
//! Minor-0 writers and their bytes remain unchanged. Old strict readers reject
//! minor 1 instead of showing a movie with a missing camera.

use fmn_hash::serial::{Reader, Schema};
use fmn_mobject::Stage;
use fmn_render::Camera;
use fmn_render::camera::{CameraSample, CameraSampleError};

use super::{BundleReadError, TimelineBundle};

/// Camera-bearing variant of FMTL/1; appends a counted fixed-size camera track.
pub const CAMERA_TIMELINE_BUNDLE_SCHEMA: Schema = Schema::new(*b"FMTL", 1, 0, 1);

pub(super) fn read_track(
    reader: &mut Reader<'_>,
    frame_count: u32,
) -> Result<Option<Vec<CameraSample>>, BundleReadError> {
    if reader.version().1 == 0 {
        return Ok(None);
    }
    let count = reader.get_u32().map_err(BundleReadError::Malformed)?;
    if count != frame_count {
        return Err(BundleReadError::PlanInconsistent(
            "camera/frame count mismatch",
        ));
    }
    let count = usize::try_from(count)
        .map_err(|_| BundleReadError::PlanInconsistent("camera count exceeds host width"))?;
    // Validate the entire fixed-size table BEFORE reserving from an input count.
    // Exact equality also refuses trailing data under the certified strict policy.
    if count.checked_mul(CameraSample::WIRE_BYTES) != Some(reader.remaining()) {
        return Err(BundleReadError::PlanInconsistent(
            "camera track payload size",
        ));
    }
    let mut cameras = Vec::new();
    cameras
        .try_reserve_exact(count)
        .map_err(|_| BundleReadError::AllocationFailed {
            context: "camera track",
            requested: count,
        })?;
    for _ in 0..count {
        cameras.push(CameraSample::read_from(reader).map_err(BundleReadError::Camera)?);
    }
    Ok(Some(cameras))
}

pub(super) fn restore(
    sample: Option<&CameraSample>,
    resolution: (u32, u32),
    fps: u32,
    index: u32,
) -> Result<Option<Camera>, BundleReadError> {
    sample
        .map(|sample| {
            sample
                .camera(resolution, fps, u64::from(index) + 1)
                .map_err(|error| BundleReadError::Camera(CameraSampleError::Camera(error)))
        })
        .transpose()
}

impl TimelineBundle {
    /// Whether every frame carries a required camera/light/background capture.
    #[must_use]
    pub const fn has_camera_track(&self) -> bool {
        self.cameras.is_some()
    }

    /// Restore a captured camera, adapting only the requested output viewport.
    /// Equal-aspect replay preserves frame shape and quaternion bits exactly.
    /// Different-aspect replay preserves authored width. No callback or clock
    /// executes; the original frame index determines cache generation.
    ///
    /// # Errors
    /// Refuses an out-of-range index or an invalid viewport/camera.
    pub fn camera_at(
        &self,
        index: u32,
        resolution: (u32, u32),
    ) -> Result<Option<Camera>, BundleReadError> {
        if index >= self.frame_count {
            return Err(BundleReadError::FrameOutOfRange {
                index,
                total: self.frame_count,
            });
        }
        restore(
            self.cameras
                .as_ref()
                .map(|cameras| &cameras[index as usize]),
            resolution,
            self.fps(),
            index,
        )
    }

    /// Reconstruct one frame together with its required captured camera.
    /// Legacy artifacts return `None` and keep their existing caller-supplied
    /// viewport/fixed-camera contract. Camera-bearing frames must not be
    /// rendered through the geometry-only `stage_at` API.
    ///
    /// # Errors
    /// Preserves clock, snapshot, camera and frame-range refusals.
    pub fn stage_at_with_camera(
        &self,
        index: u32,
        resolution: (u32, u32),
    ) -> Result<(Stage, Option<Camera>), BundleReadError> {
        let camera = self.camera_at(index, resolution)?;
        Ok((self.stage_at_inner(index)?, camera))
    }
}
