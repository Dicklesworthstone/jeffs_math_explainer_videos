//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the dots first turn an empty search into a new map
//! and where the rule runs over the Navy's map; quietest for the 99 and for
//! what really happened in 1968).

pub mod a_map_of_chances;
pub mod a_search_can_miss;
pub mod adrift;
pub mod bayes_rule;
pub mod draw;
pub mod four_maps;
pub mod glyphs;
pub mod how_well_they_looked;
pub mod ninety_nine;
pub mod nothing_found;
pub mod sound;
pub mod ten_thousand_dots;
pub mod the_question;
pub mod the_second_pass;
pub mod the_silence;
pub mod twelve_hours;
pub mod where_next;

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
        scene!("01_the_question", 0.4, the_question::TheQuestion),
        scene!("02_ninety_nine", 0.25, ninety_nine::NinetyNine),
        scene!("03_a_map_of_chances", 0.45, a_map_of_chances::AMapOfChances),
        scene!("04_ten_thousand_dots", 0.5, ten_thousand_dots::TenThousandDots),
        scene!("05_a_search_can_miss", 0.5, a_search_can_miss::ASearchCanMiss),
        scene!("06_nothing_found", 0.6, nothing_found::NothingFound),
        scene!("07_bayes_rule", 0.55, bayes_rule::BayesRule),
        scene!("08_where_next", 0.65, where_next::WhereNext),
        scene!("09_the_second_pass", 0.5, the_second_pass::TheSecondPass),
        scene!("10_adrift", 0.5, adrift::Adrift),
        scene!("11_twelve_hours", 0.45, twelve_hours::TwelveHours),
        scene!("12_the_silence", 0.4, the_silence::TheSilence),
        scene!("13_how_well_they_looked", 0.3, how_well_they_looked::HowWellTheyLooked),
        scene!("14_four_maps", 0.45, four_maps::FourMaps),
    ]
}
