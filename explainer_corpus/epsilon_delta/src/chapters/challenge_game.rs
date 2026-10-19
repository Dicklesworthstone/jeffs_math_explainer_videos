//! Chapter 04, the challenge game: chapter 03's last picture, then its
//! tolerance and window as a game with strict turns. The challenger names
//! a tolerance ε, the defender a window δ > 0, and only then may the
//! challenger pick any h in the window but 0 (04_1). Against 1/10 the
//! defender answers 1/10, and every pick lands in the band: the slope is
//! 1 + h, exactly |h| from 1, an argument for every h, not a few samples
//! (04_2). A window that catches some h is not enough: 1/5 loses at
//! h = 0.15, where the slope is 1.15 (04_3). A limit leaves h = 0 out;
//! continuity puts it in and aims at the value there, and the third rule
//! becomes |h| < δ. Filled with 1, the graph is continuous; filled with
//! 1.5, no window works: any δ holds the pick h = ½·min(δ, 2/5), whose
//! value is at least 3/10 from 1.5, the case δ = 1/10 played (04_4). The
//! prediction: could the defender choose after seeing h (04_5)? Yes, with a
//! window too narrow to hold it; h = 0 is in every window but passes, g(0)
//! being within ε of itself. So δ may depend on ε, the point and the curve,
//! never on the later h (04_6). Then continuity of x² at x = 1, the end of
//! its domain [0, 1], against 1/2: the window runs from 2^(−1/2), where x²
//! is exactly 1/2 and which it leaves out, up to 1, which it holds (04_7);
//! a winning answer covers every input of its window (04_8).
//!
//! Every number drawn comes from the facts registry; every shape from the
//! model, whose witnesses (`window_failure`, `fill_failure`, `blind_window`)
//! are the picks the scene plays.

use explainer_kit::layout::TEXT_MARGIN;
use explainer_kit::props::precisely;
use explainer_kit::{BACKGROUND, Facts, Kit, Palette, Timed, hex, rect, v};
use fmn::prelude::*;

use super::draw::{
    BRIGHT_W, END_R, NUMBER_DROP, PLOT, curve_color, curve_label, fact, fade_in_all, fade_in_lagged, fade_out_all,
    fail, fails, flat, limit_color, line_style, move_to, moving, say, slope_color, think_pause,
};
use super::old_objection::{NEXT_Y, READ_Y, RIGHT_X, SLOPES, final_picture, shown, swap, tex_frac};
use super::shrinking_region::glow;
use crate::facts::facts;
use crate::model::{
    BASE, CHALLENGE, FILLS, Frac, LUCKY, PICKS, SQUARE, WIDE, blind_window, chord_limit, chord_slope, fill_failure,
    half_height, power, window, window_failure,
};

