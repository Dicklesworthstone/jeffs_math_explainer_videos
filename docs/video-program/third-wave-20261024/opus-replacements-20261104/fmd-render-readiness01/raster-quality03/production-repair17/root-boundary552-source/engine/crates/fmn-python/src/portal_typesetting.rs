//! Production typesetting services over the SAME engines used by Tex, native
//! matrices, and markdown. No Python serialization or parallel layout kernel.

use pyo3::exceptions::{PyMemoryError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule, PyString};

use super::with_tex_template;

const TEMPLATES: [&str; 3] = ["default", "basic", "empty"];

fn stats<'py>(py: Python<'py>, engine: &fmn_library::TexEngine) -> PyResult<Bound<'py, PyDict>> {
    let memory = engine.memory_cache_stats();
    let row = PyDict::new(py);
    row.set_item("persistent", engine.persistent_cache_enabled())?;
    row.set_item("memory_hits", memory.hits)?;
    row.set_item("memory_misses", memory.misses)?;
    row.set_item("memory_entries", memory.entries)?;
    row.set_item("memory_bytes", memory.bytes)?;
    row.set_item("disk_hits", engine.persistent_cache_hits())?;
    row.set_item("disk_bytes_read", engine.persistent_bytes_read())?;
    row.set_item("disk_bytes_written", engine.persistent_bytes_written())?;
    row.set_item("disk_rejected", engine.persistent_rejected_entries())?;
    row.set_item("disk_errors", engine.persistent_store_errors())?;
    row.set_item("layout_computations", engine.layout_computations())?;
    let preflight = engine.preflight_stats();
    row.set_item("preflight_batches", preflight.batches)?;
    row.set_item("preflight_requests", preflight.requests)?;
    row.set_item("preflight_workers", preflight.workers)?;
    row.set_item("preflight_active_workers", preflight.active_workers)?;
    row.set_item("preflight_wall_ns", preflight.wall_ns)?;
    row.set_item("pack", engine.pack_content_id())?;
    Ok(row)
}

/// A storage refusal is a report, not a typesetting failure. Engine/font
/// construction failures still surface normally rather than masquerading as
/// a cache miss. `None` explicitly detaches all three persistent namespaces.
#[pyfunction]
#[pyo3(signature = (directory=None))]
fn _configure_tex_cache<'py>(
    py: Python<'py>,
    directory: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    if directory.is_some_and(|value| value.len() > 4096 || value.contains('\0')) {
        return Err(PyValueError::new_err(
            "typeset cache directory must contain at most 4096 UTF-8 bytes and no NUL",
        ));
    }
    let result = PyDict::new(py);
    result.set_item("schema", "fmn-python.typeset-cache")?;
    result.set_item("version", 1)?;
    result.set_item("requested", directory.is_some())?;
    result.set_item("directory", directory)?;
    let templates = PyDict::new(py);
    for name in TEMPLATES {
        let row = with_tex_template(name, |engine| {
            let error = engine
                .configure_host_cache(directory)
                .err()
                .map(|error| error.to_string().chars().take(2048).collect::<String>());
            let row = stats(py, engine)?;
            row.set_item("error", error)?;
            Ok(row)
        })?;
        templates.set_item(name, row)?;
    }
    result.set_item("templates", templates)?;
    Ok(result)
}

#[pyfunction]
fn _tex_cache_info(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let result = PyDict::new(py);
    for name in TEMPLATES {
        result.set_item(name, with_tex_template(name, |engine| stats(py, engine))?)?;
    }
    Ok(result)
}

const MAX_PREFLIGHT_ITEMS: usize = 4096;
const MAX_PREFLIGHT_SOURCE_BYTES: usize = 262_144;
const MAX_PREFLIGHT_BYTES: usize = 4 * 1024 * 1024;
const MAX_PREFLIGHT_WORKERS: usize = 64;

