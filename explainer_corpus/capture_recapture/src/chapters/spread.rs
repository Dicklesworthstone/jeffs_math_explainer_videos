//! Chapter 03, why the marks spread: the soup (03_1), the marked thousand
//! let go at the dock (03_2), the regions tinted by their marked share
//! (03_3), the time-lapse of the mixing the oracle checks (03_4: the scene
//! runs `model::mixing` itself), real nets of a thousand (03_5), two whole
//! corners (03_6), and the mosaic's straight line (03_7–03_8).

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::props::{Pose, RowStrips, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, dot_cloud, fade_all, hex, play, thousands, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{fish, line_style};
use super::lake::{
    LAKE_H, LAKE_W, PER_GLYPH, TABLE_C, TABLE_H, TABLE_W, cell_backs, lake_p, landmarks, table_cells, table_labels,
    water,
};
use crate::facts::FELLER;
use crate::model::Lake;
use crate::model::mixing::{self, Mixing};

/// The time-lapse: all 40,000 steps (t = 0 … 400) in this many seconds.
const TIME_LAPSE_S: f64 = 12.0;
/// How fast a drawn fish follows its simulated fish (per frame): the
/// simulated paths are Brownian; the drawn ones are smoothed ("time-lapse").
const FOLLOW: f64 = 0.18;
/// The regions' names, in the oracle's order.
const REGIONS: [&str; 9] = ["SW", "S", "SE", "W", "C", "E", "NW", "N", "NE"];

/// The water colour, under the tints.
fn water_colour() -> Srgb {
    hex("#13253A")
}

/// A region's tint for its marked share: water at 0, hot gold at the
/// dock's starting two in three.
fn tint(share: f64) -> Srgb {
    interpolate_color(water_colour(), Palette::gold(), (share / 0.66).clamp(0.0, 1.0))
}

/// The 3 × 3 tint colours, rows north to south (RowStrips' order).
fn tints(m: &Mixing) -> Vec<Vec<Srgb>> {
    let (all, marked) = m.regions();
    (0..3)
        .rev()
        .map(|ry| {
            (0..3)
                .map(|rx| {
                    let r = rx + 3 * ry;
                    tint(if all[r] > 0 {
                        f64::from(marked[r]) / f64::from(all[r])
                    } else {
                        0.0
                    })
                })
                .collect()
        })
        .collect()
}

/// The drawn fish's poses from the simulation (each drawn fish is one of
/// the simulated ones), facing east.
fn poses_of(m: &Mixing, ids: &[usize]) -> Vec<Pose> {
    ids.iter()
        .map(|&i| {
            let p = lake_p(m.x[i], m.y[i]);
            Pose {
                x: p[0],
                y: p[1],
                heading: 0.0,
            }
        })
        .collect()
}

/// Let a swarm follow its simulated fish, smoothed, turning to face where
/// it is going; `advance` is called first each frame (the driver swarm
/// advances the shared simulation).
fn follow(
    stage: &mut Stage<'_>,
    swarm: &Swarm,
    sim: Rc<RefCell<Mixing>>,
    ids: Vec<usize>,
    mut advance: impl FnMut(f64) + 'static,
) -> fmn::Result<()> {
    let start = poses_of(&sim.borrow(), &ids);
    swarm.drive(stage, start, move |dt, ps| {
        advance(dt);
        let m = sim.borrow();
        for (p, &i) in ps.iter_mut().zip(&ids) {
            let to = lake_p(m.x[i], m.y[i]);
            let (dx, dy) = ((to[0] - p.x) * FOLLOW, (to[1] - p.y) * FOLLOW);
            if dx.hypot(dy) > 1e-4 {
                p.heading = dy.atan2(dx);
            }
            p.x += dx;
            p.y += dy;
        }
    })
}

pub struct Spread {
    pub kit: Kit,
}

impl SceneConstruct for Spread {
    fn name(&self) -> &str {
        "03_why_the_marks_spread"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let lake = Lake::draw();
        let sim = Rc::new(RefCell::new(Mixing::start(&lake)));
        // drawn: one simulated fish in ten, split by its mark
        let (marked_ids, plain_ids): (Vec<usize>, Vec<usize>) = (0..lake.x.len())
            .step_by(PER_GLYPH)
            .partition(|&i| sim.borrow().marked[i]);

        // 03_1: the act card, then the lake as a pot of soup
        let act = kit.text_styled(
            stage,
            "Why should that work?",
            64.0,
            Palette::hero(),
            v(0.0, 0.3),
            true,
            false,
        )?;
        kit.say(stage, "03_1")?;
        play!(stage; write(stage.arena(), act).rt(1.0));
        stage.wait(1.0)?;
        play!(stage; fade_out(stage.arena_mut(), act, ORIGIN, 1.0)?.rt(0.5));
        let water_g = stage.arena_mut().add(v_group(water()?));
        let marks_g = stage.arena_mut().add(v_group(landmarks()?));
        let plain = Swarm::build(
            stage,
            &fish(Palette::blue()),
            &poses_of(&sim.borrow(), &plain_ids),
            None,
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), water_g, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), marks_g, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), plain.group, ORIGIN, 1.0)?.rt(0.8),
        );
        let ladle = Circle::new()
            .radius(0.55)
            .arc_center(lake_p(0.5, 0.55))
            .style(
                Style::default()
                    .fill(Palette::hero(), 0.12)
                    .stroke(Palette::hero(), 4.0, 1.0),
            )
            .build();
        let ladle = stage.arena_mut().add(ladle);
        play!(stage; fade_in(stage.arena_mut(), ladle, v(0.0, 1.0), 0.6)?.rt(0.8));
        stage.wait(0.5)?;
        play!(stage; fade_out(stage.arena_mut(), ladle, v(0.0, 1.0), 0.6)?.rt(0.8));
        kit.hold(stage)?;

        // 03_2: only we can see this: the marked thousand, let go at the dock
        let peek = kit.text_left(stage, "only we can see this", 28.0, Palette::dim(), v(-6.8, 3.55))?;
        let gold = Swarm::build(
            stage,
            &super::draw::marked_fish(),
            &poses_of(&sim.borrow(), &marked_ids),
            None,
        )?;
        kit.say(stage, "03_2")?;
        play!(stage; fade_in(stage.arena_mut(), peek, ORIGIN, 1.0)?.rt(0.6));
        play!(stage; fade_in(stage.arena_mut(), gold.group, v(0.0, -0.3), 1.0)?.rt(1.4));
        kit.hold(stage)?;

        // 03_3: each region tinted by its share of marked fish
        let cell = (LAKE_W / 3.0, LAKE_H / 3.0);
        let strips = RowStrips::build(stage, lake_p(0.0, 1.0), cell, &tints(&sim.borrow()))?;
        stage.arena_mut().attach(water_g, strips.group)?;
        let dock_share = |m: &Mixing| {
            let (all, marked) = m.regions();
            100.0 * f64::from(marked[1]) / f64::from(all[1].max(1))
        };
        let band = kit.text_left(
            stage,
            "marked share, dock region (S):",
            26.0,
            Palette::dim(),
            v(-4.2, -3.8),
        )?;
        kit.say(stage, "03_3")?;
        play!(stage;
            fade_in(stage.arena_mut(), strips.group, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(1.0),
        );
        let shared = Rc::clone(&sim);
        kit.live_value(
            stage,
            move |_| dock_share(&shared.borrow()),
            0,
            false,
            30.0,
            Palette::gold(),
            v(1.3, -3.8),
            LEFT,
        )?;
        let pct = kit.text_left(stage, "%", 26.0, Palette::gold(), v(2.0, -3.8))?;
        stage.add_to_scene(pct)?;
        let q = predict_pause(&kit, stage, v(6.4, 2.6))?;

        // 03_4: the time-lapse: the fish keep their haunts, the marks spread
        kit.say(stage, "03_4")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        let drive = Rc::clone(&sim);
        let mut elapsed = 0.0;
        follow(stage, &plain, Rc::clone(&sim), plain_ids.clone(), move |dt| {
            elapsed += dt;
            let target = ((elapsed / TIME_LAPSE_S).min(1.0) * mixing::STEPS as f64) as u64;
            let mut m = drive.borrow_mut();
            if target > m.step {
                let n = target - m.step;
                m.advance(n);
            }
        })?;
        follow(stage, &gold, Rc::clone(&sim), marked_ids.clone(), |_| {})?;
        let recolour = Rc::clone(&sim);
        let rows = strips.rows.clone();
        let grid = Rc::new(strips);
        let g2 = Rc::clone(&grid);
        stage.add_dt_updater(
            rows[0],
            move |s, _, _| {
                let colours = tints(&recolour.borrow());
                g2.recolor(s, &colours);
            },
            false,
        )?;
        let clock = kit.text_left(stage, "t = 0 → 400 (time-lapse)", 24.0, Palette::dim(), v(-6.8, 3.1))?;
        play!(stage; fade_in(stage.arena_mut(), clock, ORIGIN, 1.0)?.rt(0.5));
        stage.wait(TIME_LAPSE_S + 0.5)?;
        kit.hold(stage)?;

        // 03_5: nets of a thousand, the story's second catch, at the weeds,
        // open water and the deep end; all nine region centres underneath
        let m = sim.borrow().clone();
        let centres = mixing::region_centres();
        let nets: Vec<u32> = centres.iter().map(|&c| m.net_at(c, FELLER.1 as usize)).collect();
        let mut labels = Vec::new();
        let mut rings = Vec::new();
        for (r, colour) in [(6, Palette::green()), (4, Palette::teal()), (2, Palette::hero())] {
            let at = lake_p(centres[r].0, centres[r].1);
            rings.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.55)
                        .arc_center(at)
                        .style(Style::default().fill(colour, 0.08).stroke(colour, 4.0, 1.0))
                        .build(),
                ),
            );
            labels.push(kit.text(
                stage,
                &format!("{} of {} marked", nets[r], thousands(FELLER.1)),
                24.0,
                colour,
                v(at[0], at[1] + 0.8),
            )?);
        }
        let strip = kit.text(
            stage,
            &format!(
                "all nine regions' nets: {}",
                nets.iter().map(u32::to_string).collect::<Vec<_>>().join(" · ")
            ),
            24.0,
            Palette::ink(),
            v(0.0, 3.55),
        )?;
        let listen = kit.text(
            stage,
            "listen: about one bell for every ten notes",
            22.0,
            Palette::dim(),
            v(0.0, 3.15),
        )?;
        kit.say(stage, "03_5")?;
        for (ring, label) in rings.iter().zip(&labels) {
            play!(stage;
                fade_in(stage.arena_mut(), *ring, v(0.0, 0.8), 0.6)?.rt(0.6),
                fade_in(stage.arena_mut(), *label, ORIGIN, 1.0)?.rt(0.6),
            );
        }
        play!(stage;
            fade_out(stage.arena_mut(), clock, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), peek, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), strip, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), listen, ORIGIN, 1.0)?.rt(0.8),
        );
        kit.listen(stage, 5.0, "03_5")?;
        kit.hold(stage)?;

        // 03_6: two whole corners, every fish, picked by how many fish they
        // hold and never by their marks
        fade_all(stage, 0.8)?;
        let (all, marked) = m.regions();
        let most = (0..9).max_by_key(|&r| all[r]).unwrap_or(0);
        let least = (0..9).min_by_key(|&r| all[r]).unwrap_or(0);
        kit.say(stage, "03_6")?;
        for (r, centre) in [(most, v(-3.6, -0.3)), (least, v(3.6, -0.3))] {
            let (rx, ry) = ((r % 3) as f64, (r / 3) as f64);
            let side = 4.6;
            let mut points = Vec::new();
            let mut colours = Vec::new();
            for i in 0..m.x.len() {
                let (u, w) = (m.x[i] * 3.0 - rx, m.y[i] * 3.0 - ry);
                if (0.0..1.0).contains(&u) && (0.0..1.0).contains(&w) {
                    points.push(v(centre[0] + (u - 0.5) * side, centre[1] + (w - 0.5) * side));
                    colours.push(if m.marked[i] { Palette::gold() } else { Palette::blue() });
                }
            }
            let back = stage.arena_mut().add(
                Rectangle::new()
                    .width(side + 0.2)
                    .height(side + 0.2)
                    .style(
                        Style::default()
                            .fill(water_colour(), 1.0)
                            .stroke(water_colour(), 0.0, 0.0),
                    )
                    .build()?
                    .moved_to(centre),
            );
            let dots = stage.arena_mut().add(dot_cloud(&points, &colours, 0.03));
            let title = kit.text(
                stage,
                &format!("{} corner: every fish", REGIONS[r]),
                26.0,
                Palette::dim(),
                v(centre[0], centre[1] + side / 2.0 + 0.4),
            )?;
            let count = kit.text(
                stage,
                &format!(
                    "{} fish · {} marked ({:.1}%)",
                    thousands(all[r]),
                    thousands(marked[r]),
                    100.0 * f64::from(marked[r]) / f64::from(all[r])
                ),
                28.0,
                Palette::ink(),
                v(centre[0], centre[1] - side / 2.0 - 0.4),
            )?;
            play!(stage;
                fade_in(stage.arena_mut(), back, ORIGIN, 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), dots, ORIGIN, 1.0)?.rt(1.2),
                fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), count, ORIGIN, 1.0)?.rt(0.6),
            );
        }
        kit.hold(stage)?;

        // 03_7: the mosaic again: in both columns the second net took one in
        // ten, so the line across the table runs straight
        fade_all(stage, 0.8)?;
        let (big_m, big_n, k) = FELLER;
        let mosaic = table_cells(k as f64 / big_n as f64, k as f64 / big_m as f64);
        let backs = cell_backs(stage, &mosaic)?;
        let headers = table_labels(&kit, stage, &mosaic)?;
        let line_y = mosaic[0].c[1] - mosaic[0].h / 2.0 - super::lake::GAP / 2.0;
        let (left, right) = (TABLE_C[0] - TABLE_W / 2.0, TABLE_C[0] + TABLE_W / 2.0);
        let divider = stage.arena_mut().add(
            Line::new(v(left, line_y), v(right, line_y))
                .style(line_style(Palette::hero(), 5.0))
                .build()?,
        );
        let lake_size = big_m * big_n / k;
        let shares = kit.text(
            stage,
            &format!(
                "{} of {} marked  ·  {} of {} unmarked  ·  both one in ten",
                thousands(k),
                thousands(big_m),
                thousands(big_n - k),
                thousands(lake_size - big_m)
            ),
            26.0,
            Palette::green(),
            v(TABLE_C[0], TABLE_C[1] - TABLE_H / 2.0 - 0.45),
        )?;
        kit.say(stage, "03_7")?;
        play!(stage;
            fade_in(stage.arena_mut(), backs, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), headers, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage; fade_in(stage.arena_mut(), shares, ORIGIN, 1.0)?.rt(0.8));
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), divider, LEFT, None)?.rt(1.2));
        kit.hold(stage)?;

        // 03_8: that is what independent means here
        let name = kit.text(
            stage,
            "marks mixed fairly = independent",
            30.0,
            Palette::hero(),
            v(TABLE_C[0] + 1.2, line_y - 0.45),
        )?;
        let note = precisely(
            &kit,
            stage,
            "after mixing, being marked is independent of position; equal catchability is a separate \
             assumption (chapter 13)",
        )?;
        kit.say(stage, "03_8")?;
        play!(stage; fade_in(stage.arena_mut(), name, v(0.0, 0.15), 1.0)?.rt(0.8));
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}
