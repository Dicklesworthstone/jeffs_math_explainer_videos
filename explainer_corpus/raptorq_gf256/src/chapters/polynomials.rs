//! Chapter 09, bytes are polynomials: the goal (09_1); the place values
//! become powers of x and 3 = 2 + 1 becomes x + 1 (09_2); one plus one is
//! zero, so x + x = 0 and adding is ex-or (09_3: 5 ⊕ 7 = 2 column by
//! column); multiply (x + 1)(x² + x + 1) (09_4); the six terms, two pairs
//! vanishing, x³ + 1 = 1001 = 9 (09_5); and the chalk 3 × 7 = 9 (09_6).
//! Every exponent on screen comes from the bits of 3 and 7.

use std::ops::Range;

use explainer_kit::props::predict_pause;
use explainer_kit::{Kit, Palette, Timed, children, fade_all, hex, play, stroke_style, v};
use fmn::animation::{indicate, replacement_transform};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::draw::{Role, card_fill, group_of};
use super::register::{Register, regroup, tile};
use crate::model::poly::{exponents, foil, poly, power, survivors};

/// 09's worked example: 3 × 7, and 02's 5 ⊕ 7.
const THREE: u8 = 3;
const SEVEN: u8 = 7;
const FIVE: u8 = 5;
const BYTE_MEMBERS: u32 = 256;
const LISTEN_S: f64 = 3.0;

/// The byte range of the `nth` occurrence of `needle` in `src`.
fn nth(src: &str, needle: &str, n: usize) -> Range<usize> {
    let start = src
        .match_indices(needle)
        .nth(n)
        .map(|(i, _)| i)
        .expect("the needle occurs");
    start..start + needle.len()
}

/// The glyphs of a Tex whose source spans lie inside `range` (span entry i
/// is child i).
fn glyphs(stage: &Stage<'_>, tex: Mob, spans: &SpanMapData, range: &Range<usize>) -> Vec<Mob> {
    let kids = children(stage, tex);
    debug_assert_eq!(kids.len(), spans.entries.len(), "one span per glyph");
    spans
        .entries
        .iter()
        .zip(kids)
        .filter(|(e, _)| e.start >= range.start && e.end <= range.end)
        .map(|(_, m)| m)
        .collect()
}

/// Copies of `glyphs` (rooted), as one new group.
fn copies(stage: &mut Stage<'_>, glyphs: &[Mob]) -> fmn::Result<Mob> {
    let mut out = Vec::new();
    for &g in glyphs {
        out.push(stage.copy_family(g)?);
    }
    group_of(stage, &out)
}