pub struct TheChallengeGame {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIRST: &str = r"\text{first the challenger picks a tolerance }";
const THEN_DEFENDER: &str = r"\text{then the defender picks a window }";
const THEN_CHALLENGER: &str = r"\text{then the challenger picks any }";
const SLOPE: &str = "slope";
const NOT_IN: &str = "not in the window";
const ANY: &str = r"\text{any } \delta:\ h = \tfrac{1}{2} \min(\delta, ";
const EVERY_WINDOW: &str = r"\text{ is in every window;}";
const DEPENDS: &str = r"\delta \text{ may depend on } \varepsilon\text{, the point and the curve, never on the } h \text{ picked after it}";
const DOMAIN: &str = "domain";
const BEYOND: &str = "no x beyond";
const EXACTLY: &str = r"\text{ exactly, not within}";
const NEXT: &str = "next: how a window is found, and why a written proof hides the search";

// what the pictures wait for, each in the line the test below holds it to
const TOLERANCE_AT: &str = "names a tolerance";
const WINDOW_AT: &str = "answers with a window";
const PICK_AT: &str = "picks any h";
const AGAINST_AT: &str = "Against one tenth";
const ANSWERS_AT: &str = "answers one tenth";
const EVERY_AT: &str = "Every h the challenger";
const SINCE_AT: &str = "since each slope";
const CATCHING_AT: &str = "Catching some h";
const FIFTH_AT: &str = "Answer one fifth";
const OUTSIDE_AT: &str = "point one five";
const LEAVES_AT: &str = "leaves zero out";
const PUTS_AT: &str = "Continuity puts zero in";
const FILL_ONE_AT: &str = "Fill the hole with one";
const FILL_HALF_AT: &str = "one and a half";
const NARROW_AT: &str = "too narrow";
const DEPEND_AT: &str = "So the window may depend";
const END_AT: &str = "x equals one";
const HALF_AT: &str = "Against one half";
const STARTS_AT: &str = "the window starts";
const PASSES_AT: &str = "every x from there";
const COVERS_AT: &str = "covers every input";
const SEARCH_AT: &str = "Finding them";

// --- layout (scene units; the frame is 14.2 by 8; the plot is draw::PLOT,
// the slope panel chapter 03's SLOPES)

/// The three rules, left-aligned right of the panel's slope axis, over the
/// panel and under the top edge.
const RULE_X: f64 = 1.6;
const RULE_Y: [f64; 3] = [3.45, 3.07, 2.69];
/// A pick's readout, in the panel's empty bottom right (scene units), and
/// the two lines over it: the argument for every window (04_4), then the
/// note on h = 0 (04_6).
const READOUT: Vec3 = [4.2, 0.15, 0.0];
const OVER_READOUT: [f64; 2] = [0.75, 0.45];
/// A label of a band around the limit sits at this height (data), as in
/// chapter 03; a band around the wrong fill is labelled under it.
const LABEL_H: f64 = -0.57;
const LABEL_ABOVE: f64 = 1.33;
const LABEL_UNDER: f64 = 0.17;
/// The window's label sits inside the panel, over its foot, at h = 0.
const WINDOW_LABEL_UP: f64 = 0.3;
/// A missed pick's ring.
const MISS_R: f64 = 0.14;
/// How long a pick takes to move; the glow that follows each landing (0.9 s)
/// lets its readout stand before the next.
const HOP_S: f64 = 0.5;
/// The δ-rule's line, centred under the panel once the card has gone.
const RULE_LINE_Y: f64 = -2.75;
/// The think ring, in the plot's empty top left.
const RING_AT: Vec3 = [-4.4, 1.6, 0.0];
const RING_R: f64 = 0.45;
/// The end of the square's domain (04_7): the band reaches this far above
/// 1 (data) so the end point sits inside it; the window's end marks; the
/// right column's rows.
const BAND_PAST: f64 = 0.06;
const MARK_R: f64 = 0.07;
const END_ROWS: [f64; 6] = [3.0, 2.3, 1.5, 0.85, 0.2, -0.45];
const PASS_SAMPLES: usize = 40;

// the rules stay title-safe under the top edge (each line about 0.15
// either side of its centre at 22 pt), apart, and over the panel's top;
// the readout sits inside the panel's bottom right, over the window's
// label and under any band 0.2 or less either side of the limit (data);
// the δ-rule sits under the definitions' rows and inside the frame; the
// ring is inside the frame and clear of the plot's floor
const _: () = assert!(RULE_Y[0] + 0.2 < 4.0 - TEXT_MARGIN && RULE_Y[0] - RULE_Y[1] > 0.32);
const _: () = assert!(RULE_Y[1] - RULE_Y[2] > 0.32);
const _: () = assert!(RULE_Y[2] - 0.2 > SLOPES.frame.center[1] + SLOPES.frame.height / 2.0);
const _: () = assert!(RULE_X > SLOPES.frame.center[0] - SLOPES.frame.width / 2.0 + 0.2);
const _: () = assert!(READOUT[1] - 0.15 > SLOPES.frame.center[1] - SLOPES.frame.height / 2.0 + WINDOW_LABEL_UP + 0.15);
const _: () = assert!(READOUT[1] + 0.15 < SLOPES.frame.center[1] - 0.2 * SLOPES.frame.height / 1.5);
const _: () = assert!(RULE_LINE_Y + 0.2 < NEXT_Y - 0.2 && RULE_LINE_Y - 0.2 > -4.0 + TEXT_MARGIN);
const _: () = assert!(RING_AT[0] - RING_R > -7.1 + TEXT_MARGIN && RING_AT[1] - RING_R - 0.45 > -1.8);
const _: () = assert!(END_ROWS[0] + 0.2 < 4.0 - TEXT_MARGIN && END_ROWS[5] - 0.2 > NEXT_Y + 0.2);

/// A band across the panel, the tolerance t either side of `centre`.
fn panel_band(stage: &mut Stage<'_>, centre: f64, t: f64) -> fmn::Result<Mob> {
    let p = SLOPES.frame;
    let (y0, y1) = (SLOPES.p(0.0, centre - t)[1], SLOPES.p(0.0, centre + t)[1]);
    let band = rect(v(p.left(), y0), p.width, y1 - y0, flat(limit_color(), 0.22))?;
    Ok(stage.arena_mut().add(band))
}

/// A window up the panel, |h| < t.
fn panel_window(stage: &mut Stage<'_>, t: f64) -> fmn::Result<Mob> {
    let p = SLOPES.frame;
    let (x0, x1) = (SLOPES.p(-t, 0.0)[0], SLOPES.p(t, 0.0)[0]);
    let window = rect(v(x0, p.bottom()), x1 - x0, p.height, flat(moving(), 0.22))?;
    Ok(stage.arena_mut().add(window))
}

/// The tolerance's label: over the band when it is around the limit, under
/// it when it is around the wrong fill (`under`), detached.
fn band_label(kit: &Kit, stage: &mut Stage<'_>, t: &str, centre: f64, under: bool) -> fmn::Result<Mob> {
    let src = format!(r"\varepsilon = {}", tex_frac(t));
    let at = if under {
        SLOPES.p(LABEL_H, centre - LABEL_UNDER - 0.1)
    } else {
        SLOPES.p(LABEL_H, LABEL_ABOVE)
    };
    kit.tex_left(stage, &src, 26.0, limit_color(), &[], at)
}

/// The window's label at the panel's foot, detached.
fn window_label(kit: &Kit, stage: &mut Stage<'_>, d: &str) -> fmn::Result<Mob> {
    let foot = SLOPES.p(0.0, SLOPES.frame.y.0);
    kit.tex(
        stage,
        &format!(r"\delta = {}", tex_frac(d)),
        26.0,
        moving(),
        &[],
        v(foot[0], foot[1] + WINDOW_LABEL_UP),
    )
}

/// Where a pick at h lands on the panel: the graph's value there (h ≠ 0).
fn landing(h: Frac) -> fmn::Result<Vec3> {
    let s = chord_slope(BASE, h).map_err(fail)?;
    Ok(SLOPES.p(h.f64(), s.f64()))
}

/// A ring around a pick that missed the band, detached.
fn miss_ring(stage: &mut Stage<'_>, at: Vec3) -> Mob {
    let ring = Circle::new()
        .radius(MISS_R)
        .arc_center(at)
        .style(line_style(fails(), 4.0))
        .build();
    stage.arena_mut().add(ring)
}

/// A pick's readout, detached.
fn readout(kit: &Kit, stage: &mut Stage<'_>, text: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.text_left(stage, text, 20.0, color, READOUT)
}

/// The i-th line over the readout, detached.
fn over_readout(kit: &Kit, stage: &mut Stage<'_>, src: &str, i: usize, color: Srgb) -> fmn::Result<Mob> {
    kit.tex_left(stage, src, 22.0, color, &[], v(READOUT[0], OVER_READOUT[i]))
}

/// "h = −0.08: slope 0.92" from a pick's facts.
fn pick_text(h: &str, s: &str) -> String {
    format!("h = {}: {SLOPE} {s}", shown(h))
}

/// The square's piece over the window (x_n, 1], drawn over the curve.
fn passing_piece(stage: &mut Stage<'_>, from: f64) -> fmn::Result<Mob> {
    let pts = (0..=PASS_SAMPLES)
        .map(|k| {
            let x = if k == PASS_SAMPLES {
                1.0
            } else {
                from + (1.0 - from) * k as f64 / PASS_SAMPLES as f64
            };
            Ok(PLOT.p(x, power(x, SQUARE).map_err(fail)?))
        })
        .collect::<fmn::Result<Vec<Vec3>>>()?;
    let piece = polyline(pts).style(line_style(Palette::hero(), 7.0)).build();
    Ok(stage.arena_mut().add(piece))
}

/// An end mark on the plot's x axis: filled when the end belongs to the
/// window, hollow when it does not, detached.
fn end_mark(stage: &mut Stage<'_>, x: f64, belongs: bool) -> Mob {
    let style = if belongs {
        flat(moving(), 1.0)
    } else {
        Style::default().fill(hex(BACKGROUND), 1.0).stroke(moving(), 3.0, 1.0)
    };
    let c = Circle::new()
        .radius(MARK_R)
        .arc_center(PLOT.p(x, 0.0))
        .style(style)
        .build();
    stage.arena_mut().add(c)
}

impl SceneConstruct for TheChallengeGame {
    fn name(&self) -> &str {
        "04_the_challenge_game"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let f = facts().map_err(fail)?;
        let fin = final_picture(&kit, stage, &f)?;
        let zero = fact(&f, "C3.h.to")?;
        let target = fact(&f, "C3.limit")?;
        let level = chord_limit(BASE).map_err(fail)?.f64();

        // 04_1: the limit, the card and the panel's heading go; the three
        // rules come in, one with each turn
        let line = say(&kit, stage, "04_1")?;
        fade_out_all(stage, &fin.clear, 0.6)?;
        let rules = [
            kit.tex_left(
                stage,
                &format!(r"{FIRST}\varepsilon > {zero}"),
                22.0,
                limit_color(),
                &[],
                v(RULE_X, RULE_Y[0]),
            )?,
            kit.tex_left(
                stage,
                &format!(r"{THEN_DEFENDER}\delta > {zero}"),
                22.0,
                moving(),
                &[],
                v(RULE_X, RULE_Y[1]),
            )?,
            kit.tex_left(
                stage,
                &format!(r"{THEN_CHALLENGER}h,\ {zero} < |h| < \delta"),
                22.0,
                Palette::ink(),
                &[],
                v(RULE_X, RULE_Y[2]),
            )?,
        ];
        for (rule, at) in rules.iter().zip([TOLERANCE_AT, WINDOW_AT, PICK_AT]) {
            line.wait_for(stage, at)?;
            fade_in_all(stage, &[*rule], 0.5)?;
        }

        // 04_2: the challenger's 1/10, the defender's 1/10; picks from both
        // sides land in the band; why every pick does
        let line = say(&kit, stage, "04_2")?;
        line.wait_for(stage, AGAINST_AT)?;
        let eps = fact(&f, "C4.eps")?;
        let t = CHALLENGE.f64();
        let mut band = panel_band(stage, level, t)?;
        let mut band_tag = band_label(&kit, stage, &eps, level, false)?;
        let mut anims = vec![stage.prepare(fmn::animation::replacement_transform(fin.band, band).rt(0.6))?];
        anims.extend(swap(stage, &[fin.labels[0]], &[band_tag], 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, ANSWERS_AT)?;
        let honest = window(CHALLENGE).map_err(fail)?;
        let mut window_now = panel_window(stage, honest.f64())?;
        let mut window_tag = window_label(&kit, stage, &fact(&f, "C4.delta")?)?;
        let mut anims = vec![stage.prepare(fmn::animation::replacement_transform(fin.window, window_now).rt(0.6))?];
        anims.extend(swap(stage, &[fin.labels[1]], &[window_tag], 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, EVERY_AT)?;
        let first = landing(PICKS[0])?;
        let pick = SLOPES.dot(
            stage,
            PICKS[0].f64(),
            chord_slope(BASE, PICKS[0]).map_err(fail)?.f64(),
            moving(),
        );
        let mut shown_now = readout(
            &kit,
            stage,
            &pick_text(&fact(&f, "C4.pick.0")?, &fact(&f, "C4.pick_slope.0")?),
            Palette::ink(),
        )?;
        fade_in_all(stage, &[pick, shown_now], HOP_S)?;
        glow(&kit, stage, first, 0.15)?;
        for (i, &h) in PICKS.iter().enumerate().skip(1) {
            let next = readout(
                &kit,
                stage,
                &pick_text(
                    &fact(&f, &format!("C4.pick.{i}"))?,
                    &fact(&f, &format!("C4.pick_slope.{i}"))?,
                ),
                Palette::ink(),
            )?;
            let mut anims = vec![stage.prepare(move_to(pick, landing(h)?, HOP_S)?)?];
            anims.extend(swap(stage, &[shown_now], &[next], HOP_S)?);
            stage.play_prepared(anims)?;
            shown_now = next;
            glow(&kit, stage, landing(h)?, 0.15)?;
        }
        line.wait_for(stage, SINCE_AT)?;
        let every = kit.tex_left(
            stage,
            &format!(r"|Q(h) - {target}| = |h| < \delta = \varepsilon"),
            28.0,
            slope_color(),
            &[],
            v(RIGHT_X, READ_Y),
        )?;
        fade_in_all(stage, &[every], 0.5)?;

        // 04_3: a lucky pick lands; the wider window 1/5 loses at 0.15,
        // window_failure's pick, whose slope 1.15 misses the band
        let line = say(&kit, stage, "04_3")?;
        line.wait_for(stage, CATCHING_AT)?;
        let lucky = readout(
            &kit,
            stage,
            &pick_text(&fact(&f, "C4.lucky")?, &fact(&f, "C4.lucky_slope")?),
            Palette::ink(),
        )?;
        let mut anims = vec![stage.prepare(move_to(pick, landing(LUCKY)?, HOP_S)?)?];
        anims.extend(swap(stage, &[shown_now], &[lucky], HOP_S)?);
        stage.play_prepared(anims)?;
        shown_now = lucky;
        line.wait_for(stage, FIFTH_AT)?;
        let wide_window = panel_window(stage, WIDE.f64())?;
        let wide_tag = window_label(&kit, stage, &fact(&f, "C4.wide")?)?;
        let mut anims = vec![stage.prepare(fmn::animation::replacement_transform(window_now, wide_window).rt(0.6))?];
        anims.extend(swap(stage, &[window_tag, every], &[wide_tag], 0.6)?);
        stage.play_prepared(anims)?;
        window_now = wide_window;
        window_tag = wide_tag;
        line.wait_for(stage, OUTSIDE_AT)?;
        let catch = window_failure(BASE, CHALLENGE, WIDE)
            .map_err(fail)?
            .ok_or_else(|| fail("the wide window must lose".to_owned()))?;
        let caught = readout(
            &kit,
            stage,
            &pick_text(&fact(&f, "C4.catch")?, &fact(&f, "C4.catch_slope")?),
            fails(),
        )?;
        let mut anims = vec![stage.prepare(move_to(pick, landing(catch)?, HOP_S)?)?];
        anims.extend(swap(stage, &[shown_now], &[caught], HOP_S)?);
        stage.play_prepared(anims)?;
        let mut ring = miss_ring(stage, landing(catch)?);
        fade_in_all(stage, &[ring], 0.3)?;
        shown_now = caught;

        // 04_4: the limit leaves h = 0 out; continuity puts it in and aims at
        // the value there; filled with 1, continuous; filled with 1.5, the
        // honest window loses at fill_failure's pick
        let line = say(&kit, stage, "04_4")?;
        let back = panel_window(stage, honest.f64())?;
        let back_tag = window_label(&kit, stage, &fact(&f, "C4.delta")?)?;
        let mut anims = vec![stage.prepare(fmn::animation::replacement_transform(window_now, back).rt(0.6))?];
        anims.extend(swap(stage, &[window_tag, pick, ring, shown_now], &[back_tag], 0.6)?);
        stage.play_prepared(anims)?;
        window_now = back;
        window_tag = back_tag;
        line.wait_for(stage, LEAVES_AT)?;
        let limit_def = kit.tex_left(
            stage,
            &format!(r"\text{{limit: }} {zero} < |h| < \delta \Rightarrow |Q(h) - {target}| < \varepsilon"),
            26.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, READ_Y),
        )?;
        fade_in_all(stage, &[limit_def], 0.5)?;
        glow(&kit, stage, SLOPES.p(0.0, level), 0.2)?;
        line.wait_for(stage, PUTS_AT)?;
        let continuity_def = kit.tex_left(
            stage,
            &format!(r"\text{{continuity: }} |h| < \delta \Rightarrow |g(h) - g({zero})| < \varepsilon"),
            26.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, NEXT_Y),
        )?;
        let card_text = format!(
            "a limit at a checks only the domain's points other than a, and needs some in every window; continuity \
             at a point a of the domain checks a too, against f(a). Here g(h) = {} for h ≠ {zero}, and g({zero}) is \
             the filled value.",
            fact(&f, "C3.quotient")?
        );
        let card = precisely(&kit, stage, &card_text)?;
        // the game is continuity's now: the third rule lets in h = 0
        let rule_three = kit.tex_left(
            stage,
            &format!(r"{THEN_CHALLENGER}h,\ |h| < \delta"),
            22.0,
            Palette::ink(),
            &[],
            v(RULE_X, RULE_Y[2]),
        )?;
        let anims = swap(stage, &[rules[2]], &[continuity_def, card, rule_three], 0.6)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, FILL_ONE_AT)?;
        let [good, bad] = FILLS;
        if fill_failure(BASE, good, CHALLENGE, honest).map_err(fail)?.is_some() {
            return Err(fail("the hole filled with the limit must be continuous".to_owned()));
        }
        let filled = Dot::new()
            .point(SLOPES.p(0.0, good.f64()))
            .radius(END_R)
            .style(flat(slope_color(), 1.0))
            .build();
        let filled = stage.arena_mut().add(filled);
        fade_in_all(stage, &[filled], 0.5)?;
        line.wait_for(stage, FILL_HALF_AT)?;
        let wrong_band = panel_band(stage, bad.f64(), t)?;
        let wrong_tag = band_label(&kit, stage, &eps, bad.f64(), true)?;
        let mut anims = vec![
            stage.prepare(move_to(filled, SLOPES.p(0.0, bad.f64()), 0.7)?)?,
            stage.prepare(fmn::animation::replacement_transform(band, wrong_band).rt(0.7))?,
        ];
        anims.extend(swap(stage, &[band_tag], &[wrong_tag], 0.7)?);
        stage.play_prepared(anims)?;
        band = wrong_band;
        band_tag = wrong_tag;
        let witness = fill_failure(BASE, bad, CHALLENGE, honest)
            .map_err(fail)?
            .ok_or_else(|| fail("the wrong fill must lose".to_owned()))?;
        let pick = SLOPES.dot(
            stage,
            witness.f64(),
            chord_slope(BASE, witness).map_err(fail)?.f64(),
            moving(),
        );
        ring = miss_ring(stage, landing(witness)?);
        let witness_text = format!(
            "h = {}: g = {}",
            shown(&fact(&f, "C4.fill.witness")?),
            fact(&f, "C4.fill.witness_value")?
        );
        shown_now = readout(&kit, stage, &witness_text, fails())?;
        // no window works: every δ holds this kind of pick, the one shown
        // being δ = 1/10's
        let argument = [
            over_readout(
                &kit,
                stage,
                &format!(r"{ANY}{})", tex_frac(&fact(&f, "C4.fill.room")?)),
                0,
                Palette::ink(),
            )?,
            over_readout(
                &kit,
                stage,
                &format!(
                    r"|g(h) - {}| \ge {} > \varepsilon",
                    fact(&f, "C4.fill.bad")?,
                    tex_frac(&fact(&f, "C4.fill.gap")?)
                ),
                1,
                fails(),
            )?,
        ];
        fade_in_lagged(stage, &[pick, ring, shown_now, argument[0], argument[1]], 1.0, 0.3)?;

