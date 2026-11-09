//! The no-corpus-leak enforcement check (fm-aef, §15.3): the release-CI
//! gate that fails loudly if a CC BY-NC-SA private fixture ever lands in
//! the shippable set.
//!
//! **The private fixtures** (never shipped, never committed): the harvested
//! 3b1b TeX-string corpus (`corpus/`), the Look Gallery Reference captures
//! (`gallery/reference_captures/`), and the two local Reference checkouts
//! used for fixture generation (`scripts/manim_ref/`, `scripts/videos_ref/`).
//! What *is* public: `docs/g0/g0-4-corpus/` (hashes, counts, construct
//! names — never the strings) and the `docs/g0/*-renders/` directories,
//! which carry the engine's **own** renders (`fmn-*.png`, fmd-math layout
//! SVGs, G0-8 engine-comparison PNGs) — verified by this gate to contain
//! no Reference-capture bytes.
//!
//! **The mechanism**, three independent teeth plus one hermetic oracle:
//!
//! 1. **Path tooth** — `git ls-files` (ground truth for what packaging
//!    commits) plus the `dist/` and wheel/npm staging trees must contain
//!    no path under a private-fixture directory.
//! 2. **Gitignore tooth** — every private-fixture directory must be
//!    reported ignored by `git check-ignore` (not merely believed to be;
//!    the gate runs the real matcher against the real `.gitignore`).
//! 3. **Content tooth** — the committed `denominator.tsv` carries
//!    `sha256(mode + NUL + string)` for every corpus string (the public
//!    artifact of the private harvest). Every text file on the shippable
//!    surface is scanned in bounded chunks, with bounded line buffering,
//!    under the same hash convention — via fmn-hash's owned SHA-256, the
//!    same bytes the harvest recorded — so a copied corpus string is caught
//!    *without the corpus present in CI*.
//!    Where the private fixtures exist on disk (a developer machine),
//!    their whole-file digests are also collected and any byte-identical
//!    file on the surface is flagged (this catches copied Reference
//!    capture PNGs).
//!
//! The line scan hashes each line in three forms — raw, trimmed, and
//! trimmed with one layer of surrounding quotes plus a trailing comma or
//! semicolon stripped — under both harvest modes (`math`, `text`), so the
//! realistic leak forms (a copied `.jsonl`/`.tsv` fixture, a copied string
//! literal on its own line) are caught exactly. A corpus string
//! paraphrased or re-wrapped mid-line is out of scope by construction;
//! the path and whole-file teeth cover the artifact-level forms.
//!
//! The pure kernels (`path_violations`, `content_leaks`, …) take their
//! inputs as values so the negative tests need no filesystem fixtures.
//! [`run`] is the release-CI orchestration over a repository checkout. It
//! refuses non-UTF-8/control/escaping paths, staging symlinks, special
//! entries, read or traversal errors beneath a present root, files that
//! change while scanned, and named resource-budget overruns. Private-tree
//! symlinks are never followed and are outside the byte-identity tooth;
//! missing optional roots remain allowed. A silent skip would be a release
//! hole.

use crate::ratchet::Baseline;
use fmn_hash::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt;
use std::fs::{File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

/// The private-fixture directories, repository-relative, always with a
/// trailing slash for prefix matching (§15.3).
pub const PRIVATE_FIXTURE_DIRS: &[&str] = &[
    "corpus/",
    "gallery/reference_captures/",
    "scripts/manim_ref/",
    "scripts/videos_ref/",
];

/// The Reference-derived drawings assets by basename, lowercased (ADR-0020,
/// landed with fm-3kr tranche 3): the default art of `Lightbulb`,
/// `VideoIcon`/`VideoSeries`, the three Bubble bodies, and
/// `VectorizedEarth`. The classes take user-supplied files and the bytes
/// never ship — so a tracked or staged file carrying one of these names is
/// itself the leak. Matched case-insensitively on basename, with and
/// without the `.svg` suffix (the Reference's `file_name`s appear in both
/// forms across its loaders).
pub const REFERENCE_ASSET_BASENAMES: &[&str] = &[
    "lightbulb",
    "video_icon",
    "bubbles_speech.svg",
    "bubbles_double_speech.svg",
    "bubbles_thought.svg",
    "earth",
];

/// The distributable-surface staging trees scanned in addition to the
/// git-tracked set: `dist/` (the bundle staging this bead introduces) and
/// the wheel/npm staging locations as they appear. Missing trees are
/// skipped; present ones are scanned whether or not they are tracked.
pub const STAGING_DIRS: &[&str] = &["dist", "crates/fmn-python/dist", "wasm-smoke/pkg", "npm"];

/// The committed public artifact of the private harvest: per-string
/// content hashes, never the strings (§15.3).
pub const DENOMINATOR_PATH: &str = "docs/g0/g0-4-corpus/denominator.tsv";

/// The shared identity authority consumed by both the public coverage
/// ratchet and this release gate. The denominator is recomputed and must
/// agree with these fields before any release-surface scan begins.
pub const CORPUS_BASELINE_PATH: &str = "docs/ratchet/baseline.tsv";

/// The only accepted denominator header. The generator emits this exact
/// byte sequence; accepting variants would make the public authority less
/// specific than the private-corpus reproduction ritual.
pub const DENOMINATOR_HEADER: &str = "# sha256(mode + NUL + string)\tmode\toccurrences  — G0-4 ratchet denominator (fm-or4); strings are private fixtures per §15.3";

/// The harvest modes the denominator hashes under.
pub const CORPUS_MODES: &[&str] = &["math", "text"];

/// The minimum line-variant length the content tooth hashes. The corpus
/// holds thousands of trivially short strings (`"!"`, `" = "`, `"}"`,
/// even the empty string — harvest fragments of multi-argument `Tex()`
/// call sites); their hashes are structurally undetectable without
/// flagging ordinary source lines (every `}` line is such a fragment).
/// The NC-licensed substance — the authored strings a fixture leak would
/// actually copy — is long-form, so variants below this length are not
/// hashed. The path and whole-file teeth are length-blind and cover the
/// artifact-level leak forms regardless.
pub const MIN_PREIMAGE_LEN: usize = 16;

/// Longest repository-relative path accepted from Git or a staging tree.
pub const MAX_PATH_BYTES: usize = 4 * 1024;
/// Maximum NUL-delimited byte stream accepted from `git ls-files`.
pub const MAX_GIT_PATH_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
/// Maximum number of distinct files on the distributable surface.
pub const MAX_SURFACE_FILES: usize = 100_000;
/// Maximum directory entries visited across optional private/staging roots.
pub const MAX_TRAVERSAL_ENTRIES: usize = 250_000;
/// Maximum bytes accepted from one scanned file. Scanning is streaming, but
/// time and I/O still need a finite release-gate budget.
pub const MAX_SCANNED_FILE_BYTES: u64 = 1024 * 1024 * 1024;
/// Maximum aggregate bytes read from one scan class (private or surface).
pub const MAX_TOTAL_SCAN_BYTES: u64 = 16 * 1024 * 1024 * 1024;
/// Maximum findings retained before the gate refuses a potentially hostile
/// surface instead of allocating an unbounded report.
pub const MAX_FINDINGS: usize = 4_096;
/// Maximum text-line size eligible for corpus-preimage matching.
pub const MAX_LINE_BYTES: usize = 4_096;

/// Maximum bytes accepted from the public denominator authority.
pub const MAX_DENOMINATOR_BYTES: usize = 2 * 1024 * 1024;
/// Maximum data rows accepted from the public denominator authority.
pub const MAX_DENOMINATOR_ROWS: usize = 16_384;
/// Maximum bytes accepted from one denominator line, including the header.
pub const MAX_DENOMINATOR_LINE_BYTES: usize = 256;
/// Maximum bytes read from the shared coverage-baseline authority.
const MAX_CORPUS_BASELINE_BYTES: usize = 16 * 1024;

const IO_CHUNK_BYTES: usize = 64 * 1024;

/// One leak finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leak {
    /// Which tooth found it.
    pub kind: LeakKind,
    /// The offending repository-relative path.
    pub path: String,
    /// Human-readable detail (line number, matched form).
    pub detail: String,
}

