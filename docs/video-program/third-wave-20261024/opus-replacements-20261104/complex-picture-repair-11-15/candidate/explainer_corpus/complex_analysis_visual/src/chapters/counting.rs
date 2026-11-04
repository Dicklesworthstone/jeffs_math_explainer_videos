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

use std::f64::consts::{PI, TAU};

use explainer_kit::props::CodePanel;
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, Spoken, Walker, carded, dot, fact, fact_f64, fact_list, fade_in_all, fade_out_all, faded_line,
    flat, hero, input, line_style, marks, names_now, output, pause, pole, precisely_now, predict, reading_window,
    restroke, say, squash, tween_linear, value, wait_until, walker, warn,
};
use super::loop_sums::STEPS_SAID;
use super::the_bump::RADIUS;
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
/// 11_2's example, (z − 1)/(z + 1)²: on its two marks, then as the sum its walk just counted.
pub const SEVERAL_ORDERS: [&str; 2] = ["+1", "−2"];
pub const SEVERAL_SUM: &str = "+1 − 2 = −1";
/// 11_3's rows above the picture: what the two marks are. The loop's radius uses the bump's label.
pub const ZERO_AT: &str = "zero at ";
pub const POLE_AT: &str = "pole at ";
pub const THROUGH_POLE: &str = "loop through a pole: not allowed";
/// 11_4's readout: the output's true size as the walker nears the pole.
pub const SIZE_SAYS: &str = "|f| =";
/// After the distance the walk stopped at: the pole itself has no value to show.
pub const FROM_POLE: &str = " from the pole";
pub const OFF_WINDOW: &str = "off the window";
pub const ASK: [&str; 2] = ["z⁵ − 3z + 1: how many roots", "inside the unit circle?"];
pub const LEASH_TEX: &str = r"|z^5 + 1| \le 2 < 3 = |-3z|";
/// 11_6's name for the reference circle the output follows.
pub const MINUS_THREE_Z_TEX: &str = "-3z";
pub const INSIDE: &str = "one root inside, at distance ";
pub const SQUASHED: &str =
    "output drawn with its direction exact and its size compressed, so every turn around 0 is the function's own";
