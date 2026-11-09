//! The self-golden rig (§16.3 plane 2, D-16, fm-xb3).
//!
//! Self-goldens are FrankenManim's **own** outputs, bit-locked: the regression
//! gate that actually blocks merges. This module owns the mechanism:
//!
//! - **Lock files** hold `(name, byte length, SHA-256)` rows — content hashes
//!   via fmn-hash, never the artifact bytes themselves — one file per
//!   `(suite, key)` where the key is the platform key (per-platform locks) or
//!   the literal `certified` (one lock shared by the whole certified matrix,
//!   §16.7).
//! - **Checking** an artifact recomputes its hash and compares against the
//!   lock. Any drift — changed bytes or a missing entry — is a hard error in
//!   check mode, which is what makes CI a merge blocker.
//! - **Blessing** (`UPDATE_GOLDENS=1`) rewrites the lock entry in the working
//!   tree. The rig never commits anything: a bless shows up in `git diff` for
//!   a human to review and commit, per the bead's "never auto-committing".
//! - **`.actual` sidecars**: on drift in check mode the offending bytes are
//!   written next to the lock (under `<suite>.<key>.actual/`), so a failure on
//!   CI or another machine can be diffed byte-for-byte. Sidecars are
//!   gitignored (`*.actual`).
//! - **The re-bless protocol** (fm-5wq.46, `docs/GOVERNANCE.md` §4): a lock
//!   at a new digest carries its before/after side-by-side under
//!   [`REBLESS_ARTEFACT_DIR`], checked by [`rebless_violations`], and the
//!   semantic sanity oracles (`crate::semantic`) stay green.
//!
//! Artifact names are constrained to a conservative character set — they are
//! path components, and a fixture name must never be a traversal vector.

use fmn_hash::sha256;
use std::cmp::Ordering as CmpOrdering;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

/// Serializes every load-modify-write of a lock file within this process:
/// `cargo test` runs tests in parallel, and two concurrent blesses into one
/// suite must not lose each other's entries. (Cross-process bless runs are
/// out of scope: CI checks, humans bless.)
static LOCK_FILE_GUARD: Mutex<()> = Mutex::new(());

/// Monotonic counter making concurrent tmp-file names unique.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The lock-file format version tag; the first line of every lock file.
const LOCK_HEADER_PREFIX: &str = "# fmn-golden-lock v1";

/// Lock files are compact hash ledgers, not artifact containers. One MiB
/// bounds malformed reads and canonical writes while leaving room for
/// thousands of entries.
const MAX_LOCK_FILE_BYTES: usize = 1024 * 1024;

/// Suite and artifact names become filesystem path components. The cap keeps
/// their derived lock, sidecar-directory, and `.actual` names portable.
const MAX_NAME_BYTES: usize = 128;

/// Paths are useful in a failure, but CI build roots can be arbitrarily long.
/// Keep the tail so the lock or sidecar name survives in bounded diagnostics.
const MAX_DIAGNOSTIC_PATH_BYTES: usize = 96;

struct DiagnosticPath<'a>(&'a Path);

impl fmt::Display for DiagnosticPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.0.to_string_lossy();
        if path.len() <= MAX_DIAGNOSTIC_PATH_BYTES {
            return f.write_str(&path);
        }

        let suffix_bytes = MAX_DIAGNOSTIC_PATH_BYTES - 3;
        let mut start = path.len() - suffix_bytes;
        while !path.is_char_boundary(start) {
            start += 1;
        }
        f.write_str("...")?;
        f.write_str(&path[start..])
    }
}

/// Which machines a lock file speaks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scope {
    /// One lock file per platform key (`<os>-<arch>`): the default for
    /// anything whose bits may legitimately differ across platforms until
    /// certified arithmetic covers it.
    PerPlatform,
    /// One lock file for the whole certified matrix: bits are promised
    /// identical everywhere (§16.7), so every platform checks the same lock.
    Certified,
}

/// Whether a mismatch fails (CI) or re-locks (a deliberate local bless).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Drift is an error; sidecars are written. What CI runs.
    Check,
    /// Drift updates the lock file in the working tree (never committed by
    /// the rig). Selected by `UPDATE_GOLDENS=1`.
    Bless,
}

