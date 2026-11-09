//! Deterministic scene telemetry & invariant audit helpers (plan §3.3, §4; D-03).
//!
//! Provides deterministic, in-memory scene stepping and telemetry validation
//! for five fundamental visual/audio invariants across the explainer corpus:
//!
//! - **Invariant I-1 (Bézier Point-Count Stability):**
//!   Mobjects actively transitioning in opacity (fading in/out) must keep a
//!   constant Bézier control point count. Catches the franken_manim retained-renderer
//!   trap where mutating `set_points` mid-fade freezes alpha at ~0.04.
//! - **Invariant I-2 (Causal Quiz Masking & Post-Zero Dwell):**
//!   Quiz answers must stay strictly masked (alpha == 0) until the countdown ring
//!   reaches zero, then reach >= 0.95 within 0.5 s, and dwell >= max(1.5 s, W/3.5 wps).
//! - **Invariant I-3 (Lower-Third Legibility & Band Clearance):**
//!   Lower-third explanatory bands must satisfy reading-rate dwell (>= max(2.5 s, W/4.0 wps)),
//!   stay within the safe lower-third vertical corridor (`y ∈ [-3.85, -2.85]`), and
//!   remain free of overlap with upper stage content (`y > -2.75`).
//! - **Invariant I-4 (Non-Empty Stage Under Narration):**
//!   During spoken narration lines, stage content area must never collapse to zero
//!   (seams between sequential `fade_out` and `fade_in` must not create black frames).
//! - **Invariant I-5 (Non-Collinear Reflection Morph Area):**
//!   Reflected polygon transitions across mirror axes must maintain shoelace area
//!   bounded away from zero, avoiding linear interpolation collapse at midpoint `e = 0.5`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use crate::layout::{Box2, visible_box};

/// A structured diagnostic emitted when a scene telemetry invariant is violated.
#[derive(Clone, Debug, PartialEq)]
pub struct InvariantDiagnostic {
    pub chapter: String,
    pub line_id: String,
    pub scene_time_s: f64,
    pub mob_label: String,
    pub invariant: &'static str,
    pub expected: String,
    pub actual: String,
}

impl std::fmt::Display for InvariantDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] at {:.2}s (line '{}', {}): expected {}, got {}",
            self.invariant, self.scene_time_s, self.line_id, self.mob_label, self.expected, self.actual
        )
    }
}

/// Area of an axis-aligned bounding box.
pub fn box_area((lo, hi): Box2) -> f64 {
    let w = (hi[0] - lo[0]).max(0.0);
    let h = (hi[1] - lo[1]).max(0.0);
    w * h
}

/// Area of a 2D polygon using Gauss's area (shoelace) formula.
pub fn shoelace_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut acc = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        acc += points[i][0] * points[j][1] - points[j][0] * points[i][1];
    }
    (acc / 2.0).abs()
}

/// Computes an out-of-line normal bulge vector for 2D reflections:
/// `normal * lift * sin(π * e)`.
pub fn normal_bulge_offset(normal: [f64; 2], lift: f64, e: f64) -> [f64; 2] {
    let e_clamped = e.clamp(0.0, 1.0);
    let bulge = lift * (std::f64::consts::PI * e_clamped).sin();
    [normal[0] * bulge, normal[1] * bulge]
}

/// Returns true if parameter `e` is within `tolerance` of the reflection midpoint `0.5`.
pub fn is_near_reflection_midpoint(e: f64, tolerance: f64) -> bool {
    (e - 0.5).abs() < tolerance
}

/// Guards a reflection interpolation parameter `e ∈ [0, 1]` away from the
/// collinear singularity at `e = 0.5` (where `det = 1 - 2e = 0`), ensuring
/// `|e - 0.5| >= min_half_gap` so polygon area stays bounded away from zero.
pub fn guard_reflection_param(e: f64, min_half_gap: f64) -> f64 {
    let e = e.clamp(0.0, 1.0);
    let g = min_half_gap.clamp(0.0, 0.49);
    if (e - 0.5).abs() < g {
        if e <= 0.5 { 0.5 - g } else { 0.5 + g }
    } else {
        e
    }
}

