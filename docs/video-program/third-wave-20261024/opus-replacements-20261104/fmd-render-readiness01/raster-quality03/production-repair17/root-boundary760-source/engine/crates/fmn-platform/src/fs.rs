//! The filesystem capability: every byte the engine reads or writes flows
//! through [`FileSystem`], so the input closure can record it and the
//! deterministic lab can virtualize it (see the crate-level doctrine).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Process-wide uniquifier for temp-file names: the pid alone is not enough,
/// because two threads of one process writing the same destination would
/// collide on the temp path and race each other's rename.
static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);
/// A collision storm is an I/O failure, not permission to spin forever.
const MAX_TEMP_ATTEMPTS: usize = 4_096;

/// A candidate temp-file sibling name, distinguished across live processes
/// (pid) and within this process (sequence). Callers still open it with
/// `create_new`: PID reuse and crash leftovers must be treated as collisions,
/// never as files that may be truncated.
fn unique_temp_name(prefix: &str, path: &Path) -> String {
    format!(
        "{prefix}.{}.{}.{}",
        std::process::id(),
        TEMP_SEQ.fetch_add(1, Ordering::Relaxed),
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    )
}

fn open_unique_temp(prefix: &str, path: &Path) -> Result<(PathBuf, std::fs::File), FsError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let tmp = parent.join(unique_temp_name(prefix, path));
        let file = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
        {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(io_error(&tmp, err)),
        };
        return Ok((tmp, file));
    }
    Err(io_error(
        path,
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("could not reserve a temporary sibling after {MAX_TEMP_ATTEMPTS} collisions"),
        ),
    ))
}

// On wasm32 std's File is a non-Drop stub, so the explicit close-ordering
// drops below trip clippy::drop_non_drop there; the ordering still matters
// on native (close before remove/rename, Windows above all), so the lint is
// scoped off for the stub target only.
#[cfg_attr(target_arch = "wasm32", allow(clippy::drop_non_drop))]
fn write_unique_temp(prefix: &str, path: &Path, bytes: &[u8]) -> Result<PathBuf, FsError> {
    let (tmp, mut file) = open_unique_temp(prefix, path)?;
    if let Err(err) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(&tmp);
        return Err(io_error(&tmp, err));
    }
    drop(file);
    Ok(tmp)
}

fn create_unique_temp_dir(prefix: &str, path: &Path) -> Result<PathBuf, FsError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let tmp = parent.join(unique_temp_name(prefix, path));
        match std::fs::create_dir(&tmp) {
            Ok(()) => return Ok(tmp),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(io_error(&tmp, err)),
        }
    }
    Err(io_error(
        path,
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("could not reserve a temporary directory after {MAX_TEMP_ATTEMPTS} collisions"),
        ),
    ))
}

/// A filesystem failure, carrying the path it happened at.
#[derive(Debug)]
pub enum FsError {
    /// The path does not exist (or a parent directory is missing).
    NotFound {
        /// The missing path.
        path: PathBuf,
    },
    /// The bytes at `path` were expected to be UTF-8 and are not.
    NotUtf8 {
        /// The offending path.
        path: PathBuf,
    },
    /// The file exceeded a caller-supplied byte limit.
    TooLarge {
        /// The offending path.
        path: PathBuf,
        /// Maximum admitted file size in bytes.
        limit: usize,
    },
    /// A directory exceeded a caller-supplied direct-entry limit.
    TooManyEntries {
        /// The offending directory.
        path: PathBuf,
        /// Maximum admitted number of direct entries.
        limit: usize,
    },
    /// Any other I/O failure.
    Io {
        /// The path being accessed.
        path: PathBuf,
        /// The underlying error.
        err: std::io::Error,
    },
}

/// The kind of node found by [`FileSystem::node_kind_no_follow`].
///
/// `Link` includes symbolic links and, on Windows, every reparse point. A
/// caller that is establishing a traversal boundary must reject `Link` and
/// `Other` rather than asking a later path-based operation to interpret them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsNodeKind {
    /// A regular file.
    RegularFile,
    /// A real directory.
    Directory,
    /// A symbolic link or other platform link-like node.
    Link,
    /// A device, socket, FIFO, or another non-file/non-directory node.
    Other,
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { path } => write!(f, "not found: {}", path.display()),
            Self::NotUtf8 { path } => write!(f, "not UTF-8: {}", path.display()),
            Self::TooLarge { path, limit } => write!(
                f,
                "file at {} exceeds the {limit}-byte limit",
                path.display()
            ),
            Self::TooManyEntries { path, limit } => write!(
                f,
                "directory at {} exceeds the {limit}-entry limit",
                path.display()
            ),
            Self::Io { path, err } => write!(f, "I/O failure at {}: {err}", path.display()),
        }
    }
}

enum BoundedRead {
    Complete(Vec<u8>),
    LimitExceeded,
}

/// Read no more than `max_bytes` payload bytes plus one refusal sentinel.
fn read_stream_bounded(
    reader: &mut impl std::io::Read,
    max_bytes: usize,
) -> std::io::Result<BoundedRead> {
    const CHUNK_BYTES: usize = 8 * 1024;

    let mut bytes = Vec::with_capacity(max_bytes.min(CHUNK_BYTES));
    let mut chunk = [0_u8; CHUNK_BYTES];
    loop {
        let room = max_bytes - bytes.len();
        let target = if room == 0 {
            &mut chunk[..1]
        } else {
            &mut chunk[..room.min(CHUNK_BYTES)]
        };
        let read = match reader.read(target) {
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if read == 0 {
            return Ok(BoundedRead::Complete(bytes));
        }
        if room == 0 {
            return Ok(BoundedRead::LimitExceeded);
        }
        bytes.extend_from_slice(&target[..read]);
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { err, .. } => Some(err),
            _ => None,
        }
    }
}

/// A private, incrementally written file that is not yet visible at its
/// destination.
///
/// Dropping an unprepared writer aborts it. [`Self::prepare`] flushes and
/// syncs the private bytes but still does not mutate the destination.
pub trait AtomicFileWriter: Send {
    /// Append one bounded chunk.
    ///
    /// # Errors
    /// [`FsError`] when the private file cannot accept the bytes.
    fn write(&mut self, bytes: &[u8]) -> Result<(), FsError>;

    /// Flush and sync the private file, yielding the sole publication token.
    ///
    /// # Errors
    /// [`FsError`] when durable preparation fails.
    fn prepare(self: Box<Self>) -> Result<Box<dyn PreparedAtomicFile>, FsError>;
}

/// A fully written private file whose destination remains untouched.
///
/// Dropping this value aborts publication. [`Self::commit`] is the only
/// operation that can replace the destination.
pub trait PreparedAtomicFile: Send {
    /// Atomically publish the prepared file.
    ///
    /// # Errors
    /// [`FsError`] when publication fails.
    fn commit(self: Box<Self>) -> Result<(), FsError>;

