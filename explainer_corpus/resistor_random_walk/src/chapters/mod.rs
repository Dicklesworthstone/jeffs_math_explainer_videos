//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod a_bridge;
pub mod any_wires;
pub mod draw;
pub mod glyphs;
pub mod line_circuit;
pub mod one_answer;
pub mod only_one;
pub mod round_after_round;
pub mod the_same_number;
pub mod walkers_rule;
pub mod when_to_stop;
pub mod which_end;
pub mod why_an_average;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_which_end",
            make: |kit| Box::new(which_end::WhichEnd { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "02_the_line_as_a_circuit",
            make: |kit| Box::new(line_circuit::LineCircuit { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "03_a_bridge",
            make: |kit| Box::new(a_bridge::ABridge { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_the_same_number",
            make: |kit| Box::new(the_same_number::TheSameNumber { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_why_an_average",
            make: |kit| Box::new(why_an_average::WhyAnAverage { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_the_walkers_rule",
            make: |kit| Box::new(walkers_rule::WalkersRule { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_one_answer_two_ways",
            make: |kit| Box::new(one_answer::OneAnswer { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "08_round_after_round",
            make: |kit| Box::new(round_after_round::RoundAfterRound { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "09_only_one",
            make: |kit| Box::new(only_one::OnlyOne { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_any_wires",
            make: |kit| Box::new(any_wires::AnyWires { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "11_when_to_stop",
            make: |kit| Box::new(when_to_stop::WhenToStop { kit }),
            intensity: 0.5,
        },
    ]
}
