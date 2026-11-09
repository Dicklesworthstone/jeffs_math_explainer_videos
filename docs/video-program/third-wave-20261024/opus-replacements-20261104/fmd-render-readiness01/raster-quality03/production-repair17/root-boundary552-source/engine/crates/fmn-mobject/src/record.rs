//! The RecordBuffer: Marionette's data plane (§8.2, §1.1, D-20) — the
//! authoritative interleaved f32 buffer, the G0-1-ratified view protocol,
//! and the lazy revisioned render mirrors.
//!
//! **Layer 1 — the authoritative buffer.** Interleaved f32 records matching
//! the Reference dtypes exactly ([`RecordSchema::mobject`],
//! [`RecordSchema::vmobject`]); user-declared custom dtypes ride the same
//! schema machinery. The interleaved layout is API surface: `mobject.data`
//! exports as a zero-copy NumPy structured array (byte offsets locked by
//! test against numpy's packing). `aligned_data_keys` /
//! `pointlike_data_keys`, data locking as copy-elision markers,
//! resize-with-interpolation, and null-padding all live here.
//!
//! **Layer 2 — the view protocol** (V1–V6 of
//! `docs/g0/G0-1-object-model-ratification.md`):
//! - V1 storage generations are fixed-capacity `Arc`-owned allocations;
//!   growth swaps in a fresh generation (reallocation under a live view is
//!   impossible by construction);
//! - V2 views pin their generation;
//! - V3 live views alias the current generation, resize/restore detaches
//!   them with NumPy-natural semantics;
//! - V4 every engine-mediated write bumps revision counters — render state is
//!   dirty by comparison, never by eager notification; because a foreign
//!   zero-copy writer cannot make that callback, writable-view lifetime forces
//!   refresh and detaching one conservatively bumps every field it exposed;
//! - V5 a generation is never simultaneously snapshot-shared and
//!   view-aliased (writers/view-exporters unshare; snapshots eagerly copy
//!   viewed buffers);
//! - V6 restore behaves as resize.
//!
//! Views may be whole-buffer or **field-scoped**. While a *writable* view
//! exists, the affected scope is conservatively treated as dirty at every
//! observation — a live view never silently receives weaker semantics.
//! Ranged writes ([`RecordBuffer::write_range`]) are the precise-dirty
//! opt-in; per-field dirty spans accumulate for the render IR.
//!
//! **Layer 3 — the render mirrors** ([`MirrorSet`]): lazy, revisioned,
//! lane-major (struct-of-arrays) copies of individual fields. Hot loops
//! read mirrors, never the interleaved buffer; a mirror rematerializes only
//! when its field's revision moved, its storage generation changed, or a
//! writable view forces conservative refresh. AoSoA layouts sized to the
//! SIMD tier derive from these mirrors in Lumen (§10.8, §17.3).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use fnp_ndarray::NdLayout;

use crate::numpy::{
    ArrayDesc, ExportError, RecordArray, RecordArrayMut, RecordDType, field_layout, whole_layout,
};

/// Errors from record sizing (fm-vek.2). Every construction and resize
/// path proves its lane and byte counts with checked arithmetic *before*
/// allocating or mutating, and refuses with these typed errors instead of
/// wrapping `usize` or aborting on an impossible capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordError {
    /// The declared field widths summed past `usize::MAX` lanes.
    StrideOverflow,
    /// `len * stride` lanes — or their four-byte size — exceeds what a
    /// single allocation can address (`usize::MAX`, and the `isize::MAX`
    /// byte capacity ceiling every Rust allocation obeys).
    SizeOverflow {
        /// Requested record count.
        len: usize,
        /// Lanes per record.
        stride: usize,
    },
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StrideOverflow => write!(f, "record field widths overflow usize"),
            Self::SizeOverflow { len, stride } => write!(
                f,
                "record buffer of {len} records x {stride} lanes exceeds addressable size"
            ),
        }
    }
}

impl std::error::Error for RecordError {}

/// The lane count for `len` records of `stride` lanes, refusing anything
/// whose allocation could not exist (fm-vek.2): the lane product must fit
/// `usize`, and the four-byte cell count must fit the `isize::MAX` byte
/// ceiling `Vec` enforces — checked here so the refusal is a typed error,
/// not a capacity abort.
fn checked_lanes(len: usize, stride: usize) -> Result<usize, RecordError> {
    let lanes = len
        .checked_mul(stride)
        .ok_or(RecordError::SizeOverflow { len, stride })?;
    let bytes = lanes
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or(RecordError::SizeOverflow { len, stride })?;
    if bytes > isize::MAX as usize {
        return Err(RecordError::SizeOverflow { len, stride });
    }
    Ok(lanes)
}

/// One field of the interleaved record (name + f32 lane count).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    /// Field name (`"point"`, `"stroke_rgba"`, … or a user-declared name).
    pub name: String,
    /// Number of f32 lanes.
    pub width: usize,
}

/// The record layout plus the alignment/pointlike key sets the family
/// machinery consumes. Custom user dtypes construct these directly — a
/// schema is data, not code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSchema {
    fields: Vec<FieldSpec>,
    stride: usize,
    aligned_keys: Vec<String>,
    pointlike_keys: Vec<String>,
}

impl RecordSchema {
    /// A schema from `(name, width)` pairs, with explicit aligned and
    /// pointlike key sets.
    ///
    /// # Errors
    /// [`RecordError::StrideOverflow`] when the field widths sum past
    /// `usize::MAX` lanes (fm-vek.2: the stride is API surface — NumPy's
    /// itemsize derives from it — so it is proved, not assumed).
    pub fn new(
        fields: &[(&str, usize)],
        aligned_keys: &[&str],
        pointlike_keys: &[&str],
    ) -> Result<Self, RecordError> {
        let fields: Vec<FieldSpec> = fields
            .iter()
            .map(|(name, width)| FieldSpec {
                name: (*name).to_string(),
                width: *width,
            })
            .collect();
        let stride = fields.iter().try_fold(0usize, |stride, field| {
            stride
                .checked_add(field.width)
                .ok_or(RecordError::StrideOverflow)
        })?;
        Ok(Self {
            fields,
            stride,
            aligned_keys: aligned_keys.iter().map(|k| (*k).to_string()).collect(),
            pointlike_keys: pointlike_keys.iter().map(|k| (*k).to_string()).collect(),
        })
    }

