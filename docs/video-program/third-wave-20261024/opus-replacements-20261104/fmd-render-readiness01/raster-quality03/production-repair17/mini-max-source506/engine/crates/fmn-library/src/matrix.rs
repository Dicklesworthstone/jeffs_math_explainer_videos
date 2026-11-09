//! The `Matrix` family (Appendix A `mobject/matrix`): a grid of entries
//! between two stretchy brackets (fm-ebl).
//!
//! The Reference's `Matrix` is one class that dispatches on entry type —
//! mobjects pass through, floats become `DecimalNumber`s, everything else
//! becomes `Tex`. Rust has no such dispatch, so the family here is four
//! typed fronts over one grid engine: [`Matrix`] takes already-built
//! [`VMobject`] entries (the base's post-`element_to_mobject` state),
//! [`DecimalMatrix`] and [`IntegerMatrix`] typeset `f64` entries through
//! [`crate::numbers`], [`TexMatrix`] typesets string entries through
//! [`crate::tex::Tex`], and [`MobjectMatrix`] partitions a flat list of
//! mobjects into rows the Reference's way. The layout itself — step math,
//! alignment corner, height cap, brackets, ellipses — is shared verbatim.
//!
//! The grid (Reference `create_mobject_matrix`) places every entry by a
//! shared corner so differently-sized entries line up:
//!
//! ```text
//!   x_step = (max_width  + h_buff) · RIGHT
//!   y_step = (max_height + v_buff) · DOWN
//!   entry (i, j) moved_to(i·y_step + j·x_step, aligned_corner)
//! ```
//!
//! with `max_width`/`max_height` the largest entry extents, then the whole
//! family (entries + brackets) is re-centred on the origin.
//!
//! The brackets are the de-TeX'd half of the story (§11.6, ADR-0005). The
//! Reference typesets `\left[\begin{array}{c}\quad \\…\end{array}\right]`,
//! stretches the result to `rows.height + bracket_v_buff`, and splits the
//! submobjects in half — left half `[`, right half `]`. Here the same source
//! goes through [`crate::tex::Tex`]; fmd-math's extensible-delimiter engine
//! picks the natural glyph, a uniform scale of it (≤ 1.25×), or a drawn
//! path, and the delimiter pair is located by [`Prim::Path`] identity in the
//! drawn stage, falling back to the Reference's own half-split in the glyph
//! stages. Each bracket is then scaled uniformly to exactly
//! `rows.height + bracket_v_buff` — equivalent to the Reference's group
//! stretch, since the delimiters are the group's tallest children.
//!
//! Divergences from the Reference, all deliberate:
//!
//! * The dynamic entry dispatch is split into typed variants (above);
//!   `element_config` is each variant's own typed setters.
//! * `swap_entries_for_ellipses` is constructor-time configuration
//!   (`.ellipses_row()` / `.ellipses_col()`), because typesetting the
//!   `\vdots`/`\hdots` replacements needs the engine at build time. The
//!   swap semantics — order, sizing ratios, the −45° intersection — are
//!   the Reference's exactly.
//! * Out-of-range row/column access returns `None` instead of raising
//!   `IndexError`; a ragged or empty grid is a typed [`MatrixError`]
//!   instead of a Python `ValueError` from `max()`.

use fmn_core::color::Srgb;
use fmn_core::constants::{DEG, DOWN, LEFT, ORIGIN, OUT, RIGHT};
use fmn_core::types::Vec3;
use fmn_mobject::Mobject;
use fmn_tex::{Prim, TexEngine};
use fmn_text::FontBook;

use crate::numbers::{DecimalNumber, Integer};
use crate::tex::{Tex, TexMobjectError};
use crate::text::{DEFAULT_FONT_SIZE, TextMobjectError};
use crate::vmobject::{VMobject, v_group};

/// The Reference's `v_buff` default.
pub const DEFAULT_V_BUFF: f64 = 0.5;
/// The Reference's `h_buff` default.
pub const DEFAULT_H_BUFF: f64 = 0.5;
/// The Reference's `bracket_h_buff` default.
pub const DEFAULT_BRACKET_H_BUFF: f64 = 0.2;
/// The Reference's `bracket_v_buff` default.
pub const DEFAULT_BRACKET_V_BUFF: f64 = 0.25;
/// The Reference's `swap_entries_for_ellipses` height ratio.
pub const DEFAULT_ELLIPSES_HEIGHT_RATIO: f64 = 0.65;
/// The Reference's `swap_entries_for_ellipses` width ratio.
pub const DEFAULT_ELLIPSES_WIDTH_RATIO: f64 = 0.4;
/// The Reference's `MobjectMatrix` height default.
pub const DEFAULT_MOBJECT_MATRIX_HEIGHT: f64 = 4.0;
/// `DecimalMatrix`'s `num_decimal_places` default.
pub const DEFAULT_NUM_DECIMAL_PLACES: usize = 2;
/// Default upper bound for entries admitted by one matrix layout.
///
/// The limit is shared by every typed front and is checked before
/// cloning, typesetting, partitioning, or bracket construction.
pub const DEFAULT_MAX_MATRIX_ENTRIES: usize = 4_096;

/// A matrix-build failure: the tex/text bridges' precise errors pass
/// through untouched; the grid itself names malformed shapes, bounded
/// resource refusals, and delimiter failures.
#[derive(Debug)]
pub enum MatrixError {
    /// A bracket or `TexMatrix`/ellipses entry failed to typeset.
    Tex(TexMobjectError),
    /// A `DecimalMatrix`/`IntegerMatrix` entry failed to typeset.
    Text(TextMobjectError),
    /// No rows, or no columns — the Reference dies inside `max()`.
    Empty,
    /// Rows of unequal length; the Reference would silently drop the
    /// tail of every longer row.
    Ragged {
        /// The first row's length, which the Reference assumes for all.
        expected: usize,
        /// A row that disagreed.
        found: usize,
    },
    /// `MobjectMatrix` given fewer entries than `n_rows * n_cols` (the
    /// Reference raises a bare `Exception`).
    TooFewEntries {
        /// Entries supplied.
        have: usize,
        /// Entries the partition needs.
        need: usize,
    },
    /// Matrix dimensions describe more entries than the configured
    /// layout budget admits.
    ResourceLimit {
        /// The dimension or buffer being bounded.
        context: &'static str,
        /// Items the request would require.
        requested: usize,
        /// Maximum items admitted.
        limit: usize,
    },
    /// Matrix dimension or capacity arithmetic cannot be represented by
    /// the host.
    CapacityOverflow {
        /// The arithmetic operation being bounded.
        context: &'static str,
    },
    /// Reserving a validated bounded matrix buffer failed.
    AllocationFailed {
        /// The buffer being reserved.
        context: &'static str,
        /// Validated capacity requested.
        requested: usize,
    },
    /// The bracket typeset produced fewer than two submobjects — the
    /// delimiter engine must always emit a pair.
    Delimiters,
}

