pub mod thumbnail;
mod applications;
mod closing;
mod definitions;
mod derivative;
mod property;
mod rotation;
mod slide_rule;

use explainer_kit::Chapter;

pub const TEXT: &[&str] = &[
    "How can sliding multiply? D C What is beneath C's 3? 0123456789 × = Sliding adds distances log10(2) log10(3) log10(6) + Adding lengths multiplies the marked numbers",
    "One function, many appearances", "Growth", "Decay", "Circuits", "Heat", "Rotation", "One underlying idea",
    "Add inputs, multiply outputs f(x + y) f(x) f(y)",
    "The natural slope f′(x) = f(x) exp(x)",
    "A limit, a differential equation, a power series lim n→∞ (1 + x/n)^n; x = 1 k = 0, 1, 2, ...",
    "Where exponentials appear growth decay heat circuits finance One possible limit: capacity constant proportional rate capacity-limited example amount time →",
    "What happens when the exponent turns? e^(it) = cos t + i sin t",
    "Exponential: add inputs, multiply outputs x exp → exp(x) > 0 ← log One point, many turns e⁰ = e^(2πi) = 1 The logarithm has more to reveal",
];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "00_slide_rule", make: |kit| Box::new(slide_rule::SlideRule { kit }), intensity: 0.4 },
        Chapter { key: "01_addition_to_multiplication", make: |kit| Box::new(property::AdditionToMultiplication { kit }), intensity: 0.4 },
        Chapter { key: "02_natural_slope", make: |kit| Box::new(derivative::NaturalSlope { kit }), intensity: 0.4 },
        Chapter { key: "03_three_definitions", make: |kit| Box::new(definitions::ThreeDefinitions { kit }), intensity: 0.4 },
        Chapter { key: "04_applications", make: |kit| Box::new(applications::Applications { kit }), intensity: 0.4 },
        Chapter { key: "05_imaginary_rotation", make: |kit| Box::new(rotation::ImaginaryRotation { kit }), intensity: 0.4 },
        Chapter { key: "06_logarithm_bridge", make: |kit| Box::new(closing::Closing { kit }), intensity: 0.3 },
    ]
}
