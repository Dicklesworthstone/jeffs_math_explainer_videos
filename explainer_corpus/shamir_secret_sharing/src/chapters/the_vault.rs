//! Chapter 01, the vault: five trustees and one secret mark on an
//! eleven-mark dial, a wrong mark setting off the alarm (01_1); the strip of
//! possible secrets, all eleven for an outsider (01_2); one keeper, broken by
//! one lost note or, the other way, one bribe, the keeper's card back for the
//! second (01_3); copies, each of which opens the vault alone (01_4); the
//! goal in between, any three and no two, and the word threshold (01_5); what
//! opening means from here on, two trustees away while the other three still
//! work it out, and the prediction (01_6).
//!
//! The strip's ticks and count are `View::possible` of the model, and the
//! caption says whose view it is. The "wanted" views are a goal, not yet a
//! scheme: a trio that can work the secret out would see what a copy shows,
//! and a pair that learns nothing sees what an outsider sees.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Rail, Strip, Vault, card, fade_in_all, fade_out_all, fail, flash_all, pause, read_by, recolor_all, recolorings,
    say, swap, swaps, tag, think, wait_until,
};
use crate::model::{Dial, OPENING_TRIO, PAIR, SECRET, THRESHOLD, TRUSTEES, View, combinations};
use crate::palette;

pub struct TheVault {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FOR_OUTSIDE: &str = "for: anyone outside";
const FOR_LOST: &str = "for: everyone, once the note is lost";
const FOR_ONE: &str = "for: any one trustee";
const WANTED_TRIO: &str = "wanted, for any three";
const LOST: &str = "lost";
const AWAY: &str = "away";

/// 01_3's single keeper: the first trustee.
const KEEPER: u32 = TRUSTEES[0];
/// The wrong mark 01_1 turns the needle to: any mark but the secret.
const WRONG_MARK: u32 = 3;

// ------------------------------------------------------------------ layout

/// The think ring at the lower right, under the strip's caption.
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// The threshold card under the caption, left of the think ring.
const THRESHOLD_X: f64 = 2.3;
const THRESHOLD_Y: f64 = -0.9;
const THRESHOLD_SIZE: f64 = 30.0;

/// The caption for one trustee's view.
fn for_trustee(t: u32) -> String {
    format!("for: trustee {t}")
}

/// The caption for whoever bribed trustee `t`.
fn for_briber(t: u32) -> String {
    format!("for: whoever bribes trustee {t}")
}

/// The caption for the pair's wanted view.
fn wanted_pair(pair: [u32; 2]) -> String {
    format!("wanted, for trustees {} and {}", pair[0], pair[1])
}

/// The caption for a named trio's wanted view.
fn wanted_trio(trio: [u32; 3]) -> String {
    format!("wanted, for trustees {}, {} and {}", trio[0], trio[1], trio[2])
}

/// The threshold card.
fn threshold_card(k: usize) -> String {
    format!("threshold {k}")
}

/// 01_6's exact version: what opening the vault means from here on.
fn opening_card() -> String {
    "opening the vault means working out the secret from what the trustees were handed, not guessing it".to_owned()
}

impl SceneConstruct for TheVault {
    fn name(&self) -> &str {
        "01_the_vault"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let secret = SECRET.to_string();

        // 01_1: the trustees, the vault, and a wrong mark's alarm
        let line = say(&kit, stage, "01_1")?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 1.6)?;
        line.wait_for(stage, "one vault")?;
        let vault = Vault::build(&kit, stage, dial)?;
        vault.show(stage, 0.6)?;
        line.wait_for(stage, "a wrong mark")?;
        vault.turn(stage, WRONG_MARK, 0.7)?;
        vault.alarm(stage)?;
        vault.turn(stage, 0, 0.5)?;

        // 01_2: the strip, every mark possible for an outsider
        let line = say(&kit, stage, "01_2")?;
        line.wait_for(stage, "any of the eleven marks")?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 1.4)?;
        line.wait_for(stage, "This strip shows")?;
        strip.show(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.4)?;
        line.wait_end(stage)?;
        // the full strip stays a moment (a named hold)
        pause(stage, 1.0)?;

