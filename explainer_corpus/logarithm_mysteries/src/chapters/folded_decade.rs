//! Chapter 03, the dial that forgets laps: three decades repeat one pattern
//! (03_1), so one decade bends into a dial where a lap multiplies by ten;
//! 4 × 6 as two turns that pass the top and stop on 2.4's mark, with a lap
//! counter beside it (03_2); the dial laid flat and lifted into a spiral
//! whose points 0.24, 2.4, 24 and 240 share one shadow (03_3); the same
//! picture promised for the complex exponential (03_4).
//!
//! The bend is a real morph: every tick and the spine follow one tracker
//! from the straight decade to the circle (`bend`), and a second tracker lays
//! the circle down into the spiral's floor.
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; picture repairs
//! from the independent review staged in `logarithm-repair01-04`. Type-checked
//! against stand-ins only: not compiled against fmn, unrendered, unlistened.

use std::f64::consts::{FRAC_PI_2, TAU};

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::mobject::Stage as Arena;
use fmn::prelude::*;

use super::draw::{
    Spiral, answer_c, arc_points, dot, fact, fact_f64, fact_list, fade_in_all, fade_in_lagged, fade_out_all,
    faded_line, lap_c, line_style, number_c, position_c, ring, seg, swap, title, value,
};

pub struct FoldedDecade {
    pub kit: Kit,
}

/// The three-decade ruler of 03_1: 1 at `LONG_X`, `DEC` per decade.
const LONG_X: f64 = -6.0;
const DEC: f64 = 4.0;
const LONG_Y: f64 = 1.0;
/// The dial: centre and radius, face-on.
const DIAL_X: f64 = -2.8;
const DIAL_Y: f64 = -0.3;
const R: f64 = 2.1;
/// The floor the dial is laid on in 03_3, and the spiral standing on it.
const FLOOR_Y: f64 = -2.45;
const FLOOR_TILT: f64 = 0.3;
const RISE: f64 = 1.15;
const FLOOR_LAP: f64 = -1.8;
/// The landing's number, out along the dial's ray past the ring and the tick.
const LAND_R: f64 = R + 0.8;
/// The floor's caption, right of the tilted tick's end.
const ONE_MARK_DX: f64 = 0.55;
/// Each spiral reading's number and lap count, right of its point.
const READ_DX: f64 = 0.65;
const LAP_DX: f64 = 1.8;
/// The spiral's caption, under the floor where no lap can cross it.
const KEEP_Y: f64 = -3.55;

/// The point of the decade at fraction `u` (0 at its 1, 1 at its 10), and
/// the outward unit direction there, part way `w` from the straight first
/// decade of the long ruler (w = 0) to the dial (w = 1), clockwise from the
/// top; then laid back by `p` (0 face-on, 1 flat on the floor).
fn bend(u: f64, w: f64, p: f64) -> (Vec3, [f64; 2]) {
    let (sx, sy) = (LONG_X, LONG_Y);
    let (tx, ty) = (DIAL_X, DIAL_Y + R);
    let ax = sx + (tx - sx) * w;
    let ay = sy + (ty - sy) * w;
    let length = DEC + (TAU * R - DEC) * w;
    let (x, y, n) = if w < 1e-6 {
        (ax + length * u, ay, [0.0, 1.0])
    } else {
        let rho = length / (TAU * w);
        let th = TAU * w * u;
        (ax + rho * th.sin(), ay - rho * (1.0 - th.cos()), [th.sin(), th.cos()])
    };
    // laying the dial back squashes it about its centre, which sinks
    let tilt = 1.0 - (1.0 - FLOOR_TILT) * p;
    let cy = DIAL_Y + (FLOOR_Y - DIAL_Y) * p;
    (v(x, cy + (y - DIAL_Y) * tilt), [n[0], n[1] * tilt])
}

