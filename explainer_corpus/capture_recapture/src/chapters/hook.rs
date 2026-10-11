//! Chapter 01, the hook: the fish nobody caught. Feller's story drawn one
//! fish per ten on the seeded lake (01_1), the catches flying into the table
//! and the table growing into the mosaic (01_2–01_3), the title (01_4), then
//! one net dipped and scaled up, three times (01_5–01_8; the lake is
//! `model.rs`, which is oracle A-F27 fish for fish).

use std::f64::consts::TAU;

use explainer_kit::props::{Pose, RowStrips, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{fish, line_style, marked_fish, ring};
use super::lake::{
    Cell, LAKE_H, LAKE_W, PER_GLYPH, TABLE_C, TABLE_H, TABLE_W, Wander, cell_backs, cell_numbers, cell_rect, fly,
    lake_p, lake_picture, places_in, shuffled, table_cells, table_labels, wander,
};
use crate::facts::FELLER;
use crate::model::{GRID, LAKE_FISH, Lake, SEED, SPOTS, SplitMix64};

/// Which catch a drawn fish was in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fate {
    Twice,
    FirstOnly,
    SecondOnly,
    Untouched,
}

impl Fate {
    fn cell(self) -> usize {
        match self {
            Self::Twice => 0,
            Self::FirstOnly => 1,
            Self::SecondOnly => 2,
            Self::Untouched => 3,
        }
    }
}

pub struct Hook {
    pub kit: Kit,
}

