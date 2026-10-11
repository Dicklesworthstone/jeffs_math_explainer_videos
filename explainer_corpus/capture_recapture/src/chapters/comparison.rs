//! Chapter 07, Feller's comparison: two neighbouring bars of 05's score
//! curve, their ratio, and the long counting formulas cancelling in drawn
//! strike-throughs (07_1); the ratio's factors in table words, 529 against
//! 528 then 576 against 578 (07_2); the same comparison in letters (07_3),
//! multiplied out to Mn − kN (07_4); the table's cells, ab − k·u (07_5);
//! 7 × 7 − 3 × 16 = 1, and back to Feller's lake (07_6).

use std::ops::Range;

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, play, v};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::balance::{fixed_labels, mosaic_picture, opening_lake};
use super::curve;
use super::draw::{Role, flat, line_style};
use super::lake::{Cell, Panel, cell_tints, mosaic_in};
use crate::exact::feller_ratio;
use crate::facts::TOY;

/// The step the chapter takes: from `TOP − 1` to the curve's top, then one
/// past it.
fn top() -> u64 {
    let (m, n, k) = TOY;
    m * n / k
}

/// TeX source built piece by piece, remembering where pieces sit (the
/// span map's byte ranges), so drawn marks can find their glyphs.
#[derive(Default)]
struct Src(String);

impl Src {
    fn push(&mut self, s: &str) -> Range<usize> {
        let a = self.0.len();
        self.0.push_str(s);
        a..self.0.len()
    }
}

/// The glyphs of `tex` whose source lies in `range`, as one box (lower
/// left, upper right).
fn glyph_box(stage: &Stage<'_>, tex: Mob, spans: &SpanMapData, range: &Range<usize>) -> Option<(Vec3, Vec3)> {
    let kids = children(stage, tex);
    let mut lo = [f64::INFINITY, f64::INFINITY, 0.0];
    let mut hi = [f64::NEG_INFINITY, f64::NEG_INFINITY, 0.0];
    let mut any = false;
    for (e, &kid) in spans.entries.iter().zip(&kids) {
        if e.start >= range.start && e.end <= range.end {
            let (a, b) = (stage.get_corner(kid, DL), stage.get_corner(kid, UR));
            lo = [lo[0].min(a[0]), lo[1].min(a[1]), 0.0];
            hi = [hi[0].max(b[0]), hi[1].max(b[1]), 0.0];
            any = true;
        }
    }
    any.then_some((lo, hi))
}

/// A drawn strike-through: a slash across a box.
fn slash(stage: &mut Stage<'_>, (lo, hi): (Vec3, Vec3), color: Srgb) -> fmn::Result<Mob> {
    let pad = 0.05;
    let l = Line::new(v(lo[0] - pad, lo[1] - pad), v(hi[0] + pad, hi[1] + pad))
        .style(line_style(color, 3.0))
        .build()?;
    Ok(stage.arena_mut().add(l))
}

