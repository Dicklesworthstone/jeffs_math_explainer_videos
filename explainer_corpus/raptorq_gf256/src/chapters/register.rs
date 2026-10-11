//! The byte register of chapters 07, 09 and 10: eight slots under their
//! place values, one tile per bit, and a dark gutter left of the top slot.
//! Doubling a byte slides every tile one slot left; the top bit's tile
//! drops into the gutter (07_3: wrap-around loses it) and the bottom slot
//! takes a fresh zero. In 10 the fallen bit climbs back out as the fold.

use explainer_kit::{Kit, Palette, Timed, hex, play, v};
use fmn::prelude::*;

use super::draw::{Role, group_of};

/// Group mobjects that are already on stage, and put the group on stage in
/// their place (adding a group first removes its members' own entries from
/// the draw list, as manim's `Scene.add` does). The group lasts until one
/// member is animated on its own: that re-adds the member, which splits the
/// group back into its members.
pub fn regroup(stage: &mut Stage<'_>, members: &[Mob]) -> fmn::Result<Mob> {
    let g = group_of(stage, members)?;
    stage.add_to_scene(g)?;
    Ok(g)
}

/// The fill behind an unlit tile, a shade above the background.
fn dark() -> Srgb {
    hex("#1A2130")
}

/// One bit as a tile `cell` wide at `at`: a lit square with a `1`, or a dark
/// one with a dim `0`.
pub fn tile(kit: &Kit, stage: &mut Stage<'_>, one: bool, cell: f64, lit: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let (fill, opacity, edge) = if one {
        (lit, 0.32, lit)
    } else {
        (dark(), 0.95, Palette::faint())
    };
    let square = Rectangle::new()
        .width(cell * 0.84)
        .height(cell * 0.84)
        .corner_radius(cell * 0.08)
        .style(Style::default().fill(fill, opacity).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(at);
    let square = stage.arena_mut().add(square);
    let ink = if one { lit } else { Palette::dim() };
    let digit = kit.mono(stage, if one { "1" } else { "0" }, cell * 46.0, ink, at)?;
    group_of(stage, &[square, digit])
}

/// Eight slots, their place values (128 … 1) and the gutter; the tiles sit
/// in the slots and move.
pub struct Register {
    pub centre: Vec3,
    pub cell: f64,
    pub lit: Srgb,
    /// The bits left to right (top bit first).
    pub bits: [bool; 8],
    /// The tile in each slot, left to right.
    pub tiles: Vec<Mob>,
    /// Slots, place values and gutter (the frame; not the tiles).
    pub frame: Mob,
}

