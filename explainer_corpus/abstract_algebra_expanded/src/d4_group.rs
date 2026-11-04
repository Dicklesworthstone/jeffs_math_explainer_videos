//! Exact arithmetic of the square's eight symmetries, and the shared pictures
//! that chapters 08, 09, 14 and 16 draw from it: rack thumbnails in the
//! chapter 07 style, rigid tiles that perform a move, and mirror lines that
//! travel with a tile. Every highlighted set on screen is computed here.
use crate::tile_model::{self, Matrix};
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;

/// The move r^turns s^flip; the flip (when present) happens first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Elem {
    pub turns: u8,
    pub flip: bool,
}

pub const E: Elem = Elem::new(0, false);
pub const R: Elem = Elem::new(1, false);
pub const R2: Elem = Elem::new(2, false);
pub const R3: Elem = Elem::new(3, false);
pub const S: Elem = Elem::new(0, true);
pub const RS: Elem = Elem::new(1, true);
pub const R2S: Elem = Elem::new(2, true);
pub const R3S: Elem = Elem::new(3, true);

/// The rack order used since chapter 07: rotations above, flips below.
pub const ALL: [Elem; 8] = [E, R, R2, R3, S, RS, R2S, R3S];

impl Elem {
    pub const fn new(turns: u8, flip: bool) -> Self {
        Self { turns: turns % 4, flip }
    }

    /// `self · other`: do `other` first, then `self` (rightmost first).
    /// Uses s r^b = r^(-b) s.
    pub fn mul(self, other: Elem) -> Elem {
        let carried = if self.flip { (4 - other.turns) % 4 } else { other.turns };
        Elem::new(self.turns + carried, self.flip ^ other.flip)
    }

    pub fn inv(self) -> Elem {
        if self.flip {
            self
        } else {
            Elem::new(4 - self.turns, false)
        }
    }

    /// Position in [`ALL`].
    pub fn index(self) -> usize {
        usize::from(self.turns) + if self.flip { 4 } else { 0 }
    }

    /// The on-screen name, matching chapter 13's notation.
    pub fn name(self) -> &'static str {
        ["e", "r", "r²", "r³", "s", "rs", "r²s", "r³s"][self.index()]
    }

    /// The same name as TeX, for products typeset with `kit.tex`.
    pub fn tex(self) -> &'static str {
        ["e", "r", "r^2", "r^3", "s", "rs", "r^2s", "r^3s"][self.index()]
    }

    /// The corner map, through the canonical model in `d4.rs` (the tests
    /// check `mul` against it).
    #[cfg(test)]
    pub fn apply(self, point: (i32, i32)) -> (i32, i32) {
        crate::d4::apply(point, self.turns, self.flip)
    }

    /// The rigid 3-D pose of the tile after this move (chapter 07's matrices).
    pub fn matrix(self) -> Matrix {
        let turn = tile_model::turn(f64::from(self.turns));
        if self.flip {
            tile_model::compose(turn, tile_model::flip(1.0))
        } else {
            turn
        }
    }

    /// A flip's mirror line through the centre, in eighths of a turn:
    /// 0 horizontal, 1 the diagonal y = x, 2 vertical, 3 the diagonal y = -x.
    pub fn mirror(self) -> Option<u8> {
        self.flip.then_some(self.turns)
    }

    /// Where this move carries the line at `eighths` (lines modulo a half-turn).
    pub fn carry_line(self, eighths: u8) -> u8 {
        if self.flip {
            (2 * self.turns + 4 - eighths % 4) % 4
        } else {
            (eighths + 2 * self.turns) % 4
        }
    }
}

