//! The fm-fw6 acceptance suite, host half: two store openings (as two `fmn`
//! invocations would be) hammering one real directory through `StdFs` from
//! many threads — writers, readers, and maintainers — with the invariant that
//! **no observer ever sees corruption**: every get is a verified value or a
//! miss, every raw object file on disk is a complete, checksummed envelope
//! (no-clobber publication makes torn intermediates structurally impossible),
//! conflicting producers cannot replace an immutable keyed object, and
//! eviction racing writers never breaks either side.

use fmn_cache::{
    CacheClearAuthorization, CacheClearOutcome, CacheError, CacheKey, EvictOutcome, KeyBuilder,
    NamespacePolicy, RootRefusalCode, Store, StoreConfig,
};
use fmn_hash::{Limits, Reader, Schema, UnknownPolicy, sha256};
use fmn_platform::clock::StdClock;
use fmn_platform::fs::{FileSystem, StdFs};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};

/// The entry-envelope schema, re-declared from outside the crate: raw disk
/// bytes are validated against the *published* format, so an accidental
/// format change breaks this test deliberately.
const ENTRY_SCHEMA: Schema = Schema::new(*b"FMNC", 1, 1, 0);

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cache_{name}"));
    // Re-runnable: previous runs' state would perturb the assertions.
    let _ = StdFs.remove_dir_all(&dir);
    dir
}

fn open_result(root: &std::path::Path) -> Result<Store, CacheError> {
    Store::open(
        Arc::new(StdFs),
        Arc::new(StdClock::new()),
        root.to_path_buf(),
        StoreConfig::default(),
    )
}

fn open(root: &std::path::Path) -> Store {
    open_result(root).expect("open store")
}

fn key(i: usize) -> CacheKey {
    KeyBuilder::new("torture")
        .push_u64(i as u64)
        .finish()
        .expect("key")
}

// Every caller lives in the unix-cfg permission/corruption suites below, so
// Windows builds see the helper as dead code without this matching gate.
#[cfg(unix)]
fn object_path(
    root: &std::path::Path,
    namespace: &str,
    version: u32,
    cache_key: &CacheKey,
) -> PathBuf {
    let hex = cache_key.digest().to_hex();
    root.join("ns")
        .join(namespace)
        .join(format!("v{version}"))
        .join("objects")
        .join(&hex[..2])
        .join(&hex[2..])
}

/// The deterministic payload for key `i`: any cross-key contamination shows
/// up as a value mismatch even before checksums fire.
fn payload(i: usize) -> Vec<u8> {
    let mut v = vec![0u8; 64 + i];
    for (j, b) in v.iter_mut().enumerate() {
        *b = (i.wrapping_mul(31).wrapping_add(j) & 0xff) as u8;
    }
    v
}

#[test]
fn clear_rotates_the_generation_and_stale_handles_cannot_recreate_content() {
    let root = scratch("clear_lifecycle");
    let sibling = root.with_extension("sibling");
    std::fs::write(&sibling, b"keep").expect("sibling sentinel");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    let k = key(7);
    namespace.put(&k, &payload(7)).expect("seed cache");
    let owner_before = std::fs::read(root.join("STORE_OWNER")).expect("old owner manifest");

    let authorization = CacheClearAuthorization::authorize(&root).expect("owned root authorizes");
    assert_eq!(authorization.root(), root);
    assert_eq!(
        authorization.clear().expect("clear"),
        CacheClearOutcome::Cleared
    );
    assert!(root.join("STORE_OWNER").is_file());
    assert!(root.join("STORE_FORMAT").is_file());
    assert_ne!(
        std::fs::read(root.join("STORE_OWNER")).expect("new owner manifest"),
        owner_before,
        "clear rotates the root generation"
    );
    assert_eq!(std::fs::read(&sibling).expect("sibling survives"), b"keep");
    assert_eq!(namespace.get(&k).expect("stale read is a miss"), None);

    match namespace.put(&k, b"must not recreate") {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::GenerationChanged);
        }
        other => panic!("expected stale-generation refusal, got {other:?}"),
    }
    drop(namespace);
    assert!(
        !root.join("ns").exists(),
        "stale Drop cannot recreate the cleared namespace tree"
    );
    let reopened = open(&root);
    let fresh = reopened
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("fresh namespace");
    assert_eq!(fresh.get(&k).expect("fresh read"), None);
}

#[test]
fn clear_migrates_an_owned_old_format_even_when_namespaces_are_absent() {
    let root = scratch("clear_empty_old_format");
    drop(open(&root));
    let owner_before = std::fs::read(root.join("STORE_OWNER")).expect("old owner manifest");
    std::fs::write(root.join("STORE_FORMAT"), b"fmn-cache 999\n").expect("write old format");
    assert!(matches!(
        open_result(&root),
        Err(CacheError::FormatUnsupported { .. })
    ));

    assert_eq!(
        CacheClearAuthorization::authorize(&root)
            .expect("owned old-format root authorizes")
            .clear()
            .expect("clear migrates the format"),
        CacheClearOutcome::AlreadyAbsent
    );
    assert_eq!(
        std::fs::read(root.join("STORE_FORMAT")).expect("read refreshed format"),
        b"fmn-cache 1\n"
    );
    assert_ne!(
        std::fs::read(root.join("STORE_OWNER")).expect("read rotated owner manifest"),
        owner_before,
        "format migration rotates the root generation"
    );
    drop(open(&root));
}

