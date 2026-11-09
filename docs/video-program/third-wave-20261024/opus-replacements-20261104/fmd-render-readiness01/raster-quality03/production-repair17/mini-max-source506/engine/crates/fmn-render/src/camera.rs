//! CameraFrame, Camera, and the kept 3b1b projection (§10.4, fm-0gy).
//!
//! The Reference does not use a conventional perspective matrix. Scene code
//! depends on its exact sequence:
//!
//! 1. map world space through the inverse camera orientation and frame scale;
//! 2. mix that point with the original point by `is_fixed_in_frame`;
//! 3. apply the fixed frame-rescale factors;
//! 4. set `w = 1 - z` and replace `z` by `-0.1 z`.
//!
//! Those constants are aesthetic and compositional API, so this module keeps
//! them exactly. The rasterizer consumes [`ClipPoint`]; there is deliberately
//! no second generic "projection" abstraction beside the camera.

mod capture;
pub use capture::{CameraSample, CameraSampleError};

use fmn_core::AaPolicy;
use fmn_core::color::{LinearRgba, Srgb};
use fmn_core::constants::{
    DEFAULT_FPS, DEFAULT_PIXEL_HEIGHT, DEFAULT_PIXEL_WIDTH, FRAME_HEIGHT, FRAME_WIDTH, PI,
};
use fmn_core::types::Vec3;
use fmn_geom::rotation::{
    EulerSeq, IDENTITY_QUAT, Quat, compose_quat, euler_from_quat, matrix_from_unit_quat,
    normalized, quat_from_euler, quat_from_rotvec,
};

/// The Reference's vertical field of view, 45 degrees.
pub const DEFAULT_FOVY: f64 = PI / 4.0;
/// Camera frames sort before ordinary scene objects.
pub const CAMERA_FRAME_Z_INDEX: i32 = -1;
/// Default mutable light-source position.
pub const DEFAULT_LIGHT_POSITION: Vec3 = [-10.0, 10.0, 10.0];
/// ThreeDCamera's compatibility sample count.
pub const THREE_D_CAMERA_SAMPLES: u8 = 4;

/// A camera construction or mutation was not finite and meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraError {
    /// Pixel dimensions or frame dimensions were zero.
    ZeroDimension,
    /// A scalar or vector contained NaN or infinity.
    NonFinite,
    /// The field of view was outside `(0, π)`.
    InvalidFieldOfView,
    /// The quaternion had zero norm.
    InvalidOrientation,
    /// The Euler sequence was not accepted by scipy's Rotation convention.
    InvalidEulerAxes,
    /// Frames per second was zero.
    ZeroFrameRate,
    /// Exact camera clipping exceeded a caller-declared piece ceiling.
    ClipLimitExceeded {
        /// Stable clipping buffer name.
        resource: &'static str,
        /// Inclusive piece ceiling.
        limit: usize,
        /// Exact requested piece count.
        requested: usize,
    },
    /// A bounded camera-clipping buffer could not be reserved.
    AllocationFailed {
        /// Stable clipping buffer name.
        resource: &'static str,
        /// Elements requested from that buffer.
        requested: usize,
    },
}

impl std::fmt::Display for CameraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimension => f.write_str("camera dimensions must be nonzero"),
            Self::NonFinite => f.write_str("camera values must be finite"),
            Self::InvalidFieldOfView => {
                f.write_str("camera vertical field of view must lie strictly between 0 and pi")
            }
            Self::InvalidOrientation => {
                f.write_str("camera orientation must be a nonzero quaternion")
            }
            Self::InvalidEulerAxes => {
                f.write_str("camera Euler axes must be a valid three-axis scipy sequence")
            }
            Self::ZeroFrameRate => f.write_str("camera frame rate must be nonzero"),
            Self::ClipLimitExceeded {
                resource,
                limit,
                requested,
            } => write!(
                f,
                "{resource} needs {requested} pieces, exceeding the limit {limit}"
            ),
            Self::AllocationFailed {
                resource,
                requested,
            } => write!(f, "could not reserve {requested} rows for {resource}"),
        }
    }
}

impl std::error::Error for CameraError {}

fn finite3(value: Vec3) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn finite4(value: [f64; 4]) -> bool {
    value.iter().all(|component| component.is_finite())
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The mutable semantic camera frame.
///
/// It is a Rust value rather than a renderer-global singleton. A scene can
/// keep it in snapshotted state, clone it at capture, and animate every field
/// without making projection depend on worker timing.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraFrame {
    center: Vec3,
    shape: [f64; 2],
    fovy: f64,
    orientation: Quat,
    default_orientation: Quat,
    euler_axes: String,
    revision: u64,
}

impl Default for CameraFrame {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            shape: [FRAME_WIDTH, FRAME_HEIGHT],
            fovy: DEFAULT_FOVY,
            orientation: IDENTITY_QUAT,
            default_orientation: IDENTITY_QUAT,
            euler_axes: "zxz".to_owned(),
            revision: 1,
        }
    }
}

impl CameraFrame {
    /// Build a validated camera frame.
    pub fn new(
        frame_shape: [f64; 2],
        center: Vec3,
        fovy: f64,
        euler_axes: &str,
    ) -> Result<Self, CameraError> {
        let mut frame = Self::default();
        frame.set_shape(frame_shape)?;
        frame.set_center(center)?;
        frame.set_field_of_view(fovy)?;
        frame.set_euler_axes(euler_axes)?;
        Ok(frame)
    }

    /// Monotone camera-state revision for derived tables and tile keys.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// Frame center in scene coordinates.
    #[must_use]
    pub const fn center(&self) -> Vec3 {
        self.center
    }

    /// `(width, height)` in scene coordinates.
    #[must_use]
    pub const fn shape(&self) -> [f64; 2] {
        self.shape
    }

    /// Frame width in scene coordinates.
    #[must_use]
    pub const fn width(&self) -> f64 {
        self.shape[0]
    }

    /// Frame height in scene coordinates.
    #[must_use]
    pub const fn height(&self) -> f64 {
        self.shape[1]
    }

