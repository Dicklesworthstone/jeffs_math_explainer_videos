//! Native mathematical documents. Parsing and every shape are prepared before
//! installation; host code only hangs the resulting scene-owned child proxies.
use crate::{BridgeMobject, install_native_tree, native_error, srgb_from_py, with_font_book};
use fmn_library::markdown::{MAX_MARKDOWN_BYTES, Markdown, MarkdownMathOptions};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyModule};

type BuiltDocument<'py> = (Bound<'py, PyList>, Vec<(usize, usize, String)>);

#[pyfunction(signature = (slf, factory, source, font_size=24.0, line_width=None, block_gap=0.25, code_style="monokai", color=None))]
#[allow(clippy::too_many_arguments)]
fn _build_math_markdown<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    source: &str,
    font_size: f64,
    line_width: Option<f64>,
    block_gap: f64,
    code_style: &str,
    color: Option<&Bound<'py, PyAny>>,
) -> PyResult<BuiltDocument<'py>> {
    if slf.try_borrow()?.engine.is_some() {
        return Err(PyValueError::new_err(
            "mathematical Markdown construction requires a detached receiver",
        ));
    }
    if source.len() > MAX_MARKDOWN_BYTES {
        return Err(PyValueError::new_err(
            "Markdown source exceeds 32768 UTF-8 bytes",
        ));
    }
    let theme = fmn_library::CodeTheme::from_pygments_name(code_style)
        .ok_or_else(|| PyValueError::new_err("unsupported native Markdown code theme"))?;
    let color = color
        .map(srgb_from_py)
        .transpose()?
        .unwrap_or(fmn_core::constants::WHITE);
    let builder = Markdown::new(source)
        .font_size(font_size)
        .block_gap(block_gap)
        .theme(theme);
    let built = with_font_book(|book| {
        crate::with_tex_engine(|engine| {
            builder
                .build_with_math_options(book, engine, MarkdownMathOptions { line_width, color })
                .map_err(native_error)
        })
    })?;
    let catalog = built
        .blocks
        .iter()
        .map(|b| (b.byte_range.0, b.byte_range.1, b.kind.to_owned()))
        .collect();
    let specs = install_native_tree(slf, factory, built.vmob)?;
    Ok((specs, catalog))
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_build_math_markdown, module)?)
}
