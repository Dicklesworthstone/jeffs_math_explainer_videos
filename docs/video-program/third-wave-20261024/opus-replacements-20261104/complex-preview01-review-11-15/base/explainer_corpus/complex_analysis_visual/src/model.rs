//! The mathematics behind every picture and number of the film: complex
//! arithmetic, the functions the film draws (with their domains), the
//! enhanced phase portrait's colours, loop sums, winding counts, the bump's
//! power series about any real centre, and the two residue integrals.
//!
//! Every function returns `None` where it is undefined (a pole, a point
//! outside its region, an overflow) instead of a misleading number: the
//! pictures leave those cells dark and mark poles separately. Nothing here
//! draws; the chapters and `facts.rs` read from it. `oracle.py` recomputes
//! the printed values independently (Python, other algorithms).

use std::cmp::Ordering;
use std::f64::consts::{PI, TAU};
use std::ops::{Add, Div, Mul, Neg, Sub};

// ------------------------------------------------------------ the numbers

/// A complex number `re + i·im`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

/// i.
pub const I: C = C::new(0.0, 1.0);
/// 1.
pub const ONE: C = C::new(1.0, 0.0);
/// 0.
pub const ZERO: C = C::new(0.0, 0.0);

impl C {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub const fn real(x: f64) -> Self {
        Self::new(x, 0.0)
    }

    /// The point at length `r` and angle `theta` (radians).
    pub fn polar(r: f64, theta: f64) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    /// The length |z|.
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// The angle in (−π, π].
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    pub fn scale(self, k: f64) -> Self {
        Self::new(self.re * k, self.im * k)
    }

    pub fn recip(self) -> Self {
        let d = self.norm_sqr();
        Self::new(self.re / d, -self.im / d)
    }

    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    pub fn exp(self) -> Self {
        Self::polar(self.re.exp(), self.im)
    }

    /// The principal logarithm (angle in (−π, π]).
    pub fn ln(self) -> Self {
        Self::new(self.abs().ln(), self.arg())
    }

    /// The principal square root.
    #[cfg(test)]
    pub fn sqrt(self) -> Self {
        Self::polar(self.abs().sqrt(), self.arg() / 2.0)
    }

    /// `self` to a whole power (negative powers through the reciprocal).
    pub fn powi(self, n: i32) -> Self {
        let mut out = ONE;
        let base = if n < 0 { self.recip() } else { self };
        for _ in 0..n.unsigned_abs() {
            out = out * base;
        }
        out
    }

    /// `self^w` through the principal logarithm (used with positive real
    /// bases only, where it is unambiguous).
    pub fn powc(self, w: C) -> Self {
        (w * self.ln()).exp()
    }

    pub fn sin(self) -> Self {
        Self::new(self.re.sin() * self.im.cosh(), self.re.cos() * self.im.sinh())
    }

    pub fn cos(self) -> Self {
        Self::new(self.re.cos() * self.im.cosh(), -self.re.sin() * self.im.sinh())
    }

    /// tan z = (sin 2x + i sinh 2y) / (cos 2x + cosh 2y): no cancellation far
    /// from the real line, where it settles to ±i.
    pub fn tan(self) -> Self {
        let (x2, y2) = (2.0 * self.re, 2.0 * self.im);
        let d = x2.cos() + y2.cosh();
        Self::new(x2.sin() / d, y2.sinh() / d)
    }
}

impl Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
}

impl Sub for C {
    type Output = C;
    fn sub(self, o: C) -> C {
        C::new(self.re - o.re, self.im - o.im)
    }
}

impl Mul for C {
    type Output = C;
    fn mul(self, o: C) -> C {
        C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
}

impl Div for C {
    type Output = C;
    fn div(self, o: C) -> C {
        // a ÷ b is a × (1/b), through `Mul` itself so the two cannot drift apart
        Mul::mul(self, o.recip())
    }
}

impl Neg for C {
    type Output = C;
    fn neg(self) -> C {
        C::new(-self.re, -self.im)
    }
}

/// `Some(w)` when `w` is a finite number, else `None` (a pole, an overflow).
fn finite(w: C) -> Option<C> {
    w.is_finite().then_some(w)
}

/// Chapter 02's examples: the point drawn first, the number added to it,
/// and the multiplier applied twice.
pub const TURN_POINT: C = C::new(3.0, 2.0);
pub const TURN_ADDED: C = C::new(1.0, -1.0);
pub const TURN_BY: C = C::new(1.0, 1.0);

// ---------------------------------------------------- the recurring bump

/// The bump 1/(1 + x²) on the real line.
pub fn bump(x: f64) -> f64 {
    1.0 / (1.0 + x * x)
}

/// The recipe at zero up to x^`degree` (`degree` even): 1 − x² + x⁴ − ….
pub fn bump_recipe0(x: f64, degree: u32) -> f64 {
    (0..=degree / 2)
        .fold((0.0, 1.0), |(sum, term), _| (sum + term, -term * x * x))
        .0
}

/// The bump's power-series coefficients about the real centre `a`:
/// 1/(1 + z²) = (1/2i)(1/(z − i) − 1/(z + i)) and 1/(z − p) = −Σ (z − a)ᵏ/(p − a)ᵏ⁺¹
/// give cₖ = −Im[(i − a)^−(k+1)].
pub fn bump_coefficients(a: f64, n: usize) -> Vec<f64> {
    let q = (I - C::real(a)).recip();
    let mut power = q;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(-power.im);
        power = power * q;
    }
    out
}

/// The recipe about `a` with its first `terms` terms, at real `x`.
pub fn bump_recipe(a: f64, x: f64, terms: usize) -> f64 {
    bump_coefficients(a, terms)
        .iter()
        .rev()
        .fold(0.0, |acc, &c| acc * (x - a) + c)
}

/// How far the recipe about the real centre `a` reaches: the distance from
/// `a` to the poles at ±i.
pub fn bump_radius(a: f64) -> f64 {
    a.hypot(1.0)
}

/// The closing question's centre for the bump's recipe (storyboard 15_3).
pub const RECENTRE_AT: f64 = 3.0;

