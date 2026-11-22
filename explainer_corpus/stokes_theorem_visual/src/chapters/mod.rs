//! Twenty-three native scenes; CC3 scratch draft adds 08 to 12 and 19 to 21 and renumbers the rest. Uncompiled; requires full review.
mod opening;
pub mod thumbnail;
mod edges;
mod unit_square;
mod cap;
mod volume;
mod annulus;
mod steps;
mod areas;
mod cube;
mod cells;
mod dimensions;
mod missing;
mod wedges;
mod box_proof;
mod charts;
mod assemble;
mod faraday;
mod return_scene;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &["What survives when boundaries add? Shared edge: opposite directions Interior seam cancels Only the outside boundary remains", "An interval has a signed boundary Boundary [a,b] = b − a Interior endpoints cancel Change over the interval Integral of f prime = f(b) − f(a)", "Oriented edges add Shared edge contributes twice with opposite sign Boundary contributions: work along a step Reverse the step, reverse its contribution A one-form assigns signed values to oriented steps", "A one-form on the unit square ω = x dy Which edge contributes? horizontal: dy = 0 left: x = 0 right: x = 1, Δy = 1 ∮ ω = 1 dω = dx ∧ dy: unit area = 1 Reverse orientation boundary integral = −1 oriented area integral = −1", "One rim, two surfaces flat disk cap: z = 1 − x² − y² upward normal F = (−y/2, x/2, 0) curl F = (0,0,1) rim circulation = π curl flux = π same oriented boundary", "From volume to surface Shared face: opposite outward directions Internal flux cancels; outer surface remains F = (x,y,z)/3 div F = 1 volume integral = 4π/3 on unit sphere: F · n = 1/3 surface flux = (1/3)(4π) = 4π/3", "A boundary you might forget origin excluded Is the outer edge the whole boundary? Which way does each edge go? outer: counterclockwise inner: clockwise ω = (−y dx + x dy)/(x² + y²) smooth here; dω = 0 outer: 2π + inner: −2π = 0 Cannot fill the hole: ω undefined at 0", "What a one-form does to one step ω = x dy value on a step = x × (change in y) at (2, 1), step (1, 1/2): 2 × 1/2 = 1 reversed step?   doubled step? reversed step: −1 doubled step: 2 sideways step (1, 0): 0 same step where x = 3: 1.5 linear in the step; changes from point to point x = 1 x = 2 x = 3 (2, 1)", "Integrating by adding steps add x dy over each step, taken at its midpoint 4 steps:  2.00 8 steps:  2.83 16 steps:  3.06 64 steps:  3.14 each sum = area of the polygon walked → π = ∫ over the disk of dx ∧ dy", "What a two-form does to two steps u = (2, 0) v = (1, 1.5) +3 2 × 1.5 − 0 × 1 = 3 order v, u: −3 1.5 halfway: 1.5 0 along u: 0 in three dimensions the shadow on the x z plane both factors read only x: a shadow on a line a repeated differential gives zero", "The cancellation, worked on a cube x y z on a face with y fixed: flow in the y direction Which parts survive the wedge? dx twice dz twice dx to the front: two swaps, sign unchanged", "Both sides, on the cube inside: ∫ 2y over the cube = 1 faces with x or z fixed: dz ∧ dx = 0 y = 1: g = 1 + xz y = 0: g = xz y = 1, outward +y: ∫∫ (1 + xz) dx dz = 5/4 y = 0, outward −y: −∫∫ xz dx dz = −1/4 5/4 − 1/4 = 1 the xz part is the same on both faces and cancels", "Dimension tells us what to integrate 0-form 1-form 2-form 3-form points oriented steps oriented area oriented volume k-form: alternating multilinear rule on k directions swap two directions → change sign d maps k-forms to (k+1)-forms Generalized Stokes ∫ on M dω = ∫ on ∂M ω M: compact, oriented, smooth n-manifold with boundary ω: smooth (n−1)-form; boundary has induced orientation Proof: oriented box → curved manifold", "One term for each missing direction Three dimensions: choose an omitted coordinate omit x: f₁ dy ∧ dz omit y: −f₂ dx ∧ dz omit z: f₃ dx ∧ dy Signs encode the face orientations −f₂ dx ∧ dz = +f₂ dz ∧ dx Swap directions → reverse the sign 1st: dx 2nd: dz 2nd: dx 1st: dz ω = Σ_i (−1)^(i−1) f_i dx^1 ∧ ··· ∧ omit(dx^i) ∧ ··· ∧ dx^n df_i = Σ_j (∂_j f_i) dx^j Which terms survive the wedge?", "Why only one derivative survives Example term: −f₂ dx ∧ dz dx ∧ dx = 0 dz ∧ dz = 0 Only dy is missing; ∂_yf₂ dy survives −∂_yf₂ dy ∧ dx ∧ dz = +∂_yf₂ dx ∧ dy ∧ dz One swap cancels the earlier minus dω = (Σ_i ∂_if_i) dx^1 ∧ ··· ∧ dx^n In 3D the coefficient is divergence Next: integrate one coordinate at a time", "One coordinate at a time a_i b_i i-th coordinate → ∫ from a_i to b_i ∂_if_i dx^i = f_i(b_i) − f_i(a_i) Integrate over the other coordinates Smooth on a bounded box: use Fubini Upper face +; lower face − Σ face pairs = ∫ on ∂box ω FTC + Fubini + induced orientations", "From a box to a curved manifold interior chart: space boundary chart: half-space orientation-preserving coordinates smooth weights: supported in charts rho_1 + rho_2 + ··· = 1 local piece: rho_i ω interior: compact support boundary: endpoint on half-space outward-first orientation M lies above blue boundary green chart: box part inside M", "Local pieces fit back together example: weights add to 1 rho_1 rho_2 rho_1 ω + rho_2 ω + ··· = ω d(rho_i ω) = d(rho_i) ∧ ω + rho_i dω Σ d(rho_i) = d(Σ rho_i) = d(1) = 0 Extra weight-derivative terms cancel Pullback commutes with d Oriented change of coordinates Smooth partitions of unity exist These are ingredients, not proved by the picture", "Crossing an edge, or running along it spreading: (x/2, y/2) turning: (−y/2, x/2) crosses the edge?   runs along it? 1/4 crossing: 4 × 1/4 = 1    along: 0 crossing: 0    along: 4 × 1/4 = 1 net outflow per area = divergence circulation per area = curl divergence: crossing part     curl: along part", "Finite volumes: one number per cell each cell: one number, the amount inside each shared face: one flux, moved across it 4 2 1 flux = 0.1 × (difference across the face) outer boundary closed total now: 8 total after one step? 0.2 0.3 0.1 bottom pair equal: 0 3.5 2.1 1.3 1.1 total after one step: 8 each flux: + on one side, − on the other", "One open face 0.25 in open face: 0.25 flows in 2.35 total: 8 + 0.25 = 8.25 shared faces cancel, as the seams did exact up to rounding, for any flux formula assumed: cells fixed one flux per shared face, used with both signs nothing created inside a cell accuracy: flux formula and time step factor 0.1: stable here large square grid: unstable above 1/4 next: circulation along edges", "A fixed loop in a changing field loop and surface fixed B increases upward E circulates clockwise ∮ E · dl = − d/dt ∫ B · n dA minus sign comes from Faraday's law ∫ curl E · n dA = −∫ ∂B/∂t · n dA curl E = −∂B/∂t moving loops need extra care", "What the boundary actually tells us ∫ on M dω = ∫ on ∂M ω Different interior changes can cancel + − 0 signed endpoints cancel at every corner boundary of boundary = 0 d(dω) = 0 These identities alone do not prove Stokes line: endpoint change surface: circulation and curl volume: flux and divergence local endpoint subtraction; global assembly"];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "01_disappearing_seams", make: |kit| Box::new(opening::Seams { kit }), intensity: 0.4 },
        Chapter { key: "02_interval", make: |kit| Box::new(opening::Interval { kit }), intensity: 0.4 },
        Chapter { key: "03_edges", make: |kit| Box::new(edges::Edges { kit }), intensity: 0.4 },
        Chapter { key: "04_unit_square", make: |kit| Box::new(unit_square::UnitSquare { kit }), intensity: 0.4 },
        Chapter { key: "05_curved_surface", make: |kit| Box::new(cap::Cap { kit }), intensity: 0.4 },
        Chapter { key: "06_volume", make: |kit| Box::new(volume::Volume { kit }), intensity: 0.4 },
        Chapter { key: "07_annulus", make: |kit| Box::new(annulus::Annulus { kit }), intensity: 0.4 },
        Chapter { key: "08_signed_steps", make: |kit| Box::new(steps::SignedSteps { kit }), intensity: 0.4 },
        Chapter { key: "09_step_sums", make: |kit| Box::new(steps::StepSums { kit }), intensity: 0.4 },
        Chapter { key: "10_signed_areas", make: |kit| Box::new(areas::SignedAreas { kit }), intensity: 0.4 },
        Chapter { key: "11_cube_derivative", make: |kit| Box::new(cube::CubeDerivative { kit }), intensity: 0.4 },
        Chapter { key: "12_cube_check", make: |kit| Box::new(cube::CubeCheck { kit }), intensity: 0.4 },
        Chapter { key: "13_dimensions", make: |kit| Box::new(dimensions::Dimensions { kit }), intensity: 0.4 },
        Chapter { key: "14_missing_coordinate", make: |kit| Box::new(missing::Missing { kit }), intensity: 0.4 },
        Chapter { key: "15_wedge_cancellation", make: |kit| Box::new(wedges::Wedges { kit }), intensity: 0.4 },
        Chapter { key: "16_box_proof", make: |kit| Box::new(box_proof::BoxProof { kit }), intensity: 0.4 },
        Chapter { key: "17_charts", make: |kit| Box::new(charts::Charts { kit }), intensity: 0.4 },
        Chapter { key: "18_assemble", make: |kit| Box::new(assemble::Assemble { kit }), intensity: 0.4 },
        Chapter { key: "19_one_cell", make: |kit| Box::new(cells::OneCell { kit }), intensity: 0.4 },
        Chapter { key: "20_finite_volumes", make: |kit| Box::new(cells::FiniteVolumes { kit }), intensity: 0.4 },
        Chapter { key: "21_open_boundary", make: |kit| Box::new(cells::OpenBoundary { kit }), intensity: 0.4 },
        Chapter { key: "22_faraday", make: |kit| Box::new(faraday::Faraday { kit }), intensity: 0.4 },
        Chapter { key: "23_return", make: |kit| Box::new(return_scene::Return { kit }), intensity: 0.4 },
    ]
}
