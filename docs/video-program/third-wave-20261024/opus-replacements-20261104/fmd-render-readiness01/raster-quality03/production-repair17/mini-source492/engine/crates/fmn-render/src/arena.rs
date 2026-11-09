//! PG-6's steady-state allocation contract, as data structures (fm-e9h, W5).
//!
//! §17's "eliminate work" rung applied to the certified engine's per-frame
//! heap traffic, which was structural rather than incidental:
//! [`crate::engine::FrameJob`] built a fresh `Vec` per draw (join wedges,
//! prepared stroke segments, gradient stations, transformed geometry) plus
//! the draw list itself — thousands of small allocations per frame on a
//! glyph-heavy scene — and every worker re-allocated its row scratch per
//! [`crate::engine::FrameJob::render_into`] call. This module is the fix, and
//! it is **storage only**: no arithmetic anywhere changed, which the
//! `certified_engine.certified.lock` goldens enforce at the bit level.
//!
//! Two mechanisms:
//!
//! - **Typed bump pools** ([`Pool`]): one contiguous buffer per payload kind
//!   (join wedges, transformed segments, monotone pieces, prepared stroke
//!   segments, gradient stations, the draw list). A frame bump-allocates
//!   ranges out of them; [`FrameArena::begin_frame`] truncates every pool to
//!   length zero *without releasing capacity*, so frame N+1 reuses frame N's
//!   buffers. A pool counts every growth of its backing `Vec` — under
//!   `forbid(unsafe_code)` a `Vec` grows only by allocating, so the count is
//!   exact and there is no allocator shim anywhere in the proof.
//! - **The worker pool** ([`crate::engine::WorkerPool`], owned here): one
//!   scratch slot per requested worker and (kernel, covered row width), sized
//!   synchronously before fan-out and returned when the render finishes.
//!   Steady state creates none, independent of worker-start order.
//!
//! The zero-allocation assertion therefore rides the engine's own counters:
//! render the same scene N+1 frames through one arena and frames 2..=N+1
//! must report [`AllocStats::heap_allocs_this_frame`] of zero. Frame 1 is the
//! documented warm-up that sizes every buffer.

use crate::engine::{Draw, WorkerPool};
use crate::fill::MonoPiece;
use crate::stroke::{JoinWedge, PreparedSegment};
use crate::table::Segment;
use std::collections::TryReserveError;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};

/// The engine's allocation ledger for one frame — the PG-6 proof's input.
///
/// Produced by [`FrameArena::stats`] and surfaced as
/// [`crate::engine::FrameJob::allocation_stats`] so the PG-6 producer
/// (fm-inr.3) can journal steady-state allocation behaviour without reaching
/// into engine internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    /// Heap allocations the arena and worker pool performed since
    /// [`FrameArena::begin_frame`]. A `Vec` growth, a fresh worker slot, or a
    /// worker slot outgrowing its tile column count each count as one batch
    /// of the allocations that operation performs. Zero on every frame after
    /// warm-up for a scene whose working set has stopped growing.
    pub heap_allocs_this_frame: u64,
    /// Total reserved capacity of the bump pools, in bytes. Constant across
    /// frames once the working set has stabilized — the observable form of
    /// "the arena buffer is allocated exactly once".
    pub arena_buffer_bytes: usize,
    /// Worker scratch slots the pool currently holds: one per requested worker
    /// and (kernel, covered row width), bounded in practice by the largest
    /// worker team and viewport the execution plan admits.
    pub pool_slots: usize,
}

/// A half-open `[start, start + len)` range into one of the arena's pools.
///
/// Ranges — not pointers — are what [`Draw`] stores, which is what lets a
/// [`crate::engine::FrameJob`] *own* its arena outright with no
/// self-reference: every access resolves the range against the pool at use
/// time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PoolRange {
    /// Index of the first element.
    pub(crate) start: u32,
    /// How many elements follow it.
    pub(crate) len: u32,
}

