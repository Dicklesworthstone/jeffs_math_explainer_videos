//! Chapter 10, small experiments: a lake of 1,000 with 50 marked and 50
//! caught (10_1), the one time in fourteen that the second catch holds no
//! marked fish, and the soup guess dividing by zero (10_2), Chapman's
//! imaginary fish and the bias gauge (10_3). Every chance is exact
//! (`crate::exact::estimate_law`, oracle `af21_small_study_50`).

use std::f64::consts::PI;

use explainer_kit::props::{Pose, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::library::{AnnularSector, TransformMatchingTex, dashed_vmobject};
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use crate::exact::{EstimateLaw, estimate_law};
use crate::model::SplitMix64;

/// The small experiment: lake size, marked, caught (A-F21).
pub const SMALL: (u64, u64, u64) = (1_000, 50, 50);
/// One fish drawn per this many (the key says so).
const PER_GLYPH: u64 = 10;
/// Chapman's bias is under 2% once this many are expected caught twice
/// (the bound N·e^(−Mn/N), A-F18.bound_pct_at_expected_k4).
const EXPECT_ENOUGH: f64 = 4.0;
/// The pond, on the left.
const POND_C: Vec3 = [-3.6, -0.3, 0.0];
const POND_W: f64 = 6.0;
const POND_H: f64 = 4.4;
/// The gauge, on the right: centre, radius, and how many percent of the
/// true size a quarter turn stands for.
const GAUGE_C: Vec3 = [3.7, -0.55, 0.0];
const GAUGE_R: f64 = 2.0;
const GAUGE_PCT: f64 = 30.0;

/// The gauge's direction for a bias of `pct` percent: up for none,
/// clockwise for too high.
fn gauge_dir(pct: f64) -> (f64, f64) {
    let a = PI / 2.0 - (pct / GAUGE_PCT) * (PI / 2.0);
    (a.cos(), a.sin())
}

fn gauge_at(pct: f64, r: f64) -> Vec3 {
    let (c, s) = gauge_dir(pct);
    v(GAUGE_C[0] + r * c, GAUGE_C[1] + r * s)
}

/// A percent with its sign, one decimal unless whole after rounding.
fn signed_pct(pct: f64, places: usize) -> String {
    let body = format!("{:.*}", places, pct.abs());
    if pct < 0.0 {
        format!("\u{2212}{body}%")
    } else {
        format!("+{body}%")
    }
}

pub struct Small {
    pub kit: Kit,
}

impl SceneConstruct for Small {
    fn name(&self) -> &str {
        "10_small_experiments"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (lake, marked, caught) = SMALL;
        let law = estimate_law(lake, marked, caught);
        let pond = pond(&kit, stage)?;
        nothing_caught_twice(&kit, stage, &law, pond)?;
        chapman(&kit, stage, &law)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 10_1: the small lake; the second net comes up with no marked fish.
fn pond(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let (lake, marked, caught) = SMALL;
    let drawn = (lake / PER_GLYPH) as usize;
    let (gold_n, net_n) = ((marked / PER_GLYPH) as usize, (caught / PER_GLYPH) as usize);
    // the water and its bank
    let bank = stage.arena_mut().add(
        Rectangle::new()
            .width(POND_W + 0.3)
            .height(POND_H + 0.3)
            .corner_radius(0.5)
            .style(flat(Role::bank(), 1.0))
            .build()?
            .moved_to(POND_C),
    );
    let water = stage.arena_mut().add(
        Rectangle::new()
            .width(POND_W)
            .height(POND_H)
            .corner_radius(0.42)
            .style(flat(Role::water(), 1.0))
            .build()?
            .moved_to(POND_C),
    );
    // seeded places, fish facing every way; the first few are the marked
    let mut rng = SplitMix64::new(crate::model::SEED ^ 0x1001);
    let places: Vec<Pose> = (0..drawn)
        .map(|_| Pose {
            x: POND_C[0] + (POND_W - 0.6) * (rng.uniform() - 0.5),
            y: POND_C[1] + (POND_H - 0.6) * (rng.uniform() - 0.5),
            heading: 2.0 * PI * rng.uniform(),
        })
        .collect();
    // the net: the spot whose nearest fish are all unmarked, with no marked
    // fish inside its rim (an illustration of the case the question asks about)
    let dist = |p: &Pose, c: (f64, f64)| (p.x - c.0).hypot(p.y - c.1);
    let mut best: Option<((f64, f64), Vec<usize>, f64)> = None;
    for gx in 0..12 {
        for gy in 0..8 {
            let c = (
                POND_C[0] - POND_W / 2.0 + 0.9 + (POND_W - 1.8) * f64::from(gx) / 11.0,
                POND_C[1] - POND_H / 2.0 + 0.9 + (POND_H - 1.8) * f64::from(gy) / 7.0,
            );
            let mut order: Vec<usize> = (0..drawn).collect();
            order.sort_by(|&a, &b| dist(&places[a], c).total_cmp(&dist(&places[b], c)));
            let inside: Vec<usize> = order[..net_n].to_vec();
            let rim = dist(&places[inside[net_n - 1]], c) + 0.22;
            let clear = (0..gold_n).all(|g| dist(&places[g], c) > rim + 0.2);
            if clear && best.as_ref().is_none_or(|b| rim < b.2) {
                best = Some((c, inside, rim));
            }
        }
    }
    let Some((net_c, in_net, rim)) = best else {
        return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(
            "10_1: no spot in the pond for a net without marked fish",
        )));
    };
    let gold = Swarm::build(stage, &marked_fish().mob, &places[..gold_n], None)?;
    let blue = Swarm::build(stage, &fish(Role::unmarked()).mob, &places[gold_n..], None)?;
    let net_poses: Vec<Pose> = in_net.iter().map(|&i| places[i]).collect();
    let rings = Swarm::build(stage, &ring().mob, &net_poses, None)?;
    let net = stage.arena_mut().add(
        Circle::new()
            .radius(rim)
            .arc_center(v(net_c.0, net_c.1 - 3.0))
            .style(
                Style::default()
                    .fill(Role::second(), 0.1)
                    .stroke(Role::second(), 4.0, 1.0),
            )
            .build(),
    );
    // the words on the right
    let x0 = 0.9;
    let title = kit.text_left(
        stage,
        &format!("a lake of {} fish", thousands(lake)),
        32.0,
        Role::dark(),
        v(x0, 2.5),
    )?;
    let mark = kit.text_left(stage, &format!("mark {marked}"), 30.0, Role::marked(), v(x0, 1.8))?;
    let catch = kit.text_left(stage, &format!("catch {caught}"), 30.0, Role::second(), v(x0, 1.2))?;
    let key = kit.text(
        stage,
        &format!("each fish drawn = {PER_GLYPH} fish"),
        22.0,
        Palette::dim(),
        v(POND_C[0] + POND_W / 2.0 - 1.4, POND_C[1] - POND_H / 2.0 - 0.45),
    )?;
    kit.say(stage, "10_1")?;
    play!(stage;
        fade_in(stage.arena_mut(), bank, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), water, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), blue.group, ORIGIN, 1.0)?.rt(1.0),
        fade_in(stage.arena_mut(), key, ORIGIN, 1.0)?.rt(0.8),
    );
    // "In a lake of a thousand fish, mark fifty and catch fifty."
    kit.until_sentence(stage, "10_1", 1)?;
    play!(stage;
        fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), gold.group, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), mark, ORIGIN, 1.0)?.rt(0.8),
    );
    // "How often does the second catch hold no marked fish at all?"
    kit.until_sentence(stage, "10_1", 2)?;
    stage.add_to_scene(net)?;
    let rise = net
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.0),
            ..AnimateArgs::default()
        })?
        .move_to(v(net_c.0, net_c.1), ORIGIN)?;
    play!(stage; rise, fade_in(stage.arena_mut(), catch, ORIGIN, 1.0)?.rt(0.8));
    play!(stage; fade_in(stage.arena_mut(), rings.group, ORIGIN, 1.0)?.rt(0.6));
    let none = kit.text_left(
        stage,
        "marked fish in the second catch: 0",
        28.0,
        Palette::red(),
        v(x0, 0.2),
    )?;
    play!(stage; fade_in(stage.arena_mut(), none, ORIGIN, 1.0)?.rt(0.6));
    let q = predict_pause(kit, stage, v(5.9, 0.2))?;
    Ok(vec![
        bank,
        water,
        gold.group,
        blue.group,
        rings.group,
        net,
        title,
        mark,
        catch,
        key,
        none,
        q,
    ])
}

