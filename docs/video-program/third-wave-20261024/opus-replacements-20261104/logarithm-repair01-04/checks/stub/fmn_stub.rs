//! A stand-in for the part of fmn that chapters 01 to 04 and draw.rs call. It draws
//! nothing. It lets bare rustc type-check the scene code, and it runs each scene to
//! record what a render would do to each object: the clock, double fades, moves of
//! hidden objects, the geometry of every stroke, and rough text boxes. Its signatures
//! were copied by hand from reading fmn and the Kit, so a pass here is not a pass
//! against the real crates. Strokes get round caps of half their width, at 0.01 scene
//! units per width unit, as `fmn-render/src/stroke.rs` says at the pin.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;

pub mod scene {
    #[derive(Debug)]
    pub enum SceneError {
        InvalidConfig(&'static str),
    }
}

#[derive(Debug)]
pub enum Error {
    Scene(scene::SceneError),
    Stub(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub type Vec3 = [f64; 3];
pub const ORIGIN: Vec3 = [0.0, 0.0, 0.0];
pub const LEFT: Vec3 = [-1.0, 0.0, 0.0];
pub const RIGHT: Vec3 = [1.0, 0.0, 0.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgb(pub &'static str);

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub fill: Srgb,
    pub fill_opacity: f64,
    pub stroke: Srgb,
    pub width: f64,
    pub stroke_opacity: f64,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fill: Srgb("none"),
            fill_opacity: 0.0,
            stroke: Srgb("none"),
            width: 0.0,
            stroke_opacity: 0.0,
        }
    }
}

impl Style {
    pub fn fill(self, c: Srgb, o: f64) -> Self {
        Style {
            fill: c,
            fill_opacity: o,
            ..self
        }
    }
    pub fn stroke(self, c: Srgb, w: f64, o: f64) -> Self {
        Style {
            stroke: c,
            width: w,
            stroke_opacity: o,
            ..self
        }
    }
}

/// What a builder hands to `Arena::add`.
#[derive(Clone, Debug, Default)]
pub struct Shape {
    pub pts: Vec<[f64; 2]>,
    pub closed: bool,
    pub style: Style,
    pub text: Option<String>,
    /// Text only: centre x, centre y, width, height.
    pub boxed: Option<[f64; 4]>,
    pub tracker: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mob(pub usize);

pub mod stub {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Life {
        Hidden,
        Shown,
        Gone,
    }

    #[derive(Clone, Debug)]
    pub struct Record {
        pub life: Life,
        pub shape: Shape,
        pub parent: Option<Mob>,
        pub children: Vec<Mob>,
    }

    pub type Redraw = Rc<RefCell<dyn FnMut(&mut Arena) -> Mob>>;

    #[derive(Default)]
    pub struct Book {
        pub mobs: Vec<Record>,
        pub lives: Vec<(Mob, Redraw)>,
        pub now: f64,
        pub place: String,
        pub problems: Vec<String>,
        pub plays: usize,
        /// Pairs meeting now, and how long each pair has met in total.
        pub meeting: Vec<String>,
        pub met: BTreeMap<String, f64>,
        pub last_lint: f64,
        pub snapshots: Vec<String>,
    }

    thread_local! {
        pub static BOOK: RefCell<Book> = RefCell::new(Book::default());
    }

    pub fn with<R>(f: impl FnOnce(&mut Book) -> R) -> R {
        BOOK.with(|book| f(&mut book.borrow_mut()))
    }

    pub fn reset() {
        with(|book| *book = Book::default());
    }

    pub fn add(shape: Shape) -> Mob {
        with(|book| {
            book.mobs.push(Record {
                life: Life::Hidden,
                shape,
                parent: None,
                children: Vec::new(),
            });
            Mob(book.mobs.len() - 1)
        })
    }

    pub fn now() -> f64 {
        with(|book| book.now)
    }

    pub fn advance(seconds: f64) {
        with(|book| book.now += seconds);
    }

    pub fn set_place(place: &str) {
        with(|book| book.place = place.to_owned());
    }

    pub fn problem(message: String) {
        with(|book| {
            let line = format!("{} at {:.1}s: {message}", book.place, book.now);
            book.problems.push(line);
        });
    }

    pub fn tree(book: &Book, mob: Mob) -> Vec<Mob> {
        let mut all = vec![mob];
        let mut k = 0;
        while k < all.len() {
            all.extend(book.mobs[all[k].0].children.iter().copied());
            k += 1;
        }
        all
    }

