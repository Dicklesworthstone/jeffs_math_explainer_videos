//! The enhanced-tier network-graph mobjects (§12.5-12.6 leapfrog #7,
//! fm-n64): [`NetworkGraph`] renders an [`fnx_classes::Graph`] —
//! string-labeled vertices, undirected edges — as a detached [`VMobject`]
//! family laid out by one of four **fmn-owned deterministic kernels**.
//!
//! ## OQ-5 outcome (the layout-kernel audit, recorded here by fm-n64)
//!
//! franken_networkx at its SUITE.lock pin (`9d710b1c3`) ships **no native
//! Rust layout kernels**: `fnx-algorithms` covers traversal, centrality,
//! flow, matching, and MST, while every layout helper lives in its *Python*
//! package on top of numpy (and `fnx-algorithms` itself drags rayon,
//! mt19937, and mwmatching — inadmissible into the governed closure under
//! D1). The Python kernels are therefore nothing to audit for our purposes.
//! The enhanced tier's layouts are these native fmn implementations instead;
//! if upstream lands Rust kernels later, the revisit is a successor bead
//! that re-audits them against the same rules recorded here.
//!
//! ## The determinism rules (binding on this module)
//!
//! * **Node order** is always [`fnx_classes::Graph::nodes_ordered`]
//!   — insertion-ordered, never hash order. Every kernel indexes positions
//!   by that vector.
//! * **Circular** — pure function of node count: angle `i·τ/n` in `f64`
//!   around the origin. Behavior Note: fnx's Python `circular_layout`
//!   casts its angles through `float32`; ours stays `f64`, so coordinates
//!   differ at the last-ulp level by design.
//! * **Shell** — concentric rings from an explicit partition of the node
//!   set; within a ring, the same angular rule as Circular.
//! * **BreadthFirst** — BFS levels from a named root; each frontier expands
//!   in sorted-neighbor order (this module's owned convention: sorted
//!   label iteration), one ring per level.
//! * **Spring** — Fruchterman-Reingold with a **seeded** initial scatter:
//!   initial positions come from `RngRoot::from_seed(seed)
//!   .substream("network_graph.spring")`, forces accumulate in fixed node
//!   and edge order with plain IEEE `f64` (stable Rust never contracts
//!   FMA), and the cooling schedule length equals the fixed iteration
//!   count. Same graph + same seed ⇒ bit-identical positions on any
//!   platform; a different seed ⇒ a different, equally valid layout.
//!
//! Enhanced-tier discipline: none of this blocks core gates, and none of
//! it enters a certified claim — but what lands meets the same bars
//! (structural fixtures, determinism digests, self-goldens).

use std::collections::{BTreeMap, BTreeSet};

use fmn_core::constants::{BLUE_D, GREY_B, TAU};
use fmn_core::rng::RngRoot;
use fmn_core::types::Vec3;

use fnx_classes::Graph;

use crate::arc::Dot;
use crate::line::Line;
use crate::style::Style;
use crate::vmobject::{VMobject, v_group};

/// Edge styling: the neutral grey, slightly heavier than a gridline.
const EDGE_STROKE_WIDTH: f64 = 2.5;
/// Vertex styling radius: a shade above [`crate::arc::Dot`]'s default so
/// edge ink cannot reach the rim.
const VERTEX_RADIUS: f64 = 0.09;

/// Why a [`NetworkGraph`] could not be laid out or built.
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkGraphError {
    /// A shell/BFS kernel named a node the graph does not contain.
    UnknownNode(String),
    /// The shell partition did not cover the node set exactly once.
    ShellPartitionMismatch {
        /// Labels named twice across shells.
        duplicated: Vec<String>,
        /// Labels present in the graph but absent from every shell.
        missing: Vec<String>,
    },
    /// An edge endpoint had no position (a typed guard rather than a panic).
    UnpositionedEdge {
        /// The tail label.
        from: String,
        /// The head label.
        to: String,
    },
    /// A geometry primitive behind the family refused construction.
    Geometry(String),
}

impl std::fmt::Display for NetworkGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownNode(node) => {
                write!(f, "layout names node {node:?}, which the graph lacks")
            }
            Self::ShellPartitionMismatch {
                duplicated,
                missing,
            } => write!(
                f,
                "shell partition must cover the node set exactly once \
                 (duplicated: {duplicated:?}, missing: {missing:?})"
            ),
            Self::UnpositionedEdge { from, to } => {
                write!(f, "edge {from:?} -> {to:?} has no laid-out endpoints")
            }
            Self::Geometry(detail) => write!(f, "graph family geometry refused: {detail}"),
        }
    }
}