    /// The Reference's `Mobject.data_dtype`:
    /// `[('point', f32, 3), ('rgba', f32, 4)]`,
    /// `aligned_data_keys = pointlike_data_keys = ['point']`.
    #[must_use]
    pub fn mobject() -> Self {
        Self::new(&[("point", 3), ("rgba", 4)], &["point"], &["point"])
            .expect("the built-in mobject schema is seven lanes")
    }

    /// The Reference's `VMobject.data_dtype`, field for field.
    #[must_use]
    pub fn vmobject() -> Self {
        Self::new(
            &[
                ("point", 3),
                ("stroke_rgba", 4),
                ("stroke_width", 1),
                ("joint_angle", 1),
                ("fill_rgba", 4),
                ("base_normal", 3),
                ("fill_border_width", 1),
            ],
            &["point"],
            &["point"],
        )
        .expect("the built-in vmobject schema is seventeen lanes")
    }

    /// f32 lanes per record. `stride * 4` is the NumPy itemsize.
    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[must_use]
    pub fn fields(&self) -> &[FieldSpec] {
        &self.fields
    }

    /// `aligned_data_keys` — the fields the family-alignment machinery
    /// null-pads together.
    #[must_use]
    pub fn aligned_keys(&self) -> &[String] {
        &self.aligned_keys
    }

    /// `pointlike_data_keys` — the fields point transforms apply to.
    #[must_use]
    pub fn pointlike_keys(&self) -> &[String] {
        &self.pointlike_keys
    }

    fn index_of(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == field)
    }

    /// Lane offset of `field` within a record (NumPy byte offset ÷ 4 —
    /// numpy packs all-f32 subarray dtypes contiguously, locked by test).
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

    /// Lane count of `field`.
    #[must_use]
    pub fn field_width(&self, field: &str) -> Option<usize> {
        self.fields
            .iter()
            .find(|f| f.name == field)
            .map(|f| f.width)
    }
}

/// Inclusive dirty span of record indices, per field, since last take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirtySpan {
    min: usize,
    max: usize,
}

/// One fixed-capacity storage generation (V1).
#[derive(Debug)]
struct Storage {
    cells: RwLock<Box<[f32]>>,
    /// V4: bumped on every write through any path.
    revision: AtomicU64,
    /// V4, per field: bumped when that field is written.
    field_revisions: Vec<AtomicU64>,
    /// Live views attached to this generation (any kind).
    views: AtomicUsize,
    /// Writable whole-buffer views (forces conservative refresh of every
    /// field while nonzero).
    writable_whole_views: AtomicUsize,
    /// Writable field-scoped views, per field.
    writable_field_views: Vec<AtomicUsize>,
    /// Set on a snapshot copy of a generation that had writable views
    /// attached (whole-buffer, then per field). Those views could write
    /// without an engine callback, so the copy's revisions cannot vouch for
    /// that scope: it stays view-dirty at every observation of the copy.
    snapshot_view_dirty_whole: AtomicBool,
    snapshot_view_dirty_fields: Vec<AtomicBool>,
    /// Per-field dirty spans (record indices) since last take.
    dirty_spans: Mutex<Vec<Option<DirtySpan>>>,
}

impl Storage {
    fn new(cells: Box<[f32]>, n_fields: usize) -> Arc<Self> {
        Arc::new(Self {
            cells: RwLock::new(cells),
            revision: AtomicU64::new(0),
            field_revisions: (0..n_fields).map(|_| AtomicU64::new(0)).collect(),
            views: AtomicUsize::new(0),
            writable_whole_views: AtomicUsize::new(0),
            writable_field_views: (0..n_fields).map(|_| AtomicUsize::new(0)).collect(),
            snapshot_view_dirty_whole: AtomicBool::new(false),
            snapshot_view_dirty_fields: (0..n_fields).map(|_| AtomicBool::new(false)).collect(),
            dirty_spans: Mutex::new(vec![None; n_fields]),
        })
    }

    /// V4 across V5's eager snapshot copy: the copy of a generation with
    /// writable views keeps reporting their scope as view-dirty. A copy of
    /// such a copy inherits it.
    fn clone_for_snapshot(&self) -> Arc<Self> {
        let fresh = self.clone_with_revisions();
        fresh.snapshot_view_dirty_whole.store(
            self.writable_whole_views.load(Ordering::Acquire) > 0
                || self.snapshot_view_dirty_whole.load(Ordering::Acquire),
            Ordering::Release,
        );
        for ((dirty, views), inherited) in fresh
            .snapshot_view_dirty_fields
            .iter()
            .zip(&self.writable_field_views)
            .zip(&self.snapshot_view_dirty_fields)
        {
            dirty.store(
                views.load(Ordering::Acquire) > 0 || inherited.load(Ordering::Acquire),
                Ordering::Release,
            );
        }
        fresh
    }

    fn clone_with_revisions(&self) -> Arc<Self> {
        let n_fields = self.field_revisions.len();
        let fresh = Self::new(self.copy_cells(), n_fields);
        fresh
            .revision
            .store(self.revision.load(Ordering::Acquire), Ordering::Release);
        for (new, old) in fresh
            .field_revisions
            .iter()
            .zip(self.field_revisions.iter())
        {
            new.store(old.load(Ordering::Acquire), Ordering::Release);
        }
        *fresh.dirty_spans.lock().expect("span lock poisoned") =
            self.dirty_spans.lock().expect("span lock poisoned").clone();
        fresh
    }

