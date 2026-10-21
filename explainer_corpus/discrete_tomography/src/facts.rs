//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x discrete_tomography facts-check`): the rows
//! of docs/fact-sheet.md. DT-F rows are computed, DT-S rows carry a source's
//! literal, DT-X rows are the edges of the always/never sentences, checked on
//! every 4x4 picture.

use std::collections::HashMap;
use std::fmt::Display;

use explainer_kit::Facts;

use crate::chapters::promise::STEPS;
use crate::chapters::same_shadows::QUESTIONS;
use crate::model::{
    self, Augmenting, Deduction, Flow, FlowState, Line, Node, Picture, Reconstruction, Settled, Shadows, Switch,
    Verdict,
};

/// The year of Ryser's paper, shown on chapter 01's "precisely" card: H. J.
/// Ryser, "Combinatorial properties of matrices of zeros and ones",
/// Canadian Journal of Mathematics 9 (1957) 371–377, Theorem 3.1.
pub const RYSER_YEAR: u32 = 1957;

/// The year of the thesis on chapter 02's card: K. J. Batenburg, *Network
/// Flow Algorithms for Discrete Tomography*, PhD thesis, Leiden University,
/// 2006, §1.9.1, pp. 31–32 (counting the atoms in each projected column of a
/// nanocrystal, from few images).
pub const BATENBURG_YEAR: u32 = 2006;

/// Chapter 03's larger grid: 20 by 20 squares.
pub const BIG_SIDE: usize = 20;

/// The grid the edge facts (rows DT-X) run over: every 4 by 4 picture.
pub const EDGE_SIDE: usize = 4;

/// Chapter 06's other pairs of columns (06_9): each asks less than the two
/// largest, and the rows can give each pair the same.
pub const OTHER_PAIRS: [[usize; 2]; 2] = [[0, 2], [2, 3]];

/// Chapter 07's shadows whose totals differ: they pass every size of Gale's
/// table and fit nothing.
pub const UNEQUAL_ROWS: [usize; 2] = [2, 2];
pub const UNEQUAL_COLS: [usize; 2] = [1, 1];

/// Chapter 09's two 3 by 3 pictures with one square in every row and every
/// column, the second the first shifted a column: six squares apart, not one
/// switch apart, two switches apart.
pub const PERM_LEFT: [&str; 3] = ["100", "010", "001"];
pub const PERM_RIGHT: [&str; 3] = ["010", "001", "100"];

fn csv<T: Display>(values: &[T]) -> String {
    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

fn flag(ok: bool) -> &'static str {
    if ok { "true" } else { "false" }
}

/// The picture the opening switch turns the face into.
pub fn twin() -> Result<Picture, String> {
    model::OPENING
        .apply(&model::face())
        .ok_or_else(|| format!("{:?} is not a switch of the face", model::OPENING))
}

/// Every picture with the face's shadows, nearest the face first: each with
/// the fewest switches that reach it from the face.
pub fn gallery() -> Vec<(Picture, usize)> {
    model::reachable(&model::face())
}

/// Chapter 01's walk: the opening switch, then a shortest chain from the twin
/// to the first picture (in pattern order) as far from the face as any, and
/// one switch nearer the twin. Returns the chain and where it ends.
pub fn walk() -> Result<(Vec<Switch>, Picture), String> {
    let twin = twin()?;
    let from_face = gallery();
    let from_twin: HashMap<Picture, usize> = model::reachable(&twin).into_iter().collect();
    let farthest = from_face.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let nearer = farthest.checked_sub(1).ok_or("the face has no switch")?;
    let (end, _) = from_face
        .into_iter()
        .find(|(p, d)| *d == farthest && from_twin.get(p) == Some(&nearer))
        .ok_or("no farthest picture is one switch nearer the twin")?;
    let mut chain = vec![model::OPENING];
    chain.extend(model::walk(&twin, &end).ok_or("no chain of switches joins the twin and the end")?);
    Ok((chain, end))
}

/// Chapter 01's shadows that no picture fits: the face's row counts and
/// [`model::NO_FIT_COLS`].
pub fn no_fit() -> Shadows {
    Shadows::new(&model::face().row_counts(), &model::NO_FIT_COLS)
}

/// The bottleneck of [`no_fit`]: its columns, how many squares they ask
/// for, and how many the rows can give them.
pub fn no_fit_bottleneck() -> Result<(Vec<usize>, usize, usize), String> {
    let s = no_fit();
    let set = model::bottleneck(&s).ok_or("the no-fit shadows have no bottleneck")?;
    let asks = set.iter().map(|&j| s.cols[j]).sum();
    let can_give = model::can_give(&s.rows, set.len());
    Ok((set, asks, can_give))
}

/// What each row and each column of switch `s` loses and gains in `p`: one
/// pair per row, then per column.
fn losses_and_gains(s: Switch, p: &Picture) -> Result<Vec<(usize, usize)>, String> {
    let sides = s
        .sides(p)
        .ok_or_else(|| format!("{s:?} is not a switch of {}", p.pattern()))?;
    let mut out = Vec::new();
    for r in s.rows {
        let on = |corners: &[(usize, usize); 2]| corners.iter().filter(|rc| rc.0 == r).count();
        out.push((on(&sides.emptied), on(&sides.filled)));
    }
    for c in s.cols {
        let on = |corners: &[(usize, usize); 2]| corners.iter().filter(|rc| rc.1 == c).count();
        out.push((on(&sides.emptied), on(&sides.filled)));
    }
    Ok(out)
}

/// The years on chapter 02's "precisely" card: L. R. Ford Jr. and D. R.
/// Fulkerson, "A simple algorithm for finding maximal network flows and an
/// application to the Hitchcock problem", Canadian Journal of Mathematics 9
/// (1957) 210–218 (the labeling method, its termination, and Lemmas 1 and 2:
/// what it ends with is a flow, and a largest one); D. Gale, "A theorem on
/// flows in networks", Pacific Journal of Mathematics 7 (1957) 1073–1082
/// (filling a 0-1 matrix to given row and column sums is a flow problem).
pub const FORD_FULKERSON_YEAR: u32 = 1957;
pub const GALE_YEAR: u32 = 1957;

/// Chapter 02: the flow through the face's shadows, its augmenting paths in
/// the order they were sent, and the water after each (checked by
/// [`model::replay`]).
pub fn face_flow() -> Result<(Vec<Augmenting>, Vec<FlowState>), String> {
    let s = model::face().shadows();
    let flow = model::max_flow(&s);
    let states = model::replay(&s, &flow.paths)?;
    match states.last() {
        Some(last) if last.wet == flow.carried && last.through() == flow.value => Ok((flow.paths, states)),
        _ => Err("the replayed water is not the flow's".into()),
    }
}

/// Chapter 03: the flow through the no-fit shadows (it stops short), and the
/// water after each of its paths (checked by [`model::replay`]).
pub fn no_fit_flow() -> Result<(Flow, Vec<FlowState>), String> {
    let s = no_fit();
    let flow = model::max_flow(&s);
    let states = model::replay(&s, &flow.paths)?;
    match states.last() {
        Some(last) if last.wet == flow.carried && last.through() == flow.value => Ok((flow, states)),
        _ => Err("the replayed water is not the no-fit flow's".into()),
    }
}

/// Chapter 04: the flow through the tree's shadows, and the water after
/// each of its paths (checked by [`model::replay`]).
pub fn tree_flow() -> Result<(Flow, Vec<FlowState>), String> {
    let s = model::tree().shadows();
    let flow = model::max_flow(&s);
    let states = model::replay(&s, &flow.paths)?;
    match states.last() {
        Some(last) if last.wet == flow.carried && last.through() == flow.value => Ok((flow, states)),
        _ => Err("the replayed water is not the tree's flow's".into()),
    }
}

/// Chapter 05: the face's 64 pictures that also cast its diagonal shadow
/// ([`model::diagonal_counts`]), in increasing order; the face is one.
pub fn diagonal_survivors() -> Vec<Picture> {
    let target = model::diagonal_counts(&model::face());
    let mut out: Vec<Picture> = gallery()
        .into_iter()
        .map(|(p, _)| p)
        .filter(|p| model::diagonal_counts(p) == target)
        .collect();
    out.sort();
    out
}

/// Chapter 10: the face's 64 pictures that also cast its shadow along the
/// other diagonals ([`model::falling_counts`]), in increasing order.
pub fn falling_survivors() -> Vec<Picture> {
    let target = model::falling_counts(&model::face());
    let mut out: Vec<Picture> = gallery()
        .into_iter()
        .map(|(p, _)| p)
        .filter(|p| model::falling_counts(p) == target)
        .collect();
    out.sort();
    out
}

