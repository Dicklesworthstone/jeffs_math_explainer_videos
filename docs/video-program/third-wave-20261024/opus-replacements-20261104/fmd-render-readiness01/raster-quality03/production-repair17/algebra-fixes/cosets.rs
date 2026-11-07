//! Chapter 14: cosets, the representative test, equal sizes and Lagrange.
//! The clock shift of H = {0, 4, 8} (unchanged recorded beats) is followed
//! by a group where order matters: on the chapter 07 rack, copies of
//! H = {e, s} travel to every column (left cosets), right cosets are drawn
//! as links that cross the columns, a token walks the clock to test the
//! representatives one/nine and one/seven, s·H visibly reshuffles H, and the
//! final count strikes the sizes three, five, six and seven that chapter
//! 08's shelves left empty. Every set and count drawn is computed.
use crate::d4_group::{self as d4g, ALL, E, Elem, R, R2, R3, RS, S};
use explainer_kit::{Kit, Palette, Timed, ink_style, play, stroke_style, tween, v};
use fmn::prelude::*;

pub struct Cosets {
    pub kit: Kit,
}

/// The clock subgroup and the two narrated representative tests.
pub(crate) const CLOCK_H: [i32; 3] = [0, 4, 8];
pub(crate) const SAME_COSET: (i32, i32) = (1, 9);
pub(crate) const OTHER_COSET: (i32, i32) = (1, 7);
/// The square subgroup H = {e, s}.
pub(crate) const FLIP_H: [Elem; 2] = [E, S];

const RACK_X0: f64 = -5.4;
const RACK_DX: f64 = 1.45;
const ROW_Y: [f64; 2] = [1.0, -0.75];
const PANEL_X: f64 = 3.4;

fn clock_position(value: f64, radius: f64) -> Vec3 {
    let angle = std::f64::consts::TAU * value / 12.;
    v(-3. + radius * angle.sin(), radius * angle.cos())
}

fn shift_pattern(stage: &mut Stage<'_>, dots: &[Mob], from: f64, to: f64) -> fmn::Result<()> {
    // Small simultaneous steps trace the clock, never straight chords through it.
    for step in 1..=12 {
        let offset = from + (to - from) * f64::from(step) / 12.;
        let mut animations: Vec<Box<dyn Animation>> = Vec::new();
        for (index, &dot) in dots.iter().enumerate() {
            let move_dot = dot
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.1),
                    ..AnimateArgs::default()
                })?
                .move_to(clock_position(index as f64 * 4. + offset, 1.85), ORIGIN)?;
            animations.push(stage.prepare(move_dot)?);
        }
        stage.play_prepared(animations)?;
    }
    Ok(())
}

fn fade(stage: &mut Stage<'_>, items: &[Mob], show: bool) -> fmn::Result<()> {
    if show {
        d4g::show(stage, items, 0.4)
    } else {
        d4g::hide(stage, items, 0.4)
    }
}

/// The on-screen verdict for the step from `g` to `k`, from the test itself.
fn step_verdict(g: i32, k: i32) -> (String, Srgb) {
    let step = (k - g).rem_euclid(12);
    if same_clock_coset(g, k) {
        (format!(r"{k}-{g}={step}\in H"), Palette::green())
    } else {
        (format!(r"{k}-{g}={step}\notin H"), Palette::gold())
    }
}

fn rack_at(g: Elem) -> (f64, f64) {
    (RACK_X0 + RACK_DX * f64::from(g.turns), ROW_Y[usize::from(g.flip)])
}

/// The clock difference test: k + H = g + H exactly when k − g lies in H.
pub(crate) fn same_clock_coset(g: i32, k: i32) -> bool {
    CLOCK_H.contains(&(k - g).rem_euclid(12))
}

/// The distinct left cosets gH (rack order), one per rack column.
pub(crate) fn left_cosets(h: &[Elem]) -> Vec<Vec<Elem>> {
    let mut out: Vec<Vec<Elem>> = Vec::new();
    for g in ALL {
        let coset = d4g::left_coset(g, h);
        if !out.contains(&coset) {
            out.push(coset);
        }
    }
    out
}

pub(crate) fn right_cosets(h: &[Elem]) -> Vec<Vec<Elem>> {
    let mut out: Vec<Vec<Elem>> = Vec::new();
    for g in ALL {
        let coset = d4g::right_coset(g, h);
        if !out.contains(&coset) {
            out.push(coset);
        }
    }
    out
}

fn set_tex(members: &[Elem]) -> String {
    let names: Vec<&str> = members.iter().map(|g| g.tex()).collect();
    format!(r"\{{{}\}}", names.join(","))
}

/// The rack, as chapter 07 left it: thumbnails with their names.
fn rack(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    let mut items = Vec::new();
    for g in ALL {
        let (x, y) = rack_at(g);
        items.push(d4g::thumb(stage, kit, g, (x, y), 1.0)?);
        items.push(kit.text(stage, g.name(), 24.0, Palette::ink(), v(x, y - 0.68))?);
    }
    Ok(items)
}

