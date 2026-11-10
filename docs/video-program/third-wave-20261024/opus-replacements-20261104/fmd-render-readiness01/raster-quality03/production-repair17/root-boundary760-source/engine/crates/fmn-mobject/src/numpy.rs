//! The fnp-backed NumPy export surface (§8.2, fm-n1b): franken_numpy's
//! structured-record and stride vocabularies applied to Marionette's
//! authoritative interleaved buffer, with **no copy anywhere on the path**.
//!
//! The split is deliberate, and it is what makes the export honest:
//!
//! - **The dtype comes from fnp-dtype.** [`RecordDType`] re-expresses a
//!   [`RecordSchema`] as one [`StructuredField`] per record field, carrying
//!   the *byte* offset NumPy indexes by and the subarray lane count the
//!   Reference dtypes declare. It is a descriptor, not storage — fnp's own
//!   `StructuredStorage` is columnar (one owned [`fnp_dtype::ArrayStorage`]
//!   per field), so building one would be precisely the copy-out this bead
//!   refuses.
//! - **The layout comes from fnp-ndarray.** [`fnp_ndarray::NdLayout`] is a
//!   pure shape/stride/writeability object with no data ownership at all, so
//!   it describes a view over foreign memory natively. Field-scoped exports
//!   are built through [`NdLayout::as_strided`] against the span that
//!   actually remains after the field's byte offset, so fnp — not this
//!   crate — adjudicates view legality.
//! - **The bytes stay in Marionette.** [`RecordArray`] and
//!   [`RecordArrayMut`] borrow the pinned storage generation's own `f32`
//!   cells through the [`crate::record::RecordView`] that exported them.
//!   `data_ptr()` equals the generation's base pointer; every accessor hands
//!   back a subslice of that one allocation.
//!
//! Lifetime and dirtiness follow §8.2 unchanged. The array borrows its
//! `RecordView`, which pins the generation (V2), so reallocation underneath
//! it stays impossible by construction (V1) and a resize detaches it with
//! NumPy-natural semantics (V3/V6). Writes through [`RecordArrayMut`] are
//! *foreign* writes with exactly [`RecordView::write_foreign`]'s semantics
//! (V4): they change the aliased cells without advancing a revision, because
//! a real NumPy assignment cannot call back into the engine either.
//! Observers refresh from the writable-view lifetime flag while the view is
//! live, and the view's `Drop` conservatively bumps every field it exposed.
//!
//! [`RecordSchema`]: crate::record::RecordSchema
//! [`RecordView::write_foreign`]: crate::record::RecordView::write_foreign

use std::fmt;
use std::ops::Range;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use fnp_dtype::{DType, StructuredField};
use fnp_ndarray::{MemoryOrder, NdLayout, ShapeError};

use crate::record::RecordSchema;

/// Every lane of a Marionette record is a NumPy `float32` — the interleaved
/// buffer is all-f32 by construction (§8.2), which is what lets one lane
/// dtype describe the whole record.
pub const LANE_DTYPE: DType = DType::F32;

/// Bytes per lane, taken from fnp's dtype model rather than assumed here.
pub const LANE_BYTES: usize = LANE_DTYPE.item_size();

/// The NumPy byte-order character for natively-laid-out records. The export
/// aliases host memory, so the descriptor must claim host endianness.
const fn native_byte_order() -> char {
    if cfg!(target_endian = "little") {
        '<'
    } else {
        '>'
    }
}

/// Why a NumPy export could not be produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// fnp refused the shape/stride combination (an unrepresentable span, a
    /// zero itemsize from a field-less schema, or an arithmetic overflow).
    Shape(ShapeError),
    /// A mutable export was requested from a view exported read-only. The
    /// view protocol's writability is decided at export (V2) and never
    /// widened afterwards.
    ReadOnly,
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(err) => write!(f, "fnp rejected the record view layout: {err:?}"),
            Self::ReadOnly => write!(f, "the record view was exported read-only"),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<ShapeError> for ExportError {
    fn from(err: ShapeError) -> Self {
        Self::Shape(err)
    }
}

