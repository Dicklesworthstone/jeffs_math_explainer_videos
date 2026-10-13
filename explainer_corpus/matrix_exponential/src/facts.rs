//! Every number Video J shows or says, computed here and checked two ways:
//! `pin` against a literal from a cited source where there is one, and
//! `./x matrix_exponential facts-check` against `oracle.py` (same keys).
//! The exponentials come from `model::expm` (scaling and squaring); the
//! oracle sums exact rational series, uses closed forms, enumerates walks
//! and asks SciPy's `expm`.
//!
//! Rows of `docs/fact-sheet.md`: F-N the number exponential; F-H the hook;
//! F-J the quarter turn; F-S does the series settle; F-T two tanks; F-G
//! walks on a network; F-D spreading (heat, chance, a two-state quantum
//! system); F-C computing it; F-O order matters; F-L the doorway to Lie
//! theory; F-X the edges of the always sentences; F-B cited literals.

use explainer_kit::Facts;
use explainer_kit::stats::Data;

use crate::model::{
    self, KITE, M2f, Mat, PAIR, compound, exp_partial, expm, expm_t, factorial, hat, pade_at_one, quarter_turn,
    rotation_defect, series, spins, su_dim, symmetric_eigenvalues, tiny_steps,
};

/// Numbers and strings quoted from the sources (fact sheet rows F-B; the
/// oracle types the same literals from the sources).
pub mod cited {
    /// Moler and Van Loan, "Nineteen Dubious Ways to Compute the Exponential
    /// of a Matrix", SIAM Review 20(4), 1978, pp. 801-836; the revisit is
    /// SIAM Review 45(1), 2003.
    pub const MVL_YEAR: u32 = 1978;
    pub const MVL_REVISIT_YEAR: u32 = 2003;
    pub const MVL_WAYS: u32 = 19;
    /// Their Method 1 example (1978, pp. 807-808).
    pub const MVL_MATRIX: [[f64; 2]; 2] = [[-49.0, 24.0], [-64.0, 31.0]];
    /// e^A "to 6 decimal places", as printed.
    pub const MVL_CORRECT: &str = "-0.735759,0.551819;-1.471518,1.103638";
    /// What their IBM 370 short-arithmetic run of the series returned, after
    /// K = 59 terms, as printed. Quoted, never recomputed: the digits belong
    /// to that machine's hexadecimal arithmetic.
    pub const MVL_IBM_RESULT: &str = "-22.25880,-1.432766;-61.49931,-3.474280";
    pub const MVL_IBM_TERMS: u32 = 59;
    /// The eigenvalues their factorization gives the matrix.
    pub const MVL_EIGENVALUES: &str = "-17,-1";
    /// Higham 2005 (SIAM J. Matrix Anal. Appl. 26(4)), Algorithm 2.3, and
    /// SciPy's `_matfuncs_expm.c`: the diagonal Padé degrees tried.
    pub const PADE_DEGREES: &str = "3,5,7,9,13";
    pub const HIGHAM_YEAR: u32 = 2005;
    pub const AL_MOHY_HIGHAM_YEAR: u32 = 2009;
    /// Estrada and Hatano, "Communicability in complex networks", Phys. Rev.
    /// E 77, 036111.
    pub const ESTRADA_HATANO_YEAR: u32 = 2008;
    /// Euler, Introductio in analysin infinitorum, vol. 1 (1748), sect. 122:
    /// e to 23 decimal places as he printed it.
    pub const EULER_YEAR: u32 = 1748;
    pub const EULER_E: &str = "2.71828182845904523536028";
    /// Jacob Bernoulli's compound-interest question, Acta Eruditorum, May 1690.
    pub const BERNOULLI_YEAR: u32 = 1690;
    /// Laguerre defines e^X by the series (1867); Peano solves x' = Rx with
    /// it (Mathematische Annalen 32, 1888).
    pub const LAGUERRE_YEAR: u32 = 1867;
    pub const PEANO_YEAR: u32 = 1888;
    /// Sophus Lie, 17 December 1842 to 18 February 1899.
    pub const LIE_BORN: u32 = 1842;
    pub const LIE_DIED: u32 = 1899;
    /// Particle Data Group, Review of Particle Physics (2026), sections 9
    /// and 10: eight gluons; W+, W−; Z; the photon.
    pub const GLUONS: u32 = 8;
    pub const W_BOSONS: u32 = 2;
    pub const Z_BOSONS: u32 = 1;
    pub const PHOTONS: u32 = 1;
}

