//! One `SceneConstruct` per chapter, in storyboard order, with its score
//! intensity (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k`
//! belong to the chapter with the same `NN`.

pub mod glyphs;
pub mod intro;
pub mod outro;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_intro",
            make: |kit| Box::new(intro::Intro { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "02_outro",
            make: |kit| Box::new(outro::Outro { kit }),
            intensity: 0.40,
        },
    ]
}
