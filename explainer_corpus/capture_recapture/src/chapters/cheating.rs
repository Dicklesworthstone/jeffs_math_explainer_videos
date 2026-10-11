//! Chapter 09, check it by cheating: a lake of exactly 10,000 fish, the
//! experiment run on it 10,000 times (the 09_6 code panel's own runs,
//! `crate::panel`), each one a lit cell (09_1); where the caught-twice
//! counts pile up (09_2–09_3); the bars carried along the hyperbola
//! guess = Mn/k into the skewed spread of the guesses (09_4); the middle 95%
//! and the average (09_5); the panel itself, for programmers (09_6).

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::props::{CodePanel, RowStrips, grow_histogram, predict_pause};
use explainer_kit::{BACKGROUND, Frame2, Kit, Palette, Timed, fade_all, hex, play, rect, thousands, tween, v};
use fmn::animation::grow_arrow;
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{flat, line_style};
use crate::model::SEED;
use crate::panel::{self, Stats};
use crate::runs::{CAUGHT, LAKE, MARKED};

/// The listen window (09_1): the first runs light one at a time, each with
/// its plink, then the rest wash in, inside these five seconds.
const WINDOW_S: f64 = 5.0;
const ONE_BY_ONE: usize = 12;
const STEP_S: f64 = 0.25;
const WASH_S: f64 = 1.8;
/// The pitch of a run that caught the expected hundred (the video's tonic,
/// F4); each fish more or fewer is a semitone, snapped into F Lydian.
const TONIC_MIDI: f64 = 65.0;
const LYDIAN_F: [u8; 7] = [5, 7, 9, 11, 0, 2, 4];
/// The grid: side on screen and centre.
const GRID_SIDE: f64 = 6.2;
const GRID_C: Vec3 = [-3.0, -0.25, 0.0];

/// The plot of 09_3–09_5: caught twice across, the guess up.
fn plot() -> Frame2 {
    Frame2 {
        center: v(0.6, -0.2),
        width: 9.6,
        height: 5.5,
        x: (60.0, 140.0),
        y: (6_000.0, 16_000.0),
    }
}

/// The guess of an experiment that caught `k` twice.
fn guess(k: f64) -> f64 {
    (MARKED * CAUGHT) as f64 / k
}

/// A guess's colour: blue below the true size, teal at it, pale above
/// (the grid's cells and the guesses' bars share it).
fn guess_color(g: f64) -> Srgb {
    let truth = LAKE as f64;
    let pale = hex("#E4F7EF");
    if g < truth {
        interpolate_color(
            Palette::blue(),
            Palette::teal(),
            ((g - 0.8 * truth) / (0.2 * truth)).clamp(0.0, 1.0),
        )
    } else {
        interpolate_color(Palette::teal(), pale, ((g - truth) / (0.25 * truth)).clamp(0.0, 1.0))
    }
}

/// An integer as Python source spells it, `10_000`.
fn py_int(n: usize) -> String {
    thousands(n as u64).replace(',', "_")
}

/// The panel's code, built from the same constants the scene runs.
fn code_lines() -> Vec<String> {
    let runs = crate::runs::PANEL_RUNS;
    vec![
        "import numpy as np".into(),
        "from math import comb".into(),
        format!("rng = np.random.default_rng({SEED})"),
        format!("M = n = {}; N = {}", py_int(MARKED), py_int(LAKE)),
        "marked = np.arange(N) < M".into(),
        format!(
            "k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range({})])",
            py_int(runs)
        ),
        "est = M * n / k".into(),
        "p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]".into(),
        "print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))".into(),
    ]
}

/// How many runs are lit `t` seconds into the lighting.
fn lit_at(t: f64, runs: usize) -> usize {
    let first = ONE_BY_ONE as f64 * STEP_S;
    if t < first {
        ((t / STEP_S).floor() as usize + 1).min(ONE_BY_ONE)
    } else {
        let u = ((t - first) / WASH_S).clamp(0.0, 1.0);
        ONE_BY_ONE + ((runs - ONE_BY_ONE) as f64 * u * u).round() as usize
    }
}

pub struct Cheating {
    pub kit: Kit,
}

impl SceneConstruct for Cheating {
    fn name(&self) -> &str {
        "09_check_it_by_cheating"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let run = panel::run();
        let stats = run
            .stats()
            .map_err(|e| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(e.into_boxed_str()))))?;
        lighting(&kit, stage, &run)?;
        let hist = pile_up(&kit, stage, &run, &stats)?;
        bend(&kit, stage, &stats, hist)?;
        code(&kit, stage, &stats)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// When run `i` lights, `t` seconds into the window (the inverse of
/// [`lit_at`]).
fn lit_time(i: usize, runs: usize) -> f64 {
    if i < ONE_BY_ONE {
        STEP_S * i as f64
    } else {
        // the moment lit_at's rounded count first passes i
        let u = (((i - ONE_BY_ONE) as f64 + 0.5) / (runs - ONE_BY_ONE) as f64).sqrt();
        ONE_BY_ONE as f64 * STEP_S + WASH_S * u
    }
}

/// The run's note: a semitone per fish caught twice away from a hundred.
fn pitch_of(k: u32) -> f64 {
    explainer_score::sonify::snap(TONIC_MIDI + (f64::from(k) - 100.0), &LYDIAN_F)
}