impl Register {
    /// The bits of `value` in slots `cell` wide centred at `centre`, lit in
    /// `lit`; place values above when `places`.
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        value: u8,
        cell: f64,
        centre: Vec3,
        lit: Srgb,
        places: bool,
    ) -> fmn::Result<Self> {
        let bits: [bool; 8] = std::array::from_fn(|k| value >> (7 - k) & 1 == 1);
        let slot_at = |k: usize| v(centre[0] + (k as f64 - 3.5) * cell, centre[1]);
        let (mut slots, mut labels, mut tiles) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..8 {
            let at = slot_at(k);
            let slot = Rectangle::new()
                .width(cell * 0.96)
                .height(cell * 0.96)
                .corner_radius(cell * 0.1)
                .style(
                    Style::default()
                        .fill(hex("#0B0E13"), 0.9)
                        .stroke(Palette::faint(), 1.5, 1.0),
                )
                .build()?
                .moved_to(at);
            slots.push(stage.arena_mut().add(slot));
            if places {
                let label = (1u32 << (7 - k)).to_string();
                labels.push(kit.text(stage, &label, cell * 26.0, Palette::dim(), v(at[0], at[1] + 0.8 * cell))?);
            }
        }
        let g = gutter_at(centre, cell);
        // a trough open at the top: a floor and two walls, faint
        let (w, h) = (cell * 0.5, cell * 0.62);
        let trough = [
            v(g[0] - w, g[1] + h),
            v(g[0] - w, g[1] - h * 0.62),
            v(g[0] + w, g[1] - h * 0.62),
            v(g[0] + w, g[1] + h),
        ];
        let gutter = Polygon::polyline(trough)
            .style(
                Style::default()
                    .fill(hex("#07090C"), 0.9)
                    .stroke(Palette::faint(), 2.5, 1.0),
            )
            .build();
        let gutter = stage.arena_mut().add(gutter);
        let mut all = slots.clone();
        all.extend(&labels);
        all.push(gutter);
        let frame = group_of(stage, &all)?;
        for (k, &one) in bits.iter().enumerate() {
            tiles.push(tile(kit, stage, one, cell, lit, slot_at(k))?);
        }
        Ok(Self {
            centre,
            cell,
            lit,
            bits,
            tiles,
            frame,
        })
    }

    /// The centre of slot `k` (0 = the top bit, at the left); slot 8 is one
    /// past the right end, where a fresh zero comes from.
    pub fn slot(&self, k: usize) -> Vec3 {
        v(self.centre[0] + (k as f64 - 3.5) * self.cell, self.centre[1])
    }

    /// Where a fallen bit comes to rest, in the gutter left of slot 0.
    pub fn gutter_point(&self) -> Vec3 {
        gutter_at(self.centre, self.cell)
    }

    /// The byte the tiles show now.
    pub fn value(&self) -> u8 {
        self.bits.iter().fold(0, |acc, &b| acc << 1 | u8::from(b))
    }

    /// Everything, tiles included, as one new group (for fading or moving
    /// the whole register).
    pub fn everything(&self, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
        let mut all = vec![self.frame];
        all.extend(&self.tiles);
        regroup(stage, &all)
    }

    /// Bring the frame and the tiles in.
    pub fn show(&self, stage: &mut Stage<'_>, run_time: f64) -> fmn::Result<()> {
        play!(stage; fade_in(stage.arena_mut(), self.frame, ORIGIN, 1.0)?.rt(run_time));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &self.tiles {
            let a = fade_in(stage.arena_mut(), t, v(0.0, -0.15 * self.cell), 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let g = explainer_kit::lagged(stage, anims, 0.08)?;
        stage.play(g.rt(run_time))?;
        Ok(())
    }

    /// Double the byte: every tile slides one slot left, the top bit's tile
    /// drops into the gutter, and a zero slides into the bottom slot.
    /// Returns the fallen tile: on screen in the gutter, red, when it was a
    /// one; faded away when it was a zero.
    pub fn double(&mut self, kit: &Kit, stage: &mut Stage<'_>, run_time: f64) -> fmn::Result<Mob> {
        let fresh = tile(kit, stage, false, self.cell, self.lit, self.slot(7))?;
        let fallen = self.tiles[0];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for k in 1..8 {
            let a = self.tiles[k]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(run_time),
                    ..AnimateArgs::default()
                })?
                .move_to(self.slot(k - 1), ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        // a one that falls is lost (it turns the lost role's red); a zero
        // that falls loses nothing and simply fades on its way down
        if self.bits[0] {
            let drop = fallen
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(run_time * 1.25),
                    path_arc: Some(0.9),
                    ..AnimateArgs::default()
                })?
                .move_to(self.gutter_point(), ORIGIN)?
                .set_color(Role::lost())?;
            anims.push(stage.prepare(drop)?);
        } else {
            let shift = sub(self.gutter_point(), self.slot(0));
            let drop = fade_out(stage.arena_mut(), fallen, shift, 1.0)?;
            anims.push(stage.prepare(drop.rt(run_time))?);
        }
        // fade_in starts the tile `shift` back from where it ends: one slot right
        let enter = fade_in(stage.arena_mut(), fresh, v(-self.cell, 0.0), 1.0)?;
        anims.push(stage.prepare(enter.rt(run_time))?);
        stage.play_prepared(anims)?;
        self.tiles.remove(0);
        self.tiles.push(fresh);
        self.bits.rotate_left(1);
        self.bits[7] = false;
        Ok(fallen)
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Where a fallen bit rests for a register centred at `centre`.
fn gutter_at(centre: Vec3, cell: f64) -> Vec3 {
    v(centre[0] - 4.65 * cell, centre[1] - 0.25 * cell)
}
