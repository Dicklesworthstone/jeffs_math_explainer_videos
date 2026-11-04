//! 11_counting: walk a loop and count the output's turns around zero; the
//! argument principle and its hypotheses; a zero and a pole that cancel
//! (and why the count is a net total); a loop through a pole has no count;
//! the quintic's one root inside the unit circle by Rouché against −3z; the
//! dog on a leash for the fundamental theorem of algebra; and the
//! programmers' count with a derivative bound, so no turn hides between
//! samples (storyboard 11_1–11_8, root review Mail 622 for 11_3, 11_6, 11_8).
//!
//! Every count the film states comes from `facts` (certified windings); the
//! live counters are the walker's sampled turns, which the tests check end
//! where the facts do.

use std::f64::consts::TAU;

use explainer_kit::props::CodePanel;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, Spoken, Walker, carded, dot, fact, fade_in_all, fade_out_all, faded_line, flat, hero, input,
    line_style, marks, names_now, output, pause, precisely_now, predict, reading_window, restroke, say, squash,
    tween_linear, value, wait_until, walker, warn,
};
use super::loop_sums::STEPS_SAID;
use crate::model::{self, C, Func, ONE, ZERO};

pub const RULE: &str = "turns = zeros − poles inside";
pub const NEEDS: [&str; 3] = [
    "the loop winds once, counterclockwise",
    "it passes through no zero and no pole",
    "f is holomorphic inside, except at poles",
];
pub const LOCAL: [&str; 3] = ["simple zero", "zero of order 2", "simple pole"];
pub const TURNS: &str = "turns: ";
pub const NET: &str = "+1 − 1 = 0";
pub const THROUGH_POLE: &str = "loop through a pole: not allowed";
pub const ASK: [&str; 2] = ["z⁵ − 3z + 1: how many roots", "inside the unit circle?"];
pub const LEASH_TEX: &str = r"|z^5 + 1| \le 2 < 3 = |-3z|";
pub const INSIDE: &str = "one root inside, at distance ";
pub const SQUASHED: &str =
    "output drawn with its direction exact and its size compressed, so every turn around 0 is the function's own";
pub const WALKER_DOG: [&str; 2] = ["walker: z⁵", "dog: z⁵ − 3z + 1"];
/// The output window's labels in 11_8: the sample `a` (as the code names it),
/// zero, and the disk one step can reach.
pub const STEP_SKETCH: [&str; 3] = ["a", "0", "reach L × step"];
/// 11_8's real coarse loop (`model::COARSE_POWER`, tested).
pub const COARSE_TEX: &str = r"f(z) = z^8";
pub const SAMPLES_SAY: [&str; 2] = [" samples say ", " turns"];
pub const TRULY: &str = "between the samples:";
/// The bound the check needs on this loop (tested against the facts).
pub const LIP_TEX: &str = r"L = \max_{|z|=1} |f'(z)| = 8";
pub const REACH_IS: &str = "L × step = ";
pub const REFUSE: &str = ": refuse";
pub const PASSES: &str = ": passes";
pub const REACH_COVERS: &str = "reach covers 0";
pub const CERTAIN: &str = " turns, certain";
pub const CODE: [&str; 6] = [
    "turns = 0",
    "for k in 0..n:",
    "    a = f(loop(k / n)); b = f(loop((k + 1) / n))",
    "    if L * step >= |a|: refuse    # L bounds |f'| on the loop",
    "    turns += angle(b / a)         # in (−π, π]",
    "turns /= 2π                       # zeros − poles inside",
];
/// The code's point size: the longest line (60 columns) fits the panel.
pub const CODE_PT: f64 = 18.0;

/// The input picture's window and the output plane.
pub fn planes() -> (Plane, Plane) {
    (
        Plane::new(v(-3.45, -0.35), 5.6, (-2.0, 2.0), (-2.0, 2.0)),
        Plane::new(v(3.25, -0.35), 5.0, (-1.5, 1.5), (-1.5, 1.5)),
    )
}

/// The radius outputs are squashed into, in output-plane units.
pub const OUT_R: f64 = 1.35;

/// The path of the inset: once around 0 counterclockwise at radius 0.45,
/// then once back clockwise at 0.28 (joined along the positive axis).
pub fn unwinding_path(n: usize) -> Vec<C> {
    let mut out: Vec<C> = (0..=n).map(|k| C::polar(0.45, TAU * k as f64 / n as f64)).collect();
    out.extend((0..=n).map(|k| C::polar(0.28, TAU * (1.0 - k as f64 / n as f64))));
    out.push(C::polar(0.45, 0.0));
    out
}

