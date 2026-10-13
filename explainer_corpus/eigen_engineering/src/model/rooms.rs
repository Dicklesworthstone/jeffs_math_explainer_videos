//! Act III: the air in a room. Between hard walls the air has patterns of
//! its own, standing waves, and for a box they can be written down: the
//! mode with nx, ny and nz half-waves along the three sides rings at
//! (c/2)·√((nx/Lx)² + (ny/Ly)² + (nz/Lz)²), the formula FrankenSim's
//! `rectangular_cavity_modes` implements
//! (`crates/fs-couple/src/vibroacoustic.rs`). A room that is not a box has
//! no formula: its air is cut into cells, the cells give a stiffness table,
//! and the table's eigenvectors are the room's modes ([`Section`]).
//!
//! Sides are whole decimetres and frequencies whole hertz wherever modes are
//! counted, so that "below 1,000 Hz" is decided in whole numbers and never
//! by a rounded square root.

use super::linalg::{Eigen, Mat, jacobi_eigh};

/// The speed of sound in air at 20 °C, metres a second (FrankenSim derives
/// 343.0 to within 0.5 from dry air at 293.15 K and one atmosphere).
pub const SOUND_M_PER_S: f64 = 343.0;
const SOUND_DM_PER_S: u128 = 3430;

/// A box-shaped room with hard walls; sides in decimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Room {
    pub name: &'static str,
    pub dm: [u64; 3],
}

/// A tiled bathroom (a model): 2.5 by 1.6 m, 2.4 m high.
pub const BATHROOM: Room = Room {
    name: "bathroom",
    dm: [25, 16, 24],
};
/// A cube 3 m on a side: the worst room.
pub const CUBE: Room = Room {
    name: "cube",
    dm: [30, 30, 30],
};
/// An ordinary room (a model): 5 by 4 m, 2.5 m high.
pub const STUDY: Room = Room {
    name: "study",
    dm: [50, 40, 25],
};
/// A box the size of a large concert hall (a model): 45 by 23 m, 18 m high.
pub const HALL: Room = Room {
    name: "hall",
    dm: [450, 230, 180],
};

/// Reverberation times, seconds: the hall's is Beranek's occupied
/// mid-frequency figure for Boston Symphony Hall; the study's is a model.
pub const HALL_T60_S: f64 = 1.9;
pub const STUDY_T60_S: f64 = 0.5;
/// Boston Symphony Hall's volume, cubic metres (Beranek 2004).
pub const BOSTON_M3: f64 = 18_750.0;
/// Schroeder and Kuttruff's constant (1962): the crossover is 2000·√(T/V).
pub const SCHROEDER: f64 = 2000.0;
/// Sabine's constant in metric units, as written today: T = 0.161·V/A.
pub const SABINE: f64 = 0.161;

impl Room {
    pub fn sides_m(&self) -> [f64; 3] {
        self.dm.map(|d| d as f64 / 10.0)
    }

    pub fn volume_m3(&self) -> f64 {
        self.sides_m().iter().product()
    }

    /// The area of the six walls, square metres.
    pub fn surface_m2(&self) -> f64 {
        let [x, y, z] = self.sides_m();
        2.0 * (x * y + x * z + y * z)
    }

    /// The length of the twelve edges, metres.
    pub fn edges_m(&self) -> f64 {
        4.0 * self.sides_m().iter().sum::<f64>()
    }

    /// The frequency in hertz of the mode with `n` half-waves along the sides.
    pub fn mode_hz(&self, n: [u32; 3]) -> f64 {
        let s = self.sides_m();
        SOUND_M_PER_S / 2.0 * (0..3).map(|i| (f64::from(n[i]) / s[i]).powi(2)).sum::<f64>().sqrt()
    }

    /// Whether the mode `n` rings at or below `hertz`, decided in whole
    /// numbers: c²·Σ (nᵢ · the other two sides)² ≤ (2 · hertz · Lx Ly Lz)².
    fn at_or_below(&self, n: [u64; 3], hertz: u64) -> bool {
        let [x, y, z] = self.dm.map(u128::from);
        let terms = [
            u128::from(n[0]) * y * z,
            u128::from(n[1]) * x * z,
            u128::from(n[2]) * x * y,
        ];
        let left: u128 = SOUND_DM_PER_S * SOUND_DM_PER_S * terms.iter().map(|t| t * t).sum::<u128>();
        let right = 2 * u128::from(hertz) * x * y * z;
        left <= right * right
    }

    /// The most half-waves that fit along side `i` at or below `hertz`.
    fn most(&self, i: usize, hertz: u64) -> u64 {
        // n ≤ 2·L·f / c
        (2 * u128::from(self.dm[i]) * u128::from(hertz) / SOUND_DM_PER_S) as u64
    }

