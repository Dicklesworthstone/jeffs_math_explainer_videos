//! Mechanical exposure of Atlas seed generation and Chisel width profiles.
//! No Python callback, Stage borrow, second RNG or ODE implementation lives here.
use fmn_core::rng::RngRoot;
use fmn_core::types::Vec3;
use fmn_library::coords::CoordinateSystem;
use fmn_library::fields::{StreamLines, taper_by_true_length};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

struct SeedGrid {
    ranges: Vec<[f64; 3]>,
    dimension: usize,
    x_unit: f64,
}
impl CoordinateSystem for SeedGrid {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        [
            coords.first().copied().unwrap_or(0.0) * self.x_unit,
            coords.get(1).copied().unwrap_or(0.0),
            coords.get(2).copied().unwrap_or(0.0),
        ]
    }
    fn p2c(&self, point: Vec3) -> Vec3 {
        [point[0] / self.x_unit, point[1], point[2]]
    }
    fn all_ranges(&self) -> Vec<[f64; 3]> {
        self.ranges.clone()
    }
    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (ranges, dimension, x_unit, density, n_repeats, noise_factor, seed))]
pub(crate) fn _stream_line_samples(
    ranges: Vec<[f64; 3]>,
    dimension: usize,
    x_unit: f64,
    density: f64,
    n_repeats: usize,
    noise_factor: Option<f64>,
    seed: u64,
) -> PyResult<(Vec<Vec3>, u64)> {
    if !(1..=3).contains(&dimension)
        || ranges.len() != dimension
        || !x_unit.is_finite()
        || x_unit <= 0.0
        || ranges
            .iter()
            .any(|row| !row[2].is_finite() || row[2] <= 0.0)
    {
        return Err(PyValueError::new_err("invalid StreamLines coordinate grid"));
    }
    let cs = SeedGrid {
        ranges,
        dimension,
        x_unit,
    };
    let rng = RngRoot::from_seed(seed);
    let mut builder = StreamLines::new(|rows| vec![[0.0; 3]; rows.len()], cs, &rng)
        .with_density(density)
        .with_n_repeats(n_repeats);
    if let Some(noise) = noise_factor {
        builder = builder.with_noise_factor(noise);
    }
    builder
        .sample_coords()
        .map_err(|error| PyValueError::new_err(error.to_string()))
}

#[pyfunction]
pub(crate) fn _stream_line_widths(
    points: Vec<Vec3>,
    width: f64,
    taper: bool,
) -> PyResult<Vec<f64>> {
    if points.len() > 1_048_576
        || !width.is_finite()
        || width < 0.0
        || width > f64::from(f32::MAX)
        || points
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > f64::from(f32::MAX))
    {
        return Err(PyValueError::new_err(
            "invalid or oversized StreamLines width input",
        ));
    }
    let vmob = fmn_library::VMobject::from_points(points);
    let path = vmob
        .path()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(if taper {
        taper_by_true_length(&path, &[0.0, width, 0.0])
    } else {
        vec![width; path.points().len()]
    })
}
