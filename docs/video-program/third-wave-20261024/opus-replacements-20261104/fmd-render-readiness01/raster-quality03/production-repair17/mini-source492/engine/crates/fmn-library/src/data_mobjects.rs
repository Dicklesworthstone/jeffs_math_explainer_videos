//! The enhanced-tier Data mobjects (§12.6 leapfrog #8, fm-n64 tranche 2):
//! [`TableMobject`] renders a frankenpandas [`fp_frame::DataFrame`] (or any
//! string grid) through Scribe; [`BarChart`] ports the Reference's
//! `mobject/probability.py` `BarChart` faithfully, fed by plain values or
//! straight out of a frame column.
//!
//! ## Conventions
//!
//! * **CSV ingestion** rides `Frame::from_csv` — the suite's parser, not a
//!   second one here. Column order is the frame's `column_names()`
//!   (insertion order, matching pandas' visible order); every cell goes
//!   through [`format_scalar`], whose single documented rule set is: null →
//!   empty string, bool → `true`/`false`, Int64 → decimal, Float64 → Rust's
//!   shortest round-trip form, Utf8 verbatim, Timedelta64/Datetime64 → the
//!   raw nanosecond integer followed by `ns` (Behavior Note: pandas would
//!   localize datetimes; the wall-clock rendering is deliberately deferred
//!   until a corpus scene needs it).
//! * **BarChart geometry** is the Reference's, constant for constant:
//!   `buff = width / (2n)` bar pitch, `height·v/max` bar heights,
//!   `(2i + ½)·buff` bottom-left anchors, `linspace` y-ticks with
//!   `tick_width`-wide marks and `y_axis_label_height` labels, the
//!   `[BLUE, YELLOW]` gradient across bars, `bar_fill_opacity 0.8`,
//!   `bar_stroke_width 3`. Y-axis label text rounds through
//!   [`numpy_round2`] — numpy's half-to-even, not Rust's half-away.
//! * **Determinism**: both families are pure functions of their inputs;
//!   no RNG anywhere. Same table/chart ⇒ bit-identical geometry.

use std::collections::BTreeMap;

use fmn_core::constants::{BLACK, BLUE, DOWN, LEFT, MED_LARGE_BUFF, RIGHT, SMALL_BUFF, UP, YELLOW};
use fmn_core::types::Vec3;
use fmn_text::FontBook;

use fp_frame::DataFrame;

use crate::line::Line;
use crate::poly::Rectangle;
use crate::style::Style;
use crate::text::Text;
use crate::vmobject::{VMobject, v_group};

/// Cell padding fraction of the body font's line box.
const CELL_PADDING_X: f64 = 0.18;
/// Vertical padding per row, in units.
const CELL_PADDING_Y: f64 = 0.12;
/// Table rule stroke width.
const RULE_STROKE_WIDTH: f64 = 1.5;
/// Header rule is heavier than the body rules.
const HEADER_RULE_STROKE_WIDTH: f64 = 2.5;

/// Why a data mobject could not be built.
#[derive(Debug, Clone, PartialEq)]
pub enum DataMobjectError {
    /// The frame refused the CSV bytes; the parser's named error verbatim.
    Csv(String),
    /// A frame with zero columns or zero data rows cannot make a table.
    EmptyFrame,
    /// A chart was asked to build with no values at all.
    NoValues,
    /// A bar-values slice did not match the constructed bar count.
    BarValueCountMismatch {
        /// How many bars exist.
        bars: usize,
        /// How many replacement values arrived.
        values: usize,
    },
    /// A geometry primitive refused construction.
    Geometry(String),
}

impl std::fmt::Display for DataMobjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv(detail) => write!(f, "CSV refused by the frame parser: {detail}"),
            Self::EmptyFrame => write!(f, "frame has no columns or no data rows"),
            Self::NoValues => write!(f, "chart has no values to draw"),
            Self::BarValueCountMismatch { bars, values } => {
                write!(f, "got {values} replacement bar values for {bars} bars")
            }
            Self::Geometry(detail) => write!(f, "data family geometry refused: {detail}"),
        }
    }
}

