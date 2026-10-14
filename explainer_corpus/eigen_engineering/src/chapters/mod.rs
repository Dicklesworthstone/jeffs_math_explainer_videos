//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a pattern first comes back along itself and where the
//! truss's modes ring, quieter for the two bridges and the limits). Each
//! chapter starts as its storyboard's animatic and is replaced by its real
//! scene as it is built.

pub mod a_steel_truss;
pub mod an_economy;
pub mod arrows;
pub mod can_it_feed_itself;
pub mod carts;
pub mod draw;
pub mod glyphs;
pub mod one_idea;
pub mod own_patterns;
pub mod the_split;
pub mod truss;
pub mod two_carts;
pub mod where_modes_rule;

use explainer_kit::Chapter;
use explainer_kit::animatic::Animatic;

use crate::STORYBOARD;

macro_rules! scene {
    ($key:literal, $intensity:expr, $module:ident :: $scene:ident) => {
        Chapter {
            key: $key,
            make: |kit| Box::new($module::$scene { kit }),
            intensity: $intensity,
        }
    };
}

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
        animatic!("01_the_bridge_that_swayed", 0.4),
        scene!("02_two_carts", 0.45, two_carts::TwoCarts),
        scene!("03_own_patterns", 0.6, own_patterns::OwnPatterns),
        scene!("04_the_split", 0.55, the_split::TheSplit),
        scene!("05_a_steel_truss", 0.5, a_steel_truss::ASteelTruss),
        animatic!("06_twenty_one_patterns", 0.6),
        animatic!("07_every_mode_a_note", 0.65),
        animatic!("08_two_bridges", 0.35),
        animatic!("09_the_air_in_a_room", 0.5),
        animatic!("10_two_million_modes", 0.55),
        scene!("11_where_modes_rule", 0.45, where_modes_rule::WhereModesRule),
        scene!("12_an_economy", 0.5, an_economy::AnEconomy),
        scene!("13_can_it_feed_itself", 0.55, can_it_feed_itself::CanItFeedItself),
        scene!("14_one_idea", 0.5, one_idea::OneIdea),
    ]
}