        // 04_5: the prediction; the answer stays hidden while the ring drains
        say(&kit, stage, "04_5")?;
        let ask = kit.text(stage, "?", 60.0, Palette::hero(), RING_AT)?;
        fade_in_all(stage, &[ask], 0.3)?;
        think_pause(&kit, stage, RING_AT, RING_R)?;

        // 04_6: seeing the pick, the defender could hide it outside a window
        // too narrow to hold it; so δ never depends on the later h
        let line = say(&kit, stage, "04_6")?;
        fade_out_all(stage, &[ask], 0.3)?;
        line.wait_for(stage, NARROW_AT)?;
        let blind = blind_window(witness).map_err(fail)?;
        let blind_window_now = panel_window(stage, blind.f64())?;
        let blind_tag = window_label(&kit, stage, &fact(&f, "C4.cheat.delta")?)?;
        let hidden = readout(
            &kit,
            stage,
            &format!("h = {}: {NOT_IN}", shown(&fact(&f, "C4.fill.witness")?)),
            Palette::dim(),
        )?;
        // h = 0 is in every window, but it passes: g(0) is within ε of itself
        let note = [
            over_readout(&kit, stage, &format!(r"h = {zero}{EVERY_WINDOW}"), 0, Palette::dim())?,
            over_readout(
                &kit,
                stage,
                &format!(r"|g({zero}) - g({zero})| = {zero} < \varepsilon"),
                1,
                Palette::dim(),
            )?,
        ];
        let mut anims =
            vec![stage.prepare(fmn::animation::replacement_transform(window_now, blind_window_now).rt(0.7))?];
        anims.extend(swap(
            stage,
            &[window_tag, shown_now, ring, argument[0], argument[1]],
            &[blind_tag, hidden, note[0], note[1]],
            0.7,
        )?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, DEPEND_AT)?;
        let rule = kit.tex(stage, DEPENDS, 24.0, Palette::ink(), &[], v(0.0, RULE_LINE_Y))?;
        let anims = swap(stage, &[card], &[rule], 0.6)?;
        stage.play_prepared(anims)?;
        glow(&kit, stage, v(RULE_X - 0.2, RULE_Y[1]), 0.12)?;

