//! Chapter 08, "One step forward in time" (navier-full-script03.md cues 08_1 to 08_8; FC2 lines 13 to 27).
//!
//! The chapter opens on the last picture of root's chapter 07 (`chapter07-ledger03.rs`, sha256 a9ae04fa…),
//! rebuilt at t = 0 from that file's own numbers, and during 08_1 redraws the box in the chapter 09
//! contract (`box_view`). It ends on exactly the state chapter 09 rebuilds at its start: the corrected
//! arrows with their labels, green 0 badges, grey direction arrows, wall 0s, letters, orange pressure
//! dots, the units tag and the four step cards with "correct" lit. `construct` refuses to finish if
//! anything else is still on screen.
//!
//! Every number shown comes from `four_cell::BoxFacts`, checked before the first frame; `facts()` pins
//! the chapter's five values against the script and FC2. Voice: the kit narrator (`say`,
//! `until_sentence`, `hold`, and word gates inside 08_7's loop sentence), as in chapter 07; a silent
//! preview runs on the animation time alone.

use explainer_kit::{Facts, Kit, Palette, Timed, hex, narration};
use fmn::animation::{grow_arrow, indicate};
use fmn::prelude::*;

use crate::box_view::{
    self, BADGE_SIZE, BOX, CARD_WORD_SIZE, CARD_Y, CARDS, CORRECT, FLOW_LABEL_SIZE, LETTER_SIZE, TAG_SIZE, WALL_SIZE,
    badge_color, flow_color, p,
};
use crate::four_cell::{BoxFacts, CELL_AT, CELLS, FACES, LOOP_CELLS, Q, V4, div, sum};

/// Chapter 08's cues in spoken order and the number of sentences in each (the narrator's gates).
pub const CUES: [(&str, usize); 8] = [
    ("08_1", 1),
    ("08_2", 3),
    ("08_3", 2),
    ("08_4", 4),
    ("08_5", 4),
    ("08_6", 3),
    ("08_7", 4),
    ("08_8", 2),
];
/// Script HOLD 08_h1: the prediction countdown, in whole seconds.
pub const PREDICTION_SECONDS: u32 = 5;
/// The script's chapter heading.
const TITLE: &str = "One step forward in time";
/// The three answers 08_6 offers, in its order.
const OPTIONS: [&str; 3] = ["all of it", "none", "something in between"];
/// Where 08_7 names each loop step, in `LOOP_CELLS` order: (its first cell, seconds after sentence 2,
/// "The corrected flow goes from A to B, ...", starts). Each is the later of the whisper large-v3-turbo
/// onset on take 72704fd9 (opening-pressure recognition record 24) and the end of the pause it sits in on
/// the placed WAV a9110449, from white-ledger-projection-sync01 `phrase-markers01`. Estimates, not heard
/// boundaries.
const LOOP_WORDS: [(&str, f64); 4] = [("A", 1.30), ("B", 2.62), ("D", 3.84), ("C", 5.10)];

// Chapter 07's last picture, with `chapter07-ledger03.rs`'s own numbers (07_1 title, 07_6 box, 07_8 column).
const C7_TITLE: (&str, f64, [f64; 2]) = ("A ledger of cells", 45.0, [0.0, 3.45]);
const C7_EXTENT: f64 = 2.08;
const C7_DASHES: u32 = 11;
const C7_DASH_START: f64 = 0.065;
const C7_DASH_LENGTH: f64 = 0.23;
const C7_LETTER: (f64, f64) = (0.99, 34.0);
/// Wall zeros, in chapter 07's order: left, right, top, bottom.
const C7_WALLS: [[f64; 2]; 4] = [[-2.38, 0.0], [2.38, 0.0], [0.0, 2.40], [0.0, -2.38]];
/// Grey arrows: offset from the centre lines, half length, thickness.
const C7_SIGN: (f64, f64, f64) = (1.02, 0.22, 1.3);
const C7_CAPTIONS: [(&str, f64, f64); 2] = [("Solid walls", 28.0, -3.05), ("No flow through a wall", 24.0, -3.46)];
const C7_RIGHT: f64 = 3.35;
/// Chapter 07 draws wall zeros left, right, top, bottom; `box_view` numbers them left, right, bottom, top.
const C7_WALL_TO_VIEW: [usize; 4] = [0, 1, 3, 2];
/// Chapter 07 draws its grey arrows A→B, A→C, C→D, B→D (offset −, then +): their faces.
const C7_SIGN_TO_FACE: [usize; 4] = [0, 2, 1, 3];

// The right-hand ledger: chapter 09's columns, at this chapter's own heights.
const LX: [f64; 4] = [2.2, 3.1, 4.0, 4.9];
const LABEL_X: f64 = 0.3;
const R_FACES: f64 = 1.95;
const R_USTAR: f64 = 1.45;
const R_UNEW: f64 = 0.95;
const R_CELLS: f64 = 0.2;
const R_DIV: f64 = -0.3;
const R_SUM: f64 = -0.95;
const PANEL_X: f64 = 3.55;
const QUESTION: [f64; 4] = [3.55, -2.45, 6.6, 1.7];
const OPTION_X: [f64; 3] = [1.45, 2.85, 4.85];

pub struct Chapter08 {
    pub kit: Kit,
}

fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// Wait until `offset` seconds after sentence `j` of line `id` begins, so a picture lands on a word inside
/// that sentence. It is `Kit::until_sentence` with an offset, on the same clock arithmetic (offset 0 is that
/// gate exactly): nothing if the moment has passed or without a narrator, and never past sentence `j + 1`.
fn wait_for_word(kit: &Kit, stage: &mut Stage<'_>, id: &str, j: usize, offset: f64) -> fmn::Result<()> {
    let Some(narrator) = kit.narrator.as_ref() else {
        return Ok(());
    };
    let Some(start) = narrator.sentence_at(id, j) else {
        return Ok(());
    };
    let at = narrator
        .sentence_at(id, j + 1)
        .map_or(start + offset, |next| next.min(start + offset));
    let (now, fps) = (stage.scene().time().to_f64(), stage.scene().time().fps());
    if at > now {
        narration::wait_frames(stage, narration::frames_covering(at - now, fps))?;
    }
    Ok(())
}

/// The one face u* moves; the script says only A→B does.
fn moving_face(f: &BoxFacts) -> Result<usize, String> {
    let moving: Vec<usize> = (0..FACES.len()).filter(|&g| !f.provisional[g].is_zero()).collect();
    match moving.as_slice() {
        &[g] if (CELLS[FACES[g].0], CELLS[FACES[g].1]) == ("A", "B") => Ok(g),
        _ => Err(format!("u* must move only the A→B face, not faces {moving:?}")),
    }
}

/// What face `g`'s provisional flow alone adds to each cell's net outflow: equal and opposite.
fn face_share(f: &BoxFacts, g: usize) -> V4 {
    let mut single = [Q::ZERO; 4];
    single[g] = f.provisional[g];
    div(&single)
}

/// Each step of the loop A→B→D→C→A: its face and the corrected flow along the direction of travel.
fn loop_steps(f: &BoxFacts) -> Vec<(usize, Q)> {
    LOOP_CELLS
        .windows(2)
        .map(|w| {
            let forward = FACES.iter().position(|&(a, b)| (a, b) == (w[0], w[1]));
            match forward {
                Some(g) => (g, f.corrected[g]),
                None => {
                    let g = FACES
                        .iter()
                        .position(|&(a, b)| (a, b) == (w[1], w[0]))
                        .expect("neighbouring loop cells share a face");
                    (g, -f.corrected[g])
                }
            }
        })
        .collect()
}

/// The faces cell `i` shares with another cell (a corner cell has two).
fn cell_faces(i: usize) -> Vec<usize> {
    (0..FACES.len())
        .filter(|&g| FACES[g].0 == i || FACES[g].1 == i)
        .collect()
}

/// Which of the 08_6 answers is right for the kept fraction.
fn answer_index(kept: Q) -> usize {
    if kept == Q::ONE {
        0
    } else if kept.is_zero() {
        1
    } else {
        2
    }
}

/// Chapter 07's closing claim, recomputed: the cells' imbalances add to zero for every face's flow.
fn sealed_sum() -> Q {
    (0..FACES.len())
        .map(|g| {
            let mut unit = [Q::ZERO; 4];
            unit[g] = Q::ONE;
            sum(&div(&unit))
        })
        .fold(Q::ZERO, |a, b| a + b)
}

/// Display math for a signed sum, `1 - 1 + 0 + 0`.
fn sum_tex(terms: &[Q]) -> String {
    let mut out = String::new();
    for (k, &t) in terms.iter().enumerate() {
        match (k, t < Q::ZERO) {
            (0, _) => out.push_str(&t.tex()),
            (_, true) => out.push_str(&format!(" - {}", t.abs().tex())),
            (_, false) => out.push_str(&format!(" + {}", t.tex())),
        }
    }
    out
}

/// A signed chip, `+1` or `-1`.
fn signed_tex(q: Q) -> String {
    if q > Q::ZERO { format!("+{}", q.tex()) } else { q.tex() }
}

/// The four box facts chapter 08 shares with chapter 09's printed table (`BoxFacts::fact_rows`).
/// Chapter 08 registers them; a film-wide registry must not register them again for chapter 09.
pub const SHARED_FACTS: [&str; 4] = ["box.ustar", "box.div.ustar", "box.unew", "box.div.after"];

/// The chapter's numbers alone, checked (a preview's `facts`).
pub fn facts() -> Result<Facts, String> {
    let mut facts = Facts::new();
    register(&mut facts)?;
    facts.self_check()?;
    Ok(facts)
}

/// Add chapter 08's numbers to a film's registry, pinned to the script and FC2 (the CLI's
/// `self_check` compares every pin). The shared rows are `fact_rows`'s own strings, so both
/// chapters print each value one way.
pub fn register(facts: &mut Facts) -> Result<(), String> {
    let f = BoxFacts::solve();
    f.check()?;
    let ab = moving_face(&f)?;
    let rows = f.fact_rows();
    let row = |key: &str| {
        rows.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| format!("four_cell prints no fact {key}"))
    };
    facts
        .pin(
            "box.ustar",
            row(SHARED_FACTS[0])?,
            "1, 0, 0, 0",
            "script 08_5 picture; FC2 line 19",
        )?
        .pin(
            "box.div.ustar",
            row(SHARED_FACTS[1])?,
            "1, -1, 0, 0",
            "script 08_5 picture; FC2 line 19",
        )?
        .add("box.div.ustar.sum", sum(&f.imbalance))?
        .add("box.prediction.seconds", PREDICTION_SECONDS)?
        .pin(
            "box.kept.AB",
            f.corrected[ab] / f.provisional[ab],
            "1/4",
            "script 08_7 \"One quarter remains\"; FC2 line 25",
        )?
        .pin(
            "box.unew",
            row(SHARED_FACTS[2])?,
            "1/4, -1/4, -1/4, 1/4",
            "script 08_7 picture; FC2 line 27",
        )?
        .pin(
            "box.div.after",
            row(SHARED_FACTS[3])?,
            "0, 0, 0, 0",
            "script 08_7 picture; FC2 line 27",
        )?;
    Ok(())
}

