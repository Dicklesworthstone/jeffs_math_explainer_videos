//! Governed robot front door for performance policy, evidence, and producers.

#![forbid(unsafe_code)]

use fmn_cache::{Store, StoreConfig};
use fmn_conformance::perf::{
    BASELINE_SCHEMA, Baseline, EvidenceKind, EvidenceRef, MeasurementBatch, POLICY_SCHEMA,
    SAMPLES_SCHEMA, parse_policy_catalog, render_policy_catalog, require_compiled_cargo_profile,
    validate_producer_commit,
};
use fmn_conformance::perf_frontdoor::{
    FRONTDOOR_DEFINITION_SCHEMA, FrontdoorDefinition, FrontdoorScenario, Pg1Reference,
    frontdoor_fixture, measure_frontdoor, probe_fmn_artifact,
};
use fmn_conformance::perf_host::{
    HostMonitor, HostProfile, HostQualification, attest_current_host,
};
use fmn_conformance::perf_pg2::{
    PG2_DEFINITION_SCHEMA, PG2_SAMPLE_COUNT, PG2_THREADS, PG2_WARMUP_ITERATIONS, Pg2Definition,
    Pg2Scenario, measure_pg2,
};
use fmn_conformance::perf_pg5::{
    PG5_CORPUS_SCENES, PG5_DEFINITION_SCHEMA, PG5_DIRECT_THREADS, PG5_PIPELINE_DEPTH,
    PG5_PIPELINE_TEAMS, PG5_SAMPLE_COUNT, PG5_SCENARIO, PG5_THREADS_PER_TEAM, Pg5Definition,
    measure_pg5, pg5_identity,
};
use fmn_conformance::perf_pg6::{
    PG6_DEFINITION_SCHEMA, PG6_SAMPLE_COUNT, PG6_SCENARIO, PG6_SOAK_SCENARIO, PG6_THREADS,
    PG6_WARMUP_FRAMES_PER_SCENE, Pg6Definition, Pg6SoakDefinition, measure_pg6, measure_pg6_soak,
    pg6_identity,
};
use fmn_conformance::perf_pg6_peak::{
    PG6_PEAK_CASE_COUNT, PG6_PEAK_DEFINITION_SCHEMA, PG6_PEAK_HEIGHT, PG6_PEAK_SAMPLE_COUNT,
    PG6_PEAK_SCENARIO, PG6_PEAK_THREADS, PG6_PEAK_WIDTH, Pg6PeakDefinition, measure_pg6_peak,
};
use fmn_conformance::perf_pg7::{
    PG7_DEFINITION_SCHEMA, PG7_SAMPLE_COUNT, PG7_WARMUP_ITERATIONS, Pg7Definition, Pg7Scenario,
    measure_pg7,
};
use fmn_conformance::perf_pg8::{
    PG8_DEFINITION_SCHEMA, PG8_FRAMES_PER_REPETITION, PG8_MOBJECTS, PG8_SAMPLE_COUNT,
    PG8_WARMUP_ITERATIONS, Pg8Definition, Pg8Scenario,
};
use fmn_hash::sha256;
use fmn_platform::clock::StdClock;
use fmn_platform::fs::{FileSystem, FsNodeKind, StdFs};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

const CLI_SCHEMA: &str = "fmn-perf-cli/1";
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 64;
const EXIT_DATA: u8 = 65;
const EXIT_IO: u8 = 74;
const MAX_POLICY_BYTES: u64 = 1024 * 1024;
const MAX_BASELINE_BYTES: u64 = 64 * 1024;
const MAX_RAW_BYTES: u64 = 128 * 1024 * 1024;
const MAX_HOST_PROFILE_BYTES: u64 = 64 * 1024;
const MAX_REFERENCE_BYTES: u64 = 256 * 1024;
const MAX_CLI_DIAGNOSTIC_VALUE_BYTES: usize = 160;
const MAX_CLI_ERROR_DETAIL_BYTES: usize = 1024;
const MAX_CLI_ERROR_RECORD_BYTES: usize = 8 * 1024;

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let (output, code) = match dispatch(&arguments) {
        Ok(output) => (output, EXIT_OK),
        Err(error) => (format!("{}\n", error.to_ndjson()), error.exit_code),
    };
    if std::io::stdout().write_all(output.as_bytes()).is_err() {
        ExitCode::from(EXIT_IO)
    } else {
        ExitCode::from(code)
    }
}

