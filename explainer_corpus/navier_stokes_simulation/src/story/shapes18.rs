//! Chapter 18's numbers and teaching geometry, exact and std only, so `rustc --test` runs the
//! tests below without the engine.
//!
//! RubyRaven `ruby-car01` for root's `navier_stokes_simulation` crate (Agent Mail 36250). Every
//! picture in chapter 18 is a schematic of our own design, defined here; none is a flow field,
//! measured or computed. The numbers:
//! - the test's Reynolds number, as chapter 17 shows it: ST96 §2.2 (Um = 1.5 m/s, mean 2 Um / 3,
//!   D = 0.1 m) and §2.1 (ν = 10⁻³ m²/s);
//! - the car's: independent01 `TREATMENT.md` line 271 (30 m/s, 4.5 m long, air at
//!   ν ≈ 1.5 × 10⁻⁵ m²/s, a textbook room-temperature value, SECONDARY there and not measured);
//! - the mile per hour, exact by definition (1 mile = 1609.344 m).
//!
//! `tools/check_car01.py` recomputes each of them from the sources in Fractions.

/// A fraction p / q, held exactly.
pub type Ratio = (i64, i64);

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

pub fn reduce((p, q): Ratio) -> Ratio {
    let g = gcd(p, q).max(1);
    let s = if q < 0 { -1 } else { 1 };
    (s * p / g, s * q / g)
}

pub fn mul(a: Ratio, b: Ratio) -> Ratio {
    reduce((a.0 * b.0, a.1 * b.1))
}

pub fn div(a: Ratio, b: Ratio) -> Ratio {
    reduce((a.0 * b.1, a.1 * b.0))
}

/// The decimal digits of a fraction whose denominator has no prime but 2 and 5.
pub fn decimal(r: Ratio) -> Option<String> {
    let (p, q) = reduce(r);
    let (mut rest, mut twos, mut fives) = (q, 0u32, 0u32);
    while rest % 2 == 0 {
        rest /= 2;
        twos += 1;
    }
    while rest % 5 == 0 {
        rest /= 5;
        fives += 1;
    }
    if rest != 1 {
        return None;
    }
    let places = twos.max(fives);
    let scaled = p * 10i64.pow(places) / q;
    let digits = scaled.abs().to_string();
    let body = if places == 0 {
        digits
    } else {
        let padded = format!("{digits:0>width$}", width = places as usize + 1);
        let (whole, part) = padded.split_at(padded.len() - places as usize);
        format!("{whole}.{part}")
    };
    Some(if scaled < 0 { format!("-{body}") } else { body })
}

/// A positive fraction as mantissa (1 to 10, its decimal digits) and power of ten.
pub fn scientific(r: Ratio) -> Option<(String, i32)> {
    let (mut p, mut q) = reduce(r);
    if p <= 0 {
        return None;
    }
    let mut e = 0;
    while p >= 10 * q {
        q *= 10;
        e += 1;
    }
    while p < q {
        p *= 10;
        e -= 1;
    }
    Some((decimal((p, q))?, e))
}

/// A whole number with commas every three digits.
pub fn thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (k, ch) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn whole(r: Ratio) -> Option<i64> {
    let (p, q) = reduce(r);
    (q == 1).then_some(p)
}

// ------------------------------------------------------------------ the two flows

/// The benchmark test of chapter 17, ST96 2D-2.
pub mod test_flow {
    use super::{Ratio, div, mul};

    /// ST96 §2.2: the inflow's peak, Um = 1.5 m/s; its mean over the inlet is 2 Um / 3.
    pub const PEAK: Ratio = (3, 2);
    /// ST96 §2.2: D = 0.1 m.
    pub const DIAMETER: Ratio = (1, 10);
    /// ST96 §2.1: ν = 10⁻³ m²/s.
    pub const NU: Ratio = (1, 1000);

    pub fn mean() -> Ratio {
        mul((2, 3), PEAK)
    }

    /// Re = Ū D / ν.
    pub fn reynolds() -> Ratio {
        div(mul(mean(), DIAMETER), NU)
    }
}

/// A real car, as TREATMENT line 271 gives it.
pub mod car_flow {
    use super::{Ratio, div, mul, test_flow};

