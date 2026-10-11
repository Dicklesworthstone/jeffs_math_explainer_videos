//! Chapter 13, one rule for every failure: the soup guess aims at the lake
//! whose marked share the second catch holds. Each failure bends the
//! mosaic's line (the two columns' green shares part), and the estimate is
//! the lake size that straightens it again, read on a gauge against the
//! lake on marking day. Every case is a seeded model of verify2.py (section
//! F), run here draw for draw (`runs::verify2`): bold fish (13_2–13_3), fish
//! that love or avoid the bait (13_4), lost marks (13_5), newcomers (13_6),
//! deaths (13_7–13_8), and marks that never mix (13_9, on chapter 03's lake).

use std::f64::consts::PI;

use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style};
use super::lake::{GAP, LAKE_C, LAKE_H, LAKE_W, LakeArt, lake_p, plated, shuffled};
use super::ring_toss::Spoken;
use crate::model::{SEED, SplitMix64};
use crate::runs::{self, BOLD_SHARE, CAUGHT, DEATHS, HALVES, HAPPY, LAKE, LOSS, MARKED, NEWCOMERS, SHY};

/// The mosaic: left edge, top, height, and scene units per fish of width.
const MOS_X: f64 = -6.6;
const MOS_TOP: f64 = 2.0;
const MOS_H: f64 = 4.2;
const PER_FISH: f64 = 5.4 / LAKE as f64;
/// The mosaic shows the top quarter of each column: every share here is
/// near one in ten, and the bend is the point.
const VIEW: f64 = 0.25;
/// The gauge: centre, radius, and its full scale (fish).
const GAUGE_C: Vec3 = [4.2, -0.6, 0.0];
const GAUGE_R: f64 = 1.95;
const GAUGE_MAX: f64 = 30_000.0;
/// Where the mode's name, its model and the reading sit.
const TITLE_AT: Vec3 = [-3.0, 3.45, 0.0];
const MODEL_AT: Vec3 = [MOS_X, -2.95, 0.0];
const READ_AT: Vec3 = [GAUGE_C[0], -2.3, 0.0];
/// The third caption line under the gauge (the lake at the second catch).
const SECOND_AT: Vec3 = [GAUGE_C[0], -1.84, 0.0];

/// A value's angle on the gauge: 0 at the left, the full scale at the right.
fn angle(value: f64) -> f64 {
    PI * (1.0 - (value / GAUGE_MAX).clamp(0.0, 1.0))
}

fn on_gauge(value: f64, r: f64) -> Vec3 {
    let a = angle(value);
    v(GAUGE_C[0] + r * a.cos(), GAUGE_C[1] + r * a.sin())
}

/// An estimate as the gauge reads it, to the nearest ten ("≈ 7,750").
fn reading(estimate: f64) -> String {
    format!("≈ {}", thousands(((estimate / 10.0).round() * 10.0) as u64))
}

/// A whole percent of a fraction (`(1, 5)` → "20%").
fn pct((a, b): (u64, u64)) -> String {
    format!("{}%", 100 * a / b)
}

/// Scene error from a model error.
fn model(e: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(e.into_boxed_str())))
}

/// A filled rectangle by its top-left corner, inset by the table's gap.
fn cell(x0: f64, y1: f64, w: f64, h: f64, style: Style) -> VMobject {
    Rectangle::new()
        .width((w - GAP).max(0.01))
        .height((h - GAP).max(0.01))
        .style(style)
        .build()
        .map(|r| r.moved_to(v(x0 + w / 2.0, y1 - h / 2.0)))
        .unwrap_or_else(|_| VMobject::new())
}

/// Where a mosaic is drawn and how: its left edge and top, its height,
/// scene units per counted thing of width, the share of each column shown
/// (the rest torn off below), and the size of the second catch.
#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub x: f64,
    pub top: f64,
    pub h: f64,
    pub per: f64,
    pub view: f64,
    pub caught: f64,
}

/// Chapter 13's mosaic: the top quarter of each column, every share here
/// being near one in ten.
const CH13: Layout = Layout {
    x: MOS_X,
    top: MOS_TOP,
    h: MOS_H,
    per: PER_FISH,
    view: VIEW,
    caught: CAUGHT as f64,
};

/// The mosaic's state, each a tracker: the marked column's width and the
/// unmarked column's (in counted things), the caught-twice count, the share
/// of each column that died (13_7), and the newcomers in the unmarked
/// column (13_6).
#[derive(Clone, Copy)]
pub(super) struct Mosaic {
    pub marked: Mob,
    pub unmarked: Mob,
    pub twice: Mob,
    pub dead: Mob,
    pub newcomers: Mob,
}

