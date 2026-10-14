//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the cubes close into their square, where the three
//! stacks become a box and where the four triangles slide; quietest for the
//! pattern that breaks and for the picture that lies). Every chapter is a
//! real scene; a module is declared here only once its file is committed.

pub mod a_third;
pub mod cold_open;
pub mod draw;
pub mod every_cube;
pub mod every_height;
pub mod five_for_five;
pub mod four_triangles;
pub mod glyphs;
pub mod halves;
pub mod iso;
pub mod kinetics;
pub mod layers;
pub mod siu;
pub mod slabs;
pub mod staircases;
pub mod the_box;
pub mod the_cut_cube;
pub mod the_missing_square;
pub mod three_stacks;
pub mod what_makes_a_proof;

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
        scene!("01_cold_open", 0.5, cold_open::ColdOpen),
        scene!("02_five_for_five", 0.3, five_for_five::FiveForFive),
        scene!("03_layers", 0.4, layers::Layers),
        scene!("04_staircases", 0.4, staircases::Staircases),
        scene!("05_slabs", 0.6, slabs::Slabs),
        scene!("06_every_cube", 0.55, every_cube::EveryCube),
        scene!("07_three_stacks", 0.55, three_stacks::ThreeStacks),
        scene!("08_the_box", 0.6, the_box::TheBox),
        scene!("09_every_height", 0.5, every_height::EveryHeight),
        scene!("10_halves", 0.4, halves::Halves),
        scene!("11_a_third", 0.5, a_third::AThird),
        scene!("12_the_cut_cube", 0.55, the_cut_cube::TheCutCube),
        scene!("13_four_triangles", 0.6, four_triangles::FourTriangles),
        scene!("14_the_missing_square", 0.3, the_missing_square::TheMissingSquare),
        scene!("15_what_makes_a_proof", 0.45, what_makes_a_proof::WhatMakesAProof),
    ]
}
