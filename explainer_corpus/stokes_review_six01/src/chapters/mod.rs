//! Six native scenes for provisional narration review.
mod opening;
mod edges;
mod unit_square;
mod cap;
mod volume;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &["What survives when boundaries add? Shared edge: opposite directions Interior seam cancels Only the outside boundary remains", "An interval has a signed boundary Boundary [a,b] = b − a Interior endpoints cancel Change over the interval Integral of f prime = f(b) − f(a)", "Oriented edges add Shared edge contributes twice with opposite sign Boundary contributions: work along a step Reverse the step, reverse its contribution A one-form assigns signed values to oriented steps", "A one-form on the unit square ω = x dy Which edge contributes? horizontal: dy = 0 left: x = 0 right: x = 1, Δy = 1 ∮ ω = 1 dω = dx ∧ dy: unit area = 1 Reverse orientation boundary integral = −1 oriented area integral = −1", "One rim, two surfaces flat disk cap: z = 1 − x² − y² upward normal F = (−y/2, x/2, 0) curl F = (0,0,1) rim circulation = π curl flux = π same oriented boundary", "From volume to surface Shared face: opposite outward directions Internal flux cancels; outer surface remains F = (x,y,z)/3 div F = 1 volume integral = 4π/3 on unit sphere: F · n = 1/3 surface flux = (1/3)(4π) = 4π/3"];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "01_disappearing_seams", make: |kit| Box::new(opening::Seams { kit }), intensity: 0.4 },
        Chapter { key: "02_interval", make: |kit| Box::new(opening::Interval { kit }), intensity: 0.4 },
        Chapter { key: "03_edges", make: |kit| Box::new(edges::Edges { kit }), intensity: 0.4 },
        Chapter { key: "04_unit_square", make: |kit| Box::new(unit_square::UnitSquare { kit }), intensity: 0.4 },
        Chapter { key: "05_curved_surface", make: |kit| Box::new(cap::Cap { kit }), intensity: 0.4 },
        Chapter { key: "06_volume", make: |kit| Box::new(volume::Volume { kit }), intensity: 0.4 },
    ]
}
