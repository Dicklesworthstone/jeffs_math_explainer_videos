use fmn_conformance::perf::{
    Baseline, BenchmarkKey, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GatePolicy,
    GateScope, MeasurementBatch, Sample, Verdict,
};
use fmn_conformance::perf_pg8::{
    PG8_FRAMES_PER_REPETITION, PG8_MAX_INVALID_SAMPLES, PG8_MIN_VALID_SAMPLES, PG8_MOBJECTS,
    PG8_RESULT_STATE_BYTES, PG8_SAMPLE_COUNT, PG8_TRACE_SCHEMA, PG8_WARMUP_ITERATIONS,
    Pg8Definition, Pg8Measurement, Pg8Observation, Pg8Scenario, assemble_pg8, measure_pg8,
};
use fmn_hash::sha256;
use manimlib::perf_harness::{self, Pg8Class};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

/// Published PG-8 class budgets (docs/performance/PERF_GATES.tsv). The
/// nanosecond targets cover one full repetition (60 frames, 64 mobjects).
fn target(scenario: Pg8Scenario) -> u64 {
    match scenario {
        Pg8Scenario::NativeBuiltins => 1_100_000,
        Pg8Scenario::PerFrameCallback => 108_999_999,
        Pg8Scenario::PointTransformCallback => 210_999_999,
        Pg8Scenario::DynamicSubclass => 160_999_999,
    }
}

fn policy(scenario: Pg8Scenario) -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg8,
        scenario: scenario.name().to_owned(),
        unit: scenario.unit(),
        direction: Direction::AtMost,
        target: Some(target(scenario)),
        min_valid_samples: PG8_MIN_VALID_SAMPLES,
        max_invalid_samples: PG8_MAX_INVALID_SAMPLES,
        max_mad_bps: 1_000,
        alert_regression_bps: 500,
        block_regression_bps: 1_000,
        enforcement: Enforcement::Blocking,
        scope: GateScope::PythonOnly,
        require_regression_profile: true,
    }
}

fn key(scenario: Pg8Scenario) -> BenchmarkKey {
    let definition = Pg8Definition::new(scenario);
    BenchmarkKey {
        profile_id: "pg8-shared-dev-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        // fm-inr.1's live attestation replaces these two placeholder
        // fingerprints; the suite lock is hashed honestly today.
        host_fingerprint: sha256(b"pg8-host-unattested"),
        toolchain_fingerprint: sha256(b"pg8-toolchain-unattested"),
        suite_lock_digest: sha256(include_str!("../../../SUITE.lock").as_bytes()),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg8,
        scenario: scenario.name().to_owned(),
        unit: scenario.unit(),
        engine: "fmn-python-bridge".to_owned(),
        tier: "portable".to_owned(),
        thread_profile: "single-thread".to_owned(),
        execution_plan_digest: sha256(b"pg8-single-thread-bridge-plan"),
        config_digest: definition.config_digest(),
        cache_state: "none".to_owned(),
        output_mode: "record-buffer-state".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn batch(scenario: Pg8Scenario, value: u64) -> MeasurementBatch {
    MeasurementBatch {
        key: key(scenario),
        producer_commit: COMMIT.to_owned(),
        samples: vec![Sample::valid(value); PG8_SAMPLE_COUNT],
        evidence: Vec::new(),
    }
}

fn class(scenario: Pg8Scenario) -> Pg8Class {
    match scenario {
        Pg8Scenario::NativeBuiltins => Pg8Class::NativeBuiltins,
        Pg8Scenario::PerFrameCallback => Pg8Class::PerFrameCallback,
        Pg8Scenario::PointTransformCallback => Pg8Class::PointTransformCallback,
        Pg8Scenario::DynamicSubclass => Pg8Class::DynamicSubclass,
    }
}

fn real_sampler(
    scenario: Pg8Scenario,
) -> Result<Pg8Measurement, fmn_conformance::perf_pg8::Pg8Error> {
    let run = perf_harness::measure(
        class(scenario),
        PG8_SAMPLE_COUNT,
        PG8_WARMUP_ITERATIONS,
        PG8_FRAMES_PER_REPETITION,
        PG8_MOBJECTS,
        1.0 / 30.0,
    )
    .map_err(fmn_conformance::perf_pg8::Pg8Error::Harness)?;
    Ok(Pg8Measurement {
        observations: run
            .repetitions
            .iter()
            .map(|repetition| Pg8Observation {
                elapsed_ns: repetition.elapsed_ns,
                reference_ns: repetition.reference_ns,
                invalid_reason: repetition.invalid_reason.clone(),
            })
            .collect(),
        result_state: run.result_state,
        reference_state: run.reference_state,
    })
}

fn intentionally_drifted_measurement() -> Pg8Measurement {
    Pg8Measurement {
        observations: (0..PG8_SAMPLE_COUNT)
            .map(|_| Pg8Observation {
                elapsed_ns: 1,
                reference_ns: Some(1),
                invalid_reason: None,
            })
            .collect(),
        result_state: b"intentionally drifted state".to_vec(),
        reference_state: None,
    }
}

#[test]
fn compiled_definitions_accept_only_their_exact_baseline_identity() {
    for scenario in Pg8Scenario::ALL {
        let definition = Pg8Definition::new(scenario);
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), COMMIT)
            .expect("target baseline");
        definition
            .validate_baseline(&baseline)
            .expect("exact compiled identity");

        let mut wrong = baseline;
        wrong.key.benchmark_definition = sha256(b"other-definition");
        let error = definition
            .validate_baseline(&wrong)
            .expect_err("definition drift must fail before timing");
        assert!(error.to_string().contains("benchmark_definition"));
    }
}