impl core::fmt::Display for MatrixError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tex(e) => write!(f, "matrix tex typeset failed: {e}"),
            Self::Text(e) => write!(f, "matrix number typeset failed: {e}"),
            Self::Empty => write!(f, "a matrix needs at least one row and one column"),
            Self::Ragged { expected, found } => write!(
                f,
                "ragged matrix: first row has {expected} entries, another has {found}"
            ),
            Self::TooFewEntries { have, need } => {
                write!(f, "MobjectMatrix needs at least {need} entries, got {have}")
            }
            Self::ResourceLimit {
                context,
                requested,
                limit,
            } => write!(
                f,
                "{context} requires {requested} items, above the declared limit of {limit}"
            ),
            Self::CapacityOverflow { context } => {
                write!(f, "{context} exceeds the host's representable capacity")
            }
            Self::AllocationFailed { context, requested } => {
                write!(f, "{context} could not reserve {requested} validated items")
            }
            Self::Delimiters => {
                write!(f, "the bracket typeset produced fewer than two submobjects")
            }
        }
    }
}

impl std::error::Error for MatrixError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Tex(e) => Some(e),
            Self::Text(e) => Some(e),
            _ => None,
        }
    }
}

impl From<TexMobjectError> for MatrixError {
    fn from(e: TexMobjectError) -> Self {
        Self::Tex(e)
    }
}

impl From<TextMobjectError> for MatrixError {
    fn from(e: TextMobjectError) -> Self {
        Self::Text(e)
    }
}

/// The layout configuration every variant shares — the Reference's
/// `Matrix.__init__` keyword surface.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Spec {
    v_buff: f64,
    h_buff: f64,
    bracket_h_buff: f64,
    bracket_v_buff: f64,
    height: Option<f64>,
    corner: Vec3,
    ellipses_row: Option<isize>,
    ellipses_col: Option<isize>,
    ellipses_height_ratio: f64,
    ellipses_width_ratio: f64,
    entry_limit: usize,
}

impl Spec {
    /// The base `Matrix` defaults: `element_alignment_corner=DOWN`, no
    /// height cap, no ellipses.
    const BASE: Self = Self {
        v_buff: DEFAULT_V_BUFF,
        h_buff: DEFAULT_H_BUFF,
        bracket_h_buff: DEFAULT_BRACKET_H_BUFF,
        bracket_v_buff: DEFAULT_BRACKET_V_BUFF,
        height: None,
        corner: DOWN,
        ellipses_row: None,
        ellipses_col: None,
        ellipses_height_ratio: DEFAULT_ELLIPSES_HEIGHT_RATIO,
        ellipses_width_ratio: DEFAULT_ELLIPSES_WIDTH_RATIO,
        entry_limit: DEFAULT_MAX_MATRIX_ENTRIES,
    };

    /// `MobjectMatrix`'s overrides: `element_alignment_corner=ORIGIN`,
    /// `height=4.0`.
    const MOBJECT: Self = Self {
        height: Some(DEFAULT_MOBJECT_MATRIX_HEIGHT),
        corner: ORIGIN,
        ..Self::BASE
    };
}

/// The `Matrix.__init__` keyword surface, shared verbatim by every
/// variant — by-value setters over the variant's `spec` field.
macro_rules! spec_setters {
    () => {
        /// The vertical gap between rows (`v_buff=0.5`).
        #[must_use]
        pub fn v_buff(mut self, v_buff: f64) -> Self {
            self.spec.v_buff = v_buff;
            self
        }

        /// The horizontal gap between columns (`h_buff=0.5`).
        #[must_use]
        pub fn h_buff(mut self, h_buff: f64) -> Self {
            self.spec.h_buff = h_buff;
            self
        }

        /// The gap between the grid and each bracket
        /// (`bracket_h_buff=0.2`).
        #[must_use]
        pub fn bracket_h_buff(mut self, bracket_h_buff: f64) -> Self {
            self.spec.bracket_h_buff = bracket_h_buff;
            self
        }

        /// How much taller than the grid the brackets stand
        /// (`bracket_v_buff=0.25`).
        #[must_use]
        pub fn bracket_v_buff(mut self, bracket_v_buff: f64) -> Self {
            self.spec.bracket_v_buff = bracket_v_buff;
            self
        }

        /// Maximum entries admitted for construction and layout.
        ///
        /// The default is [`DEFAULT_MAX_MATRIX_ENTRIES`]. Raising it is
        /// an explicit opt-in to proportionally larger work; dimension
        /// overflow is still refused.
        #[must_use]
        pub fn entry_limit(mut self, limit: usize) -> Self {
            self.spec.entry_limit = limit;
            self
        }

        /// Cap the whole matrix's height (`height=`): the grid is scaled
        /// to `height - 2 * bracket_v_buff` and the brackets absorb the
        /// rest, exactly the Reference's `rows.set_height(...)`.
        #[must_use]
        pub fn height(mut self, height: f64) -> Self {
            self.spec.height = Some(height);
            self
        }

        /// The corner entries are placed by
        /// (`element_alignment_corner=DOWN` for the base class, `ORIGIN`
        /// for [`MobjectMatrix`]).
        #[must_use]
        pub fn element_alignment_corner(mut self, corner: Vec3) -> Self {
            self.spec.corner = corner;
            self
        }

        /// Swap this row for `\vdots` ellipses at build time (the
        /// Reference's constructor-time `swap_entries_for_ellipses`;
        /// negative indices wrap, out-of-range indices are ignored).
        #[must_use]
        pub fn ellipses_row(mut self, row: isize) -> Self {
            self.spec.ellipses_row = Some(row);
            self
        }

        /// Swap this column for `\hdots` ellipses at build time; with
        /// [`ellipses_row`](Self::ellipses_row) the intersection is
        /// rotated −45° into a `\ddots` stand-in, as the Reference does.
        #[must_use]
        pub fn ellipses_col(mut self, col: isize) -> Self {
            self.spec.ellipses_col = Some(col);
            self
        }

        /// The ellipses sizing ratios (`height_ratio=0.65`,
        /// `width_ratio=0.4`): a `\vdots` stands `height_ratio` of an
        /// average row tall, an `\hdots` `width_ratio` of an average
        /// column wide.
        #[must_use]
        pub fn ellipses_ratios(mut self, height_ratio: f64, width_ratio: f64) -> Self {
            self.spec.ellipses_height_ratio = height_ratio;
            self.spec.ellipses_width_ratio = width_ratio;
            self
        }
    };
}

fn checked_matrix_add(context: &'static str, lhs: usize, rhs: usize) -> Result<usize, MatrixError> {
    lhs.checked_add(rhs)
        .ok_or(MatrixError::CapacityOverflow { context })
}

fn checked_matrix_mul(context: &'static str, lhs: usize, rhs: usize) -> Result<usize, MatrixError> {
    lhs.checked_mul(rhs)
        .ok_or(MatrixError::CapacityOverflow { context })
}

fn checked_matrix_entries(
    n_rows: usize,
    n_cols: usize,
    limit: usize,
) -> Result<usize, MatrixError> {
    if n_rows == 0 || n_cols == 0 {
        return Err(MatrixError::Empty);
    }
    let requested = checked_matrix_mul("matrix entry count", n_rows, n_cols)?;
    if requested > limit {
        return Err(MatrixError::ResourceLimit {
            context: "matrix entries",
            requested,
            limit,
        });
    }
    Ok(requested)
}

fn validate_grid_shape<T>(
    grid: &[Vec<T>],
    entry_limit: usize,
) -> Result<(usize, usize, usize), MatrixError> {
    let n_rows = grid.len();
    let n_cols = grid.first().map_or(0, Vec::len);
    let n_entries = checked_matrix_entries(n_rows, n_cols, entry_limit)?;
    for row in grid {
        if row.len() != n_cols {
            return Err(MatrixError::Ragged {
                expected: n_cols,
                found: row.len(),
            });
        }
    }
    Ok((n_rows, n_cols, n_entries))
}

