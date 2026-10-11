//! Chapter 05, turning it around: the mystery lake, marked 10, caught 10,
//! caught twice 3 (05_1); the soup guess (05_2); the smallest lake it could
//! be (05_3); three lake sizes' chances of our catch, each dropping its bar
//! onto the score curve (05_4); every size swept, the histogram live (05_5);
//! the peak and the name (05_6); and the heights summed out to every lake
//! size, 16⅔, not 100% (05_7). It ends on the score curve of
//! `chapters/curve.rs`, the picture chapter 06 opens and closes on.

use std::rc::Rc;

use explainer_kit::props::{Pose, Swarm, precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, hex, play, rect, thousands, tween, v};
use fmn::prelude::*;

use super::curve;
use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use super::lake::{LakeArt, Shade, fade_in_all, fade_out_all, plated, shuffled};
use super::listen::{Plink, SCORE_FADE_S, catch_sound, last_onset, sweep_sound};
use crate::facts::{TOY, best_sizes};
use crate::law::Law;
use crate::model::{SEED, SplitMix64};

/// The three stills of 05_4.
const STILLS: [u64; 3] = [20, 33, 60];
/// How far 05_7's running sum goes on screen (the sizes past it add about
/// 0.015 more; "all sizes" is the closed form, A-F7).
const SUM_TO: u64 = 2_000;

/// The histogram panel over the curve: k = 0 … 10 for one lake size.
fn hist_frame() -> Frame2 {
    Frame2 {
        center: v(3.6, 2.6),
        width: 4.6,
        height: 1.4,
        x: (-0.6, TOY.1 as f64 + 0.6),
        y: (0.0, 0.6),
    }
}

/// One bar of the histogram for lake size `lake`: k's chance.
fn hist_bars(lake: u64) -> fmn::Result<VMobject> {
    let f = hist_frame();
    let law = Law::new(lake, TOY.0, TOY.1);
    let w = 0.72 * f.width / (f.x.1 - f.x.0);
    let bars: Vec<VMobject> = (0..=TOY.1)
        .map(|k| {
            let (base, top) = (f.p(k as f64, 0.0), f.p(k as f64, law.at(k)));
            let c = if k == TOY.2 { Role::score() } else { Role::chance() };
            rect(
                v(base[0] - w / 2.0, base[1]),
                w,
                (top[1] - base[1]).max(0.004),
                flat(c, 0.9),
            )
        })
        .collect::<fmn::Result<_>>()?;
    Ok(v_group(bars))
}

/// The histogram's floor and its k labels, detached.
fn hist_axis(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let f = hist_frame();
    let floor = Line::new(v(f.left(), f.bottom()), v(f.right(), f.bottom()))
        .style(line_style(Palette::dim(), 2.0))
        .build()?;
    let g = stage.arena_mut().add(floor);
    for k in [0, TOY.2, TOY.1] {
        let c = if k == TOY.2 { Role::score() } else { Palette::dim() };
        let m = kit.text(
            stage,
            &k.to_string(),
            20.0,
            c,
            v(f.p(k as f64, 0.0)[0], f.bottom() - 0.2),
        )?;
        stage.attach(g, m)?;
    }
    let word = kit.text(
        stage,
        "caught twice",
        20.0,
        Palette::dim(),
        v(f.center[0], f.bottom() - 0.48),
    )?;
    stage.attach(g, word)?;
    Ok(g)
}

/// Every lake size's score summed from the smallest up to `to`.
fn partial_sums(to: u64) -> Vec<f64> {
    let mut acc = 0.0;
    (*curve::sizes().start()..=to)
        .map(|n| {
            acc += curve::score(n);
            acc
        })
        .collect()
}

pub struct Turning {
    pub kit: Kit,
}