fn dispatch(arguments: &[std::ffi::OsString]) -> Result<String, CliError> {
    let Some(command) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(CliError::usage(
            "expected catalog, verify-baseline, pg2-definitions, measure-pg2, \
             pg5-definitions, measure-pg5, pg6-definitions, measure-pg6, measure-pg6-peak, \
             measure-pg6-soak, \
             pg7-definitions, measure-pg7, pg8-definitions, frontdoor-definitions, \
             or measure-frontdoor",
        ));
    };
    match command {
        "catalog" if arguments.len() == 2 => catalog(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing path"))?,
        ),
        "verify-baseline" if arguments.len() == 2 => verify_baseline(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing path"))?,
        ),
        "pg2-definitions" if arguments.len() == 1 => Ok(pg2_definitions()),
        "pg5-definitions" if arguments.len() == 1 => pg5_definitions(),
        "pg6-definitions" if arguments.len() == 1 => Ok(pg6_definitions()),
        "pg7-definitions" if arguments.len() == 1 => pg7_definitions(),
        "pg8-definitions" if arguments.len() == 1 => Ok(pg8_definitions()),
        "frontdoor-definitions" if arguments.len() == 3 => frontdoor_definitions(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing fmn executable"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing Reference evidence"))?,
        ),
        "measure-pg2" if matches!(arguments.len(), 5 | 7) => measure_pg2_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 5)?,
        ),
        "measure-pg5" if matches!(arguments.len(), 5 | 7) => measure_pg5_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 5)?,
        ),
        "measure-pg6" if matches!(arguments.len(), 5 | 7) => measure_pg6_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 5)?,
        ),
        "measure-pg6-peak" if matches!(arguments.len(), 5 | 7) => measure_pg6_peak_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 5)?,
        ),
        "measure-pg6-soak" if matches!(arguments.len(), 6 | 8) => measure_pg6_soak_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            arguments
                .get(5)
                .ok_or_else(|| CliError::usage("missing iterations-per-window"))?,
            optional_qualification(arguments, 6)?,
        ),
        "measure-pg7" if matches!(arguments.len(), 6 | 8) => measure_pg7_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing cache root"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(5)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 6)?,
        ),
        "measure-frontdoor" if matches!(arguments.len(), 8 | 10) => measure_frontdoor_command(
            arguments
                .get(1)
                .ok_or_else(|| CliError::usage("missing baseline path"))?,
            arguments
                .get(2)
                .ok_or_else(|| CliError::usage("missing producer commit"))?,
            arguments
                .get(3)
                .ok_or_else(|| CliError::usage("missing fmn executable"))?,
            arguments
                .get(4)
                .ok_or_else(|| CliError::usage("missing work root"))?,
            arguments
                .get(5)
                .ok_or_else(|| CliError::usage("missing Reference evidence or dash"))?,
            arguments
                .get(6)
                .ok_or_else(|| CliError::usage("missing trace output path"))?,
            arguments
                .get(7)
                .ok_or_else(|| CliError::usage("missing raw output path"))?,
            optional_qualification(arguments, 8)?,
        ),
        "catalog" | "verify-baseline" => Err(CliError::usage(format!(
            "{command} requires exactly one path argument"
        ))),
        "pg2-definitions" => Err(CliError::usage("pg2-definitions does not accept arguments")),
        "pg5-definitions" => Err(CliError::usage("pg5-definitions does not accept arguments")),
        "pg6-definitions" => Err(CliError::usage("pg6-definitions does not accept arguments")),
        "pg7-definitions" => Err(CliError::usage("pg7-definitions does not accept arguments")),
        "pg8-definitions" => Err(CliError::usage("pg8-definitions does not accept arguments")),
        "frontdoor-definitions" => Err(CliError::usage(
            "frontdoor-definitions requires <fmn-executable> <reference.tsv>",
        )),
        "measure-pg2" => Err(CliError::usage(
            "measure-pg2 requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-pg5" => Err(CliError::usage(
            "measure-pg5 requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-pg6" => Err(CliError::usage(
            "measure-pg6 requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-pg6-peak" => Err(CliError::usage(
            "measure-pg6-peak requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-pg6-soak" => Err(CliError::usage(
            "measure-pg6-soak requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> <iterations-per-window> [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-pg7" => Err(CliError::usage(
            "measure-pg7 requires <baseline.tsv> <producer-commit> \
             <cache-root-or-dash> <trace.tsv> <raw.tsv> \
             [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        "measure-frontdoor" => Err(CliError::usage(
            "measure-frontdoor requires <baseline.tsv> <producer-commit> <fmn-executable> \
             <work-root> <reference.tsv-or-dash> <trace.tsv> <raw.tsv> \
             [<host-profile.tsv> <host-attestation.tsv>]",
        )),
        _ => Err(CliError::usage(format!(
            "unknown command {}",
            bounded_argument_debug(command)
        ))),
    }
}

fn optional_qualification(
    arguments: &[std::ffi::OsString],
    first_optional: usize,
) -> Result<Option<(&OsStr, &OsStr)>, CliError> {
    if arguments.len() == first_optional {
        return Ok(None);
    }
    let profile = arguments
        .get(first_optional)
        .ok_or_else(|| CliError::usage("missing host profile path"))?;
    let attestation = arguments
        .get(first_optional + 1)
        .ok_or_else(|| CliError::usage("missing host attestation output path"))?;
    Ok(Some((profile.as_os_str(), attestation.as_os_str())))
}

fn catalog(path: &OsStr) -> Result<String, CliError> {
    let text = read_utf8(path, "policy catalog", MAX_POLICY_BYTES)?;
    let mut policies =
        parse_policy_catalog(&text).map_err(|error| CliError::data(error.to_string()))?;
    policies.sort_by(|left, right| {
        (left.gate, left.scenario.as_str()).cmp(&(right.gate, right.scenario.as_str()))
    });
    let canonical =
        render_policy_catalog(&policies).map_err(|error| CliError::data(error.to_string()))?;
    let mut output = String::new();
    let _ = writeln!(
        output,
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"catalog\",\
         \"policy_schema\":\"{POLICY_SCHEMA}\",\"policy_count\":{},\
         \"canonical_digest\":\"{}\"}}",
        policies.len(),
        sha256(canonical.as_bytes()),
    );
    for policy in policies {
        let _ = write!(
            output,
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"policy\",\
             \"gate\":\"{}\",\"scenario\":\"{}\",\"unit\":\"{}\",\
             \"direction\":\"{}\",\"target\":",
            policy.gate,
            escape_json(&policy.scenario),
            policy.unit.name(),
            policy.direction.name(),
        );
        if let Some(target) = policy.target {
            let _ = write!(output, "{target}");
        } else {
            output.push_str("null");
        }
        let _ = writeln!(
            output,
            ",\"min_valid_samples\":{},\"max_invalid_samples\":{},\
             \"max_mad_bps\":{},\"alert_regression_bps\":{},\
             \"block_regression_bps\":{},\"enforcement\":\"{}\",\
             \"scope\":\"{}\",\"require_regression_profile\":{}}}",
            policy.min_valid_samples,
            policy.max_invalid_samples,
            policy.max_mad_bps,
            policy.alert_regression_bps,
            policy.block_regression_bps,
            policy.enforcement.name(),
            policy.scope.name(),
            policy.require_regression_profile,
        );
    }
    Ok(output)
}

fn verify_baseline(path: &OsStr) -> Result<String, CliError> {
    let text = read_utf8(path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline = Baseline::from_tsv(&text).map_err(|error| CliError::data(error.to_string()))?;
    let observation = baseline
        .observation
        .as_ref()
        .ok_or_else(|| CliError::data("target-only baseline has no raw observation to verify"))?;
    let source_path = OsStr::new(&observation.source.path);
    preflight_regular_input(source_path, "raw observation")?;
    let raw = read_utf8(source_path, "raw observation", MAX_RAW_BYTES)?;
    let batch =
        MeasurementBatch::from_tsv(&raw).map_err(|error| CliError::data(error.to_string()))?;
    baseline
        .verify_observation_batch(&batch)
        .map_err(|error| CliError::data(error.to_string()))?;
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"baseline-verification\",\
         \"baseline_schema\":\"{BASELINE_SCHEMA}\",\
         \"sample_schema\":\"{SAMPLES_SCHEMA}\",\"gate\":\"{}\",\
         \"scenario\":\"{}\",\"generation\":{},\"producer_commit\":\"{}\",\
         \"source_path\":\"{}\",\"source_digest\":\"{}\",\"status\":\"verified\"}}\n",
        baseline.policy.gate,
        escape_json(&baseline.policy.scenario),
        baseline.generation,
        baseline.producer_commit,
        escape_json(&observation.source.path),
        observation.source.digest,
    ))
}

fn pg2_definitions() -> String {
    let mut output = String::new();
    for scenario in Pg2Scenario::ALL {
        let definition = Pg2Definition::new(scenario);
        let _ = writeln!(
            output,
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg2-definition\",\
             \"definition_schema\":\"{PG2_DEFINITION_SCHEMA}\",\"gate\":\"pg-2\",\
             \"scenario\":\"{}\",\"benchmark_definition\":\"{}\",\
             \"config_digest\":\"{}\",\"expected_frame_digest\":\"{}\",\
             \"engine\":\"fast-cpu\",\"tier\":\"{}\",\
             \"thread_profile\":\"fixed-8\",\"threads\":{PG2_THREADS},\
             \"sample_count\":{PG2_SAMPLE_COUNT},\
             \"warmup_iterations\":{PG2_WARMUP_ITERATIONS},\
             \"iterations_per_sample\":{},\"work_units_per_iteration\":{}}}",
            scenario.name(),
            definition.digest(),
            definition.config_digest(),
            definition.expected_frame_digest(),
            fmn_render::Tier::COMPILED.name(),
            definition.iterations_per_sample(),
            definition.work_units_per_iteration(),
        );
    }
    output
}

fn pg7_definitions() -> Result<String, CliError> {
    let mut output = String::new();
    for scenario in Pg7Scenario::ALL {
        let definition =
            Pg7Definition::new(scenario).map_err(|error| CliError::data(error.to_string()))?;
        let _ = writeln!(
            output,
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg7-definition\",\
             \"definition_schema\":\"{PG7_DEFINITION_SCHEMA}\",\"gate\":\"pg-7\",\
             \"scenario\":\"{}\",\"benchmark_definition\":\"{}\",\
             \"config_digest\":\"{}\",\"fixture_input_digest\":\"{}\",\
             \"expected_result_digest\":\"{}\",\"engine\":\"{}\",\"tier\":\"portable\",\
             \"thread_profile\":\"single-thread\",\"cache_state\":\"{}\",\
             \"output_mode\":\"{}\",\"sample_count\":{PG7_SAMPLE_COUNT},\
             \"warmup_iterations\":{PG7_WARMUP_ITERATIONS}}}",
            scenario.name(),
            definition.digest(),
            definition.config_digest(),
            definition.fixture_input_digest(),
            definition.expected_result_digest(),
            scenario.engine(),
            scenario.cache_state(),
            scenario.output_mode(),
        );
    }
    Ok(output)
}

fn pg8_definitions() -> String {
    let mut output = String::new();
    for scenario in Pg8Scenario::ALL {
        let definition = Pg8Definition::new(scenario);
        let _ = writeln!(
            output,
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg8-definition\",\
             \"definition_schema\":\"{PG8_DEFINITION_SCHEMA}\",\"gate\":\"pg-8\",\
             \"scenario\":\"{}\",\"unit\":\"{}\",\
             \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
             \"fixture_input_digest\":\"{}\",\"expected_result_digest\":\"{}\",\
             \"engine\":\"fmn-python-bridge\",\"tier\":\"portable\",\
             \"thread_profile\":\"single-thread\",\"cache_state\":\"none\",\
             \"output_mode\":\"record-buffer-state\",\
             \"sample_count\":{PG8_SAMPLE_COUNT},\
             \"warmup_iterations\":{PG8_WARMUP_ITERATIONS},\
             \"frames_per_repetition\":{PG8_FRAMES_PER_REPETITION},\
             \"mobjects\":{PG8_MOBJECTS},\"construction_timed\":{}}}",
            scenario.name(),
            scenario.unit().name(),
            definition.digest(),
            definition.config_digest(),
            definition.fixture_input_digest(),
            definition.expected_result_digest(),
            scenario.rebuilds_per_repetition(),
        );
    }
    output
}

fn frontdoor_definitions(executable: &OsStr, reference_path: &OsStr) -> Result<String, CliError> {
    let executable = Path::new(executable);
    let artifact = probe_fmn_artifact(executable, "release-perf")
        .map_err(|error| CliError::data(error.to_string()))?;
    let reference_path_text = utf8_argument(reference_path, "Reference evidence path")?;
    let reference_text = read_utf8(reference_path, "Reference evidence", MAX_REFERENCE_BYTES)?;
    let reference = Pg1Reference::from_tsv(reference_path_text, reference_text.as_bytes())
        .map_err(|error| CliError::data(error.to_string()))?;
    let mut output = String::new();
    for scenario in FrontdoorScenario::ALL {
        let fixture =
            frontdoor_fixture(scenario).map_err(|error| CliError::data(error.to_string()))?;
        let definition = FrontdoorDefinition::new(
            scenario,
            artifact.clone(),
            sha256(&fixture),
            (scenario.gate() == fmn_conformance::perf::GateId::Pg1).then(|| reference.clone()),
        )
        .map_err(|error| CliError::data(error.to_string()))?;
        let (width, height) = scenario.dimensions();
        let _ = writeln!(
            output,
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"frontdoor-definition\",\
             \"definition_schema\":\"{FRONTDOOR_DEFINITION_SCHEMA}\",\
             \"gate\":\"{}\",\"scenario\":\"{}\",\"unit\":\"{}\",\
             \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
             \"fixture_digest\":\"{}\",\"edit_fixture_digest\":{},\
             \"executable_digest\":\"{}\",\
             \"build_id\":\"{}\",\"target_triple\":\"{}\",\
             \"cargo_profile\":\"{}\",\"compiled_tier\":\"{}\",\
             \"resolution\":\"{}x{}\",\"fps\":30,\
             \"thread_profile\":\"fixed-8\",\"sample_count\":{},\
             \"cache_state\":\"{}\",\"output_mode\":\"{}\",\
             \"reference_digest\":{}}}",
            scenario.gate(),
            scenario.name(),
            scenario.unit().name(),
            definition.digest(),
            definition.config_digest(),
            definition.fixture_digest,
            definition
                .edit_fixture_digest
                .map_or_else(|| "null".to_owned(), |value| format!("\"{value}\""),),
            definition.artifact.executable_digest,
            escape_json(&definition.artifact.build_id),
            escape_json(&definition.artifact.target_triple),
            escape_json(&definition.artifact.cargo_profile),
            escape_json(&definition.artifact.compiled_tier),
            width,
            height,
            scenario.sample_count(),
            scenario.cache_state(),
            scenario.output_mode(),
            definition.reference.as_ref().map_or_else(
                || "null".to_owned(),
                |value| format!("\"{}\"", value.evidence.digest),
            ),
        );
    }
    Ok(output)
}

fn pg5_definitions() -> Result<String, CliError> {
    let definition = Pg5Definition::new().map_err(|error| CliError::data(error.to_string()))?;
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg5-definition\",\
         \"definition_schema\":\"{PG5_DEFINITION_SCHEMA}\",\"gate\":\"pg-5\",\
         \"scenario\":\"{PG5_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"execution_plan_digest\":\"{}\",\"corpus_lock_digest\":\"{}\",\
         \"engine\":\"{}\",\"tier\":\"{}\",\
         \"direct_threads\":[{},{},{}],\"sample_count\":{PG5_SAMPLE_COUNT},\
         \"corpus_scenes\":{PG5_CORPUS_SCENES},\
         \"pipeline_depth\":{PG5_PIPELINE_DEPTH},\
         \"pipeline_render_teams\":{PG5_PIPELINE_TEAMS},\
         \"pipeline_threads_per_team\":{PG5_THREADS_PER_TEAM}}}\n",
        definition.digest(),
        definition.config_digest(),
        definition.execution_plan_digest(),
        definition.corpus_lock_digest(),
        pg5_identity().engine.name(),
        pg5_identity().tier.name(),
        PG5_DIRECT_THREADS[0],
        PG5_DIRECT_THREADS[1],
        PG5_DIRECT_THREADS[2],
    ))
}

fn pg6_definitions() -> String {
    let definition = Pg6Definition::new();
    let peak = Pg6PeakDefinition::new();
    format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg6-definition\",\
         \"definition_schema\":\"{PG6_DEFINITION_SCHEMA}\",\"gate\":\"pg-6\",\
         \"scenario\":\"{PG6_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"corpus_lock_digest\":\"{}\",\
         \"engine\":\"{}\",\"tier\":\"{}\",\
         \"thread_profile\":\"fixed-4\",\"threads\":{PG6_THREADS},\
         \"sample_count\":{PG6_SAMPLE_COUNT},\
         \"warmup_frames_per_scene\":{PG6_WARMUP_FRAMES_PER_SCENE}}}\n\
         {{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg6-definition\",\
         \"definition_schema\":\"{PG6_PEAK_DEFINITION_SCHEMA}\",\"gate\":\"pg-6\",\
         \"scenario\":\"{PG6_PEAK_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"corpus_lock_digest\":\"{}\",\
         \"engine\":\"{}\",\"tier\":\"{}\",\
         \"thread_profile\":\"fixed-4-plus-rss-sampler\",\
         \"threads\":{PG6_PEAK_THREADS},\"sample_count\":{PG6_PEAK_SAMPLE_COUNT},\
         \"corpus_scenes\":{PG6_PEAK_CASE_COUNT},\
         \"frame_width_px\":{PG6_PEAK_WIDTH},\
         \"frame_height_px\":{PG6_PEAK_HEIGHT}}}\n",
        definition.digest(),
        definition.config_digest(),
        definition.corpus_lock_digest(),
        pg6_identity().engine.name(),
        pg6_identity().tier.name(),
        peak.digest(),
        peak.config_digest(),
        peak.corpus_lock_digest(),
        pg6_identity().engine.name(),
        pg6_identity().tier.name(),
    )
}

fn measure_pg2_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    // Validate both repository artifact paths and refuse replacement before
    // the expensive render. The attestation (when qualified) and trace precede
    // the raw bundle, so a later raw-file race can leave only a clearly
    // incomplete evidence set, never a plausible bundle naming bytes that
    // were not written.
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    let artifacts = measure_pg2(
        &baseline,
        producer_commit,
        trace_path_text,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let valid_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_none())
        .count();
    let invalid_samples = artifacts.batch.samples.len() - valid_samples;

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg2-measurement\",\
         \"gate\":\"pg-2\",\"scenario\":\"{}\",\"benchmark_definition\":\"{}\",\
         \"config_digest\":\"{}\",\"producer_commit\":\"{}\",\
         \"sample_count\":{},\"valid_samples\":{},\"invalid_samples\":{},\
         \"bare_metal\":{},\"isolation_qualified\":{},\
         \"frame_digest\":\"{}\",\"trace_path\":\"{}\",\
         \"trace_digest\":\"{}\",\"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        escape_json(&baseline.policy.scenario),
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        producer_commit,
        artifacts.batch.samples.len(),
        valid_samples,
        invalid_samples,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        artifacts.frame_digest,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

#[allow(clippy::too_many_arguments)]
fn measure_frontdoor_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    fmn_executable: &OsStr,
    work_root: &OsStr,
    reference_path: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    let work_root_text = utf8_argument(work_root, "work root")?;
    if trace_path_text == raw_path_text
        || trace_path_text == work_root_text
        || raw_path_text == work_root_text
    {
        return Err(CliError::data(
            "work root, trace, and raw output paths must be distinct",
        ));
    }
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    let fixture_path = Path::new(work_root).join("frontdoor-scene.fmtl");
    let fixture_path_text = fixture_path
        .to_str()
        .ok_or_else(|| CliError::data("work-root fixture path is not UTF-8"))?;
    EvidenceRef::from_bytes(EvidenceKind::Golden, fixture_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    validate_output_parent(work_root, "work root")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    refuse_existing(work_root, "work root")?;
    preflight_regular_input(fmn_executable, "fmn executable")?;

    let reference = if reference_path == OsStr::new("-") {
        None
    } else {
        preflight_regular_input(reference_path, "Reference evidence")?;
        let path = utf8_argument(reference_path, "Reference evidence path")?.to_owned();
        let text = read_utf8(reference_path, "Reference evidence", MAX_REFERENCE_BYTES)?;
        Some((path, text))
    };
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;
    let reference_borrow = reference
        .as_ref()
        .map(|(path, text)| (path.as_str(), text.as_bytes()));
    let artifacts = measure_frontdoor(
        &baseline,
        producer_commit,
        Path::new(fmn_executable),
        Path::new(work_root),
        trace_path_text,
        reference_borrow,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let valid_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_none())
        .count();
    let invalid_samples = artifacts.batch.samples.len() - valid_samples;
    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; work root and trace were already published and were not deleted",
            error.detail
        )));
    }
    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"frontdoor-measurement\",\
         \"sample_schema\":\"{SAMPLES_SCHEMA}\",\"gate\":\"{}\",\
         \"scenario\":\"{}\",\"producer_commit\":\"{}\",\
         \"work_root\":\"{}\",\"fixture_digest\":\"{}\",\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\",\
         \"valid_samples\":{},\"invalid_samples\":{}{} }}\n",
        artifacts.batch.key.gate,
        escape_json(&artifacts.batch.key.scenario),
        artifacts.batch.producer_commit,
        escape_json(work_root_text),
        artifacts.fixture_digest,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
        valid_samples,
        invalid_samples,
        attestation,
    ))
}