fn column_frame(stage: &mut Stage<'_>, turns: u8, color: Srgb) -> fmn::Result<Mob> {
    let x = RACK_X0 + RACK_DX * f64::from(turns);
    let shape = Rectangle::new()
        .width(1.3)
        .height(3.45)
        .corner_radius(0.2)
        .style(stroke_style(color, 4.0))
        .build()?
        .moved_to(v(x, -0.12));
    Ok(stage.arena_mut().add(shape))
}

const COSET_COLORS: [fn() -> Srgb; 4] = [Palette::hero, Palette::blue, Palette::green, Palette::gold];

/// Copies of H's thumbnails leave column 0 and land on the slots of gH.
fn carry_h(stage: &mut Stage<'_>, kit: &Kit, g: Elem, run_time: f64) -> fmn::Result<()> {
    let mut copies = Vec::new();
    let mut moves = Vec::new();
    for &h in &FLIP_H {
        let copy = d4g::thumb(stage, kit, h, rack_at(h), 1.0)?;
        copies.push(copy);
        moves.push((copy, rack_at(g.mul(h))));
    }
    d4g::show(stage, &copies, 0.2)?;
    d4g::glide_all(stage, &moves, run_time)?;
    d4g::hide(stage, &copies, 0.3)?;
    Ok(())
}

/// Half a rack tile: `d4g::thumb` draws the square at ±0.48.
const TILE_HALF: f64 = 0.48;
/// The link's half width (stroke 6.0), so its round cap just meets a tile edge.
const LINK_CAP: f64 = 0.03;
/// A link leaves or enters a tile this far right or left of the column centre,
/// clear of the name under the top tile and of the corner dots inside it.
const LINK_OFFSET: f64 = 0.25;
/// The heights where links run between columns, below the top row's names and
/// above the bottom row's tiles: one for links going right, one for left.
const LINK_LANES: [f64; 2] = [0.1, -0.1];

/// The corners of the link from a right coset's rotation (top row) to its flip
/// (bottom row). It leaves the bottom edge of the top tile and enters the top
/// edge of the bottom tile, beside the names, so it never crosses a tile, a
/// corner number or a name. A link between two columns runs along a lane in
/// the gap between the rows.
fn link_route(coset: &[Elem]) -> Vec<(f64, f64)> {
    let (top, bottom) = (rack_at(coset[0]), rack_at(coset[1]));
    let leave = (top.0 + LINK_OFFSET, top.1 - TILE_HALF - LINK_CAP);
    let arrive_y = bottom.1 + TILE_HALF + LINK_CAP;
    if coset[0].turns == coset[1].turns {
        return vec![leave, (leave.0, arrive_y)];
    }
    let arrive_x = bottom.0 - LINK_OFFSET;
    let lane = LINK_LANES[usize::from(bottom.0 < top.0)];
    vec![leave, (leave.0, lane), (arrive_x, lane), (arrive_x, arrive_y)]
}

/// A link between the two members of a right coset on the rack, drawn from
/// the rotation to the flip.
fn link_of(stage: &mut Stage<'_>, coset: &[Elem], color: Srgb) -> Mob {
    let corners = link_route(coset).into_iter().map(|(x, y)| v(x, y));
    stage
        .arena_mut()
        .add(Polygon::polyline(corners).style(stroke_style(color, 6.0)).build())
}

/// A token walks the clock from `from` to `to` one position at a time; a
/// ring marks where it started.
fn walk(stage: &mut Stage<'_>, from: i32, to: i32, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let step = stage.add_value_tracker(0.0);
    let start = f64::from(from);
    let token = stage.always_redraw(move |arena| {
        let t = arena.tracker_value(step).unwrap_or(0.0);
        arena.add(
            Circle::new()
                .radius(0.2)
                .style(ink_style(color))
                .build()
                .moved_to(clock_position(start + t, 1.85)),
        )
    });
    stage.add_to_scene(token)?;
    let home = stage.arena_mut().add(
        Circle::new()
            .radius(0.2)
            .style(stroke_style(color, 4.0))
            .build()
            .moved_to(clock_position(start, 1.85)),
    );
    d4g::show(stage, &[home], 0.2)?;
    let steps = (to - from).rem_euclid(12);
    play!(stage; tween(step, f64::from(steps), 0.3 * f64::from(steps))?);
    Ok(vec![token, home])
}

fn tex_at(stage: &mut Stage<'_>, kit: &Kit, src: &str, size: f64, color: Srgb, y: f64) -> fmn::Result<Mob> {
    kit.tex(stage, src, size, color, &[], v(PANEL_X, y))
}

