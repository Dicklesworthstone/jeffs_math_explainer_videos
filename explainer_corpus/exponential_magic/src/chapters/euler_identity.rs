//! Chapter 14: the three constructions, run at i times pi, land on one point, and only then is the famous line written.
//! Every picture comes from `turning_model`, which has no complex exponential to copy. Author source, not built or
//! rendered.
use super::ending_draw::{
    Plane, arrow, arrow_parts, fade_in_all, fade_in_lagged, fade_out_all, flat, line_of_math, line_style, live, path,
    ring, swap, title, walk,
};
use crate::phrase::until_phrase;
use crate::turning_model::{self as t, C};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use std::f64::consts::PI;

pub struct EulerIdentity {
    pub kit: Kit,
}

/// Think time after the 14_1 question, in seconds. Review 721 M10 proposed two to three seconds for this pair. The
/// value is authored here and still has to be set against Frederick Surrey takes in audio-review.
const THINK_14_1: f64 = 2.5;
/// Review 721 W3: true of every step, not only the first.
const OUTWARD: &str = "each step ends farther from zero";
/// Review 721 W6: 13_4 has already given the walk's answer, so the question is whether the paths agree with it.
const QUESTION: &str = "do these paths agree?";
/// Review 721 W8: named once 14_4 has earned it.
const IDENTITY_NAME: &str = "Euler's identity";
const CARD_X: f64 = 4.4;
const NEAR_CARD_X: f64 = 3.4;
const ZERO: C = C::new(0.0, 0.0);
const MINUS_ONE: C = C::new(-1.0, 0.0);
/// The two angles added in 14_5, chosen so that their arcs and their sum are all easy to see.
const FIRST_ANGLE: f64 = 0.7;
const SECOND_ANGLE: f64 = 1.25;
const SPIRAL_SECONDS: f64 = 5.0;

const STEP_RULE: &str = r"z_{j+1}=z_j+\frac{\pi}{n}\,i\,z_j";
const LENGTH_FACTOR: &str = r"\sqrt{1+(\pi/n)^2}";
const ONE_EXCESS: &str = r"\text{each excess}\approx\frac{\pi^2}{2n^2}";
const ALL_EXCESS: &str = r"\text{all of them}\approx\frac{\pi^2}{2n}";
const ANGLES: &str = r"\text{the angles add up to nearly}\ \pi";
const SERIES_AT_PI: &str = r"E(i\pi)=\sum_{k}\frac{(i\pi)^k}{k!}";
const NEXT_TERM: &str = r"\text{term}_k=\frac{\pi}{k}\,i\,\text{term}_{k-1}";
const WHILE_SMALL: &str = r"\frac{\pi}{k}>1\ \text{while}\ k<\pi";
const LANDING: &str = r"E(i\pi)=-1";
const WRITTEN: &str = r"\text{we write}\ e^{i\pi}";
const FAMOUS: &str = r"e^{i\pi}+1=0";
const HALF_LENGTH: &str = r"\text{half the circle has length}\ \pi";
const PRODUCT: &str = r"E(is)\,E(it)=E\left(i(s+t)\right)";
const COS_SUM: &str = r"\cos(s+t)=\cos s\cos t-\sin s\sin t";
const SIN_SUM: &str = r"\sin(s+t)=\sin s\cos t+\cos s\sin t";
const COS_SERIES: &str = r"\cos t=1-\frac{t^2}{2!}+\frac{t^4}{4!}-\cdots";
const SIN_SERIES: &str = r"\sin t=t-\frac{t^3}{3!}+\frac{t^5}{5!}-\cdots";
const BOTH_PARTS: &str = r"E\left((a+iw)\,t\right)=E(a\,t)\,E(iw\,t)";