    /// Width divided by height.
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        self.width() / self.height()
    }

    /// Reference frame scale, `height / FRAME_HEIGHT`.
    #[must_use]
    pub fn scale(&self) -> f64 {
        self.height() / FRAME_HEIGHT
    }

    /// Current normalized scipy-order quaternion `[x, y, z, w]`.
    #[must_use]
    pub const fn orientation(&self) -> Quat {
        self.orientation
    }

    /// The current Euler sequence.
    #[must_use]
    pub fn euler_axes(&self) -> &str {
        &self.euler_axes
    }

    /// Vertical field of view in radians.
    #[must_use]
    pub const fn field_of_view(&self) -> f64 {
        self.fovy
    }

    /// Focal distance implied by frame height and vertical field of view.
    #[must_use]
    pub fn focal_distance(&self) -> f64 {
        0.5 * self.height() / fmn_dmath::tan(0.5 * self.fovy)
    }

    /// Set the frame center.
    pub fn set_center(&mut self, center: Vec3) -> Result<&mut Self, CameraError> {
        if !finite3(center) {
            return Err(CameraError::NonFinite);
        }
        self.center = center;
        self.touch();
        Ok(self)
    }

    /// Set `(width, height)`.
    pub fn set_shape(&mut self, shape: [f64; 2]) -> Result<&mut Self, CameraError> {
        if !shape.iter().all(|component| component.is_finite()) {
            return Err(CameraError::NonFinite);
        }
        if shape[0] <= 0.0 || shape[1] <= 0.0 {
            return Err(CameraError::ZeroDimension);
        }
        self.shape = shape;
        self.touch();
        Ok(self)
    }

    /// Set width without changing height.
    pub fn set_width(&mut self, width: f64) -> Result<&mut Self, CameraError> {
        self.set_shape([width, self.height()])
    }

    /// Set height without changing width.
    pub fn set_height(&mut self, height: f64) -> Result<&mut Self, CameraError> {
        self.set_shape([self.width(), height])
    }

    /// Resize one frame dimension to a pixel aspect ratio.
    ///
    /// `fixed_height = false` keeps width (the Reference's default);
    /// `fixed_height = true` keeps height.
    pub fn resize_to_aspect_ratio(
        &mut self,
        aspect_ratio: f64,
        fixed_height: bool,
    ) -> Result<&mut Self, CameraError> {
        if !aspect_ratio.is_finite() {
            return Err(CameraError::NonFinite);
        }
        if aspect_ratio <= 0.0 {
            return Err(CameraError::ZeroDimension);
        }
        if fixed_height {
            self.set_width(aspect_ratio * self.height())
        } else {
            self.set_height(self.width() / aspect_ratio)
        }
    }

    /// Set a normalized orientation from scipy-order quaternion data.
    pub fn set_orientation(&mut self, orientation: Quat) -> Result<&mut Self, CameraError> {
        if !finite4(orientation) {
            return Err(CameraError::NonFinite);
        }
        self.orientation = normalized(orientation).ok_or(CameraError::InvalidOrientation)?;
        self.touch();
        Ok(self)
    }

    /// Remember the current orientation as the reset orientation.
    pub fn make_orientation_default(&mut self) -> &mut Self {
        self.default_orientation = self.orientation;
        self
    }

    /// Reset shape, center, and orientation to the Reference state.
    pub fn to_default_state(&mut self) -> &mut Self {
        self.shape = [FRAME_WIDTH, FRAME_HEIGHT];
        self.center = [0.0; 3];
        self.orientation = self.default_orientation;
        self.touch();
        self
    }

    /// Set the scipy Euler sequence.
    pub fn set_euler_axes(&mut self, axes: &str) -> Result<&mut Self, CameraError> {
        EulerSeq::parse(axes).ok_or(CameraError::InvalidEulerAxes)?;
        self.euler_axes.clear();
        self.euler_axes.push_str(axes);
        self.touch();
        Ok(self)
    }

    fn sequence(&self) -> EulerSeq {
        EulerSeq::parse(&self.euler_axes).expect("validated Euler axes remain valid")
    }

    /// `(theta, phi, gamma)` in the CameraFrame-facing order.
    ///
    /// scipy returns sequence order; the Reference reverses it and applies a
    /// wider `1e-2` pole merge for animated camera continuity.
    #[must_use]
    pub fn euler_angles(&self) -> [f64; 3] {
        if self.orientation == IDENTITY_QUAT {
            return [0.0; 3];
        }
        let Some(result) = euler_from_quat(self.orientation, self.sequence()) else {
            return [0.0; 3];
        };
        let mut angles = [result.angles[2], result.angles[1], result.angles[0]];
        if self.euler_axes == "zxz" {
            if angles[1].abs() <= 1e-2 {
                angles[0] += angles[2];
                angles[2] = 0.0;
            } else if (angles[1] - PI).abs() <= 1e-2 {
                angles[0] -= angles[2];
                angles[2] = 0.0;
            }
        }
        angles
    }

    /// Replace any subset of `(theta, phi, gamma)`.
    pub fn set_euler_angles(
        &mut self,
        theta: Option<f64>,
        phi: Option<f64>,
        gamma: Option<f64>,
    ) -> Result<&mut Self, CameraError> {
        let requested = [theta, phi, gamma];
        if requested.iter().flatten().any(|angle| !angle.is_finite()) {
            return Err(CameraError::NonFinite);
        }
        let mut angles = self.euler_angles();
        for (slot, value) in angles.iter_mut().zip(requested) {
            if let Some(value) = value {
                *slot = value;
            }
        }
        let orientation = if angles == [0.0; 3] {
            IDENTITY_QUAT
        } else {
            quat_from_euler(self.sequence(), [angles[2], angles[1], angles[0]])
        };
        self.set_orientation(orientation)
    }

    /// Increment `(theta, phi, gamma)`, applying the Reference pole ranges.
    pub fn increment_euler_angles(
        &mut self,
        dtheta: f64,
        dphi: f64,
        dgamma: f64,
    ) -> Result<&mut Self, CameraError> {
        if ![dtheta, dphi, dgamma].iter().all(|angle| angle.is_finite()) {
            return Err(CameraError::NonFinite);
        }
        let old = self.euler_angles();
        let mut new = [old[0] + dtheta, old[1] + dphi, old[2] + dgamma];
        if self.euler_axes == "zxz" {
            new[1] = new[1].clamp(0.0, PI);
        } else if self.euler_axes == "zxy" {
            new[1] = new[1].clamp(-PI / 2.0, PI / 2.0);
        }
        self.set_orientation(quat_from_euler(self.sequence(), [new[2], new[1], new[0]]))
    }

    /// Pre-compose an axis-angle rotation, matching `rot * orientation`.
    pub fn rotate(&mut self, angle: f64, axis: Vec3) -> Result<&mut Self, CameraError> {
        if !angle.is_finite() || !finite3(axis) {
            return Err(CameraError::NonFinite);
        }
        let length = dot(axis, axis).sqrt();
        if length == 0.0 {
            return Err(CameraError::InvalidOrientation);
        }
        let rot = quat_from_rotvec([
            angle * axis[0] / length,
            angle * axis[1] / length,
            angle * axis[2] / length,
        ]);
        self.set_orientation(compose_quat(rot, self.orientation))
    }

    /// Set the vertical field of view.
    pub fn set_field_of_view(&mut self, fovy: f64) -> Result<&mut Self, CameraError> {
        if !fovy.is_finite() {
            return Err(CameraError::NonFinite);
        }
        if !(0.0 < fovy && fovy < PI) {
            return Err(CameraError::InvalidFieldOfView);
        }
        self.fovy = fovy;
        self.touch();
        Ok(self)
    }

    /// Set focal distance by changing field of view.
    pub fn set_focal_distance(&mut self, focal_distance: f64) -> Result<&mut Self, CameraError> {
        if !focal_distance.is_finite() {
            return Err(CameraError::NonFinite);
        }
        if focal_distance <= 0.0 {
            return Err(CameraError::InvalidFieldOfView);
        }
        self.set_field_of_view(2.0 * fmn_dmath::atan(0.5 * self.height() / focal_distance))
    }

    /// Row-major affine view matrix, for column-vector multiplication.
    #[must_use]
    pub fn view_matrix(&self) -> [[f64; 4]; 4] {
        let rotation = matrix_from_unit_quat(self.orientation);
        let scale = self.scale();
        let mut view = [[0.0f64; 4]; 4];
        for row in 0..3 {
            // CameraFrame uses the inverse orientation, `R.T`.
            for column in 0..3 {
                view[row][column] = rotation[column][row] / scale;
            }
            view[row][3] = -(view[row][0] * self.center[0]
                + view[row][1] * self.center[1]
                + view[row][2] * self.center[2]);
        }
        view[3][3] = 1.0;
        view
    }

    /// Map a world point into the camera's fixed-frame coordinates.
    #[must_use]
    pub fn to_fixed_frame_point(&self, point: Vec3, relative: bool) -> Vec3 {
        let view = self.view_matrix();
        let w = if relative { 0.0 } else { 1.0 };
        [
            view[0][0] * point[0] + view[0][1] * point[1] + view[0][2] * point[2] + view[0][3] * w,
            view[1][0] * point[0] + view[1][1] * point[1] + view[1][2] * point[2] + view[1][3] * w,
            view[2][0] * point[0] + view[2][1] * point[1] + view[2][2] * point[2] + view[2][3] * w,
        ]
    }

    /// Inverse of [`CameraFrame::to_fixed_frame_point`].
    #[must_use]
    pub fn from_fixed_frame_point(&self, point: Vec3, relative: bool) -> Vec3 {
        let rotation = matrix_from_unit_quat(self.orientation);
        let scaled = [
            point[0] * self.scale(),
            point[1] * self.scale(),
            point[2] * self.scale(),
        ];
        let mut world = [
            rotation[0][0] * scaled[0] + rotation[0][1] * scaled[1] + rotation[0][2] * scaled[2],
            rotation[1][0] * scaled[0] + rotation[1][1] * scaled[1] + rotation[1][2] * scaled[2],
            rotation[2][0] * scaled[0] + rotation[2][1] * scaled[1] + rotation[2][2] * scaled[2],
        ];
        if !relative {
            for (component, center) in world.iter_mut().zip(self.center) {
                *component += center;
            }
        }
        world
    }

    /// Implied camera position used by the kept lighting model.
    #[must_use]
    pub fn implied_camera_location(&self) -> Vec3 {
        let rotation = matrix_from_unit_quat(self.orientation);
        let to_camera = [rotation[0][2], rotation[1][2], rotation[2][2]];
        let distance = self.focal_distance();
        [
            self.center[0] + distance * to_camera[0],
            self.center[1] + distance * to_camera[1],
            self.center[2] + distance * to_camera[2],
        ]
    }

    /// The fixed factors from the Reference's camera uniform refresh.
    #[must_use]
    pub fn frame_rescale_factors(&self) -> Vec3 {
        [
            2.0 / FRAME_WIDTH,
            2.0 / FRAME_HEIGHT,
            self.scale() / self.focal_distance(),
        ]
    }
}