    pub const SPEED: Ratio = (30, 1);
    pub const LENGTH: Ratio = (9, 2);
    /// Air at room temperature, 1.5 × 10⁻⁵ m²/s: a textbook value, not measured here.
    pub const NU_AIR: Ratio = (3, 200_000);
    /// One mile per hour in m/s, exact by definition: 1609.344 m in 3600 s.
    pub const MPH: Ratio = (1_609_344, 3_600_000);

    /// Re = U L / ν.
    pub fn reynolds() -> Ratio {
        div(mul(SPEED, LENGTH), NU_AIR)
    }

    /// How many times the test's Reynolds number the car's is.
    pub fn ratio() -> Ratio {
        div(reynolds(), test_flow::reynolds())
    }

    /// The car's speed in miles per hour.
    pub fn mph() -> Ratio {
        div(SPEED, MPH)
    }
}

/// 18_7's first formula, chapter 17's 17_9 formula with its name: "test: Re = Ū D / ν = 1 ×
/// 0.1 / 0.001 = 100".
pub fn test_re_tex() -> String {
    let d = |r: Ratio| decimal(r).expect("terminating");
    format!(
        r"\text{{test: }}\mathrm{{Re}} = \bar{{U}} D / \nu = {} \times {} / {} = {}",
        d(test_flow::mean()),
        d(test_flow::DIAMETER),
        d(test_flow::NU),
        d(test_flow::reynolds())
    )
}

/// 18_7's second formula: "car: Re = U L / ν = 30 × 4.5 / (1.5 × 10⁻⁵) = 9 × 10⁶".
pub fn car_re_tex() -> String {
    let d = |r: Ratio| decimal(r).expect("terminating");
    let (nu, nu_e) = scientific(car_flow::NU_AIR).expect("positive");
    let (re, re_e) = scientific(car_flow::reynolds()).expect("positive");
    format!(
        r"\text{{car: }}\mathrm{{Re}} = U L / \nu = {} \times {} / ({nu} \times 10^{{{nu_e}}}) = {re} \times 10^{{{re_e}}}",
        d(car_flow::SPEED),
        d(car_flow::LENGTH),
    )
}

/// "test: Re 100".
pub fn test_label() -> String {
    format!("test: Re {}", thousands(whole(test_flow::reynolds()).expect("whole")))
}

/// "car: Re ≈ 9,000,000" (approximate because the air's viscosity is).
pub fn car_label() -> String {
    format!("car: Re ≈ {}", thousands(whole(car_flow::reynolds()).expect("whole")))
}

/// "× 90,000".
pub fn ratio_label() -> String {
    format!("× {}", thousands(whole(car_flow::ratio()).expect("whole")))
}

/// "30 m/s ≈ 67 mph", to the nearest mile per hour.
pub fn mph_label() -> String {
    let (p, q) = car_flow::mph();
    let nearest = (2 * p + q) / (2 * q);
    format!("{} m/s ≈ {nearest} mph", decimal(car_flow::SPEED).expect("whole"))
}

/// The tick labels of 18_7's bar, 10¹ to 10⁷, in TeX.
pub fn decade_tex(k: i32) -> String {
    format!("10^{{{k}}}")
}

/// 18_7's logarithmic bar: decades 10¹ to 10⁷, [`log_bar::DECADE`] scene units each.
pub mod log_bar {
    pub const FIRST: i32 = 1;
    pub const LAST: i32 = 7;
    pub const LEFT: f64 = -6.0;
    pub const DECADE: f64 = 2.0;

    pub fn x(value: f64) -> f64 {
        LEFT + DECADE * (value.log10() - f64::from(FIRST))
    }

    pub fn right() -> f64 {
        LEFT + DECADE * f64::from(LAST - FIRST)
    }
}

// ------------------------------------------------------------------ the frame change (18_4)

/// Velocities along the road in units of the car's speed U, ground frame: the car +U, the air
/// and the road still. Seen from the car, subtract the car's velocity from everything.
pub const GROUND: [(&str, i64); 3] = [("car", 1), ("air", 0), ("road", 0)];

pub fn in_car_frame(v: i64) -> i64 {
    v - GROUND[0].1
}

/// "+U", "0" or "−U".
pub fn u_text(k: i64) -> String {
    match k {
        0 => "0".to_owned(),
        1 => "+U".to_owned(),
        -1 => "−U".to_owned(),
        k if k > 0 => format!("+{k}U"),
        k => format!("−{}U", -k),
    }
}

