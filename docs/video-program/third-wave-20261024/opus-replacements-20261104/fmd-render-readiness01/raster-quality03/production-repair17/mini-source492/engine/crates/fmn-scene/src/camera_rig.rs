//! Snapshot-native camera controls over the ordinary scalar tracker machinery.
//!
//! A rig contains no callback and owns no clock. Camera motion is produced by
//! Scene animations/updaters on its trackers, and a renderer samples those
//! values only after the ordinary capture boundary.

use fmn_anim::Transform;
use fmn_mobject::{Mob, Mobject, Stage};
use fmn_render::{Camera, CameraConfig};

use crate::{Scene, SceneError};

/// Twelve scalar trackers in a fixed, validated family layout.
///
/// Center (xyz), width, orientation (xyzw), vertical field of view, and light
/// position (xyz) are ordinary native state. Quaternion channels retain exact
/// authored orientation without Euler extraction/pole rounding. Scalar-channel
/// interpolation is normalized linear interpolation, NOT spherical interpolation
/// or a constant-angular-speed promise. `animate_to` chooses compatible target
/// signs; callers editing raw quaternion trackers must do so themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraRig {
    root: Mob,
    channels: [Mob; 12],
}

fn values(camera: &Camera) -> [f64; 12] {
    let frame = camera.frame();
    let center = frame.center();
    let orientation = frame.orientation();
    let light = camera.light_source_position();
    [
        center[0],
        center[1],
        center[2],
        frame.width(),
        orientation[0],
        orientation[1],
        orientation[2],
        orientation[3],
        frame.field_of_view(),
        light[0],
        light[1],
        light[2],
    ]
}

fn allocate(stage: &mut Stage, values: [f64; 12]) -> Result<CameraRig, SceneError> {
    let root = stage.add(Mobject::new());
    let channels = values.map(|value| stage.add_value_tracker(value));
    for channel in channels {
        stage.attach(root, channel)?;
    }
    Ok(CameraRig { root, channels })
}

impl CameraRig {
    /// Create and add a point-free camera family to the Scene. The initial
    /// camera is validated before any tracker is allocated. Resolution/aspect,
    /// background, samples and other capture policy stay outside the rig.
    pub fn new(scene: &mut Scene, config: &CameraConfig) -> Result<Self, SceneError> {
        let camera = Camera::new(config.clone())?;
        let rig = allocate(scene.stage_mut(), values(&camera))?;
        scene.add(&[rig.root])?;
        Ok(rig)
    }

    /// Animate the rig to the target's center, width, quaternion, FOV and light.
    /// Returns the existing Choreo Transform, so timing, rate functions and
    /// composition remain the normal native animation API. No new interpolator
    /// or clock is introduced. Resolution, background, samples and other output
    /// policy are NOT animated by this operation.
    ///
    /// Both endpoints are validated before allocating the detached target. The
    /// target is a fresh, updater-free family, not a copy that could keep moving
    /// under the source's callbacks. Quaternion target signs are chosen in the
    /// current orientation's hemisphere to avoid the q-to-minus-q zero midpoint.
    pub fn animate_to(
        self,
        scene: &mut Scene,
        target: &CameraConfig,
    ) -> Result<Transform, SceneError> {
        let target_camera = Camera::new(target.clone())?;
        let current = self.sample(scene.stage(), target)?;
        let from = current.frame.orientation();
        let to = target_camera.frame().orientation();
        let mut target_values = values(&target_camera);
        let dot = from[0] * to[0] + from[1] * to[1] + from[2] * to[2] + from[3] * to[3];
        if dot < 0.0 {
            for value in &mut target_values[4..8] {
                *value = -*value;
            }
        }
        let target = allocate(scene.stage_mut(), target_values)?;
        Ok(Transform::new(self.root, target.root))
    }

    /// Point-free family root; suitable for a native updater or suspension.
    #[must_use]
    pub const fn root(self) -> Mob {
        self.root
    }

    /// World-space center tracker handles, in xyz order.
    #[must_use]
    pub const fn center(self) -> [Mob; 3] {
        [self.channels[0], self.channels[1], self.channels[2]]
    }

    /// Frame width. Height follows the output aspect ratio on each sample.
    #[must_use]
    pub const fn width(self) -> Mob {
        self.channels[3]
    }

    /// Quaternion tracker handles in the existing scipy xyzw convention.
    #[must_use]
    pub const fn orientation(self) -> [Mob; 4] {
        [
            self.channels[4],
            self.channels[5],
            self.channels[6],
            self.channels[7],
        ]
    }

    /// Vertical field of view in radians, strictly between zero and pi.
    #[must_use]
    pub const fn field_of_view(self) -> Mob {
        self.channels[8]
    }

    /// World-space light-position handles, in xyz order.
    #[must_use]
    pub const fn light(self) -> [Mob; 3] {
        [self.channels[9], self.channels[10], self.channels[11]]
    }

    /// Ordered scene-root position, suitable for a factory binding identity.
    /// It deliberately excludes the process-local arena ID. Removing/replacing
    /// a channel or root is an error, not permission to bind a different camera.
    pub fn binding_index(self, stage: &Stage) -> Result<u64, SceneError> {
        let index = stage
            .roots()
            .iter()
            .position(|&root| root == self.root)
            .ok_or(SceneError::InvalidState(
                "camera rig root is not in this Scene",
            ))?;
        let entry = stage
            .get(self.root)
            .ok_or(SceneError::InvalidState("camera rig root is stale"))?;
        if entry.submobjects() != self.channels.as_slice() {
            return Err(SceneError::InvalidState(
                "camera rig channel topology changed",
            ));
        }
        u64::try_from(index)
            .map_err(|_| SceneError::InvalidState("camera rig root index exceeds u64"))
    }

    /// Read a capture configuration without running callbacks or mutating the
    /// Stage. This works equally on a retained in-memory FramePacket Stage.
    /// Missing/non-scalar/non-finite channels, invalid zoom/FOV, and a zero
    /// quaternion refuse through the existing camera error contract.
    pub fn sample(self, stage: &Stage, base: &CameraConfig) -> Result<CameraConfig, SceneError> {
        self.binding_index(stage)?;
        let mut values = [0.0; 12];
        for (value, handle) in values.iter_mut().zip(self.channels) {
            *value = stage.tracker_value(handle).ok_or(SceneError::InvalidState(
                "camera rig channel is not a live scalar tracker",
            ))?;
            if !value.is_finite() {
                return Err(SceneError::InvalidState("camera rig channel is non-finite"));
            }
        }
        let mut config = base.clone();
        config.frame.set_center([values[0], values[1], values[2]])?;
        config.frame.set_width(values[3])?;
        let orientation = [values[4], values[5], values[6], values[7]];
        if orientation != config.frame.orientation() {
            config.frame.set_orientation(orientation)?;
        }
        config.frame.set_field_of_view(values[8])?;
        config.light_source_position = [values[9], values[10], values[11]];
        let camera = Camera::new(config.clone())?;
        config.frame = camera.frame().clone();
        Ok(config)
    }
}
