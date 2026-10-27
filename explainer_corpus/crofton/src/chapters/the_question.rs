//! Chapter 01, how long is it? The coast drawn on its card, its parts
//! named, and the question (01_1): the length of the line itself, every
//! bend, not the distance between its ends, 14.7 cm (01_2). A ruler fits
//! the beach but touches the headland at one point (01_3). Dividers opened
//! to 2 cm walk the coast, eleven steps and a short bit (01_4); too long,
//! too short or about right? (01_5, a prediction.) Too short, always: a
//! zoom onto step 6, 2 cm straight across 3.08 cm of coast (01_6). Each
//! smaller step cuts less off its bend but the walk needs more steps (01_7;
//! nothing is claimed about the total), and the chapter ends on the card and
//! the coast alone, where the draft's chapter 02 begins.

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Dividers, EVERY, card_panel, card_point, dashed, dim_line, dot, fade_in_all, fade_in_lagged, fade_out_all, fade_to,
    fail, flat, hinge, line_style, now, path, pause, rider, say, stand, stop, swap, think, together, traced, zoom,
    zoomed,
};
use crate::facts::{self, headland_step};
use crate::model::{self, CARD_UNIT, NAMED, OPENING_STEP, P2, PIECE_NAMES, RULER, SMALLER_STEP};
use crate::palette;

pub struct TheQuestion {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const ASK: &str = "?";
const STEPS_WORD: &str = "steps";
/// 01_1 names the parts of [`model::NAMED`] with these words, in order.
const NAMED_PHRASES: [&str; 5] = ["A cove", "a long beach", "a knobbly headland", "a round bay", "a hook"];

// ------------------------------------------------------------------ layout

/// The coast's stroke, and the tracer's.
const COAST_W: f64 = 4.0;
const TRACE_W: f64 = 2.5;
/// Labels: their size, and how far outside their piece they sit, cm.
const LABEL_SIZE: f64 = 22.0;
const LABEL_OFF: f64 = 0.45;
/// "16 cm", under the card's lower edge.
const WIDTH_LABEL_AT: Vec3 = [0.0, -3.62, 0.0];
/// The band of the card under the coast: the ends' distance, the live
/// count and the dividers' sum.
const BAND_Y: f64 = -2.6;
const COUNT_RIGHT_X: f64 = -0.1;
const STEPS_WORD_X: f64 = 0.55;
const COUNT_SIZE: f64 = 28.0;
const SUM_SIZE: f64 = 30.0;
/// The question mark and the think ring, in the card's empty upper right.
const THINK_AT: Vec3 = [3.6, 2.6, 0.0];
/// The ruler's thickness, cm.
const RULER_H: f64 = 0.3;
/// How far the "2 cm" label stands beyond the opened dividers' hinge, cm.
const OPENING_OFF: f64 = 0.4;
/// The 01_6 zoom and the words beside it after the zoom.
const ZOOM: f64 = 2.2;
const ZOOM_WORDS_AT: [Vec3; 2] = [[4.3, 2.75, 0.0], [4.3, 2.15, 0.0]];
const ZOOM_SIZE: f64 = 30.0;

// ------------------------------------------------------------------ timing

const CARD_IN: f64 = 0.6;
const DRAW_S: f64 = 3.0;
const TRACE_S: f64 = 4.0;
const RULER_MOVE: f64 = 1.2;
const OPEN_S: f64 = 0.8;
/// One 2 cm swing. Ten swings with their landings and the fades come to
/// 5.4 s from "walk them", against about 4.1 s for 01_4's last eleven words
/// at the storyboard tool's rate (an estimate; the first render decides).
const SWING_S: f64 = 0.36;
const SMALL_SWING_S: f64 = 0.32;
const LAND_S: f64 = 0.12;
const ZOOM_S: f64 = 1.2;
/// 01_6's hold: the close-up still, before the method changes.
const CLOSE_UP_HOLD: f64 = 2.5;
/// The 2 cm chords' opacity while the 1 cm dividers walk.
const DIMMED: f64 = 0.3;

/// The chapter's words that carry numbers, each from the facts: the card's
/// width, the ends' distance, the dividers' opening, their sum, and the
/// close-up's two lengths. Refused unless the sum on screen adds up.
struct Words {
    card_width: String,
    ends_apart: String,
    opening: String,
    sum: String,
    straight: String,
    of_coast: String,
}

fn words() -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let (steps, opening, last, length) = (
        get("dividers.2_0.steps")?,
        get("dividers.opening_step")?,
        get("dividers.2_0.last")?,
        get("dividers.2_0.length")?,
    );
    let number = |s: &str| s.parse::<f64>().map_err(|e| format!("{s}: {e}"));
    let shown = number(&steps)? * number(&opening)? + number(&last)?;
    if facts::fixed(shown, 1) != length {
        return Err(format!("{steps} x {opening} + {last} does not come to {length}"));
    }
    Ok(Words {
        card_width: format!("{} cm", get("card.width")?),
        ends_apart: format!("{} cm end to end", get("coast.ends_apart")?),
        opening: format!("{opening} cm"),
        sum: format!(r"{steps} \times {opening}\text{{ cm}} + {last}\text{{ cm}} = {length}\text{{ cm}}"),
        straight: format!(r"{opening}\text{{ cm straight}}"),
        of_coast: format!(r"<\; {}\text{{ cm of coast}}", get("dividers.2_0.headland_coast")?),
    })
}

