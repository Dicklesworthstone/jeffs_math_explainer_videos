//! Chapter 05, an experiment: feed the sealed method every secret, 0 to
//! 10, with each of the 121 ways its two fair spins could land (05_1); for
//! each secret, count the spin pairs that deal trustees 4 and 5 exactly
//! their 2 and 6, and the question whether some secrets fit more ways than
//! others (05_2); no: every secret fits exactly one, a different one each
//! time (05_3); so with fair spins the pair's two shares are exactly as
//! likely whatever the secret is, and every secret stays as likely as it
//! was (05_4); two shares cannot determine the secret and three can, and the
//! envelope stays sealed (05_5).
//!
//! The grid is `model::tuples` of the dial, two spins long, and each
//! secret's lit cell is `model::fitting_spins` for the pair's dealt shares:
//! the scene stops with an error unless every secret fits exactly one spin
//! pair, no two the same. The tally waits in empty slots through the
//! question. The counter is a shape stepping along the strip's own labels,
//! and every number (a cell's secret, a tally's 1) comes in where there was
//! none, so no number changes in place. The strip keeps chapter 04's
//! "wanted" caption until the experiment is done; then it shows what the
//! pair's shares leave, computed from them, and checks it against the grid.
//!
//! The chance card is the likelihood, the same 1/121 under every secret, so
//! whatever the pair believed about the secret before, it is unchanged: no
//! prior is assumed, and nothing here says "1 in 11 each".

use explainer_kit::props::precisely;
use explainer_kit::{BACKGROUND, Kit, Palette, fade_all, hex, v};
use fmn::prelude::*;

use super::draw::{
    Rail, STRIP_LEFT, Strip, bundle, card, fade_in_all, fade_in_lagged, fail, panel, pause, read_by, recolorings, say,
    strip_right, swaps, think, tick_at, wait_until,
};
use super::envelope::{Envelope, PAD, dealt, for_pair, frame_halos, holding, outline, swelling, wanted_pair};
use super::two_keys::{check_strip, glow, glowing, leaving, moving, text_halo};
use crate::model::{Dial, OPENING_TRIO, PAIR, Ratio, SECRET, SPINS, THRESHOLD, TRUSTEES, View, fitting_spins, tuples};
use crate::palette::{self, spin};

