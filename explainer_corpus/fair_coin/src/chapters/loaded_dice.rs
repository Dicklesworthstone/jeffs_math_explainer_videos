//! Chapter 08, loaded dice: the coin is swapped for a loaded die, six bars
//! for its faces' chances (08_1); Alice and Bob want to settle their bet
//! with it (08_2); the square again, cut six ways along each side by those
//! chances, the six nearest the corner: a match is thrown away, Alice has
//! the blocks above the diagonal and Bob those below (08_3); one block
//! flips onto its twin, then all fifteen as one sheet (08_4); and the cuts
//! slide together until the thirty-six blocks are the coin's four (08_5).

use super::draw::predict_pause;
use explainer_kit::{BACKGROUND, Kit, Palette, Timed, fade_all, hex, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{
    COLUMN_LEFT, COLUMN_MID, SQUARE_AT, SQUARE_SIDE, alice, block_grey, bob, caption, column, die_face, fade_in_all,
    fade_out_all, faded_line, fail, flat, gold, group, heads, lettered_disc, line_style, pair_tile, panel, pause, say,
    tag, tails, thrown, wait_until,
};
use crate::facts::loaded_run;
use crate::model::{Face, LOADED, PairLaw, Q, bent, die, tally};

pub struct LoadedDice {
    pub kit: Kit,
}

/// The two rolls the narration follows across the diagonal (08_4): a six
/// then a five.
const TWIN: (usize, usize) = (6, 5);

// the chapter's fixed words (numbers are built from facts)
const MODEL_DIE: &str = "a model die (one we made up)";
const COUNTED_IN: &str = "counted in";
const ROLLS: &str = "rolls:";
const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const QUESTION: &str = "How would you settle their bet with a loaded die?";
const FIRST_ROLL: &str = "first roll";
const SECOND_ROLL: &str = "second roll";
const ALICE_WINS: &str = "Alice: the first roll is bigger";
const BOB_WINS: &str = "Bob: the second roll is bigger";
const MATCH: &str = "a match: roll two fresh ones";
/// The same three rows while the picture is the coin's (08_5).
const ALICE_COIN: &str = "Alice: heads then tails";
const BOB_COIN: &str = "Bob: tails then heads";
const MATCH_COIN: &str = "a match: toss two fresh ones";
const EQUALS: &str = "=";
const PAIRS_OF_ROLLS: &str = "pairs of rolls:";
const FOR_ALICE: &str = "for Alice,";
const FOR_BOB: &str = "for Bob,";
const MATCHES: &str = "matches";
const HEADS: &str = "heads";
const TAILS: &str = "tails";
const SAME_TWO: &str = "the same two results, either order: the same chance";

// 08_1 and 08_2: the die, its six bars under it, a player either side
const DIE_AT: Vec3 = [0.0, 1.75, 0.0];
const DIE: f64 = 1.4;
const CHART_PITCH: f64 = 0.9;
const CHART_FOOT: f64 = -1.5;
/// How tall the bar of a face that always came up would stand.
const CHART_UNIT: f64 = 3.7;
const PLAYER_X: f64 = 5.2;
const HEAD_Y: f64 = 1.6;
// the square's edges: how far outside the square a stretch lies, and its thickness
const EDGE_OFF: f64 = 0.065;
const EDGE_THICK: f64 = 0.07;
// the two pairs beside the square (08_4, 08_5), as chapter 11 sets them:
// one tile each side of a gold "=", its chance under it
const TILE_L: f64 = COLUMN_MID - 1.25;
const TILE_R: f64 = COLUMN_MID + 1.25;
const TILES_Y: f64 = 0.2;
const SHARE_DY: f64 = 0.68;
/// How long the cuts take to close into the coin's (eased), how long the
/// coin's square is held, and how long they take to open again (08_5).
const CLOSE_S: f64 = 2.0;
const COIN_HOLD_S: f64 = 3.0;
const OPEN_S: f64 = 1.2;

/// Where the chart's bar for `face` (1 to 6) stands.
fn chart_x(face: usize, faces: usize) -> f64 {
    (face as f64 - (faces as f64 + 1.0) / 2.0) * CHART_PITCH
}

/// A point of the square: `u` across and `w` up, both 0 to 1.
fn at(u: f64, w: f64) -> Vec3 {
    v(SQUARE_AT[0] + SQUARE_SIDE * u, SQUARE_AT[1] + SQUARE_SIDE * w)
}

/// A bar standing on `foot` whose height follows `tracker` (`unit` scene
/// units for each 1.0). Returns the anchor, which carries the updater and
/// must be in the scene, and the bar; both detached.
fn rising_bar(
    stage: &mut Stage<'_>,
    tracker: Mob,
    foot: Vec3,
    width: f64,
    unit: f64,
    color: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let bar = stage.arena_mut().add(column(foot, width, 0.0, color));
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let value = s.tracker_value(tracker).unwrap_or(0.0);
            if last == Some(value) {
                return;
            }
            let _ = s.set_points(bar, column(foot, width, unit * value, color).points());
            last = Some(value);
        },
        true,
    )?;
    Ok([anchor, bar])
}