/// Where piece `i`'s label goes, cm: `LABEL_OFF` outside its middle.
fn label_at(coast: &model::Curve, i: usize) -> P2 {
    let (mid, out) = coast.middle_and_outside(i);
    [mid[0] + LABEL_OFF * out[0], mid[1] + LABEL_OFF * out[1]]
}

/// Where the "2 cm" label goes, cm: beyond the hinge of the dividers
/// opened on the first step.
fn opening_label_at(landings: &[P2]) -> P2 {
    let (a, b) = (landings[0], landings[1]);
    let d = (b[0] - a[0]).hypot(b[1] - a[1]);
    let toward = [(b[0] - a[0]) / d, (b[1] - a[1]) / d];
    let h = hinge(a, b, OPENING_STEP, toward);
    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let (ux, uy) = (h[0] - mid[0], h[1] - mid[1]);
    let u = ux.hypot(uy);
    [h[0] + OPENING_OFF * ux / u, h[1] + OPENING_OFF * uy / u]
}

/// The ruler's two poses (01_3), each as (centre in the frame, angle):
/// under the beach, then on the headland's top, tangent there.
fn ruler_poses(coast: &model::Curve) -> Result<[(Vec3, f64); 2], String> {
    let (at, along) = model::ruler_on_beach(coast);
    if along[1].abs() > 1e-12 {
        return Err("the beach must run level for the ruler to lie under it".into());
    }
    let up = [-along[1], along[0]];
    let beach = card_point([at[0] - up[0] * RULER_H / 2.0, at[1] - up[1] * RULER_H / 2.0]);
    let (top, out) = coast.headland_top();
    let head = card_point([top[0] + out[0] * RULER_H / 2.0, top[1] + out[1] * RULER_H / 2.0]);
    Ok([(beach, 0.0), (head, out[1].atan2(out[0]) - std::f64::consts::FRAC_PI_2)])
}

