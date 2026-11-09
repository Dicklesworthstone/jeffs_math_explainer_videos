use fmn_conformance::perf::{
    Baseline, BenchmarkKey, Direction, Enforcement, GateId, GatePolicy, GateScope,
    MeasurementBatch, MetricUnit, Sample, Verdict,
};
use fmn_conformance::perf_pg6::{
    PG6_MAX_INVALID_SAMPLES, PG6_MIN_VALID_SAMPLES, PG6_SAMPLE_COUNT, PG6_SCENARIO,
    PG6_SOAK_SCENARIO, PG6_SOAK_UNSUPPORTED_REASON, PG6_SOAK_WINDOWS, PG6_THREADS,
    PG6_TRACE_SCHEMA, Pg6Definition, Pg6SoakDefinition, measure_pg6, measure_pg6_soak,
    pg6_identity,
};
use fmn_conformance::perf_pg6_peak::{
    PG6_PEAK_MAX_INVALID_SAMPLES, PG6_PEAK_MIN_VALID_SAMPLES, PG6_PEAK_SAMPLE_COUNT,
    PG6_PEAK_SCENARIO, PG6_PEAK_TARGET_BYTES, PG6_PEAK_TRACE_SCHEMA, PG6_PEAK_UNSUPPORTED_REASON,
    Pg6PeakDefinition, measure_pg6_peak,
};
use fmn_hash::sha256;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn policy() -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg6,
        scenario: PG6_SCENARIO.to_owned(),
        unit: MetricUnit::Allocations,
        direction: Direction::Exactly,
        target: Some(0),
        min_valid_samples: PG6_MIN_VALID_SAMPLES,
        max_invalid_samples: PG6_MAX_INVALID_SAMPLES,
        max_mad_bps: 0,
        alert_regression_bps: 0,
        block_regression_bps: 0,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: false,
    }
}

