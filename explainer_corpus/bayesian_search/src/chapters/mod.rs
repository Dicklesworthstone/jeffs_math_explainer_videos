//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the dots first turn an empty search into a new map
//! and where the rule runs over the Navy's map; quietest for the 99 and for
//! what really happened in 1968). Each chapter starts as its storyboard's
//! animatic and is replaced by its real scene as it is built.

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
        animatic!("01_the_question", 0.4),
        animatic!("02_ninety_nine", 0.25),
        animatic!("03_a_map_of_chances", 0.45),
        animatic!("04_ten_thousand_dots", 0.5),
        animatic!("05_a_search_can_miss", 0.5),
        animatic!("06_nothing_found", 0.6),
        animatic!("07_bayes_rule", 0.55),
        animatic!("08_where_next", 0.65),
        animatic!("09_the_second_pass", 0.5),
        animatic!("10_adrift", 0.5),
        animatic!("11_twelve_hours", 0.45),
        animatic!("12_the_silence", 0.4),
        animatic!("13_how_well_they_looked", 0.3),
        animatic!("14_four_maps", 0.45),
    ]
}
