//! The fm-fw6 acceptance suite, deterministic half: every store behavior over
//! [`VirtualFs`] + [`FakeClock`] in the deterministic lab — atomicity under
//! simulated crashes, corruption injection, eviction/ceiling/pinning, the
//! cross-process advisory lock with staleness breaking, versioned-namespace
//! invalidation, cold-vs-warm equivalence, and the key-traversal fuzz.
//!
//! The host-filesystem half (real `StdFs`, threads, two store openings on one
//! directory) lives in `torture.rs`.

use fmn_cache::{
    CacheError, CacheKey, EvictOutcome, KeyBuilder, Namespace, NamespacePolicy, RootRefusalCode,
    Store, StoreConfig,
};
use fmn_hash::{Digest, Limits, Reader, Schema, UnknownPolicy, Writer, sha256};
use fmn_platform::clock::{Clock, FakeClock};
use fmn_platform::fs::{FileSystem, FsError, FsNodeKind, VirtualFs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

// Store::open validates absoluteness with host path semantics even over a
// VirtualFs, and `/cache` is not absolute on Windows (no drive prefix).
#[cfg(not(windows))]
const ROOT: &str = "/cache";
#[cfg(windows)]
const ROOT: &str = r"C:\cache";

fn open_store(fs: Arc<dyn FileSystem>, clock: Arc<dyn Clock>) -> Store {
    Store::open(fs, clock, ROOT, StoreConfig::default()).expect("open store")
}

fn fresh() -> (Arc<VirtualFs>, Arc<FakeClock>, Store) {
    let fs = Arc::new(VirtualFs::new());
    let clock = Arc::new(FakeClock::new());
    let store = open_store(fs.clone(), clock.clone());
    (fs, clock, store)
}

fn ns(store: &Store, ceiling: Option<u64>) -> Namespace {
    store
        .namespace(
            "t",
            1,
            NamespacePolicy {
                ceiling_bytes: ceiling,
            },
        )
        .expect("namespace")
}

fn key(material: &str) -> CacheKey {
    KeyBuilder::new("test")
        .push_str(material)
        .finish()
        .expect("key")
}

/// Every file under the store root, for structural assertions.
fn files_under(fs: &VirtualFs, root: &Path) -> Vec<PathBuf> {
    // Walk the deterministic virtual hierarchy through the public capability.
    fn walk(fs: &VirtualFs, dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(children) = fs.list_dir(dir) {
            for child in children {
                if fs.read(&child).is_ok() {
                    out.push(child);
                } else {
                    walk(fs, &child, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(fs, root, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Round trips, misses, blobs
// ---------------------------------------------------------------------------

#[test]
fn keyed_round_trip_and_miss() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let k = key("alpha");
    assert_eq!(n.get(&k).unwrap(), None, "cold store misses");
    n.put(&k, b"typeset result").unwrap();
    assert_eq!(n.get(&k).unwrap().as_deref(), Some(&b"typeset result"[..]));
    assert_eq!(n.get(&key("beta")).unwrap(), None, "other keys still miss");
}

#[test]
fn keyed_objects_are_immutable_and_identical_republication_is_idempotent() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let k = key("one-producer-key");
    let incumbent = b"first producer";
    let offered = b"second producer";

    n.put(&k, incumbent).expect("first publication wins");
    n.put(&k, incumbent)
        .expect("identical losing publication is idempotent");

    match n.put(&k, offered) {
        Err(CacheError::KeyConflict(conflict)) => {
            assert_eq!(conflict.namespace, "t");
            assert_eq!(conflict.version, 1);
            assert_eq!(conflict.key, *k.digest());
            assert_eq!(conflict.incumbent_payload, sha256(incumbent));
            assert_eq!(conflict.offered_payload, sha256(offered));
        }
        other => panic!("expected typed immutable-key conflict, got {other:?}"),
    }
    assert_eq!(
        n.get(&k).expect("read incumbent").as_deref(),
        Some(incumbent.as_slice()),
        "conflicting producer cannot replace the first publication"
    );
}

#[test]
fn blob_round_trip_is_self_addressed() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let addr = n.put_blob(b"asset bytes").unwrap();
    assert_eq!(addr, sha256(b"asset bytes"), "address is the content hash");
    assert_eq!(
        n.get_blob(&addr).unwrap().as_deref(),
        Some(&b"asset bytes"[..])
    );
    // A keyed lookup at a blob address is a kind mismatch → corrupt → miss.
    assert_eq!(n.get(&CacheKey::from_digest(addr)).unwrap(), None);
    // …and that lookup evicted the mismatched entry (never trusted).
    assert_eq!(n.get_blob(&addr).unwrap(), None);
}

#[test]
fn oversized_entries_are_a_precise_refusal() {
    let fs: Arc<dyn FileSystem> = Arc::new(VirtualFs::new());
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
    let store = Store::open(
        fs,
        clock,
        ROOT,
        StoreConfig {
            max_entry_bytes: 16,
            ..StoreConfig::default()
        },
    )
    .unwrap();
    let n = ns(&store, None);
    match n.put(&key("big"), &[0u8; 17]) {
        Err(CacheError::EntryTooLarge {
            limit: 16,
            needed: 17,
        }) => {}
        other => panic!("expected EntryTooLarge, got {other:?}"),
    }
    // The refusal cached nothing and poisoned nothing.
    assert_eq!(n.get(&key("big")).unwrap(), None);
    n.put(&key("small"), &[0u8; 16]).unwrap();
}

// ---------------------------------------------------------------------------
// Cold-vs-warm equivalence (the determinism interaction)
// ---------------------------------------------------------------------------

#[test]
fn cold_and_warm_get_or_compute_are_identical() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let k = key("formula");
    let computed = AtomicBool::new(false);
    let cold: Vec<u8> = n
        .get_or_compute(&k, || {
            computed.store(true, Ordering::Relaxed);
            Ok::<_, std::convert::Infallible>(b"layout bytes".to_vec())
        })
        .unwrap();
    assert!(computed.load(Ordering::Relaxed), "cold path computes");

    computed.store(false, Ordering::Relaxed);
    let warm: Vec<u8> = n
        .get_or_compute(&k, || {
            computed.store(true, Ordering::Relaxed);
            Ok::<_, std::convert::Infallible>(b"layout bytes".to_vec())
        })
        .unwrap();
    assert!(!computed.load(Ordering::Relaxed), "warm path hits");
    assert_eq!(
        cold, warm,
        "a hit is definitionally equivalent to a recompute"
    );
}

#[test]
fn compute_errors_surface_and_cache_failures_do_not() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None);
    // The compute error is the only error that escapes.
    let err = n
        .get_or_compute(&key("failing"), || Err::<Vec<u8>, &str>("typeset failed"))
        .unwrap_err();
    assert_eq!(err, "typeset failed");
}

/// A filesystem that can be switched read-only: writes fail, reads pass
/// through. `get_or_compute` must still deliver the computed value.
struct ReadOnlyFs {
    inner: VirtualFs,
    frozen: AtomicBool,
}

impl ReadOnlyFs {
    fn deny(path: &Path) -> FsError {
        FsError::Io {
            path: path.to_path_buf(),
            err: std::io::Error::other("simulated read-only filesystem"),
        }
    }
}

impl FileSystem for ReadOnlyFs {
    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError> {
        self.inner.node_kind_no_follow(path)
    }
    fn create_dir(&self, path: &Path) -> Result<bool, FsError> {
        if self.frozen.load(Ordering::Relaxed) {
            return Err(Self::deny(path));
        }
        self.inner.create_dir(path)
    }
    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.inner.read(path)
    }
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        self.inner.read_bounded(path, max_bytes)
    }
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if self.frozen.load(Ordering::Relaxed) {
            return Err(Self::deny(path));
        }
        self.inner.write_atomic(path, bytes)
    }
    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError> {
        if self.frozen.load(Ordering::Relaxed) {
            return Err(Self::deny(path));
        }
        self.inner.create_new(path, bytes)
    }
    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        if self.frozen.load(Ordering::Relaxed) {
            return Err(Self::deny(path));
        }
        self.inner.remove_file(path)
    }
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        if self.frozen.load(Ordering::Relaxed) {
            return Err(Self::deny(path));
        }
        self.inner.remove_dir_all(path)
    }
    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.inner.list_dir(path)
    }
    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError> {
        self.inner.count_dir_entries_bounded(path, max_entries)
    }
}