    /// Publish only if no node exists at the destination, including a dangling
    /// symlink. The absence check and publication must be one atomic operation;
    /// a preflight `exists` followed by [`Self::commit`] is not sufficient.
    ///
    /// The default refuses rather than weakening a host's publication policy.
    /// Dropping the refused token must still discard its private staging file.
    ///
    /// # Errors
    /// [`FsError::Io`] with `AlreadyExists` for an occupied destination, or
    /// `Unsupported` when this capability cannot guarantee create-only output.
    fn commit_new(self: Box<Self>) -> Result<(), FsError> {
        Err(io_error(
            Path::new("<prepared-atomic-file>"),
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "filesystem capability does not support create-only atomic publication",
            ),
        ))
    }
}

/// Reserved last-published marker for immutable directory generations.
///
/// A directory is a complete artifact only when this regular file exists.
/// Implementations stage every child first and publish this marker last.
pub const ATOMIC_DIRECTORY_COMPLETE_LEAF: &str = "FMN_COMPLETE";

const ATOMIC_DIRECTORY_COMPLETE_BYTES: &[u8] = b"fmn-atomic-directory-v1\n";

/// A private immutable directory generation.
///
/// Children are conservative cross-platform ASCII leaves created exactly
/// once: alphanumeric first byte, then alphanumeric/`_`/`-`/`.`, with Windows
/// device names and trailing dot/space refused. Dropping the writer removes
/// only its private generation.
pub trait AtomicDirectoryWriter: Send {
    /// Write one complete child file under `leaf`.
    ///
    /// # Errors
    /// [`FsError`] for an unsafe/duplicate leaf or an I/O failure.
    fn write_file(&mut self, leaf: &Path, bytes: &[u8]) -> Result<(), FsError>;

    /// Finish the generation without making it visible.
    ///
    /// # Errors
    /// [`FsError`] when preparation fails.
    fn prepare(self: Box<Self>) -> Result<Box<dyn PreparedAtomicDirectory>, FsError>;
}

/// A complete private directory generation awaiting no-clobber publication.
///
/// The destination must be absent. Host implementations claim it with an
/// exact directory create, move prepared children, and atomically publish
/// [`ATOMIC_DIRECTORY_COMPLETE_LEAF`] last. Concurrent compliant publishers
/// therefore fail closed, and a crash can leave only an explicitly incomplete
/// directory that readers must ignore.
pub trait PreparedAtomicDirectory: Send {
    /// Publish the directory as one immutable generation.
    ///
    /// # Errors
    /// [`FsError`] when the destination exists or publication fails.
    fn commit(self: Box<Self>) -> Result<(), FsError>;
}

/// The filesystem capability. Implementations must be deterministic in
/// listing order ([`FileSystem::list_dir`] returns sorted paths) so no
/// consumer inherits host directory-iteration order.
pub trait FileSystem: Send + Sync {
    /// Stable implementation identity recorded by C9 provenance.
    ///
    /// Custom hosts should override the fail-closed default with a versioned
    /// policy name; pointer/type-debug identities are not durable inputs.
    fn identity(&self) -> &'static str {
        "opaque.file_system/v1"
    }

    /// Whether this capability explicitly represents the process's ambient
    /// host filesystem for destructive lifecycle operations.
    ///
    /// The default is fail-closed. Virtual, recording, read-only, and test
    /// implementations must not opt in merely because they delegate some
    /// reads to the host.
    fn grants_host_destructive_lifecycle(&self) -> bool {
        false
    }

    /// Classify the node at `path` without following the final component.
    ///
    /// Intermediate path components are still interpreted by the host path
    /// resolver. Consumers enforcing a no-follow traversal boundary must
    /// inspect each component in order, immediately before the operation it
    /// guards.
    ///
    /// `Ok(None)` means the node is absent. On Windows, every reparse point
    /// is classified as [`FsNodeKind::Link`], not only symbolic links.
    ///
    /// # Errors
    /// [`FsError::Io`] for failures other than absence.
    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError>;

    /// Create exactly the directory named by `path`, without creating parent
    /// directories. Returns `Ok(true)` when this call created it and
    /// `Ok(false)` when a real directory already existed.
    ///
    /// An existing link or any other node kind is an error. Callers that need
    /// a directory chain must walk it one exact leaf at a time, classifying
    /// each winner with [`FileSystem::node_kind_no_follow`].
    ///
    /// # Errors
    /// [`FsError::NotFound`] if the parent is absent, or [`FsError::Io`].
    fn create_dir(&self, path: &Path) -> Result<bool, FsError>;

    /// Read the full contents of a file.
    ///
    /// # Errors
    /// [`FsError::NotFound`] or [`FsError::Io`].
    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError>;

    /// Read a regular file subject to an allocation-enforced byte limit.
    ///
    /// Implementations must accept a file of exactly `max_bytes`, refuse a
    /// larger file with [`FsError::TooLarge`], and must not allocate or copy
    /// the complete file before enforcing the limit.
    ///
    /// # Errors
    /// [`FsError::NotFound`], [`FsError::TooLarge`], or [`FsError::Io`].
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError>;

    /// Write `bytes` to `path` atomically: the destination either keeps its
    /// old contents or holds exactly `bytes`, never a torn intermediate.
    /// Parent directories are created as needed.
    ///
    /// # Errors
    /// [`FsError::Io`].
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError>;

    /// Begin an incrementally written atomic file.
    ///
    /// Production implementations stream to a unique sibling temporary file.
    /// The default refuses the operation so read-only/test capabilities cannot
    /// accidentally claim bounded host publication.
    ///
    /// # Errors
    /// [`FsError`] when unsupported or when private-file creation fails.
    fn begin_atomic_file(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicFileWriter>, FsError> {
        Err(io_error(
            path,
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "filesystem capability does not support streaming atomic files",
            ),
        ))
    }

    /// Begin an immutable, no-clobber atomic directory generation.
    ///
    /// # Errors
    /// [`FsError`] when unsupported or when private-directory creation fails.
    fn begin_atomic_directory(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicDirectoryWriter>, FsError> {
        Err(io_error(
            path,
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "filesystem capability does not support atomic directory generations",
            ),
        ))
    }

    /// Create `path` with `bytes` **only if nothing exists there** — the
    /// lock-file primitive. Returns `Ok(true)` if this call created the file,
    /// `Ok(false)` if something already existed (no mutation). The created
    /// file appears with its full contents (never empty-then-filled), so a
    /// concurrent reader sees either absence or the complete bytes. Parent
    /// directories are created as needed.
    ///
    /// # Errors
    /// [`FsError::Io`].
    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError>;

    /// Remove the file at `path`. Exists for *defined* lifecycle operations —
    /// cache eviction, stale-lock breaking, `--clear-cache` — never for
    /// ad-hoc cleanup; every deletion a consumer performs must be part of a
    /// specified policy.
    ///
    /// # Errors
    /// [`FsError::NotFound`] if there is no file, [`FsError::Io`] otherwise.
    fn remove_file(&self, path: &Path) -> Result<(), FsError>;

    /// Remove the directory at `path` and everything under it. Same doctrine
    /// as [`remove_file`](Self::remove_file): defined lifecycle operations
    /// only (namespace-version purges, `--clear-cache`).
    ///
    /// Implementations must remove link-like children as nodes and must never
    /// recurse through their targets.
    ///
    /// # Errors
    /// [`FsError::NotFound`] if the path does not exist, [`FsError::Io`]
    /// otherwise.
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError>;

    /// Whether a file exists at `path`.
    fn exists(&self, path: &Path) -> bool;

    /// The entries directly under `path`, sorted byte-lexicographically.
    ///
    /// # Errors
    /// [`FsError::NotFound`] or [`FsError::Io`].
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError>;

    /// Count entries directly under `path` subject to a hard work limit.
    ///
    /// Implementations must accept exactly `max_entries`, inspect at most one
    /// additional entry to detect overflow, and must not collect the complete
    /// directory before enforcing the limit.
    ///
    /// # Errors
    /// [`FsError::NotFound`], [`FsError::TooManyEntries`], or [`FsError::Io`].
    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError>;

    /// Read a file and decode it as UTF-8.
    ///
    /// # Errors
    /// [`FsError`] from the read, or [`FsError::NotUtf8`].
    fn read_to_string(&self, path: &Path) -> Result<String, FsError> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|_| FsError::NotUtf8 {
            path: path.to_path_buf(),
        })
    }

    /// Read a byte-limited file and decode it as UTF-8.
    ///
    /// The size limit is enforced before UTF-8 decoding.
    ///
    /// # Errors
    /// [`FsError`] from the bounded read, or [`FsError::NotUtf8`].
    fn read_to_string_bounded(&self, path: &Path, max_bytes: usize) -> Result<String, FsError> {
        let bytes = self.read_bounded(path, max_bytes)?;
        String::from_utf8(bytes).map_err(|_| FsError::NotUtf8 {
            path: path.to_path_buf(),
        })
    }
}