/// Camera constructor state corresponding to the Reference's public fields.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraConfig {
    /// Output `(width, height)`.
    pub resolution: (u32, u32),
    /// Frame rate.
    pub fps: u32,
    /// Linear-light background.
    pub background: LinearRgba,
    /// Maximum accepted scene-space point norm.
    pub max_allowable_norm: f64,
    /// Compatibility MSAA request, mapped to an adaptive edge ceiling.
    pub samples: u8,
    /// Initial movable light position.
    pub light_source_position: Vec3,
    /// Camera-frame configuration.
    pub frame: CameraFrame,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            resolution: (DEFAULT_PIXEL_WIDTH, DEFAULT_PIXEL_HEIGHT),
            fps: DEFAULT_FPS,
            background: Srgb::from_rgb8(0, 0, 0).to_linear(1.0),
            max_allowable_norm: FRAME_WIDTH,
            samples: 0,
            light_source_position: DEFAULT_LIGHT_POSITION,
            frame: CameraFrame::default(),
        }
    }
}

/// Maximum adaptive edge grid implied by Camera's compatibility `samples`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSampleLimit {
    /// Native analytic coverage only.
    Native,
    /// Up to 2×2 on complex edge cells.
    TwoByTwo,
    /// Up to 4×4 on complex edge cells.
    FourByFour,
}

/// Capture camera plus mutable light state.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    frame: CameraFrame,
    resolution: (u32, u32),
    fps: u32,
    background: LinearRgba,
    max_allowable_norm: f64,
    samples: u8,
    light_source_position: Vec3,
    revision: u64,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(CameraConfig::default()).expect("default camera config is valid")
    }
}

impl Camera {
    /// Construct a camera and resize its frame to the pixel aspect ratio.
    pub fn new(config: CameraConfig) -> Result<Self, CameraError> {
        let (width, height) = config.resolution;
        if width == 0 || height == 0 {
            return Err(CameraError::ZeroDimension);
        }
        if config.fps == 0 {
            return Err(CameraError::ZeroFrameRate);
        }
        if !config.max_allowable_norm.is_finite()
            || !finite3(config.light_source_position)
            || ![
                config.background.r,
                config.background.g,
                config.background.b,
                config.background.a,
            ]
            .iter()
            .all(|component| component.is_finite())
        {
            return Err(CameraError::NonFinite);
        }
        let mut frame = config.frame;
        frame.resize_to_aspect_ratio(f64::from(width) / f64::from(height), false)?;
        Ok(Self {
            frame,
            resolution: config.resolution,
            fps: config.fps,
            background: config.background,
            max_allowable_norm: config.max_allowable_norm,
            samples: config.samples,
            light_source_position: config.light_source_position,
            revision: 1,
        })
    }

    /// Monotone camera-state revision, including frame and light changes.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Immutable camera frame.
    #[must_use]
    pub const fn frame(&self) -> &CameraFrame {
        &self.frame
    }

    /// Mutable camera frame.
    ///
    /// Taking this borrow invalidates camera-derived artifacts up front. That
    /// preserves one monotone camera revision without trying to compress two
    /// independent counters with a collision-prone XOR.
    pub fn frame_mut(&mut self) -> &mut CameraFrame {
        self.revision = self.revision.wrapping_add(1);
        &mut self.frame
    }

    /// Output pixel shape.
    #[must_use]
    pub const fn pixel_shape(&self) -> (u32, u32) {
        self.resolution
    }

    /// Change the output pixel shape.
    ///
    /// The internal frame follows the new pixel aspect ratio the same way
    /// construction does, so camera-derived artifacts stay consistent.
    pub fn set_pixel_shape(&mut self, width: u32, height: u32) -> Result<(), CameraError> {
        if width == 0 || height == 0 {
            return Err(CameraError::ZeroDimension);
        }
        self.frame
            .resize_to_aspect_ratio(f64::from(width) / f64::from(height), false)?;
        self.resolution = (width, height);
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// Output width.
    #[must_use]
    pub const fn pixel_width(&self) -> u32 {
        self.resolution.0
    }

    /// Output height.
    #[must_use]
    pub const fn pixel_height(&self) -> u32 {
        self.resolution.1
    }

    /// Width divided by height.
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        f64::from(self.pixel_width()) / f64::from(self.pixel_height())
    }

    /// Frames per second.
    #[must_use]
    pub const fn fps(&self) -> u32 {
        self.fps
    }

    /// Linear-light background.
    #[must_use]
    pub const fn background(&self) -> LinearRgba {
        self.background
    }

    /// Change the linear-light background for subsequent captures.
    ///
    /// # Errors
    /// Rejects non-finite components without changing the camera.
    pub fn set_background(&mut self, background: LinearRgba) -> Result<(), CameraError> {
        if ![background.r, background.g, background.b, background.a]
            .iter()
            .all(|component| component.is_finite())
        {
            return Err(CameraError::NonFinite);
        }
        if self.background != background {
            self.background = background;
            self.revision = self.revision.wrapping_add(1);
        }
        Ok(())
    }

    /// Reference point-norm guard.
    #[must_use]
    pub const fn max_allowable_norm(&self) -> f64 {
        self.max_allowable_norm
    }

    /// Compatibility sample request.
    #[must_use]
    pub const fn samples(&self) -> u8 {
        self.samples
    }