/// Alice or Bob as chapter 01 draws them: a head and shoulders in the
/// player's colour, two eyes turned on the die, a name. Detached parts.
fn player(kit: &Kit, stage: &mut Stage<'_>, name: &str, color: Srgb, x: f64) -> fmn::Result<Vec<Mob>> {
    let toward = if x < 0.0 { 1.0 } else { -1.0 };
    let (head_r, shoulders_r) = (0.46, 0.74);
    let base = HEAD_Y - 1.18;
    let mut arc = Vec::new();
    for k in 0..=24 {
        let a = PI * f64::from(k) / 24.0;
        arc.push(v(x + shoulders_r * a.cos(), base + shoulders_r * a.sin()));
    }
    let shoulders = Polygon::new(arc).style(flat(color, 1.0)).build();
    let head = Circle::new()
        .radius(head_r)
        .style(flat(color, 1.0))
        .build()
        .moved_to(v(x, HEAD_Y));
    let name = kit.text(stage, name, 28.0, color, v(x, base - 0.38))?;
    let mut parts = vec![stage.arena_mut().add(shoulders), stage.arena_mut().add(head), name];
    for side in [-1.0, 1.0] {
        let c = v(x + 0.17 * side, HEAD_Y + 0.06);
        let white = Ellipse::new()
            .width(0.2)
            .height(0.25)
            .style(flat(Palette::ink(), 1.0))
            .build()
            .moved_to(c);
        parts.push(stage.arena_mut().add(white));
        let pupil = Dot::new()
            .point(v(c[0] + 0.045 * toward, c[1] + 0.02))
            .radius(0.055)
            .style(flat(hex(BACKGROUND), 1.0))
            .build();
        parts.push(stage.arena_mut().add(pupil));
    }
    Ok(parts)
}

/// The stretch of an edge between `a` and `b` (0 to 1 along the side), as a
/// thin bar just outside the square: under the bottom edge, or beside the
/// left one. Corners run as a `column`'s do, so a bar can morph into it.
fn edge_bar((a, b): (f64, f64), left: bool, color: Srgb) -> VMobject {
    let gap = 0.022;
    let (lo, hi) = (SQUARE_SIDE * a + gap, SQUARE_SIDE * b - gap);
    let quad = if left {
        let x1 = SQUARE_AT[0] - EDGE_OFF;
        let (x0, y) = (x1 - EDGE_THICK, SQUARE_AT[1]);
        [v(x0, y + lo), v(x1, y + lo), v(x1, y + hi), v(x0, y + hi)]
    } else {
        let y1 = SQUARE_AT[1] - EDGE_OFF;
        let (y0, x) = (y1 - EDGE_THICK, SQUARE_AT[0]);
        [v(x + lo, y0), v(x + hi, y0), v(x + hi, y1), v(x + lo, y1)]
    };
    Polygon::new(quad).style(flat(color, 1.0)).build()
}

