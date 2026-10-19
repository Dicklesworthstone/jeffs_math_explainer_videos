//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Only chapter 01 exists; the storyboard's
//! arc lists the planned rest.

pub mod draw;
pub mod glyphs;
pub mod the_coin;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_the_coin",
        make: |kit| Box::new(the_coin::TheCoin { kit }),
        intensity: 0.5,
    }]
}
