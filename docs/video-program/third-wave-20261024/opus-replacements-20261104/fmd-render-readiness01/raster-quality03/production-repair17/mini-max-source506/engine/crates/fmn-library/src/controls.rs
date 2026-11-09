//! The `interactive.py` control constructions (§12.3, fm-ebl) — the visual
//! compositions plus their value surfaces.
//!
//! # Scope
//!
//! Every class here is a **composition with state**: the mobjects the
//! Reference's constructor assembles, plus the state mutators
//! (`set_value` / `toggle_value` / `set_active` / `open_panel` …) that
//! rewrite the right records when the value changes. What is deliberately
//! NOT here is the event plumbing. The Reference reaches interactivity by
//! registering listeners on the window (`add_mouse_drag_listner`,
//! `add_mouse_press_listner`, `add_mouse_scroll_listner`,
//! `add_key_press_listner`) and by two bookkeeping devices — the no-op
//! `add_updater(lambda mob: None)` that keeps a control out of
//! `lock_static_mobject_data`, and `fix_in_frame`, which pins the control
//! to the camera frame. All of that lands with **W9EVENTS** in Proscenium.
//! Concretely, the omitted hooks are:
//!
//! * `MotionMobject.mob_on_mouse_drag` (drag-to-move),
//! * `Button`'s `on_click` dispatch on mouse press,
//! * `EnableDisableButton.on_mouse_press` / `Checkbox.on_mouse_press`
//!   (click-to-toggle),
//! * `LinearNumberSlider.slider_on_mouse_drag` (its pure half,
//!   [`LinearNumberSlider::value_from_point`], IS here and is tested),
//! * `Textbox.box_on_mouse_press` and `Textbox.on_key_press` (keyboard
//!   editing),
//! * `ControlPanel.panel_opener_on_mouse_drag` /
//!   `ControlPanel.panel_on_mouse_scroll`,
//! * every `fix_in_frame` pin and every keep-alive no-op updater.
//!
//! None of these are replaced by stubs: the compositions stand on their
//! own, the mutators the handlers would have called are public, and the
//! event layer will call exactly them.
//!
//! # `ControlMobject` IS a `ValueTracker` — the tracker design
//!
//! The Reference's control base class is a `ValueTracker` with mobjects
//! grafted on, so a control's value animates through the same machinery as
//! everything else. `fmn_mobject::dynamics` already carries the Stage-side
//! half (`add_value_tracker`, `set_tracker_value`, `always_redraw`,
//! `f_always`). The design chosen here: **the detached builder carries the
//! initial value, and a companion Stage-side helper,
//! [`add_scalar_control`], registers the tracker and binds the composition
//! to it** — rather than making every control constructor take `&mut
//! Stage`. Rationale:
//!
//! * It keeps the library's G0-1 value convention intact: a control is
//!   built, positioned, and tested detached, and enters the arena through
//!   one explicit call — exactly how the scene runtime consumes every
//!   other library class.
//! * The binding mechanism is the one `dynamics.rs` ratified:
//!   [`Stage::always_redraw`] rebuilds the composition from the tracker
//!   value every tick, so `set_tracker_value` (or any animation of the
//!   tracker) drives the control with zero additional plumbing. The
//!   container handle is stable across rebuilds, so the control can be
//!   animated and positioned as one object.
//! * Boolean controls ride the scalar lane as `1.0`/`0.0` (the f64 lanes
//!   are the only encoding the tracker stores); `Textbox`, whose value is
//!   a string, cannot ride a scalar tracker at all — in the Reference it
//!   abuses `dtype=object` for this, and here it simply is not a
//!   [`ScalarControl`] (its mutators are the whole surface). `ColorSliders`
//!   is a `Group` in the Reference, not a control; each of its four
//!   sliders is independently bindable.
//!
//! # Reference quirks kept (and noted)
//!
//! * **`EnableDisableButton` constructs WHITE.** The Reference's
//!   `ValueTracker.__init__` stores the value directly and never
//!   dispatches `set_value_anim`, so the box keeps its constructor fill
//!   (the mobject-default colour at opacity 1) until the first
//!   `set_value`/toggle. Kept verbatim — the first mutator call applies
//!   the enable/disable colour.
//! * **`LinearNumberSlider` parks its handle at the axis midpoint at
//!   construction**, whatever the value — the Reference's
//!   `slider.move_to(slider_axis)` runs before the tracker exists and
//!   `set_value_anim` never fires at init. For the default
//!   `value=0, min=-10, max=10` the midpoint IS the value's fraction, so
//!   the quirk is invisible there; [`LinearNumberSlider::set_value`]
//!   always moves the handle to the true fraction.
//! * **Clamping instead of `assert`.** The Reference's `assert_value`
//!   raises outside `[min_value, max_value]`; panics are not an error
//!   channel here, so [`LinearNumberSlider::set_value`] clamps into
//!   range. The drag path (`value_from_point`) produces in-range values
//!   by construction, exactly as in the Reference.
//! * **Marks are drawn, not stroked (BN-08).** The Reference's checkbox
//!   draws two `Line`s for the checkmark and two for the cross; here the
//!   content is the native drawn pair [`crate::matchers::checkmark`] /
//!   [`crate::matchers::exmark`], the same drawn siblings the rest of
//!   the library uses, stretched onto the box and scaled by the
//!   Reference's hard `0.5` (its `box_content_buff` field is dead in the
//!   Reference and is not reproduced).
//! * **The alpha slider's BLACK→WHITE gradient handle is approximated**
//!   with the gradient's midpoint grey (`GREY_C`): per-point fill
//!   gradients are a record-plane feature the style value does not carry.

use std::rc::Rc;

use fmn_core::color::Srgb;
use fmn_core::constants::{
    DEFAULT_MOBJECT_COLOR, DEFAULT_MOBJECT_TO_MOBJECT_BUFF, DEFAULT_STROKE_WIDTH,
    DEFAULT_VMOBJECT_STROKE_COLOR, DOWN, FRAME_HEIGHT, FRAME_WIDTH, FRAME_X_RADIUS, FRAME_Y_RADIUS,
    GREEN, GREY_A, GREY_C, LEFT, MED_SMALL_BUFF, ORIGIN, RED, RIGHT, SMALL_BUFF, UP, WHITE,
};
use fmn_core::types::Vec3;
use fmn_geom::GeomError;
use fmn_mobject::Mobject;
use fmn_mobject::stage::{Mob, Stage};
use fmn_text::FontBook;

use crate::arc::Circle;
use crate::line::Line;
use crate::matchers::{SurroundingRectangle, checkmark, exmark};
use crate::numbers::DecimalNumber;
use crate::poly::Rectangle;
use crate::style::Style;
use crate::text::{Text, TextMobjectError, text_style};
use crate::vmobject::{VMobject, v_group};

// ---------------------------------------------------------------------------
// ControlMobject's Stage side: the tracker binding
// ---------------------------------------------------------------------------

/// A detached control whose value is one scalar — the Rust face of the
/// Reference's `ControlMobject(ValueTracker)`.
///
/// `bool` controls encode `false`/`true` as `0.0`/`1.0`.
pub trait ScalarControl: Clone + 'static {
    /// The current value as one f64 lane.
    fn scalar_value(&self) -> f64;
    /// A copy of this control with `value` applied through its ordinary
    /// `set_value` path (clamping, snapping, and `set_value_anim` records
    /// included).
    fn at_value(&self, value: f64) -> Self;
    /// The full visual composition at the current value.
    fn composition(&self) -> VMobject;
}

/// The Stage-side handle of a registered control: the stable container the
/// composition is redrawn into, and the tracker that drives it.
///
/// `add_to_scene(container)` is what puts the control on the clock; from
/// then on `stage.set_tracker_value(tracker, v)` (or any animation of the
/// tracker) rebuilds the composition on the next `stage.update`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlMob {
    /// The always-redrawn composition container (stable across rebuilds).
    pub container: Mob,
    /// The `ValueTracker` holding the control's scalar value.
    pub tracker: Mob,
}

/// Register a scalar control on the Stage: one `ValueTracker` holding the
/// control's current value, plus an [`Stage::always_redraw`] container whose
/// content is rebuilt from the tracker every tick.
///
/// This is the `ControlMobject(ValueTracker)` fusion, split the way the
/// scene runtime consumes it: the value lives in the tracker where
/// animations can drive it, and the composition follows.
#[must_use]
pub fn add_scalar_control<C: ScalarControl>(stage: &mut Stage, control: &C) -> ControlMob {
    let tracker = stage.add_value_tracker(control.scalar_value());
    let proto = control.clone();
    let container = stage.always_redraw(move |stage| {
        let value = stage
            .tracker_value(tracker)
            .unwrap_or_else(|| proto.scalar_value());
        stage.add(proto.at_value(value).composition())
    });
    ControlMob { container, tracker }
}

// ---------------------------------------------------------------------------
// ControlMobject — the base
// ---------------------------------------------------------------------------

/// The Reference's `ControlMobject`: a value plus the mobjects that show it.
///
/// In the Reference this is the abstract base whose `assert_value` /
/// `set_value_anim` hooks the concrete controls override; here those hooks
/// are each concrete control's native `set_value`, and this struct is the
/// plain base — a scalar value with children — useful on its own and as
/// the documentation anchor for the family.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlMobject {
    value: f64,
    children: Vec<VMobject>,
}

impl ControlMobject {
    /// `ControlMobject(value, *mobjects)`.
    #[must_use]
    pub fn new(value: f64, children: impl IntoIterator<Item = VMobject>) -> Self {
        Self {
            value,
            children: children.into_iter().collect(),
        }
    }

    /// `get_value`.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// `set_value`. The base class has no `set_value_anim`; concrete
    /// controls override this with record-rewriting versions.
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    /// The grafted mobjects, in constructor order.
    #[must_use]
    pub fn children(&self) -> &[VMobject] {
        &self.children
    }

    /// The whole control as one group.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        v_group(self.children.clone())
    }
}

impl ScalarControl for ControlMobject {
    fn scalar_value(&self) -> f64 {
        self.value
    }

    fn at_value(&self, value: f64) -> Self {
        let mut copy = self.clone();
        copy.set_value(value);
        copy
    }

    fn composition(&self) -> VMobject {
        self.composition()
    }
}

impl From<ControlMobject> for Mobject {
    fn from(c: ControlMobject) -> Self {
        c.composition().into()
    }
}

// ---------------------------------------------------------------------------
// MotionMobject
// ---------------------------------------------------------------------------

/// `MotionMobject(mobject)` — a mobject that can be held and dragged.
///
/// The drag itself is the omitted `mob_on_mouse_drag` hook (W9EVENTS);
/// what lands here is the composition: a group wrapping the mobject, the
/// unit the drag listener will move.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionMobject {
    mobject: VMobject,
}

impl MotionMobject {
    /// Wrap `mobject`.
    #[must_use]
    pub fn new(mobject: VMobject) -> Self {
        Self { mobject }
    }

    /// The wrapped mobject.
    #[must_use]
    pub fn mobject(&self) -> &VMobject {
        &self.mobject
    }

    /// Build the group.
    #[must_use]
    pub fn build(self) -> VMobject {
        v_group([self.mobject])
    }
}

impl From<MotionMobject> for Mobject {
    fn from(m: MotionMobject) -> Self {
        m.build().into()
    }
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// `Button` — a clickable: a background rect behind a label (or behind any
/// mobject, the Reference's `Button(mobject, on_click)` form).
///
/// The click dispatch is the omitted `on_click` hook (W9EVENTS); the
/// composition — rect plus content — is the deliverable.
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    content: ButtonContent,
    font_size: f64,
    buff: f64,
    rect_style: Style,
    label_style: Style,
}

#[derive(Debug, Clone, PartialEq)]
enum ButtonContent {
    Label(String),
    Mobject(Box<VMobject>),
}

impl Button {
    /// A button labelled `label`.
    #[must_use]
    pub fn new(label: &str) -> Self {
        Self {
            content: ButtonContent::Label(label.to_string()),
            font_size: crate::text::DEFAULT_FONT_SIZE,
            buff: MED_SMALL_BUFF,
            rect_style: Style::default(),
            label_style: text_style(),
        }
    }

    /// The Reference's form: any mobject as the button face.
    #[must_use]
    pub fn of(mobject: VMobject) -> Self {
        Self {
            content: ButtonContent::Mobject(Box::new(mobject)),
            ..Self::new("")
        }
    }

    /// Label font size (label form only).
    #[must_use]
    pub fn font_size(mut self, font_size: f64) -> Self {
        self.font_size = font_size;
        self
    }

    /// Gap between the content and the background rect's edge.
    #[must_use]
    pub fn buff(mut self, buff: f64) -> Self {
        self.buff = buff;
        self
    }

    /// Style of the background rect.
    #[must_use]
    pub fn rect_style(mut self, style: Style) -> Self {
        self.rect_style = style;
        self
    }

    /// Style of the label (label form only).
    #[must_use]
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Build the composition: `[background rect, content]`.
    ///
    /// # Errors
    /// [`TextMobjectError`] if the label fails to typeset.
    pub fn build(&self, book: &FontBook) -> Result<VMobject, TextMobjectError> {
        let content = match &self.content {
            ButtonContent::Label(label) => {
                Text::new(label)
                    .font_size(self.font_size)
                    .style(self.label_style)
                    .build(book)?
                    .vmob
            }
            ButtonContent::Mobject(vmob) => (**vmob).clone(),
        };
        let rect = SurroundingRectangle::new(&content)
            .buff(self.buff)
            .style(self.rect_style)
            .build();
        Ok(v_group([rect, content]))
    }
}

