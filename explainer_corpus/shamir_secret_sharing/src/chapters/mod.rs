//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Only implemented chapters are registered
//! All23 storyboard chapters are implemented and registered.

pub mod a_curve_for_three;
pub mod a_liar;
pub mod a_line_through_the_secret;
pub mod a_ten_mark_dial;
pub mod an_experiment;
pub mod any_three;
pub mod board;
pub mod cutting_it_up;
pub mod dividing_on_the_dial;
pub mod draw;
pub mod envelope;
pub mod fraction_weights;
pub mod glyphs;
pub mod helper_curves;
pub mod one_more_share;
pub mod padlocks;
pub mod plot;
pub mod ten_marks_again;
pub mod the_dial;
pub mod the_goal;
pub mod the_line_on_the_dial;
pub mod the_vault;
pub mod thirds_that_do_not_cancel;
pub mod three_keys_two_nothing;
pub mod three_shares;
pub mod two_keys;
pub mod two_shares;
pub mod what_it_promises;
pub mod why_prime;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_vault",
            make: |kit| Box::new(the_vault::TheVault { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "02_cutting_it_up",
            make: |kit| Box::new(cutting_it_up::CuttingItUp { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "03_the_dial",
            make: |kit| Box::new(the_dial::TheDial { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_the_goal",
            make: |kit| Box::new(the_goal::TheGoal { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_an_experiment",
            make: |kit| Box::new(an_experiment::AnExperiment { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_padlocks",
            make: |kit| Box::new(padlocks::ThePadlocks { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_a_line_through_the_secret",
            make: |kit| Box::new(a_line_through_the_secret::ALineThroughTheSecret { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "08_the_line_on_the_dial",
            make: |kit| Box::new(the_line_on_the_dial::TheLineOnTheDial { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "09_a_curve_for_three",
            make: |kit| Box::new(a_curve_for_three::ACurveForThree { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_two_shares",
            make: |kit| Box::new(two_shares::TwoShares { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "11_one_more_share",
            make: |kit| Box::new(one_more_share::OneMoreShare { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_a_ten_mark_dial",
            make: |kit| Box::new(a_ten_mark_dial::ATenMarkDial { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "13_why_prime",
            make: |kit| Box::new(why_prime::WhyPrime { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "14_helper_curves",
            make: |kit| Box::new(helper_curves::HelperCurves { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "15_three_shares",
            make: |kit| Box::new(three_shares::ThreeShares { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "16_fraction_weights",
            make: |kit| Box::new(fraction_weights::FractionWeights { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "17_thirds_that_do_not_cancel",
            make: |kit| Box::new(thirds_that_do_not_cancel::ThirdsThatDoNotCancel { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "18_dividing_on_the_dial",
            make: |kit| Box::new(dividing_on_the_dial::DividingOnTheDial { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "19_any_three",
            make: |kit| Box::new(any_three::AnyThree { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "20_ten_marks_again",
            make: |kit| Box::new(ten_marks_again::TenMarksAgain { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "21_what_it_promises",
            make: |kit| Box::new(what_it_promises::WhatItPromises { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "22_a_liar",
            make: |kit| Box::new(a_liar::ALiar { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "23_three_keys_two_nothing",
            make: |kit| Box::new(three_keys_two_nothing::ThreeKeysTwoNothing { kit }),
            intensity: 0.5,
        },
    ]
}