/// Chapter 10: the face's 64 pictures that cast both its diagonal shadows.
pub fn both_survivors() -> Vec<Picture> {
    let falling = falling_survivors();
    diagonal_survivors().into_iter().filter(|p| falling.contains(p)).collect()
}

/// Chapter 10: the face's 64 pictures grouped by their diagonal counts, each
/// group in increasing order; the groups smallest first, then in order of
/// their first picture.
pub fn diagonal_groups() -> Vec<Vec<Picture>> {
    let mut by_counts: HashMap<Vec<usize>, Vec<Picture>> = HashMap::new();
    for (p, _) in gallery() {
        by_counts.entry(model::diagonal_counts(&p)).or_default().push(p);
    }
    let mut groups: Vec<Vec<Picture>> = by_counts.into_values().collect();
    for g in &mut groups {
        g.sort();
    }
    groups.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    groups
}

/// Chapter 02: how many of the face's 64 pictures have each square filled,
/// row by row.
pub fn filled_in() -> Vec<Vec<usize>> {
    let face = model::face();
    let pictures = gallery();
    (0..face.rows())
        .map(|r| {
            (0..face.cols())
                .map(|c| pictures.iter().filter(|(p, _)| p.get(r, c)).count())
                .collect()
        })
        .collect()
}

/// Chapter 02's reasoning on the tree, in the order the chapter tells it:
/// the lines whose count fills them whole; then the lines whose count is
/// already met, columns before rows; then the rest. Each is checked to be,
/// in that order, the deduction its line forces ([`model::deduce`]), and
/// together they settle the tree.
pub fn tree_reasoning() -> Result<[Vec<Deduction>; 3], String> {
    let tree = model::tree();
    let s = tree.shadows();
    let (rows, cols) = (s.rows.len(), s.cols.len());
    let solve = model::line_solve(&s);
    if solve.solved().as_ref() != Some(&tree) {
        return Err("reasoning one line at a time does not settle the tree".into());
    }
    let whole = |d: &&Deduction| d.fill && d.line.count(&s) == d.line.size(rows, cols);
    let full: Vec<Deduction> = solve.steps.iter().filter(whole).cloned().collect();
    let mut met: Vec<Deduction> = solve.steps.iter().filter(|d| !d.fill).cloned().collect();
    met.sort_by_key(|d| match d.line {
        Line::Col(c) => (0, c),
        Line::Row(r) => (1, r),
    });
    let rest: Vec<Deduction> = solve
        .steps
        .iter()
        .filter(|d| d.fill && !whole(d))
        .cloned()
        .collect();
    let mut grid: Settled = vec![vec![None; cols]; rows];
    for step in full.iter().chain(&met).chain(&rest) {
        match model::deduce(&s, &grid, step.line) {
            Verdict::Step(d) if d == *step => model::settle(&mut grid, &d),
            other => {
                return Err(format!("{} is not forced in the told order: {other:?}", step.label()));
            }
        }
    }
    if grid != solve.grid {
        return Err("the told order does not settle what the sweep settles".into());
    }
    Ok([full, met, rest])
}

/// The number of decimal digits of 2^`n`, exactly: a decimal number doubled
/// `n` times, its digits least significant first.
pub fn power_of_two_digits(n: usize) -> usize {
    let mut digits: Vec<u8> = vec![1];
    for _ in 0..n {
        let mut carry = 0;
        for d in &mut digits {
            let doubled = *d * 2 + carry;
            *d = doubled % 10;
            carry = doubled / 10;
        }
        if carry > 0 {
            digits.push(carry);
        }
    }
    digits.len()
}

/// Chapter 06's shadows ([`model::STALL_ROWS`], [`model::STALL_COLS`]).
pub fn stall() -> Shadows {
    Shadows::new(&model::STALL_ROWS, &model::STALL_COLS)
}

/// Chapter 06: the flow through [`stall`]'s shadows (it stops short), and the
/// water after each of its paths (checked by [`model::replay`]).
pub fn stall_flow() -> Result<(Flow, Vec<FlowState>), String> {
    let s = stall();
    let flow = model::max_flow(&s);
    let states = model::replay(&s, &flow.paths)?;
    match states.last() {
        Some(last) if last.wet == flow.carried && last.through() == flow.value => Ok((flow, states)),
        _ => Err("the replayed water is not the stall flow's".into()),
    }
}

/// Chapter 07's shadows whose totals differ.
pub fn unequal() -> Shadows {
    Shadows::new(&UNEQUAL_ROWS, &UNEQUAL_COLS)
}

/// Chapter 09's two 3 by 3 pictures.
pub fn perms() -> Result<(Picture, Picture), String> {
    Ok((Picture::parse(&PERM_LEFT)?, Picture::parse(&PERM_RIGHT)?))
}

/// Deductions as their labels, in order: "r2+5,c0-4".
fn step_labels(steps: &[Deduction]) -> String {
    steps.iter().map(Deduction::label).collect::<Vec<_>>().join(",")
}

/// A switch as "rows;columns": "0,1;0,1".
fn switch_text(s: Switch) -> String {
    format!("{};{}", csv(&s.rows), csv(&s.cols))
}

/// Squares as "r,c;r,c".
fn cells_text(cells: &[(usize, usize)]) -> String {
    cells.iter().map(|&(r, c)| format!("{r},{c}")).collect::<Vec<_>>().join(";")
}

/// The one value every item shares, or an error naming `what`.
fn common<T: Copy + PartialEq + std::fmt::Debug>(items: &[T], what: &str) -> Result<T, String> {
    let &first = items.first().ok_or_else(|| format!("no {what}"))?;
    if items.iter().all(|&x| x == first) {
        Ok(first)
    } else {
        Err(format!("the {what} differ: {items:?}"))
    }
}

/// What each line through the squares where `a` and `b` differ loses and
/// gains from `a` to `b`: one (lost, gained) pair for each row, then each
/// column, then each diagonal that holds any of those squares.
pub fn line_changes(a: &Picture, b: &Picture) -> Vec<(usize, usize)> {
    let differ = a.differences(b);
    let families: [fn((usize, usize)) -> usize; 3] = [|(r, _)| r, |(_, c)| c, |(r, c)| r + c];
    let mut out = Vec::new();
    for line_of in families {
        let mut lines: Vec<usize> = differ.iter().map(|&rc| line_of(rc)).collect();
        lines.sort_unstable();
        lines.dedup();
        for line in lines {
            let on: Vec<(usize, usize)> = differ.iter().copied().filter(|&rc| line_of(rc) == line).collect();
            let lost = on.iter().filter(|&&(r, c)| a.get(r, c)).count();
            out.push((lost, on.len() - lost));
        }
    }
    out
}

/// Column counts from largest to smallest, as Gale's table takes them.
pub fn largest_first(cols: &[usize]) -> Vec<usize> {
    let mut sorted = cols.to_vec();
    sorted.sort_unstable();
    sorted.reverse();
    sorted
}

/// The sizes k at which Gale's table fails (the k largest columns ask for
/// more than the rows can give), or "none".
fn failing_sizes(table: &[(usize, usize)]) -> String {
    let sizes: Vec<usize> = table
        .iter()
        .zip(1..)
        .filter(|((asks, gives), _)| asks > gives)
        .map(|(_, k)| k)
        .collect();
    if sizes.is_empty() {
        "none".to_string()
    } else {
        csv(&sizes)
    }
}

/// Chapter 02's turning point: the first path that takes water back.
pub fn first_takeback(paths: &[Augmenting]) -> Result<usize, String> {
    paths
        .iter()
        .position(Augmenting::takes_back)
        .ok_or_else(|| "no path takes water back".to_string())
}

/// A path as its rows and columns in order: "r4c0r0c3".
fn path_text(p: &Augmenting) -> String {
    p.nodes
        .iter()
        .filter_map(|node| match node {
            Node::Row(r) => Some(format!("r{r}")),
            Node::Col(c) => Some(format!("c{c}")),
            Node::Source | Node::Sink => None,
        })
        .collect()
}