impl SceneConstruct for TheQuestion {
    fn name(&self) -> &str {
        "01_the_question"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let words = words().map_err(fail)?;
        let two = coast.walk(OPENING_STEP);
        let one = coast.walk(SMALLER_STEP);
        let landings: Vec<P2> = two.landings.iter().map(|&(p, _)| p).collect();
        let (headland, _) = headland_step(&coast, &two).map_err(fail)?;
        let poses = ruler_poses(&coast).map_err(fail)?;
        if one.landings.len() < 4 {
            return Err(fail("01_7's dividers need three steps".into()));
        }

        // 01_1: the card, the coast drawn by arc length, its parts named,
        // the question
        let line = say(&kit, stage, "01_1")?;
        let card = card_panel(stage)?;
        let width = kit.text(stage, &words.card_width, LABEL_SIZE, Palette::dim(), WIDTH_LABEL_AT)?;
        fade_in_all(stage, &[card, width], CARD_IN)?;
        let drawn = stage.add_value_tracker(0.0);
        let [pen, shore] = traced(stage, &coast, drawn, palette::coast(), COAST_W)?;
        fade_in_all(stage, &[pen, shore], 0.1)?;
        stage.play(tween(drawn, 1.0, DRAW_S)?)?;
        stop(stage, pen);
        let mut labels = Vec::with_capacity(NAMED.len());
        for (&i, phrase) in NAMED.iter().zip(NAMED_PHRASES) {
            line.wait_for(stage, phrase)?;
            let label = kit.text(
                stage,
                PIECE_NAMES[i],
                LABEL_SIZE,
                Palette::dim(),
                card_point(label_at(&coast, i)),
            )?;
            fade_in_all(stage, &[label], 0.3)?;
            labels.push(label);
        }
        line.wait_for(stage, "How long")?;
        let ask = kit.text(stage, ASK, 60.0, Palette::hero(), THINK_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        labels.push(width);
        fade_out_all(stage, &labels, 0.4)?;

        // 01_2: the length of the line itself, traced; the ends' distance
        let line = say(&kit, stage, "01_2")?;
        fade_out_all(stage, &[ask], 0.3)?;
        line.wait_for(stage, "the length of the line itself")?;
        let trace = stage.add_value_tracker(0.0);
        let [trail_anchor, trail] = traced(stage, &coast, trace, palette::length(), TRACE_W)?;
        let [rider_anchor, runner] = rider(stage, &coast, trace, palette::length())?;
        fade_in_all(stage, &[trail_anchor, trail, rider_anchor, runner], 0.2)?;
        stage.play(tween(trace, 1.0, TRACE_S)?)?;
        stop(stage, trail_anchor);
        stop(stage, rider_anchor);
        line.wait_for(stage, "not the distance")?;
        let (start, end) = coast.ends();
        let between = dashed(stage, start, end, Palette::dim())?;
        let apart = kit.text(stage, &words.ends_apart, LABEL_SIZE, Palette::dim(), v(0.0, BAND_Y))?;
        fade_in_all(stage, &[between, apart], 0.5)?;

        // 01_3: a ruler fits the beach, and touches the headland once
        let line = say(&kit, stage, "01_3")?;
        fade_out_all(stage, &[trail, runner, between, apart], 0.4)?;
        line.wait_for(stage, "fits the beach")?;
        let ruler = Rectangle::new()
            .width(RULER * CARD_UNIT)
            .height(RULER_H * CARD_UNIT)
            .corner_radius(0.03)
            .style(flat(Palette::ink(), 0.8))
            .build()?
            .moved_to(poses[0].0);
        let ruler = stage.arena_mut().add(ruler);
        fade_in_all(stage, &[ruler], 0.4)?;
        line.wait_for(stage, "on the headland")?;
        together(
            stage,
            vec![
                ruler
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(RULER_MOVE),
                        ..AnimateArgs::default()
                    })?
                    .rotate(poses[1].1 - poses[0].1)?
                    .move_to(poses[1].0, ORIGIN)?,
            ],
        )?;
        let (top, _) = coast.headland_top();
        let touch = dot(stage, top, 0.06, palette::touch());
        fade_in_all(stage, &[touch], 0.3)?;

        // 01_4: dividers open to 2 cm and walk the coast, a live count of
        // steps under the card, then the short bit to the end
        let line = say(&kit, stage, "01_4")?;
        fade_out_all(stage, &[ruler, touch], 0.4)?;
        line.wait_for(stage, "Dividers")?;
        let walker = Dividers::new(stage, &landings, OPENING_STEP)?;
        fade_in_all(stage, &[walker.anchor, walker.legs, walker.trail, walker.dots[0]], 0.4)?;
        line.wait_for(stage, "Open them")?;
        stage.play(tween(walker.open, 1.0, OPEN_S)?)?;
        let opened = kit.text(
            stage,
            &words.opening,
            LABEL_SIZE,
            palette::dividers(),
            card_point(opening_label_at(&landings)),
        )?;
        fade_in_all(stage, &[walker.dots[1], opened], 0.3)?;
        line.wait_for(stage, "walk them")?;
        fade_out_all(stage, &[opened], 0.3)?;
        let walk = walker.walk;
        let steps = two.steps() as f64;
        let count = kit.live_value(
            stage,
            move |s: &fmn::mobject::Stage| (s.tracker_value(walk).unwrap_or(0.0).floor() + 1.0).min(steps),
            0,
            false,
            COUNT_SIZE,
            palette::dividers(),
            v(COUNT_RIGHT_X, BAND_Y),
            RIGHT,
        )?;
        // the live count is rebuilt every frame and cannot fade, so its word
        // arrives with it rather than fading in beside it
        let count_word = kit.text(stage, STEPS_WORD, COUNT_SIZE, Palette::dim(), v(STEPS_WORD_X, BAND_Y))?;
        stage.add_to_scene(count_word)?;
        for k in 1..=walker.swings {
            stage.play(tween(walk, k as f64, SWING_S)?)?;
            fade_in_all(stage, &[walker.dots[k + 1]], LAND_S)?;
        }
        let tail = path(stage, &[landings[landings.len() - 1], end], palette::dividers(), 2.5);
        fade_in_all(stage, &[tail], 0.3)?;

