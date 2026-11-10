//! The store: versioned namespaces of content-addressed entries over the
//! filesystem capability, with advisory maintenance locking, an LRU-class
//! index, pinning, and defined eviction.
//!
//! # On-disk shape
//!
//! ```text
//! <root>/
//!   STORE_OWNER                 path + generation ownership manifest
//!   STORE_FORMAT                  the store-format stamp ("fmn-cache 1")
//!   ns/<name>/v<version>/         one versioned namespace
//!     objects/<hh>/<hex…>         entries, sharded by the first digest byte
//!     index                       the advisory LRU index (rebuildable)
//!     lock                        the advisory maintenance lock (transient)
//! ```
//!
//! Every path component below `<root>` is either a fixed literal, a validated
//! namespace name (`[a-z0-9][a-z0-9_-]*`, at most 64 bytes), a `v<u32>`
//! version directory, or digest hex — arbitrary key bytes never reach a path,
//! so no key can escape the root. Every actual traversal also classifies its
//! components without following the leaf, rejects link-like and wrong-kind
//! nodes, and creates missing directories one exact leaf at a time (the
//! traversal-protection contract, fuzzed and exercised against host symlink
//! sentinels in the crate's tests).
//!
//! # Concurrency model
//!
//! Entry writes are atomic, immutable, and digest-addressed: canonical keys
//! address keyed entries, while payload digests address blobs. The first
//! create-if-absent publication for an address wins, and every losing writer
//! verifies the complete incumbent. Identical payloads are idempotent;
//! different keyed payloads are a typed producer conflict and never replace
//! one another, so put/get take no lock. The LRU index is advisory
//! (last-writer-wins, merged on flush, reconciled against disk truth by
//! eviction; a lost or stale index degrades LRU accuracy, never correctness).
//! Only maintenance — eviction — takes the per-namespace lock file, and a
//! crashed holder is broken by wall-clock staleness; a maintainer that cannot
//! get the lock skips, it never blocks.

use crate::entry::{self, EntryKind};
use crate::key::CacheKey;
use crate::{CacheError, KeyConflict, RootRefusalCode};
use fmn_hash::{Digest, Limits, Reader, Schema, UnknownPolicy, Writer, sha256};
use fmn_platform::clock::Clock;
use fmn_platform::fs::{FileSystem, FsError, FsNodeKind};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

/// The exact store-format stamp this build reads and writes. A different
/// stamp on disk is [`CacheError::FormatUnsupported`]; the remedy is
/// `--clear-cache`.
const FORMAT_STAMP: &str = "fmn-cache 1";
/// The stamp's file name under the store root.
const FORMAT_FILE: &str = "STORE_FORMAT";
/// Stable ownership-manifest prefix, independent of the replaceable store
/// format. The two fields bind the canonical absolute host path and one live
/// store generation.
const OWNER_PREFIX: &str = "franken-manim cache root 2 ";
/// The ownership manifest's file name under the store root.
const OWNER_FILE: &str = "STORE_OWNER";
/// Owner manifests and format stamps are fixed, tiny lifecycle documents.
/// Host reads stop after one byte beyond these limits.
const OWNER_MANIFEST_MAX_BYTES: usize = 256;
const FORMAT_STAMP_MAX_BYTES: usize = 64;
/// Dedicated application leaf below the host's per-user cache directory.
///
/// This deliberately does not reuse the Reference's `manim` leaf: a Python
/// Manim cache is foreign data and must never become claimable by
/// FrankenManim's owned-store protocol.
pub const DEFAULT_CACHE_LEAF: &str = "franken-manim";

/// The advisory LRU index document.
const INDEX_SCHEMA: Schema = Schema::new(*b"FMNC", 2, 1, 0);
/// The advisory maintenance-lock token document.
const LOCK_SCHEMA: Schema = Schema::new(*b"FMNC", 4, 1, 0);

/// Process-wide store-instance counter, distinguishing lock tokens from two
/// stores (or two openings) in one process.
static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);
/// Process-local input to fresh owner generations. Wall time and process id
/// provide cross-process separation; this counter makes same-process calls
/// distinct even when the clock resolution is coarse.
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);
/// Process-local uniquifier for clear-quarantine directory names.
static NEXT_CLEAR: AtomicU64 = AtomicU64::new(1);
/// Bound collision work so a hostile directory full of guessed names cannot
/// make a lifecycle command spin forever.
const MAX_QUARANTINE_ATTEMPTS: usize = 4_096;

/// Failure to turn the effective cache configuration into one absolute host
/// path.
#[derive(Debug)]
pub enum CacheRootError {
    /// A relative configured path could not be anchored because the host
    /// current directory was unavailable.
    CurrentDirectory {
        /// The host failure.
        err: io::Error,
    },
    /// A configured path escaped through `..` or otherwise could not be made
    /// into a safe absolute store location.
    InvalidConfigured {
        /// The configured path.
        path: PathBuf,
        /// Precise refusal reason.
        reason: &'static str,
    },
    /// The platform's per-user cache base could not be derived without
    /// guessing or falling back to the current/temp directory.
    PlatformDefaultUnavailable {
        /// `std::env::consts::OS` spelling.
        platform: &'static str,
        /// Precise missing or invalid environment contract.
        reason: &'static str,
    },
}

impl fmt::Display for CacheRootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDirectory { err } => {
                write!(
                    f,
                    "could not resolve the current directory for the cache root: {err}"
                )
            }
            Self::InvalidConfigured { path, reason } => {
                write!(
                    f,
                    "invalid configured cache root {:?}: {reason}",
                    path.as_os_str()
                )
            }
            Self::PlatformDefaultUnavailable { platform, reason } => {
                write!(f, "{platform} cache default is unavailable: {reason}")
            }
        }
    }
}

impl std::error::Error for CacheRootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CurrentDirectory { err } => Some(err),
            Self::InvalidConfigured { .. } | Self::PlatformDefaultUnavailable { .. } => None,
        }
    }
}

#[derive(Debug)]
struct CacheRootEnvironment {
    platform: &'static str,
    current_dir: Option<PathBuf>,
    xdg_cache_home: Option<OsString>,
    home: Option<OsString>,
    local_app_data: Option<OsString>,
    user_profile: Option<OsString>,
}

#[derive(Clone, Copy, Debug)]
enum CacheRootOrigin {
    Configured,
    PlatformDefault,
}

impl CacheRootEnvironment {
    fn host(configured: &str) -> Result<Self, CacheRootError> {
        let configured_path = Path::new(configured);
        let current_dir = if configured.is_empty() || configured_path.is_absolute() {
            None
        } else {
            Some(env::current_dir().map_err(|err| CacheRootError::CurrentDirectory { err })?)
        };
        Ok(Self {
            platform: env::consts::OS,
            current_dir,
            xdg_cache_home: env::var_os("XDG_CACHE_HOME"),
            home: env::var_os("HOME"),
            local_app_data: env::var_os("LOCALAPPDATA"),
            user_profile: env::var_os("USERPROFILE"),
        })
    }
}

/// Resolve the effective cache setting to one absolute, byte-preserving host
/// path shared by [`Store::open_host`], doctor inspection, and
/// `--clear-cache`.
///
/// A non-empty configured value wins and is anchored to the current
/// directory when relative. An empty value selects the platform convention:
/// `XDG_CACHE_HOME` (or `HOME/.cache`) on Unix, `HOME/Library/Caches` on
/// macOS, and `LOCALAPPDATA` (or `USERPROFILE/AppData/Local`) on Windows, then
/// appends [`DEFAULT_CACHE_LEAF`]. Environment paths remain native
/// [`OsString`] bytes; this function never performs lossy Unicode conversion.
///
/// # Errors
///
/// [`CacheRootError`] if a configured value contains `..`, a relative value
/// cannot be anchored, or no trustworthy absolute platform base is
/// available.
pub fn resolve_host_cache_root(configured: &str) -> Result<PathBuf, CacheRootError> {
    let environment = CacheRootEnvironment::host(configured)?;
    resolve_cache_root(configured, &environment)
}

fn resolve_cache_root(
    configured: &str,
    environment: &CacheRootEnvironment,
) -> Result<PathBuf, CacheRootError> {
    if !configured.is_empty() {
        return absolute_configured_root(Path::new(configured), environment);
    }

    let base = match environment.platform {
        "windows" => absolute_environment_path(environment.local_app_data.as_ref())
            .or_else(|| {
                absolute_environment_path(environment.user_profile.as_ref())
                    .map(|profile| profile.join("AppData").join("Local"))
            })
            .ok_or(CacheRootError::PlatformDefaultUnavailable {
                platform: environment.platform,
                reason: "LOCALAPPDATA and an absolute USERPROFILE are unavailable",
            })?,
        "macos" => absolute_environment_path(environment.home.as_ref())
            .map(|home| home.join("Library").join("Caches"))
            .ok_or(CacheRootError::PlatformDefaultUnavailable {
                platform: environment.platform,
                reason: "an absolute HOME is unavailable",
            })?,
        "linux" | "android" | "freebsd" | "dragonfly" | "netbsd" | "openbsd" | "solaris"
        | "illumos" | "aix" | "haiku" => {
            absolute_environment_path(environment.xdg_cache_home.as_ref())
                .or_else(|| {
                    absolute_environment_path(environment.home.as_ref())
                        .map(|home| home.join(".cache"))
                })
                .ok_or(CacheRootError::PlatformDefaultUnavailable {
                    platform: environment.platform,
                    reason: "XDG_CACHE_HOME and an absolute HOME are unavailable",
                })?
        }
        _ => {
            return Err(CacheRootError::PlatformDefaultUnavailable {
                platform: environment.platform,
                reason: "this platform has no declared per-user cache convention",
            });
        }
    };
    finish_resolved_root(
        base.join(DEFAULT_CACHE_LEAF),
        environment.platform,
        CacheRootOrigin::PlatformDefault,
    )
}

fn absolute_configured_root(
    configured: &Path,
    environment: &CacheRootEnvironment,
) -> Result<PathBuf, CacheRootError> {
    if configured.as_os_str().is_empty() {
        return Err(CacheRootError::InvalidConfigured {
            path: configured.to_path_buf(),
            reason: "an explicitly configured cache root may not be empty",
        });
    }
    if configured
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CacheRootError::InvalidConfigured {
            path: configured.to_path_buf(),
            reason: "parent-directory components are not accepted",
        });
    }
    let resolved =
        if configured.is_absolute() {
            configured.to_path_buf()
        } else {
            let current_dir = environment.current_dir.as_ref().ok_or(
                CacheRootError::PlatformDefaultUnavailable {
                    platform: environment.platform,
                    reason: "the current directory was not captured for a relative cache root",
                },
            )?;
            current_dir.join(configured)
        };
    finish_resolved_root(resolved, environment.platform, CacheRootOrigin::Configured)
}

fn absolute_environment_path(value: Option<&OsString>) -> Option<PathBuf> {
    let path = PathBuf::from(value?);
    (!path.as_os_str().is_empty() && path.is_absolute()).then_some(path)
}

