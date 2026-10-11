//! `explainer_kit`: the scene kit every explainer in the corpus shares.
//!
//! Extracted from the math-explainer skill's template crate (plan §3.3, D-03)
//! module by module: palette roles; the scene kit (text and math built,
//! placed and registered in one call; live readouts); playback (`play!`,
//! `Timed`, tweens, `fade_all`); plots (`Frame2`, rectangles, dot clouds,
//! number formatting); narration (script.tsv, the Narrator, the
//! exact cue log and the score engine's timeline); the layout lint; the
//! NumPy-parity stats kit and the facts harness; the delivery encoder
//! (quality runner, raised caps, camera route, soundtrack capture); and the
//! shared CLI (`cli::run`) with the two-pass score (`render --timeline-only`,
//! `score`, `render --score`) and the PCM master (`master`: cues replayed
//! through fmn's own mixer, null-tested, joined without re-encoding).

pub mod animatic;
pub mod captions;
pub mod cli;
pub mod encode;
pub mod facts;
pub mod kit;
pub mod layout;
pub mod master;
pub mod narration;
pub mod palette;
pub mod play;
pub mod plot;
pub mod stats;
pub mod storyboard;

pub use cli::{Chapter, Explainer};
pub use facts::{Fact, Facts, choose, oracle_diff};
pub use kit::{DRAFT_SONIFY_DB, GROUP_TRANSFORM_LIMIT, Kit, Registered, TextKind, live_bar, sound_file};
pub use layout::Linter;
pub use narration::{Cue, CueFile, Narrator, Script, inspect_cue};
pub use palette::{BACKGROUND, Palette, hex, ink_style, point_colors, stroke_style};
pub use play::{Timed, fade_all, lagged, tween};
pub use plot::{Frame2, add, children, dot_cloud, lerp, rect, signed_thousands, thousands, v};