pub const WALKER_DOG: [&str; 2] = ["walker: z⁵", "dog: z⁵ − 3z + 1"];
/// 11_7, before the circle: the claim, then the polynomial it is about.
pub const ROOTS_CLAIM: &str = "every polynomial has roots";
pub const DEGREE_N_TEX: &str = r"p(z) = z^n + \cdots";
/// 11_7 names the general n and huge circle; the picture is this one (tested against `DogWalk`).
pub const DOG_DRAWN: &str = "drawn: n = 5, on the circle |z| = 3";
/// 11_7's two lengths, with their bars on one scale.
pub const DOG_ROWS: [&str; 2] = ["walker's distance from 0: ", "longest leash: "];
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
    "turns = 0; step = (length of the loop) / n",
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
        let (inp, _) = planes();
        let ring = inp.circle(stage, centre, r, hero(), 5.0)?;
        fade_in_all(stage, &[ring], RING_IN)?;
        Self::walk_on(kit, stage, ring, f, centre, r, s, stated, rt)
    }

    /// As [`Self::walk_and_count`], on a loop `ring` that is already on screen. What it returns
    /// starts with the loop and the stated count, then the walker's own pieces.
    #[allow(clippy::too_many_arguments)]
    fn walk_on(
        kit: &Kit,
        stage: &mut Stage<'_>,
        ring: Mob,
        f: Func,
        centre: C,
        r: f64,
        s: f64,
        stated: &str,
        rt: f64,
    ) -> fmn::Result<Vec<Mob>> {
        let (inp, out) = planes();
        let w = walker(stage, f, centre, r, inp, out, s, OUT_R)?;
        w.show(stage, WALKER_IN)?;
        let counter = w.counter(kit, stage, v(3.25, 3.15))?;
        w.walk(stage, rt)?;
        stage.clear_updaters(counter, true);
        let shown = kit.text(
            stage,
            &format!("{TURNS}{}", stated.replace('-', "−")),
            34.0,
            hero(),
            v(3.25, 3.15),
        )?;
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
                fade_out_all(stage, &leftover, 0.2)?;
                painter.to(stage, Some(f), 0.5)?;
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
            let mut named = mark.clone();
            named.extend(label);
            fade_in_all(stage, &named, 0.2)?;
            // each walk lands by the time its sentence is over, and never hurries below the floor
            let until = if i < 2 { line.sentence(i + 3)? } else { line.end() };
            let room = until - stage.scene().time().to_f64() - RING_IN - WALKER_IN - COUNT_HOLD;
            let rt = room.clamp(WALK_FLOOR, WALK_MOST);
            leftover = Self::walk_and_count(&kit, stage, f, ZERO, 0.8, 1.0, local[i], rt)?;
            leftover.extend(mark);
            leftover.extend(label);
            // the count has to land before the next fade can take it away
            pause(stage, COUNT_HOLD)?;
        }

        // 11_2: the rule and what it needs
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &leftover, 0.3)?;
        // "For a loop around several of them": a zero and a double pole, walked together
        painter.to(stage, Some(Func::ZeroDoublePole), 0.6)?;
        let several_marks = marks(stage, Func::ZeroDoublePole, &inp);
        fade_in_all(stage, &several_marks, 0.2)?;
        let room = line.sentence(1)? - stage.scene().time().to_f64() - RING_IN - WALKER_IN;
        let several = Self::walk_and_count(
            &kit,
            stage,
            Func::ZeroDoublePole,
            ZERO,
            SEVERAL_R,
            SEVERAL_S,
            fact("argument.several.turns")?,
            room.max(WALK_MOST),
        )?;
        line.wait_sentence(stage, 1)?;
        let rule = carded(&kit, stage, RULE, 34.0, hero(), v(-3.45, RULE_Y), hero())?;
        fade_in_all(stage, &rule, 0.5)?;
        // "each counted by its order": the walk's count, read off the two marks
        line.wait_for(stage, "each counted")?;
        let mut orders = Vec::new();
        orders.extend(carded(
            &kit,
            stage,
            SEVERAL_ORDERS[0],
            26.0,
            Palette::ink(),
            inp.p(C::new(1.0, SEVERAL_NAME_Y)),
            Palette::faint(),
        )?);
        orders.extend(carded(
            &kit,
            stage,
            SEVERAL_ORDERS[1],
            26.0,
            pole(),
            inp.p(C::new(-1.0, SEVERAL_NAME_Y)),
            Palette::faint(),
        )?);
        orders.push(kit.text(stage, SEVERAL_SUM, 24.0, hero(), v(3.25, ROW_OVER_OUTPUT))?);
        fade_in_all(stage, &orders, 0.3)?;
        line.wait_sentence(stage, 2)?;
        let search = names_now(&kit, stage, &["argument principle", "winding number"])?;
        // what it needs, with the loop and its marks still there to be checked against
        line.wait_sentence(stage, 3)?;
        fade_out_all(stage, &several[2..], 0.2)?;
        let mut needs = Vec::new();
        for (i, n) in NEEDS.iter().enumerate() {
            line.wait_for(stage, NEEDS_SAID[i])?;
            let c = carded(
                &kit,
                stage,
                n,
                22.0,
                Palette::ink(),
                v(3.25, 1.0 - 0.62 * i as f64),
                Palette::faint(),
            )?;
            fade_in_all(stage, &c, 0.3)?;
            needs.extend(c);
        }

        // 11_3: a zero at 0.45 and a pole at 1.35: radius 0.8 counts 1, radius 1.7 counts 0
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_3")?;
        let mut away = rule.to_vec();
        away.extend(needs);
        away.push(search);
        away.extend(orders);
        away.extend_from_slice(&several[..2]);
        away.extend(several_marks);
        fade_out_all(stage, &away, 0.4)?;
        painter.to(stage, Some(Func::Moebius), 1.0)?;
        let m_marks = marks(stage, Func::Moebius, &inp);
        let legend = [
            kit.text(
                stage,
                &format!("{ZERO_AT}{}", fact("argument.rational.zero")?),
                24.0,
                Palette::ink(),
                v(LEGEND_X[0], LEGEND_Y),
            )?,
            kit.text(
                stage,
                &format!("{POLE_AT}{}", fact("argument.rational.pole")?),
                24.0,
                pole(),
                v(LEGEND_X[1], LEGEND_Y),
            )?,
        ];
        fade_in_all(stage, &m_marks, 0.3)?;
        fade_in_all(stage, &legend, 0.3)?;
        line.wait_sentence(stage, 1)?;
        let radii = fact_list("argument.rational.radii")?;
        let printed: Vec<&str> = fact("argument.rational.radii")?.split(',').collect();
        let radius_small = kit.text(
            stage,
            &format!("{RADIUS}{}", printed[0]),
            24.0,
            hero(),
            v(-3.45, RADIUS_Y),
        )?;
        fade_in_all(stage, &[radius_small], 0.3)?;
        let small = Self::walk_and_count(
            &kit,
            stage,
            Func::Moebius,
            ZERO,
            radii[0],
            1.0,
            fact("argument.rational.r0_8")?,
            2.6,
        )?;
        line.wait_sentence(stage, 2)?;
        // "Grow the circle": the same loop grows; it is not taken away and redrawn
        let mut done = small[1..].to_vec();
        done.push(radius_small);
        fade_out_all(stage, &done, 0.2)?;
        grow_ring(stage, inp, small[0], radii[0], radii[1], GROW_S)?;
        let radius_big = kit.text(
            stage,
            &format!("{RADIUS}{}", printed[1]),
            24.0,
            hero(),
            v(-3.45, RADIUS_Y),
        )?;
        fade_in_all(stage, &[radius_big], 0.2)?;
        let big = Self::walk_on(
            &kit,
            stage,
            small[0],
            Func::Moebius,
            ZERO,
            radii[1],
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
        // once round one way on the outer circle, then back the other way on the inner one
        let (out_from, out_to) = unwinding_marks()[0];
        let (in_from, in_to) = unwinding_marks()[1];
        let out_mark = Arrow::new(inset.p(out_from), inset.p(out_to))
            .buff(0.0)
            .thickness(3.0)
            .color(output())
            .build()?;
        let in_mark = Arrow::new(inset.p(in_from), inset.p(in_to))
            .buff(0.0)
            .thickness(3.0)
            .color(output())
            .build()?;
        let out_mark = stage.arena_mut().add(out_mark);
        let in_mark = stage.arena_mut().add(in_mark);
        fade_in_all(stage, &[inset_card, curve, origin, net, out_mark, in_mark], 0.5)?;

        // 11_4: radius 1.35 runs through the pole: no count
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_4")?;
        let mut gone = big.clone();
        gone.extend([inset_card, curve, origin, net, out_mark, in_mark, radius_big]);
        fade_out_all(stage, &gone, 0.3)?;
        if fact("argument.rational.r1_35_refused")? != "true" {
            return Err(super::draw::fail("11_4 needs the count at radius 1.35 refused".into()));
        }
        let red_loop = dashed_circle(stage, &inp, model::MOEBIUS_POLE, warn())?;
        let refused = kit.text(stage, THROUGH_POLE, 26.0, warn(), v(3.25, 3.15))?;
        let radius_pole = kit.text(
            stage,
            &format!("{RADIUS}{}", fact("argument.rational.pole")?),
            24.0,
            warn(),
            v(-3.45, RADIUS_Y),
        )?;
        line.wait_sentence(stage, 0)?;
        fade_in_all(stage, &[red_loop, radius_pole], 0.5)?;
        let approach = approach_pole(&kit, stage, &line, inp, out)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[refused], 0.4)?;

        // 11_5: the quintic; how many roots inside the unit circle? The colouring would show
        // the answer, so it stays off until the think time is over
        kit.hold(stage)?;
        let line = say(&kit, stage, "11_5")?;
        let mut gone = vec![red_loop, refused, radius_pole];
        gone.extend(legend);
        gone.extend(m_marks);
        gone.extend(approach);
        fade_out_all(stage, &gone, 0.3)?;
        painter.to(stage, None, 1.2)?;
        let plain = inp.axes(stage)?;
        let unit = inp.circle(stage, ZERO, 1.0, hero(), 4.0)?;
        fade_in_all(stage, &plain, 0.3)?;
        fade_in_all(stage, &[unit], 0.4)?;
        let asked = predict(&kit, stage, &line, &ASK, v(ASK_AT.0, ASK_AT.1))?;

        // 11_6: one; the five roots come up with one inside the loop, which stays where it is;
        // the output follows −3z, which turns once
        let line = say(&kit, stage, "11_6")?;
        fade_out_all(stage, &asked, 0.3)?;
        fade_out_all(stage, &plain, 0.2)?;
        painter.to(stage, Some(Func::Quintic), 0.6)?;
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
        let follows = kit.tex(
            stage,
            MINUS_THREE_Z_TEX,
            26.0,
            input(),
            &[],
            out.p(C::new(THREE_Z_AT.0, THREE_Z_AT.1)),
        )?;
        fade_in_all(stage, &[leash, follows], 0.4)?;
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
        gone.extend([reference, follows, leash, squashed, unit]);
        gone.extend(inside);
        gone.extend(roots);
        fade_out_all(stage, &gone, 0.3)?;
        painter.leave(stage, 0.4)?;
        let claim = kit.text(stage, ROOTS_CLAIM, 30.0, hero(), v(-3.45, 0.7))?;
        fade_in_all(stage, &[claim], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let degree_n = kit.tex(stage, DEGREE_N_TEX, 36.0, Palette::ink(), &[], v(-3.45, -0.3))?;
        fade_in_all(stage, &[degree_n], 0.4)?;
        let wide = Plane::new(v(-3.45, -0.35), 5.6, (-3.6, 3.6), (-3.6, 3.6));
        let wide_axes = wide.axes(stage)?;
        let big_loop = wide.circle(stage, ZERO, 3.0, hero(), 5.0)?;
        let mut wide_all = wide_axes.clone();
        wide_all.push(big_loop);
        line.wait_sentence(stage, 2)?;
        fade_out_all(stage, &[claim, degree_n], 0.3)?;
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

/// How long each of 11_1's counts stays up once its walk has ended.
const COUNT_HOLD: f64 = 0.8;
/// The fade of a walk's loop, then of its walker.
const RING_IN: f64 = 0.3;
const WALKER_IN: f64 = 0.2;
/// A walk of 11_1 takes the room its sentence leaves, within these seconds.
const WALK_FLOOR: f64 = 1.8;
const WALK_MOST: f64 = 2.6;

/// 11_2's loop goes round the zero at 1 and the double pole at −1. The squash keeps the small
/// outputs near z = 1.6 clear of the output's 0.
const SEVERAL_R: f64 = 1.6;
const SEVERAL_S: f64 = 0.5;
/// How high the two marks' orders sit above them, in plane units.
const SEVERAL_NAME_Y: f64 = 0.42;
/// The words of 11_2 that name each of `NEEDS`.
const NEEDS_SAID: [&str; 3] = ["It needs", "passes through", "and a function"];
/// The rule's card, over the input picture.
const RULE_Y: f64 = 3.05;
/// A row between the turn count and the output plane's top edge.
const ROW_OVER_OUTPUT: f64 = 2.55;
/// 11_3's rows over the input picture.
const LEGEND_X: [f64; 2] = [-4.85, -2.05];
const LEGEND_Y: f64 = 2.8;
const RADIUS_Y: f64 = 3.25;
/// How long "Grow the circle" takes.
const GROW_S: f64 = 0.9;
/// 11_5's card: its ring lands inside the output plane, clear of the top edge and both axes.
const ASK_AT: (f64, f64) = (4.4, 2.75);
/// "−3z", in the output plane, outside everything the walk draws (|w| squashes to under 0.97).
const THREE_Z_AT: (f64, f64) = (0.95, 1.05);

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
    // above the window: inside it, this would be red on the red the window has just turned
    let covers = kit.text(stage, REACH_COVERS, 18.0, warn(), out.p(C::new(0.0, 2.2)))?;
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
        out.p(C::new(0.95, -1.2)),
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

    // "the output cannot move as far as its distance from zero": the reach is carried round the
    // whole output circle, with that distance drawn, and zero is never inside it
    line.wait_for(stage, "cannot move")?;
    let gap = stage.arena_mut().add(
        polyline(vec![out.p(ZERO), out.p(f(ONE))])
            .style(line_style(Palette::ink(), 2.5))
            .build(),
    );
    fade_in_all(stage, &[gap], 0.2)?;
    left.push(gap);
    let carried = stage.add_value_tracker(0.0);
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let c = C::polar(1.0, TAU * value(a, carried));
            a.move_to(disk, out.p(c), ORIGIN);
            a.move_to(a_name, out.p(c + C::new(-0.28, 0.2)), ORIGIN);
            let _ = a.set_points(gap, polyline(vec![out.p(ZERO), out.p(c)]).build().points());
        },
        true,
    )?;
    let rt = (line.sentence(5)? - stage.scene().time().to_f64()).max(1.6);
    stage.play(tween_linear(carried, 1.0, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);

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

/// Grow the circle `ring`, around 0 in `p`, from radius `from` to `to`, `rt` seconds.
fn grow_ring(stage: &mut Stage<'_>, p: Plane, ring: Mob, from: f64, to: f64, rt: f64) -> fmn::Result<()> {
    let grown = stage.add_value_tracker(from);
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let mut zs = model::circle(ZERO, value(a, grown), 240);
            zs.push(zs[0]);
            let pts: Vec<Vec3> = zs.iter().map(|&z| p.p(z)).collect();
            let _ = a.set_points(ring, polyline(pts).build().points());
        },
        true,
    )?;
    stage.play(tween_linear(grown, to, rt)?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    Ok(())
}

