//! The process capability: **the one sanctioned subprocess mechanism** (D2).
//!
//! ffmpeg is the only program the engine will ever spawn, and every rule of
//! the D2 security protocol that belongs to the *mechanism* lives here:
//!
//! - **Exact-image, argv-only interface.** [`ProcessSpec`] is an absolute
//!   native-image path plus an argument vector. `StdProcessRunner` delegates
//!   to asupersync's audited exact-image primitive: Unix uses `posix_spawn`
//!   with the absolute path (never `posix_spawnp`, `execvp`, or an ENOEXEC
//!   shell fallback), while Windows uses an explicit `CreateProcessW`
//!   application plus atomic Job Object assignment.
//! - **Environment allowlist.** The child's environment is cleared and
//!   rebuilt from [`ProcessSpec::env`] alone; nothing ambient leaks in.
//! - **Timeout.** [`ProcessSpec::timeout`] bounds wall-clock runtime; on
//!   expiry the child is killed and the outcome says so.
//! - **Output-size limits.** stdout and stderr are each capped at
//!   [`ProcessSpec::max_output_bytes`]; exceeding a cap kills the child with
//!   [`ProcessTermination::OutputLimitExceeded`] — a runaway encoder cannot
//!   fill the disk or the heap.
//! - **Bounded streaming stdin.** [`ProcessRunner::start`] yields a supervised
//!   session with explicit per-chunk and cumulative limits. The OS pipe
//!   supplies backpressure, cooperative cancellation interrupts a blocked
//!   write, and every terminal path reaps the child.
//! - **Audited ffmpeg discovery.** [`FfmpegLocator`] is deliberately not a
//!   generic executable finder. Its host implementation snapshots one explicit
//!   `PATH` value, rejects the complete search policy when any entry is empty,
//!   relative, or otherwise ambiguous, rejects interpreter scripts through a
//!   target-specific native-image check, and returns a canonical absolute
//!   [`FfmpegExecutable`] for the output boundary to fingerprint.
//!
//! - **Process-tree cancellation.** On Linux and macOS every child leads a
//!   fresh process group; on Windows every child starts atomically inside a
//!   kill-on-close Job Object. Every terminal path kills that complete tree.
//!   Targets without an audited exact-image mechanism are refused before spawn
//!   rather than silently weakening D2.
//!   Higher layers (job-scoped temp dirs and their `fm-yw7h` hardening,
//!   atomic publication, provenance fingerprinting) belong to the W8
//!   boundary, not the mechanism.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
// The io traits are used by the native-image attestation readers and the
// std runner's pipe plumbing; both are structurally absent on wasm32, and the
// latter is absent when the exact-process capability is not selected.
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
use std::io::Write as _;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read as _, Seek as _};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
use std::time::Instant;

/// The one executable identity discovery may return.
///
/// The path is canonical, absolute, UTF-8 representable for versioned
/// provenance, and names a regular file accepted by the versioned host-native
/// container policy at selection time. fmn-output still owns byte hashing,
/// private-copy binding, and protocol probing: this type closes ambient
/// search, known interpreter scripts, and symlink retargeting, not every
/// safe-`std` pathname race or every loader-specific rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FfmpegExecutable {
    canonical_path: PathBuf,
}

impl FfmpegExecutable {
    /// The canonical absolute path that fmn-output must fingerprint.
    #[must_use]
    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    /// Open the currently named source and validate its size, regular-file
    /// identity, executable permissions, and host-native container through
    /// that same handle.
    ///
    /// fmn-output consumes this operation before hashing or copying. A source
    /// replacement after selection therefore cannot turn the typed locator
    /// result back into an unchecked pathname.
    ///
    /// # Errors
    ///
    /// [`FfmpegLocatorError`] when the selected path no longer names the
    /// bounded host-native executable image for which this capability exists.
    pub fn open_current(
        &self,
    ) -> Result<(std::fs::File, NativeImageAttestation), FfmpegLocatorError> {
        open_validated_ffmpeg_image(&self.canonical_path)
    }

    /// Validate an already-open private copy through the exact handle that
    /// fmn-output will retain across spawn.
    ///
    /// This is deliberately a method on the ffmpeg-only locator token rather
    /// than a generic executable validator.
    ///
    /// # Errors
    ///
    /// [`FfmpegLocatorError`] when `file` is not a bounded regular executable
    /// or its container is malformed, non-executable, or for another host
    /// architecture.
    pub fn attest_private_copy(
        &self,
        file: &mut std::fs::File,
        path: &Path,
    ) -> Result<NativeImageAttestation, FfmpegLocatorError> {
        validate_open_ffmpeg_image(file, path)
    }

    /// Reopen a private-copy pathname and validate the exact object currently
    /// selected by that path.
    ///
    /// fmn-output uses this immediately around spawn in addition to retaining
    /// and attesting its original create-new handle. Safe `std` still cannot
    /// make pathname validation atomic with the OS loader, so the documented
    /// same-identity race qualification remains.
    ///
    /// # Errors
    ///
    /// [`FfmpegLocatorError`] when the path does not currently name a bounded
    /// host-native executable image.
    pub fn open_private_copy(
        &self,
        path: &Path,
    ) -> Result<(std::fs::File, NativeImageAttestation), FfmpegLocatorError> {
        open_validated_ffmpeg_image(path)
    }
}

/// Version of the structural native-image policy recorded in provenance.
pub const NATIVE_IMAGE_POLICY_VERSION: u32 = 2;

/// Maximum source/private executable size accepted by the ffmpeg boundary.
///
/// The limit is intentionally far above ordinary static ffmpeg builds while
/// preventing a FIFO, device, or attacker-sized sparse file from becoming an
/// unbounded hash/copy operation.
pub const MAX_FFMPEG_EXECUTABLE_BYTES: u64 = 1 << 30;

/// Native executable-container family structurally attested for one exact file
/// handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeExecutableFormat {
    /// A 64-bit ELF executable or position-independent executable.
    Elf64,
    /// A thin 64-bit Mach-O executable.
    MachO64,
    /// A fat/universal Mach-O containing a valid host slice.
    MachOUniversal,
    /// A PE32+ executable image.
    Pe32Plus,
}

/// Host architecture proved by native-image validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeExecutableArchitecture {
    /// AMD64 / x86-64.
    X86_64,
    /// AArch64 / arm64.
    Aarch64,
}

/// Structural evidence for the exact opened executable image.
///
/// This records acceptance by a bounded, versioned parser. It is deliberately
/// not an assertion that the host loader must accept the image. The paired
/// exact-image process capability guarantees that a later loader refusal stays
/// a spawn error rather than becoming an interpreter fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeImageAttestation {
    /// Native container family.
    pub format: NativeExecutableFormat,
    /// Architecture of the executable or selected universal slice.
    pub architecture: NativeExecutableArchitecture,
    /// Length observed on the validated handle.
    pub file_bytes: u64,
    /// Parser/policy version used to issue this evidence.
    pub policy_version: u32,
}

/// Typed failures from the ffmpeg-only executable locator.
#[derive(Debug)]
pub enum FfmpegLocatorError {
    /// A relative configured value was not the one fixed search name.
    UnsupportedConfiguredName {
        /// The refused configured value.
        configured: PathBuf,
    },
    /// No explicit search-path snapshot was supplied.
    SearchPathUnavailable,
    /// Search semantics are unavailable on this target.
    SearchUnsupported {
        /// The target operating-system identity.
        platform: &'static str,
    },
    /// The raw search list is structurally malformed.
    MalformedSearchPath {
        /// Stable reason that does not lossily echo hostile bytes.
        reason: &'static str,
    },
    /// One search entry was empty (and therefore must not mean the cwd).
    EmptySearchEntry {
        /// Zero-based entry position.
        index: usize,
    },
    /// One search entry could not be represented in versioned provenance.
    NonUtf8SearchEntry {
        /// Zero-based entry position.
        index: usize,
    },
    /// One search entry contained terminal/control characters.
    ControlSearchEntry {
        /// Zero-based entry position.
        index: usize,
    },
    /// The hostile-input resource bound was exceeded.
    SearchPathLimit {
        /// Stable bounded resource.
        resource: &'static str,
        /// Observed size.
        got: usize,
        /// Accepted maximum.
        max: usize,
    },
    /// One search entry was not absolute.
    RelativeSearchEntry {
        /// Zero-based entry position.
        index: usize,
        /// The refused entry.
        entry: PathBuf,
    },
    /// One search entry contained lexical parent traversal.
    ParentTraversalSearchEntry {
        /// Zero-based entry position.
        index: usize,
        /// The refused entry.
        entry: PathBuf,
    },
    /// A candidate could not be inspected or canonicalized.
    CandidateIo {
        /// Candidate path at the failing operation.
        candidate: PathBuf,
        /// Stable operation identity.
        operation: &'static str,
        /// Host I/O diagnostic.
        err: io::Error,
    },
    /// Canonicalization returned an identity unsuitable for provenance.
    InvalidCanonicalIdentity {
        /// The canonical path.
        path: PathBuf,
        /// Stable reason.
        reason: &'static str,
    },
    /// An explicitly configured path was not a regular executable file.
    NotExecutable {
        /// The inspected canonical path.
        path: PathBuf,
    },
    /// An explicitly configured file was not a host-native executable image.
    UnsupportedExecutableFormat {
        /// The inspected canonical path.
        path: PathBuf,
    },
    /// A candidate exceeded the bounded executable-image policy.
    ExecutableSizeLimit {
        /// The inspected path.
        path: PathBuf,
        /// Observed file length.
        bytes: u64,
        /// Accepted maximum.
        max: u64,
    },
    /// Host-native executable-image validation is unsupported on this target.
    NativeImageUnsupported {
        /// The target operating-system identity.
        platform: &'static str,
    },
    /// No host-native executable ffmpeg candidate existed in the search list.
    NotFound,
}

