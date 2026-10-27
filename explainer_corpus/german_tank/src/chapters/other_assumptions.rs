//! The same five boxes lose their known left endpoint. Their four inner
//! gaps estimate both hidden ends. A separate, explicitly starts-at-one
//! experiment keeps ten boxes fixed while one outline selects either half.
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    STRIP_Y, dash_spans, fade_in_all, fade_out_all, fail, game_box, line_style, pause, say, stroke, think, wait_until,
    x_at,
};
use super::friends_line::{
    FriendsLine, Start, arc_points, badge, handfuls, list, now, printed, read_s, reshape, whole,
};
use super::one_estimate::{approx, fraction};
use crate::model::Q;
use crate::{facts, model, palette};

pub struct OtherAssumptions {
    pub kit: Kit,
}

const CARD: &str = "Fair draws without replacement, consecutive numbers, unknown first number, k > 1: estimate = s(1 + 2/(k − 1)) − 1. Here s is largest minus smallest. This estimates the count, not the last serial number.";
const LESSON: &str = "Equal chances for boxes do not make every handful equally likely.";

#[derive(Clone, Debug)]
struct Picture {
    friends: Vec<u64>,
    gaps: Vec<u64>,
    hidden: u64,
    inner: u64,
    average: Q,
    decimal: String,
    span: u64,
    estimate: u64,
    deliveries: Vec<Vec<u64>>,
    estimates: Vec<u64>,
    each: Q,
    mean: u64,
    made: u64,
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        friends: list(&f, "G-F1.numbers")?,
        gaps: list(&f, "G-F15.inner_gaps")?,
        hidden: whole(&f, "G-F15.hidden_count")?,
        inner: whole(&f, "G-F15.inner_count")?,
        average: fraction(&printed(&f, "G-F15.inner_average")?)?,
        decimal: printed(&f, "G-F15.inner_average_decimal")?,
        span: whole(&f, "G-F15.span_numbers")?,
        estimate: whole(&f, "G-F15.spread_estimate")?,
        deliveries: handfuls(&f, "G-F15.deliveries")?,
        estimates: list(&f, "G-F15.delivery_estimates")?,
        each: fraction(&printed(&f, "G-F15.deliveries_each_box")?)?,
        mean: whole(&f, "G-F15.deliveries_average")?,
        made: whole(&f, "G-F15.deliveries_made")?,
    })
}

fn checked(p: Picture) -> Result<Picture, String> {
    let k = p.friends.len() as u64;
    if k < 2
        || !p.friends.windows(2).all(|w| w[0] < w[1])
        || p.gaps != model::inner_gaps(&p.friends)
        || p.inner != k - 1
        || p.hidden != k + 1 - p.inner
        || Q::new(p.gaps.iter().sum::<u64>() as i128, p.inner as i128) != p.average
        || p.average.decimal1()? != p.decimal
        || p.span != p.friends[k as usize - 1] - p.friends[0] + 1
        || Q::int(p.span as i128) + Q::int(p.hidden as i128) * p.average != Q::int(p.estimate as i128)
        || model::spread_of(p.span - 1, k) != Q::int(p.estimate as i128)
    {
        return Err("unknown-start picture disagrees with the exact gaps and count".into());
    }
    if p.deliveries.len() != p.hidden as usize
        || p.estimates.len() != p.deliveries.len()
        || p.made != k * p.deliveries.len() as u64
        || p.mean == p.made
    {
        return Err("delivery control must have distinct estimates and a biased mean".into());
    }
    for (i, h) in p.deliveries.iter().enumerate() {
        let expected: Vec<u64> = (i as u64 * k + 1..=(i as u64 + 1) * k).collect();
        if *h != expected || model::gap_recipe(h) != Q::int(p.estimates[i] as i128) {
            return Err("delivery range or original-recipe estimate disagrees".into());
        }
    }
    for b in 1..=p.made {
        let appearances = p.deliveries.iter().filter(|h| h.contains(&b)).count();
        if Q::new(appearances as i128, p.deliveries.len() as i128) != p.each {
            return Err("boxes do not all have the claimed inclusion chance".into());
        }
    }
    if Q::new(p.estimates.iter().sum::<u64>() as i128, p.estimates.len() as i128) != Q::int(p.mean as i128) {
        return Err("delivery mean disagrees".into());
    }
    Ok(p)
}