    fn copy_cells(&self) -> Box<[f32]> {
        self.cells.read().expect("storage lock poisoned").clone()
    }

    fn mark_written(&self, field_index: usize, first_record: usize, last_record: usize) {
        self.revision.fetch_add(1, Ordering::AcqRel);
        self.field_revisions[field_index].fetch_add(1, Ordering::AcqRel);
        let mut spans = self.dirty_spans.lock().expect("span lock poisoned");
        let span = &mut spans[field_index];
        *span = Some(match span {
            Some(s) => DirtySpan {
                min: s.min.min(first_record),
                max: s.max.max(last_record),
            },
            None => DirtySpan {
                min: first_record,
                max: last_record,
            },
        });
    }
}

/// Interleaved f32 records under the view protocol.
#[derive(Debug)]
pub struct RecordBuffer {
    schema: Arc<RecordSchema>,
    storage: Arc<Storage>,
    len: usize,
    /// The Reference's `_data_defaults`: one phantom record that styling
    /// writes land in while the buffer has no points, and that growth from
    /// zero adopts wholesale (`resize_points`, mobject.py). Schema-generic:
    /// exactly one stride of lanes, never counted by [`Self::len`].
    defaults: Option<Box<[f32]>>,
    /// `locked_data_keys` (field indices): the copy-elision markers the
    /// animation engine consults (`lock_matching_data` skips these in
    /// `interpolate`, fm-cye). State only — locking never gates access.
    locked: HashSet<usize>,
}

impl RecordBuffer {
    /// A zeroed buffer of `len` records over `schema`.
    ///
    /// # Errors
    /// [`RecordError::SizeOverflow`] when `len * stride` — or its byte
    /// size — exceeds what one allocation can address (fm-vek.2).
    pub fn new(schema: RecordSchema, len: usize) -> Result<Self, RecordError> {
        let stride = schema.stride();
        let n_fields = schema.fields().len();
        let lanes = checked_lanes(len, stride)?;
        Ok(Self {
            schema: Arc::new(schema),
            storage: Storage::new(vec![0.0; lanes].into_boxed_slice(), n_fields),
            len,
            defaults: None,
            locked: HashSet::new(),
        })
    }

    /// Number of records.
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