/// The square cut six ways along each side: thirty-six blocks, the cuts
/// between them and a thin frame, drawn from one tracker that slides every
/// cut towards the coin's single cut (0: the die; 1: a die with two faces).
/// Block (i, j) holds the first roll's stretch `i` and the second's `j`,
/// counted from the corner, where the biggest roll's stretch lies: so the
/// blocks with i < j, the first roll bigger, are above the diagonal.
struct Grid {
    merge: Mob,
    /// Block (i, j) is `blocks[i * n + j]`.
    blocks: Vec<Mob>,
    cuts: Vec<Mob>,
    frame: Mob,
    /// Carries the updater: it must be in the scene for the cuts to slide.
    anchor: Mob,
    /// The die's cuts, both ends included.
    die_cuts: Vec<f64>,
}

/// The cuts when they have slid `t` of the way to the coin's.
fn cuts_at(die_cuts: &[f64], coin: f64, t: f64) -> Vec<f64> {
    let last = die_cuts.len() - 1;
    die_cuts
        .iter()
        .enumerate()
        .map(|(k, &c)| if k == 0 || k == last { c } else { c + (coin - c) * t })
        .collect()
}

fn quad((u0, w0, u1, w1): (f64, f64, f64, f64)) -> [Vec3; 4] {
    [at(u0, w0), at(u1, w0), at(u1, w1), at(u0, w1)]
}

impl Grid {
    /// The grid for a die with these chances (face 1 first), the biggest
    /// face's stretch nearest the corner; `coin` is where its cuts gather.
    /// Detached; blocks neutral.
    fn build(stage: &mut Stage<'_>, chances: &[Q], coin: f64) -> fmn::Result<Self> {
        let mut die_cuts = vec![0.0];
        let mut upto = Q::ZERO;
        for &chance in chances.iter().rev() {
            upto = upto + chance;
            die_cuts.push(upto.f64());
        }
        if upto != Q::ONE {
            return Err(fail("the die's chances do not add up to one".into()));
        }
        let n = chances.len();
        let unit = [v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)];
        let blocks: Vec<Mob> = (0..n * n)
            .map(|_| {
                stage
                    .arena_mut()
                    .add(Polygon::new(unit).style(flat(block_grey(), 1.0)).build())
            })
            .collect();
        let cut_style = line_style(hex(BACKGROUND), 3.0);
        let cuts: Vec<Mob> = (0..2 * (n - 1))
            .map(|_| {
                stage
                    .arena_mut()
                    .add(polyline(vec![v(0.0, 0.0), v(1.0, 0.0)]).style(cut_style).build())
            })
            .collect();
        let frame = stage.arena_mut().add(
            Polygon::new(quad((0.0, 0.0, 1.0, 1.0)))
                .style(faded_line(Palette::dim(), 2.0, 0.9))
                .build(),
        );
        let merge = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let (block_mobs, cut_mobs, from) = (blocks.clone(), cuts.clone(), die_cuts.clone());
        let mut last = None;
        stage.add_updater(
            anchor,
            move |s, _| {
                let t = s.tracker_value(merge).unwrap_or(0.0).clamp(0.0, 1.0);
                if last == Some(t) {
                    return;
                }
                let c = cuts_at(&from, coin, t);
                for i in 0..n {
                    for j in 0..n {
                        let q = quad((c[i], c[j], c[i + 1], c[j + 1]));
                        let _ = s.set_points(block_mobs[i * n + j], Polygon::new(q).build().points());
                    }
                }
                for k in 1..n {
                    let lines = [[at(c[k], 0.0), at(c[k], 1.0)], [at(0.0, c[k]), at(1.0, c[k])]];
                    for (line, &m) in lines.iter().zip(&cut_mobs[2 * (k - 1)..]) {
                        let _ = s.set_points(m, polyline(line.to_vec()).build().points());
                    }
                }
                last = Some(t);
            },
            true,
        )?;
        Ok(Self {
            merge,
            blocks,
            cuts,
            frame,
            anchor,
            die_cuts,
        })
    }

    fn n(&self) -> usize {
        self.die_cuts.len() - 1
    }

    /// Everything to bring into the scene, back to front.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.anchor];
        out.extend(&self.blocks);
        out.extend(&self.cuts);
        out.push(self.frame);
        out
    }

    /// The stretch of a side that `face` (1 to n) has: the biggest face's
    /// lies nearest the corner.
    fn stretch(&self, face: usize) -> (f64, f64) {
        let k = self.n() - face;
        (self.die_cuts[k], self.die_cuts[k + 1])
    }

    /// The block for a first roll `first` and a second roll `second`, as
    /// the die stands: (across from, up from, across to, up to).
    fn rect(&self, first: usize, second: usize) -> (f64, f64, f64, f64) {
        let ((u0, u1), (w0, w1)) = (self.stretch(first), self.stretch(second));
        (u0, w0, u1, w1)
    }

    fn block(&self, first: usize, second: usize) -> Mob {
        let n = self.n();
        self.blocks[(n - first) * n + (n - second)]
    }

    /// Paint these blocks one after another, nearest the corner first, in
    /// `rt` seconds. The blocks are recoloured in place by an updater, not
    /// animated: an animated mobject is drawn last, which would bury the
    /// cuts under it.
    fn paint(&self, stage: &mut Stage<'_>, rolls: &[(usize, usize)], color: Srgb, rt: f64) -> fmn::Result<()> {
        let mut order = rolls.to_vec();
        order.sort_by_key(|&(a, b)| std::cmp::Reverse(a + b));
        let blocks: Vec<Mob> = order.iter().map(|&(a, b)| self.block(a, b)).collect();
        // each block takes three places of the sweep to change colour
        let (places, each) = (blocks.len() as f64 + 2.0, 3.0);
        let sweep = stage.add_value_tracker(0.0);
        let driver = stage.arena_mut().add(VMobject::new());
        stage.add_updater(
            driver,
            move |s, _| {
                let at = s.tracker_value(sweep).unwrap_or(0.0) * places;
                for (k, &m) in blocks.iter().enumerate() {
                    let t = ((at - k as f64) / each).clamp(0.0, 1.0);
                    let c = fmn::core::color::interpolate_color(block_grey(), color, t);
                    s.set_fill(m, Some(c), Some(1.0), None, false);
                }
            },
            true,
        )?;
        stage.add_to_scene(driver)?;
        play!(stage; tween(sweep, 1.0, rt)?);
        stage.clear_updaters(driver, true);
        Ok(())
    }

    /// A still copy of the block for (`first`, `second`), edged in gold, and
    /// the same shape mirrored in the diagonal with its corners in the same
    /// order: `Transform::new(copy, mirror)` turns it over. Both detached.
    fn ghost_and_mirror(&self, stage: &mut Stage<'_>, first: usize, second: usize, color: Srgb, edge: f64) -> [Mob; 2] {
        let (u0, w0, u1, w1) = self.rect(first, second);
        let style = Style::default().fill(color, 0.92).stroke(gold(), edge, 1.0);
        let ghost = Polygon::new(quad((u0, w0, u1, w1))).style(style).build();
        let mirror = Polygon::new([at(w0, u0), at(w0, u1), at(w1, u1), at(w1, u0)])
            .style(style)
            .build();
        [stage.arena_mut().add(ghost), stage.arena_mut().add(mirror)]
    }
}