fn key() -> BenchmarkKey {
    let definition = Pg6Definition::new();
    BenchmarkKey {
        profile_id: "pg6-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg6,
        scenario: PG6_SCENARIO.to_owned(),
        unit: MetricUnit::Allocations,
        engine: pg6_identity().engine.name().to_owned(),
        tier: pg6_identity().tier.name().to_owned(),
        thread_profile: "fixed-4".to_owned(),
        execution_plan_digest: sha256(b"execution-plan"),
        config_digest: definition.config_digest(),
        cache_state: "warm-reused-frame-arena".to_owned(),
        output_mode: "scene-golden-anchored".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn batch(samples: Vec<Sample>) -> MeasurementBatch {
    MeasurementBatch {
        key: key(),
        producer_commit: COMMIT.to_owned(),
        samples,
        evidence: Vec::new(),
    }
}

#[test]
fn compiled_definition_accepts_only_its_exact_baseline_identity() {
    let definition = Pg6Definition::new();
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    definition
        .validate_baseline(&baseline)
        .expect("exact compiled identity");

    let mut wrong = baseline;
    wrong.key.benchmark_definition = sha256(b"other-definition");
    let error = definition
        .validate_baseline(&wrong)
        .expect_err("definition drift must fail before corpus construction");
    assert!(error.to_string().contains("benchmark_definition"));
}

#[test]
fn producer_refuses_bad_commit_before_profile_or_corpus_setup() {
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let error = measure_pg6(
        &baseline,
        "not-a-commit",
        "tests/artifacts/perf/pg6-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before profile or corpus setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn producer_refuses_bad_trace_path_before_profile_or_corpus_setup() {
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let error = measure_pg6(&baseline, COMMIT, "outside.tsv", None)
        .expect_err("trace path must fail before profile or corpus setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn one_injected_allocation_blocks_through_the_common_verifier() {
    let baseline_batch = batch(vec![Sample::valid(0); PG6_SAMPLE_COUNT]);
    let baseline = Baseline::observed(
        1,
        policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg6-steady-allocations/baseline.tsv",
    )
    .expect("observed exact-zero baseline");
    let mut samples = vec![Sample::valid(0); PG6_SAMPLE_COUNT];
    samples[PG6_SAMPLE_COUNT - 1] = Sample::valid(1);
    let report = baseline.evaluate(Some(&baseline_batch), &batch(samples));
    assert_eq!(report.verdict, Verdict::Block);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "target-miss")
    );
}

#[test]
#[ignore = "real four-thread corpus render; run explicitly under --profile release-perf"]
fn release_perf_producer_emits_every_zero_allocation_scene_and_replayable_trace() {
    let definition = Pg6Definition::new();
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let artifacts = measure_pg6(
        &baseline,
        COMMIT,
        "tests/artifacts/perf/pg6-steady-allocations/trace.tsv",
        None,
    )
    .expect("release-perf corpus producer");

    assert_eq!(artifacts.cases.len(), PG6_SAMPLE_COUNT);
    assert_eq!(artifacts.batch.samples.len(), PG6_SAMPLE_COUNT);
    assert!(
        artifacts
            .batch
            .samples
            .iter()
            .all(|sample| sample.invalid_reason.is_none() && sample.value == 0)
    );
    assert!(artifacts.cases.iter().all(|case| {
        case.warm_frame_digest == case.measured_frame_digest
            && case.warm.heap_allocs_this_frame > 0
            && case.measured.heap_allocs_this_frame == 0
            && case.warm.arena_buffer_bytes == case.measured.arena_buffer_bytes
            && case.warm.pool_slots == case.measured.pool_slots
            && case.measured.pool_slots == PG6_THREADS
    }));
    assert_eq!(
        artifacts.result_digest,
        definition
            .result_digest(&artifacts.cases)
            .expect("result aggregate recomputes from produced cases")
    );
    assert!(
        artifacts
            .trace_tsv
            .starts_with(&format!("schema\t{PG6_TRACE_SCHEMA}\n"))
    );
    assert_eq!(
        artifacts
            .trace_tsv
            .lines()
            .filter(|line| line.starts_with("scene\t"))
            .count(),
        PG6_SAMPLE_COUNT
    );
    assert!(!artifacts.batch.key.bare_metal);
    assert!(!artifacts.batch.key.isolated);
    let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
    assert_eq!(
        MeasurementBatch::from_tsv(&raw).expect("raw bundle replay"),
        artifacts.batch
    );
}

// ---------------------------------------------------------------------------
// The leak-soak surface (`one-hour-soak-leak`)
// ---------------------------------------------------------------------------

fn soak_policy() -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg6,
        scenario: PG6_SOAK_SCENARIO.to_owned(),
        unit: MetricUnit::LeakedBytes,
        direction: Direction::Exactly,
        target: Some(0),
        min_valid_samples: PG6_SOAK_WINDOWS,
        max_invalid_samples: 0,
        max_mad_bps: 0,
        alert_regression_bps: 0,
        block_regression_bps: 0,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: true,
    }
}

fn soak_definition() -> Pg6SoakDefinition {
    Pg6SoakDefinition::new(2).expect("nonzero soak window")
}