/// A frame-local pool range cannot be represented by [`PoolRange`].
///
/// The compact `u32` coordinates are intentional: one range is carried by
/// every prepared draw. Conversion into that representation must nevertheless
/// be checked, because the frame pools aggregate rows across every retained
/// instance and can therefore be wider than any one retained table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PoolRangeError {
    /// Pool whose range could not be represented.
    pub(crate) resource: &'static str,
    /// First requested row.
    pub(crate) start: u64,
    /// Number of requested rows.
    pub(crate) len: u64,
}

fn compact_range(
    resource: &'static str,
    start: u64,
    len: u64,
) -> Result<PoolRange, PoolRangeError> {
    let start = u32::try_from(start).map_err(|_| PoolRangeError {
        resource,
        start,
        len,
    })?;
    let len = u32::try_from(len).map_err(|_| PoolRangeError {
        resource,
        start: u64::from(start),
        len,
    })?;
    Ok(PoolRange { start, len })
}

/// An append target for the geometry builders: `Vec` for the retained and
/// test paths, [`Pool`] for the engine's per-frame path.
///
/// The builders (`join_wedges_into`, `pieces_for_segments_into`,
/// `PreparedStroke::prepare_into`, `GradientField::build_into`) are generic
/// over this trait so the *same code* — same order, same arithmetic — feeds
/// either an allocating vector or a counting bump pool. Two call sites, one
/// definition: the bit-lock cannot drift between them.
pub(crate) trait Sink<T> {
    /// Current number of initialized elements.
    fn len(&self) -> usize;

    /// Reserve space for an exact preflighted append.
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError>;

    /// Append one element.
    fn put(&mut self, value: T);
}

impl<T> Sink<T> for Vec<T> {
    #[inline]
    fn len(&self) -> usize {
        Vec::len(self)
    }

    #[inline]
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        Vec::try_reserve_exact(self, additional)
    }

    #[inline]
    fn put(&mut self, value: T) {
        self.push(value);
    }
}

impl<T> Sink<T> for Pool<T> {
    #[inline]
    fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        Pool::try_reserve(self, additional)
    }

    #[inline]
    fn put(&mut self, value: T) {
        Pool::put(self, value);
    }
}

/// One typed bump pool: a single backing buffer, truncated per frame.
///
/// This is the whole "arena" mechanism — one per payload kind rather than one
/// untyped byte buffer, because `#![forbid(unsafe_code)]` forbids
/// reinterpreting bytes and because typed pools need no alignment reasoning
/// at all. Bump allocation is appending; the per-frame reset is
/// [`Vec::clear`], which keeps the capacity.
#[derive(Debug)]
pub(crate) struct Pool<T> {
    buf: Vec<T>,
    /// Backing-buffer growth events since the last [`Pool::begin_frame`].
    ///
    /// A `Vec` reallocates only when a push finds `len == capacity`, so
    /// checking the watermark before every push makes the count exact. The
    /// counter lives in the pool so builders need no extra parameter; it is
    /// atomic (not a `Cell`) only so the arena stays `Sync` for the render
    /// threads — pool mutation itself happens during single-threaded frame
    /// preparation.
    allocs: AtomicU64,
}

impl<T> Default for Pool<T> {
    fn default() -> Pool<T> {
        Pool {
            buf: Vec::new(),
            allocs: AtomicU64::new(0),
        }
    }
}

impl<T> Deref for Pool<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.buf
    }
}

