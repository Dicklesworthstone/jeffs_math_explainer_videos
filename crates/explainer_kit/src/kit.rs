//! The scene kit: one bundled font book, text and math built, placed and
//! registered in one call, and live readouts.
//!
//! Every text-like mobject the kit builds is recorded with its source string
//! ([`Kit::registered`]): the layout lint checks those for clipping and
//! collisions, the glyph probe checks their strings for missing glyphs.
//!
//! Engine rules (plan §6.6; engine review F-16, F-17; spikes S-2, S-3, S-8):
//! - `always_redraw` rebuilds, lays out and tessellates its content every
//!   frame, so live readouts are for SMALL things: a number, a bar. Never
//!   redraw a whole diagram every frame. (At the pin the replaced content is
//!   deleted and its arena slot reused; S-8 measured no slowdown over 80 s.
//!   The old leak, fm-92gu, needs a pinned entry, which native scenes never
//!   make.)
//! - Positional operations on a group dedupe its members in O(M²): never
//!   transform a group of more than [`GROUP_TRANSFORM_LIMIT`] members per
//!   frame; animate a few representatives, or split the group.
//! - One flat fill per mobject. Differing per-point fills turn a shape into
//!   a smooth colour field (neighbouring cells bleed, and every pixel pays
//!   for every edge). A grid of coloured cells is per-point *stroke* colours
//!   on polyline-tagged strips (S-3), or one flat mobject per colour class.
//! - A stroke's cost per pixel grows with its segment count. Under moving
//!   objects (dirty tiles) never draw long or closed strokes: cut them into
//!   short pieces, and give big filled shapes no outline (S-2).

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use explainer_score::timeline::ChapterTimeline;

use fmn::library::TextMobjectError;
use fmn::prelude::*;

use crate::layout::Linter;
use crate::narration::Narrator;
use crate::palette::ink_style;

/// The bundled typewriter face, for code and hex.
pub const MONO_FAMILY: &str = "CM Typewriter";

/// The largest group the kit transforms per frame (plan §6.6, F-17).
pub const GROUP_TRANSFORM_LIMIT: usize = 2_000;

/// Write a generated sound (a sonification) where the narrator and the score
/// can read it: `<temp>/explainer_kit_sounds/<label>_<hash>.wav`, 32-bit
/// float, named by a hash of its samples. It is written once (a file with
/// that name already holds these samples) and atomically, so parallel renders
/// never read half a file.
pub fn sound_file(label: &str, sound: &explainer_score::Stereo) -> std::io::Result<PathBuf> {
    let mut bytes = Vec::with_capacity(sound.len() * 16);
    for (l, r) in sound.l.iter().zip(&sound.r) {
        bytes.extend(l.to_bits().to_le_bytes());
        bytes.extend(r.to_bits().to_le_bytes());
    }
    let name: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    let stem = format!("{name}_{:016x}", explainer_score::rng::fnv1a64(&bytes));
    let dir = std::env::temp_dir().join("explainer_kit_sounds");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{stem}.wav"));
    if !path.exists() {
        let tmp = dir.join(format!("{stem}.{}.tmp", std::process::id()));
        explainer_score::wav::write(&tmp, sound, explainer_score::wav::Format::F32)?;
        std::fs::rename(&tmp, &path)?;
    }
    Ok(path)
}

/// A sonification's gain in a narrated draft with no score stem (dB): a WAV
/// written at the narration's level then sits where the score's listen
/// level would put it (V − 8).
pub const DRAFT_SONIFY_DB: f64 = -8.0;

/// What kind of text-like mobject a registry entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Text,
    Tex,
    /// A live readout, rebuilt every frame from a tracker.
    Live,
}

/// One text-like mobject the kit built.
#[derive(Clone, Debug)]
pub struct Registered {
    pub mob: Mob,
    pub source: String,
    pub kind: TextKind,
}

/// Shared, cheaply clonable resources for one scene.
#[derive(Clone)]
pub struct Kit {
    pub book: Rc<FontBook>,
    registry: Rc<RefCell<Vec<Registered>>>,
    /// The voice-over (real or scratch); `None` keeps visual-only pacing.
    pub narrator: Option<Rc<Narrator>>,
    /// The layout lint (`render --lint`); `None` costs nothing.
    pub lint: Option<Rc<Linter>>,
    /// The chapter's key (`NN_name`), naming its timeline.
    pub chapter: String,
    /// The chapter's music stem, laid under the whole scene by [`Kit::finish`].
    pub stem: Option<PathBuf>,
    /// Where [`Kit::finish`] leaves the chapter's timeline for the CLI.
    pub finished: Rc<RefCell<Option<ChapterTimeline>>>,
}

