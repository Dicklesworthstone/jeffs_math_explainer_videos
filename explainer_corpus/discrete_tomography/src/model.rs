//! The mathematics of the video, with no drawing (chapters only draw). A
//! picture is a grid of squares, each filled or empty; its two shadows are
//! how many squares are filled in each row and in each column. Here are the
//! switch, which changes a picture and neither shadow; every picture that
//! fits a pair of shadows; walks between them by switches; and Gale's network
//! of pipes, whose largest flow builds a picture that fits or shows that none
//! does.
//!
//! The literature's words, for programmers and the description: a picture is
//! a binary (0-1) matrix, its shadows are its row and column sums (its
//! horizontal and vertical projections), a switch is Ryser's interchange
//! (H. J. Ryser, Canadian Journal of Mathematics 9, 1957), and Gale's network
//! is D. Gale's (Pacific Journal of Mathematics 7, 1957).

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

/// Chapter 01's face: two eyes, an empty middle row, a smile.
pub const FACE: [&str; 5] = ["01010", "01010", "00000", "10001", "01110"];

/// A tree whose shadows no other picture casts: it has no switch.
pub const TREE: [&str; 5] = ["00100", "01110", "11111", "00100", "00100"];

/// The switch chapter 01 shows first: rows 0 and 3, columns 1 and 4.
pub const OPENING: Switch = Switch {
    rows: [0, 3],
    cols: [1, 4],
};

/// Column counts that, with the face's row counts, add up to the same total
/// and still fit no picture (chapter 01's last example).
pub const NO_FIT_COLS: [usize; 5] = [5, 1, 1, 1, 1];

// ------------------------------------------------------------------ pictures

/// A picture: `rows` × `cols` squares, filled or empty, stored row by row.
/// Pictures of one size are ordered as their patterns are (row 0 first,
/// empty before filled), the order `oracle.py` sorts pattern strings in.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Picture {
    rows: usize,
    cols: usize,
    filled: Vec<bool>,
}

impl Picture {
    /// A picture with every square empty.
    pub fn empty(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            filled: vec![false; rows * cols],
        }
    }

    /// A picture from its rows, `'1'` filled and `'0'` empty, top row first.
    pub fn parse(rows: &[&str]) -> Result<Self, String> {
        let cols = rows.first().map_or(0, |r| r.chars().count());
        if cols == 0 {
            return Err("a picture needs at least one row and one column".into());
        }
        let mut filled = Vec::with_capacity(rows.len() * cols);
        for (i, row) in rows.iter().enumerate() {
            if row.chars().count() != cols {
                return Err(format!("row {i} is {row:?}, not {cols} squares"));
            }
            for ch in row.chars() {
                match ch {
                    '1' => filled.push(true),
                    '0' => filled.push(false),
                    _ => return Err(format!("row {i} has {ch:?}, not 0 or 1")),
                }
            }
        }
        Ok(Self {
            rows: rows.len(),
            cols,
            filled,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Whether square (`r`, `c`) is filled; row 0 is the top row.
    ///
    /// # Panics
    /// Outside the picture.
    pub fn get(&self, r: usize, c: usize) -> bool {
        assert!(
            r < self.rows && c < self.cols,
            "square ({r}, {c}) is outside a {}x{} picture",
            self.rows,
            self.cols
        );
        self.filled[r * self.cols + c]
    }

    fn flip(&mut self, r: usize, c: usize) {
        let i = r * self.cols + c;
        self.filled[i] = !self.filled[i];
    }

    /// The filled squares, row by row.
    pub fn filled(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.get(r, c) {
                    out.push((r, c));
                }
            }
        }
        out
    }

    /// How many squares are filled.
    pub fn count(&self) -> usize {
        self.filled.iter().filter(|&&f| f).count()
    }

    /// How many squares each row holds, top to bottom.
    pub fn row_counts(&self) -> Vec<usize> {
        (0..self.rows)
            .map(|r| (0..self.cols).filter(|&c| self.get(r, c)).count())
            .collect()
    }

    /// How many squares each column holds, left to right.
    pub fn col_counts(&self) -> Vec<usize> {
        (0..self.cols)
            .map(|c| (0..self.rows).filter(|&r| self.get(r, c)).count())
            .collect()
    }

    /// Both shadows.
    pub fn shadows(&self) -> Shadows {
        Shadows {
            rows: self.row_counts(),
            cols: self.col_counts(),
        }
    }

    /// The pattern, rows joined by `/`: "01010/01010/00000/10001/01110".
    pub fn pattern(&self) -> String {
        (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| if self.get(r, c) { '1' } else { '0' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    /// The squares where two pictures of one size differ, row by row.
    ///
    /// # Panics
    /// On pictures of different sizes.
    pub fn differences(&self, other: &Picture) -> Vec<(usize, usize)> {
        assert_eq!(
            (self.rows, self.cols),
            (other.rows, other.cols),
            "pictures of different sizes"
        );
        let mut out = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.get(r, c) != other.get(r, c) {
                    out.push((r, c));
                }
            }
        }
        out
    }
}

/// Chapter 01's face.
///
/// # Panics
/// Never: [`FACE`] is a pattern of 0s and 1s (tested).
pub fn face() -> Picture {
    Picture::parse(&FACE).expect("FACE is a pattern of 0s and 1s")
}

/// The tree.
///
/// # Panics
/// Never: [`TREE`] is a pattern of 0s and 1s (tested).
pub fn tree() -> Picture {
    Picture::parse(&TREE).expect("TREE is a pattern of 0s and 1s")
}

/// Every picture of `rows` × `cols` squares, in increasing order.
///
/// # Panics
/// Past 20 squares (2^20 pictures).
pub fn all_pictures(rows: usize, cols: usize) -> Vec<Picture> {
    let squares = rows * cols;
    assert!(squares <= 20, "{rows}x{cols} has 2^{squares} pictures");
    let mut out: Vec<Picture> = (0u32..1 << squares)
        .map(|mask| Picture {
            rows,
            cols,
            filled: (0..squares).map(|i| (mask >> i) & 1 == 1).collect(),
        })
        .collect();
    out.sort();
    out
}

// ------------------------------------------------------------------ shadows

/// A pair of shadows: how many filled squares each row holds, top to
/// bottom, and each column holds, left to right.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Shadows {
    pub rows: Vec<usize>,
    pub cols: Vec<usize>,
}

