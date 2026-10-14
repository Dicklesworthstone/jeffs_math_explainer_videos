//! The layout lint against the defects a dense read of Video J's scored draft found and the lint did not
//! (every `*.layout.tsv` of the chapters was empty): each is a planted positive here, built the way the
//! chapter built it, with its near miss beside it that must stay quiet.
//!
//! - a stroke through the numerals of the nodes it joins (11_1, `walks.rs`): `crossed`
//! - a tag standing on a box's border (04_2): `crossed`
//! - a box whose edges cut through the number it is meant to enclose (13_3): `crossed`
//! - arrows within 0.06 of the top edge, in the middle of a beat only (09_3): `edge`, seen by a sampling lint only
//! - rows 0.18 from the left edge (04_6) and a bracket 0.16 from the top (07_4): `tight`
//! - brackets painted with a fill (11_1): `openfill`

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::{Kit, Linter, Palette, ink_style, rect, stroke_style, v};
use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::prelude::*;

type Build = Box<dyn FnOnce(&Kit, &mut Stage<'_>) -> fmn::Result<()>>;

struct Scene {
    kit: Kit,
    build: Option<Build>,
    report: Rc<RefCell<String>>,
}

impl SceneConstruct for Scene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let build = self.build.take().expect("built once");
        build(&self.kit.clone(), stage)?;
        self.kit.hold(stage)?;
        *self.report.borrow_mut() = self.kit.lint.as_ref().expect("a lint").report();
        stage.wait(0.1)?;
        Ok(())
    }
}

/// A directory no run has used (nothing is ever overwritten or deleted; the OS's temp cleaning reclaims it).
fn fresh_dir(tag: &str) -> std::path::PathBuf {
    // the counter keeps two tests that run at the same microsecond from choosing the same directory
    static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    (0u32..)
        .map(|k| std::env::temp_dir().join(format!("explainer_kit_{tag}_{}_{nanos}_{n}_{k}", std::process::id())))
        .find(|p| !p.exists())
        .expect("an unused name")
}

/// Build a scene, lint it (at the checkpoints only, or sampling), and return the findings.
fn lint(sampled: bool, build: impl FnOnce(&Kit, &mut Stage<'_>) -> fmn::Result<()> + 'static) -> String {
    let report = Rc::new(RefCell::new(String::new()));
    let linter = if sampled { Linter::sampled() } else { Linter::new() };
    let mut scene = Scene {
        kit: Kit::new().unwrap().with_lint(Rc::new(linter)),
        build: Some(Box::new(build)),
        report: Rc::clone(&report),
    };
    let mut opts = RenderOptions::with_format(fresh_dir("defects"), RenderFormat::PngSequence).unwrap();
    opts.config.camera.resolution = (32, 18);
    opts.config.camera.fps = 10;
    fmn::render(&mut scene, opts).expect("render");
    report.borrow().clone()
}

fn rows_of<'a>(report: &'a str, kind: &str) -> Vec<&'a str> {
    report.lines().filter(|r| r.split('\t').nth(1) == Some(kind)).collect()
}

fn add(stage: &mut Stage<'_>, m: VMobject) -> fmn::Result<Mob> {
    let m = stage.arena_mut().add(m);
    stage.add_to_scene(m)?;
    Ok(m)
}

fn line(stage: &mut Stage<'_>, from: Vec3, to: Vec3, width: f64) -> fmn::Result<Mob> {
    add(
        stage,
        polyline([from, to]).style(stroke_style(Palette::blue(), width)).build(),
    )
}

fn border(stage: &mut Stage<'_>, centre: Vec3, w: f64, h: f64) -> fmn::Result<Mob> {
    let r = rect(
        v(centre[0] - w / 2.0, centre[1] - h / 2.0),
        w,
        h,
        stroke_style(Palette::red(), 4.0),
    )?;
    add(stage, r)
}

fn shown(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, at: Vec3) -> fmn::Result<Mob> {
    let t = kit.text(stage, s, size, Palette::ink(), at)?;
    stage.add_to_scene(t)?;
    Ok(t)
}

// ---------------------------------------------------------------- crossed

