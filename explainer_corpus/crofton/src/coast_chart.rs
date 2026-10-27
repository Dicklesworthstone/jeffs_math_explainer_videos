//! A finite picture of line space, distinct from the exact length identity.
use crate::{chapters::sheet, line_chart, line_chart::Address, model::Curve};
use std::f64::consts::PI;

pub const COLUMNS: usize = 360;
pub const ROWS: usize = 184;
pub const CROFTON_YEAR: usize = 1868;

pub struct Grid {
    /// Column-major midpoint counts; None means the midpoint misses the card.
    pub cells: Vec<Option<usize>>,
    pub columns: usize,
    pub rows: usize,
}
impl Grid {
    pub fn build(curve: &Curve, columns: usize, rows: usize) -> Self {
        assert!(columns > 0 && rows > 0);
        let mut grid = Self {
            cells: Vec::with_capacity(columns * rows),
            columns,
            rows,
        };
        for i in 0..columns {
            for j in 0..rows {
                let a = grid.address(i, j);
                grid.cells.push(
                    (a.distance.abs() < line_chart::half_width(a.theta))
                        .then(|| curve.meetings(a.theta - PI / 2., a.distance)),
                );
            }
        }
        grid
    }
    pub fn address(&self, i: usize, j: usize) -> Address {
        Address {
            theta: PI * (i as f64 + 0.5) / self.columns as f64,
            distance: line_chart::reach() * (2. * (j as f64 + 0.5) / self.rows as f64 - 1.),
        }
    }
    pub fn cell_area(&self) -> f64 {
        PI / self.columns as f64 * 2. * line_chart::reach() / self.rows as f64
    }
    pub fn histogram(&self) -> Vec<usize> {
        let mut bins = vec![0; self.cells.iter().flatten().copied().max().unwrap_or(0) + 1];
        for &n in self.cells.iter().flatten() {
            bins[n] += 1;
        }
        bins
    }
    pub fn prefix(&self) -> Vec<f64> {
        let mut out = vec![0.];
        for column in self.cells.chunks_exact(self.rows) {
            out.push(out.last().copied().unwrap() + column.iter().flatten().sum::<usize>() as f64 * self.cell_area());
        }
        out
    }
    /// Select a cell of the requested count with the greatest distance from
    /// endpoints/tangencies and the card boundary. This is an illustration,
    /// not another random sample or a claim about every address of that color.
    pub fn representative(&self, curve: &Curve, count: usize) -> Address {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, n)| **n == Some(count))
            .map(|(k, _)| {
                let a = self.address(k / self.rows, k % self.rows);
                let clear = sheet::turning_heights(curve, a.theta - PI / 2.)
                    .iter()
                    .map(|h| (h - a.distance).abs())
                    .fold(line_chart::half_width(a.theta) - a.distance.abs(), f64::min);
                (a, clear)
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("requested count occurs in grid")
            .0
    }
}

pub fn intersections(curve: &Curve, a: Address) -> Vec<sheet::Crossing> {
    // A sheet with slide p includes our full line as k=0. Keep only that line.
    sheet::crossings(curve, a.theta - PI / 2., a.distance)
        .into_iter()
        .filter(|x| x.line == 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;
    #[test]
    fn plotted_cells_and_physical_dots_use_the_same_normal_convention() {
        let c = model::coast();
        let grid = Grid::build(&c, COLUMNS, ROWS);
        for (i, count) in grid.cells.iter().enumerate() {
            let a = grid.address(i / ROWS, i % ROWS);
            let dots = intersections(&c, a);
            assert_eq!(dots.len(), count.unwrap_or(0));
            for dot in dots {
                assert!((dot.at[0] * a.theta.cos() + dot.at[1] * a.theta.sin() - a.distance).abs() < 1e-10);
                let point = c.at(dot.along);
                assert!((point[0] - dot.at[0]).hypot(point[1] - dot.at[1]) < 1e-10);
            }
        }
    }
    #[test]
    fn finite_grid_is_not_the_exact_integral_or_a_hit_indicator() {
        let c = model::coast();
        let grid = Grid::build(&c, COLUMNS, ROWS);
        let h = grid.histogram();
        let total = *grid.prefix().last().unwrap();
        assert!((total - 2. * c.length).abs() < 0.01);
        assert!((total - 2. * c.length).abs() > 1e-5);
        let once = h.iter().skip(1).sum::<usize>() as f64 * grid.cell_area();
        assert!(total - once > 18.);
        assert_eq!(h, [19856, 26734, 2554, 6721, 892, 376, 307]);
        for n in [0, 1, 6] {
            assert_eq!(intersections(&c, grid.representative(&c, n)).len(), n);
        }
    }
    #[test]
    fn retracing_counts_traversals_and_doubles_the_integral() {
        let c = model::coast();
        let g = Grid::build(&c, 72, 46);
        let mut twice = c.clone();
        twice.parts.extend(c.parts.iter().copied());
        let h = Grid::build(&twice, 72, 46);
        for (a, b) in g.cells.iter().zip(h.cells.iter()) {
            assert_eq!(a.map(|n| 2 * n), *b);
        }
        assert!((2. * g.prefix().last().unwrap() - h.prefix().last().unwrap()).abs() < 1e-10);
    }
}