    /// Samples are a quality hint, never a semantic full-frame-MSAA switch.
    #[must_use]
    pub const fn aa_policy(&self) -> AaPolicy {
        AaPolicy::Adaptive
    }

    /// Adaptive edge ceiling derived from the compatibility sample count.
    #[must_use]
    pub const fn edge_sample_limit(&self) -> EdgeSampleLimit {
        match self.samples {
            0 | 1 => EdgeSampleLimit::Native,
            2 | 3 => EdgeSampleLimit::TwoByTwo,
            _ => EdgeSampleLimit::FourByFour,
        }
    }

    /// World-space light position.
    #[must_use]
    pub const fn light_source_position(&self) -> Vec3 {
        self.light_source_position
    }

    /// Move the light; the default is a state value, not a shader hardcode.
    pub fn set_light_source_position(&mut self, position: Vec3) -> Result<&mut Self, CameraError> {
        if !finite3(position) {
            return Err(CameraError::NonFinite);
        }
        self.light_source_position = position;
        self.revision = self.revision.wrapping_add(1);
        Ok(self)
    }

    /// Scene units per output pixel at the frame center.
    #[must_use]
    pub fn pixel_size(&self) -> f64 {
        self.frame.width() / f64::from(self.pixel_width())
    }

    /// Camera position consumed by lighting.
    #[must_use]
    pub fn location(&self) -> Vec3 {
        self.frame.implied_camera_location()
    }

    /// Apply the kept camera projection to one world-space point.
    #[must_use]
    pub fn project(&self, point: Vec3, is_fixed_in_frame: f64) -> ClipPoint {
        let view = self.frame.to_fixed_frame_point(point, false);
        let fixed = is_fixed_in_frame;
        let mut mixed = [
            view[0] + fixed * (point[0] - view[0]),
            view[1] + fixed * (point[1] - view[1]),
            view[2] + fixed * (point[2] - view[2]),
        ];
        let factors = self.frame.frame_rescale_factors();
        for (component, factor) in mixed.iter_mut().zip(factors) {
            *component *= factor;
        }
        ClipPoint {
            world: point,
            clip: [mixed[0], mixed[1], -0.1 * mixed[2], 1.0 - mixed[2]],
        }
    }

    /// Project and exactly clip one world-space quadratic before perspective
    /// division.
    ///
    /// User planes are evaluated in world space, then the six camera-volume
    /// halfspaces in homogeneous clip space. Every distance is itself a
    /// quadratic in the curve parameter, so its real roots split the curve
    /// exactly with de Casteljau. The returned spans retain matching world and
    /// clip controls plus their range in the source segment; fill, stroke and
    /// depth therefore consume one camera derivation without rebuilding the
    /// authoritative object-space [`crate::table::Segment`].
    pub fn project_quadratic(
        &self,
        world: [Vec3; 3],
        is_fixed_in_frame: f64,
        user_planes: [[f64; 4]; 4],
    ) -> Result<Vec<ClippedQuadratic>, CameraError> {
        self.project_quadratic_with_limit(world, is_fixed_in_frame, user_planes, usize::MAX)
    }

    /// [`Camera::project_quadratic`] with an inclusive intermediate-piece
    /// ceiling for aggregate 3D preparation.
    pub(crate) fn project_quadratic_with_limit(
        &self,
        world: [Vec3; 3],
        is_fixed_in_frame: f64,
        user_planes: [[f64; 4]; 4],
        max_pieces: usize,
    ) -> Result<Vec<ClippedQuadratic>, CameraError> {
        if !is_fixed_in_frame.is_finite()
            || world.iter().any(|point| !finite3(*point))
            || user_planes
                .iter()
                .flatten()
                .any(|component| !component.is_finite())
        {
            return Err(CameraError::NonFinite);
        }
        let mut pieces = Vec::new();
        pieces
            .try_reserve_exact(1)
            .map_err(|_| CameraError::AllocationFailed {
                resource: "projected quadratic clipping",
                requested: 1,
            })?;
        let clip = world.map(|point| self.project(point, is_fixed_in_frame).clip);
        pieces.push(ClippedQuadratic {
            world,
            clip,
            source_t: [0.0, 1.0],
        });
        for plane in user_planes {
            if plane[..3].iter().all(|component| *component == 0.0) {
                continue;
            }
            pieces = clip_quadratics(
                pieces,
                |piece| {
                    piece.world.map(|point| {
                        point[0] * plane[0] + point[1] * plane[1] + point[2] * plane[2] + plane[3]
                    })
                },
                max_pieces,
            )?;
        }
        for plane in 0..6 {
            pieces = clip_quadratics(
                pieces,
                |piece| {
                    piece.clip.map(|point| {
                        let [x, y, z, w] = point;
                        match plane {
                            0 => x + w,
                            1 => w - x,
                            2 => y + w,
                            3 => w - y,
                            4 => z + w,
                            _ => w - z,
                        }
                    })
                },
                max_pieces,
            )?;
        }
        Ok(pieces)
    }

    /// Project and clip one closed fill contour before perspective division.
    ///
    /// Unlike [`Camera::project_quadratic`], this clips the contour as a
    /// *region boundary*. After each halfspace, exits are joined to the next
    /// entry along that halfspace before the following plane is applied. This
    /// is the curved Sutherland-Hodgman construction: a contour which encloses
    /// the whole viewport remains a viewport-sized contour even when none of
    /// its original curves is visible, and intersections at clip-volume
    /// corners follow both boundary planes instead of an interior chord.
    ///
    /// The generated boundary pieces belong to the fill only. Stroke
    /// compilation continues to use [`Camera::project_quadratic`] so clipping
    /// never invents a stroked edge.
    pub(crate) fn project_fill_contour(
        &self,
        world: &[[Vec3; 3]],
        is_fixed_in_frame: f64,
        user_planes: [[f64; 4]; 4],
        max_pieces: usize,
    ) -> Result<Vec<ClippedFillQuadratic>, CameraError> {
        if !is_fixed_in_frame.is_finite()
            || world.iter().flatten().any(|point| !finite3(*point))
            || user_planes
                .iter()
                .flatten()
                .any(|component| !component.is_finite())
        {
            return Err(CameraError::NonFinite);
        }
        let Some(first) = world.first() else {
            return Ok(Vec::new());
        };
        let start = first[0];
        let end = world.last().map_or(first[2], |last| last[2]);
        let closes_contour = !point3_close(end, start);
        let requested = world.len().checked_add(usize::from(closes_contour)).ok_or(
            CameraError::ClipLimitExceeded {
                resource: "fill-contour clipping",
                limit: max_pieces,
                requested: usize::MAX,
            },
        )?;
        if requested > max_pieces {
            return Err(CameraError::ClipLimitExceeded {
                resource: "fill-contour clipping",
                limit: max_pieces,
                requested,
            });
        }
        let mut contour = Vec::new();
        contour
            .try_reserve_exact(requested)
            .map_err(|_| CameraError::AllocationFailed {
                resource: "fill-contour clipping",
                requested,
            })?;
        for control in world {
            contour.push(ClippedFillQuadratic {
                world: *control,
                clip: control.map(|point| self.project(point, is_fixed_in_frame).clip),
            });
        }
        if closes_contour {
            contour.push(fill_boundary_line(
                end,
                self.project(end, is_fixed_in_frame).clip,
                start,
                self.project(start, is_fixed_in_frame).clip,
            ));
        }
        for plane in user_planes {
            if plane[..3].iter().all(|component| *component == 0.0) {
                continue;
            }
            contour = clip_fill_contour(
                contour,
                |piece| {
                    piece.world.map(|point| {
                        point[0] * plane[0] + point[1] * plane[1] + point[2] * plane[2] + plane[3]
                    })
                },
                max_pieces,
            )?;
        }
        for plane in 0..6 {
            contour = clip_fill_contour(
                contour,
                |piece| {
                    piece.clip.map(|point| {
                        let [x, y, z, w] = point;
                        match plane {
                            0 => x + w,
                            1 => w - x,
                            2 => y + w,
                            3 => w - y,
                            4 => z + w,
                            _ => w - z,
                        }
                    })
                },
                max_pieces,
            )?;
        }
        Ok(contour)
    }
}