/// Two die faces side by side on a small card, as `draw::pair_tile` sets
/// two results (chapter 11 draws its dice the same way). Detached, one group.
fn dice_tile(stage: &mut Stage<'_>, rolls: (usize, usize), at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(1.38)
        .height(0.82)
        .corner_radius(0.1)
        .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
        .build()?
        .moved_to(at);
    let mut parts = vec![stage.arena_mut().add(back)];
    for (value, dx) in [(rolls.0, -0.32), (rolls.1, 0.32)] {
        parts.push(die_face(
            stage,
            value,
            v(at[0] + dx, at[1]),
            0.5,
            Palette::ink(),
            hex(BACKGROUND),
        )?);
    }
    group(stage, &parts)
}

/// Swap words for words in the same places over `rt` seconds, never both at
/// once: the old leave in the first third, the new arrive in the last.
fn swap_words(stage: &mut Stage<'_>, old: &[Mob], new: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in old {
        let mut a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        a.update_rate_info(Some(rt), Some(RateFunc::smooth().squish(0.0, 0.33)), None);
        out.push(stage.prepare(a)?);
    }
    for &m in new {
        let mut a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        a.update_rate_info(Some(rt), Some(RateFunc::smooth().squish(0.67, 1.0)), None);
        out.push(stage.prepare(a)?);
    }
    Ok(out)
}