#[test]
fn a_cache_that_cannot_write_never_fails_the_consumer() {
    let fs = Arc::new(ReadOnlyFs {
        inner: VirtualFs::new(),
        frozen: AtomicBool::new(false),
    });
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
    let store = open_store(fs.clone(), clock);
    let n = ns(&store, None);
    fs.frozen.store(true, Ordering::Relaxed);
    // put fails precisely…
    assert!(n.put(&key("k"), b"v").is_err());
    // …but the read-through contract still delivers the computed value.
    let out: Vec<u8> = n
        .get_or_compute(&key("k"), || {
            Ok::<_, std::convert::Infallible>(b"computed".to_vec())
        })
        .unwrap();
    assert_eq!(out, b"computed");
}

// ---------------------------------------------------------------------------
// Atomicity under simulated crashes (kill -9 mid-write)
// ---------------------------------------------------------------------------

/// Simulates a writer killed during atomic publication, at both crash points
/// the capability allows: before anything persists, or with an orphaned temp
/// left in the shard directory (publication never happened).
struct CrashingFs {
    inner: VirtualFs,
    crash_writes: AtomicBool,
    leave_orphan: AtomicBool,
}

impl FileSystem for CrashingFs {
    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError> {
        self.inner.node_kind_no_follow(path)
    }
    fn create_dir(&self, path: &Path) -> Result<bool, FsError> {
        self.inner.create_dir(path)
    }
    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.inner.read(path)
    }
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        self.inner.read_bounded(path, max_bytes)
    }
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        if self.crash_writes.load(Ordering::Relaxed) {
            if self.leave_orphan.load(Ordering::Relaxed)
                && let Some(parent) = path.parent()
            {
                // The killed process persisted its unique temp but never
                // renamed it into place.
                self.inner.insert(
                    parent.join(".fmn-tmp.12345.orphan"),
                    bytes[..bytes.len() / 2].to_vec(),
                );
            }
            return Err(FsError::Io {
                path: path.to_path_buf(),
                err: std::io::Error::other("simulated kill -9 mid-write"),
            });
        }
        self.inner.write_atomic(path, bytes)
    }
    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError> {
        if self.crash_writes.load(Ordering::Relaxed) {
            if self.leave_orphan.load(Ordering::Relaxed)
                && let Some(parent) = path.parent()
            {
                self.inner.insert(
                    parent.join(".fmn-new.12345.orphan"),
                    bytes[..bytes.len() / 2].to_vec(),
                );
            }
            return Err(FsError::Io {
                path: path.to_path_buf(),
                err: std::io::Error::other("simulated kill -9 during no-clobber publication"),
            });
        }
        self.inner.create_new(path, bytes)
    }
    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_file(path)
    }
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_dir_all(path)
    }
    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.inner.list_dir(path)
    }
    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError> {
        self.inner.count_dir_entries_bounded(path, max_entries)
    }
}