/// Chapter 07's last picture, as handles.
struct Chapter07End {
    title: Mob,
    panel: Mob,
    dashes: Vec<Mob>,
    letters: [Mob; 4],
    captions: [Mob; 2],
    zeros: [Mob; 4],
    signs: [Mob; 4],
    column: [Mob; 3],
}

struct Ch8<'a> {
    k: &'a Kit,
    on: Vec<Mob>,
    pending: Vec<Box<dyn Animation>>,
}

impl Ch8<'_> {
    fn text(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text(stage, s, size, color, at)
    }

    fn text_left(&self, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.text_left(stage, s, size, color, at)
    }

    fn tex(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex(stage, src, size, color, &[], at)
    }

    fn tex_left(&self, stage: &mut Stage<'_>, src: &str, size: f64, color: Srgb, at: Vec3) -> fmn::Result<Mob> {
        self.k.tex_left(stage, src, size, color, &[], at)
    }

    /// Four values in the ledger's columns at height `y`.
    fn row(&self, stage: &mut Stage<'_>, values: &V4, y: f64, color: fn(Q) -> Srgb) -> fmn::Result<[Mob; 4]> {
        let mut out = Vec::with_capacity(4);
        for (k, &value) in values.iter().enumerate() {
            out.push(self.tex(stage, &value.tex(), 30.0, color(value), p(LX[k], y))?);
        }
        Ok([out[0], out[1], out[2], out[3]])
    }

    /// Show at once (no fade).
    fn put(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                stage.add_to_scene(m)?;
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Remove at once (no fade): the countdown digits, as chapters 07 and 11 do.
    fn cut(&mut self, stage: &mut Stage<'_>, mobs: &[Mob]) {
        let gone: Vec<Mob> = mobs.iter().copied().filter(|m| self.on.contains(m)).collect();
        stage.scene_mut().remove(&gone);
        self.on.retain(|m| !gone.contains(m));
    }

    fn appear(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if !self.on.contains(&m) {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.push(m);
            }
        }
        Ok(())
    }

    /// Fade out what is on screen; anything already gone is skipped (no second fade).
    fn vanish(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
                self.on.retain(|&x| x != m);
            }
        }
        Ok(())
    }

    /// `from` (on screen) takes `to`'s shape and paint; `to` stays detached.
    fn morph(&mut self, stage: &mut Stage<'_>, from: Mob, to: Mob, secs: f64) -> fmn::Result<()> {
        if self.on.contains(&from) {
            self.pending.push(stage.prepare(Transform::new(from, to).rt(secs))?);
        }
        Ok(())
    }

    fn grow(&mut self, stage: &mut Stage<'_>, arrow: Mob, secs: f64) -> fmn::Result<()> {
        if !self.on.contains(&arrow) {
            let a = grow_arrow(stage.arena_mut(), arrow)?.rt(secs);
            self.pending.push(stage.prepare(a)?);
            self.on.push(arrow);
        }
        Ok(())
    }

    fn pulse(&mut self, stage: &mut Stage<'_>, mobs: &[Mob], secs: f64) -> fmn::Result<()> {
        for &m in mobs {
            if self.on.contains(&m) {
                let a = indicate(stage.arena_mut(), m, 1.15, None)?.rt(secs);
                self.pending.push(stage.prepare(a)?);
            }
        }
        Ok(())
    }

    fn play(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        if !self.pending.is_empty() {
            stage.play_prepared(std::mem::take(&mut self.pending))?;
        }
        Ok(())
    }

    fn wait(&mut self, stage: &mut Stage<'_>, secs: f64) -> fmn::Result<()> {
        self.play(stage)?;
        stage.wait(secs)?;
        Ok(())
    }

    /// Finish what is queued, then start line `id`.
    fn say(&mut self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        self.play(stage)?;
        self.k.say(stage, id)
    }

    /// Finish what is queued, then wait for sentence `j` of line `id`.
    fn until(&mut self, stage: &mut Stage<'_>, id: &str, j: usize) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(fail(format!("chapter 08 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        self.k.until_sentence(stage, id, j)
    }

    /// Finish what is queued, then wait for a word `offset` seconds into sentence `j` of line `id`.
    fn until_word(&mut self, stage: &mut Stage<'_>, id: &str, j: usize, offset: f64) -> fmn::Result<()> {
        let sentences = CUES.iter().find(|c| c.0 == id).map_or(0, |c| c.1);
        if j >= sentences {
            return Err(fail(format!("chapter 08 line {id} has no sentence {j}")));
        }
        self.play(stage)?;
        wait_for_word(self.k, stage, id, j, offset)
    }

    /// Card `k` lit or dim, word and rim together.
    fn light(&mut self, stage: &mut Stage<'_>, card: (Mob, Mob), k: usize, lit: bool, secs: f64) -> fmn::Result<()> {
        let (rim, ink) = box_view::card(stage, k, lit)?;
        let word = self.text(stage, CARDS[k], CARD_WORD_SIZE, ink, p(box_view::card_x(k), CARD_Y))?;
        self.morph(stage, card.0, rim, secs)?;
        self.morph(stage, card.1, word, secs)
    }

    /// Chapter 07's last picture, drawn at once with that chapter's numbers and kit text.
    fn chapter07_end(&mut self, stage: &mut Stage<'_>) -> fmn::Result<Chapter07End> {
        let (ink, dim, gold) = (Palette::ink(), Palette::dim(), Palette::gold());
        let [x, y] = BOX;
        let title = self.text(stage, C7_TITLE.0, C7_TITLE.1, ink, p(C7_TITLE.2[0], C7_TITLE.2[1]))?;
        let side = 2.0 * C7_EXTENT;
        let panel = stage.arena_mut().add(
            Rectangle::new()
                .width(side)
                .height(side)
                .style(Style::default().stroke(gold, 2.0, 1.0).fill(gold, 0.08))
                .build()?
                .moved_to(p(x, y)),
        );
        let mut dashes = Vec::new();
        for part in 0..C7_DASHES {
            let a = -C7_EXTENT + f64::from(part) * (side / f64::from(C7_DASHES)) + C7_DASH_START;
            let b = a + C7_DASH_LENGTH;
            dashes.push(box_view::line(stage, p(x, y + a), p(x, y + b), dim, 1.7)?);
            dashes.push(box_view::line(stage, p(x + a, y), p(x + b, y), dim, 1.7)?);
        }
        let mut letters = Vec::new();
        for (i, name) in CELLS.iter().enumerate() {
            let (c, r) = CELL_AT[i];
            let (sx, sy) = (2.0 * c as f64 - 1.0, 2.0 * r as f64 - 1.0);
            let at = p(x + sx * C7_LETTER.0, y + sy * C7_LETTER.0);
            letters.push(self.text(stage, name, C7_LETTER.1, ink, at)?);
        }
        let mut captions = Vec::new();
        for (k, &(words, size, cy)) in C7_CAPTIONS.iter().enumerate() {
            let color = if k == 0 { gold } else { dim };
            captions.push(self.text(stage, words, size, color, p(x, cy))?);
        }
        let mut zeros = Vec::new();
        for [dx, dy] in C7_WALLS {
            zeros.push(self.text(stage, &Q::ZERO.to_string(), WALL_SIZE, gold, p(x + dx, y + dy))?);
        }
        let (offset, half, width) = C7_SIGN;
        let mut signs = Vec::new();
        for o in [-offset, offset] {
            for horizontal in [true, false] {
                let (from, to) = if horizontal {
                    (p(x - half, y + o), p(x + half, y + o))
                } else {
                    (p(x + o, y - half), p(x + o, y + half))
                };
                // Chapter 07's own arrow: no tip ratio, filled and stroked in one colour.
                signs.push(
                    stage.arena_mut().add(
                        Arrow::new(from, to)
                            .buff(0.0)
                            .thickness(width)
                            .style(Style::default().fill(dim, 1.0).stroke(dim, 1.0, 1.0))
                            .build()?,
                    ),
                );
            }
        }
        let column = [
            self.text(
                stage,
                &format!("A + B + C + D = {}", sealed_sum()),
                36.0,
                Palette::green(),
                p(C7_RIGHT, 0.63),
            )?,
            self.text(stage, "Each inner face cancels", 26.0, ink, p(C7_RIGHT, -0.26))?,
            self.text(stage, "Every wall contributes zero", 24.0, dim, p(C7_RIGHT, -1.03))?,
        ];
        let end = Chapter07End {
            title,
            panel,
            dashes,
            letters: [letters[0], letters[1], letters[2], letters[3]],
            captions: [captions[0], captions[1]],
            zeros: [zeros[0], zeros[1], zeros[2], zeros[3]],
            signs: [signs[0], signs[1], signs[2], signs[3]],
            column,
        };
        let mut first = vec![end.title, end.panel];
        first.extend(end.dashes.iter().copied());
        first.extend(end.letters);
        first.extend(end.captions);
        first.extend(end.zeros);
        first.extend(end.signs);
        first.extend(end.column);
        self.put(stage, &first)?;
        Ok(end)
    }
}