/// Where a legend row's words start and sit.
fn legend_at(row: usize) -> Vec3 {
    v(COLUMN_LEFT + 0.7, 2.45 - 0.55 * row as f64)
}

/// One row of the legend beside the square: a swatch and its words.
fn legend_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    row: usize,
    color: Srgb,
    words: &str,
    ink: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let words_at = legend_at(row);
    let swatch = Rectangle::new()
        .width(0.36)
        .height(0.36)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(v(words_at[0] - 0.4, words_at[1]));
    let swatch = stage.arena_mut().add(swatch);
    let text = kit.text_left(stage, words, 26.0, ink, words_at)?;
    Ok([swatch, text])
}

impl SceneConstruct for LoadedDice {
    fn name(&self) -> &str {
        "08_loaded_dice"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dark = hex(BACKGROUND);
        let chances = die(&LOADED);
        let faces = chances.len();
        let run = loaded_run();
        let counts = tally(&run.iter().map(|r| r - 1).collect::<Vec<_>>(), faces);

        // the coin, about to be swapped
        let coin = lettered_disc(&kit, stage, Face::H, DIE_AT, DIE / 2.0, 80.0)?;
        fade_in_all(stage, &coin, 0.5)?;

        // 08_1: a loaded die: six bars for its faces' chances
        let line = say(&kit, stage, "08_1")?;
        let big = die_face(stage, faces, DIE_AT, DIE, Palette::ink(), dark)?;
        play!(stage;
            fade_out(stage.arena_mut(), coin[0], v(-0.9, 0.0), 1.0)?.rt(0.7),
            fade_out(stage.arena_mut(), coin[1], v(-0.9, 0.0), 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), big, v(-0.9, 0.0), 1.0)?.rt(0.7),
        );
        let model = tag(&kit, stage, MODEL_DIE, v(DIE_AT[0], DIE_AT[1] + DIE / 2.0 + 0.42), 22.0)?;
        play!(stage; fade_in(stage.arena_mut(), model, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "This one is loaded")?;
        let mut bars = Vec::new();
        let mut feet = Vec::new();
        let mut percents = Vec::new();
        let mut rises: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &chance) in chances.iter().enumerate() {
            let face = i + 1;
            let x = chart_x(face, faces);
            let t = stage.add_value_tracker(0.0);
            let bar = rising_bar(stage, t, v(x, CHART_FOOT), 0.6, CHART_UNIT, Palette::dim())?;
            for m in bar {
                stage.add_to_scene(m)?;
            }
            bars.push(bar);
            rises.push(stage.prepare(tween(t, chance.f64(), 1.2)?)?);
            let foot = die_face(stage, face, v(x, CHART_FOOT - 0.4), 0.44, Palette::ink(), dark)?;
            let a = fade_in(stage.arena_mut(), foot, ORIGIN, 1.0)?;
            rises.push(stage.prepare(a.rt(0.6))?);
            feet.push(foot);
            let label = format!("{}%", chance.percent(0));
            let label_at = v(x, CHART_FOOT + CHART_UNIT * chance.f64() + 0.25);
            let label = kit.text(stage, &label, 22.0, Palette::dim(), label_at)?;
            let a = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
            rises.push(stage.prepare(a.rt(1.2))?);
            percents.push(label);
        }
        stage.play_prepared(rises)?;
        // the six's bar is pointed at: it and its number light up
        line.wait_for(stage, "a six half the time")?;
        let six = faces - 1;
        play!(stage, 0.6;
            bars[six][1].animate().set_fill(Some(Palette::ink()), Some(1.0))?,
            percents[six].animate().set_fill(Some(Palette::ink()), Some(1.0))?,
        );
        // and what a run of the die counted
        let counted = format!(
            "{COUNTED_IN} {} {ROLLS} {}",
            thousands(run.len() as u64),
            counts
                .iter()
                .map(|&c| thousands(c as u64))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let counted = caption(&kit, stage, &counted)?;
        play!(stage; fade_in(stage.arena_mut(), counted, ORIGIN, 1.0)?.rt(0.6));

        // 08_2: Alice and Bob want to settle their bet with it
        let line = say(&kit, stage, "08_2")?;
        let mut players = player(&kit, stage, ALICE, alice(), -PLAYER_X)?;
        players.extend(player(&kit, stage, BOB, bob(), PLAYER_X)?);
        play!(stage; fade_out(stage.arena_mut(), model, ORIGIN, 1.0)?.rt(0.4));
        fade_in_all(stage, &players, 0.8)?;
        line.wait_for(stage, "How would you do it")?;
        let question = kit.text(stage, QUESTION, 30.0, Palette::ink(), v(0.0, -2.85))?;
        let card = Rectangle::new()
            .width(stage.get_width(question) + 0.8)
            .height(0.86)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
            .build()?
            .moved_to(v(0.0, -2.85));
        let card = stage.arena_mut().add(card);
        let card = group(stage, &[card, question])?;
        play!(stage;
            fade_out(stage.arena_mut(), counted, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), card, v(0.0, 0.15), 1.0)?.rt(0.6),
        );
        let q = predict_pause(&kit, stage, v(DIE_AT[0], DIE_AT[1] + DIE / 2.0 + 0.7))?;

