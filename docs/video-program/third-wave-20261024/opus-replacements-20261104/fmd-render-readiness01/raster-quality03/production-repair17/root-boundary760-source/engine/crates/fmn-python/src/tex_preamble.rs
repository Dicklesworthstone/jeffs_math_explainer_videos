//! Native preamble witnesses through the production Python and scene runtime.
#[cfg(feature = "gauntlet")]
use pyo3::prelude::*;
#[cfg(feature = "gauntlet")]
use pyo3::types::PyDictMethods;

#[cfg(feature = "gauntlet")]
pub fn run_portal_gauntlet_tex_preamble(
    destination: &std::path::Path,
    seed: u64,
) -> Result<crate::PortalLiveTexReport, String> {
    let destination = destination
        .to_str()
        .ok_or_else(|| "TeX preamble destination is not UTF-8".to_owned())?;
    crate::with_python_test_module("TeX preamble Gauntlet", |py, _module, globals| {
        let source = std::ffi::CString::new(include_str!("../tests/tex_preamble_render.py"))
            .expect("TeX preamble renderer contains no NUL");
        py.run(source.as_c_str(), Some(globals), Some(globals))
            .inspect_err(|error| error.print(py))
            .map_err(|error| error.to_string())?;
        type Rendered = (Vec<u8>, Vec<u8>, u64, u64, u64);
        let (first_png, last_png, frame_count, thread_counts, failure_paths) = globals
            .get_item("render_tex_preamble")
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "TeX preamble renderer is absent".to_owned())?
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
    #[test]
    fn tex_preamble_native_semantics() {
        crate::with_python_test_module("TeX preamble semantics", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/tex_preamble.py"))
                .expect("TeX preamble tests contain no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("load native preamble acceptance");
            py.run(
                c"result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(TexPreambleTests))\nassert result.wasSuccessful(), 'TeX preamble acceptance failed'",
                Some(globals),
                Some(globals),
            )
            .inspect_err(|error| error.print(py))
            .expect("native preamble acceptance through the embedding");
        });
    }
}
