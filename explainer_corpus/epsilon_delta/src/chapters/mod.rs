//! One scene per chapter, in storyboard order (`docs/storyboard.md`).
//! Chapters 01 to 04 are built; the continuation is written out in the
//! storyboard.

pub mod challenge_game;
pub mod draw;
pub mod glyphs;
pub mod limit_that_jumps;
pub mod old_objection;
pub mod shrinking_region;

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
        scene!("01_a_limit_that_jumps", 0.35, limit_that_jumps::ALimitThatJumps),
        scene!("02_the_shrinking_region", 0.4, shrinking_region::TheShrinkingRegion),
        scene!(
            "03_what_the_old_objection_was_about",
            0.4,
            old_objection::WhatTheOldObjectionWasAbout
        ),
        scene!("04_the_challenge_game", 0.4, challenge_game::TheChallengeGame),
    ]
}
