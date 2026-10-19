//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Chapters 01 to 05, the storyboard's
//! whole arc, exist.

pub mod asking_again;
pub mod draw;
pub mod glyphs;
pub mod the_coin;
pub mod the_dial;
pub mod the_price_of_the_coin;
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
        Chapter {
            key: "03_the_price_of_the_coin",
            make: |kit| Box::new(the_price_of_the_coin::ThePriceOfTheCoin { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_asking_again",
            make: |kit| Box::new(asking_again::AskingAgain { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_the_dial",
            make: |kit| Box::new(the_dial::TheDial { kit }),
            intensity: 0.5,
        },
    ]
}