impl SceneConstruct for Chapter08 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let f = BoxFacts::solve();
        // The render log carries the fact table, so each picture's numbers can be read against it.
        for line in facts().map_err(fail)?.print().lines() {
            println!("Chapter 08 fact {line}");
        }
        let ab = moving_face(&f).map_err(fail)?;
        let (a, b) = FACES[ab];
        let share = face_share(&f, ab);
        if share != f.imbalance || share[a] != -share[b] || !sum(&share).is_zero() {
            return Err(fail(
                "the A→B face alone must make the whole, equal and opposite imbalance".into(),
            ));
        }
        let steps = loop_steps(&f);
        if steps.iter().any(|&(_, along)| along <= Q::ZERO) {
            return Err(fail("the corrected flow must run A→B→D→C→A on every face".into()));
        }
        let mut s = Ch8 {
            k: &self.kit,
            on: Vec::new(),
            pending: Vec::new(),
        };
        let (ink, dim, hero, flow) = (Palette::ink(), Palette::dim(), Palette::hero(), flow_color());
        let c7 = s.chapter07_end(stage)?;
        println!("Chapter 08 opens on the chapter 07 end state: {} objects", s.on.len());

        // 08_1: chapter 07's box becomes this chapter's box; the title, then the four step cards.
        s.say(stage, "08_1")?;
        let mut leaving = vec![c7.title];
        leaving.extend(c7.captions);
        leaving.extend(c7.column);
        leaving.extend(c7.dashes.iter().copied());
        s.vanish(stage, &leaving, 0.5)?;
        let outline = c7.panel;
        let target = box_view::outline(stage)?;
        s.morph(stage, outline, target, 0.9)?;
        let mut walls = c7.zeros;
        for (k, &zero) in c7.zeros.iter().enumerate() {
            let w = C7_WALL_TO_VIEW[k];
            let target = s.tex(stage, &Q::ZERO.tex(), WALL_SIZE, dim, box_view::wall_label_at(w))?;
            s.morph(stage, zero, target, 0.9)?;
            walls[w] = zero;
        }
        let letters = c7.letters;
        for (i, (&letter, name)) in letters.iter().zip(CELLS.iter()).enumerate() {
            let target = s.text(stage, name, LETTER_SIZE, ink, box_view::letter_at(i))?;
            s.morph(stage, letter, target, 0.9)?;
        }
        let mut directions = c7.signs;
        for (k, &sign) in c7.signs.iter().enumerate() {
            let g = C7_SIGN_TO_FACE[k];
            let target = box_view::direction_arrow(stage, g)?;
            s.morph(stage, sign, target, 0.9)?;
            directions[g] = sign;
        }
        let faces: Vec<Mob> = (0..FACES.len())
            .map(|g| box_view::dashed_face(stage, g))
            .collect::<fmn::Result<_>>()?;
        s.appear(stage, &faces, 0.9)?;
        s.play(stage)?;
        let title = s.text(stage, TITLE, 45.0, ink, p(0.0, CARD_Y))?;
        s.appear(stage, &[title], 0.5)?;
        s.wait(stage, 0.8)?;
        s.vanish(stage, &[title], 0.4)?;
        s.play(stage)?;
        let mut cards: Vec<(Mob, Mob)> = Vec::new();
        for (k, name) in CARDS.iter().enumerate() {
            let (rim, word_color) = box_view::card(stage, k, false)?;
            let word = s.text(stage, name, CARD_WORD_SIZE, word_color, p(box_view::card_x(k), CARD_Y))?;
            s.appear(stage, &[rim, word], 0.35)?;
            s.wait(stage, 0.1)?;
            cards.push((rim, word));
        }

        // 08_2: carry, smooth and forces light in turn; their result is u*.
        s.say(stage, "08_2")?;
        s.until(stage, "08_2", 1)?;
        for (k, &card) in cards.iter().enumerate().take(CORRECT) {
            s.light(stage, card, k, true, 0.4)?;
            s.wait(stage, 0.55)?;
        }
        s.until(stage, "08_2", 2)?;
        let (left, right) = (box_view::card_x(0) - 0.55, box_view::card_x(CORRECT - 1) + 0.55);
        let rail = CARD_Y - 0.5;
        let bracket = [
            box_view::line(stage, p(left, rail), p(right, rail), flow, 2.0)?,
            box_view::line(stage, p(left, rail), p(left, rail + 0.13), flow, 2.0)?,
            box_view::line(stage, p(right, rail), p(right, rail + 0.13), flow, 2.0)?,
        ];
        let ustar = s.tex(stage, "u^*", 34.0, flow, p((left + right) / 2.0, rail - 0.32))?;
        s.appear(stage, &bracket, 0.4)?;
        s.appear(stage, &[ustar], 0.5)?;

        // 08_3: the volume rule was ignored, so the badges flicker red.
        s.say(stage, "08_3")?;
        let mut badges = Vec::new();
        for i in 0..CELLS.len() {
            badges.push(box_view::badge_frame(stage, i, dim)?);
        }
        s.appear(stage, &badges, 0.5)?;
        s.until(stage, "08_3", 1)?;
        for color in [Palette::red(), dim, Palette::red(), dim, Palette::red()] {
            for (i, &badge) in badges.iter().enumerate() {
                let target = box_view::badge_frame(stage, i, color)?;
                s.morph(stage, badge, target, 0.18)?;
            }
            s.play(stage)?;
        }

        // 08_4: "correct" lights; pressure lives at the centres; the push rule under the card.
        s.say(stage, "08_4")?;
        for (k, &card) in cards.iter().enumerate() {
            s.light(stage, card, k, k == CORRECT, 0.5)?;
        }
        s.until(stage, "08_4", 1)?;
        let dots: Vec<Mob> = (0..CELLS.len()).map(|i| box_view::dot(stage, i)).collect();
        s.appear(stage, &dots, 0.5)?;
        s.pulse(stage, &badges, 0.7)?;
        s.until(stage, "08_4", 2)?;
        let under = box_view::card_x(CORRECT) - 0.75;
        let rule = s.text_left(
            stage,
            "add the push from high to low pressure",
            22.0,
            ink,
            p(under, CARD_Y - 0.62),
        )?;
        s.appear(stage, &[rule], 0.5)?;
        s.wait(stage, 1.0)?;
        let same = s.text_left(
            stage,
            "= subtract the pressure gradient",
            22.0,
            dim,
            p(under, CARD_Y - 1.0),
        )?;
        s.appear(stage, &[same], 0.5)?;
        s.until(stage, "08_4", 3)?;
        s.pulse(stage, &[cards[CORRECT].0, cards[CORRECT].1], 0.8)?;

        // 08_5: the units tag before the first number; then one unit on A→B, zero on the rest.
        s.say(stage, "08_5")?;
        let mut notes = bracket.to_vec();
        notes.extend([ustar, rule, same]);
        s.vanish(stage, &notes, 0.4)?;
        s.play(stage)?;
        let tag = s.tex_left(stage, &box_view::tag_tex(), TAG_SIZE, dim, box_view::tag_at())?;
        s.appear(stage, &[tag], 0.5)?;
        s.wait(stage, 0.3)?;
        let mut flows: [Option<Mob>; 4] = [None; 4];
        let mut labels = Vec::new();
        for (g, &value) in f.provisional.iter().enumerate() {
            let label = s.tex(stage, &value.tex(), FLOW_LABEL_SIZE, flow, box_view::flow_label_at(g))?;
            labels.push(label);
            if let Some(m) = box_view::centred_flow(stage, g, value)? {
                s.grow(stage, m, 0.7)?;
                s.appear(stage, &[label], 0.5)?;
                flows[g] = Some(m);
            }
        }
        s.wait(stage, 0.4)?;
        let held: Vec<Mob> = (0..FACES.len())
            .filter(|&g| flows[g].is_none())
            .map(|g| labels[g])
            .collect();
        s.appear(stage, &held, 0.5)?;
        let mut ledger = Vec::new();
        for (k, &(from, to)) in FACES.iter().enumerate() {
            let head = format!("{}\\to {}", CELLS[from], CELLS[to]);
            ledger.push(s.tex(stage, &head, 26.0, dim, p(LX[k], R_FACES))?);
        }
        ledger.push(s.tex_left(stage, "u^*", 28.0, dim, p(LABEL_X, R_USTAR))?);
        let ustar_row = s.row(stage, &f.provisional, R_USTAR, |q| {
            if q.is_zero() { Palette::dim() } else { flow_color() }
        })?;
        ledger.extend(ustar_row);
        s.appear(stage, &ledger, 0.5)?;

        // A loses through the face, B gains: the face's equal and opposite pair moves into the badges.
        s.until(stage, "08_5", 1)?;
        let chips = [
            s.tex(
                stage,
                &signed_tex(share[a]),
                26.0,
                Palette::red(),
                box_view::on_face(ab, -0.42, -0.45),
            )?,
            s.tex(
                stage,
                &signed_tex(share[b]),
                26.0,
                Palette::red(),
                box_view::on_face(ab, -0.42, 0.45),
            )?,
        ];
        s.appear(stage, &chips, 0.4)?;
        s.wait(stage, 0.5)?;
        let mut values = Vec::new();
        for (i, &value) in f.imbalance.iter().enumerate() {
            let color = badge_color(value);
            let rim = box_view::badge_frame(stage, i, color)?;
            let number = s.tex(stage, &value.tex(), BADGE_SIZE, color, box_view::badge_at(i))?;
            s.morph(stage, badges[i], rim, 0.7)?;
            let shown = if i == a || i == b {
                let chip = chips[usize::from(i == b)];
                s.morph(stage, chip, number, 0.7)?;
                chip
            } else {
                s.appear(stage, &[number], 0.5)?;
                number
            };
            values.push(shown);
        }
        s.wait(stage, 0.3)?;
        let mut cells_ledger = Vec::new();
        for (k, name) in CELLS.iter().enumerate() {
            cells_ledger.push(s.text(stage, name, 22.0, dim, p(LX[k], R_CELLS))?);
        }
        let div_label = s.tex_left(stage, "\\operatorname{div} u^*", 28.0, dim, p(LABEL_X, R_DIV))?;
        let div_row = s.row(stage, &f.imbalance, R_DIV, badge_color)?;
        cells_ledger.push(div_label);
        cells_ledger.extend(div_row);
        s.appear(stage, &cells_ledger, 0.5)?;
        s.wait(stage, 0.4)?;
        let total = |v: &V4| format!("\\text{{sum: }} {} = {}", sum_tex(v), sum(v).tex());
        let sum_line = s.tex(stage, &total(&f.imbalance), 28.0, Palette::green(), p(PANEL_X, R_SUM))?;
        s.appear(stage, &[sum_line], 0.5)?;
        s.until(stage, "08_5", 2)?;
        let unbalanced: Vec<Mob> = (0..CELLS.len())
            .filter(|&i| !f.imbalance[i].is_zero())
            .flat_map(|i| [badges[i], values[i], div_row[i]])
            .collect();
        s.pulse(stage, &unbalanced, 0.8)?;
        s.until(stage, "08_5", 3)?;
        s.pulse(stage, &[cards[CORRECT].0, cards[CORRECT].1], 0.8)?;

        // 08_6: the prediction card; nothing corrected is shown until the countdown ends.
        s.say(stage, "08_6")?;
        s.pulse(stage, &dots, 0.8)?;
        s.until(stage, "08_6", 1)?;
        let [qx, qy, qw, qh] = QUESTION;
        let card = stage.arena_mut().add(
            Rectangle::new()
                .width(qw)
                .height(qh)
                .corner_radius(0.12)
                .style(Style::default().fill(hex("#182433"), 1.0).stroke(dim, 1.4, 1.0))
                .build()?
                .moved_to(p(qx, qy)),
        );
        let question_src = format!(
            "\\text{{How much of the }} {}\\to {} \\text{{ speed remains?}}",
            CELLS[a], CELLS[b]
        );
        let question = s.tex(stage, &question_src, 30.0, ink, p(qx, qy + 0.5))?;
        s.appear(stage, &[card, question], 0.5)?;
        let moving: Vec<Mob> = flows[ab].into_iter().chain([labels[ab]]).collect();
        s.pulse(stage, &moving, 0.8)?;
        s.until(stage, "08_6", 2)?;
        let mut options = Vec::new();
        for (&x, words) in OPTION_X.iter().zip(OPTIONS) {
            options.push(s.text(stage, words, 22.0, dim, p(x, qy - 0.05))?);
        }
        for &option in &options {
            s.appear(stage, &[option], 0.35)?;
            s.wait(stage, 0.15)?;
        }
        // HOLD 08_h1: the line finishes, then whole seconds count down (added and removed, as chapter 07 does).
        s.play(stage)?;
        s.k.hold(stage)?;
        s.k.mark(stage, "predict");
        for second in (1..=PREDICTION_SECONDS).rev() {
            let clock = s.text(stage, &second.to_string(), 34.0, hero, p(qx, qy - 0.55))?;
            s.put(stage, &[clock])?;
            stage.wait(1.0)?;
            s.cut(stage, &[clock]);
        }

        // 08_7: a quarter remains on A→B; the other faces move; the loop; every cell balances.
        s.say(stage, "08_7")?;
        let kept = f.corrected[ab] / f.provisional[ab];
        let pick = answer_index(kept);
        let picked = s.text(stage, OPTIONS[pick], 22.0, hero, p(OPTION_X[pick], qy - 0.05))?;
        s.morph(stage, options[pick], picked, 0.4)?;
        if let (Some(m), Some(target)) = (flows[ab], box_view::centred_flow(stage, ab, f.corrected[ab])?) {
            s.morph(stage, m, target, 0.9)?;
        }
        let quarter = s.tex(
            stage,
            &f.corrected[ab].tex(),
            FLOW_LABEL_SIZE,
            flow,
            box_view::flow_label_at(ab),
        )?;
        s.morph(stage, labels[ab], quarter, 0.9)?;
        let unew_label = s.tex_left(stage, "u_{\\text{new}}", 28.0, flow, p(LABEL_X, R_UNEW))?;
        let unew_row = s.row(stage, &f.corrected, R_UNEW, |_| flow_color())?;
        s.appear(stage, &[unew_label, unew_row[ab]], 0.5)?;
        s.until(stage, "08_7", 1)?;
        let mut card_parts = vec![card, question];
        card_parts.extend(options.iter().copied());
        s.vanish(stage, &card_parts, 0.4)?;
        for (g, &value) in f.corrected.iter().enumerate() {
            if g == ab {
                continue;
            }
            if let Some(m) = box_view::centred_flow(stage, g, value)? {
                s.grow(stage, m, 0.8)?;
                flows[g] = Some(m);
            }
            let target = s.tex(stage, &value.tex(), FLOW_LABEL_SIZE, flow, box_view::flow_label_at(g))?;
            s.morph(stage, labels[g], target, 0.8)?;
            s.appear(stage, &[unew_row[g]], 0.5)?;
        }
        s.until(stage, "08_7", 2)?;
        // Each face pulses as its step is said.
        for (&(g, _), &(_, offset)) in steps.iter().zip(&LOOP_WORDS) {
            s.until_word(stage, "08_7", 2, offset)?;
            let along: Vec<Mob> = flows[g].into_iter().chain([labels[g], unew_row[g]]).collect();
            s.pulse(stage, &along, 0.6)?;
            s.play(stage)?;
        }
        s.until(stage, "08_7", 3)?;
        let div_after = s.tex_left(
            stage,
            "\\operatorname{div} u_{\\text{new}}",
            28.0,
            dim,
            p(LABEL_X, R_DIV),
        )?;
        // Clear the provisional row before showing any corrected cell.
        // Unchecked cells keep blank entries until their two faces are highlighted.
        s.cut(stage, &[div_label, sum_line]);
        s.cut(stage, &div_row);
        s.cut(stage, &values);
        values.clear();
        let mut div_entries = Vec::with_capacity(CELLS.len());
        s.appear(stage, &[div_after], 0.5)?;
        for (i, &value) in f.corrected_div.iter().enumerate() {
            let own: Vec<Mob> = cell_faces(i).into_iter().filter_map(|g| flows[g]).collect();
            s.pulse(stage, &own, 0.5)?;
            let color = badge_color(value);
            let rim = box_view::badge_frame(stage, i, color)?;
            let number = s.tex(stage, &value.tex(), BADGE_SIZE, color, box_view::badge_at(i))?;
            let entry = s.tex(stage, &value.tex(), 30.0, color, p(LX[i], R_DIV))?;
            s.morph(stage, badges[i], rim, 0.5)?;
            s.appear(stage, &[number, entry], 0.5)?;
            values.push(number);
            div_entries.push(entry);
            s.play(stage)?;
        }
        let balanced = s.tex(
            stage,
            &total(&f.corrected_div),
            28.0,
            Palette::green(),
            p(PANEL_X, R_SUM),
        )?;
        s.appear(stage, &[balanced], 0.5)?;

        // 08_8: all four arrows indicate once; the ledger clears, leaving chapter 09's opening picture.
        s.say(stage, "08_8")?;
        let arrows: Vec<Mob> = flows.iter().flatten().copied().collect();
        s.pulse(stage, &arrows, 0.9)?;
        s.until(stage, "08_8", 1)?;
        let mut panel = ledger.clone();
        panel.extend(cells_ledger.iter().copied());
        panel.extend([unew_label, div_after, balanced]);
        panel.extend(div_entries);
        panel.extend(unew_row);
        s.vanish(stage, &panel, 0.6)?;
        s.play(stage)?;

        let mut contract = vec![outline, tag];
        contract.extend(faces.iter().copied());
        contract.extend(walls);
        contract.extend(directions);
        contract.extend(letters);
        contract.extend(dots.iter().copied());
        contract.extend(flows.iter().flatten().copied());
        contract.extend(labels.iter().copied());
        contract.extend(badges.iter().copied());
        contract.extend(values.iter().copied());
        contract.extend(cards.iter().flat_map(|&(rim, word)| [rim, word]));
        let extra: Vec<&Mob> = s.on.iter().filter(|m| !contract.contains(m)).collect();
        let missing = contract.iter().filter(|m| !s.on.contains(m)).count();
        if !extra.is_empty() || missing > 0 {
            return Err(fail(format!(
                "chapter 08 must end on chapter 09's opening picture: {} extra, {missing} missing",
                extra.len()
            )));
        }
        // Chapter 09 opens with these plus four clear cell fills, which draw nothing.
        println!("Chapter 08 ends on the chapter 09 start: {} objects", contract.len());
        s.k.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_pin_the_script() {
        let facts = facts().expect("chapter 08 facts");
        assert_eq!(facts.get("box.kept.AB"), Some("1/4"));
        assert_eq!(facts.get("box.div.ustar.sum"), Some("0"));
        assert_eq!(facts.len(), 7);
        let rows = BoxFacts::solve().fact_rows();
        for key in SHARED_FACTS {
            let row = rows.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
            assert_eq!(facts.get(key), row, "{key} prints as chapter 09 prints it");
        }
        // A film registry that already holds other chapters' keys takes these without a clash.
        let mut film = Facts::new();
        film.add("box.sealed.sum", 0).expect("a chapter 07 key");
        register(&mut film).expect("chapter 08 joins the film registry");
        assert_eq!(film.len(), 8);
        film.self_check().expect("every pin holds in the film registry");
    }

    #[test]
    fn only_a_to_b_moves_and_its_pair_cancels() {
        let f = BoxFacts::solve();
        let ab = moving_face(&f).expect("one moving face");
        let (a, b) = FACES[ab];
        let share = face_share(&f, ab);
        assert_eq!(share, f.imbalance);
        assert_eq!(share[a], -share[b]);
        assert!(share[a] > Q::ZERO, "A loses fluid: positive net outflow");
        assert!(sum(&share).is_zero() && sealed_sum().is_zero());
        assert_eq!((0..4).filter(|&g| f.provisional[g].is_zero()).count(), 3);
    }

    #[test]
    fn the_corrected_flow_circulates_and_each_cell_balances_two_faces() {
        let f = BoxFacts::solve();
        let steps = loop_steps(&f);
        assert_eq!(steps.len(), 4);
        assert!(steps.iter().all(|&(_, along)| along == steps[0].1 && along > Q::ZERO));
        let mut seen: Vec<usize> = steps.iter().map(|&(g, _)| g).collect();
        seen.sort_unstable();
        assert_eq!(seen, vec![0, 1, 2, 3]);
        for i in 0..4 {
            let faces = cell_faces(i);
            assert_eq!(faces.len(), 2, "a corner cell has two open faces");
            let out = |g: usize| {
                if FACES[g].0 == i {
                    f.corrected[g]
                } else {
                    -f.corrected[g]
                }
            };
            assert_eq!(out(faces[0]), -out(faces[1]), "cell {i}: one face in, one out");
            assert!(!out(faces[0]).is_zero());
        }
        assert_eq!(answer_index(f.corrected[0] / f.provisional[0]), 2);
    }

    #[test]
    fn the_loop_pulses_wait_for_the_spoken_steps() {
        let script = include_str!("../../script.tsv");
        let text = script
            .lines()
            .find_map(|l| l.strip_prefix("08_7\t"))
            .expect("08_7 in the script");
        let c = LOOP_CELLS.map(|i| CELLS[i]);
        let spoken = format!(
            "from {} to {}, {} to {}, {} to {} and {} back to {},",
            c[0], c[1], c[1], c[2], c[2], c[3], c[3], c[4]
        );
        assert!(
            text.contains(&spoken),
            "08_7 says the loop in LOOP_CELLS order: {spoken:?}"
        );
        assert_eq!(LOOP_WORDS.len(), loop_steps(&BoxFacts::solve()).len());
        for (k, &(cell, _)) in LOOP_WORDS.iter().enumerate() {
            assert_eq!(cell, c[k], "step {k} pulses when its first cell is said");
        }
        assert!(LOOP_WORDS[0].1 > 0.0 && LOOP_WORDS.windows(2).all(|w| w[0].1 < w[1].1));
    }

    #[test]
    fn chapter07_arrows_and_zeros_land_on_their_faces_and_walls() {
        let (offset, _, _) = C7_SIGN;
        for (k, &g) in C7_SIGN_TO_FACE.iter().enumerate() {
            let horizontal = k % 2 == 0;
            let side = if k < 2 { -offset } else { offset };
            let d = box_view::face_dir(g);
            let m = box_view::face_mid(g);
            assert_eq!(d[0] > 0.0, horizontal, "sign {k}");
            let across = if horizontal { m[1] - BOX[1] } else { m[0] - BOX[0] };
            assert!(across * side > 0.0, "sign {k} sits on face {g}'s side");
        }
        for (k, &w) in C7_WALL_TO_VIEW.iter().enumerate() {
            let [dx, dy] = C7_WALLS[k];
            let to = box_view::wall_label_at(w);
            assert!((to[0] - BOX[0]) * dx >= 0.0 && (to[1] - BOX[1]) * dy >= 0.0, "wall {k}");
            assert!((to[0] - BOX[0]).abs() > 1.0 || (to[1] - BOX[1]).abs() > 1.0);
        }
    }

    #[test]
    fn cues_and_signs() {
        let ids: Vec<&str> = CUES.iter().map(|c| c.0).collect();
        let expected: Vec<String> = (1..=8).map(|k| format!("08_{k}")).collect();
        assert_eq!(ids, expected);
        assert_eq!(sum_tex(&BoxFacts::solve().imbalance), "1 - 1 + 0 + 0");
        assert_eq!(signed_tex(Q::ONE), "+1");
        assert_eq!(signed_tex(-Q::ONE), "-1");
    }
}
