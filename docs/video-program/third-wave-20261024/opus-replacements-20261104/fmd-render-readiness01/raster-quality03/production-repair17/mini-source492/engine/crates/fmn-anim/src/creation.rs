//! The partial-reveal mechanism — family 2 of the five (§9.4, fm-cye):
//! every class here drives frames through the appropriate typed form of
//! the one partial-reveal data-plane operation: quadratic paths use
//! `Stage::pointwise_become_partial` (vectorized_mobject.py:1050), while
//! sampled UV grids use `Stage::surface_pointwise_become_partial`
//! (surface.py:176). Both are ported from the pinned Reference's
//! `animation/creation.py` call site (with the `ShowPartial` bounds
//! vocabulary shared by `indication.py`'s passing flashes):
//!
//! - [`ShowPartial`] is the mechanism: per zipped family pair, restrict
//!   the live submobject to a proportion window of its starting copy.
//!   [`RevealBounds`] carries the two bounds rules as data —
//!   `Creation` (`(0, alpha)`, creation.py:52) and `PassingFlash`
//!   (the sliding `time_width` window, indication.py:179).
//! - [`show_creation`] (creation.py:47, `lag_ratio=1.0`), [`uncreate`]
//!   (creation.py:56: reversed smooth, remover, `should_match_start` —
//!   which the pinned tree stores and never reads; kept as inert
//!   constructor surface), and [`show_passing_flash`] (indication.py:165,
//!   whose `finish` restores every pair to the full run — the
//!   [`Animation::teardown`] slot here).
//! - [`DrawBorderThenFill`] (creation.py:75): an outline copy (fill
//!   opacity 0, stroke `stroke_width`/`stroke_color`, `behind` matching
//!   the animated root) revealed over the first half, then cross-faded
//!   `outline → start` over the second, with the Reference's first-crossing
//!   `set_data` and the `finish`-time joint-angle refresh. [`write()`]
//!   (creation.py:140) is its parameterization: `run_time`/`lag_ratio`
//!   derived from the family's with-points count, linear rate, stroke
//!   color from `get_color()`.
//! - [`ShowIncreasingSubsets`] (creation.py:176) and its
//!   [`show_submobjects_one_by_one`] parameterization (creation.py:200):
//!   the Reference overrides `interpolate_mobject`, so the rate function
//!   applies to the *raw* segment alpha and the lag/time-span pipeline is
//!   bypassed — mirrored here by overriding [`Animation::interpolate`].
//!   `np.round` is banker's rounding (`round_ties_even`); `np.ceil` is
//!   ceiling — [`IntRound`] keeps both as data.
//!
//! - [`add_text_word_by_word`] (creation.py:210, fm-5wq.4.54):
//!   `ShowIncreasingSubsets` re-parameterized over span-map word groups —
//!   the caller (the portal's `StringMobject` surface) has already grouped
//!   the glyph children into word groups by native source spans, so the
//!   grouped parent's children ARE the words. Linear rate, and the
//!   Reference's `run_time < 0` sentinel recomputes
//!   `time_per_word * n_words`. A parent with no word groups is
//!   [`AnimError::NoWordGroups`] — named, never a silent no-op.

use fmn_core::rate;
use fmn_mobject::{Mob, Stage, StageError};

use crate::animation::{AnimConfig, AnimError, AnimState, Animation, AnimationSignature, RateFunc};
use crate::transform::{PathFunc, interpolate_fields};

/// `integer_interpolate` over a two-piece window — a private copy of
/// [`fmn_geom::QuadPath::integer_interpolate`]'s formula (fmn-anim's §19
/// crate edges do not include fmn-geom; the six-line formula is cheaper
/// than a new DAG edge).
fn integer_interpolate(end: i64, alpha: f64) -> (i64, f64) {
    if alpha >= 1.0 {
        return (end - 1, 1.0);
    }
    if alpha <= 0.0 {
        return (0, 0.0);
    }
    #[allow(clippy::cast_possible_truncation)]
    let value = (alpha * end as f64).floor() as i64;
    let residue = (end as f64 * alpha).rem_euclid(1.0);
    (value, residue)
}

/// `smooth(1 - t)` — Uncreate's default rate function (creation.py:60).
fn smooth_reversed(t: f64) -> f64 {
    rate::smooth(1.0 - t)
}

// ------------------------------------------------------------ ShowPartial

