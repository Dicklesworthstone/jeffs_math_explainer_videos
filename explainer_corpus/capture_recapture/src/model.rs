//! The lake the video draws (oracle.py, "A-F27 and A-CH03: one habitat
//! model"): the unit square, x east and y north, holding 10,000 fish drawn
//! i.i.d. from a habitat suitability by rejection sampling with SplitMix64,
//! a tiny portable generator, so the scene draws the oracle's lake fish for
//! fish. Chapter 01 dips its net into this lake; chapter 03 mixes its marks.

/// Fish in the lake (Feller's true size, chapters 07 and 09).
pub const LAKE_FISH: usize = 10_000;
/// The dip grid: each cell is exactly 1/100 of the lake.
pub const GRID: usize = 10;
/// The project seed (every research script and the oracle use it).
pub const SEED: u64 = 20_261_009;
/// Habitat wells: centre x, centre y, amplitude, standard deviation.
const WEEDS: (f64, f64, f64, f64) = (0.25, 0.75, 40.0, 0.08);
const DEEP: (f64, f64, f64, f64) = (0.75, 0.25, 20.0, 0.12);
/// An upper bound on the suitability (1 + 40 + 20), the rejection ceiling.
const S_MAX: f64 = 61.0;

/// The three dip spots, named by habitat (01_5–01_7): the cells holding the
/// weed-bed centre, the lake's centre (open water) and the deep hole.
pub const SPOTS: [(&str, (f64, f64)); 3] = [("weeds", (0.25, 0.75)), ("open", (0.5, 0.5)), ("deep", (0.75, 0.25))];

/// SplitMix64 (Steele, Lea & Flood 2014; Vigna's reference constants).
#[derive(Clone, Debug)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1): the top 53 bits.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }
}

fn squared(d: f64) -> f64 {
    d * d
}

/// Habitat suitability: open water 1, a weed bed (north-west) and a deep
/// hole (south-east) on top. Fish density is proportional to it.
pub fn habitat(x: f64, y: f64) -> f64 {
    let well =
        |(cx, cy, amp, sd): (f64, f64, f64, f64)| amp * (-(squared(x - cx) + squared(y - cy)) / (2.0 * sd * sd)).exp();
    1.0 + well(WEEDS) + well(DEEP)
}

/// The grid cell (column west → east, row south → north) holding a point,
/// cells half-open, the far edges folded into the last cell.
pub fn cell_of(x: f64, y: f64) -> (usize, usize) {
    (
        ((x * GRID as f64) as usize).min(GRID - 1),
        ((y * GRID as f64) as usize).min(GRID - 1),
    )
}

/// The seeded clustered lake (A-F27).
#[derive(Clone, Debug)]
pub struct Lake {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    /// Points drawn before 10,000 were accepted.
    pub candidates: u64,
    /// Fish per cell, `counts[row][col]`, row 0 the southern edge.
    pub counts: [[u32; GRID]; GRID],
}

impl Lake {
    /// Draw the lake: x, y, then the acceptance draw, in that order, until
    /// 10,000 fish are accepted.
    pub fn draw() -> Self {
        let mut rng = SplitMix64::new(SEED);
        let (mut x, mut y) = (Vec::with_capacity(LAKE_FISH), Vec::with_capacity(LAKE_FISH));
        let mut candidates = 0u64;
        while x.len() < LAKE_FISH {
            let (px, py, a) = (rng.uniform(), rng.uniform(), rng.uniform());
            candidates += 1;
            if a * S_MAX < habitat(px, py) {
                x.push(px);
                y.push(py);
            }
        }
        let mut counts = [[0u32; GRID]; GRID];
        for (&px, &py) in x.iter().zip(&y) {
            let (c, r) = cell_of(px, py);
            counts[r][c] += 1;
        }
        Self {
            x,
            y,
            candidates,
            counts,
        }
    }

    /// Fish in the cell holding a point.
    pub fn count_at(&self, (x, y): (f64, f64)) -> u32 {
        let (c, r) = cell_of(x, y);
        self.counts[r][c]
    }

