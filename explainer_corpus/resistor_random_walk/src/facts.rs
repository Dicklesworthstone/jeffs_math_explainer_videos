//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x resistor_random_walk
//! facts-check`): the rows of docs/fact-sheet.md. R-F1 to R-F3 rows are what
//! the opening shows and says, R-F4 to R-F6 chapter 02's derivation, R-X rows
//! the edges of the always/never sentences, R-P rows the primary source's own
//! worked examples.

use std::collections::BTreeSet;

use explainer_kit::Facts;

use crate::model::{self, BOTTOM, LEFT, Network, Q, RIGHT, ROUTE, TOP, Walk, csv};

/// The left junction's far junctions, in the order chapter 02 names them.
pub const LEFT_LINKS: [usize; 3] = [TOP, RIGHT, BOTTOM];

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

/// The fractions added.
pub fn sum(values: &[Q]) -> Q {
    values.iter().fold(Q::ZERO, |s, &q| s + q)
}

/// Chapter 02: the voltage drop towards the left junction along each of
/// its four identical wires, in the order they are drawn.
pub fn drops() -> Result<Vec<Q>, String> {
    let net = network()?;
    net.drops_into(&net.kirchhoff(Q::ONE, Q::ZERO)?, LEFT)
}

/// Chapter 02: the left junction's wires that lead to the top, and all of
/// its wires (the walker's first step up is the one over the other).
pub fn first_step_up() -> Result<(usize, usize), String> {
    let ends = network()?.far_ends(LEFT)?;
    Ok((ends.iter().filter(|&&y| y == TOP).count(), ends.len()))
}

/// Chapter 02: the conductance of each link at the left junction, side-by-
/// side wires added, in [`LEFT_LINKS`] order.
pub fn left_conductances() -> Result<Vec<Q>, String> {
    let net = network()?;
    Ok(LEFT_LINKS.iter().map(|&y| net.between(LEFT, y)).collect())
}

/// Chapter 02's edge, the pair counted as one wire: the walker's chance of
/// a first step up, the voltages, and the chances of reaching the top first.
pub fn single_pair() -> Result<(Q, Vec<Q>, Vec<Q>), String> {
    let one = model::single_pair()?;
    let up = one.between(LEFT, TOP) / one.total(LEFT);
    Ok((up, one.kirchhoff(Q::ONE, Q::ZERO)?, one.walk()?.top))
}

/// Chapter 03: the rounds of repeated averaging the table shows (after
/// round 0, the guess), and how many the always/never checks run.
pub const ROUNDS: usize = 5;
pub const CHECK_ROUNDS: usize = 40;

/// Chapter 03's "within a thousandth".
pub fn tolerance() -> Q {
    Q::new(1, 1000)
}

/// Chapter 03's guess: the top and the bottom at their voltages, both
/// middle junctions at zero.
pub fn guess() -> Result<Vec<Q>, String> {
    let net = network()?;
    let v = net.kirchhoff(Q::ONE, Q::ZERO)?;
    Ok((0..net.size())
        .map(|x| if net.is_end(x) { v[x] } else { Q::ZERO })
        .collect())
}

/// Chapter 03: the guess and `n` rounds of repeated averaging after it,
/// every middle junction updated at once from the previous round's values.
pub fn rounds(n: usize) -> Result<Vec<Vec<Q>>, String> {
    let net = network()?;
    let mut out = vec![guess()?];
    for _ in 0..n {
        let next = net.averaging_round(out.last().ok_or("no round to start from")?)?;
        out.push(next);
    }
    Ok(out)
}

/// Chapter 03: the bigger of the two middle junctions' gaps to their
/// voltages.
pub fn bigger_gap(values: &[Q]) -> Result<Q, String> {
    let v = voltages()?;
    let gap = |x: usize| {
        if v[x] >= values[x] {
            v[x] - values[x]
        } else {
            values[x] - v[x]
        }
    };
    Ok(gap(LEFT).max(gap(RIGHT)))
}

/// Chapter 03's comparison: the guess and `n` rounds taken one middle
/// junction at a time, each using the newest values.
pub fn sequential_rounds(n: usize) -> Result<Vec<Vec<Q>>, String> {
    let net = network()?;
    let mut out = vec![guess()?];
    for _ in 0..n {
        let next = net.sequential_round(out.last().ok_or("no round to start from")?)?;
        out.push(next);
    }
    Ok(out)
}

/// The bigger gap of each round.
pub fn gaps(rounds: &[Vec<Q>]) -> Result<Vec<Q>, String> {
    rounds.iter().map(|r| bigger_gap(&r[..])).collect()
}

