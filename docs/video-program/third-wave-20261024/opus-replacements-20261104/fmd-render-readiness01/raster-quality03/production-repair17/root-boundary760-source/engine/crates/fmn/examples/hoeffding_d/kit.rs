//! Small authoring conveniences over the `fmn` facade: one place for the
//! palette, text/Tex construction, play macros, live readouts and plots.

use std::rc::Rc;

use fmn::core::color::color_gradient;
use fmn::library::TextMobjectError;
use fmn::prelude::*;

// ------------------------------------------------------------------ palette

pub const BACKGROUND: &str = "#0E1116";

pub fn hex(s: &str) -> Srgb {
    Srgb::from_hex(s).expect("palette literals are valid hex")
}

/// Roles, not hues: every chapter colors the same idea the same way.
pub struct Palette;
impl Palette {
    pub fn ink() -> Srgb {
        hex("#ECEFF4")
    }
    pub fn dim() -> Srgb {
        hex("#8A93A3")
    }
    pub fn faint() -> Srgb {
        hex("#3B4352")
    }
    pub fn x_rank() -> Srgb {
        hex("#58C4DD")
    } // R: 3b1b BLUE_C
    pub fn y_rank() -> Srgb {
        hex("#83C167")
    } // S: 3b1b GREEN_C
    pub fn q() -> Srgb {
        hex("#FFFF00")
    } // Q: YELLOW
    pub fn d1() -> Srgb {
        hex("#5CD0B3")
    } // TEAL_C
    pub fn d2() -> Srgb {
        hex("#F0AC5F")
    } // GOLD_C
    pub fn d3() -> Srgb {
        hex("#FC6255")
    } // RED_C
    pub fn hoeffding() -> Srgb {
        hex("#FFD866")
    }
    pub fn pearson() -> Srgb {
        hex("#9A72AC")
    } // PURPLE_C
    pub fn spearman() -> Srgb {
        hex("#58C4DD")
    }
    pub fn kendall() -> Srgb {
        hex("#E07A5F")
    }
    pub fn warn() -> Srgb {
        hex("#FC6255")
    }
    pub fn good() -> Srgb {
        hex("#83C167")
    }
}

/// A smooth blue → teal → green → gold ramp, one color per point.
pub fn point_colors(n: usize) -> Vec<Srgb> {
    color_gradient(
        &[
            hex("#58C4DD"),
            hex("#5CD0B3"),
            hex("#83C167"),
            hex("#F0AC5F"),
        ],
        n,
    )
}

pub fn ink_style(color: Srgb) -> Style {
    Style::default()
        .fill(color, 1.0)
        .stroke(color, 0.0, 1.0)
        .fill_border_width(0.5)
}

pub fn stroke_style(color: Srgb, width: f64) -> Style {
    Style::default().fill(color, 0.0).stroke(color, width, 1.0)
}

// --------------------------------------------------------------- vec3 math

pub fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// ------------------------------------------------------------------- kit

/// Shared, cheaply clonable resources (the bundled font book).
#[derive(Clone)]
pub struct Kit {
    pub book: Rc<FontBook>,
    /// The procedural score; `None` renders silently (drafts, stills).
    pub score: Option<crate::sound::Score>,
    /// The voice-over; `None` keeps the visual-only pacing.
    pub narrator: Option<Rc<crate::narration::Narrator>>,
}

impl Kit {
    pub fn new() -> fmn::Result<Self> {
        Ok(Self {
            book: Rc::new(FontBook::bundled().map_err(TextMobjectError::Text)?),
            score: None,
            narrator: None,
        })
    }