/// The edge checks on every 4x4 picture (rows DT-X).
pub struct Edges {
    /// Pictures checked: every [`EDGE_SIDE`] by [`EDGE_SIDE`] picture.
    pub pictures: usize,
    /// (picture, switch) pairs checked, and how many changed a shadow or
    /// anything but the four corners.
    pub pairs: usize,
    pub failed: usize,
    /// Pairs of shadows some picture casts, and how many of them have all
    /// their pictures joined by switches.
    pub shadow_pairs: usize,
    pub connected: usize,
    /// Pictures alone with their shadows, pictures with no switch, and
    /// pictures where the two disagree.
    pub alone: usize,
    pub no_switch: usize,
    pub disagree: usize,
    /// Pairs of shadows where the flow's picture with no switch and being
    /// the only picture disagree (or the flow builds none, or a wrong one).
    pub built_disagree: usize,
    /// Pictures whose rows nest, and pictures where nesting rows and having
    /// no switch disagree.
    pub nested: usize,
    pub nested_disagree: usize,
    /// (picture, switch) pairs whose switch keeps the diagonal shadow.
    pub diagonal_kept: usize,
}

pub fn edges() -> Edges {
    let pictures = model::all_pictures(EDGE_SIDE, EDGE_SIDE);
    let mut groups: HashMap<Shadows, Vec<&Picture>> = HashMap::new();
    for p in &pictures {
        groups.entry(p.shadows()).or_default().push(p);
    }
    let (mut pairs, mut failed) = (0, 0);
    let (mut alone, mut no_switch, mut disagree) = (0, 0, 0);
    let (mut nested, mut nested_disagree) = (0, 0);
    let mut diagonal_kept = 0;
    for p in &pictures {
        let switches = model::switches(p);
        let diagonal = model::diagonal_counts(p);
        for s in &switches {
            pairs += 1;
            let kept = s
                .apply(p)
                .is_some_and(|q| q.shadows() == p.shadows() && p.differences(&q).len() == 4);
            failed += usize::from(!kept);
            diagonal_kept += usize::from(s.apply(p).is_some_and(|q| model::diagonal_counts(&q) == diagonal));
        }
        let only = groups[&p.shadows()].len() == 1;
        alone += usize::from(only);
        no_switch += usize::from(switches.is_empty());
        disagree += usize::from(only != switches.is_empty());
        let nests = model::rows_nested(p);
        nested += usize::from(nests);
        nested_disagree += usize::from(nests != switches.is_empty());
    }
    let connected = groups
        .values()
        .filter(|group| model::reachable(group[0]).len() == group.len())
        .count();
    let built_disagree = groups
        .iter()
        .filter(|(s, group)| match model::built_switches(s) {
            Some((p, found)) => !group.contains(&&p) || found.is_empty() != (group.len() == 1),
            None => true,
        })
        .count();
    Edges {
        pictures: pictures.len(),
        pairs,
        failed,
        shadow_pairs: groups.len(),
        connected,
        alone,
        no_switch,
        disagree,
        built_disagree,
        nested,
        nested_disagree,
        diagonal_kept,
    }
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let face = model::face();
    let shadows = face.shadows();
    let (rows, cols) = (&shadows.rows, &shadows.cols);
    let size = [face.rows(), face.cols()];
    f.pin("DT-F1.size", csv(&size), "5,5", "the chapter's grid")?;
    f.pin(
        "DT-F1.directions",
        [rows, cols].len(),
        "2",
        "two directions: a row shadow and a column shadow",
    )?;
    f.pin("DT-F1.face", face.pattern(), &model::FACE.join("/"), "model::FACE")?;
    f.add("DT-F1.face.rows", csv(rows))?;
    f.add("DT-F1.face.cols", csv(cols))?;
    f.add("DT-F1.face.total", face.count())?;
    let empty_rows: Vec<usize> = (0..rows.len()).filter(|&r| rows[r] == 0).collect();
    f.add("DT-F1.face.empty_rows", csv(&empty_rows))?;
    f.add("DT-F1.face.gale", flag(model::gale_ryser(&shadows)))?;

    let s = model::OPENING;
    let twin = twin()?;
    f.add("DT-F2.switch", format!("{};{}", csv(&s.rows), csv(&s.cols)))?;
    f.add("DT-F2.twin", twin.pattern())?;
    f.add("DT-F2.twin.rows", csv(&twin.row_counts()))?;
    f.add("DT-F2.twin.cols", csv(&twin.col_counts()))?;
    let differ = face.differences(&twin);
    f.add("DT-F2.differ", differ.len())?;
    let cells: Vec<String> = differ.iter().map(|&(r, c)| format!("{r},{c}")).collect();
    f.add("DT-F2.differ.cells", cells.join(";"))?;
    let changes = losses_and_gains(s, &face)?;
    let (loses, gains) = changes[0];
    if changes.iter().any(|&lg| lg != (loses, gains)) {
        return Err(format!("the switch's rows and columns change unevenly: {changes:?}"));
    }
    f.add("DT-F2.loses", loses)?;
    f.add("DT-F2.gains", gains)?;

    let fit = model::fits(&shadows);
    let gallery = gallery();
    let farthest = gallery.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let rings: Vec<usize> = (0..=farthest)
        .map(|k| gallery.iter().filter(|(_, d)| *d == k).count())
        .collect();
    let mut reached: Vec<Picture> = gallery.iter().map(|(p, _)| p.clone()).collect();
    reached.sort();
    f.pin("DT-F3.fits", fit.len(), "64", "oracle.py: column-by-column enumeration")?;
    f.add("DT-F3.face.switches", model::switches(&face).len())?;
    f.add("DT-F3.rings", csv(&rings))?;
    f.add("DT-F3.farthest", farthest)?;
    f.add("DT-F3.connected", flag(reached == fit))?;
    let (chain, end) = walk()?;
    f.add("DT-F3.walk.end", end.pattern())?;
    f.add("DT-F3.walk.steps", chain.len())?;

    let no_fit = no_fit();
    f.add("DT-F4.cols", csv(&no_fit.cols))?;
    f.add("DT-F4.totals", csv(&[no_fit.row_total(), no_fit.col_total()]))?;
    f.pin("DT-F4.fits", model::fits(&no_fit).len(), "0", "oracle.py: enumeration")?;
    f.add("DT-F4.gale", flag(model::gale_ryser(&no_fit)))?;
    let Reconstruction::Stuck {
        flow,
        total,
        bottleneck: cut,
    } = model::reconstruct(&no_fit)
    else {
        return Err("the flow fills the no-fit shadows".into());
    };
    if total != no_fit.col_total() {
        return Err(format!("the stuck flow's total {total} is not the shadows'"));
    }
    f.add("DT-F4.flow", flow)?;
    f.add("DT-F4.cut", csv(&cut))?;
    let (set, asks, can_give) = no_fit_bottleneck()?;
    f.add("DT-F4.bottleneck", csv(&set))?;
    f.add("DT-F4.bottleneck.asks", asks)?;
    f.add("DT-F4.bottleneck.can_give", can_give)?;

    f.add("DT-F5.face.flow", model::max_flow(&shadows).value)?;
    let built = match model::reconstruct(&shadows) {
        Reconstruction::Built(p) => p.shadows() == shadows && fit.contains(&p),
        Reconstruction::TotalsDiffer | Reconstruction::Stuck { .. } => false,
    };
    f.add("DT-F5.face.built", flag(built))?;

    let tree = model::tree();
    f.add("DT-F6.tree", tree.pattern())?;
    f.add("DT-F6.tree.rows", csv(&tree.row_counts()))?;
    f.add("DT-F6.tree.cols", csv(&tree.col_counts()))?;
    f.add("DT-F6.tree.fits", model::fits(&tree.shadows()).len())?;
    f.add("DT-F6.tree.switches", model::switches(&tree).len())?;

    // chapter 02: the water's real paths through the face's shadows
    let (paths, states) = face_flow()?;
    let k = first_takeback(&paths)?;
    let before = k
        .checked_sub(1)
        .map(|j| &states[j])
        .ok_or("the first path already takes water back")?;
    let steps = paths[k].steps();
    let (Some(&(stuck_row, into_col, true)), Some(&(from_row, back_col, false)), Some(&(_, to_col, true))) =
        (steps.first(), steps.get(1), steps.last())
    else {
        return Err(format!("the take-back path is not in, back and on: {steps:?}"));
    };
    if back_col != into_col || steps.len() != 3 {
        return Err(format!(
            "the take-back path does not take back where it went in: {steps:?}"
        ));
    }
    let room = paths[0].room;
    if paths.iter().any(|p| p.room != room) {
        return Err("the paths send different amounts".into());
    }
    let dry: Vec<usize> = (0..cols.len()).filter(|&c| !before.wet.get(stuck_row, c)).collect();
    if dry.iter().any(|&c| before.drained[c] < cols[c]) {
        return Err("a dry square of the stuck row leads to a drain with room".into());
    }
    let last = states.last().ok_or("the face's flow sends nothing")?;
    let from_face = gallery
        .iter()
        .find(|(p, _)| *p == last.wet)
        .map(|(_, d)| *d)
        .ok_or("the water's picture is not among the face's")?;
    f.pin("DT-F8.square", model::SQUARE, "1", "a square is filled or empty")?;
    f.add("DT-F8.paths", paths.len())?;
    f.add("DT-F8.room", room)?;
    let trace: Vec<String> = paths.iter().map(path_text).collect();
    f.add("DT-F8.trace", trace.join(","))?;
    f.add("DT-F8.direct", paths.iter().filter(|p| !p.takes_back()).count())?;
    f.add("DT-F8.takebacks", paths.iter().filter(|p| p.takes_back()).count())?;
    f.add("DT-F8.before.through", before.through())?;
    f.add("DT-F8.before.wet", before.wet.pattern())?;
    f.add("DT-F8.before.forward_room", flag(model::forward_room(&shadows, before)))?;
    f.add("DT-F8.stuck.row", stuck_row)?;
    f.add("DT-F8.stuck.left", rows[stuck_row] - before.sent[stuck_row])?;
    f.add("DT-F8.stuck.dry_full", csv(&dry))?;
    f.add("DT-F8.takeback.column", into_col)?;
    f.add("DT-F8.takeback.row", from_row)?;
    f.add("DT-F8.takeback.to_column", to_col)?;
    f.add("DT-F8.takeback.to_needs", cols[to_col] - before.drained[to_col])?;
    f.add("DT-F8.short", shadows.row_total() - before.through())?;
    f.add("DT-F8.through", last.through())?;
    f.add("DT-F8.built", last.wet.pattern())?;
    f.add("DT-F8.built.fits", flag(fit.contains(&last.wet)))?;
    f.add("DT-F8.built.is_face", flag(last.wet == face))?;
    f.add("DT-F8.built.only_one", flag(fit.len() == 1))?;
    f.add("DT-F8.built.from_face", from_face)?;

    // chapter 03: where the water stops on the no-fit shadows
    let (stop, stop_states) = no_fit_flow()?;
    let cut = stop.cut(&no_fit);
    if cut.value() != stop.value {
        return Err(format!("the cut {} is not the flow {}", cut.value(), stop.value));
    }
    if Some(&cut.cut_off) != model::bottleneck(&no_fit).as_ref() {
        return Err(format!(
            "the columns cut off, {:?}, are not the bottleneck",
            cut.cut_off
        ));
    }
    let end = stop_states.last().ok_or("the no-fit flow sends nothing")?;
    let left: Vec<usize> = (0..no_fit.rows.len())
        .filter(|&r| end.sent[r] < no_fit.rows[r])
        .collect();
    let (&[left_row], &[first]) = (left.as_slice(), cut.cut_off.as_slice()) else {
        return Err(format!(
            "not one row left with water and one column cut off: {left:?}, {cut:?}"
        ));
    };
    let stop_trace: Vec<String> = stop.paths.iter().map(path_text).collect();
    f.add("DT-F9.paths", stop.paths.len())?;
    f.add("DT-F9.trace", stop_trace.join(","))?;
    f.add("DT-F9.takebacks", stop.paths.iter().filter(|p| p.takes_back()).count())?;
    f.add("DT-F9.flow", stop.value)?;
    f.add("DT-F9.left.row", left_row)?;
    f.add("DT-F9.left.units", no_fit.rows[left_row] - end.sent[left_row])?;
    f.add(
        "DT-F9.first.wet",
        (0..no_fit.rows.len()).filter(|&r| end.wet.get(r, first)).count(),
    )?;
    f.add("DT-F9.first.needs", no_fit.cols[first])?;
    f.add("DT-F9.rows_with_water", no_fit.rows.iter().filter(|&&r| r > 0).count())?;
    f.add("DT-F9.reached.rows", csv(&cut.rows))?;
    f.add("DT-F9.reached.cols", csv(&cut.cols))?;
    f.add("DT-F9.cut.unreached_taps", cut.unreached_taps)?;
    f.add("DT-F9.cut.into_unreached", cut.pipes)?;
    f.add("DT-F9.cut.reached_drains", cut.reached_drains)?;
    f.add("DT-F9.cut", cut.value())?;
    for (key, s) in [("DT-F9", &no_fit), ("DT-F9.face", &shadows)] {
        let table = model::gale_table(s);
        let asks: Vec<usize> = table.iter().map(|&(asks, _)| asks).collect();
        let gives: Vec<usize> = table.iter().map(|&(_, gives)| gives).collect();
        f.add(&format!("{key}.sorted"), csv(&largest_first(&s.cols)))?;
        f.add(&format!("{key}.asks"), csv(&asks))?;
        f.add(&format!("{key}.gives"), csv(&gives))?;
        f.add(&format!("{key}.failing"), failing_sizes(&table))?;
    }

    // chapter 04: whether the water's picture is the only one, for the
    // face's shadows (chapter 02's water) and for the tree's
    let built = &last.wet;
    let built_switches = model::switches(built);
    let &first_switch = built_switches.first().ok_or("the water's picture has no switch")?;
    let second = first_switch
        .apply(built)
        .ok_or("the water's first switch does not apply")?;
    f.add("DT-F10.switch.rows", first_switch.rows.len())?;
    f.add("DT-F10.face.switches", built_switches.len())?;
    f.add(
        "DT-F10.face.first",
        format!("{};{}", csv(&first_switch.rows), csv(&first_switch.cols)),
    )?;
    f.add("DT-F10.face.second", second.pattern())?;
    f.add("DT-F10.face.second.differ", built.differences(&second).len())?;
    f.add(
        "DT-F10.face.second.fits",
        flag(second.shadows() == shadows && fit.contains(&second)),
    )?;
    let (tree_water, tree_states) = tree_flow()?;
    let tree_built = &tree_states.last().ok_or("the tree's flow sends nothing")?.wet;
    let tree_trace: Vec<String> = tree_water.paths.iter().map(path_text).collect();
    f.add("DT-F10.tree.total", tree.count())?;
    f.add("DT-F10.tree.paths", tree_water.paths.len())?;
    f.add("DT-F10.tree.trace", tree_trace.join(","))?;
    f.add(
        "DT-F10.tree.takebacks",
        tree_water.paths.iter().filter(|p| p.takes_back()).count(),
    )?;
    f.add("DT-F10.tree.built", tree_built.pattern())?;
    f.add("DT-F10.tree.built.is_tree", flag(*tree_built == tree))?;
    f.add("DT-F10.tree.built.switches", model::switches(tree_built).len())?;
    f.add("DT-F10.tree.rectangles", model::rectangles(tree_built))?;
    f.add("DT-F10.tree.nested", flag(model::rows_nested(tree_built)))?;

    // chapter 05: a third shadow, the diagonals, on the face's 64 pictures
    let face_diagonal = model::diagonal_counts(&face);
    let twin_diagonal = model::diagonal_counts(&twin);
    let diagonals = 0..face_diagonal.len();
    let lose: Vec<usize> = diagonals
        .clone()
        .filter(|&k| twin_diagonal[k] < face_diagonal[k])
        .collect();
    let gain: Vec<usize> = diagonals.filter(|&k| twin_diagonal[k] > face_diagonal[k]).collect();
    let &[gained] = gain.as_slice() else {
        return Err(format!("the opening switch gives to diagonals {gain:?}, not one"));
    };
    f.add("DT-F11.shadows", [rows, cols, &face_diagonal].len())?;
    f.add("DT-F11.diagonals", face_diagonal.len())?;
    f.add("DT-F11.face.diag", csv(&face_diagonal))?;
    f.add("DT-F11.twin.diag", csv(&twin_diagonal))?;
    f.add("DT-F11.twin.lose", csv(&lose))?;
    f.add("DT-F11.twin.losers", lose.len())?;
    f.add("DT-F11.twin.gain", gained)?;
    f.add("DT-F11.twin.gain.by", twin_diagonal[gained] - face_diagonal[gained])?;
    let keeping = model::switches(&face)
        .iter()
        .filter(|s| {
            s.apply(&face)
                .is_some_and(|q| model::diagonal_counts(&q) == face_diagonal)
        })
        .count();
    f.add("DT-F11.face.switches.keeping", keeping)?;
    let one_switch: Vec<&Picture> = gallery.iter().filter(|(_, d)| *d == 1).map(|(p, _)| p).collect();
    f.add("DT-F11.one_switch", one_switch.len())?;
    f.add(
        "DT-F11.one_switch.pass",
        one_switch
            .iter()
            .filter(|p| model::diagonal_counts(p) == face_diagonal)
            .count(),
    )?;
    let survivors = diagonal_survivors();
    let others: Vec<&Picture> = survivors.iter().filter(|p| **p != face).collect();
    let &[other] = others.as_slice() else {
        return Err(format!(
            "{} pictures other than the face cast its three shadows",
            others.len()
        ));
    };
    let other_from_face = gallery
        .iter()
        .find(|(p, _)| p == other)
        .map(|(_, d)| *d)
        .ok_or("the other survivor is not among the face's pictures")?;
    let changes = line_changes(&face, other);
    let &[(loses, gains), ..] = changes.as_slice() else {
        return Err("the face and the other survivor do not differ".into());
    };
    if changes.iter().any(|&lg| lg != (loses, gains)) {
        return Err(format!(
            "the lines through the six squares change unevenly: {changes:?}"
        ));
    }
    f.add("DT-F11.survivors", survivors.len())?;
    f.add("DT-F11.dropped", gallery.len() - survivors.len())?;
    f.add("DT-F11.others", others.len())?;
    f.add("DT-F11.other", other.pattern())?;
    f.add("DT-F11.other.from_face", other_from_face)?;
    f.add("DT-F11.other.differ", face.differences(other).len())?;
    f.add("DT-F11.other.lines", changes.len())?;
    f.add("DT-F11.other.loses", loses)?;
    f.add("DT-F11.other.gains", gains)?;
    f.add("DT-F11.only_one", flag(survivors.len() == 1))?;

    // chapter 02: reasoning one row or column at a time, on the tree's
    // shadows and on the face's
    let tree_shadows = tree.shadows();
    let tree_solve = model::line_solve(&tree_shadows);
    let [full, met, rest] = tree_reasoning()?;
    let told: Vec<Deduction> = full.iter().chain(&met).chain(&rest).cloned().collect();
    let full_counts: Vec<usize> = full.iter().map(|d| d.line.count(&tree_shadows)).collect();
    let met_counts: Vec<usize> = met.iter().map(|d| d.line.count(&tree_shadows)).collect();
    let met_cols: Vec<usize> = met
        .iter()
        .filter_map(|d| match d.line {
            Line::Col(c) => Some(c),
            Line::Row(_) => None,
        })
        .collect();
    let met_rows: Vec<usize> = met
        .iter()
        .filter_map(|d| match d.line {
            Line::Row(r) => Some(r),
            Line::Col(_) => None,
        })
        .collect();
    let [last_step] = rest.as_slice() else {
        return Err(format!("the tree's last deductions are {}, not one row", step_labels(&rest)));
    };
    let Line::Row(last_row) = last_step.line else {
        return Err(format!("the tree's last deduction is {}, not a row", last_step.label()));
    };
    let filled_before = tree_shadows.rows[last_row] - last_step.squares.len();
    f.add("DT-F12.tree.steps", step_labels(&tree_solve.steps))?;
    f.add("DT-F12.tree.told", step_labels(&told))?;
    f.add("DT-F12.tree.solved", flag(tree_solve.solved().as_ref() == Some(&tree)))?;
    f.add(
        "DT-F12.tree.full.lines",
        full.iter().map(|d| d.line.label()).collect::<Vec<_>>().join(","),
    )?;
    f.add("DT-F12.tree.full", common(&full_counts, "full lines' counts")?)?;
    f.add("DT-F12.tree.met.cols", csv(&met_cols))?;
    f.add("DT-F12.tree.met.rows", csv(&met_rows))?;
    f.add("DT-F12.tree.met.count", common(&met_counts, "met lines' counts")?)?;
    f.add("DT-F12.tree.last.row", last_row)?;
    f.add("DT-F12.tree.last.needs", tree_shadows.rows[last_row] - filled_before)?;
    f.add("DT-F12.tree.last.open", last_step.squares.len())?;
    let face_solve = model::line_solve(&shadows);
    let filled = filled_in();
    let total_pictures = gallery.len();
    let mut forced = 0;
    let mut free_filled = Vec::new();
    let mut free_empty = Vec::new();
    for row in &filled {
        for &n in row {
            if n == 0 || n == total_pictures {
                forced += 1;
            } else {
                free_filled.push(n);
                free_empty.push(total_pictures - n);
            }
        }
    }
    let on_face: Vec<((usize, usize), usize)> = face.filled().into_iter().map(|(r, c)| ((r, c), filled[r][c])).collect();
    let rarest = on_face.iter().map(|&(_, k)| k).min().ok_or("the face has no square")?;
    let rarest_squares: Vec<(usize, usize)> = on_face.iter().filter(|&&(_, k)| k == rarest).map(|&(rc, _)| rc).collect();
    f.add("DT-F12.face.steps", step_labels(&face_solve.steps))?;
    f.add("DT-F12.face.settled", face_solve.settled())?;
    f.add(
        "DT-F12.face.contradiction",
        face_solve.contradiction.map_or_else(|| "none".to_string(), Line::label),
    )?;
    f.add("DT-F12.face.forced", forced)?;
    f.add("DT-F12.face.free", free_filled.len())?;
    f.add(
        "DT-F12.face.filled_in",
        filled.iter().map(|row| csv(row.as_slice())).collect::<Vec<_>>().join("/"),
    )?;
    f.add("DT-F12.face.guess", csv(&[filled[0][0], total_pictures - filled[0][0]]))?;
    f.add(
        "DT-F12.face.free.least",
        csv(&[
            free_filled.iter().copied().min().unwrap_or(0),
            free_empty.iter().copied().min().unwrap_or(0),
        ]),
    )?;
    f.add("DT-F12.face.rarest", rarest)?;
    f.add("DT-F12.face.rarest.count", rarest_squares.len())?;
    f.add("DT-F12.face.rarest.squares", cells_text(&rarest_squares))?;

    // chapter 03: how many pictures a grid has
    let squares = face.rows() * face.cols();
    let pictures = u32::try_from(squares)
        .ok()
        .and_then(|n| 1u64.checked_shl(n))
        .ok_or("too many squares to count in a u64")?;
    f.pin("DT-F13.choices", [false, true].len(), "2", "a square is filled or empty")?;
    f.add("DT-F13.squares", squares)?;
    f.add("DT-F13.pictures", pictures)?;
    f.add("DT-F13.millions", pictures / 1_000_000)?;
    f.add("DT-F13.big.side", BIG_SIDE)?;
    f.add("DT-F13.big.digits", power_of_two_digits(BIG_SIDE * BIG_SIDE))?;

    // chapter 04: what the take-back did, read off the states before and
    // after it
    let after = &states[k];
    let changed = before.wet.differences(&after.wet);
    let emptied: Vec<(usize, usize)> = changed.iter().copied().filter(|&(r, c)| before.wet.get(r, c)).collect();
    let refilled: Vec<(usize, usize)> = changed.iter().copied().filter(|&(r, c)| after.wet.get(r, c)).collect();
    if before.sent[from_row] != after.sent[from_row] || before.drained[into_col] != after.drained[into_col] {
        return Err("the take-back changed the top row's or the first column's count".into());
    }
    f.add("DT-F14.emptied", emptied.len())?;
    f.add("DT-F14.emptied.cells", cells_text(&emptied))?;
    f.add("DT-F14.filled", refilled.len())?;
    f.add("DT-F14.filled.cells", cells_text(&refilled))?;
    f.add("DT-F14.top_row.sends", after.sent[from_row])?;
    f.add("DT-F14.first_col.drains", after.drained[into_col])?;
    f.add(
        "DT-F14.to_col.drains",
        csv(&[before.drained[to_col], after.drained[to_col]]),
    )?;
    f.add("DT-F14.searches", paths.len() + 1)?;

    // chapter 06: rows and columns 4, 4, 1, 1, 1
    let stall = stall();
    let stall_solve = model::line_solve(&stall);
    let pair = model::bottleneck(&stall).ok_or("the stall shadows have no bottleneck")?;
    let pair_asks: usize = pair.iter().map(|&j| stall.cols[j]).sum();
    let by_row: Vec<usize> = stall.rows.iter().map(|&r| r.min(pair.len())).collect();
    let all_counts: Vec<usize> = stall.rows.iter().chain(&stall.cols).copied().collect();
    let (stall_water, stall_states) = stall_flow()?;
    let stall_cut = stall_water.cut(&stall);
    if stall_cut.value() != stall_water.value || stall_cut.cut_off != pair {
        return Err(format!(
            "the stall flow's cut {stall_cut:?} is not its value or not the bottleneck {pair:?}"
        ));
    }
    let stall_end = stall_states.last().ok_or("the stall flow sends nothing")?;
    let stall_left: Vec<usize> = (0..stall.rows.len())
        .filter(|&r| stall_end.sent[r] < stall.rows[r])
        .collect();
    let &[stall_left_row] = stall_left.as_slice() else {
        return Err(format!("not one row left with water: {stall_left:?}"));
    };
    let stall_trace: Vec<String> = stall_water.paths.iter().map(path_text).collect();
    f.add("DT-F15.rows", csv(&stall.rows))?;
    f.add("DT-F15.cols", csv(&stall.cols))?;
    f.add("DT-F15.totals", csv(&[stall.row_total(), stall.col_total()]))?;
    f.add(
        "DT-F15.counts",
        csv(&[
            all_counts.iter().copied().min().unwrap_or(0),
            all_counts.iter().copied().max().unwrap_or(0),
        ]),
    )?;
    f.add("DT-F15.settled", stall_solve.settled())?;
    f.add(
        "DT-F15.contradiction",
        stall_solve.contradiction.map_or_else(|| "none".to_string(), Line::label),
    )?;
    f.add("DT-F15.rows_with_water", stall.rows.iter().filter(|&&r| r > 0).count())?;
    f.add("DT-F15.largest", stall.cols.iter().copied().max().unwrap_or(0))?;
    f.add("DT-F15.fits", model::fits(&stall).len())?;
    f.add("DT-F15.pair", csv(&pair))?;
    f.add("DT-F15.pair.asks", pair_asks)?;
    f.add("DT-F15.pair.by_row", csv(&by_row))?;
    f.add("DT-F15.pair.gives", model::can_give(&stall.rows, pair.len()))?;
    let stall_table = model::gale_table(&stall);
    f.add("DT-F15.sorted", csv(&largest_first(&stall.cols)))?;
    f.add(
        "DT-F15.asks",
        csv(&stall_table.iter().map(|&(asks, _)| asks).collect::<Vec<_>>()),
    )?;
    f.add(
        "DT-F15.gives",
        csv(&stall_table.iter().map(|&(_, gives)| gives).collect::<Vec<_>>()),
    )?;
    f.add("DT-F15.failing", failing_sizes(&stall_table))?;
    f.add("DT-F15.paths", stall_water.paths.len())?;
    f.add("DT-F15.trace", stall_trace.join(","))?;
    f.add(
        "DT-F15.takebacks",
        stall_water.paths.iter().filter(|p| p.takes_back()).count(),
    )?;
    f.add("DT-F15.flow", stall_water.value)?;
    f.add("DT-F15.left.row", stall_left_row)?;
    f.add("DT-F15.left.units", stall.rows[stall_left_row] - stall_end.sent[stall_left_row])?;
    f.add("DT-F15.reached.rows", csv(&stall_cut.rows))?;
    f.add("DT-F15.reached.cols", csv(&stall_cut.cols))?;
    f.add("DT-F15.cut_off", csv(&stall_cut.cut_off))?;
    f.add("DT-F15.cut.unreached_taps", stall_cut.unreached_taps)?;
    f.add("DT-F15.cut.pipes", stall_cut.pipes)?;
    f.add("DT-F15.cut.reached_drains", stall_cut.reached_drains)?;
    f.add("DT-F15.cut", stall_cut.value())?;
    f.add("DT-F15.short", stall.row_total() - stall_water.value)?;
    f.add(
        "DT-F15.other_pairs",
        OTHER_PAIRS.iter().map(|p| csv(&p[..])).collect::<Vec<_>>().join(";"),
    )?;
    f.add(
        "DT-F15.other_pairs.asks",
        csv(&OTHER_PAIRS
            .iter()
            .map(|p| p.iter().map(|&j| stall.cols[j]).sum::<usize>())
            .collect::<Vec<_>>()),
    )?;
    f.add(
        "DT-F15.other_pairs.gives",
        csv(&OTHER_PAIRS
            .iter()
            .map(|p| model::can_give(&stall.rows, p.len()))
            .collect::<Vec<_>>()),
    )?;

    // chapter 07: equal totals matter
    let unequal = unequal();
    f.add("DT-F16.rows", csv(&unequal.rows))?;
    f.add("DT-F16.cols", csv(&unequal.cols))?;
    f.add("DT-F16.totals", csv(&[unequal.row_total(), unequal.col_total()]))?;
    f.add("DT-F16.failing", failing_sizes(&model::gale_table(&unequal)))?;
    f.add("DT-F16.fits", model::fits(&unequal).len())?;

    // chapter 09: a chain can need more than one switch; the 64 from the
    // water's picture
    let (left, right) = perms()?;
    if left.shadows() != right.shadows() {
        return Err("chapter 09's two pictures cast different shadows".into());
    }
    let per_line: Vec<usize> = left.row_counts().into_iter().chain(left.col_counts()).collect();
    let chain = model::walk(&left, &right).ok_or("no chain of switches joins chapter 09's pictures")?;
    let &first_link = chain.first().ok_or("chapter 09's pictures are the same")?;
    let middle = first_link
        .apply(&left)
        .ok_or("the chain's first switch does not apply")?;
    let one_switch = model::switches(&left).iter().any(|s| s.apply(&left).as_ref() == Some(&right));
    let from_built = model::reachable(built);
    let built_far = from_built.iter().map(|(_, d)| *d).max().unwrap_or(0);
    let built_rings: Vec<usize> = (0..=built_far)
        .map(|d| from_built.iter().filter(|(_, e)| *e == d).count())
        .collect();
    let mut from_built_pictures: Vec<Picture> = from_built.into_iter().map(|(p, _)| p).collect();
    from_built_pictures.sort();
    if from_built_pictures != fit {
        return Err("switches from the water's picture do not reach exactly the 64".into());
    }
    f.add("DT-F17.perm.left", left.pattern())?;
    f.add("DT-F17.perm.right", right.pattern())?;
    f.add("DT-F17.perm.per_line", common(&per_line, "row and column counts")?)?;
    f.add("DT-F17.perm.differ", left.differences(&right).len())?;
    f.add("DT-F17.perm.one_switch", flag(one_switch))?;
    f.add("DT-F17.perm.distance", chain.len())?;
    f.add(
        "DT-F17.chain",
        chain.iter().map(|&s| switch_text(s)).collect::<Vec<_>>().join("|"),
    )?;
    f.add("DT-F17.chain.middle", middle.pattern())?;
    f.add("DT-F17.rings_from_built", csv(&built_rings))?;

    // chapter 10: the 64 grouped by diagonal counts; the other diagonals
    let groups = diagonal_groups();
    let face_group = groups
        .iter()
        .find(|g| g.contains(&face))
        .ok_or("the face is in no group")?;
    let falling = falling_survivors();
    let falling_others: Vec<&Picture> = falling.iter().filter(|p| **p != face).collect();
    let &[falling_other] = falling_others.as_slice() else {
        return Err(format!(
            "{} pictures other than the face cast its other diagonal shadow",
            falling_others.len()
        ));
    };
    let falling_from_face = gallery
        .iter()
        .find(|(p, _)| p == falling_other)
        .map(|(_, d)| *d)
        .ok_or("the other diagonals' survivor is not among the face's pictures")?;
    let both = both_survivors();
    f.add("DT-F18.groups", groups.len())?;
    f.add("DT-F18.alone", groups.iter().filter(|g| g.len() == 1).count())?;
    f.add("DT-F18.pairs", groups.iter().filter(|g| g.len() == 2).count())?;
    f.add("DT-F18.largest", groups.iter().map(Vec::len).max().unwrap_or(0))?;
    f.add("DT-F18.face.group", face_group.len())?;
    f.add("DT-F18.face.alone", flag(face_group.len() == 1))?;
    f.add("DT-F18.face.mirror", flag(model::mirrored(&face) == face))?;
    f.add("DT-F18.falling", csv(&model::falling_counts(&face)))?;
    f.add("DT-F18.falling.survivors", falling.len())?;
    f.add("DT-F18.falling.other", falling_other.pattern())?;
    f.add("DT-F18.falling.other.from_face", falling_from_face)?;
    f.add("DT-F18.falling.other.differ", face.differences(falling_other).len())?;
    f.add(
        "DT-F18.falling.other.mirror",
        flag(model::mirrored(other) == *falling_other),
    )?;
    f.add("DT-F18.shadows", [rows, cols, &face_diagonal, &model::falling_counts(&face)].len())?;
    f.add(
        "DT-F18.both",
        both.iter().map(Picture::pattern).collect::<Vec<_>>().join(","),
    )?;
    f.add("DT-F18.both.count", both.len())?;

    // chapter 11: the steps a program follows
    f.add("DT-F19.steps", STEPS.len())?;

    f.add("DT-F7.questions", QUESTIONS.len())?;
    f.pin(
        "DT-S1.ryser",
        RYSER_YEAR,
        "1957",
        "H. J. Ryser, Canad. J. Math. 9 (1957) 371-377",
    )?;
    f.pin(
        "DT-S2.ford_fulkerson",
        FORD_FULKERSON_YEAR,
        "1957",
        "L. R. Ford Jr. and D. R. Fulkerson, Canad. J. Math. 9 (1957) 210-218",
    )?;
    f.pin(
        "DT-S3.gale",
        GALE_YEAR,
        "1957",
        "D. Gale, Pacific J. Math. 7 (1957) 1073-1082",
    )?;

    f.pin(
        "DT-S4.batenburg",
        BATENBURG_YEAR,
        "2006",
        "K. J. Batenburg, Network Flow Algorithms for Discrete Tomography, PhD thesis, Leiden 2006, 1.9.1",
    )?;

    let e = edges();
    f.add("DT-X1.pairs", e.pairs)?;
    f.pin("DT-X1.failed", e.failed, "0", "every switch keeps both shadows")?;
    f.add("DT-X2.shadow_pairs", e.shadow_pairs)?;
    f.add("DT-X2.connected", e.connected)?;
    f.pin(
        "DT-X2.disconnected",
        e.shadow_pairs - e.connected,
        "0",
        "Ryser's Theorem 3.1 on every 4x4 pair of shadows",
    )?;
    f.add("DT-X3.grid", csv(&[EDGE_SIDE, EDGE_SIDE]))?;
    f.add("DT-X3.pictures", e.pictures)?;
    f.add("DT-X3.alone", e.alone)?;
    // the share alone, rounded to tenths: "about one picture in ten"
    f.add("DT-X3.alone.tenths", (10 * e.alone + e.pictures / 2) / e.pictures)?;
    f.add("DT-X3.no_switch", e.no_switch)?;
    f.pin("DT-X3.disagree", e.disagree, "0", "Ryser: alone exactly when no switch")?;
    f.pin(
        "DT-X4.disagree",
        e.built_disagree,
        "0",
        "the flow's picture has no switch exactly when it is the only one",
    )?;
    f.add("DT-X5.nested", e.nested)?;
    f.pin(
        "DT-X5.disagree",
        e.nested_disagree,
        "0",
        "no switch exactly when the rows nest",
    )?;
    f.pin(
        "DT-X6.kept",
        e.diagonal_kept,
        "0",
        "no switch keeps the diagonal shadow",
    )?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facts_hold_their_pins() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert_eq!(f.get("DT-F1.face.rows"), Some("2,2,0,2,3"));
        assert_eq!(f.get("DT-F1.face.cols"), Some("1,3,1,3,1"));
        assert_eq!(f.get("DT-F2.twin"), Some("00011/01010/00000/11000/01110"));
        assert_eq!(f.get("DT-F3.rings"), Some("1,14,33,16"));
        assert_eq!(f.get("DT-F4.cut"), f.get("DT-F4.bottleneck"));
        assert_eq!(f.get("DT-X2.connected"), f.get("DT-X2.shadow_pairs"));
        assert_eq!(f.get("DT-X3.alone"), f.get("DT-X3.no_switch"));
    }

    /// The twin has the face's shadows and differs from it exactly at the
    /// opening switch's corners.
    #[test]
    fn the_twin_differs_from_the_face_at_the_corners() {
        let (face, twin) = (model::face(), twin().unwrap());
        assert_eq!(twin.shadows(), face.shadows());
        let mut corners = model::OPENING.corners().to_vec();
        corners.sort_unstable();
        assert_eq!(face.differences(&twin), corners);
    }

    /// The walk starts with the opening switch, every step is a switch, and
    /// it ends as far from the face as any picture gets.
    #[test]
    fn the_walk_reaches_a_farthest_picture() {
        let (chain, end) = walk().unwrap();
        assert_eq!(chain[0], model::OPENING);
        let mut at = model::face();
        for s in &chain {
            at = s.apply(&at).unwrap();
        }
        assert_eq!(at, end);
        let farthest = gallery().iter().map(|(_, d)| *d).max().unwrap();
        assert_eq!(chain.len(), farthest);
    }

    /// Chapter 02's numbers: nine one-unit paths, eight straight down and one
    /// that takes water back; stuck at eight without it; a picture that fits
    /// and is not the face.
    #[test]
    fn the_flow_trace_facts_hold() {
        let f = facts().unwrap();
        assert_eq!(
            f.get("DT-F8.trace"),
            Some("r0c0,r0c1,r1c1,r1c2,r3c1,r3c3,r4c3,r4c4,r4c0r0c3")
        );
        assert_eq!(f.get("DT-F8.before.forward_room"), Some("false"));
        assert_eq!(f.get("DT-F8.through"), f.get("DT-F5.face.flow"));
        assert_eq!(f.get("DT-F8.built.fits"), Some("true"));
        assert_eq!(f.get("DT-F8.built.is_face"), Some("false"));
        let (paths, states) = face_flow().unwrap();
        let k = first_takeback(&paths).unwrap();
        assert_eq!(states[k - 1].through() + 1, states[k].through());
        // every state the chapter draws keeps each row and column within its count
        let s = model::face().shadows();
        for st in &states {
            assert!(st.sent.iter().zip(&s.rows).all(|(a, b)| a <= b));
            assert!(st.drained.iter().zip(&s.cols).all(|(a, b)| a <= b));
            assert_eq!(st.sent, st.wet.row_counts());
            assert_eq!(st.drained, st.wet.col_counts());
        }
    }

    /// Chapter 03's numbers: the stop is the cut, worth the flow; the cut
    /// cuts off exactly the bottleneck column; Gale's table fails only at
    /// the first size for these shadows, and nowhere for the face's.
    #[test]
    fn the_stop_facts_hold() {
        let f = facts().unwrap();
        assert_eq!(f.get("DT-F9.flow"), f.get("DT-F4.flow"));
        assert_eq!(f.get("DT-F9.cut"), f.get("DT-F9.flow"));
        assert_eq!(f.get("DT-F9.trace"), Some("r0c0,r0c1,r1c0,r1c2,r3c0,r3c3,r4c0,r4c4"));
        assert_eq!(f.get("DT-F9.reached.rows"), Some("0,1,3,4"));
        assert_eq!(f.get("DT-F9.reached.cols"), Some("1,2,3,4"));
        assert_eq!(f.get("DT-F9.asks"), Some("5,6,7,8,9"));
        assert_eq!(f.get("DT-F9.gives"), Some("4,8,9,9,9"));
        assert_eq!(f.get("DT-F9.failing"), Some("1"));
        assert_eq!(f.get("DT-F9.face.failing"), Some("none"));
        assert_eq!(f.get("DT-F9.first.wet"), f.get("DT-F9.cut.into_unreached"));
        // every state of the no-fit flow keeps each row and column in count
        let (_, states) = no_fit_flow().unwrap();
        let s = no_fit();
        for st in &states {
            assert!(st.sent.iter().zip(&s.rows).all(|(a, b)| a <= b));
            assert!(st.drained.iter().zip(&s.cols).all(|(a, b)| a <= b));
        }
    }

    /// Chapter 04's numbers: the face's water picture has switches, and its
    /// first makes a second picture with the same shadows, four squares
    /// away; the tree's water, eleven units with one taken back, builds the
    /// tree itself, whose rows nest, with no switch among its 100
    /// rectangles; and on every 4x4 pair of shadows the water's picture
    /// settles whether it is the only one.
    #[test]
    fn the_only_picture_facts_hold() {
        let f = facts().unwrap();
        assert_eq!(f.get("DT-F10.face.switches"), Some("12"));
        assert_eq!(f.get("DT-F10.face.first"), Some("0,1;2,3"));
        assert_eq!(f.get("DT-F10.face.second"), Some("01100/01010/00000/01010/10011"));
        assert_eq!(f.get("DT-F10.face.second.differ"), Some("4"));
        assert_eq!(f.get("DT-F10.face.second.fits"), Some("true"));
        assert_eq!(
            f.get("DT-F10.tree.trace"),
            Some("r0c0,r1c1,r1c2,r1c3,r2c1,r2c2,r2c3,r2c4,r3c2,r4c2,r2c0r0c2")
        );
        assert_eq!(f.get("DT-F10.tree.takebacks"), Some("1"));
        assert_eq!(f.get("DT-F10.tree.built"), f.get("DT-F6.tree"));
        assert_eq!(f.get("DT-F10.tree.built.switches"), Some("0"));
        assert_eq!(f.get("DT-F10.tree.rectangles"), Some("100"));
        assert_eq!(f.get("DT-F10.tree.nested"), Some("true"));
        assert_eq!(f.get("DT-F6.tree.fits"), Some("1"));
        assert_eq!(f.get("DT-X4.disagree"), Some("0"));
        assert_eq!(f.get("DT-X5.nested"), f.get("DT-X3.no_switch"));
        // the tree's flow keeps every row and column in count
        let (_, states) = tree_flow().unwrap();
        let s = model::tree().shadows();
        for st in &states {
            assert!(st.sent.iter().zip(&s.rows).all(|(a, b)| a <= b));
            assert!(st.drained.iter().zip(&s.cols).all(|(a, b)| a <= b));
        }
    }

    /// Chapter 05's numbers: the opening switch takes a square from two
    /// diagonals and gives both to a third; none of the face's switches
    /// keeps its diagonal shadow; two of the 64 cast it, the face and one
    /// picture two switches away, six squares apart, every line through them
    /// losing one and gaining one; no switch of any 4x4 picture keeps it.
    #[test]
    fn the_third_shadow_facts_hold() {
        let f = facts().unwrap();
        assert_eq!(f.get("DT-F11.face.diag"), Some("0,1,1,2,1,1,1,2,0"));
        assert_eq!(f.get("DT-F11.twin.diag"), Some("0,0,1,2,3,1,1,1,0"));
        assert_eq!(f.get("DT-F11.twin.lose"), Some("1,7"));
        assert_eq!(f.get("DT-F11.twin.gain"), Some("4"));
        assert_eq!(f.get("DT-F11.twin.gain.by"), Some("2"));
        assert_eq!(f.get("DT-F11.face.switches.keeping"), Some("0"));
        assert_eq!(f.get("DT-F11.one_switch"), f.get("DT-F3.face.switches"));
        assert_eq!(f.get("DT-F11.one_switch.pass"), Some("0"));
        assert_eq!(f.get("DT-F11.survivors"), Some("2"));
        assert_eq!(f.get("DT-F11.dropped"), Some("62"));
        assert_eq!(f.get("DT-F11.other"), Some("01010/01100/00000/00011/11010"));
        assert_eq!(f.get("DT-F11.other.from_face"), Some("2"));
        assert_eq!(f.get("DT-F11.other.differ"), Some("6"));
        assert_eq!(f.get("DT-F11.other.lines"), Some("9"));
        assert_eq!(f.get("DT-X6.kept"), Some("0"));
        let survivors = diagonal_survivors();
        let face = model::face();
        assert!(survivors.contains(&face));
        for p in &survivors {
            assert_eq!(p.shadows(), face.shadows());
            assert_eq!(model::diagonal_counts(p), model::diagonal_counts(&face));
        }
        // the line changes of a switch: its two rows and two columns, and
        // the three diagonals its corners stand on
        let twin = twin().unwrap();
        let changes = line_changes(&face, &twin);
        assert_eq!(changes.len(), 2 + 2 + 3);
        assert!(changes[..4].iter().all(|&lg| lg == (1, 1)));
    }

    /// The expansion's numbers (chapters 02, 03, 04, 06, 07, 09, 10, 11), as
    /// `check_candidate.py` computed them independently before the model
    /// did: the tree settled line by line, the face only in its empty row;
    /// 2^25 pictures and 121 digits; the take-back's three squares; the
    /// stall shadows' pair, table, water and cut; the unequal totals; the
    /// 3x3 chain; the 64 grouped by their diagonals, the other diagonals.
    #[test]
    fn the_expansion_facts_hold() {
        let f = facts().unwrap();
        let get = |key: &str| f.get(key).unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(get("DT-F12.tree.steps"), "r2+5,c0-4,c2+4,c4-4,r0-2,r1+2,r3-2,r4-2");
        assert_eq!(get("DT-F12.tree.told"), "r2+5,c2+4,c0-4,c4-4,r0-2,r3-2,r4-2,r1+2");
        assert_eq!(get("DT-F12.tree.full.lines"), "r2,c2");
        assert_eq!(get("DT-F12.tree.full"), "5");
        assert_eq!(get("DT-F12.tree.met.cols"), "0,4");
        assert_eq!(get("DT-F12.tree.met.rows"), "0,3,4");
        assert_eq!(get("DT-F12.tree.met.count"), "1");
        assert_eq!(get("DT-F12.tree.last.row"), "1");
        assert_eq!(get("DT-F12.tree.last.needs"), get("DT-F12.tree.last.open"));
        assert_eq!(get("DT-F12.face.steps"), "r2-5");
        assert_eq!(get("DT-F12.face.forced"), get("DT-F12.face.settled"));
        assert_eq!(get("DT-F12.face.free"), "20");
        assert_eq!(
            get("DT-F12.face.filled_in"),
            "12,46,12,46,12/12,46,12,46,12/0,0,0,0,0/12,46,12,46,12/28,54,28,54,28"
        );
        assert_eq!(get("DT-F12.face.guess"), "12,52");
        assert_eq!(get("DT-F12.face.rarest.squares"), "3,0;3,4");
        assert_eq!(get("DT-F13.pictures"), "33554432");
        assert_eq!(get("DT-F13.millions"), "33");
        assert_eq!(get("DT-F13.big.digits"), "121");
        assert_eq!(power_of_two_digits(10), 4);
        assert_eq!(power_of_two_digits(0), 1);
        assert_eq!(get("DT-F14.emptied.cells"), "0,0");
        assert_eq!(get("DT-F14.filled.cells"), "0,3;4,0");
        assert_eq!(get("DT-F14.to_col.drains"), "2,3");
        assert_eq!(get("DT-F14.searches"), "10");
        assert_eq!(get("DT-F15.settled"), "0");
        assert_eq!(get("DT-F15.contradiction"), "none");
        assert_eq!(get("DT-F15.fits"), "0");
        assert_eq!(get("DT-F15.pair"), "0,1");
        assert_eq!(get("DT-F15.pair.by_row"), "2,2,1,1,1");
        assert_eq!(get("DT-F15.failing"), "2");
        assert_eq!(get("DT-F15.flow"), "10");
        assert_eq!(get("DT-F15.reached.rows"), "0,1");
        assert_eq!(get("DT-F15.reached.cols"), "2,3,4");
        assert_eq!(get("DT-F15.cut_off"), get("DT-F15.pair"));
        assert_eq!(get("DT-F15.cut"), get("DT-F15.flow"));
        assert_eq!(get("DT-F16.failing"), "none");
        assert_eq!(get("DT-F16.fits"), "0");
        assert_eq!(get("DT-F17.chain"), "0,1;0,1|1,2;0,2");
        assert_eq!(get("DT-F17.chain.middle"), "010/100/001");
        assert_eq!(get("DT-F17.rings_from_built"), "1,12,37,14");
        assert_eq!(get("DT-F18.groups"), "55");
        assert_eq!(get("DT-F18.alone"), "48");
        assert_eq!(get("DT-F18.falling.other"), "01010/00110/00000/11000/01011");
        assert_eq!(get("DT-F18.both"), get("DT-F1.face"));
        assert_eq!(get("DT-X3.alone.tenths"), "1");
        // the stall flow keeps every row and column in count
        let (_, states) = stall_flow().unwrap();
        let s = stall();
        for st in &states {
            assert!(st.sent.iter().zip(&s.rows).all(|(a, b)| a <= b));
            assert!(st.drained.iter().zip(&s.cols).all(|(a, b)| a <= b));
        }
    }

    /// The gallery is every picture that fits, each exactly once.
    #[test]
    fn the_gallery_is_every_picture_that_fits() {
        let mut pictures: Vec<Picture> = gallery().into_iter().map(|(p, _)| p).collect();
        pictures.sort();
        assert_eq!(pictures, model::fits(&model::face().shadows()));
    }
}
