//! Chapter 11, how sure: the question turned around. The score of every
//! lake size for Feller's catch, relative to the top one (11_1); the
//! smallest possible lake and the 430 zeros of its chance, past where a
//! float64 rounds to zero (11_2); a million fish, a lone spike of plausible
//! sizes (11_3); only 4.43% even at the best size (11_4), like fifty heads
//! in a hundred flips (11_5). Scores are walked from the top with Feller's
//! ratio (`exact::relative_scores`); chances are exact (`exact`).

use std::rc::Rc;

use explainer_kit::props::predict_pause;
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, ink_style, play, rect, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{Role, flat, line_style};
use crate::exact::{Dd, hypergeometric, hypergeometric_pmf, relative_scores};
use crate::facts::FELLER;

/// The score plot: lake sizes across (the right end moves in 11_3).
fn plot(xmax: f64) -> Frame2 {
    Frame2 {
        center: v(0.0, 0.35),
        width: 12.0,
        height: 4.2,
        x: (0.0, xmax),
        y: (0.0, 1.12),
    }
}

/// Lake sizes drawn: every 20th from the smallest possible lake.
const STEP: usize = 20;
/// The plot's first right end, and its last (a million fish, 11_3).
const NEAR: f64 = 20_000.0;
const MILLION: u64 = 1_000_000;

/// The scores, relative to the top one, at every drawn size: (N, score).
fn scores() -> Vec<(f64, f64)> {
    let (m, n, k) = FELLER;
    let lo = m + n - k;
    let hi = NEAR as u64;
    let rel = relative_scores(FELLER, lo, hi);
    rel.iter()
        .enumerate()
        .step_by(STEP)
        .map(|(i, &r)| ((lo + i as u64) as f64, r))
        .collect()
}

/// The curve at plot width `xmax`: beyond the drawn sizes the scores are
/// zero to the eye, so the line runs on along the floor.
fn curve(points: &[(f64, f64)], xmax: f64) -> VMobject {
    let f = plot(xmax);
    let mut pts: Vec<Vec3> = points.iter().map(|&(x, y)| f.p(x, y)).collect();
    pts.push(f.p(xmax, 0.0));
    Polygon::polyline(pts).style(line_style(Palette::purple(), 4.0)).build()
}

