//! The frame buffer: one contiguous allocation described by a
//! [`FrameLayout`] (§14.1).
//!
//! Orientation is D-23's rule made structural: a `FrameBuffer` is
//! ALWAYS in output orientation — row 0 is the top row of the delivered
//! image. There is no flipped variant, no orientation flag, and no
//! vflip anywhere in the system.

use std::sync::Arc;

use crate::format::FrameLayout;

/// An unforgeable, pool-local ownership capability.
///
/// Every pool has one allocation shared by its original buffers. Pointer
/// identity, not a caller-visible numeric tag, proves membership. The type and
/// field remain crate-private so a separately allocated [`FrameBuffer`] cannot
/// acquire pool provenance through the public API.
#[derive(Debug)]
pub(crate) struct PoolIdentity;

/// A single frame's pixel storage.
///
/// Each buffer's pixel allocation is created once at construction; everything
/// afterwards is slicing. Buffers are meant to be pooled
/// ([`crate::FramePool`]) so the render hot path never allocates,
/// resizes, or frees frame-sized memory (PG-6).
///
/// Cloning creates a detached byte-for-byte snapshot. It intentionally does
/// not clone private pool provenance: a copied allocation is not one of the
/// buffers a fixed-capacity pool preallocated.
#[derive(Debug)]
pub struct FrameBuffer {
    layout: FrameLayout,
    data: Vec<u8>,
    pool_identity: Option<Arc<PoolIdentity>>,
}

impl Clone for FrameBuffer {
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            data: self.data.clone(),
            pool_identity: None,
        }
    }
}

impl FrameBuffer {
    /// Allocate a zero-filled standalone buffer for `layout`.
    #[must_use]
    pub fn new(layout: FrameLayout) -> Self {
        let data = vec![0u8; layout.total_bytes()];
        Self {
            layout,
            data,
            pool_identity: None,
        }
    }

    /// Allocate one buffer carrying the private identity of its owning pool.
    #[must_use]
    pub(crate) fn new_pooled(layout: FrameLayout, pool_identity: Arc<PoolIdentity>) -> Self {
        let data = vec![0u8; layout.total_bytes()];
        Self {
            layout,
            data,
            pool_identity: Some(pool_identity),
        }
    }

    /// Whether this is an original allocation owned by `pool_identity`.
    #[must_use]
    pub(crate) fn belongs_to(&self, pool_identity: &Arc<PoolIdentity>) -> bool {
        self.pool_identity
            .as_ref()
            .is_some_and(|identity| Arc::ptr_eq(identity, pool_identity))
    }

    /// The negotiated geometry.
    #[must_use]
    pub const fn layout(&self) -> &FrameLayout {
        &self.layout
    }

    /// The bytes of `plane` (stride-padded rows included).
    #[must_use]
    pub fn plane(&self, plane: usize) -> &[u8] {
        let start = self.layout.plane_offset(plane);
        &self.data[start..start + self.layout.plane_bytes(plane)]
    }

    /// Mutable bytes of `plane`.
    pub fn plane_mut(&mut self, plane: usize) -> &mut [u8] {
        let start = self.layout.plane_offset(plane);
        let len = self.layout.plane_bytes(plane);
        &mut self.data[start..start + len]
    }

    /// The whole allocation, all planes, padding included.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// The whole allocation, mutable.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
