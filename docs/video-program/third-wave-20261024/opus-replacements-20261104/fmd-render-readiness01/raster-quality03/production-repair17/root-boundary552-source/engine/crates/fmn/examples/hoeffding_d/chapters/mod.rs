//! One `SceneConstruct` per chapter; each renders to its own artifact, so a
//! chapter can be re-rendered (or drafted small) on its own.

pub mod counting;
pub mod formula;
pub mod gallery;
pub mod glyphs;
pub mod hook;
pub mod outro;
pub mod quadruples;
pub mod ranks;
pub mod shuffle;

use fmn::prelude::SceneConstruct;

use crate::kit::Kit;

pub struct Entry {
    pub key: &'static str,
    pub make: fn(Kit) -> Box<dyn SceneConstruct>,
}

pub fn registry() -> Vec<Entry> {
    vec![
        Entry {
            key: "01_hook",
            make: |kit| Box::new(hook::Hook { kit }),
        },
        Entry {
            key: "02_gallery",
            make: |kit| Box::new(gallery::Gallery { kit }),
        },
        Entry {
            key: "03_quadruples",
            make: |kit| Box::new(quadruples::Quadruples { kit }),
        },
        Entry {
            key: "04_ranks",
            make: |kit| Box::new(ranks::Ranks { kit }),
        },
        Entry {
            key: "05_counting",
            make: |kit| Box::new(counting::Counting { kit }),
        },
        Entry {
            key: "06_formula",
            make: |kit| Box::new(formula::Formula { kit }),
        },
        Entry {
            key: "07_shuffle",
            make: |kit| Box::new(shuffle::Shuffle { kit }),
        },
        Entry {
            key: "08_outro",
            make: |kit| Box::new(outro::Outro { kit }),
        },
    ]
}
