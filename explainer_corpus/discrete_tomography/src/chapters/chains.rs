//! Chapter 09, chains of switches: one worry remains: two 3 by 3 pictures
//! with the same shadows, one square in every row and column (09_1, a
//! prediction: can a single switch turn one into the other?); no: they
//! differ in six squares, and a switch changes only four (09_2); two
//! switches do it (09_3); any two pictures with the same shadows are joined
//! by a chain of switches (09_4, Ryser's card); that is the step the tree
//! needed: a chain to another picture would start with a switch of the tree
//! (09_5), and none can start, so no other picture fits (09_6); the same
//! chains tie the face's sixty-four together, and trying every switch, and
//! every switch of what it finds, reaches them all (09_7); the water answers
//! whether any picture fits, a switch in its picture whether it is the only
//! one (09_8).
//!
//! The 3 by 3 pictures are `facts::perms`, their chain `model::walk`; the
//! tree's uniqueness `model::fits`; the 64 and their rings `facts::gallery`
//! and `model::reachable` from chapter 03's water (`facts::face_flow`).

use std::collections::HashMap;

use explainer_kit::props::{lower_third, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, pause, say, segment, swap, think, thumb,
    wait_until,
};
use super::only::{ALONE_Y, ANSWERS, CLOSING_Y, alone_text, chain_claim};
use super::same_shadows::{QUESTIONS, THUMB_CELL, counts, gallery_at, place, run_switch};
use super::stops::read_s;
use super::water::{AT, CELL, PROGRAMMERS, TEXT_X, tap_at};
use crate::facts::{self, RYSER_YEAR};
use crate::model::{self, Picture};
use crate::palette;