fn soak_key() -> BenchmarkKey {
    let definition = soak_definition();
    BenchmarkKey {
        profile_id: "pg6-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg6,
        scenario: PG6_SOAK_SCENARIO.to_owned(),
        unit: MetricUnit::LeakedBytes,
        engine: pg6_identity().engine.name().to_owned(),
        tier: pg6_identity().tier.name().to_owned(),
        thread_profile: "fixed-4".to_owned(),
        execution_plan_digest: sha256(b"execution-plan"),
        config_digest: definition.config_digest(),
        cache_state: "warm-reused-frame-arena-soak".to_owned(),
        output_mode: "scene-golden-anchored".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn soak_batch(samples: Vec<Sample>) -> MeasurementBatch {
    MeasurementBatch {
        key: soak_key(),
        producer_commit: COMMIT.to_owned(),
        samples,
        evidence: Vec::new(),
    }
}

#[test]
fn soak_definition_accepts_only_its_exact_baseline_identity() {
    let definition = soak_definition();
    let baseline =
        Baseline::targeted(1, soak_policy(), soak_key(), COMMIT).expect("target baseline");
    definition
        .validate_baseline(&baseline)
        .expect("exact compiled identity");

    // A different window length is a different benchmark.
    let mut wrong_window = baseline.clone();
    wrong_window.key.benchmark_definition =
        Pg6SoakDefinition::new(3).expect("other window").digest();
    let error = definition
        .validate_baseline(&wrong_window)
        .expect_err("window-length drift must fail before the workload");
    assert!(error.to_string().contains("benchmark_definition"));

    // The steady-allocation scenario's identity is never accepted here.
    let mut wrong_scenario = baseline;
    wrong_scenario.policy.scenario = PG6_SCENARIO.to_owned();
    wrong_scenario.key.scenario = PG6_SCENARIO.to_owned();
    wrong_scenario.policy.unit = MetricUnit::Allocations;
    wrong_scenario.key.unit = MetricUnit::Allocations;
    assert!(definition.validate_baseline(&wrong_scenario).is_err());
}

#[test]
fn one_injected_leak_blocks_through_the_common_verifier() {
    let baseline_batch = soak_batch(vec![Sample::valid(0); PG6_SOAK_WINDOWS]);
    let baseline = Baseline::observed(
        1,
        soak_policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg6-soak/baseline.tsv",
    )
    .expect("observed exact-zero baseline");
    let mut samples = vec![Sample::valid(0); PG6_SOAK_WINDOWS];
    samples[PG6_SOAK_WINDOWS - 1] = Sample::valid(4096);
    let report = baseline.evaluate(Some(&baseline_batch), &soak_batch(samples));
    assert_eq!(report.verdict, Verdict::Block);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "target-miss")
    );
}

#[test]
fn unsupported_host_samples_are_retained_and_never_pass() {
    let baseline_batch = soak_batch(vec![Sample::valid(0); PG6_SOAK_WINDOWS]);
    let baseline = Baseline::observed(
        1,
        soak_policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg6-soak/baseline.tsv",
    )
    .expect("observed exact-zero baseline");
    let inconclusive = soak_batch(vec![
        Sample::invalid(0, PG6_SOAK_UNSUPPORTED_REASON);
        PG6_SOAK_WINDOWS
    ]);
    let report = baseline.evaluate(Some(&baseline_batch), &inconclusive);
    assert_ne!(
        report.verdict,
        Verdict::Pass,
        "a host that cannot attribute residency can never pass the gate"
    );
}

#[test]
fn soak_producer_refuses_bad_identity_before_profile_or_corpus_setup() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let baseline =
        Baseline::targeted(1, soak_policy(), soak_key(), COMMIT).expect("target baseline");
    let probe_called = AtomicBool::new(false);
    let mut deny_probe = || -> Result<Option<u64>, String> {
        probe_called.store(true, Ordering::SeqCst);
        Ok(None)
    };

    let error = measure_pg6_soak(
        &baseline,
        "not-a-commit",
        "tests/artifacts/perf/pg6-soak/trace.tsv",
        soak_definition(),
        &mut deny_probe,
        None,
    )
    .expect_err("producer commit must fail before profile or corpus setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");

    let error = measure_pg6_soak(
        &baseline,
        COMMIT,
        "outside.tsv",
        soak_definition(),
        &mut deny_probe,
        None,
    )
    .expect_err("trace path must fail before profile or corpus setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
    assert!(!probe_called.load(Ordering::SeqCst));
}

