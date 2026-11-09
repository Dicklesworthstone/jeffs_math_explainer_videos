//! Complex numeric authoring witnesses through the same production portal.

#[cfg(feature = "gauntlet")]
use pyo3::prelude::*;
#[cfg(any(feature = "gauntlet", test))]
use pyo3::types::PyDictMethods;

/// Render complex live coefficients and matrix cells with native PNG output.
#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_complex_readouts(
    destination: &std::path::Path,
    seed: u64,
) -> Result<crate::PortalLiveTexReport, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "complex readout destination is not UTF-8".to_owned())?;
    crate::with_python_test_module("complex readout Gauntlet", |py, _module, globals| {
        // The independent PNG decoder is shared with the live-equation witness
        // in the source test directory, not taken from the native codec.
        globals
            .set_item(
                "__file__",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/complex_readouts_render.py"
                ),
            )
            .map_err(|error| error.to_string())?;
        let source = std::ffi::CString::new(include_str!("../tests/complex_readouts_render.py"))
            .expect("complex readout renderer contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        type Rendered = (Vec<u8>, Vec<u8>, u64, u64, u64);
        let (first_png, last_png, frame_count, thread_counts, failure_paths) = globals
            .get_item("render_complex_readouts")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "complex readout renderer is absent".to_owned())?
            .call1((destination, seed))
            .inspect_err(|error| error.print(py))
            .and_then(|value| value.extract::<Rendered>())
            .map_err(|error| error.to_string())?;
        Ok(crate::PortalLiveTexReport {
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
    use super::*;

    fn run_semantics(suite: &'static str, source: &str, class_name: &str) {
        crate::with_python_test_module(suite, |py, _module, globals| {
            let source = std::ffi::CString::new(source).expect("acceptance source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("load complex numeric acceptance");
            // Embedded suites deliberately have their own module name. Run
            // the actual TestCase explicitly rather than relying on __main__.
            globals
                .set_item("_test_case_name", class_name)
                .expect("set numeric acceptance class");
            py.run(
                c"result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(globals()[_test_case_name]))\nassert result.wasSuccessful(), 'complex numeric acceptance failed'",
                Some(globals),
                Some(globals),
            )
            .inspect_err(|error| error.print(py))
            .expect("complex numeric semantics through the native embedding");
        });
    }

    #[test]
    fn complex_decimal_native_semantics() {
        run_semantics(
            "complex decimal",
            include_str!("../tests/decimal_authoring.py"),
            "DecimalAuthoringTests",
        );
    }

    #[test]
    fn complex_matrix_native_semantics() {
        run_semantics(
            "complex matrix",
            include_str!("../tests/complex_matrix.py"),
            "ComplexMatrixTests",
        );
    }
}