/// 10_2: one time in fourteen, and the soup guess divides by zero.
fn nothing_caught_twice(kit: &Kit, stage: &mut Stage<'_>, law: &EstimateLaw, pond: Vec<Mob>) -> fmn::Result<()> {
    let (_, marked, caught) = SMALL;
    let (q, none) = (pond[11], pond[10]);
    let one_in = kit.text_styled(
        stage,
        &format!("about 1 time in {}", law.one_in.round()),
        44.0,
        Palette::red(),
        v(3.7, -0.75),
        true,
        false,
    )?;
    let exact = kit.text(
        stage,
        &format!("chance of none: {:.1}%", 100.0 * law.p0),
        28.0,
        Palette::dim(),
        v(3.7, -1.45),
    )?;
    kit.say(stage, "10_2")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    play!(stage;
        fade_in(stage.arena_mut(), one_in, v(0.0, -0.15), 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), exact, ORIGIN, 1.0)?.rt(0.8),
    );
    // "Then the soup guess divides by zero.": the guess flies off the screen
    kit.until_sentence(stage, "10_2", 1)?;
    let zero = kit.tex(
        stage,
        &format!(r"\frac{{{marked} \times {caught}}}{{0}} = \;?"),
        48.0,
        Palette::ink(),
        &[("{0}", Palette::red())],
        v(3.0, -2.6),
    )?;
    play!(stage; write(stage.arena(), zero).rt(1.0));
    let dot = stage.arena_mut().add(
        Circle::new()
            .radius(0.12)
            .arc_center(v(4.7, -2.6))
            .style(flat(Palette::red(), 1.0))
            .build(),
    );
    stage.add_to_scene(dot)?;
    let off = dot
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.9),
            rate_func: Some(fmn::core::rate::rush_into),
            ..AnimateArgs::default()
        })?
        .move_to(v(8.0, -2.6), ORIGIN)?;
    stage.play(off)?;
    stage.remove_from_scene(dot);
    // what chapter 09's code does here (R3 Dev D6)
    let inf = crate::facts::py((marked * caught) as f64 / 0.0);
    let code_note = kit.text(
        stage,
        &format!(
            "for programmers: chapter 9's est = M * n / k gives {inf} here (a RuntimeWarning), so est.mean() is {inf} too"
        ),
        24.0,
        Palette::ink(),
        v(0.6, -3.4),
    )?;
    play!(stage; fade_in(stage.arena_mut(), code_note, ORIGIN, 1.0)?.rt(0.5));
    kit.hold(stage)?;
    // clear the stage for Chapman, keeping nothing
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for m in pond
        .into_iter()
        .filter(|&m| m != q)
        .chain([one_in, exact, zero, code_note])
    {
        if m == none {
            continue;
        }
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    let a = fade_out(stage.arena_mut(), none, ORIGIN, 1.0)?;
    out.push(stage.prepare(a)?);
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(0.8),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}

