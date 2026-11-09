//! Portable, render-only camera captures. No Euler extraction, quaternion
//! renormalization, callback, or process-local revision enters the wire format.

use super::{Camera, CameraError, CameraFrame};
use fmn_core::color::LinearRgba;
use fmn_hash::SerialError;
use fmn_hash::serial::{Reader, Writer};

/// Exact camera state observed at a frame's capture boundary.
///
/// Pixel dimensions record the authored aspect ratio. Playback at another
/// resolution keeps the frame shape exactly when the aspect ratio is unchanged;
/// otherwise it keeps frame width, like [`Camera::set_pixel_shape`]. Background,
/// adaptive sample policy, clipping norm and lighting belong to the capture.
/// Revisions and authoring-only defaults/Euler conventions are not serialized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSample {
    resolution: (u32, u32),
    center: [f64; 3],
    shape: [f64; 2],
    orientation: [f64; 4],
    fovy: f64,
    light: [f64; 3],
    max_allowable_norm: f64,
    background: [f64; 4],
    samples: u8,
}

/// Canonical camera bytes were malformed or described an invalid camera.
#[derive(Debug)]
pub enum CameraSampleError {
    /// The enclosing canonical reader refused a field.
    Serial(SerialError),
    /// The captured state was not a finite, meaningful camera.
    Camera(CameraError),
}

impl std::fmt::Display for CameraSampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial(error) => error.fmt(f),
            Self::Camera(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for CameraSampleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Serial(error) => error,
            Self::Camera(error) => error,
        })
    }
}

impl From<SerialError> for CameraSampleError {
    fn from(error: SerialError) -> Self {
        Self::Serial(error)
    }
}

impl CameraSample {
    /// Fixed encoded size: two u32 dimensions, eighteen f64s and one u8.
    pub const WIRE_BYTES: usize = 153;

    /// Freeze the actual camera without changing its orientation bits.
    ///
    /// # Errors
    /// Refuses invalid state, including overflow introduced by camera edits.
    pub fn capture(camera: &Camera) -> Result<Self, CameraError> {
        let frame = camera.frame();
        let background = camera.background();
        let sample = Self {
            resolution: camera.pixel_shape(),
            center: frame.center(),
            shape: frame.shape(),
            orientation: frame.orientation(),
            fovy: frame.field_of_view(),
            light: camera.light_source_position(),
            max_allowable_norm: camera.max_allowable_norm(),
            background: [background.r, background.g, background.b, background.a],
            samples: camera.samples(),
        };
        sample.validate()?;
        Ok(sample)
    }

    fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.center
            .into_iter()
            .chain(self.shape)
            .chain(self.orientation)
            .chain([self.fovy])
            .chain(self.light)
            .chain([self.max_allowable_norm])
            .chain(self.background)
    }

    fn validate(&self) -> Result<(), CameraError> {
        if self.values().any(|value| !value.is_finite()) {
            return Err(CameraError::NonFinite);
        }
        if self.resolution.0 == 0
            || self.resolution.1 == 0
            || self.shape[0] <= 0.0
            || self.shape[1] <= 0.0
        {
            return Err(CameraError::ZeroDimension);
        }
        if self.fovy <= 0.0 || self.fovy >= fmn_core::constants::PI {
            return Err(CameraError::InvalidFieldOfView);
        }
        // The public camera setters normalize author input. Persistence must
        // validate that invariant without normalizing AGAIN: normalization is
        // not bit-idempotent. The tolerance admits binary64 normalization error,
        // not arbitrary scaled quaternions or a second orientation convention.
        let q = self.orientation;
        let norm2 = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
        if !norm2.is_finite() || (norm2 - 1.0).abs() > 64.0 * f64::EPSILON {
            return Err(CameraError::InvalidOrientation);
        }
        Ok(())
    }

    /// Recreate this capture for a player viewport and exact timeline FPS.
    ///
    /// `revision` is a caller-owned cache generation, not serialized identity.
    /// Use the original frame index plus one, not completion order or a rebased
    /// output sequence, when dispatching compiled frames across workers.
    ///
    /// # Errors
    /// Refuses a zero viewport/FPS or a non-finite resized frame.
    pub fn camera(
        &self,
        resolution: (u32, u32),
        fps: u32,
        revision: u64,
    ) -> Result<Camera, CameraError> {
        self.validate()?;
        if resolution.0 == 0 || resolution.1 == 0 {
            return Err(CameraError::ZeroDimension);
        }
        if fps == 0 {
            return Err(CameraError::ZeroFrameRate);
        }
        // Private fields are restored only after validation. Using the public
        // set_orientation here would silently perturb an already-unit quaternion.
        let mut frame = CameraFrame {
            center: self.center,
            shape: self.shape,
            orientation: self.orientation,
            fovy: self.fovy,
            revision,
            ..CameraFrame::default()
        };
        if u64::from(resolution.0) * u64::from(self.resolution.1)
            != u64::from(self.resolution.0) * u64::from(resolution.1)
        {
            frame
                .resize_to_aspect_ratio(f64::from(resolution.0) / f64::from(resolution.1), false)?;
        }
        let [r, g, b, a] = self.background;
        Ok(Camera {
            frame,
            resolution,
            fps,
            background: LinearRgba { r, g, b, a },
            max_allowable_norm: self.max_allowable_norm,
            samples: self.samples,
            light_source_position: self.light,
            revision,
        })
    }

    /// Append the fixed, canonical capture fields to an enclosing document.
    /// The enclosing writer retains any budget failure until `finish`.
    pub fn write_to(&self, writer: &mut Writer) {
        writer.put_u32(self.resolution.0).put_u32(self.resolution.1);
        for value in self.values() {
            writer.put_finite_f64(value);
        }
        writer.put_u8(self.samples);
    }

    /// Decode and validate a single fixed-size capture, without allocation.
    ///
    /// # Errors
    /// Refuses noncanonical/nonfinite fields and invalid camera parameters.
    pub fn read_from(reader: &mut Reader<'_>) -> Result<Self, CameraSampleError> {
        let resolution = (reader.get_u32()?, reader.get_u32()?);
        let mut values = [0.0; 18];
        for value in &mut values {
            *value = reader.get_finite_f64()?;
        }
        let sample = Self {
            resolution,
            center: [values[0], values[1], values[2]],
            shape: [values[3], values[4]],
            orientation: [values[5], values[6], values[7], values[8]],
            fovy: values[9],
            light: [values[10], values[11], values[12]],
            max_allowable_norm: values[13],
            background: [values[14], values[15], values[16], values[17]],
            samples: reader.get_u8()?,
        };
        sample.validate().map_err(CameraSampleError::Camera)?;
        Ok(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::super::CameraConfig;
    use super::*;
    use fmn_hash::serial::{Limits, Schema, UnknownPolicy};

    const TEST_SCHEMA: Schema = Schema::new(*b"TEST", 1, 0, 0);

    fn round_trip(sample: &CameraSample) -> CameraSample {
        let mut writer = Writer::new(TEST_SCHEMA);
        sample.write_to(&mut writer);
        let bytes = writer.finish().unwrap();
        assert_eq!(bytes.len(), 56 + CameraSample::WIRE_BYTES);
        let mut reader =
            Reader::open(&bytes, TEST_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict).unwrap();
        let restored = CameraSample::read_from(&mut reader).unwrap();
        reader.finish().unwrap();
        restored
    }

    #[test]
    fn capture_preserves_non_idempotent_quaternion_and_projection_bits() {
        let mut non_idempotent = 0;
        for i in 1..128 {
            let mut camera = Camera::new(CameraConfig {
                resolution: (320, 180),
                ..CameraConfig::default()
            })
            .unwrap();
            camera
                .frame_mut()
                .set_orientation([f64::from(i), 2.3, -4.7, 0.9])
                .unwrap();
            camera.frame_mut().set_center([0.2, -0.3, 1.1]).unwrap();
            camera.set_light_source_position([3.0, 2.0, 7.0]).unwrap();
            let before = camera.frame().orientation();
            let mut normalized_again = camera.frame().clone();
            normalized_again.set_orientation(before).unwrap();
            non_idempotent += usize::from(normalized_again.orientation() != before);
            let sample = CameraSample::capture(&camera).unwrap();
            let restored = round_trip(&sample).camera((320, 180), 30, 91).unwrap();
            assert_eq!(
                restored.frame().orientation().map(f64::to_bits),
                before.map(f64::to_bits)
            );
            for fixed in [0.0, 0.35, 1.0] {
                for point in [[0.3, -0.2, 1.0], [1.0, 2.0, -0.5], [0.0; 3]] {
                    assert_eq!(
                        camera.project(point, fixed).clip.map(f64::to_bits),
                        restored.project(point, fixed).clip.map(f64::to_bits)
                    );
                }
            }
            assert_eq!(restored.revision(), 91);
            assert_eq!(restored.light_source_position(), [3.0, 2.0, 7.0]);
        }
        assert!(
            non_idempotent > 0,
            "the corpus must detect accidental renormalization"
        );
    }

    #[test]
    fn viewport_adaptation_preserves_equal_aspect_bits_and_width() {
        let camera = Camera::new(CameraConfig {
            resolution: (320, 180),
            ..CameraConfig::default()
        })
        .unwrap();
        let sample = round_trip(&CameraSample::capture(&camera).unwrap());
        let same = sample.camera((1280, 720), 60, 1).unwrap();
        assert_eq!(same.frame().shape(), camera.frame().shape());
        let square = sample.camera((200, 200), 60, 2).unwrap();
        assert_eq!(square.frame().width(), camera.frame().width());
        assert_eq!(square.frame().width(), square.frame().height());
        assert!(sample.camera((0, 200), 60, 1).is_err());
        assert!(sample.camera((200, 200), 0, 1).is_err());
    }

    #[test]
    fn canonical_decoder_refuses_invalid_cameras_without_normalizing() {
        let sample = CameraSample::capture(&Camera::default()).unwrap();
        let mut invalid = [sample; 6];
        invalid[0].orientation = [0.0; 4];
        invalid[1].orientation = [2.0, 0.0, 0.0, 0.0];
        invalid[2].resolution.0 = 0;
        invalid[3].shape[1] = -1.0;
        invalid[4].fovy = 0.0;
        invalid[5].fovy = fmn_core::constants::PI;
        for sample in invalid {
            let mut writer = Writer::new(TEST_SCHEMA);
            sample.write_to(&mut writer);
            let bytes = writer.finish().unwrap();
            let mut reader =
                Reader::open(&bytes, TEST_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict).unwrap();
            assert!(matches!(
                CameraSample::read_from(&mut reader),
                Err(CameraSampleError::Camera(_))
            ));
        }
    }
}
