//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Only chapters with a real native scene
//! are registered (root, Mail 6790); the draft's chapters 26 to 28 are in
//! `docs/storyboard-draft.md` until their scenes are written.

pub mod a_coin_that_never_changes;
pub mod a_crowd_of_sticks;
pub mod a_sheet_of_lines;
pub mod a_square_on_the_sheet;
pub mod a_thousand_drops;
pub mod an_exchange_rate;
pub mod barbiers_floorboards;
pub mod draw;
pub mod fixed_directions;
pub mod glyphs;
pub mod how_long_is_a_coastline;
pub mod one_count_one_placement;
pub mod placement;
pub mod ripples;
pub mod sheet;
pub mod straight_sticks;
pub mod ten_drops;
pub mod the_bent_stick;
pub mod the_bet;
pub mod the_chart_of_lines;
pub mod the_coasts_chart;
pub mod the_formula;
pub mod the_gap;
pub mod the_longimeter;
pub mod the_question;
pub mod the_same_width;
pub mod the_shadow_rule;
pub mod the_sticks_band;
pub mod turning_the_sheet;
pub mod turning_the_stick;
pub mod two_ways_to_pick_a_line;
pub mod why_every_curve;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_question",
            make: |kit| Box::new(the_question::TheQuestion { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_a_sheet_of_lines",
            make: |kit| Box::new(a_sheet_of_lines::ASheetOfLines { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_an_exchange_rate",
            make: |kit| Box::new(an_exchange_rate::AnExchangeRate { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_one_count_one_placement",
            make: |kit| Box::new(one_count_one_placement::OneCountOnePlacement { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_a_coin_that_never_changes",
            make: |kit| Box::new(a_coin_that_never_changes::ACoinThatNeverChanges { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "06_the_bet",
            make: |kit| Box::new(the_bet::TheBet { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_straight_sticks",
            make: |kit| Box::new(straight_sticks::StraightSticks { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_the_shadow_rule",
            make: |kit| Box::new(the_shadow_rule::TheShadowRule { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_turning_the_stick",
            make: |kit| Box::new(turning_the_stick::TurningTheStick { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_a_crowd_of_sticks",
            make: |kit| Box::new(a_crowd_of_sticks::ACrowdOfSticks { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "11_turning_the_sheet",
            make: |kit| Box::new(turning_the_sheet::TurningTheSheet { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_why_every_curve",
            make: |kit| Box::new(why_every_curve::WhyEveryCurve { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "13_the_bent_stick",
            make: |kit| Box::new(the_bent_stick::TheBentStick { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "14_the_same_width",
            make: |kit| Box::new(the_same_width::TheSameWidth { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "15_barbiers_floorboards",
            make: |kit| Box::new(barbiers_floorboards::BarbiersFloorboards { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "16_two_ways_to_pick_a_line",
            make: |kit| Box::new(two_ways_to_pick_a_line::TwoWaysToPickALine { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "17_the_chart_of_lines",
            make: |kit| Box::new(the_chart_of_lines::TheChartOfLines { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "18_the_sticks_band",
            make: |kit| Box::new(the_sticks_band::TheSticksBand { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "19_the_coasts_chart",
            make: |kit| Box::new(the_coasts_chart::TheCoastsChart { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "20_the_formula",
            make: |kit| Box::new(the_formula::TheFormula { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "21_ten_drops",
            make: |kit| Box::new(ten_drops::TenDrops { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "22_a_thousand_drops",
            make: |kit| Box::new(a_thousand_drops::AThousandDrops { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "23_fixed_directions",
            make: |kit| Box::new(fixed_directions::FixedDirections { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "24_the_longimeter",
            make: |kit| Box::new(the_longimeter::TheLongimeter { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "25_the_gap",
            make: |kit| Box::new(the_gap::TheGap { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "26_ripples",
            make: |kit| Box::new(ripples::Ripples { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "27_how_long_is_a_coastline",
            make: |kit| Box::new(how_long_is_a_coastline::HowLongIsACoastline { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "28_a_square_on_the_sheet",
            make: |kit| Box::new(a_square_on_the_sheet::ASquareOnTheSheet { kit }),
            intensity: 0.5,
        },
    ]
}