    /// Every cell's count, flattened.
    pub fn all_counts(&self) -> impl Iterator<Item = u32> + '_ {
        self.counts.iter().flatten().copied()
    }

    /// Dip estimates (100 × the count) within a factor of two of the truth.
    pub fn dips_within_2x(&self) -> usize {
        let n = LAKE_FISH as u32;
        self.all_counts()
            .filter(|&c| (n / 2..=2 * n).contains(&(c * 100)))
            .count()
    }

    /// The counts as the oracle prints them: rows south to north, `;`
    /// between rows, `,` between cells.
    pub fn counts_string(&self) -> String {
        self.counts
            .iter()
            .map(|r| r.iter().map(u32::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";")
    }
}

/// Chapter 03's mixing (oracle `ch03_portable`): overdamped drift up the
/// habitat (D ∇log s), a circulation along its contours (c J∇log s) and
/// noise, Euler–Maruyama at dt = 0.01, reflecting shores, from the A-F27
/// lake, with the 1,000 fish nearest the dock marked. The noise is
/// SplitMix64 as a counter stream (draw k is mix(seed + (k + 1) γ)) through
/// Irwin–Hall(4)·√3 normals, so the oracle and this port compute the same
/// increments bit for bit; every operation below is in the oracle's order.
pub mod mixing {
    use super::{DEEP, Lake, SEED, WEEDS, squared};

    pub const D: f64 = 0.01;
    pub const CIRCULATION: f64 = 0.02;
    pub const DT: f64 = 0.01;
    /// Steps to t = 400.
    pub const STEPS: u64 = 40_000;
    pub const DOCK: (f64, f64) = (0.5, 0.0);
    pub const MARKED: usize = 1_000;
    const PORT_SEED: u64 = SEED + 3;
    const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
    /// √3, spelled so both languages use the same bits.
    const SQRT3: f64 = 1.732_050_807_568_877_2;