/// The dial's spine, its ticks for 1..10 and, once `lit` is above zero, a taller
/// hero tick at the mark `mark` (a fraction of a lap), rebuilt from the trackers.
fn dial_parts(s: &mut Arena, w: f64, p: f64, mark: f64, lit: f64) -> Mob {
    let g = s.add(VMobject::new());
    let spine: Vec<Vec3> = (0..=120).map(|k| bend(f64::from(k) / 120.0, w, p).0).collect();
    let line = s.add(polyline(spine).style(line_style(number_c(), 3.0)).build());
    let _ = s.attach(g, line);
    for n in 1..=10 {
        let (q, d) = bend(f64::from(n).log10(), w, p);
        let tick = s.add(
            polyline(vec![q, v(q[0] + 0.3 * d[0], q[1] + 0.3 * d[1])])
                .style(line_style(number_c(), 2.5))
                .build(),
        );
        let _ = s.attach(g, tick);
    }
    if lit > 1e-3 {
        let (q, d) = bend(mark, w, p);
        let hero = s.add(
            polyline(vec![
                v(q[0] - 0.12 * d[0], q[1] - 0.12 * d[1]),
                v(q[0] + 0.5 * d[0], q[1] + 0.5 * d[1]),
            ])
            .style(faded_line(answer_c(), 4.0, lit))
            .build(),
        );
        let _ = s.attach(g, hero);
    }
    g
}

/// A point on the face-on dial `laps` laps clockwise from the top, at
/// radius `r`.
fn on_dial(laps: f64, r: f64) -> Vec3 {
    let a = TAU * laps;
    v(DIAL_X + r * a.sin(), DIAL_Y + r * a.cos())
}

/// A clockwise arc of the face-on dial from `from` to `to` laps at radius `r`.
fn dial_arc(from: f64, to: f64, r: f64) -> Vec<Vec3> {
    let n = ((to - from).abs() * 90.0).ceil().max(2.0) as usize;
    arc_points(v(DIAL_X, DIAL_Y), r, FRAC_PI_2 - TAU * from, FRAC_PI_2 - TAU * to, n)
}

fn spiral() -> Spiral {
    Spiral {
        centre: v(DIAL_X, FLOOR_Y),
        r: R,
        tilt: FLOOR_TILT,
        rise: RISE,
        floor_lap: FLOOR_LAP,
    }
}

impl SceneConstruct for FoldedDecade {
    fn name(&self) -> &str {
        "03_folded_decade"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, purple) = (number_c(), position_c(), answer_c(), lap_c());
        let mark = fact_f64("dial.mark")?;
        let values = fact_list("dial.values")?;
        let laps = fact_list("dial.laps")?;

