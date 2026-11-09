//! The one external-tool boundary (§14.3, D2, D-23): sandboxed,
//! fingerprinted, optional ffmpeg.
//!
//! Everything here rides fmn-platform's process capability — an argv-only
//! interface (no caller-supplied shell command), cleared environment,
//! wall-clock timeout with kill, and capped log capture. This module adds the
//! boundary-level protocol on top:
//!
//! - **Resolution + executable binding.** The tool is canonicalized and
//!   content-hashed (SHA-256), then `-version` is run from a verified private
//!   copy—not from the configured pathname. Every later probe and job repeats
//!   that exact-create/copy/hash operation, rehashes the private executable
//!   after use, and records both paths, the hash, version, encoder, and full
//!   argv in [`Provenance`], together with the selected exact-image process
//!   mechanism and its policy version. Installations that cannot execute
//!   after relocation are rejected explicitly.
//! - **Optionality as a capability error.** An absent ffmpeg yields
//!   [`BoundaryError::FfmpegUnavailable`] naming the native
//!   alternatives — never a silent format substitution.
//! - **Owned private directories.** Resolution claims one canonical private
//!   session root, and each probe/job atomically claims a child directory
//!   (mode `0700` on Unix). Recorded filesystem identities gate both later
//!   creation and cleanup: a collision or replaced path is retained untouched.
//!   The job directory is the child's `TMPDIR`; every tool and artifact path is
//!   absolute, so the child inherits the engine's working directory and avoids
//!   widening the exact-image capability with a requested `cwd`. The artifact
//!   reaches the destination only through atomic rename after verification.
//! - **Environment allowlist + locale pinning.** The child sees
//!   exactly `LANG=C`, `LC_ALL=C`, and `TMPDIR=<job dir>`.
//! - **Hardware encoders enter here and only here.** They are named,
//!   validated against the probed encoder list, and recorded in
//!   provenance; ffmpeg products are excluded from certification by
//!   construction, so none of this touches the determinism story.
//!
//! The filesystem side (job dirs, rename publication) uses
//! `std::fs` directly: the boundary is a host-only feature by
//! definition — a platform without subprocesses has no ffmpeg boundary
//! and uses the native outputs instead.
//!
//! Safe `std` has no cross-platform execute-by-handle or
//! directory-handle-relative recursive deletion. Within the caller-supplied
//! trusted workdir ancestry, the private `0700` hierarchy, identity checks, and
//! hashes close ambient-path substitution. A hostile owner of that ancestry or
//! an actor already running as the same OS identity could still race a pathname
//! between the final check and the OS operation. That stronger threat requires
//! a future host capability rather than an unsafe platform carve-out.

mod audio;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fmn_hash::{Digest, Sha256};
use fmn_platform::process::{
    FfmpegExecutable, FfmpegLocatorError, MAX_FFMPEG_EXECUTABLE_BYTES, NativeImageAttestation,
    ProcessCancellation, ProcessError, ProcessMechanism, ProcessOutcome, ProcessRunner,
    ProcessSpec, ProcessStdinLimits, ProcessTermination, RunningProcess,
};

use crate::negotiate::{NegotiationError, VideoJob, encode_argv, mux_argv, transcode_audio_argv};

/// The message every "ffmpeg is missing" error carries: the named
/// native alternative (D2 — a capability error, never a substitution).
pub const NATIVE_ALTERNATIVE: &str = "native outputs need no ffmpeg: y4m, PNG sequences, and GIF \
     are built in; ffmpeg is only required for encoded video (mp4/mov), \
     audio mux, and media transcode";

/// Hardware encoders the boundary recognizes (§14.3): reported by
/// `fmn doctor`, selectable only by explicit name.
pub const HARDWARE_ENCODERS: [&str; 6] = [
    "h264_videotoolbox",
    "hevc_videotoolbox",
    "prores_videotoolbox",
    "h264_nvenc",
    "hevc_nvenc",
    "av1_nvenc",
];

/// Typed refusals of the boundary.
#[derive(Debug)]
pub enum BoundaryError {
    /// ffmpeg is not at the resolved path — the capability error that
    /// names the native alternative.
    FfmpegUnavailable {
        /// Where the boundary looked.
        attempted: PathBuf,
        /// The native alternative, spelled out.
        alternative: &'static str,
    },
    /// A previously resolved executable no longer has its recorded bytes.
    ExecutableIdentityChanged {
        /// Canonical source or private bound path that was checked.
        path: PathBuf,
        /// Resolution-time SHA-256.
        expected_sha256: String,
        /// Current SHA-256, or `None` when the path could not be read.
        actual_sha256: Option<String>,
    },
    /// A selected source or exact private copy failed the governed native
    /// executable-image policy before it could be spawned.
    ExecutableImageRejected {
        /// Source or private-copy path that was inspected.
        path: PathBuf,
        /// Stable locator/parser or bounded-I/O diagnostic.
        detail: String,
    },
    /// The verified bytes cannot execute after relocation into the private
    /// session. This rejects installations whose loader contract depends on
    /// the configured executable's original directory.
    UnsupportedRelocatedExecutable {
        /// Canonical configured executable path.
        source: PathBuf,
        /// Spawn or process diagnostic from the bound-copy probe.
        detail: String,
    },
    /// The process mechanism itself failed.
    Mechanism(ProcessError),
    /// A probe (`-version`, `-encoders`) ran but its output was not
    /// usable.
    ProbeFailed(&'static str),
    /// A named encoder the installed ffmpeg does not offer.
    UnknownEncoder {
        /// The encoder that was requested.
        requested: String,
        /// The recognized hardware encoders this ffmpeg does offer.
        hardware_available: Vec<String>,
    },
    /// The job negotiation was refused.
    Negotiation(NegotiationError),
    /// The frame payload is not a whole number of wire frames.
    PayloadGeometry {
        /// Bytes one frame occupies on the wire.
        frame_bytes: usize,
        /// The payload length that failed the divisibility check.
        got: usize,
    },
    /// The wall-clock timeout expired and the child was killed.
    JobTimedOut {
        /// The configured timeout.
        timeout: Duration,
    },
    /// Cooperative cancellation stopped the invocation.
    Cancelled,
    /// The child's log output exceeded its cap and the child was
    /// killed.
    LogOverflow,
    /// ffmpeg ran and failed.
    EncodeFailed {
        /// The exit code, if any.
        code: Option<i32>,
        /// The tail of stderr, lossily decoded.
        stderr: String,
    },
    /// The job succeeded but produced no artifact.
    ArtifactMissing,
    /// The artifact exceeds the declared size budget; it is not
    /// published.
    ArtifactOversized {
        /// The artifact's size.
        bytes: u64,
        /// The configured budget.
        max: u64,
    },
    /// Private-directory or publication filesystem failure.
    Workdir {
        /// What broke.
        detail: String,
    },
}

impl std::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FfmpegUnavailable {
                attempted,
                alternative,
            } => write!(
                f,
                "ffmpeg not found at {}; {alternative}",
                attempted.display()
            ),
            Self::ExecutableIdentityChanged {
                path,
                expected_sha256,
                actual_sha256,
            } => write!(
                f,
                "ffmpeg executable identity changed at {} (expected SHA-256 {}, got {})",
                path.display(),
                expected_sha256,
                actual_sha256.as_deref().unwrap_or("unreadable")
            ),
            Self::ExecutableImageRejected { path, detail } => write!(
                f,
                "ffmpeg executable image at {} was rejected before spawn: {detail}",
                path.display()
            ),
            Self::UnsupportedRelocatedExecutable { source, detail } => write!(
                f,
                "ffmpeg at {} cannot execute as an identity-bound private copy: {detail}; \
                 use a relocatable/self-contained ffmpeg build",
                source.display()
            ),
            Self::Mechanism(e) => write!(f, "process mechanism: {e}"),
            Self::ProbeFailed(what) => write!(f, "ffmpeg probe failed: {what}"),
            Self::UnknownEncoder {
                requested,
                hardware_available,
            } => write!(
                f,
                "encoder {requested:?} is not offered by the installed ffmpeg \
                 (recognized hardware encoders present: {hardware_available:?})"
            ),
            Self::Negotiation(e) => write!(f, "{e}"),
            Self::PayloadGeometry { frame_bytes, got } => write!(
                f,
                "payload of {got} bytes is not a whole number of {frame_bytes}-byte frames"
            ),
            Self::JobTimedOut { timeout } => {
                write!(f, "ffmpeg exceeded its {}s timeout", timeout.as_secs())
            }
            Self::Cancelled => f.write_str("ffmpeg job was cancelled"),
            Self::LogOverflow => write!(f, "ffmpeg log output exceeded its cap"),
            Self::EncodeFailed { code, stderr } => {
                write!(f, "ffmpeg failed (code {code:?}): {stderr}")
            }
            Self::ArtifactMissing => write!(f, "ffmpeg succeeded but produced no artifact"),
            Self::ArtifactOversized { bytes, max } => write!(
                f,
                "artifact of {bytes} bytes exceeds the {max}-byte budget; not published"
            ),
            Self::Workdir { detail } => write!(f, "boundary workdir: {detail}"),
        }
    }
}

