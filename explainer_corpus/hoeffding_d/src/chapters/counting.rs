//! Chapter 5 — the bivariate count Q: for each point, how many others sit
//! strictly below-and-left in rank space (ties earn partial credit), and
//! how that compares with what independence would predict.

use fmn::animation::{grow_from_point, indicate};
use fmn::prelude::*;

use crate::kit::{Frame2, Kit, Palette, Timed, fade_all, lagged, point_colors, stroke_style, v};
use crate::play;
use crate::stats::{self, HEIGHTS, WEIGHTS};

pub struct Counting {
    pub kit: Kit,
}

const Q_X0: f64 = 1.55;
const Q_DX: f64 = 0.55;
const Q_Y: f64 = -2.95;

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

/// Visual offsets so the three tied people at (9, 9) are all visible.
fn nudge(k: usize) -> (f64, f64) {
    match k {
        7 => (-0.16, 0.1),
        8 => (0.16, 0.1),
        9 => (0.0, -0.16),
        _ => (0.0, 0.0),
    }
}

impl SceneConstruct for Counting {
    fn name(&self) -> &str {
        "05_counting"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let h = stats::hoeffding(&HEIGHTS, &WEIGHTS);
        let n = h.r.len();
        let colors = point_colors(n);
        let title = kit.text(
            stage,
            "Step 2:  count who sits below and to the left",
            40.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        kit.say(stage, "c1")?;
        play!(stage; write(stage.arena(), title).rt(1.3));

        // ---- rank space, with a faint integer grid
        let f = Frame2 {
            center: v(-3.5, -0.3),
            width: 5.9,
            height: 5.9,
            x: (0.0, 10.0),
            y: (0.0, 10.0),
        };
        let border = stage.arena_mut().add(f.border(Palette::dim())?);
        let mut grid = Vec::new();
        for i in 1..10 {
            let x = i as f64;
            grid.push(
                Line::new(f.p(x, 0.0), f.p(x, 10.0))
                    .style(stroke_style(Palette::faint(), 1.0))
                    .build()?,
            );
            grid.push(
                Line::new(f.p(0.0, x), f.p(10.0, x))
                    .style(stroke_style(Palette::faint(), 1.0))
                    .build()?,
            );
        }
        let grid = stage.arena_mut().add(v_group(grid));
        let mut ticks = Vec::new();
        for i in 1..10 {
            let x = i as f64;
            ticks.push(kit.text(
                stage,
                &format!("{i}"),
                20.0,
                Palette::x_rank(),
                crate::kit::add(f.p(x, 0.0), v(0.0, -0.22)),
            )?);
            ticks.push(kit.text(
                stage,
                &format!("{i}"),
                20.0,
                Palette::y_rank(),
                crate::kit::add(f.p(0.0, x), v(-0.22, 0.0)),
            )?);
        }
        let rl = kit.tex(
            stage,
            "R",
            30.0,
            Palette::x_rank(),
            &[],
            v(f.right() + 0.25, f.bottom()),
        )?;
        let sl = kit.tex(
            stage,
            "S",
            30.0,
            Palette::y_rank(),
            &[],
            v(f.left(), f.top() + 0.28),
        )?;
        play!(stage;
            show_creation(border).rt(1.0),
            show_creation(grid).rt(1.4).lag(0.02),
            fade_in(stage.arena_mut(), rl, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), sl, ORIGIN, 1.0)?,
        );
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &ticks {
            let a = fade_in(stage.arena_mut(), t, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.05)?;
        play!(stage; g.rt(0.8));

        let mut dots = Vec::new();
        for k in 0..n {
            let (dx, dy) = nudge(k);
            let d = Dot::new()
                .point(f.p(h.r[k] + dx, h.s[k] + dy))
                .radius(0.11)
                .color(colors[k])
                .build();
            dots.push(stage.arena_mut().add(d));
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &d in &dots {
            let a = fmn::animation::grow_from_center(stage.arena_mut(), d, None)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.15)?;
        play!(stage; g.rt(1.4));

        // ---- the definition
        let rgb = [
            ("Q", Palette::q()),
            ("R", Palette::x_rank()),
            ("S", Palette::y_rank()),
        ];
        let def = kit.tex(
            stage,
            r"Q_i = 1 + \#\{\, j : R_j < R_i,\ S_j < S_i \,\}",
            36.0,
            Palette::ink(),
            &rgb,
            v(3.75, 2.45),
        )?;
        let fine = kit.text(
            stage,
            "(a tie in one coordinate counts ½; an exact twin counts ¼)",
            22.0,
            Palette::dim(),
            v(3.75, 1.8),
        )?;
        kit.say(stage, "c2")?;
        play!(stage; write(stage.arena(), def).rt(1.6));
        play!(stage; fade_in(stage.arena_mut(), fine, ORIGIN, 1.0)?.rt(0.8));

        // ---- the Q vector, filled in as we go
        let q_label = kit.tex(stage, "Q =", 34.0, Palette::q(), &[], v(0.95, Q_Y))?;
        play!(stage; fade_in(stage.arena_mut(), q_label, ORIGIN, 1.0)?.rt(0.6));
        let mut q_cells: Vec<Option<Mob>> = vec![None; n];

        // Point 5 (R = 7, S = 7), in full, with the independence baseline.
        kit.say(stage, "c3")?;
        self.sweep(stage, &f, &h, &dots, &mut q_cells, 4, Detail::Full)?;
        // Point 4 (R = 6, S = 4).
        kit.say(stage, "c5")?;
        self.sweep(stage, &f, &h, &dots, &mut q_cells, 3, Detail::Brief)?;
        // A tied point (R = 9, S = 9): seven strictly below-left, two twins.
        kit.say(stage, "c6")?;
        self.sweep(stage, &f, &h, &dots, &mut q_cells, 7, Detail::Tie)?;
        // The rest, quickly.
        kit.say(stage, "c7")?;
        for k in [0, 1, 2, 5, 6, 8, 9] {
            self.sweep(stage, &f, &h, &dots, &mut q_cells, k, Detail::Quick)?;
        }

        let cells: Vec<Mob> = q_cells.iter().flatten().copied().collect();
        let all_q = SurroundingRectangle::from_extent(Some((
            v(0.55, Q_Y - 0.3),
            v(Q_X0 + Q_DX * 9.0 + 0.35, Q_Y + 0.3),
        )))
        .color(Palette::q())
        .build();
        let all_q = stage.arena_mut().add(all_q);
        kit.say(stage, "c8")?;
        kit.chime(stage, 659.25)?;
        play!(stage; show_creation(all_q).rt(0.8));
        let gap = kit.text(
            stage,
            "observed Q  vs.  what independence predicts:",
            28.0,
            Palette::ink(),
            v(3.75, 0.2),
        )?;
        let gap2 = kit.text(
            stage,
            "that gap is what D adds up",
            28.0,
            Palette::hoeffding(),
            v(3.75, -0.35),
        )?;
        play!(stage; write(stage.arena(), gap).rt(1.2));
        play!(stage; write(stage.arena(), gap2).rt(1.0));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &c in &cells {
            let a = indicate(stage.arena_mut(), c, 1.25, None)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.1)?;
        play!(stage; g.rt(1.6));
        stage.wait(1.2)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 4)?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Detail {
    Full,
    Brief,
    Tie,
    Quick,
}

impl Counting {
    #[allow(clippy::too_many_arguments)]
    fn sweep(
        &self,
        stage: &mut Stage<'_>,
        f: &Frame2,
        h: &stats::Hoeffding,
        dots: &[Mob],
        q_cells: &mut [Option<Mob>],
        i: usize,
        detail: Detail,
    ) -> fmn::Result<()> {
        let kit = &self.kit;
        let n = h.r.len();
        let (ri, si) = (h.r[i], h.s[i]);
        let corner = v(f.left(), f.bottom());
        let w = f.p(ri, si)[0] - f.left();
        let hgt = f.p(ri, si)[1] - f.bottom();
        let quad = crate::kit::rect(
            corner,
            w,
            hgt,
            Style::default()
                .fill(Palette::q(), 0.13)
                .stroke(Palette::q(), 2.5, 0.9),
        )?;
        let quad = stage.arena_mut().add(quad);
        let inside: Vec<usize> = (0..n).filter(|&j| h.r[j] < ri && h.s[j] < si).collect();
        let quick = detail == Detail::Quick;
        let rt = if quick { 0.45 } else { 1.0 };

        play!(stage;
            indicate(stage.arena_mut(), dots[i], 1.6, None)?.rt(rt),
            grow_from_point(stage.arena_mut(), quad, corner, None)?.rt(rt),
        );
        let mut texts = Vec::new();
        if !quick {
            let rs = [
                ("R", Palette::x_rank()),
                ("S", Palette::y_rank()),
                ("Q", Palette::q()),
            ];
            let at = kit.tex(
                stage,
                &format!(r"R_i = {},\ \ S_i = {}", fmt(ri), fmt(si)),
                34.0,
                Palette::ink(),
                &rs,
                v(3.75, 1.0),
            )?;
            play!(stage; fade_in(stage.arena_mut(), at, v(0.0, 0.15), 1.0)?.rt(0.6));
            texts.push(at);
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &j in &inside {
                let a = indicate(stage.arena_mut(), dots[j], 1.5, Some([1.0, 1.0, 0.0]))?;
                anims.push(stage.prepare(a)?);
            }
            let g = lagged(stage, anims, 0.25)?;
            let count = kit.text(
                stage,
                &format!("{} points strictly below-left", inside.len()),
                30.0,
                Palette::q(),
                v(3.75, 0.3),
            )?;
            play!(stage; g.rt(1.4), fade_in(stage.arena_mut(), count, ORIGIN, 1.0)?.rt(1.4));
            texts.push(count);
            let qsrc = if detail == Detail::Tie {
                format!(
                    r"Q_i = 1 + {} + \tfrac14 \cdot 2 = {}",
                    inside.len(),
                    fmt(h.q[i])
                )
            } else {
                format!(r"Q_i = 1 + {} = {}", inside.len(), fmt(h.q[i]))
            };
            let qt = kit.tex(stage, &qsrc, 40.0, Palette::ink(), &rs, v(3.75, -0.45))?;
            play!(stage; write(stage.arena(), qt).rt(1.0));
            texts.push(qt);
            if detail == Detail::Full {
                let expect = ((ri - 1.0) * (si - 1.0)) / (n as f64 - 1.0);
                let base = kit.tex(
                    stage,
                    &format!(
                        r"\text{{if independent:}}\ \ 1 + \frac{{(R_i-1)(S_i-1)}}{{N-1}} = 1 + \frac{{{}}}{{{}}} = {}",
                        fmt((ri - 1.0) * (si - 1.0)),
                        n - 1,
                        fmt(1.0 + expect)
                    ),
                    26.0,
                    Palette::dim(),
                    &rs,
                    v(3.7, -1.35),
                )?;
                kit.say(stage, "c4")?;
                play!(stage; fade_in(stage.arena_mut(), base, v(0.0, 0.15), 1.0)?.rt(1.0));
                texts.push(base);
                let more = kit.text(
                    stage,
                    "more clustered than chance would give",
                    26.0,
                    Palette::good(),
                    v(3.75, -2.05),
                )?;
                play!(stage; fade_in(stage.arena_mut(), more, ORIGIN, 1.0)?.rt(0.8));
                texts.push(more);
                stage.wait(2.2)?;
            } else if detail == Detail::Tie {
                let twins = kit.text(
                    stage,
                    "two exact twins share the spot: ¼ each",
                    26.0,
                    Palette::dim(),
                    v(3.75, -1.25),
                )?;
                play!(stage; fade_in(stage.arena_mut(), twins, ORIGIN, 1.0)?.rt(0.8));
                texts.push(twins);
                stage.wait(1.6)?;
            } else {
                stage.wait(1.0)?;
            }
        }

        if !quick {
            kit.hold(stage)?;
        }
        // The cell joins the Q vector.
        let cell = kit.text(
            stage,
            &fmt(h.q[i]),
            30.0,
            Palette::q(),
            v(Q_X0 + Q_DX * i as f64, Q_Y),
        )?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = fade_in(stage.arena_mut(), cell, v(0.0, 0.3), 1.0)?;
        anims.push(stage.prepare(a)?);
        let a = fade_out(stage.arena_mut(), quad, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
        for &t in &texts {
            let a = fade_out(stage.arena_mut(), t, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(if quick { 0.45 } else { 0.8 }),
                ..PlayOverrides::default()
            },
        )?;
        q_cells[i] = Some(cell);
        Ok(())
    }
}