/// A box's centre.
fn centre((lo, hi): (Vec3, Vec3)) -> Vec3 {
    v((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0)
}

/// `a·(a−1)·…·b` as TeX factors, each factor's range kept.
fn falling(src: &mut Src, from: u64, to: u64) -> Vec<(u64, Range<usize>)> {
    let mut out = Vec::new();
    for (i, f) in (to..=from).rev().enumerate() {
        if i > 0 {
            src.push(r" \cdot ");
        }
        out.push((f, src.push(&f.to_string())));
    }
    out
}

/// The mystery lake's mosaic for 07_5–07_6, on the right.
const TOY_PANEL: Panel = Panel {
    c: [3.5, -0.35, 0.0],
    w: 5.4,
    h: 4.2,
};

fn toy_cells() -> [Cell; 4] {
    let (m, n, k) = TOY;
    mosaic_in(TOY_PANEL, top() as f64, m as f64, k as f64, n as f64)
}

pub struct Comparison {
    pub kit: Kit,
}

impl SceneConstruct for Comparison {
    fn name(&self) -> &str {
        "07_feller_s_comparison"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let lake = top();
        let ratio_line = cancel(&kit, stage, lake)?;
        let step = steps(&kit, stage, lake, ratio_line)?;
        letters(&kit, stage, step)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 07_1: the curve as 05 left it, two neighbouring bars, and the ratio of
/// their counting formulas cancelling. Returns the survivors' fraction.
fn cancel(kit: &Kit, stage: &mut Stage<'_>, lake: u64) -> fmn::Result<(Mob, SpanMapData, Mob)> {
    let (m, n, k) = TOY;
    let f = curve::frame();
    let axis = curve::axis(kit, stage, &f)?;
    let bars: Vec<VMobject> = curve::bars(&f)?;
    let sizes: Vec<u64> = curve::sizes().collect();
    let bars_g = stage.arena_mut().add(v_group(bars));
    let bar_mobs = children(stage, bars_g);
    stage.add_to_scene(axis)?;
    stage.add_to_scene(bars_g)?;
    let lit: Vec<Mob> = sizes
        .iter()
        .zip(&bar_mobs)
        .filter(|&(&s, _)| s == lake - 1 || s == lake)
        .map(|(_, &b)| b)
        .collect();
    kit.say(stage, "07_1")?;
    // every other bar dims; the two neighbours light
    let mut dim: Vec<Box<dyn Animation>> = Vec::new();
    for (&s, &b) in sizes.iter().zip(&bar_mobs) {
        let a = b
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .set_opacity(if s == lake - 1 || s == lake { 1.0 } else { 0.18 })?;
        dim.push(stage.prepare(a)?);
    }
    stage.play_prepared(dim)?;
    // the two bars rise to the top left, the curve leaves
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for (&s, &b) in sizes.iter().zip(&bar_mobs) {
        if s == lake - 1 || s == lake {
            continue;
        }
        let a = fade_out(stage.arena_mut(), b, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    let a = fade_out(stage.arena_mut(), axis, ORIGIN, 1.0)?;
    out.push(stage.prepare(a)?);
    // the two neighbours become a small pair of bars at the top left, their
    // heights still in proportion to their scores
    let inset_h = 1.5 / curve::score(lake).max(1e-9);
    let mut pair_bars = Vec::new();
    for (i, (&b, size)) in lit.iter().zip([lake - 1, lake]).enumerate() {
        let h = inset_h * curve::score(size);
        let x = -6.15 + 0.75 * i as f64;
        let target = stage.arena_mut().add(
            Rectangle::new()
                .width(0.5)
                .height(h)
                .style(flat(Role::score(), 0.95))
                .build()?
                .moved_to(v(x, 1.45 + h / 2.0)),
        );
        out.push(stage.prepare(Transform::new(b, target).rt(1.0))?);
        pair_bars.push(b);
    }
    stage.play_prepared(out)?;
    let lo_word = kit.text(stage, &(lake - 1).to_string(), 22.0, Palette::dim(), v(0.0, 0.0))?;
    let hi_word = kit.text(stage, &lake.to_string(), 22.0, Palette::dim(), v(0.0, 0.0))?;
    let (b0, b1) = (stage.get_corner(lit[0], DOWN), stage.get_corner(lit[1], DOWN));
    stage.move_to(lo_word, v(b0[0], b0[1] - 0.22), ORIGIN);
    stage.move_to(hi_word, v(b1[0], b1[1] - 0.22), ORIGIN);
    let ratio_word = kit.tex(
        stage,
        &format!(r"\frac{{\text{{score}}({lake})}}{{\text{{score}}({})}}", lake - 1),
        40.0,
        Role::score(),
        &[],
        v(-2.6, 2.45),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), lo_word, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), hi_word, ORIGIN, 1.0)?.rt(0.5),
        write(stage.arena(), ratio_word).rt(0.8),
    );

    // the counting formulas: C(10,3)·C(N−10, 7)/C(N, 10) at N and N − 1
    let binom = |a: u64, b: u64| format!(r"\binom{{{a}}}{{{b}}}");
    let mut s1 = Src::default();
    s1.push(r"= \frac{");
    let first_m = s1.push(&binom(m, k));
    s1.push(&binom(lake - m, n - k));
    s1.push("}{");
    let second_m = s1.push(&binom(m, k));
    s1.push(&binom(lake - 1 - m, n - k));
    s1.push(r"} \times \frac{");
    s1.push(&binom(lake - 1, n));
    s1.push("}{");
    s1.push(&binom(lake, n));
    s1.push("}");
    let (line1, spans1) = kit.tex_spans(stage, &s1.0, 36.0, Palette::ink(), &[], v(0.9, 2.4))?;
    stage.move_to(line1, v(-1.55, 2.45), LEFT);
    play!(stage; write(stage.arena(), line1).rt(1.0));
    let mut strikes = Vec::new();
    for r in [&first_m, &second_m] {
        if let Some(bx) = glyph_box(stage, line1, &spans1, r) {
            strikes.push(slash(stage, bx, Palette::red())?);
        }
    }
    for &st in &strikes {
        stage.add_to_scene(st)?;
    }
    let mut draw: Vec<Box<dyn Animation>> = Vec::new();
    for &st in &strikes {
        draw.push(stage.prepare(show_creation(st).rt(0.4))?);
    }
    stage.play_prepared(draw)?;

    // the binomials as falling products: almost everything cancels
    let mut s2 = Src::default();
    s2.push(r"= \frac{");
    let a_top = falling(&mut s2, lake - m, lake - m - (n - k) + 1);
    s2.push("}{");
    let a_bot = falling(&mut s2, lake - 1 - m, lake - 1 - m - (n - k) + 1);
    s2.push(r"} \times \frac{");
    let b_top = falling(&mut s2, lake - 1, lake - n);
    s2.push("}{");
    let b_bot = falling(&mut s2, lake, lake - n + 1);
    s2.push("}");
    let (line2, spans2) = kit.tex_spans(stage, &s2.0, 34.0, Palette::ink(), &[], v(0.0, 0.6))?;
    if stage.get_width(line2) > 13.2 {
        stage.set_width(line2, 13.2, false);
    }
    play!(stage; write(stage.arena(), line2).rt(1.2));
    // matching factors cancel in pairs, one fraction at a time
    let mut pairs: Vec<(Range<usize>, Range<usize>)> = Vec::new();
    for (top_f, bot_f) in [(&a_top, &a_bot), (&b_top, &b_bot)] {
        for (fv, r) in top_f.iter() {
            if let Some((_, rb)) = bot_f.iter().find(|(bv, _)| bv == fv) {
                pairs.push((r.clone(), rb.clone()));
            }
        }
    }
    let mut cuts: Vec<Box<dyn Animation>> = Vec::new();
    let mut cut_mobs = Vec::new();
    for (ra, rb) in &pairs {
        for r in [ra, rb] {
            if let Some(bx) = glyph_box(stage, line2, &spans2, r) {
                let st = slash(stage, bx, Palette::red())?;
                stage.add_to_scene(st)?;
                cut_mobs.push(st);
                cuts.push(stage.prepare(show_creation(st).rt(0.3))?);
            }
        }
    }
    let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), cuts, 0.06)?;
    play!(stage; group.rt(2.2));
    // what survives
    let survivors = |fs: &[(u64, Range<usize>)], other: &[(u64, Range<usize>)]| -> Vec<u64> {
        fs.iter()
            .filter(|(fv, _)| !other.iter().any(|(ov, _)| ov == fv))
            .map(|(fv, _)| *fv)
            .collect()
    };
    let (sa_top, sa_bot) = (survivors(&a_top, &a_bot), survivors(&a_bot, &a_top));
    let (sb_top, sb_bot) = (survivors(&b_top, &b_bot), survivors(&b_bot, &b_top));
    let (num1, num2, den1, den2) = (sa_top[0], sb_top[0], sb_bot[0], sa_bot[0]);
    let (result, result_spans) = kit.tex_spans(
        stage,
        &format!(r"= \frac{{{num1} \times {num2}}}{{{den1} \times {den2}}}"),
        56.0,
        Palette::ink(),
        &[],
        v(0.0, -1.35),
    )?;
    play!(stage; fade_in(stage.arena_mut(), result, v(0.0, 0.2), 1.0)?.rt(0.8));
    let note = precisely(
        kit,
        stage,
        "L(N)/L(N−1) = (N−M)(N−n) / (N(N−M−n+k)), for N > M + n − k; code walks the curve the same way, one \
         multiply per step (in logs for big lakes, see 11_2)",
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
    kit.hold(stage)?;
    let mut clear: Vec<Box<dyn Animation>> = Vec::new();
    for mm in [line1, line2, note].into_iter().chain(strikes).chain(cut_mobs) {
        let a = fade_out(stage.arena_mut(), mm, ORIGIN, 1.0)?;
        clear.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        clear,
        PlayOverrides {
            run_time: Some(0.6),
            ..PlayOverrides::default()
        },
    )?;
    // the survivors' fraction stays for 07_2, with the bars and the ratio
    let pair = stage.arena_mut().add(VMobject::new());
    for mm in pair_bars.into_iter().chain([lo_word, hi_word, ratio_word]) {
        stage.attach(pair, mm)?;
    }
    stage.add_to_scene(pair)?;
    Ok((result, result_spans, pair))
}

/// 07_2's fraction for the step to lake size `lake`: its source and the
/// ranges of the four factors (top two, bottom two).
fn step_src(lake: u64) -> (String, [Range<usize>; 4], (u64, u64)) {
    let (m, n, k) = TOY;
    let (num, den) = feller_ratio(lake, TOY);
    let mut s = Src::default();
    s.push(&format!(r"\frac{{{num}}}{{{den}}} = \frac{{"));
    let a = s.push(&(lake - m).to_string());
    s.push(r" \times ");
    let b = s.push(&(lake - n).to_string());
    s.push("}{");
    let c = s.push(&lake.to_string());
    s.push(r" \times ");
    let d = s.push(&(lake + k - m - n).to_string());
    s.push("}");
    (s.0, [a, b, c, d], (num, den))
}

/// The four word labels under and over a fraction's factors.
fn factor_labels(
    kit: &Kit,
    stage: &mut Stage<'_>,
    tex: Mob,
    spans: &SpanMapData,
    ranges: &[Range<usize>; 4],
) -> fmn::Result<Vec<Mob>> {
    let words = [
        ("unmarked fish", Role::unmarked(), true, 0.0),
        ("fish the second net missed", Palette::ink(), true, 0.4),
        ("lake size", Role::dark(), false, 0.0),
        ("dark cell", Role::dark(), false, 0.0),
    ];
    let mut out = Vec::new();
    for (r, (s, color, above, lift)) in ranges.iter().zip(words) {
        let Some((lo, hi)) = glyph_box(stage, tex, spans, r) else {
            continue;
        };
        let x = (lo[0] + hi[0]) / 2.0;
        let y = if above {
            stage.get_top(tex)[1] + 0.25 + lift
        } else {
            stage.get_bottom(tex)[1] - 0.25
        };
        out.push(kit.text(stage, s, 22.0, color, v(x, y))?);
    }
    Ok(out)
}

/// What 07_2 leaves on screen for 07_3.
struct Step {
    frac: Mob,
    verdict: Mob,
    labels: Vec<Mob>,
    /// The two bars and their ratio, top left (they leave at 07_3).
    pair: Mob,
}

/// 07_2: 529 against 528 at the step to the top, 576 against 578 past it.
fn steps(kit: &Kit, stage: &mut Stage<'_>, lake: u64, survivors: (Mob, SpanMapData, Mob)) -> fmn::Result<Step> {
    let (survivors, survivor_spans, pair) = survivors;
    let at = v(0.6, -0.2);
    let (src_up, r_up, (num_up, den_up)) = step_src(lake);
    let (up, up_spans) = kit.tex_spans(stage, &src_up, 56.0, Palette::ink(), &[], at)?;
    kit.say(stage, "07_2")?;
    let morph = TransformMatchingTex::new(stage.arena_mut(), survivors, up, &survivor_spans, &up_spans)?;
    play!(stage; morph.rt(1.2));
    let labels = factor_labels(kit, stage, up, &up_spans, &r_up)?;
    let mut show: Vec<Box<dyn Animation>> = Vec::new();
    for &l in &labels {
        let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
        show.push(stage.prepare(a)?);
    }
    stage.play_prepared(show)?;
    // the bigger side lights, and the score goes up
    let num_range = 6..6 + num_up.to_string().len();
    let den_range = {
        let start = src_up.find("}{").map_or(0, |i| i + 2);
        start..start + den_up.to_string().len()
    };
    let winner = if num_up > den_up { &num_range } else { &den_range };
    if let Some(bx) = glyph_box(stage, up, &up_spans, winner) {
        let r = SurroundingRectangle::from_extent(Some(bx))
            .buff(0.08)
            .color(Palette::hero())
            .build();
        let ring = stage.arena_mut().add(r);
        play!(stage; show_creation(ring).rt(0.6));
        stage.wait(0.3)?;
        play!(stage; fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.4));
    }
    let verdict_up = kit.text(
        stage,
        &format!("{} → {}: the score goes up", lake - 1, lake),
        26.0,
        Palette::green(),
        v(0.6, -2.4),
    )?;
    play!(stage; fade_in(stage.arena_mut(), verdict_up, ORIGIN, 1.0)?.rt(0.5));
    // one step past the top
    let (src_down, _, (num_down, den_down)) = step_src(lake + 1);
    let (down, down_spans) = kit.tex_spans(stage, &src_down, 56.0, Palette::ink(), &[], at)?;
    let morph = TransformMatchingTex::new(stage.arena_mut(), up, down, &up_spans, &down_spans)?;
    play!(stage;
        morph.rt(1.2),
        fade_out(stage.arena_mut(), verdict_up, ORIGIN, 1.0)?.rt(0.5),
    );
    let den_range = {
        let start = src_down.find("}{").map_or(0, |i| i + 2);
        start..start + den_down.to_string().len()
    };
    let num_range = 6..6 + num_down.to_string().len();
    let winner = if num_down > den_down { &num_range } else { &den_range };
    if let Some(bx) = glyph_box(stage, down, &down_spans, winner) {
        let r = SurroundingRectangle::from_extent(Some(bx))
            .buff(0.08)
            .color(Palette::hero())
            .build();
        let ring = stage.arena_mut().add(r);
        play!(stage; show_creation(ring).rt(0.6));
        stage.wait(0.3)?;
        play!(stage; fade_out(stage.arena_mut(), ring, ORIGIN, 1.0)?.rt(0.4));
    }
    let verdict_down = kit.text(
        stage,
        &format!("{} → {}: the score goes down", lake, lake + 1),
        26.0,
        Palette::red(),
        v(0.6, -2.4),
    )?;
    play!(stage; fade_in(stage.arena_mut(), verdict_down, ORIGIN, 1.0)?.rt(0.5));
    kit.hold(stage)?;
    Ok(Step {
        frac: down,
        verdict: verdict_down,
        labels,
        pair,
    })
}