/// A label such as "road: −U", from [`GROUND`] (seen from the car when `car_frame`).
pub fn frame_label(name: &str, car_frame: bool) -> Option<String> {
    let v = GROUND.iter().find(|(n, _)| *n == name)?.1;
    Some(format!(
        "{name}: {}",
        u_text(if car_frame { in_car_frame(v) } else { v })
    ))
}

/// Rolling without slipping: the rim moves past the road at the road's speed, so the wheel
/// turns at `speed / radius` radians per second.
pub fn wheel_rate(speed: f64, radius: f64) -> f64 {
    speed / radius
}

// ------------------------------------------------------------------ poses and projection

/// The cabinet projection used for every 3D schematic: depth z (into the screen) moves a point
/// up and to the right by half its length at 30°, (z √3 / 4, z / 4).
pub fn oblique(z: f64) -> (f64, f64) {
    (z * 3f64.sqrt() / 4.0, z / 4.0)
}

/// Where a shape defined in metres sits on screen: its origin's scene point and scene units per
/// metre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub origin: (f64, f64),
    pub scale: f64,
}

impl Pose {
    pub fn at(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.origin.0 + self.scale * x, self.origin.1 + self.scale * y)
    }

    /// A point with depth z (metres, positive into the screen), in the cabinet projection.
    pub fn at3(&self, (x, y, z): (f64, f64, f64)) -> (f64, f64) {
        let (dx, dy) = oblique(z);
        self.at((x + dx, y + dy))
    }

    /// The map from this pose to `to`: scale by the factor about this origin, then shift.
    pub fn carry(&self, to: &Pose) -> (f64, (f64, f64), (f64, f64)) {
        (
            to.scale / self.scale,
            self.origin,
            (to.origin.0 - self.origin.0, to.origin.1 - self.origin.1),
        )
    }
}

// ------------------------------------------------------------------ the car outline

/// Our own car-like outline, side view, in centimetres: x from the tail (0) to the nose (450),
/// y up from the road. It is not a real car and carries no measured shape.
pub mod car {
    /// The top, tail to nose: straight up from the tail's foot, a uniform Catmull–Rom spline
    /// through the eight inner points, straight down to the nose's foot.
    pub const TOP: [(i64, i64); 10] = [
        (0, 45),
        (4, 88),
        (60, 97),
        (112, 104),
        (165, 140),
        (270, 142),
        (332, 108),
        (420, 92),
        (447, 76),
        (450, 45),
    ];
    pub const NOSE_BOTTOM: (i64, i64) = (444, 22);
    pub const TAIL_BOTTOM: (i64, i64) = (8, 24);
    /// The underside's height, and the centre height of the wheel arches.
    pub const SILL: i64 = 20;
    pub const WHEEL_X: [i64; 2] = [85, 360];
    pub const WHEEL_R: i64 = 31;
    pub const ARCH_R: i64 = 45;
    pub const SPLINE_STEPS: usize = 8;
    pub const ARCH_STEPS: usize = 16;

    fn m((x, y): (i64, i64)) -> (f64, f64) {
        (x as f64 / 100.0, y as f64 / 100.0)
    }

    /// A uniform Catmull–Rom spline through `p` (ends repeated), `steps` samples per span,
    /// ending on the last point.
    pub fn catmull_rom(p: &[(f64, f64)], steps: usize) -> Vec<(f64, f64)> {
        let n = p.len();
        let at = |k: isize| p[k.clamp(0, n as isize - 1) as usize];
        let mut out = Vec::new();
        for i in 0..n - 1 {
            let (p0, p1, p2, p3) = (
                at(i as isize - 1),
                at(i as isize),
                at(i as isize + 1),
                at(i as isize + 2),
            );
            for s in 0..steps {
                let t = s as f64 / steps as f64;
                let f = |a: f64, b: f64, c: f64, d: f64| {
                    0.5 * (2.0 * b
                        + (c - a) * t
                        + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t
                        + (3.0 * b - a - 3.0 * c + d) * t * t * t)
                };
                out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
            }
        }
        out.push(p[n - 1]);
        out
    }