impl fmt::Display for FfmpegLocatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedConfiguredName { configured } => write!(
                f,
                "relative ffmpeg configuration {:?} is not the fixed ffmpeg search name",
                configured.as_os_str()
            ),
            Self::SearchPathUnavailable => {
                f.write_str("ffmpeg search requested without an explicit PATH snapshot")
            }
            Self::SearchUnsupported { platform } => {
                write!(f, "ffmpeg PATH search is unsupported on {platform}")
            }
            Self::MalformedSearchPath { reason } => {
                write!(f, "ffmpeg search PATH is malformed: {reason}")
            }
            Self::EmptySearchEntry { index } => write!(
                f,
                "ffmpeg search PATH entry {index} is empty; cwd lookup is forbidden"
            ),
            Self::NonUtf8SearchEntry { index } => write!(
                f,
                "ffmpeg search PATH entry {index} is not valid UTF-8; lossy executable provenance is forbidden"
            ),
            Self::ControlSearchEntry { index } => write!(
                f,
                "ffmpeg search PATH entry {index} contains a control character"
            ),
            Self::SearchPathLimit { resource, got, max } => write!(
                f,
                "ffmpeg search PATH {resource} {got} exceeds the limit {max}"
            ),
            Self::RelativeSearchEntry { index, .. } => {
                write!(f, "ffmpeg search PATH entry {index} is relative")
            }
            Self::ParentTraversalSearchEntry { index, .. } => write!(
                f,
                "ffmpeg search PATH entry {index} contains parent traversal"
            ),
            Self::CandidateIo {
                candidate,
                operation,
                err,
            } => write!(
                f,
                "cannot {operation} ffmpeg candidate {:?}: {err}",
                candidate.as_os_str()
            ),
            Self::InvalidCanonicalIdentity { path, reason } => {
                write!(
                    f,
                    "invalid canonical ffmpeg identity {:?}: {reason}",
                    path.as_os_str()
                )
            }
            Self::NotExecutable { path } => write!(
                f,
                "configured ffmpeg path {:?} is not a regular executable file",
                path.as_os_str()
            ),
            Self::UnsupportedExecutableFormat { path } => write!(
                f,
                "configured ffmpeg path {:?} is not a host-native executable image; interpreter scripts are forbidden",
                path.as_os_str()
            ),
            Self::ExecutableSizeLimit { path, bytes, max } => write!(
                f,
                "configured ffmpeg image {:?} is {bytes} bytes, exceeding the {max}-byte executable limit",
                path.as_os_str()
            ),
            Self::NativeImageUnsupported { platform } => write!(
                f,
                "host-native ffmpeg executable-image validation is unsupported on {platform}"
            ),
            Self::NotFound => f.write_str(
                "no regular host-native executable ffmpeg was found in the validated PATH snapshot",
            ),
        }
    }
}

impl std::error::Error for FfmpegLocatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CandidateIo { err, .. } => Some(err),
            _ => None,
        }
    }
}

/// The ffmpeg-only discovery capability.
///
/// There is intentionally no requested-program argument and no generic
/// executable-locator sibling. Implementations either canonicalize an explicit
/// absolute configuration or resolve the one fixed ffmpeg basename.
pub trait FfmpegLocator: Send + Sync {
    /// Resolve one configured ffmpeg value to a canonical absolute identity.
    ///
    /// # Errors
    ///
    /// [`FfmpegLocatorError`] when the configuration or complete search policy
    /// is invalid, or no regular executable candidate is available.
    fn locate_ffmpeg(&self, configured: &Path) -> Result<FfmpegExecutable, FfmpegLocatorError>;
}

/// Host ffmpeg discovery over a snapshotted, explicitly supplied `PATH`.
#[derive(Clone, Debug, Default)]
pub struct StdFfmpegLocator {
    search_path: Option<OsString>,
}

impl StdFfmpegLocator {
    /// Construct the host locator from an explicit native path-list value.
    #[must_use]
    pub fn from_search_path(search_path: Option<OsString>) -> Self {
        Self { search_path }
    }

    /// Snapshot the host's `PATH` at the outer composition boundary.
    ///
    /// This is the only ambient read in executable discovery. The resulting
    /// capability owns the bytes, so later environment mutation cannot change
    /// its search policy.
    #[must_use]
    pub fn from_host_path() -> Self {
        Self::from_search_path(std::env::var_os("PATH"))
    }

    fn validated_search_directories(&self) -> Result<Vec<PathBuf>, FfmpegLocatorError> {
        let raw = self
            .search_path
            .as_deref()
            .ok_or(FfmpegLocatorError::SearchPathUnavailable)?;
        validate_raw_search_path(raw)?;

        let mut directories = Vec::new();
        for (index, entry) in std::env::split_paths(raw).enumerate() {
            if index >= MAX_FFMPEG_SEARCH_ENTRIES {
                return Err(FfmpegLocatorError::SearchPathLimit {
                    resource: "entry count",
                    got: index.saturating_add(1),
                    max: MAX_FFMPEG_SEARCH_ENTRIES,
                });
            }
            if entry.as_os_str().is_empty() {
                return Err(FfmpegLocatorError::EmptySearchEntry { index });
            }
            let Some(entry_text) = entry.to_str() else {
                return Err(FfmpegLocatorError::NonUtf8SearchEntry { index });
            };
            if entry_text.chars().any(char::is_control) {
                return Err(FfmpegLocatorError::ControlSearchEntry { index });
            }
            if !entry.is_absolute() {
                return Err(FfmpegLocatorError::RelativeSearchEntry { index, entry });
            }
            if entry
                .components()
                .any(|component| component == Component::ParentDir)
            {
                return Err(FfmpegLocatorError::ParentTraversalSearchEntry { index, entry });
            }
            directories.push(entry);
        }
        Ok(directories)
    }
}

impl FfmpegLocator for StdFfmpegLocator {
    fn locate_ffmpeg(&self, configured: &Path) -> Result<FfmpegExecutable, FfmpegLocatorError> {
        if configured.is_absolute() {
            return resolve_ffmpeg_candidate(configured.to_path_buf(), false)?.ok_or_else(|| {
                FfmpegLocatorError::NotExecutable {
                    path: configured.to_path_buf(),
                }
            });
        }
        if !is_default_ffmpeg_name(configured) {
            return Err(FfmpegLocatorError::UnsupportedConfiguredName {
                configured: configured.to_path_buf(),
            });
        }

        // Validate the complete list before touching a single candidate. A
        // successful early entry must not launder forbidden later policy.
        let directories = self.validated_search_directories()?;
        for directory in directories {
            let candidate = directory.join(ffmpeg_search_leaf());
            if let Some(executable) = resolve_ffmpeg_candidate(candidate, true)? {
                return Ok(executable);
            }
        }
        Err(FfmpegLocatorError::NotFound)
    }
}

// Only the unix/windows raw-search-path validator consumes these bounds;
// on wasm32 search is refused before either applies.
#[cfg(not(target_arch = "wasm32"))]
const MAX_FFMPEG_SEARCH_UNITS: usize = 64 * 1024;
const MAX_FFMPEG_SEARCH_ENTRIES: usize = 256;

#[cfg(any(unix, windows))]
fn validate_raw_search_path(raw: &OsStr) -> Result<(), FfmpegLocatorError> {
    let units = os_str_units(raw);
    if units > MAX_FFMPEG_SEARCH_UNITS {
        return Err(FfmpegLocatorError::SearchPathLimit {
            resource: "size",
            got: units,
            max: MAX_FFMPEG_SEARCH_UNITS,
        });
    }
    if os_str_contains_nul(raw) {
        return Err(FfmpegLocatorError::MalformedSearchPath {
            reason: "contains a NUL code point",
        });
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt as _;
        if raw
            .encode_wide()
            .filter(|unit| *unit == u16::from(b'"'))
            .count()
            % 2
            != 0
        {
            return Err(FfmpegLocatorError::MalformedSearchPath {
                reason: "contains an unbalanced double quote",
            });
        }
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn validate_raw_search_path(_raw: &OsStr) -> Result<(), FfmpegLocatorError> {
    Err(FfmpegLocatorError::SearchUnsupported {
        platform: std::env::consts::OS,
    })
}

#[cfg(unix)]
fn os_str_contains_nul(value: &OsStr) -> bool {
    use std::os::unix::ffi::OsStrExt as _;
    value.as_bytes().contains(&0)
}

#[cfg(unix)]
fn os_str_units(value: &OsStr) -> usize {
    use std::os::unix::ffi::OsStrExt as _;
    value.as_bytes().len()
}

#[cfg(windows)]
fn os_str_contains_nul(value: &OsStr) -> bool {
    use std::os::windows::ffi::OsStrExt as _;
    value.encode_wide().any(|unit| unit == 0)
}

#[cfg(windows)]
fn os_str_units(value: &OsStr) -> usize {
    use std::os::windows::ffi::OsStrExt as _;
    value.encode_wide().count()
}

#[cfg(windows)]
fn is_default_ffmpeg_name(configured: &Path) -> bool {
    let mut components = configured.components();
    let Some(Component::Normal(name)) = components.next() else {
        return false;
    };
    components.next().is_none()
        && name.to_str().is_some_and(|name| {
            name.eq_ignore_ascii_case("ffmpeg") || name.eq_ignore_ascii_case("ffmpeg.exe")
        })
}

#[cfg(not(windows))]
fn is_default_ffmpeg_name(configured: &Path) -> bool {
    configured == Path::new("ffmpeg")
}

#[cfg(windows)]
fn ffmpeg_search_leaf() -> &'static OsStr {
    OsStr::new("ffmpeg.exe")
}

#[cfg(not(windows))]
fn ffmpeg_search_leaf() -> &'static OsStr {
    OsStr::new("ffmpeg")
}