/// The worked examples the chapters draw.
pub mod shown {
    /// Compounding one unit at 100%: the numbers of times a year.
    pub const COMPOUNDINGS: [u32; 6] = [1, 2, 4, 12, 365, 1_000_000];
    /// The growth quiz: five percent a year, for twenty years.
    pub const GROWTH: (f64, f64) = (0.05, 20.0);
    /// Turning in tiny steps: the step counts shown.
    pub const TURN_STEPS: [u32; 4] = [4, 16, 100, 1000];
    /// "Does it settle?": e to five quarter turns.
    pub const SETTLE_T: f64 = 5.0;
    pub const SETTLE_TERMS: u32 = 20;
    /// Two tanks of 100 litres, 10 litres a minute each way, 100 g of salt
    /// in the first.
    pub const TANK_LITRES: f64 = 100.0;
    pub const TANK_PUMPED: f64 = 10.0;
    pub const TANK_START: [f64; 2] = [100.0, 0.0];
    pub const TANK_TIMES: [f64; 5] = [1.0, 5.0, 10.0, 20.0, 60.0];
    pub const TANK_STEP_TIME: f64 = 10.0;
    pub const TANK_STEPS: [u32; 6] = [1, 2, 5, 10, 100, 1000];
    /// Heat starts at the tail of the kite (node 4).
    pub const HEAT_START: usize = 3;
    pub const HEAT_TIMES: [f64; 5] = [0.25, 0.5, 1.0, 2.0, 5.0];
    pub const PAIR_TIMES: [f64; 3] = [0.5, 1.0, 2.0];
    /// Scaling and squaring in 32 bits: halve eight times, eight terms.
    pub const HALVINGS: u32 = 8;
    pub const SCALED_DEGREE: u32 = 8;
    /// The gap between the two sides of the broken rule, at these times.
    pub const GAP_TIMES: [f64; 3] = [1.0, 0.1, 0.01];
    /// The little loop of four spins.
    pub const LOOP_ANGLES: [f64; 2] = [0.1, 0.01];
}

/// The edge tests draw their cases from this seed (the oracle draws the
/// same ones from NumPy's PCG64DXSM).
pub const EDGE_SEED: u64 = 20_261_013;
pub const EDGE_CASES: usize = 200;

/// `x` to `places` decimals. Refuses a value so close to a rounding tie
/// that the oracle's independent route could round it the other way, and
/// prints a rounded zero without a sign.
pub fn dec(x: f64, places: usize) -> Result<String, String> {
    if !x.is_finite() {
        return Err(format!("{x} is not a number to print"));
    }
    let scaled = x.abs() * 10_f64.powi(i32::try_from(places).map_err(|e| e.to_string())?);
    if ((scaled - scaled.floor()) - 0.5).abs() < 1e-6 {
        return Err(format!("{x} is a rounding tie at {places} places"));
    }
    let s = format!("{x:.places$}");
    let zero = s.trim_start_matches('-').chars().all(|c| c == '0' || c == '.');
    Ok(if zero { s.trim_start_matches('-').to_owned() } else { s })
}

/// Numbers to `places` decimals, joined by commas.
pub fn row(v: &[f64], places: usize) -> Result<String, String> {
    Ok(v.iter()
        .map(|&x| dec(x, places))
        .collect::<Result<Vec<_>, _>>()?
        .join(","))
}

/// A matrix to `places` decimals: rows joined by `;`, entries by `,`.
pub fn mat(m: &Mat, places: usize) -> Result<String, String> {
    Ok((0..m.n())
        .map(|i| row(m.row(i), places))
        .collect::<Result<Vec<_>, _>>()?
        .join(";"))
}

