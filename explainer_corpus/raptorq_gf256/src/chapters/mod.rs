//! One scene per chapter, in storyboard order, with the score's intensity
//! arc (a bright hook, the algebra quieter, the payoff fullest at 15 and 17).
//! Each chapter starts as its storyboard's animatic and is replaced by its
//! real scene as it is built.

// Video B's look, shared by every chapter: its helpers land ahead of the
// chapters that use them (lanes B-1, B-2, B-3); drop the allow when all 17
// chapters are built.
#[allow(dead_code)]
pub mod draw;

// Chapters 01–06 (lane B-1) and the props later chapters call back to: the
// bucket and its danger meter, the risk ladder, the ring of five, the grid.
pub mod bits;
pub mod cards;
pub mod cube;
pub mod hook;
pub mod lottery;
pub mod picture;
#[allow(dead_code)]
pub mod props;
pub mod spares;

// Chapters 07–12 (lane B-2) and their props: the byte register with its
// gutter, the n-hour clock, the byte-chord sonifications (fold.rs also
// holds the code panel and the math lower-third 11 reuses).
pub mod byte_clock;
pub mod clock;
pub mod fold;
pub mod polynomials;
pub mod prime_clocks;
pub mod register;
pub mod sound;
pub mod times_table;
pub mod wrap256;

// Chapters 13–17 (lane B-3): the toy graph and its peel (14, reused by 16),
// and pictures that land on the voice's sentences.
pub mod byte_lottery;
pub mod density;
pub mod parking;
pub mod voice;

use explainer_kit::Chapter;
use explainer_kit::animatic::Animatic;
use explainer_kit::kit::CameraShot;

use crate::STORYBOARD;

macro_rules! animatic {
    ($key:literal, $intensity:expr) => {
        Chapter {
            key: $key,
            make: |kit| Box::new(Animatic::new(kit, STORYBOARD, $key)),
            intensity: $intensity,
        }
    };
}

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_hook",
            make: |kit| Box::new(hook::Hook { kit }),
            intensity: 0.35,
        },
        Chapter {
            key: "02_bits_and_ex_or",
            make: |kit| Box::new(bits::Bits { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_cards",
            make: |kit| Box::new(cards::Cards { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "04_the_cube",
            make: |kit| {
                // the cube runs on the camera route (spike S-5), starting
                // straight down, as the retained route sees it
                kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                Box::new(cube::Cube { kit })
            },
            intensity: 0.55,
        },
        Chapter {
            key: "05_the_ex_or_lottery",
            make: |kit| Box::new(lottery::Lottery { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_spares_and_rings",
            make: |kit| Box::new(spares::Spares { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "07_why_not_wrap_around_at_256",
            make: |kit| Box::new(wrap256::Wrap256 { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "08_prime_clocks",
            make: |kit| Box::new(prime_clocks::PrimeClocks { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "09_bytes_are_polynomials",
            make: |kit| Box::new(polynomials::Polynomials { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "10_the_fold",
            make: |kit| Box::new(fold::Fold { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "11_the_byte_clock",
            make: |kit| Box::new(byte_clock::TheByteClock { kit }),
            intensity: 0.6,
        },
        Chapter {
            key: "12_the_times_table",
            make: |kit| Box::new(times_table::TimesTable { kit }),
            intensity: 0.65,
        },
        Chapter {
            key: "13_the_byte_lottery",
            make: |kit| Box::new(byte_lottery::ByteLottery { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "14_the_price_of_density",
            make: |kit| Box::new(density::Density { kit }),
            intensity: 0.5,
        },
        animatic!("15_where_the_danger_lives", 0.65),
        Chapter {
            key: "16_parking_a_piece",
            make: |kit| Box::new(parking::Parking { kit }),
            intensity: 0.55,
        },
        animatic!("17_two_packets", 0.6),
    ]
}