impl Kit {
    /// A kit on the bundled fonts.
    pub fn new() -> fmn::Result<Self> {
        Ok(Self {
            book: Rc::new(FontBook::bundled().map_err(TextMobjectError::Text)?),
            registry: Rc::new(RefCell::new(Vec::new())),
            narrator: None,
            lint: None,
            chapter: String::new(),
            stem: None,
            finished: Rc::new(RefCell::new(None)),
        })
    }

    /// End the chapter (call it last in `construct`): let the last line
    /// finish, lay the chapter's music stem from t = 0, and leave the
    /// chapter's timeline for the CLI. The stem must be exactly the scene's
    /// length (frames × 48000 / fps samples): a stem scored from another
    /// timeline is refused, never stretched or cut.
    pub fn finish(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.hold(stage)?;
        let fail = |m: String| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(m.into_boxed_str())));
        if let Some(stem) = &self.stem {
            let file = crate::narration::inspect_cue(stem).map_err(|e| fail(e.to_string()))?;
            let t = stage.scene().time();
            let want = explainer_score::units::frame_to_sample(t.frames().max(0) as u64, t.fps()) as usize;
            if file.rate != explainer_score::SAMPLE_RATE || file.frames != want {
                return Err(fail(format!(
                    "{}: {} frames at {} Hz, but chapter {} is {want} samples at 48 kHz: score it from this chapter's timeline",
                    stem.display(),
                    file.frames,
                    file.rate,
                    self.chapter
                )));
            }
            let secs = t.to_f64();
            match &self.narrator {
                Some(n) => n
                    .sound(stage, stem, -secs, Some(0.0))
                    .map_err(|e| fail(e.to_string()))?,
                None => {
                    stage.scene_mut().add_sound(stem.clone(), -secs, Some(0.0), None)?;
                }
            }
        }
        if let Some(n) = &self.narrator {
            *self.finished.borrow_mut() = Some(n.timeline(stage, &self.chapter));
        }
        Ok(())
    }

    /// The same kit with the layout lint on.
    pub fn with_lint(mut self, lint: Rc<Linter>) -> Self {
        self.lint = Some(lint);
        self
    }

    /// Run the layout lint now (if on); `at` names the checkpoint. `say` and
    /// `hold` call it; call it yourself after a long silent animation.
    pub fn lint_here(&self, stage: &Stage<'_>, at: &str) {
        if let Some(l) = &self.lint {
            l.check(stage.arena(), &self.registry.borrow(), at);
        }
    }

    /// The same kit with a narrator.
    pub fn with_narrator(mut self, narrator: Rc<Narrator>) -> Self {
        self.narrator = Some(narrator);
        self
    }

    /// Speak narration line `id` now, once the previous line has finished
    /// (no-op without a narrator). Call it BEFORE the animations that
    /// illustrate the line.
    pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        self.lint_here(stage, &format!("before {id}"));
        match &self.narrator {
            Some(n) => n.say(stage, id),
            None => Ok(()),
        }
    }

    /// Let the current line finish, plus a breath, before moving on; call it
    /// before every scene wipe so the picture never leaves the voice behind.
    pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.lint_here(stage, "hold");
        match &self.narrator {
            Some(n) => n.hold(stage),
            None => Ok(()),
        }
    }

    /// An authored moment the score may answer (see `Narrator::mark`).
    pub fn mark(&self, stage: &Stage<'_>, label: &str) {
        if let Some(n) = &self.narrator {
            n.mark(stage, label);
        }
    }

    /// `seconds` of music on its own after the current line (an authored
    /// gap); without a narrator, simply a wait.
    pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
        match &self.narrator {
            Some(n) => n.music_breath(stage, seconds),
            None => crate::narration::wait_frames(
                stage,
                crate::narration::frames_covering(seconds, stage.scene().time().fps()),
            ),
        }
    }

    /// A listen window for a teaching sonification; without a narrator, a wait.
    pub fn listen(&self, stage: &mut Stage<'_>, seconds: f64, label: &str) -> fmn::Result<()> {
        match &self.narrator {
            Some(n) => n.listen(stage, seconds, label),
            None => crate::narration::wait_frames(
                stage,
                crate::narration::frames_covering(seconds, stage.scene().time().fps()),
            ),
        }
    }

    /// A sonification that teaches, in its own listen window (see
    /// `Narrator::sonify`: the window does not wait; the scene animates
    /// through it). The score stem carries the sound; a narrated draft
    /// without a stem plays it directly, [`DRAFT_SONIFY_DB`] down, so the
    /// draft is heard as it will be. Write the WAV at about the narration's
    /// −23 LUFS. Without a narrator, nothing (the scene's own animations keep
    /// the time).
    pub fn sonify(&self, stage: &mut Stage<'_>, seconds: f64, label: &str, wav: &std::path::Path) -> fmn::Result<()> {
        let Some(n) = &self.narrator else {
            return Ok(());
        };
        n.sonify(stage, seconds, label, wav)?;
        if self.stem.is_none() && n.has_audio() {
            n.sound(stage, wav, 0.0, Some(DRAFT_SONIFY_DB)).map_err(|e| {
                fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
                    format!("sonification {label}: {e}").into_boxed_str(),
                )))
            })?;
        }
        Ok(())
    }

    /// Every text-like mobject built so far, in order.
    pub fn registered(&self) -> Vec<Registered> {
        self.registry.borrow().clone()
    }

    fn note(&self, mob: Mob, source: &str, kind: TextKind) {
        self.registry.borrow_mut().push(Registered {
            mob,
            source: source.to_owned(),
            kind,
        });
    }

    /// Detached text (not yet in the scene) centred at `at`.
    pub fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let t = Text::new(s).font_size(size).style(ink_style(color)).build(&self.book)?;
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        self.note(m, s, TextKind::Text);
        Ok(m)
    }

    /// Detached code or hex (`0x1D`) in the bundled CM Typewriter, centred at
    /// `at`. fmn's TeX has no `\texttt` (spike S-7), so code is Text.
    pub fn mono(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let t = Text::new(s)
            .font(MONO_FAMILY)
            .font_size(size)
            .style(ink_style(color))
            .build(&self.book)?;
        let m = stage.arena_mut().add(t);
        stage.move_to(m, at, ORIGIN);
        self.note(m, s, TextKind::Text);
        Ok(m)
    }

    /// Detached text with its left edge at `at`.
    pub fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let m = self.text(stage, s, size, color, at)?;
        stage.move_to(m, at, LEFT);
        Ok(m)
    }

    /// Detached bold and/or italic text centred at `at`.
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
        self.note(m, s, TextKind::Text);
        Ok(m)
    }

    /// Detached display math centred at `at`; `t2c` colours substrings.
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
        self.note(m, src, TextKind::Tex);
        Ok(m)
    }

    /// As [`Kit::tex`], also returning the span map `TransformMatchingTex`
    /// pairs glyphs by (taken before the Tex moves into the arena).
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
        self.note(m, src, TextKind::Tex);
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

    /// A live number rebuilt every frame from `tracker`, its `edge` pinned at
    /// `at` so the digits never wobble as the width changes. Small readouts
    /// only (see the module doc).
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
        self.live_number_pinned(stage, tracker, places, sign, size, color, at, edge, false)
    }

    /// As [`Kit::live_number`], pinned to the screen while a 3-D camera moves
    /// (engine review E-X3: labels over Video A's landscape, Video B's cube):
    /// every rebuilt number is fixed in frame.
    #[allow(clippy::too_many_arguments)]
    pub fn live_number_hud(
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
        self.live_number_pinned(stage, tracker, places, sign, size, color, at, edge, true)
    }

    #[allow(clippy::too_many_arguments)]
    fn live_number_pinned(
        &self,
        stage: &mut Stage<'_>,
        tracker: Mob,
        places: usize,
        sign: bool,
        size: f64,
        color: Srgb,
        at: Vec3,
        edge: Vec3,
        fixed: bool,
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
            if fixed {
                s.set_fixed_in_frame(m, 1.0, true);
            }
            m
        });
        stage.add_to_scene(m)?;
        self.note(m, "live number", TextKind::Live);
        Ok(m)
    }

    /// Detached text pinned to the screen while a 3-D camera moves (E-X3),
    /// centred at `at`.
    pub fn text_hud(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        let m = self.text(stage, s, size, color, at)?;
        stage.set_fixed_in_frame(m, 1.0, true);
        Ok(m)
    }

    /// A live integer with thousands separators (big counts), centred at `at`.
    pub fn live_count(
        &self,
        stage: &mut Stage<'_>,
        tracker: Mob,
        size: f64,
        color: Srgb,
        at: Vec3,
    ) -> fmn::Result<Mob> {
        let book = Rc::clone(&self.book);
        let m = stage.always_redraw(move |s| {
            let value = s.tracker_value(tracker).unwrap_or(0.0).round();
            let d = DecimalNumber::new(value)
                .num_decimal_places(0)
                .group_with_commas(true)
                .font_size(size)
                .color(color)
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, at, ORIGIN);
            m
        });
        stage.add_to_scene(m)?;
        self.note(m, "live count", TextKind::Live);
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
        self.note(m, "live value", TextKind::Live);
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