// ------------------------------------------------------------------ dtype

/// A [`RecordSchema`] as a NumPy structured dtype, in fnp's vocabulary.
///
/// Field byte offsets are the schema's lane offsets scaled by
/// [`LANE_BYTES`]; `tests/data_plane.rs` locks those against numpy's own
/// structured-dtype packing for the Reference dtypes, so this descriptor and
/// the interleaved bytes agree by test, not by assertion.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordDType {
    fields: Vec<StructuredField>,
    /// Subarray lane count per field, parallel with `fields` — the `(3,)` of
    /// `('point', '<f4', (3,))`, which [`StructuredField`] has no room for.
    shapes: Vec<usize>,
    itemsize: usize,
}

impl RecordDType {
    /// Describe `schema` as a structured dtype.
    #[must_use]
    pub fn of(schema: &RecordSchema) -> Self {
        let mut fields = Vec::with_capacity(schema.fields().len());
        let mut shapes = Vec::with_capacity(schema.fields().len());
        let mut offset = 0usize;
        for spec in schema.fields() {
            fields.push(StructuredField {
                name: spec.name.clone(),
                dtype: LANE_DTYPE,
                offset,
            });
            shapes.push(spec.width);
            offset += spec.width * LANE_BYTES;
        }
        Self {
            fields,
            shapes,
            itemsize: schema.stride() * LANE_BYTES,
        }
    }

    /// The fnp field descriptors, in record order.
    #[must_use]
    pub fn fields(&self) -> &[StructuredField] {
        &self.fields
    }

    /// `dtype.itemsize` — the interleaved record stride in bytes.
    #[must_use]
    pub fn itemsize(&self) -> usize {
        self.itemsize
    }

    /// Number of named fields.
    #[must_use]
    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    /// Whether the dtype declares no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    /// The descriptor for `name`.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&StructuredField> {
        self.index_of(name).map(|i| &self.fields[i])
    }

    /// `dtype[name].offset` in bytes.
    #[must_use]
    pub fn byte_offset(&self, name: &str) -> Option<usize> {
        self.field(name).map(|f| f.offset)
    }

    /// The subarray lane count of `name` — the `(n,)` of its dtype tuple.
    #[must_use]
    pub fn subshape(&self, name: &str) -> Option<usize> {
        self.index_of(name).map(|i| self.shapes[i])
    }

    /// The dtype in NumPy's `descr` list form, e.g.
    /// `[('point', '<f4', (3,)), ('rgba', '<f4', (4,))]` — the Reference's
    /// `Mobject.data_dtype` spelled exactly as numpy repr's it.
    #[must_use]
    pub fn descr(&self) -> String {
        let order = native_byte_order();
        let size = LANE_DTYPE.item_size();
        let entries: Vec<String> = self
            .fields
            .iter()
            .zip(&self.shapes)
            .map(|(field, width)| format!("('{}', '{order}f{size}', ({width},))", field.name))
            .collect();
        format!("[{}]", entries.join(", "))
    }
}

// ------------------------------------------------------------------ layout

/// Build the 1-D structured layout of `len` whole records: one element per
/// record, `itemsize` the record stride in bytes — NumPy's `arr.shape ==
/// (len,)`, `arr.dtype.itemsize == stride * 4`.
pub(crate) fn whole_layout(
    len: usize,
    record_bytes: usize,
    writable: bool,
) -> Result<NdLayout, ShapeError> {
    let layout = NdLayout::contiguous(vec![len], record_bytes, MemoryOrder::C)?;
    Ok(if writable {
        layout
    } else {
        layout.as_read_only()
    })
}