pub struct Counting {
    pub kit: Kit,
}

impl Counting {
    /// A walker for `f` around `centre` at radius `r`, shown with its loop
    /// and a live counter; walked over `rt`; then the stated count (a fact)
    /// replaces the live one. Returns everything it drew.
    #[allow(clippy::too_many_arguments)]
    fn walk_and_count(
        kit: &Kit,
        stage: &mut Stage<'_>,
        f: Func,
        centre: C,
        r: f64,
        s: f64,
        stated: &str,
        rt: f64,
    ) -> fmn::Result<Vec<Mob>> {
        let (inp, out) = planes();
        let ring = inp.circle(stage, centre, r, hero(), 5.0)?;
        fade_in_all(stage, &[ring], 0.4)?;
        let w = walker(stage, f, centre, r, inp, out, s, OUT_R)?;
        w.show(stage, 0.4)?;
        let counter = w.counter(kit, stage, v(3.25, 3.15))?;
        w.walk(stage, rt)?;
        stage.clear_updaters(counter, true);
        let shown = kit.text(stage, &format!("{TURNS}{stated}"), 34.0, hero(), v(3.25, 3.15))?;
        stage.remove_from_scene(counter);
        stage.add_to_scene(shown)?;
        stage.clear_updaters(w.anchor, true);
        let mut drawn = vec![ring, shown];
        drawn.extend(&w.mobs);
        Ok(drawn)
    }
}

impl SceneConstruct for Counting {
    fn name(&self) -> &str {
        "11_counting"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (inp, out) = planes();
        let out_axes = out.axes(stage)?;
        let out_border = out.border(stage, Palette::faint())?;
        let zero_label = kit.text(stage, "0", 22.0, Palette::dim(), out.p(C::new(-0.12, -0.14)))?;

        // 11_1: around a simple zero one turn, order two twice, a simple pole once back
        let line = say(&kit, stage, "11_1")?;
        let painter = Painter::build(stage, Some(Func::Identity), inp, 1.0)?;
        painter.show(stage, 0.8)?;
        let mut frame = out_axes.clone();
        frame.extend([out_border, zero_label]);
        fade_in_all(stage, &frame, 0.5)?;
        let local: Vec<&str> = fact("argument.local")?.split(',').collect();
        let mut leftover: Vec<Mob> = Vec::new();
        for (i, (f, name)) in [Func::Identity, Func::Square, Func::Recip]
            .into_iter()
            .zip(LOCAL)
            .enumerate()
        {
            line.wait_sentence(stage, i + 2)?;
            if i > 0 {
                fade_out_all(stage, &leftover, 0.3)?;
                painter.to(stage, Some(f), 0.8)?;
            }
            let mark = marks(stage, f, &inp);
            let label = carded(
                &kit,
                stage,
                name,
                24.0,
                Palette::ink(),
                inp.p(C::new(0.0, -1.3)),
                Palette::faint(),
            )?;
            fade_in_all(stage, &mark, 0.2)?;
            fade_in_all(stage, &label, 0.2)?;
            leftover = Self::walk_and_count(&kit, stage, f, ZERO, 0.8, 1.0, local[i], 2.6)?;
            leftover.extend(mark);
            leftover.extend(label);
        }

        // 11_2: the rule and what it needs
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &leftover, 0.3)?;
        painter.to(stage, None, 0.6)?;
        line.wait_sentence(stage, 1)?;
        let rule = carded(&kit, stage, RULE, 34.0, hero(), v(-3.45, 1.0), hero())?;
        fade_in_all(stage, &rule, 0.5)?;
        line.wait_sentence(stage, 3)?;
        let mut needs = Vec::new();
        for (i, n) in NEEDS.iter().enumerate() {
            let c = carded(
                &kit,
                stage,
                n,
                22.0,
                Palette::ink(),
                v(-3.45, -0.2 - 0.62 * i as f64),
                Palette::faint(),
            )?;
            fade_in_all(stage, &c, 0.3)?;
            needs.extend(c);
            pause(stage, 0.4)?;
        }
        let search = names_now(&kit, stage, &["argument principle", "winding number"])?;

