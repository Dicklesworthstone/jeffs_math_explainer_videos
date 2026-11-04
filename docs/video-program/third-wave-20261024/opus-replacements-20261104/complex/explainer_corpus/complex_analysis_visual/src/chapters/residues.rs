//! 12_residues: why only the 1/(z − p) term survives a loop, the residue
//! theorem as a big loop shrunk onto small loops, residues of f′/f as the
//! turns of chapter 11, and the bump's residue at i (storyboard 12_1–12_4).
//! Each term near a pole gets its own tip-to-tail loop sum, as in chapter
//! 08: the negative square and the holomorphic part close up, and the
//! 1/(z − p) term stacks into the 2πi column.

use std::f64::consts::{FRAC_PI_4, TAU};

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    Painter, Plane, chain_arrows, chain_points, dot, fact, fade_in_all, fade_out_all, faded_line, fit_chain, hero,
    lay_down, marks, names_now, pause, pole, pole_mark, say, wait_until,
};
use crate::model::{self, C, Func, I, ZERO};

pub const LAURENT_TEX: &str = r"f(z) = \frac{a_{-2}}{(z-p)^2} + \frac{a_{-1}}{z-p} + h(z)";
pub const H_NOTE: &str = "h holomorphic near p";
pub const TERMS_TEX: [&str; 3] = [r"\frac{1}{(z-p)^2}", r"\frac{1}{z-p}", r"h(z) = 1 + \frac{z-p}{2}"];
pub const TOTALS: [&str; 3] = ["closes: 0", "2πi", "closes: 0"];
pub const RESIDUE_TEX: &str = r"\mathrm{residue} = a_{-1}";
pub const THEOREM_TEX: &str = r"\oint f(z)\,dz = 2\pi i \sum_k \mathrm{Res}_{p_k} f";
pub const INSIDE: &str = "sum over the poles inside the loop";
pub const SHRINK: [&str; 2] = ["the big loop = the small loops", "(Cauchy's theorem between them)"];
pub const EACH: &str = "each small loop: 2πi × its residue";
pub const NEAR_TEX: &str = r"\frac{g(z)}{h(z)} \approx \frac{g(p)}{h'(p)\,(z-p)}";
pub const SIMPLE_TEX: &str = r"h(p) = 0,\quad h'(p) \ne 0:\qquad \mathrm{Res}_p \frac{g}{h} = \frac{g(p)}{h'(p)}";
pub const ZERO_TEX: &str = r"\frac{f'}{f} \approx \frac{m}{z - z_0}";
pub const POLE_TEX: &str = r"\frac{f'}{f} \approx \frac{-n}{z - p}";
pub const ZERO_SAYS: &str = "near a zero of order m";
pub const POLE_SAYS: &str = "near a pole of order n";
pub const LOCAL_TEX: [&str; 3] = [r"f = z", r"f = z^2", r"f = 1/z"];
pub const LOG_TEX: [&str; 3] = [
    r"\frac{f'}{f} = \frac{1}{z}",
    r"\frac{f'}{f} = \frac{2}{z}",
    r"\frac{f'}{f} = \frac{-1}{z}",
];
pub const RESIDUE: &str = "residue ";
pub const TURNS: [&str; 3] = [
    "the turns of chapter 11:",
    "zeros − poles =",
    "the sum of the residues of f'/f",
];
pub const BUMP_STEPS_TEX: [&str; 2] = [r"g = 1,\quad h = 1 + z^2,\quad h'(z) = 2z", r"h'(i) = 2i"];

/// The plane of 12_1–12_2.
pub fn plane() -> Plane {
    Plane::new(v(-3.6, -0.6), 4.6, (-1.6, 1.6), (-1.6, 1.6))
}

/// Radius of 12_1's small loop about p = 0, and its steps.
pub const LOOP_R: f64 = 0.9;
pub const STEPS: usize = 24;

/// The three terms near p = 0: 1/z², 1/z, and a holomorphic part.
pub fn term(k: usize, z: C) -> Option<C> {
    match k {
        0 => Some((z * z).recip()),
        1 => Some(z.recip()),
        _ => Some(C::real(1.0) + z.scale(0.5)),
    }
}

/// The chain of term `k` around the small loop.
pub fn term_chain(k: usize) -> Vec<C> {
    chain_points(&model::loop_products(|z| term(k, z), ZERO, LOOP_R, STEPS).unwrap_or_default())
}

/// Poles of 12_2 and their small loops.
pub const POLES: [C; 3] = [C::new(-0.7, 0.5), C::new(0.6, 0.6), C::new(0.0, -0.7)];
pub const BIG_R: f64 = 1.35;
pub const SMALL_R: f64 = 0.25;