/// A matrix whose entries are whole numbers, printed as such.
pub fn whole(m: &Mat) -> Result<String, String> {
    let entry = |x: f64| {
        let r = x.round();
        if (x - r).abs() > 1e-9 {
            return Err(format!("{x} is not a whole number"));
        }
        Ok(format!("{}", r as i64))
    };
    Ok((0..m.n())
        .map(|i| {
            Ok(m.row(i)
                .iter()
                .map(|&x| entry(x))
                .collect::<Result<Vec<_>, String>>()?
                .join(","))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join(";"))
}

fn counts(rows: &[Vec<u64>]) -> String {
    rows.iter()
        .map(|r| r.iter().map(u64::to_string).collect::<Vec<_>>().join(","))
        .collect::<Vec<_>>()
        .join(";")
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// A time as a key part: `0.25` → `0.25`, `5.0` → `5`.
fn tkey(t: f64) -> String {
    if t.fract() == 0.0 {
        format!("{t:.0}")
    } else {
        format!("{t}")
    }
}

/// Equal within `tol`, relative to the larger of 1 and the entries' size.
fn same(a: &Mat, b: &Mat, tol: f64) -> bool {
    a.sub(b).max_abs() <= tol * a.max_abs().max(b.max_abs()).max(1.0)
}

pub fn shear_right() -> Mat {
    Mat::from_rows(&[&[0.0, 1.0], &[0.0, 0.0]])
}

pub fn shear_up() -> Mat {
    Mat::from_rows(&[&[0.0, 0.0], &[1.0, 0.0]])
}

pub fn mvl() -> Mat {
    let a = cited::MVL_MATRIX;
    Mat::from_rows(&[&a[0], &a[1]])
}

fn mvl_f32() -> M2f {
    let a = cited::MVL_MATRIX;
    [[a[0][0] as f32, a[0][1] as f32], [a[1][0] as f32, a[1][1] as f32]]
}

fn widen(a: &M2f) -> Mat {
    Mat::from_rows(&[
        &[f64::from(a[0][0]), f64::from(a[0][1])],
        &[f64::from(a[1][0]), f64::from(a[1][1])],
    ])
}

/// The three spins about the axes.
pub fn spin_axes() -> [Mat; 3] {
    [hat([1.0, 0.0, 0.0]), hat([0.0, 1.0, 0.0]), hat([0.0, 0.0, 1.0])]
}

/// The real 2n by 2n matrix that acts as −iH on (real parts, imaginary
/// parts): multiplying by i is a quarter turn of each pair.
pub fn minus_i_times(h: &Mat) -> Mat {
    let n = h.n();
    let mut m = Mat::zeros(2 * n);
    for i in 0..n {
        for j in 0..n {
            m.set(i, n + j, h.get(i, j));
            m.set(n + i, j, -h.get(i, j));
        }
    }
    m
}

/// The chance of finding a two-state quantum system (Hamiltonian the pair's
/// Laplacian, ħ = 1) at each node at time t, having started at the first.
pub fn quantum_pair(t: f64) -> [f64; 2] {
    let u = expm_t(&minus_i_times(&PAIR.laplacian()), t);
    let psi = u.column(0);
    [psi[0].powi(2) + psi[2].powi(2), psi[1].powi(2) + psi[3].powi(2)]
}

/// A seeded n by n matrix with entries uniform in [−3, 3), row by row.
pub fn random_matrix(data: &mut Data, n: usize) -> Mat {
    let mut m = Mat::zeros(n);
    for i in 0..n {
        for j in 0..n {
            m.set(i, j, data.uniform(-3.0, 3.0));
        }
    }
    m
}

/// One case of the edge tests: two matrices of the same size (2, 3 or 4)
/// and two times in [0, 2).
pub struct EdgeCase {
    pub a: Mat,
    pub b: Mat,
    pub s: f64,
    pub t: f64,
}

pub fn edge_cases() -> Vec<EdgeCase> {
    let mut data = Data::new(EDGE_SEED);
    (0..EDGE_CASES)
        .map(|i| {
            let n = 2 + i % 3;
            let a = random_matrix(&mut data, n);
            let b = random_matrix(&mut data, n);
            let s = data.uniform(0.0, 2.0);
            let t = data.uniform(0.0, 2.0);
            EdgeCase { a, b, s, t }
        })
        .collect()
}

/// How many of the edge cases satisfy each always sentence (fact sheet rows
/// F-X): `(name, holds, checked)`.
pub fn edge_counts() -> Vec<(&'static str, usize, usize)> {
    let cases = edge_cases();
    let count = |f: &dyn Fn(&EdgeCase) -> bool| cases.iter().filter(|c| f(c)).count();
    let n = cases.len();
    let mut out = vec![
        // every term of the series is at most size(A)^k / k!, and the
        // running total reaches the exponential
        (
            "series_settles",
            count(&|c| {
                let size = c.a.size();
                let steps = series(&c.a, 120);
                let under = steps
                    .iter()
                    .take(41)
                    .all(|s| s.term.size() <= size.powi(s.k as i32) / factorial(s.k) as f64 * (1.0 + 1e-9) + 1e-12);
                under && same(&steps.last().expect("a last term").total, &expm(&c.a), 1e-9)
            }),
            n,
        ),
        // size(AB) <= size(A) size(B)
        (
            "sizes_multiply",
            count(&|c| c.a.mul(&c.b).size() <= c.a.size() * c.b.size() * (1.0 + 1e-12)),
            n,
        ),
        // running for s and then for t is running for s + t
        (
            "flow_adds",
            count(&|c| {
                same(
                    &expm_t(&c.a, c.s + c.t),
                    &expm_t(&c.a, c.s).mul(&expm_t(&c.a, c.t)),
                    1e-9,
                )
            }),
            n,
        ),
        // when the two matrices commute, the rule of exponents holds
        (
            "commuting_rule_holds",
            count(&|c| {
                let b = c.a.mul(&c.a).scale(0.1).sub(&c.a.scale(0.5));
                c.a.commutator(&b).max_abs() <= 1e-9 && same(&expm(&c.a.add(&b)), &expm(&c.a).mul(&expm(&b)), 1e-8)
            }),
            n,
        ),
        // for these generic pairs, which do not commute, it fails
        (
            "generic_rule_fails",
            count(&|c| {
                c.a.commutator(&c.b).max_abs() > 1e-3
                    && !same(&expm(&c.a.add(&c.b)), &expm(&c.a).mul(&expm(&c.b)), 1e-3)
            }),
            n,
        ),
        // the exponential of a skew matrix is a rotation
        (
            "skew_gives_rotation",
            count(&|c| {
                let (defect, det) = rotation_defect(&expm(&c.a.sub(&c.a.transpose())));
                defect < 1e-11 && (det - 1.0).abs() < 1e-11
            }),
            n,
        ),
        // e^A is undone by e^{-A}, and its determinant is e^{trace}
        (
            "exponential_undone",
            count(&|c| {
                let a = c.a.scale(0.25);
                let e = expm(&a);
                same(&e.mul(&expm(&a.scale(-1.0))), &Mat::identity(a.n()), 1e-10)
                    && (e.det() - a.trace().exp()).abs() <= 1e-10 * a.trace().exp().max(1.0)
            }),
            n,
        ),
    ];
    // heat on the two networks: nothing created, nothing destroyed, nothing negative
    let times: Vec<f64> = (1..=50).map(|m| 0.1 * f64::from(m)).collect();
    let heat_ok = [KITE, PAIR]
        .iter()
        .flat_map(|g| times.iter().map(move |&t| expm_t(&g.laplacian(), -t)))
        .filter(|h| {
            (0..h.n()).all(|j| {
                let col = h.column(j);
                (col.iter().sum::<f64>() - 1.0).abs() < 1e-12 && col.iter().all(|&x| x > 0.0)
            })
        })
        .count();
    out.push(("heat_conserves", heat_ok, 2 * times.len()));
    // the quantum pair: total chance stays one
    let unitary_ok = times
        .iter()
        .filter(|&&t| (quantum_pair(t).iter().sum::<f64>() - 1.0).abs() < 1e-12)
        .count();
    out.push(("quantum_total_one", unitary_ok, times.len()));
    out
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    number(&mut f)?;
    hook(&mut f)?;
    quarter(&mut f)?;
    settles(&mut f)?;
    two_tanks(&mut f)?;
    network(&mut f)?;
    spreading(&mut f)?;
    computing(&mut f)?;
    order(&mut f)?;
    lie(&mut f)?;
    edges(&mut f)?;
    sources(&mut f)?;
    Ok(f)
}

/// F-N: the number exponential.
fn number(f: &mut Facts) -> Result<(), String> {
    let e = std::f64::consts::E;
    f.add("F-N.e", dec(e, 6)?)?;
    f.pin(
        "F-N.e.digits15",
        format!("{e:.15}"),
        &cited::EULER_E[..17],
        "Euler, Introductio (1748), sect. 122",
    )?;
    for n in shown::COMPOUNDINGS {
        f.add(&format!("F-N.compound.n{n}"), dec(compound(1.0, n), 5)?)?;
    }
    for k in 0..=10 {
        f.add(&format!("F-N.series.k{k}"), dec(exp_partial(1.0, k), 6)?)?;
    }
    f.add("F-N.factorials", list((0..=6).map(factorial)))?;
    let (rate, years) = shown::GROWTH;
    f.add("F-N.growth.rate", rate)?;
    f.add("F-N.growth.years", years)?;
    f.add("F-N.growth.factor", dec((rate * years).exp(), 6)?)?;
    f.add("F-N.growth.doubling_years", dec(2.0_f64.ln() / rate, 4)?)?;
    f.add("F-N.add.e2", dec(2.0_f64.exp(), 6)?)?;
    f.add("F-N.add.e3", dec(3.0_f64.exp(), 6)?)?;
    f.add("F-N.add.product", dec(2.0_f64.exp() * 3.0_f64.exp(), 6)?)?;
    f.add("F-N.add.e5", dec(5.0_f64.exp(), 6)?)?;
    // turning: e^{it} is the first column of e^{tJ}
    let j = quarter_turn();
    f.add("F-N.turn.t1", row(&expm(&j).column(0), 6)?)?;
    f.add("F-N.turn.pi", row(&expm_t(&j, std::f64::consts::PI).column(0), 6)?)?;
    for n in shown::TURN_STEPS {
        let stepped = tiny_steps(&j, std::f64::consts::PI, n);
        f.add(&format!("F-N.turn.steps.n{n}"), row(&stepped.column(0), 4)?)?;
    }
    f.add("F-N.turn.radian_degrees", dec(1.0_f64.to_degrees(), 4)?)?;
    Ok(())
}

/// F-H: the hook, e to each entry.
fn hook(f: &mut Facts) -> Result<(), String> {
    let zero = Mat::zeros(2);
    let guess = zero.entrywise_exp();
    f.add("F-H.zero.guess", whole(&guess)?)?;
    f.add("F-H.zero.guess.det", dec(guess.det(), 6)?)?;
    f.add("F-H.zero.true", whole(&expm(&zero))?)?;
    let j = quarter_turn();
    f.add("F-H.J.guess", mat(&j.entrywise_exp(), 6)?)?;
    f.add("F-H.J.true", mat(&expm(&j), 6)?)?;
    // the guess breaks "run twice = run for twice as long"; the series keeps it
    let twice = j.entrywise_exp().mul(&j.entrywise_exp());
    f.add("F-H.twice.guess", mat(&twice, 6)?)?;
    f.add("F-H.twice.guess.doubled", mat(&j.scale(2.0).entrywise_exp(), 6)?)?;
    f.add(
        "F-H.twice.true",
        same(&expm(&j).mul(&expm(&j)), &expm_t(&j, 2.0), 1e-12),
    )?;
    Ok(())
}

/// F-J: the quarter turn.
fn quarter(f: &mut Facts) -> Result<(), String> {
    let j = quarter_turn();
    for k in 1..=4 {
        f.add(&format!("F-J.power.k{k}"), whole(&j.pow(k))?)?;
    }
    let period = (1..=8).find(|&k| j.pow(k) == Mat::identity(2)).ok_or("no period")?;
    f.add("F-J.period", period)?;
    let pi = std::f64::consts::PI;
    f.add("F-J.exp.quarter", mat(&expm_t(&j, pi / 2.0), 6)?)?;
    f.add("F-J.exp.half", mat(&expm_t(&j, pi), 6)?)?;
    f.add("F-J.exp.full", mat(&expm_t(&j, 2.0 * pi), 6)?)?;
    // the series splits: even terms are multiples of I, odd terms of J
    let steps = series(&j, 30);
    let even: f64 = steps.iter().filter(|s| s.k % 2 == 0).map(|s| s.term.get(0, 0)).sum();
    let odd: f64 = steps.iter().filter(|s| s.k % 2 == 1).map(|s| s.term.get(1, 0)).sum();
    f.add("F-J.split.even", dec(even, 6)?)?;
    f.add("F-J.split.odd", dec(odd, 6)?)?;
    Ok(())
}

/// F-S: does the series settle? e to five quarter turns.
fn settles(f: &mut Facts) -> Result<(), String> {
    let t = shown::SETTLE_T;
    let a = quarter_turn().scale(t);
    let steps = series(&a, shown::SETTLE_TERMS);
    let sizes: Vec<f64> = steps.iter().map(|s| s.term.size()).collect();
    f.add("F-S.t", t)?;
    f.add("F-S.size", dec(a.size(), 6)?)?;
    f.add("F-S.terms", row(&sizes, 5)?)?;
    let peak = sizes.iter().copied().fold(0.0, f64::max);
    f.add("F-S.peak", dec(peak, 5)?)?;
    f.add(
        "F-S.peak.k",
        list(
            sizes
                .iter()
                .enumerate()
                .filter(|(_, s)| (*s - peak).abs() < 1e-9)
                .map(|(k, _)| k),
        ),
    )?;
    for s in &steps {
        f.add(&format!("F-S.total.k{}", s.k), row(&s.total.column(0), 4)?)?;
    }
    let radius = |s: &model::SeriesStep| s.total.column(0).iter().map(|x| x * x).sum::<f64>().sqrt();
    let far = steps
        .iter()
        .max_by(|a, b| radius(a).total_cmp(&radius(b)))
        .ok_or("no steps")?;
    f.add("F-S.farthest.k", far.k)?;
    f.add("F-S.farthest.radius", dec(radius(far), 4)?)?;
    f.add("F-S.limit", row(&expm(&a).column(0), 6)?)?;
    f.add("F-S.bound", dec(t.exp(), 6)?)?;
    let below = sizes
        .iter()
        .position(|&s| s < 0.001)
        .ok_or("never below a thousandth")?;
    f.add("F-S.first_below_thousandth.k", below)?;
    f.add("F-S.term.k20", dec(sizes[20], 7)?)?;
    // the size of a product is at most the product of the sizes: a shear
    let shear = Mat::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]]);
    f.add("F-S.shear.size", dec(shear.size(), 6)?)?;
    f.add("F-S.shear.squared.size", dec(shear.pow(2).size(), 6)?)?;
    f.add("F-S.shear.size.squared", dec(shear.size().powi(2), 6)?)?;
    let nil = shear_right();
    f.add("F-S.nil.size", dec(nil.size(), 6)?)?;
    f.add("F-S.nil.squared", whole(&nil.pow(2))?)?;
    Ok(())
}

