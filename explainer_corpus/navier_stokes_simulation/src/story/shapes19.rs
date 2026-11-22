//! Chapter 19's numbers and teaching geometry, std only and exact wherever the value is rational:
//! the D2Q9 lattice and its moments; the illustrative cell of 19_1 and one exact step of the
//! 5 × 5 patch in 19_2; the Taylor–Green formula and the prototype's measured ratio as its run
//! printed it (19_4); the pressure–density rule (19_5); the Mach number and the illustrative
//! squeeze (19_7); and the shear layer under the eddy-viscosity tint (19_9).
//!
//! Three kinds of value live here, and the scene labels each kind:
//! - exact facts of the method: the lattice, the weights, the equilibrium and its moments,
//!   p = ρ/3, ν = (τ − ½)/3, Ma = U/c_s;
//! - illustrations: the cell's amounts, the patch, the exaggerated densities, the slider's speeds,
//!   ½Ma² as a pressure scale, the shear layer and the swirls;
//! - one measured value, [`taylor_green::RATIO`], copied from RubyRaven's retained numpy
//!   prototype run `lbm_tg01-run03.txt` (not the film's solver).
//!
//! RubyRaven `ruby-lattice01` for root's `navier_stokes_simulation` crate (Agent Mail 36272).
//! `tools/check_lattice01.py` recomputes every value in Python Fractions and binds the measured
//! ratio and the run's settings to the run file by its sha256.

/// A fraction: numerator over a positive denominator, in lowest terms.
pub type Q = (i128, i128);

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

pub fn q(n: i128, d: i128) -> Q {
    assert!(d != 0, "zero denominator");
    let g = gcd(n, d).max(1);
    let s = if d < 0 { -1 } else { 1 };
    (s * n / g, s * d / g)
}

pub fn int(n: i128) -> Q {
    (n, 1)
}

pub fn add(a: Q, b: Q) -> Q {
    q(a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}

pub fn sub(a: Q, b: Q) -> Q {
    q(a.0 * b.1 - b.0 * a.1, a.1 * b.1)
}

pub fn mul(a: Q, b: Q) -> Q {
    q(a.0 * b.0, a.1 * b.1)
}

pub fn div(a: Q, b: Q) -> Q {
    q(a.0 * b.1, a.1 * b.0)
}

pub fn to_f64(a: Q) -> f64 {
    a.0 as f64 / a.1 as f64
}

fn fixed(k: i128, places: u32) -> String {
    if places == 0 {
        return k.to_string();
    }
    let s = 10i128.pow(places);
    format!("{}.{:0w$}", k / s, k % s, w = places as usize)
}

/// `x ≥ 0` rounded half up to `places` decimals, exactly.
pub fn decimal(x: Q, places: u32) -> String {
    assert!(x.0 >= 0, "decimal takes x ≥ 0");
    let scale = 10i128.pow(places);
    fixed((2 * x.0 * scale + x.1) / (2 * x.1), places)
}

/// √x (x ≥ 0) rounded half up to `places` decimals, exactly: the k with
/// (2k − 1)² ≤ 4·x·10^(2·places) < (2k + 1)².
pub fn sqrt_decimal(x: Q, places: u32) -> String {
    assert!(x.0 >= 0, "sqrt_decimal takes x ≥ 0");
    let scale = 10i128.pow(places);
    let (n, d) = (x.0 * scale * scale, x.1);
    let mut k = ((n as f64 / d as f64).sqrt()) as i128;
    while (2 * k + 1) * (2 * k + 1) * d <= 4 * n {
        k += 1;
    }
    while k > 0 && (2 * k - 1) * (2 * k - 1) * d > 4 * n {
        k -= 1;
    }
    fixed(k, places)
}

/// A ratio of the chapter's comparisons in words: twice, half, four times, a quarter of.
pub fn times_words(x: Q) -> &'static str {
    match x {
        (2, 1) => "twice",
        (1, 2) => "half",
        (4, 1) => "four times",
        (1, 4) => "a quarter of",
        _ => panic!("{x:?} has no words here"),
    }
}

/// A terminating fraction (x ≥ 0) written out in full, without trailing zeros.
pub fn exact_decimal(x: Q) -> String {
    for places in 0..=9 {
        let scale = 10i128.pow(places);
        if (x.0 * scale) % x.1 == 0 {
            return fixed(x.0 * scale / x.1, places);
        }
    }
    panic!("{x:?} does not terminate within 9 places")
}