impl Mode {
    /// Read the mode from the environment: `UPDATE_GOLDENS=1` means
    /// [`Mode::Bless`], anything else means [`Mode::Check`].
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("UPDATE_GOLDENS") {
            Ok(v) if v == "1" => Self::Bless,
            _ => Self::Check,
        }
    }
}

/// One locked artifact: its byte length (a fast pre-check and a human-legible
/// diff hint) and its SHA-256 in lowercase hex.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LockEntry {
    /// Artifact length in bytes.
    pub len: u64,
    /// Lowercase-hex SHA-256 of the artifact bytes.
    pub sha256_hex: String,
}

/// Outcome of a passing [`GoldenStore::check_with_mode`] call.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// The artifact matched its lock entry bit-for-bit.
    Match,
    /// Bless mode re-locked the artifact.
    Blessed {
        /// The entry that was replaced, or `None` if this is a new artifact.
        previous: Option<LockEntry>,
    },
}

/// A rig failure. [`GoldenError::Drift`] is the one CI exists to surface.
#[derive(Debug)]
pub enum GoldenError {
    /// The artifact or suite name is empty, oversized, or contains characters
    /// outside `[a-z0-9._-]` (names are path components; traversal is
    /// refused).
    InvalidName {
        /// Byte length of the rejected name; the input itself is not owned or
        /// copied into diagnostics.
        bytes: usize,
    },
    /// Filesystem failure reading or writing the lock or a sidecar.
    Io {
        /// The path being read or written.
        path: PathBuf,
        /// The underlying error.
        err: std::io::Error,
    },
    /// The lock file exists but cannot be parsed.
    Corrupt {
        /// The lock file path.
        path: PathBuf,
        /// 1-based line number of the offending line.
        line: usize,
        /// What was wrong with it.
        detail: String,
    },
    /// A lock file exceeds the format envelope on read or canonical write.
    LockTooLarge {
        /// The lock file path.
        path: PathBuf,
        /// Maximum permitted byte length.
        limit: usize,
    },
    /// The artifact does not match its lock (or has no entry). In check mode
    /// this is the merge-blocking failure; the actual bytes have been written
    /// to `sidecar` for inspection.
    Drift {
        /// The artifact name.
        name: String,
        /// The locked entry, or `None` when the artifact was never locked.
        expected: Option<LockEntry>,
        /// What the engine actually produced.
        actual: LockEntry,
        /// Where the actual bytes were written.
        sidecar: PathBuf,
    },
}

impl fmt::Display for GoldenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { bytes } => {
                write!(
                    f,
                    "invalid golden name ({bytes} bytes): expected 1..={MAX_NAME_BYTES} bytes \
                     using only [a-z0-9._-] and not starting with '.'"
                )
            }
            Self::Io { path, err } => {
                write!(f, "golden I/O failure at {}: {err}", DiagnosticPath(path))
            }
            Self::Corrupt { path, line, detail } => {
                write!(
                    f,
                    "corrupt lock file {} line {line}: {detail}",
                    DiagnosticPath(path)
                )
            }
            Self::LockTooLarge { path, limit } => write!(
                f,
                "golden lock file {} exceeds the {limit}-byte format limit",
                DiagnosticPath(path)
            ),
            Self::Drift {
                name,
                expected,
                actual,
                sidecar,
            } => match expected {
                Some(e) => write!(
                    f,
                    "self-golden drift for {name:?}: locked {} bytes sha256 {}, \
                     got {} bytes sha256 {} (actual bytes at {}; if deliberate, \
                     re-bless with UPDATE_GOLDENS=1 and commit the lock diff)",
                    e.len,
                    e.sha256_hex,
                    actual.len,
                    actual.sha256_hex,
                    DiagnosticPath(sidecar)
                ),
                None => write!(
                    f,
                    "self-golden {name:?} has no lock entry: got {} bytes sha256 {} \
                     (actual bytes at {}; lock it with UPDATE_GOLDENS=1 and commit)",
                    actual.len,
                    actual.sha256_hex,
                    DiagnosticPath(sidecar)
                ),
            },
        }
    }
}

impl std::error::Error for GoldenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { err, .. } => Some(err),
            _ => None,
        }
    }
}