    fn name(book: &Book, mob: Mob) -> String {
        let record = &book.mobs[mob.0];
        match &record.shape.text {
            Some(text) => format!("{text:?}"),
            None => {
                let at = record.shape.pts.first().copied().unwrap_or([f64::NAN, f64::NAN]);
                format!("{} shape {} at ({:.2}, {:.2})", record.shape.style.stroke.0, mob.0, at[0], at[1])
            }
        }
    }

    pub fn set_life(mob: Mob, life: Life) {
        with(|book| {
            for m in tree(book, mob) {
                book.mobs[m.0].life = life;
            }
        });
    }

    pub fn shift(mob: Mob, dx: f64, dy: f64) {
        with(|book| {
            for m in tree(book, mob) {
                let shape = &mut book.mobs[m.0].shape;
                for point in &mut shape.pts {
                    point[0] += dx;
                    point[1] += dy;
                }
                if let Some(boxed) = &mut shape.boxed {
                    boxed[0] += dx;
                    boxed[1] += dy;
                }
            }
        });
    }

    /// Left, bottom, right, top of everything under `mob`.
    pub fn bounds(mob: Mob) -> [f64; 4] {
        with(|book| {
            let mut out = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
            for m in tree(book, mob) {
                let shape = &book.mobs[m.0].shape;
                if let Some([x, y, w, h]) = shape.boxed {
                    out[0] = out[0].min(x - w / 2.0);
                    out[1] = out[1].min(y - h / 2.0);
                    out[2] = out[2].max(x + w / 2.0);
                    out[3] = out[3].max(y + h / 2.0);
                }
                for point in &shape.pts {
                    out[0] = out[0].min(point[0]);
                    out[1] = out[1].min(point[1]);
                    out[2] = out[2].max(point[0]);
                    out[3] = out[3].max(point[1]);
                }
            }
            out
        })
    }

    pub fn tracker(mob: Mob) -> Option<f64> {
        with(|book| book.mobs.get(mob.0).and_then(|r| r.shape.tracker))
    }

    pub fn set_tracker(mob: Mob, value: f64) {
        with(|book| book.mobs[mob.0].shape.tracker = Some(value));
    }

    /// Run every live closure: each replaces what its handle shows.
    pub fn redraw_all() {
        let lives: Vec<(Mob, Redraw)> = with(|book| book.lives.clone());
        for (handle, closure) in lives {
            redraw(handle, &closure);
        }
    }

    pub fn redraw(handle: Mob, closure: &Redraw) {
        let mut arena = Arena;
        let made = (closure.borrow_mut())(&mut arena);
        with(|book| {
            let old = std::mem::take(&mut book.mobs[handle.0].children);
            for child in old {
                for m in tree(book, child) {
                    book.mobs[m.0].life = Life::Gone;
                }
            }
            let life = book.mobs[handle.0].life;
            book.mobs[handle.0].children = vec![made];
            book.mobs[made.0].parent = Some(handle);
            for m in tree(book, made) {
                book.mobs[m.0].life = life;
            }
        });
    }

    fn apply_start(effect: &Effect) {
        if let Effect::Show(mob) = *effect {
            let life = with(|book| book.mobs[mob.0].life);
            if life != Life::Hidden {
                let what = with(|book| name(book, mob));
                problem(format!("fade in of {what} while {life:?}"));
            }
            set_life(mob, Life::Shown);
        }
    }

    fn apply_end(effect: &Effect) {
        match *effect {
            Effect::Hide(mob) => {
                let life = with(|book| book.mobs[mob.0].life);
                if life != Life::Shown {
                    let what = with(|book| name(book, mob));
                    problem(format!("fade out of {what} while {life:?}"));
                }
                set_life(mob, Life::Gone);
            }
            Effect::Shift(mob, dx, dy) => {
                touch(mob, "shift");
                shift(mob, dx, dy);
            }
            Effect::MoveTo(mob, to) => {
                touch(mob, "move");
                let b = bounds(mob);
                shift(mob, to[0] - (b[0] + b[2]) / 2.0, to[1] - (b[1] + b[3]) / 2.0);
            }
            _ => {}
        }
    }

    fn touch(mob: Mob, verb: &str) {
        let life = with(|book| book.mobs[mob.0].life);
        if life != Life::Shown {
            let what = with(|book| name(book, mob));
            problem(format!("{verb} of {what} while {life:?}"));
        }
    }

