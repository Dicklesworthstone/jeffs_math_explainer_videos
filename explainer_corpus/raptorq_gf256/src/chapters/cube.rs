//! Chapter 04, the cube (camera route, spike S-5): the eight possible cards
//! on three pieces, drawn flat where the cube's corners will fall (04_1);
//! the drawing lifts into a real cube as the camera tilts, one direction per
//! piece (04_2); one useful card lights two corners, two light the bottom
//! face, four (04_3); random cards fly to the corners (04_4), and half of
//! them land where the bucket can already make them (04_5). The corners are
//! the cards (bit i = piece i), the lit ones `model::Bucket::span` of A and
//! B, the chance oracle B-F4.useless_prob_3pieces_2cards_held.

use explainer_kit::props::PREDICT_HOLD_S;
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::core::constants::{FRAME_HEIGHT, FRAME_WIDTH};
use fmn::prelude::*;
use fmn::rendering::CameraConfig;

use super::draw::{Role, line_style};
use super::fold::fade_list;
use crate::model::{A, B, Bucket, PIECE_NAMES, SplitMix64, useless_chance};

/// Pieces: the cube's dimensions.
const PIECES: u32 = 3;
/// The cube's side and the flat drawing's slant for the third direction.
const SIDE: f64 = 3.2;
const SLANT: (f64, f64) = (1.5, 1.15);
/// Where the camera settles (radians): a three-quarter view.
const THETA: f64 = 0.55;
const PHI: f64 = 1.05;
/// The seed of 04_4's random cards: the hook's seed, this chapter's stream.
const DART_SEED: u64 = crate::model::FIELD_SEED ^ 0x0404;
const DARTS: usize = 8;

/// A card's corner in the cube: one direction per piece.
fn corner(c: u32) -> Vec3 {
    let bit = |i: u32| f64::from(c >> i & 1) - 0.5;
    [SIDE * bit(0), SIDE * bit(1), SIDE * bit(2)]
}

/// The same corner in the flat drawing (the third piece drawn slanted).
fn flat_corner(c: u32) -> Vec3 {
    let p = corner(c);
    let up = f64::from(c >> 2 & 1) - 0.5;
    v(p[0] + SLANT.0 * up - 0.3, p[1] + SLANT.1 * up - 0.2)
}

/// A card's name: "∅", "A", "AB", ….
fn name(c: u32) -> String {
    if c == 0 {
        return "∅".to_owned();
    }
    (0..PIECES as usize)
        .filter(|&i| c >> i & 1 == 1)
        .map(|i| PIECE_NAMES[i])
        .collect()
}

/// Where world point `p` lands on screen under camera `cfg`: the camera's
/// own projection (fmn-render `Camera::project`: the point in the frame's
/// coordinates, scaled by its rescale factors, divided by w = 1 − z).
fn on_screen(cfg: &CameraConfig, p: Vec3) -> Vec3 {
    let view = cfg.frame.to_fixed_frame_point(p, false);
    let f = cfg.frame.frame_rescale_factors();
    let w = 1.0 - view[2] * f[2];
    v(
        view[0] * f[0] / w * FRAME_WIDTH / 2.0,
        view[1] * f[1] / w * FRAME_HEIGHT / 2.0,
    )
}

/// A corner's ball, detached at `at`.
fn ball(stage: &mut Stage<'_>, color: Srgb, r: f64, at: Vec3) -> Mob {
    let s: Mobject = Sphere::new(r).resolution(24, 12).color(color).build().into();
    let m = stage.arena_mut().add(s);
    stage.move_to(m, at, ORIGIN);
    m
}

pub struct Cube {
    pub kit: Kit,
}