#[test]
fn a_killed_writer_leaves_a_consistent_store() {
    let fs = Arc::new(CrashingFs {
        inner: VirtualFs::new(),
        crash_writes: AtomicBool::new(false),
        leave_orphan: AtomicBool::new(false),
    });
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
    let store = open_store(fs.clone(), clock);
    let n = store
        .namespace(
            "t",
            1,
            NamespacePolicy {
                ceiling_bytes: Some(1 << 20),
            },
        )
        .unwrap();
    let stable = key("stable");
    n.put(&stable, b"old value").unwrap();

    // The existing immutable entry cannot be replaced. Crash point A then
    // leaves a fresh key absent and reports the publication failure.
    assert!(matches!(
        n.put(&stable, b"new value"),
        Err(CacheError::KeyConflict(_))
    ));
    fs.crash_writes.store(true, Ordering::Relaxed);
    assert!(n.put(&key("fresh"), b"v").is_err());
    fs.crash_writes.store(false, Ordering::Relaxed);
    assert_eq!(n.get(&stable).unwrap().as_deref(), Some(&b"old value"[..]));
    assert_eq!(n.get(&key("fresh")).unwrap(), None);

    // Crash point B: an orphaned temp file in a shard directory. Reads are
    // unaffected; the next eviction pass sweeps it.
    fs.crash_writes.store(true, Ordering::Relaxed);
    fs.leave_orphan.store(true, Ordering::Relaxed);
    assert!(n.put(&key("fresh"), b"v").is_err());
    fs.crash_writes.store(false, Ordering::Relaxed);
    assert_eq!(n.get(&stable).unwrap().as_deref(), Some(&b"old value"[..]));
    let report = match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => report,
        other => panic!("expected a pass, got {other:?}"),
    };
    assert_eq!(report.swept_unrecognized, 1, "the orphan was reclaimed");
    assert_eq!(report.evicted, 0);
    // After recovery the store works normally.
    n.put(&key("fresh"), b"v").unwrap();
    assert_eq!(n.get(&key("fresh")).unwrap().as_deref(), Some(&b"v"[..]));
}

/// A filesystem where stat of a planted staging file answers the way Windows
/// answers for a delete-pending file: ACCESS_DENIED instead of NotFound.
struct DeletePendingFs {
    inner: VirtualFs,
}

const DELETE_PENDING_MARKER: &str = "pending-probe";

impl FileSystem for DeletePendingFs {
    fn node_kind_no_follow(&self, path: &Path) -> Result<Option<FsNodeKind>, FsError> {
        let pending = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().contains(DELETE_PENDING_MARKER));
        if pending {
            return Err(FsError::Io {
                path: path.to_path_buf(),
                err: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            });
        }
        self.inner.node_kind_no_follow(path)
    }
    fn create_dir(&self, path: &Path) -> Result<bool, FsError> {
        self.inner.create_dir(path)
    }
    fn read(&self, path: &Path) -> Result<Vec<u8>, FsError> {
        self.inner.read(path)
    }
    fn read_bounded(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        self.inner.read_bounded(path, max_bytes)
    }
    fn write_atomic(&self, path: &Path, bytes: &[u8]) -> Result<(), FsError> {
        self.inner.write_atomic(path, bytes)
    }
    fn create_new(&self, path: &Path, bytes: &[u8]) -> Result<bool, FsError> {
        self.inner.create_new(path, bytes)
    }
    fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_file(path)
    }
    fn remove_dir_all(&self, path: &Path) -> Result<(), FsError> {
        self.inner.remove_dir_all(path)
    }
    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FsError> {
        self.inner.list_dir(path)
    }
    fn count_dir_entries_bounded(&self, path: &Path, max_entries: usize) -> Result<usize, FsError> {
        self.inner.count_dir_entries_bounded(path, max_entries)
    }
}