#[test]
fn a_link_drawn_through_the_numerals_of_its_nodes_is_found() {
    // 11_1: the lit link runs from the centre of node 1 to the centre of node 2, over both numerals
    let through = lint(false, |kit, stage| {
        shown(kit, stage, "1", 40.0, v(-2.0, 0.0))?;
        shown(kit, stage, "2", 40.0, v(2.0, 0.0))?;
        line(stage, v(-2.0, 0.0), v(2.0, 0.0), 6.0)?;
        Ok(())
    });
    let crossed = rows_of(&through, "crossed");
    assert_eq!(crossed.len(), 2, "both numerals are struck: {through}");
    assert!(
        crossed.iter().any(|r| r.contains("\"1\"")) && crossed.iter().any(|r| r.contains("\"2\"")),
        "{through}"
    );
    // the fix the dense read asked for: the stroke ends at the node's radius, clear of the numeral
    let clear = lint(false, |kit, stage| {
        shown(kit, stage, "1", 40.0, v(-2.0, 0.0))?;
        shown(kit, stage, "2", 40.0, v(2.0, 0.0))?;
        line(stage, v(-1.6, 0.0), v(1.6, 0.0), 6.0)?;
        Ok(())
    });
    assert!(
        clear.is_empty(),
        "a link that stops at the node's radius is fine: {clear}"
    );
}

#[test]
fn a_tag_standing_on_a_boxes_border_is_found() {
    // 04_2: "we need 1" on the top-left corner and edge of the slope box
    let on_corner = lint(false, |kit, stage| {
        border(stage, v(0.0, 0.0), 4.0, 2.0)?;
        shown(kit, stage, "we need 1", 30.0, v(-1.6, 1.0))?;
        Ok(())
    });
    // the border was drawn first and the tag over it: the stroke is `behind` the text
    assert_eq!(rows_of(&on_corner, "behind").len(), 1, "{on_corner}");
    assert!(rows_of(&on_corner, "crossed").is_empty(), "{on_corner}");
    let above = lint(false, |kit, stage| {
        border(stage, v(0.0, 0.0), 4.0, 2.0)?;
        shown(kit, stage, "we need 1", 30.0, v(-1.6, 1.6))?;
        Ok(())
    });
    assert!(above.is_empty(), "the tag clear above the corner is fine: {above}");
}

#[test]
fn a_box_whose_edges_cut_through_its_own_number_is_found() {
    // 13_3: the red box round the wrong-sign entry was 0.9 of the grid pitch wide on a number wider than that
    let too_narrow = lint(false, |kit, stage| {
        let n = shown(kit, stage, "\u{2212}0.839", 40.0, v(0.0, 0.0))?;
        let w = stage.get_width(n);
        border(stage, v(0.0, 0.0), w * 0.55, 0.9)?;
        Ok(())
    });
    // the box was added after the number, so its edges are painted over it
    assert!(!rows_of(&too_narrow, "crossed").is_empty(), "{too_narrow}");
    // added before it, the number is painted over the box's edges: still a defect, named `behind`
    let under = lint(false, |kit, stage| {
        let n = kit.text(stage, "\u{2212}0.839", 40.0, Palette::ink(), v(0.0, 0.0))?;
        let w = stage.get_width(n);
        border(stage, v(0.0, 0.0), w * 0.55, 0.9)?; // on the scene first ...
        stage.add_to_scene(n)?; // ... and the number over it
        Ok(())
    });
    assert!(!rows_of(&under, "behind").is_empty(), "{under}");
    // sized from the number's own width, with room: the frame the text sits inside
    let enclosing = lint(false, |kit, stage| {
        let n = shown(kit, stage, "\u{2212}0.839", 40.0, v(0.0, 0.0))?;
        let (w, h) = (stage.get_width(n), stage.get_height(n));
        border(stage, v(0.0, 0.0), w + 0.3, h + 0.3)?;
        Ok(())
    });
    assert!(
        enclosing.is_empty(),
        "a frame round the text is no crossing: {enclosing}"
    );
}

#[test]
fn a_label_inside_its_own_filled_box_or_beside_a_line_is_quiet() {
    let calm = lint(false, |kit, stage| {
        // a filled, bordered bar with its number inside; an axis line under a tick label; an underline; an arrow
        let bar = rect(v(-3.0, -1.0), 2.0, 1.0, ink_style(Palette::teal()))?;
        add(stage, bar)?;
        border(stage, v(-2.0, -0.5), 2.0, 1.0)?;
        shown(kit, stage, "42", 30.0, v(-2.0, -0.5))?;
        line(stage, v(0.5, -1.0), v(5.0, -1.0), 4.0)?;
        shown(kit, stage, "x axis", 28.0, v(2.5, -1.5))?;
        shown(kit, stage, "underlined", 30.0, v(2.5, 1.5))?;
        line(stage, v(1.4, 1.2), v(3.6, 1.2), 3.0)?;
        Ok(())
    });
    assert!(calm.is_empty(), "{calm}");
}

