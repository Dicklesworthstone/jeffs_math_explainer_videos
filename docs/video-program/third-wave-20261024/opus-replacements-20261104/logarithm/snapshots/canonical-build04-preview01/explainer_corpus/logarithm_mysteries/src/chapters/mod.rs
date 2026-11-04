//! The logarithm film's thirteen chapters, in spoken order (script.tsv).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`: replaces the
//! canonical seven-chapter draft (00_opening … 06_closing) whole; the old
//! chapter files are moved aside by root at integration, not edited.
//! Uncompiled here; root runs Cargo.
mod branch_cut;
mod building_ruler;
mod complex_turns;
mod doing_nothing;
mod draw;
mod folded_decade;
mod lifting_loops;
mod matrix_logs;
mod product_rule;
mod ratio_ruler;
mod slide_rule;
pub mod thumbnail;
mod undoing_growth;
mod unreachable;
mod what_it_forgets;
use explainer_kit::Chapter;

/// Glyph supplement: the probe captures every string the chapters build
/// through the kit; this line adds the characters that only appear in
/// computed strings (numbers from facts, live labels), so the probe page
/// shows them even before every branch of a scene has run.
pub const TEXT: &[&str] = &[
    "0123456789 . , : ; ( ) [ ] = + − × ÷ ± ½ ≈ ≠ ≤ < > ∞ ∫ √ π θ ° ₂ → ↔ – … ? ! Hz % a b c d e i k t x y z A I J X",
];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "00_ratio_ruler",
            make: |kit| Box::new(ratio_ruler::RatioRuler { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "01_building_ruler",
            make: |kit| Box::new(building_ruler::BuildingRuler { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_slide_rule",
            make: |kit| Box::new(slide_rule::SlideRule { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_folded_decade",
            make: |kit| Box::new(folded_decade::FoldedDecade { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "04_undoing_growth",
            make: |kit| Box::new(undoing_growth::UndoingGrowth { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "05_complex_turns",
            make: |kit| Box::new(complex_turns::ComplexTurns { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "06_lifting_loops",
            make: |kit| Box::new(lifting_loops::LiftingLoops { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "07_branch_cut",
            make: |kit| Box::new(branch_cut::BranchCut { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_product_rule",
            make: |kit| Box::new(product_rule::ProductRule { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_matrix_logs",
            make: |kit| Box::new(matrix_logs::MatrixLogs { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "10_doing_nothing",
            make: |kit| Box::new(doing_nothing::DoingNothing { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "11_unreachable",
            make: |kit| Box::new(unreachable::Unreachable { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "12_what_it_forgets",
            make: |kit| Box::new(what_it_forgets::WhatItForgets { kit }),
            intensity: 0.3,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_is_in_spoken_order_and_covers_every_script_prefix() {
        let keys: Vec<&str> = registry().iter().map(|c| c.key).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted);
        let script = include_str!("../../script.tsv");
        for line in script.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
            let prefix = &line[..2];
            assert!(keys.iter().any(|k| k.starts_with(prefix)), "{line}");
        }
        for k in &keys {
            assert!(script.lines().any(|l| l.starts_with(&k[..2])), "{k}");
        }
    }
}