/// Windows reports a concurrent writer's delete-pending `.fmn-new.*` staging
/// temp as ACCESS_DENIED when the evictor stats it mid-enumeration; every
/// other platform reports NotFound for the same lifecycle state. The
/// maintenance pass must treat it as a vanished entry (the writer owns its
/// temp), not a hard storage error — regression for the torture-suite
/// maintainer abort on windows-x86-64 (fm-p7gr).
#[test]
fn delete_pending_staging_files_do_not_abort_eviction_on_windows() {
    let fs = Arc::new(DeletePendingFs {
        inner: VirtualFs::new(),
    });
    let clock: Arc<dyn Clock> = Arc::new(FakeClock::new());
    let store = open_store(fs.clone(), clock);
    let n = ns(&store, Some(1 << 20));
    n.put(&key("a"), b"payload").unwrap();

    // Plant the mid-publish temp in a live shard directory.
    let shard_dir = files_under(&fs.inner, Path::new(ROOT))
        .into_iter()
        .find(|p| p.components().any(|c| c.as_os_str() == "objects"))
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .expect("one object shard exists");
    let probe = shard_dir.join(format!(".fmn-new.4396.133.{DELETE_PENDING_MARKER}"));
    fs.inner.insert(probe.clone(), b"half-written".to_vec());

    let outcome = n.evict_to_ceiling();
    if cfg!(windows) {
        // The pass completes and leaves the writer's temp alone — skipped,
        // not swept, because the writer still owns its lifecycle.
        let report = match outcome.unwrap() {
            EvictOutcome::Done(report) => report,
            other => panic!("expected a pass, got {other:?}"),
        };
        assert_eq!(report.swept_unrecognized, 0);
        assert!(fs.inner.read(&probe).is_ok(), "the temp survives the pass");
    } else {
        // Elsewhere a denied stat is a real permission problem and surfaces.
        assert!(matches!(
            outcome,
            Err(CacheError::Storage(FsError::Io { .. }))
        ));
    }
}

// ---------------------------------------------------------------------------
// Corruption injection
// ---------------------------------------------------------------------------

#[test]
fn flipped_bytes_are_detected_evicted_and_recomputed() {
    let (fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let k = key("victim");
    n.put(&k, b"good value").unwrap();

    // Find the one object file and flip a byte in the middle.
    let objects: Vec<PathBuf> = files_under(&fs, Path::new(ROOT))
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "objects"))
        .collect();
    assert_eq!(objects.len(), 1);
    let victim_path = &objects[0];
    let mut bytes = fs.read(victim_path).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x40;
    fs.write_atomic(victim_path, &bytes).unwrap();

    // Detected → evicted (file gone) → miss, never an error, and counted so
    // a front door can report the recovery instead of a silent recompute.
    assert_eq!(n.rejected_entries(), 0, "nothing rejected before the read");
    assert_eq!(n.get(&k).unwrap(), None, "corrupt entry reads as a miss");
    assert!(!fs.exists(victim_path), "corrupt entry was evicted");
    assert_eq!(n.rejected_entries(), 1, "the rejection is reported once");
    // The evicted entry is now plain absence, not a second rejection.
    assert_eq!(n.get(&k).unwrap(), None);
    assert_eq!(n.rejected_entries(), 1, "absence is not corruption");

    // The recompute path repopulates cleanly.
    let out: Vec<u8> = n
        .get_or_compute(&k, || {
            Ok::<_, std::convert::Infallible>(b"good value".to_vec())
        })
        .unwrap();
    assert_eq!(out, b"good value");
    assert_eq!(n.get(&k).unwrap().as_deref(), Some(&b"good value"[..]));
}

#[test]
fn truncation_and_garbage_files_are_misses() {
    let (fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let k = key("victim");
    n.put(&k, b"value").unwrap();
    let objects: Vec<PathBuf> = files_under(&fs, Path::new(ROOT))
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "objects"))
        .collect();
    let victim_path = &objects[0];

    // Truncated file.
    let bytes = fs.read(victim_path).unwrap();
    fs.write_atomic(victim_path, &bytes[..bytes.len() - 7])
        .unwrap();
    assert_eq!(n.get(&k).unwrap(), None);
    assert!(!fs.exists(victim_path));

    // Arbitrary garbage at the right path.
    n.put(&k, b"value").unwrap();
    fs.write_atomic(victim_path, b"not an envelope at all")
        .unwrap();
    assert_eq!(n.get(&k).unwrap(), None);
    assert!(!fs.exists(victim_path));
}

#[test]
fn a_valid_envelope_at_the_wrong_address_is_corrupt() {
    let (fs, _clock, store) = fresh();
    let n = ns(&store, None);
    let a = key("a");
    let b = key("b");
    n.put(&a, b"value of a").unwrap();
    n.put(&b, b"value of b").unwrap();
    let objects: Vec<PathBuf> = files_under(&fs, Path::new(ROOT))
        .into_iter()
        .filter(|p| p.components().any(|c| c.as_os_str() == "objects"))
        .collect();
    assert_eq!(objects.len(), 2);
    // Copy a's bytes over b's file: the envelope is pristine but its recorded
    // address is a's, so reading b must reject it.
    let a_hex = a.digest().to_hex();
    let (a_path, b_path) = if objects[0].to_string_lossy().contains(&a_hex[2..]) {
        (&objects[0], &objects[1])
    } else {
        (&objects[1], &objects[0])
    };
    let a_bytes = fs.read(a_path).unwrap();
    fs.write_atomic(b_path, &a_bytes).unwrap();
    assert_eq!(n.get(&b).unwrap(), None, "mis-placed entry is a miss");
    assert!(!fs.exists(b_path), "mis-placed entry was evicted");
    assert_eq!(n.get(&a).unwrap().as_deref(), Some(&b"value of a"[..]));
}