/// The `get_bounds` rule as composable data (journal-able, like
/// [`RateFunc`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RevealBounds {
    /// `ShowCreation.get_bounds`: `(0, alpha)` — reveal from the start.
    Creation,
    /// `ShowPassingFlash.get_bounds` (indication.py:179): a window of
    /// width `time_width` sliding across `[0, 1]`.
    PassingFlash {
        /// The window's width in proportion units.
        time_width: f64,
    },
}

impl RevealBounds {
    /// Evaluate at pipeline-final `alpha`, formula for formula.
    #[must_use]
    pub fn eval(self, alpha: f64) -> (f64, f64) {
        match self {
            Self::Creation => (0.0, alpha),
            Self::PassingFlash { time_width } => {
                let upper = alpha * (1.0 + time_width);
                let lower = upper - time_width;
                (lower.max(0.0), upper.min(1.0))
            }
        }
    }
}

/// The partial-reveal mechanism (creation.py:25): per zipped family pair,
/// the live submobject becomes the [`RevealBounds`] restriction of its
/// starting copy.
#[derive(Debug, Clone)]
pub struct ShowPartial {
    state: AnimState,
    bounds: RevealBounds,
    geometry: PartialGeometry,
    /// Stored-but-never-read in the pinned Reference (creation.py:30) —
    /// kept as inert constructor surface for the Parity Ledger.
    should_match_start: bool,
    /// `ShowPassingFlash.finish` restores every pair to the full run
    /// (indication.py:188); runs in the [`Animation::teardown`] slot.
    restore_on_teardown: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartialGeometry {
    QuadraticPath,
    SurfaceGrid {
        resolution: (usize, usize),
        axis: usize,
    },
}

impl ShowPartial {
    /// The bare mechanism under `bounds` (concrete classes are the
    /// constructors below).
    #[must_use]
    pub fn new(mobject: Mob, bounds: RevealBounds) -> Self {
        let config = AnimConfig {
            name: "ShowPartial".to_owned(),
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(mobject, config),
            bounds,
            geometry: PartialGeometry::QuadraticPath,
            should_match_start: false,
            restore_on_teardown: false,
        }
    }

    /// Replace the animation config (run_time, rate_func, lag_ratio …).
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    /// Select the Reference's sampled-Surface reveal plane.
    #[must_use]
    pub fn with_surface_grid(mut self, resolution: (usize, usize), axis: usize) -> Self {
        self.geometry = PartialGeometry::SurfaceGrid { resolution, axis };
        self
    }

    /// The inert Reference parameter, kept on the surface.
    #[must_use]
    pub fn should_match_start(&self) -> bool {
        self.should_match_start
    }
}

impl Animation for ShowPartial {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: every write is `pointwise_become_partial(start, bounds(α))` —
    /// a function of the begin snapshot and alpha alone.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// Validate the selected typed geometry up front, so interpolation can
    /// never silently skip a schema or grid mismatch.
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        let surface = match self.geometry {
            PartialGeometry::QuadraticPath => None,
            PartialGeometry::SurfaceGrid { resolution, axis } => {
                let axis_len = if axis == 0 {
                    resolution.0
                } else {
                    resolution.1
                };
                if axis > 1 || axis_len < 2 {
                    return Err(AnimError::Stage(StageError::SchemaMismatch));
                }
                Some((resolution, resolution.0.checked_mul(resolution.1)))
            }
        };
        for member in stage.family(mobject) {
            if let Some(entry) = stage.get(member)
                && !entry.buffer.is_empty()
            {
                if let Some((_resolution, expected_len)) = surface {
                    let schema = entry.buffer.schema();
                    let fields = schema.fields();
                    let ordinary_surface_schema = fields.len() == 3
                        && fields[0].name == "point"
                        && fields[0].width == 3
                        && fields[1].name == "d_normal_point"
                        && fields[1].width == 3
                        && fields[2].name == "rgba"
                        && fields[2].width == 4;
                    if expected_len != Some(entry.buffer.len()) || !ordinary_surface_schema {
                        return Err(AnimError::Stage(StageError::SchemaMismatch));
                    }
                } else if entry.buffer.schema().offset("joint_angle").is_none() {
                    // The Reference's ShowPartial asserts VMobject for this
                    // branch; Surface selects the typed branch above.
                    return Err(AnimError::Stage(StageError::SchemaMismatch));
                }
            }
        }
        Ok(())
    }

    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, starting] = *mobs else {
            return; // rows are pairs once begin has run
        };
        let (a, b) = self.bounds.eval(sub_alpha);
        if stage
            .get(starting)
            .is_none_or(|entry| entry.buffer.is_empty())
        {
            return;
        }
        // setup validated the family; a failure here means a handle died
        // mid-play, which the frame model makes unreachable.
        match self.geometry {
            PartialGeometry::QuadraticPath => {
                let _ = stage.pointwise_become_partial(submob, starting, a, b);
            }
            PartialGeometry::SurfaceGrid { resolution, axis } => {
                let _ = stage
                    .surface_pointwise_become_partial(submob, starting, resolution, axis, a, b);
            }
        }
    }

    /// indication.py:188 — after `finish`, every pair returns to the full
    /// run (the flash window must not leave a sliver behind).
    fn teardown(&mut self, stage: &mut Stage) {
        if !self.restore_on_teardown {
            return;
        }
        for row in self.state.families().to_vec() {
            if let [submob, starting] = *row {
                if stage
                    .get(starting)
                    .is_none_or(|entry| entry.buffer.is_empty())
                {
                    continue;
                }
                match self.geometry {
                    PartialGeometry::QuadraticPath => {
                        let _ = stage.pointwise_become_partial(submob, starting, 0.0, 1.0);
                    }
                    PartialGeometry::SurfaceGrid { resolution, axis } => {
                        let _ = stage.surface_pointwise_become_partial(
                            submob, starting, resolution, axis, 0.0, 1.0,
                        );
                    }
                }
            }
        }
    }
}

