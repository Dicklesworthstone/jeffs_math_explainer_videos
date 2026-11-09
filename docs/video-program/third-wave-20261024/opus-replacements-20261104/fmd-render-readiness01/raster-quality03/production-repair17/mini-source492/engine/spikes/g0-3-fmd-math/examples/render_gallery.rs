//! Writes the G0-3 visual-review gallery: one SVG per spike construct,
//! into `docs/g0/g0-3-renders/`. Reviewed against LaTeX renders of the
//! same input per the fm-nh0 acceptance (the note records verdicts).

use fmd_font::Font;
use fmn_spike_fmd_math::{Style, typeset};

fn main() {
    let cm = Font::parse(fmd_font::bundled::CM_REGULAR.to_vec()).expect("CM parses");
    let noto =
        Font::parse(fmd_font::bundled::NOTO_SANS_MATH_SYMBOLS.to_vec()).expect("Noto parses");
    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/g0/g0-3-renders");
    std::fs::create_dir_all(out_dir).expect("create render dir");
    let cases: &[(&str, &str, Style)] = &[
        (
            "frac-nested-display",
            r"\frac{1}{\frac{2}{3}}",
            Style::Display,
        ),
        ("scripts-both", r"x_i^2 + y^{a+b}", Style::Text),
        ("radical-index", r"\sqrt[3]{x} + \sqrt{a+b}", Style::Text),
        ("leftright-small", r"\left(x\right)", Style::Text),
        ("leftright-medium", r"\left(\frac{a}{b}\right)", Style::Text),
        (
            "leftright-large-drawn",
            r"\left(\frac{\frac{a}{b}}{\frac{c}{d}}\right)",
            Style::Display,
        ),
        ("sum-display-limits", r"\sum_i^n x", Style::Display),
        ("sum-text-side", r"\sum_i^n x", Style::Text),
        (
            "matrix-2x2",
            r"\begin{matrix} a & b \\ c & d \end{matrix}",
            Style::Text,
        ),
        (
            "kitchen-sink",
            r"\frac{a+b}{c} = \sqrt{x_i^2} - \left(\frac{1}{2}\right)",
            Style::Display,
        ),
    ];
    for (name, src, style) in cases {
        let layout = typeset(src, *style)
            .unwrap_or_else(|e| panic!("gallery case {name} failed to typeset: {e}"));
        let svg = layout.to_svg(&cm, &noto);
        let path = format!("{out_dir}/{name}.svg");
        std::fs::write(&path, svg).expect("write svg");
        println!("{path}");
    }
}