fn resolve_ffmpeg_candidate(
    candidate: PathBuf,
    searched: bool,
) -> Result<Option<FfmpegExecutable>, FfmpegLocatorError> {
    let canonical = match std::fs::canonicalize(&candidate) {
        Ok(canonical) => canonical,
        Err(err)
            if searched
                && matches!(
                    err.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
        {
            return Ok(None);
        }
        Err(err) => {
            return Err(FfmpegLocatorError::CandidateIo {
                candidate,
                operation: "canonicalize",
                err,
            });
        }
    };
    if !canonical.is_absolute() {
        return Err(FfmpegLocatorError::InvalidCanonicalIdentity {
            path: canonical,
            reason: "path is not absolute",
        });
    }
    if canonical.to_str().is_none() {
        return Err(FfmpegLocatorError::InvalidCanonicalIdentity {
            path: canonical,
            reason: "path is not valid UTF-8",
        });
    }
    if canonical
        .to_str()
        .is_some_and(|path| path.chars().any(char::is_control))
    {
        return Err(FfmpegLocatorError::InvalidCanonicalIdentity {
            path: canonical,
            reason: "path contains a control character",
        });
    }
    match open_validated_ffmpeg_image(&canonical) {
        Ok((_file, _attestation)) => {}
        Err(
            FfmpegLocatorError::NotExecutable { .. }
            | FfmpegLocatorError::UnsupportedExecutableFormat { .. }
            | FfmpegLocatorError::ExecutableSizeLimit { .. },
        ) if searched => return Ok(None),
        Err(error) => return Err(error),
    }
    Ok(Some(FfmpegExecutable {
        canonical_path: canonical,
    }))
}

fn open_validated_ffmpeg_image(
    path: &Path,
) -> Result<(std::fs::File, NativeImageAttestation), FfmpegLocatorError> {
    // Reject static special files before open: opening a FIFO can block
    // indefinitely. Safe `std` cannot make the later pathname open atomic
    // with this check, so the opened handle is rechecked before any bytes are
    // trusted and the residual same-identity race remains explicitly
    // qualified by the boundary contract.
    let preopen_metadata =
        std::fs::metadata(path).map_err(|err| FfmpegLocatorError::CandidateIo {
            candidate: path.to_path_buf(),
            operation: "inspect",
            err,
        })?;
    validate_ffmpeg_metadata(&preopen_metadata, path)?;

    let mut file = std::fs::File::open(path).map_err(|err| FfmpegLocatorError::CandidateIo {
        candidate: path.to_path_buf(),
        operation: "open",
        err,
    })?;
    let attestation = validate_open_ffmpeg_image(&mut file, path)?;
    Ok((file, attestation))
}

fn validate_ffmpeg_metadata(
    metadata: &std::fs::Metadata,
    path: &Path,
) -> Result<(), FfmpegLocatorError> {
    if !metadata.is_file() || !metadata_is_executable(metadata) {
        return Err(FfmpegLocatorError::NotExecutable {
            path: path.to_path_buf(),
        });
    }
    if metadata.len() > MAX_FFMPEG_EXECUTABLE_BYTES {
        return Err(FfmpegLocatorError::ExecutableSizeLimit {
            path: path.to_path_buf(),
            bytes: metadata.len(),
            max: MAX_FFMPEG_EXECUTABLE_BYTES,
        });
    }
    Ok(())
}

fn validate_open_ffmpeg_image(
    file: &mut std::fs::File,
    path: &Path,
) -> Result<NativeImageAttestation, FfmpegLocatorError> {
    let metadata = file
        .metadata()
        .map_err(|err| FfmpegLocatorError::CandidateIo {
            candidate: path.to_path_buf(),
            operation: "inspect opened executable",
            err,
        })?;
    validate_ffmpeg_metadata(&metadata, path)?;
    attest_host_native_image(file, metadata.len(), path)?.ok_or_else(|| {
        FfmpegLocatorError::UnsupportedExecutableFormat {
            path: path.to_path_buf(),
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn read_exact_at_or_short(
    file: &mut std::fs::File,
    offset: u64,
    buffer: &mut [u8],
    path: &Path,
) -> Result<bool, FfmpegLocatorError> {
    file.seek(io::SeekFrom::Start(offset))
        .map_err(|err| FfmpegLocatorError::CandidateIo {
            candidate: path.to_path_buf(),
            operation: "seek executable header",
            err,
        })?;
    match file.read_exact(buffer) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(err) => Err(FfmpegLocatorError::CandidateIo {
            candidate: path.to_path_buf(),
            operation: "read executable header",
            err,
        }),
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn validate_elf_gnu_property(
    file: &mut std::fs::File,
    file_offset: u64,
    file_bytes: u64,
    architecture: NativeExecutableArchitecture,
    path: &Path,
) -> Result<bool, FfmpegLocatorError> {
    const NOTE_LIMIT: u64 = 1_024;
    const NOTE_HEADER_BYTES: usize = 12;
    const NOTE_NAME_BYTES: usize = 4;
    const PROPERTY_HEADER_BYTES: usize = 8;
    const NOTE_TYPE_GNU_PROPERTY: u32 = 5;
    const PROPERTY_ALIGN: usize = 8;
    const GNU_PROPERTY_LOPROC: u32 = 0xc000_0000;
    const GNU_PROPERTY_HIPROC: u32 = 0xdfff_ffff;
    const GNU_PROPERTY_AARCH64_FEATURE_1_AND: u32 = 0xc000_0000;
    const GNU_PROPERTY_X86_KNOWN_START: u32 = 0xc000_0000;
    const GNU_PROPERTY_X86_KNOWN_END: u32 = 0xc001_ffff;

    if !(NOTE_HEADER_BYTES as u64 + NOTE_NAME_BYTES as u64..=NOTE_LIMIT).contains(&file_bytes) {
        return Ok(false);
    }
    let mut note = vec![0_u8; file_bytes as usize];
    if !read_exact_at_or_short(file, file_offset, &mut note, path)? {
        return Ok(false);
    }
    let name_bytes = u32::from_le_bytes([note[0], note[1], note[2], note[3]]);
    let descriptor_bytes =
        usize::try_from(u32::from_le_bytes([note[4], note[5], note[6], note[7]]))
            .unwrap_or(usize::MAX);
    let note_type = u32::from_le_bytes([note[8], note[9], note[10], note[11]]);
    if name_bytes != NOTE_NAME_BYTES as u32
        || note_type != NOTE_TYPE_GNU_PROPERTY
        || note[NOTE_HEADER_BYTES..NOTE_HEADER_BYTES + NOTE_NAME_BYTES] != *b"GNU\0"
    {
        return Ok(false);
    }
    let descriptor_start = (NOTE_HEADER_BYTES + NOTE_NAME_BYTES)
        .checked_add(PROPERTY_ALIGN - 1)
        .map(|value| value & !(PROPERTY_ALIGN - 1))
        .unwrap_or(usize::MAX);
    let Some(descriptor_end) = descriptor_start.checked_add(descriptor_bytes) else {
        return Ok(false);
    };
    if descriptor_end > note.len() {
        return Ok(false);
    }

    let mut cursor = descriptor_start;
    let mut previous_type = None;
    while cursor < descriptor_end {
        if descriptor_end - cursor < PROPERTY_HEADER_BYTES {
            return Ok(false);
        }
        let property_type = u32::from_le_bytes([
            note[cursor],
            note[cursor + 1],
            note[cursor + 2],
            note[cursor + 3],
        ]);
        let property_bytes = usize::try_from(u32::from_le_bytes([
            note[cursor + 4],
            note[cursor + 5],
            note[cursor + 6],
            note[cursor + 7],
        ]))
        .unwrap_or(usize::MAX);
        if previous_type.is_some_and(|previous| property_type <= previous) {
            return Ok(false);
        }
        let data_start = cursor + PROPERTY_HEADER_BYTES;
        let Some(data_end) = data_start.checked_add(property_bytes) else {
            return Ok(false);
        };
        let Some(next) = data_end
            .checked_add(PROPERTY_ALIGN - 1)
            .map(|value| value & !(PROPERTY_ALIGN - 1))
        else {
            return Ok(false);
        };
        if next > descriptor_end {
            return Ok(false);
        }
        if (GNU_PROPERTY_LOPROC..=GNU_PROPERTY_HIPROC).contains(&property_type) {
            let recognized = match architecture {
                NativeExecutableArchitecture::X86_64 => {
                    (GNU_PROPERTY_X86_KNOWN_START..=GNU_PROPERTY_X86_KNOWN_END)
                        .contains(&property_type)
                        && property_bytes == 4
                }
                NativeExecutableArchitecture::Aarch64 => {
                    property_type == GNU_PROPERTY_AARCH64_FEATURE_1_AND && property_bytes == 4
                }
            };
            if !recognized {
                return Ok(false);
            }
        }
        previous_type = Some(property_type);
        cursor = next;
    }
    Ok(cursor == descriptor_end)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn attest_host_native_image(
    file: &mut std::fs::File,
    file_len: u64,
    path: &Path,
) -> Result<Option<NativeImageAttestation>, FfmpegLocatorError> {
    const ELF64_HEADER_BYTES: usize = 64;
    const ELF64_PROGRAM_HEADER_BYTES: u16 = 56;
    // Linux's native loader rejects program-header tables above 65,536
    // bytes; mirror that limit so a parser-accepted image cannot reach an
    // ENOEXEC-only loader refusal on this dimension.
    const MAX_PROGRAM_HEADERS: u16 = 1_170;
    const PT_LOAD: u32 = 1;
    const PT_INTERP: u32 = 3;
    const PT_GNU_PROPERTY: u32 = 0x6474_e553;
    const PF_X: u32 = 0x1;
    const MAX_INTERPRETER_BYTES: u64 = 4_096;

    let Some(architecture) = host_native_architecture() else {
        return Ok(None);
    };
    let mut header = [0_u8; ELF64_HEADER_BYTES];
    if !read_exact_at_or_short(file, 0, &mut header, path)?
        || header[..4] != *b"\x7fELF"
        || header[4] != 2
        || header[5] != 1
        || header[6] != 1
    {
        return Ok(None);
    }
    let executable_type = u16::from_le_bytes([header[16], header[17]]);
    let machine = u16::from_le_bytes([header[18], header[19]]);
    let expected_machine = match architecture {
        NativeExecutableArchitecture::X86_64 => 62,
        NativeExecutableArchitecture::Aarch64 => 183,
    };
    if !matches!(executable_type, 2 | 3)
        || machine != expected_machine
        || u32::from_le_bytes([header[20], header[21], header[22], header[23]]) != 1
        || u16::from_le_bytes([header[52], header[53]]) != ELF64_HEADER_BYTES as u16
    {
        return Ok(None);
    }
    let program_offset = u64::from_le_bytes([
        header[32], header[33], header[34], header[35], header[36], header[37], header[38],
        header[39],
    ]);
    let entry = u64::from_le_bytes([
        header[24], header[25], header[26], header[27], header[28], header[29], header[30],
        header[31],
    ]);
    let program_entry_bytes = u16::from_le_bytes([header[54], header[55]]);
    let program_count = u16::from_le_bytes([header[56], header[57]]);
    if entry == 0
        || program_offset < ELF64_HEADER_BYTES as u64
        || program_entry_bytes != ELF64_PROGRAM_HEADER_BYTES
        || program_count == 0
        || program_count > MAX_PROGRAM_HEADERS
    {
        return Ok(None);
    }
    let Some(program_bytes) = u64::from(program_entry_bytes).checked_mul(u64::from(program_count))
    else {
        return Ok(None);
    };
    if program_offset
        .checked_add(program_bytes)
        .is_none_or(|end| end > file_len)
    {
        return Ok(None);
    }

    let mut saw_load = false;
    let mut saw_interpreter = false;
    let mut saw_gnu_property = false;
    let mut previous_load_address = None;
    let mut entry_is_file_backed_executable = false;
    for index in 0..program_count {
        let offset = program_offset + u64::from(index) * u64::from(program_entry_bytes);
        let mut program = [0_u8; ELF64_PROGRAM_HEADER_BYTES as usize];
        if !read_exact_at_or_short(file, offset, &mut program, path)? {
            return Ok(None);
        }
        let kind = u32::from_le_bytes([program[0], program[1], program[2], program[3]]);
        let flags = u32::from_le_bytes([program[4], program[5], program[6], program[7]]);
        let file_offset = u64::from_le_bytes([
            program[8],
            program[9],
            program[10],
            program[11],
            program[12],
            program[13],
            program[14],
            program[15],
        ]);
        let virtual_address = u64::from_le_bytes([
            program[16],
            program[17],
            program[18],
            program[19],
            program[20],
            program[21],
            program[22],
            program[23],
        ]);
        let file_bytes = u64::from_le_bytes([
            program[32],
            program[33],
            program[34],
            program[35],
            program[36],
            program[37],
            program[38],
            program[39],
        ]);
        let memory_bytes = u64::from_le_bytes([
            program[40],
            program[41],
            program[42],
            program[43],
            program[44],
            program[45],
            program[46],
            program[47],
        ]);
        let alignment = u64::from_le_bytes([
            program[48],
            program[49],
            program[50],
            program[51],
            program[52],
            program[53],
            program[54],
            program[55],
        ]);
        if kind == PT_LOAD {
            let Some(file_end) = file_offset.checked_add(file_bytes) else {
                return Ok(None);
            };
            let Some(memory_end) = virtual_address.checked_add(memory_bytes) else {
                return Ok(None);
            };
            let Some(file_backed_end) = virtual_address.checked_add(file_bytes) else {
                return Ok(None);
            };
            if file_end > file_len
                || memory_bytes < file_bytes
                || (alignment > 1
                    && (!alignment.is_power_of_two()
                        || virtual_address % alignment != file_offset % alignment))
                || previous_load_address.is_some_and(|previous| virtual_address < previous)
            {
                return Ok(None);
            }
            previous_load_address = Some(virtual_address);
            saw_load = true;
            if flags & PF_X != 0
                && entry >= virtual_address
                && entry < file_backed_end
                && entry < memory_end
            {
                entry_is_file_backed_executable = true;
            }
        } else if kind == PT_INTERP {
            if saw_load
                || saw_interpreter
                || !(2..=MAX_INTERPRETER_BYTES).contains(&file_bytes)
                || file_offset
                    .checked_add(file_bytes)
                    .is_none_or(|end| end > file_len)
            {
                return Ok(None);
            }
            let mut interpreter = vec![0_u8; file_bytes as usize];
            if !read_exact_at_or_short(file, file_offset, &mut interpreter, path)?
                || interpreter[0] != b'/'
                || interpreter.last() != Some(&0)
                || interpreter[1..interpreter.len() - 1].contains(&0)
            {
                return Ok(None);
            }
            saw_interpreter = true;
        } else if kind == PT_GNU_PROPERTY {
            if saw_gnu_property
                || file_offset
                    .checked_add(file_bytes)
                    .is_none_or(|end| end > file_len)
                || !validate_elf_gnu_property(file, file_offset, file_bytes, architecture, path)?
            {
                return Ok(None);
            }
            saw_gnu_property = true;
        } else if kind != 0
            && file_offset
                .checked_add(file_bytes)
                .is_none_or(|end| end > file_len)
        {
            return Ok(None);
        }
    }
    if !saw_load || !entry_is_file_backed_executable {
        return Ok(None);
    }
    Ok(Some(NativeImageAttestation {
        format: NativeExecutableFormat::Elf64,
        architecture,
        file_bytes: file_len,
        policy_version: NATIVE_IMAGE_POLICY_VERSION,
    }))
}

#[cfg(target_os = "macos")]
fn attest_host_native_image(
    file: &mut std::fs::File,
    file_len: u64,
    path: &Path,
) -> Result<Option<NativeImageAttestation>, FfmpegLocatorError> {
    const FAT_ARCH_LIMIT: u32 = 64;

    let Some(architecture) = host_native_architecture() else {
        return Ok(None);
    };
    let mut header = [0_u8; 8];
    if !read_exact_at_or_short(file, 0, &mut header, path)? {
        return Ok(None);
    }
    if matches!(header[..4], [0xcf, 0xfa, 0xed, 0xfe]) {
        return attest_mach_o_thin(
            file,
            0,
            file_len,
            file_len,
            path,
            NativeExecutableFormat::MachO64,
        );
    }

    let (endian, entry_bytes, offset_is_64) = match header[..4] {
        [0xca, 0xfe, 0xba, 0xbe] => (MachEndian::Big, 20_u64, false),
        [0xca, 0xfe, 0xba, 0xbf] => (MachEndian::Big, 32_u64, true),
        _ => return Ok(None),
    };
    let count = endian.u32([header[4], header[5], header[6], header[7]]);
    if count == 0 || count > FAT_ARCH_LIMIT {
        return Ok(None);
    }
    let Some(table_end) = 8_u64.checked_add(entry_bytes * u64::from(count)) else {
        return Ok(None);
    };
    if table_end > file_len {
        return Ok(None);
    }
    let expected_cpu = mach_cpu_type(architecture);
    let expected_subtype = mach_cpu_subtype(architecture);
    let mut host_slice = None;
    let mut slice_ranges = Vec::with_capacity(count as usize);
    for index in 0..count {
        let mut entry = [0_u8; 32];
        let entry_len = entry_bytes as usize;
        if !read_exact_at_or_short(
            file,
            8 + u64::from(index) * entry_bytes,
            &mut entry[..entry_len],
            path,
        )? {
            return Ok(None);
        }
        let cpu = endian.u32([entry[0], entry[1], entry[2], entry[3]]);
        let subtype = endian.u32([entry[4], entry[5], entry[6], entry[7]]) & 0x00ff_ffff;
        let (offset, bytes) = if offset_is_64 {
            (
                endian.u64([
                    entry[8], entry[9], entry[10], entry[11], entry[12], entry[13], entry[14],
                    entry[15],
                ]),
                endian.u64([
                    entry[16], entry[17], entry[18], entry[19], entry[20], entry[21], entry[22],
                    entry[23],
                ]),
            )
        } else {
            (
                u64::from(endian.u32([entry[8], entry[9], entry[10], entry[11]])),
                u64::from(endian.u32([entry[12], entry[13], entry[14], entry[15]])),
            )
        };
        let alignment_exponent = if offset_is_64 {
            endian.u32([entry[24], entry[25], entry[26], entry[27]])
        } else {
            endian.u32([entry[16], entry[17], entry[18], entry[19]])
        };
        let Some(alignment) = 1_u64.checked_shl(alignment_exponent) else {
            return Ok(None);
        };
        if bytes < 32
            || offset < table_end
            || offset % alignment != 0
            || offset.checked_add(bytes).is_none_or(|end| end > file_len)
        {
            return Ok(None);
        }
        let end = offset + bytes;
        if slice_ranges
            .iter()
            .any(|&(prior_start, prior_end)| offset < prior_end && prior_start < end)
        {
            return Ok(None);
        }
        slice_ranges.push((offset, end));
        if cpu == expected_cpu && subtype == expected_subtype {
            if host_slice.is_some() {
                return Ok(None);
            }
            host_slice = Some((offset, bytes));
        }
    }
    let Some((offset, bytes)) = host_slice else {
        return Ok(None);
    };
    attest_mach_o_thin(
        file,
        offset,
        bytes,
        file_len,
        path,
        NativeExecutableFormat::MachOUniversal,
    )
}

#[cfg(windows)]
fn attest_host_native_image(
    file: &mut std::fs::File,
    file_len: u64,
    path: &Path,
) -> Result<Option<NativeImageAttestation>, FfmpegLocatorError> {
    const DOS_HEADER_BYTES: usize = 64;
    const MAX_PE_HEADER_OFFSET: u64 = 1024 * 1024;
    const COFF_HEADER_BYTES: usize = 24;
    const SECTION_HEADER_BYTES: u64 = 40;
    const MAX_SECTIONS: u16 = 96;
    const PE32_PLUS_MINIMUM_BYTES: u16 = 112;
    const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
    const IMAGE_FILE_DLL: u16 = 0x2000;
    const IMAGE_SCN_CNT_CODE: u32 = 0x0000_0020;
    const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
    const MIN_FILE_ALIGNMENT: u64 = 512;
    const MAX_FILE_ALIGNMENT: u64 = 65_536;
    const PAGE_BYTES: u64 = 4_096;

    let Some(architecture) = host_native_architecture() else {
        return Ok(None);
    };

    let mut dos_header = [0_u8; DOS_HEADER_BYTES];
    if !read_exact_at_or_short(file, 0, &mut dos_header, path)? || dos_header[..2] != *b"MZ" {
        return Ok(None);
    }
    let pe_offset = u64::from(u32::from_le_bytes([
        dos_header[0x3c],
        dos_header[0x3d],
        dos_header[0x3e],
        dos_header[0x3f],
    ]));
    let Some(coff_end) = pe_offset.checked_add(COFF_HEADER_BYTES as u64) else {
        return Ok(None);
    };
    if pe_offset < DOS_HEADER_BYTES as u64
        || pe_offset > MAX_PE_HEADER_OFFSET
        || coff_end > file_len
    {
        return Ok(None);
    }
    let mut coff = [0_u8; COFF_HEADER_BYTES];
    if !read_exact_at_or_short(file, pe_offset, &mut coff, path)? || coff[..4] != *b"PE\0\0" {
        return Ok(None);
    }
    let machine = u16::from_le_bytes([coff[4], coff[5]]);
    let expected_machine = match architecture {
        NativeExecutableArchitecture::X86_64 => 0x8664,
        NativeExecutableArchitecture::Aarch64 => 0xaa64,
    };
    let section_count = u16::from_le_bytes([coff[6], coff[7]]);
    let optional_bytes = u16::from_le_bytes([coff[20], coff[21]]);
    let characteristics = u16::from_le_bytes([coff[22], coff[23]]);
    if machine != expected_machine
        || section_count == 0
        || section_count > MAX_SECTIONS
        || optional_bytes < PE32_PLUS_MINIMUM_BYTES
        || characteristics & IMAGE_FILE_EXECUTABLE_IMAGE == 0
        || characteristics & IMAGE_FILE_DLL != 0
    {
        return Ok(None);
    }
    let optional_offset = coff_end;
    let Some(section_offset) = optional_offset.checked_add(u64::from(optional_bytes)) else {
        return Ok(None);
    };
    let Some(section_end) =
        section_offset.checked_add(SECTION_HEADER_BYTES * u64::from(section_count))
    else {
        return Ok(None);
    };
    if section_end > file_len {
        return Ok(None);
    }
    let mut optional = [0_u8; PE32_PLUS_MINIMUM_BYTES as usize];
    if !read_exact_at_or_short(file, optional_offset, &mut optional, path)?
        || u16::from_le_bytes([optional[0], optional[1]]) != 0x020b
    {
        return Ok(None);
    }
    let entry_rva = u64::from(u32::from_le_bytes([
        optional[16],
        optional[17],
        optional[18],
        optional[19],
    ]));
    let image_base = u64::from_le_bytes([
        optional[24],
        optional[25],
        optional[26],
        optional[27],
        optional[28],
        optional[29],
        optional[30],
        optional[31],
    ]);
    let section_alignment = u64::from(u32::from_le_bytes([
        optional[32],
        optional[33],
        optional[34],
        optional[35],
    ]));
    let file_alignment = u64::from(u32::from_le_bytes([
        optional[36],
        optional[37],
        optional[38],
        optional[39],
    ]));
    let size_of_image = u64::from(u32::from_le_bytes([
        optional[56],
        optional[57],
        optional[58],
        optional[59],
    ]));
    let size_of_headers = u64::from(u32::from_le_bytes([
        optional[60],
        optional[61],
        optional[62],
        optional[63],
    ]));
    let subsystem = u16::from_le_bytes([optional[68], optional[69]]);
    let loader_flags =
        u32::from_le_bytes([optional[104], optional[105], optional[106], optional[107]]);
    let directory_count = u64::from(u32::from_le_bytes([
        optional[108],
        optional[109],
        optional[110],
        optional[111],
    ]));
    if entry_rva == 0
        || image_base == 0
        || image_base % 65_536 != 0
        || !file_alignment.is_power_of_two()
        || !(MIN_FILE_ALIGNMENT..=MAX_FILE_ALIGNMENT).contains(&file_alignment)
        || !section_alignment.is_power_of_two()
        || section_alignment < file_alignment
        || (section_alignment < PAGE_BYTES && section_alignment != file_alignment)
        || size_of_image == 0
        || size_of_image % section_alignment != 0
        || size_of_headers < section_end
        || size_of_headers > file_len
        || size_of_headers % file_alignment != 0
        || size_of_image < size_of_headers
        || subsystem == 0
        || loader_flags != 0
        || directory_count
            .checked_mul(8)
            .and_then(|bytes| 112_u64.checked_add(bytes))
            .is_none_or(|minimum| minimum > u64::from(optional_bytes))
    {
        return Ok(None);
    }
    let mut previous_virtual_end = size_of_headers;
    let mut previous_raw_end = size_of_headers;
    let mut entry_is_file_backed_executable = false;
    for index in 0..section_count {
        let mut section = [0_u8; SECTION_HEADER_BYTES as usize];
        if !read_exact_at_or_short(
            file,
            section_offset + u64::from(index) * SECTION_HEADER_BYTES,
            &mut section,
            path,
        )? {
            return Ok(None);
        }
        let virtual_bytes = u64::from(u32::from_le_bytes([
            section[8],
            section[9],
            section[10],
            section[11],
        ]));
        let virtual_address = u64::from(u32::from_le_bytes([
            section[12],
            section[13],
            section[14],
            section[15],
        ]));
        let raw_bytes = u64::from(u32::from_le_bytes([
            section[16],
            section[17],
            section[18],
            section[19],
        ]));
        let raw_offset = u64::from(u32::from_le_bytes([
            section[20],
            section[21],
            section[22],
            section[23],
        ]));
        let characteristics =
            u32::from_le_bytes([section[36], section[37], section[38], section[39]]);
        let mapped_bytes = virtual_bytes.max(raw_bytes);
        let Some(virtual_end) = virtual_address.checked_add(mapped_bytes) else {
            return Ok(None);
        };
        let Some(aligned_virtual_end) = checked_align_up(virtual_end, section_alignment) else {
            return Ok(None);
        };
        let raw_end = if raw_bytes == 0 {
            if raw_offset != 0 {
                return Ok(None);
            }
            previous_raw_end
        } else {
            let Some(raw_end) = raw_offset.checked_add(raw_bytes) else {
                return Ok(None);
            };
            if raw_bytes % file_alignment != 0
                || raw_offset % file_alignment != 0
                || raw_offset < previous_raw_end
                || raw_end > file_len
            {
                return Ok(None);
            }
            raw_end
        };
        if mapped_bytes == 0
            || virtual_address % section_alignment != 0
            || virtual_address < previous_virtual_end
            || aligned_virtual_end > size_of_image
        {
            return Ok(None);
        }
        let file_backed_code_bytes = if virtual_bytes == 0 {
            raw_bytes
        } else {
            virtual_bytes.min(raw_bytes)
        };
        if characteristics & IMAGE_SCN_CNT_CODE != 0
            && characteristics & IMAGE_SCN_MEM_EXECUTE != 0
            && entry_rva >= virtual_address
            && entry_rva
                < virtual_address
                    .checked_add(file_backed_code_bytes)
                    .unwrap_or(virtual_address)
        {
            entry_is_file_backed_executable = true;
        }
        previous_virtual_end = aligned_virtual_end;
        previous_raw_end = raw_end;
    }
    if !entry_is_file_backed_executable {
        return Ok(None);
    }
    Ok(Some(NativeImageAttestation {
        format: NativeExecutableFormat::Pe32Plus,
        architecture,
        file_bytes: file_len,
        policy_version: NATIVE_IMAGE_POLICY_VERSION,
    }))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    windows
)))]
fn attest_host_native_image(
    _file: &mut std::fs::File,
    _file_len: u64,
    _path: &Path,
) -> Result<Option<NativeImageAttestation>, FfmpegLocatorError> {
    Err(FfmpegLocatorError::NativeImageUnsupported {
        platform: std::env::consts::OS,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn host_native_architecture() -> Option<NativeExecutableArchitecture> {
    #[cfg(target_arch = "x86_64")]
    {
        return Some(NativeExecutableArchitecture::X86_64);
    }
    #[cfg(target_arch = "aarch64")]
    {
        return Some(NativeExecutableArchitecture::Aarch64);
    }
    #[allow(unreachable_code)]
    None
}

#[cfg(windows)]
fn checked_align_up(value: u64, alignment: u64) -> Option<u64> {
    debug_assert!(alignment.is_power_of_two());
    value
        .checked_add(alignment - 1)
        .map(|sum| sum & !(alignment - 1))
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
enum MachEndian {
    Little,
    Big,
}

#[cfg(target_os = "macos")]
impl MachEndian {
    fn u32(self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }

    fn u64(self, bytes: [u8; 8]) -> u64 {
        match self {
            Self::Little => u64::from_le_bytes(bytes),
            Self::Big => u64::from_be_bytes(bytes),
        }
    }
}

#[cfg(target_os = "macos")]
fn mach_cpu_type(architecture: NativeExecutableArchitecture) -> u32 {
    match architecture {
        NativeExecutableArchitecture::X86_64 => 0x0100_0007,
        NativeExecutableArchitecture::Aarch64 => 0x0100_000c,
    }
}

#[cfg(target_os = "macos")]
fn mach_cpu_subtype(architecture: NativeExecutableArchitecture) -> u32 {
    match architecture {
        NativeExecutableArchitecture::X86_64 => 3,
        NativeExecutableArchitecture::Aarch64 => 0,
    }
}

#[cfg(target_os = "macos")]
fn attest_mach_o_thin(
    file: &mut std::fs::File,
    slice_offset: u64,
    slice_bytes: u64,
    file_bytes: u64,
    path: &Path,
    format: NativeExecutableFormat,
) -> Result<Option<NativeImageAttestation>, FfmpegLocatorError> {
    const MACH_HEADER_64_BYTES: u64 = 32;
    const LOAD_COMMAND_HEADER_BYTES: u64 = 8;
    const SEGMENT_COMMAND_64_BYTES: u64 = 72;
    const SECTION_64_BYTES: u64 = 80;
    const MAX_LOAD_COMMANDS: u32 = 4_096;
    const MAX_SECTIONS_PER_SEGMENT: u64 = 4_096;
    const LC_SEGMENT_64: u32 = 0x19;
    const LC_MAIN: u32 = 0x8000_0028;
    const ENTRY_POINT_COMMAND_BYTES: u64 = 24;
    const VM_PROT_EXECUTE: u32 = 0x4;

    if slice_bytes < MACH_HEADER_64_BYTES {
        return Ok(None);
    }
    let mut header = [0_u8; MACH_HEADER_64_BYTES as usize];
    if !read_exact_at_or_short(file, slice_offset, &mut header, path)? {
        return Ok(None);
    }
    let endian = match header[..4] {
        [0xcf, 0xfa, 0xed, 0xfe] => MachEndian::Little,
        [0xfe, 0xed, 0xfa, 0xcf] => MachEndian::Big,
        _ => return Ok(None),
    };
    let Some(architecture) = host_native_architecture() else {
        return Ok(None);
    };
    let cpu = endian.u32([header[4], header[5], header[6], header[7]]);
    let cpu_subtype = endian.u32([header[8], header[9], header[10], header[11]]) & 0x00ff_ffff;
    let file_type = endian.u32([header[12], header[13], header[14], header[15]]);
    let command_count = endian.u32([header[16], header[17], header[18], header[19]]);
    let command_bytes = u64::from(endian.u32([header[20], header[21], header[22], header[23]]));
    if cpu != mach_cpu_type(architecture)
        || cpu_subtype != mach_cpu_subtype(architecture)
        || file_type != 2
        || header[28..32] != [0, 0, 0, 0]
        || command_count == 0
        || command_count > MAX_LOAD_COMMANDS
        || command_bytes < u64::from(command_count) * LOAD_COMMAND_HEADER_BYTES
        || MACH_HEADER_64_BYTES
            .checked_add(command_bytes)
            .is_none_or(|end| end > slice_bytes)
    {
        return Ok(None);
    }

    let Some(commands_end_in_slice) = MACH_HEADER_64_BYTES.checked_add(command_bytes) else {
        return Ok(None);
    };
    let Some(commands_end) = slice_offset.checked_add(commands_end_in_slice) else {
        return Ok(None);
    };
    let Some(mut cursor) = slice_offset.checked_add(MACH_HEADER_64_BYTES) else {
        return Ok(None);
    };
    let mut saw_segment = false;
    let mut saw_hard_pagezero = false;
    let mut entry_offset = None;
    let mut executable_file_ranges = Vec::new();
    for _ in 0..command_count {
        let mut command_header = [0_u8; LOAD_COMMAND_HEADER_BYTES as usize];
        if !read_exact_at_or_short(file, cursor, &mut command_header, path)? {
            return Ok(None);
        }
        let command = endian.u32([
            command_header[0],
            command_header[1],
            command_header[2],
            command_header[3],
        ]);
        let command_size = u64::from(endian.u32([
            command_header[4],
            command_header[5],
            command_header[6],
            command_header[7],
        ]));
        if command_size < LOAD_COMMAND_HEADER_BYTES
            || command_size % 8 != 0
            || cursor
                .checked_add(command_size)
                .is_none_or(|end| end > commands_end)
        {
            return Ok(None);
        }
        if command == LC_SEGMENT_64 {
            if command_size < SEGMENT_COMMAND_64_BYTES {
                return Ok(None);
            }
            let mut segment = [0_u8; SEGMENT_COMMAND_64_BYTES as usize];
            if !read_exact_at_or_short(file, cursor, &mut segment, path)? {
                return Ok(None);
            }
            let segment_file_offset = endian.u64([
                segment[40],
                segment[41],
                segment[42],
                segment[43],
                segment[44],
                segment[45],
                segment[46],
                segment[47],
            ]);
            let segment_file_bytes = endian.u64([
                segment[48],
                segment[49],
                segment[50],
                segment[51],
                segment[52],
                segment[53],
                segment[54],
                segment[55],
            ]);
            let segment_virtual_address = endian.u64([
                segment[24],
                segment[25],
                segment[26],
                segment[27],
                segment[28],
                segment[29],
                segment[30],
                segment[31],
            ]);
            let segment_memory_bytes = endian.u64([
                segment[32],
                segment[33],
                segment[34],
                segment[35],
                segment[36],
                segment[37],
                segment[38],
                segment[39],
            ]);
            let max_protection = endian.u32([segment[56], segment[57], segment[58], segment[59]]);
            let initial_protection =
                endian.u32([segment[60], segment[61], segment[62], segment[63]]);
            let section_count =
                u64::from(endian.u32([segment[64], segment[65], segment[66], segment[67]]));
            let Some(section_bytes) = section_count.checked_mul(SECTION_64_BYTES) else {
                return Ok(None);
            };
            let Some(segment_file_end) = segment_file_offset.checked_add(segment_file_bytes) else {
                return Ok(None);
            };
            if section_count > MAX_SECTIONS_PER_SEGMENT
                || SEGMENT_COMMAND_64_BYTES
                    .checked_add(section_bytes)
                    .is_none_or(|minimum| minimum > command_size)
                || segment_file_end > slice_bytes
                || segment_memory_bytes < segment_file_bytes
                || segment_virtual_address
                    .checked_add(segment_memory_bytes)
                    .is_none()
                || initial_protection & !max_protection != 0
            {
                return Ok(None);
            }
            let required_pagezero_bytes = match architecture {
                NativeExecutableArchitecture::X86_64 => 0x1_000,
                NativeExecutableArchitecture::Aarch64 => 0x1_0000_0000,
            };
            let pagezero_name = {
                let mut expected = [0_u8; 16];
                expected[..10].copy_from_slice(b"__PAGEZERO");
                segment[8..24] == expected
            };
            if pagezero_name {
                if saw_hard_pagezero
                    || segment_virtual_address != 0
                    || segment_memory_bytes < required_pagezero_bytes
                    || segment_file_offset != 0
                    || segment_file_bytes != 0
                    || max_protection != 0
                    || initial_protection != 0
                    || section_count != 0
                {
                    return Ok(None);
                }
                saw_hard_pagezero = true;
            }
            if initial_protection & VM_PROT_EXECUTE != 0 && segment_file_bytes != 0 {
                executable_file_ranges.push((segment_file_offset, segment_file_end));
            }
            saw_segment = true;
        } else if command == LC_MAIN {
            if command_size != ENTRY_POINT_COMMAND_BYTES || entry_offset.is_some() {
                return Ok(None);
            }
            let mut entry_command = [0_u8; ENTRY_POINT_COMMAND_BYTES as usize];
            if !read_exact_at_or_short(file, cursor, &mut entry_command, path)? {
                return Ok(None);
            }
            let entry = endian.u64([
                entry_command[8],
                entry_command[9],
                entry_command[10],
                entry_command[11],
                entry_command[12],
                entry_command[13],
                entry_command[14],
                entry_command[15],
            ]);
            if entry < commands_end_in_slice || entry >= slice_bytes {
                return Ok(None);
            }
            entry_offset = Some(entry);
        }
        cursor += command_size;
    }
    let Some(entry_offset) = entry_offset else {
        return Ok(None);
    };
    if !saw_segment
        || !saw_hard_pagezero
        || cursor != commands_end
        || !executable_file_ranges
            .iter()
            .any(|&(start, end)| entry_offset >= start && entry_offset < end)
    {
        return Ok(None);
    }
    Ok(Some(NativeImageAttestation {
        format,
        architecture,
        file_bytes,
        policy_version: NATIVE_IMAGE_POLICY_VERSION,
    }))
}

#[cfg(unix)]
fn metadata_is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn metadata_is_executable(_metadata: &std::fs::Metadata) -> bool {
    true
}

/// Stable identity of the process primitive selected by a [`ProcessRunner`].
///
/// Successful production invocations record both [`Self::identity`] and
/// [`Self::policy_version`] in C9 provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessMechanism {
    /// Absolute-path POSIX spawn into a new process group.
    PosixSpawnAbsoluteProcessGroup,
    /// Explicit Win32 application spawn with atomic Job Object assignment.
    WindowsCreateProcessJobList,
    /// Deterministic in-memory test double; no host process is issued.
    Scripted,
    /// The host has no audited exact-image implementation and will fail closed.
    ExactImageUnavailable,
}

impl ProcessMechanism {
    /// Stable machine-readable mechanism identity.
    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::PosixSpawnAbsoluteProcessGroup => "posix_spawn.absolute_path.new_process_group",
            Self::WindowsCreateProcessJobList => {
                "create_process_w.explicit_application.atomic_job_list"
            }
            Self::Scripted => "scripted.process_runner",
            Self::ExactImageUnavailable => "exact_image.unavailable",
        }
    }

    /// Version of the mechanism's executable-selection and containment policy.
    #[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
    #[must_use]
    pub const fn policy_version(self) -> u32 {
        match self {
            Self::PosixSpawnAbsoluteProcessGroup
            | Self::WindowsCreateProcessJobList
            | Self::ExactImageUnavailable => asupersync::process::EXACT_IMAGE_SPAWN_POLICY_VERSION,
            Self::Scripted => 1,
        }
    }

    /// Version of the mechanism's executable-selection and containment policy.
    ///
    /// Without the `exact-process` feature (including wasm32), the exact-image
    /// substrate is structurally absent from the build. Only the in-memory
    /// identity carries a policy version; host spawn identities are
    /// unreachable and report 0 rather than naming a substrate that is not
    /// linked.
    #[cfg(any(target_arch = "wasm32", not(feature = "exact-process")))]
    #[must_use]
    pub const fn policy_version(self) -> u32 {
        match self {
            Self::Scripted => 1,
            Self::PosixSpawnAbsoluteProcessGroup
            | Self::WindowsCreateProcessJobList
            | Self::ExactImageUnavailable => 0,
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
fn exact_image_mechanism(
    mechanism: asupersync::process::ExactImageSpawnMechanism,
) -> ProcessMechanism {
    match mechanism {
        asupersync::process::ExactImageSpawnMechanism::PosixSpawnAbsoluteProcessGroup => {
            ProcessMechanism::PosixSpawnAbsoluteProcessGroup
        }
        asupersync::process::ExactImageSpawnMechanism::WindowsCreateProcessJobList => {
            ProcessMechanism::WindowsCreateProcessJobList
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
const fn host_exact_image_mechanism() -> ProcessMechanism {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        ProcessMechanism::PosixSpawnAbsoluteProcessGroup
    }
    #[cfg(windows)]
    {
        ProcessMechanism::WindowsCreateProcessJobList
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        ProcessMechanism::ExactImageUnavailable
    }
}

/// A complete, self-contained description of one subprocess invocation.
#[derive(Clone, Debug)]
pub struct ProcessSpec {
    /// The program to execute (a path, never a shell string).
    pub program: PathBuf,
    /// The argument vector, passed verbatim.
    pub argv: Vec<String>,
    /// The child's entire environment: cleared, then exactly these pairs.
    pub env: Vec<(String, String)>,
    /// Working directory request. The exact-image host runner requires this to
    /// be `None`: all governed paths are absolute and the private directory is
    /// conveyed through the explicit environment instead of ambient process
    /// mutation.
    pub cwd: Option<PathBuf>,
    /// Bytes written to the child's stdin by [`ProcessRunner::run`] (then
    /// closed); `None` for a null stdin. [`ProcessRunner::start`] requires
    /// this field to be `None` and accepts ordered chunks through
    /// [`RunningProcess::write_stdin`].
    pub stdin: Option<Vec<u8>>,
    /// Wall-clock bound; on expiry the child is killed.
    pub timeout: Duration,
    /// Per-stream cap on captured stdout/stderr bytes; on overflow the child
    /// is killed with [`ProcessTermination::OutputLimitExceeded`].
    pub max_output_bytes: u64,
}

/// Why process supervision stopped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessTermination {
    /// The child exited; an OS exit code may be unavailable.
    Exited(Option<i32>),
    /// The wall-clock bound expired.
    TimedOut,
    /// stdout or stderr exceeded its capture cap.
    OutputLimitExceeded,
    /// Cooperative cancellation won before exit was observed.
    Cancelled,
}

/// What happened when a spawned process finished (or was stopped).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProcessOutcome {
    /// The authoritative terminal reason.
    pub termination: ProcessTermination,
    /// Captured stdout (up to the cap).
    pub stdout: Vec<u8>,
    /// Captured stderr (up to the cap).
    pub stderr: Vec<u8>,
}

impl ProcessOutcome {
    /// Whether the process ran to completion with exit code 0.
    #[must_use]
    pub fn success(&self) -> bool {
        self.termination == ProcessTermination::Exited(Some(0))
    }
}

/// Cooperative cancellation shared by a process session and its owner.
///
/// The std runner polls this token while the child is alive, including while
/// another thread is blocked feeding stdin. Cancelling therefore closes the
/// process side of a full pipe and unblocks bounded sink backpressure.
#[derive(Clone, Debug, Default)]
pub struct ProcessCancellation {
    cancelled: Arc<AtomicBool>,
}

impl ProcessCancellation {
    /// A fresh, live cancellation token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation. Repeated requests are idempotent.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// Bounds enforced by the process mechanism for incremental stdin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessStdinLimits {
    /// Largest accepted call to [`RunningProcess::write_stdin`].
    pub max_chunk_bytes: u64,
    /// Largest cumulative stdin payload.
    pub max_total_bytes: u64,
}

impl ProcessStdinLimits {
    /// Construct explicit stdin bounds.
    #[must_use]
    pub const fn new(max_chunk_bytes: u64, max_total_bytes: u64) -> Self {
        Self {
            max_chunk_bytes,
            max_total_bytes,
        }
    }
}

/// A process-mechanism failure (distinct from a process that ran and
/// failed, which is a [`ProcessOutcome`] with a nonzero code).
#[derive(Debug)]
pub enum ProcessError {
    /// The program could not be spawned at all. Only constructible where a
    /// spawn substrate exists; on wasm32 the boundary is absent and every
    /// request is [`Self::CapabilityAbsent`] instead.
    #[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
    Spawn {
        /// The program that failed to spawn.
        program: PathBuf,
        /// The underlying error.
        err: asupersync::process::ProcessError,
    },
    /// No process capability exists on this target at all: the named
    /// capability error (the D2 rule, mirroring
    /// [`crate::fetch::FetchError::CapabilityAbsent`]), never a silent
    /// fallback. [`NoProcessRunner`] — the structural default on wasm32,
    /// where the one sanctioned subprocess cannot exist — returns it for
    /// every request.
    CapabilityAbsent {
        /// The program that was requested.
        program: PathBuf,
    },
    /// I/O plumbing to the child failed mid-run.
    Plumbing {
        /// The program being run.
        program: PathBuf,
        /// What broke.
        detail: String,
    },
    /// A [`ScriptedRunner`] was asked for a program it has no script for.
    NotScripted {
        /// The unscripted program.
        program: PathBuf,
    },
    /// The spec's program path is not absolute. The mechanism refuses PATH
    /// resolution outright: the D2 boundary resolves its one tool to an
    /// absolute path (and content-hashes it into provenance) before any
    /// spawn, so an ambient `PATH` can never choose the executable.
    ProgramNotAbsolute {
        /// The offending program path.
        program: PathBuf,
    },
    /// Exact-image spawning deliberately has no current-directory mutation.
    ///
    /// The ffmpeg boundary uses absolute paths and an explicit private
    /// `TMPDIR`, so accepting an ambient working-directory request would only
    /// widen the process primitive.
    WorkingDirectoryUnsupported {
        /// The program whose invalid request was refused.
        program: PathBuf,
        /// The requested working directory.
        cwd: PathBuf,
    },
    /// A streaming start was supplied preloaded stdin bytes.
    ///
    /// Long-lived sessions receive input only through
    /// [`RunningProcess::write_stdin`], so accepting both channels would make
    /// byte order ambiguous.
    StreamingInputPreloaded {
        /// The program whose invalid specification was refused.
        program: PathBuf,
    },
    /// One stdin chunk exceeded the mechanism's declared bound.
    StdinChunkLimit {
        /// The program receiving input.
        program: PathBuf,
        /// Attempted bytes.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
    /// Cumulative stdin exceeded the mechanism's declared bound.
    StdinTotalLimit {
        /// The program receiving input.
        program: PathBuf,
        /// Attempted bytes.
        attempted: u64,
        /// Configured maximum.
        max: u64,
    },
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
            Self::Spawn { program, err } => {
                write!(f, "cannot spawn {}: {err}", program.display())
            }
            Self::CapabilityAbsent { program } => write!(
                f,
                "no ProcessRunner capability: cannot run {}; the process boundary \
                 must be explicitly enabled and handed to a native product \
                 (D2: ffmpeg is the only subprocess); under wasm32 render frames \
                 and encode on the host",
                program.display()
            ),
            Self::Plumbing { program, detail } => {
                write!(f, "I/O plumbing to {} failed: {detail}", program.display())
            }
            Self::NotScripted { program } => {
                write!(f, "no scripted outcome for {}", program.display())
            }
            Self::ProgramNotAbsolute { program } => {
                write!(
                    f,
                    "program path {} is not absolute; the process capability \
                     refuses PATH resolution (D2: resolve and fingerprint the \
                     tool first)",
                    program.display()
                )
            }
            Self::WorkingDirectoryUnsupported { program, cwd } => write!(
                f,
                "cannot spawn {} with working directory {}: the exact-image \
                 process capability accepts absolute paths and an explicit \
                 environment only",
                program.display(),
                cwd.display()
            ),
            Self::StreamingInputPreloaded { program } => write!(
                f,
                "streaming process {} must start without preloaded stdin bytes",
                program.display()
            ),
            Self::StdinChunkLimit {
                program,
                attempted,
                max,
            } => write!(
                f,
                "stdin chunk of {attempted} bytes for {} exceeds limit {max}",
                program.display()
            ),
            Self::StdinTotalLimit {
                program,
                attempted,
                max,
            } => write!(
                f,
                "stdin total of {attempted} bytes for {} exceeds limit {max}",
                program.display()
            ),
        }
    }
}

impl std::error::Error for ProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
            Self::Spawn { err, .. } => Some(err),
            _ => None,
        }
    }
}

/// A started child process with bounded, backpressured stdin.
///
/// Implementations must enforce the start specification's timeout and output
/// caps for the whole session, not only while [`Self::finish`] is waiting.
/// Dropping a live session must cancel and reap it.
pub trait RunningProcess: Send {
    /// Write the next ordered stdin chunk, blocking when the child applies
    /// backpressure.
    ///
    /// # Errors
    /// [`ProcessError`] when the pipe or supervision mechanism fails.
    fn write_stdin(&mut self, bytes: &[u8]) -> Result<(), ProcessError>;

    /// Close stdin, wait for the bounded child, and return its outcome.
    ///
    /// # Errors
    /// [`ProcessError`] when process supervision fails.
    fn finish(self: Box<Self>) -> Result<ProcessOutcome, ProcessError>;

    /// Cancel, close stdin, and reap the child.
    ///
    /// # Errors
    /// [`ProcessError`] when process supervision fails.
    fn cancel(self: Box<Self>) -> Result<(), ProcessError>;
}

/// The process capability.
pub trait ProcessRunner: Send + Sync {
    /// Stable identity and policy version of the mechanism this runner uses.
    ///
    /// The value must remain stable for the runner's lifetime and must match
    /// every successfully started process.
    fn mechanism(&self) -> ProcessMechanism;

    /// Start a long-lived process whose stdin is supplied incrementally.
    ///
    /// `spec.stdin` must be `None`; the returned session is the only stdin
    /// channel. `cancellation` is polled for the complete child lifetime.
    ///
    /// # Errors
    /// [`ProcessError`] when the process cannot be started.
    fn start(
        &self,
        spec: &ProcessSpec,
        cancellation: ProcessCancellation,
        stdin_limits: ProcessStdinLimits,
    ) -> Result<Box<dyn RunningProcess>, ProcessError>;

    /// Run the process to completion under the spec's bounds.
    ///
    /// # Errors
    /// [`ProcessError`] when the mechanism itself fails; a process that
    /// runs and exits nonzero is an `Ok` outcome.
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutcome, ProcessError> {
        let mut start_spec = spec.clone();
        let stdin = start_spec.stdin.take();
        let stdin_bytes = stdin.as_ref().map_or(0, Vec::len);
        let stdin_bytes = u64::try_from(stdin_bytes).unwrap_or(u64::MAX);
        let mut process = self.start(
            &start_spec,
            ProcessCancellation::new(),
            ProcessStdinLimits::new(stdin_bytes.max(1), stdin_bytes),
        )?;
        if let Some(bytes) = stdin
            && let Err(error) = process.write_stdin(&bytes)
        {
            let _ = process.cancel();
            return Err(error);
        }
        process.finish()
    }
}

/// How often the std runner polls the child while enforcing the timeout.
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Every runner (std and scripted alike) refuses relative program paths:
/// the trait contract, not an implementation detail.
fn require_absolute(spec: &ProcessSpec) -> Result<(), ProcessError> {
    if spec.program.is_absolute() {
        Ok(())
    } else {
        Err(ProcessError::ProgramNotAbsolute {
            program: spec.program.clone(),
        })
    }
}

/// Host implementation over asupersync's audited exact-image primitive.
/// Native exact-process builds only: otherwise the capability's structural
/// default is [`NoProcessRunner`].
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
#[derive(Clone, Copy, Debug, Default)]
pub struct StdProcessRunner;

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
fn kill_process_tree(child: &mut asupersync::process::ExactImageChild) -> std::io::Result<()> {
    match child.kill_process_tree() {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Drain one output pipe on its own thread, capturing up to `cap` bytes and
/// discarding the rest (so the child is never back-pressured into a pipe
/// deadlock). Sets `overflow` the moment the cap is exceeded — the poll loop
/// watches it and kills the child promptly. Returns the captured bytes.
#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
fn drain(
    mut pipe: impl std::io::Read + Send + 'static,
    cap: u64,
    overflow: Arc<AtomicBool>,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let cap = usize::try_from(cap).unwrap_or(usize::MAX);
        let mut captured = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            match pipe.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let room = cap.saturating_sub(captured.len());
                    captured.extend_from_slice(&buf[..n.min(room)]);
                    if n > room {
                        overflow.store(true, Ordering::Relaxed);
                        // Keep reading (and discarding) until the kill
                        // closes the pipe.
                    }
                }
            }
        }
        captured
    })
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
impl ProcessRunner for StdProcessRunner {
    fn mechanism(&self) -> ProcessMechanism {
        host_exact_image_mechanism()
    }

    fn start(
        &self,
        spec: &ProcessSpec,
        cancellation: ProcessCancellation,
        stdin_limits: ProcessStdinLimits,
    ) -> Result<Box<dyn RunningProcess>, ProcessError> {
        require_absolute(spec)?;
        if spec.stdin.is_some() {
            return Err(ProcessError::StreamingInputPreloaded {
                program: spec.program.clone(),
            });
        }
        if let Some(cwd) = &spec.cwd {
            return Err(ProcessError::WorkingDirectoryUnsupported {
                program: spec.program.clone(),
                cwd: cwd.clone(),
            });
        }

        // The trusted absolute executable capability is resolved,
        // native-image-attested, and fingerprinted by the ffmpeg boundary
        // before it reaches this runner. ExactImageCommand never performs PATH
        // lookup, shell fallback, or environment inheritance.
        let mut command = asupersync::process::ExactImageCommand::new(&spec.program);
        command
            .args(&spec.argv)
            .envs(spec.env.iter().map(|(key, value)| (key, value)));
        let mut child = command.spawn().map_err(|err| ProcessError::Spawn {
            program: spec.program.clone(),
            err,
        })?;
        let actual_mechanism = exact_image_mechanism(child.mechanism());
        if actual_mechanism != self.mechanism() {
            let _ = kill_process_tree(&mut child);
            let _ = child.wait();
            return Err(ProcessError::Plumbing {
                program: spec.program.clone(),
                detail: format!(
                    "exact-image mechanism mismatch: runner declared {}, child used {}",
                    self.mechanism().identity(),
                    actual_mechanism.identity()
                ),
            });
        }

        let plumbing = |detail: &str| ProcessError::Plumbing {
            program: spec.program.clone(),
            detail: detail.to_string(),
        };
        let Some(stdin) = child.take_stdin() else {
            let _ = kill_process_tree(&mut child);
            let _ = child.wait();
            return Err(plumbing("no stdin pipe"));
        };
        let overflow = Arc::new(AtomicBool::new(false));
        let Some(stdout) = child.take_stdout() else {
            let _ = kill_process_tree(&mut child);
            let _ = child.wait();
            return Err(plumbing("no stdout pipe"));
        };
        let Some(stderr) = child.take_stderr() else {
            let _ = kill_process_tree(&mut child);
            let _ = child.wait();
            return Err(plumbing("no stderr pipe"));
        };
        let stdout_thread = drain(stdout, spec.max_output_bytes, Arc::clone(&overflow));
        let stderr_thread = drain(stderr, spec.max_output_bytes, Arc::clone(&overflow));
        let program = spec.program.clone();
        let timeout = spec.timeout;
        let supervisor_cancellation = cancellation.clone();
        let (outcome_tx, outcome_rx) = mpsc::sync_channel(1);
        let supervisor = std::thread::spawn(move || {
            let result = supervise_child(
                child,
                &program,
                timeout,
                supervisor_cancellation,
                overflow,
                stdout_thread,
                stderr_thread,
            );
            let _ = outcome_tx.send(result);
        });
        Ok(Box::new(StdRunningProcess {
            program: spec.program.clone(),
            stdin: Some(stdin),
            outcome_rx,
            supervisor: Some(supervisor),
            cancellation,
            stdin_limits,
            stdin_bytes: 0,
            finished: false,
        }))
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
fn supervise_child(
    mut child: asupersync::process::ExactImageChild,
    program: &std::path::Path,
    timeout: Duration,
    cancellation: ProcessCancellation,
    overflow: Arc<AtomicBool>,
    stdout_thread: std::thread::JoinHandle<Vec<u8>>,
    stderr_thread: std::thread::JoinHandle<Vec<u8>>,
) -> Result<ProcessOutcome, ProcessError> {
    let start = Instant::now();
    let termination = loop {
        // Reaping the group leader can disarm group-based tree cleanup. Once
        // both inherited pipes close, kill the isolated tree before reaping.
        // For an already-exited leader this preserves its status while
        // terminating redirected descendants.
        if stdout_thread.is_finished() && stderr_thread.is_finished() {
            if let Err(error) = kill_process_tree(&mut child) {
                return Err(ProcessError::Plumbing {
                    program: program.to_path_buf(),
                    detail: format!("process-tree completion kill failed: {error}"),
                });
            }
            match child.try_wait() {
                Ok(Some(status)) => break ProcessTermination::Exited(status.code()),
                Ok(None) => {}
                Err(error) => {
                    let _ = kill_process_tree(&mut child);
                    return Err(ProcessError::Plumbing {
                        program: program.to_path_buf(),
                        detail: format!(
                            "try_wait after process-tree completion kill failed: {error}"
                        ),
                    });
                }
            }
        }
        let terminal = if overflow.load(Ordering::Acquire) {
            Some(ProcessTermination::OutputLimitExceeded)
        } else if cancellation.is_cancelled() {
            Some(ProcessTermination::Cancelled)
        } else if start.elapsed() >= timeout {
            Some(ProcessTermination::TimedOut)
        } else {
            None
        };
        if let Some(terminal) = terminal {
            if let Err(err) = kill_process_tree(&mut child) {
                return Err(ProcessError::Plumbing {
                    program: program.to_path_buf(),
                    detail: format!("process-tree kill failed: {err}"),
                });
            }
            break terminal;
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    // Reap after a terminal kill so no zombie outlives the session. A
    // pipe-closed completion observed through `try_wait` was already reaped.
    if !matches!(termination, ProcessTermination::Exited(_)) {
        child.wait().map_err(|error| ProcessError::Plumbing {
            program: program.to_path_buf(),
            detail: format!("process-tree reap failed: {error}"),
        })?;
    }
    let plumbing = |detail: &str| ProcessError::Plumbing {
        program: program.to_path_buf(),
        detail: detail.to_string(),
    };
    let stdout = stdout_thread
        .join()
        .map_err(|_| plumbing("stdout drain panicked"))?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| plumbing("stderr drain panicked"))?;
    // A cap can also trip between process exit and the final drain. Output
    // overflow is authoritative even when an exit was observed first.
    let termination = if overflow.load(Ordering::Acquire) {
        ProcessTermination::OutputLimitExceeded
    } else {
        termination
    };
    Ok(ProcessOutcome {
        termination,
        stdout,
        stderr,
    })
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
struct StdRunningProcess {
    program: PathBuf,
    stdin: Option<asupersync::process::ExactImageChildStdin>,
    outcome_rx: mpsc::Receiver<Result<ProcessOutcome, ProcessError>>,
    supervisor: Option<std::thread::JoinHandle<()>>,
    cancellation: ProcessCancellation,
    stdin_limits: ProcessStdinLimits,
    stdin_bytes: u64,
    finished: bool,
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
impl StdRunningProcess {
    fn wait(&mut self) -> Result<ProcessOutcome, ProcessError> {
        self.stdin.take();
        let outcome = self.outcome_rx.recv().map_err(|_| ProcessError::Plumbing {
            program: self.program.clone(),
            detail: "process supervisor exited without an outcome".to_string(),
        })?;
        if let Some(supervisor) = self.supervisor.take() {
            supervisor.join().map_err(|_| ProcessError::Plumbing {
                program: self.program.clone(),
                detail: "process supervisor panicked".to_string(),
            })?;
        }
        self.finished = true;
        outcome
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
impl RunningProcess for StdRunningProcess {
    fn write_stdin(&mut self, bytes: &[u8]) -> Result<(), ProcessError> {
        if self.cancellation.is_cancelled() {
            return Err(ProcessError::Plumbing {
                program: self.program.clone(),
                detail: "process was cancelled before stdin write".to_string(),
            });
        }
        let chunk_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if chunk_bytes > self.stdin_limits.max_chunk_bytes {
            return Err(ProcessError::StdinChunkLimit {
                program: self.program.clone(),
                attempted: chunk_bytes,
                max: self.stdin_limits.max_chunk_bytes,
            });
        }
        let attempted =
            self.stdin_bytes
                .checked_add(chunk_bytes)
                .ok_or(ProcessError::StdinTotalLimit {
                    program: self.program.clone(),
                    attempted: u64::MAX,
                    max: self.stdin_limits.max_total_bytes,
                })?;
        if attempted > self.stdin_limits.max_total_bytes {
            return Err(ProcessError::StdinTotalLimit {
                program: self.program.clone(),
                attempted,
                max: self.stdin_limits.max_total_bytes,
            });
        }
        self.stdin
            .as_mut()
            .ok_or_else(|| ProcessError::Plumbing {
                program: self.program.clone(),
                detail: "stdin is already closed".to_string(),
            })?
            .write_all(bytes)
            .map_err(|error| ProcessError::Plumbing {
                program: self.program.clone(),
                detail: format!("stdin write failed: {error}"),
            })?;
        self.stdin_bytes = attempted;
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<ProcessOutcome, ProcessError> {
        self.wait()
    }

    fn cancel(mut self: Box<Self>) -> Result<(), ProcessError> {
        self.cancellation.cancel();
        self.wait().map(|_| ())
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "exact-process"))]
impl Drop for StdRunningProcess {
    fn drop(&mut self) {
        if !self.finished {
            self.cancellation.cancel();
            let _ = self.wait();
        }
    }
}

/// The structurally-absent process capability (W5 wasm tier 1, fm-l97).
///
/// On wasm32 there is no process boundary: the one subprocess the engine
/// will ever spawn (D2: ffmpeg) cannot exist in a single-threaded browser
/// sandbox, so the capability is handed in as this fail-closed runner.
/// Every request is the named [`ProcessError::CapabilityAbsent`] error —
/// never a silent fallback (the D2 rule, mirroring
/// [`crate::fetch::NoNetwork`]); native outputs remain the only encode
/// sinks. Native hosts may also use it to narrow a sandboxed worker by
/// capability removal, and the deterministic lab to prove a subsystem
/// never reaches for a process.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoProcessRunner;

impl ProcessRunner for NoProcessRunner {
    fn mechanism(&self) -> ProcessMechanism {
        ProcessMechanism::ExactImageUnavailable
    }

    fn start(
        &self,
        spec: &ProcessSpec,
        _cancellation: ProcessCancellation,
        _stdin_limits: ProcessStdinLimits,
    ) -> Result<Box<dyn RunningProcess>, ProcessError> {
        Err(ProcessError::CapabilityAbsent {
            program: spec.program.clone(),
        })
    }
}

/// The test double: canned outcomes per program path, with a full log of
/// every spec it was asked to run.
#[derive(Debug, Default)]
pub struct ScriptedRunner {
    scripts: BTreeMap<PathBuf, ProcessOutcome>,
    runs: Arc<Mutex<Vec<ProcessSpec>>>,
}

impl ScriptedRunner {
    /// An empty scripted runner (every run is [`ProcessError::NotScripted`]).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Script the outcome for a program path.
    pub fn script(&mut self, program: impl Into<PathBuf>, outcome: ProcessOutcome) {
        self.scripts.insert(program.into(), outcome);
    }

    /// Every spec run so far, in order.
    #[must_use]
    pub fn runs(&self) -> Vec<ProcessSpec> {
        self.runs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ProcessRunner for ScriptedRunner {
    fn mechanism(&self) -> ProcessMechanism {
        ProcessMechanism::Scripted
    }

    fn start(
        &self,
        spec: &ProcessSpec,
        cancellation: ProcessCancellation,
        stdin_limits: ProcessStdinLimits,
    ) -> Result<Box<dyn RunningProcess>, ProcessError> {
        require_absolute(spec)?;
        if spec.stdin.is_some() {
            return Err(ProcessError::StreamingInputPreloaded {
                program: spec.program.clone(),
            });
        }
        let outcome =
            self.scripts
                .get(&spec.program)
                .cloned()
                .ok_or_else(|| ProcessError::NotScripted {
                    program: spec.program.clone(),
                })?;
        Ok(Box::new(ScriptedProcess {
            spec: spec.clone(),
            input: Vec::new(),
            outcome,
            runs: Arc::clone(&self.runs),
            cancellation,
            stdin_limits,
            stdin_bytes: 0,
            finished: false,
        }))
    }
}

struct ScriptedProcess {
    spec: ProcessSpec,
    input: Vec<u8>,
    outcome: ProcessOutcome,
    runs: Arc<Mutex<Vec<ProcessSpec>>>,
    cancellation: ProcessCancellation,
    stdin_limits: ProcessStdinLimits,
    stdin_bytes: u64,
    finished: bool,
}

impl ScriptedProcess {
    fn record(&mut self) {
        if self.finished {
            return;
        }
        if !self.input.is_empty() {
            self.spec.stdin = Some(std::mem::take(&mut self.input));
        }
        self.runs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(self.spec.clone());
        self.finished = true;
    }
}

impl RunningProcess for ScriptedProcess {
    fn write_stdin(&mut self, bytes: &[u8]) -> Result<(), ProcessError> {
        if self.cancellation.is_cancelled() {
            return Err(ProcessError::Plumbing {
                program: self.spec.program.clone(),
                detail: "scripted process was cancelled before stdin write".to_string(),
            });
        }
        let chunk_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if chunk_bytes > self.stdin_limits.max_chunk_bytes {
            return Err(ProcessError::StdinChunkLimit {
                program: self.spec.program.clone(),
                attempted: chunk_bytes,
                max: self.stdin_limits.max_chunk_bytes,
            });
        }
        let attempted =
            self.stdin_bytes
                .checked_add(chunk_bytes)
                .ok_or(ProcessError::StdinTotalLimit {
                    program: self.spec.program.clone(),
                    attempted: u64::MAX,
                    max: self.stdin_limits.max_total_bytes,
                })?;
        if attempted > self.stdin_limits.max_total_bytes {
            return Err(ProcessError::StdinTotalLimit {
                program: self.spec.program.clone(),
                attempted,
                max: self.stdin_limits.max_total_bytes,
            });
        }
        self.input.extend_from_slice(bytes);
        self.stdin_bytes = attempted;
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<ProcessOutcome, ProcessError> {
        self.record();
        let mut outcome = self.outcome.clone();
        if self.cancellation.is_cancelled() {
            outcome.termination = ProcessTermination::Cancelled;
        }
        Ok(outcome)
    }

    fn cancel(mut self: Box<Self>) -> Result<(), ProcessError> {
        self.cancellation.cancel();
        self.record();
        Ok(())
    }
}

impl Drop for ScriptedProcess {
    fn drop(&mut self) {
        self.record();
    }
}
