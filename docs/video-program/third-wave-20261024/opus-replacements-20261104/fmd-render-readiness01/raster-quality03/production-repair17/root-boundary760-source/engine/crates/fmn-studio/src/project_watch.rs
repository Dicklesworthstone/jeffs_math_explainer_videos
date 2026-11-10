//! Content-based source watching for the stable Studio development host.
//!
//! Polling is explicit and clock-injected: no background thread, scene callback
//! or platform-specific watcher is hidden inside this type. Only declared paths
//! are watched. Cargo output and .git directories are excluded from recursion.

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use crate::{BuildError, ProtocolDigest, protocol_digest};

/// Optional filtering for directory roots. Explicitly named input files are
/// always watched, including assets with a different extension. Filtering does
/// not relax traversal/read budgets or the refusal of followed symlinks.
#[derive(Clone, Debug, Default)]
pub struct WatchFilter {
    /// Empty means all files; otherwise these are extension names without dots.
    pub extensions: Vec<String>,
    /// Directory entry names to omit at every depth, in addition to .git/target.
    pub ignored_directories: Vec<String>,
}

impl WatchFilter {
    fn validate(&self) -> Result<(), BuildError> {
        if self.extensions.len() > 32
            || self.ignored_directories.len() > 32
            || self.extensions.iter().any(|ext| {
                ext.is_empty() || ext.len() > 32 || !ext.bytes().all(|b| b.is_ascii_alphanumeric())
            })
            || self.ignored_directories.iter().any(|name| {
                name.is_empty()
                    || name.len() > 128
                    || matches!(name.as_str(), "." | "..")
                    || name.contains(['/', '\\', '\0'])
            })
        {
            return Err(error("invalid source-watch filter"));
        }
        Ok(())
    }

    fn includes(&self, path: &std::path::Path) -> bool {
        self.extensions.is_empty()
            || path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| {
                    self.extensions
                        .iter()
                        .any(|wanted| ext.eq_ignore_ascii_case(wanted))
                })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Input {
    Missing,
    Directory,
    File(ProtocolDigest),
}
type Snapshot = Vec<(PathBuf, Input)>;

/// Work bounds applied to each complete source scan, before reads/expansion.
#[derive(Clone, Copy, Debug)]
pub struct WatchLimits {
    /// Maximum admitted files/directories/missing input markers.
    pub max_entries: usize,
    /// Maximum total file bytes read in one scan.
    pub max_bytes: u64,
    /// Maximum depth below an explicitly watched root.
    pub max_depth: usize,
}
impl Default for WatchLimits {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_bytes: 64 * 1024 * 1024,
            max_depth: 64,
        }
    }
}

/// Debounced source changes, including new/deleted files and atomic-save renames.
///
/// `poll` reports each stabilized changed content set once, before the caller
/// attempts its build. A failing build therefore does not cause an endless
/// rebuild loop; an edit made during the build is detected on the next poll.
/// Saving identical content or merely touching mtimes triggers no build.
/// A transient scan failure leaves both the prior snapshot and pending change
/// intact. This is change detection, not a sandbox or a Cargo dependency parser.
pub struct SourceWatch {
    inputs: Vec<PathBuf>,
    excludes: Vec<PathBuf>,
    limits: WatchLimits,
    filter: WatchFilter,
    debounce: Duration,
    observed: Snapshot,
    attempted: Snapshot,
    changed_at: Option<Duration>,
    last_poll: Option<Duration>,
}

impl SourceWatch {
    /// Watch explicit absolute files/directories. Missing inputs are recorded
    /// so a newly created source path will trigger a rebuild. Excluded absolute
    /// paths and their descendants are never traversed.
    pub fn new(
        inputs: Vec<PathBuf>,
        excludes: Vec<PathBuf>,
        debounce: Duration,
        limits: WatchLimits,
    ) -> Result<Self, BuildError> {
        Self::new_filtered(inputs, excludes, debounce, limits, WatchFilter::default())
    }

    /// Watch selected directory contents without treating unrelated generated
    /// outputs or cache directories as source edits. A file passed explicitly
    /// bypasses the extension filter. Changes in empty filtered directories do
    /// not trigger reloads, but adding/deleting an included file does.
    pub fn new_filtered(
        inputs: Vec<PathBuf>,
        excludes: Vec<PathBuf>,
        debounce: Duration,
        limits: WatchLimits,
        filter: WatchFilter,
    ) -> Result<Self, BuildError> {
        filter.validate()?;
        if inputs.is_empty()
            || inputs.len() > limits.max_entries
            || excludes.len() > limits.max_entries
            || limits.max_entries == 0
            || limits.max_entries > 65_536
            || limits.max_bytes == 0
            || limits.max_bytes > 512 * 1024 * 1024
            || limits.max_depth == 0
            || limits.max_depth > 128
            || inputs
                .iter()
                .chain(&excludes)
                .any(|path| !path.is_absolute())
        {
            return Err(error("invalid source-watch paths or work limits"));
        }
        let observed = scan(&inputs, &excludes, limits, &filter)?;
        Ok(Self {
            inputs,
            excludes,
            limits,
            filter,
            debounce,
            attempted: observed.clone(),
            observed,
            changed_at: None,
            last_poll: None,
        })
    }

