//! Chapter 7 — what D measures: the distance from independence. Shuffling
//! Y keeps both marginals (the rugs) and destroys only the pairing; D,
//! recomputed live from the moving dots every frame, falls to ~0. Then a
//! permutation test: 2,000 shuffles against the observed value, heard as
//! well as seen (a listen window: one grain per shuffle at the pitch of its
//! D, then the observed value as a bell dyad far above the pile).

use explainer_score::instr::{self, Instrument, Note};
use explainer_score::sonify::{GRAIN_S, Grain, cloud, snap};
use explainer_score::units::db_to_gain;
use explainer_score::{FS, Stereo, meter};
use fmn::animation::grow_from_edge;
use fmn::prelude::*;

use crate::kit::{
    Frame2, Kit, Palette, Timed, fade_all, hex, lagged, point_colors, stroke_style, v,
};
use crate::play;
use crate::stats::{self, Data, Shape};

pub struct Shuffle {
    pub kit: Kit,
}

const N: usize = 48;
const BINS: usize = 4;
const HEAT_CELL: f64 = 0.82;
const HEAT_CENTER: Vec3 = [3.65, 0.95, 0.0];

/// The listen window (s): the shuffles, the observed dyad, its ring.
const LISTEN_S: f64 = 4.5;
/// The grain cloud's length (s); the shuffle counter runs in step with it.
const CLOUD_S: f64 = 2.8;
/// How the shuffles rush in: after t of `CLOUD_S`, 2,000 · (t / CLOUD_S)^RUSH
/// have sounded (a few ticks, then a flood).
const RUSH: f64 = 2.5;
/// D major pentatonic: the data stays in the video's key.
const PENTATONIC: [u8; 5] = [2, 4, 6, 9, 11];

/// The counter's rate function: the share of shuffles heard by time `u`.
fn rush(u: f64) -> f64 {
    u.clamp(0.0, 1.0).powf(RUSH)
}

/// The pitch a value of D sounds at: the histogram's axis, low to high, over
/// two octaves up from E5, snapped into the key.
fn d_pitch(d: f64, axis: (f64, f64)) -> f64 {
    let u = ((d - axis.0) / (axis.1 - axis.0)).clamp(0.0, 1.0);
    snap(76.0 + 24.0 * u, &PENTATONIC)
}

/// The 2,000 shuffles heard: grain k starts as the counter reaches k + 1, at
/// the pitch of its D, so chance sounds as a narrow shimmer; then the
/// observed D as a vibraphone dyad (the A and the D above it) at its own
/// height, far above. Levelled at the narration's −23 LUFS; the score sets
/// the final level.
fn shuffle_sound(null: &[f64], observed: f64, axis: (f64, f64)) -> Stereo {
    let n = null.len() as f64;
    let mut rng = explainer_score::rng::Rng::new(0x0005_EEDD);
    let grains: Vec<Grain> = null
        .iter()
        .enumerate()
        .map(|(k, &d)| {
            let start = CLOUD_S * ((k + 1) as f64 / n).powf(1.0 / RUSH);
            // grains overlap as they rush: soften each, so the cloud swells
            // by a few dB rather than twenty
            let density = RUSH * n / CLOUD_S * (start / CLOUD_S).powf(RUSH - 1.0);
            Grain {
                start,
                pitch: d_pitch(d, axis),
                pan: rng.uniform() - 0.5,
                amp: 0.3 * (1.0 + density * GRAIN_S).powf(-0.4),
            }
        })
        .collect();
    let mut s = cloud(&grains, LISTEN_S);
    let top = d_pitch(observed, axis);
    let a = (0..12)
        .map(|k| top - f64::from(k))
        .find(|p| (*p as i64).rem_euclid(12) == 9)
        .unwrap_or(top);
    for (j, pitch) in [a, a + 5.0].into_iter().enumerate() {
        let note = Note {
            pitch,
            velocity: 0.55,
            hold: 1.0,
            seed: 7 + j as u64,
            pan: if j == 0 { -0.2 } else { 0.2 },
        };
        let at = ((CLOUD_S + 0.05 + 0.03 * j as f64) * FS) as usize;
        s.mix_in(&instr::render(Instrument::Vibraphone, &note), at, 0.5);
    }
    // the window ends in silence
    let fade = (0.3 * FS) as usize;
    let len = s.len();
    for k in 0..fade {
        let g = 0.5 - 0.5 * (std::f64::consts::PI * k as f64 / fade as f64).cos();
        s.l[len - 1 - k] *= g;
        s.r[len - 1 - k] *= g;
    }
    s.scale(db_to_gain(-23.0 - meter::integrated(&s)));
    s
}

