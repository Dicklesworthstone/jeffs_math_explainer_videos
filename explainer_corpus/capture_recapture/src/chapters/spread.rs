//! Chapter 03, why the marks spread: the soup (03_1), the marked thousand
//! let go at the dock (03_2), the 3 × 3 regions tinted and labelled by
//! their marked share (03_3), the time-lapse of the mixing the oracle
//! checks (03_4: the scene runs `model::mixing` itself, on a clock that
//! starts slow and speeds up), real nets of a thousand lifted into trays
//! (03_5), two whole corners (03_6), and the mosaic's straight line
//! (03_7–03_8).

use std::cell::{Cell as Shared, RefCell};
use std::f64::consts::TAU;
use std::rc::Rc;

use explainer_kit::props::{Pose, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, dot_cloud, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{Glyph, Paint, Role, fish, flat, line_style, marked_fish};
use super::lake::{
    GAP, LAKE_C, LAKE_H, LAKE_W, LakeArt, PER_GLYPH, Regions, Shade, TABLE_C, TABLE_W, behind, cell_backs, fade_in_all,
    fade_out_all, fold, key, lake_p, mosaic_cells, plated, region_centre, table_labels,
};
use super::listen::{Plink, SCORE_FADE_S, catch_sound, last_onset};
use crate::facts::FELLER;
use crate::model::Lake;
use crate::model::mixing::{self, Mixing};

/// The time-lapse: all 40,000 steps (t = 0 … 400) in this many seconds.
const TIME_LAPSE_S: f64 = 7.0;
/// How steeply the time-lapse clock speeds up (model time grows as
/// e^(A·τ/T) − 1): it opens at about 15 steps a second, so the first
/// marks visibly swim out of the dock region, and ends at the even mix.
const CLOCK_A: f64 = 7.3;
/// The regions' names, in the oracle's order.
const REGIONS: [&str; 9] = ["SW", "S", "SE", "W", "C", "E", "NW", "N", "NE"];
/// The three nets of 03_5: region, the habitat's word for it.
const NETS: [(usize, &str); 3] = [(6, "weeds"), (4, "open water"), (2, "deep end")];
/// Where the lake shrinks to for 03_5 (its centre) and by how much.
const SMALL_C: Vec3 = [-3.75, -0.2, 0.0];
const SMALL: f64 = 0.58;

/// Model time at `tau` seconds into the time-lapse.
fn model_time(tau: f64) -> f64 {
    let t_end = mixing::STEPS as f64 * mixing::DT;
    let u = (tau / TIME_LAPSE_S).clamp(0.0, 1.0);
    t_end * (CLOCK_A * u).exp_m1() / CLOCK_A.exp_m1()
}

/// How fast the model clock runs at `tau` (model time per second).
fn clock_rate(tau: f64) -> f64 {
    let t_end = mixing::STEPS as f64 * mixing::DT;
    let u = (tau / TIME_LAPSE_S).clamp(0.0, 1.0);
    t_end * CLOCK_A / TIME_LAPSE_S * (CLOCK_A * u).exp() / CLOCK_A.exp_m1()
}

/// How long a drawn fish takes to catch up with its simulated one: a
/// little smoothing while the clock is slow (one step is a fish length),
/// none once it is fast (smoothing a fast random walk would pull every
/// drawn fish to the middle of its haunt).
fn follow_time(rate: f64) -> f64 {
    0.15 * (1.0 - (rate - 0.15) / 3.0).clamp(0.0, 1.0)
}

/// Every region's marked share.
fn shares(m: &Mixing) -> [f64; 9] {
    let (all, marked) = m.regions();
    let mut out = [0.0; 9];
    for r in 0..9 {
        out[r] = if all[r] > 0 {
            f64::from(marked[r]) / f64::from(all[r])
        } else {
            0.0
        };
    }
    out
}

/// The drawn fish's poses from the simulation, facing east.
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

/// Let a swarm follow its simulated fish; `advance(dt)` runs first each
/// frame and returns the clock's rate (the driver swarm advances the
/// shared simulation; the others only read it). Returns the members'
/// headings, kept current (03_5 turns the caught fish back east).
fn follow(
    stage: &mut Stage<'_>,
    swarm: &Swarm,
    sim: Rc<RefCell<Mixing>>,
    ids: Vec<usize>,
    mut advance: impl FnMut(f64) -> f64 + 'static,
) -> fmn::Result<Rc<RefCell<Vec<f64>>>> {
    let start = poses_of(&sim.borrow(), &ids);
    let headings = Rc::new(RefCell::new(vec![0.0; ids.len()]));
    let out = Rc::clone(&headings);
    swarm.drive(stage, start, move |dt, ps| {
        let rate = advance(dt);
        let tf = follow_time(rate);
        let a = if tf < 1e-3 { 1.0 } else { 1.0 - (-dt / tf).exp() };
        let m = sim.borrow();
        for (p, &i) in ps.iter_mut().zip(&ids) {
            let to = lake_p(m.x[i], m.y[i]);
            let (dx, dy) = ((to[0] - p.x) * a, (to[1] - p.y) * a);
            if dx.hypot(dy) > 1e-3 {
                p.heading = dy.atan2(dx);
            }
            p.x += dx;
            p.y += dy;
        }
        for (h, p) in headings.borrow_mut().iter_mut().zip(ps.iter()) {
            *h = p.heading;
        }
    })?;
    Ok(out)
}

/// The circling along the habitat's contours (03_4): short dashes on
/// circles round the weed bed and the deep hole, crawling anticlockwise
/// (the model's circulation c·J∇log s turns that way round both wells).
fn streamlines(stage: &mut Stage<'_>) -> fmn::Result<(Swarm, Glyph)> {
    let glyph = Glyph {
        mob: Line::new(v(-0.08, 0.0), v(0.08, 0.0))
            .style(line_style(hex("#4F7FA8"), 2.5))
            .build()?,
        paints: vec![Paint::Stroke(0.7)],
    };
    let mut paths = Vec::new();
    for (cx, cy, radii) in [(0.25, 0.75, [0.06_f64, 0.11, 0.16]), (0.75, 0.25, [0.08, 0.14, 0.2])] {
        for r in radii {
            let n = (r * 160.0).round() as usize;
            for j in 0..n {
                paths.push((cx, cy, r, TAU * j as f64 / n as f64));
            }
        }
    }
    let at = |&(cx, cy, r, a): &(f64, f64, f64, f64)| {
        let p = lake_p(cx + r * a.cos(), cy + r * a.sin());
        Pose {
            x: p[0],
            y: p[1],
            heading: (r * a.cos() * LAKE_H).atan2(-r * a.sin() * LAKE_W),
        }
    };
    let poses: Vec<Pose> = paths.iter().map(at).collect();
    let swarm = Swarm::build(stage, &glyph.mob, &poses, None)?;
    // one lap in twelve seconds
    let omega = TAU / 12.0;
    swarm.drive(stage, poses, move |dt, ps| {
        for (p, path) in ps.iter_mut().zip(paths.iter_mut()) {
            path.3 += omega * dt;
            *p = at(path);
        }
    })?;
    Ok((swarm, glyph))
}

/// A live percentage on a plate, computed each frame by `f`, centred at
/// `at`.
fn live_percent(
    kit: &Kit,
    stage: &mut Stage<'_>,
    f: impl Fn() -> f64 + 'static,
    size: f64,
    color: Srgb,
    at: Vec3,
) -> fmn::Result<(Mob, Mob)> {
    let plate = plated(kit, stage, "00%", size, color, at)?;
    let text = children(stage, plate)[1];
    stage.set_opacity(text, 0.0, true);
    let book = Rc::clone(&kit.book);
    let live = stage.always_redraw(move |s| {
        let d = DecimalNumber::new((100.0 * f()).round())
            .num_decimal_places(0)
            .unit("%")
            .font_size(size)
            .color(color)
            .build(&book)
            .expect("bundled digits always lay out");
        explainer_kit::glyphs::record(explainer_kit::TextKind::Text, &format!("{}%", d.num_string()));
        let m = s.add(d.into_vmob());
        s.move_to(m, at, ORIGIN);
        m
    });
    Ok((plate, live))
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
        let art = LakeArt::build(stage)?;
        let blue = fish(Role::unmarked());
        let plain = Swarm::build(stage, &blue.mob, &poses_of(&sim.borrow(), &plain_ids), None)?;
        let the_key = key(&kit, stage)?;
        play!(stage;
            fade_in(stage.arena_mut(), art.under, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), art.over, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), plain.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), the_key, ORIGIN, 1.0)?.rt(0.8),
        );
        let ladle = Circle::new()
            .radius(0.55)
            .arc_center(lake_p(0.5, 0.55))
            .style(Style::default().fill(Role::net(), 0.1).stroke(Role::net(), 4.0, 1.0))
            .build();
        let ladle = stage.arena_mut().add(ladle);
        play!(stage; fade_in(stage.arena_mut(), ladle, v(0.0, -1.0), 1.0)?.rt(0.8));
        stage.wait(0.5)?;
        play!(stage; fade_out(stage.arena_mut(), ladle, v(0.0, 1.0), 1.0)?.rt(0.8));
        kit.hold(stage)?;

        // 03_2: only we can see this: the marked thousand, let go at the dock
        let peek = kit.text_left(stage, "only we can see this", 28.0, Palette::dim(), v(-6.6, 3.45))?;
        let share_words = kit.text_left(
            stage,
            "each region's number: the share of its fish that are marked",
            24.0,
            Role::marked(),
            v(-6.6, 3.1),
        )?;
        let gold_glyph = marked_fish();
        let gold = Swarm::build(stage, &gold_glyph.mob, &poses_of(&sim.borrow(), &marked_ids), None)?;
        kit.say(stage, "03_2")?;
        play!(stage; fade_in(stage.arena_mut(), peek, ORIGIN, 1.0)?.rt(0.6));
        play!(stage; fade_in(stage.arena_mut(), gold.group, v(0.0, -0.3), 1.0)?.rt(1.4));
        kit.hold(stage)?;

        // 03_3: each region tinted and labelled by its share of marked fish
        // (the tints lie on the water, under the landmarks and the fish)
        let regions = Regions::build(stage)?;
        let borders = Regions::borders(stage)?;
        stage.add_to_scene(regions.group)?;
        stage.add_to_scene(borders)?;
        stage.set_opacity(borders, 0.0, true);
        stage
            .scene_mut()
            .bring_to_front(&[art.over, plain.group, gold.group, the_key, peek])?;
        let tint_level = stage.add_value_tracker(0.0);
        stage.add_to_scene(tint_level)?;
        let tinting = Rc::clone(&sim);
        let cells = regions.cells.clone();
        let regions = Rc::new(regions);
        let tinter = Rc::clone(&regions);
        stage.add_dt_updater(
            cells[0],
            move |s, _, _| {
                let level = s.tracker_value(tint_level).unwrap_or(0.0);
                let sh = shares(&tinting.borrow()).map(|x| x * level);
                tinter.tint(s, &sh);
            },
            true,
        )?;
        let mut readouts = Vec::new();
        for r in 0..9 {
            let c = region_centre(r);
            let at = v(c[0] - LAKE_W / 6.0 + 0.55, c[1] + LAKE_H / 6.0 - 0.3);
            let s = Rc::clone(&sim);
            let (plate, live) = live_percent(&kit, stage, move || shares(&s.borrow())[r], 22.0, Role::marked(), at)?;
            readouts.push((plate, live));
        }
        kit.say(stage, "03_3")?;
        let border_in = borders
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        play!(stage; tween(tint_level, 1.0, 1.2)?, border_in);
        let plates: Vec<Mob> = readouts.iter().map(|r| r.0).collect();
        let mut plates_in = plates.clone();
        plates_in.push(share_words);
        fade_in_all(stage, &plates_in, 0.6)?;
        for &(_, live) in &readouts {
            stage.add_to_scene(live)?;
        }
        let q = predict_pause(&kit, stage, v(6.3, 2.4))?;

        // 03_4: the time-lapse: the fish keep their haunts, the marks spread
        let (lines, line_glyph) = streamlines(stage)?;
        let line_shade = Shade::new(stage, &lines.members, &line_glyph.paints, 0.0)?;
        stage.add_to_scene(lines.group)?;
        stage.scene_mut().bring_to_front(&[plain.group, gold.group])?;
        for &(plate, live) in &readouts {
            stage.scene_mut().bring_to_front(&[plate, live])?;
        }
        let clock_label = kit.text_left(stage, "time-lapse:  t =", 26.0, Palette::dim(), v(2.6, 3.45))?;
        let tau = Rc::new(Shared::new(0.0));
        let tau_read = Rc::clone(&tau);
        let clock = kit.live_value(
            stage,
            move |_| model_time(tau_read.get()).floor(),
            0,
            false,
            28.0,
            Palette::ink(),
            v(5.15, 3.45),
            LEFT,
        )?;
        stage.remove_from_scene(clock);
        kit.say(stage, "03_4")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), clock_label, ORIGIN, 1.0)?.rt(0.5),
            line_shade.to(1.0, 1.2)?,
        );
        stage.add_to_scene(clock)?;
        // a second at t = 0 first, so the dock's starting share is seen (R3
        // Ruth 04: the oracle's t0 shares, 66% at the dock)
        stage.wait(1.0)?;
        let drive = Rc::clone(&sim);
        let clock_tau = Rc::clone(&tau);
        let plain_heads = follow(stage, &plain, Rc::clone(&sim), plain_ids.clone(), move |dt| {
            let now = clock_tau.get() + dt;
            clock_tau.set(now);
            let target = ((model_time(now) / mixing::DT).round() as u64).min(mixing::STEPS);
            let mut m = drive.borrow_mut();
            if target > m.step {
                let n = target - m.step;
                m.advance(n);
            }
            clock_rate(now)
        })?;
        let gold_tau = Rc::clone(&tau);
        let gold_heads = follow(stage, &gold, Rc::clone(&sim), marked_ids.clone(), move |_| {
            clock_rate(gold_tau.get())
        })?;
        stage.wait(TIME_LAPSE_S + 0.4)?;
        // the clock has stopped at t = 400 and every drawn fish sits on its
        // simulated one
        stage.clear_updaters(plain.group, false);
        stage.clear_updaters(gold.group, false);
        kit.hold(stage)?;

        // 03_5: nets of a thousand, the story's second catch, at the weeds,
        // open water and the deep end, lifted into trays; every region's
        // net written where it was cast
        let m = sim.borrow().clone();
        let centres = mixing::region_centres();
        let nets: Vec<u32> = centres.iter().map(|&c| m.net_at(c, FELLER.1 as usize)).collect();
        let mut gone = vec![
            regions.group,
            borders,
            lines.group,
            clock,
            clock_label,
            peek,
            share_words,
        ];
        for &(plate, live) in &readouts {
            gone.push(plate);
            gone.push(live);
        }
        kit.say(stage, "03_5")?;
        fade_out_all(stage, &gone, 0.6)?;
        // the lake shrinks to the left half: p ↦ P + SMALL (p − P)
        let pivot = [
            (SMALL_C[0] - SMALL * LAKE_C[0]) / (1.0 - SMALL),
            (SMALL_C[1] - SMALL * LAKE_C[1]) / (1.0 - SMALL),
            0.0,
        ];
        let small = |p: Vec3| {
            v(
                pivot[0] + SMALL * (p[0] - pivot[0]),
                pivot[1] + SMALL * (p[1] - pivot[1]),
            )
        };
        let mut shrink: Vec<Box<dyn Animation>> = Vec::new();
        for g in [art.under, art.over, plain.group, gold.group] {
            let a = g
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .scale_about(SMALL, Some(pivot), None)?;
            shrink.push(stage.prepare(a)?);
        }
        stage.play_prepared(shrink)?;
        // each net is drawn at the size it really is: out to its
        // thousandth-nearest fish
        let mut trays = Vec::new();
        for (row, &(r, word)) in NETS.iter().enumerate() {
            let c = centres[r];
            let members = m.net_members(c, FELLER.1 as usize);
            let far = members.last().map_or(0.0, |&i| (m.x[i] - c.0).hypot(m.y[i] - c.1));
            let ring = stage.arena_mut().add(
                Ellipse::new()
                    .width(2.0 * far * LAKE_W * SMALL)
                    .height(2.0 * far * LAKE_H * SMALL)
                    .style(
                        Style::default()
                            .fill(Role::second(), 0.08)
                            .stroke(Role::second(), 3.5, 1.0),
                    )
                    .build()
                    .moved_to(small(lake_p(c.0, c.1))),
            );
            let tray_c = v(3.75, 2.2 - 2.2 * row as f64);
            let label = kit.text_left(
                stage,
                &format!("{word}: {} of {} marked", nets[r], thousands(FELLER.1)),
                26.0,
                Role::second(),
                v(tray_c[0] - 3.0, tray_c[1] + 0.95),
            )?;
            let card = stage.arena_mut().add(
                Rectangle::new()
                    .width(6.3)
                    .height(1.5)
                    .corner_radius(0.12)
                    .style(
                        Style::default()
                            .fill(hex("#14202E"), 1.0)
                            .stroke(Role::second(), 2.0, 0.6),
                    )
                    .build()?
                    .moved_to(tray_c),
            );
            play!(stage;
                fmn::animation::grow_from_center(stage.arena_mut(), ring, None)?.rt(0.6),
                fade_in(stage.arena_mut(), card, ORIGIN, 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.6),
            );
            trays.push((members, tray_c));
        }
        // the drawn fish of each net (one in ten) lift out in catch order:
        // the open water's and the deep end's during the line, then the
        // weed bed's in the listen window, a note for each as it lands (a
        // bell for a marked one: the true sample heard, counted on screen)
        // the trays' cards were added after the fish: the fish go on top
        stage.scene_mut().bring_to_front(&[plain.group, gold.group])?;
        let heads = (plain_heads.borrow().clone(), gold_heads.borrow().clone());
        let drawn = Drawn {
            plain: (&plain, &plain_ids),
            gold: (&gold, &marked_ids),
            heads: &heads,
        };
        let quiet = plan_flights(&m, &trays[1..], &small, &drawn, FLIGHT_GAP_S);
        fly_flights(stage, quiet)?;
        let heard = plan_flights(&m, &trays[..1], &small, &drawn, listen_gap(&trays[0].0));
        let plinks = landings(&heard);
        let bells = plinks.iter().filter(|p| p.marked).count();
        let window = plinks.last().map_or(0.0, |p| p.at) + SCORE_FADE_S + 0.4;
        assert!(
            plinks.iter().all(|p| p.at <= last_onset(window)),
            "03_5: a note in the score's fade"
        );
        let listen_note = kit.text_left(
            stage,
            &format!(
                "listen: a note for each fish into the {} tray, a bell for each marked one ({bells} of {})",
                NETS[0].1,
                plinks.len()
            ),
            24.0,
            Palette::dim(),
            v(-6.6, 3.45),
        )?;
        let wav = catch_sound("a03_5_weeds_tray", &plinks, window).map_err(|e| {
            fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
                format!("03_5 sonification: {e}").into_boxed_str(),
            )))
        })?;
        fade_in_all(stage, &[listen_note], 0.5)?;
        kit.sonify(stage, window, "03_5", &wav)?;
        fly_flights(stage, heard)?;
        stage.wait(window)?;
        // every region's net, written where it was cast
        let mut counts = Vec::new();
        for (r, &count) in nets.iter().enumerate() {
            let at = small(region_centre(r));
            counts.push(plated(&kit, stage, &count.to_string(), 26.0, Role::marked(), at)?);
        }
        let lo = nets.iter().copied().min().unwrap_or(0);
        let hi = nets.iter().copied().max().unwrap_or(0);
        let below = small(lake_p(0.5, 0.0))[1];
        let caption = kit.text(
            stage,
            &format!("every region's net of {}:", thousands(FELLER.1)),
            24.0,
            Palette::ink(),
            v(SMALL_C[0], below - 0.45),
        )?;
        let range = kit.text(
            stage,
            &format!(
                "{lo} to {hi} marked ({:.1}% to {:.1}%)",
                100.0 * f64::from(lo) / FELLER.1 as f64,
                100.0 * f64::from(hi) / FELLER.1 as f64
            ),
            24.0,
            Role::marked(),
            v(SMALL_C[0], below - 0.85),
        )?;
        let mut tags = counts.clone();
        tags.extend([caption, range]);
        fade_in_all(stage, &tags, 0.6)?;
        kit.hold(stage)?;

        // 03_6: two whole corners, every fish, picked by how many fish they
        // hold and never by their marks
        fade_all(stage, 0.8)?;
        corners(&kit, stage, &m)?;
        kit.hold(stage)?;

        // 03_7: the mosaic again: in both columns the second net took one in
        // ten, so the line across the table runs straight
        fade_all(stage, 0.8)?;
        let (big_m, big_n, k) = FELLER;
        let lake_size = big_m * big_n / k;
        let mosaic = mosaic_cells(lake_size as f64, big_m as f64, k as f64, big_n as f64);
        let backs = cell_backs(stage, &mosaic)?;
        behind(stage, backs)?;
        stage.set_opacity(backs, 0.0, true);
        let headers = table_labels(&kit, stage, &mosaic)?;
        let line_y = mosaic[0].c[1] - mosaic[0].h / 2.0 - GAP / 2.0;
        let (left, right) = (TABLE_C[0] - TABLE_W / 2.0, TABLE_C[0] + TABLE_W / 2.0);
        let divider = stage.arena_mut().add(
            Line::new(v(left, line_y), v(right, line_y))
                .style(line_style(Role::second(), 6.0))
                .build()?,
        );
        // each column's green share counts up inside the dark cell
        let dark = mosaic[3];
        let rows = [
            ("marked column:", k, big_m, Role::marked(), dark.c[1] + 1.2),
            (
                "unmarked column:",
                big_n - k,
                lake_size - big_m,
                Palette::ink(),
                dark.c[1] + 0.55,
            ),
        ];
        let x_words = dark.c[0] - 3.2;
        let x_num = dark.c[0] + 0.55;
        let mut counters = Vec::new();
        for &(word, _, _, color, y) in &rows {
            counters.push(kit.text_left(stage, word, 30.0, color, v(x_words, y))?);
        }
        kit.say(stage, "03_7")?;
        let back_in = backs
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        play!(stage;
            back_in,
            fade_in(stage.arena_mut(), headers, ORIGIN, 1.0)?.rt(0.8),
        );
        fade_in_all(stage, &counters, 0.5)?;
        let mut climbs = Vec::new();
        let mut tails = Vec::new();
        for &(_, part, whole, color, y) in &rows {
            let t = stage.add_value_tracker(0.0);
            kit.live_number(stage, t, 0, false, 30.0, color, v(x_num, y), RIGHT)?;
            climbs.push((t, part));
            tails.push(kit.text_left(
                stage,
                &format!(" of {}  =  1 in {}", thousands(whole), whole / part),
                30.0,
                color,
                v(x_num + 0.1, y),
            )?);
        }
        play!(stage; tween(climbs[0].0, climbs[0].1 as f64, 1.0)?, tween(climbs[1].0, climbs[1].1 as f64, 1.6)?);
        fade_in_all(stage, &tails, 0.5)?;
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), divider, LEFT, None)?.rt(1.2));
        kit.hold(stage)?;

        // 03_8: that is what independent means here
        let name = kit.text(
            stage,
            "marks mixed fairly = independent",
            32.0,
            Role::second(),
            v(dark.c[0], line_y - 0.42),
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

/// How long one fish of 03_5 takes to fly to its tray, and the gap between
/// one fish's start and the next (the trays filled during the line).
const FLIGHT_S: f64 = 0.55;
const FLIGHT_GAP_S: f64 = 0.016;
/// How long the heard tray takes to fill, every fish a note.
const LISTEN_FILL_S: f64 = 4.0;

/// A net's drawn fish (one simulated fish in ten is drawn), in catch order.
fn catch_order(members: &[usize]) -> Vec<usize> {
    members
        .iter()
        .copied()
        .filter(|i| i.is_multiple_of(PER_GLYPH))
        .collect()
}

/// The heard tray's gap between fish: it fills in [`LISTEN_FILL_S`].
fn listen_gap(members: &[usize]) -> f64 {
    LISTEN_FILL_S / catch_order(members).len().max(1) as f64
}
/// Fish per tray row.
const TRAY_ROW: usize = 25;

/// One fish's flight to its tray: the member, from, to, its start time,
/// the turn that brings it back to facing east, and its mark.
struct Flight {
    mob: Mob,
    from: Vec3,
    to: Vec3,
    start: f64,
    turn: f64,
    marked: bool,
}

/// The swarms that draw 03_5's lake: the plain and the gold one, the
/// simulated fish each member draws, and the members' headings.
struct Drawn<'a> {
    plain: (&'a Swarm, &'a [usize]),
    gold: (&'a Swarm, &'a [usize]),
    heads: &'a (Vec<f64>, Vec<f64>),
}