    /// Global storage revision (V4).
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.storage.revision.load(Ordering::Acquire)
    }

    /// Per-field revision (V4) — what the lazy mirrors compare.
    #[must_use]
    pub fn field_revision(&self, field: &str) -> Option<u64> {
        let index = self.schema.index_of(field)?;
        Some(self.storage.field_revisions[index].load(Ordering::Acquire))
    }

    /// Stable identity of the current storage generation. Test hook for
    /// CoW / detach / O(touched) assertions; mirrors also use it to detect
    /// generation swaps.
    #[must_use]
    pub fn storage_id(&self) -> usize {
        Arc::as_ptr(&self.storage) as usize
    }

    /// Live views attached to the current generation.
    #[must_use]
    pub fn live_view_count(&self) -> usize {
        self.storage.views.load(Ordering::Acquire)
    }

    /// Whether a writable whole-buffer view is attached (conservative
    /// invalidation: every field counts as dirty while one exists).
    #[must_use]
    pub fn has_writable_whole_view(&self) -> bool {
        self.storage.writable_whole_views.load(Ordering::Acquire) > 0
    }

    /// Whether a writable view scoped to `field` is attached.
    #[must_use]
    pub fn field_has_writable_view(&self, field: &str) -> bool {
        self.schema
            .index_of(field)
            .is_some_and(|i| self.storage.writable_field_views[i].load(Ordering::Acquire) > 0)
    }

    /// Whether a writable view can mutate `field` without an engine-mediated
    /// write callback, or, for a snapshot copy, could have before the copy
    /// was taken.
    ///
    /// A whole-buffer view affects every field; a field-scoped view affects
    /// only its declared field. Render consumers use this predicate rather than
    /// weakening field-scoped invalidation into object-wide invalidation. A
    /// frame snapshot copies a viewed buffer eagerly (V5) with its revisions,
    /// so without the snapshot flags a foreign write between two frames
    /// would reach the copy's cells but not its revisions.
    #[must_use]
    pub fn writable_view_affects(&self, field: &str) -> bool {
        self.has_writable_whole_view()
            || self.field_has_writable_view(field)
            || self
                .storage
                .snapshot_view_dirty_whole
                .load(Ordering::Acquire)
            || self
                .schema
                .index_of(field)
                .is_some_and(|i| self.storage.snapshot_view_dirty_fields[i].load(Ordering::Acquire))
    }

    fn shared_beyond_views(&self) -> bool {
        // Snapshot clones hold Arcs but never register as views; any strong
        // count beyond ourselves + live views is snapshot sharing.
        Arc::strong_count(&self.storage) > 1 + self.live_view_count()
    }

    /// V5: clone the generation if any snapshot holds it. Live views never
    /// trigger this — they are supposed to see writes. Counters and dirty
    /// spans carry over so mirror laziness stays correct across unshares.
    fn unshare(&mut self) {
        if self.shared_beyond_views() {
            self.storage = self.storage.clone_with_revisions();
        }
    }

    // ------------------------------------------------------------- access

    /// Read `field` of record `index`.
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

    /// Extract the whole column of `field`, record-major
    /// (`len * width` values) — the observation the mirror coherence tests
    /// compare against.
    #[must_use]
    pub fn read_column(&self, field: &str) -> Option<Vec<f32>> {
        let off = self.schema.offset(field)?;
        let width = self.schema.field_width(field)?;
        let stride = self.schema.stride();
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        let mut out = Vec::with_capacity(self.len * width);
        for record in 0..self.len {
            let start = record * stride + off;
            out.extend_from_slice(&cells[start..start + width]);
        }
        Some(out)
    }

    /// Compare one record-major field with another buffer without allocating
    /// temporary columns. Missing fields, incompatible widths, or different
    /// record counts are not equal. Values use the same `f32` equality as two
    /// [`RecordBuffer::read_column`] results, including NaN behavior.
    #[must_use]
    pub fn column_eq(&self, other: &Self, field: &str) -> bool {
        let (Some(off), Some(other_off), Some(width), Some(other_width)) = (
            self.schema.offset(field),
            other.schema.offset(field),
            self.schema.field_width(field),
            other.schema.field_width(field),
        ) else {
            return false;
        };
        if self.len != other.len || width != other_width {
            return false;
        }
        if Arc::ptr_eq(&self.storage, &other.storage) {
            return off == other_off && self.schema.stride() == other.schema.stride();
        }
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        let other_cells = other.storage.cells.read().expect("storage lock poisoned");
        (0..self.len).all(|record| {
            let start = record * self.schema.stride() + off;
            let other_start = record * other.schema.stride() + other_off;
            cells[start..start + width] == other_cells[other_start..other_start + width]
        })
    }

    /// Write `field` of record `index` (unsharing from snapshots first,
    /// V5), bumping global + field revisions (V4) and the dirty span.
    pub fn write(&mut self, index: usize, field: &str, values: &[f32]) -> bool {
        let Some(field_index) = self.schema.index_of(field) else {
            return false;
        };
        let off = self.schema.offset(field).expect("field exists");
        let width = self.schema.field_width(field).expect("field exists");
        if index >= self.len || values.len() != width {
            return false;
        }
        self.unshare();
        {
            let mut cells = self.storage.cells.write().expect("storage lock poisoned");
            let start = index * self.schema.stride() + off;
            cells[start..start + width].copy_from_slice(values);
        }
        self.storage.mark_written(field_index, index, index);
        true
    }

    /// Ranged write — the precise-dirty opt-in (`edit_points(range, …)`
    /// class): write `values` (a multiple of the field width) into
    /// consecutive records starting at `first_record`.
    pub fn write_range(&mut self, field: &str, first_record: usize, values: &[f32]) -> bool {
        let Some(field_index) = self.schema.index_of(field) else {
            return false;
        };
        let off = self.schema.offset(field).expect("field exists");
        let width = self.schema.field_width(field).expect("field exists");
        if width == 0 || !values.len().is_multiple_of(width) {
            return false;
        }

        let n_records = values.len() / width;
        if n_records == 0 {
            return true;
        }
        if first_record + n_records > self.len {
            return false;
        }
        self.unshare();
        {
            let mut cells = self.storage.cells.write().expect("storage lock poisoned");
            let stride = self.schema.stride();
            for (k, chunk) in values.chunks_exact(width).enumerate() {
                let start = (first_record + k) * stride + off;
                cells[start..start + width].copy_from_slice(chunk);
            }
        }
        self.storage
            .mark_written(field_index, first_record, first_record + n_records - 1);
        true
    }
    // ---------------------------------------------------------- defaults

    /// The phantom record's lanes for `field` — what a style read would see
    /// before any points exist. `None` when the schema lacks the field.
    #[must_use]
    pub fn default_record(&self, field: &str) -> Option<Vec<f32>> {
        let off = self.schema.offset(field)?;
        let width = self.schema.field_width(field)?;
        let row = self.defaults.as_ref()?;
        Some(row[off..off + width].to_vec())
    }

    /// Write `value` into the phantom record's lanes for `field`. The
    /// Reference's styling-of-a-pointless-mobject path (mobject.py:1329):
    /// the write is remembered until points arrive.
    pub fn write_default_record(&mut self, field: &str, value: &[f32]) {
        let (Some(off), Some(width)) = (self.schema.offset(field), self.schema.field_width(field))
        else {
            return;
        };
        if value.len() != width {
            return;
        }
        let stride = self.schema.stride();
        let row = self
            .defaults
            .get_or_insert_with(|| vec![0.0; stride].into_boxed_slice());
        row[off..off + width].copy_from_slice(value);
    }

    /// The Reference's style write over one 4-lane colour column (`set_fill`,
    /// `set_stroke`, `set_color`): replace the rgb lanes and/or the alpha lane
    /// of every record, leaving the other untouched — colour and opacity are
    /// independent writes, which every fade depends on. A point-less buffer
    /// writes its phantom record instead, so growth from zero adopts the
    /// style (`_data_defaults`). A schema without `field` is a no-op.
    #[allow(clippy::cast_possible_truncation)]
    pub fn write_color_lanes(&mut self, field: &str, rgb: Option<[f64; 3]>, alpha: Option<f64>) {
        if self.schema.field_width(field) != Some(4) {
            return;
        }
        let update = |rgba: &mut [f32]| {
            if let Some([r, g, b]) = rgb {
                rgba[0] = r as f32;
                rgba[1] = g as f32;
                rgba[2] = b as f32;
            }
            if let Some(a) = alpha {
                rgba[3] = a as f32;
            }
        };
        if self.is_empty() {
            if let Some(mut base) = self.default_record(field) {
                update(&mut base);
                self.write_default_record(field, &base);
            }
            return;
        }
        if let Some(mut column) = self.read_column(field) {
            for rgba in column.as_chunks_mut::<4>().0 {
                update(rgba);
            }
            self.write_range(field, 0, &column);
        }
    }

    /// Write one scalar across a 1-lane column (`stroke_width`,
    /// `fill_border_width`), or into the phantom record of a point-less
    /// buffer. A schema without `field` is a no-op.
    #[allow(clippy::cast_possible_truncation)]
    pub fn write_scalar_lanes(&mut self, field: &str, value: f64) {
        if self.schema.field_width(field) != Some(1) {
            return;
        }
        if self.is_empty() {
            self.write_default_record(field, &[value as f32]);
            return;
        }
        let column = vec![value as f32; self.len];
        self.write_range(field, 0, &column);
    }

    /// Tile the phantom record across `cells` when growth starts from zero
    /// (the `_data_defaults` adoption rule). A no-op without defaults or on
    /// non-empty growth.
    fn adopt_defaults_if_growing_from_zero(&self, cells: &mut [f32]) {
        if self.len != 0 {
            return;
        }
        let Some(row) = &self.defaults else {
            return;
        };
        let stride = self.schema.stride();
        if row.len() != stride || !cells.len().is_multiple_of(stride) {
            return;
        }
        for record in cells.chunks_exact_mut(stride) {
            record.copy_from_slice(row);
        }
    }

    /// Whether a phantom record exists (`has_defaults` for the persist
    /// format's presence flag).
    #[must_use]
    pub fn has_defaults(&self) -> bool {
        self.defaults.is_some()
    }

    /// The phantom record's full lane run, stride-long when present.
    #[must_use]
    pub fn defaults_row(&self) -> Option<&[f32]> {
        self.defaults.as_deref()
    }

    /// Install a phantom record read back from a snapshot. The row must be
    /// exactly one stride; otherwise the call is a silent no-op, matching
    /// the other format-defensive accessors here.
    pub fn restore_defaults_row(&mut self, row: &[f32]) {
        if row.len() != self.schema.stride() {
            return;
        }
        self.defaults = Some(row.to_vec().into_boxed_slice());
    }

    /// Take (and clear) the dirty span of `field`: the inclusive record
    /// range written since the last take. Feeds §10.8's dirty bounds.
    pub fn take_dirty_span(&mut self, field: &str) -> Option<(usize, usize)> {
        let index = self.schema.index_of(field)?;
        let forced = self.writable_view_affects(field) && self.len > 0;
        let mut spans = self.storage.dirty_spans.lock().expect("span lock poisoned");
        let written = spans[index].take().map(|s| (s.min, s.max));
        if forced {
            Some((0, self.len - 1))
        } else {
            written
        }
    }

    // ------------------------------------------------------------ locking

    /// `lock_data`: mark fields as unchanged for the current animation so
    /// interpolation and mirror sync can skip them (copy-elision). Pure
    /// state — access is never gated. (The `has_updaters` guard lives at
    /// the mobject layer with fm-yra, as in the Reference.)
    pub fn lock_data<'k>(&mut self, keys: impl IntoIterator<Item = &'k str>) {
        self.locked = keys
            .into_iter()
            .filter_map(|k| self.schema.index_of(k))
            .collect();
    }

    /// `unlock_data`.
    pub fn unlock_data(&mut self) {
        self.locked.clear();
    }

    /// Whether `field` is currently locked.
    #[must_use]
    pub fn is_locked(&self, field: &str) -> bool {
        self.schema
            .index_of(field)
            .is_some_and(|i| self.locked.contains(&i))
    }

    /// Locked field names, in schema order.
    #[must_use]
    pub fn locked_keys(&self) -> Vec<&str> {
        self.schema
            .fields()
            .iter()
            .enumerate()
            .filter(|(i, _)| self.locked.contains(i))
            .map(|(_, f)| f.name.as_str())
            .collect()
    }

    // ------------------------------------------------------------- resize

    /// Copy-on-resize (V1/V3): fresh generation, prefix copied, growth
    /// **null-padded** (the family-alignment padding primitive);
    /// outstanding views keep the old generation.
    ///
    /// # Errors
    /// [`RecordError::SizeOverflow`] when the target size cannot be
    /// addressed. The failure is **atomic** (fm-vek.2): the sizing proof
    /// runs before anything is allocated or swapped, so a refused resize
    /// leaves the buffer, its revisions, and every live view untouched.
    pub fn resize(&mut self, new_len: usize) -> Result<(), RecordError> {
        let stride = self.schema.stride();
        let lanes = checked_lanes(new_len, stride)?;
        let mut cells = vec![0.0f32; lanes];
        self.adopt_defaults_if_growing_from_zero(&mut cells);
        {
            let old = self.storage.cells.read().expect("storage lock poisoned");
            let keep = old.len().min(cells.len());
            cells[..keep].copy_from_slice(&old[..keep]);
        }
        self.swap_in(cells);
        self.len = new_len;
        Ok(())
    }

    /// `resize_with_interpolation`, ported from
    /// `manimlib/utils/iterables.py`: same length → no-op; a single record
    /// or an all-equal buffer repeats; zero target empties; otherwise each
    /// new record linearly interpolates its two bracketing old records
    /// (`cont_indices = linspace(0, len-1, new_len)`). Applies to every
    /// lane of every field, which equals the Reference's per-field
    /// application over the whole dtype.
    ///
    /// # Errors
    /// [`RecordError::SizeOverflow`] when the target size cannot be
    /// addressed; atomic like [`RecordBuffer::resize`] — a refusal mutates
    /// no observable state and invalidates no view.
    pub fn resize_with_interpolation(&mut self, new_len: usize) -> Result<(), RecordError> {
        if new_len == self.len {
            return Ok(());
        }
        let stride = self.schema.stride();
        let lanes = checked_lanes(new_len, stride)?;
        let old = self.storage.copy_cells();
        let is_constant = self.len > 0
            && old
                .chunks_exact(stride)
                .all(|record| record == &old[..stride]);
        let mut cells = vec![0.0f32; lanes];
        self.adopt_defaults_if_growing_from_zero(&mut cells);
        if self.len == 1 || is_constant {
            for record in cells.chunks_exact_mut(stride) {
                record.copy_from_slice(&old[..stride]);
            }
        } else if new_len > 0 && self.len > 0 {
            let n = self.len;
            let step = (n - 1) as f64
                / if new_len > 1 {
                    (new_len - 1) as f64
                } else {
                    1.0
                };
            for (i, record) in cells.chunks_exact_mut(stride).enumerate() {
                let ci = i as f64 * step;
                let lh = ci as usize;
                let rh = (ci.ceil() as usize).min(n - 1);
                let a = (ci.rem_euclid(1.0)) as f32;
                let left = &old[lh * stride..(lh + 1) * stride];
                let right = &old[rh * stride..(rh + 1) * stride];
                for (lane, slot) in record.iter_mut().enumerate() {
                    *slot = (1.0 - a) * left[lane] + a * right[lane];
                }
            }
        }
        self.swap_in(cells);
        self.len = new_len;
        Ok(())
    }

    /// `resize_preserving_order` (`utils/iterables.py`): record `i` of the
    /// result is old record `i * old_len / new_len` — proportional index
    /// repetition, order-preserving, no interpolation. The family-alignment
    /// resize primitive (§9.4). An empty buffer zero-fills (NumPy
    /// `np.resize` of an empty array). Same copy-on-resize generation rules
    /// as [`RecordBuffer::resize`] (V1/V3).
    ///
    /// # Errors
    /// [`RecordError::SizeOverflow`] when the target size cannot be
    /// addressed; atomic like [`RecordBuffer::resize`] — a refusal mutates
    /// no observable state and invalidates no view.
    pub fn resize_preserving_order(&mut self, new_len: usize) -> Result<(), RecordError> {
        if new_len == self.len {
            return Ok(());
        }
        let stride = self.schema.stride();
        let lanes = checked_lanes(new_len, stride)?;
        let mut cells = vec![0.0f32; lanes];
        self.adopt_defaults_if_growing_from_zero(&mut cells);
        if self.len > 0 {
            let old = self.storage.copy_cells();
            for (i, record) in cells.chunks_exact_mut(stride).enumerate() {
                let src = i * self.len / new_len;
                record.copy_from_slice(&old[src * stride..(src + 1) * stride]);
            }
        }
        self.swap_in(cells);
        self.len = new_len;
        Ok(())
    }

    fn swap_in(&mut self, cells: Vec<f32>) {
        let n_fields = self.schema.fields().len();
        let fresh = Storage::new(cells.into_boxed_slice(), n_fields);
        // A new generation invalidates everything: bump every revision past
        // the old ones so mirrors resync.
        fresh.revision.store(
            self.storage.revision.load(Ordering::Acquire) + 1,
            Ordering::Release,
        );
        for (new, old) in fresh
            .field_revisions
            .iter()
            .zip(self.storage.field_revisions.iter())
        {
            new.store(old.load(Ordering::Acquire) + 1, Ordering::Release);
        }
        self.storage = fresh;
    }

    /// This buffer's record layout as a NumPy structured dtype, in fnp's
    /// structured-record vocabulary (fm-n1b). Available without exporting a
    /// view: the dtype is a property of the schema, not of any generation.
    #[must_use]
    pub fn numpy_dtype(&self) -> RecordDType {
        RecordDType::of(&self.schema)
    }

    // -------------------------------------------------------------- views

    /// Export a live whole-buffer view — where fmn-python's zero-copy NumPy
    /// structured export attaches (V2). Unshares from snapshots first (V5).
    pub fn export_view(&mut self, writable: bool) -> RecordView {
        self.export_view_inner(writable, None)
    }

    /// Export a live view scoped to one field (the NumPy single-field view:
    /// `mobject.data["points"]`-class). Conservative invalidation then
    /// applies to that field only.
    pub fn export_field_view(&mut self, field: &str, writable: bool) -> Option<RecordView> {
        let index = self.schema.index_of(field)?;
        Some(self.export_view_inner(writable, Some(index)))
    }

    fn export_view_inner(&mut self, writable: bool, field: Option<usize>) -> RecordView {
        self.unshare();
        self.storage.views.fetch_add(1, Ordering::AcqRel);
        if writable {
            match field {
                Some(index) => {
                    self.storage.writable_field_views[index].fetch_add(1, Ordering::AcqRel);
                }
                None => {
                    self.storage
                        .writable_whole_views
                        .fetch_add(1, Ordering::AcqRel);
                }
            }
        }
        RecordView {
            schema: Arc::clone(&self.schema),
            storage: Arc::clone(&self.storage),
            len: self.len,
            writable,
            field,
        }
    }

    // ----------------------------------------------------------- clones

    /// Snapshot clone (V5): share the generation unless live views force an
    /// eager copy.
    #[must_use]
    pub fn snapshot_clone(&self) -> Self {
        if self.live_view_count() > 0 {
            Self {
                schema: Arc::clone(&self.schema),
                storage: self.storage.clone_for_snapshot(),
                len: self.len,
                defaults: self.defaults.clone(),
                locked: self.locked.clone(),
            }
        } else {
            Self {
                schema: Arc::clone(&self.schema),
                storage: Arc::clone(&self.storage),
                len: self.len,
                defaults: self.defaults.clone(),
                locked: self.locked.clone(),
            }
        }
    }

    /// Independent deep copy (mobject `copy()` semantics, §8.3).
    #[must_use]
    pub fn deep_clone(&self) -> Self {
        Self {
            schema: Arc::clone(&self.schema),
            storage: self.storage.clone_with_revisions(),
            len: self.len,
            defaults: self.defaults.clone(),
            locked: self.locked.clone(),
        }
    }

    /// Reference `set_data`, the `become` data plane (§8.3): replace this
    /// buffer's records with an independent copy of `other`'s. Schemas must
    /// match — the Reference asserts dtype equality — else this returns
    /// `false` and leaves the buffer untouched. Flows through the
    /// copy-on-resize path: a fresh generation with every revision bumped
    /// past the old ones, so mirrors and bounding-box signatures resync and
    /// outstanding views detach exactly as under resize (V6).
    pub fn assign_from(&mut self, other: &RecordBuffer) -> bool {
        if !(Arc::ptr_eq(&self.schema, &other.schema) || *self.schema == *other.schema) {
            return false;
        }
        let cells = other.storage.copy_cells().into_vec();
        self.swap_in(cells);
        self.len = other.len;
        true
    }
}