pub struct AnExperiment {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FIRST: &str = "first spin";
const SECOND: &str = "second spin";
const TRYING: &str = "trying";
const PENDING: &str = "?";

// ------------------------------------------------------------------ layout

/// The spin grid, upper left where the door was: its lower-left corner, a
/// cell's size, and the square drawn in each cell.
const GRID_X0: f64 = -5.9;
const GRID_Y0: f64 = -0.15;
const CELL: f64 = 0.25;
const CELL_IN: f64 = 0.21;
/// The empty cells' opacity, and the sweep bar's.
const CELL_O: f64 = 0.22;
const SWEEP_O: f64 = 0.35;
/// The spins' numbers under the columns and left of the rows, their size,
/// the axis titles, and the grid's count over it.
const AXIS_GAP: f64 = 0.2;
const AXIS_SIZE: f64 = 15.0;
const AXIS_TITLE_SIZE: f64 = 20.0;
const FIRST_DY: f64 = 0.53;
const SECOND_X: f64 = -6.2;
const SECOND_Y: f64 = 2.88;
const GRID_COUNT_Y: f64 = 3.35;
const GRID_COUNT_SIZE: f64 = 24.0;
/// A lit cell's secret, in the cell.
const CELL_LABEL_SIZE: f64 = 14.0;
/// How long the sweep takes over the eleven columns.
const SWEEP_RT: f64 = 2.0;
/// The tally under the strip's caption: a slot per secret, its number's
/// size, and its title under it.
const TALLY_Y: f64 = -0.6;
const SLOT_W: f64 = 0.4;
const SLOT_H: f64 = 0.36;
const TALLY_SIZE: f64 = 22.0;
const TALLY_TITLE_Y: f64 = -1.1;
const TALLY_TITLE_SIZE: f64 = 20.0;
/// The counter: an outline round a tick and its label, its word over it,
/// and how far it swells when its secret's cell lights.
const COUNTER_W: f64 = 0.47;
const COUNTER_BOTTOM: f64 = 0.32;
const COUNTER_TOP: f64 = 1.52;
const TRYING_Y: f64 = 1.75;
const TRYING_SIZE: f64 = 16.0;
const COUNTER_SWELL: f64 = 1.15;
/// The closing card: two lines under the tally.
const CLOSING_Y: [f64; 2] = [-1.65, -2.15];
const CLOSING_SIZE: f64 = 26.0;
/// The think ring at the lower right, under the tally.
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.6;
/// The quickest and slowest one secret's step may be, and the share of it
/// the counter's move takes before the cell lights.
const STEP_FAST: f64 = 0.4;
const STEP_SLOW: f64 = 0.8;
const MOVE_SHARE: f64 = 0.35;

/// Each secret's step when `steps` steps should take `room` seconds.
fn step_time(room: f64, steps: u32) -> f64 {
    (room / f64::from(steps.max(1))).clamp(STEP_FAST, STEP_SLOW)
}

/// The middle of the cell for first spin `a` (across) and second spin `b`
/// (up).
fn cell_at(a: u32, b: u32) -> Vec3 {
    v(
        GRID_X0 + CELL * (f64::from(a) + 0.5),
        GRID_Y0 + CELL * (f64::from(b) + 0.5),
    )
}

/// Secret `s`'s tally slot, under its tick.
fn slot_at(s: u32) -> Vec3 {
    v(tick_at(s)[0], TALLY_Y)
}

/// The grid's count: every pair of marks the two spins could land on.
fn grid_count(marks: u32, cells: usize) -> String {
    format!("{marks} × {marks} = {cells} spin pairs")
}

/// The tally's title: what each slot counts.
fn tally_title(held: [u32; 2]) -> String {
    format!("ways that deal {} and {}", held[0], held[1])
}

/// 05_4's exact version: the chance of what the pair holds, the same under
/// every secret, so it moves no belief about the secret.
fn chance_card(held: [u32; 2], chance: Ratio) -> String {
    format!(
        "with fair spins, P(shares {} and {} | s) = {chance} for every secret s, so their belief about s is unchanged",
        held[0], held[1]
    )
}

/// 05_5's two lines.
fn three_line(k: usize) -> String {
    format!("{k} shares → the secret")
}

fn two_line(k: usize, left: usize) -> String {
    format!("{k} shares → all {left} secrets still fit")
}

/// The experiment: for each secret, the spin pairs that, with it, deal the
/// pair exactly `points` (the model's brute force).
fn fitting(dial: Dial, points: &[(u32, u32)]) -> Vec<Vec<Vec<u32>>> {
    dial.all()
        .map(|s| fitting_spins(dial, points, s, SPINS.len()))
        .collect()
}

/// The one cell each secret lights, or an error unless every secret fits
/// exactly one spin pair and no two secrets the same one.
fn one_cell_each(found: &[Vec<Vec<u32>>]) -> Result<Vec<[u32; 2]>, String> {
    let mut cells = Vec::with_capacity(found.len());
    for (s, ways) in found.iter().enumerate() {
        match ways.as_slice() {
            [w] if w.len() == 2 => cells.push([w[0], w[1]]),
            _ => return Err(format!("secret {s} fits {} spin pairs, not one", ways.len())),
        }
    }
    let mut seen = cells.clone();
    seen.sort_unstable();
    seen.dedup();
    if seen.len() != cells.len() {
        return Err("two secrets fit the same spin pair".to_owned());
    }
    Ok(cells)
}

/// Under fair spins, the chance that a secret deals what the pair holds:
/// its fitting spin pairs over all of them.
fn chance(ways: usize, all: usize) -> Ratio {
    Ratio::new(ways as i64, all as i64)
}

/// The secret counter: an outline round one tick of the strip and its
/// label, with its word over it. It moves; the labels never change.
struct Counter {
    outline: Mob,
    word: Mob,
}

impl Counter {
    fn mid() -> f64 {
        (COUNTER_BOTTOM + COUNTER_TOP) / 2.0
    }

    fn build(kit: &Kit, stage: &mut Stage<'_>, s: u32) -> fmn::Result<Self> {
        let x = tick_at(s)[0];
        let outline = outline(
            stage,
            v(x, Self::mid()),
            COUNTER_W,
            COUNTER_TOP - COUNTER_BOTTOM,
            palette::secret(),
        )?;
        let word = kit.text(stage, TRYING, TRYING_SIZE, palette::secret(), v(x, TRYING_Y))?;
        Ok(Self { outline, word })
    }

    fn parts(&self) -> [Mob; 2] {
        [self.outline, self.word]
    }