/// The listen window's sound (storyboard 09_1): the first twelve runs as
/// separate vibraphone plinks at the pitch of their caught-twice count,
/// then every 25th later run as a soft grain at the moment it lights (a
/// wash), and the commonest count's note growing out of it. Written at
/// the narration's −23 LUFS.
fn sonification(k: &[u32]) -> std::io::Result<std::path::PathBuf> {
    use explainer_score::instr::{Instrument, Note, render};
    use explainer_score::sonify::{Grain, cloud};
    let runs = k.len();
    let fs = explainer_score::FS;
    let mut grains = Vec::new();
    for i in (ONE_BY_ONE..runs).step_by(25) {
        let pan = ((i * 7919) % 200) as f64 / 100.0 - 1.0;
        grains.push(Grain {
            start: lit_time(i, runs),
            pitch: pitch_of(k[i]),
            pan: 0.6 * pan,
            amp: 0.1,
        });
    }
    let mut out = cloud(&grains, WINDOW_S);
    for (i, &ki) in k.iter().take(ONE_BY_ONE).enumerate() {
        let note = Note::new(pitch_of(ki), 0.75, 0.22, 900 + i as u64);
        out.mix_in(
            &render(Instrument::Vibraphone, &note),
            (lit_time(i, runs) * fs) as usize,
            0.5,
        );
    }
    let mode = (k.iter().copied().min().unwrap_or(0)..=k.iter().copied().max().unwrap_or(0))
        .max_by_key(|&v| k.iter().filter(|&&x| x == v).count())
        .unwrap_or(100);
    let swell = Note::new(pitch_of(mode), 0.55, 1.2, 977);
    let at = ONE_BY_ONE as f64 * STEP_S + 0.6 * WASH_S;
    out.mix_in(&render(Instrument::Vibraphone, &swell), (at * fs) as usize, 0.6);
    let lufs = explainer_score::meter::integrated(&out);
    out.scale(10f64.powf((-23.0 - lufs) / 20.0));
    explainer_kit::sound_file("a09_1_runs", &out)
}