fn finish_resolved_root(
    root: PathBuf,
    platform: &'static str,
    origin: CacheRootOrigin,
) -> Result<PathBuf, CacheRootError> {
    if !root.is_absolute() {
        return Err(match origin {
            CacheRootOrigin::Configured => CacheRootError::InvalidConfigured {
                path: root,
                reason: "the resolved cache root is not absolute",
            },
            CacheRootOrigin::PlatformDefault => CacheRootError::PlatformDefaultUnavailable {
                platform,
                reason: "the resolved platform cache root is not absolute",
            },
        });
    }
    if root
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(match origin {
            CacheRootOrigin::Configured => CacheRootError::InvalidConfigured {
                path: root,
                reason: "the resolved cache root contains a parent-directory component",
            },
            CacheRootOrigin::PlatformDefault => CacheRootError::PlatformDefaultUnavailable {
                platform,
                reason: "the platform cache base contains a parent-directory component",
            },
        });
    }
    Ok(root)
}

fn lock_poisoned<T>(err: PoisonError<T>) -> T {
    err.into_inner()
}

fn root_refused(root: &Path, code: RootRefusalCode, reason: impl Into<String>) -> CacheError {
    CacheError::RootRefused {
        root: root.to_path_buf(),
        code,
        reason: reason.into(),
    }
}

fn managed_node_refused(path: &Path, expected: &str, found: FsNodeKind) -> CacheError {
    CacheError::Storage(FsError::Io {
        path: path.to_path_buf(),
        err: io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "cache traversal refused {found:?}; expected {expected} without following links"
            ),
        ),
    })
}

fn managed_path_refused(path: &Path, reason: impl Into<String>) -> CacheError {
    CacheError::Storage(FsError::Io {
        path: path.to_path_buf(),
        err: io::Error::new(io::ErrorKind::PermissionDenied, reason.into()),
    })
}

fn managed_not_found(path: &Path) -> CacheError {
    CacheError::Storage(FsError::NotFound {
        path: path.to_path_buf(),
    })
}

#[derive(Clone, Copy)]
enum DirectoryMode {
    Existing,
    Create,
    ExistingPrefix,
}

/// Validate a directory path at or below `root` one component at a time.
///
/// The root itself is already canonical and owned when this is used by a
/// live store. `Create` claims each missing component with exact-leaf
/// creation; it never delegates a missing chain to `create_dir_all`.
fn ensure_managed_directory(
    fs: &dyn FileSystem,
    root: &Path,
    directory: &Path,
    mode: DirectoryMode,
) -> Result<(), CacheError> {
    if !root.is_absolute() || !directory.starts_with(root) {
        return Err(managed_path_refused(
            directory,
            format!(
                "managed cache path is not below owned root {}",
                root.display()
            ),
        ));
    }
    if directory
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(managed_path_refused(
            directory,
            "managed cache paths may not contain parent-directory components",
        ));
    }
    match fs.node_kind_no_follow(root)? {
        Some(FsNodeKind::Directory) => {}
        Some(kind) => return Err(managed_node_refused(root, "owned root directory", kind)),
        None => return Err(managed_not_found(directory)),
    }

    let relative = directory.strip_prefix(root).map_err(|_| {
        managed_path_refused(directory, "managed cache path escaped its owned root")
    })?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            return Err(managed_path_refused(
                directory,
                "managed cache path contains a non-normal component",
            ));
        };
        current.push(component);
        match fs.node_kind_no_follow(&current)? {
            Some(FsNodeKind::Directory) => {}
            Some(kind) => {
                return Err(managed_node_refused(&current, "real directory", kind));
            }
            None => match mode {
                DirectoryMode::Existing => return Err(managed_not_found(directory)),
                DirectoryMode::ExistingPrefix => return Ok(()),
                DirectoryMode::Create => {
                    let _created = fs.create_dir(&current)?;
                    match fs.node_kind_no_follow(&current)? {
                        Some(FsNodeKind::Directory) => {}
                        Some(kind) => {
                            return Err(managed_node_refused(&current, "real directory", kind));
                        }
                        None => return Err(managed_not_found(&current)),
                    }
                }
            },
        }
    }
    Ok(())
}

