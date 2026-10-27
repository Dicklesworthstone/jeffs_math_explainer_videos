//! Chapter 02, cutting it up: whatever a trustee is handed is their share,
//! and the first idea is to make each share a piece of the secret (02_1);
//! the smallest case, a vault for A and B, the secret 5 cut into 2 and 3
//! (02_2); together they open it, and the question of what B's 3 says on
//! its own (02_3); quite a lot, since A's piece is 0 or more, so 0, 1 and 2
//! are struck off (02_4); the bigger the piece, the more it gives away
//! (02_5); the second failure, needing both: with A away, B still has eight
//! possible secrets and the vault stays shut (02_6); the two failures named
//! apart, and the dial zooms in for the fix (02_7).
//!
//! It opens where chapter 01 ends: the vault, the rail with its empty
//! frames, and the strip for anyone outside. The strip's view for B is
//! `View::OrdinaryPiece` of the piece on B's card, checked against the
//! search over A's piece from that same piece (`two_keys`). The strip keeps
//! the outsider's caption through the question, so it never shows B a set
//! that is not yet B's.

use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Rail, STRIP_LEFT, Strip, Vault, fade_in_all, fade_in_lagged, fade_out_all, fail, pause, recolor, recolor_all, say,
    stroke, swap, swaps, tag, think, tick_at,
};
use super::two_keys::{
    BIG_R, Face, MID_X, TwoKeys, big_centre, card_back, check_strip, glow, glowing, holder_card, leaving,
    leaving_all_but, moving, next_top, ordinary_secrets, ordinary_view, slot_at, slot_halo, text_halo,
};
use crate::model::{Dial, ORDINARY_PIECES, SECRET, TRUSTEES, View};
use crate::palette;

