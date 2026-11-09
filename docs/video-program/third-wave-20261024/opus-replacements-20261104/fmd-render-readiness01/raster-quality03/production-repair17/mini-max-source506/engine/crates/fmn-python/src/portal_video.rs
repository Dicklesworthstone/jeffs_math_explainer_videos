//! SceneFileWriter → Reel negotiation. This adapter owns no encoder or process.
//!
//! A transparent camera selects alpha-preserving MOV; the ordinary writer
//! defaults are promoted to RGBA/qtrle. Explicit incompatible choices fail
//! before opening the encoder or reserving an output generation.

mod quality;

use fmn_frame::convert::{rgba_to_nv12, rgba_to_p010, rgba16f_to_rgba8, swap_rb8};
use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameError, PixelFormat};
use fmn_output::{ColorDescription, Container, EncoderChoice, VideoJob, WireFormat};
use pyo3::prelude::*;

use crate::{CapabilityError, PortalFrameFormat, PyRuntimeError, PyScene, PyValueError};

pub(crate) struct PortalVideoConfig {
    pub(crate) ffmpeg_bin: String,
    pub(crate) job: VideoJob,
}

impl PortalVideoConfig {
    pub(crate) fn from_scene(
        scene: &Bound<'_, PyScene>,
        format: PortalFrameFormat,
        width: u32,
        height: u32,
        fps: u32,
    ) -> PyResult<Self> {
        check_scene_ownership(scene)?;
        let config = Self::parse_scene(scene, format, width, height, fps)?;
        check_scene_ownership(scene)?;
        Ok(config)
    }

    pub(crate) fn from_recording_scene(
        scene: &Bound<'_, PyScene>,
        format: PortalFrameFormat,
        width: u32,
        height: u32,
        fps: u32,
    ) -> PyResult<Self> {
        crate::portal_recording::check_available(scene)?;
        let config = Self::parse_scene(scene, format, width, height, fps)?;
        crate::portal_recording::check_available(scene)?;
        Ok(config)
    }

    fn parse_scene(
        scene: &Bound<'_, PyScene>,
        format: PortalFrameFormat,
        width: u32,
        height: u32,
        fps: u32,
    ) -> PyResult<Self> {
        // No engine borrow may cross a Python descriptor/callback.
        let writer = scene.getattr("file_writer")?;
        let ffmpeg_bin: String = writer.getattr("ffmpeg_bin")?.extract()?;
        let codec: String = writer.getattr("video_codec")?.extract()?;
        let pixel_format: String = writer.getattr("pixel_format")?.extract()?;
        for name in ["saturation", "gamma"] {
            let value: f64 = writer.getattr(name)?.extract()?;
            if value != 1.0 {
                return Err(CapabilityError::new_err(format!(
                    "non-default file_writer {name} requires a native color-transform stage; ffmpeg filters are forbidden"
                )));
            }
        }
        let rgba: [f64; 4] = scene
            .getattr("camera")?
            .getattr("background_rgba")?
            .extract()?;
        if !rgba.iter().all(|component| component.is_finite()) || !(0.0..=1.0).contains(&rgba[3]) {
            return Err(PyValueError::new_err(
                "camera background must be finite with opacity in 0..=1",
            ));
        }
        if ffmpeg_bin.is_empty() || ffmpeg_bin.contains('\0') {
            return Err(PyValueError::new_err(
                "file_writer.ffmpeg_bin must be nonempty and contain no NUL",
            ));
        }
        let quality = quality::from_writer(&writer)?;
        let job = video_job(
            format,
            width,
            height,
            fps,
            &codec,
            &pixel_format,
            rgba[3] < 1.0,
        )
        .map_err(PyValueError::new_err)?
        .with_quality(quality)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
        // Python getters may adopt mobjects, advance time, or recursively
        // acquire a generation. Never replace that state or probe an encoder
        // against the now-stale preflight in begin_portal_render.
        Ok(Self { ffmpeg_bin, job })
    }
}

