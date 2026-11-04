pub mod thumbnail;
mod opening;
mod group;
mod subgroup;
mod centralizer;
mod cosets;
mod quotient;
mod applications;
mod parity;
mod closing;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &[
    "What makes these moves a group?",
    "Closure: combine moves, get a move",
    "Identity: a do-nothing move",
    "Inverses: every move can be undone",
    "Associativity: regroup without reordering",
    "Turn, then flip", "Flip, then turn",
    "Do the orders land in the same arrangement?",
    "Different endpoints: order can matter", "01234",
    "Motion paths are schematic",
    "Keep only the rotations How many quarter turns return to the start? order of r group order e → r → r² → r³ → e 0123456789",
    "Which moves commute with the horizontal flip? Predict: does a quarter turn commute? identity quarter turn half turn three-quarter turn horizontal flip diagonal flip A vertical flip diagonal flip B same result different result moves commute with the chosen flip",
    "Which positions match? On the same four-step clock? All land at Reflexive · symmetric · transitive A subgroup partitions the square moves H = {e, s}; classes are left cosets gH classes of moves A symmetric mark can look unchanged Same image; different moves",
    "Can we multiply these classes? Choose e or s from the same H; choose r from rH. Will the resulting class depend on the choice? ∈ · Ignore rotations, keep reflection Rotations R Reflections F then combine Normality keeps every representative choice consistent",
    "How many ways can a square fit itself? Turn, then flip · motion paths schematic rotations reflections How do these moves combine?",
    "Ignore rotation, keep reflection original quarter turn mirror image same rotation class different class The same language, different groups Molecular symmetries Error-detecting codes Engineering transformations A simple parity check Valid: an even number of ones Send 101: 2 ones Receive 100: 1 one Odd count: error detected Two flips can go unnoticed Each application has its own assumptions",
    "Which differences matter? Group combine the moves Centralizer find moves that trade order Equivalence class choose what counts as the same Normal subgroup combine classes without ambiguity",
];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "01_square", make: |kit| Box::new(opening::Opening { kit }), intensity: 0.4 },
        Chapter { key: "02_group", make: |kit| Box::new(group::Group { kit }), intensity: 0.4 },
        Chapter { key: "03_rotation_subgroup", make: |kit| Box::new(subgroup::RotationSubgroup { kit }), intensity: 0.4 },
        Chapter { key: "04_centralizer", make: |kit| Box::new(centralizer::Centralizer { kit }), intensity: 0.4 },
        Chapter { key: "05_equivalence_cosets", make: |kit| Box::new(cosets::Cosets { kit }), intensity: 0.4 },
        Chapter { key: "06_ambiguous_cosets", make: |kit| Box::new(quotient::AmbiguousCosets { kit }), intensity: 0.4 },
        Chapter { key: "07_rotation_quotient", make: |kit| Box::new(quotient::RotationQuotient { kit }), intensity: 0.4 },
        Chapter { key: "08_applications", make: |kit| Box::new(applications::Applications { kit }), intensity: 0.4 },
        Chapter { key: "09_parity", make: |kit| Box::new(parity::Parity { kit }), intensity: 0.4 },
        Chapter { key: "10_closing", make: |kit| Box::new(closing::Closing { kit }), intensity: 0.4 },
    ]
}
