//! 04_reading_color: paint the input plane by the output's direction, add
//! brightness bands for size and phase lines every twelfth of a turn, then
//! read zeros, their orders and poles off the colours; the chapter ends on
//! the bump's two poles and the radius-1 circle that touches them
//! (storyboard 04_1–04_7). One picture, recoloured in place throughout.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, Walker, carded_lines, dot, fact, fade_in_all, fade_out_all, hero, line_style, marks, names_now,
    pole, precisely_now, predict, reading_window, say, tween_linear, value, wait_until, wheel_legend,
};
use super::gallery::minus;
use super::loop_sums::IDENTITY_TEX;
use super::two_planes::FN_TEX;
use crate::model::{C, Func, ZERO};

pub const DIRECTIONS: [&str; 4] = ["right: red", "up: yellow green", "left: teal", "down: violet"];
pub const BAND: &str = "next band: size × ";
pub const PHASE: &str = "dark lines: every twelfth of a turn";
pub const TURNS: &str = "colour cycles around 0: ";
pub const ORDER_TWO: &str = "zero of order 2";
pub const SIMPLE_POLE: &str = "simple pole";
pub const NEAR_ZERO: &str = "near a zero z = a of order m, f(z) is about c times (z − a) to the power m, so the hue turns m times as z circles a";
pub const ASK: [&str; 2] = ["where are the bump's poles?", "how many turns at each?"];
pub const EACH: &str = "at i: ";
pub const EACH_REST: [&str; 2] = [" turn, at −i: ", " turn"];
pub const RECIPE_RANGE: &str = "gold segment: where the recipe works";
/// The one function this chapter paints that no other chapter's constant already writes.
pub const BUMP_TEX: &str = r"f(z) = \frac{1}{1 + z^2}";

/// The formula's place: over the picture, so what is painted is always named.
pub const FORMULA_AT: (f64, f64) = (-2.6, 3.4);
/// Where a mark's name is carded: under the mark, inside the gold loop.
pub const NAME_AT: C = C::new(0.0, -0.36);
/// The column to the right of the picture, and where the wheel stands in it.
pub const RIGHT: f64 = 4.55;
pub const WHEEL_Y: f64 = 2.2;
/// The wheel legend's radius.
pub const WHEEL_R: f64 = 0.75;
/// How far under the wheel a walk's turns are counted, and the height of the stated count below.
pub const COUNT_BELOW: f64 = 0.35;
pub const STATED_Y: f64 = 0.6;
/// The loop's radius.
pub const LOOP_R: f64 = 0.9;
/// How far around the loop one direction head reaches, radians.
pub const HEAD_SWEEP: f64 = 0.25;

/// The probe of 04_1: its distance from 0, and the direction it starts in.
pub const PROBE_R: f64 = 1.45;
pub const PROBE_ANGLE: f64 = 0.6;
/// The directions the voice names, in the words it names them with.
pub const NAMED: [(&str, f64); 4] = [
    ("Pointing right is red", 0.0),
    ("Pointing up is yellow green", FRAC_PI_2),
    ("Pointing left is teal", PI),
    ("pointing down is violet", 3.0 * FRAC_PI_2),
];

/// Where a walk around the gold loop stands at its share `t`.
pub fn loop_point(t: f64) -> C {
    C::polar(LOOP_R, TAU * t)
}

/// The direction of `f`'s output there: where the wheel's needle points, so that the wedge
/// under the needle is the colour under the walking dot.
pub fn needle_angle(f: Func, t: f64) -> f64 {
    f.eval(loop_point(t)).map_or(0.0, |w| w.arg())
}

/// The picture's window: [−2, 2]², 6.4 units wide, left of centre.
pub fn plane() -> Plane {
    Plane::new(v(-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0))
}

/// A solid head from `a` to `b` in ink, large enough to read as a direction against a gold
/// ring or a coloured picture.
fn solid_head(stage: &mut Stage<'_>, a: Vec3, b: Vec3) -> fmn::Result<Mob> {
    let head = Arrow::new(a, b)
        .buff(0.0)
        .thickness(5.0)
        .max_width_to_length_ratio(0.25)
        .max_tip_length_to_length_ratio(0.8)
        .color(Palette::ink())
        .build()?;
    Ok(stage.arena_mut().add(head))
}