#[test]
fn concurrent_first_openers_leave_one_valid_owned_root() {
    for attempt in 0..16 {
        let root = scratch(&format!("concurrent_first_open_{attempt}"));
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let root = root.clone();
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                open_result(&root)
            }));
        }
        barrier.wait();
        let results = handles
            .into_iter()
            .map(|handle| handle.join().expect("open thread"))
            .collect::<Vec<_>>();
        assert!(
            results.iter().any(Result::is_ok),
            "both first openers failed: {results:?}"
        );
        assert!(results.iter().all(|result| {
            result.is_ok() || matches!(result, Err(CacheError::RootRefused { .. }))
        }));
        let reopened = open(&root);
        // Store::open canonicalizes its root; on Windows that carries the
        // `\\?\` verbatim prefix, so compare canonical to canonical.
        assert_eq!(
            reopened.root(),
            std::fs::canonicalize(&root).expect("canonical scratch root")
        );
        assert!(root.join("STORE_OWNER").is_file());
        assert!(root.join("STORE_FORMAT").is_file());
    }
}

#[test]
fn concurrent_clearers_have_one_linearized_clear_and_one_miss() {
    let root = scratch("concurrent_clear");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    namespace.put(&key(3), &payload(3)).expect("seed cache");
    drop(namespace);
    drop(store);

    let first = CacheClearAuthorization::authorize(&root).expect("first authorization");
    let second = CacheClearAuthorization::authorize(&root).expect("second authorization");
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for authorization in [first, second] {
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            authorization.clear()
        }));
    }
    barrier.wait();
    let mut outcomes = handles
        .into_iter()
        .map(|handle| handle.join().expect("clear thread").expect("clear result"))
        .collect::<Vec<_>>();
    outcomes.sort_by_key(|outcome| match outcome {
        CacheClearOutcome::Cleared => 0,
        CacheClearOutcome::AlreadyAbsent => 1,
    });
    assert_eq!(
        outcomes,
        [CacheClearOutcome::Cleared, CacheClearOutcome::AlreadyAbsent]
    );
    assert!(root.join("STORE_OWNER").is_file());
    assert!(root.join("STORE_FORMAT").is_file());
}

#[test]
fn clear_racing_readers_and_writers_never_exposes_corruption() {
    const KEYS: usize = 12;
    const ROUNDS: usize = 32;

    let root = scratch("clear_racing_io");
    let store = open(&root);
    let namespace = Arc::new(
        store
            .namespace("shared", 1, NamespacePolicy::default())
            .expect("namespace"),
    );
    for i in 0..KEYS {
        namespace.put(&key(i), &payload(i)).expect("seed cache");
    }
    let authorization = CacheClearAuthorization::authorize(&root).expect("authorization");
    let barrier = Arc::new(Barrier::new(6));
    let mut handles = Vec::new();

    for writer in 0..4 {
        let namespace = Arc::clone(&namespace);
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            for round in 0..ROUNDS {
                let i = (round + writer * 5) % KEYS;
                let _ = namespace.put(&key(i), &payload(i));
            }
        }));
    }
    let reader_namespace = Arc::clone(&namespace);
    let reader_barrier = Arc::clone(&barrier);
    let reader = std::thread::spawn(move || {
        reader_barrier.wait();
        for round in 0..ROUNDS * 2 {
            let i = (round * 7) % KEYS;
            if let Some(found) = reader_namespace.get(&key(i)).expect("read is hit or miss") {
                assert_eq!(found, payload(i), "reader observed cross-key corruption");
            }
        }
    });

    barrier.wait();
    assert_eq!(
        authorization.clear().expect("clear"),
        CacheClearOutcome::Cleared
    );
    for handle in handles {
        handle.join().expect("writer thread");
    }
    reader.join().expect("reader thread");
    drop(namespace);
    drop(store);

    let reopened = open(&root);
    let fresh = reopened
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("reopen namespace");
    for i in 0..KEYS {
        if let Some(found) = fresh.get(&key(i)).expect("reopened read") {
            assert_eq!(found, payload(i), "reopened cache contains corruption");
        }
    }
}