/// `ShowCreation` (creation.py:47): creation bounds, `lag_ratio = 1.0` —
/// strictly successive over the family.
#[must_use]
pub fn show_creation(mobject: Mob) -> ShowPartial {
    let mut anim = ShowPartial::new(mobject, RevealBounds::Creation);
    anim.state_mut().config.name = "ShowCreation".to_owned();
    anim.state_mut().config.lag_ratio = 1.0;
    anim
}

/// `ShowCreation` over a sampled Surface-shaped record
/// family. The grid metadata is explicit constructor data supplied by the
/// front door; Choreo remains independent of the higher Menagerie crate.
#[must_use]
pub fn show_surface_creation(mobject: Mob, resolution: (usize, usize), axis: usize) -> ShowPartial {
    show_creation(mobject).with_surface_grid(resolution, axis)
}

/// `Uncreate` (creation.py:56): ShowCreation under `smooth(1 - t)`, as a
/// remover, with the (inert) `should_match_start = true`.
#[must_use]
pub fn uncreate(mobject: Mob) -> ShowPartial {
    let mut anim = show_creation(mobject);
    anim.state_mut().config.name = "Uncreate".to_owned();
    anim.state_mut().config.rate_func = RateFunc::Base(smooth_reversed);
    anim.state_mut().config.remover = true;
    anim.should_match_start = true;
    anim
}

/// [`uncreate`] over a sampled Surface grid.
#[must_use]
pub fn uncreate_surface(mobject: Mob, resolution: (usize, usize), axis: usize) -> ShowPartial {
    let mut anim = show_surface_creation(mobject, resolution, axis);
    anim.state_mut().config.name = "Uncreate".to_owned();
    anim.state_mut().config.rate_func = RateFunc::Base(smooth_reversed);
    anim.state_mut().config.remover = true;
    anim.should_match_start = true;
    anim
}

/// `ShowPassingFlash` (indication.py:165): the sliding window, as a
/// remover, restoring the full run at teardown.
#[must_use]
pub fn show_passing_flash(mobject: Mob, time_width: f64) -> ShowPartial {
    let mut anim = ShowPartial::new(mobject, RevealBounds::PassingFlash { time_width });
    anim.state_mut().config.name = "ShowPassingFlash".to_owned();
    anim.state_mut().config.remover = true;
    anim.restore_on_teardown = true;
    anim
}

// ---------------------------------------------------- DrawBorderThenFill

/// `DrawBorderThenFill` (creation.py:75): first half reveals a stroke
/// outline, second half cross-fades outline → start.
#[derive(Debug, Clone)]
pub struct DrawBorderThenFill {
    state: AnimState,
    outline: Option<Mob>,
    stroke_width: f64,
    stroke_color: Option<[f32; 3]>,
    crossed: Vec<Mob>,
}