// ---------------------------------------------------------------------------
// Eviction, ceilings, pinning, LRU order
// ---------------------------------------------------------------------------

/// Entry envelope overhead is fixed; measure it once so ceiling arithmetic in
/// the tests is exact rather than magic.
fn envelope_size(store: &Store, payload_len: usize) -> u64 {
    let n = store
        .namespace("sizing", 1, NamespacePolicy::default())
        .unwrap();
    n.put(&key("probe"), &vec![0u8; payload_len]).unwrap();
    n.usage().unwrap()
}

const INDEX_SCHEMA: Schema = Schema::new(*b"FMNC", 2, 1, 0);

fn index_path() -> PathBuf {
    Path::new(ROOT).join("ns/t/v1/index")
}

/// An advisory index as another store opening (or an older build) could have
/// published. Re-declaring the schema here makes format drift test-visible.
fn foreign_index(next_seq: u64, entries: &[(Digest, u64, u64)]) -> Vec<u8> {
    let mut writer = Writer::new(INDEX_SCHEMA);
    writer.put_u64(next_seq);
    writer.put_u64(u64::try_from(entries.len()).expect("index entry count"));
    for (digest, size, last_seq) in entries {
        writer.put_digest(digest);
        writer.put_u64(*size);
        writer.put_u64(*last_seq);
    }
    writer.finish().expect("foreign index")
}

fn decoded_index(fs: &VirtualFs) -> (u64, Vec<(Digest, u64, u64)>) {
    let bytes = fs.read(&index_path()).expect("read index");
    let mut reader = Reader::open(&bytes, INDEX_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict)
        .expect("open index");
    let next_seq = reader.get_u64().expect("next sequence");
    let count = reader.get_u64().expect("entry count");
    let mut entries = Vec::new();
    for _ in 0..count {
        entries.push((
            reader.get_digest().expect("entry digest"),
            reader.get_u64().expect("entry size"),
            reader.get_u64().expect("entry sequence"),
        ));
    }
    reader.finish().expect("clean index tail");
    (next_seq, entries)
}

#[test]
fn a_schema_valid_max_sequence_is_rebased_before_touch() {
    let (fs, clock, store_a) = fresh();
    let per_entry = envelope_size(&store_a, 8);
    let older = key("sequence-older");
    let newer = key("sequence-newer");
    let n = ns(&store_a, None);
    n.put(&older, &[1u8; 8]).unwrap();
    n.put(&newer, &[2u8; 8]).unwrap();
    drop(n);

    fs.write_atomic(
        &index_path(),
        &foreign_index(
            u64::MAX,
            &[
                (*older.digest(), per_entry, u64::MAX - 1),
                (*newer.digest(), per_entry, u64::MAX),
            ],
        ),
    )
    .unwrap();

    let store_b = open_store(fs.clone(), clock);
    let n2 = ns(&store_b, None);
    assert_eq!(
        n2.get(&older).unwrap().as_deref(),
        Some(&[1u8; 8][..]),
        "touching a maximum-sequence index remains a normal cache hit"
    );
    n2.flush().expect("rewrite compact index");

    let (next_seq, entries) = decoded_index(&fs);
    assert_eq!(next_seq, 2, "two entries need only two compact ranks");
    let sequence = |digest: &Digest| {
        entries
            .iter()
            .find(|(candidate, _, _)| candidate == digest)
            .map(|(_, _, last_seq)| *last_seq)
            .expect("indexed digest")
    };
    assert_eq!(sequence(newer.digest()), 0);
    assert_eq!(sequence(older.digest()), 1, "the touched entry is newest");
}

#[test]
fn stale_index_sizes_cannot_distort_usage_or_eviction() {
    let (fs, clock, store_a) = fresh();
    let per_entry = envelope_size(&store_a, 8);
    let keys = [key("stale-a"), key("stale-b"), key("stale-c")];
    let n = ns(&store_a, None);
    for (index, cache_key) in keys.iter().enumerate() {
        n.put(cache_key, &[u8::try_from(index).unwrap(); 8])
            .unwrap();
    }
    drop(n);

    fs.write_atomic(
        &index_path(),
        &foreign_index(
            3,
            &[
                (*keys[0].digest(), 0, 0),
                (*keys[1].digest(), u64::MAX, 1),
                (*keys[2].digest(), u64::MAX, 2),
            ],
        ),
    )
    .unwrap();

    let store_b = open_store(fs.clone(), clock);
    let n2 = ns(&store_b, Some(2 * per_entry));
    assert_eq!(
        n2.usage().unwrap(),
        3 * per_entry,
        "usage measures objects instead of trusting stale zero or maximum sizes"
    );
    let report = match n2.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => report,
        other => panic!("expected a pass, got {other:?}"),
    };
    assert_eq!(report.evicted, 1);
    assert_eq!(report.evicted_bytes, per_entry);
    assert_eq!(report.retained_bytes, 2 * per_entry);
    assert_eq!(n2.usage().unwrap(), 2 * per_entry);
    assert_eq!(n2.get(&keys[0]).unwrap(), None, "true LRU was evicted");
    assert!(n2.get(&keys[1]).unwrap().is_some());
    assert!(n2.get(&keys[2]).unwrap().is_some());
}

