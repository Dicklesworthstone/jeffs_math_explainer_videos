//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the cubes close into their square, where the three
//! stacks become a box and where the four triangles slide; quietest for the
//! pattern that breaks and for the picture that lies). Each chapter starts
//! as its storyboard's animatic and is replaced by its real scene as it is
//! built; a module is declared here only once its file is committed.

pub mod a_third;
pub mod cold_open;
pub mod draw;
pub mod five_for_five;
pub mod glyphs;
pub mod iso;

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

/// A chapter that is a real scene: `$module::$ty { kit }`.
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
        scene!("01_cold_open", 0.5, cold_open::ColdOpen),
        scene!("02_five_for_five", 0.3, five_for_five::FiveForFive),
        animatic!("03_layers", 0.4),
        animatic!("04_staircases", 0.4),
        animatic!("05_slabs", 0.6),
        animatic!("06_every_cube", 0.55),
        animatic!("07_three_stacks", 0.55),
        animatic!("08_the_box", 0.6),
        animatic!("09_every_height", 0.5),
        animatic!("10_halves", 0.4),
        scene!("11_a_third", 0.5, a_third::AThird),
        animatic!("12_the_cut_cube", 0.55),
        animatic!("13_four_triangles", 0.6),
        animatic!("14_the_missing_square", 0.3),
        animatic!("15_what_makes_a_proof", 0.45),
    ]
}
