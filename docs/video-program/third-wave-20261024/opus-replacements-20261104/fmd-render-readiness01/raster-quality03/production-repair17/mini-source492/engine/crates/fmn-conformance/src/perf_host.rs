//! Live, fail-closed benchmark-host qualification (plan §17.2, fm-inr.1).
//!
//! A baseline is caller-controlled data, so its `bare_metal=true` and
//! `isolated=true` fields are never authority.  This module is the sole
//! constructor of [`HostQualification`].  On Linux it compares a strict,
//! versioned host profile with live `/proc`, sysfs, cgroup-v2, thermal, power,
//! and mount evidence and with identities embedded in the compiled artifact.
//! A producer may consume the resulting in-process token, but cannot recreate
//! one from serialized bytes.  Unsupported platforms and incomplete probes
//! fail before measurement rather than silently becoming qualified.

use crate::perf::{Baseline, BenchmarkKey, EvidenceKind, EvidenceRef, PerfError};
use fmn_hash::{Digest, Sha256, sha256};
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_platform::topology::{HardwareTopology, parse_cpu_list};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Stable exact-host profile schema.
pub const HOST_PROFILE_SCHEMA: &str = "fmn-perf-host-profile/1";
/// Stable content-addressed live-attestation schema.
pub const HOST_ATTESTATION_SCHEMA: &str = "fmn-perf-host-attestation/3";

/// Fixed continuous-monitor policy recorded in every qualified attestation.
pub const HOST_MONITOR_POLICY: &str =
    "linux-live-state-250ms-slice-quiescence-plus-full-postflight-v2";
/// Sampling interval for volatile live host state during a qualified run.
pub const HOST_MONITOR_INTERVAL_MILLIS: u64 = 250;

const MAX_PROFILE_BYTES: usize = 64 * 1024;
const MAX_PROFILE_LINES: usize = 64;
const MAX_HOST_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOKEN_BYTES: usize = 160;
const MAX_PATH_BYTES: usize = 512;
const MAX_KERNEL_BYTES: usize = 4 * 1024;
const MAX_CGROUP_PROCESSES: usize = 4_096;
/// Size of the benchmark slice in distinct physical cores. The machine may be
/// larger: ADR-0024 qualifies an isolated slice, not a machine shape.
const LINUX_PHYSICAL_CORES: u32 = 8;
/// Busy ceiling, in permille of their CPU time since the monitor started, for
/// the SMT siblings of the benchmark cores (ADR-0024: siblings carry no work).
const MAX_SIBLING_BUSY_PERMILLE: u64 = 10;
/// Aggregate busy ceiling, in permille of their CPU time since the monitor
/// started, for every CPU outside the benchmark cores and their siblings
/// (ADR-0024: the rest of the machine is quiescent while a slice measures).
const MAX_BACKGROUND_BUSY_PERMILLE: u64 = 50;

const SUITE_LOCK_BYTES: &[u8] = include_bytes!("../../../SUITE.lock");
const RUST_TOOLCHAIN_BYTES: &[u8] = include_bytes!("../../../rust-toolchain.toml");
const COMPILED_RUSTC_IDENTITY: &str = env!("FMN_CONFORMANCE_RUSTC_IDENTITY");

/// Exact profile family.  macOS is represented in the durable schema now,
/// but remains deliberately unqualified until safe native introspection lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostPlatform {
    /// Dedicated eight-core x86-64 Linux profile required by plan §17.2.
    LinuxX86_64,
    /// Dedicated Apple-silicon macOS profile required by plan §17.2.
    MacosAarch64,
}

impl HostPlatform {
    const fn name(self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "linux-x86_64",
            Self::MacosAarch64 => "macos-aarch64",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "linux-x86_64" => Self::LinuxX86_64,
            "macos-aarch64" => Self::MacosAarch64,
            _ => return None,
        })
    }
}

/// Exact, reviewable requirements for one dedicated benchmark machine.
///
/// Digests intentionally protect host-sensitive hardware strings while still
/// making every compared byte exact and replayable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostProfile {
    /// Versioned profile name carried by every [`BenchmarkKey`].
    pub profile_id: String,
    /// OS/architecture family.
    pub platform: HostPlatform,
    /// SHA-256 of the exact `/etc/os-release` bytes.
    pub os_release_digest: Digest,
    /// Exact kernel release.
    pub kernel_release: String,
    /// SHA-256 of stable CPU identity rows from `/proc/cpuinfo`.
    pub cpu_identity_digest: Digest,
    /// SHA-256 of the canonical non-serial DMI identity.
    pub dmi_identity_digest: Digest,
    /// SHA-256 of [`HardwareTopology::snapshot_text`].
    pub topology_digest: Digest,
    /// The exact eight CPUs made available to the benchmark process.
    pub benchmark_cpus: Vec<u32>,
    /// Exact cgroup-v2 path dedicated to the benchmark process.
    pub cgroup_path: String,
    /// Required scaling governor on every benchmark CPU.
    pub governor: String,
    /// Sysfs boost/turbo policy leaf.
    pub boost_path: PathBuf,
    /// Exact required contents of the boost/turbo policy leaf.
    pub boost_value: String,
    /// Sysfs temperature leaf used for the start/end ceiling.
    pub thermal_path: PathBuf,
    /// Maximum admitted start/end temperature.
    pub max_temperature_millicelsius: u64,
    /// Maximum admitted one-minute host load, in thousandths.
    pub max_load_milli: u64,
    /// Exact mount point containing measurement artifacts.
    pub storage_mount: PathBuf,
    /// Exact filesystem type from mountinfo.
    pub storage_fs: String,
    /// SHA-256 of the mount source (keeps device identity non-public).
    pub storage_source_digest: Digest,
}

impl HostProfile {
    /// Parse the strict key/value TSV profile.
    ///
    /// # Errors
    /// Returns a bounded schema error for duplicate, missing, unknown, or
    /// malformed fields.
    pub fn from_tsv(input: &str) -> Result<Self, HostError> {
        if input.len() > MAX_PROFILE_BYTES {
            return Err(HostError::Profile(format!(
                "profile exceeds the {MAX_PROFILE_BYTES}-byte limit"
            )));
        }
        let mut values = BTreeMap::new();
        let mut lines = input.lines();
        if lines.next() != Some(HOST_PROFILE_SCHEMA) {
            return Err(HostError::Profile(format!(
                "expected schema {HOST_PROFILE_SCHEMA:?}"
            )));
        }
        for (index, line) in lines.enumerate() {
            if index + 2 > MAX_PROFILE_LINES {
                return Err(HostError::Profile(format!(
                    "profile exceeds the {MAX_PROFILE_LINES}-line limit"
                )));
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (name, value) = line.split_once('\t').ok_or_else(|| {
                HostError::Profile(format!("profile line {} is not key/value TSV", index + 2))
            })?;
            if value.contains('\t') {
                return Err(HostError::Profile(format!(
                    "profile field {name:?} contains an extra column"
                )));
            }
            if values.insert(name.to_owned(), value.to_owned()).is_some() {
                return Err(HostError::Profile(format!(
                    "duplicate profile field {name:?}"
                )));
            }
        }
        let expected: BTreeSet<&str> = PROFILE_FIELDS.iter().copied().collect();
        let found: BTreeSet<&str> = values.keys().map(String::as_str).collect();
        if expected != found {
            let missing: Vec<_> = expected.difference(&found).copied().collect();
            let unknown: Vec<_> = found.difference(&expected).copied().collect();
            return Err(HostError::Profile(format!(
                "profile field set mismatch; missing={missing:?}, unknown={unknown:?}"
            )));
        }
        let get = |name: &'static str| {
            values
                .get(name)
                .cloned()
                .ok_or_else(|| HostError::Profile(format!("missing profile field {name:?}")))
        };
        let platform_text = get("platform")?;
        let platform = HostPlatform::parse(&platform_text)
            .ok_or_else(|| HostError::Profile(format!("unsupported platform {platform_text:?}")))?;
        let benchmark_cpus_text = get("benchmark_cpus")?;
        let benchmark_cpus = parse_cpu_list(&benchmark_cpus_text)
            .map_err(|error| HostError::Profile(format!("bad benchmark_cpus: {error}")))?;
        let profile = Self {
            profile_id: get("profile_id")?,
            platform,
            os_release_digest: parse_digest(&get("os_release_digest")?, "os_release_digest")?,
            kernel_release: get("kernel_release")?,
            cpu_identity_digest: parse_digest(&get("cpu_identity_digest")?, "cpu_identity_digest")?,
            dmi_identity_digest: parse_digest(&get("dmi_identity_digest")?, "dmi_identity_digest")?,
            topology_digest: parse_digest(&get("topology_digest")?, "topology_digest")?,
            benchmark_cpus,
            cgroup_path: get("cgroup_path")?,
            governor: get("governor")?,
            boost_path: PathBuf::from(get("boost_path")?),
            boost_value: get("boost_value")?,
            thermal_path: PathBuf::from(get("thermal_path")?),
            max_temperature_millicelsius: parse_u64(
                &get("max_temperature_millicelsius")?,
                "max_temperature_millicelsius",
            )?,
            max_load_milli: parse_u64(&get("max_load_milli")?, "max_load_milli")?,
            storage_mount: PathBuf::from(get("storage_mount")?),
            storage_fs: get("storage_fs")?,
            storage_source_digest: parse_digest(
                &get("storage_source_digest")?,
                "storage_source_digest",
            )?,
        };
        profile.validate()?;
        Ok(profile)
    }

