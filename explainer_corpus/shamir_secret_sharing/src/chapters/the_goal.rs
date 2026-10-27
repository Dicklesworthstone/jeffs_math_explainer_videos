//! Chapter 04, the goal: back to the five trustees and a threshold of
//! three; from the secret and two fair spins, the dealer hands out the
//! shares 8, 3, 1, 2 and 6 by a method kept sealed until chapters 07 to 09
//! (04_1); trustees 1, 2 and 3 work out the secret, 5, from their 8, 3 and
//! 1, and the door opens, how they do it left for later (04_2); trustees 4
//! and 5 hold 2 and 6 and, like anyone, can only guess, right one time in
//! eleven (04_3); the demand is not that two never get lucky, but that two
//! shares tell them nothing (04_4); and the question whether that is
//! possible at all (04_5).
//!
//! The shares are the model's (`envelope::dealt`), never typed in, and the
//! envelope stands for the method without opening. The trio's strip is
//! computed from their three shares. The pair's strip is their goal,
//! captioned "wanted" as in chapter 01: what two shares do leave is chapter
//! 05's experiment, after its question, so it is not shown here.

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    DIAL_Y, Rail, STRIP_LEFT, STRIP_Y, Strip, TICK_H, Vault, card, fade_in_all, fade_in_lagged, fail, frame_at, pause,
    read_by, recolor, recolor_all, recolorings, say, strip_right, swaps, think, wait_until,
};
use super::envelope::{
    ENV_Y, Envelope, PAD, Spinner, alarm, dealt, door_glow, fills, frame_halos, going_in, holding, outline, show_spins,
    wanted_pair,
};
use super::two_keys::{check_strip, glow, glowing, leaving, moving, swell_all, text_halo};
use crate::model::{Dial, MARKS, OPENING_TRIO, PAIR, SECRET, SPINS, THRESHOLD, TRUSTEES, View};
use crate::palette;