/// The leak kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeakKind {
    /// A private-fixture path is present in the shippable set.
    PrivatePathShipped,
    /// A private-fixture directory is NOT excluded by `.gitignore`.
    PrivateDirNotIgnored,
    /// A file on the surface contains a corpus string (denominator hash
    /// match).
    CorpusStringShipped,
    /// A file on the surface is byte-identical to a private-fixture file.
    PrivateFileBytesShipped,
}

impl fmt::Display for Leak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            LeakKind::PrivatePathShipped => "private-fixture path shipped",
            LeakKind::PrivateDirNotIgnored => "private-fixture directory not gitignored",
            LeakKind::CorpusStringShipped => "CC BY-NC-SA corpus string shipped",
            LeakKind::PrivateFileBytesShipped => "private-fixture bytes shipped",
        };
        write!(f, "{kind}: {} ({})", self.path, self.detail)
    }
}

/// A gate-level failure (as opposed to a finding): the check itself could
/// not run to completion.
#[derive(Debug)]
pub enum LeakError {
    /// `git` could not be run or reported failure. The gate refuses to
    /// pass without ground truth.
    Git { what: String },
    /// A required file could not be read.
    Io { path: String, what: String },
    /// A path or file type violates the closed release-surface policy.
    Policy { path: String, what: String },
    /// A finite resource budget was exceeded.
    Resource { what: String },
}

impl fmt::Display for LeakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git { what } => write!(f, "git ground truth unavailable: {what}"),
            Self::Io { path, what } => write!(f, "reading {path}: {what}"),
            Self::Policy { path, what } => write!(f, "surface policy for {path}: {what}"),
            Self::Resource { what } => write!(f, "no-corpus-leak resource limit: {what}"),
        }
    }
}

impl std::error::Error for LeakError {}

/// The corpus string hash convention, exactly as the G0-4 harvest recorded
/// it (`docs/g0/g0-4-corpus/denominator.tsv`): `sha256(mode + NUL + string)`.
#[must_use]
pub fn corpus_hash(mode: &str, string: &[u8]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(mode.as_bytes());
    hasher.update(&[0]);
    hasher.update(string);
    hasher.finalize()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CorpusIdentity {
    digest: Digest,
    rows: u64,
    occurrences: u64,
}

#[derive(Debug)]
struct ParsedDenominator {
    digests: Vec<Digest>,
    identity: CorpusIdentity,
}

fn denominator_error(what: impl Into<String>) -> LeakError {
    LeakError::Io {
        path: DENOMINATOR_PATH.to_owned(),
        what: what.into(),
    }
}

fn canonical_positive_u64(value: &str, row: usize) -> Result<u64, LeakError> {
    if value.is_empty()
        || value.starts_with('0')
        || value.len() > 20
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(denominator_error(format!(
            "row {row}: occurrence count must be canonical positive decimal"
        )));
    }
    value.parse::<u64>().map_err(|error| {
        denominator_error(format!(
            "row {row}: occurrence count is out of range: {error}"
        ))
    })
}

fn parse_denominator_document(text: &str) -> Result<ParsedDenominator, LeakError> {
    if text.len() > MAX_DENOMINATOR_BYTES {
        return Err(LeakError::Resource {
            what: format!(
                "{DENOMINATOR_PATH} has {} bytes, above limit {MAX_DENOMINATOR_BYTES}",
                text.len()
            ),
        });
    }
    if text.as_bytes().contains(&b'\r') {
        return Err(denominator_error("CR line endings are not canonical"));
    }
    if !text.ends_with('\n') {
        return Err(denominator_error("missing canonical final LF"));
    }

    let mut lines = text[..text.len() - 1].split('\n');
    let header = lines
        .next()
        .ok_or_else(|| denominator_error("missing canonical header"))?;
    if header != DENOMINATOR_HEADER {
        return Err(denominator_error(
            "header disagrees with the G0-4 authority",
        ));
    }

    let mut digests = Vec::new();
    let mut seen = HashSet::new();
    let mut identity_hasher = Sha256::new();
    let mut occurrences = 0_u64;
    let mut rows = 0_usize;
    for (line_index, line) in lines.enumerate() {
        let row = line_index + 1;
        if line.is_empty() || line.starts_with('#') {
            return Err(denominator_error(format!(
                "row {row}: blank and additional comment lines are not canonical"
            )));
        }
        if line.len() > MAX_DENOMINATOR_LINE_BYTES {
            return Err(LeakError::Resource {
                what: format!(
                    "{DENOMINATOR_PATH} row {row} has {} bytes, above line limit {MAX_DENOMINATOR_LINE_BYTES}",
                    line.len()
                ),
            });
        }
        rows = rows.checked_add(1).ok_or_else(|| LeakError::Resource {
            what: "denominator row counter overflow".to_owned(),
        })?;
        if rows > MAX_DENOMINATOR_ROWS {
            return Err(LeakError::Resource {
                what: format!("{DENOMINATOR_PATH} has more than {MAX_DENOMINATOR_ROWS} data rows"),
            });
        }

        let mut fields = line.split('\t');
        let hex = fields.next().unwrap_or_default();
        let mode = fields.next().ok_or_else(|| {
            denominator_error(format!(
                "row {row}: expected exactly three tab-separated fields"
            ))
        })?;
        let count = fields.next().ok_or_else(|| {
            denominator_error(format!(
                "row {row}: expected exactly three tab-separated fields"
            ))
        })?;
        if fields.next().is_some() {
            return Err(denominator_error(format!(
                "row {row}: expected exactly three tab-separated fields"
            )));
        }
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(denominator_error(format!(
                "row {row}: hash must be exactly 64 lowercase hexadecimal digits"
            )));
        }
        if !CORPUS_MODES.contains(&mode) {
            return Err(denominator_error(format!(
                "row {row}: unknown harvest mode '{mode}'"
            )));
        }
        let digest = Digest::from_hex(hex).map_err(|error| {
            denominator_error(format!("row {row}: malformed hash '{hex}': {error}"))
        })?;
        if !seen.insert(digest) {
            return Err(denominator_error(format!(
                "row {row}: duplicate corpus digest '{hex}'"
            )));
        }
        let count = canonical_positive_u64(count, row)?;
        occurrences = occurrences
            .checked_add(count)
            .ok_or_else(|| denominator_error("occurrence total overflows u64"))?;

        if rows > 1 {
            identity_hasher.update(b"\n");
        }
        identity_hasher.update(line.as_bytes());
        digests.push(digest);
    }
    if rows == 0 {
        return Err(denominator_error("denominator carries no data rows"));
    }
    let rows = u64::try_from(rows).map_err(|error| LeakError::Resource {
        what: format!("denominator row count cannot be represented: {error}"),
    })?;
    Ok(ParsedDenominator {
        digests,
        identity: CorpusIdentity {
            digest: identity_hasher.finalize(),
            rows,
            occurrences,
        },
    })
}

/// Parse the exact canonical denominator document into its distinct digest
/// rows. Header, newline, schema, field, vocabulary, count, uniqueness, and
/// resource-limit drift are all named gate failures.
pub fn parse_denominator(text: &str) -> Result<Vec<Digest>, LeakError> {
    Ok(parse_denominator_document(text)?.digests)
}

