//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod draw;
pub mod glyphs;
pub mod the_same_number;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_the_same_number",
        make: |kit| Box::new(the_same_number::TheSameNumber { kit }),
        intensity: 0.5,
    }]
}