// ---------------------------------------------------------------------------
// EnableDisableButton
// ---------------------------------------------------------------------------

/// `EnableDisableButton(value=True)` — a box that shows GREEN enabled /
/// RED disabled. The bool rides the scalar lane as `1.0`/`0.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct EnableDisableButton {
    value: bool,
    width: f64,
    height: f64,
    fill_opacity: f64,
    enable_color: Srgb,
    disable_color: Srgb,
    /// Whether `set_value` has run at least once — see the module docs for
    /// the Reference's white-at-construction quirk.
    colored: bool,
    rect: VMobject,
}

impl EnableDisableButton {
    /// The Reference's defaults: a 0.5×0.5 box, fill opacity 1.
    #[must_use]
    pub fn new(value: bool) -> Self {
        let mut button = Self {
            value,
            width: 0.5,
            height: 0.5,
            fill_opacity: 1.0,
            enable_color: GREEN,
            disable_color: RED,
            colored: false,
            rect: VMobject::new(),
        };
        button.rebuild_rect();
        button
    }

    /// Box width.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self.rebuild_rect();
        self
    }

    /// Box height.
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self.rebuild_rect();
        self
    }

    /// Box fill opacity (`rect_kwargs["fill_opacity"]`).
    #[must_use]
    pub fn fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self.rebuild_rect();
        self
    }

    /// The enabled fill (`enable_color=GREEN`).
    #[must_use]
    pub fn enable_color(mut self, color: Srgb) -> Self {
        self.enable_color = color;
        self.rebuild_rect();
        self
    }

    /// The disabled fill (`disable_color=RED`).
    #[must_use]
    pub fn disable_color(mut self, color: Srgb) -> Self {
        self.disable_color = color;
        self.rebuild_rect();
        self
    }

    /// `get_value`.
    #[must_use]
    pub fn value(&self) -> bool {
        self.value
    }

    /// `set_value`: store the bool and run `set_value_anim` — fill the box
    /// with the enable/disable colour.
    pub fn set_value(&mut self, value: bool) {
        self.value = value;
        self.colored = true;
        self.rebuild_rect();
    }

    /// `toggle_value`.
    pub fn toggle_value(&mut self) {
        let value = !self.value;
        self.set_value(value);
    }

    /// The box.
    #[must_use]
    pub fn rect(&self) -> &VMobject {
        &self.rect
    }

    /// The whole control as one group.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        v_group([self.rect.clone()])
    }

    fn rebuild_rect(&mut self) {
        // The Reference constructs the box with fill_opacity=1 and the
        // mobject-default colour; the enable/disable fill only appears
        // once set_value_anim has run (ValueTracker.__init__ bypasses it).
        let fill = if self.colored {
            if self.value {
                self.enable_color
            } else {
                self.disable_color
            }
        } else {
            DEFAULT_MOBJECT_COLOR
        };
        self.rect = Rectangle::new()
            .width(self.width)
            .height(self.height)
            .style(Style::default().fill(fill, self.fill_opacity))
            .build()
            .expect("an unrounded control rectangle cannot request arc components");
    }
}

impl ScalarControl for EnableDisableButton {
    fn scalar_value(&self) -> f64 {
        if self.value { 1.0 } else { 0.0 }
    }

    fn at_value(&self, value: f64) -> Self {
        let mut copy = self.clone();
        copy.set_value(value != 0.0);
        copy
    }

    fn composition(&self) -> VMobject {
        self.composition()
    }
}

impl From<EnableDisableButton> for Mobject {
    fn from(b: EnableDisableButton) -> Self {
        b.composition().into()
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// `Checkbox(value=True)` — a box whose content is a checkmark (checked)
/// or a cross (unchecked). The bool rides the scalar lane as `1.0`/`0.0`.
///
/// The marks are the native drawn pair (BN-08), stretched onto the box and
/// scaled by the Reference's hard `0.5`, so the content stands half the
/// box's size, centred on it.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkbox {
    value: bool,
    box_width: f64,
    box_height: f64,
    fill_opacity: f64,
    checkmark_color: Srgb,
    checkmark_stroke_width: f64,
    cross_color: Srgb,
    cross_stroke_width: f64,
    content_scale: f64,
    rect: VMobject,
    content: VMobject,
}

impl Checkbox {
    /// The Reference's defaults: a 0.5×0.5 box, fill opacity 0, GREEN
    /// checkmark, RED cross.
    #[must_use]
    pub fn new(value: bool) -> Self {
        let mut checkbox = Self {
            value,
            box_width: 0.5,
            box_height: 0.5,
            fill_opacity: 0.0,
            checkmark_color: GREEN,
            checkmark_stroke_width: 0.0,
            cross_color: RED,
            cross_stroke_width: 0.0,
            content_scale: 0.5,
            rect: VMobject::new(),
            content: VMobject::new(),
        };
        checkbox.rect = checkbox.build_rect();
        checkbox.content = checkbox.build_content();
        checkbox
    }

    /// Box width.
    #[must_use]
    pub fn box_width(mut self, width: f64) -> Self {
        self.box_width = width;
        self.rect = self.build_rect();
        self.content = self.build_content();
        self
    }

    /// Box height.
    #[must_use]
    pub fn box_height(mut self, height: f64) -> Self {
        self.box_height = height;
        self.rect = self.build_rect();
        self.content = self.build_content();
        self
    }

    /// Box fill opacity (`rect_kwargs["fill_opacity"]`).
    #[must_use]
    pub fn box_fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self.rect = self.build_rect();
        self
    }

    /// Checkmark colour (`checkmark_kwargs` stroke GREEN).
    #[must_use]
    pub fn checkmark_color(mut self, color: Srgb) -> Self {
        self.checkmark_color = color;
        self.content = self.build_content();
        self
    }

    /// Checkmark stroke width (`checkmark_kwargs["stroke_width"]`).
    /// Native construction keeps `0` (BN-08 filled silhouettes); the
    /// portal default `6` is the Reference TeX-dingbat stroke.
    #[must_use]
    pub fn checkmark_stroke_width(mut self, width: f64) -> Self {
        self.checkmark_stroke_width = width;
        self.content = self.build_content();
        self
    }

    /// Cross colour (`cross_kwargs` stroke RED).
    #[must_use]
    pub fn cross_color(mut self, color: Srgb) -> Self {
        self.cross_color = color;
        self.content = self.build_content();
        self
    }

    /// Cross stroke width (`cross_kwargs["stroke_width"]`).
    #[must_use]
    pub fn cross_stroke_width(mut self, width: f64) -> Self {
        self.cross_stroke_width = width;
        self.content = self.build_content();
        self
    }

    /// `get_value`.
    #[must_use]
    pub fn value(&self) -> bool {
        self.value
    }

    /// `set_value`: store the bool and run `set_value_anim` — the content
    /// becomes the checkmark or the cross.
    pub fn set_value(&mut self, value: bool) {
        self.value = value;
        self.content = self.build_content();
    }

    /// `toggle_value`.
    pub fn toggle_value(&mut self) {
        let value = !self.value;
        self.set_value(value);
    }

    /// The box.
    #[must_use]
    pub fn rect(&self) -> &VMobject {
        &self.rect
    }

    /// The box content (checkmark or cross).
    #[must_use]
    pub fn content(&self) -> &VMobject {
        &self.content
    }

    /// The whole control as one group: `[box, content]`.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        v_group([self.rect.clone(), self.content.clone()])
    }

    fn build_rect(&self) -> VMobject {
        Rectangle::new()
            .width(self.box_width)
            .height(self.box_height)
            .style(Style::default().fill_opacity(self.fill_opacity))
            .build()
            .expect("an unrounded checkbox cannot request arc components")
    }

    /// `get_checkmark` / `get_cross`: the drawn mark stretched onto the
    /// box, scaled by 0.5, centred on the box.
    fn build_content(&self) -> VMobject {
        let (color, width) = if self.value {
            (self.checkmark_color, self.checkmark_stroke_width)
        } else {
            (self.cross_color, self.cross_stroke_width)
        };
        let mark = if self.value {
            checkmark(self.checkmark_color)
        } else {
            exmark(self.cross_color)
        };
        mark.with_width(self.box_width, true)
            .with_height(self.box_height, true)
            .scaled_about(self.content_scale, ORIGIN)
            .moved_to(self.rect.center_point())
            .map_style(|style| style.stroke(color, width, 1.0).fill(color, 1.0))
    }
}

impl ScalarControl for Checkbox {
    fn scalar_value(&self) -> f64 {
        if self.value { 1.0 } else { 0.0 }
    }

    fn at_value(&self, value: f64) -> Self {
        let mut copy = self.clone();
        copy.set_value(value != 0.0);
        copy
    }

    fn composition(&self) -> VMobject {
        self.composition()
    }
}

impl From<Checkbox> for Mobject {
    fn from(c: Checkbox) -> Self {
        c.composition().into()
    }
}

// ---------------------------------------------------------------------------
// LinearNumberSlider
// ---------------------------------------------------------------------------

/// A refused slider configuration, input value, projection, or readout update.
#[derive(Debug)]
pub enum SliderError {
    /// The rounded bar or its straight axis refused invalid geometry.
    Geometry(GeomError),
    /// Bounds must be finite, ordered, and have a representable span.
    InvalidRange,
    /// The snap step must be positive, finite, and usable over the range.
    InvalidStep,
    /// Constructor and mutator values must be finite.
    NonFiniteValue,
    /// Projection points must have three finite coordinates.
    NonFinitePoint,
    /// The configured axis cannot support finite projection arithmetic.
    InvalidAxis,
    /// The optional number readout could not be rebuilt.
    Text(TextMobjectError),
    /// A checkerboard needs at least one background colour.
    EmptyGridColors,
    /// Checkerboard square length must be positive and finite.
    InvalidGridSquareLen,
}

impl core::fmt::Display for SliderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Geometry(e) => write!(f, "slider geometry failed: {e}"),
            Self::InvalidRange => {
                write!(
                    f,
                    "slider bounds must be finite, ordered, and representable"
                )
            }
            Self::InvalidStep => {
                write!(
                    f,
                    "slider step must be positive, finite, and usable over the range"
                )
            }
            Self::NonFiniteValue => write!(f, "slider values must be finite"),
            Self::NonFinitePoint => write!(f, "slider projection points must be finite"),
            Self::InvalidAxis => write!(f, "slider axis must support finite projection"),
            Self::Text(e) => write!(f, "slider readout update failed: {e}"),
            Self::EmptyGridColors => write!(f, "checkerboard colors must not be empty"),
            Self::InvalidGridSquareLen => {
                write!(f, "checkerboard square length must be positive and finite")
            }
        }
    }
}

impl std::error::Error for SliderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(e) => Some(e),
            Self::Text(e) => Some(e),
            _ => None,
        }
    }
}

impl From<TextMobjectError> for SliderError {
    fn from(e: TextMobjectError) -> Self {
        Self::Text(e)
    }
}

impl From<GeomError> for SliderError {
    fn from(e: GeomError) -> Self {
        Self::Geometry(e)
    }
}

/// `LinearNumberSlider(value=0, min_value=-10, max_value=10, step=1)` — a
/// rounded bar, a round handle, and an invisible axis the handle rides,
/// plus an optional [`DecimalNumber`] readout above the bar.
#[derive(Clone)]
pub struct LinearNumberSlider {
    value: f64,
    min_value: f64,
    max_value: f64,
    step: f64,
    bar_width: f64,
    bar_height: f64,
    corner_radius: f64,
    handle_radius: f64,
    handle_fill_opacity: f64,
    handle_stroke_color: Srgb,
    handle_fill_color: Srgb,
    bar: VMobject,
    handle: VMobject,
    axis: VMobject,
    axis_ends: (Vec3, Vec3),
    readout: Option<DecimalNumber>,
    readout_book: Option<Rc<FontBook>>,
    num_decimal_places: usize,
}

impl LinearNumberSlider {
    /// The Reference's defaults: bar 2×0.075 (corner radius 0.0375),
    /// handle radius 0.1 in `GREY_A`, value 0 on [-10, 10] stepping by 1.
    ///
    /// The handle parks at the axis midpoint at construction (the
    /// Reference's `slider.move_to(slider_axis)` quirk — see the module
    /// docs); for the default range the midpoint IS value 0's fraction.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError::NonFiniteValue`] when `value` is not finite.
    pub fn new(value: f64) -> Result<Self, SliderError> {
        Self::validate_value(value)?;
        let mut slider = Self {
            value,
            min_value: -10.0,
            max_value: 10.0,
            step: 1.0,
            bar_width: 2.0,
            bar_height: 0.075,
            corner_radius: 0.0375,
            handle_radius: 0.1,
            handle_fill_opacity: 1.0,
            handle_stroke_color: GREY_A,
            handle_fill_color: GREY_A,
            bar: VMobject::new(),
            handle: VMobject::new(),
            axis: VMobject::new(),
            axis_ends: (LEFT, RIGHT),
            readout: None,
            readout_book: None,
            num_decimal_places: 2,
        };
        slider.rebuild_geometry()?;
        Ok(slider)
    }

