//! The same friends' boxes can estimate a shop while missing the company.
//! A persistent red outline moves between two eligible sampling pools.
//! Bars are the exact maximum distribution, not a simulated experiment.
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    STRIP_Y, fade_in_all, fade_out_all, fail, flat, line_style, pause, say, stroke, think, wait_until, x_at,
};
use super::friends_line::{FriendsLine, Start, arc_points, badge, bind_to, list, now, read_s, reshape, whole};
use super::one_estimate::{approx, fraction};
use super::what_we_assume::{halo, words_left};
use crate::model::Q;
use crate::{facts, model, palette};

pub struct AnUnfairDraw {
    pub kit: Kit,
}

const WARNING: &str = "An unlikely handful is a warning, not proof.";
const DISTRIBUTION: &str = "Shop estimates; every handful equally likely";

#[derive(Clone, Debug)]
struct Picture {
    friends: Vec<u64>,
    late: Vec<u64>,
    made: u64,
    shop: u64,
    from: u64,
    estimate: u64,
    mean: u64,
    short: u64,
    late_estimate: u64,
    late_mean: u64,
    gaps: Vec<u64>,
    chance: Q,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    let mut gaps = vec![whole(&f, "G-F14.late_first_gap")?];
    gaps.extend(list(&f, "G-F14.late_other_gaps")?);
    checked(Picture {
        friends: list(&f, "G-F14.shop_handful")?,
        late: list(&f, "G-F14.late_handful")?,
        made: whole(&f, "G-F14.made")?,
        shop: whole(&f, "G-F14.shop_boxes")?,
        from: whole(&f, "G-F14.late_from")?,
        estimate: whole(&f, "G-F14.shop_estimate")?,
        mean: whole(&f, "G-F14.shop_average")?,
        short: whole(&f, "G-F14.shop_short_by")?,
        late_estimate: whole(&f, "G-F14.late_estimate")?,
        late_mean: whole(&f, "G-F14.late_average")?,
        gaps,
        chance: fraction(f.get("G-F14.fair_all_above").ok_or("missing fair probability")?)?,
    })
}

fn checked(p: Picture) -> Result<Picture, String> {
    let k = p.friends.len() as u64;
    if k == 0
        || p.late.len() != k as usize
        || p.shop >= p.made
        || p.from <= 1
        || p.from > p.made
        || !p.friends.windows(2).all(|w| w[0] < w[1])
        || !p.late.windows(2).all(|w| w[0] < w[1])
        || p.friends.iter().any(|&n| n == 0 || n > p.shop)
        || p.late.iter().any(|&n| n < p.from || n > p.made)
    {
        return Err("invalid company, pool or sorted handful".into());
    }
    let int = |n| Q::int(n as i128);
    let pool = p.made - p.from + 1;
    if model::gap_recipe(&p.friends) != int(p.estimate)
        || model::gap_recipe(&p.late) != int(p.late_estimate)
        || model::gaps(&p.late) != p.gaps
        || model::mean_by_largest(p.shop, k, |m| model::gap_of(m, k)) != int(p.mean)
        || model::mean_by_largest(pool, k, |m| model::gap_of(p.from - 1 + m, k)) != int(p.late_mean)
        || p.made - p.mean != p.short
        || model::all_at_most(pool, p.made, k) != p.chance
        || p.chance <= Q::ZERO
        || p.chance >= Q::new(1, 1000)
    {
        return Err("sample, gaps, exact distribution or fair-draw probability disagree with facts".into());
    }
    Ok(p)
}

fn pool_points(first: f64, last: f64, end: f64) -> Vec<Vec3> {
    let (a, b) = (x_at(first, end) - 0.10, x_at(last, end) + 0.10);
    let (lo, hi) = (STRIP_Y - 0.14, STRIP_Y + 0.14);
    vec![v(a, lo), v(b, lo), v(b, hi), v(a, hi), v(a, lo)]
}