/// 03_5's flights for some trays: each net's drawn fish (one in ten) in
/// catch order, from the (shrunken) lake to the tray's slots, starting
/// `gap` seconds apart.
fn plan_flights(
    m: &Mixing,
    trays: &[(Vec<usize>, Vec3)],
    small: &impl Fn(Vec3) -> Vec3,
    drawn: &Drawn<'_>,
    gap: f64,
) -> Vec<Flight> {
    let member = |i: usize| -> Option<(Mob, f64)> {
        let ((swarm, ids), h) = if m.marked[i] {
            (drawn.gold, &drawn.heads.1)
        } else {
            (drawn.plain, &drawn.heads.0)
        };
        ids.binary_search(&i).ok().map(|k| (swarm.members[k], h[k]))
    };
    let mut flights: Vec<Flight> = Vec::new();
    for (members, tray_c) in trays {
        let caught: Vec<(usize, Mob, f64)> = catch_order(members)
            .into_iter()
            .filter_map(|i| member(i).map(|(mob, h)| (i, mob, h)))
            .collect();
        for (slot, &(i, mob, heading)) in caught.iter().enumerate() {
            let (col, row) = (slot % TRAY_ROW, slot / TRAY_ROW);
            flights.push(Flight {
                mob,
                from: small(lake_p(m.x[i], m.y[i])),
                to: v(
                    tray_c[0] - 2.82 + 0.235 * col as f64,
                    tray_c[1] + 0.5 - 0.25 * row as f64,
                ),
                start: gap * flights.len() as f64,
                turn: -fold(heading),
                marked: m.marked[i],
            });
        }
    }
    flights
}

