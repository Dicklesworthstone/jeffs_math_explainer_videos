//! The `.animate` prototype: deferred-command recording (§15.1's lean).
//!
//! `mob.animate()` returns a recorder; chained calls append commands; the
//! recording is inert until `stage.play(...)` consumes it. This mirrors the
//! Reference's builder semantics — `.animate` records against the target
//! and the play call realizes the transition — while staying borrow-checker
//! clean: the recorder owns only a `Copy` handle, never a borrow of the
//! stage, which is exactly why the arena + handle model makes the fluent
//! surface work.
//!
//! The spike applies commands instantly (begin/end interpolation is
//! Choreo's, fm-wuq and friends); what is being ratified is the *shape*.

use crate::stage::{Mob, Stage};

/// A recorded mutation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// Translate every point.
    Shift([f64; 3]),
    /// Rotate every point about the origin in the xy-plane.
    Rotate(f64),
    /// Set the alpha lane of every record.
    SetOpacity(f32),
}

/// The deferred-command recorder returned by [`Mob::animate`].
#[derive(Debug, Clone)]
pub struct AnimBuilder {
    target: Mob,
    commands: Vec<Command>,
}

impl AnimBuilder {
    #[must_use]
    pub fn new(target: Mob) -> Self {
        Self {
            target,
            commands: Vec::new(),
        }
    }

    #[must_use]
    pub fn shift(mut self, delta: [f64; 3]) -> Self {
        self.commands.push(Command::Shift(delta));
        self
    }

    #[must_use]
    pub fn rotate(mut self, angle: f64) -> Self {
        self.commands.push(Command::Rotate(angle));
        self
    }

    #[must_use]
    pub fn set_opacity(mut self, opacity: f32) -> Self {
        self.commands.push(Command::SetOpacity(opacity));
        self
    }

    #[must_use]
    pub fn target(&self) -> Mob {
        self.target
    }

    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}

impl Mob {
    /// Start recording a deferred animation against this handle.
    #[must_use]
    pub fn animate(self) -> AnimBuilder {
        AnimBuilder::new(self)
    }
}

/// Anything `stage.play` accepts: one recording or a tuple of them.
pub trait IntoAnimations {
    fn into_animations(self) -> Vec<AnimBuilder>;
}

impl IntoAnimations for AnimBuilder {
    fn into_animations(self) -> Vec<AnimBuilder> {
        vec![self]
    }
}

impl IntoAnimations for (AnimBuilder, AnimBuilder) {
    fn into_animations(self) -> Vec<AnimBuilder> {
        vec![self.0, self.1]
    }
}

impl IntoAnimations for Vec<AnimBuilder> {
    fn into_animations(self) -> Vec<AnimBuilder> {
        self
    }
}

/// A play error: the fluent surface refuses stale or foreign handles
/// instead of silently dropping work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleHandle(pub Mob);

impl std::fmt::Display for StaleHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "play() target handle is stale or from another stage")
    }
}

impl std::error::Error for StaleHandle {}

impl Stage {
    /// Realize recorded animations (instantly, in the spike) and advance
    /// time by one nominal second.
    pub fn play(&mut self, animations: impl IntoAnimations) -> Result<(), StaleHandle> {
        let animations = animations.into_animations();
        for anim in &animations {
            if !self.contains(anim.target()) {
                return Err(StaleHandle(anim.target()));
            }
        }
        for anim in animations {
            let entry = self.get_mut(anim.target()).expect("checked above");
            for command in anim.commands() {
                apply(&mut entry.buffer, *command);
            }
        }
        self.update(1.0);
        Ok(())
    }
}

fn apply(buffer: &mut crate::record::RecordBuffer, command: Command) {
    for i in 0..buffer.len() {
        match command {
            Command::Shift([dx, dy, dz]) => {
                if let Some(p) = buffer.read(i, "point") {
                    buffer.write(
                        i,
                        "point",
                        &[p[0] + dx as f32, p[1] + dy as f32, p[2] + dz as f32],
                    );
                }
            }
            Command::Rotate(angle) => {
                if let Some(p) = buffer.read(i, "point") {
                    let (sin, cos) = angle.sin_cos();
                    let (x, y) = (f64::from(p[0]), f64::from(p[1]));
                    buffer.write(
                        i,
                        "point",
                        &[(x * cos - y * sin) as f32, (x * sin + y * cos) as f32, p[2]],
                    );
                }
            }
            Command::SetOpacity(alpha) => {
                if let Some(rgba) = buffer.read(i, "rgba") {
                    buffer.write(i, "rgba", &[rgba[0], rgba[1], rgba[2], alpha]);
                }
            }
        }
    }
}