/// Wide enough for the partial sums of the series, which swing out to about -3.93 and up to 3.14.
fn far() -> Plane {
    Plane {
        centre: v(-2.5, -0.3),
        scale: 0.85,
        half_w: 3.9,
        half_h: 2.75,
    }
}
fn near() -> Plane {
    Plane {
        centre: v(-3.3, -0.25),
        scale: 1.85,
        half_w: 3.0,
        half_h: 2.55,
    }
}
fn math(k: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, c: Srgb, y: f64) -> fmn::Result<Mob> {
    line_of_math(k, stage, src, size, c, CARD_X, y)
}
fn card(k: &Kit, stage: &mut Stage<'_>, words: &str, c: Srgb, y: f64) -> fmn::Result<Mob> {
    k.text(stage, words, 24.0, c, v(CARD_X, y))
}
fn near_math(k: &Kit, stage: &mut Stage<'_>, src: &str, size: f64, c: Srgb, y: f64) -> fmn::Result<Mob> {
    line_of_math(k, stage, src, size, c, NEAR_CARD_X, y)
}
/// An arrow between two complex numbers on a plane.
fn between(stage: &mut Stage<'_>, plane: Plane, a: C, b: C, c: Srgb, w: f64) -> Mob {
    arrow(stage, plane.p(a), plane.p(b), c, w)
}
fn polygon(stage: &mut Stage<'_>, n: u32, c: Srgb, w: f64) -> Mob {
    let points = t::tangent_steps(PI, n).into_iter().map(|z| far().p(z)).collect();
    path(stage, points, c, w)
}

struct Stay {
    heading: Mob,
    axes: Mob,
    circle: Mob,
}

/// 14_1: the tiny multipliers at i times pi. Four cells, each step at a right angle to its own arrow. The landing
/// point is not marked yet.
fn tangent_walk(k: &Kit, stage: &mut Stage<'_>) -> fmn::Result<(Stay, Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "14_1")?;
    let plane = far();
    let heading = title(k, stage, "Where does each construction land?")?;
    let axes = plane.axes(stage)?;
    let circle = plane.unit_circle(stage);
    fade_in_all(stage, &[heading, axes, circle], 0.5)?;

    k.until_sentence(stage, "14_1", 1)?;
    let name = card(k, stage, "tiny multipliers", Palette::ink(), 2.3)?;
    fade_in_all(stage, &[name], 0.4)?;
    k.until_sentence(stage, "14_1", 2)?;
    let cells = math(
        k,
        stage,
        r"\text{time}\ \pi,\ n\ \text{cells}",
        28.0,
        Palette::dim(),
        1.65,
    )?;
    fade_in_all(stage, &[cells], 0.4)?;

    k.until_sentence(stage, "14_1", 3)?;
    let points = t::tangent_steps(PI, 4);
    let rule = math(k, stage, STEP_RULE, 31.0, Palette::ink(), 0.8)?;
    let turned = between(
        stage,
        plane,
        points[0],
        points[0].plus(points[0].times_i()),
        Palette::faint(),
        3.5,
    );
    fade_in_all(stage, &[rule, turned], 0.5)?;
    let first = between(stage, plane, points[0], points[1], Palette::hero(), 4.5);
    fade_in_all(stage, &[first], 0.5)?;

    k.until_sentence(stage, "14_1", 4)?;
    fade_out_all(stage, &[turned], 0.3)?;
    let mut steps = vec![first];
    let rest: Vec<Mob> = points[1..]
        .windows(2)
        .map(|pair| between(stage, plane, pair[0], pair[1], Palette::hero(), 4.5))
        .collect();
    fade_in_lagged(stage, &rest, 1.3, 0.3)?;
    steps.extend(rest);

    k.until_sentence(stage, "14_1", 5)?;
    let outward = card(k, stage, OUTWARD, Palette::dim(), -0.1)?;
    fade_in_all(stage, &[outward], 0.4)?;
    k.until_sentence(stage, "14_1", 7)?;
    let question = k.text(stage, QUESTION, 28.0, Palette::hero(), v(CARD_X, -1.0))?;
    fade_in_all(stage, &[question], 0.5)?;
    k.hold(stage)?;
    k.mark(stage, "predict");
    k.music_breath(stage, THINK_14_1)?;

    let stay = Stay { heading, axes, circle };
    Ok((stay, vec![name, cells, rule, outward, question], steps))
}