#[test]
fn clear_never_reuses_or_overwrites_a_guessed_quarantine_name() {
    let root = scratch("quarantine_collision");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    namespace.put(&key(4), &payload(4)).expect("seed cache");
    drop(namespace);
    drop(store);

    let mut occupied = Vec::new();
    for sequence in 1..=128 {
        let path = root.join(format!(".fmn-clear.{}.{}", std::process::id(), sequence));
        std::fs::create_dir(&path).expect("occupy guessed quarantine name");
        occupied.push(path);
    }

    assert_eq!(
        CacheClearAuthorization::authorize(&root)
            .expect("authorization")
            .clear()
            .expect("clear"),
        CacheClearOutcome::Cleared
    );
    assert!(
        occupied.iter().all(|path| path.is_dir()),
        "a pre-existing quarantine path was overwritten"
    );
}

#[test]
fn clear_revalidates_the_owner_marker_before_renaming_namespaces() {
    let root = scratch("owner_revalidation");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    let cache_key = key(5);
    namespace.put(&cache_key, &payload(5)).expect("seed cache");
    drop(namespace);
    drop(store);

    let authorization = CacheClearAuthorization::authorize(&root).expect("authorization");
    std::fs::write(root.join("STORE_OWNER"), b"foreign owner\n").expect("replace owner marker");
    match authorization.clear() {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::MarkerInvalid);
        }
        other => panic!("expected marker refusal, got {other:?}"),
    }
    assert!(root.join("ns").is_dir(), "namespace tree must survive");
}

#[test]
fn clear_classifies_a_valid_generation_replacement_without_touching_namespaces() {
    let root = scratch("owner_generation_revalidation");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    namespace.put(&key(45), &payload(45)).expect("seed cache");
    drop(namespace);
    drop(store);

    let authorization = CacheClearAuthorization::authorize(&root).expect("authorization");
    let owner_path = root.join("STORE_OWNER");
    let marker = String::from_utf8(std::fs::read(&owner_path).expect("owner manifest"))
        .expect("owner manifest UTF-8");
    let generation_start = marker.rfind(' ').expect("generation separator") + 1;
    let mut replacement = marker;
    replacement.replace_range(generation_start..generation_start + 64, &"0".repeat(64));
    std::fs::write(&owner_path, replacement).expect("replace generation");

    match authorization.clear() {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::GenerationChanged);
        }
        other => panic!("expected generation refusal, got {other:?}"),
    }
    assert!(root.join("ns").is_dir(), "namespace tree must survive");
}

#[test]
fn replacing_the_owned_root_invalidates_stale_mutation_maintenance_and_drop() {
    let root = scratch("root_generation_replacement");
    let stale_store = open(&root);
    let stale = stale_store
        .namespace(
            "shared",
            1,
            NamespacePolicy {
                ceiling_bytes: Some(0),
            },
        )
        .expect("stale namespace");
    stale.put(&key(41), &payload(41)).expect("seed old root");

    let parked = scratch("root_generation_replacement_parked");
    std::fs::rename(&root, &parked).expect("park the complete old root");
    let fresh_store = open(&root);
    let fresh = fresh_store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("fresh namespace");
    fresh.put(&key(42), &payload(42)).expect("seed new root");
    let fresh_index = std::fs::read(root.join("ns/shared/v1/index")).expect("fresh index");

    for result in [
        stale.put(&key(43), &payload(43)),
        stale.evict_to_ceiling().map(|_| ()),
    ] {
        match result {
            Err(CacheError::RootRefused { code, .. }) => {
                assert_eq!(code, RootRefusalCode::GenerationChanged);
            }
            other => panic!("expected stale-root generation refusal, got {other:?}"),
        }
    }
    drop(stale);
    assert_eq!(
        std::fs::read(root.join("ns/shared/v1/index")).expect("index after stale Drop"),
        fresh_index,
        "stale Drop cannot merge old-root access state into the replacement root"
    );
    assert_eq!(
        fresh
            .get(&key(42))
            .expect("fresh value survives")
            .as_deref(),
        Some(payload(42).as_slice())
    );
    assert_eq!(fresh.get(&key(41)).expect("old value stays parked"), None);
}

#[test]
fn oversized_lifecycle_markers_are_bounded_typed_refusals() {
    let root = scratch("oversized_owner_manifest");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    std::fs::write(root.join("STORE_OWNER"), vec![b'x'; 257])
        .expect("replace owner with oversized marker");

    for result in [
        namespace.put(&key(44), &payload(44)),
        CacheClearAuthorization::authorize(&root).map(|_| ()),
    ] {
        match result {
            Err(CacheError::RootRefused { code, .. }) => {
                assert_eq!(code, RootRefusalCode::MarkerTooLarge);
            }
            other => panic!("expected bounded-marker refusal, got {other:?}"),
        }
    }

    let format_root = scratch("oversized_format_stamp");
    drop(open(&format_root));
    std::fs::write(format_root.join("STORE_FORMAT"), vec![b'x'; 65])
        .expect("replace format with oversized marker");
    for result in [
        open_result(&format_root).map(|_| ()),
        CacheClearAuthorization::authorize(&format_root).map(|_| ()),
    ] {
        match result {
            Err(CacheError::RootRefused { code, .. }) => {
                assert_eq!(code, RootRefusalCode::MarkerTooLarge);
            }
            other => panic!("expected bounded-format refusal, got {other:?}"),
        }
    }
}