        // 03_1: three decades, one pattern; 2.4, 24 and 240 at the same place
        kit.say(stage, "03_1")?;
        let heading = title(&kit, stage, "Every decade looks the same")?;
        let mut long = Vec::new();
        long.push(seg(stage, v(LONG_X, LONG_Y), v(LONG_X + 3.0 * DEC, LONG_Y), green, 3.0));
        for d in 0..3 {
            for n in 1..=9 {
                let x = LONG_X + DEC * (f64::from(d) + f64::from(n).log10());
                long.push(seg(stage, v(x, LONG_Y), v(x, LONG_Y + 0.3), green, 2.5));
            }
        }
        long.push(seg(
            stage,
            v(LONG_X + 3.0 * DEC, LONG_Y),
            v(LONG_X + 3.0 * DEC, LONG_Y + 0.3),
            green,
            2.5,
        ));
        let mut ends = Vec::new();
        for (d, label) in ["1", "10", "100", "1000"].iter().enumerate() {
            ends.push(kit.text(stage, label, 24.0, green, v(LONG_X + DEC * d as f64, LONG_Y - 0.4))?);
        }
        fade_in_all(stage, &[heading], 0.5)?;
        fade_in_all(stage, &long, 0.8)?;
        fade_in_all(stage, &ends, 0.5)?;
        let mut same = Vec::new();
        for d in 0..3 {
            let x = LONG_X + DEC * (d as f64 + mark);
            same.push(ring(stage, v(x, LONG_Y + 0.15), 0.17, gold, 3.0));
            same.push(kit.text(stage, &format!("{}", values[d + 1]), 24.0, gold, v(x, LONG_Y + 0.75))?);
        }
        fade_in_lagged(stage, &same, 1.2, 0.3)?;
        kit.until_sentence(stage, "03_1", 1)?;
        let mut tens = Vec::new();
        for d in 0..2 {
            let a = LONG_X + DEC * (f64::from(d) + mark);
            tens.push(super::draw::arrow(
                stage,
                v(a, LONG_Y - 0.95),
                v(a + DEC, LONG_Y - 0.95),
                blue,
                3.0,
            )?);
            tens.push(kit.text(
                stage,
                "×10 = one decade along",
                20.0,
                blue,
                v(a + DEC / 2.0, LONG_Y - 1.3),
            )?);
        }
        fade_in_lagged(stage, &tens, 1.2, 0.4)?;
        kit.until_sentence(stage, "03_1", 2)?;
        // keep the first decade; it becomes the bending one
        let mut rest: Vec<Mob> = long.clone();
        rest.extend(ends.iter().copied());
        rest.extend(same.iter().copied());
        rest.extend(tens.iter().copied());
        let w_bend = stage.add_value_tracker(0.0);
        let p_lay = stage.add_value_tracker(0.0);
        // the hero tick stays dark until the turn lands on it in 03_2
        let lit_tick = stage.add_value_tracker(0.0);
        let dial = stage.always_redraw(move |s| {
            let (w, p, lit) = (value(s, w_bend), value(s, p_lay), value(s, lit_tick));
            dial_parts(s, w, p, mark, lit)
        });
        stage.add_to_scene(dial)?;
        fade_out_all(stage, &rest, 0.6)?;
        play!(stage; tween(w_bend, 1.0, 2.6)?);
        kit.until_sentence(stage, "03_1", 3)?;
        let mut labels = Vec::new();
        labels.push(kit.text(stage, "1, 10, 100", 24.0, green, on_dial(0.0, R + 0.5))?);
        for n in 2..=9 {
            labels.push(kit.text(
                stage,
                &n.to_string(),
                24.0,
                green,
                on_dial(f64::from(n).log10(), R + 0.45),
            )?);
        }
        let lap_arc = super::draw::path(stage, dial_arc(0.04, 0.96, R + 0.85), purple, 3.0);
        let lap_word = kit.text(stage, "one lap = ×10", 24.0, purple, v(DIAL_X + R + 1.5, DIAL_Y - 1.9))?;
        fade_in_all(stage, &labels, 0.6)?;
        fade_in_all(stage, &[lap_arc, lap_word], 0.6)?;

