//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Only chapters with native scenes are
//! registered (and only they are in `docs/storyboard.md` and `script.tsv`);
//! the rest of the film is planned in `docs/storyboard-draft.md` and joins
//! here, words unchanged, when its scene is written.

pub mod does_b_win;
pub mod draw;
pub mod glyphs;
pub mod looks_hopeless;
pub mod mix_the_games;
pub mod net_flow;
pub mod one_dial;
pub mod one_player_to_a_crowd;
pub mod ten_four_twelve;
pub mod the_catch;
pub mod the_clue;
pub mod the_half_point;
pub mod the_puzzle;
pub mod the_rule;
pub mod the_trap;
pub mod three_corners;
pub mod two_products;
pub mod what_winning_promises;
pub mod when_switching_fails;
pub mod where_you_stand;

use explainer_kit::Chapter;

macro_rules! scene {
    ($key:literal, $intensity:expr, $module:ident::$ty:ident) => {
        Chapter {
            key: $key,
            make: |kit| Box::new($module::$ty { kit }),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        scene!("01_the_puzzle", 0.45, the_puzzle::ThePuzzle),
        scene!("02_the_rule", 0.45, the_rule::TheRule),
        scene!("03_does_b_win", 0.45, does_b_win::DoesBWin),
        scene!("04_looks_hopeless", 0.45, looks_hopeless::LooksHopeless),
        scene!("05_the_clue", 0.45, the_clue::TheClue),
        scene!("06_three_corners", 0.45, three_corners::ThreeCorners),
        scene!("07_where_you_stand", 0.45, where_you_stand::WhereYouStand),
        scene!(
            "08_one_player_to_a_crowd",
            0.45,
            one_player_to_a_crowd::OnePlayerToACrowd
        ),
        scene!("09_ten_four_twelve", 0.45, ten_four_twelve::TenFourTwelve),
        scene!("10_the_trap", 0.45, the_trap::TheTrap),
        scene!("11_mix_the_games", 0.45, mix_the_games::MixTheGames),
        scene!("12_one_dial", 0.45, one_dial::OneDial),
        scene!("13_the_half_point", 0.45, the_half_point::TheHalfPoint),
        scene!("14_net_flow", 0.45, net_flow::NetFlow),
        scene!("15_two_products", 0.45, two_products::TwoProducts),
        scene!(
            "16_when_switching_fails",
            0.45,
            when_switching_fails::WhenSwitchingFails
        ),
        scene!(
            "17_what_winning_promises",
            0.45,
            what_winning_promises::WhatWinningPromises
        ),
        scene!("18_the_catch", 0.35, the_catch::TheCatch),
    ]
}
