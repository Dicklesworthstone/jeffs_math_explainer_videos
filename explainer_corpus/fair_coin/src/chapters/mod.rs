//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the trick first works and where the dice arrive,
//! quieter for the price and the breaks).

pub mod a_die_from_a_coin;
pub mod call_it;
pub mod draw;
pub mod every_coin;
pub mod glyphs;
pub mod loaded_dice;
pub mod same_counts;
pub mod six_orders;
pub mod sound;
pub mod the_fair_coin_inside;
pub mod the_price;
pub mod thumbnail;
pub mod toss_twice;
pub mod what_is_left;
pub mod where_it_breaks;

use explainer_kit::Chapter;

macro_rules! scene {
    ($key:literal, $intensity:expr, $module:ident :: $scene:ident) => {
        Chapter {
            key: $key,
            make: |kit| Box::new($module::$scene { kit }),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        scene!("01_call_it", 0.35, call_it::CallIt),
        scene!("02_toss_twice", 0.55, toss_twice::TossTwice),
        scene!("03_every_coin", 0.5, every_coin::EveryCoin),
        scene!("04_the_price", 0.4, the_price::ThePrice),
        scene!("05_where_it_breaks", 0.45, where_it_breaks::WhereItBreaks),
        scene!("06_same_counts", 0.5, same_counts::SameCounts),
        scene!("07_a_die_from_a_coin", 0.55, a_die_from_a_coin::ADieFromACoin),
        scene!("08_loaded_dice", 0.6, loaded_dice::LoadedDice),
        scene!("09_six_orders", 0.65, six_orders::SixOrders),
        scene!("10_what_is_left", 0.45, what_is_left::WhatIsLeft),
        scene!("11_the_fair_coin_inside", 0.4, the_fair_coin_inside::TheFairCoinInside),
    ]
}