// ------------------------------------------------------------------ the lattice

pub mod d2q9 {
    use super::{Q, add, div, int, mul, q, sub};

    /// The nine lattice velocities, in the prototype's order (`lbm_tg01.py`): standing still,
    /// the four sides (east, north, west, south), the four corners (north-east, north-west,
    /// south-west, south-east).
    pub const C: [(i64, i64); 9] = [
        (0, 0),
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
        (1, -1),
    ];
    /// The weights in 36ths: 4/9 standing still, 1/9 to each side, 1/36 to each corner.
    pub const W36: [i64; 9] = [16, 4, 4, 4, 4, 1, 1, 1, 1];
    /// The weights' common denominator, and the unit of the illustrative amounts.
    pub const DENOM: i64 = 36;

    /// Nine amounts, one per lattice velocity.
    pub type Pops = [Q; 9];

    pub fn weight(i: usize) -> Q {
        q(W36[i] as i128, DENOM as i128)
    }

    pub fn from36(a: [i64; 9]) -> Pops {
        a.map(|x| q(x as i128, DENOM as i128))
    }

    /// The sum of the nine amounts.
    pub fn density(f: &Pops) -> Q {
        f.iter().fold(int(0), |s, &x| add(s, x))
    }

    /// The amounts weighted by their velocities: Σ f_i c_i.
    pub fn momentum(f: &Pops) -> (Q, Q) {
        f.iter().zip(C).fold((int(0), int(0)), |(mx, my), (&a, (cx, cy))| {
            (add(mx, mul(a, int(cx as i128))), add(my, mul(a, int(cy as i128))))
        })
    }

    pub fn velocity(f: &Pops) -> (Q, Q) {
        let (rho, (mx, my)) = (density(f), momentum(f));
        (div(mx, rho), div(my, rho))
    }

    /// Σ f_i c_ia c_ib.
    #[cfg(test)]
    pub fn second_moment(f: &Pops) -> [[Q; 2]; 2] {
        let mut m = [[int(0); 2]; 2];
        for (&a, (cx, cy)) in f.iter().zip(C) {
            let c = [cx as i128, cy as i128];
            for (ra, &ca) in m.iter_mut().zip(&c) {
                for (cell, &cb) in ra.iter_mut().zip(&c) {
                    *cell = add(*cell, mul(a, int(ca * cb)));
                }
            }
        }
        m
    }

    /// The standard equilibrium: f_i^eq = w_i ρ (1 + 3 c_i·u + 9/2 (c_i·u)² − 3/2 |u|²).
    pub fn equilibrium(rho: Q, u: (Q, Q)) -> Pops {
        let uu = add(mul(u.0, u.0), mul(u.1, u.1));
        std::array::from_fn(|i| {
            let cu = add(mul(int(C[i].0 as i128), u.0), mul(int(C[i].1 as i128), u.1));
            let bracket = add(
                sub(add(int(1), mul(int(3), cu)), mul(q(3, 2), uu)),
                mul(q(9, 2), mul(cu, cu)),
            );
            mul(mul(weight(i), rho), bracket)
        })
    }

    /// The balanced mix a cell relaxes toward: the equilibrium at its own density and velocity.
    pub fn balanced(f: &Pops) -> Pops {
        equilibrium(density(f), velocity(f))
    }

    /// BGK: f_i* = f_i − (f_i − f_i^eq)/τ.
    pub fn collide(f: &Pops, tau: Q) -> Pops {
        let e = balanced(f);
        std::array::from_fn(|i| sub(f[i], div(sub(f[i], e[i]), tau)))
    }

    /// ν = (τ − ½)/3, lattice units.
    pub fn viscosity(tau: Q) -> Q {
        div(sub(tau, q(1, 2)), int(3))
    }

    /// c_s² = 1/3: Σ w_i c_ia c_ib = δ_ab / 3.
    pub const SOUND_SPEED_SQ: Q = (1, 3);
}

// ------------------------------------------------------------------ 19_1 and 19_3: one cell

/// The cell of 19_1 and 19_3: nine illustrative amounts, in 36ths.
pub mod hero {
    use super::d2q9::{self, C, Pops};
    use super::{decimal, div};

