//! fm-wuq acceptance: the RationalFrameClock against the Reference's
//! emission formula, the drift guarantee, and the no-off-grid property.

use fmn_anim::{ClockError, RationalFrameClock, RationalTime};

/// Sample-count fixtures across awkward run_times. `expected` is
/// `ceil(rational(run_time_f64) · fps)` computed exactly (cross-checked
/// with Python `fractions.Fraction`); `reference` is what the Python
/// engine's `np.arange(0, run_time, 1/fps)` emits. Divergences are the
/// BN-02 set: binary representation excess the float pipeline drops.
const COUNT_FIXTURES: &[(f64, u32, i64, i64)] = &[
    // (run_time, fps, expected_ours, reference)
    (1.0, 30, 30, 30),
    (1.0, 60, 60, 60),
    (1.0, 24, 24, 24),
    (0.1, 30, 4, 3), // BN-02: f64(0.1) > 1/10, three frames don't cover it
    (0.1, 60, 7, 6), // BN-02
    (0.1, 24, 3, 3),
    (1.0 / 3.0, 30, 10, 10),
    (1.0 / 3.0, 60, 20, 20),
    (1.0 / 3.0, 24, 8, 8),
    (2.9999, 30, 90, 90),
    (2.9999, 60, 180, 180),
    (2.9999, 24, 72, 72),
    (2.0, 30, 60, 60),
    (0.5, 60, 30, 30),
    (1.0 / 60.0, 30, 1, 1),
    (3.0, 30, 90, 90),
    (10.0, 24, 240, 240),
];

#[test]
fn sample_counts_match_exact_rational_ceiling() {
    for &(run_time, fps, expected, _reference) in COUNT_FIXTURES {
        let clock = RationalFrameClock::new(fps).unwrap();
        let segment = clock.segment(run_time).unwrap();
        assert_eq!(
            segment.n_frames(),
            expected,
            "run_time {run_time} at {fps} fps"
        );
        assert_eq!(segment.samples().count() as i64, expected);
    }
}

#[test]
fn emission_semantics_match_reference_shape() {
    let clock = RationalFrameClock::new(30).unwrap();
    let segment = clock.segment(1.0).unwrap();
    let samples: Vec<_> = segment.samples().collect();

    // No alpha-zero emission frame: the first sample is 1/fps.
    assert_eq!(samples[0].frame, 1);
    assert!(samples[0].alpha > 0.0);
    assert_eq!(samples[0].time, RationalTime::zero(30) + 1);

    // Alphas are nondecreasing and end exactly at 1.
    for pair in samples.windows(2) {
        assert!(pair[1].alpha >= pair[0].alpha);
    }
    assert_eq!(samples.last().unwrap().alpha, 1.0);

    // Times are exactly k/fps — the whole grid, nothing off it.
    for (k, sample) in samples.iter().enumerate() {
        assert_eq!(sample.time.frames(), k as i64 + 1);
        assert_eq!(sample.time.fps(), 30);
    }
}

#[test]
fn upward_rounding_covers_the_duration_and_clamps_alpha() {
    // 0.1 at 30 fps: the fourth frame overshoots; its alpha clamps.
    let clock = RationalFrameClock::new(30).unwrap();
    let segment = clock.segment(0.1).unwrap();
    let samples: Vec<_> = segment.samples().collect();
    assert_eq!(samples.len(), 4);
    // The grid end covers the duration…
    assert_eq!(
        segment.end_time().cmp_seconds(0.1),
        Some(std::cmp::Ordering::Greater)
    );
    // …the third frame alone would not have (BN-02's reason).
    let third = samples[2].time;
    assert_eq!(third.cmp_seconds(0.1), Some(std::cmp::Ordering::Less));
    // Raw alpha of the last frame exceeds 1; emitted alpha is clamped.
    assert_eq!(samples[3].alpha, 1.0);
    assert!(samples[3].time.to_f64() / 0.1 > 1.0);
}

#[test]
fn coverage_property_over_many_durations() {
    // For every positive duration: (n-1)/fps < run_time <= n/fps … the
    // right-closed form of upward rounding, exact, no floats involved.
    let clock = RationalFrameClock::new(60).unwrap();
    for i in 1..20_000u32 {
        let run_time = f64::from(i) * 7.3e-4;
        let segment = clock.segment(run_time).unwrap();
        let n = segment.n_frames();
        assert_ne!(
            segment.end_time().cmp_seconds(run_time),
            Some(std::cmp::Ordering::Less),
            "n/fps must reach run_time {run_time}"
        );
        let prev = RationalTime::zero(60) + (n - 1);
        assert_eq!(
            prev.cmp_seconds(run_time),
            Some(std::cmp::Ordering::Less),
            "(n-1)/fps must fall short of run_time {run_time}"
        );
    }
}

