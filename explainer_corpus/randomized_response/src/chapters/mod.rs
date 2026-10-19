//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Chapters 01 and 02 exist; the
//! storyboard's arc lists the planned rest.

pub mod draw;
pub mod glyphs;
pub mod the_coin;
pub mod what_a_yes_says;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_coin",
            make: |kit| Box::new(the_coin::TheCoin { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "02_what_a_yes_says",
            make: |kit| Box::new(what_a_yes_says::WhatAYesSays { kit }),
            intensity: 0.5,
        },
    ]
}