/// Current (x, y) in rank units for every dot, read from the live arena.
fn positions(s: &fmn::mobject::Stage, dots: &[Mob], f: &Frame2) -> (Vec<f64>, Vec<f64>) {
    dots.iter().map(|&d| f.data(s.get_center(d))).unzip()
}

fn heat_color(dev: f64) -> Srgb {
    let neutral = hex("#2A303C");
    if dev >= 0.0 {
        fmn::core::color::interpolate_color(neutral, hex("#F0AC5F"), (dev / 6.0).min(1.0))
    } else {
        fmn::core::color::interpolate_color(neutral, hex("#1F5F8B"), (-dev / 3.0).min(1.0))
    }
}

impl SceneConstruct for Shuffle {
    fn name(&self) -> &str {
        "07_shuffle"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let raw = stats::shape_points(Shape::Parabola, N, 77);
        let rx = stats::ranks(&raw.iter().map(|p| p.0).collect::<Vec<_>>());
        let ry = stats::ranks(&raw.iter().map(|p| p.1).collect::<Vec<_>>());
        let observed = stats::hoeffding(&rx, &ry).d;
        let mut rng = Data::new(2024);
        let perm = stats::permutation(N, &mut rng);
        let ry_shuffled: Vec<f64> = perm.iter().map(|&k| ry[k]).collect();

        let top = N as f64 + 1.0;
        let f = Frame2 {
            center: v(-3.45, -0.4),
            width: 5.6,
            height: 5.6,
            x: (0.0, top),
            y: (0.0, top),
        };
        let title = kit.text(
            stage,
            "What D measures: the distance from independence",
            40.0,
            Palette::ink(),
            v(0.0, 3.45),
        )?;
        kit.say(stage, "s1")?;
        play!(stage; write(stage.arena(), title).rt(1.3));

        let border = stage.arena_mut().add(f.border(Palette::dim())?);
        let rl = kit.tex(
            stage,
            "R",
            30.0,
            Palette::x_rank(),
            &[],
            v(f.right() + 0.25, f.bottom() - 0.3),
        )?;
        let sl = kit.tex(
            stage,
            "S",
            30.0,
            Palette::y_rank(),
            &[],
            v(f.left() - 0.45, f.top()),
        )?;
        let colors = point_colors(N);
        let mut dots = Vec::new();
        for k in 0..N {
            let d = Dot::new()
                .point(f.p(rx[k], ry[k]))
                .radius(0.075)
                .color(colors[(rx[k] as usize) - 1])
                .build();
            dots.push(stage.arena_mut().add(d));
        }
        play!(stage;
            show_creation(border).rt(0.9),
            fade_in(stage.arena_mut(), rl, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), sl, ORIGIN, 1.0)?,
        );
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &d in &dots {
            let a = fade_in(stage.arena_mut(), d, ORIGIN, 0.2)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.04)?;
        play!(stage; g.rt(1.6));

        // ---- the marginals as rugs, redrawn from the live dots
        let rug_dots = dots.clone();
        let rugs = stage.always_redraw(move |s| {
            let (xs, ys) = positions(s, &rug_dots, &f);
            let mut marks = Vec::with_capacity(2 * xs.len());
            for x in xs {
                let p = f.p(x, 0.0);
                marks.push(
                    Line::new(v(p[0], f.bottom() - 0.08), v(p[0], f.bottom() - 0.3))
                        .style(stroke_style(Palette::x_rank(), 2.0))
                        .build()
                        .expect("a rug tick"),
                );
            }
            for y in ys {
                let p = f.p(0.0, y);
                marks.push(
                    Line::new(v(f.left() - 0.08, p[1]), v(f.left() - 0.3, p[1]))
                        .style(stroke_style(Palette::y_rank(), 2.0))
                        .build()
                        .expect("a rug tick"),
                );
            }
            s.add(v_group(marks))
        });
        stage.add_to_scene(rugs)?;
        let marg = kit.text(
            stage,
            "rugs = the marginals: where X values fall, where Y values fall",
            24.0,
            Palette::dim(),
            v(-3.45, -3.72),
        )?;
        kit.say(stage, "s2")?;
        play!(stage; fade_in(stage.arena_mut(), marg, v(0.0, 0.15), 1.0)?.rt(1.0));