/// 09_1: the act card, then 10,000 experiments as 10,000 lit cells.
fn lighting(kit: &Kit, stage: &mut Stage<'_>, run: &panel::Panel) -> fmn::Result<()> {
    let runs = run.k.len();
    let side = (runs as f64).sqrt().round() as usize;
    assert_eq!(side * side, runs, "the experiments fill a square grid");
    let act = kit.text_styled(
        stage,
        "How sure can we be?",
        64.0,
        Palette::hero(),
        v(0.0, 0.3),
        true,
        false,
    )?;
    kit.say(stage, "09_1")?;
    play!(stage; write(stage.arena(), act).rt(1.0));
    stage.wait(1.0)?;
    play!(stage; fade_out(stage.arena_mut(), act, ORIGIN, 1.0)?.rt(0.5));

    // each grid cell is two thirds lit cell, one third gap, both ways
    let pitch = GRID_SIDE / side as f64;
    let (dot, unlit, gap) = (2.0 * pitch / 3.0, hex("#36425C"), hex(BACKGROUND));
    let ests = run.est.clone();
    let colors_for = move |lit: usize, row: usize| -> Vec<Srgb> {
        let mut out = Vec::with_capacity(3 * side);
        for col in 0..side {
            let i = row * side + col;
            let c = if i < lit { guess_color(ests[i]) } else { unlit };
            out.extend([c, c, gap]);
        }
        out
    };
    let (left, top) = (GRID_C[0] - GRID_SIDE / 2.0, GRID_C[1] + GRID_SIDE / 2.0);
    let mut strips = Vec::with_capacity(side);
    for row in 0..side {
        let s = RowStrips::build(
            stage,
            v(left, top - pitch * row as f64),
            (pitch / 3.0, dot),
            &[colors_for(0, row)],
        )?;
        strips.push(s);
    }
    let grid = stage.arena_mut().add(VMobject::new());
    for s in &strips {
        stage.attach(grid, s.group)?;
    }

    // the right-hand panel: what we built, the run counter, the legend
    let x0 = 0.9;
    let lake_words = kit.text_left(
        stage,
        &format!("a lake of exactly {} fish", thousands(LAKE as u64)),
        30.0,
        Palette::teal(),
        v(x0, 2.6),
    )?;
    let design = kit.text_left(
        stage,
        &format!(
            "each experiment: mark {}, catch {}",
            thousands(MARKED as u64),
            thousands(CAUGHT as u64)
        ),
        26.0,
        Palette::ink(),
        v(x0, 2.0),
    )?;
    let counter_label = kit.text_left(stage, "experiment", 28.0, Palette::dim(), v(x0, 1.0))?;
    let k_label = kit.text_left(stage, "caught twice", 28.0, Palette::dim(), v(x0, 0.35))?;
    let g_label = kit.text_left(stage, "its guess", 28.0, Palette::dim(), v(x0, -0.3))?;
    let legend = kit.text_left(
        stage,
        "each dot = one whole experiment",
        24.0,
        Palette::dim(),
        v(x0, -1.5),
    )?;
    let key_y = -2.3;
    let key_w = 4.6;
    let key: Vec<VMobject> = (0..60)
        .map(|i| {
            let u = f64::from(i) / 59.0;
            let g = LAKE as f64 * (0.8 + 0.45 * u);
            rect(
                v(x0 + key_w * u, key_y - 0.09),
                key_w / 59.0 + 0.004,
                0.18,
                flat(guess_color(g), 1.0),
            )
        })
        .collect::<fmn::Result<_>>()?;
    let key = stage.arena_mut().add(v_group(key));
    let truth_at = x0 + key_w * (0.2 / 0.45);
    let lower = kit.text_left(stage, "lower guess", 22.0, Palette::blue(), v(x0, key_y - 0.38))?;
    let higher = kit.text(
        stage,
        "higher guess",
        22.0,
        hex("#E4F7EF"),
        v(x0 + key_w - 0.65, key_y - 0.38),
    )?;
    let truth_tick = stage.arena_mut().add(
        Line::new(v(truth_at, key_y - 0.16), v(truth_at, key_y + 0.16))
            .style(line_style(Palette::ink(), 3.0))
            .build()?,
    );
    let truth_word = kit.text(
        stage,
        &thousands(LAKE as u64),
        22.0,
        Palette::teal(),
        v(truth_at, key_y + 0.36),
    )?;

    play!(stage;
        fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), lake_words, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), design, ORIGIN, 1.0)?.rt(0.8),
    );
    play!(stage;
        fade_in(stage.arena_mut(), counter_label, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), k_label, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), g_label, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), legend, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), higher, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), truth_tick, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), truth_word, ORIGIN, 1.0)?.rt(0.5),
    );

    // the listen window opens when the line ends; the lighting runs in it
    let wav = sonification(&run.k).map_err(|e| {
        fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
            format!("09_1 sonification: {e}").into_boxed_str(),
        )))
    })?;
    kit.sonify(stage, WINDOW_S, "09_1", &wav)?;
    // the live readouts follow the lighting
    let (t_run, t_k, t_guess) = (
        stage.add_value_tracker(1.0),
        stage.add_value_tracker(f64::from(run.k[0])),
        stage.add_value_tracker(run.est[0].round()),
    );
    let readout_x = x0 + 3.6;
    let run_no = kit.live_count(stage, t_run, 34.0, Palette::ink(), v(readout_x, 1.0))?;
    let k_no = kit.live_count(stage, t_k, 34.0, Palette::hero(), v(readout_x, 0.35))?;
    let g_no = kit.live_count(stage, t_guess, 34.0, Palette::teal(), v(readout_x, -0.3))?;
    // a ring finds each of the first twelve as it lights
    let cell_at = move |i: usize| -> Vec3 {
        let (row, col) = (i / side, i % side);
        v(
            left + pitch * (col as f64 + 1.0 / 3.0),
            top - pitch * (row as f64 + 1.0 / 3.0),
        )
    };
    let finder = stage.arena_mut().add(
        Circle::new()
            .radius(0.16)
            .arc_center(cell_at(0))
            .style(line_style(Palette::hero(), 3.0))
            .build(),
    );
    stage.add_to_scene(finder)?;
    let strips = Rc::new(strips);
    let state = Rc::new(RefCell::new((0.0f64, 0usize)));
    let (ks, ests) = (run.k.clone(), run.est.clone());
    let (s2, st) = (Rc::clone(&strips), Rc::clone(&state));
    stage.add_dt_updater(
        grid,
        move |s, _, dt| {
            let mut state = st.borrow_mut();
            state.0 += dt;
            let lit = lit_at(state.0, runs);
            if lit == state.1 {
                return;
            }
            let (from, to) = (state.1 / side, (lit - 1) / side);
            for row in from..=to.min(side - 1) {
                s2[row].recolor(s, &[colors_for(lit, row)]);
            }
            state.1 = lit;
            let last = lit - 1;
            if lit <= ONE_BY_ONE {
                s.move_to(finder, cell_at(last), ORIGIN);
            } else {
                s.set_opacity(finder, 0.0, true);
            }
            let _ = s.set_tracker_value(t_run, lit as f64);
            let _ = s.set_tracker_value(t_k, f64::from(ks[last]));
            let _ = s.set_tracker_value(t_guess, ests[last].round());
        },
        false,
    )?;
    stage.wait(WINDOW_S)?;
    stage.clear_updaters(grid, false);
    for m in [run_no, k_no, g_no] {
        stage.clear_updaters(m, true);
    }
    kit.hold(stage)?;
    // clear the right-hand panel; the grid shrinks into a corner
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in [
        lake_words,
        design,
        counter_label,
        k_label,
        g_label,
        legend,
        key,
        lower,
        higher,
        truth_tick,
        truth_word,
        run_no,
        k_no,
        g_no,
        finder,
    ] {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    // the corner keeps a small copy (strokes do not scale with a group,
    // so the copy is drawn at its own size, cells touching)
    let inset_side = 1.7;
    let cell = inset_side / side as f64;
    let colors: Vec<Vec<Srgb>> = (0..side)
        .map(|row| (0..side).map(|col| guess_color(run.est[row * side + col])).collect())
        .collect();
    let inset = RowStrips::build(
        stage,
        v(-5.75 - inset_side / 2.0, 2.75 + inset_side / 2.0),
        (cell, cell),
        &colors,
    )?;
    let a = fade_out(stage.arena_mut(), grid, ORIGIN, 1.0)?;
    out.push(stage.prepare(a)?);
    let a = fade_in(stage.arena_mut(), inset.group, ORIGIN, 1.0)?;
    out.push(stage.prepare(a)?);
    stage.play_prepared(out)?;
    let caption = kit.text(
        stage,
        &format!("{} experiments", thousands(runs as u64)),
        22.0,
        Palette::dim(),
        v(-5.75, 1.7),
    )?;
    play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.5));
    Ok(())
}

