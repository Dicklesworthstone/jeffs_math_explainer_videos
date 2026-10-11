//! Spike S-7 (plan §6.5): a glyph probe of every planned on-screen string of
//! Videos A, B and C at the workspace's franken_manim pin, through the kit's
//! own `glyphs` command (one sheet per video; any string that fails to
//! typeset is listed and the run exits 1). Read every page: a string can
//! typeset and still be wrong.
//!
//! ```text
//! cargo run --release -p explainer_kit --example glyph_probe_s7 -- OUT_DIR
//! ```
//!
//! The text strings are the quoted and bold on-screen strings of the
//! storyboards' picture columns (plan §9.4, §10.4, §11.4); the TeX strings are
//! each video's formulas as they will be typeset, the code in typewriter, and
//! the notation table's risks (§6.5).

use explainer_kit::Facts;
use explainer_kit::cli::{self, Chapter, Explainer};
use explainer_score::score::Identity;

const A_TEXT: &[&str] = &[
    "100 | 900 / 900 | ?",
    "≈ 8,100",
    "How to Count Fish You've Never Seen",
    "real answer: ?",
    "This example was used in the first edition without knowledge that the method is widely used in practice.",
    "one tenth of ?",
    "only we can see this",
    "how many fish",
    "what fraction is marked",
    "independent catches",
    "twenty choose four",
    "20·19·18·17 / 24 = 4,845",
    "lake → catch",
    "≈ 33",
    "score",
    "It is, therefore, meaningless to ask for the probability that n is greater than, say, 6000.",
    "lake size",
    "marked fish caught",
    "the cancelling is in the description",
    "Mn − kN = ab − k·u",
    "confidence interval",
];

const A_TEX: &[&str] = &[
    r"\binom{M}{k}",
    r"\binom{20}{4} = \frac{20\cdot 19\cdot 18\cdot 17}{24} = 4{,}845",
    r"\binom{5}{2}\binom{15}{2} = 10\cdot 105 = 1{,}050",
    r"P_N(K=k) = \frac{\binom{M}{k}\binom{N-M}{n-k}}{\binom{N}{n}}",
    r"\frac{L(N)}{L(N-1)} = \frac{(N-M)(N-n)}{N(N-M-n+k)}",
    r"Mn - kN = ab - k\cdot u",
    r"\hat N = \frac{Mn}{k} \qquad \hat N = \frac{(M+1)(n+1)}{k+1} - 1",
    r"\approx 8{,}100",
];

const B_TEXT: &[&str] = &[
    "3 × 7 = 9",
    "The Arithmetic Where 3 × 7 = 9",
    "no return channel",
    "A? B? C? D?",
    "useful cards",
    "packet #48213 → pieces {…}",
    "nothing",
    "The danger meter",
    "field",
    "change",
    "pieces",
    "packets",
    "1 of 5 runs collapsed",
    "three times five is fifteen here too, but three times seven was nine",
    "3A = 15 ⇒ A = 5",
    "x⁸ = x⁴+x³+x²+1",
    "1 in 100 / 1 in 10,000 / 1 in a million",
    "Reed–Solomon",
];

const B_TEX: &[&str] = &[
    r"3 \times 7 = 9",
    r"2\cdot 128 \equiv 0 \qquad x + x = 0 \qquad 3\cdot 255 = 0",
    r"x^8 = x^4 + x^3 + x^2 + 1",
    r"n \le q + k - 1",
    r"3A = 15 \Rightarrow A = 5",
    r"\mathsf{\alpha} \qquad \text{α} \qquad \alpha",
    // \alpha in math renders as an italic Latin a: the two safe forms
    r"\alpha^8 = \alpha^4 + \alpha^3 + \alpha^2 + 1",
    r"\text{α}^8 = \text{α}^4 + \text{α}^3 + \text{α}^2 + 1",
    r"\mathsf{\alpha}^8 = \mathsf{\alpha}^4 + \mathsf{\alpha}^3 + \mathsf{\alpha}^2 + 1",
    r"A = 2 \oplus c \qquad B = 6 \oplus 2 \oplus c",
    r"\Pr[\text{fail}] < 256^{-h}/255 \qquad \prod_{i\ge 1}\left(1 - 2^{-i}\right)",
    r"\frac{K^3}{3} \approx 4.2\times 10^{13}",
    r"\text{Reed–Solomon}",
];

const C_TEXT: &[&str] = &[
    "The Algorithm Lamport Is Proudest Of",
    "time runs left to right; a read is an arrow in, a write is an arrow out",
    "Progress square",
    "both inside",
    "for programmers",
    "N = 4, tickets ≤ 4: 3,027,633 states, no violation",
    "every N",
    "safe",
    "value unused",
    "outboxes",
];

const C_TEX: &[&str] = &[
    r"(n_j, j) \prec (n_i, i)",
    r"n_i = 1 + \max(n_1, \dots, n_N)",
    r"\lfloor x \rfloor",
];

// code and hex: fmn's TeX has no \texttt, so they are Text in CM Typewriter
const A_MONO: &[&str] = &["k = rng.hypergeometric(M, N-M, n, size=10_000)", "est = M*n/k"];
const B_MONO: &[&str] = &[
    "0x1D   0x57 · 0x83 = 0x31",
    "fn xtime(a: u8) -> u8 { (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 } }",
    "acc ^= a;  b >>= 1;  // mul(2, 142) == 1",
];
const C_MONO: &[&str] = &[
    "volatile int[]   AtomicIntegerArray",
    "number[i] = 1 + max(number);",
    "while (choosing[j]) {}",
    "while (number[j] != 0 && (number[j], j) < (number[i], i)) {}",
];

fn explainer(
    name: &'static str,
    text: &'static [&'static str],
    tex: &'static [&'static str],
    mono: &'static [&'static str],
) -> Explainer {
    Explainer {
        crate_name: name,
        script: "",
        chapters: Vec::<Chapter>::new,
        facts: || Ok(Facts::new()),
        glyph_text: text,
        glyph_tex: tex,
        glyph_mono: mono,
        identity: Identity::hoeffding,
        narration_gain_db: 0.0,
        line_owner: None,
        thumbnail: None,
    }
}

fn main() {
    let out = std::env::args().nth(1).expect("usage: glyph_probe_s7 OUT_DIR");
    let mut failed = 0;
    for (name, text, tex, mono) in [
        ("video_a", A_TEXT, A_TEX, A_MONO),
        ("video_b", B_TEXT, B_TEX, B_MONO),
        ("video_c", C_TEXT, C_TEX, C_MONO),
    ] {
        let dir = format!("{out}/{name}");
        println!("== {name}: {} text, {} tex, {} mono", text.len(), tex.len(), mono.len());
        let code = cli::run(&explainer(name, text, tex, mono), vec!["glyphs".into(), dir]);
        failed += i32::from(code != 0);
    }
    std::process::exit(failed);
}
