//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a picture first makes a trick true and in the mixed
//! round, quieter for the story and the honest limits). Each chapter starts
//! as its storyboard's animatic and is replaced by its real scene as it is
//! built.

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
        animatic!("01_eighty_five_squared", 0.4),
        animatic!("02_ends_in_five", 0.55),
        animatic!("03_your_turn", 0.5),
        animatic!("04_by_parts", 0.5),
        animatic!("05_running_total", 0.55),
        animatic!("06_the_middle", 0.55),
        animatic!("07_bethe", 0.35),
        animatic!("08_near_fifty", 0.55),
        animatic!("09_your_turn_fifty", 0.5),
        animatic!("10_percents", 0.45),
        animatic!("11_round_numbers", 0.5),
        animatic!("12_which_trick", 0.65),
        animatic!("13_nothing_to_grab", 0.4),
        animatic!("14_near_something_you_know", 0.45),
    ]
}
