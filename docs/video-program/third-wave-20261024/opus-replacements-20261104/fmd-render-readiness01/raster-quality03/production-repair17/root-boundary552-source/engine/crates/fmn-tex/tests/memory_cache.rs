//! The production TeX cache works without filesystem capabilities and keeps
//! public mutable layouts, source spans, and semantic keys independent.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_tex::{MacroSet, Mode, Style, TYPESET_FORMAT_VERSION, TexEngine};
use std::path::Path;
use std::sync::Arc;

const MATH: Mode = Mode::Math(Style::Display);
const ROOT: &str = if cfg!(windows) { r"C:\cache" } else { "/cache" };

fn engine() -> TexEngine {
    TexEngine::new("fmd-math/pack/default", None).unwrap()
}

fn store() -> Store {
    store_with_fs().0
}

fn store_with_fs() -> (Store, Arc<VirtualFs>) {
    let fs = Arc::new(VirtualFs::new());
    let store = Store::open(
        fs.clone(),
        Arc::new(FakeClock::new()),
        ROOT,
        StoreConfig::default(),
    )
    .unwrap();
    (store, fs)
}

#[test]
fn no_store_hits_are_bit_identical_and_do_not_share_mutable_spans_or_layouts() {
    let e = engine();
    let source = r"\frac{a}{b} + \begin{pmatrix} x & y \\ z & w \end{pmatrix}";
    let mut cold = e.typeset(MATH, source).unwrap();
    let expected = cold.to_bytes().unwrap();
    let selection = cold.occurrences("x");
    // All three public containers may be edited by a caller. No mutation can
    // reach the immutable payload retained by the engine.
    cold.source.clear();
    cold.layout.glyphs.clear();
    cold.layout.paths.clear();
    cold.subs.clear();
    let warm = e.typeset(MATH, source).unwrap();
    assert_eq!(warm.to_bytes().unwrap(), expected);
    assert_eq!(warm.occurrences("x"), selection);
    let stats = e.memory_cache_stats();
    assert_eq!((stats.hits, stats.misses, stats.entries), (1, 1, 1));
    assert_eq!(stats.bytes, expected.len());
}

#[test]
fn memory_keys_separate_source_mode_style_and_macro_engines() {
    let e = engine();
    let modes = [
        Mode::Math(Style::Display),
        Mode::Math(Style::Text),
        Mode::Math(Style::Script),
        Mode::Math(Style::ScriptScript),
        Mode::Text,
    ];
    for source in ["x", "y"] {
        for mode in modes {
            let cold = e.typeset(mode, source).unwrap().to_bytes().unwrap();
            assert_eq!(e.typeset(mode, source).unwrap().to_bytes().unwrap(), cold);
        }
    }
    let stats = e.memory_cache_stats();
    assert_eq!((stats.hits, stats.misses, stats.entries), (10, 10, 10));
    let mut macros = MacroSet::new();
    macros.define("half", 1, r"\frac{#1}{2}").unwrap();
    let other = TexEngine::new("fmd-math/pack/default", Some(&macros)).unwrap();
    assert_ne!(e.cache_key(MATH, "x"), other.cache_key(MATH, "x"));
    assert_eq!(other.memory_cache_stats().entries, 0);
    assert_eq!(
        other.typeset(MATH, r"\half{x}").unwrap().layout.rules.len(),
        1
    );
    assert_eq!(other.memory_cache_stats().entries, 1);
    assert_eq!(e.memory_cache_stats(), stats);
}

#[test]
fn preflight_warms_memory_without_a_store_and_does_not_cache_failures() {
    let e = engine();
    let items = [(MATH, "x"), (MATH, "a^b^c"), (Mode::Text, "area $x^2$")];
    let outcomes = e.preflight(&items).unwrap();
    assert!(outcomes[0].is_ok());
    assert!(outcomes[1].is_err());
    assert!(outcomes[2].is_ok());
    assert_eq!(e.memory_cache_stats().entries, 2);
    for (mode, source) in [items[0], items[2]] {
        e.typeset(mode, source).unwrap();
    }
    let stats = e.memory_cache_stats();
    assert_eq!((stats.hits, stats.misses, stats.entries), (2, 3, 2));
    let error = e.typeset(MATH, "a^b^c").unwrap_err();
    assert_eq!(
        error.to_string(),
        outcomes[1].as_ref().unwrap_err().to_string()
    );
    assert_eq!(e.memory_cache_stats().misses, 4);
    assert_eq!(e.memory_cache_stats().entries, 2);
}