impl std::error::Error for BoundaryError {}

impl From<ProcessError> for BoundaryError {
    fn from(e: ProcessError) -> Self {
        Self::Mechanism(e)
    }
}

impl From<NegotiationError> for BoundaryError {
    fn from(e: NegotiationError) -> Self {
        Self::Negotiation(e)
    }
}

/// An opaque resolved tool plus its owned private-copy session.
#[derive(Debug, Clone)]
pub struct FfmpegTool {
    executable: FfmpegExecutable,
    path: PathBuf,
    sha256_hex: String,
    native_image: NativeImageAttestation,
    version: String,
    session: Arc<ToolSession>,
}

/// Probe timeouts are short: a probe is milliseconds of work.
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const PROBE_LOG_CAP: u64 = 1 << 20;
const FILE_HASH_BUFFER_BYTES: usize = 64 * 1024;

fn executable_image_error(path: &Path, error: FfmpegLocatorError) -> BoundaryError {
    BoundaryError::ExecutableImageRejected {
        path: path.to_path_buf(),
        detail: error.to_string(),
    }
}

fn sha256_open_file(
    file: &mut File,
    path: &Path,
    expected_bytes: u64,
) -> Result<String, BoundaryError> {
    file.seek(std::io::SeekFrom::Start(0)).map_err(|error| {
        BoundaryError::ExecutableImageRejected {
            path: path.to_path_buf(),
            detail: format!("seek exact executable handle: {error}"),
        }
    })?;
    let mut hasher = fmn_hash::sha256::Sha256::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_BYTES];
    let mut total = 0_u64;
    loop {
        let read =
            file.read(&mut buffer)
                .map_err(|error| BoundaryError::ExecutableImageRejected {
                    path: path.to_path_buf(),
                    detail: format!("read exact executable handle: {error}"),
                })?;
        if read == 0 {
            break;
        }
        total = total.checked_add(read as u64).ok_or_else(|| {
            BoundaryError::ExecutableImageRejected {
                path: path.to_path_buf(),
                detail: "executable byte count overflowed".to_string(),
            }
        })?;
        if total > MAX_FFMPEG_EXECUTABLE_BYTES {
            return Err(BoundaryError::ExecutableImageRejected {
                path: path.to_path_buf(),
                detail: format!(
                    "executable grew beyond the {}-byte limit while hashing",
                    MAX_FFMPEG_EXECUTABLE_BYTES
                ),
            });
        }
        hasher.update(&buffer[..read]);
    }
    if total != expected_bytes {
        return Err(BoundaryError::ExecutableImageRejected {
            path: path.to_path_buf(),
            detail: format!(
                "executable length changed through its opened handle (expected {expected_bytes}, read {total})"
            ),
        });
    }
    Ok(hasher.finalize().to_hex())
}

fn copy_and_hash_executable(
    source: &mut File,
    source_path: &Path,
    expected_bytes: u64,
    private: &mut File,
    private_path: &Path,
) -> Result<String, BoundaryError> {
    source.seek(std::io::SeekFrom::Start(0)).map_err(|error| {
        BoundaryError::ExecutableImageRejected {
            path: source_path.to_path_buf(),
            detail: format!("seek exact source handle: {error}"),
        }
    })?;
    let mut hasher = fmn_hash::sha256::Sha256::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_BYTES];
    let mut total = 0_u64;
    loop {
        let read =
            source
                .read(&mut buffer)
                .map_err(|error| BoundaryError::ExecutableImageRejected {
                    path: source_path.to_path_buf(),
                    detail: format!("read exact source handle: {error}"),
                })?;
        if read == 0 {
            break;
        }
        total = total.checked_add(read as u64).ok_or_else(|| {
            BoundaryError::ExecutableImageRejected {
                path: source_path.to_path_buf(),
                detail: "executable byte count overflowed".to_string(),
            }
        })?;
        if total > MAX_FFMPEG_EXECUTABLE_BYTES {
            return Err(BoundaryError::ExecutableImageRejected {
                path: source_path.to_path_buf(),
                detail: format!(
                    "executable grew beyond the {}-byte limit while copying",
                    MAX_FFMPEG_EXECUTABLE_BYTES
                ),
            });
        }
        private
            .write_all(&buffer[..read])
            .map_err(|error| BoundaryError::Workdir {
                detail: format!("copy executable to {}: {error}", private_path.display()),
            })?;
        hasher.update(&buffer[..read]);
    }
    if total != expected_bytes {
        return Err(BoundaryError::ExecutableImageRejected {
            path: source_path.to_path_buf(),
            detail: format!(
                "source length changed through its opened handle (expected {expected_bytes}, copied {total})"
            ),
        });
    }
    Ok(hasher.finalize().to_hex())
}

#[derive(Debug)]
struct BoundFfmpeg {
    path: PathBuf,
    file: File,
    native_image: NativeImageAttestation,
}

fn environment_allowlist(workdir: &Path) -> Vec<(String, String)> {
    vec![
        ("LANG".into(), "C".into()),
        ("LC_ALL".into(), "C".into()),
        ("TMPDIR".into(), workdir.display().to_string()),
    ]
}

fn probe_spec(tool: &Path, argv: &[&str], workdir: &Path) -> ProcessSpec {
    ProcessSpec {
        program: tool.to_path_buf(),
        argv: argv.iter().map(|s| (*s).to_string()).collect(),
        env: environment_allowlist(workdir),
        cwd: None,
        stdin: None,
        timeout: PROBE_TIMEOUT,
        max_output_bytes: PROBE_LOG_CAP,
    }
}

impl FfmpegTool {
    /// Consume an audited ffmpeg locator token, hash its currently validated
    /// source handle, claim a private session below `workdir_parent`, and
    /// probe `-version` from an exact native-image-attested private copy.
    ///
    /// # Errors
    /// [`BoundaryError::ExecutableImageRejected`] when the selected source or
    /// exact private copy is no longer a bounded host-native image;
    /// [`BoundaryError::UnsupportedRelocatedExecutable`] when the verified
    /// bytes cannot execute from the private hierarchy;
    /// [`BoundaryError::ProbeFailed`] when they execute but do not behave like
    /// ffmpeg; [`BoundaryError::Workdir`] when a private session cannot be
    /// proved.
    pub fn resolve(
        executable: FfmpegExecutable,
        runner: &dyn ProcessRunner,
        workdir_parent: &Path,
    ) -> Result<Self, BoundaryError> {
        let path = executable.canonical_path().to_path_buf();
        let (mut source, native_image) = executable
            .open_current()
            .map_err(|error| executable_image_error(&path, error))?;
        let sha256_hex = sha256_open_file(&mut source, &path, native_image.file_bytes)?;
        let session = ToolSession::create(workdir_parent)?;
        let mut tool = Self {
            executable,
            path,
            sha256_hex,
            native_image,
            version: String::new(),
            session,
        };
        let outcome = match tool.run_bound_probe(runner, &["-version"]) {
            Ok(outcome) => outcome,
            Err(BoundaryError::Mechanism(error)) => {
                return Err(BoundaryError::UnsupportedRelocatedExecutable {
                    source: tool.path.clone(),
                    detail: error.to_string(),
                });
            }
            Err(error) => return Err(error),
        };
        if !outcome.success() {
            return Err(BoundaryError::UnsupportedRelocatedExecutable {
                source: tool.path.clone(),
                detail: format!(
                    "-version exited {:?}: {}",
                    outcome.termination,
                    String::from_utf8_lossy(&outcome.stderr).trim()
                ),
            });
        }
        let first_line = outcome
            .stdout
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        let first_line = std::str::from_utf8(first_line)
            .map_err(|_| BoundaryError::ProbeFailed("-version output is not valid UTF-8"))?;
        let version = first_line.strip_suffix('\r').unwrap_or(first_line);
        if version.is_empty() {
            return Err(BoundaryError::ProbeFailed("-version produced no output"));
        }
        if !version.starts_with("ffmpeg version ") {
            return Err(BoundaryError::ProbeFailed(
                "-version first line is not an ffmpeg version banner",
            ));
        }
        if version.chars().any(char::is_control) {
            return Err(BoundaryError::ProbeFailed(
                "-version first line contains a control character",
            ));
        }
        tool.version = version.to_owned();
        Ok(tool)
    }

    /// Canonical configured executable path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Resolution-time executable SHA-256.
    #[must_use]
    pub fn sha256_hex(&self) -> &str {
        &self.sha256_hex
    }

    /// Structural native-image evidence for the resolution-time source.
    #[must_use]
    pub const fn native_image(&self) -> NativeImageAttestation {
        self.native_image
    }

