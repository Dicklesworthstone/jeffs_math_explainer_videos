//! Eight native scenes for provisional narration review.
mod opening;
mod edges;
mod unit_square;
mod cap;
mod volume;
mod annulus;
mod dimensions;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &["What survives when boundaries add? Shared edge: opposite directions Interior seam cancels Only the outside boundary remains", "An interval has a signed boundary Boundary [a,b] = b − a Interior endpoints cancel Change over the interval Integral of f prime = f(b) − f(a)", "Oriented edges add Shared edge contributes twice with opposite sign Boundary contributions: work along a step Reverse the step, reverse its contribution A one-form assigns signed values to oriented steps", "A one-form on the unit square ω = x dy Which edge contributes? horizontal: dy = 0 left: x = 0 right: x = 1, Δy = 1 ∮ ω = 1 dω = dx ∧ dy: unit area = 1 Reverse orientation boundary integral = −1 oriented area integral = −1", "One rim, two surfaces flat disk cap: z = 1 − x² − y² upward normal F = (−y/2, x/2, 0) curl F = (0,0,1) rim circulation = π curl flux = π same oriented boundary", "From volume to surface Shared face: opposite outward directions Internal flux cancels; outer surface remains F = (x,y,z)/3 div F = 1 volume integral = 4π/3 on unit sphere: F · n = 1/3 surface flux = (1/3)(4π) = 4π/3", "A boundary you might forget origin excluded Is the outer edge the whole boundary? Which way does each edge go? outer: counterclockwise inner: clockwise ω = (−y dx + x dy)/(x² + y²) smooth here; dω = 0 outer: 2π + inner: −2π = 0 Cannot fill the hole: ω undefined at 0", "Dimension tells us what to integrate 0-form 1-form 2-form 3-form points oriented steps oriented area oriented volume k-form: alternating multilinear rule on k directions swap two directions → change sign d maps k-forms to (k+1)-forms Generalized Stokes ∫ on M dω = ∫ on ∂M ω M: compact, oriented, smooth n-manifold with boundary ω: smooth (n−1)-form; boundary has induced orientation Proof: oriented box → curved manifold"];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "01_disappearing_seams", make: |kit| Box::new(opening::Seams { kit }), intensity: 0.4 },
        Chapter { key: "02_interval", make: |kit| Box::new(opening::Interval { kit }), intensity: 0.4 },
        Chapter { key: "03_edges", make: |kit| Box::new(edges::Edges { kit }), intensity: 0.4 },
        Chapter { key: "04_unit_square", make: |kit| Box::new(unit_square::UnitSquare { kit }), intensity: 0.4 },
        Chapter { key: "05_curved_surface", make: |kit| Box::new(cap::Cap { kit }), intensity: 0.4 },
        Chapter { key: "06_volume", make: |kit| Box::new(volume::Volume { kit }), intensity: 0.4 },
        Chapter { key: "07_annulus", make: |kit| Box::new(annulus::Annulus { kit }), intensity: 0.4 },
        Chapter { key: "08_dimensions", make: |kit| Box::new(dimensions::Dimensions { kit }), intensity: 0.4 },
    ]
}