#[test]
#[ignore = "real corpus soak with the live RSS probe; run explicitly under --profile release-perf"]
fn release_perf_soak_measures_three_flat_windows_and_a_deliberate_leak_blocks() {
    use fmn_platform::fs::StdFs;
    use fmn_platform::topology::current_rss_bytes;

    let definition = soak_definition();
    let baseline =
        Baseline::targeted(1, soak_policy(), soak_key(), COMMIT).expect("target baseline");

    let mut real_probe = || current_rss_bytes(&StdFs).map_err(|error| error.to_string());
    let artifacts = measure_pg6_soak(
        &baseline,
        COMMIT,
        "tests/artifacts/perf/pg6-soak/trace.tsv",
        definition,
        &mut real_probe,
        None,
    )
    .expect("release-perf soak producer");
    assert_eq!(artifacts.windows.len(), PG6_SOAK_WINDOWS);
    assert_eq!(artifacts.batch.samples.len(), PG6_SOAK_WINDOWS);
    assert!(
        artifacts
            .batch
            .samples
            .iter()
            .all(|sample| sample.invalid_reason.is_none())
    );

    // A deliberate ~64 MiB touched leak per probe must be observed. The
    // optimizer may legally elide an unobserved heap allocation — the first
    // version of this test proved that under release-perf — so the pointer is
    // laundered through `black_box` and every page is touched before the
    // buffer is forgotten.
    let mut leak_probe = || -> Result<Option<u64>, String> {
        let mut leak = vec![0u8; 64 << 20];
        for byte in leak.iter_mut().step_by(4096) {
            *byte = 1;
        }
        std::hint::black_box(leak.as_ptr());
        // ubs:ignore -- the test deliberately creates a process-lifetime leak
        // so the live RSS capability must observe it.
        std::mem::forget(leak);
        current_rss_bytes(&StdFs).map_err(|error| error.to_string())
    };
    let leaked = measure_pg6_soak(
        &baseline,
        COMMIT,
        "tests/artifacts/perf/pg6-soak/leak-trace.tsv",
        definition,
        &mut leak_probe,
        None,
    )
    .expect("leaking soak still measures");
    assert!(
        leaked
            .batch
            .samples
            .iter()
            .any(|sample| sample.value > 32 << 20),
        "the injected leak must dominate the window deltas: {:?}",
        leaked.windows
    );
}

// ---------------------------------------------------------------------------
// The 4K 3D peak-residency surface (`gallery-4k-3d-peak`)
// ---------------------------------------------------------------------------

fn peak_policy() -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg6,
        scenario: PG6_PEAK_SCENARIO.to_owned(),
        unit: MetricUnit::Bytes,
        direction: Direction::AtMost,
        target: Some(PG6_PEAK_TARGET_BYTES),
        min_valid_samples: PG6_PEAK_MIN_VALID_SAMPLES,
        max_invalid_samples: PG6_PEAK_MAX_INVALID_SAMPLES,
        max_mad_bps: 1000,
        alert_regression_bps: 500,
        block_regression_bps: 1000,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: true,
    }
}

fn peak_key() -> BenchmarkKey {
    let definition = Pg6PeakDefinition::new();
    BenchmarkKey {
        profile_id: "pg6-peak-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg6,
        scenario: PG6_PEAK_SCENARIO.to_owned(),
        unit: MetricUnit::Bytes,
        engine: pg6_identity().engine.name().to_owned(),
        tier: pg6_identity().tier.name().to_owned(),
        thread_profile: "fixed-4-plus-rss-sampler".to_owned(),
        execution_plan_digest: sha256(b"execution-plan"),
        config_digest: definition.config_digest(),
        cache_state: "cold-gallery-pass".to_owned(),
        output_mode: "raw-rgba16f".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn peak_batch(samples: Vec<Sample>) -> MeasurementBatch {
    MeasurementBatch {
        key: peak_key(),
        producer_commit: COMMIT.to_owned(),
        samples,
        evidence: Vec::new(),
    }
}

#[test]
fn peak_definition_accepts_only_its_exact_baseline_identity() {
    let definition = Pg6PeakDefinition::new();
    let baseline =
        Baseline::targeted(1, peak_policy(), peak_key(), COMMIT).expect("target baseline");
    definition
        .validate_baseline(&baseline)
        .expect("exact compiled identity");

    let mut wrong = baseline;
    wrong.key.thread_profile = "fixed-4".to_owned();
    let error = definition
        .validate_baseline(&wrong)
        .expect_err("a sampler-free identity must fail before rendering");
    assert!(error.to_string().contains("thread_profile"));
}

#[test]
fn injected_gallery_peak_blocks_through_the_common_verifier() {
    let ordinary = 256 * 1024 * 1024;
    let baseline_batch = peak_batch(vec![Sample::valid(ordinary); PG6_PEAK_SAMPLE_COUNT]);
    let baseline = Baseline::observed(
        1,
        peak_policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg6-gallery-peak/baseline.tsv",
    )
    .expect("observed peak baseline");
    let regression = peak_batch(vec![
        Sample::valid(PG6_PEAK_TARGET_BYTES + 1);
        PG6_PEAK_SAMPLE_COUNT
    ]);
    let report = baseline.evaluate(Some(&baseline_batch), &regression);
    assert_eq!(report.verdict, Verdict::Block);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "target-miss")
    );
}