        // 04_7: the panel goes; continuity of x² at x = 1, the end of its
        // domain, against 1/2: the window from 2^(−1/2), left out, to 1, held
        let line = say(&kit, stage, "04_7")?;
        let mut leaving = fin.on_plot.clone();
        leaving.extend(&fin.panel);
        leaving.extend(fin.graph);
        leaving.extend([
            fin.hole,
            filled,
            pick,
            hidden,
            band,
            band_tag,
            blind_window_now,
            blind_tag,
        ]);
        leaving.extend([rules[0], rules[1], rule_three, note[0], note[1]]);
        leaving.extend([limit_def, continuity_def, rule]);
        fade_out_all(stage, &leaving, 0.6)?;
        line.wait_for(stage, END_AT)?;
        let domain = kit.text_left(
            stage,
            &end_domain_text(&f)?,
            22.0,
            Palette::dim(),
            v(RIGHT_X, END_ROWS[0]),
        )?;
        fade_in_all(stage, &[domain], 0.5)?;
        glow(&kit, stage, PLOT.p(1.0, 1.0), 0.2)?;
        line.wait_for(stage, HALF_AT)?;
        // the band is |y − 1| < 1/2: from the level 1 − 1/2 up past the top
        let band_at_end = end_band(stage, &f)?;
        let end_eps = kit.tex_left(
            stage,
            &end_eps_tex(&f)?,
            28.0,
            limit_color(),
            &[],
            v(RIGHT_X, END_ROWS[1]),
        )?;
        fade_in_all(stage, &[band_at_end, end_eps], 0.5)?;
        line.wait_for(stage, STARTS_AT)?;
        let edge = half_height(SQUARE).map_err(fail)?;
        let window_at_end = end_window(&kit, stage, &f)?;
        fade_in_all(stage, &window_at_end, 0.6)?;
        line.wait_for(stage, PASSES_AT)?;
        let piece = passing_piece(stage, edge)?;
        let rows = end_rows(&kit, stage, &f)?;
        let mut ins = vec![piece];
        ins.extend(rows);
        fade_in_lagged(stage, &ins, 1.2, 0.25)?;

