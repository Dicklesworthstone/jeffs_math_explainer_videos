//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x resistor_random_walk
//! facts-check`): the rows of docs/fact-sheet.md. R-F rows are what the
//! opening shows and says, R-X rows the edges of its always/never
//! sentences, R-P rows the primary source's own worked examples.

use std::collections::BTreeSet;

use explainer_kit::Facts;

use crate::model::{self, BOTTOM, LEFT, Network, Q, RIGHT, ROUTE, TOP, csv};

/// How many steps of "still walking" the opening's exact version lists.
pub const STILL_STEPS: usize = 3;

/// Doyle and Snell's book, as the pins cite it.
const DOYLE_SNELL: &str = "Doyle and Snell, Random Walks and Electric Networks (arXiv math/0001057, version 3.02)";

/// The opening network.
pub fn network() -> Result<Network, String> {
    model::opening()
}

/// The opening network's voltages, the top at one volt and the bottom at
/// zero, junction by junction.
pub fn voltages() -> Result<Vec<Q>, String> {
    network()?.kirchhoff(Q::ONE, Q::ZERO)
}

/// The walker's chance, from every junction, of reaching the top first.
pub fn chances() -> Result<Vec<Q>, String> {
    Ok(network()?.walk()?.top)
}

/// `values` at the far end of each of junction `x`'s identical wires, in
/// the order the wires are drawn.
pub fn far_values(values: &[Q], x: usize) -> Result<Vec<Q>, String> {
    Ok(network()?.far_ends(x)?.into_iter().map(|y| values[y]).collect())
}

/// The plain average of `values` (each identical wire's far end once).
pub fn mean(values: &[Q]) -> Result<Q, String> {
    if values.is_empty() {
        return Err("the average of no values".into());
    }
    Ok(values.iter().fold(Q::ZERO, |s, &q| s + q) / Q::int(values.len() as i128))
}

/// The one number of identical wires every middle junction has.
pub fn wires_at_middle() -> Result<usize, String> {
    let net = network()?;
    let counts: BTreeSet<usize> = net
        .middle()
        .iter()
        .map(|&x| net.far_ends(x).map(|ends| ends.len()))
        .collect::<Result<_, _>>()?;
    match (counts.first(), counts.len()) {
        (Some(&k), 1) => Ok(k),
        _ => Err(format!("the middle junctions have {counts:?} wires, not one number")),
    }
}

/// The one number of identical wires in each doubled link.
pub fn side_by_side() -> Result<i128, String> {
    let doubled: BTreeSet<i128> = model::LINKS.iter().map(|l| l.2).filter(|&k| k > 1).collect();
    match (doubled.first(), doubled.len()) {
        (Some(&k), 1) => Ok(k),
        _ => Err(format!("the doubled links have {doubled:?} wires, not one number")),
    }
}

/// The chance that the walker from the left junction is still going after
/// each of its first [`STILL_STEPS`] steps.
pub fn still_walking() -> Result<Vec<Q>, String> {
    network()?.still_walking(LEFT, STILL_STEPS)
}

