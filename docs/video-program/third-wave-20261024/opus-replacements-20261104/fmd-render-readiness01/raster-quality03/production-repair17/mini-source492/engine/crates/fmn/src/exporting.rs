//! Export ordinary native scenes for the standalone CLI and code-free players.
//!
//! The real Scene lifecycle records its immutable captures. Playback consumes
//! FMTL/1 snapshots, not Rust callbacks. Unlike the declarative Timeline export,
//! this route does not guess a compact interpolation law for imperative code.
//! Same scene inputs yield the same bundle bytes; this is not an attestation of
//! arbitrary callback inputs or a complete certified input-closure manifest.
//!
//! The original exports preserve FMTL/1 minor 0: geometry and the frame grid,
//! with replay policy supplied separately. [`export_camera_bundle_bytes`] and
//! its publication siblings record minor 1 with exact per-frame cameras, light
//! and background. Neither format serializes callbacks or audio. Sound requests
//! are refused rather than silently stripped.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn_config::Config;
use fmn_platform::fs::{FileSystem, FsError, StdFs};
use fmn_scene::recording::{RecordedSceneBundle, RecordingError, SceneBundleRecorder};
use fmn_scene::{BundleExportLimits, RuntimeConfig, SceneRunReport};

use crate::SceneConstruct;

mod camera;
pub use camera::{export_camera_bundle, export_camera_bundle_bytes, export_camera_bundle_with_fs};

/// Explicit semantic settings and bounded storage for one native scene export.
#[derive(Clone, Debug)]
pub struct BundleExportOptions {
    /// Scene timing, deterministic seed and the native typesetting template
    /// affect construction. Renderer/output policy is supplied on playback.
    pub config: Config,
    /// Cumulative snapshot/table budget and maximum admitted output frames.
    pub limits: BundleExportLimits,
    /// Maximum complete canonical artifact size, enforced during encoding.
    pub max_output_bytes: usize,
}

impl BundleExportOptions {
    /// Resolve bundled defaults without reading ambient configuration files.
    ///
    /// # Errors
    /// Returns the existing configuration error if bundled defaults are invalid.
    pub fn new() -> Result<Self, fmn_config::ConfigError> {
        Ok(Self {
            config: Config::resolve(&[], None)?.config,
            limits: BundleExportLimits::default(),
            max_output_bytes: fmn_scene::DEFAULT_MAX_BUNDLE_BYTES,
        })
    }

    fn validate(&self) -> Result<(), BundleExportError> {
        if self.config.camera.fps == 0 {
            return Err(BundleExportError::InvalidOptions(
                "bundle fps must be nonzero",
            ));
        }
        if self.limits.max_frames == 0
            || self.limits.max_capture_bytes == 0
            || self.max_output_bytes == 0
        {
            return Err(BundleExportError::InvalidOptions(
                "bundle export budgets must be nonzero",
            ));
        }
        Ok(())
    }
}

/// A completed scene and its validated in-memory artifact.
#[derive(Debug)]
pub struct SceneBundleExport {
    /// Actual native lifecycle result, before any explicit terminal still.
    pub scene: SceneRunReport,
    /// Canonical, content-addressed FMTL/1 output.
    pub bundle: RecordedSceneBundle,
}

/// Receipt for successfully published, no-clobber FMTL output.
#[derive(Clone, Debug)]
pub struct BundleExportReport {
    /// Published destination. No existing node is ever replaced.
    pub output: PathBuf,
    /// Lowercase hexadecimal SHA-256 of the exact file bytes.
    pub digest: String,
    /// File size in bytes.
    pub bytes: usize,
    /// Number of recorded output frames.
    pub frame_count: u32,
    /// Number of recorded play/wait segments and still holds.
    pub segment_count: usize,
    /// Actual scene time and play count (stills do not advance that clock).
    pub scene: SceneRunReport,
}

/// Typed, fail-closed export failures; underlying errors are retained.
#[derive(Debug)]
pub enum BundleExportError {
    /// An invalid option was refused before scene execution.
    InvalidOptions(&'static str),
    /// The format cannot carry a requested side channel.
    Capability(&'static str),
    /// A native construction/lifecycle failure, in its original variant.
    Scene(crate::Error),
    /// Snapshot capture, allocation, clock or format validation failed.
    Recording(RecordingError),
    /// Encoding would exceed the effective canonical output limit.
    OutputLimit { needed: usize, limit: usize },
    /// Private staging, durable preparation or create-only publication failed.
    FileSystem(FsError),
}

impl fmt::Display for BundleExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions(message) | Self::Capability(message) => f.write_str(message),
            Self::Scene(error) => error.fmt(f),
            Self::Recording(error) => error.fmt(f),
            Self::OutputLimit { needed, limit } => {
                write!(
                    f,
                    "scene bundle needs {needed} bytes, exceeding the {limit}-byte output budget"
                )
            }
            Self::FileSystem(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for BundleExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Scene(error) => Some(error),
            Self::Recording(error) => Some(error),
            Self::FileSystem(error) => Some(error),
            _ => None,
        }
    }
}