impl std::error::Error for DataMobjectError {}

/// One documented rendering rule per [`Scalar`](fp_types::Scalar) variant.
#[must_use]
pub fn format_scalar(value: Option<&fp_types::Scalar>) -> String {
    use fp_types::Scalar;
    match value {
        None | Some(Scalar::Null(_)) => String::new(),
        Some(Scalar::Bool(b)) => b.to_string(),
        Some(Scalar::Int64(i)) => i.to_string(),
        Some(Scalar::Float64(x)) => fmt_float(*x),
        Some(Scalar::Utf8(s)) => s.clone(),
        Some(Scalar::Timedelta64(ns)) => format!("{ns}ns"),
        Some(Scalar::Datetime64(ns)) => format!("{ns}ns"),
        Some(other) => format!("{other:?}"),
    }
}

/// Rust's shortest round-trip float form; `-0` normalizes to `0`.
#[must_use]
pub fn fmt_float(x: f64) -> String {
    if x == 0.0 {
        return "0".to_owned();
    }
    let s = x.to_string();
    s.strip_suffix(".0").unwrap_or(&s).to_owned()
}

/// numpy's `round(value, 2)`: half-to-even at the second decimal.
#[must_use]
pub fn numpy_round2(value: f64) -> f64 {
    let scaled = value * 100.0;
    let floor = scaled.floor();
    let frac = scaled - floor;
    let rounded = if (frac - 0.5).abs() < 1e-9 {
        if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        scaled.round()
    };
    rounded / 100.0
}

// --------------------------------------------------------------- Table

/// A string grid ready to lay out through Scribe: headers plus body rows,
/// every cell already rendered by [`format_scalar`] when built from a
/// frame.
#[derive(Debug, Clone, PartialEq)]
pub struct TableMobject {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TableMobject {
    /// Ingest CSV text through the suite's frame parser.
    ///
    /// # Errors
    /// [`DataMobjectError::Csv`] for parser refusals, [`DataMobjectError::
    /// EmptyFrame`] for a degenerate frame.
    pub fn from_csv(text: &str, separator: char) -> Result<Self, DataMobjectError> {
        let frame = DataFrame::from_csv(text, separator)
            .map_err(|error| DataMobjectError::Csv(error.to_string()))?;
        Self::from_frame(&frame)
    }

    /// Project an existing frame: headers in insertion order, cells through
    /// [`format_scalar`].
    ///
    /// # Errors
    /// [`DataMobjectError::EmptyFrame`] for a degenerate frame.
    pub fn from_frame(frame: &DataFrame) -> Result<Self, DataMobjectError> {
        let headers: Vec<String> = frame
            .column_names()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();
        let (row_count, column_count) = frame.shape();
        if headers.is_empty() || column_count == 0 || row_count == 0 {
            return Err(DataMobjectError::EmptyFrame);
        }
        let columns: BTreeMap<String, Vec<String>> = headers
            .iter()
            .map(|header| {
                let column = frame
                    .column(header)
                    .expect("column exists — name came from the frame");
                (
                    header.clone(),
                    (0..row_count)
                        .map(|i| format_scalar(column.value(i)))
                        .collect(),
                )
            })
            .collect();
        let rows = (0..row_count)
            .map(|r| headers.iter().map(|h| columns[h][r].clone()).collect())
            .collect();
        Ok(Self { headers, rows })
    }

    /// A grid straight from the caller (tests, hand-built tables).
    #[must_use]
    pub fn from_grid(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self { headers, rows }
    }

