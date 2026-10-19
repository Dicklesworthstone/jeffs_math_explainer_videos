//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x discrete_tomography facts-check`): the rows
//! of docs/fact-sheet.md. DT-F rows are computed, DT-S rows carry a source's
//! literal, DT-X rows are the edges of the always/never sentences, checked on
//! every 4x4 picture.

use std::collections::HashMap;
use std::fmt::Display;

use explainer_kit::Facts;

use crate::chapters::same_shadows::QUESTIONS;
use crate::model::{self, Picture, Reconstruction, Shadows, Switch};

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

    f.add("DT-F7.questions", QUESTIONS.len())?;
    f.pin(
        "DT-S1.ryser",
        RYSER_YEAR,
        "1957",
        "H. J. Ryser, Canad. J. Math. 9 (1957) 371-377",
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

    /// The gallery is every picture that fits, each exactly once.
    #[test]
    fn the_gallery_is_every_picture_that_fits() {
        let mut pictures: Vec<Picture> = gallery().into_iter().map(|(p, _)| p).collect();
        pictures.sort();
        assert_eq!(pictures, model::fits(&model::face().shadows()));
    }
}