    /// An arch over wheel `k`: centre (WHEEL_X, SILL), from its front foot up and over to its
    /// rear foot.
    pub fn arch(k: usize) -> Vec<(f64, f64)> {
        let (cx, cy, r) = (WHEEL_X[k] as f64 / 100.0, SILL as f64 / 100.0, ARCH_R as f64 / 100.0);
        (0..=ARCH_STEPS)
            .map(|s| {
                let a = std::f64::consts::PI * s as f64 / ARCH_STEPS as f64;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect()
    }

    /// The closed outline in metres (the first point is not repeated): the top, the nose, the
    /// underside with both arches, the tail.
    pub fn outline() -> Vec<(f64, f64)> {
        let top: Vec<(f64, f64)> = TOP.iter().map(|&p| m(p)).collect();
        let mut out = vec![top[0]];
        out.extend(catmull_rom(&top[1..top.len() - 1], SPLINE_STEPS));
        out.push(top[top.len() - 1]);
        out.push(m(NOSE_BOTTOM));
        out.extend(arch(1));
        out.extend(arch(0));
        out.push(m(TAIL_BOTTOM));
        out
    }

    /// The wheel centres (on the road at radius height) and radius, metres.
    pub fn wheels() -> [(f64, f64); 2] {
        let r = WHEEL_R as f64 / 100.0;
        [(WHEEL_X[0] as f64 / 100.0, r), (WHEEL_X[1] as f64 / 100.0, r)]
    }

    pub fn wheel_radius() -> f64 {
        WHEEL_R as f64 / 100.0
    }

    /// Points the 18_3 extrusion's lines run through: the tail, the roof's two ends, the
    /// windscreen's foot, the nose and the two bottom corners, all on the outline.
    pub fn ridge_points() -> Vec<(f64, f64)> {
        [TOP[0], TOP[4], TOP[5], TOP[6], TOP[9], NOSE_BOTTOM, TAIL_BOTTOM]
            .iter()
            .map(|&p| m(p))
            .collect()
    }

    #[cfg(test)]
    pub fn length() -> f64 {
        TOP[TOP.len() - 1].0 as f64 / 100.0
    }

    #[cfg(test)]
    pub fn height() -> f64 {
        outline().iter().map(|p| p.1).fold(f64::MIN, f64::max)
    }
}

/// 18_3's endless wall: copies of the outline every [`ridge::STEP`] metres of depth, both ways,
/// fading toward the ends.
pub mod ridge {
    pub const STEP: f64 = 0.6;
    pub const COPIES: usize = 6;

    #[cfg(test)]
    pub fn reach() -> f64 {
        STEP * COPIES as f64
    }

    /// The copies' depths, nearest first, both ways.
    #[cfg(test)]
    pub fn depths() -> Vec<f64> {
        (1..=COPIES)
            .flat_map(|k| [-(k as f64) * STEP, k as f64 * STEP])
            .collect()
    }

    /// Opacity at depth z: 1 at the slice, falling to zero one step past the last copy.
    pub fn fade(z: f64) -> f64 {
        1.0 - z.abs() / (STEP * (COPIES + 1) as f64)
    }
}

/// 18_3's real car, as boxes of finite width (cm): x, y and z ranges of the body and of the
/// cabin on it. 1.8 m wide; the same length and wheels as the outline.
pub mod block {
    pub const BODY: [(i64, i64); 3] = [(0, 450), (20, 90), (0, 180)];
    pub const CABIN: [(i64, i64); 3] = [(120, 330), (90, 140), (15, 165)];
    /// The arrow around the near side, metres: at height `AROUND_Y`, from mid-width ahead of the
    /// nose (x = 4.5 + `AROUND_REACH`) out to z = `AROUND_SIDE` alongside the whole body, back to
    /// mid-width behind the tail; parabolic in x outside the body's length.
    pub const AROUND_Y: f64 = 0.55;
    pub const AROUND_SIDE: f64 = -0.8;
    pub const AROUND_REACH: f64 = 0.7;
    pub const AROUND_STEPS: usize = 48;

    pub fn around() -> Vec<(f64, f64, f64)> {
        let (x0, x1) = (BODY[0].0 as f64 / 100.0, BODY[0].1 as f64 / 100.0);
        let mid = (BODY[2].0 + BODY[2].1) as f64 / 200.0;
        let (ahead, behind) = (x1 + AROUND_REACH, x0 - AROUND_REACH);
        (0..=AROUND_STEPS)
            .map(|k| {
                let x = ahead + (behind - ahead) * k as f64 / AROUND_STEPS as f64;
                let out = if x > x1 {
                    (x - x1) / AROUND_REACH
                } else if x < x0 {
                    (x0 - x) / AROUND_REACH
                } else {
                    0.0
                };
                (x, AROUND_Y, AROUND_SIDE + (mid - AROUND_SIDE) * out * out)
            })
            .collect()
    }

    /// The three faces the cabinet view sees (the near side z = min, the top y = max and the
    /// nose end x = max), each as four corners in metres, in drawing order.
    pub fn faces(b: [(i64, i64); 3]) -> [[(f64, f64, f64); 4]; 3] {
        let m = |v: i64| v as f64 / 100.0;
        let ((x0, x1), (y0, y1), (z0, z1)) = (b[0], b[1], b[2]);
        let (x0, x1, y0, y1, z0, z1) = (m(x0), m(x1), m(y0), m(y1), m(z0), m(z1));
        [
            [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)],
            [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)],
            [(x1, y0, z0), (x1, y0, z1), (x1, y1, z1), (x1, y1, z0)],
        ]
    }
}