impl DrawBorderThenFill {
    /// Reference defaults: `run_time = 2.0`, `rate_func = double_smooth`,
    /// `stroke_width = 2.0`, stroke color per-submobject.
    #[must_use]
    pub fn new(vmobject: Mob) -> Self {
        let config = AnimConfig {
            name: "DrawBorderThenFill".to_owned(),
            run_time: 2.0,
            rate_func: RateFunc::Base(rate::double_smooth),
            ..AnimConfig::default()
        };
        Self {
            state: AnimState::new(vmobject, config),
            outline: None,
            stroke_width: 2.0,
            stroke_color: None,
            crossed: Vec::new(),
        }
    }

    /// Outline stroke width (Reference default `2.0`).
    #[must_use]
    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Outline stroke color; `None` keeps each submobject's own
    /// (`stroke_color or sm.get_stroke_color()`, creation.py:113).
    #[must_use]
    pub fn with_stroke_color(mut self, rgb: Option<[f32; 3]>) -> Self {
        self.stroke_color = rgb;
        self
    }

    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    /// The outline copy (test hook; `None` before `begin`).
    #[must_use]
    pub fn outline(&self) -> Option<Mob> {
        self.outline
    }

    /// creation.py:108 `get_outline`: a family copy with fill opacity 0
    /// and the configured stroke on every member with points.
    fn make_outline(&self, stage: &mut Stage) -> Result<Mob, AnimError> {
        let mobject = self.state.mobject();
        let outline = stage.copy_family(mobject)?;
        let behind = stage
            .get(mobject)
            .is_some_and(|e| e.uniforms().stroke_behind);
        for member in stage.family(outline) {
            let Some(entry) = stage.get_mut(member) else {
                continue;
            };
            // set_fill(opacity=0), whole family.
            if let Some(mut fill) = entry.buffer.read_column("fill_rgba") {
                for alpha in fill.iter_mut().skip(3).step_by(4) {
                    *alpha = 0.0;
                }
                entry.buffer.write_range("fill_rgba", 0, &fill);
            }
            // set_stroke(color?, width, behind), members with points only.
            if entry.buffer.is_empty() {
                continue;
            }
            if let Some(rgb) = self.stroke_color
                && let Some(mut stroke) = entry.buffer.read_column("stroke_rgba")
            {
                for row in stroke.as_chunks_mut::<4>().0 {
                    row[..3].copy_from_slice(&rgb);
                }
                entry.buffer.write_range("stroke_rgba", 0, &stroke);
            }
            if let Some(widths) = entry.buffer.read_column("stroke_width") {
                #[allow(clippy::cast_possible_truncation)]
                let flat = vec![self.stroke_width as f32; widths.len()];
                entry.buffer.write_range("stroke_width", 0, &flat);
            }
            entry.uniforms_mut().stroke_behind = behind;
        }
        Ok(outline)
    }
}

/// mobject.py match-style surface: the record columns `match_style`
/// rewrites through `set_style(**other.get_style())`.
const STYLE_FIELDS: [&str; 4] = [
    "fill_rgba",
    "fill_border_width",
    "stroke_rgba",
    "stroke_width",
];

/// `match_style` over two aligned families (vectorized_mobject.py:275):
/// copy the style columns and style uniforms member-for-member. The
/// families are structurally identical here (one is a copy of the other),
/// so the Reference's best-effort submobject pairing is exact pairing.
/// Shared with the indication family (`VShowPassingFlash.finish`).
pub(crate) fn match_style_from(stage: &mut Stage, mobject: Mob, source: Mob) {
    let fa = stage.family(mobject);
    let fb = stage.family(source);
    for (&dst, &src) in fa.iter().zip(fb.iter()) {
        let columns: Vec<(String, Vec<f32>)> = match stage.get(src) {
            Some(entry) => STYLE_FIELDS
                .iter()
                .filter_map(|&f| entry.buffer.read_column(f).map(|c| (f.to_owned(), c)))
                .collect(),
            None => continue,
        };
        let (shading, flat, behind) = match stage.get(src) {
            Some(e) => {
                let u = e.uniforms();
                (u.shading, u.flat_stroke, u.stroke_behind)
            }
            None => continue,
        };
        if let Some(entry) = stage.get_mut(dst) {
            for (field, column) in columns {
                if entry.buffer.read_column(&field).map(|c| c.len()) == Some(column.len()) {
                    entry.buffer.write_range(&field, 0, &column);
                }
            }
            let u = entry.uniforms_mut();
            u.shading = shading;
            u.flat_stroke = flat;
            u.stroke_behind = behind;
        }
    }
}

