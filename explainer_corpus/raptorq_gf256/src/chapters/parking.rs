//! Chapter 16, parking a piece: the stuck graph of 14_5 parks C as the
//! unknown c (16_1); chapter 03's cards solve the rest in terms of c, and the
//! two c's in D cancel (16_2); the leftover card pins c down, and every piece
//! follows (16_3); the ring of five with a byte card in place of its fifth
//! card (16_4); and A ⊕ 2A = 3A, so 3A = 15 and A = 5 (16_5).
//!
//! Every line is built from the peel the model runs (`decoding::peel_toy`
//! with C parked) and the ring's own cards, so what is written is what is
//! computed.

use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::library::TransformMatchingTex;
use fmn::prelude::*;

use super::density::{GRAPH_BIG, affine_tex, toy_graph};
use super::draw::{OPLUS, Role, badge, badge_shape, card, card_with_amounts, group_of};
use super::props::RingProp;
use super::voice::{Spoken, board};
use crate::decoding::{BIG, PeelStep, TOY_CARDS, peel_toy};
use crate::model::{A, B, C, D, FOLD, PIECE_NAMES, PIECES, RING, inv, mul, result};

const KEY: &str = "16_parking_a_piece";

/// Dividing `total` by `by` on the byte clock (11's clicks): their logs,
/// and the clicks of the quotient, log total − log by (mod 255).
pub fn division_clicks(total: u8, by: u8) -> [u8; 3] {
    let log = crate::model::logs();
    let (lt, lb) = (log[usize::from(total)], log[usize::from(by)]);
    [lt, lb, ((u16::from(lt) + 255 - u16::from(lb)) % 255) as u8]
}

/// The parked piece (C) and the colour its unknown c is written in.
const PARKED: usize = 2;
fn ghost() -> Srgb {
    Palette::dim()
}

/// An animation of a card's edge (its stroke only) to colour `c`.
fn edge(m: Mob, c: Srgb, rt: f64) -> fmn::Result<AnimBuilder> {
    Ok(m.animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        })?
        .set_stroke(Some(c), None, None)?)
}

/// Pulse only the paper's edge: recolouring a whole card hides its
/// equation against its fill. Keep every symbol in its semantic colour.
fn pulse_edge(stage: &mut Stage<'_>, back: Mob, restore: Srgb, rt: f64) -> fmn::Result<()> {
    stage.play(edge(back, Palette::ink(), rt / 2.0)?)?;
    stage.play(edge(back, restore, rt / 2.0)?)?;
    Ok(())
}

/// `x ⊕ y ⊕ …` as TeX.
fn xor_tex<I: IntoIterator<Item = String>>(terms: I) -> String {
    terms.into_iter().collect::<Vec<_>>().join(&format!(r" {OPLUS} "))
}

/// The derivation line of one peel step: `P = r ⊕ X ⊕ c`, then (when it
/// used a piece found before) `= r ⊕ (X's value) ⊕ c`, then `= v` when the
/// c's cancel. `found` holds each earlier piece's written value.
pub fn step_lines(step: &PeelStep, found: &[Option<String>; 5]) -> (String, Option<String>, Option<String>) {
    let card = TOY_CARDS[step.card];
    let r = result(card).to_string();
    let others: Vec<usize> = (0..5).filter(|&p| card >> p & 1 == 1 && p != step.piece).collect();
    let name = |p: usize| {
        if p == PARKED {
            "c".to_owned()
        } else {
            PIECE_NAMES[p].to_owned()
        }
    };
    let first = format!(
        "{} = {}",
        PIECE_NAMES[step.piece],
        xor_tex(std::iter::once(r.clone()).chain(others.iter().map(|&p| name(p))))
    );
    let known: Vec<usize> = others.iter().copied().filter(|&p| p != PARKED).collect();
    if known.is_empty() {
        return (first, None, None);
    }
    let n = others.len();
    let expanded = xor_tex(std::iter::once(r).chain(others.iter().enumerate().map(|(i, &p)| {
        let value = found[p].clone().unwrap_or_else(|| name(p));
        // a sum in the middle of a sum gets brackets, to show where it went
        if p != PARKED && value.contains(OPLUS) && i + 1 < n {
            format!("({value})")
        } else if p == PARKED {
            "c".to_owned()
        } else {
            value
        }
    })));
    let last = (!step.value.with_c).then(|| step.value.constant.to_string());
    (first, Some(expanded), last)
}