impl SceneConstruct for Hook {
    fn name(&self) -> &str {
        "01_hook"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let lake = Lake::draw();
        // the drawn fish: one in ten of the lake's fish
        let drawn: Vec<Vec3> = (0..LAKE_FISH)
            .step_by(PER_GLYPH)
            .map(|i| lake_p(lake.x[i], lake.y[i]))
            .collect();
        let n = drawn.len();
        let (big_m, big_n, k) = (FELLER.0 as usize, FELLER.1 as usize, FELLER.2 as usize);
        let (dm, dn, dk) = (big_m / PER_GLYPH, big_n / PER_GLYPH, k / PER_GLYPH);
        // who was caught: seeded, so every render draws the same story
        let mut rng = SplitMix64::new(SEED ^ 0x0101);
        let order = shuffled(n, &mut rng);
        let mut fate = vec![Fate::Untouched; n];
        for (rank, &i) in order.iter().enumerate() {
            fate[i] = if rank < dk {
                Fate::Twice
            } else if rank < dm {
                Fate::FirstOnly
            } else if rank < dm + dn - dk {
                Fate::SecondOnly
            } else {
                Fate::Untouched
            };
        }
        let marked: Vec<usize> = (0..n)
            .filter(|&i| matches!(fate[i], Fate::Twice | Fate::FirstOnly))
            .collect();
        let second: Vec<usize> = (0..n)
            .filter(|&i| matches!(fate[i], Fate::Twice | Fate::SecondOnly))
            .collect();
        let wanders: Vec<Wander> = drawn
            .iter()
            .map(|&at| {
                let omega = (0.35 + 0.45 * rng.uniform()) * if rng.uniform() < 0.5 { -1.0 } else { 1.0 };
                Wander {
                    at,
                    omega,
                    phase: TAU * rng.uniform(),
                }
            })
            .collect();

        // 01_1: the lake, a thousand fish, the two catches
        let picture = lake_picture(stage)?;
        let key = kit.text(stage, "each fish drawn = 10 fish", 24.0, Palette::dim(), v(5.0, -3.8))?;
        let poses0: Vec<Pose> = wanders.iter().map(|w| w.pose(0.0)).collect();
        let plain = Swarm::build(stage, &fish(Palette::blue()), &poses0, None)?;
        kit.say(stage, "01_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), picture, ORIGIN, 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), plain.group, ORIGIN, 1.0)?.rt(1.6),
            fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(1.0),
        );
        // the wander clock starts with the wandering: every later swarm joins
        // it at `now`, so the gold bodies and the rings sit on their fish
        let t_start = stage.scene().time().to_f64();
        wander(stage, &plain, wanders.clone(), 0.0)?;
        stage.wait(1.0)?;
        // the first catch: marked, glowing, let go where it was
        let now = stage.scene().time().to_f64() - t_start;
        let gold_w: Vec<Wander> = marked.iter().map(|&i| wanders[i]).collect();
        let gold_poses: Vec<Pose> = gold_w.iter().map(|w| w.pose(now)).collect();
        let gold = Swarm::build(stage, &marked_fish(), &gold_poses, None)?;
        wander(stage, &gold, gold_w, now)?;
        play!(stage; fade_in(stage.arena_mut(), gold.group, ORIGIN, 1.4)?.rt(1.2));
        stage.wait(1.2)?;
        // the second catch: a green ring on each fish it caught
        let now = stage.scene().time().to_f64() - t_start;
        let ring_w: Vec<Wander> = second.iter().map(|&i| wanders[i]).collect();
        let ring_poses: Vec<Pose> = ring_w.iter().map(|w| w.pose(now)).collect();
        let rings = Swarm::build(stage, &ring(), &ring_poses, None)?;
        wander(stage, &rings, ring_w, now)?;
        play!(stage; fade_in(stage.arena_mut(), rings.group, ORIGIN, 0.6)?.rt(1.0));
        kit.hold(stage)?;

        // 01_2: the fish no net touched; every fish flies into the table
        let cells = table_cells(0.5, 0.5);
        let backs = cell_backs(stage, &cells)?;
        let mut place_rng = SplitMix64::new(SEED ^ 0x0102);
        let targets = targets_for(&fate, &cells, &mut place_rng);
        kit.say(stage, "01_2")?;
        let now = stage.scene().time().to_f64() - t_start;
        let here = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| wanders[i].pose(now)).collect() };
        let all: Vec<usize> = (0..n).collect();
        let to = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| targets[i]).collect() };
        play!(stage;
            fade_out(stage.arena_mut(), picture, ORIGIN, 1.0)?.rt(1.0),
            fade_out(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), backs, ORIGIN, 1.0)?.rt(1.0),
        );
        fly(stage, &plain, here(&all), to(&all), 2.2)?;
        fly(stage, &gold, here(&marked), to(&marked), 2.2)?;
        fly(stage, &rings, here(&second), to(&second), 2.2)?;
        stage.wait(2.3)?;
        let untouched: Vec<Mob> = (0..n)
            .filter(|&i| fate[i] == Fate::Untouched)
            .map(|i| plain.members[i])
            .collect();
        let mut dim: Vec<Box<dyn Animation>> = Vec::new();
        for m in untouched {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .set_opacity(0.25)?;
            dim.push(stage.prepare(a)?);
        }
        stage.play_prepared(dim)?;
        let headers = table_labels(&kit, stage, &cells)?;
        let numbers = cell_numbers(&kit, stage, &cells, [k, big_m - k, big_n - k])?;
        let question = kit.text(stage, "?", 96.0, Palette::dim(), cells[3].c)?;
        play!(stage;
            fade_in(stage.arena_mut(), headers, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), numbers, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.8),
        );
        kit.hold(stage)?;

        // 01_3: the best guess for the dark cell, and the table becomes the
        // mosaic: areas in proportion
        let u = (big_m - k) * (big_n - k) / k;
        kit.say(stage, "01_3")?;
        let count = stage.add_value_tracker(0.0);
        play!(stage; fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.4));
        let live = kit.live_count(stage, count, 60.0, Palette::ink(), cells[3].c)?;
        stage.play(tween(count, u as f64, 1.4)?)?;
        let guess = kit.text(
            stage,
            &format!("≈ {}", thousands(u as u64)),
            60.0,
            Palette::ink(),
            cells[3].c,
        )?;
        stage.remove_from_scene(live);
        stage.add_to_scene(guess)?;
        // the mosaic: the marked column is k/n of the width (M of an
        // estimated Mn/k), the top row k/M of the height
        let mosaic = table_cells(k as f64 / big_n as f64, k as f64 / big_m as f64);
        let mut place_rng = SplitMix64::new(SEED ^ 0x0103);
        let mosaic_targets = targets_for(&fate, &mosaic, &mut place_rng);
        let to2 = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| mosaic_targets[i]).collect() };
        fly(stage, &plain, to(&all), to2(&all), 2.4)?;
        fly(stage, &gold, to(&marked), to2(&marked), 2.4)?;
        fly(stage, &rings, to(&second), to2(&second), 2.4)?;
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for (back, cell) in explainer_kit::children(stage, backs).into_iter().zip(mosaic) {
            let a = back
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.4),
                    ..AnimateArgs::default()
                })?
                .set_width(cell.w, true)?
                .set_height(cell.h, true)?
                .move_to(cell.c, ORIGIN)?;
            morph.push(stage.prepare(a)?);
        }
        // the headers follow their columns and rows: [marked, unmarked,
        // in the second catch, not]
        let (top, left) = (TABLE_C[1] + TABLE_H / 2.0 + 0.35, TABLE_C[0] - TABLE_W / 2.0);
        let heads = explainer_kit::children(stage, headers);
        let moved = [
            (guess, mosaic[3].c, ORIGIN),
            (heads[0], v(left, top), LEFT),
            (heads[1], v(mosaic[2].c[0], top), ORIGIN),
            (heads[2], v(left - 0.25, mosaic[0].c[1]), RIGHT),
            (heads[3], v(left - 0.25, mosaic[1].c[1]), RIGHT),
        ];
        for (m, at, edge) in moved {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.4),
                    ..AnimateArgs::default()
                })?
                .move_to(at, edge)?;
            morph.push(stage.prepare(a)?);
        }
        for (num, cell) in explainer_kit::children(stage, numbers).into_iter().zip(mosaic) {
            let a = num
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(2.4),
                    ..AnimateArgs::default()
                })?
                .move_to(cell.c, ORIGIN)?;
            morph.push(stage.prepare(a)?);
        }
        stage.play_prepared(morph)?;
        kit.hold(stage)?;

        // 01_4: the title over the unresolved motif
        fade_all(stage, 1.0)?;
        let title = kit.text_styled(
            stage,
            "How to Count Fish You've Never Seen",
            72.0,
            Palette::hero(),
            v(0.0, 0.2),
            true,
            false,
        )?;
        play!(stage; write(stage.arena(), title).rt(1.8));
        kit.listen(stage, 3.0, "01_4")?;
        fade_all(stage, 0.8)?;

        // 01_5–01_8: the frozen lake, a 10 × 10 grid, one net dipped
        dips(&kit, stage, &lake, &drawn)?;

        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// Where each drawn fish sits in the table: its fate's cell, on a grid.