/// Warm the production engine, not a temporary engine whose memory cache is
/// discarded. Own all Python inputs before releasing the GIL; native workers
/// never observe Python objects, mutate a Scene, or run authored callbacks.
#[pyfunction]
#[pyo3(signature = (sources, *, template="", preamble="", text_mode=false, alignment=None, max_workers=32))]
fn _preflight_tex<'py>(
    py: Python<'py>,
    sources: &Bound<'py, PyList>,
    template: &str,
    preamble: &str,
    text_mode: bool,
    alignment: Option<&str>,
    max_workers: usize,
) -> PyResult<Bound<'py, PyDict>> {
    if sources.len() > MAX_PREFLIGHT_ITEMS {
        return Err(PyValueError::new_err("Tex preflight exceeds 4096 sources"));
    }
    if !(1..=MAX_PREFLIGHT_WORKERS).contains(&max_workers) {
        return Err(PyValueError::new_err(
            "Tex preflight max_workers must be in 1..=64",
        ));
    }
    if preamble.len() > MAX_PREFLIGHT_SOURCE_BYTES {
        return Err(PyValueError::new_err(
            "Tex preflight preamble exceeds 262144 UTF-8 bytes",
        ));
    }
    let align = match alignment.unwrap_or(if text_mode { "center" } else { "left" }) {
        "left" => fmn_library::LineAlign::Left,
        "center" if text_mode => fmn_library::LineAlign::Center,
        "right" if text_mode => fmn_library::LineAlign::Right,
        _ => {
            return Err(PyValueError::new_err(
                "Tex preflight alignment must be left for mathematics, or left/center/right for text",
            ));
        }
    };
    // Validate the entire batch before opening an engine or warming any entry.
    // The aggregate bound charges repeated preambles, not merely input storage.
    let allocation_error = || PyMemoryError::new_err("cannot allocate Tex preflight inputs");
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(sources.len())
        .map_err(|_| allocation_error())?;
    let mut total = 0usize;
    for (index, value) in sources.iter().enumerate() {
        let value = value.cast::<PyString>().map_err(|_| {
            PyTypeError::new_err(format!("Tex preflight source {index} must be str"))
        })?;
        let source = value.to_str()?;
        if source.len() > MAX_PREFLIGHT_SOURCE_BYTES {
            return Err(PyValueError::new_err(format!(
                "Tex preflight source {index} exceeds 262144 UTF-8 bytes",
            )));
        }
        total = total
            .saturating_add(source.len())
            .saturating_add(preamble.len());
        if total > MAX_PREFLIGHT_BYTES {
            return Err(PyValueError::new_err(
                "Tex preflight exceeds 4 MiB of source and preamble bytes",
            ));
        }
        let mut text = String::new();
        text.try_reserve_exact(source.len())
            .map_err(|_| allocation_error())?;
        text.push_str(source);
        owned.push(text);
    }
    let mut declarations = String::new();
    declarations
        .try_reserve_exact(preamble.len())
        .map_err(|_| allocation_error())?;
    declarations.push_str(preamble);
    let mut requests = Vec::new();
    requests
        .try_reserve_exact(owned.len() + usize::from(!owned.is_empty()))
        .map_err(|_| allocation_error())?;
    for source in &owned {
        let mut request = if text_mode {
            fmn_library::TypesetRequest::text(source)
        } else {
            fmn_library::TypesetRequest::math(source)
        };
        request.preamble = &declarations;
        request.align = align;
        requests.push(request);
    }
    // Tex::build and TexText::build share the text-style zero calibration.
    // Warm it too so the first constructor does not need a hidden cold layout.
    if !owned.is_empty() {
        requests.push(fmn_library::TypesetRequest::inline_math("0"));
    }
    let limit = std::num::NonZeroUsize::new(max_workers)
        .ok_or_else(|| PyValueError::new_err("Tex preflight needs at least one worker"))?;
    py.check_signals()?;
    with_tex_template(template, |engine| {
        let before = stats(py, engine)?;
        let mut outcomes = py
            .detach(|| engine.preflight_requests(&requests, limit))
            .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
        if !owned.is_empty() {
            outcomes
                .pop()
                .ok_or_else(|| PyValueError::new_err("Tex preflight lost calibration outcome"))?
                .map_err(|error| {
                    PyValueError::new_err(format!(
                        "Tex preflight scale calibration failed: {error}",
                    ))
                })?;
        }
        py.check_signals()?;
        let result = PyDict::new(py);
        let rows = PyList::empty(py);
        let mut succeeded = 0usize;
        for (index, outcome) in outcomes.into_iter().enumerate() {
            let row = PyDict::new(py);
            row.set_item("index", index)?;
            row.set_item("ok", outcome.is_ok())?;
            let error = outcome
                .err()
                .map(|error| error.to_string().chars().take(2048).collect::<String>());
            succeeded += usize::from(error.is_none());
            row.set_item("error", error)?;
            rows.append(row)?;
        }
        result.set_item("schema", "fmn-python.typeset-preflight")?;
        result.set_item("version", 1)?;
        result.set_item("count", owned.len())?;
        result.set_item("succeeded", succeeded)?;
        result.set_item("failed", owned.len() - succeeded)?;
        // This is a ceiling, not a claim about successful thread creation.
        // fmn-tex owns CPU availability and spawn-refusal fallback.
        result.set_item("worker_limit", max_workers)?;
        result.set_item("before", before)?;
        result.set_item("after", stats(py, engine)?)?;
        result.set_item("items", rows)?;
        Ok(result)
    })
}

pub(super) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_configure_tex_cache, module)?)?;
    module.add_function(wrap_pyfunction!(_tex_cache_info, module)?)?;
    module.add_function(wrap_pyfunction!(_preflight_tex, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_warms_the_production_constructor_engines() {
        crate::with_python_test_module("typeset preflight", |py, module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/typeset_preflight.py"))
                .expect("typeset preflight tests contain no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_typeset_preflight")
                .unwrap()
                .unwrap()
                .call1((module,))
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }

    #[test]
    fn scene_run_preflights_literal_tex_and_honours_the_configured_cache() {
        crate::with_python_test_module("static typeset preflight", |py, module, globals| {
            let source =
                std::ffi::CString::new(include_str!("../tests/typeset_static_preflight.py"))
                    .expect("static preflight tests contain no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            for suite in [
                "run_static_preflight",
                "run_discovery_robustness",
                "run_store_errors_reported",
                "run_config_selected_cache",
                "run_refused_config_cache",
            ] {
                globals
                    .get_item(suite)
                    .unwrap()
                    .unwrap()
                    .call1((module,))
                    .inspect_err(|error| error.print(py))
                    .unwrap();
            }
        });
    }

    #[test]
    fn production_tex_constructors_use_the_persistent_cache() {
        crate::with_python_test_module("typeset cache", |py, module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/typeset_cache.py"))
                .expect("typeset cache tests contain no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_typeset_cache")
                .unwrap()
                .unwrap()
                .call1((module,))
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