/// ThreeDCamera is Camera with the Reference's four-sample constructor default.
#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDCamera(Camera);

impl Default for ThreeDCamera {
    fn default() -> Self {
        let config = CameraConfig {
            samples: THREE_D_CAMERA_SAMPLES,
            ..CameraConfig::default()
        };
        Self::new(config).expect("default 3D camera config is valid")
    }
}

impl ThreeDCamera {
    /// Construct with explicit capture configuration.
    ///
    /// Unlike [`ThreeDCamera::default`], this preserves `samples = 0`, which
    /// is the Reference's supported way to disable multisampling explicitly.
    pub fn new(config: CameraConfig) -> Result<Self, CameraError> {
        Camera::new(config).map(Self)
    }

    /// Borrow the shared Camera surface.
    #[must_use]
    pub const fn camera(&self) -> &Camera {
        &self.0
    }

    /// Mutably borrow the shared Camera surface.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.0
    }

    /// Consume the wrapper.
    #[must_use]
    pub fn into_camera(self) -> Camera {
        self.0
    }
}

impl std::ops::Deref for ThreeDCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ThreeDCamera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Homogeneous clip-space output from [`Camera::project`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipPoint {
    /// Original world point; user clip planes are evaluated here.
    pub world: Vec3,
    /// `(x, y, z, w)` after the kept projection constants.
    pub clip: [f64; 4],
}

/// One exact visible span of a camera-projected quadratic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippedQuadratic {
    /// World-space Bézier controls after exact clipping.
    pub world: [Vec3; 3],
    /// Homogeneous `(x, y, z, w)` controls under [`Camera::project`].
    pub clip: [[f64; 4]; 3],
    /// Parameter range in the source object-space segment.
    pub source_t: [f64; 2],
}

impl ClippedQuadratic {
    /// Homogeneous output-pixel controls `(X, Y, W)` for
    /// [`crate::fill::RationalPiece`].
    #[must_use]
    pub fn screen_controls(self, resolution: (u32, u32)) -> [[f64; 3]; 3] {
        screen_controls(self.clip, resolution)
    }
}

/// One curve of a closed fill contour after region clipping.
///
/// This is crate-private because synthetic clip-boundary pieces intentionally
/// have no source-segment parameter range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ClippedFillQuadratic {
    pub(crate) world: [Vec3; 3],
    pub(crate) clip: [[f64; 4]; 3],
}

impl ClippedFillQuadratic {
    /// Homogeneous output-pixel controls `(X, Y, W)`.
    pub(crate) fn screen_controls(self, resolution: (u32, u32)) -> [[f64; 3]; 3] {
        screen_controls(self.clip, resolution)
    }
}

fn clip_quadratics(
    input: Vec<ClippedQuadratic>,
    distances: impl Fn(&ClippedQuadratic) -> [f64; 3],
    max_pieces: usize,
) -> Result<Vec<ClippedQuadratic>, CameraError> {
    let mut output = Vec::new();
    for piece in input {
        let distance = distances(&piece);
        let (roots, root_count) = roots_in_unit_interval(distance);
        let mut cuts = [0.0; 4];
        cuts[0] = 0.0;
        cuts[1..1 + root_count].copy_from_slice(&roots[..root_count]);
        cuts[1 + root_count] = 1.0;
        let scale = distance
            .iter()
            .fold(1.0f64, |largest, value| largest.max(value.abs()));
        let tolerance = 64.0 * f64::EPSILON * scale;
        for pair in cuts[..root_count + 2].windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b - a <= 64.0 * f64::EPSILON {
                continue;
            }
            let midpoint = 0.5 * (a + b);
            if bernstein_scalar(distance, midpoint) >= -tolerance {
                push_clipped_piece(
                    &mut output,
                    quadratic_span(piece, a, b),
                    "projected quadratic clipping",
                    max_pieces,
                )?;
            }
        }
    }
    Ok(output)
}

fn clip_fill_contour(
    input: Vec<ClippedFillQuadratic>,
    distances: impl Fn(&ClippedFillQuadratic) -> [f64; 3],
    max_pieces: usize,
) -> Result<Vec<ClippedFillQuadratic>, CameraError> {
    let mut visible = Vec::new();
    for piece in input {
        let distance = distances(&piece);
        let (roots, root_count) = roots_in_unit_interval(distance);
        let mut cuts = [0.0; 4];
        cuts[0] = 0.0;
        cuts[1..1 + root_count].copy_from_slice(&roots[..root_count]);
        cuts[1 + root_count] = 1.0;
        let scale = distance
            .iter()
            .fold(1.0f64, |largest, value| largest.max(value.abs()));
        let tolerance = 64.0 * f64::EPSILON * scale;
        for pair in cuts[..root_count + 2].windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if b - a <= 64.0 * f64::EPSILON {
                continue;
            }
            let midpoint = 0.5 * (a + b);
            if bernstein_scalar(distance, midpoint) >= -tolerance {
                push_clipped_piece(
                    &mut visible,
                    fill_quadratic_span(piece, a, b),
                    "fill-contour clipping",
                    max_pieces,
                )?;
            }
        }
    }
    if visible.is_empty() {
        return Ok(visible);
    }

    let capacity = visible
        .len()
        .checked_mul(2)
        .ok_or(CameraError::ClipLimitExceeded {
            resource: "closed fill-contour clipping",
            limit: max_pieces,
            requested: usize::MAX,
        })?;
    let mut closed = Vec::new();
    closed
        .try_reserve_exact(capacity.min(max_pieces))
        .map_err(|_| CameraError::AllocationFailed {
            resource: "closed fill-contour clipping",
            requested: capacity.min(max_pieces),
        })?;
    for index in 0..visible.len() {
        let current = visible[index];
        let next = visible[(index + 1) % visible.len()];
        push_clipped_piece(
            &mut closed,
            current,
            "closed fill-contour clipping",
            max_pieces,
        )?;
        if !point4_close(current.clip[2], next.clip[0]) {
            push_clipped_piece(
                &mut closed,
                fill_boundary_line(
                    current.world[2],
                    current.clip[2],
                    next.world[0],
                    next.clip[0],
                ),
                "closed fill-contour clipping",
                max_pieces,
            )?;
        }
    }
    Ok(closed)
}

fn push_clipped_piece<T>(
    output: &mut Vec<T>,
    value: T,
    resource: &'static str,
    limit: usize,
) -> Result<(), CameraError> {
    let requested = output
        .len()
        .checked_add(1)
        .ok_or(CameraError::ClipLimitExceeded {
            resource,
            limit,
            requested: usize::MAX,
        })?;
    if requested > limit {
        return Err(CameraError::ClipLimitExceeded {
            resource,
            limit,
            requested,
        });
    }
    output
        .try_reserve(1)
        .map_err(|_| CameraError::AllocationFailed {
            resource,
            requested,
        })?;
    output.push(value);
    Ok(())
}