    /// Run animations: fades in begin, trackers run in steps so every live closure
    /// sees the path, then moves land and fades out end.
    pub fn play(effects: &[Effect], run_time: f64) {
        if effects.is_empty() {
            problem("a play with no animations".to_owned());
        }
        for effect in effects {
            apply_start(effect);
        }
        let tracks: Vec<(Mob, f64, f64)> = effects
            .iter()
            .filter_map(|e| match *e {
                Effect::Track(mob, to) => Some((mob, tracker(mob).unwrap_or(0.0), to)),
                _ => None,
            })
            .collect();
        if !tracks.is_empty() {
            for step in 1..=8 {
                let alpha = f64::from(step) / 8.0;
                for &(mob, from, to) in &tracks {
                    set_tracker(mob, from + (to - from) * alpha);
                }
                redraw_all();
                if step < 8 {
                    lint_path();
                }
            }
        }
        for effect in effects {
            apply_end(effect);
        }
        with(|book| book.plays += 1);
        advance(run_time);
        lint();
    }

    /// Everything on screen now, as (text, box) and (strokes).
    struct Scene {
        texts: Vec<(String, [f64; 4], Mob)>,
        strokes: Vec<(String, Vec<[f64; 2]>, f64, Mob)>,
    }

    fn scene() -> Scene {
        with(|book| {
            let mut scene = Scene {
                texts: Vec::new(),
                strokes: Vec::new(),
            };
            for (k, record) in book.mobs.iter().enumerate() {
                if record.life != Life::Shown {
                    continue;
                }
                let shape = &record.shape;
                if let (Some(text), Some(boxed)) = (&shape.text, shape.boxed) {
                    scene.texts.push((text.clone(), boxed, Mob(k)));
                } else if shape.pts.len() >= 2 {
                    let mut pts = shape.pts.clone();
                    if shape.closed {
                        pts.push(shape.pts[0]);
                    }
                    let paint = if shape.style.width > 0.0 && shape.style.stroke_opacity > 0.0 {
                        shape.style.stroke.0
                    } else {
                        shape.style.fill.0
                    };
                    scene.strokes.push((paint.to_owned(), pts, shape.style.width * 0.01 / 2.0, Mob(k)));
                }
            }
            scene
        })
    }