/// The caught-twice histogram: one bar per count seen, in k order.
struct Histogram {
    group: Mob,
    bars: Vec<Mob>,
    /// (k, experiments) per bar.
    counts: Vec<(u32, u32)>,
    /// Scene units per experiment.
    per_run: f64,
    axis: Mob,
}

/// 09_2–09_3: empty axes, the question, then the bell.
fn pile_up(kit: &Kit, stage: &mut Stage<'_>, run: &panel::Panel, stats: &Stats) -> fmn::Result<Histogram> {
    let f = plot();
    let counts: Vec<(u32, u32)> = (stats.k_min..=stats.k_max)
        .map(|v| (v, run.k.iter().filter(|&&k| k == v).count() as u32))
        .filter(|&(_, c)| c > 0)
        .collect();
    let most = counts.iter().map(|&(_, c)| c).max().unwrap_or(1);
    let band_h = 2.6;
    let per_run = band_h / f64::from(most);
    let unit = f.width / (f.x.1 - f.x.0);
    let mut parts = vec![
        Line::new(v(f.left(), f.bottom()), v(f.right(), f.bottom()))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    let ticks: Vec<u32> = (0..=8).map(|i| 60 + 10 * i).collect();
    for &t in &ticks {
        let p = f.p(f64::from(t), f.y.0);
        parts.push(
            Line::new(v(p[0], p[1] - 0.08), v(p[0], p[1]))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
    }
    let axis_lines = stage.arena_mut().add(v_group(parts));
    let axis = stage.arena_mut().add(VMobject::new());
    stage.attach(axis, axis_lines)?;
    for &t in &ticks {
        let p = f.p(f64::from(t), f.y.0);
        let m = kit.text(stage, &t.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.27))?;
        stage.attach(axis, m)?;
    }
    let title = kit.text(
        stage,
        "fish caught twice",
        26.0,
        Palette::hero(),
        v(f.center[0], f.bottom() - 0.6),
    )?;
    stage.attach(axis, title)?;

    kit.say(stage, "09_2")?;
    play!(stage; fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.8));
    let q = predict_pause(kit, stage, v(4.6, 1.6))?;

    // 09_3: the bell, in the probability-over-k role
    let chance = hex("#8A93A3");
    let bars_v: Vec<VMobject> = counts
        .iter()
        .map(|&(k, c)| {
            let p = f.p(f64::from(k), f.y.0);
            rect(
                v(p[0] - 0.42 * unit, p[1]),
                0.84 * unit,
                f64::from(c) * per_run,
                flat(chance, 0.9),
            )
        })
        .collect::<fmn::Result<_>>()?;
    let group = stage.arena_mut().add(v_group(bars_v));
    let bars = explainer_kit::children(stage, group);
    kit.say(stage, "09_3")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    stage.add_to_scene(group)?;
    let grow = grow_histogram(stage, group, 0.02)?;
    play!(stage; grow.rt(1.6));
    // within nine of a hundred: the band, its share, the spread
    let (lo, hi) = (f.p(90.5, f.y.0), f.p(109.5, f.y.0));
    let band = stage.arena_mut().add(rect(
        v(lo[0], f.bottom()),
        hi[0] - lo[0],
        band_h + 0.35,
        flat(Palette::ink(), 0.07),
    )?);
    let share = (stats.k_within_9_share * 100.0).round() as u32;
    let within = kit.text(
        stage,
        &format!("91 to 109: {share}% of the experiments"),
        26.0,
        Palette::ink(),
        v((lo[0] + hi[0]) / 2.0, f.bottom() + band_h + 0.6),
    )?;
    let exact_sd = exact_sd_of_k();
    let spread = kit.text_left(
        stage,
        &format!("SD {exact_sd:.1}   (exact {exact_sd:.2}; these runs {:.2})", stats.k_sd),
        24.0,
        Palette::dim(),
        v(f.center[0] + 1.6, f.bottom() + band_h + 0.05),
    )?;
    play!(stage; fade_in(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(0.6));
    stage.scene_mut().bring_to_back(&[band])?;
    play!(stage;
        fade_in(stage.arena_mut(), within, v(0.0, -0.1), 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), spread, ORIGIN, 1.0)?.rt(0.6),
    );
    kit.hold(stage)?;
    play!(stage;
        fade_out(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(0.5),
        fade_out(stage.arena_mut(), within, ORIGIN, 1.0)?.rt(0.5),
        fade_out(stage.arena_mut(), spread, ORIGIN, 1.0)?.rt(0.5),
    );
    Ok(Histogram {
        group,
        bars,
        counts,
        per_run,
        axis,
    })
}

/// The exact SD of k at the true lake: √(n·M(N − M)(N − n) / (N²(N − 1))).
fn exact_sd_of_k() -> f64 {
    let (n, m, big) = (CAUGHT as f64, MARKED as f64, LAKE as f64);
    (n * m * (big - m) * (big - n) / (big * big * (big - 1.0))).sqrt()
}

/// 09_4–09_5: the hyperbola carries each bar to its guess.
fn bend(kit: &Kit, stage: &mut Stage<'_>, stats: &Stats, hist: Histogram) -> fmn::Result<()> {
    let f = plot();
    let unit = f.width / (f.x.1 - f.x.0);
    let scale_y = f.height / (f.y.1 - f.y.0);
    // the guess axis, its ticks, the curve
    let mut parts = vec![
        Line::new(v(f.left(), f.bottom()), v(f.left(), f.top()))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    let gticks: Vec<u32> = (0..=5).map(|i| 6_000 + 2_000 * i).collect();
    for &t in &gticks {
        let p = f.p(f.x.0, f64::from(t));
        parts.push(
            Line::new(v(p[0] - 0.08, p[1]), v(p[0], p[1]))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
    }
    let gaxis_lines = stage.arena_mut().add(v_group(parts));
    let gaxis = stage.arena_mut().add(VMobject::new());
    stage.attach(gaxis, gaxis_lines)?;
    let mut tick_labels = Vec::new();
    for &t in gticks.iter().skip(1) {
        let p = f.p(f.x.0, f64::from(t));
        let m = kit.text(stage, &thousands(t), 22.0, Palette::dim(), v(p[0] - 0.15, p[1]))?;
        stage.move_to(m, v(p[0] - 0.15, p[1]), RIGHT);
        stage.attach(gaxis, m)?;
        tick_labels.push((t, m));
    }
    let gtitle = kit.text_left(
        stage,
        &format!(
            "the guess = {} × {} ÷ caught twice",
            thousands(MARKED as u64),
            thousands(CAUGHT as u64)
        ),
        26.0,
        Palette::teal(),
        v(f.left() - 0.3, f.top() + 0.35),
    )?;
    stage.attach(gaxis, gtitle)?;
    let pts: Vec<Vec3> = (0..=200)
        .map(|i| {
            let k = 62.5 + (f.x.1 - 62.5) * f64::from(i) / 200.0;
            f.p(k, guess(k))
        })
        .collect();
    let curve = stage
        .arena_mut()
        .add(Polygon::polyline(pts).style(line_style(Palette::teal(), 4.0)).build());
    kit.say(stage, "09_4")?;
    // the corner grid and its caption leave (the guess axis needs the room)
    let mut fades: Vec<Box<dyn Animation>> = Vec::new();
    for m in stage.scene().mobjects().to_vec() {
        if m != hist.group && m != hist.axis {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            fades.push(stage.prepare(a)?);
        }
    }
    stage.play_prepared(fades)?;
    play!(stage;
        fade_in(stage.arena_mut(), gaxis, ORIGIN, 1.0)?.rt(0.8),
        show_creation(curve).rt(1.2),
    );
    // each bar rises to its point on the curve, then slides to the guess
    // axis as a horizontal bar of the same area (so the bars' areas still
    // count experiments): thinner where guesses crowd, thicker where the
    // curve spreads them
    let longest = hist
        .counts
        .iter()
        .map(|&(k, c)| f64::from(c) / spread_of(f64::from(k)))
        .fold(0.0, f64::max);
    // the longest guess bar is 3.2 units: length = stretch · area / thickness
    let stretch = 3.2 * scale_y / (unit * hist.per_run * longest);
    let chance = hex("#8A93A3");
    let n = hist.counts.len().max(2);
    let trips: Vec<Trip> = hist
        .bars
        .iter()
        .zip(&hist.counts)
        .enumerate()
        .map(|(i, (&bar, &(k, c)))| {
            let kf = f64::from(k);
            let g = guess(kf);
            let at = f.p(kf, g);
            let area = unit * f64::from(c) * hist.per_run;
            let side = area.sqrt().clamp(0.03, 0.4);
            let thick = (spread_of(kf) * scale_y).max(0.012);
            let length = stretch * area / thick;
            let start = (stage.get_center(bar), stage.get_width(bar), stage.get_height(bar));
            Trip {
                keys: [
                    start,
                    (at, side, side),
                    (v(f.left() + length / 2.0, at[1]), length, thick),
                ],
                colors: (chance, guess_color(g)),
                begin: 0.55 * i as f64 / (n - 1) as f64,
            }
        })
        .collect();
    let finals: Vec<VMobject> = trips.iter().map(|t| t.at(1.0)).collect::<fmn::Result<_>>()?;
    let clock = stage.add_value_tracker(0.0);
    let moving = Rc::new(trips);
    let shared = Rc::clone(&moving);
    let travel = stage.always_redraw(move |s| {
        let u = s.tracker_value(clock).unwrap_or(0.0);
        let parts: Vec<VMobject> = shared
            .iter()
            .filter_map(|t| t.at(((u - t.begin) / 0.45).clamp(0.0, 1.0)).ok())
            .collect();
        s.add(v_group(parts))
    });
    stage.remove_from_scene(hist.group);
    stage.add_to_scene(travel)?;
    stage.play(tween(clock, 1.0, 4.5)?)?;
    stage.clear_updaters(travel, true);
    stage.remove_from_scene(travel);
    let guesses = stage.arena_mut().add(v_group(finals));
    stage.add_to_scene(guesses)?;
    play!(stage; fade_out(stage.arena_mut(), hist.axis, ORIGIN, 1.0)?.rt(0.6));

    // ninety and a hundred and ten: ten too few against ten too many
    let truth = LAKE as f64;
    let ref_y = f.p(f.x.0, truth)[1];
    let reference = stage.arena_mut().add(
        DashedLine::new(v(f.left(), ref_y), v(f.right(), ref_y))
            .dash_length(0.12)
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let ref_label = kit.text(
        stage,
        &format!("true size {}", thousands(LAKE as u64)),
        22.0,
        Palette::ink(),
        v(0.0, ref_y + 0.22),
    )?;
    stage.move_to(ref_label, v(f.right(), ref_y + 0.22), RIGHT);
    play!(stage;
        fade_in(stage.arena_mut(), reference, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), ref_label, ORIGIN, 1.0)?.rt(0.6),
    );
    let near = |c: u64| ((MARKED * CAUGHT) as u64 + c / 2) / c;
    let mut guides = Vec::new();
    for (k, color) in [(90u64, Palette::red()), (110, Palette::blue())] {
        let g = near(k);
        let foot = f.p(k as f64, f.y.0);
        let hit = f.p(k as f64, guess(k as f64));
        let up = stage.arena_mut().add(
            DashedLine::new(foot, hit)
                .dash_length(0.1)
                .style(line_style(color, 2.5))
                .build()?,
        );
        let across = stage.arena_mut().add(
            DashedLine::new(hit, v(f.left(), hit[1]))
                .dash_length(0.1)
                .style(line_style(color, 2.5))
                .build()?,
        );
        let k_tag = kit.text(stage, &k.to_string(), 24.0, color, v(foot[0] - 0.3, foot[1] + 0.2))?;
        // the guess reads off the guess axis, between its ticks
        let g_tag = kit.text(stage, &thousands(g), 24.0, color, v(0.0, hit[1]))?;
        stage.move_to(g_tag, v(f.left() - 0.15, hit[1]), RIGHT);
        play!(stage;
            show_creation(up).rt(0.6),
            fade_in(stage.arena_mut(), k_tag, ORIGIN, 1.0)?.rt(0.4),
        );
        play!(stage;
            show_creation(across).rt(0.6),
            fade_in(stage.arena_mut(), g_tag, ORIGIN, 1.0)?.rt(0.4),
        );
        guides.extend([up, across, k_tag, g_tag]);
    }
    // the two pushes, side by side on the guess axis
    let (above, below) = (near(90) - LAKE as u64, LAKE as u64 - near(110));
    let x_arrow = f.right() - 3.2;
    let (y90, y110) = (f.p(f.x.0, near(90) as f64)[1], f.p(f.x.0, near(110) as f64)[1]);
    let push_up = stage.arena_mut().add(
        Arrow::new(v(x_arrow, ref_y), v(x_arrow, y90))
            .buff(0.0)
            .thickness(4.0)
            .style(flat(Palette::red(), 1.0))
            .build()?,
    );
    let push_down = stage.arena_mut().add(
        Arrow::new(v(x_arrow + 0.35, ref_y), v(x_arrow + 0.35, y110))
            .buff(0.0)
            .thickness(4.0)
            .style(flat(Palette::blue(), 1.0))
            .build()?,
    );
    let up_tag = kit.text(
        stage,
        &format!("+{}", thousands(above)),
        26.0,
        Palette::red(),
        v(0.0, (ref_y + y90) / 2.0),
    )?;
    stage.move_to(up_tag, v(x_arrow - 0.15, (ref_y + y90) / 2.0), RIGHT);
    let down_tag = kit.text_left(
        stage,
        &format!("\u{2212}{}", thousands(below)),
        26.0,
        Palette::blue(),
        v(x_arrow + 0.55, (ref_y + y110) / 2.0),
    )?;
    // the two levels, carried across to the arrows
    let level90 = stage.arena_mut().add(
        DashedLine::new(f.p(90.0, guess(90.0)), v(x_arrow + 0.5, y90))
            .dash_length(0.1)
            .style(line_style(Palette::red(), 2.0))
            .build()?,
    );
    let level110 = stage.arena_mut().add(
        DashedLine::new(f.p(110.0, guess(110.0)), v(x_arrow + 0.6, y110))
            .dash_length(0.1)
            .style(line_style(Palette::blue(), 2.0))
            .build()?,
    );
    play!(stage;
        show_creation(level90).rt(0.5),
        show_creation(level110).rt(0.5),
    );
    guides.extend([level90, level110]);
    play!(stage;
        grow_arrow(stage.arena_mut(), push_up)?.rt(0.7),
        grow_arrow(stage.arena_mut(), push_down)?.rt(0.7),
        fade_in(stage.arena_mut(), up_tag, ORIGIN, 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), down_tag, ORIGIN, 1.0)?.rt(0.7),
    );
    kit.hold(stage)?;

    // 09_5: the middle 95% of the guesses, and their average
    let mut clear: Vec<Box<dyn Animation>> = Vec::new();
    for m in guides.into_iter().chain([push_up, push_down, up_tag, down_tag]) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        clear.push(stage.prepare(a)?);
    }
    // the true size's label steps below its line, out of the average's way
    let down = ref_label
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.6),
            ..AnimateArgs::default()
        })?
        .move_to(v(f.right(), ref_y - 0.24), RIGHT)?;
    clear.push(stage.prepare(down)?);
    kit.say(stage, "09_5")?;
    stage.play_prepared(clear)?;
    let (y_lo, y_hi) = (f.p(f.x.0, stats.p2_5)[1], f.p(f.x.0, stats.p97_5)[1]);
    let band = stage.arena_mut().add(rect(
        v(f.left() + 0.02, y_lo),
        3.7,
        y_hi - y_lo,
        flat(Palette::teal(), 0.12),
    )?);
    let lo_tag = kit.text(
        stage,
        &thousands(stats.p2_5.round() as u64),
        24.0,
        Palette::teal(),
        v(0.0, y_lo),
    )?;
    stage.move_to(lo_tag, v(f.left() - 0.15, y_lo), RIGHT);
    let hi_tag = kit.text(
        stage,
        &thousands(stats.p97_5.round() as u64),
        24.0,
        Palette::teal(),
        v(0.0, y_hi),
    )?;
    stage.move_to(hi_tag, v(f.left() - 0.15, y_hi), RIGHT);
    let middle = kit.text_left(
        stage,
        "95% of the guesses",
        26.0,
        Palette::teal(),
        v(f.left() + 4.6, (y_lo + y_hi) / 2.0 + 0.6),
    )?;
    // the guess axis' own tick labels near the two ends make way for them
    let mut reveal: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, m) in &tick_labels {
        let y = f.p(f.x.0, f64::from(t))[1];
        if (y - y_lo).abs() < 0.3 || (y - y_hi).abs() < 0.3 {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.5),
                    ..AnimateArgs::default()
                })?
                .set_opacity(0.0)?;
            reveal.push(stage.prepare(a)?);
        }
    }
    for m in [band, lo_tag, hi_tag, middle] {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        reveal.push(stage.prepare(a)?);
    }
    stage.play_prepared(reveal)?;
    stage.scene_mut().bring_to_back(&[band])?;
    let mean_y = f.p(f.x.0, stats.mean)[1];
    let mean_line = stage.arena_mut().add(
        Line::new(v(f.left(), mean_y), v(f.right(), mean_y))
            .style(line_style(Palette::hero(), 3.0))
            .build()?,
    );
    let rounded = (stats.mean / 10.0).round() as u64 * 10;
    let mean_tag = kit.text_left(
        stage,
        &format!("average guess ≈ {}: a little high", thousands(rounded)),
        24.0,
        Palette::hero(),
        v(f.right() - 3.9, mean_y + 0.24),
    )?;
    play!(stage;
        show_creation(mean_line).rt(0.8),
        fade_in(stage.arena_mut(), mean_tag, ORIGIN, 1.0)?.rt(0.8),
    );
    kit.hold(stage)?;
    Ok(())
}