/// The move as a motion, `t` from 0 (identity) to 1 (`g`): a rotation turns
/// through its quarter-turns; a flip turns over about its own mirror line.
pub fn motion(g: Elem, t: f64) -> Matrix {
    let t = t.clamp(0.0, 1.0);
    match g.mirror() {
        None => tile_model::turn(f64::from(g.turns) * t),
        Some(eighths) => {
            let half = f64::from(eighths) / 2.0;
            tile_model::compose(
                tile_model::turn(half),
                tile_model::compose(tile_model::flip(t), tile_model::turn(-half)),
            )
        }
    }
}

/// `first`, then `second`, along one tracker: 0 to 1 performs `first`,
/// 1 to 2 performs `second` on the result.
pub fn two_step(first: Elem, second: Elem, t: f64) -> Matrix {
    tile_model::compose(motion(second, t - 1.0), motion(first, t))
}

/// Rounds of the closure machine: round 0 holds the generators; each later
/// round holds the products of every ordered pair that were not yet present.
pub fn closure_rounds(generators: &[Elem]) -> Vec<Vec<Elem>> {
    let mut have: Vec<Elem> = Vec::new();
    for &g in generators {
        if !have.contains(&g) {
            have.push(g);
        }
    }
    let mut rounds = vec![have.clone()];
    loop {
        let mut fresh = Vec::new();
        for &a in &have {
            for &b in &have {
                let c = a.mul(b);
                if !have.contains(&c) && !fresh.contains(&c) {
                    fresh.push(c);
                }
            }
        }
        if fresh.is_empty() {
            return rounds;
        }
        have.extend(fresh.iter().copied());
        rounds.push(fresh);
    }
}

/// Every subgroup, by brute force over the 255 nonempty subsets, smallest
/// first; members listed in rack order.
pub fn subgroups() -> Vec<Vec<Elem>> {
    let mut found: Vec<Vec<Elem>> = (1_u16..256)
        .map(|mask| {
            ALL.iter()
                .copied()
                .filter(|g| mask & (1 << g.index()) != 0)
                .collect::<Vec<_>>()
        })
        .filter(|set: &Vec<Elem>| set.iter().all(|&a| set.iter().all(|&b| set.contains(&a.mul(b.inv())))))
        .collect();
    found.sort_by_key(|set| (set.len(), set.iter().map(|g| g.index()).collect::<Vec<_>>()));
    found
}

pub fn commutes(a: Elem, b: Elem) -> bool {
    a.mul(b) == b.mul(a)
}

pub fn centralizer(x: Elem) -> Vec<Elem> {
    ALL.into_iter().filter(|&g| commutes(g, x)).collect()
}

pub fn center() -> Vec<Elem> {
    ALL.into_iter()
        .filter(|&g| ALL.iter().all(|&h| commutes(g, h)))
        .collect()
}

/// g H, in rack order.
pub fn left_coset(g: Elem, h: &[Elem]) -> Vec<Elem> {
    let set: Vec<Elem> = h.iter().map(|&x| g.mul(x)).collect();
    ALL.into_iter().filter(|x| set.contains(x)).collect()
}

/// H g, in rack order.
pub fn right_coset(g: Elem, h: &[Elem]) -> Vec<Elem> {
    let set: Vec<Elem> = h.iter().map(|&x| x.mul(g)).collect();
    ALL.into_iter().filter(|x| set.contains(x)).collect()
}

/// One mobject holding `members`, so a picture moves and fades as a unit.
pub fn group_of(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let group = stage.arena_mut().add(VMobject::new());
    for &member in members {
        stage.attach(group, member)?;
    }
    Ok(group)
}

pub fn corner_color(corner: usize) -> Srgb {
    [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()][corner]
}