/// The floor, the score axis and five ticks at quarters of `xmax`.
fn axes(xmax: f64) -> fmn::Result<VMobject> {
    let f = plot(xmax);
    let mut parts = vec![
        Line::new(v(f.left(), f.bottom()), v(f.right(), f.bottom()))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
        Line::new(v(f.left(), f.bottom()), v(f.left(), f.top()))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    for q in 0..=4 {
        let p = f.p(xmax * f64::from(q) / 4.0, 0.0);
        parts.push(
            Line::new(v(p[0], p[1] - 0.1), v(p[0], p[1]))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
    }
    Ok(v_group(parts))
}

/// A tick's words: the size, rounded to the nearest hundred.
fn tick_word(x: f64) -> String {
    thousands(((x / 100.0).round() * 100.0) as u64)
}

pub struct HowSure {
    pub kit: Kit,
}

impl SceneConstruct for HowSure {
    fn name(&self) -> &str {
        "11_how_sure"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let points = Rc::new(scores());
        let (curve_mob, mut ticks, peak_word) = score_curve(&kit, stage, &points)?;
        ticks.push(peak_word);
        let smallest = smallest_lake(&kit, stage)?;
        a_million(&kit, stage, &points, curve_mob, ticks, smallest)?;
        not_a_problem(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 11_1: every lake size's score for our hundred caught twice.
fn score_curve(kit: &Kit, stage: &mut Stage<'_>, points: &[(f64, f64)]) -> fmn::Result<(Mob, Vec<Mob>, Mob)> {
    let f = plot(NEAR);
    let axis = stage.arena_mut().add(axes(NEAR)?);
    let mut ticks = Vec::new();
    for q in 0..=4 {
        let x = NEAR * f64::from(q) / 4.0;
        let p = f.p(x, 0.0);
        ticks.push(kit.text(stage, &tick_word(x), 22.0, Palette::dim(), v(p[0], p[1] - 0.32))?);
    }
    let size_word = kit.text(
        stage,
        "lake size",
        26.0,
        Role::dark(),
        v(f.center[0], f.bottom() - 0.72),
    )?;
    let score_word = kit.text_left(
        stage,
        "score, compared with the top one",
        26.0,
        Palette::purple(),
        v(f.left() + 0.15, f.top() + 0.25),
    )?;
    let line = stage.arena_mut().add(curve(points, NEAR));
    let (fm, fnn, fk) = FELLER;
    let top = (fm * fnn / fk) as f64;
    let peak = f.p(top, 1.0);
    let peak_word = kit.text(
        stage,
        &thousands(top as u64),
        24.0,
        Role::dark(),
        v(peak[0], peak[1] + 0.28),
    )?;
    let catch = kit.text(
        stage,
        &format!(
            "our catch: {} marked, {} caught, {} caught twice",
            thousands(fm),
            thousands(fnn),
            thousands(fk)
        ),
        24.0,
        Palette::ink(),
        v(f.center[0] + 2.6, f.top() + 0.25),
    )?;
    kit.say(stage, "11_1")?;
    let mut intro: Vec<Box<dyn Animation>> = Vec::new();
    for m in [axis, size_word, score_word, catch]
        .into_iter()
        .chain(ticks.iter().copied())
    {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        intro.push(stage.prepare(a)?);
    }
    stage.play_prepared(intro)?;
    // "Which sizes could plausibly have given us our hundred caught twice?"
    kit.until_sentence(stage, "11_1", 3)?;
    play!(stage; show_creation(line).rt(2.0));
    play!(stage; fade_in(stage.arena_mut(), peak_word, v(0.0, -0.1), 1.0)?.rt(0.6));
    kit.hold(stage)?;
    play!(stage; fade_out(stage.arena_mut(), catch, ORIGIN, 1.0)?.rt(0.4));
    // the words that stay through 11_3 ride along with the axis' group
    stage.attach(axis, size_word)?;
    stage.attach(axis, score_word)?;
    Ok((line, ticks, peak_word))
}

/// What 11_2 leaves for 11_3: the smallest lake's marker and its words.
struct Smallest {
    marker: Mob,
    words: Vec<Mob>,
}

/// 11_2: only 1,900 fish, and the zeros of its chance.
fn smallest_lake(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Smallest> {
    let f = plot(NEAR);
    let (fm, fnn, fk) = FELLER;
    let lake = fm + fnn - fk;
    let x = f.p(lake as f64, 0.0)[0];
    let marker = stage.arena_mut().add(
        DashedLine::new(v(x, f.bottom()), v(x, f.top() - 0.3))
            .dash_length(0.1)
            .style(line_style(Palette::red(), 2.5))
            .build()?,
    );
    let tag = kit.text(
        stage,
        &format!("{} fish", thousands(lake)),
        26.0,
        Palette::red(),
        v(x, f.top() - 0.1),
    )?;
    let tx = x + 0.3;
    let line1 = kit.text_left(stage, "the second net took all", 24.0, Palette::ink(), v(tx, 1.85))?;
    let line2 = kit.text_left(
        stage,
        &format!("{} unmarked fish", thousands(lake - fm)),
        24.0,
        Role::unmarked(),
        v(tx, 1.5),
    )?;
    let line3 = kit.text_left(
        stage,
        &format!("and only {} of the {} marked", thousands(fk), thousands(fm)),
        24.0,
        Role::marked(),
        v(tx, 1.15),
    )?;
    // the chance, exactly: 5.35… × 10^−430
    let p = hypergeometric(lake, FELLER);
    let (mant, ex) = p.sci10();
    let chance = kit.tex(
        stage,
        &format!(r"\text{{chance}} \approx {:.0} \times 10^{{{}}}", mant, ex),
        40.0,
        Palette::red(),
        &[],
        v(0.0, 0.45),
    )?;
    stage.move_to(chance, v(tx, 0.45), LEFT);
    kit.say(stage, "11_2")?;
    play!(stage;
        show_creation(marker).rt(0.6),
        fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.6),
    );
    // "Then the second net scooped up all nine hundred unmarked fish and
    // only a hundred of the thousand marked ones."
    kit.until_sentence(stage, "11_2", 1)?;
    play!(stage;
        fade_in(stage.arena_mut(), line1, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), line2, ORIGIN, 1.0)?.rt(0.6),
    );
    play!(stage; fade_in(stage.arena_mut(), line3, ORIGIN, 1.0)?.rt(0.6));
    // "The chance of that is about five in a one followed by four hundred
    // thirty zeros." The chance, then the zeros under the words (R3 Dev D4)
    kit.until_sentence(stage, "11_2", 2)?;
    play!(stage; write(stage.arena(), chance).rt(0.8));
    // the zeros: 0.000…0535…, the dashed line where float64 gives up
    let zeros = ex.unsigned_abs() as usize - 1;
    let digits = format!("{:.2}", mant).replace('.', "");
    let floor_z = float64_last_nonzero_place(p);
    let s = format!("0.{}{digits}", "0".repeat(zeros));
    let y = -3.4;
    let strip = kit.mono(stage, &s, 30.0, Palette::ink(), v(0.0, y))?;
    let w = stage.get_width(strip);
    let char_w = w / s.chars().count() as f64;
    // start with the "0." at the left of the frame, end with the digits at
    // the right
    let start_left = -6.6;
    stage.move_to(strip, v(start_left, y), LEFT);
    let cut_x = start_left + char_w * (2 + floor_z) as f64;
    let cut = stage.arena_mut().add(
        DashedLine::new(v(cut_x, y - 0.3), v(cut_x, y + 0.42))
            .dash_length(0.08)
            .style(line_style(Palette::hero(), 3.0))
            .build()?,
    );
    // the label stays put over the middle while the cut is held there
    let cut_word = kit.text(
        stage,
        "a computer's everyday numbers (float64) round to zero past here",
        24.0,
        Palette::hero(),
        v(0.0, y + 0.6),
    )?;
    let reel = stage.arena_mut().add(VMobject::new());
    for m in [strip, cut] {
        stage.attach(reel, m)?;
    }
    play!(stage; fade_in(stage.arena_mut(), reel, ORIGIN, 1.0)?.rt(0.4));
    // scroll: bring the cut to the middle, let it be read, run to the end
    let travel = (start_left + w) - 6.6;
    let to_cut = cut_x;
    let leg = |stage: &mut Stage<'_>, dx: f64, secs: f64| -> fmn::Result<()> {
        let a = reel
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .shift([-dx, 0.0, 0.0])?;
        stage.play(a)?;
        Ok(())
    };
    leg(stage, to_cut, 1.6)?;
    play!(stage; fade_in(stage.arena_mut(), cut_word, v(0.0, -0.1), 1.0)?.rt(0.4));
    stage.wait(2.6)?;
    play!(stage; fade_out(stage.arena_mut(), cut_word, ORIGIN, 1.0)?.rt(0.4));
    leg(stage, travel - to_cut, 1.4)?;
    play!(stage; fade_out(stage.arena_mut(), reel, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;
    Ok(Smallest {
        marker,
        words: vec![tag, line1, line2, line3, chance],
    })
}

/// The last decimal place a number with its leading digit there still
/// survives in as a float64 (P_1900's mantissa slid along: A-F32.zeros).
fn float64_last_nonzero_place(p: Dd) -> usize {
    let (_, ex) = p.sci10();
    let digits = p.mul(Dd::pow10(ex.unsigned_abs()));
    (1..=ex.unsigned_abs())
        .find(|&z| digits.div(Dd::pow10(z)).to_f64() == 0.0)
        .map_or(0, |z| z as usize - 1)
}

/// 11_3: a million fish; the plot widens and the plausible sizes become a
/// spike at its left end.
fn a_million(
    kit: &Kit,
    stage: &mut Stage<'_>,
    points: &Rc<Vec<(f64, f64)>>,
    line: Mob,
    ticks: Vec<Mob>,
    smallest: Smallest,
) -> fmn::Result<()> {
    let (fm, fnn, fk) = FELLER;
    let (marker, words) = (smallest.marker, smallest.words);
    kit.say(stage, "11_3")?;
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in words.into_iter().chain(ticks).chain([marker]) {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.5),
            ..PlayOverrides::default()
        },
    )?;
    // the widening: the curve and its ticks redrawn for each right end
    let xmax = stage.add_value_tracker(NEAR);
    let pts = Rc::clone(points);
    let book = Rc::clone(&kit.book);
    let live = stage.always_redraw(move |s| {
        let w = s.tracker_value(xmax).unwrap_or(NEAR);
        let f = plot(w);
        let mut parts = vec![curve(&pts, w)];
        for q in 0..=4 {
            let x = w * f64::from(q) / 4.0;
            let p = f.p(x, 0.0);
            if let Ok(t) = Text::new(&tick_word(x))
                .font_size(22.0)
                .style(ink_style(Palette::dim()))
                .build(&book)
            {
                parts.push(t.vmob.moved_to(v(p[0], p[1] - 0.32)));
            }
        }
        s.add(v_group(parts))
    });
    stage.remove_from_scene(line);
    stage.add_to_scene(live)?;
    stage.play(tween(xmax, MILLION as f64, 2.5)?)?;
    stage.clear_updaters(live, true);
    // the million: a marker at the far right, and what we would expect
    let f = plot(MILLION as f64);
    let x = f.p(MILLION as f64, 0.0)[0];
    let far = stage.arena_mut().add(
        DashedLine::new(v(x, f.bottom()), v(x, f.top() - 0.3))
            .dash_length(0.1)
            .style(line_style(Palette::red(), 2.5))
            .build()?,
    );
    let far_tag = kit.text(stage, "a million fish", 26.0, Palette::red(), v(x - 0.6, f.top() - 0.1))?;
    let expect = fm * fnn / MILLION;
    let expect_word = kit.text(
        stage,
        &format!(
            "expect {} × {} ÷ {} = {} marked fish caught twice, not {}",
            thousands(fm),
            thousands(fnn),
            thousands(MILLION),
            expect,
            thousands(fk)
        ),
        24.0,
        Palette::ink(),
        v(1.5, 1.5),
    )?;
    let spike = f.p((fm * fnn / fk) as f64, 1.0);
    let spike_word = kit.text_left(
        stage,
        "every plausible size",
        24.0,
        Palette::purple(),
        v(spike[0] + 0.35, spike[1] - 0.25),
    )?;
    play!(stage;
        show_creation(far).rt(0.6),
        fade_in(stage.arena_mut(), far_tag, ORIGIN, 1.0)?.rt(0.6),
    );
    // "Then we'd expect to catch about one marked fish, not a hundred."
    kit.until_sentence(stage, "11_3", 1)?;
    play!(stage;
        fade_in(stage.arena_mut(), expect_word, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), spike_word, ORIGIN, 1.0)?.rt(0.8),
    );
    kit.hold(stage)?;
    Ok(())
}

/// A histogram of chances over counts, one flat bar per count, with one
/// count lit.
fn chance_bars(f: &Frame2, counts: &[(u32, f64)], lit: u32) -> fmn::Result<(VMobject, VMobject)> {
    let unit = f.width / (f.x.1 - f.x.0);
    let mut dim = Vec::new();
    let mut hot = Vec::new();
    for &(k, p) in counts {
        let base = f.p(f64::from(k), 0.0);
        let top = f.p(f64::from(k), p);
        let (c, o) = if k == lit {
            (Palette::hero(), 1.0)
        } else {
            (Role::chance(), 0.85)
        };
        let r = rect(
            v(base[0] - 0.4 * unit, base[1]),
            0.8 * unit,
            top[1] - base[1],
            flat(c, o),
        )?;
        if k == lit {
            hot.push(r);
        } else {
            dim.push(r);
        }
    }
    Ok((v_group(dim), v_group(hot)))
}

/// 11_4–11_5: the best size gives exactly a hundred only 4.43% of the time;
/// exactly fifty heads in a hundred flips, 7.96%.
fn not_a_problem(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    fade_all(stage, 0.8)?;
    let (fm, fnn, fk) = FELLER;
    let best = fm * fnn / fk;
    let pmf = hypergeometric_pmf(best, fm, fnn);
    let lo = (fk as u32).saturating_sub(30);
    let hi = fk as u32 + 30;
    let counts: Vec<(u32, f64)> = (lo..=hi).map(|k| (k, pmf[k as usize].to_f64())).collect();
    let left = Frame2 {
        center: v(-3.5, -0.4),
        width: 6.0,
        height: 3.4,
        x: (f64::from(lo) - 0.5, f64::from(hi) + 0.5),
        y: (0.0, 0.05),
    };
    let (dim, hot) = chance_bars(&left, &counts, fk as u32)?;
    let (dim, hot) = (stage.arena_mut().add(dim), stage.arena_mut().add(hot));
    let floor = stage.arena_mut().add(
        Line::new(v(left.left(), left.bottom()), v(left.right(), left.bottom()))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let mut words = Vec::new();
    for k in [lo, fk as u32, hi] {
        let p = left.p(f64::from(k), 0.0);
        words.push(kit.text(stage, &k.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.28))?);
    }
    let title = kit.text(
        stage,
        &format!("a lake of {}: fish caught twice", thousands(best)),
        26.0,
        Role::dark(),
        v(left.center[0], left.top() + 0.55),
    )?;
    let p_best = hypergeometric(best, FELLER).to_f64();
    let best_word = kit.text(
        stage,
        &format!("exactly {}: {:.2}%", fk, 100.0 * p_best),
        30.0,
        Palette::hero(),
        v(left.center[0], left.bottom() - 0.75),
    )?;
    kit.say(stage, "11_4")?;
    let mut show: Vec<Box<dyn Animation>> = Vec::new();
    for m in [dim, floor, title].into_iter().chain(words.iter().copied()) {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        show.push(stage.prepare(a)?);
    }
    stage.play_prepared(show)?;
    play!(stage;
        fade_in(stage.arena_mut(), hot, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), best_word, ORIGIN, 1.0)?.rt(0.6),
    );
    let q = predict_pause(kit, stage, v(3.5, -0.4))?;

    // 11_5: a hundred coin flips
    let flips = 100u64;
    let half = (flips / 2) as u32;
    let coin: Vec<(u32, f64)> = (half - 20..=half + 20)
        .map(|j| {
            let p = Dd::int128(explainer_kit::choose(flips, u64::from(j)))
                .div(Dd::pow2(flips as i32))
                .to_f64();
            (j, p)
        })
        .collect();
    let right = Frame2 {
        center: v(3.5, -0.4),
        width: 6.0,
        height: 3.4,
        x: (f64::from(half - 20) - 0.5, f64::from(half + 20) + 0.5),
        y: (0.0, 0.09),
    };
    let (cdim, chot) = chance_bars(&right, &coin, half)?;
    let (cdim, chot) = (stage.arena_mut().add(cdim), stage.arena_mut().add(chot));
    let cfloor = stage.arena_mut().add(
        Line::new(v(right.left(), right.bottom()), v(right.right(), right.bottom()))
            .style(line_style(Palette::dim(), 2.0))
            .build()?,
    );
    let mut cwords = Vec::new();
    for j in [half - 20, half, half + 20] {
        let p = right.p(f64::from(j), 0.0);
        cwords.push(kit.text(stage, &j.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.28))?);
    }
    let ctitle = kit.text(
        stage,
        &format!("{flips} coin flips: heads"),
        26.0,
        Palette::ink(),
        v(right.center[0], right.top() + 0.55),
    )?;
    let p_half = coin.iter().find(|&&(j, _)| j == half).map_or(0.0, |&(_, p)| p);
    let half_word = kit.text(
        stage,
        &format!("exactly {half}: {:.2}%", 100.0 * p_half),
        30.0,
        Palette::hero(),
        v(right.center[0], right.bottom() - 0.75),
    )?;
    let mode_word = kit.text(
        stage,
        "and still the most likely count",
        24.0,
        Palette::ink(),
        v(right.center[0], right.bottom() - 1.25),
    )?;
    kit.say(stage, "11_5")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    let mut show: Vec<Box<dyn Animation>> = Vec::new();
    for m in [cdim, cfloor, ctitle].into_iter().chain(cwords.iter().copied()) {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        show.push(stage.prepare(a)?);
    }
    stage.play_prepared(show)?;
    // "Exactly fifty heads comes up only about eight percent of the time..."
    kit.until_sentence(stage, "11_5", 1)?;
    play!(stage;
        fade_in(stage.arena_mut(), chot, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), half_word, ORIGIN, 1.0)?.rt(0.6),
    );
    play!(stage; fade_in(stage.arena_mut(), mode_word, ORIGIN, 1.0)?.rt(0.6));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::py;

    #[test]
    fn what_chapter_11_says_holds_exactly() {
        let (fm, fnn, fk) = FELLER;
        // 11_2: 1,900 fish; all 900 unmarked and 100 of the 1,000 marked;
        // "about five in a one followed by four hundred thirty zeros"
        let lake = fm + fnn - fk;
        assert_eq!((lake, lake - fm, fk, fm), (1_900, 900, 100, 1_000));
        let p = hypergeometric(lake, FELLER);
        let (mant, ex) = p.sci10();
        assert_eq!((format!("{mant:.0}"), ex), ("5".to_string(), -430));
        assert_eq!(format!("{mant:.2}"), "5.35");
        // the dashed line: a float64 still holds 5.35 × 10^−324, not 10^−325
        assert_eq!(float64_last_nonzero_place(p), 324);
        // 11_3: a million fish, about one marked fish expected
        assert_eq!(fm * fnn / MILLION, 1);
        assert_eq!((fm * fnn) % MILLION, 0);
        // 11_4: "about a four percent chance", screen 4.43%
        let best = hypergeometric(fm * fnn / fk, FELLER).to_f64();
        assert_eq!(py(best), "0.04428917172067476");
        assert_eq!(format!("{:.2}", 100.0 * best), "4.43");
        assert_eq!((100.0 * best).round(), 4.0);
        // 11_5: "about eight percent of the time", screen 7.96%; the mode
        let coin = Dd::int128(explainer_kit::choose(100, 50)).div(Dd::pow2(100)).to_f64();
        assert_eq!(format!("{:.2}", 100.0 * coin), "7.96");
        assert_eq!((100.0 * coin).round(), 8.0);
        assert!((0..=100).all(|j| explainer_kit::choose(100, 50) >= explainer_kit::choose(100, j)));
    }

    #[test]
    fn the_curve_peaks_at_ten_thousand_and_vanishes_at_the_ends() {
        let s = scores();
        let (top_n, top) = s.iter().copied().fold((0.0, 0.0), |b, p| if p.1 > b.1 { p } else { b });
        assert!((top - 1.0).abs() < 1e-12 && top_n == 10_000.0, "{top_n} {top}");
        assert!(s.first().unwrap().1 < 1e-300);
        assert!(s.last().unwrap().1 < 1e-9);
    }
}