/// The platform key per-platform locks are filed under: `<os>-<arch>` from
/// the running build (e.g. `linux-x86_64`, `macos-aarch64`).
#[must_use]
pub fn platform_key() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_NAME_BYTES
        && name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
        && !name.starts_with('.')
}

/// Split one lock row without allocating a field vector for malformed input
/// carrying an arbitrary number of tab separators.
fn split_lock_row(line: &str) -> Option<[&str; 3]> {
    let mut fields = line.split('\t');
    let exact = [fields.next()?, fields.next()?, fields.next()?];
    fields.next().is_none().then_some(exact)
}

/// A suite of bit-locked artifacts rooted at one directory.
#[derive(Clone, Debug)]
pub struct GoldenStore {
    dir: PathBuf,
    suite: String,
    key: String,
}

impl GoldenStore {
    fn lock_header(&self) -> String {
        format!("{LOCK_HEADER_PREFIX} suite={} key={}", self.suite, self.key)
    }

    /// Open (or designate) a golden store. `dir` is the directory holding the
    /// lock files (conventionally a committed `goldens/` under the crate);
    /// `suite` names the lock-file family.
    ///
    /// # Errors
    /// [`GoldenError::InvalidName`] if `suite` is not a safe path component.
    pub fn new(dir: impl Into<PathBuf>, suite: &str, scope: Scope) -> Result<Self, GoldenError> {
        if !valid_name(suite) {
            return Err(GoldenError::InvalidName { bytes: suite.len() });
        }
        let key = match scope {
            Scope::PerPlatform => platform_key(),
            Scope::Certified => "certified".to_string(),
        };
        Ok(Self {
            dir: dir.into(),
            suite: suite.to_string(),
            key,
        })
    }

    /// The lock file this store reads and (in bless mode) rewrites.
    #[must_use]
    pub fn lock_path(&self) -> PathBuf {
        self.dir.join(format!("{}.{}.lock", self.suite, self.key))
    }

    /// The directory drift sidecars are written into.
    #[must_use]
    pub fn sidecar_dir(&self) -> PathBuf {
        self.dir.join(format!("{}.{}.actual", self.suite, self.key))
    }

    /// Check `bytes` against the lock under the mode selected by the
    /// `UPDATE_GOLDENS` environment variable ([`Mode::from_env`]).
    ///
    /// # Errors
    /// See [`GoldenStore::check_with_mode`].
    pub fn check(&self, name: &str, bytes: &[u8]) -> Result<Verdict, GoldenError> {
        self.check_with_mode(name, bytes, Mode::from_env())
    }

    /// Check `bytes` against the lock entry for `name`.
    ///
    /// In [`Mode::Check`], a mismatch or missing entry writes the actual
    /// bytes to a `.actual` sidecar and returns [`GoldenError::Drift`]. In
    /// [`Mode::Bless`], the lock file is rewritten in place (sorted, atomic
    /// via tmp-and-rename) and the call succeeds with [`Verdict::Blessed`].
    ///
    /// # Errors
    /// [`GoldenError::InvalidName`] for a bad `name`; [`GoldenError::Io`] /
    /// [`GoldenError::Corrupt`] for lock-file trouble; [`GoldenError::Drift`]
    /// for a mismatch in check mode.
    pub fn check_with_mode(
        &self,
        name: &str,
        bytes: &[u8],
        mode: Mode,
    ) -> Result<Verdict, GoldenError> {
        if !valid_name(name) {
            return Err(GoldenError::InvalidName { bytes: name.len() });
        }
        // Hold the guard across load-modify-write so parallel tests blessing
        // into one suite cannot lose entries. A poisoned guard means another
        // test panicked mid-section; the lock file itself is still consistent
        // (writes are atomic renames), so continue rather than cascade.
        let _guard = LOCK_FILE_GUARD
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut entries = self.load_entries()?;
        let actual = LockEntry {
            len: bytes.len() as u64,
            sha256_hex: sha256(bytes).to_hex(),
        };
        if entries.get(name) == Some(&actual) {
            return Ok(Verdict::Match);
        }
        match mode {
            Mode::Bless => {
                let previous = entries.insert(name.to_string(), actual);
                self.write_entries(&entries)?;
                Ok(Verdict::Blessed { previous })
            }
            Mode::Check => {
                let sidecar = self.write_sidecar(name, bytes)?;
                Err(GoldenError::Drift {
                    name: name.to_string(),
                    expected: entries.remove(name),
                    actual,
                    sidecar,
                })
            }
        }
    }