fn measure_pg5_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    let artifacts = measure_pg5(
        &baseline,
        producer_commit,
        trace_path_text,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let mismatch_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.value != 0)
        .count();

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg5-measurement\",\
         \"gate\":\"pg-5\",\"scenario\":\"{PG5_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"execution_plan_digest\":\"{}\",\"producer_commit\":\"{}\",\
         \"sample_count\":{},\"mismatch_samples\":{},\
         \"bare_metal\":{},\"isolation_qualified\":{},\
         \"reference_digest\":\"{}\",\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        artifacts.batch.key.execution_plan_digest,
        producer_commit,
        artifacts.batch.samples.len(),
        mismatch_samples,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        artifacts.reference_digest,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn measure_pg7_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    cache_root: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let scenario = Pg7Scenario::parse(&baseline.policy.scenario).ok_or_else(|| {
        CliError::data(format!(
            "unsupported PG-7 scenario {:?}",
            baseline.policy.scenario
        ))
    })?;
    let definition =
        Pg7Definition::new(scenario).map_err(|error| CliError::data(error.to_string()))?;
    definition
        .validate_baseline(&baseline)
        .map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let cache_root_text = utf8_argument(cache_root, "cache root")?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    if cache_root_text != "-"
        && (cache_root_text == trace_path_text || cache_root_text == raw_path_text)
    {
        return Err(CliError::data(
            "cache root, trace output, and raw output paths must be distinct",
        ));
    }
    if let Some((_, attestation_path)) = qualification_arguments
        && cache_root_text != "-"
        && cache_root_text == utf8_argument(attestation_path, "host attestation output path")?
    {
        return Err(CliError::data(
            "cache root and host attestation output paths must be distinct",
        ));
    }
    match scenario {
        Pg7Scenario::FormulaCached if cache_root_text == "-" || cache_root_text.is_empty() => {
            return Err(CliError::data(
                "formula-cached requires a fresh, nonexistent cache-root path",
            ));
        }
        Pg7Scenario::FormulaCold | Pg7Scenario::Text10kGlyph if cache_root_text != "-" => {
            return Err(CliError::data(format!(
                "{scenario} requires '-' for cache-root"
            )));
        }
        _ => {}
    }

    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    require_release_perf_front_door()?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    let store = if scenario == Pg7Scenario::FormulaCached {
        validate_cache_root(cache_root)?;
        refuse_existing(cache_root, "cache root")?;
        Some(
            Store::open_host(
                Arc::new(StdFs),
                Arc::new(StdClock::new()),
                cache_root_text,
                StoreConfig::default(),
            )
            .map_err(|error| CliError::data(format!("cannot open fresh cache root: {error}")))?,
        )
    } else {
        None
    };
    let artifacts = measure_pg7(
        &baseline,
        producer_commit,
        store.as_ref(),
        trace_path_text,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let valid_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_none())
        .count();
    let invalid_samples = artifacts.batch.samples.len() - valid_samples;

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg7-measurement\",\
         \"gate\":\"pg-7\",\"scenario\":\"{}\",\"benchmark_definition\":\"{}\",\
         \"config_digest\":\"{}\",\"producer_commit\":\"{}\",\
         \"sample_count\":{},\"valid_samples\":{},\"invalid_samples\":{},\
         \"bare_metal\":{},\"isolation_qualified\":{},\
         \"result_digest\":\"{}\",\"cache_state\":\"{}\",\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        escape_json(&baseline.policy.scenario),
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        producer_commit,
        artifacts.batch.samples.len(),
        valid_samples,
        invalid_samples,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        artifacts.result_digest,
        artifacts.batch.key.cache_state,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn measure_pg6_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    let artifacts = measure_pg6(
        &baseline,
        producer_commit,
        trace_path_text,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let nonzero_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.value != 0)
        .count();

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg6-measurement\",\
         \"gate\":\"pg-6\",\"scenario\":\"{PG6_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"producer_commit\":\"{}\",\"sample_count\":{},\
         \"nonzero_samples\":{},\"bare_metal\":{},\
         \"isolation_qualified\":{},\"result_digest\":\"{}\",\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        producer_commit,
        artifacts.batch.samples.len(),
        nonzero_samples,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        artifacts.result_digest,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn measure_pg6_peak_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    let probe =
        || fmn_platform::topology::current_rss_bytes(&StdFs).map_err(|error| error.to_string());
    let artifacts = measure_pg6_peak(
        &baseline,
        producer_commit,
        trace_path_text,
        &probe,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let invalid_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_some())
        .count();
    let peak_rss_bytes = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_none())
        .map(|sample| sample.value)
        .max();

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let peak = peak_rss_bytes
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned());
    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg6-peak-measurement\",\
         \"gate\":\"pg-6\",\"scenario\":\"{PG6_PEAK_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"producer_commit\":\"{}\",\"sample_count\":{},\
         \"invalid_samples\":{},\"peak_rss_bytes\":{},\
         \"bare_metal\":{},\"isolation_qualified\":{},\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        producer_commit,
        artifacts.batch.samples.len(),
        invalid_samples,
        peak,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn measure_pg6_soak_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    iterations_per_window: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline =
        Baseline::from_tsv(&baseline_text).map_err(|error| CliError::data(error.to_string()))?;
    let producer_commit = producer_commit_argument(producer_commit)?;
    let trace_path_text = utf8_argument(trace_path, "trace output path")?;
    let raw_path_text = utf8_argument(raw_path, "raw output path")?;
    if trace_path_text == raw_path_text {
        return Err(CliError::data(
            "trace and raw output paths must be distinct",
        ));
    }
    let iterations: u32 = utf8_argument(iterations_per_window, "iterations-per-window")?
        .parse()
        .map_err(|_| CliError::data("iterations-per-window must be a positive u32"))?;
    let definition =
        Pg6SoakDefinition::new(iterations).map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    EvidenceRef::from_bytes(EvidenceKind::RawSamples, raw_path_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(trace_path, "trace output")?;
    validate_output_parent(raw_path, "raw output")?;
    refuse_existing(trace_path, "trace output")?;
    refuse_existing(raw_path, "raw output")?;
    let qualification = prepare_host_qualification(qualification_arguments, trace_path, raw_path)?;
    let monitor = qualification.start_monitor()?;

    // The one production residency capability: fmn-platform over the host
    // filesystem. Unsupported hosts flow through as retained-invalid samples.
    let mut probe =
        || fmn_platform::topology::current_rss_bytes(&StdFs).map_err(|error| error.to_string());
    let artifacts = measure_pg6_soak(
        &baseline,
        producer_commit,
        trace_path_text,
        definition,
        &mut probe,
        qualification.token(),
    )
    .map_err(|error| CliError::data(error.to_string()))?;
    qualification.revalidate_postflight(monitor)?;
    let raw = artifacts
        .batch
        .to_tsv()
        .map_err(|error| CliError::data(error.to_string()))?;
    let raw_digest = sha256(raw.as_bytes());
    let trace_digest = sha256(artifacts.trace_tsv.as_bytes());
    let invalid_samples = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_some())
        .count();
    let leaked_windows = artifacts
        .batch
        .samples
        .iter()
        .filter(|sample| sample.invalid_reason.is_none() && sample.value != 0)
        .count();

    qualification.publish_then_trace(trace_path, artifacts.trace_tsv.as_bytes())?;
    if let Err(error) = write_new(raw_path, raw.as_bytes(), "raw output") {
        return Err(CliError::io(format!(
            "{}; trace output {trace_path_text:?} was already published and was not deleted",
            error.detail
        )));
    }

    let attestation = qualification.json_fragment();
    Ok(format!(
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg6-soak-measurement\",\
         \"gate\":\"pg-6\",\"scenario\":\"{PG6_SOAK_SCENARIO}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"producer_commit\":\"{}\",\"iterations_per_window\":{},\
         \"sample_count\":{},\"invalid_samples\":{},\
         \"leaked_windows\":{},\"bare_metal\":{},\
         \"isolation_qualified\":{},\
         \"trace_path\":\"{}\",\"trace_digest\":\"{}\",\
         \"raw_path\":\"{}\",\"raw_digest\":\"{}\"{attestation},\
         \"status\":\"measured-not-evaluated\"}}\n",
        artifacts.batch.key.benchmark_definition,
        artifacts.batch.key.config_digest,
        producer_commit,
        definition.iterations_per_window,
        artifacts.batch.samples.len(),
        invalid_samples,
        leaked_windows,
        artifacts.batch.key.bare_metal,
        artifacts.batch.key.isolated,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn validate_cache_root(path: &OsStr) -> Result<(), CliError> {
    let path = Path::new(path);
    if path.components().next().is_none()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CliError::data(
            "cache root is not a canonical relative path",
        ));
    }
    let artifact_root = Path::new("tests/artifacts/perf");
    if !path.starts_with(artifact_root) || path == artifact_root {
        return Err(CliError::data(
            "cache root must be a fresh child below tests/artifacts/perf/",
        ));
    }
    validate_output_parent(path.as_os_str(), "cache root")
}