    /// Canonical profile bytes.  Their digest is the host fingerprint.
    #[must_use]
    pub fn to_tsv(&self) -> String {
        format!(
            "{HOST_PROFILE_SCHEMA}\nprofile_id\t{}\nplatform\t{}\n\
             os_release_digest\t{}\nkernel_release\t{}\ncpu_identity_digest\t{}\n\
             dmi_identity_digest\t{}\ntopology_digest\t{}\nbenchmark_cpus\t{}\n\
             cgroup_path\t{}\ngovernor\t{}\nboost_path\t{}\nboost_value\t{}\n\
             thermal_path\t{}\nmax_temperature_millicelsius\t{}\nmax_load_milli\t{}\n\
             storage_mount\t{}\nstorage_fs\t{}\nstorage_source_digest\t{}\n",
            self.profile_id,
            self.platform.name(),
            self.os_release_digest,
            self.kernel_release,
            self.cpu_identity_digest,
            self.dmi_identity_digest,
            self.topology_digest,
            format_cpu_list(&self.benchmark_cpus),
            self.cgroup_path,
            self.governor,
            self.boost_path.display(),
            self.boost_value,
            self.thermal_path.display(),
            self.max_temperature_millicelsius,
            self.max_load_milli,
            self.storage_mount.display(),
            self.storage_fs,
            self.storage_source_digest,
        )
    }

    /// Exact profile digest carried as [`BenchmarkKey::host_fingerprint`].
    #[must_use]
    pub fn digest(&self) -> Digest {
        sha256(self.to_tsv().as_bytes())
    }

    fn validate(&self) -> Result<(), HostError> {
        validate_token("profile_id", &self.profile_id)?;
        validate_scalar("kernel_release", &self.kernel_release, MAX_TOKEN_BYTES)?;
        validate_scalar("cgroup_path", &self.cgroup_path, MAX_PATH_BYTES)?;
        validate_token("governor", &self.governor)?;
        validate_scalar("boost_value", &self.boost_value, MAX_TOKEN_BYTES)?;
        validate_token("storage_fs", &self.storage_fs)?;
        if self.platform == HostPlatform::LinuxX86_64 && self.benchmark_cpus.len() != 8 {
            return Err(HostError::Profile(
                "linux-x86_64 benchmark_cpus must name exactly eight CPUs".to_owned(),
            ));
        }
        if self.benchmark_cpus.is_empty() {
            return Err(HostError::Profile(
                "benchmark_cpus must not be empty".to_owned(),
            ));
        }
        if !self.cgroup_path.starts_with('/') || self.cgroup_path.contains("..") {
            return Err(HostError::Profile(
                "cgroup_path must be an absolute traversal-free cgroup-v2 path".to_owned(),
            ));
        }
        validate_sysfs_leaf(
            "boost_path",
            &self.boost_path,
            &["/sys/devices/system/cpu/"],
        )?;
        validate_sysfs_leaf(
            "thermal_path",
            &self.thermal_path,
            &["/sys/class/thermal/", "/sys/class/hwmon/"],
        )?;
        validate_absolute_path("storage_mount", &self.storage_mount)?;
        if self.max_temperature_millicelsius == 0 || self.max_temperature_millicelsius > 150_000 {
            return Err(HostError::Profile(
                "max_temperature_millicelsius must be in 1..=150000".to_owned(),
            ));
        }
        if self.max_load_milli > 64_000 {
            return Err(HostError::Profile(
                "max_load_milli exceeds the 64000 resource ceiling".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Opaque proof that the running process matched one exact host profile.
///
/// Fields are private and there is no deserializer.  Serialized attestation
/// bytes are evidence, not a reusable authority token.
#[derive(Clone, Debug)]
pub struct HostQualification {
    profile_id: String,
    host_fingerprint: Digest,
    toolchain_fingerprint: Digest,
    suite_lock_digest: Digest,
    evidence: EvidenceRef,
    attestation_tsv: String,
    profile: HostProfile,
    plan: IsolationPlan,
    artifact_path: PathBuf,
    process_id: u32,
}

/// Running continuous live-state monitor for one qualified measurement.
///
/// The monitor samples the volatile Linux qualification surface at a fixed
/// interval and always performs one final sample when
/// [`Self::stop_and_validate`] is called. Dropping it also stops and joins the
/// thread, but only `stop_and_validate` reports a policy excursion; successful
/// producers must call it before publishing evidence.
#[must_use = "a qualified host monitor must be finished before evidence publication"]
#[derive(Debug)]
pub struct HostMonitor {
    stop: Option<mpsc::Sender<()>>,
    outcome: mpsc::Receiver<Result<u64, HostError>>,
    thread: Option<JoinHandle<()>>,
}

impl HostMonitor {
    fn start_with_probe(
        interval: Duration,
        mut probe: impl FnMut() -> Result<(), HostError> + Send + 'static,
    ) -> Result<Self, HostError> {
        let (stop_tx, stop_rx) = mpsc::channel();
        let (outcome_tx, outcome_rx) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("fmn-host-monitor".to_owned())
            .spawn(move || {
                let mut samples = 0_u64;
                loop {
                    match stop_rx.recv_timeout(interval) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                            samples = samples.saturating_add(1);
                            let result = probe().map(|()| samples);
                            let _ = outcome_tx.send(result);
                            return;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            samples = samples.saturating_add(1);
                            if let Err(error) = probe() {
                                let _ = outcome_tx.send(Err(HostError::Mismatch(format!(
                                    "continuous live-state sample {samples} failed: {error}"
                                ))));
                                return;
                            }
                        }
                    }
                }
            })
            .map_err(|error| HostError::Probe(format!("cannot start host monitor: {error}")))?;
        Ok(Self {
            stop: Some(stop_tx),
            outcome: outcome_rx,
            thread: Some(thread),
        })
    }

    /// Stop the monitor, perform its mandatory final live-state sample, and
    /// return the number of completed samples.
    ///
    /// # Errors
    /// Returns the first policy excursion, probe failure, thread panic, or
    /// monitor-channel failure.
    pub fn stop_and_validate(mut self) -> Result<u64, HostError> {
        self.finish_inner()
    }

    fn finish_inner(&mut self) -> Result<u64, HostError> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        let thread = self
            .thread
            .take()
            .ok_or_else(|| HostError::Probe("host monitor was already finished".to_owned()))?;
        if thread.join().is_err() {
            return Err(HostError::Probe("host monitor thread panicked".to_owned()));
        }
        self.outcome
            .recv()
            .map_err(|_| HostError::Probe("host monitor produced no outcome".to_owned()))?
    }
}

impl Drop for HostMonitor {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _ = self.finish_inner();
        }
    }
}

impl HostQualification {
    /// Exact live-attestation artifact bytes the caller must publish before
    /// the raw sample bundle.
    #[must_use]
    pub fn attestation_tsv(&self) -> &str {
        &self.attestation_tsv
    }

    /// Content-addressed evidence reference added to the measurement batch.
    #[must_use]
    pub fn evidence(&self) -> &EvidenceRef {
        &self.evidence
    }