    /// Headers in display order.
    #[must_use]
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Body rows in display order.
    #[must_use]
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Lay the table out through Scribe: a header band, body rows, an outer
    /// rectangle, a heavy rule under the header, light rules between rows
    /// and columns. Left-aligned cells; column widths derive from the
    /// widest cell.
    ///
    /// # Errors
    /// [`DataMobjectError::Geometry`] when Scribe refuses a cell.
    pub fn build(&self, book: &FontBook) -> Result<VMobject, DataMobjectError> {
        let mut texts: Vec<VMobject> = Vec::new();
        let column_count = self.headers.len();
        let measure =
            |cell: &str, texts: &mut Vec<VMobject>| -> Result<[f64; 2], DataMobjectError> {
                let mob = Text::new(cell)
                    .build(book)
                    .map_err(|error| DataMobjectError::Geometry(error.to_string()))?
                    .vmob;
                let right = mob.bbox_point(RIGHT);
                let left = mob.bbox_point(LEFT);
                let up = mob.bbox_point(UP);
                let down = mob.bbox_point(DOWN);
                let width = match (right, left) {
                    (Some(r), Some(l)) => r[0] - l[0],
                    _ => 0.0,
                };
                let height = match (up, down) {
                    (Some(u), Some(d)) => u[1] - d[1],
                    _ => 0.0,
                };
                texts.push(mob);
                Ok([width, height])
            };

        let mut widths = vec![0.0_f64; column_count];
        let mut row_heights = vec![0.0_f64; self.rows.len() + 1];
        for (c, header) in self.headers.iter().enumerate() {
            let [w, h] = measure(header, &mut texts)?;
            widths[c] = widths[c].max(w);
            row_heights[0] = row_heights[0].max(h);
        }
        for (r, row) in self.rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let [w, h] = measure(cell, &mut texts)?;
                widths[c] = widths[c].max(w);
                row_heights[r + 1] = row_heights[r + 1].max(h);
            }
        }

        // Guard degenerate empties so rules never collapse onto themselves.
        for width in &mut widths {
            *width += 2.0 * CELL_PADDING_X;
        }
        for height in &mut row_heights {
            *height += CELL_PADDING_Y;
        }
        let total_width: f64 = widths.iter().sum();
        let total_height: f64 = row_heights.iter().sum();
        // Place cells: header band first, then body rows; origin at the
        // table's lower-left corner.
        let mut placed: Vec<VMobject> = Vec::with_capacity(texts.len());
        let mut cell_iter = texts.into_iter();
        let mut y_cursor = total_height;
        for height in row_heights.iter() {
            let row_top = y_cursor;
            let baseline_center = row_top - height / 2.0;
            let mut x_cursor = 0.0;
            for width in widths.iter().take(column_count) {
                let Some(mut mob) = cell_iter.next() else {
                    break;
                };
                let center_x = x_cursor + width / 2.0;
                mob = mob.moved_to([center_x, baseline_center, 0.0]);
                placed.push(mob);
                x_cursor += width;
            }
            y_cursor -= height;
        }

        // Rules: outline, header rule, row separators, column separators.
        let mut rules: Vec<VMobject> = Vec::new();
        let outline = Rectangle::new()
            .width(total_width)
            .height(total_height)
            .build()
            .map_err(|error| DataMobjectError::Geometry(error.to_string()))?
            .moved_to([total_width / 2.0, total_height / 2.0, 0.0])
            .map_style_deep(|style| style.stroke(BLACK, RULE_STROKE_WIDTH, 1.0));
        rules.push(outline);

        let header_rule_height = row_heights[0];
        rules.push(horizontal_rule(
            total_width,
            total_height - header_rule_height,
            HEADER_RULE_STROKE_WIDTH,
        )?);
        for r in 1..self.rows.len() {
            let top = row_heights[..=r].iter().sum::<f64>();
            rules.push(horizontal_rule(
                total_width,
                total_height - top,
                RULE_STROKE_WIDTH,
            )?);
        }
        let mut x_cursor = 0.0;
        for width in &widths[..column_count - 1] {
            x_cursor += width;
            let line = Line::new([x_cursor, 0.0, 0.0], [x_cursor, total_height, 0.0])
                .style(Style::default().stroke(BLACK, RULE_STROKE_WIDTH, 1.0))
                .build()
                .map_err(|error| DataMobjectError::Geometry(error.to_string()))?;
            rules.push(line);
        }

        let group = v_group(rules);
        Ok(group.with_children(placed))
    }
}