// ------------------------------------------------------------------ the vortex tube (18_5)

/// 18_5's stretched swirl: an idealised tube of uniform spin, kept at constant volume (the fluid
/// is incompressible) and constant circulation (Kelvin's theorem, standard, not read here).
pub mod tube {
    /// The view turns cross-sections 30° about the vertical: a circle of radius r shows as an
    /// ellipse r / 2 wide and r tall, and the axis shows at cos 30° of its length.
    pub const SIN_VIEW: f64 = 0.5;

    pub fn cos_view() -> f64 {
        3f64.sqrt() / 2.0
    }

    /// Stretched from length l0 to l at constant volume π r² l: r = r0 √(l0 / l).
    pub fn radius(r0: f64, l0: f64, l: f64) -> f64 {
        r0 * (l0 / l).sqrt()
    }

    /// Spin rate at constant circulation 2π r² Ω: Ω = Ω0 (r0 / r)² = Ω0 l / l0.
    pub fn spin(w0: f64, l0: f64, l: f64) -> f64 {
        w0 * l / l0
    }

    /// The stretch's progress: 3 s² − 2 s³, clamped to [0, 1].
    pub fn smoothstep(s: f64) -> f64 {
        let s = s.clamp(0.0, 1.0);
        s * s * (3.0 - 2.0 * s)
    }

    /// Part of a cross-section at screen x `cx`, axis height `cy`: angles `from` to `to`
    /// (radians from the top, turning with the spin), `steps` spans.
    pub fn section(c: (f64, f64), r: f64, from: f64, to: f64, steps: usize) -> Vec<(f64, f64)> {
        (0..=steps)
            .map(|k| {
                let t = from + (to - from) * k as f64 / steps as f64;
                (c.0 + r * SIN_VIEW * t.sin(), c.1 + r * t.cos())
            })
            .collect()
    }

    /// The same for a flat swirl seen face on: a circle.
    pub fn circle_arc(c: (f64, f64), r: f64, from: f64, to: f64, steps: usize) -> Vec<(f64, f64)> {
        (0..=steps)
            .map(|k| {
                let t = from + (to - from) * k as f64 / steps as f64;
                (c.0 + r * t.sin(), c.1 + r * t.cos())
            })
            .collect()
    }
}

// ------------------------------------------------------------------ the slice (18_6)

/// 18_6's schematic 3D flow, in scene units before projection: the slice is the plane z = 0,
/// x from 0 to [`slice::W`], y from 0 to [`slice::H`]; depth runs ±[`slice::DEPTH`].
pub mod slice {
    pub type P3 = (f64, f64, f64);

    pub const W: f64 = 4.4;
    pub const H: f64 = 3.2;
    pub const DEPTH: f64 = 2.0;
    /// Arrows that cross the slice, from in front of it (z < 0) to behind it (z > 0): air
    /// crossing it, w ≠ 0.
    pub const CROSSING: [(P3, P3); 3] = [
        ((0.6, 2.4, -1.6), (1.4, 2.6, 1.6)),
        ((2.1, 1.7, -1.6), (2.8, 2.0, 1.6)),
        ((0.9, 0.7, -1.6), (1.7, 0.8, 1.6)),
    ];
    /// Two arrows parallel to the slice (w = 0) at depths ∓1.4, pointing different ways: the
    /// flow changes along z.
    pub const ALONG: [(P3, P3); 2] = [((2.8, 0.4, -1.4), (3.8, 0.4, -1.4)), ((2.8, 0.4, 1.4), (3.4, 1.1, 1.4))];

