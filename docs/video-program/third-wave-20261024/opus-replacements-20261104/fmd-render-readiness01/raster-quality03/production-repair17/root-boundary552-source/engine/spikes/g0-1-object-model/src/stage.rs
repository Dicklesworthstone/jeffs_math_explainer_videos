//! The Stage arena prototype: generational handles, rooted lifetimes,
//! multiple parents, copy remapping, proxy pinning, updaters, and CoW
//! snapshots — the §8.1 ownership model this spike ratifies (D-11).
//!
//! Lifetime rules proved here:
//! - Entries are **arena-owned**. Scene membership (`roots`) is a root set,
//!   not ownership: removing from the scene never frees anything.
//! - `Mob` handles are `Copy`, generational, and **stage-scoped** (the
//!   two-scene policy): a handle from another stage resolves to `None`;
//!   moving content between stages is `copy_into`.
//! - Explicit `delete` is the only destructor, and it defers while proxy
//!   pins are outstanding — the Python bridge's identity story: a proxy
//!   pins its entry, so collection round-trips keep handle → object stable.
//! - Updaters are shared **by reference** on copy (manim's documented copy
//!   semantics) via `Rc`; they receive the stage and their own handle, so
//!   closures capture plain `Mob` values.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::mobject::Mobject;
use crate::record::RecordBuffer;

static NEXT_STAGE_ID: AtomicU64 = AtomicU64::new(1);

/// Generational, stage-scoped, `Copy` mobject handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mob {
    stage_id: u64,
    index: u32,
    generation: u32,
}

impl Mob {
    /// The raw `(stage_id, index, generation)` triple — the stable identity
    /// the G0-5 bridge exposes for hashing/equality observability. Opaque
    /// to semantics: only resolution through a `Stage` means anything.
    #[must_use]
    pub fn token(self) -> (u64, u32, u32) {
        (self.stage_id, self.index, self.generation)
    }
}

/// An updater closure: receives the stage, its own handle, and `dt`.
/// `Rc` because manim's `copy()` keeps updater callables by reference.
pub type Updater = Rc<RefCell<dyn FnMut(&mut Stage, Mob, f64)>>;

/// Arena entry: the mobject's data plus its graph edges and lifetime state.
pub struct Entry {
    pub buffer: RecordBuffer,
    pub submobjects: Vec<Mob>,
    pub parents: Vec<Mob>,
    updaters: Vec<Updater>,
    pins: usize,
    pending_delete: bool,
}

impl Entry {
    fn from_data(buffer: RecordBuffer) -> Self {
        Self {
            buffer,
            submobjects: Vec::new(),
            parents: Vec::new(),
            updaters: Vec::new(),
            pins: 0,
            pending_delete: false,
        }
    }
}

struct Slot {
    generation: u32,
    entry: Option<Entry>,
}

/// A CoW snapshot of the whole stage (begin-state for pure segments,
/// checkpoint for the Studio journal).
pub struct Snapshot {
    slots: Vec<(u32, Option<SnapshotEntry>)>,
    free: Vec<u32>,
    roots: Vec<Mob>,
}

struct SnapshotEntry {
    buffer: RecordBuffer,
    submobjects: Vec<Mob>,
    parents: Vec<Mob>,
    updaters: Vec<Updater>,
    pins: usize,
    pending_delete: bool,
}