    /// How many modes ring at or below `hertz` (the all-zero "mode", still
    /// air at rest, is not one).
    pub fn count_below(&self, hertz: u64) -> u64 {
        let mut count = 0u64;
        for nx in 0..=self.most(0, hertz) {
            for ny in 0..=self.most(1, hertz) {
                for nz in 0..=self.most(2, hertz) {
                    if self.at_or_below([nx, ny, nz], hertz) {
                        count += 1;
                    }
                }
            }
        }
        count - 1
    }

    /// The modes at or below `hertz`, lowest first (ties in the order of
    /// their numbers), with their frequencies.
    pub fn modes_below(&self, hertz: u64) -> Vec<([u32; 3], f64)> {
        let mut out = Vec::new();
        for nx in 0..=self.most(0, hertz) {
            for ny in 0..=self.most(1, hertz) {
                for nz in 0..=self.most(2, hertz) {
                    if (nx, ny, nz) != (0, 0, 0) && self.at_or_below([nx, ny, nz], hertz) {
                        let n = [nx as u32, ny as u32, nz as u32];
                        out.push((n, self.mode_hz(n)));
                    }
                }
            }
        }
        out.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        out
    }

    /// The textbook estimate of the count: (4π/3)·V·(f/c)³ + (π/4)·S·(f/c)²
    /// + (L/8)·(f/c).
    pub fn count_estimate(&self, hertz: f64) -> f64 {
        let r = hertz / SOUND_M_PER_S;
        4.0 * std::f64::consts::PI / 3.0 * self.volume_m3() * r.powi(3)
            + std::f64::consts::PI / 4.0 * self.surface_m2() * r.powi(2)
            + self.edges_m() / 8.0 * r
    }

    /// Modes per hertz near `hertz`, from the estimate's leading term:
    /// 4π·V·f²/c³.
    pub fn modes_per_hertz(&self, hertz: f64) -> f64 {
        4.0 * std::f64::consts::PI * self.volume_m3() * hertz * hertz / SOUND_M_PER_S.powi(3)
    }

    /// The Schroeder frequency for a reverberation time of `t60` seconds:
    /// above it the modes overlap too thickly to be told apart.
    pub fn schroeder_hz(&self, t60: f64) -> f64 {
        SCHROEDER * (t60 / self.volume_m3()).sqrt()
    }

    /// The absorption, in square metres, that Sabine's formula asks for to
    /// reach a reverberation time of `t60` seconds.
    pub fn sabine_m2(&self, t60: f64) -> f64 {
        SABINE * self.volume_m3() / t60
    }
}

/// The side view of a hall, cut into square cells of air: `air[row][col]`,
/// row 0 at the floor. Each cell trades pressure with the cells it touches,
/// and a wall trades nothing, so the stiffness table has, for every pair of
/// touching cells, +1 on their two diagonal entries and −1 between them.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub cols: usize,
    pub rows: usize,
    pub cell_m: f64,
    pub air: Vec<Vec<bool>>,
}

pub const SECTION_COLS: usize = 30;
pub const SECTION_ROWS: usize = 12;
pub const SECTION_CELL_M: f64 = 1.5;

impl Section {
    /// The plain rectangle, 45 by 18 m: the hall's box seen from the side.
    pub fn boxed() -> Self {
        Self {
            cols: SECTION_COLS,
            rows: SECTION_ROWS,
            cell_m: SECTION_CELL_M,
            air: vec![vec![true; SECTION_COLS]; SECTION_ROWS],
        }
    }

    /// A hall that is not a box (a model): the ceiling drops by 6 m over
    /// the first 9 m (the stage), and the floor climbs toward the back in
    /// five steps of 1.5 m, each 3 m deep (the raked seats).
    pub fn hall() -> Self {
        let mut s = Self::boxed();
        for row in 0..s.rows {
            for col in 0..s.cols {
                let over_the_stage = col < 6 && row >= s.rows - 4;
                let steps = if col >= 20 { (col - 20) / 2 + 1 } else { 0 };
                s.air[row][col] = !over_the_stage && row >= steps;
            }
        }
        s
    }

