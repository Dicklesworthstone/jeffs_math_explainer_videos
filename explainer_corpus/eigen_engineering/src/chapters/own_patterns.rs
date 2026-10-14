//! Chapter 03, own patterns: the first time the arrow test is run. Slide the
//! carts toward each other and the pushes are three times the moves, the
//! same pattern, bigger (03_1, 03_2). Draw a move as one arrow on a plane,
//! the left cart's move across and the right cart's up, and the push the
//! same way, with the carts above driven by the blue arrow (03_3). Turn the
//! move round: the push runs ahead of it or falls behind, and the two line
//! up along two lines only (03_4). Along those lines the push is the move
//! at another size (03_5), and the lines are named: eigenvector, and its
//! size change, eigenvalue (03_6).
//!
//! Every number is a fact or the model's: the eigenvalues `K-A2.eigenvalues`,
//! the lines `K-A2.pattern.soft` and `.stiff`, the lined-up angles
//! `K-A2.lined_up_deg`, the largest turn `K-A2.worst_turn_deg`; the orange
//! arrow is the carts' table applied to the blue one every frame.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::arrows::{ArrowTest, axis_labels, own_lines, turn_deg};
use super::carts::{AGAINST, CartsPic, CartsSpec, pushes_for};
use super::draw::{
    MatrixGrid, Plane, chip, fade_in_all, fade_out_all, map2, minus, moves, numbers, own, panel, pause, predict_hold,
    pushes, run, say, tween_all, whole,
};
use super::two_carts::column_entries;
use crate::model::carts::table;

// ------------------------------------------------------------------ choices

/// The carts sit above the plane, at this scale.
const CARTS_AT: Vec3 = [0.0, 2.6, 0.0];
const CARTS_SCALE: f64 = 0.75;
/// The plane: its origin on screen and scene units to one plane unit; the
/// axes reach this far (plane units) across and up.
const PLANE_AT: Vec3 = [-0.9, -0.95, 0.0];
const PLANE_UNIT: f64 = 0.7;
const REACH: (f64, f64) = (3.4, 2.9);
/// Where the lists of 03_1 and 03_2 sit, and their type size.
const LIST_X: f64 = 3.0;
const LIST_Y: f64 = 0.2;
const LIST_SIZE: f64 = 52.0;
/// The turn readout, on the right.
const READOUT_X: f64 = 5.4;

// ------------------------------------------------------------- the geometry

/// The angle, in degrees, of a pattern drawn as a pair of numbers.
pub fn pattern_degrees(p: [f64; 2]) -> f64 {
    p[1].atan2(p[0]).to_degrees()
}

