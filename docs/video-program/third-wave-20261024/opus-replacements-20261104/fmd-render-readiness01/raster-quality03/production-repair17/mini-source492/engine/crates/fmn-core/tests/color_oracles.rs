//! Round-trip and algebraic oracles for the §6.3 color pipeline (BN-04).

use fmn_core::color::{
    LinearRgba, PremulRgba, Srgb, color_gradient_by_hsl, interpolate_color,
    interpolate_color_by_hsl, interpolate_color_oklab, oklab_to_srgb, srgb_eotf, srgb_oetf,
    srgb_to_oklab,
};

#[test]
fn transfer_functions_invert_each_other() {
    // decode ∘ encode = id on a dense sweep of linear values, and
    // encode ∘ decode = id on every 8-bit code point.
    for i in 0..=10_000 {
        let l = f64::from(i) / 10_000.0;
        assert!((srgb_eotf(srgb_oetf(l)) - l).abs() < 1e-12, "linear {l}");
    }
    for v in 0..=255u8 {
        let enc = f64::from(v) / 255.0;
        assert!((srgb_oetf(srgb_eotf(enc)) - enc).abs() < 1e-12, "code {v}");
    }
}

#[test]
fn eight_bit_round_trip_is_exact() {
    // Encoded → decoded → re-encoded → quantized must return every 8-bit
    // triple unchanged: the pipeline may not shift stored colors.
    for v in 0..=255u8 {
        let c = Srgb::from_rgb8(v, v, v);
        let round = c.to_linear(1.0).to_srgb().to_rgb8();
        assert_eq!(round, [v, v, v]);
    }
}

#[test]
fn premultiply_laws_hold() {
    let c = LinearRgba {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 0.4,
    };
    // premultiply / unpremultiply round-trip
    let back = c.premultiply().unpremultiply();
    assert!((back.r - c.r).abs() < 1e-12);
    assert!((back.g - c.g).abs() < 1e-12);
    assert!((back.b - c.b).abs() < 1e-12);
    assert_eq!(back.a, c.a);

    // Opaque source-over replaces the destination entirely.
    let opaque = LinearRgba {
        r: 0.9,
        g: 0.1,
        b: 0.3,
        a: 1.0,
    }
    .premultiply();
    let dst = LinearRgba {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 0.8,
    }
    .premultiply();
    assert_eq!(opaque.over(dst), opaque);

    // Transparent source-over is the identity on the destination.
    assert_eq!(PremulRgba::TRANSPARENT.over(dst), dst);

    // Alpha composits like color: a_out = a_src + (1 - a_src) * a_dst.
    let src = LinearRgba {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 0.6,
    }
    .premultiply();
    let out = src.over(dst);
    assert!((out.a - (0.6 + 0.4 * 0.8)).abs() < 1e-12);

    // Source-over is associative (on a sample triple).
    let a = LinearRgba {
        r: 0.1,
        g: 0.9,
        b: 0.4,
        a: 0.3,
    }
    .premultiply();
    let b = LinearRgba {
        r: 0.7,
        g: 0.2,
        b: 0.6,
        a: 0.5,
    }
    .premultiply();
    let c3 = LinearRgba {
        r: 0.3,
        g: 0.3,
        b: 0.9,
        a: 0.7,
    }
    .premultiply();
    let left = a.over(b).over(c3);
    let right = a.over(b.over(c3));
    for (l, r) in [
        (left.r, right.r),
        (left.g, right.g),
        (left.b, right.b),
        (left.a, right.a),
    ] {
        assert!((l - r).abs() < 1e-12);
    }
}

#[test]
fn interpolate_color_hits_its_endpoints() {
    let (c1, c2) = (fmn_core::constants::BLUE_E, fmn_core::constants::YELLOW_C);
    assert_eq!(interpolate_color(c1, c2, 0.0), c1);
    assert_eq!(interpolate_color(c1, c2, 1.0), c2);
    assert_eq!(interpolate_color_oklab(c1, c2, 0.0).to_rgb8(), c1.to_rgb8());
    assert_eq!(interpolate_color_oklab(c1, c2, 1.0).to_rgb8(), c2.to_rgb8());
}

#[test]
fn reference_hsl_interpolation_uses_the_long_linear_hue_coordinate() {
    let red = Srgb::from_hex("#FF0000").unwrap();
    let blue = Srgb::from_hex("#0000FF").unwrap();

    // `colour.Color` reports red and blue hues as 0 and 2/3. The Reference
    // linearly interpolates those scalars, so the midpoint is green rather
    // than the shortest-arc magenta that another HSL policy might choose.
    let midpoint = interpolate_color_by_hsl(red, blue, 0.5);
    assert!((midpoint.r - 0.0).abs() < 1e-12);
    assert!((midpoint.g - 1.0).abs() < 1e-12);
    assert!((midpoint.b - 0.0).abs() < 1e-12);

    let gradient = color_gradient_by_hsl(&[red, blue], 3);
    assert_eq!(gradient.len(), 3);
    assert_eq!(gradient[0], red);
    assert_eq!(gradient[2], blue);
    assert_eq!(gradient[1], midpoint);

    // `colour` 0.1.5 deliberately collapses sub-5e-7 channel ranges to an
    // achromatic HSL value; this is one of its own published regressions.
    let almost_white = Srgb {
        r: 0.999_999_999_999_999_9,
        g: 1.0,
        b: 0.999_999_999_999_999_4,
    };
    let collapsed = interpolate_color_by_hsl(almost_white, red, 0.0);
    assert_eq!(collapsed.r, collapsed.g);
    assert_eq!(collapsed.g, collapsed.b);
}

#[test]
fn oklab_round_trips_the_palette() {
    for c in [
        fmn_core::constants::BLUE_E,
        fmn_core::constants::RED_C,
        fmn_core::constants::GREEN_A,
        fmn_core::constants::WHITE,
        fmn_core::constants::BLACK,
        fmn_core::constants::ORANGE,
    ] {
        let round = oklab_to_srgb(srgb_to_oklab(c));
        assert!((round.r - c.r).abs() < 1e-6, "{c:?} → {round:?}");
        assert!((round.g - c.g).abs() < 1e-6);
        assert!((round.b - c.b).abs() < 1e-6);
    }
}