/// Build the 2-D lane layout of one field: `arr[name]` with shape
/// `(len, width)` and strides `(record_bytes, 4)`.
///
/// The base layout deliberately spans only the lanes that remain *after* the
/// field's byte offset, so [`NdLayout::as_strided`]'s bounds check is a real
/// adjudication of this view's legality rather than a formality.
pub(crate) fn field_layout(
    len: usize,
    record_bytes: usize,
    byte_offset: usize,
    width: usize,
    writable: bool,
) -> Result<NdLayout, ShapeError> {
    let total_bytes = len.checked_mul(record_bytes).ok_or(ShapeError::Overflow)?;
    let reachable_lanes = total_bytes.saturating_sub(byte_offset) / LANE_BYTES;
    let base = NdLayout::contiguous(vec![reachable_lanes], LANE_BYTES, MemoryOrder::C)?;
    let record_stride = isize::try_from(record_bytes).map_err(|_| ShapeError::Overflow)?;
    let lane_stride = isize::try_from(LANE_BYTES).map_err(|_| ShapeError::Overflow)?;
    let layout = base.as_strided(vec![len, width], vec![record_stride, lane_stride])?;
    Ok(if writable {
        layout
    } else {
        layout.as_read_only()
    })
}

// ------------------------------------------------------------------ arrays

/// Shared descriptor half of an exported array — everything except the
/// borrow of the cells.
#[derive(Debug, Clone)]
pub(crate) struct ArrayDesc {
    pub(crate) dtype: RecordDType,
    pub(crate) layout: NdLayout,
    /// Base byte offset of this export within the generation: `0` for a
    /// whole-buffer export, the field's byte offset for a field-scoped one.
    pub(crate) base_offset: usize,
    /// `Some(name)` for a field-scoped export; other fields are out of scope.
    pub(crate) scope: Option<String>,
    pub(crate) len: usize,
}

impl ArrayDesc {
    /// The record stride in bytes, read back out of the fnp layout so index
    /// arithmetic below travels through the descriptor rather than around it.
    fn record_bytes(&self) -> Option<usize> {
        let stride = *self.layout.strides.first()?;
        usize::try_from(stride).ok()
    }

    /// Lane range of `(index, name)` within the whole generation, or `None`
    /// when the index is past the end, the field is unknown, or the field is
    /// outside a field-scoped export's scope.
    fn lane_span(&self, index: usize, name: &str) -> Option<Range<usize>> {
        if index >= self.len {
            return None;
        }
        if self.scope.as_deref().is_some_and(|scoped| scoped != name) {
            return None;
        }
        let byte_offset = self.dtype.byte_offset(name)?;
        let width = self.dtype.subshape(name)?;
        let start = index
            .checked_mul(self.record_bytes()?)?
            .checked_add(byte_offset)?
            / LANE_BYTES;
        Some(start..start.checked_add(width)?)
    }
}

/// A read-only NumPy-shaped view aliasing a pinned storage generation.
///
/// Holds the generation's read lock for its lifetime: this *is* the
/// exported array's memory, not a snapshot of it.
#[derive(Debug)]
pub struct RecordArray<'v> {
    cells: RwLockReadGuard<'v, Box<[f32]>>,
    desc: ArrayDesc,
}

impl<'v> RecordArray<'v> {
    pub(crate) fn new(cells: RwLockReadGuard<'v, Box<[f32]>>, desc: ArrayDesc) -> Self {
        Self { cells, desc }
    }

    /// The structured dtype of the whole record, regardless of scope.
    #[must_use]
    pub fn dtype(&self) -> &RecordDType {
        &self.desc.dtype
    }

    /// The fnp layout of this export.
    #[must_use]
    pub fn layout(&self) -> &NdLayout {
        &self.desc.layout
    }

    /// Base byte offset of this export within the generation.
    #[must_use]
    pub fn byte_offset(&self) -> usize {
        self.desc.base_offset
    }

    /// `Some(name)` when this is a single-field export.
    #[must_use]
    pub fn scope(&self) -> Option<&str> {
        self.desc.scope.as_deref()
    }

    /// Records visible to this export.
    #[must_use]
    pub fn len(&self) -> usize {
        self.desc.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.desc.len == 0
    }