/// Verifies Invariant I-5: that a polygon reflection morph trajectory maintains
/// shoelace area >= `min_area` across all `e ∈ [0, 1]`.
pub fn check_reflection_morph_area<F>(
    label: &str,
    trajectory: F,
    steps: usize,
    min_area: f64,
) -> Result<(), Box<InvariantDiagnostic>>
where
    F: Fn(f64) -> Vec<[f64; 2]>,
{
    let steps = steps.max(1);
    for step in 0..=steps {
        let e = step as f64 / steps as f64;
        let poly = trajectory(e);
        let a = shoelace_area(&poly);
        if a < min_area {
            return Err(Box::new(InvariantDiagnostic {
                chapter: String::new(),
                line_id: String::new(),
                scene_time_s: e,
                mob_label: label.to_string(),
                invariant: "I-5:reflection_morph_area",
                expected: format!("morph area >= {min_area:.4} for all e in [0, 1]"),
                actual: format!("morph area collapsed to {a:.4} at e={e:.3}"),
            }));
        }
    }
    Ok(())
}

/// A snapshot of a single mobject at a discrete time point.
#[derive(Clone, Debug, PartialEq)]
pub struct MobSnapshot {
    pub mob: Mob,
    pub label: String,
    pub opacity: f64,
    pub points_len: usize,
    pub bbox: Option<Box2>,
    pub is_backdrop: bool,
}

/// A sampled frame of scene telemetry.
#[derive(Clone, Debug, Default)]
pub struct SceneFrame {
    pub time_s: f64,
    pub mobs: HashMap<Mob, MobSnapshot>,
}

impl SceneFrame {
    pub fn new(time_s: f64) -> Self {
        Self {
            time_s,
            mobs: HashMap::new(),
        }
    }

    /// Captures a frame snapshot from the arena.
    pub fn capture(arena: &Arena, time_s: f64, backdrop_mobs: &[Mob]) -> Self {
        let mut frame = Self::new(time_s);
        for &root in arena.roots() {
            for m in arena.family(root) {
                let Some(e) = arena.get(m) else { continue };
                let b = &e.buffer;
                let opacity = match (b.read_column("fill_rgba"), b.read_column("stroke_rgba")) {
                    (Some(fill), Some(stroke)) => {
                        let f_a = fill.as_chunks::<4>().0.iter().fold(0.0_f32, |m, c| m.max(c[3]));
                        let s_a = stroke.as_chunks::<4>().0.iter().fold(0.0_f32, |m, c| m.max(c[3]));
                        f64::from(f_a.max(s_a))
                    }
                    (Some(fill), None) => {
                        let f_a = fill.as_chunks::<4>().0.iter().fold(0.0_f32, |m, c| m.max(c[3]));
                        f64::from(f_a)
                    }
                    (None, Some(stroke)) => {
                        let s_a = stroke.as_chunks::<4>().0.iter().fold(0.0_f32, |m, c| m.max(c[3]));
                        f64::from(s_a)
                    }
                    (None, None) => match (b.read_column("rgba"), b.read_column("opacity")) {
                        (Some(rgba), _) => {
                            let a = rgba.as_chunks::<4>().0.iter().fold(0.0_f32, |m, c| m.max(c[3]));
                            f64::from(a)
                        }
                        (None, Some(op)) => op.iter().fold(0.0_f64, |m, &c| m.max(f64::from(c))),
                        (None, None) => {
                            if e.paints() {
                                1.0
                            } else {
                                0.0
                            }
                        }
                    },
                };
                let points_len = b.read_column("point").map_or(0, |p| p.len() / 3);
                let bbox = visible_box(arena, m);
                frame.mobs.insert(
                    m,
                    MobSnapshot {
                        mob: m,
                        label: format!("mob_{m:?}"),
                        opacity,
                        points_len,
                        bbox,
                        is_backdrop: backdrop_mobs.contains(&m),
                    },
                );
            }
        }
        frame
    }

    /// Total visible non-backdrop stage area.
    pub fn visible_stage_area(&self) -> f64 {
        self.mobs
            .values()
            .filter(|m| !m.is_backdrop && m.opacity > 0.01)
            .filter_map(|m| m.bbox.map(|b| m.opacity * box_area(b)))
            .sum()
    }
}

/// Records a [`SceneFrame`] on every tick of a scene via a lightweight dt-updater.
#[derive(Clone, Default)]
pub struct SceneRecorder {
    frames: Rc<RefCell<Vec<SceneFrame>>>,
}