/// Create `root` and any missing ancestors as real directories.
///
/// With `system_links_trusted` (only the ambient host filesystem can vouch
/// for one), an operating-system link ([`trusted_system_link`]) is accepted
/// as a component, the rule [`reject_symlink_components`] applies above a
/// host store root. `root` must then be a strict ancestor of the store root.
fn ensure_capability_root(
    fs: &dyn FileSystem,
    root: &Path,
    system_links_trusted: bool,
) -> Result<(), CacheError> {
    let mut ancestors: Vec<&Path> = root.ancestors().collect();
    ancestors.reverse();
    for directory in ancestors {
        if directory.as_os_str().is_empty() {
            continue;
        }
        match fs.node_kind_no_follow(directory)? {
            Some(FsNodeKind::Directory) => {}
            Some(FsNodeKind::Link) if system_links_trusted && trusted_system_link(directory) => {}
            Some(kind) => {
                return Err(root_refused(
                    root,
                    RootRefusalCode::WrongNodeKind,
                    format!(
                        "capability root component {} is {kind:?}, not a real directory",
                        directory.display()
                    ),
                ));
            }
            None => {
                let _created = fs.create_dir(directory)?;
                match fs.node_kind_no_follow(directory)? {
                    Some(FsNodeKind::Directory) => {}
                    Some(kind) => {
                        return Err(root_refused(
                            root,
                            RootRefusalCode::WrongNodeKind,
                            format!(
                                "capability root component {} became {kind:?}",
                                directory.display()
                            ),
                        ));
                    }
                    None => {
                        return Err(root_refused(
                            root,
                            RootRefusalCode::IdentityChanged,
                            format!(
                                "capability root component {} remained absent",
                                directory.display()
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Prepare only the platform cache base above a resolved application leaf.
///
/// Platform conventions name directories such as `$HOME/.cache` which may
/// legitimately be absent on a fresh profile. The application-owned leaf is
/// intentionally left absent so [`Store::open`] remains the sole operation
/// that can atomically claim and stamp it. A `$HOME` below an operating-system
/// link (macOS `/var`, fm-macos-var-symlink-gate-aqr1) keeps its cache.
fn ensure_platform_cache_parent(fs: &dyn FileSystem, root: &Path) -> Result<(), CacheError> {
    let parent = root.parent().ok_or_else(|| {
        root_refused(
            root,
            RootRefusalCode::InvalidPath,
            "the resolved platform cache root must name a dedicated leaf",
        )
    })?;
    ensure_capability_root(fs, parent, fs.grants_host_destructive_lifecycle())
}

fn managed_leaf_kind(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
    parent_mode: DirectoryMode,
) -> Result<Option<FsNodeKind>, CacheError> {
    if path == root
        || !path.starts_with(root)
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        || !matches!(
            path.components().next_back(),
            Some(std::path::Component::Normal(_))
        )
    {
        return Err(managed_path_refused(
            path,
            "managed cache file is not a normal leaf below its owned root",
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        managed_path_refused(path, "managed cache file must have a parent directory")
    })?;
    ensure_managed_directory(fs, root, parent, parent_mode)?;
    fs.node_kind_no_follow(path).map_err(CacheError::Storage)
}

fn read_managed_file(fs: &dyn FileSystem, root: &Path, path: &Path) -> Result<Vec<u8>, CacheError> {
    match managed_leaf_kind(fs, root, path, DirectoryMode::Existing)? {
        Some(FsNodeKind::RegularFile) => fs.read(path).map_err(CacheError::Storage),
        Some(kind) => Err(managed_node_refused(path, "regular file", kind)),
        None => Err(managed_not_found(path)),
    }
}

fn write_managed_file(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<(), CacheError> {
    match managed_leaf_kind(fs, root, path, DirectoryMode::Create)? {
        Some(FsNodeKind::RegularFile) | None => {
            fs.write_atomic(path, bytes).map_err(CacheError::Storage)
        }
        Some(kind) => Err(managed_node_refused(
            path,
            "regular file or absent leaf",
            kind,
        )),
    }
}

fn create_managed_file(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
    bytes: &[u8],
) -> Result<bool, CacheError> {
    match managed_leaf_kind(fs, root, path, DirectoryMode::Create)? {
        Some(FsNodeKind::RegularFile) | None => {
            fs.create_new(path, bytes).map_err(CacheError::Storage)
        }
        Some(kind) => Err(managed_node_refused(
            path,
            "regular file or absent leaf",
            kind,
        )),
    }
}

fn remove_managed_file(fs: &dyn FileSystem, root: &Path, path: &Path) -> Result<(), CacheError> {
    match managed_leaf_kind(fs, root, path, DirectoryMode::Existing)? {
        Some(FsNodeKind::RegularFile) => fs.remove_file(path).map_err(CacheError::Storage),
        Some(kind) => Err(managed_node_refused(path, "regular file", kind)),
        None => Err(managed_not_found(path)),
    }
}

#[derive(Debug)]
struct ManagedDirEntry {
    path: PathBuf,
    is_directory: bool,
}

fn list_managed_directory(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
) -> Result<Vec<ManagedDirEntry>, CacheError> {
    ensure_managed_directory(fs, root, path, DirectoryMode::Existing)?;
    let children = fs.list_dir(path)?;
    let mut checked = Vec::with_capacity(children.len());
    for child in children {
        if child.parent() != Some(path) {
            return Err(managed_path_refused(
                &child,
                format!(
                    "filesystem listing returned a path outside direct parent {}",
                    path.display()
                ),
            ));
        }
        let kind = match fs.node_kind_no_follow(&child) {
            Ok(kind) => kind,
            // Windows reports a delete-pending file as ACCESS_DENIED from
            // stat where every other platform reports NotFound. Both mean
            // the same thing here — a cooperating writer or evictor is
            // removing the child mid-enumeration — so map it to the
            // vanished-entry skip below instead of a hard storage error.
            // Other platforms stay strict: a denied stat there is a real
            // permission problem, not a lifecycle state.
            Err(FsError::Io { ref err, .. })
                if cfg!(windows) && err.kind() == std::io::ErrorKind::PermissionDenied =>
            {
                None
            }
            Err(err) => return Err(err.into()),
        };
        match kind {
            Some(FsNodeKind::RegularFile) => checked.push(ManagedDirEntry {
                path: child,
                is_directory: false,
            }),
            Some(FsNodeKind::Directory) => checked.push(ManagedDirEntry {
                path: child,
                is_directory: true,
            }),
            Some(kind) => {
                return Err(managed_node_refused(
                    &child,
                    "regular file or real directory",
                    kind,
                ));
            }
            // Cooperating writers and evictors may remove an entry between
            // enumeration and classification.
            None => {}
        }
    }
    Ok(checked)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OwnerManifest {
    path: Digest,
    generation: Digest,
}

impl OwnerManifest {
    fn fresh(root: &Path) -> Self {
        let mut material =
            Vec::with_capacity(root.as_os_str().as_encoded_bytes().len().saturating_add(32));
        material.extend_from_slice(root.as_os_str().as_encoded_bytes());
        material.push(0);
        material.extend_from_slice(&std::process::id().to_le_bytes());
        material.extend_from_slice(
            &SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
                .to_le_bytes(),
        );
        material.extend_from_slice(
            &NEXT_GENERATION
                .fetch_add(1, Ordering::Relaxed)
                .to_le_bytes(),
        );
        Self {
            path: sha256(root.as_os_str().as_encoded_bytes()),
            generation: sha256(&material),
        }
    }

    fn encode(self) -> Vec<u8> {
        format!(
            "{OWNER_PREFIX}{} {}\n",
            self.path.to_hex(),
            self.generation.to_hex()
        )
        .into_bytes()
    }
}

fn parse_owner_manifest(root: &Path, bytes: &[u8]) -> Result<OwnerManifest, CacheError> {
    if bytes.len() > OWNER_MANIFEST_MAX_BYTES {
        return Err(root_refused(
            root,
            RootRefusalCode::MarkerTooLarge,
            format!("ownership manifest exceeds the {OWNER_MANIFEST_MAX_BYTES}-byte limit"),
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest is not UTF-8",
        )
    })?;
    let line = text.strip_suffix('\n').ok_or_else(|| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest is not canonically newline-terminated",
        )
    })?;
    let fields = line.strip_prefix(OWNER_PREFIX).ok_or_else(|| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest has an unsupported schema",
        )
    })?;
    let (path, generation) = fields.split_once(' ').ok_or_else(|| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest is missing its generation",
        )
    })?;
    let canonical_digest = |field: &str| {
        field.len() == 64
            && field
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if !canonical_digest(path) || !canonical_digest(generation) {
        return Err(root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest fields are not canonical digests",
        ));
    }
    let path = Digest::from_hex(path).map_err(|_| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest path digest is malformed",
        )
    })?;
    let generation = Digest::from_hex(generation).map_err(|_| {
        root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            "ownership manifest generation is malformed",
        )
    })?;
    let expected_path = sha256(root.as_os_str().as_encoded_bytes());
    if path != expected_path {
        return Err(root_refused(
            root,
            RootRefusalCode::OwnershipMismatch,
            "ownership manifest is bound to a different canonical path",
        ));
    }
    Ok(OwnerManifest { path, generation })
}

fn read_managed_file_bounded(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, CacheError> {
    let bytes = read_managed_file(fs, root, path)?;
    if bytes.len() > max_bytes {
        return Err(root_refused(
            root,
            RootRefusalCode::MarkerTooLarge,
            format!("{label} exceeds the {max_bytes}-byte limit"),
        ));
    }
    Ok(bytes)
}

fn read_lifecycle_marker_for_open(
    fs: &dyn FileSystem,
    root: &Path,
    path: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, CacheError> {
    if !fs.grants_host_destructive_lifecycle() {
        return read_managed_file_bounded(fs, root, path, max_bytes, label);
    }
    match host_node_kind_no_follow(path)? {
        Some(FsNodeKind::RegularFile) => read_host_file_bounded(path, root, label, max_bytes),
        Some(_) => Err(root_refused(
            root,
            RootRefusalCode::WrongNodeKind,
            format!("{label} is not a regular file"),
        )),
        None => Err(managed_not_found(path)),
    }
}

fn verify_fixed_marker(
    fs: &dyn FileSystem,
    path: &Path,
    expected: &[u8],
    root: &Path,
    label: &str,
) -> Result<(), CacheError> {
    let found = read_lifecycle_marker_for_open(fs, root, path, expected.len(), label)?;
    if found == expected {
        Ok(())
    } else {
        Err(root_refused(
            root,
            RootRefusalCode::MarkerInvalid,
            format!("{label} is foreign or bound to a different path"),
        ))
    }
}

#[derive(Clone, Copy, Debug)]
enum RootOpenState {
    /// The root existed before this opener inspected it.
    Existing,
    /// This opener atomically created the exact host leaf.
    CreatedHostLeaf,
    /// A non-host capability may establish an isolated missing root.
    CapabilityManaged,
}

/// Establish ownership only when this opener proved it created the exact host
/// leaf, or when a non-host capability reports a missing isolated root.
fn ensure_owned_root(
    fs: &dyn FileSystem,
    root: &Path,
    state: RootOpenState,
) -> Result<OwnerManifest, CacheError> {
    if !root.is_absolute() {
        return Err(root_refused(
            root,
            RootRefusalCode::InvalidPath,
            "the store root must be absolute",
        ));
    }
    let root_existed = match fs.node_kind_no_follow(root)? {
        Some(FsNodeKind::Directory) => true,
        Some(kind) => {
            return Err(root_refused(
                root,
                RootRefusalCode::WrongNodeKind,
                format!("store root is {kind:?}, not a real directory"),
            ));
        }
        None if matches!(state, RootOpenState::CapabilityManaged) => {
            ensure_capability_root(fs, root, false)?;
            false
        }
        None => {
            return Err(root_refused(
                root,
                RootRefusalCode::IdentityChanged,
                "store root disappeared while opening",
            ));
        }
    };
    let owner_path = root.join(OWNER_FILE);
    let owner = match read_lifecycle_marker_for_open(
        fs,
        root,
        &owner_path,
        OWNER_MANIFEST_MAX_BYTES,
        "ownership manifest",
    ) {
        Ok(found) => parse_owner_manifest(root, &found)?,
        Err(CacheError::Storage(FsError::NotFound { .. })) => {
            match state {
                RootOpenState::Existing => {
                    return Err(root_refused(
                        root,
                        RootRefusalCode::OwnershipMissing,
                        "an existing directory has no ownership manifest",
                    ));
                }
                RootOpenState::CreatedHostLeaf => {}
                RootOpenState::CapabilityManaged => {
                    if root_existed {
                        return Err(root_refused(
                            root,
                            RootRefusalCode::OwnershipMissing,
                            "an existing directory has no ownership manifest",
                        ));
                    }
                }
            }
            let candidate = OwnerManifest::fresh(root);
            if create_managed_file(fs, root, &owner_path, &candidate.encode())? {
                candidate
            } else {
                // Another claimant won. Its complete published manifest, not
                // our candidate generation, is the shared root identity.
                let winner = read_lifecycle_marker_for_open(
                    fs,
                    root,
                    &owner_path,
                    OWNER_MANIFEST_MAX_BYTES,
                    "ownership manifest",
                )?;
                parse_owner_manifest(root, &winner)?
            }
        }
        Err(err) => return Err(err),
    };

    let format_path = root.join(FORMAT_FILE);
    let expected_format = format!("{FORMAT_STAMP}\n");
    match read_lifecycle_marker_for_open(
        fs,
        root,
        &format_path,
        FORMAT_STAMP_MAX_BYTES,
        "format stamp",
    )
    .and_then(|bytes| {
        String::from_utf8(bytes).map_err(|_| {
            root_refused(
                root,
                RootRefusalCode::MarkerInvalid,
                "format stamp is not UTF-8",
            )
        })
    }) {
        Ok(found) if found.trim_end() == FORMAT_STAMP => {}
        Ok(found) => Err(CacheError::FormatUnsupported {
            found: found.trim_end().to_owned(),
        })?,
        Err(CacheError::Storage(FsError::NotFound { .. })) => {
            if !create_managed_file(fs, root, &format_path, expected_format.as_bytes())? {
                verify_fixed_marker(
                    fs,
                    &format_path,
                    expected_format.as_bytes(),
                    root,
                    "format stamp",
                )?;
            }
        }
        Err(err) => return Err(err),
    }
    Ok(owner)
}

fn validate_existing_host_store_root(root: &Path, cwd: &Path) -> Result<PathBuf, CacheError> {
    reject_symlink_components(root)?;
    let canonical_root = fs::canonicalize(root).map_err(|err| host_io(root, err))?;
    if host_node_kind_no_follow(&canonical_root)? != Some(FsNodeKind::Directory) {
        return Err(root_refused(
            root,
            RootRefusalCode::WrongNodeKind,
            "root is not a real directory",
        ));
    }
    reject_protected_host_root(&canonical_root, root, cwd)?;
    Ok(canonical_root)
}

fn prepare_host_store_root(root: &Path) -> Result<(PathBuf, RootOpenState), CacheError> {
    if !root.is_absolute() {
        return Err(root_refused(
            root,
            RootRefusalCode::InvalidPath,
            "the store root must be absolute",
        ));
    }
    if root
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(root_refused(
            root,
            RootRefusalCode::InvalidPath,
            "parent-directory components are not accepted in a store root",
        ));
    }
    let cwd = env::current_dir().map_err(|err| host_io(Path::new("."), err))?;
    match host_node_kind_no_follow(root)? {
        Some(_) => validate_existing_host_store_root(root, &cwd)
            .map(|root| (root, RootOpenState::Existing)),
        None => {
            let parent = root.parent().ok_or_else(|| {
                root_refused(
                    root,
                    RootRefusalCode::InvalidPath,
                    "a store root must have an existing parent directory",
                )
            })?;
            match host_node_kind_no_follow(parent)? {
                Some(_) => {}
                None => {
                    return Err(root_refused(
                        root,
                        RootRefusalCode::InvalidPath,
                        "a store root's immediate parent must already exist",
                    ));
                }
            }
            reject_symlink_components(parent)?;
            let canonical_parent = match fs::canonicalize(parent) {
                Ok(parent) => parent,
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    return Err(root_refused(
                        root,
                        RootRefusalCode::IdentityChanged,
                        "a store root's immediate parent must already exist",
                    ));
                }
                Err(err) => return Err(host_io(parent, err)),
            };
            if host_node_kind_no_follow(&canonical_parent)? != Some(FsNodeKind::Directory) {
                return Err(root_refused(
                    root,
                    RootRefusalCode::WrongNodeKind,
                    "a store root's parent is not a real directory",
                ));
            }
            let leaf = root.file_name().ok_or_else(|| {
                root_refused(
                    root,
                    RootRefusalCode::InvalidPath,
                    "a store root must name a dedicated leaf directory",
                )
            })?;
            let canonical_root = canonical_parent.join(leaf);
            reject_protected_host_root(&canonical_root, root, &cwd)?;
            match fs::create_dir(&canonical_root) {
                Ok(()) => Ok((canonical_root, RootOpenState::CreatedHostLeaf)),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                    validate_existing_host_store_root(root, &cwd)
                        .map(|root| (root, RootOpenState::Existing))
                }
                Err(err) => Err(host_io(&canonical_root, err)),
            }
        }
    }
}

/// Result of a one-shot cache clear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheClearOutcome {
    /// A managed namespace tree was quarantined and removed.
    Cleared,
    /// The configured root or its managed namespace tree was already absent.
    AlreadyAbsent,
}

#[derive(Debug)]
enum ClearState {
    Absent,
    Owned {
        canonical_root: PathBuf,
        owner: OwnerManifest,
    },
}

/// A non-creating, single-use authorization for `--clear-cache`.
///
/// Authorization is intentionally host-only: it binds the configured path to
/// its canonical identity, rejects symlinked components and protected roots,
/// and verifies the path-and-generation ownership manifest plus a format
/// stamp. Clearing retains the root, rotates its generation, and atomically
/// quarantines only the managed `ns` subtree before recursive removal.
#[derive(Debug)]
pub struct CacheClearAuthorization {
    configured_root: PathBuf,
    state: ClearState,
}

impl CacheClearAuthorization {
    /// Inspect `root` without creating or stamping it.
    ///
    /// # Errors
    ///
    /// [`CacheError::RootRefused`] for dangerous, symlinked, foreign, or
    /// unstamped roots, and [`CacheError::Storage`] for host I/O failures.
    pub fn authorize(root: impl AsRef<Path>) -> Result<Self, CacheError> {
        let cwd = env::current_dir().map_err(|err| host_io(Path::new("."), err))?;
        let configured_root = absolute_without_parent_components(root.as_ref(), &cwd)?;
        let state = match host_node_kind_no_follow(&configured_root)? {
            Some(_) => {
                let (canonical_root, owner) = validate_host_owned_root(&configured_root, &cwd)?;
                ClearState::Owned {
                    canonical_root,
                    owner,
                }
            }
            None => ClearState::Absent,
        };
        Ok(Self {
            configured_root,
            state,
        })
    }

    /// The absolute configured root this authorization names.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.configured_root
    }

    /// Consume this authorization and clear only the managed namespace tree.
    ///
    /// The root is revalidated immediately before the linearization point.
    /// `ns` is then renamed to a unique sibling inside the retained root and
    /// the owner generation is rotated. Stale Store and Namespace handles
    /// fail their next mutation or maintenance check instead of writing into
    /// the post-clear root.
    pub fn clear(self) -> Result<CacheClearOutcome, CacheError> {
        let ClearState::Owned {
            canonical_root,
            owner,
        } = self.state
        else {
            return Ok(CacheClearOutcome::AlreadyAbsent);
        };
        let cwd = env::current_dir().map_err(|err| host_io(Path::new("."), err))?;
        let (current_root, current_owner) = validate_host_owned_root(&self.configured_root, &cwd)?;
        if current_root != canonical_root {
            return Err(root_refused(
                &self.configured_root,
                RootRefusalCode::IdentityChanged,
                "root identity changed after clear authorization",
            ));
        }
        if current_owner.generation != owner.generation {
            return Err(root_refused(
                &self.configured_root,
                RootRefusalCode::GenerationChanged,
                "owner generation changed after clear authorization",
            ));
        }

        let namespaces = canonical_root.join("ns");
        let mut quarantine = None;
        for _ in 0..MAX_QUARANTINE_ATTEMPTS {
            let candidate = canonical_root.join(format!(
                ".fmn-clear.{}.{}",
                std::process::id(),
                NEXT_CLEAR.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => {
                    quarantine = Some(candidate);
                    break;
                }
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
                Err(err) => return Err(host_io(&candidate, err)),
            }
        }
        let quarantine = quarantine.ok_or_else(|| {
            root_refused(
                &self.configured_root,
                RootRefusalCode::LifecycleCollision,
                format!(
                    "could not reserve a quarantine directory after \
                     {MAX_QUARANTINE_ATTEMPTS} collision attempts"
                ),
            )
        })?;
        let (final_root, final_owner) = validate_host_owned_root(&self.configured_root, &cwd)?;
        if final_root != canonical_root {
            return Err(root_refused(
                &self.configured_root,
                RootRefusalCode::IdentityChanged,
                "root identity changed while reserving the clear quarantine",
            ));
        }
        if final_owner.generation != owner.generation {
            return Err(root_refused(
                &self.configured_root,
                RootRefusalCode::GenerationChanged,
                "owner generation changed while reserving the clear quarantine",
            ));
        }
        let quarantined_namespaces = quarantine.join("ns");
        match with_transient_windows_retry(|| fs::rename(&namespaces, &quarantined_namespaces)) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::remove_dir(&quarantine).map_err(|cleanup_err| {
                    host_io(
                        &quarantine,
                        io::Error::new(
                            cleanup_err.kind(),
                            format!(
                                "cache namespace was absent but its empty quarantine could not be removed: {cleanup_err}"
                            ),
                        ),
                    )
                })?;
                rotate_host_generation_and_format(&canonical_root, owner)?;
                return Ok(CacheClearOutcome::AlreadyAbsent);
            }
            Err(err) => {
                let _ = fs::remove_dir(&quarantine);
                return Err(host_io(&namespaces, err));
            }
        }

        rotate_host_generation_and_format(&canonical_root, owner)?;
        with_transient_windows_retry(|| fs::remove_dir_all(&quarantine))
            .map_err(|err| host_io(&quarantine, err))?;
        Ok(CacheClearOutcome::Cleared)
    }
}

