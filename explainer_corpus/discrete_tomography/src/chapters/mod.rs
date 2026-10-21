//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod chains;
pub mod draw;
pub mod glyphs;
pub mod group;
pub mod misses;
pub mod only;
pub mod promise;
pub mod reasoning;
pub mod same_shadows;
pub mod stops;
pub mod third;
pub mod water;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_same_shadows",
            make: |kit| Box::new(same_shadows::SameShadows { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "02_square_by_square",
            make: |kit| Box::new(reasoning::SquareBySquare { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_water_through_the_grid",
            make: |kit| Box::new(water::WaterThroughTheGrid { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_taking_water_back",
            make: |kit| Box::new(water::TakingWaterBack { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_where_the_water_stops",
            make: |kit| Box::new(stops::WhereTheWaterStops { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_a_group_that_asks_too_much",
            make: |kit| Box::new(group::AGroupThatAsksTooMuch { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_why_the_test_never_misses",
            make: |kit| Box::new(misses::WhyTheTestNeverMisses { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "08_the_only_picture",
            make: |kit| Box::new(only::TheOnlyPicture { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "09_chains_of_switches",
            make: |kit| Box::new(chains::ChainsOfSwitches { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "10_more_shadows",
            make: |kit| Box::new(third::AThirdShadow { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "11_what_two_shadows_promise",
            make: |kit| Box::new(promise::WhatTwoShadowsPromise { kit }),
            intensity: 0.4,
        },
    ]
}