#[cfg(unix)]
#[test]
fn clear_removes_a_symlinked_namespace_without_following_its_target() {
    let root = scratch("symlinked_namespace");
    let victim = scratch("symlinked_namespace_victim");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    namespace.put(&key(6), &payload(6)).expect("seed cache");
    drop(namespace);
    drop(store);

    std::fs::remove_dir_all(root.join("ns")).expect("replace managed namespace tree");
    std::fs::create_dir_all(&victim).expect("create sentinel target");
    let sentinel = victim.join("important.txt");
    std::fs::write(&sentinel, b"keep").expect("write sentinel");
    std::os::unix::fs::symlink(&victim, root.join("ns")).expect("symlink managed namespace");

    assert_eq!(
        CacheClearAuthorization::authorize(&root)
            .expect("authorization")
            .clear()
            .expect("clear"),
        CacheClearOutcome::Cleared
    );
    assert_eq!(std::fs::read(&sentinel).expect("target survives"), b"keep");
    assert!(!root.join("ns").exists());
}

#[test]
fn clear_authorization_refuses_dangerous_and_foreign_roots() {
    for dangerous in [
        PathBuf::from(std::path::MAIN_SEPARATOR.to_string()),
        std::env::current_dir().expect("cwd"),
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .expect("test home"),
    ] {
        assert!(
            matches!(
                CacheClearAuthorization::authorize(&dangerous),
                Err(CacheError::RootRefused { .. })
            ),
            "dangerous root authorized: {}",
            dangerous.display()
        );
    }

    let foreign = scratch("foreign_root");
    std::fs::create_dir_all(&foreign).expect("foreign dir");
    std::fs::write(foreign.join("important.txt"), b"keep").expect("foreign sentinel");
    assert!(matches!(
        CacheClearAuthorization::authorize(&foreign),
        Err(CacheError::RootRefused { .. })
    ));
    assert_eq!(
        std::fs::read(foreign.join("important.txt")).expect("foreign data survives"),
        b"keep"
    );

    let copied_from = scratch("copied_owner_source");
    drop(open(&copied_from));
    let copied_to = scratch("copied_owner_target");
    std::fs::create_dir(&copied_to).expect("copied-marker target");
    std::fs::copy(
        copied_from.join("STORE_OWNER"),
        copied_to.join("STORE_OWNER"),
    )
    .expect("copy owner marker");
    std::fs::copy(
        copied_from.join("STORE_FORMAT"),
        copied_to.join("STORE_FORMAT"),
    )
    .expect("copy format marker");
    std::fs::write(copied_to.join("important.txt"), b"keep").expect("copied-root sentinel");
    match CacheClearAuthorization::authorize(&copied_to) {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::OwnershipMismatch);
        }
        other => panic!("expected copied-owner refusal, got {other:?}"),
    }
    assert_eq!(
        std::fs::read(copied_to.join("important.txt")).expect("copied-root sentinel survives"),
        b"keep"
    );

    let empty_foreign = scratch("empty_foreign_root");
    std::fs::create_dir_all(&empty_foreign).expect("empty foreign dir");
    assert!(matches!(
        open_result(&empty_foreign),
        Err(CacheError::RootRefused { .. })
    ));
    assert!(empty_foreign.is_dir());
    assert!(
        std::fs::read_dir(&empty_foreign)
            .expect("empty foreign root remains readable")
            .next()
            .is_none()
    );

    assert!(matches!(
        CacheClearAuthorization::authorize("relative/cache"),
        Err(CacheError::RootRefused { .. })
    ));

    let absent = scratch("absent_clear_root");
    assert_eq!(
        CacheClearAuthorization::authorize(&absent)
            .expect("absent root is a non-creating authorization")
            .clear()
            .expect("absent clear"),
        CacheClearOutcome::AlreadyAbsent
    );
    assert!(!absent.exists(), "absent clear must not create its target");

    let missing_parent = scratch("missing_store_parent");
    let nested = missing_parent.join("cache");
    assert!(matches!(
        open_result(&nested),
        Err(CacheError::RootRefused { .. })
    ));
    assert!(
        !missing_parent.exists(),
        "a refused missing parent must not be created"
    );
}

