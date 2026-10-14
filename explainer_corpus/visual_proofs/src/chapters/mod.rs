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
pub mod sound;
pub mod staircases;
pub mod the_box;
pub mod the_cut_cube;
pub mod the_missing_square;
pub mod three_stacks;
pub mod what_makes_a_proof;

use explainer_kit::Chapter;

use self::draw::Filed;

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

/// The proofs whose cards file a picture on the shelf, in the order the cards
/// pass: the chapter that files, its slot, and the function that rebuilds
/// the filed thumbnail (every chapter after it opens with
/// `draw::shelf_so_far`, which puts these up). A chapter that lands its
/// filing adds its line here; a card that refuses (chapter 14) leaves its
/// slot's empty outline.
pub const SHELF: &[Filed] = &[
    Filed {
        chapter: 3,
        slot: layers::SLOT,
        thumb: layers::thumb,
    },
    Filed {
        chapter: 4,
        slot: staircases::SLOT,
        thumb: staircases::thumb,
    },
    Filed {
        chapter: 6,
        slot: every_cube::SLOT,
        thumb: every_cube::thumb,
    },
    Filed {
        chapter: 9,
        slot: every_height::SLOT,
        thumb: every_height::thumb,
    },
    Filed {
        chapter: 10,
        slot: halves::SLOT,
        thumb: halves::thumb,
    },
    Filed {
        chapter: 11,
        slot: a_third::SLOT,
        thumb: a_third::thumb,
    },
    Filed {
        chapter: 12,
        slot: the_cut_cube::SLOT,
        thumb: the_cut_cube::thumb,
    },
    Filed {
        chapter: 13,
        slot: four_triangles::SLOT,
        thumb: four_triangles::thumb,
    },
    Filed {
        chapter: 14,
        slot: the_missing_square::SLOT,
        thumb: the_missing_square::thumb,
    },
];

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
