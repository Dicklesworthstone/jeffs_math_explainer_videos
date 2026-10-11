//! Chapter 12, the ring toss: a range that keeps its promise. The careful
//! range built by asking every lake size whether our catch lands in its
//! middle 95% (12_1: a slice of each lake's catches swept across the sizes,
//! the band it traces, and the slices at the two ends); chapter 09's own
//! experiments tossed as rings at the true size (12_2: 98 of the first 100
//! ring it; of all 10,000, 464 miss against 455 expected); Feller's range
//! beside chapter 09's (12_3); and what tightens it (12_4–12_5). Every
//! range is `law::careful_range`, every ring a run of `runs::panel`.

use std::cell::Cell;
use std::rc::Rc;

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, hex, play, sound_file, thousands, v};
use explainer_score::instr::{Instrument, Note, render};
use explainer_score::{FS, Stereo};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{flat, line_style};
use crate::facts::FELLER;
use crate::law::{ALPHA, FELLER_BET, Law, careful_range, range_tails};
use crate::runs;

/// Probability over k: the slices and the estimate's bars (storyboard §3).
fn chance() -> Srgb {
    Palette::dim()
}

/// A card behind a big sentence.
fn card(w: f64, h: f64, at: Vec3) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at))
}

/// A whole percent with its sign, typographic minus: "−16%", "+21%".
fn signed_pct(x: f64) -> String {
    let r = x.round();
    if r < 0.0 {
        format!("\u{2212}{}%", -r)
    } else {
        format!("+{r}%")
    }
}

/// A horizontal axis along the bottom of `frame`: the line, a tick and a
/// label at each of `ticks` (data x), formatted by `fmt`.
fn x_axis(
    kit: &Kit,
    stage: &mut Stage<'_>,
    frame: &Frame2,
    y: f64,
    ticks: &[f64],
    fmt: impl Fn(f64) -> String,
) -> fmn::Result<Mob> {
    let mut parts = vec![
        Line::new(v(frame.left(), y), v(frame.right(), y))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    for &t in ticks {
        let x = frame.p(t, 0.0)[0];
        parts.push(
            Line::new(v(x, y - 0.08), v(x, y + 0.08))
                .style(line_style(Palette::dim(), 2.5))
                .build()?,
        );
    }
    let g = stage.arena_mut().add(v_group(parts));
    for &t in ticks {
        let x = frame.p(t, 0.0)[0];
        let label = kit.text(stage, &fmt(t), 22.0, Palette::dim(), v(x, y - 0.3))?;
        stage.attach(g, label)?;
    }
    Ok(g)
}

/// One lake's slice of catches, drawn sideways from `x0`: the law over k as
/// a filled outline, `scale` scene units per unit of probability, split
/// into the low tail, the middle 95% and the high tail. Clipped to the
/// window's k range.
fn slice(law: &Law, frame: &Frame2, x0: f64, scale: f64, (lo, hi): (u64, u64)) -> [VMobject; 3] {
    let (k_min, k_max) = (frame.y.0.ceil() as u64, frame.y.1.floor() as u64);
    let piece = |a: u64, b: u64, color: Srgb, opacity: f64| -> VMobject {
        let (a, b) = (a.max(k_min).max(law.first), b.min(k_max).min(law.last()));
        if a > b {
            return VMobject::new();
        }
        let y = |k: f64| frame.p(0.0, k)[1];
        let mut pts = vec![v(x0, y(a as f64 - 0.5)), v(x0 + scale * law.at(a), y(a as f64 - 0.5))];
        pts.extend((a..=b).map(|k| v(x0 + scale * law.at(k), y(k as f64))));
        pts.push(v(x0 + scale * law.at(b), y(b as f64 + 0.5)));
        pts.push(v(x0, y(b as f64 + 0.5)));
        Polygon::new(pts).style(flat(color, opacity)).build()
    };
    [
        piece(0, lo.saturating_sub(1), Palette::red(), 0.6),
        piece(lo, hi, chance(), 0.8),
        piece(hi + 1, u64::MAX, Palette::red(), 0.6),
    ]
}

/// The clacks of 12_2's listen window: a wooden marimba knock when a ring
/// lands round the peg, a low muffled pluck when one misses and hits the
/// ground; `events` are (seconds from the window's start, rang).
fn clacks(events: &[(f64, bool)], seconds: f64) -> Stereo {
    let frames = |s: f64| (s * FS).round() as usize;
    let mut out = Stereo::silence(frames(seconds + 3.0));
    for (i, &(at, rang)) in events.iter().enumerate() {
        let seed = i as u64 + 1;
        let (inst, mut note) = if rang {
            // the same knock each time, a little brighter as the stack grows
            (Instrument::Marimba, Note::new(79.0, 0.55, 0.12, seed))
        } else {
            (Instrument::Pluck, Note::new(43.0, 0.9, 0.5, seed))
        };
        note.pan = ((i % 7) as f64 - 3.0) / 3.0 * 0.25;
        out.mix_in(&render(inst, &note), frames(at), 1.0);
    }
    // cut to the window, fade the last quarter second to zero, and set the
    // level at the narration's −23 LUFS (the score re-levels it)
    let n = frames(seconds);
    out.l.resize(n, 0.0);
    out.r.resize(n, 0.0);
    let tail = frames(0.25).min(n);
    for k in 0..tail {
        let g = 0.5 - 0.5 * (std::f64::consts::PI * k as f64 / tail as f64).cos();
        out.l[n - 1 - k] *= g;
        out.r[n - 1 - k] *= g;
    }
    let lufs = explainer_score::meter::integrated(&out);
    if lufs.is_finite() {
        out.scale(10f64.powf((-23.0 - lufs) / 20.0));
    }
    out
}

/// When each of the cascade's rings lands, seconds from its start: slow
/// enough at first to hear each knock, then a rattle.
fn landings(n: usize, fall: f64) -> Vec<f64> {
    let mut t = 0.0;
    (0..n)
        .map(|i| {
            let at = t + fall;
            t += (0.22 * 0.88f64.powi(i as i32)).max(0.025);
            at
        })
        .collect()
}

/// A narration line as it is spoken: when it started and how long it is
/// (the take's length, or the scratch voice's estimate), so a beat can wait
/// for a phrase of it instead of a guessed number of seconds. A phrase is
/// placed at its share of the line's words.
pub(super) struct Spoken {
    start: f64,
    len: f64,
    words: Vec<String>,
}

/// Words as the beats compare them: letters and digits only, lower case.
fn plain(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

impl Spoken {
    /// Speak line `id` (`Kit::say`) and remember when it started.
    pub(super) fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Self> {
        kit.say(stage, id)?;
        let script = explainer_kit::Script::parse(crate::SCRIPT).map_err(model)?;
        let text = script
            .text(id)
            .ok_or_else(|| model(format!("no line {id} in script.tsv")))?;
        let words: Vec<String> = text.split_whitespace().map(plain).collect();
        let estimate = words.len() as f64 / explainer_kit::narration::SCRATCH_WPS + 0.4;
        let len = kit.narrator.as_ref().and_then(|n| n.length(id)).unwrap_or(estimate);
        Ok(Self {
            start: stage.scene().time().to_f64(),
            len,
            words,
        })
    }

    /// Seconds into the line at which `phrase` starts.
    fn offset(&self, phrase: &str) -> Result<f64, String> {
        let want: Vec<String> = phrase.split_whitespace().map(plain).collect();
        let at = (0..self.words.len())
            .find(|&i| self.words[i..].starts_with(&want))
            .ok_or_else(|| format!("{phrase:?} is not in the line"))?;
        Ok(self.len * at as f64 / self.words.len() as f64)
    }

    /// Wait until `phrase` is spoken (no wait if that is already past).
    pub(super) fn at(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let due = self.start + self.offset(phrase).map_err(model)?;
        let now = stage.scene().time().to_f64();
        if due - now > 1e-3 {
            stage.wait(due - now)?;
        }
        Ok(())
    }

    /// Seconds from now until `phrase` is spoken (at least `floor`).
    pub(super) fn until(&self, stage: &Stage<'_>, phrase: &str, floor: f64) -> fmn::Result<f64> {
        let due = self.start + self.offset(phrase).map_err(model)?;
        Ok((due - stage.scene().time().to_f64()).max(floor))
    }
}

/// Scene error from a model error (a careful range refused by the guard).
pub(super) fn model(e: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(e.into_boxed_str())))
}