/// A rack thumbnail exactly as chapter 07 draws it, scaled by `k` (1.0 is
/// the rack size, 0.96 across), as one detached group.
pub fn thumb(stage: &mut Stage<'_>, kit: &Kit, g: Elem, at: (f64, f64), k: f64) -> fmn::Result<Mob> {
    let vertices = tile_model::vertices(g.matrix());
    let (x, y) = at;
    let mut parts = vec![
        stage.arena_mut().add(
            Polygon::new(vertices.map(|p| v(x + p[0] * 0.48 * k, y + p[1] * 0.48 * k)))
                .style(stroke_style(Palette::dim(), 2.0))
                .build(),
        ),
    ];
    for (corner, p) in vertices.iter().enumerate() {
        parts.push(
            stage.arena_mut().add(
                Circle::new()
                    .radius(0.1 * k)
                    .style(ink_style(corner_color(corner)))
                    .build()
                    .moved_to(v(x + p[0] * 0.36 * k, y + p[1] * 0.36 * k)),
            ),
        );
        parts.push(kit.text(
            stage,
            &(corner + 1).to_string(),
            16.0 * k,
            Palette::ink(),
            v(x + p[0] * 0.22 * k, y + p[1] * 0.22 * k),
        )?);
    }
    group_of(stage, &parts)
}

/// A rounded frame around a thumbnail slot (membership, verdicts, cosets).
pub fn slot_frame(stage: &mut Stage<'_>, at: (f64, f64), k: f64, color: Srgb) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(1.2 * k)
        .height(1.2 * k)
        .corner_radius(0.14 * k)
        .style(stroke_style(color, 4.0))
        .build()?
        .moved_to(v(at.0, at.1));
    Ok(stage.arena_mut().add(frame))
}

/// A mirror line through a tile's centre that moves with the tile's pose.
#[allow(clippy::too_many_arguments)]
pub fn carried_line(
    stage: &mut Stage<'_>,
    progress: Mob,
    center: f64,
    height: f64,
    scale: f64,
    eighths: u8,
    color: Srgb,
    pose: impl Fn(f64) -> Matrix + Copy + Send + Sync + 'static,
) -> fmn::Result<Mob> {
    let angle = std::f64::consts::FRAC_PI_4 * f64::from(eighths);
    let reach = 1.45;
    let line = stage.always_redraw(move |arena| {
        let matrix = pose(arena.tracker_value(progress).unwrap_or(0.0));
        let ends = [1.0, -1.0].map(|sign| {
            let p = tile_model::apply(matrix, [sign * reach * angle.cos(), sign * reach * angle.sin(), 0.0]);
            let p = tile_model::project(p, 6.0).expect("mirror line stays in front of camera");
            v(center + scale * p[0], height + scale * p[1])
        });
        arena.add(Polygon::new(ends).style(stroke_style(color, 5.0)).build())
    });
    stage.add_to_scene(line)?;
    Ok(line)
}

/// Fade a set in together.
pub fn show(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(run_time);
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

/// Fade a set out together; live (redrawn) members stop updating first so
/// they fade instead of being rebuilt at full opacity.
pub fn hide(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        stage.clear_updaters(mob, true);
        let a = fade_out(stage.arena_mut(), mob, ORIGIN, 1.0)?.rt(run_time);
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

/// Move one group to `to` over `run_time` seconds.
pub fn glide(stage: &mut Stage<'_>, mob: Mob, to: (f64, f64), run_time: f64) -> fmn::Result<()> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            ..AnimateArgs::default()
        })?
        .move_to(v(to.0, to.1), ORIGIN)?;
    let a = stage.prepare(a)?;
    stage.play_prepared(vec![a])?;
    Ok(())
}

