//! The truss (storyboard 05 to 07, and faintly behind 02_4): the model's
//! Pratt truss drawn from its own joints and members, its supports, and (in
//! 05 and after) the moves and forces of the stiffness method on it.
//!
//! Positions are the model's, in metres, placed on screen by a [`Layout`]:
//! nothing here is typed from the picture.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::v;
use fmn::prelude::*;

use super::draw::faded_line;
use crate::model::truss::{Kind, Truss};

/// Where the truss sits on the screen: joint 0 (the pinned end, at the
/// bottom left) at `origin`, `scale` scene units to a metre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub origin: Vec3,
    pub scale: f64,
}

impl Layout {
    /// The screen point `dx`, `dy` metres from the pinned end.
    pub fn at(&self, dx: f64, dy: f64) -> Vec3 {
        v(self.origin[0] + self.scale * dx, self.origin[1] + self.scale * dy)
    }

    /// Joint `j` of `truss`, at rest.
    pub fn joint(&self, truss: &Truss, j: usize) -> Vec3 {
        let (x, y) = truss.joints[j];
        self.at(x, y)
    }

    /// The width the truss takes on screen.
    pub fn width(&self, truss: &Truss) -> f64 {
        self.scale * truss.span_m()
    }
}

/// How heavy a member is drawn: the chords and end posts, which carry the
/// most steel, over the posts and diagonals.
pub fn member_width(kind: Kind) -> f64 {
    match kind {
        Kind::Bottom | Kind::Top | Kind::EndPost => 6.0,
        Kind::Vertical | Kind::Diagonal => 3.4,
    }
}

/// The truss at rest as one polyline per member, in `colour`, detached.
pub fn outline(stage: &mut Stage<'_>, truss: &Truss, layout: Layout, colour: Srgb, opacity: f64) -> Vec<Mob> {
    truss
        .members
        .iter()
        .map(|m| {
            let line = polyline(vec![layout.joint(truss, m.a), layout.joint(truss, m.b)])
                .style(faded_line(colour, member_width(m.kind), opacity))
                .build();
            stage.arena_mut().add(line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::truss::{DEPTH_M, PANEL_M, PANELS};

    #[test]
    fn the_layout_places_the_model_in_metres() {
        let t = Truss::pratt();
        let l = Layout {
            origin: v(-3.0, -1.0),
            scale: 0.25,
        };
        assert_eq!(l.joint(&t, 0), v(-3.0, -1.0));
        assert_eq!(l.joint(&t, PANELS), v(-3.0 + 0.25 * PANELS as f64 * PANEL_M, -1.0));
        // the first top joint is one panel in and one depth up
        let top = l.joint(&t, PANELS + 1);
        assert!((top[0] - (-3.0 + 0.25 * PANEL_M)).abs() < 1e-12 && (top[1] - (-1.0 + 0.25 * DEPTH_M)).abs() < 1e-12);
        assert!((l.width(&t) - 0.25 * 24.0).abs() < 1e-12);
    }

    #[test]
    fn chords_are_drawn_heavier_than_the_web() {
        assert!(member_width(Kind::Bottom) > member_width(Kind::Diagonal));
        assert_eq!(member_width(Kind::Top), member_width(Kind::EndPost));
        assert_eq!(member_width(Kind::Vertical), member_width(Kind::Diagonal));
    }
}
