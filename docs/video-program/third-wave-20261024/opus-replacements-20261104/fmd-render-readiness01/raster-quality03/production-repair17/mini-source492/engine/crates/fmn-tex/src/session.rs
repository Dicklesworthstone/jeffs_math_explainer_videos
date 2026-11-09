//! Lazy, capability-bound typesetting for a single native scene run.
//!
//! Constructing a session does not load fonts or open files. The first actual
//! TeX request creates one engine, resolves its template, and attaches the
//! existing content-addressed store. Storage is optional; layout is not.

use std::cell::{Cell, OnceCell, RefCell};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use fmn_cache::{Store, StoreConfig, resolve_host_cache_root};
use fmn_config::{Config, PackRegistry};
use fmn_platform::clock::Clock;
use fmn_platform::fs::FileSystem;

use crate::{TexEngine, TexError, TypesetCacheStats, TypesetPreflightStats};

/// Actual work performed by a session, not part of certified scene identity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypesetSessionReport {
    /// Whether any caller requested an engine (shape-only scenes stay lazy).
    pub initialized: bool,
    /// Whether the engine has an attached persistent namespace.
    pub persistent: bool,
    /// A storage refusal leaves layout available and is reported here.
    pub cache_error: Option<String>,
    /// Engine-local memory-front counters and resident encoded bytes.
    pub memory: TypesetCacheStats,
    /// Checksum-verified persistent hits, excluding memory-front hits.
    pub persistent_hits: u64,
    /// Encoded bytes those persistent hits read.
    pub persistent_bytes_read: u64,
    /// Encoded bytes of fresh layouts published to the persistent cache.
    pub persistent_bytes_written: u64,
    /// Corrupt persistent entries detected, evicted, and recomputed.
    pub persistent_rejected: u64,
    /// Reads and writes the attached store failed (not misses, not corrupt
    /// entries). Nonzero means the store opened but is not caching: each
    /// failure degraded to the memory front and a fresh layout.
    pub persistent_store_errors: u64,
    /// Actual layouts, excluding engine-fingerprint probes. Every request
    /// that neither cache layer served is one layout: the run's misses.
    pub layout_computations: u64,
    /// Parallel preflight batches run on this session's engine.
    pub preflight: TypesetPreflightStats,
    /// Layouts completed before the scene's `construct` began: the work its
    /// front-door preflight moved ahead of construction. `None` when the
    /// front door never reached `construct`.
    pub layouts_before_construct: Option<u64>,
    /// Layouts completed before the scene's first frame (the first play,
    /// wait, or still); `None` when the scene produced no frame. Minus
    /// `layouts_before_construct`, this is what `construct` laid out itself.
    pub layouts_before_first_frame: Option<u64>,
    /// Layouts computed while a play/wait segment was driving frames: the
    /// typesetting a complete preflight leaves inside `play()`, ideally none.
    pub layouts_inside_segments: u64,
}

impl TypesetSessionReport {
    /// Requests served by either cache layer (memory front or verified disk).
    #[must_use]
    pub const fn cache_hits(&self) -> u64 {
        self.memory.hits.saturating_add(self.persistent_hits)
    }

    /// Requests no cache layer could serve, each of which was laid out.
    #[must_use]
    pub const fn cache_misses(&self) -> u64 {
        self.layout_computations
    }
}

/// Segment-boundary accounting, recorded by the scene front door.
#[derive(Default)]
struct SegmentAccounting {
    before_construct: Cell<Option<u64>>,
    before_first: Cell<Option<u64>>,
    open_since: Cell<Option<u64>>,
    inside: Cell<u64>,
}

struct CacheBinding {
    root: Result<Option<PathBuf>, String>,
    fs: Arc<dyn FileSystem>,
    clock: Arc<dyn Clock>,
}

/// One template, one engine and one cache binding for a native scene run.
///
/// The session belongs to the serial scene owner. Its engine can run the
/// existing scoped parallel preflight; no live scene or mutable callback is
/// sent to those workers. Separate sessions never share mutable macro state.
pub struct TexSession {
    template: String,
    cache: Option<CacheBinding>,
    engine: OnceCell<TexEngine>,
    cache_error: RefCell<Option<String>>,
    segments: SegmentAccounting,
}

impl TexSession {
    /// Filesystem-free authoring, retaining only the bounded memory cache.
    #[must_use]
    pub fn memory(template: impl Into<String>) -> Self {
        Self {
            template: template.into(),
            cache: None,
            engine: OnceCell::new(),
            cache_error: RefCell::new(None),
            segments: SegmentAccounting::default(),
        }
    }

    /// Use the resolved template and cache directory with explicit capabilities.
    ///
    /// Native filesystems use the same owned-root resolver as `fmn doctor` and
    /// `--clear-cache`. An injected filesystem requires an explicit absolute
    /// cache path: it never falls through to the ambient host filesystem,
    /// environment or working directory. Root ownership and entry validation
    /// remain fmn-cache's responsibility. Explicit host paths are anchored
    /// before scene code can change the working directory. The platform
    /// default is resolved on first use. An unavailable cache is nonfatal.
    #[must_use]
    pub fn with_cache(config: &Config, fs: Arc<dyn FileSystem>, clock: Arc<dyn Clock>) -> Self {
        Self {
            cache: Some(CacheBinding {
                root: cache_root(&config.directories.cache, fs.as_ref()),
                fs,
                clock,
            }),
            ..Self::memory(config.tex.template.clone())
        }
    }