/// The one factor by which every gap from round `from` on is the one
/// before it, or an error when there is not exactly one.
pub fn steady_factor(gaps: &[Q], from: usize) -> Result<Q, String> {
    let ratios: BTreeSet<Q> = gaps
        .get(from.saturating_sub(1)..)
        .unwrap_or(&[])
        .windows(2)
        .map(|w| w[1] / w[0])
        .collect();
    match (ratios.first(), ratios.len()) {
        (Some(&r), 1) => Ok(r),
        _ => Err(format!(
            "from round {from} the gaps shrink by {ratios:?}, not one factor"
        )),
    }
}

/// The round from which one-junction-at-a-time updates shrink the gap by
/// one steady factor (round 1's factor differs).
pub const SEQUENTIAL_FROM: usize = 2;

/// How many wires at each middle junction lead to another middle junction,
/// and the weight of that wire in its junction's average: one number each
/// for every middle junction, or an error.
pub fn unknown_link() -> Result<(usize, Q), String> {
    let net = network()?;
    let middle = net.middle();
    let counts: BTreeSet<usize> = middle
        .iter()
        .map(|&x| net.far_ends(x).map(|e| e.iter().filter(|&&y| !net.is_end(y)).count()))
        .collect::<Result<_, _>>()?;
    let weights: BTreeSet<Q> = middle
        .iter()
        .flat_map(|&x| middle.iter().filter(move |&&y| y != x).map(move |&y| (x, y)))
        .map(|(x, y)| net.between(x, y) / net.total(x))
        .collect();
    match (counts.first(), counts.len(), weights.first(), weights.len()) {
        (Some(&k), 1, Some(&w), 1) => Ok((k, w)),
        _ => Err(format!(
            "unknown wires {counts:?} with weights {weights:?}, not one each"
        )),
    }
}

