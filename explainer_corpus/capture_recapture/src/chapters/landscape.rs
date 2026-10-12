//! Chapter 06, the landscape (the camera route): every lake size and every
//! caught-twice count at once, P_N(K = k) as lit bars. The field is laid out
//! so that its k = 3 row, seen side-on, is exactly chapter 05's score curve
//! (`chapters/curve.rs`: each size at the curve's x, heights on its scale,
//! the row in the camera's focal plane), so the chapter opens on 05's last
//! picture, rises into the landscape (06_1), cuts across at one lake size
//! (06_2–06_3: the chances, adding to one), and swings back to cut along at
//! our three (06_4), ending on the 2D curve chapter 07 opens with.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::kit::CameraShot;
use explainer_kit::{Frame2, Kit, Palette, Timed, hex, play, v};
use fmn::core::color::interpolate_color;
use fmn::library::Prism;
use fmn::prelude::*;

use super::curve;
use super::draw::{Role, line_style};
use super::lake::{fade_in_all, fade_out_all, plated};
use crate::facts::TOY;
use crate::law::Law;

/// The counts on the field's depth axis: 0 … caught (10).
const COUNTS: u64 = TOY.1;
/// The depth between neighbouring counts' rows.
const ROW_DY: f64 = 0.45;
/// A bar's depth.
const BAR_DEPTH: f64 = 0.3;
/// The side view (05's curve, 07's first frame) and the three-quarter view.
pub const SIDE: CameraShot = CameraShot {
    theta: 0.0,
    phi: FRAC_PI_2,
};
/// A camera view: its angles, how much more of the world than the side
/// view's frame it shows (zoom out above 1), and the point it looks at.
#[derive(Clone, Copy, Debug)]
struct View {
    theta: f64,
    phi: f64,
    zoom: f64,
    centre: Vec3,
}

/// Above and in front of the field, a little from the small-lake side, far
/// enough back that the whole field fits.
const THREE_QUARTER: View = View {
    theta: 0.3,
    phi: 1.12,
    zoom: 1.48,
    centre: [0.6, 0.9, -1.0],
};
/// The fixed-size slice of 06_2–06_3.
const SLICE: u64 = TOY.0 * TOY.1 / TOY.2;

fn field_legend() -> [String; 3] {
    let sizes = curve::sizes();
    [
        format!("left to right\nlake size: {} to {}", sizes.start(), sizes.end()),
        format!("front to back\ncaught twice: 0 to {COUNTS}"),
        "height\nchance".to_owned(),
    ]
}

fn legend_labels(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    field_legend()
        .iter()
        .zip([-4.4, 0.0, 4.4])
        .map(|(text, x)| plated(kit, stage, text, 40.0, Palette::ink(), v(x, 3.2)))
        .collect()
}

/// Where count `k`'s row lies (y): our caught-twice count's row in the
/// focal plane (y = 0), smaller counts nearer the side camera.
fn row_y(k: u64) -> f64 {
    (k as f64 - TOY.2 as f64) * ROW_DY
}

/// A bar's height for chance `p`, on the score curve's scale.
fn bar_height(frame: &Frame2, p: f64) -> f64 {
    frame.p(0.0, p)[1] - frame.bottom()
}

/// The field's colour for a chance: dim to bright blue-grey by height.
fn field_colour(p: f64, top: f64) -> Srgb {
    interpolate_color(hex("#2E3644"), hex("#C3CAD5"), (p / top).clamp(0.0, 1.0).sqrt())
}

/// The field: one bar per (size, count) with a chance worth drawing, as
/// (size, count, chance, bar).
struct Field {
    bars: Vec<(u64, u64, f64, Mob)>,
}

impl Field {
    fn build(stage: &mut Stage<'_>, frame: &Frame2) -> fmn::Result<Self> {
        let w = curve::BAR * frame.width / (frame.x.1 - frame.x.0);
        let top = curve::sizes()
            .flat_map(|n| {
                let law = Law::new(n, TOY.0, TOY.1);
                (0..=COUNTS).map(move |k| law.at(k))
            })
            .fold(0.0, f64::max);
        let mut bars = Vec::new();
        for n in curve::sizes() {
            let law = Law::new(n, TOY.0, TOY.1);
            for k in 0..=COUNTS {
                let p = law.at(k);
                if p < 1e-4 {
                    continue;
                }
                let h = bar_height(frame, p);
                let c = if k == TOY.2 {
                    Role::score()
                } else {
                    field_colour(p, top)
                };
                let b = stage.arena_mut().add(Prism::new(w, BAR_DEPTH, h).color(c).build());
                let x = frame.p(n as f64, 0.0)[0];
                stage.shift(b, [x, row_y(k), frame.bottom() + h / 2.0]);
                bars.push((n, k, p, b));
            }
        }
        Ok(Self { bars })
    }

