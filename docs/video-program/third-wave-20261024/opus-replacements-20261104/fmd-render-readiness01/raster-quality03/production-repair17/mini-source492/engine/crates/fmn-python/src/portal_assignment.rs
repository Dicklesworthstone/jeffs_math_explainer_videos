//! Native `linear_sum_assignment` for the `manimlib` namespace.
//!
//! The Reference imports `scipy.optimize.linear_sum_assignment` into
//! `manimlib.mobject.svg.string_mobject`, so `from manimlib import *` exports
//! it. FrankenManim has no scipy dependency; the name is served by the
//! suite's Hungarian solver (plan D4: assignment belongs to frankenscipy)
//! instead of a stand-in that ignores the cost matrix.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Minimum-cost assignment of a rectangular cost matrix, scipy's contract:
/// row indices ascend, `min(rows, cols)` pairs are returned, and `maximize`
/// negates the objective.
#[pyfunction]
#[pyo3(signature = (cost_matrix, maximize = false))]
pub(crate) fn _linear_sum_assignment(
    cost_matrix: Vec<Vec<f64>>,
    maximize: bool,
) -> PyResult<(Vec<usize>, Vec<usize>)> {
    let cost = if maximize {
        cost_matrix
            .into_iter()
            .map(|row| row.into_iter().map(|value| -value).collect())
            .collect()
    } else {
        cost_matrix
    };
    fsci_opt::linear_sum_assignment(&cost).map_err(|error| PyValueError::new_err(error.to_string()))
}
