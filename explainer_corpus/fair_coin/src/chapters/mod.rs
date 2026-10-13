//! One scene per chapter, in storyboard order, with the score's intensity
//! arc. Each chapter starts as its storyboard's animatic and is replaced by
//! its real scene as it is built.

pub mod glyphs;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    Vec::new()
}
