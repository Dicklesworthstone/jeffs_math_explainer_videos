//! Chapter 14, the dark cell everywhere: the same four-cell table counts
//! France (Laplace, 14_1–14_3) and the bugs two reviewers missed (14_4–14_6).
//! The table stays at the centre; its four icons change what they stand for
//! (fish, then newborns and people, then bugs) and its headers relabel in
//! place. Laplace's inputs are recorded (`facts::LAPLACE`); the reviewers'
//! careful range is `law::careful_range`; 14_6's model is `runs::reviewers`.

use explainer_kit::props::{lower_third, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use super::failures::{Layout, Mosaic, go};
use super::ring_toss::Spoken;
use crate::facts::{LAPLACE, REVIEWERS};
use crate::law::careful_range;
use crate::runs::{self, BUGS, EASY, HARD};

/// The table: centre, and each cell's size.
const TABLE_C: Vec3 = [-1.25, 0.6, 0.0];
const CELL_W: f64 = 3.3;
const CELL_H: f64 = 1.45;
/// The inset to the table's right (France, the Venn, the bent mosaic).
const INSET_C: Vec3 = [4.85, 0.55, 0.0];
/// The lines under the table: the arithmetic, then the range.
const SUM_Y: [f64; 2] = [-1.55, -2.1];

/// Metropolitan France and Corsica, longitude and latitude in degrees:
/// Natural Earth's 1:110m countries (public domain), the two rings of
/// France in Europe, as published (credited on the receipts card, 15_3).
const FRANCE: [(f64, f64); 48] = [
    (6.186, 49.464),
    (6.658, 49.202),
    (8.099, 49.018),
    (7.594, 48.333),
    (7.467, 47.621),
    (7.192, 47.450),
    (6.737, 47.542),
    (6.769, 47.288),
    (6.037, 46.726),
    (6.023, 46.273),
    (6.500, 46.430),
    (6.844, 45.991),
    (6.802, 45.709),
    (7.097, 45.333),
    (6.750, 45.029),
    (7.008, 44.255),
    (7.550, 44.128),
    (7.435, 43.694),
    (6.529, 43.129),
    (4.557, 43.400),
    (3.100, 43.075),
    (2.986, 42.473),
    (1.827, 42.343),
    (0.702, 42.796),
    (0.338, 42.580),
    (-1.503, 43.034),
    (-1.901, 43.423),
    (-1.384, 44.023),
    (-1.194, 46.015),
    (-2.226, 47.064),
    (-2.963, 47.570),
    (-4.492, 47.955),
    (-4.592, 48.684),
    (-3.296, 48.902),
    (-1.617, 48.644),
    (-1.933, 49.776),
    (-0.989, 49.347),
    (1.339, 50.127),
    (1.639, 50.947),
    (2.514, 51.149),
    (2.658, 50.797),
    (3.123, 50.780),
    (3.588, 50.379),
    (4.286, 49.907),
    (4.799, 49.985),
    (5.674, 49.529),
    (5.898, 49.443),
    (6.186, 49.464),
];
const CORSICA: [(f64, f64); 7] = [
    (8.746, 42.628),
    (9.390, 43.010),
    (9.560, 42.152),
    (9.230, 41.380),
    (8.776, 41.584),
    (8.544, 42.257),
    (8.746, 42.628),
];

/// Laplace's bet, verbatim (Essai philosophique, tr. Truscott & Emory 1902,
/// p. 67; fact sheet row 10).
const LAPLACE_BET: &str = "“…the number of annual births in France being supposed to be 1000000, which brings the \
                           population to 28352845 inhabitants, it is a bet of almost 300000 against 1 that the \
                           error of this result is not half a million.”";

/// Mills's dictionary (Software Productivity, p. 73; fact sheet row 11).
const MILLS: &str = "H. D. Mills (1970; IBM report, 1972), Software Productivity p. 73: “'lake' is synonymous \
                     with 'system'…”; his marked fish were bugs planted on purpose";

/// The four cells' centres: caught twice, first only, second only, dark.
fn cells() -> [Vec3; 4] {
    let (x0, x1) = (TABLE_C[0] - CELL_W / 2.0, TABLE_C[0] + CELL_W / 2.0);
    let (y0, y1) = (TABLE_C[1] + CELL_H / 2.0, TABLE_C[1] - CELL_H / 2.0);
    [v(x0, y0), v(x0, y1), v(x1, y0), v(x1, y1)]
}

/// Where a cell's icon sits, and where its words start (left-aligned).
fn icon_at(c: Vec3) -> Vec3 {
    v(c[0] - CELL_W / 2.0 + 0.6, c[1])
}
fn words_at(c: Vec3) -> Vec3 {
    v(c[0] - CELL_W / 2.0 + 1.2, c[1])
}

/// A cell's words: after its icon, shrunk to fit the cell.
fn cell_text(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, c: Vec3) -> fmn::Result<Mob> {
    let at = words_at(c);
    let m = kit.text(stage, s, size, color, at)?;
    let room = CELL_W - 1.35;
    if stage.get_width(m) > room {
        stage.set_width(m, room, false);
    }
    stage.move_to(m, at, LEFT);
    Ok(m)
}

/// What the four icons stand for.
#[derive(Clone, Copy, PartialEq)]
enum World {
    Fish,
    People,
    Bugs,
}

/// A newborn, swaddled: a head and a rounded body, pointing east.
fn newborn(c: Srgb) -> VMobject {
    let body = Ellipse::new()
        .width(0.14)
        .height(0.08)
        .style(flat(c, 1.0))
        .build()
        .moved_to(v(-0.02, 0.0));
    let head = Circle::new()
        .radius(0.038)
        .arc_center(v(0.07, 0.0))
        .style(flat(c, 1.0))
        .build();
    v_group([body, head])
}

/// A person: a head over a rounded body.
fn person(c: Srgb) -> VMobject {
    let head = Circle::new()
        .radius(0.03)
        .arc_center(v(0.0, 0.065))
        .style(flat(c, 1.0))
        .build();
    let body = Ellipse::new()
        .width(0.075)
        .height(0.1)
        .style(flat(c, 1.0))
        .build()
        .moved_to(v(0.0, -0.025));
    v_group([head, body])
}

/// A bug: a body, a head, three legs a side and two feelers, pointing east.
fn bug(c: Srgb) -> VMobject {
    let mut parts = vec![
        Ellipse::new().width(0.11).height(0.066).style(flat(c, 1.0)).build(),
        Circle::new()
            .radius(0.024)
            .arc_center(v(0.068, 0.0))
            .style(flat(c, 1.0))
            .build(),
    ];
    let leg = line_style(c, 2.0);
    for dx in [-0.03, 0.0, 0.03] {
        for side in [-1.0, 1.0] {
            if let Ok(l) = Line::new(v(dx, 0.02 * side), v(dx - 0.012, 0.055 * side))
                .style(leg)
                .build()
            {
                parts.push(l);
            }
        }
    }
    for side in [-1.0, 1.0] {
        if let Ok(l) = Line::new(v(0.085, 0.012 * side), v(0.11, 0.04 * side))
            .style(leg)
            .build()
        {
            parts.push(l);
        }
    }
    v_group(parts)
}

/// The icon for `cell` (0 caught twice, 1 first only, 2 second only, 3
/// dark) in `world`: the first catch's colour, a green ring for the second.
fn icon(world: World, cell: usize) -> VMobject {
    let first = cell <= 1;
    let second = cell == 0 || cell == 2;
    let body = match (world, first) {
        (World::Fish, true) => marked_fish().mob,
        (World::Fish, false) => fish(Role::unmarked()).mob,
        (World::People, true) => newborn(Role::marked()),
        (World::People, false) => person(Role::unmarked()),
        (World::Bugs, true) => bug(Role::marked()),
        (World::Bugs, false) => bug(Role::unmarked()),
    };
    let mut parts = vec![body];
    if second {
        parts.push(ring().mob);
    }
    let g = v_group(parts);
    if cell == 3 {
        g.map_style_deep(|s| {
            s.fill(s.fill_color, s.fill_opacity * 0.3)
                .stroke(s.stroke_color, s.stroke_width, s.stroke_opacity * 0.3)
        })
    } else {
        g
    }
}

/// The table's four icons, scaled into their cells, detached, in cell order.
fn icons(stage: &mut Stage<'_>, world: World) -> [Mob; 4] {
    let make = |stage: &mut Stage<'_>, i: usize| {
        let m = stage.arena_mut().add(icon(world, i));
        stage.scale(m, 3.2);
        stage.move_to(m, icon_at(cells()[i]), ORIGIN);
        m
    };
    [make(stage, 0), make(stage, 1), make(stage, 2), make(stage, 3)]
}

