//! One scene per chapter, in storyboard order. Every chapter is a real
//! scene; a module is declared here only once its file is committed
//! (chapters 03 to 06 of the storyboard's arc are not boarded yet).

pub mod draw;
pub mod four_pins;
pub mod glyphs;
pub mod the_turned_tile;

use explainer_kit::Chapter;

/// A chapter that is a real scene: `$module::$ty { kit }`.
macro_rules! scene {
    ($key:literal, $intensity:expr, $module:ident::$ty:ident) => {
        Chapter {
            key: $key,
            make: |kit| Box::new($module::$ty { kit }),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        scene!("01_the_turned_tile", 0.45, the_turned_tile::TheTurnedTile),
        scene!("02_four_pins", 0.5, four_pins::FourPins),
    ]
}
