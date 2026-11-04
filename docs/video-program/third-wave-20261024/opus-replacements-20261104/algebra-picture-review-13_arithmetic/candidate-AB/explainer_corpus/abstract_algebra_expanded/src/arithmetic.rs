//! Action arithmetic: rigid worked examples, word reduction and regrouping;
//! then (13_9 to 13_12) packaging shown on two tiles and on a number passing
//! through three machines, the four properties as cards, and a small
//! calculation for each property on the rack of eight moves.
use crate::axioms::{mark, pulse, recolor, until_phrase};
use crate::d4_group::{self as d4g, ALL, E, Elem, R, R2, S};
use crate::{
    tile_actor,
    tile_model::{self, Matrix},
};
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, play, stroke_style, tween, v};
use fmn::prelude::*;

#[derive(Clone, Copy)]
enum Move {
    Turn(f64),
    Flip,
}
const LEFT: [Move; 9] = [
    Move::Turn(1.),
    Move::Turn(1.),
    Move::Turn(1.),
    Move::Turn(1.),
    Move::Flip,
    Move::Flip,
    Move::Flip,
    Move::Turn(1.),
    Move::Flip,
];
const RIGHT: [Move; 5] = [Move::Turn(1.), Move::Flip, Move::Flip, Move::Turn(-1.), Move::Turn(-1.)];
/// 13_1's long route, which ends where one quarter-turn ends (a test checks).
const LONG_ROUTE: [Move; 5] = [Move::Turn(1.), Move::Flip, Move::Flip, Move::Turn(1.), Move::Turn(-1.)];
fn pose(actions: &[Move], t: f64) -> Matrix {
    actions
        .iter()
        .enumerate()
        .fold(tile_model::IDENTITY, |state, (i, action)| {
            let fraction = (t - i as f64).clamp(0., 1.);
            let next = match action {
                Move::Turn(n) => tile_model::turn(n * fraction),
                Move::Flip => tile_model::flip(fraction),
            };
            tile_model::compose(next, state)
        })
}
fn advance(stage: &mut Stage<'_>, tracker: Mob, to: f64) -> fmn::Result<()> {
    let movement = tween(tracker, to, 1.7)?;
    let movement = stage.prepare(movement)?;
    stage.play_prepared(vec![movement])?;
    stage.wait(0.5)?;
    Ok(())
}
fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut actions = Vec::new();
    for &mob in mobs {
        if !incoming {
            stage.clear_updaters(mob, true);
        }
        let a = if incoming {
            fade_in(stage.arena_mut(), mob, ORIGIN, 1.)?
        } else {
            fade_out(stage.arena_mut(), mob, ORIGIN, 1.)?
        };
        actions.push(stage.prepare(a.rt(0.4))?);
    }
    stage.play_prepared(actions)?;
    Ok(())
}
fn caption(stage: &mut Stage<'_>, kit: &Kit, text: &str, x: f64, y: f64) -> fmn::Result<Mob> {
    let math = match text {
        "e" | "r" | "s" | "s r" | "r s" | "s r s" => Some(text),
        "r²" => Some("r^2"),
        "r³" => Some("r^3"),
        "r⁴ = e" => Some("r^4=e"),
        "s² = e" => Some("s^2=e"),
        "r⁻¹" => Some("r^{-1}"),
        "s r s = r⁻¹ = r³" => Some("srs=r^{-1}=r^3"),
        "s r = r⁻¹ s" => Some("sr=r^{-1}s"),
        "rᵏ  or  rᵏ s" => Some(r"r^k\quad\mathrm{or}\quad r^k s"),
        "r³ r = e" => Some("r^3r=e"),
        _ => None,
    };
    let mob = if let Some(math) = math {
        kit.tex(stage, math, 36., Palette::ink(), &[], v(x, y))?
    } else {
        kit.text(stage, text, 30., Palette::ink(), v(x, y))?
    };
    fade(stage, &[mob], true)?;
    Ok(mob)
}
fn replace(stage: &mut Stage<'_>, kit: &Kit, old: Mob, text: &str, x: f64, y: f64) -> fmn::Result<Mob> {
    fade(stage, &[old], false)?;
    caption(stage, kit, text, x, y)
}
/// Run a move and rename the tile. The old name leaves first, so no frame shows
/// a new pose above the old name. It takes the same 3 s as `advance` then `replace`.
fn step(
    stage: &mut Stage<'_>,
    kit: &Kit,
    tracker: Mob,
    to: f64,
    old: Mob,
    text: &str,
    at: (f64, f64),
) -> fmn::Result<Mob> {
    fade(stage, &[old], false)?;
    advance(stage, tracker, to)?;
    caption(stage, kit, text, at.0, at.1)
}
fn grouping(stage: &mut Stage<'_>, left: f64, right: f64, y: f64) -> fmn::Result<Mob> {
    let mob = stage.arena_mut().add(
        Polygon::new([
            v(left, y - 0.45),
            v(right, y - 0.45),
            v(right, y + 0.45),
            v(left, y + 0.45),
        ])
        .style(stroke_style(Palette::gold(), 3.))
        .build(),
    );
    fade(stage, &[mob], true)?;
    Ok(mob)
}
pub struct Arithmetic {
    pub kit: Kit,
}
impl SceneConstruct for Arithmetic {
    fn name(&self) -> &str {
        "13_arithmetic"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "13_1")?;
        let heading = caption(stage, &kit, "Arithmetic with actions", 0., 3.05)?;
        let left = stage.add_value_tracker(0.);
        let right = stage.add_value_tracker(0.);
        // Two histories, one destination: a long route and one quarter-turn.
        let long_route = stage.add_value_tracker(0.);
        let short_route = stage.add_value_tracker(0.);
        let mut routes = tile_actor::add_tile(stage, &kit, long_route, -3.1, 0., 1.25, |t| pose(&LONG_ROUTE, t))?;
        routes.extend(tile_actor::add_tile(
            stage,
            &kit,
            short_route,
            3.1,
            0.,
            1.25,
            tile_model::turn,
        )?);
        kit.until_sentence(stage, "13_1", 1)?;
        perform(stage, long_route, LONG_ROUTE.len() as f64, 3.4, &[])?;
        perform(stage, short_route, 1., LONE_RT, &[])?;
        // The eight arrangements, unnamed: their names arrive in 13_2 and 13_3.
        until_phrase(stage, &kit, "13_1", 2, EIGHT_PHRASE)?;
        fade(stage, &routes, false)?;
        let eight = bare_rack(stage, &kit)?;
        appear(stage, &eight, 0.3, 0.1)?;
        kit.say(stage, "13_2")?;
        fade(stage, &eight, false)?;
        // Separate demonstration actors leave the worked-calculation histories intact.
        let demo_turn = stage.add_value_tracker(0.);
        let demo_flip = stage.add_value_tracker(0.);
        let demo_left = tile_actor::add_tile(stage, &kit, demo_turn, -3.1, 0., 1.25, tile_model::turn)?;
        let demo_right = tile_actor::add_tile(stage, &kit, demo_flip, 3.1, 0., 1.25, |t| {
            if t <= 1. {
                tile_model::flip(t)
            } else {
                tile_model::compose(tile_model::turn(t - 1.), tile_model::flip(1.))
            }
        })?;
        // The new symbols first name actions we can actually see.
        advance(stage, demo_turn, 1.)?;
        let demo_r = caption(stage, &kit, "r", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_2", 1)?;
        let mut demo_s = caption(stage, &kit, "s", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        advance(stage, demo_flip, 1.)?;
        kit.until_sentence(stage, "13_2", 2)?;
        let convention = caption(stage, &kit, "Rightmost move first", 0., 2.3)?;
        kit.until_sentence(stage, "13_2", 4)?;
        // The right tile has already flipped; append the turn on screen.
        demo_s = step(stage, &kit, demo_flip, 2., demo_s, "r s", RIGHT_LABEL)?;
        kit.hold(stage)?;
        let mut leaving = demo_left;
        leaving.extend(demo_right);
        leaving.extend([demo_r, demo_s]);
        fade(stage, &leaving, false)?;
        let left_objects = tile_actor::add_tile(stage, &kit, left, -3.1, 0., 1.25, |t| pose(&LEFT, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, right, 3.1, 0., 1.25, |t| pose(&RIGHT, t))?;
        // `e` is not defined until 13_3's last sentence; until then this pose is "start".
        let mut left_label = caption(stage, &kit, "start", LEFT_LABEL.0, LEFT_LABEL.1)?;
        let mut right_label = caption(stage, &kit, "start", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.say(stage, "13_3")?;
        for (index, text) in ["r", "r²", "r³"].into_iter().enumerate() {
            left_label = step(stage, &kit, left, (index + 1) as f64, left_label, text, LEFT_LABEL)?;
            stage.wait(1.)?;
        }
        // Four turns bring the tile home; the name e waits for the sentence defining it.
        fade(stage, &[left_label], false)?;
        advance(stage, left, 4.)?;
        kit.until_sentence(stage, "13_3", 6)?;
        left_label = caption(stage, &kit, "r⁴ = e", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.say(stage, "13_4")?;
        kit.until_sentence(stage, "13_4", 1)?;
        left_label = step(stage, &kit, left, 5., left_label, "s", LEFT_LABEL)?;
        stage.wait(1.)?;
        left_label = step(stage, &kit, left, 6., left_label, "s² = e", LEFT_LABEL)?;
        kit.until_sentence(stage, "13_4", 5)?;
        right_label = replace(stage, &kit, right_label, "r⁴ = e", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.say(stage, "13_5")?;
        // Each spoken pair of moves runs as one motion, named when it stops.
        kit.until_sentence(stage, "13_5", 1)?;
        fade(stage, &[right_label], false)?;
        perform(stage, right, 2., PACKAGE_RT, &[])?;
        right_label = caption(stage, &kit, "s r", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        kit.until_sentence(stage, "13_5", 2)?;
        fade(stage, &[left_label], false)?;
        perform(stage, left, 8., PACKAGE_RT, &[])?;
        left_label = caption(stage, &kit, "r s", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.say(stage, "13_6")?;
        // Both tiles start again: the left performs s r s as spoken, the right
        // one quarter-turn the other way; then their corners can be compared.
        kit.until_sentence(stage, "13_6", 1)?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label]);
        fade(stage, &leaving, false)?;
        let conjugate = stage.add_value_tracker(0.);
        let opposite = stage.add_value_tracker(0.);
        let left_objects = tile_actor::add_tile(stage, &kit, conjugate, -3.1, 0., 1.25, |t| pose(&SEQUENCE, t))?;
        let right_objects = tile_actor::add_tile(stage, &kit, opposite, 3.1, 0., 1.25, |t| tile_model::turn(-t))?;
        perform(stage, conjugate, 3., PACKAGE_RT, &[])?;
        left_label = caption(stage, &kit, "s r s", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_6", 3)?;
        perform(stage, opposite, 1., LONE_RT, &[])?;
        right_label = caption(stage, &kit, "r⁻¹", RIGHT_LABEL.0, RIGHT_LABEL.1)?;
        // The relation waits for the words that state it.
        until_phrase(stage, &kit, "13_6", 4, RELATION_PHRASE)?;
        let relation = caption(stage, &kit, "s r s = r⁻¹ = r³", 0., -2.8)?;
        kit.say(stage, "13_7")?;
        let mut leaving = left_objects;
        leaving.extend(right_objects);
        leaving.extend([left_label, right_label, relation]);
        fade(stage, &leaving, false)?;
        // A concrete word, reduced by the rules as the voice states them. Keep the
        // factors separate: the highlighted subword has explicit positions, rather
        // than guessing its location inside a TeX object.
        let mut factors = Vec::new();
        for (x, text) in [(-2., "s"), (-1., "r"), (0., "s"), (1., "r²"), (2., "s")] {
            factors.push(caption(stage, &kit, text, x, 0.6)?);
        }
        // Move the flip past the turn, reversing the turn.
        kit.until_sentence(stage, "13_7", 1)?;
        let rule = caption(stage, &kit, "s r = r⁻¹ s", 0., -0.7)?;
        let bracket = grouping(stage, -2.4, -0.6, 0.6)?;
        stage.wait(1.6)?;
        fade(stage, &[bracket, factors[0], factors[1]], false)?;
        let inverse = caption(stage, &kit, "r⁻¹", -2., 0.6)?;
        let moved = caption(stage, &kit, "s", -1., 0.6)?;
        // The pair of flips cancels; then the two turns combine.
        kit.until_sentence(stage, "13_7", 2)?;
        let pair = grouping(stage, -1.4, 0.4, 0.6)?;
        stage.wait(1.)?;
        fade(stage, &[pair, moved, factors[2]], false)?;
        d4g::glide(stage, inverse, (0., 0.6), 0.4)?;
        let combine = grouping(stage, -0.5, 1.5, 0.6)?;
        stage.wait(0.8)?;
        fade(stage, &[combine, inverse, factors[3]], false)?;
        let reduced = caption(stage, &kit, "r", 1., 0.6)?;
        kit.until_sentence(stage, "13_7", 3)?;
        let normal = caption(stage, &kit, "rᵏ  or  rᵏ s", 0., -1.6)?;
        // Those two forms are the rack's two rows.
        kit.until_sentence(stage, "13_7", 4)?;
        fade(stage, &[reduced, factors[4], rule], false)?;
        d4g::glide(stage, normal, (0., 1.25), 0.4)?;
        let moves = rack(stage, &kit, MOVES_AT, MOVES_STEP, 0.8)?;
        appear(stage, &moves.slots, 0.3, 0.08)?;
        let row_frames = [row_frame(stage, 0)?, row_frame(stage, 1)?];
        d4g::show(stage, &row_frames, 0.35)?;
        kit.say(stage, "13_8")?;
        // The rack stays while the voice talks about it.
        fade(stage, &[row_frames[0], row_frames[1], normal], false)?;
        // The motions fit the sentences that name them.
        kit.until_sentence(stage, "13_8", 2)?;
        d4g::hide(stage, &moves.slots, 0.4)?;
        let undo = stage.add_value_tracker(0.);
        let undo_objects = tile_actor::add_tile(stage, &kit, undo, 0., 0., 1.25, tile_model::turn)?;
        perform(stage, undo, 1., 0.8, &[])?;
        perform(stage, undo, 4., 1.4, &[])?;
        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
        undo_label = replace(stage, &kit, undo_label, "Half-turn, then half-turn", 0., BELOW_TURNING)?;
        perform(stage, undo, 6., 0.7, &[])?;
        perform(stage, undo, 8., 0.7, &[])?;
        kit.until_sentence(stage, "13_8", 4)?;
        fade(stage, &undo_objects, false)?;
        let flip_back = stage.add_value_tracker(0.);
        let flip_objects = tile_actor::add_tile(stage, &kit, flip_back, 0., 0., 1.25, |t| {
            if t <= 1. {
                tile_model::flip(t)
            } else {
                tile_model::compose(tile_model::flip(t - 1.), tile_model::flip(1.))
            }
        })?;
        undo_label = replace(stage, &kit, undo_label, "Flip, then flip", 0., BELOW_TURNING)?;
        perform(stage, flip_back, 1., 1., &[])?;
        perform(stage, flip_back, 2., 1., &[])?;
        kit.say(stage, "13_9")?;
        let mut leaving = flip_objects.clone();
        leaving.extend([undo_label, convention]);
        d4g::hide(stage, &leaving, 0.4)?;
        let [w0, w1, w2] = WORD.map(Elem::tex);
        let mut tokens = Vec::new();
        for (x, text) in [(TOKEN_X[0], w0), (TOKEN_X[1], w1), (TOKEN_X[2], w2)] {
            tokens.push(kit.tex(stage, text, 36., Palette::ink(), &[], v(x, 1.8))?);
        }
        d4g::show(stage, &tokens, 0.4)?;
        // Both tiles perform the same word, rightmost move first; the
        // packaging decides only which two moves run as one instruction.
        let grouped_left = stage.add_value_tracker(0.);
        let grouped_right = stage.add_value_tracker(0.);
        let grouped_left_objects =
            tile_actor::add_tile(stage, &kit, grouped_left, -3., -0.2, 0.8, |t| pose(&SEQUENCE, t))?;
        let grouped_right_objects =
            tile_actor::add_tile(stage, &kit, grouped_right, 3., -0.2, 0.8, |t| pose(&SEQUENCE, t))?;
        // Below the turning corners' reach (1.25 under each tile's centre).
        let left_package = kit.tex(
            stage,
            &format!("({w0}{w1}){w2}"),
            32.,
            Palette::ink(),
            &[],
            v(-3., PACKAGE_Y),
        )?;
        let right_package = kit.tex(
            stage,
            &format!("{w0}({w1}{w2})"),
            32.,
            Palette::ink(),
            &[],
            v(3., PACKAGE_Y),
        )?;
        d4g::show(stage, &[left_package, right_package], 0.4)?;
        kit.until_sentence(stage, "13_9", 1)?;
        let (a, b) = SWAP;
        let swapped = format!(r"{}{}\neq {}{}", a.tex(), b.tex(), b.tex(), a.tex());
        let swap = kit.tex(stage, &swapped, 34., Palette::red(), &[], v(0., NOTE_Y))?;
        d4g::show(stage, &[swap], 0.4)?;
        kit.until_sentence(stage, "13_9", 2)?;
        d4g::hide(stage, &[swap], 0.3)?;
        let note = caption(stage, &kit, "Same sequence; different packaging", 0., NOTE_Y)?;
        // (sr)s: the lone rightmost s first, then the package sr in one motion.
        let first = grouping(stage, TOKEN_X[0] - 0.45, TOKEN_X[1] + 0.35, 1.8)?;
        perform(stage, grouped_left, 1., LONE_RT, &[tokens[2]])?;
        perform(stage, grouped_left, 3., PACKAGE_RT, &[first])?;
        // s(rs): the first two moves performed are the package, as the line says.
        fade(stage, &[first], false)?;
        let last = grouping(stage, TOKEN_X[1] - 0.35, TOKEN_X[2] + 0.45, 1.8)?;
        until_phrase(stage, &kit, "13_9", 3, GROUPING_PHRASE)?;
        perform(stage, grouped_right, 2., PACKAGE_RT, &[last])?;
        perform(stage, grouped_right, 3., LONE_RT, &[tokens[0]])?;
        kit.until_sentence(stage, "13_9", 4)?;
        let law = kit.tex(
            stage,
            &format!("({w0}{w1}){w2}={w0}({w1}{w2})"),
            34.,
            Palette::green(),
            &[],
            v(0., NOTE_Y),
        )?;
        d4g::hide(stage, &[note], 0.3)?;
        d4g::show(stage, &[law], 0.4)?;
        pulse(stage, &[left_package, right_package])?;
        kit.until_sentence(stage, "13_9", 5)?;
        let reorder = kit.tex(stage, &swapped, 34., Palette::red(), &[], v(1.8, NOTE_Y))?;
        d4g::glide(stage, law, (-1.8, NOTE_Y), 0.4)?;
        d4g::show(stage, &[reorder], 0.4)?;

        kit.say(stage, "13_10")?;
        let mut leaving = tokens;
        leaving.extend(grouped_left_objects);
        leaving.extend(grouped_right_objects);
        leaving.extend([last, law, reorder, left_package, right_package]);
        d4g::hide(stage, &leaving, 0.4)?;
        // Any object: one number. Three machines, innermost (h) on the left.
        let values = trail(INPUT);
        until_phrase(stage, &kit, "13_10", 0, OBJECT_PHRASE)?;
        let mut value = chip(stage, &kit, values[0])?;
        d4g::show(stage, &[value.disc, value.number], 0.35)?;
        until_phrase(stage, &kit, "13_10", 0, FUNCTIONS_PHRASE)?;
        let mut machines = Vec::new();
        for (x, rule) in MACHINE_X.into_iter().zip(machine_rules()) {
            machines.push(frame(stage, (x, MACHINE_Y), (2.0, 1.0), Palette::ink(), 2.5)?);
            machines.push(kit.tex(stage, &rule, 30., Palette::ink(), &[], v(x, RULE_Y))?);
        }
        appear(stage, &machines, 0.4, 0.15)?;
        kit.until_sentence(stage, "13_10", 1)?;
        // Package the first two applications: the value still visits h, then g.
        let first_two = package(stage, FIRST_TWO_SPAN)?;
        let rows = [
            trail_row(stage, &kit, values, ROW_Y[0], FIRST_TWO_TEX)?,
            trail_row(stage, &kit, values, ROW_Y[1], LAST_TWO_TEX)?,
        ];
        d4g::show(stage, &[first_two, rows[0].items[0], rows[0].label], 0.4)?;
        run(stage, &kit, &mut value, values, &rows[0], 0.6, 0.4)?;
        until_phrase(stage, &kit, "13_10", 1, FIRST_TWO_PHRASE)?;
        let first_bracket = trail_box(stage, FIRST_TWO_TRAIL, ROW_Y[0])?;
        d4g::show(stage, &[first_bracket], 0.35)?;
        pulse(stage, &[first_two])?;
        // A fresh input waits at the door; the spent one leaves.
        let mut second = chip(stage, &kit, values[0])?;
        play!(stage;
            fade_out(stage.arena_mut(), value.disc, ORIGIN, 1.)?.rt(0.35),
            fade_out(stage.arena_mut(), value.number, ORIGIN, 1.)?.rt(0.35),
            fade_in(stage.arena_mut(), second.disc, ORIGIN, 1.)?.rt(0.35),
            fade_in(stage.arena_mut(), second.number, ORIGIN, 1.)?.rt(0.35)
        );
        until_phrase(stage, &kit, "13_10", 1, LAST_TWO_PHRASE)?;
        let last_two = package(stage, LAST_TWO_SPAN)?;
        play!(stage;
            fmn::animation::replacement_transform(first_two, last_two).rt(0.5),
            fade_in(stage.arena_mut(), rows[1].items[0], ORIGIN, 1.)?.rt(0.5),
            fade_in(stage.arena_mut(), rows[1].label, ORIGIN, 1.)?.rt(0.5)
        );
        run(stage, &kit, &mut second, values, &rows[1], 0.45, 0.3)?;
        let last_bracket = trail_box(stage, LAST_TWO_TRAIL, ROW_Y[1])?;
        d4g::show(stage, &[last_bracket], 0.35)?;
        kit.until_sentence(stage, "13_10", 2)?;
        // The same machines in the same order: every column of the trails agrees.
        let columns: Vec<Mob> = rows
            .iter()
            .flat_map(|row| [row.items[2], row.items[4], row.items[6]])
            .collect();
        pulse(stage, &columns)?;
        let associative = kit.tex(stage, ASSOCIATIVE_TEX, 36., Palette::green(), &[], v(0., EQUATION_Y))?;
        d4g::show(stage, &[associative], 0.4)?;

        kit.say(stage, "13_11")?;
        let mut leaving = machines;
        leaving.extend([
            last_two,
            second.disc,
            second.number,
            first_bracket,
            last_bracket,
            associative,
        ]);
        for row in &rows {
            leaving.extend(&row.items);
            leaving.push(row.label);
        }
        d4g::hide(stage, &leaving, 0.4)?;
        // Each card lands as its property is named.
        let cards = (0..4).map(|k| card(stage, &kit, k)).collect::<fmn::Result<Vec<_>>>()?;
        for (shown_card, phrase) in cards.iter().zip(LAW_PHRASES) {
            until_phrase(stage, &kit, "13_11", 0, phrase)?;
            d4g::show(stage, &shown_card.parts(), 0.35)?;
        }
        kit.until_sentence(stage, "13_11", 1)?;
        let together = frame(stage, (0., CARD_Y), (13.75, 1.35), Palette::gold(), 3.)?;
        let named = kit.text(stage, "Together: a group", 30., Palette::hero(), v(0., 1.0))?;
        d4g::show(stage, &[together, named], 0.5)?;
        kit.until_sentence(stage, "13_11", 2)?;
        let example = kit.text(
            stage,
            "One example: the square's eight moves",
            30.,
            Palette::hero(),
            v(0., 1.0),
        )?;
        play!(stage; fmn::animation::replacement_transform(named, example).rt(0.4));
        let centre_rack = rack(stage, &kit, (-2.25, 0.1), (1.5, 1.45), 0.8)?;
        appear(stage, &centre_rack.slots, 0.35, 0.2)?;
        kit.until_sentence(stage, "13_11", 3)?;
        let mut leaving = centre_rack.slots.clone();
        leaving.extend([example, together]);
        d4g::hide(stage, &leaving, 0.4)?;
        let line = number_line(stage, &kit)?;
        let walker = stage.arena_mut().add(
            Circle::new()
                .radius(0.17)
                .style(ink_style(Palette::blue()))
                .build()
                .moved_to(v(f64::from(ZERO_AT), LINE_Y)),
        );
        let mut drawn = vec![line.line];
        drawn.extend(line.ticks.iter().chain(&line.labels));
        appear(stage, &drawn, 0.3, 0.04)?;
        d4g::show(stage, &[walker], 0.3)?;
        until_phrase(stage, &kit, "13_11", 3, ZERO_PHRASE)?;
        let sum = sum_tex(stage, &kit, &format!("{ZERO_AT}+0={ZERO_AT}"), Palette::ink())?;
        d4g::show(stage, &[sum], 0.35)?;
        pulse(stage, &[walker])?;
        until_phrase(stage, &kit, "13_11", 3, NEGATIVE_PHRASE)?;
        let there = ZERO_AT + STEP;
        let added = sum_tex(stage, &kit, &format!("{ZERO_AT}+{STEP}={there}"), Palette::ink())?;
        walk(stage, walker, sum, added, there)?;
        let undone = sum_tex(stage, &kit, &format!("{there}+(-{STEP})={ZERO_AT}"), Palette::ink())?;
        walk(stage, walker, added, undone, there - STEP)?;
        kit.until_sentence(stage, "13_11", 4)?;
        // Only the positive integers remain: zero and the negatives go dark,
        // and with them the identity and every inverse.
        let gone: Vec<Mob> = line
            .ticks
            .iter()
            .chain(&line.labels)
            .copied()
            .filter(|&m| line.non_positive(m))
            .collect();
        recolor(stage, &gone, Palette::faint(), 0.5)?;
        let shade = stage.arena_mut().add(
            Line::new(v(f64::from(LINE.0) - 0.6, LINE_Y), v(0.5, LINE_Y))
                .style(stroke_style(Palette::faint(), 7.))
                .build()?,
        );
        let missing = kit.text(
            stage,
            "Positive integers: zero and the negatives are missing",
            28.,
            Palette::dim(),
            v(0., CAPTION_Y),
        )?;
        let mut lost = vec![shade, missing];
        for k in [2, 3] {
            lost.push(mark(stage, (CARD_X[k] + 1.25, CARD_Y + 0.25), false)?);
        }
        d4g::show(stage, &lost, 0.4)?;
        let forward = sum_tex(stage, &kit, &format!("{ZERO_AT}+{STEP}={there}"), Palette::ink())?;
        walk(stage, walker, undone, forward, there)?;
        until_phrase(stage, &kit, "13_11", 4, UNDONE_PHRASE)?;
        // Undoing the addition needs -3, which is not in the collection.
        let attempt = sum_tex(stage, &kit, &format!("{there}+(-{STEP})"), Palette::red())?;
        let why = kit.text(
            stage,
            &format!("Undoing +{STEP} needs −{STEP}, which is missing"),
            28.,
            Palette::dim(),
            v(0., CAPTION_Y),
        )?;
        play!(stage;
            fmn::animation::replacement_transform(forward, attempt).rt(0.4),
            fmn::animation::replacement_transform(missing, why).rt(0.4)
        );
        let needed = stage.arena_mut().add(
            Circle::new()
                .radius(0.3)
                .style(stroke_style(Palette::red(), 4.))
                .build()
                .moved_to(v(f64::from(-STEP), LINE_Y - 0.45)),
        );
        let refused = mark(stage, (1.25, SUM_Y), false)?;
        d4g::show(stage, &[needed, refused], 0.4)?;

        kit.say(stage, "13_12")?;
        let mut leaving = drawn;
        leaving.extend([walker, attempt, why, shade, needed, refused]);
        leaving.extend(&lost[2..]);
        d4g::hide(stage, &leaving, 0.4)?;
        // Not a checklist: each card is dimmed until a calculation needs it.
        let texts: Vec<Mob> = cards.iter().flat_map(|c| [c.name, c.rule]).collect();
        recolor(stage, &texts, Palette::dim(), 0.4)?;
        let side_rack = rack(stage, &kit, (-5.6, 0.55), (1.2, 1.45), 0.72)?;
        appear(stage, &side_rack.slots, 0.3, 0.15)?;
        // Closure: r times s is again one of the eight.
        kit.until_sentence(stage, "13_12", 1)?;
        let (p, q) = CLOSURE_PAIR;
        let lit = relight(stage, &cards, None, 0, &[])?;
        let factors = [
            side_rack.frame(stage, p, Palette::blue())?,
            side_rack.frame(stage, q, Palette::blue())?,
        ];
        d4g::show(stage, &factors, 0.35)?;
        let closed = eq_tex(
            stage,
            &kit,
            &format!(r"{}\,{}={}", p.tex(), q.tex(), p.mul(q).tex()),
            EQ_Y[0],
        )?;
        let landed = side_rack.frame(stage, p.mul(q), Palette::green())?;
        let aim = purpose(stage, &kit, "Stay inside the calculation")?;
        d4g::show(stage, &[closed, landed, aim], 0.35)?;
        let mut shown = vec![lit, factors[0], factors[1], closed, landed, aim];
        // Associativity: package the long word into steps.
        kit.until_sentence(stage, "13_12", 2)?;
        let lit = relight(stage, &cards, Some(0), 1, &shown)?;
        let (head, tail) = LONG_WORD.split_at(3);
        let lines = [
            spaced(&LONG_WORD),
            format!(r"=({})\,{}=r^{{-1}}\,{}", spaced(head), spaced(tail), spaced(tail)),
            format!(
                r"=(r^{{-1}}\,{})\,{}={}",
                spaced(&tail[..1]),
                spaced(&tail[1..]),
                product(&LONG_WORD).tex()
            ),
        ];
        shown = vec![lit];
        for (k, line) in lines.iter().enumerate() {
            let step = eq_tex(stage, &kit, line, EQ_Y[k])?;
            d4g::show(stage, &[step], 0.35)?;
            stage.wait(0.35)?;
            shown.push(step);
        }
        let landed = side_rack.frame(stage, product(&LONG_WORD), Palette::green())?;
        let aim = purpose(stage, &kit, "Bundle a long instruction")?;
        d4g::show(stage, &[landed, aim], 0.35)?;
        shown.extend([landed, aim]);
        // Identity: e changes nothing on either side.
        kit.until_sentence(stage, "13_12", 3)?;
        let lit = relight(stage, &cards, Some(1), 2, &shown)?;
        let neutral = side_rack.frame(stage, E, Palette::green())?;
        let example = IDENTITY_EXAMPLE;
        let unchanged = eq_tex(
            stage,
            &kit,
            &format!(
                r"e\,{}={}={}\,e",
                example.tex(),
                E.mul(example).tex(),
                example.mul(E).tex()
            ),
            EQ_Y[0],
        )?;
        let aim = purpose(stage, &kit, "Do nothing")?;
        d4g::show(stage, &[neutral, unchanged, aim], 0.35)?;
        shown = vec![lit, neutral, unchanged, aim];
        // Inverses: solve r x = s by undoing r.
        kit.until_sentence(stage, "13_12", 4)?;
        let lit = relight(stage, &cards, Some(2), 3, &shown)?;
        let (a, b) = SOLVE;
        let x = solve(a, b);
        let lines = [
            format!(r"{}\,x={}", a.tex(), b.tex()),
            format!(r"x={}^{{-1}}\,{}", a.tex(), b.tex()),
            format!("x={}", x.tex()),
        ];
        shown = vec![lit];
        for (k, line) in lines.iter().enumerate() {
            let step = eq_tex(stage, &kit, line, EQ_Y[k])?;
            d4g::show(stage, &[step], 0.35)?;
            stage.wait(0.3)?;
            shown.push(step);
        }
        let answer = side_rack.frame(stage, x, Palette::gold())?;
        let check = eq_tex(
            stage,
            &kit,
            &format!(r"{}\,({})={}", a.tex(), x.tex(), a.mul(x).tex()),
            EQ_Y[3],
        )?;
        let checked = mark(stage, (EQ_X + 1.6, EQ_Y[3]), true)?;
        let aim = purpose(stage, &kit, "Undo to solve")?;
        d4g::show(stage, &[answer, check, checked, aim], 0.35)?;
        shown.extend([answer, check, checked, aim]);
        // Next: the same four rules, about objects we know nothing else about.
        kit.until_sentence(stage, "13_12", 5)?;
        shown.extend(side_rack.slots);
        d4g::hide(stage, &shown, 0.4)?;
        let names: Vec<Mob> = cards.iter().map(|c| c.name).collect();
        let rules: Vec<Mob> = cards.iter().map(|c| c.rule).collect();
        recolor(stage, &names, Palette::hero(), 0.3)?;
        recolor(stage, &rules, Palette::ink(), 0.3)?;
        let mut unknown = Vec::new();
        for cx in [-2., 0., 2.] {
            unknown.push(
                stage.arena_mut().add(
                    Circle::new()
                        .radius(0.45)
                        .style(stroke_style(Palette::dim(), 3.))
                        .build()
                        .moved_to(v(cx, -0.3)),
                ),
            );
            unknown.push(kit.text(stage, "?", 40., Palette::dim(), v(cx, -0.3))?);
        }
        appear(stage, &unknown, 0.35, 0.15)?;
        until_phrase(stage, &kit, "13_12", 5, UNKNOWN_PHRASE)?;
        let only = kit.text(
            stage,
            "Only these four rules are used",
            30.,
            Palette::hero(),
            v(0., -1.7),
        )?;
        d4g::show(stage, &[only], 0.4)?;
        pulse(stage, &rules)?;
        let _ = heading;
        kit.finish(stage)
    }
}

// ---- 13_9 to 13_12: every value and product drawn, computed here -----------

/// 13_9: the pair whose order matters (`s r` against `r s`) and the word
/// both tiles perform, as written (the rightmost move acts first).
pub(crate) const SWAP: (Elem, Elem) = (S, R);
pub(crate) const WORD: [Elem; 3] = [S, R, S];
/// The word's letters, close enough to read as one word and centred between the tiles.
const TOKEN_X: [f64; 3] = [-0.9, 0., 0.9];
/// [`WORD`] in the order performed (a test ties the two together).
const SEQUENCE: [Move; 3] = [Move::Flip, Move::Turn(1.), Move::Flip];
/// A lone move and a packaged pair performed as one instruction, seconds.
const LONE_RT: f64 = 1.3;
const PACKAGE_RT: f64 = 2.4;
const PACKAGE_Y: f64 = -1.95;
const NOTE_Y: f64 = -2.65;

/// 13_10: h adds two, g triples, f subtracts one; the input is one.
pub(crate) const INPUT: i32 = 1;
pub(crate) const H_ADD: i32 = 2;
pub(crate) const G_TIMES: i32 = 3;
pub(crate) const F_SUB: i32 = 1;
fn h(x: i32) -> i32 {
    x + H_ADD
}
fn g(x: i32) -> i32 {
    G_TIMES * x
}
fn f(x: i32) -> i32 {
    x - F_SUB
}
/// The input, then the value leaving each machine, innermost first.
pub(crate) fn trail(x: i32) -> [i32; 4] {
    let after_h = h(x);
    let after_g = g(after_h);
    [x, after_h, after_g, f(after_g)]
}
/// f∘(g∘h): the first two applications packaged as one machine.
pub(crate) fn first_two_packaged(x: i32) -> i32 {
    let package = |y| g(h(y));
    f(package(x))
}
/// (f∘g)∘h: the last two applications packaged as one machine.
pub(crate) fn last_two_packaged(x: i32) -> i32 {
    let package = |y| f(g(y));
    package(h(x))
}
fn machine_rules() -> [String; 3] {
    [
        format!("h(x)=x+{H_ADD}"),
        format!("g(x)={G_TIMES}x"),
        format!("f(x)=x-{F_SUB}"),
    ]
}
const MACHINE_X: [f64; 3] = [-3., 0., 3.];
const MACHINE_Y: f64 = 0.9;
const RULE_Y: f64 = 1.95;
const CHIP_IN: f64 = -5.4;
const CHIP_OUT: f64 = 5.4;
const FIRST_TWO_SPAN: (f64, f64) = (-4.2, 1.2);
const LAST_TWO_SPAN: (f64, f64) = (-1.2, 4.2);
const ROW_Y: [f64; 2] = [-0.55, -1.55];
/// From a package's input through its output: `1 → 3 → 9` for g∘h, `3 → 9 → 8` for f∘g.
const FIRST_TWO_TRAIL: (f64, f64) = (CHIP_IN - 0.35, MACHINE_X[1] + 0.35);
const LAST_TWO_TRAIL: (f64, f64) = (MACHINE_X[0] - 0.35, MACHINE_X[2] + 0.35);
// Each box holds its values and stops short of the next arrow out.
const _: () = assert!(FIRST_TWO_TRAIL.1 < (MACHINE_X[1] + MACHINE_X[2]) / 2. - 0.2);
const _: () = assert!(LAST_TWO_TRAIL.0 > (CHIP_IN + MACHINE_X[0]) / 2. + 0.2);
const ROW_LABEL_X: f64 = 5.4;
const EQUATION_Y: f64 = -2.7;
const FIRST_TWO_TEX: &str = r"f\circ(g\circ h)";
const LAST_TWO_TEX: &str = r"(f\circ g)\circ h";
const ASSOCIATIVE_TEX: &str = r"f\circ(g\circ h)=(f\circ g)\circ h";

/// 13_11: the drawn integers, where adding zero is shown, and the step undone.
pub(crate) const LINE: (i32, i32) = (-5, 5);
pub(crate) const ZERO_AT: i32 = 2;
pub(crate) const STEP: i32 = 3;
const LINE_Y: f64 = -0.3;
const SUM_Y: f64 = 0.8;
const CAPTION_Y: f64 = -1.9;
const CARD_X: [f64; 4] = [-5.1, -1.7, 1.7, 5.1];
const CARD_Y: f64 = 2.05;
const LAWS: [(&str, &str); 4] = [
    ("Closure", r"a\,b\ \text{stays inside}"),
    ("Associativity", "(ab)c=a(bc)"),
    ("Identity", r"e\,a=a=a\,e"),
    ("Inverse", r"a^{-1}a=e=a\,a^{-1}"),
];

/// 13_12: the rack calculations.
/// `s r` is not a name on the rack, and the product `r³ s` is: closure with content,
/// where the old `r s = rs` read as a tautology.
pub(crate) const CLOSURE_PAIR: (Elem, Elem) = (S, R);
pub(crate) const LONG_WORD: [Elem; 5] = [S, R, S, R2, S];
pub(crate) const IDENTITY_EXAMPLE: Elem = R;
/// Solve `a x = b` for `(a, b)`.
pub(crate) const SOLVE: (Elem, Elem) = (R, S);
/// The rack that 13_7 and 13_8 point at, and the frame round one of its rows.
const MOVES_AT: (f64, f64) = (-2.25, 0.1);
const MOVES_STEP: (f64, f64) = (1.5, 1.45);
/// The chapter's two tile-name positions.
const LEFT_LABEL: (f64, f64) = (-3.1, -2.);
const RIGHT_LABEL: (f64, f64) = (3.1, -2.);
/// A label that stays while a 1.25-scale tile turns: its corner discs reach 1.81 below centre.
const BELOW_TURNING: f64 = -2.3;
const EQ_X: f64 = 2.4;
const EQ_Y: [f64; 4] = [0.75, 0.0, -0.75, -1.5];

/// The product of a written word (rightmost acts first).
pub(crate) fn product(word: &[Elem]) -> Elem {
    word.iter().fold(E, |acc, &g| acc.mul(g))
}
/// `x` with `a x = b`: undo `a` on the left.
pub(crate) fn solve(a: Elem, b: Elem) -> Elem {
    a.inv().mul(b)
}
fn spaced(word: &[Elem]) -> String {
    word.iter().map(|g| g.tex()).collect::<Vec<_>>().join(r"\,")
}

/// Phrases the pictures wait for: (cue, sentence, phrase); a test finds each.
const EIGHT_PHRASE: &str = "its final action";
const RELATION_PHRASE: &str = "s r s equals";
const GROUPING_PHRASE: &str = "grouping the first two";
const OBJECT_PHRASE: &str = "any object";
const FUNCTIONS_PHRASE: &str = "three functions";
const FIRST_TWO_PHRASE: &str = "package the first two";
const LAST_TWO_PHRASE: &str = "or the last two";
const LAW_PHRASES: [&str; 4] = ["Closure", "associativity", "an identity", "an inverse"];
const ZERO_PHRASE: &str = "zero does nothing";
const NEGATIVE_PHRASE: &str = "adding the negative";
const UNDONE_PHRASE: &str = "because most additions";
const UNKNOWN_PHRASE: &str = "without knowing";
#[cfg(test)]
const PHRASES: [(&str, usize, &str); 15] = [
    ("13_1", 2, EIGHT_PHRASE),
    ("13_6", 4, RELATION_PHRASE),
    ("13_9", 3, GROUPING_PHRASE),
    ("13_10", 0, OBJECT_PHRASE),
    ("13_10", 0, FUNCTIONS_PHRASE),
    ("13_10", 1, FIRST_TWO_PHRASE),
    ("13_10", 1, LAST_TWO_PHRASE),
    ("13_11", 0, LAW_PHRASES[0]),
    ("13_11", 0, LAW_PHRASES[1]),
    ("13_11", 0, LAW_PHRASES[2]),
    ("13_11", 0, LAW_PHRASES[3]),
    ("13_11", 3, ZERO_PHRASE),
    ("13_11", 3, NEGATIVE_PHRASE),
    ("13_11", 4, UNDONE_PHRASE),
    ("13_12", 5, UNKNOWN_PHRASE),
];

/// Run a tile's tracker to `to` in one motion while `marks` are indicated.
fn perform(stage: &mut Stage<'_>, tracker: Mob, to: f64, run_time: f64, marks: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(tracker, to, run_time)?)?];
    for &mob in marks {
        let a = fmn::animation::indicate(stage.arena_mut(), mob, 1.15, None)?.rt(run_time);
        animations.push(stage.prepare(a)?);
    }
    stage.play_prepared(animations)?;
    stage.wait(0.25)?;
    Ok(())
}

/// Fade a set in, one after another.
fn appear(stage: &mut Stage<'_>, mobs: &[Mob], each: f64, lag: f64) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.)?.rt(each);
        animations.push(stage.prepare(a)?);
    }
    let group = explainer_kit::lagged(stage, animations, lag)?;
    play!(stage; group);
    Ok(())
}

fn frame(stage: &mut Stage<'_>, center: (f64, f64), size: (f64, f64), color: Srgb, width: f64) -> fmn::Result<Mob> {
    let frame = Rectangle::new()
        .width(size.0)
        .height(size.1)
        .corner_radius(0.16)
        .style(stroke_style(color, width))
        .build()?
        .moved_to(v(center.0, center.1));
    Ok(stage.arena_mut().add(frame))
}

/// The gold box packaging two machines.
fn package(stage: &mut Stage<'_>, span: (f64, f64)) -> fmn::Result<Mob> {
    frame(
        stage,
        ((span.0 + span.1) / 2., MACHINE_Y),
        (span.1 - span.0, 1.4),
        Palette::gold(),
        4.,
    )
}

/// The gold box round a package's stretch of a trail.
fn trail_box(stage: &mut Stage<'_>, span: (f64, f64), y: f64) -> fmn::Result<Mob> {
    frame(
        stage,
        ((span.0 + span.1) / 2., y),
        (span.1 - span.0, 0.66),
        Palette::gold(),
        3.,
    )
}

/// The value travelling through the machines: a disc with its number.
struct Chip {
    disc: Mob,
    number: Mob,
}
fn chip(stage: &mut Stage<'_>, kit: &Kit, n: i32) -> fmn::Result<Chip> {
    let disc = stage.arena_mut().add(
        Circle::new()
            .radius(0.34)
            .style(
                Style::default()
                    .fill(hex("#173148"), 1.)
                    .stroke(Palette::blue(), 4., 1.),
            )
            .build()
            .moved_to(v(CHIP_IN, MACHINE_Y)),
    );
    let number = kit.tex(stage, &n.to_string(), 34., Palette::ink(), &[], v(CHIP_IN, MACHINE_Y))?;
    Ok(Chip { disc, number })
}

/// One trail under the machines: the input, then an arrow and the value
/// leaving each machine, in the order the chip produces them.
struct Trail {
    items: Vec<Mob>,
    label: Mob,
}
fn trail_row(stage: &mut Stage<'_>, kit: &Kit, values: [i32; 4], y: f64, label: &str) -> fmn::Result<Trail> {
    let mut items = vec![kit.tex(stage, &values[0].to_string(), 34., Palette::ink(), &[], v(CHIP_IN, y))?];
    let mut from = CHIP_IN;
    for (k, &x) in MACHINE_X.iter().enumerate() {
        items.push(kit.tex(stage, r"\rightarrow", 30., Palette::dim(), &[], v((from + x) / 2., y))?);
        items.push(kit.tex(stage, &values[k + 1].to_string(), 34., Palette::ink(), &[], v(x, y))?);
        from = x;
    }
    let label = kit.tex(stage, label, 30., Palette::gold(), &[], v(ROW_LABEL_X, y))?;
    Ok(Trail { items, label })
}

/// Carry the chip through h, g and f: inside each machine its number
/// becomes the machine's output, and the trail records it.
fn run(
    stage: &mut Stage<'_>,
    kit: &Kit,
    chip: &mut Chip,
    values: [i32; 4],
    row: &Trail,
    hop: f64,
    change: f64,
) -> fmn::Result<()> {
    for (k, &x) in MACHINE_X.iter().enumerate() {
        d4g::glide_all(
            stage,
            &[(chip.disc, (x, MACHINE_Y)), (chip.number, (x, MACHINE_Y))],
            hop,
        )?;
        let next = kit.tex(
            stage,
            &values[k + 1].to_string(),
            34.,
            Palette::ink(),
            &[],
            v(x, MACHINE_Y),
        )?;
        let (arrow, value) = (row.items[2 * k + 1], row.items[2 * k + 2]);
        play!(stage;
            fmn::animation::replacement_transform(chip.number, next).rt(change),
            fade_in(stage.arena_mut(), arrow, ORIGIN, 1.)?.rt(change),
            fade_in(stage.arena_mut(), value, ORIGIN, 1.)?.rt(change)
        );
        chip.number = next;
    }
    let out = (CHIP_OUT, MACHINE_Y);
    d4g::glide_all(stage, &[(chip.disc, out), (chip.number, out)], hop)
}

/// One property card: its name above its rule.
struct Card {
    frame: Mob,
    name: Mob,
    rule: Mob,
}
impl Card {
    fn parts(&self) -> [Mob; 3] {
        [self.frame, self.name, self.rule]
    }
}
fn card(stage: &mut Stage<'_>, kit: &Kit, k: usize) -> fmn::Result<Card> {
    let (name, rule) = LAWS[k];
    let x = CARD_X[k];
    Ok(Card {
        frame: frame(stage, (x, CARD_Y), (3.15, 1.05), Palette::dim(), 2.5)?,
        name: kit.text(stage, name, 24., Palette::hero(), v(x, CARD_Y + 0.25))?,
        rule: kit.tex(stage, rule, 26., Palette::ink(), &[], v(x, CARD_Y - 0.22))?,
    })
}
/// Move the light to card `to` in one 0.4 s play: the last demonstration fades,
/// the last card dims, the new card brightens and gains its frame. Played one
/// after another these cost 1.35 s of a sentence that may last under 3 s.
fn relight(stage: &mut Stage<'_>, cards: &[Card], from: Option<usize>, to: usize, leaving: &[Mob]) -> fmn::Result<Mob> {
    const RUN: f64 = 0.4;
    let lit = frame(stage, (CARD_X[to], CARD_Y), (3.35, 1.2), Palette::gold(), 4.)?;
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in leaving {
        stage.clear_updaters(mob, true);
        let a = fade_out(stage.arena_mut(), mob, ORIGIN, 1.)?.rt(RUN);
        animations.push(stage.prepare(a)?);
    }
    let mut colors = vec![(cards[to].name, Palette::hero()), (cards[to].rule, Palette::ink())];
    if let Some(k) = from {
        colors.extend([(cards[k].name, Palette::dim()), (cards[k].rule, Palette::dim())]);
    }
    for (mob, color) in colors {
        let a = mob
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(RUN),
                ..AnimateArgs::default()
            })?
            .set_color(color)?;
        animations.push(stage.prepare(a)?);
    }
    let a = fade_in(stage.arena_mut(), lit, ORIGIN, 1.)?.rt(RUN);
    animations.push(stage.prepare(a)?);
    stage.play_prepared(animations)?;
    Ok(lit)
}

/// The eight moves as rack thumbnails with their names, rotations above.
struct Rack {
    slots: Vec<Mob>,
    at: [(f64, f64); 8],
    k: f64,
}
fn rack(stage: &mut Stage<'_>, kit: &Kit, origin: (f64, f64), step: (f64, f64), k: f64) -> fmn::Result<Rack> {
    let at: [(f64, f64); 8] =
        std::array::from_fn(|i| (origin.0 + step.0 * (i % 4) as f64, origin.1 - step.1 * (i / 4) as f64));
    let mut slots = Vec::new();
    for (g, &(x, y)) in ALL.into_iter().zip(&at) {
        let thumb = d4g::thumb(stage, kit, g, (x, y), k)?;
        let name = kit.tex(stage, g.tex(), 26., Palette::dim(), &[], v(x, y - 0.8 * k))?;
        slots.push(d4g::group_of(stage, &[thumb, name])?);
    }
    Ok(Rack { slots, at, k })
}
impl Rack {
    /// A coloured frame round `g`'s thumbnail, not yet shown.
    fn frame(&self, stage: &mut Stage<'_>, g: Elem, color: Srgb) -> fmn::Result<Mob> {
        d4g::slot_frame(stage, self.at[g.index()], self.k, color)
    }
}

/// One row of the 13_7 rack, thumbnails and names together.
fn row_frame(stage: &mut Stage<'_>, row: usize) -> fmn::Result<Mob> {
    let y = MOVES_AT.1 - MOVES_STEP.1 * row as f64 - 0.2;
    frame(stage, (0., y), (5.9, 1.35), Palette::gold(), 3.)
}
/// The eight arrangements before they have names, rotations above.
fn bare_rack(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<Vec<Mob>> {
    ALL.into_iter()
        .enumerate()
        .map(|(i, g)| {
            let at = (-2.25 + 1.5 * (i % 4) as f64, 0.8 - 1.6 * (i / 4) as f64);
            d4g::thumb(stage, kit, g, at, 1.)
        })
        .collect()
}

/// The integers drawn on 13_11's line, each with its tick and label.
struct NumberLine {
    line: Mob,
    ticks: Vec<Mob>,
    labels: Vec<Mob>,
}
impl NumberLine {
    /// Whether `mob` is the tick or label of zero or a negative integer.
    fn non_positive(&self, mob: Mob) -> bool {
        let count = usize::try_from(1 - LINE.0).unwrap_or(0);
        self.ticks[..count].contains(&mob) || self.labels[..count].contains(&mob)
    }
}
fn number_line(stage: &mut Stage<'_>, kit: &Kit) -> fmn::Result<NumberLine> {
    let line = stage.arena_mut().add(
        Line::new(v(f64::from(LINE.0) - 0.6, LINE_Y), v(f64::from(LINE.1) + 0.6, LINE_Y))
            .style(stroke_style(Palette::ink(), 2.5))
            .build()?,
    );
    let (mut ticks, mut labels) = (Vec::new(), Vec::new());
    for n in LINE.0..=LINE.1 {
        let x = f64::from(n);
        ticks.push(
            stage.arena_mut().add(
                Line::new(v(x, LINE_Y - 0.1), v(x, LINE_Y + 0.1))
                    .style(stroke_style(Palette::ink(), 2.5))
                    .build()?,
            ),
        );
        labels.push(kit.tex(stage, &n.to_string(), 28., Palette::ink(), &[], v(x, LINE_Y - 0.45))?);
    }
    Ok(NumberLine { line, ticks, labels })
}
fn sum_tex(stage: &mut Stage<'_>, kit: &Kit, src: &str, color: Srgb) -> fmn::Result<Mob> {
    kit.tex(stage, src, 34., color, &[], v(0., SUM_Y))
}
/// Move the walker to `to` while the sum above it changes.
fn walk(stage: &mut Stage<'_>, walker: Mob, from: Mob, to_sum: Mob, to: i32) -> fmn::Result<()> {
    let motion = walker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.8),
            ..AnimateArgs::default()
        })?
        .move_to(v(f64::from(to), LINE_Y), ORIGIN)?;
    play!(stage; motion, fmn::animation::replacement_transform(from, to_sum).rt(0.8));
    Ok(())
}
fn eq_tex(stage: &mut Stage<'_>, kit: &Kit, src: &str, y: f64) -> fmn::Result<Mob> {
    kit.tex(stage, src, 32., Palette::ink(), &[], v(EQ_X, y))
}
/// A beat's one-line purpose, not yet shown: it arrives with the beat's last marks.
fn purpose(stage: &mut Stage<'_>, kit: &Kit, text: &str) -> fmn::Result<Mob> {
    kit.text(stage, text, 30., Palette::hero(), v(0., -2.6))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axioms::{line_text, sentence_ranges};
    #[test]
    fn conjugation_and_reduction_agree_on_every_corner() {
        let s = tile_model::flip(1.);
        let r = tile_model::turn(1.);
        let word = tile_model::compose(
            tile_model::compose(tile_model::compose(tile_model::compose(s, r), s), tile_model::turn(2.)),
            s,
        );
        let reduced = tile_model::compose(r, s);
        for p in tile_model::CORNERS {
            for (a, b) in [(pose(&LEFT, 9.), pose(&RIGHT, 5.)), (word, reduced)] {
                let a = tile_model::apply(a, p);
                let b = tile_model::apply(b, p);
                for k in 0..3 {
                    assert!((a[k] - b[k]).abs() < 1e-10);
                }
            }
        }
    }

    fn same_corners(a: Matrix, b: Matrix) -> bool {
        tile_model::CORNERS.iter().all(|&p| {
            let (a, b) = (tile_model::apply(a, p), tile_model::apply(b, p));
            a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-9)
        })
    }