        // 01_3: one keeper; a lost note; a bribe
        let line = say(&kit, stage, "01_3")?;
        line.wait_for(stage, "tell one trustee")?;
        let keeper_card = card(&kit, stage, KEEPER, &secret, palette::secret())?;
        fade_in_all(stage, &[keeper_card], 0.4)?;
        flash_all(stage, &[rail.disc(KEEPER)], 0.4)?;
        strip.show(&kit, stage, &View::Copy(SECRET), &for_trustee(KEEPER), 0.8)?;
        if strip.lit() != [SECRET] {
            return Err(fail(format!("the keeper's strip lights {:?}", strip.lit())));
        }
        line.wait_for(stage, "one lost note")?;
        let lost = tag(&kit, stage, KEEPER, LOST, palette::leak())?;
        swap(stage, &[keeper_card], &[lost], 0.4)?;
        strip.show(&kit, stage, &View::Outsider, FOR_LOST, 0.8)?;
        vault.pulse(stage, 0.5)?;
        // the bribe is the other failure, not the next one: the keeper holds
        // the note again, now red because it reaches the wrong person, and the
        // door and the strip change in the same move
        line.wait_for(stage, "one bribe")?;
        let leaked = card(&kit, stage, KEEPER, &secret, palette::leak())?;
        let mut with = swaps(stage, &[lost], &[leaked], 0.6)?;
        with.extend(recolorings(stage, &[(vault.door, palette::leak())], 0.6)?);
        strip.show_with(&kit, stage, &View::Copy(SECRET), &for_briber(KEEPER), 0.6, with)?;

        // 01_4: a copy each; one absence is fine, but any one opens it alone
        let line = say(&kit, stage, "01_4")?;
        line.wait_for(stage, "a copy")?;
        let mut copies = Vec::with_capacity(TRUSTEES.len());
        for &t in &TRUSTEES {
            copies.push(card(&kit, stage, t, &secret, palette::secret())?);
        }
        // one move, so "One absence" is not drawn late
        let mut with = swaps(stage, &[leaked], &copies, 0.6)?;
        with.extend(recolorings(stage, &[(vault.door, palette::ghost())], 0.6)?);
        strip.show_with(&kit, stage, &View::Copy(SECRET), FOR_ONE, 0.6, with)?;
        line.wait_for(stage, "One absence")?;
        recolor_all(stage, &[(rail.disc(KEEPER), palette::ghost())], 0.3)?;
        recolor_all(stage, &[(vault.door, palette::open())], 0.3)?;
        pause(stage, 0.4)?;
        recolor_all(
            stage,
            &[(rail.disc(KEEPER), palette::share()), (vault.door, palette::ghost())],
            0.3,
        )?;
        line.wait_for(stage, "open it alone")?;
        // each trustee in turn, alone, and the door opens for each (about
        // two seconds in all, so "let it slip" is not left far behind)
        for &t in &TRUSTEES {
            recolor_all(stage, &[(vault.door, palette::open())], 0.08)?;
            // Pulse the trustee, not its typeset digit: the pinned transform
            // duplicates the glyph during a text pulse in actual frames.
            flash_all(stage, &[rail.disc(t)], 0.26)?;
            recolor_all(stage, &[(vault.door, palette::ghost())], 0.08)?;
        }
        line.wait_for(stage, "let it slip")?;
        // The pinned text colour transform duplicates glyphs in rendered
        // frames. Crossfade complete replacement cards at the same anchors.
        let red = TRUSTEES
            .iter()
            .map(|&t| card(&kit, stage, t, &secret, palette::leak()))
            .collect::<fmn::Result<Vec<_>>>()?;
        swap(stage, &copies, &red, 0.25)?;
        copies = red;
        // Keep the red warning legible instead of scaling the glyphs.
        pause(stage, 0.5)?;