        // 03_2: multiplying is turning; 4 then 6 pass the top and stop on 2.4
        kit.say(stage, "03_2")?;
        let heading2 = title(&kit, stage, "Multiplying is turning")?;
        swap(stage, &[heading, lap_arc, lap_word], &[heading2], 0.5)?;
        let turn = stage.add_value_tracker(0.0);
        let (first, second) = (fact_f64("slide.pos_c")?, fact_f64("slide.pos_d")?);
        let pointer = stage.always_redraw(move |s| {
            let t = value(s, turn);
            super::draw::arrow_parts(s, v(DIAL_X, DIAL_Y), on_dial(t, 0.82 * R), blue, 4.0, 0.22)
        });
        let trail = stage.always_redraw(move |s| {
            let t = value(s, turn);
            let g = s.add(VMobject::new());
            if t > 1e-3 {
                let a = s.add(
                    polyline(dial_arc(0.0, t.min(first), 0.55 * R))
                        .style(line_style(blue, 5.0))
                        .build(),
                );
                let _ = s.attach(g, a);
            }
            if t > first + 1e-3 {
                let b = s.add(
                    polyline(dial_arc(first, t, 0.42 * R))
                        .style(line_style(blue, 5.0))
                        .build(),
                );
                let _ = s.attach(g, b);
            }
            g
        });
        stage.add_to_scene(pointer)?;
        stage.add_to_scene(trail)?;
        let count_word = kit.text_left(stage, "laps past the top:", 26.0, purple, v(1.4, -1.2))?;
        let count = kit.live_value(
            stage,
            move |s| value(s, turn).floor(),
            0,
            false,
            30.0,
            purple,
            v(3.6, -1.2),
            LEFT,
        )?;
        fade_in_all(stage, &[count_word], 0.4)?;
        kit.until_sentence(stage, "03_2", 1)?;
        let four = kit.text_left(
            stage,
            &format!("×{}: turn {} lap", fact("slide.c")?, fact("slide.pos_c")?),
            26.0,
            blue,
            v(1.4, 1.6),
        )?;
        fade_in_all(stage, &[four], 0.4)?;
        play!(stage; tween(turn, first, 1.6)?);
        kit.until_sentence(stage, "03_2", 2)?;
        let six = kit.text_left(
            stage,
            &format!("×{}: turn {} lap more", fact("slide.d")?, fact("slide.pos_d")?),
            26.0,
            blue,
            v(1.4, 1.0),
        )?;
        fade_in_all(stage, &[six], 0.4)?;
        kit.until_sentence(stage, "03_2", 3)?;
        play!(stage; tween(turn, first + second, 2.4)?);
        let land = ring(stage, on_dial(mark, R + 0.05), 0.2, gold, 3.5);
        let land_word = kit.text(stage, fact("slide.cd_digits")?, 26.0, gold, on_dial(mark, LAND_R))?;
        let total = kit.text_left(
            stage,
            &format!(
                "total {} laps = {} lap + {}",
                fact("slide.pos_cd")?,
                fact("slide.cd_laps")?,
                fact("slide.pos_cd_folded")?
            ),
            26.0,
            Palette::ink(),
            v(1.4, 0.4),
        )?;
        play!(stage; tween(lit_tick, 1.0, 0.3)?);
        fade_in_all(stage, &[land, land_word, total], 0.5)?;
        kit.until_sentence(stage, "03_2", 4)?;
        let ten = kit.text_left(
            stage,
            &format!(
                "{} × {} = 10 × {} = {}",
                fact("slide.c")?,
                fact("slide.d")?,
                fact("slide.cd_digits")?,
                fact("slide.cd")?
            ),
            24.0,
            gold,
            v(1.4, -0.3),
        )?;
        let lap_tag = kit.text_left(stage, "one lap = ×10", 24.0, purple, v(1.4, -1.7))?;
        fade_in_all(stage, &[ten, lap_tag], 0.5)?;