pub struct CuttingItUp {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FOR_OUTSIDE: &str = "for: anyone outside";
const FOR_B: &str = "for: B";
const FOR_B_ALONE: &str = "for: B, with A away";
const SHARE: &str = "share";
const LEAKS: &str = "each piece leaks";
const NEEDS_BOTH: &str = "needs A and B both";
const A_AT_LEAST_NOTHING: &str = "A's piece ≥ 0";

/// A and B, as holder indices.
const A: usize = 0;
const B: usize = 1;

// ------------------------------------------------------------------ layout

/// The whole secret of 02_1, in the gap between the big door and the
/// strip, and its cut line's half height and dashes.
const WHOLE_X: f64 = -0.85;
const WHOLE_Y: f64 = 1.0;
const WHOLE_SIZE: f64 = 72.0;
const CUT_HALF: f64 = 0.62;
const CUT_DASHES: usize = 6;
/// The sum and the notes, in the two-key vault's middle column.
const SUM_Y: f64 = 1.6;
const SUM_SIZE: f64 = 48.0;
const NOTE_Y: [f64; 2] = [0.7, 0.2];
const NOTE_SIZE: f64 = 22.0;
/// The two failures' tags: the leak under the strip's caption, needing
/// both beside the door.
const LEAK_Y: f64 = -0.75;
const BOTH_Y: f64 = -0.55;
const TAG_SIZE: f64 = 24.0;
/// The think ring at the lower right, under the strip's caption (as 01's).
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// How far the rail and the big vault slide as they go aside.
const ASIDE: f64 = -2.5;

/// The note that the secret is at least B's piece.
fn at_least(b: u32) -> String {
    format!("secret = A's piece + {b} ≥ {b}")
}

/// The sum the two pieces make.
fn sum_tex(a: u32, b: u32) -> String {
    format!("{a} + {b} = {}", a + b)
}

impl SceneConstruct for CuttingItUp {
    fn name(&self) -> &str {
        "02_cutting_it_up"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let [a_piece, b_piece] = ORDINARY_PIECES;
        if a_piece + b_piece != SECRET {
            return Err(fail(format!(
                "the pieces {a_piece} and {b_piece} do not add up to the secret {SECRET}"
            )));
        }

        // 02_1: where 01 ended; the frames hold shares; a piece of the secret
        let line = say(&kit, stage, "02_1")?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        let vault = Vault::build(&kit, stage, dial)?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        vault.show(stage, 0.5)?;
        rail.reveal(stage, 0.6)?;
        // what goes aside in 02_2: the vault and the rail, as they sit now
        let first: Vec<Mob> = stage.scene().mobjects().to_vec();
        strip.reveal(&kit, stage, 0.6)?;
        strip.show(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.3)?;
        line.wait_for(stage, "call it their share")?;
        let mut shares = Vec::with_capacity(TRUSTEES.len());
        for &t in &TRUSTEES {
            shares.push(tag(&kit, stage, t, SHARE, Palette::dim())?);
        }
        fade_in_lagged(stage, &shares, 0.8, 0.3)?;
        line.wait_for(stage, "a piece of the secret")?;
        let five = kit.text(
            stage,
            &SECRET.to_string(),
            WHOLE_SIZE,
            palette::secret(),
            v(WHOLE_X, WHOLE_Y),
        )?;
        let mut whole = vec![five];
        let dash = 2.0 * CUT_HALF / (2 * CUT_DASHES - 1) as f64;
        for i in 0..CUT_DASHES {
            let top = WHOLE_Y + CUT_HALF - 2.0 * dash * i as f64;
            whole.push(stroke(
                stage,
                vec![v(WHOLE_X, top), v(WHOLE_X, top - dash)],
                Palette::ink(),
                3.0,
            ));
        }
        fade_in_all(stage, &whole, 0.5)?;

        // 02_2: the smallest case, A and B under a small door; the cut
        let line = say(&kit, stage, "02_2")?;
        line.wait_for(stage, "the smallest case")?;
        let two = TwoKeys::build(&kit, stage, dial, 0)?;
        // the rail and the big vault slide aside with their share tags as
        // the small door comes, the strip stays, and the whole secret moves
        // to the middle column
        let mut going = first;
        going.extend(&shares);
        let mut with = leaving(stage, &going, v(ASIDE, 0.0), 0.8)?;
        with.extend(two.arriving(stage, 0.8)?);
        let face_in = fade_in(stage.arena_mut(), two.face.parts()[0], ORIGIN, 1.0)?;
        with.push(stage.prepare(face_in.rt(0.8))?);
        for &m in &whole {
            let at = stage.get_center(m);
            with.push(moving(stage, m, v(at[0] + MID_X - WHOLE_X, at[1]), 0.8)?);
        }
        stage.play_prepared(with)?;
        stage.add_to_scene(two.face.parts()[1])?;
        line.wait_for(stage, "two for A")?;
        // the 5 splits into the two pieces, A's first
        let a_card = holder_card(&kit, stage, A, &a_piece.to_string(), palette::share())?;
        let b_card = holder_card(&kit, stage, B, &b_piece.to_string(), palette::share())?;
        stage.move_to(a_card, v(MID_X - 0.3, WHOLE_Y), ORIGIN);
        stage.move_to(b_card, v(MID_X + 0.3, WHOLE_Y), ORIGIN);
        swap(stage, &whole, &[a_card, b_card], 0.4)?;
        let to_a = moving(stage, a_card, slot_at(A), 0.6)?;
        stage.play_prepared(vec![to_a])?;
        line.wait_for(stage, "three for B")?;
        let to_b = moving(stage, b_card, slot_at(B), 0.6)?;
        stage.play_prepared(vec![to_b])?;

        // 02_3: together they open it; what does B's piece say alone?
        let line = say(&kit, stage, "02_3")?;
        let sum = kit.tex(
            stage,
            &sum_tex(a_piece, b_piece),
            SUM_SIZE,
            Palette::ink(),
            &[],
            v(MID_X, SUM_Y),
        )?;
        fade_in_all(stage, &[sum], 0.4)?;
        let cards = [slot_halo(stage, A)?, slot_halo(stage, B)?];
        glow(stage, &cards, 0.4)?;
        line.wait_for(stage, "the door opens")?;
        two.face.turn(stage, i64::from(a_piece + b_piece), 0.5)?;
        recolor_all(stage, &[(two.door, palette::open())], 0.3)?;
        pause(stage, 0.3)?;
        // shut again, the needle on to the top, in one move, so the secret
        // is not left showing on the dial for the question
        let to_top = next_top(two.face.at(stage), dial.marks());
        let anims = vec![
            stage.prepare(recolor(two.door, palette::ghost(), 0.4)?)?,
            stage.prepare(tween(two.face.needle, to_top as f64, 0.4)?)?,
        ];
        stage.play_prepared(anims)?;
        line.wait_for(stage, "what does B's three say")?;
        let a_back = card_back(stage, A)?;
        let mut with = swaps(stage, &[a_card, sum], &[a_back], 0.4)?;
        let b_halo = slot_halo(stage, B)?;
        with.extend(glowing(stage, &[b_halo], 0.5)?);
        stage.play_prepared(with)?;
        line.wait_end(stage)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 02_4: quite a lot: A's piece is 0 or more, so 0, 1, 2 go
        let line = say(&kit, stage, "02_4")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "A's piece can't be less than zero")?;
        let floor = kit.text(
            stage,
            A_AT_LEAST_NOTHING,
            NOTE_SIZE,
            Palette::ink(),
            v(MID_X, NOTE_Y[0]),
        )?;
        fade_in_all(stage, &[floor], 0.4)?;
        let a_halo = slot_halo(stage, A)?;
        glow(stage, &[a_halo], 0.4)?;
        line.wait_for(stage, "at least three")?;
        let least = kit.text(
            stage,
            &at_least(b_piece),
            NOTE_SIZE,
            palette::share(),
            v(MID_X, NOTE_Y[1]),
        )?;
        fade_in_all(stage, &[least], 0.4)?;
        line.wait_for(stage, "Zero, one and two")?;
        // the view and the search both come from the piece on B's card
        let view = ordinary_view(b_piece);
        let found = ordinary_secrets(dial, b_piece);
        let struck: Vec<u32> = dial.all().filter(|s| !found.contains(s)).collect();
        let mut strikes = Vec::with_capacity(struck.len());
        for &k in &struck {
            let at = tick_at(k);
            strikes.push(stroke(
                stage,
                vec![v(at[0] - 0.22, at[1] - 0.42), v(at[0] + 0.22, at[1] + 0.42)],
                palette::leak(),
                5.0,
            ));
        }
        let with = swaps(stage, &[], &strikes, 0.6)?;
        strip.show_with(&kit, stage, &view, FOR_B, 0.6, with)?;
        check_strip(&strip, &found, FOR_B)?;