pub struct TheGoal {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FOR_OUTSIDE: &str = "for: anyone outside";
const UNKNOWN: &str = "?";
const ODD: &str = "odd?";
const DEMAND: &str = "the demand:";

/// The wrong guess of 04_3 and the mark the pair's doubt names in 04_4: any
/// mark but the secret (chapter 01's wrong mark).
const WRONG_MARK: u32 = 3;
/// Whole turns each spin makes before it stops.
const TURNS: [u32; 2] = [2, 3];

// ------------------------------------------------------------------ layout

/// The secret and the two spins, left of the envelope, before they go in.
const SECRET_IN_X: f64 = 0.15;
const SECRET_IN_SIZE: f64 = 36.0;
const SPIN_IN_X: [f64; 2] = [0.95, 1.65];
/// How long the secret and the spins take to go in, and the quickest and
/// slowest the spins may be, so they finish before the dealing words.
const INTO_RT: f64 = 0.6;
const SPIN_FAST: f64 = 0.7;
const SPIN_SLOW: f64 = 1.4;
/// How long the five cards take to come in, one after another.
const DEAL_RT: f64 = 2.0;
/// The box beside the dial where the trio's cards go (04_2), the step
/// between the cards in it, and the "?" over it.
const BOX_X: f64 = -0.8;
const BOX_W: f64 = 1.9;
const BOX_H: f64 = 0.7;
const BOX_DX: f64 = 0.55;
const BOX_Q_DY: f64 = 0.7;
const BOX_Q_SIZE: f64 = 36.0;
/// The gamble card under the box (04_3).
const GUESS_Y: f64 = -0.35;
const GUESS_SIZE: f64 = 28.0;
/// The doubts that rise over the pair (04_4): where each ends up, how far
/// it rises, and its size.
const DOUBT_AT: [[f64; 2]; 2] = [[-1.45, -0.2], [-0.35, 0.5]];
const DOUBT_RISE: f64 = 0.35;
const DOUBT_SIZE: f64 = 24.0;
/// The threshold card and the think ring, as chapter 01's.
const THRESHOLD_X: f64 = 2.3;
const THRESHOLD_Y: f64 = -0.9;
const THRESHOLD_SIZE: f64 = 30.0;
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// The trio's discs, dimmed for the question (still teal).
const FAINT: f64 = 0.45;

/// The threshold card (as chapter 01's).
fn threshold_card(k: usize) -> String {
    format!("threshold {k}")
}

/// The strip's caption for the trio who opened the door.
fn for_trio(trio: [u32; 3]) -> String {
    format!("for: trustees {}, {} and {}", trio[0], trio[1], trio[2])
}

/// 04_3's gamble card: one mark of the dial's, picked blind.
fn guess_card(marks: u32) -> String {
    format!("guess: 1 in {marks}")
}

/// A doubt that would rule a mark out.
fn doubt(k: u32) -> String {
    format!("not {k}?")
}

/// 04_4's card: what two shares must not do.
fn demand_card(marks: u32) -> String {
    format!("two shares give no extra clue; a random guess still wins 1 in {marks}")
}

/// Where the trio's `i`th card sits in the box.
fn box_slot(i: usize) -> Vec3 {
    v(BOX_X + BOX_DX * (i as f64 - 1.0), DIAL_Y)
}

/// Trustee `t`'s card, of the five dealt.
fn card_of(cards: &[Mob], t: u32) -> Mob {
    cards[(t - TRUSTEES[0]) as usize]
}

/// An outline round the strip's ticks, in `color`. Detached.
fn strip_halo(stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
    let right = strip_right(MARKS);
    outline(
        stage,
        v((STRIP_LEFT + right) / 2.0, STRIP_Y),
        right - STRIP_LEFT + 2.0 * PAD,
        TICK_H + 2.0 * PAD,
        color,
    )
}

/// A doubt: its words at `at`, in the pair's color, with an outline of its
/// own. Detached.
fn bubble(kit: &Kit, stage: &mut Stage<'_>, text: &str, at: [f64; 2]) -> fmn::Result<[Mob; 2]> {
    let words = kit.text(stage, text, DOUBT_SIZE, palette::pair(), v(at[0], at[1]))?;
    let round = text_halo(stage, words, palette::pair())?;
    Ok([words, round])
}

/// Fade-ins, prepared, of `mobs`, each rising `DOUBT_RISE` into place, over
/// `rt`.
fn rising(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(mobs.len());
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, v(0.0, DOUBT_RISE), 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    Ok(anims)
}

impl SceneConstruct for TheGoal {
    fn name(&self) -> &str {
        "04_the_goal"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let shares = dealt(dial, &TRUSTEES);
        // the trio's three shares single out the secret, as the model says
        let trio_view = holding(dial, &OPENING_TRIO);
        let opened = trio_view.possible(dial);
        if opened != [SECRET] {
            return Err(fail(format!(
                "trustees {OPENING_TRIO:?}'s shares leave {opened:?}, not the secret alone"
            )));
        }

        // 04_1: back to the five; the sealed method deals five shares
        let line = say(&kit, stage, "04_1")?;
        let vault = Vault::build(&kit, stage, dial)?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        vault.show(stage, 0.5)?;
        rail.reveal(stage, 0.6)?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 0.6)?;
        line.wait_for(stage, "a threshold of three")?;
        let threshold = kit.text(
            stage,
            &threshold_card(THRESHOLD),
            THRESHOLD_SIZE,
            palette::trio(),
            v(THRESHOLD_X, THRESHOLD_Y),
        )?;
        let with = swaps(stage, &[], &[threshold], 0.4)?;
        strip.show_with(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.4, with)?;
        line.wait_for(stage, "Using the secret")?;
        let envelope = Envelope::build(&kit, stage)?;
        let secret = kit.text(
            stage,
            &SECRET.to_string(),
            SECRET_IN_SIZE,
            palette::secret(),
            v(SECRET_IN_X, ENV_Y),
        )?;
        let spins = [
            Spinner::build(stage, dial, v(SPIN_IN_X[0], ENV_Y))?,
            Spinner::build(stage, dial, v(SPIN_IN_X[1], ENV_Y))?,
        ];
        let mut arriving = envelope.parts().to_vec();
        arriving.push(secret);
        fade_in_all(stage, &arriving, 0.5)?;
        show_spins(stage, &spins, 0.3)?;
        line.wait_for(stage, "two fair spins")?;
        // the spins land on the example's, 7 and 7, unnumbered; they finish,
        // and go in with the secret, before the dealing words
        let room = line.at("hands out five shares")? - now(stage) - INTO_RT;
        let spin_rt = room.clamp(SPIN_FAST, SPIN_SLOW);
        let mut anims = Vec::with_capacity(spins.len());
        for ((s, &k), &turns) in spins.iter().zip(&SPINS).zip(&TURNS) {
            anims.push(s.spinning(stage, k, turns, spin_rt)?);
        }
        stage.play_prepared(anims)?;
        let mut going = vec![secret];
        for s in &spins {
            going.extend(s.parts());
        }
        let anims = going_in(stage, &going, INTO_RT)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "hands out five shares")?;
        let halo = Envelope::halo(stage, Palette::dim())?;
        glow(stage, &[halo], 0.5)?;
        line.wait_for(stage, "eight, three, one, two and six")?;
        let mut cards = Vec::with_capacity(shares.len());
        for &(t, y) in &shares {
            cards.push(card(&kit, stage, t, &y.to_string(), palette::share())?);
        }
        fade_in_lagged(stage, &cards, DEAL_RT, 0.35)?;
        line.wait_end(stage)?;