/// One bar's trip from the caught-twice axis, up to the curve, across to
/// the guess axis: three keyframes (centre, width, height), its colours at
/// the start and the end, and when it sets off (as a share of the trip).
struct Trip {
    keys: [(Vec3, f64, f64); 3],
    colors: (Srgb, Srgb),
    begin: f64,
}

impl Trip {
    /// The bar at progress `tau` (0 … 1): up to the curve in the first
    /// 45%, across in the rest, each leg eased.
    fn at(&self, tau: f64) -> fmn::Result<VMobject> {
        let ease = |t: f64| t * t * (3.0 - 2.0 * t);
        let (a, b, u) = if tau < 0.45 {
            (self.keys[0], self.keys[1], ease(tau / 0.45))
        } else {
            (self.keys[1], self.keys[2], ease((tau - 0.45) / 0.55))
        };
        let mix = |x: f64, y: f64| x + (y - x) * u;
        let c = v(mix(a.0[0], b.0[0]), mix(a.0[1], b.0[1]));
        let (w, h) = (mix(a.1, b.1), mix(a.2, b.2));
        let color = interpolate_color(self.colors.0, self.colors.1, ease((tau / 0.45).min(1.0)));
        rect(v(c[0] - w / 2.0, c[1] - h / 2.0), w, h, flat(color, 0.92))
    }
}