/// Tooth 1: every repository-relative path that falls under a
/// private-fixture directory, or carries a Reference-derived drawings
/// asset basename (ADR-0020).
#[must_use]
pub fn path_violations(rel_paths: &[String]) -> Vec<Leak> {
    let mut leaks = Vec::new();
    for path in rel_paths {
        let normalized = path.replace('\\', "/");
        let basename = normalized.rsplit('/').next().unwrap_or(&normalized);
        let lowercase = basename.to_ascii_lowercase();
        let stem = lowercase.strip_suffix(".svg").unwrap_or(&lowercase);
        if REFERENCE_ASSET_BASENAMES
            .iter()
            .any(|name| *name == lowercase || *name == stem)
        {
            leaks.push(Leak {
                kind: LeakKind::PrivatePathShipped,
                path: normalized.clone(),
                detail: format!(
                    "Reference-derived drawings asset '{basename}' must never \
                     ship — the classes take user-supplied files (ADR-0020)"
                ),
            });
        }
        for dir in PRIVATE_FIXTURE_DIRS {
            let dir = dir.trim_end_matches('/');
            if normalized == dir || normalized.starts_with(&format!("{dir}/")) {
                leaks.push(Leak {
                    kind: LeakKind::PrivatePathShipped,
                    path: normalized.clone(),
                    detail: format!("under private-fixture directory '{dir}/' (§15.3)"),
                });
            }
        }
    }
    leaks
}

/// The candidate byte forms of one text line the content tooth hashes: the
/// raw line, the trimmed line, and the trimmed line with one layer of
/// surrounding single/double quotes plus a trailing `,`/`;` stripped (the
/// string-literal-on-its-own-line form).
#[must_use]
pub fn line_variants(line: &str) -> Vec<&str> {
    let mut variants = vec![line];
    let trimmed = line.trim();
    if trimmed != line {
        variants.push(trimmed);
    }
    let body = trimmed
        .strip_suffix(',')
        .or_else(|| trimmed.strip_suffix(';'))
        .unwrap_or(trimmed);
    let unquoted = body
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            body.strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        });
    if let Some(unquoted) = unquoted
        && unquoted != trimmed
        && unquoted != line
    {
        variants.push(unquoted);
    }
    variants
}

/// Tooth 3 (content): the leaks inside one surface file. `corpus` is the
/// denominator digest set; `private_files` is the whole-file digest set of
/// any private fixtures present on disk (empty where they are absent).
#[must_use]
pub fn content_leaks(
    rel_path: &str,
    bytes: &[u8],
    corpus: &HashSet<Digest>,
    private_files: &HashSet<Digest>,
) -> Vec<Leak> {
    let mut leaks = Vec::new();
    if !bytes.is_empty() && private_files.contains(&fmn_hash::sha256(bytes)) {
        leaks.push(Leak {
            kind: LeakKind::PrivateFileBytesShipped,
            path: rel_path.to_owned(),
            detail: "byte-identical to a private fixture on disk".to_owned(),
        });
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return leaks;
    };
    for (index, line) in text.lines().enumerate() {
        if line.len() > 4096 {
            continue;
        }
        for variant in line_variants(line) {
            if variant.len() < MIN_PREIMAGE_LEN {
                continue;
            }
            for mode in CORPUS_MODES {
                if corpus.contains(&corpus_hash(mode, variant.as_bytes())) {
                    leaks.push(Leak {
                        kind: LeakKind::CorpusStringShipped,
                        path: rel_path.to_owned(),
                        detail: format!(
                            "line {} matches a denominator corpus hash (mode {mode})",
                            index + 1
                        ),
                    });
                }
            }
        }
    }
    leaks
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileSnapshot {
    len: u64,
    modified: SystemTime,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    ctime_seconds: i64,
    #[cfg(unix)]
    ctime_nanoseconds: i64,
    #[cfg(windows)]
    volume_serial_number: Option<u32>,
    #[cfg(windows)]
    file_index: Option<u64>,
    #[cfg(windows)]
    last_write_time: u64,
}

impl FileSnapshot {
    fn from_metadata(metadata: &Metadata, label: &str) -> Result<Self, LeakError> {
        let modified = metadata.modified().map_err(|error| LeakError::Io {
            path: label.to_owned(),
            what: format!("reading modification identity: {error}"),
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            Ok(Self {
                len: metadata.len(),
                modified,
                device: metadata.dev(),
                inode: metadata.ino(),
                ctime_seconds: metadata.ctime(),
                ctime_nanoseconds: metadata.ctime_nsec(),
            })
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt as _;
            Ok(Self {
                len: metadata.len(),
                modified,
                volume_serial_number: metadata.volume_serial_number(),
                file_index: metadata.file_index(),
                last_write_time: metadata.last_write_time(),
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(Self {
                len: metadata.len(),
                modified,
            })
        }
    }
}

#[derive(Debug)]
struct ScanBudget {
    label: &'static str,
    files: usize,
    bytes: u64,
    max_files: usize,
}

#[derive(Clone, Copy)]
enum SymlinkPolicy {
    /// Private trees are inputs to the optional byte-identity tooth, not
    /// release artifacts. Links there are never followed; skipping them
    /// keeps virtualenv and checkout links outside the scan authority.
    Skip,
    /// Staging trees are distributable artifacts, so a link is an
    /// ambiguous/out-of-root release input and must fail the gate.
    Refuse,
}

impl ScanBudget {
    fn new(label: &'static str, max_files: usize) -> Self {
        Self {
            label,
            files: 0,
            bytes: 0,
            max_files,
        }
    }

    fn admit(&mut self, path: &str, len: u64) -> Result<(), LeakError> {
        if len > MAX_SCANNED_FILE_BYTES {
            return Err(LeakError::Resource {
                what: format!(
                    "{} file {path} declares {len} bytes, above the per-file limit {MAX_SCANNED_FILE_BYTES}",
                    self.label
                ),
            });
        }
        let files = self
            .files
            .checked_add(1)
            .ok_or_else(|| LeakError::Resource {
                what: format!("{} file counter overflow", self.label),
            })?;
        if files > self.max_files {
            return Err(LeakError::Resource {
                what: format!(
                    "{} file count {files} exceeds limit {}",
                    self.label, self.max_files
                ),
            });
        }
        let bytes = self
            .bytes
            .checked_add(len)
            .ok_or_else(|| LeakError::Resource {
                what: format!("{} byte counter overflow", self.label),
            })?;
        if bytes > MAX_TOTAL_SCAN_BYTES {
            return Err(LeakError::Resource {
                what: format!(
                    "{} declared bytes {bytes} exceed aggregate limit {MAX_TOTAL_SCAN_BYTES}",
                    self.label
                ),
            });
        }
        self.files = files;
        self.bytes = bytes;
        Ok(())
    }
}

fn validate_relative_path(path: &str) -> Result<(), LeakError> {
    if path.is_empty() {
        return Err(LeakError::Policy {
            path: "<empty>".to_owned(),
            what: "repository-relative paths must not be empty".to_owned(),
        });
    }
    if path.len() > MAX_PATH_BYTES {
        return Err(LeakError::Resource {
            what: format!(
                "path has {} bytes, above the per-path limit {MAX_PATH_BYTES}",
                path.len()
            ),
        });
    }
    if path.chars().any(char::is_control) {
        return Err(LeakError::Policy {
            path: path.escape_default().to_string(),
            what: "control characters are not accepted in release-surface paths".to_owned(),
        });
    }
    if path.contains('\\') {
        return Err(LeakError::Policy {
            path: path.to_owned(),
            what: "git paths must use forward slashes".to_owned(),
        });
    }
    for component in Path::new(path).components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(LeakError::Policy {
                path: path.to_owned(),
                what: "path must contain only normal relative components".to_owned(),
            });
        }
    }
    Ok(())
}

fn relative_utf8(root: &Path, path: &Path) -> Result<String, LeakError> {
    let relative = path.strip_prefix(root).map_err(|error| LeakError::Policy {
        path: path.display().to_string(),
        what: format!("path escaped repository root: {error}"),
    })?;
    let relative = relative.to_str().ok_or_else(|| LeakError::Policy {
        path: path.display().to_string(),
        what: "non-UTF-8 release-surface paths are refused".to_owned(),
    })?;
    let normalized = relative.replace('\\', "/");
    validate_relative_path(&normalized)?;
    Ok(normalized)
}

fn admit_traversal_entry(visited: &mut usize) -> Result<(), LeakError> {
    *visited = visited.checked_add(1).ok_or_else(|| LeakError::Resource {
        what: "traversal entry counter overflow".to_owned(),
    })?;
    if *visited > MAX_TRAVERSAL_ENTRIES {
        return Err(LeakError::Resource {
            what: format!(
                "optional-root traversal visited {visited} entries, above limit {MAX_TRAVERSAL_ENTRIES}"
            ),
        });
    }
    Ok(())
}

fn validate_git_output_len(len: usize) -> Result<(), LeakError> {
    if len > MAX_GIT_PATH_OUTPUT_BYTES {
        return Err(LeakError::Resource {
            what: format!(
                "git ls-files emitted {len} bytes, above limit {MAX_GIT_PATH_OUTPUT_BYTES}"
            ),
        });
    }
    Ok(())
}

fn ensure_canonical_containment(root: &Path, path: &Path, label: &str) -> Result<(), LeakError> {
    let canonical_root = root.canonicalize().map_err(|error| LeakError::Io {
        path: root.display().to_string(),
        what: format!("canonicalizing repository root: {error}"),
    })?;
    let canonical_path = path.canonicalize().map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("canonicalizing scan path: {error}"),
    })?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: format!(
                "canonical path {} escapes repository root {}",
                canonical_path.display(),
                canonical_root.display()
            ),
        });
    }
    Ok(())
}

