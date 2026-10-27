//! Chapter 01, five boxes: one game box whose maker won't say how many it
//! made (01_1); five friends' boxes, and why the count matters to them, a
//! scale from rare to ordinary on which the "?" never settles (01_2); the
//! line of every number the company could have printed, counting up from
//! one with nothing skipped (01_3); the boxes turned over one at a time as
//! their numbers are read, then held so the five can be read (01_4); each
//! box shrinks onto its own number, keeping its number, and a tick joins
//! them (01_5); the prediction (01_6); at least sixty, the one certain
//! thing (01_7); above sixty nobody knows: the boxes spread into a row that
//! keeps them readable, then the line zooms out to a thousand under them
//! (01_8); nothing rules out a thousand, but would five from a thousand
//! look like these (01_9).
//!
//! Everything on the line is bound to three trackers through the one map
//! `draw::x_at`: the line's last number shown, how far the boxes have
//! spread, how far the ticks have grown. The zoom is those trackers moving,
//! not a second picture, in the legs `draw::zoom_legs` orders so the boxes
//! never meet on the way (the tests check every state the moves pass
//! through, not only where they end).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::{Facts, Kit, Timed, fade_all, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    BASE_PIECES, BASE_W, BRACKET_DY, BRACKET_W, DOT_OPACITY, DOT_R, LABEL_DY, LABEL_SIZE, LIT_OPACITY, Leg,
    NUMBER_SIZE, OFF, SMALL, STRIP_X1, STRIP_Y, TICK_W, base_piece, box_spot, bracket_points, dash_spans, dot_opacity,
    fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, flash_lagged, flat, game_box, lid_spot, line_style,
    move_all, pause, play_legs, refill, say, squash_all, stroke, think, tick_ends, wait_until, x_at, zoom_legs,
};
use crate::facts;
use crate::model::{self, LINE_DOTS};
use crate::palette;

pub struct FiveBoxes {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const HOW_MANY: &str = "how many?";
const FEW: &str = "a few dozen";
const RARE: &str = "rare";
const MANY: &str = "tens of thousands";
const ORDINARY: &str = "ordinary";
const AT_LEAST: &str = "at least";
const UNKNOWN: &str = "?";

/// The five numbers as 01_4 says them, in the order they are read: each box
/// turns over as its own number is spoken.
const NUMBER_WORDS: [&str; 5] = ["Fourteen", "twenty-seven", "thirty-three", "fifty-one", "sixty"];

/// The first box, alone (01_1), and its question beside it; the band of
/// five boxes across the top, the first one in its middle slot; the stakes
/// scale under the band, its two ends' labels and its "?" (01_2); the
/// panel's question and its certain fact, and the countdown, on the right.
const FIRST_AT: [f64; 2] = [-2.4, 0.9];
const HOW_MANY_AT: [f64; 2] = [-0.45, 0.9];
const BAND_X: [f64; 5] = [-5.4, -3.9, -2.4, -0.9, 0.6];
const BAND_Y: f64 = 2.35;
const FIRST_SLOT: usize = 2;
const SCALE_Y: f64 = 1.25;
const SCALE_PIECES: usize = 8;
const SCALE_LABEL_Y: f64 = 0.9;
const SCALE_NOTE_Y: f64 = 0.55;
const MARKER_Y: f64 = 1.55;
const PANEL_X: f64 = 4.6;
const QUESTION_Y: f64 = 3.3;
const CERTAIN_Y: f64 = 2.75;
const THINK_AT: [f64; 2] = [4.6, 1.3];
/// The tail's "?", under its dashed bracket near the strip's right end
/// (inside the bracket however far the line is shown); the zoomed line's
/// end label over it (its marks are `Picture::marks`).
const TAIL_X: f64 = 5.4;
const END_LABEL_DX: f64 = 0.45;
const END_LABEL_DY: f64 = 0.35;
/// Text sizes: the panel and the scale's labels.
const PANEL_SIZE: f64 = 30.0;
const SCALE_SIZE: f64 = 24.0;
/// A flip: the body squashed to this share of its height, the whole turn,
/// and how far ahead of its spoken number a box starts turning; a box's
/// move onto the line; the ticks' growth; the boxes' spread into their row
/// and the zoom under them; the beat after the last line.
const SQUASH: f64 = 0.06;
const FLIP_S: f64 = 0.28;
const FLIP_LEAD: f64 = 0.2;
const BOX_MOVE_S: f64 = 0.5;
const RISE_S: f64 = 0.5;
const FAN_S: f64 = 0.7;
const ZOOM_S: f64 = 2.0;
const END_BEAT: f64 = 0.6;

/// The order the boxes go onto the line (01_5), by their place in the band:
/// right to left. Each box then slides away from the boxes still in the
/// band, all to its left, and lands left of the boxes already down. Left to
/// right, a sliding box met the next one still in the band (the first
/// opening preview, 42.267 s: 51 under 60).
fn place_order(count: usize) -> Vec<usize> {
    (0..count).rev().collect()
}

fn at(p: [f64; 2]) -> Vec3 {
    v(p[0], p[1])
}

fn band(slot: usize) -> Vec3 {
    v(BAND_X[slot], BAND_Y)
}

/// A whole-number fact.
fn whole(f: &Facts, key: &str) -> Result<u64, String> {
    f.get(key)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("no whole-number fact {key}"))
}

