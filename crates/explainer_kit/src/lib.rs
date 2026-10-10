//! `explainer_kit`: the scene kit every explainer in the corpus shares.
//!
//! Extracted from the math-explainer skill's template crate (plan §3.3, D-03)
//! module by module. What is here so far: palette roles, number formatting,
//! vector helpers and the `Frame2` data window. The rest of the template's kit
//! (text/Tex helpers, live readouts, play macros, layout lint, narration
//! timeline, encoder wrapper, stats kit, CLI) lands through beads
//! W1-kit-core … W1-kit-cli.

pub mod palette;
pub mod plot;

pub use palette::{BACKGROUND, Palette, hex, ink_style, point_colors, stroke_style};
pub use plot::{Frame2, add, lerp, thousands, v};
