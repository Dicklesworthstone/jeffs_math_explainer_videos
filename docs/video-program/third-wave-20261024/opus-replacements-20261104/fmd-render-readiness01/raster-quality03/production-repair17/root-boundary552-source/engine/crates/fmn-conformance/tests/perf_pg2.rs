use fmn_conformance::perf::{
    Baseline, BenchmarkKey, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GatePolicy,
    GateScope, MeasurementBatch, MetricUnit, Sample, Verdict,
};
use fmn_conformance::perf_pg2::{
    PG2_MAX_INVALID_SAMPLES, PG2_MIN_VALID_SAMPLES, PG2_SAMPLE_COUNT, PG2_TRACE_SCHEMA,
    Pg2Definition, Pg2Scenario, measure_pg2,
};
use fmn_hash::sha256;
use fmn_render::{EngineIdentity, Tier};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn policy(scenario: Pg2Scenario) -> GatePolicy {
    GatePolicy {
        gate: GateId::Pg2,
        scenario: scenario.name().to_owned(),
        unit: MetricUnit::MegaPixelsPerSecondMilli,
        direction: Direction::AtLeast,
        target: Some(match scenario {
            Pg2Scenario::FillCanonical => 300_000,
            Pg2Scenario::StrokeCanonical => 120_000,
        }),
        min_valid_samples: PG2_MIN_VALID_SAMPLES,
        max_invalid_samples: PG2_MAX_INVALID_SAMPLES,
        max_mad_bps: 1_000,
        alert_regression_bps: 500,
        block_regression_bps: 1_000,
        enforcement: Enforcement::Blocking,
        scope: GateScope::Core,
        require_regression_profile: true,
    }
}

fn key(scenario: Pg2Scenario) -> BenchmarkKey {
    let definition = Pg2Definition::new(scenario);
    BenchmarkKey {
        profile_id: "pg2-test-host".to_owned(),
        build_profile: "release-perf".to_owned(),
        host_fingerprint: sha256(b"host"),
        toolchain_fingerprint: sha256(b"toolchain"),
        suite_lock_digest: sha256(b"suite-lock"),
        benchmark_definition: definition.digest(),
        gate: GateId::Pg2,
        scenario: scenario.name().to_owned(),
        unit: MetricUnit::MegaPixelsPerSecondMilli,
        engine: EngineIdentity::fast().engine.name().to_owned(),
        tier: Tier::COMPILED.name().to_owned(),
        thread_profile: "fixed-8".to_owned(),
        execution_plan_digest: sha256(b"execution-plan"),
        config_digest: definition.config_digest(),
        cache_state: "warm".to_owned(),
        output_mode: "raw-rgba16f".to_owned(),
        external_tool_fingerprint: None,
        bare_metal: true,
        isolated: true,
    }
}

fn batch(scenario: Pg2Scenario, value: u64) -> MeasurementBatch {
    MeasurementBatch {
        key: key(scenario),
        producer_commit: COMMIT.to_owned(),
        samples: vec![Sample::valid(value); PG2_SAMPLE_COUNT],
        evidence: Vec::new(),
    }
}

#[test]
fn compiled_definition_accepts_only_its_exact_baseline_identity() {
    for scenario in Pg2Scenario::ALL {
        let definition = Pg2Definition::new(scenario);
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
fn producer_refuses_bad_commit_before_profile_or_workload() {
    let scenario = Pg2Scenario::FillCanonical;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = measure_pg2(
        &baseline,
        "not-a-commit",
        "tests/artifacts/perf/pg2-preflight/trace.tsv",
        None,
    )
    .expect_err("producer commit must fail before profile or workload setup");
    assert!(error.to_string().contains("producer_commit"), "{error}");
}

#[test]
fn producer_refuses_bad_trace_path_before_profile_or_workload() {
    let scenario = Pg2Scenario::FillCanonical;
    let baseline =
        Baseline::targeted(1, policy(scenario), key(scenario), COMMIT).expect("target baseline");
    let error = measure_pg2(&baseline, COMMIT, "outside.tsv", None)
        .expect_err("trace path must fail before profile or workload setup");
    assert!(error.to_string().contains("artifact path"), "{error}");
}

#[test]
fn injected_pg2_slowdown_blocks_through_the_common_verifier() {
    for scenario in Pg2Scenario::ALL {
        let baseline_batch = batch(scenario, 400_000);
        let baseline = Baseline::observed(
            1,
            policy(scenario),
            &baseline_batch,
            format!("tests/artifacts/perf/pg2-{scenario}/baseline.tsv"),
        )
        .expect("observed baseline");

        let mut slowed = batch(scenario, 200_000);
        slowed.evidence.push(
            EvidenceRef::from_bytes(
                EvidenceKind::CpuProfile,
                format!("tests/artifacts/perf/pg2-{scenario}/slowdown.pb"),
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
#[ignore = "real 8-thread timing probe; run explicitly under --profile release-perf"]
fn release_perf_producer_emits_replayable_real_samples_and_trace() {
    for scenario in Pg2Scenario::ALL {
        let baseline = Baseline::targeted(1, policy(scenario), key(scenario), COMMIT)
            .expect("target baseline");
        let artifacts = measure_pg2(
            &baseline,
            COMMIT,
            format!("tests/artifacts/perf/pg2-release-probe/{scenario}-trace.tsv"),
            None,
        )
        .expect("release-perf measurement");
        assert_eq!(artifacts.batch.samples.len(), PG2_SAMPLE_COUNT);
        assert!(!artifacts.batch.key.bare_metal);
        assert!(!artifacts.batch.key.isolated);
        assert_eq!(
            artifacts.frame_digest,
            Pg2Definition::new(scenario).expected_frame_digest()
        );
        assert!(artifacts.trace_tsv.starts_with(&format!(
            "schema\t{PG2_TRACE_SCHEMA}\ngate\tpg-2\nscenario\t{scenario}\n"
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
    }
}