    fn row(&self, k: u64) -> Vec<Mob> {
        self.bars.iter().filter(|b| b.1 == k).map(|b| b.3).collect()
    }

    fn not_row(&self, k: u64) -> Vec<Mob> {
        self.bars.iter().filter(|b| b.1 != k).map(|b| b.3).collect()
    }

    fn not_size(&self, n: u64) -> Vec<Mob> {
        self.bars.iter().filter(|b| b.0 != n).map(|b| b.3).collect()
    }
}

/// Prepared opacity changes for `mobs`.
fn opacity(stage: &mut Stage<'_>, mobs: &[Mob], to: f64, secs: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    mobs.iter()
        .map(|&m| {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(secs),
                    ..AnimateArgs::default()
                })?
                .set_opacity(to)?;
            stage.prepare(a)
        })
        .collect()
}

/// A camera move to `view` over `secs`, prepared: from the shot's own
/// configuration, its frame scaled by the zoom (width and height together,
/// so the aspect never changes) and centred on the view's point. `None` is
/// the side view exactly as the chapter began (05's last frame, 07's first).
fn turn(kit: &Kit, stage: &mut Stage<'_>, view: Option<View>, secs: f64) -> fmn::Result<Box<dyn Animation>> {
    let fail = |m: &'static str| fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(m));
    let camera = |e| fmn::Error::Scene(fmn::scene::SceneError::from(e));
    let (rig, start) = kit
        .camera()
        .ok_or_else(|| fail("06 runs on the camera route (Kit::request_camera)"))?;
    let mut target = start.clone();
    if let Some(v) = view {
        let (w, h) = (start.frame.width(), start.frame.height());
        target
            .frame
            .set_euler_angles(Some(v.theta), Some(v.phi), None)
            .map_err(camera)?;
        target.frame.set_shape([w * v.zoom, h * v.zoom]).map_err(camera)?;
        target.frame.set_center(v.centre).map_err(camera)?;
    }
    let t = rig.animate_to(stage.scene_mut(), &target)?.rt(secs);
    stage.prepare(t)
}

/// The prediction pause (as `props::predict_pause`) with its "?" pinned to
/// the screen: let the question finish, mark it for the score, hold.
fn predict_hud(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text_hud(stage, "?", 90.0, Palette::hero(), at)?;
    fade_in_all(stage, &[q], 0.3)?;
    let fps = stage.scene().time().fps();
    explainer_kit::narration::wait_frames(
        stage,
        explainer_kit::narration::frames_covering(explainer_kit::props::PREDICT_HOLD_S, fps),
    )?;
    Ok(q)
}

/// Pin `m` to the screen (a HUD label while the camera moves).
fn hud(stage: &mut Stage<'_>, m: Mob) -> Mob {
    stage.set_fixed_in_frame(m, 1.0, true);
    m
}

pub struct Landscape {
    pub kit: Kit,
}

impl Landscape {
    /// The chapter, asking for the camera route at the side view.
    pub fn new(kit: Kit) -> Self {
        kit.request_camera(SIDE);
        Self { kit }
    }
}