/// A junction's name, as the facts print it.
fn name(x: usize) -> &'static str {
    ["top", "left", "right", "bottom"].get(x).copied().unwrap_or("?")
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let net = network()?;
    let (v, walk) = (voltages()?, net.walk()?);

    // 01_1: the network
    f.add("R-F1.junctions", net.size())?;
    f.add("R-F1.middle", net.middle().len())?;
    f.add("R-F1.links", model::LINKS.len())?;
    f.add("R-F1.wires", model::LINKS.iter().map(|l| l.2).sum::<i128>())?;
    f.add("R-F1.doubled", model::LINKS.iter().filter(|l| l.2 > 1).count())?;
    f.add("R-F1.side_by_side", side_by_side()?)?;
    f.add("R-F1.wires_at_middle", wires_at_middle()?)?;
    f.add("R-F1.top_volts", v[net.top()])?;
    f.add("R-F1.bottom_volts", v[net.bottom()])?;

    // 01_2, 01_3: the electrical picture, each middle junction the average
    // of its wires' far ends
    f.pin(
        "R-F2.voltages",
        csv(&v),
        "1,3/5,2/5,0",
        "the two averages solved by hand: left = (1 + 1 + right + 0)/4, right = (1 + left + 0 + 0)/4",
    )?;
    f.add("R-F2.left", v[LEFT])?;
    f.add("R-F2.right", v[RIGHT])?;
    for x in [LEFT, RIGHT] {
        let ends = far_values(&v, x)?;
        let average = mean(&ends)?;
        // the plain average over identical wires is the conductance-weighted
        // one over links, in both pictures
        let weighted = [net.average(&v, x), net.average(&walk.top, x)];
        if average != v[x] || weighted != [Some(v[x]), Some(walk.top[x])] {
            return Err(format!(
                "the {} junction is {}, its far ends average {average}, weighted {weighted:?}",
                name(x),
                v[x]
            ));
        }
        f.add(&format!("R-F2.{}.far_ends", name(x)), csv(&ends))?;
        f.add(&format!("R-F2.{}.average", name(x)), average)?;
    }
    f.add("R-F2.currents", csv(&net.currents(&v)))?;
    let balanced = net.middle().iter().all(|&x| net.net_out(&v, x).is_zero());
    f.add("R-F2.balance", balanced)?;

    // 01_4 to 01_7: the walk, the same rule on the same wires
    f.add("R-F3.chances", csv(&walk.top))?;
    f.add("R-F3.left", walk.top[LEFT])?;
    f.add("R-F3.right", walk.top[RIGHT])?;
    f.add("R-F3.top_chance", walk.top[TOP])?;
    f.add("R-F3.bottom_chance", walk.top[BOTTOM])?;
    f.add("R-F3.wire_chance", Q::ONE / Q::int(net.far_ends(LEFT)?.len() as i128))?;
    let ends = far_values(&walk.top, LEFT)?;
    f.add("R-F3.left.far_ends", csv(&ends))?;
    f.add("R-F3.left.average", mean(&ends)?)?;
    let stops: Vec<Q> = net.middle().iter().map(|&x| walk.top[x] + walk.bottom[x]).collect();
    f.add("R-F3.stops", csv(&stops))?;
    f.add("R-F3.still_walking", csv(&still_walking()?))?;
    if !net.is_route(&ROUTE) {
        return Err(format!("the drawn route {ROUTE:?} is not a walk on the network"));
    }
    f.add("R-F3.route", ROUTE.map(name).join(","))?;
    f.add("R-F3.same", walk.top == v)?;

    // 01_7: exactly one solution, and its edge: two junctions joined only
    // to each other leave the balance singular and the walk endless
    let det = net.middle_determinant();
    f.add("R-X1.determinant", det)?;
    f.add("R-X1.solutions", if det.is_zero() { "many" } else { "1" })?;
    f.add("R-X1.zero_ends", csv(&net.kirchhoff(Q::ZERO, Q::ZERO)?))?;
    let island = model::with_island()?;
    let cut: Vec<String> = island.cut_off().iter().map(usize::to_string).collect();
    f.add("R-X2.island.cut_off", cut.join(","))?;
    f.add("R-X2.island.determinant", island.middle_determinant())?;
    f.add(
        "R-X2.island.refused",
        island.kirchhoff(Q::ONE, Q::ZERO).is_err() && island.walk().is_err(),
    )?;
    family(&mut f)?;

    // the primary source's own numbers
    let ds = model::doyle_snell()?;
    let dv = ds.kirchhoff(Q::ONE, Q::ZERO)?;
    f.pin(
        "R-P1.voltages",
        csv(&dv[2..]),
        "7/16,3/8",
        &format!("{DOYLE_SNELL}, §1.3.2: vc = 7/16, vd = 3/8"),
    )?;
    f.pin(
        "R-P1.chances",
        csv(&ds.walk()?.top[2..]),
        "7/16,3/8",
        &format!(
            "{DOYLE_SNELL}, §1.3.2: the voltage at c, 7/16, is also the probability from c of reaching a before b"
        ),
    )?;
    f.pin(
        "R-P1.current_cd",
        ds.currents(&dv)[4],
        "1/8",
        &format!("{DOYLE_SNELL}, §1.3.2: icd = (7/16 - 3/8) x 2 = 1/8"),
    )?;
    let steps: Vec<Q> = [0, 1, 3].iter().map(|&y| ds.between(2, y) / ds.total(2)).collect();
    f.pin(
        "R-P1.steps_from_c",
        csv(&steps),
        "1/4,1/4,1/2",
        &format!("{DOYLE_SNELL}, §1.3.1: row c of the walk's transition matrix, to a, b and d"),
    )?;
    f.pin(
        "R-P2.line5",
        csv(&model::line(5)?.kirchhoff(Q::ONE, Q::ZERO)?),
        "0,1/5,2/5,3/5,4/5,1",
        &format!("{DOYLE_SNELL}, §1.1.5: f(x) = x/N for N equal resistors in a line"),
    )?;
    f.self_check()?;
    Ok(f)
}