pub struct Parking {
    pub kit: Kit,
}

impl SceneConstruct for Parking {
    fn name(&self) -> &str {
        KEY
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let board = board(KEY)?;
        self.park(&kit, stage, &board)?;
        self.ring(&kit, stage, &board)?;
        // a short breath: with michael's review-2 reads the chapter ran 86.7 s
        // (Codex B review 2, P2); 0.8 s keeps it at 85
        kit.music_breath(stage, 0.8)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl Parking {
    /// 16_1–16_3: park C, solve in terms of c, and pin c down.
    fn park(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        // 16_1: the frozen graph of 14_5 (the first card lost)
        let cards: Vec<usize> = (1..TOY_CARDS.len()).collect();
        let centre = v(0.0, 0.7);
        let g = toy_graph(kit, stage, centre, GRAPH_BIG, &cards)?;
        let stuck = kit.text(
            stage,
            "no card has one unknown left",
            32.0,
            Palette::dim(),
            v(0.0, -2.2),
        )?;
        let line = Spoken::say(kit, stage, board, "16_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), g.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), stuck, ORIGIN, 1.0)?.rt(0.8),
        );
        line.at(stage, 1)?;
        let parks = lower_third(
            kit,
            stage,
            "precisely:",
            &format!(
                "RaptorQ parks {} pieces on purpose at {} pieces (search: inactivation decoding); the sender also runs \
                 one solve, to send the originals unchanged first",
                BIG.p(),
                explainer_kit::thousands(BIG.k as u64)
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), parks, v(0.0, 0.2), 1.0)?.rt(0.6));
        line.at(stage, 2)?;
        // C becomes the ghost c: carried along, not solved for
        let at = g.piece_at[PARKED];
        let r = 0.3 * GRAPH_BIG;
        let outline = badge_shape(
            PARKED,
            r,
            Style::default().fill(ghost(), 0.05).stroke(ghost(), 2.5, 0.8),
            at,
        );
        let outline = stage.arena_mut().add(outline);
        let letter = kit.tex(stage, "c", 44.0 * GRAPH_BIG, ghost(), &[], at)?;
        let ghost_c = group_of(stage, &[outline, letter])?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), g.badges[PARKED], 1.3, None)?.rt(0.8));
        line.at(stage, 3)?;
        let c_edges: Vec<Mob> = g.edges.iter().filter(|e| e.1 == PARKED).map(|e| e.2).collect();
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let swap = fmn::animation::replacement_transform(g.badges[PARKED], ghost_c);
        anims.push(stage.prepare(swap.rt(1.0))?);
        for &e in &c_edges {
            let a = e
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .set_stroke(Some(ghost()), None, Some(0.35))?;
            anims.push(stage.prepare(a)?);
        }
        let a = fade_out(stage.arena_mut(), stuck, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(anims)?;
        kit.hold(stage)?;

        // 16_2: chapter 03's cards return beside the new two; each line
        // lights the card it comes from
        let (steps, leftover) = peel_toy(&cards, Some(PARKED));
        let card_y = [2.75, 1.75, 0.75, -0.25, -1.25];
        let card_x = -4.6;
        let mut shown = Vec::new();
        for (i, &j) in cards.iter().enumerate() {
            shown.push(card(
                kit,
                stage,
                TOY_CARDS[j],
                Some(result(TOY_CARDS[j])),
                0.72,
                v(card_x, card_y[i]),
            )?);
        }
        let line = Spoken::say(kit, stage, board, "16_2")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let a = fade_out(stage.arena_mut(), g.group, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        let a = fade_out(stage.arena_mut(), ghost_c, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        let a = fade_out(stage.arena_mut(), parks, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(anims)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for k in &shown {
            let a = fade_in(stage.arena_mut(), k.group, v(0.2, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        let group = explainer_kit::lagged(stage, anims, 0.15)?;
        stage.play(group)?;
        // "three of these are our cards from the start": 03's are A ⊕ C, B ⊕ C ⊕ D, A ⊕ B
        let from_03 = [A | C, B | C | D, A | B];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &j) in cards.iter().enumerate() {
            if from_03.contains(&TOY_CARDS[j]) {
                anims.push(super::draw::swell(stage, shown[i].group, 1.12, 0.9)?);
            }
        }
        stage.play_prepared(anims)?;
        let row_of = |j: usize| cards.iter().position(|&c| c == j).expect("a shown card");
        let line_x = -1.95;
        let t2c = [("c", ghost())];
        let mut found: [Option<String>; 5] = Default::default();
        let mut lines = Vec::new();
        for (n, step) in steps.iter().enumerate() {
            // A on sentence 1, B with it, D on sentence 2, E on sentence 4
            match n {
                0 => line.at(stage, 1)?,
                2 => line.at(stage, 2)?,
                3 => line.at(stage, 3)?,
                _ => {}
            }
            let (first, expanded, last) = step_lines(step, &found);
            let y = card_y[n];
            let from = &shown[row_of(step.card)];
            let head = kit.tex(stage, &first, 46.0, Palette::ink(), &t2c, v(0.0, y))?;
            stage.move_to(head, v(line_x, y), LEFT);
            play!(stage;
                edge(from.back, Role::lit(), 0.4)?,
                write(stage.arena(), head).rt(0.8),
            );
            let mut row = vec![head];
            if let Some(exp) = &expanded {
                let right = stage.get_right(head)[0] + 0.25;
                let (e, e_spans) = kit.tex_spans(stage, &format!("= {exp}"), 46.0, Palette::ink(), &t2c, v(0.0, y))?;
                stage.move_to(e, v(right, y), LEFT);
                play!(stage; fade_in(stage.arena_mut(), e, v(0.2, 0.0), 1.0)?.rt(0.7));
                row.push(e);
                if let Some(value) = &last {
                    // the two c's cancel: the sum folds to its value
                    let (f, f_spans) =
                        kit.tex_spans(stage, &format!("= {value}"), 46.0, Role::lit(), &[], v(0.0, y))?;
                    stage.move_to(f, v(right, y), LEFT);
                    stage.wait(0.3)?;
                    let fold = TransformMatchingTex::new(stage.arena_mut(), e, f, &e_spans, &f_spans)?;
                    play!(stage; fold.rt(0.9));
                    row.pop();
                    row.push(f);
                }
            }
            found[step.piece] = Some(match (&expanded, &last) {
                (_, Some(value)) => value.clone(),
                (Some(exp), None) => exp.clone(),
                (None, None) => affine_tex(step.value),
            });
            lines.push(row);
            play!(stage; edge(from.back, Role::card(), 0.3)?);
        }
        kit.hold(stage)?;

        // 16_3: the leftover card and the cards that cancel all but c
        let left_card = leftover[0];
        let pick = cancel_to_c(&cards, left_card);
        let values: Vec<u8> = pick.iter().map(|&j| result(TOY_CARDS[j])).collect();
        let c_value = values.iter().fold(0, |a, b| a ^ b);
        let c_line = format!("c = {} = {c_value}", xor_tex(values.iter().map(u8::to_string)));
        let (cl, _) = kit.tex_spans(
            stage,
            &c_line,
            50.0,
            Palette::ink(),
            &[("c", ghost())],
            v(0.0, card_y[4]),
        )?;
        stage.move_to(cl, v(line_x, card_y[4]), LEFT);
        let line = Spoken::say(kit, stage, board, "16_3")?;
        line.at(stage, 1)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &j in &pick {
            let a = edge(shown[row_of(j)].back, Role::lit(), 0.5)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        play!(stage; write(stage.arena(), cl).rt(1.0));
        line.at(stage, 2)?;
        // back-substitution: every piece's value, c included
        let mut solved = Vec::new();
        for p in 0..5 {
            let value = if p == PARKED {
                c_value
            } else {
                let s = steps
                    .iter()
                    .find(|s| s.piece == p)
                    .expect("every other piece was peeled");
                s.value.constant ^ if s.value.with_c { c_value } else { 0 }
            };
            let x = -3.9 + 2.15 * p as f64;
            let b = badge(kit, stage, p, 0.3, v(x, -2.3))?;
            let t = kit.tex(stage, &format!("= {value}"), 44.0, Role::lit(), &[], v(x + 0.88, -2.3))?;
            solved.push(group_of(stage, &[b, t])?);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &solved {
            let a = fade_in(stage.arena_mut(), m, v(0.0, 0.2), 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let group = explainer_kit::lagged(stage, anims, 0.25)?;
        stage.play(group)?;
        let note = precisely(
            kit,
            stage,
            "decoding succeeds exactly when the cards' rank is L (they pin down every piece, old and new), in any \
             elimination order (RFC 6330 §5.4.2.1)",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        Ok(())
    }

    /// 16_4–16_5: the ring of five with a byte card for its fifth card.
    fn ring(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let centre = v(-2.75, 0.15);
        let ring = RingProp::build(kit, stage, centre, 2.75, 0.62, true)?;
        let fifth_at = stage.get_center(ring.cards[4].group);
        // the byte card E ⊕ 2A: E plus two times A
        let two = 2u8;
        let amounts = [(4usize, 1u8), (0usize, two)];
        let value = PIECES[4] ^ mul(two, PIECES[0], FOLD);
        let byte = card_with_amounts(kit, stage, &amounts, Some(value), 0.62, fifth_at)?;
        let line = Spoken::say(kit, stage, board, "16_4")?;
        fade_all(stage, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), ring.group, ORIGIN, 1.0)?.rt(0.9));
        line.at(stage, 1)?;
        play!(stage;
            fade_out(stage.arena_mut(), ring.cards[4].group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), byte.card.group, ORIGIN, 1.0)?.rt(0.6),
        );
        pulse_edge(stage, byte.card.back, Role::byte(), 0.9)?;
        line.at(stage, 2)?;
        // the walk: each card names its next piece in terms of A
        let x0 = 1.75;
        let mut expr = vec!["A".to_owned()];
        let mut walk = Vec::new();
        let mut cards: Vec<Mob> = ring.cards.iter().take(4).map(|c| c.back).collect();
        cards.push(byte.card.back);
        for k in 0..4 {
            // card k is (piece k) ⊕ (piece k+1) = r: piece k+1 = r ⊕ piece k
            let r = result(RING[k]);
            expr.insert(0, r.to_string());
            let text = format!("{} = {}", PIECE_NAMES[k + 1], xor_tex(expr.iter().cloned()));
            let y = 2.6 - 0.75 * k as f64;
            let m = kit.tex_left(stage, &text, 40.0, Palette::ink(), &[("A", Role::byte())], v(x0, y))?;
            play!(stage;
                edge(cards[k], Palette::ink(), 0.7)?,
                fade_in(stage.arena_mut(), m, v(0.2, 0.0), 1.0)?.rt(0.7),
            );
            walk.push(m);
        }
        // the fifth card: E ⊕ 2A = 6, with E written out
        let rest: Vec<String> = expr.clone();
        let close = format!(r"{} {OPLUS} {two}A = {value}", xor_tex(rest.iter().cloned()));
        let y = 2.6 - 0.75 * 4.0;
        let m = kit.tex_left(stage, &close, 40.0, Palette::ink(), &[("A", Role::byte())], v(x0, y))?;
        play!(stage;
            edge(cards[4], Palette::ink(), 0.7)?,
            fade_in(stage.arena_mut(), m, v(0.2, 0.0), 1.0)?.rt(0.7),
        );
        walk.push(m);
        let q = predict_pause(kit, stage, v(5.8, -2.6))?;

        // 16_5: A ⊕ 2A = (1 ⊕ 2)A = 3A; the results ex-or to 15; A = 5
        let three = 1 ^ two;
        let line = Spoken::say(kit, stage, board, "16_5")?;
        let exor = kit.tex(
            stage,
            r"\text{ex-or: } A \oplus A = 0",
            32.0,
            Palette::dim(),
            &[],
            v(4.3, 1.75),
        )?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in walk.iter().take(4) {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let a = q.animate().set_opacity(0.0)?;
        anims.push(stage.prepare(a)?);
        let a = walk[4]
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .move_to(v(x0, 2.6), LEFT)?;
        anims.push(stage.prepare(a)?);
        let a = fade_in(stage.arena_mut(), exor, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(anims)?;
        stage.scene_mut().remove(&[q]);
        line.at(stage, 1)?;
        let at = v(4.3, 0.85);
        let colors = [("A", Role::byte())];
        let (s0, sp0) = kit.tex_spans(stage, &format!(r"A {OPLUS} {two}A"), 54.0, Palette::ink(), &colors, at)?;
        let (s1, sp1) = kit.tex_spans(
            stage,
            &format!(r"(1 {OPLUS} {two})A"),
            54.0,
            Palette::ink(),
            &colors,
            at,
        )?;
        let (s2, sp2) = kit.tex_spans(stage, &format!("{three}A"), 54.0, Palette::ink(), &colors, at)?;
        let byte_three = lower_third(
            kit,
            stage,
            "precisely:",
            &format!("here {three} is the byte x + 1, not a count: A ⊕ A ⊕ A is just A"),
        )?;
        play!(stage;
            write(stage.arena(), s0).rt(0.8),
            fade_in(stage.arena_mut(), byte_three, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        stage.wait(0.4)?;
        let m1 = TransformMatchingTex::new(stage.arena_mut(), s0, s1, &sp0, &sp1)?;
        play!(stage; m1.rt(1.2));
        stage.wait(0.3)?;
        let m2 = TransformMatchingTex::new(stage.arena_mut(), s1, s2, &sp1, &sp2)?;
        play!(stage; m2.rt(1.2));
        line.at(stage, 2)?;
        let results: Vec<u8> = (0..4).map(|k| result(RING[k])).chain([value]).collect();
        let total = results.iter().fold(0, |a, b| a ^ b);
        let sum = kit.tex(
            stage,
            &format!("{} = {total}", xor_tex(results.iter().map(u8::to_string))),
            40.0,
            Palette::ink(),
            &[],
            v(4.3, -0.15),
        )?;
        play!(stage; write(stage.arena(), sum).rt(1.0));
        let a_value = mul(total, inv(three), FOLD);
        let solve = kit.tex(
            stage,
            &format!(r"{three}A = {total} \;\Rightarrow\; A = {a_value}"),
            44.0,
            Palette::ink(),
            &[("A", Role::byte())],
            v(4.3, -1.05),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), exor, ORIGIN, 1.0)?.rt(0.4),
            write(stage.arena(), solve).rt(1.0),
        );
        stage.wait(0.4)?;
        // the same pieces as at the start: the walk's values, from A
        let mut piece = [0u8; 5];
        piece[0] = a_value;
        for k in 0..4 {
            piece[k + 1] = result(RING[k]) ^ piece[k];
        }
        let same = kit.tex(
            stage,
            &format!("A = {}, B = {}, C = {}, D = {}", piece[0], piece[1], piece[2], piece[3]),
            38.0,
            Role::lit(),
            &[],
            v(4.3, -1.9),
        )?;
        let caption = kit.tex(
            stage,
            &format!(
                r"3 \times 5 = {}\ \text{{here too, but}}\ 3 \times 7 = {}",
                mul(3, 5, FOLD),
                mul(3, 7, FOLD)
            ),
            40.0,
            Role::byte(),
            &[],
            v(1.9, 3.45),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), same, v(0.0, 0.15), 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.7),
        );
        // dividing is the byte clock's subtraction (R3 Dev D8)
        let [lt, lb, d] = division_clicks(total, three);
        let note = lower_third(
            kit,
            stage,
            "precisely:",
            &format!(
                "{total} ÷ {three} on the byte clock: {lt} − {lb} = {d} clicks → {a_value}. A ring of cards a·X ⊕ b·Y \
                 is stuck exactly when the ratios b/a multiply to 1 going around; with ex-or every ratio is 1"
            ),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), byte_three, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;
        Ok(())
    }
}

/// The fewest of `cards` (with `leftover` among them) whose ex-or leaves
/// only the parked piece: B ⊕ E, D ⊕ E and B ⊕ C ⊕ D give C.
fn cancel_to_c(cards: &[usize], leftover: usize) -> Vec<usize> {
    let n = cards.len();
    (1u32..1 << n)
        .map(|mask| {
            (0..n)
                .filter(|&i| mask >> i & 1 == 1)
                .map(|i| cards[i])
                .collect::<Vec<_>>()
        })
        .filter(|pick| pick.contains(&leftover))
        .filter(|pick| pick.iter().fold(0, |acc, &j| acc ^ TOY_CARDS[j]) == 1 << PARKED)
        .min_by_key(Vec::len)
        .map(|mut pick| {
            // the leftover card first, then the rest as they sit on screen, last first
            pick.sort_by_key(|&j| std::cmp::Reverse(j));
            pick
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lines 16_2 writes are the storyboard's, built from the peel.
    #[test]
    fn the_written_lines_are_the_storyboards() {
        let cards: Vec<usize> = (1..TOY_CARDS.len()).collect();
        let (steps, leftover) = peel_toy(&cards, Some(PARKED));
        let mut found: [Option<String>; 5] = Default::default();
        let mut got = Vec::new();
        for step in &steps {
            let (first, expanded, last) = step_lines(step, &found);
            found[step.piece] = Some(match (&expanded, &last) {
                (_, Some(value)) => value.clone(),
                (Some(exp), None) => exp.clone(),
                (None, None) => affine_tex(step.value),
            });
            got.push((first, expanded, last));
        }
        let o = OPLUS;
        assert_eq!(got[0], (format!("A = 2 {o} c"), None, None));
        assert_eq!(got[1], (format!("B = 6 {o} A"), Some(format!("6 {o} 2 {o} c")), None));
        assert_eq!(
            got[2],
            (
                format!("D = 6 {o} B {o} c"),
                Some(format!("6 {o} (6 {o} 2 {o} c) {o} c")),
                Some("2".to_owned())
            )
        );
        assert_eq!(got[3].0, format!("E = 14 {o} D"));
        // 16_3: the leftover card is B ⊕ E = 15; with D ⊕ E and B ⊕ C ⊕ D it gives c
        let pick = cancel_to_c(&cards, leftover[0]);
        let values: Vec<u8> = pick.iter().map(|&j| result(TOY_CARDS[j])).collect();
        assert_eq!(values, [15, 14, 6]);
        assert_eq!(values.iter().fold(0, |a, b| a ^ b), PIECES[PARKED]);
    }

    /// Narration truth for 16_4–16_5 on the ring the chapter draws.
    #[test]
    fn the_ring_with_a_byte_card_gives_a_five() {
        let e_plus_2a = PIECES[4] ^ mul(2, PIECES[0], FOLD);
        assert_eq!(e_plus_2a, 6, "E plus two times A is six");
        assert_eq!(1 ^ 2, 3, "one ex-or two is three");
        let total = (0..4).map(|k| result(RING[k])).fold(e_plus_2a, |a, b| a ^ b);
        assert_eq!(total, 15, "the five results ex-or to fifteen");
        assert_eq!(mul(total, inv(3), FOLD), PIECES[0], "A is five");
        assert_eq!(mul(3, 5, FOLD), 15);
        assert_eq!(mul(3, 7, FOLD), 9);
        // the lower-third's division in clicks: 15 ÷ 3 = 5, no wrap shown
        let [lt, lb, d] = division_clicks(total, 3);
        assert!(lt >= lb && d == lt - lb);
        assert_eq!(crate::model::exps()[usize::from(d)], PIECES[0]);
        // with the ex-or fifth card, A met itself: the ring is stuck
        assert_eq!(crate::model::rank_gf2(&RING), 4);
        // B, C, D as at the start (03's values)
        let mut piece = PIECES[0];
        for k in 0..3 {
            piece ^= result(RING[k]);
            assert_eq!(piece, PIECES[k + 1]);
        }
    }
}