    /// First line of the resolution-time `-version` probe.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    fn verify_source_identity(&self) -> Result<(), BoundaryError> {
        let (mut source, attestation) = self
            .executable
            .open_current()
            .map_err(|error| executable_image_error(&self.path, error))?;
        if attestation != self.native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: self.path.clone(),
                detail: format!(
                    "native-image attestation changed from {:?} to {:?}",
                    self.native_image, attestation
                ),
            });
        }
        let actual_sha256 = sha256_open_file(&mut source, &self.path, attestation.file_bytes)?;
        verify_digest(&self.path, &self.sha256_hex, actual_sha256)
    }

    fn bound_path(&self, workdir: &Path) -> PathBuf {
        #[cfg(windows)]
        let leaf = "fmn-bound-ffmpeg.exe";
        #[cfg(not(windows))]
        let leaf = "fmn-bound-ffmpeg";
        workdir.join(leaf)
    }

    fn bind_into(&self, workdir: &OwnedWorkdir) -> Result<BoundFfmpeg, BoundaryError> {
        self.bind_into_after_copy(workdir, |_, _| Ok(()))
    }

    fn bind_into_after_copy<F>(
        &self,
        workdir: &OwnedWorkdir,
        after_copy: F,
    ) -> Result<BoundFfmpeg, BoundaryError>
    where
        F: FnOnce(&mut File, &Path) -> Result<(), BoundaryError>,
    {
        workdir.verify_current("bind executable")?;
        let bound = self.bound_path(workdir.path());
        let (mut source, source_image) = self
            .executable
            .open_current()
            .map_err(|error| executable_image_error(&self.path, error))?;
        if source_image != self.native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: self.path.clone(),
                detail: format!(
                    "native-image attestation changed from {:?} to {:?}",
                    self.native_image, source_image
                ),
            });
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o500);
        }
        let mut private = options
            .open(&bound)
            .map_err(|error| BoundaryError::Workdir {
                detail: format!("create bound executable {}: {error}", bound.display()),
            })?;
        let copied_sha256 = copy_and_hash_executable(
            &mut source,
            &self.path,
            source_image.file_bytes,
            &mut private,
            &bound,
        )?;
        verify_digest(&self.path, &self.sha256_hex, copied_sha256)?;
        after_copy(&mut private, &bound)?;
        private.flush().map_err(|error| BoundaryError::Workdir {
            detail: format!("flush bound executable {}: {error}", bound.display()),
        })?;
        private.sync_all().map_err(|error| BoundaryError::Workdir {
            detail: format!("sync bound executable {}: {error}", bound.display()),
        })?;
        #[cfg(unix)]
        private
            .set_permissions(
                <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o500),
            )
            .map_err(|error| BoundaryError::Workdir {
                detail: format!("protect bound executable {}: {error}", bound.display()),
            })?;
        let native_image = self
            .executable
            .attest_private_copy(&mut private, &bound)
            .map_err(|error| executable_image_error(&bound, error))?;
        if native_image != self.native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: bound,
                detail: format!(
                    "private-copy attestation {:?} differs from selected source {:?}",
                    native_image, self.native_image
                ),
            });
        }
        let private_sha256 = sha256_open_file(&mut private, &bound, native_image.file_bytes)?;
        verify_digest(&bound, &self.sha256_hex, private_sha256)?;
        // Linux refuses exec while any process retains a write-capable handle
        // to the image (`ETXTBSY`). The exact create-new handle has now been
        // flushed, synced, permissioned, parsed, and hashed; close it, then
        // reopen and retain a read-only attested handle across spawn.
        drop(private);
        let (mut retained, retained_image) = self
            .executable
            .open_private_copy(&bound)
            .map_err(|error| executable_image_error(&bound, error))?;
        if retained_image != native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: bound,
                detail: format!(
                    "read-only private handle attests as {:?}, create-new handle attested as {:?}",
                    retained_image, native_image
                ),
            });
        }
        let retained_sha256 = sha256_open_file(&mut retained, &bound, retained_image.file_bytes)?;
        verify_digest(&bound, &self.sha256_hex, retained_sha256)?;
        workdir.verify_current("finish executable binding")?;
        Ok(BoundFfmpeg {
            path: bound,
            file: retained,
            native_image: retained_image,
        })
    }

    fn run_bound_probe(
        &self,
        runner: &dyn ProcessRunner,
        argv: &[&str],
    ) -> Result<ProcessOutcome, BoundaryError> {
        self.run_bound_probe_after_copy(runner, argv, |_, _| Ok(()))
    }

    fn run_bound_probe_after_copy<F>(
        &self,
        runner: &dyn ProcessRunner,
        argv: &[&str],
        after_copy: F,
    ) -> Result<ProcessOutcome, BoundaryError>
    where
        F: FnOnce(&mut File, &Path) -> Result<(), BoundaryError>,
    {
        let workdir = self.session.make_probe_workdir()?;
        let result = (|| {
            let mut bound = self.bind_into_after_copy(&workdir, after_copy)?;
            bound.verify_current(self)?;
            let outcome = runner.run(&probe_spec(bound.path(), argv, workdir.path()));
            workdir.verify_current("finish executable probe")?;
            bound.verify_current(self)?;
            self.verify_source_identity()?;
            outcome.map_err(BoundaryError::from)
        })();
        workdir.cleanup_recursive();
        result
    }
}

impl BoundFfmpeg {
    fn path(&self) -> &Path {
        &self.path
    }

    fn verify_current(&mut self, tool: &FfmpegTool) -> Result<(), BoundaryError> {
        let handle_image = tool
            .executable
            .attest_private_copy(&mut self.file, &self.path)
            .map_err(|error| executable_image_error(&self.path, error))?;
        if handle_image != self.native_image || handle_image != tool.native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: self.path.clone(),
                detail: format!(
                    "retained private handle attests as {:?}, expected {:?}",
                    handle_image, tool.native_image
                ),
            });
        }
        let handle_sha256 = sha256_open_file(&mut self.file, &self.path, handle_image.file_bytes)?;
        verify_digest(&self.path, tool.sha256_hex(), handle_sha256)?;

        let (mut current, path_image) = tool
            .executable
            .open_private_copy(&self.path)
            .map_err(|error| executable_image_error(&self.path, error))?;
        if path_image != self.native_image || path_image != tool.native_image {
            return Err(BoundaryError::ExecutableImageRejected {
                path: self.path.clone(),
                detail: format!(
                    "private pathname attests as {:?}, expected {:?}",
                    path_image, tool.native_image
                ),
            });
        }
        let path_sha256 = sha256_open_file(&mut current, &self.path, path_image.file_bytes)?;
        verify_digest(&self.path, tool.sha256_hex(), path_sha256)
    }
}

fn verify_digest(
    path: &Path,
    expected_sha256: &str,
    actual_sha256: String,
) -> Result<(), BoundaryError> {
    if actual_sha256 == expected_sha256 {
        return Ok(());
    }
    Err(BoundaryError::ExecutableIdentityChanged {
        path: path.to_path_buf(),
        expected_sha256: expected_sha256.to_string(),
        actual_sha256: Some(actual_sha256),
    })
}

/// The installed ffmpeg's encoder inventory.
#[derive(Debug, Clone, Default)]
pub struct EncoderCapabilities {
    names: std::collections::BTreeSet<String>,
}

impl EncoderCapabilities {
    /// Probe `-encoders` and parse the inventory.
    ///
    /// # Errors
    /// [`BoundaryError`] when the probe cannot run or parse.
    pub fn probe(tool: &FfmpegTool, runner: &dyn ProcessRunner) -> Result<Self, BoundaryError> {
        let outcome = tool.run_bound_probe(runner, &["-hide_banner", "-encoders"])?;
        if !outcome.success() {
            return Err(BoundaryError::ProbeFailed("-encoders exited nonzero"));
        }
        Ok(Self::parse(&String::from_utf8_lossy(&outcome.stdout)))
    }

    /// Parse `-encoders` output: after the `------` separator, each
    /// line is ` FLAGS name description`.
    #[must_use]
    pub fn parse(listing: &str) -> Self {
        let mut names = std::collections::BTreeSet::new();
        let mut seen_separator = false;
        for line in listing.lines() {
            if !seen_separator {
                seen_separator = line.trim_start().starts_with("---");
                continue;
            }
            let mut fields = line.split_whitespace();
            let (Some(_flags), Some(name)) = (fields.next(), fields.next()) else {
                continue;
            };
            names.insert(name.to_string());
        }
        Self { names }
    }

    /// Whether `encoder` is offered.
    #[must_use]
    pub fn offers(&self, encoder: &str) -> bool {
        self.names.contains(encoder)
    }

