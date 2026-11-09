//! Reference box oracle for native `Tex` layout (fm-tex-display-style-nclg,
//! fm-tex-layout-oracle-bkbc). `fixtures/tex_reference_boxes.v1.tsv` holds
//! authored tier-1 formulas with the pinned Reference's measured
//! `Tex(source)` width and height at the default font size (see its header).
//! Native boxes are compared with them; the counts of formulas inside a
//! relative tolerance on both axes may only rise. Every row is printed,
//! worst first, so each run names what to fix next.
#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic, clippy::print_stderr)]

use fmn_library::{Tex, TexEngine, TexText};

/// Formulas within ±5% of the Reference on both axes: 24 when this oracle
/// landed (2026-10-04), 26 after the display-integral fix (franken_markdown
/// 68fe29b), 39 after rule-17 italic corrections (4c8caa2), 42 after plain
/// TeX's `\ldots` and amsmath's `\pmod` (a3578a0, fm-y5fl), 43 after TeX's
/// null delimiters around fractions (e4e8e2e), 44 with TeX's accent geometry,
/// \big sizes and array struts (bd00a24), 49 with TeX's \scriptspace,
/// cmsy's prime character and LaTeX's brace fills (3ed1d33). Raise it when
/// layout improves; never lower it to land a change.
const WITHIN_5_PERCENT_FLOOR: usize = 49;
/// The same at ±10%: 57, then 60, then 65, then 67, then 70, then 73, then 75.
const WITHIN_10_PERCENT_FLOOR: usize = 75;

struct Row {
    width: f64,
    height: f64,
    source: &'static str,
}

fn rows() -> Vec<Row> {
    include_str!("fixtures/tex_reference_boxes.v1.tsv")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.splitn(3, '\t');
            let mut next = || fields.next().expect("three tab-separated fields");
            let width = next().parse().expect("reference width");
            let height = next().parse().expect("reference height");
            Row {
                width,
                height,
                source: next(),
            }
        })
        .collect()
}

/// fm-5wq.56: TeX's text fonts set the apostrophe of "can't" as the curly
/// right quote. The pinned Reference measures that glyph at 0.080 x 0.186
/// (font_size 60, the corpus differential's UniversalProblemSolvingTip
/// repro); the straight ASCII wedge it replaces measured 0.056 x 0.220.
#[test]
fn texttext_apostrophe_is_the_reference_right_quote() {
    let engine = TexEngine::new("fmd-math/pack/default", None).expect("engine");
    let text = TexText::new("can't")
        .font_size(60.0)
        .build(&engine)
        .expect("TexText typesets");
    // One child per glyph, in emission order.
    let glyphs = text.vmob.children();
    assert_eq!(glyphs.len(), 5, "c a n quote t");
    let (min, max) = glyphs[3].extent().expect("the quote has extent");
    let (width, height) = (max[0] - min[0], max[1] - min[1]);
    assert!(
        (width / 0.080 - 1.0).abs() <= 0.10 && (height / 0.186 - 1.0).abs() <= 0.10,
        "apostrophe {width:.3} x {height:.3} against the Reference's 0.080 x 0.186"
    );
}

/// fm-y5fl: `\cdots` and `\ldots` are three dots, one glyph each, as the
/// Reference's SVG sets them (they were one ellipsis glyph), and the
/// leading space of `\text{ terms}` is an interword space (it was dropped).
#[test]
fn ellipses_are_three_glyphs_and_text_boxes_keep_their_leading_space() {
    let engine = TexEngine::new("fmd-math/pack/default", None).expect("engine");
    for source in [r"\cdots", r"\ldots"] {
        let tex = Tex::new(source).build(&engine).expect(source);
        assert_eq!(tex.vmob.children().len(), 3, "{source}");
    }
    let width = |source: &str| {
        let tex = Tex::new(source).build(&engine).expect(source);
        let (min, max) = tex.vmob.extent().expect("a typeset formula has extent");
        max[0] - min[0]
    };
    let space = width(r"n\text{ terms}") - width(r"n\text{terms}");
    assert!(space > 0.05, "the leading space added {space:.3} units");
}

#[test]
fn native_tex_boxes_track_the_reference_and_never_regress() {
    let engine = TexEngine::new("fmd-math/pack/default", None).expect("engine");
    let rows = rows();
    assert!(
        rows.len() >= 100,
        "the oracle thinned out: {} rows",
        rows.len()
    );
    let mut measured = Vec::with_capacity(rows.len());
    for row in &rows {
        let error = match Tex::new(row.source).build(&engine) {
            Ok(tex) => {
                let (min, max) = tex.vmob.extent().expect("a typeset formula has extent");
                let (width, height) = (max[0] - min[0], max[1] - min[1]);
                (width / row.width - 1.0, height / row.height - 1.0)
            }
            Err(_) => (f64::INFINITY, f64::INFINITY),
        };
        measured.push((error, row.source));
    }
    let within = |tolerance: f64| {
        measured
            .iter()
            .filter(|((dw, dh), _)| dw.abs() <= tolerance && dh.abs() <= tolerance)
            .count()
    };
    let (five, ten) = (within(0.05), within(0.10));
    measured.sort_by(|a, b| {
        let key = |((dw, dh), _): &((f64, f64), &str)| dw.abs().max(dh.abs());
        key(b).total_cmp(&key(a))
    });
    eprintln!(
        "tex reference boxes: {five}/{} within 5%, {ten}/{} within 10% (floors {WITHIN_5_PERCENT_FLOOR}, {WITHIN_10_PERCENT_FLOOR})",
        rows.len(),
        rows.len()
    );
    // Every row, worst first: formula, relative width and height error.
    for ((dw, dh), source) in &measured {
        eprintln!("  dw={dw:+.3} dh={dh:+.3}  {source}");
    }
    assert!(
        five >= WITHIN_5_PERCENT_FLOOR,
        "{five} formulas within 5% of the Reference, below the floor {WITHIN_5_PERCENT_FLOOR}"
    );
    assert!(
        ten >= WITHIN_10_PERCENT_FLOOR,
        "{ten} formulas within 10% of the Reference, below the floor {WITHIN_10_PERCENT_FLOOR}"
    );
}