    #[test]
    fn swapping_can_change_a_product_but_regrouping_never_does() {
        let (a, b) = SWAP;
        assert_ne!(a.mul(b), b.mul(a));
        for x in ALL {
            for y in ALL {
                for z in ALL {
                    assert_eq!(x.mul(y).mul(z), x.mul(y.mul(z)));
                }
            }
        }
    }

    #[test]
    fn both_packagings_pass_through_the_same_poses() {
        // The left tile stops after the rightmost move; the right tile after
        // the package of the first two moves performed.
        assert!(same_corners(pose(&SEQUENCE, 1.), WORD[2].matrix()));
        assert!(same_corners(pose(&SEQUENCE, 2.), WORD[1].mul(WORD[2]).matrix()));
        assert!(same_corners(pose(&SEQUENCE, 3.), product(&WORD).matrix()));
        assert_eq!(WORD[0].mul(WORD[1]).mul(WORD[2]), WORD[0].mul(WORD[1].mul(WORD[2])));
    }

    #[test]
    fn the_machines_give_one_output_under_either_packaging() {
        for x in -20..=20 {
            assert_eq!(first_two_packaged(x), trail(x)[3]);
            assert_eq!(last_two_packaged(x), trail(x)[3]);
        }
        // Each machine visibly changes the chip's single digit.
        let values = trail(INPUT);
        for (k, &n) in values.iter().enumerate() {
            assert!((0..=9).contains(&n));
            assert!(!values[..k].contains(&n));
        }
        // Order matters for these machines, as it does for moves.
        assert_ne!(g(h(INPUT)), h(g(INPUT)));
    }

