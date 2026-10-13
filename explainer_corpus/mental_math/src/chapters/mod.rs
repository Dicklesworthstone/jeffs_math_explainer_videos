//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a picture first makes a trick true and in the mixed
//! round, quieter for the story and the honest limits). A chapter is its
//! storyboard's animatic until its real scene is built: each real scene
//! sits behind a cargo feature (`ch01` … `ch14`) while the chapters are
//! being built side by side.

pub mod draw;
pub mod glyphs;
pub mod sound;

#[cfg(feature = "ch01")]
pub mod eighty_five_squared;
#[cfg(feature = "ch02")]
pub mod ends_in_five;

/// The chapters still being built: declared through a macro so that
/// `cargo fmt` does not look for a file that is not there yet.
macro_rules! being_built {
    ($($feature:literal $module:ident),* $(,)?) => {
        $(
            #[cfg(feature = $feature)]
            pub mod $module;
        )*
    };
}

being_built!(
    "ch03" your_turn,
    "ch04" by_parts,
    "ch05" running_total,
    "ch06" percents,
    "ch07" round_numbers,
    "ch08" the_middle,
    "ch09" bethe,
    "ch10" near_fifty,
    "ch11" your_turn_fifty,
    "ch12" which_trick,
    "ch13" nothing_to_grab,
    "ch14" near_something_you_know,
);

use explainer_kit::Chapter;
#[allow(unused_imports, reason = "unused once every chapter is a real scene")]
use explainer_kit::animatic::Animatic;

#[allow(unused_imports, reason = "unused once every chapter is a real scene")]
use crate::STORYBOARD;

/// A chapter: its real scene when its feature is on, its animatic otherwise.
macro_rules! chapter {
    ($feature:literal, $key:literal, $intensity:expr, $module:ident :: $scene:ident) => {{
        #[cfg(feature = $feature)]
        let c = Chapter {
            key: $key,
            make: |kit| Box::new($module::$scene { kit }),
            intensity: $intensity,
        };
        #[cfg(not(feature = $feature))]
        let c = Chapter {
            key: $key,
            make: |kit| Box::new(Animatic::new(kit, STORYBOARD, $key)),
            intensity: $intensity,
        };
        c
    }};
}

pub fn registry() -> Vec<Chapter> {
    vec![
        chapter!(
            "ch01",
            "01_eighty_five_squared",
            0.4,
            eighty_five_squared::EightyFiveSquared
        ),
        chapter!("ch02", "02_ends_in_five", 0.55, ends_in_five::EndsInFive),
        chapter!("ch03", "03_your_turn", 0.5, your_turn::YourTurn),
        chapter!("ch04", "04_by_parts", 0.5, by_parts::ByParts),
        chapter!("ch05", "05_running_total", 0.55, running_total::RunningTotal),
        chapter!("ch06", "06_percents", 0.45, percents::Percents),
        chapter!("ch07", "07_round_numbers", 0.5, round_numbers::RoundNumbers),
        chapter!("ch08", "08_the_middle", 0.55, the_middle::TheMiddle),
        chapter!("ch09", "09_bethe", 0.35, bethe::Bethe),
        chapter!("ch10", "10_near_fifty", 0.6, near_fifty::NearFifty),
        chapter!("ch11", "11_your_turn_fifty", 0.55, your_turn_fifty::YourTurnFifty),
        chapter!("ch12", "12_which_trick", 0.65, which_trick::WhichTrick),
        chapter!("ch13", "13_nothing_to_grab", 0.4, nothing_to_grab::NothingToGrab),
        chapter!(
            "ch14",
            "14_near_something_you_know",
            0.45,
            near_something_you_know::NearSomethingYouKnow
        ),
    ]
}