/// A card with TeX on it, detached: the lower-third of 03_6 (`K q = λ q`).
fn tex_card(kit: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, edge: Srgb, at: Vec3) -> fmn::Result<Mob> {
    let tex = kit.tex(
        stage,
        src,
        size,
        Palette::ink(),
        &[("q", own()), (r"\lambda", own())],
        at,
    )?;
    let (w, h) = (stage.get_width(tex), stage.get_height(tex));
    let back = Rectangle::new()
        .width(w + 0.6)
        .height(h + 0.4)
        .corner_radius(0.12)
        .style(Style::default().fill(panel(), 0.96).stroke(edge, 2.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    super::draw::group(stage, &[back, tex])
}

// -------------------------------------------------------------- the scene

pub struct OwnPatterns {
    pub kit: Kit,
}

impl SceneConstruct for OwnPatterns {
    fn name(&self) -> &str {
        "03_own_patterns"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange, gold) = (moves(), pushes(), own());
        let eig = numbers("K-A2.eigenvalues")?;
        let lines = [numbers("K-A2.pattern.soft")?, numbers("K-A2.pattern.stiff")?];
        let line_degrees = [
            pattern_degrees([lines[0][0], lines[0][1]]),
            pattern_degrees([lines[1][0], lines[1][1]]),
        ];

        // the carts, driven by the trackers the arrow plane shares
        let u = [stage.add_value_tracker(0.0), stage.add_value_tracker(0.0)];
        let plane = Plane {
            centre: PLANE_AT,
            unit: PLANE_UNIT,
        };
        let test = ArrowTest::build(stage, plane, map2(&table()), u, REACH)?;
        let pic = CartsPic::build(
            &kit,
            stage,
            CartsSpec {
                centre: CARTS_AT,
                scale: CARTS_SCALE,
            },
            u,
        )?;

        // ---- 03_1: toward each other: what pushes does that take?
        let line = say(&kit, stage, "03_1")?;
        pic.start(stage)?;
        let mut first = pic.frame.clone();
        first.extend(pic.carts);
        first.extend(pic.springs);
        fade_in_all(stage, &first, 0.7)?;
        let moves_label = kit.text(stage, "moves", 30.0, blue, v(-LIST_X, LIST_Y + 1.15))?;
        let pushes_label = kit.text(stage, "pushes", 30.0, orange, v(LIST_X, LIST_Y + 1.15))?;
        let moves_list = MatrixGrid::build(
            &kit,
            stage,
            &column_entries(AGAINST),
            LIST_SIZE,
            blue,
            v(-LIST_X, LIST_Y),
        )?;
        let unknown = [
            kit.text(stage, "?", 60.0, orange, v(LIST_X, LIST_Y + 0.4))?,
            kit.text(stage, "?", 60.0, orange, v(LIST_X, LIST_Y - 0.4))?,
        ];
        line.wait_for(stage, "slide them one centimetre")?;
        tween_all(
            stage,
            &[(pic.moves_on, 1.0), (u[0], AGAINST[0]), (u[1], AGAINST[1])],
            1.4,
        )?;
        fade_in_all(
            stage,
            &[moves_label, pushes_label, moves_list.group, unknown[0], unknown[1]],
            0.5,
        )?;
        predict_hold(&kit, stage)?;

        // ---- 03_2: three and minus three, the same pattern three times the size
        let line = say(&kit, stage, "03_2")?;
        let answer = pushes_for(AGAINST);
        let pushes_list = MatrixGrid::build(
            &kit,
            stage,
            &column_entries(answer),
            LIST_SIZE,
            orange,
            v(LIST_X, LIST_Y),
        )?;
        fade_out_all(stage, &unknown, 0.2)?;
        fade_in_all(stage, &[pushes_list.group], 0.4)?;
        tween_all(
            stage,
            &[
                (pic.hands_on[0], 1.0),
                (pic.hands_on[1], 1.0),
                (pic.pushes_on[0], 1.0),
                (pic.pushes_on[1], 1.0),
            ],
            0.7,
        )?;
        line.wait_for(stage, "The same pattern, three times the size")?;
        let mut pulse = Vec::new();
        for g in [moves_list.group, pushes_list.group] {
            pulse.push(stage.prepare(g.animate().set_anim_args(run(0.5))?.set_color(gold)?)?);
        }
        stage.play_prepared(pulse)?;

        // ---- 03_3: a move as one arrow, a push the same way
        let line = say(&kit, stage, "03_3")?;
        let mut leaving = vec![moves_label, pushes_label, moves_list.group, pushes_list.group];
        leaving.extend(pic.hands);
        leaving.extend(pic.push_arrows);
        fade_out_all(stage, &leaving, 0.5)?;
        // the carts' own arrows and hands are gone with them; the plane has its own
        for m in [
            pic.moves_on,
            pic.hands_on[0],
            pic.hands_on[1],
            pic.pushes_on[0],
            pic.pushes_on[1],
        ] {
            stage.set_tracker_value(m, 0.0)?;
        }
        test.start(stage)?;
        let labels = axis_labels(&kit, stage, plane, REACH)?;
        let mut plane_mobs = test.axes.clone();
        plane_mobs.extend(&labels);
        line.wait_for(stage, "Draw the moves as one arrow")?;
        fade_in_all(stage, &plane_mobs, 0.6)?;
        // a move of 1, 0 as one arrow, with its two numbers dropped to the axes
        tween_all(
            stage,
            &[(u[0], 1.0), (u[1], 0.0), (test.blue_on, 1.0), (test.drops_on, 1.0)],
            0.9,
        )?;
        line.wait_for(stage, "Draw the pushes the same way")?;
        tween_all(stage, &[(test.orange_on, 1.0)], 0.6)?;
        // the three pairs already found, one after another
        for pair in [[1.0, 1.0], [1.0, -1.0], [1.0, 0.0]] {
            tween_all(stage, &[(u[0], pair[0]), (u[1], pair[1])], 0.9)?;
            pause(stage, 0.5)?;
        }

        // ---- 03_4: turn the move round: two lines only
        let line = say(&kit, stage, "03_4")?;
        // the turn readout, in degrees
        let table_for_readout = map2(&table());
        let (ux, uy) = (u[0], u[1]);
        let turn_label = kit.text(stage, "turn", 30.0, dim, v(READOUT_X, 1.3))?;
        fade_in_all(stage, &[turn_label], 0.4)?;
        let readout = kit.live_value(
            stage,
            move |s| {
                let mv = [s.tracker_value(ux).unwrap_or(0.0), s.tracker_value(uy).unwrap_or(0.0)];
                // a 0-place DecimalNumber truncates: round the reading itself
                // (26.57 degrees is 27, and the 30 at the most is not 29)
                turn_deg(&table_for_readout, mv).round()
            },
            0,
            false,
            96.0,
            ink,
            v(READOUT_X + 0.7, 0.35),
            RIGHT,
        )?;
        let degree = kit.text(stage, "\u{b0}", 72.0, ink, v(READOUT_X + 1.1, 0.5))?;
        fade_in_all(stage, &[degree], 0.3)?;
        // the sweep starts from the move 1, 0 (degrees 0, a full-length arrow)
        stage.set_tracker_value(test.degrees, 0.0)?;
        stage.set_tracker_value(test.radius, 1.0)?;
        stage.set_tracker_value(test.polar, 1.0)?;
        let bands = own_lines(
            stage,
            plane,
            &[[lines[0][0], lines[0][1]], [lines[1][0], lines[1][1]]],
            REACH.0,
            REACH.1,
        );
        stage.add_many_to_scene(&bands)?;
        // round, slowing where the two line up
        let (soft, stiff) = (line_degrees[0], line_degrees[1]);
        tween_all(stage, &[(test.degrees, soft - 12.0)], 1.5)?;
        tween_all(stage, &[(test.degrees, soft + 12.0)], 1.7)?;
        line.wait_for(stage, "They line up along two lines only")?;
        // a wide soft band and a thin core on each line (own_lines: soft, core)
        let mut band_in = Vec::new();
        for (k, &b) in bands.iter().enumerate() {
            let opacity = if k % 2 == 0 { 0.22 } else { 0.9 };
            band_in.push(stage.prepare(b.animate().set_anim_args(run(0.8))?.set_opacity(opacity)?)?);
        }
        stage.play_prepared(band_in)?;
        tween_all(stage, &[(test.degrees, 180.0 + stiff - 12.0)], 1.5)?;
        tween_all(stage, &[(test.degrees, 180.0 + stiff + 12.0)], 1.5)?;
        line.wait_for(stage, "Both carts together")?;
        // together: the pushes slide out from under the moves so both are seen
        tween_all(stage, &[(test.degrees, soft), (test.slide, 0.22)], 1.4)?;
        line.wait_for(stage, "the carts against each other")?;
        tween_all(stage, &[(test.degrees, stiff), (test.slide, 0.0)], 1.4)?;

        // ---- 03_5: the pushes come in the same pattern, only the size changes
        let line = say(&kit, stage, "03_5")?;
        let largest = numbers("K-A2.worst_turn_deg")?;
        let strip = lower_third(
            &kit,
            stage,
            "largest turn:",
            &format!(
                "sin = ({} {} {}) \u{f7} ({} + {}), {}\u{b0}",
                whole(eig[1]),
                minus("-"),
                whole(eig[0]),
                whole(eig[1]),
                whole(eig[0]),
                whole(largest[0])
            ),
        )?;
        fade_in_all(stage, &[strip], 0.5)?;
        // the line ends, where each size change is written
        let ends = {
            let (_, soft_end) = super::arrows::band_ends([lines[0][0], lines[0][1]], REACH.0, REACH.1);
            let (_, stiff_end) = super::arrows::band_ends([lines[1][0], lines[1][1]], REACH.0, REACH.1);
            [plane.p(soft_end[0], soft_end[1]), plane.p(stiff_end[0], stiff_end[1])]
        };
        let sizes = [
            kit.text_left(
                stage,
                &format!("\u{d7} {}", whole(eig[0])),
                30.0,
                gold,
                v(ends[0][0] + 0.2, ends[0][1] + 0.1),
            )?,
            kit.text_left(
                stage,
                &format!("\u{d7} {}", whole(eig[1])),
                30.0,
                gold,
                v(ends[1][0] + 0.2, ends[1][1] + 0.1),
            )?,
        ];
        line.wait_for(stage, "Move the carts in one of them")?;
        tween_all(stage, &[(test.degrees, soft), (test.slide, 0.22)], 1.0)?;
        fade_in_all(stage, &[sizes[0]], 0.4)?;
        tween_all(stage, &[(test.degrees, stiff), (test.slide, 0.0)], 1.0)?;
        fade_in_all(stage, &[sizes[1]], 0.4)?;
        line.wait_for(stage, "Only the size changes")?;

        // ---- 03_6: the names land
        let line = say(&kit, stage, "03_6")?;
        let vectors = [
            kit.text_left(stage, "eigenvector", 28.0, gold, v(ends[0][0] + 0.2, ends[0][1] + 0.55))?,
            kit.text_left(stage, "eigenvector", 28.0, gold, v(ends[1][0] + 0.2, ends[1][1] + 0.55))?,
        ];
        let values = [
            kit.text_left(
                stage,
                &format!("eigenvalue {}", whole(eig[0])),
                28.0,
                gold,
                v(ends[0][0] + 0.2, ends[0][1] - 0.35),
            )?,
            kit.text_left(
                stage,
                &format!("eigenvalue {}", whole(eig[1])),
                28.0,
                gold,
                v(ends[1][0] + 0.2, ends[1][1] - 0.35),
            )?,
        ];
        let german = chip(&kit, stage, "eigen (German): own", 28.0, gold, v(READOUT_X, 2.8))?;
        let equation = tex_card(&kit, stage, r"K\,q=\lambda\,q", 50.0, gold, v(READOUT_X - 0.4, -2.4))?;
        fade_out_all(stage, &[strip, sizes[0], sizes[1]], 0.4)?;
        line.wait_for(stage, "An own pattern is called an eigenvector")?;
        fade_in_all(stage, &vectors, 0.5)?;
        line.wait_for(stage, "its size change an eigenvalue")?;
        fade_in_all(stage, &values, 0.5)?;
        line.wait_for(stage, "Eigen is German for own")?;
        fade_in_all(stage, &[german], 0.5)?;
        line.wait_for(stage, "For the carts, the eigenvalues are one and three")?;
        fade_in_all(stage, &[equation], 0.6)?;
        kit.hold(stage)?;
        test.stop(stage);
        pic.stop(stage);
        stage.clear_updaters(readout, true);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{items, testing};
    use super::*;

    /// 03_1 and 03_2: sliding the carts toward each other asks for three
    /// and minus three, the moves again at three times the size.
    #[test]
    fn toward_each_other_the_pushes_are_three_times_the_moves() {
        assert_eq!(pushes_for(AGAINST), [3.0, -3.0]);
        assert_eq!(numbers("K-A1.push.against").unwrap(), [3.0, -3.0]);
        let eig = numbers("K-A2.eigenvalues").unwrap();
        assert_eq!(pushes_for(AGAINST), [eig[1] * AGAINST[0], eig[1] * AGAINST[1]]);
    }

    /// 03_4 and 03_5: the two lines the sweep slows on are the facts'
    /// patterns, at 45 and -45 degrees, and the readout's largest value is
    /// the lower-third's thirty degrees: sin = (3 - 1) / (3 + 1).
    #[test]
    fn the_gold_lines_are_the_patterns_and_the_lower_third_is_arithmetic() {
        let soft = numbers("K-A2.pattern.soft").unwrap();
        let stiff = numbers("K-A2.pattern.stiff").unwrap();
        assert!((pattern_degrees([soft[0], soft[1]]) - 45.0).abs() < 1e-9);
        assert!((pattern_degrees([stiff[0], stiff[1]]) + 45.0).abs() < 1e-9);
        let eig = numbers("K-A2.eigenvalues").unwrap();
        let worst = ((eig[1] - eig[0]) / (eig[1] + eig[0])).asin().to_degrees();
        assert!(
            (worst - numbers("K-A2.worst_turn_deg").unwrap()[0]).abs() < 1e-9,
            "{worst}"
        );
        // each line is an own direction: the table sends it along itself
        for p in [soft, stiff] {
            assert!(turn_deg(&map2(&table()), [p[0], p[1]]) < 1e-9);
        }
        assert_eq!(items("K-A2.directions").unwrap(), ["2"]);
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("03_own_patterns").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "turn",
            "largest turn:",
            "sin = (3 \u{2013} 1) \u{f7} (3 + 1), 30\u{b0}",
            "eigenvector",
            "eigenvalue 1",
            "eigenvalue 3",
            "eigen (German): own",
            r"K\,q=\lambda\,q",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // the length that counts is the one with the scratch voice placed
        let s = testing::run_voiced("03_own_patterns").unwrap().ran.seconds;
        eprintln!("03_own_patterns: {s:.1} s on the scratch voice");
        assert!(s > 45.0 && s < 85.0, "{s} s");
    }
}
