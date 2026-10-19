//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`. Chapters 03 and 04 are named in the
//! storyboard's arc and not written yet.

pub mod check_with_coins;
pub mod draw;
pub mod glyphs;
pub mod never_more_than_half;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_check_with_coins",
            make: |kit| Box::new(check_with_coins::CheckWithCoins { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "02_never_more_than_half",
            make: |kit| Box::new(never_more_than_half::NeverMoreThanHalf { kit }),
            intensity: 0.5,
        },
    ]
}