/// The host filesystem, via `std::fs`. The engine's production capability.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdFs;

impl FileSystem for StdFs {
    fn identity(&self) -> &'static str {
        "std.host_file_system/v1"
    }

    fn grants_host_destructive_lifecycle(&self) -> bool {
        true
    }

    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError> {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) => Ok(Some(metadata_node_kind(&metadata))),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(io_error(path, err)),
        }
    }

    fn create_dir(&self, path: &Path) -> Result<bool, FsError> {
        match std::fs::create_dir(path) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                match self.node_kind_no_follow(path)? {
                    Some(FsNodeKind::Directory) => Ok(false),
                    Some(kind) => Err(io_error(
                        path,
                        std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            format!("expected a directory, found {kind:?}"),
                        ),
                    )),
                    None => Err(io_error(
                        path,
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "directory disappeared after create collision",
                        ),
                    )),
                }
            }
            Err(err) => Err(io_error(path, err)),
        }
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        std::fs::read(path).map_err(|err| io_error(path, err))
    }

    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        // Reject ordinary non-file nodes before opening them. The opened
        // handle is classified again so a completed replacement race cannot
        // turn a bounded read into an unbounded special-node read.
        let metadata = std::fs::metadata(path).map_err(|error| io_error(path, error))?;
        if !metadata.is_file() {
            return Err(io_error(
                path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "bounded reads require a regular file",
                ),
            ));
        }
        if metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
            return Err(FsError::TooLarge {
                path: path.to_path_buf(),
                limit: max_bytes,
            });
        }

        let mut file = std::fs::File::open(path).map_err(|error| io_error(path, error))?;
        let opened_metadata = file.metadata().map_err(|error| io_error(path, error))?;
        if !opened_metadata.is_file() {
            return Err(io_error(
                path,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "bounded reads require a regular file",
                ),
            ));
        }
        match read_stream_bounded(&mut file, max_bytes).map_err(|error| io_error(path, error))? {
            BoundedRead::Complete(bytes) => Ok(bytes),
            BoundedRead::LimitExceeded => Err(FsError::TooLarge {
                path: path.to_path_buf(),
                limit: max_bytes,
            }),
        }
    }

    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        // Unique sibling temp name, then rename into place.
        let tmp = write_unique_temp(".fmn-tmp", path, bytes)?;
        match std::fs::rename(&tmp, path) {
            Ok(()) => Ok(()),
            Err(err) => {
                let _ = std::fs::remove_file(&tmp);
                Err(io_error(path, err))
            }
        }
    }

    fn begin_atomic_file(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicFileWriter>, FsError> {
        let (temporary, file) = open_unique_temp(".fmn-stream", path)?;
        Ok(Box::new(StdAtomicFileWriter {
            destination: path.to_path_buf(),
            temporary,
            file: Some(file),
            cleanup: true,
        }))
    }

    fn begin_atomic_directory(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicDirectoryWriter>, FsError> {
        let temporary = create_unique_temp_dir(".fmn-generation", path)?;
        Ok(Box::new(StdAtomicDirectoryWriter {
            destination: path.to_path_buf(),
            temporary,
            cleanup: true,
        }))
    }

    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError> {
        // Write the contents to a unique sibling, then `hard_link` it into
        // place: link creation is atomic and fails if the destination exists,
        // so the file appears fully written or not at all — the lock-file
        // guarantee. A plain `File::create_new` + write would expose an
        // empty-then-filled window to concurrent readers.
        let tmp = write_unique_temp(".fmn-new", path, bytes)?;
        let linked = match std::fs::hard_link(&tmp, path) {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(err) => Err(io_error(path, err)),
        };
        // Best-effort: the unique temp is invisible to consumers either way.
        let _ = std::fs::remove_file(&tmp);
        linked
    }

    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        std::fs::remove_file(path).map_err(|err| io_error(path, err))
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        std::fs::remove_dir_all(path).map_err(|err| io_error(path, err))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|err| io_error(path, err))? {
            out.push(entry.map_err(|err| io_error(path, err))?.path());
        }
        out.sort();
        Ok(out)
    }

    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError> {
        let mut count = 0_usize;
        for entry in std::fs::read_dir(path).map_err(|error| io_error(path, error))? {
            entry.map_err(|error| io_error(path, error))?;
            if count == max_entries {
                return Err(FsError::TooManyEntries {
                    path: path.to_path_buf(),
                    limit: max_entries,
                });
            }
            count += 1;
        }
        Ok(count)
    }
}

struct StdAtomicFileWriter {
    destination: PathBuf,
    temporary: PathBuf,
    file: Option<std::fs::File>,
    cleanup: bool,
}