    /// `min_value=`.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when the resulting range or existing step
    /// cannot support finite slider arithmetic.
    pub fn min_value(mut self, min_value: f64) -> Result<Self, SliderError> {
        Self::validate_configuration_values(min_value, self.max_value, self.step)?;
        self.min_value = min_value;
        Ok(self)
    }

    /// `max_value=`.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when the resulting range or existing step
    /// cannot support finite slider arithmetic.
    pub fn max_value(mut self, max_value: f64) -> Result<Self, SliderError> {
        Self::validate_configuration_values(self.min_value, max_value, self.step)?;
        self.max_value = max_value;
        Ok(self)
    }

    /// Set both range endpoints atomically, avoiding an invalid transient
    /// when the requested interval does not overlap the current one.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when the requested range or existing step
    /// cannot support finite slider arithmetic.
    pub fn range(mut self, min_value: f64, max_value: f64) -> Result<Self, SliderError> {
        Self::validate_configuration_values(min_value, max_value, self.step)?;
        self.min_value = min_value;
        self.max_value = max_value;
        Ok(self)
    }

    /// `step=`.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError::InvalidStep`] unless the step is positive,
    /// finite, and usable over the configured range.
    pub fn step(mut self, step: f64) -> Result<Self, SliderError> {
        Self::validate_configuration_values(self.min_value, self.max_value, step)?;
        self.step = step;
        Ok(self)
    }

    /// Bar width (`rounded_rect_kwargs["width"]`).
    ///
    /// # Errors
    /// [`SliderError::Geometry`] if the rounded bar refuses the dimensions.
    pub fn bar_width(mut self, width: f64) -> Result<Self, SliderError> {
        self.bar_width = width;
        self.rebuild_geometry()?;
        Ok(self)
    }

    /// Bar height (`rounded_rect_kwargs["height"]`).
    ///
    /// # Errors
    /// [`SliderError::Geometry`] if the rounded bar refuses the dimensions.
    pub fn bar_height(mut self, height: f64) -> Result<Self, SliderError> {
        self.bar_height = height;
        self.rebuild_geometry()?;
        Ok(self)
    }

    /// Bar corner radius (`rounded_rect_kwargs["corner_radius"]`).
    ///
    /// # Errors
    /// [`SliderError::Geometry`] if the rounded bar refuses the radius.
    pub fn corner_radius(mut self, radius: f64) -> Result<Self, SliderError> {
        self.corner_radius = radius;
        self.rebuild_geometry()?;
        Ok(self)
    }

    /// Handle radius (`circle_kwargs["radius"]`).
    #[must_use]
    pub fn handle_radius(mut self, radius: f64) -> Self {
        self.handle_radius = radius;
        self.handle = self.build_handle(self.handle_center());
        self
    }

    /// Handle fill opacity (`circle_kwargs["fill_opacity"]`).
    #[must_use]
    pub fn handle_fill_opacity(mut self, opacity: f64) -> Self {
        self.handle_fill_opacity = opacity;
        self.handle = self.build_handle(self.handle_center());
        self
    }

    /// Handle colour (`circle_kwargs` stroke+fill). Used by
    /// [`ColorSliders`] to tint the R/G/B handles; sets both lanes.
    #[must_use]
    pub fn handle_color(mut self, color: Srgb) -> Self {
        self.handle_stroke_color = color;
        self.handle_fill_color = color;
        self.handle = self.build_handle(self.handle_center());
        self
    }

    /// Handle stroke colour (`circle_kwargs["stroke_color"]`).
    #[must_use]
    pub fn handle_stroke_color(mut self, color: Srgb) -> Self {
        self.handle_stroke_color = color;
        self.handle = self.build_handle(self.handle_center());
        self
    }

    /// Handle fill colour (`circle_kwargs["fill_color"]`).
    #[must_use]
    pub fn handle_fill_color(mut self, color: Srgb) -> Self {
        self.handle_fill_color = color;
        self.handle = self.build_handle(self.handle_center());
        self
    }

    /// Add the value readout: a [`DecimalNumber`] above the bar, updated
    /// by every `set_value`. The book is retained (shared) so later
    /// `set_value` calls can re-typeset.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when the configuration is invalid or the
    /// initial number fails to typeset.
    pub fn with_value_readout(
        mut self,
        book: Rc<FontBook>,
        num_decimal_places: usize,
    ) -> Result<Self, SliderError> {
        self.validate_configuration()?;
        Self::validate_value(self.value)?;
        self.num_decimal_places = num_decimal_places;
        let readout = DecimalNumber::new(self.value)
            .num_decimal_places(num_decimal_places)
            .build(&book)?;
        self.readout = Some(readout);
        self.readout_book = Some(book);
        Ok(self)
    }

    /// `get_value`.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// `min_value`.
    #[must_use]
    pub fn min(&self) -> f64 {
        self.min_value
    }

    /// `max_value`.
    #[must_use]
    pub fn max(&self) -> f64 {
        self.max_value
    }

    /// `set_value`: clamp into `[min_value, max_value]` (the Reference's
    /// `assert_value` panics; see the module docs), run `set_value_anim` —
    /// move the handle to the value's fraction of the axis — and refresh
    /// the readout.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] before mutation when the configuration or
    /// value is invalid, or when the optional readout cannot be rebuilt.
    /// Readout, value, and handle commit atomically.
    pub fn set_value(&mut self, value: f64) -> Result<(), SliderError> {
        self.validate_configuration()?;
        Self::validate_value(value)?;
        let clamped = value.clamp(self.min_value, self.max_value);
        let center = self.handle_point(clamped);
        let handle = self.build_handle(center);
        let next_readout = if let (Some(readout), Some(book)) = (&self.readout, &self.readout_book)
        {
            let mut candidate = readout.clone();
            candidate.set_value(clamped, book)?;
            Some(candidate)
        } else {
            None
        };

        self.value = clamped;
        self.handle = handle;
        if let Some(readout) = next_readout {
            self.readout = Some(readout);
        }
        Ok(())
    }

    /// `get_value_from_point`: project onto the axis, turn the fraction
    /// into a value, and snap DOWN to the nearest step (the Reference's
    /// `int()` truncation). This is the pure half of the omitted drag
    /// handler; the drag itself is W9EVENTS.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when the slider configuration, point, or
    /// derived projection arithmetic is not finite.
    pub fn value_from_point(&self, point: Vec3) -> Result<f64, SliderError> {
        self.validate_configuration()?;
        if !point.into_iter().all(f64::is_finite) {
            return Err(SliderError::NonFinitePoint);
        }
        self.validate_axis()?;

        let (start, end) = self.axis_ends;
        let d = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
        let len2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if !len2.is_finite() {
            return Err(SliderError::InvalidAxis);
        }
        let prop = if len2 == 0.0 {
            0.0
        } else {
            let w = [
                point[0] - start[0],
                point[1] - start[1],
                point[2] - start[2],
            ];
            let projected = (w[0] * d[0] + w[1] * d[1] + w[2] * d[2]) / len2;
            if projected.is_nan() {
                return Err(SliderError::InvalidAxis);
            }
            projected.clamp(0.0, 1.0)
        };
        let value = self.min_value + prop * (self.max_value - self.min_value);
        if !value.is_finite() {
            return Err(SliderError::InvalidRange);
        }
        self.snap_value(value)
    }

    fn snap_value(&self, value: f64) -> Result<f64, SliderError> {
        self.validate_configuration()?;
        Self::validate_value(value)?;
        let clamped = value.clamp(self.min_value, self.max_value);
        let steps = ((clamped - self.min_value) / self.step).trunc();
        if !steps.is_finite() {
            return Err(SliderError::InvalidStep);
        }
        let snapped = self.min_value + steps * self.step;
        if !snapped.is_finite() {
            return Err(SliderError::InvalidStep);
        }
        Ok(snapped.clamp(self.min_value, self.max_value))
    }

    /// The bar.
    #[must_use]
    pub fn bar(&self) -> &VMobject {
        &self.bar
    }

    /// The handle.
    #[must_use]
    pub fn handle(&self) -> &VMobject {
        &self.handle
    }

    /// The invisible axis the handle rides.
    #[must_use]
    pub fn axis(&self) -> &VMobject {
        &self.axis
    }

    /// The readout, if enabled.
    #[must_use]
    pub fn readout(&self) -> Option<&DecimalNumber> {
        self.readout.as_ref()
    }

    /// The whole control as one group: `[bar, handle, axis]`, readout
    /// trailing when enabled.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        let mut children = vec![self.bar.clone(), self.handle.clone(), self.axis.clone()];
        if let Some(readout) = &self.readout {
            children.push(
                readout
                    .vmob()
                    .clone()
                    .next_to(&self.bar, UP, SMALL_BUFF, ORIGIN),
            );
        }
        v_group(children)
    }

    fn validate_value(value: f64) -> Result<(), SliderError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(SliderError::NonFiniteValue)
        }
    }

    fn validate_configuration_values(
        min_value: f64,
        max_value: f64,
        step: f64,
    ) -> Result<(), SliderError> {
        if !min_value.is_finite() || !max_value.is_finite() || min_value > max_value {
            return Err(SliderError::InvalidRange);
        }
        let span = max_value - min_value;
        if !span.is_finite() {
            return Err(SliderError::InvalidRange);
        }
        if !step.is_finite() || step <= 0.0 || (span != 0.0 && !(span / step).is_finite()) {
            return Err(SliderError::InvalidStep);
        }
        Ok(())
    }

    fn validate_configuration(&self) -> Result<(), SliderError> {
        Self::validate_configuration_values(self.min_value, self.max_value, self.step)
    }

    fn validate_axis(&self) -> Result<(), SliderError> {
        let (start, end) = self.axis_ends;
        if !start.into_iter().chain(end).all(f64::is_finite)
            || !end
                .into_iter()
                .zip(start)
                .all(|(end, start)| (end - start).is_finite())
        {
            return Err(SliderError::InvalidAxis);
        }
        Ok(())
    }

    /// The value's fraction of the axis, guarded against a degenerate range.
    fn proportion(&self, value: f64) -> f64 {
        let span = self.max_value - self.min_value;
        if span == 0.0 {
            0.0
        } else {
            (value - self.min_value) / span
        }
    }

    /// `slider_axis.point_from_proportion(prop)` for the straight axis.
    fn handle_point(&self, value: f64) -> Vec3 {
        let prop = self.proportion(value);
        let (start, end) = self.axis_ends;
        [
            start[0] + prop * (end[0] - start[0]),
            start[1] + prop * (end[1] - start[1]),
            start[2] + prop * (end[2] - start[2]),
        ]
    }

    /// Where the handle sits right now.
    fn handle_center(&self) -> Vec3 {
        self.handle.center_point()
    }

    fn build_handle(&self, center: Vec3) -> VMobject {
        Circle::new()
            .radius(self.handle_radius)
            .style(
                Style::default()
                    .stroke(self.handle_stroke_color, Style::default().stroke_width, 1.0)
                    .fill(self.handle_fill_color, self.handle_fill_opacity),
            )
            .build()
            .moved_to(center)
    }

    fn rebuild_geometry(&mut self) -> Result<(), SliderError> {
        self.bar = Rectangle::new()
            .width(self.bar_width)
            .height(self.bar_height)
            .corner_radius(self.corner_radius)
            .build()?;
        let left = self.bar.bbox_point(LEFT).unwrap_or(LEFT);
        let right = self.bar.bbox_point(RIGHT).unwrap_or(RIGHT);
        self.axis_ends = (left, right);
        self.axis = Line::new(left, right)
            .style(Style::default().stroke(WHITE, 0.0, 0.0).fill(WHITE, 0.0))
            .build()?;
        // The Reference parks the handle at the axis midpoint at
        // construction (move_to(slider_axis)); set_value moves it to the
        // value's fraction.
        let midpoint = [
            0.5 * (left[0] + right[0]),
            0.5 * (left[1] + right[1]),
            0.5 * (left[2] + right[2]),
        ];
        self.handle = self.build_handle(midpoint);
        Ok(())
    }
}

impl ScalarControl for LinearNumberSlider {
    fn scalar_value(&self) -> f64 {
        self.value
    }

    fn at_value(&self, value: f64) -> Self {
        let mut copy = self.clone();
        // The Stage tracker has no error channel. The atomic mutator keeps
        // this last valid snapshot intact if a hostile tracker value or a
        // readout failure reaches the redraw closure.
        let _ = copy.set_value(value);
        copy
    }

    fn composition(&self) -> VMobject {
        self.composition()
    }
}

impl From<LinearNumberSlider> for Mobject {
    fn from(s: LinearNumberSlider) -> Self {
        s.composition().into()
    }
}

// ---------------------------------------------------------------------------
// ColorSliders
// ---------------------------------------------------------------------------

/// `ColorSliders` — four [`LinearNumberSlider`]s (R, G, B in `[0, 255]`
/// stepping 1; A in `[0, 1]` stepping 0.04) under a swatch box showing the
/// picked colour over a checkerboard. A `Group` in the Reference, not a
/// control; each slider is independently a [`ScalarControl`].
#[derive(Clone)]
pub struct ColorSliders {
    sliders: [LinearNumberSlider; 4],
    snap_on_set: [bool; 4],
    rect_width: f64,
    rect_height: f64,
    grid_colors: Vec<Srgb>,
    single_square_len: f64,
    sliders_buff: f64,
    color_box: VMobject,
}