    #[test]
    fn the_routes_and_the_restarted_tiles_end_as_spoken() {
        assert!(same_corners(
            pose(&LONG_ROUTE, LONG_ROUTE.len() as f64),
            tile_model::turn(1.)
        ));
        assert!(same_corners(pose(&SEQUENCE, 3.), tile_model::turn(-1.)));
    }

    #[test]
    fn the_word_reduction_uses_the_spoken_rules() {
        assert_eq!(LONG_WORD, [S, R, S, R2, S]);
        assert_eq!(S.mul(R), R.inv().mul(S));
        assert_eq!(S.mul(S), E);
        assert_eq!(R.inv().mul(R2), R);
    }

    #[test]
    fn each_trail_box_holds_its_packages_input_and_output() {
        let values = trail(INPUT);
        assert_eq!(g(h(values[0])), values[2]);
        assert_eq!(f(g(values[1])), values[3]);
    }

    #[test]
    fn the_number_line_holds_every_value_it_names() {
        let drawn = LINE.0..=LINE.1;
        for n in [ZERO_AT, ZERO_AT + STEP, -STEP, 0] {
            assert!(drawn.contains(&n));
        }
        // The walk stays among the positive integers; the inverse step and
        // zero lie outside them, as 13_11's last sentence needs.
        let positive = 1..;
        assert!(positive.contains(&ZERO_AT) && positive.contains(&(ZERO_AT + STEP)));
        assert!(!positive.contains(&-STEP) && !positive.contains(&0));
    }

