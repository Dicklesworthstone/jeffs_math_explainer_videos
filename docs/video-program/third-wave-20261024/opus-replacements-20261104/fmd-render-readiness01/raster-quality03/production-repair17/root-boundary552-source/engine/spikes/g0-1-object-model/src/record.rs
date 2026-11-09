//! The RecordBuffer prototype and the §8.2 view protocol.
//!
//! Authoritative mobject data is interleaved f32 records (the representation
//! zero-copy NumPy structured export requires). The protocol this spike
//! ratifies:
//!
//! 1. **Storage generations are fixed-capacity.** Record data lives in a
//!    `Box<[f32]>` behind an `Arc` — an allocation that can never grow, so
//!    reallocation under a live view is impossible *by construction*, not by
//!    discipline. Resize always allocates a fresh generation and swaps it in
//!    (copy-on-resize); detached views keep the old generation alive.
//! 2. **Views pin their generation.** A [`RecordView`] holds the `Arc`, so
//!    the memory NumPy would point at outlives any engine-side mutation of
//!    the owning mobject.
//! 3. **Live views see in-place writes; detached views stop at the swap.**
//!    That is NumPy-natural: `resize` gives you a new array, old views keep
//!    the old data.
//! 4. **Mutation through a view marks render state dirty** via the storage
//!    revision counter — the lazy, revisioned render mirrors compare
//!    revisions, they are never notified eagerly.
//! 5. **A generation is never shared by a snapshot and a live view at the
//!    same time.** Writers and view-exporters unshare first (clone the
//!    generation) whenever a snapshot holds it; snapshots eagerly copy
//!    buffers that have live views. This is the R12 resolution: snapshot
//!    cost is O(viewed objects) eager + O(1) CoW for everything else.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// One field of the interleaved record (name + f32 lane count).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: &'static str,
    pub width: usize,
}

/// The record layout. Custom user dtypes ride the same machinery — a schema
/// is data, not code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSchema {
    fields: Vec<FieldSpec>,
    stride: usize,
}

impl RecordSchema {
    #[must_use]
    pub fn new(fields: Vec<FieldSpec>) -> Self {
        let stride = fields.iter().map(|f| f.width).sum();
        Self { fields, stride }
    }

    /// The Reference's core VMobject layout, abridged for the spike.
    #[must_use]
    pub fn manim_default() -> Self {
        Self::new(vec![
            FieldSpec {
                name: "point",
                width: 3,
            },
            FieldSpec {
                name: "rgba",
                width: 4,
            },
        ])
    }

    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Lane offset of `field` within a record.
    #[must_use]
    pub fn offset(&self, field: &str) -> Option<usize> {
        let mut off = 0;
        for f in &self.fields {
            if f.name == field {
                return Some(off);
            }
            off += f.width;
        }
        None
    }

    #[must_use]
    pub fn field_width(&self, field: &str) -> Option<usize> {
        self.fields
            .iter()
            .find(|f| f.name == field)
            .map(|f| f.width)
    }

    /// Field names in record order (the G0-5 bridge iterates the schema of
    /// Python-declared dtypes; production W3 keeps this surface).
    pub fn field_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.fields.iter().map(|f| f.name)
    }
}

/// One fixed-capacity storage generation.
#[derive(Debug)]
pub struct Storage {
    cells: RwLock<Box<[f32]>>,
    /// Bumped on every write through any path (buffer or view): the signal
    /// the lazy render mirrors compare against.
    revision: AtomicU64,
    /// Live views attached to this generation (read or write).
    views: AtomicUsize,
}

impl Storage {
    fn new(cells: Box<[f32]>) -> Arc<Self> {
        Arc::new(Self {
            cells: RwLock::new(cells),
            revision: AtomicU64::new(0),
            views: AtomicUsize::new(0),
        })
    }

    fn snapshot_cells(&self) -> Box<[f32]> {
        self.cells.read().expect("storage lock poisoned").clone()
    }
}

/// Interleaved f32 records with the view protocol.
#[derive(Debug)]
pub struct RecordBuffer {
    schema: Arc<RecordSchema>,
    storage: Arc<Storage>,
    len: usize,
}