fn caption(kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    words_left(kit, stage, text, &[(text.to_owned(), color)], v(-5.9, y))
}

/// Columns sit at the estimate made by each possible maximum. Their height
/// is proportional to C(m-1,k-1); each handful therefore gets equal weight.
fn distribution(stage: &mut Stage<'_>, p: &Picture) -> fmn::Result<Vec<Mob>> {
    let k = p.friends.len() as u64;
    let max_weight = model::with_largest(p.shop, k);
    let end = p.made as f64;
    let width = (x_at(approx(model::gap_of(k + 1, k)).map_err(fail)?, end) - x_at(k as f64, end)) * 0.8;
    let mut out = vec![stroke(
        stage,
        vec![
            v(x_at(k as f64, end), -2.45),
            v(x_at(approx(model::gap_of(p.shop, k)).map_err(fail)?, end), -2.45),
        ],
        line_style(Palette::dim(), 1.5, 1.0),
    )];
    for m in k..=p.shop {
        let h = 0.85 * approx(model::with_largest(m, k) / max_weight).map_err(fail)?;
        let rect = Rectangle::new()
            .width(width)
            .height(h.max(0.0001))
            .style(flat(palette::estimate(), 0.7))
            .build()?
            .moved_to(v(
                x_at(approx(model::gap_of(m, k)).map_err(fail)?, end),
                -2.45 + h / 2.0,
            ));
        out.push(stage.arena_mut().add(rect));
    }
    out.push(stroke(
        stage,
        vec![v(x_at(p.mean as f64, end), -2.55), v(x_at(p.mean as f64, end), -1.45)],
        line_style(Palette::ink(), 2.0, 1.0),
    ));
    Ok(out)
}

fn card_text(p: &Picture) -> String {
    let k = p.friends.len();
    format!(
        "For {k} fair draws without replacement from a+1 to a+P: mean estimate = P + {}a/{k}. Here a is the starting offset, P the pool size. An unlikely handful alone cannot prove unfair sampling.",
        k + 1
    )
}