/// 14_2: the steps miss by less and less, because the excess length is of order one over n squared. The ring at -1
/// and its name are returned together, so 14_5 fades both (review 721 M7).
fn they_land(k: &Kit, stage: &mut Stage<'_>, cards: &[Mob]) -> fmn::Result<([Mob; 2], Vec<Mob>, Vec<Mob>)> {
    k.say(stage, "14_2")?;
    let plane = far();
    fade_out_all(stage, cards, 0.4)?;
    let landing = ring(stage, plane.p(MINUS_ONE), 0.14, Palette::green(), 3.5);
    let name = {
        let at = plane.p(MINUS_ONE);
        k.tex(stage, "-1", 27.0, Palette::green(), &[], v(at[0] - 0.65, at[1] - 0.38))?
    };
    fade_in_all(stage, &[landing, name], 0.4)?;

    k.until_sentence(stage, "14_2", 1)?;
    let factor = math(k, stage, LENGTH_FACTOR, 33.0, Palette::ink(), 2.3)?;
    fade_in_all(stage, &[factor], 0.5)?;
    k.until_sentence(stage, "14_2", 2)?;
    let excess = [
        math(k, stage, ONE_EXCESS, 27.0, Palette::dim(), 1.35)?,
        math(k, stage, ALL_EXCESS, 27.0, Palette::dim(), 0.55)?,
    ];
    fade_in_all(stage, &excess, 0.5)?;
    k.until_sentence(stage, "14_2", 3)?;
    let angles = math(k, stage, ANGLES, 26.0, Palette::ink(), -0.15)?;
    fade_in_all(stage, &[angles], 0.4)?;

    k.until_sentence(stage, "14_2", 4)?;
    let widths = [3.0, 2.5, 2.0, 1.6];
    let colours = [Palette::hero(), Palette::teal(), Palette::blue(), Palette::gold()];
    let mut polygons = Vec::new();
    let mut rows = Vec::new();
    for (i, n) in t::CELLS.into_iter().enumerate() {
        let miss = t::tangent_end(PI, n).plus(t::ONE).abs();
        let row = line_of_math(
            k,
            stage,
            &format!(r"{n}\ \text{{cells: off by}}\ {}", t::five(miss)),
            25.0,
            colours[i],
            CARD_X,
            -0.8 - 0.5 * i as f64,
        )?;
        // Review 721 L4: each count waits for its own words, so 1024 is not on screen while the voice is on sixteen.
        match n {
            64 => until_phrase(k, stage, "14_2", "sixty-four")?,
            1024 => until_phrase(k, stage, "14_2", "one thousand twenty-four")?,
            _ => {}
        }
        if n == 4 {
            fade_in_all(stage, &[row], 0.3)?;
        } else {
            let walk_path = polygon(stage, n, colours[i], widths[i]);
            fade_in_all(stage, &[walk_path, row], 0.4)?;
            polygons.push(walk_path);
        }
        rows.push(row);
    }
    k.until_sentence(stage, "14_2", 5)?;
    let note = card(
        k,
        stage,
        "the proof only compared the sizes of terms",
        Palette::dim(),
        -2.9,
    )?;
    fade_in_all(stage, &[note], 0.4)?;
    k.hold(stage)?;

    let mut leaving = vec![factor, excess[0], excess[1], angles, note];
    leaving.extend(rows);
    Ok(([landing, name], leaving, polygons))
}