        // 08_3: roll it twice: the square, cut six ways along each side
        let line = say(&kit, stage, "08_3")?;
        let mut asked = players.clone();
        asked.extend([card, q, big]);
        asked.extend(&feet);
        asked.extend(&percents);
        fade_out_all(stage, &asked, 0.4)?;
        let grid = Grid::build(stage, &chances, bent().f64())?;
        // the six bars lie down along the square's two edges, the six
        // nearest the corner: the die's chances are the cuts
        let mut lays: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &grid.mobs() {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            lays.push(stage.prepare(a.rt(0.8))?);
        }
        let mut die_edges = Vec::new();
        for (i, bar) in bars.iter().enumerate() {
            let stretch = grid.stretch(i + 1);
            stage.clear_updaters(bar[0], true);
            let bottom = stage.arena_mut().add(edge_bar(stretch, false, Palette::ink()));
            let left = stage.arena_mut().add(edge_bar(stretch, true, Palette::ink()));
            let a = fmn::animation::transform_from_copy(stage.arena_mut(), bar[1], left)?;
            lays.push(stage.prepare(a.rt(1.3))?);
            lays.push(stage.prepare(Transform::new(bar[1], bottom).rt(1.3))?);
            die_edges.extend([bar[1], left]);
        }
        stage.play_prepared(lays)?;
        let mut names = Vec::new();
        for face in 1..=faces {
            let (a, b) = grid.stretch(face);
            let mid = (a + b) / 2.0;
            let under = v(at(mid, 0.0)[0], SQUARE_AT[1] - 0.4);
            let beside = v(SQUARE_AT[0] - 0.38, at(0.0, mid)[1]);
            for spot in [under, beside] {
                names.push(kit.text(stage, &face.to_string(), 22.0, Palette::ink(), spot)?);
            }
        }
        names.push(kit.text(
            stage,
            FIRST_ROLL,
            22.0,
            Palette::dim(),
            v(at(0.5, 0.0)[0], SQUARE_AT[1] - 0.8),
        )?);
        names.push(kit.text_left(
            stage,
            SECOND_ROLL,
            22.0,
            Palette::dim(),
            v(SQUARE_AT[0] - 0.95, at(0.0, 1.0)[1] + 0.32),
        )?);
        fade_in_all(stage, &names, 0.5)?;
        die_edges.extend(&names);
        // a match is thrown away; Alice above the diagonal, Bob below
        let all_rolls: Vec<(usize, usize)> = (1..=faces).flat_map(|a| (1..=faces).map(move |b| (a, b))).collect();
        let pick = |keep: fn(usize, usize) -> bool| -> Vec<(usize, usize)> {
            all_rolls.iter().copied().filter(|&(a, b)| keep(a, b)).collect()
        };
        let (matches, first_bigger, second_bigger) = (pick(|a, b| a == b), pick(|a, b| a > b), pick(|a, b| b > a));
        line.wait_for(stage, "If the two rolls match")?;
        let match_row = legend_row(&kit, stage, 2, thrown(), MATCH, Palette::dim())?;
        fade_in_all(stage, &match_row, 0.4)?;
        grid.paint(stage, &matches, thrown(), 0.9)?;
        line.wait_for(stage, "Alice wins")?;
        let alice_row = legend_row(&kit, stage, 0, alice(), ALICE_WINS, Palette::ink())?;
        fade_in_all(stage, &alice_row, 0.4)?;
        grid.paint(stage, &first_bigger, alice(), 1.3)?;
        line.wait_for(stage, "and Bob wins")?;
        let bob_row = legend_row(&kit, stage, 1, bob(), BOB_WINS, Palette::ink())?;
        fade_in_all(stage, &bob_row, 0.4)?;
        grid.paint(stage, &second_bigger, bob(), 1.3)?;
        // the legend's words, which 08_5 swaps for the coin's while the
        // picture is the coin's square
        let die_words = [alice_row[1], bob_row[1], match_row[1]];
        // the gold diagonal returns
        let diagonal = DashedLine::new(at(0.0, 0.0), at(1.0, 1.0))
            .dash_length(0.16)
            .style(line_style(gold(), 3.0))
            .build()
            .map_err(|e| fail(format!("the square's diagonal: {e}")))?;
        let diagonal = stage.arena_mut().add(diagonal);
        play!(stage; show_creation(diagonal).rt(0.8));