impl RecordBuffer {
    #[must_use]
    pub fn new(schema: RecordSchema, len: usize) -> Self {
        let stride = schema.stride();
        Self {
            schema: Arc::new(schema),
            storage: Storage::new(vec![0.0; len * stride].into_boxed_slice()),
            len,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub fn schema(&self) -> &RecordSchema {
        &self.schema
    }

    /// Current storage revision (the dirty signal for lazy mirrors).
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.storage.revision.load(Ordering::Acquire)
    }

    /// Stable identity of the current storage generation — test hook for the
    /// CoW / detach assertions.
    #[must_use]
    pub fn storage_id(&self) -> usize {
        Arc::as_ptr(&self.storage) as usize
    }

    #[must_use]
    pub fn live_view_count(&self) -> usize {
        self.storage.views.load(Ordering::Acquire)
    }

    fn shared_beyond_views(&self) -> bool {
        // Snapshot clones hold Arcs but never register as views; any strong
        // count beyond ourselves + live views is snapshot sharing.
        Arc::strong_count(&self.storage) > 1 + self.live_view_count()
    }

    /// Unshare from snapshots (rule 5): clone the generation if any snapshot
    /// holds it. Live views never trigger this — they are *supposed* to see
    /// writes.
    fn unshare(&mut self) {
        if self.shared_beyond_views() {
            let cells = self.storage.snapshot_cells();
            self.storage = Storage::new(cells);
        }
    }

    /// Read `field` of record `index` into a small vector.
    #[must_use]
    pub fn read(&self, index: usize, field: &str) -> Option<Vec<f32>> {
        let off = self.schema.offset(field)?;
        let width = self.schema.field_width(field)?;
        if index >= self.len {
            return None;
        }
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        let start = index * self.schema.stride() + off;
        Some(cells[start..start + width].to_vec())
    }

    /// Write `field` of record `index` in place (unsharing from snapshots
    /// first), bumping the revision.
    pub fn write(&mut self, index: usize, field: &str, values: &[f32]) -> bool {
        let Some(off) = self.schema.offset(field) else {
            return false;
        };
        let Some(width) = self.schema.field_width(field) else {
            return false;
        };
        if index >= self.len || values.len() != width {
            return false;
        }
        self.unshare();
        {
            let mut cells = self.storage.cells.write().expect("storage lock poisoned");
            let start = index * self.schema.stride() + off;
            cells[start..start + width].copy_from_slice(values);
        }
        self.storage.revision.fetch_add(1, Ordering::AcqRel);
        true
    }

    /// Copy-on-resize: allocate a fresh generation (copy prefix, null-pad
    /// growth), swap it in. Outstanding views keep the old generation —
    /// pinned, detached, NumPy-natural.
    pub fn resize(&mut self, new_len: usize) {
        let stride = self.schema.stride();
        let mut cells = vec![0.0f32; new_len * stride];
        {
            let old = self.storage.cells.read().expect("storage lock poisoned");
            let keep = old.len().min(cells.len());
            cells[..keep].copy_from_slice(&old[..keep]);
        }
        self.storage = Storage::new(cells.into_boxed_slice());
        self.len = new_len;
    }

    /// Export a live view (the PyO3 bridge's NumPy structured view stands
    /// here). Unshares from snapshots first so rule 5 holds.
    pub fn export_view(&mut self, writable: bool) -> RecordView {
        self.unshare();
        self.storage.views.fetch_add(1, Ordering::AcqRel);
        RecordView {
            schema: Arc::clone(&self.schema),
            storage: Arc::clone(&self.storage),
            len: self.len,
            writable,
        }
    }

    /// Snapshot clone (rule 5): share the generation unless live views make
    /// eager copy necessary.
    #[must_use]
    pub fn snapshot_clone(&self) -> Self {
        if self.live_view_count() > 0 {
            let cells = self.storage.snapshot_cells();
            Self {
                schema: Arc::clone(&self.schema),
                storage: Storage::new(cells),
                len: self.len,
            }
        } else {
            Self {
                schema: Arc::clone(&self.schema),
                storage: Arc::clone(&self.storage),
                len: self.len,
            }
        }
    }

    /// Independent deep copy (used by mobject `copy()`).
    #[must_use]
    pub fn deep_clone(&self) -> Self {
        Self {
            schema: Arc::clone(&self.schema),
            storage: Storage::new(self.storage.snapshot_cells()),
            len: self.len,
        }
    }
}

/// A live view over one storage generation — the Rust model of the exported
/// NumPy structured array. Dropping it unpins the generation.
#[derive(Debug)]
pub struct RecordView {
    schema: Arc<RecordSchema>,
    storage: Arc<Storage>,
    len: usize,
    writable: bool,
}

impl RecordView {
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether this view still aliases `buffer`'s current generation (false
    /// once a resize/restore has swapped generations).
    #[must_use]
    pub fn is_attached_to(&self, buffer: &RecordBuffer) -> bool {
        Arc::ptr_eq(&self.storage, &buffer.storage)
    }

    #[must_use]
    pub fn read(&self, index: usize, field: &str) -> Option<Vec<f32>> {
        let off = self.schema.offset(field)?;
        let width = self.schema.field_width(field)?;
        if index >= self.len {
            return None;
        }
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        let start = index * self.schema.stride() + off;
        Some(cells[start..start + width].to_vec())
    }

    /// Write through the view: visible to the engine while attached, and it
    /// marks render state dirty via the revision (rule 4).
    pub fn write(&self, index: usize, field: &str, values: &[f32]) -> bool {
        if !self.writable {
            return false;
        }
        let Some(off) = self.schema.offset(field) else {
            return false;
        };
        let Some(width) = self.schema.field_width(field) else {
            return false;
        };
        if index >= self.len || values.len() != width {
            return false;
        }
        {
            let mut cells = self.storage.cells.write().expect("storage lock poisoned");
            let start = index * self.schema.stride() + off;
            cells[start..start + width].copy_from_slice(values);
        }
        self.storage.revision.fetch_add(1, Ordering::AcqRel);
        true
    }
}

impl Drop for RecordView {
    fn drop(&mut self) {
        self.storage.views.fetch_sub(1, Ordering::AcqRel);
    }
}