    /// The pinned generation's own lanes — the aliased allocation itself.
    #[must_use]
    pub fn lanes(&self) -> &[f32] {
        &self.cells[..]
    }

    /// Base pointer of the aliased allocation; equal to
    /// [`crate::record::RecordView::foreign_data_ptr`] for the same view.
    #[must_use]
    pub fn data_ptr(&self) -> *const f32 {
        self.cells.as_ptr()
    }

    /// Borrow one field of one record, addressed purely through the fnp
    /// descriptors. No allocation, no copy — a subslice of [`Self::lanes`].
    #[must_use]
    pub fn field(&self, index: usize, name: &str) -> Option<&[f32]> {
        let span = self.desc.lane_span(index, name)?;
        self.cells.get(span)
    }
}

/// A writable NumPy-shaped view aliasing a pinned storage generation.
///
/// Writes land in the engine's own cells (V4 foreign-write semantics, see
/// the module docs). Holds the generation's write lock for its lifetime, so
/// scope it around the mutation rather than holding it across engine calls.
#[derive(Debug)]
pub struct RecordArrayMut<'v> {
    cells: RwLockWriteGuard<'v, Box<[f32]>>,
    desc: ArrayDesc,
}

impl<'v> RecordArrayMut<'v> {
    pub(crate) fn new(cells: RwLockWriteGuard<'v, Box<[f32]>>, desc: ArrayDesc) -> Self {
        Self { cells, desc }
    }

    /// The structured dtype of the whole record, regardless of scope.
    #[must_use]
    pub fn dtype(&self) -> &RecordDType {
        &self.desc.dtype
    }

    /// The fnp layout of this export. Its `writeable` flag is `true`.
    #[must_use]
    pub fn layout(&self) -> &NdLayout {
        &self.desc.layout
    }

    /// Base byte offset of this export within the generation.
    #[must_use]
    pub fn byte_offset(&self) -> usize {
        self.desc.base_offset
    }

    /// `Some(name)` when this is a single-field export.
    #[must_use]
    pub fn scope(&self) -> Option<&str> {
        self.desc.scope.as_deref()
    }

    /// Records visible to this export.
    #[must_use]
    pub fn len(&self) -> usize {
        self.desc.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.desc.len == 0
    }

    /// The pinned generation's own lanes.
    #[must_use]
    pub fn lanes(&self) -> &[f32] {
        &self.cells[..]
    }

    /// The pinned generation's own lanes, mutably.
    pub fn lanes_mut(&mut self) -> &mut [f32] {
        &mut self.cells[..]
    }

    /// Base pointer of the aliased allocation.
    #[must_use]
    pub fn data_ptr(&self) -> *const f32 {
        self.cells.as_ptr()
    }

    /// Borrow one field of one record, addressed through the fnp descriptors.
    #[must_use]
    pub fn field(&self, index: usize, name: &str) -> Option<&[f32]> {
        let span = self.desc.lane_span(index, name)?;
        self.cells.get(span)
    }

