//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod draw;
pub mod glyphs;
pub mod two_bad_echoes;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_two_bad_echoes",
        make: |kit| Box::new(two_bad_echoes::TwoBadEchoes { kit }),
        intensity: 0.5,
    }]
}
