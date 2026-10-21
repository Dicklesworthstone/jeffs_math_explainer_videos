//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x resistor_random_walk
//! facts-check`): the rows of docs/fact-sheet.md. R-F10 and R-F11 are the
//! line of chapters 01 and 02; R-F1 to R-F3 the bridge network and its walk
//! (chapters 03 and 04), R-F12 chapter 03's school shortcuts; R-F4 to R-F6
//! and R-F13 the balances and the walker's equations (chapters 05 and 06)
//! and their solve (07), R-F14 chapter 07's routes; R-F7 and R-F8 chapter
//! 08's rounds, R-F9 chapter 09's difference argument; R-F15 chapter 10's
//! unequal wires and R-F16 chapter 11's two runs. R-X rows are the edges of
//! the always/never sentences, R-P rows the primary source's own worked
//! examples.

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

/// What the battery holds the top and the bottom at: one volt and zero.
pub const BATTERY: [Q; 2] = [Q::ONE, Q::ZERO];

/// The opening network's voltages, the top and the bottom held at
/// [`BATTERY`], junction by junction.
pub fn voltages() -> Result<Vec<Q>, String> {
    network()?.kirchhoff(BATTERY[0], BATTERY[1])
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

/// Chapter 04: the difference of two solutions with the circuit's top and
/// bottom, at the top and at the bottom. Both are held at the battery's
/// values there, so it is exactly 0 at each; refused unless the solved
/// circuit does sit at the battery's values at its ends.
pub fn difference_ends() -> Result<(Q, Q), String> {
    let v = voltages()?;
    if [v[TOP], v[BOTTOM]] != BATTERY {
        return Err(format!(
            "the circuit's ends are {} and {}, not the battery's {} and {}",
            v[TOP], v[BOTTOM], BATTERY[0], BATTERY[1]
        ));
    }
    Ok((Q::ZERO, Q::ZERO))
}

/// Chapter 04's island: the opening network with two more junctions wired
/// only to each other, and those two junctions (the ones it cuts off).
pub fn island() -> Result<(Network, Vec<usize>), String> {
    let net = model::with_island()?;
    let cut = net.cut_off();
    if cut.len() != 2 {
        return Err(format!("the island cuts off junctions {cut:?}, not two"));
    }
    Ok((net, cut))
}

/// Chapter 04: the equal values the island's two junctions are shown
/// taking, in turn.
pub fn island_levels() -> [Q; 3] {
    [Q::ZERO, Q::new(1, 2), Q::ONE]
}

/// Chapter 04: how many steps a walker on the island is followed for.
pub const ISLAND_STEPS: usize = 12;

/// Chapter 04: the island network's values, the circuit at its voltages
/// and the island's two junctions at `a` and `b`.
pub fn with_island_values(a: Q, b: Q) -> Result<Vec<Q>, String> {
    let (net, cut) = island()?;
    let mut values = voltages()?;
    values.resize(net.size(), Q::ZERO);
    values[cut[0]] = a;
    values[cut[1]] = b;
    Ok(values)
}

/// Whether `values` keep the rule on `net`: every middle junction is the
/// conductance-weighted average of its far ends.
pub fn keeps_rule(net: &Network, values: &[Q]) -> bool {
    net.middle().iter().all(|&x| net.average(values, x) == Some(values[x]))
}

/// A junction's name, as the facts print it.
fn name(x: usize) -> &'static str {
    ["top", "left", "right", "bottom"].get(x).copied().unwrap_or("?")
}

/// Chapters 01 and 02: the line of identical wires, junction 0 the bottom
/// and its last the top.
pub fn line() -> Result<Network, String> {
    model::line(model::LINE_WIRES)
}

/// Each step of a route along the line, "up" or "down".
pub fn moves(route: &[usize]) -> Vec<&'static str> {
    route
        .windows(2)
        .map(|s| if s[1] > s[0] { "up" } else { "down" })
        .collect()
}