    /// Speak narration line `id` now, once the previous line has finished.
    pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        match &self.narrator {
            Some(n) => n.say(stage, id),
            None => Ok(()),
        }
    }

    /// Let the current narration line finish before moving on.
    pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        match &self.narrator {
            Some(n) => n.hold(stage),
            None => Ok(()),
        }
    }

    /// Lay this chapter's pad under the whole scene (call last).
    pub fn pad(&self, stage: &mut Stage<'_>, chapter: usize) -> fmn::Result<()> {
        match &self.score {
            Some(score) => score.pad_under(stage, chapter),
            None => Ok(()),
        }
    }

    /// A soft bell now.
    pub fn chime(&self, stage: &mut Stage<'_>, freq: f64) -> fmn::Result<()> {
        match &self.score {
            Some(score) => score.chime(stage, freq),
            None => Ok(()),
        }
    }

    /// Detached text (not yet in the scene) centered at `at`.
    pub fn text(
        &self,
        stage: &mut Stage<'_>,
        s: &str,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let t = Text::new(s)
            .font_size(size)
            .style(ink_style(color))
            .build(&self.book)?;
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        Ok(m)
    }

    /// Detached text with its left edge at `at`.
    pub fn text_left(
        &self,
        stage: &mut Stage<'_>,
        s: &str,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let m = self.text(stage, s, size, color, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }

    /// Detached italic/bold-capable text.
    #[allow(clippy::too_many_arguments)]
    pub fn text_styled(
        &self,
        stage: &mut Stage<'_>,
        s: &str,
        size: f64,
        color: Srgb,
        at: Vec3,
        bold: bool,
        italic: bool,
    ) -> fmn::Result<Mob> {
        let t = Text::new(s)
            .font_size(size)
            .bold(bold)
            .italic(italic)
            .style(ink_style(color))
            .build(&self.book)?;
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        Ok(m)
    }

    /// Detached display-style math centered at `at`.
    pub fn tex(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        t2c: &[(&str, Srgb)],
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let t = Tex::new(src)
            .font_size(size)
            .style(ink_style(color))
            .t2c(t2c)
            .build(stage.tex_engine()?)?;
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        Ok(m)
    }

    /// As [`Kit::tex`], also returning the native span map (layout
    /// provenance) that `TransformMatchingTex` pairs glyphs by.
    pub fn tex_spans(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        t2c: &[(&str, Srgb)],
        at: Vec3,
    ) -> fmn::Result<(Mob, fmn::library::SpanMapData)> {
        let t = Tex::new(src)
            .font_size(size)
            .style(ink_style(color))
            .t2c(t2c)
            .build(stage.tex_engine()?)?;
        let spans = t.span_map();
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        Ok((m, spans))
    }

    /// As [`Kit::tex`], left edge at `at`.
    pub fn tex_left(
        &self,
        stage: &mut Stage<'_>,
        src: &str,
        size: f64,
        color: Srgb,
        t2c: &[(&str, Srgb)],
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let m = self.tex(stage, src, size, color, t2c, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }

    /// A live number: rebuilt every frame from `tracker`, its `edge` pinned
    /// at `at` so digits never wobble as the width changes.
    #[allow(clippy::too_many_arguments)]
    pub fn live_number(
        &self,
        stage: &mut Stage<'_>,
        tracker: Mob,
        places: usize,
        sign: bool,
        size: f64,
        color: Srgb,
        at: Vec3,
        edge: Vec3,
    ) -> fmn::Result<Mob> {
        let book = Rc::clone(&self.book);
        let m = stage.always_redraw(move |s| {
            let value = s.tracker_value(tracker).unwrap_or(0.0);
            let d = DecimalNumber::new(value)
                .num_decimal_places(places)
                .include_sign(sign)
                .font_size(size)
                .color(color)
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, at, edge);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }

    /// A live number computed by `f` from the arena each frame.
    #[allow(clippy::too_many_arguments)]
    pub fn live_value(
        &self,
        stage: &mut Stage<'_>,
        f: impl Fn(&fmn::mobject::Stage) -> f64 + 'static,
        places: usize,
        sign: bool,
        size: f64,
        color: Srgb,
        at: Vec3,
        edge: Vec3,
    ) -> fmn::Result<Mob> {
        let book = Rc::clone(&self.book);
        let m = stage.always_redraw(move |s| {
            let value = f(s);
            let d = DecimalNumber::new(value)
                .num_decimal_places(places)
                .include_sign(sign)
                .font_size(size)
                .color(color)
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, at, edge);
            m
        });
        stage.add_to_scene(m)?;
        Ok(m)
    }
}

/// A horizontal bar whose length follows `tracker` (`unit` scene units per
/// 1.0), growing right from `origin`; negative values grow left.
pub fn live_bar(
    stage: &mut Stage<'_>,
    tracker: Mob,
    origin: Vec3,
    unit: f64,
    height: f64,
    color: Srgb,
) -> fmn::Result<Mob> {
    let m = stage.always_redraw(move |s| {
        let value = s.tracker_value(tracker).unwrap_or(0.0);
        let w = (value.abs() * unit).max(0.004);
        let r = Rectangle::new()
            .width(w)
            .height(height)
            .style(Style::default().fill(color, 0.85).stroke(color, 0.0, 1.0))
            .build()
            .expect("positive extents");
        let m = s.add(r);
        let edge = if value >= 0.0 { LEFT } else { RIGHT };
        s.move_to(m, origin, edge);
        m
    });
    stage.add_to_scene(m)?;
    Ok(m)
}

/// A plain rectangle with its lower-left corner at `dl`.
pub fn rect(dl: Vec3, w: f64, h: f64, style: Style) -> fmn::Result<VMobject> {
    let r = Rectangle::new()
        .width(w.max(1e-4))
        .height(h.max(1e-4))
        .style(style)
        .build()?;
    Ok(r.moved_to([dl[0] + w / 2.0, dl[1] + h / 2.0, 0.0]))
}

// ------------------------------------------------------------------ plots

/// A square data window: maps a data rectangle onto a scene rectangle.
#[derive(Clone, Copy, Debug)]
pub struct Frame2 {
    pub center: Vec3,
    pub width: f64,
    pub height: f64,
    pub x: (f64, f64),
    pub y: (f64, f64),
}

impl Frame2 {
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        let u = (x - self.x.0) / (self.x.1 - self.x.0) - 0.5;
        let w = (y - self.y.0) / (self.y.1 - self.y.0) - 0.5;
        [
            self.center[0] + u * self.width,
            self.center[1] + w * self.height,
            0.0,
        ]
    }

    /// Inverse of [`Frame2::p`].
    pub fn data(&self, p: Vec3) -> (f64, f64) {
        let u = (p[0] - self.center[0]) / self.width + 0.5;
        let w = (p[1] - self.center[1]) / self.height + 0.5;
        (lerp(self.x.0, self.x.1, u), lerp(self.y.0, self.y.1, w))
    }

    pub fn left(&self) -> f64 {
        self.center[0] - self.width / 2.0
    }
    pub fn right(&self) -> f64 {
        self.center[0] + self.width / 2.0
    }
    pub fn bottom(&self) -> f64 {
        self.center[1] - self.height / 2.0
    }
    pub fn top(&self) -> f64 {
        self.center[1] + self.height / 2.0
    }

    /// The window's border.
    pub fn border(&self, color: Srgb) -> fmn::Result<VMobject> {
        let r = Rectangle::new()
            .width(self.width)
            .height(self.height)
            .style(stroke_style(color, 2.0))
            .build()?;
        Ok(r.moved_to(self.center))
    }
}