impl AtomicFileWriter for StdAtomicFileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), FsError> {
        self.file
            .as_mut()
            .ok_or_else(|| {
                io_error(
                    &self.temporary,
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "atomic file is already prepared",
                    ),
                )
            })?
            .write_all(bytes)
            .map_err(|error| io_error(&self.temporary, error))
    }

    // wasm32: see write_unique_temp for the scoped drop_non_drop allow.
    #[cfg_attr(target_arch = "wasm32", allow(clippy::drop_non_drop))]
    fn prepare(mut self: Box<Self>) -> Result<Box<dyn PreparedAtomicFile>, FsError> {
        let file = self.file.take().ok_or_else(|| {
            io_error(
                &self.temporary,
                std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "atomic file is already prepared",
                ),
            )
        })?;
        file.sync_all()
            .map_err(|error| io_error(&self.temporary, error))?;
        drop(file);
        self.cleanup = false;
        Ok(Box::new(StdPreparedAtomicFile {
            destination: self.destination.clone(),
            temporary: self.temporary.clone(),
            cleanup: true,
        }))
    }
}

impl Drop for StdAtomicFileWriter {
    fn drop(&mut self) {
        if self.cleanup {
            self.file.take();
            let _ = std::fs::remove_file(&self.temporary);
        }
    }
}

struct StdPreparedAtomicFile {
    destination: PathBuf,
    temporary: PathBuf,
    cleanup: bool,
}

impl PreparedAtomicFile for StdPreparedAtomicFile {
    fn commit(mut self: Box<Self>) -> Result<(), FsError> {
        std::fs::rename(&self.temporary, &self.destination)
            .map_err(|error| io_error(&self.destination, error))?;
        self.cleanup = false;
        Ok(())
    }

    fn commit_new(self: Box<Self>) -> Result<(), FsError> {
        // Both names are siblings on the same filesystem. Link creation is
        // atomic and cannot replace any existing node. Bytes were synced by
        // prepare(); Drop removes only our private name, on success or error.
        // Filesystems without hard links fail closed, never fall back to rename.
        std::fs::hard_link(&self.temporary, &self.destination)
            .map_err(|error| io_error(&self.destination, error))
    }
}

impl Drop for StdPreparedAtomicFile {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_file(&self.temporary);
        }
    }
}

struct StdAtomicDirectoryWriter {
    destination: PathBuf,
    temporary: PathBuf,
    cleanup: bool,
}

impl AtomicDirectoryWriter for StdAtomicDirectoryWriter {
    fn write_file(&mut self, leaf: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if !is_safe_relative_leaf(leaf) || leaf == Path::new(ATOMIC_DIRECTORY_COMPLETE_LEAF) {
            return Err(io_error(
                leaf,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "atomic-directory child must be one normal relative component",
                ),
            ));
        }
        let path = self.temporary.join(leaf);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| io_error(&path, error))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| io_error(&path, error))
    }

    fn prepare(mut self: Box<Self>) -> Result<Box<dyn PreparedAtomicDirectory>, FsError> {
        let marker = self.temporary.join(ATOMIC_DIRECTORY_COMPLETE_LEAF);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&marker)
            .map_err(|error| io_error(&marker, error))?;
        file.write_all(ATOMIC_DIRECTORY_COMPLETE_BYTES)
            .and_then(|()| file.sync_all())
            .map_err(|error| io_error(&marker, error))?;
        self.cleanup = false;
        Ok(Box::new(StdPreparedAtomicDirectory {
            destination: self.destination.clone(),
            temporary: self.temporary.clone(),
            cleanup: true,
        }))
    }
}

impl Drop for StdAtomicDirectoryWriter {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_dir_all(&self.temporary);
        }
    }
}

struct StdPreparedAtomicDirectory {
    destination: PathBuf,
    temporary: PathBuf,
    cleanup: bool,
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), FsError> {
    std::fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| io_error(path, error))
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> Result<(), FsError> {
    use std::os::windows::fs::OpenOptionsExt as _;

    // Directory handles need backup semantics to open at all, and
    // `sync_all` is `FlushFileBuffers`, which refuses a read-only handle
    // with ERROR_ACCESS_DENIED — the handle must carry GENERIC_WRITE even
    // though nothing is written through it.
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| io_error(path, error))
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(path: &Path) -> Result<(), FsError> {
    Err(io_error(
        path,
        std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "directory durability sync is unavailable on this target",
        ),
    ))
}

impl PreparedAtomicDirectory for StdPreparedAtomicDirectory {
    fn commit(mut self: Box<Self>) -> Result<(), FsError> {
        match std::fs::create_dir(&self.destination) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(io_error(
                    &self.destination,
                    std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "atomic directory destination already exists",
                    ),
                ));
            }
            Err(error) => return Err(io_error(&self.destination, error)),
        }
        if let Some(parent) = nonempty_parent(&self.destination) {
            sync_directory(parent)?;
        }

        let mut entries = std::fs::read_dir(&self.temporary)
            .map_err(|error| io_error(&self.temporary, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| io_error(&self.temporary, error))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        let marker = self.temporary.join(ATOMIC_DIRECTORY_COMPLETE_LEAF);
        for entry in entries {
            let source = entry.path();
            if source == marker {
                continue;
            }
            if !entry
                .file_type()
                .map_err(|error| io_error(&source, error))?
                .is_file()
            {
                return Err(io_error(
                    &source,
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "atomic directory contains a non-file child",
                    ),
                ));
            }
            let destination = self.destination.join(entry.file_name());
            std::fs::rename(&source, &destination)
                .map_err(|error| io_error(&destination, error))?;
        }

        // Persist every child directory entry before the completion marker can
        // become visible. A successful marker sync is therefore also a
        // durable ordering boundary for the entire generation.
        sync_directory(&self.destination)?;
        let published_marker = self.destination.join(ATOMIC_DIRECTORY_COMPLETE_LEAF);
        std::fs::rename(&marker, &published_marker)
            .map_err(|error| io_error(&published_marker, error))?;
        sync_directory(&self.destination)?;
        let _ = std::fs::remove_dir(&self.temporary);
        self.cleanup = false;
        Ok(())
    }
}

impl Drop for StdPreparedAtomicDirectory {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_dir_all(&self.temporary);
        }
    }
}

fn is_safe_relative_leaf(path: &Path) -> bool {
    let mut components = path.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return false;
    }
    let Some(leaf) = path.to_str() else {
        return false;
    };
    let mut bytes = leaf.bytes();
    let first_is_safe = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric());
    let rest_is_safe =
        bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if !first_is_safe || !rest_is_safe || leaf.ends_with('.') || leaf.ends_with(' ') {
        return false;
    }
    let basename = leaf
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !is_windows_device_name(&basename)
}