impl Animation for DrawBorderThenFill {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    /// Pure: the first-crossing `set_data` is redundancy elimination — at
    /// any alpha the written bits are a function of (start, outline,
    /// alpha), because the second-half lerp rewrites every field it
    /// touches. Reconstruction at an arbitrary alpha replays identically.
    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }

    /// creation.py:100 `begin`: build the outline before the canonical
    /// sequence runs. (The Reference flips `set_animating_status(True)`
    /// first, so its outline copy carries the animating flag — a
    /// render-cache detail with no record-state consequence here.)
    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        let mobject = self.state.mobject();
        if !stage.contains(mobject) {
            return Err(AnimError::StaleHandle(mobject));
        }
        self.crossed.clear();
        let outline = self.make_outline(stage)?;
        self.outline = Some(outline);
        Ok(())
    }

    /// creation.py:105 — `match_style(outline)` right after
    /// `super().begin()`.
    fn after_begin(&mut self, stage: &mut Stage) {
        if let Some(outline) = self.outline {
            match_style_from(stage, self.state.mobject(), outline);
        }
    }

    /// creation.py:106 `finish` — refresh the family's joint angles.
    fn teardown(&mut self, stage: &mut Stage) {
        let _ = stage.refresh_family_joint_angles(self.state.mobject());
    }

    /// `(mobject, starting_mobject, outline)` (creation.py:118).
    fn all_mobjects(&self) -> Vec<Mob> {
        let mut mobs = vec![self.state.mobject()];
        if let Some(starting) = self.state.starting_mobject() {
            mobs.push(starting);
        }
        if let Some(outline) = self.outline {
            mobs.push(outline);
        }
        mobs
    }

    /// creation.py:122: first half partial-reveals the outline, second
    /// half lerps outline → start, with the first-crossing `set_data`.
    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        let [submob, start, outline] = *mobs else {
            return; // rows are triples once begin has run
        };
        let (index, subalpha) = integer_interpolate(2, sub_alpha);
        if index == 1 && !self.crossed.contains(&submob) {
            // First crossing: submob.set_data(outline.data).
            let source = match stage.get(outline) {
                Some(entry) => entry.buffer.snapshot_clone(),
                None => return,
            };
            if let Some(entry) = stage.get_mut(submob) {
                entry.buffer.assign_from(&source);
            }
            self.crossed.push(submob);
        }
        if index == 0 {
            let _ = stage.pointwise_become_partial(submob, outline, 0.0, subalpha);
        } else {
            interpolate_fields(stage, submob, outline, start, subalpha, PathFunc::Straight);
        }
    }
}

/// `Write` (creation.py:140): DrawBorderThenFill with `run_time` and
/// `lag_ratio` derived from the with-points family size, a linear rate,
/// and the mobject's own color as the outline stroke.
#[must_use]
pub fn write(stage: &Stage, vmobject: Mob) -> DrawBorderThenFill {
    let family_size = stage
        .family(vmobject)
        .iter()
        .filter(|&&m| stage.get(m).is_some_and(|e| !e.buffer.is_empty()))
        .count();
    let run_time = if family_size < 15 { 1.0 } else { 2.0 };
    #[allow(clippy::cast_precision_loss)]
    let lag_ratio = (4.0 / (family_size as f64 + 1.0)).min(0.2);
    // VMobject.get_color(): fill color when there is fill, else stroke.
    let color = stage.get(vmobject).and_then(|entry| {
        let fill = entry.buffer.read_column("fill_rgba");
        let stroke = entry.buffer.read_column("stroke_rgba");
        match (fill, stroke) {
            (Some(f), _) if f.len() >= 4 && f[3] > 0.0 => Some([f[0], f[1], f[2]]),
            (_, Some(s)) if s.len() >= 4 => Some([s[0], s[1], s[2]]),
            _ => None,
        }
    });
    let mut anim = DrawBorderThenFill::new(vmobject).with_stroke_color(color);
    anim.state_mut().config.name = "Write".to_owned();
    anim.state_mut().config.run_time = run_time;
    anim.state_mut().config.lag_ratio = lag_ratio;
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim
}