/// The three small planes of 12_3.
pub fn small_planes() -> [Plane; 3] {
    [-4.9, -1.7, 1.5].map(|x| Plane::new(v(x, -1.0), 2.8, (-1.2, 1.2), (-1.2, 1.2)))
}

/// The bump's picture in 12_4.
pub fn bump_plane() -> Plane {
    Plane::new(v(-3.4, -0.3), 5.0, (-2.0, 2.0), (-2.0, 2.0))
}

pub struct Residues {
    pub kit: Kit,
}

impl SceneConstruct for Residues {
    fn name(&self) -> &str {
        "12_residues"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = plane();

        // 12_1: near a pole, only the 1/(z − p) term survives a small loop
        let line = say(&kit, stage, "12_1")?;
        let mut frame = vec![p.border(stage, Palette::faint())?];
        frame.extend(p.axes(stage)?);
        let pm = pole_mark(stage, p.p(ZERO));
        let p_name = kit.tex(stage, "p", 28.0, pole(), &[], p.p(C::new(0.2, -0.22)))?;
        fade_in_all(stage, &frame, 0.4)?;
        fade_in_all(stage, &pm, 0.3)?;
        fade_in_all(stage, &[p_name], 0.3)?;
        line.wait_sentence(stage, 1)?;
        let laurent = kit.tex(
            stage,
            LAURENT_TEX,
            34.0,
            Palette::ink(),
            &[("a_{-1}", hero())],
            v(0.6, 3.0),
        )?;
        let h_note = kit.text(stage, H_NOTE, 20.0, Palette::dim(), v(0.6, 2.2))?;
        fade_in_all(stage, &[laurent, h_note], 0.5)?;
        line.wait_sentence(stage, 2)?;
        let small = p.circle(stage, ZERO, LOOP_R, hero(), 5.0)?;
        let arrow = stage.arena_mut().add(
            Arrow::new(
                p.p(C::polar(LOOP_R, FRAC_PI_4)),
                p.p(C::polar(LOOP_R, FRAC_PI_4 + 0.35)),
            )
            .thickness(5.0)
            .color(hero())
            .build()?,
        );
        fade_in_all(stage, &[small, arrow], 0.4)?;
        let mut boxes = Vec::new();
        for (k, x) in [0.6, 3.1, 5.6].into_iter().enumerate() {
            let label = kit.tex(stage, TERMS_TEX[k], 28.0, Palette::ink(), &[], v(x, 1.15))?;
            let pts = term_chain(k);
            let (origin, scale) = fit_chain(&pts, v(x, -0.6), 2.0, 2.4, 1.6);
            let color = if k == 1 { hero() } else { Palette::ink() };
            let arrows = chain_arrows(stage, origin, scale, &pts, color)?;
            fade_in_all(stage, &[label], 0.3)?;
            lay_down(stage, &arrows, 0.04)?;
            let total = kit.text(stage, TOTALS[k], 24.0, color, v(x, -2.25))?;
            fade_in_all(stage, &[total], 0.3)?;
            boxes.push(label);
            boxes.extend(arrows);
            boxes.push(total);
        }
        line.wait_sentence(stage, 3)?;
        let residue = kit.tex(stage, RESIDUE_TEX, 30.0, hero(), &[], v(3.1, -2.75))?;
        fade_in_all(stage, &[residue], 0.4)?;
        let search = names_now(&kit, stage, &["Laurent series", "residue"])?;
        wait_until(stage, line.end())?;

        // 12_2: the residue theorem: a big loop is the small loops around its poles
        kit.hold(stage)?;
        let line = say(&kit, stage, "12_2")?;
        let mut gone = vec![laurent, h_note, small, arrow, residue, search, p_name];
        gone.extend(boxes);
        gone.extend(pm);
        fade_out_all(stage, &gone, 0.4)?;
        let theorem = kit.tex(stage, THEOREM_TEX, 34.0, Palette::ink(), &[], v(0.6, 3.0))?;
        let inside = kit.text(stage, INSIDE, 20.0, Palette::dim(), v(0.6, 2.25))?;
        let mut pms = Vec::new();
        for &q in &POLES {
            pms.extend(pole_mark(stage, p.p(q)));
        }
        let big = p.circle(stage, ZERO, BIG_R, hero(), 5.0)?;
        fade_in_all(stage, &pms, 0.3)?;
        fade_in_all(stage, &[big], 0.4)?;
        fade_in_all(stage, &[theorem, inside], 0.5)?;
        // shrink the big loop onto small loops joined by corridors walked both ways
        let mut smalls = Vec::new();
        for &q in &POLES {
            smalls.push(p.circle(stage, q, SMALL_R, hero(), 4.0)?);
            let out_dir = if q.abs() > 0.0 {
                q.scale(1.0 / q.abs())
            } else {
                C::real(1.0)
            };
            let (a, b) = (q + out_dir.scale(SMALL_R), out_dir.scale(BIG_R));
            for side in [-0.04, 0.04] {
                let n = C::new(-out_dir.im, out_dir.re).scale(side);
                smalls.push(
                    stage.arena_mut().add(
                        polyline(vec![p.p(a + n), p.p(b + n)])
                            .style(faded_line(hero(), 2.5, 0.7))
                            .build(),
                    ),
                );
            }
        }
        let shrink = [
            kit.text(stage, SHRINK[0], 22.0, Palette::ink(), v(3.6, 1.2))?,
            kit.text(stage, SHRINK[1], 20.0, Palette::dim(), v(3.6, 0.75))?,
        ];
        fade_in_all(stage, &smalls, 0.6)?;
        fade_in_all(stage, &shrink, 0.4)?;
        pause(stage, 0.4)?;
        super::draw::restroke(stage, big, hero(), 0.25);
        let each = kit.text(stage, EACH, 22.0, hero(), v(3.6, 0.1))?;
        fade_in_all(stage, &[each], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let search = names_now(&kit, stage, &["residue theorem"])?;
        line.wait_sentence(stage, 2)?;
        let near = kit.tex(stage, NEAR_TEX, 30.0, Palette::ink(), &[], v(3.6, -1.0))?;
        let simple = kit.tex(stage, SIMPLE_TEX, 26.0, hero(), &[], v(2.7, -2.1))?;
        fade_in_all(stage, &[near], 0.4)?;
        fade_in_all(stage, &[simple], 0.4)?;
        wait_until(stage, line.end())?;

        // 12_3: the turns of chapter 11 are residues of f′/f
        kit.hold(stage)?;
        let line = say(&kit, stage, "12_3")?;
        let mut gone = vec![theorem, inside, big, each, search, near, simple, shrink[0], shrink[1]];
        gone.extend(pms);
        gone.extend(smalls);
        gone.extend(frame);
        fade_out_all(stage, &gone, 0.4)?;
        line.wait_sentence(stage, 1)?;
        let zero_tex = kit.tex(stage, ZERO_TEX, 32.0, Palette::ink(), &[], v(-3.3, 2.9))?;
        let zero_says = kit.text(stage, ZERO_SAYS, 20.0, Palette::dim(), v(-3.3, 2.1))?;
        fade_in_all(stage, &[zero_tex, zero_says], 0.4)?;
        let local: Vec<&str> = fact("argument.local")?.split(',').collect();
        let funcs = [Func::Identity, Func::Square, Func::Recip];
        let mut painters = Vec::new();
        let mut minis = Vec::new();
        for (k, sp) in small_planes().into_iter().enumerate() {
            if k == 2 {
                line.wait_sentence(stage, 2)?;
                let pole_tex = kit.tex(stage, POLE_TEX, 32.0, Palette::ink(), &[], v(2.4, 2.9))?;
                let pole_says = kit.text(stage, POLE_SAYS, 20.0, Palette::dim(), v(2.4, 2.1))?;
                fade_in_all(stage, &[pole_tex, pole_says], 0.4)?;
                minis.extend([pole_tex, pole_says]);
            }
            let painter = Painter::build(stage, Some(funcs[k]), sp, 1.0)?;
            painter.show(stage, 0.6)?;
            let mut extra = vec![
                sp.border(stage, Palette::faint())?,
                kit.tex(
                    stage,
                    LOCAL_TEX[k],
                    26.0,
                    Palette::ink(),
                    &[],
                    v(sp.frame.center[0], sp.frame.top() + 0.35),
                )?,
                kit.tex(
                    stage,
                    LOG_TEX[k],
                    26.0,
                    Palette::ink(),
                    &[],
                    v(sp.frame.center[0], sp.frame.bottom() - 0.55),
                )?,
                kit.text(
                    stage,
                    &format!("{RESIDUE}{}", local[k].replace('-', "−")),
                    22.0,
                    hero(),
                    v(sp.frame.center[0], sp.frame.bottom() - 1.25),
                )?,
            ];
            extra.extend(marks(stage, funcs[k], &sp));
            fade_in_all(stage, &extra, 0.4)?;
            minis.extend(extra);
            painters.push(painter);
        }
        let turns: Vec<Mob> = TURNS
            .iter()
            .enumerate()
            .map(|(i, t)| kit.text(stage, t, 20.0, Palette::ink(), v(5.2, 0.4 - 0.45 * i as f64)))
            .collect::<fmn::Result<_>>()?;
        fade_in_all(stage, &turns, 0.5)?;
        wait_until(stage, line.end())?;

        // 12_4: the bump's residue at i: g/h′ = 1/(2i)
        kit.hold(stage)?;
        let line = say(&kit, stage, "12_4")?;
        let mut gone = vec![zero_tex, zero_says];
        gone.extend(minis);
        gone.extend(turns);
        fade_out_all(stage, &gone, 0.4)?;
        for painter in &painters {
            painter.leave(stage, 0.4)?;
        }
        let bp = bump_plane();
        let bump = Painter::build(stage, Some(Func::Bump), bp, 1.0)?;
        let border = bp.border(stage, Palette::faint())?;
        bump.show(stage, 0.8)?;
        let bump_marks = marks(stage, Func::Bump, &bp);
        fade_in_all(stage, &[border], 0.3)?;
        fade_in_all(stage, &bump_marks, 0.3)?;
        let steps0 = kit.tex(stage, BUMP_STEPS_TEX[0], 30.0, Palette::ink(), &[], v(3.6, 2.3))?;
        fade_in_all(stage, &[steps0], 0.4)?;
        line.wait_sentence(stage, 1)?;
        let around = bp.circle(stage, I, 0.35, hero(), 5.0)?;
        let at_i = dot(stage, bp.p(I), 0.05, Palette::ink());
        let steps1 = kit.tex(stage, BUMP_STEPS_TEX[1], 30.0, Palette::ink(), &[], v(3.6, 1.3))?;
        fade_in_all(stage, &[around, at_i, steps1], 0.4)?;
        line.wait_sentence(stage, 2)?;
        let res: Vec<&str> = fact("residue.bump.at_i")?.split(',').collect();
        let answer = kit.tex(
            stage,
            &format!(
                r"\mathrm{{Res}}_{{z=i}}\,\frac{{1}}{{1+z^2}} = \frac{{1}}{{2i}} = {}\,i",
                res[1]
            ),
            32.0,
            hero(),
            &[],
            v(3.6, 0.0),
        )?;
        fade_in_all(stage, &[answer], 0.5)?;
        wait_until(stage, line.end())?;
        kit.music_breath(stage, 2.0)?;
        let mut all = vec![border, steps0, steps1, around, at_i, answer];
        all.extend(bump_marks);
        fade_out_all(stage, &all, 0.5)?;
        bump.leave(stage, 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_one_over_z_term_survives_the_loop() {
        for k in 0..3 {
            let total = *term_chain(k).last().unwrap();
            if k == 1 {
                // 24 steps already give the column 6.27i, near 2πi
                assert!((total - C::new(0.0, 6.2653)).abs() < 1e-3, "{total:?}");
                assert!((total.im - TAU).abs() < 0.02);
            } else {
                assert!(total.abs() < 1e-9, "term {k}: {total:?}");
            }
        }
        // the chains fit their boxes
        for k in 0..3 {
            let pts = term_chain(k);
            let (o, s) = fit_chain(&pts, v(0.0, 0.0), 2.0, 2.4, 1.6);
            for z in pts {
                let q = super::super::draw::chain_at(o, s, z);
                assert!(q[0].abs() <= 1.0 + 1e-9 && q[1].abs() <= 1.2 + 1e-9);
            }
        }
    }

    #[test]
    fn the_big_loop_holds_every_pole_and_the_small_loops_do_not_touch() {
        for (i, &q) in POLES.iter().enumerate() {
            assert!(q.abs() + SMALL_R < BIG_R);
            for &r in &POLES[i + 1..] {
                assert!((q - r).abs() > 2.0 * SMALL_R);
            }
            assert!(plane().contains(q));
        }
        assert!(BIG_R < 1.6);
    }

    #[test]
    fn the_residues_shown_are_the_facts() {
        let f = crate::facts::facts().unwrap();
        assert_eq!(f.get("argument.local"), Some("1,2,-1"));
        assert_eq!(f.get("residue.bump.at_i"), Some("0.0,-0.5"));
        // 1/(2i) = −i/2, and the bump's loop sum around i agrees
        let r = C::real(1.0) / C::new(0.0, 2.0);
        assert!((r - C::new(0.0, -0.5)).abs() < 1e-12);
        let loop_i = model::loop_sum(|z| Func::Bump.eval(z), I, 0.35, 256).unwrap();
        assert!((loop_i - C::new(0.0, TAU) * r).abs() < 1e-3);
        // the small planes do not overlap
        let s = small_planes();
        assert!(s[0].frame.right() < s[1].frame.left() && s[1].frame.right() < s[2].frame.left());
    }
}