        // 02_5: the bigger the piece, the more it gives away
        let line = say(&kit, stage, "02_5")?;
        line.wait_for(stage, "the bigger the piece")?;
        let mut leaving_notes = leaving(stage, &strikes, ORIGIN, 0.3)?;
        leaving_notes.extend(leaving(stage, &[floor, least], ORIGIN, 0.3)?);
        stage.play_prepared(leaving_notes)?;
        // the secret stays 5: B's piece runs 0 to 5, A's is the rest, shown
        // dim beside it, and the strip follows B's piece
        let (mut a_now, mut b_now) = (a_back, b_card);
        for b in 0..=SECRET {
            let a_new = holder_card(&kit, stage, A, &(SECRET - b).to_string(), Palette::dim())?;
            let b_new = holder_card(&kit, stage, B, &b.to_string(), palette::share())?;
            let with = swaps(stage, &[a_now, b_now], &[a_new, b_new], 0.35)?;
            strip.show_with(&kit, stage, &ordinary_view(b), FOR_B, 0.35, with)?;
            check_strip(&strip, &ordinary_secrets(dial, b), FOR_B)?;
            (a_now, b_now) = (a_new, b_new);
            pause(stage, 0.12)?;
        }
        // back where the sweep began: B holds 3, A's card face down
        let a_back = card_back(stage, A)?;
        let b_card = holder_card(&kit, stage, B, &b_piece.to_string(), palette::share())?;
        let with = swaps(stage, &[a_now, b_now], &[a_back, b_card], 0.35)?;
        strip.show_with(&kit, stage, &view, FOR_B, 0.35, with)?;
        check_strip(&strip, &found, FOR_B)?;