/// Move several groups at once, each to its own target.
pub fn glide_all(stage: &mut Stage<'_>, moves: &[(Mob, (f64, f64))], run_time: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &(mob, to) in moves {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(run_time),
                ..AnimateArgs::default()
            })?
            .move_to(v(to.0, to.1), ORIGIN)?;
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORNERS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

    fn same(a: Matrix, b: Matrix) -> bool {
        tile_model::CORNERS.iter().chain([&[0.0, 0.0, 1.0]]).all(|&p| {
            tile_model::apply(a, p)
                .iter()
                .zip(tile_model::apply(b, p))
                .all(|(x, y)| (x - y).abs() < 1e-9)
        })
    }

    #[test]
    fn multiplication_is_composition_of_corner_maps_rightmost_first() {
        for a in ALL {
            for b in ALL {
                for p in CORNERS {
                    assert_eq!(a.mul(b).apply(p), a.apply(b.apply(p)), "{} {}", a.name(), b.name());
                }
            }
            assert_eq!(a.mul(a.inv()), E);
            assert_eq!(a.inv().mul(a), E);
        }
    }

    #[test]
    fn motions_end_at_the_rack_pose_and_two_steps_compose() {
        for g in ALL {
            assert!(same(motion(g, 0.0), tile_model::IDENTITY));
            assert!(same(motion(g, 1.0), g.matrix()), "{}", g.name());
            for h in ALL {
                assert!(same(two_step(g, h, 2.0), h.mul(g).matrix()));
            }
        }
        // A flip turns the tile over; a rotation keeps its face.
        assert!(same(motion(RS, 1.0), RS.matrix()));
    }

    #[test]
    fn closure_machine_rounds_match_the_narration() {
        assert_eq!(closure_rounds(&[S]), vec![vec![S], vec![E]]);
        assert_eq!(closure_rounds(&[S, R2]), vec![vec![S, R2], vec![E, R2S]]);
        let full = closure_rounds(&[R, S]);
        assert_eq!(full.iter().map(Vec::len).collect::<Vec<_>>(), vec![2, 4, 2]);
        assert_eq!(full.concat().len(), 8);
        assert_eq!(closure_rounds(&[E, R, R2]), vec![vec![E, R, R2], vec![R3]]);
    }

    #[test]
    fn exactly_ten_subgroups_with_sizes_dividing_eight() {
        let all = subgroups();
        let sizes: Vec<usize> = all.iter().map(Vec::len).collect();
        assert_eq!(sizes, vec![1, 2, 2, 2, 2, 2, 4, 4, 4, 8]);
        assert!(all.contains(&vec![E, R2, S, R2S]));
        // Closure alone already identifies them (finite subgroup test).
        for mask in 1_u16..256 {
            let set: Vec<Elem> = ALL.into_iter().filter(|g| mask & (1 << g.index()) != 0).collect();
            let closed = set.iter().all(|&a| set.iter().all(|&b| set.contains(&a.mul(b))));
            assert_eq!(closed, all.contains(&set));
        }
    }

    #[test]
    fn centralizer_is_the_mirror_preserving_subgroup_and_center_is_e_r2() {
        let c = centralizer(S);
        assert_eq!(c, vec![E, R2, S, R2S]);
        for g in ALL {
            assert_eq!(g.carry_line(0) == 0, c.contains(&g));
            // Undo g, flip, redo g: the flip across the carried line.
            assert_eq!(g.mul(S).mul(g.inv()).mirror(), Some(g.carry_line(0)));
        }
        assert_eq!(center(), vec![E, R2]);
        let keeps_all = ALL
            .into_iter()
            .filter(|g| (0..4).all(|m| g.carry_line(m) == m))
            .collect::<Vec<_>>();
        assert_eq!(keeps_all, center());
        for x in ALL {
            let cx = centralizer(x);
            assert!(cx.iter().all(|&a| cx.iter().all(|&b| cx.contains(&a.mul(b.inv())))));
        }
    }

    #[test]
    fn left_and_right_cosets_of_the_flip_subgroup_differ() {
        let h = [E, S];
        assert_eq!(left_coset(R, &h), vec![R, RS]);
        assert_eq!(right_coset(R, &h), vec![R, R3S]);
        assert_eq!(left_coset(R2, &h), right_coset(R2, &h));
        let rotations = [E, R, R2, R3];
        for g in ALL {
            assert_eq!(left_coset(g, &rotations), right_coset(g, &rotations));
        }
    }
}
