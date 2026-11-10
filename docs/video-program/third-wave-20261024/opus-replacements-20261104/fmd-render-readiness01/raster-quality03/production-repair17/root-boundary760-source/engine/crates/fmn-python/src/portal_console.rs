//! Real native witnesses for checkpointed host scene editing (fm-ffj.70).

use pyo3::prelude::*;
use pyo3::types::PyDictMethods;

/// Exercise SceneState, callback identity, rational time and native readback.
///
/// # Errors
/// Returns the actual Python/native acceptance failure, never a ledger verdict.
pub fn run_portal_gauntlet_console() -> Result<u64, String> {
    crate::with_python_test_module("scene console", |py, _module, globals| {
        let source = std::ffi::CString::new(include_str!("../tests/scene_console_acceptance.py"))
            .map_err(|error| error.to_string())?;
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        globals
            .get_item("run_console_acceptance")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "scene console acceptance entry point is absent".to_owned())?
            .call0()
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<u64>())
            .map_err(|error| error.to_string())
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_scene_console_acceptance() {
        assert_eq!(super::run_portal_gauntlet_console().unwrap(), 7);
    }
}
