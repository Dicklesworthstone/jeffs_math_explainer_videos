//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod draw;
pub mod glyphs;
pub mod same_shadows;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_same_shadows",
        make: |kit| Box::new(same_shadows::SameShadows { kit }),
        intensity: 0.45,
    }]
}
