//! Small geometry and number helpers: vectors, the `Frame2` data window,
//! rectangles and dot clouds, and thousands separators for numbers built from
//! `facts` (never typed).

use fmn::prelude::*;

use crate::palette::stroke_style;

/// A point in the scene plane.
pub fn v(x: f64, y: f64) -> Vec3 {
    [x, y, 0.0]
}

pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// An integer with thousands separators ("5,000"), for numbers inside
/// on-screen strings. Format them from `facts`; never type them. (Commas are
/// fine on screen and never in script.tsv: the TTS reads them as pauses.)
pub fn thousands(n: impl Into<u128>) -> String {
    let digits = n.into().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// As [`thousands`] for signed values, with a typographic minus (U+2212),
/// which the bundled fonts carry and which reads better than a hyphen.
pub fn signed_thousands(n: i128) -> String {
    let body = thousands(n.unsigned_abs());
    if n < 0 { format!("\u{2212}{body}") } else { body }
}

/// A plain rectangle with its lower-left corner at `dl` (degenerate sizes are
/// clamped to a hairline instead of failing).
pub fn rect(dl: Vec3, w: f64, h: f64, style: Style) -> fmn::Result<VMobject> {
    let r = Rectangle::new()
        .width(w.max(1e-4))
        .height(h.max(1e-4))
        .style(style)
        .build()?;
    Ok(r.moved_to([dl[0] + w / 2.0, dl[1] + h / 2.0, 0.0]))
}

/// Dots, one per point, as a single group, coloured per point.
///
/// # Panics
/// If `colors` and `points` differ in length (a shorter list would silently
/// drop dots); `point_colors(points.len())` always fits.
pub fn dot_cloud(points: &[Vec3], colors: &[Srgb], radius: f64) -> VMobject {
    assert_eq!(points.len(), colors.len(), "dot_cloud: one color per point");
    v_group(
        points
            .iter()
            .zip(colors)
            .map(|(&p, &c)| Dot::new().point(p).radius(radius).color(c).build()),
    )
}

/// The children of a group, in order.
pub fn children(stage: &Stage<'_>, group: Mob) -> Vec<Mob> {
    stage.get(group).map(|e| e.submobjects().to_vec()).unwrap_or_default()
}

/// A data window: maps the data rectangle `x × y` onto a scene rectangle
/// centred at `center`. Native `Axes` can be placed with `c2p` following (since
/// franken_manim 33e2ba17, placed before it is added), but `NumberPlane`,
/// `ComplexPlane` and `ThreeDAxes` cannot (fm-native-axes-reposition-n1yd), and a
/// plain window is often all a plot needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame2 {
    pub center: Vec3,
    pub width: f64,
    pub height: f64,
    pub x: (f64, f64),
    pub y: (f64, f64),
}

impl Frame2 {
    /// Data → scene.
    pub fn p(&self, x: f64, y: f64) -> Vec3 {
        let u = (x - self.x.0) / (self.x.1 - self.x.0) - 0.5;
        let w = (y - self.y.0) / (self.y.1 - self.y.0) - 0.5;
        [self.center[0] + u * self.width, self.center[1] + w * self.height, 0.0]
    }

    /// Scene → data (inverse of [`Frame2::p`]).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_formats_like_the_screen_wants() {
        assert_eq!(thousands(0u32), "0");
        assert_eq!(thousands(999u32), "999");
        assert_eq!(thousands(1_000u32), "1,000");
        assert_eq!(thousands(4_845u32), "4,845");
        assert_eq!(thousands(2_126_816_238u64), "2,126,816,238");
        assert_eq!(thousands(1_234_567u32), "1,234,567");
        assert_eq!(thousands(u128::MAX).matches(',').count(), 12);
        assert_eq!(signed_thousands(-1_234_567), "\u{2212}1,234,567");
        assert_eq!(signed_thousands(-999), "\u{2212}999");
        assert_eq!(signed_thousands(0), "0");
        assert_eq!(signed_thousands(1_000), "1,000");
        assert_eq!(signed_thousands(i128::MIN).matches(',').count(), 12);
    }

    #[test]
    #[should_panic(expected = "one color per point")]
    fn dot_cloud_refuses_a_short_color_list() {
        dot_cloud(&[v(0.0, 0.0), v(1.0, 0.0)], &[crate::Palette::blue()], 0.05);
    }

    #[test]
    fn frame2_round_trips_and_edges() {
        let f = Frame2 {
            center: v(-3.45, -0.4),
            width: 5.6,
            height: 3.2,
            x: (-0.04, 0.2),
            y: (0.0, 49.0),
        };
        for i in 0..=20 {
            for j in 0..=20 {
                let (x, y) = (lerp(f.x.0, f.x.1, i as f64 / 20.0), lerp(f.y.0, f.y.1, j as f64 / 20.0));
                let (x2, y2) = f.data(f.p(x, y));
                assert!((x2 - x).abs() < 1e-12 && (y2 - y).abs() < 1e-12);
            }
        }
        assert!((f.p(f.x.0, f.y.0)[0] - f.left()).abs() < 1e-12);
        assert!((f.p(f.x.1, f.y.1)[1] - f.top()).abs() < 1e-12);
        assert!((f.right() - f.left() - f.width).abs() < 1e-12);
    }

    #[test]
    fn border_builds() {
        let f = Frame2 {
            center: v(0.0, 0.0),
            width: 4.0,
            height: 2.0,
            x: (0.0, 1.0),
            y: (0.0, 1.0),
        };
        assert!(f.border(crate::Palette::dim()).is_ok());
    }
}