fn label(kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, 28.0, color, v(0.0, y))
}

fn bracket(stage: &mut Stage<'_>, a: f64, b: f64, y: f64, color: Srgb) -> Mob {
    stroke(
        stage,
        vec![v(a, y + 0.1), v(a, y), v(b, y), v(b, y + 0.1)],
        line_style(color, 2.5, 1.0),
    )
}

fn delivery_outline(first: f64, last: f64, made: f64) -> Vec<Vec3> {
    let (a, b) = (x_at(first, made) - 0.43, x_at(last, made) + 0.43);
    vec![v(a, -0.45), v(b, -0.45), v(b, 0.45), v(a, 0.45), v(a, -0.45)]
}

/// Only the two complete deliveries can occur. Reposition the same border
/// while detached: nominal 1.2 s, one frame shorter at 15 fps (three at 60).
/// Later cue starts and inspection holds use unchanged absolute times.
fn switch_delivery(stage: &mut Stage<'_>, border: Mob, hand: &[u64], made: u64) -> fmn::Result<()> {
    fade_out_all(stage, &[border], 0.1)?;
    pause(stage, 0.1)?;
    reshape(
        stage,
        border,
        delivery_outline(hand[0] as f64, *hand.last().unwrap() as f64, made as f64),
    );
    pause(stage, 0.8)?;
    pause(stage, 0.1)?;
    fade_in_all(stage, &[border], 0.1)
}