impl Mosaic {
    /// The trackers at (`marked`, `unmarked`, `twice`), and the picture
    /// redrawn from them every frame: four cells in the hero mosaic's hues
    /// and the line across the table at each column's second-catch share.
    pub(super) fn build(stage: &mut Stage<'_>, at: Layout, (marked, unmarked, twice): (f64, f64, f64)) -> (Self, Mob) {
        let m = Self {
            marked: stage.add_value_tracker(marked),
            unmarked: stage.add_value_tracker(unmarked),
            twice: stage.add_value_tracker(twice),
            dead: stage.add_value_tracker(0.0),
            newcomers: stage.add_value_tracker(0.0),
        };
        let Layout {
            x,
            top,
            h,
            per,
            view,
            caught: n,
        } = at;
        let picture = stage.always_redraw(move |s| {
            let get = |t: Mob| s.tracker_value(t).unwrap_or(0.0);
            let (marked, unmarked, k) = (get(m.marked), get(m.unmarked), get(m.twice));
            let (wm, wu) = (marked * per, unmarked * per);
            let (dead, new) = (get(m.dead), get(m.newcomers) * per);
            let (hl, hr) = (h * k / marked / view, h * (n - k) / unmarked / view);
            let xr = x + wm;
            let mut parts = vec![
                cell(x, top, wm, hl, flat(Role::marked(), 0.6)),
                cell(x, top - hl, wm, h - hl, flat(Role::marked(), 0.16)),
                cell(xr, top, wu, hr, flat(Role::second(), 0.5)),
                cell(xr, top - hr, wu, h - hr, flat(hex("#10262A"), 1.0)),
            ];
            // newcomers: the unmarked column's right-hand strip, lighter
            if new > 1e-6 {
                parts.push(cell(xr + wu - new, top, new, h, flat(Role::unmarked(), 0.14)));
            }
            // the dead: the right-hand share of each column, darkened
            if dead > 1e-6 {
                for (x0, w) in [(x, wm), (xr, wu)] {
                    parts.push(cell(
                        x0 + w * (1.0 - dead),
                        top,
                        w * dead,
                        h,
                        flat(hex("#030405"), 0.72),
                    ));
                }
            }
            // the line across the table, at each column's green share
            let (yl, yr) = (top - hl - GAP / 2.0, top - hr - GAP / 2.0);
            let ink = line_style(Role::marked(), 5.0);
            let seg = |a: Vec3, b: Vec3| Line::new(a, b).style(ink).build().ok();
            parts.extend(seg(v(x, yl), v(xr, yl)));
            if (yl - yr).abs() > 1e-4 {
                parts.extend(seg(v(xr, yl), v(xr, yr)));
            }
            parts.extend(seg(v(xr, yr), v(xr + wu, yr)));
            // a torn bottom edge when the columns go on below the view
            if view < 1.0 {
                let (bottom, right) = (top - h, xr + wu);
                let teeth = ((right - x) / 0.16).ceil().max(1.0) as usize;
                let mut torn = vec![v(x - 0.05, bottom - 0.12)];
                for i in 0..=teeth {
                    let tx = x + (right - x) * i as f64 / teeth as f64;
                    torn.push(v(tx, bottom + if i % 2 == 0 { 0.02 } else { 0.1 }));
                }
                torn.push(v(right + 0.05, bottom - 0.12));
                parts.push(
                    Polygon::new(torn)
                        .style(flat(hex(explainer_kit::BACKGROUND), 1.0))
                        .build(),
                );
            }
            s.add(v_group(parts))
        });
        (m, picture)
    }
}