impl SceneConstruct for Turning {
    fn name(&self) -> &str {
        "05_turning_it_around"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (m, n, k) = TOY;
        let first = *curve::sizes().start();

        // 05_1: the mystery lake, in fog, and its second catch of ten
        let art = LakeArt::whole(stage)?;
        // fog: soft banks, many faint layers over the water (seeded)
        let mut fog_rng = SplitMix64::new(SEED ^ 0x0500);
        let mut fog_parts = Vec::new();
        for _ in 0..14 {
            let (x, y) = (-4.6 + 9.2 * fog_rng.uniform(), -2.6 + 5.0 * fog_rng.uniform());
            let (w, h) = (2.6 + 3.4 * fog_rng.uniform(), 0.9 + 1.2 * fog_rng.uniform());
            fog_parts.push(
                Ellipse::new()
                    .width(w)
                    .height(h)
                    .style(flat(hex("#8E98A6"), 0.11))
                    .build()
                    .moved_to(v(x, y)),
            );
        }
        let fog = stage.arena_mut().add(v_group(fog_parts));
        let mystery = kit.text_styled(stage, "how many fish?", 44.0, Palette::ink(), v(0.0, 0.3), true, false)?;
        let name = kit.text(stage, "the mystery lake", 30.0, Role::dark(), v(0.0, 3.5))?;
        kit.say(stage, "05_1")?;
        fade_in_all(stage, &[art, fog, mystery], 1.0)?;
        fade_in_all(stage, &[name], 0.6)?;
        // the second catch comes up: ten fish in a row, three of them marked,
        // in catch order (seeded)
        let mut rng = SplitMix64::new(SEED ^ 0x0501);
        let mut gold_at: Vec<usize> = shuffled(n as usize, &mut rng)[..k as usize].to_vec();
        gold_at.sort_unstable();
        let row_y = -2.45;
        let slot = |i: usize| v(-3.6 + 0.8 * i as f64, row_y);
        let card = stage.arena_mut().add(
            Rectangle::new()
                .width(8.6)
                .height(1.0)
                .corner_radius(0.12)
                .style(
                    Style::default()
                        .fill(hex("#14202E"), 1.0)
                        .stroke(Role::second(), 2.0, 0.8),
                )
                .build()?
                .moved_to(v(0.0, row_y)),
        );
        let big = 2.4;
        let pose = |i: usize| {
            let p = slot(i);
            Pose {
                x: p[0],
                y: p[1],
                heading: 0.0,
            }
        };
        let plain_ids: Vec<usize> = (0..n as usize).filter(|i| !gold_at.contains(i)).collect();
        let blue = fish(Role::unmarked());
        let plain = Swarm::build(
            stage,
            &blue.mob.clone().scaled_about(big, ORIGIN),
            &plain_ids.iter().map(|&i| pose(i)).collect::<Vec<_>>(),
            None,
        )?;
        let gold_glyph = marked_fish();
        let gold = Swarm::build(
            stage,
            &gold_glyph.mob.clone().scaled_about(big, ORIGIN),
            &gold_at.iter().map(|&i| pose(i)).collect::<Vec<_>>(),
            None,
        )?;
        let ring_glyph = ring();
        let rings = Swarm::build(
            stage,
            &ring_glyph.mob.clone().scaled_about(big, ORIGIN),
            &(0..n as usize).map(pose).collect::<Vec<_>>(),
            None,
        )?;
        let data = plated(
            &kit,
            stage,
            &format!("marked {m}  ·  caught {n}  ·  caught twice {k}"),
            30.0,
            Palette::ink(),
            v(0.0, row_y + 0.95),
        )?;
        fade_in_all(stage, &[card, data], 0.6)?;
        // one fish at a time through the listen window (the score: seven
        // kalimba notes, three bells)
        let mut each: Vec<Mob> = Vec::new();
        let (mut pi, mut gi) = (0usize, 0usize);
        for i in 0..n as usize {
            if gold_at.contains(&i) {
                each.push(gold.members[gi]);
                gi += 1;
            } else {
                each.push(plain.members[pi]);
                pi += 1;
            }
        }
        for &f in &each {
            stage.set_opacity(f, 0.0, true);
        }
        stage.add_to_scene(plain.group)?;
        stage.add_to_scene(gold.group)?;
        stage.add_to_scene(rings.group)?;
        // every ring starts hidden; each comes in with its fish
        Shade::new(stage, &rings.members, &ring_glyph.paints, 0.0)?;
        // the window: each fish a note as it appears, a bell for a marked
        // one (seven plinks, three bells: k = 3 heard)
        let step = 2.6 / n as f64;
        let plinks: Vec<Plink> = (0..n as usize)
            .map(|i| Plink {
                at: step * i as f64,
                marked: gold_at.contains(&i),
            })
            .collect();
        let window = plinks.last().map_or(0.0, |p| p.at) + SCORE_FADE_S + 0.5;
        assert!(
            plinks.iter().all(|p| p.at <= last_onset(window)),
            "05_1: a note in the score's fade"
        );
        let wav = catch_sound("a05_1_catch", &plinks, window).map_err(|e| sound_error("05_1", &e))?;
        kit.sonify(stage, window, "05_1", &wav)?;
        for (i, &f) in each.iter().enumerate() {
            let is_gold = gold_at.contains(&i);
            let paints = if is_gold { &gold_glyph.paints } else { &blue.paints };
            let s = Shade::new(stage, &[f], paints, 0.0)?;
            let r = Shade::new(stage, &[rings.members[i]], &ring_glyph.paints, 0.0)?;
            play!(stage; s.to(1.0, step * 0.8)?, r.to(1.0, step * 0.8)?);
            stage.wait(step * 0.2)?;
        }
        stage.wait((window - step * n as f64).max(0.0))?;

        // 05_2: the soup guess first
        let soup_q = kit.tex(
            stage,
            &format!(r"\frac{{{m} \times {n}}}{{{k}}} \approx\ ?"),
            56.0,
            Palette::ink(),
            &[],
            v(0.0, 0.6),
        )?;
        kit.say(stage, "05_2")?;
        fade_out_all(stage, &[mystery], 0.4)?;
        play!(stage; write(stage.arena(), soup_q).rt(1.0));
        let q = predict_pause(&kit, stage, v(3.2, 0.6))?;

        // 05_3: about 33; the smallest the lake could be is the 17 fish seen
        let soup = (m * n) as f64 / k as f64;
        let soup_a = kit.tex(
            stage,
            &format!(r"\frac{{{m} \times {n}}}{{{k}}} \approx {:.0}", soup),
            56.0,
            Palette::ink(),
            &[],
            v(0.0, 0.6),
        )?;
        kit.say(stage, "05_3")?;
        fade_out_all(stage, &[q], 0.2)?;
        play!(stage; fmn::animation::replacement_transform(soup_q, soup_a).rt(0.8));
        stage.wait(0.6)?;
        // the lake fades; the curve's axis comes in, a wall at the smallest
        let mut lake_parts = vec![art, fog, name, card, data, plain.group, gold.group, rings.group];
        lake_parts.push(soup_a);
        fade_out_all(stage, &lake_parts, 0.6)?;
        let frame = curve::frame();
        let axis = curve::axis(&kit, stage, &frame)?;
        fade_in_all(stage, &[axis], 0.6)?;
        let wall_x = frame.p(first as f64 - 0.5, 0.0)[0];
        let impossible = stage.arena_mut().add(rect(
            v(frame.left(), frame.bottom()),
            wall_x - frame.left(),
            frame.height,
            flat(Role::bias(), 0.12),
        )?);
        let wall = stage.arena_mut().add(
            Line::new(v(wall_x, frame.bottom()), v(wall_x, frame.top()))
                .style(line_style(Role::bias(), 3.0))
                .build()?,
        );
        let seen = plated(
            &kit,
            stage,
            &format!("{m} + {n} − {k} = {first} different fish seen"),
            26.0,
            Role::bias(),
            v(wall_x + 2.6, frame.top() - 0.35),
        )?;
        fade_in_all(stage, &[impossible, wall, seen], 0.6)?;
        kit.hold(stage)?;

        // 05_4: three lake sizes: the chance of exactly three marked, each
        // dropped onto the curve as that size's bar
        let hist_axis = hist_axis(&kit, stage)?;
        let hist_title_at = v(hist_frame().center[0], hist_frame().top() + 0.25);
        kit.say(stage, "05_4")?;
        fade_out_all(stage, &[seen], 0.3)?;
        fade_in_all(stage, &[hist_axis], 0.4)?;
        // each lake comes in as its own sentence begins ("A lake of twenty,
        // …"), before its number is said, and stays until the next is named
        let mut still_bars = Vec::new();
        let mut on_show: Vec<Mob> = Vec::new();
        for (j, &lake) in STILLS.iter().enumerate() {
            let h = stage.arena_mut().add(hist_bars(lake)?);
            let title = kit.text(
                stage,
                &format!("if the lake had {lake} fish"),
                24.0,
                Palette::ink(),
                hist_title_at,
            )?;
            let score = curve::score(lake);
            let pct = plated(
                &kit,
                stage,
                &format!("{:.1}%", 100.0 * score),
                24.0,
                Role::score(),
                v(
                    hist_frame().p(TOY.2 as f64, score)[0] + 0.15,
                    hist_frame().p(0.0, score)[1] + 0.25,
                ),
            )?;
            kit.until_sentence(stage, "05_4", j + 1)?;
            if !on_show.is_empty() {
                fade_out_all(stage, &on_show, 0.2)?;
            }
            fade_in_all(stage, &[h, title, pct], 0.3)?;
            on_show = vec![h, title, pct];
            // the k = 3 bar drops onto the curve at this size
            let landed = stage.arena_mut().add(curve::bar(&frame, lake, score)?);
            let from = hist_frame().p(TOY.2 as f64, 0.0);
            let to = frame.p(lake as f64, 0.0);
            let tag = plated(
                &kit,
                stage,
                &format!("{:.1}%", 100.0 * score),
                22.0,
                Role::score(),
                v(to[0], frame.p(0.0, score)[1] + 0.25),
            )?;
            play!(stage;
                fade_in(stage.arena_mut(), landed, v(to[0] - from[0], to[1] - from[1]), 1.0)?.rt(0.7),
            );
            fade_in_all(stage, &[tag], 0.3)?;
            still_bars.push((landed, tag));
        }
        kit.hold(stage)?;

        // 05_5: every size: the lake size sweeps, the histogram follows,
        // and each size's bar lands on the curve as the sweep passes it
        let sweep = stage.add_value_tracker(first as f64);
        stage.add_to_scene(sweep)?;
        let live_hist = stage.always_redraw(move |s| {
            let lake = s.tracker_value(sweep).unwrap_or(first as f64).round() as u64;
            let bars = hist_bars(lake).expect("positive bar sizes");
            s.add(bars)
        });
        let book = Rc::clone(&kit.book);
        let live_title = stage.always_redraw(move |s| {
            let lake = s.tracker_value(sweep).unwrap_or(first as f64).round();
            let t = Text::new(&format!("if the lake had {lake} fish"))
                .font_size(24.0)
                .style(explainer_kit::ink_style(Palette::ink()))
                .build(&book)
                .expect("bundled text lays out");
            let m = s.add(t);
            s.move_to(m, hist_title_at, ORIGIN);
            m
        });
        let bars: Vec<Mob> = curve::bars(&frame)?
            .into_iter()
            .map(|b| stage.arena_mut().add(b))
            .collect();
        for &b in &bars {
            stage.set_opacity(b, 0.0, true);
        }
        let bar_group = stage.arena_mut().add(VMobject::new());
        for &b in &bars {
            stage.attach(bar_group, b)?;
        }
        stage.add_to_scene(bar_group)?;
        let sizes: Vec<u64> = curve::sizes().collect();
        let lit = bars.clone();
        let mut shown = vec![false; lit.len()];
        stage.add_dt_updater(
            bar_group,
            move |s, _, _| {
                let now = s.tracker_value(sweep).unwrap_or(0.0);
                for (i, (&b, &size)) in lit.iter().zip(&sizes).enumerate() {
                    if !shown[i] && now >= size as f64 - 0.5 {
                        s.set_fill(b, None, Some(0.9), None, true);
                        shown[i] = true;
                    }
                }
            },
            false,
        )?;
        kit.say(stage, "05_5")?;
        stage.add_to_scene(live_hist)?;
        stage.add_to_scene(live_title)?;
        let mut stills_out = on_show.clone();
        for &(b, t) in &still_bars {
            stills_out.extend([b, t]);
        }
        fade_out_all(stage, &stills_out, 0.3)?;
        let sweep_s = 4.5;
        // the window: a soft tone for each size as the sweep passes it,
        // swelling with the score
        let span = (curve::SIZE_MAX - first) as f64;
        let passes: Vec<(f64, f64)> = curve::sizes()
            .map(|size| {
                (
                    sweep_s * (size - first) as f64 / span,
                    curve::score(size) / curve::score(best_sizes(TOY).last().copied().unwrap_or(first)),
                )
            })
            .collect();
        let sweep_window = sweep_s + SCORE_FADE_S + 0.4;
        assert!(
            passes.iter().all(|&(at, _)| at <= last_onset(sweep_window)),
            "05_5: a tone in the score's fade"
        );
        let wav = sweep_sound("a05_5_sweep", &passes, sweep_window).map_err(|e| sound_error("05_5", &e))?;
        kit.sonify(stage, sweep_window, "05_5", &wav)?;
        let run = sweep
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(sweep_s),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(curve::SIZE_MAX as f64)?;
        stage.play(run)?;
        stage.wait(sweep_window - sweep_s)?;

        // 05_6: the peak is the soup guess; the name: the likelihood, a score
        let best = best_sizes(TOY).last().copied().unwrap_or(first);
        let peak = curve::score(best);
        let top = frame.p(best as f64, peak);
        let crown = stage.arena_mut().add(rect(
            v(top[0] - 0.16, frame.bottom()),
            0.32,
            top[1] - frame.bottom() + 0.08,
            Style::default()
                .fill(Role::score(), 0.0)
                .stroke(Palette::ink(), 3.0, 1.0),
        )?);
        let peak_tag = plated(
            &kit,
            stage,
            &format!("peak at {best}: score ≈ {:.1}%", 100.0 * peak),
            26.0,
            Palette::ink(),
            v(top[0] + 2.4, top[1] + 0.15),
        )?;
        let name_card = plated(
            &kit,
            stage,
            "score of a lake size  =  how likely our catch would be if the lake were that size",
            24.0,
            Role::score(),
            v(1.2, 2.55),
        )?;
        let likelihood = kit.text(stage, "the likelihood", 34.0, Role::score(), v(1.2, 3.35))?;
        kit.say(stage, "05_6")?;
        fade_out_all(stage, &[live_hist, live_title, hist_axis], 0.4)?;
        fade_in_all(stage, &[crown, peak_tag], 0.6)?;
        fade_in_all(stage, &[likelihood], 0.6)?;
        fade_in_all(stage, &[name_card], 0.6)?;
        kit.hold(stage)?;

        // 05_7: add every height, out to lakes as big as you like
        let sums = partial_sums(SUM_TO);
        let to60 = sums[(curve::SIZE_MAX - first) as usize];
        let reach = stage.add_value_tracker(curve::SIZE_MAX as f64);
        stage.add_to_scene(reach)?;
        // the sum so far, and the area out to the reach (one polygon,
        // rebuilt each frame: a few hundred points)
        let book = Rc::clone(&kit.book);
        let sums_live = sums.clone();
        let sum_at = v(4.2, 1.6);
        let live_sum = stage.always_redraw(move |s| {
            let r = s.tracker_value(reach).unwrap_or(60.0).round() as u64;
            let i = (r.max(first) - first) as usize;
            let total = sums_live[i.min(sums_live.len() - 1)];
            let t = Text::new(&format!("{first} to {}: {:.2}", thousands(r), total))
                .font_size(30.0)
                .style(explainer_kit::ink_style(Role::score()))
                .build(&book)
                .expect("bundled text lays out");
            let m = s.add(t);
            s.move_to(m, sum_at, ORIGIN);
            m
        });
        let area = stage.always_redraw(move |s| {
            let r = s.tracker_value(reach).unwrap_or(60.0);
            let f = Frame2 {
                x: (frame.x.0, r + 1.5),
                ..frame
            };
            let mut pts = vec![f.p(first as f64 - 0.5, 0.0)];
            let steps = 240usize;
            for j in 0..=steps {
                // denser near the start, where the score lives
                let u = j as f64 / steps as f64;
                let size = first as f64 - 0.5 + (r + 0.5 - first as f64 + 0.5) * u * u;
                let lo = size.floor().max(first as f64) as u64;
                pts.push(f.p(size, curve::score(lo)));
            }
            pts.push(f.p(r + 0.5, 0.0));
            s.add(Polygon::new(pts).style(flat(Role::score(), 0.75)).build())
        });
        let sum_60 = plated(
            &kit,
            stage,
            &format!("{first} to {}: {to60:.2}", curve::SIZE_MAX),
            30.0,
            Role::score(),
            sum_at,
        )?;
        kit.say(stage, "05_7")?;
        fade_out_all(stage, &[name_card, peak_tag, crown, likelihood], 0.4)?;
        fade_in_all(stage, &[sum_60], 0.5)?;
        stage.wait(0.8)?;
        // the bars become one area, and the axis runs out to 2,000
        stage.add_to_scene(area)?;
        stage.clear_updaters(bar_group, false);
        fade_out_all(stage, &[bar_group, axis, sum_60, impossible, wall], 0.4)?;
        stage.add_to_scene(live_sum)?;
        let far_axis = kit.text(
            stage,
            &format!("lake size, out to {}", thousands(SUM_TO)),
            26.0,
            Palette::dim(),
            v(frame.center[0], frame.bottom() - 0.45),
        )?;
        let floor = stage.arena_mut().add(
            Line::new(v(frame.left(), frame.bottom()), v(frame.right(), frame.bottom()))
                .style(line_style(Palette::dim(), 2.5))
                .build()?,
        );
        fade_in_all(stage, &[floor, far_axis], 0.4)?;
        play!(stage; tween(reach, SUM_TO as f64, 4.0)?);
        // all sizes: Mn/(k(k−1)) exactly (A-F7)
        let all_sizes = (m * n) as f64 / (k * (k - 1)) as f64;
        let total = plated(
            &kit,
            stage,
            &format!(
                "all sizes: {all_sizes:.2}  =  {}%",
                thousands((100.0 * all_sizes).round() as u64)
            ),
            32.0,
            Role::score(),
            v(sum_at[0], sum_at[1] - 0.75),
        )?;
        fade_in_all(stage, &[total], 0.6)?;
        // real chances add to one: a hundred percent
        let unit = 0.12;
        let base = frame.bottom() + 0.3;
        let real = stage
            .arena_mut()
            .add(rect(v(2.6, base), 0.6, unit, flat(Role::chance(), 0.9))?);
        let ours = stage
            .arena_mut()
            .add(rect(v(4.6, base), 0.6, unit * all_sizes, flat(Role::score(), 0.9))?);
        let real_tag = kit.text(
            stage,
            "real chances add to 100%",
            22.0,
            Role::chance(),
            v(2.9, base + unit + 0.3),
        )?;
        let ours_tag = kit.text(
            stage,
            "these scores",
            22.0,
            Role::score(),
            v(4.9, base + unit * all_sizes + 0.25),
        )?;
        fade_in_all(stage, &[real, real_tag], 0.5)?;
        fade_in_all(stage, &[ours, ours_tag], 0.5)?;
        kit.hold(stage)?;
        // Feller's own warning, in the hold after the line
        let feller = kit.text_styled(
            stage,
            "\u{201c}It is, therefore, meaningless to ask for the probability that n is greater than, say, 6000.\u{201d}",
            26.0,
            Palette::ink(),
            v(0.0, 3.25),
            false,
            true,
        )?;
        let feller_cite = kit.text(
            stage,
            "Feller (1950); his n is our lake size",
            20.0,
            Palette::dim(),
            v(0.0, 2.8),
        )?;
        fade_in_all(stage, &[feller, feller_cite], 0.5)?;
        let note = precisely(
            &kit,
            stage,
            &format!(
                "the scores over every lake size sum to Mn/(k(k−1)) = {}/{} when k ≥ 2 (infinite when k ≤ 1)",
                (m * n) / gcd(m * n, k * (k - 1)),
                (k * (k - 1)) / gcd(m * n, k * (k - 1))
            ),
        )?;
        fade_in_all(stage, &[note], 0.4)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        kit.hold(stage)?;
        // back to the curve itself, 17 to 60: where chapter 06 begins
        fade_all(stage, 0.4)?;
        let axis = curve::axis(&kit, stage, &frame)?;
        let bars: Vec<Mob> = curve::bars(&frame)?
            .into_iter()
            .map(|b| stage.arena_mut().add(b))
            .collect();
        let mut last = vec![axis];
        last.extend(bars);
        fade_in_all(stage, &last, 0.4)?;
        kit.finish(stage)
    }
}

/// A sonification that could not be written, as a scene error.
fn sound_error(id: &str, e: &std::io::Error) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        format!("{id} sonification: {e}").into_boxed_str(),
    )))
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stills_and_sums_are_the_ledgers() {
        // A-F5.still_N{20,33,60}_pct: 7.8 / 31.8 / 15.9
        let pct: Vec<String> = STILLS
            .iter()
            .map(|&n| format!("{:.1}", 100.0 * curve::score(n)))
            .collect();
        assert_eq!(pct, ["7.8", "31.8", "15.9"]);
        let sums = partial_sums(SUM_TO);
        // A-F5.area_17_to_60 = 9.89
        assert_eq!(format!("{:.2}", sums[(curve::SIZE_MAX - 17) as usize]), "9.89");
        // out to 2,000 the running sum is still a little under all sizes'
        // 50/3 (A-F7): the sizes past it add about 0.015
        let to_2000 = sums.last().copied().unwrap_or(0.0);
        assert!(to_2000 < 50.0 / 3.0 && 50.0 / 3.0 - to_2000 < 0.02, "{to_2000}");
    }
}
