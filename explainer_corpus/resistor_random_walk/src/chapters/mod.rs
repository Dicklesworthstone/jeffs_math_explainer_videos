//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod draw;
pub mod glyphs;
pub mod round_after_round;
pub mod the_same_number;
pub mod why_an_average;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_same_number",
            make: |kit| Box::new(the_same_number::TheSameNumber { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "02_why_an_average",
            make: |kit| Box::new(why_an_average::WhyAnAverage { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "03_round_after_round",
            make: |kit| Box::new(round_after_round::RoundAfterRound { kit }),
            intensity: 0.5,
        },
    ]
}