    fn mix(mut z: u64) -> u64 {
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn draw(k: u64) -> f64 {
        let state = PORT_SEED.wrapping_add(k.wrapping_add(1).wrapping_mul(GAMMA));
        (mix(state) >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// Step `step`'s noise for fish `fish` of `n`: (x normal, y normal).
    pub fn normals(step: u64, fish: u64, n: u64) -> (f64, f64) {
        let base = 8 * n * step + 8 * fish;
        let z = |o: u64| {
            ((((draw(base + o) + draw(base + o + 1)) + draw(base + o + 2)) + draw(base + o + 3)) - 2.0) * SQRT3
        };
        (z(0), z(4))
    }

    /// ∇log s at a point, in the oracle's `habitat_np` order.
    fn grad_log_s(x: f64, y: f64) -> (f64, f64) {
        let (wx, wy, wa, ws) = WEEDS;
        let (dx, dy, da, ds) = DEEP;
        let gw = wa * (-(squared(x - wx) + squared(y - wy)) / (2.0 * ws * ws)).exp();
        let gd = da * (-(squared(x - dx) + squared(y - dy)) / (2.0 * ds * ds)).exp();
        let s = 1.0 + gw + gd;
        let gx = (-gw * (x - wx) / (ws * ws) - gd * (x - dx) / (ds * ds)) / s;
        let gy = (-gw * (y - wy) / (ws * ws) - gd * (y - dy) / (ds * ds)) / s;
        (gx, gy)
    }

    fn reflect(v: f64) -> f64 {
        1.0 - (1.0 - v.abs()).abs()
    }

    /// The population being mixed.
    #[derive(Clone, Debug)]
    pub struct Mixing {
        pub x: Vec<f64>,
        pub y: Vec<f64>,
        pub marked: Vec<bool>,
        /// Steps taken so far.
        pub step: u64,
    }

    impl Mixing {
        /// The A-F27 lake, the 1,000 fish nearest the dock marked (squared
        /// distance, ties to the lower index).
        pub fn start(lake: &Lake) -> Self {
            let n = lake.x.len();
            let mut order: Vec<usize> = (0..n).collect();
            let d2 = |i: usize| squared(lake.x[i] - DOCK.0) + squared(lake.y[i] - DOCK.1);
            order.sort_by(|&a, &b| d2(a).total_cmp(&d2(b)).then(a.cmp(&b)));
            let mut marked = vec![false; n];
            for &i in &order[..MARKED] {
                marked[i] = true;
            }
            Self {
                x: lake.x.clone(),
                y: lake.y.clone(),
                marked,
                step: 0,
            }
        }

        /// Advance `steps` steps. Fish are independent (each draws its own
        /// slice of the counter stream), so they move in parallel chunks.
        pub fn advance(&mut self, steps: u64) {
            let n = self.x.len() as u64;
            let first = self.step;
            let amp = (2.0 * D * DT).sqrt();
            let threads = std::thread::available_parallelism().map_or(1, |t| t.get()).min(16);
            let chunk = self.x.len().div_ceil(threads).max(1);
            std::thread::scope(|scope| {
                for (c, (xs, ys)) in self.x.chunks_mut(chunk).zip(self.y.chunks_mut(chunk)).enumerate() {
                    scope.spawn(move || {
                        for (k, (x, y)) in xs.iter_mut().zip(ys.iter_mut()).enumerate() {
                            let fish = (c * chunk + k) as u64;
                            for s in first..first + steps {
                                let (gx, gy) = grad_log_s(*x, *y);
                                let vx = CIRCULATION * gy + D * gx;
                                let vy = -(CIRCULATION * gx) + D * gy;
                                let (zx, zy) = normals(s, fish, n);
                                *x = reflect((*x + vx * DT) + amp * zx);
                                *y = reflect((*y + vy * DT) + amp * zy);
                            }
                        }
                    });
                }
            });
            self.step += steps;
        }

        /// Fish and marked fish in each 3 × 3 region, in the oracle's order
        /// (SW, S, SE, W, C, E, NW, N, NE).
        pub fn regions(&self) -> ([u32; 9], [u32; 9]) {
            let (mut all, mut marked) = ([0u32; 9], [0u32; 9]);
            for ((&x, &y), &m) in self.x.iter().zip(&self.y).zip(&self.marked) {
                let rx = ((x * 3.0) as usize).min(2);
                let ry = ((y * 3.0) as usize).min(2);
                all[rx + 3 * ry] += 1;
                if m {
                    marked[rx + 3 * ry] += 1;
                }
            }
            (all, marked)
        }

        /// Marked fish among the `net` fish nearest a point (squared
        /// distance, ties to the lower index).
        pub fn net_at(&self, p: (f64, f64), net: usize) -> u32 {
            self.net_members(p, net).iter().filter(|&&i| self.marked[i]).count() as u32
        }

        /// The `net` fish nearest a point (squared distance, ties to the
        /// lower index), nearest first: the fish a net there brings up.
        pub fn net_members(&self, (px, py): (f64, f64), net: usize) -> Vec<usize> {
            let d2 = |i: usize| squared(self.x[i] - px) + squared(self.y[i] - py);
            let mut order: Vec<usize> = (0..self.x.len()).collect();
            order.sort_by(|&a, &b| d2(a).total_cmp(&d2(b)).then(a.cmp(&b)));
            order.truncate(net);
            order
        }
    }

    /// The nine region centres, SW … NE.
    pub fn region_centres() -> [(f64, f64); 9] {
        let mut out = [(0.0, 0.0); 9];
        for (k, c) in out.iter_mut().enumerate() {
            *c = ((2 * (k % 3) + 1) as f64 / 6.0, (2 * (k / 3) + 1) as f64 / 6.0);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_matches_the_reference_sequence() {
        // Vigna's splitmix64.c from seed 1234567: the first outputs
        let mut r = SplitMix64::new(1_234_567);
        assert_eq!(r.next_u64(), 6_457_827_717_110_365_317);
        assert_eq!(r.next_u64(), 3_203_168_211_198_807_973);
        assert_eq!(r.next_u64(), 9_817_491_932_198_370_423);
    }

    #[test]
    fn the_lake_is_the_oracles_fish_for_fish() {
        let lake = Lake::draw();
        assert_eq!(lake.x.len(), LAKE_FISH);
        assert_eq!(lake.candidates, 139_559, "A-F27.rejection_candidates");
        assert_eq!(lake.all_counts().sum::<u32>(), 10_000);
        let spots: Vec<u32> = SPOTS.iter().map(|&(_, p)| lake.count_at(p)).collect();
        assert_eq!(spots, [880, 28, 451], "weeds, open water, deep end (A-F27.spot.*)");
        assert_eq!(cell_of(0.25, 0.75), (2, 7));
        assert_eq!(cell_of(0.5, 0.5), (5, 5));
        assert_eq!(cell_of(0.75, 0.25), (7, 2));
        assert_eq!(lake.all_counts().min(), Some(12));
        assert_eq!(lake.all_counts().max(), Some(880));
        assert_eq!(lake.dips_within_2x(), 22);
        assert!(
            lake.counts_string()
                .starts_with("21,20,22,27,49,48,96,139,105,63;23,23,20,27,51,100,242,335,250,114;")
        );
    }

    #[test]
    fn habitat_is_one_in_open_water_and_peaks_in_the_wells() {
        // open water, far from both wells (the weed bed is 0.08 wide, the hole 0.12)
        assert!((habitat(1.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((habitat(0.25, 0.75) - 41.0).abs() < 1e-3);
        assert!((habitat(0.75, 0.25) - 21.0).abs() < 1e-3);
        assert!(habitat(0.25, 0.75) < S_MAX);
    }
    #[test]
    fn the_mixing_noise_is_the_oracles_bit_for_bit() {
        // A-CH03.portable.normal_bits_step0_fish{0,1}_x_y
        let (x0, y0) = mixing::normals(0, 0, LAKE_FISH as u64);
        let (x1, y1) = mixing::normals(0, 1, LAKE_FISH as u64);
        assert_eq!(
            (x0.to_bits(), y0.to_bits()),
            (4_593_401_984_071_895_964, 13_831_029_455_220_284_115)
        );
        assert_eq!(
            (x1.to_bits(), y1.to_bits()),
            (13_833_251_294_890_088_072, 4_603_129_941_165_871_377)
        );
    }

    #[test]
    fn the_marks_spread_as_in_the_oracle() {
        let lake = Lake::draw();
        let mut m = mixing::Mixing::start(&lake);
        let (all, marked) = m.regions();
        assert_eq!(
            all,
            [232, 1027, 2531, 712, 575, 1030, 2964, 667, 262],
            "A-CH03.portable.t0.count"
        );
        assert_eq!(marked, [33, 677, 290, 0, 0, 0, 0, 0, 0], "A-CH03.portable.t0.marked");
        m.advance(1);
        // one step of drift involves exp(): equal bits unless the platform's
        // exp() differs from the oracle's in the last place
        assert_eq!(
            (m.x[0].to_bits(), m.y[0].to_bits()),
            (4_605_997_233_781_696_083, 4_597_239_076_297_835_640),
            "A-CH03.portable.position_bits_step1_fish0_x_y"
        );
        m.advance(2_500 - 1);
        assert_eq!(m.regions().0, [278, 1057, 2681, 695, 556, 1044, 2772, 708, 209], "t25");
        m.advance(mixing::STEPS - 2_500);
        let (all, marked) = m.regions();
        assert_eq!(
            all,
            [250, 1114, 2811, 689, 580, 1003, 2690, 659, 204],
            "A-CH03.portable.t400.count"
        );
        assert_eq!(
            marked,
            [24, 105, 282, 73, 51, 105, 278, 57, 25],
            "A-CH03.portable.t400.marked"
        );
        let nets: Vec<u32> = mixing::region_centres().iter().map(|&c| m.net_at(c, 1_000)).collect();
        assert_eq!(
            nets,
            [90, 93, 103, 100, 94, 100, 105, 100, 99],
            "A-CH03.portable.nets1000"
        );
        // a net's members: the thousand nearest, nearest first, and no fish
        // left in the lake is nearer than the farthest one caught
        let c = mixing::region_centres()[4];
        let members = m.net_members(c, 1_000);
        let d2 = |i: usize| (m.x[i] - c.0).powi(2) + (m.y[i] - c.1).powi(2);
        assert_eq!(members.len(), 1_000);
        assert!(members.windows(2).all(|w| d2(w[0]) <= d2(w[1])));
        let far = d2(members[999]);
        let inside: std::collections::HashSet<usize> = members.iter().copied().collect();
        assert!((0..m.x.len()).all(|i| inside.contains(&i) || d2(i) >= far));
    }
}