#[test]
fn drift_free_over_a_million_frames() {
    let mut clock = RationalFrameClock::new(30).unwrap();
    for _ in 0..1_000_000 {
        clock.advance_frames(1).expect("frame");
    }
    // Time derives: exactly the closed form, not an accumulation.
    assert_eq!(clock.now().frames(), 1_000_000);
    assert_eq!(
        clock.now().to_f64().to_bits(),
        (1_000_000.0f64 / 30.0).to_bits()
    );
    // And exact rational equality with the closed-form instant.
    let closed_form = RationalTime::zero(30) + 1_000_000;
    assert_eq!(clock.now(), closed_form);
    // The float pipeline this replaces disagrees with itself here:
    // accumulating 1/30 a million times drifts by many ulps.
    let mut accumulated = 0.0f64;
    for _ in 0..1_000_000 {
        accumulated += 1.0 / 30.0;
    }
    assert_ne!(accumulated.to_bits(), clock.now().to_f64().to_bits());
}

#[test]
fn wait_and_wait_until_interaction() {
    // wait(): consume the whole progression, advance per emitted frame.
    let mut clock = RationalFrameClock::new(30).unwrap();
    let segment = clock.segment(2.0).unwrap();
    for _sample in segment.samples() {
        clock.advance_frames(1).expect("frame");
    }
    assert_eq!(clock.now(), RationalTime::zero(30) + 60);

    // wait_until(): the stop condition breaks the loop early — the clock
    // advances only for emitted frames, and remains exactly on the grid.
    let segment = clock.segment(60.0).unwrap();
    let mut emitted = 0;
    for sample in segment.samples() {
        clock.advance_frames(1).expect("frame");
        emitted += 1;
        let condition_met = sample.time.to_f64() >= 0.5;
        if condition_met {
            break;
        }
    }
    assert_eq!(emitted, 15); // 15/30 = 0.5 triggers the stop
    assert_eq!(clock.now(), RationalTime::zero(30) + 75);
}

#[test]
fn terminal_sample_is_the_last_grid_point() {
    let clock = RationalFrameClock::new(30).unwrap();
    let segment = clock.segment(1.5).unwrap();
    let sample = segment.samples().last().unwrap();
    assert_eq!(sample.frame, 45);
    assert_eq!(sample.alpha, 1.0);
    assert_eq!(sample.time, segment.end_time());
    assert!(clock.segment(0.0).unwrap().samples().next().is_none());
}

#[test]
fn degenerate_and_refused_inputs() {
    assert_eq!(RationalFrameClock::new(0).err(), Some(ClockError::ZeroFps));
    let clock = RationalFrameClock::new(30).unwrap();
    assert_eq!(clock.segment(0.0).unwrap().n_frames(), 0);
    assert_eq!(clock.segment(-1.0).unwrap().n_frames(), 0);
    assert_eq!(
        clock.segment(f64::NAN).err(),
        Some(ClockError::NonFiniteRunTime)
    );
    assert_eq!(
        clock.segment(f64::INFINITY).err(),
        Some(ClockError::NonFiniteRunTime)
    );
    assert_eq!(clock.segment(1e18).err(), Some(ClockError::RunTimeTooLong));
    // A positive duration below one frame still takes one frame.
    assert_eq!(clock.segment(5e-324).unwrap().n_frames(), 1);
    assert_eq!(clock.segment(1e-9).unwrap().n_frames(), 1);
}

#[test]
fn frame_counter_bounds_are_typed_and_atomic() {
    let mut clock = RationalFrameClock::new(1).expect("fps");
    assert_eq!(
        clock.advance_frames(-1),
        Err(ClockError::NegativeFrameAdvance)
    );
    assert_eq!(clock.now().frames(), 0);

    clock
        .advance_frames(i64::MAX)
        .expect("the largest representable frame");
    assert_eq!(
        clock.advance_frames(1),
        Err(ClockError::FrameCounterOverflow)
    );
    assert_eq!(clock.now().frames(), i64::MAX, "refusal is atomic");
    assert_eq!(
        clock.segment(1.0).err(),
        Some(ClockError::FrameCounterOverflow),
        "planning also refuses before any animation state can mutate"
    );
}

// The no-off-grid property (D-18's permanent refusal, encoded in types):
// every time value reachable through the public API is a whole number of
// frames over fps. This test enumerates every constructor path.
#[test]
fn no_off_grid_time_is_reachable() {
    let mut clock = RationalFrameClock::new(24).unwrap();
    let times: Vec<RationalTime> = vec![
        clock.now(),
        clock.dt(),
        clock.segment(0.7).unwrap().end_time(),
        RationalTime::zero(24),
    ];
    clock.advance_frames(3).expect("frames");
    let mut all = times;
    all.push(clock.now());
    all.extend(clock.segment(0.7).unwrap().samples().map(|s| s.time));
    for t in all {
        // Integral frames over the clock's fps — by construction.
        assert_eq!(t.fps(), 24);
        let _integral: i64 = t.frames();
    }
}