pub struct ChainsOfSwitches {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const QUESTION_MARK: &str = "?";
const WATERS_PICTURE: &str = "the water's picture";

/// The two 3 by 3 pictures (09_1 to 09_3): their centres and pitch, and the
/// words right of them.
const LEFT3: [f64; 2] = [-4.6, 0.6];
const RIGHT3: [f64; 2] = [-1.0, 0.6];
const CELL3: f64 = 0.8;
const WORDS_X: f64 = 1.6;
const WORDS_Y: [f64; 2] = [1.6, 1.0];
/// The think ring's centre (09_1).
const THINK_AT: [f64; 2] = [4.3, 0.6];
/// The "?" picture on the right (09_5), and the chain's arrows between it
/// and the tree.
const UNKNOWN_AT: [f64; 2] = [2.6, 0.45];
const UNKNOWN_CELL: f64 = 0.4;
const LINKS: usize = 3;
const LINK_GAP: f64 = 0.12;

fn changes_text(squares: usize) -> String {
    format!("a switch changes {squares} squares")
}

fn links_text(links: usize) -> String {
    format!("{links} switches")
}

/// The "for programmers" lower-third (09_7): listing every picture that
/// fits by switches; Ryser's theorem is why it reaches them all.
fn listing_text() -> String {
    format!(
        "list every picture that fits: start from one, apply every switch, keep each new picture, repeat \
         until nothing is new (Ryser, {RYSER_YEAR}: this reaches them all)"
    )
}

/// A small picture drawn: its grid's lines, its squares by place, and its
/// counts.
type Drawn = (Vec<Mob>, HashMap<(usize, usize), Mob>, Vec<Mob>);

/// A 3 by 3 picture's board, its squares and its counts in green.
/// Detached.
fn three_by_three(kit: &Kit, stage: &mut Stage<'_>, board: &Board, p: &Picture) -> fmn::Result<Drawn> {
    let lines = board.lines(stage);
    let squares = place(stage, board, p)?;
    let mut shadow = counts(kit, stage, &p.row_counts(), palette::matches(), |r| board.row_count_at(r))?;
    shadow.extend(counts(kit, stage, &p.col_counts(), palette::matches(), |c| {
        board.col_count_at(c)
    })?);
    Ok((lines, squares, shadow))
}

impl SceneConstruct for ChainsOfSwitches {
    fn name(&self) -> &str {
        "09_chains_of_switches"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (left, right) = facts::perms().map_err(fail)?;
        if left.shadows() != right.shadows() || left.row_counts().iter().chain(&left.col_counts()).any(|&n| n != 1) {
            return Err(fail("chapter 09's pictures do not both have one square in every row and column".into()));
        }
        let one_switch = model::switches(&left).iter().any(|s| s.apply(&left).as_ref() == Some(&right));
        let chain = model::walk(&left, &right).ok_or_else(|| fail("no chain joins chapter 09's pictures".into()))?;
        let differ = left.differences(&right);
        let opening_squares = model::OPENING.corners().len();
        if one_switch || chain.len() != 2 || differ.len() <= opening_squares {
            return Err(fail(format!(
                "chapter 09's pictures are {} squares and {} switches apart",
                differ.len(),
                chain.len()
            )));
        }
        let tree = model::tree();
        let tree_shadows = tree.shadows();
        if !model::switches(&tree).is_empty() || model::fits(&tree_shadows).len() != 1 {
            return Err(fail("the tree has a switch or is not alone with its shadows".into()));
        }
        let (_, face_states) = facts::face_flow().map_err(fail)?;
        let built = face_states
            .last()
            .ok_or_else(|| fail("the face's flow sends nothing".into()))?
            .wet
            .clone();
        let gallery = facts::gallery();
        let from_built = model::reachable(&built);
        if from_built.len() != gallery.len() {
            return Err(fail("switches from the water's picture do not reach every one of the 64".into()));
        }
        let left_board = Board::new(LEFT3, CELL3, left.rows(), left.cols());
        let right_board = Board::new(RIGHT3, CELL3, right.rows(), right.cols());

        // 09_1: two 3 by 3 pictures with the same shadows; the question
        let _asked = say(&kit, stage, "09_1")?;
        let (left_lines, mut left_squares, left_shadow) = three_by_three(&kit, stage, &left_board, &left)?;
        let (right_lines, right_squares, right_shadow) = three_by_three(&kit, stage, &right_board, &right)?;
        let mut arriving = left_lines.clone();
        arriving.extend(left_squares.values().copied());
        arriving.extend(&left_shadow);
        arriving.extend(&right_lines);
        arriving.extend(right_squares.values().copied());
        arriving.extend(&right_shadow);
        fade_in_all(stage, &arriving, 0.6)?;
        let q = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 09_2: no: six squares apart, and a switch changes four
        let line = say(&kit, stage, "09_2")?;
        fade_out_all(stage, &[q], 0.3)?;
        line.wait_for(stage, "six squares")?;
        let mut rings = Vec::with_capacity(2 * differ.len());
        for &(r, c) in &differ {
            rings.push(left_board.ring(stage, r, c, palette::switch()));
            rings.push(right_board.ring(stage, r, c, palette::switch()));
        }
        fade_in_all(stage, &rings, 0.4)?;
        line.wait_for(stage, "only four")?;
        let changes = kit.text_left(
            stage,
            &changes_text(opening_squares),
            24.0,
            Palette::ink(),
            v(WORDS_X, WORDS_Y[0]),
        )?;
        fade_in_all(stage, &[changes], 0.3)?;

        // 09_3: two switches do it, the model's own shortest chain, run on
        // the left-hand picture
        let line = say(&kit, stage, "09_3")?;
        fade_out_all(stage, &rings, 0.3)?;
        let mut at = left.clone();
        for (step, phrase) in chain.iter().zip(["First this rectangle", "then this one"]) {
            line.wait_for(stage, phrase)?;
            let rect = left_board.rectangle(stage, *step, palette::switch());
            fade_in_all(stage, &[rect], 0.2)?;
            flash_all(stage, &[rect], 0.4)?;
            at = run_switch(stage, &left_board, &mut left_squares, &at, *step, 0.5)?;
            fade_out_all(stage, &[rect], 0.2)?;
        }
        if at != right {
            return Err(fail(format!("the chain ended at {}, not the right-hand picture", at.pattern())));
        }
        let two = kit.text_left(
            stage,
            &links_text(chain.len()),
            24.0,
            palette::switch(),
            v(WORDS_X, WORDS_Y[1]),
        )?;
        fade_in_all(stage, &[two], 0.3)?;
        line.wait_to_end(stage)?;
        pause(stage, 0.6)?;

        // 09_4: Ryser's theorem, scoped, held until it can be read
        let line = say(&kit, stage, "09_4")?;
        let claim = chain_claim();
        let card = precisely(&kit, stage, &claim)?;
        fade_in_all(stage, &[card], 0.4)?;
        let card_up = stage.scene().time().to_f64();
        line.wait_to_end(stage)?;

        // 09_5: the step the tree needed: a chain from the tree to another
        // picture would start with a switch of the tree
        let line = say(&kit, stage, "09_5")?;
        let mut leaving = left_lines;
        leaving.extend(left_squares.into_values());
        leaving.extend(left_shadow);
        leaving.extend(right_lines);
        leaving.extend(right_squares.into_values());
        leaving.extend(right_shadow);
        leaving.extend([changes, two]);
        fade_out_all(stage, &leaving, 0.4)?;
        let board = Board::new(AT, CELL, tree.rows(), tree.cols());
        let mut tree_mobs = board.lines(stage);
        tree_mobs.extend(place(stage, &board, &tree)?.into_values());
        tree_mobs.extend(counts(&kit, stage, &tree_shadows.rows, Palette::ink(), |r| tap_at(&board, r))?);
        tree_mobs.extend(counts(&kit, stage, &tree_shadows.cols, Palette::ink(), |c| {
            board.col_count_at(c)
        })?);
        let unknown = Board::new(UNKNOWN_AT, UNKNOWN_CELL, tree.rows(), tree.cols());
        let mut unknown_mobs = unknown.lines(stage);
        unknown_mobs.push(kit.text(stage, QUESTION_MARK, 60.0, Palette::dim(), unknown.centre)?);
        fade_in_all(stage, &tree_mobs, 0.5)?;
        fade_in_all(stage, &unknown_mobs, 0.4)?;
        line.wait_for(stage, "a chain of switches")?;
        let (from, to) = (board.right() + LINK_GAP, unknown.left() - LINK_GAP);
        let step = (to - from) / LINKS as f64;
        let y = board.centre[1];
        let links: Vec<Mob> = (0..LINKS)
            .map(|i| {
                let color = if i == 0 { palette::switch() } else { Palette::faint() };
                let x0 = from + step * i as f64 + LINK_GAP / 2.0;
                segment(stage, v(x0, y), v(x0 + step - LINK_GAP, y), color, 4.0)
            })
            .collect();
        fade_in_lagged(stage, &links, 0.9, 0.3)?;

        // 09_6: from the tree no chain can start: one picture fits
        let line = say(&kit, stage, "09_6")?;
        let x0 = from + LINK_GAP / 2.0;
        let red = segment(stage, v(x0, y), v(x0 + step - LINK_GAP, y), palette::impossible(), 4.0);
        swap(stage, &links[..1], &[red], 0.3)?;
        line.wait_for(stage, "no other picture fits")?;
        let mut going = links[1..].to_vec();
        going.push(red);
        going.extend(&unknown_mobs);
        fade_out_all(stage, &going, 0.4)?;
        let alone = kit.text_left(
            stage,
            &alone_text(model::fits(&tree_shadows).len()),
            26.0,
            palette::matches(),
            v(TEXT_X, ALONE_Y),
        )?;
        fade_in_all(stage, &[alone], 0.4)?;
        wait_until(stage, card_up + read_s(&claim))?;

        // 09_7: the face's 64, in their waves from the face; the water's
        // picture framed, the 64 lighting out from it switch by switch; the
        // listing for programmers
        let line = say(&kit, stage, "09_7")?;
        let mut going = tree_mobs;
        going.extend([alone, card]);
        fade_out_all(stage, &going, 0.4)?;
        let n = gallery.len();
        let mut thumbs: Vec<Vec<Mob>> = Vec::with_capacity(n);
        let mut index = HashMap::with_capacity(n);
        for (k, (p, d)) in gallery.iter().enumerate() {
            let frame = if *d == 0 { Palette::ink() } else { Palette::faint() };
            thumbs.push(thumb(stage, p, gallery_at(k, n), THUMB_CELL, frame)?);
            index.insert(p.clone(), k);
        }
        let far = gallery.iter().map(|(_, d)| *d).max().unwrap_or(0);
        line.wait_for(stage, "the face's sixty-four")?;
        for d in 0..=far {
            let wave: Vec<Mob> = thumbs
                .iter()
                .zip(&gallery)
                .filter(|(_, (_, e))| *e == d)
                .flat_map(|(t, _)| t.iter().copied())
                .collect();
            fade_in_lagged(stage, &wave, 0.5, 0.2)?;
        }
        let big = Board::new(AT, CELL, built.rows(), built.cols());
        let mut water_mobs = big.lines(stage);
        water_mobs.extend(place(stage, &big, &built)?.into_values());
        water_mobs.push(kit.text(
            stage,
            WATERS_PICTURE,
            24.0,
            Palette::dim(),
            v(big.centre[0], big.top() + 0.4),
        )?);
        fade_in_all(stage, &water_mobs, 0.4)?;
        line.wait_for(stage, "Start from any picture")?;
        let k_built = *index
            .get(&built)
            .ok_or_else(|| fail("the water's picture is not in the gallery".into()))?;
        let framed = thumb(stage, &built, gallery_at(k_built, n), THUMB_CELL, palette::matches())?;
        swap(stage, &thumbs[k_built], &framed, 0.3)?;
        thumbs[k_built] = framed;
        let listing = listing_text();
        let code = lower_third(&kit, stage, PROGRAMMERS, &listing)?;
        fade_in_all(stage, &[code], 0.4)?;
        let code_up = stage.scene().time().to_f64();
        let built_far = from_built.iter().map(|(_, d)| *d).max().unwrap_or(0);
        for d in 0..=built_far {
            let ring: Vec<Mob> = from_built
                .iter()
                .filter(|(_, e)| *e == d)
                .filter_map(|(p, _)| index.get(p))
                .flat_map(|&k| thumbs[k].iter().copied())
                .collect();
            flash_all(stage, &ring, 0.5)?;
        }
        line.wait_to_end(stage)?;
        wait_until(stage, code_up + read_s(&listing))?;

        // 09_8: chapter 01's two questions, each with its answer as it is
        // spoken; held until they can be read, then the fade
        let line = say(&kit, stage, "09_8")?;
        let mut going: Vec<Mob> = thumbs.into_iter().flatten().collect();
        going.extend(water_mobs);
        going.push(code);
        fade_out_all(stage, &going, 0.4)?;
        let mut closing = Vec::with_capacity(4);
        for (i, (question, answer)) in QUESTIONS.iter().zip(ANSWERS).enumerate() {
            closing.push(kit.text_left(stage, question, 28.0, Palette::ink(), v(TEXT_X, CLOSING_Y[2 * i]))?);
            closing.push(kit.text_left(stage, answer, 22.0, Palette::ink(), v(TEXT_X, CLOSING_Y[2 * i + 1]))?);
        }
        line.wait_for(stage, "the water answers")?;
        fade_in_all(stage, &closing[..2], 0.4)?;
        let shown = stage.scene().time().to_f64();
        line.wait_for(stage, "A switch in")?;
        fade_in_all(stage, &closing[2..], 0.4)?;
        line.wait_to_end(stage)?;
        let all_words = [QUESTIONS[0], ANSWERS[0], QUESTIONS[1], ANSWERS[1]].join(" ");
        wait_until(stage, shown + read_s(&all_words))?;
        pause(stage, 1.2)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use explainer_kit::animatic::wrap;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("09_2", "six squares"),
            ("09_2", "only four"),
            ("09_3", "First this rectangle"),
            ("09_3", "then this one"),
            ("09_5", "a chain of switches"),
            ("09_6", "no other picture fits"),
            ("09_7", "the face's sixty-four"),
            ("09_7", "Start from any picture"),
            ("09_8", "the water answers"),
            ("09_8", "A switch in"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The words on screen are the facts'; the listing card keeps to two
    /// whole lines at every width the band may narrow to and is held under
    /// twelve seconds.
    #[test]
    fn the_words_on_screen_are_built_from_the_facts() {
        let f = facts::facts().unwrap();
        let get = |key: &str| -> usize { f.get(key).unwrap().parse().unwrap() };
        assert_eq!(changes_text(get("DT-F2.differ")), "a switch changes 4 squares");
        assert_eq!(links_text(get("DT-F17.perm.distance")), "2 switches");
        assert_eq!(f.get("DT-F17.perm.differ"), Some("6"));
        let listing = listing_text();
        assert!(listing.contains("Ryser, 1957"), "{listing}");
        assert!(read_s(&listing) < 12.0, "{}", read_s(&listing));
        for width in (80..=92).step_by(4) {
            let lines = wrap(&listing, width, 3);
            assert!(
                lines.len() <= 2 && !lines.iter().any(|l| l.ends_with('…')),
                "{width}: {lines:?}"
            );
        }
    }

    /// The chapter's layout keeps its pieces apart and inside the
    /// title-safe area (about 0.0045 units a point a character).
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let left = Board::new(LEFT3, CELL3, 3, 3);
        let right = Board::new(RIGHT3, CELL3, 3, 3);
        let board = Board::new(AT, CELL, 5, 5);
        let unknown = Board::new(UNKNOWN_AT, UNKNOWN_CELL, 5, 5);
        let safe_x = FRAME_X_RADIUS - 0.25;
        let safe_y = FRAME_Y_RADIUS - 0.25;
        let ends = |x: f64, text: &str, size: f64| x + text.chars().count() as f64 * size * 0.0045;
        // the 3 by 3 pictures: inside, apart, their counts clear of each
        // other and of the words
        assert!(left.left() > -safe_x);
        assert!(left.row_count_at(0)[0] + 0.3 < right.left());
        assert!(right.row_count_at(0)[0] + 0.3 < WORDS_X);
        assert!(ends(WORDS_X, &changes_text(4), 24.0) < safe_x);
        assert!(left.top() + 0.2 < safe_y);
        // the tree, the chain and the "?" picture apart and inside
        assert!(board.right() + 3.0 * LINK_GAP < unknown.left());
        assert!(unknown.right() < safe_x);
        // the lowest words stand over a two-line card
        let card_top = -FRAME_Y_RADIUS + 0.2 + 0.3 + 2.0 * 0.36;
        assert!(left.col_count_at(0)[1] - 0.2 > card_top);
        assert!(ALONE_Y - 0.15 > card_top);
        assert!(THINK_AT[0] + 1.2 < safe_x);
    }
}
