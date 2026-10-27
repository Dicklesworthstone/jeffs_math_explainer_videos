//! Chapter 03, the dial: adding on the dial is turning, from 9 seven
//! clicks on, past 10 and round through 0, to 5 (03_1); so nine plus seven
//! is five, ten plus one is zero, and taking away is turning back, 5 back
//! seven clicks to 9 (03_2); sharing on the dial: the dealer's fair spin
//! gives A 9, and B gets the turn from 9 to the secret, 7 (03_3); together
//! they open it, and the question of what B's 7 says now (03_4); nothing:
//! each mark A could hold makes a different secret, all eleven, once each
//! (03_5); the spin is fair, so B's 7 fits every secret equally well, beside
//! the cut piece's floor (03_6); the leak is fixed, but this vault still
//! needs both, and the five trustees return (03_7).
//!
//! It opens where chapter 02 ends, on the enlarged dial. Every sum is shown
//! as the needle's clicks before it is written. The strip's view for B is
//! computed from the share on B's card (`two_keys::dial_view`) and checked
//! against the table's search over A's marks from that same share; the
//! strip keeps the outsider's caption through the question.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Rail, STRIP_DX, STRIP_LEFT, STRIP_Y, Strip, TICK_H, TICK_W, Vault, bundle, disc, fade_in_all, fade_in_lagged, fail,
    panel, pause, read_by, recolor, recolor_all, say, stroke, swap, swaps, think, tick_at, wait_until,
};
use super::two_keys::{
    ARC_LABEL_IN, BIG_R, BIG_X, BIG_Y, Face, MID_X, SPARE_R, SPARE_Y, TwoKeys, big_centre, card_back, check_strip,
    dial_secrets, dial_view, glow, glowing, holder_card, if_a_held, leaving, move_all, next_top, ordinary_secrets,
    ordinary_view, slot_at, slot_halo, swell_all,
};
use crate::model::{A_SHARE, Dial, ORDINARY_PIECES, Ratio, SECRET, THRESHOLD, TRUSTEES, View};
use crate::palette;