fn validated_regular_path(root: &Path, label: &str) -> Result<PathBuf, LeakError> {
    validate_relative_path(label)?;
    let mut path = root.to_path_buf();
    let mut components = Path::new(label).components().peekable();
    while let Some(component) = components.next() {
        path.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| LeakError::Io {
            path: label.to_owned(),
            what: format!("validating path ancestry: {error}"),
        })?;
        if metadata.file_type().is_symlink() {
            return Err(LeakError::Policy {
                path: label.to_owned(),
                what: format!(
                    "path component {} is a symlink",
                    path.strip_prefix(root).unwrap_or(&path).display()
                ),
            });
        }
        if components.peek().is_some() && !metadata.file_type().is_dir() {
            return Err(LeakError::Policy {
                path: label.to_owned(),
                what: format!(
                    "intermediate component {} is not a directory",
                    path.strip_prefix(root).unwrap_or(&path).display()
                ),
            });
        }
    }
    ensure_canonical_containment(root, &path, label)?;
    Ok(path)
}

fn walk_optional_roots<F>(
    root: &Path,
    relative_roots: &[&str],
    symlink_policy: SymlinkPolicy,
    mut visit_file: F,
) -> Result<(), LeakError>
where
    F: FnMut(&Path, &str) -> Result<(), LeakError>,
{
    let mut visited = 0_usize;
    for relative_root in relative_roots {
        let root_path = root.join(relative_root);
        let root_metadata = match std::fs::symlink_metadata(&root_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(LeakError::Io {
                    path: relative_root.to_string(),
                    what: format!("inspecting optional root: {error}"),
                });
            }
        };
        if root_metadata.file_type().is_symlink() && matches!(symlink_policy, SymlinkPolicy::Skip) {
            continue;
        }
        if !root_metadata.file_type().is_dir() {
            return Err(LeakError::Policy {
                path: relative_root.to_string(),
                what:
                    "present optional root must be a real directory, not a symlink or special entry"
                        .to_owned(),
            });
        }

        let mut stack = vec![root_path];
        while let Some(directory) = stack.pop() {
            let label = relative_utf8(root, &directory)?;
            let metadata =
                std::fs::symlink_metadata(&directory).map_err(|error| LeakError::Io {
                    path: label.clone(),
                    what: format!("revalidating traversed directory: {error}"),
                })?;
            if !metadata.file_type().is_dir() {
                return Err(LeakError::Policy {
                    path: label,
                    what: "traversed directory changed type or became a symlink".to_owned(),
                });
            }
            let directory_snapshot = FileSnapshot::from_metadata(&metadata, &label)?;
            ensure_canonical_containment(root, &directory, &label)?;
            let entries = std::fs::read_dir(&directory).map_err(|error| LeakError::Io {
                path: label.clone(),
                what: format!("reading present directory: {error}"),
            })?;
            for entry in entries {
                let entry = entry.map_err(|error| LeakError::Io {
                    path: directory.display().to_string(),
                    what: format!("reading directory entry: {error}"),
                })?;
                admit_traversal_entry(&mut visited)?;
                let path = entry.path();
                let relative = relative_utf8(root, &path)?;
                let file_type = entry.file_type().map_err(|error| LeakError::Io {
                    path: relative.clone(),
                    what: format!("reading directory-entry type: {error}"),
                })?;
                if file_type.is_symlink() {
                    match symlink_policy {
                        SymlinkPolicy::Skip => continue,
                        SymlinkPolicy::Refuse => {
                            return Err(LeakError::Policy {
                                path: relative,
                                what: "symlinks are refused on release staging surfaces".to_owned(),
                            });
                        }
                    }
                }
                if file_type.is_dir() {
                    stack.push(path);
                } else if file_type.is_file() {
                    visit_file(&path, &relative)?;
                } else {
                    return Err(LeakError::Policy {
                        path: relative,
                        what: "only regular files and real directories are accepted".to_owned(),
                    });
                }
            }
            let metadata_after =
                std::fs::symlink_metadata(&directory).map_err(|error| LeakError::Io {
                    path: label.clone(),
                    what: format!("postflight directory revalidation: {error}"),
                })?;
            if !metadata_after.file_type().is_dir()
                || FileSnapshot::from_metadata(&metadata_after, &label)? != directory_snapshot
            {
                return Err(LeakError::Policy {
                    path: label.clone(),
                    what: "directory changed while it was being traversed".to_owned(),
                });
            }
            ensure_canonical_containment(root, &directory, &label)?;
        }
    }
    Ok(())
}

fn open_regular_file(
    root: &Path,
    path: &Path,
    label: &str,
    budget: &mut ScanBudget,
) -> Result<(File, FileSnapshot), LeakError> {
    let validated_path = validated_regular_path(root, label)?;
    if validated_path != path {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "enumerated path does not match its validated repository path".to_owned(),
        });
    }
    let path_metadata = std::fs::symlink_metadata(path).map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("opening enumerated file: {error}"),
    })?;
    if !path_metadata.file_type().is_file() {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "enumerated surface entry is not a regular file".to_owned(),
        });
    }
    let path_snapshot = FileSnapshot::from_metadata(&path_metadata, label)?;
    budget.admit(label, path_snapshot.len)?;
    let file = File::open(path).map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("opening enumerated regular file: {error}"),
    })?;
    let handle_metadata = file.metadata().map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("reading opened-file identity: {error}"),
    })?;
    if !handle_metadata.file_type().is_file() {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "opened surface handle is not a regular file".to_owned(),
        });
    }
    let handle_snapshot = FileSnapshot::from_metadata(&handle_metadata, label)?;
    if path_snapshot != handle_snapshot {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "file identity changed between enumeration and open".to_owned(),
        });
    }
    ensure_canonical_containment(root, path, label)?;
    Ok((file, handle_snapshot))
}

fn validate_regular_file_postflight(
    root: &Path,
    path: &Path,
    label: &str,
    file: &File,
    expected: &FileSnapshot,
    bytes_read: u64,
) -> Result<(), LeakError> {
    if bytes_read != expected.len {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: format!(
                "stream read {bytes_read} bytes but preflight declared {}",
                expected.len
            ),
        });
    }
    let handle_after = file.metadata().map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("reading postflight handle identity: {error}"),
    })?;
    let handle_after = FileSnapshot::from_metadata(&handle_after, label)?;
    if &handle_after != expected {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "file changed while it was being scanned".to_owned(),
        });
    }
    let path_after = std::fs::symlink_metadata(path).map_err(|error| LeakError::Io {
        path: label.to_owned(),
        what: format!("revalidating scanned path: {error}"),
    })?;
    if !path_after.file_type().is_file() {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "scanned path changed type or became a symlink".to_owned(),
        });
    }
    let path_after = FileSnapshot::from_metadata(&path_after, label)?;
    if &path_after != expected {
        return Err(LeakError::Policy {
            path: label.to_owned(),
            what: "scanned path was replaced during the gate".to_owned(),
        });
    }
    ensure_canonical_containment(root, path, label)?;
    Ok(())
}