    pub const AMOUNTS: [i64; 9] = [16, 6, 4, 3, 4, 2, 1, 1, 2];

    pub fn pops() -> Pops {
        d2q9::from36(AMOUNTS)
    }

    /// "16 + 6 + … = 39": the density, in 36ths.
    pub fn sum_label() -> String {
        let terms: Vec<String> = AMOUNTS.iter().map(|a| a.to_string()).collect();
        format!("{} = {}", terms.join(" + "), AMOUNTS.iter().sum::<i64>())
    }

    /// The momentum's x (axis 0) or y (axis 1) part, term by term, in 36ths.
    pub fn axis_label(axis: usize) -> String {
        let mut out = String::new();
        let mut total = 0;
        for (i, &a) in AMOUNTS.iter().enumerate() {
            let c = if axis == 0 { C[i].0 } else { C[i].1 };
            if c == 0 {
                continue;
            }
            total += c * a;
            out += &match (out.is_empty(), c > 0) {
                (true, true) => a.to_string(),
                (true, false) => format!("−{a}"),
                (false, true) => format!(" + {a}"),
                (false, false) => format!(" − {a}"),
            };
        }
        let name = ["x", "y"][axis];
        let total = if total < 0 {
            format!("−{}", -total)
        } else {
            total.to_string()
        };
        format!("{name}: {out} = {total}")
    }

    /// "illustrative amounts, in 36ths".
    pub fn amounts_tag() -> String {
        format!("illustrative amounts, in {}ths", d2q9::DENOM)
    }

    /// The velocity, momentum ÷ density, as a fraction and to two places.
    pub fn velocity_label() -> String {
        let f = pops();
        let (mx, rho) = (d2q9::momentum(&f).0, d2q9::density(&f));
        let u = div(mx, rho);
        format!(
            "velocity = momentum ÷ density = {}/{} ≈ {} cells per step",
            u.0,
            u.1,
            decimal(u, 2)
        )
    }
}

// ------------------------------------------------------------------ 19_2: one exact step

/// The 5 × 5 patch of 19_2: every cell holds the resting mix (the weights), except the middle
/// one, which holds the hero cell's amounts. One stream, then one collide at τ = 2, all exact.
pub mod patch {
    use super::d2q9::{self, C, Pops, W36};
    use super::{Q, hero};

    /// The window is −HALF..=HALF cells each way; the ring around it feeds the edge cells.
    pub const HALF: i64 = 2;
    /// The illustration's relaxation time: each amount moves halfway to its target.
    pub const TAU: Q = (2, 1);

    pub fn before(x: i64, y: i64) -> Pops {
        if (x, y) == (0, 0) {
            hero::pops()
        } else {
            d2q9::from36(W36)
        }
    }

    /// Stream: f_i(x + c_i, t + 1) = f_i*(x, t), so after the hop cell x holds amount i from
    /// x − c_i.
    pub fn after_stream(x: i64, y: i64) -> Pops {
        std::array::from_fn(|i| before(x - C[i].0, y - C[i].1)[i])
    }

    pub fn after_collide(x: i64, y: i64) -> Pops {
        d2q9::collide(&after_stream(x, y), TAU)
    }

    /// Whether amount i of cell (x, y), after the hop, is one of the hero cell's.
    #[cfg(test)]
    pub fn from_hero(x: i64, y: i64, i: usize) -> bool {
        (x - C[i].0, y - C[i].1) == (0, 0)
    }

    /// The patch's τ, and what it does: each amount moves 1/τ of the way to its balanced mix.
    pub fn tau_tex() -> String {
        let share = match super::div((1, 1), TAU) {
            (1, 2) => "halfway",
            other => panic!("no words for a share of {other:?}"),
        };
        format!(
            r"\tau = {}\ \text{{here: each amount moves {share}}}",
            super::exact_decimal(TAU)
        )
    }

    pub fn window() -> Vec<(i64, i64)> {
        let span = -HALF..=HALF;
        span.clone().flat_map(|y| span.clone().map(move |x| (x, y))).collect()
    }

    /// The window cells whose amounts the collide changes (the others already hold their
    /// balanced mix).
    pub fn changed() -> Vec<(i64, i64)> {
        window()
            .into_iter()
            .filter(|&(x, y)| after_collide(x, y) != after_stream(x, y))
            .collect()
    }
}