#[cfg(unix)]
#[test]
fn clear_authorization_refuses_a_symlinked_root() {
    let root = scratch("symlink_target");
    let _store = open(&root);
    let alias = root.with_extension("alias");
    let _ = std::fs::remove_file(&alias);
    std::os::unix::fs::symlink(&root, &alias).expect("symlink alias");
    assert!(matches!(
        CacheClearAuthorization::authorize(&alias),
        Err(CacheError::RootRefused { .. })
    ));
    match open_result(&alias) {
        Err(CacheError::RootRefused { reason, .. }) => {
            assert!(reason.contains("symlinked path component"));
        }
        other => panic!("expected Store::open symlink refusal, got {other:?}"),
    }

    let real_parent = scratch("symlink_ancestor_parent");
    std::fs::create_dir(&real_parent).expect("create real parent");
    let nested_root = real_parent.join("cache");
    let _store = open(&nested_root);
    let parent_alias = real_parent.with_extension("alias");
    let _ = std::fs::remove_file(&parent_alias);
    std::os::unix::fs::symlink(&real_parent, &parent_alias).expect("symlink parent alias");
    match CacheClearAuthorization::authorize(parent_alias.join("cache")) {
        Err(CacheError::RootRefused { reason, .. }) => {
            assert!(reason.contains("symlinked path component"));
        }
        other => panic!("expected symlink-ancestor refusal, got {other:?}"),
    }
    match open_result(&parent_alias.join("new-cache")) {
        Err(CacheError::RootRefused { reason, .. }) => {
            assert!(reason.contains("symlinked path component"));
        }
        other => panic!("expected Store::open symlink-ancestor refusal, got {other:?}"),
    }
}

/// fm-macos-var-symlink-gate-aqr1: `$TMPDIR` on macOS is `/var/folders/...`,
/// and `/var` is the operating system's root-owned link to `/private/var`.
/// A cache root there opens persistently and clears under authorization;
/// the canonical root is the `/private` spelling.
#[cfg(target_os = "macos")]
#[test]
fn a_cache_root_under_the_macos_tmpdir_opens_persistently() {
    let root = std::env::temp_dir().join(format!("fmn-cache-tmpdir-{}", std::process::id()));
    assert!(
        root.starts_with("/var/"),
        "macOS TMPDIR is expected under /var: {}",
        root.display()
    );
    let store = open(&root);
    store
        .namespace("tmpdir", 1, NamespacePolicy::default())
        .expect("namespace")
        .put(&key(1), b"persistent")
        .expect("store a value");
    drop(store);
    let reopened = open(&root);
    let value = reopened
        .namespace("tmpdir", 1, NamespacePolicy::default())
        .expect("namespace")
        .get(&key(1))
        .expect("read back");
    assert_eq!(value.as_deref(), Some(&b"persistent"[..]));
    drop(reopened);
    CacheClearAuthorization::authorize(&root).expect("clear authorization under /var");
}

#[cfg(unix)]
#[test]
fn store_reopen_refuses_a_linked_ownership_marker() {
    let root = scratch("linked_owner_marker");
    drop(open(&root));
    let owner = root.join("STORE_OWNER");
    let parked = root.join("STORE_OWNER.real");
    std::fs::rename(&owner, &parked).expect("park real owner marker");
    let victim = root.join("important.txt");
    std::fs::copy(&parked, &victim).expect("prepare matching external bytes");
    std::os::unix::fs::symlink(&victim, &owner).expect("link owner marker");

    match open_result(&root) {
        Err(CacheError::RootRefused { code, .. }) => {
            assert_eq!(code, RootRefusalCode::WrongNodeKind);
        }
        other => panic!("opening must classify the marker rather than follow it: {other:?}"),
    }
    assert_eq!(
        std::fs::read(&victim).expect("victim survives"),
        std::fs::read(&parked).expect("parked marker")
    );
}

#[cfg(unix)]
#[test]
fn get_refuses_a_symlinked_shard_before_corruption_cleanup() {
    let root = scratch("get_symlinked_shard");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    let cache_key = key(31);
    namespace
        .put(&cache_key, &payload(31))
        .expect("seed object path");
    let object = object_path(&root, "shared", 1, &cache_key);
    let shard = object.parent().expect("shard").to_path_buf();
    std::fs::rename(&shard, shard.with_extension("parked")).expect("park real shard");

    let victim = scratch("get_symlinked_shard_victim");
    std::fs::create_dir(&victim).expect("create victim directory");
    let sentinel = victim.join(object.file_name().expect("object name"));
    std::fs::write(&sentinel, b"not a cache envelope").expect("write corrupt-looking sentinel");
    std::os::unix::fs::symlink(&victim, &shard).expect("replace shard with link");

    assert!(
        matches!(namespace.get(&cache_key), Err(CacheError::Storage(_))),
        "get must fail before reading or corrupt-cleaning through the link"
    );
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"not a cache envelope"
    );
}

#[cfg(unix)]
#[test]
fn get_refuses_a_linked_object_leaf_before_corruption_cleanup() {
    let root = scratch("get_linked_object");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    let cache_key = key(34);
    namespace
        .put(&cache_key, &payload(34))
        .expect("seed object");
    let object = object_path(&root, "shared", 1, &cache_key);
    std::fs::rename(&object, object.with_extension("parked")).expect("park real object");
    let victim = root.join("important.txt");
    std::fs::write(&victim, b"not a cache envelope").expect("write corrupt-looking sentinel");
    std::os::unix::fs::symlink(&victim, &object).expect("replace object with link");

    assert!(
        matches!(namespace.get(&cache_key), Err(CacheError::Storage(_))),
        "get must reject the linked leaf before corrupt-entry cleanup"
    );
    assert_eq!(
        std::fs::read(&victim).expect("sentinel survives"),
        b"not a cache envelope"
    );
}