impl Shadows {
    pub fn new(rows: &[usize], cols: &[usize]) -> Self {
        Self {
            rows: rows.to_vec(),
            cols: cols.to_vec(),
        }
    }

    /// The squares the row shadow counts.
    pub fn row_total(&self) -> usize {
        self.rows.iter().sum()
    }

    /// The squares the column shadow counts.
    pub fn col_total(&self) -> usize {
        self.cols.iter().sum()
    }
}

/// Every picture that fits the shadows, in increasing order: row by row,
/// each row any choice of its count among the columns with room left,
/// keeping the pictures whose columns all end exactly full.
///
/// # Panics
/// Past 20 columns.
pub fn fits(s: &Shadows) -> Vec<Picture> {
    let n = s.cols.len();
    assert!(n <= 20, "{n} columns");
    let mut out = Vec::new();
    if s.row_total() != s.col_total() {
        return out;
    }
    let mut room = s.cols.clone();
    let mut picture = Picture::empty(s.rows.len(), n);
    fill_row(s, 0, &mut room, &mut picture, &mut out);
    out.sort();
    out
}

fn fill_row(s: &Shadows, r: usize, room: &mut [usize], picture: &mut Picture, out: &mut Vec<Picture>) {
    if r == s.rows.len() {
        if room.iter().all(|&left| left == 0) {
            out.push(picture.clone());
        }
        return;
    }
    let n = s.cols.len();
    for mask in 0u32..1 << n {
        if mask.count_ones() as usize != s.rows[r] {
            continue;
        }
        let chosen: Vec<usize> = (0..n).filter(|&c| (mask >> c) & 1 == 1).collect();
        if chosen.iter().any(|&c| room[c] == 0) {
            continue;
        }
        for &c in &chosen {
            room[c] -= 1;
            picture.flip(r, c);
        }
        fill_row(s, r + 1, room, picture, out);
        for &c in &chosen {
            room[c] += 1;
            picture.flip(r, c);
        }
    }
}

// ------------------------------------------------------------------ switches

/// A switch: two rows and two columns, each pair in increasing order, whose
/// four corner squares alternate around the rectangle they span (filled,
/// empty, filled, empty). Swapping filled and empty there changes the
/// picture and neither shadow: each of the two rows, and each of the two
/// columns, loses one filled square and gains one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Switch {
    pub rows: [usize; 2],
    pub cols: [usize; 2],
}

/// A switch's corners in one picture: the two it empties (filled before)
/// and the two it fills (empty before).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sides {
    pub emptied: [(usize, usize); 2],
    pub filled: [(usize, usize); 2],
}

impl Switch {
    /// Its four corners, around the rectangle: top left, top right, bottom
    /// right, bottom left.
    pub fn corners(&self) -> [(usize, usize); 4] {
        let ([r0, r1], [c0, c1]) = (self.rows, self.cols);
        [(r0, c0), (r0, c1), (r1, c1), (r1, c0)]
    }

    /// Whether this is a switch of `p`: inside it, each pair in increasing
    /// order, and the corners alternating.
    pub fn fits(&self, p: &Picture) -> bool {
        let ([r0, r1], [c0, c1]) = (self.rows, self.cols);
        let inside = r0 < r1 && r1 < p.rows && c0 < c1 && c1 < p.cols;
        if !inside {
            return false;
        }
        let [tl, tr, br, bl] = self.corners().map(|(r, c)| p.get(r, c));
        tl == br && tr == bl && tl != tr
    }

    /// Which corners it empties and which it fills in `p`; `None` if it is
    /// not a switch of `p`.
    pub fn sides(&self, p: &Picture) -> Option<Sides> {
        if !self.fits(p) {
            return None;
        }
        let [tl, tr, br, bl] = self.corners();
        Some(if p.get(tl.0, tl.1) {
            Sides {
                emptied: [tl, br],
                filled: [tr, bl],
            }
        } else {
            Sides {
                emptied: [tr, bl],
                filled: [tl, br],
            }
        })
    }

    /// `p` with filled and empty swapped at the corners; `None` if this is
    /// not a switch of `p`.
    pub fn apply(&self, p: &Picture) -> Option<Picture> {
        if !self.fits(p) {
            return None;
        }
        let mut q = p.clone();
        for (r, c) in self.corners() {
            q.flip(r, c);
        }
        Some(q)
    }
}

/// Every switch of `p`: pairs of rows, then pairs of columns, in increasing
/// order.
pub fn switches(p: &Picture) -> Vec<Switch> {
    let mut out = Vec::new();
    for r0 in 0..p.rows {
        for r1 in r0 + 1..p.rows {
            for c0 in 0..p.cols {
                for c1 in c0 + 1..p.cols {
                    let s = Switch {
                        rows: [r0, r1],
                        cols: [c0, c1],
                    };
                    if s.fits(p) {
                        out.push(s);
                    }
                }
            }
        }
    }
    out
}