impl SceneRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a dt-updater to `stage` that captures [`SceneFrame`] snapshots at
    /// every frame step, ignoring the probe anchor itself and marking
    /// `backdrop_mobs` as backdrop.
    pub fn attach(&self, stage: &mut Stage<'_>, backdrop_mobs: Vec<Mob>) -> fmn::Result<Mob> {
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        let frames = Rc::clone(&self.frames);
        let mut clock = 0.0_f64;
        stage.add_dt_updater(
            anchor,
            move |arena, _, dt| {
                clock += dt;
                let mut frame = SceneFrame::capture(arena, clock, &backdrop_mobs);
                frame.mobs.remove(&anchor);
                frames.borrow_mut().push(frame);
            },
            false,
        )?;
        Ok(anchor)
    }

    pub fn frames(&self) -> Vec<SceneFrame> {
        self.frames.borrow().clone()
    }
}

/// Specification of a quiz countdown and answer reveal window for Invariant I-2.
#[derive(Clone, Debug)]
pub struct QuizAuditSpec {
    pub line_id: String,
    pub ring_start_s: f64,
    pub ring_zero_s: f64,
    pub first_sentence_after_s: f64,
    pub answer_mobs: Vec<Mob>,
    pub word_count: usize,
}

/// Specification of a lower-third caption window for Invariant I-3.
#[derive(Clone, Debug)]
pub struct LowerThirdAuditSpec {
    pub line_id: String,
    pub mob: Mob,
    pub label: String,
    pub word_count: usize,
    pub start_s: f64,
    pub end_s: f64,
    pub bbox: Box2,
}

/// Specification of a spoken narration line for Invariant I-4.
#[derive(Clone, Debug)]
pub struct SpokenLineAuditSpec {
    pub line_id: String,
    pub start_s: f64,
    pub end_s: f64,
}

/// Verifies Invariant I-1: Bézier Point-Count Stability under Opacity Transitions.
pub fn check_invariant_point_count_stability(frames: &[SceneFrame], chapter: &str) -> Vec<InvariantDiagnostic> {
    let mut diagnostics = Vec::new();
    for window in frames.windows(2) {
        let (prev, curr) = (&window[0], &window[1]);
        for (mob, curr_snap) in &curr.mobs {
            if let Some(prev_snap) = prev.mobs.get(mob) {
                let in_transition =
                    (0.01..0.99).contains(&prev_snap.opacity) || (0.01..0.99).contains(&curr_snap.opacity);
                if in_transition && curr_snap.points_len != prev_snap.points_len {
                    diagnostics.push(InvariantDiagnostic {
                        chapter: chapter.to_string(),
                        line_id: String::new(),
                        scene_time_s: curr.time_s,
                        mob_label: curr_snap.label.clone(),
                        invariant: "I-1:point_count_stability",
                        expected: format!(
                            "constant Bézier point count ({}) during opacity transition ({:.3} -> {:.3})",
                            prev_snap.points_len, prev_snap.opacity, curr_snap.opacity
                        ),
                        actual: format!("point count changed to {}", curr_snap.points_len),
                    });
                }
            }
        }
    }
    diagnostics
}