/// Tween several trackers together.
pub(super) fn go(stage: &mut Stage<'_>, to: &[(Mob, f64)], secs: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(t, value) in to {
        anims.push(stage.prepare(tween(t, value, secs)?)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// The gauge's fixed parts: the arc, ticks every 5,000, labels every
/// 10,000, the truth's teal tick, and the caption.
fn gauge(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let mut parts = Vec::new();
    let steps = 48;
    for i in 0..steps {
        let (a, b) = (
            GAUGE_MAX * f64::from(i) / f64::from(steps),
            GAUGE_MAX * f64::from(i + 1) / f64::from(steps),
        );
        parts.push(
            Line::new(on_gauge(a, GAUGE_R), on_gauge(b, GAUGE_R))
                .style(line_style(Palette::dim(), 6.0))
                .build()?,
        );
    }
    for i in 0..=6 {
        let value = GAUGE_MAX * f64::from(i) / 6.0;
        let len = if i % 2 == 0 { 0.22 } else { 0.12 };
        parts.push(
            Line::new(on_gauge(value, GAUGE_R - len), on_gauge(value, GAUGE_R))
                .style(line_style(Palette::dim(), 3.0))
                .build()?,
        );
    }
    let truth = LAKE as f64;
    parts.push(
        Line::new(on_gauge(truth, GAUGE_R - 0.32), on_gauge(truth, GAUGE_R + 0.12))
            .style(line_style(Role::dark(), 5.0))
            .build()?,
    );
    let g = stage.arena_mut().add(v_group(parts));
    for i in [0, 2, 4, 6] {
        let value = GAUGE_MAX * f64::from(i) / 6.0;
        let color = if (value - truth).abs() < 1.0 {
            Role::dark()
        } else {
            Palette::dim()
        };
        let m = kit.text(
            stage,
            &thousands(value as u64),
            22.0,
            color,
            on_gauge(value, GAUGE_R + 0.36),
        )?;
        stage.attach(g, m)?;
    }
    let caption = kit.text(stage, "estimated lake size", 26.0, Palette::ink(), v(GAUGE_C[0], -1.1))?;
    let truth_line = kit.text(
        stage,
        &format!("truth on marking day: {}", thousands(LAKE as u64)),
        22.0,
        Role::dark(),
        v(GAUGE_C[0], -1.47),
    )?;
    stage.attach(g, caption)?;
    stage.attach(g, truth_line)?;
    Ok(g)
}

/// The needle, redrawn from its tracker, with a red wedge between the truth
/// and the reading (the bias) whenever they differ.
fn needle(stage: &mut Stage<'_>, at: Mob) -> Mob {
    stage.always_redraw(move |s| {
        let value = s.tracker_value(at).unwrap_or(LAKE as f64);
        let mut parts = Vec::new();
        let (a0, a1) = (angle(LAKE as f64), angle(value));
        if (a1 - a0).abs() > 1e-3 {
            let steps = 24;
            let mut pts = vec![GAUGE_C];
            for i in 0..=steps {
                let a = a0 + (a1 - a0) * f64::from(i) / f64::from(steps);
                pts.push(v(
                    GAUGE_C[0] + 0.88 * GAUGE_R * a.cos(),
                    GAUGE_C[1] + 0.88 * GAUGE_R * a.sin(),
                ));
            }
            parts.push(Polygon::new(pts).style(flat(Role::bias(), 0.3)).build());
        }
        let tip = on_gauge(value, GAUGE_R - 0.1);
        let (dx, dy) = (-a1.sin() * 0.06, a1.cos() * 0.06);
        parts.push(
            Polygon::new([
                v(GAUGE_C[0] + dx, GAUGE_C[1] + dy),
                tip,
                v(GAUGE_C[0] - dx, GAUGE_C[1] - dy),
            ])
            .style(flat(Role::dark(), 1.0))
            .build(),
        );
        parts.push(
            Circle::new()
                .radius(0.1)
                .arc_center(GAUGE_C)
                .style(flat(Role::dark(), 1.0))
                .build(),
        );
        s.add(v_group(parts))
    })
}

/// One failure mode as the chapter shows it: its name, its model's label,
/// the median soup guess of its runs and their median caught-twice count.
struct Case {
    title: String,
    label: String,
    median: f64,
    twice: f64,
}

/// What chapter 13 keeps on stage between beats.
struct Bench {
    mos: Mosaic,
    estimate: Mob,
    readout: Mob,
    title: Mob,
    label: Mob,
}

impl Bench {
    /// The next case's name and model label, swapped in place.
    fn relabel(&mut self, kit: &Kit, stage: &mut Stage<'_>, case: &Case) -> fmn::Result<()> {
        let t = kit.text(stage, &case.title, 32.0, Palette::ink(), TITLE_AT)?;
        let l = kit.text_left(stage, &case.label, 24.0, Palette::dim(), MODEL_AT)?;
        play!(stage;
            replacement_transform(self.title, t).rt(0.5),
            replacement_transform(self.label, l).rt(0.5),
        );
        (self.title, self.label) = (t, l);
        Ok(())
    }

    /// Back to the true lake, the needle at the truth.
    fn reset(&mut self, kit: &Kit, stage: &mut Stage<'_>, twice: f64) -> fmn::Result<()> {
        let m = self.mos;
        go(
            stage,
            &[
                (m.marked, MARKED as f64),
                (m.unmarked, (LAKE - MARKED) as f64),
                (m.twice, twice),
                (m.dead, 0.0),
                (m.newcomers, 0.0),
                (self.estimate, LAKE as f64),
            ],
            0.5,
        )?;
        self.read(kit, stage, LAKE as f64)
    }

    /// The reading under the gauge, swapped in place.
    fn read(&mut self, kit: &Kit, stage: &mut Stage<'_>, value: f64) -> fmn::Result<()> {
        let r = kit.text(stage, &reading(value), 40.0, Role::dark(), READ_AT)?;
        play!(stage; replacement_transform(self.readout, r).rt(0.4));
        self.readout = r;
        Ok(())
    }

    /// The estimate: the unmarked column widens or narrows to the lake
    /// that makes the line straight for the typical run (M·n / k fish in
    /// all, its newcomers included) while the needle swings to the median
    /// of the runs; then the reading.
    fn straighten(&mut self, kit: &Kit, stage: &mut Stage<'_>, case: &Case) -> fmn::Result<()> {
        let lake = (MARKED * CAUGHT) as f64 / case.twice;
        go(
            stage,
            &[(self.mos.unmarked, lake - MARKED as f64), (self.estimate, case.median)],
            1.3,
        )?;
        kit.mark(stage, "tick:needle");
        self.read(kit, stage, case.median)
    }
}

/// A mark on the gauge for the lake at the second catch, with its caption
/// line; detached, as one group.
fn second_mark(kit: &Kit, stage: &mut Stage<'_>, lake: u64) -> fmn::Result<Mob> {
    let tick = Line::new(
        on_gauge(lake as f64, GAUGE_R - 0.28),
        on_gauge(lake as f64, GAUGE_R + 0.1),
    )
    .style(line_style(Palette::ink(), 4.0))
    .build()?;
    let g = stage.arena_mut().add(v_group([tick]));
    let words = kit.text(
        stage,
        &format!("lake at the second catch: {}", thousands(lake)),
        22.0,
        Palette::ink(),
        SECOND_AT,
    )?;
    stage.attach(g, words)?;
    Ok(g)
}

/// A small school at the top right in which 3 fish in 10 are bold
/// (brighter, bigger, haloed): (the plain fish, the bold ones), detached.
fn school(stage: &mut Stage<'_>) -> fmn::Result<(Mob, Mob)> {
    let (w, h, centre) = (4.6, 1.15, v(GAUGE_C[0], 2.45));
    let (cols, rows) = (10usize, 3usize);
    let total = cols * rows;
    let bold_count = total * BOLD_SHARE.0 as usize / BOLD_SHARE.1 as usize;
    let mut rng = SplitMix64::new(SEED ^ 0x1302);
    let order = shuffled(total, &mut rng);
    let plain = stage.arena_mut().add(VMobject::new());
    let bold = stage.arena_mut().add(VMobject::new());
    for (slot, &i) in order.iter().enumerate() {
        let (c, r) = (slot % cols, slot / cols);
        let x = centre[0] - w / 2.0 + w * (c as f64 + 0.5 + 0.3 * (rng.uniform() - 0.5)) / cols as f64;
        let y = centre[1] - h / 2.0 + h * (r as f64 + 0.5 + 0.3 * (rng.uniform() - 0.5)) / rows as f64;
        let turn = 0.5 * (rng.uniform() - 0.5);
        let is_bold = i < bold_count;
        let glyph = if is_bold {
            let outer = Circle::new().radius(0.13).style(flat(Role::unmarked(), 0.12)).build();
            let inner = Circle::new().radius(0.085).style(flat(Role::unmarked(), 0.22)).build();
            v_group([outer, inner, fish(hex("#D4F5FD")).mob])
        } else {
            fish(Role::unmarked()).mob
        };
        let m = stage.arena_mut().add(glyph);
        stage.scale(m, if is_bold { 1.6 } else { 1.15 });
        stage.rotate(m, turn, OUT, Some(ORIGIN), None);
        stage.shift(m, v(x, y));
        stage.attach(if is_bold { bold } else { plain }, m)?;
    }
    Ok((plain, bold))
}

pub struct Failures {
    pub kit: Kit,
}

impl SceneConstruct for Failures {
    fn name(&self) -> &str {
        "13_one_rule_for_every_failure"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let runs = runs::verify2();
        let per_runs = format!("median of {} runs", runs::FAILURE_RUNS);
        let case = |key: &str, title: &str, label: String| -> Case {
            let f = runs.failure(key);
            Case {
                title: title.to_owned(),
                label,
                median: f.median(),
                twice: f.median_caught_twice(),
            }
        };
        let ideal = case("ideal", "", String::new());

        // 13_1: the act card, then the true lake's mosaic and the gauge
        let act = kit.text_styled(
            stage,
            "When does it fail?",
            64.0,
            Role::marked(),
            v(0.0, 0.3),
            true,
            false,
        )?;
        let line = Spoken::say(&kit, stage, "13_1")?;
        play!(stage; write(stage.arena(), act).rt(1.0));
        line.at(stage, "On average")?;
        play!(stage; fade_out(stage.arena_mut(), act, ORIGIN, 1.0)?.rt(0.5));
        let (mos, picture) = Mosaic::build(stage, CH13, (MARKED as f64, (LAKE - MARKED) as f64, ideal.twice));
        let marked_head = kit.text_left(stage, "marked", 24.0, Role::marked(), v(MOS_X, MOS_TOP + 0.3))?;
        let unmarked_head = kit.text(
            stage,
            "unmarked",
            24.0,
            Palette::ink(),
            v(MOS_X + (MARKED + (LAKE - MARKED) / 2) as f64 * PER_FISH, MOS_TOP + 0.3),
        )?;
        let dark_word = kit.text(stage, "the dark cell", 24.0, Palette::dim(), v(-3.9, -1.25))?;
        let crop_word = kit.text(
            stage,
            "(top quarter of each column shown)",
            18.0,
            Palette::dim(),
            v(-3.9, -1.6),
        )?;
        let dial = gauge(&kit, stage)?;
        let estimate = stage.add_value_tracker(ideal.median);
        let pointer = needle(stage, estimate);
        let readout = kit.text(stage, &reading(ideal.median), 40.0, Role::dark(), READ_AT)?;
        stage.add_to_scene(picture)?;
        play!(stage;
            fade_in(stage.arena_mut(), marked_head, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), unmarked_head, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), dark_word, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), crop_word, ORIGIN, 1.0)?.rt(0.8),
        );
        stage.add_to_scene(pointer)?;
        play!(stage;
            fade_in(stage.arena_mut(), dial, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), readout, ORIGIN, 1.0)?.rt(0.8),
        );
        let note = precisely(
            &kit,
            stage,
            "Mn/k = M ÷ (marked share of the second catch). It aims at a lake of L fish (Mn/E[k] = L) exactly \
             when that share is, on average, M/L. Even then it runs a little high (chapter 09).",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        kit.hold(stage)?;

        // 13_2: bold fish, both times
        let bold = case(
            "bold_fish",
            "bold fish",
            format!(
                "model: {} in {} fish {}× as catchable, both times; {per_runs}",
                BOLD_SHARE.0,
                BOLD_SHARE.1,
                runs::BOLD_FACTOR
            ),
        );
        let (plain, bolder) = school(stage)?;
        let title = kit.text(stage, &bold.title, 32.0, Palette::ink(), TITLE_AT)?;
        let label = kit.text_left(stage, &bold.label, 24.0, Palette::dim(), MODEL_AT)?;
        let line = Spoken::say(&kit, stage, "13_2")?;
        play!(stage;
            fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), plain, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.8),
        );
        line.at(stage, "are bold")?;
        play!(stage; fade_in(stage.arena_mut(), bolder, ORIGIN, 1.0)?.rt(0.8));
        play!(stage; fmn::animation::indicate(stage.arena_mut(), bolder, 1.15, None)?.rt(1.0));
        let q = predict_pause(&kit, stage, v(6.5, 1.1))?;

        // 13_3: too small. The bold fish crowd the caught-twice cell, the
        // line bends, and the estimate straightens it by shrinking the dark
        // cell; the dashed outline keeps the lake on marking day
        let ghost = {
            let (x0, x1, y0, y1) = (MOS_X, MOS_X + LAKE as f64 * PER_FISH, MOS_TOP - MOS_H, MOS_TOP);
            let mut sides = Vec::new();
            for (a, b) in [(v(x0, y0), v(x1, y0)), (v(x1, y0), v(x1, y1)), (v(x1, y1), v(x0, y1))] {
                sides.push(
                    DashedLine::new(a, b)
                        .dash_length(0.1)
                        .style(line_style(Role::dark(), 2.0))
                        .build()
                        .map_err(|e| model(e.to_string()))?,
                );
            }
            stage.arena_mut().add(v_group(sides))
        };
        let ghost_label = kit.text(
            stage,
            &format!("the lake on marking day: {}", thousands(LAKE as u64)),
            22.0,
            Role::dark(),
            v(MOS_X + LAKE as f64 * PER_FISH / 2.0, MOS_TOP - MOS_H - 0.28),
        )?;
        let line = Spoken::say(&kit, stage, "13_3")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ghost, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ghost_label, ORIGIN, 1.0)?.rt(0.6),
        );
        let mut bench = Bench {
            mos,
            estimate,
            readout,
            title,
            label,
        };
        line.at(stage, "Bold fish land")?;
        go(stage, &[(mos.twice, bold.twice)], 1.4)?;
        line.at(stage, "so the dark cell shrinks")?;
        bench.straighten(&kit, stage, &bold)?;
        play!(stage;
            fade_out(stage.arena_mut(), plain, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), bolder, ORIGIN, 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;

        // 13_4: fish that love the bait, then fish that avoid it
        let happy = case(
            "trap_happy",
            "fish that love the bait",
            format!("model: marked fish {HAPPY}× as catchable the second time; {per_runs}"),
        );
        let shy = case(
            "trap_shy",
            "fish that avoid it",
            format!(
                "model: marked fish {}/{}× as catchable the second time; {per_runs}",
                SHY.0, SHY.1
            ),
        );
        let line = Spoken::say(&kit, stage, "13_4")?;
        for (c, cue) in [(&happy, "Fish that learn to love"), (&shy, "Fish that learn to avoid")] {
            line.at(stage, cue)?;
            bench.relabel(&kit, stage, c)?;
            bench.reset(&kit, stage, ideal.twice)?;
            go(stage, &[(mos.twice, c.twice)], 0.7)?;
            bench.straighten(&kit, stage, c)?;
        }
        kit.hold(stage)?;

        // 13_5: marks that wash off; marked fish look unmarked
        let lost = case(
            "marks_lost_20pct",
            "marks that wash off",
            format!("model: {} of marks lost; {per_runs}", pct(LOSS)),
        );
        let line = Spoken::say(&kit, stage, "13_5")?;
        bench.relabel(&kit, stage, &lost)?;
        bench.reset(&kit, stage, ideal.twice)?;
        go(stage, &[(mos.twice, lost.twice)], 0.9)?;
        line.at(stage, "That pushes it up")?;
        bench.straighten(&kit, stage, &lost)?;
        kit.hold(stage)?;

        // 13_6: newcomers; the line is straight for the lake at the second catch
        let new = case(
            "newcomers_2000",
            "newcomers",
            format!(
                "model: {} unmarked fish join between the catches; {per_runs}",
                thousands(NEWCOMERS as u64)
            ),
        );
        let grown = (LAKE + NEWCOMERS) as u64;
        let mark_12k = second_mark(&kit, stage, grown)?;
        let strip_x = MOS_X + (LAKE as f64 + NEWCOMERS as f64 / 2.0) * PER_FISH;
        let new_word = kit.text(stage, "newcomers", 20.0, Role::unmarked(), v(strip_x, -1.9))?;
        let line = Spoken::say(&kit, stage, "13_6")?;
        bench.relabel(&kit, stage, &new)?;
        bench.reset(&kit, stage, ideal.twice)?;
        go(
            stage,
            &[
                (mos.unmarked, (LAKE - MARKED + NEWCOMERS) as f64),
                (mos.newcomers, NEWCOMERS as f64),
            ],
            0.9,
        )?;
        play!(stage; fade_in(stage.arena_mut(), new_word, ORIGIN, 1.0)?.rt(0.4));
        line.at(stage, "dilute the marks")?;
        go(stage, &[(mos.twice, new.twice)], 0.8)?;
        line.at(stage, "so you count")?;
        bench.straighten(&kit, stage, &new)?;
        play!(stage; fade_in(stage.arena_mut(), mark_12k, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 13_7: deaths; the lake thins
        let deaths = case(
            "deaths_30pct",
            "deaths",
            format!(
                "model: each fish dies with chance {} in {}; {per_runs}",
                DEATHS.0, DEATHS.1
            ),
        );
        let line = Spoken::say(&kit, stage, "13_7")?;
        play!(stage;
            fade_out(stage.arena_mut(), mark_12k, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), new_word, ORIGIN, 1.0)?.rt(0.4),
        );
        bench.relabel(&kit, stage, &deaths)?;
        bench.reset(&kit, stage, ideal.twice)?;
        let dead_share = DEATHS.0 as f64 / DEATHS.1 as f64;
        let died_x = MOS_X + (MARKED as f64 + (LAKE - MARKED) as f64 * (1.0 - dead_share / 2.0)) * PER_FISH;
        let died_word = kit.text(stage, "died", 22.0, Palette::dim(), v(died_x, -1.9))?;
        line.at(stage, "die between")?;
        go(stage, &[(mos.dead, dead_share)], 1.2)?;
        play!(stage; fade_in(stage.arena_mut(), died_word, ORIGIN, 1.0)?.rt(0.4));
        let q = predict_pause(&kit, stage, v(1.6, 1.1))?;

        // 13_8: the estimate doesn't move: it counts the lake on marking
        // day, not the 7,000 left
        let (left_num, left_den) = runs::deaths_left();
        let mark_7k = second_mark(&kit, stage, left_num / left_den)?;
        let line = Spoken::say(&kit, stage, "13_8")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        go(stage, &[(mos.twice, deaths.twice), (estimate, deaths.median)], 0.6)?;
        bench.read(&kit, stage, deaths.median)?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), bench.readout, 1.15, None)?.rt(1.0));
        line.at(stage, "not the seven thousand left")?;
        play!(stage; fade_in(stage.arena_mut(), mark_7k, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 13_9: marks that never mix, on chapter 03's lake
        let mosaic_parts = [
            picture,
            ghost,
            ghost_label,
            marked_head,
            unmarked_head,
            dark_word,
            crop_word,
            died_word,
            mark_7k,
        ];
        self.unmixed(&kit, stage, &mut bench, &runs, &mosaic_parts, &per_runs)?;

        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl Failures {
    /// 13_9: all the marks in the half by the dock; the second catch takes
    /// 800 from one half and 200 from the other.
    fn unmixed(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        bench: &mut Bench,
        runs: &runs::Verify2,
        mosaic_parts: &[Mob],
        per_runs: &str,
    ) -> fmn::Result<()> {
        let near = runs.failure("release_site");
        let far = runs.failure("far_side");
        let share = pct((HALVES.0 as u64, (HALVES.0 + HALVES.1) as u64));
        let mixed = Case {
            title: "marks that never mix".to_owned(),
            label: format!("two-halves model: all marks in one half; second catch {share} from one half; {per_runs}"),
            median: LAKE as f64,
            twice: (MARKED * CAUGHT / LAKE) as f64,
        };
        // the lake, shrunk into the mosaic's place: the dock's half holds
        // every mark
        let art = LakeArt::whole(stage)?;
        let (scale, centre) = (0.6, v(-3.5, -0.15));
        let half = cell(
            LAKE_C[0] - LAKE_W / 2.0,
            LAKE_C[1],
            LAKE_W,
            LAKE_H / 2.0,
            flat(Role::marked(), 0.3),
        );
        let hot = stage.arena_mut().add(half);
        let lake = stage.arena_mut().add(VMobject::new());
        stage.attach(lake, art)?;
        stage.attach(lake, hot)?;
        stage.scale(lake, scale);
        stage.move_to(lake, centre, ORIGIN);
        let to_screen = |p: Vec3| {
            v(
                centre[0] + (p[0] - LAKE_C[0]) * scale,
                centre[1] + (p[1] - LAKE_C[1]) * scale,
            )
        };
        let marks_here = plated(
            kit,
            stage,
            &format!("all {} marks: the dock's half", thousands(MARKED as u64)),
            22.0,
            Palette::ink(),
            to_screen(lake_p(0.36, 0.2)),
        )?;
        let net = |stage: &mut Stage<'_>, y: f64| -> fmn::Result<Mob> {
            Ok(stage.arena_mut().add(
                Circle::new()
                    .radius(0.42)
                    .arc_center(to_screen(lake_p(0.68, y)))
                    .style(
                        Style::default()
                            .fill(Role::second(), 0.12)
                            .stroke(Role::second(), 4.0, 1.0),
                    )
                    .build(),
            ))
        };
        let net_text = |kit: &Kit, stage: &mut Stage<'_>, y: f64| -> fmn::Result<Mob> {
            plated(
                kit,
                stage,
                &format!("{} of the second catch", HALVES.0),
                22.0,
                Role::second(),
                to_screen(lake_p(0.68, y + 0.19)),
            )
        };
        let near_net = net(stage, 0.28)?;
        let near_text = net_text(kit, stage, 0.28)?;
        let line = Spoken::say(kit, stage, "13_9")?;
        // the mosaic and its words give way to the lake
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for &m in mosaic_parts {
            stage.clear_updaters(m, true);
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
            out.push(stage.prepare(a)?);
        }
        stage.play_prepared(out)?;
        bench.relabel(kit, stage, &mixed)?;
        go(stage, &[(bench.estimate, mixed.median)], 0.5)?;
        bench.read(kit, stage, mixed.median)?;
        play!(stage;
            fade_in(stage.arena_mut(), lake, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), marks_here, ORIGIN, 1.0)?.rt(0.8),
        );
        // near the dock
        line.at(stage, "Near the dock")?;
        play!(stage;
            fade_in(stage.arena_mut(), near_net, v(0.0, 0.4), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), near_text, ORIGIN, 1.0)?.rt(0.6),
        );
        go(stage, &[(bench.estimate, near.median())], 1.1)?;
        kit.mark(stage, "tick:needle");
        bench.read(kit, stage, near.median())?;
        // the far side
        line.at(stage, "on the far side")?;
        let far_net = net(stage, 0.78)?;
        let far_text = net_text(kit, stage, 0.78)?;
        play!(stage;
            fade_out(stage.arena_mut(), near_net, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), near_text, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), far_net, v(0.0, -0.4), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), far_text, ORIGIN, 1.0)?.rt(0.6),
        );
        go(stage, &[(bench.estimate, far.median())], 1.1)?;
        kit.mark(stage, "tick:needle");
        bench.read(kit, stage, far.median())?;
        kit.hold(stage)?;
        // Feller's caveat, silent
        let caveat = lower_third(
            kit,
            stage,
            "Feller (1950, p. 37):",
            "“(In practice this assumption excludes situations where the two catches are made at one locality \
             and within a short time.)”",
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), bench.label, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), caveat, v(0.0, 0.2), 1.0)?.rt(0.8),
        );
        kit.music_breath(stage, 2.5)
    }
}

