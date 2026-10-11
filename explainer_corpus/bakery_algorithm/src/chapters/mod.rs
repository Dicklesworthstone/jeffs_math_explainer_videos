//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (rubato and quieter for the proof, the room's tunes fullest at 06 and
//! 13).

pub mod cpu;
pub mod draw;
pub mod fair;
pub mod flag;
pub mod garbage;
pub mod glyphs;
pub mod old_zero;
pub mod one_run;
pub mod proudest;
pub mod room;
pub mod scoreboard;
pub mod three_customers;
pub mod tickets;
pub mod ties;
pub mod tunes;
pub mod why_never_two;

use explainer_kit::Chapter;

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
        Chapter {
            key: "05_the_unwritten_ticket",
            make: |kit| Box::new(flag::UnwrittenTicket { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "06_three_customers",
            make: |kit| Box::new(three_customers::ThreeCustomers { kit }),
            intensity: 0.65,
        },
        Chapter {
            key: "07_one_run",
            make: |kit| Box::new(one_run::OneRun { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "08_why_never_two",
            make: |kit| Box::new(why_never_two::WhyNeverTwo { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_reading_garbage_safely",
            make: |kit| Box::new(garbage::ReadingGarbage { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "10_the_old_zero",
            make: |kit| Box::new(old_zero::OldZero { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "11_fair_at_a_price",
            make: |kit| Box::new(fair::FairAtAPrice { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_your_cpu",
            make: |kit| Box::new(cpu::YourCpu { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "13_proudest",
            make: |kit| Box::new(proudest::Proudest { kit }),
            intensity: 0.4,
        },
    ]
}
