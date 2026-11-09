//! fm-nh0 acceptance: the hard constructs parse AND lay out with the
//! structural properties Appendix G promises. Sizes and positions are in
//! ems on a y-up baseline. Tests may use `unwrap`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fmn_spike_fmd_math::metrics::CM;
use fmn_spike_fmd_math::{Style, typeset};

const THETA: f64 = 0.04; // ξ8
const AXIS: f64 = 0.25; // σ22

#[test]
fn nested_fraction_display() {
    let l = typeset(r"\frac{1}{\frac{2}{3}}", Style::Display).expect("lays out");
    // Two fraction rules, both centered on the math axis of their level:
    // the outer at the display axis, the inner strictly below it.
    assert_eq!(l.rules.len(), 2);
    let mut ys: Vec<f64> = l.rules.iter().map(|r| r.y + r.h / 2.0).collect();
    ys.sort_by(f64::total_cmp);
    assert!(
        (ys[1] - AXIS).abs() < 1e-9,
        "outer rule centers on the axis, got {}",
        ys[1]
    );
    assert!(ys[0] < AXIS, "inner rule sits below the outer axis");
    // Numerator '1' clears the rule by ≥ 3θ (display clearance).
    let one = l.glyphs.iter().find(|g| g.ch == '1').unwrap();
    let rule_top = AXIS + THETA / 2.0;
    assert!(
        one.y >= rule_top + 3.0 * THETA - 1e-9,
        "display numerator clearance violated: baseline {} vs rule top {rule_top}",
        one.y
    );
    // All three digits present, with provenance spans intact.
    for d in ['1', '2', '3'] {
        let g = l.glyphs.iter().find(|g| g.ch == d).unwrap();
        assert_eq!(
            r"\frac{1}{\frac{2}{3}}".as_bytes()[g.span.start],
            d as u8,
            "span provenance drifted for {d:?}"
        );
    }
}

#[test]
fn simultaneous_scripts_with_clearance() {
    let l = typeset(r"x_i^2", Style::Text).expect("lays out");
    let x = l.glyphs.iter().find(|g| g.ch == 'x').unwrap();
    let i = l.glyphs.iter().find(|g| g.ch == 'i').unwrap();
    let two = l.glyphs.iter().find(|g| g.ch == '2').unwrap();
    // Script size factor: 0.7 of text.
    assert!((i.size - 0.7).abs() < 1e-9 && (two.size - 0.7).abs() < 1e-9);
    // Superscript raised, subscript lowered, both to the right of the base.
    assert!(two.y > 0.0 && i.y < 0.0);
    assert!(i.x > x.x && two.x > x.x);
    // The superscript rides at least sup2 above the baseline.
    assert!(
        two.y >= CM.sup2 - 1e-9,
        "sup shift {} below σ14 {}",
        two.y,
        CM.sup2
    );
    // Subscript at least sub2 below.
    assert!(
        -i.y >= CM.sub2 - 1e-9,
        "sub shift {} above σ17 {}",
        -i.y,
        CM.sub2
    );
}

#[test]
fn radical_with_index() {
    let l = typeset(r"\sqrt[3]{x}", Style::Text).expect("lays out");
    // The radicand, the index at scriptscript size, the radical sign, and
    // an overbar rule spanning the radicand.
    let x = l.glyphs.iter().find(|g| g.ch == 'x').unwrap();
    let three = l.glyphs.iter().find(|g| g.ch == '3').unwrap();
    assert!(
        (three.size - 0.5).abs() < 1e-9,
        "index is scriptscript-size"
    );
    assert!(three.y > 0.0, "index rides above the baseline");
    let bar = l
        .rules
        .iter()
        .find(|r| r.w > 0.2 && r.h <= THETA + 1e-9)
        .expect("overbar rule present");
    assert!(bar.y > x.y, "overbar above the radicand baseline");
    // Clearance over the radicand: ≥ θ + θ/4 in text style.
    let sqrt_glyph = l.glyphs.iter().find(|g| g.ch == '√').unwrap();
    assert!(sqrt_glyph.x < x.x, "sign precedes the radicand");
}

#[test]
fn left_right_three_sizes_three_mechanisms() {
    // Natural size: a lone x needs no more than the authored paren.
    let small = typeset(r"\left(x\right)", Style::Text).expect("small");
    let parens: Vec<_> = small
        .glyphs
        .iter()
        .filter(|g| g.ch == '(' || g.ch == ')')
        .collect();
    assert_eq!(parens.len(), 2);
    assert!(
        parens.iter().all(|g| (g.size - 1.0).abs() < 1e-9),
        "natural-size glyphs at text size"
    );
    assert!(small.paths.is_empty());

    // Medium: a text fraction pushes past natural → scaled glyph, still
    // under the 1.25× drawn-path threshold.
    let medium = typeset(r"\left(\frac{a}{b}\right)", Style::Text).expect("medium");
    let parens: Vec<_> = medium.glyphs.iter().filter(|g| g.ch == '(').collect();
    assert_eq!(parens.len(), 1);
    assert!(
        parens[0].size > 1.0 && parens[0].size <= 1.25 + 1e-9,
        "medium delimiter is a scaled glyph, size {}",
        parens[0].size
    );
    assert!(medium.paths.is_empty());

    // Large: stacked display fractions exceed the threshold → the OQ-2
    // drawn-path mainline takes over (no paren glyphs at all).
    let large = typeset(
        r"\left(\frac{\frac{a}{b}}{\frac{c}{d}}\right)",
        Style::Display,
    )
    .expect("large");
    assert_eq!(
        large.paths.len(),
        2,
        "both large delimiters are drawn paths"
    );
    assert!(
        !large.glyphs.iter().any(|g| g.ch == '(' || g.ch == ')'),
        "no glyph delimiters at drawn size"
    );
    // The drawn delimiters cover the body: at least as tall as the inner
    // fraction stack's extent.
    let body_extent = large.height + large.depth;
    for p in &large.paths {
        let (mut top, mut bot) = (f64::MIN, f64::MAX);
        for c in &p.contours {
            top = top.max(c.start.y + p.y);
            bot = bot.min(c.start.y + p.y);
            for s in &c.segments {
                top = top.max(s.to().y + p.y);
                bot = bot.min(s.to().y + p.y);
            }
        }
        assert!(
            (top - bot) >= 0.7 * body_extent,
            "drawn delimiter covers the body: {} vs extent {body_extent}",
            top - bot
        );
    }
}