fn try_matrix_vec<T>(context: &'static str, capacity: usize) -> Result<Vec<T>, MatrixError> {
    let element_size = core::mem::size_of::<T>();
    if element_size != 0 {
        let bytes = capacity
            .checked_mul(element_size)
            .ok_or(MatrixError::CapacityOverflow { context })?;
        if bytes > isize::MAX as usize {
            return Err(MatrixError::CapacityOverflow { context });
        }
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| MatrixError::AllocationFailed {
            context,
            requested: capacity,
        })?;
    Ok(values)
}

fn clone_grid<T: Clone>(
    grid: &[Vec<T>],
    n_rows: usize,
    n_cols: usize,
) -> Result<Vec<Vec<T>>, MatrixError> {
    let mut cloned = try_matrix_vec("matrix rows", n_rows)?;
    for row in grid {
        let mut cloned_row = try_matrix_vec("matrix row entries", n_cols)?;
        cloned_row.extend(row.iter().cloned());
        cloned.push(cloned_row);
    }
    Ok(cloned)
}

/// `Matrix(matrix)` — a grid of already-built entries between brackets.
///
/// This is the Reference base class at its post-`element_to_mobject`
/// state: entries are [`VMobject`]s and pass through untouched, laid out
/// by the shared engine. Entries that need typesetting are the typed
/// variants' job ([`TexMatrix`], [`DecimalMatrix`], [`IntegerMatrix`]).
#[derive(Debug, Clone)]
pub struct Matrix {
    spec: Spec,
    entries: Vec<Vec<VMobject>>,
}

impl Matrix {
    /// A matrix of mobject entries, row-major rows.
    #[must_use]
    pub fn new(entries: Vec<Vec<VMobject>>) -> Self {
        Self {
            spec: Spec::BASE,
            entries,
        }
    }

    spec_setters!();

    /// Lay the grid out and typeset the brackets (and any ellipses).
    ///
    /// # Errors
    /// [`MatrixError::Empty`]/[`MatrixError::Ragged`] for malformed
    /// grids, [`MatrixError::Tex`] when a bracket or ellipsis fails to
    /// typeset, [`MatrixError::Delimiters`] when the delimiter engine
    /// emits no pair.
    pub fn build(&self, engine: &TexEngine) -> Result<MatrixMobject, MatrixError> {
        let (n_rows, n_cols, _) = validate_grid_shape(&self.entries, self.spec.entry_limit)?;
        let entries = clone_grid(&self.entries, n_rows, n_cols)?;
        build_grid(entries, &self.spec, engine)
    }
}

/// `TexMatrix(matrix)` — string entries typeset as mathematics through
/// [`Tex`], then laid out by the shared engine.
#[derive(Debug, Clone)]
pub struct TexMatrix<'a> {
    spec: Spec,
    entries: Vec<Vec<&'a str>>,
    font_size: f64,
}