/// How far apart neighbouring guesses lie at k: Mn/(k − ½) − Mn/(k + ½).
fn spread_of(k: f64) -> f64 {
    guess(k - 0.5) - guess(k + 0.5)
}

/// 09_6: the panel itself, for programmers.
fn code(kit: &Kit, stage: &mut Stage<'_>, stats: &Stats) -> fmn::Result<()> {
    fade_all(stage, 0.8)?;
    let lines = code_lines();
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let panel = CodePanel::build(kit, stage, &refs, 26.0, 13.4)?;
    for &m in &panel.lines[..5] {
        stage.set_color(m, Palette::dim(), None, true);
    }
    kit.say(stage, "09_6")?;
    stage.add_to_scene(panel.group)?;
    play!(stage; panel.slide_in(1.0)?);
    // the two lines that run the experiments
    let box_around = |stage: &mut Stage<'_>, a: Mob, b: Mob, color: Srgb| -> fmn::Result<Mob> {
        let lo = stage.get_corner(b, DL);
        let hi_a = stage.get_corner(a, UR);
        let hi_b = stage.get_corner(b, UR);
        let hi = v(hi_a[0].max(hi_b[0]), hi_a[1]);
        let r = SurroundingRectangle::from_extent(Some((lo, hi)))
            .buff(0.05)
            .color(color)
            .build();
        Ok(stage.arena_mut().add(r))
    };
    let run_box = box_around(stage, panel.lines[5], panel.lines[6], Palette::hero())?;
    play!(stage; show_creation(run_box).rt(0.8));
    stage.wait(1.4)?;
    let check_box = box_around(stage, panel.lines[7], panel.lines[8], Palette::teal())?;
    play!(stage;
        fade_out(stage.arena_mut(), run_box, ORIGIN, 1.0)?.rt(0.5),
        show_creation(check_box).rt(0.8),
    );
    // the printed line, under the panel
    let last = panel.lines[8];
    let left = stage.get_left(last)[0];
    let y = stage.get_bottom(panel.group)[1] - 0.45;
    let out = kit.mono(
        stage,
        &format!(
            "{} {}",
            crate::facts::py(stats.mean),
            crate::facts::py(stats.exact_mean)
        ),
        26.0,
        Palette::green(),
        v(left, y),
    )?;
    stage.move_to(out, v(left, y), LEFT);
    play!(stage; fade_in(stage.arena_mut(), out, v(0.0, 0.15), 1.0)?.rt(0.6));
    let w = stage.get_width(out);
    let sim = kit.text(
        stage,
        &format!("the {} runs", thousands(crate::runs::PANEL_RUNS as u64)),
        22.0,
        Palette::dim(),
        v(left + w * 0.25, y - 0.38),
    )?;
    let exact = kit.text(stage, "exact", 22.0, Palette::dim(), v(left + w * 0.75, y - 0.38))?;
    let gap = kit.text(
        stage,
        &format!(
            "apart by {:.2} of the simulation's own wobble (standard error {:.1})",
            stats.gap_in_standard_errors, stats.standard_error
        ),
        24.0,
        Palette::ink(),
        v(0.0, y - 0.95),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), sim, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), exact, ORIGIN, 1.0)?.rt(0.5),
    );
    play!(stage; fade_in(stage.arena_mut(), gap, ORIGIN, 1.0)?.rt(0.6));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_chapter_09_says_holds_on_its_own_runs() {
        let run = panel::run();
        let s = run.stats().unwrap();
        // 09_1: ten thousand experiments, a square grid of them
        assert_eq!(run.k.len(), 10_000);
        // 09_3 "in a bell around a hundred": the law's peak is at 100
        // (⌊(n + 1)(M + 1)/(N + 2)⌋), and these runs' mean is within one of it
        assert_eq!((CAUGHT + 1) * (MARKED + 1) / (LAKE + 2), 100);
        assert!((s.k_mean - 100.0).abs() < 1.0, "{}", s.k_mean);
        // "most land within about nine of it"
        assert!(s.k_within_9_share > 0.5, "{}", s.k_within_9_share);
        assert_eq!(format!("{:.1}", exact_sd_of_k()), "9.0");
        assert_eq!(format!("{:.2}", s.k_sd), "8.96");
        // 09_4: about 11,100 and about 9,100; ten too few pushes harder
        let near = |c: u64| (1_000_000 + c / 2) / c;
        assert_eq!((near(90) + 50) / 100 * 100, 11_100);
        assert_eq!((near(110) + 50) / 100 * 100, 9_100);
        assert!(near(90) - 10_000 > 10_000 - near(110));
        // 09_5: about eighty-five hundred, just over twelve thousand, about
        // ten thousand and ninety, a little high
        assert_eq!((s.p2_5 / 100.0).round() * 100.0, 8_500.0);
        assert!(s.p97_5 > 12_000.0 && s.p97_5 < 12_100.0);
        assert_eq!((s.mean / 10.0).round() * 10.0, 10_090.0);
        assert!(s.mean > 10_000.0);
        // 09_6 "they agree, within the simulation's own wobble"
        assert!(s.gap_in_standard_errors.abs() < 2.0);
    }

    #[test]
    fn the_lighting_reaches_every_run_once_in_order() {
        let runs = 10_000;
        assert_eq!(lit_at(0.0, runs), 1);
        assert_eq!(lit_at(STEP_S * 11.5, runs), 12);
        let mut last = 0;
        for i in 0..2_000 {
            let n = lit_at(i as f64 / 100.0, runs);
            assert!(n >= last && n <= runs);
            last = n;
        }
        assert_eq!(lit_at(ONE_BY_ONE as f64 * STEP_S + WASH_S, runs), runs);
        assert!(
            ONE_BY_ONE as f64 * STEP_S + WASH_S < WINDOW_S,
            "the wash ends inside the window"
        );
        // each run's plink or grain sounds as its cell lights
        for i in [0, 5, 11, 12, 100, 5_000, 9_999] {
            let t = lit_time(i, runs);
            assert!(lit_at(t + 1e-6, runs) > i, "run {i} at {t}");
            assert!(i == 0 || lit_at((t - 0.02).max(0.0), runs) <= i + 30, "run {i} early");
        }
    }

    #[test]
    fn the_code_on_screen_is_the_oracles_panel() {
        // the oracle's ch09_panel comment block, line for line
        let want = [
            "import numpy as np",
            "from math import comb",
            "rng = np.random.default_rng(20261009)",
            "M = n = 1_000; N = 10_000",
            "marked = np.arange(N) < M",
            "k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])",
            "est = M * n / k",
            "p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]",
            "print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))",
        ];
        assert_eq!(code_lines(), want);
        let oracle = include_str!("../../oracle.py");
        for line in &want[2..] {
            assert!(
                oracle.contains(&format!("#     {line}")) || oracle.contains(line),
                "{line}"
            );
        }
    }
}
