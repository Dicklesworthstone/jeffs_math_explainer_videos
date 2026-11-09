//! Bounded admission of a scene's declared native constructor requests.

use std::fmt;
use std::num::NonZeroUsize;

use fmn_tex::{
    Mode, PreflightError, Style, TEX_PREAMBLE_MAX_BYTES, TEX_PREAMBLE_SOURCE_MAX_BYTES, TexSession,
    TypesetRequest,
};

use crate::SceneConstruct;

pub(crate) const DEFAULT_PREFLIGHT_WORKERS: NonZeroUsize = NonZeroUsize::new(32).unwrap();
const MAX_REQUESTS: usize = 4096;
const MAX_BATCH_BYTES: usize = 4 * 1024 * 1024;

/// A native scene's preflight budget or batch-infrastructure refusal.
/// Individual formula failures retain their original native `TexError` instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TexPreflightError {
    /// Admission failed before any engine initialization or cache access.
    Limit {
        /// The bounded resource: requests, source bytes, or preamble bytes.
        resource: &'static str,
        /// Input index for a per-request refusal; None for the whole batch.
        request: Option<usize>,
        /// Required amount observed at the admission boundary.
        needed: usize,
        /// Maximum admitted amount.
        limit: usize,
    },
    /// The existing engine could not reserve the ordered outcome storage.
    Infrastructure(PreflightError),
}

impl fmt::Display for TexPreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit {
                resource,
                request,
                needed,
                limit,
            } => {
                write!(f, "native Tex preflight {resource}")?;
                if let Some(index) = request {
                    write!(f, " (request {index})")?;
                }
                write!(f, " needs {needed}, limit {limit}")
            }
            Self::Infrastructure(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TexPreflightError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Limit { .. } => None,
            Self::Infrastructure(error) => Some(error),
        }
    }
}

fn admission_error(
    resource: &'static str,
    request: Option<usize>,
    needed: usize,
    limit: usize,
) -> crate::Error {
    TexPreflightError::Limit {
        resource,
        request,
        needed,
        limit,
    }
    .into()
}

pub(crate) fn preflight<P: SceneConstruct + ?Sized>(
    program: &P,
    session: &TexSession,
    max_workers: NonZeroUsize,
) -> crate::Result<()> {
    warm_requests(&program.tex_preflight(), session, max_workers)
}

impl crate::Stage<'_> {
    /// Warm formulas discovered during construction on this scene's engine.
    ///
    /// Call this before building their `Tex`/`TexText` mobjects or starting the
    /// play that consumes them. Unlike the static `SceneConstruct` manifest,
    /// the sources may be generated from data already loaded by `construct`.
    /// The batch does not advance scene time, invoke updaters or emit frames.
    ///
    /// Requests must match the constructors' source, mode, preamble and line
    /// alignment. Layouts share the bounded memory front and verified persistent
    /// cache used by `tex_engine`; no temporary engine or Python worker is used.
    /// More than the resident cache budget may evict earlier results, so this
    /// is a warm-up operation, not a promise that arbitrary batches stay pinned.
    /// `max_workers` is a ceiling (capped at 64), not a guaranteed thread count.
    /// An empty batch leaves even the fonts and persistent cache uninitialized.
    ///
    /// # Errors
    /// The same limits as the static manifest apply: 4096 requests, 4 MiB total,
    /// and the native per-source/preamble limits. Budget refusals occur before
    /// cache I/O. A formula failure retains its native error; the entire admitted
    /// batch has completed before the first ordered error is returned, leaving
    /// later valid requests warm. No scene state is rolled back on error.
    pub fn preflight_tex(
        &self,
        requests: &[TypesetRequest<'_>],
        max_workers: NonZeroUsize,
    ) -> crate::Result<()> {
        warm_requests(requests, self.typesetting, max_workers)
    }
}

fn warm_requests(
    requests: &[TypesetRequest<'_>],
    session: &TexSession,
    max_workers: NonZeroUsize,
) -> crate::Result<()> {
    if requests.is_empty() {
        return Ok(());
    }
    if requests.len() > MAX_REQUESTS {
        return Err(admission_error(
            "requests",
            None,
            requests.len(),
            MAX_REQUESTS,
        ));
    }
    let mut bytes = 0usize;
    for (index, request) in requests.iter().enumerate() {
        if request.source.len() > TEX_PREAMBLE_SOURCE_MAX_BYTES {
            return Err(admission_error(
                "source bytes",
                Some(index),
                request.source.len(),
                TEX_PREAMBLE_SOURCE_MAX_BYTES,
            ));
        }
        if request.preamble.len() > TEX_PREAMBLE_MAX_BYTES {
            return Err(admission_error(
                "preamble bytes",
                Some(index),
                request.preamble.len(),
                TEX_PREAMBLE_MAX_BYTES,
            ));
        }
        bytes = bytes
            .saturating_add(request.source.len())
            .saturating_add(request.preamble.len());
        if bytes > MAX_BATCH_BYTES {
            return Err(admission_error("batch bytes", None, bytes, MAX_BATCH_BYTES));
        }
    }

    // Only immutable typesetting requests enter these existing scoped workers.
    // The scene, callbacks, RNG and frame clock remain on the scene-owning thread.
    let engine = session.engine()?;
    let workers = NonZeroUsize::new(max_workers.get().min(64)).unwrap();
    let outcomes = engine
        .preflight_requests(requests, workers)
        .map_err(TexPreflightError::Infrastructure)?;
    for outcome in outcomes {
        // The complete batch has finished, so later valid requests are warmed
        // even when an earlier source failed. Surface the first native error.
        outcome?;
    }
    // Tex and TexText calibrate scene-unit scale with this same text-style
    // probe. Warm it too, so an ordinary build does not introduce a surprise
    // calibration layout after the declared preflight has completed.
    engine.typeset(Mode::Math(Style::Text), "0")?;
    Ok(())
}