    /// Start the fixed-policy continuous monitor for this qualified run.
    /// Calibration runs have no [`HostQualification`] and therefore cannot
    /// create a monitor accidentally.
    ///
    /// # Errors
    /// Returns a precise unsupported-platform or thread-start failure.
    pub fn start_live_monitor(&self) -> Result<HostMonitor, HostError> {
        match self.profile.platform {
            HostPlatform::LinuxX86_64 => {
                if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                    return Err(HostError::Unsupported(format!(
                        "profile {} requires linux-x86_64",
                        self.profile.profile_id
                    )));
                }
                self.start_live_monitor_with_fs(
                    Arc::new(StdFs),
                    Duration::from_millis(HOST_MONITOR_INTERVAL_MILLIS),
                )
            }
            HostPlatform::MacosAarch64 => Err(HostError::Unsupported(
                "macos-aarch64 lacks a safe native topology/power/isolation probe; fallback topology cannot qualify a pinned host"
                    .to_owned(),
            )),
        }
    }

    fn start_live_monitor_with_fs(
        &self,
        fs: Arc<dyn FileSystem>,
        interval: Duration,
    ) -> Result<HostMonitor, HostError> {
        let profile = self.profile.clone();
        let plan = self.plan.clone();
        let artifact_path = self.artifact_path.clone();
        let pid = self.process_id;
        // Quiescence is judged cumulatively from this baseline, so the final
        // sample covers the whole measurement window.
        let baseline = read_cpu_ticks(fs.as_ref())?;
        HostMonitor::start_with_probe(interval, move || {
            validate_linux_live_state(&profile, &plan, fs.as_ref(), &artifact_path, pid)?;
            require_quiescence(&plan, &baseline, &read_cpu_ticks(fs.as_ref())?)
        })
    }

    /// Re-run the complete live authority after the workload. A qualified
    /// CLI run publishes nothing when thermal, load, cgroup, affinity, power,
    /// mount, or any exact identity drifted during the measurement window.
    ///
    /// # Errors
    /// The same fail-closed [`HostError`] surface as preflight attestation.
    pub fn revalidate_current_host(&self) -> Result<(), HostError> {
        match self.profile.platform {
            HostPlatform::LinuxX86_64 => {
                if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                    return Err(HostError::Unsupported(format!(
                        "profile {} requires linux-x86_64",
                        self.profile.profile_id
                    )));
                }
                self.revalidate_linux_with_fs(&StdFs)
            }
            HostPlatform::MacosAarch64 => Err(HostError::Unsupported(
                "macos-aarch64 lacks a safe native topology/power/isolation probe; fallback topology cannot qualify a pinned host"
                    .to_owned(),
            )),
        }
    }

    fn revalidate_linux_with_fs(&self, fs: &dyn FileSystem) -> Result<(), HostError> {
        let found = attest_linux_host(
            &self.profile,
            fs,
            &self.artifact_path,
            self.evidence.path.clone(),
            self.process_id,
        )?;
        if found.profile_id != self.profile_id
            || found.host_fingerprint != self.host_fingerprint
            || found.toolchain_fingerprint != self.toolchain_fingerprint
            || found.suite_lock_digest != self.suite_lock_digest
        {
            return Err(HostError::Mismatch(
                "postflight qualification identity changed".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Host qualification failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    /// Malformed or unsafe profile.
    Profile(String),
    /// A required host capability is unavailable.
    Unsupported(String),
    /// Live state differs from the exact profile or violates isolation.
    Mismatch(String),
    /// Bounded host evidence could not be read or parsed.
    Probe(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(detail) => write!(formatter, "host profile: {detail}"),
            Self::Unsupported(detail) => {
                write!(formatter, "host attestation unavailable: {detail}")
            }
            Self::Mismatch(detail) => write!(formatter, "host attestation mismatch: {detail}"),
            Self::Probe(detail) => write!(formatter, "host attestation probe: {detail}"),
        }
    }
}

impl std::error::Error for HostError {}

/// Fingerprint of the actual compiler identity embedded by the package build
/// script plus the exact `rust-toolchain.toml` policy bytes.
#[must_use]
pub fn compiled_toolchain_fingerprint() -> Digest {
    let mut hash = Sha256::new();
    hash.update(b"fmn-perf-compiled-toolchain-v1");
    hash_field(&mut hash, COMPILED_RUSTC_IDENTITY.as_bytes());
    hash_field(&mut hash, RUST_TOOLCHAIN_BYTES);
    hash.finalize()
}

/// Digest of the exact `SUITE.lock` bytes compiled into this artifact.
#[must_use]
pub fn compiled_suite_lock_digest() -> Digest {
    sha256(SUITE_LOCK_BYTES)
}

/// Attest the current process against `profile` and bind the evidence to the
/// intended repository-relative attestation output path.
///
/// # Errors
/// Fails closed on unsupported platforms, incomplete inspection, any live
/// mismatch, or an invalid evidence path.
pub fn attest_current_host(
    profile: &HostProfile,
    artifact_path: &Path,
    evidence_path: impl Into<String>,
) -> Result<HostQualification, HostError> {
    match profile.platform {
        HostPlatform::LinuxX86_64 => {
            if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                return Err(HostError::Unsupported(format!(
                    "profile {} requires linux-x86_64",
                    profile.profile_id
                )));
            }
            attest_linux_host(
                profile,
                &StdFs,
                artifact_path,
                evidence_path.into(),
                std::process::id(),
            )
        }
        HostPlatform::MacosAarch64 => Err(HostError::Unsupported(
            "macos-aarch64 lacks a safe native topology/power/isolation probe; fallback topology cannot qualify a pinned host"
                .to_owned(),
        )),
    }
}

/// Return a measurement key and optional attestation evidence.  Without a
/// token the key is always explicitly unqualified.
///
/// # Errors
/// Refuses a token whose live identities do not exactly match the supplied
/// baseline key.
pub fn measurement_identity(
    baseline: &BenchmarkKey,
    qualification: Option<&HostQualification>,
) -> Result<(BenchmarkKey, Vec<EvidenceRef>), PerfError> {
    let Some(qualification) = qualification else {
        let mut key = baseline.clone();
        key.bare_metal = false;
        key.isolated = false;
        return Ok((key, Vec::new()));
    };
    let mut differences = Vec::new();
    if baseline.profile_id != qualification.profile_id {
        differences.push("profile_id");
    }
    if baseline.host_fingerprint != qualification.host_fingerprint {
        differences.push("host_fingerprint");
    }
    if baseline.toolchain_fingerprint != qualification.toolchain_fingerprint {
        differences.push("toolchain_fingerprint");
    }
    if baseline.suite_lock_digest != qualification.suite_lock_digest {
        differences.push("suite_lock_digest");
    }
    if baseline.build_profile != crate::perf::COMPILED_CARGO_PROFILE {
        differences.push("build_profile");
    }
    if !differences.is_empty() {
        return Err(PerfError::Identity(format!(
            "live host qualification differs from baseline fields: {}",
            differences.join(", ")
        )));
    }
    let mut key = baseline.clone();
    key.bare_metal = true;
    key.isolated = true;
    Ok((key, vec![qualification.evidence.clone()]))
}

/// Reviewed exact-host profiles, one `<profile_id>.tsv` per host, relative
/// to the fmn-conformance crate.
pub const REVIEWED_HOST_PROFILE_DIR: &str = "tests/artifacts/perf/hosts";

/// Refuse a committed baseline that claims a qualified host without a
/// reviewed profile for it (fm-5wq.32).
///
/// [`measurement_identity`] never trusts a baseline's `bare_metal` and
/// `isolated` fields, but a committed fixture that states them for a host
/// with no reviewed profile still mislabels its evidence class for every
/// reader of the file.
///
/// # Errors
/// [`PerfError::Baseline`] naming `path`, its `profile_id` and the claimed
/// fields.
pub fn check_baseline_host_claims(
    path: &str,
    baseline: &Baseline,
    reviewed_profiles: &BTreeSet<String>,
) -> Result<(), PerfError> {
    let key = &baseline.key;
    let claimed: Vec<&str> = [("bare_metal", key.bare_metal), ("isolated", key.isolated)]
        .into_iter()
        .filter_map(|(name, value)| value.then_some(name))
        .collect();
    if claimed.is_empty() || reviewed_profiles.contains(&key.profile_id) {
        return Ok(());
    }
    Err(PerfError::Baseline(format!(
        "{path} claims {} for profile_id {} without a reviewed host profile in \
         {REVIEWED_HOST_PROFILE_DIR}",
        claimed.join(" and "),
        key.profile_id
    )))
}

fn attest_linux_host(
    profile: &HostProfile,
    fs: &dyn FileSystem,
    artifact_path: &Path,
    evidence_path: String,
    pid: u32,
) -> Result<HostQualification, HostError> {
    profile.validate()?;
    let os_release = read_required(fs, Path::new("/etc/os-release"), MAX_HOST_FILE_BYTES)?;
    require_digest(
        "os_release_digest",
        profile.os_release_digest,
        sha256(os_release.as_bytes()),
    )?;
    let kernel_release = read_required(
        fs,
        Path::new("/proc/sys/kernel/osrelease"),
        MAX_KERNEL_BYTES,
    )?;
    require_text(
        "kernel_release",
        &profile.kernel_release,
        kernel_release.trim(),
    )?;

    let cpuinfo = read_required(fs, Path::new("/proc/cpuinfo"), MAX_HOST_FILE_BYTES)?;
    if cpuinfo_has_hypervisor(&cpuinfo) {
        return Err(HostError::Mismatch(
            "/proc/cpuinfo reports a hypervisor".to_owned(),
        ));
    }
    if fs.exists(Path::new("/sys/hypervisor/type")) {
        return Err(HostError::Mismatch(
            "/sys/hypervisor/type exists".to_owned(),
        ));
    }
    let cpu_identity = canonical_cpu_identity(&cpuinfo)?;
    require_digest(
        "cpu_identity_digest",
        profile.cpu_identity_digest,
        sha256(cpu_identity.as_bytes()),
    )?;

    let dmi_identity = canonical_dmi_identity(fs)?;
    if contains_virtual_machine_marker(&dmi_identity) {
        return Err(HostError::Mismatch(
            "DMI identity contains a virtual-machine marker".to_owned(),
        ));
    }
    require_digest(
        "dmi_identity_digest",
        profile.dmi_identity_digest,
        sha256(dmi_identity.as_bytes()),
    )?;

    let topology = HardwareTopology::detect_linux(fs)
        .map_err(|error| HostError::Probe(format!("topology: {error}")))?;
    // The digest pins the whole machine's topology; the slice below is what
    // is measured and isolated, whatever the machine's size (ADR-0024).
    require_digest(
        "topology_digest",
        profile.topology_digest,
        sha256(topology.snapshot_text().as_bytes()),
    )?;
    let plan = isolation_plan(profile, &topology)?;

    let live = validate_linux_live_state(profile, &plan, fs, artifact_path, pid)?;

    let host_fingerprint = profile.digest();
    let toolchain_fingerprint = compiled_toolchain_fingerprint();
    let suite_lock_digest = compiled_suite_lock_digest();
    let mut attestation = String::new();
    writeln!(&mut attestation, "{HOST_ATTESTATION_SCHEMA}").expect("string write");
    for (name, value) in [
        ("profile_id", profile.profile_id.clone()),
        ("profile_digest", host_fingerprint.to_string()),
        ("platform", profile.platform.name().to_owned()),
        ("os_release_digest", profile.os_release_digest.to_string()),
        ("kernel_release", profile.kernel_release.clone()),
        (
            "cpu_identity_digest",
            profile.cpu_identity_digest.to_string(),
        ),
        (
            "dmi_identity_digest",
            profile.dmi_identity_digest.to_string(),
        ),
        ("topology_digest", profile.topology_digest.to_string()),
        ("benchmark_cpus", format_cpu_list(&profile.benchmark_cpus)),
        ("reserved_cpus", format_cpu_list(&plan.reserved)),
        ("numa_node", plan.numa_node.to_string()),
        ("cgroup_path", profile.cgroup_path.clone()),
        ("governor", profile.governor.clone()),
        ("boost_value", profile.boost_value.clone()),
        (
            "temperature_millicelsius",
            live.temperature_millicelsius.to_string(),
        ),
        ("load_milli", live.load_milli.to_string()),
        (
            "storage_mount",
            live.mount.mount_point.display().to_string(),
        ),
        ("storage_fs", live.mount.fs_type),
        (
            "storage_source_digest",
            profile.storage_source_digest.to_string(),
        ),
        ("toolchain_fingerprint", toolchain_fingerprint.to_string()),
        ("suite_lock_digest", suite_lock_digest.to_string()),
        ("monitor_policy", HOST_MONITOR_POLICY.to_owned()),
        (
            "monitor_interval_millis",
            HOST_MONITOR_INTERVAL_MILLIS.to_string(),
        ),
        (
            "sibling_busy_ceiling_permille",
            MAX_SIBLING_BUSY_PERMILLE.to_string(),
        ),
        (
            "background_busy_ceiling_permille",
            MAX_BACKGROUND_BUSY_PERMILLE.to_string(),
        ),
        ("process_id", pid.to_string()),
        ("bare_metal", "true".to_owned()),
        ("isolated", "true".to_owned()),
    ] {
        writeln!(&mut attestation, "{name}\t{value}").expect("string write");
    }
    let evidence = EvidenceRef::from_bytes(
        EvidenceKind::HostAttestation,
        evidence_path,
        attestation.as_bytes(),
    )
    .map_err(|error| HostError::Profile(error.to_string()))?;
    Ok(HostQualification {
        profile_id: profile.profile_id.clone(),
        host_fingerprint,
        toolchain_fingerprint,
        suite_lock_digest,
        evidence,
        attestation_tsv: attestation,
        profile: profile.clone(),
        plan,
        artifact_path: artifact_path.to_path_buf(),
        process_id: pid,
    })
}

#[derive(Debug)]
struct LinuxLiveState {
    temperature_millicelsius: u64,
    load_milli: u64,
    mount: MountRecord,
}

fn validate_linux_live_state(
    profile: &HostProfile,
    plan: &IsolationPlan,
    fs: &dyn FileSystem,
    artifact_path: &Path,
    pid: u32,
) -> Result<LinuxLiveState, HostError> {
    let status = read_required(fs, Path::new("/proc/self/status"), MAX_HOST_FILE_BYTES)?;
    let allowed = parse_status_cpu_list(&status, "Cpus_allowed_list")?;
    require_cpu_list("Cpus_allowed_list", &profile.benchmark_cpus, &allowed)?;
    require_initial_pid_namespace(&status)?;
    for (label, path) in [
        ("isolated", "/sys/devices/system/cpu/isolated"),
        ("nohz_full", "/sys/devices/system/cpu/nohz_full"),
        ("rcu_nocbs", "/sys/devices/system/cpu/rcu_nocbs"),
    ] {
        let text = read_required(fs, Path::new(path), MAX_KERNEL_BYTES)?;
        let cpus = parse_cpu_list(text.trim())
            .map_err(|error| HostError::Probe(format!("{label} CPU list: {error}")))?;
        require_cpu_superset(label, &plan.reserved, &cpus)?;
    }

    let cgroup = parse_unified_cgroup(&read_required(
        fs,
        Path::new("/proc/self/cgroup"),
        MAX_KERNEL_BYTES,
    )?)?;
    require_text("cgroup_path", &profile.cgroup_path, &cgroup)?;
    let cgroup_root = Path::new("/sys/fs/cgroup").join(cgroup.trim_start_matches('/'));
    let effective = read_required(
        fs,
        &cgroup_root.join("cpuset.cpus.effective"),
        MAX_KERNEL_BYTES,
    )?;
    let effective = parse_cpu_list(effective.trim())
        .map_err(|error| HostError::Probe(format!("cgroup cpuset: {error}")))?;
    require_cpu_list("cgroup cpuset", &profile.benchmark_cpus, &effective)?;
    let mems = read_required(
        fs,
        &cgroup_root.join("cpuset.mems.effective"),
        MAX_KERNEL_BYTES,
    )?;
    let mems = parse_cpu_list(mems.trim())
        .map_err(|error| HostError::Probe(format!("cgroup cpuset.mems: {error}")))?;
    require_cpu_list("cgroup cpuset.mems", &[plan.numa_node], &mems)?;
    let processes = parse_processes(&read_required(
        fs,
        &cgroup_root.join("cgroup.procs"),
        MAX_HOST_FILE_BYTES,
    )?)?;
    if processes.as_slice() != [pid] {
        return Err(HostError::Mismatch(format!(
            "dedicated cgroup must contain only measurement pid {pid}, found {processes:?}"
        )));
    }

    for &cpu in &profile.benchmark_cpus {
        let path = PathBuf::from(format!(
            "/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor"
        ));
        let found = read_required(fs, &path, MAX_KERNEL_BYTES)?;
        require_text("scaling_governor", &profile.governor, found.trim())?;
    }
    let boost = read_required(fs, &profile.boost_path, MAX_KERNEL_BYTES)?;
    require_text("boost_value", &profile.boost_value, boost.trim())?;
    let temperature_millicelsius = parse_trimmed_u64(
        &read_required(fs, &profile.thermal_path, MAX_KERNEL_BYTES)?,
        "temperature",
    )?;
    if temperature_millicelsius > profile.max_temperature_millicelsius {
        return Err(HostError::Mismatch(format!(
            "temperature {temperature_millicelsius} mC exceeds profile ceiling {} mC",
            profile.max_temperature_millicelsius
        )));
    }
    let load_milli = parse_load_milli(&read_required(
        fs,
        Path::new("/proc/loadavg"),
        MAX_KERNEL_BYTES,
    )?)?;
    if load_milli > profile.max_load_milli {
        return Err(HostError::Mismatch(format!(
            "one-minute load {load_milli} milli exceeds profile ceiling {}",
            profile.max_load_milli
        )));
    }

    let mount = find_mount(
        &read_required(fs, Path::new("/proc/self/mountinfo"), MAX_HOST_FILE_BYTES)?,
        artifact_path,
    )?;
    if mount.mount_point != profile.storage_mount {
        return Err(HostError::Mismatch(format!(
            "artifact path resolves under mount {:?}, expected {:?}",
            mount.mount_point, profile.storage_mount
        )));
    }
    require_text("storage_fs", &profile.storage_fs, &mount.fs_type)?;
    require_digest(
        "storage_source_digest",
        profile.storage_source_digest,
        sha256(mount.source.as_bytes()),
    )?;

    Ok(LinuxLiveState {
        temperature_millicelsius,
        load_milli,
        mount,
    })
}

const PROFILE_FIELDS: [&str; 18] = [
    "profile_id",
    "platform",
    "os_release_digest",
    "kernel_release",
    "cpu_identity_digest",
    "dmi_identity_digest",
    "topology_digest",
    "benchmark_cpus",
    "cgroup_path",
    "governor",
    "boost_path",
    "boost_value",
    "thermal_path",
    "max_temperature_millicelsius",
    "max_load_milli",
    "storage_mount",
    "storage_fs",
    "storage_source_digest",
];

fn parse_digest(value: &str, name: &str) -> Result<Digest, HostError> {
    Digest::from_hex(value).map_err(|error| HostError::Profile(format!("bad {name}: {error}")))
}

fn parse_u64(value: &str, name: &str) -> Result<u64, HostError> {
    value
        .parse()
        .map_err(|_| HostError::Profile(format!("{name} is not a canonical u64")))
}

fn validate_token(name: &str, value: &str) -> Result<(), HostError> {
    if value.is_empty()
        || value.len() > MAX_TOKEN_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+' | b':')
        })
    {
        return Err(HostError::Profile(format!(
            "{name} is not a portable token"
        )));
    }
    Ok(())
}