// ------------------------------------------------------------------ how a cell's amounts are drawn

/// A cell's star, in cell widths from its centre: amount i ≠ 0 is an arrow along c_i whose
/// vector is ARROW·f_i·c_i (so arrows add tip to tail to ARROW × momentum), starting R0 out;
/// the resting amount is a dot whose area is proportional to it.
pub mod star {
    use super::d2q9::C;

    pub const R0: f64 = 0.14;
    pub const ARROW: f64 = 1.95;
    pub const DOT: f64 = 0.16;

    pub fn dot_radius(a: f64) -> f64 {
        DOT * a.sqrt()
    }

    /// Amount i's arrow vector, cell widths.
    pub fn vector(i: usize, a: f64) -> (f64, f64) {
        (ARROW * a * C[i].0 as f64, ARROW * a * C[i].1 as f64)
    }

    /// Amount i's arrow: tail and tip, cell widths from the centre.
    pub fn arrow(i: usize, a: f64) -> [(f64, f64); 2] {
        let (cx, cy) = (C[i].0 as f64, C[i].1 as f64);
        let len = cx.hypot(cy);
        let (ux, uy) = (cx / len, cy / len);
        let (vx, vy) = vector(i, a);
        [(R0 * ux, R0 * uy), (R0 * ux + vx, R0 * uy + vy)]
    }

    /// The open head's size for an arrow `l` cell widths long.
    pub fn head(l: f64) -> f64 {
        (0.4 * l).min(0.07)
    }
}

// ------------------------------------------------------------------ 19_4: the retained check

/// The prototype check of 19_4: the formula drawn, and the measured ratio as its run printed it.
pub mod taylor_green {
    use super::{Q, d2q9, q, sub};
    use std::f64::consts::TAU as TURN;

    /// The prototype's grid, cells each way (run03's last row, N 128).
    pub const N: i64 = 128;
    /// The prototype's relaxation time (run03's header: "tau 0.8, nu 0.100000, Re 10.0").
    pub const TAU: Q = (4, 5);
    /// run03, N 128, both starts: "decay rate / 2nu k^2 = 1.00006". MEASURED.
    pub const RATIO: &str = "1.00006";

    /// The measured ratio as a fraction (its printed decimals).
    pub fn ratio() -> Q {
        let (whole, frac) = RATIO.split_once('.').expect("RATIO has a decimal point");
        let scale = 10i128.pow(frac.len() as u32);
        q(
            whole.parse::<i128>().unwrap() * scale + frac.parse::<i128>().unwrap(),
            scale,
        )
    }

    /// "within 0.006%": (ratio − 1) × 100, as printed.
    pub fn within_label() -> String {
        let pct = super::mul(sub(ratio(), (1, 1)), (100, 1));
        format!("within {}%", super::exact_decimal(pct))
    }

    /// "measured rate / predicted = 1.00006" (19_4's picture line).
    pub fn ratio_label() -> String {
        format!("measured rate / predicted = {RATIO}")
    }

    /// The prototype's τ and the viscosity it sets, in TeX.
    pub fn nu_tex() -> String {
        let nu = d2q9::viscosity(TAU);
        format!(
            r"\text{{prototype: }}\tau = {},\ \text{{so }}\nu = {}\ \text{{(lattice units)}}",
            super::exact_decimal(TAU),
            super::exact_decimal(nu)
        )
    }

    pub fn grid_label() -> String {
        format!("{N} × {N}")
    }

    /// "diagnostic prototype (numpy), not the film's solver; 128 × 128" (19_4's picture line).
    pub fn proto_label() -> String {
        format!("diagnostic prototype (numpy), not the film's solver; {}", grid_label())
    }

    /// Arrows per side over one period of the pattern.
    pub const SAMPLES: usize = 8;

    /// The pattern's velocity, u = (sin x cos y, −cos x sin y), x and y in radians; its decay,
    /// e^(−2νk²t), is [`amplitude`].
    pub fn velocity(x: f64, y: f64) -> (f64, f64) {
        (x.sin() * y.cos(), -(x.cos() * y.sin()))
    }

    /// Sample point m's coordinate, radians.
    pub fn sample(m: usize) -> f64 {
        (m as f64 + 0.5) * TURN / SAMPLES as f64
    }