    /// The cells of air, as (row, col), row by row from the floor.
    pub fn cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for (row, across) in self.air.iter().enumerate() {
            for (col, air) in across.iter().enumerate() {
                if *air {
                    cells.push((row, col));
                }
            }
        }
        cells
    }

    /// The cells' stiffness table (in units of one shared face).
    pub fn table(&self) -> Mat {
        let cells = self.cells();
        let index = |row: usize, col: usize| cells.iter().position(|&c| c == (row, col));
        let mut k = Mat::zeros(cells.len());
        for (i, &(row, col)) in cells.iter().enumerate() {
            for (r, c) in [(row + 1, col), (row, col + 1)] {
                if r < self.rows && c < self.cols && self.air[r][c] {
                    let j = index(r, c).expect("an air cell is in the list");
                    k.add(i, i, 1.0);
                    k.add(j, j, 1.0);
                    k.add(i, j, -1.0);
                    k.add(j, i, -1.0);
                }
            }
        }
        k
    }

    /// The table's eigenpairs, lowest first. The first is always zero: all
    /// the air at one pressure, at rest.
    pub fn eigen(&self) -> Eigen {
        jacobi_eigh(&self.table())
    }

    /// An eigenvalue as a frequency in hertz: (c / 2πh)·√λ for cells h wide.
    pub fn hertz(&self, lambda: f64) -> f64 {
        SOUND_M_PER_S / (std::f64::consts::TAU * self.cell_m) * lambda.max(0.0).sqrt()
    }

    /// The box's own answer for the side view's mode (nx, ny), by the formula.
    pub fn box_formula_hz(&self, nx: u32, ny: u32) -> f64 {
        let (lx, ly) = (self.cols as f64 * self.cell_m, self.rows as f64 * self.cell_m);
        SOUND_M_PER_S / 2.0 * ((f64::from(nx) / lx).powi(2) + (f64::from(ny) / ly).powi(2)).sqrt()
    }

    /// What the cells give for the plain box's mode (nx, ny). A full
    /// rectangle of cells has eigenvalues known in closed form,
    /// 4·sin²(π nx / 2·cols) + 4·sin²(π ny / 2·rows), so the cell method can
    /// be checked against the formula where the formula exists (a test
    /// confirms the closed form against the table's own eigenvalues, and the
    /// oracle solves the full table).
    pub fn box_cells_hz(&self, nx: u32, ny: u32) -> f64 {
        let half = std::f64::consts::FRAC_PI_2;
        let sx = (half * f64::from(nx) / self.cols as f64).sin();
        let sy = (half * f64::from(ny) / self.rows as f64).sin();
        self.hertz(4.0 * (sx * sx + sy * sy))
    }
}

/// The modes of the hall's side view, solved once (306 cells take a moment).
pub fn hall_section() -> &'static (Section, Eigen) {
    static SOLVED: std::sync::OnceLock<(Section, Eigen)> = std::sync::OnceLock::new();
    SOLVED.get_or_init(|| {
        let hall = Section::hall();
        let eigen = hall.eigen();
        (hall, eigen)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_agrees_with_listing_the_modes() {
        for room in [BATHROOM, CUBE, STUDY] {
            for hertz in [50, 100, 200] {
                assert_eq!(room.count_below(hertz) as usize, room.modes_below(hertz).len());
            }
        }
        // the cube's first three modes are one frequency
        let cube = CUBE.modes_below(60);
        assert_eq!(cube.len(), 3);
        assert!(cube.iter().all(|(_, hz)| (hz - cube[0].1).abs() < 1e-12));
    }

    #[test]
    fn the_estimate_tracks_the_exact_count_in_the_hall() {
        for hertz in [100u64, 500, 1000] {
            let exact = HALL.count_below(hertz) as f64;
            let estimate = HALL.count_estimate(hertz as f64);
            assert!((estimate / exact - 1.0).abs() < 0.002, "{hertz}: {exact} vs {estimate}");
        }
    }

    #[test]
    fn the_hall_section_is_one_piece_of_air_with_one_mode_at_rest() {
        let (hall, e) = hall_section();
        assert_eq!(hall.cells().len(), 306);
        assert!(e.values[0].abs() < 1e-10);
        assert!(e.values[1] > 1e-4, "one piece: only one eigenvalue of zero");
        assert!(hall.table().asymmetry() == 0.0);
    }

    /// The closed form for a full rectangle of cells is the table's own
    /// eigenvalues (checked on a small rectangle, every eigenvalue), and for
    /// the hall's box it sits within a tenth of a percent of the formula.
    #[test]
    fn cells_ring_close_to_the_formula_for_the_plain_box() {
        let small = Section {
            cols: 7,
            rows: 4,
            cell_m: SECTION_CELL_M,
            air: vec![vec![true; 7]; 4],
        };
        let mut closed: Vec<f64> = (0..7u32)
            .flat_map(|nx| (0..4u32).map(move |ny| (nx, ny)))
            .map(|(nx, ny)| small.box_cells_hz(nx, ny))
            .collect();
        closed.sort_by(f64::total_cmp);
        let solved = small.eigen();
        for (lambda, want) in solved.values.iter().zip(&closed) {
            assert!((small.hertz(*lambda) - want).abs() < 1e-9, "{lambda} vs {want}");
        }
        let b = Section::boxed();
        assert!((b.box_cells_hz(1, 0) / b.box_formula_hz(1, 0) - 1.0).abs() < 1e-3);
    }
}
