//! A stand-in for the part of fmn that chapter 11 calls. It draws nothing. It
//! lets bare rustc type-check the scene code, and it runs the scene to record
//! what a render would do to each object: the clock, double fades, moves of
//! hidden objects, and rough text boxes. Its signatures were copied by hand from
//! reading fmn and the Kit, so a pass here is not a pass against the real crates.
use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

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
        pub x: f64,
        pub y: f64,
        pub w: f64,
        pub h: f64,
        pub text: Option<String>,
    }

    #[derive(Default)]
    pub struct Book {
        pub mobs: Vec<Record>,
        pub now: f64,
        pub place: String,
        pub problems: Vec<String>,
        pub plays: usize,
    }

    thread_local! {
        pub static BOOK: RefCell<Book> = RefCell::new(Book::default());
    }

    pub fn with<R>(f: impl FnOnce(&mut Book) -> R) -> R {
        BOOK.with(|book| f(&mut book.borrow_mut()))
    }

    pub fn add(x: f64, y: f64, w: f64, h: f64, text: Option<String>) -> Mob {
        with(|book| {
            book.mobs.push(Record {
                life: Life::Hidden,
                x,
                y,
                w,
                h,
                text,
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

    pub fn name(book: &Book, mob: Mob) -> String {
        let record = &book.mobs[mob.0];
        match &record.text {
            Some(text) => format!("{text:?}"),
            None => format!("object {} at ({:.2}, {:.2})", mob.0, record.x, record.y),
        }
    }

    pub fn apply(effect: &Effect) {
        match *effect {
            Effect::Show(mob) => {
                let life = with(|book| book.mobs[mob.0].life);
                if life != Life::Hidden {
                    let what = with(|book| name(book, mob));
                    problem(format!("fade in of {what} while {life:?}"));
                }
                with(|book| book.mobs[mob.0].life = Life::Shown);
            }
            Effect::Hide(mob) => {
                let life = with(|book| book.mobs[mob.0].life);
                if life != Life::Shown {
                    let what = with(|book| name(book, mob));
                    problem(format!("fade out of {what} while {life:?}"));
                }
                with(|book| book.mobs[mob.0].life = Life::Gone);
            }
            Effect::Move(mob, x, y) => {
                touch(mob, "move");
                with(|book| {
                    book.mobs[mob.0].x = x;
                    book.mobs[mob.0].y = y;
                });
            }
            Effect::Touch(mob) => touch(mob, "pulse or recolour"),
        }
    }

    fn touch(mob: Mob, verb: &str) {
        let life = with(|book| book.mobs[mob.0].life);
        if life != Life::Shown {
            let what = with(|book| name(book, mob));
            problem(format!("{verb} of {what} while {life:?}"));
        }
    }

    pub fn play(animations: &[&dyn Animation]) {
        if animations.is_empty() {
            problem("a play with no animations".to_owned());
        }
        let mut longest: f64 = 0.0;
        for animation in animations {
            animation.apply();
            longest = longest.max(animation.run_time());
        }
        with(|book| book.plays += 1);
        advance(longest);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
}

pub const ORIGIN: Vec3 = Vec3 { x: 0.0, y: 0.0 };

impl std::ops::Index<usize> for Vec3 {
    type Output = f64;
    fn index(&self, axis: usize) -> &f64 {
        match axis {
            0 => &self.x,
            1 => &self.y,
            _ => &0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Srgb;

/// Not a unit struct, so Clippy treats `Style::default()` as it does for the real one.
#[derive(Clone, Copy, Debug, Default)]
pub struct Style {
    _width: u8,
}

impl Style {
    pub fn fill(self, _: Srgb, _: f64) -> Self {
        self
    }
    pub fn stroke(self, _: Srgb, _: f64, _: f64) -> Self {
        self
    }
    pub fn fill_border_width(self, _: f64) -> Self {
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mob(pub usize);

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    Show(Mob),
    Hide(Mob),
    Move(Mob, f64, f64),
    Touch(Mob),
}

pub trait Animation {
    fn run_time(&self) -> f64;
    fn apply(&self);
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
    fn apply(&self) {
        for effect in &self.effects {
            stub::apply(effect);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimateArgs {
    pub run_time: Option<f64>,
    pub path_arc: Option<f64>,
    /// The real struct has more fields than the chapter sets.
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
            effects: vec![Effect::Move(self.mob, to.x, to.y)],
        })
    }
    pub fn set_color(self, _: Srgb) -> Result<Anim> {
        Ok(Anim {
            run_time: self.run_time,
            effects: vec![Effect::Touch(self.mob)],
        })
    }
}

pub struct Shape {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Shape {
    pub fn moved_to(self, to: Vec3) -> Self {
        Shape {
            x: to.x,
            y: to.y,
            ..self
        }
    }
}

#[derive(Default)]
pub struct Circle {
    radius: f64,
}

impl Circle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn radius(self, radius: f64) -> Self {
        Circle { radius }
    }
    pub fn style(self, _: Style) -> Self {
        self
    }
    pub fn build(self) -> Shape {
        Shape {
            x: 0.0,
            y: 0.0,
            w: 2.0 * self.radius,
            h: 2.0 * self.radius,
        }
    }
}

#[derive(Default)]
pub struct Rectangle {
    width: f64,
    height: f64,
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
    pub fn corner_radius(self, _: f64) -> Self {
        self
    }
    pub fn style(self, _: Style) -> Self {
        self
    }
    pub fn build(self) -> Result<Shape> {
        Ok(Shape {
            x: 0.0,
            y: 0.0,
            w: self.width,
            h: self.height,
        })
    }
}

pub struct Arrow;

impl Arrow {
    pub fn new(_: Vec3, _: Vec3) -> Self {
        Arrow
    }
    pub fn path_arc(self, _: f64) -> Self {
        self
    }
    pub fn buff(self, _: f64) -> Self {
        self
    }
    pub fn thickness(self, _: f64) -> Self {
        self
    }
    pub fn style(self, _: Style) -> Self {
        self
    }
    pub fn build(self) -> Result<Shape> {
        Ok(Shape {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        })
    }
}

pub struct Line;

impl Line {
    pub fn new(_: Vec3, _: Vec3) -> Self {
        Line
    }
    pub fn style(self, _: Style) -> Self {
        self
    }
    pub fn build(self) -> Result<Shape> {
        Ok(Shape {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        })
    }
}

pub struct Polygon;

impl Polygon {
    pub fn new(_: impl IntoIterator<Item = Vec3>) -> Self {
        Polygon
    }
    pub fn style(self, _: Style) -> Self {
        self
    }
    pub fn build(self) -> Shape {
        Shape {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }
}

pub struct Arena;

impl Arena {
    pub fn add(&mut self, shape: Shape) -> Mob {
        stub::add(shape.x, shape.y, shape.w, shape.h, None)
    }
    pub fn set_z_index(&mut self, _: Mob, _: i32, _: bool) {}
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
    pub fn prepare<A: Animation + 'static>(&mut self, animation: A) -> Result<Box<dyn Animation>> {
        Ok(Box::new(animation))
    }
    pub fn play_prepared(&mut self, animations: Vec<Box<dyn Animation>>) -> Result<Report> {
        let parts: Vec<&dyn Animation> = animations.iter().map(|boxed| boxed.as_ref()).collect();
        stub::play(&parts);
        Ok(Report)
    }
    pub fn wait(&mut self, seconds: f64) -> Result<Report> {
        stub::advance(seconds);
        Ok(Report)
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

pub mod animation {
    use super::*;

    pub fn indicate(_: &mut Arena, mob: Mob, _: f64, _: Option<Srgb>) -> Result<Anim> {
        Ok(Anim {
            run_time: 1.0,
            effects: vec![Effect::Touch(mob)],
        })
    }

    pub fn replacement_transform(from: Mob, to: Mob) -> Anim {
        Anim {
            run_time: 1.0,
            effects: vec![Effect::Hide(from), Effect::Show(to)],
        }
    }
}

pub mod prelude {
    pub use super::{
        AnimateArgs, Animation, Arrow, Circle, Line, Mob, ORIGIN, Polygon, Rectangle, SceneConstruct, Srgb, Stage,
        Style, Vec3, fade_in, fade_out,
    };
}