        // ---- the joint, as a live 4x4 heat map of rank-quartile counts
        let heat_dots = dots.clone();
        let book = std::rc::Rc::clone(&kit.book);
        let heat = stage.always_redraw(move |s| {
            let (xs, ys) = positions(s, &heat_dots, &f);
            let per = N as f64 / BINS as f64;
            let mut counts = [[0usize; BINS]; BINS];
            for (x, y) in xs.iter().zip(&ys) {
                let bx = (((x - 0.5) / per).floor() as isize).clamp(0, BINS as isize - 1) as usize;
                let by = (((y - 0.5) / per).floor() as isize).clamp(0, BINS as isize - 1) as usize;
                counts[by][bx] += 1;
            }
            let expected = N as f64 / (BINS * BINS) as f64;
            let mut cells = Vec::new();
            for (by, row) in counts.iter().enumerate() {
                for (bx, &count) in row.iter().enumerate() {
                    let cx = HEAT_CENTER[0] + (bx as f64 - 1.5) * HEAT_CELL;
                    let cy = HEAT_CENTER[1] + (by as f64 - 1.5) * HEAT_CELL;
                    let fill = heat_color(count as f64 - expected);
                    let r = Rectangle::new()
                        .width(HEAT_CELL - 0.06)
                        .height(HEAT_CELL - 0.06)
                        .style(Style::default().fill(fill, 1.0).stroke(fill, 0.0, 1.0))
                        .build()
                        .expect("a heat cell")
                        .moved_to(v(cx, cy));
                    cells.push(r);
                    let n = DecimalNumber::new(count as f64)
                        .num_decimal_places(0)
                        .font_size(24.0)
                        .color(Palette::ink())
                        .build(&book)
                        .expect("digits")
                        .into_vmob()
                        .moved_to(v(cx, cy));
                    cells.push(n);
                }
            }
            s.add(v_group(cells))
        });
        let heat_title = kit.text(
            stage,
            "the joint: counts per rank-quartile cell",
            24.0,
            Palette::dim(),
            v(HEAT_CENTER[0], HEAT_CENTER[1] + 2.0 * HEAT_CELL + 0.3),
        )?;
        let heat_note = kit.text(
            stage,
            "independence: every cell ≈ 3",
            24.0,
            Palette::dim(),
            v(HEAT_CENTER[0], HEAT_CENTER[1] - 2.0 * HEAT_CELL - 0.3),
        )?;
        kit.say(stage, "s3")?;
        stage.add_to_scene(heat)?;
        play!(stage;
            fade_in(stage.arena_mut(), heat, ORIGIN, 0.8)?.rt(1.0),
            fade_in(stage.arena_mut(), heat_title, ORIGIN, 1.0)?,
            fade_in(stage.arena_mut(), heat_note, ORIGIN, 1.0)?,
        );

        // ---- D, computed live from the dots every frame
        let d_label = kit.tex(
            stage,
            "D =",
            48.0,
            Palette::hoeffding(),
            &[],
            v(2.85, -2.35),
        )?;
        kit.say(stage, "s4")?;
        play!(stage; fade_in(stage.arena_mut(), d_label, ORIGIN, 1.0)?.rt(0.6));
        let live_dots = dots.clone();
        kit.live_value(
            stage,
            move |s| {
                let (xs, ys) = positions(s, &live_dots, &f);
                stats::hoeffding(&xs, &ys).d
            },
            3,
            true,
            48.0,
            Palette::hoeffding(),
            v(3.45, -2.35),
            LEFT,
        )?;
        stage.wait(1.6)?;