/// 10_3: the imaginary fish, Chapman's formula, the gauge.
fn chapman(kit: &Kit, stage: &mut Stage<'_>, law: &EstimateLaw) -> fmn::Result<()> {
    let (lake, marked, caught) = SMALL;
    // the table of this experiment (cells equal, not to scale)
    let (tx, ty, cw, ch) = (-2.8, 0.9, 2.4, 1.25);
    let cells = [
        v(tx - cw / 2.0, ty + ch / 2.0),
        v(tx - cw / 2.0, ty - ch / 2.0),
        v(tx + cw / 2.0, ty + ch / 2.0),
        v(tx + cw / 2.0, ty - ch / 2.0),
    ];
    let tints = super::lake::cell_tints();
    let backs: Vec<VMobject> = cells
        .iter()
        .zip(tints)
        .map(|(&c, t)| {
            Rectangle::new()
                .width(cw - 0.06)
                .height(ch - 0.06)
                .style(flat(t, 1.0))
                .build()
                .map(|r| r.moved_to(c))
        })
        .collect::<Result<_, _>>()?;
    let backs = stage.arena_mut().add(v_group(backs));
    let col_m = kit.text(stage, "marked", 26.0, Role::marked(), v(cells[0][0], ty + ch + 0.3))?;
    let col_u = kit.text(stage, "unmarked", 26.0, Palette::ink(), v(cells[2][0], ty + ch + 0.3))?;
    let row_s = kit.text(stage, "second catch", 24.0, Role::second(), v(0.0, cells[0][1]))?;
    stage.move_to(row_s, v(tx - cw - 0.2, cells[0][1]), RIGHT);
    let row_n = kit.text(stage, "not", 24.0, Palette::dim(), v(0.0, cells[1][1]))?;
    stage.move_to(row_n, v(tx - cw - 0.2, cells[1][1]), RIGHT);
    let k0 = kit.text(stage, "0", 40.0, Role::marked(), cells[0])?;
    let a = kit.text(stage, &thousands(marked), 36.0, Role::marked(), cells[1])?;
    let b = kit.text(stage, &thousands(caught), 36.0, Role::second(), cells[2])?;
    let dark = kit.text(stage, "?", 40.0, Palette::dim(), cells[3])?;
    // the counts the formula uses, under the table
    let counts_y = ty - ch - 0.45;
    let m_word = kit.text_left(
        stage,
        &format!("marked {marked}"),
        26.0,
        Role::marked(),
        v(tx - cw, counts_y),
    )?;
    let n_word = kit.text_left(
        stage,
        &format!("second catch {caught}"),
        26.0,
        Role::second(),
        v(tx - 0.15, counts_y),
    )?;
    kit.say(stage, "10_3")?;
    play!(stage;
        fade_in(stage.arena_mut(), backs, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), col_m, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), col_u, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), row_s, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), row_n, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), k0, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), a, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), b, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), dark, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), m_word, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), n_word, ORIGIN, 1.0)?.rt(0.8),
    );
    // the imaginary fish: a dashed gold outline drops into the caught-twice
    // cell, and every count it touches goes up by one
    let outline = fish(Role::marked())
        .mob
        .map_style_deep(|s| s.fill(Role::marked(), 0.12).stroke(Role::marked(), 3.0, 1.0));
    let parts: Vec<VMobject> = outline
        .children()
        .iter()
        .map(|c| dashed_vmobject(c, 14, 0.55, 0.0))
        .collect::<Result<_, _>>()?;
    let ghost = stage.arena_mut().add(v_group(parts));
    stage.scale(ghost, 5.0);
    stage.move_to(ghost, v(cells[0][0] + 0.62, 3.0), ORIGIN);
    let ghost_word = kit.text(
        stage,
        "an imaginary fish",
        24.0,
        Role::marked(),
        v(cells[0][0] + 2.2, 3.45),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), ghost, ORIGIN, 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), ghost_word, ORIGIN, 1.0)?.rt(0.5),
    );
    let drop = ghost
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(1.0),
            ..AnimateArgs::default()
        })?
        .move_to(v(cells[0][0] + 0.62, cells[0][1]), ORIGIN)?;
    stage.play(drop)?;
    let k1 = kit.text(stage, "0 + 1", 32.0, Role::marked(), v(cells[0][0] - 0.55, cells[0][1]))?;
    let m1 = kit.text_left(
        stage,
        &format!("marked {} + 1", marked),
        26.0,
        Role::marked(),
        v(tx - cw, counts_y),
    )?;
    let n1 = kit.text_left(
        stage,
        &format!("second catch {} + 1", caught),
        26.0,
        Role::second(),
        v(tx - 0.15, counts_y),
    )?;
    stage.move_to(n1, v(tx + 0.25, counts_y), LEFT);
    stage.move_to(m1, v(tx - cw - 0.6, counts_y), LEFT);
    play!(stage;
        fmn::animation::replacement_transform(k0, k1).rt(0.6),
        fmn::animation::replacement_transform(m_word, m1).rt(0.6),
        fmn::animation::replacement_transform(n_word, n1).rt(0.6),
    );
    // the soup guess becomes Chapman's
    let plain_src = r"\text{guess} = \frac{M\,n}{k}";
    let chap_src = r"\text{guess} = \frac{(M+1)(n+1)}{k+1} - 1";
    let colors = [("M", Role::marked()), ("n", Role::second()), ("k", Role::marked())];
    let (plain, plain_spans) = kit.tex_spans(stage, plain_src, 44.0, Palette::ink(), &colors, v(-3.4, -1.75))?;
    let (chap, chap_spans) = kit.tex_spans(stage, chap_src, 44.0, Palette::ink(), &colors, v(-3.4, -1.75))?;
    play!(stage; write(stage.arena(), plain).rt(0.8));
    let morph = TransformMatchingTex::new(stage.arena_mut(), plain, chap, &plain_spans, &chap_spans)?;
    play!(stage; morph.rt(1.6));
    // "It never divides by zero.": with none caught twice it still answers
    kit.until_sentence(stage, "10_3", 1)?;
    let at_zero = kit.text(
        stage,
        &format!(
            "none caught twice: {} × {} ÷ 1 − 1 = {}",
            marked + 1,
            caught + 1,
            thousands((marked + 1) * (caught + 1) - 1)
        ),
        24.0,
        Palette::ink(),
        v(-3.4, -2.45),
    )?;
    play!(stage; fade_in(stage.arena_mut(), at_zero, ORIGIN, 1.0)?.rt(0.6));
    // "...a number, not an answer." (R3 Dev D1)
    kit.until_sentence(stage, "10_3", 2)?;
    let caveat = kit.text(
        stage,
        "a number, not an answer: chapter 12's range has no top",
        24.0,
        Palette::red(),
        v(-3.4, -2.95),
    )?;
    play!(stage; fade_in(stage.arena_mut(), caveat, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;
    // 10_4 (split from 10_3, R3 Maya maya-13): the gauge
    kit.say(stage, "10_4")?;

    // the gauge: the average guess against the true size
    let mut parts = vec![
        Arc::new()
            .radius(GAUGE_R)
            .start_angle(0.0)
            .angle(PI)
            .arc_center(GAUGE_C)
            .style(line_style(Palette::dim(), 3.0))
            .build()?,
    ];
    for i in -3..=3 {
        let pct = 10.0 * f64::from(i);
        parts.push(
            Line::new(gauge_at(pct, GAUGE_R - 0.12), gauge_at(pct, GAUGE_R + 0.06))
                .style(line_style(Palette::dim(), if i == 0 { 3.5 } else { 2.0 }))
                .build()?,
        );
    }
    let dial = stage.arena_mut().add(v_group(parts));
    let mut tick_words = Vec::new();
    for i in [-3, 3] {
        let pct = 10.0 * f64::from(i);
        tick_words.push(kit.text(
            stage,
            &signed_pct(pct, 0),
            22.0,
            Palette::dim(),
            gauge_at(pct, GAUGE_R + 0.42),
        )?);
    }
    let truth = kit.text(
        stage,
        &thousands(lake),
        24.0,
        Role::dark(),
        gauge_at(0.0, GAUGE_R + 0.3),
    )?;
    let expected = (marked * caught) as f64 / lake as f64;
    let expect_word = kit.text(
        stage,
        &format!("expected caught twice here: {expected:.1}"),
        24.0,
        Palette::ink(),
        v(GAUGE_C[0], GAUGE_C[1] - 0.45),
    )?;
    let mut appear: Vec<Box<dyn Animation>> = Vec::new();
    for m in [dial, truth, expect_word].into_iter().chain(tick_words.iter().copied()) {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        appear.push(stage.prepare(a)?);
    }
    stage.play_prepared(appear)?;
    let needle = |stage: &mut Stage<'_>, pct: f64, color: Srgb| -> fmn::Result<Mob> {
        let r = Arrow::new(GAUGE_C, gauge_at(pct, GAUGE_R - 0.2))
            .buff(0.0)
            .thickness(5.0)
            .style(flat(color, 1.0))
            .build()?;
        Ok(stage.arena_mut().add(r))
    };
    // "Over the experiments that catch any, the soup guess runs about
    // twenty-four percent high, and Chapman's runs about six and a half
    // percent low."
    kit.until_sentence(stage, "10_4", 1)?;
    let plain_needle = needle(stage, law.plain_bias_pct, Palette::red())?;
    let chap_needle = needle(stage, law.chapman_bias_pct, Palette::teal())?;
    let hub = stage.arena_mut().add(
        Circle::new()
            .radius(0.09)
            .arc_center(GAUGE_C)
            .style(flat(Palette::ink(), 1.0))
            .build(),
    );
    // short tags at the needles; what each needle is, in a legend below
    let plain_tag = kit.text(
        stage,
        &signed_pct(law.plain_bias_pct, 0),
        26.0,
        Palette::red(),
        gauge_at(law.plain_bias_pct, GAUGE_R + 0.45),
    )?;
    let chap_tag = kit.text(
        stage,
        &signed_pct(law.chapman_bias_pct, 1),
        26.0,
        Palette::teal(),
        gauge_at(law.chapman_bias_pct, GAUGE_R + 0.45),
    )?;
    let lx = GAUGE_C[0] - 3.2;
    let plain_note = kit.text_left(
        stage,
        "plain soup guess, over experiments that caught any",
        22.0,
        Palette::red(),
        v(lx, GAUGE_C[1] - 0.95),
    )?;
    let chap_note = kit.text_left(
        stage,
        "Chapman's guess, over every experiment",
        22.0,
        Palette::teal(),
        v(lx, GAUGE_C[1] - 1.35),
    )?;
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), plain_needle)?.rt(0.8),
        fade_in(stage.arena_mut(), plain_tag, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), plain_note, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), hub, ORIGIN, 1.0)?.rt(0.8),
    );
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), chap_needle)?.rt(0.8),
        fade_in(stage.arena_mut(), chap_tag, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), chap_note, ORIGIN, 1.0)?.rt(0.8),
    );
    stage.scene_mut().bring_to_front(&[hub])?;
    // "Once you expect four or more, its average lands within two percent
    // of the truth, though any one guess still scatters widely."
    kit.until_sentence(stage, "10_4", 2)?;
    let bound = 100.0 * (-EXPECT_ENOUGH).exp();
    let band = stage.arena_mut().add(
        AnnularSector::new()
            .inner_radius(0.0)
            .outer_radius(GAUGE_R - 0.05)
            .start_angle(PI / 2.0 - (bound.ceil() / GAUGE_PCT) * (PI / 2.0))
            .angle(2.0 * (bound.ceil() / GAUGE_PCT) * (PI / 2.0))
            .arc_center(GAUGE_C)
            .color(Palette::green())
            .build()?
            .map_style_deep(|s| s.fill(Palette::green(), 0.25).stroke(Palette::green(), 0.0, 0.0)),
    );
    let band_word = kit.text(
        stage,
        &format!(
            "expect {} or more caught twice: the average within {}%",
            EXPECT_ENOUGH,
            bound.ceil()
        ),
        24.0,
        Palette::green(),
        v(GAUGE_C[0], GAUGE_C[1] - 1.15),
    )?;
    play!(stage;
        fade_in(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), band_word, ORIGIN, 1.0)?.rt(0.8),
        fade_out(stage.arena_mut(), at_zero, ORIGIN, 1.0)?.rt(0.5),
        fade_out(stage.arena_mut(), caveat, ORIGIN, 1.0)?.rt(0.5),
        // the legend has done its work; the band's words take its place,
        // clear of the lower-third below
        fade_out(stage.arena_mut(), plain_note, ORIGIN, 1.0)?.rt(0.4),
        fade_out(stage.arena_mut(), chap_note, ORIGIN, 1.0)?.rt(0.4),
    );
    stage.scene_mut().bring_to_back(&[band])?;
    let note = precisely(
        kit,
        stage,
        &format!(
            "Chapman's average = N − (N+1)·C(N−M, n+1)/C(N+1, n+1): exactly N when M + n ≥ N, never low by \
             more than N·e^(−Mn/N), under {}% once you expect {} caught twice. That bounds the bias; one \
             guess's typical error there is still about ±{}% (chapter 12's rule of thumb).",
            bound.ceil(),
            EXPECT_ENOUGH,
            (100.0 / EXPECT_ENOUGH.sqrt()).round()
        ),
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::py;

    #[test]
    fn what_chapter_10_says_holds_exactly() {
        let (lake, marked, caught) = SMALL;
        let law = estimate_law(lake, marked, caught);
        // the small study is the fact sheet's (A-F21.P_k0)
        assert_eq!(py(law.p0), "0.07197699188484312");
        // 10_2 "about one time in fourteen"; the screen's 7.2%
        assert_eq!(law.one_in.round(), 14.0);
        assert_eq!(format!("{:.1}", 100.0 * law.p0), "7.2");
        // 10_3 "two and a half", "about six and a half percent low", "+24%"
        assert_eq!((marked * caught) as f64 / lake as f64, 2.5);
        assert_eq!(signed_pct(law.chapman_bias_pct, 1), "\u{2212}6.5%");
        assert_eq!(signed_pct(law.plain_bias_pct, 0), "+24%");
        // "within two percent once you expect four or more" (A-F18)
        let bound = 100.0 * (-EXPECT_ENOUGH).exp();
        assert_eq!(py(bound), "1.8315638888734178");
        assert_eq!(bound.ceil(), 2.0);
        // Chapman at k = 0 (the description's 2,600)
        assert_eq!((marked + 1) * (caught + 1) - 1, 2_600);
        // the precisely note's one-guess error at 4 expected: 1 ÷ √4 = 50%
        assert_eq!((100.0 / EXPECT_ENOUGH.sqrt()).round(), 50.0);
        // 10_2's note: chapter 9's code divides by zero to inf
        assert_eq!(py((marked * caught) as f64 / 0.0), "inf");
    }

    #[test]
    fn the_gauge_turns_the_right_way() {
        let (_, up) = gauge_dir(0.0);
        assert!((up - 1.0).abs() < 1e-12);
        assert!(gauge_dir(24.0).0 > 0.0, "too high points right");
        assert!(gauge_dir(-6.5).0 < 0.0, "too low points left");
        assert!(gauge_dir(GAUGE_PCT).1.abs() < 1e-12);
    }
}