fn horizontal_rule(width: f64, y: f64, stroke: f64) -> Result<VMobject, DataMobjectError> {
    Line::new([0.0, y, 0.0], [width, y, 0.0])
        .style(Style::default().stroke(BLACK, stroke, 1.0))
        .build()
        .map_err(|error| DataMobjectError::Geometry(error.to_string()))
}

// ------------------------------------------------------------ BarChart

/// The Reference's defaults for [`BarChart`].
pub const BAR_CHART_HEIGHT: f64 = 4.0;
/// Reference default chart width.
pub const BAR_CHART_WIDTH: f64 = 6.0;
/// Reference default tick count on the value axis.
pub const BAR_CHART_N_TICKS: usize = 4;

/// The Reference's `BarChart`: axes, ticks, gradient-filled bars, optional
/// y-axis labels and x-axis names — geometry constant-for-constant with
/// `mobject/probability.py`.
#[derive(Debug, Clone)]
pub struct BarChart {
    values: Vec<f64>,
    height: f64,
    width: f64,
    n_ticks: usize,
    include_x_ticks: bool,
    tick_width: f64,
    tick_height: f64,
    label_y_axis: bool,
    y_axis_label_height: f64,
    max_value: Option<f64>,
    bar_colors: Vec<fmn_core::color::Srgb>,
    bar_fill_opacity: f64,
    bar_stroke_width: f64,
    bar_names: Vec<String>,
    bar_label_scale_val: f64,
}