/// 01_3 reads the recipe and the bump here, just past the recipe's reach.
pub const BUMP_READ_AT: f64 = 1.1;

/// 09_3–09_5 average the bump over this many samples of a circle.
pub const AVERAGE_SAMPLES: [usize; 2] = [8, 16];
/// 09_10 checks the recipe centred at 1: this many terms, at these two points.
pub const RECENTRE_CHECK_TERMS: usize = 60;
pub const RECENTRE_CHECK_AT: [f64; 2] = [2.0, 2.6];

/// 07_2 shows a nonzero derivative keeping a right angle here (2z₁ ≠ 0).
pub const CONFORMAL_POINT: C = C::new(-0.7, 0.6);

/// 05_8 squashes the strip of ζ up to this height, above its fourth zero.
pub const ZETA_STRIP_TOP: f64 = 32.0;

/// 05's tangent reads this far up (and its conjugate this far down).
pub const TAN_FAR: C = C::new(0.4, 12.0);

// ------------------------------------------------------- special functions

/// Lanczos coefficients (g = 7, n = 9), the widely published set.
const LANCZOS_G: f64 = 7.0;
const LANCZOS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];

/// The gamma function, meromorphic on the plane with simple poles at
/// 0, −1, −2, …: Lanczos for Re z ≥ ½, the reflection
/// Γ(z)Γ(1 − z) = π / sin(πz) to the left. `None` at the poles.
pub fn gamma(z: C) -> Option<C> {
    // sin(πz) is not exactly zero at −1, −2, … in floating point: name the
    // poles instead of returning a huge finite number there
    if z.im == 0.0 && z.re <= 0.0 && z.re.fract() == 0.0 {
        return None;
    }
    if z.re < 0.5 {
        let s = (z.scale(PI)).sin();
        let g = gamma(ONE - z)?;
        return finite(C::real(PI) / (s * g));
    }
    let z = z - ONE;
    let mut x = C::real(LANCZOS[0]);
    for (k, &p) in LANCZOS.iter().enumerate().skip(1) {
        x = x + C::real(p) / (z + C::real(k as f64));
    }
    let t = z + C::real(LANCZOS_G + 0.5);
    let w = C::real((TAU).sqrt()) * t.powc(z + C::real(0.5)) * (-t).exp() * x;
    finite(w)
}

/// B₂, B₄, …, B₂₄ (the Bernoulli numbers the Euler–Maclaurin tail uses).
const BERNOULLI_EVEN: [f64; 12] = [
    1.0 / 6.0,
    -1.0 / 30.0,
    1.0 / 42.0,
    -1.0 / 30.0,
    5.0 / 66.0,
    -691.0 / 2730.0,
    7.0 / 6.0,
    -3617.0 / 510.0,
    43867.0 / 798.0,
    -174_611.0 / 330.0,
    854_513.0 / 138.0,
    -236_364_091.0 / 2730.0,
];
/// Terms summed directly before the Euler–Maclaurin tail.
const ZETA_N: u32 = 30;
/// Bernoulli corrections used.
const ZETA_M: usize = 10;

/// ζ by Euler–Maclaurin summation (Re s ≥ ½ in practice; accurate to about
/// 1e−12 for |Im s| ≤ 40 with N = 30, M = 10).
fn zeta_em(s: C) -> Option<C> {
    let n = f64::from(ZETA_N);
    let mut sum = ZERO;
    for k in 1..ZETA_N {
        sum = sum + C::real(f64::from(k)).powc(-s);
    }
    let n_pow = C::real(n).powc(-s); // N^−s
    sum = sum + (n_pow.scale(n)) / (s - ONE) + n_pow.scale(0.5);
    // term k: B₂ₖ/(2k)! · s(s+1)…(s+2k−2) · N^(−s−2k+1)
    let mut rising = s; // s for k = 1
    let mut factorial = 2.0; // (2k)!
    let mut n_power = n_pow.scale(1.0 / n); // N^(−s−1)
    for k in 1..=ZETA_M {
        sum = sum + (rising * n_power).scale(BERNOULLI_EVEN[k - 1] / factorial);
        let j = 2.0 * k as f64;
        rising = rising * (s + C::real(j - 1.0)) * (s + C::real(j));
        factorial *= (j + 1.0) * (j + 2.0);
        n_power = n_power.scale(1.0 / (n * n));
    }
    finite(sum)
}

/// The Riemann zeta function, continued to the plane with its one pole at
/// 1: Euler–Maclaurin for Re s ≥ ½, the functional equation
/// ζ(s) = 2ˢ πˢ⁻¹ sin(πs/2) Γ(1 − s) ζ(1 − s) to the left. `None` at 1.
pub fn zeta(s: C) -> Option<C> {
    if s.re >= 0.5 {
        return zeta_em(s);
    }
    let one_minus = ONE - s;
    let factor = C::real(2.0).powc(s) * C::real(PI).powc(s - ONE) * (s.scale(PI / 2.0)).sin();
    finite(factor * gamma(one_minus)? * zeta_em(one_minus)?)
}

/// A zero of ζ near `guess`, by Newton's method with a central difference.
pub fn zeta_zero_near(guess: C) -> Option<C> {
    let h = 1e-6;
    let mut s = guess;
    for _ in 0..60 {
        let f = zeta(s)?;
        let d = (zeta(s + C::real(h))? - zeta(s - C::real(h))?).scale(1.0 / (2.0 * h));
        let step = f / d;
        s = s - step;
        if step.abs() < 1e-14 {
            return Some(s);
        }
    }
    zeta(s).filter(|w| w.abs() < 1e-10).map(|_| s)
}

/// The first four nontrivial zeros' heights the gallery marks, as published
/// (A. Odlyzko's tables); the film's Newton refinement must land on them.
pub const ZETA_ZERO_HEIGHTS: [f64; 4] = [
    14.134_725_141_734_694,
    21.022_039_638_771_555,
    25.010_857_580_145_69,
    30.424_876_125_859_513,
];

// ------------------------------------------------- the functions drawn