/// The table's headers: two columns, two rows.
fn headers(kit: &Kit, stage: &mut Stage<'_>, words: [&str; 4]) -> fmn::Result<[Mob; 4]> {
    let c = cells();
    let top = TABLE_C[1] + CELL_H + 0.62;
    let left = TABLE_C[0] - CELL_W - 0.15;
    let col0 = kit.text(stage, words[0], 26.0, Role::marked(), v(c[0][0], top))?;
    let col1 = kit.text(stage, words[1], 26.0, Palette::ink(), v(c[2][0], top))?;
    let row0 = kit.text(stage, words[2], 24.0, Role::second(), v(left, c[0][1]))?;
    stage.move_to(row0, v(left, c[0][1]), RIGHT);
    let row1 = kit.text(stage, words[3], 24.0, Palette::dim(), v(left, c[1][1]))?;
    stage.move_to(row1, v(left, c[1][1]), RIGHT);
    Ok([col0, col1, row0, row1])
}

/// The table's lines: a cross and a frame, faint.
fn grid(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let (x0, x1) = (TABLE_C[0] - CELL_W, TABLE_C[0] + CELL_W);
    let (y0, y1) = (TABLE_C[1] - CELL_H, TABLE_C[1] + CELL_H);
    let st = line_style(Palette::faint(), 3.0);
    let mut parts = Vec::new();
    for (a, b) in [
        (v(x0, y0), v(x1, y0)),
        (v(x1, y0), v(x1, y1)),
        (v(x1, y1), v(x0, y1)),
        (v(x0, y1), v(x0, y0)),
        (v(TABLE_C[0], y0), v(TABLE_C[0], y1)),
        (v(x0, TABLE_C[1]), v(x1, TABLE_C[1])),
    ] {
        parts.push(Line::new(a, b).style(st).build()?);
    }
    // the dark cell, dark
    parts.push(
        Rectangle::new()
            .width(CELL_W - 0.06)
            .height(CELL_H - 0.06)
            .style(flat(hex("#0B1113"), 1.0))
            .build()?
            .moved_to(cells()[3]),
    );
    Ok(stage.arena_mut().add(v_group(parts)))
}