    /// The recognized hardware encoders present in this inventory
    /// (`fmn doctor`'s report).
    #[must_use]
    pub fn hardware(&self) -> Vec<String> {
        HARDWARE_ENCODERS
            .iter()
            .filter(|name| self.offers(name))
            .map(|s| (*s).to_string())
            .collect()
    }
}

/// Per-job resource bounds.
#[derive(Debug, Clone)]
pub struct JobLimits {
    /// Wall-clock bound on each ffmpeg invocation.
    pub timeout: Duration,
    /// Cap on captured stdout/stderr per stream.
    pub max_log_bytes: u64,
    /// Cap on the produced artifact's size; larger is refused, not
    /// published.
    pub max_artifact_bytes: u64,
    /// Keep the job-scoped working directory after the job (the repro-
    /// bundle hook; default false).
    pub keep_workdir: bool,
}

impl Default for JobLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(600),
            max_log_bytes: 1 << 20,
            max_artifact_bytes: 8 << 30,
            keep_workdir: false,
        }
    }
}

/// The provenance record of one ffmpeg invocation.
///
/// This names the canonical configured identity, the private executable copy
/// bound to this job, and the exact argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// Canonical configured tool path.
    pub tool_path: PathBuf,
    /// SHA-256 shared by the configured tool and private bound copy.
    pub tool_sha256_hex: String,
    /// Structural native-image evidence observed from the exact retained
    /// private-copy handle under the versioned parser policy.
    pub native_image: NativeImageAttestation,
    /// The tool's `-version` first line.
    pub tool_version: String,
    /// Private job-scoped executable path actually passed to the process
    /// mechanism.
    pub bound_tool_path: PathBuf,
    /// Stable identity of the exact process mechanism selected for this run.
    pub process_mechanism: String,
    /// Version of that mechanism's executable-selection and containment
    /// policy.
    pub process_policy_version: u32,
    /// The resolved encoder (`None` for muxer-level modes like GIF and
    /// stream-copy jobs).
    pub encoder: Option<String>,
    /// The complete argv, verbatim.
    pub argv: Vec<String>,
}

/// One completed invocation within a boundary job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationReport {
    /// The provenance record.
    pub provenance: Provenance,
    /// The captured stderr log (up to the cap).
    pub stderr: Vec<u8>,
    /// Private artifact path named by this invocation's argv.
    pub artifact: PathBuf,
}

/// A completed and published boundary job.
#[derive(Debug)]
pub struct BoundaryReport {
    /// Every invocation in execution order (video encode, then optional mux).
    pub invocations: Vec<InvocationReport>,
    /// The atomically published destination.
    pub destination: PathBuf,
    /// Exact published artifact bytes.
    pub artifact_bytes: u64,
    /// SHA-256 of the exact published artifact.
    pub artifact_digest: Digest,
}

const WORKDIR_CREATE_ATTEMPTS: u64 = 128;

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt as _};

    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o700).create(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

/// Windows (and every other non-Unix target) fails closed before any ffmpeg
/// probe or job is spawned, exactly as published v0.4.0 did.
///
/// DLL loader boundary (issue #3): the exact-image process mechanism launches
/// the relocated `fmn-bound-ffmpeg.exe` with `CreateProcessW` and a NULL
/// current-directory argument, so the child inherits the *caller's* working
/// directory. Under the default `SafeDllSearchMode` the Windows loader searches
/// the current directory for any non-KnownDLL that the executable imports or
/// later loads by module name. Relocating and hashing the EXE binds only the
/// EXE: its DLL closure is unbound, and a DLL planted in a directory the user
/// happens to run `fmn doctor` from could be loaded into the probe.
/// `SetDefaultDllDirectories` / `LOAD_LIBRARY_SEARCH_*` only govern the calling
/// process, and mutating this process's cwd or DLL directory to influence the
/// child would be a process-global race in a multithreaded renderer. `std::fs`
/// also cannot create a directory with a provably private ACL.
///
/// Until the process mechanism can start the child with a controlled current
/// directory and a bounded, verified DLL closure (proved on native Windows),
/// the private workdir is refused here, so every `FfmpegTool::resolve` (the
/// default `doctor` probe included) reports a `Workdir` refusal instead of
/// executing a native image whose loader search we cannot bound.
#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "cannot prove a private ffmpeg workdir for {} on this target \
             (Windows child DLL search would include the caller's working directory)",
            path.display()
        ),
    ))
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectoryIdentity {
    volume_serial: u32,
    file_index: u64,
}

#[cfg(not(any(unix, windows)))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DirectoryIdentity;

fn directory_identity(path: &Path) -> Result<DirectoryIdentity, BoundaryError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| BoundaryError::Workdir {
        detail: format!("inspect claimed directory {}: {error}", path.display()),
    })?;
    if !metadata.file_type().is_dir() {
        return Err(BoundaryError::Workdir {
            detail: format!(
                "claimed workdir {} is no longer a directory",
                path.display()
            ),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;

        Ok(DirectoryIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;

        let Some(volume_serial) = metadata.volume_serial_number() else {
            return Err(BoundaryError::Workdir {
                detail: format!(
                    "filesystem supplied no volume identity for {}",
                    path.display()
                ),
            });
        };
        let Some(file_index) = metadata.file_index() else {
            return Err(BoundaryError::Workdir {
                detail: format!(
                    "filesystem supplied no file identity for {}",
                    path.display()
                ),
            });
        };
        Ok(DirectoryIdentity {
            volume_serial,
            file_index,
        })
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = metadata;
        Ok(DirectoryIdentity)
    }
}

#[derive(Clone, Debug)]
struct OwnedWorkdir {
    path: PathBuf,
    identity: DirectoryIdentity,
}

impl std::ops::Deref for OwnedWorkdir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.path()
    }
}

impl OwnedWorkdir {
    fn path(&self) -> &Path {
        &self.path
    }

    fn owns_current_path(&self) -> bool {
        directory_identity(&self.path).is_ok_and(|identity| identity == self.identity)
    }

    fn verify_current(&self, operation: &str) -> Result<(), BoundaryError> {
        if self.owns_current_path() {
            Ok(())
        } else {
            Err(BoundaryError::Workdir {
                detail: format!(
                    "claimed directory identity changed at {} before {operation}",
                    self.path.display()
                ),
            })
        }
    }

    fn cleanup_recursive(&self) {
        if self.owns_current_path() {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn remove_if_empty(&self) {
        if self.owns_current_path() {
            let _ = std::fs::remove_dir(&self.path);
        }
    }
}

fn ensure_private_workdir_parent(path: &Path) -> Result<(), BoundaryError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(BoundaryError::Workdir {
                detail: format!("inspect workdir parent {}: {error}", path.display()),
            });
        }
    }
    let parent = path.parent().ok_or_else(|| BoundaryError::Workdir {
        detail: format!(
            "workdir parent {} has no creatable ancestor",
            path.display()
        ),
    })?;
    ensure_private_workdir_parent(parent)?;
    match create_private_directory(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(BoundaryError::Workdir {
            detail: format!("create private workdir parent {}: {error}", path.display()),
        }),
    }
}

fn canonical_workdir_parent(path: &Path) -> Result<PathBuf, BoundaryError> {
    if path
        .components()
        .any(|component| component == std::path::Component::ParentDir)
    {
        return Err(BoundaryError::Workdir {
            detail: format!(
                "workdir parent {} contains a parent-directory component",
                path.display()
            ),
        });
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| BoundaryError::Workdir {
                detail: format!("resolve current directory for {}: {error}", path.display()),
            })?
            .join(path)
    };
    ensure_private_workdir_parent(&absolute)?;
    let canonical = std::fs::canonicalize(&absolute).map_err(|error| BoundaryError::Workdir {
        detail: format!("canonicalize workdir parent {}: {error}", path.display()),
    })?;
    let metadata = std::fs::metadata(&canonical).map_err(|error| BoundaryError::Workdir {
        detail: format!(
            "inspect canonical workdir parent {}: {error}",
            canonical.display()
        ),
    })?;
    if !metadata.is_dir() {
        return Err(BoundaryError::Workdir {
            detail: format!(
                "canonical workdir parent {} is not a directory",
                canonical.display()
            ),
        });
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        for ancestor in canonical.ancestors() {
            let metadata = std::fs::metadata(ancestor).map_err(|error| BoundaryError::Workdir {
                detail: format!("inspect workdir ancestor {}: {error}", ancestor.display()),
            })?;
            let mode = metadata.permissions().mode();
            if mode & 0o022 != 0 && mode & 0o1000 == 0 {
                return Err(BoundaryError::Workdir {
                    detail: format!(
                        "workdir ancestor {} is group/other-writable without the sticky bit",
                        ancestor.display()
                    ),
                });
            }
        }
    }
    Ok(canonical)
}