/// Every function a picture of the film draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Func {
    /// f(z) = z, the colour wheel itself.
    Identity,
    Square,
    Recip,
    /// 1/(1 + z²).
    Bump,
    /// z³ − 1.
    Cubic,
    /// (z − 1)/(z + 1)²: a simple zero and a double pole.
    ZeroDoublePole,
    Exp,
    Sin,
    Tan,
    Gamma,
    Zeta,
    /// (z − 0.45)/(z − 1.35), the counting example.
    Moebius,
    /// z⁵ − 3z + 1.
    Quintic,
    /// |z|², complex differentiable at 0 only.
    AbsSquare,
    /// sin(1/z), holomorphic off 0.
    SinRecip,
    /// Σ z^(2ⁿ), n = 0 … 11, drawn only inside |z| < 0.98.
    Lacunary,
}

/// Where the counting example's zero and pole sit.
pub const MOEBIUS_ZERO: f64 = 0.45;
pub const MOEBIUS_POLE: f64 = 1.35;
/// Terms of the natural-boundary series drawn, and the radius it is drawn to.
pub const LACUNARY_TERMS: u32 = 12;
pub const LACUNARY_LIMIT: f64 = 0.98;

impl Func {
    /// f(z), or `None` where f is undefined or not drawn.
    pub fn eval(self, z: C) -> Option<C> {
        let w = match self {
            Func::Identity => z,
            Func::Square => z * z,
            Func::Recip => z.recip(),
            Func::Bump => (ONE + z * z).recip(),
            Func::Cubic => z * z * z - ONE,
            Func::ZeroDoublePole => (z - ONE) / ((z + ONE) * (z + ONE)),
            Func::Exp => z.exp(),
            Func::Sin => z.sin(),
            Func::Tan => z.tan(),
            Func::Gamma => return gamma(z),
            Func::Zeta => return zeta(z),
            Func::Moebius => (z - C::real(MOEBIUS_ZERO)) / (z - C::real(MOEBIUS_POLE)),
            Func::Quintic => z.powi(5) - z.scale(3.0) + ONE,
            Func::AbsSquare => C::real(z.norm_sqr()),
            Func::SinRecip => z.recip().sin(),
            Func::Lacunary => {
                if z.abs() >= LACUNARY_LIMIT {
                    return None;
                }
                let (mut sum, mut p) = (ZERO, z);
                for _ in 0..LACUNARY_TERMS {
                    sum = sum + p;
                    p = p * p;
                }
                sum
            }
        };
        finite(w)
    }

    /// Its exact zeros inside the window `x × y` (the dots a picture marks).
    pub fn zeros(self, x: (f64, f64), y: (f64, f64)) -> Vec<C> {
        let inside = |z: &C| z.re >= x.0 && z.re <= x.1 && z.im >= y.0 && z.im <= y.1;
        let all: Vec<C> = match self {
            Func::Identity | Func::Square => vec![ZERO],
            Func::Cubic => (0..3).map(|k| C::polar(1.0, TAU * f64::from(k) / 3.0)).collect(),
            Func::ZeroDoublePole => vec![ONE],
            Func::Sin | Func::Tan => (-8..=8).map(|k| C::real(PI * f64::from(k))).collect(),
            Func::Moebius => vec![C::real(MOEBIUS_ZERO)],
            Func::Quintic => quintic_roots().to_vec(),
            Func::Zeta => {
                let mut out: Vec<C> = (1..=8).map(|k| C::real(-2.0 * f64::from(k))).collect();
                out.extend(ZETA_ZERO_HEIGHTS.iter().map(|&t| C::new(0.5, t)));
                out
            }
            Func::SinRecip => (1..=40)
                .flat_map(|k| [1.0, -1.0].map(|s| C::real(s / (PI * f64::from(k)))))
                .collect(),
            _ => Vec::new(),
        };
        all.into_iter().filter(inside).collect()
    }

    /// Its poles inside the window, with their orders.
    pub fn poles(self, x: (f64, f64), y: (f64, f64)) -> Vec<(C, u32)> {
        let inside = |z: &C| z.re >= x.0 && z.re <= x.1 && z.im >= y.0 && z.im <= y.1;
        let all: Vec<(C, u32)> = match self {
            Func::Recip => vec![(ZERO, 1)],
            Func::Bump => vec![(I, 1), (-I, 1)],
            Func::ZeroDoublePole => vec![(-ONE, 2)],
            Func::Tan => (-8..=7).map(|k| (C::real(PI * (f64::from(k) + 0.5)), 1)).collect(),
            Func::Gamma => (0..=8).map(|k| (C::real(-f64::from(k)), 1)).collect(),
            Func::Zeta => vec![(ONE, 1)],
            Func::Moebius => vec![(C::real(MOEBIUS_POLE), 1)],
            _ => Vec::new(),
        };
        all.into_iter().filter(|(z, _)| inside(z)).collect()
    }
}

// --------------------------------------------- the enhanced phase portrait

/// Phase lines per full turn, and brightness bands per factor e^(2π/12) in
/// size: equal steps in log|f| and arg f, so a holomorphic function's tiles
/// are nearly square wherever f′/f is finite and nonzero.
pub const PHASE_LINES: u32 = 12;

/// The step, in radians of angle and in natural log of size.
pub fn band_step() -> f64 {
    TAU / f64::from(PHASE_LINES)
}

/// How much the output's size grows from one brightness band to the next.
pub fn band_ratio() -> f64 {
    band_step().exp()
}

/// The colour wheel's twelve anchors, output direction 0 (red) counter-
/// clockwise through a quarter turn (yellow green), a half (teal) and three
/// quarters (violet).
pub const WHEEL: [[u8; 3]; 12] = [
    [0xED, 0x5B, 0x62],
    [0xF0, 0x8A, 0x51],
    [0xE7, 0xB4, 0x4E],
    [0xB7, 0xC7, 0x5B],
    [0x71, 0xC9, 0x79],
    [0x4D, 0xC3, 0xA5],
    [0x4A, 0xBA, 0xCD],
    [0x57, 0x9B, 0xE2],
    [0x7A, 0x83, 0xE7],
    [0xA3, 0x73, 0xD8],
    [0xC6, 0x6D, 0xB3],
    [0xE3, 0x65, 0x8C],
];

