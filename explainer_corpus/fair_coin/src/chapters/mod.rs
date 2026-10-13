//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the trick first works and where the dice arrive,
//! quieter for the price and the breaks). Each chapter starts as its
//! storyboard's animatic and is replaced by its real scene as it is built.

pub mod glyphs;

use explainer_kit::Chapter;
use explainer_kit::animatic::Animatic;

use crate::STORYBOARD;

macro_rules! animatic {
    ($key:literal, $intensity:expr) => {
        Chapter {
            key: $key,
            make: |kit| Box::new(Animatic::new(kit, STORYBOARD, $key)),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        animatic!("01_call_it", 0.35),
        animatic!("02_toss_twice", 0.55),
        animatic!("03_every_coin", 0.5),
        animatic!("04_the_price", 0.4),
        animatic!("05_where_it_breaks", 0.45),
        animatic!("06_same_counts", 0.5),
        animatic!("07_a_die_from_a_coin", 0.55),
        animatic!("08_loaded_dice", 0.6),
        animatic!("09_six_bricks", 0.65),
        animatic!("10_what_is_left", 0.45),
        animatic!("11_rigorously_unbiased", 0.4),
    ]
}
