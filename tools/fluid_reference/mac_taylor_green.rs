//! Independent periodic staggered-grid check for the fluid explainer.
//! Explicit midpoint, centered conservative fluxes, and a mean-zero pressure.

use std::f64::consts::TAU;

const VISCOSITY: f64 = 0.05;
const END: f64 = 0.8;

#[derive(Clone, Copy)]
struct Neighbors {
    east: usize,
    west: usize,
    north: usize,
    south: usize,
}

struct Grid {
    n: usize,
    h: f64,
    neighbors: Vec<Neighbors>,
}

#[derive(Clone)]
struct Velocity {
    u: Vec<f64>,
    v: Vec<f64>,
}

fn inf_norm(values: &[f64]) -> f64 {
    values.iter().map(|x| x.abs()).fold(0.0, f64::max)
}

fn dot(x: &[f64], y: &[f64]) -> f64 {
    x.iter().zip(y).map(|(a, b)| a * b).sum()
}

impl Grid {
    fn new(n: usize) -> Self {
        let index = |i: usize, j: usize| (j % n) * n + i % n;
        let neighbors = (0..n * n)
            .map(|k| {
                let (i, j) = (k % n, k / n);
                Neighbors {
                    east: index(i + 1, j),
                    west: index(i + n - 1, j),
                    north: index(i, j + 1),
                    south: index(i, j + n - 1),
                }
            })
            .collect();
        Self { n, h: TAU / n as f64, neighbors }
    }

    fn exact(&self, time: f64) -> Velocity {
        let decay = (-2.0 * VISCOSITY * time).exp();
        let mut result = Velocity::zeros(self.n * self.n);
        for k in 0..self.n * self.n {
            let x = self.h * (k % self.n) as f64;
            let y = self.h * (k / self.n) as f64;
            result.u[k] = decay * x.sin() * (y + self.h / 2.0).cos();
            result.v[k] = -decay * (x + self.h / 2.0).cos() * y.sin();
        }
        result
    }

    fn divergence(&self, v: &Velocity, k: usize) -> f64 {
        let Neighbors { east, north, .. } = self.neighbors[k];
        (v.u[east] - v.u[k] + v.v[north] - v.v[k]) / self.h
    }

    fn negative_laplacian(&self, q: &[f64], k: usize) -> f64 {
        let Neighbors { east, west, north, south } = self.neighbors[k];
        (4.0 * q[k] - q[east] - q[west] - q[north] - q[south]) / self.h.powi(2)
    }

    fn momentum(&self, v: &Velocity, rhs: &mut Velocity) {
        let (u, v) = (&v.u, &v.v);
        for (k, &Neighbors { east, west, north, south }) in self.neighbors.iter().enumerate() {
            let average = |a, b| (a + b) / 2.0;
            let u_e = average(u[k], u[east]);
            let u_w = average(u[west], u[k]);
            let v_n = average(v[k], v[north]);
            let v_s = average(v[south], v[k]);
            let uv_n = average(u[k], u[north]) * average(v[north], v[self.neighbors[north].west]);
            let uv_s = average(u[south], u[k]) * average(v[k], v[west]);
            let vu_e = average(v[k], v[east]) * average(u[east], u[self.neighbors[east].south]);
            let vu_w = average(v[west], v[k]) * average(u[k], u[south]);
            rhs.u[k] = -(u_e * u_e - u_w * u_w + uv_n - uv_s) / self.h
                - VISCOSITY * self.negative_laplacian(u, k);
            rhs.v[k] = -(vu_e - vu_w + v_n * v_n - v_s * v_s) / self.h
                - VISCOSITY * self.negative_laplacian(v, k);
        }
    }
}

impl Velocity {
    fn zeros(count: usize) -> Self {
        Self { u: vec![0.0; count], v: vec![0.0; count] }
    }

    fn advance(&mut self, source: &Self, rhs: &Self, step: f64) {
        for k in 0..self.u.len() {
            self.u[k] = source.u[k] + step * rhs.u[k];
            self.v[k] = source.v[k] + step * rhs.v[k];
        }
    }

    fn relative_error(&self, reference: &Self) -> f64 {
        let error: f64 = self.u.iter().zip(&reference.u).chain(self.v.iter().zip(&reference.v))
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        (error / (dot(&reference.u, &reference.u) + dot(&reference.v, &reference.v))).sqrt()
    }

    fn mean_energy(&self) -> f64 {
        (dot(&self.u, &self.u) + dot(&self.v, &self.v)) / (2.0 * self.u.len() as f64)
    }
}

struct Projection {
    original_rhs: Vec<f64>,
    pressure: Vec<f64>,
    residual: Vec<f64>,
    direction: Vec<f64>,
    product: Vec<f64>,
    total_iterations: usize,
    max_volume_defect: f64,
    max_scaled_residual: f64,
    max_identity_error: f64,
}

impl Projection {
    fn new(count: usize) -> Self {
        Self {
            original_rhs: vec![0.0; count],
            pressure: vec![0.0; count],
            residual: vec![0.0; count],
            direction: vec![0.0; count],
            product: vec![0.0; count],
            total_iterations: 0,
            max_volume_defect: 0.0,
            max_scaled_residual: 0.0,
            max_identity_error: 0.0,
        }
    }

