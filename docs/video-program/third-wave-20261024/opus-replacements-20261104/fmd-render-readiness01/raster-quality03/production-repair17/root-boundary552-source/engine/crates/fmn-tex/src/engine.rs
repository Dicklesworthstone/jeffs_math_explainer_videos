//! The Tex engine: fmd-math behind the preamble pack, the content-addressed
//! typeset cache, and the pre-play preflight (§11.4–11.5).
//!
//! # The cache key (§14.4's contract, made structural)
//!
//! A typeset result is cached under the digest of its **complete semantic
//! inputs**: the mode and style, the source string, the macro table's
//! canonical bytes (pack plus caller definitions — a pack edit re-typesets,
//! correctly), and the **engine fingerprint**. The fingerprint folds in
//! the engine's identity three ways:
//!
//! - **Font hashes**: the SHA-256 of every bundled face's exact bytes, so a
//!   glyph edit cold-starts the cache even where no probe draws that glyph.
//! - **Engine version**: the pinned `franken_markdown` revision recorded in
//!   `SUITE.lock` (fmd-math and fmd-font come from that one rev) and this
//!   crate's version, so a pin bump cold-starts the cache **by construction**
//!   even when the layout change it carries is one no probe touches.
//! - **Probe layouts**: a fixed probe set typeset at construction, a dozen
//!   constructs spanning every mechanism (glyph metrics, fractions, scripts,
//!   radicals, drawn delimiters, environments, stretchy bands), resolved to
//!   canonical path bytes. A semantic change that slips past both identities
//!   above (an unpinned local build, say) still shows up here.
//!
//! There is no manually-bumped version constant to forget. Cold and warm
//! are definitionally equivalent; the serialization codec round-trips
//! bit-for-bit (tested), so certified renders are cache-consistent per
//! §16.7. A change to this crate's own typeset encoding bumps
//! [`TYPESET_FORMAT_VERSION`], which opens a fresh namespace.
//!
//! # The preflight (§11.5 — PG-4's design mechanism)
//!
//! [`TexEngine::preflight`] typesets a batch of strings across a scoped
//! thread pool, warming the cache before the first `play()` — so cold
//! start pays typesetting once, in parallel, off the critical path, and
//! PG-7's cached-path lookups are the common case afterward. W9's scene
//! runtime walks the constructed scene and hands the static strings here
//! (the walk hook lands with fm-5xm/fm-39s); the mechanism, its
//! parallelism, and its cache-warming contract are this crate's and are
//! tested here. Errors are collected per string — a preflight never
//! aborts the batch (the failing string will fail again, precisely, at
//! construction time).

use crate::error::{PreflightError, TexError};
use crate::memory_cache::{MemoryCache, TypesetCacheStats};
use crate::request::TypesetRequest;
use crate::typeset::{KEYWORD_INK_COMMANDS, Prim, TYPESET_FORMAT_VERSION, Typeset};
use fmd_math::{Layout, MacroSet, PathContour, Style};
use fmn_cache::{CacheKey, KeyBuilder, Namespace};
use fmn_config::{Config, PackRegistry};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

trait ScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope;
}

struct NativeScopedSpawner;

impl ScopedSpawner for NativeScopedSpawner {
    fn spawn<'scope, 'env: 'scope, F>(
        &self,
        scope: &'scope std::thread::Scope<'scope, 'env>,
        work: F,
    ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
    where
        F: FnOnce() + Send + 'scope,
    {
        std::thread::Builder::new().spawn_scoped(scope, work)
    }
}

type PreflightOutcome = Result<(), TexError>;
type PreflightSlot = Mutex<Option<PreflightOutcome>>;

/// Observed preflight work on one engine. Diagnostic only: worker counts and
/// wall time depend on the host and its load, so none of this is certified
/// scene identity, and none of it can change a layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TypesetPreflightStats {
    /// Non-empty preflight batches run on this engine.
    pub batches: u64,
    /// Requests submitted across those batches, each typeset (or served from
    /// the cache) exactly once.
    pub requests: u64,
    /// The most worker threads any one batch ran. When no thread could be
    /// started, the calling thread ran the batch and counts as one worker.
    pub workers: u64,
    /// The most workers in any one batch that typeset at least one request.
    pub active_workers: u64,
    /// Wall-clock nanoseconds across all batches. Zero on targets without a
    /// monotonic clock (wasm32), where preflight also runs on the caller.
    pub wall_ns: u64,
}

/// What one batch's scheduler actually did.
#[derive(Clone, Copy, Debug, Default)]
struct PreflightRun {
    workers: usize,
    active_workers: usize,
}

/// A monotonic stopwatch for preflight diagnostics; inert on wasm32, whose
/// `Instant::now` panics and whose preflight runs on the calling thread.
struct Stopwatch {
    #[cfg(not(target_arch = "wasm32"))]
    started: std::time::Instant,
}

impl Stopwatch {
    fn start() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            started: std::time::Instant::now(),
        }
    }

    fn elapsed_ns(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            u64::try_from(self.started.elapsed().as_nanos()).unwrap_or(u64::MAX)
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    }
}

/// How a string is typeset: mathematics at a style, or the TexText
/// text-mainland contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The `Tex` surface (whole string is mathematics).
    Math(Style),
    /// The `TexText` surface (text mainland with `$…$` islands).
    Text,
}

