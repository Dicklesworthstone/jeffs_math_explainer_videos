//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (fullest where the cubes close into their square, where the three
//! stacks become a box and where the four triangles slide; quietest for the
//! pattern that breaks and for the picture that lies). While several
//! builders write scenes at once, each chapter sits behind a switch and
//! renders as its storyboard's animatic when the switch is off.

#[cfg(feature = "ch11")]
pub mod a_third;
#[cfg(feature = "ch01")]
pub mod cold_open;
#[cfg(any(
    feature = "ch02",
    feature = "ch03",
    feature = "ch04",
    feature = "ch05",
    feature = "ch06",
    feature = "ch07",
    feature = "ch08",
    feature = "ch09",
    feature = "ch10",
    feature = "ch11",
    feature = "ch12",
    feature = "ch13",
    feature = "ch14",
    feature = "ch15"
))]
pub mod draw;
#[cfg(feature = "ch06")]
pub mod every_cube;
#[cfg(feature = "ch09")]
pub mod every_height;
#[cfg(feature = "ch02")]
pub mod five_for_five;
#[cfg(feature = "ch13")]
pub mod four_triangles;
pub mod glyphs;
#[cfg(feature = "ch10")]
pub mod halves;
#[cfg(any(
    feature = "ch01",
    feature = "ch05",
    feature = "ch06",
    feature = "ch07",
    feature = "ch08",
    feature = "ch09",
    feature = "ch12",
    feature = "ch15"
))]
pub mod iso;
#[cfg(feature = "ch03")]
pub mod layers;
#[cfg(feature = "ch05")]
pub mod slabs;
#[cfg(feature = "ch04")]
pub mod staircases;
#[cfg(feature = "ch08")]
pub mod the_box;
#[cfg(feature = "ch12")]
pub mod the_cut_cube;
#[cfg(feature = "ch14")]
pub mod the_missing_square;
#[cfg(feature = "ch07")]
pub mod three_stacks;
#[cfg(feature = "ch15")]
pub mod what_makes_a_proof;

use explainer_kit::Chapter;
use explainer_kit::animatic::Animatic;

use crate::STORYBOARD;

/// A chapter's real scene when its switch is on, its animatic when not.
macro_rules! scene {
    ($feature:literal, $key:literal, $intensity:expr, $module:ident :: $scene:ident) => {{
        #[cfg(feature = $feature)]
        let chapter = Chapter {
            key: $key,
            make: |kit| Box::new($module::$scene { kit }),
            intensity: $intensity,
        };
        #[cfg(not(feature = $feature))]
        let chapter = Chapter {
            key: $key,
            make: |kit| Box::new(Animatic::new(kit, STORYBOARD, $key)),
            intensity: $intensity,
        };
        chapter
    }};
}

pub fn registry() -> Vec<Chapter> {
    vec![
        scene!("ch01", "01_cold_open", 0.5, cold_open::ColdOpen),
        scene!("ch02", "02_five_for_five", 0.3, five_for_five::FiveForFive),
        scene!("ch03", "03_layers", 0.4, layers::Layers),
        scene!("ch04", "04_staircases", 0.4, staircases::Staircases),
        scene!("ch05", "05_slabs", 0.6, slabs::Slabs),
        scene!("ch06", "06_every_cube", 0.55, every_cube::EveryCube),
        scene!("ch07", "07_three_stacks", 0.55, three_stacks::ThreeStacks),
        scene!("ch08", "08_the_box", 0.6, the_box::TheBox),
        scene!("ch09", "09_every_height", 0.5, every_height::EveryHeight),
        scene!("ch10", "10_halves", 0.4, halves::Halves),
        scene!("ch11", "11_a_third", 0.5, a_third::AThird),
        scene!("ch12", "12_the_cut_cube", 0.55, the_cut_cube::TheCutCube),
        scene!("ch13", "13_four_triangles", 0.6, four_triangles::FourTriangles),
        scene!("ch14", "14_the_missing_square", 0.3, the_missing_square::TheMissingSquare),
        scene!("ch15", "15_what_makes_a_proof", 0.45, what_makes_a_proof::WhatMakesAProof),
    ]
}