/// France at the inset, `height` tall, as one detached group.
fn france(stage: &mut Stage<'_>, height: f64) -> fmn::Result<Mob> {
    let (lat0, lat1) = (41.3, 51.2);
    let s = height / (lat1 - lat0);
    let squash = (46.5f64).to_radians().cos();
    let lon_mid = 2.5;
    let p = |&(lon, lat): &(f64, f64)| {
        v(
            INSET_C[0] + (lon - lon_mid) * squash * s,
            INSET_C[1] + (lat - (lat0 + lat1) / 2.0) * s,
        )
    };
    let style = Style::default()
        .fill(Palette::blue(), 0.12)
        .stroke(Palette::ink(), 2.5, 1.0);
    let main = Polygon::new(FRANCE.iter().map(p)).style(style).build();
    let isle = Polygon::new(CORSICA.iter().map(p)).style(style).build();
    Ok(stage.arena_mut().add(v_group([main, isle])))
}

/// Replace `old` with a new text in place.
#[allow(clippy::too_many_arguments)]
fn swap_text(
    kit: &Kit,
    stage: &mut Stage<'_>,
    old: Mob,
    s: &str,
    size: f64,
    color: Srgb,
    at: Vec3,
    edge: Vec3,
) -> fmn::Result<(Mob, Box<dyn Animation>)> {
    let m = kit.text(stage, s, size, color, at)?;
    stage.move_to(m, at, edge);
    let a = stage.prepare(replacement_transform(old, m).rt(0.8))?;
    Ok((m, a))
}

/// Scene error from a model error.
fn model(e: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(e.into_boxed_str())))
}

pub struct Everywhere {
    pub kit: Kit,
}