/// The friends' five numbers, in order, from the facts.
fn numbers(f: &Facts) -> Result<Vec<u64>, String> {
    let list = f.get("G-F1.numbers").ok_or("no fact G-F1.numbers")?;
    list.split(',')
        .map(|s| s.parse().map_err(|_| format!("G-F1.numbers holds {s:?}")))
        .collect()
}

/// What chapter 01 shows, from the facts: the five numbers, the first two
/// numbers of the line (its labels at 01_3), the lower bound, the count the
/// line zooms out to and the zoomed line's marks (01_9).
struct Picture {
    numbers: Vec<u64>,
    first: u64,
    second: u64,
    at_least: u64,
    thousand: u64,
    marks: Vec<u64>,
}

/// A list fact, comma separated.
fn list(f: &Facts, key: &str) -> Result<Vec<u64>, String> {
    let s = f.get(key).ok_or_else(|| format!("no fact {key}"))?;
    s.split(',')
        .map(|x| x.parse().map_err(|_| format!("{key} holds {x:?}")))
        .collect()
}

fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        numbers: numbers(&f)?,
        first: whole(&f, "G-F1.first")?,
        second: whole(&f, "G-F1.second")?,
        at_least: whole(&f, "G-F1.at_least")?,
        thousand: whole(&f, "G-F1.thousand")?,
        marks: list(&f, "G-F1.zoom_marks")?,
    })
}

/// The picture only if the scene can draw it truthfully: five different
/// numbers in order (the boxes keep their order on the line), the bound
/// the largest of them and inside the opening line, which shows the
/// numbers past it fading, and a zoom that reaches past the opening line.
fn checked(pic: Picture) -> Result<Picture, String> {
    let n = &pic.numbers;
    if n.len() != NUMBER_WORDS.len() || !n.windows(2).all(|w| w[0] < w[1]) || n[0] == 0 {
        return Err(format!("the five numbers {n:?} must be five different boxes, in order"));
    }
    if n.last() != Some(&pic.at_least) || pic.at_least >= LINE_DOTS {
        return Err(format!(
            "the bound {} must be the largest number, below {LINE_DOTS}",
            pic.at_least
        ));
    }
    if pic.thousand <= LINE_DOTS {
        return Err(format!("the zoom to {} must reach past the opening line", pic.thousand));
    }
    // the line's map puts number 1 at the strip's left end, and the label
    // after it is the next number: nothing skipped
    if pic.first != 1 || pic.second != pic.first + 1 || n[0] <= pic.second {
        return Err(format!(
            "the line starts at 1 and counts on without skipping, left of every box: {} then {}",
            pic.first, pic.second
        ));
    }
    // the marks the facts print are the ones the zoom's own extent gives
    let marks = model::marks(pic.first, pic.thousand, model::ZOOM_PARTS)?;
    if pic.marks != marks {
        return Err(format!(
            "the zoomed line's marks {:?} are not its extent's {marks:?}",
            pic.marks
        ));
    }
    Ok(pic)
}

/// Everything on the line, bound through `x_at` to three trackers: the
/// line's last number shown, how far the small boxes have spread into the
/// fan, and how far their ticks have grown.
struct Bound {
    numbers: Vec<u64>,
    first: u64,
    at_least: f64,
    dots: Vec<Mob>,
    base: Vec<Mob>,
    bodies: Vec<Mob>,
    labels: Vec<Mob>,
    ticks: Vec<Mob>,
    certain: Mob,
    dashes: Vec<Mob>,
}

fn reshape(s: &mut fmn::mobject::Stage, m: Mob, points: Vec<Vec3>) {
    let _ = s.set_points(m, polyline(points).build().points());
}

fn hidden() -> Vec<Vec3> {
    vec![v(OFF[0], OFF[1]), v(OFF[0] + 0.01, OFF[1])]
}

/// A stroke the binding places (off the frame until it does). Detached.
fn thin(stage: &mut Stage<'_>, color: Srgb, w: f64, o: f64) -> Mob {
    stroke(stage, hidden(), line_style(color, w, o))
}