pub struct TheDial {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const FOR_OUTSIDE: &str = "for: anyone outside";
const FOR_B: &str = "for: B";
const TURNED: &str = "marks turned";
const TURNED_BACK: &str = "marks turned back";
const SPARE_TITLE: &str = "dealer's fair spin";
const FLOOR: &str = "floor";
const LEAKS_NOTHING: &str = "B's share leaks nothing";
const STILL_BOTH: &str = "still needs A and B both";

/// A and B, as holder indices.
const A: usize = 0;
const B: usize = 1;

// ------------------------------------------------------------------ layout

/// The equations right of the enlarged dial: the sum, its unwrapping in
/// small grey, ten plus one, and the taking away.
const TEX_X: f64 = 2.9;
const TEX_Y: [f64; 4] = [2.3, 1.65, 0.6, -0.6];
const TEX_SIZE: f64 = 52.0;
const SMALL_TEX_SIZE: f64 = 32.0;
/// The click count under the enlarged dial: the number, then its words.
const COUNT_Y: f64 = BIG_Y - BIG_R - 0.55;
const COUNT_SIZE: f64 = 32.0;
/// The spare dial's title and the dealer's secret, over and under it.
const SPARE_TITLE_DY: f64 = SPARE_R + 0.35;
/// 03_5's table in the middle column, top row first, and the dots that
/// mark each secret met, over the strip's ticks.
const ROW_Y0: f64 = 2.6;
const ROW_DY: f64 = 0.4;
const ROW_SIZE: f64 = 20.0;
const DOT_Y: f64 = STRIP_Y + TICK_H / 2.0 + 0.2;
const DOT_R: f64 = 0.07;
/// 03_6's line of the cut piece, under the strip's caption, column for
/// column with the strip.
const MINI_Y: f64 = -0.85;
const MINI_H: f64 = 0.34;
const MINI_LABEL_DY: f64 = 0.42;
const MINI_CAPTION_DY: f64 = 0.8;
/// The closing tags, as chapter 02's, and the threshold card, as 01's.
const LEAK_Y: f64 = -0.75;
const BOTH_Y: f64 = -0.55;
const TAG_SIZE: f64 = 24.0;
const THRESHOLD_X: f64 = 2.3;
const THRESHOLD_Y: f64 = -0.9;
const THRESHOLD_SIZE: f64 = 30.0;
/// The think ring at the lower right, under the strip's caption (as 01's).
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// The quickest and slowest a click may be, so seven clicks fill the words
/// that describe them without blurring.
const CLICK_FAST: f64 = 0.3;
const CLICK_SLOW: f64 = 0.6;

/// Each click's time when `clicks` clicks should take `room` seconds.
fn click_time(room: f64, clicks: u32) -> f64 {
    (room / f64::from(clicks.max(1))).clamp(CLICK_FAST, CLICK_SLOW)
}

/// The sum as the dial makes it: the ordinary sum, then where it lands.
fn sum_tex(dial: Dial, a: u32, b: u32) -> String {
    format!(r"{a} + {b} = {} \to {}", a + b, dial.add(a, b))
}

/// The unwrapping, when the ordinary sum runs past the top.
fn unwrap_tex(dial: Dial, a: u32, b: u32) -> Option<String> {
    let total = a + b;
    (total >= dial.marks()).then(|| format!("{total} - {} = {}", dial.marks(), total - dial.marks()))
}

/// One past the top lands on zero.
fn top_tex(dial: Dial) -> String {
    let top = dial.marks() - 1;
    format!(r"{top} + 1 \to {}", dial.add(top, 1))
}

/// Taking away, as the dial does it.
fn back_tex(dial: Dial, s: u32, b: u32) -> String {
    format!(r"{s} - {b} \to {}", dial.sub(s, b))
}

/// A row of 03_5's table; the first says what the number is.
fn row_text(a: u32, s: u32, first: bool) -> String {
    if first {
        format!("A had {a} → secret {s}")
    } else {
        format!("A had {a} → {s}")
    }
}

/// 03_6's exact version.
fn chance_card(dial: Dial, b: u32) -> String {
    let m = dial.marks();
    format!(
        "for every secret s, exactly one of A's {m} equally likely marks gives B a {b}: P(B = {b} | s) = {}",
        Ratio::new(1, i64::from(m))
    )
}

/// The strip's caption for B on the dial in 03_6.
fn for_b_on_the_dial(b: u32) -> String {
    format!("for: B, holding {b} on the dial")
}

/// The caption of 03_6's line of the cut piece.
fn for_b_cut(piece: u32, left: usize) -> String {
    format!("for: B, holding a cut piece {piece}: {left} possible")
}

/// The threshold card (as chapter 01's).
fn threshold_card(k: usize) -> String {
    format!("threshold {k}")
}

impl SceneConstruct for TheDial {
    fn name(&self) -> &str {
        "03_the_dial"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let top = dial.marks() - 1;
        // B's share: the turn from A's spin to the secret
        let b = dial.sub(SECRET, A_SHARE);

        // 03_1: adding is turning, one mark a click
        let line = say(&kit, stage, "03_1")?;
        let big = Face::build(&kit, stage, dial, big_centre(), BIG_R, Palette::dim(), 0)?;
        big.show(stage, 0.6)?;
        line.wait_for(stage, "From nine")?;
        big.turn(stage, i64::from(A_SHARE), 0.5)?;
        let nine = big.halo(stage, A_SHARE, palette::secret());
        glow(stage, &[nine], 0.3)?;
        line.wait_for(stage, "turn seven marks forward")?;
        let count = stage.add_value_tracker(0.0);
        let words = kit.text_left(stage, TURNED, 24.0, Palette::dim(), v(BIG_X - 0.05, COUNT_Y))?;
        fade_in_all(stage, &[words], 0.3)?;
        let number = kit.live_number(
            stage,
            count,
            0,
            false,
            COUNT_SIZE,
            palette::secret(),
            v(BIG_X - 0.25, COUNT_Y),
            RIGHT,
        )?;
        let each = click_time(line.at("you stop at five")? - now(stage), b);
        let landed = big.click(stage, i64::from(b), each, Some(count))?;
        if landed.last() != Some(&dial.add(A_SHARE, b)) || !landed.contains(&top) || !landed.contains(&0) {
            return Err(fail(format!("the clicks from {A_SHARE} landed on {landed:?}")));
        }
        line.wait_for(stage, "stop at five")?;
        let five = big.halo(stage, SECRET, palette::secret());
        glow(stage, &[five], 0.5)?;

        // 03_2: so nine plus seven is five; ten plus one; turning back
        let line = say(&kit, stage, "03_2")?;
        line.wait_for(stage, "nine plus seven is five")?;
        let sum = kit.tex(
            stage,
            &sum_tex(dial, A_SHARE, b),
            TEX_SIZE,
            Palette::ink(),
            &[],
            v(TEX_X, TEX_Y[0]),
        )?;
        let unwrap = unwrap_tex(dial, A_SHARE, b).ok_or_else(|| fail("the sum does not run past the top".into()))?;
        let unwrapped = kit.tex(stage, &unwrap, SMALL_TEX_SIZE, Palette::dim(), &[], v(TEX_X, TEX_Y[1]))?;
        fade_in_all(stage, &[sum, unwrapped], 0.5)?;
        line.wait_for(stage, "Ten plus one is zero")?;
        let one_on = big.arc(
            &kit,
            stage,
            f64::from(top),
            f64::from(top) + 1.0,
            "+1",
            palette::secret(),
        )?;
        let ten_one = kit.tex(stage, &top_tex(dial), TEX_SIZE, Palette::ink(), &[], v(TEX_X, TEX_Y[2]))?;
        fade_in_all(stage, &[one_on[0], one_on[1], ten_one], 0.4)?;
        let ten = big.halo(stage, top, palette::secret());
        glow(stage, &[ten], 0.3)?;
        let zero = big.halo(stage, dial.add(top, 1), palette::secret());
        glow(stage, &[zero], 0.3)?;
        line.wait_for(stage, "turning back")?;
        let back_words = kit.text_left(stage, TURNED_BACK, 24.0, Palette::dim(), v(BIG_X - 0.05, COUNT_Y))?;
        // the arc goes, the count resets and its words change, in one move
        let mut anims = leaving(stage, &one_on, ORIGIN, 0.3)?;
        anims.push(stage.prepare(tween(count, 0.0, 0.3)?)?);
        anims.extend(swaps(stage, &[words], &[back_words], 0.3)?);
        stage.play_prepared(anims)?;
        let each = click_time(line.at("is nine")? - now(stage), b);
        let landed = big.click(stage, -i64::from(b), each, Some(count))?;
        if landed.last() != Some(&dial.sub(SECRET, b)) || !landed.contains(&0) || !landed.contains(&top) {
            return Err(fail(format!("the clicks back from {SECRET} landed on {landed:?}")));
        }
        line.wait_for(stage, "is nine")?;
        let back = kit.tex(
            stage,
            &back_tex(dial, SECRET, b),
            TEX_SIZE,
            Palette::ink(),
            &[],
            v(TEX_X, TEX_Y[3]),
        )?;
        fade_in_all(stage, &[back], 0.4)?;
        let nine = big.halo(stage, dial.sub(SECRET, b), palette::secret());
        glow(stage, &[nine], 0.4)?;

        // 03_3: sharing on the dial: a fair spin for A, the turn for B
        let line = say(&kit, stage, "03_3")?;
        line.wait_for(stage, "share the secret on the dial instead")?;
        // the equations and the count go, and the needle comes on to the top
        let mut anims = leaving(stage, &[sum, unwrapped, ten_one, back, back_words, number], ORIGIN, 0.5)?;
        let to_top = next_top(big.at(stage), dial.marks());
        anims.push(stage.prepare(tween(big.needle, to_top as f64, 0.5)?)?);
        stage.play_prepared(anims)?;
        // the dial shrinks back onto the small door; A and B return with
        // their old pieces; the strip returns for anyone outside
        let two = TwoKeys::build(&kit, stage, dial, 0)?;
        let [old_a, old_b] = ORDINARY_PIECES;
        let old = [
            holder_card(&kit, stage, A, &old_a.to_string(), palette::share())?,
            holder_card(&kit, stage, B, &old_b.to_string(), palette::share())?,
        ];
        let mut with = two.arriving(stage, 0.8)?;
        for &m in &old {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            with.push(stage.prepare(a.rt(0.8))?);
        }
        big.zoom(stage, &two.face, 0.8, with)?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 0.6)?;
        strip.show(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.3)?;
        line.wait_for(stage, "The dealer")?;
        let spare = Face::build(&kit, stage, dial, v(MID_X, SPARE_Y), SPARE_R, palette::spin(), 0)?;
        let title = kit.text(
            stage,
            SPARE_TITLE,
            22.0,
            palette::spin(),
            v(MID_X, SPARE_Y + SPARE_TITLE_DY),
        )?;
        let dealer_secret = kit.text(
            stage,
            &format!("secret {SECRET}"),
            24.0,
            palette::secret(),
            v(MID_X, SPARE_Y - SPARE_TITLE_DY),
        )?;
        swap(stage, &old, &[title, dealer_secret], 0.5)?;
        spare.show(stage, 0.4)?;
        line.wait_for(stage, "spins a spare dial fairly")?;
        // two whole turns and on to where it stops, slowing down
        let stop = f64::from(A_SHARE + 2 * dial.marks());
        let spin_it = spare
            .needle
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.6),
                rate_func: Some(fmn::core::rate::rush_from),
                ..AnimateArgs::default()
            })?
            .set_value(stop)?;
        stage.play(spin_it)?;
        line.wait_for(stage, "stops, nine")?;
        // a ring round the spare dial's 9 as A's card appears by it
        let a_card = holder_card(&kit, stage, A, &A_SHARE.to_string(), palette::share())?;
        stage.move_to(a_card, spare.point(f64::from(A_SHARE), SPARE_R * ARC_LABEL_IN), ORIGIN);
        let mut anims = swaps(stage, &[], &[a_card], 0.3)?;
        let spun = spare.halo(stage, A_SHARE, palette::spin());
        anims.extend(glowing(stage, &[spun], 0.3)?);
        stage.play_prepared(anims)?;
        move_all(stage, &[(a_card, slot_at(A))], 0.5)?;
        line.wait_for(stage, "B gets the turn")?;
        let turn = two.face.arc(
            &kit,
            stage,
            f64::from(A_SHARE),
            f64::from(A_SHARE + b),
            &format!("+{b}"),
            palette::share(),
        )?;
        fade_in_all(stage, &turn, 0.5)?;
        let five = two.face.halo(stage, SECRET, palette::secret());
        glow(stage, &[five], 0.4)?;
        line.wait_for(stage, "to the secret, seven")?;
        let b_card = holder_card(&kit, stage, B, &b.to_string(), palette::share())?;
        let from = stage.get_center(turn[1]);
        stage.move_to(b_card, from, ORIGIN);
        fade_in_all(stage, &[b_card], 0.2)?;
        move_all(stage, &[(b_card, slot_at(B))], 0.6)?;

        // 03_4: together they open it; what does B's share say now?
        let line = say(&kit, stage, "03_4")?;
        let mut dealer = spare.parts().to_vec();
        dealer.extend([title, dealer_secret]);
        let anims = leaving(stage, &dealer, ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "turn nine on by seven")?;
        two.face.turn(stage, i64::from(A_SHARE), 0.5)?;
        let a_halo = slot_halo(stage, A)?;
        glow(stage, &[a_halo], 0.3)?;
        let each = click_time(line.at("reach five")? - now(stage), b);
        let landed = two.face.click(stage, i64::from(b), each, None)?;
        if landed.last() != Some(&SECRET) {
            return Err(fail(format!("A's {A_SHARE} turned on by {b} landed on {landed:?}")));
        }
        line.wait_for(stage, "the door opens")?;
        recolor_all(stage, &[(two.door, palette::open())], 0.3)?;
        pause(stage, 0.3)?;
        // shut again, the arc gone and the needle on to the top, in one
        // move, so the secret is not left showing for the question
        let to_top = next_top(two.face.at(stage), dial.marks());
        let mut anims = leaving(stage, &turn, ORIGIN, 0.4)?;
        anims.push(stage.prepare(recolor(two.door, palette::ghost(), 0.4)?)?);
        anims.push(stage.prepare(tween(two.face.needle, to_top as f64, 0.4)?)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "What does B's seven say")?;
        let a_back = card_back(stage, A)?;
        swap(stage, &[a_card], &[a_back], 0.4)?;
        let b_halo = slot_halo(stage, B)?;
        glow(stage, &[b_halo], 0.5)?;
        line.wait_end(stage)?;
        let q = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 03_5: nothing: every mark A could hold makes its own secret
        let line = say(&kit, stage, "03_5")?;
        let anims = leaving(stage, &[q], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        let rows = if_a_held(dial, b);
        for s in dial.all() {
            let ways = rows.iter().filter(|&&(_, t)| t == s).count();
            if ways != 1 {
                return Err(fail(format!("B's {b} meets the secret {s} {ways} times")));
            }
        }
        let (mut texts, mut dots) = (Vec::with_capacity(rows.len()), Vec::with_capacity(rows.len()));
        for (i, &(a, s)) in rows.iter().enumerate() {
            texts.push(kit.text(
                stage,
                &row_text(a, s, i == 0),
                ROW_SIZE,
                Palette::ink(),
                v(MID_X, ROW_Y0 - ROW_DY * i as f64),
            )?);
            dots.push(disc(stage, palette::share(), v(tick_at(s)[0], DOT_Y), DOT_R));
        }
        line.wait_for(stage, "If A had zero")?;
        fade_in_all(stage, &[texts[0], dots[0]], 0.4)?;
        line.wait_for(stage, "If A had one")?;
        fade_in_all(stage, &[texts[1], dots[1]], 0.4)?;
        line.wait_for(stage, "Go through all eleven marks")?;
        let mut rest = Vec::with_capacity(rows.len() - 2);
        for (&t, &d) in texts.iter().zip(&dots).skip(2) {
            rest.push(bundle(stage, &[t, d])?);
        }
        fade_in_lagged(stage, &rest, 2.6, 0.3)?;
        line.wait_for(stage, "once each")?;
        // B's view, from the share on B's card, is what the table found
        let found = dial_secrets(dial, b);
        strip.show(&kit, stage, &dial_view(b), FOR_B, 0.5)?;
        check_strip(&strip, &found, FOR_B)?;
        swell_all(stage, &dots, palette::share(), 0.6)?;
        line.wait_end(stage)?;
        // the full strip stays a moment (a named hold)
        pause(stage, 1.0)?;

        // 03_6: a fair spin; no floor and no ceiling on a circle
        let line = say(&kit, stage, "03_6")?;
        line.wait_for(stage, "A's mark came from a fair spin")?;
        let mut table = texts[..2].to_vec();
        table.extend(&dots[..2]);
        table.extend(&rest);
        let anims = leaving(stage, &table, ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        let a_halo = slot_halo(stage, A)?;
        glow(stage, &[a_halo], 0.4)?;
        line.wait_for(stage, "fits every secret equally well")?;
        let exact = chance_card(dial, b);
        let note = precisely(&kit, stage, &exact)?;
        fade_in_all(stage, &[note], 0.5)?;
        let read = read_by(stage, &exact);
        line.wait_for(stage, "A circle has no floor")?;
        // the same B holding a cut piece instead: its line, column for
        // column under the dial's strip, from the same kind of search
        let piece = ORDINARY_PIECES[1];
        let left = ordinary_view(piece).possible(dial);
        if left != ordinary_secrets(dial, piece) {
            return Err(fail(format!("the cut piece {piece} leaves {left:?}")));
        }
        let mut cut = Vec::with_capacity(dial.marks() as usize + 3);
        for k in dial.all() {
            let c = if left.contains(&k) {
                palette::secret()
            } else {
                palette::ghost()
            };
            cut.push(panel(stage, c, 0.9, v(tick_at(k)[0], MINI_Y), TICK_W, MINI_H)?);
        }
        let floor_x = tick_at(piece)[0] - STRIP_DX / 2.0;
        let bar = stroke(
            stage,
            vec![v(floor_x, MINI_Y - 0.3), v(floor_x, MINI_Y + 0.3)],
            palette::leak(),
            5.0,
        );
        cut.push(bar);
        cut.push(kit.text(stage, FLOOR, 18.0, palette::leak(), v(floor_x, MINI_Y - MINI_LABEL_DY))?);
        cut.push(kit.text_left(
            stage,
            &for_b_cut(piece, left.len()),
            22.0,
            Palette::dim(),
            v(STRIP_LEFT, MINI_Y - MINI_CAPTION_DY),
        )?);
        let mut with = Vec::with_capacity(cut.len());
        for &m in &cut {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            with.push(stage.prepare(a.rt(0.6))?);
        }
        strip.show_with(&kit, stage, &dial_view(b), &for_b_on_the_dial(b), 0.6, with)?;
        check_strip(&strip, &found, "for B on the dial")?;
        line.wait_for(stage, "above or below anything")?;
        swell_all(stage, &[bar], palette::leak(), 0.5)?;
        wait_until(stage, read)?;
        line.wait_end(stage)?;

        // 03_7: the leak is fixed, but both are still needed
        let line = say(&kit, stage, "03_7")?;
        line.wait_for(stage, "That fixes the leak")?;
        let mut going = cut;
        going.push(note);
        let anims = leaving(stage, &going, ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        let fixed = kit.text_left(stage, LEAKS_NOTHING, TAG_SIZE, palette::open(), v(STRIP_LEFT, LEAK_Y))?;
        fade_in_all(stage, &[fixed], 0.4)?;
        line.wait_for(stage, "still needs both trustees")?;
        let both = kit.text(stage, STILL_BOTH, TAG_SIZE, palette::leak(), v(MID_X, BOTH_Y))?;
        fade_in_all(stage, &[both], 0.4)?;
        two.pulse(stage, 0.4)?;
        line.wait_end(stage)?;
        // A and B go; the five trustees' vault returns, for anyone outside
        let mut going = two.pieces();
        going.extend(two.face.parts());
        going.extend([a_back, b_card, both, fixed]);
        let with = leaving(stage, &going, ORIGIN, 0.6)?;
        strip.show_with(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.6, with)?;
        let vault = Vault::build(&kit, stage, dial)?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        vault.show(stage, 0.5)?;
        rail.reveal(stage, 0.6)?;
        let threshold = kit.text(
            stage,
            &threshold_card(THRESHOLD),
            THRESHOLD_SIZE,
            palette::trio(),
            v(THRESHOLD_X, THRESHOLD_Y),
        )?;
        fade_in_all(stage, &[threshold], 0.4)?;
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

    use super::super::draw::{STRIP_CAPTION_Y, STRIP_TITLE_Y, THINK_R, strip_right};
    use super::super::two_keys::{SMALL_W, SMALL_X};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, csv};

    const SCRIPT: &str = crate::SCRIPT;

    /// Every phrase the scene waits for, or times its clicks by, is in its
    /// line of the draft script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "From nine"),
            ("03_1", "turn seven marks forward"),
            ("03_1", "you stop at five"),
            ("03_1", "stop at five"),
            ("03_2", "nine plus seven is five"),
            ("03_2", "Ten plus one is zero"),
            ("03_2", "turning back"),
            ("03_2", "is nine"),
            ("03_3", "share the secret on the dial instead"),
            ("03_3", "The dealer"),
            ("03_3", "spins a spare dial fairly"),
            ("03_3", "stops, nine"),
            ("03_3", "B gets the turn"),
            ("03_3", "to the secret, seven"),
            ("03_4", "turn nine on by seven"),
            ("03_4", "reach five"),
            ("03_4", "the door opens"),
            ("03_4", "What does B's seven say"),
            ("03_5", "If A had zero"),
            ("03_5", "If A had one"),
            ("03_5", "Go through all eleven marks"),
            ("03_5", "once each"),
            ("03_6", "A's mark came from a fair spin"),
            ("03_6", "fits every secret equally well"),
            ("03_6", "A circle has no floor"),
            ("03_6", "above or below anything"),
            ("03_7", "That fixes the leak"),
            ("03_7", "still needs both trustees"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        // "is nine" is the line's last words, not part of an earlier sum
        let text = script.text("03_2").unwrap();
        assert_eq!(text.matches("is nine").count(), 1);
    }

    /// What the scene draws is the facts': A's 9, B's 7, seven clicks from
    /// 9 passing the top 10 and 0 to 5, 16 unwrapping to 5, ten plus one
    /// landing on 0, five back by seven landing on 9, the table's eleven
    /// secrets once each, and the chance 1/11.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let b = dial.sub(SECRET, A_SHARE);
        assert_eq!(f.get("S-F2.a_share"), Some(A_SHARE.to_string().as_str()));
        assert_eq!(f.get("S-F2.b_share"), Some(b.to_string().as_str()));
        assert_eq!(f.get("S-F2.top"), Some((MARKS - 1).to_string().as_str()));
        assert_eq!(f.get("S-F2.sum_unwrapped"), Some((A_SHARE + b).to_string().as_str()));
        assert_eq!(f.get("S-F2.sum"), Some(dial.add(A_SHARE, b).to_string().as_str()));
        assert_eq!(
            f.get("S-F2.ten_plus_one"),
            Some(dial.add(MARKS - 1, 1).to_string().as_str())
        );
        assert_eq!(f.get("S-F2.turn_back"), Some(dial.sub(SECRET, b).to_string().as_str()));
        let secrets: Vec<u32> = if_a_held(dial, b).iter().map(|&(_, s)| s).collect();
        assert_eq!(f.get("S-F2.secret_if_a_held"), Some(csv(&secrets).as_str()));
        assert_eq!(
            f.get("S-F2.distinct_secrets"),
            Some(dial_secrets(dial, b).len().to_string().as_str())
        );
        assert_eq!(f.get("S-F2.ways_each"), Some("1"));
        assert_eq!(
            f.get("S-F2.chance_each"),
            Some(Ratio::new(1, i64::from(MARKS)).to_string().as_str())
        );
        assert_eq!(sum_tex(dial, A_SHARE, b), r"9 + 7 = 16 \to 5");
        assert_eq!(unwrap_tex(dial, A_SHARE, b).as_deref(), Some("16 - 11 = 5"));
        assert_eq!(unwrap_tex(dial, 2, 3), None);
        assert_eq!(top_tex(dial), r"10 + 1 \to 0");
        assert_eq!(back_tex(dial, SECRET, b), r"5 - 7 \to 9");
        assert_eq!(row_text(0, 7, true), "A had 0 → secret 7");
        assert_eq!(row_text(1, 8, false), "A had 1 → 8");
        assert_eq!(
            chance_card(dial, b),
            "for every secret s, exactly one of A's 11 equally likely marks gives B a 7: P(B = 7 | s) = 1/11"
        );
        // the cut piece of 03_6 is chapter 02's B
        let piece = ORDINARY_PIECES[1];
        assert_eq!(
            for_b_cut(piece, ordinary_secrets(dial, piece).len()),
            "for: B, holding a cut piece 3: 8 possible"
        );
        assert_eq!(threshold_card(THRESHOLD), "threshold 3");
    }

    /// Seven clicks fit the room the words leave, never quicker than
    /// CLICK_FAST or slower than CLICK_SLOW.
    #[test]
    fn the_clicks_fit_their_words() {
        assert!((click_time(2.8, 7) - 0.4).abs() < 1e-12);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(near(click_time(0.5, 7), CLICK_FAST));
        assert!(near(click_time(-1.0, 7), CLICK_FAST));
        assert!(near(click_time(9.0, 7), CLICK_SLOW));
        assert!(near(click_time(1.0, 0), CLICK_SLOW));
    }

    /// The equations sit right of the enlarged dial and inside the frame,
    /// the table in the middle column, the captions and tags inside the
    /// frame, and the cut piece's line between the strip's caption and the
    /// lower-third band.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const BAND_TOP: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        let dial = Dial::vault();
        let b = dial.sub(SECRET, A_SHARE);
        // generously 0.012 units a point per TeX glyph, spaces dropped
        for (tex, size) in [
            (sum_tex(dial, A_SHARE, b), TEX_SIZE),
            (top_tex(dial), TEX_SIZE),
            (back_tex(dial, SECRET, b), TEX_SIZE),
        ] {
            let glyphs = tex.replace(r"\to", "→").replace(' ', "").chars().count() as f64;
            let w = 0.012 * size * glyphs;
            assert!(
                TEX_X - w / 2.0 > BIG_X + super::super::two_keys::ARC_IN * BIG_R + 0.3,
                "{tex}"
            );
            assert!(TEX_X + w / 2.0 < FRAME_X_RADIUS - 0.25, "{tex}");
        }
        for (caption, size) in [
            (FOR_OUTSIDE.to_owned(), 22.0),
            (FOR_B.to_owned(), 22.0),
            (for_b_on_the_dial(b), 22.0),
            (for_b_cut(ORDINARY_PIECES[1], 8), 22.0),
            (LEAKS_NOTHING.to_owned(), TAG_SIZE),
        ] {
            let right = STRIP_LEFT + 0.0055 * size * caption.chars().count() as f64;
            assert!(right < FRAME_X_RADIUS - 0.25, "{caption}");
        }
        let right = STRIP_LEFT + 0.12 * FOR_B.chars().count() as f64;
        assert!(right < THINK_X - THINK_R - 0.2);
        let row_w = 0.0055 * ROW_SIZE * row_text(10, 10, true).chars().count() as f64;
        assert!(MID_X - row_w / 2.0 > SMALL_X + SMALL_W / 2.0 + 0.1);
        assert!(MID_X + row_w / 2.0 < STRIP_LEFT - 0.1);
        let both_w = 0.0055 * TAG_SIZE * STILL_BOTH.chars().count() as f64;
        assert!(MID_X - both_w / 2.0 > SMALL_X + SMALL_W / 2.0 + 0.05);
        assert!(MID_X + both_w / 2.0 < STRIP_LEFT - 0.05);
        let threshold_w = 0.18 * threshold_card(THRESHOLD).chars().count() as f64;
        assert!(THRESHOLD_X + threshold_w / 2.0 < strip_right(MARKS));
        const {
            assert!(
                ROW_Y0 + 0.2 < FRAME_Y_RADIUS - 0.25,
                "the table's top row inside the frame"
            );
            assert!(
                ROW_Y0 - ROW_DY * 10.0 - 0.2 > BAND_TOP,
                "the table's last row over the band"
            );
            assert!(DOT_Y + DOT_R < STRIP_TITLE_Y - 0.2, "the dots under the strip's title");
            assert!(
                MINI_Y + MINI_H / 2.0 < STRIP_CAPTION_Y - 0.25,
                "the cut line under the caption"
            );
            assert!(
                MINI_Y - MINI_CAPTION_DY - 0.2 > BAND_TOP,
                "the cut line's caption over the lower-third band"
            );
            assert!(
                COUNT_Y - 0.25 > -FRAME_Y_RADIUS + 0.2,
                "the click count inside the frame"
            );
            assert!(
                LEAK_Y + 0.2 < STRIP_CAPTION_Y - 0.2,
                "the closing tag under the caption"
            );
            assert!(THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2, "the ring under the caption");
        }
    }
}