/// A live view over one storage generation — the engine-side model of the
/// exported NumPy (structured or single-field) array. Dropping it unpins
/// the generation.
#[derive(Debug)]
pub struct RecordView {
    schema: Arc<RecordSchema>,
    storage: Arc<Storage>,
    len: usize,
    writable: bool,
    /// `Some(i)` for a field-scoped view.
    field: Option<usize>,
}

impl RecordView {
    /// Number of records visible to this view (fixed at export).
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Record layout pinned by this view's storage generation.
    ///
    /// This is the schema source for fmn-python's NumPy dtype descriptor.
    /// It remains valid after the originating buffer resizes because the
    /// view owns the schema independently of the buffer's current generation.
    #[doc(hidden)]
    #[must_use]
    pub fn schema(&self) -> &RecordSchema {
        &self.schema
    }

    /// Whether the foreign view was exported writable.
    #[doc(hidden)]
    #[must_use]
    pub fn is_writable(&self) -> bool {
        self.writable
    }

    /// Pointer to the first lane in this pinned storage generation.
    ///
    /// The pointer is stable for the lifetime of `self`: every resize swaps
    /// in a new `Storage` rather than reallocating this generation, and the
    /// view's `Arc` keeps the old allocation alive. Dereferencing it is
    /// deliberately confined to fmn-python's CPython buffer slot, whose
    /// single-scene-worker thread rule prevents concurrent Rust access.
    #[doc(hidden)]
    #[must_use]
    pub fn foreign_data_ptr(&self) -> *mut f32 {
        self.storage
            .cells
            .write()
            .expect("storage lock poisoned")
            .as_mut_ptr()
    }