impl SceneConstruct for OtherAssumptions {
    fn name(&self) -> &str {
        "15_other_assumptions"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = picture().map_err(fail)?;
        let end = model::LINE_DOTS as f64;
        let first = p.friends[0];
        let last = *p.friends.last().ok_or_else(|| fail("empty friends".into()))?;
        let friends = FriendsLine::build(
            &kit,
            stage,
            1,
            model::LINE_DOTS,
            &p.friends,
            &[],
            Start {
                end,
                fan: 0.0,
                rise: 1.0,
                widest: end,
            },
        )?;
        let corner = badge(&kit, stage, v(-5.1, 3.5))?;
        let from_one = kit.text(stage, "1", 22.0, Palette::ink(), v(x_at(1.0, end), -0.95))?;
        let l = say(&kit, stage, "15_1")?;
        fade_in_all(stage, &friends.opening(), 0.5)?;
        fade_in_all(stage, &[corner, from_one], 0.3)?;
        l.wait_for(stage, "an unknown number")?;
        let mut hidden = vec![from_one];
        hidden.extend(
            (1..first)
                .chain(last + 1..=model::LINE_DOTS)
                .map(|n| friends.dots[(n - 1) as usize]),
        );
        fade_out_all(stage, &hidden, 0.5)?;
        let unknown = label(
            &kit,
            stage,
            "First number unknown; same five observed boxes",
            2.7,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[unknown], 0.4)?;

        let l = say(&kit, stage, "15_2")?;
        let mut ends = Vec::new();
        for (a, b) in [
            (x_at(1.0, end), x_at(first as f64 - 0.6, end)),
            (x_at(last as f64 + 0.6, end), x_at(end, end)),
        ] {
            for (x, y) in dash_spans(a, b) {
                ends.push(stroke(
                    stage,
                    vec![v(x, -1.15), v(y, -1.15)],
                    line_style(Palette::hero(), 2.5, 1.0),
                ));
            }
            ends.push(kit.text(stage, "?", 28.0, Palette::hero(), v((a + b) / 2.0, -1.5))?);
        }
        fade_in_all(stage, &ends, 0.5)?;
        l.wait_for(stage, "four inner gaps")?;
        let mut arcs = Vec::new();
        let mut counts = Vec::new();
        for (w, gap) in p.friends.windows(2).zip(&p.gaps) {
            let a = x_at((w[0] + 1) as f64, end);
            let b = x_at((w[1] - 1) as f64, end);
            arcs.push(stroke(
                stage,
                arc_points(a, b, STRIP_Y - 0.17, -0.3),
                line_style(palette::unseen(), 2.5, 1.0),
            ));
            counts.push(kit.text(
                stage,
                &gap.to_string(),
                24.0,
                palette::unseen(),
                v((a + b) / 2.0, -1.45),
            )?);
        }
        fade_in_all(stage, &arcs, 0.5)?;
        fade_in_all(stage, &counts, 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let l = say(&kit, stage, "15_3")?;
        fade_out_all(stage, &[unknown], 0.3)?;
        let total: u64 = p.gaps.iter().sum();
        let sum = label(
            &kit,
            stage,
            &format!(
                "{} = {total}",
                p.gaps.iter().map(u64::to_string).collect::<Vec<_>>().join(" + ")
            ),
            2.7,
            palette::unseen(),
        )?;
        l.wait_for(stage, "forty-two boxes")?;
        fade_in_all(stage, &[sum], 0.4)?;
        l.wait_for(stage, "ten and a half per gap")?;
        let average = label(
            &kit,
            stage,
            &format!("{total} / {} = {} unseen boxes per inner gap", p.inner, p.decimal),
            2.05,
            palette::unseen(),
        )?;
        fade_in_all(stage, &[average], 0.4)?;
        l.wait_for(stage, "spans forty-seven numbers")?;
        // Half-cell edges make the inclusive span and the estimated added
        // counts use the same scale. No sampled endpoint is counted twice.
        let a = x_at(first as f64 - 0.5, end);
        let b = x_at(last as f64 + 0.5, end);
        let span = bracket(stage, a, b, -2.0, Palette::ink());
        let span_label = kit.text(
            stage,
            &format!("{} numbers, including both ends", p.span),
            25.0,
            Palette::ink(),
            v((a + b) / 2.0, -2.4),
        )?;
        fade_in_all(stage, &[span, span_label], 0.5)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let _l = say(&kit, stage, "15_4")?;
        fade_out_all(stage, &ends, 0.3)?;
        let width = (x_at(2.0, end) - x_at(1.0, end)) * approx(p.average).map_err(fail)?;
        let add_left = bracket(stage, a - width, a, -2.0, Palette::hero());
        let add_right = bracket(stage, b, b + width, -2.0, Palette::hero());
        let left_number = kit.text(stage, &p.decimal, 25.0, Palette::hero(), v(a - width / 2.0, -2.4))?;
        let right_number = kit.text(stage, &p.decimal, 25.0, Palette::hero(), v(b + width / 2.0, -2.4))?;
        let estimated = label(
            &kit,
            stage,
            "Estimate the missing counts at both ends",
            1.35,
            Palette::hero(),
        )?;
        fade_in_all(stage, &[add_left, add_right, left_number, right_number, estimated], 0.5)?;
        let q = think(&kit, stage, v(4.8, 0.9))?;
        let _l = say(&kit, stage, "15_5")?;
        fade_out_all(stage, &[q, estimated], 0.3)?;
        let answer = label(
            &kit,
            stage,
            &format!("{} + {} × {} = {} boxes", p.span, p.hidden, p.decimal, p.estimate),
            1.35,
            palette::estimate(),
        )?;
        fade_in_all(stage, &[answer], 0.5)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;
        let mut lower = arcs;
        lower.extend(counts);
        lower.extend([span, span_label, add_left, add_right, left_number, right_number]);
        fade_out_all(stage, &lower, 0.4)?;
        let card = precisely(&kit, stage, CARD)?;
        fade_in_all(stage, &[card], 0.4)?;
        let fully = now(stage);
        wait_until(stage, fully + read_s(CARD))?;
        pause(stage, 0.8)?;
        let mut opening = friends.opening();
        opening.retain(|m| !hidden.contains(m));
        opening.extend([corner, sum, average, answer, card]);
        fade_out_all(stage, &opening, 0.6)?;

        let l = say(&kit, stage, "15_6")?;
        let warning = label(&kit, stage, "Same chance for each box?", 2.8, Palette::ink())?;
        fade_in_all(stage, &[warning], 0.4)?;
        l.wait_for(stage, "Every handful of five")?;
        let rule = label(
            &kit,
            stage,
            "The rule: every handful equally likely",
            2.1,
            Palette::hero(),
        )?;
        fade_in_all(stage, &[rule], 0.4)?;

        let l = say(&kit, stage, "15_7")?;
        let mut boxes = Vec::new();
        for n in 1..=p.made {
            // Same ten objects persist for both possible deliveries.
            let at = v(x_at(n as f64, p.made as f64), 0.0);
            let [body, _lid] = game_box(stage, at)?;
            stage.scale(body, 0.7);
            boxes.push(body);
            boxes.push(kit.text(stage, &n.to_string(), 28.0, palette::seen(), at)?);
        }
        let original = label(
            &kit,
            stage,
            "Original recipe: numbering starts at one",
            1.35,
            Palette::dim(),
        )?;
        fade_in_all(stage, &boxes, 0.6)?;
        fade_in_all(stage, &[original], 0.4)?;
        let outline = stroke(
            stage,
            delivery_outline(
                p.deliveries[0][0] as f64,
                *p.deliveries[0].last().unwrap() as f64,
                p.made as f64,
            ),
            line_style(Palette::hero(), 3.0, 1.0),
        );
        l.wait_for(stage, "boxes one to five")?;
        fade_in_all(stage, &[outline], 0.4)?;
        let mut deliveries = Vec::new();
        for (i, h) in p.deliveries.iter().enumerate() {
            let x = if i == 0 { -3.5 } else { 3.5 };
            deliveries.push(kit.text(
                stage,
                &format!("{} to {}: chance {}", h[0], h.last().unwrap(), p.each),
                25.0,
                Palette::hero(),
                v(x, -0.95),
            )?);
        }
        fade_in_all(stage, &deliveries[..1], 0.3)?;
        l.wait_for(stage, "or boxes six to ten")?;
        switch_delivery(stage, outline, &p.deliveries[1], p.made)?;
        fade_in_all(stage, &deliveries[1..], 0.3)?;

        let _l = say(&kit, stage, "15_8")?;
        let mut chances = Vec::new();
        for n in 1..=p.made {
            chances.push(kit.text(
                stage,
                &p.each.to_string(),
                22.0,
                palette::unseen(),
                v(x_at(n as f64, p.made as f64), 0.75),
            )?);
        }
        fade_in_all(stage, &chances, 0.5)?;
        switch_delivery(stage, outline, &p.deliveries[0], p.made)?;
        let jointly = label(
            &kit,
            stage,
            "Only these deliveries can occur; mixed handfuls never do",
            -1.65,
            Palette::red(),
        )?;
        fade_in_all(stage, &[jointly], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.0)?;

        let l = say(&kit, stage, "15_9")?;
        let mut recipes = Vec::new();
        for (i, h) in p.deliveries.iter().enumerate() {
            let maximum = *h.last().unwrap();
            let k = h.len();
            recipes.push(kit.text(
                stage,
                &format!("{maximum} + ({maximum} − {k})/{k} = {}", p.estimates[i]),
                26.0,
                palette::estimate(),
                v(if i == 0 { -3.5 } else { 3.5 }, -2.3),
            )?);
        }
        fade_in_all(stage, &recipes, 0.6)?;
        l.wait_for(stage, "average eight, not ten")?;
        let mean = label(
            &kit,
            stage,
            &format!(
                "({} + {}) / {} = {}     actual count: {}",
                p.estimates[0],
                p.estimates[1],
                p.deliveries.len(),
                p.mean,
                p.made
            ),
            -2.95,
            Palette::red(),
        )?;
        fade_in_all(stage, &[mean], 0.5)?;
        kit.hold(stage)?;
        pause(stage, 3.0)?;
        fade_out_all(stage, &[warning, rule], 0.4)?;
        let lesson = label(&kit, stage, LESSON, 2.65, Palette::ink())?;
        fade_in_all(stage, &[lesson], 0.4)?;
        let fully = now(stage);
        wait_until(stage, fully + read_s(LESSON))?;
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
    fn endpoint_counts_and_dependent_deliveries_are_distinct_models() {
        let p = picture().unwrap();
        let mut wrong = p.clone();
        wrong.span -= 1;
        assert!(checked(wrong).is_err());
        let mut wrong = p.clone();
        wrong.hidden -= 1;
        assert!(checked(wrong).is_err());
        let mut wrong = p.clone();
        wrong.mean = p.made;
        assert!(checked(wrong).is_err());
        let mut wrong = p;
        wrong.estimates[1] -= 1;
        assert!(checked(wrong).is_err());
    }
}