/// The Tex engine: fmd-math + the resolved preamble pack + the cache.
pub struct TexEngine {
    math: fmd_math::Engine,
    macros: MacroSet,
    /// The resolved pack's stable content id (for provenance/doctor).
    pack_content_id: &'static str,
    /// The engine fingerprint: sha-256 over the probe set's canonical
    /// bytes plus the macro table — the cache key's engine component.
    fingerprint: CacheKey,
    cache: Mutex<Option<Arc<Namespace>>>,
    memory_cache: Mutex<MemoryCache>,
    persistent_hits: AtomicU64,
    persistent_bytes_read: AtomicU64,
    persistent_bytes_written: AtomicU64,
    /// Verified envelopes whose payload did not decode to this request.
    persistent_payload_rejections: AtomicU64,
    /// Store reads and writes that failed (absence and corruption excluded).
    persistent_store_errors: AtomicU64,
    layout_computations: AtomicU64,
    preflight: Mutex<TypesetPreflightStats>,
}

impl core::fmt::Debug for TexEngine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TexEngine")
            .field("pack_content_id", &self.pack_content_id)
            .field("macros", &self.macros.len())
            .field("fingerprint", &self.fingerprint)
            .field("cached", &self.persistent_cache_enabled())
            .finish_non_exhaustive()
    }
}

impl TexEngine {
    /// An engine over the bundled faces with the given pack content id
    /// (`fmd-math/pack/default` etc. — the ids fmn-config's registry
    /// records) and optional caller macro definitions layered on top.
    ///
    /// # Errors
    ///
    /// [`TexError::Faces`] if the bundled faces fail to load (build
    /// corruption); [`TexError::UnknownPack`] if the content id names no
    /// pack (registry/pack drift — a wiring bug, reported precisely).
    pub fn new(pack_content_id: &'static str, extra: Option<&MacroSet>) -> Result<Self, TexError> {
        let math = fmd_math::Engine::bundled().map_err(|e| TexError::Faces {
            what: e.to_string(),
        })?;
        let mut macros = MacroSet::pack(pack_content_id).ok_or(TexError::UnknownPack {
            content_id: pack_content_id,
        })?;
        if let Some(extra) = extra {
            // Caller definitions layer over the pack, last wins.
            macros = merged(&macros, extra);
        }
        let fingerprint = fingerprint(&math, &macros);
        Ok(Self {
            math,
            macros,
            pack_content_id,
            fingerprint,
            cache: Mutex::new(None),
            memory_cache: Mutex::new(MemoryCache::default()),
            persistent_hits: AtomicU64::new(0),
            persistent_bytes_read: AtomicU64::new(0),
            persistent_bytes_written: AtomicU64::new(0),
            persistent_payload_rejections: AtomicU64::new(0),
            persistent_store_errors: AtomicU64::new(0),
            layout_computations: AtomicU64::new(0),
            preflight: Mutex::new(TypesetPreflightStats::default()),
        })
    }

    /// An engine wired from the typed config: `tex.template` resolves
    /// through the pack registry's compatibility mapping (an out-of-tier
    /// template is the registry's named refusal).
    ///
    /// # Errors
    ///
    /// [`TexError::Pack`] for template refusals, plus [`TexEngine::new`]'s.
    pub fn from_config(config: &Config, registry: &PackRegistry) -> Result<Self, TexError> {
        let pack = registry
            .resolve_template(&config.tex.template)
            .map_err(TexError::Pack)?;
        Self::new(pack.content_id, None)
    }

    /// Attach a cache namespace. The namespace version is the typeset
    /// serialization format's ([`TYPESET_FORMAT_VERSION`]); engine
    /// semantics live in the key's fingerprint instead, so a pin bump
    /// cold-starts without a namespace bump. Attaching a store resets the
    /// memory front so already-warm strings populate the newly attached store.
    ///
    /// # Errors
    ///
    /// [`TexError::Cache`] if the namespace cannot be opened.
    pub fn with_cache(self, store: &fmn_cache::Store) -> Result<Self, TexError> {
        self.set_cache(Some(store))?;
        Ok(self)
    }

    /// Attach, replace, or detach a store on an already-live engine.
    ///
    /// This is the production portal's configuration seam: its engines live
    /// in thread-local slots and may have typeset module-level objects before
    /// a Scene chooses its cache. No font, macro, fingerprint, or layout state
    /// changes. Each request retains its namespace across concurrent changes;
    /// neither layout nor filesystem I/O holds the configuration mutex.
    /// Detaching never removes cache files. The memory front is cleared so a
    /// subsequently requested formula populates a newly attached store.
    ///
    /// # Errors
    /// [`TexError::Cache`] if the new namespace cannot be opened. On that
    /// failure the prior binding is unchanged.
    pub fn set_cache(&self, store: Option<&fmn_cache::Store>) -> Result<(), TexError> {
        let next = store
            .map(|store| {
                store.namespace(
                    "typeset",
                    TYPESET_FORMAT_VERSION,
                    fmn_cache::NamespacePolicy::default(),
                )
            })
            .transpose()
            .map_err(|error| TexError::Cache {
                what: error.to_string(),
            })?
            .map(Arc::new);
        let previous = {
            let mut binding = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            std::mem::replace(&mut *binding, next)
        };
        // Namespace Drop may flush its advisory index. Never hold a cache
        // mutex across that filesystem work.
        drop(previous);
        *self
            .memory_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = MemoryCache::default();
        Ok(())
    }