        // 04_8: every answer covers every input of its window; finding one is
        // the next chapter's search
        let line = say(&kit, stage, "04_8")?;
        line.wait_for(stage, COVERS_AT)?;
        glow(&kit, stage, PLOT.p(f64::midpoint(edge, 1.0), 0.5), 0.25)?;
        line.wait_for(stage, SEARCH_AT)?;
        let next = kit.text(stage, NEXT, 22.0, Palette::dim(), v(0.0, RULE_LINE_Y))?;
        fade_in_all(stage, &[next], 0.6)?;
        kit.hold(stage)?;
        kit.music_breath(stage, 0.8)?;
        kit.finish(stage)
    }
}

/// 04_7's domain note, from facts.
fn end_domain_text(f: &Facts) -> fmn::Result<String> {
    Ok(format!(
        "{DOMAIN} [{}, {}]: {BEYOND} {}",
        fact(f, "C1.domain.left")?,
        fact(f, "C1.domain.right")?,
        fact(f, "C4.end.x")?
    ))
}

/// 04_7's tolerance around 1, from facts.
fn end_eps_tex(f: &Facts) -> fmn::Result<String> {
    Ok(format!(
        r"\varepsilon = {} \text{{ around }} {}",
        tex_frac(&fact(f, "C4.end.eps")?),
        fact(f, "C4.end.x")?
    ))
}

