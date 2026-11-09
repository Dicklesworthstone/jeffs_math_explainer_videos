//! The enhanced-tier probability module (fm-n64 tranche 3): [`SampleSpace`]
//! ports the Reference's `mobject/probability.py` `SampleSpace` — the base
//! rectangle, p-list completion, horizontal/vertical divisions, and
//! brace-and-label subdivisions — through Scribe text and this crate's
//! parametric [`Brace`].
//!
//! Geometry conventions are the Reference's, constant for constant:
//!
//! * A horizontal division (`vect = DOWN`) stacks the completed `p_list`
//!   bands from the **top** edge downward; a vertical division
//!   (`vect = RIGHT`) stacks from the **left** edge rightward. The first
//!   band carries `p_list[0]`, not the remainder — only
//!   `complete_p_list` appends the leftover slice.
//! * Band fills run through [`fmn_core::color::color_gradient`] over the
//!   caller's anchors at the completed list length; strokes stay the base
//!   rectangle's 0.5-wide `GREY_B`.
//! * Labels are plain Scribe [`Text`] scaled by
//!   `default_label_scale_val`, seated one `buff` beyond their brace in
//!   the brace's own direction.
//!
//! Pure geometry: no RNG, no clock — identical inputs build identical
//! families.

use fmn_core::constants::{
    BLUE_E, DOWN, GREEN_E, GREY_B, GREY_D, LEFT, MAROON_B, MED_SMALL_BUFF, RIGHT, UP, YELLOW,
};
use fmn_core::types::Vec3;
use fmn_text::FontBook;

use crate::brace::Brace;
use crate::poly::Rectangle;
use crate::text::Text;
use crate::vmobject::{VMobject, v_group};

/// The Reference's `EPSILON` gate for appending the remainder band.
const P_LIST_EPSILON: f64 = 1e-8;

/// Why a sample-space family could not be built.
#[derive(Debug, Clone, PartialEq)]
pub enum ProbabilityError {
    /// A probability outside `[0, 1]`, or a list whose completion overflows
    /// past zero within `P_LIST_EPSILON`.
    BadProbability {
        /// The offending value.
        value: f64,
    },
    /// Scribe refused a title or band label.
    Text(String),
    /// A geometry primitive refused construction.
    Geometry(String),
}

impl std::fmt::Display for ProbabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadProbability { value } => {
                write!(f, "probability {value} is outside [0, 1]")
            }
            Self::Text(detail) => write!(f, "sample-space text refused: {detail}"),
            Self::Geometry(detail) => write!(f, "probability geometry refused: {detail}"),
        }
    }
}

impl std::error::Error for ProbabilityError {}

/// The Reference's `SampleSpace`: a unit-probability rectangle that divides
/// horizontally or vertically into filled bands with optional braces and
/// labels.
#[derive(Debug, Clone)]
pub struct SampleSpace {
    width: f64,
    height: f64,
    fill_color: fmn_core::color::Srgb,
    fill_opacity: f64,
    stroke_width: f64,
    stroke_color: fmn_core::color::Srgb,
    default_label_scale_val: f64,
}