fn is_windows_device_name(name: &str) -> bool {
    matches!(name, "CON" | "PRN" | "AUX" | "NUL")
        || name.strip_prefix("COM").is_some_and(|number| {
            matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
        || name.strip_prefix("LPT").is_some_and(|number| {
            matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
}

fn metadata_node_kind(metadata: &std::fs::Metadata) -> FsNodeKind {
    if metadata_is_link_like(metadata) {
        FsNodeKind::Link
    } else if metadata.is_file() {
        FsNodeKind::RegularFile
    } else if metadata.is_dir() {
        FsNodeKind::Directory
    } else {
        FsNodeKind::Other
    }
}

#[cfg(windows)]
fn metadata_is_link_like(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt as _;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn metadata_is_link_like(metadata: &std::fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn io_error(path: &Path, err: std::io::Error) -> FsError {
    if err.kind() == std::io::ErrorKind::NotFound {
        FsError::NotFound {
            path: path.to_path_buf(),
        }
    } else {
        FsError::Io {
            path: path.to_path_buf(),
            err,
        }
    }
}

/// The in-memory test double: a `path → bytes` regular-file map plus explicit
/// directories. Deterministic by construction (`BTreeMap`/`BTreeSet`
/// ordering); shared mutability behind one `RwLock` so compound filesystem
/// operations remain atomic and a populated instance can be handed to
/// consumers as `&dyn FileSystem`.
#[derive(Debug, Default)]
struct VirtualFsState {
    files: BTreeMap<PathBuf, Vec<u8>>,
    directories: BTreeSet<PathBuf>,
}

/// The in-memory filesystem: a `path → bytes` regular-file map plus explicit
/// directories. Deterministic by construction (`BTreeMap`/`BTreeSet`
/// ordering); shared mutability behind one `RwLock` so compound filesystem
/// operations remain atomic and a populated instance can be handed to
/// consumers as `&dyn FileSystem`.
///
/// Two consumers: the deterministic lab (journaled tests substitute it for
/// the host fs) and the W5 wasm tier-1 build (fm-l97), where it IS the
/// filesystem capability — the browser sandbox has no host fs, assets arrive
/// as bytes (fetch/upload/OPFS at the host boundary), and the engine's reads
/// and writes all land here.
#[derive(Debug, Default)]
pub struct VirtualFs {
    state: RwLock<VirtualFsState>,
}

impl VirtualFs {
    /// An empty virtual filesystem.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace) a file.
    ///
    /// The latest fixture insertion wins while preserving a hierarchy the
    /// host filesystem could represent: file ancestors become directories,
    /// and descendants of a path that becomes a file are discarded.
    pub fn insert(&self, path: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) {
        let path = path.into();
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .files
            .retain(|candidate, _| !candidate.starts_with(&path));
        state
            .directories
            .retain(|candidate| !candidate.starts_with(&path));
        if let Some(parent) = nonempty_parent(&path) {
            for ancestor in parent.ancestors() {
                state.files.remove(ancestor);
            }
        }
        insert_parent_directories(&mut state, &path);
        state.files.insert(path, bytes.into());
    }

    /// Load a `path<TAB>contents` manifest (one file per line, `\n` in
    /// contents escaped as `\\n`) — the format the committed synthetic
    /// sysfs fixtures use.
    pub fn load_manifest(&self, manifest: &str) {
        for line in manifest.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((path, contents)) = line.split_once('\t') {
                self.insert(path, contents.replace("\\n", "\n").into_bytes());
            }
        }
    }

    fn with_state<T>(&self, f: impl FnOnce(&VirtualFsState) -> T) -> T {
        f(&self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner))
    }
}

impl FileSystem for VirtualFs {
    fn identity(&self) -> &'static str {
        "fmn.virtual_file_system/v1"
    }

    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError> {
        self.with_state(|state| {
            if let Some(kind) = virtual_node_kind(state, path) {
                return Ok(Some(kind));
            }
            if let Some(blocker) = virtual_file_ancestor(state, path) {
                return Err(io_error(
                    &blocker,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "a parent component is a file",
                    ),
                ));
            }
            Ok(None)
        })
    }

    fn create_dir(&self, path: &Path) -> Result<bool, FsError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if virtual_node_kind(&state, path) == Some(FsNodeKind::Directory) {
            return Ok(false);
        }
        if state.files.contains_key(path) {
            return Err(io_error(
                path,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "a file already occupies the directory path",
                ),
            ));
        }
        if let Some(blocker) = virtual_file_ancestor(&state, path) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        if let Some(parent) = nonempty_parent(path)
            && virtual_node_kind(&state, parent) != Some(FsNodeKind::Directory)
        {
            return Err(FsError::NotFound {
                path: parent.to_path_buf(),
            });
        }
        state.directories.insert(path.to_path_buf());
        Ok(true)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.read_bounded(path, usize::MAX)
    }

    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        self.with_state(|state| {
            if let Some(blocker) = virtual_file_ancestor(state, path) {
                return Err(io_error(
                    &blocker,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "a parent component is a file",
                    ),
                ));
            }
            if let Some(bytes) = state.files.get(path) {
                if bytes.len() > max_bytes {
                    Err(FsError::TooLarge {
                        path: path.to_path_buf(),
                        limit: max_bytes,
                    })
                } else {
                    Ok(bytes.clone())
                }
            } else if virtual_node_kind(state, path) == Some(FsNodeKind::Directory) {
                Err(io_error(
                    path,
                    std::io::Error::new(std::io::ErrorKind::IsADirectory, "path is a directory"),
                ))
            } else {
                Err(FsError::NotFound {
                    path: path.to_path_buf(),
                })
            }
        })
    }

    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if virtual_node_kind(&state, path) == Some(FsNodeKind::Directory) {
            return Err(io_error(
                path,
                std::io::Error::new(std::io::ErrorKind::IsADirectory, "path is a directory"),
            ));
        }
        if let Some(blocker) = virtual_file_ancestor(&state, path) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        insert_parent_directories(&mut state, path);
        state.files.insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn begin_atomic_file(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicFileWriter>, FsError> {
        Ok(Box::new(VirtualAtomicFileWriter {
            fs: self,
            destination: path.to_path_buf(),
            bytes: Vec::new(),
        }))
    }

    fn begin_atomic_directory(
        self: Arc<Self>,
        path: &Path,
    ) -> Result<Box<dyn AtomicDirectoryWriter>, FsError> {
        Ok(Box::new(VirtualAtomicDirectoryWriter {
            fs: self,
            destination: path.to_path_buf(),
            files: BTreeMap::new(),
        }))
    }

    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError> {
        let mut files = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // One write-lock hold makes check-and-insert atomic, matching the
        // host implementation's create-if-absent guarantee.
        if virtual_node_kind(&files, path).is_some() {
            return Ok(false);
        }
        if let Some(blocker) = virtual_file_ancestor(&files, path) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        insert_parent_directories(&mut files, path);
        files.files.insert(path.to_path_buf(), bytes.to_vec());
        Ok(true)
    }

    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(blocker) = virtual_file_ancestor(&state, path) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        if state.files.remove(path).is_some() {
            Ok(())
        } else if virtual_node_kind(&state, path) == Some(FsNodeKind::Directory) {
            Err(io_error(
                path,
                std::io::Error::new(std::io::ErrorKind::IsADirectory, "path is a directory"),
            ))
        } else {
            Err(FsError::NotFound {
                path: path.to_path_buf(),
            })
        }
    }

    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(blocker) = virtual_file_ancestor(&state, path) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        match virtual_node_kind(&state, path) {
            Some(FsNodeKind::Directory) => {}
            Some(_) => {
                return Err(io_error(
                    path,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "path is not a directory",
                    ),
                ));
            }
            None => {
                return Err(FsError::NotFound {
                    path: path.to_path_buf(),
                });
            }
        }
        state
            .files
            .retain(|candidate, _| !candidate.starts_with(path));
        state
            .directories
            .retain(|candidate| !candidate.starts_with(path));
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.with_state(|state| virtual_node_kind(state, path).is_some())
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.with_state(|state| {
            if let Some(blocker) = virtual_file_ancestor(state, path) {
                return Err(io_error(
                    &blocker,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "a parent component is a file",
                    ),
                ));
            }
            match virtual_node_kind(state, path) {
                Some(FsNodeKind::Directory) => {}
                Some(_) => {
                    return Err(io_error(
                        path,
                        std::io::Error::new(
                            std::io::ErrorKind::NotADirectory,
                            "path is not a directory",
                        ),
                    ));
                }
                None => {
                    return Err(FsError::NotFound {
                        path: path.to_path_buf(),
                    });
                }
            }
            let mut out = BTreeSet::new();
            for candidate in state.files.keys().chain(state.directories.iter()) {
                if candidate == path {
                    continue;
                }
                if let Ok(rest) = candidate.strip_prefix(path)
                    && let Some(first) = rest.components().next()
                {
                    out.insert(path.join(first));
                }
            }
            Ok(out.into_iter().collect())
        })
    }

    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError> {
        self.with_state(|state| {
            if let Some(blocker) = virtual_file_ancestor(state, path) {
                return Err(io_error(
                    &blocker,
                    std::io::Error::new(
                        std::io::ErrorKind::NotADirectory,
                        "a parent component is a file",
                    ),
                ));
            }
            match virtual_node_kind(state, path) {
                Some(FsNodeKind::Directory) => {}
                Some(_) => {
                    return Err(io_error(
                        path,
                        std::io::Error::new(
                            std::io::ErrorKind::NotADirectory,
                            "path is not a directory",
                        ),
                    ));
                }
                None => {
                    return Err(FsError::NotFound {
                        path: path.to_path_buf(),
                    });
                }
            }
            let mut entries = BTreeSet::new();
            for candidate in state.files.keys().chain(state.directories.iter()) {
                if candidate == path {
                    continue;
                }
                if let Ok(rest) = candidate.strip_prefix(path)
                    && let Some(first) = rest.components().next()
                    && entries.insert(path.join(first))
                    && entries.len() > max_entries
                {
                    return Err(FsError::TooManyEntries {
                        path: path.to_path_buf(),
                        limit: max_entries,
                    });
                }
            }
            Ok(entries.len())
        })
    }
}