        // 01_5: what they want: any three, no two; the threshold
        let line = say(&kit, stage, "01_5")?;
        fade_out_all(stage, &copies, 0.4)?;
        line.wait_for(stage, "Any three of them")?;
        strip.show(&kit, stage, &View::Copy(SECRET), WANTED_TRIO, 0.4)?;
        for trio in combinations(&TRUSTEES, THRESHOLD) {
            let mut lit = rail.tint(&trio, palette::trio());
            lit.push((vault.door, palette::open()));
            recolor_all(stage, &lit, 0.15)?;
            pause(stage, 0.12)?;
            let mut back = rail.tint(&trio, palette::share());
            back.push((vault.door, palette::ghost()));
            recolor_all(stage, &back, 0.15)?;
        }
        line.wait_for(stage, "any two should learn nothing")?;
        recolor_all(stage, &rail.tint(&PAIR, palette::pair()), 0.3)?;
        strip.show(&kit, stage, &View::Outsider, &wanted_pair(PAIR), 0.6)?;
        vault.pulse(stage, 0.4)?;
        line.wait_for(stage, "That number, three")?;
        let threshold = kit.text(
            stage,
            &threshold_card(THRESHOLD),
            THRESHOLD_SIZE,
            palette::trio(),
            v(THRESHOLD_X, THRESHOLD_Y),
        )?;
        fade_in_all(stage, &[threshold], 0.4)?;