fn targets_for(fate: &[Fate], cells: &[Cell; 4], rng: &mut SplitMix64) -> Vec<Pose> {
    let mut out = vec![
        Pose {
            x: 0.0,
            y: 0.0,
            heading: 0.0
        };
        fate.len()
    ];
    for f in [Fate::Twice, Fate::FirstOnly, Fate::SecondOnly, Fate::Untouched] {
        let ids: Vec<usize> = (0..fate.len()).filter(|&i| fate[i] == f).collect();
        for (&i, p) in ids.iter().zip(places_in(cells[f.cell()], ids.len(), rng)) {
            out[i] = p;
        }
    }
    out
}

/// 01_5–01_8: count one patch and scale up; two other patches; the heat
/// map that explains why one net cannot know its share of the fish.
fn dips(kit: &Kit, stage: &mut Stage<'_>, lake: &Lake, drawn: &[Vec3]) -> fmn::Result<()> {
    let picture = lake_picture(stage)?;
    let mut rng = SplitMix64::new(SEED ^ 0x0105);
    let poses: Vec<Pose> = drawn
        .iter()
        .map(|p| Pose {
            x: p[0],
            y: p[1],
            heading: TAU * rng.uniform(),
        })
        .collect();
    let school = Swarm::build(stage, &fish(Palette::blue()), &poses, None)?;
    let key = kit.text(stage, "each fish drawn = 10 fish", 24.0, Palette::dim(), v(5.0, -3.8))?;
    let mut lines = Vec::new();
    for i in 0..=GRID {
        let f = i as f64 / GRID as f64;
        lines.push(
            Line::new(lake_p(f, 0.0), lake_p(f, 1.0))
                .style(line_style(Palette::faint(), 1.5))
                .build()?,
        );
        lines.push(
            Line::new(lake_p(0.0, f), lake_p(1.0, f))
                .style(line_style(Palette::faint(), 1.5))
                .build()?,
        );
    }
    let grid = stage.arena_mut().add(v_group(lines));

    // 01_5: one net on the deep end, counted and multiplied by a hundred
    let deep = lake.count_at(SPOTS[2].1);
    let (deep_x, deep_y) = SPOTS[2].1;
    let (dc, dr) = crate::model::cell_of(deep_x, deep_y);
    let (c, w, h) = cell_rect(dc, dr);
    let net = Rectangle::new()
        .width(w)
        .height(h)
        .style(
            Style::default()
                .fill(Palette::hero(), 0.16)
                .stroke(Palette::hero(), 4.0, 1.0),
        )
        .build()?
        .moved_to(v(c[0], c[1] + 2.0));
    let net = stage.arena_mut().add(net);
    kit.say(stage, "01_5")?;
    play!(stage;
        fade_in(stage.arena_mut(), picture, ORIGIN, 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), school.group, ORIGIN, 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(1.0),
    );
    play!(stage; fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.8));
    stage.add_to_scene(net)?;
    let drop = net
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.9),
            ..AnimateArgs::default()
        })?
        .move_to(c, ORIGIN)?;
    stage.play(drop)?;
    let label = kit.text_left(stage, "fish in this net:", 30.0, Palette::dim(), v(-6.8, 3.55))?;
    stage.add_to_scene(label)?;
    let tracker = stage.add_value_tracker(0.0);
    let readout = kit.live_count(stage, tracker, 40.0, Palette::hero(), v(-2.6, 3.55))?;
    stage.play(tween(tracker, f64::from(deep), 1.4)?)?;
    let sum = kit.text(
        stage,
        &format!("{} × 100 = {}", thousands(deep), thousands(deep * 100)),
        40.0,
        Palette::ink(),
        v(2.6, 3.55),
    )?;
    play!(stage; write(stage.arena(), sum).rt(1.0));
    kit.hold(stage)?;

    // 01_6: the same net over the weeds and over open water
    kit.say(stage, "01_6")?;
    play!(stage; fade_out(stage.arena_mut(), sum, ORIGIN, 1.0)?.rt(0.4));
    stage.remove_from_scene(readout);
    stage.remove_from_scene(label);
    for spot in [SPOTS[0].1, SPOTS[1].1] {
        let (sc, sr) = crate::model::cell_of(spot.0, spot.1);
        let (to, _, _) = cell_rect(sc, sr);
        let hover = net
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        stage.play(hover)?;
        stage.wait(0.4)?;
    }
    let q = predict_pause(kit, stage, v(6.4, 2.6))?;

    // 01_7: three answers on one number line, 31 times apart
    let estimates: Vec<(u32, Srgb)> = [
        (SPOTS[1].1, Palette::teal()),
        (SPOTS[2].1, Palette::hero()),
        (SPOTS[0].1, Palette::green()),
    ]
    .into_iter()
    .map(|(p, color)| (lake.count_at(p) * 100, color))
    .collect();
    kit.say(stage, "01_7")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    // the strip above the lake (its bank tops out at y = 3.0)
    let (x0, x1, y) = (-6.2, 4.4, 3.3);
    let scale = |e: u32| x0 + (x1 - x0) * f64::from(e) / 100_000.0;
    let mut marks = vec![
        Line::new(v(x0, y), v(x1, y))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    for &(e, color) in &estimates {
        marks.push(
            Line::new(v(scale(e), y - 0.15), v(scale(e), y + 0.15))
                .style(line_style(color, 4.0))
                .build()?,
        );
    }
    let axis = stage.arena_mut().add(v_group(marks));
    let mut tags = Vec::new();
    for &(e, color) in &estimates {
        tags.push(kit.text(stage, &thousands(e), 28.0, color, v(scale(e), y + 0.38))?);
    }
    let lo = estimates.iter().map(|e| e.0).min().unwrap_or(1).max(1);
    let hi = estimates.iter().map(|e| e.0).max().unwrap_or(1);
    let apart = kit.text(
        stage,
        &format!("{}× apart", (f64::from(hi) / f64::from(lo)).floor()),
        30.0,
        Palette::red(),
        v(5.8, y),
    )?;
    play!(stage; fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.8));
    for t in tags {
        play!(stage; fade_in(stage.arena_mut(), t, v(0.0, -0.15), 1.0)?.rt(0.5));
    }
    play!(stage; fade_in(stage.arena_mut(), apart, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;

    // 01_8: the heat map: a hundredth of the water is not a hundredth of
    // the fish
    let max = f64::from(lake.all_counts().max().unwrap_or(1));
    let water = hex("#13253A");
    let colors: Vec<Vec<Srgb>> = (0..GRID)
        .rev()
        .map(|r| {
            (0..GRID)
                .map(|c| interpolate_color(water, Palette::hero(), (f64::from(lake.counts[r][c]) / max).sqrt()))
                .collect()
        })
        .collect();
    let heat = RowStrips::build(
        stage,
        lake_p(0.0, 1.0),
        (LAKE_W / GRID as f64, LAKE_H / GRID as f64),
        &colors,
    )?;
    let box_text = kit.text(
        stage,
        "how many fish in all?  (coming up)",
        30.0,
        Palette::ink(),
        v(0.0, -0.35),
    )?;
    let (bw, bh) = (stage.get_width(box_text) + 0.5, stage.get_height(box_text) + 0.35);
    let frame = stage.arena_mut().add(
        Rectangle::new()
            .width(bw)
            .height(bh)
            .corner_radius(0.1)
            .style(
                Style::default()
                    .fill(hex("#1A2130"), 0.94)
                    .stroke(Palette::faint(), 2.0, 1.0),
            )
            .build()?
            .moved_to(v(0.0, -0.35)),
    );
    kit.say(stage, "01_8")?;
    play!(stage;
        fade_out(stage.arena_mut(), net, ORIGIN, 1.0)?.rt(0.6),
        fade_out(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), heat.group, ORIGIN, 1.0)?.rt(1.6),
    );
    play!(stage;
        fade_in(stage.arena_mut(), frame, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), box_text, ORIGIN, 1.0)?.rt(0.6),
    );
    let note = precisely(
        kit,
        stage,
        &format!(
            "averaging many random dips does work (all 100 squares here average exactly {}), but only {} of the 100 \
             single dips land within 2× of it. One net can't tell what share of the fish it reached.",
            thousands(LAKE_FISH as u64),
            lake.dips_within_2x()
        ),
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
    Ok(())
}