#[cfg(unix)]
#[test]
fn put_refuses_a_symlinked_objects_ancestor() {
    let root = scratch("put_symlinked_objects");
    let store = open(&root);
    let namespace = store
        .namespace("shared", 1, NamespacePolicy::default())
        .expect("namespace");
    let cache_key = key(32);
    let object = object_path(&root, "shared", 1, &cache_key);
    let objects = root.join("ns/shared/v1/objects");
    std::fs::create_dir_all(objects.parent().expect("version directory"))
        .expect("create version directory");

    let victim = scratch("put_symlinked_objects_victim");
    let victim_shard = victim.join(
        object
            .parent()
            .expect("shard")
            .file_name()
            .expect("shard name"),
    );
    std::fs::create_dir_all(&victim_shard).expect("create victim shard");
    let sentinel = victim_shard.join(object.file_name().expect("object name"));
    std::fs::write(&sentinel, b"keep").expect("write target sentinel");
    std::os::unix::fs::symlink(&victim, &objects).expect("link objects ancestor");

    assert!(
        matches!(
            namespace.put(&cache_key, &payload(32)),
            Err(CacheError::Storage(_))
        ),
        "put must fail before publishing through the link"
    );
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"keep"
    );
}

#[cfg(unix)]
#[test]
fn eviction_refuses_a_symlinked_shard_without_enumerating_its_target() {
    let root = scratch("evict_symlinked_shard");
    let store = open(&root);
    let namespace = store
        .namespace(
            "shared",
            1,
            NamespacePolicy {
                ceiling_bytes: Some(0),
            },
        )
        .expect("namespace");
    let cache_key = key(33);
    namespace
        .put(&cache_key, &payload(33))
        .expect("seed object path");
    let object = object_path(&root, "shared", 1, &cache_key);
    let shard = object.parent().expect("shard").to_path_buf();
    std::fs::rename(&shard, shard.with_extension("parked")).expect("park real shard");

    let victim = scratch("evict_symlinked_shard_victim");
    std::fs::create_dir(&victim).expect("create victim directory");
    let sentinel = victim.join(object.file_name().expect("object name"));
    std::fs::write(&sentinel, b"keep").expect("write target sentinel");
    std::os::unix::fs::symlink(&victim, &shard).expect("replace shard with link");

    assert!(
        matches!(namespace.evict_to_ceiling(), Err(CacheError::Storage(_))),
        "eviction must fail before listing the linked shard"
    );
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"keep"
    );
}

#[cfg(unix)]
#[test]
fn namespace_open_refuses_a_symlinked_name_ancestor() {
    let root = scratch("namespace_name_symlink");
    let store = open(&root);
    std::fs::create_dir(root.join("ns")).expect("create namespace parent");
    let victim = scratch("namespace_name_symlink_victim");
    std::fs::create_dir(&victim).expect("create victim directory");
    let sentinel = victim.join("important.txt");
    std::fs::write(&sentinel, b"keep").expect("write sentinel");
    std::os::unix::fs::symlink(&victim, root.join("ns/shared")).expect("link namespace name");

    assert!(
        matches!(
            store.namespace("shared", 1, NamespacePolicy::default()),
            Err(CacheError::Storage(_))
        ),
        "namespace construction must reject an existing linked component"
    );
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"keep"
    );
}

#[cfg(unix)]
#[test]
fn opening_a_new_version_ignores_a_linked_sibling_version() {
    let root = scratch("ignored_symlinked_version");
    let store = open(&root);
    std::fs::create_dir_all(root.join("ns/shared")).expect("create namespace parent");
    let victim = scratch("ignored_symlinked_version_victim");
    std::fs::create_dir(&victim).expect("create victim directory");
    let sentinel = victim.join("important.txt");
    std::fs::write(&sentinel, b"keep").expect("write sentinel");
    let sibling = root.join("ns/shared/v1");
    std::os::unix::fs::symlink(&victim, &sibling).expect("link sibling version");

    store
        .namespace("shared", 2, NamespacePolicy::default())
        .expect("current version remains usable");
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"keep"
    );
    assert!(
        std::fs::symlink_metadata(&sibling)
            .expect("sibling link survives")
            .file_type()
            .is_symlink(),
        "opening one version never inspects or removes a sibling version"
    );
}