impl<'a> TexMatrix<'a> {
    /// A matrix of tex-source entries (the Reference's
    /// `element_to_mobject` fallback branch).
    #[must_use]
    pub fn new(entries: Vec<Vec<&'a str>>) -> Self {
        Self {
            spec: Spec::BASE,
            entries,
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    spec_setters!();

    /// The entries' font size (`font_size=48`).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Typeset the entries and lay the grid out.
    ///
    /// # Errors
    /// [`MatrixError::Tex`] when an entry, bracket, or ellipsis fails to
    /// typeset; the grid errors of [`Matrix::build`].
    pub fn build(&self, engine: &TexEngine) -> Result<MatrixMobject, MatrixError> {
        let (n_rows, n_cols, _) = validate_grid_shape(&self.entries, self.spec.entry_limit)?;
        let mut grid = try_matrix_vec("tex-matrix rows", n_rows)?;
        for row in &self.entries {
            let mut placed = try_matrix_vec("tex-matrix entries", n_cols)?;
            for entry in row {
                let m = Tex::new(entry).font_size(self.font_size).build(engine)?;
                placed.push(m.vmob);
            }
            grid.push(placed);
        }
        build_grid(grid, &self.spec, engine)
    }
}

/// `DecimalMatrix(matrix, num_decimal_places=2)` — `f64` entries typeset
/// through [`DecimalNumber`], then laid out by the shared engine.
#[derive(Debug, Clone)]
pub struct DecimalMatrix {
    spec: Spec,
    entries: Vec<Vec<f64>>,
    num_decimal_places: usize,
    font_size: f64,
}

impl DecimalMatrix {
    /// A matrix of decimal entries.
    #[must_use]
    pub fn new(entries: Vec<Vec<f64>>) -> Self {
        Self {
            spec: Spec::BASE,
            entries,
            num_decimal_places: DEFAULT_NUM_DECIMAL_PLACES,
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    spec_setters!();

    /// `num_decimal_places=` (default 2).
    #[must_use]
    pub fn num_decimal_places(mut self, num_decimal_places: usize) -> Self {
        self.num_decimal_places = num_decimal_places;
        self
    }

    /// The entries' font size (`font_size=48`).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Typeset the entries and lay the grid out. The engine serves the
    /// brackets and ellipses; the book serves the numbers.
    ///
    /// # Errors
    /// [`MatrixError::Text`] when an entry fails to typeset;
    /// [`MatrixError::Tex`] for the brackets and ellipses; the grid
    /// errors of [`Matrix::build`].
    pub fn build(&self, engine: &TexEngine, book: &FontBook) -> Result<MatrixMobject, MatrixError> {
        let (n_rows, n_cols, _) = validate_grid_shape(&self.entries, self.spec.entry_limit)?;
        let mut grid = try_matrix_vec("decimal-matrix rows", n_rows)?;
        for row in &self.entries {
            let mut placed = try_matrix_vec("decimal-matrix entries", n_cols)?;
            for entry in row {
                let number = DecimalNumber::new(*entry)
                    .num_decimal_places(self.num_decimal_places)
                    .font_size(self.font_size)
                    .build(book)?;
                placed.push(number.into_vmob());
            }
            grid.push(placed);
        }
        build_grid(grid, &self.spec, engine)
    }
}

/// `IntegerMatrix(matrix)` — the Reference's `DecimalMatrix` subclass
/// with `num_decimal_places=0`; entries typeset through [`Integer`].
#[derive(Debug, Clone)]
pub struct IntegerMatrix {
    spec: Spec,
    entries: Vec<Vec<f64>>,
    font_size: f64,
}

impl IntegerMatrix {
    /// A matrix of integer entries (values are rounded to display, the
    /// Reference's `Integer` semantics).
    #[must_use]
    pub fn new(entries: Vec<Vec<f64>>) -> Self {
        Self {
            spec: Spec::BASE,
            entries,
            font_size: DEFAULT_FONT_SIZE,
        }
    }

    spec_setters!();

    /// The entries' font size (`font_size=48`).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Typeset the entries and lay the grid out.
    ///
    /// # Errors
    /// [`MatrixError::Text`] when an entry fails to typeset;
    /// [`MatrixError::Tex`] for the brackets and ellipses; the grid
    /// errors of [`Matrix::build`].
    pub fn build(&self, engine: &TexEngine, book: &FontBook) -> Result<MatrixMobject, MatrixError> {
        let (n_rows, n_cols, _) = validate_grid_shape(&self.entries, self.spec.entry_limit)?;
        let mut grid = try_matrix_vec("integer-matrix rows", n_rows)?;
        for row in &self.entries {
            let mut placed = try_matrix_vec("integer-matrix entries", n_cols)?;
            for entry in row {
                let number = Integer::new(*entry).font_size(self.font_size).build(book)?;
                placed.push(number.into_vmob());
            }
            grid.push(placed);
        }
        build_grid(grid, &self.spec, engine)
    }
}

/// `MobjectMatrix(group, n_rows, n_cols, height=4.0)` — partition a flat
/// list of mobjects into a grid, the Reference's way: a missing `n_rows`
/// is `floor(sqrt(n))` (or `n // n_cols`), a missing `n_cols` is
/// `n // n_rows`, and the result is height-capped to 4.0 by default with
/// entries placed by their centres (`element_alignment_corner=ORIGIN`).
#[derive(Debug, Clone)]
pub struct MobjectMatrix {
    spec: Spec,
    entries: Vec<VMobject>,
    n_rows: Option<usize>,
    n_cols: Option<usize>,
}

impl MobjectMatrix {
    /// A matrix partitioned out of a flat entry list.
    #[must_use]
    pub fn new(entries: Vec<VMobject>) -> Self {
        Self {
            spec: Spec::MOBJECT,
            entries,
            n_rows: None,
            n_cols: None,
        }
    }

    spec_setters!();

    /// Fix the row count (`n_rows=`); the column count defaults to
    /// `n // n_rows`.
    #[must_use]
    pub fn n_rows(mut self, n_rows: usize) -> Self {
        self.n_rows = Some(n_rows);
        self
    }

    /// Fix the column count (`n_cols=`); the row count defaults to
    /// `n // n_cols`.
    #[must_use]
    pub fn n_cols(mut self, n_cols: usize) -> Self {
        self.n_cols = Some(n_cols);
        self
    }

    /// Partition and lay the grid out.
    ///
    /// # Errors
    /// [`MatrixError::TooFewEntries`] when fewer than `n_rows * n_cols`
    /// entries were supplied, [`MatrixError::Empty`] when a partition
    /// dimension collapses to zero, plus [`Matrix::build`]'s errors.
    pub fn build(&self, engine: &TexEngine) -> Result<MatrixMobject, MatrixError> {
        let n = self.entries.len();
        // The Reference's fallback chain, verbatim.
        let n_rows = match (self.n_rows, self.n_cols) {
            (Some(r), _) => r,
            (None, Some(c)) => {
                if c == 0 {
                    return Err(MatrixError::Empty);
                }
                n / c
            }
            (None, None) => (n as f64).sqrt() as usize,
        };
        let n_cols = match self.n_cols {
            Some(c) => c,
            None => {
                if n_rows == 0 {
                    return Err(MatrixError::Empty);
                }
                n / n_rows
            }
        };
        let n_entries = checked_matrix_entries(n_rows, n_cols, self.spec.entry_limit)?;
        if n < n_entries {
            return Err(MatrixError::TooFewEntries {
                have: n,
                need: n_entries,
            });
        }
        let entries = self
            .entries
            .get(..n_entries)
            .ok_or(MatrixError::TooFewEntries {
                have: n,
                need: n_entries,
            })?;
        let mut grid = try_matrix_vec("mobject-matrix rows", n_rows)?;
        for row in entries.chunks(n_cols) {
            let mut cloned_row = try_matrix_vec("mobject-matrix row entries", n_cols)?;
            cloned_row.extend(row.iter().cloned());
            grid.push(cloned_row);
        }
        build_grid(grid, &self.spec, engine)
    }
}

/// A built matrix: the family plus the grid metadata the accessors
/// slice by.
///
/// `vmob`'s children are the entries in row-major grid order — child
/// `i * n_cols + j` is entry `(i, j)`, including any entries swapped for
/// ellipses — followed by the left and right brackets. Swapping replaces
/// a child in place, so grid indices never shift (the Reference's
/// `become` semantics).
#[derive(Debug, Clone)]
pub struct MatrixMobject {
    /// The family: entries row-major, then the two brackets.
    pub vmob: VMobject,
    n_rows: usize,
    n_cols: usize,
    n_entries: usize,
    ellipses: Vec<usize>,
    brackets_drawn: bool,
}

impl MatrixMobject {
    /// The row count.
    #[must_use]
    pub fn n_rows(&self) -> usize {
        self.n_rows
    }

    /// The column count.
    #[must_use]
    pub fn n_cols(&self) -> usize {
        self.n_cols
    }

    /// True when the delimiter engine drew the brackets as paths
    /// (ADR-0005's tall stage) rather than emitting glyphs.
    #[must_use]
    pub fn brackets_are_drawn_paths(&self) -> bool {
        self.brackets_drawn
    }

    /// `get_row(index)`: the row's entries as one detached group —
    /// `None` out of range, where the Reference raises `IndexError`.
    #[must_use]
    pub fn get_row(&self, index: usize) -> Option<VMobject> {
        if index >= self.n_rows {
            return None;
        }
        let start = index.checked_mul(self.n_cols)?;
        let end = start.checked_add(self.n_cols)?;
        let row = self.vmob.children().get(start..end)?;
        Some(v_group(row.to_vec()))
    }

    /// `get_column(index)`: the column's entries as one detached group.
    #[must_use]
    pub fn get_column(&self, index: usize) -> Option<VMobject> {
        if index >= self.n_cols {
            return None;
        }
        let mut column = Vec::new();
        column.try_reserve_exact(self.n_rows).ok()?;
        for row in 0..self.n_rows {
            let entry = row.checked_mul(self.n_cols)?.checked_add(index)?;
            column.push(self.vmob.children().get(entry)?.clone());
        }
        Some(v_group(column))
    }

    /// `get_rows()`: every row as a detached group, top to bottom.
    #[must_use]
    pub fn get_rows(&self) -> Vec<VMobject> {
        (0..self.n_rows).filter_map(|i| self.get_row(i)).collect()
    }

    /// `get_columns()`: every column as a detached group, left to right.
    #[must_use]
    pub fn get_columns(&self) -> Vec<VMobject> {
        (0..self.n_cols)
            .filter_map(|j| self.get_column(j))
            .collect()
    }

    /// `get_entries()`: the entries that were not swapped for ellipses,
    /// in row-major grid order, as one detached group.
    #[must_use]
    pub fn get_entries(&self) -> VMobject {
        let entries = self
            .vmob
            .children()
            .get(..self.n_entries)
            .unwrap_or_default();
        v_group(
            entries
                .iter()
                .enumerate()
                .filter(|(k, _)| !self.ellipses.contains(k))
                .map(|(_, child)| child.clone())
                .collect::<Vec<_>>(),
        )
    }

    /// `get_brackets()`: the left and right brackets as one detached
    /// group.
    #[must_use]
    pub fn get_brackets(&self) -> VMobject {
        let brackets = self
            .vmob
            .children()
            .get(self.n_entries..)
            .unwrap_or_default();
        v_group(brackets.to_vec())
    }

    /// `get_ellipses()`: the ellipses swapped in for entries, as one
    /// detached group.
    #[must_use]
    pub fn get_ellipses(&self) -> VMobject {
        v_group(
            self.ellipses
                .iter()
                .filter_map(|&k| self.vmob.children().get(k).cloned())
                .collect::<Vec<_>>(),
        )
    }

    /// `set_column_colors(*colors)`: recolor whole columns, extra colors
    /// ignored and missing columns untouched, exactly the Reference's
    /// `zip`. The recolor recurses into each entry's glyph family.
    #[must_use]
    pub fn set_column_colors(mut self, colors: &[Srgb]) -> Self {
        let n_cols = self.n_cols;
        let n_entries = self.n_entries;
        let mut index = 0usize;
        self.vmob = self.vmob.map_children(|child| {
            let k = index;
            index = index.saturating_add(1);
            if k < n_entries && n_cols != 0 {
                match colors.get(k % n_cols) {
                    Some(&color) => child.map_style_deep(|s| s.color(color)),
                    None => child,
                }
            } else {
                child
            }
        });
        self
    }
}

impl From<MatrixMobject> for Mobject {
    fn from(m: MatrixMobject) -> Self {
        m.vmob.into()
    }
}

/// The shared grid engine: place entries by the Reference's step math,
/// cap the height, hang the brackets, centre, swap ellipses.
fn build_grid(
    grid: Vec<Vec<VMobject>>,
    spec: &Spec,
    engine: &TexEngine,
) -> Result<MatrixMobject, MatrixError> {
    let (n_rows, n_cols, n_entries) = validate_grid_shape(&grid, spec.entry_limit)?;
    let child_capacity = checked_matrix_add("matrix child capacity", n_entries, 2)?;
    let ellipses_capacity =
        checked_matrix_add("matrix ellipses capacity", n_rows, n_cols)?.min(n_entries);

    // create_mobject_matrix: shared-corner placement on the step grid.
    let max_width = grid
        .iter()
        .flatten()
        .map(|e| e.length_over_dim(0))
        .fold(0.0, f64::max);
    let max_height = grid
        .iter()
        .flatten()
        .map(|e| e.length_over_dim(1))
        .fold(0.0, f64::max);
    let x_step = max_width + spec.h_buff; // · RIGHT
    let y_step = max_height + spec.v_buff; // · DOWN
    let mut grid: Vec<Vec<VMobject>> = grid
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            row.into_iter()
                .enumerate()
                .map(|(j, elem)| {
                    #[allow(clippy::cast_precision_loss)]
                    let target = [j as f64 * x_step, -(i as f64) * y_step, 0.0];
                    elem.moved_to_aligned(target, spec.corner)
                })
                .collect()
        })
        .collect();

    // `if height is not None: rows.set_height(height - 2 * bracket_v_buff)`
    // — a uniform scale about the grid's own centre.
    if let Some(height) = spec.height {
        let (min, max) = grid_extent(&grid);
        let rows_height = max[1] - min[1];
        if rows_height != 0.0 {
            let centre = midpoint(min, max);
            let factor = (height - 2.0 * spec.bracket_v_buff) / rows_height;
            grid = grid
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|e| e.scaled_about(factor, centre))
                        .collect()
                })
                .collect();
        }
    }

    // The brackets measure against the (possibly capped) grid.
    let (rows_min, rows_max) = grid_extent(&grid);
    let rows_height = rows_max[1] - rows_min[1];
    let rows_mid = midpoint(rows_min, rows_max);
    let (left_bracket, right_bracket, drawn) = brackets(n_rows, engine)?;
    let bracket_height = rows_height + spec.bracket_v_buff;
    let left_bracket = left_bracket
        .rescaled_to_fit(bracket_height, 1, false)
        .next_to_point(
            [rows_min[0], rows_mid[1], rows_mid[2]],
            LEFT,
            spec.bracket_h_buff,
            ORIGIN,
        );
    let right_bracket = right_bracket
        .rescaled_to_fit(bracket_height, 1, false)
        .next_to_point(
            [rows_max[0], rows_mid[1], rows_mid[2]],
            RIGHT,
            spec.bracket_h_buff,
            ORIGIN,
        );

    // `self.center()`: entries and brackets together, about the origin.
    let (min, max) = union_extent(
        Some(grid_extent(&grid)),
        union_extent(left_bracket.extent(), right_bracket.extent()),
    )
    .unwrap_or((ORIGIN, ORIGIN));
    let centre = midpoint(min, max);
    let offset = [-centre[0], -centre[1], -centre[2]];
    let mut grid: Vec<Vec<VMobject>> = grid
        .into_iter()
        .map(|row| row.into_iter().map(|e| e.shifted(offset)).collect())
        .collect();
    let left_bracket = left_bracket.shifted(offset);
    let right_bracket = right_bracket.shifted(offset);

    // The constructor-time ellipses swap (Reference semantics exactly).
    let mut ellipses = try_matrix_vec("matrix ellipses", ellipses_capacity)?;
    swap_entries_for_ellipses(&mut grid, spec, engine, &mut ellipses)?;

    let mut children = try_matrix_vec("matrix children", child_capacity)?;
    for row in grid {
        children.extend(row);
    }
    children.push(left_bracket);
    children.push(right_bracket);
    Ok(MatrixMobject {
        vmob: v_group(children),
        n_rows,
        n_cols,
        n_entries,
        ellipses,
        brackets_drawn: drawn,
    })
}