    /// The predicted amplitude after `s` e-foldings of the decay.
    pub fn amplitude(s: f64) -> f64 {
        (-s).exp()
    }
}

// ------------------------------------------------------------------ 19_5: pressure from density

/// 19_5's two neighbouring cells, densities 1 ± δ (exaggerated); at twice the speed the same
/// pattern's pressure differences, so its density differences, are four times larger.
pub mod squeeze_cells {
    use super::{Q, add, int, mul, sub};

    pub const SPLIT: [Q; 2] = [(1, 10), (2, 5)];
    /// The second picture's speed, in multiples of the first's.
    pub const SPEED_UP: i64 = 2;

    pub fn densities(k: usize) -> (Q, Q) {
        (add(int(1), SPLIT[k]), sub(int(1), SPLIT[k]))
    }

    #[cfg(test)]
    pub fn pressure(rho: Q) -> Q {
        mul(rho, super::d2q9::SOUND_SPEED_SQ)
    }

    /// "ρ = 1.1", in TeX.
    pub fn density_tex(rho: Q) -> String {
        format!(r"\rho = {}", super::exact_decimal(rho))
    }

    /// The flow arrow's label at `k` times the first speed: "speed U", "speed 2U".
    pub fn speed_label(k: i64) -> String {
        if k == 1 {
            "speed U".to_owned()
        } else {
            format!("speed {k}U")
        }
    }

    /// The faster picture against the first, in words, from the numbers.
    pub fn ratio_label() -> String {
        let grows = super::div(SPLIT[1], SPLIT[0]);
        assert_eq!(
            grows,
            mul(int(SPEED_UP as i128), int(SPEED_UP as i128)),
            "the split grows as the speed squared"
        );
        format!(
            "{} the speed, same pattern: {} the pressure difference",
            super::times_words(int(SPEED_UP as i128)),
            super::times_words(grows)
        )
    }
}

// ------------------------------------------------------------------ 19_7: the squeeze

/// 19_7's slider: speed in cells per step, Ma = U/c_s, and ½Ma² as an illustrative density
/// difference (Δρ/ρ = (Δp/ρU²)·Ma² exactly; ½ρU² is only a pressure scale).
pub mod mach {
    use super::{Q, d2q9, decimal, div, exact_decimal, mul, q, sqrt_decimal};

    /// The example: SOLVER's U_max for its 2D-2 design (not a run).
    pub const EXAMPLE: Q = (1, 20);
    pub const FASTER: Q = (1, 10);
    pub const SLOWER: Q = (1, 40);
    /// The slider's end.
    pub const TOP: Q = (1, 10);
    pub const TICKS: [Q; 3] = [(0, 1), (1, 20), (1, 10)];

    pub fn ma_sq(u: Q) -> Q {
        div(mul(u, u), d2q9::SOUND_SPEED_SQ)
    }

    pub fn squeeze(u: Q) -> Q {
        mul(q(1, 2), ma_sq(u))
    }

    pub fn speed_label(u: Q) -> String {
        format!("U = {} cells per step", exact_decimal(u))
    }

    pub fn ma_label(u: Q) -> String {
        format!("Ma ≈ {}", sqrt_decimal(ma_sq(u), 3))
    }

    /// "Δρ/ρ ≈ 0.004", in TeX.
    pub fn squeeze_tex(u: Q) -> String {
        format!(r"\Delta\rho/\rho \approx {}", decimal(squeeze(u), 3))
    }

    /// A speed against the example, and its squeeze against the example's, in words.
    pub fn compare_label(u: Q) -> String {
        format!(
            "{} the example speed: {} its squeeze",
            super::times_words(div(u, EXAMPLE)),
            super::times_words(div(squeeze(u), squeeze(EXAMPLE)))
        )
    }

    pub fn tick_label(u: Q) -> String {
        exact_decimal(u)
    }
}

// ------------------------------------------------------------------ 19_9: the shear layer