        // 01_5: the dividers go, the sum stays; long, short or right?
        say(&kit, stage, "01_5")?;
        fade_out_all(stage, &[walker.anchor, walker.legs], 0.3)?;
        let sum = kit.tex(stage, &words.sum, SUM_SIZE, Palette::ink(), &[], v(0.0, BAND_Y))?;
        swap(stage, &[count, count_word], &[sum], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 01_6: too short, always: zoom onto the step that leaves the
        // headland, its chord and the coast under it
        let line = say(&kit, stage, "01_6")?;
        fade_out_all(stage, &[q, sum], 0.3)?;
        line.wait_for(stage, "Each step goes straight")?;
        let (from, to) = (two.landings[headland - 1], two.landings[headland]);
        let about = card_point([(from.0[0] + to.0[0]) / 2.0, (from.0[1] + to.0[1]) / 2.0]);
        let mut on_card = vec![card, shore, walker.trail, tail];
        on_card.extend(&walker.dots);
        zoom(stage, &on_card, ZOOM, about, ZOOM_S)?;
        let under: Vec<Vec3> = coast
            .between(from.1, to.1, EVERY)
            .into_iter()
            .map(|p| zoomed(p, ZOOM, about))
            .collect();
        let under = stage
            .arena_mut()
            .add(polyline(under).style(line_style(palette::coast(), 9.0)).build());
        let chord = stage.arena_mut().add(
            polyline(vec![zoomed(from.0, ZOOM, about), zoomed(to.0, ZOOM, about)])
                .style(line_style(palette::dividers(), 7.0))
                .build(),
        );
        fade_in_all(stage, &[under, chord], 0.4)?;
        let lengths = [
            kit.tex(
                stage,
                &words.straight,
                ZOOM_SIZE,
                palette::dividers(),
                &[],
                ZOOM_WORDS_AT[0],
            )?,
            kit.tex(
                stage,
                &words.of_coast,
                ZOOM_SIZE,
                palette::coast(),
                &[],
                ZOOM_WORDS_AT[1],
            )?,
        ];
        fade_in_lagged(stage, &lengths, 0.8, 0.5)?;
        kit.hold(stage)?;
        pause(stage, CLOSE_UP_HOLD)?;

        // 01_7: back out; the 2 cm chords dim; dividers closed to 1 cm take
        // three quick steps; then only the card and the coast remain
        let line = say(&kit, stage, "01_7")?;
        let mut close_up = vec![under, chord];
        close_up.extend(lengths);
        fade_out_all(stage, &close_up, 0.3)?;
        zoom(stage, &on_card, 1.0 / ZOOM, about, ZOOM_S)?;
        let mut dim = vec![dim_line(walker.trail, DIMMED, 0.4)?, dim_line(tail, DIMMED, 0.4)?];
        for &d in &walker.dots {
            dim.push(fade_to(d, DIMMED, 0.4)?);
        }
        together(stage, dim)?;
        let small_landings: Vec<P2> = one.landings[..4].iter().map(|&(p, _)| p).collect();
        let small = Dividers::new(stage, &small_landings, SMALLER_STEP)?;
        fade_in_all(stage, &[small.anchor, small.legs, small.trail, small.dots[0]], 0.3)?;
        stage.play(tween(small.open, 1.0, OPEN_S / 2.0)?)?;
        fade_in_all(stage, &[small.dots[1]], LAND_S)?;
        line.wait_for(stage, "needs more steps")?;
        for k in 1..=small.swings {
            stage.play(tween(small.walk, k as f64, SMALL_SWING_S)?)?;
            fade_in_all(stage, &[small.dots[k + 1]], LAND_S)?;
        }
        line.wait_for(stage, "every step still has to follow the coast")?;
        let mut leaving = vec![small.anchor, small.legs, small.trail, walker.trail, tail];
        leaving.extend(&small.dots);
        leaving.extend(&walker.dots);
        fade_out_all(stage, &leaving, 0.5)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use crate::model::{CARD_CENTRE, CARD_H, CARD_W};

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The words on screen say what the model says.
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        let w = words().unwrap();
        assert_eq!(w.card_width, "16 cm");
        assert_eq!(w.ends_apart, "14.7 cm end to end");
        assert_eq!(w.opening, "2 cm");
        assert_eq!(w.sum, r"11 \times 2\text{ cm} + 0.64\text{ cm} = 22.6\text{ cm}");
        assert_eq!(w.straight, r"2\text{ cm straight}");
        assert_eq!(w.of_coast, r"<\; 3.08\text{ cm of coast}");
    }

