//! Chapter 03: why transitivity is the property that makes classes. The
//! closeness rule hops along a number line until a chain of close steps
//! reaches anything; the twelve-lane grid of integers shows a class as a
//! whole lane; two class bubbles that share one member are linked through it
//! and merge; the closeness windows around one and zero overlap without
//! coinciding; lanes cut the integers into pieces and, conversely, pieces
//! define "same piece"; fractions become classes on lines through the
//! origin; and the clock's residues collapse into four new objects, the
//! preview of cosets and quotients. Every member drawn is computed here.
use crate::d4_group as d4g;
use explainer_kit::{Kit, Palette, Timed, ink_style, play, stroke_style, v};
use fmn::prelude::*;

pub struct Counterexample {
    pub kit: Kit,
}

/// Integers on the closeness number line.
const LINE: std::ops::RangeInclusive<i32> = -4..=6;
const LINE_Y: f64 = -0.2;
/// Integers in the twelve-lane grid: four full rows.
const GRID: std::ops::RangeInclusive<i32> = -12..=35;

fn close(a: i32, b: i32) -> bool {
    (a - b).abs() <= 1
}

/// The members of `LINE` close to `center`.
pub(crate) fn close_to(center: i32) -> Vec<i32> {
    LINE.filter(|&n| close(n, center)).collect()
}

fn same_lane(a: i32, b: i32) -> bool {
    (a - b).rem_euclid(12) == 0
}

/// The members of `GRID` in the lane of `n`, top to bottom.
fn lane_of(n: i32) -> Vec<i32> {
    GRID.filter(|&m| same_lane(m, n)).collect()
}

/// Text for an integer with a true minus sign (the glyph set uses U+2212).
fn signed(n: i32) -> String {
    if n < 0 { format!("−{}", -n) } else { n.to_string() }
}

fn line_x(n: i32) -> f64 {
    1.15 * f64::from(n - 1)
}

fn cell_at(n: i32) -> (f64, f64) {
    let row = (n - *GRID.start()).div_euclid(12);
    let column = n.rem_euclid(12);
    (-6.05 + 1.1 * f64::from(column), 1.6 - 0.8 * f64::from(row))
}

/// A curved link from `from` to `to`, bulging upward for positive `bend`.
fn link(stage: &mut Stage<'_>, from: (f64, f64), to: (f64, f64), bend: f64, color: Srgb) -> fmn::Result<Mob> {
    let (start, end) = if from.0 <= to.0 { (to, from) } else { (from, to) };
    let arc = ArcBetweenPoints::new(v(start.0, start.1), v(end.0, end.1))
        .angle(bend)
        .style(stroke_style(color, 4.0))
        .build()?;
    Ok(stage.arena_mut().add(arc))
}

/// A hop between two integers on the number line.
fn hop(stage: &mut Stage<'_>, a: i32, b: i32, color: Srgb) -> fmn::Result<Mob> {
    link(
        stage,
        (line_x(a), LINE_Y + 0.25),
        (line_x(b), LINE_Y + 0.25),
        0.55 * std::f64::consts::PI,
        color,
    )
}

fn draw(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        animations.push(stage.prepare(show_creation(mob).rt(run_time))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn frame(stage: &mut Stage<'_>, center: (f64, f64), size: (f64, f64), color: Srgb) -> fmn::Result<Mob> {
    let shape = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.22)
        .style(stroke_style(color, 4.0))
        .build()?
        .moved_to(v(center.0, center.1));
    Ok(stage.arena_mut().add(shape))
}

/// A frame around the integers `members` of the number line, raised by `lift`.
fn window(stage: &mut Stage<'_>, members: &[i32], lift: f64, color: Srgb) -> fmn::Result<Mob> {
    let (lo, hi) = (members[0], members[members.len() - 1]);
    let center = (0.5 * (line_x(lo) + line_x(hi)), LINE_Y - 0.3 + lift);
    frame(stage, center, (line_x(hi) - line_x(lo) + 0.9, 1.25), color)
}

/// A measuring bar with end ticks from `x0` to `x1` at height `y`.
fn measure(stage: &mut Stage<'_>, x0: f64, x1: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    let mut parts = Vec::new();
    for (from, to) in [
        (v(x0, y), v(x1, y)),
        (v(x0, y - 0.15), v(x0, y + 0.15)),
        (v(x1, y - 0.15), v(x1, y + 0.15)),
    ] {
        parts.push(
            stage
                .arena_mut()
                .add(Line::new(from, to).style(stroke_style(color, 4.0)).build()?),
        );
    }
    d4g::group_of(stage, &parts)
}

fn bubble(stage: &mut Stage<'_>, at: (f64, f64), radius: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(stroke_style(color, 4.0))
            .build()
            .moved_to(v(at.0, at.1)),
    )
}