/// The arena.
pub struct Stage {
    id: u64,
    slots: Vec<Slot>,
    free: Vec<u32>,
    roots: Vec<Mob>,
    time: f64,
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: NEXT_STAGE_ID.fetch_add(1, Ordering::Relaxed),
            slots: Vec::new(),
            free: Vec::new(),
            roots: Vec::new(),
            time: 0.0,
        }
    }

    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    // ------------------------------------------------------------ handles

    fn alloc(&mut self, entry: Entry) -> Mob {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.entry = Some(entry);
            Mob {
                stage_id: self.id,
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                entry: Some(entry),
            });
            Mob {
                stage_id: self.id,
                index,
                generation: 0,
            }
        }
    }

    fn slot(&self, mob: Mob) -> Option<&Slot> {
        if mob.stage_id != self.id {
            return None; // two-scene policy: foreign handles never resolve
        }
        let slot = self.slots.get(mob.index as usize)?;
        (slot.generation == mob.generation).then_some(slot)
    }

    /// Resolve a handle. `None` for stale, deleted, or foreign handles.
    #[must_use]
    pub fn get(&self, mob: Mob) -> Option<&Entry> {
        self.slot(mob)?.entry.as_ref()
    }

    #[must_use]
    pub fn get_mut(&mut self, mob: Mob) -> Option<&mut Entry> {
        if mob.stage_id != self.id {
            return None;
        }
        let slot = self.slots.get_mut(mob.index as usize)?;
        if slot.generation != mob.generation {
            return None;
        }
        slot.entry.as_mut()
    }

    #[must_use]
    pub fn contains(&self, mob: Mob) -> bool {
        self.get(mob).is_some()
    }

    // ----------------------------------------------------- add and compose

    /// Move a detached mobject (and its detached children, recursively) into
    /// the arena. Detached construction is just plain values before this;
    /// builders enter through `Into<Mobject>`.
    pub fn add(&mut self, mobject: impl Into<Mobject>) -> Mob {
        let Mobject {
            buffer,
            submobjects,
        } = mobject.into();
        let mob = self.alloc(Entry::from_data(buffer));
        for child in submobjects {
            let child_mob = self.add(child);
            self.attach(mob, child_mob);
        }
        mob
    }

    /// Add to the scene's draw list (idempotent). Rooting is membership,
    /// not ownership.
    pub fn add_to_scene(&mut self, mob: Mob) -> bool {
        if !self.contains(mob) {
            return false;
        }
        if !self.roots.contains(&mob) {
            self.roots.push(mob);
        }
        true
    }

    /// Remove from the draw list only; the entry and every handle stay
    /// valid.
    pub fn remove_from_scene(&mut self, mob: Mob) {
        self.roots.retain(|m| *m != mob);
    }

    #[must_use]
    pub fn roots(&self) -> &[Mob] {
        &self.roots
    }

    /// Attach `child` under `parent`. A child may have any number of
    /// parents; the family DAG is deduplicated per edge.
    pub fn attach(&mut self, parent: Mob, child: Mob) -> bool {
        if !self.contains(parent) || !self.contains(child) || parent == child {
            return false;
        }
        {
            let entry = self.get_mut(parent).expect("checked above");
            if entry.submobjects.contains(&child) {
                return true;
            }
            entry.submobjects.push(child);
        }
        let child_entry = self.get_mut(child).expect("checked above");
        if !child_entry.parents.contains(&parent) {
            child_entry.parents.push(parent);
        }
        true
    }

    /// Detach `child` from `parent` (the entry itself stays alive).
    pub fn detach(&mut self, parent: Mob, child: Mob) {
        if let Some(entry) = self.get_mut(parent) {
            entry.submobjects.retain(|m| *m != child);
        }
        if let Some(entry) = self.get_mut(child) {
            entry.parents.retain(|m| *m != parent);
        }
    }

    /// The family under `mob` in depth-first order, each member once even
    /// through diamond composition (multiple parents).
    #[must_use]
    pub fn family(&self, mob: Mob) -> Vec<Mob> {
        let mut out = Vec::new();
        let mut stack = vec![mob];
        while let Some(current) = stack.pop() {
            if out.contains(&current) {
                continue;
            }
            if let Some(entry) = self.get(current) {
                out.push(current);
                for &child in entry.submobjects.iter().rev() {
                    stack.push(child);
                }
            }
        }
        out
    }

    // ------------------------------------------------------------ lifetime

    /// Pin an entry (the Python proxy holds one pin for its lifetime).
    pub fn pin(&mut self, mob: Mob) -> bool {
        match self.get_mut(mob) {
            Some(entry) => {
                entry.pins += 1;
                true
            }
            None => false,
        }
    }

    /// Release a pin; a deferred delete completes when the last pin drops.
    pub fn unpin(&mut self, mob: Mob) {
        let finalize = match self.get_mut(mob) {
            Some(entry) => {
                entry.pins = entry.pins.saturating_sub(1);
                entry.pins == 0 && entry.pending_delete
            }
            None => false,
        };
        if finalize {
            self.finalize_delete(mob);
        }
    }

    /// Explicit destruction — the only way an entry dies. While pins are
    /// outstanding the delete defers (the handle stays live for the
    /// proxies); it completes on the last unpin.
    pub fn delete(&mut self, mob: Mob) -> bool {
        let deferred = match self.get_mut(mob) {
            Some(entry) => {
                if entry.pins > 0 {
                    entry.pending_delete = true;
                    true
                } else {
                    false
                }
            }
            None => return false,
        };
        if !deferred {
            self.finalize_delete(mob);
        }
        true
    }

    fn finalize_delete(&mut self, mob: Mob) {
        // Unlink from parents, children, and the root set first.
        let entry_edges = self
            .get(mob)
            .map(|e| (e.parents.clone(), e.submobjects.clone()));
        let Some((parents, children)) = entry_edges else {
            return;
        };
        for parent in parents {
            if let Some(p) = self.get_mut(parent) {
                p.submobjects.retain(|m| *m != mob);
            }
        }
        for child in children {
            if let Some(c) = self.get_mut(child) {
                c.parents.retain(|m| *m != mob);
            }
        }
        self.remove_from_scene(mob);
        let slot = &mut self.slots[mob.index as usize];
        slot.entry = None;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(mob.index);
    }

    // ---------------------------------------------------------------- copy

    /// manim `copy()`: deep-copy the family subtree, remapping
    /// family-internal references; updater callables shared by reference;
    /// record data independent. External parents are not copied — the copy
    /// is a detached family.
    pub fn copy_family(&mut self, mob: Mob) -> Option<Mob> {
        if !self.contains(mob) {
            return None;
        }
        let family = self.family(mob);
        let mut map: HashMap<Mob, Mob> = HashMap::new();
        for &old in &family {
            let entry = self.get(old).expect("family members resolve");
            let new_entry = Entry {
                buffer: entry.buffer.deep_clone(),
                submobjects: entry.submobjects.clone(), // remapped below
                parents: entry.parents.clone(),         // remapped below
                updaters: entry.updaters.clone(),       // by reference
                pins: 0,
                pending_delete: false,
            };
            let new = self.alloc(new_entry);
            map.insert(old, new);
        }
        for new in map.values() {
            let remap = |edges: &mut Vec<Mob>| {
                let mut seen = Vec::new();
                edges.retain_mut(|m| match map.get(m) {
                    Some(mapped) => {
                        *m = *mapped;
                        if seen.contains(mapped) {
                            false
                        } else {
                            seen.push(*mapped);
                            true
                        }
                    }
                    // Family-external references are dropped: the copy
                    // is detached (external parents stay with the
                    // original).
                    None => false,
                });
            };
            let entry = self.get_mut(*new).expect("just allocated");
            remap(&mut entry.submobjects);
            remap(&mut entry.parents);
        }
        map.get(&mob).copied()
    }

    /// Cross-stage transfer under the two-scene policy: content moves by
    /// copy, never by handle.
    pub fn copy_into(&self, mob: Mob, target: &mut Stage) -> Option<Mob> {
        let entry = self.get(mob)?;
        let children: Vec<Mob> = entry.submobjects.clone();
        let new = target.alloc(Entry::from_data(entry.buffer.deep_clone()));
        for child in children {
            if let Some(new_child) = self.copy_into(child, target) {
                target.attach(new, new_child);
            }
        }
        Some(new)
    }

    // ------------------------------------------------------------ updaters

    /// Insertion-ordered updaters. `call_now` runs the updater once,
    /// immediately — exactly once (the Reference's double-call is a bug we
    /// fix, Behavior Note).
    pub fn add_updater(
        &mut self,
        mob: Mob,
        updater: impl FnMut(&mut Stage, Mob, f64) + 'static,
        call_now: bool,
    ) -> bool {
        let updater: Updater = Rc::new(RefCell::new(updater));
        match self.get_mut(mob) {
            Some(entry) => entry.updaters.push(Rc::clone(&updater)),
            None => return false,
        }
        if call_now {
            updater.borrow_mut()(self, mob, 0.0);
        }
        true
    }

    /// Run every rooted family's updaters in insertion order, then advance
    /// time. (The real six-step frame order is Choreo's; the spike proves
    /// only that handle-capturing closures compose with the arena.)
    pub fn update(&mut self, dt: f64) {
        let targets: Vec<Mob> = self
            .roots
            .clone()
            .into_iter()
            .flat_map(|root| self.family(root))
            .collect();
        for target in targets {
            let updaters = match self.get(target) {
                Some(entry) => entry.updaters.clone(),
                None => continue,
            };
            for updater in updaters {
                updater.borrow_mut()(self, target, dt);
            }
        }
        self.time += dt;
    }

    // ----------------------------------------------------------- snapshots

    /// CoW snapshot of the whole stage: record storages are shared until
    /// someone writes (entries with live views are eagerly copied — rule 5
    /// of the view protocol).
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            slots: self
                .slots
                .iter()
                .map(|slot| {
                    (
                        slot.generation,
                        slot.entry.as_ref().map(|entry| SnapshotEntry {
                            buffer: entry.buffer.snapshot_clone(),
                            submobjects: entry.submobjects.clone(),
                            parents: entry.parents.clone(),
                            updaters: entry.updaters.clone(),
                            pins: entry.pins,
                            pending_delete: entry.pending_delete,
                        }),
                    )
                })
                .collect(),
            free: self.free.clone(),
            roots: self.roots.clone(),
        }
    }

    /// Restore a snapshot. Outstanding views keep the storage generation
    /// they were exported against (detached, NumPy-natural) — restoring
    /// never mutates memory a view can see.
    pub fn restore(&mut self, snapshot: &Snapshot) {
        self.slots = snapshot
            .slots
            .iter()
            .map(|(generation, entry)| Slot {
                generation: *generation,
                entry: entry.as_ref().map(|e| Entry {
                    buffer: e.buffer.snapshot_clone(),
                    submobjects: e.submobjects.clone(),
                    parents: e.parents.clone(),
                    updaters: e.updaters.clone(),
                    pins: e.pins,
                    pending_delete: e.pending_delete,
                }),
            })
            .collect();
        self.free = snapshot.free.clone();
        self.roots = snapshot.roots.clone();
    }
}