/// Run any native SceneConstruct once and export its actual captured frames.
///
/// Stateful updaters and noncatalog rate functions are supported: every observed
/// frame is recorded, without executing callbacks during replay. Static scenes
/// produce one terminal still, matching the ordinary native render API. Early
/// scene termination follows the runtime's normal successful EndScene contract.
///
/// Capture failure is sticky even when user code catches a play/wait error.
/// Limits bound capture work/storage, not execution of arbitrary user code or
/// the deterministic completion of an animation already in flight.
///
/// # Errors
/// Refuses invalid budgets, native errors, audio, oversized output, and malformed
/// captures. No filesystem operation occurs, including when scene code panics.
pub fn export_bundle_bytes<P: SceneConstruct + ?Sized>(
    program: &mut P,
    options: BundleExportOptions,
) -> Result<SceneBundleExport, BundleExportError> {
    options.validate()?;
    let runtime = RuntimeConfig::from_config(&options.config);
    let mut recorder = SceneBundleRecorder::new(runtime.effective_fps(), options.limits)
        .map_err(BundleExportError::Recording)?;
    let typesetting = fmn_tex::TexSession::memory(options.config.tex.template.clone());
    let run = crate::run_scene_with_typesetting(
        program,
        runtime,
        options.config.determinism.seed,
        &mut recorder,
        &typesetting,
        crate::typesetting::DEFAULT_PREFLIGHT_WORKERS,
    );
    let completed = match run {
        Ok(completed) => completed,
        Err(error) => {
            return Err(match recorder.into_error() {
                Some(error) => BundleExportError::Recording(error),
                None => BundleExportError::Scene(error),
            });
        }
    };
    if !completed.scene().sound_requests().is_empty() {
        return Err(BundleExportError::Capability(
            "FMTL/1 cannot carry audio; use the audio-capable render/output path instead",
        ));
    }
    if recorder.frame_count() == 0 {
        // A terminal still has no lifecycle event and does not rerun updaters.
        // The typed failure is retained by the recorder and returned by finish.
        let _ = recorder.capture_terminal_still(completed.scene().stage());
    }
    let bundle = recorder
        .finish_with_max_bytes(options.max_output_bytes)
        .map_err(|error| match error {
            RecordingError::OutputLimit { needed, limit } => {
                BundleExportError::OutputLimit { needed, limit }
            }
            error => BundleExportError::Recording(error),
        })?;
    Ok(SceneBundleExport {
        scene: *completed.report(),
        bundle,
    })
}

/// Export a native scene as a new `.fmtl` file using the host filesystem.
///
/// All scene work, recording and production-reader validation finish before a
/// private file is staged. Publication uses the filesystem's atomic create-only
/// operation, not an existence check followed by replacement. Existing output
/// survives errors, races and unwinding panics unchanged.
///
/// # Errors
/// Returns recording errors from [`export_bundle_bytes`] or typed filesystem
/// failures. The filesystem must support create-only atomic publication.
pub fn export_bundle<P: SceneConstruct + ?Sized>(
    program: &mut P,
    output: impl AsRef<Path>,
    options: BundleExportOptions,
) -> Result<BundleExportReport, BundleExportError> {
    export_bundle_with_fs(program, output, options, Arc::new(StdFs))
}

/// [`export_bundle`] using an explicit filesystem capability (including WASM
/// hosts and deterministic VirtualFs tests).
///
/// # Errors
/// Returns the same failures as [`export_bundle`].
pub fn export_bundle_with_fs<P: SceneConstruct + ?Sized>(
    program: &mut P,
    output: impl AsRef<Path>,
    options: BundleExportOptions,
    fs: Arc<dyn FileSystem>,
) -> Result<BundleExportReport, BundleExportError> {
    let output = output.as_ref();
    if output.as_os_str().is_empty() || output.file_name().is_none() {
        return Err(BundleExportError::InvalidOptions(
            "bundle output must name a file",
        ));
    }
    let export = export_bundle_bytes(program, options)?;
    publish(export, output, fs)
}

fn publish(
    export: SceneBundleExport,
    output: &Path,
    fs: Arc<dyn FileSystem>,
) -> Result<BundleExportReport, BundleExportError> {
    let report = BundleExportReport {
        output: output.to_owned(),
        digest: export.bundle.digest.to_hex(),
        bytes: export.bundle.bytes.len(),
        frame_count: export.bundle.frame_count,
        segment_count: export.bundle.segment_count,
        scene: export.scene,
    };
    let mut writer = fs
        .begin_atomic_file(output)
        .map_err(BundleExportError::FileSystem)?;
    writer
        .write(&export.bundle.bytes)
        .map_err(BundleExportError::FileSystem)?;
    writer
        .prepare()
        .map_err(BundleExportError::FileSystem)?
        .commit_new()
        .map_err(BundleExportError::FileSystem)?;
    Ok(report)
}
