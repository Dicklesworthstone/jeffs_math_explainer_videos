//! Chapter 4 — into rank space, on the article's worked example: ten
//! people's heights and weights become ranks (ties share the average),
//! the scatter moves into rank coordinates, and an outlier stops mattering.

use fmn::animation::{indicate, replacement_transform, transform_from_copy};
use fmn::prelude::*;

use crate::kit::{
    Frame2, Kit, Palette, Timed, children, dot_cloud, fade_all, lagged, point_colors, v,
};
use crate::play;
use crate::stats::{self, HEIGHTS, WEIGHTS};

pub struct Ranks {
    pub kit: Kit,
}

const CELL_X0: f64 = -3.4;
const CELL_DX: f64 = 1.0;
const ROWS: [f64; 4] = [2.55, 1.95, 1.0, 0.4];

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

impl SceneConstruct for Ranks {
    fn name(&self) -> &str {
        "04_ranks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let h = stats::hoeffding(&HEIGHTS, &WEIGHTS);
        let colors = point_colors(10);
        let title = kit.text(
            stage,
            "Step 1:  replace values with their ranks",
            40.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        kit.say(stage, "r1")?;
        play!(stage; write(stage.arena(), title).rt(1.3));

        // ---- the raw table
        let labels = [
            ("height", "X", Palette::ink()),
            ("weight", "Y", Palette::ink()),
            ("rank of X", "R", Palette::x_rank()),
            ("rank of Y", "S", Palette::y_rank()),
        ];
        let mut row_labels = Vec::new();
        for (i, &(word, sym, color)) in labels.iter().enumerate() {
            let w = kit.text_left(stage, word, 28.0, color, v(-6.75, ROWS[i]))?;
            let s = kit.tex(stage, sym, 32.0, color, &[], v(-4.3, ROWS[i]))?;
            row_labels.push((w, s));
        }
        let cell = |stage: &mut Stage<'_>,
                    row: usize,
                    k: usize,
                    s: &str,
                    color: Srgb|
         -> fmn::Result<Mob> {
            kit.text(
                stage,
                s,
                30.0,
                color,
                v(CELL_X0 + CELL_DX * k as f64, ROWS[row]),
            )
        };
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for k in 0..10 {
            xs.push(cell(stage, 0, k, &fmt(HEIGHTS[k]), Palette::ink())?);
            ys.push(cell(stage, 1, k, &fmt(WEIGHTS[k]), Palette::ink())?);
        }
        for (row, cells) in [(0usize, &xs), (1, &ys)] {
            let (w, s) = row_labels[row];
            play!(stage; fade_in(stage.arena_mut(), w, ORIGIN, 1.0)?.rt(0.5), fade_in(stage.arena_mut(), s, ORIGIN, 1.0)?.rt(0.5));
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &c in cells.iter() {
                let a = fade_in(stage.arena_mut(), c, v(0.0, 0.15), 1.0)?;
                anims.push(stage.prepare(a)?);
            }
            let g = lagged(stage, anims, 0.12)?;
            play!(stage; g.rt(1.2));
        }

        // ---- the raw scatter
        let raw = Frame2 {
            center: v(-3.4, -2.25),
            width: 5.2,
            height: 2.75,
            x: (52.0, 81.0),
            y: (110.0, 265.0),
        };
        let rank = Frame2 {
            center: v(3.55, -2.25),
            width: 5.2,
            height: 2.75,
            x: (0.0, 10.2),
            y: (0.0, 10.2),
        };
        let raw_pts: Vec<Vec3> = (0..10).map(|k| raw.p(HEIGHTS[k], WEIGHTS[k])).collect();
        let rank_pts: Vec<Vec3> = (0..10).map(|k| rank.p(h.r[k], h.s[k])).collect();
        let raw_border = stage.arena_mut().add(raw.border(Palette::faint())?);
        let raw_x = kit.text(
            stage,
            "height",
            24.0,
            Palette::dim(),
            v(raw.center[0], raw.bottom() - 0.27),
        )?;
        let raw_y = kit.text(
            stage,
            "weight",
            24.0,
            Palette::dim(),
            v(raw.left() + 0.5, raw.top() + 0.22),
        )?;
        let raw_cloud = stage.arena_mut().add(dot_cloud(&raw_pts, &colors, 0.085));
        let raw_x3 = kit.text(
            stage,
            "×3",
            22.0,
            Palette::dim(),
            crate::kit::add(raw_pts[8], v(0.32, 0.0)),
        )?;
        kit.say(stage, "r2")?;
        play!(stage;
            show_creation(raw_border).rt(0.8),
            fade_in(stage.arena_mut(), raw_x, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), raw_y, ORIGIN, 1.0)?,
        );
        play!(stage;
            fade_in(stage.arena_mut(), raw_cloud, ORIGIN, 0.2)?.rt(1.4).lag(0.15),
            fade_in(stage.arena_mut(), raw_x3, ORIGIN, 1.0)?.rt(1.4),
        );
        stage.wait(0.8)?;

        // ---- values become ranks
        let mut rs = Vec::new();
        let mut ss = Vec::new();
        for k in 0..10 {
            rs.push(cell(stage, 2, k, &fmt(h.r[k]), Palette::x_rank())?);
            ss.push(cell(stage, 3, k, &fmt(h.s[k]), Palette::y_rank())?);
        }
        kit.say(stage, "r3")?;
        for (row, src, dst) in [(2usize, &xs, &rs), (3, &ys, &ss)] {
            let (w, s) = row_labels[row];
            play!(stage; fade_in(stage.arena_mut(), w, ORIGIN, 1.0)?.rt(0.5), fade_in(stage.arena_mut(), s, ORIGIN, 1.0)?.rt(0.5));
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for (&a, &b) in src.iter().zip(dst.iter()) {
                let t = transform_from_copy(stage.arena_mut(), a, b)?;
                anims.push(stage.prepare(t)?);
            }
            let g = lagged(stage, anims, 0.1)?;
            play!(stage; g.rt(1.8));
        }
        stage.wait(0.5)?;

        // ---- ties share the average rank
        let tie_box_x = SurroundingRectangle::from_extent(Some((
            v(CELL_X0 + 7.0 * CELL_DX - 0.4, ROWS[0] - 0.25),
            v(CELL_X0 + 9.0 * CELL_DX + 0.4, ROWS[0] + 0.25),
        )))
        .color(Palette::q())
        .build();
        let tie_box_r = SurroundingRectangle::from_extent(Some((
            v(CELL_X0 + 7.0 * CELL_DX - 0.4, ROWS[2] - 0.25),
            v(CELL_X0 + 9.0 * CELL_DX + 0.4, ROWS[2] + 0.25),
        )))
        .color(Palette::q())
        .build();
        let tie_box_x = stage.arena_mut().add(tie_box_x);
        let tie_box_r = stage.arena_mut().add(tie_box_r);
        let tie_note = kit.tex(
            stage,
            r"\text{three-way tie for places 8, 9, 10:}\quad \frac{8+9+10}{3} = 9",
            30.0,
            Palette::q(),
            &[],
            v(1.4, -0.3),
        )?;
        kit.say(stage, "r4")?;
        play!(stage; show_creation(tie_box_x).rt(0.8), show_creation(tie_box_r).rt(0.8));
        kit.chime(stage, 587.33)?;
        play!(stage; fade_in(stage.arena_mut(), tie_note, v(0.0, 0.2), 1.0)?.rt(1.0));
        stage.wait(2.0)?;
        play!(stage;
            fade_out(stage.arena_mut(), tie_box_x, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), tie_box_r, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), tie_note, ORIGIN, 1.0)?,
        );

        // ---- the same people in rank coordinates
        let rank_border = stage.arena_mut().add(rank.border(Palette::faint())?);
        let rank_x = kit.tex(
            stage,
            "R",
            28.0,
            Palette::x_rank(),
            &[],
            v(rank.center[0], rank.bottom() - 0.27),
        )?;
        let rank_y = kit.tex(
            stage,
            "S",
            28.0,
            Palette::y_rank(),
            &[],
            v(rank.left() - 0.25, rank.top() - 0.2),
        )?;
        let rank_cloud = stage.arena_mut().add(dot_cloud(&rank_pts, &colors, 0.085));
        let rank_x3 = kit.text(
            stage,
            "×3",
            22.0,
            Palette::dim(),
            crate::kit::add(rank_pts[8], v(0.32, 0.0)),
        )?;
        let bridge = Arrow::new(v(-0.55, -2.25), v(0.75, -2.25))
            .buff(0.0)
            .color(Palette::dim())
            .build()?;
        let bridge = stage.arena_mut().add(bridge);
        kit.say(stage, "r5")?;
        play!(stage;
            show_creation(rank_border).rt(0.8),
            fade_in(stage.arena_mut(), rank_x, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), rank_y, ORIGIN, 1.0)?,
            fmn::animation::grow_arrow(stage.arena_mut(), bridge)?,
        );
        play!(stage;
            transform_from_copy(stage.arena_mut(), raw_cloud, rank_cloud)?.rt(2.0),
            fade_in(stage.arena_mut(), rank_x3, ORIGIN, 1.0)?.rt(2.0),
        );
        let same = kit.text(
            stage,
            "same people, new coordinates",
            28.0,
            Palette::dim(),
            v(0.1, -0.3),
        )?;
        play!(stage; fade_in(stage.arena_mut(), same, v(0.0, 0.15), 1.0)?.rt(0.8));
        stage.wait(1.5)?;

        // ---- an outlier: the raw point jumps, the rank does not move
        let raw_dots = children(stage, raw_cloud);
        let rank_dots = children(stage, rank_cloud);
        let wild = 248.0;
        let new_y = kit.text(
            stage,
            &fmt(wild),
            30.0,
            Palette::warn(),
            v(CELL_X0 + 4.0 * CELL_DX, ROWS[1]),
        )?;
        let mover = raw_dots[4];
        let moved = mover.animate().move_to(raw.p(HEIGHTS[4], wild), ORIGIN)?;
        kit.say(stage, "r6")?;
        play!(stage, 1.6;
            replacement_transform(ys[4], new_y),
            moved,
        );
        let robust = kit.text(
            stage,
            "a wild value moves the raw point; its rank stays 7",
            28.0,
            Palette::q(),
            v(0.1, -0.3),
        )?;
        play!(stage; fade_out(stage.arena_mut(), same, ORIGIN, 1.0)?.rt(0.4));
        play!(stage;
            indicate(stage.arena_mut(), ss[4], 1.4, None)?.rt(1.4),
            indicate(stage.arena_mut(), rank_dots[4], 1.8, None)?.rt(1.4),
            fade_in(stage.arena_mut(), robust, v(0.0, 0.2), 1.0)?.rt(1.0),
        );
        stage.wait(1.4)?;
        let moral = kit.text(
            stage,
            "Ranks keep only the order, so no single value can dominate.",
            30.0,
            Palette::ink(),
            v(0.1, -0.3),
        )?;
        kit.say(stage, "r7")?;
        play!(stage; replacement_transform(robust, moral).rt(1.0));
        stage.wait(1.2)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 3)?;
        Ok(())
    }
}