fn claim_owned_directory(path: &Path) -> Result<Option<OwnedWorkdir>, BoundaryError> {
    match create_private_directory(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(None),
        Err(error) => {
            return Err(BoundaryError::Workdir {
                detail: format!("create exclusive {}: {error}", path.display()),
            });
        }
    }
    let identity = directory_identity(path)?;
    Ok(Some(OwnedWorkdir {
        path: path.to_path_buf(),
        identity,
    }))
}

#[derive(Debug)]
struct ToolSession {
    parent: PathBuf,
    root: OwnedWorkdir,
    next_probe: AtomicU64,
}

impl ToolSession {
    fn create(parent: &Path) -> Result<Arc<Self>, BoundaryError> {
        let parent = canonical_workdir_parent(parent)?;
        // A name discriminator, not an authentication secret: exclusive
        // directory creation, permissions and identity checks grant ownership.
        let process_id = std::process::id();
        for sequence in 0..WORKDIR_CREATE_ATTEMPTS {
            let root = parent.join(format!("fmn-ffmpeg-session-{process_id}-{sequence}"));
            if let Some(root) = claim_owned_directory(&root)? {
                return Ok(Arc::new(Self {
                    parent,
                    root,
                    next_probe: AtomicU64::new(0),
                }));
            }
        }
        Err(BoundaryError::Workdir {
            detail: format!(
                "could not claim a private ffmpeg session under {} after \
                 {WORKDIR_CREATE_ATTEMPTS} collisions",
                parent.display()
            ),
        })
    }

    fn verify_root(&self) -> Result<(), BoundaryError> {
        if self.root.owns_current_path() {
            Ok(())
        } else {
            Err(BoundaryError::Workdir {
                detail: format!(
                    "private ffmpeg session identity changed at {}; refusing path-based work",
                    self.root.path().display()
                ),
            })
        }
    }

    fn claim_child(&self, path: &Path) -> Result<Option<OwnedWorkdir>, BoundaryError> {
        self.verify_root()?;
        let claimed = claim_owned_directory(path)?;
        self.verify_root()?;
        Ok(claimed)
    }

    fn make_probe_workdir(&self) -> Result<OwnedWorkdir, BoundaryError> {
        let process_id = std::process::id();
        for _ in 0..WORKDIR_CREATE_ATTEMPTS {
            let sequence = self.next_probe.fetch_add(1, Ordering::Relaxed);
            let path = self
                .root
                .path()
                .join(format!("fmn-probe-{process_id}-{sequence}"));
            if let Some(workdir) = self.claim_child(&path)? {
                return Ok(workdir);
            }
        }
        Err(BoundaryError::Workdir {
            detail: format!(
                "could not claim an executable-probe directory under {} after \
                 {WORKDIR_CREATE_ATTEMPTS} collisions",
                self.root.path().display()
            ),
        })
    }
}

impl Drop for ToolSession {
    fn drop(&mut self) {
        // Never recurse at the session level: retained repro workdirs keep the
        // root nonempty, and a replaced/nonempty root is left untouched.
        self.root.remove_if_empty();
    }
}

/// The boundary: one resolved tool + one process runner + bounds.
pub struct Boundary {
    tool: FfmpegTool,
    runner: Arc<dyn ProcessRunner>,
    limits: JobLimits,
    next_job: AtomicU64,
}

impl Boundary {
    /// A boundary over a resolved tool.
    ///
    /// # Errors
    /// The supplied workdir parent is unavailable or differs from the
    /// canonical parent used to prove the tool's private-copy execution.
    pub fn new(
        tool: FfmpegTool,
        runner: Arc<dyn ProcessRunner>,
        limits: JobLimits,
        workdir_parent: PathBuf,
    ) -> Result<Self, BoundaryError> {
        let workdir_parent = canonical_workdir_parent(&workdir_parent)?;
        // ubs:ignore — compares public canonical filesystem paths, not secrets.
        if workdir_parent != tool.session.parent {
            return Err(BoundaryError::Workdir {
                detail: format!(
                    "boundary workdir parent {} differs from the resolved tool parent {}",
                    workdir_parent.display(),
                    tool.session.parent.display()
                ),
            });
        }
        tool.session.verify_root()?;
        Ok(Self {
            tool,
            runner,
            limits,
            next_job: AtomicU64::new(0),
        })
    }

    /// The resolved tool.
    #[must_use]
    pub const fn tool(&self) -> &FfmpegTool {
        &self.tool
    }

    /// Start a negotiated long-lived encode whose stdin is supplied in
    /// ordered chunks.
    ///
    /// The returned job writes only inside its job-scoped work directory.
    /// [`StreamingEncode::prepare`] closes stdin, waits, verifies, and runs an
    /// optional audio mux without publishing. Only
    /// [`PreparedFfmpegArtifact::commit`] can mutate `destination`.
    ///
    /// # Errors
    /// Negotiation, capability, work-directory, argv, or process-start
    /// refusals.
    pub fn start_encode(
        &self,
        job: &VideoJob,
        caps: &EncoderCapabilities,
        destination: &Path,
        audio: Option<&Path>,
        cancellation: ProcessCancellation,
        stdin_limits: ProcessStdinLimits,
    ) -> Result<StreamingEncode, BoundaryError> {
        let encoder = job.resolved_encoder()?;
        if let Some(name) = &encoder
            && !caps.offers(name)
        {
            return Err(BoundaryError::UnknownEncoder {
                requested: name.clone(),
                hardware_available: caps.hardware(),
            });
        }
        let audio = audio
            .map(|input| {
                std::fs::canonicalize(input).map_err(|error| BoundaryError::Workdir {
                    detail: format!("resolve audio input {}: {error}", input.display()),
                })
            })
            .transpose()?;
        let workdir = self.make_workdir()?;
        let mut bound_tool = match self.tool.bind_into(&workdir) {
            Ok(bound_tool) => bound_tool,
            Err(error) => {
                self.cleanup(&workdir);
                return Err(error);
            }
        };
        let video_artifact = if audio.is_some() {
            workdir.join(format!("video.{}", job.container.extension()))
        } else {
            workdir.join(format!("out.{}", job.container.extension()))
        };
        let argv = match encode_argv(job, &video_artifact) {
            Ok(argv) => argv,
            Err(error) => {
                self.cleanup(&workdir);
                return Err(error.into());
            }
        };
        if let Err(error) = workdir.verify_current("start encode") {
            self.cleanup(&workdir);
            return Err(error);
        }
        if let Err(error) = bound_tool.verify_current(&self.tool) {
            self.cleanup(&workdir);
            return Err(error);
        }
        let spec = self.spec(bound_tool.path(), argv.clone(), &workdir, None);
        let process = match self.runner.start(&spec, cancellation.clone(), stdin_limits) {
            Ok(process) => process,
            Err(error) => {
                self.cleanup(&workdir);
                return Err(error.into());
            }
        };
        let post_start = workdir
            .verify_current("finish encode spawn")
            .and_then(|()| bound_tool.verify_current(&self.tool));
        if let Err(error) = post_start {
            let _ = process.cancel();
            self.cleanup(&workdir);
            return Err(error);
        }
        Ok(StreamingEncode {
            tool: self.tool.clone(),
            runner: Arc::clone(&self.runner),
            limits: self.limits.clone(),
            workdir,
            bound_tool,
            destination: destination.to_path_buf(),
            video_artifact,
            audio,
            encoder,
            video_argv: argv,
            process: Some(process),
            cancellation,
            cleanup: true,
        })
    }

    fn make_workdir(&self) -> Result<OwnedWorkdir, BoundaryError> {
        let process_id = std::process::id();
        for _ in 0..WORKDIR_CREATE_ATTEMPTS {
            let sequence = self.next_job.fetch_add(1, Ordering::Relaxed);
            let dir = self
                .tool
                .session
                .root
                .path()
                .join(format!("fmn-job-{process_id}-{sequence}"));
            if let Some(workdir) = self.tool.session.claim_child(&dir)? {
                return Ok(workdir);
            }
        }
        Err(BoundaryError::Workdir {
            detail: format!(
                "could not claim an exclusive job directory under {} after \
                 {WORKDIR_CREATE_ATTEMPTS} collisions",
                self.tool.session.root.path().display()
            ),
        })
    }

    /// The environment allowlist (D2): locale pinned, `TMPDIR` inside
    /// the job-scoped directory, nothing else.
    fn env_allowlist(workdir: &Path) -> Vec<(String, String)> {
        environment_allowlist(workdir)
    }

    fn spec(
        &self,
        bound_tool: &Path,
        argv: Vec<String>,
        workdir: &Path,
        stdin: Option<Vec<u8>>,
    ) -> ProcessSpec {
        ProcessSpec {
            program: bound_tool.to_path_buf(),
            argv,
            env: Self::env_allowlist(workdir),
            cwd: None,
            stdin,
            timeout: self.limits.timeout,
            max_output_bytes: self.limits.max_log_bytes,
        }
    }