    fn segment_box_distance(a: [f64; 2], b: [f64; 2], boxed: [f64; 4]) -> f64 {
        // Sampled along the segment: the boxes are estimates, so exactness buys nothing.
        let steps = (((b[0] - a[0]).hypot(b[1] - a[1]) / 0.02).ceil() as usize).max(1);
        let mut best = f64::INFINITY;
        for k in 0..=steps {
            let t = k as f64 / steps as f64;
            let (x, y) = (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
            let dx = ((x - boxed[0]).abs() - boxed[2] / 2.0).max(0.0);
            let dy = ((y - boxed[1]).abs() - boxed[3] / 2.0).max(0.0);
            best = best.min(dx.hypot(dy));
        }
        best
    }

    fn meetings() -> Vec<String> {
        let scene = scene();
        let mut found = Vec::new();
        for (i, (text, a, _)) in scene.texts.iter().enumerate() {
            if a[0].abs() + a[2] / 2.0 > 6.86 || a[1].abs() + a[3] / 2.0 > 3.85 {
                found.push(format!("{text:?} outside the safe frame (x {:.2} ± {:.2}, y {:.2})", a[0], a[2] / 2.0, a[1]));
            }
            for (other, b, _) in scene.texts.iter().skip(i + 1) {
                let gap_x = (a[0] - b[0]).abs() - (a[2] + b[2]) / 2.0;
                let gap_y = (a[1] - b[1]).abs() - (a[3] + b[3]) / 2.0;
                if gap_x < 0.02 && gap_y < 0.02 {
                    found.push(format!("{text:?} meets {other:?}"));
                }
            }
            for (paint, pts, half, _) in &scene.strokes {
                let close = pts.windows(2).any(|pair| segment_box_distance(pair[0], pair[1], *a) < half + 0.01);
                if close {
                    found.push(format!("{text:?} meets a {paint} stroke"));
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// Count time for every pair that met since the last look, then look again.
    pub fn lint() {
        let now = now();
        with(|book| {
            let elapsed = now - book.last_lint;
            let meeting = std::mem::take(&mut book.meeting);
            for pair in meeting {
                *book.met.entry(pair).or_insert(0.0) += elapsed;
            }
            book.last_lint = now;
        });
        let found = meetings();
        with(|book| book.meeting = found);
    }

    /// A moment inside a tracker's run: pairs seen only here count as brief crossings.
    fn lint_path() {
        for pair in meetings() {
            with(|book| {
                book.met.entry(format!("(in motion) {pair}")).or_insert(0.0);
            });
        }
    }

    /// One row per visible thing, for checks/state_checks.py.
    pub fn snapshot(cue: &str, sentence: &str) {
        let scene = scene();
        let t = now();
        let mut rows = Vec::new();
        for (text, b, _) in &scene.texts {
            let clean: String = text.chars().map(|c| if c == '\t' || c == '\n' { ' ' } else { c }).collect();
            rows.push(format!("{cue}\t{sentence}\t{t:.2}\ttext\t{}\t{}\t{}\t{}\t{clean}", b[0], b[1], b[2], b[3]));
        }
        for (paint, pts, half, _) in &scene.strokes {
            let joined = pts.iter().map(|p| format!("{:.4},{:.4}", p[0], p[1])).collect::<Vec<_>>().join(";");
            rows.push(format!("{cue}\t{sentence}\t{t:.2}\tstroke\t{paint}\t{half}\t{joined}"));
        }
        with(|book| book.snapshots.extend(rows));
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    Show(Mob),
    Hide(Mob),
    Shift(Mob, f64, f64),
    MoveTo(Mob, Vec3),
    Track(Mob, f64),
}

pub trait Animation {
    fn run_time(&self) -> f64;
    fn effects(&self) -> Vec<Effect>;
}

#[derive(Clone, Debug)]
pub struct Anim {
    pub run_time: f64,
    pub effects: Vec<Effect>,
}

impl Animation for Anim {
    fn run_time(&self) -> f64 {
        self.run_time
    }
    fn effects(&self) -> Vec<Effect> {
        self.effects.clone()
    }
}

pub struct AnimationGroup {
    run_time: f64,
    effects: Vec<Effect>,
}

impl AnimationGroup {
    pub fn with_lag_ratio(_: &mut Arena, parts: Vec<Box<dyn Animation>>, _: f64) -> Result<Self> {
        let run_time = parts.iter().map(|p| p.run_time()).fold(0.0, f64::max);
        let effects = parts.iter().flat_map(|p| p.effects()).collect();
        Ok(AnimationGroup { run_time, effects })
    }
    pub fn with_run_time(self, run_time: f64) -> Self {
        AnimationGroup { run_time, ..self }
    }
}

impl Animation for AnimationGroup {
    fn run_time(&self) -> f64 {
        self.run_time
    }
    fn effects(&self) -> Vec<Effect> {
        self.effects.clone()
    }
}

/// The real struct has more fields than the chapters set.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimateArgs {
    pub run_time: Option<f64>,
    pub path_arc: Option<f64>,
    pub lag_ratio: Option<f64>,
}

pub struct Builder {
    mob: Mob,
    run_time: f64,
}

impl Mob {
    pub fn animate(self) -> Builder {
        Builder {
            mob: self,
            run_time: 1.0,
        }
    }
}

impl Builder {
    pub fn set_anim_args(self, args: AnimateArgs) -> Result<Self> {
        Ok(Builder {
            mob: self.mob,
            run_time: args.run_time.unwrap_or(self.run_time),
        })
    }
    pub fn move_to(self, to: Vec3, _: Vec3) -> Result<Anim> {
        Ok(Anim {
            run_time: self.run_time,
            effects: vec![Effect::MoveTo(self.mob, to)],
        })
    }
    pub fn shift(self, by: Vec3) -> Result<Anim> {
        Ok(Anim {
            run_time: self.run_time,
            effects: vec![Effect::Shift(self.mob, by[0], by[1])],
        })
    }
}

pub struct PathBuilder {
    pts: Vec<[f64; 2]>,
    closed: bool,
    style: Style,
}

impl PathBuilder {
    pub fn style(self, style: Style) -> Self {
        PathBuilder { style, ..self }
    }
    pub fn build(self) -> Shape {
        Shape {
            pts: self.pts,
            closed: self.closed,
            style: self.style,
            ..Shape::default()
        }
    }
}

pub fn polyline(pts: Vec<Vec3>) -> PathBuilder {
    PathBuilder {
        pts: pts.iter().map(|p| [p[0], p[1]]).collect(),
        closed: false,
        style: Style::default(),
    }
}

pub struct Polygon;

impl Polygon {
    pub fn new<const N: usize>(pts: [Vec3; N]) -> PathBuilder {
        PathBuilder {
            pts: pts.iter().map(|p| [p[0], p[1]]).collect(),
            closed: true,
            style: Style::default(),
        }
    }
}

fn circle_points(centre: Vec3, r: f64) -> Vec<[f64; 2]> {
    (0..32)
        .map(|k| {
            let a = std::f64::consts::TAU * f64::from(k) / 32.0;
            [centre[0] + r * a.cos(), centre[1] + r * a.sin()]
        })
        .collect()
}

#[derive(Default)]
pub struct Circle {
    radius: f64,
    at: Vec3,
    style: Style,
}

impl Circle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn radius(self, radius: f64) -> Self {
        Circle { radius, ..self }
    }
    pub fn arc_center(self, at: Vec3) -> Self {
        Circle { at, ..self }
    }
    pub fn style(self, style: Style) -> Self {
        Circle { style, ..self }
    }
    pub fn build(self) -> Shape {
        Shape {
            pts: circle_points(self.at, self.radius),
            closed: true,
            style: self.style,
            ..Shape::default()
        }
    }
}

#[derive(Default)]
pub struct Dot {
    at: Vec3,
    radius: f64,
    style: Style,
}

impl Dot {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn point(self, at: Vec3) -> Self {
        Dot { at, ..self }
    }
    pub fn radius(self, radius: f64) -> Self {
        Dot { radius, ..self }
    }
    pub fn style(self, style: Style) -> Self {
        Dot { style, ..self }
    }
    pub fn build(self) -> Shape {
        Shape {
            pts: circle_points(self.at, self.radius),
            closed: true,
            style: self.style,
            ..Shape::default()
        }
    }
}

#[derive(Default)]
pub struct Rectangle {
    width: f64,
    height: f64,
    style: Style,
}

impl Rectangle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn width(self, width: f64) -> Self {
        Rectangle { width, ..self }
    }
    pub fn height(self, height: f64) -> Self {
        Rectangle { height, ..self }
    }
    pub fn style(self, style: Style) -> Self {
        Rectangle { style, ..self }
    }
    pub fn build(self) -> Result<Shape> {
        let (w, h) = (self.width / 2.0, self.height / 2.0);
        Ok(Shape {
            pts: vec![[-w, -h], [w, -h], [w, h], [-w, h]],
            closed: true,
            style: self.style,
            ..Shape::default()
        })
    }
}

pub struct Arrow {
    a: Vec3,
    b: Vec3,
    style: Style,
}

impl Arrow {
    pub fn new(a: Vec3, b: Vec3) -> Self {
        Arrow {
            a,
            b,
            style: Style::default(),
        }
    }
    pub fn buff(self, _: f64) -> Self {
        self
    }
    pub fn style(self, style: Style) -> Self {
        Arrow { style, ..self }
    }
    pub fn build(self) -> Result<Shape> {
        Ok(Shape {
            pts: vec![[self.a[0], self.a[1]], [self.b[0], self.b[1]]],
            style: self.style,
            ..Shape::default()
        })
    }
}

pub struct VMobject;

impl VMobject {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Shape {
        Shape::default()
    }
}

/// The arena: what `always_redraw` closures see.
pub struct Arena;

impl Arena {
    pub fn add(&mut self, shape: Shape) -> Mob {
        stub::add(shape)
    }
    pub fn attach(&mut self, parent: Mob, child: Mob) -> Result<()> {
        stub::with(|book| {
            book.mobs[parent.0].children.push(child);
            book.mobs[child.0].parent = Some(parent);
        });
        Ok(())
    }
    pub fn tracker_value(&self, tracker: Mob) -> Option<f64> {
        stub::tracker(tracker)
    }
    pub fn set_tracker_value(&mut self, tracker: Mob, value: f64) -> Result<()> {
        stub::set_tracker(tracker, value);
        Ok(())
    }
    pub fn move_to(&mut self, mob: Mob, at: Vec3, edge: Vec3) {
        place(mob, at, edge);
    }
}

fn place(mob: Mob, at: Vec3, edge: Vec3) {
    let b = stub::bounds(mob);
    let x = (b[0] + b[2]) / 2.0 + edge[0] * (b[2] - b[0]) / 2.0;
    let y = (b[1] + b[3]) / 2.0 + edge[1] * (b[3] - b[1]) / 2.0;
    stub::shift(mob, at[0] - x, at[1] - y);
}

pub mod mobject {
    pub use super::Arena as Stage;
}

pub struct Report;

#[derive(Clone, Copy)]
pub struct Time;

impl Time {
    pub fn to_f64(self) -> f64 {
        stub::now()
    }
    pub fn fps(self) -> u32 {
        60
    }
}

pub struct Scene;

impl Scene {
    pub fn time(&self) -> Time {
        Time
    }
}

pub struct Stage<'a> {
    arena: Arena,
    scene: Scene,
    marker: PhantomData<&'a ()>,
}

impl Stage<'_> {
    pub fn new() -> Self {
        Stage {
            arena: Arena,
            scene: Scene,
            marker: PhantomData,
        }
    }
    pub fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn attach(&mut self, parent: Mob, child: Mob) -> Result<()> {
        self.arena.attach(parent, child)
    }
    pub fn move_to(&mut self, mob: Mob, at: Vec3, edge: Vec3) {
        place(mob, at, edge);
    }
    pub fn add_value_tracker(&mut self, value: f64) -> Mob {
        stub::add(Shape {
            tracker: Some(value),
            ..Shape::default()
        })
    }
    pub fn add_to_scene(&mut self, mob: Mob) -> Result<()> {
        stub::set_life(mob, stub::Life::Shown);
        stub::lint();
        Ok(())
    }
    pub fn always_redraw<F>(&mut self, f: F) -> Mob
    where
        F: FnMut(&mut Arena) -> Mob + 'static,
    {
        let handle = stub::add(Shape::default());
        let closure: stub::Redraw = Rc::new(RefCell::new(f));
        stub::redraw(handle, &closure);
        stub::with(|book| book.lives.push((handle, closure)));
        handle
    }
    pub fn add_dt_updater<F>(&mut self, _: Mob, mut f: F, _: bool) -> Result<()>
    where
        F: FnMut(&mut Arena, Mob, f64) + 'static,
    {
        f(&mut self.arena, Mob(0), 0.0);
        Ok(())
    }
    pub fn clear_updaters(&mut self, mob: Mob, _: bool) {
        stub::with(|book| book.lives.retain(|(handle, _)| *handle != mob));
    }
    pub fn prepare<A: Animation + 'static>(&mut self, animation: A) -> Result<Box<dyn Animation>> {
        Ok(Box::new(animation))
    }
    pub fn play_prepared(&mut self, animations: Vec<Box<dyn Animation>>) -> Result<Report> {
        let run_time = animations.iter().map(|a| a.run_time()).fold(0.0, f64::max);
        let effects: Vec<Effect> = animations.iter().flat_map(|a| a.effects()).collect();
        stub::play(&effects, run_time);
        Ok(Report)
    }
    pub fn play<A: Animation + 'static>(&mut self, animation: A) -> Result<Report> {
        stub::play(&animation.effects(), animation.run_time());
        Ok(Report)
    }
    pub fn wait(&mut self, seconds: f64) -> Result<Report> {
        stub::advance(seconds);
        Ok(Report)
    }
    pub fn get_center(&self, mob: Mob) -> Vec3 {
        let b = stub::bounds(mob);
        [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, 0.0]
    }
}

impl Default for Stage<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait SceneConstruct {
    fn name(&self) -> &str;
    fn construct(&mut self, stage: &mut Stage<'_>) -> Result<()>;
}

pub fn fade_in(_: &mut Arena, mob: Mob, _: Vec3, _: f64) -> Result<Anim> {
    Ok(Anim {
        run_time: 1.0,
        effects: vec![Effect::Show(mob)],
    })
}

pub fn fade_out(_: &mut Arena, mob: Mob, _: Vec3, _: f64) -> Result<Anim> {
    Ok(Anim {
        run_time: 1.0,
        effects: vec![Effect::Hide(mob)],
    })
}

pub mod prelude {
    pub use super::{
        AnimateArgs, Animation, AnimationGroup, Arrow, Circle, Dot, LEFT, Mob, ORIGIN, Polygon, RIGHT, Rectangle,
        SceneConstruct, Srgb, Stage, Style, VMobject, Vec3, fade_in, fade_out, polyline,
    };
}