/// The gold loop with three direction heads, around 0 at radius `r`.
fn loop_with_arrow(stage: &mut Stage<'_>, p: &Plane, r: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![p.circle(stage, ZERO, r, hero(), 5.0)?];
    for k in 0..3 {
        let from = FRAC_PI_4 + TAU * f64::from(k) / 3.0;
        out.push(solid_head(
            stage,
            p.p(C::polar(r, from)),
            p.p(C::polar(r, from + HEAD_SWEEP)),
        )?);
    }
    Ok(out)
}

/// The wheel's own direction of travel: a short arc just outside it, ending in the same head
/// the loop carries, so "the same order as the wheel" can be checked by eye.
fn wheel_way(stage: &mut Stage<'_>, at: Vec3, r: f64) -> fmn::Result<Vec<Mob>> {
    let outer = r + 0.13;
    let on = |a: f64| v(at[0] + outer * a.cos(), at[1] + outer * a.sin());
    let arc: Vec<Vec3> = (0..=24).map(|k| on(0.45 + 0.65 * f64::from(k) / 24.0)).collect();
    let tail = stage
        .arena_mut()
        .add(polyline(arc).style(line_style(Palette::ink(), 3.0)).build());
    Ok(vec![tail, solid_head(stage, on(1.1), on(1.5))?])
}

/// The wheel's needle: from its centre to its rim, in direction `th`.
fn needle_points(at: Vec3, r: f64, th: f64) -> Vec<Vec3> {
    vec![at, v(at[0] + r * th.cos(), at[1] + r * th.sin())]
}

/// A walk around the gold loop: a dot on the loop, the wheel's needle on the colour under that
/// dot, and the output's turns counted under the wheel as they happen.
struct LoopWalk {
    walked: Mob,
    anchor: Mob,
    count: Mob,
    mobs: Vec<Mob>,
}

impl LoopWalk {
    /// The dot and the needle arrive at the walk's start; the count is on screen from now.
    fn start(kit: &Kit, stage: &mut Stage<'_>, f: Func, p: Plane, wheel_at: Vec3) -> fmn::Result<Self> {
        let walked = stage.add_value_tracker(0.0);
        let walker = dot(stage, p.p(loop_point(0.0)), 0.09, Palette::ink());
        let needle = stage.arena_mut().add(
            polyline(needle_points(wheel_at, WHEEL_R, needle_angle(f, 0.0)))
                .style(line_style(Palette::ink(), 4.0))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let t = value(a, walked);
                a.move_to(walker, p.p(loop_point(t)), ORIGIN);
                let _ = a.set_points(
                    needle,
                    polyline(needle_points(wheel_at, WHEEL_R, needle_angle(f, t)))
                        .build()
                        .points(),
                );
            },
            true,
        )?;
        let mobs = vec![walker, needle];
        fade_in_all(stage, &mobs, 0.25)?;
        let count = kit.live_value(
            stage,
            move |s| Walker::turns_so_far(f, ZERO, LOOP_R, value(s, walked)),
            1,
            true,
            28.0,
            hero(),
            v(wheel_at[0], wheel_at[1] - WHEEL_R - COUNT_BELOW),
            ORIGIN,
        )?;
        Ok(Self {
            walked,
            anchor,
            count,
            mobs,
        })
    }

    /// Once around from the start, at a steady pace, with the count from zero.
    fn lap(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<()> {
        stage.set_tracker_value(self.walked, 0.0)?;
        stage.play(tween_linear(self.walked, 1.0, rt)?)?;
        Ok(())
    }

    /// Nothing follows the walk any more: what is left for the caller to fade.
    fn stop(self, stage: &mut Stage<'_>) -> Vec<Mob> {
        stage.clear_updaters(self.anchor, true);
        stage.remove_from_scene(self.anchor);
        stage.clear_updaters(self.count, true);
        let mut left = self.mobs;
        left.push(self.count);
        left
    }
}

