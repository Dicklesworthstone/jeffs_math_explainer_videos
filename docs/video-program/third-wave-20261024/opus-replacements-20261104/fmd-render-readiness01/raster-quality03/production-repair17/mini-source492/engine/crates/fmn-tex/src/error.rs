//! Scribe II's error surface: fmd-math's precise construct errors pass
//! through untouched (the user sees `` `\substack` is not yet supported;
//! tier T2, tracked at … `` at construction time — never a blank render),
//! and everything else is equally named.

use crate::typeset::TypesetError;
use core::fmt;

/// A batch-level typesetting-preflight failure.
///
/// Per-string parser/layout failures remain [`TexError`] values in the
/// ordered preflight outcome. This type is reserved for infrastructure that
/// prevents the batch from owning that outcome at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightError {
    /// The complete ordered result storage could not be reserved before work.
    ResultStorageAllocationFailed {
        /// Number of input strings whose outcomes were requested.
        items: usize,
    },
}

impl fmt::Display for PreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResultStorageAllocationFailed { items } => write!(
                f,
                "typesetting preflight could not reserve result storage for {items} items"
            ),
        }
    }
}

impl std::error::Error for PreflightError {}

/// A Tex/TexText failure.
#[derive(Debug)]
pub enum TexError {
    /// fmd-math refused the string: unsupported construct (named,
    /// tier-tagged, tracked), malformed input (byte-positioned), or an
    /// unmapped character. The Display text is the ratchet's stable
    /// format, surfaced verbatim.
    Math(fmd_math::MathError),
    /// Invalid additional preamble, or ink produced before the formula.
    Preamble {
        /// Native parser diagnostic or admission failure.
        what: String,
    },
    /// The `tex.template` config value refused to resolve (an out-of-tier
    /// or unknown Reference template — the registry's named refusal).
    Pack(fmn_config::PackError),
    /// A pack content id from the registry names no fmd-math pack —
    /// registry/pack drift, which is a wiring bug worth its own name.
    UnknownPack {
        /// The content id that failed.
        content_id: &'static str,
    },
    /// A submobject primitive index outside the typeset's primitive
    /// lists — a consumer wiring bug, reported precisely (fm-p5d's
    /// [`crate::TexEngine::resolve_prim`]).
    BadPrim {
        /// Which index and how many primitives exist.
        what: String,
    },
    /// The bundled faces failed to load (build corruption).
    Faces {
        /// The underlying failure.
        what: String,
    },
    /// Cache wiring failed (opening the namespace). Typesetting itself
    /// never fails on cache trouble — reads/writes degrade to recompute.
    Cache {
        /// The underlying failure.
        what: String,
    },
    /// Owning the source string or canonical submobject table failed while
    /// constructing the in-memory result.
    Typeset(TypesetError),
}

impl fmt::Display for TexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Math(e) => e.fmt(f),
            Self::Preamble { what } => write!(f, "additional_preamble: {what}"),
            Self::Pack(e) => e.fmt(f),
            Self::BadPrim { what } => write!(f, "submobject primitive out of range: {what}"),
            Self::UnknownPack { content_id } => write!(
                f,
                "pack content id {content_id:?} names no fmd-math pack (registry/pack drift)"
            ),
            Self::Faces { what } => write!(f, "bundled faces failed to load: {what}"),
            Self::Cache { what } => write!(f, "typeset cache unavailable: {what}"),
            Self::Typeset(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Math(e) => Some(e),
            Self::Pack(e) => Some(e),
            Self::Typeset(error) => Some(error),
            _ => None,
        }
    }
}

impl From<fmd_math::MathError> for TexError {
    fn from(e: fmd_math::MathError) -> Self {
        Self::Math(e)
    }
}

impl From<TypesetError> for TexError {
    fn from(error: TypesetError) -> Self {
        Self::Typeset(error)
    }
}