impl Bound {
    fn place(&self, s: &mut fmn::mobject::Stage, end: f64, fan: f64, rise: f64) {
        for (i, &d) in self.dots.iter().enumerate() {
            s.move_to(d, v(x_at((self.first + i as u64) as f64, end), STRIP_Y), ORIGIN);
        }
        let last = self.first + self.dots.len() as u64 - 1;
        for (k, &m) in self.base.iter().enumerate() {
            let points = match base_piece(k, last, end) {
                Some((a, b)) => vec![v(a, STRIP_Y), v(b, STRIP_Y)],
                None => hidden(),
            };
            reshape(s, m, points);
        }
        for (i, &n) in self.numbers.iter().enumerate() {
            let spot = box_spot(i, n, end, fan);
            s.move_to(self.bodies[i], spot, ORIGIN);
            s.move_to(self.labels[i], spot, ORIGIN);
            let [a, b] = tick_ends(n, end, spot, rise);
            reshape(s, self.ticks[i], vec![a, b]);
        }
        reshape(s, self.certain, bracket_points(self.first as f64, self.at_least, end));
        let y = STRIP_Y - BRACKET_DY;
        let spans = dash_spans(x_at(self.at_least + 1.0, end), STRIP_X1);
        for (j, &m) in self.dashes.iter().enumerate() {
            let points = match spans.get(j) {
                Some(&(a, b)) => vec![v(a, y), v(b, y)],
                None => hidden(),
            };
            reshape(s, m, points);
        }
    }
}