    /// Map a finished outcome to success or a typed refusal.
    fn check_outcome(&self, outcome: &ProcessOutcome) -> Result<(), BoundaryError> {
        check_process_outcome(&self.limits, outcome)
    }

    /// Verify the artifact and publish it to `destination` by atomic
    /// rename. A failure at any point leaves the destination untouched.
    fn publish(&self, artifact: &Path, destination: &Path) -> Result<(), BoundaryError> {
        let meta = std::fs::metadata(artifact).map_err(|_| BoundaryError::ArtifactMissing)?;
        if meta.len() > self.limits.max_artifact_bytes {
            return Err(BoundaryError::ArtifactOversized {
                bytes: meta.len(),
                max: self.limits.max_artifact_bytes,
            });
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|e| BoundaryError::Workdir {
                detail: format!("create {}: {e}", parent.display()),
            })?;
        }
        std::fs::rename(artifact, destination).map_err(|e| BoundaryError::Workdir {
            detail: format!(
                "publish {} -> {}: {e}",
                artifact.display(),
                destination.display()
            ),
        })
    }

    fn cleanup(&self, workdir: &OwnedWorkdir) {
        if !self.limits.keep_workdir {
            workdir.cleanup_recursive();
        }
    }

    /// Run one invocation inside a job-scoped dir, then verify + publish
    /// the artifact it should have produced.
    fn run_publishing(
        &self,
        bound_tool: &mut BoundFfmpeg,
        argv: Vec<String>,
        workdir: &OwnedWorkdir,
        artifact: &Path,
        destination: &Path,
        encoder: Option<String>,
    ) -> Result<BoundaryReport, BoundaryError> {
        workdir.verify_current("start ffmpeg invocation")?;
        bound_tool.verify_current(&self.tool)?;
        let spec = self.spec(bound_tool.path(), argv.clone(), workdir, None);
        let outcome = self.runner.run(&spec);
        workdir.verify_current("finish ffmpeg invocation")?;
        bound_tool.verify_current(&self.tool)?;
        let outcome = outcome?;
        self.check_outcome(&outcome)?;
        workdir.verify_current("publish ffmpeg artifact")?;
        let (artifact_bytes, artifact_digest) =
            hash_private_artifact(artifact, self.limits.max_artifact_bytes)?;
        self.publish(artifact, destination)?;
        Ok(BoundaryReport {
            invocations: vec![invocation_report(
                &self.tool,
                bound_tool.path(),
                self.runner.mechanism(),
                encoder,
                argv,
                artifact.to_path_buf(),
                outcome.stderr,
            )],
            destination: destination.to_path_buf(),
            artifact_bytes,
            artifact_digest,
        })
    }

    /// Encode `frames` (concatenated tightly-packed wire frames, output
    /// orientation) to `destination`.
    ///
    /// # Errors
    /// Every refusal in [`BoundaryError`]; the destination is written
    /// only on success, atomically.
    pub fn encode(
        &self,
        job: &VideoJob,
        frames: Vec<u8>,
        caps: &EncoderCapabilities,
        destination: &Path,
    ) -> Result<BoundaryReport, BoundaryError> {
        let frame_bytes = job.wire.frame_bytes(job.width, job.height);
        if frame_bytes == 0 || frames.is_empty() || !frames.len().is_multiple_of(frame_bytes) {
            return Err(BoundaryError::PayloadGeometry {
                frame_bytes,
                got: frames.len(),
            });
        }
        let bytes = u64::try_from(frames.len()).unwrap_or(u64::MAX);
        let mut encode = self.start_encode(
            job,
            caps,
            destination,
            None,
            ProcessCancellation::new(),
            ProcessStdinLimits::new(bytes, bytes),
        )?;
        encode.write_stdin(&frames)?;
        encode.prepare()?.commit()
    }

    /// The two-stage audio mux: stage 1 encodes video into the job-scoped
    /// dir; stage 2 muxes with `-c:v copy` (never re-encoding video)
    /// and publishes.
    ///
    /// # Errors
    /// Every refusal in [`BoundaryError`].
    pub fn encode_with_audio(
        &self,
        job: &VideoJob,
        frames: Vec<u8>,
        audio: &Path,
        caps: &EncoderCapabilities,
        destination: &Path,
    ) -> Result<BoundaryReport, BoundaryError> {
        let frame_bytes = job.wire.frame_bytes(job.width, job.height);
        if frame_bytes == 0 || frames.is_empty() || !frames.len().is_multiple_of(frame_bytes) {
            return Err(BoundaryError::PayloadGeometry {
                frame_bytes,
                got: frames.len(),
            });
        }
        let bytes = u64::try_from(frames.len()).unwrap_or(u64::MAX);
        let mut encode = self.start_encode(
            job,
            caps,
            destination,
            Some(audio),
            ProcessCancellation::new(),
            ProcessStdinLimits::new(bytes, bytes),
        )?;
        encode.write_stdin(&frames)?;
        encode.prepare()?.commit()
    }

    /// Decode any ffmpeg-readable audio source to native s16 PCM WAV.
    ///
    /// The source is resolved before creating the job-scoped working directory
    /// so relative caller paths retain their meaning without reaching the
    /// cwd-free child. The decoded WAV is born in that directory and reaches
    /// `destination` only through the same bounded, atomic publication path as
    /// encoded video. Certified runs consume the resulting PCM through the
    /// native decoder; the ffmpeg product itself is outside certification by
    /// construction.
    ///
    /// # Errors
    /// Every refusal in [`BoundaryError`].
    pub fn transcode_audio(
        &self,
        input: &Path,
        destination: &Path,
    ) -> Result<BoundaryReport, BoundaryError> {
        let input = std::fs::canonicalize(input).map_err(|error| BoundaryError::Workdir {
            detail: format!("resolve audio input {}: {error}", input.display()),
        })?;
        let workdir = self.make_workdir()?;
        let mut bound_tool = match self.tool.bind_into(&workdir) {
            Ok(bound_tool) => bound_tool,
            Err(error) => {
                self.cleanup(&workdir);
                return Err(error);
            }
        };
        let artifact = workdir.join("decoded.wav");
        let argv = transcode_audio_argv(&input, &artifact);
        let result = self.run_publishing(
            &mut bound_tool,
            argv,
            &workdir,
            &artifact,
            destination,
            None,
        );
        self.cleanup(&workdir);
        result
    }

    /// Concatenate already-encoded partial files with stream copy (the
    /// insert-file mechanism).
    ///
    /// # Errors
    /// Every refusal in [`BoundaryError`]; single quotes in input paths
    /// are refused rather than escaped.
    pub fn concat(
        &self,
        inputs: &[PathBuf],
        destination: &Path,
    ) -> Result<BoundaryReport, BoundaryError> {
        if inputs.is_empty() {
            return Err(BoundaryError::Negotiation(NegotiationError(
                "concat of zero inputs",
            )));
        }
        let mut listing = String::new();
        for input in inputs {
            let input = std::fs::canonicalize(input).map_err(|error| BoundaryError::Workdir {
                detail: format!("resolve concat input {}: {error}", input.display()),
            })?;
            let text = input
                .to_str()
                .ok_or(BoundaryError::Negotiation(NegotiationError(
                    "concat input path is not valid UTF-8",
                )))?;
            if text.contains('\'') || text.contains('\n') || text.contains('\r') {
                return Err(BoundaryError::Negotiation(NegotiationError(
                    "concat input path contains a quote or line break",
                )));
            }
            listing.push_str(&format!("file '{text}'\n"));
        }
        let workdir = self.make_workdir()?;
        let result = (|| {
            let mut bound_tool = self.tool.bind_into(&workdir)?;
            let list_file = workdir.join("concat.txt");
            std::fs::write(&list_file, listing).map_err(|e| BoundaryError::Workdir {
                detail: format!("write {}: {e}", list_file.display()),
            })?;
            let ext = destination
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4");
            let artifact = workdir.join(format!("joined.{ext}"));
            let argv = crate::negotiate::concat_argv(&list_file, &artifact);
            self.run_publishing(
                &mut bound_tool,
                argv,
                &workdir,
                &artifact,
                destination,
                None,
            )
        })();
        self.cleanup(&workdir);
        result
    }
}

/// A live, job-scoped ffmpeg encode with backpressured stdin.
pub struct StreamingEncode {
    tool: FfmpegTool,
    runner: Arc<dyn ProcessRunner>,
    limits: JobLimits,
    workdir: OwnedWorkdir,
    bound_tool: BoundFfmpeg,
    destination: PathBuf,
    video_artifact: PathBuf,
    audio: Option<PathBuf>,
    encoder: Option<String>,
    video_argv: Vec<String>,
    process: Option<Box<dyn RunningProcess>>,
    cancellation: ProcessCancellation,
    cleanup: bool,
}