/// The play of several prepared animations at once.
fn together(stage: &mut Stage<'_>, anims: Vec<Box<dyn Animation>>, run_time: f64) -> fmn::Result<()> {
    stage.play_prepared_with(
        anims,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}

/// A rounded card with a line of text: 09_4's "pause and try it".
fn note_card(kit: &Kit, stage: &mut Stage<'_>, text: &str, at: Vec3) -> fmn::Result<Mob> {
    let t = kit.text(stage, text, 30.0, Palette::ink(), at)?;
    let w = stage.get_width(t) + 0.6;
    let back = Rectangle::new()
        .width(w)
        .height(0.75)
        .corner_radius(0.12)
        .style(Style::default().fill(card_fill(), 0.96).stroke(Role::card(), 3.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    group_of(stage, &[back, t])
}

const ROW_CELL: f64 = 0.72;

pub struct Polynomials {
    pub kit: Kit,
}

impl SceneConstruct for Polynomials {
    fn name(&self) -> &str {
        "09_bytes_are_polynomials"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let terms = foil(THREE, SEVEN);
        let nine = survivors(&terms);
        assert_eq!(
            nine,
            crate::model::mul(THREE, SEVEN, crate::model::FOLD),
            "no fold: 3 × 7 is the same in the field"
        );

        // 09_1: the goal; whole numbers can't, polynomials can
        let goal = kit.text(
            stage,
            &format!("a field with {BYTE_MEMBERS} members"),
            46.0,
            Role::byte(),
            v(0.0, 2.2),
        )?;
        let whole = kit.text(stage, "whole numbers", 40.0, Palette::dim(), v(-3.0, 0.4))?;
        let strike = Line::new(v(-4.5, 0.42), v(-1.5, 0.42))
            .style(stroke_style(Role::lost(), 5.0))
            .build()?;
        let strike = stage.arena_mut().add(strike);
        let polys = kit.text(stage, "polynomials", 46.0, Role::byte(), v(3.0, 0.4))?;
        kit.say(stage, "09_1")?;
        play!(stage; write(stage.arena(), goal).rt(0.9));
        play!(stage; fade_in(stage.arena_mut(), whole, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; show_creation(strike).rt(0.5));
        play!(stage; write(stage.arena(), polys).rt(0.8));
        kit.hold(stage)?;

        // 09_2: place values become powers of x; 3 = 2 + 1 = x + 1
        fade_all(stage, 0.6)?;
        let row_c = v(-0.6, -0.6);
        let reg = Register::build(&kit, stage, THREE, ROW_CELL, row_c, Role::byte(), false)?;
        let mut places = Vec::new();
        for k in 0..8u32 {
            let at = v(reg.slot(k as usize)[0], row_c[1] + 0.85 * ROW_CELL);
            places.push(kit.tex(stage, &(1u32 << (7 - k)).to_string(), 36.0, Palette::dim(), &[], at)?);
        }
        let three = kit.tex(stage, &format!("= {THREE}"), 56.0, Role::byte(), &[], v(3.2, -0.6))?;
        kit.say(stage, "09_2")?;
        reg.show(stage, 0.6)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &p in &places {
            let a = fade_in(stage.arena_mut(), p, v(0.0, -0.1), 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        anims.push({
            let a = fade_in(stage.arena_mut(), three, ORIGIN, 1.0)?;
            stage.prepare(a)?
        });
        together(stage, anims, 0.6)?;
        // "one, two, four, eight": right to left
        for k in [7usize, 6, 5, 4] {
            play!(stage; indicate(stage.arena_mut(), places[k], 1.35, None)?.rt(0.45));
        }
        // swap the two for an x; then x², x³, and the rest
        let powers: Vec<Mob> = (0..8u32)
            .map(|k| {
                let at = stage.get_center(places[k as usize]);
                kit.tex(stage, &power(7 - k), 40.0, Palette::ink(), &[], at)
            })
            .collect::<fmn::Result<_>>()?;
        stage.wait(0.3)?;
        play!(stage; replacement_transform(places[6], powers[6]).rt(0.9));
        stage.wait(0.6)?;
        for k in [5usize, 4] {
            play!(stage; replacement_transform(places[k], powers[k]).rt(0.6));
        }
        play!(stage; replacement_transform(places[7], powers[7]).rt(0.4));
        let mut rest: Vec<Box<dyn Animation>> = Vec::new();
        for k in 0..4 {
            rest.push(stage.prepare(replacement_transform(places[k], powers[k]))?);
        }
        let rest = explainer_kit::lagged(stage, rest, 0.15)?;
        play!(stage; rest.rt(1.0));
        let (two_one, two_one_spans) = kit.tex_spans(
            stage,
            &format!("{THREE} = 2 + 1"),
            60.0,
            Palette::ink(),
            &[],
            v(-0.6, 1.0),
        )?;
        let x_one_src = format!("{THREE} = {}", poly(THREE));
        let (x_one, x_one_spans) = kit.tex_spans(stage, &x_one_src, 60.0, Palette::ink(), &[], v(-0.6, 1.0))?;
        play!(stage; write(stage.arena(), two_one).rt(0.8));
        stage.wait(0.5)?;
        let morph = TransformMatchingTex::new(stage.arena_mut(), two_one, x_one, &two_one_spans, &x_one_spans)?;
        play!(stage; morph.rt(1.2));
        kit.hold(stage)?;

        // 09_3: one plus one is zero; x + x = 0; adding is ex-or (5 ⊕ 7 = 2)
        let low = v(-0.6, -2.55);
        let mut row_members: Vec<Mob> = vec![reg.frame];
        row_members.extend(&reg.tiles);
        row_members.extend(&powers);
        // one group to move the row; a later animation of a single tile
        // dissolves it back into its members (fmn, like manim, re-adds an
        // animated mobject, which splits a rooted parent)
        let row = regroup(stage, &row_members)?;
        let one_one = kit.tex(stage, "1 + 1 = 0", 60.0, Role::ex_or(), &[], v(-4.4, 2.6))?;
        let x_x = kit.tex(stage, "x + x = 0", 60.0, Role::ex_or(), &[], v(-4.4, 1.55))?;
        kit.say(stage, "09_3")?;
        play!(stage;
            row.animate().scale(0.75)?.move_to(low, ORIGIN)?,
            fade_out(stage.arena_mut(), three, ORIGIN, 1.0)?.rt(0.6),
            x_one.animate().scale(0.75)?.move_to(v(5.0, -2.55), ORIGIN)?,
        );
        // the numbers in front are the bits: zero or one
        let mut flash: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &reg.tiles {
            let a = indicate(stage.arena_mut(), t, 1.2, None)?;
            flash.push(stage.prepare(a)?);
        }
        let flash = explainer_kit::lagged(stage, flash, 0.1)?;
        play!(stage; flash.rt(1.2));
        play!(stage; write(stage.arena(), one_one).rt(0.7));
        stage.wait(0.6)?;
        play!(stage; write(stage.arena(), x_x).rt(0.7));
        // 5 ⊕ 7 column by column: x² and 1 cancel, x is left
        let two = FIVE ^ SEVEN;
        let cols = [2u32, 1, 0];
        let col_x = |e: u32| 1.2 + 1.35 * f64::from(2 - e);
        let rows_y = [2.6, 1.75, 0.55];
        let mut cells: Vec<Vec<Mob>> = Vec::new();
        for (r, byte) in [FIVE, SEVEN, two].into_iter().enumerate() {
            let mut line = Vec::new();
            let es = exponents(byte);
            for (i, &e) in es.iter().enumerate() {
                if i > 0 {
                    line.push(kit.tex(stage, "+", 52.0, Palette::dim(), &[], v(col_x(e) - 0.68, rows_y[r]))?);
                }
                line.push(kit.tex(stage, &power(e), 52.0, Palette::ink(), &[], v(col_x(e), rows_y[r]))?);
            }
            let label = if r == 0 {
                format!("{byte}")
            } else if r == 1 {
                format!(r"\oplus\; {byte}")
            } else {
                format!("= {byte}")
            };
            line.push(kit.tex(stage, &label, 52.0, Role::ex_or(), &[], v(5.6, rows_y[r]))?);
            cells.push(line);
        }
        let bar = Line::new(v(col_x(2) - 0.6, 1.15), v(col_x(0) + 0.6, 1.15))
            .style(stroke_style(Palette::dim(), 2.5))
            .build()?;
        let bar = stage.arena_mut().add(bar);
        // what 09_3 puts up, so 09_4 can take exactly that down
        let mut sec3 = vec![one_one, x_x, bar];
        for line in &cells[..2] {
            let g = group_of(stage, line)?;
            play!(stage; fade_in(stage.arena_mut(), g, v(0.0, -0.15), 1.0)?.rt(0.5));
            sec3.push(g);
        }
        play!(stage; show_creation(bar).rt(0.4));
        // the matching columns light, then the answer row
        let mut marks: Vec<Box<dyn Animation>> = Vec::new();
        for &e in &cols {
            let both = exponents(FIVE).contains(&e) && exponents(SEVEN).contains(&e);
            if both {
                let ring = Rectangle::new()
                    .width(0.95)
                    .height(1.55)
                    .corner_radius(0.15)
                    .style(stroke_style(Role::ex_or(), 3.0))
                    .build()?
                    .moved_to(v(col_x(e), 2.17));
                let ring = stage.arena_mut().add(ring);
                marks.push(stage.prepare(show_creation(ring))?);
                sec3.push(ring);
            }
        }
        together(stage, marks, 0.6)?;
        let answer = group_of(stage, &cells[2])?;
        play!(stage; fade_in(stage.arena_mut(), answer, v(0.0, -0.15), 1.0)?.rt(0.6));
        sec3.push(answer);
        kit.hold(stage)?;

        // 09_4: multiply (x + 1)(x² + x + 1); pause and try it
        let mut out: Vec<Box<dyn Animation>> = Vec::new();
        for m in sec3 {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            out.push(stage.prepare(a)?);
        }
        let seven_poly = kit.tex(
            stage,
            &format!("{SEVEN} = {}", poly(SEVEN)),
            45.0,
            Palette::ink(),
            &[],
            v(5.0, -1.75),
        )?;
        let p_src = format!("({})({})", poly(THREE), poly(SEVEN));
        let (product, p_spans) = kit.tex_spans(stage, &p_src, 80.0, Palette::ink(), &[], v(-0.6, 2.45))?;
        let try_it = note_card(&kit, stage, "pause and try it", v(4.6, 1.25))?;
        kit.say(stage, "09_4")?;
        together(stage, out, 0.5)?;
        play!(stage; fade_in(stage.arena_mut(), seven_poly, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; write(stage.arena(), product).rt(1.2));
        play!(stage; fade_in(stage.arena_mut(), try_it, v(0.0, -0.15), 1.0)?.rt(0.5));
        let q = predict_pause(&kit, stage, v(-0.6, 1.2))?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;

        // 09_5: FOIL one term at a time; the matching pairs vanish
        let pieces: Vec<String> = terms
            .iter()
            .enumerate()
            .map(|(i, &e)| if i == 0 { power(e) } else { format!(" + {}", power(e)) })
            .collect();
        let e_src: String = pieces.concat();
        let mut ranges = Vec::new();
        let mut at = 0;
        for p in &pieces {
            ranges.push(at..at + p.len());
            at += p.len();
        }
        let (expanded, e_spans) = kit.tex_spans(stage, &e_src, 72.0, Palette::ink(), &[], v(-0.6, 1.0))?;
        stage.add_to_scene(expanded)?;
        for g in children(stage, expanded) {
            stage.set_opacity(g, 0.0, true);
        }
        // the factors' terms in the product line, FOIL order
        let ea = exponents(THREE);
        let eb = exponents(SEVEN);
        let poly3 = poly(THREE);
        let first = |i: usize| -> Range<usize> {
            let r = nth(&poly3, &power(ea[i]), 0);
            1 + r.start..1 + r.end
        };
        let second_at = 1 + poly3.len() + 2;
        let poly7 = poly(SEVEN);
        let mut offset = 0;
        let mut second_ranges = Vec::new();
        for &e in &eb {
            let t = power(e);
            let r = nth(&poly7[offset..], &t, 0);
            second_ranges.push(second_at + offset + r.start..second_at + offset + r.end);
            offset += r.end;
        }
        kit.say(stage, "09_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_out(stage.arena_mut(), try_it, ORIGIN, 1.0)?.rt(0.3),
        );
        let mut k = 0;
        for i in 0..ea.len() {
            for sr in &second_ranges {
                let fa = glyphs(stage, product, &p_spans, &first(i));
                let fb = glyphs(stage, product, &p_spans, sr);
                let target = glyphs(stage, expanded, &e_spans, &ranges[k]);
                let mut both = fa.clone();
                both.extend(&fb);
                let flying = copies(stage, &both)?;
                let dest = {
                    let term_r = nth(&e_src[ranges[k].clone()], &power(terms[k]), 0);
                    let r = ranges[k].start + term_r.start..ranges[k].start + term_r.end;
                    let gs = glyphs(stage, expanded, &e_spans, &r);
                    let xs: Vec<f64> = gs.iter().map(|&g| stage.get_center(g)[0]).collect();
                    let ys: Vec<f64> = gs.iter().map(|&g| stage.get_center(g)[1]).collect();
                    v(
                        xs.iter().sum::<f64>() / xs.len() as f64,
                        ys.iter().sum::<f64>() / ys.len() as f64,
                    )
                };
                let mut anims: Vec<Box<dyn Animation>> = Vec::new();
                for &g in &both {
                    let a = indicate(stage.arena_mut(), g, 1.25, Some([0.35, 0.77, 0.87]))?;
                    anims.push(stage.prepare(a)?);
                }
                let from = stage.get_center(flying);
                let fly = fade_out(
                    stage.arena_mut(),
                    flying,
                    [dest[0] - from[0], dest[1] - from[1], 0.0],
                    0.6,
                )?;
                anims.push(stage.prepare(fly)?);
                for &g in &target {
                    let a = g.animate().set_opacity(1.0)?;
                    anims.push(stage.prepare(a)?);
                }
                together(stage, anims, if k == 0 { 0.9 } else { 0.55 })?;
                k += 1;
            }
        }
        // two pairs match: x² + x² and x + x, each lit, then gone
        let mut pairs: Vec<(u32, Vec<usize>)> = Vec::new();
        for (i, &e) in terms.iter().enumerate() {
            match pairs.iter_mut().find(|(p, _)| *p == e) {
                Some((_, idx)) => idx.push(i),
                None => pairs.push((e, vec![i])),
            }
        }
        pairs.retain(|(_, idx)| idx.len() == 2);
        let hues = [Role::ex_or(), Palette::teal()];
        for ((_, idx), &hue) in pairs.iter().zip(&hues) {
            let mut lit: Vec<Box<dyn Animation>> = Vec::new();
            let mut gone = Vec::new();
            for &i in idx {
                let gs = glyphs(stage, expanded, &e_spans, &ranges[i]);
                for &g in &gs {
                    let a = g.animate().set_color(hue)?;
                    lit.push(stage.prepare(a)?);
                }
                gone.extend(gs);
            }
            together(stage, lit, 0.5)?;
            stage.wait(0.35)?;
            let mut fade: Vec<Box<dyn Animation>> = Vec::new();
            for &g in &gone {
                let a = g.animate().set_opacity(0.0)?.scale(0.4)?;
                fade.push(stage.prepare(a)?);
            }
            together(stage, fade, 0.6)?;
        }
        // what's left, x³ + 1, closes up; it is 1001: nine
        let left_src = poly(nine);
        let (left, left_spans) = kit.tex_spans(stage, &left_src, 72.0, Palette::ink(), &[], v(-1.1, 1.0))?;
        let close = TransformMatchingTex::new(stage.arena_mut(), expanded, left, &e_spans, &left_spans)?;
        play!(stage; close.rt(1.0));
        let new_tiles: Vec<Mob> = (0..8)
            .map(|k| {
                let at = stage.get_center(reg.tiles[k]);
                tile(&kit, stage, nine >> (7 - k) & 1 == 1, ROW_CELL * 0.75, Role::byte(), at)
            })
            .collect::<fmn::Result<_>>()?;
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for (&old, &new) in reg.tiles.iter().zip(&new_tiles) {
            swap.push(stage.prepare(replacement_transform(old, new))?);
        }
        let bits = kit.tex(stage, &format!("= {nine:04b}"), 60.0, Palette::dim(), &[], v(2.05, 1.0))?;
        let is_nine = kit.tex(stage, &format!("= {nine}"), 72.0, Role::byte(), &[], v(4.35, 1.0))?;
        together(stage, swap, 0.8)?;
        play!(stage; fade_in(stage.arena_mut(), bits, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; write(stage.arena(), is_nine).rt(0.6));
        kit.hold(stage)?;

        // 09_6: three times seven is nine, in chalk; the chords of 3, 7, 9
        fade_all(stage, 0.6)?;
        let chalk_src = format!(r"{THREE} \times {SEVEN} = {nine}");
        let (chalk, chalk_spans) = kit.tex_spans(
            stage,
            &chalk_src,
            150.0,
            hex("#F4F0E6"),
            &[(&nine.to_string(), Role::byte())],
            v(0.0, -0.2),
        )?;
        kit.say(stage, "09_6")?;
        play!(stage; write(stage.arena(), chalk).rt(1.4));
        let wav = super::sound::wav(
            "09_6_chords",
            &super::sound::chords(&[(THREE, 0.0, 0.8), (SEVEN, 1.0, 0.8), (nine, 2.0, 0.9)], LISTEN_S),
        )?;
        kit.sonify(stage, LISTEN_S, "09_6", &wav)?;
        // each number lights with its chord: 3, then 7, then the product
        let at_3 = nth(&chalk_src, &THREE.to_string(), 0);
        let at_7 = nth(&chalk_src, &SEVEN.to_string(), 0);
        let eq = nth(&chalk_src, "=", 0).end;
        let at_9 = eq..chalk_src.len();
        for r in [at_3, at_7, at_9] {
            let gs = glyphs(stage, chalk, &chalk_spans, &r);
            let mut lit: Vec<Box<dyn Animation>> = Vec::new();
            for &g in &gs {
                let a = indicate(stage.arena_mut(), g, 1.2, Some([1.0, 0.85, 0.4]))?;
                lit.push(stage.prepare(a)?);
            }
            together(stage, lit, 0.9)?;
            stage.wait(0.1)?;
        }
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}