#[test]
fn eviction_is_lru_ordered_and_respects_the_ceiling() {
    let (_fs, _clock, store) = fresh();
    let per_entry = envelope_size(&store, 8);
    // Room for exactly three entries.
    let n = ns(&store, Some(3 * per_entry));
    let keys: Vec<CacheKey> = (0..5).map(|i| key(&format!("k{i}"))).collect();
    for k in &keys {
        n.put(k, &[7u8; 8]).unwrap();
    }
    // Touch k0 and k1 so k2 becomes the least recently used.
    assert!(n.get(&keys[0]).unwrap().is_some());
    assert!(n.get(&keys[1]).unwrap().is_some());

    let report = match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => report,
        other => panic!("expected a pass, got {other:?}"),
    };
    assert_eq!(report.examined, 5);
    assert_eq!(report.evicted, 2);
    assert_eq!(report.evicted_bytes, 2 * per_entry);
    assert_eq!(report.retained_bytes, 3 * per_entry);
    assert!(report.retained_bytes <= 3 * per_entry, "ceiling honored");

    // The least-recently-used entries (k2, k3 — put early, never re-touched)
    // are the ones gone; the touched and the freshest survive.
    assert_eq!(n.get(&keys[2]).unwrap(), None);
    assert_eq!(n.get(&keys[3]).unwrap(), None);
    assert!(n.get(&keys[0]).unwrap().is_some());
    assert!(n.get(&keys[1]).unwrap().is_some());
    assert!(n.get(&keys[4]).unwrap().is_some());
}

#[test]
fn pinned_entries_survive_eviction_until_unpinned() {
    let (_fs, _clock, store) = fresh();
    let per_entry = envelope_size(&store, 8);
    let n = ns(&store, Some(2 * per_entry));
    let oldest = key("oldest");
    n.put(&oldest, &[1u8; 8]).unwrap();
    for i in 0..3 {
        n.put(&key(&format!("younger{i}")), &[2u8; 8]).unwrap();
    }

    // The oldest entry is LRU-first, but a pin holds it in place.
    let pin = n.pin(&oldest);
    let report = match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => report,
        other => panic!("expected a pass, got {other:?}"),
    };
    assert!(
        report.skipped_pinned >= 1,
        "the pin was honored: {report:?}"
    );
    assert!(n.get(&oldest).unwrap().is_some(), "pinned entry survived");

    // Unpinned, the same entry is evictable again. Its recent get bumped it,
    // so age it below fresher writes first.
    drop(pin);
    for i in 0..4 {
        n.put(&key(&format!("newest{i}")), &[3u8; 8]).unwrap();
    }
    match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => {
            assert_eq!(report.skipped_pinned, 0);
            assert!(report.evicted > 0);
        }
        other => panic!("expected a pass, got {other:?}"),
    }
    assert_eq!(n.get(&oldest).unwrap(), None, "unpinned entry evicted");
}

#[test]
fn manual_policy_never_evicts() {
    let (_fs, _clock, store) = fresh();
    let n = ns(&store, None); // the journal-namespace policy
    for i in 0..10 {
        n.put(&key(&format!("segment{i}")), &[0u8; 128]).unwrap();
    }
    assert!(matches!(
        n.evict_to_ceiling().unwrap(),
        EvictOutcome::Unlimited
    ));
    for i in 0..10 {
        assert!(n.get(&key(&format!("segment{i}"))).unwrap().is_some());
    }
}

#[test]
fn a_lost_index_degrades_lru_never_correctness() {
    let (fs, _clock, store) = fresh();
    let per_entry = envelope_size(&store, 8);
    let n = ns(&store, Some(2 * per_entry));
    for i in 0..4 {
        n.put(&key(&format!("k{i}")), &[9u8; 8]).unwrap();
    }
    // Retire the handle first (drop flushes bookkeeping), then destroy the
    // index outright.
    drop(n);
    let index_path = Path::new(ROOT).join("ns/t/v1/index");
    assert!(fs.exists(&index_path), "index exists after puts");
    fs.remove_file(&index_path).unwrap();

    // A fresh handle (no index, no in-memory log) must still evict to the
    // ceiling by rebuilding from disk truth.
    let n2 = ns(&store, Some(2 * per_entry));
    let report = match n2.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => report,
        other => panic!("expected a pass, got {other:?}"),
    };
    assert_eq!(report.examined, 4);
    assert_eq!(report.evicted, 2);
    assert!(report.retained_bytes <= 2 * per_entry);
    // Every surviving entry still round-trips (correctness intact).
    let survivors = (0..4)
        .filter(|i| n2.get(&key(&format!("k{i}"))).unwrap().is_some())
        .count();
    assert_eq!(survivors, 2);
}

// ---------------------------------------------------------------------------
// The advisory maintenance lock
// ---------------------------------------------------------------------------

/// A lock token as another fmn process would write it (the on-disk lock
/// format is deliberately exercised from outside the crate: a change to it
/// must be a conscious, test-breaking act).
fn foreign_lock_token(wall_nanos: u64) -> Vec<u8> {
    let mut w = fmn_hash::Writer::new(fmn_hash::Schema::new(*b"FMNC", 4, 1, 0));
    w.put_u64(99_999); // the foreign process id
    w.put_u64(7); // its store-instance id
    w.put_u64(wall_nanos); // when it acquired
    w.finish().expect("lock token")
}