        // 04_2: the trio brings 8, 3 and 1 and opens the door; how, later
        let line = say(&kit, stage, "04_2")?;
        recolor_all(stage, &rail.tint(&OPENING_TRIO, palette::trio()), 0.4)?;
        line.wait_for(stage, "bring their eight, three and one")?;
        let tray = outline(stage, v(BOX_X, DIAL_Y), BOX_W, BOX_H, palette::trio())?;
        let unknown = kit.text(stage, UNKNOWN, BOX_Q_SIZE, Palette::dim(), v(BOX_X, DIAL_Y + BOX_Q_DY))?;
        let mut anims = swaps(stage, &[], &[tray, unknown], 0.8)?;
        for (i, &t) in OPENING_TRIO.iter().enumerate() {
            anims.push(moving(stage, card_of(&cards, t), box_slot(i), 0.8)?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "work out the secret, five")?;
        // the needle finds the secret as the strip shows what the three
        // shares leave, in one move
        let turn = stage.prepare(tween(vault.needle, f64::from(SECRET), 0.8)?)?;
        strip.show_with(&kit, stage, &trio_view, &for_trio(OPENING_TRIO), 0.8, vec![turn])?;
        check_strip(&strip, &opened, "for the trio")?;
        line.wait_for(stage, "the door opens")?;
        recolor_all(stage, &[(vault.door, palette::open())], 0.3)?;
        door_glow(stage, &vault, palette::open(), 0.6)?;
        line.wait_for(stage, "How they do it comes later")?;
        let halo = Envelope::halo(stage, Palette::dim())?;
        glow(stage, &[halo], 0.6)?;
        line.wait_end(stage)?;
        // the open door stays a moment (a named hold)
        pause(stage, 1.0)?;
        // shut again, in one move: the trio's cards go home, the needle to
        // the top, the strip back to an outsider's
        let mut with = leaving(stage, &[tray, unknown], ORIGIN, 0.6)?;
        for &t in &OPENING_TRIO {
            with.push(moving(stage, card_of(&cards, t), frame_at(t), 0.6)?);
        }
        with.extend(recolorings(stage, &rail.tint(&OPENING_TRIO, palette::share()), 0.6)?);
        with.push(stage.prepare(recolor(vault.door, palette::ghost(), 0.6)?)?);
        with.push(stage.prepare(tween(vault.needle, 0.0, 0.6)?)?);
        strip.show_with(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.6, with)?;

        // 04_3: the pair holds 2 and 6 and, like anyone, can only guess
        let line = say(&kit, stage, "04_3")?;
        // their strip is the goal, as in chapter 01, not yet a result
        let with = recolorings(stage, &rail.tint(&PAIR, palette::pair()), 0.5)?;
        strip.show_with(&kit, stage, &View::Outsider, &wanted_pair(PAIR), 0.5, with)?;
        line.wait_for(stage, "hold two and six")?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        glow(stage, &halos, 0.5)?;
        line.wait_for(stage, "They'd be right one time in eleven")?;
        let guess = kit.text(
            stage,
            &guess_card(dial.marks()),
            GUESS_SIZE,
            palette::pair(),
            v(BOX_X, GUESS_Y),
        )?;
        fade_in_all(stage, &[guess], 0.4)?;
        line.wait_for(stage, "the other ten times")?;
        vault.turn(stage, WRONG_MARK, 0.5)?;
        alarm(stage, &vault)?;
        vault.turn(stage, 0, 0.4)?;
        line.wait_end(stage)?;

        // 04_4: the demand: not that two never get lucky, but that their
        // two shares tell them nothing
        let line = say(&kit, stage, "04_4")?;
        let anims = leaving(stage, &[guess], ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "It's that their two shares tell them nothing")?;
        let text = demand_card(dial.marks());
        let demand = lower_third(&kit, stage, DEMAND, &text)?;
        let mut anims = swaps(stage, &[], &[demand], 0.5)?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        anims.extend(glowing(stage, &halos, 0.5)?);
        stage.play_prepared(anims)?;
        let read = read_by(stage, &text);
        line.wait_for(stage, "not one mark ruled out")?;
        let full = strip_halo(stage, palette::secret())?;
        let not_that = bubble(&kit, stage, &doubt(WRONG_MARK), DOUBT_AT[0])?;
        let mut anims = glowing(stage, &[full], 0.8)?;
        anims.extend(rising(stage, &not_that, 0.5)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "not even a hint that the secret is odd")?;
        let odd = bubble(&kit, stage, ODD, DOUBT_AT[1])?;
        let anims = rising(stage, &odd, 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_end(stage)?;
        // the doubts drift off unanswered; the strip stays full
        let mut doubts = not_that.to_vec();
        doubts.extend(odd);
        let anims = leaving(stage, &doubts, v(0.0, DOUBT_RISE), 0.5)?;
        stage.play_prepared(anims)?;
        wait_until(stage, read)?;

        // 04_5: is that possible at all?
        let line = say(&kit, stage, "04_5")?;
        let trio_discs: Vec<Mob> = OPENING_TRIO.iter().map(|&t| rail.disc(t)).collect();
        let faint: Vec<(Mob, Srgb, f64)> = trio_discs.iter().map(|&d| (d, palette::trio(), FAINT)).collect();
        let mut anims = leaving(stage, &[demand], ORIGIN, 0.4)?;
        anims.extend(fills(stage, &faint, 0.4)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "Could two shares be completely useless")?;
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        glow(stage, &halos, 0.5)?;
        line.wait_for(stage, "when three open the vault")?;
        swell_all(stage, &trio_discs, palette::trio(), 0.5)?;
        line.wait_end(stage)?;
        // the question stays open: chapter 05's experiment answers it
        think(&kit, stage, v(THINK_X, THINK_Y))?;
        pause(stage, 0.5)?;

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

    use super::super::draw::{DOOR_H, DOOR_PULSE, DOOR_W, DOOR_X, DOOR_Y, FRAME_H, FRAME_Y, STRIP_CAPTION_Y, THINK_R};
    use super::super::envelope::{ENV_W, ENV_X, SPINNER_R};
    use super::super::two_keys::TEXT_PAD;
    use super::*;
    use crate::facts;
    use crate::model::{Ratio, csv};

    /// The production narration: registering chapter 04 brings its lines
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

    /// Every phrase the scene waits for, or times its spins by, is in its
    /// line of the script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("04_1", "a threshold of three"),
            ("04_1", "Using the secret"),
            ("04_1", "two fair spins"),
            ("04_1", "hands out five shares"),
            ("04_1", "eight, three, one, two and six"),
            ("04_2", "bring their eight, three and one"),
            ("04_2", "work out the secret, five"),
            ("04_2", "the door opens"),
            ("04_2", "How they do it comes later"),
            ("04_3", "hold two and six"),
            ("04_3", "They'd be right one time in eleven"),
            ("04_3", "the other ten times"),
            ("04_4", "It's that their two shares tell them nothing"),
            ("04_4", "not one mark ruled out"),
            ("04_4", "not even a hint that the secret is odd"),
            ("04_5", "Could two shares be completely useless"),
            ("04_5", "when three open the vault"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the scene draws is the facts': the five shares the method deals,
    /// the trio and its three shares leaving the secret 5 alone, the pair
    /// and its 2 and 6, a guess right 1 time in 11, two spins, and the
    /// storyboard's words for the captions and the demand.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let values = |xs: &[u32]| csv(&dealt(dial, xs).iter().map(|&(_, y)| y).collect::<Vec<_>>());
        assert_eq!(f.get("S-F6.shares"), Some(values(&TRUSTEES).as_str()));
        assert_eq!(f.get("S-F1.opening_trio"), Some(csv(&OPENING_TRIO).as_str()));
        assert_eq!(f.get("S-F1.opening_shares"), Some(values(&OPENING_TRIO).as_str()));
        assert_eq!(f.get("S-F1.pair"), Some(csv(&PAIR).as_str()));
        assert_eq!(f.get("S-F1.pair_shares"), Some(values(&PAIR).as_str()));
        assert_eq!(holding(dial, &OPENING_TRIO).possible(dial), vec![SECRET]);
        assert_eq!(f.get("S-F1.secret"), Some(SECRET.to_string().as_str()));
        assert_eq!(f.get("S-F1.threshold"), Some(THRESHOLD.to_string().as_str()));
        assert_eq!(
            f.get("S-F12.guess_chance"),
            Some(Ratio::new(1, i64::from(dial.marks())).to_string().as_str())
        );
        // "two fair spins", "two shares": one spin fewer than the threshold
        assert_eq!(SPINS.len(), 2);
        assert_eq!(THRESHOLD - 1, 2);
        assert_eq!(PAIR.len(), THRESHOLD - 1);
        assert_eq!(f.get("S-F6.spins"), Some(csv(&SPINS).as_str()));
        assert_eq!(TURNS.len(), SPINS.len());
        assert_ne!(WRONG_MARK, SECRET, "the guess is a wrong one");
        assert!(WRONG_MARK < dial.marks());
        assert_eq!(threshold_card(THRESHOLD), "threshold 3");
        assert_eq!(for_trio(OPENING_TRIO), "for: trustees 1, 2 and 3");
        assert_eq!(guess_card(dial.marks()), "guess: 1 in 11");
        assert_eq!(doubt(WRONG_MARK), "not 3?");
        assert_eq!(
            demand_card(dial.marks()),
            "two shares give no extra clue; a random guess still wins 1 in 11"
        );
    }

    /// The demand card stays two lines at every wrap the lower-third can
    /// narrow to.
    #[test]
    fn the_card_is_two_lines() {
        let text = demand_card(MARKS);
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// The secret and the spins wait between the door and the envelope; the
    /// trio's box sits beside the dial, between the pulsing door and the
    /// strip, its "?" under the inputs; the gamble card and the doubts keep
    /// to the middle column, over the card frames; the captions, the
    /// threshold card and the think ring keep apart, as chapter 01's.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const DOOR_RIGHT: f64 = DOOR_X + DOOR_PULSE * DOOR_W / 2.0;
        const DOOR_BOTTOM: f64 = DOOR_Y - DOOR_PULSE * DOOR_H / 2.0;
        const FRAMES_TOP: f64 = FRAME_Y + FRAME_H / 2.0;
        const {
            assert!(SECRET_IN_X > DOOR_RIGHT + 0.5, "the secret right of the door");
            assert!(
                SPIN_IN_X[0] - SPINNER_R > SECRET_IN_X + 0.3,
                "the spins right of the secret"
            );
            assert!(SPIN_IN_X[1] - SPIN_IN_X[0] > 2.0 * SPINNER_R + 0.1, "the spins apart");
            assert!(
                SPIN_IN_X[1] + SPINNER_R < ENV_X - ENV_W / 2.0 - PAD - 0.1,
                "the spins left of the envelope"
            );
            assert!(
                ENV_Y + SPINNER_R < FRAME_Y_RADIUS - 0.2,
                "the spins inside the top edge"
            );
            assert!(
                BOX_X - BOX_W / 2.0 > DOOR_RIGHT + 0.3,
                "the box right of the pulsing door"
            );
            assert!(BOX_X + BOX_W / 2.0 < STRIP_LEFT - 0.3, "the box left of the strip");
            assert!(BOX_DX * 2.0 + 0.4 < BOX_W, "the three cards inside the box");
            assert!(
                DIAL_Y + BOX_Q_DY + 0.0065 * BOX_Q_SIZE / 2.0 < ENV_Y - SPINNER_R - 0.3,
                "the box's ? under the inputs"
            );
            assert!(
                GUESS_Y + 0.0065 * GUESS_SIZE / 2.0 < DIAL_Y - BOX_H / 2.0 - 0.3,
                "the guess under the box's place"
            );
            assert!(
                GUESS_Y - 0.0065 * GUESS_SIZE / 2.0 > FRAMES_TOP + 0.3,
                "the guess over the frames"
            );
            assert!(
                THRESHOLD_Y + 0.25 < STRIP_CAPTION_Y - 0.2,
                "the threshold card under the caption"
            );
            assert!(THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2, "the ring under the caption");
            assert!(THINK_Y - THINK_R - 0.45 > BAND_TOP, "the think note over the band");
            assert!(DOOR_BOTTOM > FRAMES_TOP, "draw.rs's door over the frames");
        }
        let guess_w = text_w(&guess_card(MARKS), GUESS_SIZE);
        assert!(BOX_X - guess_w / 2.0 > DOOR_RIGHT + 0.2 && BOX_X + guess_w / 2.0 < STRIP_LEFT - 0.2);
        // the doubts, outlined: right of the pulsing door, left of the
        // strip, over the frames, each rising from clear of the frames, and
        // apart from each other
        let boxes: Vec<[f64; 4]> = [doubt(WRONG_MARK), ODD.to_owned()]
            .iter()
            .zip(DOUBT_AT)
            .map(|(t, at)| {
                let (hw, hh) = (
                    text_w(t, DOUBT_SIZE) / 2.0 + TEXT_PAD,
                    text_h(DOUBT_SIZE) / 2.0 + TEXT_PAD,
                );
                [at[0] - hw, at[0] + hw, at[1] - hh, at[1] + hh]
            })
            .collect();
        for b in &boxes {
            assert!(b[0] > DOOR_RIGHT + 0.2 && b[1] < STRIP_LEFT - 0.2, "{b:?}");
            assert!(b[2] - DOUBT_RISE > FRAMES_TOP + 0.2, "{b:?}");
            assert!(b[3] < DIAL_Y + BOX_H, "{b:?}");
        }
        assert!(
            boxes[0][3] < boxes[1][2] || boxes[0][1] < boxes[1][0],
            "the doubts apart"
        );
        // the captions, as chapter 01's, end left of the think ring
        for caption in [FOR_OUTSIDE.to_owned(), for_trio(OPENING_TRIO), wanted_pair(PAIR)] {
            let right = STRIP_LEFT + text_w(&caption, 22.0);
            assert!(right < FRAME_X_RADIUS - 0.25, "{caption}");
            assert!(right < THINK_X - THINK_R - 0.2, "{caption}");
        }
        let card_w = text_w(&threshold_card(THRESHOLD), THRESHOLD_SIZE);
        assert!(THRESHOLD_X - card_w / 2.0 > STRIP_LEFT - 0.6);
        assert!(THRESHOLD_X + card_w / 2.0 < THINK_X - THINK_R - 0.3);
        assert!(THINK_X + THINK_R < strip_right(MARKS) + 0.3);
        // the trio's cards fit their box, apart
        assert!(box_slot(0)[0] - 0.3 > BOX_X - BOX_W / 2.0 && box_slot(2)[0] + 0.3 < BOX_X + BOX_W / 2.0);
    }
}