/// Run a host directory rename/removal, absorbing Windows' transient
/// refusals.
///
/// Windows reports `ERROR_ACCESS_DENIED` for renaming a directory while any
/// child file is open, and removal of a tree whose children are
/// delete-pending can surface as `PermissionDenied` or `DirectoryNotEmpty`.
/// Concurrent cache readers hold those handles only for the duration of one
/// bounded read, so the linearized clear retries briefly before surfacing
/// the refusal as real. On POSIX the first attempt always stands: `rename`
/// and `unlink` succeed against open descriptors, so the loop body is
/// unreachable there.
fn with_transient_windows_retry<T>(mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut result = op();
    if cfg!(windows) {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while let Err(err) = &result {
            let transient = matches!(
                err.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::DirectoryNotEmpty
            );
            if !transient || std::time::Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
            result = op();
        }
    }
    result
}

fn rotate_host_generation_and_format(
    root: &Path,
    expected: OwnerManifest,
) -> Result<OwnerManifest, CacheError> {
    let current = read_host_owner_manifest(root, root)?;
    if current != expected {
        return Err(root_refused(
            root,
            RootRefusalCode::GenerationChanged,
            "owner generation changed before lifecycle rotation",
        ));
    }
    let next = OwnerManifest::fresh(root);
    let fs = fmn_platform::fs::StdFs;
    // The owner manifest is the generation commit point. Refresh the format
    // first so observing `next` always implies the current format stamp.
    fs.write_atomic(
        &root.join(FORMAT_FILE),
        format!("{FORMAT_STAMP}\n").as_bytes(),
    )
    .map_err(CacheError::Storage)?;
    fs.write_atomic(&root.join(OWNER_FILE), &next.encode())
        .map_err(CacheError::Storage)?;
    Ok(next)
}

fn absolute_without_parent_components(path: &Path, cwd: &Path) -> Result<PathBuf, CacheError> {
    if !path.is_absolute() {
        return Err(root_refused(
            path,
            RootRefusalCode::InvalidPath,
            format!(
                "destructive cache targets must be absolute (current directory: {})",
                cwd.display()
            ),
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(root_refused(
            path,
            RootRefusalCode::InvalidPath,
            "parent-directory components are not accepted for destructive targets",
        ));
    }
    Ok(path.to_path_buf())
}

fn validate_host_owned_root(
    configured_root: &Path,
    cwd: &Path,
) -> Result<(PathBuf, OwnerManifest), CacheError> {
    reject_symlink_components(configured_root)?;
    let canonical_root =
        fs::canonicalize(configured_root).map_err(|err| host_io(configured_root, err))?;
    if host_node_kind_no_follow(&canonical_root)? != Some(FsNodeKind::Directory) {
        return Err(root_refused(
            configured_root,
            RootRefusalCode::WrongNodeKind,
            "root is not a real directory",
        ));
    }
    reject_protected_host_root(&canonical_root, configured_root, cwd)?;
    let owner_path = canonical_root.join(OWNER_FILE);
    reject_symlink_leaf(&owner_path, configured_root, "ownership manifest")?;
    let owner = read_host_owner_manifest(&canonical_root, configured_root)?;
    let format_path = canonical_root.join(FORMAT_FILE);
    reject_symlink_leaf(&format_path, configured_root, "format stamp")?;
    let format = read_host_file_bounded(
        &format_path,
        configured_root,
        "format stamp",
        FORMAT_STAMP_MAX_BYTES,
    )?;
    let format = std::str::from_utf8(&format).map_err(|_| {
        root_refused(
            configured_root,
            RootRefusalCode::MarkerInvalid,
            "format stamp is not UTF-8",
        )
    })?;
    if !format.trim_end().starts_with("fmn-cache ") {
        return Err(root_refused(
            configured_root,
            RootRefusalCode::MarkerInvalid,
            "format stamp is absent or foreign",
        ));
    }
    Ok((canonical_root, owner))
}

fn read_host_owner_manifest(
    canonical_root: &Path,
    diagnostic_root: &Path,
) -> Result<OwnerManifest, CacheError> {
    let path = canonical_root.join(OWNER_FILE);
    reject_symlink_leaf(&path, diagnostic_root, "ownership manifest")?;
    let bytes = read_host_file_bounded(
        &path,
        diagnostic_root,
        "ownership manifest",
        OWNER_MANIFEST_MAX_BYTES,
    )?;
    parse_owner_manifest(canonical_root, &bytes)
}

fn read_host_file_bounded(
    path: &Path,
    root: &Path,
    label: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, CacheError> {
    let file = fs::File::open(path).map_err(|err| host_io(path, err))?;
    let mut bytes = Vec::with_capacity(max_bytes.saturating_add(1));
    file.take(
        u64::try_from(max_bytes)
            .unwrap_or(u64::MAX)
            .saturating_add(1),
    )
    .read_to_end(&mut bytes)
    .map_err(|err| host_io(path, err))?;
    if bytes.len() > max_bytes {
        return Err(root_refused(
            root,
            RootRefusalCode::MarkerTooLarge,
            format!("{label} exceeds the {max_bytes}-byte limit"),
        ));
    }
    Ok(bytes)
}

fn reject_protected_host_root(
    canonical_root: &Path,
    configured_root: &Path,
    cwd: &Path,
) -> Result<(), CacheError> {
    if canonical_root.parent().is_none() {
        return Err(root_refused(
            configured_root,
            RootRefusalCode::ProtectedPath,
            "filesystem roots are never cache roots",
        ));
    }
    let canonical_cwd = fs::canonicalize(cwd).map_err(|err| host_io(cwd, err))?;
    if canonical_cwd.starts_with(canonical_root) {
        return Err(root_refused(
            configured_root,
            RootRefusalCode::ProtectedPath,
            "root is the current directory or one of its ancestors",
        ));
    }
    let home = host_home().ok_or_else(|| {
        root_refused(
            configured_root,
            RootRefusalCode::ProtectedPath,
            "home directory is unavailable; destructive authorization fails closed",
        )
    })?;
    let canonical_home = fs::canonicalize(&home).map_err(|err| host_io(&home, err))?;
    if canonical_home.starts_with(canonical_root) {
        return Err(root_refused(
            configured_root,
            RootRefusalCode::ProtectedPath,
            "root is the home directory or one of its ancestors",
        ));
    }
    Ok(())
}

fn reject_symlink_components(path: &Path) -> Result<(), CacheError> {
    let mut ancestors: Vec<&Path> = path.ancestors().collect();
    ancestors.reverse();
    for component in ancestors {
        match host_node_kind_no_follow(component)? {
            // An operating-system link above the root (macOS's `/var` and
            // `/tmp`, through which every `$TMPDIR` runs) cannot be
            // redirected by anyone but root; the root itself never may be
            // a link.
            Some(FsNodeKind::Link) if component != path && trusted_system_link(component) => {}
            Some(FsNodeKind::Link) => {
                return Err(root_refused(
                    path,
                    RootRefusalCode::WrongNodeKind,
                    "root contains a symlinked path component or reparse point",
                ));
            }
            Some(_) => {}
            None => return Err(managed_not_found(component)),
        }
    }
    Ok(())
}

/// Whether a symlinked ancestor of a cache root is an operating-system link
/// (fm-macos-var-symlink-gate-aqr1): a direct child of the filesystem root,
/// owned by root. macOS ships `/var`, `/tmp` and `/etc` this way, and
/// merged-`/usr` Linux ships `/bin` and `/lib`. Only root can place or
/// retarget such a link, and root can redirect anything anyway, so it is no
/// traversal risk the cache could defend against. Every other link stays a
/// refusal, including any link a process creates elsewhere, even as root.
/// Reparse points (non-Unix) are never trusted.
pub(crate) fn trusted_system_link(link: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        link.is_absolute()
            && link.parent() == Some(Path::new("/"))
            && fs::symlink_metadata(link)
                .is_ok_and(|metadata| metadata.file_type().is_symlink() && metadata.uid() == 0)
    }
    #[cfg(not(unix))]
    {
        let _ = link;
        false
    }
}

fn reject_symlink_leaf(path: &Path, root: &Path, label: &str) -> Result<(), CacheError> {
    match host_node_kind_no_follow(path)? {
        Some(FsNodeKind::RegularFile) => Ok(()),
        Some(_) => Err(root_refused(
            root,
            RootRefusalCode::WrongNodeKind,
            format!("{label} is not a regular file"),
        )),
        None => Err(root_refused(
            root,
            RootRefusalCode::OwnershipMissing,
            format!("{label} is absent"),
        )),
    }
}

fn host_home() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            Some(PathBuf::from(drive).join(path))
        })
}