    /// Byte length of the complete interleaved record generation.
    #[doc(hidden)]
    #[must_use]
    pub fn foreign_byte_len(&self) -> usize {
        self.len
            .checked_mul(self.schema.stride())
            .and_then(|lanes| lanes.checked_mul(std::mem::size_of::<f32>()))
            .expect("RecordBuffer allocation already proved its byte length")
    }

    /// Whether this view still aliases `buffer`'s current generation
    /// (false once a resize or restore has swapped generations, V3/V6).
    #[must_use]
    pub fn is_attached_to(&self, buffer: &RecordBuffer) -> bool {
        Arc::ptr_eq(&self.storage, &buffer.storage)
    }

    fn allows(&self, field_index: usize) -> bool {
        self.field.is_none_or(|scoped| scoped == field_index)
    }

    #[must_use]
    pub fn read(&self, index: usize, field: &str) -> Option<Vec<f32>> {
        let field_index = self.schema.index_of(field)?;
        if !self.allows(field_index) || index >= self.len {
            return None;
        }
        let off = self.schema.offset(field)?;
        let width = self.schema.field_width(field)?;
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        let start = index * self.schema.stride() + off;
        Some(cells[start..start + width].to_vec())
    }

    fn write_cells(&self, index: usize, field: &str, values: &[f32]) -> Option<usize> {
        if !self.writable {
            return None;
        }
        let field_index = self.schema.index_of(field)?;
        if !self.allows(field_index) {
            return None;
        }
        let off = self.schema.offset(field).expect("field exists");
        let width = self.schema.field_width(field).expect("field exists");
        if index >= self.len || values.len() != width {
            return None;
        }
        let mut cells = self.storage.cells.write().expect("storage lock poisoned");
        let start = index * self.schema.stride() + off;
        cells[start..start + width].copy_from_slice(values);
        Some(field_index)
    }

