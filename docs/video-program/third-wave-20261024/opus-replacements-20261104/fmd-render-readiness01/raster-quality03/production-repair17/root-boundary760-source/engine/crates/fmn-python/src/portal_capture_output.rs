//! Explicit PNG publication of an immutable Lumen capture through Reel.
//!
//! Preparation encodes and syncs a private file. Commit is create-only and
//! invokes no Python code. Dropping or aborting a preparation cancels its sink.

use std::path::PathBuf;
use std::sync::Arc;

use fmn_frame::FrameBuffer;
use fmn_output::{FrameSink, PngSink, PngSinkConfig, PngTarget, SinkLimits};
use fmn_platform::fs::{FileSystem, StdFs};
use pyo3::prelude::*;

use crate::{PyRuntimeError, PyValueError, native_error};

type Receipt = (PathBuf, u64, String);

#[pyclass(unsendable, name = "_PreparedCapturePng")]
pub(crate) struct PreparedPng {
    sink: Option<PngSink>,
    result: Option<Receipt>,
}

pub(crate) fn prepare(
    frame: &FrameBuffer,
    width: u32,
    height: u32,
    destination: PathBuf,
    threads: usize,
) -> PyResult<PreparedPng> {
    if destination.as_os_str().is_empty() || !(1..=96).contains(&threads) {
        return Err(PyValueError::new_err(
            "PNG capture requires a nonempty destination and 1..=96 threads",
        ));
    }
    let path = std::path::absolute(destination).map_err(native_error)?;
    let fs: Arc<dyn FileSystem> = Arc::new(StdFs);
    if fs
        .node_kind_no_follow(&path)
        .map_err(native_error)?
        .is_some()
    {
        return Err(pyo3::exceptions::PyFileExistsError::new_err(format!(
            "PNG capture destination already exists: {}",
            path.display()
        )));
    }
    // Capture itself is already capped at 16M pixels. Include encoded bytes
    // and codec scratch in the resident budget, and cap this sink to one frame.
    let bytes = u64::from(width) * u64::from(height) * 4;
    let limits = SinkLimits::new(1, bytes * 3 + 4 * 1024 * 1024, bytes, bytes * 2 + 4096)
        .and_then(|limits| limits.requiring_exact_frames(1))
        .map_err(native_error)?;
    let mut sink = PngSink::new(
        fs,
        PngSinkConfig {
            target: PngTarget::Single(path),
            width,
            height,
            first_sequence: 0,
            compression: fmn_codec::CompressionLevel::Default,
            threads,
            limits,
            profile: None,
        },
    )
    .map_err(native_error)?
    .with_no_clobber();
    sink.write_frame(0, frame).map_err(native_error)?;
    sink.prepare_finish().map_err(native_error)?;
    Ok(PreparedPng {
        sink: Some(sink),
        result: None,
    })
}

#[pymethods]
impl PreparedPng {
    /// Publish once; return the same verified receipt on repeated success.
    fn commit(&mut self, py: Python<'_>) -> PyResult<Receipt> {
        if let Some(result) = &self.result {
            return Ok(result.clone());
        }
        let mut sink = self.sink.take().ok_or_else(|| {
            PyRuntimeError::new_err("PNG preparation is aborted or already failed")
        })?;
        let result = py.detach(move || {
            let receipt = sink.receipt();
            sink.commit_finish().map_err(native_error)?;
            let report = receipt.take().map_err(native_error)?;
            Ok::<_, PyErr>((report.path, report.bytes, report.digest.to_hex()))
        })?;
        self.result = Some(result.clone());
        Ok(result)
    }

    /// Cancel only this prepared native file; published files are never removed.
    fn abort(&mut self) {
        if let Some(mut sink) = self.sink.take() {
            sink.abort();
        }
    }
}

pub(crate) fn save(
    py: Python<'_>,
    frame: &FrameBuffer,
    width: u32,
    height: u32,
    destination: PathBuf,
    threads: usize,
) -> PyResult<Receipt> {
    let mut pending = py.detach(|| prepare(frame, width, height, destination, threads))?;
    pending.commit(py)
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_capture_publication_acceptance() {
        crate::with_python_test_module("capture output", |py, _module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/capture_output.py")).unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native prepared PNG publication and cancellation");
        });
    }
}