#[test]
fn a_link_under_the_disc_of_its_node_does_not_strike_the_numeral_on_the_disc() {
    // the network's links run centre to centre, under the opaque disc of each node, and the numeral stands on the disc
    let with_disc = lint(false, |kit, stage| {
        line(stage, v(-2.0, 0.0), v(2.0, 0.0), 6.0)?;
        add(stage, rect(v(-2.4, -0.4), 0.8, 0.8, ink_style(Palette::teal()))?)?;
        shown(kit, stage, "1", 40.0, v(-2.0, 0.0))?;
        Ok(())
    });
    assert!(with_disc.is_empty(), "{with_disc}");
    let without = lint(false, |kit, stage| {
        line(stage, v(-2.0, 0.0), v(2.0, 0.0), 6.0)?;
        shown(kit, stage, "1", 40.0, v(-2.0, 0.0))?;
        Ok(())
    });
    assert_eq!(rows_of(&without, "behind").len(), 1, "{without}");
}

#[test]
fn a_leader_line_that_starts_at_its_label_is_not_a_crossing_but_one_that_ends_in_it_is() {
    // E 09_2: "5 6 4" with a line to its block that begins at the label's right edge
    let leader = |into: f64| {
        lint(false, move |kit, stage| {
            let t = shown(kit, stage, "5 6 4", 30.0, v(0.0, 0.0))?;
            let right = stage.get_right(t)[0];
            line(stage, v(right - into, 0.0), v(right + 2.0, 1.0), 4.0)?;
            Ok(())
        })
    };
    let touching = leader(0.03);
    assert!(touching.is_empty(), "{touching}");
    // a label 0.64 wide: a line that starts half way into it runs through the cores of its last glyphs
    let ending_in_the_label = leader(0.5);
    assert!(
        !rows_of(&ending_in_the_label, "crossed").is_empty(),
        "{ending_in_the_label}"
    );
}

#[test]
fn a_strike_through_the_chapter_declares_is_left_alone() {
    // J 02: a red X over "the guess"; F 13: a red stroke over a wrong answer on the calculator
    let strike = |declared: bool| {
        lint(false, move |kit, stage| {
            shown(kit, stage, "522.6", 40.0, v(0.0, 0.0))?;
            let x = line(stage, v(-0.8, -0.3), v(0.8, 0.3), 8.0)?;
            if declared {
                kit.allow_crossing(x);
            }
            Ok(())
        })
    };
    assert_eq!(rows_of(&strike(false), "crossed").len(), 1);
    assert!(strike(true).is_empty(), "{}", strike(true));
}

#[test]
fn a_label_on_its_translucent_card_hides_the_line_it_stands_on() {
    // K 05_8: the "0 kN" chip sits on the member it names; the card is not quite opaque
    let chip = lint(false, |kit, stage| {
        line(stage, v(0.0, -1.0), v(0.0, 1.0), 4.0)?;
        add(
            stage,
            rect(
                v(-0.6, -0.3),
                1.2,
                0.6,
                ink_style(Palette::faint()).fill(Palette::faint(), 0.9),
            )?,
        )?;
        shown(kit, stage, "0 kN", 30.0, v(0.0, 0.0))?;
        Ok(())
    });
    assert!(chip.is_empty(), "{chip}");
}

#[test]
fn a_faint_background_line_behind_a_text_is_a_texture_not_a_crossing() {
    // H's title card stands over a faint grid; the same line in `dim` or brighter would be a stroke through it
    let through = |color: Srgb| {
        lint(false, move |kit, stage| {
            add(
                stage,
                polyline([v(-3.0, 0.0), v(3.0, 0.0)])
                    .style(stroke_style(color, 4.0))
                    .build(),
            )?;
            shown(kit, stage, "How to Find a Lost Submarine", 36.0, v(0.0, 0.0))?;
            Ok(())
        })
    };
    let faint = through(Palette::faint());
    assert!(faint.is_empty(), "{faint}");
    let dim = through(Palette::dim());
    assert_eq!(rows_of(&dim, "behind").len(), 1, "{dim}");
}