        // 11_3: a zero at 0.45 and a pole at 1.35: radius 0.8 counts 1, radius 1.7 counts 0
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_3")?;
        let mut away = rule.to_vec();
        away.extend(needs);
        away.push(search);
        fade_out_all(stage, &away, 0.4)?;
        painter.to(stage, Some(Func::Moebius), 1.0)?;
        let m_marks = marks(stage, Func::Moebius, &inp);
        fade_in_all(stage, &m_marks, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let small = Self::walk_and_count(
            &kit,
            stage,
            Func::Moebius,
            ZERO,
            0.8,
            1.0,
            fact("argument.rational.r0_8")?,
            2.6,
        )?;
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &small, 0.3)?;
        let big = Self::walk_and_count(
            &kit,
            stage,
            Func::Moebius,
            ZERO,
            1.7,
            1.0,
            fact("argument.rational.r1_7")?,
            2.8,
        )?;
        line.wait_sentence(stage, 4)?;
        // the inset: a schematic curve that circles 0 and unwinds, still 0 net
        let inset = Plane::new(v(5.75, 2.35), 1.5, (-0.6, 0.6), (-0.6, 0.6));
        let inset_card = super::draw::card(stage, 1.75, 1.95, v(5.75, 2.25), Palette::faint())?;
        let curve = inset.path(stage, &unwinding_path(90), output(), 3.0)?;
        let origin = dot(stage, inset.p(ZERO), 0.05, Palette::ink());
        let net = kit.text(stage, NET, 22.0, hero(), v(5.75, 1.5))?;
        fade_in_all(stage, &[inset_card, curve, origin, net], 0.5)?;

        // 11_4: radius 1.35 runs through the pole: no count
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_4")?;
        let mut gone = big.clone();
        gone.extend([inset_card, curve, origin, net]);
        fade_out_all(stage, &gone, 0.3)?;
        if fact("argument.rational.r1_35_refused")? != "true" {
            return Err(super::draw::fail("11_4 needs the count at radius 1.35 refused".into()));
        }
        let red_loop = dashed_circle(stage, &inp, model::MOEBIUS_POLE, warn())?;
        let refused = kit.text(stage, THROUGH_POLE, 26.0, warn(), v(3.25, 3.15))?;
        line.wait_sentence(stage, 0)?;
        fade_in_all(stage, &[red_loop], 0.5)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[refused], 0.4)?;

        // 11_5: the quintic; how many roots inside the unit circle?
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_5")?;
        let mut gone = vec![red_loop, refused];
        gone.extend(m_marks);
        fade_out_all(stage, &gone, 0.3)?;
        painter.to(stage, Some(Func::Quintic), 1.2)?;
        let unit = inp.circle(stage, ZERO, 1.0, hero(), 4.0)?;
        fade_in_all(stage, &[unit], 0.4)?;
        let asked = predict(&kit, stage, &line, &ASK, v(3.25, 3.0))?;

        // 11_6: one; the output follows −3z, which turns once
        let line = say(&kit, stage, "11_6")?;
        fade_out_all(stage, &asked, 0.3)?;
        fade_out_all(stage, &[unit], 0.2)?;
        let reference = {
            let mut pts: Vec<Vec3> = (0..=240)
                .map(|k| {
                    out.p(squash(
                        C::polar(3.0, TAU * k as f64 / 240.0) * C::real(-1.0),
                        2.0,
                        OUT_R,
                    ))
                })
                .collect();
            pts.push(pts[0]);
            stage
                .arena_mut()
                .add(polyline(pts).style(faded_line(input(), 3.0, 0.5)).build())
        };
        fade_in_all(stage, &[reference], 0.4)?;
        let one = Self::walk_and_count(
            &kit,
            stage,
            Func::Quintic,
            ZERO,
            1.0,
            2.0,
            fact("argument.quintic.unit")?,
            3.0,
        )?;
        line.wait_sentence(stage, 2)?;
        let leash = kit.tex(stage, LEASH_TEX, 34.0, Palette::ink(), &[], v(-3.45, 3.1))?;
        fade_in_all(stage, &[leash], 0.4)?;
        let roots = marks(stage, Func::Quintic, &inp);
        fade_in_all(stage, &roots, 0.4)?;
        let inside = carded(
            &kit,
            stage,
            &format!("{INSIDE}{}", fact("argument.quintic.smallest")?),
            24.0,
            Palette::ink(),
            inp.p(C::new(0.0, -1.6)),
            Palette::faint(),
        )?;
        fade_in_all(stage, &inside, 0.3)?;
        // how the output is drawn has nothing to do with the minus three z the voice is on,
        // so the card comes only when it has finished
        wait_until(stage, line.end())?;
        let squashed = precisely_now(&kit, stage, SQUASHED)?;

        // 11_7: the dog on a leash: on |z| = 3 both circle 5 times
        reading_window(&kit, stage, &line)?;
        let line = say(&kit, stage, "11_7")?;
        let mut gone = one.clone();
        gone.extend([reference, leash, squashed]);
        gone.extend(inside);
        gone.extend(roots);
        fade_out_all(stage, &gone, 0.3)?;
        painter.leave(stage, 0.4)?;
        let wide = Plane::new(v(-3.45, -0.35), 5.6, (-3.6, 3.6), (-3.6, 3.6));
        let wide_axes = wide.axes(stage)?;
        let big_loop = wide.circle(stage, ZERO, 3.0, hero(), 5.0)?;
        let mut wide_all = wide_axes.clone();
        wide_all.push(big_loop);
        line.wait_sentence(stage, 2)?;
        fade_in_all(stage, &wide_all, 0.5)?;
        let dog = DogWalk::build(&kit, stage, wide, out)?;
        line.wait_sentence(stage, 3)?;
        dog.show(stage)?;
        let rt = (line.sentence(5)? - stage.scene().time().to_f64()).max(5.0);
        stage.play(tween_linear(dog.walked, 1.0, rt)?)?;
        stage.clear_updaters(dog.anchor, true);
        stage.clear_updaters(dog.counter, true);
        let five = kit.text(
            stage,
            &format!("{TURNS}{}", fact("argument.quintic.big")?),
            34.0,
            hero(),
            v(3.25, 3.15),
        )?;
        stage.remove_from_scene(dog.counter);
        stage.add_to_scene(five)?;
        let search = names_now(&kit, stage, &["fundamental theorem of algebra", "Rouché's theorem"])?;

        // 11_8: for programmers: the count with a derivative bound
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_8")?;
        let mut gone = dog.mobs.clone();
        gone.extend([five, search, big_loop]);
        gone.extend(wide_axes);
        gone.extend(out_axes.iter().copied());
        gone.extend([out_border, zero_label]);
        fade_out_all(stage, &gone, 0.4)?;
        let panel = CodePanel::build(&kit, stage, &CODE, CODE_PT, 7.6)?;
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.8)?)?;
        let mut last = coarse_loop(&kit, stage, &line, panel.lines[REFUSE_LINE])?;
        wait_until(stage, line.end())?;
        // the panel has shared the screen with the running check for the whole cue; now the
        // check has stopped, so the code can be read against what the voice just described
        reading_window(&kit, stage, &line)?;
        kit.music_breath(stage, 2.0)?;
        last.push(panel.group);
        fade_out_all(stage, &last, 0.5)?;
        kit.finish(stage)
    }
}