impl Default for SampleSpace {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleSpace {
    /// `SampleSpace()` — three units square, `GREY_D` body, hairline
    /// `GREY_B` frame.
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 3.0,
            height: 3.0,
            fill_color: GREY_D,
            fill_opacity: 1.0,
            stroke_width: 0.5,
            stroke_color: GREY_B,
            default_label_scale_val: 1.0,
        }
    }

    /// Space width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Space height.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// The scale applied to subdivision labels.
    #[must_use]
    pub fn label_scale(mut self, scale: f64) -> Self {
        self.default_label_scale_val = scale;
        self
    }

    /// Build the undivided base rectangle centred at the origin.
    ///
    /// # Errors
    /// [`ProbabilityError::Geometry`] if the rectangle refuses.
    pub fn build(&self) -> Result<VMobject, ProbabilityError> {
        let rect = Rectangle::new()
            .width(self.width)
            .height(self.height)
            .build()
            .map_err(|error| ProbabilityError::Geometry(error.to_string()))?;
        Ok(rect.map_style_deep(|style| {
            style.fill(self.fill_color, self.fill_opacity).stroke(
                self.stroke_color,
                self.stroke_width,
                1.0,
            )
        }))
    }

    /// `add_title`: the title above the space, shrunk to the space's width
    /// when wider, one `MED_SMALL_BUFF` away.
    ///
    /// # Errors
    /// [`ProbabilityError::Text`] from Scribe.
    pub fn title(
        &self,
        book: &FontBook,
        title: &str,
        buff: f64,
    ) -> Result<VMobject, ProbabilityError> {
        let mut title_mob = Text::new(title)
            .build(book)
            .map_err(|error| ProbabilityError::Text(error.to_string()))?
            .vmob;
        let right = title_mob.bbox_point(RIGHT);
        let left = title_mob.bbox_point(LEFT);
        if let (Some(r), Some(l)) = (right, left) {
            let width = r[0] - l[0];
            if width > self.width && width > 1e-12 {
                let center = title_mob.center_point();
                title_mob = title_mob.scaled_about(self.width / width, center);
            }
        }
        Ok(title_mob.next_to_point([0.0, self.height / 2.0, 0.0], UP, buff, DOWN))
    }

    /// `complete_p_list`: pad with the remainder so the slices sum to one.
    ///
    /// # Errors
    /// [`ProbabilityError::BadProbability`] for entries outside `[0, 1]`.
    pub fn complete_p_list(&self, p_list: &[f64]) -> Result<Vec<f64>, ProbabilityError> {
        for value in p_list {
            if !(*value >= 0.0 && *value <= 1.0) {
                return Err(ProbabilityError::BadProbability { value: *value });
            }
        }
        let mut completed = p_list.to_vec();
        let remainder = 1.0 - completed.iter().sum::<f64>();
        if completed.is_empty() || (remainder - 0.0).abs() > P_LIST_EPSILON {
            completed.push(remainder);
        }
        Ok(completed)
    }

    /// `get_horizontal_division`: bands stacked from the top edge downward,
    /// colored along `colors`. The Reference defaults are
    /// [`GREEN_E`] →
    /// [`BLUE_E`].
    ///
    /// # Errors
    /// [`ProbabilityError`] for bad probabilities or refused geometry.
    pub fn horizontal_division(
        &self,
        p_list: &[f64],
        colors: &[fmn_core::color::Srgb],
    ) -> Result<VMobject, ProbabilityError> {
        self.division(p_list, true, colors)
    }

    /// `get_vertical_division`: bands stacked from the left edge rightward,
    /// colored along `colors`. The Reference defaults are
    /// [`MAROON_B`] →
    /// [`YELLOW`].
    ///
    /// # Errors
    /// [`ProbabilityError`] for bad probabilities or refused geometry.
    pub fn vertical_division(
        &self,
        p_list: &[f64],
        colors: &[fmn_core::color::Srgb],
    ) -> Result<VMobject, ProbabilityError> {
        self.division(p_list, false, colors)
    }

    /// `get_subdivision_braces_and_labels`: one brace plus one scaled label
    /// per band, seated in `direction` beyond its band.
    ///
    /// # Errors
    /// [`ProbabilityError::Text`] from Scribe.
    pub fn subdivision_braces_and_labels(
        &self,
        parts: &VMobject,
        labels: &[&str],
        direction: Vec3,
        buff: f64,
        book: &FontBook,
    ) -> Result<VMobject, ProbabilityError> {
        let mut braces: Vec<VMobject> = Vec::new();
        let mut label_mobs: Vec<VMobject> = Vec::new();
        for (label, part) in labels.iter().zip(parts.children()) {
            let brace = Brace::around(part, direction).buff(buff).build();
            let mut label_mob = Text::new(label)
                .build(book)
                .map_err(|error| ProbabilityError::Text(error.to_string()))?
                .vmob;
            let center = label_mob.center_point();
            label_mob = label_mob.scaled_about(self.default_label_scale_val, center);
            label_mob = label_mob.next_to(&brace, direction, buff, direction);
            braces.push(brace);
            label_mobs.push(label_mob);
        }
        let brace_group = v_group(braces);
        Ok(brace_group.with_children(label_mobs))
    }

    // ------------------------------------------------------- internals

    fn division(
        &self,
        p_list: &[f64],
        horizontal: bool,
        colors: &[fmn_core::color::Srgb],
    ) -> Result<VMobject, ProbabilityError> {
        let completed = self.complete_p_list(p_list)?;
        let palette = fmn_core::color::color_gradient(colors, completed.len());
        let mut parts: Vec<VMobject> = Vec::new();
        // The Reference walks from the -vect edge: for vect DOWN that is the
        // TOP edge (stack downward), for vect RIGHT the LEFT edge (stack
        // rightward).
        let span = if horizontal { self.height } else { self.width };
        let mut consumed = 0.0_f64;
        for (index, factor) in completed.iter().enumerate() {
            let band_span = factor * span;
            let (band_width, band_height) = if horizontal {
                (self.width, band_span)
            } else {
                (band_span, self.height)
            };
            let band = Rectangle::new()
                .width(band_width)
                .height(band_height)
                .build()
                .map_err(|error| ProbabilityError::Geometry(error.to_string()))?
                .map_style_deep(|style| style.fill(palette[index], 1.0).stroke(GREY_B, 0.5, 1.0));
            // Bands stack from the centred frame's edge: horizontal runs
            // downward from +height/2, vertical rightward from -width/2.
            let centre_along = if horizontal {
                self.height / 2.0 - consumed - band_span / 2.0
            } else {
                -self.width / 2.0 + consumed + band_span / 2.0
            };
            let centre: Vec3 = if horizontal {
                [self.width / 2.0, centre_along, 0.0]
            } else {
                [centre_along, self.height / 2.0, 0.0]
            };
            parts.push(band.moved_to(centre));
            consumed += band_span;
        }
        Ok(v_group(parts))
    }
}

/// The Reference's default horizontal palette anchors.
#[must_use]
pub fn horizontal_default_colors() -> Vec<fmn_core::color::Srgb> {
    vec![GREEN_E, BLUE_E]
}

/// The Reference's default vertical palette anchors.
#[must_use]
pub fn vertical_default_colors() -> Vec<fmn_core::color::Srgb> {
    vec![MAROON_B, YELLOW]
}

/// The Reference's `add_title` buff.
pub const TITLE_BUFF: f64 = MED_SMALL_BUFF;

/// The stacking direction of [`SampleSpace::horizontal_division`], kept as
/// a named constant so callers composing divisions read like the Reference.
pub const HORIZONTAL_STACK_VECT: Vec3 = DOWN;
/// The stacking direction of [`SampleSpace::vertical_division`].
pub const VERTICAL_STACK_VECT: Vec3 = RIGHT;