impl ColorSliders {
    /// The Reference's defaults: R/G/B at 255, A at 1.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] if the locked component-slider
    /// configuration stops satisfying the slider contract.
    pub fn new() -> Result<Self, SliderError> {
        let rgb = |color: Srgb| -> Result<LinearNumberSlider, SliderError> {
            let slider = LinearNumberSlider::new(255.0)?
                .min_value(0.0)?
                .max_value(255.0)?
                .step(1.0)?;
            Ok(slider.handle_color(color))
        };
        let alpha = LinearNumberSlider::new(1.0)?
            .min_value(0.0)?
            .max_value(1.0)?
            .step(0.04)?
            // set_color_by_gradient(BLACK, WHITE) — approximated by the
            // gradient's midpoint (module docs).
            .handle_color(GREY_C);
        let mut group = Self {
            sliders: [
                rgb(RED)?,
                rgb(GREEN)?,
                rgb(fmn_core::constants::BLUE)?,
                alpha,
            ],
            snap_on_set: [false; 4],
            rect_width: 2.0,
            rect_height: 0.5,
            grid_colors: vec![GREY_A, GREY_C],
            single_square_len: 0.1,
            sliders_buff: fmn_core::constants::MED_LARGE_BUFF,
            color_box: VMobject::new(),
        };
        group.color_box = group.build_color_box()?;
        Ok(group)
    }

    /// Swatch rectangle size (`rect_kwargs["width"]` / `["height"]`).
    ///
    /// # Errors
    /// [`SliderError::Geometry`] if the rectangle refuses either dimension.
    pub fn rect_size(mut self, width: f64, height: f64) -> Result<Self, SliderError> {
        // A color-swatch rectangle must stay visible; degenerate or garbage
        // extents would silently hide the slider's own state mark.
        for extent in [width, height] {
            if !extent.is_finite() || extent <= 0.0 {
                return Err(SliderError::Geometry(GeomError::InvalidExtent));
            }
        }
        self.rect_width = width;
        self.rect_height = height;
        self.color_box = self.build_color_box()?;
        Ok(self)
    }

    /// Checkerboard colours and nominal square length
    /// (`background_grid_kwargs["colors"]` / `["single_square_len"]`).
    ///
    /// # Errors
    ///
    /// Returns [`SliderError::EmptyGridColors`] for an empty palette or
    /// [`SliderError::InvalidGridSquareLen`] unless `square_len` is positive
    /// and finite.
    pub fn background_grid(
        mut self,
        colors: Vec<Srgb>,
        square_len: f64,
    ) -> Result<Self, SliderError> {
        if colors.is_empty() {
            return Err(SliderError::EmptyGridColors);
        }
        if !square_len.is_finite() || square_len <= 0.0 {
            return Err(SliderError::InvalidGridSquareLen);
        }
        self.grid_colors = colors;
        self.single_square_len = square_len;
        Ok(self)
    }

    /// The slider spacing (`sliders_buff=MED_LARGE_BUFF`).
    #[must_use]
    pub fn sliders_buff(mut self, buff: f64) -> Self {
        self.sliders_buff = buff;
        self
    }

    /// Apply the Reference's shared `sliders_kwargs` after the RGB/A range
    /// defaults have been established.
    ///
    /// A shared step larger than the unchanged alpha `[0, 1]` span remains
    /// an RGB-only override, preserving alpha's usable `0.04` default. If
    /// the caller also expands the shared range enough to accommodate the
    /// step, alpha receives the override as well.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] when a requested range, step, or geometry
    /// override violates the underlying [`LinearNumberSlider`] contract.
    #[allow(clippy::too_many_arguments)]
    pub fn slider_overrides(
        mut self,
        min_value: Option<f64>,
        max_value: Option<f64>,
        step: Option<f64>,
        bar_width: Option<f64>,
        bar_height: Option<f64>,
        corner_radius: Option<f64>,
        handle_radius: Option<f64>,
        handle_fill_opacity: Option<f64>,
        handle_stroke_color: Option<Srgb>,
        handle_fill_color: Option<Srgb>,
    ) -> Result<Self, SliderError> {
        for (index, current) in self.sliders.iter_mut().enumerate() {
            let mut slider = current.clone();
            if let (Some(minimum), Some(maximum)) = (min_value, max_value) {
                slider = slider.range(minimum, maximum)?;
            } else {
                if let Some(minimum) = min_value {
                    slider = slider.min_value(minimum)?;
                }
                if let Some(maximum) = max_value {
                    slider = slider.max_value(maximum)?;
                }
            }
            if let Some(requested_step) = step {
                let alpha_range_is_too_small =
                    index == 3 && requested_step > slider.max() - slider.min();
                if !alpha_range_is_too_small {
                    slider = slider.step(requested_step)?;
                    self.snap_on_set[index] = true;
                }
            }
            if let Some(width) = bar_width {
                slider = slider.bar_width(width)?;
            }
            if let Some(height) = bar_height {
                slider = slider.bar_height(height)?;
            }
            if let Some(radius) = corner_radius {
                slider = slider.corner_radius(radius)?;
            }
            if let Some(radius) = handle_radius {
                slider = slider.handle_radius(radius);
            }
            if let Some(opacity) = handle_fill_opacity {
                slider = slider.handle_fill_opacity(opacity);
            }
            if let Some(color) = handle_stroke_color {
                slider = slider.handle_stroke_color(color);
            }
            if let Some(color) = handle_fill_color {
                slider = slider.handle_fill_color(color);
            }
            *current = slider;
        }
        Ok(self)
    }

    /// `set_value(r, g, b, a)`: set all four sliders and refill the swatch.
    ///
    /// # Errors
    ///
    /// Returns [`SliderError`] if any component value or readout update is
    /// refused. All four sliders and the swatch commit atomically.
    pub fn set_value(&mut self, r: f64, g: f64, b: f64, a: f64) -> Result<(), SliderError> {
        let mut candidate = self.clone();
        for (index, value) in [r, g, b, a].into_iter().enumerate() {
            let value = if candidate.snap_on_set[index] {
                candidate.sliders[index].snap_value(value)?
            } else {
                value
            };
            candidate.sliders[index].set_value(value)?;
        }
        candidate.color_box = candidate.build_color_box()?;
        *self = candidate;
        Ok(())
    }

    /// `get_value`: `[r/255, g/255, b/255, a]`, each in `[0, 1]`.
    #[must_use]
    pub fn value(&self) -> [f64; 4] {
        [
            self.sliders[0].value() / 255.0,
            self.sliders[1].value() / 255.0,
            self.sliders[2].value() / 255.0,
            self.sliders[3].value(),
        ]
    }

    /// `get_picked_color`.
    #[must_use]
    pub fn picked_color(&self) -> Srgb {
        let [r, g, b, _] = self.value();
        Srgb { r, g, b }
    }

    /// `get_picked_opacity`.
    #[must_use]
    pub fn picked_opacity(&self) -> f64 {
        self.value()[3]
    }

    /// The four sliders, R, G, B, A.
    #[must_use]
    pub fn sliders(&self) -> &[LinearNumberSlider; 4] {
        &self.sliders
    }

    /// The swatch box (fill follows the picked colour).
    #[must_use]
    pub fn color_box(&self) -> &VMobject {
        &self.color_box
    }

    /// `get_background`: the checkerboard behind the swatch.
    ///
    /// Rows/columns come from integer-dividing the rect by the square
    /// length, with an even column count bumped odd (the Reference's own
    /// rule, so the alternation mirrors left-to-right); the grid is then
    /// stretched onto the rect exactly.
    #[must_use]
    pub fn background(&self) -> VMobject {
        let len = self.single_square_len;
        let rows = ((self.rect_height / len).trunc() as usize).max(1);
        let mut cols = ((self.rect_width / len).trunc() as usize).max(1);
        if cols.is_multiple_of(2) {
            cols += 1;
        }
        let mut squares = Vec::with_capacity(rows * cols);
        for row in 0..rows {
            for col in 0..cols {
                let idx = row * cols + col;
                let color = self.grid_colors[idx % self.grid_colors.len()];
                let x = (col as f64 - (cols as f64 - 1.0) / 2.0) * len;
                let y = ((rows as f64 - 1.0) / 2.0 - row as f64) * len;
                squares.push(
                    Rectangle::square(len)
                        .style(Style::default().stroke(color, 0.0, 0.0).fill(color, 1.0))
                        .build()
                        .expect("an unrounded checker square cannot request arc components")
                        .shifted([x, y, 0.0]),
                );
            }
        }
        v_group(squares)
            .with_width(self.rect_width, true)
            .with_height(self.rect_height, true)
            .moved_to(self.color_box.center_point())
    }

    /// The whole group: `[v_group(checkerboard, swatch), sliders]`,
    /// arranged DOWN and centred (the Reference's closing `arrange(DOWN)`).
    #[must_use]
    pub fn composition(&self) -> VMobject {
        let swatch = v_group([self.background(), self.color_box.clone()]);
        let sliders = VMobject::arranged(
            self.sliders.iter().map(LinearNumberSlider::composition),
            DOWN,
            self.sliders_buff,
            ORIGIN,
        )
        .moved_to(ORIGIN);
        VMobject::arranged(
            [swatch, sliders],
            DOWN,
            DEFAULT_MOBJECT_TO_MOBJECT_BUFF,
            ORIGIN,
        )
        .moved_to(ORIGIN)
    }

    fn build_color_box(&self) -> Result<VMobject, SliderError> {
        Ok(Rectangle::new()
            .width(self.rect_width)
            .height(self.rect_height)
            .style(
                Style::default()
                    .stroke(WHITE, Style::default().stroke_width, 1.0)
                    .fill(self.picked_color(), self.picked_opacity()),
            )
            .build()?)
    }
}

impl From<ColorSliders> for Mobject {
    fn from(c: ColorSliders) -> Self {
        c.composition().into()
    }
}

// ---------------------------------------------------------------------------
// Textbox
// ---------------------------------------------------------------------------

/// `Textbox(value="")` — a box with editable text. The value is a string,
/// so it cannot ride a scalar tracker (the Reference abuses
/// `dtype=object`); the mutators are the whole surface.
///
/// Construction needs the font book eagerly because `set_value`
/// re-typesets; the book is retained (shared).
#[derive(Clone)]
pub struct Textbox {
    value: String,
    box_width: f64,
    box_height: f64,
    box_fill: Srgb,
    box_fill_opacity: f64,
    text_color: Srgb,
    text_buff: f64,
    font_size: f64,
    active_color: Srgb,
    deactive_color: Srgb,
    active: bool,
    book: Rc<FontBook>,
    rect: VMobject,
    text: VMobject,
}

impl Textbox {
    /// The Reference's defaults: box 2×1 filled with the mobject-default
    /// colour, BLUE text, `MED_SMALL_BUFF` padding, inactive (RED frame).
    ///
    /// # Errors
    /// [`TextMobjectError`] if the initial text fails to typeset.
    pub fn new(book: Rc<FontBook>, value: &str) -> Result<Self, TextMobjectError> {
        let mut textbox = Self {
            value: value.to_string(),
            box_width: 2.0,
            box_height: 1.0,
            box_fill: DEFAULT_MOBJECT_COLOR,
            box_fill_opacity: 1.0,
            text_color: fmn_core::constants::BLUE,
            text_buff: MED_SMALL_BUFF,
            font_size: crate::text::DEFAULT_FONT_SIZE,
            active_color: fmn_core::constants::BLUE,
            deactive_color: RED,
            active: false,
            book,
            rect: VMobject::new(),
            text: VMobject::new(),
        };
        textbox.rect = textbox.build_rect();
        textbox.update_text()?;
        Ok(textbox)
    }

    /// Start active (`isInitiallyActive`).
    #[must_use]
    pub fn initially_active(mut self, active: bool) -> Self {
        self.active = active;
        self.rect = self.build_rect();
        self
    }

    /// Box size (`box_kwargs["width"]` / `["height"]`).
    ///
    /// # Errors
    /// [`TextMobjectError`] if the fitted text fails to re-typeset.
    pub fn box_size(mut self, width: f64, height: f64) -> Result<Self, TextMobjectError> {
        self.box_width = width;
        self.box_height = height;
        self.rect = self.build_rect();
        self.update_text()?;
        Ok(self)
    }

    /// Box fill (`box_kwargs["fill_color"]` / `["fill_opacity"]`).
    #[must_use]
    pub fn box_fill(mut self, color: Srgb, opacity: f64) -> Self {
        self.box_fill = color;
        self.box_fill_opacity = opacity;
        self.rect = self.build_rect();
        self
    }

    /// Text colour (`text_kwargs["color"]`).
    ///
    /// # Errors
    /// [`TextMobjectError`] if the coloured text fails to re-typeset.
    pub fn text_color(mut self, color: Srgb) -> Result<Self, TextMobjectError> {
        self.text_color = color;
        self.update_text()?;
        Ok(self)
    }