/// 14_3: the series as arrows laid tip to tail. Each is the last one turned and rescaled by pi over k.
fn spiral_of_terms(
    k: &Kit,
    stage: &mut Stage<'_>,
    cards: &[Mob],
    walks: &[Mob],
    steps: &[Mob],
) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "14_3")?;
    let plane = far();
    fade_out_all(stage, cards, 0.4)?;
    fade_out_all(stage, walks, 0.4)?;
    fade_out_all(stage, steps, 0.4)?;

    let sums = t::partial_sums(PI, t::TERMS);
    let arrows: Vec<Mob> = sums
        .windows(2)
        .enumerate()
        .map(|(j, pair)| {
            let colour = if j <= 3 { Palette::gold() } else { Palette::teal() };
            between(stage, plane, pair[0], pair[1], colour, 4.0)
        })
        .collect();

    k.until_sentence(stage, "14_3", 1)?;
    let series = math(k, stage, SERIES_AT_PI, 31.0, Palette::ink(), 2.3)?;
    fade_in_all(stage, &[series, arrows[0]], 0.5)?;
    k.until_sentence(stage, "14_3", 2)?;
    let next = math(k, stage, NEXT_TERM, 29.0, Palette::ink(), 1.25)?;
    fade_in_all(stage, &[next, arrows[1]], 0.5)?;
    k.until_sentence(stage, "14_3", 3)?;
    let grow = math(k, stage, WHILE_SMALL, 28.0, Palette::gold(), 0.35)?;
    fade_in_all(stage, &[grow], 0.4)?;
    fade_in_lagged(stage, &arrows[2..4], 1.0, 0.4)?;

    k.until_sentence(stage, "14_3", 4)?;
    let sizes: Vec<String> = t::series_terms(PI, 7)
        .iter()
        .map(|z| format!("{:.2}", z.abs()))
        .collect();
    let shrink = [
        card(k, stage, "past pi the factorial wins", Palette::teal(), -0.45)?,
        math(
            k,
            stage,
            &format!(r"{},\ \ldots", sizes.join(r",\ ")),
            24.0,
            Palette::dim(),
            -1.05,
        )?,
    ];
    fade_in_all(stage, &shrink, 0.4)?;
    fade_in_lagged(stage, &arrows[4..], 1.6, 0.25)?;

    k.until_sentence(stage, "14_3", 5)?;
    let same = card(k, stage, "the same landing point", Palette::green(), -1.9)?;
    fade_in_all(stage, &[same], 0.4)?;
    k.hold(stage)?;
    let mut leaving = vec![series, next, grow, shrink[0], shrink[1], same];
    leaving.extend(arrows);
    Ok(leaving)
}

/// 14_4: three pictures overlaid, then the notation earns its name and the famous line follows.
fn the_famous_line(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "14_4")?;
    let plane = far();
    fade_out_all(stage, last, 0.4)?;
    let upper_half = plane.arc(0.0, PI);
    let sums: Vec<Vec3> = t::partial_sums(PI, t::TERMS).into_iter().map(|z| plane.p(z)).collect();
    let three = [
        path(stage, upper_half.clone(), Palette::gold(), 5.0),
        polygon(stage, 64, Palette::teal(), 2.5),
        path(stage, sums, Palette::hero(), 2.5),
    ];
    fade_in_all(stage, &three, 0.6)?;

    k.until_sentence(stage, "14_4", 1)?;
    let value_line = math(k, stage, LANDING, 36.0, Palette::green(), 2.35)?;
    fade_in_all(stage, &[value_line], 0.5)?;
    k.until_sentence(stage, "14_4", 2)?;
    let written = math(k, stage, WRITTEN, 33.0, Palette::ink(), 1.55)?;
    fade_in_all(stage, &[written], 0.4)?;
    k.until_sentence(stage, "14_4", 3)?;
    let not_repeated = card(k, stage, "not repeated multiplication", Palette::red(), 0.95)?;
    fade_in_all(stage, &[not_repeated], 0.4)?;
    k.until_sentence(stage, "14_4", 4)?;
    let earned = [
        card(k, stage, "the name of our function at that input", Palette::ink(), 0.45)?,
        card(
            k,
            stage,
            "earned because the multiplication law holds",
            Palette::dim(),
            0.0,
        )?,
    ];
    fade_in_all(stage, &earned, 0.4)?;

    k.until_sentence(stage, "14_4", 6)?;
    let famous = math(k, stage, FAMOUS, 58.0, Palette::hero(), -1.05)?;
    fade_in_all(stage, &[famous], 0.8)?;
    k.until_sentence(stage, "14_4", 7)?;
    let identity_name = card(k, stage, IDENTITY_NAME, Palette::hero(), -1.62)?;
    fade_in_all(stage, &[identity_name], 0.4)?;
    k.until_sentence(stage, "14_4", 8)?;
    let thick = path(stage, upper_half, Palette::gold(), 8.0);
    let half = math(k, stage, HALF_LENGTH, 25.0, Palette::gold(), -2.15)?;
    swap(stage, &[three[0]], &[thick, half], 0.5)?;
    k.until_sentence(stage, "14_4", 9)?;
    let z = t::on_circle(0.9);
    let reversed = [
        between(stage, plane, ZERO, z, Palette::green(), 4.5),
        between(stage, plane, ZERO, z.scale(-1.0), Palette::red(), 4.5),
        card(k, stage, "a half turn reverses every arrow", Palette::red(), -2.8)?,
    ];
    fade_in_all(stage, &reversed, 0.5)?;
    k.hold(stage)?;

    let mut leaving = vec![
        three[1],
        three[2],
        thick,
        value_line,
        written,
        not_repeated,
        earned[0],
        earned[1],
        famous,
        identity_name,
        half,
    ];
    leaving.extend(reversed);
    Ok(leaving)
}

