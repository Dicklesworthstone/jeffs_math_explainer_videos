//! The carts (storyboard 02 and everywhere the idea is rehearsed on them):
//! two carts between two walls, joined by three equal springs, a ruler under
//! the track, the move of each cart as a blue arrow and the push that holds
//! it as an orange one. Everything on it is rewritten by one updater from two
//! trackers, the carts' moves in centimetres, and only when they change; the
//! pushes are the carts' table applied to the moves every frame
//! (`model::carts::pushes`), never typed.
//!
//! The geometry is pure functions at scale 1 (walls 4.6 either side of the
//! picture's centre); `CartsSpec::scale` shrinks the whole picture, for a quiz
//! card or a corner.
#![allow(dead_code, reason = "shared props land before the chapters that draw them")]

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{arrow_points, faded_line, flat, group, moves, panel, pushes, squeezed, stretched};
use crate::model::carts;

/// The walls' inner faces, either side of the centre.
pub const HALF: f64 = 4.6;
/// A cart's width and height.
pub const CART_W: f64 = 1.2;
pub const CART_H: f64 = 0.85;
/// The length of each spring at rest: the three equal springs fill the
/// distance between the walls that the two carts leave.
pub const REST_LEN: f64 = (2.0 * HALF - 2.0 * CART_W) / 3.0;
/// Scene units for one centimetre of move.
pub const CM: f64 = 0.9;
/// Scene units for one newton of push, as an arrow.
pub const NEWTON: f64 = 0.55;
/// Coils in a spring, and their height either side of the axis.
const COILS: usize = 7;
const COIL_AMP: f64 = 0.17;

/// Where a picture of the carts sits and how large it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CartsSpec {
    /// The middle of the track, between the carts.
    pub centre: Vec3,
    /// 1.0 is the full picture, walls at ±[`HALF`].
    pub scale: f64,
}

/// The x of cart `i`'s centre (0 left, 1 right), relative to the picture's
/// centre at scale 1, when it has moved `u` centimetres to the right.
pub fn cart_x(i: usize, u: f64) -> f64 {
    let rest = -HALF + REST_LEN + CART_W / 2.0 + i as f64 * (CART_W + REST_LEN);
    rest + CM * u
}

/// The x of the two ends of spring `s` (0 left, 1 middle, 2 right) for the
/// carts' moves `u`: the wall or a cart's face, to a cart's face or the wall.
pub fn spring_ends(s: usize, u: [f64; 2]) -> (f64, f64) {
    let (a, b) = (cart_x(0, u[0]), cart_x(1, u[1]));
    match s {
        0 => (-HALF, a - CART_W / 2.0),
        1 => (a + CART_W / 2.0, b - CART_W / 2.0),
        _ => (b + CART_W / 2.0, HALF),
    }
}

/// How much longer spring `s` is than at rest, in centimetres of move
/// (negative when squeezed).
pub fn stretch_cm(s: usize, u: [f64; 2]) -> f64 {
    let (x0, x1) = spring_ends(s, u);
    (x1 - x0 - REST_LEN) / CM
}

/// A spring drawn as a zigzag from `x0` to `x1` at height `y`: a straight
/// lead at each end and `coils` full coils between.
pub fn zigzag(x0: f64, x1: f64, y: f64, coils: usize, amp: f64) -> Vec<Vec3> {
    let len = x1 - x0;
    let lead = (0.15 * len.abs()).min(0.3).copysign(len);
    let (a, b) = (x0 + lead, x1 - lead);
    let steps = 2 * coils;
    let mut pts = vec![v(x0, y), v(a, y)];
    for k in 0..steps {
        let x = a + (b - a) * (k as f64 + 0.5) / steps as f64;
        let side = if k % 2 == 0 { amp } else { -amp };
        pts.push(v(x, y + side));
    }
    pts.push(v(b, y));
    pts.push(v(x1, y));
    pts
}

/// A spring's colour and line width for a stretch of `s` centimetres:
/// unchanged is white, stretched thins and tints teal, squeezed thickens
/// and tints magenta, fully at one centimetre.
pub fn spring_look(s: f64) -> (Srgb, f64) {
    let k = s.abs().min(1.0);
    let role = if s >= 0.0 { stretched() } else { squeezed() };
    let ink = Palette::ink();
    let mix = |a: f64, b: f64| a + (b - a) * k;
    let colour = Srgb {
        r: mix(ink.r, role.r),
        g: mix(ink.g, role.g),
        b: mix(ink.b, role.b),
    };
    let width = if s >= 0.0 { 5.0 - 1.5 * k } else { 5.0 + 3.0 * k };
    (colour, width)
}