/// The inset's two direction marks, each a start and an end: on the outer circle going
/// anticlockwise, and on the inner circle going clockwise, as `unwinding_path` runs.
fn unwinding_marks() -> [(C, C); 2] {
    [
        (C::polar(0.45, 1.25), C::polar(0.45, 1.95)),
        (C::polar(0.28, -1.25), C::polar(0.28, -2.05)),
    ]
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

/// 11_4's readout row: where "|f| =" ends, and its height.
const SIZE_ROW: (f64, f64) = (2.75, 2.55);
/// Where the distance note starts in that row.
const FROM_POLE_X: f64 = 3.5;

/// The angle still to go to the pole at progress `s` of 11_4's walk: π at the start, and at
/// the end the angle whose chord is `model::MOEBIUS_NEAR_POLE`. It shrinks by the same factor
/// in each equal step of `s`, so the size the readout shows climbs steadily and stays finite.
fn pole_gap(s: f64) -> f64 {
    let end = 2.0 * (model::MOEBIUS_NEAR_POLE / (2.0 * model::MOEBIUS_POLE)).asin();
    PI * (end / PI).powf(s.clamp(0.0, 1.0))
}

/// The walker of 11_4: along |z| = 1.35, anticlockwise from −1.35 towards the pole.
fn pole_walk(s: f64) -> C {
    C::polar(model::MOEBIUS_POLE, -pole_gap(s))
}

/// The true output there. The walk stops short of the pole, so `eval` always has a value.
fn pole_walk_output(s: f64) -> C {
    Func::Moebius.eval(pole_walk(s)).unwrap_or(ZERO)
}

/// Where the output is drawn: true to scale, and held at the window's top edge once it has
/// gone past (the image of this circle is the straight line Re w = 2/3).
fn clipped_output(s: f64, top: f64) -> C {
    let w = pole_walk_output(s);
    C::new(w.re, w.im.min(top))
}

/// The progress at which the output leaves the window through `top`: on this circle
/// w = 2/3 + (i/3)·cot(gap/2), so it leaves where cot(gap/2)/3 = top.
fn pole_walk_exit(top: f64) -> f64 {
    let k = (model::MOEBIUS_POLE - model::MOEBIUS_ZERO) / (2.0 * model::MOEBIUS_POLE);
    let gap = 2.0 * (k / top).atan();
    (PI / gap).ln() / (PI / pole_gap(1.0)).ln()
}

/// 11_4's approach: the walker heads for the pole while the true output runs straight up the
/// output window and leaves it. The readout then carries on, showing |f| climbing, and stops
/// at a finite value a small distance short of the pole. No value is ever drawn at the pole,
/// because there is none. Returns everything it left on screen.
fn approach_pole(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken, inp: Plane, out: Plane) -> fmn::Result<Vec<Mob>> {
    let top = out.frame.y.1;
    let walked = stage.add_value_tracker(0.0);
    let start = out.p(clipped_output(0.0, top));
    let in_dot = dot(stage, inp.p(pole_walk(0.0)), 0.08, input());
    let out_dot = dot(stage, start, 0.08, output());
    let trail = stage
        .arena_mut()
        .add(polyline(vec![start, start]).style(line_style(output(), 4.0)).build());
    let (name_x, row_y) = SIZE_ROW;
    let name = kit.text(stage, SIZE_SAYS, 26.0, Palette::ink(), v(name_x, row_y))?;
    stage.move_to(name, v(name_x, row_y), RIGHT);
    fade_in_all(stage, &[trail, in_dot, out_dot, name], 0.3)?;
    let number_x = name_x + 0.12;
    let size = kit.live_value(
        stage,
        move |s| pole_walk_output(value(s, walked)).abs(),
        1,
        false,
        26.0,
        warn(),
        v(number_x, row_y),
        LEFT,
    )?;
    let anchor = stage.arena_mut().add(VMobject::new());
    stage.add_to_scene(anchor)?;
    stage.add_updater(
        anchor,
        move |a, _| {
            let s = value(a, walked);
            let end = out.p(clipped_output(s, top));
            a.move_to(in_dot, inp.p(pole_walk(s)), ORIGIN);
            a.move_to(out_dot, end, ORIGIN);
            let _ = a.set_points(trail, polyline(vec![start, end]).build().points());
        },
        true,
    )?;
    let total = line.until(stage, "There the count", 3.0)?;
    stage.play(tween_linear(walked, pole_walk_exit(top), 0.5 * total)?)?;
    let edge = out.p(clipped_output(1.0, top));
    let leaving = Arrow::new(v(edge[0], edge[1] - 0.45), edge)
        .buff(0.0)
        .thickness(4.0)
        .color(output())
        .build()?;
    let leaving = stage.arena_mut().add(leaving);
    let off = kit.text_left(
        stage,
        OFF_WINDOW,
        18.0,
        Palette::dim(),
        v(edge[0] + 0.16, edge[1] - 0.22),
    )?;
    fade_in_all(stage, &[leaving, off], 0.2)?;
    stage.play(tween_linear(walked, 1.0, (0.5 * total - 0.2).max(0.5))?)?;
    stage.clear_updaters(anchor, true);
    stage.remove_from_scene(anchor);
    stage.clear_updaters(size, true);
    let settled = kit.text_left(
        stage,
        fact("argument.rational.near_pole.size")?,
        26.0,
        warn(),
        v(number_x, row_y),
    )?;
    stage.remove_from_scene(size);
    stage.add_to_scene(settled)?;
    let from_pole = kit.text_left(
        stage,
        &format!("{}{FROM_POLE}", fact("argument.rational.near_pole.distance")?),
        20.0,
        Palette::dim(),
        v(FROM_POLE_X, row_y),
    )?;
    fade_in_all(stage, &[from_pole], 0.2)?;
    Ok(vec![trail, in_dot, out_dot, name, leaving, off, settled, from_pole])
}

/// 11_7's rows, above the plane: what is drawn, then the two lengths.
const DOG_ROW_X: f64 = -6.25;
const DOG_ROW_Y: [f64; 3] = [3.62, 3.2, 2.82];
/// The bars start here, right of the longest row, at one scale.
const DOG_BAR_X: f64 = -3.4;
const DOG_BAR_PER_UNIT: f64 = 3.0 / 243.0;

/// One of 11_7's length bars, `length` long at the shared scale, at height `y`.
fn length_bar(stage: &mut Stage<'_>, y: f64, length: f64, color: Srgb) -> Mob {
    let (x0, x1) = (DOG_BAR_X, DOG_BAR_X + DOG_BAR_PER_UNIT * length);
    stage.arena_mut().add(
        Polygon::new(vec![v(x0, y - 0.06), v(x1, y - 0.06), v(x1, y + 0.06), v(x0, y + 0.06)])
            .style(flat(color, 1.0))
            .build(),
    )
}

/// The dog on a leash (11_7): on |z| = 3 the walker z⁵ (blue) and the
/// polynomial (green) in the output plane, joined by the leash, with the
/// dog's trail and a live count of the dog's turns. The leash is far shorter
/// than the dots are wide at this scale, so two bars on one scale show its
/// length against the walker's distance.
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
        let far = fact_f64("argument.quintic.walker")?;
        let most = fact_f64("argument.quintic.leash")?;
        mobs.extend([
            kit.text_left(stage, DOG_DRAWN, 20.0, Palette::dim(), v(DOG_ROW_X, DOG_ROW_Y[0]))?,
            kit.text_left(
                stage,
                &format!("{}{}", DOG_ROWS[0], fact("argument.quintic.walker")?),
                20.0,
                input(),
                v(DOG_ROW_X, DOG_ROW_Y[1]),
            )?,
            kit.text_left(
                stage,
                &format!("{}{}", DOG_ROWS[1], fact("argument.quintic.leash")?),
                20.0,
                Palette::ink(),
                v(DOG_ROW_X, DOG_ROW_Y[2]),
            )?,
            length_bar(stage, DOG_ROW_Y[1], far, input()),
            length_bar(stage, DOG_ROW_Y[2], most, Palette::ink()),
        ]);
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
    fn the_inset_marks_run_the_way_the_path_does() {
        let [(out_from, out_to), (in_from, in_to)] = unwinding_marks();
        // the path's own direction: anticlockwise at 0.45, then clockwise at 0.28
        let path = unwinding_path(90);
        assert!((path[1] / path[0]).arg() > 0.0);
        assert!((path[93] / path[92]).arg() < 0.0);
        assert!((out_to / out_from).arg() > 0.0 && (in_to / in_from).arg() < 0.0);
        assert!((out_from.abs() - 0.45).abs() < 1e-12 && (out_to.abs() - 0.45).abs() < 1e-12);
        assert!((in_from.abs() - 0.28).abs() < 1e-12 && (in_to.abs() - 0.28).abs() < 1e-12);
    }

    #[test]
    fn the_example_of_several_counts_each_by_its_order() {
        // the zero at 1 and the double pole at −1 are both inside the loop drawn
        let w = (-2.0, 2.0);
        let zeros = Func::ZeroDoublePole.zeros(w, w);
        let poles = Func::ZeroDoublePole.poles(w, w);
        assert_eq!(zeros.len(), 1);
        assert_eq!(poles.len(), 1);
        assert!(zeros[0].abs() < SEVERAL_R && poles[0].0.abs() < SEVERAL_R);
        assert_eq!(poles[0].1, 2);
        // the sum on screen is the certified count of that walk, and its live counter ends there
        assert_eq!(fact("argument.several.turns").ok(), Some("-1"));
        assert!((Walker::turns_so_far(Func::ZeroDoublePole, ZERO, SEVERAL_R, 1.0) + 1.0).abs() < 1e-9);
        assert_eq!(SEVERAL_ORDERS, ["+1", "−2"]);
        assert_eq!(SEVERAL_SUM, "+1 − 2 = −1");
        // the loop is in the input window, and its smallest output is drawn clear of 0
        let (inp, out) = planes();
        assert!(inp.contains(C::real(SEVERAL_R)) && inp.contains(C::new(0.0, -SEVERAL_R)));
        let least = circle(ZERO, SEVERAL_R, 720)
            .into_iter()
            .map(|z| squash(Func::ZeroDoublePole.eval(z).unwrap(), SEVERAL_S, OUT_R).abs())
            .fold(f64::INFINITY, f64::min);
        assert!(least * out.unit() > 0.3, "{least}");
    }

    #[test]
    fn the_rows_of_the_third_cue_print_the_walks_it_draws() {
        let radii = fact_list("argument.rational.radii").unwrap();
        assert_eq!(radii, [0.8, 1.7]);
        assert_eq!(fact("argument.rational.zero").ok(), Some("0.45"));
        assert_eq!(fact("argument.rational.pole").ok(), Some("1.35"));
        // radius 0.8 holds only the zero, 1.7 holds both, and the growth passes the pole
        let (zero, at_pole) = (model::MOEBIUS_ZERO, model::MOEBIUS_POLE);
        assert!(zero < radii[0] && radii[0] < at_pole && at_pole < radii[1]);
        // the rows sit over the input window, inside the frame
        let (inp, _) = planes();
        assert!(LEGEND_Y > inp.frame.top() && RADIUS_Y > LEGEND_Y && RADIUS_Y < 3.75);
    }

    /// Read from the source, because no render can run here: the quintic's colouring shows its
    /// roots, so it may not be painted until the question's think time is over.
    #[test]
    fn the_quintic_is_not_painted_before_its_question_is_over() {
        let source = include_str!("counting.rs");
        let asked = source.find("predict(&kit, stage, &line, &ASK").unwrap();
        let painted = source.find("painter.to(stage, Some(Func::Quintic)").unwrap();
        assert!(asked < painted);
        assert_eq!(source.matches("Some(Func::Quintic)").count(), 3);
    }

    #[test]
    fn the_ring_of_the_question_clears_the_output_window_edges_and_axes() {
        // as `draw::predict` lays it out, for two lines of 28 point
        let height = 2.0 * (28.0 / 48.0 * 0.62) + 0.3;
        let (x, y) = (ASK_AT.0, ASK_AT.1 - height / 2.0 - 0.45);
        let (_, out) = planes();
        let ring = 0.26 + 0.04;
        assert!(y + ring < out.frame.top() && y - ring > out.p(ZERO)[1]);
        assert!(x - ring > out.p(ZERO)[0] && x + ring < out.frame.right());
        // and the card itself stops above the window
        assert!(ASK_AT.1 - height / 2.0 > out.frame.top());
    }

    #[test]
    fn the_name_of_the_circle_stays_clear_of_the_walk() {
        let reach = circle(ZERO, 1.0, 720)
            .into_iter()
            .map(|z| squash(Func::Quintic.eval(z).unwrap(), 2.0, OUT_R).abs())
            .fold(0.0, f64::max);
        assert!(C::new(THREE_Z_AT.0, THREE_Z_AT.1).abs() - 0.2 > reach, "{reach}");
    }

    #[test]
    fn the_leash_is_shorter_than_the_walkers_distance_on_the_big_circle() {
        for z in circle(ZERO, 3.0, 2000) {
            let p = Func::Quintic.eval(z).unwrap();
            assert!((p - z.powi(5)).abs() < z.powi(5).abs());
        }
    }

    #[test]
    fn the_rows_print_the_lengths_the_walk_really_has() {
        let far: f64 = fact("argument.quintic.walker").unwrap().parse().unwrap();
        let most: f64 = fact("argument.quintic.leash").unwrap().parse().unwrap();
        let mut longest = 0.0_f64;
        for z in circle(ZERO, 3.0, 720) {
            let leash = (Func::Quintic.eval(z).unwrap() - z.powi(5)).abs();
            longest = longest.max(leash);
            assert!((z.powi(5).abs() - far).abs() < 1e-9);
        }
        assert!((longest - most).abs() < 1e-9 && most < far);
        // the note names the picture: degree 5, on the walk's circle
        assert_eq!(fact("argument.quintic.big").ok(), Some("5"));
        assert!(DOG_DRAWN.contains("n = 5") && DOG_DRAWN.contains("|z| = 3"));
        // the longer bar ends left of the output window
        let (_, out) = planes();
        assert!(DOG_BAR_X + DOG_BAR_PER_UNIT * far < out.frame.left());
    }

    #[test]
    fn the_walk_to_the_pole_is_finite_and_its_output_runs_up_a_straight_line() {
        for k in 0..=64 {
            let s = f64::from(k) / 64.0;
            let w = pole_walk_output(s);
            assert!((w.re - 2.0 / 3.0).abs() < 1e-9, "{s}");
            assert!(w.im >= -1e-9, "{s}");
            assert!(Func::Moebius.eval(pole_walk(s)).is_some(), "{s}");
        }
        assert!((pole_walk(0.0) - C::real(-model::MOEBIUS_POLE)).abs() < 1e-12);
        let stop = (pole_walk(1.0) - C::real(model::MOEBIUS_POLE)).abs();
        assert!((stop - model::MOEBIUS_NEAR_POLE).abs() < 1e-12);
        // the readout ends on the printed size, the row on the printed distance
        let size: f64 = fact("argument.rational.near_pole.size").unwrap().parse().unwrap();
        let far: f64 = fact("argument.rational.near_pole.distance").unwrap().parse().unwrap();
        assert!((pole_walk_output(1.0).abs() - size).abs() < 0.05);
        assert!((far - model::MOEBIUS_NEAR_POLE).abs() < 1e-12);
        // the output leaves the window part way, and the clip holds it at the edge after that
        let (_, out) = planes();
        let top = out.frame.y.1;
        let exit = pole_walk_exit(top);
        assert!(exit > 0.0 && exit < 1.0);
        assert!((pole_walk_output(exit).im - top).abs() < 1e-9);
        assert!((clipped_output(1.0, top).im - top).abs() < 1e-12);
        assert!(out.contains(clipped_output(0.0, top)) && out.contains(clipped_output(1.0, top)));
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
        // carried round the whole output circle it stays in the window and never covers zero,
        // and the next sample always lands inside it
        for c in circle(ZERO, 1.0, 360) {
            assert!(c.abs() > reach);
            assert!(w.x.0 < c.re - reach && c.re + reach < w.x.1 && w.y.0 < c.im - reach && c.im + reach < w.y.1);
        }
        let step = C::polar(1.0, TAU / enough as f64).powi(power);
        assert!((step - ONE).abs() < reach);
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