    /// Where the arrow from `a` to `b` pierces z = 0.
    pub fn pierce(a: P3, b: P3) -> Option<P3> {
        if a.2 * b.2 >= 0.0 {
            return None;
        }
        let t = -a.2 / (b.2 - a.2);
        Some((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1), 0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn reynolds_numbers_are_exact() {
        assert_eq!(test_flow::mean(), (1, 1));
        assert_eq!(test_flow::reynolds(), (100, 1));
        assert_eq!(car_flow::reynolds(), (9_000_000, 1));
        assert_eq!(car_flow::ratio(), (90_000, 1));
        assert_eq!(car_flow::mph(), (93_750, 1397));
        assert_eq!(reduce(car_flow::MPH), (1397, 3125));
    }

    #[test]
    fn labels_come_from_the_numbers() {
        assert_eq!(
            test_re_tex(),
            r"\text{test: }\mathrm{Re} = \bar{U} D / \nu = 1 \times 0.1 / 0.001 = 100"
        );
        assert_eq!(
            car_re_tex(),
            r"\text{car: }\mathrm{Re} = U L / \nu = 30 \times 4.5 / (1.5 \times 10^{-5}) = 9 \times 10^{6}"
        );
        assert_eq!(test_label(), "test: Re 100");
        assert_eq!(car_label(), "car: Re ≈ 9,000,000");
        assert_eq!(ratio_label(), "× 90,000");
        assert_eq!(mph_label(), "30 m/s ≈ 67 mph");
        assert_eq!(decade_tex(7), "10^{7}");
        assert_eq!(scientific((3, 200_000)), Some(("1.5".to_owned(), -5)));
        assert_eq!(decimal((-1, 8)), Some("-0.125".to_owned()));
        assert_eq!(decimal((1, 3)), None);
        assert_eq!(thousands(1_000), "1,000");
    }

    #[test]
    fn frame_change_subtracts_the_car() {
        assert_eq!(frame_label("car", false).unwrap(), "car: +U");
        assert_eq!(frame_label("car", true).unwrap(), "car: 0");
        assert_eq!(frame_label("air", true).unwrap(), "air: −U");
        assert_eq!(frame_label("road", true).unwrap(), "road: −U");
        assert!(frame_label("wheel", true).is_none());
        // the rim's speed past the axle equals the road's
        let (u, r) = (1.3, 0.4);
        assert!(close(wheel_rate(u, r) * r, u));
    }

    #[test]
    fn log_bar_places_both_numbers() {
        assert!(close(log_bar::x(100.0), -4.0));
        assert!(close(log_bar::x(1e7), log_bar::right()));
        let car = log_bar::x(9e6);
        assert!(car < log_bar::right() && car > log_bar::x(1e6));
        // the gap is log10(90 000) decades
        assert!(close(car - log_bar::x(100.0), log_bar::DECADE * 90_000f64.log10()));
    }

    #[test]
    fn carry_moves_every_point_between_poses() {
        let a = Pose {
            origin: (-2.9, -3.9),
            scale: 1.3,
        };
        let b = Pose {
            origin: (-5.3, -1.0),
            scale: 1.0,
        };
        let (f, about, by) = a.carry(&b);
        for p in car::outline() {
            let s = a.at(p);
            let moved = (
                about.0 + f * (s.0 - about.0) + by.0,
                about.1 + f * (s.1 - about.1) + by.1,
            );
            let want = b.at(p);
            assert!(close(moved.0, want.0) && close(moved.1, want.1));
        }
    }

    #[test]
    fn outline_is_a_car_sized_silhouette_of_our_own() {
        let o = car::outline();
        assert!(o.iter().all(|p| p.0 >= 0.0 && p.0 <= car::length() && p.1 >= 0.19));
        assert!(close(car::length(), 4.5));
        let h = car::height();
        assert!(h > 1.42 && h < 1.46, "{h}");
        // the spline passes through its points
        let top = car::catmull_rom(&[(0.0, 0.0), (1.0, 2.0), (3.0, 1.0)], 4);
        assert_eq!(top[0], (0.0, 0.0));
        assert_eq!(top[4], (1.0, 2.0));
        assert_eq!(*top.last().unwrap(), (3.0, 1.0));
        // wheels stand on the road and sit inside their arches
        let r = car::wheel_radius();
        for (k, &(cx, cy)) in car::wheels().iter().enumerate() {
            assert!(close(cy - r, 0.0));
            let arch = (car::WHEEL_X[k] as f64 / 100.0, car::SILL as f64 / 100.0);
            let farthest = ((cx - arch.0).powi(2) + (cy - arch.1).powi(2)).sqrt() + r;
            assert!(farthest < car::ARCH_R as f64 / 100.0);
        }
        // every extrusion point lies on the outline
        for p in car::ridge_points() {
            assert!(o.iter().any(|q| close(q.0, p.0) && close(q.1, p.1)), "{p:?}");
        }
    }

    #[test]
    fn ridge_fades_both_ways_alike() {
        let d = ridge::depths();
        assert_eq!(d.len(), 2 * ridge::COPIES);
        assert!(d.iter().all(|&z| ridge::fade(z) > 0.0 && ridge::fade(z) < 1.0));
        assert!(close(ridge::fade(ridge::reach()), ridge::fade(-ridge::reach())));
        assert!(close(ridge::fade(0.0), 1.0));
    }

    #[test]
    fn around_arrow_never_enters_the_body() {
        let path = block::around();
        let (first, last) = (path[0], path[path.len() - 1]);
        assert!(close(first.2, 0.9) && close(last.2, 0.9), "mid-width at both ends");
        assert!(first.0 > 4.5 && last.0 < 0.0, "ahead of the nose, behind the tail");
        for &(x, y, z) in &path {
            let inside = (0.0..=4.5).contains(&x) && (0.2..=0.9).contains(&y) && (0.0..=1.8).contains(&z);
            assert!(!inside, "{x} {y} {z}");
            if (0.0..=4.5).contains(&x) {
                assert!(close(z, block::AROUND_SIDE), "alongside the near side");
            }
        }
        assert!(path.windows(2).all(|w| w[1].0 < w[0].0), "nose to tail");
    }

    #[test]
    fn block_car_has_width_and_sits_on_its_body() {
        let body = block::faces(block::BODY);
        let cabin = block::faces(block::CABIN);
        assert!(close(body[1][2].2 - body[1][0].2, 1.8), "1.8 m wide");
        assert_eq!(block::CABIN[1].0, block::BODY[1].1, "the cabin stands on the body");
        assert!(block::CABIN[2].0 > block::BODY[2].0 && block::CABIN[2].1 < block::BODY[2].1);
        assert!(cabin.iter().flatten().all(|p| p.0 >= 0.0 && p.0 <= 4.5));
    }

    #[test]
    fn stretching_keeps_volume_and_circulation() {
        let (r0, l0, w0) = (0.9, 3.4, 1.2);
        for l in [3.4, 4.0, 5.1, 6.8] {
            let (r, w) = (tube::radius(r0, l0, l), tube::spin(w0, l0, l));
            assert!(close(r * r * l, r0 * r0 * l0), "volume");
            assert!(close(r * r * w, r0 * r0 * w0), "circulation");
        }
        assert!(close(tube::radius(r0, l0, 2.0 * l0), r0 / 2f64.sqrt()));
        assert!(close(tube::spin(w0, l0, 2.0 * l0), 2.0 * w0));
        assert!(close(tube::smoothstep(0.5), 0.5) && tube::smoothstep(2.0) == 1.0);
        assert!(close(tube::cos_view().powi(2) + tube::SIN_VIEW.powi(2), 1.0));
    }

    #[test]
    fn slice_arrows_cross_or_lie_along_it() {
        for &(a, b) in &slice::CROSSING {
            let p = slice::pierce(a, b).expect("crosses z = 0");
            assert!(p.0 > 0.0 && p.0 < slice::W && p.1 > 0.0 && p.1 < slice::H);
        }
        for &(a, b) in &slice::ALONG {
            assert_eq!(a.2, b.2, "w = 0");
            assert!(slice::pierce(a, b).is_none());
        }
        let d = |(a, b): (slice::P3, slice::P3)| (b.0 - a.0, b.1 - a.1);
        assert_ne!(d(slice::ALONG[0]), d(slice::ALONG[1]), "different at the two depths");
        assert!(slice::ALONG[0].0.2 < 0.0 && slice::ALONG[1].0.2 > 0.0);
        let (x, y) = oblique(4.0);
        assert!(close(x, 3f64.sqrt()) && close(y, 1.0));
    }
}