/// Every picture a chain of switches reaches from `start` (`start` too),
/// each with the fewest switches it takes, nearest first and, at one
/// distance, in increasing order.
pub fn reachable(start: &Picture) -> Vec<(Picture, usize)> {
    let mut seen: HashMap<Picture, usize> = HashMap::from([(start.clone(), 0)]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(p) = queue.pop_front() {
        let d = seen[&p];
        for s in switches(&p) {
            let Some(q) = s.apply(&p) else { continue };
            if let Entry::Vacant(slot) = seen.entry(q.clone()) {
                slot.insert(d + 1);
                queue.push_back(q);
            }
        }
    }
    let mut out: Vec<(Picture, usize)> = seen.into_iter().collect();
    out.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// A shortest chain of switches from `from` to `to` (the first one found
/// breadth first, switches in order), or `None` when no chain joins them.
pub fn walk(from: &Picture, to: &Picture) -> Option<Vec<Switch>> {
    let mut came: HashMap<Picture, Option<(Picture, Switch)>> = HashMap::from([(from.clone(), None)]);
    let mut queue = VecDeque::from([from.clone()]);
    while let Some(p) = queue.pop_front() {
        if p == *to {
            let mut chain = Vec::new();
            let mut at = p;
            while let Some((before, s)) = came[&at].clone() {
                chain.push(s);
                at = before;
            }
            chain.reverse();
            return Some(chain);
        }
        for s in switches(&p) {
            let Some(q) = s.apply(&p) else { continue };
            if let Entry::Vacant(slot) = came.entry(q.clone()) {
                slot.insert(Some((p.clone(), s)));
                queue.push_back(q);
            }
        }
    }
    None
}

// ---------------------------------------------------------------- the flow

/// What one square's pipe carries at most: one unit of water.
pub const SQUARE: usize = 1;

/// A node of Gale's network: the source that feeds every row's tap, a row,
/// a column, and the sink every column's drain empties into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    Source,
    Row(usize),
    Col(usize),
    Sink,
}

/// One augmenting path the flow took: its nodes from the source to the sink,
/// and how much it sent. A row-to-column step wets that square; a
/// column-to-row step takes water back out of that square.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Augmenting {
    pub nodes: Vec<Node>,
    pub room: usize,
}

impl Augmenting {
    /// Its squares in path order: (row, column, whether it wets the square
    /// or takes the water in it back).
    pub fn steps(&self) -> Vec<(usize, usize, bool)> {
        self.nodes
            .windows(2)
            .filter_map(|pair| match (pair[0], pair[1]) {
                (Node::Row(r), Node::Col(c)) => Some((r, c, true)),
                (Node::Col(c), Node::Row(r)) => Some((r, c, false)),
                _ => None,
            })
            .collect()
    }

    /// Whether it takes water back out of some square.
    pub fn takes_back(&self) -> bool {
        self.steps().iter().any(|&(_, _, wets)| !wets)
    }
}

/// The largest flow through Gale's network for a pair of shadows (D. Gale,
/// "A theorem on flows in networks", 1957): water enters each row as much
/// as its count, crosses any square of the row into that square's column
/// (one unit at most a square), and leaves each column as much as its
/// count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flow {
    /// How much water arrives.
    pub value: usize,
    /// The squares that carry water: with a full flow, a picture that fits.
    pub carried: Picture,
    /// For each column, whether the water can still reach it through spare
    /// capacity. The columns it cannot reach are the sink's side of the
    /// minimum cut with the smallest source side.
    pub reached: Vec<bool>,
    /// For each row, whether the water can still reach it: the labeled rows
    /// when the method stops ([`Flow::cut`]).
    pub reached_rows: Vec<bool>,
    /// Every augmenting path, in the order they were sent ([`replay`]
    /// rebuilds and checks the flow from them).
    pub paths: Vec<Augmenting>,
}

/// Where the water stops: the cut that Ford and Fulkerson's labeling leaves
/// when no path is left (their Lemma 2: it is a smallest cut, and its value
/// is the flow's). The water can still reach the rows and columns on the
/// source's side; every unit that gets through crosses the cut, by one of
/// three ways: out of a tap the water cannot reach, through a pipe from a
/// reached row into a column it cannot reach, or out of a reached column's
/// drain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cut {
    /// The rows and the columns the water can still reach.
    pub rows: Vec<usize>,
    pub cols: Vec<usize>,
    /// The columns it cannot reach.
    pub cut_off: Vec<usize>,
    /// What can cross each way: the counts of the taps it cannot reach; one
    /// unit a pipe from each reached row into each cut-off column; the
    /// counts of the reached columns' drains.
    pub unreached_taps: usize,
    pub pipes: usize,
    pub reached_drains: usize,
}

impl Cut {
    /// The most water that can cross the cut.
    pub fn value(&self) -> usize {
        self.unreached_taps + self.pipes + self.reached_drains
    }
}

impl Flow {
    /// The cut the method stopped at, on the shadows it ran on.
    pub fn cut(&self, s: &Shadows) -> Cut {
        let rows: Vec<usize> = (0..s.rows.len()).filter(|&i| self.reached_rows[i]).collect();
        let cols: Vec<usize> = (0..s.cols.len()).filter(|&j| self.reached[j]).collect();
        let cut_off: Vec<usize> = (0..s.cols.len()).filter(|&j| !self.reached[j]).collect();
        let unreached_taps = (0..s.rows.len())
            .filter(|&i| !self.reached_rows[i])
            .map(|i| s.rows[i])
            .sum();
        // each pipe carries one unit at most ([`SQUARE`])
        let pipes = rows.len() * cut_off.len();
        let reached_drains = cols.iter().map(|&j| s.cols[j]).sum();
        Cut {
            rows,
            cols,
            cut_off,
            unreached_taps,
            pipes,
            reached_drains,
        }
    }
}