#[test]
fn a_fresh_foreign_lock_skips_and_a_stale_one_is_broken() {
    let fs = Arc::new(VirtualFs::new());
    let clock = Arc::new(FakeClock::new());
    // Advance off the epoch so lock timestamps are nonzero.
    clock.advance(Duration::from_secs(1000));
    let store = open_store(fs.clone(), clock.clone());
    let per_entry = envelope_size(&store, 8);
    let n = ns(&store, Some(per_entry));
    n.put(&key("a"), &[0u8; 8]).unwrap();
    n.put(&key("b"), &[0u8; 8]).unwrap();

    // Another fmn invocation holds a fresh lock: this maintainer skips —
    // non-blocking, no waiting, no corruption.
    let lock_path = Path::new(ROOT).join("ns/t/v1/lock");
    let now_nanos = 1000u64 * 1_000_000_000;
    fs.write_atomic(&lock_path, &foreign_lock_token(now_nanos))
        .unwrap();
    assert!(matches!(
        n.evict_to_ceiling().unwrap(),
        EvictOutcome::SkippedLockHeld
    ));
    assert!(fs.exists(&lock_path), "a fresh foreign lock is left alone");

    // Time passes beyond the staleness horizon (the holder crashed): the
    // next maintainer breaks the lock and completes the pass.
    clock.advance(Duration::from_secs(120));
    match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => assert_eq!(report.evicted, 1),
        other => panic!("expected a pass after breaking stale lock, got {other:?}"),
    }
    assert!(!fs.exists(&lock_path), "the pass released its own lock");
}

