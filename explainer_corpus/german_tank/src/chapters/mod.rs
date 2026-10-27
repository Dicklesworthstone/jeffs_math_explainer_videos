//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Only chapters with a native scene are
//! registered: all16 planned chapters now have native scenes.
//! Chapters02 to04 and06 to
//! 11 share one number line, built in [`friends_line`]; chapters 04 to 13
//! share the halo, replacement and panel builders of [`what_we_assume`];
//! chapters 07 and 11 to 13 reuse the gap line or the halos of
//! [`gaps_trade_places`], and chapters 09, 10 and 13 the bars, count mark,
//! spans or fractions of [`one_estimate`].

pub mod a_rival_recipe;
pub mod a_thousand_boxes;
pub mod an_unfair_draw;
pub mod averaging_the_wobble;
pub mod counting_the_gaps;
pub mod draw;
pub mod every_handful;
pub mod five_boxes;
pub mod five_boxes_again;
pub mod friends_line;
pub mod gaps_trade_places;
pub mod glyphs;
pub mod how_far_off;
pub mod nothing_does_better;
pub mod one_estimate;
pub mod other_assumptions;
pub mod right_on_average;
pub mod the_largest_says_it_all;
pub mod what_we_assume;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_five_boxes",
            make: |kit| Box::new(five_boxes::FiveBoxes { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_a_thousand_boxes",
            make: |kit| Box::new(a_thousand_boxes::AThousandBoxes { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "03_counting_the_gaps",
            make: |kit| Box::new(counting_the_gaps::CountingTheGaps { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_what_we_assume",
            make: |kit| Box::new(what_we_assume::WhatWeAssume { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_every_handful",
            make: |kit| Box::new(every_handful::EveryHandful { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "06_gaps_trade_places",
            make: |kit| Box::new(gaps_trade_places::GapsTradePlaces { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "07_right_on_average",
            make: |kit| Box::new(right_on_average::RightOnAverage { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "08_one_estimate",
            make: |kit| Box::new(one_estimate::OneEstimate { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "09_how_far_off",
            make: |kit| Box::new(how_far_off::HowFarOff { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "10_a_rival_recipe",
            make: |kit| Box::new(a_rival_recipe::ARivalRecipe { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "11_the_largest_says_it_all",
            make: |kit| Box::new(the_largest_says_it_all::TheLargestSaysItAll { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "12_averaging_the_wobble",
            make: |kit| Box::new(averaging_the_wobble::AveragingTheWobble { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "13_nothing_does_better",
            make: |kit| Box::new(nothing_does_better::NothingDoesBetter { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "14_an_unfair_draw",
            make: |kit| Box::new(an_unfair_draw::AnUnfairDraw { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "15_other_assumptions",
            make: |kit| Box::new(other_assumptions::OtherAssumptions { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "16_five_boxes_again",
            make: |kit| Box::new(five_boxes_again::FiveBoxesAgain { kit }),
            intensity: 0.55,
        },
    ]
}
