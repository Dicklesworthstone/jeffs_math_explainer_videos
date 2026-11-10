#![feature(test)]
#![forbid(unsafe_code)]

extern crate test;

use fmn_anim::{PathFunc, interpolate_fields};
use fmn_mobject::{Mob, Mobject, RecordBuffer, RecordSchema, Stage};
use std::hint::black_box;
use test::Bencher;

const LANES: usize = 1 << 20;

fn field(stage: &mut Stage, values: &[f32]) -> Mob {
    let mut buffer = RecordBuffer::new(
        RecordSchema::new(&[("value", 1)], &["value"], &[]).unwrap(),
        values.len(),
    )
    .unwrap();
    assert!(buffer.write_range("value", 0, values));
    stage.add(Mobject::from_buffer(buffer))
}

struct Fixture {
    stage: Stage,
    live: Mob,
    from: Mob,
    to: Mob,
}

impl Fixture {
    fn new() -> Self {
        let from: Vec<f32> = (0..LANES)
            .map(|i| i as f32 * (1.0 / LANES as f32) - 0.5)
            .collect();
        let to: Vec<f32> = from.iter().rev().map(|value| 1.25 - value).collect();
        let mut stage = Stage::new();
        let live = field(&mut stage, &vec![0.0; LANES]);
        let from = field(&mut stage, &from);
        let to = field(&mut stage, &to);
        Self {
            stage,
            live,
            from,
            to,
        }
    }

    fn interpolate(&mut self) {
        interpolate_fields(
            &mut self.stage,
            self.live,
            self.from,
            self.to,
            black_box(0.314_159_265_358_979_3),
            PathFunc::Straight,
        );
        black_box(
            self.stage
                .get(self.live)
                .expect("live benchmark mobject")
                .buffer
                .field_revision("value"),
        );
    }
}

#[bench]
fn million_lane_linear_field(bench: &mut Bencher) {
    let mut fixture = Fixture::new();
    bench.iter(|| fixture.interpolate());
}

#[test]
fn profile_one_million_lane_field() {
    Fixture::new().interpolate();
}
