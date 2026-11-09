//! Render-only persistence for already-observed frames.
//!
//! An undo/replay snapshot must retain the complete arena and its identities.
//! A recorded picture must not: unreachable animation copies, saved states,
//! proxy pins and updater registrations are not inputs to drawing that frame.
//! This projection retains the rooted family DAG, all record columns, shape
//! hints, placements, uniforms and image resources. It uses the existing FMNA
//! snapshot encoder, not a second wire format or a second renderer.
//!
//! Handles are renumbered in first-encounter, root/child order. Shared children
//! remain shared, and roots are neither flattened nor re-sorted: separate root
//! placements must still composite a shared drawable once per root. The result
//! is deliberately exposed only as durable bytes. It is NOT an undo state and
//! must never replace the live arena behind existing handles or Python proxies.

use std::collections::HashMap;

use fmn_hash::{Limits, SerialError};

use crate::StageError;
use crate::shape::ShapeSlot;
use crate::stage::{Mob, Snapshot, SnapshotEntry};

/// Graph workspace admitted while projecting a recorded frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderSnapshotLimits {
    /// Cumulative logical storage for traversal, identity and destination graph
    /// tables. Record/image payloads retain the existing snapshot CoW policy;
    /// encoding is separately bounded by [`Limits::DEFAULT`]. This is not a
    /// bound on the live scene, allocator bookkeeping or encoder scratch space.
    pub max_graph_bytes: usize,
}

impl RenderSnapshotLimits {
    /// The same graph-work headroom as the ordinary snapshot decoder: four
    /// times the canonical document ceiling, independent of arena high-water.
    pub const DEFAULT: Self = Self {
        max_graph_bytes: 4 * Limits::DEFAULT.max_total,
    };
}

impl Default for RenderSnapshotLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A projection refusal leaves the input snapshot and live scene untouched.
#[derive(Debug)]
pub enum RenderSnapshotError {
    /// A rooted edge is stale/foreign, or the selected graph contains a cycle.
    Graph(StageError),
    /// Duplicate roots or child edges cannot be represented canonically.
    InvalidGraph(&'static str),
    /// Graph work was refused before the corresponding reservation.
    GraphLimit {
        /// Graph budget in force.
        limit: usize,
        /// Cumulative logical bytes requested.
        needed: usize,
        /// Destination or traversal table that exhausted it.
        context: &'static str,
    },
    /// The allocator refused graph storage already admitted by the budget.
    AllocationFailed {
        /// Destination or traversal table.
        context: &'static str,
    },
    /// The shared canonical snapshot writer refused the projected state.
    Serial(SerialError),
}

impl std::fmt::Display for RenderSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graph(error) => write!(f, "render snapshot graph: {error}"),
            Self::InvalidGraph(reason) => write!(f, "render snapshot graph: {reason}"),
            Self::GraphLimit {
                limit,
                needed,
                context,
            } => write!(
                f,
                "render snapshot {context} needs {needed} graph bytes; limit is {limit}"
            ),
            Self::AllocationFailed { context } => {
                write!(f, "render snapshot could not allocate {context}")
            }
            Self::Serial(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RenderSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Graph(error) => Some(error),
            Self::Serial(error) => Some(error),
            Self::GraphLimit { .. } | Self::AllocationFailed { .. } | Self::InvalidGraph(_) => None,
        }
    }
}

struct Budget {
    limit: usize,
    charged: usize,
}

impl Budget {
    fn charge<T>(
        &mut self,
        count: usize,
        context: &'static str,
    ) -> Result<(), RenderSnapshotError> {
        let needed = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| self.charged.checked_add(bytes));
        if needed.is_none_or(|needed| needed > self.limit) {
            return Err(RenderSnapshotError::GraphLimit {
                limit: self.limit,
                needed: needed.unwrap_or(usize::MAX),
                context,
            });
        }
        self.charged = needed.unwrap_or(self.limit);
        Ok(())
    }

    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        context: &'static str,
    ) -> Result<(), RenderSnapshotError> {
        self.charge::<T>(additional, context)?;
        values
            .try_reserve(additional)
            .map_err(|_| RenderSnapshotError::AllocationFailed { context })
    }
}