/// Chapter 03's edge: free rounds (the ends averaged too) from the solved
/// voltages, until every value is within [`tolerance`] of where they settle;
/// the rounds and that number.
pub fn drift() -> Result<(Vec<Vec<Q>>, Q), String> {
    let net = network()?;
    let mut out = vec![voltages()?];
    let mean = net.weighted_mean(&out[0])?;
    loop {
        let last = out.last().ok_or("no round")?;
        let off = last
            .iter()
            .map(|&x| if x >= mean { x - mean } else { mean - x })
            .max()
            .unwrap_or(Q::ZERO);
        if off < tolerance() {
            return Ok((out, mean));
        }
        if out.len() > CHECK_ROUNDS {
            return Err(format!(
                "free rounds are not within {} of {mean} after {CHECK_ROUNDS}",
                tolerance()
            ));
        }
        let next = net.free_round(last)?;
        out.push(next);
    }
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
    chapter_02(&mut f, &net, &v, &walk)?;
    chapter_03(&mut f, &net, &v)?;
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

/// Chapter 02's numbers: the balance at the left junction wire by wire
/// (which way each current flows, what flows in and out, the coefficient
/// the unknown gathers), the walker's first step over the same four wires,
/// the conductance weights both pictures share, and the pair counted once.
fn chapter_02(f: &mut Facts, net: &Network, v: &[Q], walk: &Walk) -> Result<(), String> {
    let drops = drops()?;
    if drops.iter().any(|d| d.is_zero()) {
        return Err(format!("a wire at the left junction carries no current: {drops:?}"));
    }
    let flow: Vec<&str> = drops
        .iter()
        .map(|d| if d.is_positive() { "in" } else { "out" })
        .collect();
    let into: Vec<Q> = drops.iter().copied().filter(|d| d.is_positive()).collect();
    let out: Vec<Q> = drops.iter().filter(|d| !d.is_positive()).map(|&d| -d).collect();
    if sum(&into) != sum(&out) || !sum(&drops).is_zero() {
        return Err(format!("the left junction's drops {drops:?} do not balance"));
    }
    f.add("R-F4.drops_into", csv(&drops))?;
    f.add("R-F4.flow", flow.join(","))?;
    f.add("R-F4.in", csv(&into))?;
    f.add("R-F4.out", csv(&out))?;
    f.add("R-F4.in_total", sum(&into))?;
    f.add("R-F4.out_total", sum(&out))?;
    f.add("R-F4.net", sum(&drops))?;
    // the drop along each link the way its current flows, from high to low
    for y in LEFT_LINKS {
        let d = v[y] - v[LEFT];
        f.add(&format!("R-F4.drop.{}", name(y)), if d.is_positive() { d } else { -d })?;
    }
    // each wire's drop is its far end minus the unknown, so the unknown
    // gathers one minus sign per wire
    f.add("R-F4.coefficient", drops.len())?;

    // the walker's first step: four wires, each equally likely
    let (up, wires) = first_step_up()?;
    let to_top = Q::int(up as i128) / Q::int(wires as i128);
    if to_top != net.between(LEFT, TOP) / net.total(LEFT) {
        return Err(format!(
            "counting wires gives {to_top} for a first step up, conductances another"
        ));
    }
    f.add("R-F5.top_wires", up)?;
    f.add("R-F5.to_top", to_top)?;
    let ends = net.far_ends(LEFT)?;
    let neighbours: BTreeSet<usize> = ends.iter().copied().collect();
    f.add("R-F5.neighbours", neighbours.len())?;
    f.add("R-F5.trap", Q::ONE / Q::int(neighbours.len() as i128))?;
    let each = Q::ONE / Q::int(wires as i128);
    let terms: Vec<Q> = ends.iter().map(|&y| each * walk.top[y]).collect();
    if sum(&terms) != walk.top[LEFT] {
        return Err(format!(
            "the first-step terms {terms:?} do not add to the left's chance"
        ));
    }
    f.add("R-F5.terms", csv(&terms))?;
    f.add("R-F5.sum", sum(&terms))?;

    // the general rule: each link's conductance over the junction's total,
    // and the same weights from counting identical wires
    let links = left_conductances()?;
    let weights: Vec<Q> = links.iter().map(|&c| c / net.total(LEFT)).collect();
    let counted: Vec<Q> = LEFT_LINKS
        .iter()
        .map(|&y| Q::int(ends.iter().filter(|&&e| e == y).count() as i128) * each)
        .collect();
    f.add("R-F6.left.conductances", csv(&links))?;
    f.add("R-F6.left.total", net.total(LEFT))?;
    f.add("R-F6.left.weights", csv(&weights))?;
    f.add("R-F6.pair_adds", counted == weights)?;
    // the general rule's domain: at the top the same weighted average is
    // not the top's value, so the rule is stated for middle junctions only
    let top_average = net.average(v, TOP).ok_or("the top has no wires")?;
    if top_average == v[TOP] {
        return Err(format!("the top's weighted average {top_average} equals its value"));
    }
    f.add("R-X5.top_average", top_average)?;

    // the edge: the pair counted as one wire is another network
    let (one_up, one_v, one_p) = single_pair()?;
    f.add("R-X4.single.to_top", one_up)?;
    f.add("R-X4.single.voltages", csv(&one_v))?;
    f.add("R-X4.single.chances", csv(&one_p))?;
    Ok(())
}

/// The two middle junctions' values in a round, as the facts print them.
fn pair(values: &[Q]) -> String {
    csv(&[values[LEFT], values[RIGHT]])
}

/// Chapter 03's numbers: the guess and the rounds the table shows, the far
/// ends each of the first two rounds averages, the bigger gap and how it
/// shrinks (and why: one wire at each junction leads to an unknown), when
/// it falls under a thousandth, the power-of-two denominators that keep
/// every round off 3/5, one junction at a time for comparison, and two
/// edges: the ends averaged too, and an island.
fn chapter_03(f: &mut Facts, net: &Network, v: &[Q]) -> Result<(), String> {
    let all = rounds(CHECK_ROUNDS)?;
    let shown = &all[..=ROUNDS];
    let column = |x: usize| csv(&shown.iter().map(|r| r[x]).collect::<Vec<_>>());
    f.add("R-F7.start", pair(&shown[0]))?;
    f.add("R-F7.left", column(LEFT))?;
    f.add("R-F7.right", column(RIGHT))?;
    for n in [1, 2] {
        let before = &shown[n - 1];
        for x in [LEFT, RIGHT] {
            let ends: Vec<Q> = net.far_ends(x)?.into_iter().map(|y| before[y]).collect();
            if mean(&ends)? != shown[n][x] {
                return Err(format!("round {n} at junction {x} is not its far ends' average"));
            }
            f.add(&format!("R-F7.round{n}.{}_ends", name(x)), csv(&ends))?;
        }
    }
    for n in [1, 2, 3, ROUNDS] {
        f.add(&format!("R-F7.round{n}"), pair(&shown[n]))?;
    }

    // the bigger gap, shrinking to a quarter every round, over every round
    // the check runs
    let all_gaps = gaps(&all)?;
    f.add("R-F7.gaps", csv(&all_gaps[..=ROUNDS]))?;
    f.add("R-F7.shrink", steady_factor(&all_gaps, 1)?)?;
    // why a quarter: at each middle junction one wire leads to an unknown,
    // and its weight is a quarter
    let (unknown, weight) = unknown_link()?;
    f.add("R-F7.unknown_wires", unknown)?;
    f.add("R-F7.unknown_weight", weight)?;
    let within = all_gaps
        .iter()
        .position(|&g| g < tolerance())
        .ok_or("the gap never falls under the tolerance")?;
    f.add("R-F7.tolerance", tolerance())?;
    f.add("R-F7.within", within)?;
    // every round's values are over a power of two, and none is the answer
    let dyadic = all
        .iter()
        .all(|r| [LEFT, RIGHT].iter().all(|&x| (r[x].denom() & (r[x].denom() - 1)) == 0));
    let never = all.iter().all(|r| r[LEFT] != v[LEFT] && r[RIGHT] != v[RIGHT]);
    f.add("R-F7.dyadic", dyadic)?;
    f.add("R-F7.never", never)?;
    f.add("R-F7.limit", pair(v))?;

    // one junction at a time, with the newest values (the card's comparison)
    let seq = sequential_rounds(8)?;
    let seq_gaps = gaps(&seq)?;
    let first = seq_gaps[1] / seq_gaps[0];
    let steady = steady_factor(&seq_gaps, SEQUENTIAL_FROM)?;
    if steady == first {
        return Err(format!(
            "one at a time the gap shrinks by {first} from round 1, so the card's round {SEQUENTIAL_FROM} is wrong"
        ));
    }
    f.add(
        "R-F8.gs.left",
        csv(&seq[..4].iter().map(|r| r[LEFT]).collect::<Vec<_>>()),
    )?;
    f.add(
        "R-F8.gs.right",
        csv(&seq[..4].iter().map(|r| r[RIGHT]).collect::<Vec<_>>()),
    )?;
    f.add("R-F8.gs.first_ratio", first)?;
    f.add("R-F8.gs.from", SEQUENTIAL_FROM)?;
    f.add("R-F8.gs.shrink", steady)?;

    // edge: the ends averaged too, from the solved voltages
    let (free, settle) = drift()?;
    let totals: Vec<Q> = (0..net.size()).map(|x| net.total(x)).collect();
    let kept = free
        .iter()
        .map(|r| net.weighted_mean(r))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .all(|&m| m == settle);
    f.add("R-X6.weights", csv(&totals))?;
    f.add("R-X6.limit", settle)?;
    f.add("R-X6.round1", csv(free.get(1).ok_or("no free round")?))?;
    f.add("R-X6.within", free.len() - 1)?;
    f.add("R-X6.kept", kept)?;

    // edge: an island's two values swap each round and never settle
    let island = model::with_island()?;
    let mut start = vec![Q::ZERO; island.size()];
    start[island.top()] = Q::ONE;
    start[model::JUNCTIONS + 1] = Q::ONE;
    let once = island.averaging_round(&start)?;
    let twice = island.averaging_round(&once)?;
    f.add("R-X7.island.round1", csv(&once[model::JUNCTIONS..]))?;
    f.add("R-X7.island.round2", csv(&twice[model::JUNCTIONS..]))?;
    Ok(())
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
            ("R-F4.drops_into", "2/5,2/5,-1/5,-3/5"),
            ("R-F4.flow", "in,in,out,out"),
            ("R-F4.in_total", "4/5"),
            ("R-F4.out_total", "4/5"),
            ("R-F4.net", "0"),
            ("R-F4.coefficient", "4"),
            ("R-F5.to_top", "1/2"),
            ("R-F5.trap", "1/3"),
            ("R-F5.terms", "1/4,1/4,1/10,0"),
            ("R-F5.sum", "3/5"),
            ("R-F6.left.conductances", "2,1,1"),
            ("R-F6.left.weights", "1/2,1/4,1/4"),
            ("R-F6.pair_adds", "true"),
            ("R-X4.single.to_top", "1/3"),
            ("R-X4.single.voltages", "1,5/11,4/11,0"),
            ("R-X4.single.chances", "1,5/11,4/11,0"),
            ("R-X5.top_average", "8/15"),
            ("R-F7.left", "0,1/2,9/16,19/32,153/256,307/512"),
            ("R-F7.right", "0,1/4,3/8,25/64,51/128,409/1024"),
            ("R-F7.round2.left_ends", "1,1,1/4,0"),
            ("R-F7.round2.right_ends", "1,1/2,0,0"),
            ("R-F7.gaps", "3/5,3/20,3/80,3/320,3/1280,3/5120"),
            ("R-F7.shrink", "1/4"),
            ("R-F7.unknown_weight", "1/4"),
            ("R-F7.within", "5"),
            ("R-F7.dyadic", "true"),
            ("R-F7.never", "true"),
            ("R-F8.gs.first_ratio", "1/6"),
            ("R-F8.gs.shrink", "1/16"),
            ("R-X6.limit", "1/2"),
            ("R-X6.round1", "8/15,3/5,2/5,7/15"),
            ("R-X6.within", "6"),
            ("R-X6.kept", "true"),
            ("R-X7.island.round1", "1,0"),
            ("R-X7.island.round2", "0,1"),
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
