#![feature(test)]
#![forbid(unsafe_code)]

extern crate test;

use fmn_core::types::Vec3;
use fmn_geom::cubic::{DEFAULT_TOLERANCE_SCENE, cubic_to_quadratics, segments_for_tolerance};
use std::hint::black_box;
use test::Bencher;

const INGESTION_CORPUS: [([Vec3; 4], f64); 5] = [
    (
        [
            [-1.0, -1.0, 0.0],
            [-1.0, 1.6, 0.0],
            [1.0, 1.6, 0.0],
            [1.0, -1.0, 0.0],
        ],
        DEFAULT_TOLERANCE_SCENE,
    ),
    (
        [
            [0.0, 0.0, 0.0],
            [2.0, 3.0, 0.0],
            [-2.0, 3.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        DEFAULT_TOLERANCE_SCENE,
    ),
    (
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        DEFAULT_TOLERANCE_SCENE,
    ),
    (
        [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ],
        DEFAULT_TOLERANCE_SCENE,
    ),
    (
        [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0 / 3.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
        DEFAULT_TOLERANCE_SCENE,
    ),
];

#[bench]
fn size_ingestion_cubics(bench: &mut Bencher) {
    bench.iter(|| {
        let mut total = 0usize;
        for (p, tolerance) in INGESTION_CORPUS {
            total +=
                segments_for_tolerance(p[0], p[1], p[2], p[3], tolerance).expect("fixture fits");
        }
        black_box(total)
    });
}

#[bench]
fn convert_ingestion_cubics(bench: &mut Bencher) {
    bench.iter(|| {
        let mut total = 0usize;
        for (p, tolerance) in INGESTION_CORPUS {
            total += cubic_to_quadratics(p[0], p[1], p[2], p[3], tolerance)
                .expect("fixture fits")
                .len();
        }
        black_box(total)
    });
}