#[derive(Clone, Copy)]
struct Visit {
    index: u32,
    active: bool,
    root_seen: bool,
}

fn entry(snapshot: &Snapshot, mob: Mob) -> Result<&SnapshotEntry, RenderSnapshotError> {
    let (index, generation) = mob.parts();
    if mob != Mob::from_parts(snapshot.stage_id, index, generation) {
        return Err(RenderSnapshotError::Graph(StageError::StaleHandle));
    }
    snapshot
        .slots
        .get(index as usize)
        .filter(|(stored, _)| *stored == generation)
        .and_then(|(_, entry)| entry.as_ref())
        .ok_or(RenderSnapshotError::Graph(StageError::StaleHandle))
}

fn discover(
    snapshot: &Snapshot,
    mob: Mob,
    visits: &mut HashMap<Mob, Visit>,
    order: &mut Vec<Mob>,
    stack: &mut Vec<(Mob, usize)>,
    budget: &mut Budget,
) -> Result<(), RenderSnapshotError> {
    entry(snapshot, mob)?;
    let index = u32::try_from(order.len()).map_err(|_| RenderSnapshotError::GraphLimit {
        limit: budget.limit,
        needed: usize::MAX,
        context: "addressable objects",
    })?;
    budget.charge::<(Mob, Visit)>(1, "identity table")?;
    visits
        .try_reserve(1)
        .map_err(|_| RenderSnapshotError::AllocationFailed {
            context: "identity table",
        })?;
    budget.reserve(order, 1, "object order")?;
    budget.reserve(stack, 1, "traversal stack")?;
    visits.insert(
        mob,
        Visit {
            index,
            active: true,
            root_seen: false,
        },
    );
    order.push(mob);
    stack.push((mob, 0));
    Ok(())
}