#[test]
fn an_unparseable_lock_is_treated_as_abandoned() {
    let (fs, _clock, store) = fresh();
    let per_entry = envelope_size(&store, 8);
    let n = ns(&store, Some(per_entry));
    n.put(&key("a"), &[0u8; 8]).unwrap();
    n.put(&key("b"), &[0u8; 8]).unwrap();
    // Garbage where the lock token should be: it can never renew or expire,
    // so it must not deadlock maintenance forever.
    fs.write_atomic(&Path::new(ROOT).join("ns/t/v1/lock"), b"garbage")
        .unwrap();
    match n.evict_to_ceiling().unwrap() {
        EvictOutcome::Done(report) => assert_eq!(report.evicted, 1),
        other => panic!("expected a pass, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Versioned namespaces
// ---------------------------------------------------------------------------

#[test]
fn live_namespace_versions_are_cold_and_never_delete_each_other() {
    let (fs, clock, store_a) = fresh();
    let k = key("entry");

    let v1 = store_a
        .namespace("t", 1, NamespacePolicy::default())
        .unwrap();
    v1.put(&k, b"v1 value").unwrap();
    let unrelated = store_a
        .namespace("other", 1, NamespacePolicy::default())
        .unwrap();
    unrelated.put(&k, b"unrelated value").unwrap();

    // A separate opening selects v2. It is cold, but cannot infer that the
    // still-live v1 handle (or an equivalent handle in another process) is
    // abandoned.
    let store_b = open_store(fs.clone(), clock);
    let v2 = store_b
        .namespace("t", 2, NamespacePolicy::default())
        .unwrap();
    assert_eq!(
        v2.get(&k).unwrap(),
        None,
        "version bump invalidates cleanly"
    );
    assert!(
        fs.exists(&Path::new(ROOT).join("ns/t/v1")),
        "opening v2 leaves the live v1 tree intact"
    );
    assert_eq!(v1.get(&k).unwrap().as_deref(), Some(&b"v1 value"[..]));
    v1.put(&key("v1-after-v2"), b"still live").unwrap();

    // The unrelated namespace was never touched.
    assert_eq!(
        unrelated.get(&k).unwrap().as_deref(),
        Some(&b"unrelated value"[..])
    );
    // Both versions continue to fill independently.
    v2.put(&k, b"v2 value").unwrap();
    assert_eq!(v2.get(&k).unwrap().as_deref(), Some(&b"v2 value"[..]));
    assert_eq!(v1.get(&k).unwrap().as_deref(), Some(&b"v1 value"[..]));
}

// ---------------------------------------------------------------------------
// Owned-root establishment — the --clear-cache prerequisite
// ---------------------------------------------------------------------------

#[test]
fn an_existing_nonempty_unowned_root_is_never_claimed() {
    let fs = Arc::new(VirtualFs::new());
    fs.insert(Path::new(ROOT).join("important.txt"), b"keep me".to_vec());
    match Store::open(
        fs.clone(),
        Arc::new(FakeClock::new()),
        ROOT,
        StoreConfig::default(),
    ) {
        Err(CacheError::RootRefused { code, reason, .. }) => {
            assert_eq!(code, RootRefusalCode::OwnershipMissing);
            assert!(reason.contains("ownership manifest"));
        }
        other => panic!("expected RootRefused, got {other:?}"),
    }
    assert_eq!(
        fs.read(&Path::new(ROOT).join("important.txt")).unwrap(),
        b"keep me"
    );
    assert!(!fs.exists(&Path::new(ROOT).join("STORE_OWNER")));
    assert!(!fs.exists(&Path::new(ROOT).join("STORE_FORMAT")));
}

#[test]
fn a_generation_change_refuses_stale_mutation_and_drop_flush() {
    let (fs, _clock, store) = fresh();
    let namespace = ns(&store, None);
    let cache_key = key("generation-bound");
    namespace.put(&cache_key, b"old generation").unwrap();
    assert_eq!(
        namespace.get(&cache_key).unwrap().as_deref(),
        Some(&b"old generation"[..])
    );

    let owner_path = Path::new(ROOT).join("STORE_OWNER");
    let marker = String::from_utf8(fs.read(&owner_path).unwrap()).unwrap();
    let generation_start = marker.rfind(' ').expect("generation separator") + 1;
    let mut replacement = marker;
    replacement.replace_range(generation_start..generation_start + 64, &"0".repeat(64));
    fs.write_atomic(&owner_path, replacement.as_bytes())
        .unwrap();

    match namespace.put(&key("new"), b"must not publish") {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::GenerationChanged);
        }
        other => panic!("expected generation refusal, got {other:?}"),
    }
    let index_before_drop = fs.read(&index_path()).unwrap();
    drop(namespace);
    assert_eq!(
        fs.read(&index_path()).unwrap(),
        index_before_drop,
        "a stale best-effort Drop cannot flush into the new generation"
    );
}

#[test]
fn a_foreign_format_stamp_is_a_precise_refusal() {
    let (fs, _clock, _store) = fresh();
    fs.insert(
        Path::new(ROOT).join("STORE_FORMAT"),
        b"fmn-cache 999\n".to_vec(),
    );
    match Store::open(fs, Arc::new(FakeClock::new()), ROOT, StoreConfig::default()) {
        Err(CacheError::FormatUnsupported { found }) => {
            assert_eq!(found, "fmn-cache 999");
        }
        other => panic!("expected FormatUnsupported, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Traversal protection
// ---------------------------------------------------------------------------

#[test]
fn hostile_namespace_names_are_rejected() {
    let (_fs, _clock, store) = fresh();
    for hostile in [
        "..",
        "../../etc",
        "a/b",
        "a\\b",
        ".",
        "",
        ".hidden",
        "name.ext",
        "UPPER",
        "space name",
        "null\0byte",
        "über",
        "-flag",
    ] {
        match store.namespace(hostile, 1, NamespacePolicy::default()) {
            Err(CacheError::InvalidNamespace { name, .. }) => assert_eq!(name, hostile),
            other => panic!("{hostile:?} unexpectedly accepted: {other:?}"),
        }
    }
}

#[test]
fn arbitrary_key_bytes_never_map_outside_the_root() {
    // The traversal fuzz from the acceptance list: adversarial and random key
    // material must only ever produce object files under
    // <root>/ns/<name>/v<n>/objects/<hex>/<hex>. Keys never touch paths —
    // only their digests do — so this holds by construction; the fuzz
    // verifies the construction.
    let (fs, _clock, store) = fresh();
    let n = ns(&store, None);

    let mut hostile: Vec<Vec<u8>> = vec![
        b"../../../etc/passwd".to_vec(),
        b"..\\..\\windows\\system32".to_vec(),
        b"/absolute/path".to_vec(),
        b"nul\0byte".to_vec(),
        b"".to_vec(),
        vec![0xff; 1024],
        b"a/../b/../c".to_vec(),
    ];
    // Deterministic pseudo-random keys (LCG, no external deps).
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    for len in 0..64usize {
        let mut buf = vec![0u8; len * 4];
        for b in &mut buf {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *b = (state >> 33) as u8;
        }
        hostile.push(buf);
    }

    for material in &hostile {
        let k = KeyBuilder::new("fuzz")
            .push_bytes(material)
            .finish()
            .unwrap();
        n.put(&k, b"payload").unwrap();
        assert_eq!(n.get(&k).unwrap().as_deref(), Some(&b"payload"[..]));
    }

    let objects_root = Path::new(ROOT).join("ns/t/v1/objects");
    for file in files_under(&fs, Path::new("/")) {
        let s = file.to_string_lossy();
        if s.contains("/objects/") {
            // Under the one sanctioned objects directory…
            let rel = file
                .strip_prefix(&objects_root)
                .unwrap_or_else(|_| panic!("object escaped the namespace: {s}"));
            // …with exactly <2-hex>/<62-hex> components.
            let parts: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            assert_eq!(parts.len(), 2, "unexpected shape: {s}");
            assert_eq!(parts[0].len(), 2);
            assert_eq!(parts[1].len(), 62);
            assert!(
                parts
                    .iter()
                    .all(|p| p.bytes().all(|b| b.is_ascii_hexdigit())),
                "non-hex path component: {s}"
            );
        } else {
            // Everything else the store wrote is one of its fixed artifacts.
            assert!(
                s == format!("{ROOT}/STORE_FORMAT")
                    || s == format!("{ROOT}/STORE_OWNER")
                    || s.ends_with("/index")
                    || s.ends_with("/lock"),
                "unexpected file: {s}"
            );
        }
    }
}