/// Dots (one per point) as a single group; colors per point.
pub fn dot_cloud(points: &[Vec3], colors: &[Srgb], radius: f64) -> VMobject {
    v_group(
        points
            .iter()
            .zip(colors)
            .map(|(&p, &c)| Dot::new().point(p).radius(radius).color(c).build()),
    )
}

/// The children of a group, in order.
pub fn children(stage: &Stage<'_>, group: Mob) -> Vec<Mob> {
    stage
        .get(group)
        .map(|e| e.submobjects().to_vec())
        .unwrap_or_default()
}

// --------------------------------------------------------------- playback

/// Set run time / lag on any animation, fluently.
pub trait Timed: Sized {
    fn rt(self, run_time: f64) -> Self;
    fn lag(self, lag_ratio: f64) -> Self;
}

impl<A: Animation> Timed for A {
    fn rt(mut self, run_time: f64) -> Self {
        self.update_rate_info(Some(run_time), None, None);
        self
    }
    fn lag(mut self, lag_ratio: f64) -> Self {
        self.update_rate_info(None, None, Some(lag_ratio));
        self
    }
}

/// `play!(stage; a, b, c)` — simultaneous animations, each built before it
/// is prepared so builders may borrow the stage. `play!(stage, 2.0; ...)`
/// overrides every member's run time.
#[macro_export]
macro_rules! play {
    ($stage:ident; $($a:expr),+ $(,)?) => {{
        let mut v: Vec<Box<dyn fmn::prelude::Animation>> = Vec::new();
        $( { let anim = $a; v.push($stage.prepare(anim)?); } )+
        $stage.play_prepared(v)?;
    }};
    ($stage:ident, $rt:expr; $($a:expr),+ $(,)?) => {{
        let mut v: Vec<Box<dyn fmn::prelude::Animation>> = Vec::new();
        $( { let anim = $a; v.push($stage.prepare(anim)?); } )+
        $stage.play_prepared_with(v, fmn::prelude::PlayOverrides {
            run_time: Some($rt),
            ..fmn::prelude::PlayOverrides::default()
        })?;
    }};
}

/// Prepare a list of animations into a lagged group.
pub fn lagged(
    stage: &mut Stage<'_>,
    anims: Vec<Box<dyn Animation>>,
    lag: f64,
) -> fmn::Result<AnimationGroup> {
    Ok(AnimationGroup::with_lag_ratio(
        stage.arena_mut(),
        anims,
        lag,
    )?)
}

/// Animate a value tracker to `value`.
pub fn tween(tracker: Mob, value: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}

/// Fade every top-level mobject out together (updaters stopped first so
/// live readouts fade instead of being redrawn at full opacity).
pub fn fade_all(stage: &mut Stage<'_>, run_time: f64) -> fmn::Result<()> {
    let mobs: Vec<Mob> = stage.scene().mobjects().to_vec();
    if mobs.is_empty() {
        return Ok(());
    }
    let mut v: Vec<Box<dyn Animation>> = Vec::new();
    for m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        v.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        v,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}
