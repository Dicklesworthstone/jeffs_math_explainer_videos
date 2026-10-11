//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (a quiet hook, a fuller middle, the main result at 08, a sober close).
//! Each chapter starts as its storyboard's animatic and is replaced by its
//! real scene as it is built (01–03 are built).

pub mod cheating;
pub mod draw;
pub mod hook;
pub mod how_sure;
pub mod lake;
pub mod marks;
pub mod netfuls;
pub mod ring_toss;
pub mod small;
pub mod spread;

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
            key: "01_hook",
            make: |kit| Box::new(hook::Hook { kit }),
            intensity: 0.3,
        },
        Chapter {
            key: "02_the_marks_measure_the_net",
            make: |kit| Box::new(marks::Marks { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_why_the_marks_spread",
            make: |kit| Box::new(spread::Spread { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "04_netfuls",
            make: |kit| Box::new(netfuls::Netfuls { kit }),
            intensity: 0.5,
        },
        animatic!("05_turning_it_around", 0.5),
        animatic!("06_the_landscape", 0.6),
        animatic!("07_feller_s_comparison", 0.5),
        animatic!("08_the_table_balances", 0.7),
        Chapter {
            key: "09_check_it_by_cheating",
            make: |kit| Box::new(cheating::Cheating { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "10_small_experiments",
            make: |kit| Box::new(small::Small { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "11_how_sure",
            make: |kit| Box::new(how_sure::HowSure { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_the_ring_toss",
            make: |kit| Box::new(ring_toss::RingToss { kit }),
            intensity: 0.55,
        },
        animatic!("13_one_rule_for_every_failure", 0.5),
        animatic!("14_the_dark_cell_everywhere", 0.45),
        animatic!("15_counting_the_dead", 0.25),
    ]
}