/// F-T: two tanks.
fn two_tanks(f: &mut Facts) -> Result<(), String> {
    let a = model::tanks(shown::TANK_LITRES, shown::TANK_PUMPED);
    let start = shown::TANK_START;
    f.add("F-T.rule", mat(&a, 1)?)?;
    f.add("F-T.start", row(&start, 0)?)?;
    for t in shown::TANK_TIMES {
        f.add(&format!("F-T.at.t{}", tkey(t)), row(&expm_t(&a, t).apply(&start), 4)?)?;
    }
    for n in shown::TANK_STEPS {
        let stepped = tiny_steps(&a, shown::TANK_STEP_TIME, n).apply(&start);
        f.add(&format!("F-T.steps.n{n}"), row(&stepped, 4)?)?;
    }
    let total: f64 = expm_t(&a, 7.0).apply(&start).iter().sum();
    f.add("F-T.total", dec(total, 6)?)?;
    f.add("F-T.settles", row(&expm_t(&a, 1000.0).apply(&start), 4)?)?;
    f.add("F-T.eigenvalues", row(&symmetric_eigenvalues(&a), 6)?)?;
    // the first tank is within a gram of its final 50 when 50 e^{-0.2 t} = 1
    f.add("F-T.within_a_gram.minutes", dec(50.0_f64.ln() / 0.2, 4)?)?;
    Ok(())
}