/// Chapters 10 and 11: Doyle and Snell's unequal wires, on the opening's
/// shape.
pub fn unequal() -> Result<Network, String> {
    model::unequal()
}

/// The far junctions chapter 10 names at a middle junction, in the order
/// it says them: up, across, down.
pub fn across_links(x: usize) -> [usize; 3] {
    [TOP, if x == LEFT { RIGHT } else { LEFT }, BOTTOM]
}

/// The walker's first-step chances from middle junction `x` of `net` to
/// each of the junctions `to`: each link's conductance over the total.
pub fn steps(net: &Network, x: usize, to: &[usize]) -> Vec<Q> {
    to.iter().map(|&y| net.between(x, y) / net.total(x)).collect()
}

/// Chapter 11: how many rounds the two runs are checked over (on the
/// unequal wires and beside the island), and the rounds its table shows.
pub const RUN_CHECK_ROUNDS: usize = 20;
pub const RUN_ROWS: [usize; 5] = [0, 1, 2, 8, 9];

/// The two runs' gap at each of the junctions `at` in round `n`.
pub fn widths(low: &[Vec<Q>], high: &[Vec<Q>], n: usize, at: &[usize]) -> Vec<Q> {
    at.iter().map(|&x| high[n][x] - low[n][x]).collect()
}

/// The first round in which the two runs are within [`tolerance`] of each
/// other at every one of the junctions `at`, if any is.
pub fn runs_within(low: &[Vec<Q>], high: &[Vec<Q>], at: &[usize]) -> Option<usize> {
    (0..low.len().min(high.len())).find(|&n| widths(low, high, n, at).iter().all(|&w| w < tolerance()))
}