    /// Borrow one field of one record mutably — the foreign write path.
    /// Mutating the returned slice mutates the engine's buffer in place.
    pub fn field_mut(&mut self, index: usize, name: &str) -> Option<&mut [f32]> {
        let span = self.desc.lane_span(index, name)?;
        self.cells.get_mut(span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::RecordBuffer;

    /// The bead's proof (fm-n1b): a Reference-shaped record buffer makes the
    /// whole round trip — Rust write → fnp-described structured view →
    /// foreign write through the aliased lanes → Rust read — over **one**
    /// allocation, with no defensive clone anywhere on the happy path.
    #[test]
    fn fnp_structured_view_round_trips_without_a_defensive_clone() {
        let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 4).expect("4 mobject records");

        // The dtype is the Reference's `Mobject.data_dtype`, from fnp's
        // structured-record vocabulary.
        let dtype = buffer.numpy_dtype();
        assert_eq!(
            dtype.descr(),
            "[('point', '<f4', (3,)), ('rgba', '<f4', (4,))]"
        );
        assert_eq!(dtype.itemsize(), 28);
        assert_eq!(dtype.byte_offset("point"), Some(0));
        assert_eq!(dtype.byte_offset("rgba"), Some(12));
        assert_eq!(dtype.field("rgba").map(|f| f.dtype), Some(DType::F32));

        // Leg 1 — the engine writes.
        assert!(buffer.write(2, "point", &[1.0, 2.0, 3.0]));
        let generation = buffer.storage_id();
        let revision_before = buffer.field_revision("point").expect("point exists");

        let view = buffer.export_view(true);
        let base = view.foreign_data_ptr();

        // Exporting did not copy: same generation, and it is the buffer's.
        assert_eq!(buffer.storage_id(), generation);
        assert!(view.is_attached_to(&buffer));

        {
            let mut array = view.as_numpy_mut().expect("the view is writable");

            // The layout is NumPy's: one element per record, itemsize 28.
            assert_eq!(array.layout().shape, vec![4]);
            assert_eq!(array.layout().strides, vec![28]);
            assert_eq!(array.layout().item_size, 28);
            assert!(array.layout().is_writeable());
            assert!(array.layout().is_contiguous());

            // Zero-copy, proved by identity rather than by assertion: the
            // array's data pointer *is* the generation's base pointer, and
            // its lane count is the whole allocation.
            assert_eq!(array.data_ptr(), base.cast_const());
            assert_eq!(array.lanes().len(), 4 * 7);

            // Leg 2 — the engine's write is visible through the fnp view,
            // addressed purely through the fnp descriptors.
            assert_eq!(array.field(2, "point"), Some(&[1.0, 2.0, 3.0][..]));

            // Leg 3 — a foreign write, as NumPy performs it: mutate the
            // aliased lanes in place. No engine callback, no copy back.
            array
                .field_mut(2, "rgba")
                .expect("rgba is in scope")
                .copy_from_slice(&[0.25, 0.5, 0.75, 1.0]);
            array.lanes_mut()[0] = -1.5; // record 0, point.x, straight through.

            assert_eq!(array.data_ptr(), base.cast_const());
        }

        // Leg 4 — back in the engine, over the same generation.
        drop(view);
        assert_eq!(
            buffer.storage_id(),
            generation,
            "no generation was swapped in"
        );
        assert_eq!(buffer.read(2, "rgba"), Some(vec![0.25, 0.5, 0.75, 1.0]));
        assert_eq!(buffer.read(0, "point"), Some(vec![-1.5, 0.0, 0.0]));

        // V4: the foreign write advanced no revision while it happened; the
        // writable view's detach conservatively bumped the exposed fields.
        assert!(
            buffer.field_revision("point").expect("point exists") > revision_before,
            "detaching a writable view conservatively revises what it exposed"
        );
    }

    #[test]
    fn field_scoped_export_is_a_strided_lane_view_of_the_same_allocation() {
        let mut buffer =
            RecordBuffer::new(RecordSchema::vmobject(), 3).expect("3 vmobject records");
        let generation = buffer.storage_id();
        let view = buffer
            .export_field_view("fill_rgba", true)
            .expect("fill_rgba exists");
        let base = view.foreign_data_ptr();

        let mut array = view.as_numpy_mut().expect("the view is writable");
        assert_eq!(array.scope(), Some("fill_rgba"));
        // numpy: vmobject itemsize 68, fill_rgba at byte 36, four lanes.
        // (32 is joint_angle; the whole table is pinned in
        // `vmobject_dtype_offsets_match_numpy_structured_packing` below.)
        assert_eq!(array.byte_offset(), 36);
        assert_eq!(array.layout().shape, vec![3, 4]);
        assert_eq!(array.layout().strides, vec![68, 4]);
        assert_eq!(array.layout().item_size, 4);
        assert!(!array.layout().has_internal_overlap());
        assert_eq!(array.data_ptr(), base.cast_const());

        // Out of scope stays out of scope, in both directions.
        assert!(array.field(0, "stroke_rgba").is_none());
        assert!(array.field_mut(0, "point").is_none());

        array
            .field_mut(1, "fill_rgba")
            .expect("in scope")
            .copy_from_slice(&[0.1, 0.2, 0.3, 0.4]);
        drop(array);
        drop(view);

        assert_eq!(buffer.storage_id(), generation);
        assert_eq!(buffer.read(1, "fill_rgba"), Some(vec![0.1, 0.2, 0.3, 0.4]));
    }

    /// The dtype's whole byte-offset table, against the same numbers
    /// `tests/data_plane.rs` locks to numpy's structured-dtype packing.
    ///
    /// A field-scoped export's `byte_offset()` is one row of this table, so
    /// pinning the table is what keeps a hand-typed constant in any single
    /// export test from being the only guard on the packing.
    #[test]
    fn vmobject_dtype_offsets_match_numpy_structured_packing() {
        let dtype = RecordDType::of(&RecordSchema::vmobject());
        assert_eq!(dtype.itemsize(), 68);
        for (field, byte_offset, lanes) in [
            ("point", 0, 3),
            ("stroke_rgba", 12, 4),
            ("stroke_width", 28, 1),
            ("joint_angle", 32, 1),
            ("fill_rgba", 36, 4),
            ("base_normal", 52, 3),
            ("fill_border_width", 64, 1),
        ] {
            assert_eq!(
                dtype.byte_offset(field),
                Some(byte_offset),
                "byte offset of {field}"
            );
            assert_eq!(dtype.subshape(field), Some(lanes), "subshape of {field}");
        }

        // Every field-scoped export reports its row of that table, and the
        // last field's lane view lands exactly on fnp's legality bound —
        // offset 64 leaves 35 reachable lanes for a (3, 1) view of stride 68.
        let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).expect("3 records");
        for (field, byte_offset, lanes) in [
            ("point", 0, 3),
            ("stroke_width", 28, 1),
            ("fill_rgba", 36, 4),
            ("fill_border_width", 64, 1),
        ] {
            let view = buffer
                .export_field_view(field, false)
                .expect("the field exists");
            let array = view.as_numpy().expect("the field view exports");
            assert_eq!(array.byte_offset(), byte_offset, "export offset of {field}");
            assert_eq!(array.layout().shape, vec![3, lanes], "shape of {field}");
            assert_eq!(array.layout().strides, vec![68, 4], "strides of {field}");
        }
    }

