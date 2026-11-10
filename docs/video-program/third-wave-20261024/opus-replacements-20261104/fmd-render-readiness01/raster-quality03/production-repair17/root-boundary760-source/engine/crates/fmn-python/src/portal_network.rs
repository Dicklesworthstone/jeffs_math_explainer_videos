//! Bounded host entry to Atlas's deterministic network layouts. No Python
//! layout implementation, external networkx installation, or scene mutation.
use std::collections::BTreeSet;

use fmn_library::{GraphLayout, NetworkGraph};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyString, PyTuple};

const MAX_NODES: usize = 1024;
const MAX_EDGES: usize = 8192;
const MAX_LABEL_BYTES: usize = 1_048_576;
const MAX_SPRING_WORK: usize = 16_777_216;

type LayoutRows = (Vec<String>, Vec<(String, String)>, Vec<[f64; 3]>);

fn label(value: &Bound<'_, PyAny>, bytes: &mut usize) -> PyResult<String> {
    let text = value.cast::<PyString>()?.to_str()?;
    *bytes = bytes
        .checked_add(text.len())
        .filter(|&n| n <= MAX_LABEL_BYTES)
        .ok_or_else(|| PyValueError::new_err("network labels exceed the 1 MiB budget"))?;
    Ok(text.to_owned())
}

#[pyfunction(signature = (nodes, edges, layout="spring", *, seed=0, iterations=50, ideal_edge=1.0, shells=None, root=None))]
#[allow(clippy::too_many_arguments)] // The host-facing layout option schema.
fn _network_graph_layout(
    py: Python<'_>,
    nodes: &Bound<'_, PyList>,
    edges: &Bound<'_, PyList>,
    layout: &str,
    seed: u64,
    iterations: usize,
    ideal_edge: f64,
    shells: Option<&Bound<'_, PyList>>,
    root: Option<String>,
) -> PyResult<LayoutRows> {
    if nodes.len() > MAX_NODES || edges.len() > MAX_EDGES {
        return Err(PyValueError::new_err(
            "network exceeds 1024 nodes or 8192 edges",
        ));
    }
    let mut bytes = 0;
    let mut names = Vec::new();
    let mut known = BTreeSet::new();
    for item in nodes.iter() {
        let name = label(&item, &mut bytes)?;
        if known.insert(name.clone()) {
            names.push(name);
        }
    }
    let mut pairs = Vec::new();
    let mut seen_edges = BTreeSet::new();
    for item in edges.iter() {
        let pair = item.cast::<PyTuple>()?;
        if pair.len() != 2 {
            return Err(PyValueError::new_err("a network edge requires two labels"));
        }
        let a = label(&pair.get_item(0)?, &mut bytes)?;
        let b = label(&pair.get_item(1)?, &mut bytes)?;
        for name in [&a, &b] {
            if known.insert(name.clone()) {
                if names.len() == MAX_NODES {
                    return Err(PyValueError::new_err("network exceeds 1024 nodes"));
                }
                names.push(name.clone());
            }
        }
        let key = if a <= b {
            (a.clone(), b.clone())
        } else {
            (b.clone(), a.clone())
        };
        if seen_edges.insert(key) {
            pairs.push((a, b));
        }
    }
    let algorithm = match layout {
        "circular" => GraphLayout::Circular,
        "shell" => {
            let groups =
                shells.ok_or_else(|| PyValueError::new_err("shell layout requires shells"))?;
            if groups.len() > MAX_NODES {
                return Err(PyValueError::new_err("too many network shells"));
            }
            let mut partition = Vec::new();
            let mut count = 0usize;
            for group in groups.iter() {
                let group = group.cast::<PyList>()?;
                count = count
                    .checked_add(group.len())
                    .filter(|&n| n <= MAX_NODES)
                    .ok_or_else(|| PyValueError::new_err("too many shell members"))?;
                let mut ring = Vec::new();
                for item in group.iter() {
                    ring.push(label(&item, &mut bytes)?);
                }
                partition.push(ring);
            }
            GraphLayout::Shell { shells: partition }
        }
        "breadth_first" => GraphLayout::BreadthFirst {
            root: root
                .ok_or_else(|| PyValueError::new_err("breadth_first layout requires root"))?,
        },
        "spring" => {
            let work = names
                .len()
                .checked_mul(names.len().saturating_sub(1))
                .map(|n| n / 2)
                .and_then(|n| n.checked_add(pairs.len()))
                .and_then(|n| n.checked_mul(iterations));
            if iterations == 0 || iterations > 1000 || work.is_none_or(|n| n > MAX_SPRING_WORK) {
                return Err(PyValueError::new_err(
                    "spring layout exceeds its bounded iteration work",
                ));
            }
            if !ideal_edge.is_finite() || !(1e-6..=1e6).contains(&ideal_edge) {
                return Err(PyValueError::new_err(
                    "spring ideal_edge must be finite and in [1e-6, 1e6]",
                ));
            }
            GraphLayout::Spring {
                seed,
                iterations,
                ideal_edge,
            }
        }
        _ => return Err(PyValueError::new_err("unknown native network layout")),
    };
    py.detach(move || {
        let names_ref: Vec<_> = names.iter().map(String::as_str).collect();
        let pairs_ref: Vec<_> = pairs
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let graph = NetworkGraph::from_edge_list(&names_ref, &pairs_ref)
            .laid_out(&algorithm)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let mut points = Vec::with_capacity(names.len());
        for name in &names {
            let point = graph
                .position(name)
                .filter(|p| p.iter().all(|v| v.is_finite()))
                .ok_or_else(|| {
                    PyValueError::new_err("network layout did not produce a finite position")
                })?;
            points.push(point);
        }
        Ok((names, pairs, points))
    })
}

pub(crate) fn install(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_network_graph_layout, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pyo3::types::{PyAnyMethods, PyDictMethods};

    #[test]
    fn native_network_layouts() {
        crate::with_python_test_module("native network layouts", |py, _module, globals| {
            let code =
                std::ffi::CString::new(include_str!("../tests/native_network_layout.py")).unwrap();
            py.run(code.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .unwrap();
            globals
                .get_item("run_network_layout_acceptance")
                .unwrap()
                .unwrap()
                .call0()
                .inspect_err(|error| error.print(py))
                .unwrap();
        });
    }
}
