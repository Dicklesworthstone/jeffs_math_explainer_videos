//! One scene per chapter, in storyboard order (`docs/storyboard.md`).
//! Chapter 01 is built; the continuation is written out in the storyboard.

pub mod draw;
pub mod glyphs;
pub mod limit_that_jumps;

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
    vec![scene!("01_a_limit_that_jumps", 0.35, limit_that_jumps::ALimitThatJumps)]
}