/// 14_5: on the circle the multiplication law adds angles, and the classic formulas fall out of it.
fn the_turn_rule(
    k: &Kit,
    stage: &mut Stage<'_>,
    last: &[Mob],
    leaving: Stay,
    landing: [Mob; 2],
) -> fmn::Result<Vec<Mob>> {
    k.say(stage, "14_5")?;
    fade_out_all(stage, last, 0.4)?;
    fade_out_all(stage, &[leaving.axes, leaving.circle, landing[0], landing[1]], 0.4)?;
    let plane = near();
    let axes = plane.axes(stage)?;
    let circle = plane.unit_circle(stage);
    let heading = title(k, stage, "The circle is a slide rule for turns")?;
    swap(stage, &[leaving.heading], &[heading, axes, circle], 0.5)?;

    k.until_sentence(stage, "14_5", 1)?;
    let (s, u) = (t::on_circle(FIRST_ANGLE), t::on_circle(SECOND_ANGLE));
    let product = s.times(u);
    let arrows = [
        between(stage, plane, ZERO, s, Palette::gold(), 4.5),
        between(stage, plane, ZERO, u, Palette::teal(), 4.5),
        between(stage, plane, ZERO, product, Palette::hero(), 4.5),
    ];
    let law = near_math(k, stage, PRODUCT, 32.0, Palette::ink(), 2.3)?;
    fade_in_all(stage, &arrows, 0.5)?;
    fade_in_all(stage, &[law], 0.5)?;

    k.until_sentence(stage, "14_5", 2)?;
    let arcs = [
        path(stage, plane.arc(0.0, FIRST_ANGLE), Palette::gold(), 7.0),
        path(
            stage,
            plane.arc(FIRST_ANGLE, FIRST_ANGLE + SECOND_ANGLE),
            Palette::teal(),
            7.0,
        ),
    ];
    let adds = k.text(
        stage,
        "multiplying these arrows adds their angles",
        24.0,
        Palette::dim(),
        v(NEAR_CARD_X, 1.5),
    )?;
    fade_in_all(stage, &arcs, 0.6)?;
    fade_in_all(stage, &[adds], 0.4)?;

    k.until_sentence(stage, "14_5", 3)?;
    let sums = [
        near_math(k, stage, COS_SUM, 26.0, Palette::blue(), 0.7)?,
        near_math(k, stage, SIN_SUM, 26.0, Palette::teal(), 0.1)?,
    ];
    fade_in_all(stage, &sums, 0.5)?;
    k.until_sentence(stage, "14_5", 4)?;
    let series = [
        near_math(k, stage, COS_SERIES, 26.0, Palette::blue(), -0.95)?,
        near_math(k, stage, SIN_SERIES, 26.0, Palette::teal(), -1.85)?,
    ];
    fade_in_all(stage, &series, 0.5)?;
    k.hold(stage)?;

    let mut shown = arrows.to_vec();
    shown.extend(arcs);
    shown.extend([law, adds, sums[0], sums[1], series[0], series[1]]);
    Ok(shown)
}

