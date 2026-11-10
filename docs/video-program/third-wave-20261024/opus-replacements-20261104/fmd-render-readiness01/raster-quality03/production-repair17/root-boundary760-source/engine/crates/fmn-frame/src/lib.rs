//! Frame buffers, pixel formats (RGBA/RGBA16F/NV12/P010), and transfer
//! functions — the Reel substrate (§14.1, D-23).
//!
//! This crate owns the pixel-storage layer under the codecs, the
//! ordered emitter, and the negotiated ffmpeg boundary:
//!
//! - [`PixelFormat`] / [`FrameLayout`] — the negotiated-sink format
//!   vocabulary with row-stride discipline (sinks negotiate stride,
//!   internal buffers may pad).
//! - [`FrameBuffer`] / [`FramePool`] — pooled, preallocated frame
//!   storage; zero frame-sized allocations on the hot path (PG-6), and
//!   pool exhaustion is the backpressure signal, never a growth event.
//! - [`transfer`] — transfer functions applied once, natively, at the
//!   defined point, over deterministic `fmn-dmath` arithmetic (D-17):
//!   the certified canonical-RGBA path is table-driven and bit-exact
//!   on every platform.
//! - [`convert`] — the conversion kernels: linear RGBA16F → sRGB RGBA8
//!   (certified), RGB → BT.709 NV12/P010 in defined-rounding integer
//!   fixed point (standard-mode, for the ffmpeg boundary), and the
//!   RGBA⇄BGRA swizzle.
//!
//! Two structural rules hold everywhere: **orientation is always output
//! orientation** — row 0 is the top row of the delivered image, and no
//! vflip exists anywhere in the system (D-23) — and no kernel ever
//! allocates, resizes, or synchronously flushes a frame-sized buffer
//! (§14.3).
#![forbid(unsafe_code)]

pub mod buffer;
pub mod convert;
pub mod format;
pub mod half;
pub mod pool;
pub mod transfer;

pub use buffer::FrameBuffer;
pub use format::{ChromaSiting, ColorRange, FrameLayout, PixelFormat};
pub use pool::FramePool;

/// Typed refusals of the frame layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// A zero width or height is not a frame.
    ZeroDimension,
    /// 4:2:0 formats require even dimensions.
    OddDimensions {
        /// The subsampled format that was refused.
        format: PixelFormat,
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
    },
    /// Row alignment must be a power of two.
    BadAlignment {
        /// The alignment that was refused.
        alignment: usize,
    },
    /// A negotiated stride slice had the wrong number of planes.
    WrongStrideCount {
        /// Planes the format has.
        expected: usize,
        /// Strides provided.
        got: usize,
    },
    /// A negotiated stride does not cover the payload row.
    StrideTooSmall {
        /// Plane index.
        plane: usize,
        /// The stride that was refused.
        stride: usize,
        /// The minimum payload row bytes.
        min: usize,
    },
    /// A stride would split a multi-byte sample across rows.
    StrideMisaligned {
        /// Plane index.
        plane: usize,
        /// The stride that was refused.
        stride: usize,
        /// The format's sample size in bytes.
        sample_size: usize,
    },
    /// Layout arithmetic overflowed — the frame cannot exist in memory.
    TooLarge,
    /// The host refused part of a requested CPU render team.
    WorkerSpawnFailed {
        /// Total workers required before rendering could begin.
        requested: usize,
        /// Workers successfully started before the refusal.
        spawned: usize,
    },
    /// A buffer without this pool's private ownership capability, or with a
    /// foreign layout, was released into the pool.
    ForeignBuffer,
    /// More buffers were released than the pool owns.
    PoolOverflow,
    /// A conversion kernel got a buffer of the wrong format.
    FormatMismatch {
        /// What the kernel required.
        expected: &'static str,
        /// The format it got.
        got: PixelFormat,
    },
    /// Source and destination dimensions differ.
    DimensionMismatch,
    /// A caller-owned byte slice does not match the exact tight frame size.
    BufferLengthMismatch {
        /// Bytes required by the frame geometry.
        expected: usize,
        /// Bytes supplied by the caller.
        got: usize,
    },
    /// The requested conversion is a typed refusal, never a silent
    /// substitution.
    UnsupportedConversion(&'static str),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimension => write!(f, "frame dimensions must be nonzero"),
            Self::OddDimensions {
                format,
                width,
                height,
            } => write!(
                f,
                "{format:?} is 4:2:0-subsampled and requires even dimensions, got {width}x{height}"
            ),
            Self::BadAlignment { alignment } => {
                write!(f, "row alignment {alignment} is not a power of two")
            }
            Self::WrongStrideCount { expected, got } => {
                write!(f, "format has {expected} plane(s), got {got} stride(s)")
            }
            Self::StrideTooSmall { plane, stride, min } => write!(
                f,
                "plane {plane} stride {stride} is below the payload row width {min}"
            ),
            Self::StrideMisaligned {
                plane,
                stride,
                sample_size,
            } => write!(
                f,
                "plane {plane} stride {stride} is not a multiple of the {sample_size}-byte sample"
            ),
            Self::TooLarge => write!(f, "frame layout overflows addressable memory"),
            Self::WorkerSpawnFailed { requested, spawned } => write!(
                f,
                "CPU render team requested {requested} workers, but only {spawned} could be started"
            ),
            Self::ForeignBuffer => {
                write!(
                    f,
                    "buffer does not belong to this pool or has a foreign layout"
                )
            }
            Self::PoolOverflow => write!(f, "pool release would exceed its capacity"),
            Self::FormatMismatch { expected, got } => {
                write!(f, "conversion kernel expected {expected}, got {got:?}")
            }
            Self::DimensionMismatch => {
                write!(f, "source and destination dimensions differ")
            }
            Self::BufferLengthMismatch { expected, got } => {
                write!(f, "frame buffer needs exactly {expected} bytes, got {got}")
            }
            Self::UnsupportedConversion(what) => write!(f, "unsupported conversion: {what}"),
        }
    }
}

impl std::error::Error for FrameError {}

#[cfg(test)]
mod tests {
    use super::FrameError;

    #[test]
    fn worker_spawn_refusal_names_the_partial_team() {
        assert_eq!(
            FrameError::WorkerSpawnFailed {
                requested: 4,
                spawned: 1,
            }
            .to_string(),
            "CPU render team requested 4 workers, but only 1 could be started"
        );
    }

    #[test]
    fn foreign_buffer_refusal_names_pool_membership() {
        assert_eq!(
            FrameError::ForeignBuffer.to_string(),
            "buffer does not belong to this pool or has a foreign layout"
        );
    }
}