    /// The counter moving to secret `s`, prepared, over `rt`.
    fn stepping(&self, stage: &mut Stage<'_>, s: u32, rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let x = tick_at(s)[0];
        Ok(vec![
            moving(stage, self.outline, v(x, Self::mid()), rt)?,
            moving(stage, self.word, v(x, TRYING_Y), rt)?,
        ])
    }

    /// Its outline swelling, a shape in the secret's gold, prepared.
    fn swelling(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Box<dyn Animation>> {
        swelling(stage, self.outline, COUNTER_SWELL, palette::secret(), rt)
    }
}

impl SceneConstruct for AnExperiment {
    fn name(&self) -> &str {
        "05_an_experiment"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let marks = dial.marks();
        let pair = dealt(dial, &PAIR);
        let held = [pair[0].1, pair[1].1];
        // the experiment, run before anything is drawn: the grid, each
        // secret's fitting cells, the chance under each secret
        let grid: Vec<Vec<u32>> = tuples(dial, SPINS.len()).collect();
        let found = fitting(dial, &pair);
        let cells = one_cell_each(&found).map_err(fail)?;
        let chances: Vec<Ratio> = found.iter().map(|w| chance(w.len(), grid.len())).collect();
        if chances.windows(2).any(|w| w[0] != w[1]) {
            return Err(fail(format!(
                "the pair's shares are likelier under some secrets: {chances:?}"
            )));
        }
        let fit: Vec<u32> = dial.all().filter(|&s| !found[s as usize].is_empty()).collect();

        // 05_1: the experiment: every secret, every way the spins could land
        let line = say(&kit, stage, "05_1")?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        let pair_cards = [
            card(&kit, stage, PAIR[0], &held[0].to_string(), palette::share())?,
            card(&kit, stage, PAIR[1], &held[1].to_string(), palette::share())?,
        ];
        let mut anims = recolorings(stage, &rail.tint(&PAIR, palette::pair()), 0.4)?;
        anims.extend(swaps(stage, &[], &pair_cards, 0.4)?);
        stage.play_prepared(anims)?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 0.6)?;
        let envelope = Envelope::build(&kit, stage)?;
        let with = swaps(stage, &[], envelope.parts(), 0.4)?;
        // still the goal, as chapter 04 left it: nothing is computed yet
        strip.show_with(&kit, stage, &View::Outsider, &wanted_pair(PAIR), 0.4, with)?;
        line.wait_for(stage, "Feed the dealer's method")?;
        let halo = Envelope::halo(stage, Palette::dim())?;
        glow(stage, &[halo], 0.5)?;
        line.wait_for(stage, "every secret from zero to ten")?;
        let counter = Counter::build(&kit, stage, 0)?;
        fade_in_all(stage, &counter.parts(), 0.4)?;
        line.wait_for(stage, "to ten")?;
        // along every secret and back: the shape moves, the labels stay
        let anims = counter.stepping(stage, marks - 1, 0.9)?;
        stage.play_prepared(anims)?;
        let anims = counter.stepping(stage, 0, 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "hundred and twenty-one ways")?;
        let mut columns: Vec<Vec<Mob>> = vec![Vec::new(); marks as usize];
        for t in &grid {
            let c = panel(stage, spin(), CELL_O, cell_at(t[0], t[1]), CELL_IN, CELL_IN)?;
            columns[t[0] as usize].push(c);
        }
        let mut column_mobs = Vec::with_capacity(columns.len());
        for c in &columns {
            column_mobs.push(bundle(stage, c)?);
        }
        let mut axes = Vec::with_capacity(2 * marks as usize + 3);
        for k in dial.all() {
            let label = k.to_string();
            axes.push(kit.text(
                stage,
                &label,
                AXIS_SIZE,
                spin(),
                v(cell_at(k, 0)[0], GRID_Y0 - AXIS_GAP),
            )?);
            axes.push(kit.text(
                stage,
                &label,
                AXIS_SIZE,
                spin(),
                v(GRID_X0 - AXIS_GAP, cell_at(0, k)[1]),
            )?);
        }
        let middle = GRID_X0 + CELL * f64::from(marks) / 2.0;
        let first = kit.text(
            stage,
            FIRST,
            AXIS_TITLE_SIZE,
            Palette::dim(),
            v(middle, GRID_Y0 - FIRST_DY),
        )?;
        let second = kit.text_left(stage, SECOND, AXIS_TITLE_SIZE, Palette::dim(), v(SECOND_X, SECOND_Y))?;
        let count = kit.text(
            stage,
            &grid_count(marks, grid.len()),
            GRID_COUNT_SIZE,
            spin(),
            v(middle, GRID_COUNT_Y),
        )?;
        axes.extend([first, second, count]);
        fade_in_all(stage, &axes, 0.5)?;
        fade_in_lagged(stage, &column_mobs, 1.2, 0.2)?;
        line.wait_for(stage, "its two spins could land")?;
        let titles = [text_halo(stage, first, spin())?, text_halo(stage, second, spin())?];
        glow(stage, &titles, 0.6)?;
        line.wait_end(stage)?;

