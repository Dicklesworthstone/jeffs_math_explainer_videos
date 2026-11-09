//! The preallocated frame pool (§14.1, §14.3, PG-6).
//!
//! The in-flight frame budget is real memory — a 4K RGBA frame is
//! ~33 MB — so the pool preallocates its whole capacity up front and
//! never grows. An exhausted pool returns `None` from
//! [`FramePool::try_acquire`]; that refusal IS the backpressure signal
//! the ordered emitter (fm-hv4) and the pipeline (§17.4) propagate.
//! Zero frame-sized allocations happen after construction, which is
//! exactly what the steady-state allocation gate instruments.

use std::sync::Arc;

use crate::FrameError;
use crate::buffer::{FrameBuffer, PoolIdentity};
use crate::format::FrameLayout;

/// A fixed-capacity pool of interchangeable [`FrameBuffer`]s sharing
/// one [`FrameLayout`].
#[derive(Debug)]
pub struct FramePool {
    layout: FrameLayout,
    identity: Arc<PoolIdentity>,
    free: Vec<FrameBuffer>,
    capacity: usize,
}

impl FramePool {
    /// Preallocate `capacity` buffers of `layout`. This is the only
    /// place the pool ever allocates frame memory.
    #[must_use]
    pub fn new(layout: FrameLayout, capacity: usize) -> Self {
        let identity = Arc::new(PoolIdentity);
        let free = (0..capacity)
            .map(|_| FrameBuffer::new_pooled(layout.clone(), Arc::clone(&identity)))
            .collect();
        Self {
            layout,
            identity,
            free,
            capacity,
        }
    }

    /// Take a buffer, or `None` if the pool is exhausted (backpressure —
    /// the pool never allocates to satisfy demand).
    ///
    /// The returned buffer's contents are stale (whatever the previous
    /// user wrote); the hot path overwrites, it does not re-zero.
    pub fn try_acquire(&mut self) -> Option<FrameBuffer> {
        self.free.pop()
    }

    /// Return one of this pool's original buffers.
    ///
    /// Refuses standalone allocations, buffers from another pool, detached
    /// clones, and buffers of a foreign layout. Layout equality alone is not
    /// membership: accepting a separately allocated same-layout buffer would
    /// silently replace preallocated storage and strand a genuine lease.
    pub fn release(&mut self, buffer: FrameBuffer) -> Result<(), FrameError> {
        if !buffer.belongs_to(&self.identity) || *buffer.layout() != self.layout {
            return Err(FrameError::ForeignBuffer);
        }
        if self.free.len() >= self.capacity {
            return Err(FrameError::PoolOverflow);
        }
        self.free.push(buffer);
        Ok(())
    }

    /// The layout every pooled buffer shares.
    #[must_use]
    pub const fn layout(&self) -> &FrameLayout {
        &self.layout
    }

    /// Total buffers the pool owns (free + outstanding).
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Buffers currently available to acquire.
    #[must_use]
    pub fn available(&self) -> usize {
        self.free.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::PixelFormat;

    fn rgba_layout() -> FrameLayout {
        FrameLayout::tight(PixelFormat::Rgba8, 2, 2).expect("test layout")
    }

    #[test]
    fn owned_buffer_round_trips_without_growth() {
        let layout = rgba_layout();
        let mut pool = FramePool::new(layout.clone(), 2);
        assert_eq!(pool.layout(), &layout);
        assert_eq!(pool.capacity(), 2);
        assert_eq!(pool.available(), 2);

        let mut first = pool.try_acquire().expect("first pooled buffer");
        let second = pool.try_acquire().expect("second pooled buffer");
        assert!(pool.try_acquire().is_none());
        first.as_bytes_mut().fill(0x5a);

        pool.release(first).expect("return first buffer");
        pool.release(second).expect("return second buffer");
        assert_eq!(pool.available(), 2);

        let recycled = [
            pool.try_acquire().expect("first recycled buffer"),
            pool.try_acquire().expect("second recycled buffer"),
        ];
        assert!(
            recycled
                .iter()
                .any(|buffer| buffer.as_bytes().iter().all(|&byte| byte == 0x5a))
        );
        assert!(pool.try_acquire().is_none());
    }

    #[test]
    fn capacity_guard_rejects_a_forged_extra_pool_member() {
        let layout = rgba_layout();
        let mut pool = FramePool::new(layout.clone(), 1);
        let extra = FrameBuffer::new_pooled(layout, Arc::clone(&pool.identity));

        assert_eq!(pool.release(extra), Err(FrameError::PoolOverflow));
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn standalone_same_layout_buffer_is_foreign() {
        let layout = rgba_layout();
        let mut pool = FramePool::new(layout.clone(), 2);
        let _lease = pool.try_acquire().expect("pooled lease");
        assert_eq!(pool.available(), 1);

        let standalone = FrameBuffer::new(layout);
        assert_eq!(pool.release(standalone), Err(FrameError::ForeignBuffer));
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn buffer_from_another_pool_is_foreign() {
        let layout = rgba_layout();
        let mut first_pool = FramePool::new(layout.clone(), 1);
        let mut second_pool = FramePool::new(layout, 1);
        let foreign = second_pool.try_acquire().expect("foreign pooled buffer");

        assert_eq!(first_pool.release(foreign), Err(FrameError::ForeignBuffer));
        assert_eq!(first_pool.available(), 1);
        assert_eq!(second_pool.available(), 0);
    }

    #[test]
    fn clone_is_a_detached_snapshot_not_a_second_lease() {
        let mut pool = FramePool::new(rgba_layout(), 1);
        let original = pool.try_acquire().expect("original pooled buffer");
        let detached = original.clone();

        assert_eq!(pool.release(detached), Err(FrameError::ForeignBuffer));
        assert_eq!(pool.available(), 0);
        pool.release(original).expect("return original lease");
        assert_eq!(pool.available(), 1);
    }
}
