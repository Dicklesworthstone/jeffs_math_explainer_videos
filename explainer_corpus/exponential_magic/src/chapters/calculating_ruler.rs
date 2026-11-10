//! Native slide-rule teaching scene. Unrendered author candidate; no visual acceptance.
use crate::phrase::until_phrase;
use crate::slide_model as m;
use explainer_kit::{Kit, Palette, Timed, ink_style, stroke_style, v};
use fmn::prelude::*;
pub struct CalculatingRuler {
    pub kit: Kit,
}
/// The rails sit side by side as on a real rule: ticks face each other across the joint and labels stand
/// outside, so an upper mark is read against the lower mark beneath it.
const UPPER_Y: f64 = 0.30;
const LOWER_Y: f64 = -0.30;
const LABEL_FROM_RAIL: f64 = 0.27;
const RAIL_TEXT: f64 = 22.0;
/// How far a reading line reaches from the joint: past both rails, short of every label.
const CURSOR_REACH: f64 = 0.36;
const ALIGNMENT_NAME_Y: f64 = 0.95;
/// Clear of the strip at the right-index alignment, whose left end reaches x = -5.19.
const SIDE_NAME_X: f64 = -6.1;
/// The distance bars, under the lower labels.
const BAR_Y: f64 = -1.0;
const SECOND_BAR_Y: f64 = -1.3;
const POINTER_FOOT_Y: f64 = -0.82;
const POINTER_TEXT_Y: f64 = -1.02;
/// Just wider than a digit at `RAIL_TEXT`, and short of the rail above the label.
const FIRST_MARK_RADIUS: f64 = 0.2;
fn line(s: &mut Stage<'_>, a: Vec3, b: Vec3, color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(s.arena_mut()
        .add(Line::new(a, b).style(stroke_style(color, width)).build()?))
}
fn show(s: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fade_in(s.arena_mut(), mob, ORIGIN, 1.0)?;
        moves.push(s.prepare(a.rt(0.5))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
fn hide(s: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fade_out(s.arena_mut(), mob, ORIGIN, 1.0)?;
        moves.push(s.prepare(a.rt(0.4))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
fn slide(s: &mut Stage<'_>, mob: Mob, by: Vec3, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .shift(by)?;
    s.prepare(a)
}
fn glide(s: &mut Stage<'_>, mob: Mob, to: Vec3, seconds: f64) -> fmn::Result<Box<dyn Animation>> {
    let a = mob
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(seconds),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?;
    s.prepare(a)
}
fn shift(s: &mut Stage<'_>, mobs: &[Mob], dx: f64, dy: f64) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        moves.push(slide(s, mob, v(dx, dy), 1.4)?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
/// A scale's line and ticks. The sign of `y` says which rail it is: ticks always point at the other rail.
fn rail_strokes(s: &mut Stage<'_>, y: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let toward_joint = -y.signum();
    let mut out = vec![line(s, v(m::LEFT, y), v(m::LEFT + m::SPAN, y), color, 2.5)?];
    for i in 10..=100 {
        let n = f64::from(i) / 10.0;
        let major = i % 10 == 0;
        let foot = y + toward_joint * if major { 0.20 } else { 0.09 };
        out.push(line(
            s,
            v(m::position(n), y),
            v(m::position(n), foot),
            color,
            if major { 2.0 } else { 1.0 },
        )?);
    }
    Ok(out)
}
/// Where a scale's number stands: outside its rail, away from the joint.
fn label_place(value: u32, y: f64) -> Vec3 {
    v(m::position(f64::from(value)), y + y.signum() * LABEL_FROM_RAIL)
}
fn rail_labels(k: &Kit, s: &mut Stage<'_>, y: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for n in m::LABELS {
        out.push(k.text(s, &n.to_string(), RAIL_TEXT, color, label_place(n, y))?);
    }
    Ok(out)
}
fn caption(k: &Kit, s: &mut Stage<'_>, text: &str, color: Srgb) -> fmn::Result<Mob> {
    k.text(s, text, 29.0, color, v(0.0, -2.6))
}
/// The line a value is read along. It stops short of the labels, so it never strikes the digits being read.
fn cursor(s: &mut Stage<'_>, x: f64, color: Srgb) -> fmn::Result<Mob> {
    line(s, v(x, -CURSOR_REACH), v(x, CURSOR_REACH), color, 3.0)
}
/// The line joining the two marks being aligned across the joint, named when one of them is an index.
fn alignment(k: &Kit, s: &mut Stage<'_>, x: f64, name: Option<&str>) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![line(s, v(x, LOWER_Y), v(x, UPPER_Y), Palette::ink(), 2.5)?];
    if let Some(name) = name {
        out.push(k.text(s, name, 21.0, Palette::ink(), v(x, ALIGNMENT_NAME_Y))?);
    }
    Ok(out)
}
/// A temporary mark naming an unlabelled value on the lower scale, for the prediction.
fn pointer(k: &Kit, s: &mut Stage<'_>, value: f64) -> fmn::Result<Vec<Mob>> {
    let x = m::position(value);
    Ok(vec![
        line(s, v(x, LOWER_Y), v(x, POINTER_FOOT_Y), Palette::hero(), 2.0)?,
        k.text(s, &m::label(value), RAIL_TEXT, Palette::hero(), v(x, POINTER_TEXT_Y))?,
    ])
}
/// A copy of every source mark travels to its target, so the strip is visibly the same scale.
fn copy_across(s: &mut Stage<'_>, from: &[Mob], to: &[Mob], seconds: f64) -> fmn::Result<()> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    for (&source, &target) in from.iter().zip(to) {
        let copy = fmn::animation::transform_from_copy(s.arena_mut(), source, target)?;
        moves.push(s.prepare(copy.rt(seconds))?);
    }
    s.play_prepared(moves)?;
    Ok(())
}
/// One sampled height: the horizontal distance that produces it, its point on the curve and its number.
struct Sample {
    value: u32,
    at: Vec3,
    /// Height one is produced by distance zero, so it has no segment to draw.
    distance: Option<Mob>,
    dot: Mob,
    height: Mob,
}
fn sample(k: &Kit, s: &mut Stage<'_>, value: u32, at: Vec3) -> fmn::Result<Sample> {
    let distance = if value == 1 {
        None
    } else {
        Some(line(s, v(m::LEFT, at[1]), at, Palette::blue(), 4.0)?)
    };
    let dot = s.arena_mut().add(
        Circle::new()
            .radius(0.055)
            .style(ink_style(Palette::gold()))
            .build()
            .moved_to(at),
    );
    // Up and to the left of its point, the side of a rising curve that stays empty, and midway between this
    // height's distance and the next one's.
    let height = k.text(
        s,
        &value.to_string(),
        RAIL_TEXT,
        Palette::gold(),
        v(at[0] - 0.22, at[1] + 0.155),
    )?;
    Ok(Sample {
        value,
        at,
        distance,
        dot,
        height,
    })
}
/// The distance grows out of the height axis until it meets the curve, and the height's number appears there.
fn reveal(s: &mut Stage<'_>, sample: &Sample, seconds: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut moves: Vec<Box<dyn Animation>> = Vec::new();
    if let Some(distance) = sample.distance {
        let grow = fmn::animation::grow_from_edge(s.arena_mut(), distance, LEFT, None)?;
        moves.push(s.prepare(grow.rt(seconds))?);
    }
    for mob in [sample.dot, sample.height] {
        let a = fade_in(s.arena_mut(), mob, ORIGIN, 1.0)?;
        moves.push(s.prepare(a.rt(seconds))?);
    }
    Ok(moves)
}
impl SceneConstruct for CalculatingRuler {
    fn name(&self) -> &str {
        "07_calculating_ruler"
    }
    fn construct(&mut self, s: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();
        let (a, b) = m::EXAMPLES[0];
        k.say(s, "07_1")?;
        let title = k.text(
            s,
            "Turn the growth law into a ruler",
            35.0,
            Palette::ink(),
            v(0.0, 3.35),
        )?;
        show(s, &[title])?;
        let mut graph = vec![
            line(s, v(m::LEFT, -1.7), v(m::LEFT + m::SPAN, -1.7), Palette::dim(), 2.0)?,
            line(s, v(m::LEFT, -1.7), v(m::LEFT, 1.3), Palette::dim(), 2.0)?,
        ];
        let point = |t: f64| v(m::LEFT + m::SPAN * t, -1.7 + 2.8 * (10.0_f64.powf(t) - 1.0) / 9.0);
        for i in 0..60 {
            graph.push(line(
                s,
                point(f64::from(i) / 60.0),
                point(f64::from(i + 1) / 60.0),
                Palette::gold(),
                3.0,
            )?);
        }
        graph.push(k.text(s, "input distance", 22.0, Palette::blue(), v(0.0, -2.1))?);
        show(s, &graph)?;
        let mut samples = Vec::new();
        for n in m::LABELS {
            samples.push(sample(&k, s, n, point(f64::from(n).log10()))?);
        }
        // Each question gets its own answer, on the sentence that asks it.
        k.until_sentence(s, "07_1", 3)?;
        let two = reveal(s, &samples[1], 0.7)?;
        s.play_prepared(two)?;
        k.until_sentence(s, "07_1", 4)?;
        let three = reveal(s, &samples[2], 0.7)?;
        s.play_prepared(three)?;
        k.say(s, "07_2")?;
        for rest in samples.iter().take(1).chain(&samples[3..]) {
            let shown = reveal(s, rest, 0.35)?;
            s.play_prepared(shown)?;
        }
        // Each distance keeps its identity on the way down: it lands on the rail, its end is the mark, and its
        // number becomes that mark's label.
        k.until_sentence(s, "07_2", 1)?;
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for each in &samples {
            let down = v(0.0, LOWER_Y - each.at[1]);
            if let Some(distance) = each.distance {
                moves.push(slide(s, distance, down, 1.2)?);
            }
            moves.push(slide(s, each.dot, down, 1.2)?);
            moves.push(glide(s, each.height, label_place(each.value, LOWER_Y), 1.2)?);
        }
        s.play_prepared(moves)?;
        hide(s, &graph)?;
        let fixed = rail_strokes(s, LOWER_Y, Palette::gold())?;
        show(s, &fixed)?;
        let mut spent: Vec<Mob> = samples.iter().map(|x| x.dot).collect();
        spent.extend(samples.iter().filter_map(|x| x.distance));
        hide(s, &spent)?;
        let fixed_labels: Vec<Mob> = samples.iter().map(|x| x.height).collect();
        // Height one has no distance segment, so a ring marks it while the voice places it at the beginning.
        k.until_sentence(s, "07_2", 2)?;
        let first_mark = s.arena_mut().add(
            Circle::new()
                .radius(FIRST_MARK_RADIUS)
                .style(stroke_style(Palette::hero(), 2.5))
                .build()
                .moved_to(label_place(m::LABELS[0], LOWER_Y)),
        );
        show(s, &[first_mark])?;
        k.until_sentence(s, "07_2", 3)?;
        hide(s, &[first_mark])?;
        k.until_sentence(s, "07_2", 4)?;
        let ratio = caption(&k, s, "Equal distances represent equal factors", Palette::ink())?;
        show(s, &[ratio])?;
        k.say(s, "07_3")?;
        hide(s, &[ratio])?;
        let law = k.tex(
            s,
            r"\log_{10}(ab)=\log_{10}(a)+\log_{10}(b)",
            31.0,
            Palette::ink(),
            &[],
            v(0.0, 2.0),
        )?;
        k.until_sentence(s, "07_3", 2)?;
        show(s, &[law])?;
        k.until_sentence(s, "07_3", 4)?;
        let d1 = line(
            s,
            v(m::position(1.0), BAR_Y),
            v(m::position(a), BAR_Y),
            Palette::blue(),
            6.0,
        )?;
        // Measured from one, as 07_2 defined a number's distance, and only then moved to follow the first.
        let d2 = line(
            s,
            v(m::position(1.0), SECOND_BAR_Y),
            v(m::position(b), SECOND_BAR_Y),
            Palette::green(),
            6.0,
        )?;
        show(s, &[d1])?;
        until_phrase(&k, s, "07_3", "the distance for three")?;
        show(s, &[d2])?;
        until_phrase(&k, s, "07_3", "is the distance for six")?;
        let joined_on = slide(s, d2, v(m::distance(a), BAR_Y - SECOND_BAR_Y), 1.0)?;
        s.play_prepared(vec![joined_on])?;
        k.say(s, "07_4")?;
        hide(s, &[d1, d2, law])?;
        // The strip is built from the fixed scale: every mark and number travels across the joint.
        let mut moving = rail_strokes(s, UPPER_Y, Palette::blue())?;
        moving.extend(rail_labels(&k, s, UPPER_Y, Palette::blue())?);
        let mut sources = fixed.clone();
        sources.extend(fixed_labels.iter().copied());
        copy_across(s, &sources, &moving, 1.2)?;
        let names = [
            k.text(s, "upper: sliding", 21.0, Palette::blue(), v(SIDE_NAME_X, UPPER_Y))?,
            k.text(s, "lower: fixed", 21.0, Palette::gold(), v(SIDE_NAME_X, LOWER_Y))?,
        ];
        show(s, &names)?;
        // Both end marks are named where they stand, on the sentence that defines the word.
        k.until_sentence(s, "07_4", 1)?;
        let mut index_names = Vec::new();
        for value in [m::LABELS[0], m::LABELS[m::LABELS.len() - 1]] {
            let x = m::position(f64::from(value));
            index_names.push(k.text(s, "index", 21.0, Palette::ink(), v(x, ALIGNMENT_NAME_Y))?);
        }
        show(s, &index_names)?;
        k.until_sentence(s, "07_4", 2)?;
        hide(s, &index_names)?;
        let mut at = m::offset(a, false);
        shift(s, &moving, at, 0.0)?;
        let mut reading = alignment(&k, s, m::position(a), Some("left index"))?;
        show(s, &reading)?;
        k.until_sentence(s, "07_4", 4)?;
        let hair = cursor(s, m::position(a * b), Palette::green())?;
        let answer = caption(&k, s, &m::equation(a, b), Palette::green())?;
        show(s, &[hair, answer])?;
        reading.extend([hair, answer]);
        // The strip goes home after this line, so the next one can truly say nothing has moved yet.
        k.hold(s)?;
        hide(s, &reading)?;
        shift(s, &moving, -at, 0.0)?;
        at = 0.0;
        k.say(s, "07_5")?;
        let (p, q) = m::EXAMPLES[1];
        k.until_sentence(s, "07_5", 1)?;
        let mut prediction = pointer(&k, s, p)?;
        prediction.push(caption(
            &k,
            s,
            &format!("{} × {} = ?", m::label(p), m::label(q)),
            Palette::hero(),
        )?);
        show(s, &prediction)?;
        k.hold(s)?;
        k.mark(s, "predict");
        k.music_breath(s, 4.0)?;
        k.say(s, "07_6")?;
        hide(s, &prediction)?;
        let new = m::offset(p, false);
        shift(s, &moving, new - at, 0.0)?;
        at = new;
        let mut reading = alignment(&k, s, m::position(p), Some("left index"))?;
        let hair = cursor(s, m::position(p * q), Palette::green())?;
        let answer = caption(&k, s, &m::equation(p, q), Palette::green())?;
        reading.extend([hair, answer]);
        show(s, &reading)?;
        k.until_sentence(s, "07_6", 3)?;
        hide(s, &reading)?;
        let (p, q) = m::EXAMPLES[2];
        let new = m::offset(p, false);
        shift(s, &moving, new - at, 0.0)?;
        at = new;
        let mut reading = alignment(&k, s, m::position(p), Some("left index"))?;
        let outside = cursor(s, m::position(p * q), Palette::hero())?;
        let warning = caption(
            &k,
            s,
            "Beyond the lower scale: there is no marking to read",
            Palette::hero(),
        )?;
        reading.extend([outside, warning]);
        show(s, &reading)?;
        k.say(s, "07_7")?;
        hide(s, &reading)?;
        let new = m::offset(p, true);
        shift(s, &moving, new - at, 0.0)?;
        at = new;
        let mut joined = alignment(&k, s, m::position(p), Some("right index"))?;
        show(s, &joined)?;
        k.until_sentence(s, "07_7", 3)?;
        let value = m::reading(p, q, true);
        let hair = cursor(s, m::position(value), Palette::green())?;
        let read = caption(&k, s, &format!("Read {}", m::label(value)), Palette::green())?;
        // Lower 1.2 is an unlabelled minor tick, so the mark being read is named.
        let marked = pointer(&k, s, value)?;
        show(s, &[hair, read, marked[0], marked[1]])?;
        // The answer waits for the sentence that says it.
        k.until_sentence(s, "07_7", 4)?;
        let restored = caption(
            &k,
            s,
            &format!(
                "Read {}   →   restore the decimal place   →   {}",
                m::label(value),
                m::label(p * q)
            ),
            Palette::green(),
        )?;
        hide(s, &[read])?;
        show(s, &[restored])?;
        joined.extend([hair, restored]);
        joined.extend(marked);
        k.say(s, "07_8")?;
        hide(s, &joined)?;
        k.until_sentence(s, "07_8", 2)?;
        let first = line(
            s,
            v(m::position(1.0), BAR_Y),
            v(m::position(2.0), BAR_Y),
            Palette::green(),
            6.0,
        )?;
        let second = line(
            s,
            v(m::position(2.0), SECOND_BAR_Y),
            v(m::position(4.0), SECOND_BAR_Y),
            Palette::green(),
            6.0,
        )?;
        show(s, &[first, second])?;
        k.say(s, "07_9")?;
        hide(s, &[first, second])?;
        // The strip moves when the voice gives the instruction.
        k.until_sentence(s, "07_9", 1)?;
        let new = m::distance(a * b) - m::distance(b);
        shift(s, &moving, new - at, 0.0)?;
        let mut joined = alignment(&k, s, m::position(a * b), None)?;
        show(s, &joined)?;
        k.until_sentence(s, "07_9", 2)?;
        let hair = cursor(s, m::position(a), Palette::green())?;
        let divide = caption(
            &k,
            s,
            &format!("{} ÷ {} = {}", m::label(a * b), m::label(b), m::label(a)),
            Palette::green(),
        )?;
        show(s, &[hair, divide])?;
        joined.extend([hair, divide]);
        k.say(s, "07_10")?;
        hide(s, &joined)?;
        let limit = caption(&k, s, "Finite marks → finite precision", Palette::hero())?;
        show(s, &[limit])?;
        k.until_sentence(s, "07_10", 4)?;
        hide(s, &[limit])?;
        let recap = caption(
            &k,
            s,
            "Exponential law → logarithmic distances → calculation",
            Palette::ink(),
        )?;
        show(s, &[recap])?;
        k.finish(s)
    }
}