impl SceneConstruct for Everywhere {
    fn name(&self) -> &str {
        "14_the_dark_cell_everywhere"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let c = cells();
        let top = TABLE_C[1] + CELL_H + 0.62;
        let left = TABLE_C[0] - CELL_W - 0.15;

        // 14_1: the table as the fish left it; France at the side; each
        // newborn is a marked fish
        let lines = grid(stage)?;
        let mut icon_mobs = icons(stage, World::Fish);
        let mut heads = headers(&kit, stage, ["marked", "unmarked", "second catch", "not"])?;
        let question = cell_text(&kit, stage, "?", 60.0, Palette::dim(), c[3])?;
        let line = Spoken::say(&kit, stage, "14_1")?;
        let mut intro: Vec<Box<dyn Animation>> = Vec::new();
        for m in [lines, question].into_iter().chain(icon_mobs).chain(heads) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.8);
            intro.push(stage.prepare(a)?);
        }
        stage.play_prepared(intro)?;
        let outline = france(stage, 3.4)?;
        line.at(stage, "to count France")?;
        play!(stage; fmn::animation::show_creation(outline).rt(1.6));
        line.at(stage, "each newborn")?;
        // the marked column's fish become newborns
        let people = icons(stage, World::People);
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for i in [0, 1] {
            morph.push(stage.prepare(replacement_transform(icon_mobs[i], people[i]).rt(1.2))?);
            icon_mobs[i] = people[i];
        }
        let (h0, a) = swap_text(
            &kit,
            stage,
            heads[0],
            "born this year",
            26.0,
            Role::marked(),
            v(c[0][0], top),
            ORIGIN,
        )?;
        heads[0] = h0;
        morph.push(a);
        stage.play_prepared(morph)?;
        kit.hold(stage)?;

        // 14_2: officials counted everyone in chosen towns; one birth for
        // every 28 and a bit people; a million births make 28.4 million
        let (people_counted, boys, girls, assumed) = LAPLACE;
        let births3 = boys + girls;
        let (whole, third) = (births3 / 3, births3 % 3);
        let per_year = if third == 0 {
            thousands(whole)
        } else {
            format!("{}{}", thousands(whole), ["", "⅓", "⅔"][third as usize])
        };
        let ratio = (3 * people_counted) as f64 / births3 as f64;
        let estimate = (3 * people_counted * assumed + births3 / 2) / births3;
        let line = Spoken::say(&kit, stage, "14_2")?;
        let mut relabel: Vec<Box<dyn Animation>> = Vec::new();
        for i in [2, 3] {
            relabel.push(stage.prepare(replacement_transform(icon_mobs[i], people[i]).rt(1.0))?);
            icon_mobs[i] = people[i];
        }
        let (h1, a) = swap_text(
            &kit,
            stage,
            heads[1],
            "everyone else",
            26.0,
            Palette::ink(),
            v(c[2][0], top),
            ORIGIN,
        )?;
        heads[1] = h1;
        relabel.push(a);
        let (h2, a) = swap_text(
            &kit,
            stage,
            heads[2],
            "counted",
            24.0,
            Role::second(),
            v(left, c[0][1]),
            RIGHT,
        )?;
        heads[2] = h2;
        relabel.push(a);
        let (h3, a) = swap_text(
            &kit,
            stage,
            heads[3],
            "not counted",
            24.0,
            Palette::dim(),
            v(left, c[1][1]),
            RIGHT,
        )?;
        heads[3] = h3;
        relabel.push(a);
        stage.play_prepared(relabel)?;
        let towns = kit.text(
            stage,
            "chosen towns across France",
            22.0,
            Role::second(),
            v(INSET_C[0], INSET_C[1] - 2.0),
        )?;
        play!(stage; fade_in(stage.arena_mut(), towns, ORIGIN, 1.0)?.rt(0.6));
        // "about two million people, with about seventy-two thousand births a year"
        let counted_at = v(left, c[0][1] - 0.36);
        let counted = kit.text(
            stage,
            &format!("{} people", thousands(people_counted)),
            22.0,
            Role::second(),
            counted_at,
        )?;
        stage.move_to(counted, counted_at, RIGHT);
        let births = cell_text(
            &kit,
            stage,
            &format!("{per_year} births a year"),
            26.0,
            Role::marked(),
            c[0],
        )?;
        let assumed_births = kit.text(
            stage,
            &format!("{} a year in all France (assumed)", thousands(assumed)),
            20.0,
            Role::marked(),
            v(c[0][0], TABLE_C[1] + CELL_H + 0.27),
        )?;
        line.at(stage, "Just over two million people")?;
        play!(stage; fade_in(stage.arena_mut(), counted, v(0.0, -0.1), 1.0)?.rt(0.7));
        line.at(stage, "about seventy-two thousand")?;
        play!(stage; fade_in(stage.arena_mut(), births, ORIGIN, 1.0)?.rt(0.7));
        line.at(stage, "That's one birth")?;
        // "one birth for every twenty-eight and a bit people"
        let per_birth = kit.text(
            stage,
            &format!(
                "{} ÷ {per_year} ≈ {ratio:.2} people per birth",
                thousands(people_counted)
            ),
            28.0,
            Palette::ink(),
            v(TABLE_C[0], SUM_Y[0]),
        )?;
        play!(stage; write(stage.arena(), per_birth).rt(1.2));
        line.at(stage, "Taking France's births")?;
        play!(stage; fade_in(stage.arena_mut(), assumed_births, ORIGIN, 1.0)?.rt(0.7));
        let total = kit.text(
            stage,
            &format!(
                "{ratio:.6} × {} = {} ≈ {:.1} million people",
                thousands(assumed),
                thousands(estimate),
                estimate as f64 / 1e6
            ),
            28.0,
            Role::dark(),
            v(TABLE_C[0], SUM_Y[1]),
        )?;
        play!(stage; write(stage.arena(), total).rt(1.4));
        let dark_total = cell_text(&kit, stage, "the rest of France", 24.0, Role::dark(), c[3])?;
        play!(stage; replacement_transform(question, dark_total).rt(0.8));
        kit.hold(stage)?;

        // 14_3: the odds card, silent
        let bet_lines = explainer_kit::animatic::wrap(LAPLACE_BET, 64, 4);
        let card_h = 0.5 * bet_lines.len() as f64 + 1.3;
        let back = stage.arena_mut().add(
            Rectangle::new()
                .width(11.6)
                .height(card_h)
                .corner_radius(0.12)
                .style(
                    Style::default()
                        .fill(hex("#1A2130"), 0.97)
                        .stroke(Palette::faint(), 1.5, 1.0),
                )
                .build()?
                .moved_to(v(0.0, 0.2)),
        );
        let mut card = vec![back];
        for (i, line) in bet_lines.iter().enumerate() {
            card.push(kit.text(
                stage,
                line,
                30.0,
                Palette::ink(),
                v(0.0, 0.2 + card_h / 2.0 - 0.55 - 0.5 * i as f64),
            )?);
        }
        card.push(kit.text(
            stage,
            "Laplace, Essai philosophique (tr. Truscott & Emory 1902), p. 67",
            22.0,
            Palette::dim(),
            v(0.0, 0.2 - card_h / 2.0 + 0.72),
        )?);
        card.push(kit.text(
            stage,
            "his calculation; it treats the chosen towns as typical",
            22.0,
            Palette::dim(),
            v(0.0, 0.2 - card_h / 2.0 + 0.35),
        )?);
        let mut show: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &card {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.15), 1.0)?.rt(0.8);
            show.push(stage.prepare(a)?);
        }
        stage.play_prepared(show)?;
        kit.music_breath(stage, 4.0)?;
        let mut hide: Vec<Box<dyn Animation>> = Vec::new();
        for m in card.iter().copied().chain([
            outline,
            towns,
            counted,
            births,
            assumed_births,
            per_birth,
            total,
            dark_total,
        ]) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.6);
            hide.push(stage.prepare(a)?);
        }
        stage.play_prepared(hide)?;

        // 14_4: it counts bugs too
        let (ra, rb, rk) = REVIEWERS;
        let bugs = icons(stage, World::Bugs);
        let line = Spoken::say(&kit, stage, "14_4")?;
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..4 {
            morph.push(stage.prepare(replacement_transform(icon_mobs[i], bugs[i]).rt(1.0))?);
            icon_mobs[i] = bugs[i];
        }
        for (i, (word, size, color, at, edge)) in [
            ("found by A", 26.0, Role::marked(), v(c[0][0], top), ORIGIN),
            ("missed by A", 26.0, Palette::ink(), v(c[2][0], top), ORIGIN),
            ("found by B", 24.0, Role::second(), v(left, c[0][1]), RIGHT),
            ("missed by B", 24.0, Palette::dim(), v(left, c[1][1]), RIGHT),
        ]
        .into_iter()
        .enumerate()
        {
            let (m, a) = swap_text(&kit, stage, heads[i], word, size, color, at, edge)?;
            heads[i] = m;
            morph.push(a);
        }
        stage.play_prepared(morph)?;
        // the Venn: A's bugs and B's, twelve in both
        let venn = {
            let r = 1.15;
            let a = Circle::new()
                .radius(r)
                .arc_center(v(INSET_C[0] - 0.55, INSET_C[1]))
                .style(
                    Style::default()
                        .fill(Role::marked(), 0.18)
                        .stroke(Role::marked(), 3.0, 1.0),
                )
                .build();
            let b = Circle::new()
                .radius(r * (rb as f64 / ra as f64).sqrt())
                .arc_center(v(INSET_C[0] + 0.55, INSET_C[1]))
                .style(
                    Style::default()
                        .fill(Role::second(), 0.18)
                        .stroke(Role::second(), 3.0, 1.0),
                )
                .build();
            stage.arena_mut().add(v_group([a, b]))
        };
        let venn_a = kit.text(
            stage,
            &format!("A: {ra}"),
            26.0,
            Role::marked(),
            v(INSET_C[0] - 1.25, INSET_C[1] + 1.45),
        )?;
        let venn_b = kit.text(
            stage,
            &format!("B: {rb}"),
            26.0,
            Role::second(),
            v(INSET_C[0] + 1.25, INSET_C[1] + 1.45),
        )?;
        let venn_k = kit.text(stage, &rk.to_string(), 30.0, Role::marked(), v(INSET_C[0], INSET_C[1]))?;
        line.at(stage, "Say two reviewers")?;
        play!(stage;
            fmn::animation::grow_from_center(stage.arena_mut(), venn, None)?.rt(0.9),
            fade_in(stage.arena_mut(), venn_a, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), venn_b, ORIGIN, 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), venn_k, ORIGIN, 1.0)?.rt(0.9),
        );
        let numbers = [
            cell_text(&kit, stage, &format!("{rk} found by both"), 26.0, Role::marked(), c[0])?,
            cell_text(
                &kit,
                stage,
                &format!("{} by A only", ra - rk),
                26.0,
                Role::marked(),
                c[1],
            )?,
            cell_text(
                &kit,
                stage,
                &format!("{} by B only", rb - rk),
                26.0,
                Role::second(),
                c[2],
            )?,
        ];
        let hiding = cell_text(&kit, stage, "?", 60.0, Palette::dim(), c[3])?;
        line.at(stage, "One finds thirty")?;
        let mut fill: Vec<Box<dyn Animation>> = Vec::new();
        for m in numbers.iter().copied().chain([hiding]) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.7);
            fill.push(stage.prepare(a)?);
        }
        stage.play_prepared(fill)?;
        let mills = lower_third(&kit, stage, "source:", MILLS)?;
        play!(stage; fade_in(stage.arena_mut(), mills, v(0.0, 0.2), 1.0)?.rt(0.8));
        let q = predict_pause(&kit, stage, v(2.3, 1.8))?;

        // 14_5: 30 × 24 ÷ 12 = 60; 18 still hiding; the careful range
        let estimate = ra * rb / rk;
        let seen = ra + rb - rk;
        let (lo, hi) = careful_range(ra, rb, rk).map_err(model)?;
        let line = Spoken::say(&kit, stage, "14_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_out(stage.arena_mut(), mills, ORIGIN, 1.0)?.rt(0.6),
        );
        let sum = kit.text(
            stage,
            &format!(
                "{ra} × {rb} ÷ {rk} = {estimate} bugs;  found {seen}, so about {} hiding",
                estimate - seen
            ),
            28.0,
            Palette::ink(),
            v(TABLE_C[0], SUM_Y[0]),
        )?;
        play!(stage; write(stage.arena(), sum).rt(1.4));
        let about = cell_text(&kit, stage, &format!("≈ {}", estimate - seen), 40.0, Role::dark(), c[3])?;
        line.at(stage, "They found forty-two")?;
        play!(stage; replacement_transform(hiding, about).rt(0.8));
        line.at(stage, "If every bug")?;
        // the range, on a line of total bugs
        let (x0, x1, y) = (-5.6, 3.0, -2.75);
        let lo_n = 40.0;
        let hi_n = 100.0;
        let xs = |n: f64| x0 + (x1 - x0) * (n - lo_n) / (hi_n - lo_n);
        let mut axis_parts = vec![
            Line::new(v(x0, y), v(x1, y))
                .style(line_style(Palette::dim(), 2.5))
                .build()?,
        ];
        for t in [40.0, 60.0, 80.0, 100.0] {
            axis_parts.push(
                Line::new(v(xs(t), y - 0.07), v(xs(t), y + 0.07))
                    .style(line_style(Palette::dim(), 2.5))
                    .build()?,
            );
        }
        let axis = stage.arena_mut().add(v_group(axis_parts));
        for t in [40.0, 60.0, 80.0, 100.0] {
            let m = kit.text(stage, &format!("{t}"), 20.0, Palette::dim(), v(xs(t), y - 0.28))?;
            stage.attach(axis, m)?;
        }
        let range = stage.arena_mut().add(explainer_kit::rect(
            v(xs(lo as f64), y + 0.1),
            xs(hi as f64) - xs(lo as f64),
            0.16,
            flat(Role::dark(), 0.95),
        )?);
        let range_words = kit.text_left(
            stage,
            &format!(
                "careful range for all the bugs: {lo} to {hi}, so {} to {} still hiding",
                lo - seen,
                hi - seen
            ),
            24.0,
            Role::dark(),
            v(x0, y + 0.5),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.6),
            fmn::animation::grow_from_edge(stage.arena_mut(), range, LEFT, None)?.rt(1.0),
        );
        play!(stage; fade_in(stage.arena_mut(), range_words, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 14_6: skill is fine; difficulty is not. Easy bugs swell the
        // caught-twice cell, like the bold fish
        let [(_, _), (_, difficulty)] = runs::reviewers();
        let found: Vec<&runs::Review> = difficulty.iter().filter(|r| r.both > 0).collect();
        let mut sides = [0usize; 3];
        for r in &found {
            sides[runs::range_side(r).map_err(model)? as usize] += 1;
        }
        let below_pct = (100.0 * sides[0] as f64 / found.len() as f64).round();
        if sides[2] != 0 {
            return Err(model(format!(
                "14_6 says the range is always below 60; {} runs were above",
                sides[2]
            )));
        }
        let half = BUGS / 2;
        // the bent table of the expected counts: A finds 30, B 24, both 17.4
        let (ea, eb) = (half as f64 * (EASY.0 + HARD.0), half as f64 * (EASY.1 + HARD.1));
        let ek = half as f64 * (EASY.0 * EASY.1 + HARD.0 * HARD.1);
        let small = Layout {
            x: 3.05,
            top: 1.95,
            h: 2.7,
            per: 3.6 / BUGS as f64,
            view: 1.0,
            caught: eb,
        };
        let (mos, picture) = Mosaic::build(stage, small, (ea, BUGS as f64 - ea, eb * ea / BUGS as f64));
        let like = kit.text(
            stage,
            "like the bold fish (chapter 13)",
            22.0,
            Palette::dim(),
            v(INSET_C[0], 2.3),
        )?;
        let skill_ok = kit.text_left(
            stage,
            "reviewers of different skill: no trouble",
            22.0,
            Role::second(),
            v(2.9, -1.2),
        )?;
        let hard_no = kit.text_left(
            stage,
            "bugs of different difficulty: trouble",
            22.0,
            Role::bias(),
            v(2.9, -1.58),
        )?;
        // the opposite case, so "no trouble" is not over-read (R3: Dev D12, Ruth 11)
        let specialists = kit.text_left(
            stage,
            "specialist reviewers: guess runs high",
            20.0,
            Palette::dim(),
            v(2.9, -1.94),
        )?;
        let line = Spoken::say(&kit, stage, "14_6")?;
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        for m in [venn, venn_a, venn_b, venn_k] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.5);
            swap.push(stage.prepare(a)?);
        }
        stage.play_prepared(swap)?;
        play!(stage; fade_in(stage.arena_mut(), skill_ok, ORIGIN, 1.0)?.rt(0.6));
        line.at(stage, "Bugs of different difficulty")?;
        play!(stage;
            fade_in(stage.arena_mut(), hard_no, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), specialists, ORIGIN, 1.0)?.rt(0.6),
        );
        stage.add_to_scene(picture)?;
        // the true 60, dashed, so the estimate's shrinking shows
        let ghost = {
            let (x0, x1) = (small.x, small.x + BUGS as f64 * small.per);
            let (y0, y1) = (small.top - small.h, small.top);
            let mut sides = Vec::new();
            for (a, b) in [(v(x0, y0), v(x1, y0)), (v(x1, y0), v(x1, y1)), (v(x1, y1), v(x0, y1))] {
                sides.push(
                    DashedLine::new(a, b)
                        .dash_length(0.08)
                        .style(line_style(Role::dark(), 2.0))
                        .build()
                        .map_err(|e| model(e.to_string()))?,
                );
            }
            stage.arena_mut().add(v_group(sides))
        };
        play!(stage;
            fade_in(stage.arena_mut(), picture, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), like, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), ghost, ORIGIN, 1.0)?.rt(0.6),
        );
        // easy bugs: both reviewers find them, so the caught-twice cell swells
        line.at(stage, "Bugs that are easy")?;
        go(stage, &[(mos.twice, ek)], 1.2)?;
        line.at(stage, "pull the guess down")?;
        // the estimate: the lake of bugs that straightens it, short of 60
        go(stage, &[(mos.unmarked, ea * eb / ek - ea)], 1.2)?;
        let card = kit.text_left(
            stage,
            &format!(
                "model: {half} easy bugs, {} hard; the range missed the true {BUGS} in {below_pct}% of runs, always below it",
                BUGS - half
            ),
            22.0,
            Palette::dim(),
            v(-6.6, -3.55),
        )?;
        line.at(stage, "So expect sixty")?;
        play!(stage; fade_in(stage.arena_mut(), card, ORIGIN, 1.0)?.rt(0.7));
        kit.hold(stage)?;
        stage.clear_updaters(picture, true);
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    //! Narration truth: every number chapter 14 speaks, as it rounds.
    use super::*;
    use crate::chapters::ring_toss::beats_are_spoken;

    #[test]
    fn laplace_as_said() {
        let (people, boys, girls, assumed) = LAPLACE;
        let births3 = boys + girls;
        // "about two million people, with about seventy-two thousand births a year"
        assert_eq!((people + 500_000) / 1_000_000, 2);
        assert_eq!((births3 / 3 + 500) / 1_000, 72);
        // "one birth for every twenty-eight and a bit people"
        let ratio = (3 * people) as f64 / births3 as f64;
        assert!(ratio > 28.0 && ratio < 28.5);
        assert_eq!(format!("{ratio:.6}"), "28.352845", "Laplace's printed ratio");
        // "about twenty-eight point four million people"
        let estimate = (3 * people * assumed + births3 / 2) / births3;
        assert_eq!(estimate, 28_352_845, "Laplace's printed population");
        assert_eq!(format!("{:.1}", estimate as f64 / 1e6), "28.4");
    }

    #[test]
    fn the_reviewers_as_said() {
        let (a, b, k) = REVIEWERS;
        // "Thirty times twenty-four, over twelve, is sixty. They found
        // forty-two, so about eighteen are still hiding."
        assert_eq!((a * b % k, a * b / k), (0, 60));
        assert_eq!((a + b - k, a * b / k - (a + b - k)), (42, 18));
        // "the careful range runs from forty-seven to ninety-four"
        assert_eq!(careful_range(a, b, k), Ok((47, 94)));
        // "Reviewers of different skill cause no trouble. Bugs of different
        // difficulty do": skill's range holds 60 more than 95% of the time;
        // difficulty's lies wholly below it in 77% of runs, never above
        let [(_, skill), (_, difficulty)] = runs::reviewers();
        let sides = |reviews: &[runs::Review]| {
            let mut n = [0usize; 3];
            for r in reviews.iter().filter(|r| r.both > 0) {
                n[runs::range_side(r).unwrap() as usize] += 1;
            }
            n
        };
        let s = sides(&skill);
        assert!(s[1] as f64 / skill.len() as f64 > 0.95);
        let d = sides(&difficulty);
        assert_eq!(d[2], 0, "never above");
        assert_eq!((100.0 * d[0] as f64 / difficulty.len() as f64).round(), 77.0);
    }

    #[test]
    fn the_beats_wait_for_words_in_their_lines() {
        beats_are_spoken(&[
            ("14_1", "to count France"),
            ("14_1", "each newborn"),
            ("14_2", "Just over two million people"),
            ("14_2", "about seventy-two thousand"),
            ("14_2", "That's one birth"),
            ("14_2", "Taking France's births"),
            ("14_4", "Say two reviewers"),
            ("14_4", "One finds thirty"),
            ("14_5", "They found forty-two"),
            ("14_5", "If every bug"),
            ("14_6", "Bugs of different difficulty"),
            ("14_6", "Bugs that are easy"),
            ("14_6", "pull the guess down"),
            ("14_6", "So expect sixty"),
        ])
        .unwrap();
    }
}
