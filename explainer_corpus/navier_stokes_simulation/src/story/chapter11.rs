//! Chapter 11, driven by the four actual transport rules.

use explainer_kit::{Kit, Palette, hex, v};
use fmn::prelude::*;

use crate::transport::{COURANT, PULSE_WIDTH, Pulse, Rule, TRAVEL};

pub struct Chapter11 {
    pub kit: Kit,
}

const PLOT_LEFT: f64 = -6.55;
const PLOT_RIGHT: f64 = 2.70;
const PLOT_BOTTOM: f64 = -0.72;
const PLOT_HEIGHT: f64 = 2.25;
const STATS_X: f64 = 4.94;
const WINDOW: [f64; 2] = [-12.0, 16.0];

fn dye() -> Srgb {
    hex("#F49BBE")
}

fn plot_x(x: f64) -> f64 {
    PLOT_LEFT + (PLOT_RIGHT - PLOT_LEFT) * (x - WINDOW[0]) / (WINDOW[1] - WINDOW[0])
}

fn line(stage: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, weight: f64) -> fmn::Result<Mob> {
    stage.add(Line::new(a, b).style(Style::default().stroke(color, weight, 1.0)).build()?)
}

fn box_at(
    stage: &mut Stage<'_>, [x, y]: [f64; 2], [width, height]: [f64; 2],
    color: Srgb, opacity: f64, edge: f64,
) -> fmn::Result<Mob> {
    stage.add(
        Rectangle::new().width(width).height(height)
            .style(Style::default().fill(color, opacity).stroke(color, edge, 1.0))
            .build()?.moved_to(v(x, y)),
    )
}

fn multi_text(
    stage: &mut Stage<'_>, kit: &Kit, labels: &[(&str, f64, f64, Srgb)], size: f64,
) -> fmn::Result<Vec<Mob>> {
    labels.iter().map(|&(s, x, y, color)| visible_text(kit, stage, s, size, color, v(x, y))).collect()
}

struct PulseView {
    persistent: Vec<Mob>,
    values: Vec<Mob>,
}