fn stream_digest<R: Read>(reader: &mut R, label: &str) -> Result<(Digest, u64), LeakError> {
    let mut hasher = Sha256::new();
    let mut bytes_read = 0_u64;
    let mut chunk = [0_u8; IO_CHUNK_BYTES];
    loop {
        let count = reader.read(&mut chunk).map_err(|error| LeakError::Io {
            path: label.to_owned(),
            what: format!("streaming file contents: {error}"),
        })?;
        if count == 0 {
            break;
        }
        bytes_read = bytes_read
            .checked_add(u64::try_from(count).map_err(|error| LeakError::Resource {
                what: format!("read-size conversion failed: {error}"),
            })?)
            .ok_or_else(|| LeakError::Resource {
                what: format!("byte counter overflow while scanning {label}"),
            })?;
        if bytes_read > MAX_SCANNED_FILE_BYTES {
            return Err(LeakError::Resource {
                what: format!(
                    "streamed bytes for {label} exceed per-file limit {MAX_SCANNED_FILE_BYTES}"
                ),
            });
        }
        hasher.update(&chunk[..count]);
    }
    Ok((hasher.finalize(), bytes_read))
}

fn push_finding(findings: &mut Vec<Leak>, leak: Leak) -> Result<(), LeakError> {
    if findings.len() == MAX_FINDINGS {
        return Err(LeakError::Resource {
            what: format!("finding count exceeds limit {MAX_FINDINGS}"),
        });
    }
    findings.push(leak);
    Ok(())
}

fn extend_findings(
    findings: &mut Vec<Leak>,
    additions: impl IntoIterator<Item = Leak>,
) -> Result<(), LeakError> {
    for leak in additions {
        push_finding(findings, leak)?;
    }
    Ok(())
}

fn line_leaks(
    rel_path: &str,
    line_number: usize,
    bytes: &[u8],
    corpus: &HashSet<Digest>,
    findings: &mut Vec<Leak>,
) -> Result<(), LeakError> {
    let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
    let Ok(line) = std::str::from_utf8(bytes) else {
        return Ok(());
    };
    for variant in line_variants(line) {
        if variant.len() < MIN_PREIMAGE_LEN {
            continue;
        }
        for mode in CORPUS_MODES {
            if corpus.contains(&corpus_hash(mode, variant.as_bytes())) {
                push_finding(
                    findings,
                    Leak {
                        kind: LeakKind::CorpusStringShipped,
                        path: rel_path.to_owned(),
                        detail: format!(
                            "line {line_number} matches a denominator corpus hash (mode {mode})"
                        ),
                    },
                )?;
            }
        }
    }
    Ok(())
}

fn stream_surface<R: Read>(
    reader: &mut R,
    rel_path: &str,
    corpus: &HashSet<Digest>,
) -> Result<(Digest, u64, Vec<Leak>), LeakError> {
    let mut hasher = Sha256::new();
    let mut bytes_read = 0_u64;
    let mut chunk = [0_u8; IO_CHUNK_BYTES];
    let mut line = Vec::with_capacity(MAX_LINE_BYTES);
    let mut overlong_line = false;
    let mut line_number = 1_usize;
    let mut findings = Vec::new();
    loop {
        let count = reader.read(&mut chunk).map_err(|error| LeakError::Io {
            path: rel_path.to_owned(),
            what: format!("streaming surface contents: {error}"),
        })?;
        if count == 0 {
            break;
        }
        bytes_read = bytes_read
            .checked_add(u64::try_from(count).map_err(|error| LeakError::Resource {
                what: format!("read-size conversion failed: {error}"),
            })?)
            .ok_or_else(|| LeakError::Resource {
                what: format!("byte counter overflow while scanning {rel_path}"),
            })?;
        if bytes_read > MAX_SCANNED_FILE_BYTES {
            return Err(LeakError::Resource {
                what: format!(
                    "streamed bytes for {rel_path} exceed per-file limit {MAX_SCANNED_FILE_BYTES}"
                ),
            });
        }
        hasher.update(&chunk[..count]);
        for &byte in &chunk[..count] {
            if byte == b'\n' {
                if !overlong_line {
                    line_leaks(rel_path, line_number, &line, corpus, &mut findings)?;
                }
                line.clear();
                overlong_line = false;
                line_number = line_number
                    .checked_add(1)
                    .ok_or_else(|| LeakError::Resource {
                        what: format!("line counter overflow while scanning {rel_path}"),
                    })?;
            } else if !overlong_line {
                if line.len() == MAX_LINE_BYTES {
                    line.clear();
                    overlong_line = true;
                } else {
                    line.push(byte);
                }
            }
        }
    }
    if !overlong_line && !line.is_empty() {
        line_leaks(rel_path, line_number, &line, corpus, &mut findings)?;
    }
    Ok((hasher.finalize(), bytes_read, findings))
}

fn scan_surface_file(
    root: &Path,
    rel_path: &str,
    corpus: &HashSet<Digest>,
    private_files: &HashSet<Digest>,
    budget: &mut ScanBudget,
) -> Result<Vec<Leak>, LeakError> {
    validate_relative_path(rel_path)?;
    let path = root.join(rel_path);
    let (mut file, snapshot) = open_regular_file(root, &path, rel_path, budget)?;
    let (digest, bytes_read, line_findings) = stream_surface(&mut file, rel_path, corpus)?;
    validate_regular_file_postflight(root, &path, rel_path, &file, &snapshot, bytes_read)?;

    let mut findings = Vec::new();
    if bytes_read != 0 && private_files.contains(&digest) {
        push_finding(
            &mut findings,
            Leak {
                kind: LeakKind::PrivateFileBytesShipped,
                path: rel_path.to_owned(),
                detail: "byte-identical to a private fixture on disk".to_owned(),
            },
        )?;
    }
    extend_findings(&mut findings, line_findings)?;
    Ok(findings)
}

/// Collects the whole-file digests of every regular file under private
/// fixture roots that exist on disk. Missing roots are optional; every error
/// beneath a present root is a named gate failure. Files are hashed in fixed
/// chunks and revalidated after reading, so no whole-file allocation or
/// silent read race can turn into a pass.
pub fn collect_private_file_digests(root: &Path) -> Result<HashSet<Digest>, LeakError> {
    let mut digests = HashSet::new();
    let mut budget = ScanBudget::new("private fixture", MAX_TRAVERSAL_ENTRIES);
    walk_optional_roots(
        root,
        PRIVATE_FIXTURE_DIRS,
        SymlinkPolicy::Skip,
        |path, relative| {
            let (mut file, snapshot) = open_regular_file(root, path, relative, &mut budget)?;
            let (digest, bytes_read) = stream_digest(&mut file, relative)?;
            validate_regular_file_postflight(root, path, relative, &file, &snapshot, bytes_read)?;
            if bytes_read != 0 {
                digests.insert(digest);
            }
            Ok(())
        },
    )?;
    Ok(digests)
}