impl Default for BarChart {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl BarChart {
    /// `BarChart(values, …)` with every Reference default.
    #[must_use]
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            height: BAR_CHART_HEIGHT,
            width: BAR_CHART_WIDTH,
            n_ticks: BAR_CHART_N_TICKS,
            include_x_ticks: false,
            tick_width: 0.2,
            tick_height: 0.15,
            label_y_axis: true,
            y_axis_label_height: 0.25,
            max_value: Some(1.0),
            bar_colors: vec![BLUE, YELLOW],
            bar_fill_opacity: 0.8,
            bar_stroke_width: 3.0,
            bar_names: Vec::new(),
            bar_label_scale_val: 0.75,
        }
    }

    /// Chart height.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// Chart width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Value-axis tick count.
    #[must_use]
    pub fn n_ticks(mut self, n_ticks: usize) -> Self {
        self.n_ticks = n_ticks;
        self
    }

    /// Draw category ticks along the value axis.
    #[must_use]
    pub fn include_x_ticks(mut self, include: bool) -> Self {
        self.include_x_ticks = include;
        self
    }

    /// Pin the scale top; [`None`] derives it from `max(values)`.
    #[must_use]
    pub fn max_value(mut self, max_value: Option<f64>) -> Self {
        self.max_value = max_value;
        self
    }

    /// Draw numeric labels beside the value-axis ticks.
    #[must_use]
    pub fn label_y_axis(mut self, label: bool) -> Self {
        self.label_y_axis = label;
        self
    }

    /// Category names under the bars.
    #[must_use]
    pub fn bar_names(mut self, bar_names: Vec<String>) -> Self {
        self.bar_names = bar_names;
        self
    }

    /// Build the whole chart through Scribe for its labels.
    ///
    /// # Errors
    /// [`DataMobjectError`] for refused primitives or empty values with a
    /// derived max.
    pub fn build(&self, book: &FontBook) -> Result<VMobject, DataMobjectError> {
        if self.values.is_empty() {
            return Err(DataMobjectError::NoValues);
        }
        let max_value = match self.max_value {
            Some(max) => max,
            None => self
                .values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
        };
        let mut members: Vec<VMobject> = Vec::new();

        // add_axes: x then y axis, ticks, optional labels.
        let x_axis_start: Vec3 = [-self.tick_width / 2.0, 0.0, 0.0];
        let x_axis_end: Vec3 = [self.width, 0.0, 0.0];
        members.push(line_styled(x_axis_start, x_axis_end)?);
        let y_axis_start: Vec3 = [0.0, -MED_LARGE_BUFF, 0.0];
        let y_axis_end: Vec3 = [0.0, self.height, 0.0];
        members.push(line_styled(y_axis_start, y_axis_end)?);

        let tick_count = self.n_ticks.max(1);
        let mut y_tick_points: Vec<(f64, f64)> = Vec::new();
        for k in 0..=tick_count {
            let fraction = k as f64 / tick_count as f64;
            let y = fraction * self.height;
            let value = fraction * max_value;
            y_tick_points.push((y, value));
            let tick = line_styled(
                [-self.tick_width / 2.0, y, 0.0],
                [self.tick_width / 2.0, y, 0.0],
            )?;
            members.push(tick);
        }
        if self.include_x_ticks {
            let n = self.values.len();
            for k in 0..=n {
                let x = k as f64 * self.width / n as f64;
                members.push(line_styled(
                    [x, -self.tick_height / 2.0, 0.0],
                    [x, self.tick_height / 2.0, 0.0],
                )?);
            }
        }
        if self.label_y_axis {
            for (y, value) in y_tick_points.iter().skip(1).copied() {
                let rounded = numpy_round2(value);
                let label_text = if (rounded - rounded.trunc()).abs() < 1e-9 {
                    format!("{}", rounded as i64)
                } else {
                    format!("{rounded}")
                };
                let mut label = Text::new(&label_text)
                    .build(book)
                    .map_err(|error| DataMobjectError::Geometry(error.to_string()))?
                    .vmob;
                label = scale_to_height(label, self.y_axis_label_height);
                label =
                    label.next_to_point([-self.tick_width / 2.0, y, 0.0], LEFT, SMALL_BUFF, RIGHT);
                members.push(label);
            }
        }

        // add_bars.
        let count = self.values.len();
        let buff = self.width / (2.0 * count as f64);
        let colors = fmn_core::color::color_gradient(&self.bar_colors, count);
        let mut bars: Vec<VMobject> = Vec::new();
        for (i, value) in self.values.iter().enumerate() {
            let bar_height = (value / max_value) * self.height;
            let bar = Rectangle::new()
                .width(buff)
                .height(bar_height)
                .build()
                .map_err(|error| DataMobjectError::Geometry(error.to_string()))?;
            let anchor_x = (2 * i + 1) as f64 * buff * 0.5;
            let bar = bar
                .moved_to_aligned([anchor_x, 0.0, 0.0], [-5.0, -1.0, 0.0])
                .map_style_deep(|style| {
                    style
                        .color(colors[i])
                        .fill_opacity(self.bar_fill_opacity)
                        .stroke_width(self.bar_stroke_width)
                });
            bars.push(bar);
        }
        members.push(v_group(bars));

        for (i, name) in self.bar_names.iter().enumerate().take(count) {
            let anchor_x = (2 * i + 1) as f64 * buff * 0.5;
            let mut label = Text::new(name)
                .build(book)
                .map_err(|error| DataMobjectError::Geometry(error.to_string()))?
                .vmob;
            label = scale_to_height(label, self.bar_label_scale_val);
            label = label.next_to_point([anchor_x, 0.0, 0.0], DOWN, SMALL_BUFF, UP);
            members.push(label);
        }

        let group = v_group(members);
        // The Reference ends __init__ with self.center().
        Ok(group.moved_to([0.0, 0.0, 0.0]))
    }
}

fn line_styled(start: Vec3, end: Vec3) -> Result<VMobject, DataMobjectError> {
    Line::new(start, end)
        .style(Style::default().stroke(BLACK, 2.5, 1.0))
        .build()
        .map_err(|error| DataMobjectError::Geometry(error.to_string()))
}

fn scale_to_height(mob: VMobject, target_height: f64) -> VMobject {
    let up = mob.bbox_point(UP);
    let down = mob.bbox_point(DOWN);
    let current = match (up, down) {
        (Some(u), Some(d)) => u[1] - d[1],
        _ => 0.0,
    };
    if current > 1e-12 && (current - target_height).abs() > 1e-12 {
        {
            let center = mob.center_point();
            mob.scaled_about(target_height / current, center)
        }
    } else {
        mob
    }
}
