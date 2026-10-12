//! Chapter 02, the room: the stretch of code only one program may be in
//! (02_1), a sign on its door (02_2–02_3), the sign failing in eight of its
//! fourteen orders (02_4; `model::sign_orders`, oracle C-F2), the problem as
//! Dijkstra posed it and Knuth extended it, as four pictograms (02_5–02_6),
//! the hardware's magic sign and Dijkstra's ordinary ones (02_7), and the
//! harder question: a sign caught mid-flip (02_8).

use explainer_kit::props::{lower_third, names_to_search, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Figure, Lattice, Sign, access_arrow, chalk, customer, fade_in_all, fade_out_all, fan, figure, group, order_gold,
    pictogram, room, room_red, say, scoreboard, sign,
};
use super::glyphs as g;
use crate::facts::cited;
use crate::model::{SCORE_START, red_point, sign_orders, sign_program};

pub struct TheRoom {
    pub kit: Kit,
}

/// The room around the scoreboard.
const ROOM_C: Vec3 = [2.4, 0.2, 0.0];
const ROOM_W: f64 = 4.2;
const ROOM_H: f64 = 3.6;
/// The door: the room's left wall.
const WALL_X: f64 = ROOM_C[0] - ROOM_W / 2.0;
/// Where the two programs stand: far off, at the door, inside.
const BLUE_Y: f64 = 0.95;
const GREEN_Y: f64 = -0.75;
const START_X: f64 = -5.4;
const DOOR_X: f64 = WALL_X - 1.25;
const INSIDE_X: f64 = WALL_X + 0.85;

/// How far the fourteen staircases fan.
const FAN: f64 = 0.8;

/// The progress square of 02_4: each program's three steps.
const SQUARE: Lattice = Lattice {
    center: [-2.3, -0.05, 0.0],
    side: 4.4,
    steps: 3,
};