struct PreparedQualification {
    token: Option<HostQualification>,
    output_path: Option<PathBuf>,
}

impl PreparedQualification {
    fn token(&self) -> Option<&HostQualification> {
        self.token.as_ref()
    }

    fn start_monitor(&self) -> Result<Option<HostMonitor>, CliError> {
        self.token
            .as_ref()
            .map(HostQualification::start_live_monitor)
            .transpose()
            .map_err(|error| CliError::data(error.to_string()))
    }

    fn revalidate_postflight(&self, monitor: Option<HostMonitor>) -> Result<(), CliError> {
        match (&self.token, monitor) {
            (Some(token), Some(monitor)) => {
                monitor
                    .stop_and_validate()
                    .map_err(|error| CliError::data(format!("continuous {error}")))?;
                token
                    .revalidate_current_host()
                    .map_err(|error| CliError::data(format!("postflight {error}")))?;
            }
            (None, None) => {}
            _ => {
                return Err(CliError::data(
                    "host qualification monitor lifecycle mismatch",
                ));
            }
        }
        Ok(())
    }

    fn publish_then_trace(&self, trace_path: &OsStr, trace_bytes: &[u8]) -> Result<(), CliError> {
        if let (Some(token), Some(path)) = (&self.token, &self.output_path) {
            write_new(
                path.as_os_str(),
                token.attestation_tsv().as_bytes(),
                "host attestation output",
            )?;
        }
        if let Err(error) = write_new(trace_path, trace_bytes, "trace output") {
            if let Some(path) = &self.output_path {
                return Err(CliError::io(format!(
                    "{}; host attestation output {:?} was already published and was not deleted",
                    error.detail, path
                )));
            }
            return Err(error);
        }
        Ok(())
    }