        // 08_4: the coin trick again: every block has an equal twin
        let line = say(&kit, stage, "08_4")?;
        let w = |face: usize| chances[face - 1];
        // a pair of rolls as a tile, its chance under it
        let twin = |kit: &Kit, stage: &mut Stage<'_>, x: f64, (a, b): (usize, usize)| -> fmn::Result<[Mob; 2]> {
            let chance = format!("{}%", (w(a) * w(b)).percent(0));
            Ok([
                dice_tile(stage, (a, b), v(x, TILES_Y))?,
                kit.text(stage, &chance, 26.0, Palette::ink(), v(x, TILES_Y - SHARE_DY))?,
            ])
        };
        line.wait_for(stage, "A six then a five")?;
        let (first, second) = TWIN;
        let [ghost, mirror] = grid.ghost_and_mirror(stage, first, second, alice(), 3.0);
        let there = twin(&kit, stage, TILE_L, (first, second))?;
        let mut lifts = vec![ghost];
        lifts.extend(there);
        fade_in_all(stage, &lifts, 0.5)?;
        stage.scene_mut().bring_to_front(&[diagonal])?;
        pause(stage, 0.5)?;
        wait_until(stage, line.at("as a five then a six")? - 1.5)?;
        play!(stage; Transform::new(ghost, mirror).rt(1.5));
        let back = twin(&kit, stage, TILE_R, (second, first))?;
        let equals = kit.text(stage, EQUALS, 44.0, gold(), v(COLUMN_MID, TILES_Y))?;
        let mut arrives = back.to_vec();
        arrives.push(equals);
        fade_in_all(stage, &arrives, 0.5)?;
        pause(stage, 0.5)?;
        fade_out_all(stage, &[ghost], 0.4)?;
        // the column beside the square, as it stands for the die
        let mut dice_column = there.to_vec();
        dice_column.extend(arrives);
        // then the whole green triangle, as one sheet
        line.wait_for(stage, "and across the diagonal")?;
        let (mut sheet, mut turns) = (Vec::new(), Vec::new());
        for &(a, b) in &first_bigger {
            let [ghost, mirror] = grid.ghost_and_mirror(stage, a, b, alice(), 1.5);
            sheet.push(ghost);
            turns.push((ghost, mirror));
        }
        fade_in_all(stage, &sheet, 0.3)?;
        stage.scene_mut().bring_to_front(&[diagonal])?;
        let mut flips: Vec<Box<dyn Animation>> = Vec::new();
        for (ghost, mirror) in turns {
            flips.push(stage.prepare(Transform::new(ghost, mirror).rt(1.8))?);
        }
        stage.play_prepared(flips)?;
        // what a run of the die says, read two rolls at a time
        let pairs: Vec<(usize, usize)> = run.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])).collect();
        let tallied = |keep: fn(usize, usize) -> bool| pairs.iter().filter(|&&(a, b)| keep(a, b)).count();
        let counted = format!(
            "{COUNTED_IN} {} {PAIRS_OF_ROLLS} {} {FOR_ALICE} {} {FOR_BOB} {} {MATCHES}",
            thousands(pairs.len() as u64),
            thousands(tallied(|a, b| a > b) as u64),
            thousands(tallied(|a, b| b > a) as u64),
            thousands(tallied(|a, b| a == b) as u64),
        );
        let counted = caption(&kit, stage, &counted)?;
        play!(stage; fade_in(stage.arena_mut(), counted, ORIGIN, 1.0)?.rt(0.6));
        fade_out_all(stage, &sheet, 0.5)?;

        dice_column.push(counted);

        // 08_5: a coin is a die with two faces: the cuts close into the
        // coin's one cut on each side, and everything beside the square
        // becomes the coin's while the picture is the coin's
        say(&kit, stage, "08_5")?;
        let p = bent().f64();
        let mut coin = Vec::new();
        for left in [false, true] {
            coin.push(stage.arena_mut().add(edge_bar((0.0, p), left, heads())));
            coin.push(stage.arena_mut().add(edge_bar((p, 1.0), left, tails())));
        }
        let (below, beside) = (SQUARE_AT[1] - 0.4, SQUARE_AT[0] - 0.55);
        coin.extend([
            kit.text(stage, HEADS, 24.0, heads(), v(at(p / 2.0, 0.0)[0], below))?,
            kit.text(stage, TAILS, 24.0, tails(), v(at((1.0 + p) / 2.0, 0.0)[0], below))?,
            kit.text(stage, HEADS, 24.0, heads(), v(beside, at(0.0, p / 2.0)[1]))?,
            kit.text(stage, TAILS, 24.0, tails(), v(beside, at(0.0, (1.0 + p) / 2.0)[1]))?,
        ]);
        // the coin's two pairs that decide, where the die's two stood
        let law = PairLaw::same_coin(bent());
        for (pair, x) in [((Face::H, Face::T), TILE_L), ((Face::T, Face::H), TILE_R)] {
            let parts = pair_tile(&kit, stage, pair, v(x, TILES_Y), 1.0)?;
            coin.push(group(stage, &parts)?);
            let share = format!("{}%", law.prob(pair.0, pair.1).percent(0));
            coin.push(kit.text(stage, &share, 26.0, Palette::ink(), v(x, TILES_Y - SHARE_DY))?);
        }
        coin.push(kit.text(stage, EQUALS, 44.0, gold(), v(COLUMN_MID, TILES_Y))?);
        let mut coin_words = Vec::new();
        for (row, words) in [ALICE_COIN, BOB_COIN, MATCH_COIN].into_iter().enumerate() {
            let ink = if row == 2 { Palette::dim() } else { Palette::ink() };
            coin_words.push(kit.text_left(stage, words, 26.0, ink, legend_at(row))?);
        }
        // the die's readouts leave before the picture moves (rule 3); the
        // legend's words turn into the coin's while the cuts close
        let mut die_side = die_edges.clone();
        die_side.extend(&dice_column);
        fade_out_all(stage, &die_side, 0.3)?;
        let mut closing = vec![stage.prepare(tween(grid.merge, 1.0, CLOSE_S)?)?];
        closing.extend(swap_words(stage, &die_words, &coin_words, CLOSE_S)?);
        stage.play_prepared(closing)?;
        fade_in_all(stage, &coin, 0.4)?;
        // hold on the coin's four blocks
        pause(stage, COIN_HOLD_S - 0.4)?;
        // then out again, with what both squares say
        fade_out_all(stage, &coin, 0.3)?;
        let same = kit.text(stage, SAME_TWO, 28.0, gold(), v(COLUMN_MID, -1.3))?;
        let mut opening = vec![stage.prepare(tween(grid.merge, 0.0, OPEN_S)?)?];
        let a = fade_in(stage.arena_mut(), same, ORIGIN, 1.0)?;
        opening.push(stage.prepare(a.rt(OPEN_S))?);
        opening.extend(swap_words(stage, &coin_words, &die_words, OPEN_S)?);
        stage.play_prepared(opening)?;
        fade_in_all(stage, &die_side, 0.4)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