    /// Content identity of the last complete observed scan. This is independent
    /// of debounce state and binds paths, entry kinds and file content, not
    /// mtimes. Construct a fresh watcher for a launch/publication snapshot.
    /// Path encodings are host-local; this is not a certified portable closure.
    #[must_use]
    pub fn content_fingerprint(&self) -> ProtocolDigest {
        let mut hash = fmn_hash::Sha256::new();
        hash.update(b"FMN-SOURCE-WATCH\0\x01");
        for (path, input) in &self.observed {
            let bytes = path.as_os_str().as_encoded_bytes();
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
            match input {
                Input::Missing => hash.update(&[0]),
                Input::Directory => hash.update(&[1]),
                Input::File(digest) => {
                    hash.update(&[2]);
                    hash.update(digest.as_bytes());
                }
            }
        }
        hash.finalize()
    }

    /// Borrow the file rows from that same observed scan, in path order.
    /// Consumers can compare actually compiled source bytes without rescanning
    /// or inventing another directory traversal/filtering policy.
    pub fn source_files(&self) -> impl Iterator<Item = (&std::path::Path, ProtocolDigest)> {
        self.observed
            .iter()
            .filter_map(|(path, input)| match input {
                Input::File(digest) => Some((path.as_path(), *digest)),
                _ => None,
            })
    }

    /// Return true once a different content set has remained stable for the
    /// debounce interval. `now` is monotone elapsed time from the host's Clock.
    pub fn poll(&mut self, now: Duration) -> Result<bool, BuildError> {
        if self.last_poll.is_some_and(|last| now < last) {
            return Err(error("source-watch clock moved backwards"));
        }
        let current = scan(&self.inputs, &self.excludes, self.limits, &self.filter)?;
        self.last_poll = Some(now);
        if current != self.observed {
            self.observed = current;
            self.changed_at = Some(now);
        }
        if self.observed == self.attempted {
            self.changed_at = None;
            return Ok(false);
        }
        if self
            .changed_at
            .is_some_and(|changed| now.saturating_sub(changed) >= self.debounce)
        {
            self.attempted = self.observed.clone();
            self.changed_at = None;
            return Ok(true);
        }
        Ok(false)
    }
}