    fn json_fragment(&self) -> String {
        match (&self.token, &self.output_path) {
            (Some(token), Some(path)) => format!(
                ",\"host_attestation_path\":\"{}\",\"host_attestation_digest\":\"{}\"",
                escape_json(&path.to_string_lossy()),
                token.evidence().digest
            ),
            _ => ",\"host_attestation_path\":null,\"host_attestation_digest\":null".to_owned(),
        }
    }
}

fn prepare_host_qualification(
    arguments: Option<(&OsStr, &OsStr)>,
    trace_path: &OsStr,
    raw_path: &OsStr,
) -> Result<PreparedQualification, CliError> {
    let Some((profile_path, attestation_path)) = arguments else {
        return Ok(PreparedQualification {
            token: None,
            output_path: None,
        });
    };
    require_release_perf_front_door()?;
    preflight_regular_input(profile_path, "host profile")?;
    let profile_text = read_utf8(profile_path, "host profile", MAX_HOST_PROFILE_BYTES)?;
    let profile =
        HostProfile::from_tsv(&profile_text).map_err(|error| CliError::data(error.to_string()))?;
    let attestation_text = utf8_argument(attestation_path, "host attestation output path")?;
    let trace_text = utf8_argument(trace_path, "trace output path")?;
    let raw_text = utf8_argument(raw_path, "raw output path")?;
    if attestation_text == trace_text || attestation_text == raw_text {
        return Err(CliError::data(
            "host attestation, trace, and raw output paths must be distinct",
        ));
    }
    EvidenceRef::from_bytes(EvidenceKind::HostAttestation, attestation_text, &[])
        .map_err(|error| CliError::data(error.to_string()))?;
    validate_output_parent(attestation_path, "host attestation output")?;
    refuse_existing(attestation_path, "host attestation output")?;

    let raw = Path::new(raw_path);
    let parent = raw
        .parent()
        .ok_or_else(|| CliError::data("raw output has no parent directory"))?;
    let absolute_parent = fs::canonicalize(parent)
        .map_err(|error| CliError::io(format!("cannot resolve raw output parent: {error}")))?;
    let leaf = raw
        .file_name()
        .ok_or_else(|| CliError::data("raw output has no file name"))?;
    let artifact_path = absolute_parent.join(leaf);
    let token = attest_current_host(&profile, &artifact_path, attestation_text)
        .map_err(|error| CliError::data(error.to_string()))?;
    Ok(PreparedQualification {
        token: Some(token),
        output_path: Some(PathBuf::from(attestation_path)),
    })
}