fn host_io(path: &Path, err: io::Error) -> CacheError {
    let err = if err.kind() == io::ErrorKind::NotFound {
        FsError::NotFound {
            path: path.to_path_buf(),
        }
    } else {
        FsError::Io {
            path: path.to_path_buf(),
            err,
        }
    };
    CacheError::Storage(err)
}

fn host_node_kind_no_follow(path: &Path) -> Result<Option<FsNodeKind>, CacheError> {
    fmn_platform::fs::StdFs
        .node_kind_no_follow(path)
        .map_err(CacheError::Storage)
}

/// Config-visible store knobs (surfaced through fmn-config once fm-3gl's
/// typed config lands; constructed directly until then).
#[derive(Clone, Copy, Debug)]
pub struct StoreConfig {
    /// The per-entry payload ceiling. An over-limit payload is a precise
    /// [`CacheError::EntryTooLarge`] and the value simply goes uncached.
    pub max_entry_bytes: usize,
    /// How long a maintenance lock may sit unrenewed before another process
    /// may break it as stale. Must exceed any plausible eviction duration.
    pub lock_stale_after: Duration,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            max_entry_bytes: 256 * 1024 * 1024,
            lock_stale_after: Duration::from_secs(60),
        }
    }
}

/// Per-namespace eviction policy, config-visible.
#[derive(Clone, Copy, Debug, Default)]
pub struct NamespacePolicy {
    /// The size ceiling automatic eviction trims toward. `None` is the manual
    /// policy: no automatic eviction ever (the replay journal's namespace —
    /// its lifecycle is explicit).
    pub ceiling_bytes: Option<u64>,
}

/// What [`Namespace::evict_to_ceiling`] did.
#[derive(Debug)]
pub enum EvictOutcome {
    /// Eviction ran; the report says what happened.
    Done(EvictReport),
    /// Another maintainer holds a fresh lock; nothing was done. Callers just
    /// retry on their next maintenance tick.
    SkippedLockHeld,
    /// The namespace has no ceiling ([`NamespacePolicy::ceiling_bytes`] is
    /// `None`); automatic eviction is disabled by policy.
    Unlimited,
}

/// The accounting from one eviction pass.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct EvictReport {
    /// Entries found on disk before eviction.
    pub examined: usize,
    /// Entries removed to reach the ceiling.
    pub evicted: usize,
    /// Bytes those removals reclaimed.
    pub evicted_bytes: u64,
    /// Bytes remaining after the pass.
    pub retained_bytes: u64,
    /// Entries the pass would have evicted but skipped because they were
    /// pinned.
    pub skipped_pinned: usize,
    /// Unrecognized files swept from the object directories (orphaned temp
    /// files from killed writers, junk).
    pub swept_unrecognized: usize,
}

/// In-use marking per namespace directory, shared by every [`Namespace`]
/// handle onto that directory so pinning is a property of the store, not of
/// handle discipline.
#[derive(Debug, Default)]
struct PinSet {
    counts: Mutex<BTreeMap<Digest, usize>>,
}

impl PinSet {
    fn pin(&self, digest: Digest) {
        let mut counts = self.counts.lock().unwrap_or_else(lock_poisoned);
        *counts.entry(digest).or_insert(0) += 1;
    }

    fn unpin(&self, digest: &Digest) {
        let mut counts = self.counts.lock().unwrap_or_else(lock_poisoned);
        if let Some(n) = counts.get_mut(digest) {
            *n -= 1;
            if *n == 0 {
                counts.remove(digest);
            }
        }
    }

    fn snapshot(&self) -> BTreeSet<Digest> {
        self.counts
            .lock()
            .unwrap_or_else(lock_poisoned)
            .keys()
            .copied()
            .collect()
    }
}

/// An in-use marker: while any [`Pin`] for an address is alive, eviction will
/// not remove that entry. Dropping the pin releases it.
#[derive(Debug)]
pub struct Pin {
    set: Arc<PinSet>,
    digest: Digest,
}

impl Drop for Pin {
    fn drop(&mut self) {
        self.set.unpin(&self.digest);
    }
}

/// One entry's advisory bookkeeping.
#[derive(Clone, Copy, Debug)]
struct IndexEntry {
    /// The entry file's size in bytes (the full envelope, as stored).
    size: u64,
    /// The logical access sequence at last touch — the LRU ordinate.
    last_seq: u64,
}

/// The in-memory access log: the loaded index plus everything this handle has
/// touched since. Flushed (merged, last-writer-wins) on put, on eviction, on
/// [`Namespace::flush`], and best-effort on drop.
#[derive(Debug, Default)]
struct AccessLog {
    next_seq: u64,
    entries: BTreeMap<Digest, IndexEntry>,
}

impl AccessLog {
    /// Compact arbitrary persisted sequence values while preserving the exact
    /// eviction order `(last_seq, digest)` defines.
    fn rebase_sequences(&mut self) {
        let mut order: Vec<(u64, Digest)> = self
            .entries
            .iter()
            .map(|(digest, entry)| (entry.last_seq, *digest))
            .collect();
        order.sort_unstable();
        for (rank, (_, digest)) in order.into_iter().enumerate() {
            let rank = u64::try_from(rank).unwrap_or(u64::MAX);
            if let Some(entry) = self.entries.get_mut(&digest) {
                entry.last_seq = rank;
            }
        }
        self.next_seq = u64::try_from(self.entries.len()).unwrap_or(u64::MAX);
    }

    fn take_next_sequence(&mut self) -> u64 {
        if self.next_seq == u64::MAX {
            self.rebase_sequences();
        }
        let sequence = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        sequence
    }

    fn total_size(&self) -> u64 {
        self.entries
            .values()
            .fold(0u64, |total, entry| total.saturating_add(entry.size))
    }
}

#[derive(Debug)]
struct RootContext {
    path: PathBuf,
    owner: OwnerManifest,
}

impl RootContext {
    fn validate_generation(&self, fs: &dyn FileSystem) -> Result<(), CacheError> {
        if fs.grants_host_destructive_lifecycle() {
            reject_symlink_components(&self.path)?;
            if host_node_kind_no_follow(&self.path)? != Some(FsNodeKind::Directory) {
                return Err(root_refused(
                    &self.path,
                    RootRefusalCode::IdentityChanged,
                    "owned host root is no longer the opened directory",
                ));
            }
            let current = read_host_owner_manifest(&self.path, &self.path)?;
            return self.validate_current(current);
        }
        match fs.node_kind_no_follow(&self.path)? {
            Some(FsNodeKind::Directory) => {}
            Some(_) => {
                return Err(root_refused(
                    &self.path,
                    RootRefusalCode::IdentityChanged,
                    "owned root is no longer the opened directory",
                ));
            }
            None => {
                return Err(root_refused(
                    &self.path,
                    RootRefusalCode::IdentityChanged,
                    "owned root disappeared after opening",
                ));
            }
        }
        let owner_path = self.path.join(OWNER_FILE);
        let bytes = match fs.node_kind_no_follow(&owner_path)? {
            Some(FsNodeKind::RegularFile) => fs.read(&owner_path)?,
            Some(_) => {
                return Err(root_refused(
                    &self.path,
                    RootRefusalCode::WrongNodeKind,
                    "ownership manifest is no longer a regular file",
                ));
            }
            None => {
                return Err(root_refused(
                    &self.path,
                    RootRefusalCode::OwnershipMissing,
                    "ownership manifest disappeared after opening",
                ));
            }
        };
        let current = parse_owner_manifest(&self.path, &bytes)?;
        self.validate_current(current)
    }

    fn validate_current(&self, current: OwnerManifest) -> Result<(), CacheError> {
        if current.generation != self.owner.generation {
            return Err(root_refused(
                &self.path,
                RootRefusalCode::GenerationChanged,
                "owner generation changed after this handle opened",
            ));
        }
        Ok(())
    }
}

struct StoreInner {
    fs: Arc<dyn FileSystem>,
    clock: Arc<dyn Clock>,
    root: RootContext,
    config: StoreConfig,
    /// Serial limits for entry envelopes, derived from `max_entry_bytes`.
    entry_limits: Limits,
    /// This store opening's instance id (lock-token uniqueness).
    instance: u64,
    /// Shared pin sets, one per namespace directory.
    pins: Mutex<HashMap<PathBuf, Arc<PinSet>>>,
}

impl StoreInner {
    fn validate_existing_directory_prefix(&self, path: &Path) -> Result<(), CacheError> {
        self.root.validate_generation(self.fs.as_ref())?;
        ensure_managed_directory(
            self.fs.as_ref(),
            &self.root.path,
            path,
            DirectoryMode::ExistingPrefix,
        )
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, CacheError> {
        read_managed_file(self.fs.as_ref(), &self.root.path, path)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<(), CacheError> {
        self.root.validate_generation(self.fs.as_ref())?;
        write_managed_file(self.fs.as_ref(), &self.root.path, path, bytes)
    }

    fn create_file(&self, path: &Path, bytes: &[u8]) -> Result<bool, CacheError> {
        self.root.validate_generation(self.fs.as_ref())?;
        create_managed_file(self.fs.as_ref(), &self.root.path, path, bytes)
    }

    fn remove_file(&self, path: &Path) -> Result<(), CacheError> {
        self.root.validate_generation(self.fs.as_ref())?;
        remove_managed_file(self.fs.as_ref(), &self.root.path, path)
    }

    fn list_directory(&self, path: &Path) -> Result<Vec<ManagedDirEntry>, CacheError> {
        self.root.validate_generation(self.fs.as_ref())?;
        list_managed_directory(self.fs.as_ref(), &self.root.path, path)
    }
}

/// The persistent content-addressed store. Cheap to clone conceptually — open
/// namespaces via [`Store::namespace`]; the store itself is just the root,
/// the capabilities, and the config.
pub struct Store {
    inner: Arc<StoreInner>,
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("root", &self.inner.root.path)
            .field("config", &self.inner.config)
            .finish_non_exhaustive()
    }
}

impl Store {
    /// Resolve the effective host cache setting and open its owned store.
    ///
    /// This is the construction entry point for a user-facing cache setting:
    /// a configured absolute or relative path follows
    /// [`resolve_host_cache_root`], while an empty setting selects the
    /// platform default. For that default only, a missing platform cache base
    /// is created one real directory component at a time; the dedicated
    /// [`DEFAULT_CACHE_LEAF`] remains for [`Store::open`] to claim and stamp.
    ///
    /// # Errors
    ///
    /// [`CacheError::RootResolution`] if the setting cannot be resolved,
    /// [`CacheError::RootRefused`] if the capability is not the ambient host
    /// filesystem or ownership cannot be proven,
    /// [`CacheError::FormatUnsupported`] if the owned root carries a stamp
    /// from a different store format, or [`CacheError::Storage`].
    pub fn open_host(
        fs: Arc<dyn FileSystem>,
        clock: Arc<dyn Clock>,
        configured: &str,
        config: StoreConfig,
    ) -> Result<Self, CacheError> {
        let root = resolve_host_cache_root(configured)?;
        if !fs.grants_host_destructive_lifecycle() {
            return Err(root_refused(
                &root,
                RootRefusalCode::CapabilityRequired,
                "host cache configuration requires the ambient host filesystem capability",
            ));
        }
        if configured.is_empty() {
            ensure_platform_cache_parent(fs.as_ref(), &root)?;
        }
        Self::open(fs, clock, root, config)
    }