        // 01_6: what opening means; two away, three still open; the question
        let line = say(&kit, stage, "01_6")?;
        recolor_all(stage, &rail.tint(&PAIR, palette::share()), 0.3)?;
        let meaning = opening_card();
        let note = precisely(&kit, stage, &meaning)?;
        fade_in_all(stage, &[note], 0.5)?;
        let read = read_by(stage, &meaning);
        line.wait_for(stage, "two trustees can be away")?;
        let mut away = Vec::with_capacity(PAIR.len());
        for &t in &PAIR {
            away.push(tag(&kit, stage, t, AWAY, Palette::dim())?);
        }
        let mut changes = rail.tint(&PAIR, palette::ghost());
        changes.extend(rail.tint(&OPENING_TRIO, palette::trio()));
        changes.push((vault.door, palette::open()));
        // the strip follows the trio that is left, not the pair that went
        let mut with = recolorings(stage, &changes, 0.4)?;
        with.extend(swaps(stage, &[], &away, 0.4)?);
        strip.show_with(&kit, stage, &View::Copy(SECRET), &wanted_trio(OPENING_TRIO), 0.4, with)?;
        line.wait_for(stage, "Before any mathematics")?;
        wait_until(stage, read)?;
        let mut back = rail.tint(&TRUSTEES, palette::share());
        back.push((vault.door, palette::ghost()));
        let with = recolorings(stage, &back, 0.4)?;
        strip.show_with(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.4, with)?;
        away.push(note);
        fade_out_all(stage, &away, 0.3)?;
        flash_all(stage, &rail.frames, 0.6)?;
        // the question stays open: chapter 02 takes its likely first answer
        think(&kit, stage, v(THINK_X, THINK_Y))?;
        pause(stage, 0.5)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{
        DOOR_H, DOOR_PULSE, DOOR_Y, FRAME_H, FRAME_Y, LAMP_R, LAMP_Y, STRIP_CAPTION_Y, STRIP_LEFT, THINK_R, strip_right,
    };
    use super::*;
    use crate::facts;
    use crate::model::MARKS;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "one vault"),
            ("01_1", "a wrong mark"),
            ("01_2", "any of the eleven marks"),
            ("01_2", "This strip shows"),
            ("01_3", "tell one trustee"),
            ("01_3", "one lost note"),
            ("01_3", "one bribe"),
            ("01_4", "a copy"),
            ("01_4", "One absence"),
            ("01_4", "open it alone"),
            ("01_4", "let it slip"),
            ("01_5", "Any three of them"),
            ("01_5", "any two should learn nothing"),
            ("01_5", "That number, three"),
            ("01_6", "two trustees can be away"),
            ("01_6", "Before any mathematics"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the scene draws is the facts': five trustees, eleven marks, the
    /// copy's "5", one possible secret for a keeper, a door opened once per
    /// copy, ten trios, the pair 4 and 5 who are away, and threshold 3.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        assert_eq!(f.get("S-F1.trustees"), Some(TRUSTEES.len().to_string().as_str()));
        assert_eq!(f.get("S-F1.marks"), Some(dial.marks().to_string().as_str()));
        assert_eq!(f.get("S-F1.secret"), Some(SECRET.to_string().as_str()));
        let keeper = View::Copy(SECRET).possible(dial).len();
        assert_eq!(
            f.get("S-F0.copies.secrets_left_for_one"),
            Some(keeper.to_string().as_str())
        );
        assert_eq!(View::Outsider.possible(dial).len(), MARKS as usize);
        assert_eq!(
            f.get("S-F0.copies.who_can_open_alone"),
            Some(TRUSTEES.len().to_string().as_str())
        );
        let trios = combinations(&TRUSTEES, THRESHOLD).len();
        assert_eq!(f.get("S-F3.trios"), Some(trios.to_string().as_str()));
        assert_eq!(f.get("S-F1.pair"), Some(format!("{},{}", PAIR[0], PAIR[1]).as_str()));
        assert_eq!(f.get("S-F1.too_few"), Some(PAIR.len().to_string().as_str()));
        assert_eq!(
            f.get("S-F1.can_be_away"),
            Some(PAIR.len().to_string().as_str()),
            "01_6's pair away"
        );
        assert_eq!(f.get("S-F1.threshold"), Some(THRESHOLD.to_string().as_str()));
        assert_eq!(f.get("S-F1.opening_trio"), Some("1,2,3"));
        // the trio still opening when the pair is away is the pair's complement
        let mut everyone: Vec<u32> = OPENING_TRIO.iter().chain(&PAIR).copied().collect();
        everyone.sort_unstable();
        assert_eq!(everyone, TRUSTEES);
        assert_eq!(threshold_card(THRESHOLD), "threshold 3");
        assert_eq!(wanted_pair(PAIR), "wanted, for trustees 4 and 5");
        assert_eq!(wanted_trio(OPENING_TRIO), "wanted, for trustees 1, 2 and 3");
        assert_eq!(for_trustee(KEEPER), "for: trustee 1");
        assert_ne!(WRONG_MARK, SECRET, "the alarm's mark is a wrong one");
        assert!(WRONG_MARK < dial.marks());
    }

    /// The cards stay two lines at every wrap the lower-third can narrow to.
    #[test]
    fn the_card_is_two_lines() {
        let text = opening_card();
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// The captions fit right of the strip's left edge inside the frame
    /// (about 0.12 units a character at their size, generously), and the
    /// threshold card and think ring keep apart, under the caption and over
    /// the lower-third band.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        const CHAR_W: f64 = 0.12;
        for caption in [
            FOR_OUTSIDE.to_owned(),
            FOR_LOST.to_owned(),
            FOR_ONE.to_owned(),
            WANTED_TRIO.to_owned(),
            for_trustee(KEEPER),
            for_briber(KEEPER),
            wanted_pair(PAIR),
            wanted_trio(OPENING_TRIO),
        ] {
            let right = STRIP_LEFT + CHAR_W * caption.chars().count() as f64;
            assert!(right < FRAME_X_RADIUS - 0.25, "{caption}");
            // the caption ends left of the think ring, or sits above it
            assert!(right < THINK_X - THINK_R - 0.2 || STRIP_CAPTION_Y - 0.2 > THINK_Y + THINK_R + 0.2);
        }
        let card_w = 0.18 * threshold_card(THRESHOLD).chars().count() as f64;
        assert!(THRESHOLD_X - card_w / 2.0 > STRIP_LEFT - 0.6);
        assert!(THRESHOLD_X + card_w / 2.0 < THINK_X - THINK_R - 0.3);
        const {
            assert!(THRESHOLD_Y + 0.25 < STRIP_CAPTION_Y - 0.2, "the card under the caption");
            assert!(THRESHOLD_Y - 0.25 > BAND_TOP + 0.3, "the card over the band");
            assert!(THINK_Y - THINK_R - 0.45 > BAND_TOP, "the think note over the band");
            assert!(THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2, "the ring under the caption");
            // the door's pulse grows it about its centre
            assert!(
                DOOR_Y - DOOR_PULSE * DOOR_H / 2.0 > FRAME_Y + FRAME_H / 2.0 + 0.1,
                "the pulsing door over the card frames"
            );
            assert!(
                DOOR_Y + DOOR_PULSE * DOOR_H / 2.0 < LAMP_Y - LAMP_R - 0.03,
                "the pulsing door under the lamp"
            );
        }
        assert!(THINK_X + THINK_R < strip_right(MARKS) + 0.3);
        const { assert!(THINK_X + THINK_R < FRAME_X_RADIUS - 0.25) };
    }
}
