//! One `SceneConstruct` per chapter; each renders to its own MP4 so no
//! single ffmpeg session outlives the boundary's job limits. The shared CLI
//! builds each chapter's kit; the chapters see it through `crate::kit::Kit`.

pub mod counting;
pub mod formula;
pub mod gallery;
pub mod glyphs;
pub mod hook;
pub mod outro;
pub mod quadruples;
pub mod ranks;
pub mod shuffle;
pub mod thumbnail;

use explainer_kit::Chapter;

use crate::kit::Kit;

/// The chapters in order, with the score's intensity arc (a quiet hook, a
/// fuller middle, a settled outro).
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_hook",
            make: |kit| {
                Box::new(hook::Hook {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.3,
        },
        Chapter {
            key: "02_gallery",
            make: |kit| {
                Box::new(gallery::Gallery {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.45,
        },
        Chapter {
            key: "03_quadruples",
            make: |kit| {
                Box::new(quadruples::Quadruples {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.5,
        },
        Chapter {
            key: "04_ranks",
            make: |kit| {
                Box::new(ranks::Ranks {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.5,
        },
        Chapter {
            key: "05_counting",
            make: |kit| {
                Box::new(counting::Counting {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.55,
        },
        Chapter {
            key: "06_formula",
            make: |kit| {
                Box::new(formula::Formula {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.6,
        },
        Chapter {
            key: "07_shuffle",
            make: |kit| {
                Box::new(shuffle::Shuffle {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.65,
        },
        Chapter {
            key: "08_outro",
            make: |kit| {
                Box::new(outro::Outro {
                    kit: Kit::wrap(kit),
                })
            },
            intensity: 0.4,
        },
    ]
}