/// 07_3–07_6: letters, the expansion, the table's cells, and back.
fn letters(kit: &Kit, stage: &mut Stage<'_>, prev: Step) -> fmn::Result<()> {
    let (m, n, k) = TOY;
    let lake = top();
    let (frac, verdict, labels, pair) = (prev.frac, prev.verdict, prev.labels, prev.pair);
    // 07_3: the same fraction in letters, then top minus bottom
    let colors = [
        ("N", Role::dark()),
        ("M", Role::marked()),
        ("n", Role::second()),
        ("k", Role::marked()),
    ];
    let mut sf = Src::default();
    sf.push(r"\frac{");
    let fa = sf.push("(N-M)");
    let fb = sf.push("(N-n)");
    sf.push("}{");
    let fc = sf.push("N");
    let fd = sf.push("(N-M-n+k)");
    sf.push("}");
    let (frac_l, frac_spans) = kit.tex_spans(stage, &sf.0, 56.0, Palette::ink(), &colors, v(0.6, -0.2))?;
    kit.say(stage, "07_3")?;
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &l in labels.iter().chain([&verdict, &pair]) {
        let a = fade_out(stage.arena_mut(), l, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.4),
            ..PlayOverrides::default()
        },
    )?;
    // the numbers give way to the letters they stand for, in place
    play!(stage;
        fade_out(stage.arena_mut(), frac, ORIGIN, 1.0)?.rt(0.7),
        fade_in(stage.arena_mut(), frac_l, ORIGIN, 1.0)?.rt(0.9),
    );
    let labels_l = factor_labels(kit, stage, frac_l, &frac_spans, &[fa, fb, fc, fd])?;
    let mut show: Vec<Box<dyn Animation>> = Vec::new();
    for &l in &labels_l {
        let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
        show.push(stage.prepare(a)?);
    }
    stage.play_prepared(show)?;
    // top minus bottom, on one line
    let mut sl = Src::default();
    let la = sl.push("(N-M)");
    let lb = sl.push("(N-n)");
    sl.push(" - ");
    let lc = sl.push("N");
    let ld = sl.push("(N-M-n+k)");
    let (line, line_spans) = kit.tex_spans(stage, &sl.0, 56.0, Palette::ink(), &colors, v(0.0, 1.6))?;
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &l in &labels_l {
        let a = fade_out(stage.arena_mut(), l, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.3),
            ..PlayOverrides::default()
        },
    )?;
    let morph = TransformMatchingTex::new(stage.arena_mut(), frac_l, line, &frac_spans, &line_spans)?;
    play!(stage; morph.rt(1.2));
    let labels_1 = factor_labels(kit, stage, line, &line_spans, &[la, lb, lc, ld])?;
    let mut show: Vec<Box<dyn Animation>> = Vec::new();
    for &l in &labels_1 {
        let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
        show.push(stage.prepare(a)?);
    }
    stage.play_prepared(show)?;
    let q = predict_pause(kit, stage, v(5.6, 1.6))?;

    // 07_4: multiplied out, three pairs cancel, Mn − kN survives
    let mut se = Src::default();
    se.push("= ");
    let e1 = se.push("N^2");
    let e2 = se.push(" - Nn");
    let e3 = se.push(" - NM");
    se.push(" + Mn");
    let e5 = se.push(" - N^2");
    let e6 = se.push(" + NM");
    let e7 = se.push(" + Nn");
    se.push(" - kN");
    let (exp, exp_spans) = kit.tex_spans(stage, &se.0, 48.0, Palette::ink(), &colors, v(0.0, 0.25))?;
    kit.say(stage, "07_4")?;
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &l in labels_1.iter().chain([&q]) {
        let a = fade_out(stage.arena_mut(), l, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.3),
            ..PlayOverrides::default()
        },
    )?;
    play!(stage; write(stage.arena(), exp).rt(1.2));
    let mut slashes = Vec::new();
    for (a, b) in [(&e1, &e5), (&e2, &e7), (&e3, &e6)] {
        let mut pair_anims: Vec<Box<dyn Animation>> = Vec::new();
        for r in [a, b] {
            if let Some(bx) = glyph_box(stage, exp, &exp_spans, r) {
                let st = slash(stage, bx, Palette::red())?;
                stage.add_to_scene(st)?;
                slashes.push(st);
                pair_anims.push(stage.prepare(show_creation(st).rt(0.35))?);
            }
        }
        stage.play_prepared(pair_anims)?;
    }
    let rest = kit.tex(stage, "= Mn - kN", 52.0, Palette::ink(), &colors, v(0.0, -1.0))?;
    play!(stage; fade_in(stage.arena_mut(), rest, v(0.0, 0.2), 1.0)?.rt(0.7));
    let mn_value = (m * n) as i64 - (k * lake) as i64;
    let numbers = kit.tex(
        stage,
        &format!(
            r"{m} \times {n} - {k} \times {lake} = {} - {} = {mn_value}",
            m * n,
            k * lake
        ),
        44.0,
        Palette::ink(),
        &[],
        v(0.0, -2.05),
    )?;
    play!(stage; write(stage.arena(), numbers).rt(1.0));
    let rising = kit.text(
        stage,
        "positive: the score is still rising",
        26.0,
        Palette::green(),
        v(0.0, -2.85),
    )?;
    play!(stage; fade_in(stage.arena_mut(), rising, ORIGIN, 1.0)?.rt(0.5));
    kit.hold(stage)?;

    // 07_5: in the table's cells, ab − k·u
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for mm in [line, exp, numbers, rising].into_iter().chain(slashes) {
        let a = fade_out(stage.arena_mut(), mm, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    kit.say(stage, "07_5")?;
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.5),
            ..PlayOverrides::default()
        },
    )?;
    let cells = toy_cells();
    let backs: Vec<VMobject> = cells
        .iter()
        .zip(cell_tints())
        .map(|(c, t)| {
            Rectangle::new()
                .width(c.w)
                .height(c.h)
                .style(flat(t, 1.0))
                .build()
                .map(|r| r.moved_to(c.c))
        })
        .collect::<Result<_, _>>()?;
    let backs = stage.arena_mut().add(v_group(backs));
    let names = [
        ("caught twice", Role::marked()),
        ("first catch only", Role::marked()),
        ("second catch only", Role::second()),
        ("dark cell", Role::dark()),
    ];
    let mut name_mobs = Vec::new();
    for (c, (s, color)) in cells.iter().zip(names) {
        let t = kit.text(stage, s, 18.0, color, v(c.c[0], c.c[1] + c.h / 2.0 - 0.2))?;
        let w = stage.get_width(t);
        if w > c.w - 0.1 {
            stage.set_width(t, c.w - 0.1, false);
        }
        name_mobs.push(t);
    }
    let top_y = TOY_PANEL.c[1] + TOY_PANEL.h / 2.0 + 0.3;
    let head_m = kit.text(stage, "marked", 24.0, Role::marked(), v(cells[0].c[0], top_y))?;
    let head_u = kit.text(stage, "unmarked", 24.0, Palette::ink(), v(cells[2].c[0], top_y))?;
    let lake_word = kit.text(
        stage,
        &format!("the mystery lake at {lake}"),
        24.0,
        Role::dark(),
        v(TOY_PANEL.c[0], top_y + 0.45),
    )?;
    let mut appear: Vec<Box<dyn Animation>> = Vec::new();
    for mm in [backs, head_m, head_u, lake_word]
        .into_iter()
        .chain(name_mobs.iter().copied())
    {
        let a = fade_in(stage.arena_mut(), mm, ORIGIN, 1.0)?;
        appear.push(stage.prepare(a)?);
    }
    let slide = rest
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.8),
            ..AnimateArgs::default()
        })?
        .move_to(v(-3.6, 1.2), ORIGIN)?;
    appear.push(stage.prepare(slide)?);
    stage.play_prepared(appear)?;
    // Mn − kN = ab − k·u, and each letter flies to its cell
    let mut sc = Src::default();
    sc.push("= ");
    let ca = sc.push("a");
    let cb = sc.push("b");
    sc.push(r" - ");
    let ck = sc.push("k");
    sc.push(r"\cdot ");
    let cu = sc.push("u");
    let cell_colors = [
        ("a", Role::marked()),
        ("b", Role::second()),
        ("k", Role::marked()),
        ("u", Role::dark()),
    ];
    let (cells_line, cells_spans) = kit.tex_spans(stage, &sc.0, 52.0, Palette::ink(), &cell_colors, v(-3.6, 0.1))?;
    play!(stage; write(stage.arena(), cells_line).rt(0.8));
    let mut flies: Vec<Box<dyn Animation>> = Vec::new();
    let mut flown = Vec::new();
    for (r, (letter, color), cell) in [
        (&ca, ("a", Role::marked()), cells[1]),
        (&cb, ("b", Role::second()), cells[2]),
        (&ck, ("k", Role::marked()), cells[0]),
        (&cu, ("u", Role::dark()), cells[3]),
    ] {
        let Some(bx) = glyph_box(stage, cells_line, &cells_spans, r) else {
            continue;
        };
        let copy = kit.tex(stage, letter, 52.0, color, &[], centre(bx))?;
        stage.add_to_scene(copy)?;
        let a = copy
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                path_arc: Some(-0.6),
                ..AnimateArgs::default()
            })?
            .scale(1.3)?
            .move_to(v(cell.c[0], cell.c[1] - 0.1), ORIGIN)?;
        flies.push(stage.prepare(a)?);
        flown.push((copy, cell, color));
    }
    let group = AnimationGroup::with_lag_ratio(stage.arena_mut(), flies, 0.2)?;
    play!(stage; group.rt(2.0));
    kit.hold(stage)?;

    // 07_6: 7 × 7 − 3 × 16 = 1, the same one; back to Feller's lake
    let (a, b, u) = (m - k, n - k, lake + k - m - n);
    let values = [a, b, k, u];
    kit.say(stage, "07_6")?;
    let mut swaps: Vec<Box<dyn Animation>> = Vec::new();
    for ((copy, cell, color), value) in flown.iter().zip(values) {
        let num = kit.text(stage, &value.to_string(), 44.0, *color, v(cell.c[0], cell.c[1] - 0.1))?;
        swaps.push(stage.prepare(fmn::animation::replacement_transform(*copy, num).rt(0.8))?);
    }
    stage.play_prepared(swaps)?;
    let cells_value = (a * b) as i64 - (k * u) as i64;
    let cell_numbers = kit.tex(
        stage,
        &format!(
            r"= {a} \times {b} - {k} \times {u} = {} - {} = {cells_value}",
            a * b,
            k * u
        ),
        40.0,
        Palette::ink(),
        &[],
        v(-3.4, -1.0),
    )?;
    if stage.get_width(cell_numbers) > 6.6 {
        stage.set_width(cell_numbers, 6.6, false);
    }
    let before = kit.tex(
        stage,
        &format!(r"\text{{before: }} {} - {} = {mn_value}", m * n, k * lake),
        36.0,
        Palette::dim(),
        &[],
        v(-3.4, -2.0),
    )?;
    play!(stage; write(stage.arena(), cell_numbers).rt(1.0));
    play!(stage; fade_in(stage.arena_mut(), before, ORIGIN, 1.0)?.rt(0.6));
    stage.wait(0.6)?;
    // the toy table gives way to Feller's, as 08 opens on it
    fade_all(stage, 0.8)?;
    let feller = stage.arena_mut().add(mosaic_picture(&kit.book, opening_lake() as f64));
    let words = fixed_labels(kit, stage)?;
    play!(stage;
        fade_in(stage.arena_mut(), feller, ORIGIN, 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), words, ORIGIN, 1.0)?.rt(1.0),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use explainer_kit::choose;

    #[test]
    fn what_chapter_07_says_holds_exactly() {
        let (m, n, k) = TOY;
        let lake = top();
        assert_eq!(lake, 33);
        // 07_1: the ratio of the counting formulas is the falling products'
        // ratio, and that is (23 × 23)/(33 × 16)
        let lhs = choose(lake - m, n - k) * choose(lake - 1, n);
        let rhs = choose(lake - 1 - m, n - k) * choose(lake, n);
        let falling = |a: u64, b: u64| (b..=a).map(u128::from).product::<u128>();
        assert_eq!(
            lhs * falling(lake - 1 - m, lake - 1 - m - (n - k) + 1) * falling(lake, lake - n + 1),
            rhs * falling(lake - m, lake - m - (n - k) + 1) * falling(lake - 1, lake - n)
        );
        let (num, den) = feller_ratio(lake, TOY);
        assert_eq!(lhs * u128::from(den), rhs * u128::from(num));
        // 07_2: 23 × 23 over 33 × 16 is 529 against 528 (up); 576 against 578 (down)
        assert_eq!(((lake - m) * (lake - n), lake * (lake + k - m - n)), (529, 528));
        assert!(num > den);
        let (num2, den2) = feller_ratio(lake + 1, TOY);
        assert_eq!((num2, den2), (576, 578));
        assert!(num2 < den2);
        // 07_4: top − bottom = Mn − kN for every lake; here 100 − 99 = 1
        for nl in (m + n - k + 1)..200 {
            let (a, b) = feller_ratio(nl, TOY);
            assert_eq!(a as i64 - b as i64, (m * n) as i64 - (k * nl) as i64);
        }
        assert_eq!((m * n) as i64 - (k * lake) as i64, 1);
        // 07_6: 7 × 7 − 3 × 16 = 49 − 48 = 1, the same one
        let (a, b, u) = (m - k, n - k, lake + k - m - n);
        assert_eq!((a, b, u, a * b, k * u), (7, 7, 16, 49, 48));
    }

    #[test]
    fn the_strike_pairs_leave_the_four_survivors() {
        let (m, n, k) = TOY;
        let lake = top();
        let mut s = Src::default();
        let a_top = falling(&mut s, lake - m, lake - m - (n - k) + 1);
        let a_bot = falling(&mut s, lake - 1 - m, lake - 1 - m - (n - k) + 1);
        let b_top = falling(&mut s, lake - 1, lake - n);
        let b_bot = falling(&mut s, lake, lake - n + 1);
        let only = |x: &[(u64, Range<usize>)], y: &[(u64, Range<usize>)]| -> Vec<u64> {
            x.iter()
                .filter(|(v, _)| !y.iter().any(|(w, _)| w == v))
                .map(|(v, _)| *v)
                .collect()
        };
        assert_eq!(only(&a_top, &a_bot), vec![23]);
        assert_eq!(only(&a_bot, &a_top), vec![16]);
        assert_eq!(only(&b_top, &b_bot), vec![23]);
        assert_eq!(only(&b_bot, &b_top), vec![33]);
        // every recorded range spells its factor
        for (v, r) in a_top.iter().chain(&b_bot) {
            assert_eq!(&s.0[r.clone()], v.to_string());
        }
    }
}