        // ---- shuffle Y: marginals stay, the pairing goes
        let cue = kit.text(
            stage,
            "Shuffle the Y values among the points...",
            30.0,
            Palette::ink(),
            v(3.65, -3.35),
        )?;
        kit.say(stage, "s5")?;
        play!(stage; fade_in(stage.arena_mut(), cue, v(0.0, 0.15), 1.0)?.rt(0.8));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (k, &d) in dots.iter().enumerate() {
            let a = d.animate().move_to(f.p(rx[k], ry_shuffled[k]), ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(3.5),
                ..PlayOverrides::default()
            },
        )?;
        kit.say(stage, "s6")?;
        let same = kit.text(
            stage,
            "same marginals, pairing destroyed: D ≈ 0",
            30.0,
            Palette::good(),
            v(3.65, -3.35),
        )?;
        // The rugs are redrawn every frame, so highlight them from outside.
        let rug_x = SurroundingRectangle::from_extent(Some((
            v(f.left(), f.bottom() - 0.32),
            v(f.right(), f.bottom() - 0.06),
        )))
        .buff(0.06)
        .color(Palette::good())
        .build();
        let rug_y = SurroundingRectangle::from_extent(Some((
            v(f.left() - 0.32, f.bottom()),
            v(f.left() - 0.06, f.top()),
        )))
        .buff(0.06)
        .color(Palette::good())
        .build();
        let rug_x = stage.arena_mut().add(rug_x);
        let rug_y = stage.arena_mut().add(rug_y);
        play!(stage;
            fmn::animation::replacement_transform(cue, same).rt(0.8),
            show_creation(rug_x).rt(0.8),
            show_creation(rug_y).rt(0.8),
        );
        stage.wait(1.6)?;
        play!(stage;
            fade_out(stage.arena_mut(), rug_x, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), rug_y, ORIGIN, 1.0)?,
        );
        stage.wait(0.6)?;
        let back = kit.text(
            stage,
            "...and put them back.",
            30.0,
            Palette::ink(),
            v(3.65, -3.35),
        )?;
        kit.say(stage, "s7")?;
        play!(stage; fmn::animation::replacement_transform(same, back).rt(0.6));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (k, &d) in dots.iter().enumerate() {
            let a = d.animate().move_to(f.p(rx[k], ry[k]), ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(2.5),
                ..PlayOverrides::default()
            },
        )?;
        stage.wait(1.4)?;

        // ---- the permutation test
        let mut null = Vec::with_capacity(2000);
        for _ in 0..2000 {
            let p = stats::permutation(N, &mut rng);
            let ys: Vec<f64> = p.iter().map(|&k| ry[k]).collect();
            null.push(stats::hoeffding(&rx, &ys).d);
        }
        let exceed = null.iter().filter(|&&d| d >= observed).count();
        play!(stage;
            fade_out(stage.arena_mut(), heat, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), heat_title, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), heat_note, ORIGIN, 1.0)?,
            fade_out(stage.arena_mut(), back, ORIGIN, 1.0)?,
        );
        stage.clear_updaters(heat, true);
        let hist = Frame2 {
            center: v(3.65, 0.65),
            width: 5.6,
            height: 3.2,
            x: (-0.04, 0.2),
            y: (0.0, 1.0),
        };
        let axis = Line::new(
            v(hist.left(), hist.bottom()),
            v(hist.right(), hist.bottom()),
        )
        .style(stroke_style(Palette::dim(), 2.0))
        .build()?;
        let axis = stage.arena_mut().add(axis);
        let mut labels = Vec::new();
        for x in [0.0, 0.1, 0.2] {
            let p = hist.p(x, 0.0);
            labels.push(kit.text(
                stage,
                &format!("{x:.1}"),
                22.0,
                Palette::dim(),
                v(p[0], p[1] - 0.28),
            )?);
        }
        let head = kit.text_left(
            stage,
            "D for 2,000 random shuffles of Y",
            26.0,
            Palette::dim(),
            v(hist.left(), hist.top() + 0.35),
        )?;
        let width = 0.005;
        let nbins = ((hist.x.1 - hist.x.0) / width).round() as usize;
        let mut counts = vec![0usize; nbins];
        for &d in &null {
            let b = ((d - hist.x.0) / width).floor() as isize;
            if (0..nbins as isize).contains(&b) {
                counts[b as usize] += 1;
            }
        }
        let peak = *counts.iter().max().unwrap_or(&1) as f64;
        let mut bars = Vec::new();
        for (b, &c) in counts.iter().enumerate() {
            if c == 0 {
                continue;
            }
            let x0 = hist.x.0 + b as f64 * width;
            let dl = hist.p(x0, 0.0);
            let w = hist.p(x0 + width, 0.0)[0] - dl[0];
            let hgt = (c as f64 / peak) * (hist.height - 0.2);
            let bar = crate::kit::rect(
                dl,
                w - 0.01,
                hgt,
                Style::default()
                    .fill(Palette::dim(), 0.9)
                    .stroke(Palette::dim(), 0.0, 1.0),
            )?;
            bars.push(stage.arena_mut().add(bar));
        }
        kit.say(stage, "s8")?;
        play!(stage;
            show_creation(axis).rt(0.8),
            fade_in(stage.arena_mut(), head, ORIGIN, 1.0)?,
        );
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &l in &labels {
            let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        for &b in &bars {
            let a = grow_from_edge(stage.arena_mut(), b, DOWN, None)?;
            anims.push(stage.prepare(a)?);
        }
        let g = lagged(stage, anims, 0.03)?;
        play!(stage; g.rt(1.6));
        let ox = hist.p(observed, 0.0)[0];
        let obs = Line::new(v(ox, hist.bottom()), v(ox, hist.top()))
            .style(stroke_style(Palette::hoeffding(), 5.0))
            .build()?;
        let obs = stage.arena_mut().add(obs);
        let obs_l = kit.text(
            stage,
            "observed",
            24.0,
            Palette::hoeffding(),
            v(ox, hist.top() + 0.35),
        )?;
        stage.move_to(obs_l, v(ox + 0.12, hist.top() - 0.15), LEFT);
        // ---- listen: the shuffles heard, the counter in step, then the
        // observed value rings far above the pile, before it is named
        let sound = shuffle_sound(&null, observed, hist.x);
        let wav = explainer_kit::sound_file("hoeffding_shuffles", &sound).map_err(|e| {
            fmn::Error::Scene(SceneError::InvalidConfig(Box::leak(
                format!("shuffle sonification: {e}").into_boxed_str(),
            )))
        })?;
        kit.sonify(stage, LISTEN_S, "shuffles", &wav)?;
        let count = stage.add_value_tracker(0.0);
        kit.live_count(
            stage,
            count,
            30.0,
            Palette::dim(),
            v(3.0, hist.top() - 0.35),
        )?;
        let counter_l = kit.text(
            stage,
            "shuffles",
            22.0,
            Palette::dim(),
            v(3.0, hist.top() - 0.8),
        )?;
        let rushing = count
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(CLOUD_S),
                rate_func: Some(rush),
                ..AnimateArgs::default()
            })?
            .set_value(null.len() as f64)?;
        play!(stage; rushing, fade_in(stage.arena_mut(), counter_l, ORIGIN, 1.0)?.rt(0.6));
        play!(stage; show_creation(obs).rt(0.9), fade_in(stage.arena_mut(), obs_l, ORIGIN, 1.0)?);
        stage.wait(LISTEN_S - CLOUD_S - 0.9)?;
        kit.say(stage, "s9")?;
        let verdict = kit.text(
            stage,
            &format!("{exceed} of 2,000 shuffles reach it: X and Y are dependent."),
            28.0,
            Palette::good(),
            v(3.65, -3.35),
        )?;
        kit.chime(stage, 880.0)?;
        play!(stage; write(stage.arena(), verdict).rt(1.4));
        stage.wait(1.5)?;
        // "dependent": the motif resolves in the act's closing breath
        kit.hold(stage)?;
        kit.mark(stage, "resolve:dependent");
        kit.music_breath(stage, crate::kit::ACT_BREATH)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 6)?;
        Ok(())
    }
}