/// The two pushes (newtons) that hold the carts at the moves `u`
/// (centimetres): the table of `model::carts` applied to them.
pub fn pushes_for(u: [f64; 2]) -> [f64; 2] {
    carts::pushes(u)
}

/// The moves the chapters try, in centimetres: the left cart alone (02_2),
/// both together (02_6, 02_7), toward each other (03_1). Their pushes are
/// the facts `K-A1.push.left`, `.together` and `.against`.
pub const LEFT_ONLY: [f64; 2] = [1.0, 0.0];
pub const TOGETHER: [f64; 2] = [1.0, 1.0];
pub const AGAINST: [f64; 2] = [1.0, -1.0];

/// The row of the table `row` applied to the moves `u`, as the sum the
/// storyboard writes: `2 \times 1 + (-1) \times 0 = 2`, a negative number in
/// brackets.
pub fn row_sum_tex(row: [f64; 2], u: [f64; 2]) -> String {
    let term = |a: f64, b: f64| {
        let a = super::draw::whole(a);
        let b = super::draw::whole(b);
        if a.starts_with('-') {
            format!("({a}) \\times {b}")
        } else {
            format!("{a} \\times {b}")
        }
    };
    let total = row[0] * u[0] + row[1] * u[1];
    format!(
        "{} + {} = {}",
        term(row[0], u[0]),
        term(row[1], u[1]),
        super::draw::whole(total)
    )
}

/// A drawn picture of the carts.
pub struct CartsPic {
    pub spec: CartsSpec,
    /// The carts' moves in centimetres: trackers the chapter tweens.
    pub u: [Mob; 2],
    /// How much of the blue move arrows shows, 0 to 1; how much of each
    /// orange push arrow shows (left cart, right cart), 0 to 1; how much of
    /// the two hands shows, 0 to 1: trackers the chapter tweens.
    pub moves_on: Mob,
    pub pushes_on: [Mob; 2],
    pub hands_on: Mob,
    /// Walls, track and ruler: still while the carts move.
    pub frame: Vec<Mob>,
    pub carts: [Mob; 2],
    pub springs: [Mob; 3],
    pub move_arrows: [Mob; 2],
    pub push_arrows: [Mob; 2],
    pub hands: [Mob; 2],
    anchor: Mob,
}