pub(crate) fn check_scene_ownership(scene: &Bound<'_, PyScene>) -> PyResult<()> {
    let scene = scene.try_borrow()?;
    if scene
        .render
        .lock()
        .map_err(|_| PyRuntimeError::new_err("portal render session lock was poisoned"))?
        .is_some()
    {
        return Err(PyRuntimeError::new_err(
            "a portal render generation is already active",
        ));
    }
    if !scene.proxies.borrow().is_empty()
        || !scene.engine.borrow().stage().roots().is_empty()
        || scene.engine.borrow().stage().time() != 0.0
        || !scene.engine.borrow().sound_requests().is_empty()
    {
        return Err(PyRuntimeError::new_err(
            "render configuration must be installed before Scene construction mutates engine state",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn video_job(
    format: PortalFrameFormat,
    width: u32,
    height: u32,
    fps: u32,
    codec: &str,
    pixel_format: &str,
    transparent: bool,
) -> Result<VideoJob, String> {
    if transparent && format != PortalFrameFormat::Mov {
        return Err("transparent video requires MOV output; select format='mov'".into());
    }
    let wire = match pixel_format.to_ascii_lowercase().as_str() {
        "yuv420p" if transparent => WireFormat::Rgba8,
        "rgba" | "rgba8" => WireFormat::Rgba8,
        "bgra" | "bgra8" => WireFormat::Bgra8,
        "nv12" | "yuv420p" => WireFormat::Nv12,
        "p010" | "p010le" | "yuv420p10le" => WireFormat::P010,
        _ => {
            return Err(format!(
                "unsupported file_writer.pixel_format {pixel_format:?}; choose rgba, bgra, nv12/yuv420p, or p010le"
            ));
        }
    };
    let codec = codec.trim();
    if codec.is_empty()
        || !codec
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err("file_writer.video_codec must be a nonempty encoder name (letters, digits, underscores)".into());
    }
    let encoder = if codec.eq_ignore_ascii_case("auto") || (transparent && codec == "libx264") {
        EncoderChoice::Auto
    } else {
        EncoderChoice::Named(codec.to_owned())
    };
    let job = VideoJob {
        width,
        height,
        fps: (fps, 1),
        wire,
        color: if wire.has_alpha() {
            ColorDescription::srgb_full()
        } else {
            ColorDescription::video_bt709()
        },
        container: if transparent {
            Container::MovTransparent
        } else if format == PortalFrameFormat::Mov {
            Container::Mov
        } else {
            Container::Mp4
        },
        encoder,
        crf: None,
    };
    let encoder = job.resolved_encoder().map_err(|error| error.to_string())?;
    if transparent && encoder.as_deref() != Some("qtrle") {
        return Err("transparent video currently requires the qtrle encoder".into());
    }
    // Both ordinary container paths encode 4:2:0, even for an RGBA input wire.
    if !transparent && (!width.is_multiple_of(2) || !height.is_multiple_of(2)) {
        return Err("ordinary video requires even width and height for 4:2:0 output".into());
    }
    Ok(job)
}

/// Use only the shared native conversion kernels. No ffmpeg filters, channel
/// reinterpretation, changed orientation, or hidden alpha-dropping wire.
pub(crate) fn convert_frame(
    source: &FrameBuffer,
    destination: &mut FrameBuffer,
    scratch: Option<&mut FrameBuffer>,
) -> Result<(), FrameError> {
    let Some(rgba8) = scratch else {
        return rgba16f_to_rgba8(source, destination);
    };
    rgba16f_to_rgba8(source, rgba8)?;
    match destination.layout().format() {
        PixelFormat::Nv12 => {
            rgba_to_nv12(rgba8, destination, ColorRange::Limited, ChromaSiting::Left)
        }
        PixelFormat::P010 => {
            rgba_to_p010(rgba8, destination, ColorRange::Limited, ChromaSiting::Left)
        }
        PixelFormat::Bgra8 => swap_rb8(rgba8, destination),
        _ => Err(FrameError::UnsupportedConversion(
            "unsupported portal output wire",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_video_defaults_and_all_wire_aliases() {
        for (name, expected) in [
            ("rgba", WireFormat::Rgba8),
            ("rgba8", WireFormat::Rgba8),
            ("BGRA", WireFormat::Bgra8),
            ("bgra8", WireFormat::Bgra8),
            ("nv12", WireFormat::Nv12),
            ("yuv420p", WireFormat::Nv12),
            ("p010", WireFormat::P010),
            ("p010le", WireFormat::P010),
            ("yuv420p10le", WireFormat::P010),
        ] {
            let job = video_job(PortalFrameFormat::Mp4, 96, 54, 8, "libx264", name, false).unwrap();
            assert_eq!(job.wire, expected);
            assert_eq!(job.resolved_encoder().unwrap().as_deref(), Some("libx264"));
            assert_eq!(job.container, Container::Mp4);
        }
    }

    #[test]
    fn portal_video_transparent_defaults_and_bgra_preserve_alpha() {
        for name in ["yuv420p", "rgba", "bgra"] {
            let job = video_job(PortalFrameFormat::Mov, 95, 53, 8, "libx264", name, true).unwrap();
            assert!(job.wire.has_alpha());
            assert_eq!(job.container, Container::MovTransparent);
            assert_eq!(job.resolved_encoder().unwrap().as_deref(), Some("qtrle"));
            assert_eq!(job.color, ColorDescription::srgb_full());
        }
    }

    #[test]
    fn portal_video_incompatible_profiles_fail_before_process_creation() {
        for (format, codec, wire, transparent) in [
            (PortalFrameFormat::Mp4, "libx264", "rgba", true),
            (PortalFrameFormat::Mov, "qtrle", "nv12", true),
            (PortalFrameFormat::Mov, "qtrle", "p010", true),
            (PortalFrameFormat::Mov, "libx265", "rgba", true),
            (PortalFrameFormat::Mp4, "libx264", "yuv444p", false),
            (PortalFrameFormat::Mp4, "", "rgba", false),
            (PortalFrameFormat::Mp4, "libx264 -vf eq", "rgba", false),
        ] {
            assert!(video_job(format, 96, 54, 8, codec, wire, transparent).is_err());
        }
        assert!(video_job(PortalFrameFormat::Mp4, 95, 54, 8, "auto", "rgba", false).is_err());
    }

    #[test]
    fn portal_video_callback_ownership_is_fail_closed() {
        crate::with_python_test_module("portal video ownership", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/video_ownership.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("writer callbacks cannot invalidate generation ownership");
        });
    }

    #[test]
    fn portal_video_production_acceptance_suite() {
        crate::with_python_test_module("portal video negotiation", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/video_options.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real decoded negotiated video output");
        });
    }
}