/// When each flight lands, and whether it carries a marked fish: what the
/// window plays.
fn landings(flights: &[Flight]) -> Vec<Plink> {
    flights
        .iter()
        .map(|f| Plink {
            at: f.start + FLIGHT_S,
            marked: f.marked,
        })
        .collect()
}

/// Fly the fish: each grows back to full size and turns to face east as it
/// settles in its slot. One updater moves them all, by placement only; its
/// clock starts now.
fn fly_flights(stage: &mut Stage<'_>, flights: Vec<Flight>) -> fmn::Result<()> {
    let clock = stage.add_value_tracker(0.0);
    stage.add_to_scene(clock)?;
    let mut t = 0.0;
    // (where it is, how big, how far turned) as of the last frame
    let mut last: Vec<(Vec3, f64, f64)> = flights.iter().map(|f| (f.from, SMALL, 0.0)).collect();
    stage.add_dt_updater(
        clock,
        move |s, _, dt| {
            t += dt;
            for (f, prev) in flights.iter().zip(last.iter_mut()) {
                let u = ((t - f.start) / FLIGHT_S).clamp(0.0, 1.0);
                let e = u * u * (3.0 - 2.0 * u);
                let p = v(
                    f.from[0] + (f.to[0] - f.from[0]) * e,
                    f.from[1] + (f.to[1] - f.from[1]) * e,
                );
                let (size, turned) = (SMALL + (1.0 - SMALL) * e, f.turn * e);
                if p != prev.0 {
                    s.shift(f.mob, [p[0] - prev.0[0], p[1] - prev.0[1], 0.0]);
                    s.rotate(f.mob, turned - prev.2, OUT, Some(p), None);
                    s.scale_about(f.mob, size / prev.1, Some(p), None);
                    *prev = (p, size, turned);
                }
            }
        },
        false,
    )?;
    Ok(())
}

