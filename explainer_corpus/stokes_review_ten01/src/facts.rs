//! Author calculations for the first two scenes, not independent review.
use explainer_kit::Facts;
pub fn facts() -> Result<Facts,String> {
    let mut f = Facts::new();
    f.add("two_squares.directed_edges", 8)?;
    f.add("two_squares.canceled_edges", 2)?;
    f.add("two_squares.outer_edges", 6)?;
    f.add("interval.internal_endpoint_pairs", 2)?;
    f.self_check()?;
    Ok(f)
}