fn parse_git_paths(bytes: &[u8]) -> Result<Vec<String>, LeakError> {
    validate_git_output_len(bytes.len())?;
    if !bytes.is_empty() && !bytes.ends_with(&[0]) {
        return Err(LeakError::Git {
            what: "git ls-files -z output lacks its final NUL terminator".to_owned(),
        });
    }
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for (index, raw) in bytes[..bytes.len() - 1]
        .split(|byte| *byte == 0)
        .enumerate()
    {
        if raw.is_empty() {
            return Err(LeakError::Git {
                what: format!("git ls-files emitted an empty path at index {index}"),
            });
        }
        let path = std::str::from_utf8(raw).map_err(|error| LeakError::Policy {
            path: format!("git path #{index}"),
            what: format!("non-UTF-8 paths are refused: {error}"),
        })?;
        validate_relative_path(path)?;
        paths.push(path.to_owned());
        if paths.len() > MAX_SURFACE_FILES {
            return Err(LeakError::Resource {
                what: format!(
                    "git tracked-file count exceeds release-surface limit {MAX_SURFACE_FILES}"
                ),
            });
        }
    }
    Ok(paths)
}

/// The git-tracked file set, repository-relative with forward slashes —
/// the ground truth of what a commit (and therefore any package built
/// from one) carries.
pub fn git_tracked_files(root: &Path) -> Result<Vec<String>, LeakError> {
    let mut child = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| LeakError::Git {
            what: format!("spawning git ls-files: {e}"),
        })?;
    let mut stdout = child.stdout.take().ok_or_else(|| LeakError::Git {
        what: "git ls-files did not provide its requested stdout pipe".to_owned(),
    })?;
    let mut bytes = Vec::new();
    let output_limit = u64::try_from(MAX_GIT_PATH_OUTPUT_BYTES)
        .map_err(|error| LeakError::Resource {
            what: format!("git-output limit conversion failed: {error}"),
        })?
        .checked_add(1)
        .ok_or_else(|| LeakError::Resource {
            what: "git-output limit overflow".to_owned(),
        })?;
    let read_result = stdout.by_ref().take(output_limit).read_to_end(&mut bytes);
    drop(stdout);
    if let Err(error) = read_result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(LeakError::Git {
            what: format!("reading git ls-files output: {error}"),
        });
    }
    if bytes.len() > MAX_GIT_PATH_OUTPUT_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        return Err(LeakError::Resource {
            what: format!("git ls-files output exceeds {MAX_GIT_PATH_OUTPUT_BYTES} bytes"),
        });
    }
    let status = child.wait().map_err(|error| LeakError::Git {
        what: format!("waiting for git ls-files: {error}"),
    })?;
    if !status.success() {
        return Err(LeakError::Git {
            what: format!("git ls-files exited with {status}"),
        });
    }
    parse_git_paths(&bytes)
}

/// Tooth 2: every private-fixture directory must be reported ignored by
/// the real `.gitignore` matcher.
pub fn gitignore_violations(root: &Path) -> Result<Vec<Leak>, LeakError> {
    let mut leaks = Vec::new();
    for &dir in PRIVATE_FIXTURE_DIRS {
        // Keep the trailing slash on the query: git only applies dir-only
        // patterns (`/corpus/`) to paths it knows are directories — a
        // slash-suffixed query matches even where the directory does not
        // exist on disk (fresh clones, CI checkouts), while a bare query
        // matches only when git can stat a real directory, which would
        // make this gate vacuous exactly where it matters most.
        let status = Command::new("git")
            .args(["check-ignore", "-q", "--", dir])
            .current_dir(root)
            .status()
            .map_err(|e| LeakError::Git {
                what: format!("spawning git check-ignore: {e}"),
            })?;
        match status.code() {
            Some(0) => {}
            Some(1) => leaks.push(Leak {
                kind: LeakKind::PrivateDirNotIgnored,
                path: dir.to_string(),
                detail: "git check-ignore does not exclude it — a private fixture could be \
                         committed (§15.3)"
                    .to_string(),
            }),
            code => {
                return Err(LeakError::Git {
                    what: format!("git check-ignore for {dir} exited with {code:?}"),
                });
            }
        }
    }
    Ok(leaks)
}

/// Lists regular files beneath every staging tree. Missing trees are
/// optional; traversal or type errors beneath a present tree fail closed.
fn staging_files(root: &Path) -> Result<Vec<String>, LeakError> {
    let mut files = Vec::new();
    walk_optional_roots(
        root,
        STAGING_DIRS,
        SymlinkPolicy::Refuse,
        |_path, relative| {
            files.push(relative.to_owned());
            if files.len() > MAX_SURFACE_FILES {
                return Err(LeakError::Resource {
                    what: format!(
                        "staging-file count exceeds release-surface limit {MAX_SURFACE_FILES}"
                    ),
                });
            }
            Ok(())
        },
    )?;
    Ok(files)
}

fn release_surface(root: &Path) -> Result<Vec<String>, LeakError> {
    let mut surface = git_tracked_files(root)?;
    surface.extend(staging_files(root)?);
    surface.sort();
    surface.dedup();
    if surface.len() > MAX_SURFACE_FILES {
        return Err(LeakError::Resource {
            what: format!(
                "distinct release-surface file count {} exceeds limit {MAX_SURFACE_FILES}",
                surface.len()
            ),
        });
    }
    for path in &surface {
        validate_relative_path(path)?;
    }
    Ok(surface)
}

fn initial_findings(root: &Path, surface: &[String]) -> Result<Vec<Leak>, LeakError> {
    let mut findings = Vec::new();
    extend_findings(&mut findings, gitignore_violations(root)?)?;
    for path in surface {
        extend_findings(&mut findings, path_violations(std::slice::from_ref(path)))?;
    }
    Ok(findings)
}

fn read_authority_utf8(root: &Path, relative: &str, max_bytes: usize) -> Result<String, LeakError> {
    let path = root.join(relative);
    let mut budget = ScanBudget::new("release authority", 1);
    let (mut file, snapshot) = open_regular_file(root, &path, relative, &mut budget)?;
    let max_bytes_u64 = u64::try_from(max_bytes).map_err(|error| LeakError::Resource {
        what: format!("authority limit conversion failed: {error}"),
    })?;
    if snapshot.len > max_bytes_u64 {
        return Err(LeakError::Resource {
            what: format!(
                "{relative} declares {} bytes, above authority limit {max_bytes}",
                snapshot.len
            ),
        });
    }
    let limit = max_bytes_u64
        .checked_add(1)
        .ok_or_else(|| LeakError::Resource {
            what: format!("authority limit overflow for {relative}"),
        })?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| LeakError::Io {
            path: relative.to_owned(),
            what: format!("streaming authority: {error}"),
        })?;
    if bytes.len() > max_bytes {
        return Err(LeakError::Resource {
            what: format!("{relative} exceeds authority limit {max_bytes}"),
        });
    }
    let bytes_read = u64::try_from(bytes.len()).map_err(|error| LeakError::Resource {
        what: format!("authority read count cannot be represented: {error}"),
    })?;
    validate_regular_file_postflight(root, &path, relative, &file, &snapshot, bytes_read)?;
    String::from_utf8(bytes).map_err(|error| LeakError::Io {
        path: relative.to_owned(),
        what: format!("authority is not UTF-8: {error}"),
    })
}

fn baseline_corpus_identity(text: &str) -> Result<CorpusIdentity, LeakError> {
    let baseline = Baseline::from_tsv(text).map_err(|what| LeakError::Io {
        path: CORPUS_BASELINE_PATH.to_owned(),
        what,
    })?;
    let digest = Digest::from_hex(&baseline.corpus_hash).map_err(|error| LeakError::Io {
        path: CORPUS_BASELINE_PATH.to_owned(),
        what: format!("corpus_hash is not a digest: {error}"),
    })?;
    Ok(CorpusIdentity {
        digest,
        rows: baseline.unique_total,
        occurrences: baseline.occurrence_total,
    })
}

fn validate_denominator_identity(
    found: &CorpusIdentity,
    expected: &CorpusIdentity,
) -> Result<(), LeakError> {
    if found == expected {
        return Ok(());
    }
    Err(denominator_error(format!(
        "recomputed identity disagrees with {CORPUS_BASELINE_PATH}: \
         digest {} vs {}, rows {} vs {}, occurrences {} vs {}",
        found.digest,
        expected.digest,
        found.rows,
        expected.rows,
        found.occurrences,
        expected.occurrences
    )))
}