/// F-G: walks on the kite.
fn network(f: &mut Facts) -> Result<(), String> {
    f.add("F-G.nodes", KITE.n)?;
    f.add(
        "F-G.links",
        list(KITE.links.iter().map(|(i, j)| format!("{}-{}", i + 1, j + 1))),
    )?;
    f.add("F-G.degrees", list((0..KITE.n).map(|i| KITE.degree(i))))?;
    for k in 1..=4 {
        f.add(&format!("F-G.walks.k{k}"), counts(&KITE.walks(k)))?;
    }
    let a = KITE.adjacency();
    let e = expm(&a);
    f.add("F-G.exp", mat(&e, 4)?)?;
    f.add("F-G.exp.trace", dec(e.trace(), 4)?)?;
    // node 1 to node 4, walk length by walk length
    let mut partial = 0.0;
    let mut running = Vec::new();
    for k in 0..=8 {
        partial += KITE.walks(k)[0][3] as f64 / factorial(k) as f64;
        running.push(partial);
    }
    f.add("F-G.exp.one_to_four.running", row(&running, 4)?)?;
    f.add("F-G.walks.one_to_four", list((0..=8).map(|k| KITE.walks(k)[0][3])))?;
    Ok(())
}

/// F-D: spreading. Heat and chance on the kite; the pair; one letter i.
fn spreading(f: &mut Facts) -> Result<(), String> {
    let l = KITE.laplacian();
    f.add("F-D.L", whole(&l)?)?;
    for t in shown::HEAT_TIMES {
        let heat = expm_t(&l, -t).column(shown::HEAT_START);
        f.add(&format!("F-D.heat.t{}", tkey(t)), row(&heat, 4)?)?;
    }
    f.add(
        "F-D.heat.settles",
        row(&expm_t(&l, -200.0).column(shown::HEAT_START), 4)?,
    )?;
    // the hopper's rule is Q = −L: every row sums to zero, and e^{tQ} has rows that sum to one
    let q = l.scale(-1.0);
    let rows_zero = (0..q.n()).all(|i| q.row(i).iter().sum::<f64>() == 0.0);
    f.add("F-D.chance.rows_sum_to_zero", rows_zero)?;
    f.add("F-D.chance.from_hub.t1", row(expm_t(&q, 1.0).row(2), 4)?)?;
    f.add("F-D.chance.from_hub.settles", row(expm_t(&q, 200.0).row(2), 4)?)?;
    // the pair: the two tanks are heat on two nodes
    let pair = PAIR.laplacian();
    f.add("F-D.pair.L", whole(&pair)?)?;
    let tank_rule = model::tanks(shown::TANK_LITRES, shown::TANK_PUMPED);
    f.add("F-D.pair.is_the_tanks", same(&tank_rule, &pair.scale(-0.1), 1e-15))?;
    for t in shown::PAIR_TIMES {
        f.add(
            &format!("F-D.pair.heat.t{}", tkey(t)),
            row(&expm_t(&pair, -t).column(0), 6)?,
        )?;
        f.add(&format!("F-D.pair.quantum.t{}", tkey(t)), row(&quantum_pair(t), 6)?)?;
    }
    let pi = std::f64::consts::PI;
    f.add("F-D.pair.quantum.quarter", row(&quantum_pair(pi / 4.0), 6)?)?;
    f.add("F-D.pair.quantum.half", row(&quantum_pair(pi / 2.0), 6)?)?;
    f.add("F-D.pair.quantum.full", row(&quantum_pair(pi), 6)?)?;
    Ok(())
}