struct VirtualAtomicFileWriter {
    fs: Arc<VirtualFs>,
    destination: PathBuf,
    bytes: Vec<u8>,
}

impl AtomicFileWriter for VirtualAtomicFileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), FsError> {
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn prepare(self: Box<Self>) -> Result<Box<dyn PreparedAtomicFile>, FsError> {
        let Self {
            fs,
            destination,
            bytes,
        } = *self;
        Ok(Box::new(VirtualPreparedAtomicFile {
            fs,
            destination,
            bytes,
        }))
    }
}

struct VirtualPreparedAtomicFile {
    fs: Arc<VirtualFs>,
    destination: PathBuf,
    bytes: Vec<u8>,
}

impl PreparedAtomicFile for VirtualPreparedAtomicFile {
    fn commit(self: Box<Self>) -> Result<(), FsError> {
        self.fs.write_atomic(&self.destination, &self.bytes)
    }

    fn commit_new(self: Box<Self>) -> Result<(), FsError> {
        if self.fs.create_new(&self.destination, &self.bytes)? {
            Ok(())
        } else {
            Err(io_error(
                &self.destination,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "atomic file destination already exists",
                ),
            ))
        }
    }
}

struct VirtualAtomicDirectoryWriter {
    fs: Arc<VirtualFs>,
    destination: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
}

impl AtomicDirectoryWriter for VirtualAtomicDirectoryWriter {
    fn write_file(&mut self, leaf: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if !is_safe_relative_leaf(leaf) || leaf == Path::new(ATOMIC_DIRECTORY_COMPLETE_LEAF) {
            return Err(io_error(
                leaf,
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "atomic-directory child must be one normal relative component",
                ),
            ));
        }
        if self.files.contains_key(leaf) {
            return Err(io_error(
                leaf,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "atomic-directory child already exists",
                ),
            ));
        }
        self.files.insert(leaf.to_path_buf(), bytes.to_vec());
        Ok(())
    }

    fn prepare(self: Box<Self>) -> Result<Box<dyn PreparedAtomicDirectory>, FsError> {
        let Self {
            fs,
            destination,
            mut files,
        } = *self;
        files.insert(
            PathBuf::from(ATOMIC_DIRECTORY_COMPLETE_LEAF),
            ATOMIC_DIRECTORY_COMPLETE_BYTES.to_vec(),
        );
        Ok(Box::new(VirtualPreparedAtomicDirectory {
            fs,
            destination,
            files,
        }))
    }
}

struct VirtualPreparedAtomicDirectory {
    fs: Arc<VirtualFs>,
    destination: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
}

impl PreparedAtomicDirectory for VirtualPreparedAtomicDirectory {
    fn commit(self: Box<Self>) -> Result<(), FsError> {
        let mut state = self
            .fs
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if virtual_node_kind(&state, &self.destination).is_some() {
            return Err(io_error(
                &self.destination,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "atomic directory destination already exists",
                ),
            ));
        }
        if let Some(blocker) = virtual_file_ancestor(&state, &self.destination) {
            return Err(io_error(
                &blocker,
                std::io::Error::new(
                    std::io::ErrorKind::NotADirectory,
                    "a parent component is a file",
                ),
            ));
        }
        insert_parent_directories(&mut state, &self.destination);
        state.directories.insert(self.destination.clone());
        for (leaf, bytes) in &self.files {
            state
                .files
                .insert(self.destination.join(leaf), bytes.clone());
        }
        Ok(())
    }
}

fn nonempty_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn is_virtual_root(path: &Path) -> bool {
    path.has_root() && path.parent().is_none()
}

fn virtual_node_kind(state: &VirtualFsState, path: &Path) -> Option<FsNodeKind> {
    if state.files.contains_key(path) {
        Some(FsNodeKind::RegularFile)
    } else if is_virtual_root(path) || state.directories.contains(path) {
        Some(FsNodeKind::Directory)
    } else {
        None
    }
}

fn virtual_file_ancestor(state: &VirtualFsState, path: &Path) -> Option<PathBuf> {
    nonempty_parent(path)?
        .ancestors()
        .find(|ancestor| state.files.contains_key(*ancestor))
        .map(Path::to_path_buf)
}