#[test]
fn shapes_parked_off_screen_and_backdrops_are_not_cut_off() {
    let quiet = lint(false, |_, stage| {
        // parked far outside the frame to come in later; a backdrop that covers the whole frame
        add(stage, rect(v(-20.0, -20.0), 0.2, 0.2, ink_style(Palette::teal()))?)?;
        let (w, h) = (2.0 * FRAME_X_RADIUS, 2.0 * FRAME_Y_RADIUS);
        add(
            stage,
            rect(v(-FRAME_X_RADIUS, -FRAME_Y_RADIUS), w, h, ink_style(Palette::faint()))?,
        )?;
        Ok(())
    });
    assert!(quiet.is_empty(), "{quiet}");
    // half in, half out is a cut-off shape
    let cut = lint(false, |_, stage| {
        add(
            stage,
            rect(v(FRAME_X_RADIUS - 0.5, 0.0), 1.0, 1.0, ink_style(Palette::teal()))?,
        )?;
        Ok(())
    });
    assert_eq!(rows_of(&cut, "outside").len(), 1, "{cut}");
}

// ---------------------------------------------------------------- sampling

/// A link that sits over a numeral from `from` to `to` seconds of a beat and is back where it was after.
fn sweep(from: f64, to: f64, wait: f64) -> impl FnOnce(&Kit, &mut Stage<'_>) -> fmn::Result<()> {
    move |kit, stage| {
        kit.say(stage, "t_1")?;
        shown(kit, stage, "7", 40.0, v(0.0, 0.0))?;
        let l = line(stage, v(-2.0, 2.5), v(2.0, 2.5), 6.0)?;
        let mut clock = 0.0_f64;
        let mut at = 0.0_f64;
        stage.add_dt_updater(
            l,
            move |s, m, dt| {
                clock += dt;
                let want = if (from..to).contains(&clock) { -2.5 } else { 0.0 };
                s.shift(m, [0.0, want - at, 0.0]);
                at = want;
            },
            false,
        )?;
        stage.wait(wait)?;
        Ok(())
    }
}

#[test]
fn what_is_only_there_in_the_middle_of_a_beat_is_seen_by_a_sampling_lint_alone() {
    // a link sits over a numeral for more than a second of the beat and is gone again when the beat ends
    let checkpoints_only = lint(false, sweep(0.4, 1.8, 2.6));
    assert!(rows_of(&checkpoints_only, "crossed").is_empty(), "{checkpoints_only}");
    let sampled = lint(true, sweep(0.4, 1.8, 2.6));
    let crossed = rows_of(&sampled, "crossed");
    assert_eq!(crossed.len(), 1, "{sampled}");
    assert!(
        crossed[0].starts_with("t_1 +"),
        "a sampled look is named after the line being said: {sampled}"
    );
}

#[test]
fn the_sampling_survives_a_wipe_that_clears_every_updater() {
    // fade_all clears the updaters of every top-level mobject, the lint's probe included: the next beat must
    // put a working one back, or the rest of the chapter would be looked at only at its checkpoints
    let after_wipe = lint(true, |kit, stage| {
        kit.say(stage, "t_1")?;
        shown(kit, stage, "3", 40.0, v(0.0, 0.0))?;
        stage.wait(0.5)?;
        explainer_kit::fade_all(stage, 0.3)?;
        // the same sweep as above in the beat after the wipe: over the numeral from 0.4 s to 1.8 s
        sweep(0.4, 1.8, 2.6)(kit, stage)
    });
    let crossed = rows_of(&after_wipe, "crossed");
    assert_eq!(crossed.len(), 1, "{after_wipe}");
}

#[test]
fn a_crossing_that_lasts_less_than_a_second_is_a_transition_not_a_finding() {
    // pictures change over a second or so, their old and new parts passing through each other
    let brief = lint(true, sweep(0.4, 1.0, 2.0));
    assert!(brief.is_empty(), "{brief}");
}