        // 02_6: the second failure: both are needed
        let line = say(&kit, stage, "02_6")?;
        line.wait_for(stage, "with A away")?;
        recolor_all(
            stage,
            &[(two.disc(A), palette::ghost()), (a_back, palette::ghost())],
            0.4,
        )?;
        line.wait_for(stage, "B's three still leaves")?;
        strip.show(&kit, stage, &view, FOR_B_ALONE, 0.4)?;
        check_strip(&strip, &found, FOR_B_ALONE)?;
        let b_halo = slot_halo(stage, B)?;
        glow(stage, &[b_halo], 0.4)?;
        line.wait_for(stage, "the vault stays shut")?;
        two.pulse(stage, 0.5)?;

        // 02_7: the two failures, apart; the dial zooms in for the fix
        let line = say(&kit, stage, "02_7")?;
        line.wait_for(stage, "leaks")?;
        let leak = kit.text_left(stage, LEAKS, TAG_SIZE, palette::leak(), v(STRIP_LEFT, LEAK_Y))?;
        fade_in_all(stage, &[leak], 0.4)?;
        line.wait_for(stage, "it needs every piece")?;
        let both = kit.text(stage, NEEDS_BOTH, TAG_SIZE, palette::leak(), v(MID_X, BOTH_Y))?;
        fade_in_all(stage, &[both], 0.4)?;
        two.pulse(stage, 0.4)?;
        line.wait_for(stage, "The leak gets fixed first")?;
        let leak_halo = text_halo(stage, leak, palette::leak())?;
        glow(stage, &[leak_halo], 0.5)?;
        line.wait_for(stage, "the fix is the dial itself")?;
        // everything else goes, and the needle points where it did
        let big = Face::build(&kit, stage, dial, big_centre(), BIG_R, Palette::dim(), 0)?;
        let rest = leaving_all_but(stage, &two.face.parts(), ORIGIN, 0.9)?;
        two.face.zoom(stage, &big, 0.9, rest)?;
        line.wait_end(stage)?;
        // the enlarged dial stays a moment (the chapter's transition)
        pause(stage, 0.6)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::FRAME_X_RADIUS;

    use super::super::draw::{DOOR_W, DOOR_X, STRIP_CAPTION_Y, THINK_R, strip_right};
    use super::super::two_keys::{SMALL_DIAL_R, SMALL_W, SMALL_X, TEXT_PAD};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, csv};

    const SCRIPT: &str = crate::SCRIPT;