fn validate_scalar(name: &str, value: &str, max: usize) -> Result<(), HostError> {
    if value.is_empty()
        || value.len() > max
        || value
            .chars()
            .any(|character| character.is_control() || character == '\t')
    {
        return Err(HostError::Profile(format!("invalid {name}")));
    }
    Ok(())
}

fn validate_absolute_path(name: &str, path: &Path) -> Result<(), HostError> {
    let text = path
        .to_str()
        .ok_or_else(|| HostError::Profile(format!("{name} is not UTF-8")))?;
    validate_scalar(name, text, MAX_PATH_BYTES)?;
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(HostError::Profile(format!(
            "{name} must be an absolute traversal-free path"
        )));
    }
    Ok(())
}

fn validate_sysfs_leaf(name: &str, path: &Path, roots: &[&str]) -> Result<(), HostError> {
    validate_absolute_path(name, path)?;
    let text = path.to_string_lossy();
    if !roots.iter().any(|root| text.starts_with(root)) {
        return Err(HostError::Profile(format!(
            "{name} is outside the admitted sysfs roots"
        )));
    }
    Ok(())
}

fn read_required(fs: &dyn FileSystem, path: &Path, limit: usize) -> Result<String, HostError> {
    fs.read_to_string_bounded(path, limit)
        .map_err(|error| HostError::Probe(error.to_string()))
}

fn require_digest(name: &str, expected: Digest, found: Digest) -> Result<(), HostError> {
    if expected == found {
        Ok(())
    } else {
        Err(HostError::Mismatch(format!(
            "{name} expected {expected}, found {found}"
        )))
    }
}

fn require_text(name: &str, expected: &str, found: &str) -> Result<(), HostError> {
    if expected == found {
        Ok(())
    } else {
        Err(HostError::Mismatch(format!(
            "{name} expected {expected:?}, found {found:?}"
        )))
    }
}

fn require_cpu_list(name: &str, expected: &[u32], found: &[u32]) -> Result<(), HostError> {
    if expected == found {
        Ok(())
    } else {
        Err(HostError::Mismatch(format!(
            "{name} expected {}, found {}",
            format_cpu_list(expected),
            format_cpu_list(found)
        )))
    }
}

fn format_cpu_list(cpus: &[u32]) -> String {
    cpus.iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn canonical_cpu_identity(cpuinfo: &str) -> Result<String, HostError> {
    let admitted = [
        "processor",
        "vendor_id",
        "cpu family",
        "model",
        "model name",
        "stepping",
        "microcode",
    ];
    let mut output = String::new();
    for line in cpuinfo.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if admitted.contains(&name) {
            writeln!(&mut output, "{name}\t{}", normalize_space(value)).expect("string write");
        }
    }
    if output.is_empty() {
        return Err(HostError::Probe(
            "/proc/cpuinfo has no stable CPU identity rows".to_owned(),
        ));
    }
    Ok(output)
}

fn cpuinfo_has_hypervisor(cpuinfo: &str) -> bool {
    cpuinfo.lines().any(|line| {
        let Some((name, value)) = line.split_once(':') else {
            return false;
        };
        matches!(name.trim(), "flags" | "Features")
            && value.split_whitespace().any(|flag| flag == "hypervisor")
    })
}