        // 03_3: the dial forgets laps; lay it flat and let the laps rise
        kit.say(stage, "03_3")?;
        let heading3 = title(&kit, stage, "Laps the dial forgets")?;
        let forgets = kit.text_left(
            stage,
            "the dial shows only the mark",
            26.0,
            Palette::dim(),
            v(1.4, -2.3),
        )?;
        swap(stage, &[heading2], &[heading3, forgets], 0.5)?;
        let mut clear = vec![
            pointer, trail, count, count_word, four, six, total, ten, land, land_word, lap_tag,
        ];
        clear.extend(labels.iter().copied());
        fade_out_all(stage, &clear, 0.6)?;
        play!(stage; tween(p_lay, 1.0, 2.0)?);
        kit.until_sentence(stage, "03_3", 1)?;
        fade_out_all(stage, &[forgets], 0.4)?;
        let climb = stage.add_value_tracker(laps[0] - 0.1);
        let helix = stage.always_redraw(move |s| {
            let top = value(s, climb);
            let from = laps_from();
            let pts = if top > from + 1e-3 {
                spiral().points(from, top, 90)
            } else {
                vec![spiral().at(from); 2]
            };
            s.add(polyline(pts).style(line_style(purple, 3.5)).build())
        });
        stage.add_to_scene(helix)?;
        play!(stage; tween(climb, laps[laps.len() - 1] + mark + 0.25, 3.2)?);
        let shadow_at = spiral().shadow(mark);
        let shadow = dot(stage, shadow_at, 0.1, gold);
        let mut points = Vec::new();
        for &k in &laps {
            points.push(dot(stage, spiral().at(k + mark), 0.09, purple));
        }
        // one dashed line from the top point: every lower point sits on it
        let top_point = spiral().at(laps[laps.len() - 1] + mark);
        let drop_line = super::draw::dashed(stage, top_point, shadow_at, Palette::faint(), 2.0, 16)?;
        let one_mark = kit.text_left(
            stage,
            &format!("the {} mark: one place on the dial", fact("slide.cd_digits")?),
            22.0,
            gold,
            v(shadow_at[0] + ONE_MARK_DX, shadow_at[1] - 0.05),
        )?;
        fade_in_all(stage, &points, 0.4)?;
        fade_in_all(stage, &[drop_line], 0.6)?;
        fade_in_all(stage, &[shadow, one_mark], 0.5)?;
        // the readings, lap by lap, as the narration names them
        let reading = |kit: &Kit, stage: &mut Stage<'_>, i: usize| -> fmn::Result<[Mob; 2]> {
            let p = spiral().at(laps[i] + mark);
            let n = kit.text_left(stage, &format!("{}", values[i]), 26.0, green, v(p[0] + READ_DX, p[1]))?;
            let l = laps[i] as i64;
            let word = if l.abs() == 1 { "lap" } else { "laps" };
            let lap = kit.text_left(
                stage,
                &format!("{} {word}", super::draw::shown(&l.to_string())),
                22.0,
                purple,
                v(p[0] + LAP_DX, p[1]),
            )?;
            Ok([n, lap])
        };
        kit.until_sentence(stage, "03_3", 2)?;
        let zero = reading(&kit, stage, 1)?;
        fade_in_all(stage, &zero, 0.4)?;
        kit.until_sentence(stage, "03_3", 3)?;
        let one = reading(&kit, stage, 2)?;
        fade_in_all(stage, &one, 0.4)?;
        kit.until_sentence(stage, "03_3", 4)?;
        let minus = reading(&kit, stage, 0)?;
        let more = reading(&kit, stage, 3)?;
        fade_in_all(stage, &minus, 0.4)?;
        fade_in_all(stage, &[more[0], more[1]], 0.4)?;

        // 03_4: keep the spiral and its shadow; the exponential folds by itself
        kit.say(stage, "03_4")?;
        let keep = kit.text(
            stage,
            "a spiral, and its shadow",
            26.0,
            Palette::ink(),
            v(DIAL_X, KEEP_Y),
        )?;
        fade_in_all(stage, &[keep], 0.4)?;
        kit.until_sentence(stage, "03_4", 1)?;
        // the promise: a vertical column of inputs, a lap apart, one output
        let col_x = 3.3;
        let column = seg(stage, v(col_x, -2.2), v(col_x, 2.4), blue, 3.0);
        let mut inputs = Vec::new();
        for k in 0..4 {
            inputs.push(dot(stage, v(col_x, -1.5 + 1.15 * f64::from(k)), 0.09, purple));
        }
        let out_c = v(5.7, 0.2);
        let circle = ring(stage, out_c, 0.9, Palette::dim(), 2.5);
        let target = v(out_c[0] + 0.9 * 0.7f64.cos(), out_c[1] + 0.9 * 0.7f64.sin());
        let out = dot(stage, target, 0.11, gold);
        let caption = kit.text(stage, "inputs a lap apart, one output", 22.0, purple, v(4.6, -2.65))?;
        fade_in_all(stage, &[column, circle, out, caption], 0.5)?;
        fade_in_lagged(stage, &inputs, 0.8, 0.3)?;
        kit.until_sentence(stage, "03_4", 2)?;
        let mut folds = Vec::new();
        for k in 0..4 {
            let from = v(col_x + 0.12, -1.5 + 1.15 * f64::from(k));
            folds.push(seg(stage, from, target, purple, 1.8));
        }
        fade_in_lagged(stage, &folds, 1.2, 0.25)?;
        kit.until_sentence(stage, "03_4", 3)?;
        let exp = kit.tex(stage, r"e^{x+iy}", 40.0, Palette::ink(), &[], v(4.6, 2.3))?;
        fade_in_all(stage, &[exp], 0.5)?;
        kit.finish(stage)
    }
}