// ------------------------------------------------- ShowIncreasingSubsets

/// The Reference's `int_func` parameter as data: `np.round` is banker's
/// rounding, `np.ceil` the ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntRound {
    /// `np.round` — ties to even.
    Round,
    /// `np.ceil`.
    Ceil,
}

impl IntRound {
    fn apply(self, x: f64) -> i64 {
        #[allow(clippy::cast_possible_truncation)]
        match self {
            Self::Round => x.round_ties_even() as i64,
            Self::Ceil => x.ceil() as i64,
        }
    }
}

/// `ShowIncreasingSubsets` (creation.py:176) and, via
/// [`show_submobjects_one_by_one`], `ShowSubmobjectsOneByOne`
/// (creation.py:200): the group's child list is rewritten each frame from
/// the construction-time snapshot of its children.
#[derive(Debug, Clone)]
pub struct ShowIncreasingSubsets {
    state: AnimState,
    all_submobs: Vec<Mob>,
    int_round: IntRound,
    one_by_one: bool,
}

impl ShowIncreasingSubsets {
    /// Replace the animation config.
    #[must_use]
    pub fn with_config(mut self, config: AnimConfig) -> Self {
        self.state.config = config;
        self
    }

    /// Override the rounding rule (the Reference's `int_func` knob).
    #[must_use]
    pub fn with_int_round(mut self, int_round: IntRound) -> Self {
        self.int_round = int_round;
        self
    }

    /// creation.py:196 / creation.py:207 `update_submobject_list`.
    fn update_submobject_list(&self, stage: &mut Stage, index: i64) {
        let n = self.all_submobs.len();
        if n == 0 {
            return;
        }
        let desired: Vec<Mob> = if self.one_by_one {
            // index = int(clip(index, 0, n - 1)); 0 → empty, else the
            // single (index-1)-th child.
            #[allow(clippy::cast_sign_loss)]
            let idx = index.clamp(0, n as i64 - 1) as usize;
            if idx == 0 {
                Vec::new()
            } else {
                vec![self.all_submobs[idx - 1]]
            }
        } else {
            // all_submobs[:index] — Python slices clamp.
            #[allow(clippy::cast_sign_loss)]
            let idx = index.clamp(0, n as i64) as usize;
            self.all_submobs[..idx].to_vec()
        };
        let root = self.state.mobject();
        let current = match stage.get(root) {
            Some(entry) => entry.submobjects().to_vec(),
            None => return,
        };
        if current == desired {
            return; // set_submobjects' identity short-circuit
        }
        // One pass for the usual case; a child `attach` would refuse (a
        // stale construction-time child) keeps the per-edge loop, which
        // skips it.
        if stage.replace_children(root, &desired).is_err() {
            for &child in &current {
                stage.detach(root, child);
            }
            for &child in &desired {
                // Re-attaching a construction-time child cannot form a cycle.
                let _ = stage.attach(root, child);
            }
        }
    }
}

impl Animation for ShowIncreasingSubsets {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    // Deliberately Unclassified (the default made explicit): the frame
    // writes are structural (arena child-list rewiring), and until the
    // scene runtime's membership rules land (fm-5xm) the conservative
    // R20 demotion is the correct classification.

    /// The Reference overrides `interpolate_mobject` (creation.py:190):
    /// the rate function applies to the raw alpha, and the lag/time-span
    /// pipeline is bypassed — mirrored by overriding `interpolate` itself.
    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        let rated = self.state.config.rate_func.eval(alpha);
        #[allow(clippy::cast_precision_loss)]
        let index = self.int_round.apply(rated * self.all_submobs.len() as f64);
        self.update_submobject_list(stage, index);
    }

    fn interpolate_submobject(&mut self, _stage: &mut Stage, _mobs: &[Mob], _sub_alpha: f64) {
        // Unreachable: interpolate is overridden and never zips families.
    }
}

/// `ShowIncreasingSubsets` with Reference defaults (`int_func=np.round`,
/// `suspend_mobject_updating=False` — the base default, restated by the
/// Reference).
///
/// # Errors
/// [`AnimError::StaleHandle`] on a dead group handle.
pub fn show_increasing_subsets(
    stage: &Stage,
    group: Mob,
) -> Result<ShowIncreasingSubsets, AnimError> {
    let all_submobs = stage
        .get(group)
        .ok_or(AnimError::StaleHandle(group))?
        .submobjects()
        .to_vec();
    let config = AnimConfig {
        name: "ShowIncreasingSubsets".to_owned(),
        ..AnimConfig::default()
    };
    Ok(ShowIncreasingSubsets {
        state: AnimState::new(group, config),
        all_submobs,
        int_round: IntRound::Round,
        one_by_one: false,
    })
}