fn canonical_dmi_identity(fs: &dyn FileSystem) -> Result<String, HostError> {
    let mut output = String::new();
    for leaf in [
        "sys_vendor",
        "product_name",
        "product_version",
        "board_vendor",
        "board_name",
        "board_version",
        "bios_version",
    ] {
        let path = Path::new("/sys/class/dmi/id").join(leaf);
        let value = read_required(fs, &path, MAX_KERNEL_BYTES)?;
        let value = normalize_space(&value);
        if value.is_empty() {
            return Err(HostError::Probe(format!(
                "DMI identity leaf {leaf:?} is empty"
            )));
        }
        writeln!(&mut output, "{leaf}\t{value}").expect("string write");
    }
    Ok(output)
}

fn contains_virtual_machine_marker(identity: &str) -> bool {
    let lower = identity.to_ascii_lowercase();
    [
        "qemu",
        "kvm",
        "vmware",
        "virtualbox",
        "xen",
        "bochs",
        "parallels",
        "openstack",
        "amazon ec2",
        "google compute engine",
        "microsoft corporation\tvirtual machine",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn normalize_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The CPUs a qualified run reserves (ADR-0024): the benchmark cores with
/// their SMT siblings, inside one NUMA node. Every other online CPU is
/// background that must stay quiescent while the slice measures.
#[derive(Clone, Debug, PartialEq, Eq)]
struct IsolationPlan {
    /// Every logical CPU of a benchmark core, ascending.
    reserved: Vec<u32>,
    /// Reserved CPUs the benchmark itself does not run on.
    siblings: Vec<u32>,
    /// Online CPUs outside the reservation.
    background: Vec<u32>,
    /// The NUMA node holding every benchmark CPU.
    numa_node: u32,
}

fn isolation_plan(
    profile: &HostProfile,
    topology: &HardwareTopology,
) -> Result<IsolationPlan, HostError> {
    let mut cores = BTreeSet::new();
    for &id in &profile.benchmark_cpus {
        let cpu = topology
            .cpus
            .iter()
            .find(|cpu| cpu.id == id)
            .ok_or_else(|| HostError::Mismatch(format!("benchmark CPU {id} is not online")))?;
        cores.insert((cpu.package_id, cpu.core_id));
    }
    if cores.len() != usize::try_from(LINUX_PHYSICAL_CORES).unwrap_or(8) {
        return Err(HostError::Mismatch(format!(
            "benchmark CPU set spans {} distinct physical cores, expected {LINUX_PHYSICAL_CORES}",
            cores.len()
        )));
    }
    let numa_node = topology
        .numa_nodes
        .iter()
        .find(|node| {
            profile
                .benchmark_cpus
                .iter()
                .all(|cpu| node.cpus.contains(cpu))
        })
        .ok_or_else(|| {
            HostError::Mismatch(format!(
                "benchmark CPUs {} span more than one NUMA node",
                format_cpu_list(&profile.benchmark_cpus)
            ))
        })?
        .id;
    let mut plan = IsolationPlan {
        reserved: Vec::new(),
        siblings: Vec::new(),
        background: Vec::new(),
        numa_node,
    };
    for cpu in &topology.cpus {
        if cores.contains(&(cpu.package_id, cpu.core_id)) {
            plan.reserved.push(cpu.id);
            if !profile.benchmark_cpus.contains(&cpu.id) {
                plan.siblings.push(cpu.id);
            }
        } else {
            plan.background.push(cpu.id);
        }
    }
    for cpus in [&mut plan.reserved, &mut plan.siblings, &mut plan.background] {
        cpus.sort_unstable();
    }
    Ok(plan)
}

fn require_cpu_superset(name: &str, required: &[u32], found: &[u32]) -> Result<(), HostError> {
    let missing: Vec<u32> = required
        .iter()
        .copied()
        .filter(|cpu| !found.contains(cpu))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(HostError::Mismatch(format!(
            "{name} must include {}, missing {}",
            format_cpu_list(required),
            format_cpu_list(&missing)
        )))
    }
}

/// Per-CPU `(busy, total)` USER_HZ ticks from `/proc/stat`.
type CpuTicks = BTreeMap<u32, (u64, u64)>;

fn read_cpu_ticks(fs: &dyn FileSystem) -> Result<CpuTicks, HostError> {
    let stat = read_required(fs, Path::new("/proc/stat"), MAX_HOST_FILE_BYTES)?;
    let mut ticks = CpuTicks::new();
    for row in stat.lines() {
        // `cpuN user nice system idle iowait irq softirq steal guest
        // guest_nice`; the aggregate `cpu ` row has no id and is skipped.
        // Guest time is already inside user/nice, so only eight fields count.
        let Some((id, fields)) = row
            .strip_prefix("cpu")
            .and_then(|rest| rest.split_once(' '))
        else {
            continue;
        };
        let Ok(id) = id.parse::<u32>() else {
            continue;
        };
        let values = fields
            .split_whitespace()
            .take(8)
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| HostError::Probe(format!("/proc/stat cpu{id} row is malformed")))?;
        if values.len() < 5 {
            return Err(HostError::Probe(format!(
                "/proc/stat cpu{id} row lacks idle and iowait"
            )));
        }
        let total = values
            .iter()
            .fold(0_u64, |sum, value| sum.saturating_add(*value));
        let idle = values[3].saturating_add(values[4]);
        ticks.insert(id, (total.saturating_sub(idle), total));
    }
    Ok(ticks)
}

fn require_quiescence(
    plan: &IsolationPlan,
    before: &CpuTicks,
    after: &CpuTicks,
) -> Result<(), HostError> {
    for (label, cpus, ceiling) in [
        (
            "benchmark-core SMT siblings",
            &plan.siblings,
            MAX_SIBLING_BUSY_PERMILLE,
        ),
        (
            "background CPUs",
            &plan.background,
            MAX_BACKGROUND_BUSY_PERMILLE,
        ),
    ] {
        let (mut busy, mut total) = (0_u64, 0_u64);
        for cpu in cpus {
            let ((busy_before, total_before), (busy_after, total_after)) =
                match (before.get(cpu), after.get(cpu)) {
                    (Some(before), Some(after)) => (*before, *after),
                    _ => {
                        return Err(HostError::Probe(format!(
                            "/proc/stat lost CPU {cpu} during the run"
                        )));
                    }
                };
            busy = busy.saturating_add(busy_after.saturating_sub(busy_before));
            total = total.saturating_add(total_after.saturating_sub(total_before));
        }
        // One tick of grace per CPU absorbs USER_HZ quantization in short
        // windows; the fraction governs any window long enough to matter.
        let cpu_count = u64::try_from(cpus.len()).unwrap_or(u64::MAX);
        let allowance = (total.saturating_mul(ceiling) / 1000).saturating_add(cpu_count);
        if busy > allowance {
            return Err(HostError::Mismatch(format!(
                "{label} {} were busy {busy} of {total} ticks since the run began, over the \
                 {ceiling} permille ceiling",
                format_cpu_list(cpus)
            )));
        }
    }
    Ok(())
}

fn parse_status_cpu_list(status: &str, field: &str) -> Result<Vec<u32>, HostError> {
    let prefix = format!("{field}:");
    let value = status
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .ok_or_else(|| HostError::Probe(format!("/proc/self/status lacks {field}")))?;
    parse_cpu_list(value.trim()).map_err(|error| HostError::Probe(format!("bad {field}: {error}")))
}

fn require_initial_pid_namespace(status: &str) -> Result<(), HostError> {
    let row = status
        .lines()
        .find_map(|line| line.strip_prefix("NSpid:"))
        .ok_or_else(|| HostError::Probe("/proc/self/status lacks NSpid".to_owned()))?;
    let count = row.split_whitespace().count();
    if count == 1 {
        Ok(())
    } else {
        Err(HostError::Mismatch(format!(
            "process is nested in {count} PID namespaces"
        )))
    }
}

fn parse_unified_cgroup(input: &str) -> Result<String, HostError> {
    let mut rows = input.lines();
    let row = rows
        .next()
        .ok_or_else(|| HostError::Probe("/proc/self/cgroup is empty".to_owned()))?;
    if rows.next().is_some() {
        return Err(HostError::Mismatch(
            "host is not using a single unified cgroup-v2 hierarchy".to_owned(),
        ));
    }
    let path = row
        .strip_prefix("0::")
        .ok_or_else(|| HostError::Mismatch("host is not using cgroup v2".to_owned()))?;
    validate_scalar("live cgroup path", path, MAX_PATH_BYTES)?;
    if !path.starts_with('/') || path.contains("..") {
        return Err(HostError::Probe(
            "live cgroup path is not absolute and traversal-free".to_owned(),
        ));
    }
    Ok(path.to_owned())
}

fn parse_processes(input: &str) -> Result<Vec<u32>, HostError> {
    let mut processes = Vec::new();
    for row in input.lines() {
        if processes.len() == MAX_CGROUP_PROCESSES {
            return Err(HostError::Probe(format!(
                "cgroup.procs exceeds the {MAX_CGROUP_PROCESSES}-process limit"
            )));
        }
        processes.push(
            row.parse()
                .map_err(|_| HostError::Probe(format!("invalid cgroup pid {row:?}")))?,
        );
    }
    processes.sort_unstable();
    processes.dedup();
    Ok(processes)
}

fn parse_trimmed_u64(input: &str, name: &str) -> Result<u64, HostError> {
    input
        .trim()
        .parse()
        .map_err(|_| HostError::Probe(format!("{name} is not a u64")))
}

fn parse_load_milli(input: &str) -> Result<u64, HostError> {
    let value = input
        .split_whitespace()
        .next()
        .ok_or_else(|| HostError::Probe("/proc/loadavg is empty".to_owned()))?;
    let (whole, fractional) = value.split_once('.').unwrap_or((value, ""));
    let whole: u64 = whole
        .parse()
        .map_err(|_| HostError::Probe("load average has an invalid integer part".to_owned()))?;
    if fractional.len() > 3 || !fractional.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(HostError::Probe(
            "load average has more than three decimal places".to_owned(),
        ));
    }
    let mut fraction = fractional.to_owned();
    while fraction.len() < 3 {
        fraction.push('0');
    }
    let fraction: u64 = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse()
            .map_err(|_| HostError::Probe("load average fraction is invalid".to_owned()))?
    };
    whole
        .checked_mul(1000)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| HostError::Probe("load average exceeds u64".to_owned()))
}