    /// Open (creating if needed) the owned store at `root`.
    ///
    /// Callers starting from the user-facing cache configuration use
    /// [`Store::open_host`]; this lower-level constructor accepts an already
    /// resolved absolute root.
    ///
    /// A missing absolute leaf under an existing real parent may be claimed.
    /// Any existing unstamped directory (including an empty one), a symlinked
    /// host path, a copied/path-mismatched ownership marker, or a lost stamp
    /// race is refused.
    ///
    /// # Errors
    /// [`CacheError::RootRefused`] if ownership cannot be proven,
    /// [`CacheError::FormatUnsupported`] if the owned root carries a stamp
    /// from a different store format, or [`CacheError::Storage`].
    pub fn open(
        fs: Arc<dyn FileSystem>,
        clock: Arc<dyn Clock>,
        root: impl Into<PathBuf>,
        config: StoreConfig,
    ) -> Result<Self, CacheError> {
        let requested_root = root.into();
        let (root, state) = if fs.grants_host_destructive_lifecycle() {
            prepare_host_store_root(&requested_root)?
        } else {
            (requested_root, RootOpenState::CapabilityManaged)
        };
        let owner = ensure_owned_root(fs.as_ref(), &root, state)?;

        let entry_limits = Limits {
            max_field: config.max_entry_bytes,
            // Envelope overhead (header, kind, address, length prefix,
            // checksum) is well under this slack.
            max_total: config.max_entry_bytes.saturating_add(4096),
        };
        Ok(Self {
            inner: Arc::new(StoreInner {
                fs,
                clock,
                root: RootContext { path: root, owner },
                config,
                entry_limits,
                instance: NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed),
                pins: Mutex::new(HashMap::new()),
            }),
        })
    }

    /// The store root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.inner.root.path
    }

    /// Open a versioned namespace. The name is validated (the traversal
    /// boundary); the version selects the directory, so a bump is a clean,
    /// namespace-local cold start. Sibling versions are never removed here:
    /// another process may still have a live handle onto any one of them.
    ///
    /// # Errors
    /// [`CacheError::InvalidNamespace`], or [`CacheError::Storage`] when an
    /// existing namespace component is link-like or has the wrong node kind.
    pub fn namespace(
        &self,
        name: &str,
        version: u32,
        policy: NamespacePolicy,
    ) -> Result<Namespace, CacheError> {
        validate_namespace_name(name)?;
        let dir = self
            .inner
            .root
            .path
            .join("ns")
            .join(name)
            .join(format!("v{version}"));
        self.inner.validate_existing_directory_prefix(&dir)?;
        let pins = {
            let mut registry = self.inner.pins.lock().unwrap_or_else(lock_poisoned);
            Arc::clone(registry.entry(dir.clone()).or_default())
        };
        let ns = Namespace {
            inner: Arc::clone(&self.inner),
            name: name.to_owned(),
            version,
            objects_dir: dir.join("objects"),
            index_path: dir.join("index"),
            lock_path: dir.join("lock"),
            policy,
            pins,
            access: Mutex::new(AccessLog::default()),
            held_lock: Mutex::new(None),
            rejected: AtomicU64::new(0),
        };
        ns.load_index();
        Ok(ns)
    }
}

/// Reject any namespace name that could perturb pathing. The rule is strict
/// on purpose: lowercase alphanumerics, `-`, `_`, first byte alphanumeric,
/// at most 64 bytes. No dots ever, so no `.`/`..`; no separators; no
/// platform-magic names.
pub fn validate_namespace_name(name: &str) -> Result<(), CacheError> {
    let reject = |reason: &'static str| {
        Err(CacheError::InvalidNamespace {
            name: name.to_owned(),
            reason,
        })
    };
    if name.is_empty() {
        return reject("empty");
    }
    if name.len() > 64 {
        return reject("longer than 64 bytes");
    }
    let bytes = name.as_bytes();
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return reject("must start with a lowercase letter or digit");
    }
    if !bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-' || *b == b'_')
    {
        return reject("only [a-z0-9_-] allowed");
    }
    Ok(())
}

/// One versioned namespace of the store. See the crate docs for the get/put,
/// pinning, and eviction contracts.
pub struct Namespace {
    inner: Arc<StoreInner>,
    name: String,
    version: u32,
    objects_dir: PathBuf,
    index_path: PathBuf,
    lock_path: PathBuf,
    policy: NamespacePolicy,
    pins: Arc<PinSet>,
    access: Mutex<AccessLog>,
    /// The exact lock-token bytes we hold, if any (release verifies them).
    held_lock: Mutex<Option<Vec<u8>>>,
    /// Entries this handle found corrupt on read, evicted, and reported as
    /// misses. Diagnostic only; never part of any certified identity.
    rejected: AtomicU64,
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Namespace")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl Namespace {
    /// The namespace name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The namespace schema version.
    #[must_use]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Entries this handle detected as corrupt on read since it was opened.
    /// Each was evicted and served as a miss, so the consumer recomputed and
    /// may republish it. Diagnostic only, like every cache counter.
    #[must_use]
    pub fn rejected_entries(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }
}
/// The traversal-safe object path relative to a namespace's objects
/// directory: `<hh>/<hex…>`, derived from digest hex only. Pure by design —
/// the fm-tdp fuzz target asserts the invariant (exactly two components,
/// both lowercase hex, lengths 2 and 62) over arbitrary digests.
pub fn object_relative_path(digest: &Digest) -> PathBuf {
    let hex = digest.to_hex();
    PathBuf::from(&hex[..2]).join(&hex[2..])
}

impl Namespace {
    /// The object path for an address: `<objects_dir>/<hh>/<hex…>`,
    /// derived from digest hex only — the traversal protection.
    fn object_path(&self, digest: &Digest) -> PathBuf {
        self.objects_dir.join(object_relative_path(digest))
    }

    // ------------------------------------------------------------------
    // Keyed entries
    // ------------------------------------------------------------------

    /// Look up a keyed entry. `Ok(None)` is a miss — including the corrupt
    /// case, where the bad entry is first evicted (never trusted, never
    /// fatal).
    ///
    /// # Errors
    /// [`CacheError::Storage`] only for real filesystem failures (not
    /// absence, not corruption).
    pub fn get(&self, key: &CacheKey) -> Result<Option<Vec<u8>>, CacheError> {
        self.get_at(key.digest(), EntryKind::Keyed)
    }

    /// Store a keyed entry through create-if-absent publication. Re-publishing
    /// an identical payload is idempotent; a different payload at the same
    /// key is refused without replacing the incumbent.
    ///
    /// # Errors
    /// [`CacheError::EntryTooLarge`] over the per-entry ceiling,
    /// [`CacheError::KeyConflict`] when another producer has already published
    /// different bytes, [`CacheError::Encode`], or [`CacheError::Storage`].
    /// All are safely ignorable — an unwritten cache entry is a future
    /// recompute.
    pub fn put(&self, key: &CacheKey, payload: &[u8]) -> Result<(), CacheError> {
        self.put_at(key.digest(), EntryKind::Keyed, payload)
    }

    /// The read-through composition: a verified hit, or `compute` — with
    /// every cache failure (storage trouble included) degraded to a
    /// recompute, and the computed value stored best-effort. This is the
    /// never-fatal contract as an API shape; `compute`'s own error is the
    /// only error that escapes.
    ///
    /// # Errors
    /// Exactly the errors of `compute`.
    pub fn get_or_compute<E>(
        &self,
        key: &CacheKey,
        compute: impl FnOnce() -> Result<Vec<u8>, E>,
    ) -> Result<Vec<u8>, E> {
        if let Ok(Some(hit)) = self.get(key) {
            return Ok(hit);
        }
        let value = compute()?;
        let _ = self.put(key, &value);
        Ok(value)
    }

    // ------------------------------------------------------------------
    // Blob (content-addressed) entries
    // ------------------------------------------------------------------

    /// Store content under its own hash and return the address. Fetched
    /// assets live here: the address doubles as the integrity statement.
    ///
    /// # Errors
    /// [`CacheError::EntryTooLarge`] over the per-entry ceiling,
    /// [`CacheError::Encode`], or [`CacheError::Storage`]. A verified payload
    /// mismatch at the same content digest is reported as storage corruption,
    /// not as a keyed-producer conflict.
    pub fn put_blob(&self, payload: &[u8]) -> Result<Digest, CacheError> {
        let digest = sha256(payload);
        self.put_at(&digest, EntryKind::Blob, payload)?;
        Ok(digest)
    }

    /// Look up content by its hash; the payload is verified against the
    /// address itself (self-certifying), on top of the envelope checksum.
    ///
    /// # Errors
    /// As [`Namespace::get`].
    pub fn get_blob(&self, digest: &Digest) -> Result<Option<Vec<u8>>, CacheError> {
        self.get_at(digest, EntryKind::Blob)
    }

    fn get_at(&self, digest: &Digest, kind: EntryKind) -> Result<Option<Vec<u8>>, CacheError> {
        let path = self.object_path(digest);
        let bytes = match self.inner.read_file(&path) {
            Ok(bytes) => bytes,
            Err(CacheError::Storage(FsError::NotFound { .. })) => {
                // A ghost (evicted elsewhere): drop any bookkeeping.
                self.forget(digest);
                return Ok(None);
            }
            Err(err) => return Err(err),
        };
        match entry::decode(&bytes, kind, digest, self.inner.entry_limits) {
            Ok(payload) => {
                self.touch(digest, bytes.len() as u64);
                Ok(Some(payload))
            }
            Err(_corrupt) => {
                // Evicted, never trusted, never fatal: the next lookup is a
                // clean miss and the consumer recomputes.
                self.rejected.fetch_add(1, Ordering::Relaxed);
                let _ = self.inner.remove_file(&path);
                self.forget(digest);
                Ok(None)
            }
        }
    }