fn denominator_set(root: &Path) -> Result<HashSet<Digest>, LeakError> {
    let denominator_text = read_authority_utf8(root, DENOMINATOR_PATH, MAX_DENOMINATOR_BYTES)?;
    let denominator = parse_denominator_document(&denominator_text)?;
    let baseline_text = read_authority_utf8(root, CORPUS_BASELINE_PATH, MAX_CORPUS_BASELINE_BYTES)?;
    let expected = baseline_corpus_identity(&baseline_text)?;
    validate_denominator_identity(&denominator.identity, &expected)?;
    Ok(denominator.digests.into_iter().collect())
}

fn scan_release_surface(
    root: &Path,
    surface: &[String],
    corpus: &HashSet<Digest>,
    private_files: &HashSet<Digest>,
    findings: &mut Vec<Leak>,
) -> Result<(), LeakError> {
    let mut budget = ScanBudget::new("release surface", MAX_SURFACE_FILES);
    for relative in surface {
        extend_findings(
            findings,
            scan_surface_file(root, relative, corpus, private_files, &mut budget)?,
        )?;
    }
    Ok(())
}

fn validate_root(root: &Path) -> Result<(), LeakError> {
    let metadata = std::fs::symlink_metadata(root).map_err(|error| LeakError::Io {
        path: root.display().to_string(),
        what: format!("inspecting repository root: {error}"),
    })?;
    if !metadata.file_type().is_dir() {
        return Err(LeakError::Policy {
            path: root.display().to_string(),
            what: "repository root must be a real directory, not a symlink or special entry"
                .to_owned(),
        });
    }
    Ok(())
}

fn run_inner(root: &Path) -> Result<Vec<Leak>, LeakError> {
    let corpus = denominator_set(root)?;
    let private_files = collect_private_file_digests(root)?;
    let surface = release_surface(root)?;
    let mut findings = initial_findings(root, &surface)?;
    scan_release_surface(root, &surface, &corpus, &private_files, &mut findings)?;
    Ok(findings)
}