/// The code line holding the check that refuses.
const REFUSE_LINE: usize = 3;

/// 11_8's input window, then its output window, left of the code panel.
fn coarse_planes() -> (Plane, Plane) {
    (
        Plane::new(v(-5.45, 0.55), 2.8, (-1.4, 1.4), (-1.4, 1.4)),
        Plane::new(v(-2.55, 0.55), 2.8, (-2.0, 2.0), (-2.0, 2.0)),
    )
}

/// "f(z) = z⁸", over the two windows.
const COARSE_TITLE_AT: (f64, f64) = (-4.0, 2.4);
/// The rows under the windows, left-aligned at `COARSE_ROW_X`: what the
/// samples say, what they missed, the bound, then the test.
const COARSE_ROW_X: f64 = -6.85;
const COARSE_ROWS: [f64; 4] = [-1.2, -1.62, -2.05, -2.5];
/// The certified count, below the code panel (y within ±1.02 once in place).
const CERTAIN_AT: (f64, f64) = (2.96, -1.85);

/// A tinted band over one line of the code.
fn line_band(stage: &mut Stage<'_>, m: Mob, color: Srgb) -> Mob {
    let c = stage.get_center(m);
    let half = stage.get_width(m) / 2.0 + 0.1;
    let band = vec![
        v(c[0] - half, c[1] - 0.14),
        v(c[0] + half, c[1] - 0.14),
        v(c[0] + half, c[1] + 0.14),
        v(c[0] - half, c[1] + 0.14),
    ];
    stage
        .arena_mut()
        .add(Polygon::new(band).style(flat(color, 0.3)).build())
}

