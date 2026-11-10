//! The cross-platform vector gate (fm-7y6): every function must land
//! within its documented ULP bound of the committed mpmath ground truth in
//! `vectors/` — the same files on every certified target, which is what
//! makes bit-identity a CI property instead of a hope.
//!
//! Bounds asserted here are the crate's documented accuracy claims.
//! Tighten only with evidence; loosening requires a doc update in the
//! owning module.

use std::fmt::Write as _;

fn vectors(name: &str) -> Vec<Vec<u64>> {
    let path = format!("{}/vectors/{name}.txt", env!("CARGO_MANIFEST_DIR"));
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| {
            l.split('\t')
                .map(|w| u64::from_str_radix(w, 16).expect("hex bits"))
                .collect()
        })
        .collect()
}

/// Monotone key: total order over f64 bit patterns (−∞ … −0, +0 … +∞).
fn key(x: f64) -> u64 {
    let b = x.to_bits();
    if b >> 63 == 1 { !b } else { b | (1 << 63) }
}

fn ulp_diff(a: f64, b: f64) -> u64 {
    if a.to_bits() == b.to_bits() {
        return 0;
    }
    key(a).abs_diff(key(b))
}

fn check_single(name: &str, f: fn(f64) -> f64, bound: u64) {
    let mut max_seen = 0u64;
    let mut failures = String::new();
    let mut count = 0usize;
    for row in vectors(name) {
        let x = f64::from_bits(row[0]);
        let expected = f64::from_bits(row[1]);
        let actual = f(x);
        let diff = ulp_diff(actual, expected);
        max_seen = max_seen.max(diff);
        count += 1;
        if diff > bound {
            let _ = writeln!(
                failures,
                "  {name}({x:e}) = {actual:e} vs {expected:e} ({diff} ulp)"
            );
        }
    }
    assert!(
        failures.is_empty(),
        "{name}: exceeded {bound}-ulp bound (max {max_seen} over {count} vectors):\n{failures}"
    );
    println!("{name}: max {max_seen} ulp over {count} vectors (bound {bound})");
}

fn check_pair(name: &str, f: fn(f64, f64) -> f64, bound: u64) {
    let mut max_seen = 0u64;
    let mut failures = String::new();
    let mut count = 0usize;
    for row in vectors(name) {
        let x = f64::from_bits(row[0]);
        let y = f64::from_bits(row[1]);
        let expected = f64::from_bits(row[2]);
        let actual = f(x, y);
        let diff = ulp_diff(actual, expected);
        max_seen = max_seen.max(diff);
        count += 1;
        if diff > bound {
            let _ = writeln!(
                failures,
                "  {name}({x:e}, {y:e}) = {actual:e} vs {expected:e} ({diff} ulp)"
            );
        }
    }
    assert!(
        failures.is_empty(),
        "{name}: exceeded {bound}-ulp bound (max {max_seen} over {count} vectors):\n{failures}"
    );
    println!("{name}: max {max_seen} ulp over {count} vectors (bound {bound})");
}

#[test]
fn vec_sin() {
    check_single("sin", fmn_dmath::sin, 1);
}
#[test]
fn vec_cos() {
    check_single("cos", fmn_dmath::cos, 1);
}
#[test]
fn vec_tan() {
    check_single("tan", fmn_dmath::tan, 2);
}
#[test]
fn vec_atan() {
    check_single("atan", fmn_dmath::atan, 1);
}
#[test]
fn vec_atan2() {
    check_pair("atan2", fmn_dmath::atan2, 2);
}
#[test]
fn vec_asin() {
    check_single("asin", fmn_dmath::asin, 1);
}
#[test]
fn vec_acos() {
    check_single("acos", fmn_dmath::acos, 1);
}
#[test]
fn vec_exp() {
    check_single("exp", fmn_dmath::exp, 1);
}
#[test]
fn vec_expm1() {
    check_single("expm1", fmn_dmath::expm1, 1);
}
#[test]
fn vec_ln() {
    check_single("ln", fmn_dmath::ln, 1);
}
#[test]
fn vec_log2() {
    check_single("log2", fmn_dmath::log2, 2);
}
#[test]
fn vec_pow() {
    check_pair("pow", fmn_dmath::pow, 2);
}
#[test]
fn vec_cbrt() {
    check_single("cbrt", fmn_dmath::cbrt, 1);
}
#[test]
fn vec_sinh() {
    check_single("sinh", fmn_dmath::sinh, 2);
}
#[test]
fn vec_cosh() {
    check_single("cosh", fmn_dmath::cosh, 2);
}
#[test]
fn vec_tanh() {
    check_single("tanh", fmn_dmath::tanh, 2);
}
#[test]
fn vec_sqrt_correctly_rounded() {
    // sqrt is hardware; spot-check the 0-ulp claim over a sweep.
    for i in 0..10_000u32 {
        let x = f64::from(i) * 0.1 + 0.001;
        assert_eq!(fmn_dmath::sqrt(x).to_bits(), x.sqrt().to_bits());
    }
}