    #[test]
    fn the_rack_calculations_hold() {
        let (p, q) = CLOSURE_PAIR;
        assert!(ALL.contains(&p.mul(q)));
        let (head, tail) = LONG_WORD.split_at(3);
        assert_eq!(product(head), R.inv());
        assert_eq!(tail[0], R2);
        assert_eq!(R.inv().mul(tail[0]), R);
        assert_eq!(product(&LONG_WORD), R.mul(tail[1]));
        assert_eq!(product(&LONG_WORD[..4]).mul(LONG_WORD[4]), product(&LONG_WORD));
        let example = IDENTITY_EXAMPLE;
        assert!(E.mul(example) == example && example.mul(E) == example);
        let (a, b) = SOLVE;
        assert_eq!(a.mul(solve(a, b)), b);
        assert_eq!(a, R);
        for x in ALL {
            assert_eq!(a.mul(x) == b, x == solve(a, b));
        }
    }

    #[test]
    fn every_waited_phrase_is_in_its_sentence() {
        for (cue, j, phrase) in PHRASES {
            let text = line_text(cue).expect("cue in script");
            let (from, to) = sentence_ranges(text)[j];
            assert!(text[from..to].contains(phrase), "{cue} sentence {j}: {phrase}");
        }
        for (cue, sentences) in [
            ("13_1", 3),
            ("13_2", 6),
            ("13_3", 7),
            ("13_4", 6),
            ("13_5", 5),
            ("13_6", 5),
            ("13_7", 5),
            ("13_8", 5),
            ("13_9", 6),
            ("13_10", 3),
            ("13_11", 5),
            ("13_12", 6),
        ] {
            assert_eq!(sentence_ranges(line_text(cue).unwrap()).len(), sentences, "{cue}");
        }
    }
}