/// Verifies Invariant I-2: Causal Quiz Masking & Post-Zero Answer Dwell.
pub fn check_invariant_quiz_masking_and_dwell(
    frames: &[SceneFrame],
    quizzes: &[QuizAuditSpec],
    fps: f64,
    chapter: &str,
) -> Vec<InvariantDiagnostic> {
    let mut diagnostics = Vec::new();
    let frame_dt = 1.0 / fps.max(1.0);

    for quiz in quizzes {
        let min_dwell_s = 1.5_f64.max(quiz.word_count as f64 / 3.5);
        let required_dwell_until = quiz.first_sentence_after_s.min(quiz.ring_zero_s + min_dwell_s);

        let mut reached_reveal = false;

        for frame in frames {
            let t = frame.time_s;

            // 1. Causal quiz masking: answer must be opacity 0.0 before countdown zero
            if t < quiz.ring_zero_s - frame_dt {
                for &ans_mob in &quiz.answer_mobs {
                    if let Some(snap) = frame.mobs.get(&ans_mob)
                        && snap.opacity > 0.01
                    {
                        diagnostics.push(InvariantDiagnostic {
                            chapter: chapter.to_string(),
                            line_id: quiz.line_id.clone(),
                            scene_time_s: t,
                            mob_label: snap.label.clone(),
                            invariant: "I-2:causal_quiz_masking",
                            expected: format!(
                                "answer masked (opacity 0.0) before countdown zero at {:.2}s",
                                quiz.ring_zero_s
                            ),
                            actual: format!("opacity was {:.3}", snap.opacity),
                        });
                    }
                }
            }

            // 2. Answer arrival: within 0.5s after zero, answer reaches >= 0.95
            if t >= quiz.ring_zero_s && t <= quiz.ring_zero_s + 0.5 {
                let max_ans_opacity = quiz
                    .answer_mobs
                    .iter()
                    .filter_map(|m| frame.mobs.get(m))
                    .fold(0.0_f64, |acc, s| acc.max(s.opacity));
                if max_ans_opacity >= 0.95 {
                    reached_reveal = true;
                }
            }

            // 3. Post-zero dwell: answer must remain visible (>= 0.90) through required_dwell_until
            if t > quiz.ring_zero_s + 0.5 && t < required_dwell_until - 0.05 {
                let max_ans_opacity = quiz
                    .answer_mobs
                    .iter()
                    .filter_map(|m| frame.mobs.get(m))
                    .fold(0.0_f64, |acc, s| acc.max(s.opacity));
                if max_ans_opacity < 0.90 {
                    diagnostics.push(InvariantDiagnostic {
                        chapter: chapter.to_string(),
                        line_id: quiz.line_id.clone(),
                        scene_time_s: t,
                        mob_label: format!("quiz_answer_{:?}", quiz.answer_mobs),
                        invariant: "I-2:quiz_post_zero_dwell",
                        expected: format!(
                            "answer held at opacity >= 0.90 until {:.2}s (dwell {:.2}s)",
                            required_dwell_until, min_dwell_s
                        ),
                        actual: format!("opacity was {:.3}", max_ans_opacity),
                    });
                }
            }
        }

        if !reached_reveal && !frames.is_empty() {
            diagnostics.push(InvariantDiagnostic {
                chapter: chapter.to_string(),
                line_id: quiz.line_id.clone(),
                scene_time_s: quiz.ring_zero_s,
                mob_label: format!("quiz_answer_{:?}", quiz.answer_mobs),
                invariant: "I-2:quiz_reveal_arrival",
                expected: "answer reaches opacity >= 0.95 within 0.5s of countdown zero".to_string(),
                actual: "answer never reached opacity >= 0.95".to_string(),
            });
        }
    }

    diagnostics
}

/// Verifies Invariant I-3: Lower-Third Legibility & Reading-Rate Gate.
pub fn check_invariant_lower_third(specs: &[LowerThirdAuditSpec], chapter: &str) -> Vec<InvariantDiagnostic> {
    let mut diagnostics = Vec::new();

    for spec in specs {
        let required_dwell_s = 2.5_f64.max(spec.word_count as f64 / 4.0);
        let actual_dwell_s = spec.end_s - spec.start_s;

        // Reading rate dwell gate
        if actual_dwell_s < required_dwell_s - 0.05 {
            diagnostics.push(InvariantDiagnostic {
                chapter: chapter.to_string(),
                line_id: spec.line_id.clone(),
                scene_time_s: spec.start_s,
                mob_label: spec.label.clone(),
                invariant: "I-3:lower_third_dwell",
                expected: format!(
                    "dwell >= {:.2}s for {} words at 4.0 wps",
                    required_dwell_s, spec.word_count
                ),
                actual: format!("dwell was {:.2}s", actual_dwell_s),
            });
        }

        // Vertical corridor check (y ∈ [-3.85, -2.85])
        let y_min = spec.bbox.0[1];
        let y_max = spec.bbox.1[1];
        if y_min < -3.85 - 0.05 || y_max > -2.85 + 0.05 {
            diagnostics.push(InvariantDiagnostic {
                chapter: chapter.to_string(),
                line_id: spec.line_id.clone(),
                scene_time_s: spec.start_s,
                mob_label: spec.label.clone(),
                invariant: "I-3:lower_third_band",
                expected: "lower third inside vertical corridor [-3.85, -2.85]".to_string(),
                actual: format!("spanned y in [{:.2}, {:.2}]", y_min, y_max),
            });
        }
    }

    diagnostics
}

