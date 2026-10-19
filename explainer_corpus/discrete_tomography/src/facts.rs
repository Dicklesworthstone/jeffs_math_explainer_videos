//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x discrete_tomography facts-check`): the rows
//! of docs/fact-sheet.md. DT-F rows are computed, DT-S rows carry a source's
//! literal, DT-X rows are the edges of the always/never sentences, checked on
//! every 4x4 picture.

use std::collections::HashMap;
use std::fmt::Display;

use explainer_kit::Facts;

use crate::chapters::same_shadows::QUESTIONS;
use crate::model::{self, Augmenting, Flow, FlowState, Node, Picture, Reconstruction, Shadows, Switch};

/// The year of Ryser's paper, shown on chapter 01's "precisely" card: H. J.
/// Ryser, "Combinatorial properties of matrices of zeros and ones",
/// Canadian Journal of Mathematics 9 (1957) 371–377, Theorem 3.1.
pub const RYSER_YEAR: u32 = 1957;

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
}

pub fn edges() -> Edges {
    let pictures = model::all_pictures(4, 4);
    let mut groups: HashMap<Shadows, Vec<&Picture>> = HashMap::new();
    for p in &pictures {
        groups.entry(p.shadows()).or_default().push(p);
    }
    let (mut pairs, mut failed) = (0, 0);
    let (mut alone, mut no_switch, mut disagree) = (0, 0, 0);
    for p in &pictures {
        let switches = model::switches(p);
        for s in &switches {
            pairs += 1;
            let kept = s
                .apply(p)
                .is_some_and(|q| q.shadows() == p.shadows() && p.differences(&q).len() == 4);
            failed += usize::from(!kept);
        }
        let only = groups[&p.shadows()].len() == 1;
        alone += usize::from(only);
        no_switch += usize::from(switches.is_empty());
        disagree += usize::from(only != switches.is_empty());
    }
    let connected = groups
        .values()
        .filter(|group| model::reachable(group[0]).len() == group.len())
        .count();
    Edges {
        pairs,
        failed,
        shadow_pairs: groups.len(),
        connected,
        alone,
        no_switch,
        disagree,
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

    let e = edges();
    f.add("DT-X1.pairs", e.pairs)?;
    f.pin("DT-X1.failed", e.failed, "0", "every switch keeps both shadows")?;
    f.add("DT-X2.shadow_pairs", e.shadow_pairs)?;
    f.add("DT-X2.connected", e.connected)?;
    f.add("DT-X3.alone", e.alone)?;
    f.add("DT-X3.no_switch", e.no_switch)?;
    f.pin("DT-X3.disagree", e.disagree, "0", "Ryser: alone exactly when no switch")?;
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

    /// The gallery is every picture that fits, each exactly once.
    #[test]
    fn the_gallery_is_every_picture_that_fits() {
        let mut pictures: Vec<Picture> = gallery().into_iter().map(|(p, _)| p).collect();
        pictures.sort();
        assert_eq!(pictures, model::fits(&model::face().shadows()));
    }
}