    /// The true length stays sealed until the draft's 06_4: no word on
    /// screen in chapter 01 gives it away.
    #[test]
    fn the_true_length_is_not_on_screen() {
        let f = facts::facts().unwrap();
        let w = words().unwrap();
        for key in ["coast.length", "coast.length_2dp"] {
            let length = f.get(key).unwrap();
            for text in [
                &w.card_width,
                &w.ends_apart,
                &w.opening,
                &w.sum,
                &w.straight,
                &w.of_coast,
            ] {
                assert!(!text.contains(length), "{text} shows {key}");
            }
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let mut waits: Vec<(&str, &str)> = NAMED_PHRASES.iter().map(|&p| ("01_1", p)).collect();
        waits.extend([
            ("01_1", "How long"),
            ("01_2", "the length of the line itself"),
            ("01_2", "not the distance"),
            ("01_3", "fits the beach"),
            ("01_3", "on the headland"),
            ("01_4", "Dividers"),
            ("01_4", "Open them"),
            ("01_4", "walk them"),
            ("01_6", "Each step goes straight"),
            ("01_7", "needs more steps"),
            ("01_7", "every step still has to follow the coast"),
        ]);
        for (id, phrase) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // the names said are the names labelled, in order
        for (&i, phrase) in NAMED.iter().zip(NAMED_PHRASES) {
            assert!(phrase.ends_with(PIECE_NAMES[i]), "{phrase} labels {}", PIECE_NAMES[i]);
        }
    }

    /// Labels sit clear of the coast and inside the card.
    #[test]
    fn the_labels_sit_clear_of_the_coast() {
        let coast = model::coast();
        let pts = coast.upto(coast.length, 0.02);
        let two = coast.walk(OPENING_STEP);
        let landings: Vec<P2> = two.landings.iter().map(|&(p, _)| p).collect();
        let mut anchors: Vec<(String, P2)> = NAMED
            .iter()
            .map(|&i| (PIECE_NAMES[i].to_owned(), label_at(&coast, i)))
            .collect();
        anchors.push(("2 cm".into(), opening_label_at(&landings)));
        for (name, at) in anchors {
            let near = pts.iter().map(|&p| dist(p, at)).fold(f64::INFINITY, f64::min);
            assert!(near > 0.35, "{name} is {near:.3} cm from the coast");
            assert!(
                at[0].abs() < CARD_W / 2.0 - 0.5 && at[1].abs() < CARD_H / 2.0 - 0.3,
                "{name} at {at:?}"
            );
        }
    }

    /// The think ring, its note and the zoomed close-up's words sit over
    /// empty card: the coast there stays well below them.
    #[test]
    fn the_words_over_the_card_keep_off_the_coast() {
        // how far the zoomed coast keeps under the words, frame units
        const ZOOM_CLEAR: f64 = 0.3;
        let coast = model::coast();
        let note_y = THINK_AT[1] - super::super::draw::THINK_R - 0.3;
        for p in coast.upto(coast.length, 0.02) {
            let q = card_point(p);
            if (q[0] - THINK_AT[0]).abs() < 1.4 {
                assert!(q[1] < note_y - 0.3, "the coast at {q:?} reaches the think note");
            }
        }
        let two = coast.walk(OPENING_STEP);
        let (k, _) = headland_step(&coast, &two).unwrap();
        let (from, to) = (two.landings[k - 1].0, two.landings[k].0);
        let about = card_point([(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0]);
        for p in coast.upto(coast.length, 0.02) {
            let z = zoomed(p, ZOOM, about);
            if (z[0] - ZOOM_WORDS_AT[0][0]).abs() < 2.0 {
                assert!(
                    z[1] < ZOOM_WORDS_AT[1][1] - ZOOM_CLEAR,
                    "the zoomed coast at {z:?} meets the words"
                );
            }
        }
    }

    /// The ruler first lies level under the beach, its top edge on the
    /// beach's line, under the beach's middle; its second pose is tangent to
    /// the headland's top, tilted by the headland's outward direction less a
    /// right angle, its lower edge through the top.
    #[test]
    fn the_ruler_settles_on_the_headland_tilted_seven_and_a_half_degrees() {
        let coast = model::coast();
        let poses = ruler_poses(&coast).unwrap();
        let model::Seg::Line { from, to } = coast.parts[model::BEACH].seg else {
            panic!("the beach is a straight run")
        };
        let beach = card_point([(from[0] + to[0]) / 2.0, from[1]]);
        assert_eq!(poses[0].1, 0.0);
        assert!((poses[0].0[0] - beach[0]).abs() < 1e-12);
        assert!((poses[0].0[1] + CARD_UNIT * RULER_H / 2.0 - beach[1]).abs() < 1e-12);
        assert!(
            (poses[1].1.to_degrees() - 7.5).abs() < 1e-9,
            "{}",
            poses[1].1.to_degrees()
        );
        let (top, out) = coast.headland_top();
        let lower_edge = [
            poses[1].0[0] - out[0] * CARD_UNIT * RULER_H / 2.0,
            poses[1].0[1] - out[1] * CARD_UNIT * RULER_H / 2.0,
        ];
        let top = card_point(top);
        assert!((lower_edge[0] - top[0]).abs() < 1e-12 && (lower_edge[1] - top[1]).abs() < 1e-12);
    }

    /// The layout keeps its pieces inside the frame and the card.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const CARD_TOP: f64 = CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0;
        const CARD_BOTTOM: f64 = CARD_CENTRE[1] - CARD_UNIT * CARD_H / 2.0;
        // the coast reaches 2.02 cm below the card's middle
        const COAST_LOW: f64 = CARD_CENTRE[1] - CARD_UNIT * 2.02;
        const { assert!(CARD_UNIT * CARD_W / 2.0 < FRAME_X_RADIUS - 0.2, "card inside the frame") };
        const { assert!(CARD_TOP < FRAME_Y_RADIUS - 0.1, "card inside the frame") };
        const { assert!(WIDTH_LABEL_AT[1] + 0.15 < CARD_BOTTOM && WIDTH_LABEL_AT[1] - 0.15 > -FRAME_Y_RADIUS) };
        const {
            assert!(
                BAND_Y + 0.25 < COAST_LOW - 0.15 && BAND_Y - 0.25 > CARD_BOTTOM,
                "band under the coast"
            )
        };
        const { assert!(STEPS_WORD_X - 0.45 > COUNT_RIGHT_X + 0.1, "count and word apart") };
        const { assert!(THINK_AT[1] + 0.62 < CARD_TOP - 0.1, "ring on the card") };
        const { assert!(ZOOM_WORDS_AT[0][1] + 0.25 < FRAME_Y_RADIUS && ZOOM_WORDS_AT[0][1] - ZOOM_WORDS_AT[1][1] > 0.45) };
        const {
            assert!(
                ZOOM_WORDS_AT[0][0] + 2.0 < FRAME_X_RADIUS + 0.1,
                "zoom words inside the frame"
            )
        };
        // and the coast keeps above the band, as COAST_LOW takes it
        let coast = model::coast();
        let low = coast
            .upto(coast.length, 0.02)
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min);
        assert!(low >= -2.02, "the coast dips to {low} cm, into the band");
    }
}