/// [`Flow`] by Ford and Fulkerson's labeling method (Canadian Journal of
/// Mathematics 9, 1957, pp. 210–218), labeling breadth first, so each path is
/// a shortest one: send one more unit along a path with room, where a square
/// already carrying water has room backwards (that water can be taken back),
/// until no path is left. Nodes: 0 the source, 1 to m the rows, then the n
/// columns, then the sink, scanned in that order.
pub fn max_flow(s: &Shadows) -> Flow {
    let (m, n) = (s.rows.len(), s.cols.len());
    let (source, sink) = (0, m + n + 1);
    let size = m + n + 2;
    let node = |k: usize| {
        if k == source {
            Node::Source
        } else if k == sink {
            Node::Sink
        } else if k <= m {
            Node::Row(k - 1)
        } else {
            Node::Col(k - 1 - m)
        }
    };
    // spare[u][v]: how much more can go from u to v
    let mut spare = vec![vec![0usize; size]; size];
    for (i, &r) in s.rows.iter().enumerate() {
        spare[source][1 + i] = r;
        for j in 0..n {
            spare[1 + i][1 + m + j] = SQUARE;
        }
    }
    for (j, &c) in s.cols.iter().enumerate() {
        spare[1 + m + j][sink] = c;
    }
    let mut value = 0;
    let mut paths = Vec::new();
    loop {
        // breadth first from the source, along edges with room
        let mut came_from = vec![usize::MAX; size];
        came_from[source] = source;
        let mut queue = VecDeque::from([source]);
        while let Some(u) = queue.pop_front() {
            for (w, from) in came_from.iter_mut().enumerate() {
                if *from == usize::MAX && spare[u][w] > 0 {
                    *from = u;
                    queue.push_back(w);
                }
            }
        }
        if came_from[sink] == usize::MAX {
            let mut carried = Picture::empty(m, n);
            for i in 0..m {
                for j in 0..n {
                    // a square's edge starts with room 1; none left means it carries water
                    if spare[1 + i][1 + m + j] == 0 {
                        carried.flip(i, j);
                    }
                }
            }
            let reached = (0..n).map(|j| came_from[1 + m + j] != usize::MAX).collect();
            let reached_rows = (0..m).map(|i| came_from[1 + i] != usize::MAX).collect();
            return Flow {
                value,
                carried,
                reached,
                reached_rows,
                paths,
            };
        }
        let mut path = vec![sink];
        let mut w = sink;
        while w != source {
            w = came_from[w];
            path.push(w);
        }
        path.reverse();
        let room = path.windows(2).map(|e| spare[e[0]][e[1]]).min().unwrap_or(0);
        for e in path.windows(2) {
            spare[e[0]][e[1]] -= room;
            spare[e[1]][e[0]] += room;
        }
        value += room;
        paths.push(Augmenting {
            nodes: path.iter().map(|&k| node(k)).collect(),
            room,
        });
    }
}

/// The water after some augmenting paths: the wet squares, and how much each
/// row's tap has sent and each column's drain has taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowState {
    pub wet: Picture,
    pub sent: Vec<usize>,
    pub drained: Vec<usize>,
}

impl FlowState {
    /// How many units have got through.
    pub fn through(&self) -> usize {
        self.drained.iter().sum()
    }
}

/// Replay augmenting `paths` on the shadows' network, starting with no water,
/// and check every one: it runs from the source into a row, then alternates
/// a column and a row, and leaves through a column into the sink, never
/// visiting a row or a column twice; it sends one unit; it wets only dry
/// squares and takes water back only from wet ones; no tap sends more than
/// its row's count and no drain takes more than its column's count. Those
/// rules keep every row and column balanced (each row has sent, and each
/// column drained, exactly what its wet squares carry); the replay asserts
/// that balance after each path as a structural invariant, not as a separate
/// way a path can fail. Returns the state after each path.
pub fn replay(s: &Shadows, paths: &[Augmenting]) -> Result<Vec<FlowState>, String> {
    let (m, n) = (s.rows.len(), s.cols.len());
    let mut state = FlowState {
        wet: Picture::empty(m, n),
        sent: vec![0; m],
        drained: vec![0; n],
    };
    let mut out = Vec::with_capacity(paths.len());
    for (k, path) in paths.iter().enumerate() {
        let bad = |why: String| format!("path {k}: {why}");
        if path.room != SQUARE {
            return Err(bad(format!("sends {} units, not {SQUARE}", path.room)));
        }
        let inner = match path.nodes.as_slice() {
            [Node::Source, inner @ .., Node::Sink] => inner,
            _ => return Err(bad("does not run from the source to the sink".into())),
        };
        let (pairs, rest) = inner.as_chunks::<2>();
        if pairs.is_empty() || !rest.is_empty() {
            return Err(bad("is not rows and columns in turn".into()));
        }
        let mut seen = HashSet::with_capacity(inner.len());
        if let Some(node) = inner.iter().find(|&&node| !seen.insert(node)) {
            return Err(bad(format!("visits {node:?} twice, a walk with a cycle")));
        }
        let mut squares = Vec::with_capacity(pairs.len());
        for pair in pairs {
            match pair {
                [Node::Row(r), Node::Col(c)] if *r < m && *c < n => squares.push((*r, *c)),
                _ => return Err(bad(format!("{pair:?} is not a row and then a column of the grid"))),
            }
        }
        let (Some(&(tap, _)), Some(&(_, drain))) = (squares.first(), squares.last()) else {
            return Err(bad("has no square".into()));
        };
        if state.sent[tap] >= s.rows[tap] {
            return Err(bad(format!("row {tap}'s tap has no water left")));
        }
        state.sent[tap] += 1;
        for (j, &(r, c)) in squares.iter().enumerate() {
            if j > 0 {
                let back = squares[j - 1].1;
                if !state.wet.get(r, back) {
                    return Err(bad(format!("takes water back from the dry square ({r}, {back})")));
                }
                state.wet.flip(r, back);
            }
            if state.wet.get(r, c) {
                return Err(bad(format!("wets the square ({r}, {c}), already wet")));
            }
            state.wet.flip(r, c);
        }
        if state.drained[drain] >= s.cols[drain] {
            return Err(bad(format!("column {drain}'s drain is already full")));
        }
        state.drained[drain] += 1;
        // the invariant the rules above imply, asserted rather than trusted
        if state.sent != state.wet.row_counts() || state.drained != state.wet.col_counts() {
            return Err(bad("leaves a row or a column unbalanced".into()));
        }
        out.push(state.clone());
    }
    Ok(out)
}