impl StreamingEncode {
    /// Attach a finalized native WAV after frame production, before the
    /// existing encode/mux preparation. The bytes stay in this job's private
    /// directory and never become a separately published artifact.
    ///
    /// # Errors
    /// Refuses an existing soundtrack, oversized input, or a changed private
    /// directory. The normal preparation path still validates every process
    /// and artifact before publication.
    pub fn prepare_with_wav(mut self, wav: &[u8]) -> Result<PreparedFfmpegArtifact, BoundaryError> {
        if self.audio.is_some() {
            return Err(BoundaryError::Workdir {
                detail: "streaming ffmpeg already has an audio input".to_owned(),
            });
        }
        if wav.len() as u64 > self.limits.max_artifact_bytes {
            return Err(BoundaryError::Workdir {
                detail: "native WAV exceeds the ffmpeg artifact budget".to_owned(),
            });
        }
        self.workdir.verify_current("stage finalized native WAV")?;
        let audio = self.workdir.join("soundtrack.wav");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let write = || -> std::io::Result<()> {
            let mut file = options.open(&audio)?;
            file.write_all(wav)?;
            file.sync_all()
        };
        write().map_err(|error| BoundaryError::Workdir {
            detail: format!("stage finalized native WAV: {error}"),
        })?;
        self.workdir.verify_current("finish native WAV staging")?;
        self.audio = Some(audio);
        self.prepare()
    }

    /// Feed the next tightly packed ordered byte chunk.
    ///
    /// The OS pipe supplies bounded backpressure; the process supervisor can
    /// cancel and close its read end while this call is blocked.
    ///
    /// # Errors
    /// [`BoundaryError`] when process stdin refuses the chunk.
    pub fn write_stdin(&mut self, bytes: &[u8]) -> Result<(), BoundaryError> {
        let write = self
            .process
            .as_mut()
            .ok_or_else(|| BoundaryError::Workdir {
                detail: "streaming ffmpeg stdin is already closed".to_string(),
            })?
            .write_stdin(bytes);
        let Err(write_error) = write else {
            return Ok(());
        };

        // A killed/exited child commonly surfaces first as BrokenPipe on the
        // bounded writer. Reap it here and prefer the supervisor's typed
        // terminal reason (timeout/log cap/cancellation/nonzero exit) over the
        // incidental pipe diagnostic. Input-limit errors retain their precise
        // mechanism category if the child itself had not failed.
        let process = self.process.take().ok_or_else(|| BoundaryError::Workdir {
            detail: "streaming ffmpeg process disappeared after stdin failure".to_string(),
        })?;
        if matches!(
            &write_error,
            ProcessError::StdinChunkLimit { .. } | ProcessError::StdinTotalLimit { .. }
        ) {
            let _ = process.cancel();
            return Err(BoundaryError::Mechanism(write_error));
        }
        match process.finish() {
            Ok(outcome) => match check_process_outcome(&self.limits, &outcome) {
                Ok(()) => Err(BoundaryError::Mechanism(write_error)),
                Err(terminal) => Err(terminal),
            },
            Err(_) => Err(BoundaryError::Mechanism(write_error)),
        }
    }

    /// Finish every job-scoped invocation and verify the final artifact without
    /// publishing it.
    ///
    /// # Errors
    /// Process, timeout, log, codec, artifact, or cancellation refusals.
    pub fn prepare(mut self) -> Result<PreparedFfmpegArtifact, BoundaryError> {
        let process = self.process.take().ok_or_else(|| BoundaryError::Workdir {
            detail: "streaming ffmpeg process was already finalized".to_string(),
        })?;
        let outcome = process.finish();
        self.workdir.verify_current("finalize video encode")?;
        self.bound_tool.verify_current(&self.tool)?;
        let outcome = outcome?;
        check_process_outcome(&self.limits, &outcome)?;
        self.workdir.verify_current("verify video artifact")?;
        verify_private_artifact(&self.video_artifact, self.limits.max_artifact_bytes)?;
        let mut invocations = vec![invocation_report(
            &self.tool,
            self.bound_tool.path(),
            self.runner.mechanism(),
            self.encoder.clone(),
            self.video_argv.clone(),
            self.video_artifact.clone(),
            outcome.stderr,
        )];

        let artifact = if let Some(audio) = &self.audio {
            let muxed = self.workdir.join(format!(
                "muxed.{}",
                extension_or_default(&self.video_artifact)
            ));
            let argv = mux_argv(&self.video_artifact, audio, &muxed);
            self.workdir.verify_current("start audio mux")?;
            self.bound_tool.verify_current(&self.tool)?;
            let spec = ProcessSpec {
                program: self.bound_tool.path().to_path_buf(),
                argv: argv.clone(),
                env: Boundary::env_allowlist(&self.workdir),
                cwd: None,
                stdin: None,
                timeout: self.limits.timeout,
                max_output_bytes: self.limits.max_log_bytes,
            };
            let process = self.runner.start(
                &spec,
                self.cancellation.clone(),
                ProcessStdinLimits::new(1, 0),
            )?;
            let post_start = self
                .workdir
                .verify_current("finish audio-mux spawn")
                .and_then(|()| self.bound_tool.verify_current(&self.tool));
            if let Err(error) = post_start {
                let _ = process.cancel();
                return Err(error);
            }
            let outcome = process.finish();
            self.workdir.verify_current("finalize audio mux")?;
            self.bound_tool.verify_current(&self.tool)?;
            let outcome = outcome?;
            check_process_outcome(&self.limits, &outcome)?;
            self.workdir.verify_current("verify muxed artifact")?;
            verify_private_artifact(&muxed, self.limits.max_artifact_bytes)?;
            invocations.push(invocation_report(
                &self.tool,
                self.bound_tool.path(),
                self.runner.mechanism(),
                None,
                argv,
                muxed.clone(),
                outcome.stderr,
            ));
            muxed
        } else {
            self.video_artifact.clone()
        };

        self.workdir.verify_current("prepare ffmpeg artifact")?;
        self.cleanup = false;
        Ok(PreparedFfmpegArtifact {
            limits: self.limits.clone(),
            workdir: self.workdir.clone(),
            artifact,
            destination: self.destination.clone(),
            invocations,
            cleanup: true,
        })
    }

    /// Cancel and reap a live encode.
    ///
    /// # Errors
    /// [`BoundaryError`] when supervision fails while reaping.
    pub fn cancel(mut self) -> Result<(), BoundaryError> {
        self.cancellation.cancel();
        if let Some(process) = self.process.take() {
            process.cancel()?;
        }
        Ok(())
    }
}

impl Drop for StreamingEncode {
    fn drop(&mut self) {
        if self.cleanup {
            self.cancellation.cancel();
            if let Some(process) = self.process.take() {
                let _ = process.cancel();
            }
            cleanup_workdir(&self.limits, &self.workdir);
        }
    }
}

/// A verified unpublished ffmpeg artifact. Dropping it never publishes.
pub struct PreparedFfmpegArtifact {
    limits: JobLimits,
    workdir: OwnedWorkdir,
    artifact: PathBuf,
    destination: PathBuf,
    invocations: Vec<InvocationReport>,
    cleanup: bool,
}

impl PreparedFfmpegArtifact {
    /// Atomically publish the verified artifact.
    ///
    /// # Errors
    /// Artifact revalidation or rename failure.
    pub fn commit(self) -> Result<BoundaryReport, BoundaryError> {
        self.commit_with_policy(false)
    }

    /// Publish verified video without replacing any destination node.
    ///
    /// Stream through a private destination sibling so the encode workdir may
    /// be on another filesystem. Resident copy memory is bounded independently
    /// of movie length. Rehash the copied bytes before atomic link publication.
    ///
    /// # Errors
    /// Artifact revalidation, staging, or atomic create-only publication fails.
    pub fn commit_new(self) -> Result<BoundaryReport, BoundaryError> {
        self.commit_with_policy(true)
    }

    fn commit_with_policy(mut self, no_clobber: bool) -> Result<BoundaryReport, BoundaryError> {
        self.workdir.verify_current("commit ffmpeg artifact")?;
        let (artifact_bytes, artifact_digest) =
            hash_private_artifact(&self.artifact, self.limits.max_artifact_bytes)?;
        if let Some(parent) = self.destination.parent() {
            std::fs::create_dir_all(parent).map_err(|error| BoundaryError::Workdir {
                detail: format!("create {}: {error}", parent.display()),
            })?;
        }
        self.workdir
            .verify_current("publish prepared ffmpeg artifact")?;
        verify_private_artifact(&self.artifact, self.limits.max_artifact_bytes)?;
        if no_clobber {
            publish_artifact_new(
                &self.artifact,
                &self.destination,
                artifact_bytes,
                artifact_digest,
                self.limits.max_artifact_bytes,
            )?;
        } else {
            std::fs::rename(&self.artifact, &self.destination).map_err(|error| {
                BoundaryError::Workdir {
                    detail: format!(
                        "publish {} -> {}: {error}",
                        self.artifact.display(),
                        self.destination.display()
                    ),
                }
            })?;
        }
        self.cleanup = false;
        cleanup_workdir(&self.limits, &self.workdir);
        Ok(BoundaryReport {
            invocations: std::mem::take(&mut self.invocations),
            destination: self.destination.clone(),
            artifact_bytes,
            artifact_digest,
        })
    }
}

