//! One scene per chapter, in storyboard order. Chapters 01 and 02 are
//! built; the continuation (03 to 06) is written out in
//! `docs/storyboard.md` §2 and joins this registry, with its storyboard
//! rows, chapter by chapter.

pub mod draw;
pub mod five_percent;
pub mod glyphs;
pub mod imaginary_world;
pub mod same_test;

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
        scene!("01_five_percent_of_what", 0.35, five_percent::FivePercentOfWhat),
        scene!("02_the_same_test", 0.4, same_test::TheSameTest),
        scene!("03_the_imaginary_world", 0.45, imaginary_world::TheImaginaryWorld),
    ]
}