pub struct ReadingColor {
    pub kit: Kit,
}

impl SceneConstruct for ReadingColor {
    fn name(&self) -> &str {
        "04_reading_color"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = plane();
        let right = RIGHT;
        let wheel_at = v(right, WHEEL_Y);
        let formula_at = v(FORMULA_AT.0, FORMULA_AT.1);

        // 04_1: the wheel legend, then the plane painted with f(z) = z
        let line = say(&kit, stage, "04_1")?;
        let legend = wheel_legend(&kit, stage, wheel_at, WHEEL_R)?;
        let way = wheel_way(stage, wheel_at, WHEEL_R)?;
        fade_in_all(stage, &legend, 0.6)?;
        fade_in_all(stage, &way, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let painter = Painter::build(stage, Some(Func::Identity), p, 0.0)?;
        let border = p.border(stage, Palette::faint())?;
        painter.show(stage, 1.2)?;
        let formula = kit.tex(stage, IDENTITY_TEX, 32.0, Palette::ink(), &[], formula_at)?;
        fade_in_all(stage, &[border, formula], 0.3)?;
        // the rule, shown: a point, its output's direction (for f(z) = z the point's own),
        // and that direction on the wheel, which is the colour the point is painted
        line.wait_sentence(stage, 2)?;
        let aim = stage.add_value_tracker(PROBE_ANGLE);
        let start = C::polar(PROBE_R, PROBE_ANGLE);
        let probe = dot(stage, p.p(start), 0.09, Palette::ink());
        let arm = stage.arena_mut().add(
            polyline(vec![p.p(ZERO), p.p(start)])
                .style(line_style(Palette::ink(), 3.0))
                .build(),
        );
        let needle = stage.arena_mut().add(
            polyline(needle_points(wheel_at, WHEEL_R, PROBE_ANGLE))
                .style(line_style(Palette::ink(), 4.0))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(anchor)?;
        stage.add_updater(
            anchor,
            move |a, _| {
                let th = value(a, aim);
                let z = C::polar(PROBE_R, th);
                a.move_to(probe, p.p(z), ORIGIN);
                let _ = a.set_points(arm, polyline(vec![p.p(ZERO), p.p(z)]).build().points());
                let _ = a.set_points(needle, polyline(needle_points(wheel_at, WHEEL_R, th)).build().points());
            },
            true,
        )?;
        fade_in_all(stage, &[arm, probe, needle], 0.4)?;
        let mut names = Vec::new();
        for (i, d) in DIRECTIONS.iter().enumerate() {
            names.push(kit.text(stage, d, 22.0, Palette::ink(), v(right, 1.0 - 0.38 * i as f64))?);
        }
        for (&m, (phrase, th)) in names.iter().zip(NAMED) {
            line.wait_for(stage, phrase)?;
            stage.play(tween(aim, th, 0.35)?)?;
            fade_in_all(stage, &[m], 0.25)?;
        }

        // 04_2: brightness bands for size, dark lines every twelfth of a turn
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_2")?;
        stage.clear_updaters(anchor, true);
        stage.remove_from_scene(anchor);
        fade_out_all(stage, &[arm, probe, needle], 0.3)?;
        line.wait_sentence(stage, 1)?;
        painter.enhance_to(stage, 1.0, 2.0)?;
        line.wait_sentence(stage, 2)?;
        let band = kit.text(
            stage,
            &format!("{BAND}{}", fact("color.band_ratio")?),
            24.0,
            hero(),
            v(right, -0.75),
        )?;
        fade_in_all(stage, &[band], 0.4)?;
        line.wait_sentence(stage, 3)?;
        let phase = kit.text(stage, PHASE, 22.0, Palette::ink(), v(right, -1.2))?;
        fade_in_all(stage, &[phase], 0.4)?;

        // 04_3: z²: every colour twice around zero, in the wheel's order
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_3")?;
        let mut old = names;
        old.push(formula);
        fade_out_all(stage, &old, 0.3)?;
        // the name comes with what it names, not over the picture it is replacing
        painter.to(stage, Some(Func::Square), 1.2)?;
        let squared = kit.tex(stage, FN_TEX[0], 32.0, Palette::ink(), &[], formula_at)?;
        fade_in_all(stage, &[squared], 0.25)?;
        let ring = loop_with_arrow(stage, &p, LOOP_R)?;
        fade_in_all(stage, &ring, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let count = kit.text(
            stage,
            &format!("{TURNS}{}", fact("color.square.turns")?),
            26.0,
            hero(),
            v(right, STATED_Y),
        )?;
        fade_in_all(stage, &[count], 0.4)?;
        // "in the same order as the wheel" is performed, not left for the eye to work out: the
        // dot walks the loop and the needle runs around the wheel, twice, over the same colours
        line.wait_sentence(stage, 2)?;
        let walk = LoopWalk::start(&kit, stage, Func::Square, p, wheel_at)?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(2.0);
        walk.lap(stage, rt)?;

        // 04_4: the order of a zero, read at a glance
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_4")?;
        let zero = marks(stage, Func::Square, &p);
        let order = carded_lines(
            &kit,
            stage,
            &[ORDER_TWO],
            22.0,
            Palette::ink(),
            p.p(NAME_AT),
            Palette::faint(),
        )?;
        fade_in_all(stage, &zero, 0.3)?;
        // the voice says "cycle in this order" and the walk does it again, around the marked zero
        let rt = line.until(stage, "Once for a simple zero", 2.0)?;
        walk.lap(stage, rt)?;
        line.wait_for(stage, "twice for a zero of order two")?;
        fade_in_all(stage, &order, 0.3)?;
        line.wait_sentence(stage, 2)?;
        let walked = walk.stop(stage);
        fade_out_all(stage, &walked, 0.3)?;
        // the voice is still asking the eye to count colours around the dot, so the card
        // comes only when it has finished, and all of it is read in the window below
        wait_until(stage, line.end())?;
        let exact = precisely_now(&kit, stage, NEAR_ZERO)?;

        // 04_5: 1/z runs the colours backwards around a pole
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "04_5")?;
        let mut old = vec![exact, count, squared];
        old.extend(&order);
        fade_out_all(stage, &old, 0.3)?;
        fade_out_all(stage, &zero, 0.2)?;
        painter.to(stage, Some(Func::Recip), 1.2)?;
        let inverse = kit.tex(stage, FN_TEX[1], 32.0, Palette::ink(), &[], formula_at)?;
        fade_in_all(stage, &[inverse], 0.25)?;
        let pole_mark = marks(stage, Func::Recip, &p);
        let simple = carded_lines(&kit, stage, &[SIMPLE_POLE], 22.0, Palette::ink(), p.p(NAME_AT), pole())?;
        let back = kit.text(
            stage,
            &format!("{TURNS}{}", minus(fact("color.recip.turns")?)),
            26.0,
            pole(),
            v(right, STATED_Y),
        )?;
        line.wait_sentence(stage, 1)?;
        // the mark and what it is called arrive together, so the walk starts while the voice
        // is still on "backwards": once around, and this needle runs against the wheel's order
        let mut named = pole_mark.clone();
        named.extend(&simple);
        named.push(back);
        fade_in_all(stage, &named, 0.3)?;
        let walk = LoopWalk::start(&kit, stage, Func::Recip, p, wheel_at)?;
        let rt = (line.end() - stage.scene().time().to_f64()).max(2.0);
        walk.lap(stage, rt)?;
        let walked = walk.stop(stage);

        // 04_6: before the colours appear, where are the bump's poles?
        kit.hold(stage)?;
        let line = say(&kit, stage, "04_6")?;
        // the size and phase notes describe a picture that has gone; the wheel stays to answer with
        let mut clearing = vec![back, inverse, band, phase];
        clearing.extend(&simple);
        clearing.extend(&ring);
        clearing.extend(&pole_mark);
        clearing.extend(&walked);
        fade_out_all(stage, &clearing, 0.4)?;
        painter.to(stage, None, 0.8)?;
        // the voice names the bump as the picture clears, so the question has its formula
        let bump = kit.tex(stage, BUMP_TEX, 32.0, Palette::ink(), &[], formula_at)?;
        fade_in_all(stage, &[bump], 0.3)?;
        let unit = p.circle(stage, ZERO, 1.0, Palette::faint(), 3.0)?;
        let axes = p.axes(stage)?;
        fade_in_all(stage, &[unit], 0.4)?;
        fade_in_all(stage, &axes, 0.3)?;
        let asked = predict(&kit, stage, &line, &ASK, v(right - 0.15, 0.75))?;

        // 04_7: two poles at ±i, one backwards cycle each; radius 1 touches them
        let line = say(&kit, stage, "04_7")?;
        fade_out_all(stage, &asked, 0.3)?;
        painter.to(stage, Some(Func::Bump), 1.4)?;
        let bump_marks = marks(stage, Func::Bump, &p);
        fade_in_all(stage, &bump_marks, 0.3)?;
        let each = kit.text(
            stage,
            &format!(
                "{EACH}{}{}{}{}",
                minus(fact("color.bump.turns_at_i")?),
                EACH_REST[0],
                minus(fact("color.bump.turns_at_minus_i")?),
                EACH_REST[1]
            ),
            24.0,
            pole(),
            v(right, STATED_Y),
        )?;
        fade_in_all(stage, &[each], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let touching = p.circle(stage, ZERO, 1.0, hero(), 5.0)?;
        let segment = stage.arena_mut().add(
            polyline(vec![p.p(C::real(-1.0)), p.p(C::real(1.0))])
                .style(line_style(hero(), 9.0))
                .build(),
        );
        let range_says = kit.text(stage, RECIPE_RANGE, 20.0, hero(), v(right, -0.2))?;
        // the circle touches the poles: their marks stay on top of it from its first frame, and
        // the faint circle goes only once the gold one covers it, so the place is never empty
        stage.add_to_scene(touching)?;
        stage.add_to_scene(segment)?;
        for &m in &bump_marks {
            stage.add_to_scene(m)?;
        }
        fade_in_all(stage, &[touching, segment, range_says], 0.6)?;
        fade_out_all(stage, &[unit], 0.2)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["domain colouring", "enhanced phase portrait"])?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![border, bump, each, touching, segment, range_says, search];
        all.extend(legend);
        all.extend(way);
        all.extend(axes);
        all.extend(bump_marks);
        fade_out_all(stage, &all, 0.5)?;
        painter.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{I, WHEEL, circle, portrait, wheel, winding_count};

    #[test]
    fn the_window_holds_every_point_the_chapter_marks() {
        let p = plane();
        for z in [
            ZERO,
            I,
            -I,
            C::real(1.0),
            C::real(-1.0),
            C::polar(LOOP_R, FRAC_PI_4 + HEAD_SWEEP),
            C::polar(PROBE_R, PROBE_ANGLE),
        ] {
            assert!(p.contains(z));
        }
        // the picture fits the frame's safe area
        assert!(p.frame.top() <= 3.85 && p.frame.bottom() >= -3.85 && p.frame.left() >= -6.96);
    }

    #[test]
    fn what_the_cards_say_is_what_the_loops_count() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("color.square.turns"), Some("2"));
        assert_eq!(f.get("color.recip.turns"), Some("-1"));
        // the drawn gold loop (radius 0.9) counts the same as the facts' (0.5)
        let sq = winding_count(|z| Func::Square.eval(z), &circle(ZERO, 0.9, 2048));
        let inv = winding_count(|z| Func::Recip.eval(z), &circle(ZERO, 0.9, 2048));
        assert_eq!((sq, inv), (Some(2), Some(-1)));
        // a turn count is printed with a true minus sign
        assert_eq!(minus(f.get("color.bump.turns_at_i").unwrap()), "−1");
        assert_eq!(minus(f.get("color.bump.turns_at_minus_i").unwrap()), "−1");
    }

    /// The probe turns to each direction as the voice names it, and each is the wheel's own
    /// anchor for that direction, so the needle lands on the colour the voice says.
    #[test]
    fn the_probe_turns_where_the_voice_points() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let said = script.text("04_1").unwrap();
        let mut from = 0;
        for (k, (phrase, th)) in NAMED.into_iter().enumerate() {
            let at = said[from..].find(phrase).unwrap_or_else(|| panic!("{phrase}"));
            from += at + phrase.len();
            let anchor = WHEEL[3 * k].map(|c| f64::from(c) / 255.0);
            for (a, b) in wheel(th).into_iter().zip(anchor) {
                assert!((a - b).abs() < 1e-9, "{phrase}");
            }
        }
        assert!(plane().contains(C::polar(PROBE_R, 3.0 * FRAC_PI_2)));
    }

