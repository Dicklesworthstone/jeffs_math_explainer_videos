//! G0-3 — the fmd-math architecture spike (fm-nh0, plan §20.1 spike 3 → §11.4).
//!
//! A parse-and-layout proof over the constructs that force every
//! load-bearing architectural question: nested `\frac`, simultaneous
//! sub+superscripts, radicals with indices, `\left(…\right)` at three
//! sizes, `\sum` with limits in display and text style, and a small
//! matrix. The engine shape proven here — token stream → node tree →
//! atom-classified horizontal lists with the TeX inter-atom spacing
//! table → Appendix-G constructions over synthesized Computer Modern
//! metrics → positioned glyphs + rules with source-span provenance —
//! is the shape fmd-math freezes at (R8) until G2.
//!
//! Three architectural verdicts this spike records (see
//! `docs/g0/G0-3-fmd-math-ratification.md`):
//!
//! 1. **Metrics synthesis (go):** the published TFM fontdimen family
//!    (cmr10/cmsy10/cmex10 — the Appendix-G σ/ξ parameters) transfers to
//!    the bundled CM Unicode faces essentially unchanged: measured
//!    x-height is within 0.15 % of σ5 and the '+'/'=' glyphs center on
//!    σ22's axis exactly. Parameters are compiled in as calibration
//!    constants and *validated* against decoded glyph geometry in tests.
//! 2. **Extensible delimiters (OQ-2):** CM Unicode ships **no** bracket
//!    extension pieces (U+239B…U+23AE unmapped) and no size-variant sets,
//!    so glyph assembly cannot be the mechanism for the bundled set.
//!    Ruling: natural-size glyphs up to a scale threshold, **drawn-path
//!    construction as the mainline beyond it** — not merely a fallback.
//! 3. **Multi-face layout is structural:** the big operators (∑ ∫ ∏) do
//!    not exist in CM Unicode at all; they resolve through the Noto Math
//!    fallback face. Per-glyph face selection is in the core data model
//!    (every positioned glyph names its face), not an afterthought.

pub mod layout;
pub mod metrics;
pub mod output;
pub mod parse;

pub use layout::{Style, typeset};
pub use metrics::MathMetrics;
pub use output::{Face, Layout, PlacedGlyph, PlacedRule};
pub use parse::{MathError, Node, parse};
