//! One scene per chapter, in storyboard order. Chapter 01 is built; the
//! continuation (02 to 06) is written out in `docs/storyboard.md` §2 and
//! joins this registry, with its storyboard rows, chapter by chapter.

pub mod draw;
pub mod five_percent;
pub mod glyphs;

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
    vec![scene!("01_five_percent_of_what", 0.35, five_percent::FivePercentOfWhat)]
}