impl std::error::Error for NetworkGraphError {}

/// The deterministic layout kernels (see the module's OQ-5 rules).
#[derive(Debug, Clone, PartialEq)]
pub enum GraphLayout {
    /// Vertices evenly spaced on one circle, insertion order.
    Circular,
    /// Concentric rings from an explicit partition; outermost shell first.
    Shell {
        /// The rings, outermost first; must partition the node set once.
        shells: Vec<Vec<String>>,
    },
    /// BFS levels from `root`; each level on its own ring, expansion in
    /// sorted-neighbor order.
    BreadthFirst {
        /// The level-0 vertex.
        root: String,
    },
    /// Seeded Fruchterman-Reingold. `ideal_edge` scales the whole layout;
    /// `iterations` fixes the cooling schedule length.
    Spring {
        /// The one-RNG seed keying the initial-scatter substream.
        seed: u64,
        /// Fixed iteration count (the cooling schedule's length).
        iterations: usize,
        /// Rest length every edge pulls toward.
        ideal_edge: f64,
    },
}

/// Default spring iteration count for [`GraphLayout::Spring`].
pub const DEFAULT_SPRING_ITERATIONS: usize = 50;
/// Default rest length for [`GraphLayout::Spring`].
pub const DEFAULT_SPRING_IDEAL_EDGE: f64 = 1.0;

/// A laid-out network graph: the fnx structure plus one position per node,
/// ready to [`NetworkGraph::build`] into dots-and-lines geometry.
#[derive(Debug, Clone)]
pub struct NetworkGraph {
    graph: Graph,
    labels: Vec<String>,
    index: BTreeMap<String, Vec3>,
}

impl NetworkGraph {
    /// Build from a node list plus an edge list — the constructor callers
    /// use when they own plain data (tests, CSV columns, bindings) rather
    /// than an existing [`Graph`]. Nodes are inserted in the given order;
    /// every edge endpoint is pre-inserted so the strict graph accepts it.
    #[must_use]
    pub fn from_edge_list(nodes: &[&str], edges: &[(&str, &str)]) -> Self {
        let mut graph = Graph::strict();
        for label in nodes {
            graph.add_node(*label);
        }
        for (from, to) in edges {
            graph.add_node(*from);
            graph.add_node(*to);
            graph
                .add_edge(*from, *to)
                .expect("from_edge_list pre-adds every endpoint");
        }
        Self::new(graph)
    }

    /// Wrap an undirected graph; positions are unset until a layout runs.
    #[must_use]
    pub fn new(graph: Graph) -> Self {
        let labels = graph
            .nodes_ordered()
            .into_iter()
            .map(str::to_owned)
            .collect();
        Self {
            graph,
            labels,
            index: BTreeMap::new(),
        }
    }

    /// Run one layout kernel, replacing any previous positions.
    ///
    /// # Errors
    /// [`NetworkGraphError`] for unknown shell/BFS roots or a bad partition.
    pub fn laid_out(mut self, layout: &GraphLayout) -> Result<Self, NetworkGraphError> {
        let positions = match layout {
            GraphLayout::Circular => Ok(self.circular_positions()),
            GraphLayout::Shell { shells } => self.shell_positions(shells),
            GraphLayout::BreadthFirst { root } => self.breadth_first_positions(root),
            GraphLayout::Spring {
                seed,
                iterations,
                ideal_edge,
            } => Ok(self.spring_positions(*seed, *iterations, *ideal_edge)),
        }?;
        self.index = positions;
        Ok(self)
    }

    /// The laid-out position of one node.
    #[must_use]
    pub fn position(&self, node: &str) -> Option<Vec3> {
        self.index.get(node).copied()
    }

    /// Node labels in the deterministic iteration order.
    #[must_use]
    pub fn nodes(&self) -> &[String] {
        &self.labels
    }