impl SceneConstruct for Cosets {
    fn name(&self) -> &str {
        "14_cosets"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let k = self.kit.clone();

        // 14_1 .. 14_3: the recorded clock beats, unchanged.
        k.say(stage, "14_1")?;
        let title = k.text(
            stage,
            "Shift a subgroup around the clock",
            36.0,
            Palette::ink(),
            v(0.0, 3.0),
        )?;
        fade(stage, &[title], true)?;
        let dial = stage.arena_mut().add(
            Circle::new()
                .radius(1.85)
                .style(stroke_style(Palette::dim(), 2.))
                .build()
                .moved_to(v(-3., 0.)),
        );
        let mut clock = vec![dial];
        for n in 0..12 {
            clock.push(k.text(
                stage,
                &n.to_string(),
                23.,
                Palette::ink(),
                clock_position(f64::from(n), 2.2),
            )?);
        }
        fade(stage, &clock, true)?;
        let colors = [Palette::blue(), Palette::hero(), Palette::green(), Palette::gold()];
        let mut rows: Vec<Vec<Mob>> = Vec::new();
        let mut patterns: Vec<Vec<Mob>> = Vec::new();
        for (row, color) in colors.iter().enumerate() {
            let y = 1.6 - row as f64 * 1.05;
            let label = k.text(stage, &format!("{} + H", row), 27.0, *color, v(0.4, y))?;
            let mut one_row = vec![label];
            let mut pattern = Vec::new();
            for (col, base) in CLOCK_H.iter().enumerate() {
                let x = 2.0 + col as f64 * 1.25;
                let value = (*base as usize + row) % 12;
                let circle = stage.arena_mut().add(
                    Circle::new()
                        .radius(0.31)
                        .style(ink_style(*color))
                        .build()
                        .moved_to(v(x, y)),
                );
                let number = k.text(stage, &value.to_string(), 25.0, explainer_kit::hex("#0E1116"), v(x, y))?;
                one_row.extend([circle, number]);
                pattern.push(
                    stage.arena_mut().add(
                        Circle::new()
                            .radius(0.16)
                            .style(ink_style(*color))
                            .build()
                            .moved_to(clock_position(f64::from(*base), 1.85)),
                    ),
                );
            }
            rows.push(one_row);
            patterns.push(pattern);
        }
        fade(stage, &patterns[0], true)?;
        fade(stage, &rows[0], true)?;
        k.until_sentence(stage, "14_1", 3)?;
        fade(stage, &patterns[1], true)?;
        shift_pattern(stage, &patterns[1], 0., 1.)?;
        fade(stage, &rows[1], true)?;
        k.until_sentence(stage, "14_1", 4)?;
        fade(stage, &patterns[2], true)?;
        shift_pattern(stage, &patterns[2], 0., 2.)?;
        fade(stage, &rows[2], true)?;
        fade(stage, &patterns[3], true)?;
        shift_pattern(stage, &patterns[3], 0., 3.)?;
        fade(stage, &rows[3], true)?;
        k.say(stage, "14_2")?;
        let q = k.text(stage, "Is 4 + H a new set?", 29.0, Palette::gold(), v(0., -2.8))?;
        fade(stage, &[q], true)?;
        k.hold(stage)?;
        stage.wait(3.0)?;
        k.say(stage, "14_13")?;
        shift_pattern(stage, &patterns[0], 0., 4.)?;
        fade(stage, &[q], false)?;
        let answer = k.tex(stage, r"4+H=\{4,8,0\}=H", 30.0, Palette::green(), &[], v(0., -2.8))?;
        fade(stage, &[answer], true)?;
        // Preserve a readable settled answer before the next explanation.
        stage.wait(3.0)?;
        k.say(stage, "14_3")?;
        fade(stage, &[answer], false)?;
        let count = k.text(stage, "Four disjoint sets of three", 27.0, Palette::ink(), v(0., -2.8))?;
        fade(stage, &[count], true)?;

        // 14_4: left cosets of H = {e, s}; each is a rack column.
        k.say(stage, "14_4")?;
        let mut first = vec![title, count];
        first.extend(clock.iter().copied());
        for pattern in &patterns {
            first.extend(pattern.iter().copied());
        }
        for row in &rows {
            first.extend(row.iter().copied());
        }
        fade(stage, &first, false)?;
        let heading = k.text(
            stage,
            "When order matters: left cosets",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        let rack_items = rack(stage, &k)?;
        let mut shown = vec![heading];
        shown.extend(rack_items.iter().copied());
        d4g::show(stage, &shown, 0.5)?;
        k.until_sentence(stage, "14_4", 1)?;
        let h_frame = column_frame(stage, 0, Palette::hero())?;
        play!(stage; show_creation(h_frame).rt(0.6));
        let h_name = tex_at(
            stage,
            &k,
            &format!("H={}", set_tex(&FLIP_H)),
            38.0,
            Palette::hero(),
            2.0,
        )?;
        d4g::show(stage, &[h_name], 0.4)?;
        k.until_sentence(stage, "14_4", 2)?;
        let definition = tex_at(stage, &k, r"gH=\{gh:h\in H\}", 36.0, Palette::ink(), 1.1)?;
        d4g::show(stage, &[definition], 0.4)?;
        k.until_sentence(stage, "14_4", 3)?;
        let mut columns = vec![h_frame];
        let mut example = None;
        for (index, g) in [R, R2, R3].into_iter().enumerate() {
            carry_h(stage, &k, g, if index == 0 { 1.4 } else { 0.8 })?;
            let frame = column_frame(stage, g.turns, COSET_COLORS[usize::from(g.turns)]())?;
            play!(stage; show_creation(frame).rt(0.4));
            columns.push(frame);
            if index == 0 {
                let line = tex_at(
                    stage,
                    &k,
                    &format!("{}H={}", g.tex(), set_tex(&d4g::left_coset(g, &FLIP_H))),
                    34.0,
                    Palette::blue(),
                    0.2,
                )?;
                d4g::show(stage, &[line], 0.3)?;
                example = Some(line);
            }
        }
        k.until_sentence(stage, "14_4", 4)?;
        let column_note = k.text(
            stage,
            "Each left coset: one column",
            28.0,
            Palette::ink(),
            v(PANEL_X, -0.75),
        )?;
        d4g::show(stage, &[column_note], 0.4)?;

        // 14_14: right cosets cross the columns; the heading now names both.
        k.say(stage, "14_14")?;
        let mut old = vec![definition, column_note, heading];
        old.extend(example);
        d4g::hide(stage, &old, 0.3)?;
        let both = k.text(
            stage,
            "When order matters: left and right cosets",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        d4g::show(stage, &[both], 0.4)?;
        k.until_sentence(stage, "14_14", 1)?;
        let hr = tex_at(stage, &k, r"Hr=\{r,sr\}", 36.0, Palette::purple(), 1.1)?;
        d4g::show(stage, &[hr], 0.4)?;
        k.until_sentence(stage, "14_14", 2)?;
        let sr = S.mul(R);
        let rewrite = tex_at(stage, &k, &format!("sr={}", sr.tex()), 36.0, Palette::purple(), 0.3)?;
        d4g::show(stage, &[rewrite], 0.4)?;
        let first_link = link_of(stage, &d4g::right_coset(R, &FLIP_H), Palette::purple());
        play!(stage; show_creation(first_link).rt(0.8));
        k.until_sentence(stage, "14_14", 4)?;
        let mut links = vec![first_link];
        for coset in right_cosets(&FLIP_H) {
            if coset != d4g::right_coset(R, &FLIP_H) {
                let link = link_of(stage, &coset, Palette::purple());
                play!(stage; show_creation(link).rt(0.5));
                links.push(link);
            }
        }
        let cuts = k.text(
            stage,
            "Columns: left cosets. Links: right cosets",
            26.0,
            Palette::ink(),
            v(PANEL_X, -0.75),
        )?;
        d4g::show(stage, &[cuts], 0.4)?;
        k.until_sentence(stage, "14_14", 5)?;
        let differ = tex_at(
            stage,
            &k,
            r"gH\quad\text{may differ from}\quad Hg",
            32.0,
            Palette::hero(),
            -1.7,
        )?;
        d4g::show(stage, &[differ], 0.4)?;

        // 14_5: back on the clock, test representatives by the step between them.
        k.say(stage, "14_5")?;
        let mut old = rack_items.clone();
        old.extend(columns.iter().copied());
        old.extend(links);
        old.extend([both, h_name, hr, rewrite, cuts, differ]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = k.text(stage, "Same coset? Look at the step", 34.0, Palette::ink(), v(0.0, 3.2))?;
        let dial = stage.arena_mut().add(
            Circle::new()
                .radius(1.85)
                .style(stroke_style(Palette::dim(), 2.))
                .build()
                .moved_to(v(-3., 0.)),
        );
        let mut clock = vec![heading, dial];
        for n in 0..12 {
            let color = if CLOCK_H.contains(&n) {
                Palette::blue()
            } else {
                Palette::ink()
            };
            clock.push(k.text(stage, &n.to_string(), 23., color, clock_position(f64::from(n), 2.2))?);
        }
        let h_clock = tex_at(stage, &k, r"H=\{0,4,8\}", 36.0, Palette::blue(), 2.1)?;
        clock.push(h_clock);
        d4g::show(stage, &clock, 0.5)?;
        k.until_sentence(stage, "14_5", 1)?;
        let (g, kk) = SAME_COSET;
        let mut walkers = walk(stage, g, kk, Palette::green())?;
        let (text, color) = step_verdict(g, kk);
        let same = tex_at(stage, &k, &text, 36.0, color, 0.9)?;
        d4g::show(stage, &[same], 0.4)?;
        k.until_sentence(stage, "14_5", 2)?;
        d4g::hide(stage, &walkers, 0.3)?;
        let (g, kk) = OTHER_COSET;
        walkers = walk(stage, g, kk, Palette::gold())?;
        k.until_sentence(stage, "14_5", 3)?;
        let (text, color) = step_verdict(g, kk);
        let other = tex_at(stage, &k, &text, 36.0, color, 0.0)?;
        d4g::show(stage, &[other], 0.4)?;
        k.until_sentence(stage, "14_5", 4)?;
        let criterion = tex_at(stage, &k, r"g+H=k+H\iff k-g\in H", 36.0, Palette::ink(), -1.2)?;
        d4g::show(stage, &[criterion], 0.4)?;

        // 14_15: the same test in multiplication language; s·H reshuffles H.
        k.say(stage, "14_15")?;
        let mut old = clock;
        old.extend(walkers);
        old.extend([same, other, criterion]);
        d4g::hide(stage, &old, 0.5)?;
        let heading = k.text(
            stage,
            "Same coset: the correction lies in H",
            34.0,
            Palette::ink(),
            v(0.0, 3.3),
        )?;
        let mut shown = vec![heading];
        shown.extend(rack_items.iter().copied());
        shown.extend(columns.iter().copied());
        d4g::show(stage, &shown, 0.5)?;
        let line0 = tex_at(stage, &k, r"k=gh", 34.0, Palette::ink(), 2.2)?;
        d4g::show(stage, &[line0], 0.3)?;
        k.until_sentence(stage, "14_15", 1)?;
        let line1 = tex_at(stage, &k, r"h=g^{-1}k", 34.0, Palette::ink(), 1.5)?;
        let correction = R.inv().mul(RS);
        let concrete = tex_at(
            stage,
            &k,
            &format!(r"r^{{-1}}\cdot rs={}\in H", correction.tex()),
            30.0,
            Palette::blue(),
            0.85,
        )?;
        d4g::show(stage, &[line1, concrete], 0.4)?;
        k.until_sentence(stage, "14_15", 2)?;
        let line2 = tex_at(stage, &k, r"kH=g(hH)", 34.0, Palette::ink(), 0.1)?;
        d4g::show(stage, &[line2], 0.4)?;
        k.until_sentence(stage, "14_15", 3)?;
        // s·{e, s} = {s, e}: the two members trade places, and H is unchanged.
        let demo_at = [(2.6, -1.35), (4.2, -1.35)];
        let mut demo = Vec::new();
        for (&h, &at) in FLIP_H.iter().zip(&demo_at) {
            demo.push(d4g::thumb(stage, &k, h, at, 0.8)?);
        }
        d4g::show(stage, &demo, 0.3)?;
        let mut swaps = Vec::new();
        for (index, &h) in FLIP_H.iter().enumerate() {
            let landing = FLIP_H.iter().position(|&x| x == S.mul(h)).expect("sH is H");
            swaps.push((demo[index], demo_at[landing]));
        }
        d4g::glide_all(stage, &swaps, 1.2)?;
        let reshuffle = tex_at(stage, &k, r"sH=\{s,e\}=H", 30.0, Palette::hero(), -2.3)?;
        let line2b = tex_at(stage, &k, r"kH=g(hH)=gH", 34.0, Palette::green(), 0.1)?;
        d4g::show(stage, &[reshuffle], 0.3)?;
        play!(stage; fmn::animation::replacement_transform(line2, line2b).rt(0.6));
        k.until_sentence(stage, "14_15", 4)?;
        let converse = tex_at(
            stage,
            &k,
            r"k\in kH=gH\quad\Longrightarrow\quad g^{-1}k\in H",
            30.0,
            Palette::green(),
            -3.05,
        )?;
        d4g::show(stage, &[converse], 0.4)?;

        // 14_6: the relation, its three properties, and the classes.
        k.say(stage, "14_6")?;
        let mut old = demo;
        old.extend([line0, line1, concrete, line2b, reshuffle, converse]);
        d4g::hide(stage, &old, 0.4)?;
        let relation = tex_at(
            stage,
            &k,
            r"g\sim k\quad\Longleftrightarrow\quad g^{-1}k\in H",
            34.0,
            Palette::ink(),
            2.2,
        )?;
        let refl = tex_at(stage, &k, r"g^{-1}g=e\in H", 32.0, Palette::blue(), 1.2)?;
        let sym = tex_at(stage, &k, r"(g^{-1}k)^{-1}=k^{-1}g\in H", 32.0, Palette::hero(), 0.3)?;
        let trans = tex_at(
            stage,
            &k,
            r"(g^{-1}k)(k^{-1}m)=g^{-1}m\in H",
            32.0,
            Palette::green(),
            -0.6,
        )?;
        d4g::show(stage, &[relation], 0.4)?;
        k.until_sentence(stage, "14_6", 2)?;
        d4g::show(stage, &[refl], 0.4)?;
        k.until_sentence(stage, "14_6", 3)?;
        d4g::show(stage, &[sym], 0.4)?;
        k.until_sentence(stage, "14_6", 4)?;
        d4g::show(stage, &[trans], 0.4)?;
        k.until_sentence(stage, "14_6", 5)?;
        let mut pulses: Vec<Box<dyn Animation>> = Vec::new();
        for &frame in &columns {
            let a = fmn::animation::indicate(stage.arena_mut(), frame, 1.05, None)?.rt(0.7);
            pulses.push(stage.prepare(a)?);
        }
        let pulses = explainer_kit::lagged(stage, pulses, 0.2)?;
        play!(stage; pulses);
        let classes = k.text(
            stage,
            "Cosets: classes that never overlap",
            28.0,
            Palette::ink(),
            v(PANEL_X, -1.6),
        )?;
        d4g::show(stage, &[classes], 0.4)?;

        // 14_7: every coset is a copy of H.
        k.say(stage, "14_7")?;
        d4g::hide(stage, &[relation, refl, sym, trans, classes], 0.4)?;
        let carry_name = tex_at(
            stage,
            &k,
            r"H\longrightarrow gH,\qquad h\longmapsto gh",
            34.0,
            Palette::blue(),
            2.3,
        )?;
        d4g::show(stage, &[carry_name], 0.4)?;
        k.until_sentence(stage, "14_7", 1)?;
        // The clock callback: the triangle 0, 4, 8 carried one step.
        let shift = stage.add_value_tracker(0.0);
        let small = |value: f64| {
            let angle = std::f64::consts::TAU * value / 12.;
            v(PANEL_X + 1.05 * angle.sin(), 0.55 + 1.05 * angle.cos())
        };
        let small_dial = stage.arena_mut().add(
            Circle::new()
                .radius(1.05)
                .style(stroke_style(Palette::dim(), 2.0))
                .build()
                .moved_to(v(PANEL_X, 0.55)),
        );
        let triangle = stage.always_redraw(move |arena| {
            let t = arena.tracker_value(shift).unwrap_or(0.0);
            arena.add(
                Polygon::new(CLOCK_H.map(|n| small(f64::from(n) + t)))
                    .style(stroke_style(Palette::blue(), 4.0))
                    .build(),
            )
        });
        d4g::show(stage, &[small_dial], 0.3)?;
        stage.add_to_scene(triangle)?;
        play!(stage; tween(shift, 1.0, 1.0)?);
        for g in [R, R2, R3] {
            carry_h(stage, &k, g, 0.7)?;
        }
        k.until_sentence(stage, "14_7", 2)?;
        let cancel = tex_at(
            stage,
            &k,
            r"gh_1=gh_2\quad\Longrightarrow\quad h_1=h_2",
            32.0,
            Palette::green(),
            -1.0,
        )?;
        d4g::show(stage, &[cancel], 0.4)?;
        k.until_sentence(stage, "14_7", 3)?;
        let copy = k.text(
            stage,
            "Each coset: a perfect copy of H",
            28.0,
            Palette::ink(),
            v(PANEL_X, -1.9),
        )?;
        d4g::show(stage, &[copy], 0.4)?;

        // 14_8: count. Lagrange; the clock; the empty shelves explained.
        k.say(stage, "14_8")?;
        d4g::hide(stage, &[carry_name, small_dial, triangle, cancel, copy], 0.4)?;
        let cosets = left_cosets(&FLIP_H);
        k.until_sentence(stage, "14_8", 1)?;
        let mut sizes = Vec::new();
        for coset in &cosets {
            let x = rack_at(coset[0]).0;
            sizes.push(k.text(stage, &coset.len().to_string(), 28.0, Palette::green(), v(x, -2.15))?);
        }
        d4g::show(stage, &sizes, 0.4)?;
        k.until_sentence(stage, "14_8", 2)?;
        let square_count = tex_at(
            stage,
            &k,
            &format!(r"{}={}\cdot{}", ALL.len(), cosets.len(), FLIP_H.len()),
            40.0,
            Palette::green(),
            1.6,
        )?;
        let lagrange = tex_at(stage, &k, r"|G|=k\,|H|", 44.0, Palette::green(), 2.5)?;
        d4g::show(stage, &[square_count, lagrange], 0.4)?;
        k.until_sentence(stage, "14_8", 3)?;
        let name = k.text(stage, "Lagrange's theorem", 30.0, Palette::ink(), v(PANEL_X, 0.85))?;
        d4g::show(stage, &[name], 0.4)?;
        k.until_sentence(stage, "14_8", 4)?;
        let divides = tex_at(stage, &k, r"|H|\ \text{divides}\ |G|", 34.0, Palette::ink(), 0.15)?;
        d4g::show(stage, &[divides], 0.4)?;
        k.until_sentence(stage, "14_8", 5)?;
        let clock_cosets = (0..12)
            .map(|n| n % CLOCK_H[1])
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let clock_count = tex_at(
            stage,
            &k,
            &format!(r"12={}\cdot{}", clock_cosets, CLOCK_H.len()),
            36.0,
            Palette::hero(),
            -0.6,
        )?;
        d4g::show(stage, &[clock_count], 0.4)?;
        k.until_sentence(stage, "14_8", 6)?;
        let mut shelf = Vec::new();
        let mut strikes = Vec::new();
        let order = ALL.len();
        for size in 1..=order {
            let x = PANEL_X - 2.8 + 0.8 * (size as f64 - 1.0);
            let color = if order.is_multiple_of(size) {
                Palette::green()
            } else {
                Palette::gold()
            };
            shelf.push(k.text(stage, &size.to_string(), 32.0, color, v(x, -1.6))?);
            if !order.is_multiple_of(size) {
                strikes.push(
                    stage.arena_mut().add(
                        Line::new(v(x - 0.22, -1.82), v(x + 0.22, -1.38))
                            .style(stroke_style(Palette::gold(), 4.0))
                            .build()?,
                    ),
                );
            }
        }
        let possible = k.text(
            stage,
            "possible subgroup sizes",
            24.0,
            Palette::dim(),
            v(PANEL_X, -2.25),
        )?;
        shelf.push(possible);
        d4g::show(stage, &shelf, 0.5)?;
        k.until_sentence(stage, "14_8", 7)?;
        let mut cross: Vec<Box<dyn Animation>> = Vec::new();
        for &strike in &strikes {
            cross.push(stage.prepare(show_creation(strike).rt(0.4))?);
        }
        let cross = explainer_kit::lagged(stage, cross, 0.3)?;
        play!(stage; cross);

        // 14_9 .. 14_12: the recorded beats, unchanged in substance.
        k.say(stage, "14_9")?;
        let mut old = rack_items;
        old.extend(columns);
        old.extend(sizes);
        old.extend(shelf);
        old.extend(strikes);
        old.extend([heading, square_count, name, divides, clock_count]);
        d4g::hide(stage, &old, 0.5)?;
        let example = k.tex(stage, r"12=4\cdot3", 42.0, Palette::hero(), &[], v(0.0, -2.6))?;
        let lagrange_center = k.tex(stage, r"|G|=k\,|H|", 46.0, Palette::green(), &[], v(0.0, 2.8))?;
        play!(stage; fmn::animation::replacement_transform(lagrange, lagrange_center).rt(0.6));
        let order_tex = k.tex(
            stage,
            r"\operatorname{ord}(g)=|\langle g\rangle|\mid |G|",
            36.0,
            Palette::blue(),
            &[],
            v(0.0, -1.4),
        )?;
        fade(stage, &[order_tex, example], true)?;
        k.say(stage, "14_10")?;
        fade(stage, &[lagrange_center, example, order_tex], false)?;
        let a4 = k.tex(
            stage,
            r"|A_4|=12,\qquad 6\mid12",
            38.0,
            Palette::ink(),
            &[],
            v(0.0, 1.5),
        )?;
        let missing = k.text(stage, "No subgroup of order six", 32.0, Palette::hero(), v(0.0, 0.3))?;
        let reason = k.text(
            stage,
            "Divisibility is necessary, not sufficient",
            27.0,
            Palette::blue(),
            v(0.0, -0.8),
        )?;
        fade(stage, &[a4, missing], true)?;
        k.until_sentence(stage, "14_10", 4)?;
        fade(stage, &[reason], true)?;
        k.say(stage, "14_11")?;
        fade(stage, &[a4, missing, reason], false)?;
        let caution = k.text(stage, "Can we multiply cosets?", 36.0, Palette::hero(), v(0.0, 1.0))?;
        let condition = k.text(
            stage,
            "First: the answer must not depend on representatives",
            29.0,
            Palette::ink(),
            v(0.0, -0.2),
        )?;
        fade(stage, &[caution, condition], true)?;
        k.say(stage, "14_12")?;
        k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_representative_tests_match_the_narration() {
        assert!(same_clock_coset(SAME_COSET.0, SAME_COSET.1));
        assert_eq!((SAME_COSET.1 - SAME_COSET.0).rem_euclid(12), 8);
        assert!(!same_clock_coset(OTHER_COSET.0, OTHER_COSET.1));
        assert_eq!((OTHER_COSET.1 - OTHER_COSET.0).rem_euclid(12), 6);
        // The test agrees with comparing the shifted sets themselves.
        for g in 0..12 {
            for k in 0..12 {
                let mut a: Vec<i32> = CLOCK_H.iter().map(|h| (g + h) % 12).collect();
                let mut b: Vec<i32> = CLOCK_H.iter().map(|h| (k + h) % 12).collect();
                a.sort();
                b.sort();
                assert_eq!(a == b, same_clock_coset(g, k));
            }
        }
    }

    #[test]
    fn left_cosets_of_the_flip_subgroup_are_the_rack_columns() {
        let cosets = left_cosets(&FLIP_H);
        assert_eq!(cosets.len(), 4);
        for coset in &cosets {
            assert_eq!(coset.len(), FLIP_H.len());
            assert!(coset.iter().all(|g| g.turns == coset[0].turns));
        }
        // carry_h lands g·h for each member: the column of g.
        for g in [R, R2, R3] {
            for h in FLIP_H {
                assert_eq!(rack_at(g.mul(h)).0, rack_at(g).0);
            }
        }
        assert_eq!(d4g::left_coset(R, &FLIP_H), vec![R, RS]);
    }

    #[test]
    fn right_cosets_cross_the_columns_exactly_where_narrated() {
        assert_eq!(S.mul(R), crate::d4_group::R3S);
        assert_eq!(d4g::right_coset(R, &FLIP_H), vec![R, crate::d4_group::R3S]);
        let rights = right_cosets(&FLIP_H);
        assert_eq!(rights.len(), 4);
        assert_ne!(rights, left_cosets(&FLIP_H));
        assert!(rights.iter().all(|c| c.len() == 2));
    }

    /// `[x0, x1, y0, y1]` boxes overlap by more than rounding.
    fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
        a[0] < b[1] - 1e-9 && b[0] < a[1] - 1e-9 && a[2] < b[3] - 1e-9 && b[2] < a[3] - 1e-9
    }

    #[test]
    fn right_coset_links_touch_their_tiles_and_cross_no_tile_or_name() {
        // A top-row name: 24 pt at 0.68 below the tile centre, at most "r³" wide.
        let (name_half_w, name_half_h) = (0.15, 0.1);
        let routes: Vec<Vec<(f64, f64)>> = right_cosets(&FLIP_H).iter().map(|c| link_route(c.as_slice())).collect();
        // The left-coset column frames are 1.3 wide; no link runs along a border.
        let borders: Vec<f64> = (0..4u8)
            .flat_map(|t| [-0.65, 0.65].map(|side| RACK_X0 + RACK_DX * f64::from(t) + side))
            .collect();
        let mut segments = Vec::new();
        for (coset, route) in right_cosets(&FLIP_H).iter().zip(&routes) {
            // Rotation first, then the flip: the link runs from the top row down.
            assert!(!coset[0].flip && coset[1].flip, "{coset:?}");
            let (top, bottom) = (rack_at(coset[0]), rack_at(coset[1]));
            let (first, last) = (route[0], route[route.len() - 1]);
            assert!((first.1 + LINK_CAP - (top.1 - TILE_HALF)).abs() < 1e-9);
            assert!((last.1 - LINK_CAP - (bottom.1 + TILE_HALF)).abs() < 1e-9);
            assert!((first.0 - top.0).abs() < TILE_HALF && (last.0 - bottom.0).abs() < TILE_HALF);
            for pair in route.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                assert!(a.0 == b.0 || a.1 == b.1, "segments are vertical or horizontal");
                if a.0 == b.0 {
                    assert!(
                        borders.iter().all(|x| (x - a.0).abs() > 0.1),
                        "{coset:?} runs along a column frame"
                    );
                }
                let stroke = [
                    a.0.min(b.0) - LINK_CAP,
                    a.0.max(b.0) + LINK_CAP,
                    a.1.min(b.1) - LINK_CAP,
                    a.1.max(b.1) + LINK_CAP,
                ];
                for g in ALL {
                    let (x, y) = rack_at(g);
                    let tile = [x - TILE_HALF, x + TILE_HALF, y - TILE_HALF, y + TILE_HALF];
                    assert!(!overlaps(stroke, tile), "{coset:?} crosses the tile of {g:?}");
                    let name = [
                        x - name_half_w,
                        x + name_half_w,
                        y - 0.68 - name_half_h,
                        y - 0.68 + name_half_h,
                    ];
                    assert!(!overlaps(stroke, name), "{coset:?} crosses the name of {g:?}");
                }
                segments.push((a, b));
            }
        }
        // Two links may cross, but never run along each other.
        for (i, &(a, b)) in segments.iter().enumerate() {
            for &(c, d) in &segments[i + 1..] {
                let same_x = a.0 == b.0 && c.0 == d.0 && a.0 == c.0;
                let same_y = a.1 == b.1 && c.1 == d.1 && a.1 == c.1;
                if same_x {
                    assert!(a.1.max(b.1) < c.1.min(d.1) || c.1.max(d.1) < a.1.min(b.1));
                }
                if same_y {
                    assert!(a.0.max(b.0) < c.0.min(d.0) || c.0.max(d.0) < a.0.min(b.0));
                }
            }
        }
        // The two links that change column use different lanes.
        let lanes: Vec<f64> = routes.iter().filter(|r| r.len() == 4).map(|r| r[1].1).collect();
        assert_eq!(lanes.len(), 2);
        assert!(lanes[0] != lanes[1]);
    }

    #[test]
    fn correction_and_reshuffle_examples_hold() {
        assert_eq!(R.inv().mul(RS), S);
        let mut sh = d4g::left_coset(S, &FLIP_H);
        sh.sort();
        let mut h = FLIP_H.to_vec();
        h.sort();
        assert_eq!(sh, h);
        assert_eq!(S.mul(E), S);
        assert_eq!(S.mul(S), E);
        // In every case: gH = kH exactly when g⁻¹k ∈ H.
        for g in ALL {
            for k in ALL {
                assert_eq!(
                    d4g::left_coset(g, &FLIP_H) == d4g::left_coset(k, &FLIP_H),
                    FLIP_H.contains(&g.inv().mul(k))
                );
            }
        }
    }

    #[test]
    fn lagrange_counts_and_the_struck_sizes() {
        assert_eq!(ALL.len(), left_cosets(&FLIP_H).len() * FLIP_H.len());
        let clock_cosets = (0..12)
            .map(|n| n % CLOCK_H[1])
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        assert_eq!(12, clock_cosets * CLOCK_H.len());
        let struck: Vec<usize> = (1..=8).filter(|d| 8 % d != 0).collect();
        assert_eq!(struck, vec![3, 5, 6, 7]);
        // The struck sizes are exactly the empty shelves of chapter 08.
        let sizes: std::collections::BTreeSet<usize> = d4g::subgroups().iter().map(|h| h.len()).collect();
        assert!(struck.iter().all(|d| !sizes.contains(d)));
        assert!(sizes.iter().all(|d| 8 % d == 0));
    }
}