impl SceneConstruct for Cube {
    fn name(&self) -> &str {
        "04_the_cube"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (_, start) = kit.camera().ok_or(fmn::Error::Scene(SceneError::InvalidConfig(
            "04 runs on the camera route",
        )))?;
        let mut settled = start.clone();
        settled
            .frame
            .set_euler_angles(Some(THETA), Some(PHI), None)
            .map_err(SceneError::from)?;
        let cards: Vec<u32> = (0..1u32 << PIECES).collect();

        // 04_1: the eight cards, drawn flat where the corners will fall
        let balls: Vec<Mob> = cards
            .iter()
            .map(|&c| ball(stage, Palette::dim(), 0.16, flat_corner(c)))
            .collect();
        let edges = |pos: &dyn Fn(u32) -> Vec3| -> fmn::Result<Vec<VMobject>> {
            let mut out = Vec::new();
            for &c in &cards {
                for i in 0..PIECES {
                    let d = c | 1 << i;
                    if d != c {
                        out.push(
                            Line::new(pos(c), pos(d))
                                .style(line_style(Palette::dim(), 3.5))
                                .build()?,
                        );
                    }
                }
            }
            Ok(out)
        };
        let flat_edges: Vec<Mob> = edges(&flat_corner)?
            .into_iter()
            .map(|e| stage.arena_mut().add(e))
            .collect();
        let cube_edges: Vec<VMobject> = edges(&corner)?;
        let flat_labels: Vec<Mob> = cards
            .iter()
            .map(|&c| {
                let p = flat_corner(c);
                kit.text(stage, &name(c), 30.0, Palette::ink(), v(p[0] + 0.42, p[1] + 0.3))
            })
            .collect::<fmn::Result<_>>()?;
        kit.say(stage, "04_1")?;
        let mut intro: Vec<Box<dyn Animation>> = Vec::new();
        for &m in flat_edges.iter() {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            intro.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            intro,
            PlayOverrides {
                run_time: Some(0.8),
                ..PlayOverrides::default()
            },
        )?;
        for (&b, &l) in balls.iter().zip(&flat_labels) {
            play!(stage;
                fade_in(stage.arena_mut(), b, ORIGIN, 0.5)?.rt(0.35),
                fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?.rt(0.35),
            );
        }
        // the storyboard's hold on the layout
        kit.hold(stage)?;
        stage.wait(1.5)?;

        // 04_2: lift into a cube, the camera tilting; one direction per piece
        kit.say(stage, "04_2")?;
        let mut lift: Vec<Box<dyn Animation>> = Vec::new();
        for &l in &flat_labels {
            let a = fade_out(stage.arena_mut(), l, ORIGIN, 1.0)?;
            lift.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            lift,
            PlayOverrides {
                run_time: Some(0.4),
                ..PlayOverrides::default()
            },
        )?;
        let mut lift: Vec<Box<dyn Animation>> = Vec::new();
        for (&b, &c) in balls.iter().zip(&cards) {
            let a = b
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.4),
                    ..AnimateArgs::default()
                })?
                .move_to(corner(c), ORIGIN)?;
            lift.push(stage.prepare(a)?);
        }
        for (&e, target) in flat_edges.iter().zip(cube_edges) {
            let t = stage.arena_mut().add(target);
            lift.push(stage.prepare(Transform::new(e, t))?);
        }
        let turn = kit.camera_to(stage, THETA, PHI)?;
        lift.push(stage.prepare(turn)?);
        stage.play_prepared_with(
            lift,
            PlayOverrides {
                run_time: Some(2.4),
                ..PlayOverrides::default()
            },
        )?;
        // labels and the three directions, pinned to the screen where the
        // settled camera puts each corner
        let labels: Vec<Mob> = cards
            .iter()
            .map(|&c| {
                let p = on_screen(&settled, corner(c));
                kit.text_hud(stage, &name(c), 30.0, Palette::ink(), v(p[0] + 0.38, p[1] + 0.3))
            })
            .collect::<fmn::Result<_>>()?;
        let label_group = super::draw::group_of(stage, &labels)?;
        play!(stage; fade_in(stage.arena_mut(), label_group, ORIGIN, 1.0)?.rt(0.7).lag(0.08));
        // each direction's tag beside its edge from ∅, 55% along it, on the
        // edge's left: below the A edge, left of the C edge, inside the cube
        // above the B edge, never on a corner's label
        for i in 0..PIECES {
            let (a, b) = (on_screen(&settled, corner(0)), on_screen(&settled, corner(1 << i)));
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let len = (dx * dx + dy * dy).sqrt().max(1e-9);
            let (mut nx, mut ny) = (-dy / len, dx / len);
            if nx > 0.0 {
                (nx, ny) = (-nx, -ny);
            }
            let at = v(a[0] + 0.55 * dx + 0.42 * nx, a[1] + 0.55 * dy + 0.42 * ny);
            let tag = kit.text_hud(
                stage,
                &format!("use {}", PIECE_NAMES[i as usize]),
                34.0,
                Role::ex_or(),
                at,
            )?;
            play!(stage; fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5));
        }
        kit.hold(stage)?;

        // 04_3: one useful card lights two corners; two light four
        kit.say(stage, "04_3")?;
        let mut bucket = Bucket::new();
        let can_make = kit.text_hud(stage, "can make: 1", 48.0, Role::lit(), v(-4.9, 3.2))?;
        stage.add_to_scene(can_make)?;
        let mut shown = can_make;
        let light = |stage: &mut Stage<'_>, b: Mob| -> fmn::Result<Box<dyn Animation>> {
            let a = b
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .set_color(Role::lit())?
                .scale(1.35)?;
            stage.prepare(a)
        };
        let first = light(stage, balls[0])?;
        stage.play_prepared(vec![first])?;
        for useful in [A, B] {
            bucket.add(useful);
            let span = bucket.span();
            let mut anims = Vec::new();
            for &c in &span {
                if c != 0 && (c == useful || span.len() == 4 && c == (A | B)) {
                    anims.push(light(stage, balls[c as usize])?);
                }
            }
            stage.wait(1.2)?;
            stage.play_prepared(anims)?;
            let next = kit.text_hud(
                stage,
                &format!("can make: {}", span.len()),
                48.0,
                Role::lit(),
                v(-4.9, 3.2),
            )?;
            play!(stage; fmn::animation::replacement_transform(shown, next).rt(0.4));
            shown = next;
        }
        // the lit face: what A and B can make
        let face: Vec<Vec3> = [0, A, A | B, B].iter().map(|&c| corner(c)).collect();
        let face = Polygon::new(face)
            .style(Style::default().fill(Role::lit(), 0.28).stroke(Role::lit(), 0.0, 0.0))
            .build();
        let face = stage.arena_mut().add(face);
        play!(stage; fade_in(stage.arena_mut(), face, ORIGIN, 1.0)?.rt(0.8));
        kit.hold(stage)?;

        // 04_4: a random card: each piece in or out by a coin flip
        kit.say(stage, "04_4")?;
        let source = [0.0, 0.0, SIDE * 1.4];
        let mut rng = SplitMix64::new(DART_SEED);
        let targets: Vec<u32> = (0..DARTS).map(|_| (rng.next_u64() % 8) as u32).collect();
        // a tally of where they land, dots and no digits: lit, or dark
        let tally_x = 3.2;
        let lit_row = kit.text_hud(stage, "lit corner:\nuseless", 52.0, Role::lit(), v(tally_x, 1.5))?;
        let dark_row = kit.text_hud(stage, "dark corner:\nuseful", 52.0, Role::ex_or(), v(tally_x, -0.45))?;
        for m in [lit_row, dark_row] {
            let y = stage.get_center(m)[1];
            stage.move_to(m, v(tally_x, y), LEFT);
        }
        play!(stage;
            fade_in(stage.arena_mut(), lit_row, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), dark_row, ORIGIN, 1.0)?.rt(0.5),
        );
        let (mut lit_n, mut dark_n) = (0, 0);
        let mut marks = vec![lit_row, dark_row];
        for &t in &targets {
            let d = ball(stage, Role::card(), 0.1, source);
            stage.add_to_scene(d)?;
            let fly = d
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.35),
                    ..AnimateArgs::default()
                })?
                .move_to(corner(t), ORIGIN)?;
            play!(stage; fly);
            let useless = bucket.can_make(t);
            let landed = if useless { Role::lit() } else { Role::ex_or() };
            let (row, n) = if useless {
                (1.2, &mut lit_n)
            } else {
                (-0.75, &mut dark_n)
            };
            let tally = Dot::new()
                .point(v(tally_x + 0.25 + 0.32 * f64::from(*n), row - 0.4))
                .radius(0.09)
                .color(landed)
                .build();
            *n += 1;
            let tally = stage.arena_mut().add(tally);
            stage.set_fixed_in_frame(tally, 1.0, true);
            marks.push(tally);
            let rgb = [landed.r as f32, landed.g as f32, landed.b as f32];
            play!(stage;
                fade_out(stage.arena_mut(), d, ORIGIN, 1.0)?.rt(0.3),
                fmn::animation::indicate(stage.arena_mut(), balls[t as usize], 1.6, Some(rgb))?.rt(0.3),
                fade_in(stage.arena_mut(), tally, ORIGIN, 1.6)?.rt(0.3),
            );
        }
        // the prediction pause, its "?" pinned to the screen
        kit.hold(stage)?;
        kit.mark(stage, "predict");
        let q = kit.text_hud(stage, "?", 90.0, Palette::hero(), v(5.2, 2.6))?;
        play!(stage; fade_in(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        stage.wait(PREDICT_HOLD_S)?;

        // 04_5: four lit corners out of eight, half
        kit.say(stage, "04_5")?;
        let lit = bucket.span().len();
        let total = 1usize << PIECES;
        let half = kit.text_hud(
            stage,
            &format!(
                "{lit} lit of {total}: in the long run, useless {:.0}% of the time",
                useless_chance(2, PIECES) * 100.0
            ),
            44.0,
            Role::lit(),
            v(0.0, -3.35),
        )?;
        // the eight throws' tally leaves as the long-run half arrives, so
        // the frame never sets 3 of 8 against 4 of 8 (Maya R3 maya-08, Dev D9)
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        fade_list(stage, &marks, 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), half, ORIGIN, 1.0)?.rt(0.8));
        kit.hold(stage)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_corners_are_the_cards_and_the_lit_face_is_the_span_of_a_and_b() {
        // each corner one step from another per piece it lacks
        for c in 0..8u32 {
            for i in 0..PIECES {
                let d = c ^ 1 << i;
                let (p, q) = (corner(c), corner(d));
                let dist: f64 = (0..3).map(|k| (p[k] - q[k]).powi(2)).sum::<f64>().sqrt();
                assert!((dist - SIDE).abs() < 1e-12);
            }
        }
        let mut b = Bucket::new();
        b.add(A);
        b.add(B);
        assert_eq!(b.span(), [0, A, B, A | B], "the bottom face, z = 0 side");
        assert!(b.span().iter().all(|&c| c >> 2 & 1 == 0));
        assert_eq!(useless_chance(2, PIECES), 0.5);
        assert_eq!(name(0b101), "AC");
    }
}