/// 04_7's band on the plot, |y − 1| < 1/2, from the level up past the top,
/// detached.
fn end_band(stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<Mob> {
    let lo = PLOT.p(0.0, level_frac(&fact(f, "C4.end.level")?)?.f64());
    let hi = PLOT.p(1.0, 1.0 + BAND_PAST);
    let band = rect(lo, hi[0] - lo[0], hi[1] - lo[1], flat(limit_color(), 0.22))?;
    Ok(stage.arena_mut().add(band))
}

/// 04_7's window on the plot, from 2^(−1/2) to 1, its end marks and its
/// edge's label, detached.
fn end_window(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<[Mob; 4]> {
    let edge = half_height(SQUARE).map_err(fail)?;
    let w_lo = PLOT.p(edge, 0.0);
    let w_hi = PLOT.p(1.0, 1.0 + BAND_PAST);
    let window = rect(w_lo, w_hi[0] - w_lo[0], w_hi[1] - w_lo[1], flat(moving(), 0.22))?;
    let window = stage.arena_mut().add(window);
    let marks = [end_mark(stage, edge, false), end_mark(stage, 1.0, true)];
    let edge_label = kit.tex(
        stage,
        &format!(r"{} \approx {}", end_root(f)?, fact(f, "C4.end.edge.4dp")?),
        22.0,
        moving(),
        &[],
        v(w_lo[0], w_lo[1] - NUMBER_DROP),
    )?;
    Ok([window, marks[0], marks[1], edge_label])
}

/// 2^(−1/n) for the end's power, as TeX.
fn end_root(f: &Facts) -> fmn::Result<String> {
    Ok(format!(r"2^{{-1/{}}}", fact(f, "C4.end.power")?))
}

/// 04_7's four rows, detached.
fn end_rows(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<[Mob; 4]> {
    let one = fact(f, "C4.end.x")?;
    let n = fact(f, "C4.end.power")?;
    let half = tex_frac(&fact(f, "C4.end.eps")?);
    let lvl = tex_frac(&fact(f, "C4.end.level")?);
    let root = end_root(f)?;
    Ok([
        kit.tex_left(
            stage,
            &format!(r"{root} < x \le {one}:\quad {lvl} < x^{{{n}}} \le {one}"),
            28.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, END_ROWS[2]),
        )?,
        kit.tex_left(
            stage,
            &format!(r"\text{{so }} |x^{{{n}}} - {one}| < {half} \text{{ for every such }} x"),
            28.0,
            Palette::ink(),
            &[],
            v(RIGHT_X, END_ROWS[3]),
        )?,
        kit.tex_left(
            stage,
            &format!(r"x = {root}:\ x^{{{n}}} = {lvl}{EXACTLY}"),
            24.0,
            Palette::dim(),
            &[],
            v(RIGHT_X, END_ROWS[4]),
        )?,
        kit.tex_left(
            stage,
            &format!(r"\delta = {one} - {root} \approx {}", fact(f, "C4.end.delta.4dp")?),
            28.0,
            moving(),
            &[],
            v(RIGHT_X, END_ROWS[5]),
        )?,
    ])
}

/// This chapter's last picture, for the chapter after it: everything 04_7
/// and 04_8 leave on screen. (The plot's axes, the square and its label stay
/// in the scene and are not returned.)
pub(super) struct End {
    /// What the chapter after clears: the domain note, the band within 1/2
    /// of 1 and its tolerance, the window from 2^(−1/2) to 1 with its end
    /// marks and its edge's label, the lit piece, the four rows and the
    /// "next:" line.
    pub clear: Vec<Mob>,
}

/// This chapter's last picture, rebuilt with its own builders in the state
/// 04_8 leaves it, back to front as 04_7 and 04_8 add it: the plot (axes,
/// the square, its label); the domain note; the band and its tolerance; the
/// window, its end marks and its edge's label; the lit piece and the rows;
/// the "next:" line.
pub(super) fn end_picture(kit: &Kit, stage: &mut Stage<'_>, f: &Facts) -> fmn::Result<End> {
    let mut plot = PLOT.axes(kit, stage)?;
    plot.push(PLOT.curve(stage, SQUARE, curve_color(), BRIGHT_W)?);
    plot.push(curve_label(kit, stage, SQUARE)?);
    let domain = kit.text_left(
        stage,
        &end_domain_text(f)?,
        22.0,
        Palette::dim(),
        v(RIGHT_X, END_ROWS[0]),
    )?;
    let band = end_band(stage, f)?;
    let eps = kit.tex_left(
        stage,
        &end_eps_tex(f)?,
        28.0,
        limit_color(),
        &[],
        v(RIGHT_X, END_ROWS[1]),
    )?;
    let window = end_window(kit, stage, f)?;
    let piece = passing_piece(stage, half_height(SQUARE).map_err(fail)?)?;
    let rows = end_rows(kit, stage, f)?;
    let next = kit.text(stage, NEXT, 22.0, Palette::dim(), v(0.0, RULE_LINE_Y))?;
    let mut clear = vec![domain, band, eps];
    clear.extend(window);
    clear.push(piece);
    clear.extend(rows);
    clear.push(next);
    for &m in plot.iter().chain(&clear) {
        stage.add_to_scene(m)?;
    }
    Ok(End { clear })
}

/// A fact's fraction ("1/2") as an exact fraction.
fn level_frac(s: &str) -> fmn::Result<Frac> {
    let bad = || fail(format!("{s} is not a fraction"));
    let (n, d) = s.split_once('/').ok_or_else(bad)?;
    let n: i64 = n.parse().map_err(|_| bad())?;
    let d: i64 = d.parse().map_err(|_| bad())?;
    Frac::whole(n).over(Frac::whole(d)).map_err(fail)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every phrase a picture waits for is said once, in the line that
    /// waits for it, in the order the pictures take them.
    #[test]
    fn the_pictures_wait_on_words_the_script_says() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let waits = [
            ("04_1", TOLERANCE_AT),
            ("04_1", WINDOW_AT),
            ("04_1", PICK_AT),
            ("04_2", AGAINST_AT),
            ("04_2", ANSWERS_AT),
            ("04_2", EVERY_AT),
            ("04_2", SINCE_AT),
            ("04_3", CATCHING_AT),
            ("04_3", FIFTH_AT),
            ("04_3", OUTSIDE_AT),
            ("04_4", LEAVES_AT),
            ("04_4", PUTS_AT),
            ("04_4", FILL_ONE_AT),
            ("04_4", FILL_HALF_AT),
            ("04_6", NARROW_AT),
            ("04_6", DEPEND_AT),
            ("04_7", END_AT),
            ("04_7", HALF_AT),
            ("04_7", STARTS_AT),
            ("04_7", PASSES_AT),
            ("04_8", COVERS_AT),
            ("04_8", SEARCH_AT),
        ];
        for (id, phrase) in waits {
            assert_eq!(text(id).matches(phrase).count(), 1, "{id} says {phrase:?} once");
        }
        for pair in waits.windows(2).filter(|w| w[0].0 == w[1].0) {
            let line = text(pair[0].0);
            assert!(line.find(pair[0].1) < line.find(pair[1].1), "{pair:?}");
        }
    }

    /// The turns come in their order, the question is asked before its
    /// answer, and the δ-rule names what δ may depend on.
    #[test]
    fn the_turns_and_the_rule_are_said_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let text = |id: &str| script.text(id).unwrap();
        let one = text("04_1");
        let at = |p: &str| one.find(p).unwrap();
        assert!(at("tolerance") < at("window") && at("window") < at("picks any h"));
        assert!(one.contains("epsilon") && one.contains("delta") && one.ends_with("but zero."));
        assert!(text("04_5").ends_with('?') && text("04_6").starts_with("Yes"));
        assert!(text("04_6").contains("never on the later h"));
        assert!(DEPENDS.contains("never on the"));
        // one losing window decides nothing (04_3), so the wrong fill is
        // said to lose every window, and only a winning answer is said to
        // cover its window
        assert!(text("04_4").contains("no window works") && !text("04_4").contains("continuity fails"));
        assert!(text("04_8").starts_with("A winning answer covers every input"));
    }

    /// The picks land where the model puts them: inside the panel, clear of
    /// the hollow mark at h = 0 and of each other; the readout clears the
    /// graph and the band; the windows drawn are wider than the hollow mark.
    #[test]
    fn the_picks_and_windows_fit_the_panel() {
        let p = SLOPES.frame;
        let hole = SLOPES.p(0.0, chord_limit(BASE).unwrap().f64());
        let catch = window_failure(BASE, CHALLENGE, WIDE).unwrap().unwrap();
        let witness = fill_failure(BASE, FILLS[1], CHALLENGE, window(CHALLENGE).unwrap())
            .unwrap()
            .unwrap();
        for h in PICKS.into_iter().chain([LUCKY, catch, witness]) {
            let at = landing(h).unwrap();
            assert!(
                p.left() < at[0] && at[0] < p.right() && p.bottom() < at[1] && at[1] < p.top(),
                "{h}"
            );
            let gap = (at[0] - hole[0]).hypot(at[1] - hole[1]);
            assert!(gap > 0.07 + END_R + 0.02, "h = {h} sits {gap} from the hollow mark");
        }
        // the readout's box and the two lines over it stay under the graph
        // at every x they span, apart and over the window's label
        let width = 2.7;
        let top = OVER_READOUT[0] + 0.15;
        for k in 0..=10 {
            let x = READOUT[0] + width * f64::from(k) / 10.0;
            let h = (x - p.center[0]) / (p.width / (p.x.1 - p.x.0));
            if h < p.x.1 {
                assert!(SLOPES.p(0.0, 1.0 + h)[1] > top + 0.15, "x = {x}");
            }
        }
        const { assert!(OVER_READOUT[0] - OVER_READOUT[1] >= 0.3 && OVER_READOUT[1] - READOUT[1] >= 0.3) };
        // the witness's ring stays left of the lines
        let ring_right = landing(witness).unwrap()[0] + MISS_R;
        assert!(ring_right + 0.05 < READOUT[0], "{ring_right}");
        let sx = p.width / (p.x.1 - p.x.0);
        for d in [window(CHALLENGE).unwrap(), WIDE, blind_window(witness).unwrap()] {
            assert!(2.0 * d.f64() * sx > 2.0 * END_R + 0.05, "δ = {d}");
        }
        assert_eq!(level_frac("1/2").unwrap(), Frac::whole(1).over(Frac::whole(2)).unwrap());
    }
}