/// `ShowSubmobjectsOneByOne` (creation.py:200): ceiling rounding, one
/// visible child at a time.
///
/// # Errors
/// [`AnimError::StaleHandle`] on a dead group handle.
pub fn show_submobjects_one_by_one(
    stage: &Stage,
    group: Mob,
) -> Result<ShowIncreasingSubsets, AnimError> {
    let mut anim = show_increasing_subsets(stage, group)?;
    anim.state.config.name = "ShowSubmobjectsOneByOne".to_owned();
    anim.int_round = IntRound::Ceil;
    anim.one_by_one = true;
    Ok(anim)
}

/// `AddTextWordByWord` (creation.py:210): `ShowIncreasingSubsets` over a
/// grouped parent whose children are the span-map word groups, with the
/// Reference's parameterization — `rate_func=linear`, and a negative
/// `run_time` (the Reference's `-1.0` default sentinel) recomputed as
/// `time_per_word * n_words`.
///
/// Word grouping itself is the caller's: the portal's `StringMobject`
/// surface partitions the glyph children into word groups by their native
/// source spans (`build_groups()`' boundaries) and hands the grouped
/// parent here. This constructor owns the reveal semantics only.
///
/// # Errors
/// [`AnimError::StaleHandle`] on a dead group handle;
/// [`AnimError::NoWordGroups`] when the grouped parent has no children —
/// an empty string mobject has no words to add, and the Reference's
/// `run_time = 0.2 * 0` degenerate play is refused by name instead.
pub fn add_text_word_by_word(
    stage: &Stage,
    grouped: Mob,
    time_per_word: f64,
    run_time: f64,
) -> Result<ShowIncreasingSubsets, AnimError> {
    let mut anim = show_increasing_subsets(stage, grouped)?;
    if anim.all_submobs.is_empty() {
        return Err(AnimError::NoWordGroups);
    }
    anim.state.config.name = "AddTextWordByWord".to_owned();
    anim.state.config.rate_func = RateFunc::linear();
    #[allow(clippy::cast_precision_loss)]
    let run_time = if run_time < 0.0 {
        time_per_word * anim.all_submobs.len() as f64
    } else {
        run_time
    };
    anim.state.config.run_time = run_time;
    Ok(anim)
}

/// `AddTextLetterByLetter` (fm-5wq.4.59): the letter-granularity sibling
/// of [`add_text_word_by_word`] — `ShowIncreasingSubsets` over a grouped
/// parent whose children are the span-map letter groups (one non-whitespace
/// glyph each, in reading order), linear rate, and a negative `run_time`
/// sentinel recomputed as `time_per_char * n_letters`.
///
/// Provenance note: the pinned Reference's `creation.py` has no
/// letter-granularity class — this is compatibility surface for the wider
/// manim ecosystem's spelling, built from the same native span maps and
/// reveal mechanism as the word-granularity Reference class, so the two
/// stay one mechanism with two grain sizes.
///
/// # Errors
/// [`AnimError::StaleHandle`] on a dead group handle;
/// [`AnimError::NoLetterGroups`] when the grouped parent has no children —
/// a string mobject with no glyphs has no letters to add, refused by name
/// instead of playing a degenerate zero-letter reveal.
pub fn add_text_letter_by_letter(
    stage: &Stage,
    grouped: Mob,
    time_per_char: f64,
    run_time: f64,
) -> Result<ShowIncreasingSubsets, AnimError> {
    let mut anim = show_increasing_subsets(stage, grouped)?;
    if anim.all_submobs.is_empty() {
        return Err(AnimError::NoLetterGroups);
    }
    anim.state.config.name = "AddTextLetterByLetter".to_owned();
    anim.state.config.rate_func = RateFunc::linear();
    #[allow(clippy::cast_precision_loss)]
    let run_time = if run_time < 0.0 {
        time_per_char * anim.all_submobs.len() as f64
    } else {
        run_time
    };
    anim.state.config.run_time = run_time;
    Ok(anim)
}