    /// The locked entries, sorted by name. An absent lock file reads as empty
    /// (the bootstrap state).
    ///
    /// # Errors
    /// [`GoldenError::Io`] / [`GoldenError::Corrupt`] on unreadable or
    /// malformed lock files.
    pub fn load_entries(&self) -> Result<BTreeMap<String, LockEntry>, GoldenError> {
        let path = self.lock_path();
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(err) => return Err(GoldenError::Io { path, err }),
        };
        let mut bytes = Vec::new();
        file.take((MAX_LOCK_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|err| GoldenError::Io {
                path: path.clone(),
                err,
            })?;
        if bytes.len() > MAX_LOCK_FILE_BYTES {
            return Err(GoldenError::LockTooLarge {
                path,
                limit: MAX_LOCK_FILE_BYTES,
            });
        }
        let text = std::str::from_utf8(&bytes).map_err(|err| GoldenError::Io {
            path: path.clone(),
            err: std::io::Error::new(std::io::ErrorKind::InvalidData, err),
        })?;
        if let Some(offset) = bytes.iter().position(|&byte| byte == b'\r') {
            return Err(GoldenError::Corrupt {
                path,
                line: bytes
                    .iter()
                    .take(offset)
                    .filter(|&&byte| byte == b'\n')
                    .count()
                    + 1,
                detail: "carriage returns are forbidden; use LF line endings".to_string(),
            });
        }
        if bytes.is_empty() {
            return Err(GoldenError::Corrupt {
                path,
                line: 1,
                detail: "empty lock file (delete it or restore the header)".to_string(),
            });
        }
        if !bytes.ends_with(b"\n") {
            return Err(GoldenError::Corrupt {
                path,
                line: bytes.iter().filter(|&&byte| byte == b'\n').count() + 1,
                detail: "lock file must end with exactly one LF".to_string(),
            });
        }
        let mut lines = text.split_terminator('\n').enumerate();
        let expected_header = self.lock_header();
        match lines.next() {
            Some((_, first)) if first == expected_header => {}
            Some((_, first)) => {
                return Err(GoldenError::Corrupt {
                    path,
                    line: 1,
                    detail: format!(
                        "expected header {expected_header:?}, found {} bytes",
                        first.len()
                    ),
                });
            }
            None => {
                return Err(GoldenError::Corrupt {
                    path,
                    line: 1,
                    detail: "lock file has no header line".to_string(),
                });
            }
        }
        let mut entries = BTreeMap::new();
        let mut previous_name = None;
        for (idx, line) in lines {
            if line.is_empty() {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: "blank data rows are not canonical".to_string(),
                });
            }
            if line.starts_with('#') {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: "comment data rows are not canonical".to_string(),
                });
            }
            let Some([name, len, hex]) = split_lock_row(line) else {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: format!(
                        "expected 3 tab-separated fields, found {}",
                        line.split('\t').count()
                    ),
                });
            };
            if !valid_name(name) {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: format!("invalid artifact name ({} bytes)", name.len()),
                });
            }
            let parsed_len: u64 = len.parse().map_err(|_| GoldenError::Corrupt {
                path: path.clone(),
                line: idx + 1,
                detail: format!("invalid length field ({} bytes)", len.len()),
            })?;
            if parsed_len.to_string() != len {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: format!("noncanonical length field ({} bytes)", len.len()),
                });
            }
            if hex.len() != 64
                || !hex
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            {
                return Err(GoldenError::Corrupt {
                    path,
                    line: idx + 1,
                    detail: format!(
                        "invalid sha256 field ({} bytes; expected 64 lowercase hex digits)",
                        hex.len()
                    ),
                });
            }
            if let Some(previous) = previous_name {
                match name.cmp(previous) {
                    CmpOrdering::Less => {
                        return Err(GoldenError::Corrupt {
                            path,
                            line: idx + 1,
                            detail: format!(
                                "artifact names are not strictly increasing ({} then {} bytes)",
                                previous.len(),
                                name.len()
                            ),
                        });
                    }
                    CmpOrdering::Equal => {
                        return Err(GoldenError::Corrupt {
                            path,
                            line: idx + 1,
                            detail: format!("duplicate artifact name ({} bytes)", name.len()),
                        });
                    }
                    CmpOrdering::Greater => {}
                }
            }
            previous_name = Some(name);
            entries.insert(
                name.to_string(),
                LockEntry {
                    len: parsed_len,
                    sha256_hex: hex.to_string(),
                },
            );
        }
        if self.canonical_lock_text(&entries, &path)?.as_bytes() != bytes {
            return Err(GoldenError::Corrupt {
                path,
                line: 1,
                detail: "lock bytes do not match the canonical writer".to_string(),
            });
        }
        Ok(entries)
    }

    /// Rewrite the lock file: versioned header, then one sorted
    /// `name\tlen\tsha256` row per artifact. Written to a `.tmp` sibling and
    /// renamed into place so a crash never leaves a torn lock.
    fn write_entries(&self, entries: &BTreeMap<String, LockEntry>) -> Result<(), GoldenError> {
        let path = self.lock_path();
        let out = self.canonical_lock_text(entries, &path)?;
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let tmp = path.with_extension(format!(
            "lock.tmp.{}.{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        write_file(&tmp, out.as_bytes())?;
        std::fs::rename(&tmp, &path).map_err(|err| GoldenError::Io { path, err })
    }

    fn canonical_lock_text(
        &self,
        entries: &BTreeMap<String, LockEntry>,
        path: &Path,
    ) -> Result<String, GoldenError> {
        let header = self.lock_header();
        let mut len = header.len().saturating_add(1);
        for (name, entry) in entries {
            len = len
                .saturating_add(name.len())
                .saturating_add(entry.len.to_string().len())
                .saturating_add(entry.sha256_hex.len())
                .saturating_add(3);
        }
        if len > MAX_LOCK_FILE_BYTES {
            return Err(GoldenError::LockTooLarge {
                path: path.to_path_buf(),
                limit: MAX_LOCK_FILE_BYTES,
            });
        }
        let mut out = String::with_capacity(len);
        out.push_str(&header);
        out.push('\n');
        for (name, entry) in entries {
            out.push_str(name);
            out.push('\t');
            out.push_str(&entry.len.to_string());
            out.push('\t');
            out.push_str(&entry.sha256_hex);
            out.push('\n');
        }
        Ok(out)
    }

    /// Write the drift sidecar and return its path.
    fn write_sidecar(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, GoldenError> {
        let dir = self.sidecar_dir();
        create_dir_all(&dir)?;
        let path = dir.join(format!("{name}.actual"));
        write_file(&path, bytes)?;
        Ok(path)
    }
}

fn create_dir_all(path: &Path) -> Result<(), GoldenError> {
    std::fs::create_dir_all(path).map_err(|err| GoldenError::Io {
        path: path.to_path_buf(),
        err,
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), GoldenError> {
    let mut f = std::fs::File::create(path).map_err(|err| GoldenError::Io {
        path: path.to_path_buf(),
        err,
    })?;
    f.write_all(bytes).map_err(|err| GoldenError::Io {
        path: path.to_path_buf(),
        err,
    })?;
    f.sync_all().map_err(|err| GoldenError::Io {
        path: path.to_path_buf(),
        err,
    })
}

// ---------------------------------------------------------------------------
// The re-bless protocol (fm-5wq.46)
// ---------------------------------------------------------------------------
//
// A bit-lock proves "unchanged", never "correct": G1 passed on goldens that
// had locked mirrored frames for a month (fm-sq8.9). So a re-bless carries
// its evidence. Every golden lock at its current digest has, in the tree, the
// before/after side-by-side its adjudicator looked at, named for that digest
// ([`rebless_artefact_name`]), and the semantic sanity oracles
// (`crate::semantic`) stay green in the same `cargo test`. A lock edited
// without its artefact fails [`rebless_violations`] by construction.

/// Repository-relative directory of re-bless artefacts.
pub const REBLESS_ARTEFACT_DIR: &str = "crates/fmn-conformance/goldens/rebless";

/// The lock digests the protocol launched with (2026-10-06). They were
/// blessed before it existed, so they carry no artefact; never extend this
/// list.
pub const REBLESS_LAUNCH_BASELINE: &[(&str, &str)] = &[
    (
        "certified_engine.certified.lock",
        "0acafed2ad2974ff8e8d9862857c4bc3d8ea002a53f27f1b52fe37daa4001b6f",
    ),
    (
        "e2e.certified.lock",
        "fde00e29ec1c470009d54b42023832bcd93a2542ea853812e2acdbb7d81c9307",
    ),
    (
        "gallery_3d.certified.lock",
        "43778109c8520888c53415780fd69ecdaa9e5f4f1b2e8dca657f76fbd852b1ca",
    ),
    (
        "scene_goldens.certified.lock",
        "5fae39c1f9e7a84560fdc0b11280d28792286691ead698987cc5aa754dd87182",
    ),
    (
        "scene_runtime.certified.lock",
        "4d501cee9e747ce57f2525836ca6a1420ba3602dc08ea0ea2068402e735a7af4",
    ),
    (
        "self_goldens.certified.lock",
        "c6cc392fe473bf77fc0c8bc1bd0043636fe85ef9cd84006976ad820d61b05b5d",
    ),
];

/// The artefact a lock file at `digest` (SHA-256 hex of its bytes) must
/// carry: `<lock stem>-<first 12 hex digits>.png` in
/// [`REBLESS_ARTEFACT_DIR`].
#[must_use]
pub fn rebless_artefact_name(lock: &str, digest: &str) -> String {
    let stem = lock.strip_suffix(".lock").unwrap_or(lock);
    let prefix = digest.get(..12).unwrap_or(digest);
    format!("{stem}-{prefix}.png")
}

/// The protocol's violations for `locks` (`(file name, SHA-256 hex)` of
/// every lock file). `artefact` returns an artefact's bytes by file name in
/// [`REBLESS_ARTEFACT_DIR`], or `None` when it is absent.
#[must_use]
pub fn rebless_violations(
    locks: &[(String, String)],
    artefact: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Vec<String> {
    let mut violations = Vec::new();
    for (lock, digest) in locks {
        if REBLESS_LAUNCH_BASELINE
            .iter()
            .any(|(name, launched)| name == lock && launched == digest)
        {
            continue;
        }
        let name = rebless_artefact_name(lock, digest);
        match artefact(&name) {
            None => violations.push(format!(
                "{lock} changed to {digest} without its before/after artefact \
                 {REBLESS_ARTEFACT_DIR}/{name}"
            )),
            Some(bytes) if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") => violations.push(format!(
                "{REBLESS_ARTEFACT_DIR}/{name} is not a PNG side-by-side"
            )),
            Some(_) => {}
        }
    }
    violations
}

/// A before | after panel for a re-bless artefact: two same-size RGBA8
/// frames side by side with a 4-pixel magenta divider, PNG-encoded.
///
/// # Errors
/// When either frame is not `width * height * 4` bytes.
pub fn side_by_side_png(
    width: u32,
    height: u32,
    before: &[u8],
    after: &[u8],
) -> Result<Vec<u8>, String> {
    const DIVIDER: u32 = 4;
    let (w, h) = (width as usize, height as usize);
    if before.len() != w * h * 4 || after.len() != w * h * 4 {
        return Err(format!(
            "side-by-side frames must be {width}x{height} RGBA8 ({} bytes); got {} and {}",
            w * h * 4,
            before.len(),
            after.len()
        ));
    }
    let row_bytes = w * 4;
    let mut panel = Vec::with_capacity((2 * w + DIVIDER as usize) * h * 4);
    for row in 0..h {
        panel.extend_from_slice(&before[row * row_bytes..(row + 1) * row_bytes]);
        for _ in 0..DIVIDER {
            panel.extend_from_slice(&[255, 0, 255, 255]);
        }
        panel.extend_from_slice(&after[row * row_bytes..(row + 1) * row_bytes]);
    }
    Ok(fmn_codec::encode_rgba8(
        2 * width + DIVIDER,
        height,
        &panel,
        fmn_codec::CompressionLevel::Best,
    ))
}