#[test]
fn peak_unsupported_host_samples_are_retained_and_never_pass() {
    let baseline_batch = peak_batch(vec![Sample::valid(256 << 20); PG6_PEAK_SAMPLE_COUNT]);
    let baseline = Baseline::observed(
        1,
        peak_policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg6-gallery-peak/baseline.tsv",
    )
    .expect("observed peak baseline");
    let inconclusive = peak_batch(vec![
        Sample::invalid(0, PG6_PEAK_UNSUPPORTED_REASON);
        PG6_PEAK_SAMPLE_COUNT
    ]);
    let report = baseline.evaluate(Some(&baseline_batch), &inconclusive);
    assert_ne!(report.verdict, Verdict::Pass);
}

#[test]
fn peak_producer_refuses_bad_identity_before_profile_corpus_or_probe() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let baseline =
        Baseline::targeted(1, peak_policy(), peak_key(), COMMIT).expect("target baseline");
    let probe_called = AtomicBool::new(false);
    let deny_probe = || -> Result<Option<u64>, String> {
        probe_called.store(true, Ordering::SeqCst);
        Ok(None)
    };
    let error = measure_pg6_peak(
        &baseline,
        "not-a-commit",
        "tests/artifacts/perf/pg6-gallery-peak/trace.tsv",
        &deny_probe,
        None,
    )
    .expect_err("producer commit must fail before measurement setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");

    let error = measure_pg6_peak(&baseline, COMMIT, "outside.tsv", &deny_probe, None)
        .expect_err("trace path must fail before measurement setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
    assert!(!probe_called.load(Ordering::SeqCst));
}

#[test]
#[ignore = "eleven real three-frame UHD passes; run explicitly under --profile release-perf"]
fn release_perf_peak_producer_renders_every_sample_and_replays_raw_evidence() {
    use fmn_platform::fs::StdFs;
    use fmn_platform::topology::current_rss_bytes;

    let baseline =
        Baseline::targeted(1, peak_policy(), peak_key(), COMMIT).expect("target baseline");
    let probe = || current_rss_bytes(&StdFs).map_err(|error| error.to_string());
    let artifacts = measure_pg6_peak(
        &baseline,
        COMMIT,
        "tests/artifacts/perf/pg6-gallery-peak/trace.tsv",
        &probe,
        None,
    )
    .expect("release-perf 4K producer");
    assert_eq!(artifacts.passes.len(), PG6_PEAK_SAMPLE_COUNT);
    assert_eq!(artifacts.batch.samples.len(), PG6_PEAK_SAMPLE_COUNT);
    assert!(
        artifacts
            .batch
            .samples
            .iter()
            .all(|sample| { sample.invalid_reason.is_none() && sample.value > 64 * 1024 * 1024 })
    );
    assert!(
        artifacts
            .trace_tsv
            .starts_with(&format!("schema\t{PG6_PEAK_TRACE_SCHEMA}\n"))
    );
    assert_eq!(
        artifacts
            .trace_tsv
            .lines()
            .filter(|line| line.starts_with("frame\t"))
            .count(),
        PG6_PEAK_SAMPLE_COUNT * 3,
    );
    assert!(!artifacts.batch.key.bare_metal);
    assert!(!artifacts.batch.key.isolated);
    let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
    assert_eq!(
        MeasurementBatch::from_tsv(&raw).expect("raw bundle replay"),
        artifacts.batch,
    );
}