/// The Reference's `swap_entries_for_ellipses`: vdots first (so the
/// intersection cell ends as an hdots), then hdots, then the −45° turn.
fn normalized_index(index: isize, len: usize) -> Option<usize> {
    if index >= 0 {
        usize::try_from(index).ok().filter(|&value| value < len)
    } else {
        len.checked_sub(index.unsigned_abs())
    }
}

fn checked_grid_index(row: usize, n_cols: usize, col: usize) -> Result<usize, MatrixError> {
    checked_matrix_add(
        "matrix entry index",
        checked_matrix_mul("matrix entry index", row, n_cols)?,
        col,
    )
}

fn swap_entries_for_ellipses(
    grid: &mut [Vec<VMobject>],
    spec: &Spec,
    engine: &TexEngine,
    ellipses: &mut Vec<usize>,
) -> Result<(), MatrixError> {
    let n_rows = grid.len();
    let n_cols = grid.first().map_or(0, Vec::len);
    let row_index = spec
        .ellipses_row
        .and_then(|index| normalized_index(index, n_rows));
    let col_index = spec
        .ellipses_col
        .and_then(|index| normalized_index(index, n_cols));
    if row_index.is_none() && col_index.is_none() {
        return Ok(());
    }

    let (min, max) = grid_extent(grid);
    let avg_row_height = (max[1] - min[1]) / n_rows as f64;
    let avg_col_width = (max[0] - min[0]) / n_cols as f64;
    let vdots_height = spec.ellipses_height_ratio * avg_row_height;
    let hdots_width = spec.ellipses_width_ratio * avg_col_width;

    if let Some(ri) = row_index {
        let row = grid.get_mut(ri).ok_or(MatrixError::CapacityOverflow {
            context: "matrix ellipses row index",
        })?;
        for (j, entry) in row.iter_mut().enumerate() {
            let dots = Tex::new(r"\vdots")
                .build(engine)?
                .vmob
                .with_height(vdots_height, false)
                .moved_to(entry.center_point());
            *entry = dots;
            let k = checked_grid_index(ri, n_cols, j)?;
            if !ellipses.contains(&k) {
                ellipses.push(k);
            }
        }
    }
    if let Some(ci) = col_index {
        for (i, row) in grid.iter_mut().enumerate() {
            let entry = row.get_mut(ci).ok_or(MatrixError::CapacityOverflow {
                context: "matrix ellipses column index",
            })?;
            let dots = Tex::new(r"\hdots")
                .build(engine)?
                .vmob
                .with_width(hdots_width, false)
                .moved_to(entry.center_point());
            *entry = dots;
            let k = checked_grid_index(i, n_cols, ci)?;
            if !ellipses.contains(&k) {
                ellipses.push(k);
            }
        }
    }
    if let (Some(ri), Some(ci)) = (row_index, col_index) {
        let entry = grid.get_mut(ri).and_then(|row| row.get_mut(ci)).ok_or(
            MatrixError::CapacityOverflow {
                context: "matrix ellipses intersection index",
            },
        )?;
        let centre = entry.center_point();
        *entry = entry.clone().rotated_about(-45.0 * DEG, OUT, centre);
    }
    Ok(())
}