impl CartsPic {
    /// Build the picture, detached. The updater runs from the moment the
    /// picture is added to the scene (any of its parts).
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, spec: CartsSpec, u: [Mob; 2]) -> fmn::Result<Self> {
        let CartsSpec { centre, scale } = spec;
        let at = move |x: f64, y: f64| v(centre[0] + scale * x, centre[1] + scale * y);
        let ink = Palette::ink();
        let dim = Palette::dim();

        // walls, the track and the ruler
        let mut frame = Vec::new();
        for side in [-1.0, 1.0] {
            let wall = Rectangle::new()
                .width(0.16 * scale)
                .height(1.7 * scale)
                .style(flat(dim, 1.0))
                .build()?
                .moved_to(at(side * (HALF + 0.08), 0.0));
            frame.push(stage.arena_mut().add(wall));
        }
        let track_y = -CART_H / 2.0 - 0.04;
        let track = polyline(vec![at(-HALF, track_y), at(HALF, track_y)])
            .style(faded_line(Palette::faint(), 3.0, 1.0))
            .build();
        frame.push(stage.arena_mut().add(track));
        // the ruler: a tick a centimetre, from -3 to 3, with 0 and the unit named
        let ruler_y = -CART_H / 2.0 - 0.62;
        let ruler = polyline(vec![at(cart_x(0, -3.0), ruler_y), at(cart_x(0, 3.0), ruler_y)])
            .style(faded_line(dim, 2.0, 1.0))
            .build();
        frame.push(stage.arena_mut().add(ruler));
        for k in -3..=3 {
            let x = cart_x(0, f64::from(k));
            let tall = if k == 0 { 0.16 } else { 0.1 };
            let tick = polyline(vec![at(x, ruler_y - tall), at(x, ruler_y + tall)])
                .style(faded_line(dim, 2.0, 1.0))
                .build();
            frame.push(stage.arena_mut().add(tick));
        }
        let cm = kit.text(
            stage,
            "cm",
            22.0 * scale.max(0.7),
            dim,
            at(cart_x(0, 3.0) + 0.45, ruler_y),
        )?;
        frame.push(cm);

        // carts, springs, arrows and hands
        let cart_style = Style::default().fill(panel(), 1.0).stroke(ink, 2.5, 1.0);
        let mut cart_mobs = Vec::new();
        for i in 0..2 {
            let r = Rectangle::new()
                .width(CART_W * scale)
                .height(CART_H * scale)
                .corner_radius(0.1 * scale)
                .style(cart_style)
                .build()?
                .moved_to(at(cart_x(i, 0.0), 0.0));
            cart_mobs.push(stage.arena_mut().add(r));
        }
        let mut spring_mobs = Vec::new();
        for s in 0..3 {
            let (x0, x1) = spring_ends(s, [0.0, 0.0]);
            let pts: Vec<Vec3> = zigzag(x0, x1, 0.0, COILS, COIL_AMP)
                .into_iter()
                .map(|p| at(p[0], p[1]))
                .collect();
            let m = polyline(pts).style(faded_line(ink, 5.0, 1.0)).build();
            spring_mobs.push(stage.arena_mut().add(m));
        }
        let mut move_mobs = Vec::new();
        let mut push_mobs = Vec::new();
        let mut hand_mobs = Vec::new();
        for i in 0..2 {
            let x = cart_x(i, 0.0);
            let m = polyline(arrow_points(
                at(x, -CART_H / 2.0 - 0.3),
                at(x + 1e-3, -CART_H / 2.0 - 0.3),
                0.22 * scale,
            ))
            .style(faded_line(moves(), 6.0, 0.0))
            .build();
            move_mobs.push(stage.arena_mut().add(m));
            let p = polyline(arrow_points(
                at(x, CART_H / 2.0 + 0.5),
                at(x + 1e-3, CART_H / 2.0 + 0.5),
                0.22 * scale,
            ))
            .style(faded_line(pushes(), 6.0, 0.0))
            .build();
            push_mobs.push(stage.arena_mut().add(p));
            // a mitten: palm and thumb, resting on the cart
            let palm = Rectangle::new()
                .width(0.46 * scale)
                .height(0.3 * scale)
                .corner_radius(0.09 * scale)
                .style(flat(ink, 0.0))
                .build()?
                .moved_to(at(x, CART_H / 2.0 + 0.5));
            let thumb = Rectangle::new()
                .width(0.14 * scale)
                .height(0.24 * scale)
                .corner_radius(0.05 * scale)
                .style(flat(ink, 0.0))
                .build()?
                .moved_to(at(x - 0.2, CART_H / 2.0 + 0.34));
            let (palm, thumb) = (stage.arena_mut().add(palm), stage.arena_mut().add(thumb));
            hand_mobs.push(group(stage, &[palm, thumb])?);
        }

        let moves_on = stage.add_value_tracker(0.0);
        let pushes_on = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let hands_on = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (carts_m, springs_m) = (
            [cart_mobs[0], cart_mobs[1]],
            [spring_mobs[0], spring_mobs[1], spring_mobs[2]],
        );
        let (move_m, push_m, hand_m) = (
            [move_mobs[0], move_mobs[1]],
            [push_mobs[0], push_mobs[1]],
            [hand_mobs[0], hand_mobs[1]],
        );
        let mut last = [f64::NAN; 6];
        stage.add_updater(
            anchor,
            move |s, _| {
                let moved = [
                    s.tracker_value(u[0]).unwrap_or(0.0),
                    s.tracker_value(u[1]).unwrap_or(0.0),
                ];
                let mo = s.tracker_value(moves_on).unwrap_or(0.0);
                let po = [
                    s.tracker_value(pushes_on[0]).unwrap_or(0.0),
                    s.tracker_value(pushes_on[1]).unwrap_or(0.0),
                ];
                let ho = s.tracker_value(hands_on).unwrap_or(0.0);
                let now = [moved[0], moved[1], mo, po[0], po[1], ho];
                if now.iter().zip(&last).all(|(a, b)| a.to_bits() == b.to_bits()) {
                    return;
                }
                last = now;
                let force = pushes_for(moved);
                for i in 0..2 {
                    let x = cart_x(i, moved[i]);
                    s.move_to(carts_m[i], at(x, 0.0), ORIGIN);
                    // the blue move: from where the cart rests to where it is
                    let (from, to) = (cart_x(i, 0.0), x);
                    let _ = s.set_points(
                        move_m[i],
                        &arrow_points(at(from, -CART_H / 2.0 - 0.3), at(to, -CART_H / 2.0 - 0.3), 0.22 * scale),
                    );
                    s.set_stroke(move_m[i], None, None, Some(mo), None, false);
                    // the orange push, from the hand outward
                    let tip = x + NEWTON * force[i];
                    let _ = s.set_points(
                        push_m[i],
                        &arrow_points(at(x, CART_H / 2.0 + 0.5), at(tip, CART_H / 2.0 + 0.5), 0.22 * scale),
                    );
                    s.set_stroke(push_m[i], None, None, Some(po[i]), None, false);
                    s.move_to(hand_m[i], at(x, CART_H / 2.0 + 0.5), ORIGIN);
                    s.set_fill(hand_m[i], None, Some(0.9 * ho), None, true);
                }
                for (k, &spring) in springs_m.iter().enumerate() {
                    let (x0, x1) = spring_ends(k, moved);
                    let pts: Vec<Vec3> = zigzag(x0, x1, 0.0, COILS, COIL_AMP)
                        .into_iter()
                        .map(|p| at(p[0], p[1]))
                        .collect();
                    let _ = s.set_points(spring, &pts);
                    let (colour, width) = spring_look(stretch_cm(k, moved));
                    s.set_stroke(spring, Some(colour), Some(width), None, None, false);
                }
            },
            true,
        )?;
        Ok(Self {
            spec,
            u,
            moves_on,
            pushes_on,
            hands_on,
            frame,
            carts: carts_m,
            springs: springs_m,
            move_arrows: move_m,
            push_arrows: push_m,
            hands: hand_m,
            anchor,
        })
    }

    /// The parts that fade in and out with the picture: walls, track,
    /// ruler, carts and springs. The move arrows, the push arrows and the
    /// hands are shown by their trackers ([`CartsPic::moves_on`],
    /// [`CartsPic::pushes_on`], [`CartsPic::hands_on`]), never faded.
    pub fn base(&self) -> Vec<Mob> {
        let mut all = self.frame.clone();
        all.extend(self.carts);
        all.extend(self.springs);
        all
    }

    /// Start the updater that places the picture from the trackers: add it
    /// to the scene together with the picture's first fade-in.
    pub fn start(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        // the arrows and the hands are in the scene from the start, at opacity 0
        for m in self.move_arrows.iter().chain(&self.push_arrows).chain(&self.hands) {
            stage.add_to_scene(*m)?;
        }
        Ok(())
    }

    /// Stop the updater (before the picture goes, or the chapter ends).
    pub fn stop(&self, stage: &mut Stage<'_>) {
        stage.clear_updaters(self.anchor, true);
    }

    /// Where cart `i`'s centre is on screen when it has moved `u` cm.
    pub fn cart_at(&self, i: usize, u: f64) -> Vec3 {
        let CartsSpec { centre, scale } = self.spec;
        v(centre[0] + scale * cart_x(i, u), centre[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three equal springs and two carts fill the space between the walls.
    #[test]
    fn at_rest_the_three_springs_are_equal_and_fill_the_gap() {
        for s in 0..3 {
            let (a, b) = spring_ends(s, [0.0, 0.0]);
            assert!((b - a - REST_LEN).abs() < 1e-12, "spring {s}");
            assert!(stretch_cm(s, [0.0, 0.0]).abs() < 1e-12, "spring {s} at rest");
        }
        assert!(
            (cart_x(0, 0.0) + cart_x(1, 0.0)).abs() < 1e-12,
            "the carts sit symmetrically"
        );
        assert!(REST_LEN > 0.0 && (3.0 * REST_LEN + 2.0 * CART_W - 2.0 * HALF).abs() < 1e-12);
    }

    /// 02_2 and 02_3: slide the left cart one centimetre right and the left
    /// spring stretches, the middle one squeezes and the right is unchanged;
    /// the push that holds each cart is the difference of its two springs'
    /// stretches, and equals the table applied to the moves: 2 and -1.
    #[test]
    fn a_push_is_the_difference_of_the_springs_either_side() {
        let u = [1.0, 0.0];
        let s: Vec<f64> = (0..3).map(|k| stretch_cm(k, u)).collect();
        assert!(
            (s[0] - 1.0).abs() < 1e-12 && (s[1] + 1.0).abs() < 1e-12 && s[2].abs() < 1e-12,
            "{s:?}"
        );
        let f = pushes_for(u);
        assert_eq!(f, [2.0, -1.0]);
        for (u, name) in [
            ([1.0, 0.0], "left"),
            ([1.0, 1.0], "together"),
            ([1.0, -1.0], "against"),
            ([0.3, -2.0], "odd"),
        ] {
            let s: Vec<f64> = (0..3).map(|k| stretch_cm(k, u)).collect();
            let want = [s[0] - s[1], s[1] - s[2]];
            let got = pushes_for(u);
            assert!(
                (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
                "{name}: {got:?} against {want:?}"
            );
        }
    }

    /// 02_7: both carts together leave the middle spring exactly unstretched.
    #[test]
    fn moving_together_leaves_the_middle_spring_alone() {
        assert!(stretch_cm(1, [1.0, 1.0]).abs() < 1e-12);
        let (colour, width) = spring_look(0.0);
        assert_eq!(colour, Palette::ink());
        assert_eq!(width, 5.0);
        // and the pushes come back as the moves: 1 and 1
        assert_eq!(pushes_for([1.0, 1.0]), [1.0, 1.0]);
    }

    #[test]
    fn stretched_springs_thin_and_tint_teal_and_squeezed_ones_thicken_and_tint_magenta() {
        let (stretched_colour, thin) = spring_look(1.0);
        let (squeezed_colour, thick) = spring_look(-1.0);
        for (got, want) in [(stretched_colour, stretched()), (squeezed_colour, squeezed())] {
            assert!(
                (got.r - want.r).abs() < 1e-12 && (got.g - want.g).abs() < 1e-12 && (got.b - want.b).abs() < 1e-12,
                "{got:?} against {want:?}"
            );
        }
        assert!(thin < 5.0 && thick > 5.0);
        // beyond a centimetre the tint is full and the width does not run away
        assert_eq!(spring_look(4.0), spring_look(1.0));
        // half a centimetre is halfway to the role
        let (half, _) = spring_look(0.5);
        let ink = Palette::ink();
        assert!((half.r - (ink.r + stretched().r) / 2.0).abs() < 1e-12);
    }

    /// The moves the chapters try give the pushes the facts hold, and the
    /// rows are written as the storyboard writes them (02_5, 02_7).
    #[test]
    fn the_example_moves_give_the_facts_pushes_and_the_rows_read_as_written() {
        for (u, key) in [
            (LEFT_ONLY, "K-A1.push.left"),
            (TOGETHER, "K-A1.push.together"),
            (AGAINST, "K-A1.push.against"),
        ] {
            let want = super::super::draw::numbers(key).unwrap();
            assert_eq!(pushes_for(u).to_vec(), want, "{key}");
        }
        assert_eq!(row_sum_tex([2.0, -1.0], LEFT_ONLY), r"2 \times 1 + (-1) \times 0 = 2");
        assert_eq!(row_sum_tex([-1.0, 2.0], LEFT_ONLY), r"(-1) \times 1 + 2 \times 0 = -1");
        assert_eq!(row_sum_tex([2.0, -1.0], TOGETHER), r"2 \times 1 + (-1) \times 1 = 1");
    }

    #[test]
    fn a_zigzag_runs_end_to_end_and_alternates_sides() {
        let z = zigzag(-1.0, 2.0, 0.5, COILS, 0.2);
        assert_eq!(z.len(), 2 * COILS + 4);
        assert_eq!(z[0], v(-1.0, 0.5));
        assert_eq!(z[z.len() - 1], v(2.0, 0.5));
        let ys: Vec<f64> = z[2..z.len() - 2].iter().map(|p| p[1] - 0.5).collect();
        assert!(ys.windows(2).all(|w| (w[0] + w[1]).abs() < 1e-12 && w[0].abs() > 0.19));
        // x only moves forward
        assert!(z.windows(2).all(|w| w[1][0] >= w[0][0]));
        // a squeezed spring (a short one) still has points in order
        let short = zigzag(0.0, 0.4, 0.0, COILS, 0.2);
        assert!(short.windows(2).all(|w| w[1][0] >= w[0][0] - 1e-12));
    }
}