#[derive(Debug)]
struct MountRecord {
    mount_point: PathBuf,
    fs_type: String,
    source: String,
}

fn find_mount(input: &str, artifact_path: &Path) -> Result<MountRecord, HostError> {
    if !artifact_path.is_absolute() {
        return Err(HostError::Profile(
            "artifact path must be absolute for storage attestation".to_owned(),
        ));
    }
    let mut best: Option<MountRecord> = None;
    for line in input.lines() {
        let Some((before, after)) = line.split_once(" - ") else {
            return Err(HostError::Probe("malformed mountinfo row".to_owned()));
        };
        let fields: Vec<_> = before.split_whitespace().collect();
        let tail: Vec<_> = after.split_whitespace().collect();
        if fields.len() < 6 || tail.len() < 2 {
            return Err(HostError::Probe("truncated mountinfo row".to_owned()));
        }
        let mount_point = PathBuf::from(decode_mount_field(fields[4])?);
        if !artifact_path.starts_with(&mount_point) {
            continue;
        }
        let candidate = MountRecord {
            mount_point,
            fs_type: tail[0].to_owned(),
            source: decode_mount_field(tail[1])?,
        };
        if best.as_ref().is_none_or(|current| {
            candidate.mount_point.components().count() > current.mount_point.components().count()
        }) {
            best = Some(candidate);
        }
    }
    best.ok_or_else(|| HostError::Probe("artifact path has no mountinfo record".to_owned()))
}