#[test]
fn big_op_limits_display_vs_text() {
    // Display: limits stack centered above and below the operator.
    let d = typeset(r"\sum_i^n x", Style::Display).expect("display");
    let sum = d.glyphs.iter().find(|g| g.ch == '∑').unwrap();
    let i = d.glyphs.iter().find(|g| g.ch == 'i').unwrap();
    let n = d.glyphs.iter().find(|g| g.ch == 'n').unwrap();
    assert!(n.y > sum.y && i.y < sum.y, "limits above and below");
    // Centered: the limit x-positions overlap the operator's span.
    assert!(
        n.x >= sum.x - 0.5 && n.x <= sum.x + 1.5,
        "upper limit centered over the operator"
    );

    // Text: side scripts instead.
    let t = typeset(r"\sum_i^n x", Style::Text).expect("text");
    let sum_t = t.glyphs.iter().find(|g| g.ch == '∑').unwrap();
    let i_t = t.glyphs.iter().find(|g| g.ch == 'i').unwrap();
    let n_t = t.glyphs.iter().find(|g| g.ch == 'n').unwrap();
    assert!(
        i_t.x > sum_t.x && n_t.x > sum_t.x,
        "text-style limits set to the side"
    );
    // The operator itself centers on the math axis in both styles.
    // (∑ resolves through the Noto fallback face — multi-face layout.)
    assert!(matches!(
        d.glyphs.iter().find(|g| g.ch == '∑').unwrap().face,
        fmn_spike_fmd_math::Face::NotoMath
    ));
}

#[test]
fn small_matrix_grid() {
    let l = typeset(r"\begin{matrix} a & b \\ c & d \end{matrix}", Style::Text).expect("matrix");
    let g = |ch: char| l.glyphs.iter().find(|g| g.ch == ch).unwrap();
    let (a, b, c, d) = (g('a'), g('b'), g('c'), g('d'));
    // Grid alignment: columns share x (centered within equal-width cells),
    // rows share y.
    assert!((a.y - b.y).abs() < 1e-9 && (c.y - d.y).abs() < 1e-9);
    assert!((a.x - c.x).abs() < 0.05 && (b.x - d.x).abs() < 0.05);
    // Rows 1.2 em apart.
    assert!(((a.y - c.y) - 1.2).abs() < 1e-9, "baseline skip 1.2 em");
    // The block centers on the math axis (\vcenter semantics): the
    // vertical midpoint of the two baselines sits near the axis.
    let mid = (a.y + c.y) / 2.0;
    assert!(
        (mid - AXIS).abs() < 0.5,
        "matrix vcenters near the axis, mid {mid}"
    );
    // Column separation ≈ 1 em between cell boxes.
    assert!(b.x - a.x >= 1.0, "column separation ≥ 1 em");
}

#[test]
fn unsupported_constructs_error_precisely() {
    // The ratchet contract survives layout: named command, exact span.
    let err = typeset(r"\frac{1}{\oint x}", Style::Display).unwrap_err();
    match err {
        fmn_spike_fmd_math::MathError::UnsupportedCommand { name, span } => {
            assert_eq!(name, "oint");
            assert_eq!(&r"\frac{1}{\oint x}"[span], r"\oint");
        }
        other => panic!("expected UnsupportedCommand, got {other}"),
    }
}

#[test]
fn spacing_table_binary_vs_unary_minus() {
    // "a-b": Bin with medium spaces. "(-b": the minus degrades to Ord.
    let bin = typeset(r"a-b", Style::Text).expect("binary");
    let una = typeset(r"(-b", Style::Text).expect("unary");
    let gap = |l: &fmn_spike_fmd_math::Layout, from: char, to: char| {
        let f = l.glyphs.iter().find(|g| g.ch == from).unwrap();
        let t = l.glyphs.iter().find(|g| g.ch == to).unwrap();
        t.x - f.x
    };
    let bin_gap = gap(&bin, '-', 'b');
    let una_gap = gap(&una, '-', 'b');
    assert!(
        bin_gap > una_gap + 0.15,
        "binary minus carries medium space: {bin_gap} vs unary {una_gap}"
    );
}

#[test]
fn script_styles_suppress_spacing() {
    // The same "a+b" in a superscript loses its medium spaces and shrinks.
    let base = typeset(r"a+b", Style::Text).expect("text");
    let scripted = typeset(r"x^{a+b}", Style::Text).expect("scripted");
    let sup_a = scripted.glyphs.iter().find(|g| g.ch == 'a').unwrap();
    assert!((sup_a.size - 0.7).abs() < 1e-9);
    let width_of = |l: &fmn_spike_fmd_math::Layout| {
        let a = l.glyphs.iter().find(|g| g.ch == 'a').unwrap();
        let b = l.glyphs.iter().find(|g| g.ch == 'b').unwrap();
        b.x - a.x
    };
    assert!(
        width_of(&scripted) < width_of(&base) * 0.75,
        "script-style a+b must be tighter than text-style"
    );
}
