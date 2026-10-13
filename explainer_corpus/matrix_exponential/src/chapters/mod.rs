//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the matrix first goes into the series, where the flow
//! picture arrives and at the bridge to Lie theory; quieter for the honest
//! failures and the computing chapter). Each chapter starts as its
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
        animatic!("01_e_to_a_matrix", 0.45),
        animatic!("02_the_obvious_guess", 0.35),
        animatic!("03_growth", 0.5),
        animatic!("04_the_series", 0.55),
        animatic!("05_one_function", 0.5),
        animatic!("06_turning", 0.6),
        animatic!("07_a_matrix_in", 0.65),
        animatic!("08_does_it_settle", 0.5),
        animatic!("09_follow_the_arrows", 0.65),
        animatic!("10_two_tanks", 0.5),
        animatic!("11_walks", 0.5),
        animatic!("12_heat_and_chance", 0.55),
        animatic!("13_how_computers_do_it", 0.4),
        animatic!("14_order_matters", 0.5),
        animatic!("15_the_doorway", 0.65),
        animatic!("16_where_it_leads", 0.55),
    ]
}
