//! Production cache binding: actual layout counts, not timing thresholds.

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::VirtualFs;
use fmn_tex::{Mode, Style, TYPESET_FORMAT_VERSION, TexEngine};
use std::sync::Arc;

fn store() -> Store {
    Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        if cfg!(windows) { r"C:\cache" } else { "/cache" },
        StoreConfig::default(),
    )
    .unwrap()
}

fn engine() -> TexEngine {
    TexEngine::new("fmd-math/pack/default", None).unwrap()
}

#[test]
fn attaching_after_a_layout_populates_disk_and_a_new_engine_reads_it() {
    let store = store();
    let first = engine();
    let mode = Mode::Math(Style::Display);
    let source = r"\frac{x^2+1}{2}";
    let expected = first.typeset(mode, source).unwrap().to_bytes().unwrap();
    assert_eq!(first.layout_computations(), 1);
    first.set_cache(Some(&store)).unwrap();
    assert!(first.persistent_cache_enabled());
    assert_eq!(
        first.typeset(mode, source).unwrap().to_bytes().unwrap(),
        expected
    );
    let second = engine().with_cache(&store).unwrap();
    assert_eq!(
        second.typeset(mode, source).unwrap().to_bytes().unwrap(),
        expected
    );
    assert_eq!(
        second.layout_computations(),
        0,
        "disk hit must bypass the layout engine"
    );
    assert_eq!(second.persistent_cache_hits(), 1);
    second.typeset(mode, source).unwrap();
    assert_eq!(second.memory_cache_stats().hits, 1);
    assert_eq!(second.persistent_cache_hits(), 1);
}

#[test]
fn switching_stores_and_detaching_do_not_change_layout_or_write_to_old_store() {
    let old = store();
    let new = store();
    let engine = engine().with_cache(&old).unwrap();
    let mode = Mode::Math(Style::Display);
    let before = engine.typeset(mode, "x").unwrap().to_bytes().unwrap();
    engine.set_cache(Some(&new)).unwrap();
    assert_eq!(
        engine.typeset(mode, "x").unwrap().to_bytes().unwrap(),
        before
    );
    engine.typeset(mode, "y").unwrap();
    let old_ns = old
        .namespace(
            "typeset",
            TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let new_ns = new
        .namespace(
            "typeset",
            TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let y = engine.cache_key(mode, "y").unwrap();
    assert!(old_ns.get(&y).unwrap().is_none());
    assert!(new_ns.get(&y).unwrap().is_some());
    engine.set_cache(None).unwrap();
    assert!(!engine.persistent_cache_enabled());
    engine.typeset(mode, "z").unwrap();
    assert!(
        new_ns
            .get(&engine.cache_key(mode, "z").unwrap())
            .unwrap()
            .is_none()
    );
    assert!(
        old_ns
            .get(&engine.cache_key(mode, "x").unwrap())
            .unwrap()
            .is_some()
    );
}

#[test]
fn persisted_math_modes_and_preamble_expansions_keep_independent_keys() {
    let store = store();
    let first = engine().with_cache(&store).unwrap();
    let display = Mode::Math(Style::Display);
    let inline = Mode::Math(Style::Text);
    let source = r"\frac{1}{2}";
    let a = first.typeset(display, source).unwrap().to_bytes().unwrap();
    let b = first.typeset(inline, source).unwrap().to_bytes().unwrap();
    assert_ne!(a, b);
    let macro_source = r"\answer";
    let p = r"\newcommand{\answer}{x}";
    let q = r"\newcommand{\answer}{y}";
    let x = first
        .typeset_with_preamble(display, macro_source, p)
        .unwrap()
        .to_bytes()
        .unwrap();
    let y = first
        .typeset_with_preamble(display, macro_source, q)
        .unwrap()
        .to_bytes()
        .unwrap();
    assert_ne!(x, y);
    let second = engine().with_cache(&store).unwrap();
    assert_eq!(
        second.typeset(display, source).unwrap().to_bytes().unwrap(),
        a
    );
    assert_eq!(
        second.typeset(inline, source).unwrap().to_bytes().unwrap(),
        b
    );
    assert_eq!(
        second
            .typeset_with_preamble(display, macro_source, p)
            .unwrap()
            .to_bytes()
            .unwrap(),
        x
    );
    assert_eq!(
        second
            .typeset_with_preamble(display, macro_source, q)
            .unwrap()
            .to_bytes()
            .unwrap(),
        y
    );
    assert_eq!(second.layout_computations(), 0);
}

#[test]
fn concurrent_layouts_and_cache_rebinding_preserve_owned_results() {
    let old = store();
    let new = store();
    let engine = engine().with_cache(&old).unwrap();
    let mode = Mode::Math(Style::Display);
    let expected = engine.typeset(mode, "x^2+1").unwrap().to_bytes().unwrap();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let engine = &engine;
            let expected = &expected;
            scope.spawn(move || {
                for _ in 0..8 {
                    assert_eq!(
                        &engine.typeset(mode, "x^2+1").unwrap().to_bytes().unwrap(),
                        expected
                    );
                }
            });
        }
        engine.set_cache(Some(&new)).unwrap();
        engine.set_cache(None).unwrap();
    });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn refused_host_configuration_detaches_instead_of_retaining_the_previous_directory() {
    let store = store();
    let engine = engine().with_cache(&store).unwrap();
    assert!(
        engine
            .configure_host_cache(Some(if cfg!(windows) { r"C:\" } else { "/" }))
            .is_err()
    );
    assert!(!engine.persistent_cache_enabled());
    assert!(engine.typeset(Mode::Math(Style::Display), "x+1").is_ok());
}