fn insert_parent_directories(state: &mut VirtualFsState, path: &Path) {
    let Some(parent) = nonempty_parent(path) else {
        return;
    };
    for ancestor in parent.ancestors() {
        if !ancestor.as_os_str().is_empty() && !is_virtual_root(ancestor) {
            state.directories.insert(ancestor.to_path_buf());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_fs_read_write_list() {
        let fs = VirtualFs::new();
        fs.insert("/a/b/one.txt", b"1".to_vec());
        fs.insert("/a/b/two.txt", b"2".to_vec());
        fs.insert("/a/c.txt", b"3".to_vec());
        assert_eq!(fs.read(Path::new("/a/b/one.txt")).unwrap(), b"1");
        assert!(matches!(
            fs.read(Path::new("/a/missing")),
            Err(FsError::NotFound { .. })
        ));
        assert!(fs.exists(Path::new("/a/b")));
        let listed = fs.list_dir(Path::new("/a")).unwrap();
        assert_eq!(
            listed,
            vec![PathBuf::from("/a/b"), PathBuf::from("/a/c.txt")]
        );
        fs.write_atomic(Path::new("/a/c.txt"), b"replaced").unwrap();
        assert_eq!(
            fs.read_to_string(Path::new("/a/c.txt")).unwrap(),
            "replaced"
        );
    }

    #[test]
    fn virtual_fs_bounded_entry_count_accepts_exactly_the_limit() {
        let fs = VirtualFs::new();
        fs.insert("/root/a", Vec::new());
        fs.insert("/root/nested/one", Vec::new());
        fs.insert("/root/nested/two", Vec::new());

        assert_eq!(
            fs.count_dir_entries_bounded(Path::new("/root"), 2).unwrap(),
            2
        );
        assert!(matches!(
            fs.count_dir_entries_bounded(Path::new("/root"), 1),
            Err(FsError::TooManyEntries { limit: 1, .. })
        ));
    }

    #[test]
    fn bounded_stream_accepts_the_limit_and_reads_one_refusal_sentinel() {
        let mut exact = std::io::Cursor::new(b"abc");
        let BoundedRead::Complete(bytes) = read_stream_bounded(&mut exact, 3).unwrap() else {
            panic!("an exact-limit stream must be complete");
        };
        assert_eq!(bytes, b"abc");
        assert_eq!(exact.position(), 3);

        let mut oversized = std::io::Cursor::new(b"abcde");
        assert!(matches!(
            read_stream_bounded(&mut oversized, 3).unwrap(),
            BoundedRead::LimitExceeded
        ));
        assert_eq!(oversized.position(), 4, "only one sentinel byte is read");
    }

    #[test]
    fn virtual_fs_enforces_bounded_reads_before_copy_or_utf8_decode() {
        let fs = VirtualFs::new();
        fs.insert("/exact", b"abc".to_vec());
        fs.insert("/oversized", vec![0xff; 4]);

        assert_eq!(
            fs.read_to_string_bounded(Path::new("/exact"), 3).unwrap(),
            "abc"
        );
        assert!(matches!(
            fs.read_to_string_bounded(Path::new("/oversized"), 3),
            Err(FsError::TooLarge { limit: 3, .. })
        ));
        assert!(matches!(
            fs.read_to_string_bounded(Path::new("/oversized"), 4),
            Err(FsError::NotUtf8 { .. })
        ));
    }

    #[test]
    fn virtual_fs_create_new_is_create_if_absent() {
        let fs = VirtualFs::new();
        assert!(fs.create_new(Path::new("/lock"), b"a").unwrap());
        assert!(!fs.create_new(Path::new("/lock"), b"b").unwrap());
        // The losing create mutated nothing.
        assert_eq!(fs.read(Path::new("/lock")).unwrap(), b"a");
    }

    #[test]
    fn virtual_fs_exact_directory_creation_and_node_kinds() {
        let fs = VirtualFs::new();
        assert!(fs.create_dir(Path::new("/empty")).unwrap());
        assert!(!fs.create_dir(Path::new("/empty")).unwrap());
        assert_eq!(
            fs.node_kind_no_follow(Path::new("/empty")).unwrap(),
            Some(FsNodeKind::Directory)
        );
        assert_eq!(
            fs.list_dir(Path::new("/empty")).unwrap(),
            Vec::<PathBuf>::new()
        );
        assert!(matches!(
            fs.read(Path::new("/empty")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::IsADirectory
        ));
        assert!(matches!(
            fs.remove_file(Path::new("/empty")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::IsADirectory
        ));

        fs.insert("/empty/file", b"bytes".to_vec());
        assert_eq!(
            fs.node_kind_no_follow(Path::new("/empty/file")).unwrap(),
            Some(FsNodeKind::RegularFile)
        );
        assert!(matches!(
            fs.list_dir(Path::new("/empty/file")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.remove_dir_all(Path::new("/empty/file")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert_eq!(fs.node_kind_no_follow(Path::new("/absent")).unwrap(), None);
    }

    #[test]
    fn virtual_fs_fixture_insertion_preserves_a_host_representable_tree() {
        let fs = VirtualFs::new();
        fs.insert("/tree", b"former file".to_vec());
        fs.insert("/tree/leaf", b"leaf".to_vec());
        assert_eq!(
            fs.node_kind_no_follow(Path::new("/tree")).unwrap(),
            Some(FsNodeKind::Directory)
        );
        assert_eq!(fs.read(Path::new("/tree/leaf")).unwrap(), b"leaf");

        fs.insert("/tree", b"replacement file".to_vec());
        assert_eq!(fs.read(Path::new("/tree")).unwrap(), b"replacement file");
        assert!(matches!(
            fs.node_kind_no_follow(Path::new("/tree/leaf")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
    }

    #[test]
    fn virtual_fs_never_creates_through_a_file_ancestor() {
        let fs = VirtualFs::new();
        fs.insert("/parent", b"file".to_vec());
        assert!(matches!(
            fs.node_kind_no_follow(Path::new("/parent/child")),
            Err(FsError::Io { path, err })
                if path == Path::new("/parent")
                    && err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.create_dir(Path::new("/parent/child")),
            Err(FsError::Io { path, err })
                if path == Path::new("/parent")
                    && err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.write_atomic(Path::new("/parent/child"), b"bytes"),
            Err(FsError::Io { .. })
        ));
        assert!(matches!(
            fs.create_new(Path::new("/parent/lock"), b"bytes"),
            Err(FsError::Io { .. })
        ));
        assert!(matches!(
            fs.read(Path::new("/parent/child")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.remove_file(Path::new("/parent/child")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.list_dir(Path::new("/parent/child")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert!(matches!(
            fs.remove_dir_all(Path::new("/parent/child")),
            Err(FsError::Io { err, .. })
                if err.kind() == std::io::ErrorKind::NotADirectory
        ));
        assert_eq!(fs.read(Path::new("/parent")).unwrap(), b"file");
    }

    #[test]
    fn virtual_fs_remove_file_and_dir() {
        let fs = VirtualFs::new();
        fs.insert("/ns/a/one", b"1".to_vec());
        fs.insert("/ns/a/two", b"2".to_vec());
        fs.insert("/ns/b/three", b"3".to_vec());
        fs.remove_file(Path::new("/ns/a/one")).unwrap();
        assert!(!fs.exists(Path::new("/ns/a/one")));
        assert!(matches!(
            fs.remove_file(Path::new("/ns/a/one")),
            Err(FsError::NotFound { .. })
        ));
        fs.remove_dir_all(Path::new("/ns/a")).unwrap();
        assert!(!fs.exists(Path::new("/ns/a")));
        // The sibling namespace is untouched.
        assert_eq!(fs.read(Path::new("/ns/b/three")).unwrap(), b"3");
        assert!(matches!(
            fs.remove_dir_all(Path::new("/ns/a")),
            Err(FsError::NotFound { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn std_fs_temp_name_collisions_never_follow_preexisting_symlinks() {
        let root = std::env::temp_dir().join(format!(
            "fmn-platform-temp-collision-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir(&root).expect("create test root");
        let victim = root.join("victim");
        std::fs::write(&victim, b"keep").expect("write victim");
        let destination = root.join("destination");
        let exclusive = root.join("exclusive");

        let first_sequence = TEMP_SEQ.load(Ordering::Relaxed);
        for sequence in first_sequence..first_sequence.saturating_add(256) {
            let write_collision = root.join(format!(
                ".fmn-tmp.{}.{}.destination",
                std::process::id(),
                sequence
            ));
            std::os::unix::fs::symlink(&victim, write_collision).expect("preplant write symlink");
        }

        StdFs
            .write_atomic(&destination, b"replacement")
            .expect("atomic write skips collisions");

        let create_sequence = TEMP_SEQ.load(Ordering::Relaxed);
        for sequence in create_sequence..create_sequence.saturating_add(256) {
            let create_collision = root.join(format!(
                ".fmn-new.{}.{}.exclusive",
                std::process::id(),
                sequence
            ));
            std::os::unix::fs::symlink(&victim, create_collision).expect("preplant create symlink");
        }

        assert!(
            StdFs
                .create_new(&exclusive, b"exclusive contents")
                .expect("exclusive create skips collisions")
        );
        assert_eq!(std::fs::read(&victim).expect("victim survives"), b"keep");
        assert_eq!(
            std::fs::read(&destination).expect("destination"),
            b"replacement"
        );
        assert_eq!(
            std::fs::read(&exclusive).expect("exclusive"),
            b"exclusive contents"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn manifest_loading_unescapes_newlines() {
        let fs = VirtualFs::new();
        fs.load_manifest("# comment\n/sys/x\t0-3\\n\n/sys/y\tabc\n");
        assert_eq!(fs.read_to_string(Path::new("/sys/x")).unwrap(), "0-3\n");
        assert_eq!(fs.read_to_string(Path::new("/sys/y")).unwrap(), "abc");
    }

    fn prepared_bytes(
        fs: Arc<dyn FileSystem>,
        path: &Path,
        byte: u8,
    ) -> Box<dyn PreparedAtomicFile> {
        let mut writer = fs.begin_atomic_file(path).unwrap();
        for _ in 0..4 {
            writer.write(&[byte; 1024]).unwrap();
        }
        writer.prepare().unwrap()
    }

    #[test]
    fn virtual_stream_create_new_arbitrates_at_commit_and_keeps_replace_explicit() {
        let fs = Arc::new(VirtualFs::new());
        let path = Path::new("/out/file");
        let first = prepared_bytes(fs.clone(), path, 1);
        let second = prepared_bytes(fs.clone(), path, 2);
        assert!(!fs.exists(path));
        first.commit_new().unwrap();
        assert!(matches!(second.commit_new(), Err(FsError::Io { err, .. })
            if err.kind() == std::io::ErrorKind::AlreadyExists));
        assert_eq!(fs.read(path).unwrap(), vec![1; 4096]);
        prepared_bytes(fs.clone(), path, 3).commit().unwrap();
        assert_eq!(fs.read(path).unwrap(), vec![3; 4096]);
        drop(prepared_bytes(fs.clone(), Path::new("/out/aborted"), 4));
        assert!(!fs.exists(Path::new("/out/aborted")));
        fs.create_dir(Path::new("/out/directory")).unwrap();
        assert!(
            prepared_bytes(fs.clone(), Path::new("/out/directory"), 5)
                .commit_new()
                .is_err()
        );
    }

    #[test]
    fn unimplemented_create_only_capability_never_falls_back_to_replace() {
        struct ReplaceOnly;
        impl PreparedAtomicFile for ReplaceOnly {
            fn commit(self: Box<Self>) -> Result<(), FsError> {
                panic!("a create-only request must never dispatch replacement")
            }
        }
        assert!(
            matches!(Box::new(ReplaceOnly).commit_new(), Err(FsError::Io { err, .. })
            if err.kind() == std::io::ErrorKind::Unsupported)
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn std_stream_create_new_has_one_complete_winner_and_cleans_staging() {
        let root = std::env::temp_dir().join(format!(
            "fmn-create-only-{}-{}",
            std::process::id(),
            TEMP_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("winner");
        let fs = Arc::new(StdFs);
        let ready = Arc::new(std::sync::Barrier::new(8));
        let workers: Vec<_> = (0_u8..8)
            .map(|byte| {
                let fs = fs.clone();
                let path = path.clone();
                let ready = ready.clone();
                std::thread::spawn(move || {
                    let token = prepared_bytes(fs, &path, byte);
                    ready.wait();
                    match token.commit_new() {
                        Ok(()) => Some(byte),
                        Err(FsError::Io { err, .. })
                            if err.kind() == std::io::ErrorKind::AlreadyExists =>
                        {
                            None
                        }
                        Err(error) => panic!("unexpected commit failure: {error}"),
                    }
                })
            })
            .collect();
        let winners: Vec<_> = workers
            .into_iter()
            .filter_map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(winners.len(), 1);
        assert_eq!(std::fs::read(&path).unwrap(), vec![winners[0]; 4096]);
        assert_eq!(fs.list_dir(&root).unwrap(), vec![path.clone()]);
        drop(prepared_bytes(fs.clone(), &root.join("abort"), 9));
        assert_eq!(fs.list_dir(&root).unwrap(), vec![path.clone()]);
        assert!(prepared_bytes(fs.clone(), &root, 8).commit_new().is_err());
        #[cfg(unix)]
        for target in [path.clone(), root.join("absent-target")] {
            let link = root.join(format!(
                "link-{}",
                target.file_name().unwrap().to_string_lossy()
            ));
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert!(prepared_bytes(fs.clone(), &link, 7).commit_new().is_err());
            assert_eq!(std::fs::read_link(&link).unwrap(), target);
        }
        assert!(!root.join("absent-target").exists());
        assert_eq!(std::fs::read(&path).unwrap(), vec![winners[0]; 4096]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