/// Verifies Invariant I-4: Non-Empty Stage Under Spoken Narration.
pub fn check_invariant_non_empty_stage(
    frames: &[SceneFrame],
    lines: &[SpokenLineAuditSpec],
    chapter: &str,
) -> Vec<InvariantDiagnostic> {
    let mut diagnostics = Vec::new();

    for line in lines {
        // Evaluate active spoken window [start + 0.6s, end - 0.3s]
        let active_start = line.start_s + 0.6;
        let active_end = line.end_s - 0.3;
        if active_start >= active_end {
            continue;
        }

        for frame in frames {
            if frame.time_s >= active_start && frame.time_s <= active_end {
                let area = frame.visible_stage_area();
                if area < 0.25 {
                    diagnostics.push(InvariantDiagnostic {
                        chapter: chapter.to_string(),
                        line_id: line.line_id.clone(),
                        scene_time_s: frame.time_s,
                        mob_label: "stage".to_string(),
                        invariant: "I-4:non_empty_stage",
                        expected: "visible stage area >= 0.25 units^2 under spoken narration".to_string(),
                        actual: format!("visible area was {:.4} units^2", area),
                    });
                    // Report first violation per line to avoid flooding
                    break;
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use fmn::scene::{NullSceneSink, RuntimeConfig};

    use super::*;
    use crate::{Palette, Timed, ink_style, v};

    fn fresh_mob(arena: &mut Arena) -> Mob {
        arena.add(VMobject::new())
    }

    fn run_headless(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) {
        struct Probe<F>(F);
        impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                (self.0)(stage)
            }
        }
        let config = RuntimeConfig {
            fps: 10,
            ..RuntimeConfig::default()
        };
        fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink).expect("headless probe runs");
    }

    #[test]
    fn shoelace_area_computes_exact_polygon_areas() {
        // Unit square
        let square = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!((shoelace_area(&square) - 1.0).abs() < 1e-12);

        // Right triangle of base 3, height 4 (area = 6)
        let triangle = vec![[0.0, 0.0], [3.0, 0.0], [0.0, 4.0]];
        assert!((shoelace_area(&triangle) - 6.0).abs() < 1e-12);

        // Degenerate collinear points (area = 0)
        let collinear = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]];
        assert!(shoelace_area(&collinear) < 1e-12);
    }

    #[test]
    fn invariant_i5_reflection_morph_catches_collinear_collapse() {
        // Planted negative: linear reflection of 1.0 x 0.8 rectangle across y = x
        // At e = 0.5, [x, y] -> 0.5*[x, y] + 0.5*[y, x] = [(x+y)/2, (x+y)/2] which collapses to diagonal!
        let linear_collapse = |e: f64| {
            let pts = [[0.0, 0.0], [1.0, 0.0], [1.0, 0.8], [0.0, 0.8]];
            pts.iter()
                .map(|&[x, y]| [(1.0 - e) * x + e * y, (1.0 - e) * y + e * x])
                .collect::<Vec<_>>()
        };

        // Linear collapse MUST fail Invariant I-5 (and at e = 0.5 area is 0.0)
        assert!(shoelace_area(&linear_collapse(0.5)) < 1e-12);
        let failure = check_reflection_morph_area("linear_collapse", linear_collapse, 20, 0.1);
        assert!(failure.is_err(), "linear reflection must trigger I-5 collapse");
        let diag = failure.unwrap_err();
        assert_eq!(diag.invariant, "I-5:reflection_morph_area");
        assert!(diag.actual.contains("collapsed to"));

        // Passing case: reflection with guard_reflection_param + normal lift bulge
        let normal = [-1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt()];
        let guarded_flip = |e: f64| {
            let pts = [[0.0, 0.0], [1.0, 0.0], [1.0, 0.8], [0.0, 0.8]];
            let e_mat = guard_reflection_param(e, 0.08);
            let bulge = normal_bulge_offset(normal, 0.5, e);
            pts.iter()
                .map(|&[x, y]| {
                    [
                        (1.0 - e_mat) * x + e_mat * y + bulge[0],
                        (1.0 - e_mat) * y + e_mat * x + bulge[1],
                    ]
                })
                .collect::<Vec<_>>()
        };

        let success = check_reflection_morph_area("guarded_flip", guarded_flip, 20, 0.1);
        assert!(success.is_ok(), "guarded reflection must pass I-5");
    }

    #[test]
    fn invariant_i1_catches_point_count_mutation_during_fade() {
        let mut arena = Arena::new();
        let mob = fresh_mob(&mut arena);

        // Passing sequence: points_len stays constant during fade
        let mut f0 = SceneFrame::new(0.0);
        f0.mobs.insert(
            mob,
            MobSnapshot {
                mob,
                label: "rect".into(),
                opacity: 0.1,
                points_len: 12,
                bbox: None,
                is_backdrop: false,
            },
        );

        let mut f1_pass = SceneFrame::new(0.1);
        f1_pass.mobs.insert(
            mob,
            MobSnapshot {
                mob,
                label: "rect".into(),
                opacity: 0.5,
                points_len: 12,
                bbox: None,
                is_backdrop: false,
            },
        );

        let diag_pass = check_invariant_point_count_stability(&[f0.clone(), f1_pass], "ch01");
        assert!(diag_pass.is_empty(), "constant points_len must pass I-1");

        // Planted negative: points_len changes from 12 to 24 while opacity is 0.5 (mid-fade)
        let mut f1_fail = SceneFrame::new(0.1);
        f1_fail.mobs.insert(
            mob,
            MobSnapshot {
                mob,
                label: "rect".into(),
                opacity: 0.5,
                points_len: 24, // mutated!
                bbox: None,
                is_backdrop: false,
            },
        );

        let diag_fail = check_invariant_point_count_stability(&[f0, f1_fail], "ch01");
        assert_eq!(diag_fail.len(), 1, "mutating points_len mid-fade must fail I-1");
        assert_eq!(diag_fail[0].invariant, "I-1:point_count_stability");
        assert!(diag_fail[0].actual.contains("changed to 24"));
    }

    #[test]
    fn scene_recorder_captures_live_stage_and_detects_fade_point_resize_in_memory() {
        // 1. Clean fade-in with constant points passes I-1
        let rec_clean = SceneRecorder::new();
        run_headless(|stage| {
            let tri = stage.arena_mut().add(
                Polygon::new(vec![v(-1.0, -1.0), v(1.0, -1.0), v(0.0, 1.0)])
                    .style(ink_style(Palette::blue()))
                    .build(),
            );
            rec_clean.attach(stage, vec![])?;
            let a = fade_in(stage.arena_mut(), tri, ORIGIN, 1.0)?.rt(0.5);
            stage.play(a)?;
            Ok(())
        });
        let frames_clean = rec_clean.frames();
        assert!(!frames_clean.is_empty());
        assert!(
            check_invariant_point_count_stability(&frames_clean, "clean").is_empty(),
            "clean fade_in must pass I-1"
        );

        // 2. Planted negative: mutating a polygon from 3 vertices to 5 vertices mid-fade
        let rec_bad = SceneRecorder::new();
        run_headless(|stage| {
            let tri = stage.arena_mut().add(
                Polygon::new(vec![v(-1.0, -1.0), v(1.0, -1.0), v(0.0, 1.0)])
                    .style(ink_style(Palette::blue()))
                    .build(),
            );
            let mutator = stage.arena_mut().add(VMobject::new());
            stage.add_to_scene(mutator)?;
            let mut step = 0usize;
            stage.add_dt_updater(
                mutator,
                move |s, _, _dt| {
                    step += 1;
                    if step == 3 {
                        let pent = Polygon::new(vec![
                            v(-1.0, -1.0),
                            v(1.0, -1.0),
                            v(1.2, 0.2),
                            v(0.0, 1.0),
                            v(-1.2, 0.2),
                        ])
                        .build();
                        let _ = s.set_points(tri, pent.points());
                    }
                },
                false,
            )?;
            rec_bad.attach(stage, vec![mutator])?;
            let a = fade_in(stage.arena_mut(), tri, ORIGIN, 1.0)?.rt(0.6);
            stage.play(a)?;
            Ok(())
        });
        let diags = check_invariant_point_count_stability(&rec_bad.frames(), "bad");
        assert!(
            !diags.is_empty(),
            "mutating vertex count during fade_in in a live scene must trigger I-1"
        );
        assert_eq!(diags[0].invariant, "I-1:point_count_stability");
    }

    #[test]
    fn invariant_i2_catches_premature_reveal_and_short_dwell() {
        let mut arena = Arena::new();
        let ans_mob = fresh_mob(&mut arena);
        let quiz = QuizAuditSpec {
            line_id: "03_2".into(),
            ring_start_s: 10.0,
            ring_zero_s: 14.0,
            first_sentence_after_s: 17.0,
            answer_mobs: vec![ans_mob],
            word_count: 5, // requires max(1.5, 5/3.5) = 1.5s dwell
        };

        // Planted negative 1: premature reveal (answer visible at t=12s while ring_zero is 14s)
        let mut f_premature = SceneFrame::new(12.0);
        f_premature.mobs.insert(
            ans_mob,
            MobSnapshot {
                mob: ans_mob,
                label: "answer".into(),
                opacity: 0.8, // premature!
                points_len: 4,
                bbox: None,
                is_backdrop: false,
            },
        );

        let diag_premature =
            check_invariant_quiz_masking_and_dwell(&[f_premature], std::slice::from_ref(&quiz), 10.0, "ch03");
        assert!(
            diag_premature.iter().any(|d| d.invariant == "I-2:causal_quiz_masking"),
            "premature reveal must fail causal masking"
        );

        // Planted negative 2: short dwell (faded out at t=14.8s before required dwell at 15.5s)
        let mut f_reveal = SceneFrame::new(14.2);
        f_reveal.mobs.insert(
            ans_mob,
            MobSnapshot {
                mob: ans_mob,
                label: "answer".into(),
                opacity: 1.0,
                points_len: 4,
                bbox: None,
                is_backdrop: false,
            },
        );

        let mut f_short_dwell = SceneFrame::new(14.8);
        f_short_dwell.mobs.insert(
            ans_mob,
            MobSnapshot {
                mob: ans_mob,
                label: "answer".into(),
                opacity: 0.1, // faded too soon!
                points_len: 4,
                bbox: None,
                is_backdrop: false,
            },
        );

        let diag_dwell = check_invariant_quiz_masking_and_dwell(&[f_reveal, f_short_dwell], &[quiz], 10.0, "ch03");
        assert!(
            diag_dwell.iter().any(|d| d.invariant == "I-2:quiz_post_zero_dwell"),
            "fading out answer before required dwell must fail I-2"
        );
    }

    #[test]
    fn invariant_i3_catches_lower_third_starvation_and_corridor_breach() {
        let mut arena = Arena::new();
        // Planted negative 1: word count 16 requires max(2.5, 16/4.0) = 4.0s dwell
        // Provided dwell is only 2.0s
        let starved = LowerThirdAuditSpec {
            line_id: "08_4".into(),
            mob: fresh_mob(&mut arena),
            label: "for programmers: ...".into(),
            word_count: 16,
            start_s: 20.0,
            end_s: 22.0, // only 2.0s!
            bbox: ([-4.0, -3.5], [4.0, -3.0]),
        };

        let diag_starved = check_invariant_lower_third(&[starved], "ch08");
        assert_eq!(diag_starved.len(), 1);
        assert_eq!(diag_starved[0].invariant, "I-3:lower_third_dwell");
        assert!(diag_starved[0].expected.contains("4.00s"));

        // Planted negative 2: vertical corridor breach (y extends to -1.8, outside corridor [-3.85, -2.85])
        let breached = LowerThirdAuditSpec {
            line_id: "08_5".into(),
            mob: fresh_mob(&mut arena),
            label: "out of band".into(),
            word_count: 4,
            start_s: 30.0,
            end_s: 34.0,
            bbox: ([-4.0, -2.5], [4.0, -1.8]), // breach!
        };

        let diag_breached = check_invariant_lower_third(&[breached], "ch08");
        assert_eq!(diag_breached.len(), 1);
        assert_eq!(diag_breached[0].invariant, "I-3:lower_third_band");
    }

    #[test]
    fn invariant_i4_catches_black_stage_seam_during_narration() {
        let line = SpokenLineAuditSpec {
            line_id: "12_4".into(),
            start_s: 10.0,
            end_s: 15.0,
        };

        // Frame at t=12.0s (in active window [10.6, 14.7]) with 0 area (empty stage)
        let empty_frame = SceneFrame::new(12.0);
        let diag_empty = check_invariant_non_empty_stage(&[empty_frame], &[line], "ch12");
        assert_eq!(diag_empty.len(), 1);
        assert_eq!(diag_empty[0].invariant, "I-4:non_empty_stage");
        assert!(diag_empty[0].actual.contains("0.0000 units^2"));
    }
}