    /// The wrapped structure (for animation wiring over adjacency).
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Build the family: edges first (underneath), then vertex dots —
    /// painter order keeps ink off the dots' rims.
    ///
    /// # Errors
    /// [`NetworkGraphError`] for unpositioned endpoints or refused
    /// primitives.
    pub fn build(&self) -> Result<VMobject, NetworkGraphError> {
        let mut edges: Vec<VMobject> = Vec::new();
        for (from, to, _) in self.graph.edges_ordered_borrowed() {
            let (Some(start), Some(end)) = (self.index.get(from), self.index.get(to)) else {
                return Err(NetworkGraphError::UnpositionedEdge {
                    from: from.to_owned(),
                    to: to.to_owned(),
                });
            };
            let line = Line::new(*start, *end)
                .style(Style::default().stroke(GREY_B, EDGE_STROKE_WIDTH, 1.0))
                .build()
                .map_err(|error| NetworkGraphError::Geometry(error.to_string()))?;
            edges.push(line);
        }
        let mut vertices: Vec<VMobject> = Vec::new();
        for label in &self.labels {
            let Some(point) = self.index.get(label) else {
                return Err(NetworkGraphError::UnknownNode(label.clone()));
            };
            let dot = Dot::new()
                .point(*point)
                .radius(VERTEX_RADIUS)
                .color(BLUE_D)
                .build();
            vertices.push(dot);
        }
        // Edges underneath, vertices appended after (painter order).
        let group = v_group(edges);
        Ok(group.with_children(vertices))
    }

    // ------------------------------------------------------- kernels

    fn circular_angles(count: usize) -> impl Iterator<Item = f64> {
        (0..count).map(move |i| i as f64 * TAU / count.max(1) as f64)
    }

    fn circular_positions(&self) -> BTreeMap<String, Vec3> {
        let n = self.labels.len().max(1);
        self.labels
            .iter()
            .cloned()
            .zip(Self::circular_angles(n))
            .map(|(label, theta)| (label, [fmn_dmath::cos(theta), fmn_dmath::sin(theta), 0.0]))
            .collect()
    }

    fn shell_positions(
        &self,
        shells: &[Vec<String>],
    ) -> Result<BTreeMap<String, Vec3>, NetworkGraphError> {
        let known: BTreeSet<&str> = self.labels.iter().map(String::as_str).collect();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut duplicated: Vec<String> = Vec::new();
        // Membership first: a foreign label is the primary defect.
        for shell in shells {
            for label in shell {
                if !known.contains(label.as_str()) {
                    return Err(NetworkGraphError::UnknownNode(label.clone()));
                }
                if !seen.insert(label.as_str()) {
                    duplicated.push(label.clone());
                }
            }
        }
        let missing: Vec<String> = self
            .labels
            .iter()
            .filter(|label| !seen.contains(label.as_str()))
            .cloned()
            .collect();
        if !duplicated.is_empty() || !missing.is_empty() {
            return Err(NetworkGraphError::ShellPartitionMismatch {
                duplicated,
                missing,
            });
        }
        let mut positions = BTreeMap::new();
        let ring_count = shells.len().max(1) as f64;
        for (ring_index, shell) in shells.iter().enumerate() {
            // Outermost shell first: ring 0 sits at radius 1, the last at
            // 1/(ring_count + 1) — strictly decreasing, never zero.
            let radius = 1.0 - ring_index as f64 / (ring_count + 1.0);
            let n = shell.len().max(1);
            for (label, theta) in shell.iter().cloned().zip(Self::circular_angles(n)) {
                positions.insert(
                    label,
                    [
                        radius * fmn_dmath::cos(theta),
                        radius * fmn_dmath::sin(theta),
                        0.0,
                    ],
                );
            }
        }
        Ok(positions)
    }

