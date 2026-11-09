#![feature(test)]
#![forbid(unsafe_code)]

extern crate test;

use fmn_frame::convert::{rgba_to_nv12, rgba_to_p010, rgba16f_to_rgba8, swap_rb8};
use fmn_frame::{ChromaSiting, ColorRange, FrameBuffer, FrameLayout, PixelFormat};
use std::hint::black_box;
use test::Bencher;

/// Seed every source with an order-sensitive, nonuniform byte stream.
///
/// Setup stays outside `Bencher::iter`; the kernel alone is measured. For the
/// binary16 source this visits arbitrary raw bit patterns, including the
/// negative, infinite, and NaN entries whose table semantics are certified.
fn seed_bytes(buffer: &mut FrameBuffer) {
    let mut state = 0x9e37_79b9_u32;
    for byte in buffer.as_bytes_mut() {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *byte = (state >> 24) as u8;
    }
}

struct Rgba16Fixture {
    src: FrameBuffer,
    dst: FrameBuffer,
}

impl Rgba16Fixture {
    fn new(pixels: u32) -> Self {
        let mut src = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba16F, pixels, 1).expect("benchmark source layout"),
        );
        seed_bytes(&mut src);
        let dst = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, pixels, 1)
                .expect("benchmark destination layout"),
        );
        Self { src, dst }
    }

    fn convert(&mut self) {
        rgba16f_to_rgba8(black_box(&self.src), black_box(&mut self.dst))
            .expect("benchmark conversion");
        black_box(self.dst.plane(0));
    }
}

struct SwapFixture {
    src: FrameBuffer,
    dst: FrameBuffer,
}

impl SwapFixture {
    fn new(pixels: u32) -> Self {
        let mut src = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, pixels, 1).expect("benchmark source layout"),
        );
        seed_bytes(&mut src);
        let dst = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Bgra8, pixels, 1)
                .expect("benchmark destination layout"),
        );
        Self { src, dst }
    }

    fn convert(&mut self) {
        swap_rb8(black_box(&self.src), black_box(&mut self.dst)).expect("benchmark swizzle");
        black_box(self.dst.plane(0));
    }
}

struct YuvFixture {
    src: FrameBuffer,
    dst: FrameBuffer,
}

impl YuvFixture {
    fn new(format: PixelFormat) -> Self {
        const SIDE: u32 = 512;
        let mut src = FrameBuffer::new(
            FrameLayout::tight(PixelFormat::Rgba8, SIDE, SIDE).expect("benchmark source layout"),
        );
        seed_bytes(&mut src);
        let dst = FrameBuffer::new(
            FrameLayout::tight(format, SIDE, SIDE).expect("benchmark destination layout"),
        );
        Self { src, dst }
    }

    fn nv12(&mut self) {
        rgba_to_nv12(
            black_box(&self.src),
            black_box(&mut self.dst),
            ColorRange::Limited,
            ChromaSiting::Center,
        )
        .expect("benchmark NV12 conversion");
        black_box(self.dst.as_bytes());
    }

    fn p010(&mut self) {
        rgba_to_p010(
            black_box(&self.src),
            black_box(&mut self.dst),
            ColorRange::Limited,
            ChromaSiting::Center,
        )
        .expect("benchmark P010 conversion");
        black_box(self.dst.as_bytes());
    }
}

fn benchmark_rgba16(bench: &mut Bencher, pixels: u32) {
    let mut fixture = Rgba16Fixture::new(pixels);
    bench.iter(|| fixture.convert());
}

fn benchmark_swap(bench: &mut Bencher, pixels: u32) {
    let mut fixture = SwapFixture::new(pixels);
    bench.iter(|| fixture.convert());
}

macro_rules! rgba16_bench {
    ($name:ident, $pixels:expr) => {
        #[bench]
        fn $name(bench: &mut Bencher) {
            benchmark_rgba16(bench, $pixels);
        }
    };
}

macro_rules! swap_bench {
    ($name:ident, $pixels:expr) => {
        #[bench]
        fn $name(bench: &mut Bencher) {
            benchmark_swap(bench, $pixels);
        }
    };
}

rgba16_bench!(rgba16f_to_rgba8_0001, 1);
rgba16_bench!(rgba16f_to_rgba8_0004, 4);
rgba16_bench!(rgba16f_to_rgba8_0008, 8);
rgba16_bench!(rgba16f_to_rgba8_0016, 16);
rgba16_bench!(rgba16f_to_rgba8_0256, 256);
rgba16_bench!(rgba16f_to_rgba8_4096, 4_096);
rgba16_bench!(rgba16f_to_rgba8_65536, 65_536);
rgba16_bench!(rgba16f_to_rgba8_262144, 262_144);

swap_bench!(swap_rb8_0001, 1);
swap_bench!(swap_rb8_0004, 4);
swap_bench!(swap_rb8_0008, 8);
swap_bench!(swap_rb8_0016, 16);
swap_bench!(swap_rb8_0256, 256);
swap_bench!(swap_rb8_4096, 4_096);
swap_bench!(swap_rb8_65536, 65_536);
swap_bench!(swap_rb8_262144, 262_144);

#[bench]
fn rgba_to_nv12_512x512(bench: &mut Bencher) {
    let mut fixture = YuvFixture::new(PixelFormat::Nv12);
    bench.iter(|| fixture.nv12());
}

#[bench]
fn rgba_to_p010_512x512(bench: &mut Bencher) {
    let mut fixture = YuvFixture::new(PixelFormat::P010);
    bench.iter(|| fixture.p010());
}