fn dot(stage: &mut Stage<'_>, at: (f64, f64), radius: f64, color: Srgb) -> Mob {
    stage.arena_mut().add(
        Circle::new()
            .radius(radius)
            .style(ink_style(color))
            .build()
            .moved_to(v(at.0, at.1)),
    )
}

fn pulse(stage: &mut Stage<'_>, mobs: &[Mob], scale: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), mob, scale, None)?.rt(0.8);
        animations.push(stage.prepare(a)?);
    }
    let group = explainer_kit::lagged(stage, animations, 0.15)?;
    play!(stage; group);
    Ok(())
}

type IntegerDiagram = (Vec<Mob>, Vec<(i32, Mob)>);

/// The number line with ticks and labels; returns (all parts, label per integer).
fn number_line(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<IntegerDiagram> {
    let axis = stage.arena_mut().add(
        Line::new(
            v(line_x(*LINE.start()) - 0.6, LINE_Y),
            v(line_x(*LINE.end()) + 0.6, LINE_Y),
        )
        .style(stroke_style(Palette::dim(), 3.0))
        .build()?,
    );
    let mut parts = vec![axis];
    let mut labels = Vec::new();
    for n in LINE {
        let x = line_x(n);
        parts.push(
            stage.arena_mut().add(
                Line::new(v(x, LINE_Y - 0.12), v(x, LINE_Y + 0.12))
                    .style(stroke_style(Palette::dim(), 3.0))
                    .build()?,
            ),
        );
        let label = kit.text(stage, &signed(n), 28.0, Palette::ink(), v(x, LINE_Y - 0.55))?;
        parts.push(label);
        labels.push((n, label));
    }
    Ok((parts, labels))
}

fn label_of(labels: &[(i32, Mob)], n: i32) -> Mob {
    labels.iter().find(|l| l.0 == n).expect("integer is on the line").1
}

/// The twelve-lane grid; returns (all parts, cell per integer).
fn lane_grid(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<IntegerDiagram> {
    let mut parts = Vec::new();
    let mut cells = Vec::new();
    for lane in 0..12 {
        let (x, _) = cell_at(lane);
        parts.push(kit.text(stage, &signed(lane), 24.0, Palette::dim(), v(x, 2.45))?);
    }
    for n in GRID {
        let (x, y) = cell_at(n);
        let cell = kit.text(stage, &signed(n), 26.0, Palette::ink(), v(x, y))?;
        parts.push(cell);
        cells.push((n, cell));
    }
    // The lanes continue both ways.
    for y in [2.05, -1.35] {
        parts.push(kit.tex(stage, r"\vdots", 30.0, Palette::dim(), &[], v(0.0, y))?);
    }
    Ok((parts, cells))
}

impl SceneConstruct for Counterexample {
    fn name(&self) -> &str {
        "03_counterexample"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let [a, b, c] = crate::COUNTEREXAMPLE;

        // 03_1: the closeness rule on a number line.
        kit.say(stage, "03_1")?;
        let title = kit.text(stage, "Close means at most 1 apart", 36.0, Palette::ink(), v(0.0, 3.2))?;
        let (line_parts, _) = number_line(stage, &kit)?;
        let mut shown = vec![title];
        shown.extend(line_parts.iter().copied());
        d4g::show(stage, &shown, 0.5)?;
        kit.until_sentence(stage, "03_1", 1)?;
        // A window of width two slides along: whatever it covers is close to its centre.
        let sweep = window(stage, &close_to(4), 0.0, Palette::gold())?;
        d4g::show(stage, &[sweep], 0.3)?;
        d4g::glide(stage, sweep, (0.5 * (line_x(-1) + line_x(1)), LINE_Y - 0.3), 1.6)?;
        d4g::hide(stage, &[sweep], 0.3)?;
        kit.until_sentence(stage, "03_1", 2)?;
        let first = hop(stage, a, b, Palette::green())?;
        draw(stage, &[first], 0.6)?;
        let second = hop(stage, b, c, Palette::green())?;
        draw(stage, &[second], 0.6)?;

        // 03_2: the question, as a missing hop.
        kit.say(stage, "03_2")?;
        kit.until_sentence(stage, "03_2", 1)?;
        let wanted = link(
            stage,
            (line_x(a), LINE_Y + 0.25),
            (line_x(c), LINE_Y + 0.25),
            0.7 * std::f64::consts::PI,
            Palette::faint(),
        )?;
        draw(stage, &[wanted], 0.8)?;
        let question = kit.text(stage, "Must 0 be close to 2?", 34.0, Palette::hero(), v(0.0, 2.3))?;
        d4g::show(stage, &[question], 0.4)?;
        kit.hold(stage)?;
        stage.wait(3.0)?;

        // 03_3: two apart; and a chain of close steps reaches anything.
        kit.say(stage, "03_3")?;
        kit.until_sentence(stage, "03_3", 1)?;
        let gap = measure(stage, line_x(a), line_x(c), LINE_Y - 1.0, Palette::gold())?;
        let answer = kit.text(
            stage,
            "0 and 2 are 2 apart",
            30.0,
            Palette::gold(),
            v(line_x(b), LINE_Y - 1.6),
        )?;
        d4g::show(stage, &[gap, answer], 0.4)?;
        let wanted_gold = link(
            stage,
            (line_x(a), LINE_Y + 0.25),
            (line_x(c), LINE_Y + 0.25),
            0.7 * std::f64::consts::PI,
            Palette::gold(),
        )?;
        play!(stage; fmn::animation::replacement_transform(wanted, wanted_gold).rt(0.5));
        kit.until_sentence(stage, "03_3", 2)?;
        let verdict = kit.text(stage, "Not transitive", 32.0, Palette::gold(), v(0.0, -2.75))?;
        d4g::show(stage, &[verdict], 0.4)?;
        kit.until_sentence(stage, "03_3", 3)?;
        let mut chain = vec![first, second];
        for n in c..*LINE.end() {
            let step = hop(stage, n, n + 1, Palette::green())?;
            draw(stage, &[step], 0.35)?;
            chain.push(step);
        }
        let reach = link(
            stage,
            (line_x(a), LINE_Y + 0.25),
            (line_x(*LINE.end()), LINE_Y + 0.25),
            0.35 * std::f64::consts::PI,
            Palette::hero(),
        )?;
        draw(stage, &[reach], 0.8)?;
        let lumped = kit.text(
            stage,
            "Chained, every integer would land in one lump",
            28.0,
            Palette::hero(),
            v(0.0, 2.3),
        )?;
        d4g::hide(stage, &[question], 0.3)?;
        d4g::show(stage, &[lumped], 0.4)?;

        // 03_4: a class is a whole lane of the twelve-lane grid.
        kit.say(stage, "03_4")?;
        let mut old = line_parts.clone();
        old.extend(chain);
        old.extend([reach, wanted_gold, gap, answer, verdict, lumped, title]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(stage, "Same clock lane: one class", 34.0, Palette::ink(), v(0.0, 3.3))?;
        let (grid, cells) = lane_grid(stage, &kit)?;
        let mut shown = vec![heading];
        shown.extend(grid.iter().copied());
        d4g::show(stage, &shown, 0.6)?;
        kit.until_sentence(stage, "03_4", 1)?;
        let seed = bubble(stage, cell_at(crate::LANE_FOUR[0]), 0.36, Palette::blue());
        draw(stage, &[seed], 0.5)?;
        kit.until_sentence(stage, "03_4", 2)?;
        let class_four = lane_of(crate::LANE_FOUR[0]);
        let lane_x = cell_at(crate::LANE_FOUR[0]).0;
        let lane_frame = frame(stage, (lane_x, 0.4), (0.95, 3.35), Palette::blue())?;
        draw(stage, &[lane_frame], 0.6)?;
        let members: Vec<Mob> = cells
            .iter()
            .filter(|cell| class_four.contains(&cell.0))
            .map(|cell| cell.1)
            .collect();
        let mut recolor: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &members {
            recolor.push(stage.prepare(m.animate().set_color(Palette::blue())?)?);
        }
        stage.play_prepared(recolor)?;
        pulse(stage, &members, 1.25)?;

        // 03_5: two classes that share one member must coincide.
        kit.say(stage, "03_5")?;
        let mut old = grid.clone();
        old.extend([heading, seed, lane_frame]);
        d4g::hide(stage, &old, 0.5)?;
        let left_c = (-1.05, 0.3);
        let right_c = (1.05, 0.3);
        let class_a = bubble(stage, left_c, 1.7, Palette::blue());
        let class_b = bubble(stage, right_c, 1.7, Palette::gold());
        let name_a = kit.text(stage, "class of a", 26.0, Palette::blue(), v(-2.3, 2.35))?;
        let name_b = kit.text(stage, "class of b", 26.0, Palette::gold(), v(2.3, 2.35))?;
        let pa = (-2.0, 0.3);
        let pb = (2.0, 0.3);
        let pc = (0.0, 0.3);
        let dots = [
            dot(stage, pa, 0.13, Palette::blue()),
            dot(stage, pb, 0.13, Palette::gold()),
            dot(stage, pc, 0.13, Palette::hero()),
        ];
        let tags = [
            kit.tex(stage, "a", 34.0, Palette::blue(), &[], v(pa.0, pa.1 - 0.45))?,
            kit.tex(stage, "b", 34.0, Palette::gold(), &[], v(pb.0, pb.1 - 0.45))?,
            kit.tex(stage, "c", 34.0, Palette::hero(), &[], v(pc.0, pc.1 - 0.45))?,
        ];
        let mut scene5 = vec![class_a, class_b];
        scene5.extend(dots);
        scene5.extend(tags);
        d4g::show(stage, &scene5, 0.5)?;
        d4g::show(stage, &[name_a, name_b], 0.4)?;
        pulse(stage, &[dots[2]], 1.6)?;
        kit.until_sentence(stage, "03_5", 1)?;
        let ac = link(stage, pa, pc, 0.45 * std::f64::consts::PI, Palette::ink())?;
        draw(stage, &[ac], 0.5)?;
        let cb = link(stage, pc, pb, 0.45 * std::f64::consts::PI, Palette::ink())?;
        draw(stage, &[cb], 0.5)?;
        kit.until_sentence(stage, "03_5", 2)?;
        let ab = link(stage, pa, pb, -0.5 * std::f64::consts::PI, Palette::green())?;
        draw(stage, &[ab], 0.8)?;
        let joined = kit.text(
            stage,
            "symmetric, then transitive: a matches b",
            26.0,
            Palette::green(),
            v(0.0, -2.35),
        )?;
        d4g::show(stage, &[joined], 0.4)?;
        kit.until_sentence(stage, "03_5", 3)?;
        d4g::glide_all(stage, &[(class_a, (0.0, 0.3)), (class_b, (0.0, 0.3))], 1.4)?;
        let same = kit.text(stage, "one class", 28.0, Palette::green(), v(0.0, 2.35))?;
        play!(stage; fmn::animation::replacement_transform(name_a, same).rt(0.6));
        d4g::hide(stage, &[name_b], 0.3)?;
        kit.until_sentence(stage, "03_5", 4)?;
        let apart_l = bubble(stage, (-4.9, 0.3), 0.9, Palette::purple());
        let apart_r = bubble(stage, (4.9, 0.3), 0.9, Palette::teal());
        let rule = kit.text(
            stage,
            "Coincide completely, or never touch",
            28.0,
            Palette::ink(),
            v(0.0, -3.1),
        )?;
        d4g::show(stage, &[apart_l, apart_r, rule], 0.5)?;

        // 03_6: closeness windows overlap without coinciding.
        kit.say(stage, "03_6")?;
        let mut old = scene5.clone();
        old.extend([ac, cb, ab, joined, same, apart_l, apart_r, rule]);
        d4g::hide(stage, &old, 0.5)?;
        let (line_parts, labels) = number_line(stage, &kit)?;
        let heading = kit.text(
            stage,
            "Closeness: groups that overlap",
            34.0,
            Palette::ink(),
            v(0.0, 3.2),
        )?;
        let mut shown = vec![heading];
        shown.extend(line_parts.iter().copied());
        d4g::show(stage, &shown, 0.5)?;
        kit.until_sentence(stage, "03_6", 1)?;
        let near_one = close_to(b);
        let one_window = window(stage, &near_one, 0.0, Palette::gold())?;
        draw(stage, &[one_window], 0.6)?;
        let one_name = kit.text(stage, "close to 1", 26.0, Palette::gold(), v(line_x(b), LINE_Y + 1.0))?;
        d4g::show(stage, &[one_name], 0.3)?;
        kit.until_sentence(stage, "03_6", 2)?;
        let near_zero = close_to(a);
        let zero_window = window(stage, &near_zero, -0.22, Palette::teal())?;
        draw(stage, &[zero_window], 0.6)?;
        let zero_name = kit.text(stage, "close to 0", 26.0, Palette::teal(), v(line_x(a), LINE_Y - 1.55))?;
        d4g::show(stage, &[zero_name], 0.3)?;
        kit.until_sentence(stage, "03_6", 3)?;
        let shared: Vec<Mob> = near_one
            .iter()
            .filter(|n| near_zero.contains(n))
            .map(|&n| label_of(&labels, n))
            .collect();
        pulse(stage, &shared, 1.4)?;
        let unshared: Vec<Mob> = near_one
            .iter()
            .chain(near_zero.iter())
            .filter(|n| !(near_one.contains(n) && near_zero.contains(n)))
            .map(|&n| label_of(&labels, n))
            .collect();
        let mut mark: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &unshared {
            mark.push(stage.prepare(m.animate().set_color(Palette::hero())?)?);
        }
        stage.play_prepared(mark)?;
        kit.until_sentence(stage, "03_6", 4)?;
        let overlap = kit.text(
            stage,
            "Overlap without coinciding",
            30.0,
            Palette::hero(),
            v(0.0, -2.75),
        )?;
        d4g::show(stage, &[overlap], 0.4)?;

        // 03_7: the lanes are a partition; and every partition is a relation.
        kit.say(stage, "03_7")?;
        let mut old = line_parts;
        old.extend([heading, one_window, one_name, zero_window, zero_name, overlap]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(
            stage,
            "Classes cut the integers into lanes",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        let (grid, _cells) = lane_grid(stage, &kit)?;
        let mut shown = vec![heading];
        shown.extend(grid.iter().copied());
        d4g::show(stage, &shown, 0.5)?;
        kit.until_sentence(stage, "03_7", 1)?;
        let colors = explainer_kit::point_colors(12);
        let mut lanes: Vec<Box<dyn Animation>> = Vec::new();
        let mut lane_frames = Vec::new();
        for lane in 0..12 {
            let f = frame(stage, (cell_at(lane).0, 0.4), (0.95, 3.35), colors[lane as usize])?;
            lane_frames.push(f);
            lanes.push(stage.prepare(show_creation(f).rt(0.5))?);
        }
        let lanes = explainer_kit::lagged(stage, lanes, 0.12)?;
        play!(stage; lanes);
        kit.until_sentence(stage, "03_7", 2)?;
        let mut old = grid;
        old.extend(lane_frames);
        old.push(heading);
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(
            stage,
            "Any cut into pieces gives a relation",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        d4g::show(stage, &[heading], 0.4)?;
        // Eight objects, cut into pieces of three, two and three.
        let pieces: [&[usize]; 3] = [&[0, 1, 2], &[3, 4], &[5, 6, 7]];
        let spot = |k: usize| (-4.9 + 1.4 * k as f64, 0.4);
        let piece_colors = [Palette::blue(), Palette::gold(), Palette::green()];
        let mut objects = Vec::new();
        for k in 0..8 {
            objects.push(dot(stage, spot(k), 0.2, Palette::ink()));
        }
        d4g::show(stage, &objects, 0.4)?;
        kit.until_sentence(stage, "03_7", 3)?;
        let mut piece_frames = Vec::new();
        for (piece, &color) in pieces.iter().zip(&piece_colors) {
            let (lo, hi) = (spot(piece[0]).0, spot(piece[piece.len() - 1]).0);
            piece_frames.push(frame(stage, (0.5 * (lo + hi), 0.4), (hi - lo + 1.0, 1.0), color)?);
        }
        draw(stage, &piece_frames, 0.6)?;
        let mut links = Vec::new();
        for (piece, &color) in pieces.iter().zip(&piece_colors) {
            for (i, &p) in piece.iter().enumerate() {
                for &q in &piece[i + 1..] {
                    links.push(link(stage, spot(p), spot(q), 0.6 * std::f64::consts::PI, color)?);
                }
            }
        }
        draw(stage, &links, 0.6)?;
        let relation = kit.text(
            stage,
            "Related means: in the same piece",
            28.0,
            Palette::ink(),
            v(0.0, -1.6),
        )?;
        d4g::show(stage, &[relation], 0.4)?;

        // 03_8: fractions are classes of pairs, lying on lines through the origin.
        kit.say(stage, "03_8")?;
        let mut old = objects;
        old.extend(piece_frames);
        old.extend(links);
        old.extend([relation, heading]);
        d4g::hide(stage, &old, 0.5)?;
        let origin = (-5.6, -2.7);
        let unit = 1.15;
        let at = |b: i32, a: i32| (origin.0 + unit * f64::from(b), origin.1 + unit * f64::from(a));
        let heading = kit.text(
            stage,
            "Fractions: pairs that name the same number",
            32.0,
            Palette::ink(),
            v(0.0, 3.35),
        )?;
        let horizontal = stage.arena_mut().add(
            Line::new(v(origin.0, origin.1), v(at(8, 0).0 + 0.5, origin.1))
                .style(stroke_style(Palette::dim(), 3.0))
                .build()?,
        );
        let vertical = stage.arena_mut().add(
            Line::new(v(origin.0, origin.1), v(origin.0, at(0, 4).1 + 0.5))
                .style(stroke_style(Palette::dim(), 3.0))
                .build()?,
        );
        let b_name = kit.text(stage, "bottom b", 24.0, Palette::dim(), v(at(8, 0).0, origin.1 - 0.45))?;
        let a_name = kit.text(
            stage,
            "top a",
            24.0,
            Palette::dim(),
            v(origin.0 + 0.6, at(0, 4).1 + 0.75),
        )?;
        let mut lattice = vec![heading, horizontal, vertical, b_name, a_name];
        for (bottom, top) in fraction_lattice() {
            lattice.push(dot(stage, at(bottom, top), 0.08, Palette::faint()));
        }
        d4g::show(stage, &lattice, 0.5)?;
        kit.until_sentence(stage, "03_8", 1)?;
        let half = kit.tex(
            stage,
            r"\tfrac12",
            34.0,
            Palette::blue(),
            &[],
            v(at(2, 1).0 - 0.35, at(2, 1).1 + 0.4),
        )?;
        let quarters = kit.tex(
            stage,
            r"\tfrac24",
            34.0,
            Palette::blue(),
            &[],
            v(at(4, 2).0 - 0.35, at(4, 2).1 + 0.4),
        )?;
        let pair_dots = [
            dot(stage, at(2, 1), 0.13, Palette::blue()),
            dot(stage, at(4, 2), 0.13, Palette::blue()),
        ];
        let mut shown = vec![half, quarters];
        shown.extend(pair_dots);
        d4g::show(stage, &shown, 0.4)?;
        kit.until_sentence(stage, "03_8", 2)?;
        let rule = kit.tex(
            stage,
            r"\frac ab\sim\frac cd\iff ad=bc",
            36.0,
            Palette::ink(),
            &[],
            v(4.6, 1.9),
        )?;
        let check = kit.tex(stage, r"1\cdot4=2\cdot2", 32.0, Palette::blue(), &[], v(4.6, 0.9))?;
        d4g::show(stage, &[rule, check], 0.4)?;
        kit.until_sentence(stage, "03_8", 3)?;
        let classes = [
            ((1, 2), Palette::blue()),
            ((1, 1), Palette::gold()),
            ((2, 3), Palette::green()),
        ];
        let mut class_marks = Vec::new();
        for ((top, bottom), color) in classes {
            let members = class_in_lattice(top, bottom);
            let far = *members.last().expect("each drawn class has members");
            // The ray runs a little past the farthest member, along (b, a).
            let length = f64::from(bottom).hypot(f64::from(top));
            let (fx, fy) = at(far.0, far.1);
            let end = (
                fx + 0.35 * f64::from(bottom) / length,
                fy + 0.35 * f64::from(top) / length,
            );
            let ray = stage.arena_mut().add(
                Line::new(v(origin.0, origin.1), v(end.0, end.1))
                    .style(stroke_style(color, 3.0))
                    .build()?,
            );
            class_marks.push(ray);
            for &(bb, aa) in &members {
                class_marks.push(dot(stage, at(bb, aa), 0.11, color));
            }
        }
        draw(stage, &class_marks, 0.9)?;
        kit.until_sentence(stage, "03_8", 4)?;
        let mut slopes = Vec::new();
        for ((top, bottom), color) in classes {
            let far = *class_in_lattice(top, bottom)
                .last()
                .expect("each drawn class has members");
            let tex = if bottom == 1 {
                top.to_string()
            } else {
                format!(r"\tfrac{{{top}}}{{{bottom}}}")
            };
            slopes.push(kit.tex(
                stage,
                &tex,
                34.0,
                color,
                &[],
                v(at(far.0, far.1).0 + 0.55, at(far.0, far.1).1 + 0.1),
            )?);
        }
        d4g::show(stage, &slopes, 0.4)?;
        kit.until_sentence(stage, "03_8", 5)?;
        let rational = kit.text(
            stage,
            "One rational number = one whole class",
            28.0,
            Palette::ink(),
            v(4.0, -0.6),
        )?;
        d4g::show(stage, &[rational], 0.4)?;

        // 03_9: subgroups will do the same; preview on the clock.
        kit.say(stage, "03_9")?;
        let mut old = lattice;
        old.extend(shown);
        old.extend(class_marks);
        old.extend(slopes);
        old.extend([rule, check, rational]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = kit.text(
            stage,
            "Next: classes made by a subgroup",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        d4g::show(stage, &[heading], 0.4)?;
        let clock_at = |n: i32| {
            let angle = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f64::from(n) / 12.0;
            (-3.2 + 2.0 * angle.cos(), 0.1 + 2.0 * angle.sin())
        };
        let coset_colors = [Palette::blue(), Palette::gold(), Palette::green(), Palette::hero()];
        let mut clock = Vec::new();
        let mut hours = Vec::new();
        for n in 0..12 {
            let color = coset_colors[clock_coset(n) as usize];
            let (x, y) = clock_at(n);
            let token = dot(stage, (x, y), 0.3, color);
            let number = kit.text(stage, &signed(n), 24.0, explainer_kit::hex("#0E1116"), v(x, y))?;
            let group = d4g::group_of(stage, &[token, number])?;
            hours.push(group);
            clock.push(group);
        }
        d4g::show(stage, &clock, 0.5)?;
        kit.until_sentence(stage, "03_9", 1)?;
        let sub = kit.tex(stage, r"H=\{0,4,8\}", 36.0, Palette::blue(), &[], v(3.2, 1.9))?;
        d4g::show(stage, &[sub], 0.4)?;
        pulse(stage, &[hours[0], hours[4], hours[8]], 1.3)?;
        kit.until_sentence(stage, "03_9", 2)?;
        let cosets = kit.text(stage, "Its classes: cosets", 28.0, Palette::ink(), v(3.2, 1.0))?;
        d4g::show(stage, &[cosets], 0.4)?;
        kit.until_sentence(stage, "03_9", 3)?;
        // Each class collapses into one object of a new collection.
        let mut targets = Vec::new();
        for n in 0..12 {
            let k = f64::from(clock_coset(n));
            targets.push((hours[n as usize], (1.7 + 1.3 * k, -0.9 + 0.12 * f64::from(n / 4))));
        }
        d4g::glide_all(stage, &targets, 1.4)?;
        let new_group = kit.text(
            stage,
            "Four classes, ready to become elements",
            28.0,
            Palette::ink(),
            v(3.6, -2.0),
        )?;
        d4g::show(stage, &[new_group], 0.4)?;
        let mut wipe = clock;
        wipe.extend([heading, sub, cosets, new_group]);
        kit.hold(stage)?;
        d4g::hide(stage, &wipe, 0.6)?;
        kit.finish(stage)
    }
}

/// The clock coset of n for H = {0, 4, 8}: which of 0+H, 1+H, 2+H, 3+H holds n.
fn clock_coset(n: i32) -> i32 {
    n.rem_euclid(4)
}

/// Pairs (bottom, top) drawn in the fraction picture.
fn fraction_lattice() -> Vec<(i32, i32)> {
    (1..=8)
        .flat_map(|bottom| (1..=4).map(move |top| (bottom, top)))
        .collect()
}

/// Members of the lattice in the class of top/bottom, by cross-multiplication.
pub(crate) fn class_in_lattice(top: i32, bottom: i32) -> Vec<(i32, i32)> {
    fraction_lattice()
        .into_iter()
        .filter(|&(b, a)| a * bottom == b * top)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closeness_is_reflexive_and_symmetric_but_not_transitive_on_the_drawn_line() {
        for x in LINE {
            assert!(close(x, x));
            for y in LINE {
                assert_eq!(close(x, y), close(y, x));
            }
        }
        let [a, b, c] = crate::COUNTEREXAMPLE;
        assert!(close(a, b) && close(b, c) && !close(a, c));
        // The drawn chain of close steps reaches the end of the line.
        assert!((c..*LINE.end()).all(|n| close(n, n + 1)));
    }

    #[test]
    fn closeness_groups_share_two_members_yet_differ() {
        assert_eq!(close_to(1), vec![0, 1, 2]);
        assert_eq!(close_to(0), vec![-1, 0, 1]);
        let shared = close_to(1).into_iter().filter(|n| close_to(0).contains(n)).count();
        assert_eq!(shared, 2);
        assert_ne!(close_to(1), close_to(0));
    }

    #[test]
    fn lanes_partition_the_grid_and_lane_four_is_the_narrated_class() {
        let mut four = crate::LANE_FOUR.to_vec();
        four.sort();
        assert_eq!(lane_of(4), four);
        for n in GRID {
            let holding: Vec<i32> = (0..12).filter(|&lane| lane_of(lane).contains(&n)).collect();
            assert_eq!(holding.len(), 1, "{n} lies in exactly one lane");
            for m in GRID {
                let (p, q) = (lane_of(n), lane_of(m));
                assert!(
                    p == q || p.iter().all(|x| !q.contains(x)),
                    "classes coincide or are disjoint"
                );
            }
        }
        // The grid layout puts each lane in one column.
        for n in GRID {
            assert!((cell_at(n).0 - cell_at(n.rem_euclid(12)).0).abs() < 1e-9);
        }
    }

    #[test]
    fn same_piece_is_an_equivalence_relation() {
        let pieces: [&[usize]; 3] = [&[0, 1, 2], &[3, 4], &[5, 6, 7]];
        let piece = |x: usize| pieces.iter().position(|p| p.contains(&x)).unwrap();
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    if piece(x) == piece(y) && piece(y) == piece(z) {
                        assert_eq!(piece(x), piece(z));
                    }
                }
            }
        }
        assert_eq!(pieces.iter().map(|p| p.len()).sum::<usize>(), 8);
    }

    #[test]
    fn fraction_classes_lie_on_lines_through_the_origin_with_their_value_as_slope() {
        assert_eq!(class_in_lattice(1, 2), vec![(2, 1), (4, 2), (6, 3), (8, 4)]);
        assert_eq!(class_in_lattice(2, 3), vec![(3, 2), (6, 4)]);
        for (top, bottom) in [(1, 2), (1, 1), (2, 3)] {
            for (b, a) in class_in_lattice(top, bottom) {
                let slope = f64::from(a) / f64::from(b);
                assert!((slope - f64::from(top) / f64::from(bottom)).abs() < 1e-12);
            }
        }
        // Each lattice pair lies in exactly one class: equal slopes iff ad = bc.
        for (b, a) in fraction_lattice() {
            for (d, c) in fraction_lattice() {
                let same_slope = (f64::from(a) / f64::from(b) - f64::from(c) / f64::from(d)).abs() < 1e-12;
                assert_eq!(same_slope, a * d == b * c);
            }
        }
    }

    #[test]
    fn clock_colours_are_the_cosets_of_zero_four_eight() {
        let h = [0, 4, 8];
        for n in 0..12 {
            let coset: Vec<i32> = h.iter().map(|x| (x + n) % 12).collect();
            assert!(coset.iter().all(|&m| clock_coset(m) == clock_coset(n)));
        }
        assert_eq!((0..12).filter(|&n| clock_coset(n) == 0).collect::<Vec<_>>(), h);
    }
}