fn fill_boundary_line(
    world_start: Vec3,
    clip_start: [f64; 4],
    world_end: Vec3,
    clip_end: [f64; 4],
) -> ClippedFillQuadratic {
    ClippedFillQuadratic {
        world: [world_start, midpoint3(world_start, world_end), world_end],
        clip: [clip_start, midpoint4(clip_start, clip_end), clip_end],
    }
}

fn point3_close(a: Vec3, b: Vec3) -> bool {
    let scale = a
        .iter()
        .chain(&b)
        .fold(1.0f64, |largest, value| largest.max(value.abs()));
    a.iter()
        .zip(b)
        .all(|(left, right)| (left - right).abs() <= 128.0 * f64::EPSILON * scale)
}

fn point4_close(a: [f64; 4], b: [f64; 4]) -> bool {
    let scale = a
        .iter()
        .chain(&b)
        .fold(1.0f64, |largest, value| largest.max(value.abs()));
    a.iter()
        .zip(b)
        .all(|(left, right)| (left - right).abs() <= 128.0 * f64::EPSILON * scale)
}

fn midpoint3(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|axis| 0.5 * (a[axis] + b[axis]))
}

fn midpoint4(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    std::array::from_fn(|axis| 0.5 * (a[axis] + b[axis]))
}

fn bernstein_scalar(control: [f64; 3], t: f64) -> f64 {
    let u = 1.0 - t;
    u * u * control[0] + 2.0 * u * t * control[1] + t * t * control[2]
}