fn scan(
    inputs: &[PathBuf],
    excludes: &[PathBuf],
    limits: WatchLimits,
    filter: &WatchFilter,
) -> Result<Snapshot, BuildError> {
    let mut work: Vec<_> = inputs.iter().cloned().map(|path| (path, 0)).collect();
    let mut seen = BTreeSet::new();
    let mut snapshot = Vec::new();
    let mut remaining = limits.max_bytes;
    while let Some((path, depth)) = work.pop() {
        if excludes.iter().any(|exclude| path.starts_with(exclude)) || seen.contains(&path) {
            continue;
        }
        if seen.len() >= limits.max_entries || depth > limits.max_depth {
            return Err(error("source-watch entry/depth budget exceeded"));
        }
        seen.insert(path.clone());
        let explicit = depth == 0 || inputs.contains(&path);
        snapshot.try_reserve(1).map_err(error)?;
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if explicit || filter.includes(&path) {
                    snapshot.push((path, Input::Missing));
                }
                continue;
            }
            Err(err) => return Err(error(err)),
        };
        if metadata.is_dir() {
            if explicit || filter.extensions.is_empty() {
                snapshot.push((path.clone(), Input::Directory));
            }
            for child in fs::read_dir(&path).map_err(error)? {
                let child = child.map_err(error)?;
                let child_path = child.path();
                if (matches!(child.file_name().to_str(), Some("target" | ".git"))
                    || filter
                        .ignored_directories
                        .iter()
                        .any(|name| child.file_name() == name.as_str()))
                    && child.file_type().map_err(error)?.is_dir()
                {
                    continue;
                }
                if work.len().saturating_add(seen.len()) >= limits.max_entries {
                    return Err(error("source-watch pending-entry budget exceeded"));
                }
                work.try_reserve(1).map_err(error)?;
                work.push((child_path, depth + 1));
            }
        } else if metadata.is_file() {
            if !explicit && !filter.includes(&path) {
                continue;
            }
            if metadata.len() > remaining {
                return Err(error("source-watch byte budget exceeded"));
            }
            let file = File::open(&path).map_err(error)?;
            if !file.metadata().map_err(error)?.is_file() {
                return Err(error("source changed type during watch scan"));
            }
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(usize::try_from(metadata.len()).map_err(error)?)
                .map_err(error)?;
            file.take(remaining.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(error)?;
            if bytes.len() as u64 > remaining {
                return Err(error("source grew past the watch byte budget"));
            }
            remaining -= bytes.len() as u64;
            snapshot.push((path, Input::File(protocol_digest(&bytes))));
        } else {
            return Err(error(format!(
                "source watcher refuses symlink or non-regular input: {}",
                path.display()
            )));
        }
    }
    snapshot.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(snapshot)
}
fn error(value: impl std::fmt::Display) -> BuildError {
    BuildError::new(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "fmn-watch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn watch(&self) -> SourceWatch {
            SourceWatch::new(
                vec![self.0.clone()],
                Vec::new(),
                Duration::from_millis(100),
                WatchLimits::default(),
            )
            .unwrap()
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn stable_content_is_reported_once_and_failed_builds_do_not_spin() {
        let temp = Temp::new();
        let source = temp.0.join("scene.rs");
        fs::write(&source, "one").unwrap();
        let mut watch = temp.watch();
        fs::write(&source, "two").unwrap();
        assert!(!watch.poll(ms(0)).unwrap());
        assert!(!watch.poll(ms(99)).unwrap());
        assert!(watch.poll(ms(100)).unwrap());
        assert!(!watch.poll(ms(1000)).unwrap());
        fs::write(&source, "two").unwrap();
        assert!(!watch.poll(ms(1100)).unwrap());
        fs::write(&source, "edit while building").unwrap();
        assert!(!watch.poll(ms(1200)).unwrap());
        assert!(watch.poll(ms(1300)).unwrap());
    }
    #[test]
    fn repeated_edits_debounce_and_reverts_cancel_pending_builds() {
        let temp = Temp::new();
        let source = temp.0.join("scene.rs");
        fs::write(&source, "one").unwrap();
        let mut watch = temp.watch();
        fs::write(&source, "two").unwrap();
        assert!(!watch.poll(ms(0)).unwrap());
        fs::write(&source, "three").unwrap();
        assert!(!watch.poll(ms(90)).unwrap());
        assert!(!watch.poll(ms(100)).unwrap());
        assert!(watch.poll(ms(190)).unwrap());
        fs::write(&source, "partial").unwrap();
        assert!(!watch.poll(ms(200)).unwrap());
        fs::write(&source, "three").unwrap();
        assert!(!watch.poll(ms(500)).unwrap());
    }
    #[test]
    fn directory_creation_deletion_and_output_exclusion() {
        let temp = Temp::new();
        let mut watch = temp.watch();
        fs::create_dir(temp.0.join("target")).unwrap();
        fs::write(temp.0.join("target/binary"), "not source").unwrap();
        assert!(!watch.poll(ms(0)).unwrap());
        fs::create_dir(temp.0.join("src")).unwrap();
        fs::write(temp.0.join("src/new.rs"), "new").unwrap();
        assert!(!watch.poll(ms(1)).unwrap());
        assert!(watch.poll(ms(101)).unwrap());
        fs::remove_file(temp.0.join("src/new.rs")).unwrap();
        assert!(!watch.poll(ms(102)).unwrap());
        assert!(watch.poll(ms(202)).unwrap());
    }
    #[test]
    fn refused_scan_preserves_pending_change_and_clock_order() {
        let temp = Temp::new();
        let source = temp.0.join("scene.rs");
        fs::write(&source, "a").unwrap();
        let mut watch = SourceWatch::new(
            vec![source.clone()],
            Vec::new(),
            ms(100),
            WatchLimits {
                max_bytes: 2,
                ..WatchLimits::default()
            },
        )
        .unwrap();
        fs::write(&source, "b").unwrap();
        assert!(!watch.poll(ms(1)).unwrap());
        fs::write(&source, "too large").unwrap();
        assert!(watch.poll(ms(150)).is_err());
        fs::write(&source, "b").unwrap();
        assert!(watch.poll(ms(151)).unwrap());
        assert!(watch.poll(ms(100)).is_err());
        assert!(!watch.poll(ms(200)).unwrap());
    }
    #[test]
    fn missing_explicit_file_is_watched_and_excluded_paths_do_not_trigger() {
        let temp = Temp::new();
        let source = temp.0.join("later.rs");
        let mut watch = SourceWatch::new(
            vec![source.clone()],
            Vec::new(),
            Duration::ZERO,
            WatchLimits::default(),
        )
        .unwrap();
        fs::write(source, "new").unwrap();
        assert!(watch.poll(ms(0)).unwrap());
        let generated = temp.0.join("generated");
        fs::create_dir(&generated).unwrap();
        let mut watch = SourceWatch::new(
            vec![temp.0.clone()],
            vec![generated.clone()],
            Duration::ZERO,
            WatchLimits::default(),
        )
        .unwrap();
        fs::write(generated.join("output"), "x").unwrap();
        assert!(!watch.poll(ms(0)).unwrap());
    }
    #[test]
    fn too_many_entries_refuse_instead_of_watching_an_incomplete_tree() {
        let temp = Temp::new();
        fs::write(temp.0.join("a"), "").unwrap();
        fs::write(temp.0.join("b"), "").unwrap();
        assert!(
            SourceWatch::new(
                vec![temp.0.clone()],
                Vec::new(),
                ms(100),
                WatchLimits {
                    max_entries: 2,
                    ..WatchLimits::default()
                }
            )
            .is_err()
        );
    }

    #[test]
    fn filtered_sources_ignore_generated_outputs_but_observe_helpers_and_explicit_assets() {
        let temp = Temp::new();
        fs::write(temp.0.join("scene.py"), "one").unwrap();
        let asset = temp.0.join("data.csv");
        fs::write(&asset, "one").unwrap();
        // The explicit asset comes first and is therefore traversed last: the
        // directory walk must not hide it behind an already-seen filtered path.
        let mut watch = SourceWatch::new_filtered(
            vec![asset.clone(), temp.0.clone()],
            Vec::new(),
            ms(100),
            WatchLimits::default(),
            WatchFilter {
                extensions: vec!["py".into(), "pyw".into()],
                ignored_directories: vec!["__pycache__".into(), ".venv".into()],
            },
        )
        .unwrap();
        fs::create_dir(temp.0.join("__pycache__")).unwrap();
        fs::create_dir(temp.0.join("media")).unwrap();
        fs::create_dir(temp.0.join(".venv")).unwrap();
        fs::write(temp.0.join("__pycache__/source.pyc"), "cache").unwrap();
        fs::write(temp.0.join(".venv/irrelevant.py"), "venv").unwrap();
        fs::write(temp.0.join("media/preview.png"), "render").unwrap();
        assert!(!watch.poll(ms(0)).unwrap());
        assert!(!watch.poll(ms(1000)).unwrap());
        fs::write(temp.0.join("helper.PY"), "new").unwrap();
        assert!(!watch.poll(ms(1100)).unwrap());
        assert!(watch.poll(ms(1200)).unwrap());
        fs::write(&asset, "two").unwrap();
        assert!(!watch.poll(ms(1300)).unwrap());
        assert!(watch.poll(ms(1400)).unwrap());
        fs::remove_file(temp.0.join("helper.PY")).unwrap();
        assert!(!watch.poll(ms(1500)).unwrap());
        assert!(watch.poll(ms(1600)).unwrap());
        assert!(!watch.poll(ms(1700)).unwrap());
    }

    #[test]
    fn filtered_watch_still_bounds_directory_work_and_selected_source_bytes() {
        let temp = Temp::new();
        let filter = WatchFilter {
            extensions: vec!["py".into()],
            ..WatchFilter::default()
        };
        fs::write(temp.0.join("large.png"), [0; 100]).unwrap();
        let mut watch = SourceWatch::new_filtered(
            vec![temp.0.clone()],
            Vec::new(),
            Duration::ZERO,
            WatchLimits {
                max_bytes: 4,
                ..WatchLimits::default()
            },
            filter.clone(),
        )
        .unwrap();
        fs::write(temp.0.join("large.py"), "over budget").unwrap();
        assert!(watch.poll(ms(0)).is_err());
        fs::write(temp.0.join("large.py"), "okay").unwrap();
        assert!(watch.poll(ms(1)).unwrap());
        assert!(
            SourceWatch::new_filtered(
                vec![temp.0.clone()],
                Vec::new(),
                Duration::ZERO,
                WatchLimits {
                    max_entries: 1,
                    ..WatchLimits::default()
                },
                filter
            )
            .is_err()
        );
        assert!(
            SourceWatch::new_filtered(
                vec![temp.0.clone()],
                Vec::new(),
                Duration::ZERO,
                WatchLimits::default(),
                WatchFilter {
                    extensions: vec!["../py".into()],
                    ..WatchFilter::default()
                }
            )
            .is_err()
        );
    }
}
