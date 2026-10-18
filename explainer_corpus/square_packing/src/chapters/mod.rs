//! One scene per chapter, in storyboard order. Every chapter is a real
//! scene; a module is declared here only once its file is committed
//! (chapters 07 and 08 of the storyboard's arc are not boarded yet).

pub mod a_triangle_of_points;
pub mod a_turn_by_a_hair;
pub mod draw;
pub mod four_pins;
pub mod glyphs;
pub mod near_a_side;
pub mod the_corner_lemma;
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
        scene!("03_a_turn_by_a_hair", 0.45, a_turn_by_a_hair::ATurnByAHair),
        scene!("04_the_corner_lemma", 0.45, the_corner_lemma::TheCornerLemma),
        scene!("05_near_a_side", 0.45, near_a_side::NearASide),
        scene!("06_a_triangle_of_points", 0.45, a_triangle_of_points::ATriangleOfPoints),
    ]
}
