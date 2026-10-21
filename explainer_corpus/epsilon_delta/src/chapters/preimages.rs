//! Preimages need no inverse function. Openness is relative to the domain.
//! The square's preimage is drawn under its curve; "open" is said and
//! shown before the theorem uses it.
use super::draw::{
    BRIGHT_W, curve_color, fade_in_all, fade_in_lagged, fade_out_all, fails, flat, line_style, moving, pieces, say,
    think_pause, wait_until,
};
#[cfg(test)]
use crate::model::Frac;
use explainer_kit::{BACKGROUND, Frame2, Kit, Palette, fade_all, hex, rect, v};
use fmn::prelude::*;

pub struct Preimages {
    pub kit: Kit,
}
pub const ROOTS: [i64; 2] = [1, 3];

// the chapter's fixed words (numbers are built from ROOTS)
const QUESTION: &str = "Must its inputs form a single piece?";
const WHOLE_WINDOW: &str = "each preimage point has a whole input window";
const NO_ROOM: &str = "no interval around zero fits inside";
// what the pictures wait for, each in the line the test below holds it to
const LITTLE_AT: &str = "a little interval";
const AROUND_AT: &str = "every interval around zero";

/// The plot 11_2 to 11_4 draw the square on (scene units): inputs from −3.6
/// to 3.6 along its foot, outputs from 0 to 10 up its middle.
const SQUARE_PLOT: Frame2 = Frame2 {
    center: [-1.6, 0.25, 0.0],
    width: 7.2,
    height: 3.3,
    x: (-3.6, 3.6),
    y: (0.0, 10.0),
};
/// Samples of the drawn curve (a multiple of its pieces' length).
const CURVE_SAMPLES: usize = 80;
/// The think ring of 11_3, right of the plot; the question and, after it,
/// the preimage sit under the plot.
const RING_AT: Vec3 = [4.6, 0.6, 0.0];
const FOOT_Y: [f64; 2] = [-2.45, -3.25];
/// 11_5's strip: the interval (1, 3) again, under the key, and a little
/// interval inside it near its right end.
const STRIP_Y: f64 = -2.9;
const STRIP_SCALE: f64 = 1.6;
const ROOM: [f64; 2] = [2.55, 2.85];
/// 11_7's interval around zero, just over the line, and its half-width.
const AROUND_Y: f64 = -0.15;
const AROUND_R: f64 = 0.6;
pub fn output_bounds() -> [i64; 2] {
    ROOTS.map(|n| n * n)
}
#[cfg(test)]
pub fn square_in_target(x: Frac) -> Result<bool, String> {
    let y = x.times(x)?;
    let [lo, hi] = output_bounds();
    Ok(y > Frac::whole(lo) && y < Frac::whole(hi))
}
fn text(kit: &Kit, s: &mut Stage<'_>, t: &str, y: f64, size: f64) -> fmn::Result<Mob> {
    kit.text(s, t, size, Palette::ink(), v(0.0, y))
}
fn tex(kit: &Kit, s: &mut Stage<'_>, t: &str, y: f64, size: f64, c: Srgb) -> fmn::Result<Mob> {
    kit.tex(s, t, size, c, &[], v(0.0, y))
}
fn line(s: &mut Stage<'_>, a: Vec3, b: Vec3, c: Srgb, w: f64) -> fmn::Result<Mob> {
    Ok(s.arena_mut().add(Line::new(a, b).style(line_style(c, w)).build()?))
}
fn end(s: &mut Stage<'_>, at: Vec3, c: Srgb, closed: bool) -> fmn::Result<Mob> {
    let style = if closed {
        flat(c, 1.0)
    } else {
        Style::default().fill(hex(BACKGROUND), 1.0).stroke(c, 3.0, 1.0)
    };
    Ok(s.arena_mut()
        .add(Dot::new().point(at).radius(0.085).style(style).build()))
}
/// The square's plot, detached: its input axis, its output axis, and
/// y = x² for |x| ≤ √10, as short pieces.
fn square_picture(s: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (x0, x1) = SQUARE_PLOT.x;
    let input_axis = line(s, SQUARE_PLOT.p(x0, 0.0), SQUARE_PLOT.p(x1, 0.0), Palette::dim(), 3.0)?;
    let output_axis = line(
        s,
        SQUARE_PLOT.p(0.0, 0.0),
        SQUARE_PLOT.p(0.0, SQUARE_PLOT.y.1 + 0.4),
        Palette::dim(),
        2.5,
    )?;
    let top = SQUARE_PLOT.y.1.sqrt();
    let pts: Vec<Vec3> = (0..=CURVE_SAMPLES)
        .map(|k| {
            let x = -top + 2.0 * top * k as f64 / CURVE_SAMPLES as f64;
            SQUARE_PLOT.p(x, x * x)
        })
        .collect();
    let curve = s.arena_mut().add(pieces(&pts, curve_color(), BRIGHT_W));
    Ok(vec![input_axis, output_axis, curve])
}
/// 11_5's picture of "open", detached: the interval (1, 3) again on a short
/// line, its ends hollow, and a little interval around a point near its
/// right end that still fits inside.
fn open_strip(kit: &Kit, s: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let [a, b] = ROOTS.map(|n| n as f64);
    let at = |u: f64| v(STRIP_SCALE * (u - f64::midpoint(a, b)), STRIP_Y);
    let mut out = vec![
        line(s, at(a - 0.5), at(b + 0.5), Palette::dim(), 3.0)?,
        line(s, at(a), at(b), Palette::teal(), 9.0)?,
    ];
    for n in ROOTS {
        let p = at(n as f64);
        out.push(end(s, p, Palette::teal(), false)?);
        out.push(kit.tex(s, &n.to_string(), 25.0, Palette::ink(), &[], v(p[0], p[1] - 0.35))?);
    }
    out.push(line(s, at(ROOM[0]), at(ROOM[1]), moving(), 13.0)?);
    Ok(out)
}
impl SceneConstruct for Preimages {
    fn name(&self) -> &str {
        "11_pull_back_open_sets"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [lo, hi] = output_bounds();
        let [a, b] = ROOTS;
        let _line = say(&kit, stage, "11_1")?;
        let title = text(&kit, stage, "Pull back a whole output set", 3.45, 34.0)?;
        let definition = tex(
            &kit,
            stage,
            r"f^{-1}(U)=\{x\in D:f(x)\in U\}",
            2.3,
            36.0,
            Palette::teal(),
        )?;
        let note = text(&kit, stage, "preimage: all inputs that land in U", 1.4, 28.0)?;
        fade_in_lagged(stage, &[title, definition, note], 0.7, 0.1)?;
        kit.hold(stage)?;
        fade_out_all(stage, &[definition, note], 0.4)?;
        let _line = say(&kit, stage, "11_2")?;
        let formula = tex(
            &kit,
            stage,
            &format!(r"f(x)=x^2,\quad D=\mathbb{{R}},\quad U=({lo},{hi})"),
            2.6,
            34.0,
            Palette::ink(),
        )?;
        // the outputs strictly between lo and hi: a band across the plot,
        // its ends hollow on the output axis; nothing about the inputs yet
        let mut picture = vec![formula];
        picture.extend(square_picture(stage)?);
        let (band_lo, band_hi) = (
            SQUARE_PLOT.p(SQUARE_PLOT.x.0, lo as f64),
            SQUARE_PLOT.p(SQUARE_PLOT.x.1, hi as f64),
        );
        let band = rect(
            band_lo,
            band_hi[0] - band_lo[0],
            band_hi[1] - band_lo[1],
            flat(Palette::gold(), 0.18),
        )?;
        picture.push(stage.arena_mut().add(band));
        for y in [lo, hi] {
            let at = SQUARE_PLOT.p(0.0, y as f64);
            picture.push(end(stage, at, Palette::gold(), false)?);
            picture.push(kit.tex(stage, &y.to_string(), 25.0, Palette::ink(), &[], v(at[0] - 0.32, at[1]))?);
        }
        fade_in_lagged(stage, &picture, 0.6, 0.07)?;
        let _question = say(&kit, stage, "11_3")?;
        let ask = kit.text(stage, "?", 44.0, Palette::ink(), RING_AT)?;
        let question = text(&kit, stage, QUESTION, FOOT_Y[0], 28.0)?;
        fade_in_all(stage, &[ask, question], 0.4)?;
        think_pause(&kit, stage, RING_AT, 0.4)?;
        fade_out_all(stage, &[ask, question], 0.3)?;
        // 11_4: where the curve meets the band's edges, down to the inputs;
        // the fades take the 0.3 + 0.5 + 0.5 + 0.6 s they took before, so
        // the card's 7 s reading wait starts where it did
        let _line = say(&kit, stage, "11_4")?;
        let mut drops = Vec::new();
        for n in [-b, -a, a, b] {
            let x = n as f64;
            let drop_line = DashedLine::new(SQUARE_PLOT.p(x, x * x), SQUARE_PLOT.p(x, 0.0))
                .dash_length(0.1)
                .style(line_style(Palette::dim(), 2.5))
                .build()?;
            drops.push(stage.arena_mut().add(drop_line));
        }
        fade_in_all(stage, &drops, 0.3)?;
        for (left, right) in [(-b, -a), (a, b)] {
            let segment = line(
                stage,
                SQUARE_PLOT.p(left as f64, 0.0),
                SQUARE_PLOT.p(right as f64, 0.0),
                Palette::teal(),
                9.0,
            )?;
            let mut group = vec![segment];
            for n in [left, right] {
                let at = SQUARE_PLOT.p(n as f64, 0.0);
                group.push(end(stage, at, Palette::teal(), false)?);
                group.push(kit.tex(stage, &n.to_string(), 25.0, Palette::ink(), &[], v(at[0], at[1] - 0.35))?);
            }
            fade_in_all(stage, &group, 0.5)?;
        }
        let preimage = tex(
            &kit,
            stage,
            &format!(r"f^{{-1}}(({lo},{hi}))=(-{b},-{a})\cup({a},{b})"),
            FOOT_Y[0],
            35.0,
            Palette::teal(),
        )?;
        let endpoints = text(
            &kit,
            stage,
            "endpoints excluded; two open input intervals",
            FOOT_Y[1],
            26.0,
        )?;
        fade_in_lagged(stage, &[preimage, endpoints], 0.6, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 7.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let said = say(&kit, stage, "11_5")?;
        let title2 = text(&kit, stage, "The same local promise, in set language", 3.45, 33.0)?;
        let first = text(
            &kit,
            stage,
            "continuous ⇒ every open output set has an open preimage",
            2.5,
            27.0,
        )?;
        let ball = tex(
            &kit,
            stage,
            r"a\in f^{-1}(U),\quad B_\varepsilon(f(a))\subset U",
            1.4,
            33.0,
            Palette::gold(),
        )?;
        let pull = tex(
            &kit,
            stage,
            r"B_\delta(a)\cap D\ \subset\ f^{-1}(U)",
            0.4,
            36.0,
            Palette::teal(),
        )?;
        let all = text(&kit, stage, WHOLE_WINDOW, -0.45, 26.0)?;
        // what B means, at its first use, under the proof; it stays for the
        // converse, which uses it again
        let key = tex(&kit, stage, r"B_r(c)=(c-r,\,c+r),\quad r>0", -1.9, 28.0, Palette::dim())?;
        fade_in_lagged(stage, &[title2, first, ball, pull, all, key], 0.7, 0.1)?;
        // the card's reading time is counted from here; "open", said first,
        // is drawn under the key inside it
        let read = stage.scene().time().to_f64() + 9.0;
        said.wait_for(stage, LITTLE_AT)?;
        let strip = open_strip(&kit, stage)?;
        fade_in_lagged(stage, &strip, 0.5, 0.1)?;
        wait_until(stage, read)?;
        kit.hold(stage)?;
        let mut proof = vec![first, ball, pull, all];
        proof.extend(strip);
        fade_out_all(stage, &proof, 0.4)?;
        let _line = say(&kit, stage, "11_6")?;
        let converse = text(&kit, stage, "open preimages ⇒ continuity", 2.4, 29.0)?;
        let choose = tex(
            &kit,
            stage,
            r"U=B_\varepsilon(f(a)),\quad a\in f^{-1}(U)",
            1.3,
            34.0,
            Palette::gold(),
        )?;
        let open = tex(
            &kit,
            stage,
            r"f^{-1}(U)\text{ open in }D\ \Longrightarrow\ \exists\delta>0",
            0.3,
            32.0,
            Palette::teal(),
        )?;
        let local = tex(
            &kit,
            stage,
            r"x\in D,\ |x-a|<\delta\ \Longrightarrow\ |f(x)-f(a)|<\varepsilon",
            -0.8,
            31.0,
            Palette::ink(),
        )?;
        fade_in_lagged(stage, &[converse, choose, open, local], 0.7, 0.1)?;
        wait_until(stage, stage.scene().time().to_f64() + 10.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let said = say(&kit, stage, "11_7")?;
        let title3 = text(&kit, stage, "A jump fails this test", 3.3, 34.0)?;
        let step = tex(
            &kit,
            stage,
            r"s(x)=0\ (x<0),\qquad s(x)=1\ (x\ge0)",
            2.1,
            34.0,
            Palette::ink(),
        )?;
        let target = tex(&kit, stage, r"s^{-1}((1/2,3/2))=[0,\infty)", 0.9, 37.0, Palette::gold())?;
        fade_in_lagged(stage, &[title3, step, target], 0.7, 0.1)?;
        let ax = line(stage, v(-5.5, -0.4), v(5.5, -0.4), Palette::dim(), 3.0)?;
        let right = line(stage, v(0.0, -0.4), v(5.5, -0.4), Palette::gold(), 9.0)?;
        let closed = end(stage, v(0.0, -0.4), Palette::gold(), true)?;
        let z = text(&kit, stage, "0", -0.95, 25.0)?;
        let no = text(&kit, stage, NO_ROOM, -2.0, 28.0)?;
        fade_in_lagged(stage, &[ax, right, closed, z, no], 0.6, 0.1)?;
        // the card's 8 s are counted from here, so the interval around zero
        // (its left half lands at 0, outside the target) comes in inside them
        let shown = stage.scene().time().to_f64();
        said.wait_for(stage, AROUND_AT)?;
        let left = line(stage, v(-AROUND_R, AROUND_Y), v(0.0, AROUND_Y), fails(), 7.0)?;
        let right_half = line(stage, v(0.0, AROUND_Y), v(AROUND_R, AROUND_Y), moving(), 7.0)?;
        fade_in_all(stage, &[left, right_half], 0.5)?;
        wait_until(stage, shown + 8.0)?;
        kit.hold(stage)?;
        fade_all(stage, 0.5)?;

        let _line = say(&kit, stage, "11_8")?;
        let domain = text(&kit, stage, "Open in the domain", 3.3, 34.0)?;
        let g = tex(
            &kit,
            stage,
            r"g(x)=\sqrt{x},\qquad D=[0,\infty)",
            2.1,
            35.0,
            Palette::ink(),
        )?;
        let example = tex(
            &kit,
            stage,
            r"g^{-1}((-1,1))=[0,1)\quad\text{open in }D",
            0.9,
            35.0,
            Palette::teal(),
        )?;
        let rel = tex(&kit, stage, r"[0,1)=(-1,1)\cap D", -0.3, 35.0, Palette::blue())?;
        fade_in_lagged(stage, &[domain, g, example, rel], 0.7, 0.1)?;
        let _line = say(&kit, stage, "11_9")?;
        let result = text(&kit, stage, "continuity: preimages of open sets are open", -1.5, 29.0)?;
        let next = text(&kit, stage, "next: what the definition does not supply", -2.7, 26.0)?;
        fade_in_lagged(stage, &[result, next], 0.6, 0.1)?;
        kit.hold(stage)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_intervals_equal_the_square_preimage_including_endpoints() {
        for n in -160..=160 {
            let x = Frac::whole(n).over(Frac::whole(40)).unwrap();
            let actual = square_in_target(x).unwrap();
            let [a, b] = ROOTS;
            let in_intervals =
                (x > Frac::whole(-b) && x < Frac::whole(-a)) || (x > Frac::whole(a) && x < Frac::whole(b));
            assert_eq!(actual, in_intervals, "x={x}");
        }
        for n in [-3, -1, 1, 3] {
            assert!(!square_in_target(Frac::whole(n)).unwrap());
        }
    }
    /// Every phrase a picture waits for is said once, in its line; "open"
    /// is defined in the line that first relies on it.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [("11_5", LITTLE_AT), ("11_7", AROUND_AT)] {
            assert_eq!(script.text(id).unwrap().matches(phrase).count(), 1, "{id}: {phrase}");
        }
        assert!(script.text("11_5").unwrap().starts_with("Open means every point"));
    }
    /// 11_5's little interval fits inside (1, 3), clear of its hollow end.
    #[test]
    fn the_room_drawn_is_inside_the_interval() {
        let [a, b] = ROOTS.map(|n| n as f64);
        assert!(a < ROOM[0] && ROOM[0] < ROOM[1] && ROOM[1] < b);
        // on screen, its right end clears the hollow end mark (radius 0.085)
        // by more than 0.1 scene units
        assert!(STRIP_SCALE * (b - ROOM[1]) > 0.085 + 0.1);
    }
}
