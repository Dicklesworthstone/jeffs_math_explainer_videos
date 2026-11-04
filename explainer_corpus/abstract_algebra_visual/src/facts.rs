//! Values for the native square-symmetry prototype.
use explainer_kit::Facts;
use crate::model::{all, left_coset, Symmetry};
pub const CLOCK_MODULUS: i32 = 4;
pub const CLOCK_FIRST: i32 = 1;

pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    let group = all();
    let s = Symmetry::S;
    facts.add("square.group_order", group.len())?;
    facts.add("square.centralizer_order", group.iter().filter(|x| x.compose(s) == s.compose(**x)).count())?;
    let rotations = [Symmetry::ID, Symmetry::R, Symmetry::new(2, false), Symmetry::new(3, false)];
    facts.add("square.reflection_coset_order", left_coset(s, &rotations).len())?;
    facts.add("square.rotation_order", rotations.len())?;
    let subgroup = [Symmetry::ID, s];
    let mut cosets: Vec<Vec<Symmetry>> = Vec::new();
    for action in group {
        if !cosets.iter().any(|coset| coset.contains(&action)) {
            cosets.push(left_coset(action, &subgroup));
        }
    }
    if group.iter().any(|action| cosets.iter().filter(|coset| coset.contains(action)).count() != 1)
        || cosets.iter().any(|coset| coset.len() != subgroup.len())
    { return Err("invalid coset partition".into()); }
    facts.add("square.coset_count", cosets.len())?;
    facts.add("square.coset_size", subgroup.len())?;
    facts.add("clock.modulus", CLOCK_MODULUS)?;
    for i in 0..3 {
        facts.add(&format!("clock.value.{}", i + 1), CLOCK_FIRST + i * CLOCK_MODULUS)?;
    }
    facts.add("clock.position", CLOCK_FIRST.rem_euclid(CLOCK_MODULUS))?;
    let quotient = [left_coset(Symmetry::ID, &rotations), left_coset(s, &rotations)];
    for (row, left) in quotient.iter().enumerate() {
        for (column, right) in quotient.iter().enumerate() {
            let reflected = left[0].compose(right[0]).reflected;
            if left.iter().any(|a| right.iter().any(|b| a.compose(*b).reflected != reflected)) {
                return Err("quotient product depends on representative".into());
            }
            facts.add(&format!("square.quotient.{row}{column}"), if reflected { "F" } else { "R" })?;
        }
    }
    for (name, action) in [("turn_then_flip", s.compose(Symmetry::R)), ("flip_then_turn", Symmetry::R.compose(s))] {
        for (index, point) in [(-1,-1), (1,-1), (1,1), (-1,1)].into_iter().enumerate() {
            let (x, y) = action.apply(point);
            facts.add(&format!("square.{name}.{}", index + 1), format!("{x},{y}"))?;
        }
    }
    facts.self_check()?;
    Ok(facts)
}