    /// Write through the view: visible to the engine while attached, and it
    /// marks render state dirty via the revisions (V4). Read-only views and
    /// out-of-scope fields return `false`.
    pub fn write(&self, index: usize, field: &str, values: &[f32]) -> bool {
        let Some(field_index) = self.write_cells(index, field, values) else {
            return false;
        };
        self.storage.mark_written(field_index, index, index);
        true
    }

    // --------------------------------------------- fnp NumPy export (fm-n1b)

    /// This view's record layout as a NumPy structured dtype.
    ///
    /// Pinned with the generation: it stays valid — and stays the dtype the
    /// exported array was built against — after the originating buffer
    /// resizes away underneath (V3).
    #[must_use]
    pub fn numpy_dtype(&self) -> RecordDType {
        RecordDType::of(&self.schema)
    }

    /// The dtype and fnp layout this view exports, before borrowing the cells.
    fn numpy_desc(&self, writable: bool) -> Result<ArrayDesc, ExportError> {
        let dtype = RecordDType::of(&self.schema);
        let record_bytes = dtype.itemsize();
        let (layout, base_offset, scope) = match self.field {
            None => (whole_layout(self.len, record_bytes, writable)?, 0, None),
            Some(index) => {
                let spec = &self.schema.fields()[index];
                let offset = dtype
                    .byte_offset(&spec.name)
                    .expect("the dtype describes every schema field");
                let layout = field_layout(self.len, record_bytes, offset, spec.width, writable)?;
                (layout, offset, Some(spec.name.clone()))
            }
        };
        Ok(ArrayDesc {
            dtype,
            layout,
            base_offset,
            scope,
            len: self.len,
        })
    }

    /// The fnp layout of this view's export: `(len,)` of whole records for a
    /// whole-buffer view, `(len, width)` of lanes for a field-scoped one.
    /// The layout's `writeable` flag mirrors the view's own.
    ///
    /// # Errors
    /// [`ExportError::Shape`] when fnp refuses the shape/stride combination —
    /// a field-less schema's zero itemsize, or an unrepresentable span.
    pub fn numpy_layout(&self) -> Result<NdLayout, ExportError> {
        Ok(self.numpy_desc(self.writable)?.layout)
    }