fn project(
    snapshot: &Snapshot,
    limits: RenderSnapshotLimits,
) -> Result<Snapshot, RenderSnapshotError> {
    let mut budget = Budget {
        limit: limits.max_graph_bytes,
        charged: 0,
    };
    let mut visits = HashMap::<Mob, Visit>::new();
    let mut order = Vec::new();
    let mut stack = Vec::new();
    // Iterative DFS uses stack space proportional to depth, not the number of
    // paths through a shared DAG. HashMap is used only for lookup; its random
    // iteration order cannot assign an identity or change serialized order.
    for &root in &snapshot.roots {
        if let Some(visit) = visits.get_mut(&root) {
            if visit.root_seen {
                return Err(RenderSnapshotError::InvalidGraph("duplicate scene root"));
            }
            visit.root_seen = true;
            continue;
        }
        discover(
            snapshot,
            root,
            &mut visits,
            &mut order,
            &mut stack,
            &mut budget,
        )?;
        visits
            .get_mut(&root)
            .ok_or(RenderSnapshotError::Graph(StageError::StaleHandle))?
            .root_seen = true;
        while let Some((mob, next)) = stack.last_mut() {
            let source = entry(snapshot, *mob)?;
            if let Some(&child) = source.submobjects.get(*next) {
                *next += 1;
                match visits.get(&child) {
                    Some(visit) if visit.active => {
                        return Err(RenderSnapshotError::Graph(StageError::CycleDetected));
                    }
                    Some(_) => {}
                    None => discover(
                        snapshot,
                        child,
                        &mut visits,
                        &mut order,
                        &mut stack,
                        &mut budget,
                    )?,
                }
            } else {
                let done = *mob;
                stack.pop();
                if let Some(visit) = visits.get_mut(&done) {
                    visit.active = false;
                }
            }
        }
    }
    let remap = |mob: Mob| -> Result<Mob, RenderSnapshotError> {
        visits
            .get(&mob)
            .map(|visit| Mob::from_parts(snapshot.stage_id, visit.index, 0))
            .ok_or(RenderSnapshotError::Graph(StageError::StaleHandle))
    };
    let mut slots = Vec::new();
    budget.reserve(&mut slots, order.len(), "projected slots")?;
    for &mob in &order {
        let source = entry(snapshot, mob)?;
        let mut children = Vec::new();
        budget.reserve(&mut children, source.submobjects.len(), "child links")?;
        for &child in &source.submobjects {
            children.push(remap(child)?);
        }
        let mut identities = Vec::new();
        budget.reserve(&mut identities, children.len(), "child uniqueness")?;
        identities.extend(children.iter().map(|child| child.parts().0));
        identities.sort_unstable();
        if identities.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(RenderSnapshotError::InvalidGraph("duplicate child edge"));
        }
        let mut buffer = source.buffer.snapshot_clone();
        // Interpolation locks govern future writes, not the already-observed
        // record values. Removing them also avoids serializing execution history.
        buffer.unlock_data();
        // Preserve exactly the validity decision made for the frozen frame;
        // never revive a hint invalidated by point edits or writable views.
        let hint_is_live = source.shape.point_revision.is_some()
            && source.shape.point_revision == source.buffer.field_revision("point")
            && !source.buffer.writable_view_affects("point");
        let shape = ShapeSlot {
            tag: source.shape.tag,
            point_revision: hint_is_live
                .then(|| buffer.field_revision("point"))
                .flatten(),
        };
        slots.push((
            0,
            Some(SnapshotEntry {
                buffer,
                placement: source.placement,
                placement_revision: 0,
                submobjects: children,
                parents: Vec::new(),
                updaters: Vec::new(),
                updater_states: Vec::new(),
                updating_suspended: false,
                is_animating: false,
                tracker: None,
                target: None,
                saved_state: None,
                pins: 0,
                pending_delete: false,
                uniforms: source.uniforms,
                z_index: source.z_index,
                shape,
                render_primitive: source.render_primitive,
                image: source.image.clone(),
                image_revision: u64::from(source.image.is_some()),
            }),
        ));
    }
    // Rebuild reciprocal parents in canonical source/child order. Filtering
    // the original parent list would leak unrooted owners and attachment order.
    for &parent in &order {
        for &child in &entry(snapshot, parent)?.submobjects {
            let index = visits[&child].index as usize;
            let target = slots[index]
                .1
                .as_mut()
                .ok_or(RenderSnapshotError::Graph(StageError::StaleHandle))?;
            budget.reserve(&mut target.parents, 1, "parent links")?;
            target.parents.push(remap(parent)?);
        }
    }
    let mut roots = Vec::new();
    budget.reserve(&mut roots, snapshot.roots.len(), "scene roots")?;
    for &root in &snapshot.roots {
        roots.push(remap(root)?);
    }
    Ok(Snapshot {
        stage_id: snapshot.stage_id,
        next_updater_id: 1,
        slots,
        free: Vec::new(),
        roots,
    })
}

impl Snapshot {
    /// Canonical bytes of this frame's renderable state, independent of
    /// unrooted arena history, handle generations and updater registration.
    ///
    /// The existing [`Snapshot::from_bytes`] decodes these bytes. This is a
    /// render-only artifact: handles are remapped and executable/restoration
    /// state is intentionally absent. For undo, scene state, pure animation
    /// endpoints and replay barriers, keep using [`Snapshot::to_bytes`].
    ///
    /// # Errors
    /// A malformed rooted graph, graph workspace refusal, allocation failure,
    /// or a refusal from the standard snapshot encoder.
    pub fn to_render_bytes(&self) -> Result<Vec<u8>, RenderSnapshotError> {
        self.to_render_bytes_with_limits(RenderSnapshotLimits::DEFAULT)
    }

    /// Render-only encoding with an explicit graph workspace budget.
    ///
    /// # Errors
    /// As [`Snapshot::to_render_bytes`].
    pub fn to_render_bytes_with_limits(
        &self,
        limits: RenderSnapshotLimits,
    ) -> Result<Vec<u8>, RenderSnapshotError> {
        project(self, limits)?
            .to_bytes()
            .map_err(RenderSnapshotError::Serial)
    }
}

#[cfg(test)]
#[path = "render_snapshot_tests.rs"]
mod tests;