/// Whether one more unit could get through without taking any water back:
/// a row with water left, a dry square in it, and that square's column with
/// room in its drain.
pub fn forward_room(s: &Shadows, state: &FlowState) -> bool {
    (0..s.rows.len()).any(|r| {
        state.sent[r] < s.rows[r] && (0..s.cols.len()).any(|c| !state.wet.get(r, c) && state.drained[c] < s.cols[c])
    })
}

/// What the flow says about a pair of shadows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reconstruction {
    /// The water filled every row and every column: this picture fits.
    Built(Picture),
    /// The shadows count different totals, so no picture fits.
    TotalsDiffer,
    /// The flow stopped short of the total, so no picture fits; the columns
    /// the water cannot reach ask for more squares than the rows can give
    /// them ([`can_give`]).
    Stuck {
        flow: usize,
        total: usize,
        bottleneck: Vec<usize>,
    },
}

/// Build a picture that fits the shadows, or say why none does.
pub fn reconstruct(s: &Shadows) -> Reconstruction {
    let total = s.row_total();
    if total != s.col_total() {
        return Reconstruction::TotalsDiffer;
    }
    let flow = max_flow(s);
    if flow.value == total {
        return Reconstruction::Built(flow.carried);
    }
    let bottleneck = (0..s.cols.len()).filter(|&j| !flow.reached[j]).collect();
    Reconstruction::Stuck {
        flow: flow.value,
        total,
        bottleneck,
    }
}

/// How many squares the rows can give a set of `k` columns: a row of count
/// r gives each column at most one square, so at most min(r, k) (Gale's
/// count).
pub fn can_give(rows: &[usize], k: usize) -> usize {
    rows.iter().map(|&r| r.min(k)).sum()
}

/// The smallest set of columns that asks for more squares than the rows can
/// give it (of that size, the first in increasing order), or `None`.
///
/// # Panics
/// Past 20 columns.
pub fn bottleneck(s: &Shadows) -> Option<Vec<usize>> {
    let n = s.cols.len();
    assert!(n <= 20, "{n} columns");
    let mut sets: Vec<Vec<usize>> = (1u32..1 << n)
        .map(|mask| (0..n).filter(|&j| (mask >> j) & 1 == 1).collect())
        .collect();
    sets.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
    sets.into_iter()
        .find(|set| set.iter().map(|&j| s.cols[j]).sum::<usize>() > can_give(&s.rows, set.len()))
}

/// Gale's table: for k = 1 to the number of columns, what the k largest
/// columns ask for together, and what the rows can give any k columns
/// ([`can_give`]).
pub fn gale_table(s: &Shadows) -> Vec<(usize, usize)> {
    let mut cols = s.cols.clone();
    cols.sort_unstable();
    cols.reverse();
    let mut asked = 0;
    cols.iter()
        .enumerate()
        .map(|(i, &c)| {
            asked += c;
            (asked, can_give(&s.rows, i + 1))
        })
        .collect()
}