/// 19_9's resolved flow: a shear layer u(y) = tanh(y/δ) across a coarse grid, its shear
/// |du/dy| = sech²(y/δ)/δ, and the eddy-viscosity tint, proportional to the shear (ν_t ∝ |S|
/// with one grid spacing). The small swirls are schematic, placed by hand.
pub mod shear {
    /// The layer's half-width, scene units.
    pub const DELTA: f64 = 0.7;
    pub const COLS: usize = 8;
    pub const ROWS: usize = 5;
    pub const CELL: (f64, f64) = (1.0, 1.04);
    /// The strongest tint's opacity.
    pub const TINT: f64 = 0.45;
    /// Swirl centres (from the panel's centre) and radius, scene units: schematic.
    pub const SWIRLS: [(f64, f64); 10] = [
        (-3.55, 0.2),
        (-2.75, -0.25),
        (-1.95, 0.3),
        (-1.15, -0.15),
        (-0.35, 0.25),
        (0.45, -0.3),
        (1.25, 0.15),
        (2.05, -0.2),
        (2.85, 0.3),
        (3.55, -0.1),
    ];
    pub const SWIRL_R: f64 = 0.2;

    pub fn speed(y: f64) -> f64 {
        (y / DELTA).tanh()
    }

    pub fn shear(y: f64) -> f64 {
        let c = (y / DELTA).cosh();
        1.0 / (DELTA * c * c)
    }

    /// Row r's centre, from the panel's centre (row 0 at the top).
    pub fn row_y(r: usize) -> f64 {
        CELL.1 * ((ROWS as f64 - 1.0) / 2.0 - r as f64)
    }

    pub fn col_x(c: usize) -> f64 {
        CELL.0 * (c as f64 - (COLS as f64 - 1.0) / 2.0)
    }

