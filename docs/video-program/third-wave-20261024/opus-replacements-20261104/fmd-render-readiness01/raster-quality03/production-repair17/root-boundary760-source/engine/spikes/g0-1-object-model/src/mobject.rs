//! Detached mobject values and the §15.1 builder shape: kwargs become
//! `Default` config structs with chainable setters, `build()`/`stage.add()`
//! is the boundary where a value enters the arena.

use fmn_core::color::Srgb;
use fmn_core::constants::WHITE;
use fmn_core::types::Vec3;

use crate::record::{RecordBuffer, RecordSchema};

/// A detached mobject: plain data, constructible before any stage exists
/// (lifetime scenario 1). `Stage::add` moves it — and its detached
/// children — into the arena.
pub struct Mobject {
    pub buffer: RecordBuffer,
    pub submobjects: Vec<Mobject>,
}

impl Mobject {
    /// A mobject over the given points with a uniform color.
    #[must_use]
    pub fn from_points(points: &[Vec3], color: Srgb) -> Self {
        let mut buffer = RecordBuffer::new(RecordSchema::manim_default(), points.len());
        let [r, g, b] = [color.r, color.g, color.b];
        for (i, p) in points.iter().enumerate() {
            buffer.write(i, "point", &[p[0] as f32, p[1] as f32, p[2] as f32]);
            buffer.write(i, "rgba", &[r as f32, g as f32, b as f32, 1.0]);
        }
        Self {
            buffer,
            submobjects: Vec::new(),
        }
    }

    /// Group composition while detached.
    #[must_use]
    pub fn group(children: Vec<Mobject>) -> Self {
        Self {
            buffer: RecordBuffer::new(RecordSchema::manim_default(), 0),
            submobjects: children,
        }
    }
}

/// The `Square::new().side_length(2.0).color(BLUE)` builder — the §15.1
/// config-struct shape (every field has a default; setters chain by value).
#[derive(Debug, Clone, Copy)]
pub struct Square {
    side_length: f64,
    color: Srgb,
}

impl Default for Square {
    fn default() -> Self {
        Self {
            side_length: 2.0,
            color: WHITE,
        }
    }
}

impl Square {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn side_length(mut self, side_length: f64) -> Self {
        self.side_length = side_length;
        self
    }

    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.color = color;
        self
    }

    /// Corner points only — the spike needs identifiable geometry, not the
    /// real QuadPath wiring (that composition is W3's job).
    #[must_use]
    pub fn build(self) -> Mobject {
        let half = self.side_length / 2.0;
        let corners = [
            [half, half, 0.0],
            [-half, half, 0.0],
            [-half, -half, 0.0],
            [half, -half, 0.0],
        ];
        Mobject::from_points(&corners, self.color)
    }
}

impl From<Square> for Mobject {
    fn from(square: Square) -> Self {
        square.build()
    }
}
