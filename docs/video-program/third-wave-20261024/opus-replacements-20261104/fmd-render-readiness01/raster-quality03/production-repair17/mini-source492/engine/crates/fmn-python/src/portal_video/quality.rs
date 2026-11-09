//! Python writer controls are converted to Reel's existing typed policy.
//! No argv fragments, alternate encoder defaults, or process wrapper live here.

use fmn_output::negotiate::{EncoderPreset, EncoderTune, VideoQuality};
use pyo3::exceptions::{PyAttributeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBool;

/// Stock and older writers have no quality attributes. Missing/None means
/// no override, preserving the installed encoder's defaults. Do not suppress
/// descriptor failures other than Python's ordinary missing-attribute signal.
fn optional<'py>(writer: &Bound<'py, PyAny>, name: &str) -> PyResult<Option<Bound<'py, PyAny>>> {
    match writer.getattr(name) {
        Ok(value) if value.is_none() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(error) if error.is_instance_of::<PyAttributeError>(writer.py()) => Ok(None),
        Err(error) => Err(error),
    }
}

fn unsigned(
    writer: &Bound<'_, PyAny>,
    name: &str,
    minimum: u32,
    maximum: u32,
) -> PyResult<Option<u32>> {
    let Some(value) = optional(writer, name)? else {
        return Ok(None);
    };
    if value.is_instance_of::<PyBool>() {
        return Err(PyTypeError::new_err(format!(
            "file_writer.{name} must be an integer, not bool"
        )));
    }
    // PyO3's integer extraction honors __index__, not lossy float conversion.
    // Keep user conversion exceptions intact; they are not encoder failures.
    let value = value.extract::<u32>()?;
    if !(minimum..=maximum).contains(&value) {
        return Err(PyValueError::new_err(format!(
            "file_writer.{name} must lie between {minimum} and {maximum}"
        )));
    }
    Ok(Some(value))
}

pub(super) fn from_writer(writer: &Bound<'_, PyAny>) -> PyResult<VideoQuality> {
    let crf = unsigned(writer, "video_crf", 0, 51)?
        .map(u8::try_from)
        .transpose()
        .map_err(|_| PyValueError::new_err("file_writer.video_crf exceeds u8"))?;
    let preset = optional(writer, "video_preset")?
        .map(|value| {
            value
                .extract::<String>()?
                .parse::<EncoderPreset>()
                .map_err(|error| {
                    PyValueError::new_err(format!("file_writer.video_preset: {error}"))
                })
        })
        .transpose()?;
    let tune = optional(writer, "video_tune")?
        .map(|value| {
            value
                .extract::<String>()?
                .parse::<EncoderTune>()
                .map_err(|error| PyValueError::new_err(format!("file_writer.video_tune: {error}")))
        })
        .transpose()?;
    let bitrate = unsigned(writer, "video_bitrate", 1, u32::MAX)?;
    // Encoder/container/rate-mode compatibility belongs to VideoJob::with_quality,
    // shared with the Rust facade, not to a second Python policy implementation.
    Ok(VideoQuality {
        crf,
        preset,
        tune,
        bitrate,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn portal_video_quality_acceptance_suite() {
        crate::with_python_test_module("portal video quality", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../../tests/video_quality.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("writer quality reaches native negotiation and the video bitstream");
        });
    }

    #[test]
    fn portal_video_quality_console_acceptance_suite() {
        crate::with_python_test_module("portal console video quality", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../../tests/video_quality_console.py"))
                    .unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("console controls reach single, batch, subdivided, and paired exports");
        });
    }
}