    /// Padding between text and box edge (`text_buff`).
    ///
    /// # Errors
    /// [`TextMobjectError`] if the fitted text fails to re-typeset.
    pub fn text_buff(mut self, buff: f64) -> Result<Self, TextMobjectError> {
        self.text_buff = buff;
        self.update_text()?;
        Ok(self)
    }

    /// Active frame colour (`active_color`).
    #[must_use]
    pub fn active_color(mut self, color: Srgb) -> Self {
        self.active_color = color;
        self.rect = self.build_rect();
        self
    }

    /// Inactive frame colour (`deactive_color`).
    #[must_use]
    pub fn deactive_color(mut self, color: Srgb) -> Self {
        self.deactive_color = color;
        self.rect = self.build_rect();
        self
    }

    /// `get_value`.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// `set_value`: store the string and run `set_value_anim` —
    /// re-typeset the text into the box.
    ///
    /// # Errors
    /// [`TextMobjectError`] if the new text fails to typeset.
    pub fn set_value(&mut self, value: &str) -> Result<(), TextMobjectError> {
        self.value = value.to_string();
        self.update_text()
    }

    /// `isActive`.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// `active_anim`: frame the box in the active/deactive colour.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        self.rect = self.build_rect();
    }

    /// The box's click-toggle, as a plain mutator (the click is W9EVENTS).
    pub fn toggle_active(&mut self) {
        self.set_active(!self.active);
    }

    /// The box.
    #[must_use]
    pub fn rect(&self) -> &VMobject {
        &self.rect
    }

    /// The current text family.
    #[must_use]
    pub fn text(&self) -> &VMobject {
        &self.text
    }

    /// The whole control as one group: `[box, text]`.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        v_group([self.rect.clone(), self.text.clone()])
    }

    fn build_rect(&self) -> VMobject {
        let frame = if self.active {
            self.active_color
        } else {
            self.deactive_color
        };
        Rectangle::new()
            .width(self.box_width)
            .height(self.box_height)
            .style(
                Style::default()
                    .fill(self.box_fill, self.box_fill_opacity)
                    .stroke(frame, Style::default().stroke_width, 1.0),
            )
            .build()
            .expect("an unrounded text box cannot request arc components")
    }

    /// `update_text`: re-typeset into the box, fit the width to
    /// `box_width - 2·text_buff`, and cap the height at what the text
    /// stood before the fit (the Reference's exact sequence), centred on
    /// the box.
    fn update_text(&mut self) -> Result<(), TextMobjectError> {
        let mut text = Text::new(&self.value)
            .font_size(self.font_size)
            .style(text_style().color(self.text_color))
            .build(&self.book)?
            .vmob;
        let height = text.length_over_dim(1);
        text = text.with_width(self.box_width - 2.0 * self.text_buff, false);
        if text.length_over_dim(1) > height {
            text = text.with_height(height, false);
        }
        self.text = text.moved_to(self.rect.center_point());
        Ok(())
    }
}

impl From<Textbox> for Mobject {
    fn from(t: Textbox) -> Self {
        t.composition().into()
    }
}

// ---------------------------------------------------------------------------
// ControlPanel
// ---------------------------------------------------------------------------

/// `ControlPanel(*controls)` — a GREY_C panel with an opener tab and a
/// column of controls. Build with the font book (the opener's info text);
/// the built panel keeps no book: its mutators only move geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPanel {
    controls: Vec<VMobject>,
    opener_text: String,
    opener_font_size: f64,
    opener_text_color: Srgb,
    panel_width: f64,
    panel_height: f64,
    panel_fill: Srgb,
    panel_fill_opacity: f64,
    panel_stroke: Srgb,
    panel_stroke_width: f64,
    panel_stroke_opacity: f64,
    opener_width: f64,
    opener_height: f64,
    opener_fill: Srgb,
    opener_fill_opacity: f64,
    opener_stroke: Srgb,
    opener_stroke_width: f64,
    opener_stroke_opacity: f64,
}

impl ControlPanel {
    /// The Reference's defaults: panel `FRAME_WIDTH/4` wide and
    /// `MED_SMALL_BUFF + FRAME_HEIGHT` tall, opener `FRAME_WIDTH/8 × 0.5`,
    /// info text "Control Panel" at font size 20.
    #[must_use]
    pub fn new(controls: impl IntoIterator<Item = VMobject>) -> Self {
        Self {
            controls: controls.into_iter().collect(),
            opener_text: "Control Panel".to_string(),
            opener_font_size: 20.0,
            opener_text_color: DEFAULT_MOBJECT_COLOR,
            panel_width: FRAME_WIDTH / 4.0,
            panel_height: MED_SMALL_BUFF + FRAME_HEIGHT,
            panel_fill: GREY_C,
            panel_fill_opacity: 1.0,
            panel_stroke: DEFAULT_VMOBJECT_STROKE_COLOR,
            panel_stroke_width: 0.0,
            panel_stroke_opacity: 1.0,
            opener_width: FRAME_WIDTH / 8.0,
            opener_height: 0.5,
            opener_fill: GREY_C,
            opener_fill_opacity: 1.0,
            opener_stroke: DEFAULT_VMOBJECT_STROKE_COLOR,
            opener_stroke_width: DEFAULT_STROKE_WIDTH,
            opener_stroke_opacity: 1.0,
        }
    }

    /// The opener's label (`opener_text_kwargs["text"]`).
    #[must_use]
    pub fn opener_text(mut self, text: &str) -> Self {
        self.opener_text = text.to_string();
        self
    }

    /// The opener label's font size (`opener_text_kwargs["font_size"]`).
    #[must_use]
    pub fn opener_font_size(mut self, font_size: f64) -> Self {
        self.opener_font_size = font_size;
        self
    }

    /// The opener label's colour (`opener_text_kwargs["color"]` /
    /// `["fill_color"]`).
    #[must_use]
    pub fn opener_text_color(mut self, color: Srgb) -> Self {
        self.opener_text_color = color;
        self
    }

    /// Panel width (`panel_kwargs["width"]`).
    #[must_use]
    pub fn panel_width(mut self, width: f64) -> Self {
        self.panel_width = width;
        self
    }

    /// Panel height (`panel_kwargs["height"]`).
    #[must_use]
    pub fn panel_height(mut self, height: f64) -> Self {
        self.panel_height = height;
        self
    }

    /// Panel fill (`panel_kwargs["fill_color"]` / `["fill_opacity"]`).
    #[must_use]
    pub fn panel_fill(mut self, color: Srgb, opacity: f64) -> Self {
        self.panel_fill = color;
        self.panel_fill_opacity = opacity;
        self
    }

    /// Panel stroke (`panel_kwargs["stroke_color"]` / `["stroke_width"]` /
    /// `["stroke_opacity"]`).
    #[must_use]
    pub fn panel_stroke(mut self, color: Srgb, width: f64, opacity: f64) -> Self {
        self.panel_stroke = color;
        self.panel_stroke_width = width;
        self.panel_stroke_opacity = opacity;
        self
    }

    /// Panel stroke width (`panel_kwargs["stroke_width"]`).
    #[must_use]
    pub fn panel_stroke_width(mut self, width: f64) -> Self {
        self.panel_stroke_width = width;
        self
    }

    /// Opener tab width (`opener_kwargs["width"]`).
    #[must_use]
    pub fn opener_width(mut self, width: f64) -> Self {
        self.opener_width = width;
        self
    }

    /// Opener tab height (`opener_kwargs["height"]`).
    #[must_use]
    pub fn opener_height(mut self, height: f64) -> Self {
        self.opener_height = height;
        self
    }

    /// Opener fill (`opener_kwargs["fill_color"]` / `["fill_opacity"]`).
    #[must_use]
    pub fn opener_fill(mut self, color: Srgb, opacity: f64) -> Self {
        self.opener_fill = color;
        self.opener_fill_opacity = opacity;
        self
    }

    /// Opener stroke (`opener_kwargs["stroke_color"]` / `["stroke_width"]` /
    /// `["stroke_opacity"]`).
    #[must_use]
    pub fn opener_stroke(mut self, color: Srgb, width: f64, opacity: f64) -> Self {
        self.opener_stroke = color;
        self.opener_stroke_width = width;
        self.opener_stroke_opacity = opacity;
        self
    }

    /// Build the panel.
    ///
    /// # Errors
    /// [`TextMobjectError`] if the opener text fails to typeset.
    pub fn build(&self, book: &FontBook) -> Result<ControlPanelMobject, TextMobjectError> {
        let panel_height = self.panel_height;
        let panel = Rectangle::new()
            .width(self.panel_width)
            .height(panel_height)
            .style(
                Style::default()
                    .fill(self.panel_fill, self.panel_fill_opacity)
                    .stroke(
                        self.panel_stroke,
                        self.panel_stroke_width,
                        self.panel_stroke_opacity,
                    ),
            )
            .build()
            .expect("an unrounded control panel cannot request arc components");
        // to_corner(UP + LEFT, buff=0), then shift up by the panel height.
        let panel = to_corner(panel, [LEFT[0], UP[1], 0.0], 0.0).shifted([0.0, panel_height, 0.0]);

        let opener_rect = Rectangle::new()
            .width(self.opener_width)
            .height(self.opener_height)
            .style(
                Style::default()
                    .fill(self.opener_fill, self.opener_fill_opacity)
                    .stroke(
                        self.opener_stroke,
                        self.opener_stroke_width,
                        self.opener_stroke_opacity,
                    ),
            )
            .build()
            .expect("an unrounded opener cannot request arc components");
        let info_text = Text::new(&self.opener_text)
            .font_size(self.opener_font_size)
            .style(text_style().color(self.opener_text_color))
            .build(book)?
            .vmob
            .moved_to(opener_rect.center_point());
        // panel_opener.next_to(panel, DOWN, aligned_edge=DOWN), as a group.
        let opener = v_group([opener_rect, info_text]).next_to(
            &panel,
            DOWN,
            DEFAULT_MOBJECT_TO_MOBJECT_BUFF,
            DOWN,
        );
        let mut opener_children = opener.children().to_vec();
        let info_text = opener_children.remove(1);
        let opener_rect = opener_children.remove(0);

        // controls.arrange(DOWN, center=False, aligned_edge=ORIGIN),
        // controls.move_to(panel).
        let controls = VMobject::arranged(
            self.controls.clone(),
            DOWN,
            DEFAULT_MOBJECT_TO_MOBJECT_BUFF,
            ORIGIN,
        )
        .moved_to(panel.center_point());

        let mut built = ControlPanelMobject {
            panel,
            opener_rect,
            info_text,
            controls,
        };
        built.move_panel_and_controls_to_panel_opener();
        Ok(built)
    }
}

/// The built panel: panel, opener (rect + info text), and the controls
/// column, with the open/close/add/remove mutators.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlPanelMobject {
    panel: VMobject,
    opener_rect: VMobject,
    info_text: VMobject,
    controls: VMobject,
}

impl ControlPanelMobject {
    /// The panel rect.
    #[must_use]
    pub fn panel(&self) -> &VMobject {
        &self.panel
    }

    /// The opener: `[opener rect, info text]`.
    #[must_use]
    pub fn opener(&self) -> VMobject {
        v_group([self.opener_rect.clone(), self.info_text.clone()])
    }

    /// The opener rect.
    #[must_use]
    pub fn opener_rect(&self) -> &VMobject {
        &self.opener_rect
    }

    /// The opener's info text.
    #[must_use]
    pub fn info_text(&self) -> &VMobject {
        &self.info_text
    }

    /// The controls column.
    #[must_use]
    pub fn controls(&self) -> &VMobject {
        &self.controls
    }

    /// Reference family order: `[panel, opener, controls]`.
    #[must_use]
    pub fn family_parts(&self) -> (&VMobject, VMobject, &VMobject) {
        (&self.panel, self.opener(), &self.controls)
    }

    /// The Reference's `move_panel_and_controls_to_panel_opener` as a
    /// pure layout: panel sits on the opener; controls keep their x and
    /// sit `MED_SMALL_BUFF` above it. The portal bind uses this after
    /// hanging live Python control children onto a native panel.
    #[must_use]
    pub fn layout_against_opener(
        panel: VMobject,
        opener_rect: &VMobject,
        controls: VMobject,
    ) -> (VMobject, VMobject) {
        let panel = panel.next_to(opener_rect, UP, 0.0, ORIGIN);
        let controls_x = controls.center_point()[0];
        let controls = controls.next_to(opener_rect, UP, MED_SMALL_BUFF, ORIGIN);
        let new_x = controls.center_point()[0];
        let controls = controls.shifted([controls_x - new_x, 0.0, 0.0]);
        (panel, controls)
    }

    /// `panel_opener.to_corner(corner, buff=0)` keeping the opener's x.
    #[must_use]
    pub fn opener_to_corner(opener: VMobject, corner: Vec3) -> VMobject {
        let x = opener.center_point()[0];
        let opener = to_corner(opener, corner, 0.0);
        let new_x = opener.center_point()[0];
        opener.shifted([x - new_x, 0.0, 0.0])
    }

    /// The whole group: `[panel, opener, controls]`.
    #[must_use]
    pub fn composition(&self) -> VMobject {
        v_group([self.panel.clone(), self.opener(), self.controls.clone()])
    }

