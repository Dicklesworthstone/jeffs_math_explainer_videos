//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (rubato and quieter for the proof, the room's tunes fullest at 06 and
//! 13). Each chapter starts as its storyboard's animatic and is replaced by
//! its real scene as it is built.

pub mod draw;
pub mod glyphs;
pub mod room;
pub mod scoreboard;
pub mod three_customers;
pub mod tickets;
pub mod ties;

use explainer_kit::Chapter;
use explainer_kit::animatic::Animatic;

use crate::STORYBOARD;

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
        Chapter {
            key: "01_one_plus_one_is_one",
            make: |kit| Box::new(scoreboard::OnePlusOne { kit }),
            intensity: 0.35,
        },
        Chapter {
            key: "02_the_room",
            make: |kit| Box::new(room::TheRoom { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_take_a_number",
            make: |kit| Box::new(tickets::TakeANumber { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_same_number",
            make: |kit| Box::new(ties::SameNumber { kit }),
            intensity: 0.5,
        },
        animatic!("05_the_unwritten_ticket", 0.55),
        Chapter {
            key: "06_three_customers",
            make: |kit| Box::new(three_customers::ThreeCustomers { kit }),
            intensity: 0.65,
        },
        animatic!("07_one_run", 0.6),
        animatic!("08_why_never_two", 0.4),
        animatic!("09_reading_garbage_safely", 0.45),
        animatic!("10_the_old_zero", 0.45),
        animatic!("11_fair_at_a_price", 0.5),
        animatic!("12_your_cpu", 0.55),
        animatic!("13_proudest", 0.4),
    ]
}