/// Where the spiral starts: a little below the lowest reading's lap.
fn laps_from() -> f64 {
    -1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene code without these tests, for checks on the order of its beats.
    fn scene_source() -> &'static str {
        include_str!("folded_decade.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

    #[test]
    fn the_bend_starts_on_the_long_ruler_and_ends_on_the_dial() {
        for u in [0.0, 0.3, 0.38, 0.77, 1.0] {
            let (a, n) = bend(u, 0.0, 0.0);
            assert!((a[0] - (LONG_X + DEC * u)).abs() < 1e-12 && (a[1] - LONG_Y).abs() < 1e-12);
            assert_eq!(n, [0.0, 1.0]);
            let (b, _) = bend(u, 1.0, 0.0);
            let c = on_dial(u, R);
            assert!((b[0] - c[0]).abs() < 1e-9 && (b[1] - c[1]).abs() < 1e-9, "{u}");
        }
        // nearly straight is continuous with straight
        let (a, _) = bend(0.6, 1e-5, 0.0);
        assert!((a[0] - (LONG_X + DEC * 0.6)).abs() < 1e-3);
    }

    #[test]
    fn four_then_six_stops_on_two_point_four_after_one_lap() {
        let t = 4f64.log10() + 6f64.log10();
        assert_eq!(t.floor(), 1.0);
        assert!((t.fract() - 2.4f64.log10()).abs() < 1e-12);
        let (a, b) = (on_dial(t, R), on_dial(2.4f64.log10(), R));
        assert!((a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9);
    }

    #[test]
    fn the_spiral_readings_share_one_shadow_and_fit_the_frame() {
        let m = 2.4f64.log10();
        let s = spiral();
        let shadow = s.shadow(m);
        let mut last = f64::NEG_INFINITY;
        for k in [-1.0, 0.0, 1.0, 2.0] {
            let p = s.at(k + m);
            assert!((p[0] - shadow[0]).abs() < 1e-9);
            assert!(p[1] > last + 1.0);
            last = p[1];
            assert!(p[1] > FLOOR_Y && p[1] < 3.0, "{k}: {}", p[1]);
        }
        // the whole spiral, floor included, stays under the title
        for p in s.points(laps_from(), 2.0 + 0.25 + m, 90) {
            assert!(p[1] < 3.0 && p[1] > -3.3 && p[0] > -7.1 && p[0] < 7.1);
        }
        // the lowest reading clears the floor ellipse's back edge
        assert!(s.at(-1.0 + m)[1] > shadow[1] + 2.0 * R * FLOOR_TILT);
    }

    #[test]
    fn the_landing_number_clears_the_ring_the_tick_and_the_dial_numerals() {
        let m = 2.4f64.log10();
        // the printed "2.4" reaches at most this far along the ray towards the ring
        let reach = 0.21;
        assert!(LAND_R - reach > R + 0.5 + 0.02, "the tick ends at R + 0.5");
        assert!(LAND_R - reach > R + 0.05 + 0.2 + 0.02, "the ring ends at R + 0.25");
        let at = on_dial(m, LAND_R);
        for n in [2.0f64, 3.0] {
            let other = on_dial(n.log10(), R + 0.45);
            assert!((at[0] - other[0]).hypot(at[1] - other[1]) > 0.6, "{n}");
        }
        assert!(at[1] > -3.85 && at[0] < 6.86);
    }

    #[test]
    fn the_hero_tick_stays_dark_until_the_turn_lands() {
        assert!(found("tween(turn, first + second, 2.4)") < found("tween(lit_tick, 1.0, 0.3)"));
        assert!(found("tween(lit_tick, 1.0, 0.3)") < found("&[land, land_word, total]"));
    }

    #[test]
    fn the_lap_words_leave_with_their_arc_and_return_by_the_counter() {
        let swap_at = found("swap(stage, &[heading, lap_arc, lap_word], &[heading2], 0.5)");
        let tag = found(r#""one lap = ×10", 24.0, purple, v(1.4, -1.7)"#);
        assert!(swap_at < tag);
        // the words leave once only: a second fade would flash them
        assert_eq!(scene_source().matches("lap_word").count(), 3);
    }

    #[test]
    fn the_new_title_and_the_caption_arrive_on_the_first_sentence_of_03_3() {
        let swap_at = found("swap(stage, &[heading2], &[heading3, forgets], 0.5)");
        assert!(swap_at < found("tween(p_lay, 1.0, 2.0)"));
        // "the dial shows only the mark" holds through the lay, until sentence 1
        let hold = found(r#"kit.until_sentence(stage, "03_3", 1)"#);
        assert!(found("tween(p_lay, 1.0, 2.0)") < hold && hold < found("fade_out_all(stage, &[forgets], 0.4)"));
    }

    #[test]
    fn the_right_panel_has_its_caption_while_the_voice_describes_it() {
        let sentence = found(r#"kit.until_sentence(stage, "03_4", 1)"#);
        let next = found(r#"kit.until_sentence(stage, "03_4", 2)"#);
        let shown = found("&[column, circle, out, caption]");
        assert!(sentence < shown && shown < next);
    }

    #[test]
    fn the_spiral_caption_sits_under_the_floor_inside_the_frame() {
        let half = 0.88 * 26.0 * 0.01 / 2.0;
        assert!(
            FLOOR_Y - R * FLOOR_TILT - (KEEP_Y + half) >= 0.15,
            "the floor's lowest edge"
        );
        assert!(KEEP_Y - half >= -3.85);
    }

    #[test]
    fn the_floor_caption_starts_past_the_tilted_tick() {
        let m = 2.4f64.log10();
        let (q, d) = bend(m, 1.0, 1.0);
        let tick_end = q[0] + 0.5 * d[0];
        assert!(spiral().shadow(m)[0] + ONE_MARK_DX >= tick_end + 0.1, "{tick_end}");
    }

    #[test]
    fn every_reading_clears_the_helix() {
        let m = 2.4f64.log10();
        let s = spiral();
        let helix = s.points(laps_from(), 2.0 + 0.25 + m, 400);
        // text boxes: "0.24" is the widest number; 22-point lap words are 1.0 wide
        let half_height = 0.69 * 26.0 * 0.01 / 2.0;
        for k in [-1.0, 0.0, 1.0, 2.0] {
            let p = s.at(k + m);
            let (left, right) = (p[0] + READ_DX, p[0] + LAP_DX + 1.0);
            for q in &helix {
                let dx = (left - q[0]).max(q[0] - right).max(0.0);
                let dy = ((q[1] - p[1]).abs() - half_height).max(0.0);
                assert!(dx.hypot(dy) > 0.05, "{k}: {q:?}");
            }
        }
    }
}
