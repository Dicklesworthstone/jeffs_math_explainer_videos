//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.
//!
//! Only chapters with native scene source are registered, and they are
//! exactly the chapters of `docs/storyboard.md` and `script.tsv`: all twelve
//! chapters of the full draft, `docs/storyboard-draft.md`, which holds the
//! storyboard verbatim. A new chapter moves into the storyboard and this
//! registry only when its scene source exists, never as a stand-in.

pub mod at_random;
pub mod back_to_the_stop;
pub mod draw;
pub mod glyphs;
pub mod no_bus_late;
pub mod one_a_minute;
pub mod one_more_timetable;
pub mod same_sum;
pub mod stretch_one_gap;
pub mod the_formula;
pub mod the_stop;
pub mod timetable;
pub mod triangles;
pub mod twelve_minutes;
pub mod two_averages;
pub mod two_ways_to_count;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_the_stop",
            make: |kit| Box::new(the_stop::TheStop { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_two_ways_to_count",
            make: |kit| Box::new(two_ways_to_count::TwoWaysToCount { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "03_stretch_one_gap",
            make: |kit| Box::new(stretch_one_gap::StretchOneGap { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "04_one_a_minute",
            make: |kit| Box::new(one_a_minute::OneAMinute { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_triangles",
            make: |kit| Box::new(triangles::Triangles { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_two_averages",
            make: |kit| Box::new(two_averages::TwoAverages { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "07_no_bus_late",
            make: |kit| Box::new(no_bus_late::NoBusLate { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "08_the_formula",
            make: |kit| Box::new(the_formula::TheFormula { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_at_random",
            make: |kit| Box::new(at_random::AtRandom { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_same_sum",
            make: |kit| Box::new(same_sum::SameSum { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "11_back_to_the_stop",
            make: |kit| Box::new(back_to_the_stop::BackToTheStop { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "12_one_more_timetable",
            make: |kit| Box::new(one_more_timetable::OneMoreTimetable { kit }),
            intensity: 0.5,
        },
    ]
}

#[cfg(test)]
mod tests {
    use explainer_kit::Kit;

    use super::*;

    /// Every chapter's scene reports its own key.
    #[test]
    fn every_scene_names_its_chapter() {
        let kit = Kit::new().unwrap();
        for c in registry() {
            assert_eq!((c.make)(kit.clone()).name(), c.key);
        }
    }
}
