//! Explicit seed authoring over the production RK45/arc-length path.
use fmn_core::rng::RngRoot;
use fmn_core::types::Vec3;
use fmn_library::coords::CoordinateSystem;
use fmn_library::fields::{StreamLineStyle, StreamLines, get_sample_coords};
use fmn_library::graphs::SamplingBudget;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct Grid {
    ranges: Vec<[f64; 3]>,
}
impl CoordinateSystem for Grid {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        [
            coords.first().copied().unwrap_or(0.0),
            coords.get(1).copied().unwrap_or(0.0),
            0.0,
        ]
    }
    fn p2c(&self, point: Vec3) -> Vec3 {
        point
    }
    fn all_ranges(&self) -> Vec<[f64; 3]> {
        self.ranges.clone()
    }
    fn dimension(&self) -> usize {
        2
    }
}
fn grid() -> Grid {
    Grid {
        ranges: vec![[-1.0, 1.0, 1.0], [-1.0, 1.0, 1.0]],
    }
}
fn field(rows: &[[f64; 3]]) -> Vec<Vec3> {
    vec![[1.0, 0.0, 0.0]; rows.len()]
}
fn builder() -> StreamLines {
    StreamLines::new(field, grid(), &RngRoot::from_seed(0))
        .with_solution_time(0.5)
        .with_dt(0.125)
        .with_samples_per_line(5)
        .with_style_config(StreamLineStyle {
            color_by_magnitude: false,
            ..Default::default()
        })
}

#[test]
fn explicit_seeds_are_ordered_and_consume_no_jitter() {
    let seeds = vec![[0.0, 2.0, 0.0], [-1.0, -2.0, 0.0]];
    let sampler = builder().with_sample_coords(seeds.clone());
    assert_eq!(sampler.sample_coords().unwrap(), (seeds.clone(), 0));
    let built = sampler.build().unwrap();
    assert_eq!(built.lines().len(), 2);
    assert_eq!(built.rng_draws(), 0);
    for (child, seed) in built.vmob().children().iter().zip(seeds) {
        let points = child.points();
        assert_eq!(points[0], seed);
        // Native dense samples use [0, solution_time), with dt = 0.125.
        assert!((points.last().unwrap()[0] - seed[0] - 0.375).abs() < 1e-9);
    }
}

#[test]
fn generated_seeds_reintegrate_to_the_same_geometry() {
    let original = builder().with_noise_factor(0.25).with_n_repeats(2);
    let (seeds, draws) = original.sample_coords().unwrap();
    let rebuilt = builder().with_sample_coords(seeds).build().unwrap();
    let original = original.build().unwrap();
    assert_eq!(original.rng_draws(), draws);
    assert!(draws > 0);
    for (left, right) in original
        .vmob()
        .children()
        .iter()
        .zip(rebuilt.vmob().children())
    {
        assert_eq!(left.points(), right.points());
    }
    assert_eq!(original.lines(), rebuilt.lines());
}

#[test]
fn explicit_nonfinite_seeds_refuse_before_any_field_callback() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let invalid = StreamLines::new(
            move |rows| {
                observed.set(observed.get() + 1);
                field(rows)
            },
            grid(),
            &RngRoot::from_seed(0),
        )
        .with_sample_coords(vec![[0.0; 3], [value, 0.0, 0.0]]);
        assert!(invalid.build().is_err());
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn empty_explicit_seeds_produce_an_empty_family_without_callbacks() {
    let empty = StreamLines::new(
        |_| panic!("empty seed set called the field"),
        grid(),
        &RngRoot::from_seed(0),
    )
    .with_sample_coords(vec![])
    .build()
    .unwrap();
    assert!(empty.lines().is_empty());
    assert!(empty.vmob().children().is_empty());
    assert_eq!(empty.rng_draws(), 0);
}

#[test]
fn empty_grid_axes_are_an_empty_cartesian_product_not_a_panic() {
    let empty = Grid {
        ranges: vec![[2.0, -2.0, 1.0], [-1.0, 1.0, 1.0]],
    };
    assert!(
        get_sample_coords(&empty, 1.0, SamplingBudget::DEFAULT)
            .unwrap()
            .is_empty()
    );
    let lines = StreamLines::new(
        |_| panic!("empty grid called field"),
        empty,
        &RngRoot::from_seed(0),
    )
    .build()
    .unwrap();
    assert!(lines.lines().is_empty());
}

#[test]
fn invalid_grid_dimensions_refuse_before_indexing() {
    for ranges in [vec![], vec![[0.0, 1.0, 1.0]; 4]] {
        assert!(get_sample_coords(&Grid { ranges }, 1.0, SamplingBudget::DEFAULT).is_err());
    }
}