#[test]
fn arrows_that_come_within_a_hair_of_the_top_edge_mid_beat_are_found_when_sampling() {
    // 09_3: the outermost arrows of the field reached 0.06 from the top edge for three seconds
    fn scene(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        kit.say(stage, "t_2")?;
        let a = polyline([v(-1.0, 3.2), v(1.0, 3.2)])
            .style(stroke_style(Palette::blue(), 4.0))
            .build();
        let a = add(stage, a)?;
        let mut clock = 0.0_f64;
        let mut at = 0.0_f64;
        let lift = FRAME_Y_RADIUS - 0.06 - 3.2;
        stage.add_dt_updater(
            a,
            move |s, m, dt| {
                clock += dt;
                let want = if (0.3..1.9).contains(&clock) { lift } else { 0.0 };
                s.shift(m, [0.0, want - at, 0.0]);
                at = want;
            },
            false,
        )?;
        stage.wait(2.6)?;
        Ok(())
    }
    assert!(
        lint(false, scene).is_empty(),
        "the checkpoints see the arrows back in place"
    );
    let sampled = lint(true, scene);
    assert_eq!(rows_of(&sampled, "edge").len(), 1, "{sampled}");
}

// ---------------------------------------------------------------- edges of text

#[test]
fn text_close_to_the_left_or_top_edge_is_tight_but_a_shape_is_not() {
    // 04_6: rows start 0.18 from the left edge; 07_4: the J^2 bracket's top is 0.16 from the top
    let tight = lint(false, |kit, stage| {
        let row = shown(kit, stage, "f(x) = 1 + x + x^2", 30.0, v(0.0, 0.0))?;
        let at = stage.get_left(row);
        stage.shift(row, [-FRAME_X_RADIUS + 0.18 - at[0], 0.0, 0.0]);
        let top = shown(kit, stage, "J^2 = -I", 30.0, v(0.0, 0.0))?;
        let up = stage.get_top(top);
        stage.shift(top, [0.0, FRAME_Y_RADIUS - 0.16 - up[1], 0.0]);
        // the same distances for a shape: not the text rule's business
        let bar = rect(
            v(-1.0, FRAME_Y_RADIUS - 0.2 - 0.3),
            2.0,
            0.3,
            ink_style(Palette::teal()),
        )?;
        add(stage, bar)?;
        Ok(())
    });
    let rows = rows_of(&tight, "tight");
    assert_eq!(rows.len(), 2, "{tight}");
    assert!(
        rows.iter().any(|r| r.contains("left edge")) && rows.iter().any(|r| r.contains("top edge")),
        "{tight}"
    );
    assert!(
        rows_of(&tight, "edge").is_empty(),
        "outside the 0.15 margin, so not an `edge`: {tight}"
    );
    let roomy = lint(false, |kit, stage| {
        let row = shown(kit, stage, "f(x) = 1 + x + x^2", 30.0, v(0.0, 0.0))?;
        let at = stage.get_left(row);
        stage.shift(row, [-FRAME_X_RADIUS + 0.45 - at[0], 0.0, 0.0]);
        // the lower third keeps its 0.15: a text at the bottom corridor's limit is not tight
        let low = shown(kit, stage, "for programmers: update A's effect", 26.0, v(0.0, 0.0))?;
        let bottom = stage.get_bottom(low);
        stage.shift(low, [0.0, -FRAME_Y_RADIUS + 0.17 - bottom[1], 0.0]);
        Ok(())
    });
    assert!(roomy.is_empty(), "{roomy}");
}

// ---------------------------------------------------------------- fills

#[test]
fn a_bracket_painted_with_a_fill_is_found_and_a_thin_one_is_not() {
    // 11_1: `set_opacity(.., true)` on the bracket's polyline filled it into a solid bar
    let bracket = |fill: f64| {
        let style = Style::default()
            .fill(Palette::green(), fill)
            .stroke(Palette::green(), 4.0, 1.0);
        polyline([v(-3.6, 1.0), v(-3.9, 1.0), v(-3.9, -1.0), v(-3.6, -1.0)])
            .style(style)
            .build()
    };
    let solid = lint(false, move |_, stage| {
        add(stage, bracket(1.0))?;
        Ok(())
    });
    assert_eq!(rows_of(&solid, "openfill").len(), 1, "{solid}");
    let thin = lint(false, move |_, stage| {
        add(stage, bracket(0.0))?;
        // a closed shape with a fill, and an arrow's shaft with a fill, are not brackets
        add(stage, rect(v(0.0, 0.0), 1.0, 1.0, ink_style(Palette::teal()))?)?;
        let shaft = polyline([v(1.0, 2.0), v(3.0, 2.0)])
            .style(
                Style::default()
                    .fill(Palette::blue(), 1.0)
                    .stroke(Palette::blue(), 4.0, 1.0),
            )
            .build();
        add(stage, shaft)?;
        Ok(())
    });
    assert!(thin.is_empty(), "{thin}");
}