impl SceneConstruct for TheRoom {
    fn name(&self) -> &str {
        "02_the_room"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let scene = the_sign(&kit, stage)?;
        sign_fails(&kit, stage, scene)?;
        rules(&kit, stage)?;
        magic(&kit, stage)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// What 02_4 plays out and then clears.
struct DoorScene {
    blue: Figure,
    green: Figure,
    door: Sign,
    mobs: Vec<Mob>,
}

/// Move a figure to `to` (a walk: a little bob on the way).
fn walk(f: &Figure, to: Vec3, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(f.group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .move_to(to, ORIGIN)?)
}

/// Turn a figure's eyes towards `dx` (−1 left, +1 right, 0 ahead).
fn look(stage: &mut Stage<'_>, f: &Figure, dx: f64) {
    for p in f.pupils {
        let c = stage.get_center(p);
        let body = stage.get_center(f.body);
        let eye_x = if c[0] < body[0] { body[0] - 0.11 } else { body[0] + 0.11 };
        stage.move_to(p, v(eye_x + 0.025 * dx, c[1]), ORIGIN);
    }
}

// --------------------------------------------------------------- 02_1–02_3

fn the_sign(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<DoorScene> {
    // 02_1: the room around the scoreboard
    let line = say(kit, stage, "02_1")?;
    let board = scoreboard(kit, stage, SCORE_START, ROOM_C, 1.0)?;
    let blue = figure(kit, stage, 1, v(START_X, BLUE_Y), 1.0)?;
    let green = figure(kit, stage, 2, v(START_X, GREEN_Y), 1.0)?;
    for f in [&blue, &green] {
        stage.set_opacity(f.badge, 0.0, true); // badges come at 03_1
    }
    fade_in_all(stage, &[board.panel, board.value, blue.group, green.group], 0.8)?;
    line.wait_for(stage, "Only one program")?;
    let [tint, edge] = room(ROOM_W, ROOM_H, ROOM_C)?;
    let tint = stage.arena_mut().add(tint);
    let edge = stage.arena_mut().add(edge);
    play!(stage;
        fade_in(stage.arena_mut(), tint, ORIGIN, 0.6)?.rt(1.0),
        fmn::animation::grow_from_center(stage.arena_mut(), edge, None)?.rt(1.0),
    );
    line.wait_for(stage, "Call that stretch")?;
    let label = kit.text(
        stage,
        g::THE_ROOM,
        34.0,
        room_red(),
        v(ROOM_C[0], ROOM_C[1] + ROOM_H / 2.0 + 0.35),
    )?;
    play!(stage; write(stage.arena(), label).rt(0.7));
    let names = lower_third(kit, stage, g::NAMES, g::CRITICAL_SECTION)?;
    play!(stage; fade_in(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;

    // 02_2: a sign on the door; Blue checks it, flips it, goes in (and,
    // with the line done, comes back out and flips it back)
    let line = say(kit, stage, "02_2")?;
    play!(stage; fade_out(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.4));
    let mut door = sign(kit, stage, false, v(WALL_X, 2.55), 0.85, false)?;
    fade_in_all(stage, &door.parts(), 0.6)?;
    line.wait_for(stage, "Check that it says")?;
    play!(stage; walk(&blue, v(DOOR_X, BLUE_Y), 1.0)?);
    look(stage, &blue, 1.0);
    let seen = stage
        .arena_mut()
        .add(access_arrow(v(WALL_X - 0.3, 2.2), v(DOOR_X + 0.15, BLUE_Y + 0.45), 1)?);
    play!(stage; fmn::animation::grow_arrow(stage.arena_mut(), seen)?.rt(0.5));
    line.wait_for(stage, "flip it to occupied")?;
    play!(stage; fade_out(stage.arena_mut(), seen, ORIGIN, 1.0)?.rt(0.3));
    door.flip(kit, stage, true, 0.5)?;
    line.wait_for(stage, "and go in")?;
    play!(stage; walk(&blue, v(INSIDE_X, BLUE_Y), 0.8)?);
    kit.hold(stage)?;
    play!(stage; walk(&blue, v(DOOR_X, BLUE_Y), 0.6)?);
    door.flip(kit, stage, false, 0.4)?;
    play!(stage; walk(&blue, v(START_X, BLUE_Y), 0.6)?);
    look(stage, &blue, 0.0);

    // 02_3: both walk up to the sign
    say(kit, stage, "02_3")?;
    play!(stage;
        walk(&blue, v(DOOR_X, BLUE_Y), 1.1)?,
        walk(&green, v(DOOR_X, GREEN_Y), 1.1)?,
    );
    look(stage, &blue, 1.0);
    look(stage, &green, 1.0);
    let q = predict_pause(kit, stage, v(5.8, -2.6))?;
    let mobs = vec![board.panel, board.value, tint, edge, label, q];
    Ok(DoorScene {
        blue,
        green,
        door,
        mobs,
    })
}

// --------------------------------------------------------------------- 02_4

fn sign_fails(kit: &Kit, stage: &mut Stage<'_>, mut scene: DoorScene) -> fmn::Result<()> {
    let line = say(kit, stage, "02_4")?;
    let q = scene
        .mobs
        .pop()
        .ok_or_else(|| super::draw::fail("the 02_3 scene lost its question mark".into()))?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    // both read "vacant" before either flips it
    line.wait_for(stage, "Both read vacant")?;
    let mut reads = Vec::new();
    for (k, f_y) in [(1, BLUE_Y), (2, GREEN_Y)] {
        let a = access_arrow(v(WALL_X - 0.3, 2.2), v(DOOR_X + 0.15, f_y + 0.45), k)?;
        reads.push(stage.arena_mut().add(a));
    }
    play!(stage;
        fmn::animation::grow_arrow(stage.arena_mut(), reads[0])?.rt(0.5),
        fmn::animation::grow_arrow(stage.arena_mut(), reads[1])?.rt(0.5),
    );
    line.wait_for(stage, "before either flips")?;
    fade_out_all(stage, &reads, 0.3)?;
    scene.door.flip(kit, stage, true, 0.4)?;
    scene.door.flip(kit, stage, true, 0.4)?;
    play!(stage;
        walk(&scene.blue, v(INSIDE_X, BLUE_Y), 0.7)?,
        walk(&scene.green, v(INSIDE_X, GREEN_Y), 0.7)?,
    );
    // two in the room
    let flash = Rectangle::new()
        .width(ROOM_W + 0.15)
        .height(ROOM_H + 0.15)
        .corner_radius(0.26)
        .style(Style::default().fill(room_red(), 0.0).stroke(room_red(), 8.0, 1.0))
        .build()?
        .moved_to(ROOM_C);
    let flash = stage.arena_mut().add(flash);
    play!(stage; fmn::animation::indicate(stage.arena_mut(), flash, 1.05, None)?.rt(0.6));
    stage.remove_from_scene(flash);

    // every order: the progress square with its both-inside point
    line.wait_for(stage, "Of the fourteen")?;
    let mut out = scene.mobs.clone();
    out.extend([scene.blue.group, scene.green.group]);
    out.extend(scene.door.parts());
    fade_out_all(stage, &out, 0.6)?;
    let orders = sign_orders();
    let steps = sign_program().len();
    let red = red_point(steps);
    let l = SQUARE;
    let nf = steps as f64;
    let grid = stage.arena_mut().add(l.grid()?);
    let mark = stage.arena_mut().add(l.mark(red.0, red.1, room_red()));
    let mut labels = Vec::new();
    for (k, word) in [g::CHECK, g::FLIP, g::LEAVE].into_iter().enumerate() {
        let c = k as f64 + 0.5;
        labels.push(kit.text(
            stage,
            word,
            22.0,
            customer(1),
            v(l.p(c, 0.0)[0], l.p(0.0, 0.0)[1] - 0.6),
        )?);
        labels.push(kit.text(
            stage,
            word,
            22.0,
            customer(2),
            v(l.p(0.0, 0.0)[0] - 0.95, l.p(0.0, c)[1]),
        )?);
    }
    let both = kit.text(
        stage,
        g::BOTH_INSIDE,
        22.0,
        room_red(),
        v(l.p(red.0 as f64, nf)[0], l.p(0.0, nf)[1] + 0.3),
    )?;
    let [start, end] = l.ends(FAN);
    let ends = [stage.arena_mut().add(start), stage.arena_mut().add(end)];
    let mut ins = vec![grid, mark, both, ends[0], ends[1]];
    ins.extend(&labels);
    fade_in_all(stage, &ins, 0.6)?;
    let n = orders.len();
    let n_bad = orders.iter().filter(|o| o.visits(red)).count();
    let count = kit.text_left(stage, &format!("{n} {}", g::ORDERS), 40.0, Palette::ink(), v(1.2, 1.3))?;
    play!(stage; write(stage.arena(), count).rt(0.5));
    let calm = Style::default().fill(chalk(), 0.0).stroke(chalk(), 3.0, 0.75);
    let hot = Style::default().fill(room_red(), 0.0).stroke(room_red(), 3.5, 1.0);
    // the safe orders first, then the eight through the red square, counted
    let (bad, good): (Vec<_>, Vec<_>) = orders.iter().enumerate().partition(|(_, o)| o.visits(red));
    let mut stairs = Vec::new();
    let mut grow: Vec<Box<dyn Animation>> = Vec::new();
    for &(r, o) in &good {
        let m = stage.arena_mut().add(l.staircase(o, fan(r, n, FAN), calm));
        grow.push(Box::new(show_creation(m)));
        stairs.push(m);
    }
    let g_good = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, 0.3)?.with_run_time(1.4);
    stage.play(g_good)?;
    stage.scene_mut().bring_to_front(&ends)?;
    let tracker = stage.add_value_tracker(0.0);
    kit.live_number(stage, tracker, 0, false, 36.0, room_red(), v(1.2, 0.65), LEFT)?;
    let in_room = kit.text_left(stage, g::PUT_BOTH_IN, 30.0, room_red(), v(1.85, 0.65))?;
    play!(stage; fade_in(stage.arena_mut(), in_room, ORIGIN, 1.0)?.rt(0.3));
    let mut grow: Vec<Box<dyn Animation>> = Vec::new();
    for &(r, o) in &bad {
        let m = stage.arena_mut().add(l.staircase(o, fan(r, n, FAN), hot));
        grow.push(Box::new(show_creation(m)));
        stairs.push(m);
    }
    let g_bad = AnimationGroup::with_lag_ratio(stage.arena_mut(), grow, 0.35)?.with_run_time(2.4);
    let tick = tween(tracker, n_bad as f64, 2.4)?;
    let tick = stage.prepare(tick)?;
    let g_bad: Box<dyn Animation> = Box::new(g_bad);
    stage.play_prepared(vec![g_bad, tick])?;
    stage.scene_mut().bring_to_front(&ends)?;
    // why 14 and not every way to mix 3 + 3 steps (R3 maya-07): a check
    // that finds "occupied" waits, so some mixes can't run
    let (mixes, runnable) = crate::facts::step_mixes(&sign_program());
    if runnable != n {
        return Err(super::draw::fail(format!(
            "{runnable} mixes can run, but there are {n} orders"
        )));
    }
    let why_1 = format!("{mixes} {}", g::MIXES_A);
    let why_2 = format!("{} {} {}", g::MIXES_B, mixes - runnable, g::MIXES_C);
    let why_1 = kit.text_left(stage, &why_1, 24.0, Palette::ink(), v(1.2, -0.45))?;
    let why_2 = kit.text_left(stage, &why_2, 24.0, Palette::ink(), v(1.2, -0.83))?;
    let lower = lower_third(kit, stage, g::COUNTING, g::COUNTING_RULE)?;
    play!(stage;
        fade_in(stage.arena_mut(), why_1, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), why_2, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), lower, ORIGIN, 1.0)?.rt(0.6),
    );
    kit.hold(stage)?;
    fade_all(stage, 0.7)?;
    Ok(())
}

// --------------------------------------------------------------- 02_5–02_6

/// How much the last ordinary sign grows for 02_8's blur.
const BLUR_SCALE: f64 = 2.6;

/// Where the four pictograms stand while they are named, and how big.
const PICTO_Y: f64 = -0.2;
const PICTO_SIZE: f64 = 2.3;
const PICTO_X: [f64; 4] = [-4.65, -1.55, 1.55, 4.65];

/// A pictogram's caption under it, wrapped to two short centred lines so
/// neighbours never touch.
fn caption(kit: &Kit, stage: &mut Stage<'_>, text: &str, x: f64) -> fmn::Result<Mob> {
    let lines = explainer_kit::animatic::wrap(text, 22, 2);
    let top = PICTO_Y - PICTO_SIZE / 2.0 - 0.36;
    let mut mobs = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        mobs.push(kit.text(stage, l, 24.0, Palette::ink(), v(x, top - 0.34 * i as f64))?);
    }
    group(stage, &mobs)
}

fn rules(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let line = say(kit, stage, "02_5")?;
    let name = kit.text(
        stage,
        &format!("{}, {}", g::DIJKSTRA, cited::DIJKSTRA_YEAR),
        40.0,
        Palette::ink(),
        v(0.0, 2.9),
    )?;
    play!(stage; write(stage.arena(), name).rt(0.9));
    let captions = [g::RULE_1, g::RULE_2, g::RULE_3, g::RULE_4];
    let phrases = ["Never two", "Someone who isn't", "And fast or slow"];
    let mut pictures = Vec::new();
    for (i, phrase) in phrases.into_iter().enumerate() {
        line.wait_for(stage, phrase)?;
        let pic = pictogram(kit, stage, i + 1, v(PICTO_X[i], PICTO_Y), PICTO_SIZE)?;
        let cap = caption(kit, stage, captions[i], PICTO_X[i])?;
        play!(stage;
            fade_in(stage.arena_mut(), pic, v(0.0, 0.2), 0.9)?.rt(0.6),
            fade_in(stage.arena_mut(), cap, ORIGIN, 1.0)?.rt(0.6),
        );
        pictures.push((pic, cap));
    }
    kit.hold(stage)?;
    // 02_6: Knuth, a year later: everyone gets in eventually
    let line = say(kit, stage, "02_6")?;
    let knuth = kit.text(
        stage,
        &format!("{}, {}", g::KNUTH, cited::KNUTH_YEAR),
        40.0,
        Palette::ink(),
        v(0.0, 2.9),
    )?;
    play!(stage;
        fade_out(stage.arena_mut(), name, v(0.0, 0.3), 1.0)?.rt(0.5),
        fade_in(stage.arena_mut(), knuth, v(0.0, -0.3), 1.0)?.rt(0.5),
    );
    line.wait_for(stage, "Everyone who wants in")?;
    let pic = pictogram(kit, stage, 4, v(PICTO_X[3], PICTO_Y), PICTO_SIZE)?;
    let cap = caption(kit, stage, captions[3], PICTO_X[3])?;
    play!(stage;
        fade_in(stage.arena_mut(), pic, v(0.0, 0.2), 0.9)?.rt(0.6),
        fade_in(stage.arena_mut(), cap, ORIGIN, 1.0)?.rt(0.6),
    );
    pictures.push((pic, cap));
    kit.hold(stage)?;
    // parked in the top-right corner for the rest of the chapter
    let mut park: Vec<Box<dyn Animation>> = Vec::new();
    for (i, &(pic, cap)) in pictures.iter().enumerate() {
        let at = v(4.15 + 0.8 * i as f64, 3.3);
        let a = pic
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .scale(0.3)?
            .move_to(at, ORIGIN)?;
        park.push(stage.prepare(a)?);
        let c = fade_out(stage.arena_mut(), cap, ORIGIN, 1.0)?;
        park.push(stage.prepare(c.rt(0.5))?);
    }
    let k = fade_out(stage.arena_mut(), knuth, ORIGIN, 1.0)?;
    park.push(stage.prepare(k.rt(0.5))?);
    stage.play_prepared(park)?;
    Ok(())
}

// --------------------------------------------------------------- 02_7–02_8

fn magic(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    // 02_7: the hardware's magic sign: check and flip in one motion
    let line = say(kit, stage, "02_7")?;
    let mut magic = sign(kit, stage, false, v(-3.6, 0.6), 1.15, true)?;
    let caption = kit.text(stage, g::MAGIC, 26.0, order_gold(), v(-3.6, -0.55))?;
    fade_in_all(stage, &magic.parts(), 0.6)?;
    // what to type to find it (R3 dev-08)
    let names = names_to_search(kit, stage, &[g::TEST_AND_SET, g::COMPARE_AND_SWAP])?;
    play!(stage; fade_in(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.5));
    line.wait_for(stage, "check and flip")?;
    play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.4));
    magic.flip(kit, stage, true, 0.5)?;
    // Dijkstra: several ordinary signs, used cleverly
    line.wait_for(stage, "Dijkstra showed")?;
    let mut ordinary = Vec::new();
    for (i, y) in [1.55, 0.35, -0.85].into_iter().enumerate() {
        let s = sign(kit, stage, i == 1, v(3.4, y), 0.8, false)?;
        ordinary.push(s);
    }
    let parts: Vec<Mob> = ordinary.iter().flat_map(Sign::parts).collect();
    fade_in_all(stage, &parts, 0.8)?;
    let plain = kit.text(stage, g::ORDINARY, 26.0, Palette::ink(), v(3.4, -1.95))?;
    play!(stage; fade_in(stage.arena_mut(), plain, ORIGIN, 1.0)?.rt(0.4));
    line.wait_for(stage, "as long as nobody")?;
    let atomic = lower_third(kit, stage, g::THE_WORD, g::ATOMIC)?;
    play!(stage;
        fade_out(stage.arena_mut(), names, ORIGIN, 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), atomic, ORIGIN, 1.0)?.rt(0.6),
    );
    kit.hold(stage)?;

    // 02_8: a glance that catches a sign mid-flip, as a blur
    let line = say(kit, stage, "02_8")?;
    let mut out: Vec<Mob> = magic.parts().to_vec();
    out.extend([caption, plain, atomic]);
    for s in &ordinary[..2] {
        out.extend(s.parts());
    }
    fade_out_all(stage, &out, 0.5)?;
    let last = &ordinary[2];
    let centre = v(0.0, 0.4);
    let shift = |m: Mob, at: Vec3| -> fmn::Result<AnimBuilder> {
        Ok(m.animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.9),
                ..AnimateArgs::default()
            })?
            .shift(v(centre[0] - at[0], centre[1] - at[1]))?
            .scale(BLUR_SCALE)?)
    };
    let c = last.center;
    play!(stage;
        shift(last.hanger, c)?,
        shift(last.board, c)?,
        shift(last.text, c)?,
    );
    line.wait_for(stage, "catch a sign mid-flip")?;
    // the blur: the board half turned, both words ghosted through each other
    let board = last.board;
    play!(stage; board.animate().set_anim_args(AnimateArgs { run_time: Some(0.5), ..AnimateArgs::default() })?.stretch(0.5, 0)?);
    let mut ghosts = Vec::new();
    for (k, dx) in [-0.14, 0.0, 0.14].into_iter().enumerate() {
        let word = if k % 2 == 0 {
            super::draw::OCCUPIED
        } else {
            super::draw::VACANT
        };
        let color = if k % 2 == 0 { room_red() } else { chalk() };
        let gm = kit.text(
            stage,
            word,
            30.0 * 0.8 * BLUR_SCALE,
            color,
            v(centre[0] + dx, centre[1]),
        )?;
        stage.stretch_about(gm, 0.5, 0, Some(v(centre[0] + dx, centre[1])), None);
        stage.set_opacity(gm, 0.35, true);
        ghosts.push(gm);
    }
    // the ghosts are one blurred word: each nests in the one before, so the
    // layout lint reads one text
    for pair in ghosts.windows(2) {
        stage.attach(pair[0], pair[1])?;
    }
    let blur = ghosts[0];
    play!(stage;
        fade_out(stage.arena_mut(), last.text, ORIGIN, 1.0)?.rt(0.4),
        fade_in(stage.arena_mut(), blur, ORIGIN, 1.0)?.rt(0.6),
    );
    let hmm = kit.text(
        stage,
        g::QUESTION,
        90.0,
        Palette::hero(),
        v(centre[0] + 3.2, centre[1] + 0.2),
    )?;
    play!(stage; fade_in(stage.arena_mut(), hmm, ORIGIN, 1.0)?.rt(0.4));
    kit.hold(stage)?;
    kit.music_breath(stage, 2.0)?;
    Ok(())
}