    /// `add_controls(*new_controls)`.
    pub fn add_controls(&mut self, new_controls: impl IntoIterator<Item = VMobject>) {
        self.controls = self.controls.clone().with_children(new_controls);
        self.move_panel_and_controls_to_panel_opener();
    }

    /// `remove_controls(*controls_to_remove)`, by child index.
    pub fn remove_controls(&mut self, indices: &[usize]) {
        let kept: Vec<VMobject> = self
            .controls
            .children()
            .iter()
            .enumerate()
            .filter(|(i, _)| !indices.contains(i))
            .map(|(_, c)| c.clone())
            .collect();
        self.controls = VMobject::new().with_children(kept);
        self.move_panel_and_controls_to_panel_opener();
    }

    /// `open_panel`: opener tab to the bottom-left frame corner (keeping
    /// its x), panel and controls riding above it.
    pub fn open_panel(&mut self) -> &mut Self {
        self.move_opener_to_corner([DOWN[0] + LEFT[0], DOWN[1] + LEFT[1], 0.0]);
        self.move_panel_and_controls_to_panel_opener();
        self
    }

    /// `close_panel`: opener tab to the top-left frame corner (keeping
    /// its x), panel and controls riding above it.
    pub fn close_panel(&mut self) -> &mut Self {
        self.move_opener_to_corner([UP[0] + LEFT[0], UP[1] + LEFT[1], 0.0]);
        self.move_panel_and_controls_to_panel_opener();
        self
    }

    /// The Reference's `move_panel_and_controls_to_panel_opener`: the
    /// panel sits directly on the opener; the controls keep their x and
    /// sit `MED_SMALL_BUFF` above the opener.
    fn move_panel_and_controls_to_panel_opener(&mut self) {
        let (panel, controls) = Self::layout_against_opener(
            self.panel.clone(),
            &self.opener_rect,
            self.controls.clone(),
        );
        self.panel = panel;
        self.controls = controls;
    }

    /// `panel_opener.to_corner(corner, buff=0)` keeping the opener's x,
    /// moving rect and info text as one group.
    fn move_opener_to_corner(&mut self, corner: Vec3) {
        let opener = Self::opener_to_corner(self.opener(), corner);
        let mut children = opener.children().to_vec();
        self.info_text = children.remove(1);
        self.opener_rect = children.remove(0);
    }
}

impl From<ControlPanelMobject> for Mobject {
    fn from(p: ControlPanelMobject) -> Self {
        p.composition().into()
    }
}