fn require_release_perf_front_door() -> Result<(), CliError> {
    require_compiled_cargo_profile("release-perf")
        .map_err(|error| CliError::data(error.to_string()))
}

fn utf8_argument<'a>(value: &'a OsStr, label: &str) -> Result<&'a str, CliError> {
    value
        .to_str()
        .ok_or_else(|| CliError::data(format!("{label} is not UTF-8")))
}

fn producer_commit_argument(value: &OsStr) -> Result<&str, CliError> {
    let value = utf8_argument(value, "producer commit")?;
    validate_producer_commit(value).map_err(|error| CliError::data(error.to_string()))?;
    Ok(value)
}

fn refuse_existing(path: &OsStr, label: &str) -> Result<(), CliError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(CliError::data(format!(
            "{label} already exists; refusing to overwrite it"
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CliError::io(format!("cannot inspect {label}: {error}"))),
    }
}

fn validate_output_parent(path: &OsStr, label: &str) -> Result<(), CliError> {
    validate_output_parent_with(path, label, host_node_kind)
}

fn validate_output_parent_with(
    path: &OsStr,
    label: &str,
    mut node_kind: impl FnMut(&Path) -> Result<Option<FsNodeKind>, String>,
) -> Result<(), CliError> {
    let parent = Path::new(path)
        .parent()
        .ok_or_else(|| CliError::data(format!("{label} has no parent directory")))?;
    let mut cursor = PathBuf::new();
    for component in parent.components() {
        let Component::Normal(name) = component else {
            return Err(CliError::data(format!(
                "{label} parent is not a canonical relative path"
            )));
        };
        cursor.push(name);
        let Some(kind) = node_kind(&cursor)
            .map_err(|error| CliError::io(format!("cannot inspect {label} parent: {error}")))?
        else {
            return Err(CliError::io(format!(
                "cannot inspect {label} parent: missing path component {cursor:?}"
            )));
        };
        if kind == FsNodeKind::Link {
            return Err(CliError::data(format!(
                "{label} parent contains a symbolic link or reparse point at {cursor:?}"
            )));
        }
        if kind != FsNodeKind::Directory {
            return Err(CliError::data(format!(
                "{label} parent component is not a directory: {cursor:?}"
            )));
        }
    }
    Ok(())
}

fn preflight_regular_input(path: &OsStr, label: &str) -> Result<(), CliError> {
    preflight_regular_input_with(path, label, host_node_kind)
}

fn host_node_kind(path: &Path) -> Result<Option<FsNodeKind>, String> {
    StdFs
        .node_kind_no_follow(path)
        .map_err(|error| error.to_string())
}

fn preflight_regular_input_with(
    path: &OsStr,
    label: &str,
    mut node_kind: impl FnMut(&Path) -> Result<Option<FsNodeKind>, String>,
) -> Result<(), CliError> {
    let mut components = Path::new(path).components().peekable();
    if components.peek().is_none() {
        return Err(CliError::data(format!("{label} path is empty")));
    }

    let mut cursor = PathBuf::new();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(CliError::data(format!(
                "{label} is not a canonical relative path"
            )));
        };
        cursor.push(name);
        let Some(kind) = node_kind(&cursor)
            .map_err(|error| CliError::data(format!("cannot inspect {label}: {error}")))?
        else {
            return Err(CliError::data(format!(
                "{label} path component is missing: {cursor:?}"
            )));
        };
        if kind == FsNodeKind::Link {
            return Err(CliError::data(format!(
                "{label} contains a symbolic link or reparse point at {cursor:?}"
            )));
        }
        if components.peek().is_some() {
            if kind != FsNodeKind::Directory {
                return Err(CliError::data(format!(
                    "{label} parent component is not a directory: {cursor:?}"
                )));
            }
        } else if kind != FsNodeKind::RegularFile {
            return Err(CliError::data(format!(
                "{label} is not a regular file: {cursor:?}"
            )));
        }
    }
    Ok(())
}

fn write_new(path: &OsStr, bytes: &[u8], label: &str) -> Result<(), CliError> {
    write_new_with_fs(&StdFs, path, bytes, label)
}

fn write_new_with_fs(
    fs: &dyn FileSystem,
    path: &OsStr,
    bytes: &[u8],
    label: &str,
) -> Result<(), CliError> {
    match fs.create_new(Path::new(path), bytes) {
        Ok(true) => Ok(()),
        Ok(false) => Err(CliError::io(format!(
            "cannot create {label}: destination appeared before atomic publication"
        ))),
        Err(error) => Err(CliError::io(format!("cannot create {label}: {error}"))),
    }
}

fn read_utf8(path: &OsStr, label: &'static str, limit: u64) -> Result<String, CliError> {
    preflight_regular_leaf(path, label)?;
    let file = fs::File::open(path)
        .map_err(|error| CliError::data(format!("cannot read {label}: {error}")))?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| CliError::data(format!("cannot read {label}: {error}")))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(CliError::data(format!(
            "{label} exceeds the {limit}-byte resource limit"
        )));
    }
    String::from_utf8(bytes)
        .map_err(|error| CliError::data(format!("{label} is not UTF-8: {error}")))
}

fn preflight_regular_leaf(path: &OsStr, label: &str) -> Result<(), CliError> {
    preflight_regular_leaf_with(path, label, host_node_kind)
}

fn preflight_regular_leaf_with(
    path: &OsStr,
    label: &str,
    mut node_kind: impl FnMut(&Path) -> Result<Option<FsNodeKind>, String>,
) -> Result<(), CliError> {
    match node_kind(Path::new(path)) {
        Ok(Some(FsNodeKind::RegularFile)) => Ok(()),
        Ok(Some(FsNodeKind::Link)) => Err(CliError::data(format!(
            "{label} is a symbolic link or reparse point"
        ))),
        Ok(Some(FsNodeKind::Directory | FsNodeKind::Other)) => {
            Err(CliError::data(format!("{label} is not a regular file")))
        }
        Ok(None) => Err(CliError::data(format!(
            "cannot read {label}: path is missing"
        ))),
        Err(error) => Err(CliError::data(format!("cannot inspect {label}: {error}"))),
    }
}

#[derive(Debug)]
struct CliError {
    exit_code: u8,
    kind: &'static str,
    detail: String,
}

impl CliError {
    fn usage(detail: impl AsRef<str>) -> Self {
        Self::new(EXIT_USAGE, "usage", detail.as_ref())
    }