/// The hue of output direction `angle` (radians), blended between anchors.
pub fn wheel(angle: f64) -> [f64; 3] {
    let t = angle.rem_euclid(TAU) / TAU * WHEEL.len() as f64;
    let k = (t.floor() as usize).min(WHEEL.len() - 1);
    let f = t - k as f64;
    let (a, b) = (WHEEL[k], WHEEL[(k + 1) % WHEEL.len()]);
    [0, 1, 2].map(|c| (f64::from(a[c]) + (f64::from(b[c]) - f64::from(a[c])) * f) / 255.0)
}

/// The colour of output `w`: hue from its direction; with `enhance` = 1,
/// brightness rising through each size band and each twelfth of a turn
/// (sawtooth shading, after Wegert's enhanced phase portraits); `enhance` =
/// 0 gives hue alone. `None` (undefined, zero or overflow) stays unpainted.
pub fn portrait(w: Option<C>, enhance: f64) -> Option<[f64; 3]> {
    let w = w?;
    let k = 1.0 + (shade(w)? - 1.0) * enhance.clamp(0.0, 1.0);
    Some(wheel(w.arg()).map(|c| c * k))
}

/// The enhanced portrait's brightness factor for output `w`: a sawtooth in
/// log size (bands) times a gentler one in angle (phase lines). `None` at 0
/// and for non-finite outputs.
pub fn shade(w: C) -> Option<f64> {
    let size = w.abs();
    if !(size > 0.0 && size.is_finite()) {
        return None;
    }
    let step = band_step();
    let size_band = (size.ln() / step).rem_euclid(1.0);
    let phase_band = (w.arg().rem_euclid(TAU) / step).rem_euclid(1.0);
    Some((0.58 + 0.42 * size_band) * (0.80 + 0.20 * phase_band))
}

/// The cell centres of a `cols` × `rows` picture of the window `x × y`, row
/// 0 at the top: the input each cell samples.
pub fn cell_input(x: (f64, f64), y: (f64, f64), cols: usize, rows: usize, row: usize, col: usize) -> C {
    let u = (col as f64 + 0.5) / cols as f64;
    let w = (row as f64 + 0.5) / rows as f64;
    C::new(x.0 + (x.1 - x.0) * u, y.1 - (y.1 - y.0) * w)
}

// ------------------------------------------------------------- loop sums

/// Points 2πk/n around the circle `|z − centre| = r`, k = 0 … n−1.
pub fn circle(centre: C, r: f64, n: usize) -> Vec<C> {
    (0..n)
        .map(|k| centre + C::polar(r, TAU * k as f64 / n as f64))
        .collect()
}

/// The `n` products of a loop sum around the circle, counterclockwise: the
/// step from point k to point k+1 times f at the arc's midpoint.
pub fn loop_products(f: impl Fn(C) -> Option<C>, centre: C, r: f64, n: usize) -> Option<Vec<C>> {
    let pts = circle(centre, r, n);
    (0..n)
        .map(|k| {
            let step = pts[(k + 1) % n] - pts[k];
            let mid = centre + C::polar(r, TAU * (k as f64 + 0.5) / n as f64);
            f(mid).map(|v| v * step)
        })
        .collect()
}

/// The loop sum: the products added.
pub fn loop_sum(f: impl Fn(C) -> Option<C>, centre: C, r: f64, n: usize) -> Option<C> {
    loop_products(f, centre, r, n).map(|v| v.into_iter().fold(ZERO, |a, b| a + b))
}

/// The average of f at the `n` points of the circle.
pub fn circle_average(f: impl Fn(C) -> Option<C>, centre: C, r: f64, n: usize) -> Option<C> {
    let total = circle(centre, r, n)
        .into_iter()
        .map(&f)
        .try_fold(ZERO, |acc, w| w.map(|w| acc + w))?;
    Some(total.scale(1.0 / n as f64))
}

/// The k-th power-series coefficient about `a` read from the circle:
/// (1/rᵏ) · average of f(a + r e^{iθ}) e^{−ikθ} (Cauchy's formula for
/// f⁽ᵏ⁾(a)/k! with the trapezoid rule, exact up to aliasing).
pub fn circle_coefficient(f: impl Fn(C) -> Option<C>, a: C, r: f64, n: usize, k: i32) -> Option<C> {
    let mut total = ZERO;
    for j in 0..n {
        let theta = TAU * j as f64 / n as f64;
        let w = f(a + C::polar(r, theta))?;
        total = total + w * C::polar(1.0, -f64::from(k) * theta);
    }
    Some(total.scale(1.0 / (n as f64 * r.powi(k))))
}

// ---------------------------------------------------------------- winding

/// 11_8's coarse loop: z^`COARSE_POWER` on the unit circle, where |f′| is
/// exactly `COARSE_POWER`. In the first number of steps every sample lands on
/// 1 and a whole turn hides in each step; in the second the count certifies.
pub const COARSE_POWER: i32 = 8;
pub const COARSE_STEPS: [usize; 2] = [8, 64];

/// The most an output may turn between two samples before the count is
/// refused as under-sampled (a quarter turn).
pub const MAX_STEP_TURN: f64 = PI / 2.0;

/// Net turns of f around zero along the closed path (the last point joins
/// the first), positive counterclockwise. `None` if f is undefined or zero
/// at a sample, or two samples are too far apart in angle to trust.
///
/// This is a sampled estimate, used for the on-screen turn counters of a
/// path already drawn: samples alone cannot see a turn completed between
/// them (z⁸ sampled at eight points reads 0). Counts the film states as
/// facts come from [`winding_certified`].
pub fn winding(f: impl Fn(C) -> Option<C>, path: &[C]) -> Option<f64> {
    let values: Vec<C> = path.iter().map(|&z| f(z)).collect::<Option<_>>()?;
    if values.iter().any(|w| w.abs() == 0.0) {
        return None;
    }
    let mut total = 0.0;
    for k in 0..values.len() {
        let turn = (values[(k + 1) % values.len()] / values[k]).arg();
        if turn.abs() > MAX_STEP_TURN {
            return None;
        }
        total += turn;
    }
    Some(total / TAU)
}