/// Every network of the opening's family (the six possible links each
/// absent, one wire or two): how many there are, how many leave no junction
/// cut off, how many do, how many have a singular balance, and among the
/// determined ones how many have the walk equal to the voltages and every
/// middle value between the ends'.
fn family(f: &mut Facts) -> Result<(), String> {
    let all = model::family()?;
    let (mut determined, mut cut_off, mut singular, mut agree, mut between) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for net in &all {
        if net.middle_determinant().is_zero() {
            singular += 1;
        }
        if !net.cut_off().is_empty() {
            cut_off += 1;
            continue;
        }
        determined += 1;
        let v = net.kirchhoff(Q::ONE, Q::ZERO)?;
        agree += usize::from(net.walk()?.top == v);
        between += usize::from(net.middle().iter().all(|&x| (Q::ZERO..=Q::ONE).contains(&v[x])));
    }
    f.add("R-X3.family.networks", all.len())?;
    f.add("R-X3.family.determined", determined)?;
    f.add("R-X3.family.cut_off", cut_off)?;
    f.add("R-X3.family.singular", singular)?;
    f.add("R-X3.family.agree", agree)?;
    f.add("R-X3.family.between_ends", between)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the oracle prints for these keys (kept in step with
    /// oracle.py; `./x resistor_random_walk facts-check` compares the two
    /// outputs in full).
    #[test]
    fn the_facts_print_what_the_oracle_prints() {
        let f = facts().unwrap();
        for (key, value) in [
            ("R-F1.junctions", "4"),
            ("R-F1.wires", "7"),
            ("R-F1.doubled", "2"),
            ("R-F1.side_by_side", "2"),
            ("R-F1.wires_at_middle", "4"),
            ("R-F2.voltages", "1,3/5,2/5,0"),
            ("R-F2.left.far_ends", "1,1,2/5,0"),
            ("R-F2.left.average", "3/5"),
            ("R-F2.right.far_ends", "1,3/5,0,0"),
            ("R-F2.right.average", "2/5"),
            ("R-F2.currents", "4/5,3/5,1/5,3/5,4/5"),
            ("R-F2.balance", "true"),
            ("R-F3.chances", "1,3/5,2/5,0"),
            ("R-F3.wire_chance", "1/4"),
            ("R-F3.left.far_ends", "1,1,2/5,0"),
            ("R-F3.stops", "1,1"),
            ("R-F3.still_walking", "1/4,1/16,1/64"),
            ("R-F3.route", "left,right,left,right,top"),
            ("R-F3.same", "true"),
            ("R-X1.determinant", "15"),
            ("R-X1.solutions", "1"),
            ("R-X1.zero_ends", "0,0,0,0"),
            ("R-X2.island.cut_off", "4,5"),
            ("R-X2.island.determinant", "0"),
            ("R-X2.island.refused", "true"),
            ("R-X3.family.networks", "729"),
            ("R-X3.family.determined", "672"),
            ("R-X3.family.cut_off", "57"),
            ("R-X3.family.singular", "57"),
            ("R-X3.family.agree", "672"),
            ("R-X3.family.between_ends", "672"),
            ("R-P1.voltages", "7/16,3/8"),
            ("R-P1.current_cd", "1/8"),
            ("R-P1.steps_from_c", "1/4,1/4,1/2"),
            ("R-P2.line5", "0,1/5,2/5,3/5,4/5,1"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    #[test]
    fn the_average_refuses_nothing_to_average() {
        assert!(mean(&[]).is_err());
        assert_eq!(mean(&[Q::ONE, Q::ZERO]), Ok(Q::new(1, 2)));
    }
}