/// 14_6: a rate with both parts stretches and turns at once.
fn both_parts(k: &Kit, stage: &mut Stage<'_>, last: &[Mob]) -> fmn::Result<()> {
    k.say(stage, "14_6")?;
    fade_out_all(stage, last, 0.4)?;
    let plane = near();
    let rate = near_math(k, stage, r"\text{rate}=a+iw", 32.0, Palette::ink(), 2.3)?;
    fade_in_all(stage, &[rate], 0.5)?;

    k.until_sentence(stage, "14_6", 1)?;
    let split = near_math(k, stage, BOTH_PARTS, 30.0, Palette::green(), 1.4)?;
    fade_in_all(stage, &[split], 0.5)?;
    k.until_sentence(stage, "14_6", 2)?;
    let parts = [
        near_math(k, stage, r"\text{length}=E(a\,t)", 27.0, Palette::gold(), 0.6)?,
        near_math(k, stage, r"\text{angle}=w\,t", 27.0, Palette::teal(), 0.05)?,
    ];
    fade_in_all(stage, &parts, 0.4)?;

    k.until_sentence(stage, "14_6", 3)?;
    let chosen = near_math(
        k,
        stage,
        &format!(r"a={:.2},\quad w=1", t::SPIRAL_REAL),
        25.0,
        Palette::dim(),
        -0.65,
    )?;
    fade_in_all(stage, &[chosen], 0.3)?;
    let clock = stage.add_value_tracker(0.0);
    let trail = live(stage, clock, move |s, now| {
        let end = now.max(1e-3);
        let points: Vec<Vec3> = (0..=300)
            .map(|j| plane.p(t::spiral(end * f64::from(j) / 300.0)))
            .collect();
        s.add(polyline(points).style(line_style(Palette::hero(), 3.5)).build())
    })?;
    let tip = live(stage, clock, move |s, now| {
        s.add(
            Dot::new()
                .point(plane.p(t::spiral(now)))
                .radius(0.09)
                .style(flat(Palette::hero(), 1.0))
                .build(),
        )
    })?;
    let radius = live(stage, clock, move |s, now| {
        arrow_parts(s, plane.p(ZERO), plane.p(t::spiral(now)), Palette::gold(), 3.5, 0.15)
    })?;
    walk(stage, clock, t::SPIRAL_END, SPIRAL_SECONDS)?;
    for mob in [trail, tip, radius] {
        stage.clear_updaters(mob, true);
    }

    k.until_sentence(stage, "14_6", 4)?;
    let kinds: [(&str, &str, Srgb); 4] = [
        (r"w=0,\ a<0", "decay", Palette::red()),
        (r"w=0,\ a>0", "growth", Palette::green()),
        (r"a=0,\ w\neq 0", "rotation", Palette::blue()),
        (r"a\neq 0,\ w\neq 0", "spirals", Palette::hero()),
    ];
    for (i, (condition, name, colour)) in kinds.into_iter().enumerate() {
        let y = -1.3 - 0.47 * i as f64;
        let row = [
            line_of_math(k, stage, condition, 25.0, colour, NEAR_CARD_X - 0.9, y)?,
            k.text_left(stage, name, 25.0, colour, v(NEAR_CARD_X + 1.0, y))?,
        ];
        fade_in_all(stage, &row, 0.3)?;
    }
    Ok(())
}