/// The full release-CI gate over a repository checkout. Returns every
/// finding across all teeth; an empty vector is a pass. Gate-level
/// failures are [`LeakError`]s and never degrade into a silent skip.
pub fn run(root: &Path) -> Result<Vec<Leak>, LeakError> {
    validate_root(root)?;
    run_inner(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ShortReader<'a> {
        remaining: &'a [u8],
        max_chunk: usize,
    }

    impl Read for ShortReader<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output.len().min(self.max_chunk).min(self.remaining.len());
            output[..count].copy_from_slice(&self.remaining[..count]);
            self.remaining = &self.remaining[count..];
            Ok(count)
        }
    }

    struct FailingReader {
        first_read: bool,
    }

    impl Read for FailingReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.first_read {
                self.first_read = false;
                output[..3].copy_from_slice(b"abc");
                Ok(3)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected read failure",
                ))
            }
        }
    }

    fn example_finding() -> Leak {
        Leak {
            kind: LeakKind::CorpusStringShipped,
            path: "dist/example.txt".to_owned(),
            detail: "test finding".to_owned(),
        }
    }

    #[test]
    fn corpus_hash_matches_the_documented_convention() {
        // sha256("math" + NUL + "\\frac{1}{2}") — the denominator's own
        // construction, independently computable.
        let digest = corpus_hash("math", br"\frac{1}{2}");
        let mut hasher = Sha256::new();
        hasher.update(b"math\0\\frac{1}{2}");
        assert_eq!(digest, hasher.finalize());
    }

    #[test]
    fn path_violations_flag_private_dirs_and_pass_public_renders() {
        let paths = vec![
            "corpus/tex_corpus.jsonl".to_owned(),
            "gallery/reference_captures/glow.png".to_owned(),
            "scripts/manim_ref/manimlib/scene.py".to_owned(),
            "docs/g0/g0-2-renders/fmn-glow.png".to_owned(),
            "docs/g0/g0-4-corpus/denominator.tsv".to_owned(),
            "dist/FONT_BUNDLE.json".to_owned(),
        ];
        let leaks = path_violations(&paths);
        let flagged: Vec<&str> = leaks.iter().map(|l| l.path.as_str()).collect();
        assert_eq!(
            flagged,
            [
                "corpus/tex_corpus.jsonl",
                "gallery/reference_captures/glow.png",
                "scripts/manim_ref/manimlib/scene.py"
            ]
        );
    }

    #[test]
    fn content_tooth_catches_a_copied_corpus_string_in_each_form() {
        let corpus: HashSet<Digest> = [corpus_hash("math", br"\int_0^1 x^2\,dx")]
            .into_iter()
            .collect();
        let private = HashSet::new();
        for (form, bytes) in [
            (
                "raw line",
                b"header\n\\int_0^1 x^2\\,dx\nfooter\n".as_slice(),
            ),
            ("trimmed", b"   \\int_0^1 x^2\\,dx   \n".as_slice()),
            (
                "quoted literal",
                b"    \"\\int_0^1 x^2\\,dx\",\n".as_slice(),
            ),
        ] {
            let leaks = content_leaks("fixture.jsonl", bytes, &corpus, &private);
            assert_eq!(leaks.len(), 1, "{form}: {leaks:?}");
            assert_eq!(leaks[0].kind, LeakKind::CorpusStringShipped);
        }
        // A near miss is not a leak.
        assert!(content_leaks("ok.txt", b"\\int_0^1 x^2\\,dy\n", &corpus, &private).is_empty());
        // Nor is a corpus string below the minimum preimage length (short
        // harvest fragments are structurally undetectable without flagging
        // ordinary source lines — the documented boundary).
        let short: HashSet<Digest> = [corpus_hash("math", b"x")].into_iter().collect();
        assert!(content_leaks("ok.txt", b"x\n", &short, &private).is_empty());
    }

    #[test]
    fn content_tooth_catches_byte_identical_private_files() {
        let private: HashSet<Digest> = [fmn_hash::sha256(b"png-bytes")].into_iter().collect();
        let corpus = HashSet::new();
        let leaks = content_leaks(
            "docs/g0/g0-2-renders/glow.png",
            b"png-bytes",
            &corpus,
            &private,
        );
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].kind, LeakKind::PrivateFileBytesShipped);
    }

    fn denominator_document(rows: &[String]) -> String {
        format!("{DENOMINATOR_HEADER}\n{}\n", rows.join("\n"))
    }

    #[test]
    fn denominator_parser_recomputes_the_complete_canonical_identity() {
        let row = format!("{}\tmath\t3", "ab".repeat(32));
        let document = denominator_document(std::slice::from_ref(&row));
        let parsed = parse_denominator_document(&document).expect("canonical denominator parses");
        assert_eq!(parsed.digests.len(), 1);
        assert_eq!(parsed.identity.digest, fmn_hash::sha256(row.as_bytes()));
        assert_eq!(parsed.identity.rows, 1);
        assert_eq!(parsed.identity.occurrences, 3);
    }

    #[test]
    fn denominator_parser_rejects_noncanonical_envelopes_and_rows() {
        let lower = "ab".repeat(32);
        let other = "cd".repeat(32);
        let good_row = format!("{lower}\tmath\t1");
        let good = denominator_document(std::slice::from_ref(&good_row));
        assert!(parse_denominator(&good).is_ok());

        let invalid = [
            format!("wrong header\n{good_row}\n"),
            good.replace('\n', "\r\n"),
            good.trim_end_matches('\n').to_owned(),
            format!("{DENOMINATOR_HEADER}\n\n{good_row}\n"),
            format!("{DENOMINATOR_HEADER}\n# extra comment\n{good_row}\n"),
            format!("{DENOMINATOR_HEADER}\n"),
            denominator_document(&[format!("{lower}\tmath")]),
            denominator_document(&[format!("{lower}\tmath\t1\textra")]),
            denominator_document(&[format!("{}\tmath\t1", "AB".repeat(32))]),
            denominator_document(&[format!("{lower}\tMath\t1")]),
            denominator_document(&[format!("{lower}\tmath\t0")]),
            denominator_document(&[format!("{lower}\tmath\t01")]),
            denominator_document(&[format!("{lower}\tmath\t+1")]),
            denominator_document(&[good_row.clone(), good_row]),
            denominator_document(&[
                format!("{lower}\tmath\t{}", u64::MAX),
                format!("{other}\ttext\t1"),
            ]),
        ];
        for document in invalid {
            assert!(
                parse_denominator(&document).is_err(),
                "noncanonical denominator unexpectedly parsed: {document:?}"
            );
        }
    }

    #[test]
    fn denominator_parser_checks_document_line_and_row_limits() {
        let oversized = "x".repeat(MAX_DENOMINATOR_BYTES + 1);
        assert!(matches!(
            parse_denominator(&oversized),
            Err(LeakError::Resource { .. })
        ));

        let overlong_line = denominator_document(&["x".repeat(MAX_DENOMINATOR_LINE_BYTES + 1)]);
        assert!(matches!(
            parse_denominator(&overlong_line),
            Err(LeakError::Resource { .. })
        ));

        use std::fmt::Write as _;
        let mut too_many_rows = String::from(DENOMINATOR_HEADER);
        too_many_rows.push('\n');
        for row in 0..=MAX_DENOMINATOR_ROWS {
            writeln!(&mut too_many_rows, "{row:064x}\tmath\t1")
                .expect("writing to String cannot fail");
        }
        assert!(
            too_many_rows.len() <= MAX_DENOMINATOR_BYTES,
            "row-limit fixture must reach the row check before the document check"
        );
        assert!(matches!(
            parse_denominator(&too_many_rows),
            Err(LeakError::Resource { .. })
        ));
    }

    #[test]
    fn denominator_identity_is_bound_to_the_shared_ratchet_authority() {
        let digest = fmn_hash::sha256(b"one canonical row");
        let expected = CorpusIdentity {
            digest,
            rows: 7,
            occurrences: 11,
        };
        assert!(validate_denominator_identity(&expected, &expected).is_ok());
        for found in [
            CorpusIdentity {
                digest: fmn_hash::sha256(b"different rows"),
                ..expected.clone()
            },
            CorpusIdentity {
                rows: expected.rows + 1,
                ..expected.clone()
            },
            CorpusIdentity {
                occurrences: expected.occurrences + 1,
                ..expected.clone()
            },
        ] {
            assert!(validate_denominator_identity(&found, &expected).is_err());
        }
    }

    #[test]
    fn committed_denominator_matches_the_shared_ratchet_identity() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let corpus = denominator_set(&root).expect("committed corpus authorities agree");
        let baseline_text =
            read_authority_utf8(&root, CORPUS_BASELINE_PATH, MAX_CORPUS_BASELINE_BYTES)
                .expect("committed baseline reads");
        let expected = baseline_corpus_identity(&baseline_text).expect("baseline identity parses");
        assert_eq!(
            u64::try_from(corpus.len()).expect("bounded corpus length fits u64"),
            expected.rows
        );
    }

    #[test]
    fn streaming_scan_handles_short_reads_and_hashes_the_complete_file() {
        let copied = br"\int_0^1 x^2\,dx";
        let bytes = b"header\n\\int_0^1 x^2\\,dx\nfooter\n";
        let corpus = [corpus_hash("math", copied)].into_iter().collect();
        let mut reader = ShortReader {
            remaining: bytes,
            max_chunk: 3,
        };

        let (digest, count, findings) = stream_surface(&mut reader, "dist/example.txt", &corpus)
            .expect("short reads must not truncate a scan");

        assert_eq!(digest, fmn_hash::sha256(bytes));
        assert_eq!(count, bytes.len() as u64);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, LeakKind::CorpusStringShipped);
    }

    #[test]
    fn streaming_read_errors_are_named_gate_errors() {
        let error = stream_digest(
            &mut FailingReader { first_read: true },
            "dist/unreadable.bin",
        )
        .expect_err("an I/O failure after a short read must fail closed");
        assert!(matches!(
            error,
            LeakError::Io { path, what }
                if path == "dist/unreadable.bin" && what.contains("injected read failure")
        ));
    }

    #[test]
    fn git_path_parser_is_lossless_and_rejects_ambiguous_paths() {
        assert_eq!(
            parse_git_paths(b"src/lib.rs\0docs/read me.md\0").expect("valid paths"),
            ["src/lib.rs", "docs/read me.md"]
        );
        assert!(parse_git_paths(b"src/lib.rs").is_err());
        assert!(parse_git_paths(b"src/lib.rs\0\0").is_err());
        assert!(parse_git_paths(b"src/\xff.rs\0").is_err());
        assert!(parse_git_paths(b"../escaped.rs\0").is_err());
        assert!(parse_git_paths(b"windows\\separator.rs\0").is_err());
        let mut overlong_path = vec![b'a'; MAX_PATH_BYTES + 1];
        overlong_path.push(0);
        assert!(matches!(
            parse_git_paths(&overlong_path),
            Err(LeakError::Resource { .. })
        ));
        assert!(matches!(
            validate_git_output_len(MAX_GIT_PATH_OUTPUT_BYTES + 1),
            Err(LeakError::Resource { .. })
        ));
    }

    #[test]
    fn scan_budgets_refuse_oversized_files_and_unbounded_findings() {
        let mut budget = ScanBudget::new("test", 1);
        assert!(matches!(
            budget.admit("huge.bin", MAX_SCANNED_FILE_BYTES + 1),
            Err(LeakError::Resource { .. })
        ));
        budget.admit("first.bin", 0).expect("the first file fits");
        assert!(matches!(
            budget.admit("second.bin", 0),
            Err(LeakError::Resource { .. })
        ));

        let mut aggregate_budget = ScanBudget {
            label: "test",
            files: 0,
            bytes: MAX_TOTAL_SCAN_BYTES,
            max_files: 1,
        };
        assert!(matches!(
            aggregate_budget.admit("one-more-byte.bin", 1),
            Err(LeakError::Resource { .. })
        ));

        let mut visited = MAX_TRAVERSAL_ENTRIES;
        assert!(matches!(
            admit_traversal_entry(&mut visited),
            Err(LeakError::Resource { .. })
        ));

        let mut findings = vec![example_finding(); MAX_FINDINGS];
        assert!(matches!(
            push_finding(&mut findings, example_finding()),
            Err(LeakError::Resource { .. })
        ));
    }

    #[test]
    fn overlong_lines_are_bounded_without_hiding_later_lines() {
        let copied = br"\int_0^1 x^2\,dx";
        let corpus = [corpus_hash("math", copied)].into_iter().collect();
        let mut bytes = vec![b'x'; MAX_LINE_BYTES + 1];
        bytes.extend_from_slice(b"\n\\int_0^1 x^2\\,dx\n");

        let (_, count, findings) =
            stream_surface(&mut std::io::Cursor::new(&bytes), "dist/long.txt", &corpus)
                .expect("an overlong line is skipped without unbounding memory");

        assert_eq!(count, bytes.len() as u64);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].detail.contains("line 2"));
    }

    #[test]
    fn missing_enumerated_surface_file_is_not_a_pass() {
        let mut budget = ScanBudget::new("test", 1);
        let error = scan_surface_file(
            Path::new("/definitely-not-a-franken-manim-checkout"),
            "dist/missing.bin",
            &HashSet::new(),
            &HashSet::new(),
            &mut budget,
        )
        .expect_err("a file disappearing after enumeration must fail closed");
        assert!(matches!(error, LeakError::Io { .. }));
    }

    #[test]
    fn absent_private_roots_are_optional() {
        let digests =
            collect_private_file_digests(Path::new("/definitely-not-a-franken-manim-checkout"))
                .expect("missing private roots are allowed");
        assert!(digests.is_empty());
    }
}