/// The winding as a whole number, refusing a count that is not close to one.
pub fn winding_count(f: impl Fn(C) -> Option<C>, path: &[C]) -> Option<i64> {
    let w = winding(f, path)?;
    let n = w.round();
    ((w - n).abs() < 1e-6).then_some(n as i64)
}

/// Net turns of f around zero along the circle `|z − centre| = r`, walked
/// counterclockwise in `n` equal steps, with no turn able to hide between
/// samples. `lipschitz` must bound |f′| on the circle (the caller proves
/// it). Each step's arc has length s = 2πr/n, so its image stays within
/// L·s of the image of the step's start. When L·s < |f(start)| that disk
/// misses zero, the output turns less than a quarter turn along the arc,
/// and the principal angle of f(end)/f(start) is the true change. `None`
/// if f is undefined on a sample or any step fails the test.
pub fn winding_certified(f: impl Fn(C) -> Option<C>, centre: C, r: f64, n: usize, lipschitz: f64) -> Option<i64> {
    let values: Vec<C> = circle(centre, r, n).into_iter().map(&f).collect::<Option<_>>()?;
    let reach = lipschitz * TAU * r / n as f64;
    // only a strict "less" passes: a tie fails, and a NaN compares as neither
    if values
        .iter()
        .any(|w| reach.partial_cmp(&w.abs()) != Some(Ordering::Less))
    {
        return None;
    }
    let total: f64 = (0..n).map(|k| (values[(k + 1) % n] / values[k]).arg()).sum();
    let turns = total / TAU;
    let count = turns.round();
    ((turns - count).abs() < 1e-6).then_some(count as i64)
}

/// The five roots of z⁵ − 3z + 1 (Durand–Kerner), sorted by length.
pub fn quintic_roots() -> [C; 5] {
    let p = |z: C| Func::Quintic.eval(z).unwrap_or(ZERO);
    let seed = C::new(0.4, 0.9);
    let mut r: [C; 5] = [ONE, seed, seed * seed, seed.powi(3), seed.powi(4)];
    for _ in 0..500 {
        for i in 0..5 {
            let zi = r[i];
            let d = r
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .fold(ONE, |acc, (_, &rj)| acc * (zi - rj));
            r[i] = zi - p(zi) / d;
        }
    }
    r.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
    r
}

// ------------------------------------------------------------ derivatives

/// The difference quotient (f(z₀ + h) − f(z₀)) / h.
pub fn quotient(f: impl Fn(C) -> Option<C>, z0: C, h: C) -> Option<C> {
    Some((f(z0 + h)? - f(z0)?) / h)
}

/// For z² at `z0`: the largest gap between the image of the square of side
/// `side` centred at z0 and its linear image z0² + 2z0·h, as a share of the
/// linear image's side |2z0|·side (sampled densely on the square's edge).
pub fn square_gap_ratio(z0: C, side: f64) -> f64 {
    let d = z0.scale(2.0);
    let per_edge = 200;
    let half = side / 2.0;
    let mut worst: f64 = 0.0;
    for e in 0..4 {
        for k in 0..=per_edge {
            let t = -half + side * f64::from(k) / f64::from(per_edge);
            let h = match e {
                0 => C::new(t, -half),
                1 => C::new(half, t),
                2 => C::new(-t, half),
                _ => C::new(-half, -t),
            };
            let exact = (z0 + h) * (z0 + h) - z0 * z0;
            worst = worst.max((exact - d * h).abs());
        }
    }
    worst / (d.abs() * side)
}

// ------------------------------------------------------------ integrals

/// The four poles of 1/(1 + z⁴), e^{iπ(2k+1)/4}, k = 0 … 3.
pub fn quartic_roots() -> [C; 4] {
    [0, 1, 2, 3].map(|k| C::polar(1.0, PI * f64::from(2 * k + 1) / 4.0))
}

/// The residue of 1/(1 + z⁴) at its pole p: 1/(4p³) = −p/4.
pub fn quartic_residue(p: C) -> C {
    (p.powi(3).scale(4.0)).recip()
}

/// The residue sum over the poles inside the upper half disk.
pub fn quartic_upper_residue_sum() -> C {
    quartic_roots()
        .into_iter()
        .filter(|p| p.im > 0.0)
        .fold(ZERO, |acc, p| acc + quartic_residue(p))
}

/// The bound on the half circle's part: (length πR) × (largest size of
/// 1/(1 + z^`power`) there, at most 1/(R^power − 1)), for R > 1.
pub fn arc_bound(power: i32, r: f64) -> f64 {
    PI * r * arc_max(power, r)
}

/// The most 1/(1 + z^`power`) can be on |z| = R > 1: 1/(R^power − 1).
pub fn arc_max(power: i32, r: f64) -> f64 {
    1.0 / (r.powi(power) - 1.0)
}

/// The disk of steps h in 07_5's squeeze: it starts this wide and shrinks to this.
pub const SQUEEZE_RADII: [f64; 2] = [0.55, 0.06];

/// The half circles 13_3 grows through; the first is the contour drawn.
pub const ARC_RADII: [f64; 3] = [2.0, 5.0, 10.0];

/// 13_10's numerical sum along the real line runs over [−this, this].
pub const NUMERIC_REACH: f64 = 200.0;

/// The half circles 13_10 reads the quartic's bound at.
pub const QUARTIC_ARC_RADII: [f64; 2] = [2.0, 10.0];

/// The size of 1/(1 + z^`power`) at `z`: on |z| = R it is at most
/// 1/(R^power − 1), since |1 + z^power| ≥ R^power − 1.
pub fn arc_size(power: i32, z: C) -> f64 {
    1.0 / (ONE + z.powi(power)).abs()
}

/// ∫ g over [−l, l] by the midpoint rule with `n` steps.
pub fn midpoint_integral(g: impl Fn(f64) -> f64, l: f64, n: u32) -> f64 {
    let h = 2.0 * l / f64::from(n);
    (0..n).map(|k| g(-l + h * (f64::from(k) + 0.5))).sum::<f64>() * h
}

