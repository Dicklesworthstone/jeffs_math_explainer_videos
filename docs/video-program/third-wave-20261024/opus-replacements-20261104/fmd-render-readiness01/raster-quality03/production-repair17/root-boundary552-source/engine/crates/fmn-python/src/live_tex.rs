//! Executable live-equation witnesses shared by embedded and wheel acceptance.

#[cfg(feature = "gauntlet")]
use pyo3::prelude::*;
#[cfg(feature = "gauntlet")]
use pyo3::types::PyDictMethods;

/// Actual native PNG observations, not source-resolution or ledger evidence.
#[cfg(feature = "gauntlet")]
#[derive(Debug)]
pub struct PortalLiveTexReport {
    /// Initial equation and matrix before changing their numeric values.
    pub first_png: Vec<u8>,
    /// Final equation and matrix after native number animations.
    pub last_png: Vec<u8>,
    /// Frames independently decoded in each output generation.
    pub frame_count: u64,
    /// Thread counts with identical complete sequences (1, 4, and 16).
    pub thread_counts: u64,
    /// Cancellation and existing-destination refusal witnesses.
    pub failure_paths: u64,
}

/// Render live coefficients and matrix entries through the production portal.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_live_tex(
    destination: &std::path::Path,
    seed: u64,
) -> Result<PortalLiveTexReport, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "live equation destination is not UTF-8".to_owned())?;
    crate::with_python_test_module("live equations Gauntlet", |py, _module, globals| {
        let source = std::ffi::CString::new(include_str!("../tests/live_tex_render.py"))
            .expect("live equation renderer contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        type Rendered = (Vec<u8>, Vec<u8>, u64, u64, u64);
        let (first_png, last_png, frame_count, thread_counts, failure_paths) = globals
            .get_item("render_live_tex")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "live equation renderer is absent".to_owned())?
            .call1((destination, seed))
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<Rendered>())
            .map_err(|error| error.to_string())?;
        Ok(PortalLiveTexReport {
            first_png,
            last_png,
            frame_count,
            thread_counts,
            failure_paths,
        })
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn live_tex_native_semantics() {
        crate::with_python_test_module("live numeric TeX", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/live_tex.py"))
                .expect("live TeX tests contain no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("live numeric TeX semantics through native mobjects");
        });
    }
}
