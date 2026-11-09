//! Structured typography: bounded native CSV, table and Markdown construction.
//! Atlas owns parsing/layout; this boundary only admits owned host values.
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyString};

use crate::{BridgeMobject, install_native_tree, native_error, with_font_book};
use fmn_library::data_mobjects::TableMobject;

const MAX_BYTES: usize = 262_144;
const MAX_CELLS: usize = 4096;
type StringGrid = (Vec<String>, Vec<Vec<String>>);

fn validate(headers: &[String], rows: &[Vec<String>]) -> PyResult<()> {
    if headers.is_empty() || rows.is_empty() {
        return Err(PyValueError::new_err(
            "table requires headers and at least one body row",
        ));
    }
    if rows
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(headers.len()))
        .is_none_or(|n| n > MAX_CELLS)
    {
        return Err(PyValueError::new_err(
            "table exceeds 4096 cells including headers",
        ));
    }
    let mut bytes = 0usize;
    for row in std::iter::once(headers).chain(rows.iter().map(Vec::as_slice)) {
        if row.len() != headers.len() {
            return Err(PyValueError::new_err(
                "table rows must match the header count",
            ));
        }
        for value in row {
            bytes = bytes
                .checked_add(value.len())
                .filter(|&n| n <= MAX_BYTES)
                .ok_or_else(|| PyValueError::new_err("table exceeds 262144 UTF-8 bytes"))?;
        }
    }
    Ok(())
}

#[pyfunction(signature = (text, separator=','))]
fn _table_from_csv(py: Python<'_>, text: &str, separator: char) -> PyResult<StringGrid> {
    if text.len() > MAX_BYTES {
        return Err(PyValueError::new_err(
            "table CSV exceeds 262144 UTF-8 bytes",
        ));
    }
    if ['\r', '\n', '\0', '"'].contains(&separator) {
        return Err(PyValueError::new_err(
            "table CSV separator must not be a quote, NUL or newline",
        ));
    }
    // fp-frame's convenience CSV reader is delimiter-only at the governed
    // pin. Refuse syntax it would silently truncate/misparse. A quote-aware
    // reader lives in fp-io, whose Arrow/Excel/parser closure is not admitted
    // here. This is structural admission, not a second scalar/CSV decoder.
    if text.contains('"') {
        return Err(PyValueError::new_err(
            "quoted CSV is unavailable in the pinned native frame reader; supply a literal string grid instead",
        ));
    }
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    if let Some(header) = lines.next() {
        let separators = header.matches(separator).count();
        if lines.any(|line| line.matches(separator).count() != separators) {
            return Err(PyValueError::new_err(
                "table CSV rows must match the header field count; no fields were discarded",
            ));
        }
    }
    py.detach(|| {
        let table = TableMobject::from_csv(text, separator).map_err(native_error)?;
        validate(table.headers(), table.rows())?;
        Ok((table.headers().to_vec(), table.rows().to_vec()))
    })
}

fn strings(values: &Bound<'_, PyList>, bytes: &mut usize) -> PyResult<Vec<String>> {
    let mut result = Vec::with_capacity(values.len());
    for value in values.iter() {
        let text = value.cast::<PyString>()?.to_str()?;
        *bytes = bytes
            .checked_add(text.len())
            .filter(|&n| n <= MAX_BYTES)
            .ok_or_else(|| PyValueError::new_err("table exceeds 262144 UTF-8 bytes"))?;
        result.push(text.to_owned());
    }
    Ok(result)
}

#[pyfunction]
fn _build_table<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    headers: &Bound<'py, PyList>,
    rows: &Bound<'py, PyList>,
) -> PyResult<Bound<'py, PyList>> {
    if headers.is_empty()
        || rows.is_empty()
        || rows
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(headers.len()))
            .is_none_or(|n| n > MAX_CELLS)
    {
        return Err(PyValueError::new_err(
            "table requires a nonempty grid of at most 4096 cells",
        ));
    }
    // Check row lengths before cloning any authored strings or invoking Scribe.
    for row in rows.iter() {
        if row.cast::<PyList>()?.len() != headers.len() {
            return Err(PyValueError::new_err(
                "table rows must match the header count",
            ));
        }
    }
    let mut bytes = 0;
    let names = strings(headers, &mut bytes)?;
    let mut body = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        body.push(strings(row.cast::<PyList>()?, &mut bytes)?);
    }
    validate(&names, &body)?;
    let table = TableMobject::from_grid(names, body);
    let tree = with_font_book(|book| table.build(book).map_err(native_error))?;
    install_native_tree(slf, factory, tree)
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_table_from_csv, module)?)?;
    module.add_function(wrap_pyfunction!(_build_table, module)?)?;
    module.add_function(wrap_pyfunction!(_build_markdown, module)?)
}

type MarkdownSpecs<'py> = (Bound<'py, PyList>, Vec<(usize, usize)>, Vec<String>);

/// Preserve fmd source byte ranges alongside Scribe's real native blocks.
#[pyfunction(signature = (slf, factory, source, font_size=48.0, theme="monokai", block_gap=0.45))]
fn _build_markdown<'py>(
    slf: &Bound<'py, BridgeMobject>,
    factory: &Bound<'py, PyAny>,
    source: &str,
    font_size: f64,
    theme: &str,
    block_gap: f64,
) -> PyResult<MarkdownSpecs<'py>> {
    if slf.try_borrow()?.engine.is_some() {
        return Err(PyValueError::new_err(
            "Markdown construction requires a detached receiver",
        ));
    }
    let theme = fmn_library::CodeTheme::from_pygments_name(theme)
        .ok_or_else(|| PyValueError::new_err("unknown native Markdown code theme"))?;
    let built = with_font_book(|book| {
        fmn_library::Markdown::new(source)
            .font_size(font_size)
            .theme(theme)
            .block_gap(block_gap)
            .build(book)
            .map_err(native_error)
    })?;
    let ranges = built.blocks.iter().map(|block| block.byte_range).collect();
    let kinds = built
        .blocks
        .iter()
        .map(|block| block.kind.to_owned())
        .collect();
    let specs = install_native_tree(slf, factory, built.vmob)?;
    Ok((specs, ranges, kinds))
}

#[cfg(test)]
mod markdown_tests {
    use pyo3::types::{PyAnyMethods, PyDictMethods};

    #[test]
    fn native_markdown_source_and_rendering() {
        crate::with_python_test_module("native Markdown", |py, _module, globals| {
            let code = std::ffi::CString::new(include_str!("../tests/native_markdown.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_native_markdown_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