/// F-C: computing it.
fn computing(f: &mut Facts) -> Result<(), String> {
    let a = mvl();
    f.add("F-C.mvl.A", whole(&a)?)?;
    // a 2 by 2's eigenvalues from its trace and determinant
    let (tr, det) = (a.trace(), a.det());
    let root = (tr * tr - 4.0 * det).sqrt();
    f.pin(
        "F-C.mvl.eigenvalues",
        format!("{},{}", dec((tr - root) / 2.0, 0)?, dec((tr + root) / 2.0, 0)?),
        cited::MVL_EIGENVALUES,
        "Moler and Van Loan (1978), p. 807",
    )?;
    let exact = expm(&a);
    f.pin(
        "F-C.mvl.exact",
        mat(&exact, 6)?,
        cited::MVL_CORRECT,
        "Moler and Van Loan (1978), p. 807",
    )?;
    // the terms grow to millions before they cancel
    let steps = series(&a, 80);
    for k in [16_usize, 17] {
        f.add(&format!("F-C.mvl.term.k{k}"), mat(&steps[k].term, 1)?)?;
    }
    let largest = steps.iter().map(|s| s.term.max_abs()).fold(0.0, f64::max);
    f.add("F-C.mvl.term.largest", dec(largest, 1)?)?;
    f.add("F-C.mvl.size", dec(a.size(), 4)?)?;
    // the series in 32-bit arithmetic
    let (wrong, terms) = model::series_f32(&mvl_f32());
    let wrong = widen(&wrong);
    f.add("F-C.mvl.f32.series", mat(&wrong, 6)?)?;
    f.add("F-C.mvl.f32.terms", terms)?;
    f.add("F-C.mvl.f32.worst_error", dec(wrong.sub(&exact).max_abs(), 4)?)?;
    let wrong_signs = wrong
        .entries()
        .iter()
        .zip(exact.entries())
        .filter(|(w, x)| w.signum() != x.signum())
        .count();
    f.add("F-C.mvl.f32.wrong_signs", wrong_signs)?;
    // scaling and squaring, same 32 bits
    let scaled = widen(&model::scale_square_f32(
        &mvl_f32(),
        shown::HALVINGS,
        shown::SCALED_DEGREE,
    ));
    f.add("F-C.mvl.f32.scaled", mat(&scaled, 6)?)?;
    f.add("F-C.mvl.f32.scaled.halvings", shown::HALVINGS)?;
    f.add("F-C.mvl.f32.scaled.divisor", 2_u32.pow(shown::HALVINGS))?;
    f.add("F-C.mvl.f32.scaled.terms", shown::SCALED_DEGREE)?;
    f.add("F-C.mvl.f32.scaled.worst_error", dec(scaled.sub(&exact).max_abs(), 6)?)?;
    // Padé: a ratio of two polynomials, at x = 1
    for m in 1..=4 {
        let (p, q) = pade_at_one(m);
        f.add(&format!("F-C.pade.m{m}"), format!("{p}/{q}"))?;
        f.add(&format!("F-C.pade.m{m}.value"), dec(p as f64 / q as f64, 6)?)?;
    }
    let e = std::f64::consts::E;
    let (p3, q3) = pade_at_one(3);
    let pade_error = (p3 as f64 / q3 as f64 - e).abs();
    let series_error = (exp_partial(1.0, 6) - e).abs();
    f.add("F-C.pade.m3.error", dec(pade_error, 6)?)?;
    f.add("F-C.series.k6.error", dec(series_error, 6)?)?;
    f.add("F-C.pade.m3.times_closer", dec(series_error / pade_error, 1)?)?;
    // diagonalizing: the tanks' eigenvalues, exponentiated
    let tanks = model::tanks(shown::TANK_LITRES, shown::TANK_PUMPED);
    let eig = symmetric_eigenvalues(&tanks);
    let t = shown::TANK_STEP_TIME;
    f.add(
        "F-C.diag.tanks.exp_eigenvalues.t10",
        row(&eig.iter().map(|l| (l * t).exp()).collect::<Vec<_>>(), 6)?,
    )?;
    // the tanks' eigenvectors, as rows: (1, −1) for −0.2 and (1, 1) for 0,
    // each over root 2
    let h = 0.5_f64.sqrt();
    let v = Mat::from_rows(&[&[h, -h], &[h, h]]);
    let mut d = Mat::zeros(2);
    d.set(0, 0, (eig[0] * t).exp());
    d.set(1, 1, (eig[1] * t).exp());
    let rebuilt = v.transpose().mul(&d).mul(&v);
    f.add("F-C.diag.tanks.rebuilt", same(&rebuilt, &expm_t(&tanks, t), 1e-12))?;
    // a matrix that cannot be diagonalized: its series stops by itself
    let nil = shear_right();
    f.add("F-C.nil.exp.t3", whole(&expm_t(&nil, 3.0))?)?;
    Ok(())
}

