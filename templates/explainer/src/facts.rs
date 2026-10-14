//! Every number the video shows or says, computed from [`crate::model`] and
//! pinned against [`oracle.py`](../oracle.py).

use explainer_kit::Facts;

pub use crate::model::{PICK, POINTS};

pub fn quadruples() -> u128 {
    crate::model::quadruples(POINTS, PICK)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let total = quadruples();
    let trillions = crate::model::trillions_floor(total);
    f.pin("points", POINTS, "5000", "demo parameter N = 5,000")?;
    f.pin("pick", PICK, "4", "demo parameter k = 4")?;
    f.pin(
        "quadruples_5000",
        total,
        "26010428123750",
        "C(5000, 4) = 5000 * 4999 * 4998 * 4997 / 24",
    )?;
    f.pin(
        "quadruples_trillions_floor",
        trillions,
        "26",
        "floor(C(5000, 4) / 10^12)",
    )?;
    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_self_check_passes() {
        let f = facts().expect("facts self_check");
        assert_eq!(f.len(), 4);
    }
}
