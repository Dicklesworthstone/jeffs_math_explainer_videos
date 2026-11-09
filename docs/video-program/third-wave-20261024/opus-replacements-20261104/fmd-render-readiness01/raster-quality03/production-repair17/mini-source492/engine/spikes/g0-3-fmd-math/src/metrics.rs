//! Math-metrics synthesis for the bundled faces — the (b) question the
//! spike answers.
//!
//! The method under proof: take the **published TFM fontdimen family**
//! (cmr10's σ1–7, cmsy10's σ5–22, cmex10's ξ8–13 — the exact parameters
//! TeX's Appendix G consumes) as compiled-in calibration constants in
//! units of 1/1000 em, and *validate* them against geometry decoded from
//! the bundled CM Unicode faces by fmd-font. The spike's tests hold the
//! measurable pairs together (x-height vs σ5, axis height vs the
//! '+'/'=' glyph centers, quad vs the em) — if the bundled faces ever
//! drift from the Computer Modern the parameters describe, the
//! validation fails rather than the layout silently degrading.
//!
//! All engine mathematics is in **ems** (f64); font design units convert
//! at the boundary via each face's `units_per_em`.

use fmd_font::Font;

/// The Appendix-G parameter family, in ems (published TFM values / 1000).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathMetrics {
    /// σ5 — x-height.
    pub x_height: f64,
    /// σ6 — quad (1 em).
    pub quad: f64,
    /// σ8 — numerator shift-up in display style.
    pub num1: f64,
    /// σ9 — numerator shift-up in text/script styles.
    pub num2: f64,
    /// σ11 — denominator shift-down in display style.
    pub denom1: f64,
    /// σ12 — denominator shift-down in text/script styles.
    pub denom2: f64,
    /// σ13 — superscript shift-up in display style.
    pub sup1: f64,
    /// σ14 — superscript shift-up in text style (uncramped).
    pub sup2: f64,
    /// σ15 — superscript shift-up when cramped.
    pub sup3: f64,
    /// σ16 — subscript shift-down without a superscript.
    pub sub1: f64,
    /// σ17 — subscript shift-down with a superscript.
    pub sub2: f64,
    /// σ18 — superscript drop for boxy bases.
    pub sup_drop: f64,
    /// σ19 — subscript drop for boxy bases.
    pub sub_drop: f64,
    /// σ20 — minimum delimiter size in display style.
    pub delim1: f64,
    /// σ21 — minimum delimiter size in text style.
    pub delim2: f64,
    /// σ22 — axis height above the baseline.
    pub axis_height: f64,
    /// ξ8 — default rule thickness.
    pub rule_thickness: f64,
    /// ξ9 — big-op limit gap above (minimum).
    pub big_op_spacing1: f64,
    /// ξ10 — big-op limit gap below (minimum).
    pub big_op_spacing2: f64,
    /// ξ11 — big-op limit clearance above.
    pub big_op_spacing3: f64,
    /// ξ12 — big-op limit clearance below.
    pub big_op_spacing4: f64,
    /// ξ13 — padding above/below big-op limits.
    pub big_op_spacing5: f64,
}

/// The published cmr10/cmsy10/cmex10 values — the calibration source.
pub const CM: MathMetrics = MathMetrics {
    x_height: 0.4306,
    quad: 1.0,
    num1: 0.6765,
    num2: 0.3937,
    denom1: 0.6859,
    denom2: 0.3448,
    sup1: 0.4129,
    sup2: 0.3629,
    sup3: 0.2889,
    sub1: 0.1500,
    sub2: 0.2472,
    sup_drop: 0.3861,
    sub_drop: 0.0500,
    delim1: 2.3900,
    delim2: 1.0100,
    axis_height: 0.2500,
    rule_thickness: 0.0400,
    big_op_spacing1: 0.1111,
    big_op_spacing2: 0.1667,
    big_op_spacing3: 0.2000,
    big_op_spacing4: 0.6000,
    big_op_spacing5: 0.1000,
};

/// A measured validation of the compiled parameters against a decoded CM
/// face. Returns `(measured x-height, measured axis height)` in ems.
///
/// The synthesis method's go/no-go: these must sit within tight relative
/// tolerance of σ5/σ22 (the spike's tests assert 1 %) — proof that the
/// TFM family describes the bundled sfnt faces.
#[must_use]
pub fn measure_cm(font: &Font) -> (f64, f64) {
    let upm = f64::from(font.units_per_em);
    let x_height = font
        .glyph_outline(font.glyph_index('x'))
        .ok()
        .and_then(|o| o.bbox)
        .map_or(0.0, |b| f64::from(b[3]) / upm);
    let axis = font
        .glyph_outline(font.glyph_index('+'))
        .ok()
        .and_then(|o| o.bbox)
        .map_or(0.0, |b| (f64::from(b[1]) + f64::from(b[3])) / 2.0 / upm);
    (x_height, axis)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn published_parameters_describe_the_bundled_cm_face() {
        let font = Font::parse(fmd_font::bundled::CM_REGULAR.to_vec()).expect("CM parses");
        let (x_height, axis) = measure_cm(&font);
        let x_rel = (x_height - CM.x_height).abs() / CM.x_height;
        let a_rel = (axis - CM.axis_height).abs() / CM.axis_height;
        assert!(
            x_rel < 0.01,
            "measured x-height {x_height} vs σ5 {}: {x_rel:.4} relative",
            CM.x_height
        );
        assert!(
            a_rel < 0.01,
            "measured axis {axis} vs σ22 {}: {a_rel:.4} relative",
            CM.axis_height
        );
    }

    #[test]
    fn equals_sign_centers_on_the_axis() {
        // Independent axis witness: '=' must straddle σ22.
        let font = Font::parse(fmd_font::bundled::CM_REGULAR.to_vec()).expect("CM parses");
        let upm = f64::from(font.units_per_em);
        let b = font
            .glyph_outline(font.glyph_index('='))
            .unwrap()
            .bbox
            .unwrap();
        let center = (f64::from(b[1]) + f64::from(b[3])) / 2.0 / upm;
        assert!(
            (center - CM.axis_height).abs() < 0.005,
            "'=' centers at {center}, axis is {}",
            CM.axis_height
        );
    }
}