    fn put_at(&self, digest: &Digest, kind: EntryKind, payload: &[u8]) -> Result<(), CacheError> {
        if payload.len() > self.inner.config.max_entry_bytes {
            return Err(CacheError::EntryTooLarge {
                limit: self.inner.config.max_entry_bytes,
                needed: payload.len(),
            });
        }
        let doc = entry::encode(kind, digest, payload, self.inner.entry_limits)?;
        let path = self.object_path(digest);
        let size = if self.inner.create_file(&path, &doc)? {
            doc.len() as u64
        } else {
            // A complete incumbent won create-if-absent publication. Verify
            // its envelope, address, kind, and payload before treating this
            // losing write as idempotent.
            let incumbent = self.inner.read_file(&path)?;
            let incumbent_payload =
                entry::decode(&incumbent, kind, digest, self.inner.entry_limits).map_err(
                    |err| {
                        CacheError::Storage(FsError::Io {
                            path: path.clone(),
                            err: io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "immutable cache object could not be verified after a \
                                     create collision: {err:?}"
                                ),
                            ),
                        })
                    },
                )?;
            if incumbent_payload != payload {
                if kind == EntryKind::Keyed {
                    return Err(CacheError::KeyConflict(Box::new(KeyConflict {
                        namespace: self.name.clone(),
                        version: self.version,
                        key: *digest,
                        incumbent_payload: sha256(&incumbent_payload),
                        offered_payload: sha256(payload),
                    })));
                }
                return Err(CacheError::Storage(FsError::Io {
                    path,
                    err: io::Error::new(
                        io::ErrorKind::InvalidData,
                        "content-addressed blob collision carried different verified payloads",
                    ),
                }));
            }
            incumbent.len() as u64
        };
        self.touch(digest, size);
        // Durable bookkeeping rides the (cold) write path; read bumps stay
        // in memory until some flush point.
        let _ = self.flush();
        Ok(())
    }

    // ------------------------------------------------------------------
    // Pinning
    // ------------------------------------------------------------------

    /// Pin a keyed entry's address against eviction while the guard lives.
    /// Pinning is per-store-process and shared across every handle onto this
    /// namespace directory; pin-then-put is legitimate.
    #[must_use]
    pub fn pin(&self, key: &CacheKey) -> Pin {
        self.pin_digest(*key.digest())
    }

    /// Pin an address directly (blob addresses, journal segments).
    #[must_use]
    pub fn pin_digest(&self, digest: Digest) -> Pin {
        self.pins.pin(digest);
        Pin {
            set: Arc::clone(&self.pins),
            digest,
        }
    }

    // ------------------------------------------------------------------
    // The advisory index
    // ------------------------------------------------------------------

    fn touch(&self, digest: &Digest, size: u64) {
        let mut log = self.access.lock().unwrap_or_else(lock_poisoned);
        let seq = log.take_next_sequence();
        log.entries.insert(
            *digest,
            IndexEntry {
                size,
                last_seq: seq,
            },
        );
    }

    fn forget(&self, digest: &Digest) {
        let mut log = self.access.lock().unwrap_or_else(lock_poisoned);
        log.entries.remove(digest);
    }

    /// Load the on-disk index into the access log; any failure (absent,
    /// corrupt, foreign version) is an empty log — the index is advisory and
    /// eviction rebuilds it from disk truth.
    fn load_index(&self) {
        if let Some(loaded) = self.read_index_file() {
            *self.access.lock().unwrap_or_else(lock_poisoned) = loaded;
        }
    }

    fn read_index_file(&self) -> Option<AccessLog> {
        let bytes = self.inner.read_file(&self.index_path).ok()?;
        let mut r =
            Reader::open(&bytes, INDEX_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict).ok()?;
        let next_seq = r.get_u64().ok()?;
        let count = r.get_u64().ok()?;
        let mut entries = BTreeMap::new();
        for _ in 0..count {
            let digest = r.get_digest().ok()?;
            let size = r.get_u64().ok()?;
            let last_seq = r.get_u64().ok()?;
            entries.insert(digest, IndexEntry { size, last_seq });
        }
        r.finish().ok()?;
        let mut log = AccessLog { next_seq, entries };
        log.rebase_sequences();
        Some(log)
    }

    /// Merge this handle's access log with the on-disk index (max sequence
    /// wins per entry) and write it back atomically. Concurrent flushes are
    /// last-writer-wins — the index is advisory by design.
    ///
    /// # Errors
    /// [`CacheError::Storage`] or [`CacheError::Encode`]; callers on the hot
    /// path ignore both (bookkeeping, not data).
    pub fn flush(&self) -> Result<(), CacheError> {
        let mut log = self.access.lock().unwrap_or_else(lock_poisoned);
        if let Some(disk) = self.read_index_file() {
            for (digest, theirs) in disk.entries {
                log.entries
                    .entry(digest)
                    .and_modify(|ours| {
                        if theirs.last_seq > ours.last_seq {
                            ours.last_seq = theirs.last_seq;
                        }
                    })
                    .or_insert(theirs);
            }
        }
        log.rebase_sequences();
        self.write_index(&log)
    }

    fn write_index(&self, log: &AccessLog) -> Result<(), CacheError> {
        let mut w = Writer::new(INDEX_SCHEMA);
        w.put_u64(log.next_seq);
        w.put_u64(log.entries.len() as u64);
        for (digest, e) in &log.entries {
            w.put_digest(digest);
            w.put_u64(e.size);
            w.put_u64(e.last_seq);
        }
        let doc = w.finish()?;
        self.inner.write_file(&self.index_path, &doc)?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Maintenance: scan, eviction, version purge
    // ------------------------------------------------------------------

    /// Walk the object directories: every parseable entry address, plus the
    /// paths of unrecognized files (orphaned writer temps, junk).
    fn scan(&self) -> Result<(BTreeSet<Digest>, Vec<PathBuf>), CacheError> {
        let mut digests = BTreeSet::new();
        let mut unrecognized = Vec::new();
        let shards = match self.inner.list_directory(&self.objects_dir) {
            Ok(shards) => shards,
            Err(CacheError::Storage(FsError::NotFound { .. })) => {
                return Ok((digests, unrecognized));
            }
            Err(err) => return Err(err),
        };
        for shard in shards {
            if !shard.is_directory {
                unrecognized.push(shard.path);
                continue;
            }
            let Some(shard_name) = shard
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
            else {
                continue;
            };
            if shard_name.len() != 2 || !shard_name.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                continue;
            }
            let files = match self.inner.list_directory(&shard.path) {
                Ok(files) => files,
                Err(CacheError::Storage(FsError::NotFound { .. })) => continue,
                Err(err) => return Err(err),
            };
            for file in files {
                if file.is_directory {
                    continue;
                }
                let Some(file_name) = file
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                else {
                    continue;
                };
                match Digest::from_hex(&format!("{shard_name}{file_name}")) {
                    Ok(digest) => {
                        digests.insert(digest);
                    }
                    Err(_) => unrecognized.push(file.path),
                }
            }
        }
        Ok((digests, unrecognized))
    }

    fn object_size(&self, digest: &Digest) -> Result<Option<u64>, CacheError> {
        match self.inner.read_file(&self.object_path(digest)) {
            Ok(bytes) => Ok(Some(u64::try_from(bytes.len()).unwrap_or(u64::MAX))),
            Err(CacheError::Storage(FsError::NotFound { .. })) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Reconcile the advisory index against one disk scan. Persisted sizes are
    /// never authoritative: every surviving object is measured again.
    fn reconcile_index(
        &self,
        on_disk: &BTreeSet<Digest>,
        log: &mut AccessLog,
    ) -> Result<(), CacheError> {
        log.entries.retain(|digest, _| on_disk.contains(digest));
        for digest in on_disk {
            match self.object_size(digest)? {
                Some(size) => {
                    log.entries
                        .entry(*digest)
                        .and_modify(|entry| entry.size = size)
                        .or_insert(IndexEntry { size, last_seq: 0 });
                }
                None => {
                    // A racing remover won after the scan. Absence is disk
                    // truth too, so do not retain a ghost index entry.
                    log.entries.remove(digest);
                }
            }
        }
        Ok(())
    }

    /// Total bytes currently stored in this namespace (entry envelopes as on
    /// disk).
    ///
    /// # Errors
    /// [`CacheError::Storage`].
    pub fn usage(&self) -> Result<u64, CacheError> {
        let (digests, _) = self.scan()?;
        let mut log = self.access.lock().unwrap_or_else(lock_poisoned);
        self.reconcile_index(&digests, &mut log)?;
        Ok(log.total_size())
    }

    /// Trim this namespace toward its ceiling: least-recently-used first
    /// (logical access order, ties by digest for determinism), skipping
    /// pinned entries, sweeping unrecognized files, and reconciling the
    /// advisory index against disk truth. Non-blocking: if another
    /// maintainer holds a fresh lock this returns
    /// [`EvictOutcome::SkippedLockHeld`].
    ///
    /// # Errors
    /// [`CacheError::Storage`] on real filesystem failures mid-pass.
    pub fn evict_to_ceiling(&self) -> Result<EvictOutcome, CacheError> {
        let Some(ceiling) = self.policy.ceiling_bytes else {
            return Ok(EvictOutcome::Unlimited);
        };
        if !self.acquire_maintenance_lock()? {
            return Ok(EvictOutcome::SkippedLockHeld);
        }
        let outcome = self.evict_under_lock(ceiling);
        self.release_maintenance_lock();
        outcome.map(EvictOutcome::Done)
    }

    fn evict_under_lock(&self, ceiling: u64) -> Result<EvictReport, CacheError> {
        let (on_disk, unrecognized) = self.scan()?;
        let mut report = EvictReport {
            examined: on_disk.len(),
            ..EvictReport::default()
        };
        for path in unrecognized {
            if self.inner.remove_file(&path).is_ok() {
                report.swept_unrecognized = report.swept_unrecognized.saturating_add(1);
            }
        }

        let mut log = self.access.lock().unwrap_or_else(lock_poisoned);
        // Reconcile: disk is the truth. Ghost log entries drop; strangers
        // (entries other processes wrote) enter with sequence 0, so they are
        // first out unless someone touches them.
        self.reconcile_index(&on_disk, &mut log)?;

        let mut total = log.total_size();
        if total > ceiling {
            let pinned = self.pins.snapshot();
            let mut order: Vec<(u64, Digest, u64)> = log
                .entries
                .iter()
                .map(|(digest, e)| (e.last_seq, *digest, e.size))
                .collect();
            order.sort_unstable();
            for (_, digest, size) in order {
                if total <= ceiling {
                    break;
                }
                if pinned.contains(&digest) {
                    report.skipped_pinned = report.skipped_pinned.saturating_add(1);
                    continue;
                }
                match self.inner.remove_file(&self.object_path(&digest)) {
                    Ok(()) | Err(CacheError::Storage(FsError::NotFound { .. })) => {
                        log.entries.remove(&digest);
                        total = total.saturating_sub(size);
                        report.evicted = report.evicted.saturating_add(1);
                        report.evicted_bytes = report.evicted_bytes.saturating_add(size);
                    }
                    Err(err) => return Err(err),
                }
            }
        }
        report.retained_bytes = total;
        log.rebase_sequences();
        self.write_index(&log)?;
        Ok(report)
    }

    // ------------------------------------------------------------------
    // The advisory maintenance lock
    // ------------------------------------------------------------------

    fn now_wall_nanos(&self) -> u64 {
        self.inner
            .clock
            .wall()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    fn lock_token(&self) -> Result<Vec<u8>, CacheError> {
        let mut w = Writer::new(LOCK_SCHEMA);
        w.put_u64(u64::from(std::process::id()));
        w.put_u64(self.inner.instance);
        w.put_u64(self.now_wall_nanos());
        Ok(w.finish()?)
    }

    fn lock_acquired_nanos(bytes: &[u8]) -> Option<u64> {
        let mut r =
            Reader::open(bytes, LOCK_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict).ok()?;
        let _pid = r.get_u64().ok()?;
        let _instance = r.get_u64().ok()?;
        let acquired = r.get_u64().ok()?;
        r.finish().ok()?;
        Some(acquired)
    }

    /// Try to take the maintenance lock: `Ok(true)` if held after this call.
    /// A fresh foreign lock means `Ok(false)` (skip, never block); a stale or
    /// unparseable one is broken and re-contended once.
    fn acquire_maintenance_lock(&self) -> Result<bool, CacheError> {
        let token = self.lock_token()?;
        if self.inner.create_file(&self.lock_path, &token)? {
            *self.held_lock.lock().unwrap_or_else(lock_poisoned) = Some(token);
            return Ok(true);
        }
        // Occupied: fresh means skip; stale or garbage means break and
        // re-contend (create_new arbitrates the re-contention race).
        let breakable = match self.inner.read_file(&self.lock_path) {
            Ok(existing) => match Self::lock_acquired_nanos(&existing) {
                Some(acquired) => {
                    let age = self.now_wall_nanos().saturating_sub(acquired);
                    u128::from(age) > self.inner.config.lock_stale_after.as_nanos()
                }
                // An unparseable token can never renew or expire; treat it
                // as abandoned.
                None => true,
            },
            // Vanished between create_new and read: the holder released;
            // re-contend.
            Err(CacheError::Storage(FsError::NotFound { .. })) => true,
            Err(err) => return Err(err),
        };
        if !breakable {
            return Ok(false);
        }
        let _ = self.inner.remove_file(&self.lock_path);
        if self.inner.create_file(&self.lock_path, &token)? {
            *self.held_lock.lock().unwrap_or_else(lock_poisoned) = Some(token);
            return Ok(true);
        }
        Ok(false)
    }

    /// Release the maintenance lock if the file still carries our token (a
    /// staleness-breaker may have replaced it; never remove someone else's).
    fn release_maintenance_lock(&self) {
        let mut held = self.held_lock.lock().unwrap_or_else(lock_poisoned);
        if let Some(token) = held.take()
            && self
                .inner
                .read_file(&self.lock_path)
                .is_ok_and(|cur| cur == token)
        {
            let _ = self.inner.remove_file(&self.lock_path);
        }
    }
}

impl Drop for Namespace {
    fn drop(&mut self) {
        // Best-effort: persist read bumps. The index is advisory, so a
        // failure here costs LRU accuracy only.
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_absolute(name: &str) -> PathBuf {
        #[cfg(windows)]
        {
            PathBuf::from(format!(r"C:\{name}"))
        }
        #[cfg(not(windows))]
        {
            PathBuf::from(format!("/{name}"))
        }
    }

    fn root_environment(platform: &'static str) -> CacheRootEnvironment {
        CacheRootEnvironment {
            platform,
            current_dir: Some(test_absolute("work")),
            xdg_cache_home: None,
            home: None,
            local_app_data: None,
            user_profile: None,
        }
    }

    /// fm-macos-var-symlink-gate-aqr1: every root-owned link directly under
    /// `/` on this host is a trusted ancestor, and a link anywhere else is
    /// not, whoever owns it (the torture suite holds the full refusal).
    #[cfg(unix)]
    #[test]
    fn only_root_owned_top_level_links_are_trusted_ancestors() {
        use std::os::unix::fs::MetadataExt;
        for entry in fs::read_dir("/")
            .expect("the filesystem root lists")
            .flatten()
        {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            let expected = metadata.file_type().is_symlink() && metadata.uid() == 0;
            assert_eq!(trusted_system_link(&path), expected, "{}", path.display());
        }
        assert!(!trusted_system_link(Path::new("/")));
        let dir = std::env::temp_dir().join(format!("fmn-cache-trust-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch directory");
        let link = dir.join("alias");
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&dir, &link).expect("scratch link");
        assert!(
            !trusted_system_link(&link),
            "a link below the filesystem root is never trusted"
        );
    }

    /// fm-macos-var-symlink-gate-aqr1: the platform cache base may sit below
    /// an operating-system link (a `$HOME` under macOS `/var`), as an explicit
    /// store root may; a link a user planted above it is still refused and
    /// nothing is created through it. The accepting half needs a root-owned
    /// top-level directory link on the host (macOS `/var`, merged-`/usr`
    /// Linux `/bin`).
    #[cfg(unix)]
    #[test]
    fn platform_parent_preparation_trusts_only_operating_system_links() {
        use fmn_platform::fs::StdFs;

        let system_link = fs::read_dir("/")
            .expect("the filesystem root lists")
            .flatten()
            .map(|entry| entry.path())
            .find(|path| trusted_system_link(path) && path.is_dir());
        if let Some(link) = system_link {
            ensure_platform_cache_parent(&StdFs, &link.join(DEFAULT_CACHE_LEAF))
                .expect("a base below an operating-system link is accepted");
        }

        let dir =
            std::env::temp_dir().join(format!("fmn-cache-platform-link-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch directory");
        let canonical = fs::canonicalize(&dir).expect("canonical scratch directory");
        let alias = canonical.join("alias");
        let _ = fs::remove_file(&alias);
        std::os::unix::fs::symlink(&canonical, &alias).expect("scratch link");
        let base = alias.join(".cache");
        match ensure_platform_cache_parent(&StdFs, &base.join(DEFAULT_CACHE_LEAF)) {
            Err(CacheError::RootRefused { code, .. }) => {
                assert_eq!(code, RootRefusalCode::WrongNodeKind);
            }
            other => panic!("a user link above the cache base must be refused: {other:?}"),
        }
        assert!(
            !canonical.join(".cache").exists(),
            "nothing is created through the refused link"
        );
    }

    #[test]
    fn configured_cache_roots_become_absolute_without_rewriting() {
        let environment = root_environment("linux");
        assert_eq!(
            resolve_cache_root("relative/cache", &environment).unwrap(),
            test_absolute("work").join("relative/cache")
        );
        let absolute = test_absolute("explicit-cache");
        assert_eq!(
            resolve_cache_root(absolute.to_str().unwrap(), &environment).unwrap(),
            absolute
        );
        assert!(matches!(
            resolve_cache_root("../escape", &environment),
            Err(CacheRootError::InvalidConfigured { .. })
        ));
    }

    #[test]
    fn platform_cache_defaults_have_one_dedicated_leaf() {
        let unix_base = test_absolute("xdg");
        let mut unix = root_environment("linux");
        unix.xdg_cache_home = Some(unix_base.clone().into_os_string());
        unix.home = Some(test_absolute("home").into_os_string());
        assert_eq!(
            resolve_cache_root("", &unix).unwrap(),
            unix_base.join(DEFAULT_CACHE_LEAF)
        );

        let home = test_absolute("home");
        unix.xdg_cache_home = Some(PathBuf::from("relative-xdg").into_os_string());
        unix.home = Some(home.clone().into_os_string());
        assert_eq!(
            resolve_cache_root("", &unix).unwrap(),
            home.join(".cache").join(DEFAULT_CACHE_LEAF)
        );

        let mut macos = root_environment("macos");
        macos.home = Some(home.clone().into_os_string());
        assert_eq!(
            resolve_cache_root("", &macos).unwrap(),
            home.join("Library").join("Caches").join(DEFAULT_CACHE_LEAF)
        );

        let local = test_absolute("local-app-data");
        let mut windows = root_environment("windows");
        windows.local_app_data = Some(local.clone().into_os_string());
        assert_eq!(
            resolve_cache_root("", &windows).unwrap(),
            local.join(DEFAULT_CACHE_LEAF)
        );
        windows.local_app_data = None;
        let profile = test_absolute("windows-profile");
        windows.user_profile = Some(profile.clone().into_os_string());
        assert_eq!(
            resolve_cache_root("", &windows).unwrap(),
            profile
                .join("AppData")
                .join("Local")
                .join(DEFAULT_CACHE_LEAF)
        );
    }

    #[test]
    fn platform_cache_default_never_guesses_cwd_or_temp() {
        let environment = root_environment("linux");
        assert!(matches!(
            resolve_cache_root("", &environment),
            Err(CacheRootError::PlatformDefaultUnavailable { .. })
        ));
        let unsupported = root_environment("unknown");
        assert!(matches!(
            resolve_cache_root("", &unsupported),
            Err(CacheRootError::PlatformDefaultUnavailable { .. })
        ));
    }

    #[test]
    fn platform_parent_preparation_leaves_the_owned_leaf_for_store_claim() {
        use fmn_platform::clock::FakeClock;
        use fmn_platform::fs::VirtualFs;

        let fs = Arc::new(VirtualFs::new());
        let root = test_absolute("fresh")
            .join("cache-base")
            .join(DEFAULT_CACHE_LEAF);
        ensure_platform_cache_parent(fs.as_ref(), &root).unwrap();
        assert_eq!(
            fs.node_kind_no_follow(root.parent().unwrap()).unwrap(),
            Some(FsNodeKind::Directory)
        );
        assert_eq!(fs.node_kind_no_follow(&root).unwrap(), None);

        let store = Store::open(
            fs.clone(),
            Arc::new(FakeClock::new()),
            root.clone(),
            StoreConfig::default(),
        )
        .unwrap();
        assert_eq!(store.root(), root);
        assert_eq!(
            fs.node_kind_no_follow(store.root()).unwrap(),
            Some(FsNodeKind::Directory)
        );
    }

    #[test]
    fn host_constructor_resolves_before_requiring_the_host_capability() {
        use fmn_platform::clock::FakeClock;
        use fmn_platform::fs::VirtualFs;

        let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
        let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
        assert!(matches!(
            Store::open_host(
                fs.clone(),
                clock.clone(),
                "../escape",
                StoreConfig::default()
            ),
            Err(CacheError::RootResolution(
                CacheRootError::InvalidConfigured { .. }
            ))
        ));

        let configured = test_absolute("configured-cache");
        match Store::open_host(
            fs,
            clock,
            configured.to_str().unwrap(),
            StoreConfig::default(),
        ) {
            Err(CacheError::RootRefused { root, code, reason }) => {
                assert_eq!(root, configured);
                assert_eq!(code, RootRefusalCode::CapabilityRequired);
                assert!(reason.contains("ambient host filesystem capability"));
            }
            other => panic!("expected host-capability refusal, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn platform_cache_default_preserves_non_utf8_home_bytes() {
        use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};

        let mut environment = root_environment("linux");
        environment.home = Some(OsString::from_vec(b"/home/native-\xff".to_vec()));
        let root = resolve_cache_root("", &environment).unwrap();
        assert_eq!(
            root.as_os_str().as_bytes(),
            b"/home/native-\xff/.cache/franken-manim"
        );
    }

    #[test]
    fn namespace_names_are_strictly_validated() {
        for good in ["typeset", "a", "journal-2", "x_1", "0start"] {
            assert!(validate_namespace_name(good).is_ok(), "{good:?} rejected");
        }
        for bad in [
            "",
            "..",
            ".",
            "a/b",
            "a\\b",
            "A",
            "café",
            "-lead",
            "_lead",
            ".hidden",
            "name.ext",
            "spa ce",
            &"x".repeat(65),
        ] {
            assert!(
                validate_namespace_name(bad).is_err(),
                "{bad:?} unexpectedly accepted"
            );
        }
    }

    #[test]
    fn evict_report_default_is_zeroed() {
        let r = EvictReport::default();
        assert_eq!(
            r.examined + r.evicted + r.skipped_pinned + r.swept_unrecognized,
            0
        );
        assert_eq!(r.evicted_bytes + r.retained_bytes, 0);
    }
}