    /// Alias this view's pinned generation as a read-only NumPy-shaped array.
    ///
    /// Zero-copy: the returned array borrows the generation's own cells, and
    /// its `data_ptr()` equals [`RecordView::foreign_data_ptr`]. It holds the
    /// generation's read lock, so engine writes block until it is dropped.
    ///
    /// # Errors
    /// [`ExportError::Shape`] as for [`RecordView::numpy_layout`].
    pub fn as_numpy(&self) -> Result<RecordArray<'_>, ExportError> {
        let desc = self.numpy_desc(false)?;
        let cells = self.storage.cells.read().expect("storage lock poisoned");
        Ok(RecordArray::new(cells, desc))
    }

    /// Alias this view's pinned generation as a writable NumPy-shaped array.
    ///
    /// Writes through it are foreign writes (V4): they mutate the engine's
    /// cells in place without advancing a revision, exactly as a NumPy
    /// assignment would. Observers refresh from
    /// [`RecordBuffer::writable_view_affects`] while the view lives, and the
    /// view's `Drop` bumps every field it exposed.
    ///
    /// # Errors
    /// [`ExportError::ReadOnly`] when the view was exported read-only;
    /// [`ExportError::Shape`] as for [`RecordView::numpy_layout`].
    pub fn as_numpy_mut(&self) -> Result<RecordArrayMut<'_>, ExportError> {
        if !self.writable {
            return Err(ExportError::ReadOnly);
        }
        let desc = self.numpy_desc(true)?;
        let cells = self.storage.cells.write().expect("storage lock poisoned");
        Ok(RecordArrayMut::new(cells, desc))
    }

    /// Model a write performed directly through foreign zero-copy memory.
    ///
    /// A NumPy assignment cannot call back into [`RecordView::write`], so it
    /// changes the aliased cells without advancing a revision. This bridge and
    /// conformance hook has those exact semantics: consumers must refresh from
    /// the writable-view lifetime flag, and [`Drop`] advances every exposed
    /// field once the foreign writer detaches. Engine-mediated callers should
    /// use [`RecordView::write`] instead.
    #[doc(hidden)]
    pub fn write_foreign(&self, index: usize, field: &str, values: &[f32]) -> bool {
        self.write_cells(index, field, values).is_some()
    }
}

impl Drop for RecordView {
    fn drop(&mut self) {
        if self.writable {
            // A real zero-copy consumer (NumPy is the compatibility contract)
            // writes the exposed cells directly and cannot call
            // `RecordView::write`, so revision counters cannot be the only
            // invalidation channel. While the view is live, observers refresh
            // through `writable_view_affects`; on detach, conservatively bump
            // every exposed field so a write after the final observation cannot
            // disappear when the lifetime flag clears.
            if self.len > 0 {
                match self.field {
                    Some(index) => self.storage.mark_written(index, 0, self.len - 1),
                    None => {
                        for index in 0..self.schema.fields().len() {
                            self.storage.mark_written(index, 0, self.len - 1);
                        }
                    }
                }
            }
            match self.field {
                Some(index) => {
                    self.storage.writable_field_views[index].fetch_sub(1, Ordering::AcqRel);
                }
                None => {
                    self.storage
                        .writable_whole_views
                        .fetch_sub(1, Ordering::AcqRel);
                }
            }
        }
        self.storage.views.fetch_sub(1, Ordering::AcqRel);
    }
}

// ---------------------------------------------------------------- mirrors

/// One field's lane-major (SoA) mirror: `lanes[lane * len + record]`.
#[derive(Debug)]
struct FieldMirror {
    lanes: Vec<f32>,
    len: usize,
    seen_revision: u64,
    seen_storage: usize,
}

/// The lazy revisioned render mirrors (§8.2 Rev 4, D-20): what the hot
/// loops read instead of the interleaved buffer. Owned by the render side
/// (one set per renderable object), synchronized at most once per
/// observation, and only when the field actually changed.
#[derive(Debug, Default)]
pub struct MirrorSet {
    fields: std::collections::HashMap<String, FieldMirror>,
    materializations: u64,
}

impl MirrorSet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many times any field actually rematerialized — the laziness
    /// tests' observable.
    #[must_use]
    pub fn materializations(&self) -> u64 {
        self.materializations
    }

    /// Synchronize the mirror of `field` and return its lane-major data
    /// (`lanes[lane * len + record]`). Rematerializes only when the field
    /// revision moved, the storage generation swapped, or a writable view
    /// forces conservative refresh (a live view never silently receives
    /// weaker semantics). Locked fields (copy-elision) refresh like any
    /// other when their revision moves — locking is an animation-engine
    /// contract, not a mirror bypass.
    pub fn sync<'a>(&'a mut self, buffer: &RecordBuffer, field: &str) -> Option<&'a [f32]> {
        let revision = buffer.field_revision(field)?;
        let width = buffer.schema().field_width(field)?;
        let storage = buffer.storage_id();
        let forced = buffer.writable_view_affects(field);
        let stale = match self.fields.get(field) {
            Some(mirror) => {
                mirror.seen_revision != revision
                    || mirror.seen_storage != storage
                    || mirror.len != buffer.len()
                    || forced
            }
            None => true,
        };
        if stale {
            let column = buffer.read_column(field)?;
            let len = buffer.len();
            // Transpose record-major column into lane-major SoA.
            let mut lanes = vec![0.0f32; column.len()];
            for record in 0..len {
                for lane in 0..width {
                    lanes[lane * len + record] = column[record * width + lane];
                }
            }
            self.fields.insert(
                field.to_string(),
                FieldMirror {
                    lanes,
                    len,
                    seen_revision: revision,
                    seen_storage: storage,
                },
            );
            self.materializations += 1;
        }
        self.fields.get(field).map(|m| m.lanes.as_slice())
    }
}