// ---------------------------------------------------------- continuation

/// 14_2's bars: the last power summed, and the points inside and outside the
/// unit circle whose partial sums they show.
pub const SERIES_TOP: u32 = 10;
pub const SERIES_INSIDE: f64 = 0.5;
pub const SERIES_OUTSIDE: f64 = 1.2;

/// 1 + z + … + zⁿ for real z.
pub fn geometric_partial(z: f64, n: u32) -> f64 {
    (0..=n).fold((0.0, 1.0), |(sum, power), _| (sum + power, power * z)).0
}

/// The continued square root carried from 1 around the unit circle to angle
/// `theta`: e^{iθ/2} (the principal root agrees only while θ < π).
pub fn sqrt_carried(theta: f64) -> C {
    C::polar(1.0, theta / 2.0)
}

/// Centres of the chain of disks that carries 1/(1 − z) around its pole:
/// 1 + e^{iφ} for φ = π + 2πk/`n`, k = 0 … n, each disk of radius 1 (the
/// distance to the pole).
pub fn chain_centres(n: usize) -> Vec<C> {
    (0..=n)
        .map(|k| ONE + C::polar(1.0, PI + TAU * k as f64 / n as f64))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: C, b: C, tol: f64) -> bool {
        (a - b).abs() <= tol * (1.0 + b.abs())
    }

    #[test]
    fn arithmetic_matches_the_turning_story() {
        let w = C::new(1.0, 1.0);
        assert!(close(w * w, C::new(0.0, 2.0), 1e-15));
        assert!(close(I * I, -ONE, 1e-15));
        assert!((w.abs() - 2f64.sqrt()).abs() < 1e-15);
        assert!((w.arg() - PI / 4.0).abs() < 1e-15);
        let z = C::new(3.0, 2.0);
        assert!(close(z / z, ONE, 1e-15));
        assert!(close(z.sqrt() * z.sqrt(), z, 1e-14));
        assert!(close(z.ln().exp(), z, 1e-14));
    }

    #[test]
    fn tangent_is_sine_over_cosine_and_settles_to_plus_minus_i() {
        for z in [C::new(0.3, 0.2), C::new(-1.1, 1.5), C::new(2.0, -0.7)] {
            assert!(close(z.tan(), z.sin() / z.cos(), 1e-12));
        }
        assert!(close(C::new(0.4, 12.0).tan(), I, 1e-9));
        assert!(close(C::new(0.4, -12.0).tan(), -I, 1e-9));
    }

    #[test]
    fn the_recipe_at_zero_converges_inside_one_and_fails_outside() {
        assert!((bump_recipe0(0.5, 40) - bump(0.5)).abs() < 1e-12);
        assert!(bump_recipe0(1.1, 40) > 20.0);
        // the coefficients about 0 are 1, 0, −1, 0, 1, …
        let c = bump_coefficients(0.0, 8);
        for (k, &ck) in c.iter().enumerate() {
            let want = if k % 2 == 1 {
                0.0
            } else if k % 4 == 0 {
                1.0
            } else {
                -1.0
            };
            assert!((ck - want).abs() < 1e-15, "c{k} = {ck}");
        }
        // the closed form agrees with the alternating recipe
        assert!((bump_recipe(0.0, 0.7, 41) - bump_recipe0(0.7, 40)).abs() < 1e-12);
    }

    #[test]
    fn the_recentred_recipe_reaches_exactly_to_the_poles() {
        let r = bump_radius(1.0);
        assert!((r - 2f64.sqrt()).abs() < 1e-15);
        // inside the interval (1 − √2, 1 + √2) the error falls with more terms
        for x in [-0.3, 0.0, 2.0, 2.3] {
            let e40 = (bump_recipe(1.0, x, 40) - bump(x)).abs();
            let e120 = (bump_recipe(1.0, x, 120) - bump(x)).abs();
            assert!(e120 < e40 && e120 < 1e-3, "x = {x}: {e40} → {e120}");
        }
        // outside it grows
        for x in [-0.6, 2.6] {
            let e40 = (bump_recipe(1.0, x, 40) - bump(x)).abs();
            let e60 = (bump_recipe(1.0, x, 60) - bump(x)).abs();
            assert!(e60 > e40 && e60 > 10.0, "x = {x}");
        }
    }

    #[test]
    fn gamma_extends_the_factorial_and_has_its_poles() {
        let mut fact = 1.0;
        for n in 1..=10 {
            let g = gamma(C::real(f64::from(n))).unwrap();
            assert!((g.re - fact).abs() < 1e-10 * fact && g.im.abs() < 1e-10 * fact);
            fact *= f64::from(n);
        }
        assert!((gamma(C::real(0.5)).unwrap().re - PI.sqrt()).abs() < 1e-13);
        for z in [C::new(0.3, 1.2), C::new(-2.4, 0.8), C::new(3.1, -2.0)] {
            assert!(close(gamma(z + ONE).unwrap(), z * gamma(z).unwrap(), 1e-12));
        }
        for k in 0..6 {
            assert!(gamma(C::real(-f64::from(k))).is_none(), "pole at −{k}");
        }
        assert!(gamma(C::new(-3.0, 1e-3)).is_some());
    }

    #[test]
    fn zeta_matches_known_values_on_both_sides() {
        assert!(close(zeta(C::real(2.0)).unwrap(), C::real(PI * PI / 6.0), 1e-12));
        assert!(close(zeta(C::real(4.0)).unwrap(), C::real(PI.powi(4) / 90.0), 1e-12));
        assert!(close(zeta(C::real(-1.0)).unwrap(), C::real(-1.0 / 12.0), 1e-12));
        assert!(close(
            zeta(C::real(0.5)).unwrap(),
            C::real(-1.460_354_508_809_586_8),
            1e-12
        ));
        for k in 1..=4 {
            assert!(zeta(C::real(-2.0 * f64::from(k))).unwrap().abs() < 1e-12);
        }
        assert!(zeta(ONE).is_none());
        // the two formulas agree across the switch at Re s = ½
        for t in [0.0, 3.0, 14.0, 30.0] {
            let a = zeta(C::new(0.5 + 1e-9, t)).unwrap();
            let b = zeta(C::new(0.5 - 1e-9, t)).unwrap();
            assert!(close(a, b, 1e-7), "t = {t}");
        }
    }

    #[test]
    fn newton_finds_the_published_zeros_on_the_line() {
        for &t in &ZETA_ZERO_HEIGHTS {
            let z = zeta_zero_near(C::new(0.5, t.round())).unwrap();
            assert!((z.re - 0.5).abs() < 1e-10 && (z.im - t).abs() < 1e-9, "{t}: {z:?}");
        }
    }

    #[test]
    fn the_wheel_and_portrait_conventions() {
        assert_eq!(wheel(0.0), WHEEL[0].map(|c| f64::from(c) / 255.0));
        assert_eq!(wheel(PI / 2.0), WHEEL[3].map(|c| f64::from(c) / 255.0));
        assert_eq!(wheel(PI), WHEEL[6].map(|c| f64::from(c) / 255.0));
        assert!(portrait(None, 1.0).is_none());
        assert!(portrait(Some(ZERO), 1.0).is_none());
        assert_eq!(portrait(Some(C::new(-2.0, 0.0)), 0.0), Some(wheel(PI)));
        // the size bands: one step in log size is the band ratio
        assert!((band_ratio() - (TAU / 12.0).exp()).abs() < 1e-15);
        // exp's tiles are exact squares: its shading repeats every 2π/12 to
        // the right (size bands) and every 2π/12 up (phase lines)
        let step = band_step();
        let at = |x: f64, y: f64| shade(Func::Exp.eval(C::new(x, y)).unwrap()).unwrap();
        for (x, y) in [(0.1, 0.1), (-1.3, 2.2), (0.7, -0.4)] {
            assert!((at(x, y) - at(x + step, y)).abs() < 1e-9);
            assert!((at(x, y) - at(x, y + step)).abs() < 1e-9);
        }
    }

    #[test]
    fn only_one_over_z_survives_a_loop() {
        let n = 24;
        for p in [-3, -2, 0, 1, 2] {
            let s = loop_sum(|z| Some(z.powi(p)), ZERO, 1.3, n).unwrap();
            assert!(s.abs() < 1e-12, "power {p}: {s:?}");
        }
        let s = loop_sum(|z| Some(z.recip()), ZERO, 1.0, n).unwrap();
        let exact = 2.0 * n as f64 * (PI / n as f64).sin();
        assert!(s.re.abs() < 1e-12 && (s.im - exact).abs() < 1e-12);
        // every product of 1/z points straight up
        for v in loop_products(|z| Some(z.recip()), ZERO, 2.0, n).unwrap() {
            assert!(v.re.abs() < 1e-13 && v.im > 0.0);
        }
        // and the total does not depend on the radius
        let big = loop_sum(|z| Some(z.recip()), ZERO, 2.0, n).unwrap();
        assert!(close(big, s, 1e-13));
    }

    #[test]
    fn the_bump_average_holds_inside_and_fails_with_poles_inside() {
        let f = |z| Func::Bump.eval(z);
        let a8 = circle_average(f, ZERO, 0.5, 8).unwrap();
        assert!((a8.re - 1.0 / (1.0 - 0.5f64.powi(8))).abs() < 1e-14 && a8.im.abs() < 1e-14);
        let a16 = circle_average(f, ZERO, 2.0, 16).unwrap();
        assert!((a16.re + 1.0 / (2f64.powi(16) - 1.0)).abs() < 1e-14);
        // coefficients from the boundary: 1, 0, −1, 0, 1
        for (k, want) in [(0, 1.0), (1, 0.0), (2, -1.0), (3, 0.0), (4, 1.0)] {
            let c = circle_coefficient(f, ZERO, 0.5, 16, k).unwrap();
            assert!((c.re - want).abs() < 1e-3 && c.im.abs() < 1e-12, "c{k} = {c:?}");
        }
    }

    #[test]
    fn counting_turns_on_the_examples_and_at_the_edges() {
        let f = |z| Func::Moebius.eval(z);
        assert_eq!(winding_count(f, &circle(ZERO, 0.8, 2048)), Some(1));
        assert_eq!(winding_count(f, &circle(ZERO, 1.7, 2048)), Some(0));
        // a loop through the pole or the zero is refused
        assert_eq!(winding_count(f, &circle(ZERO, MOEBIUS_POLE, 2048)), None);
        let q = |z| Func::Quintic.eval(z);
        assert_eq!(winding_count(q, &circle(ZERO, 1.0, 4096)), Some(1));
        assert_eq!(winding_count(q, &circle(ZERO, 3.0, 4096)), Some(5));
        let roots = quintic_roots();
        assert_eq!(roots.iter().filter(|r| r.abs() < 1.0).count(), 1);
        for r in roots {
            assert!(q(r).unwrap().abs() < 1e-12);
        }
        // the leash: on |z| = 3, |p(z) − z⁵| < |z⁵|; on |z| = 1, the rest of
        // p is |z⁵ + 1| ≤ 2 < 3 = |−3z|, so p turns as −3z does, once
        for z in circle(ZERO, 3.0, 720) {
            assert!((q(z).unwrap() - z.powi(5)).abs() < z.powi(5).abs());
        }
        for z in circle(ZERO, 1.0, 720) {
            assert!((q(z).unwrap() + z.scale(3.0)).abs() <= 2.0 + 1e-12);
        }
        // the bump around a pole of order one turns back once; z² twice forward
        assert_eq!(winding_count(|z| Func::Bump.eval(z), &circle(I, 0.5, 2048)), Some(-1));
        assert_eq!(
            winding_count(|z| Func::Square.eval(z), &circle(ZERO, 0.5, 2048)),
            Some(2)
        );
        assert_eq!(
            winding_count(|z| Func::Cubic.eval(z), &circle(ZERO, 1.8, 2048)),
            Some(3)
        );
    }

    #[test]
    fn a_turn_hidden_between_samples_is_refused_not_miscounted() {
        // z⁸ at eight points: every sample is 1, the naive count says 0
        let z8 = |z: C| Some(z.powi(8));
        assert_eq!(winding_count(z8, &circle(ZERO, 1.0, 8)), Some(0));
        // the certified count refuses it (L = 8, reach 2π > 1) …
        assert_eq!(winding_certified(z8, ZERO, 1.0, 8, 8.0), None);
        // … and with enough steps it finds the eight turns
        assert_eq!(winding_certified(z8, ZERO, 1.0, 4096, 8.0), Some(8));
        // the quintic on |z| = 1: |f′| = |5z⁴ − 3| ≤ 8 there
        let q = |z| Func::Quintic.eval(z);
        assert_eq!(winding_certified(q, ZERO, 1.0, 4096, 8.0), Some(1));
        // a loop through a pole has no count at all
        let m = |z| Func::Moebius.eval(z);
        assert_eq!(winding_certified(m, ZERO, MOEBIUS_POLE, 4096, f64::MAX), None);
    }

    #[test]
    fn a_tie_or_a_nan_never_certifies_a_winding() {
        fn constant(w: f64) -> impl Fn(C) -> Option<C> {
            move |_| Some(C::real(w))
        }
        let (r, l, n) = (1.0, 3.0, 64_usize);
        // the certificate's own reach, computed the way it computes it
        let reach = l * TAU * r / n as f64;
        // a constant clear of the reach has no turns, and passes
        assert_eq!(winding_certified(constant(2.0 * reach), ZERO, r, n, l), Some(0));
        // equal to the reach, it fails: the test is strict
        assert_eq!(winding_certified(constant(reach), ZERO, r, n, l), None);
        // a NaN on either side fails, and so does an unbounded reach
        assert_eq!(winding_certified(constant(f64::NAN), ZERO, r, n, l), None);
        assert_eq!(winding_certified(constant(1.0), ZERO, r, n, f64::NAN), None);
        assert_eq!(winding_certified(constant(1.0), ZERO, r, n, f64::INFINITY), None);
    }

    #[test]
    fn division_is_the_product_with_the_reciprocal() {
        let (a, b) = (C::new(3.0, -2.0), C::new(0.7, 1.9));
        let (q, p) = (a / b, a * b.recip());
        assert_eq!((q.re.to_bits(), q.im.to_bits()), (p.re.to_bits(), p.im.to_bits()));
        // a zero divisor stays non-finite, which `finite` turns into `None`
        assert!(!(a / ZERO).is_finite());
        assert_eq!(finite(a / ZERO), None);
    }

    #[test]
    fn the_derivative_story_numbers() {
        let z0 = C::new(1.0, 1.0);
        let f = |z| Func::Square.eval(z);
        for dir in [ONE, I, C::polar(1.0, 1.0)] {
            let q = quotient(f, z0, dir.scale(1e-7)).unwrap();
            assert!(close(q, C::new(2.0, 2.0), 1e-6));
        }
        // conjugation: 1 from the right, −1 from above
        let g = |z: C| Some(z.conj());
        assert!(close(quotient(g, z0, C::real(1e-3)).unwrap(), ONE, 1e-12));
        assert!(close(quotient(g, z0, C::new(0.0, 1e-3)).unwrap(), -ONE, 1e-12));
        // |z|² at 0: the quotient is the conjugate of h; at 1 it depends on direction
        let a = |z| Func::AbsSquare.eval(z);
        assert!(quotient(a, ZERO, C::new(3e-4, 4e-4)).unwrap().abs() < 1e-3);
        assert!(close(quotient(a, ONE, C::real(1e-8)).unwrap(), C::real(2.0), 1e-6));
        assert!(quotient(a, ONE, C::new(0.0, 1e-8)).unwrap().abs() < 1e-6);
        // the square's gap shrinks in proportion to its side
        let r1 = square_gap_ratio(z0, 0.5);
        let r2 = square_gap_ratio(z0, 0.05);
        assert!((r1 / r2 - 10.0).abs() < 1e-6);
        assert!((r1 - 0.5 / (2.0 * 8f64.sqrt())).abs() < 1e-12);
    }

    #[test]
    fn the_quartic_integral_and_its_arc() {
        let s = quartic_upper_residue_sum();
        assert!(close(s, C::new(0.0, -(2f64.sqrt()) / 4.0), 1e-15));
        for p in quartic_roots() {
            assert!(close(quartic_residue(p), -p.scale(0.25), 1e-15));
            assert!(close(p.powi(4), -ONE, 1e-15));
        }
        let value = (C::new(0.0, TAU) * s).re;
        assert!((value - PI / 2f64.sqrt()).abs() < 1e-15);
        assert!(arc_bound(4, 10.0) < 1.0 / 300.0);
        let numeric = midpoint_integral(|x| 1.0 / (1.0 + x.powi(4)), 200.0, 400_000);
        assert!((numeric - value).abs() < 1e-6);
        let bump_area = midpoint_integral(bump, 2000.0, 4_000_000);
        assert!((bump_area - PI).abs() < 2e-3);
    }

    #[test]
    fn continuation_examples() {
        assert!((geometric_partial(0.5, 10) - 1.999_023_437_5).abs() < 1e-12);
        assert!(geometric_partial(1.2, 10) > 32.0);
        assert!(close(sqrt_carried(TAU), -ONE, 1e-15));
        assert!(close(sqrt_carried(0.3) * sqrt_carried(0.3), C::polar(1.0, 0.3), 1e-15));
        let chain = chain_centres(6);
        assert!(close(chain[0], ZERO, 1e-15) && close(chain[6], ZERO, 1e-14));
        for w in chain.windows(2) {
            assert!((w[1] - w[0]).abs() < 2.0, "neighbouring disks overlap");
        }
        assert!(Func::Lacunary.eval(C::real(0.99)).is_none());
        assert!(Func::Lacunary.eval(C::real(0.97)).unwrap().re > 4.0);
    }
}