    fn breadth_first_positions(
        &self,
        root: &str,
    ) -> Result<BTreeMap<String, Vec3>, NetworkGraphError> {
        if !self.labels.iter().any(|label| label == root) {
            return Err(NetworkGraphError::UnknownNode(root.to_owned()));
        }
        let mut levels: Vec<Vec<String>> = vec![vec![root.to_owned()]];
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        visited.insert(root);
        loop {
            let mut next: Vec<String> = Vec::new();
            let last = levels.last().expect("levels starts non-empty");
            for node in last {
                let Some(mut neighbors) = self.graph.neighbors(node.as_str()) else {
                    continue;
                };
                neighbors.sort_unstable();
                for neighbor in neighbors {
                    if visited.insert(neighbor) {
                        next.push(neighbor.to_owned());
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            levels.push(next);
        }
        let mut positions = BTreeMap::new();
        let depth = levels.len().saturating_sub(1) as f64;
        for (level_index, level) in levels.iter().enumerate() {
            let radius = if depth == 0.0 {
                1.0
            } else {
                level_index as f64 / depth
            };
            let n = level.len().max(1);
            for (label, theta) in level.iter().cloned().zip(Self::circular_angles(n)) {
                positions.insert(
                    label,
                    [
                        radius * fmn_dmath::cos(theta),
                        radius * fmn_dmath::sin(theta),
                        0.0,
                    ],
                );
            }
        }
        // Keep the rooted component bit-identical, and include every
        // disconnected vertex on an outer ring in node insertion order.
        let remaining: Vec<_> = self
            .labels
            .iter()
            .filter(|label| !visited.contains(label.as_str()))
            .collect();
        for (label, theta) in remaining.iter().zip(Self::circular_angles(remaining.len())) {
            positions.insert(
                (*label).clone(),
                [
                    1.5 * fmn_dmath::cos(theta),
                    1.5 * fmn_dmath::sin(theta),
                    0.0,
                ],
            );
        }
        Ok(positions)
    }

    fn spring_positions(
        &self,
        seed: u64,
        iterations: usize,
        ideal_edge: f64,
    ) -> BTreeMap<String, Vec3> {
        let count = self.labels.len();
        if count == 0 {
            return BTreeMap::new();
        }
        // Seeded initial scatter: the ONE RNG, named substream, uniform in
        // [-1, 1]^2 — same graph + same seed ⇒ identical everything after.
        let mut rng = RngRoot::from_seed(seed)
            .substream("network_graph.spring")
            .sequential();
        let mut xs: Vec<f64> = Vec::with_capacity(count);
        let mut ys: Vec<f64> = Vec::with_capacity(count);
        for _ in 0..count {
            xs.push(rng.next_f64() * 2.0 - 1.0);
            ys.push(rng.next_f64() * 2.0 - 1.0);
        }
        let index_of: BTreeMap<&str, usize> = self
            .labels
            .iter()
            .enumerate()
            .map(|(i, label)| (label.as_str(), i))
            .collect();
        let edge_pairs: Vec<(usize, usize)> = self
            .graph
            .edges_ordered_borrowed()
            .into_iter()
            .filter_map(
                |(from, to, _)| match (index_of.get(from), index_of.get(to)) {
                    (Some(&a), Some(&b)) => Some((a, b)),
                    _ => None,
                },
            )
            .collect();

        let iterations = iterations.max(1);
        for step in 0..iterations {
            // Cooling schedule: linear from 0.1 toward zero.
            let temperature = 0.1 * (iterations - step) as f64 / iterations as f64;
            let mut fx = vec![0.0; count];
            let mut fy = vec![0.0; count];
            // Repulsion: every pair i < j, symmetric accumulation in fixed
            // index order.
            for i in 0..count {
                for j in (i + 1)..count {
                    let dx = xs[i] - xs[j];
                    let dy = ys[i] - ys[j];
                    let dist = (dx * dx + dy * dy).max(1e-12).sqrt();
                    let strength = ideal_edge * ideal_edge / dist;
                    let ux = dx / dist;
                    let uy = dy / dist;
                    fx[i] += ux * strength;
                    fy[i] += uy * strength;
                    fx[j] -= ux * strength;
                    fy[j] -= uy * strength;
                }
            }
            // Attraction along edges, fixed edge order.
            for &(a, b) in &edge_pairs {
                let dx = xs[a] - xs[b];
                let dy = ys[a] - ys[b];
                let dist = (dx * dx + dy * dy).max(1e-12).sqrt();
                let strength = dist * dist / ideal_edge;
                let ux = dx / dist;
                let uy = dy / dist;
                fx[a] -= ux * strength;
                fy[a] -= uy * strength;
                fx[b] += ux * strength;
                fy[b] += uy * strength;
            }
            // Displacement clamped to the temperature, applied in index order.
            for i in 0..count {
                let magnitude = (fx[i] * fx[i] + fy[i] * fy[i]).sqrt();
                if magnitude > 1e-12 {
                    let capped = magnitude.min(temperature);
                    xs[i] += fx[i] / magnitude * capped;
                    ys[i] += fy[i] / magnitude * capped;
                }
            }
        }
        self.labels
            .iter()
            .cloned()
            .zip(xs.into_iter().zip(ys))
            .map(|(label, (x, y))| (label, [x, y, 0.0]))
            .collect()
    }
}