impl PulseView {
    fn new(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Self> {
        let baseline = line(stage, v(PLOT_LEFT, PLOT_BOTTOM), v(PLOT_RIGHT, PLOT_BOTTOM), Palette::dim(), 1.6)?;
        let exact_top = PLOT_BOTTOM + PLOT_HEIGHT;
        let mut persistent = vec![baseline];
        for n in 0..8 {
            let a = plot_x(n as f64 * PULSE_WIDTH as f64 / 8.0);
            let b = plot_x((n as f64 + 0.63) * PULSE_WIDTH as f64 / 8.0);
            persistent.push(line(stage, v(a, exact_top), v(b, exact_top), Palette::ink(), 1.45)?);
        }
        for x in [plot_x(0.0), plot_x(PULSE_WIDTH as f64)] {
            for n in 0..12 {
                let a = PLOT_BOTTOM + PLOT_HEIGHT * n as f64 / 12.0;
                let b = PLOT_BOTTOM + PLOT_HEIGHT * (n as f64 + 0.63) / 12.0;
                persistent.push(line(stage, v(x, a), v(x, b), Palette::ink(), 1.45)?);
            }
        }
        persistent.push(visible_text(kit, stage, "Exact pulse in the moving window", 24.0,
                                 Palette::ink(), v(-1.92, 2.15))?);
        persistent.push(visible_text(kit, stage, "Readouts include dye outside the window", 19.0,
                                 Palette::dim(), v(-1.92, -1.61))?);
        Ok(Self { persistent, values: Vec::new() })
    }

    fn clear_values(&mut self, stage: &mut Stage<'_>) {
        stage.scene_mut().remove(&self.values);
        self.values.clear();
    }

    fn clear(self, stage: &mut Stage<'_>) {
        stage.scene_mut().remove(&self.persistent);
        stage.scene_mut().remove(&self.values);
    }

    fn show(
        &mut self, stage: &mut Stage<'_>, kit: &Kit, pulse: &Pulse, statistics: bool, escaped: bool,
    ) -> fmn::Result<()> {
        self.clear_values(stage);
        for (index, &content) in pulse.values.iter().enumerate() {
            let [a, b] = pulse.riding_bounds(index);
            if b <= WINDOW[0] || a >= WINDOW[1] {
                continue;
            }
            let x0 = plot_x(a.max(WINDOW[0]));
            let x1 = plot_x(b.min(WINDOW[1]));
            let h = PLOT_HEIGHT * content;
            let outside = b <= 0.0 || a >= PULSE_WIDTH as f64;
            let color = if content < 0.0 { Palette::red() }
                        else if escaped && outside { Palette::dim() }
                        else { dye() };
            if h.abs() > 1e-5 {
                let width = (x1 - x0) * 0.91;
                self.values.push(box_at(stage, [(x0 + x1) / 2.0, PLOT_BOTTOM + h / 2.0],
                                        [width, h.abs()], color, 0.73, 0.0)?);
            }
            self.values.push(line(stage, v(x0, PLOT_BOTTOM - 0.14), v(x0, PLOT_BOTTOM),
                                  Palette::dim(), 1.0)?);
        }
        self.values.push(visible_text(kit, stage, &format!("Step {} of {}", pulse.step, pulse.step_count()),
                                  23.0, Palette::dim(), v(-1.92, -2.21))?);
        if statistics {
            let m = pulse.measure();
            let metrics = [
                (format!("Peak  {:.2}", m.peak), 1.33, dye()),
                (format!("Lowest  {:.2}", m.minimum), 0.50,
                 if m.minimum < 0.0 { Palette::red() } else { Palette::dim() }),
                (format!("Dye total  {:.2}", m.total), -0.33, Palette::ink()),
                (format!("Error  {:.2}", m.error), -1.16, Palette::dim()),
            ];
            for (text, y, color) in metrics {
                self.values.push(visible_text(kit, stage, &text.replace('-', "−"), 26.0, color, v(STATS_X, y))?);
            }
        }
        Ok(())
    }

    fn replay(
        &mut self, stage: &mut Stage<'_>, kit: &Kit, rule: Rule, snapshot_time: f64,
    ) -> fmn::Result<Pulse> {
        let mut pulse = Pulse::new(0);
        self.show(stage, kit, &pulse, true, false)?;
        stage.wait(snapshot_time)?;
        for _ in 0..pulse.step_count() {
            pulse.advance(rule, false);
            self.show(stage, kit, &pulse, true, false)?;
            stage.wait(snapshot_time)?;
        }
        Ok(pulse)
    }
}

fn predict(stage: &mut Stage<'_>, kit: &Kit, question: &str, options: &str) -> fmn::Result<()> {
    let card = [
        box_at(stage, [-0.85, 0.51], [8.9, 2.42], hex("#182433"), 1.0, 1.4)?,
        visible_text(kit, stage, question, 38.0, Palette::ink(), v(-0.85, 1.08))?,
        visible_text(kit, stage, options, 25.0, Palette::dim(), v(-0.85, 0.31))?,
    ];
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    for second in (1..=5).rev() {
        let clock = visible_text(kit, stage, &second.to_string(), 34.0, Palette::hero(), v(-0.85, -0.19))?;
        stage.wait(1.0)?;
        stage.scene_mut().remove(&[clock]);
    }
    stage.scene_mut().remove(&card);
    Ok(())
}

fn end_state(rule: Rule, halves: u32, diffuse: bool) -> Pulse {
    let mut pulse = Pulse::new(halves);
    for _ in 0..pulse.step_count() {
        pulse.advance(rule, diffuse);
    }
    pulse
}

fn refinement_chart(stage: &mut Stage<'_>, k: &Kit, diffusion: bool) -> fmn::Result<Vec<Mob>> {
    let mut labels = vec![visible_text(k, stage, "Increase in variance", 37.0, Palette::ink(), v(-0.18, 2.38))?];
    let mut bars = Vec::new();
    const BAR_X: f64 = -2.8;
    const SCALE: f64 = 0.403;
    for halves in 0..3 {
        let y = 1.18 - f64::from(halves) * 1.29;
        let initial = Pulse::new(halves).measure().variance.unwrap();
        let numerical = end_state(Rule::Blend, halves, false).measure().variance.unwrap() - initial;
        let total = end_state(Rule::Blend, halves, diffusion).measure().variance.unwrap() - initial;
        let physical = total - numerical;
        labels.push(visible_text(k, stage, ["Cell width 1", "Cell width ½", "Cell width ¼"][halves as usize],
                           27.0, Palette::ink(), v(-4.50, y))?);
        if diffusion {
            let width = physical * SCALE;
            bars.push(box_at(stage, [BAR_X + width / 2.0, y], [width, 0.62], dye(), 0.80, 0.0)?);
            labels.push(visible_text(k, stage, &format!("{physical:.0}"), 26.0, Palette::ink(),
                               v(BAR_X + width / 2.0, y))?);
        }
        let x = BAR_X + physical * SCALE;
        let width = numerical * SCALE;
        bars.push(box_at(stage, [x + width / 2.0, y], [width, 0.62], Palette::faint(), 1.0, 0.0)?);
        labels.push(visible_text(k, stage, &format!("{numerical:.0}"), 26.0, Palette::ink(), v(x + width / 2.0, y))?);
        if diffusion {
            labels.push(visible_text(k, stage, &format!("= {total:.0}"), 27.0, Palette::ink(),
                               v(BAR_X + total * SCALE + 0.55, y))?);
        }
    }
    labels.push(visible_text(k, stage, "From the method", 23.0, Palette::dim(), v(1.50, -2.82))?);
    if diffusion {
        labels.push(visible_text(k, stage, "From physical diffusion", 23.0, dye(), v(-2.17, -2.82))?);
    }
    labels.push(visible_text(k, stage, "Same distance, measured in coarse cell widths squared", 21.0,
                       Palette::dim(), v(-0.18, -3.46))?);
    bars.extend(labels);
    Ok(bars)
}

fn signed_spreading(pulse: &Pulse) -> f64 {
    let mass = pulse.values.iter().sum::<f64>();
    let positions = (0..pulse.values.len()).map(|i| {
        let [a, b] = pulse.riding_bounds(i);
        (a + b) / 2.0
    });
    let center = positions.clone().zip(&pulse.values).map(|(x, q)| x * q).sum::<f64>() / mass;
    let variance = positions.zip(&pulse.values).map(|(x, q)| (x - center).powi(2) * q).sum::<f64>() / mass;
    variance - Pulse::new(0).measure().variance.unwrap()
}

// Kit text is registered but detached until the scene admits it.
fn visible_text(
    kit: &Kit, stage: &mut Stage<'_>, words: &str, size: f64, color: Srgb, at: Vec3,
) -> fmn::Result<Mob> {
    let text = kit.text(stage, words, size, color, at)?;
    stage.scene_mut().add(&[text])?;
    Ok(text)
}

impl SceneConstruct for Chapter11 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = &self.kit;
        let ink = Palette::ink();
        let dim = Palette::dim();
        k.say(stage, "11_1")?;
        visible_text(k, stage, "A stable lie", 46.0, ink, v(0.0, 3.34))?;
        let steps = multi_text(stage, k, &[
            ("Carry", -4.50, 1.05, Palette::blue()), ("Smooth", -1.50, 1.05, dim),
            ("Forces", 1.50, 1.05, dim), ("Correct", 4.50, 1.05, dim),
        ], 34.0)?;
        k.say(stage, "11_2")?;
        stage.scene_mut().remove(&steps);
        let mut view = PulseView::new(stage, k)?;
        let mut pulse = Pulse::new(0);
        view.show(stage, k, &pulse, false, false)?;
        let wind = visible_text(k, stage, &format!("Steady wind  →    {} cells of dye", PULSE_WIDTH), 23.0,
                          Palette::blue(), v(-1.92, 2.72))?;
        k.say(stage, "11_3")?;
        stage.wait(0.48)?;
        k.say(stage, "11_4")?;
        let lookup = visible_text(k, stage, &format!("Look back {:.1} cell", COURANT), 26.0,
                            Palette::gold(), v(STATS_X, 0.88))?;
        k.say(stage, "11_5")?;
        k.until_sentence(stage, "11_5", 2)?;
        stage.scene_mut().remove(&[lookup]);
        pulse.advance(Rule::Blend, false);
        view.show(stage, k, &pulse, false, false)?;
        let weights = visible_text(k, stage, "½ from each neighbor", 25.0, Palette::gold(), v(STATS_X, 0.88))?;
        k.say(stage, "11_6")?;
        stage.scene_mut().remove(&[weights]);
        view.clear_values(stage);
        let counter = visible_text(k, stage, &format!("{TRAVEL} cell widths   ·   {} steps", pulse.step_count()),
                             24.0, dim, v(-1.92, -2.21))?;
        predict(stage, k, "How tall is the peak now?", "About 1      About ¾      Less than ½")?;
        stage.scene_mut().remove(&[counter]);
        k.say(stage, "11_7")?;
        let blend = view.replay(stage, k, Rule::Blend, 0.049)?;
        k.say(stage, "11_8")?;
        let stable = visible_text(k, stage, "In range, with the total preserved", 24.0, Palette::green(), v(-1.92, -2.90))?;
        k.say(stage, "11_9")?;
        stage.scene_mut().remove(&[stable]);
        view.show(stage, k, &blend, true, true)?;
        let spreading = visible_text(k, stage, "Numerical diffusion", 30.0, Palette::gold(), v(-1.92, -2.90))?;
        k.say(stage, "11_10")?;
        stage.scene_mut().remove(&[wind, spreading]);
        view.clear(stage);
        let scale = visible_text(k, stage, "Halve the cells and the step together", 33.0, ink, v(0.0, 0.46))?;
        k.say(stage, "11_11")?;
        stage.scene_mut().remove(&[scale]);
        let numerical = refinement_chart(stage, k, false)?;
        k.say(stage, "11_12")?;
        stage.wait(0.58)?;
        k.say(stage, "11_13")?;
        stage.scene_mut().remove(&numerical);
        let physical = refinement_chart(stage, k, true)?;
        k.say(stage, "11_14")?;
        stage.wait(0.58)?;
        k.say(stage, "11_15")?;
        stage.scene_mut().remove(&physical);
        let mut analogy = multi_text(stage, k, &[
            ("Air's viscosity", -3.40, 1.36, Palette::purple()),
            ("Smoothing from the carry step", 3.25, 1.36, dim),
            ("In the fluid law", -3.40, 0.22, ink),
            ("Depends on cells and steps", 3.25, 0.22, ink),
        ], 26.0)?;
        k.say(stage, "11_16")?;
        analogy.push(visible_text(k, stage, "Small swirls can disappear", 36.0, Palette::gold(), v(0.0, -1.46))?);
        k.say(stage, "11_17")?;
        stage.scene_mut().remove(&analogy);
        let mut view = PulseView::new(stage, k)?;
        let mut curve = Pulse::new(0);
        view.show(stage, k, &curve, false, false)?;
        let curve_label = visible_text(k, stage, "Cubic interpolation", 28.0, Palette::blue(), v(STATS_X, 2.15))?;
        k.say(stage, "11_18")?;
        predict(stage, k, "Can a cell hold less than no dye?", "Yes               No")?;
        k.say(stage, "11_19")?;
        curve.advance(Rule::Curve, false);
        view.show(stage, k, &curve, true, false)?;
        let minimum = (16.0 * curve.measure().minimum).round() as i64;
        let first_step = visible_text(k, stage, &format!("First undershoot   {minimum}/16").replace('-', "−"),
                                25.0, Palette::red(), v(-1.92, -2.90))?;
        k.say(stage, "11_20")?;
        stage.scene_mut().remove(&[first_step, curve_label]);
        let curve = view.replay(stage, k, Rule::Curve, 0.084)?;
        let signed = signed_spreading(&curve).abs();
        let invalid = visible_text(k, stage, &format!("Signed spreading  {signed:.0}   ·   Negative dye invalidates width"),
                             21.0, Palette::red(), v(-0.85, -2.91))?;
        k.say(stage, "11_21")?;
        let invented = visible_text(k, stage, "A wiggle invented by the rule", 25.0, Palette::gold(), v(-1.92, 2.74))?;
        k.say(stage, "11_23")?;
        k.until_sentence(stage, "11_23", 2)?;
        stage.scene_mut().remove(&[invalid, invented]);
        let clamp = view.replay(stage, k, Rule::ClampedCurve, 0.065)?;
        k.say(stage, "11_24")?;
        let lost = 100.0 * (1.0 - clamp.measure().total / PULSE_WIDTH as f64);
        let loss = visible_text(k, stage, &format!("{lost:.2}% of the dye disappeared"), 25.0,
                          Palette::red(), v(-1.92, -2.90))?;
        k.say(stage, "11_25")?;
        stage.scene_mut().remove(&[loss]);
        view.clear(stage);
        let left = box_at(stage, [-2.92, 0.44], [2.0, 1.60], Palette::faint(), 0.34, 1.6)?;
        let right = box_at(stage, [-0.92, 0.44], [2.0, 1.60], Palette::blue(), 0.20, 1.6)?;
        let transfer = stage.add(
            Arrow::new(v(-2.26, 1.04), v(-1.58, 1.04)).buff(0.0).thickness(1.6)
                .style(Style::default().fill(dye(), 1.0).stroke(dye(), 1.0, 1.0))
                .build()?,
        )?;
        let mut face = multi_text(stage, k, &[
            ("− F", -2.92, 0.41, dye()), ("+ F", -0.92, 0.41, dye()),
            ("Same transfer across the shared face", -1.92, -1.06, ink),
        ], 26.0)?;
        face.extend([left, right, transfer]);
        k.say(stage, "11_26")?;
        stage.scene_mut().remove(&face);
        let mut view = PulseView::new(stage, k)?;
        let limited = view.replay(stage, k, Rule::LimitedFlux, 0.084)?;
        k.say(stage, "11_27")?;
        view.clear(stage);
        let computed = [blend, curve, clamp, limited];
        let rows = ["Blend", "Cubic", "Clamped cubic", "Limited flux"];
        let columns = [-1.67, 0.42, 2.51, 4.60];
        let mut scoreboard = Vec::new();
        for (ix, label) in ["Peak", "Lowest", "Dye total", "Error"].into_iter().enumerate() {
            scoreboard.push(visible_text(k, stage, label, 24.0, dim, v(columns[ix], 1.94))?);
        }
        for (i, pulse) in computed.iter().enumerate() {
            let y = 1.04 - i as f64 * 0.88;
            let m = pulse.measure();
            scoreboard.push(visible_text(k, stage, rows[i], 27.0, ink, v(-4.48, y))?);
            for (col, value) in [m.peak, m.minimum, m.total, m.error].into_iter().enumerate() {
                let color = if value < 0.0 || (col == 2 && value < 3.99) { Palette::red() } else { ink };
                scoreboard.push(visible_text(k, stage, &format!("{value:.2}").replace('-', "−"), 26.0,
                                       color, v(columns[col], y))?);
            }
        }
        k.say(stage, "11_28")?;
        scoreboard.push(box_at(stage, [0.24, -1.60], [11.56, 0.65], Palette::green(), 0.0, 2.0)?);
        k.say(stage, "11_29")?;
        scoreboard.push(visible_text(k, stage, "Which errors shrink when we refine?", 28.0,
                               Palette::hero(), v(0.0, -3.1))?);
        k.say(stage, "11_30")?;
        stage.scene_mut().remove(&scoreboard);
        visible_text(k, stage, "An added force changes the model", 39.0, ink, v(0.0, 0.58))?;
        visible_text(k, stage, "Declare any force used to restore spin", 26.0, dim, v(0.0, -0.40))?;
        k.finish(stage)
    }
}
