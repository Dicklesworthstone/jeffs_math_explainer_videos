//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (a bright hook, the algebra quieter, the payoff fullest at 15 and 17).
//! Each chapter starts as its storyboard's animatic and is replaced by its
//! real scene as it is built.

// Video B's look, shared by every chapter: its helpers land ahead of the
// chapters that use them (lanes B-1, B-2, B-3); drop the allow when all 17
// chapters are built.
#[allow(dead_code)]
pub mod draw;

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
        animatic!("01_hook", 0.35),
        animatic!("02_bits_and_ex_or", 0.4),
        animatic!("03_cards", 0.45),
        animatic!("04_the_cube", 0.55),
        animatic!("05_the_ex_or_lottery", 0.5),
        animatic!("06_spares_and_rings", 0.45),
        animatic!("07_why_not_wrap_around_at_256", 0.45),
        animatic!("08_prime_clocks", 0.45),
        animatic!("09_bytes_are_polynomials", 0.5),
        animatic!("10_the_fold", 0.55),
        animatic!("11_the_byte_clock", 0.6),
        animatic!("12_the_times_table", 0.65),
        animatic!("13_the_byte_lottery", 0.55),
        animatic!("14_the_price_of_density", 0.5),
        animatic!("15_where_the_danger_lives", 0.65),
        animatic!("16_parking_a_piece", 0.55),
        animatic!("17_two_packets", 0.6),
    ]
}