    /// Row r's tint opacity: TINT × its shear over the largest row's.
    pub fn tint(r: usize) -> f64 {
        let top = (0..ROWS).map(|k| shear(row_y(k))).fold(0.0, f64::max);
        TINT * shear(row_y(r)) / top
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use d2q9::{C, Pops, W36};

    fn samples() -> [(Q, (Q, Q)); 3] {
        [
            (int(1), (q(1, 10), int(0))),
            (q(6, 5), (q(-1, 20), q(1, 25))),
            (q(19, 18), (q(1, 19), int(0))),
        ]
    }

    #[test]
    fn lattice_moments_are_exact() {
        let w: Pops = std::array::from_fn(d2q9::weight);
        assert_eq!(d2q9::density(&w), int(1));
        assert_eq!(d2q9::momentum(&w), (int(0), int(0)));
        assert_eq!(
            d2q9::second_moment(&w),
            [[d2q9::SOUND_SPEED_SQ, int(0)], [int(0), d2q9::SOUND_SPEED_SQ]]
        );
        assert_eq!(W36.iter().sum::<i64>(), 36);
        for (rho, u) in samples() {
            let e = d2q9::equilibrium(rho, u);
            assert_eq!(d2q9::density(&e), rho);
            assert_eq!(d2q9::momentum(&e), (mul(rho, u.0), mul(rho, u.1)));
            let m = d2q9::second_moment(&e);
            let uu = [[mul(u.0, u.0), mul(u.0, u.1)], [mul(u.1, u.0), mul(u.1, u.1)]];
            for a in 0..2 {
                for b in 0..2 {
                    let iso = if a == b { mul(rho, d2q9::SOUND_SPEED_SQ) } else { int(0) };
                    assert_eq!(m[a][b], add(iso, mul(rho, uu[a][b])), "{a}{b}");
                }
            }
        }
        assert_eq!(d2q9::viscosity(taylor_green::TAU), q(1, 10));
        assert_eq!(d2q9::viscosity(q(27, 50)), q(1, 75));
    }

    #[test]
    fn hero_cell_reads_off_density_and_momentum() {
        let f = hero::pops();
        assert_eq!(d2q9::density(&f), q(39, 36));
        assert_eq!(d2q9::momentum(&f), (q(5, 36), int(0)));
        assert_eq!(hero::sum_label(), "16 + 6 + 4 + 3 + 4 + 2 + 1 + 1 + 2 = 39");
        assert_eq!(hero::axis_label(0), "x: 6 − 3 + 2 − 1 − 1 + 2 = 5");
        assert_eq!(hero::axis_label(1), "y: 4 − 4 + 2 + 1 − 1 − 2 = 0");
        assert_eq!(
            hero::velocity_label(),
            "velocity = momentum ÷ density = 5/39 ≈ 0.13 cells per step"
        );
        assert_eq!(hero::amounts_tag(), "illustrative amounts, in 36ths");
        // the tip-to-tail chain of the eight arrows ends at ARROW × momentum
        let (mut x, mut y) = (0.0, 0.0);
        for (i, &a) in f.iter().enumerate().skip(1) {
            let (vx, vy) = star::vector(i, to_f64(a));
            (x, y) = (x + vx, y + vy);
        }
        assert!((x - star::ARROW * 5.0 / 36.0).abs() < 1e-12 && y.abs() < 1e-12);
    }

    #[test]
    fn one_exact_step_of_the_patch() {
        // the background is the resting mix, so it is balanced and unchanged by a step
        let rest = d2q9::from36(W36);
        assert_eq!(d2q9::balanced(&rest), rest);
        // after the hop the hero's amounts sit in its eight neighbours, its rest at home
        for (i, &(cx, cy)) in C.iter().enumerate() {
            assert!(patch::from_hero(cx, cy, i));
            assert_eq!(patch::after_stream(cx, cy)[i], hero::pops()[i]);
        }
        assert_eq!(patch::after_stream(0, 0), rest);
        // the collide keeps each cell's density and momentum and moves halfway to the balance
        for (x, y) in patch::window() {
            let (a, b) = (patch::after_stream(x, y), patch::after_collide(x, y));
            assert_eq!(d2q9::density(&a), d2q9::density(&b));
            assert_eq!(d2q9::momentum(&a), d2q9::momentum(&b));
            let e = d2q9::balanced(&a);
            for i in 0..9 {
                assert_eq!(sub(b[i], a[i]), mul(q(1, 2), sub(e[i], a[i])));
            }
        }
        assert_eq!(patch::changed(), vec![(1, -1), (-1, 0), (1, 0), (1, 1)]);
        assert_eq!(patch::tau_tex(), r"\tau = 2\ \text{here: each amount moves halfway}");
        // the east neighbour: density 38/36, momentum (2/36, 0)
        let e = patch::after_stream(1, 0);
        assert_eq!((d2q9::density(&e), d2q9::momentum(&e)), (q(19, 18), (q(1, 18), int(0))));
    }

    #[test]
    fn every_star_fits_its_cell() {
        let mut stars: Vec<Pops> = vec![hero::pops()];
        for (x, y) in patch::window() {
            stars.extend([
                patch::before(x, y),
                patch::after_stream(x, y),
                patch::after_collide(x, y),
            ]);
            stars.push(d2q9::balanced(&patch::after_stream(x, y)));
        }
        for k in 0..2 {
            let (hi, lo) = squeeze_cells::densities(k);
            stars.push(d2q9::equilibrium(hi, (int(0), int(0))));
            stars.push(d2q9::equilibrium(lo, (int(0), int(0))));
        }
        for f in &stars {
            assert!(f.iter().all(|a| a.0 > 0), "{f:?}");
            assert!(star::dot_radius(to_f64(f[0])) < star::R0 - 0.01);
            for (i, &a) in f.iter().enumerate().skip(1) {
                let [_, tip] = star::arrow(i, to_f64(a));
                assert!(tip.0.abs() <= 0.475 && tip.1.abs() <= 0.475, "{i} {tip:?}");
            }
        }
    }

    #[test]
    fn decimals_round_exactly() {
        assert_eq!(decimal(q(3, 800), 3), "0.004");
        assert_eq!(decimal(q(3, 200), 3), "0.015");
        assert_eq!(decimal(q(3, 3200), 3), "0.001");
        assert_eq!(sqrt_decimal(q(3, 400), 3), "0.087");
        assert_eq!(sqrt_decimal(q(3, 100), 3), "0.173");
        assert_eq!(sqrt_decimal(q(3, 1600), 3), "0.043");
        assert_eq!(sqrt_decimal(q(1, 4), 3), "0.500");
        assert_eq!(exact_decimal(q(1, 40)), "0.025");
        assert_eq!(exact_decimal(int(0)), "0");
    }

    #[test]
    fn the_squeeze_grows_with_the_square_of_the_speed() {
        assert_eq!(mach::ma_sq(mach::EXAMPLE), q(3, 400));
        assert_eq!(mach::squeeze(mach::EXAMPLE), q(3, 800));
        assert_eq!(mach::FASTER, mul(int(2), mach::EXAMPLE));
        assert_eq!(mach::SLOWER, mul(q(1, 2), mach::EXAMPLE));
        assert_eq!(mach::squeeze(mach::FASTER), mul(int(4), mach::squeeze(mach::EXAMPLE)));
        assert_eq!(mach::squeeze(mach::SLOWER), mul(q(1, 4), mach::squeeze(mach::EXAMPLE)));
        assert_eq!(mach::speed_label(mach::EXAMPLE), "U = 0.05 cells per step");
        assert_eq!(mach::ma_label(mach::EXAMPLE), "Ma ≈ 0.087");
        assert_eq!(mach::squeeze_tex(mach::EXAMPLE), r"\Delta\rho/\rho \approx 0.004");
        assert_eq!(mach::speed_label(mach::FASTER), "U = 0.1 cells per step");
        assert_eq!(mach::speed_label(mach::SLOWER), "U = 0.025 cells per step");
        assert_eq!(mach::ma_label(mach::FASTER), "Ma ≈ 0.173");
        assert_eq!(mach::squeeze_tex(mach::FASTER), r"\Delta\rho/\rho \approx 0.015");
        assert_eq!(mach::ma_label(mach::SLOWER), "Ma ≈ 0.043");
        assert_eq!(mach::squeeze_tex(mach::SLOWER), r"\Delta\rho/\rho \approx 0.001");
        assert_eq!(
            mach::compare_label(mach::FASTER),
            "twice the example speed: four times its squeeze"
        );
        assert_eq!(
            mach::compare_label(mach::SLOWER),
            "half the example speed: a quarter of its squeeze"
        );
        assert_eq!(mach::TICKS.map(mach::tick_label), ["0", "0.05", "0.1"]);
        // 19_5's cells: twice the speed, four times the split; p = ρ/3
        assert_eq!(squeeze_cells::SPLIT[1], mul(int(4), squeeze_cells::SPLIT[0]));
        assert_eq!(
            squeeze_cells::ratio_label(),
            "twice the speed, same pattern: four times the pressure difference"
        );
        assert_eq!(squeeze_cells::density_tex(squeeze_cells::densities(1).1), r"\rho = 0.6");
        assert_eq!(squeeze_cells::speed_label(1), "speed U");
        assert_eq!(squeeze_cells::speed_label(squeeze_cells::SPEED_UP), "speed 2U");
        let (hi, lo) = squeeze_cells::densities(0);
        assert_eq!(
            sub(squeeze_cells::pressure(hi), squeeze_cells::pressure(lo)),
            mul(q(1, 3), sub(hi, lo))
        );
    }

    #[test]
    fn retained_check_labels() {
        assert_eq!(taylor_green::ratio(), q(100_006, 100_000));
        assert_eq!(taylor_green::within_label(), "within 0.006%");
        assert_eq!(
            taylor_green::nu_tex(),
            r"\text{prototype: }\tau = 0.8,\ \text{so }\nu = 0.1\ \text{(lattice units)}"
        );
        assert_eq!(taylor_green::ratio_label(), "measured rate / predicted = 1.00006");
        assert_eq!(
            taylor_green::proto_label(),
            "diagnostic prototype (numpy), not the film's solver; 128 × 128"
        );
        // the formula: four counter-rotating swirls, divergence-free, decaying as e^(−s)
        let (x, y) = (0.3, 1.1);
        let h = 1e-6;
        let div = (taylor_green::velocity(x + h, y).0 - taylor_green::velocity(x - h, y).0
            + taylor_green::velocity(x, y + h).1
            - taylor_green::velocity(x, y - h).1)
            / (2.0 * h);
        assert!(div.abs() < 1e-8);
        assert!((taylor_green::amplitude(1.0) - (-1.0f64).exp()).abs() < 1e-15);
    }

    #[test]
    fn the_tint_follows_the_shear() {
        let tints: Vec<f64> = (0..shear::ROWS).map(shear::tint).collect();
        assert!((tints[2] - shear::TINT).abs() < 1e-12);
        assert!(tints[1] < tints[2] && tints[0] < tints[1] && (tints[1] - tints[3]).abs() < 1e-12);
        assert!(shear::speed(shear::row_y(0)) > 0.99 && shear::speed(shear::row_y(4)) < -0.99);
        for &(x, y) in &shear::SWIRLS {
            assert!(y.abs() + shear::SWIRL_R < shear::CELL.1 / 2.0 && x.abs() + shear::SWIRL_R < 4.0);
        }
    }
}