    /// Every phrase the scene waits for is in its line of the draft script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("02_1", "call it their share"),
            ("02_1", "a piece of the secret"),
            ("02_2", "the smallest case"),
            ("02_2", "two for A"),
            ("02_2", "three for B"),
            ("02_3", "the door opens"),
            ("02_3", "what does B's three say"),
            ("02_4", "A's piece can't be less than zero"),
            ("02_4", "at least three"),
            ("02_4", "Zero, one and two"),
            ("02_5", "the bigger the piece"),
            ("02_6", "with A away"),
            ("02_6", "B's three still leaves"),
            ("02_6", "the vault stays shut"),
            ("02_7", "leaks"),
            ("02_7", "it needs every piece"),
            ("02_7", "The leak gets fixed first"),
            ("02_7", "the fix is the dial itself"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the scene draws is the facts': the pieces 2 and 3 adding up to
    /// the secret, B's piece leaving 3 to 10 (eight secrets) and striking
    /// 0, 1 and 2, and two pieces both needed; the sweep's pieces leave
    /// 11 − b for each b, and each pair adds up to the secret.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let [a, b] = ORDINARY_PIECES;
        assert_eq!(f.get("S-F0.pieces"), Some(csv(&ORDINARY_PIECES).as_str()));
        assert_eq!(f.get("S-F0.pieces.sum"), Some((a + b).to_string().as_str()));
        assert_eq!(sum_tex(a, b), "2 + 3 = 5");
        let found = ordinary_secrets(dial, b);
        assert_eq!(f.get("S-F0.pieces.b_alone_candidates"), Some(csv(&found).as_str()));
        assert_eq!(
            f.get("S-F0.pieces.b_alone_left"),
            Some(found.len().to_string().as_str())
        );
        let struck: Vec<u32> = dial.all().filter(|s| !found.contains(s)).collect();
        assert_eq!(f.get("S-F0.pieces.b_alone_ruled_out"), Some(csv(&struck).as_str()));
        assert_eq!(struck, (0..b).collect::<Vec<_>>());
        assert_eq!(
            f.get("S-F0.pieces.both_needed"),
            Some(ORDINARY_PIECES.len().to_string().as_str())
        );
        assert_eq!(at_least(b), "secret = A's piece + 3 ≥ 3");
        for piece in 0..=SECRET {
            let left = ordinary_view(piece).possible(dial);
            assert_eq!(left.len() as u32, MARKS - piece);
            assert_eq!(left, ordinary_secrets(dial, piece));
            assert!(left.contains(&SECRET), "the secret stays possible");
        }
    }

    /// The captions and tags fit inside the frame, left of the think ring
    /// or above it (about 0.12 units a character at 22 points, 0.13 at 24);
    /// the notes and the tag beside the door stay in the middle column, and
    /// the whole secret of 02_1 between the big door and the strip.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        for caption in [FOR_OUTSIDE, FOR_B, FOR_B_ALONE] {
            let right = STRIP_LEFT + 0.12 * caption.chars().count() as f64;
            assert!(right < FRAME_X_RADIUS - 0.25, "{caption}");
            assert!(right < THINK_X - THINK_R - 0.2 || STRIP_CAPTION_Y - 0.2 > THINK_Y + THINK_R + 0.2);
        }
        let leak_right = STRIP_LEFT + 0.13 * LEAKS.chars().count() as f64;
        assert!(leak_right < strip_right(MARKS));
        assert!(leak_right + TEXT_PAD < FRAME_X_RADIUS - 0.25, "the leak tag's halo");
        let both_w = 0.13 * NEEDS_BOTH.chars().count() as f64;
        assert!(MID_X - both_w / 2.0 > SMALL_X + SMALL_W / 2.0 + 0.1);
        assert!(
            MID_X + both_w / 2.0 < STRIP_LEFT - TEXT_PAD - 0.1,
            "the leak tag's halo clear of the tag beside the door"
        );
        let note_w = 0.12 * at_least(ORDINARY_PIECES[1]).chars().count() as f64;
        assert!(MID_X - note_w / 2.0 > SMALL_X + SMALL_W / 2.0 + 0.1);
        assert!(MID_X + note_w / 2.0 < STRIP_LEFT - 0.1);
        const {
            assert!(LEAK_Y + 0.2 < STRIP_CAPTION_Y - 0.2, "the leak tag under the caption");
            assert!(
                LEAK_Y + 0.0065 * TAG_SIZE / 2.0 + TEXT_PAD < STRIP_CAPTION_Y - 0.15,
                "the leak tag's halo under the caption"
            );
            assert!(
                WHOLE_X - 0.5 > DOOR_X + DOOR_W / 2.0,
                "the whole secret right of the big door"
            );
            assert!(BIG_R > SMALL_DIAL_R, "the dial zooms in");
            assert!(WHOLE_X + 0.5 < STRIP_LEFT, "the whole secret left of the strip");
            assert!(NOTE_Y[1] - 0.2 > BOTH_Y + 0.2, "the notes over the tag");
            assert!(SUM_Y - 0.3 > NOTE_Y[0] + 0.2, "the sum over the notes");
        }
    }
}