/// The bracket pair: typeset the Reference's own source — one `\quad \\`
/// row per matrix row — and locate the delimiters. In the drawn stage
/// (ADR-0005) the two [`Prim::Path`] children *are* the delimiters; in
/// the glyph stages the Reference's own half-split applies. The boolean
/// reports which stage engaged.
fn brackets(n_rows: usize, engine: &TexEngine) -> Result<(VMobject, VMobject, bool), MatrixError> {
    const PREFIX: &str = r"\left[\begin{array}{c}";
    const ROW: &str = "\\quad \\\\";
    const SUFFIX: &str = r"\end{array}\right]";
    let row_bytes = checked_matrix_mul("matrix bracket source bytes", n_rows, ROW.len())?;
    let source_bytes = checked_matrix_add(
        "matrix bracket source bytes",
        checked_matrix_add("matrix bracket source bytes", PREFIX.len(), row_bytes)?,
        SUFFIX.len(),
    )?;
    let mut source = String::new();
    source
        .try_reserve_exact(source_bytes)
        .map_err(|_| MatrixError::AllocationFailed {
            context: "matrix bracket source",
            requested: source_bytes,
        })?;
    source.push_str(PREFIX);
    for _ in 0..n_rows {
        source.push_str(ROW);
    }
    source.push_str(SUFFIX);
    let brackets = Tex::new(&source).build(engine)?;
    let children = brackets.vmob.children();
    let path_ordinals: Vec<usize> = brackets
        .typeset
        .subs
        .iter()
        .enumerate()
        .filter_map(|(i, sub)| matches!(sub.prim, Prim::Path(_)).then_some(i))
        .collect();
    if path_ordinals.len() == 2 {
        let left = children
            .get(path_ordinals[0])
            .ok_or(MatrixError::Delimiters)?
            .clone();
        let right = children
            .get(path_ordinals[1])
            .ok_or(MatrixError::Delimiters)?
            .clone();
        return Ok((left, right, true));
    }
    let n = children.len();
    if n < 2 {
        return Err(MatrixError::Delimiters);
    }
    Ok((
        v_group(children[..n / 2].to_vec()),
        v_group(children[n / 2..].to_vec()),
        false,
    ))
}

/// The union of every entry's extent; an all-empty grid reports the
/// origin point (never divides by the result unchecked).
fn grid_extent(grid: &[Vec<VMobject>]) -> (Vec3, Vec3) {
    grid.iter()
        .flatten()
        .fold(None, |acc, e| union_extent(acc, e.extent()))
        .unwrap_or((ORIGIN, ORIGIN))
}

fn union_extent(a: Option<(Vec3, Vec3)>, b: Option<(Vec3, Vec3)>) -> Option<(Vec3, Vec3)> {
    match (a, b) {
        (Some((amin, amax)), Some((bmin, bmax))) => Some((
            [
                amin[0].min(bmin[0]),
                amin[1].min(bmin[1]),
                amin[2].min(bmin[2]),
            ],
            [
                amax[0].max(bmax[0]),
                amax[1].max(bmax[1]),
                amax[2].max(bmax[2]),
            ],
        )),
        (Some(e), None) | (None, Some(e)) => Some(e),
        (None, None) => None,
    }
}