    fn data(detail: impl AsRef<str>) -> Self {
        Self::new(EXIT_DATA, "data", detail.as_ref())
    }

    fn io(detail: impl AsRef<str>) -> Self {
        Self::new(EXIT_IO, "io", detail.as_ref())
    }

    fn new(exit_code: u8, kind: &'static str, detail: &str) -> Self {
        Self {
            exit_code,
            kind,
            detail: bounded_cli_detail(detail),
        }
    }

    fn to_ndjson(&self) -> String {
        let output = format!(
            "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"error\",\
             \"error_kind\":\"{}\",\"exit_code\":{},\"detail\":\"{}\"}}",
            self.kind,
            self.exit_code,
            escape_json(&self.detail),
        );
        if output.len() <= MAX_CLI_ERROR_RECORD_BYTES {
            output
        } else {
            format!(
                "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"error\",\
                 \"error_kind\":\"{}\",\"exit_code\":{},\
                 \"detail\":\"error detail exceeded the internal record limit\"}}",
                self.kind, self.exit_code,
            )
        }
    }
}

fn bounded_argument_debug(value: &str) -> String {
    if value.len() <= MAX_CLI_DIAGNOSTIC_VALUE_BYTES {
        return format!("{value:?}");
    }
    let mut end = MAX_CLI_DIAGNOSTIC_VALUE_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{:?}... <{} bytes total>", &value[..end], value.len())
}

