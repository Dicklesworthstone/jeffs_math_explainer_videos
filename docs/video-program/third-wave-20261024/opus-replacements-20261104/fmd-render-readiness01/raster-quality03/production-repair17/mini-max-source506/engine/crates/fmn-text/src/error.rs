//! Text-plane errors: precise, named, capability-style.

/// Why a text request failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextError {
    /// Malformed markup, with one-based line/column and what went wrong.
    Markup {
        /// What is wrong, in one clause.
        what: String,
        /// One-based line of the offense.
        line: usize,
        /// One-based column (in characters) of the offense.
        col: usize,
    },
    /// A requested font family is not available. Never a silent
    /// substitution (D-08): the error names what was asked and what
    /// exists.
    FontUnavailable {
        /// The family asked for.
        family: String,
        /// The families the book can serve.
        available: Vec<String>,
    },
    /// A user family name would resolve to an already registered family.
    FontFamilyConflict {
        /// The family name the caller tried to register.
        requested: String,
        /// The existing canonical family that wins lookup for that name.
        existing: String,
    },
    /// A user font file failed to parse.
    FontParse {
        /// The path as given.
        path: String,
        /// fmd-font's diagnosis.
        what: String,
    },
    /// A character no face in the book maps (named, per the coverage
    /// doctrine — never a tofu box, never a silent substitution).
    UnmappedChar {
        /// The character.
        ch: char,
        /// Its byte span in the source text.
        span: (usize, usize),
    },
    /// A glyph outline failed to decode (font-table corruption).
    Outline {
        /// The character whose glyph failed.
        ch: char,
        /// fmd-font's diagnosis.
        what: String,
    },
}

impl core::fmt::Display for TextError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Markup { what, line, col } => {
                write!(f, "malformed markup at {line}:{col}: {what}")
            }
            Self::FontUnavailable { family, available } => write!(
                f,
                "font family '{family}' is not available; bundled families: {} \
                 (load a TTF by path to add one — nothing is ever silently substituted)",
                available.join(", ")
            ),
            Self::FontFamilyConflict {
                requested,
                existing,
            } => write!(
                f,
                "font family '{requested}' conflicts with already registered family \
                 '{existing}' after canonical name and alias normalization"
            ),
            Self::FontParse { path, what } => {
                write!(f, "font file '{path}' failed to parse: {what}")
            }
            Self::UnmappedChar { ch, span } => write!(
                f,
                "character '{ch}' (U+{:04X}) has no glyph in the selected faces \
                 (bytes {}..{})",
                *ch as u32, span.0, span.1
            ),
            Self::Outline { ch, what } => {
                write!(f, "glyph outline for '{ch}' failed to decode: {what}")
            }
        }
    }
}

impl std::error::Error for TextError {}