impl SceneConstruct for Landscape {
    fn name(&self) -> &str {
        "06_the_landscape"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let frame = curve::frame();
        let field = Field::build(stage, &frame)?;
        // 05's last picture, pinned to the screen: the curve and its axis
        let axis = curve::axis(&kit, stage, &frame)?;
        let flat_bars: Vec<Mob> = curve::bars(&frame)?
            .into_iter()
            .map(|b| stage.arena_mut().add(b))
            .collect();
        let flat_group = stage.arena_mut().add(VMobject::new());
        for &b in &flat_bars {
            stage.attach(flat_group, b)?;
        }
        hud(stage, axis);
        hud(stage, flat_group);
        stage.add_to_scene(axis)?;
        stage.add_to_scene(flat_group)?;
        // the k = 3 row stands exactly behind the flat curve
        let ours = field.row(TOY.2);
        let others = field.not_row(TOY.2);
        for &b in &ours {
            stage.add_to_scene(b)?;
        }

        // 06_1: every lake size and every count at once
        let labels = legend_labels(&kit, stage)?;
        let legend = stage.arena_mut().add(VMobject::new());
        for &label in &labels {
            stage.attach(legend, label)?;
        }
        hud(stage, legend);
        kit.say(stage, "06_1")?;
        // the flat curve gives way to its row in the landscape
        fade_out_all(stage, &[axis, flat_group], 0.6)?;
        for &b in &others {
            stage.add_to_scene(b)?;
            stage.set_opacity(b, 0.0, true);
        }
        let mut rise = opacity(stage, &others, 1.0, 2.0)?;
        rise.push(turn(&kit, stage, Some(THREE_QUARTER), 2.4)?);
        stage.play_prepared(rise)?;
        fade_in_all(stage, &[legend], 0.4)?;
        kit.hold(stage)?;
        // a beat to take in the landscape before the first cut (R3 maya-01)
        stage.wait(1.0)?;

        // 06_2: cut across at one lake size
        let slice_x = frame.p(SLICE as f64, 0.0)[0];
        let plane = stage.arena_mut().add(
            Prism::new(0.02, row_y(COUNTS) - row_y(0) + 0.8, frame.height + 0.4)
                .color(Palette::ink())
                .opacity(0.16)
                .build(),
        );
        stage.shift(
            plane,
            [
                slice_x,
                (row_y(COUNTS) + row_y(0)) / 2.0,
                frame.bottom() + frame.height / 2.0,
            ],
        );
        let rest = field.not_size(SLICE);
        kit.say(stage, "06_2")?;
        // every other lake size goes, the camera holds still, and the one
        // lake's row stands alone in its plane (R3 maya-01)
        stage.add_to_scene(plane)?;
        stage.set_opacity(plane, 0.0, true);
        let mut cut = opacity(stage, &rest, 0.0, 1.4)?;
        cut.extend(opacity(stage, &[plane], 0.16, 1.4)?);
        stage.play_prepared(cut)?;
        let q = predict_hud(&kit, stage, v(5.9, 0.6))?;

        // 06_3: that slice is the chances of every count for that lake;
        // they add to one
        // lifted out flat beside the landscape: 05_4's chart of one lake's
        // catch, for this lake
        let sum: f64 = (0..=COUNTS).map(|k| Law::new(SLICE, TOY.0, TOY.1).at(k)).sum();
        let chart_f = Frame2 {
            width: 6.0,
            height: 2.6,
            y: (0.0, 0.45),
            ..curve::catch_frame(v(3.5, 0.9))
        };
        let chart = stage.arena_mut().add(curve::catch_bars(&chart_f, SLICE)?);
        let chart_axis = curve::catch_axis(&kit, stage, &chart_f)?;
        hud(stage, chart);
        hud(stage, chart_axis);
        let sigma = plated(
            &kit,
            stage,
            &format!("lake of {SLICE}: the chances add to {}", sum.round()),
            28.0,
            Role::chance(),
            v(chart_f.center[0], chart_f.top() + 0.45),
        )?;
        hud(stage, sigma);
        let question1 = plated(
            &kit,
            stage,
            "what catch will this lake give?",
            26.0,
            Palette::ink(),
            v(-3.6, 2.0),
        )?;
        hud(stage, question1);
        kit.say(stage, "06_3")?;
        fade_out_all(stage, &[q], 0.3)?;
        play!(stage;
            fade_in(stage.arena_mut(), chart, v(1.2, 0.0), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), chart_axis, v(1.2, 0.0), 1.0)?.rt(0.8),
        );
        fade_in_all(stage, &[sigma], 0.5)?;
        kit.hold(stage)?;
        // time to read it, the camera still
        stage.wait(1.5)?;