#[cfg(test)]
mod tests {
    //! Narration truth: every verdict chapter 13 speaks, on the runs it draws.
    use super::*;
    use crate::chapters::ring_toss::beats_are_spoken;

    #[test]
    fn every_verdict_holds_on_the_runs() {
        let v = runs::verify2();
        let m = |key: &str| v.failure(key).median();
        let truth = LAKE as f64;
        assert_eq!(m("ideal"), truth);
        // 13_3: "Too small", "around seventy-seven fifty"
        assert!(m("bold_fish") < truth);
        assert_eq!(reading(m("bold_fish")), "≈ 7,750");
        // 13_4: love the bait "do the same"; avoid it "push the estimate up"
        assert!(m("trap_happy") < truth && m("trap_shy") > truth);
        // 13_5: "That pushes it up too"
        assert!(m("marks_lost_20pct") > truth);
        // 13_6: "you count the lake at the second catch": within 2% of it
        let grown = (LAKE + NEWCOMERS) as f64;
        assert!((m("newcomers_2000") - grown).abs() / grown < 0.02);
        // 13_8: "The estimate doesn't move", "not the seven thousand left"
        assert_eq!(m("deaths_30pct"), truth);
        assert_eq!(runs::deaths_left(), (7 * LAKE as u64, 10));
        // 13_9: "sixty-two fifty", "twenty-five thousand"
        assert_eq!(reading(m("release_site")), "≈ 6,250");
        assert_eq!(reading(m("far_side")), "≈ 25,000");
        // the straightened mosaic is the median run's lake wherever the
        // middle two runs agree (every case but newcomers: 82 and 83)
        for key in [
            "bold_fish",
            "trap_happy",
            "trap_shy",
            "marks_lost_20pct",
            "deaths_30pct",
            "release_site",
        ] {
            let f = v.failure(key);
            assert_eq!(f.median(), (MARKED * CAUGHT) as f64 / f.median_caught_twice(), "{key}");
        }
    }

    #[test]
    fn the_gauge_spans_every_reading() {
        assert!((angle(0.0) - PI).abs() < 1e-12 && angle(GAUGE_MAX).abs() < 1e-12);
        assert!(angle(25_000.0) > 0.0, "25,000 is on the dial");
    }

    #[test]
    fn the_beats_wait_for_words_in_their_lines() {
        beats_are_spoken(&[
            ("13_1", "On average"),
            ("13_2", "are bold"),
            ("13_3", "Bold fish land"),
            ("13_3", "so the dark cell shrinks"),
            ("13_4", "Fish that learn to love"),
            ("13_4", "Fish that learn to avoid"),
            ("13_5", "That pushes it up"),
            ("13_6", "dilute the marks"),
            ("13_6", "so you count"),
            ("13_7", "die between"),
            ("13_8", "not the seven thousand left"),
            ("13_9", "Near the dock"),
            ("13_9", "on the far side"),
        ])
        .unwrap();
    }
}