#[test]
fn producer_refuses_baseline_drift_before_profile_or_sampler() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let mut baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    baseline.key.benchmark_definition = sha256(b"other-definition");
    let sampler_called = std::cell::Cell::new(false);
    let sampler = |_| {
        sampler_called.set(true);
        Err(fmn_conformance::perf_pg8::Pg8Error::Harness(
            "sampler must not run".to_owned(),
        ))
    };
    let result = measure_pg8(
        &baseline,
        COMMIT,
        &sampler,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    );
    let error = result.expect_err("baseline drift must fail before timing");
    assert!(
        !sampler_called.get(),
        "sampler ran before identity preflight"
    );
    assert!(
        error.to_string().contains("benchmark_definition"),
        "{error}"
    );
}

#[test]
fn producer_refuses_bad_commit_before_profile_or_sampler() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let sampler_called = std::cell::Cell::new(false);
    let sampler = |_| {
        sampler_called.set(true);
        Err(fmn_conformance::perf_pg8::Pg8Error::Harness(
            "sampler must not run".to_owned(),
        ))
    };
    let error = measure_pg8(
        &baseline,
        "not-a-commit",
        &sampler,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before profile or sampling");
    assert!(!sampler_called.get(), "sampler ran before commit preflight");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn producer_refuses_bad_trace_path_before_profile_or_sampler() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let sampler_called = std::cell::Cell::new(false);
    let sampler = |_| {
        sampler_called.set(true);
        Err(fmn_conformance::perf_pg8::Pg8Error::Harness(
            "sampler must not run".to_owned(),
        ))
    };
    let error = measure_pg8(&baseline, COMMIT, &sampler, "outside.tsv", None)
        .expect_err("trace path must fail before profile or sampling");
    assert!(!sampler_called.get(), "sampler ran before path preflight");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn assembler_refuses_bad_commit_before_measurement_content() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = assemble_pg8(
        &baseline,
        "not-a-commit",
        &intentionally_drifted_measurement(),
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before measurement validation");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn assembler_refuses_bad_trace_path_before_measurement_content() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = assemble_pg8(
        &baseline,
        COMMIT,
        &intentionally_drifted_measurement(),
        "outside.tsv",
        None,
    )
    .expect_err("trace path must fail before measurement validation");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn assembler_refuses_wrong_state_size_before_hashing() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let mut measurement = intentionally_drifted_measurement();
    measurement.result_state.clear();
    let error = assemble_pg8(
        &baseline,
        COMMIT,
        &measurement,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("the fixed fixture state size must be checked before hashing");
    assert!(error.to_string().contains("result_state"), "{error}");
}

#[test]
fn assembler_refuses_bad_invalid_reason_before_state_hashing() {
    let scenario = Pg8Scenario::NativeBuiltins;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let mut measurement = intentionally_drifted_measurement();
    measurement.result_state.resize(PG8_RESULT_STATE_BYTES, 0);
    for observation in measurement.observations.iter_mut().take(1) {
        observation.invalid_reason = Some("x".repeat(1_025));
    }
    let error = assemble_pg8(
        &baseline,
        COMMIT,
        &measurement,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("invalid observation detail must be checked before state hashing");
    assert!(error.to_string().contains("invalid_reason"), "{error}");
}

#[test]
fn assembler_refuses_non_native_twin_timing_before_state_hashing() {
    let scenario = Pg8Scenario::DynamicSubclass;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let mut measurement = intentionally_drifted_measurement();
    measurement.result_state.resize(PG8_RESULT_STATE_BYTES, 0);
    measurement.reference_state = None;
    let error = assemble_pg8(
        &baseline,
        COMMIT,
        &measurement,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("non-native observations must not carry pure-Rust twin timings");
    assert!(
        error.to_string().contains("pure-Rust twin timing"),
        "{error}"
    );
}

#[test]
fn assembler_refuses_wrong_twin_state_shape_before_state_hashing() {
    let native = Pg8Scenario::NativeBuiltins;
    let native_baseline =
        Baseline::targeted(1, policy(native), key(native), COMMIT).expect("target baseline");
    let mut native_measurement = intentionally_drifted_measurement();
    native_measurement
        .result_state
        .resize(PG8_RESULT_STATE_BYTES, 0);
    native_measurement.reference_state = Some(Vec::new());
    let native_error = assemble_pg8(
        &native_baseline,
        COMMIT,
        &native_measurement,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("the native twin state size must be checked before state hashing");
    assert!(
        native_error.to_string().contains("reference_state"),
        "{native_error}"
    );

    let callback = Pg8Scenario::DynamicSubclass;
    let callback_baseline =
        Baseline::targeted(1, policy(callback), key(callback), COMMIT).expect("target baseline");
    let mut callback_measurement = intentionally_drifted_measurement();
    callback_measurement
        .result_state
        .resize(PG8_RESULT_STATE_BYTES, 0);
    callback_measurement
        .observations
        .iter_mut()
        .for_each(|observation| observation.reference_ns = None);
    callback_measurement.reference_state = Some(vec![0; PG8_RESULT_STATE_BYTES]);
    let callback_error = assemble_pg8(
        &callback_baseline,
        COMMIT,
        &callback_measurement,
        "tests/artifacts/perf/pg8-preflight/trace.tsv",
        None,
    )
    .expect_err("callback scenarios must not carry a pure-Rust twin state");
    assert!(
        callback_error
            .to_string()
            .contains("must not carry a pure-Rust twin state"),
        "{callback_error}"
    );
}

#[test]
fn injected_pg8_slowdown_blocks_through_the_common_verifier() {
    for scenario in Pg8Scenario::ALL {
        let baseline_batch = batch(scenario, target(scenario) / 2);
        let baseline = Baseline::observed(
            1,
            policy(scenario),
            &baseline_batch,
            format!("tests/artifacts/perf/pg8-{scenario}/baseline.tsv"),
        )
        .expect("observed baseline");

        let mut slowed = batch(scenario, target(scenario) * 2);
        slowed.evidence.push(
            EvidenceRef::from_bytes(
                EvidenceKind::CpuProfile,
                format!("tests/artifacts/perf/pg8-{scenario}/slowdown.pb"),
                b"injected profile",
            )
            .expect("profile evidence"),
        );
        let report = baseline.evaluate(Some(&baseline_batch), &slowed);
        assert_eq!(report.verdict, Verdict::Block, "{scenario}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "baseline-regression"),
            "{scenario}"
        );
    }
}

#[test]
fn bridge_harness_smoke_runs_every_class_deterministically() {
    for scenario in Pg8Scenario::ALL {
        let first = perf_harness::measure(class(scenario), 1, 1, 4, 3, 1.0 / 30.0)
            .expect("first smoke run");
        let second = perf_harness::measure(class(scenario), 1, 1, 4, 3, 1.0 / 30.0)
            .expect("second smoke run");
        assert_eq!(first.repetitions.len(), 1, "{scenario}");
        assert!(!first.result_state.is_empty(), "{scenario}");
        assert_eq!(
            first.result_state, second.result_state,
            "{scenario} state must be bit-deterministic across runs"
        );
        if scenario == Pg8Scenario::NativeBuiltins {
            assert_eq!(
                first.reference_state.as_ref(),
                Some(&first.result_state),
                "pure-Rust twin must end bit-identical to the bridge"
            );
            assert!(first.repetitions[0].reference_ns.is_some());
        } else {
            assert_eq!(first.reference_state, None, "{scenario}");
        }
    }
}

/// Run the full canonical sample plan in the dev profile and require the
/// locked state self-goldens. The workload's arithmetic is bit-exact by
/// construction (binary64 declared ops, no transcendentals), so dev and
/// release-perf artifacts must produce identical states; only the timing
/// evidence requires the release-perf probe.
#[test]
fn canonical_plan_reproduces_the_locked_state_goldens() {
    for scenario in Pg8Scenario::ALL {
        let measurement = real_sampler(scenario).expect("canonical plan measurement");
        let digest = sha256(&measurement.result_state);
        assert_eq!(
            digest,
            Pg8Definition::new(scenario).expected_result_digest(),
            "{scenario} state drifted"
        );
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), COMMIT)
            .expect("target baseline");
        let directory = format!("tests/artifacts/perf/pg8-{scenario}");
        let artifacts = assemble_pg8(
            &baseline,
            COMMIT,
            &measurement,
            format!("{directory}/trace.tsv"),
            None,
        )
        .expect("dev-profile assembly");
        let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
        assert_eq!(
            MeasurementBatch::from_tsv(&raw).expect("replayable raw bundle"),
            artifacts.batch
        );
    }
}

/// One committed PG-8 observation bundle and its locked statistics.
struct CommittedBundle {
    scenario: Pg8Scenario,
    raw: &'static str,
    bundle_sha256: &'static str,
    median: u64,
    min: u64,
    max: u64,
    mad_bps: u32,
}

/// The recorded PG-8 baseline (fm-zoi): measured on this host under the
/// release-perf profile by `release_perf_producer_records_replayable_pg8_baseline`.
/// Keys are honestly unqualified (`bare_metal = false`, `isolated = false`),
/// so these bundles are calibration evidence — fm-inr.1's attestation is
/// what turns a pinned-host rerun into gate evidence.
const COMMITTED_BUNDLES: &[CommittedBundle] = &[
    CommittedBundle {
        scenario: Pg8Scenario::NativeBuiltins,
        raw: include_str!("artifacts/perf/pg8-native-builtins/samples.tsv"),
        bundle_sha256: "375b939dd3968268b367f7d900debcbdba88e0491bcf0ebca104ff558f3c95ac",
        median: 3_545_767,
        min: 3_422_458,
        max: 3_857_539,
        mad_bps: 86,
    },
    CommittedBundle {
        scenario: Pg8Scenario::PerFrameCallback,
        raw: include_str!("artifacts/perf/pg8-per-frame-callback/samples.tsv"),
        bundle_sha256: "78fe983b61944ceca9c4dbb5aa4e63aa9ee39983f58955032680589fe709abd2",
        median: 72_517_928,
        min: 71_527_241,
        max: 77_146_900,
        mad_bps: 37,
    },
    CommittedBundle {
        scenario: Pg8Scenario::PointTransformCallback,
        raw: include_str!("artifacts/perf/pg8-point-transform-callback/samples.tsv"),
        bundle_sha256: "36cf9c181f9fc1e36e9fc7040e4df7277ed74182e440707483a619cb64d42ef9",
        median: 143_938_670,
        min: 142_433_283,
        max: 149_853_479,
        mad_bps: 49,
    },
    CommittedBundle {
        scenario: Pg8Scenario::DynamicSubclass,
        raw: include_str!("artifacts/perf/pg8-dynamic-subclass/samples.tsv"),
        bundle_sha256: "ddbf04f085598e7a5af2748c4960cc5299404752c7d645821c02e82a2016d6aa",
        median: 106_000_320,
        min: 104_711_065,
        max: 108_682_121,
        mad_bps: 24,
    },
];

#[test]
fn committed_pg8_baseline_bundles_replay_through_the_verifier() {
    for bundle in COMMITTED_BUNDLES {
        let scenario = bundle.scenario;
        assert_eq!(
            sha256(bundle.raw.as_bytes()).to_string(),
            bundle.bundle_sha256,
            "{scenario} committed bundle drifted"
        );
        let batch = MeasurementBatch::from_tsv(bundle.raw).expect("committed bundle parses");
        assert_eq!(batch.samples.len(), PG8_SAMPLE_COUNT, "{scenario}");
        assert!(
            batch
                .samples
                .iter()
                .all(|sample| sample.invalid_reason.is_none()),
            "{scenario} committed run had no host-quality failures"
        );
        let stats =
            fmn_conformance::perf::RobustStats::from_samples(&batch.samples, PG8_MIN_VALID_SAMPLES)
                .expect("committed statistics");
        assert_eq!(stats.median, bundle.median, "{scenario}");
        assert_eq!(stats.min, bundle.min, "{scenario}");
        assert_eq!(stats.max, bundle.max, "{scenario}");
        assert_eq!(stats.mad_bps, bundle.mad_bps, "{scenario}");
        assert_eq!(stats.valid_samples, PG8_SAMPLE_COUNT);
        // The host is unqualified by construction: self-evaluation is
        // inconclusive, never a pass.
        let baseline = Baseline::targeted(
            1,
            policy(scenario),
            batch.key.clone(),
            batch.producer_commit.clone(),
        )
        .expect("target baseline");
        let report = baseline.evaluate(Some(&batch), &batch);
        assert_eq!(report.verdict, Verdict::Inconclusive, "{scenario}");
        // The recorded nanosecond classes sit under their published budgets.
        if scenario != Pg8Scenario::NativeBuiltins {
            assert!(
                bundle.median <= target(scenario),
                "{scenario} observation exceeds its published budget"
            );
        }
    }
}

fn producer_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().to_owned())
        .filter(|revision| revision.len() == 40)
        .unwrap_or_else(|| COMMIT.to_owned())
}