/// The check of 11_8 run on a real loop, z⁸ around the unit circle, in step
/// with sentences 2–5 of `line`. Eight samples all land on 1 and read no
/// turns; between them the output really goes round eight times. With
/// L = 8, one step's reach is far more than the distance from `a` to zero, so
/// the check refuses (the reach covers the whole window). With 64 steps the
/// reach is a disk that misses zero, the check passes and the count is 8.
/// Returns everything it left on screen.
fn coarse_loop(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken, refuse: Mob) -> fmn::Result<Vec<Mob>> {
    let power = model::COARSE_POWER;
    let f = move |z: C| z.powi(power);
    let (inp, out) = coarse_planes();
    let steps: Vec<&str> = fact("argument.coarse.steps")?.split(',').collect();
    let reaches: Vec<&str> = fact("argument.coarse.reach")?.split(',').collect();
    let (&[few, enough], &[reach_few, reach_enough]) = (steps.as_slice(), reaches.as_slice()) else {
        return Err(super::draw::fail("11_8 needs two step counts and two reaches".into()));
    };
    // the disk is drawn at the true reach; the rows print it rounded
    let reach_fine = f64::from(power) * TAU / model::COARSE_STEPS[1] as f64;
    if fact("argument.coarse.refused")? != "true" {
        return Err(super::draw::fail("11_8 needs the coarse count refused".into()));
    }
    let size = fact("argument.coarse.size")?;

    // "Walk the loop in small steps": the eight samples, all landing on one point
    line.wait_sentence(stage, 2)?;
    let mut frame = inp.axes(stage)?;
    frame.extend(out.axes(stage)?);
    frame.push(inp.border(stage, Palette::faint())?);
    frame.push(out.border(stage, Palette::faint())?);
    frame.push(inp.circle(stage, ZERO, 1.0, input(), 3.0)?);
    frame.push(kit.tex(
        stage,
        COARSE_TEX,
        28.0,
        Palette::ink(),
        &[],
        v(COARSE_TITLE_AT.0, COARSE_TITLE_AT.1),
    )?);
    frame.push(kit.text(stage, STEP_SKETCH[1], 20.0, Palette::ink(), out.p(C::new(-0.25, -0.3)))?);
    let image = dot(stage, out.p(f(ONE)), 0.08, output());
    frame.push(image);
    fade_in_all(stage, &frame, 0.4)?;
    let samples = model::circle(ZERO, 1.0, model::COARSE_STEPS[0]);
    let each = line.until(stage, "But samples", 2.0)? / samples.len() as f64;
    let mut left = frame;
    for &z in &samples {
        let d = dot(stage, inp.p(z), 0.07, hero());
        fade_in_all(stage, &[d], 0.1)?;
        pause(stage, (each - 0.1).max(0.04))?;
        left.push(d);
    }
    let say_row = kit.text_left(
        stage,
        &format!(
            "{few}{}{}{}",
            SAMPLES_SAY[0],
            fact("argument.coarse.naive")?,
            SAMPLES_SAY[1]
        ),
        24.0,
        Palette::ink(),
        v(COARSE_ROW_X, COARSE_ROWS[0]),
    )?;
    fade_in_all(stage, &[say_row], 0.3)?;
    left.push(say_row);

    // "samples alone can miss a turn": one whole turn hides in every step
    line.wait_sentence(stage, 3)?;
    let walked = stage.add_value_tracker(0.0);
    let walker_in = dot(stage, inp.p(ONE), 0.08, Palette::ink());
    let walker_out = dot(stage, out.p(f(ONE)), 0.08, Palette::ink());
    let trail = stage.arena_mut().add(
        polyline(vec![out.p(f(ONE)), out.p(f(ONE))])
            .style(faded_line(output(), 3.0, 0.9))
            .build(),
    );
    left.push(trail);
    let truly = kit.text_left(stage, TRULY, 24.0, hero(), v(COARSE_ROW_X, COARSE_ROWS[1]))?;
    left.push(truly);
    fade_in_all(stage, &[walker_in, walker_out, truly], 0.2)?;
    stage.add_to_scene(trail)?;
    let count_x = COARSE_ROW_X + stage.get_width(truly) + 0.12;
    let count = kit.live_value(
        stage,
        move |s| (f64::from(power) * value(s, walked)).floor(),
        0,
        false,
        24.0,
        hero(),
        v(count_x, COARSE_ROWS[1]),
        LEFT,
    )?;
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let t = value(a, walked).clamp(0.0, 1.0);
            let z = C::polar(1.0, TAU * t);
            a.move_to(walker_in, inp.p(z), ORIGIN);
            a.move_to(walker_out, out.p(f(z)), ORIGIN);
            let k = ((t * 960.0).round() as usize).max(1);
            let pts: Vec<Vec3> = (0..=k)
                .map(|j| out.p(f(C::polar(1.0, TAU * j as f64 / 960.0))))
                .collect();
            let _ = a.set_points(trail, polyline(pts).build().points());
        },
        true,
    )?;
    let rt = line.until(stage, "So bound", 1.8)?;
    stage.play(tween_linear(walked, 1.0, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    stage.clear_updaters(count, true);
    let settled = kit.text_left(
        stage,
        &format!("{}{}", fact("argument.coarse.certified")?, SAMPLES_SAY[1]),
        24.0,
        hero(),
        v(count_x, COARSE_ROWS[1]),
    )?;
    stage.remove_from_scene(count);
    stage.add_to_scene(settled)?;
    left.push(settled);

    // "So bound how fast f can change": the coarse step fails the check; its
    // reach is bigger than this window, so the window itself is shaded
    line.wait_sentence(stage, 4)?;
    fade_out_all(stage, &[walker_in, walker_out], 0.2)?;
    restroke(stage, trail, Palette::dim(), 0.5);
    let lip = kit.tex(
        stage,
        LIP_TEX,
        24.0,
        Palette::ink(),
        &[],
        v(COARSE_ROW_X, COARSE_ROWS[2]),
    )?;
    stage.move_to(lip, v(COARSE_ROW_X, COARSE_ROWS[2]), LEFT);
    left.push(lip);
    fade_in_all(stage, &[lip], 0.3)?;
    let w = out.frame;
    let covered = stage.arena_mut().add(
        Polygon::new(vec![
            w.p(w.x.0, w.y.0),
            w.p(w.x.1, w.y.0),
            w.p(w.x.1, w.y.1),
            w.p(w.x.0, w.y.1),
        ])
        .style(flat(warn(), 0.22))
        .build(),
    );
    let covers = kit.text(stage, REACH_COVERS, 18.0, warn(), out.p(C::new(0.0, 1.65)))?;
    let refused = kit.text_left(
        stage,
        &format!("{REACH_IS}{reach_few} ≥ {size}{REFUSE}"),
        24.0,
        warn(),
        v(COARSE_ROW_X, COARSE_ROWS[3]),
    )?;
    let band = line_band(stage, refuse, warn());
    fade_in_all(stage, &[covered, covers, refused, band], 0.4)?;

    // "take steps so small": the fine samples; one step's reach misses zero
    line.wait_for(stage, "take steps")?;
    fade_out_all(stage, &[covered, covers, refused, band], 0.3)?;
    let fine = model::circle(ZERO, 1.0, model::COARSE_STEPS[1]);
    let mut fine_dots = Vec::with_capacity(2 * fine.len());
    for &z in &fine {
        fine_dots.push(dot(stage, inp.p(z), 0.04, hero()));
        fine_dots.push(dot(stage, out.p(f(z)), 0.05, output()));
    }
    fade_in_all(stage, &fine_dots, 0.4)?;
    left.extend(fine_dots);
    let disk = Circle::new()
        .radius(reach_fine * out.unit())
        .style(faded_line(derivative_teal(), 3.5, 0.95))
        .build()
        .moved_to(out.p(f(ONE)));
    let disk = stage.arena_mut().add(disk);
    let a_name = kit.text(
        stage,
        STEP_SKETCH[0],
        20.0,
        output(),
        out.p(f(ONE) + C::new(-0.28, 0.2)),
    )?;
    let reach_name = kit.text(
        stage,
        STEP_SKETCH[2],
        18.0,
        derivative_teal(),
        out.p(C::new(0.55, -1.2)),
    )?;
    let passes = kit.text_left(
        stage,
        &format!("{REACH_IS}{reach_enough} < {size}{PASSES}"),
        24.0,
        derivative_teal(),
        v(COARSE_ROW_X, COARSE_ROWS[3]),
    )?;
    let pass_band = line_band(stage, refuse, derivative_teal());
    let lit = [disk, a_name, reach_name, passes, pass_band];
    fade_in_all(stage, &lit, 0.4)?;
    left.extend(lit);

    // "the count is certain"
    line.wait_sentence(stage, 5)?;
    let certain = carded(
        kit,
        stage,
        &format!("{enough}{STEPS_SAID}{}{CERTAIN}", fact("argument.coarse.certified")?),
        28.0,
        hero(),
        v(CERTAIN_AT.0, CERTAIN_AT.1),
        hero(),
    )?;
    fade_in_all(stage, &certain, 0.4)?;
    left.extend(certain);
    Ok(left)
}

/// A dashed circle `|z| = r` in `color` (dashes as separate short arcs).
fn dashed_circle(stage: &mut Stage<'_>, p: &Plane, r: f64, color: Srgb) -> fmn::Result<Mob> {
    let dashes = 48;
    let mut parts = Vec::with_capacity(dashes);
    for k in 0..dashes {
        let (a0, a1) = (TAU * k as f64 / dashes as f64, TAU * (k as f64 + 0.55) / dashes as f64);
        let pts: Vec<Vec3> = (0..=6)
            .map(|j| p.p(C::polar(r, a0 + (a1 - a0) * f64::from(j) / 6.0)))
            .collect();
        parts.push(
            stage
                .arena_mut()
                .add(polyline(pts).style(line_style(color, 5.0)).build()),
        );
    }
    super::draw::group(stage, &parts)
}

fn derivative_teal() -> Srgb {
    super::draw::derivative()
}

/// The dog on a leash (11_7): on |z| = 3 the walker z⁵ (blue) and the
/// polynomial (green) in the output plane, joined by the leash, with the
/// dog's trail and a live count of the dog's turns.
struct DogWalk {
    walked: Mob,
    anchor: Mob,
    counter: Mob,
    mobs: Vec<Mob>,
}

/// Output squash for the big circle: |z⁵| = 243 there.
const DOG_S: f64 = 200.0;

impl DogWalk {
    fn build(kit: &Kit, stage: &mut Stage<'_>, wide: Plane, out: Plane) -> fmn::Result<Self> {
        let walked = stage.add_value_tracker(0.0);
        let r = 3.0;
        let at = move |t: f64| C::polar(r, TAU * t);
        let w = |z: C| squash(z.powi(5), DOG_S, OUT_R);
        let d = |z: C| squash(Func::Quintic.eval(z).unwrap_or(ZERO), DOG_S, OUT_R);
        let z0 = at(0.0);
        let in_dot = dot(stage, wide.p(z0), 0.08, input());
        let walker_dot = dot(stage, out.p(w(z0)), 0.09, input());
        let dog_dot = dot(stage, out.p(d(z0)), 0.08, output());
        let leash = stage.arena_mut().add(
            polyline(vec![out.p(w(z0)), out.p(d(z0))])
                .style(line_style(Palette::ink(), 2.5))
                .build(),
        );
        let trail = stage.arena_mut().add(
            polyline(vec![out.p(d(z0)), out.p(d(z0))])
                .style(faded_line(output(), 3.0, 0.8))
                .build(),
        );
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = value(s, walked).clamp(0.0, 1.0);
                if t == last {
                    return;
                }
                let z = at(t);
                s.move_to(in_dot, wide.p(z), ORIGIN);
                s.move_to(walker_dot, out.p(w(z)), ORIGIN);
                s.move_to(dog_dot, out.p(d(z)), ORIGIN);
                let _ = s.set_points(leash, polyline(vec![out.p(w(z)), out.p(d(z))]).build().points());
                let k = ((t * 1440.0).round() as usize).max(1);
                let pts: Vec<Vec3> = (0..=k).map(|j| out.p(d(at(j as f64 / 1440.0)))).collect();
                let _ = s.set_points(trail, polyline(pts).build().points());
                last = t;
            },
            true,
        )?;
        let counter = kit.live_value(
            stage,
            move |s| Walker::turns_so_far(Func::Quintic, ZERO, 3.0, value(s, walked)),
            1,
            true,
            34.0,
            hero(),
            v(3.25, 3.15),
            ORIGIN,
        )?;
        let names = [
            kit.text(stage, WALKER_DOG[0], 22.0, input(), v(1.75, 2.5))?,
            kit.text(stage, WALKER_DOG[1], 22.0, output(), v(4.75, 2.5))?,
        ];
        let mut mobs = vec![trail, leash, in_dot, walker_dot, dog_dot];
        mobs.extend(names);
        Ok(Self {
            walked,
            anchor,
            counter,
            mobs,
        })
    }

    fn show(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add_to_scene(self.anchor)?;
        fade_in_all(stage, &self.mobs, 0.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{circle, winding_certified, winding_count};

    #[test]
    fn the_leash_formula_holds_on_the_unit_circle() {
        // 11_6's card: |z⁵ + 1| ≤ 2 < 3 = |−3z|, and 2 is reached, at z = 1
        assert!(LEASH_TEX.contains(r"\le 2 < 3 ="));
        let mut longest = 0.0_f64;
        for z in circle(ZERO, 1.0, 400) {
            let rest = (z.powi(5) + ONE).abs();
            assert!(rest <= 2.0 + 1e-12);
            longest = longest.max(rest);
            assert!((z.scale(-3.0).abs() - 3.0).abs() < 1e-12);
        }
        assert!((longest - 2.0).abs() < 1e-12);
    }

    #[test]
    fn the_inset_circles_zero_and_unwinds_to_a_net_zero() {
        let path = unwinding_path(90);
        assert_eq!(winding_count(Some, &path), Some(0));
        // it does make a full turn on the way: the angle reaches 2π before unwinding
        let mut total: f64 = 0.0;
        let mut most: f64 = 0.0;
        for k in 1..=91 {
            total += (path[k] / path[k - 1]).arg();
            most = most.max(total);
        }
        assert!(most > TAU - 1e-9);
    }

    #[test]
    fn the_leash_is_shorter_than_the_walkers_distance_on_the_big_circle() {
        for z in circle(ZERO, 3.0, 2000) {
            let p = Func::Quintic.eval(z).unwrap();
            assert!((p - z.powi(5)).abs() < z.powi(5).abs());
        }
    }

    #[test]
    fn minus_three_z_dominates_on_the_unit_circle_and_turns_once() {
        for z in circle(ZERO, 1.0, 2000) {
            assert!((z.powi(5) + C::real(1.0)).abs() <= 2.0 + 1e-12);
            assert!((z.scale(-3.0)).abs() > 2.0);
        }
        assert_eq!(
            winding_count(|z: C| Some(z.scale(-3.0)), &circle(ZERO, 1.0, 512)),
            Some(1)
        );
    }

    #[test]
    fn the_programmers_code_names_the_bound() {
        assert!(CODE.iter().any(|l| l.contains("L * step >= |a|")));
        assert!(CODE.iter().all(|l| l.chars().count() <= 64), "code lines fit the panel");
    }

    #[test]
    fn the_coarse_loop_is_refused_and_the_finer_one_certified() {
        let (_, out) = coarse_planes();
        let power = model::COARSE_POWER;
        let f = |z: C| Some(z.powi(power));
        let l = f64::from(power);
        let [few, enough] = model::COARSE_STEPS;
        // L is the true bound: |f′| = 8|z|⁷ is 8 all round the unit circle
        for z in circle(ZERO, 1.0, 360) {
            let derivative = z.powi(power - 1).scale(l);
            assert!((derivative.abs() - l).abs() < 1e-12);
        }
        // few steps: every sample lands on 1, so the samples read no turns
        for z in circle(ZERO, 1.0, few) {
            assert!((z.powi(power) - ONE).abs() < 1e-12);
        }
        assert_eq!(winding_count(f, &circle(ZERO, 1.0, few)), Some(0));
        assert_eq!(winding_certified(f, ZERO, 1.0, few, l), None);
        // one step's reach covers the whole output window, zero with it
        let reach = l * TAU / few as f64;
        let w = out.frame;
        for corner in [
            C::new(w.x.0, w.y.0),
            C::new(w.x.1, w.y.0),
            C::new(w.x.1, w.y.1),
            C::new(w.x.0, w.y.1),
        ] {
            assert!((corner - ONE).abs() < reach);
        }
        // enough steps: the reach is a disk inside the window that misses zero
        let reach = l * TAU / enough as f64;
        assert!(reach < ONE.abs());
        assert!(1.0 + reach < w.x.1 && reach < w.y.1);
        assert_eq!(winding_certified(f, ZERO, 1.0, enough, l), Some(i64::from(power)));
    }

    #[test]
    fn the_coarse_formulas_name_the_loop() {
        assert_eq!(fact("argument.coarse.power").ok(), Some("8"));
        assert_eq!(fact("argument.coarse.lipschitz").ok(), Some("8"));
        assert!(COARSE_TEX.contains("z^8") && LIP_TEX.ends_with("= 8"));
    }

    #[test]
    fn the_coarse_windows_sit_left_of_the_panel() {
        let (inp, out) = coarse_planes();
        // the panel, 7.6 wide and slid in, starts at x = −0.84
        assert!(inp.frame.left() >= -6.96 && inp.frame.right() < out.frame.left());
        assert!(out.frame.right() < -0.84);
        assert!(inp.contains(C::real(1.0)) && out.contains(C::real(1.0)));
        // rows stay under the windows and above the lower third
        assert!(COARSE_ROWS[0] < out.frame.bottom() && COARSE_ROWS[3] > -2.78);
        assert!(COARSE_TITLE_AT.1 > out.frame.top());
        const { assert!(COARSE_ROW_X >= -6.96) };
    }

    #[test]
    fn the_walks_fit_their_windows() {
        let (inp, out) = planes();
        for r in [0.8, 1.0, 1.7] {
            assert!(inp.contains(C::real(r)) && inp.contains(C::new(0.0, -r)));
        }
        assert!(out.contains(C::real(OUT_R)) && out.contains(C::real(-OUT_R)));
        assert!(inp.frame.right() < out.frame.left());
    }
}
