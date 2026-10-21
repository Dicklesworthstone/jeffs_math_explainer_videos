//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Chapters 01 to 13, the storyboard's
//! whole arc, exist.

pub mod any_class;
pub mod asking_again;
pub mod asking_straight_out;
pub mod draw;
pub mod glyphs;
pub mod one_students_coins;
pub mod rule_not_followed;
pub mod running_one;
pub mod the_coin;
pub mod the_dial;
pub mod the_price_of_the_coin;
pub mod what_a_yes_says;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_asking_straight_out",
            make: |kit| Box::new(asking_straight_out::AskingStraightOut { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "02_one_students_coins",
            make: |kit| Box::new(one_students_coins::OneStudentsCoins { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "03_the_class_on_average",
            make: |kit| Box::new(the_coin::TheClassOnAverage { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_any_class",
            make: |kit| Box::new(any_class::AnyClass { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_what_a_yes_says",
            make: |kit| Box::new(what_a_yes_says::WhatAYesSays { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_real_coins_scatter",
            make: |kit| Box::new(the_price_of_the_coin::RealCoinsScatter { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_how_far_one_survey_misses",
            make: |kit| Box::new(the_price_of_the_coin::HowFarOneSurveyMisses { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "08_the_price_of_the_coin",
            make: |kit| Box::new(the_price_of_the_coin::ThePriceOfTheCoin { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "09_when_the_rule_is_not_followed",
            make: |kit| Box::new(rule_not_followed::WhenTheRuleIsNotFollowed { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_asking_again",
            make: |kit| Box::new(asking_again::AskingAgain { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "11_one_flip",
            make: |kit| Box::new(asking_again::OneFlip { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_the_dial",
            make: |kit| Box::new(the_dial::TheDial { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "13_running_one",
            make: |kit| Box::new(running_one::RunningOne { kit }),
            intensity: 0.5,
        },
    ]
}