fn publish_artifact_new(
    artifact: &Path,
    destination: &Path,
    expected_bytes: u64,
    expected_digest: Digest,
    limit: u64,
) -> Result<(), BoundaryError> {
    use fmn_platform::fs::{FileSystem as _, StdFs};

    let failure = |error: &dyn std::fmt::Display| BoundaryError::Workdir {
        detail: format!("create-only publication {}: {error}", destination.display()),
    };
    let mut source = File::open(artifact).map_err(|error| failure(&error))?;
    let mut writer = Arc::new(StdFs)
        .begin_atomic_file(destination)
        .map_err(|error| failure(&error))?;
    let mut bytes = 0_u64;
    let mut digest = Sha256::new();
    let mut chunk = [0_u8; 64 * 1024];
    loop {
        let read = match source.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(failure(&error)),
        };
        bytes = bytes.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
        if bytes > limit {
            return Err(BoundaryError::ArtifactOversized { bytes, max: limit });
        }
        digest.update(&chunk[..read]);
        writer
            .write(&chunk[..read])
            .map_err(|error| failure(&error))?;
    }
    if bytes != expected_bytes || digest.finalize() != expected_digest {
        return Err(failure(&"prepared artifact changed during publication"));
    }
    writer
        .prepare()
        .map_err(|error| failure(&error))?
        .commit_new()
        .map_err(|error| failure(&error))
}

impl Drop for PreparedFfmpegArtifact {
    fn drop(&mut self) {
        if self.cleanup {
            cleanup_workdir(&self.limits, &self.workdir);
        }
    }
}

fn extension_or_default(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("mp4")
        .to_string()
}

fn verify_private_artifact(path: &Path, max_bytes: u64) -> Result<(), BoundaryError> {
    let metadata = std::fs::metadata(path).map_err(|_| BoundaryError::ArtifactMissing)?;
    if !metadata.is_file() {
        return Err(BoundaryError::ArtifactMissing);
    }
    if metadata.len() > max_bytes {
        return Err(BoundaryError::ArtifactOversized {
            bytes: metadata.len(),
            max: max_bytes,
        });
    }
    Ok(())
}

fn hash_private_artifact(path: &Path, max_bytes: u64) -> Result<(u64, Digest), BoundaryError> {
    verify_private_artifact(path, max_bytes)?;
    let mut file = File::open(path).map_err(|error| BoundaryError::Workdir {
        detail: format!(
            "open verified artifact {} for hashing: {error}",
            path.display()
        ),
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; FILE_HASH_BUFFER_BYTES];
    let mut bytes = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| BoundaryError::Workdir {
                detail: format!(
                    "read verified artifact {} for hashing: {error}",
                    path.display()
                ),
            })?;
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(read as u64)
            .ok_or(BoundaryError::ArtifactOversized {
                bytes: u64::MAX,
                max: max_bytes,
            })?;
        if bytes > max_bytes {
            return Err(BoundaryError::ArtifactOversized {
                bytes,
                max: max_bytes,
            });
        }
        hasher.update(&buffer[..read]);
    }
    Ok((bytes, hasher.finalize()))
}

fn invocation_report(
    tool: &FfmpegTool,
    bound_tool: &Path,
    mechanism: ProcessMechanism,
    encoder: Option<String>,
    argv: Vec<String>,
    artifact: PathBuf,
    stderr: Vec<u8>,
) -> InvocationReport {
    InvocationReport {
        provenance: Provenance {
            tool_path: tool.path().to_path_buf(),
            tool_sha256_hex: tool.sha256_hex().to_string(),
            native_image: tool.native_image(),
            tool_version: tool.version().to_string(),
            bound_tool_path: bound_tool.to_path_buf(),
            process_mechanism: mechanism.identity().to_string(),
            process_policy_version: mechanism.policy_version(),
            encoder,
            argv,
        },
        stderr,
        artifact,
    }
}

fn check_process_outcome(
    limits: &JobLimits,
    outcome: &ProcessOutcome,
) -> Result<(), BoundaryError> {
    match outcome.termination {
        ProcessTermination::Exited(Some(0)) => Ok(()),
        ProcessTermination::TimedOut => Err(BoundaryError::JobTimedOut {
            timeout: limits.timeout,
        }),
        ProcessTermination::OutputLimitExceeded => Err(BoundaryError::LogOverflow),
        ProcessTermination::Cancelled => Err(BoundaryError::Cancelled),
        ProcessTermination::Exited(code) => {
            let stderr = String::from_utf8_lossy(&outcome.stderr);
            let tail: String = stderr
                .chars()
                .rev()
                .take(2048)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            Err(BoundaryError::EncodeFailed {
                code,
                stderr: tail.trim().to_string(),
            })
        }
    }
}

fn cleanup_workdir(limits: &JobLimits, workdir: &OwnedWorkdir) {
    if !limits.keep_workdir {
        workdir.cleanup_recursive();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use fmn_platform::process::{FfmpegLocator as _, ScriptedRunner, StdFfmpegLocator};

    #[test]
    fn create_only_video_copy_checks_content_and_budget_before_publication() {
        let root = std::env::temp_dir().join(format!(
            "fmn-video-copy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let source = root.join("private-video");
        let destination = root.join("output.mp4");
        let bytes = vec![37_u8; 150_000];
        let digest = fmn_hash::sha256(&bytes);
        std::fs::write(&source, &bytes).unwrap();
        for (length, expected, limit) in [
            (149_999, digest, 150_000),
            (150_000, fmn_hash::sha256(b"wrong bytes"), 150_000),
            (150_000, digest, 149_999),
        ] {
            assert!(publish_artifact_new(&source, &destination, length, expected, limit).is_err());
            assert!(!destination.exists());
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        }
        publish_artifact_new(&source, &destination, 150_000, digest, 150_000).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
        std::fs::write(&source, b"changed private bytes").unwrap();
        assert_eq!(
            std::fs::read(&destination).unwrap(),
            bytes,
            "published video must not alias a retained debug artifact"
        );
        assert!(
            publish_artifact_new(
                &source,
                &destination,
                21,
                fmn_hash::sha256(b"changed private bytes"),
                150_000
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn private_copy_path_substitution_is_rejected_before_the_runner() {
        let source_path = std::env::current_exe().expect("current native test executable");
        let executable = StdFfmpegLocator::default()
            .locate_ffmpeg(&source_path)
            .expect("issue an executable token for the native test harness");
        let canonical_path = executable.canonical_path().to_path_buf();
        let (mut source, native_image) = executable
            .open_current()
            .expect("reopen the native test harness");
        let sha256_hex = sha256_open_file(&mut source, &canonical_path, native_image.file_bytes)
            .expect("hash the native test harness");
        // RCH and other build harnesses may point TMPDIR into a transferred
        // checkout whose shared ancestor is intentionally group-writable. The
        // production boundary must reject that tree, so exercise the private
        // copy invariant under Unix's sticky temporary root or Windows temp dir instead.
        #[cfg(unix)]
        let temp_root = PathBuf::from("/tmp");
        #[cfg(windows)]
        let temp_root = std::env::temp_dir();
        let session = ToolSession::create(&temp_root).expect("claim a private test session");
        let tool = FfmpegTool {
            executable,
            path: canonical_path,
            sha256_hex,
            native_image,
            version: String::new(),
            session,
        };
        let runner = ScriptedRunner::new();

        let error = tool
            .run_bound_probe_after_copy(&runner, &["-version"], |_private, bound| {
                let displaced = bound.with_extension("displaced");
                std::fs::rename(bound, &displaced).map_err(|error| BoundaryError::Workdir {
                    detail: format!(
                        "displace exact private-copy pathname {}: {error}",
                        bound.display()
                    ),
                })?;
                std::fs::write(bound, b"#!/bin/sh\nexit 0\n").map_err(|error| {
                    BoundaryError::Workdir {
                        detail: format!(
                            "install malformed private-copy replacement {}: {error}",
                            bound.display()
                        ),
                    }
                })?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt as _;
                    std::fs::set_permissions(bound, std::fs::Permissions::from_mode(0o500))
                        .map_err(|error| BoundaryError::Workdir {
                            detail: format!(
                                "permission private-copy replacement {}: {error}",
                                bound.display()
                            ),
                        })?;
                }
                Ok(())
            })
            .expect_err("the reopened private pathname must be re-attested before the runner");

        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
        assert!(
            runner.runs().is_empty(),
            "private-copy attestation must fail before ProcessRunner::run"
        );
    }
}