/// The one value in `values`, or an error naming `what`.
fn only<T: Ord + Copy + std::fmt::Debug>(values: &BTreeSet<T>, what: &str) -> Result<T, String> {
    match (values.first(), values.len()) {
        (Some(&k), 1) => Ok(k),
        _ => Err(format!("{what}: {values:?}, not one value")),
    }
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
    chapter_04(&mut f, &net)?;
    line_walk(&mut f)?;
    line_circuit(&mut f)?;
    shortcuts(&mut f, &net)?;
    balances(&mut f, &net, &v, &walk)?;
    routes(&mut f, &net, &walk)?;
    unequal_wires(&mut f)?;
    bracket_runs(&mut f)?;

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

/// Chapter 04's numbers: why the rule has one solution here (two solutions
/// with the same top and bottom differ by values that are 0 at both ends
/// and keep the rule; a peak spreads along a path of wires to the top, so
/// the only such difference is 0) and its edge, an island: any equal pair
/// keeps its rule, a walker there is never at either end, and across the
/// family a second solution exists exactly where a junction is cut off.
fn chapter_04(f: &mut Facts, net: &Network) -> Result<(), String> {
    let (top, bottom) = difference_ends()?;
    f.add("R-F9.ends", csv(&[top, bottom]))?;
    let ends: Vec<&str> = net.far_ends(LEFT)?.into_iter().map(name).collect();
    f.add("R-F9.left.far_ends", ends.join(","))?;
    for x in [LEFT, RIGHT] {
        let path = net
            .path(x, TOP)
            .ok_or_else(|| format!("no path of wires joins the {} junction to the top", name(x)))?;
        let names: Vec<&str> = path.into_iter().map(name).collect();
        f.add(&format!("R-F9.path.{}", name(x)), names.join(","))?;
    }
    // the only solution with both ends at 0: its largest and smallest value
    let zero = net.kirchhoff(Q::ZERO, Q::ZERO)?;
    f.add("R-F9.peak", zero.iter().copied().max().ok_or("no junctions")?)?;
    f.add("R-F9.low", zero.iter().copied().min().ok_or("no junctions")?)?;

    // the island: two junctions, one wire between them, no path to an end
    let (island, cut) = island()?;
    f.add("R-X8.island.junctions", cut.len())?;
    let inside = island
        .wires()
        .iter()
        .filter(|w| cut.contains(&w.a) && cut.contains(&w.b))
        .fold(Q::ZERO, |s, w| s + w.conductance);
    f.add("R-X8.island.wires", inside)?;
    let reaches = cut
        .iter()
        .any(|&x| island.path(x, island.top()).is_some() || island.path(x, island.bottom()).is_some());
    f.add("R-X8.island.path", reaches)?;
    // any equal pair keeps its rule, beside the circuit's own values; an
    // unequal pair does not
    let levels = island_levels();
    let keeps: Vec<String> = levels
        .iter()
        .map(|&c| with_island_values(c, c).map(|values| keeps_rule(&island, &values).to_string()))
        .collect::<Result<_, _>>()?;
    f.add("R-X8.island.levels", csv(&levels))?;
    f.add("R-X8.island.keeps", keeps.join(","))?;
    let unequal = with_island_values(levels[0], levels[levels.len() - 1])?;
    f.add("R-X8.island.unequal", keeps_rule(&island, &unequal))?;
    // a walker from the island, step by step: never at either end, always
    // on the island
    let steps = island.step_chances(cut[0], ISLAND_STEPS)?;
    let most_at = |end: usize| steps.iter().map(|s| s[end]).max().unwrap_or(Q::ZERO);
    let on = steps
        .iter()
        .map(|s| cut.iter().fold(Q::ZERO, |t, &x| t + s[x]))
        .min()
        .unwrap_or(Q::ZERO);
    f.add("R-X8.island.steps", ISLAND_STEPS)?;
    f.add(
        "R-X8.island.at_ends",
        csv(&[most_at(island.top()), most_at(island.bottom())]),
    )?;
    f.add("R-X8.island.on_island", on)?;

    // across the family: where a junction is cut off, marking the cut-off
    // junctions 1 and every other 0 keeps the balance with both ends at 0,
    // a second solution beside 0; everywhere else 0 is the only one
    let (mut free, mut only_zero) = (0usize, 0usize);
    for fam in model::family()? {
        let cut = fam.cut_off();
        if cut.is_empty() {
            only_zero += usize::from(fam.kirchhoff(Q::ZERO, Q::ZERO)?.iter().all(|q| q.is_zero()));
        } else {
            let marked: Vec<Q> = (0..fam.size())
                .map(|x| if cut.contains(&x) { Q::ONE } else { Q::ZERO })
                .collect();
            free += usize::from(fam.middle().iter().all(|&x| fam.net_out(&marked, x).is_zero()));
        }
    }
    f.add("R-X8.family.free", free)?;
    f.add("R-X8.family.only_zero", only_zero)?;
    Ok(())
}

/// Chapter 01's numbers (R-F10): the line of five identical wires, the
/// walker one wire above the bottom, its drawn route and the ghost's first
/// step down, each middle junction's two wires picked half the time, the
/// chances climbing in equal steps of a fifth, and the halfway rule with
/// both ends at zero leaving only zero (its chance of walking for ever).
fn line_walk(f: &mut Facts) -> Result<(), String> {
    let row = line()?;
    let walk = row.walk()?;
    let start = model::LINE_START;
    f.add("R-F10.junctions", row.size())?;
    f.add("R-F10.wires", row.wires().len())?;
    f.add("R-F10.start", start)?;
    let counts: BTreeSet<usize> = row
        .middle()
        .iter()
        .map(|&x| row.far_ends(x).map(|ends| ends.len()))
        .collect::<Result<_, _>>()?;
    let wires = only(&counts, "the line's middle junctions' wires")?;
    f.add("R-F10.wires_at_middle", wires)?;
    f.add("R-F10.step", Q::ONE / Q::int(wires as i128))?;
    f.add("R-F10.top_chance", walk.top[row.top()])?;
    f.add("R-F10.bottom_chance", walk.top[row.bottom()])?;
    if !row.is_route(&model::LINE_ROUTE) || !row.is_route(&model::LINE_GHOST) {
        return Err("the line's drawn routes are not walks on it".into());
    }
    f.add("R-F10.route", model::LINE_ROUTE.map(|x| x.to_string()).join(","))?;
    f.add("R-F10.moves", moves(&model::LINE_ROUTE).join(","))?;
    f.add("R-F10.ghost", model::LINE_GHOST.map(|x| x.to_string()).join(","))?;
    f.add("R-F10.chances", csv(&walk.top))?;
    f.add("R-F10.answer", walk.top[start])?;
    let halfway = row
        .middle()
        .iter()
        .all(|&x| (walk.top[x - 1] + walk.top[x + 1]) / Q::int(2) == walk.top[x]);
    f.add("R-F10.halfway", halfway)?;
    let rises: BTreeSet<Q> = walk.top.windows(2).map(|w| w[1] - w[0]).collect();
    f.add("R-F10.step_size", only(&rises, "the line's chances rise by")?)?;
    f.add("R-F10.forever", csv(&row.kirchhoff(Q::ZERO, Q::ZERO)?))?;
    let stops: Vec<Q> = row.middle().iter().map(|&x| walk.top[x] + walk.bottom[x]).collect();
    f.add("R-F10.stops", csv(&stops))?;
    Ok(())
}

/// Chapter 02's numbers (R-F11): the same line with the battery, every
/// wire the same current, so equal drops of a fifth of a volt, the
/// voltages junction by junction, and the walker's chances equal to them.
fn line_circuit(f: &mut Facts) -> Result<(), String> {
    let row = line()?;
    let v = row.kirchhoff(BATTERY[0], BATTERY[1])?;
    f.add("R-F11.top_volts", v[row.top()])?;
    f.add("R-F11.bottom_volts", v[row.bottom()])?;
    let currents: BTreeSet<Q> = row.currents(&v).into_iter().collect();
    only(&currents, "the line's wires' currents")?;
    f.add("R-F11.same_current", true)?;
    // each wire's drop, from its upper junction to its lower one
    let drops: Vec<Q> = row.wires().iter().map(|w| v[w.b] - v[w.a]).collect();
    f.add("R-F11.drops", csv(&drops))?;
    f.add("R-F11.drop_count", drops.len())?;
    let sizes: BTreeSet<Q> = drops.iter().copied().collect();
    f.add("R-F11.drop", only(&sizes, "the line's drops")?)?;
    f.add("R-F11.volts", csv(&v))?;
    f.add("R-F11.answer", v[model::LINE_START])?;
    f.add("R-F11.same", v == row.walk()?.top)?;
    Ok(())
}

/// Chapter 03's dead end (R-F12): with each pair of wires side by side
/// merged, how many links remain, how many pairs of them still join the
/// same two junctions, how many links meet at each middle junction and at
/// how many middle junctions exactly two meet (in a row), the bridge
/// between the two middle junctions; and what the two shortcuts do,
/// side by side (conductances add) and in a row (half as easy).
fn shortcuts(f: &mut Facts, net: &Network) -> Result<(), String> {
    let mut pairs: Vec<(usize, usize)> = net.wires().iter().map(|w| (w.a.min(w.b), w.a.max(w.b))).collect();
    f.add("R-F12.links", pairs.len())?;
    pairs.sort_unstable();
    f.add("R-F12.side_by_side", pairs.windows(2).filter(|p| p[0] == p[1]).count())?;
    let meeting: Vec<String> = [LEFT, RIGHT]
        .iter()
        .map(|&x| net.neighbours(x).len().to_string())
        .collect();
    f.add("R-F12.meeting", meeting.join(","))?;
    let in_a_row = net.middle().iter().filter(|&&x| net.neighbours(x).len() == 2).count();
    f.add("R-F12.in_a_row", in_a_row)?;
    let middle = net.middle();
    let bridges: Vec<String> = middle
        .iter()
        .flat_map(|&x| middle.iter().filter(move |&&y| y > x).map(move |&y| (x, y)))
        .filter(|&(x, y)| !net.between(x, y).is_zero())
        .map(|(x, y)| format!("{}-{}", name(x), name(y)))
        .collect();
    f.add("R-F12.bridge", bridges.join(","))?;
    f.add("R-F12.pair_conductance", net.between(TOP, LEFT))?;
    // two identical wires in a row with one volt across: the current they
    // pass is the conductance of the one wire they act as
    let two = model::line(2)?;
    let v2 = two.kirchhoff(Q::ONE, Q::ZERO)?;
    f.add("R-F12.row_conductance", (v2[2] - v2[1]) * two.between(2, 1))?;
    Ok(())
}

/// Chapter 07's solve of the two balances, as the narration speaks it:
/// each balance times its wires is `a x = c + cross x'`; the left is
/// (c_left + cross right)/a, put into the right's and multiplied by a, so
/// a² right = a c_right + c_left + cross right, (a² − cross²) right = six.
pub struct Solve {
    /// The wires at each middle junction (both carry the same number).
    pub a: Q,
    /// The bridge's wires between the two.
    pub cross: Q,
    /// Each balance's fixed far ends added: the left's and the right's.
    pub left_constant: Q,
    pub right_constant: Q,
    /// a², a times the right's constant, a² − cross² (the balance's
    /// determinant), and a c_right + cross c_left.
    pub sixteen: Q,
    pub four: Q,
    pub fifteen: Q,
    pub six: Q,
    /// The right, then the left's numerator over a, and the left.
    pub right: Q,
    pub left_sum: Q,
    pub left: Q,
}

/// The solve of `net`'s two balances with the battery's top and bottom;
/// refused unless the two middle junctions carry the same wires, a² − cross²
/// is the balance's determinant, and the result is the circuit's `v`.
pub fn solve(net: &Network, v: &[Q]) -> Result<Solve, String> {
    let a = net.total(LEFT);
    if net.total(RIGHT) != a {
        return Err(format!("the middle junctions carry {} and {} wires", a, net.total(RIGHT)));
    }
    let cross = net.between(LEFT, RIGHT);
    let fixed = |x: usize| net.between(x, TOP) * BATTERY[0] + net.between(x, BOTTOM) * BATTERY[1];
    let (left_constant, right_constant) = (fixed(LEFT), fixed(RIGHT));
    let sixteen = a * a;
    let four = a * right_constant;
    let fifteen = sixteen - cross * cross;
    let six = four + cross * left_constant;
    if fifteen != net.middle_determinant() {
        return Err(format!(
            "{fifteen} is not the balance's determinant {}",
            net.middle_determinant()
        ));
    }
    let right = six / fifteen;
    let left_sum = left_constant + cross * right;
    let left = left_sum / a;
    if [left, right] != [v[LEFT], v[RIGHT]] {
        return Err(format!(
            "the spoken solve gives {left} and {right}, the circuit {} and {}",
            v[LEFT], v[RIGHT]
        ));
    }
    Ok(Solve {
        a,
        cross,
        left_constant,
        right_constant,
        sixteen,
        four,
        fifteen,
        six,
        right,
        left_sum,
        left,
    })
}

/// Chapter 07's routes from the left junction of `net`, until the walker
/// stops or first comes back: straight up, across then up, across and back,
/// straight down, across then down; the chance of the top first as the
/// first two over the chance of not coming back, and of the bottom first.
pub struct Routes {
    pub up_now: Q,
    pub across_up: Q,
    pub back: Q,
    pub down_now: Q,
    pub across_down: Q,
    pub exit_top: Q,
    pub kept: Q,
    pub sum: Q,
    pub bottom_by_routes: Q,
}

/// The routes from the left of `net`; refused unless they add to one and
/// give the walk's own chances.
pub fn routes_from_left(net: &Network, walk: &Walk) -> Result<Routes, String> {
    let p = |x: usize, y: usize| net.between(x, y) / net.total(x);
    let (up_now, across_up, back) = (
        p(LEFT, TOP),
        p(LEFT, RIGHT) * p(RIGHT, TOP),
        p(LEFT, RIGHT) * p(RIGHT, LEFT),
    );
    let (down_now, across_down) = (p(LEFT, BOTTOM), p(LEFT, RIGHT) * p(RIGHT, BOTTOM));
    let outcomes = up_now + across_up + back + down_now + across_down;
    if outcomes != Q::ONE {
        return Err(format!("the left's first outcomes add to {outcomes}, not 1"));
    }
    let exit_top = up_now + across_up;
    let kept = Q::ONE - back;
    let sum = exit_top / kept;
    let bottom_by_routes = (down_now + across_down) / kept;
    if sum != walk.top[LEFT] || bottom_by_routes != walk.bottom[LEFT] {
        return Err(format!(
            "the routes give {sum} and {bottom_by_routes}, the walk {} and {}",
            walk.top[LEFT], walk.bottom[LEFT]
        ));
    }
    Ok(Routes {
        up_now,
        across_up,
        back,
        down_now,
        across_down,
        exit_top,
        kept,
        sum,
        bottom_by_routes,
    })
}

/// Chapters 05 to 07's numbers (R-F13): the right junction's far ends and
/// first-step chances, each balance's fixed far ends added, the walker's
/// equations the circuit's, and the solve the narration speaks: the left is
/// (2 + right)/4, put into 4 right = 1 + left and multiplied by 4, so
/// 16 right = 4 + 2 + right, 15 right = 6, and the left (2 + 2/5)/4.
fn balances(f: &mut Facts, net: &Network, v: &[Q], walk: &Walk) -> Result<(), String> {
    let ends: Vec<&str> = net.far_ends(RIGHT)?.into_iter().map(name).collect();
    f.add("R-F13.right.far_ends", ends.join(","))?;
    f.add("R-F13.right.steps", csv(&steps(net, RIGHT, &[TOP, LEFT, BOTTOM])))?;
    let s = solve(net, v)?;
    f.add("R-F13.left_constant", s.left_constant)?;
    f.add("R-F13.right_constant", s.right_constant)?;
    // both pictures' values keep a x = c + cross x' at both junctions
    let keeps = |values: &[Q]| {
        s.a * values[LEFT] == s.left_constant + s.cross * values[RIGHT]
            && s.a * values[RIGHT] == s.right_constant + s.cross * values[LEFT]
    };
    f.add("R-F13.walk_same", keeps(v) && keeps(&walk.top))?;
    f.add("R-F13.sixteen", s.sixteen)?;
    f.add("R-F13.four", s.four)?;
    f.add("R-F13.fifteen", s.fifteen)?;
    f.add("R-F13.six", s.six)?;
    f.add("R-F13.right", s.right)?;
    f.add("R-F13.left_sum", s.left_sum)?;
    f.add("R-F13.left", s.left)?;
    Ok(())
}

/// Chapter 07's routes (R-F14): from the left, until the walker stops or
/// first comes back (the five outcomes add to one); the chance of reaching
/// the top first as nine sixteenths over the fifteen sixteenths that do not
/// come back; and the chance of reaching the bottom first, by the walk and
/// by the routes.
fn routes(f: &mut Facts, net: &Network, walk: &Walk) -> Result<(), String> {
    let r = routes_from_left(net, walk)?;
    f.add("R-F14.up_now", r.up_now)?;
    f.add("R-F14.across_up", r.across_up)?;
    f.add("R-F14.back", r.back)?;
    f.add("R-F14.down_now", r.down_now)?;
    f.add("R-F14.across_down", r.across_down)?;
    f.add(
        "R-F14.outcomes",
        r.up_now + r.across_up + r.back + r.down_now + r.across_down,
    )?;
    f.add("R-F14.exit_top", r.exit_top)?;
    f.add("R-F14.kept", r.kept)?;
    f.add("R-F14.sum", r.sum)?;
    f.add("R-F14.bottom_first", pair(&walk.bottom))?;
    f.add("R-F14.bottom_first.left", walk.bottom[LEFT])?;
    f.add("R-F14.bottom_by_routes", r.bottom_by_routes)?;
    Ok(())
}

/// Chapter 10's numbers (R-F15): Doyle and Snell's unequal wires on the
/// opening's shape, each middle junction's links up, across and down, their
/// total and the walker's first-step chances, the left's likeliest step,
/// and the voltages and chances (their 7/16 and 3/8, `R-P1`).
fn unequal_wires(f: &mut Facts) -> Result<(), String> {
    let net = unequal()?;
    let all: Vec<Q> = net.wires().iter().map(|w| w.conductance).collect();
    f.add("R-F15.conductances", csv(&all))?;
    for x in [LEFT, RIGHT] {
        let to = across_links(x);
        let c: Vec<Q> = to.iter().map(|&y| net.between(x, y)).collect();
        f.add(&format!("R-F15.{}.conductances", name(x)), csv(&c))?;
        f.add(&format!("R-F15.{}.total", name(x)), net.total(x))?;
        f.add(&format!("R-F15.{}.steps", name(x)), csv(&steps(&net, x, &to)))?;
    }
    let left = steps(&net, LEFT, &across_links(LEFT));
    let most = left.iter().copied().max().ok_or("the left has no wire")?;
    let likeliest: Vec<usize> = (0..left.len()).filter(|&i| left[i] == most).collect();
    if likeliest.len() != 1 {
        return Err(format!("the left's likeliest steps are {likeliest:?}, not one"));
    }
    f.add("R-F15.most_likely", ["up", "across", "down"][likeliest[0]])?;
    let v = net.kirchhoff(BATTERY[0], BATTERY[1])?;
    let p = net.walk()?.top;
    let theirs = model::doyle_snell()?.kirchhoff(Q::ONE, Q::ZERO)?;
    if [v[LEFT], v[RIGHT]] != [theirs[2], theirs[3]] {
        return Err(format!("the drawn wires give {} and {}, not Doyle and Snell's", v[LEFT], v[RIGHT]));
    }
    f.add("R-F15.voltages", pair(&v))?;
    f.add("R-F15.chances", pair(&p))?;
    f.add("R-F15.same", v == p)?;
    Ok(())
}

/// Chapter 11's numbers (R-F16, R-X9): the two runs on the unequal wires,
/// the top held at 1 and the bottom at 0 in both, from all zeros and from
/// all ones at the middle junctions, every junction at once from the round
/// before; the table's rounds and their gaps; the first round both gaps
/// are under a thousandth; the order kept, the answer between, and the gap
/// the chance of still walking. Beside an island: the island's runs never
/// move, a gap of 1 for ever, while the unequal wires' own still close.
fn bracket_runs(f: &mut Facts) -> Result<(), String> {
    let net = unequal()?;
    let v = net.kirchhoff(BATTERY[0], BATTERY[1])?;
    let middle = net.middle();
    let (low, high) = net.two_runs(RUN_CHECK_ROUNDS)?;
    f.add("R-F16.starts", csv(&[low[0][LEFT], high[0][LEFT]]))?;
    let held = low
        .iter()
        .chain(&high)
        .all(|r| [r[TOP], r[BOTTOM]] == BATTERY);
    f.add("R-F16.ends_held", held)?;
    // every junction at once from the round before, not one at a time
    let simultaneous = (1..low.len()).all(|n| net.averaging_round(&low[n - 1]).is_ok_and(|r| r == low[n]))
        && net.sequential_round(&low[0])? != low[1];
    f.add("R-F16.simultaneous", simultaneous)?;
    let mut bigger = Vec::new();
    for n in RUN_ROWS {
        f.add(&format!("R-F16.low.round{n}"), pair(&low[n]))?;
        f.add(&format!("R-F16.high.round{n}"), pair(&high[n]))?;
        let gaps = widths(&low, &high, n, &[LEFT, RIGHT]);
        f.add(&format!("R-F16.width.round{n}"), csv(&gaps))?;
        bigger.push(gaps.iter().copied().max().ok_or("no gap")?);
    }
    f.add("R-F16.bigger_width", csv(&bigger))?;
    let within = runs_within(&low, &high, &middle).ok_or("the two runs never come within the tolerance")?;
    f.add("R-F16.first_within", within)?;
    let monotone = (1..low.len()).all(|n| middle.iter().all(|&x| low[n][x] >= low[n - 1][x] && high[n][x] <= high[n - 1][x]));
    let between = (0..low.len()).all(|n| middle.iter().all(|&x| low[n][x] <= v[x] && v[x] <= high[n][x]));
    f.add("R-F16.monotone", monotone)?;
    f.add("R-F16.between", between)?;
    let mut walking = true;
    for &x in &middle {
        let still = net.still_walking(x, STILL_CHECK)?;
        walking &= (1..=STILL_CHECK).all(|n| high[n][x] - low[n][x] == still[n - 1]);
    }
    f.add("R-F16.width_is_still_walking", walking)?;

    // beside an island: the island's runs stay at 0 and 1, so its gap is 1
    // for ever, and the unequal wires' junctions close as before
    let island = model::beside_island(&net)?;
    let cut = island.cut_off();
    f.add("R-X9.island.junctions", cut.len())?;
    let (ilow, ihigh) = island.two_runs(RUN_CHECK_ROUNDS)?;
    let most_low = ilow
        .iter()
        .flat_map(|r| cut.iter().map(move |&x| r[x]))
        .max()
        .ok_or("no island junction")?;
    let least_high = ihigh
        .iter()
        .flat_map(|r| cut.iter().map(move |&x| r[x]))
        .min()
        .ok_or("no island junction")?;
    let least_width = (0..ilow.len())
        .flat_map(|n| widths(&ilow, &ihigh, n, &cut))
        .min()
        .ok_or("no island junction")?;
    f.add("R-X9.island.low", most_low)?;
    f.add("R-X9.island.high", least_high)?;
    f.add("R-X9.island.moves", most_low != Q::ZERO || least_high != Q::ONE)?;
    f.add("R-X9.island.width", least_width)?;
    f.add("R-X9.island.within", runs_within(&ilow, &ihigh, &cut).is_some())?;
    let own = runs_within(&ilow, &ihigh, &middle).ok_or("the unequal wires never close beside the island")?;
    f.add("R-X9.network.within", own)?;
    f.add("R-X9.determinant", island.middle_determinant())?;
    Ok(())
}

/// How many steps chapter 11's gap is checked against the chance of
/// still walking.
const STILL_CHECK: usize = 12;

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
            ("R-F9.ends", "0,0"),
            ("R-F9.left.far_ends", "top,top,right,bottom"),
            ("R-F9.path.left", "left,top"),
            ("R-F9.path.right", "right,top"),
            ("R-F9.peak", "0"),
            ("R-F9.low", "0"),
            ("R-X8.island.junctions", "2"),
            ("R-X8.island.wires", "1"),
            ("R-X8.island.path", "false"),
            ("R-X8.island.levels", "0,1/2,1"),
            ("R-X8.island.keeps", "true,true,true"),
            ("R-X8.island.unequal", "false"),
            ("R-X8.island.steps", "12"),
            ("R-X8.island.at_ends", "0,0"),
            ("R-X8.island.on_island", "1"),
            ("R-X8.family.free", "57"),
            ("R-X8.family.only_zero", "672"),
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