impl SceneConstruct for AnUnfairDraw {
    fn name(&self) -> &str {
        "14_an_unfair_draw"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = picture().map_err(fail)?;
        let end = p.made as f64;
        let start = || Start {
            end,
            fan: 1.0,
            rise: 1.0,
            widest: end,
        };
        let friends = FriendsLine::build(&kit, stage, 1, p.made, &p.friends, &[], start())?;
        let corner = badge(&kit, stage, v(-5.1, 3.5))?;
        let l = say(&kit, stage, "14_1")?;
        fade_in_all(stage, &friends.opening(), 0.5)?;
        fade_in_all(stage, &[corner], 0.3)?;
        l.wait_for(stage, "Now break it")?;
        let cross = stroke(
            stage,
            vec![v(-6.3, 3.25), v(-3.85, 3.75)],
            line_style(Palette::red(), 3.0, 1.0),
        );
        fade_in_all(stage, &[cross], 0.3)?;
        l.wait_for(stage, "one store")?;
        let a = stage.add_value_tracker(1.0);
        let b = stage.add_value_tracker(p.shop as f64);
        let pool = stroke(
            stage,
            pool_points(1.0, p.shop as f64, end),
            line_style(Palette::red(), 3.0, 1.0),
        );
        bind_to(stage, &[a, b], move |s, values| {
            reshape(s, pool, pool_points(values[0], values[1], end))
        })?;
        let shop_text = format!("Only this shop: 1 to {}", p.shop);
        let shop_label = caption(&kit, stage, &shop_text, 2.65, Palette::red())?;
        fade_in_all(stage, &[pool, shop_label], 0.5)?;

        let l = say(&kit, stage, "14_2")?;
        l.wait_for(stage, "two hundred")?;
        let company = kit.text(
            stage,
            &format!("company: {}", p.made),
            26.0,
            Palette::ink(),
            v(4.8, 1.0),
        )?;
        let mark = stroke(
            stage,
            vec![v(x_at(end, end), STRIP_Y - 0.3), v(x_at(end, end), 0.6)],
            line_style(Palette::ink(), 2.5, 1.0),
        );
        fade_in_all(stage, &[company, mark], 0.4)?;
        l.wait_for(stage, "still says seventy-one")?;
        let estimate = caption(
            &kit,
            stage,
            &format!("Same friends; same estimate: {}", p.estimate),
            1.85,
            palette::estimate(),
        )?;
        fade_in_all(stage, &[estimate], 0.4)?;

        let l = say(&kit, stage, "14_3")?;
        let bars = distribution(stage, &p)?;
        let dist_label = caption(&kit, stage, DISTRIBUTION, -2.95, Palette::dim())?;
        fade_in_all(stage, &bars, 0.8)?;
        fade_in_all(stage, &[dist_label], 0.4)?;
        l.wait_for(stage, "exactly seventy-one")?;
        let mean = kit.text(
            stage,
            &format!("mean {}", p.mean),
            25.0,
            palette::estimate(),
            v(-2.8, -1.25),
        )?;
        fade_in_all(stage, &[mean], 0.4)?;
        l.wait_for(stage, "short by a hundred and twenty-nine")?;
        let absent = stroke(
            stage,
            vec![
                v(x_at(p.shop as f64, end), -1.0),
                v(x_at(p.shop as f64, end), -1.15),
                v(x_at(end, end), -1.15),
                v(x_at(end, end), -1.0),
            ],
            line_style(Palette::red(), 2.5, 1.0),
        );
        let short = kit.text(
            stage,
            &format!("{} never on sale here", p.short),
            25.0,
            Palette::red(),
            v(2.7, -1.55),
        )?;
        fade_in_all(stage, &[absent, short], 0.5)?;
        kit.hold(stage)?;
        pause(stage, 3.0)?;

        let l = say(&kit, stage, "14_4")?;
        let mut old = bars;
        old.extend([dist_label, mean, absent, short, estimate, shop_label]);
        fade_out_all(stage, &old, 0.4)?;
        l.wait_for(stage, "from a hundred and fifty-one up")?;
        let move_a = stage.prepare(tween(a, p.from as f64, 1.8)?)?;
        let move_b = stage.prepare(tween(b, end, 1.8)?)?;
        stage.play_prepared(vec![move_a, move_b])?;
        let late_label = caption(
            &kit,
            stage,
            &format!("Only the late shelf: {} to {}", p.from, p.made),
            2.65,
            Palette::red(),
        )?;
        // The last sampled box is the company endpoint itself. Keep its
        // marker below the box instead of drawing through the number 200.
        reshape(
            stage,
            mark,
            vec![v(x_at(end, end), STRIP_Y - 0.3), v(x_at(end, end), STRIP_Y + 0.25)],
        );
        let late = FriendsLine::build(&kit, stage, 1, p.made, &p.late, &[], Start { fan: 0.0, ..start() })?;
        // Short vertical connections keep the late shelf traceable. Narrow
        // only the box bodies to fit the closest pair; preserve label size
        // and body height so the connector endpoints still meet their feet.
        let width = 0.8
            * p.late
                .windows(2)
                .map(|w| x_at(w[1] as f64, end) - x_at(w[0] as f64, end))
                .fold(f64::INFINITY, f64::min);
        for &body in &late.bodies {
            stage.stretch_to_fit_width(body, width);
        }
        // Keep the axis and its unaltered dots. A different handful changes
        // only the boxes, their connectors, and the ten selected-dot roles.
        let mut gone: Vec<Mob> = friends
            .bodies
            .iter()
            .chain(&friends.labels)
            .chain(&friends.ticks)
            .copied()
            .collect();
        let mut arriving: Vec<Mob> = late
            .bodies
            .iter()
            .chain(&late.labels)
            .chain(&late.ticks)
            .copied()
            .collect();
        for n in 1..=p.made {
            if p.friends.contains(&n) || p.late.contains(&n) {
                gone.push(friends.dots[(n - 1) as usize]);
                arriving.push(late.dots[(n - 1) as usize]);
            }
        }
        fade_out_all(stage, &gone, 0.4)?;
        fade_in_all(stage, &arriving, 0.5)?;
        fade_in_all(stage, &[late_label], 0.4)?;
        let q = think(&kit, stage, v(1.6, 1.85))?;

        let l = say(&kit, stage, "14_5")?;
        fade_out_all(stage, &[q], 0.3)?;
        let answer = caption(
            &kit,
            stage,
            &format!("This handful estimates {}", p.late_estimate),
            1.85,
            palette::estimate(),
        )?;
        fade_in_all(stage, &[answer], 0.4)?;
        l.wait_for(stage, "gap below the smallest number")?;
        let mut arcs = Vec::new();
        let mut previous = 0;
        for (&n, &gap) in p.late.iter().zip(&p.gaps) {
            let left = x_at((previous + 1) as f64, end);
            let right = x_at((n - 1) as f64, end);
            arcs.push(stroke(
                stage,
                arc_points(left, right, STRIP_Y - 0.25, -0.3),
                line_style(palette::unseen(), 2.0, 1.0),
            ));
            let label = kit.text(
                stage,
                &gap.to_string(),
                23.0,
                palette::unseen(),
                v((left + right) / 2.0, -1.4),
            )?;
            arcs.push(label);
            previous = n;
        }
        fade_in_all(stage, &arcs, 0.6)?;
        let sum: u64 = p.gaps.iter().sum();
        let calculation = caption(
            &kit,
            stage,
            &format!("{} + {sum}/{} = {}", p.made, p.gaps.len(), p.late_estimate),
            -2.1,
            palette::estimate(),
        )?;
        let late_mean = caption(
            &kit,
            stage,
            &format!("Average over this shelf: {}", p.late_mean),
            -2.75,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[calculation, late_mean], 0.5)?;

        let l = say(&kit, stage, "14_6")?;
        let glow = halo(stage, &arcs[..2], Palette::red())?;
        fade_in_all(stage, &[glow], 0.3)?;
        l.wait_for(stage, "less than once in a thousand")?;
        fade_out_all(stage, &[calculation, late_mean], 0.4)?;
        let rare = caption(&kit, stage, "Fair draw: less than 1 in 1,000", -2.1, Palette::red())?;
        fade_in_all(stage, &[rare], 0.4)?;

        let l = say(&kit, stage, "14_7")?;
        fade_out_all(stage, &[glow], 0.3)?;
        let other = halo(stage, &late.bodies[..late.bodies.len() - 1], palette::seen())?;
        fade_in_all(stage, &[other], 0.4)?;
        l.wait_for(stage, "An unlikely handful is a warning")?;
        let warning = caption(&kit, stage, WARNING, -2.75, Palette::ink())?;
        fade_in_all(stage, &[warning], 0.4)?;
        kit.hold(stage)?;
        pause(stage, read_s(WARNING))?;
        fade_out_all(stage, &[rare, warning], 0.4)?;
        let text = card_text(&p);
        let card = precisely(&kit, stage, &text)?;
        fade_in_all(stage, &[card], 0.4)?;
        let up = now(stage);
        wait_until(stage, up + read_s(&text))?;
        pause(stage, 1.0)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_two_pools_disagree_with_the_company_and_mutated_claims_fail() {
        let p = picture().unwrap();
        assert!(p.mean < p.made && p.late_mean > p.made);
        let mut wrong = p.clone();
        wrong.mean = p.made;
        assert!(checked(wrong).is_err());
        let mut wrong = p.clone();
        wrong.late_estimate = p.made;
        assert!(checked(wrong).is_err());
        let mut wrong = p.clone();
        wrong.chance = Q::ZERO;
        assert!(checked(wrong).is_err());
        let mut wrong = p;
        wrong.gaps[0] -= 1;
        assert!(checked(wrong).is_err());
    }
}