impl SceneConstruct for FiveBoxes {
    fn name(&self) -> &str {
        "01_five_boxes"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let pic = picture().map_err(fail)?;
        let opening = LINE_DOTS as f64;
        let largest = pic.numbers.len() - 1;

        // 01_1: one box, alone, and its maker's silence
        let line = say(&kit, stage, "01_1")?;
        let first = game_box(stage, at(FIRST_AT))?;
        fade_in_lagged(stage, &first, 0.8, 0.3)?;
        line.wait_for(stage, "how many copies")?;
        let how_many = kit.text(stage, HOW_MANY, PANEL_SIZE, palette::outline(), at(HOW_MANY_AT))?;
        fade_in_all(stage, &[how_many], 0.4)?;

        // 01_2: five friends' boxes; rare or ordinary, and the "?" settles
        // on neither
        let line = say(&kit, stage, "01_2")?;
        line.wait_for(stage, "Five friends")?;
        let mut boxes: Vec<[Mob; 2]> = Vec::new();
        let mut arriving = Vec::new();
        for slot in 0..BAND_X.len() {
            if slot == FIRST_SLOT {
                boxes.push(first);
            } else {
                let b = game_box(stage, band(slot))?;
                arriving.extend(b);
                boxes.push(b);
            }
        }
        move_all(
            stage,
            &[
                (first[0], band(FIRST_SLOT)),
                (first[1], lid_spot(band(FIRST_SLOT))),
                (how_many, v(PANEL_X, QUESTION_Y)),
            ],
            1.0,
            0.8,
        )?;
        fade_in_lagged(stage, &arriving, 1.0, 0.25)?;
        line.wait_for(stage, "a few dozen")?;
        let (left, right) = (BAND_X[0], BAND_X[BAND_X.len() - 1]);
        let piece = (right - left) / SCALE_PIECES as f64;
        let mut stakes: Vec<Mob> = (0..SCALE_PIECES)
            .map(|k| {
                let x = left + piece * k as f64;
                stroke(
                    stage,
                    vec![v(x, SCALE_Y), v(x + piece, SCALE_Y)],
                    line_style(palette::outline(), 2.0, 1.0),
                )
            })
            .collect();
        stakes.push(kit.text(stage, FEW, SCALE_SIZE, palette::outline(), v(left, SCALE_LABEL_Y))?);
        stakes.push(kit.text(stage, MANY, SCALE_SIZE, palette::outline(), v(right, SCALE_LABEL_Y))?);
        let marker = kit.text(
            stage,
            UNKNOWN,
            36.0,
            palette::top_gap(),
            v((left + right) / 2.0, MARKER_Y),
        )?;
        stakes.push(marker);
        fade_in_lagged(stage, &stakes, 0.8, 0.2)?;
        line.wait_for(stage, "their boxes are rare")?;
        let rare = kit.text(stage, RARE, SCALE_SIZE, palette::outline(), v(left, SCALE_NOTE_Y))?;
        move_all(stage, &[(marker, v(left, MARKER_Y))], 1.0, 0.7)?;
        fade_in_all(stage, &[rare], 0.3)?;
        line.wait_for(stage, "tens of thousands")?;
        move_all(stage, &[(marker, v(right, MARKER_Y))], 1.0, 0.9)?;
        line.wait_for(stage, "they're ordinary")?;
        let ordinary = kit.text(stage, ORDINARY, SCALE_SIZE, palette::outline(), v(right, SCALE_NOTE_Y))?;
        fade_in_all(stage, &[ordinary], 0.3)?;
        move_all(stage, &[(marker, v((left + right) / 2.0 + 0.4, MARKER_Y))], 1.0, 1.0)?;
        stakes.extend([rare, ordinary]);

        // 01_3: the line, one dot per box made, from one, nothing skipped
        let line = say(&kit, stage, "01_3")?;
        fade_out_all(stage, &stakes, 0.5)?;
        // dots[i] is number pic.first + i
        let dots: Vec<Mob> = (pic.first..=LINE_DOTS)
            .map(|n| {
                let x = x_at(n as f64, opening);
                let dot = Dot::new()
                    .point(v(x, STRIP_Y))
                    .radius(DOT_R)
                    .style(flat(palette::line(), dot_opacity(x)))
                    .build();
                stage.arena_mut().add(dot)
            })
            .collect();
        line.wait_for(stage, "counting up from one")?;
        fade_in_lagged(stage, &dots, 2.4, 0.06)?;
        line.wait_for(stage, "number one")?;
        let one = kit.text(
            stage,
            &pic.first.to_string(),
            LABEL_SIZE,
            palette::outline(),
            v(x_at(pic.first as f64, opening), STRIP_Y - LABEL_DY),
        )?;
        fade_in_all(stage, &[one], 0.3)?;
        line.wait_for(stage, "number two")?;
        // staggered over the line: two labels 0.15 apart would touch
        let two = kit.text(
            stage,
            &pic.second.to_string(),
            LABEL_SIZE,
            palette::outline(),
            v(x_at(pic.second as f64, opening), STRIP_Y + LABEL_DY),
        )?;
        fade_in_all(stage, &[two], 0.3)?;
        line.wait_for(stage, "nothing is skipped")?;
        flash_lagged(stage, &dots[..12], 1.2, 0.15)?;

        // 01_4: each box turns over as its number is read; then a hold to
        // read all five
        let line = say(&kit, stage, "01_4")?;
        let mut labels = Vec::new();
        for (i, (&n, word)) in pic.numbers.iter().zip(NUMBER_WORDS).enumerate() {
            wait_until(stage, line.at(word)? - FLIP_LEAD)?;
            let [body, lid] = boxes[i];
            squash_all(stage, &[body, lid], SQUASH, FLIP_S / 2.0)?;
            let label = kit.text(stage, &n.to_string(), NUMBER_SIZE, palette::seen(), band(i))?;
            play!(stage;
                body.animate()
                    .set_anim_args(AnimateArgs { run_time: Some(FLIP_S / 2.0), ..AnimateArgs::default() })?
                    .stretch(1.0 / SQUASH, 1)?,
                fade_out(stage.arena_mut(), lid, ORIGIN, 1.0)?.rt(FLIP_S / 2.0),
                fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(FLIP_S / 2.0),
            );
            labels.push(label);
        }
        wait_until(stage, line.start + line.len)?;
        kit.music_breath(stage, HOLD_S)?;
        let bodies: Vec<Mob> = boxes.iter().map(|b| b[0]).collect();

        // 01_5: each box onto its own number, keeping its number, from the
        // right; its dot turns gold and a tick joins them; then a ripple
        // along the line
        let line = say(&kit, stage, "01_5")?;
        line.wait_for(stage, "Put each box")?;
        for i in place_order(pic.numbers.len()) {
            let spot = box_spot(i, pic.numbers[i], opening, 0.0);
            move_all(stage, &[(bodies[i], spot), (labels[i], spot)], SMALL, BOX_MOVE_S)?;
        }
        let tail_dashes = dash_spans(x_at(pic.at_least as f64 + 1.0, pic.thousand as f64), STRIP_X1).len();
        let zoomed = pic.thousand as f64;
        let base: Vec<Mob> = (0..BASE_PIECES)
            .map(|k| {
                let o = base_piece(k, LINE_DOTS, zoomed).map_or(DOT_OPACITY, |(a, b)| dot_opacity((a + b) / 2.0));
                thin(stage, palette::line(), BASE_W, o)
            })
            .collect();
        let ticks: Vec<Mob> = (0..pic.numbers.len())
            .map(|_| thin(stage, palette::seen(), TICK_W, 1.0))
            .collect();
        let certain = thin(stage, palette::certain(), BRACKET_W, 1.0);
        let dashes: Vec<Mob> = (0..tail_dashes)
            .map(|_| thin(stage, palette::top_gap(), BRACKET_W, 1.0))
            .collect();
        let (end, fan, rise) = (
            stage.add_value_tracker(opening),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let bound = Bound {
            numbers: pic.numbers.clone(),
            first: pic.first,
            at_least: pic.at_least as f64,
            dots: dots.clone(),
            base: base.clone(),
            bodies: bodies.clone(),
            labels: labels.clone(),
            ticks: ticks.clone(),
            certain,
            dashes: dashes.clone(),
        };
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let value = |m: Mob| s.tracker_value(m).unwrap_or(0.0);
                let now = [value(end), value(fan), value(rise)];
                if last == Some(now) {
                    return;
                }
                bound.place(s, now[0], now[1], now[2]);
                last = Some(now);
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let gold: Vec<(Mob, Srgb, f64)> = pic
            .numbers
            .iter()
            .map(|&n| (dots[(n - pic.first) as usize], palette::seen(), 1.0))
            .collect();
        refill(stage, &gold, 0.4)?;
        for &t in &ticks {
            stage.add_to_scene(t)?;
        }
        play!(stage; tween(rise, 1.0, RISE_S)?);
        line.wait_for(stage, "every number the company could have printed")?;
        flash_lagged(stage, &dots, 1.6, 0.04)?;

        // 01_6: the prediction, and the countdown
        say(&kit, stage, "01_6")?;
        let countdown = think(&kit, stage, at(THINK_AT))?;

        // 01_7: at least sixty: a bracket from one to sixty, the panel's
        // certain fact; box sixty; the dots under the bracket brighten
        let line = say(&kit, stage, "01_7")?;
        let certain_text = kit.text(
            stage,
            &format!("{AT_LEAST} {}", pic.at_least),
            PANEL_SIZE,
            palette::certain(),
            v(PANEL_X, CERTAIN_Y),
        )?;
        fade_out_all(stage, &[countdown], 0.2)?;
        fade_in_all(stage, &[certain, certain_text], 0.6)?;
        line.wait_for(stage, "Box sixty exists")?;
        flash_all(stage, &[bodies[largest], labels[largest]], 0.6)?;
        line.wait_for(stage, "boxes one to sixty")?;
        let lit: Vec<(Mob, Srgb, f64)> = (pic.first..=pic.at_least)
            .filter(|n| !pic.numbers.contains(n))
            .map(|n| (dots[(n - pic.first) as usize], palette::line(), LIT_OPACITY))
            .collect();
        refill(stage, &lit, 0.6)?;

        // 01_8: above sixty nobody knows: a dashed bracket from sixty-one off
        // the faded end, and its "?"; then the boxes spread into their row
        // and the line zooms out to a thousand under them
        let line = say(&kit, stage, "01_8")?;
        let shown = dash_spans(x_at(pic.at_least as f64 + 1.0, opening), STRIP_X1).len();
        let mut tail: Vec<Mob> = dashes[..shown].iter().rev().copied().collect();
        let tail_q = kit.text(
            stage,
            UNKNOWN,
            PANEL_SIZE,
            palette::top_gap(),
            v(TAIL_X, STRIP_Y - BRACKET_DY - 0.35),
        )?;
        tail.push(tail_q);
        fade_out_all(stage, &[one, two], 0.3)?;
        fade_in_lagged(stage, &tail, 0.8, 0.2)?;
        // the dashes still off the frame join the bracket as it grows
        for &m in &dashes[shown..] {
            stage.add_to_scene(m)?;
        }
        for &m in &base {
            stage.add_to_scene(m)?;
        }
        // the boxes spread while the line still shows its opening numbers;
        // the end moves under them on "hundreds more"
        line.wait_for(stage, "or run on")?;
        for leg in zoom_legs([opening, 0.0], [zoomed, 1.0]) {
            if matches!(leg, Leg::End(_)) {
                line.wait_for(stage, "hundreds more")?;
            }
            play_legs(stage, end, fan, &[leg], FAN_S, ZOOM_S)?;
        }

        // 01_9: the zoomed line's marks and its end; would five from a
        // thousand look like these? The five boxes and their ticks flash
        let line = say(&kit, stage, "01_9")?;
        line.wait_for(stage, "a thousand boxes")?;
        let mut marks = Vec::new();
        for &n in &pic.marks {
            let x = x_at(n as f64, zoomed);
            marks.push(kit.text(
                stage,
                &thousands(n),
                LABEL_SIZE,
                palette::outline(),
                v(x, STRIP_Y - LABEL_DY),
            )?);
        }
        marks.push(kit.text(
            stage,
            &format!("{}?", thousands(pic.thousand)),
            SCALE_SIZE,
            palette::outline(),
            v(STRIP_X1 - END_LABEL_DX, STRIP_Y + END_LABEL_DY),
        )?);
        fade_in_lagged(stage, &marks, 0.8, 0.2)?;
        line.wait_for(stage, "look like these")?;
        let mut five = bodies.clone();
        five.extend(&ticks);
        flash_all(stage, &five, 0.7)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;
    use crate::chapters::draw::{BOX_H, BOX_RISE, BOX_W, FAN_STEP, FAN_X0, THINK_R, all_clear, leg_views};
    use crate::model::{BIG, FRIENDS};

    /// Views sampled along each move: enough that a box crossing another
    /// at the moves' speeds cannot slip between two samples.
    const STEPS: usize = 400;

    /// Every state 01_5 passes through when the boxes go onto the line in
    /// `order` (band slots), one at a time: the moving box slides in a
    /// straight line from its slot in the band to its spot over its number,
    /// shrinking at the same pace (`move_all`'s one transform); the boxes
    /// already down are small on their spots, the rest full size in the
    /// band. Each state is every box's centre and scale.
    fn placing(order: &[usize]) -> Vec<Vec<(Vec3, f64)>> {
        let end = LINE_DOTS as f64;
        let mut states = Vec::new();
        for (j, &moving) in order.iter().enumerate() {
            for s in 0..=STEPS {
                let a = s as f64 / STEPS as f64;
                let boxes = (0..FRIENDS.len())
                    .map(|b| {
                        let (from, to) = (band(b), box_spot(b, FRIENDS[b], end, 0.0));
                        if b == moving {
                            let at = v(from[0] + (to[0] - from[0]) * a, from[1] + (to[1] - from[1]) * a);
                            (at, 1.0 + (SMALL - 1.0) * a)
                        } else if order[..j].contains(&b) {
                            (to, SMALL)
                        } else {
                            (from, 1.0)
                        }
                    })
                    .collect();
                states.push(boxes);
            }
        }
        states
    }

    /// The small boxes over the line in the view `[end, fan]`.
    fn on_line(view: [f64; 2]) -> Vec<(Vec3, f64)> {
        FRIENDS
            .iter()
            .enumerate()
            .map(|(i, &n)| (box_spot(i, n, view[0], view[1]), SMALL))
            .collect()
    }

    /// The boxes never meet on the way, not only where they stop: in every
    /// state of 01_5's moves onto the line, in the scene's order, and of
    /// 01_8's zoom, in the legs the scene plays. The negative controls are
    /// the two trajectories the first opening preview showed colliding
    /// (left to right, 51 meeting 60 still in the band; the end and the fan
    /// tweened together, 27 meeting 33): the same check refuses both.
    #[test]
    fn the_boxes_never_meet_on_the_way() {
        let order = place_order(FRIENDS.len());
        assert_eq!(order.len(), FRIENDS.len());
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..FRIENDS.len()).collect::<Vec<_>>(), "every box once");
        for (k, state) in placing(&order).iter().enumerate() {
            assert!(all_clear(state), "01_5 state {k}: {state:?}");
        }
        let left_to_right: Vec<usize> = (0..FRIENDS.len()).collect();
        assert!(
            !placing(&left_to_right).iter().all(|s| all_clear(s.as_slice())),
            "the preview's order collides"
        );

        let (opening, zoomed) = (LINE_DOTS as f64, BIG as f64);
        let (from, to) = ([opening, 0.0], [zoomed, 1.0]);
        let views = leg_views(from, &zoom_legs(from, to), STEPS);
        assert_eq!(*views.last().unwrap(), to, "the zoom ends where 01_9 reads it");
        for &view in &views {
            assert!(all_clear(&on_line(view)), "01_8 view {view:?}");
        }
        let together = (0..=STEPS).map(|s| {
            let a = s as f64 / STEPS as f64;
            [opening + (zoomed - opening) * a, a]
        });
        assert!(
            together.map(on_line).any(|boxes| !all_clear(&boxes)),
            "the preview's zoom collides"
        );
    }

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let mut waits = vec![
            ("01_1", "how many copies"),
            ("01_2", "Five friends"),
            ("01_2", "a few dozen"),
            ("01_2", "their boxes are rare"),
            ("01_2", "tens of thousands"),
            ("01_2", "they're ordinary"),
            ("01_3", "counting up from one"),
            ("01_3", "number one"),
            ("01_3", "number two"),
            ("01_3", "nothing is skipped"),
            ("01_5", "Put each box"),
            ("01_5", "every number the company could have printed"),
            ("01_7", "Box sixty exists"),
            ("01_7", "boxes one to sixty"),
            ("01_8", "or run on"),
            ("01_8", "hundreds more"),
            ("01_9", "a thousand boxes"),
            ("01_9", "look like these"),
        ];
        waits.extend(NUMBER_WORDS.iter().map(|&w| ("01_4", w)));
        for (id, phrase) in waits {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The English name of a number below a hundred, as the script writes
    /// it.
    fn spelled(n: u64) -> String {
        const ONES: [&str; 20] = [
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];
        const TENS: [&str; 10] = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        match n {
            0..20 => ONES[n as usize].to_string(),
            20..100 if n.is_multiple_of(10) => TENS[(n / 10) as usize].to_string(),
            20..100 => format!("{}-{}", TENS[(n / 10) as usize], ONES[(n % 10) as usize]),
            _ => panic!("{n} is not below a hundred"),
        }
    }

    /// Each box turns over on its own number's name: the words are the five
    /// numbers, in the order 01_4 reads them, which is the boxes' order.
    #[test]
    fn each_box_turns_over_as_its_number_is_read() {
        let pic = picture().unwrap();
        for (&n, word) in pic.numbers.iter().zip(NUMBER_WORDS) {
            assert_eq!(word.to_lowercase(), spelled(n), "{n}");
        }
        let text = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .text("01_4")
            .unwrap()
            .to_owned();
        let places: Vec<usize> = NUMBER_WORDS.iter().map(|w| text.find(w).unwrap()).collect();
        assert!(places.is_sorted(), "read in order: {places:?}");
        // the spelling itself, at its edges
        assert_eq!(spelled(14), "fourteen");
        assert_eq!(spelled(60), "sixty");
        assert_eq!(spelled(51), "fifty-one");
        assert_eq!(spelled(19), "nineteen");
        assert_eq!(spelled(20), "twenty");
    }

    /// The picture is the facts': the five numbers in order, the bracket
    /// ending at the largest (the lower bound) and the tail starting one
    /// past it, the zoom reaching the thousand 01_9 names.
    #[test]
    fn the_picture_is_the_facts() {
        let f = facts::facts().unwrap();
        let pic = picture().unwrap();
        assert_eq!(pic.numbers, FRIENDS.to_vec());
        assert_eq!(f.get("G-F1.numbers"), Some("14,27,33,51,60"));
        assert_eq!(pic.at_least, 60);
        assert_eq!(f.get("G-F1.largest"), Some("60"));
        assert_eq!(pic.thousand, BIG);
        // the line's first two labels and the zoomed line's marks are facts
        assert_eq!((pic.first, pic.second), (1, 2));
        assert_eq!((f.get("G-F1.first"), f.get("G-F1.second")), (Some("1"), Some("2")));
        assert_eq!(pic.marks, [1, 250, 500, 750]);
        assert_eq!(f.get("G-F1.zoom_marks"), Some("1,250,500,750"));
        // the bracket and the tail meet with nothing between them
        let certain = bracket_points(pic.first as f64, pic.at_least as f64, LINE_DOTS as f64);
        let tail = dash_spans(x_at(pic.at_least as f64 + 1.0, LINE_DOTS as f64), STRIP_X1);
        assert!(certain[2][0] < tail.last().unwrap().0);
        assert!((tail.last().unwrap().0 - x_at(61.0, LINE_DOTS as f64)).abs() < 1e-9);
        // negative controls: the check refuses a picture it could not draw
        // truthfully, and accepts the facts' own
        let good = || Picture {
            numbers: FRIENDS.to_vec(),
            first: 1,
            second: 2,
            at_least: 60,
            thousand: 1000,
            marks: vec![1, 250, 500, 750],
        };
        let make = |numbers: &[u64], at_least: u64, thousand: u64| Picture {
            numbers: numbers.to_vec(),
            at_least,
            thousand,
            ..good()
        };
        assert!(checked(good()).is_ok());
        assert!(checked(make(&[60, 27, 33, 51, 14], 14, 1000)).is_err(), "out of order");
        assert!(
            checked(make(&[14, 27, 27, 51, 60], 60, 1000)).is_err(),
            "the same box twice"
        );
        assert!(checked(make(&[14, 27, 33, 51], 51, 1000)).is_err(), "four boxes");
        assert!(checked(make(&FRIENDS, 59, 1000)).is_err(), "a bound below the largest");
        assert!(checked(make(&FRIENDS, 61, 1000)).is_err(), "a bound above the largest");
        assert!(
            checked(make(&[14, 27, 33, 51, 90], 90, 1000)).is_err(),
            "past the opening line"
        );
        assert!(
            checked(make(&FRIENDS, 60, LINE_DOTS)).is_err(),
            "a zoom that goes nowhere"
        );
        // the labels can't drift from the line's map: it starts at 1 and
        // skips nothing, and no box sits on a labelled number
        assert!(checked(Picture { first: 0, ..good() }).is_err(), "a line from 0");
        assert!(
            checked(Picture {
                first: 2,
                second: 3,
                ..good()
            })
            .is_err(),
            "a line from 2"
        );
        assert!(checked(Picture { second: 3, ..good() }).is_err(), "a skipped number");
        assert!(
            checked(make(&[2, 27, 33, 51, 60], 60, 1000)).is_err(),
            "a box on the label 2"
        );
        // nor the marks from the zoom: a new extent needs its own marks
        assert!(
            checked(Picture {
                thousand: 2000,
                ..good()
            })
            .is_err(),
            "marks of the old zoom"
        );
        assert!(
            checked(Picture {
                marks: vec![1, 250, 500, 760],
                ..good()
            })
            .is_err(),
            "a moved mark"
        );
        assert!(
            checked(Picture {
                thousand: 1002,
                ..good()
            })
            .is_err(),
            "marks that aren't whole"
        );
        assert!(
            checked(Picture {
                thousand: 2000,
                marks: vec![1, 500, 1000, 1500],
                ..good()
            })
            .is_ok(),
            "a zoom to 2000 with its own marks"
        );
    }

    /// The band, the stakes scale and the panel stay inside the title-safe
    /// area and apart; the countdown clears the band, the panel and the
    /// boxes on the line; the zoomed line's marks and end label clear each
    /// other. Text widths are estimates (0.13 per character at size 24,
    /// scaled with size): the layout lint measures the real ones.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let width = |s: &str, size: f64| s.chars().count() as f64 * 0.13 * size / 24.0;
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.15;
        // the band
        assert!(BAND_Y + BOX_H / 2.0 < safe_y);
        assert!(BAND_X[0] - BOX_W / 2.0 > -safe_x);
        for w in BAND_X.windows(2) {
            assert!(w[1] - w[0] > BOX_W + 0.2);
        }
        assert_eq!(BAND_X[FIRST_SLOT], FIRST_AT[0]);
        // the stakes scale under the band, its labels inside the frame
        const { assert!(MARKER_Y + 0.25 < BAND_Y - BOX_H / 2.0) };
        assert!(BAND_X[0] - width(FEW, SCALE_SIZE) / 2.0 > -safe_x);
        let many_right = BAND_X[4] + width(MANY, SCALE_SIZE) / 2.0;
        const { assert!(SCALE_NOTE_Y - 0.2 > STRIP_Y + 0.3, "the scale clears the line") };
        // the panel and the countdown on the right
        let panel_left = PANEL_X - width("at least 60", PANEL_SIZE) / 2.0;
        assert!(many_right + 0.3 < panel_left);
        assert!(PANEL_X + width("at least 60", PANEL_SIZE) / 2.0 < safe_x);
        assert!(QUESTION_Y + 0.2 < safe_y);
        const { assert!(THINK_AT[1] + THINK_R + 0.15 < CERTAIN_Y - 0.2) };
        let last_box = box_spot(4, FRIENDS[4], LINE_DOTS as f64, 0.0);
        assert!(last_box[0] + SMALL * BOX_W / 2.0 + 0.3 < THINK_AT[0] - THINK_R);
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > STRIP_Y + 0.3, "the note clears the line") };
        // the first box and its question
        assert!(FIRST_AT[0] + BOX_W / 2.0 + 0.2 < HOW_MANY_AT[0] - width(HOW_MANY, PANEL_SIZE) / 2.0);
        // the zoomed line's marks apart, its end label inside the frame
        let zoomed = BIG as f64;
        let pic = picture().unwrap();
        for w in pic.marks.windows(2) {
            let (a, b) = (x_at(w[0] as f64, zoomed), x_at(w[1] as f64, zoomed));
            assert!(b - a > width("750", LABEL_SIZE) + 0.3);
        }
        assert!(STRIP_X1 - END_LABEL_DX + width("1,000?", SCALE_SIZE) / 2.0 < safe_x);
        // the fanned boxes sit over the line and its marks under it, so the
        // fan (which reaches past the mark at 250) never meets them
        const { assert!(STRIP_Y + BOX_RISE - SMALL * BOX_H / 2.0 > STRIP_Y - LABEL_DY + 0.15) };
        let fan_right = FAN_X0 + FAN_STEP * 4.0 + SMALL * BOX_W / 2.0;
        assert!(
            fan_right < STRIP_X1 - END_LABEL_DX - width("1,000?", SCALE_SIZE) / 2.0,
            "nor the end label"
        );
        // the tail's "?" sits under its bracket on both lines
        for end in [LINE_DOTS as f64, zoomed] {
            assert!(TAIL_X > x_at(61.0, end) && TAIL_X < STRIP_X1);
        }
    }
}