impl SceneConstruct for EulerIdentity {
    fn name(&self) -> &str {
        "14_euler_identity"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (stay, cards, steps) = tangent_walk(&k, stage)?;
        let (landing, cards, walks) = they_land(&k, stage, &cards)?;
        let last = spiral_of_terms(&k, stage, &cards, &walks, &steps)?;
        let last = the_famous_line(&k, stage, &last)?;
        let shown = the_turn_rule(&k, stage, &last, stay, landing)?;
        both_parts(&k, stage, &shown)?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ending_draw::{assert_in_order, scene_source};
    const SOURCE: &str = include_str!("euler_identity.rs");

    use super::{OUTWARD, QUESTION, THINK_14_1};
    use crate::turning_model as t;
    use std::f64::consts::PI;

    #[test]
    fn the_landing_point_waits_for_the_answer() {
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "14_1", 5)?;"#,
                "OUTWARD",
                r#"k.until_sentence(stage, "14_1", 7)?;"#,
                "QUESTION",
                "k.hold(stage)?;",
                r#"k.mark(stage, "predict");"#,
                "k.music_breath(stage, THINK_14_1)?;",
                r#"k.say(stage, "14_2")?;"#,
                "ring(stage, plane.p(MINUS_ONE)",
            ],
        );
        assert_eq!(QUESTION, "do these paths agree?");
        // Review 721 M10 proposed two to three seconds for this pair; audio-review decides the final length.
        const { assert!(THINK_14_1 >= 2.0) };
        const { assert!(THINK_14_1 <= 3.0) };
        // Review 721 W3: every four-cell step ends farther from zero, not only the first.
        assert!(OUTWARD.contains("each step"));
        let points = t::tangent_steps(PI, 4);
        assert!(points.windows(2).all(|p| p[1].abs() > p[0].abs()));
        let before = scene_source(SOURCE).split("fn they_land").next().unwrap();
        assert!(
            !before.contains("MINUS_ONE)"),
            "nothing marks minus one before the question is held"
        );
    }
    #[test]
    fn the_famous_line_comes_after_the_notation_has_earned_it() {
        assert_in_order(
            SOURCE,
            &[
                "LANDING",
                "WRITTEN",
                "not repeated multiplication",
                "the name of our function at that input",
                "FAMOUS",
            ],
        );
    }
    #[test]
    fn the_tangent_step_is_shown_before_it_is_repeated() {
        assert_in_order(
            SOURCE,
            &["points[0].times_i()", "points[1], Palette::hero()", "points[1..]"],
        );
    }
    #[test]
    fn the_angle_sum_is_computed_from_the_product_of_two_points() {
        assert!(scene_source(SOURCE).contains("let product = s.times(u);"));
    }
    #[test]
    fn the_landing_ring_leaves_with_its_name() {
        // Review 721 M7: the ring was never faded and stayed through 14_5 and 14_6.
        assert_in_order(
            SOURCE,
            &[
                "let landing = ring(stage, plane.p(MINUS_ONE)",
                "Ok(([landing, name], leaving, polygons))",
                r#"k.say(stage, "14_5")?;"#,
                "fade_out_all(stage, &[leaving.axes, leaving.circle, landing[0], landing[1]], 0.4)?;",
                r#"k.say(stage, "14_6")?;"#,
            ],
        );
    }
    #[test]
    fn the_larger_cell_counts_wait_for_their_words() {
        // Review 721 L4.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "14_2", 4)?;"#,
                r#"64 => until_phrase(k, stage, "14_2", "sixty-four")?,"#,
                r#"1024 => until_phrase(k, stage, "14_2", "one thousand twenty-four")?,"#,
                "polygon(stage, n, colours[i], widths[i])",
                r#"k.until_sentence(stage, "14_2", 5)?;"#,
            ],
        );
        assert_eq!(t::CELLS, [4, 16, 64, 1024]);
    }
    #[test]
    fn the_identity_is_named_after_the_famous_line() {
        // Review 721 W8.
        assert_in_order(
            SOURCE,
            &[
                r#"k.until_sentence(stage, "14_4", 6)?;"#,
                "FAMOUS",
                r#"k.until_sentence(stage, "14_4", 7)?;"#,
                "IDENTITY_NAME",
                r#"k.until_sentence(stage, "14_4", 8)?;"#,
                "HALF_LENGTH",
                r#"k.until_sentence(stage, "14_4", 9)?;"#,
                "a half turn reverses every arrow",
            ],
        );
    }
}