/// Gale's test (1957; Ryser's Theorem 2.1 in other words): a picture fits
/// exactly when the totals agree and, for every k, the k largest columns ask
/// for at most what the rows can give k columns.
pub fn gale_ryser(s: &Shadows) -> bool {
    s.row_total() == s.col_total() && gale_table(s).iter().all(|&(asks, gives)| asks <= gives)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::*;

    /// Every picture of a size, grouped by its shadows.
    fn by_shadows(rows: usize, cols: usize) -> HashMap<Shadows, Vec<Picture>> {
        let mut groups: HashMap<Shadows, Vec<Picture>> = HashMap::new();
        for p in all_pictures(rows, cols) {
            groups.entry(p.shadows()).or_default().push(p);
        }
        groups
    }

    #[test]
    fn patterns_parse_and_print_back() {
        assert_eq!(face().pattern(), FACE.join("/"));
        assert_eq!(tree().pattern(), TREE.join("/"));
        assert_eq!(face().row_counts(), [2, 2, 0, 2, 3]);
        assert_eq!(face().col_counts(), [1, 3, 1, 3, 1]);
        assert_eq!(face().count(), 9);
        assert!(Picture::parse(&[]).is_err());
        assert!(Picture::parse(&["01", "0"]).unwrap_err().contains("not 2 squares"));
        assert!(Picture::parse(&["01", "0x"]).unwrap_err().contains("not 0 or 1"));
    }

    /// The order pictures sort in is the order of their pattern strings.
    #[test]
    fn pictures_sort_as_their_patterns() {
        let pictures = all_pictures(2, 3);
        let patterns: Vec<String> = pictures.iter().map(Picture::pattern).collect();
        let mut sorted = patterns.clone();
        sorted.sort();
        assert_eq!(patterns, sorted);
        assert_eq!(pictures.len(), 64);
    }

    /// "It changes the picture, but neither shadow": on every switch of
    /// every 4x4 picture, both shadows stay and exactly the four corners
    /// change; each of the switch's rows and columns loses one square and
    /// gains one.
    #[test]
    fn every_switch_keeps_both_shadows_and_changes_four_squares() {
        let mut checked = 0;
        for p in all_pictures(4, 4) {
            for s in switches(&p) {
                let q = s.apply(&p).unwrap();
                assert_eq!(q.shadows(), p.shadows(), "{} by {s:?}", p.pattern());
                let mut changed = p.differences(&q);
                changed.sort_unstable();
                let mut corners = s.corners().to_vec();
                corners.sort_unstable();
                assert_eq!(changed, corners);
                let sides = s.sides(&p).unwrap();
                for r in s.rows {
                    assert_eq!(sides.emptied.iter().filter(|rc| rc.0 == r).count(), 1);
                    assert_eq!(sides.filled.iter().filter(|rc| rc.0 == r).count(), 1);
                }
                for c in s.cols {
                    assert_eq!(sides.emptied.iter().filter(|rc| rc.1 == c).count(), 1);
                    assert_eq!(sides.filled.iter().filter(|rc| rc.1 == c).count(), 1);
                }
                // a switch undoes itself
                assert_eq!(s.apply(&q), Some(p.clone()));
                checked += 1;
            }
        }
        assert_eq!(checked, 294_912);
    }

    /// A rectangle whose corners do not alternate is not a switch.
    #[test]
    fn a_rectangle_that_does_not_alternate_is_no_switch() {
        let p = Picture::parse(&["11", "00"]).unwrap();
        let s = Switch {
            rows: [0, 1],
            cols: [0, 1],
        };
        assert!(!s.fits(&p));
        assert_eq!(s.apply(&p), None);
        assert_eq!(s.sides(&p), None);
        assert!(switches(&p).is_empty());
        let backwards = Switch {
            rows: [1, 0],
            cols: [0, 1],
        };
        assert!(!backwards.fits(&Picture::parse(&["10", "01"]).unwrap()));
        let outside = Switch {
            rows: [0, 2],
            cols: [0, 1],
        };
        assert!(!outside.fits(&p));
    }

    /// `fits` finds exactly the pictures with the shadows, on every pair of
    /// shadows a 3x3 or 4x4 picture casts, and none for shadows nothing
    /// casts.
    #[test]
    fn fits_finds_exactly_the_pictures_with_those_shadows() {
        for (rows, cols) in [(3, 3), (4, 4), (2, 5)] {
            let groups = by_shadows(rows, cols);
            for (s, group) in &groups {
                assert_eq!(&fits(s), group, "{s:?}");
            }
        }
        let face = face();
        assert_eq!(fits(&face.shadows()).len(), 64);
        assert!(fits(&face.shadows()).contains(&face));
        assert!(fits(&Shadows::new(&face.row_counts(), &NO_FIT_COLS)).is_empty());
        // totals that differ, and a row asking for more squares than it has
        assert!(fits(&Shadows::new(&[1, 1], &[1, 0])).is_empty());
        assert!(fits(&Shadows::new(&[3, 0], &[1, 1])).is_empty());
        // all empty and all full fit exactly one picture each
        assert_eq!(fits(&Shadows::new(&[0, 0], &[0, 0, 0])), [Picture::empty(2, 3)]);
        let full = Picture::parse(&["111", "111"]).unwrap();
        assert_eq!(fits(&full.shadows()), [full]);
    }

    /// Ryser's Theorem 3.1 on every 4x4 pair of shadows: the pictures that
    /// cast them are all joined by switches.
    #[test]
    fn switches_join_every_picture_with_the_same_shadows() {
        let groups = by_shadows(4, 4);
        assert_eq!(groups.len(), 16_145);
        for (s, group) in &groups {
            let reached: Vec<Picture> = {
                let mut r: Vec<Picture> = reachable(&group[0]).into_iter().map(|(p, _)| p).collect();
                r.sort();
                r
            };
            assert_eq!(&reached, group, "{s:?}");
        }
    }

    /// A picture is the only one with its shadows exactly when it has no
    /// switch, on every 4x4 picture; the tree is such a picture.
    #[test]
    fn a_picture_is_alone_with_its_shadows_exactly_when_it_has_no_switch() {
        let groups = by_shadows(4, 4);
        let mut alone = 0;
        for p in all_pictures(4, 4) {
            let only = groups[&p.shadows()].len() == 1;
            assert_eq!(only, switches(&p).is_empty(), "{}", p.pattern());
            alone += usize::from(only);
        }
        assert_eq!(alone, 6_902);
        let tree = tree();
        assert!(switches(&tree).is_empty());
        assert_eq!(fits(&tree.shadows()), [tree]);
    }

    /// The flow, Gale's test and enumeration agree on every pair of 4x4
    /// shadows with equal totals (counts 0 to 4) and on every pair of 3x3
    /// shadows (counts 0 to 3, totals equal or not); a full flow's picture
    /// fits; a stuck flow names columns that ask for more than the rows can
    /// give.
    #[test]
    fn the_flow_builds_a_picture_exactly_when_one_fits() {
        for (size, top) in [(3usize, 3usize), (4, 4)] {
            let cast: HashSet<Shadows> = by_shadows(size, size).into_keys().collect();
            let counts: Vec<Vec<usize>> = (0..(top + 1).pow(size as u32))
                .map(|k| (0..size).map(|i| k / (top + 1).pow(i as u32) % (top + 1)).collect())
                .collect();
            for rows in &counts {
                for cols in &counts {
                    let s = Shadows::new(rows, cols);
                    if size == 4 && s.row_total() != s.col_total() {
                        continue;
                    }
                    let fits_one = cast.contains(&s);
                    assert_eq!(gale_ryser(&s), fits_one, "{s:?}");
                    match reconstruct(&s) {
                        Reconstruction::Built(p) => {
                            assert!(fits_one, "{s:?}");
                            assert_eq!(p.shadows(), s);
                        }
                        Reconstruction::TotalsDiffer => {
                            assert!(!fits_one);
                            assert_ne!(s.row_total(), s.col_total());
                        }
                        Reconstruction::Stuck {
                            flow,
                            total,
                            bottleneck: cut,
                        } => {
                            assert!(!fits_one, "{s:?}");
                            assert!(flow < total);
                            let asks: usize = cut.iter().map(|&j| s.cols[j]).sum();
                            assert!(asks > can_give(&s.rows, cut.len()), "{s:?}: {cut:?}");
                            assert_eq!(bottleneck(&s).is_some(), !fits_one);
                        }
                    }
                }
            }
        }
    }

    /// Chapter 01's last example: the flow stops one square short, the
    /// first column alone is the bottleneck, and it asks for a square in
    /// every row while the middle row has none to give.
    #[test]
    fn the_face_rows_with_the_other_columns_fit_nothing() {
        let face = face();
        let s = Shadows::new(&face.row_counts(), &NO_FIT_COLS);
        assert_eq!(s.row_total(), s.col_total());
        assert!(!gale_ryser(&s));
        assert_eq!(max_flow(&s).value, 8);
        assert_eq!(
            reconstruct(&s),
            Reconstruction::Stuck {
                flow: 8,
                total: 9,
                bottleneck: vec![0]
            }
        );
        assert_eq!(bottleneck(&s), Some(vec![0]));
        assert_eq!(NO_FIT_COLS[0], face.rows());
        assert_eq!(can_give(&s.rows, 1), face.rows() - 1);
        // the face's own shadows: a full flow, and its picture is one of the 64
        match reconstruct(&face.shadows()) {
            Reconstruction::Built(p) => assert!(fits(&face.shadows()).contains(&p)),
            other => panic!("the face's shadows: {other:?}"),
        }
    }

    /// The walk between two pictures is a chain of switches of the right
    /// length, and no chain joins pictures with different shadows.
    #[test]
    fn walks_are_chains_of_switches() {
        let face = face();
        let distances: HashMap<Picture, usize> = reachable(&face).into_iter().collect();
        assert_eq!(distances.len(), 64);
        for (p, d) in &distances {
            let chain = walk(&face, p).unwrap();
            assert_eq!(chain.len(), *d);
            let mut at = face.clone();
            for s in &chain {
                at = s.apply(&at).unwrap();
            }
            assert_eq!(&at, p);
        }
        assert_eq!(walk(&face, &face), Some(vec![]));
        assert_eq!(walk(&face, &tree()), None);
        let rings: BTreeSet<usize> = distances.values().copied().collect();
        assert_eq!(rings.into_iter().collect::<Vec<_>>(), [0, 1, 2, 3]);
    }

    /// Every flow the method sends, on every 4x4 pair of shadows with equal
    /// totals (counts 0 to 4), replays path by path within every capacity
    /// and with every row and column balanced, one unit a path, and ends at
    /// the flow's own wet squares and value; where it stops, not even a path
    /// that takes nothing back is left.
    #[test]
    fn every_flow_replays_within_its_capacities() {
        let counts: Vec<Vec<usize>> = (0..625usize)
            .map(|k| (0..4u32).map(|i| k / 5usize.pow(i) % 5).collect())
            .collect();
        let mut replayed = 0;
        for rows in &counts {
            for cols in &counts {
                let s = Shadows::new(rows, cols);
                if s.row_total() != s.col_total() {
                    continue;
                }
                let flow = max_flow(&s);
                let states = replay(&s, &flow.paths).unwrap_or_else(|e| panic!("{s:?}: {e}"));
                assert_eq!(states.len(), flow.value, "{s:?}");
                let last = states.last().cloned().unwrap_or_else(|| FlowState {
                    wet: Picture::empty(4, 4),
                    sent: vec![0; 4],
                    drained: vec![0; 4],
                });
                assert_eq!(last.wet, flow.carried, "{s:?}");
                assert_eq!(last.through(), flow.value);
                assert!(flow.paths.iter().all(|p| p.room == SQUARE));
                // a stopped flow has no room left even without taking back
                assert!(!forward_room(&s, &last), "{s:?}");
                replayed += 1;
            }
        }
        assert_eq!(replayed, 38_165);
    }

    /// Chapter 02's trace: on the face's shadows the method sends eight
    /// units straight down, finds the bottom row stuck without taking back,
    /// then sends a ninth that takes the top row's unit back out of the first
    /// column; it ends at a picture with the face's shadows that is not the
    /// face.
    #[test]
    fn the_face_flow_takes_water_back_once() {
        let face = face();
        let s = face.shadows();
        let flow = max_flow(&s);
        let states = replay(&s, &flow.paths).unwrap();
        assert_eq!(flow.paths.len(), 9);
        let back: Vec<usize> = (0..9).filter(|&k| flow.paths[k].takes_back()).collect();
        assert_eq!(back, [8]);
        assert!(flow.paths[..8].iter().all(|p| p.steps().len() == 1));
        assert!(!forward_room(&s, &states[7]));
        assert_eq!(states[7].through(), 8);
        assert_eq!(
            flow.paths[8].nodes,
            [
                Node::Source,
                Node::Row(4),
                Node::Col(0),
                Node::Row(0),
                Node::Col(3),
                Node::Sink
            ]
        );
        assert_eq!(flow.paths[8].steps(), [(4, 0, true), (0, 0, false), (0, 3, true)]);
        let built = &states[8].wet;
        assert_eq!(built.pattern(), "01010/01100/00000/01010/10011");
        assert_eq!(built.shadows(), s);
        assert_ne!(*built, face);
        assert!(fits(&s).contains(built));
    }

    /// The replay refuses every way a path can break the network's rules.
    #[test]
    fn replay_refuses_broken_paths() {
        use Node::{Col, Row, Sink, Source};
        // rows 2 and 1, columns 2 and 1: the first row can send twice, the
        // second row and the second column only once
        let s = Shadows::new(&[2, 1], &[2, 1]);
        let path = |nodes: &[Node]| Augmenting {
            nodes: nodes.to_vec(),
            room: SQUARE,
        };
        let first = path(&[Source, Row(0), Col(0), Sink]);
        assert!(replay(&s, std::slice::from_ref(&first)).is_ok());
        // a walk through the top row twice: within the counts, every square
        // it wets dry and every square it takes back from wet, yet not a
        // path (the method's breadth-first paths never repeat a node)
        let cyclic = path(&[Source, Row(0), Col(0), Row(0), Col(1), Sink]);
        let cases: [(&str, Vec<Augmenting>); 9] = [
            ("twice", vec![cyclic]),
            ("already wet", vec![first.clone(), first.clone()]),
            (
                "dry square",
                vec![path(&[Source, Row(0), Col(0), Row(1), Col(1), Sink])],
            ),
            (
                "no water left",
                vec![
                    path(&[Source, Row(1), Col(0), Sink]),
                    path(&[Source, Row(1), Col(1), Sink]),
                ],
            ),
            (
                "already full",
                vec![
                    path(&[Source, Row(0), Col(1), Sink]),
                    path(&[Source, Row(1), Col(1), Sink]),
                ],
            ),
            ("from the source", vec![path(&[Row(0), Col(0), Sink])]),
            ("in turn", vec![path(&[Source, Row(0), Sink])]),
            ("of the grid", vec![path(&[Source, Row(0), Col(5), Sink])]),
            (
                "units",
                vec![Augmenting {
                    nodes: first.nodes.clone(),
                    room: 2,
                }],
            ),
        ];
        for (why, paths) in cases {
            let e = replay(&s, &paths).unwrap_err();
            assert!(e.contains(why), "{why}: {e}");
        }
    }

    /// "Each row gives it at most one square per column, and no more than
    /// its count": on every 4x4 picture, every group of columns holds at most
    /// the sum over rows of min(row count, group size).
    #[test]
    fn a_group_of_columns_gets_at_most_min_count_k_from_each_row() {
        for p in all_pictures(4, 4) {
            let (rows, cols) = (p.row_counts(), p.col_counts());
            for mask in 1u32..1 << 4 {
                let group: Vec<usize> = (0..4).filter(|&j| (mask >> j) & 1 == 1).collect();
                let holds: usize = group.iter().map(|&j| cols[j]).sum();
                assert!(holds <= can_give(&rows, group.len()), "{} {group:?}", p.pattern());
            }
        }
    }

    /// Where the method stops is a cut of exactly the flow's size (Ford and
    /// Fulkerson's Lemma 2), on every 4x4 pair of shadows with equal totals;
    /// it stops short exactly when Gale's table fails at some size, and then
    /// the columns it cannot reach ask for more than the rows can give them.
    #[test]
    fn the_method_stops_at_a_cut_of_its_own_size() {
        let counts: Vec<Vec<usize>> = (0..625usize)
            .map(|k| (0..4u32).map(|i| k / 5usize.pow(i) % 5).collect())
            .collect();
        let mut short = 0;
        for rows in &counts {
            for cols in &counts {
                let s = Shadows::new(rows, cols);
                if s.row_total() != s.col_total() {
                    continue;
                }
                let flow = max_flow(&s);
                let cut = flow.cut(&s);
                assert_eq!(cut.value(), flow.value, "{s:?}");
                let stops_short = flow.value < s.row_total();
                let fails = gale_table(&s).iter().any(|&(asks, gives)| asks > gives);
                assert_eq!(stops_short, fails, "{s:?}");
                if stops_short {
                    let asks: usize = cut.cut_off.iter().map(|&j| s.cols[j]).sum();
                    assert!(asks > can_give(&s.rows, cut.cut_off.len()), "{s:?}: {cut:?}");
                    short += 1;
                }
            }
        }
        assert!(short > 0);
    }

    /// Chapter 03's example: the face's rows with columns 5, 1, 1, 1, 1. Eight
    /// units go straight down and the method stops; the water still reaches
    /// rows 0, 1, 3, 4 and columns 1 to 4, never the first column; the cut
    /// is four pipes into that column and the four other drains. Gale's
    /// table fails only for the single largest column; the face's columns
    /// pass every size.
    #[test]
    fn the_no_fit_flow_stops_at_the_first_column() {
        let face = face();
        let s = Shadows::new(&face.row_counts(), &NO_FIT_COLS);
        let flow = max_flow(&s);
        assert_eq!(flow.value, 8);
        assert_eq!(flow.paths.len(), 8);
        assert!(flow.paths.iter().all(|p| !p.takes_back()));
        let cut = flow.cut(&s);
        assert_eq!(
            cut,
            Cut {
                rows: vec![0, 1, 3, 4],
                cols: vec![1, 2, 3, 4],
                cut_off: vec![0],
                unreached_taps: 0,
                pipes: 4,
                reached_drains: 4,
            }
        );
        assert_eq!(gale_table(&s), [(5, 4), (6, 8), (7, 9), (8, 9), (9, 9)]);
        assert_eq!(gale_table(&face.shadows()), [(3, 4), (6, 8), (7, 9), (8, 9), (9, 9)]);
        assert!(gale_ryser(&face.shadows()) && !gale_ryser(&s));
        // equal totals matter: rows 2, 2 with columns 1, 1 pass every size of
        // the table, yet their totals are 4 and 2 and nothing fits
        let unequal = Shadows::new(&[2, 2], &[1, 1]);
        assert!(gale_table(&unequal).iter().all(|&(asks, gives)| asks <= gives));
        assert_ne!(unequal.row_total(), unequal.col_total());
        assert!(!gale_ryser(&unequal) && fits(&unequal).is_empty());
    }
}