/// `to_corner(corner, buff)`: move the mobject's `corner` bbox point to the
/// frame's matching corner, inset by `buff`.
fn to_corner(mob: VMobject, corner: Vec3, buff: f64) -> VMobject {
    let target = [
        corner[0].signum() * (FRAME_X_RADIUS - buff),
        corner[1].signum() * (FRAME_Y_RADIUS - buff),
        0.0,
    ];
    mob.moved_to_aligned(target, corner)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> FontBook {
        FontBook::bundled().expect("bundled faces parse")
    }

    fn assert_vec3_close(actual: Vec3, expected: Vec3, tol: f64, what: &str) {
        for dim in 0..3 {
            assert!(
                (actual[dim] - expected[dim]).abs() <= tol,
                "{what}: dim {dim}: {} vs {}",
                actual[dim],
                expected[dim]
            );
        }
    }

    // ----------------------------------------------------- MotionMobject

    #[test]
    fn motion_mobject_wraps_its_mobject_in_a_group() {
        let inner = Circle::new().radius(0.5).build();
        let motion = MotionMobject::new(inner.clone()).build();
        assert_eq!(motion.children().len(), 1);
        assert_eq!(motion.children()[0], inner);
    }

    // ------------------------------------------------------------ Button

    #[test]
    fn button_composes_background_rect_and_label() {
        let book = book();
        let button = Button::new("Play").build(&book).expect("label typesets");
        assert_eq!(button.children().len(), 2, "[rect, label]");
        let rect = &button.children()[0];
        let label = &button.children()[1];
        assert!(
            !label.children().is_empty(),
            "the label carries its glyph children"
        );
        // The rect surrounds the label with the buff on every side.
        let (rmin, rmax) = rect.extent().expect("rect has extent");
        let (lmin, lmax) = label.extent().expect("label has extent");
        let buff = MED_SMALL_BUFF;
        assert!((rmin[0] - (lmin[0] - buff)).abs() < 1e-9);
        assert!((rmax[0] - (lmax[0] + buff)).abs() < 1e-9);
        assert!((rmin[1] - (lmin[1] - buff)).abs() < 1e-9);
        assert!((rmax[1] - (lmax[1] + buff)).abs() < 1e-9);
    }

    #[test]
    fn button_of_mobject_form_wraps_arbitrary_content() {
        let book = book();
        let face = Circle::new().radius(0.25).build();
        let button = Button::of(face).build(&book).expect("no label to typeset");
        assert_eq!(button.children().len(), 2);
        assert!(!button.children()[1].points().is_empty());
    }

    // ----------------------------------------------------- ControlMobject

    #[test]
    fn control_mobject_base_holds_value_and_children() {
        let mut base = ControlMobject::new(2.5, [Circle::new().build()]);
        assert_eq!(base.value(), 2.5);
        base.set_value(7.0);
        assert_eq!(base.value(), 7.0);
        assert_eq!(base.composition().children().len(), 1);
    }

    // ------------------------------------------------- EnableDisableButton

    #[test]
    fn enable_disable_button_constructs_white_then_colors_on_set() {
        let mut button = EnableDisableButton::new(true);
        // The Reference quirk: ValueTracker.__init__ bypasses
        // set_value_anim, so the box constructs with the default fill.
        assert_eq!(button.rect().style().fill_color, WHITE);
        assert_eq!(button.rect().style().fill_opacity, 1.0);

        button.toggle_value();
        assert!(!button.value());
        assert_eq!(button.rect().style().fill_color, RED);

        button.toggle_value();
        assert!(button.value());
        assert_eq!(button.rect().style().fill_color, GREEN);

        button.set_value(false);
        assert_eq!(button.rect().style().fill_color, RED);
    }

    #[test]
    fn enable_disable_button_uses_custom_colors_when_set_and_toggled() {
        let enable_color = fmn_core::constants::BLUE;
        let disable_color = fmn_core::constants::YELLOW;
        let mut button = EnableDisableButton::new(true)
            .enable_color(enable_color)
            .disable_color(disable_color);
        assert_eq!(button.rect().style().fill_color, WHITE);

        button.set_value(true);
        assert_eq!(button.rect().style().fill_color, enable_color);

        button.toggle_value();
        assert!(!button.value());
        assert_eq!(button.rect().style().fill_color, disable_color);
    }

    #[test]
    fn enable_disable_button_custom_fill_opacity() {
        let mut button = EnableDisableButton::new(true).fill_opacity(0.4);
        assert_eq!(button.rect().style().fill_opacity, 0.4);
        button.set_value(false);
        assert_eq!(button.rect().style().fill_opacity, 0.4);
        assert_eq!(button.rect().style().fill_color, RED);
    }

    #[test]
    fn enable_disable_button_scalar_encoding() {
        let button = EnableDisableButton::new(true);
        assert_eq!(button.scalar_value(), 1.0);
        let off = button.at_value(0.0);
        assert!(!off.value());
        assert_eq!(off.rect().style().fill_color, RED);
        assert_eq!(button.composition().children().len(), 1);
    }

    // ----------------------------------------------------------- Checkbox

    #[test]
    fn checkbox_composition_and_toggle_records() {
        let mut checkbox = Checkbox::new(true);
        let comp = checkbox.composition();
        assert_eq!(comp.children().len(), 2, "[box, content]");

        // Box: 0.5×0.5, transparent fill.
        let rect = checkbox.rect();
        assert!((rect.length_over_dim(0) - 0.5).abs() < 1e-9);
        assert!((rect.length_over_dim(1) - 0.5).abs() < 1e-9);
        assert_eq!(rect.style().fill_opacity, 0.0);

        let filled = Checkbox::new(true).box_fill_opacity(0.35);
        assert_eq!(filled.rect().style().fill_opacity, 0.35);

        // Checked: GREEN checkmark, half the box's size, centred on it.
        assert_eq!(checkbox.content().style().fill_color, GREEN);
        assert_eq!(checkbox.content().style().stroke_width, 0.0);
        assert!((checkbox.content().length_over_dim(0) - 0.25).abs() < 1e-9);
        assert!((checkbox.content().length_over_dim(1) - 0.25).abs() < 1e-9);
        assert_vec3_close(
            checkbox.content().center_point(),
            rect.center_point(),
            1e-9,
            "content centred on box",
        );

        // Toggle: the content becomes the RED cross, same size and place.
        checkbox.toggle_value();
        assert!(!checkbox.value());
        assert_eq!(checkbox.content().style().fill_color, RED);
        assert!((checkbox.content().length_over_dim(0) - 0.25).abs() < 1e-9);

        checkbox.toggle_value();
        assert!(checkbox.value());
        assert_eq!(checkbox.content().style().fill_color, GREEN);

        let mut stroked = Checkbox::new(true)
            .checkmark_stroke_width(3.0)
            .cross_stroke_width(4.0);
        assert_eq!(stroked.content().style().stroke_width, 3.0);
        stroked.toggle_value();
        assert_eq!(stroked.content().style().stroke_width, 4.0);
        assert_eq!(stroked.content().style().fill_color, RED);
    }

    // --------------------------------------------------- LinearNumberSlider

    #[test]
    fn slider_default_composition() {
        let slider = LinearNumberSlider::new(0.0).expect("valid slider");
        let comp = slider.composition();
        assert_eq!(comp.children().len(), 3, "[bar, handle, axis]");

        // Bar: 2 wide, 0.075 tall.
        assert!((slider.bar().length_over_dim(0) - 2.0).abs() < 1e-9);
        assert!((slider.bar().length_over_dim(1) - 0.075).abs() < 1e-9);

        // Axis: invisible, spanning the bar.
        assert_eq!(slider.axis().style().stroke_opacity, 0.0);
        assert_eq!(slider.axis().style().fill_opacity, 0.0);
        assert!((slider.axis().length_over_dim(0) - 2.0).abs() < 1e-9);

        // Handle: radius 0.1, GREY_A, at the origin for value 0 on
        // [-10, 10] — midpoint and fraction coincide here.
        assert!((slider.handle().length_over_dim(0) - 0.2).abs() < 1e-9);
        assert_eq!(slider.handle().style().fill_color, GREY_A);
        assert_eq!(slider.handle().style().fill_opacity, 1.0);
        assert_vec3_close(slider.handle().center_point(), ORIGIN, 1e-9, "handle");
    }

    #[test]
    fn slider_custom_corner_radius_and_handle() {
        let slider = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .corner_radius(0.02)
            .expect("corner radius fits")
            .handle_radius(0.2)
            .handle_fill_opacity(0.4);
        assert!((slider.handle().length_over_dim(0) - 0.4).abs() < 1e-9);
        assert_eq!(slider.handle().style().fill_opacity, 0.4);

        let split = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .handle_stroke_color(RED)
            .handle_fill_color(GREEN);
        assert_eq!(split.handle().style().stroke_color, RED);
        assert_eq!(split.handle().style().fill_color, GREEN);
    }

    #[test]
    fn slider_set_value_moves_handle_to_value_fraction() {
        let mut slider = LinearNumberSlider::new(0.0).expect("valid slider");
        // Bar spans x in [-1, 1]; fraction (v+10)/20 maps linearly.
        slider.set_value(10.0).expect("in range");
        assert_vec3_close(slider.handle().center_point(), [1.0, 0.0, 0.0], 1e-9, "max");
        slider.set_value(-10.0).expect("in range");
        assert_vec3_close(
            slider.handle().center_point(),
            [-1.0, 0.0, 0.0],
            1e-9,
            "min",
        );
        slider.set_value(5.0).expect("in range");
        assert_vec3_close(slider.handle().center_point(), [0.5, 0.0, 0.0], 1e-9, "mid");
        assert_eq!(slider.value(), 5.0);
    }

    #[test]
    fn slider_set_value_clamps_instead_of_panicking() {
        let mut slider = LinearNumberSlider::new(0.0).expect("valid slider");
        slider.set_value(100.0).expect("clamped, not panicked");
        assert_eq!(slider.value(), 10.0);
        assert_vec3_close(
            slider.handle().center_point(),
            [1.0, 0.0, 0.0],
            1e-9,
            "clamped",
        );
        slider.set_value(-100.0).expect("clamped, not panicked");
        assert_eq!(slider.value(), -10.0);
    }

    #[test]
    fn slider_range_clamps_without_snapping_and_refuses_invalid_bounds() {
        let mut slider = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .range(0.0, 1.0)
            .expect("ordered finite range");
        slider.set_value(2.0).expect("out-of-range value clamps");
        assert_eq!(slider.value(), 1.0);
        slider.set_value(0.25).expect("in-range value is accepted");
        assert_eq!(slider.value(), 0.25);

        assert!(matches!(
            LinearNumberSlider::new(0.0)
                .expect("valid slider")
                .range(1.0, 0.0),
            Err(SliderError::InvalidRange)
        ));
        assert!(matches!(
            LinearNumberSlider::new(0.0)
                .expect("valid slider")
                .range(f64::NAN, 1.0),
            Err(SliderError::InvalidRange)
        ));
    }

    #[test]
    fn slider_refuses_non_finite_values_before_mutation() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                LinearNumberSlider::new(value),
                Err(SliderError::NonFiniteValue)
            ));
        }

        let mut slider = LinearNumberSlider::new(0.0).expect("valid slider");
        let before_handle = slider.handle().clone();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                slider.set_value(value),
                Err(SliderError::NonFiniteValue)
            ));
            assert_eq!(slider.value(), 0.0);
            assert_eq!(slider.handle(), &before_handle);
        }
    }

    #[test]
    fn slider_builders_refuse_invalid_ranges_and_steps() {
        for min in [11.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                LinearNumberSlider::new(0.0)
                    .expect("valid slider")
                    .min_value(min),
                Err(SliderError::InvalidRange)
            ));
        }
        for max in [-11.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                LinearNumberSlider::new(0.0)
                    .expect("valid slider")
                    .max_value(max),
                Err(SliderError::InvalidRange)
            ));
        }
        for step in [
            0.0,
            -1.0,
            f64::from_bits(1),
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert!(matches!(
                LinearNumberSlider::new(0.0)
                    .expect("valid slider")
                    .step(step),
                Err(SliderError::InvalidStep)
            ));
        }

        let wide = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .min_value(-f64::MAX)
            .expect("one extreme bound still has a finite span");
        assert!(matches!(
            wide.max_value(f64::MAX),
            Err(SliderError::InvalidRange)
        ));
    }

    #[test]
    fn slider_value_from_point_projects_and_snaps() {
        let slider = LinearNumberSlider::new(0.0).expect("valid slider");
        // y is ignored: the point is projected onto the axis.
        assert_eq!(
            slider
                .value_from_point([-1.0, 0.3, 0.0])
                .expect("finite point"),
            -10.0
        );
        assert_eq!(
            slider
                .value_from_point([0.0, 0.0, 0.0])
                .expect("finite point"),
            0.0
        );
        // prop 0.775 → value 5.5 → snapped down to step 1 → 5.
        assert_eq!(
            slider
                .value_from_point([0.55, 0.0, 0.0])
                .expect("finite point"),
            5.0
        );
        // Off the ends, clamped to the axis.
        assert_eq!(
            slider
                .value_from_point([5.0, 0.0, 0.0])
                .expect("finite point"),
            10.0
        );
        assert_eq!(
            slider
                .value_from_point([-5.0, 0.0, 0.0])
                .expect("finite point"),
            -10.0
        );
    }

    #[test]
    fn slider_projection_refuses_invalid_inputs_and_preserves_degenerate_axes() {
        let slider = LinearNumberSlider::new(0.0).expect("valid slider");
        for point in [
            [f64::NAN, 0.0, 0.0],
            [0.0, f64::INFINITY, 0.0],
            [0.0, 0.0, f64::NEG_INFINITY],
        ] {
            assert!(matches!(
                slider.value_from_point(point),
                Err(SliderError::NonFinitePoint)
            ));
        }

        let zero_axis = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .bar_width(0.0)
            .expect("a zero-width finite bar remains defined");
        assert_eq!(
            zero_axis
                .value_from_point([100.0, 5.0, 0.0])
                .expect("zero-width axes are defined"),
            -10.0
        );

        let mut invalid_range = slider.clone();
        invalid_range.min_value = 1.0;
        invalid_range.max_value = -1.0;
        assert!(matches!(
            invalid_range.value_from_point(ORIGIN),
            Err(SliderError::InvalidRange)
        ));

        let mut invalid_step = slider;
        invalid_step.step = 0.0;
        assert!(matches!(
            invalid_step.value_from_point(ORIGIN),
            Err(SliderError::InvalidStep)
        ));
    }

    #[test]
    fn slider_equal_bounds_are_valid_and_stable() {
        let mut slider = LinearNumberSlider::new(5.0)
            .expect("valid slider")
            .min_value(5.0)
            .expect("ordered lower bound")
            .max_value(5.0)
            .expect("equal bounds are valid");
        slider.set_value(-100.0).expect("finite value clamps");
        assert_eq!(slider.value(), 5.0);
        assert_eq!(
            slider
                .value_from_point([1.0, 2.0, 3.0])
                .expect("zero-span ranges are defined"),
            5.0
        );
        assert!(
            slider
                .handle()
                .center_point()
                .into_iter()
                .all(f64::is_finite)
        );
    }

    #[test]
    fn slider_handle_parks_at_axis_midpoint_at_construction() {
        // The Reference quirk: value 5 on [-10, 10] still constructs with
        // the handle at the axis midpoint; the first set_value fixes it.
        let mut slider = LinearNumberSlider::new(5.0).expect("valid slider");
        assert_vec3_close(slider.handle().center_point(), ORIGIN, 1e-9, "parked");
        slider.set_value(5.0).expect("in range");
        assert_vec3_close(
            slider.handle().center_point(),
            [0.5, 0.0, 0.0],
            1e-9,
            "fixed",
        );
    }

    #[test]
    fn slider_readout_failure_is_atomic() {
        let book = Rc::new(book());
        let constrained = DecimalNumber::new(0.0)
            .num_decimal_places(0)
            .character_limit(1)
            .build(&book)
            .expect("one displayed digit fits");
        let mut slider = LinearNumberSlider::new(0.0).expect("valid slider");
        slider.readout = Some(constrained);
        slider.readout_book = Some(Rc::clone(&book));
        slider.num_decimal_places = 0;

        let before_handle = slider.handle().clone();
        let before_readout = slider.readout().expect("installed readout").vmob().clone();
        assert!(matches!(
            slider.set_value(10.0),
            Err(SliderError::Text(TextMobjectError::ResourceLimit { .. }))
        ));
        assert_eq!(slider.value(), 0.0);
        assert_eq!(slider.handle(), &before_handle);
        assert_eq!(
            slider.readout().expect("readout remains installed").value(),
            0.0
        );
        assert_eq!(
            slider.readout().expect("readout remains installed").vmob(),
            &before_readout
        );
    }

    #[test]
    fn slider_readout_tracks_the_value() {
        let book = Rc::new(book());
        let mut slider = LinearNumberSlider::new(0.0)
            .expect("valid slider")
            .with_value_readout(Rc::clone(&book), 2)
            .expect("readout typesets");
        assert_eq!(slider.composition().children().len(), 4, "readout trails");
        let before = slider
            .readout()
            .expect("readout enabled")
            .vmob()
            .children()
            .len();
        assert!(before > 0, "the readout drew digits");
        slider.set_value(3.5).expect("in range");
        let readout = slider.readout().expect("readout enabled");
        assert_eq!(readout.value(), 3.5);
        // The readout sits above the bar.
        let comp = slider.composition();
        let readout_vmob = &comp.children()[3];
        let (_, rmax) = slider.bar().extent().expect("bar extent");
        let (vmin, _) = readout_vmob.extent().expect("readout extent");
        assert!(
            (vmin[1] - (rmax[1] + SMALL_BUFF)).abs() < 1e-6,
            "readout above the bar: {} vs {}",
            vmin[1],
            rmax[1] + SMALL_BUFF
        );
    }

    // --------------------------------------------------------- ColorSliders

    #[test]
    fn color_sliders_defaults_and_structure() {
        let sliders = ColorSliders::new().expect("locked sliders are valid");
        assert_eq!(sliders.value(), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sliders.picked_color(), WHITE);
        assert_eq!(sliders.picked_opacity(), 1.0);

        // Slider configs: rgb 0..255 step 1, a 0..1 step 0.04.
        assert_eq!(sliders.sliders()[0].min(), 0.0);
        assert_eq!(sliders.sliders()[0].max(), 255.0);
        assert_eq!(sliders.sliders()[3].max(), 1.0);

        // Handle tints: RED, GREEN, BLUE, midpoint grey.
        assert_eq!(sliders.sliders()[0].handle().style().fill_color, RED);
        assert_eq!(sliders.sliders()[1].handle().style().fill_color, GREEN);
        assert_eq!(
            sliders.sliders()[2].handle().style().fill_color,
            fmn_core::constants::BLUE
        );
        assert_eq!(sliders.sliders()[3].handle().style().fill_color, GREY_C);

        // Composition: [swatch group (checkerboard + box), slider column].
        let comp = sliders.composition();
        assert_eq!(comp.children().len(), 2);
        assert_eq!(comp.children()[0].children().len(), 2);
        assert_eq!(comp.children()[1].children().len(), 4);
    }

    #[test]
    fn color_sliders_checkerboard() {
        let sliders = ColorSliders::new().expect("locked sliders are valid");
        let background = sliders.background();
        // rows = 5, cols = 20 → bumped to 21: 105 squares.
        assert_eq!(background.children().len(), 105);
        // Alternating GREY_A / GREY_C fills, no stroke.
        assert_eq!(background.children()[0].style().fill_color, GREY_A);
        assert_eq!(background.children()[1].style().fill_color, GREY_C);
        assert_eq!(background.children()[0].style().stroke_width, 0.0);
        // Stretched exactly onto the 2×0.5 rect, centred on the box.
        assert!((background.length_over_dim(0) - 2.0).abs() < 1e-9);
        assert!((background.length_over_dim(1) - 0.5).abs() < 1e-9);
        assert_vec3_close(
            background.center_point(),
            sliders.color_box().center_point(),
            1e-9,
            "checkerboard centred on swatch",
        );
    }

    #[test]
    fn color_sliders_rect_size_rebuilds_or_refuses_geometry() {
        let sliders = ColorSliders::new()
            .expect("locked sliders are valid")
            .rect_size(1.0, 0.25)
            .expect("positive swatch dimensions");
        assert!((sliders.color_box().length_over_dim(0) - 1.0).abs() < 1e-9);
        assert!((sliders.color_box().length_over_dim(1) - 0.25).abs() < 1e-9);

        for width in [0.0, f64::NAN] {
            assert!(matches!(
                ColorSliders::new()
                    .expect("locked sliders are valid")
                    .rect_size(width, 0.5),
                Err(SliderError::Geometry(_))
            ));
        }
    }

    #[test]
    fn color_sliders_set_value_updates_swatch() {
        let mut sliders = ColorSliders::new().expect("locked sliders are valid");
        sliders.set_value(0.0, 0.0, 0.0, 1.0).expect("in range");
        assert_eq!(sliders.picked_color(), fmn_core::constants::BLACK);
        assert_eq!(
            sliders.color_box().style().fill_color,
            fmn_core::constants::BLACK
        );

        sliders.set_value(255.0, 0.0, 0.0, 0.5).expect("in range");
        let picked = sliders.picked_color();
        assert_eq!(
            picked,
            Srgb {
                r: 1.0,
                g: 0.0,
                b: 0.0
            }
        );
        assert_eq!(sliders.picked_opacity(), 0.5);
        assert_eq!(sliders.color_box().style().fill_opacity, 0.5);
    }

    #[test]
    fn color_slider_overrides_snap_rgb_but_keep_alpha_step() {
        let mut sliders = ColorSliders::new()
            .expect("locked sliders are valid")
            .slider_overrides(
                None,
                None,
                Some(2.0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("shared RGB step is valid");
        assert_eq!(sliders.sliders()[0].step, 2.0);
        assert_eq!(sliders.sliders()[3].step, 0.04);

        sliders
            .set_value(63.0, 63.0, 63.0, 0.53)
            .expect("finite component values");
        assert_eq!(sliders.sliders()[0].value(), 62.0);
        assert_eq!(sliders.sliders()[1].value(), 62.0);
        assert_eq!(sliders.sliders()[2].value(), 62.0);
        assert_eq!(sliders.sliders()[3].value(), 0.53);
    }

    #[test]
    fn color_slider_overrides_apply_shared_range_and_clamp_values() {
        let mut sliders = ColorSliders::new()
            .expect("locked sliders are valid")
            .slider_overrides(
                Some(0.0),
                Some(1.0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("shared unit range is valid");
        assert_eq!(sliders.sliders()[0].min(), 0.0);
        assert_eq!(sliders.sliders()[0].max(), 1.0);

        sliders
            .set_value(2.0, 2.0, 2.0, 2.0)
            .expect("component values clamp into the configured range");
        assert_eq!(
            sliders.value(),
            [1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0, 1.0]
        );
    }

    #[test]
    fn color_slider_overrides_refuse_invalid_steps() {
        for step in [-1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                ColorSliders::new()
                    .expect("locked sliders are valid")
                    .slider_overrides(
                        None,
                        None,
                        Some(step),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                Err(SliderError::InvalidStep)
            ));
        }
    }

    #[test]
    fn color_slider_updates_are_atomic_on_invalid_component_values() {
        let mut sliders = ColorSliders::new().expect("locked sliders are valid");
        let before = sliders.value();
        let before_box = sliders.color_box().clone();
        assert!(matches!(
            sliders.set_value(0.0, f64::NAN, 0.0, 1.0),
            Err(SliderError::NonFiniteValue)
        ));
        assert_eq!(sliders.value(), before);
        assert_eq!(sliders.color_box(), &before_box);
    }

    // ------------------------------------------------------------ Textbox

    #[test]
    fn textbox_composition_and_active_frame() {
        let book = Rc::new(book());
        let mut textbox = Textbox::new(Rc::clone(&book), "").expect("empty text typesets");
        let comp = textbox.composition();
        assert_eq!(comp.children().len(), 2, "[box, text]");

        // Box: 2×1, default-colour fill, RED frame while inactive.
        assert!((textbox.rect().length_over_dim(0) - 2.0).abs() < 1e-9);
        assert!((textbox.rect().length_over_dim(1) - 1.0).abs() < 1e-9);
        assert_eq!(textbox.rect().style().fill_color, DEFAULT_MOBJECT_COLOR);
        assert_eq!(textbox.rect().style().stroke_color, RED);
        assert!(!textbox.is_active());

        textbox.set_active(true);
        assert!(textbox.is_active());
        assert_eq!(
            textbox.rect().style().stroke_color,
            fmn_core::constants::BLUE
        );

        textbox.toggle_active();
        assert!(!textbox.is_active());
        assert_eq!(textbox.rect().style().stroke_color, RED);
    }

    #[test]
    fn textbox_set_value_retypesets_into_the_box() {
        let book = Rc::new(book());
        let mut textbox = Textbox::new(Rc::clone(&book), "").expect("empty text typesets");
        textbox.set_value("hi").expect("text typesets");
        assert_eq!(textbox.value(), "hi");
        assert!(
            !textbox.text().children().is_empty(),
            "the new text drew glyphs"
        );
        // Fit to box width minus the padding on both sides, centred.
        assert!(textbox.text().length_over_dim(0) <= 2.0 - 2.0 * MED_SMALL_BUFF + 1e-9);
        assert_vec3_close(
            textbox.text().center_point(),
            textbox.rect().center_point(),
            1e-9,
            "text centred on box",
        );
        // Text colour from text_kwargs.
        assert_eq!(textbox.text().style().fill_color, fmn_core::constants::BLUE);
    }

    #[test]
    fn textbox_custom_box_text_and_frame_kwargs() {
        let book = Rc::new(book());
        let mut textbox = Textbox::new(Rc::clone(&book), "hi")
            .expect("text typesets")
            .box_size(3.0, 1.5)
            .expect("resized text typesets")
            .box_fill(GREEN, 0.4)
            .text_color(RED)
            .expect("coloured text typesets")
            .text_buff(0.1)
            .expect("padded text typesets")
            .active_color(WHITE)
            .deactive_color(GREEN);
        assert!((textbox.rect().length_over_dim(0) - 3.0).abs() < 1e-9);
        assert!((textbox.rect().length_over_dim(1) - 1.5).abs() < 1e-9);
        assert_eq!(textbox.rect().style().fill_color, GREEN);
        assert_eq!(textbox.rect().style().fill_opacity, 0.4);
        assert_eq!(textbox.rect().style().stroke_color, GREEN);
        assert_eq!(textbox.text().style().fill_color, RED);
        textbox.set_active(true);
        assert_eq!(textbox.rect().style().stroke_color, WHITE);
        assert_eq!(textbox.rect().style().fill_color, GREEN);
    }

    // -------------------------------------------------------- ControlPanel

    #[test]
    fn control_panel_empty_family_and_info_text() {
        let book = book();
        let panel = ControlPanel::new([])
            .opener_text("Panel")
            .build(&book)
            .expect("opener text typesets");
        let (panel_part, opener, controls) = panel.family_parts();
        assert_eq!(panel.composition().children().len(), 3);
        assert_eq!(opener.children().len(), 2, "opener: [rect, text]");
        assert_eq!(controls.children().len(), 0);
        assert_eq!(panel_part, panel.panel());
        assert_eq!(panel.info_text(), &opener.children()[1]);
        assert!(
            !panel.info_text().children().is_empty(),
            "opener label typeset glyphs"
        );
        let (laid_panel, laid_controls) = ControlPanelMobject::layout_against_opener(
            panel.panel().clone(),
            panel.opener_rect(),
            panel.controls().clone(),
        );
        assert_eq!(&laid_panel, panel.panel());
        assert_eq!(&laid_controls, panel.controls());
    }

    #[test]
    fn control_panel_composition() {
        let book = book();
        let panel = ControlPanel::new([Checkbox::new(true).composition()])
            .build(&book)
            .expect("opener text typesets");
        let comp = panel.composition();
        assert_eq!(comp.children().len(), 3, "[panel, opener, controls]");
        assert_eq!(
            comp.children()[1].children().len(),
            2,
            "opener: [rect, text]"
        );
        assert_eq!(comp.children()[2].children().len(), 1, "one control");
        assert_eq!(
            panel.info_text(),
            &comp.children()[1].children()[1],
            "info text is the opener's second child"
        );

        // Panel geometry: FRAME_WIDTH/4 × MED_SMALL_BUFF + FRAME_HEIGHT.
        assert!((panel.panel().length_over_dim(0) - FRAME_WIDTH / 4.0).abs() < 1e-9);
        assert!((panel.panel().length_over_dim(1) - (MED_SMALL_BUFF + FRAME_HEIGHT)).abs() < 1e-9);
        assert_eq!(panel.panel().style().fill_color, GREY_C);
        assert_eq!(panel.panel().style().fill_opacity, 1.0);
        assert_eq!(
            panel.panel().style().stroke_color,
            DEFAULT_VMOBJECT_STROKE_COLOR
        );
        assert_eq!(panel.panel().style().stroke_width, 0.0);
        assert_eq!(panel.panel().style().stroke_opacity, 1.0);
        assert!((panel.opener_rect().length_over_dim(0) - FRAME_WIDTH / 8.0).abs() < 1e-9);
        assert!((panel.opener_rect().length_over_dim(1) - 0.5).abs() < 1e-9);
        assert_eq!(panel.opener_rect().style().fill_color, GREY_C);
        assert_eq!(panel.opener_rect().style().fill_opacity, 1.0);
        assert_eq!(
            panel.opener_rect().style().stroke_color,
            DEFAULT_VMOBJECT_STROKE_COLOR
        );
        assert_eq!(
            panel.opener_rect().style().stroke_width,
            DEFAULT_STROKE_WIDTH
        );
        assert_eq!(panel.opener_rect().style().stroke_opacity, 1.0);

        // After move_panel_and_controls_to_panel_opener the panel sits
        // directly on the opener (buff 0)…
        let (pmin, _) = panel.panel().extent().expect("panel extent");
        let (_, omax) = panel.opener_rect().extent().expect("opener extent");
        assert!(
            (pmin[1] - omax[1]).abs() < 1e-9,
            "panel bottom on opener top: {} vs {}",
            pmin[1],
            omax[1]
        );
        // …and the controls sit MED_SMALL_BUFF above the opener.
        let (cmin, _) = panel.controls().extent().expect("controls extent");
        assert!(
            (cmin[1] - (omax[1] + MED_SMALL_BUFF)).abs() < 1e-9,
            "controls above opener: {} vs {}",
            cmin[1],
            omax[1] + MED_SMALL_BUFF
        );
    }

    #[test]
    fn control_panel_open_close_and_add_remove() {
        let book = book();
        let mut panel = ControlPanel::new([Checkbox::new(true).composition()])
            .build(&book)
            .expect("opener text typesets");

        panel.open_panel();
        let (omin, _) = panel.opener_rect().extent().expect("opener extent");
        assert!(
            (omin[1] - (-FRAME_Y_RADIUS)).abs() < 1e-9,
            "opener bottom on the frame bottom: {}",
            omin[1]
        );
        let opener_x = panel.opener_rect().center_point()[0];

        panel.close_panel();
        let (_, omax) = panel.opener_rect().extent().expect("opener extent");
        assert!(
            (omax[1] - FRAME_Y_RADIUS).abs() < 1e-9,
            "opener top on the frame top: {}",
            omax[1]
        );
        // The opener keeps its x across open/close.
        assert!(
            (panel.opener_rect().center_point()[0] - opener_x).abs() < 1e-9,
            "opener x preserved"
        );

        // add/remove ride along.
        panel.add_controls([EnableDisableButton::new(true).composition()]);
        assert_eq!(panel.controls().children().len(), 2);
        panel.remove_controls(&[0]);
        assert_eq!(panel.controls().children().len(), 1);
    }

    #[test]
    fn control_panel_custom_panel_and_opener_kwargs() {
        let book = book();
        let panel = ControlPanel::new([])
            .panel_width(3.0)
            .panel_height(2.0)
            .panel_fill(RED, 0.4)
            .panel_stroke(WHITE, 2.0, 0.5)
            .opener_width(1.5)
            .opener_height(0.75)
            .opener_fill(GREEN, 0.8)
            .opener_stroke(GREY_A, 1.5, 0.25)
            .opener_text_color(RED)
            .build(&book)
            .expect("opener text typesets");
        assert!((panel.panel().length_over_dim(0) - 3.0).abs() < 1e-9);
        assert!((panel.panel().length_over_dim(1) - 2.0).abs() < 1e-9);
        assert_eq!(panel.panel().style().fill_color, RED);
        assert_eq!(panel.panel().style().fill_opacity, 0.4);
        assert_eq!(panel.panel().style().stroke_color, WHITE);
        assert_eq!(panel.panel().style().stroke_width, 2.0);
        assert_eq!(panel.panel().style().stroke_opacity, 0.5);
        assert!((panel.opener_rect().length_over_dim(0) - 1.5).abs() < 1e-9);
        assert!((panel.opener_rect().length_over_dim(1) - 0.75).abs() < 1e-9);
        assert_eq!(panel.opener_rect().style().fill_color, GREEN);
        assert_eq!(panel.opener_rect().style().fill_opacity, 0.8);
        assert_eq!(panel.opener_rect().style().stroke_color, GREY_A);
        assert_eq!(panel.opener_rect().style().stroke_width, 1.5);
        assert_eq!(panel.opener_rect().style().stroke_opacity, 0.25);
        assert_eq!(panel.info_text().style().fill_color, RED);
    }

    // --------------------------------------------- tracker integration

    /// Read one entry's first fill RGBA record off the Stage.
    fn stage_fill(stage: &Stage, mob: Mob) -> [f32; 4] {
        let entry = stage.get(mob).expect("entry alive");
        let rgba = entry.buffer.read(0, "fill_rgba").expect("fill_rgba field");
        [rgba[0], rgba[1], rgba[2], rgba[3]]
    }

    /// The horizontal centre of an entry's points.
    fn stage_x_center(stage: &Stage, mob: Mob) -> f64 {
        let entry = stage.get(mob).expect("entry alive");
        let points = entry.buffer.read_column("point").expect("point field");
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for x in points.iter().step_by(3) {
            min = min.min(f64::from(*x));
            max = max.max(f64::from(*x));
        }
        0.5 * (min + max)
    }

    #[test]
    fn checkbox_tracks_the_stage_value() {
        let mut stage = Stage::new();
        let control = add_scalar_control(&mut stage, &Checkbox::new(true));
        stage.add_to_scene(control.container).expect("fresh handle");
        assert_eq!(stage.tracker_value(control.tracker), Some(1.0));

        // Initially checked: the content child fills GREEN.
        let comp = stage
            .get(control.container)
            .expect("container")
            .submobjects()[0];
        let content = stage.get(comp).expect("composition").submobjects()[1];
        let green = stage_fill(&stage, content);
        assert!((f64::from(green[1]) - GREEN.g).abs() < 1e-3, "green fill");

        // Drive the tracker to false: the next tick rebuilds the
        // composition with the RED cross.
        stage
            .set_tracker_value(control.tracker, 0.0)
            .expect("tracker");
        stage.update(0.1);
        let comp = stage
            .get(control.container)
            .expect("container")
            .submobjects()[0];
        let content = stage.get(comp).expect("composition").submobjects()[1];
        let red = stage_fill(&stage, content);
        assert!((f64::from(red[0]) - RED.r).abs() < 1e-3, "red fill");
        assert!((f64::from(red[1]) - RED.g).abs() < 1e-3, "red fill");
    }

    #[test]
    fn slider_tracks_the_stage_value() {
        let mut stage = Stage::new();
        let slider = LinearNumberSlider::new(0.0).expect("valid slider");
        let control = add_scalar_control(&mut stage, &slider);
        stage.add_to_scene(control.container).expect("fresh handle");

        // Tracker to max: the next tick parks the handle at x = 1.
        stage
            .set_tracker_value(control.tracker, 10.0)
            .expect("tracker");
        stage.update(0.1);
        let comp = stage
            .get(control.container)
            .expect("container")
            .submobjects()[0];
        let handle = stage.get(comp).expect("composition").submobjects()[1];
        assert!(
            (stage_x_center(&stage, handle) - 1.0).abs() < 1e-3,
            "handle at the max end: {}",
            stage_x_center(&stage, handle)
        );

        // And back: x = -0.25 for value -2.5 on [-10, 10].
        stage
            .set_tracker_value(control.tracker, -2.5)
            .expect("tracker");
        stage.update(0.1);
        let comp = stage
            .get(control.container)
            .expect("container")
            .submobjects()[0];
        let handle = stage.get(comp).expect("composition").submobjects()[1];
        assert!(
            (stage_x_center(&stage, handle) - (-0.25)).abs() < 1e-3,
            "handle at the value fraction: {}",
            stage_x_center(&stage, handle)
        );
    }
}