fn midpoint(min: Vec3, max: Vec3) -> Vec3 {
    [
        0.5 * (min[0] + max[0]),
        0.5 * (min[1] + max[1]),
        0.5 * (min[2] + max[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::Rectangle;
    use fmn_core::constants::{BLUE, RED};

    /// The family's whole point count, own run plus descendants.
    fn family_points(vmob: &VMobject) -> usize {
        vmob.points().len() + vmob.children().iter().map(family_points).sum::<usize>()
    }

    fn engine() -> TexEngine {
        TexEngine::new("fmd-math/pack/default", None).expect("engine")
    }

    fn book() -> FontBook {
        FontBook::bundled().expect("book")
    }

    /// A w×h rectangle entry centred on the origin.
    fn cell(w: f64, h: f64) -> VMobject {
        Rectangle::new()
            .width(w)
            .height(h)
            .build()
            .expect("the test cell is unrounded")
    }

    /// An n×n grid of identical w×h cells.
    fn grid_of(n_rows: usize, n_cols: usize, w: f64, h: f64) -> Vec<Vec<VMobject>> {
        (0..n_rows)
            .map(|_| (0..n_cols).map(|_| cell(w, h)).collect())
            .collect()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn grid_layout_matches_the_reference_step_math() {
        // 3×2 of identical 1×0.5 cells, default buffs, corner DOWN.
        let m = Matrix::new(grid_of(3, 2, 1.0, 0.5))
            .build(&engine())
            .expect("builds");
        let x_step = 1.0 + DEFAULT_H_BUFF;
        let y_step = 0.5 + DEFAULT_V_BUFF;
        let children = m.vmob.children();
        let anchor = children[0].bbox_point(DOWN).expect("anchor");
        for i in 0..3 {
            for j in 0..2 {
                let p = children[i * 2 + j].bbox_point(DOWN).expect("corner");
                // Identical cells: the shift from (0,0) is exactly the
                // step vector (the common re-centring cancels).
                assert_eq!(p[0] - anchor[0], j as f64 * x_step, "x step at ({i},{j})");
                assert_eq!(
                    p[1] - anchor[1],
                    -(i as f64) * y_step,
                    "y step at ({i},{j})"
                );
                assert_eq!(p[2] - anchor[2], 0.0);
            }
        }
        // The whole family (entries + brackets) is centred on the origin.
        let centre = m.vmob.center_point();
        assert!(
            close(centre[0], 0.0) && close(centre[1], 0.0),
            "centre {centre:?}"
        );
        // Rows run down, columns run right.
        let first = children[0].bbox_point(DOWN).expect("first");
        let below = children[2].bbox_point(DOWN).expect("below");
        let right = children[1].bbox_point(DOWN).expect("right");
        assert!(below[1] < first[1] && right[0] > first[0]);
    }

    #[test]
    fn aligned_corner_is_the_placement_corner() {
        // Differently-sized entries share the DOWN edge, not centres.
        let entries = vec![vec![cell(1.0, 1.0), cell(0.5, 0.25)]];
        let m = Matrix::new(entries).build(&engine()).expect("builds");
        let children = m.vmob.children();
        let a = children[0].bbox_point(DOWN).expect("a");
        let b = children[1].bbox_point(DOWN).expect("b");
        assert!(close(a[1], b[1]), "DOWN edges align: {a:?} vs {b:?}");
        // x_step uses the LARGEST width in the grid.
        assert!(close(b[0] - a[0], 1.0 + DEFAULT_H_BUFF));
    }

    #[test]
    fn bracket_height_is_rows_height_plus_v_buff_for_1_to_8_rows() {
        for n in 1..=8usize {
            let m = Matrix::new(grid_of(n, 1, 0.5, 0.4))
                .build(&engine())
                .expect("builds");
            let rows_height = {
                let rows = m.get_rows();
                let (min, max) = rows
                    .iter()
                    .fold(None, |acc, r| union_extent(acc, r.extent()))
                    .expect("rows");
                max[1] - min[1]
            };
            let brackets = m.get_brackets();
            assert_eq!(brackets.children().len(), 2);
            for bracket in brackets.children() {
                assert!(
                    family_points(bracket) > 0,
                    "bracket family has points at {n} rows"
                );
                if m.brackets_are_drawn_paths() {
                    // ADR-0005's tall stage: the bracket child IS the
                    // drawn path, points and all.
                    assert!(
                        !bracket.points().is_empty(),
                        "drawn-path bracket child is a path with points at {n} rows"
                    );
                }
                let h = bracket.length_over_dim(1);
                assert!(
                    close(h, rows_height + DEFAULT_BRACKET_V_BUFF),
                    "{n} rows: bracket height {h} != {} + {}",
                    rows_height,
                    DEFAULT_BRACKET_V_BUFF
                );
            }
            // The brackets hang outside the grid, bracket_h_buff away.
            let grid_left = m
                .get_column(0)
                .expect("column")
                .bbox_point(LEFT)
                .expect("edge");
            let left_bracket = brackets.children()[0].bbox_point(RIGHT).expect("edge");
            assert!(close(
                grid_left[0] - left_bracket[0],
                DEFAULT_BRACKET_H_BUFF
            ));
        }
    }

    #[test]
    fn the_drawn_path_stage_engages_for_tall_matrices() {
        // ADR-0005: natural glyph → uniform scale (≤1.25×) → drawn path.
        // Eight rows of quads far exceed the scale ceiling, so the
        // delimiters are drawn paths; a single row stays a glyph.
        let tall = Matrix::new(grid_of(8, 1, 0.5, 0.4))
            .build(&engine())
            .expect("builds");
        assert!(tall.brackets_are_drawn_paths(), "8 rows should draw paths");
        let short = Matrix::new(grid_of(1, 1, 0.5, 0.4))
            .build(&engine())
            .expect("builds");
        assert!(
            !short.brackets_are_drawn_paths(),
            "1 row should stay a natural glyph"
        );
    }

    #[test]
    fn height_cap_scales_rows_and_brackets_absorb_the_rest() {
        let m = Matrix::new(grid_of(2, 2, 1.0, 1.0))
            .height(3.0)
            .build(&engine())
            .expect("builds");
        // rows.set_height(height - 2 * bracket_v_buff)
        let rows = m.get_rows();
        let (min, max) = rows
            .iter()
            .fold(None, |acc, r| union_extent(acc, r.extent()))
            .expect("rows");
        let rows_height = max[1] - min[1];
        assert!(
            close(rows_height, 3.0 - 2.0 * DEFAULT_BRACKET_V_BUFF),
            "rows height {rows_height}"
        );
        // And the brackets then stand height - bracket_v_buff tall.
        let bracket = m.get_brackets().children()[0].length_over_dim(1);
        assert!(close(bracket, 3.0 - DEFAULT_BRACKET_V_BUFF));
    }

    #[test]
    fn row_and_column_slices_follow_grid_order() {
        let m = Matrix::new(grid_of(3, 4, 0.5, 0.5))
            .build(&engine())
            .expect("builds");
        let row = m.get_row(1).expect("row 1");
        assert_eq!(row.children().len(), 4);
        let col = m.get_column(2).expect("column 2");
        assert_eq!(col.children().len(), 3);
        // Slice children are the family's children, in grid order.
        for (j, child) in row.children().iter().enumerate() {
            let a = child.center_point();
            let b = m.vmob.children()[4 + j].center_point();
            assert_eq!(a, b);
        }
        for (i, child) in col.children().iter().enumerate() {
            let a = child.center_point();
            let b = m.vmob.children()[i * 4 + 2].center_point();
            assert_eq!(a, b);
        }
        assert_eq!(m.get_rows().len(), 3);
        assert_eq!(m.get_columns().len(), 4);
        // Out of range is None, not a panic.
        assert!(m.get_row(3).is_none());
        assert!(m.get_column(4).is_none());
        // Entries and brackets partition the family.
        assert_eq!(m.get_entries().children().len(), 12);
        assert_eq!(m.get_brackets().children().len(), 2);
        assert!(m.get_ellipses().children().is_empty());
    }

    #[test]
    fn set_column_colors_recolors_exactly_the_column() {
        let m = TexMatrix::new(vec![vec!["a", "b"], vec!["c", "d"]])
            .build(&engine())
            .expect("builds")
            .set_column_colors(&[RED, BLUE]);
        fn check_family(vmob: &VMobject, color: Srgb) {
            assert_eq!(vmob.style().fill_color, color);
            assert_eq!(vmob.style().stroke_color, color);
            for child in vmob.children() {
                check_family(child, color);
            }
        }
        // Column 0 red, column 1 blue, through the glyph families.
        for i in 0..2 {
            check_family(&m.vmob.children()[i * 2], RED);
            check_family(&m.vmob.children()[i * 2 + 1], BLUE);
        }
        // The brackets are untouched — identical to a fresh build's.
        let fresh = TexMatrix::new(vec![vec!["a", "b"], vec!["c", "d"]])
            .build(&engine())
            .expect("builds");
        for (recolored, pristine) in m
            .get_brackets()
            .children()
            .iter()
            .zip(fresh.get_brackets().children())
        {
            assert_eq!(recolored.style().fill_color, pristine.style().fill_color);
            assert_eq!(
                recolored.center_point(),
                pristine.center_point(),
                "recoloring must not move geometry"
            );
        }
        // Fewer colors than columns leaves the tail alone — exactly the
        // style an uncolored build gives the same entry.
        let partial = TexMatrix::new(vec![vec!["a", "b"]])
            .build(&engine())
            .expect("builds")
            .set_column_colors(&[RED]);
        check_family(&partial.vmob.children()[0], RED);
        let fresh = TexMatrix::new(vec![vec!["a", "b"]])
            .build(&engine())
            .expect("builds");
        assert_eq!(
            partial.vmob.children()[1].style().fill_color,
            fresh.vmob.children()[1].style().fill_color
        );
    }

    #[test]
    fn swap_entries_for_ellipses_matches_the_reference_structure() {
        // 4×4 with row 1 and column 2 swapped: 4 vdots + 4 hdots − 1
        // shared intersection = 7 ellipses, 16 − 7 = 9 entries left.
        let m = TexMatrix::new(vec![
            vec!["a", "b", "c", "d"],
            vec!["e", "f", "g", "h"],
            vec!["i", "j", "k", "l"],
            vec!["m", "n", "o", "p"],
        ])
        .ellipses_row(1)
        .ellipses_col(2)
        .build(&engine())
        .expect("builds");
        assert_eq!(m.vmob.children().len(), 16 + 2, "swaps replace in place");
        assert_eq!(m.get_ellipses().children().len(), 7);
        assert_eq!(m.get_entries().children().len(), 9);
        // The ellipses children are the swapped grid cells, in place:
        // the row pass swaps 4,5,6,7, then the column pass swaps 2,10,14
        // (the intersection 6 was already swapped and is not duplicated).
        for (e, &k) in m
            .get_ellipses()
            .children()
            .iter()
            .zip([4usize, 5, 6, 7, 2, 10, 14].iter())
        {
            assert_eq!(e.center_point(), m.vmob.children()[k].center_point());
        }
        // Ellipses have real geometry (typeset dots, not empties).
        for e in m.get_ellipses().children() {
            let mut points = e.points().len();
            for child in e.children() {
                points += child.points().len();
            }
            assert!(points > 0, "an ellipse must draw something");
        }
        // Negative indices wrap, as Python's do.
        let wrapped = TexMatrix::new(vec![vec!["a", "b"], vec!["c", "d"]])
            .ellipses_row(-1)
            .build(&engine())
            .expect("builds");
        assert_eq!(wrapped.get_ellipses().children().len(), 2);
        // Out of range swaps nothing.
        let ignored = TexMatrix::new(vec![vec!["a"]])
            .ellipses_row(5)
            .build(&engine())
            .expect("builds");
        assert!(ignored.get_ellipses().children().is_empty());
        assert_eq!(ignored.get_entries().children().len(), 1);
    }

    #[test]
    fn tex_matrix_typesets_every_entry() {
        let m = TexMatrix::new(vec![vec![r"\alpha", "2"], vec!["x", r"\frac{1}{2}"]])
            .build(&engine())
            .expect("builds");
        for (k, child) in m.vmob.children()[..4].iter().enumerate() {
            assert!(!child.children().is_empty(), "entry {k} has glyph children");
            assert!(
                child.extent().is_some(),
                "entry {k} has measurable geometry"
            );
        }
        // The fraction entry is a multi-primitive family (glyphs + rule).
        let frac = &m.vmob.children()[3];
        assert!(frac.children().len() >= 3, "frac children");
    }

    #[test]
    fn decimal_and_integer_matrices_typeset_numbers() {
        let book = book();
        let d = DecimalMatrix::new(vec![vec![1.5, -2.25], vec![3.0, 4.125]])
            .build(&engine(), &book)
            .expect("builds");
        for child in &d.vmob.children()[..4] {
            assert!(child.extent().is_some(), "a digit family has geometry");
        }
        // ndp=2 default: "1.50" is three glyphs wide of digits and a dot.
        assert!(d.vmob.children()[0].children().len() >= 4, "1.50 glyphs");

        let i = IntegerMatrix::new(vec![vec![1.4, 2.6]])
            .build(&engine(), &book)
            .expect("builds");
        for child in &i.vmob.children()[..2] {
            assert!(child.extent().is_some(), "a digit family has geometry");
            // Integers render without a decimal point: single glyph per
            // one-digit value (1, 3 after rounding).
            assert_eq!(child.children().len(), 1, "one digit, no point");
        }
    }

    #[test]
    fn mobject_matrix_partitions_and_passes_entries_through() {
        let entries: Vec<VMobject> = (0..4).map(|_| cell(0.5, 0.5)).collect();
        // Pin the height cap to exactly the natural grid height
        // (2·0.5 + v_buff) + 2·bracket_v_buff = 2.0, so the cap's scale
        // factor is 1 and the passthrough is observable.
        let m = MobjectMatrix::new(entries)
            .n_rows(2)
            .height(2.0)
            .build(&engine())
            .expect("builds");
        assert_eq!(m.n_rows(), 2);
        assert_eq!(m.n_cols(), 2);
        assert_eq!(m.vmob.children().len(), 4 + 2);
        // Entries pass through un-reshaped: each keeps its square extent.
        for child in &m.vmob.children()[..4] {
            assert!(close(child.length_over_dim(0), 0.5));
            assert!(close(child.length_over_dim(1), 0.5));
        }
        // The defaults partition by floor(sqrt(n)).
        let m = MobjectMatrix::new((0..9).map(|_| cell(0.2, 0.2)).collect())
            .build(&engine())
            .expect("builds");
        assert_eq!((m.n_rows(), m.n_cols()), (3, 3));
        // And the default caps the grid at 4.0 − 2·bracket_v_buff tall.
        let rows_height = m.vmob.length_over_dim(1);
        assert!(rows_height <= DEFAULT_MOBJECT_MATRIX_HEIGHT + 1e-9);
        // Too few entries is a typed error, not a panic.
        let err = MobjectMatrix::new(vec![cell(1.0, 1.0)])
            .n_rows(2)
            .n_cols(2)
            .build(&engine());
        assert!(matches!(err, Err(MatrixError::TooFewEntries { .. })));
    }

    #[test]
    fn matrix_entry_budget_accepts_the_boundary_and_refuses_one_over() {
        let exact = Matrix::new(grid_of(2, 2, 0.25, 0.25))
            .entry_limit(4)
            .build(&engine())
            .expect("the declared boundary is admitted");
        assert_eq!((exact.n_rows(), exact.n_cols()), (2, 2));

        let err = Matrix::new(grid_of(2, 2, 0.25, 0.25))
            .entry_limit(3)
            .build(&engine());
        assert!(matches!(
            err,
            Err(MatrixError::ResourceLimit {
                context: "matrix entries",
                requested: 4,
                limit: 3,
            })
        ));
    }

    #[test]
    fn hostile_mobject_matrix_dimensions_are_typed_refusals() {
        let err = MobjectMatrix::new(vec![cell(0.25, 0.25)])
            .n_rows(1)
            .n_cols(2)
            .entry_limit(1)
            .build(&engine());
        assert!(matches!(
            err,
            Err(MatrixError::ResourceLimit {
                context: "matrix entries",
                requested: 2,
                limit: 1,
            })
        ));

        let err = MobjectMatrix::new(vec![cell(0.25, 0.25)])
            .n_rows(usize::MAX)
            .n_cols(2)
            .entry_limit(usize::MAX)
            .build(&engine());
        assert!(matches!(
            err,
            Err(MatrixError::CapacityOverflow {
                context: "matrix entry count",
            })
        ));
    }

    #[test]
    fn accessors_refuse_unrepresentable_internal_ranges_without_panicking() {
        let malformed = MatrixMobject {
            vmob: VMobject::new(),
            n_rows: usize::MAX,
            n_cols: usize::MAX,
            n_entries: usize::MAX,
            ellipses: vec![usize::MAX],
            brackets_drawn: false,
        };
        assert!(malformed.get_row(0).is_none());
        assert!(malformed.get_row(2).is_none());
        assert!(malformed.get_column(0).is_none());
        assert!(malformed.get_entries().children().is_empty());
        assert!(malformed.get_brackets().children().is_empty());
        assert!(malformed.get_ellipses().children().is_empty());
    }

    #[test]
    fn empty_and_ragged_grids_are_typed_errors() {
        let err = Matrix::new(Vec::new()).build(&engine());
        assert!(matches!(err, Err(MatrixError::Empty)));
        let err = Matrix::new(vec![vec![cell(1.0, 1.0)], Vec::new()]).build(&engine());
        assert!(matches!(err, Err(MatrixError::Ragged { .. })));
        let err = Matrix::new(vec![Vec::new()]).build(&engine());
        assert!(matches!(err, Err(MatrixError::Empty)));

        // Shape validation precedes entry typesetting.
        let err = TexMatrix::new(vec![vec![r"\not-a-command"], Vec::new()]).build(&engine());
        assert!(matches!(err, Err(MatrixError::Ragged { .. })));
    }
}
