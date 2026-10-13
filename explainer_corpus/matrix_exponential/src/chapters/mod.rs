//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the matrix first goes into the series, where the flow
//! picture arrives and at the bridge to Lie theory; quieter for the honest
//! failures and the computing chapter). A chapter not yet built stands in as
//! its storyboard's animatic, and is replaced by its real scene as it lands.

pub mod draw;
pub mod e_to_a_matrix;
pub mod flow;
pub mod follow_the_arrows;
pub mod glyphs;
pub mod growth;
pub mod heat_and_chance;
pub mod how_computers_do_it;
pub mod one_function;
pub mod the_obvious_guess;
pub mod the_series;
pub mod turning;
pub mod two_tanks;
pub mod walks;

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
        scene!("01_e_to_a_matrix", 0.45, e_to_a_matrix::EToAMatrix),
        scene!("02_the_obvious_guess", 0.35, the_obvious_guess::TheObviousGuess),
        scene!("03_growth", 0.5, growth::Growth),
        scene!("04_the_series", 0.55, the_series::TheSeries),
        scene!("05_one_function", 0.5, one_function::OneFunction),
        scene!("06_turning", 0.6, turning::Turning),
        animatic!("07_a_matrix_in", 0.65),
        animatic!("08_does_it_settle", 0.5),
        scene!("09_follow_the_arrows", 0.65, follow_the_arrows::FollowTheArrows),
        scene!("10_two_tanks", 0.5, two_tanks::TwoTanks),
        scene!("11_walks", 0.5, walks::Walks),
        scene!("12_heat_and_chance", 0.55, heat_and_chance::HeatAndChance),
        scene!("13_how_computers_do_it", 0.4, how_computers_do_it::HowComputersDoIt),
        animatic!("14_order_matters", 0.5),
        animatic!("15_the_doorway", 0.65),
        animatic!("16_where_it_leads", 0.55),
    ]
}