    fn apply(&mut self, grid: &Grid, v: &mut Velocity, step: f64, tolerance: f64) {
        let count = v.u.len();
        for (k, rhs) in self.original_rhs.iter_mut().enumerate() {
            *rhs = -grid.divergence(v, k) / step;
        }
        let mean = self.original_rhs.iter().sum::<f64>() / count as f64;
        assert!(mean.abs() < 1e-10, "periodic fluxes do not telescope");
        for k in 0..count {
            self.residual[k] = self.original_rhs[k] - mean;
        }
        self.direction.copy_from_slice(&self.residual);
        self.pressure.fill(0.0);
        let bound = tolerance / step.powi(2);
        let mut squared = dot(&self.residual, &self.residual);
        let mut iterations = 0;
        while inf_norm(&self.residual) > bound {
            assert!(iterations < 4 * grid.n, "pressure solver hit its iteration cap");
            self.product.iter_mut().enumerate().for_each(|(k, value)| {
                *value = grid.negative_laplacian(&self.direction, k);
            });
            let curvature = dot(&self.direction, &self.product);
            assert!(curvature.is_finite() && curvature > 0.0);
            let fraction = squared / curvature;
            for k in 0..count {
                self.pressure[k] += fraction * self.direction[k];
                self.residual[k] -= fraction * self.product[k];
            }
            let next = dot(&self.residual, &self.residual);
            let beta = next / squared;
            for k in 0..count {
                self.direction[k] = self.residual[k] + beta * self.direction[k];
            }
            squared = next;
            iterations += 1;
        }

        let p_mean = self.pressure.iter().sum::<f64>() / count as f64;
        for k in 0..count {
            self.pressure[k] -= p_mean;
        }
        for (k, &Neighbors { west, south, .. }) in grid.neighbors.iter().enumerate() {
            v.u[k] -= step / grid.h * (self.pressure[k] - self.pressure[west]);
            v.v[k] -= step / grid.h * (self.pressure[k] - self.pressure[south]);
        }

        self.total_iterations += iterations;
        for k in 0..count {
            let actual = step * grid.divergence(v, k);
            let residual = self.original_rhs[k] - grid.negative_laplacian(&self.pressure, k);
            let predicted = -step.powi(2) * residual;
            self.max_volume_defect = self.max_volume_defect.max(actual.abs());
            self.max_scaled_residual = self.max_scaled_residual.max(predicted.abs());
            self.max_identity_error = self.max_identity_error.max((actual - predicted).abs());
        }
    }
}

fn simulate(label: &str, n: usize, steps: usize, tolerance: f64) -> Velocity {
    let grid = Grid::new(n);
    let dt = END / steps as f64;
    let mut v = grid.exact(0.0);
    let mut half = v.clone();
    let mut next = v.clone();
    let mut rhs = Velocity::zeros(n * n);
    let mut projection = Projection::new(n * n);
    let mut max_cfl = 0.0_f64;
    for _ in 0..steps {
        max_cfl = max_cfl.max(dt * (inf_norm(&v.u) + inf_norm(&v.v)) / grid.h);
        grid.momentum(&v, &mut rhs);
        half.advance(&v, &rhs, dt / 2.0);
        projection.apply(&grid, &mut half, dt / 2.0, tolerance);
        grid.momentum(&half, &mut rhs);
        next.advance(&v, &rhs, dt);
        projection.apply(&grid, &mut next, dt, tolerance);
        std::mem::swap(&mut v, &mut next);
    }
    let exact = grid.exact(END);
    let error = v.relative_error(&exact);
    assert!(error.is_finite());
    assert!(projection.max_identity_error < 1e-12);
    assert!(projection.max_volume_defect <= 1.001 * tolerance + 1e-13);
    assert!(max_cfl < 0.65 && VISCOSITY * dt / grid.h.powi(2) < 0.25);
    println!(
        "{{\"run\":\"{label}\",\"cells_per_side\":{n},\"steps\":{steps},\"t\":{END},\
         \"spacing\":{},\"dt\":{dt},\"pressure_budget\":{tolerance},\"relative_velocity_l2\":{error},\
         \"mean_kinetic_energy\":{},\"exact_mean_kinetic_energy\":{},\"max_cfl\":{max_cfl},\
         \"pressure_iterations\":{},\"max_volume_defect\":{},\"max_scaled_pressure_residual\":{},\
         \"max_residual_identity_error\":{}}}",
        grid.h, v.mean_energy(), exact.mean_energy(), projection.total_iterations,
        projection.max_volume_defect, projection.max_scaled_residual, projection.max_identity_error,
    );
    v
}

fn main() {
    let space = [16, 32, 64].map(|n| {
        simulate("space", n, 160, 1e-11).relative_error(&Grid::new(n).exact(END))
    });
    let ratios = [space[0] / space[1], space[1] / space[2]];
    assert!(ratios.iter().all(|&ratio| (3.7..4.3).contains(&ratio)), "{ratios:?}");
    println!("{{\"space_error_ratios\":{ratios:?}}}");

    let reference = simulate("time_reference", 32, 640, 1e-11);
    let time = [20, 40, 80].map(|steps| {
        simulate("time", 32, steps, 1e-11).relative_error(&reference)
    });
    let ratios = [time[0] / time[1], time[1] / time[2]];
    println!("{{\"same_grid_time_errors\":{time:?},\"time_error_ratios\":{ratios:?}}}");
    assert!(ratios.iter().all(|&ratio| (3.5..4.6).contains(&ratio)), "{ratios:?}");
    let loose = simulate("pressure_budget", 32, 80, 1e-8);
    let tight = simulate("pressure_budget_100x_tighter", 32, 80, 1e-10);
    let change = loose.relative_error(&tight);
    println!("{{\"velocity_change_after_tightening_pressure\":{change}}}");
    assert!(change < 1e-5);
}