fn decode_mount_field(value: &str) -> Result<String, HostError> {
    let mut output = String::new();
    let mut bytes = value.bytes().peekable();
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            output.push(char::from(byte));
            continue;
        }
        let digits: String = bytes.by_ref().take(3).map(char::from).collect();
        let decoded = match digits.as_str() {
            "040" => ' ',
            "011" => '\t',
            "012" => '\n',
            "134" => '\\',
            _ => {
                return Err(HostError::Probe(format!(
                    "unsupported mountinfo escape \\{digits}"
                )));
            }
        };
        output.push(decoded);
    }
    Ok(output)
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update(&(bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_platform::fs::VirtualFs;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn digest(value: &str) -> Digest {
        sha256(value.as_bytes())
    }

    fn crate_file(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    fn reviewed_host_profiles() -> BTreeSet<String> {
        std::fs::read_dir(crate_file(REVIEWED_HOST_PROFILE_DIR))
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| Some(entry.path().file_stem()?.to_str()?.to_owned()))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn committed_baselines_claim_no_unreviewed_host_qualification() {
        // fm-5wq.32: the PG-8 fixtures once claimed bare_metal and isolated for
        // the shared dev host, which has no reviewed profile.
        let reviewed = reviewed_host_profiles();
        let mut checked = 0;
        for entry in std::fs::read_dir(crate_file("tests/artifacts/perf")).expect("perf artifacts")
        {
            let path = entry
                .expect("perf artifact entry")
                .path()
                .join("baseline.tsv");
            if !path.is_file() {
                continue;
            }
            let relative = path
                .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                .expect("inside the crate")
                .display()
                .to_string();
            let text = std::fs::read_to_string(&path).expect("baseline text");
            let baseline = Baseline::from_tsv(&text).expect("committed baseline parses");
            let verdict = check_baseline_host_claims(&relative, &baseline, &reviewed);
            println!(
                "{relative}\tprofile_id={}\tbare_metal={}\tisolated={}\tverdict={}",
                baseline.key.profile_id,
                baseline.key.bare_metal,
                baseline.key.isolated,
                if verdict.is_ok() { "ok" } else { "refused" }
            );
            verdict.expect("committed baseline host claims");
            checked += 1;
        }
        assert!(checked >= 4, "found {checked} committed baselines");
    }

    #[test]
    fn a_planted_overclaimed_baseline_is_refused_by_name() {
        let path = "tests/artifacts/perf/planted/pg8-overclaimed-baseline.tsv";
        let text = std::fs::read_to_string(crate_file(path)).expect("planted fixture");
        let baseline = Baseline::from_tsv(&text).expect("planted fixture parses");
        let message = check_baseline_host_claims(path, &baseline, &BTreeSet::new())
            .expect_err("an unreviewed qualification claim is refused")
            .to_string();
        assert!(
            message.contains(path)
                && message.contains("bare_metal and isolated")
                && message.contains("pg8-shared-dev-host"),
            "{message}"
        );
        let reviewed = BTreeSet::from(["pg8-shared-dev-host".to_owned()]);
        check_baseline_host_claims(path, &baseline, &reviewed)
            .expect("a reviewed profile admits the same claim");
    }

    fn profile_text() -> String {
        format!(
            "{HOST_PROFILE_SCHEMA}\nprofile_id\tlinux-8c-test-v1\nplatform\tlinux-x86_64\n\
             os_release_digest\t{}\nkernel_release\t6.12.1\ncpu_identity_digest\t{}\n\
             dmi_identity_digest\t{}\ntopology_digest\t{}\nbenchmark_cpus\t0,1,2,3,4,5,6,7\n\
             cgroup_path\t/fmn-benchmark\ngovernor\tperformance\n\
             boost_path\t/sys/devices/system/cpu/cpufreq/boost\nboost_value\t0\n\
             thermal_path\t/sys/class/thermal/thermal_zone0/temp\n\
             max_temperature_millicelsius\t70000\nmax_load_milli\t500\n\
             storage_mount\t/data\nstorage_fs\text4\nstorage_source_digest\t{}\n",
            digest("NAME=Test\n"),
            digest("cpu"),
            digest("dmi"),
            digest("topology"),
            digest("/dev/nvme0n1p1"),
        )
    }

    /// The canonical 8-core, single-node, SMT-free host.
    fn synthetic_linux_fs(pid: u32) -> VirtualFs {
        synthetic_host_fs(pid, 8, 1, 1)
    }

    /// A bare-metal host of `cores` physical cores with `smt` threads each,
    /// split evenly across `nodes` NUMA nodes. Linux numbering: CPU
    /// `thread * cores + core`, so core 0's SMT sibling is CPU `cores`. The
    /// benchmark slice is cores 0-7 on CPUs 0-7, isolated together with its
    /// siblings and bound to node 0.
    fn synthetic_host_fs(pid: u32, cores: u32, smt: u32, nodes: u32) -> VirtualFs {
        let cpus = cores * smt;
        let fs = VirtualFs::new();
        fs.insert("/etc/os-release", b"NAME=Test\n".to_vec());
        fs.insert("/proc/sys/kernel/osrelease", b"6.12.1\n".to_vec());
        let mut cpuinfo = String::new();
        for cpu in 0..cpus {
            writeln!(&mut cpuinfo, "processor : {cpu}").expect("cpuinfo");
            writeln!(&mut cpuinfo, "vendor_id : GenuineTest").expect("cpuinfo");
            writeln!(&mut cpuinfo, "cpu family : 1").expect("cpuinfo");
            writeln!(&mut cpuinfo, "model : 2").expect("cpuinfo");
            writeln!(&mut cpuinfo, "model name : Test CPU").expect("cpuinfo");
            writeln!(&mut cpuinfo, "stepping : 3").expect("cpuinfo");
            writeln!(&mut cpuinfo, "microcode : 0x4").expect("cpuinfo");
            writeln!(&mut cpuinfo, "flags : sse avx").expect("cpuinfo");
        }
        fs.insert("/proc/cpuinfo", cpuinfo.into_bytes());
        for (leaf, value) in [
            ("sys_vendor", "Test Vendor"),
            ("product_name", "Bare Metal Workstation"),
            ("product_version", "1"),
            ("board_vendor", "Test Vendor"),
            ("board_name", "Board 8C"),
            ("board_version", "1"),
            ("bios_version", "1.0"),
        ] {
            fs.insert(
                Path::new("/sys/class/dmi/id").join(leaf),
                format!("{value}\n").into_bytes(),
            );
        }
        fs.insert(
            "/sys/devices/system/cpu/online",
            format!("0-{}\n", cpus - 1).into_bytes(),
        );
        for cpu in 0..cpus {
            fs.insert(
                format!("/sys/devices/system/cpu/cpu{cpu}/topology/physical_package_id"),
                b"0\n".to_vec(),
            );
            fs.insert(
                format!("/sys/devices/system/cpu/cpu{cpu}/topology/core_id"),
                format!("{}\n", cpu % cores).into_bytes(),
            );
            fs.insert(
                format!("/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_governor"),
                b"performance\n".to_vec(),
            );
        }
        if nodes > 1 {
            let cores_per_node = cores / nodes;
            for node in 0..nodes {
                let members: Vec<u32> = (0..cpus)
                    .filter(|cpu| (cpu % cores) / cores_per_node == node)
                    .collect();
                fs.insert(
                    format!("/sys/devices/system/node/node{node}/cpulist"),
                    format!("{}\n", format_cpu_list(&members)).into_bytes(),
                );
            }
        }
        fs.insert(
            "/proc/self/status",
            format!("Name:\tfmn-perf\nNSpid:\t{pid}\nCpus_allowed_list:\t0-7\n").into_bytes(),
        );
        let reserved: Vec<u32> = (0..cpus).filter(|cpu| cpu % cores < 8).collect();
        for path in [
            "/sys/devices/system/cpu/isolated",
            "/sys/devices/system/cpu/nohz_full",
            "/sys/devices/system/cpu/rcu_nocbs",
        ] {
            fs.insert(
                path,
                format!("{}\n", format_cpu_list(&reserved)).into_bytes(),
            );
        }
        fs.insert("/proc/self/cgroup", b"0::/fmn-benchmark\n".to_vec());
        fs.insert(
            "/sys/fs/cgroup/fmn-benchmark/cpuset.cpus.effective",
            b"0-7\n".to_vec(),
        );
        fs.insert(
            "/sys/fs/cgroup/fmn-benchmark/cpuset.mems.effective",
            b"0\n".to_vec(),
        );
        fs.insert(
            "/sys/fs/cgroup/fmn-benchmark/cgroup.procs",
            format!("{pid}\n").into_bytes(),
        );
        fs.insert("/sys/devices/system/cpu/cpufreq/boost", b"0\n".to_vec());
        fs.insert("/sys/class/thermal/thermal_zone0/temp", b"42000\n".to_vec());
        fs.insert("/proc/loadavg", b"0.25 0.20 0.10 1/100 7\n".to_vec());
        fs.insert(
            "/proc/self/mountinfo",
            b"1 0 8:1 / / rw - ext4 /dev/root rw\n2 1 8:2 / /data rw - ext4 /dev/nvme0n1p1 rw\n"
                .to_vec(),
        );
        write_proc_stat(&fs, cpus, 0, |_| 0);
        fs
    }

    /// Rewrite `/proc/stat`: every CPU at a 150-tick busy and 10,010-tick idle
    /// base, plus `idle` more idle ticks and `busy(cpu)` more busy ticks.
    fn write_proc_stat(fs: &VirtualFs, cpus: u32, idle: u64, busy: impl Fn(u32) -> u64) {
        let mut stat = String::from("cpu  0 0 0 0 0 0 0 0 0 0\n");
        for cpu in 0..cpus {
            writeln!(
                &mut stat,
                "cpu{cpu} {} 0 50 {} 10 0 0 0 0 0",
                100 + busy(cpu),
                10_000 + idle
            )
            .expect("stat");
        }
        stat.push_str("intr 0\nctxt 0\n");
        fs.insert("/proc/stat", stat.into_bytes());
    }

    fn synthetic_profile(fs: &VirtualFs) -> HostProfile {
        let os_release = read_required(fs, Path::new("/etc/os-release"), MAX_HOST_FILE_BYTES)
            .expect("os release");
        let cpuinfo =
            read_required(fs, Path::new("/proc/cpuinfo"), MAX_HOST_FILE_BYTES).expect("cpuinfo");
        let topology = HardwareTopology::detect_linux(fs).expect("topology");
        HostProfile {
            profile_id: "linux-8c-test-v1".to_owned(),
            platform: HostPlatform::LinuxX86_64,
            os_release_digest: sha256(os_release.as_bytes()),
            kernel_release: "6.12.1".to_owned(),
            cpu_identity_digest: sha256(
                canonical_cpu_identity(&cpuinfo)
                    .expect("cpu identity")
                    .as_bytes(),
            ),
            dmi_identity_digest: sha256(
                canonical_dmi_identity(fs).expect("dmi identity").as_bytes(),
            ),
            topology_digest: sha256(topology.snapshot_text().as_bytes()),
            benchmark_cpus: (0..8).collect(),
            cgroup_path: "/fmn-benchmark".to_owned(),
            governor: "performance".to_owned(),
            boost_path: PathBuf::from("/sys/devices/system/cpu/cpufreq/boost"),
            boost_value: "0".to_owned(),
            thermal_path: PathBuf::from("/sys/class/thermal/thermal_zone0/temp"),
            max_temperature_millicelsius: 70_000,
            max_load_milli: 500,
            storage_mount: PathBuf::from("/data"),
            storage_fs: "ext4".to_owned(),
            storage_source_digest: sha256(b"/dev/nvme0n1p1"),
        }
    }

    #[test]
    fn profile_round_trips_and_rejects_unknown_fields() {
        let profile = HostProfile::from_tsv(&profile_text()).expect("profile");
        assert_eq!(
            HostProfile::from_tsv(&profile.to_tsv()).expect("canonical profile"),
            profile
        );
        let bad = profile_text() + "surprise\tvalue\n";
        assert!(
            HostProfile::from_tsv(&bad)
                .expect_err("unknown field")
                .to_string()
                .contains("unknown")
        );
    }

    #[test]
    fn profile_parser_enforces_resource_and_path_bounds() {
        let oversized = format!("{HOST_PROFILE_SCHEMA}\n{}", "x".repeat(MAX_PROFILE_BYTES));
        assert!(HostProfile::from_tsv(&oversized).is_err());
        let bad = profile_text().replace(
            "/sys/class/thermal/thermal_zone0/temp",
            "/tmp/caller-controlled-temperature",
        );
        assert!(
            HostProfile::from_tsv(&bad)
                .expect_err("unsafe thermal path")
                .to_string()
                .contains("sysfs")
        );
    }

    #[test]
    fn measurement_identity_never_trusts_baseline_booleans_without_token() {
        let profile = HostProfile::from_tsv(&profile_text()).expect("profile");
        let key = BenchmarkKey {
            profile_id: profile.profile_id.clone(),
            build_profile: crate::perf::COMPILED_CARGO_PROFILE.to_owned(),
            host_fingerprint: profile.digest(),
            toolchain_fingerprint: compiled_toolchain_fingerprint(),
            suite_lock_digest: compiled_suite_lock_digest(),
            benchmark_definition: digest("definition"),
            gate: crate::perf::GateId::Pg2,
            scenario: "fill-canonical".to_owned(),
            unit: crate::perf::MetricUnit::MegaPixelsPerSecondMilli,
            engine: "fast-cpu".to_owned(),
            tier: "portable".to_owned(),
            thread_profile: "fixed-8".to_owned(),
            execution_plan_digest: digest("plan"),
            config_digest: digest("config"),
            cache_state: "warm".to_owned(),
            output_mode: "raw".to_owned(),
            external_tool_fingerprint: None,
            bare_metal: true,
            isolated: true,
        };
        let (found, evidence) = measurement_identity(&key, None).expect("calibration identity");
        assert!(!found.bare_metal);
        assert!(!found.isolated);
        assert!(evidence.is_empty());
    }

    #[test]
    fn linux_attestation_rejects_virtualization_before_qualification() {
        let fs = VirtualFs::new();
        fs.insert("/etc/os-release", b"NAME=Test\n".to_vec());
        fs.insert("/proc/sys/kernel/osrelease", b"6.12.1\n".to_vec());
        fs.insert(
            "/proc/cpuinfo",
            b"processor : 0\nmodel name : Test\nflags : sse hypervisor avx\n".to_vec(),
        );
        let profile = HostProfile::from_tsv(&profile_text()).expect("profile");
        let error = attest_linux_host(
            &profile,
            &fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/host.tsv".to_owned(),
            7,
        )
        .expect_err("virtualized host");
        assert!(error.to_string().contains("hypervisor"));
    }

    #[test]
    fn linux_attestation_issues_opaque_token_and_binds_all_key_identities() {
        let pid = 71;
        let fs = synthetic_linux_fs(pid);
        let profile = synthetic_profile(&fs);
        let qualification = attest_linux_host(
            &profile,
            &fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
        .expect("qualified synthetic host");
        assert!(qualification.attestation_tsv().contains("bare_metal\ttrue"));
        assert!(
            qualification
                .attestation_tsv()
                .contains(&format!("monitor_policy\t{HOST_MONITOR_POLICY}"))
        );
        assert!(qualification.attestation_tsv().contains(&format!(
            "monitor_interval_millis\t{HOST_MONITOR_INTERVAL_MILLIS}"
        )));
        assert_eq!(qualification.evidence().kind, EvidenceKind::HostAttestation);

        let mut key = BenchmarkKey {
            profile_id: profile.profile_id.clone(),
            build_profile: crate::perf::COMPILED_CARGO_PROFILE.to_owned(),
            host_fingerprint: profile.digest(),
            toolchain_fingerprint: compiled_toolchain_fingerprint(),
            suite_lock_digest: compiled_suite_lock_digest(),
            benchmark_definition: digest("definition"),
            gate: crate::perf::GateId::Pg2,
            scenario: "fill-canonical".to_owned(),
            unit: crate::perf::MetricUnit::MegaPixelsPerSecondMilli,
            engine: "fast-cpu".to_owned(),
            tier: "portable".to_owned(),
            thread_profile: "fixed-8".to_owned(),
            execution_plan_digest: digest("plan"),
            config_digest: digest("config"),
            cache_state: "warm".to_owned(),
            output_mode: "raw".to_owned(),
            external_tool_fingerprint: None,
            bare_metal: false,
            isolated: false,
        };
        let (qualified, evidence) =
            measurement_identity(&key, Some(&qualification)).expect("qualified key");
        assert!(qualified.bare_metal && qualified.isolated);
        assert_eq!(evidence, vec![qualification.evidence().clone()]);

        key.toolchain_fingerprint = digest("caller-forged-toolchain");
        assert!(
            measurement_identity(&key, Some(&qualification))
                .expect_err("identity mismatch")
                .to_string()
                .contains("toolchain_fingerprint")
        );
    }

    #[test]
    fn linux_attestation_rejects_peer_processes_and_hot_hosts() {
        let pid = 71;
        let fs = synthetic_linux_fs(pid);
        let profile = synthetic_profile(&fs);
        fs.insert(
            "/sys/fs/cgroup/fmn-benchmark/cgroup.procs",
            format!("{pid}\n72\n").into_bytes(),
        );
        let peer_error = attest_linux_host(
            &profile,
            &fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
        .expect_err("peer process must invalidate isolation");
        assert!(peer_error.to_string().contains("only measurement pid"));

        fs.insert(
            "/sys/fs/cgroup/fmn-benchmark/cgroup.procs",
            format!("{pid}\n").into_bytes(),
        );
        fs.insert("/sys/class/thermal/thermal_zone0/temp", b"70001\n".to_vec());
        let thermal_error = attest_linux_host(
            &profile,
            &fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
        .expect_err("hot host must fail");
        assert!(
            thermal_error
                .to_string()
                .contains("exceeds profile ceiling")
        );
    }

    #[test]
    fn postflight_revalidation_catches_environment_drift() {
        let pid = 71;
        let fs = synthetic_linux_fs(pid);
        let profile = synthetic_profile(&fs);
        let qualification = attest_linux_host(
            &profile,
            &fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
        .expect("preflight");
        qualification
            .revalidate_linux_with_fs(&fs)
            .expect("stable postflight");

        fs.insert("/proc/loadavg", b"0.501 0.20 0.10 1/100 7\n".to_vec());
        let error = qualification
            .revalidate_linux_with_fs(&fs)
            .expect_err("load drift must invalidate the run");
        assert!(error.to_string().contains("exceeds profile ceiling"));
    }

    #[test]
    fn live_monitor_final_sample_catches_drift_before_publication() {
        let pid = 71;
        let fs = Arc::new(synthetic_linux_fs(pid));
        let profile = synthetic_profile(fs.as_ref());
        let qualification = attest_linux_host(
            &profile,
            fs.as_ref(),
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
        .expect("preflight");
        let monitor = qualification
            .start_live_monitor_with_fs(fs.clone(), Duration::from_secs(60))
            .expect("monitor");
        fs.insert("/sys/class/thermal/thermal_zone0/temp", b"70001\n".to_vec());
        let error = monitor
            .stop_and_validate()
            .expect_err("final sample must reject drift");
        assert!(error.to_string().contains("exceeds profile ceiling"));
    }

    #[test]
    fn live_monitor_retains_a_transient_failure_after_recovery() {
        let failing = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&failing);
        let (seen_tx, seen_rx) = mpsc::sync_channel(1);
        let monitor = HostMonitor::start_with_probe(Duration::from_millis(1), move || {
            if observed.load(Ordering::SeqCst) {
                let _ = seen_tx.try_send(());
                Err(HostError::Mismatch("synthetic transient".to_owned()))
            } else {
                Ok(())
            }
        })
        .expect("monitor");

        failing.store(true, Ordering::SeqCst);
        seen_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("monitor observed transient state");
        failing.store(false, Ordering::SeqCst);

        let error = monitor
            .stop_and_validate()
            .expect_err("a recovered transient must still invalidate the run");
        assert!(error.to_string().contains("synthetic transient"));
    }

    fn attest(
        fs: &VirtualFs,
        profile: &HostProfile,
        pid: u32,
    ) -> Result<HostQualification, HostError> {
        attest_linux_host(
            profile,
            fs,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv".to_owned(),
            pid,
        )
    }

    #[test]
    fn an_isolated_slice_of_a_larger_smt_numa_machine_qualifies() {
        // ADR-0024: 16 cores × 2 threads over two nodes. The verifier used to
        // refuse this on machine size alone.
        let pid = 71;
        let fs = Arc::new(synthetic_host_fs(pid, 16, 2, 2));
        let profile = synthetic_profile(fs.as_ref());
        let qualification = attest(fs.as_ref(), &profile, pid).expect("isolated slice qualifies");
        let attestation = qualification.attestation_tsv();
        assert!(attestation.contains("reserved_cpus\t0,1,2,3,4,5,6,7,16,17,18,19,20,21,22,23\n"));
        assert!(attestation.contains("numa_node\t0\n"));
        assert!(attestation.contains(&format!(
            "background_busy_ceiling_permille\t{MAX_BACKGROUND_BUSY_PERMILLE}\n"
        )));
        assert_eq!(qualification.plan.siblings, (16..24).collect::<Vec<_>>());
        assert_eq!(
            qualification.plan.background,
            (8..16).chain(24..32).collect::<Vec<_>>()
        );

        // Saturated benchmark cores, idle siblings and a ~1% background pass
        // the whole-run quiescence check.
        let monitor = qualification
            .start_live_monitor_with_fs(fs.clone(), Duration::from_secs(60))
            .expect("monitor");
        write_proc_stat(&fs, 32, 1_000, |cpu| match cpu {
            0..8 => 1_000,
            16..24 => 0,
            _ => 10,
        });
        assert_eq!(monitor.stop_and_validate().expect("quiet machine"), 1);
    }

    #[test]
    fn live_monitor_refuses_busy_siblings_and_a_busy_background() {
        let pid = 71;
        for (busy_cpus, expected) in [(16..24, "SMT siblings"), (8..16, "background CPUs")] {
            let fs = Arc::new(synthetic_host_fs(pid, 16, 2, 2));
            let profile = synthetic_profile(fs.as_ref());
            let qualification = attest(fs.as_ref(), &profile, pid).expect("preflight");
            let monitor = qualification
                .start_live_monitor_with_fs(fs.clone(), Duration::from_secs(60))
                .expect("monitor");
            write_proc_stat(&fs, 32, 1_000, |cpu| {
                if busy_cpus.contains(&cpu) { 200 } else { 0 }
            });
            let error = monitor
                .stop_and_validate()
                .expect_err("contended slice must not publish");
            assert!(error.to_string().contains(expected), "{expected}: {error}");
        }
    }

    #[test]
    fn quiescence_grants_one_tick_per_cpu_for_short_windows_only() {
        let fs = synthetic_host_fs(71, 16, 2, 2);
        let profile = synthetic_profile(&fs);
        let plan = isolation_plan(
            &profile,
            &HardwareTopology::detect_linux(&fs).expect("topology"),
        )
        .expect("plan");
        let before = read_cpu_ticks(&fs).expect("before");
        // A 250 ms window at USER_HZ 100: one stray tick per sibling passes.
        write_proc_stat(&fs, 32, 24, |cpu| u64::from((16..24).contains(&cpu)));
        require_quiescence(&plan, &before, &read_cpu_ticks(&fs).expect("short")).expect("grace");
        // Three ticks each across a 1 s window (3%) exceed 1% plus the grace.
        write_proc_stat(&fs, 32, 97, |cpu| 3 * u64::from((16..24).contains(&cpu)));
        assert!(
            require_quiescence(&plan, &before, &read_cpu_ticks(&fs).expect("long"))
                .expect_err("sustained sibling work")
                .to_string()
                .contains("SMT siblings")
        );
        // A CPU that disappears mid-run is a probe failure, never a pass.
        write_proc_stat(&fs, 31, 98, |_| 0);
        assert!(
            require_quiescence(&plan, &before, &read_cpu_ticks(&fs).expect("offline"))
                .expect_err("lost CPU")
                .to_string()
                .contains("lost CPU 31")
        );
    }

    #[test]
    fn every_isolation_list_must_cover_the_slice_siblings() {
        let pid = 71;
        for path in [
            "/sys/devices/system/cpu/isolated",
            "/sys/devices/system/cpu/nohz_full",
            "/sys/devices/system/cpu/rcu_nocbs",
        ] {
            let fs = synthetic_host_fs(pid, 16, 2, 2);
            let profile = synthetic_profile(&fs);
            fs.insert(path, b"0-7\n".to_vec());
            let error = attest(&fs, &profile, pid).expect_err("uncovered siblings");
            assert!(
                error
                    .to_string()
                    .contains("missing 16,17,18,19,20,21,22,23"),
                "{path}: {error}"
            );
        }
        // Isolating more than the slice is admitted.
        let fs = synthetic_host_fs(pid, 16, 2, 2);
        let profile = synthetic_profile(&fs);
        fs.insert("/sys/devices/system/cpu/isolated", b"0-31\n".to_vec());
        attest(&fs, &profile, pid).expect("superset isolation");
    }

    #[test]
    fn the_slice_must_sit_in_one_numa_node_with_memory_bound_to_it() {
        let pid = 71;
        let fs = synthetic_host_fs(pid, 16, 2, 2);
        let mut profile = synthetic_profile(&fs);
        profile.benchmark_cpus = (4..12).collect();
        let error = attest(&fs, &profile, pid).expect_err("cross-node slice");
        assert!(error.to_string().contains("span more than one NUMA node"));

        let profile = synthetic_profile(&fs);
        for mems in ["1", "0-1"] {
            fs.insert(
                "/sys/fs/cgroup/fmn-benchmark/cpuset.mems.effective",
                format!("{mems}\n").into_bytes(),
            );
            let error = attest(&fs, &profile, pid).expect_err("unbound memory");
            assert!(
                error.to_string().contains("cgroup cpuset.mems"),
                "{mems}: {error}"
            );
        }
    }

    #[test]
    fn the_slice_is_exactly_eight_distinct_cores_on_any_machine() {
        let pid = 71;
        let fs = synthetic_host_fs(pid, 16, 2, 2);
        let mut profile = synthetic_profile(&fs);
        // CPU 16 is core 0's sibling, so this set covers seven cores.
        profile.benchmark_cpus = (0..7).chain([16]).collect();
        let error = attest(&fs, &profile, pid).expect_err("seven-core slice");
        assert!(
            error
                .to_string()
                .contains("spans 7 distinct physical cores")
        );

        let small = synthetic_host_fs(pid, 6, 1, 1);
        let profile = synthetic_profile(&small);
        let error = attest(&small, &profile, pid).expect_err("six-core machine");
        assert!(error.to_string().contains("benchmark CPU 6 is not online"));
    }

    #[test]
    fn macos_profile_is_explicitly_unsupported_without_native_introspection() {
        let mut profile = HostProfile::from_tsv(&profile_text()).expect("profile");
        profile.platform = HostPlatform::MacosAarch64;
        let error = attest_current_host(
            &profile,
            Path::new("/data/artifacts/raw.tsv"),
            "tests/artifacts/perf/run/host.tsv",
        )
        .expect_err("macOS fallback topology is not evidence");
        assert!(error.to_string().contains("safe native topology"));
    }

    #[test]
    fn mount_parser_uses_longest_matching_mount_and_decodes_fields() {
        let input = "1 0 8:1 / / rw - ext4 /dev/root rw\n\
                     2 1 8:2 / /data rw - xfs /dev/nvme\\040disk rw\n";
        let mount = find_mount(input, Path::new("/data/artifacts/raw.tsv")).expect("mount");
        assert_eq!(mount.mount_point, Path::new("/data"));
        assert_eq!(mount.fs_type, "xfs");
        assert_eq!(mount.source, "/dev/nvme disk");
    }
}
