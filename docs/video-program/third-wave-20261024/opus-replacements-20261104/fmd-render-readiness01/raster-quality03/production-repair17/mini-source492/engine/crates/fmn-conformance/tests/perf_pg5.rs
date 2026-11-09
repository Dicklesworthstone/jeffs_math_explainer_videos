use fmn_conformance::perf::{
    Baseline, BenchmarkKey, Direction, Enforcement, GateId, GatePolicy, GateScope,
    MeasurementBatch, MetricUnit, Sample, Verdict,
};
use fmn_conformance::perf_pg5::{
    PG5_CORPUS_SCENES, PG5_SAMPLE_COUNT, PG5_SCENARIO, PG5_TRACE_SCHEMA, Pg5Definition,
    measure_pg5, pg5_identity,
};
use fmn_hash::sha256;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn policy() -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg5,
        scenario: PG5_SCENARIO.to_owned(),
        unit: MetricUnit::Mismatches,
        direction: Direction::Exactly,
        target: Some(0),
        min_valid_samples: PG5_SAMPLE_COUNT,
        max_invalid_samples: 0,
        max_mad_bps: 0,
        alert_regression_bps: 0,
        block_regression_bps: 0,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: false,
    }
}

fn key() -> BenchmarkKey {
    let definition = Pg5Definition::new().expect("fixed PG-5 definition");
    BenchmarkKey {
        profile_id: "pg5-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg5,
        scenario: PG5_SCENARIO.to_owned(),
        unit: MetricUnit::Mismatches,
        engine: pg5_identity().engine.name().to_owned(),
        tier: pg5_identity().tier.name().to_owned(),
        thread_profile: "matrix-1-4-16-frame-parallel-ordered-pipeline".to_owned(),
        execution_plan_digest: definition.execution_plan_digest(),
        config_digest: definition.config_digest(),
        cache_state: "independent-cold-scenes".to_owned(),
        output_mode: "scene-golden-anchored-documents".to_owned(),
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
    let definition = Pg5Definition::new().expect("fixed PG-5 definition");
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    definition
        .validate_baseline(&baseline)
        .expect("exact compiled identity");

    let mut wrong = baseline;
    wrong.key.execution_plan_digest = sha256(b"other-plan");
    let error = definition
        .validate_baseline(&wrong)
        .expect_err("plan drift must fail before corpus construction");
    assert!(error.to_string().contains("execution_plan_digest"));
}

#[test]
fn producer_refuses_bad_commit_before_profile_or_corpus_setup() {
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let error = measure_pg5(
        &baseline,
        "not-a-commit",
        "tests/artifacts/perf/pg5-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before profile or corpus setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn producer_refuses_bad_trace_path_before_profile_or_corpus_setup() {
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let error = measure_pg5(&baseline, COMMIT, "outside.tsv", None)
        .expect_err("trace path must fail before profile or corpus setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn one_injected_schedule_mismatch_blocks_through_the_common_verifier() {
    let baseline_batch = batch(vec![Sample::valid(0); PG5_SAMPLE_COUNT]);
    let baseline = Baseline::observed(
        1,
        policy(),
        &baseline_batch,
        "tests/artifacts/perf/pg5-thread-matrix/baseline.tsv",
    )
    .expect("observed exact-zero baseline");
    let report = baseline.evaluate(
        Some(&baseline_batch),
        &batch(vec![Sample::valid(0), Sample::valid(0), Sample::valid(1)]),
    );
    assert_eq!(report.verdict, Verdict::Block);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "target-miss")
    );
}

#[test]
#[ignore = "real certified corpus and two-team schedule; run explicitly under --profile release-perf"]
fn release_perf_producer_emits_three_zero_mismatch_schedule_samples() {
    let definition = Pg5Definition::new().expect("fixed PG-5 definition");
    let baseline = Baseline::targeted(1, policy(), key(), COMMIT).expect("target baseline");
    let artifacts = measure_pg5(
        &baseline,
        COMMIT,
        "tests/artifacts/perf/pg5-thread-matrix/trace.tsv",
        None,
    )
    .expect("release-perf PG-5 producer");

    assert_eq!(artifacts.cases.len(), PG5_CORPUS_SCENES);
    assert_eq!(artifacts.batch.samples.len(), PG5_SAMPLE_COUNT);
    assert!(
        artifacts
            .batch
            .samples
            .iter()
            .all(|sample| sample.invalid_reason.is_none() && sample.value == 0)
    );
    assert!(artifacts.cases.iter().all(|case| {
        case.four_threads == case.one_thread
            && case.sixteen_threads == case.one_thread
            && case.frame_parallel == case.one_thread
            && case.ordered_pipeline == case.one_thread
    }));
    assert_eq!(
        artifacts.reference_digest,
        definition
            .reference_digest(&artifacts.cases)
            .expect("reference aggregate recomputes from produced cases")
    );
    assert!(
        artifacts
            .trace_tsv
            .starts_with(&format!("schema\t{PG5_TRACE_SCHEMA}\n"))
    );
    assert_eq!(
        artifacts
            .trace_tsv
            .lines()
            .filter(|line| line.starts_with("scene\t"))
            .count(),
        PG5_CORPUS_SCENES
    );
    assert!(!artifacts.batch.key.bare_metal);
    assert!(!artifacts.batch.key.isolated);
    let raw = artifacts.batch.to_tsv().expect("canonical raw bundle");
    assert_eq!(
        MeasurementBatch::from_tsv(&raw).expect("raw bundle replay"),
        artifacts.batch
    );
}