pub struct RingToss {
    pub kit: Kit,
}

impl SceneConstruct for RingToss {
    fn name(&self) -> &str {
        "12_the_ring_toss"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (fm, fnn, fk) = FELLER;
        let lake = runs::LAKE as u64;
        let (lo, hi) = careful_range(fm, fnn, fk).map_err(model)?;
        let middle_pct = (100.0 * (1.0 - 2.0 * ALPHA)).round();
        let tail_pct = 100.0 * ALPHA;

        self.careful(stage, &kit, (lo, hi), middle_pct, tail_pct)?;
        self.toss(stage, &kit, lake, middle_pct)?;
        self.compare(stage, &kit, lake, (lo, hi))?;
        self.tighten(stage, &kit, lake, (lo, hi))?;

        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl RingToss {
    /// 12_1: for each lake size, does our catch land in its middle 95%?
    fn careful(
        &self,
        stage: &mut Stage<'_>,
        kit: &Kit,
        (lo, hi): (u64, u64),
        middle_pct: f64,
        tail_pct: f64,
    ) -> fmn::Result<()> {
        let (fm, fnn, fk) = FELLER;
        let frame = Frame2 {
            center: v(0.55, -0.05),
            width: 11.2,
            height: 5.3,
            x: (6_000.0, 16_000.0),
            y: (40.0, 200.0),
        };
        let scale = 26.0;
        // each lake size's middle 95%, on a grid of sizes (the band)
        let step = 25u64;
        let grid: Vec<(u64, u64, u64)> = (frame.x.0 as u64..=frame.x.1 as u64)
            .step_by(step as usize)
            .map(|n| Law::new(n, fm, fnn).middle().map(|(a, b)| (n, a, b)))
            .collect::<Result<_, _>>()
            .map_err(model)?;

        let title = kit.text(stage, "the careful range", 36.0, Palette::teal(), v(0.0, 3.45))?;
        let axis_y = frame.bottom();
        let ticks: Vec<f64> = (0..=5).map(|i| frame.x.0 + 2_000.0 * f64::from(i)).collect();
        let x_ax = x_axis(kit, stage, &frame, axis_y, &ticks, |t| thousands(t as u64))?;
        let x_title = kit.text(
            stage,
            "lake size",
            24.0,
            Palette::teal(),
            v(frame.right() - 0.6, axis_y - 0.62),
        )?;
        let mut y_parts = vec![
            Line::new(v(frame.left(), frame.bottom()), v(frame.left(), frame.top()))
                .style(line_style(Palette::dim(), 2.5))
                .build()?,
        ];
        for k in [50.0, 100.0, 150.0, 200.0] {
            let y = frame.p(0.0, k)[1];
            y_parts.push(
                Line::new(v(frame.left() - 0.08, y), v(frame.left() + 0.08, y))
                    .style(line_style(Palette::dim(), 2.5))
                    .build()?,
            );
        }
        let y_ax = stage.arena_mut().add(v_group(y_parts));
        for k in [50.0, 100.0, 150.0, 200.0] {
            let at = v(frame.left() - 0.2, frame.p(0.0, k)[1]);
            let m = kit.text(stage, &format!("{k}"), 22.0, Palette::dim(), at)?;
            stage.move_to(m, at, RIGHT);
            stage.attach(y_ax, m)?;
        }
        let y_title = kit.text_left(
            stage,
            "caught twice",
            24.0,
            Palette::hero(),
            v(frame.left() - 0.6, frame.top() + 0.32),
        )?;
        // our catch: a gold line at k = 100
        let line_y = frame.p(0.0, fk as f64)[1];
        let ours = stage.arena_mut().add(
            Line::new(v(frame.left(), line_y), v(frame.right(), line_y))
                .style(line_style(Palette::hero(), 4.0))
                .build()?,
        );
        let ours_at = v(frame.right() - 0.05, line_y + 0.3);
        let ours_label = kit.text(
            stage,
            &format!("our catch: {fk} caught twice"),
            24.0,
            Palette::hero(),
            ours_at,
        )?;
        stage.move_to(ours_label, ours_at, RIGHT);
        let line = Spoken::say(kit, stage, "12_1")?;
        play!(stage;
            write(stage.arena(), title).rt(0.7),
            fade_in(stage.arena_mut(), x_ax, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), y_ax, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), x_title, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), y_title, ORIGIN, 1.0)?.rt(0.7),
        );
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), ours, LEFT, None)?.rt(0.7),
            fade_in(stage.arena_mut(), ours_label, ORIGIN, 1.0)?.rt(0.7),
        );

        // the sweep: a tracker walks the lake size; the slice, the band it
        // has traced and the kept sizes follow it
        let size = stage.add_value_tracker(frame.x.0);
        let band_grid = Rc::new(grid);
        let bg = Rc::clone(&band_grid);
        let band = stage.always_redraw(move |s| {
            let now = s.tracker_value(size).unwrap_or(frame.x.0);
            let upto: Vec<&(u64, u64, u64)> = bg.iter().filter(|g| g.0 as f64 <= now).collect();
            let mut pts: Vec<Vec3> = upto.iter().map(|g| frame.p(g.0 as f64, g.1 as f64 - 0.5)).collect();
            pts.extend(upto.iter().rev().map(|g| frame.p(g.0 as f64, g.2 as f64 + 0.5)));
            let shape = if pts.len() >= 4 {
                Polygon::new(pts).style(flat(chance(), 0.22)).build()
            } else {
                VMobject::new()
            };
            s.add(shape)
        });
        stage.add_to_scene(band)?;
        stage.scene_mut().bring_to_back(&[band])?;
        let moving = stage.always_redraw(move |s| {
            let n = s.tracker_value(size).unwrap_or(frame.x.0).round() as u64;
            let law = Law::new(n, fm, fnn);
            let parts = match law.middle() {
                Ok(m) => slice(&law, &frame, frame.p(n as f64, 0.0)[0], scale, m).to_vec(),
                Err(_) => Vec::new(),
            };
            s.add(v_group(parts))
        });
        stage.add_to_scene(moving)?;
        let kept = stage.always_redraw(move |s| {
            let now = s.tracker_value(size).unwrap_or(frame.x.0);
            let shape = if now >= lo as f64 {
                let a = frame.p(lo as f64, 0.0)[0];
                let b = frame.p(now.min(hi as f64), 0.0)[0];
                Rectangle::new()
                    .width((b - a).max(0.01))
                    .height(0.16)
                    .style(flat(Palette::teal(), 0.95))
                    .build()
                    .map(|r| r.moved_to(v((a + b) / 2.0, axis_y + 0.12)))
                    .unwrap_or_else(|_| VMobject::new())
            } else {
                VMobject::new()
            };
            s.add(shape)
        });
        stage.add_to_scene(kept)?;
        let dot = stage.always_redraw(move |s| {
            let n = s.tracker_value(size).unwrap_or(frame.x.0).round() as u64;
            let inside = (lo..=hi).contains(&n);
            let p = frame.p(n as f64, fk as f64);
            let c = if inside { Palette::teal() } else { Palette::red() };
            s.add(Dot::new().point(p).radius(0.08).color(c).build())
        });
        stage.add_to_scene(dot)?;
        let band_label = kit.text(
            stage,
            &format!("each lake size's middle {middle_pct}% of catches"),
            24.0,
            chance(),
            frame.p(12_900.0, 186.0),
        )?;
        line.at(stage, "For each lake size")?;
        play!(stage; fade_in(stage.arena_mut(), band_label, ORIGIN, 1.0)?.rt(0.5));
        let sweep_s = line.until(stage, "Keep the sizes", 3.5)? + 1.2;
        let sweep = size
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(sweep_s),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .set_value(frame.x.1)?;
        stage.play(sweep)?;

        // the two ends: the slices whose tail edge our catch sits on
        for m in [moving, dot] {
            stage.clear_updaters(m, true);
        }
        play!(stage;
            fade_out(stage.arena_mut(), moving, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), dot, ORIGIN, 1.0)?.rt(0.5),
        );
        let mut ends = Vec::new();
        for (n, tail_k) in [(lo, fk as f64 - 9.0), (hi, fk as f64 + 9.0)] {
            let law = Law::new(n, fm, fnn);
            let middle = law.middle().map_err(model)?;
            let x0 = frame.p(n as f64, 0.0)[0];
            let shape = stage.arena_mut().add(v_group(slice(&law, &frame, x0, scale, middle)));
            let base = stage.arena_mut().add(
                Line::new(v(x0, frame.bottom()), frame.p(n as f64, 165.0))
                    .style(line_style(Palette::teal(), 2.0))
                    .build()?,
            );
            let pct = kit.text(
                stage,
                &format!("{tail_pct}%"),
                22.0,
                Palette::ink(),
                v(x0 + 0.55, frame.p(0.0, tail_k)[1]),
            )?;
            let at = v(x0, frame.bottom() + 0.45);
            let tag = kit.text(stage, &thousands(n), 26.0, Palette::teal(), at)?;
            stage.move_to(tag, at, if n == lo { RIGHT } else { LEFT });
            ends.push((shape, base, pct, tag));
        }
        for &(shape, base, pct, tag) in &ends {
            play!(stage;
                fade_in(stage.arena_mut(), shape, ORIGIN, 1.0)?.rt(0.7),
                fmn::animation::grow_from_edge(stage.arena_mut(), base, DOWN, None)?.rt(0.7),
                fade_in(stage.arena_mut(), pct, ORIGIN, 1.0)?.rt(0.7),
                fade_in(stage.arena_mut(), tag, v(0.0, 0.15), 1.0)?.rt(0.7),
            );
        }
        kit.hold(stage)?;
        fade_all(stage, 0.8)
    }

    /// 12_2: chapter 09's experiments, each tossed as its careful range at
    /// the true size.
    fn toss(&self, stage: &mut Stage<'_>, kit: &Kit, lake: u64, middle_pct: f64) -> fmn::Result<()> {
        let (fm, fnn, _) = FELLER;
        let panel = runs::panel();
        let tossed = 100usize;
        let ranges: Vec<(u64, u64)> = panel[..tossed]
            .iter()
            .map(|&k| careful_range(fm, fnn, u64::from(k)))
            .collect::<Result<_, _>>()
            .map_err(model)?;
        let rang: Vec<bool> = ranges.iter().map(|&(a, b)| (a..=b).contains(&lake)).collect();
        let lo_all = ranges.iter().map(|r| r.0).min().unwrap_or(lake) as f64;
        let hi_all = ranges.iter().map(|r| r.1).max().unwrap_or(lake) as f64;
        let x0 = ((lo_all - 400.0) / 1_000.0).floor() * 1_000.0;
        let x1 = ((hi_all + 400.0) / 1_000.0).ceil() * 1_000.0;
        let floor = -2.0;
        let frame = Frame2 {
            center: v(0.0, 0.0),
            width: 12.6,
            height: 1.0,
            x: (x0, x1),
            y: (-0.5, 0.5),
        };
        let xs = |n: f64| frame.p(n, 0.0)[0];
        let ticks: Vec<f64> = (0..)
            .map(|i| x0 + 2_000.0 * f64::from(i))
            .take_while(|&t| t <= x1)
            .collect();
        let axis = x_axis(kit, stage, &frame, -2.75, &ticks, |t| thousands(t as u64))?;
        let ground = stage.arena_mut().add(
            Line::new(v(frame.left(), floor), v(frame.right(), floor))
                .style(line_style(Palette::faint(), 3.0))
                .build()?,
        );
        let peg_x = xs(lake as f64);
        let peg_h = 4.4;
        let peg = stage.arena_mut().add(
            Rectangle::new()
                .width(0.09)
                .height(peg_h)
                .corner_radius(0.04)
                .style(flat(Palette::teal(), 1.0))
                .build()?
                .moved_to(v(peg_x, floor + peg_h / 2.0)),
        );
        let peg_label = kit.text(
            stage,
            &format!("true size: {}", thousands(lake)),
            26.0,
            Palette::teal(),
            v(peg_x, floor + peg_h + 0.25),
        )?;

        // each ring: its careful range as an ellipse, a back arc behind the
        // peg and a front arc before it, built where it comes to rest and
        // then lifted `drop` out of sight
        let dy = 0.042;
        let ring_h = 0.08;
        let miss_y = floor - 0.22;
        let mut rest = Vec::with_capacity(tossed);
        let (mut stack, mut down) = (0usize, 0usize);
        for &r in &rang {
            if r {
                rest.push(floor + 0.1 + dy * stack as f64);
                stack += 1;
            } else {
                rest.push(miss_y - 0.2 * down as f64);
                down += 1;
            }
        }
        let arc = |a: f64, b: f64, y: f64, from: f64, color: Srgb| -> fmn::Result<VMobject> {
            let (c, rx, ry) = ((a + b) / 2.0, (b - a) / 2.0, ring_h / 2.0);
            ParametricCurve::new(move |t: f64| v(c + rx * t.cos(), y + ry * t.sin()))
                .t_range([from, from + std::f64::consts::PI, std::f64::consts::PI / 24.0])
                .style(line_style(color, 2.5))
                .build()
                .map_err(|e| model(e.to_string()))
        };
        let drop = 7.2;
        let (mut backs, mut fronts) = (Vec::new(), Vec::new());
        for (i, (&(a, b), &y)) in ranges.iter().zip(&rest).enumerate() {
            let color = if rang[i] { Palette::teal() } else { Palette::red() };
            let (xa, xb) = (xs(a as f64), xs(b as f64));
            backs.push(arc(xa, xb, y + drop, 0.0, color)?);
            fronts.push(arc(xa, xb, y + drop, std::f64::consts::PI, color)?);
        }
        let back = stage.arena_mut().add(v_group(backs));
        let front = stage.arena_mut().add(v_group(fronts));
        let back_rings = explainer_kit::children(stage, back);
        let front_rings = explainer_kit::children(stage, front);

        // experiment 1, made in front of us: its catch and its range
        let (first_lo, first_hi) = ranges[0];
        let run1 = kit.text_left(
            stage,
            &format!(
                "experiment 1: {} caught twice → range {} – {}",
                panel[0],
                thousands(first_lo),
                thousands(first_hi)
            ),
            26.0,
            Palette::ink(),
            v(-6.8, 3.45),
        )?;
        let line = Spoken::say(kit, stage, "12_2")?;
        play!(stage;
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ground, ORIGIN, 1.0)?.rt(0.6),
            fmn::animation::grow_from_edge(stage.arena_mut(), peg, DOWN, None)?.rt(0.8),
            fade_in(stage.arena_mut(), peg_label, ORIGIN, 1.0)?.rt(0.8),
        );
        // draw order: back arcs, the peg, front arcs
        stage.add_to_scene(back)?;
        stage.scene_mut().bring_to_front(&[peg, peg_label])?;
        stage.add_to_scene(front)?;
        // ring 1 hangs over the peg while its numbers are read, then drops
        let hang = 2.95;
        let lift = hang - (rest[0] + drop);
        for m in [back_rings[0], front_rings[0]] {
            stage.shift(m, v(0.0, lift));
        }
        play!(stage; fade_in(stage.arena_mut(), run1, ORIGIN, 1.0)?.rt(0.6));
        line.at(stage, "In the long run")?;
        let mut falls = Vec::new();
        for m in [back_rings[0], front_rings[0]] {
            let c = stage.get_center(m);
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    rate_func: Some(fmn::core::rate::rush_into),
                    ..AnimateArgs::default()
                })?
                .move_to(v(c[0], c[1] - (hang - rest[0])), ORIGIN)?;
            falls.push(stage.prepare(a)?);
        }
        stage.play_prepared(falls)?;

        // rings 2 to 6 drop quietly while the line is spoken; the rest fall
        // in a cascade heard in the listen window. Each falls `drop` under
        // gravity, starting `fall` before it lands; the cascade's clock starts
        // when the window opens (after the line), shared with the updater.
        let fall = 0.42;
        let quiet = 6usize;
        let land = landings(tossed - quiet, fall);
        let events: Vec<(f64, bool)> = land.iter().zip(&rang[quiet..]).map(|(&t, &r)| (t, r)).collect();
        let window = land.last().copied().unwrap_or(0.0) + 0.6;
        let wav =
            sound_file("capture_recapture_12_2_clacks", &clacks(&events, window)).map_err(|e| model(e.to_string()))?;
        let clock = Rc::new(Cell::new(0.0));
        let opened = Rc::new(Cell::new(f64::INFINITY));
        let mut moving: Vec<(Mob, Mob, f64, bool, f64)> = (1..tossed)
            .map(|i| {
                if i < quiet {
                    (back_rings[i], front_rings[i], 0.7 * i as f64 - 0.4, false, 0.0)
                } else {
                    (back_rings[i], front_rings[i], land[i - quiet] - fall, true, 0.0)
                }
            })
            .collect();
        let (tick, open) = (Rc::clone(&clock), Rc::clone(&opened));
        stage.add_dt_updater(
            back,
            move |s, _, dt| {
                if dt <= 0.0 {
                    return;
                }
                tick.set(tick.get() + dt);
                let t = tick.get();
                for (b, f, start, in_window, fallen) in &mut moving {
                    let start = if *in_window { open.get() + *start } else { *start };
                    if t < start || *fallen >= drop {
                        continue;
                    }
                    let u = ((t - start) / fall).min(1.0);
                    let now = drop * u * u;
                    let step = now - *fallen;
                    s.shift(*b, v(0.0, -step));
                    s.shift(*f, v(0.0, -step));
                    *fallen = now;
                }
            },
            false,
        )?;
        kit.sonify(stage, window, "12_2", &wav)?;
        opened.set(clock.get());
        stage.wait(window - 0.7)?;

        // the tally: the first hundred, then all ten thousand (a run's range
        // rings the true size exactly when its catch is in that lake's
        // middle 95%: law.rs proves it on the edges)
        let missed: Vec<usize> = (0..tossed).filter(|&i| !rang[i]).collect();
        let mut miss_tags = Vec::new();
        for &i in &missed {
            let (a, b) = ranges[i];
            let left_of_peg = b < lake;
            let at = if left_of_peg {
                v(xs(a as f64) - 0.12, rest[i])
            } else {
                v(xs(b as f64) + 0.12, rest[i])
            };
            let tag = kit.text(stage, &format!("run {}", i + 1), 22.0, Palette::red(), at)?;
            stage.move_to(tag, at, if left_of_peg { RIGHT } else { LEFT });
            miss_tags.push(tag);
        }
        let first100 = kit.text_left(
            stage,
            &format!(
                "first {tossed}: {} rang the true size, {} missed",
                tossed - missed.len(),
                missed.len()
            ),
            26.0,
            Palette::ink(),
            v(-6.8, 3.45),
        )?;
        let truth = Law::new(lake, fm, fnn);
        let (keep_lo, keep_hi) = truth.middle().map_err(model)?;
        let all_missed = panel
            .iter()
            .filter(|&&k| !(keep_lo..=keep_hi).contains(&u64::from(k)))
            .count();
        let coverage = truth.middle_mass().map_err(model)?;
        let expected = ((1.0 - coverage) * panel.len() as f64).round() as u64;
        let all = kit.text_left(
            stage,
            &format!(
                "all {}: {} missed ({} expected)",
                thousands(panel.len() as u64),
                all_missed,
                expected
            ),
            26.0,
            Palette::ink(),
            v(-6.8, 3.0),
        )?;
        let mut tally: Vec<Box<dyn Animation>> = vec![stage.prepare(replacement_transform(run1, first100).rt(0.7))?];
        let a = fade_in(stage.arena_mut(), all, v(0.0, 0.1), 1.0)?.rt(0.7);
        tally.push(stage.prepare(a)?);
        for &t in &miss_tags {
            let a = fade_in(stage.arena_mut(), t, ORIGIN, 1.0)?.rt(0.7);
            tally.push(stage.prepare(a)?);
        }
        stage.play_prepared(tally)?;
        stage.clear_updaters(back, true);

        // the name, on a card in the top right, above the stack
        let name = kit.text_styled(
            stage,
            &format!("{middle_pct}% confidence interval"),
            38.0,
            Palette::teal(),
            ORIGIN,
            true,
            false,
        )?;
        let (w, h) = (stage.get_width(name) + 0.6, stage.get_height(name) + 0.5);
        let centre = v(6.9 - w / 2.0, 3.2);
        stage.move_to(name, centre, ORIGIN);
        let name_card = stage.arena_mut().add(card(w, h, centre)?);
        play!(stage;
            fade_in(stage.arena_mut(), name_card, ORIGIN, 1.0)?.rt(0.5),
            write(stage.arena(), name).rt(0.8),
        );
        let note = precisely(
            kit,
            stage,
            &format!(
                "exact test inversion; coverage ≥ {middle_pct}% at every lake size; {:.2}% at this one",
                100.0 * coverage
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.7));
        stage.wait(1.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)
    }

    /// 12_3: Feller's careful range beside chapter 09's spread of estimates.
    fn compare(&self, stage: &mut Stage<'_>, kit: &Kit, lake: u64, (lo, hi): (u64, u64)) -> fmn::Result<()> {
        let (fm, fnn, fk) = FELLER;
        let frame = Frame2 {
            center: v(0.3, 0.0),
            width: 12.2,
            height: 1.0,
            x: (6_000.0, 16_000.0),
            y: (-0.5, 0.5),
        };
        let xs = |n: f64| frame.p(n, 0.0)[0];
        let ticks: Vec<f64> = (0..=5).map(|i| 6_000.0 + 2_000.0 * f64::from(i)).collect();
        let axis = x_axis(kit, stage, &frame, -2.55, &ticks, |t| thousands(t as u64))?;
        let axis_title = kit.text(stage, "lake size", 24.0, Palette::teal(), v(xs(15_200.0), -2.2))?;

        // chapter 09's question: the lake fixed at 10,000, the catch moves;
        // each bar is one catch's soup guess, as tall as its chance
        let truth = Law::new(lake, fm, fnn);
        let (q025, q975) = truth.estimate_quantiles().map_err(model)?;
        let (in_lo, in_hi) = truth.middle().map_err(model)?;
        let base = 0.75;
        let tall = 46.0;
        let mut bars = Vec::new();
        for k in truth.first.max(1)..=truth.last() {
            let est = (fm * fnn) as f64 / k as f64;
            if !(frame.x.0..=frame.x.1).contains(&est) || truth.at(k) * tall < 0.01 {
                continue;
            }
            let lit = (in_lo..=in_hi).contains(&k);
            bars.push(explainer_kit::rect(
                v(xs(est) - 0.025, base),
                0.05,
                truth.at(k) * tall,
                flat(if lit { chance() } else { Palette::faint() }, 0.95),
            )?);
        }
        let bars = stage.arena_mut().add(v_group(bars));
        let (e_lo, e_hi) = ((fm * fnn) as f64 / q025 as f64, (fm * fnn) as f64 / q975 as f64);
        let spread_bar = stage.arena_mut().add(explainer_kit::rect(
            v(xs(e_lo), base - 0.3),
            xs(e_hi) - xs(e_lo),
            0.16,
            flat(chance(), 0.95),
        )?);
        let spread_lo = kit.text(
            stage,
            &thousands(e_lo.round() as u64),
            24.0,
            chance(),
            v(xs(e_lo) - 0.55, base - 0.22),
        )?;
        let spread_hi = kit.text(
            stage,
            &thousands(e_hi.round() as u64),
            24.0,
            chance(),
            v(xs(e_hi) + 0.6, base - 0.22),
        )?;
        let there = kit.text_left(
            stage,
            &format!("chapter 09: the lake fixed at {}, the catch moved", thousands(lake)),
            26.0,
            chance(),
            v(-6.6, 3.2),
        )?;

        // ours: the catch fixed at 100, every lake size tried
        let ours_y = -1.35;
        let careful_bar = stage.arena_mut().add(explainer_kit::rect(
            v(xs(lo as f64), ours_y - 0.09),
            xs(hi as f64) - xs(lo as f64),
            0.18,
            flat(Palette::teal(), 0.95),
        )?);
        let careful_lo = kit.text(
            stage,
            &thousands(lo),
            26.0,
            Palette::teal(),
            v(xs(lo as f64) - 0.6, ours_y),
        )?;
        let careful_hi = kit.text(
            stage,
            &thousands(hi),
            26.0,
            Palette::teal(),
            v(xs(hi as f64) + 0.65, ours_y),
        )?;
        let here = kit.text_left(
            stage,
            &format!("here: the catch fixed at {fk} caught twice, every lake size tried"),
            26.0,
            Palette::teal(),
            v(-6.6, -0.75),
        )?;

        let line = Spoken::say(kit, stage, "12_3")?;
        play!(stage;
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), axis_title, ORIGIN, 1.0)?.rt(0.7),
        );
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), careful_bar, LEFT, None)?.rt(1.0),
            fade_in(stage.arena_mut(), careful_lo, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), careful_hi, ORIGIN, 1.0)?.rt(1.0),
        );
        line.at(stage, "Close to the range")?;
        let grow = explainer_kit::props::grow_histogram(stage, bars, 0.02)?;
        play!(stage; grow.rt(1.6));
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), spread_bar, LEFT, None)?.rt(0.8),
            fade_in(stage.arena_mut(), spread_lo, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), spread_hi, ORIGIN, 1.0)?.rt(0.8),
        );
        line.at(stage, "There the lake")?;
        play!(stage;
            fade_in(stage.arena_mut(), there, v(0.0, -0.1), 1.0)?.rt(0.7),
            fmn::animation::indicate(stage.arena_mut(), bars, 1.04, None)?.rt(1.2),
        );
        line.at(stage, "Here the catch")?;
        play!(stage;
            fade_in(stage.arena_mut(), here, v(0.0, 0.1), 1.0)?.rt(0.7),
            fmn::animation::indicate(stage.arena_mut(), careful_bar, 1.08, None)?.rt(1.2),
        );
        let (low, high) = range_tails(fm, fnn, fk, FELLER_BET);
        let note = precisely(
            kit,
            stage,
            &format!(
                "Feller's own {}–{} matches one-sided levels {:.1}% and {:.1}%: confidence ≈ {:.1}%",
                thousands(FELLER_BET.0),
                thousands(FELLER_BET.1),
                100.0 * low,
                100.0 * high,
                100.0 * (1.0 - low - high)
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        kit.hold(stage)?;
        fade_all(stage, 0.8)
    }

    /// 12_4–12_5: what makes the range tight. The lake fixed at 10,000;
    /// marked = second catch = √(10,000 × expected caught twice).
    fn tighten(&self, stage: &mut Stage<'_>, kit: &Kit, lake: u64, (lo, hi): (u64, u64)) -> fmn::Result<()> {
        // the axis in percent of the true size
        let frame = Frame2 {
            center: v(0.0, -0.4),
            width: 11.0,
            height: 1.0,
            x: (-40.0, 60.0),
            y: (-0.5, 0.5),
        };
        let xs = |pct: f64| frame.p(pct, 0.0)[0];
        let pct = |n: u64| 100.0 * (n as f64 - lake as f64) / lake as f64;
        let bar_y = -0.1;
        let bar = |a: u64, b: u64| -> fmn::Result<VMobject> {
            let (xa, xb) = (xs(pct(a)), xs(pct(b)));
            Ok(Rectangle::new()
                .width((xb - xa).max(0.06))
                .height(0.3)
                .corner_radius(0.05)
                .style(flat(Palette::teal(), 0.95))
                .build()?
                .moved_to(v((xa + xb) / 2.0, bar_y)))
        };

        // 12_4: the range alone, and three knobs
        let range = stage.arena_mut().add(bar(lo, hi)?);
        let knobs: Vec<(&str, Srgb)> = vec![
            ("marked", Palette::hero()),
            ("second catch", Palette::green()),
            ("caught twice", Palette::hero()),
        ];
        let mut knob_mobs = Vec::new();
        for (i, (word, color)) in knobs.iter().enumerate() {
            let x = -3.6 + 3.6 * i as f64;
            let dial = Circle::new()
                .radius(0.42)
                .arc_center(v(x, 2.1))
                .style(Style::default().fill(hex("#1A2130"), 1.0).stroke(*color, 4.0, 1.0))
                .build();
            let pointer = Line::new(v(x, 2.1), v(x + 0.3, 2.4))
                .style(line_style(*color, 4.0))
                .build()?;
            let g = stage.arena_mut().add(v_group([dial, pointer]));
            let label = kit.text(stage, word, 26.0, *color, v(x, 1.35))?;
            stage.attach(g, label)?;
            knob_mobs.push(g);
        }
        kit.say(stage, "12_4")?;
        play!(stage; fade_in(stage.arena_mut(), range, ORIGIN, 1.0)?.rt(0.6));
        for g in &knob_mobs {
            play!(stage; fade_in(stage.arena_mut(), *g, v(0.0, -0.15), 1.0)?.rt(0.4));
        }
        let q = predict_pause(kit, stage, v(5.8, 2.1))?;

        // 12_5: the slider over expected caught twice
        let stops: Vec<u64> = vec![25, 100, 400, lake];
        // equal steps for each fourfold step, the whole lake past a break
        let slider_x = |k: u64| -> f64 {
            match stops.iter().position(|&s| s == k) {
                Some(i) => -4.8 + 3.2 * i as f64,
                None => 0.0,
            }
        };
        let track_y = 2.55;
        let (near, far) = (slider_x(stops[stops.len() - 2]), slider_x(lake));
        let solid = Line::new(v(slider_x(stops[0]), track_y), v(near, track_y))
            .style(line_style(Palette::dim(), 4.0))
            .build()?;
        let gap = DashedLine::new(v(near, track_y), v(far, track_y))
            .dash_length(0.12)
            .style(line_style(Palette::dim(), 4.0))
            .build()
            .map_err(|e| model(e.to_string()))?;
        let track = stage.arena_mut().add(v_group([solid, gap]));
        let mut stop_labels = Vec::new();
        for &k in &stops {
            let word = if k == lake {
                format!("{} (all of it)", thousands(k))
            } else {
                thousands(k)
            };
            let at = v(slider_x(k), track_y - 0.38);
            stop_labels.push(kit.text(stage, &word, 22.0, Palette::dim(), at)?);
        }
        let slider_title = kit.text(stage, "expected caught twice", 26.0, Palette::hero(), v(0.0, 3.25))?;
        let knob = stage.arena_mut().add(
            Circle::new()
                .radius(0.14)
                .arc_center(v(slider_x(stops[0]), track_y))
                .style(flat(Palette::hero(), 1.0))
                .build(),
        );
        let axis = x_axis(kit, stage, &frame, -0.75, &[-40.0, -20.0, 0.0, 20.0, 40.0, 60.0], |t| {
            if t == 0.0 { thousands(lake) } else { signed_pct(t) }
        })?;
        let truth_tick = stage.arena_mut().add(
            Line::new(v(xs(0.0), -0.75), v(xs(0.0), 0.5))
                .style(line_style(Palette::teal(), 2.0))
                .build()?,
        );

        // each stop's numbers, from the careful range at that stop
        struct Stop {
            m: u64,
            lo: u64,
            hi: u64,
            typical: f64,
            thumb: f64,
        }
        let stop_data: Vec<Stop> = stops
            .iter()
            .map(|&k| {
                let m = (lake * k).isqrt();
                let (a, b) = careful_range(m, m, k)?;
                let share = 1.0 - m as f64 / lake as f64;
                Ok(Stop {
                    m,
                    lo: a,
                    hi: b,
                    typical: 100.0 * (share * share / k as f64).sqrt(),
                    thumb: 100.0 / (k as f64).sqrt(),
                })
            })
            .collect::<Result<_, String>>()
            .map_err(model)?;
        let info = |kit: &Kit, stage: &mut Stage<'_>, i: usize| -> fmn::Result<Mob> {
            let s = &stop_data[i];
            let g = stage.arena_mut().add(VMobject::new());
            let catches = kit.text(
                stage,
                &format!("marked = second catch = {}", thousands(s.m)),
                26.0,
                Palette::ink(),
                v(0.0, 1.5),
            )?;
            stage.attach(g, catches)?;
            if s.lo == s.hi {
                let one = kit.text(
                    stage,
                    &format!("one size: {}", thousands(s.lo)),
                    30.0,
                    Palette::teal(),
                    v(0.0, -2.0),
                )?;
                stage.attach(g, one)?;
            } else {
                let ends = [
                    (signed_pct(pct(s.lo)), v(xs(pct(s.lo)) - 0.15, bar_y + 0.45), RIGHT),
                    (signed_pct(pct(s.hi)), v(xs(pct(s.hi)) + 0.15, bar_y + 0.45), LEFT),
                ];
                for (word, at, edge) in ends {
                    let m = kit.text(stage, &word, 26.0, Palette::teal(), at)?;
                    stage.move_to(m, at, edge);
                    stage.attach(g, m)?;
                }
                let typical = kit.text(
                    stage,
                    &format!("typical error ±{:.0}%", s.typical),
                    28.0,
                    Palette::ink(),
                    v(0.0, -2.0),
                )?;
                let thumb = kit.text(
                    stage,
                    &format!("rule of thumb: 1 ÷ √(caught twice) = {:.0}%", s.thumb),
                    24.0,
                    Palette::dim(),
                    v(0.0, -2.55),
                )?;
                stage.attach(g, typical)?;
                stage.attach(g, thumb)?;
            }
            Ok(g)
        };
        let line = Spoken::say(kit, stage, "12_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_out(stage.arena_mut(), knob_mobs[0], ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), knob_mobs[1], ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), knob_mobs[2], ORIGIN, 1.0)?.rt(0.5),
        );
        let mut first_bar = stage.arena_mut().add(bar(stop_data[0].lo, stop_data[0].hi)?);
        let mut shown = info(kit, stage, 0)?;
        play!(stage;
            fade_in(stage.arena_mut(), track, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), slider_title, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), knob, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), truth_tick, ORIGIN, 1.0)?.rt(0.6),
        );
        for l in &stop_labels {
            play!(stage; fade_in(stage.arena_mut(), *l, ORIGIN, 1.0)?.rt(0.25));
        }
        play!(stage;
            replacement_transform(range, first_bar).rt(0.9),
            fade_in(stage.arena_mut(), shown, ORIGIN, 1.0)?.rt(0.9),
        );
        let cues = ["Four times as many", "Catching a bigger share", "catching all of it"];
        for i in 1..stop_data.len() {
            line.at(stage, cues[i - 1])?;
            let next_bar = stage.arena_mut().add(bar(stop_data[i].lo, stop_data[i].hi)?);
            let next_info = info(kit, stage, i)?;
            let slide = knob
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.2),
                    ..AnimateArgs::default()
                })?
                .move_to(v(slider_x(stops[i]), track_y), ORIGIN)?;
            play!(stage;
                slide,
                replacement_transform(first_bar, next_bar).rt(1.2),
                replacement_transform(shown, next_info).rt(1.2),
            );
            first_bar = next_bar;
            shown = next_info;
            if i < stop_data.len() - 1 {
                let ratio =
                    (stop_data[i].hi - stop_data[i].lo) as f64 / (stop_data[i - 1].hi - stop_data[i - 1].lo) as f64;
                let tag = kit.text(
                    stage,
                    &format!("width × {ratio:.2}"),
                    24.0,
                    Palette::dim(),
                    v(4.9, bar_y + 0.95),
                )?;
                play!(stage; fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5));
                stage.wait(1.0)?;
                play!(stage; fade_out(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.4));
            }
        }
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Narration truth: every verdict chapter 12 speaks, on the data it draws.
    use super::*;

    #[test]
    fn the_band_keeps_exactly_the_careful_range() {
        // 12_1: "keep the sizes where it would", every size the sweep crosses
        let (fm, fnn, fk) = FELLER;
        let (lo, hi) = careful_range(fm, fnn, fk).unwrap();
        for n in (6_000..=16_000)
            .step_by(7)
            .chain(lo - 3..=lo + 3)
            .chain(hi - 3..=hi + 3)
        {
            let (a, b) = Law::new(n, fm, fnn).middle().unwrap();
            assert_eq!((a..=b).contains(&fk), (lo..=hi).contains(&n), "lake {n}");
        }
    }

    #[test]
    fn it_rings_the_true_size_whatever_that_size_is() {
        // 12_2: "at least ninety-five times in a hundred, whatever that size is";
        // the coverage at a lake size is the chance its catch lands in its
        // middle 95%. The oracle's minimum over 27,901 sizes is 95.007% at
        // 10,967; here, every size around it and a spread of others.
        let (fm, fnn, _) = FELLER;
        let sizes = (10_900..=11_050).chain((2_000..=60_000).step_by(997));
        let mut worst = (1.0, 0);
        for n in sizes {
            let c = Law::new(n, fm, fnn).middle_mass().unwrap();
            assert!(c >= 0.95, "lake {n}: coverage {c}");
            if c < worst.0 {
                worst = (c, n);
            }
        }
        assert_eq!(worst.1, 10_967, "A-CH12.coverage.argmin_N");
        assert!((worst.0 - 0.950_073_362_903_808_2).abs() < 1e-12, "A-CH12.coverage.min");
    }

    #[test]
    fn the_spoken_ranges_round_as_said() {
        let (fm, fnn, fk) = FELLER;
        let (lo, hi) = careful_range(fm, fnn, fk).unwrap();
        // 12_3: "from about eighty-four hundred to about twelve thousand one hundred"
        assert_eq!(((lo + 50) / 100 * 100, (hi + 50) / 100 * 100), (8_400, 12_100));
        // "close to the range we watched when the lake was known": chapter
        // 09's 8,475 to 12,048, each end within 1% of the lake
        let truth = Law::new(runs::LAKE as u64, fm, fnn);
        let (q025, q975) = truth.estimate_quantiles().unwrap();
        let (e_lo, e_hi) = ((fm * fnn) as f64 / q025 as f64, (fm * fnn) as f64 / q975 as f64);
        assert!((e_lo - lo as f64).abs() < 100.0 && (e_hi - hi as f64).abs() < 100.0);
        assert_ne!(
            (e_lo.round() as u64, e_hi.round() as u64),
            (lo, hi),
            "a different question"
        );
    }

    #[test]
    fn four_times_as_many_roughly_halves_it_and_all_of_it_leaves_one_size() {
        // 12_5, on the slider's stops
        let lake = runs::LAKE as u64;
        let width = |k: u64| {
            let m = (lake * k).isqrt();
            let (a, b) = careful_range(m, m, k).unwrap();
            (b - a) as f64
        };
        for (a, b) in [(25, 100), (100, 400)] {
            let r = width(b) / width(a);
            assert!((0.4..0.5).contains(&r), "{a} to {b}: × {r}");
        }
        // "helps more and more": the typical error over the rule of thumb,
        // √((1 − M/N)²/k) ÷ (1/√k) = 1 − M/N, falls at each stop
        let trim: Vec<f64> = [25u64, 100, 400]
            .iter()
            .map(|&k| 1.0 - (lake * k).isqrt() as f64 / lake as f64)
            .collect();
        assert!(trim[0] > trim[1] && trim[1] > trim[2]);
        assert_eq!(careful_range(lake, lake, lake), Ok((lake, lake)));
    }

    /// The phrases this chapter's beats wait for, by line (checked by a test).
    const BEATS: [(&str, &str); 9] = [
        ("12_1", "For each lake size"),
        ("12_1", "Keep the sizes"),
        ("12_2", "In the long run"),
        ("12_3", "Close to the range"),
        ("12_3", "There the lake"),
        ("12_3", "Here the catch"),
        ("12_5", "Four times as many"),
        ("12_5", "Catching a bigger share"),
        ("12_5", "catching all of it"),
    ];

    #[test]
    fn a_phrase_sits_at_its_share_of_the_line() {
        let spoken = Spoken {
            start: 10.0,
            len: 8.0,
            words: "one two three, four".split_whitespace().map(plain).collect(),
        };
        assert_eq!(spoken.offset("one"), Ok(0.0));
        assert_eq!(spoken.offset("Three four"), Ok(4.0));
        assert!(spoken.offset("five").is_err());
        // every phrase this chapter waits for is in its line
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in BEATS {
            let words: Vec<String> = script.text(id).unwrap().split_whitespace().map(plain).collect();
            let s = Spoken {
                start: 0.0,
                len: 1.0,
                words,
            };
            assert!(s.offset(phrase).is_ok(), "{id}: {phrase:?}");
        }
    }

    #[test]
    fn the_cascade_lands_inside_its_window() {
        let land = landings(94, 0.42);
        assert!(land.windows(2).all(|w| w[1] > w[0]));
        assert!(*land.last().unwrap() < 5.0, "the listen window stays short");
    }
}
