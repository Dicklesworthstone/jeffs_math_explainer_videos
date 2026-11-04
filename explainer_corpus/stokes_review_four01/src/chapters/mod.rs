//! Four native scenes implemented; chapters 05–15 remain to be built.
mod opening;
mod edges;
mod unit_square;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &["What survives when boundaries add? Shared edge: opposite directions Interior seam cancels Only the outside boundary remains", "An interval has a signed boundary Boundary [a,b] = b − a Interior endpoints cancel Change over the interval Integral of f prime = f(b) − f(a)", "Oriented edges add Shared edge contributes twice with opposite sign Boundary contributions: work along a step Reverse the step, reverse its contribution A one-form assigns signed values to oriented steps", "A one-form on the unit square ω = x dy Which edge contributes? horizontal: dy = 0 left: x = 0 right: x = 1, Δy = 1 ∮ ω = 1 dω = dx ∧ dy: unit area = 1 Reverse orientation boundary integral = −1 oriented area integral = −1"];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "01_disappearing_seams", make: |kit| Box::new(opening::Seams { kit }), intensity: 0.4 },
        Chapter { key: "02_interval", make: |kit| Box::new(opening::Interval { kit }), intensity: 0.4 },
        Chapter { key: "03_edges", make: |kit| Box::new(edges::Edges { kit }), intensity: 0.4 },
        Chapter { key: "04_unit_square", make: |kit| Box::new(unit_square::UnitSquare { kit }), intensity: 0.4 },
    ]
}