        // 06_4: cut along at our three: the score curve. The crest is the
        // ridge over the tops; then back to the side view and the flat curve
        let crest = crest_ridge(stage, &frame)?;
        kit.say(stage, "06_4")?;
        fade_out_all(stage, &[sigma, plane, chart, chart_axis], 0.4)?;
        let mut back = opacity(stage, &rest, 1.0, 1.0)?;
        back.push(turn(&kit, stage, Some(THREE_QUARTER), 2.2)?);
        stage.play_prepared(back)?;
        play!(stage; fmn::animation::show_creation(crest).rt(1.4));
        // the other counts fall away; our row stays, side-on
        let mut gone = others.clone();
        gone.push(crest);
        let mut cut = opacity(stage, &gone, 0.0, 1.6)?;
        cut.push(turn(&kit, stage, None, 2.6)?);
        stage.play_prepared(cut)?;
        let first = *curve::sizes().start();
        let to60: f64 = curve::sizes().map(curve::score).sum();
        let (m, n, k) = TOY;
        let all = (m * n) as f64 / (k * (k - 1)) as f64;
        let sums = plated(
            &kit,
            stage,
            &format!("{first} to {}: {to60:.2}   ·   all sizes: {all:.2}", curve::SIZE_MAX),
            26.0,
            Role::score(),
            v(3.2, 2.55),
        )?;
        hud(stage, sums);
        let question2 = plated(
            &kit,
            stage,
            "which lake gave our catch?",
            26.0,
            Role::score(),
            v(3.2, 1.95),
        )?;
        hud(stage, question2);
        fade_in_all(stage, &[sums], 0.5)?;
        // both cuts, both questions: the fixed-size slice's question returns
        fade_in_all(stage, &[question1, question2], 0.6)?;
        // the flat curve again, exactly where 07 begins
        let axis = curve::axis(&kit, stage, &frame)?;
        hud(stage, axis);
        fade_in_all(stage, &[axis], 0.5)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[legend, sums, question1, question2], 0.6)?;
        let flat_bars: Vec<Mob> = curve::bars(&frame)?
            .into_iter()
            .map(|b| stage.arena_mut().add(b))
            .collect();
        for &b in &flat_bars {
            hud(stage, b);
        }
        fade_in_all(stage, &flat_bars, 0.4)?;
        let mut last = Vec::new();
        for &b in &ours {
            last.push(b);
        }
        fade_out_all(stage, &last, 0.2)?;
        kit.finish(stage)
    }
}

/// The crest: for each lake size, the count with the top chance (the mode,
/// near Mn/N), joined over the bar tops as one 3D line.
fn crest_ridge(stage: &mut Stage<'_>, frame: &Frame2) -> fmn::Result<Mob> {
    let pts: Vec<Vec3> = curve::sizes()
        .map(|n| {
            let law = Law::new(n, TOY.0, TOY.1);
            let (k, p) = (0..=COUNTS)
                .map(|k| (k, law.at(k)))
                .fold((0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
            [
                frame.p(n as f64, 0.0)[0],
                row_y(k),
                frame.bottom() + bar_height(frame, p) + 0.05,
            ]
        })
        .collect();
    Ok(stage
        .arena_mut()
        .add(polyline(pts).style(line_style(Palette::ink(), 4.0)).build()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn our_row_is_the_score_curve_and_each_lake_s_slice_adds_to_one() {
        let frame = curve::frame();
        for n in curve::sizes() {
            let law = Law::new(n, TOY.0, TOY.1);
            // the k = 3 bar is the curve's bar: same height on the same scale
            let h = bar_height(&frame, law.at(TOY.2));
            let top = frame.p(n as f64, curve::score(n))[1];
            assert!((frame.bottom() + h - top).abs() < 1e-9);
            let total: f64 = (0..=COUNTS).map(|k| law.at(k)).sum();
            assert!((total - 1.0).abs() < 1e-12, "N {n}: {total}");
        }
        // our row lies in the focal plane: seen side-on it is the 2D curve
        assert_eq!(row_y(TOY.2), 0.0);
        // the whole field fits under the frame's top
        let tallest = curve::sizes()
            .flat_map(|n| (0..=COUNTS).map(move |k| Law::new(n, TOY.0, TOY.1).at(k)))
            .fold(0.0, f64::max);
        assert!(frame.bottom() + bar_height(&frame, tallest) < 3.4, "{tallest}");
    }

    #[test]
    fn the_chosen_slice_cuts_the_peak_of_the_score_curve() {
        assert!(curve::sizes().contains(&SLICE));
        let peak = curve::score(SLICE);
        for n in curve::sizes() {
            assert!(peak >= curve::score(n), "slice {SLICE} misses the highest score at {n}");
        }
    }

    #[test]
    fn the_axis_headings_fit_without_touching_each_other() {
        struct Probe;
        impl SceneConstruct for Probe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                let kit = Kit::new()?;
                let labels = legend_labels(&kit, stage)?;
                for &label in &labels {
                    assert!(stage.get_top(label)[1] < 3.85, "top: {:?}", stage.get_top(label));
                    assert!(stage.get_left(label)[0] > -6.96, "left: {:?}", stage.get_left(label));
                    assert!(stage.get_right(label)[0] < 6.96, "right: {:?}", stage.get_right(label));
                }
                for pair in labels.windows(2) {
                    assert!(stage.get_right(pair[0])[0] + 0.1 < stage.get_left(pair[1])[0]);
                }
                Ok(())
            }
        }
        fmn::run_scene(
            &mut Probe,
            fmn::scene::RuntimeConfig::default(),
            7,
            &mut fmn::scene::NullSceneSink,
        )
        .expect("actual landscape axis labels");
    }
}