        // 05_2: for each secret, count the ways that deal exactly 2 and 6;
        // the count waits until after the question
        let line = say(&kit, stage, "05_2")?;
        line.wait_for(stage, "For each secret")?;
        let a = counter.swelling(stage, 0.4)?;
        stage.play_prepared(vec![a])?;
        line.wait_for(stage, "count the ways of spinning")?;
        let mut slots = Vec::with_capacity(marks as usize);
        for s in dial.all() {
            slots.push(outline(stage, slot_at(s), SLOT_W, SLOT_H, Palette::dim())?);
        }
        let title = kit.text_left(
            stage,
            &tally_title(held),
            TALLY_TITLE_SIZE,
            Palette::dim(),
            v(STRIP_LEFT, TALLY_TITLE_Y),
        )?;
        let mut tally = slots.clone();
        tally.push(title);
        fade_in_all(stage, &tally, 0.4)?;
        // the method tries every cell for secret 0: a bar sweeps the columns
        let bar = panel(
            stage,
            spin(),
            SWEEP_O,
            v(cell_at(0, 0)[0], GRID_Y0 + CELL * f64::from(marks) / 2.0),
            CELL,
            CELL * f64::from(marks) + 0.1,
        )?;
        fade_in_all(stage, &[bar], 0.2)?;
        let last = v(cell_at(marks - 1, 0)[0], stage.get_center(bar)[1]);
        let across = bar
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(SWEEP_RT),
                rate_func: Some(fmn::core::rate::linear),
                ..AnimateArgs::default()
            })?
            .move_to(last, ORIGIN)?;
        stage.play(across)?;
        let anims = leaving(stage, &[bar], ORIGIN, 0.2)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "a two and a six")?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        let pending = kit.text(stage, PENDING, TALLY_SIZE, Palette::dim(), slot_at(0))?;
        let mut anims = glowing(stage, &halos, 0.5)?;
        anims.extend(swaps(stage, &[], &[pending], 0.4)?);
        stage.play_prepared(anims)?;
        line.wait_end(stage)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 05_3: no: each secret fits exactly one way, a different one each
        // time, so the pair's 2 and 6 fit every secret equally well
        let line = say(&kit, stage, "05_3")?;
        // the question's mark and the waiting "?" go before any count comes
        let anims = leaving(stage, &[q, pending], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "Every secret fits exactly one way")?;
        let each = step_time(line.at("So the pair's two and six")? - now(stage), marks);
        for (s, &[a, b]) in (0..marks).zip(&cells) {
            if s > 0 {
                let anims = counter.stepping(stage, s, MOVE_SHARE * each)?;
                stage.play_prepared(anims)?;
            }
            let at = cell_at(a, b);
            let cell = panel(stage, palette::secret(), 1.0, at, CELL_IN, CELL_IN)?;
            let label = kit.text(stage, &s.to_string(), CELL_LABEL_SIZE, hex(BACKGROUND), at)?;
            let ways = found[s as usize].len().to_string();
            let one = kit.text(stage, &ways, TALLY_SIZE, Palette::ink(), slot_at(s))?;
            let rt = (1.0 - MOVE_SHARE) * each;
            let mut anims = swaps(stage, &[], &[cell, label, one], rt)?;
            anims.push(counter.swelling(stage, rt)?);
            stage.play_prepared(anims)?;
        }
        line.wait_for(stage, "So the pair's two and six")?;
        // what the pair's shares leave, computed from them, is what the grid
        // found: every secret, so the caption is no longer only a goal
        let mut with = leaving(stage, &counter.parts(), ORIGIN, 0.6)?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        with.extend(glowing(stage, &halos, 0.6)?);
        strip.show_with(&kit, stage, &holding(dial, &PAIR), &for_pair(PAIR), 0.6, with)?;
        check_strip(&strip, &fit, "for the pair")?;
        line.wait_end(stage)?;
        // the grid stays a moment (a named hold)
        pause(stage, 1.0)?;

        // 05_4: their shares cannot say what to guess; every secret is as
        // likely as it was
        let line = say(&kit, stage, "05_4")?;
        line.wait_for(stage, "With fair spins")?;
        let exact = chance_card(held, chances[0]);
        let note = precisely(&kit, stage, &exact)?;
        fade_in_all(stage, &[note], 0.5)?;
        let read = read_by(stage, &exact);
        line.wait_for(stage, "every secret is exactly as likely")?;
        let right = strip_right(marks);
        let row = outline(
            stage,
            v((STRIP_LEFT + right) / 2.0, TALLY_Y),
            right - STRIP_LEFT + 2.0 * PAD,
            SLOT_H + 2.0 * PAD,
            palette::secret(),
        )?;
        glow(stage, &[row], 0.8)?;
        wait_until(stage, read)?;
        line.wait_end(stage)?;

        // 05_5: two shares cannot determine the secret, and three can; the
        // envelope stays sealed
        let line = say(&kit, stage, "05_5")?;
        let anims = leaving(stage, &[note], ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        let opened = holding(dial, &OPENING_TRIO).possible(dial);
        if opened != [SECRET] {
            return Err(fail(format!("three shares leave {opened:?}, not the secret alone")));
        }
        line.wait_for(stage, "Two shares can't determine the secret")?;
        let two = kit.text_left(
            stage,
            &two_line(PAIR.len(), strip.lit().len()),
            CLOSING_SIZE,
            palette::pair(),
            v(STRIP_LEFT, CLOSING_Y[1]),
        )?;
        fade_in_all(stage, &[two], 0.4)?;
        line.wait_for(stage, "and three can")?;
        let three = kit.text_left(
            stage,
            &three_line(THRESHOLD),
            CLOSING_SIZE,
            palette::trio(),
            v(STRIP_LEFT, CLOSING_Y[0]),
        )?;
        fade_in_all(stage, &[three], 0.4)?;
        line.wait_for(stage, "The rest of the story")?;
        let halo = Envelope::halo(stage, Palette::dim())?;
        glow(stage, &[halo], 0.6)?;
        line.wait_end(stage)?;
        pause(stage, 0.6)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

/// The scene's time now, in seconds.
fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{
        FRAME_H, FRAME_Y, STRIP_CAPTION_Y, STRIP_DX, STRIP_LABEL_DY, STRIP_TITLE_Y, STRIP_Y, THINK_R, TICK_H, TICK_W,
    };
    use super::super::envelope::{ENV_W, ENV_X};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, csv};

    /// The production narration: registering chapter 05 brings its lines
    /// into script.tsv (as root did for chapter 03).
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes: 0.0055 units a point a character wide (a
    /// quarter over the corpus's measured 0.0045) and 0.0065 a point tall.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Every phrase the scene waits for, or times its steps by, is in its
    /// line of the script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("05_1", "Feed the dealer's method"),
            ("05_1", "every secret from zero to ten"),
            ("05_1", "to ten"),
            ("05_1", "hundred and twenty-one ways"),
            ("05_1", "its two spins could land"),
            ("05_2", "For each secret"),
            ("05_2", "count the ways of spinning"),
            ("05_2", "a two and a six"),
            ("05_3", "Every secret fits exactly one way"),
            ("05_3", "So the pair's two and six"),
            ("05_4", "With fair spins"),
            ("05_4", "every secret is exactly as likely"),
            ("05_5", "Two shares can't determine the secret"),
            ("05_5", "and three can"),
            ("05_5", "The rest of the story"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // "to ten" is the range's end, said once
        assert_eq!(script.text("05_1").unwrap().matches("to ten").count(), 1);
    }

    /// The experiment is the model's and the facts': 121 spin pairs; for
    /// the pair's 2 and 6, exactly one fitting pair per secret, eleven in
    /// all, a different one each time, the true secret's being the
    /// example's spins; and each one, checked here by writing the share out
    /// (secret + first·x + second·x², wrapped), deals 2 at 4 and 6 at 5.
    #[test]
    fn the_experiment_is_the_models() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let pair = dealt(dial, &PAIR);
        assert_eq!(f.get("S-F1.pair_shares"), Some("2,6"));
        assert_eq!(pair, vec![(4, 2), (5, 6)]);
        let grid: Vec<Vec<u32>> = tuples(dial, SPINS.len()).collect();
        assert_eq!(f.get("S-F6.spin_pairs"), Some(grid.len().to_string().as_str()));
        assert_eq!(grid.len(), (MARKS * MARKS) as usize);
        let found = fitting(dial, &pair);
        let cells = one_cell_each(&found).unwrap();
        assert_eq!(cells.len(), MARKS as usize);
        assert_eq!(f.get("S-F7.fitting"), Some(cells.len().to_string().as_str()));
        assert_eq!(f.get("S-F7.ways_each"), Some("1"));
        let by_secret: Vec<String> = cells.iter().map(|c| format!("{}:{}", c[0], c[1])).collect();
        assert_eq!(f.get("S-F7.spins_by_secret"), Some(by_secret.join(",").as_str()));
        assert_eq!(
            cells[SECRET as usize], SPINS,
            "the true secret's cell is the dealer's spins"
        );
        for (s, &[a, b]) in (0..MARKS).zip(&cells) {
            for &(x, y) in &pair {
                let written = (s + a * x + b * x * x) % MARKS;
                assert_eq!(written, y, "secret {s}, spins {a} and {b}, trustee {x}");
            }
        }
        // the brute force agrees with every other spin pair failing
        let fits = grid
            .iter()
            .filter(|t| (0..MARKS).any(|s| pair.iter().all(|&(x, y)| (s + t[0] * x + t[1] * x * x) % MARKS == y)))
            .count();
        assert_eq!(fits, MARKS as usize, "each fitting pair fits one secret");
        // the strip's view of the pair and the grid's answer agree
        let fit: Vec<u32> = dial.all().filter(|&s| !found[s as usize].is_empty()).collect();
        assert_eq!(holding(dial, &PAIR).possible(dial), fit);
        assert_eq!(fit.len(), MARKS as usize);
    }

    /// Two secrets fitting one cell, or a secret fitting none or two, stops
    /// the scene.
    #[test]
    fn a_secret_not_fitting_once_is_an_error() {
        let one = |a: u32, b: u32| vec![vec![a, b]];
        assert!(one_cell_each(&[one(1, 4), one(0, 9)]).is_ok());
        assert!(one_cell_each(&[one(1, 4), one(1, 4)]).is_err());
        assert!(one_cell_each(&[one(1, 4), vec![]]).is_err());
        assert!(one_cell_each(&[vec![vec![1, 4], vec![2, 5]]]).is_err());
    }

    /// The chance is the likelihood, the same 1/121 under every secret; so
    /// for any prior over the secrets, the posterior is the prior (written
    /// cross-multiplied in whole numbers: prior(s) · P(seen | s) · Σ prior =
    /// prior(s) · Σ prior · P(seen | ·)), not only for an even one. The
    /// cards say so without assuming one.
    #[test]
    fn the_chance_moves_no_belief_whatever_the_prior() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let pair = dealt(dial, &PAIR);
        let found = fitting(dial, &pair);
        let all = tuples(dial, SPINS.len()).count();
        let chances: Vec<Ratio> = found.iter().map(|w| chance(w.len(), all)).collect();
        assert!(chances.windows(2).all(|w| w[0] == w[1]));
        assert_eq!(f.get("S-F7.chance_each"), Some(chances[0].to_string().as_str()));
        let ways: Vec<u64> = found.iter().map(|w| w.len() as u64).collect();
        let priors: [Vec<u64>; 4] = [
            vec![1; MARKS as usize],
            (1..=u64::from(MARKS)).collect(),
            (0..u64::from(MARKS)).map(|s| (7 * s) % 11 + 1).collect(),
            (0..u64::from(MARKS)).map(|s| if s == 3 { 1000 } else { 1 }).collect(),
        ];
        for prior in &priors {
            let total: u64 = prior.iter().sum();
            let evidence: u64 = prior.iter().zip(&ways).map(|(p, w)| p * w).sum();
            for (s, (&p, &w)) in prior.iter().zip(&ways).enumerate() {
                // posterior p·w / evidence equals prior p / total
                assert_eq!(p * w * total, p * evidence, "secret {s}, prior {prior:?}");
            }
        }
        assert_eq!(
            chance_card([2, 6], chances[0]),
            "with fair spins, P(shares 2 and 6 | s) = 1/121 for every secret s, so their belief about s is unchanged"
        );
        assert_eq!(three_line(THRESHOLD), "3 shares → the secret");
        assert_eq!(
            two_line(PAIR.len(), MARKS as usize),
            "2 shares → all 11 secrets still fit"
        );
        assert_eq!(grid_count(MARKS, all), "11 × 11 = 121 spin pairs");
        assert_eq!(tally_title([2, 6]), "ways that deal 2 and 6");
        assert_eq!(f.get("S-F1.threshold"), Some(THRESHOLD.to_string().as_str()));
        assert_eq!(f.get("S-F1.pair"), Some(csv(&PAIR).as_str()));
    }

    /// The chance card stays two lines at every wrap the lower-third can
    /// narrow to.
    #[test]
    fn the_card_is_two_lines() {
        let text = chance_card([2, 6], Ratio::new(1, 121));
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// Eleven steps fit the room the words leave, never quicker than
    /// STEP_FAST or slower than STEP_SLOW.
    #[test]
    fn the_steps_fit_their_words() {
        let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(near(step_time(6.6, 11), 0.6));
        assert!(near(step_time(1.0, 11), STEP_FAST));
        assert!(near(step_time(-1.0, 11), STEP_FAST));
        assert!(near(step_time(20.0, 11), STEP_SLOW));
        assert!(near(step_time(1.0, 0), STEP_SLOW));
    }

    /// The grid, its numbers and titles sit upper left, inside the frame,
    /// over the card frames and left of the strip; a lit cell's secret fits
    /// its cell; the counter rings one tick and its label, clear of the
    /// next tick, the caption and the title even as it swells; the tally's
    /// slots sit under the caption, apart, over the think ring; the closing
    /// card fits under the tally, over the band.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const GRID_RIGHT: f64 = GRID_X0 + CELL * MARKS as f64;
        const GRID_TOP: f64 = GRID_Y0 + CELL * MARKS as f64;
        const FRAMES_TOP: f64 = FRAME_Y + FRAME_H / 2.0;
        const COUNTER_H: f64 = COUNTER_TOP - COUNTER_BOTTOM;
        const {
            assert!(CELL_IN < CELL - 0.02, "the cells apart");
            assert!(
                GRID_RIGHT < STRIP_LEFT - 2.0,
                "the grid left of the strip, room between"
            );
            assert!(
                GRID_Y0 - FIRST_DY - 0.0065 * AXIS_TITLE_SIZE / 2.0 > FRAMES_TOP + 0.3,
                "the first title over the frames"
            );
            assert!(
                GRID_Y0 - AXIS_GAP - 0.0065 * AXIS_SIZE / 2.0
                    > GRID_Y0 - FIRST_DY + 0.0065 * AXIS_TITLE_SIZE / 2.0 + 0.05,
                "the column numbers over the first title"
            );
            assert!(
                SECOND_Y - 0.0065 * AXIS_TITLE_SIZE / 2.0 > GRID_TOP + 0.1,
                "the second title over the grid"
            );
            assert!(
                GRID_COUNT_Y - 0.0065 * GRID_COUNT_SIZE / 2.0 > SECOND_Y + 0.0065 * AXIS_TITLE_SIZE / 2.0 + 0.1,
                "the count over the second title"
            );
            assert!(
                GRID_COUNT_Y + 0.0065 * GRID_COUNT_SIZE / 2.0 < FRAME_Y_RADIUS - 0.25,
                "the count inside the top edge"
            );
            assert!(
                SECOND_X > -FRAME_X_RADIUS + 0.25,
                "the second title inside the left edge"
            );
            assert!(
                COUNTER_TOP + 0.03 < TRYING_Y - 0.0065 * TRYING_SIZE / 2.0,
                "the word over the counter"
            );
            assert!(STRIP_Y + TICK_H / 2.0 + 0.03 < COUNTER_TOP, "the counter over the tick");
            assert!(
                STRIP_Y - STRIP_LABEL_DY - 0.0065 * 20.0 / 2.0 > COUNTER_BOTTOM + 0.03,
                "the counter under the label"
            );
            assert!(
                TRYING_Y + 0.0065 * TRYING_SIZE / 2.0 < STRIP_TITLE_Y - 0.0065 * 40.0 / 2.0 - 0.1,
                "the word under the title and count"
            );
            assert!(
                (COUNTER_TOP + COUNTER_BOTTOM) / 2.0 - COUNTER_SWELL * COUNTER_H / 2.0
                    > STRIP_CAPTION_Y + 0.0065 * 22.0 / 2.0 + 0.05,
                "the swollen counter over the caption"
            );
            assert!(
                (COUNTER_TOP + COUNTER_BOTTOM) / 2.0 + COUNTER_SWELL * COUNTER_H / 2.0
                    < TRYING_Y - 0.0065 * TRYING_SIZE / 2.0,
                "the swollen counter under its word"
            );
            assert!(
                COUNTER_SWELL * COUNTER_W / 2.0 < STRIP_DX - TICK_W / 2.0 - 0.05,
                "the swollen counter off the next tick"
            );
            assert!(SLOT_W < STRIP_DX - 0.1, "the slots apart");
            assert!(
                TALLY_Y + SLOT_H / 2.0 < STRIP_CAPTION_Y - 0.0065 * 22.0 / 2.0 - 0.15,
                "the slots under the caption"
            );
            assert!(
                TALLY_TITLE_Y + 0.0065 * TALLY_TITLE_SIZE / 2.0 < TALLY_Y - SLOT_H / 2.0 - 0.1,
                "the title under the slots"
            );
            assert!(
                THINK_Y + THINK_R < TALLY_Y - SLOT_H / 2.0 - 0.15,
                "the ring under the slots"
            );
            assert!(THINK_Y - THINK_R - 0.45 > BAND_TOP, "the think note over the band");
            assert!(
                CLOSING_Y[0] + 0.0065 * CLOSING_SIZE / 2.0 < TALLY_TITLE_Y - 0.0065 * TALLY_TITLE_SIZE / 2.0 - 0.15,
                "the closing card under the tally"
            );
            assert!(
                CLOSING_Y[0] - CLOSING_Y[1] > 0.0065 * CLOSING_SIZE + 0.15,
                "its two lines apart"
            );
            assert!(
                CLOSING_Y[1] - 0.0065 * CLOSING_SIZE / 2.0 > BAND_TOP + 0.2,
                "the closing card over the band"
            );
        }
        // the numbers along the axes: inside their cells, inside the frame
        let ten = text_w("10", AXIS_SIZE);
        assert!(ten < CELL - 0.05);
        assert!(GRID_X0 - AXIS_GAP - ten / 2.0 > -FRAME_X_RADIUS + 0.25);
        assert!(GRID_X0 - AXIS_GAP + ten / 2.0 < GRID_X0 - 0.03);
        assert!(text_h(AXIS_SIZE) < CELL - 0.05);
        // a lit cell's secret fits its square
        assert!(text_w("10", CELL_LABEL_SIZE) < CELL_IN - 0.03);
        assert!(text_h(CELL_LABEL_SIZE) < CELL_IN - 0.06);
        // the titles and the count inside the frame, left of the envelope
        let middle = GRID_X0 + CELL * f64::from(MARKS) / 2.0;
        let count_w = text_w(&grid_count(MARKS, (MARKS * MARKS) as usize), GRID_COUNT_SIZE);
        assert!(middle - count_w / 2.0 > -FRAME_X_RADIUS + 0.25);
        assert!(middle + count_w / 2.0 < ENV_X - ENV_W / 2.0 - PAD - 0.3);
        assert!(SECOND_X + text_w(SECOND, AXIS_TITLE_SIZE) < GRID_X0 + CELL * f64::from(MARKS));
        let first_w = text_w(FIRST, AXIS_TITLE_SIZE);
        assert!(middle - first_w / 2.0 > GRID_X0 - 0.3);
        // the word at the last tick inside the right edge
        let last = tick_at(MARKS - 1)[0];
        assert!(last + text_w(TRYING, TRYING_SIZE) / 2.0 < FRAME_X_RADIUS - 0.25);
        // the tally's title and the closing card: left of the ring, inside
        let title_w = text_w(&tally_title([2, 6]), TALLY_TITLE_SIZE);
        assert!(STRIP_LEFT + title_w < THINK_X - THINK_R - 0.3);
        for text in [three_line(THRESHOLD), two_line(2, 11)] {
            assert!(
                STRIP_LEFT + text_w(&text, CLOSING_SIZE) < FRAME_X_RADIUS - 0.25,
                "{text}"
            );
        }
        // the caption it ends on, as chapter 04's
        for caption in [wanted_pair(PAIR), for_pair(PAIR)] {
            assert!(STRIP_LEFT + text_w(&caption, 22.0) < FRAME_X_RADIUS - 0.25, "{caption}");
        }
        assert!(THINK_X + THINK_R < strip_right(MARKS) + 0.3);
        const {
            assert!(
                THINK_X + THINK_R < FRAME_X_RADIUS - 0.25,
                "the ring inside the right edge"
            )
        };
    }
}
