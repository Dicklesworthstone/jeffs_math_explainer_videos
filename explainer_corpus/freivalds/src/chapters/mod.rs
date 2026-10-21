//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod cheap_checks;
pub mod check_with_coins;
pub mod draw;
pub mod exact_and_rounded;
pub mod glyphs;
pub mod handed_an_answer;
pub mod never_more_than_half;
pub mod toss_again;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_handed_an_answer",
            make: |kit| Box::new(handed_an_answer::HandedAnAnswer { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "02_cheap_checks",
            make: |kit| Box::new(cheap_checks::CheapChecks { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "03_coins_choose",
            make: |kit| Box::new(check_with_coins::CoinsChoose { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_eight_tosses",
            make: |kit| Box::new(check_with_coins::EightTosses { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_shapes_of_mistakes",
            make: |kit| Box::new(never_more_than_half::ShapesOfMistakes { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_never_more_than_half",
            make: |kit| Box::new(never_more_than_half::NeverMoreThanHalf { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_toss_again",
            make: |kit| Box::new(toss_again::TossAgain { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "08_seen_first",
            make: |kit| Box::new(toss_again::SeenFirst { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "09_what_it_costs",
            make: |kit| Box::new(exact_and_rounded::WhatItCosts { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "10_exact_and_rounded",
            make: |kit| Box::new(exact_and_rounded::ExactAndRounded { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "11_the_promise",
            make: |kit| Box::new(exact_and_rounded::ThePromise { kit }),
            intensity: 0.45,
        },
    ]
}