#[cfg(unix)]
#[test]
fn opening_a_new_version_never_preflights_sibling_descendants() {
    let root = scratch("ignored_sibling_descendant");
    let store = open(&root);
    let stale_objects = root.join("ns/shared/v1/objects");
    std::fs::create_dir_all(&stale_objects).expect("create real sibling version");
    let victim = scratch("ignored_sibling_descendant_victim");
    std::fs::create_dir(&victim).expect("create victim directory");
    let sentinel = victim.join("important.txt");
    std::fs::write(&sentinel, b"keep").expect("write sentinel");
    let linked = stale_objects.join("linked");
    std::os::unix::fs::symlink(&victim, &linked).expect("link sibling descendant");

    store
        .namespace("shared", 2, NamespacePolicy::default())
        .expect("current version remains usable");
    assert_eq!(
        std::fs::read(&sentinel).expect("sentinel survives"),
        b"keep"
    );
    assert!(
        root.join("ns/shared/v1").is_dir(),
        "opening v2 leaves the complete v1 tree intact"
    );
    assert!(
        std::fs::symlink_metadata(&linked)
            .expect("descendant link survives")
            .file_type()
            .is_symlink(),
        "no sibling descendant is traversed"
    );
}

const VERSION_CHILD_ROOT: &str = "FMN_CACHE_VERSION_CHILD_ROOT";
const VERSION_CHILD_VERSION: &str = "FMN_CACHE_VERSION_CHILD_VERSION";

#[test]
fn live_version_process_entry() {
    let Some(root) = std::env::var_os(VERSION_CHILD_ROOT).map(PathBuf::from) else {
        return;
    };
    let version = std::env::var(VERSION_CHILD_VERSION)
        .expect("child version")
        .parse::<u32>()
        .expect("numeric child version");
    let namespace = open(&root)
        .namespace("live-versions", version, NamespacePolicy::default())
        .expect("child namespace");
    namespace
        .put(
            &key(500),
            &payload(usize::try_from(version).expect("host-sized child version")),
        )
        .expect("child publication");

    // Model a process that terminates without running Namespace::drop. There
    // is no lease to abandon and no sibling opener may erase these objects.
    std::process::exit(0);
}

fn spawn_version_child(root: &std::path::Path, version: u32) -> std::process::Child {
    Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", "live_version_process_entry", "--nocapture"])
        .env(VERSION_CHILD_ROOT, root)
        .env(VERSION_CHILD_VERSION, version.to_string())
        .spawn()
        .expect("spawn version child")
}

#[test]
fn simultaneous_process_versions_survive_exit_and_reopen() {
    let root = scratch("simultaneous_live_versions");
    drop(open(&root));
    let mut v1_child = spawn_version_child(&root, 1);
    let mut v2_child = spawn_version_child(&root, 2);
    assert!(v1_child.wait().expect("wait for v1 child").success());
    assert!(v2_child.wait().expect("wait for v2 child").success());

    let v1 = open(&root)
        .namespace("live-versions", 1, NamespacePolicy::default())
        .expect("reopen v1");
    let v2 = open(&root)
        .namespace("live-versions", 2, NamespacePolicy::default())
        .expect("reopen v2");
    assert_eq!(
        v1.get(&key(500)).expect("read v1").as_deref(),
        Some(payload(1).as_slice())
    );
    assert_eq!(
        v2.get(&key(500)).expect("read v2").as_deref(),
        Some(payload(2).as_slice())
    );

    v1.put(&key(501), &payload(3)).expect("write reopened v1");
    v2.put(&key(501), &payload(4)).expect("write reopened v2");
    assert_eq!(
        v1.get(&key(501)).unwrap().as_deref(),
        Some(payload(3).as_slice())
    );
    assert_eq!(
        v2.get(&key(501)).unwrap().as_deref(),
        Some(payload(4).as_slice())
    );
}