/// 03_6: the fullest and the emptiest of the nine regions at t = 400,
/// every fish drawn, beside a map of where they are.
fn corners(kit: &Kit, stage: &mut Stage<'_>, m: &Mixing) -> fmn::Result<()> {
    let (all, marked) = m.regions();
    let most = (0..9).max_by_key(|&r| all[r]).unwrap_or(0);
    let least = (0..9).min_by_key(|&r| all[r]).unwrap_or(0);
    // the map: the water and the nine regions, the two picked ones outlined
    let (map_c, map_s) = (v(0.0, 2.55), 0.26);
    let map_p = |x: f64, y: f64| {
        v(
            map_c[0] + (x - 0.5) * LAKE_W * map_s,
            map_c[1] + (y - 0.5) * LAKE_H * map_s,
        )
    };
    let mut map_parts = vec![
        Rectangle::new()
            .width(LAKE_W * map_s)
            .height(LAKE_H * map_s)
            .corner_radius(0.12)
            .style(flat(Role::water(), 1.0))
            .build()?
            .moved_to(map_c),
    ];
    for k in 1..3 {
        let f = k as f64 / 3.0;
        map_parts.push(
            Line::new(map_p(f, 0.0), map_p(f, 1.0))
                .style(line_style(hex("#3A5470"), 1.5))
                .build()?,
        );
        map_parts.push(
            Line::new(map_p(0.0, f), map_p(1.0, f))
                .style(line_style(hex("#3A5470"), 1.5))
                .build()?,
        );
    }
    let map = stage.arena_mut().add(v_group(map_parts));
    kit.say(stage, "03_6")?;
    play!(stage; fade_in(stage.arena_mut(), map, ORIGIN, 1.0)?.rt(0.6));
    let side = 4.3;
    for (r, centre, word) in [(most, v(-3.7, -0.75), "fullest"), (least, v(3.7, -0.75), "emptiest")] {
        let (rx, ry) = ((r % 3) as f64, (r / 3) as f64);
        let pick = stage.arena_mut().add(
            Rectangle::new()
                .width(LAKE_W * map_s / 3.0)
                .height(LAKE_H * map_s / 3.0)
                .style(
                    Style::default()
                        .fill(Palette::ink(), 0.12)
                        .stroke(Palette::ink(), 3.0, 1.0),
                )
                .build()?
                .moved_to(map_p((rx + 0.5) / 3.0, (ry + 0.5) / 3.0)),
        );
        let mut points = Vec::new();
        let mut colours = Vec::new();
        for i in 0..m.x.len() {
            let (u, w) = (m.x[i] * 3.0 - rx, m.y[i] * 3.0 - ry);
            if (0.0..1.0).contains(&u) && (0.0..1.0).contains(&w) {
                points.push(v(centre[0] + (u - 0.5) * side, centre[1] + (w - 0.5) * side));
                colours.push(if m.marked[i] { Role::marked() } else { Role::unmarked() });
            }
        }
        let back = stage.arena_mut().add(
            Rectangle::new()
                .width(side + 0.2)
                .height(side + 0.2)
                .corner_radius(0.1)
                .style(
                    Style::default()
                        .fill(Role::water(), 1.0)
                        .stroke(Palette::ink(), 2.0, 0.6),
                )
                .build()?
                .moved_to(centre),
        );
        let dots = stage.arena_mut().add(dot_cloud(&points, &colours, 0.032));
        let title = kit.text(
            stage,
            &format!("{word} corner ({}): every fish in it", REGIONS[r]),
            26.0,
            Palette::dim(),
            v(centre[0], centre[1] + side / 2.0 + 0.35),
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
            v(centre[0], centre[1] - side / 2.0 - 0.35),
        )?;
        // the inset grows out of its region on the map
        let from = map_p((rx + 0.5) / 3.0, (ry + 0.5) / 3.0);
        let shift = v(centre[0] - from[0], centre[1] - from[1]);
        let grow = (side + 0.2) / (LAKE_W * map_s / 3.0);
        play!(stage; fade_in(stage.arena_mut(), pick, ORIGIN, 1.0)?.rt(0.5));
        play!(stage;
            fade_in(stage.arena_mut(), back, shift, grow)?.rt(0.9),
            fade_in(stage.arena_mut(), dots, shift, grow)?.rt(0.9),
        );
        play!(stage;
            fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), count, ORIGIN, 1.0)?.rt(0.5),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heard_tray_is_the_weed_beds_drawn_catch_and_fits_its_window() {
        let lake = Lake::draw();
        let mut m = Mixing::start(&lake);
        m.advance(mixing::STEPS);
        let c = mixing::region_centres()[NETS[0].0];
        let members = m.net_members(c, FELLER.1 as usize);
        let heard = catch_order(&members);
        // one note per drawn fish of the net, a bell per marked one: the
        // drawn sample, not a made-up one in ten
        assert_eq!(
            heard.len(),
            members.iter().filter(|&&i| i.is_multiple_of(PER_GLYPH)).count()
        );
        let bells = heard.iter().filter(|&&i| m.marked[i]).count();
        let marked_drawn = members
            .iter()
            .filter(|&&i| i.is_multiple_of(PER_GLYPH) && m.marked[i])
            .count();
        assert_eq!(bells, marked_drawn);
        // the true net's share (A-CH03.portable.nets1000: 105 of 1,000) and
        // the drawn sample's stay near one in ten
        assert_eq!(m.net_at(c, FELLER.1 as usize), 105);
        assert!((5..=20).contains(&bells), "{bells} bells of {}", heard.len());
        // every note starts before the score's fade
        let gap = listen_gap(&members);
        let last = gap * (heard.len() - 1) as f64 + FLIGHT_S;
        let window = last + SCORE_FADE_S + 0.4;
        assert!(last <= last_onset(window));
        assert!((gap * heard.len() as f64 - LISTEN_FILL_S).abs() < 1e-9);
    }

    #[test]
    fn the_time_lapse_clock_runs_from_zero_to_the_last_step() {
        assert_eq!(model_time(0.0), 0.0);
        assert!((model_time(TIME_LAPSE_S) - mixing::STEPS as f64 * mixing::DT).abs() < 1e-9);
        // slow at first: under a step per frame at 60 fps
        assert!(clock_rate(0.0) / mixing::DT < 60.0);
        assert!(follow_time(clock_rate(0.0)) > 0.1 && follow_time(clock_rate(TIME_LAPSE_S)) == 0.0);
    }
}
