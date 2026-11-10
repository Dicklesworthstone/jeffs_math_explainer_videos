//! Gauntlet-only robot producer for the real PG-8 Python binding workloads.
//!
//! This is an example target deliberately: examples may use the
//! `fmn-python` dev-dependency, while the shipped `fmn-perf` dependency
//! graph remains free of PyO3. Run it from `crates/fmn-conformance/` under
//! the compiled `release-perf` profile.

#![forbid(unsafe_code)]

use fmn_conformance::perf::{
    BASELINE_SCHEMA, EvidenceKind, EvidenceRef, SAMPLES_SCHEMA, require_compiled_cargo_profile,
    validate_producer_commit,
};
use fmn_conformance::perf_host::{
    HostMonitor, HostProfile, HostQualification, attest_current_host,
};
use fmn_conformance::perf_pg8::{
    PG8_FRAMES_PER_REPETITION, PG8_MOBJECTS, PG8_SAMPLE_COUNT, PG8_WARMUP_ITERATIONS, Pg8Error,
    Pg8Measurement, Pg8Observation, Pg8Scenario, measure_pg8,
};
use fmn_hash::sha256;
use fmn_platform::fs::{FileSystem, FsNodeKind, StdFs};
use manimlib::perf_harness::{self, Pg8Class};
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const CLI_SCHEMA: &str = "fmn-perf-cli/1";
const EXIT_OK: u8 = 0;
const EXIT_USAGE: u8 = 64;
const EXIT_DATA: u8 = 65;
const EXIT_IO: u8 = 74;
const MAX_BASELINE_BYTES: u64 = 64 * 1024;
const MAX_HOST_PROFILE_BYTES: u64 = 64 * 1024;
const MAX_ERROR_DETAIL_BYTES: usize = 1024;
const MAX_ERROR_RECORD_BYTES: usize = 8 * 1024;

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
            "expected measure-pg8 <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> \
             [<host-profile.tsv> <host-attestation.tsv>]",
        ));
    };
    match command {
        "measure-pg8" if matches!(arguments.len(), 5 | 7) => measure_pg8_command(
            arguments.get(1).expect("length checked"),
            arguments.get(2).expect("length checked"),
            arguments.get(3).expect("length checked"),
            arguments.get(4).expect("length checked"),
            optional_qualification(arguments, 5)?,
        ),
        "measure-pg8" => Err(CliError::usage(
            "measure-pg8 requires <baseline.tsv> <producer-commit> <trace.tsv> <raw.tsv> \
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

fn measure_pg8_command(
    baseline_path: &OsStr,
    producer_commit: &OsStr,
    trace_path: &OsStr,
    raw_path: &OsStr,
    qualification_arguments: Option<(&OsStr, &OsStr)>,
) -> Result<String, CliError> {
    let baseline_text = read_utf8(baseline_path, "baseline", MAX_BASELINE_BYTES)?;
    let baseline = fmn_conformance::perf::Baseline::from_tsv(&baseline_text)
        .map_err(|error| CliError::data(error.to_string()))?;
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

    let artifacts = measure_pg8(
        &baseline,
        producer_commit,
        &real_sampler,
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
        "{{\"schema\":\"{CLI_SCHEMA}\",\"kind\":\"pg8-measurement\",\
         \"baseline_schema\":\"{BASELINE_SCHEMA}\",\"sample_schema\":\"{SAMPLES_SCHEMA}\",\
         \"gate\":\"pg-8\",\"scenario\":\"{}\",\
         \"benchmark_definition\":\"{}\",\"config_digest\":\"{}\",\
         \"producer_commit\":\"{}\",\"sample_count\":{},\
         \"valid_samples\":{},\"invalid_samples\":{},\
         \"bare_metal\":{},\"isolation_qualified\":{},\
         \"result_digest\":\"{}\",\"trace_path\":\"{}\",\
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
        artifacts.result_digest,
        escape_json(trace_path_text),
        trace_digest,
        escape_json(raw_path_text),
        raw_digest,
    ))
}

fn real_sampler(scenario: Pg8Scenario) -> Result<Pg8Measurement, Pg8Error> {
    let class = match scenario {
        Pg8Scenario::NativeBuiltins => Pg8Class::NativeBuiltins,
        Pg8Scenario::PerFrameCallback => Pg8Class::PerFrameCallback,
        Pg8Scenario::PointTransformCallback => Pg8Class::PointTransformCallback,
        Pg8Scenario::DynamicSubclass => Pg8Class::DynamicSubclass,
    };
    let run = perf_harness::measure(
        class,
        PG8_SAMPLE_COUNT,
        PG8_WARMUP_ITERATIONS,
        PG8_FRAMES_PER_REPETITION,
        PG8_MOBJECTS,
        1.0 / 30.0,
    )
    .map_err(Pg8Error::Harness)?;
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
    require_compiled_cargo_profile("release-perf")
        .map_err(|error| CliError::data(error.to_string()))?;
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
        let Some(kind) = host_node_kind(&cursor)
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
        let Some(kind) = host_node_kind(&cursor)
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

fn host_node_kind(path: &Path) -> Result<Option<FsNodeKind>, String> {
    StdFs
        .node_kind_no_follow(path)
        .map_err(|error| error.to_string())
}

fn write_new(path: &OsStr, bytes: &[u8], label: &str) -> Result<(), CliError> {
    match StdFs.create_new(Path::new(path), bytes) {
        Ok(true) => Ok(()),
        Ok(false) => Err(CliError::io(format!(
            "cannot create {label}: destination appeared before atomic publication"
        ))),
        Err(error) => Err(CliError::io(format!("cannot create {label}: {error}"))),
    }
}

fn read_utf8(path: &OsStr, label: &'static str, limit: u64) -> Result<String, CliError> {
    match host_node_kind(Path::new(path)) {
        Ok(Some(FsNodeKind::RegularFile)) => {}
        Ok(Some(FsNodeKind::Link)) => {
            return Err(CliError::data(format!(
                "{label} is a symbolic link or reparse point"
            )));
        }
        Ok(Some(FsNodeKind::Directory | FsNodeKind::Other)) => {
            return Err(CliError::data(format!("{label} is not a regular file")));
        }
        Ok(None) => {
            return Err(CliError::data(format!(
                "cannot read {label}: path is missing"
            )));
        }
        Err(error) => {
            return Err(CliError::data(format!("cannot inspect {label}: {error}")));
        }
    }
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
            detail: bounded_detail(detail),
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
        if output.len() <= MAX_ERROR_RECORD_BYTES {
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
    const LIMIT: usize = 160;
    if value.len() <= LIMIT {
        return format!("{value:?}");
    }
    let mut end = LIMIT;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{:?}... <{} bytes total>", &value[..end], value.len())
}

fn bounded_detail(value: &str) -> String {
    if value.len() <= MAX_ERROR_DETAIL_BYTES {
        return value.to_owned();
    }
    let suffix = format!("... <{} bytes total>", value.len());
    let mut end = MAX_ERROR_DETAIL_BYTES.saturating_sub(suffix.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &value[..end], suffix)
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

    #[test]
    fn usage_is_one_bounded_robot_record() {
        let error = dispatch(&[]).expect_err("missing command");
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert_eq!(error.to_ndjson().lines().count(), 1);
        assert!(error.to_ndjson().contains("\"error_kind\":\"usage\""));
    }

    #[test]
    fn ambiguous_qualification_pair_is_refused_before_io() {
        let arguments = vec![
            "measure-pg8".into(),
            "baseline.tsv".into(),
            "0123456789abcdef0123456789abcdef01234567".into(),
            "trace.tsv".into(),
            "raw.tsv".into(),
            "profile-without-attestation.tsv".into(),
        ];
        let error = dispatch(&arguments).expect_err("incomplete pair");
        assert_eq!(error.exit_code, EXIT_USAGE);
        assert!(error.detail.contains("host-attestation.tsv"));
    }

    #[test]
    fn historical_pg8_calibration_is_not_relabelled_as_the_current_producer() {
        let historical = fmn_conformance::perf::Baseline::from_tsv(include_str!(
            "../tests/artifacts/perf/pg8-native-builtins/baseline.tsv"
        ))
        .expect("historical baseline remains parseable");
        let error = fmn_conformance::perf_pg8::Pg8Definition::new(
            fmn_conformance::perf_pg8::Pg8Scenario::NativeBuiltins,
        )
        .validate_baseline(&historical)
        .expect_err("the current producer must refuse a historical definition");
        assert!(error.to_string().contains("benchmark_definition"));
    }
}