/// F-O: order matters.
fn order(f: &mut Facts) -> Result<(), String> {
    let (a, b) = (shear_right(), shear_up());
    f.add("F-O.A", whole(&a)?)?;
    f.add("F-O.B", whole(&b)?)?;
    f.add("F-O.expA", whole(&expm(&a))?)?;
    f.add("F-O.expB", whole(&expm(&b))?)?;
    f.add("F-O.expA_expB", whole(&expm(&a).mul(&expm(&b)))?)?;
    f.add("F-O.expB_expA", whole(&expm(&b).mul(&expm(&a)))?)?;
    f.add("F-O.exp_sum", mat(&expm(&a.add(&b)), 6)?)?;
    f.add("F-O.AB", whole(&a.mul(&b))?)?;
    f.add("F-O.BA", whole(&b.mul(&a))?)?;
    f.add("F-O.commutator", whole(&a.commutator(&b))?)?;
    for t in shown::GAP_TIMES {
        let gap = expm_t(&a, t).mul(&expm_t(&b, t)).sub(&expm_t(&a.add(&b), t));
        f.add(&format!("F-O.gap.t{}", tkey(t)), mat(&gap.scale(2.0 / (t * t)), 3)?)?;
    }
    // a pair that does commute
    let c = Mat::from_rows(&[&[1.0, 2.0], &[2.0, 1.0]]);
    let d = Mat::from_rows(&[&[3.0, 1.0], &[1.0, 3.0]]);
    f.add("F-O.commuting.commutator", whole(&c.commutator(&d))?)?;
    f.add(
        "F-O.commuting.rule_holds",
        same(&expm(&c.add(&d)), &expm(&c).mul(&expm(&d)), 1e-12),
    )?;
    // the rule can hold without commuting: 3, 4 and 5 full turns
    let tau = std::f64::consts::TAU;
    let x = hat([3.0 * tau, 0.0, 0.0]);
    let y = hat([0.0, 4.0 * tau, 0.0]);
    let id = Mat::identity(3);
    f.add("F-O.turns.x_is_identity", same(&expm(&x), &id, 1e-9))?;
    f.add("F-O.turns.y_is_identity", same(&expm(&y), &id, 1e-9))?;
    f.add("F-O.turns.sum_is_identity", same(&expm(&x.add(&y)), &id, 1e-9))?;
    f.add(
        "F-O.turns.sum_full_turns",
        dec((3.0_f64.powi(2) + 4.0_f64.powi(2)).sqrt(), 6)?,
    )?;
    f.add("F-O.turns.commute", x.commutator(&y).max_abs() < 1e-9)?;
    Ok(())
}

/// F-L: the doorway to Lie theory.
fn lie(f: &mut Facts) -> Result<(), String> {
    let [lx, ly, lz] = spin_axes();
    f.add("F-L.Lz", whole(&lz)?)?;
    f.add("F-L.bracket.xy", whole(&lx.commutator(&ly))?)?;
    f.add("F-L.bracket.xy_is_z", lx.commutator(&ly) == lz)?;
    f.add("F-L.bracket.yz_is_x", ly.commutator(&lz) == lx)?;
    f.add("F-L.bracket.zx_is_y", lz.commutator(&lx) == ly)?;
    f.add("F-L.exp.Lz.t1", mat(&expm(&lz), 6)?)?;
    for eps in shown::LOOP_ANGLES {
        let there = expm_t(&lx, eps).mul(&expm_t(&ly, eps));
        let back = expm_t(&lx, -eps).mul(&expm_t(&ly, -eps));
        let around = there.mul(&back).sub(&Mat::identity(3)).scale(1.0 / (eps * eps));
        f.add(&format!("F-L.loop.t{}", tkey(eps)), mat(&around, 3)?)?;
    }
    for n in 2..=5 {
        f.add(&format!("F-L.spins.n{n}"), spins(n))?;
    }
    f.add("F-L.su3", su_dim(3))?;
    f.add("F-L.su2", su_dim(2))?;
    f.add("F-L.u1", 1)?;
    let total = su_dim(3) + su_dim(2) + 1;
    f.pin(
        "F-L.standard_model.steps",
        total,
        &(cited::GLUONS + cited::W_BOSONS + cited::Z_BOSONS + cited::PHOTONS).to_string(),
        "Particle Data Group (2026), sects. 9 and 10: 8 gluons, W+, W-, Z, photon",
    )?;
    // a full turn and no turn are the same motion: exp is not one-to-one
    let j = quarter_turn();
    f.add(
        "F-L.full_turn_is_identity",
        same(&expm_t(&j, std::f64::consts::TAU), &Mat::identity(2), 1e-12),
    )?;
    Ok(())
}