#[test]
fn disk_hits_promote_to_memory_and_attaching_a_store_populates_it() {
    let (store, fs) = store_with_fs();
    let e = engine();
    let source = r"\sqrt{x}";
    let expected = e.typeset(MATH, source).unwrap().to_bytes().unwrap();
    let e = e.with_cache(&store).unwrap();
    assert_eq!(e.memory_cache_stats().entries, 0);
    assert_eq!(
        e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    let ns = store
        .namespace(
            "typeset",
            TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let key = e.cache_key(MATH, source).unwrap();
    assert_eq!(ns.get(&key).unwrap().unwrap(), expected);

    let reader = engine().with_cache(&store).unwrap();
    assert_eq!(
        reader.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    // Remove the virtual disk store after promotion. The resident document
    // still serves the request, without a disk read or a new layout.
    fs.remove_dir_all(Path::new(ROOT)).unwrap();
    assert!(ns.get(&key).ok().flatten().is_none());
    assert_eq!(
        reader.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    let stats = reader.memory_cache_stats();
    assert_eq!((stats.hits, stats.misses, stats.entries), (1, 1, 1));
}

#[test]
fn clearing_the_memory_front_forces_a_fresh_layout_or_a_verified_store_hit() {
    let source = "q^7+z";
    // No store: a repeat request is a memory hit; after a clear it is a
    // fresh layout again, with the counters cumulative across the clear.
    let e = engine();
    let expected = e.typeset(MATH, source).unwrap().to_bytes().unwrap();
    e.typeset(MATH, source).unwrap();
    assert_eq!(e.layout_computations(), 1);
    assert_eq!(e.memory_cache_stats().hits, 1);
    e.clear_memory_cache();
    let cleared = e.memory_cache_stats();
    assert_eq!(
        (cleared.hits, cleared.misses, cleared.entries, cleared.bytes),
        (1, 1, 0, 0)
    );
    assert_eq!(
        e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    assert_eq!(e.layout_computations(), 2);
    assert_eq!(e.memory_cache_stats().hits, 1);

    // With a store: a cleared front is served by a verified store hit, not
    // by a layout, and the clear leaves the store's entry in place.
    let store = store();
    let e = engine().with_cache(&store).unwrap();
    e.typeset(MATH, source).unwrap();
    e.clear_memory_cache();
    assert_eq!(
        e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    assert_eq!(e.layout_computations(), 1);
    assert_eq!(e.persistent_cache_hits(), 1);
    assert_eq!(e.memory_cache_stats().hits, 0);
    // Without a clear the same request is a memory hit and the store is
    // never read: exactly what a store-latency probe must not time.
    e.typeset(MATH, source).unwrap();
    assert_eq!(e.persistent_cache_hits(), 1);
    assert_eq!(e.memory_cache_stats().hits, 1);
    assert!(e.persistent_cache_enabled());
}

#[test]
fn an_attached_store_that_can_neither_read_nor_write_counts_every_failure() {
    let source = "q^7+z";
    let expected = engine().typeset(MATH, source).unwrap().to_bytes().unwrap();
    let (store, fs) = store_with_fs();
    let e = engine().with_cache(&store).unwrap();
    // The namespace's object directory becomes a plain file: the store stays
    // attached (it opened fine) but can neither read nor publish an entry,
    // as with a cache directory that turned read-only or was replaced.
    let objects = Path::new(ROOT)
        .join("ns")
        .join("typeset")
        .join(format!("v{TYPESET_FORMAT_VERSION}"))
        .join("objects");
    fs.insert(objects, b"not a directory".to_vec());
    let got = e.typeset(MATH, source).unwrap();
    assert_eq!(
        got.to_bytes().unwrap(),
        expected,
        "a broken store never changes a layout"
    );
    assert!(e.persistent_cache_enabled());
    // One failed read and one failed write; nothing served, nothing stored,
    // and no entry was corrupt.
    assert_eq!(e.persistent_store_errors(), 2);
    assert_eq!(e.persistent_cache_hits(), 0);
    assert_eq!(e.persistent_bytes_written(), 0);
    assert_eq!(e.persistent_rejected_entries(), 0);
    assert_eq!(e.layout_computations(), 1);
    // The memory front serves the repeat without touching the store.
    e.typeset(MATH, source).unwrap();
    assert_eq!(e.persistent_store_errors(), 2);
    assert_eq!(e.memory_cache_stats().hits, 1);

    // A healthy store counts no failure: a miss is not an error.
    let healthy = engine().with_cache(&store_with_fs().0).unwrap();
    healthy.typeset(MATH, source).unwrap();
    assert_eq!(healthy.persistent_store_errors(), 0);
    assert_eq!(healthy.persistent_bytes_written(), expected.len() as u64);
}

#[test]
fn a_layout_over_the_store_ceiling_bypasses_retention_without_a_store_error() {
    let source = "q^7+z";
    let expected = engine().typeset(MATH, source).unwrap().to_bytes().unwrap();
    // The store's per-entry ceiling sits one byte below this layout: the
    // documented retention bypass, not a store that failed to write.
    let store = Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        ROOT,
        StoreConfig {
            max_entry_bytes: expected.len() - 1,
            ..StoreConfig::default()
        },
    )
    .unwrap();
    let e = engine().with_cache(&store).unwrap();
    assert_eq!(
        e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    assert_eq!(e.persistent_store_errors(), 0);
    assert_eq!(e.persistent_bytes_written(), 0);
    assert_eq!(e.persistent_rejected_entries(), 0);
    assert_eq!(e.layout_computations(), 1);
}

#[test]
fn corrupt_or_wrong_source_disk_documents_recompute_and_warm_memory() {
    let store = store();
    let e = engine().with_cache(&store).unwrap();
    let ns = store
        .namespace(
            "typeset",
            TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let x_key = e.cache_key(MATH, "x").unwrap();
    let y_key = e.cache_key(MATH, "y").unwrap();
    let other_source = engine().typeset(MATH, "z").unwrap().to_bytes().unwrap();
    ns.put(&x_key, &other_source).unwrap();
    ns.put(&y_key, b"not a typeset document").unwrap();
    for source in ["x", "y"] {
        let result = e.typeset(MATH, source).unwrap();
        assert_eq!(result.source, source);
        assert_eq!(result.layout.glyphs.len(), 1);
        assert_eq!(result.layout.glyphs[0].ch.to_string(), source);
        let expected = result.to_bytes().unwrap();
        assert_eq!(
            e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
            expected
        );
    }
    let stats = e.memory_cache_stats();
    assert_eq!((stats.hits, stats.misses, stats.entries), (2, 2, 2));
    // Both foreign documents sit in checksum-valid envelopes, so each was
    // rejected once, and the store refused to replace either incumbent with
    // the recomputed payload: a write failure, reported, never silent.
    assert_eq!(e.persistent_rejected_entries(), 2);
    assert_eq!(e.persistent_store_errors(), 2);
    assert_eq!(e.persistent_bytes_written(), 0);
}

#[test]
fn concurrent_requests_publish_one_payload_without_span_or_accounting_drift() {
    let e = engine();
    let source = r"\frac{x}{y} + \widehat{ab}";
    let expected = engine().typeset(MATH, source).unwrap().to_bytes().unwrap();
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..8 {
            let e = &e;
            workers.push(scope.spawn(move || e.typeset(MATH, source).unwrap().to_bytes().unwrap()));
        }
        for worker in workers {
            assert_eq!(worker.join().unwrap(), expected);
        }
    });
    let stats = e.memory_cache_stats();
    assert_eq!(stats.entries, 1);
    assert_eq!(stats.bytes, expected.len());
    assert_eq!(stats.hits + stats.misses, 8);
    assert_eq!(
        e.typeset(MATH, source).unwrap().to_bytes().unwrap(),
        expected
    );
    assert_eq!(e.memory_cache_stats().hits, stats.hits + 1);
}