    /// Configure the optional native-host disk cache without replacing this
    /// engine. An empty path uses FrankenManim's owned per-user cache leaf;
    /// `None` disables disk caching without deleting anything.
    ///
    /// Explicitly host-only: ordinary `new`/`from_config` stay filesystem-free,
    /// and capability-injected hosts use `set_cache` with their own Store.
    /// Callers may report storage failure and continue typesetting normally.
    ///
    /// # Errors
    /// [`TexError::Cache`] for an unavailable/refused host store. Unlike
    /// `set_cache`, this convenience method detaches an earlier binding on
    /// failure, so it cannot keep writing to a previously selected directory.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn configure_host_cache(&self, configured: Option<&str>) -> Result<(), TexError> {
        let Some(configured) = configured else {
            return self.set_cache(None);
        };
        let attached = fmn_cache::Store::open_native(configured, fmn_cache::StoreConfig::default())
            .map_err(|error| TexError::Cache {
                what: error.to_string(),
            })
            .and_then(|store| self.set_cache(Some(&store)));
        if attached.is_err() {
            self.set_cache(None)?;
        }
        attached
    }

    /// Whether a persistent namespace is attached. Storage may still become
    /// unavailable; individual failures remain cache misses, never blank ink.
    #[must_use]
    pub fn persistent_cache_enabled(&self) -> bool {
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// Verified disk hits since this engine was created. Diagnostic only;
    /// neither this counter nor scheduling-dependent cache state is certified.
    #[must_use]
    pub fn persistent_cache_hits(&self) -> u64 {
        self.persistent_hits.load(Ordering::Relaxed)
    }

    /// Actual layout attempts since creation, excluding fingerprint probes.
    /// A memory/disk hit does not increment this counter. Failed layouts do.
    #[must_use]
    pub fn layout_computations(&self) -> u64 {
        self.layout_computations.load(Ordering::Relaxed)
    }

    /// Encoded payload bytes served by verified disk hits since creation.
    #[must_use]
    pub fn persistent_bytes_read(&self) -> u64 {
        self.persistent_bytes_read.load(Ordering::Relaxed)
    }

    /// Encoded payload bytes of fresh layouts accepted by the attached store
    /// since creation (an identical incumbent from a concurrent writer counts:
    /// the entry is published either way).
    #[must_use]
    pub fn persistent_bytes_written(&self) -> u64 {
        self.persistent_bytes_written.load(Ordering::Relaxed)
    }

    /// Corrupt persistent entries detected on read: envelopes the currently
    /// attached namespace evicted, plus verified envelopes whose payload did
    /// not decode to the requested source. Each was served as a miss and
    /// recomputed; none was ever trusted. Diagnostic only.
    #[must_use]
    pub fn persistent_rejected_entries(&self) -> u64 {
        let envelopes = self
            .cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map_or(0, |ns| ns.rejected_entries());
        envelopes.saturating_add(self.persistent_payload_rejections.load(Ordering::Relaxed))
    }

    /// Persistent-store reads and writes that failed since creation: a store
    /// that opened but can no longer read or write (a read-only or replaced
    /// directory, a full disk), or a write refused because a different
    /// payload already holds the key. Absence is a miss and corruption is
    /// [`Self::persistent_rejected_entries`]; neither counts here, and an
    /// entry over the store's size ceiling is the documented retention
    /// bypass, not a failure. Every failure degrades to the memory front and
    /// a fresh layout, so a nonzero count means "attached, but not caching".
    /// Diagnostic only.
    #[must_use]
    pub fn persistent_store_errors(&self) -> u64 {
        self.persistent_store_errors.load(Ordering::Relaxed)
    }

    /// Preflight batches observed on this engine since creation.
    #[must_use]
    pub fn preflight_stats(&self) -> TypesetPreflightStats {
        *self
            .preflight
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// The resolved pack's content id (provenance, `fmn doctor`).
    #[must_use]
    pub fn pack_content_id(&self) -> &'static str {
        self.pack_content_id
    }

    /// The engine fingerprint (provenance / the input closure).
    #[must_use]
    pub fn fingerprint(&self) -> &CacheKey {
        &self.fingerprint
    }

    /// The cache key for one (mode, source) under this engine.
    ///
    /// `None` means the canonical key material exceeded its fixed format
    /// budget, so the value is deliberately uncacheable. No reduced key is
    /// substituted: dropping mode, source, or fingerprint identity could turn
    /// a cache optimization into a semantic collision.
    #[must_use]
    pub fn cache_key(&self, mode: Mode, source: &str) -> Option<CacheKey> {
        let (tag, style) = match mode {
            Mode::Math(Style::Display) => ("math", 0_u32),
            Mode::Math(Style::Text) => ("math", 1),
            Mode::Math(Style::Script) => ("math", 2),
            Mode::Math(Style::ScriptScript) => ("math", 3),
            Mode::Text => ("text", 0),
        };
        KeyBuilder::new("fmn-tex/typeset")
            .push_str(tag)
            .push_u32(style)
            .push_str(source)
            .push_digest(self.fingerprint.digest())
            .finish()
            .ok()
    }

    /// Diagnostics for the bounded engine-local cache. This layer is always
    /// available, including without a disk store or filesystem capability.
    #[must_use]
    pub fn memory_cache_stats(&self) -> TypesetCacheStats {
        self.memory_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stats()
    }

    /// Drop every payload resident in the engine-local memory front, so the
    /// next request for any source is served by the attached store or by a
    /// fresh layout. The store binding, its files, and every cumulative
    /// counter are unchanged; typesets already returned are caller-owned and
    /// unaffected. A latency probe uses this to time the layer it names
    /// rather than a memory hit.
    pub fn clear_memory_cache(&self) {
        self.memory_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }

    /// Typeset through the memory cache, then the optional disk cache. A
    /// verified hit reconstructs the layout and spans without re-layout;
    /// callers always receive independently mutable data. Preflight warms
    /// both layers. Oversized documents remain usable but bypass retention.
    /// Cache trouble degrades to computing, never to a blank render, and is
    /// counted ([`Self::persistent_store_errors`]).
    ///
    /// # Errors
    ///
    /// [`TexError::Math`]: the precise, named, tier-tagged construct
    /// errors surface at construction time — never a blank render.
    pub fn typeset(&self, mode: Mode, source: &str) -> Result<Typeset, TexError> {
        let Some(key) = self.cache_key(mode, source) else {
            return self.layout(mode, source);
        };
        // Release the lock before decoding, disk I/O, or layout. Distinct
        // preflight jobs remain parallel; only cache metadata is serialized.
        let resident = self
            .memory_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key);
        if let Some(bytes) = resident
            && let Ok(hit) = Typeset::from_bytes(&bytes)
            && hit.source == source
        {
            return Ok(hit);
        }
        let cache = self
            .cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(ns) = &cache {
            match ns.get(&key) {
                Ok(Some(bytes)) => match Typeset::from_bytes(&bytes) {
                    Ok(hit) if hit.source == source => {
                        self.persistent_hits.fetch_add(1, Ordering::Relaxed);
                        self.persistent_bytes_read
                            .fetch_add(bytes.len() as u64, Ordering::Relaxed);
                        self.remember(key, bytes);
                        return Ok(hit);
                    }
                    // A checksum-valid envelope with a foreign payload is
                    // never trusted either: count it, then recompute.
                    _ => {
                        self.persistent_payload_rejections
                            .fetch_add(1, Ordering::Relaxed);
                    }
                },
                Ok(None) => {}
                // Storage trouble is a miss for this request, but counted:
                // an attached store that cannot read is not silently "warm".
                Err(_) => {
                    self.persistent_store_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        let fresh = self.layout(mode, source)?;
        if let Ok(bytes) = fresh.to_bytes() {
            if let Some(ns) = &cache {
                match ns.put(&key, &bytes) {
                    Ok(()) => {
                        self.persistent_bytes_written
                            .fetch_add(bytes.len() as u64, Ordering::Relaxed);
                    }
                    // Oversized documents bypass retention by design.
                    Err(fmn_cache::CacheError::EntryTooLarge { .. }) => {}
                    Err(_) => {
                        self.persistent_store_errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            self.remember(key, bytes);
        }
        Ok(fresh)
    }

    fn remember(&self, key: CacheKey, bytes: Vec<u8>) {
        self.memory_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, bytes);
    }

    fn layout(&self, mode: Mode, source: &str) -> Result<Typeset, TexError> {
        self.layout_computations.fetch_add(1, Ordering::Relaxed);
        let layout = match mode {
            Mode::Math(style) => self
                .math
                .typeset_with_macros(source, style, &self.macros)
                .map_err(TexError::Math)?,
            Mode::Text => self
                .math
                .typeset_text_with_macros(source, &self.macros)
                .map_err(TexError::Math)?,
        };
        Typeset::from_borrowed(source, layout).map_err(TexError::from)
    }

    /// Resolve one submobject primitive into its closed quadratic
    /// contours — the span-preserving form of
    /// [`fmd_math::paths::resolve_paths`], which flattens the whole layout
    /// and would destroy the per-`Sub` grouping `TransformMatchingTex`
    /// consumes. The library tier builds one VMobject per `Sub` from
    /// these contours (fm-p5d); glyph resolution reuses the engine's
    /// pinned size/upm transform verbatim (a synthetic one-primitive
    /// layout through `resolve_paths`), rules arrive as rectangle
    /// contours, and drawn paths pass through positioned.
    ///
    /// The output is in ems, y-up, baseline at 0 — the same frame as the
    /// layout itself.
    ///
    /// # Errors
    ///
    /// [`TexError::BadPrim`] if `prim` indexes outside the typeset's
    /// primitive lists (a consumer wiring bug, named); [`TexError::Math`]
    /// if a glyph's outline fails to decode.
    pub fn resolve_prim(
        &self,
        typeset: &Typeset,
        prim: Prim,
    ) -> Result<Vec<PathContour>, TexError> {
        let layout = &typeset.layout;
        let single = match prim {
            Prim::Glyph(i) => Layout {
                glyphs: vec![layout.glyphs.get(i).cloned().ok_or(TexError::BadPrim {
                    what: format!("glyph {i} of {}", layout.glyphs.len()),
                })?],
                ..Layout::default()
            },
            Prim::Rule(i) => Layout {
                rules: vec![layout.rules.get(i).cloned().ok_or(TexError::BadPrim {
                    what: format!("rule {i} of {}", layout.rules.len()),
                })?],
                ..Layout::default()
            },
            Prim::Path(i) => Layout {
                paths: vec![layout.paths.get(i).cloned().ok_or(TexError::BadPrim {
                    what: format!("path {i} of {}", layout.paths.len()),
                })?],
                ..Layout::default()
            },
        };
        fmd_math::paths::resolve_paths(&self.math, &single).map_err(TexError::Math)
    }

    /// Warm the cache for a batch of strings, in parallel, before the
    /// first frame (§11.5). Returns per-string outcomes in input order;
    /// one failing string never aborts the batch.
    ///
    /// Worker availability affects only scheduling: every successfully
    /// started worker remains useful, and a refusal before the first worker
    /// falls back to the caller thread. No input is silently skipped.
    ///
    /// # Errors
    ///
    /// [`PreflightError::ResultStorageAllocationFailed`] if the complete
    /// ordered outcome cannot be reserved before cache-warming work starts.
    pub fn preflight(
        &self,
        items: &[(Mode, &str)],
    ) -> Result<Vec<Result<(), TexError>>, PreflightError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        let workers = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
            .min(items.len());
        self.preflight_with_spawner(items, workers, &NativeScopedSpawner)
    }

    /// Preflight complete constructor requests on the same bounded worker path.
    ///
    /// Preambles and text alignment use `typeset_aligned`, exactly as Tex and
    /// TexText construction do. Results stay in input order; a bad formula
    /// cannot prevent a later valid request from warming its cache. The worker
    /// limit is explicit and never changes layout, spans, or frame sampling.
    ///
    /// # Errors
    /// [`PreflightError::ResultStorageAllocationFailed`] if ordered results
    /// cannot be reserved before work starts.
    pub fn preflight_requests(
        &self,
        items: &[TypesetRequest<'_>],
        max_workers: std::num::NonZeroUsize,
    ) -> Result<Vec<Result<(), TexError>>, PreflightError> {
        let workers = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
            .min(max_workers.get());
        self.observed_preflight(items.len(), workers, &NativeScopedSpawner, &|index| {
            let item = items[index];
            self.typeset_aligned(item.mode, item.source, item.preamble, item.align)
                .map(|_| ())
        })
    }

    /// Run one batch and fold what its scheduler did into the diagnostics.
    fn observed_preflight<Spawner: ScopedSpawner, Job: Fn(usize) -> PreflightOutcome + Sync>(
        &self,
        count: usize,
        workers: usize,
        spawner: &Spawner,
        job: &Job,
    ) -> Result<Vec<PreflightOutcome>, PreflightError> {
        let stopwatch = Stopwatch::start();
        let (outcomes, run) = preflight_jobs(count, workers, spawner, job)?;
        if count > 0 {
            let wall_ns = stopwatch.elapsed_ns();
            let mut stats = self
                .preflight
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            stats.batches = stats.batches.saturating_add(1);
            stats.requests = stats.requests.saturating_add(count as u64);
            stats.workers = stats.workers.max(run.workers as u64);
            stats.active_workers = stats.active_workers.max(run.active_workers as u64);
            stats.wall_ns = stats.wall_ns.saturating_add(wall_ns);
        }
        Ok(outcomes)
    }

    fn preflight_with_spawner<Spawner>(
        &self,
        items: &[(Mode, &str)],
        workers: usize,
        spawner: &Spawner,
    ) -> Result<Vec<Result<(), TexError>>, PreflightError>
    where
        Spawner: ScopedSpawner,
    {
        self.observed_preflight(items.len(), workers, spawner, &|index| {
            let (mode, source) = items[index];
            self.typeset(mode, source).map(|_| ())
        })
    }
}

fn preflight_storage(
    items: usize,
) -> Result<(Vec<PreflightSlot>, Vec<PreflightOutcome>), PreflightError> {
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(items)
        .map_err(|_| PreflightError::ResultStorageAllocationFailed { items })?;
    for _ in 0..items {
        slots.push(Mutex::new(None));
    }

    let mut outcomes = Vec::new();
    outcomes
        .try_reserve_exact(items)
        .map_err(|_| PreflightError::ResultStorageAllocationFailed { items })?;
    Ok((slots, outcomes))
}

fn preflight_jobs<Spawner: ScopedSpawner, Job: Fn(usize) -> PreflightOutcome + Sync>(
    count: usize,
    workers: usize,
    spawner: &Spawner,
    job: &Job,
) -> Result<(Vec<PreflightOutcome>, PreflightRun), PreflightError> {
    if count == 0 {
        return Ok((Vec::new(), PreflightRun::default()));
    }
    let workers = workers.clamp(1, count);
    let next = AtomicUsize::new(0);
    let active = AtomicUsize::new(0);
    let (results, mut outcomes) = preflight_storage(count)?;
    let started = std::thread::scope(|scope| {
        let mut spawned = 0;
        for _ in 0..workers {
            let next = &next;
            let results = &results;
            let active = &active;
            if spawner
                .spawn(scope, move || preflight_worker(job, next, results, active))
                .is_err()
            {
                break;
            }
            spawned += 1;
        }
        if spawned == 0 {
            preflight_worker(job, &next, &results, &active);
            1
        } else {
            spawned
        }
    });
    for (index, slot) in results.into_iter().enumerate() {
        outcomes.push(
            slot.into_inner()
                .unwrap_or_else(PoisonError::into_inner)
                .unwrap_or_else(|| job(index)),
        );
    }
    let run = PreflightRun {
        workers: started,
        active_workers: active.into_inner(),
    };
    Ok((outcomes, run))
}

fn preflight_worker<Job: Fn(usize) -> PreflightOutcome + Sync>(
    job: &Job,
    next: &AtomicUsize,
    results: &[PreflightSlot],
    active: &AtomicUsize,
) {
    let mut typeset_any = false;
    loop {
        let index = next.fetch_add(1, Ordering::Relaxed);
        let Some(slot) = results.get(index) else {
            break;
        };
        if !typeset_any {
            typeset_any = true;
            active.fetch_add(1, Ordering::Relaxed);
        }
        let outcome = job(index);
        let mut slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
        *slot = Some(outcome);
    }
}

/// Layer `extra` over `base` (last wins), through canonical bytes: the
/// merged set is rebuilt definition-by-definition so validation and
/// canonical identity stay uniform.
fn merged(base: &MacroSet, extra: &MacroSet) -> MacroSet {
    // MacroSet has no direct iterator over bodies; canonical_bytes is the
    // exchange format. Parse it back: `name US params US body RS` records
    // after the version tag.
    let mut out = base.clone();
    let bytes = extra.canonical_bytes();
    let Some(tag_end) = bytes.iter().position(|&b| b == 0x1e) else {
        return out;
    };
    let mut rest = &bytes[tag_end + 1..];
    while let Some(rec_end) = rest.iter().position(|&b| b == 0x1e) {
        let rec = &rest[..rec_end];
        rest = &rest[rec_end + 1..];
        let mut fields = rec.split(|&b| b == 0x1f);
        let (Some(name), Some(params), Some(body)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if let (Ok(name), Some(&p), Ok(body)) = (
            core::str::from_utf8(name),
            params.first(),
            core::str::from_utf8(body),
        ) {
            // Definitions already validated on the way into `extra`.
            let _ = out.define(name, p.saturating_sub(b'0'), body);
        }
    }
    out
}

/// The engine fingerprint: canonical layout bytes of a fixed probe set
/// spanning every mechanism, plus the macro table's canonical bytes and the
/// engine's static identity ([`engine_identity`]).
fn fingerprint(math: &fmd_math::Engine, macros: &MacroSet) -> CacheKey {
    fingerprint_with_identity(math, macros, engine_identity())
}

/// [`fingerprint`] over an explicit static identity, so a test can prove a
/// pin bump or a font edit reaches every cache key.
fn fingerprint_with_identity(
    math: &fmd_math::Engine,
    macros: &MacroSet,
    identity: &[u8],
) -> CacheKey {
    /// Constructs chosen to touch every layout mechanism: glyph metrics
    /// and kerning, scripts and primes, fractions, radicals, big operators, accents,
    /// drawn delimiters past the ceiling, environments, stretchy bands,
    /// inner, vertical and diagonal dots, generated material, the Tex
    /// surface's align* rows, and text mode. A semantics change anywhere
    /// shows up here, so a layout change that no probe touches needs a
    /// probe that does.
    const PROBES: &[&str] = &[
        r"ax + b^2_c + f''(x)",
        r"\frac{1}{1+\frac{1}{x}}",
        r"\sqrt[3]{x+1}",
        r"\sum_{n=1}^{N} n \int_0^1 x\,dx",
        r"\hat x + \vec{v} + \overline{AB} + \big( x \big)",
        r"\left(\frac{\frac{1}{2}}{\frac{3}{4}}\right)",
        r"\begin{pmatrix} a & \cdots & b \\ \vdots & \ddots & \vdots \\ c & \cdots & d \end{pmatrix}",
        r"\begin{cases} x & x > 0 \\ -x & x \le 0 \end{cases}",
        r"\widehat{x+y} + \overbrace{a+b}",
        r"\mathbb{R} \mathrm{d} \mathbf{v}",
        r"1 + \cdots + n, \ldots \equiv k \pmod{p}",
        r"x + 1 &= y \\ &\approx 2 \\ z",
    ];
    /// TexText material: the text font's quote and dash ligatures, and the
    /// interword space after an inline island.
    const TEXT_PROBES: &[&str] = &["can't -- ``x'' --- $y$ z"];
    let mut material = Vec::new();
    let laid = PROBES
        .iter()
        .map(|probe| math.typeset(probe, Style::Display))
        .chain(TEXT_PROBES.iter().map(|probe| math.typeset_text(probe)));
    for layout in laid {
        match layout {
            Ok(layout) => {
                material.extend_from_slice(fmd_math::paths::layout_dump(&layout).as_bytes());
                if let Ok(contours) = fmd_math::paths::resolve_paths(math, &layout) {
                    material
                        .extend_from_slice(fmd_math::paths::canonical_dump(&contours).as_bytes());
                }
            }
            Err(e) => {
                // A probe that stops typesetting is itself a semantic
                // change; fold the error text in.
                material.extend_from_slice(e.to_string().as_bytes());
            }
        }
        material.push(0x1e);
    }
    material.extend_from_slice(&macros.canonical_bytes());
    // Typeset::new re-spans keyword ink after layout, so the probes above
    // cannot see that policy; fold it in so a change cold-starts the cache.
    material.extend_from_slice(b"keyword-ink:");
    material.extend_from_slice(KEYWORD_INK_COMMANDS.join(",").as_bytes());
    material.push(0x1e);
    material.extend_from_slice(identity);
    CacheKey::of_content(&material)
}

/// The pinned suite identity every typeset output depends on, as recorded in
/// `SUITE.lock` (the CI-enforced source of the Cargo git revs).
const SUITE_LOCK: &str = include_str!("../../../SUITE.lock");

/// The engine's static identity, computed once per process: the pinned
/// `franken_markdown` revision (fmd-math layout and fmd-font decoding), this
/// crate's version, and the SHA-256 of every bundled face's exact bytes.
/// Probes sample layout behaviour; this names the code and fonts outright,
/// so a pin bump or a font edit cold-starts the cache even when no probe
/// would have noticed.
fn engine_identity() -> &'static [u8] {
    static IDENTITY: OnceLock<Vec<u8>> = OnceLock::new();
    IDENTITY.get_or_init(|| {
        let mut identity = Vec::new();
        identity.extend_from_slice(b"fmd-rev:");
        identity.extend_from_slice(suite_revision(SUITE_LOCK, "franken_markdown").as_bytes());
        identity.extend_from_slice(b"\x1ffmn-tex:");
        identity.extend_from_slice(env!("CARGO_PKG_VERSION").as_bytes());
        for (name, bytes) in fmn_text::bundled_faces() {
            identity.push(0x1f);
            identity.extend_from_slice(name.as_bytes());
            identity.push(b'=');
            identity.extend_from_slice(CacheKey::of_content(bytes).digest().to_hex().as_bytes());
        }
        identity
    })
}

/// The pinned revision of one suite repository in `SUITE.lock`'s
/// tab-separated `name<TAB>rev<TAB>note` rows. A missing row folds the whole
/// lock in instead, which is still a correct (if over-eager) identity.
fn suite_revision<'a>(lock: &'a str, repository: &str) -> &'a str {
    lock.lines()
        .find_map(|line| {
            let mut fields = line.split('\t');
            (fields.next() == Some(repository))
                .then(|| fields.next())
                .flatten()
                .filter(|rev| !rev.is_empty())
        })
        .unwrap_or(lock)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use fmn_cache::{NamespacePolicy, Store, StoreConfig};
    use fmn_platform::clock::FakeClock;
    use fmn_platform::fs::VirtualFs;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct RefusingScopedSpawner {
        refuse_at: usize,
        attempts: AtomicUsize,
    }

    impl RefusingScopedSpawner {
        const fn new(refuse_at: usize) -> Self {
            Self {
                refuse_at,
                attempts: AtomicUsize::new(0),
            }
        }

        fn attempts(&self) -> usize {
            self.attempts.load(Ordering::Relaxed)
        }
    }

    impl ScopedSpawner for RefusingScopedSpawner {
        fn spawn<'scope, 'env: 'scope, F>(
            &self,
            scope: &'scope std::thread::Scope<'scope, 'env>,
            work: F,
        ) -> std::io::Result<std::thread::ScopedJoinHandle<'scope, ()>>
        where
            F: FnOnce() + Send + 'scope,
        {
            let attempt = self.attempts.fetch_add(1, Ordering::Relaxed);
            if attempt == self.refuse_at {
                return Err(std::io::ErrorKind::WouldBlock.into());
            }
            NativeScopedSpawner.spawn(scope, work)
        }
    }

    // Host path semantics validate the root even over a VirtualFs.
    const VIRTUAL_CACHE_ROOT: &str = if cfg!(windows) { r"C:\cache" } else { "/cache" };

    fn store() -> Store {
        Store::open(
            Arc::new(VirtualFs::new()),
            Arc::new(FakeClock::new()),
            VIRTUAL_CACHE_ROOT,
            StoreConfig::default(),
        )
        .expect("virtual cache store")
    }

    fn assert_refused_worker_still_warms_every_item(refuse_at: usize) {
        let store = store();
        let engine = TexEngine::new("fmd-math/pack/default", None)
            .expect("bundled engine")
            .with_cache(&store)
            .expect("cache namespace");
        let items = [
            (Mode::Math(Style::Display), "x + 1"),
            (Mode::Math(Style::Display), r"\frac{1}{2}"),
            (Mode::Text, r"area $\pi r^2$"),
            (Mode::Math(Style::Display), r"\sqrt{x + 1}"),
        ];
        let spawner = RefusingScopedSpawner::new(refuse_at);

        let outcomes = engine
            .preflight_with_spawner(&items, items.len(), &spawner)
            .expect("result storage");

        assert_eq!(spawner.attempts(), refuse_at.saturating_add(1));
        assert_eq!(outcomes.len(), items.len());
        assert!(outcomes.iter().all(Result::is_ok));
        let namespace = store
            .namespace(
                "typeset",
                TYPESET_FORMAT_VERSION,
                NamespacePolicy::default(),
            )
            .expect("typeset namespace");
        for (mode, source) in items {
            let key = engine
                .cache_key(mode, source)
                .expect("small test source has a canonical cache key");
            assert!(
                namespace.get(&key).expect("cache read").is_some(),
                "refused startup left {source:?} cold"
            );
        }
    }

    #[test]
    fn first_worker_refusal_falls_back_to_caller_and_warms_every_item() {
        assert_refused_worker_still_warms_every_item(0);
    }

    #[test]
    fn intermediate_worker_refusal_keeps_the_started_subset_semantic() {
        assert_refused_worker_still_warms_every_item(2);
    }

    #[test]
    fn empty_preflight_starts_no_workers() {
        let engine = TexEngine::new("fmd-math/pack/default", None).expect("bundled engine");
        let spawner = RefusingScopedSpawner::new(0);
        let outcomes = engine
            .preflight_with_spawner(&[], 4, &spawner)
            .expect("empty storage");
        assert!(outcomes.is_empty());
        assert_eq!(spawner.attempts(), 0);
    }

    #[test]
    fn engine_identity_names_the_pinned_layout_code_and_every_bundled_face() {
        let identity = std::str::from_utf8(engine_identity()).expect("identity is text");
        let rev = suite_revision(SUITE_LOCK, "franken_markdown");
        assert_eq!(
            rev.len(),
            40,
            "SUITE.lock pins franken_markdown to a full rev"
        );
        assert!(rev.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(identity.starts_with(&format!("fmd-rev:{rev}")));
        for (name, bytes) in fmn_text::bundled_faces() {
            let digest = CacheKey::of_content(bytes).digest().to_hex();
            assert!(
                identity.contains(&format!("{name}={digest}")),
                "face {name} is not hashed into the engine identity"
            );
        }
    }

    #[test]
    fn a_pin_bump_or_a_font_edit_changes_the_fingerprint_every_cache_key_carries() {
        let engine = TexEngine::new("fmd-math/pack/default", None).expect("bundled engine");
        let identity = engine_identity();
        // The production fingerprint is the identity-bearing one.
        assert_eq!(
            *engine.fingerprint(),
            fingerprint_with_identity(&engine.math, &engine.macros, identity)
        );
        let fingerprint_of =
            |identity: &[u8]| fingerprint_with_identity(&engine.math, &engine.macros, identity);
        // Another franken_markdown rev with the very same probe layouts (a
        // pin bump whose layout change no probe touches).
        let rev = suite_revision(SUITE_LOCK, "franken_markdown");
        let text = std::str::from_utf8(identity).expect("identity is text");
        let bumped = text.replacen(rev, &"0".repeat(rev.len()), 1);
        assert_ne!(bumped, text);
        assert_ne!(fingerprint_of(bumped.as_bytes()), *engine.fingerprint());
        // One edited bundled face: its digest is the last field.
        let mut edited = identity.to_vec();
        let last = edited.last_mut().expect("identity names the faces");
        *last = if *last == b'0' { b'1' } else { b'0' };
        assert_ne!(fingerprint_of(&edited), *engine.fingerprint());
        // (The fingerprint reaching every cache key is pinned in
        // tests/scribe2.rs, where a macro-table change separates keys.)
    }

    #[test]
    fn suite_revision_reads_the_named_row_and_never_a_reduced_identity() {
        let lock = "# comment\nfranken_numpy\tabc\tnote\nfranken_markdown\tdef123\tnote\n";
        assert_eq!(suite_revision(lock, "franken_markdown"), "def123");
        assert_eq!(suite_revision(lock, "franken_numpy"), "abc");
        // A missing row folds the whole lock in rather than an empty string.
        assert_eq!(suite_revision(lock, "frankentorch"), lock);
        assert_eq!(
            suite_revision("franken_markdown\t\tnote", "franken_markdown"),
            "franken_markdown\t\tnote"
        );
    }

    #[test]
    fn preflight_reports_its_workers_requests_and_wall_time() {
        let engine = TexEngine::new("fmd-math/pack/default", None).expect("bundled engine");
        assert_eq!(engine.preflight_stats(), TypesetPreflightStats::default());
        let items = [
            (Mode::Math(Style::Display), "a + b"),
            (Mode::Math(Style::Display), r"\frac{a}{b}"),
            (Mode::Math(Style::Display), r"\sqrt{a}"),
            (Mode::Math(Style::Display), r"a^{b^c}"),
        ];
        let outcomes = engine
            .preflight_with_spawner(&items, 3, &NativeScopedSpawner)
            .expect("result storage");
        assert!(outcomes.iter().all(Result::is_ok));
        let stats = engine.preflight_stats();
        assert_eq!((stats.batches, stats.requests, stats.workers), (1, 4, 3));
        assert!((1..=3).contains(&stats.active_workers));
        assert!(stats.wall_ns > 0, "a native batch has a measured wall time");

        // A batch whose every spawn is refused runs on the caller: one worker.
        let fallback = TexEngine::new("fmd-math/pack/default", None).expect("bundled engine");
        fallback
            .preflight_with_spawner(&items, 4, &RefusingScopedSpawner::new(0))
            .expect("result storage");
        let stats = fallback.preflight_stats();
        assert_eq!(
            (stats.workers, stats.active_workers, stats.requests),
            (1, 1, 4)
        );
        // Empty batches are not batches.
        fallback.preflight(&[]).expect("empty batch");
        assert_eq!(fallback.preflight_stats().batches, 1);
    }

    #[test]
    fn preflight_result_storage_refuses_capacity_overflow() {
        assert!(matches!(
            preflight_storage(usize::MAX),
            Err(PreflightError::ResultStorageAllocationFailed { items: usize::MAX })
        ));
    }
}
