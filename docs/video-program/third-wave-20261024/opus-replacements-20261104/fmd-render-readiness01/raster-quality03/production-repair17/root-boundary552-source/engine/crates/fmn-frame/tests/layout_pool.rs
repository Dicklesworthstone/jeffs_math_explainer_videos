//! fm-a25 acceptance: stride/negotiation unit tests and pool reuse
//! tests (steady-state render loop performs zero frame allocations).

use fmn_frame::{FrameBuffer, FrameError, FrameLayout, FramePool, PixelFormat};

#[test]
fn tight_strides_per_format() {
    let l = FrameLayout::tight(PixelFormat::Rgba8, 640, 480).unwrap();
    assert_eq!(l.stride(0), 640 * 4);
    assert_eq!(l.total_bytes(), 640 * 4 * 480);

    let l = FrameLayout::tight(PixelFormat::Rgba16F, 640, 480).unwrap();
    assert_eq!(l.stride(0), 640 * 8);

    let l = FrameLayout::tight(PixelFormat::Nv12, 640, 480).unwrap();
    assert_eq!(l.stride(0), 640);
    assert_eq!(l.stride(1), 640);
    assert_eq!(l.plane_offset(1), 640 * 480);
    // Chroma plane is half height: total = luma + w * h/2.
    assert_eq!(l.total_bytes(), 640 * 480 + 640 * 240);

    let l = FrameLayout::tight(PixelFormat::P010, 640, 480).unwrap();
    assert_eq!(l.stride(0), 640 * 2);
    assert_eq!(l.stride(1), 640 * 2);
    assert_eq!(l.total_bytes(), 640 * 2 * 480 + 640 * 2 * 240);
}

#[test]
fn negotiated_row_alignment_pads_strides() {
    let l = FrameLayout::with_row_alignment(PixelFormat::Rgba8, 130, 4, 64).unwrap();
    // 130 * 4 = 520 → padded to 576.
    assert_eq!(l.stride(0), 576);
    assert_eq!(l.total_bytes(), 576 * 4);

    let l = FrameLayout::with_row_alignment(PixelFormat::Nv12, 130, 4, 64).unwrap();
    assert_eq!(l.stride(0), 192); // 130 → 192
    assert_eq!(l.stride(1), 192);

    // Alignment 1 is the tight layout.
    assert_eq!(
        FrameLayout::with_row_alignment(PixelFormat::Rgba8, 130, 4, 1).unwrap(),
        FrameLayout::tight(PixelFormat::Rgba8, 130, 4).unwrap()
    );

    assert_eq!(
        FrameLayout::with_row_alignment(PixelFormat::Rgba8, 130, 4, 48),
        Err(FrameError::BadAlignment { alignment: 48 })
    );
}

#[test]
fn negotiated_strides_are_validated() {
    // Below payload width.
    assert_eq!(
        FrameLayout::with_strides(PixelFormat::Rgba8, 100, 4, &[399]),
        Err(FrameError::StrideTooSmall {
            plane: 0,
            stride: 399,
            min: 400
        })
    );
    // A P010 stride that splits a u16 sample.
    assert_eq!(
        FrameLayout::with_strides(PixelFormat::P010, 100, 4, &[201, 200]),
        Err(FrameError::StrideMisaligned {
            plane: 0,
            stride: 201,
            sample_size: 2
        })
    );
    // Wrong plane count.
    assert_eq!(
        FrameLayout::with_strides(PixelFormat::Nv12, 100, 4, &[100]),
        Err(FrameError::WrongStrideCount {
            expected: 2,
            got: 1
        })
    );
    // Over-wide negotiated strides are allowed.
    let l = FrameLayout::with_strides(PixelFormat::Nv12, 100, 4, &[128, 256]).unwrap();
    assert_eq!(l.stride(0), 128);
    assert_eq!(l.stride(1), 256);
}

#[test]
fn dimension_refusals() {
    assert_eq!(
        FrameLayout::tight(PixelFormat::Rgba8, 0, 4),
        Err(FrameError::ZeroDimension)
    );
    assert_eq!(
        FrameLayout::tight(PixelFormat::Nv12, 3, 4),
        Err(FrameError::OddDimensions {
            format: PixelFormat::Nv12,
            width: 3,
            height: 4
        })
    );
    // Odd dimensions are fine for single-plane formats.
    assert!(FrameLayout::tight(PixelFormat::Rgba8, 3, 5).is_ok());
}

#[test]
fn buffer_planes_are_disjoint_and_sized() {
    let l = FrameLayout::tight(PixelFormat::Nv12, 8, 6).unwrap();
    let mut b = FrameBuffer::new(l.clone());
    assert_eq!(b.plane(0).len(), 8 * 6);
    assert_eq!(b.plane(1).len(), 8 * 3);
    assert_eq!(b.as_bytes().len(), l.total_bytes());

    // Writing all of plane 0 leaves plane 1 untouched.
    b.plane_mut(0).fill(0xFF);
    assert!(b.plane(1).iter().all(|&x| x == 0));
}

#[test]
fn pool_backpressure_and_reuse() {
    let layout = FrameLayout::tight(PixelFormat::Rgba8, 16, 16).unwrap();
    let mut pool = FramePool::new(layout.clone(), 2);
    assert_eq!(pool.capacity(), 2);
    assert_eq!(pool.available(), 2);

    let mut a = pool.try_acquire().unwrap();
    let b = pool.try_acquire().unwrap();
    // Exhaustion is backpressure, never an allocation.
    assert!(pool.try_acquire().is_none());

    // Mark a buffer, release it, re-acquire: the same storage comes
    // back stale (proof of reuse — the pool neither reallocates nor
    // re-zeroes).
    a.plane_mut(0)[0] = 0xEE;
    pool.release(a).unwrap();
    let a2 = pool.try_acquire().unwrap();
    assert_eq!(a2.plane(0)[0], 0xEE);

    pool.release(a2).unwrap();
    pool.release(b).unwrap();
    assert_eq!(pool.available(), 2);
}

#[test]
fn pool_steady_state_is_allocation_free() {
    // Structural form of the PG-6 gate at this layer: a long
    // acquire/release loop never hits an allocation path — the pool has
    // none after construction — and never changes shape.
    let layout = FrameLayout::tight(PixelFormat::Rgba8, 64, 64).unwrap();
    let mut pool = FramePool::new(layout, 3);
    for _ in 0..1000 {
        let f0 = pool.try_acquire().unwrap();
        let f1 = pool.try_acquire().unwrap();
        pool.release(f0).unwrap();
        pool.release(f1).unwrap();
    }
    assert_eq!(pool.capacity(), 3);
    assert_eq!(pool.available(), 3);
}

#[test]
fn pool_refuses_foreign_buffers_before_capacity_checks() {
    let layout = FrameLayout::tight(PixelFormat::Rgba8, 16, 16).unwrap();
    let other = FrameLayout::tight(PixelFormat::Rgba8, 16, 17).unwrap();
    let mut pool = FramePool::new(layout.clone(), 1);

    assert_eq!(
        pool.release(FrameBuffer::new(other)),
        Err(FrameError::ForeignBuffer)
    );
    assert_eq!(pool.available(), 1);

    // Layout equality is not pool membership. Provenance is checked
    // before capacity so a full pool cannot mask a foreign buffer.
    assert_eq!(
        pool.release(FrameBuffer::new(layout)),
        Err(FrameError::ForeignBuffer)
    );
    assert_eq!(pool.available(), 1);
}
