//! Typesetting probe lists: every Text and Tex string the video intends to
//! use. The shared CLI's `glyphs` command lays them out on PNG pages and
//! fails on any string that does not typeset; a string can typeset and still
//! be wrong (math ρ/θ draw as ϱ/ϑ), so look at every page. (The v5 probe
//! scene is in `v5_legacy/glyphs.rs`.)

pub const TEXT: &[&str] = &[
    "Pearson  Spearman  Kendall  Hoeffding's D",
    "ρ τ α θ ≈ × − → · • … ≤ ≥ “quotes”",
    "✓ ✗",
    "IBM Plex Sans: 0123456789 ρ τ ≈ × − →",
];

pub const TEX: &[&str] = &[
    r"\rho \quad \tau \quad \alpha \quad \theta \quad \varrho \quad \vartheta",
    r"r_s \quad \hat\rho \quad \mathrm{\rho}",
    r"Q_i = 1 + \#\{\, j : R_j < R_i,\ S_j < S_i \,\}",
    r"\binom{N}{4} = \frac{N(N-1)(N-2)(N-3)}{24} \approx 2.6\times 10^{13}",
    r"D_1 = \sum_i (Q_i-1)(Q_i-2) \qquad -0.5 \le D \le 1",
    r"\text{Pearson } r \qquad \text{Spearman } \rho \qquad \text{Kendall } \tau",
];
