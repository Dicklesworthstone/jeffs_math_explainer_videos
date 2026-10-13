//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a picture first makes a trick true and in the mixed
//! round, quieter for the story and the honest limits).

pub mod bethe;
pub mod by_parts;
pub mod draw;
pub mod eighty_five_squared;
pub mod ends_in_five;
pub mod glyphs;
pub mod near_fifty;
pub mod near_something_you_know;
pub mod nothing_to_grab;
pub mod percents;
pub mod round_numbers;
pub mod running_total;
pub mod sound;
pub mod the_middle;
pub mod which_trick;
pub mod your_turn;
pub mod your_turn_fifty;

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
        scene!("01_eighty_five_squared", 0.4, eighty_five_squared::EightyFiveSquared),
        scene!("02_ends_in_five", 0.55, ends_in_five::EndsInFive),
        scene!("03_your_turn", 0.5, your_turn::YourTurn),
        scene!("04_by_parts", 0.5, by_parts::ByParts),
        scene!("05_running_total", 0.55, running_total::RunningTotal),
        scene!("06_percents", 0.45, percents::Percents),
        scene!("07_round_numbers", 0.5, round_numbers::RoundNumbers),
        scene!("08_the_middle", 0.55, the_middle::TheMiddle),
        scene!("09_bethe", 0.35, bethe::Bethe),
        scene!("10_near_fifty", 0.6, near_fifty::NearFifty),
        scene!("11_your_turn_fifty", 0.55, your_turn_fifty::YourTurnFifty),
        scene!("12_which_trick", 0.65, which_trick::WhichTrick),
        scene!("13_nothing_to_grab", 0.4, nothing_to_grab::NothingToGrab),
        scene!(
            "14_near_something_you_know",
            0.45,
            near_something_you_know::NearSomethingYouKnow
        ),
    ]
}
