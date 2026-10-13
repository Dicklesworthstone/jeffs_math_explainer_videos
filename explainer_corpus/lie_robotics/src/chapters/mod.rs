//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where a picture first makes an idea true: the flat piece,
//! the screw, the product of exponentials; quieter for Apollo's story and
//! the honest limits). Each chapter starts as its storyboard's animatic and
//! is replaced by its real scene as it is built.

pub mod by_hand;
pub mod draw;
pub mod glyphs;
pub mod halfway;
pub mod lost_directions;
pub mod one_turn;
pub mod screws;
pub mod spin;
pub mod steps;
pub mod the_book;
pub mod the_platform;
pub mod the_spare_joint;
pub mod three_angles;
pub mod velocities_add;

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
        scene!("01_the_book", 0.4, the_book::TheBook),
        scene!("02_one_turn", 0.5, one_turn::OneTurn),
        scene!("03_three_angles", 0.4, three_angles::ThreeAngles),
        scene!("04_halfway", 0.6, halfway::Halfway),
        scene!("05_spin", 0.55, spin::Spin),
        scene!("06_the_platform", 0.5, the_platform::ThePlatform),
        scene!("07_screws", 0.6, screws::Screws),
        animatic!("08_seven_screws", 0.6),
        scene!("09_by_hand", 0.5, by_hand::ByHand),
        scene!("10_velocities_add", 0.5, velocities_add::VelocitiesAdd),
        scene!("11_lost_directions", 0.4, lost_directions::LostDirections),
        scene!("12_the_spare_joint", 0.5, the_spare_joint::TheSpareJoint),
        scene!("13_steps", 0.55, steps::Steps),
        animatic!("14_the_steady_path", 0.5),
        animatic!("15_everywhere", 0.65),
    ]
}