    /// The same lazily initialized engine for preflight and real constructors.
    ///
    /// # Errors
    /// A template or bundled-font refusal retains its original [`TexError`].
    /// Storage errors are available in [`Self::report`], not substituted for
    /// typesetting errors and never a reason to emit blank geometry.
    pub fn engine(&self) -> Result<&TexEngine, TexError> {
        if let Some(engine) = self.engine.get() {
            return Ok(engine);
        }
        let registry = PackRegistry::builtin();
        let template = if self.template.is_empty() {
            "default"
        } else {
            &self.template
        };
        let pack = registry
            .resolve_template(template)
            .map_err(TexError::Pack)?;
        let engine = TexEngine::new(pack.content_id, None)?;
        if let Some(binding) = &self.cache {
            let result = open_store(binding).and_then(|store| {
                engine
                    .set_cache(Some(&store))
                    .map_err(|error| error.to_string())
            });
            if let Err(error) = result {
                *self.cache_error.borrow_mut() = Some(error);
            }
        }
        Ok(self.engine.get_or_init(|| engine))
    }

    /// Inspect counters without initializing the engine or touching the cache.
    ///
    /// A session whose engine was never needed reports exactly the default:
    /// no fonts, no cache, no typesetting, so no segment accounting either.
    #[must_use]
    pub fn report(&self) -> TypesetSessionReport {
        let Some(engine) = self.engine.get() else {
            return TypesetSessionReport::default();
        };
        let inside = self.segments.inside.get().saturating_add(
            self.segments.open_since.get().map_or(0, |start| {
                engine.layout_computations().saturating_sub(start)
            }),
        );
        TypesetSessionReport {
            initialized: true,
            persistent: engine.persistent_cache_enabled(),
            cache_error: self.cache_error.borrow().clone(),
            memory: engine.memory_cache_stats(),
            persistent_hits: engine.persistent_cache_hits(),
            persistent_bytes_read: engine.persistent_bytes_read(),
            persistent_bytes_written: engine.persistent_bytes_written(),
            persistent_rejected: engine.persistent_rejected_entries(),
            persistent_store_errors: engine.persistent_store_errors(),
            layout_computations: engine.layout_computations(),
            preflight: engine.preflight_stats(),
            layouts_before_construct: self.segments.before_construct.get(),
            layouts_before_first_frame: self.segments.before_first.get(),
            layouts_inside_segments: inside,
        }
    }

    /// Record that the scene's `construct` is about to run, after any
    /// front-door preflight. Only the first call counts.
    pub fn note_construct_begin(&self) {
        if self.segments.before_construct.get().is_none() {
            self.segments.before_construct.set(Some(self.layouts()));
        }
    }

    /// Record that the scene is about to produce its first frame.
    ///
    /// The scene front door calls this and the segment notes below at the
    /// runtime's lifecycle boundaries, so the report can prove where
    /// typesetting happened: before the first frame (the preflight's job) or
    /// inside a play/wait (dynamic strings no preflight could know). These
    /// notes are accounting only; they never initialize the engine.
    pub fn note_first_frame(&self) {
        if self.segments.before_first.get().is_none() {
            self.segments.before_first.set(Some(self.layouts()));
        }
    }

    /// Record that a play/wait segment begins driving frames.
    pub fn note_segment_begin(&self) {
        self.note_first_frame();
        let now = self.layouts();
        // A segment that ended without its notification is closed here.
        self.close_segment(now);
        self.segments.open_since.set(Some(now));
    }

    /// Record that the current segment finished.
    pub fn note_segment_end(&self) {
        self.close_segment(self.layouts());
    }

    fn close_segment(&self, now: u64) {
        if let Some(start) = self.segments.open_since.take() {
            self.segments.inside.set(
                self.segments
                    .inside
                    .get()
                    .saturating_add(now.saturating_sub(start)),
            );
        }
    }

    fn layouts(&self) -> u64 {
        self.engine.get().map_or(0, TexEngine::layout_computations)
    }
}

impl Default for TexSession {
    fn default() -> Self {
        Self::memory("default")
    }
}

fn cache_root(directory: &str, fs: &dyn FileSystem) -> Result<Option<PathBuf>, String> {
    if fs.grants_host_destructive_lifecycle() {
        return if directory.is_empty() {
            Ok(None)
        } else {
            resolve_host_cache_root(directory)
                .map(Some)
                .map_err(|error| error.to_string())
        };
    }
    let root = Path::new(directory);
    if !root.is_absolute() || root.components().any(|part| part == Component::ParentDir) {
        return Err(
            "injected typeset cache requires an explicit absolute directory without '..'"
                .to_owned(),
        );
    }
    Ok(Some(root.to_path_buf()))
}

fn open_store(binding: &CacheBinding) -> Result<Store, String> {
    let root = binding.root.as_ref().map_err(Clone::clone)?;
    let Some(root) = root else {
        return Store::open_host(
            Arc::clone(&binding.fs),
            Arc::clone(&binding.clock),
            "",
            StoreConfig::default(),
        )
        .map_err(|error| error.to_string());
    };
    Store::open(
        Arc::clone(&binding.fs),
        Arc::clone(&binding.clock),
        root,
        StoreConfig::default(),
    )
    .map_err(|error| error.to_string())
}
