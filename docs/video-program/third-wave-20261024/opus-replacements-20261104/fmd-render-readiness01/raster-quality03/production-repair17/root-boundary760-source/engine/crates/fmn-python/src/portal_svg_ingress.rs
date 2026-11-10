//! Bounded SVG paint ingress. Chisel owns XML, styles, booleans and arclength;
//! this bridge only admits overrides and installs the completed native family.
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::{BridgeMobject, install_native_tree, native_error, srgb_from_py};
use fmn_library::svg::{SvgPaintOverrides, svg_mobject_with_paints};

#[pyfunction]
fn _build_svg_paints<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    source: &str,
    options: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyList>> {
    // The native parser enforces its own byte budget before parsing, too.
    if source.len() > 1_048_576 {
        return Err(PyValueError::new_err("SVG input exceeds 1048576 bytes"));
    }
    let mut overrides = SvgPaintOverrides::default();
    for (key, value) in options.iter() {
        match key.extract::<String>()?.as_str() {
            "fill_color" => overrides.fill_color = Some(srgb_from_py(&value)?),
            "stroke_color" => overrides.stroke_color = Some(srgb_from_py(&value)?),
            "fill_opacity" => overrides.fill_opacity = Some(value.extract()?),
            "stroke_opacity" => overrides.stroke_opacity = Some(value.extract()?),
            "stroke_width" => overrides.stroke_width = Some(value.extract()?),
            name => {
                return Err(PyTypeError::new_err(format!(
                    "unknown SVG paint override: {name}"
                )));
            }
        }
    }
    let prepared = slf
        .py()
        .detach(|| svg_mobject_with_paints(source.as_bytes(), overrides).map_err(native_error))?;
    install_native_tree(slf, factory, prepared)
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_build_svg_paints, module)?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_svg_subclass_lifecycle() {
        crate::with_python_test_module("SVG subclass lifecycle", |py, _module, globals| {
            let source = std::ffi::CString::new(concat!(
                include_str!("../tests/svg_lifecycle.py"),
                "\n_suite = unittest.defaultTestLoader.loadTestsFromTestCase(SvgLifecycleTests)\n",
                "_result = unittest.TextTestRunner(verbosity=2).run(_suite)\n",
                "assert _result.wasSuccessful(), 'native SVG lifecycle acceptance failed'\n",
            ))
            .unwrap();
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("native SVG subclass lifecycle");
        });
    }
}