    #[test]
    fn resize_detaches_the_exported_array_with_numpy_natural_semantics() {
        let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 2).expect("2 records");
        assert!(buffer.write(0, "point", &[7.0, 8.0, 9.0]));
        let view = buffer.export_view(true);
        let pinned = view.foreign_data_ptr();

        // V3/V6: growth swaps in a fresh generation; the view keeps the old
        // one alive and keeps reading it, exactly as a detached NumPy array
        // aliasing freed-from-the-owner memory would not be allowed to.
        buffer.resize(5).expect("growth fits");
        assert!(!view.is_attached_to(&buffer));

        let array = view
            .as_numpy()
            .expect("the pinned generation still exports");
        assert_eq!(array.len(), 2, "the view's extent is fixed at export");
        assert_eq!(array.data_ptr(), pinned.cast_const());
        assert_eq!(array.field(0, "point"), Some(&[7.0, 8.0, 9.0][..]));
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn read_only_views_refuse_a_writable_export() {
        let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 1).expect("1 record");
        let view = buffer.export_view(false);
        assert_eq!(view.as_numpy_mut().err(), Some(ExportError::ReadOnly));
        let array = view.as_numpy().expect("read-only export");
        assert!(!array.layout().is_writeable());
        assert!(!view.numpy_layout().expect("layout").is_writeable());
    }
}