    /// The zero's and pole's cards stand inside the gold loop and clear of the mark above
    /// them. The half sizes are CMR10 widths at 22 plus the card's margins, an estimate: the
    /// pictures decide.
    #[test]
    fn the_cards_clear_the_loop_and_the_mark() {
        let p = plane();
        let (half_w, half_h) = (0.96, 0.3);
        let at = p.p(NAME_AT);
        let c = p.p(ZERO);
        let (dx, dy) = (half_w, (at[1] - c[1]).abs() + half_h);
        assert!((dx * dx + dy * dy).sqrt() < LOOP_R * p.unit() - 0.1);
        assert!((at[1] - c[1]).abs() - half_h > 0.13);
    }

    /// The source of one cue's picture, up to the next cue: read from the source because no
    /// render can run here. What it does to the pixels is for the pictures to show.
    fn beat(id: &str, next: &str) -> &'static str {
        let source = include_str!("reading_color.rs");
        let code = &source[..source.find("#[cfg(test)]\nmod tests").unwrap()];
        let start = code.find(&format!("say(&kit, stage, \"{id}\")")).unwrap();
        let end = start + code[start..].find(next).unwrap();
        &code[start..end]
    }

    fn runs_in_order(section: &str, order: &[&str]) {
        let mut from = 0;
        for piece in order {
            let at = section[from..]
                .find(piece)
                .unwrap_or_else(|| panic!("{piece}: missing or early"));
            from += at + piece.len();
        }
    }

    /// The needle follows the output, not the point: twice around the wheel for z², once
    /// against it for 1/z. And the wedge it stands on is the colour under the walking dot
    /// (`portrait` at enhance 0 is the hue; the bands only change its brightness).
    #[test]
    fn the_needle_stands_on_the_colour_under_the_dot() {
        for (f, turns) in [(Func::Square, 2.0), (Func::Recip, -1.0)] {
            for k in 0..=96 {
                let t = f64::from(k) / 96.0 + 0.003;
                let expected = turns * TAU * t;
                let off = (needle_angle(f, t) - expected).rem_euclid(TAU);
                assert!(off.min(TAU - off) < 1e-9, "{f:?} at {t}: {off}");
                let under_dot = portrait(f.eval(loop_point(t)), 0.0).unwrap();
                let under_needle = wheel(needle_angle(f, t));
                for (a, b) in under_dot.into_iter().zip(under_needle) {
                    assert!((a - b).abs() < 1e-12, "{f:?} at {t}");
                }
            }
        }
        // a lap closes, so a second lap starts where the first ended
        assert!((loop_point(0.0) - loop_point(1.0)).abs() < 1e-12);
    }

    /// What the count reaches on the drawn loop is what the facts and the stated cards say,
    /// and it stands between the wheel and the stated row. The half heights are estimates.
    #[test]
    fn the_walk_counts_what_the_cards_state() {
        let f = crate::facts::facts().unwrap();
        for (func, key) in [(Func::Square, "color.square.turns"), (Func::Recip, "color.recip.turns")] {
            let stated: f64 = f.get(key).unwrap().parse().unwrap();
            let counted = Walker::turns_so_far(func, ZERO, LOOP_R, 1.0);
            assert!((counted - stated).abs() < 1e-9, "{key}: {counted}");
            assert!((Walker::turns_so_far(func, ZERO, LOOP_R, 0.5) - stated / 2.0).abs() < 1e-9);
        }
        let y = WHEEL_Y - WHEEL_R - COUNT_BELOW;
        const { assert!(COUNT_BELOW > 0.2) };
        assert!(y - 0.15 > STATED_Y + 0.15, "the count reaches the stated row");
    }

    /// The cycle is walked on the words that name it, and the walk ends in time for the card
    /// that the reading window protects: nothing moves under its last seconds.
    #[test]
    fn the_cycle_is_walked_on_the_words_that_name_it() {
        runs_in_order(
            beat("04_3", "say(&kit, stage, \"04_4\")"),
            &[
                "painter.to(stage, Some(Func::Square), 1.2)?;",
                "let squared = ",
                "fade_in_all(stage, &ring, 0.5)?;",
                "line.wait_sentence(stage, 2)?;",
                "LoopWalk::start(&kit, stage, Func::Square, p, wheel_at)?;",
                "walk.lap(stage, rt)?;",
            ],
        );
        runs_in_order(
            beat("04_4", "say(&kit, stage, \"04_5\")"),
            &[
                "fade_in_all(stage, &zero, 0.3)?;",
                "line.until(stage, \"Once for a simple zero\", 2.0)?;",
                "walk.lap(stage, rt)?;",
                "line.wait_for(stage, \"twice for a zero of order two\")?;",
                "fade_in_all(stage, &order, 0.3)?;",
                "line.wait_sentence(stage, 2)?;",
                "walk.stop(stage);",
                "fade_out_all(stage, &walked, 0.3)?;",
                "wait_until(stage, line.end())?;",
                "let exact = ",
                "reading_window(&kit, stage, &line)?;",
            ],
        );
        runs_in_order(
            beat("04_5", "say(&kit, stage, \"04_6\")"),
            &[
                "painter.to(stage, Some(Func::Recip), 1.2)?;",
                "let inverse = ",
                "line.wait_sentence(stage, 1)?;",
                "fade_in_all(stage, &named, 0.3)?;",
                "LoopWalk::start(&kit, stage, Func::Recip, p, wheel_at)?;",
                "walk.lap(stage, rt)?;",
                "walk.stop(stage);",
            ],
        );
        let six = beat("04_6", "say(&kit, stage, \"04_7\")");
        runs_in_order(
            six,
            &[
                "clearing.extend(&walked);",
                "fade_out_all(stage, &clearing, 0.4)?;",
                "painter.to(stage, None, 0.8)?;",
                "let bump = ",
                "let asked = predict",
            ],
        );
    }

    /// The gold circle touches the poles: from its first frame the marks stand on it, and the
    /// faint circle under it goes only after it is there.
    #[test]
    fn the_gold_never_crosses_the_marks_and_never_leaves_a_gap() {
        runs_in_order(
            beat("04_7", "kit.music_breath"),
            &[
                "stage.add_to_scene(touching)?;",
                "stage.add_to_scene(segment)?;",
                "for &m in &bump_marks {",
                "stage.add_to_scene(m)?;",
                "fade_in_all(stage, &[touching, segment, range_says], 0.6)?;",
                "fade_out_all(stage, &[unit], 0.2)?;",
            ],
        );
    }

    /// The card names the point it uses: "a zero a" reads as a slip, and 04_4's clean window
    /// (7.7 s at 15 characters a second) holds no more than 115 characters.
    #[test]
    fn the_exact_card_names_its_point_and_fits_its_window() {
        assert!(NEAR_ZERO.contains("zero z = a") && !NEAR_ZERO.contains("zero a "));
        assert!(NEAR_ZERO.chars().count() <= 115, "{}", NEAR_ZERO.chars().count());
    }

    /// The formula stands over the picture, inside the frame.
    #[test]
    fn the_formula_stands_over_the_picture() {
        let half_h = 0.3;
        assert!(FORMULA_AT.1 - half_h > plane().frame.top());
        assert!(FORMULA_AT.1 + half_h < 3.85);
    }
}