/// F-X: the edges of the always sentences, as "how many hold of how many
/// checked".
fn edges(f: &mut Facts) -> Result<(), String> {
    f.add("F-X.seed", EDGE_SEED)?;
    for (name, holds, checked) in edge_counts() {
        f.add(&format!("F-X.{name}"), format!("{holds} of {checked}"))?;
    }
    Ok(())
}

/// F-B: cited literals.
fn sources(f: &mut Facts) -> Result<(), String> {
    f.add("F-B.mvl.year", cited::MVL_YEAR)?;
    f.add("F-B.mvl.revisit_year", cited::MVL_REVISIT_YEAR)?;
    f.add("F-B.mvl.ways", cited::MVL_WAYS)?;
    f.add("F-B.mvl.ibm.result", cited::MVL_IBM_RESULT)?;
    f.add("F-B.mvl.ibm.terms", cited::MVL_IBM_TERMS)?;
    f.add("F-B.pade.degrees", cited::PADE_DEGREES)?;
    f.add("F-B.higham.year", cited::HIGHAM_YEAR)?;
    f.add("F-B.al_mohy_higham.year", cited::AL_MOHY_HIGHAM_YEAR)?;
    f.add("F-B.estrada_hatano.year", cited::ESTRADA_HATANO_YEAR)?;
    f.add("F-B.euler.year", cited::EULER_YEAR)?;
    f.add("F-B.euler.e", cited::EULER_E)?;
    f.add("F-B.bernoulli.year", cited::BERNOULLI_YEAR)?;
    f.add("F-B.laguerre.year", cited::LAGUERRE_YEAR)?;
    f.add("F-B.peano.year", cited::PEANO_YEAR)?;
    f.add("F-B.lie.born", cited::LIE_BORN)?;
    f.add("F-B.lie.died", cited::LIE_DIED)?;
    f.add(
        "F-B.bosons",
        list([cited::GLUONS, cited::W_BOSONS, cited::Z_BOSONS, cited::PHOTONS]),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fact_computes_and_every_pin_holds() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 150, "{} facts", f.len());
    }

    #[test]
    fn dec_refuses_ties_and_drops_the_sign_of_zero() {
        assert_eq!(dec(2.25, 2).unwrap(), "2.25");
        assert_eq!(dec(-1.2e-17, 6).unwrap(), "0.000000");
        assert_eq!(dec(-0.5, 3).unwrap(), "-0.500");
        assert!(dec(2.5, 0).is_err());
        assert!(dec(f64::NAN, 2).is_err());
    }

    /// Every always sentence holds on every edge case (and the one "can
    /// fail" sentence fails on every generic pair).
    #[test]
    fn edge_every_always_sentence_holds_on_every_case() {
        for (name, holds, checked) in edge_counts() {
            assert_eq!(holds, checked, "{name}");
        }
    }

    /// The rule of exponents holds for all times exactly when the matrices
    /// commute: where the commutator is not zero, the gap grows like
    /// t²/2 times it.
    #[test]
    fn edge_the_gap_is_half_t_squared_times_the_commutator() {
        for c in edge_cases().iter().take(60) {
            let t = 1e-3;
            let gap = expm_t(&c.a, t).mul(&expm_t(&c.b, t)).sub(&expm_t(&c.a.add(&c.b), t));
            let want = c.a.commutator(&c.b).scale(t * t / 2.0);
            assert!(gap.sub(&want).max_abs() <= 2e-2 * want.max_abs(), "{:?}", c.a);
        }
    }

    /// Narration truth: the 32-bit series is wrong in the first digit of
    /// every entry, and scaling and squaring in the same 32 bits is right
    /// to four decimal places.
    #[test]
    fn the_thirty_two_bit_series_fails_and_scaling_and_squaring_does_not() {
        let exact = expm(&mvl());
        let (wrong, _) = model::series_f32(&mvl_f32());
        let wrong = widen(&wrong);
        for (w, x) in wrong.entries().iter().zip(exact.entries()) {
            assert!((w - x).abs() > 0.05, "{w} against {x}");
        }
        let scaled = widen(&model::scale_square_f32(
            &mvl_f32(),
            shown::HALVINGS,
            shown::SCALED_DEGREE,
        ));
        assert!(scaled.sub(&exact).max_abs() < 5e-5);
    }

    /// The quantum pair sloshes: all the chance is back at the start after
    /// time π, and all of it is at the other node at π/2.
    #[test]
    fn the_quantum_pair_sloshes_and_heat_settles() {
        let pi = std::f64::consts::PI;
        assert!((quantum_pair(pi / 2.0)[1] - 1.0).abs() < 1e-12);
        assert!((quantum_pair(pi)[0] - 1.0).abs() < 1e-12);
        for t in [0.3, 1.7, 4.0_f64] {
            assert!((quantum_pair(t)[0] - t.cos().powi(2)).abs() < 1e-12);
            let heat = expm_t(&PAIR.laplacian(), -t).get(0, 0);
            assert!((heat - (1.0 + (-2.0 * t).exp()) / 2.0).abs() < 1e-12);
        }
    }
}