impl<T> Pool<T> {
    /// Reserve a preflighted append and account for backing-buffer growth.
    pub(crate) fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        let capacity = self.buf.capacity();
        self.buf.try_reserve_exact(additional)?;
        if self.buf.capacity() != capacity {
            self.allocs.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Append, counting the growth event if this push forces one.
    pub(crate) fn put(&mut self, value: T) {
        if self.buf.len() == self.buf.capacity() {
            self.allocs.fetch_add(1, Ordering::Relaxed);
        }
        self.buf.push(value);
    }

    /// Append a whole iterator, returning the range it landed in.
    pub(crate) fn extend(
        &mut self,
        resource: &'static str,
        values: impl IntoIterator<Item = T>,
    ) -> Result<PoolRange, PoolRangeError> {
        let start = self.buf.len();
        for value in values {
            self.put(value);
        }
        self.range_from(resource, start)
    }

    /// The range `[start, len)` — the second half of every "record, append,
    /// range" construction.
    pub(crate) fn range_from(
        &self,
        resource: &'static str,
        start: usize,
    ) -> Result<PoolRange, PoolRangeError> {
        debug_assert!(start <= self.buf.len());
        compact_range(
            resource,
            u64::try_from(start).unwrap_or(u64::MAX),
            u64::try_from(self.buf.len() - start).unwrap_or(u64::MAX),
        )
    }

    /// Resolve a range to its slice.
    pub(crate) fn slice(&self, range: PoolRange) -> &[T] {
        let lo = range.start as usize;
        let hi = lo + range.len as usize;
        debug_assert!(hi <= self.buf.len());
        &self.buf[lo..hi.min(self.buf.len())]
    }

    /// Resolve a range to its mutable slice (frame preparation only).
    pub(crate) fn slice_mut(&mut self, range: PoolRange) -> &mut [T] {
        let lo = range.start as usize;
        let hi = lo + range.len as usize;
        debug_assert!(hi <= self.buf.len());
        let hi = hi.min(self.buf.len());
        &mut self.buf[lo..hi]
    }

    /// Truncate to zero length, keeping the backing buffer, and zero the
    /// allocation counter for the new frame.
    pub(crate) fn begin_frame(&mut self) {
        self.buf.clear();
        self.allocs.store(0, Ordering::Relaxed);
    }

    /// Truncate scratch reuse *within* a frame (construction temporaries).
    pub(crate) fn clear(&mut self) {
        self.buf.clear();
    }

    /// Reserved capacity of the backing buffer, in bytes.
    pub(crate) fn buffer_bytes(&self) -> usize {
        self.buf.capacity() * size_of::<T>()
    }

    /// Growth events since [`Pool::begin_frame`].
    pub(crate) fn allocs(&self) -> u64 {
        self.allocs.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::{PoolRange, PoolRangeError, compact_range};

    #[test]
    fn compact_ranges_accept_the_exact_coordinate_boundary() {
        assert_eq!(
            compact_range("test rows", u64::from(u32::MAX), u64::from(u32::MAX)),
            Ok(PoolRange {
                start: u32::MAX,
                len: u32::MAX,
            })
        );
    }

    #[test]
    fn compact_ranges_refuse_start_or_length_truncation() {
        let past = u64::from(u32::MAX) + 1;
        assert_eq!(
            compact_range("test rows", past, 1),
            Err(PoolRangeError {
                resource: "test rows",
                start: past,
                len: 1,
            })
        );
        assert_eq!(
            compact_range("test rows", 1, past),
            Err(PoolRangeError {
                resource: "test rows",
                start: 1,
                len: past,
            })
        );
    }
}

/// The caller-owned per-frame scratch: bump pools for the draw derivation
/// plus the worker-scratch pool.
///
/// One arena serves a whole animation: the caller constructs it once, passes
/// it to [`crate::engine::FrameJob::new_in`] for every frame, and reads the
/// PG-6 ledger back through [`FrameArena::stats`]. `begin_frame` is called by
/// `FrameJob::new_in` itself, so the per-frame discipline cannot be skipped
/// by accident. The worker pool is *not* reset per frame — its slots are the
/// steady state — only its allocation counter is.
#[derive(Debug)]
pub struct FrameArena {
    /// Join wedges for every draw, contiguous.
    pub(crate) joins: Pool<JoinWedge>,
    /// Frame-local transformed segments for affine placements.
    pub(crate) segments: Pool<Segment>,
    /// Monotone pieces derived from those transformed segments.
    pub(crate) pieces: Pool<MonoPiece>,
    /// Per-draw prepared stroke segments (slabs and arc lengths).
    pub(crate) stroke_segments: Pool<PreparedSegment>,
    /// Gradient station positions.
    pub(crate) gradient_points: Pool<[f64; 2]>,
    /// Gradient station parameters.
    pub(crate) gradient_params: Pool<f64>,
    /// Per-station next indices for multi-contour gradient fields.
    pub(crate) gradient_next: Pool<usize>,
    /// Boundary parameter at the far end of each multi-contour station edge.
    pub(crate) gradient_edge_params: Pool<f64>,
    /// The draw list itself, index-aligned with the instance list.
    pub(crate) draws: Pool<Option<Draw>>,
    /// Construction scratch for `join_wedges_into`'s corner index pairs.
    pub(crate) join_pairs: Pool<(usize, usize)>,
    /// Construction scratch for `pieces_for_segments_into`'s control points.
    pub(crate) piece_curves: Pool<[[f64; 2]; 3]>,
    /// Worker row scratch, keyed on kernel and maximum covered row width. Sized
    /// by use: a render team of `t` threads converges to exactly `t` slots after
    /// the warm-up frame, which is how the execution plan's team geometry
    /// bounds the pool.
    pub(crate) workers: WorkerPool,
}

impl Default for FrameArena {
    fn default() -> FrameArena {
        FrameArena {
            joins: Pool::default(),
            segments: Pool::default(),
            pieces: Pool::default(),
            stroke_segments: Pool::default(),
            gradient_points: Pool::default(),
            gradient_params: Pool::default(),
            gradient_next: Pool::default(),
            gradient_edge_params: Pool::default(),
            draws: Pool::default(),
            join_pairs: Pool::default(),
            piece_curves: Pool::default(),
            workers: WorkerPool::new(),
        }
    }
}

impl FrameArena {
    /// An empty arena; every buffer sizes itself on the first frame.
    #[must_use]
    pub fn new() -> FrameArena {
        FrameArena::default()
    }

    /// Reset every bump pool for a new frame, keeping all backing buffers,
    /// and zero the per-frame allocation counters. Worker slots persist.
    pub fn begin_frame(&mut self) {
        self.joins.begin_frame();
        self.segments.begin_frame();
        self.pieces.begin_frame();
        self.stroke_segments.begin_frame();
        self.gradient_points.begin_frame();
        self.gradient_params.begin_frame();
        self.gradient_next.begin_frame();
        self.gradient_edge_params.begin_frame();
        self.draws.begin_frame();
        self.join_pairs.begin_frame();
        self.piece_curves.begin_frame();
        self.workers.begin_frame();
    }

    /// The PG-6 allocation report for the current frame so far.
    #[must_use]
    pub fn stats(&self) -> AllocStats {
        let pool_allocs = self.joins.allocs()
            + self.segments.allocs()
            + self.pieces.allocs()
            + self.stroke_segments.allocs()
            + self.gradient_points.allocs()
            + self.gradient_params.allocs()
            + self.gradient_next.allocs()
            + self.gradient_edge_params.allocs()
            + self.draws.allocs()
            + self.join_pairs.allocs()
            + self.piece_curves.allocs();
        AllocStats {
            heap_allocs_this_frame: pool_allocs + self.workers.allocs(),
            arena_buffer_bytes: self.joins.buffer_bytes()
                + self.segments.buffer_bytes()
                + self.pieces.buffer_bytes()
                + self.stroke_segments.buffer_bytes()
                + self.gradient_points.buffer_bytes()
                + self.gradient_params.buffer_bytes()
                + self.gradient_next.buffer_bytes()
                + self.gradient_edge_params.buffer_bytes()
                + self.draws.buffer_bytes()
                + self.join_pairs.buffer_bytes()
                + self.piece_curves.buffer_bytes(),
            pool_slots: self.workers.slots(),
        }
    }
}