#[test]
#[ignore = "real timing probe; run explicitly under --profile release-perf"]
fn release_perf_producer_records_replayable_pg8_baseline() {
    let commit = producer_commit();
    for scenario in Pg8Scenario::ALL {
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), &commit)
            .expect("target baseline");
        let directory = format!("tests/artifacts/perf/pg8-{scenario}");
        std::fs::create_dir_all(&directory).expect("artifact directory");
        let artifacts = measure_pg8(
            &baseline,
            &commit,
            &real_sampler,
            format!("{directory}/trace.tsv"),
            None,
        )
        .expect("release-perf measurement");
        assert_eq!(artifacts.batch.samples.len(), PG8_SAMPLE_COUNT);
        let valid_samples = artifacts
            .batch
            .samples
            .iter()
            .filter(|sample| sample.invalid_reason.is_none())
            .count();
        assert!(valid_samples >= PG8_MIN_VALID_SAMPLES, "{scenario}");
        assert!(
            artifacts.batch.samples.len() - valid_samples <= PG8_MAX_INVALID_SAMPLES,
            "{scenario}"
        );
        assert!(!artifacts.batch.key.bare_metal);
        assert!(!artifacts.batch.key.isolated);
        assert_eq!(
            artifacts.result_digest,
            Pg8Definition::new(scenario).expected_result_digest()
        );
        assert!(artifacts.trace_tsv.starts_with(&format!(
            "schema\t{PG8_TRACE_SCHEMA}\ngate\tpg-8\nscenario\t{scenario}\n"
        )));
        assert_eq!(artifacts.batch.evidence.len(), 1);
        assert_eq!(
            artifacts.batch.evidence[0].digest,
            sha256(artifacts.trace_tsv.as_bytes())
        );
        let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
        assert_eq!(
            MeasurementBatch::from_tsv(&raw).expect("replayable raw bundle"),
            artifacts.batch
        );
        std::fs::write(format!("{directory}/samples.tsv"), &raw).expect("record raw bundle");
        std::fs::write(format!("{directory}/trace.tsv"), &artifacts.trace_tsv)
            .expect("record trace");
        let baseline_tsv = baseline.to_tsv().expect("canonical baseline");
        std::fs::write(format!("{directory}/baseline.tsv"), baseline_tsv).expect("record baseline");
    }
}