fn roots_in_unit_interval(control: [f64; 3]) -> ([f64; 2], usize) {
    let [p0, p1, p2] = control;
    let a = p0 - 2.0 * p1 + p2;
    let b = 2.0 * (p1 - p0);
    let c = p0;
    let scale = a.abs().max(b.abs()).max(c.abs()).max(1.0);
    let tolerance = 64.0 * f64::EPSILON * scale;
    let mut roots = [0.0; 2];
    let mut count = 0;
    if a.abs() <= tolerance {
        if b.abs() > tolerance {
            let root = -c / b;
            if root > 0.0 && root < 1.0 {
                roots[count] = root;
                count += 1;
            }
        }
        return (roots, count);
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < -tolerance * scale {
        return (roots, count);
    }
    let root_discriminant = discriminant.max(0.0).sqrt();
    let q = -0.5
        * if b >= 0.0 {
            b + root_discriminant
        } else {
            b - root_discriminant
        };
    if q == 0.0 {
        let root = -b / (2.0 * a);
        if root > 0.0 && root < 1.0 {
            roots[count] = root;
            count += 1;
        }
        return (roots, count);
    }
    for root in [q / a, c / q] {
        if root > 0.0 && root < 1.0 {
            roots[count] = root;
            count += 1;
        }
    }
    roots[..count].sort_by(f64::total_cmp);
    if count == 2 && (roots[0] - roots[1]).abs() <= 64.0 * f64::EPSILON {
        count = 1;
    }
    (roots, count)
}

fn quadratic_span(piece: ClippedQuadratic, a: f64, b: f64) -> ClippedQuadratic {
    if a == 0.0 && b == 1.0 {
        return piece;
    }
    let left = if b < 1.0 {
        split_quadratic(piece, b).0
    } else {
        piece
    };
    if a == 0.0 {
        return left;
    }
    split_quadratic(left, a / b).1
}

fn fill_quadratic_span(piece: ClippedFillQuadratic, a: f64, b: f64) -> ClippedFillQuadratic {
    if a == 0.0 && b == 1.0 {
        return piece;
    }
    let left = if b < 1.0 {
        split_fill_quadratic(piece, b).0
    } else {
        piece
    };
    if a == 0.0 {
        return left;
    }
    split_fill_quadratic(left, a / b).1
}

fn split_fill_quadratic(
    piece: ClippedFillQuadratic,
    t: f64,
) -> (ClippedFillQuadratic, ClippedFillQuadratic) {
    let (world_left, world_right) = split3(piece.world, t);
    let (clip_left, clip_right) = split4(piece.clip, t);
    (
        ClippedFillQuadratic {
            world: world_left,
            clip: clip_left,
        },
        ClippedFillQuadratic {
            world: world_right,
            clip: clip_right,
        },
    )
}

fn split_quadratic(piece: ClippedQuadratic, t: f64) -> (ClippedQuadratic, ClippedQuadratic) {
    let (world_left, world_right) = split3(piece.world, t);
    let (clip_left, clip_right) = split4(piece.clip, t);
    let middle = piece.source_t[0] + (piece.source_t[1] - piece.source_t[0]) * t;
    (
        ClippedQuadratic {
            world: world_left,
            clip: clip_left,
            source_t: [piece.source_t[0], middle],
        },
        ClippedQuadratic {
            world: world_right,
            clip: clip_right,
            source_t: [middle, piece.source_t[1]],
        },
    )
}

fn split3(control: [Vec3; 3], t: f64) -> ([Vec3; 3], [Vec3; 3]) {
    let lerp = |a: Vec3, b: Vec3| {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };
    let q0 = lerp(control[0], control[1]);
    let q1 = lerp(control[1], control[2]);
    let r = lerp(q0, q1);
    ([control[0], q0, r], [r, q1, control[2]])
}

fn split4(control: [[f64; 4]; 3], t: f64) -> ([[f64; 4]; 3], [[f64; 4]; 3]) {
    let lerp = |a: [f64; 4], b: [f64; 4]| {
        std::array::from_fn(|index| a[index] + (b[index] - a[index]) * t)
    };
    let q0 = lerp(control[0], control[1]);
    let q1 = lerp(control[1], control[2]);
    let r = lerp(q0, q1);
    ([control[0], q0, r], [r, q1, control[2]])
}

fn screen_controls(clip: [[f64; 4]; 3], resolution: (u32, u32)) -> [[f64; 3]; 3] {
    let width = 0.5 * f64::from(resolution.0);
    let height = 0.5 * f64::from(resolution.1);
    clip.map(|[x, y, _z, w]| {
        // x_px = width * (x / w + 1)
        // y_px = height * (1 - y / w)
        [width * (x + w), height * (w - y), w]
    })
}

impl ClipPoint {
    /// Perspective-divided NDC coordinates.
    #[must_use]
    pub fn ndc(self) -> Option<Vec3> {
        let w = self.clip[3];
        if w == 0.0 || !w.is_finite() {
            return None;
        }
        Some([self.clip[0] / w, self.clip[1] / w, self.clip[2] / w])
    }

    /// Output-oriented pixel coordinates and OpenGL-style `[0, 1]` depth.
    #[must_use]
    pub fn pixel(self, resolution: (u32, u32)) -> Option<[f64; 3]> {
        let ndc = self.ndc()?;
        Some([
            0.5 * (ndc[0] + 1.0) * f64::from(resolution.0),
            0.5 * (1.0 - ndc[1]) * f64::from(resolution.1),
            0.5 * (ndc[2] + 1.0),
        ])
    }

    /// Whether this point lies inside the six homogeneous clip-volume planes.
    #[must_use]
    pub fn inside_clip_volume(self) -> bool {
        let [x, y, z, w] = self.clip;
        w.is_finite() && x >= -w && x <= w && y >= -w && y <= w && z >= -w && z <= w
    }

    /// Signed distance to one user plane `[a, b, c, d]`.
    ///
    /// A zero normal disables the slot exactly as the Reference shader does.
    #[must_use]
    pub fn user_clip_distance(self, plane: [f64; 4]) -> Option<f64> {
        if plane[..3].iter().all(|component| *component == 0.0) {
            return None;
        }
        Some(
            self.world[0] * plane[0]
                + self.world[1] * plane[1]
                + self.world[2] * plane[2]
                + plane[3],
        )
    }

    /// True when all four active user clip planes keep this point.
    #[must_use]
    pub fn inside_user_clip_planes(self, planes: [[f64; 4]; 4]) -> bool {
        planes.into_iter().all(|plane| {
            self.user_clip_distance(plane)
                .is_none_or(|distance| distance >= 0.0)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::DEG;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 2e-12
    }

    #[test]
    fn default_camera_matches_reference_constructor_fields() {
        let camera = Camera::default();
        assert_eq!(
            camera.pixel_shape(),
            (DEFAULT_PIXEL_WIDTH, DEFAULT_PIXEL_HEIGHT)
        );
        assert_eq!(camera.fps(), 30);
        assert_eq!(camera.samples(), 0);
        assert_eq!(camera.edge_sample_limit(), EdgeSampleLimit::Native);
        assert_eq!(camera.light_source_position(), [-10.0, 10.0, 10.0]);
        assert_eq!(camera.frame().shape(), [FRAME_WIDTH, FRAME_HEIGHT]);
        assert_eq!(camera.frame().center(), [0.0; 3]);
        assert_eq!(camera.frame().orientation(), IDENTITY_QUAT);
        assert_eq!(camera.frame().euler_axes(), "zxz");
        assert_eq!(CAMERA_FRAME_Z_INDEX, -1);
        assert!(close(camera.frame().field_of_view(), 45.0 * DEG));
        assert!(close(camera.pixel_size(), FRAME_WIDTH / 1920.0));
    }

    #[test]
    fn camera_config_preserves_capture_fields_and_resizes_the_frame_aspect() {
        let background = Srgb::from_rgb8(12, 34, 56).to_linear(0.25);
        let camera = Camera::new(CameraConfig {
            resolution: (800, 600),
            fps: 60,
            background,
            max_allowable_norm: 42.0,
            samples: 3,
            light_source_position: [1.0, 2.0, 3.0],
            frame: CameraFrame::default(),
        })
        .expect("valid camera configuration");
        assert_eq!(camera.pixel_shape(), (800, 600));
        assert_eq!(camera.fps(), 60);
        assert_eq!(camera.background(), background);
        assert_eq!(camera.max_allowable_norm(), 42.0);
        assert_eq!(camera.samples(), 3);
        assert_eq!(camera.edge_sample_limit(), EdgeSampleLimit::TwoByTwo);
        assert_eq!(camera.light_source_position(), [1.0, 2.0, 3.0]);
        assert!(close(camera.frame().width(), FRAME_WIDTH));
        assert!(close(camera.frame().height(), FRAME_WIDTH * 3.0 / 4.0));
        assert!(close(camera.frame().aspect_ratio(), 4.0 / 3.0));
    }

    #[test]
    fn set_pixel_shape_updates_resolution_frame_aspect_and_revision() {
        let mut camera = Camera::default();
        assert_eq!(camera.pixel_shape(), (1920, 1080));
        let revision = camera.revision();

        camera
            .set_pixel_shape(640, 480)
            .expect("nonzero pixel shape");

        assert_eq!(camera.pixel_width(), 640);
        assert_eq!(camera.pixel_height(), 480);
        assert!(close(camera.frame().width(), FRAME_WIDTH));
        assert!(close(camera.frame().height(), FRAME_WIDTH * 3.0 / 4.0));
        assert!(close(camera.frame().aspect_ratio(), 4.0 / 3.0));
        assert!(close(
            camera.frame().width() / camera.frame().height(),
            f64::from(camera.pixel_width()) / f64::from(camera.pixel_height()),
        ));
        assert!(camera.revision() > revision);
    }

    #[test]
    fn set_pixel_shape_refuses_zero_dimensions() {
        let mut camera = Camera::default();
        assert_eq!(
            camera.set_pixel_shape(0, 480),
            Err(CameraError::ZeroDimension)
        );
        assert_eq!(
            camera.set_pixel_shape(640, 0),
            Err(CameraError::ZeroDimension)
        );
    }

    #[test]
    fn invalid_camera_configuration_is_named() {
        assert_eq!(
            Camera::new(CameraConfig {
                resolution: (0, 1080),
                ..CameraConfig::default()
            }),
            Err(CameraError::ZeroDimension)
        );
        assert_eq!(
            Camera::new(CameraConfig {
                fps: 0,
                ..CameraConfig::default()
            }),
            Err(CameraError::ZeroFrameRate)
        );
        assert_eq!(
            Camera::new(CameraConfig {
                light_source_position: [f64::NAN, 0.0, 0.0],
                ..CameraConfig::default()
            }),
            Err(CameraError::NonFinite)
        );

        let mut frame = CameraFrame::default();
        assert_eq!(
            frame.set_field_of_view(PI),
            Err(CameraError::InvalidFieldOfView)
        );
        assert_eq!(
            frame.set_orientation([0.0; 4]),
            Err(CameraError::InvalidOrientation)
        );
        assert_eq!(
            frame.set_euler_axes("xxz"),
            Err(CameraError::InvalidEulerAxes)
        );
    }

    #[test]
    fn projection_ports_the_reference_constants_in_order() {
        let camera = Camera::default();
        let point = [2.0, -1.0, 3.0];
        let projected = camera.project(point, 0.0);
        let scale = camera.frame().scale();
        let focal = camera.frame().focal_distance();
        let z = point[2] * scale / focal;
        assert!(close(projected.clip[0], 2.0 * point[0] / FRAME_WIDTH));
        assert!(close(projected.clip[1], 2.0 * point[1] / FRAME_HEIGHT));
        assert!(close(projected.clip[2], -0.1 * z));
        assert!(close(projected.clip[3], 1.0 - z));

        let fixed = camera.project(point, 1.0);
        assert_eq!(
            fixed, projected,
            "identity frame makes the mix endpoints equal"
        );

        let mut moved = camera.clone();
        moved
            .frame_mut()
            .set_center([5.0, 2.0, -1.0])
            .expect("finite");
        let world = moved.project(point, 0.0);
        let frame = moved.project(point, 1.0);
        let middle = moved.project(point, 0.25);
        for axis in 0..3 {
            assert!(close(
                middle.clip[axis],
                world.clip[axis] + 0.25 * (frame.clip[axis] - world.clip[axis])
            ));
        }
    }

    #[test]
    fn clip_planes_use_original_world_points_and_zero_normals_disable() {
        let point = Camera::default().project([2.0, -3.0, 4.0], 1.0);
        assert_eq!(point.user_clip_distance([0.0; 4]), None);
        assert_eq!(point.user_clip_distance([1.0, 0.0, 0.0, -1.5]), Some(0.5));
        assert!(point.inside_user_clip_planes([
            [1.0, 0.0, 0.0, -1.5],
            [0.0, -1.0, 0.0, -2.5],
            [0.0; 4],
            [0.0; 4],
        ]));
        assert!(!point.inside_user_clip_planes([
            [-1.0, 0.0, 0.0, 1.5],
            [0.0; 4],
            [0.0; 4],
            [0.0; 4],
        ]));
    }

    fn curve_point(control: [Vec3; 3], t: f64) -> Vec3 {
        let u = 1.0 - t;
        std::array::from_fn(|axis| {
            u * u * control[0][axis] + 2.0 * u * t * control[1][axis] + t * t * control[2][axis]
        })
    }

    #[test]
    fn quadratic_user_clipping_is_exact_and_keeps_source_parameters() {
        let camera = Camera::default();
        let world = [[-2.0, 0.0, 0.0], [0.0, 0.5, 0.0], [2.0, 0.0, 0.0]];
        let pieces = camera
            .project_quadratic(
                world,
                0.0,
                [[1.0, 0.0, 0.0, 0.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            )
            .expect("finite curve");
        assert_eq!(pieces.len(), 1);
        assert!(close(pieces[0].source_t[0], 0.5));
        assert_eq!(pieces[0].source_t[1], 1.0);
        assert!(close(pieces[0].world[0][0], 0.0));

        // Dense evaluation is only an oracle here; clipping itself solved the
        // quadratic root and split de Casteljau exactly.
        for sample in 0..=256 {
            let t = f64::from(sample) / 256.0;
            let source = curve_point(world, t);
            let retained = pieces
                .iter()
                .any(|piece| t >= piece.source_t[0] && t <= piece.source_t[1]);
            assert_eq!(retained, source[0] >= -1e-13, "t={t}");
        }
    }

    #[test]
    fn camera_near_plane_clips_before_the_perspective_divide() {
        let camera = Camera::default();
        let world = [[0.0, 0.0, 0.0], [0.0, 0.0, 8.0], [0.0, 0.0, 16.0]];
        let pieces = camera
            .project_quadratic(world, 0.0, [[0.0; 4]; 4])
            .expect("finite curve");
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].source_t[0], 0.0);
        assert!(pieces[0].source_t[1] > 0.0 && pieces[0].source_t[1] < 1.0);
        for control in pieces[0].clip {
            let [x, y, z, w] = control;
            // A Bézier whose distance controls are nonnegative stays inside
            // every convex homogeneous halfspace.
            assert!(x + w >= -1e-12);
            assert!(w - x >= -1e-12);
            assert!(y + w >= -1e-12);
            assert!(w - y >= -1e-12);
            assert!(z + w >= -1e-12);
            assert!(w - z >= -1e-12);
        }
        let screen = pieces[0].screen_controls(camera.pixel_shape());
        assert!(screen.iter().all(|point| point[2] > 0.0));
    }

    #[test]
    fn user_plane_tangency_does_not_invent_a_visible_interval() {
        let camera = Camera::default();
        // x(t) = -(t - 1/2)^2 touches x=0 once and is otherwise outside.
        let world = [[-0.25, 0.0, 0.0], [0.25, 0.5, 0.0], [-0.25, 1.0, 0.0]];
        let pieces = camera
            .project_quadratic(
                world,
                0.0,
                [[1.0, 0.0, 0.0, 0.0], [0.0; 4], [0.0; 4], [0.0; 4]],
            )
            .expect("finite curve");
        assert!(
            pieces.is_empty(),
            "a zero-measure touch is not a curve span"
        );
    }

    #[test]
    fn homogeneous_screen_controls_match_dense_camera_projection() {
        let camera = Camera::new(CameraConfig {
            resolution: (320, 180),
            ..CameraConfig::default()
        })
        .expect("camera");
        let world = [[-1.0, -0.5, 0.0], [0.25, 1.0, 2.0], [1.5, -0.25, 3.0]];
        let piece = camera
            .project_quadratic(world, 0.0, [[0.0; 4]; 4])
            .expect("finite curve")[0];
        let screen = piece.screen_controls(camera.pixel_shape());
        for sample in 0..=128 {
            let t = f64::from(sample) / 128.0;
            let projected = camera
                .project(curve_point(world, t), 0.0)
                .pixel(camera.pixel_shape())
                .expect("positive weight");
            let u = 1.0 - t;
            let h: [f64; 3] = std::array::from_fn(|axis| {
                u * u * screen[0][axis] + 2.0 * u * t * screen[1][axis] + t * t * screen[2][axis]
            });
            assert!((h[0] / h[2] - projected[0]).abs() <= 2e-11);
            assert!((h[1] / h[2] - projected[1]).abs() <= 2e-11);
        }
    }

    #[test]
    fn view_and_inverse_round_trip_points_and_vectors() {
        let mut frame = CameraFrame::default();
        frame
            .set_center([1.5, -2.0, 0.25])
            .expect("finite")
            .set_height(5.0)
            .expect("positive")
            .set_euler_angles(Some(0.4), Some(1.1), Some(-0.2))
            .expect("valid rotation");
        for relative in [false, true] {
            let point = [2.0, -0.5, 7.0];
            let fixed = frame.to_fixed_frame_point(point, relative);
            let round_trip = frame.from_fixed_frame_point(fixed, relative);
            for axis in 0..3 {
                assert!(close(round_trip[axis], point[axis]));
            }
        }
    }

    #[test]
    fn euler_animation_stays_continuous_at_both_zxz_singularities() {
        let mut frame = CameraFrame::default();
        frame
            .set_euler_angles(Some(0.7), Some(1e-4), Some(-0.2))
            .expect("near north pole");
        let north = frame.euler_angles();
        assert!(north[1].abs() <= 1e-2);
        assert_eq!(north[2], 0.0);

        frame
            .set_euler_angles(Some(0.7), Some(PI - 1e-4), Some(-0.2))
            .expect("near south pole");
        let south = frame.euler_angles();
        assert!((south[1] - PI).abs() <= 1e-2);
        assert_eq!(south[2], 0.0);

        // Repeated ambient increments remain normalized through the lock.
        for _ in 0..128 {
            frame
                .increment_euler_angles(0.01, 0.0, 0.0)
                .expect("finite increment");
            let q = frame.orientation();
            let norm = q.iter().map(|v| v * v).sum::<f64>();
            assert!(close(norm, 1.0));
        }
    }

    #[test]
    fn three_d_samples_are_an_adaptive_quality_ceiling() {
        let camera = ThreeDCamera::default();
        assert_eq!(camera.samples(), 4);
        assert_eq!(camera.aa_policy(), AaPolicy::Adaptive);
        assert_eq!(camera.edge_sample_limit(), EdgeSampleLimit::FourByFour);

        let one = Camera::new(CameraConfig {
            samples: 1,
            ..CameraConfig::default()
        })
        .expect("valid");
        assert_eq!(one.edge_sample_limit(), EdgeSampleLimit::Native);

        let explicit_zero = ThreeDCamera::new(CameraConfig::default()).expect("valid");
        assert_eq!(explicit_zero.samples(), 0);
        assert_eq!(explicit_zero.edge_sample_limit(), EdgeSampleLimit::Native);
    }

    #[test]
    fn moving_the_light_is_revisioned_state() {
        let mut camera = Camera::default();
        let before = camera.revision();
        camera
            .set_light_source_position([3.0, 4.0, 5.0])
            .expect("finite");
        assert_eq!(camera.light_source_position(), [3.0, 4.0, 5.0]);
        assert_ne!(camera.revision(), before);
    }

    #[test]
    fn mutably_borrowing_the_frame_invalidates_camera_artifacts_monotonically() {
        let mut camera = Camera::default();
        let initial = camera.revision();
        camera
            .frame_mut()
            .set_center([1.0, 2.0, 3.0])
            .expect("finite center");
        let moved = camera.revision();
        assert!(moved > initial);

        camera
            .frame_mut()
            .set_field_of_view(60.0 * DEG)
            .expect("valid fov");
        assert!(camera.revision() > moved);
    }
}
