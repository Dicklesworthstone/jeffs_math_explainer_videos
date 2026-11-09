//! The transcendental funnel: every trig/inverse-trig call in fmn-geom
//! routes through here, and here delegates to **fmn-dmath** (§6.6, D-17) —
//! the owned deterministic elementary-function layer — so geometry is
//! bit-identical on every certified target by construction. `sqrt` is used
//! directly at call sites: IEEE 754 requires correct rounding, so
//! `f64::sqrt` is already bit-reproducible everywhere.
//!
//! Geometry construction is object-space semantics, part of the input to
//! the certified pipeline, so it always uses the certified layer — the
//! standard/fast seam (`fmn_dmath::FAST`) is a renderer-side choice and
//! deliberately not plumbed here.

#[inline]
pub(crate) fn sin(x: f64) -> f64 {
    fmn_dmath::sin(x)
}

#[inline]
pub(crate) fn cos(x: f64) -> f64 {
    fmn_dmath::cos(x)
}

#[inline]
pub(crate) fn tan(x: f64) -> f64 {
    fmn_dmath::tan(x)
}

#[inline]
pub(crate) fn acos(x: f64) -> f64 {
    fmn_dmath::acos(x)
}

#[inline]
pub(crate) fn atan2(y: f64, x: f64) -> f64 {
    fmn_dmath::atan2(y, x)
}

#[inline]
pub(crate) fn ln(x: f64) -> f64 {
    fmn_dmath::ln(x)
}

#[inline]
pub(crate) fn cbrt(x: f64) -> f64 {
    fmn_dmath::cbrt(x)
}