fn bounded_cli_detail(value: &str) -> String {
    if value.len() <= MAX_CLI_ERROR_DETAIL_BYTES {
        return value.to_owned();
    }
    let suffix = format!("... <{} bytes total>", value.len());
    let mut end = MAX_CLI_ERROR_DETAIL_BYTES.saturating_sub(suffix.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    let mut output = String::with_capacity(end + suffix.len());
    output.push_str(&value[..end]);
    output.push_str(&suffix);
    output
}

fn escape_json(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn usage_errors_are_single_line_robot_records() {
        let error = dispatch(&[]).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert_eq!(error.to_ndjson().lines().count(), 1);
        assert!(error.to_ndjson().contains("\"error_kind\":\"usage\""));
    }

    #[test]
    fn robot_error_escaping_cannot_inject_a_record() {
        let error = CliError::data("quote \" and newline\nstay data");
        let record = error.to_ndjson();
        assert_eq!(record.lines().count(), 1);
        assert!(record.contains("quote \\\" and newline\\nstay data"));
    }

    #[test]
    fn argv_diagnostics_and_error_records_are_bounded_before_rendering() {
        let short = dispatch(&["unknown".into()]).unwrap_err();
        assert_eq!(short.detail, "unknown command \"unknown\"");

        let command = format!("bad{}", "\u{00e9}".repeat(MAX_CLI_ERROR_DETAIL_BYTES));
        let command_bytes = command.len();
        let error = dispatch(&[command.into()]).unwrap_err();
        assert!(error.detail.len() <= MAX_CLI_ERROR_DETAIL_BYTES);
        assert!(
            error
                .detail
                .contains(&format!("<{command_bytes} bytes total>"))
        );
        let record = error.to_ndjson();
        assert!(record.len() <= MAX_CLI_ERROR_RECORD_BYTES);
        assert_eq!(record.lines().count(), 1);

        let raw_detail = "\u{1f}".repeat(MAX_CLI_ERROR_DETAIL_BYTES + 1);
        let raw_bytes = raw_detail.len();
        let error = CliError::data(&raw_detail);
        assert_eq!(error.detail.len(), MAX_CLI_ERROR_DETAIL_BYTES);
        assert!(error.detail.contains(&format!("<{raw_bytes} bytes total>")));
        let record = error.to_ndjson();
        assert!(record.len() <= MAX_CLI_ERROR_RECORD_BYTES);
        assert_eq!(record.lines().count(), 1);
    }

    #[test]
    fn release_perf_front_door_is_compile_bound() {
        let result = require_release_perf_front_door();
        let compiled_profile = fmn_conformance::perf::COMPILED_CARGO_PROFILE;
        assert_eq!(result.is_ok(), compiled_profile == "release-perf");
        if let Err(error) = result {
            assert!(error.detail.contains(compiled_profile));
        }
    }

    #[test]
    fn producer_commit_argument_is_validated_before_measurement_setup() {
        let error = producer_commit_argument(OsStr::new("not-a-commit")).unwrap_err();
        assert_eq!(error.kind, "data");
        assert!(error.detail.contains("producer_commit"));
        assert_eq!(
            producer_commit_argument(OsStr::new("0123456789abcdef0123456789abcdef01234567"))
                .unwrap(),
            "0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn committed_catalog_robot_surface_is_line_oriented_and_complete() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/performance/PERF_GATES.tsv");
        let output = catalog(path.as_os_str()).unwrap();
        assert_eq!(output.lines().count(), 22);
        assert!(output.lines().all(|line| {
            line.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && line.ends_with('}')
        }));
        assert!(output.contains("\"kind\":\"catalog\""));
        assert!(output.contains("\"gate\":\"pg-a\""));
    }

    #[test]
    fn pg2_definition_surface_is_closed_and_line_oriented() {
        let output = pg2_definitions();
        assert_eq!(output.lines().count(), 2);
        assert!(output.lines().all(|line| {
            line.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && line.ends_with('}')
        }));
        assert!(output.contains("\"scenario\":\"fill-canonical\""));
        assert!(output.contains("\"scenario\":\"stroke-canonical\""));
        assert!(!output.contains("\"status\""));
    }

    #[test]
    fn pg6_definition_surface_is_closed_and_line_oriented() {
        let output = pg6_definitions();
        assert_eq!(output.lines().count(), 2);
        assert!(output.lines().all(|line| {
            line.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && line.ends_with('}')
        }));
        assert!(output.contains("\"scenario\":\"primitive-steady-allocations\""));
        assert!(output.contains("\"scenario\":\"gallery-4k-3d-peak\""));
        assert!(output.contains("\"frame_width_px\":3840"));
        assert!(output.contains(&format!("\"sample_count\":{PG6_SAMPLE_COUNT}")));
        assert!(!output.contains("\"status\""));
    }

    #[test]
    fn pg5_definition_surface_is_closed_and_line_oriented() {
        let output = pg5_definitions().expect("fixed PG-5 definition");
        assert_eq!(output.lines().count(), 1);
        assert!(output.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && output.ends_with("}\n"));
        assert!(output.contains("\"scenario\":\"certified-thread-matrix\""));
        assert!(output.contains("\"direct_threads\":[1,4,16]"));
        assert!(!output.contains("\"status\""));
    }

    #[test]
    fn pg7_definition_surface_is_closed_and_line_oriented() {
        let output = pg7_definitions().unwrap();
        assert_eq!(output.lines().count(), 3);
        assert!(output.lines().all(|line| {
            line.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && line.ends_with('}')
        }));
        assert!(output.contains("\"scenario\":\"formula-cold\""));
        assert!(output.contains("\"scenario\":\"formula-cached\""));
        assert!(output.contains("\"scenario\":\"text-10k-glyph\""));
        assert!(!output.contains("\"status\""));
    }

    #[test]
    fn pg8_definition_surface_is_closed_and_line_oriented() {
        let output = pg8_definitions();
        assert_eq!(output.lines().count(), 4);
        assert!(output.lines().all(|line| {
            line.starts_with("{\"schema\":\"fmn-perf-cli/1\"") && line.ends_with('}')
        }));
        for scenario in Pg8Scenario::ALL {
            assert!(
                output.contains(&format!("\"scenario\":\"{}\"", scenario.name())),
                "missing {scenario}"
            );
        }
        assert!(output.contains("\"construction_timed\":true"));
        assert!(output.contains(&format!("\"sample_count\":{PG8_SAMPLE_COUNT}")));
        assert!(!output.contains("\"status\""));
    }

    #[test]
    fn measure_pg2_refuses_ambiguous_argument_counts_before_io() {
        let arguments = vec![std::ffi::OsString::from("measure-pg2")];
        let error = dispatch(&arguments).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("<baseline.tsv>"));

        let arguments = vec![
            "measure-pg2".into(),
            "baseline.tsv".into(),
            "0123456789abcdef0123456789abcdef01234567".into(),
            "trace.tsv".into(),
            "raw.tsv".into(),
            "profile-without-attestation.tsv".into(),
        ];
        let error = dispatch(&arguments).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("host-attestation.tsv"));
    }

    #[test]
    fn measure_pg6_refuses_ambiguous_argument_counts_before_io() {
        let arguments = vec![std::ffi::OsString::from("measure-pg6")];
        let error = dispatch(&arguments).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("<baseline.tsv>"));
    }

    #[test]
    fn measure_pg5_refuses_ambiguous_argument_counts_before_io() {
        let arguments = vec![std::ffi::OsString::from("measure-pg5")];
        let error = dispatch(&arguments).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("<baseline.tsv>"));
    }

    #[test]
    fn measure_pg7_refuses_ambiguous_argument_counts_before_io() {
        let arguments = vec![std::ffi::OsString::from("measure-pg7")];
        let error = dispatch(&arguments).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("<cache-root-or-dash>"));
    }

    #[test]
    fn frontdoor_commands_refuse_ambiguous_argument_counts_before_io() {
        let definitions = vec![std::ffi::OsString::from("frontdoor-definitions")];
        let error = dispatch(&definitions).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("<fmn-executable>"));
        assert!(error.detail.contains("<reference.tsv>"));

        let measurement = vec![
            "measure-frontdoor".into(),
            "baseline.tsv".into(),
            "0123456789abcdef0123456789abcdef01234567".into(),
            "fmn".into(),
            "tests/artifacts/perf/frontdoor/work".into(),
            "-".into(),
            "tests/artifacts/perf/frontdoor/trace.tsv".into(),
            "tests/artifacts/perf/frontdoor/raw.tsv".into(),
            "profile-without-attestation.tsv".into(),
        ];
        let error = dispatch(&measurement).unwrap_err();
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("host-attestation.tsv"));
    }

    #[test]
    fn output_parent_check_requires_canonical_existing_directories() {
        validate_output_parent(OsStr::new("src/new-output.tsv"), "test output").unwrap();
        let error =
            validate_output_parent(OsStr::new("src/../new-output.tsv"), "test output").unwrap_err();
        assert_eq!(error.kind, "data");
        assert!(error.detail.contains("canonical relative path"));

        const OUTPUT: &str = "tests/artifacts/perf/run/raw.tsv";
        for (override_kind, expected_kind, expected_detail) in [
            (
                Some(FsNodeKind::Link),
                "data",
                "symbolic link or reparse point",
            ),
            (Some(FsNodeKind::RegularFile), "data", "not a directory"),
            (Some(FsNodeKind::Other), "data", "not a directory"),
            (None, "io", "missing path component"),
        ] {
            let error =
                validate_output_parent_with(OsStr::new(OUTPUT), "test output", |candidate| {
                    Ok(if candidate == Path::new("tests/artifacts") {
                        override_kind
                    } else {
                        Some(FsNodeKind::Directory)
                    })
                })
                .unwrap_err();
            assert_eq!(error.kind, expected_kind);
            assert!(error.detail.contains(expected_detail), "{}", error.detail);
        }
    }

    #[test]
    fn replay_source_preflight_requires_a_no_follow_regular_file_walk() {
        const SOURCE: &str = "tests/artifacts/perf/run/raw.tsv";
        let leaf = Path::new(SOURCE);
        let ordinary = |candidate: &Path| {
            Ok::<_, String>(Some(if candidate == leaf {
                FsNodeKind::RegularFile
            } else {
                FsNodeKind::Directory
            }))
        };
        preflight_regular_input_with(OsStr::new(SOURCE), "raw observation", ordinary).unwrap();

        for (override_path, override_kind, expected) in [
            (
                "tests/artifacts",
                Some(FsNodeKind::Link),
                "symbolic link or reparse point",
            ),
            (
                SOURCE,
                Some(FsNodeKind::Link),
                "symbolic link or reparse point",
            ),
            (SOURCE, Some(FsNodeKind::Other), "not a regular file"),
            (
                "tests/artifacts/perf/run",
                None,
                "path component is missing",
            ),
        ] {
            let override_path = Path::new(override_path);
            let error =
                preflight_regular_input_with(OsStr::new(SOURCE), "raw observation", |candidate| {
                    if candidate == override_path {
                        Ok(override_kind)
                    } else {
                        ordinary(candidate)
                    }
                })
                .unwrap_err();
            assert!(error.detail.contains(expected), "{}", error.detail);
        }

        preflight_regular_input(OsStr::new("Cargo.toml"), "test input").unwrap();
        let error = preflight_regular_input(OsStr::new("src"), "test input").unwrap_err();
        assert!(error.detail.contains("not a regular file"));
    }

    #[test]
    fn bounded_reader_preflight_requires_a_regular_leaf() {
        let path = OsStr::new("/explicit/authority.tsv");
        preflight_regular_leaf_with(path, "authority", |_| Ok(Some(FsNodeKind::RegularFile)))
            .unwrap();

        for (kind, expected) in [
            (Some(FsNodeKind::Link), "symbolic link or reparse point"),
            (Some(FsNodeKind::Directory), "not a regular file"),
            (Some(FsNodeKind::Other), "not a regular file"),
            (None, "path is missing"),
        ] {
            let error = preflight_regular_leaf_with(path, "authority", |_| Ok(kind)).unwrap_err();
            assert_eq!(error.kind, "data");
            assert!(error.detail.contains(expected), "{}", error.detail);
        }

        let error = preflight_regular_leaf_with(path, "authority", |_| {
            Err("classification unavailable".to_owned())
        })
        .unwrap_err();
        assert_eq!(error.kind, "data");
        assert!(error.detail.contains("classification unavailable"));
    }

    #[test]
    fn write_new_uses_complete_no_clobber_publication() {
        let fs = fmn_platform::fs::VirtualFs::new();
        fs.create_dir(Path::new("/evidence")).unwrap();
        let path = OsStr::new("/evidence/raw.tsv");

        write_new_with_fs(&fs, path, b"complete artifact", "raw output").unwrap();
        assert_eq!(
            fs.read(Path::new(path)).unwrap(),
            b"complete artifact".to_vec()
        );

        let error = write_new_with_fs(&fs, path, b"replacement", "raw output").unwrap_err();
        assert_eq!(error.kind, "io");
        assert!(error.detail.contains("before atomic publication"));
        assert_eq!(
            fs.read(Path::new(path)).unwrap(),
            b"complete artifact".to_vec()
        );
    }

    #[test]
    fn cache_root_check_is_canonical_and_artifact_scoped() {
        let error = validate_cache_root(OsStr::new("/tmp/pg7-cache")).unwrap_err();
        assert!(error.detail.contains("canonical relative path"));
        let error = validate_cache_root(OsStr::new("src/pg7-cache")).unwrap_err();
        assert!(error.detail.contains("tests/artifacts/perf"));
        let error = validate_cache_root(OsStr::new("tests/artifacts/perf/missing-parent/cache"))
            .unwrap_err();
        assert!(error.detail.contains("cannot inspect cache root parent"));
    }
}