#[test]
fn two_stores_many_threads_no_observer_ever_sees_corruption() {
    const KEYS: usize = 24;
    const ROUNDS: usize = 120;

    let root = scratch("torture");
    let store_a = open(&root);
    let store_b = open(&root);
    let policy = NamespacePolicy {
        // Small enough that eviction churns constantly under the writers.
        ceiling_bytes: Some(6 * 1024),
    };
    let ns_a = Arc::new(store_a.namespace("shared", 1, policy).unwrap());
    let ns_b = Arc::new(store_b.namespace("shared", 1, policy).unwrap());

    let stop = Arc::new(AtomicBool::new(false));
    let barrier = Arc::new(Barrier::new(7));
    let mut handles = Vec::new();

    // Four writers, two per store opening, all cycling over the same keys.
    for (w, ns) in [(0, &ns_a), (1, &ns_a), (2, &ns_b), (3, &ns_b)] {
        let ns = Arc::clone(ns);
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            for round in 0..ROUNDS {
                let i = (round * 7 + w * 3) % KEYS;
                // Storage failures under racing eviction are legal (a lost
                // cache write is a future recompute); corruption is not.
                let _ = ns.put(&key(i), &payload(i));
            }
        }));
    }

    // Two readers: every hit must be the exact expected payload.
    for ns in [&ns_a, &ns_b] {
        let ns = Arc::clone(ns);
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            while !stop.load(Ordering::Relaxed) {
                for i in 0..KEYS {
                    match ns.get(&key(i)) {
                        Ok(Some(v)) => {
                            assert_eq!(v, payload(i), "cross-contamination at key {i}");
                        }
                        Ok(None) => {}
                        Err(err) => panic!("reader hit a hard error: {err}"),
                    }
                }
            }
        }));
    }

    // One maintainer per… one is plenty; the second store's maintainer runs
    // implicitly via the skip path in the sibling test below. Here: evict in
    // a loop while writers churn.
    {
        let ns = Arc::clone(&ns_a);
        let barrier = Arc::clone(&barrier);
        let stop = Arc::clone(&stop);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            while !stop.load(Ordering::Relaxed) {
                match ns.evict_to_ceiling() {
                    Ok(EvictOutcome::Done(_) | EvictOutcome::SkippedLockHeld) => {}
                    Ok(EvictOutcome::Unlimited) => panic!("policy has a ceiling"),
                    Err(err) => panic!("maintainer hit a hard error: {err}"),
                }
            }
        }));
    }

    // Writers finish first; then release the loopers.
    let mut writer_handles = handles;
    let looper_handles = writer_handles.split_off(4);
    for h in writer_handles {
        h.join().expect("writer thread");
    }
    stop.store(true, Ordering::Relaxed);
    for h in looper_handles {
        h.join().expect("looper thread");
    }

    // Post-conditions. Every raw object file on disk is a complete, valid
    // envelope — no-clobber publication left nothing torn, eviction left no
    // half-deleted state.
    let objects_dir = root.join("ns/shared/v1/objects");
    let fs = StdFs;
    if fs.exists(&objects_dir) {
        for shard in fs.list_dir(&objects_dir).expect("list shards") {
            for file in fs.list_dir(&shard).expect("list shard") {
                let name = file.file_name().unwrap().to_string_lossy().into_owned();
                if name.starts_with(".fmn-") {
                    // In-flight temp from the final instants of the run;
                    // invisible to the store, swept by future maintenance.
                    continue;
                }
                let bytes = fs.read(&file).expect("read object");
                let mut r =
                    Reader::open(&bytes, ENTRY_SCHEMA, Limits::DEFAULT, UnknownPolicy::Strict)
                        .unwrap_or_else(|err| panic!("torn or corrupt object {name}: {err}"));
                let _kind = r.get_u8().expect("kind");
                let _address = r.get_digest().expect("address");
                let _payload = r.get_bytes().expect("payload");
                r.finish().expect("clean tail");
            }
        }
    }

    // And the store still works end-to-end from both openings.
    for i in 0..KEYS {
        ns_a.put(&key(i), &payload(i)).expect("final put");
    }
    for i in 0..KEYS {
        assert_eq!(
            ns_b.get(&key(i)).expect("final get").as_deref(),
            Some(payload(i).as_slice()),
            "opening B reads what opening A wrote"
        );
    }
}

const CONFLICT_CHILD_ROOT: &str = "FMN_CACHE_CONFLICT_CHILD_ROOT";
const CONFLICT_CHILD_ROLE: &str = "FMN_CACHE_CONFLICT_CHILD_ROLE";

#[test]
fn keyed_conflict_subprocess_entry() {
    let Some(root) = std::env::var_os(CONFLICT_CHILD_ROOT).map(PathBuf::from) else {
        return;
    };
    let role = std::env::var(CONFLICT_CHILD_ROLE).expect("child role");
    let namespace = open(&root)
        .namespace("race", 1, NamespacePolicy::default())
        .expect("child namespace");
    let cache_key = key(0);

    match role.as_str() {
        "incumbent" => namespace
            .put(&cache_key, &payload(1))
            .expect("first process publishes the immutable object"),
        "conflicting" => match namespace.put(&cache_key, &payload(2)) {
            Err(CacheError::KeyConflict(conflict)) => {
                assert_eq!(conflict.namespace, "race");
                assert_eq!(conflict.version, 1);
                assert_eq!(conflict.key, *cache_key.digest());
                assert_eq!(conflict.incumbent_payload, sha256(&payload(1)));
                assert_eq!(conflict.offered_payload, sha256(&payload(2)));
            }
            other => panic!("expected cross-process key conflict, got {other:?}"),
        },
        other => panic!("unknown conflict-child role {other:?}"),
    }
}

fn run_conflict_child(root: &std::path::Path, role: &str) {
    let output = Command::new(std::env::current_exe().expect("current test executable"))
        .args(["--exact", "keyed_conflict_subprocess_entry", "--nocapture"])
        .env(CONFLICT_CHILD_ROOT, root)
        .env(CONFLICT_CHILD_ROLE, role)
        .output()
        .expect("run keyed-conflict child");
    assert!(
        output.status.success(),
        "{role} child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn different_processes_cannot_replace_one_keyed_object() {
    let root = scratch("same_key_processes");
    run_conflict_child(&root, "incumbent");
    run_conflict_child(&root, "conflicting");

    let namespace = open(&root)
        .namespace("race", 1, NamespacePolicy::default())
        .expect("reopened namespace");
    assert_eq!(
        namespace.get(&key(0)).expect("final get").as_deref(),
        Some(payload(1).as_slice()),
        "the first process remains the immutable winner"
    );
}
