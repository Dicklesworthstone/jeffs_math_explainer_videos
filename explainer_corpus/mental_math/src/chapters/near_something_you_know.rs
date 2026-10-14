//! Chapter 14, near something you know: the last quiz of the mixed round,
//! fifty-five squared (14_1), which two tricks fit and must agree on (14_2);
//! the one move behind both, made three times, on 35, 48 and 50 (14_3); the
//! three sums as one line in letters, the middle trick's line with the small
//! square moved across (14_4); the video's own numbers on a number line,
//! each near a round one, and Feynman's sentence turned to the viewer
//! (14_5); and the receipts (14_6).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::receipts_card;
use explainer_kit::{Kit, Palette, Timed, children, fade_all, ink_style, play, v};
use fmn::library::{SpanMapData, TransformMatchingTex};
use fmn::prelude::*;

use super::draw::{
    Board, MoveFigure, NumberLine1, QuizLook, TURN_S, Tile, TileKind, Turn, WHICH_TRICK, corner, eq, fade_in_all,
    fade_out_all, faded_line, fail, fly, glyphs, hundreds, line_style, middle_of, mover, num, pause, rgb, say, slate,
    strip, swap, tiles_top, wait_until, your_turn,
};
use super::sound;
use crate::facts::{cited, shown};
use crate::model::{FIFTY, OneMove, ends_in_five, estimate, near_fifty, one_move};

pub struct NearSomethingYouKnow {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const THE_MIDDLE: &str = "the middle";
const THE_DISTANCE: &str = "the distance";
/// Feynman's sentence as 11_4 quotes it, its source, and what it becomes
/// once it is said to the viewer.
const FEYNMAN_LINE: &str = "“It was easy for him—every number was near something he knew.”";
const FEYNMAN_ON: &str = "Richard P. Feynman, on Hans Bethe";
const BOOK: &str = "“Surely You’re Joking, Mr. Feynman!”";
const BOOK_CHAPTER: &str = "“Lucky Numbers”";
const NEAR_LINE: &str = "every number is near something you know.";
const AFTER_FEYNMAN: &str = "after Feynman";
/// The receipts.
const RECEIPT_CHECKED: &str = "every calculation in this video was checked by two independent programs";
const RECEIPT_REPO: &str = "github.com/Dicklesworthstone/jeffs_math_explainer_videos";
const RECEIPT_NAMES: &str = "names to search: the distributive law, difference of squares, the area model";
const RECEIPT_CHASE: &str = "three of these tricks are on one page of a school arithmetic of";
const RECEIPT_CHASE_PAGE: &str = "Chase, p.";
/// The one line in letters, and the middle trick's line (08_5) above it.
const LETTERS: &str = r"a^2 = (a - b)(a + b) + b^2";
const MIDDLE_LINE: &str = r"(a + b)(a - b) = a^2 - b^2";

/// The quiz card at the left, under the tiles, with the working beside it.
const CARD_AT: Vec3 = [-3.3, -0.45, 0.0];
const CARD_SCALE: f64 = 0.8;
const ROW_PT: f64 = 72.0;
/// The three moves: their baseline, the gap between them (units), the
/// lines under them, and the widest a line may be.
const MOVES_Y: f64 = -2.35;
const MOVES_GAP: f64 = 8.0;
const MOVES_ROOM: (f64, f64) = (13.7, 6.15);
const SUM_PT: f64 = 44.0;
/// The line in letters, the middle trick's line, and the names under a and b.
const LETTERS_PT: f64 = 96.0;
/// The three lines stacked in the middle before they become one.
const STACK_PT: f64 = 64.0;
const STACK_Y: [f64; 3] = [1.15, -0.2, -1.55];
const LETTERS_AT: Vec3 = [0.0, -0.2, 0.0];
const MIDDLE_PT: f64 = 60.0;
const NAME_PT: f64 = 34.0;
/// The number line of 14_5 and the sentence above it.
const LINE_Y: f64 = -1.7;
const NUMBER_PT: f64 = 48.0;
const QUOTE_PT: f64 = 36.0;
const QUOTE_ROOM: f64 = 13.0;
const QUOTE_Y: f64 = 1.85;
const SOURCE_PT: f64 = 24.0;

/// Which parts of a tile's picture are fills (the rest are lines), read
/// before anything dims them.
fn fills_of(stage: &Stage<'_>, tile: &Tile) -> Vec<bool> {
    tile.parts
        .iter()
        .map(|&p| stage.get_fill_opacity(p).unwrap_or(0.0) > 0.0)
        .collect()
}

/// A tile lit: its frame in ink and its picture at full strength, each
/// part by the alpha it is drawn with. Animations to play together.
fn tile_light(stage: &mut Stage<'_>, tile: &Tile, fills: &[bool], rt: f64) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let args = || AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    };
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    let frame = tile
        .frame
        .animate()
        .set_anim_args(args())?
        .set_stroke(Some(Palette::ink()), None, Some(1.0))?;
    anims.push(stage.prepare(frame)?);
    for (&p, &fill) in tile.parts.iter().zip(fills) {
        let a = p.animate().set_anim_args(args())?;
        let a = if fill {
            a.set_fill(None, Some(1.0))?
        } else {
            a.set_stroke(None, None, Some(1.0))?
        };
        anims.push(stage.prepare(a)?);
    }
    Ok(anims)
}

/// The move on `side` with a strip `deep`, or a scene error.
/// The recall after 14_3's three sums (storyboard: "listen 2 s, the move's
/// sound, three times"): the window carries the move's pluck, and at each of
/// its three plucks one square's strip, standing against its side, hops once,
/// in the order the squares came. The narrator is silent; the scene fills the
/// window.
fn recall(kit: &Kit, stage: &mut Stage<'_>, figures: &[MoveFigure]) -> fmn::Result<()> {
    let wav = sound::the_move_thrice()?;
    kit.sonify(stage, sound::RECALL_S, "14_near_something_you_know:14_3", &wav)?;
    let start = stage.scene().time().to_f64();
    for (i, (figure, &at)) in figures.iter().zip(&sound::RECALL_AT).enumerate() {
        wait_until(stage, start + at)?;
        #[cfg(test)]
        super::draw::probe::record(&kit.chapter, "hop", i, stage.scene().time().to_f64() - start, 1.0);
        figure.strip.hop(stage)?;
        #[cfg(test)]
        super::draw::probe::record(
            &kit.chapter,
            "rest",
            i,
            stage.scene().time().to_f64() - start,
            stage.tracker_value(figure.strip.t).unwrap_or(f64::NAN),
        );
        #[cfg(not(test))]
        let _ = i;
    }
    wait_until(stage, start + sound::RECALL_S)
}

fn the_move_of(side: u64, deep: u64) -> fmn::Result<OneMove> {
    one_move(side, deep).ok_or_else(|| fail(format!("no strip {deep} deep comes off a square of {side}")))
}

/// How far right and how far up (units, from the square's lower-left
/// corner) the strip of `m` reaches at any moment of its turn.
fn reach(m: OneMove) -> (f64, f64) {
    let turn = Turn::of(m);
    let (mut x, mut y) = (m.wide() as f64, m.side as f64);
    for k in 0..=200 {
        for [px, py] in turn.quad(f64::from(k) / 200.0) {
            x = x.max(px);
            y = y.max(py);
        }
    }
    (x, y)
}

/// A line of mathematics in two pieces, the head and the tail, each set
/// where it sits in the whole `head tail` (the whole itself is only
/// measured), so the tail can give way to another on the same baseline.
/// The whole's left edge is at `left`.
struct Split {
    head: Mob,
    tail: Mob,
}

/// Where the glyphs of `head` and of `tail` sit in `head tail`, set at `pt`
/// with its left edge at `left`: the middle of each, and how many glyphs the
/// head has.
fn split_places(
    kit: &Kit,
    stage: &mut Stage<'_>,
    head: &str,
    tail: &str,
    pt: f64,
    left: Vec3,
) -> fmn::Result<(Vec3, Vec3, Mob)> {
    let whole = eq(kit, stage, &format!("{head} {tail}"), pt, left)?;
    stage.move_to(whole, left, LEFT);
    let all = glyphs(stage, whole);
    let probe = eq(kit, stage, tail, pt, ORIGIN)?;
    let n = glyphs(stage, probe).len();
    if all.len() <= n {
        return Err(fail(format!("{head} {tail} has no head before its tail")));
    }
    let k = all.len() - n;
    Ok((middle_of(stage, &all[..k])?, middle_of(stage, &all[k..])?, whole))
}

/// `head tail` in two pieces, the tail in `tail_color`. Detached.
fn split_line(
    kit: &Kit,
    stage: &mut Stage<'_>,
    head: &str,
    tail: &str,
    pt: f64,
    left: Vec3,
    tail_color: Srgb,
) -> fmn::Result<Split> {
    let (head_at, tail_at, _) = split_places(kit, stage, head, tail, pt, left)?;
    // the head ends in a plus: an empty group after it keeps its spacing
    let head = eq(kit, stage, &format!("{head} {{}}"), pt, head_at)?;
    let tail = kit.tex(stage, tail, pt, tail_color, &[], tail_at)?;
    Ok(Split { head, tail })
}

/// The type size at which `src` is at most `room` wide (at most `pt`).
fn fitting_pt(kit: &Kit, stage: &mut Stage<'_>, src: &str, pt: f64, room: f64) -> fmn::Result<f64> {
    let probe = eq(kit, stage, src, pt, ORIGIN)?;
    let w = stage.get_width(probe);
    Ok(if w > room { pt * room / w } else { pt })
}

/// The span map with entry `index` keyed `key` instead of its source text,
/// so `TransformMatchingTex` does not pair it.
fn rekey(spans: &SpanMapData, index: usize, key: &str) -> SpanMapData {
    let mut source = spans.source.to_string();
    let start = source.len();
    source.push_str(key);
    let mut entries = spans.entries.clone();
    if let Some(e) = entries.get_mut(index) {
        e.start = start;
        e.end = source.len();
    }
    SpanMapData {
        source: source.into(),
        entries,
    }
}

/// The source text of each glyph of a span map.
fn keys(spans: &SpanMapData) -> Vec<&str> {
    spans
        .entries
        .iter()
        .map(|e| spans.source.get(e.start..e.end).unwrap_or(""))
        .collect()
}

/// The words of a sentence as byte ranges: runs of letters and marks, an
/// opening or closing quotation mark on its own, a dash ending its word.
fn tokens(s: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in s.char_indices() {
        let end = i + c.len_utf8();
        if c.is_whitespace() || c == '“' || c == '”' {
            if let Some(s0) = start.take() {
                out.push((s0, i));
            }
            if !c.is_whitespace() {
                out.push((i, end));
            }
            continue;
        }
        let s0 = *start.get_or_insert(i);
        if c == '—' {
            out.push((s0, end));
            start = None;
        }
    }
    if let Some(s0) = start {
        out.push((s0, s.len()));
    }
    out
}

/// A sentence set as one italic text, and where each of its words sits.
struct Sentence {
    whole: Mob,
    /// Each word's text and the middle of its glyphs.
    words: Vec<(String, Vec3)>,
}

impl Sentence {
    /// `s` in italic at `pt`, centred at `at`. Detached.
    fn build(kit: &Kit, stage: &mut Stage<'_>, s: &str, pt: f64, at: Vec3) -> fmn::Result<Self> {
        let whole = kit.text_styled(stage, s, pt, Palette::ink(), at, false, true)?;
        // the same text built again only for its span map: which source
        // bytes each glyph comes from
        let spans = Text::new(s)
            .font_size(pt)
            .bold(false)
            .italic(true)
            .style(ink_style(Palette::ink()))
            .build(&kit.book)
            .map_err(|e| fail(format!("setting {s:?}: {e}")))?
            .span_map();
        let all = glyphs(stage, whole);
        if all.len() < spans.entries.len() {
            return Err(fail(format!("{s:?} has fewer glyphs than its span map")));
        }
        let mut words = Vec::new();
        for (a, b) in tokens(s) {
            let mine: Vec<Mob> = spans
                .entries
                .iter()
                .zip(&all)
                .filter(|(e, _)| e.start >= a && e.start < b)
                .map(|(_, &g)| g)
                .collect();
            words.push((s[a..b].to_owned(), middle_of(stage, &mine)?));
        }
        Ok(Self { whole, words })
    }

    /// Each word as its own text, where it sits in the sentence. Detached.
    fn word_mobs(&self, kit: &Kit, stage: &mut Stage<'_>, pt: f64) -> fmn::Result<Vec<Mob>> {
        self.words
            .iter()
            .map(|(w, at)| kit.text_styled(stage, w, pt, Palette::ink(), *at, false, true))
            .collect()
    }
}

impl SceneConstruct for NearSomethingYouKnow {
    fn name(&self) -> &str {
        "14_near_something_you_know"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, orange, ink) = (hundreds(), corner(), Palette::ink());
        let twist = shown::TWIST;
        let five = ends_in_five(twist).ok_or_else(|| fail(format!("{twist} does not end in five")))?;
        let near = near_fifty(twist);
        let m55 = the_move_of(twist, twist % 10)?;
        if five.hundreds as i64 != near.hundreds || near.corner != m55.corner() {
            return Err(fail(format!("the two tricks disagree on {twist} squared")));
        }

        // 14_1: the tiles along the top, there from the first frame and at
        // rest, as chapter 13 leaves them (none hints at the answer during
        // the ring: R2, M14); the quiz card under them, at the left
        let tiles = tiles_top(stage)?;
        let fills: Vec<Vec<bool>> = tiles.iter().map(|t| fills_of(stage, t)).collect();
        for t in &tiles {
            for m in t.mobs() {
                stage.add_to_scene(m)?;
            }
        }
        let look = QuizLook {
            at: CARD_AT,
            label: Some(WHICH_TRICK),
            pause_line: true,
            lands: true,
            scale: CARD_SCALE,
        };
        let card = your_turn(&kit, stage, "14_1", look)?;

        // 14_2: two tiles light together; two rows, the same 30 and the
        // same 25 one under the other; both fly into the 3025 on the card
        let line = say(&kit, stage, "14_2")?;
        let mut anims = Vec::new();
        for (t, f) in tiles.iter().zip(&fills) {
            if matches!(t.kind, TileKind::Strip | TileKind::Fifty) {
                anims.extend(tile_light(stage, t, f, 0.6)?);
            } else {
                anims.extend(t.light(stage, false, 0.6)?);
            }
        }
        stage.play_prepared(anims)?;
        let lead_srcs = [
            format!(r"{} \times {} =", five.n, five.n + 1),
            format!("{} + {} =", near.hundreds - near.distance, near.distance),
        ];
        let leads: Vec<Mob> = lead_srcs
            .iter()
            .map(|s| eq(&kit, stage, s, ROW_PT, ORIGIN))
            .collect::<fmn::Result<_>>()?;
        let front_src = num(five.hundreds);
        let back_src = format!("{:02}", near.corner);
        let fronts = [
            kit.tex(stage, &front_src, ROW_PT, blue, &[], ORIGIN)?,
            kit.tex(stage, &near.hundreds.to_string(), ROW_PT, blue, &[], ORIGIN)?,
        ];
        let backs = [
            kit.tex(stage, &num(m55.corner()), ROW_PT, orange, &[], ORIGIN)?,
            kit.tex(stage, &back_src, ROW_PT, orange, &[], ORIGIN)?,
        ];
        let lead_w = leads.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
        let (gap, pair_gap) = (0.3, 0.32);
        let row_w = lead_w + gap + stage.get_width(fronts[0]) + pair_gap + stage.get_width(backs[0]);
        let right_of_lead = 3.4 - row_w / 2.0 + lead_w;
        for (i, y) in [0.55, -0.95].into_iter().enumerate() {
            stage.move_to(leads[i], v(right_of_lead, y), RIGHT);
            stage.move_to(fronts[i], v(right_of_lead + gap, y), LEFT);
            let after = stage.get_right(fronts[i])[0] + pair_gap;
            stage.move_to(backs[i], v(after, y), LEFT);
        }
        line.wait_sentence(stage, 1)?;
        play!(stage;
            fade_in(stage.arena_mut(), leads[0], v(0.0, -0.12), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), fronts[0], v(0.0, -0.12), 1.0)?.rt(0.5),
        );
        line.wait_for(stage, "then twenty-five")?;
        play!(stage; fade_in(stage.arena_mut(), backs[0], v(0.3, 0.0), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 3)?;
        play!(stage;
            fade_in(stage.arena_mut(), leads[1], v(0.0, -0.12), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), fronts[1], v(0.0, -0.12), 1.0)?.rt(0.5),
        );
        // its "then twenty-five" is the second in the line
        let (s3, s4) = (line.sentence(3)?, line.sentence(4)?);
        wait_until(stage, s3 + 0.62 * (s4 - s3))?;
        play!(stage; fade_in(stage.arena_mut(), backs[1], v(0.3, 0.0), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 4)?;
        // copies of both rows' numbers fly up into the answer already on
        // the card, which takes their colours as they arrive
        let digits = glyphs(stage, card.answer);
        let split = front_src.len();
        if digits.len() <= split {
            return Err(fail(format!("{} has no digits after its hundreds", m55.square())));
        }
        let (lead_at, tail_at) = (middle_of(stage, &digits[..split])?, middle_of(stage, &digits[split..])?);
        let mut copies = Vec::new();
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for i in 0..2 {
            for (m, src, color, to) in [
                (fronts[i], &front_src, blue, lead_at),
                (backs[i], &back_src, orange, tail_at),
            ] {
                let c = kit.tex(stage, src, ROW_PT, color, &[], stage.get_center(m))?;
                stage.add_to_scene(c)?;
                moves.push(stage.prepare(fly(c, to, look.pt() / ROW_PT, 0.9)?)?);
                copies.push(c);
            }
        }
        stage.play_prepared(moves)?;
        for &g in &digits[..split] {
            stage.set_color(g, blue, None, true);
        }
        for &g in &digits[split..] {
            stage.set_color(g, orange, None, true);
        }
        for c in copies {
            stage.remove_from_scene(c);
        }

        // 14_3: the same move, three times, one square after another, each
        // as its sum is spoken, on one common unit
        let line = say(&kit, stage, "14_3")?;
        let mut leaving: Vec<Mob> = tiles.iter().flat_map(|t| t.mobs()).collect();
        leaving.extend(card.mobs());
        leaving.extend(leads.iter().chain(&fronts).chain(&backs));
        fade_out_all(stage, &leaving, 0.6)?;
        // 35 with its strip of five; Bethe's 48, two below fifty; and fifty
        // as the middle of 47 and 53, three from each
        let below = shown::BELOW;
        let moves = [
            the_move_of(shown::FIVE, shown::FIVE % 10)?,
            the_move_of(below, FIFTY - below)?,
            the_move_of(FIFTY, shown::PAIR.1 - FIFTY)?,
        ];
        // the squares come one at a time, so a strip turning may pass over
        // the room of the next square; the last needs its whole reach
        let last = moves[2];
        let (reach_x, reach_y) = reach(last);
        let span: f64 = moves[..2].iter().map(|m| m.wide() as f64 + MOVES_GAP).sum::<f64>() + reach_x;
        let unit = (MOVES_ROOM.0 / span).min(MOVES_ROOM.1 / reach_y);
        let mut x = -MOVES_ROOM.0 / 2.0;
        let mut figures = Vec::new();
        let mut sums = Vec::new();
        for (i, &m) in moves.iter().enumerate() {
            let board = Board::new(v(x, MOVES_Y), unit);
            // the first square is sliced on screen; the other two arrive
            // already in their roles' colours
            let (body, piece) = if i == 0 { (slate(), slate()) } else { (blue, strip()) };
            figures.push(MoveFigure::build(stage, board, m, body, piece, 0.0)?);
            let head = format!(r"{}^2 = {} \times {} +", m.side, m.tall(), m.wide());
            let tail = num(m.corner());
            let next_x = x + (m.wide() as f64 + MOVES_GAP) * unit;
            let room = if i < 2 {
                next_x - x - 0.2
            } else {
                MOVES_ROOM.0 / 2.0 - x
            };
            let pt = fitting_pt(&kit, stage, &format!("{head} {tail}"), SUM_PT, room)?;
            // centred under the rectangle the move makes, but never further
            // left than its square nor into the next one's room
            let probe = eq(&kit, stage, &format!("{head} {tail}"), pt, ORIGIN)?;
            let w = stage.get_width(probe);
            let centre = x + m.wide() as f64 * unit / 2.0;
            let left = v((centre - w / 2.0).min(x + room - w).max(x), MOVES_Y - 0.55);
            sums.push((split_line(&kit, stage, &head, &tail, pt, left, orange)?, head, pt, left));
            x = next_x;
        }
        let first = &figures[0];
        fade_in_all(stage, &first.square(), 0.5)?;
        line.wait_sentence(stage, 1)?;
        first.slice(stage, 0.6)?;
        for (i, figure) in figures.iter().enumerate() {
            line.wait_sentence(stage, 2 + i)?;
            if i > 0 {
                let mut ins = figure.square();
                ins.push(figure.cut);
                fade_in_all(stage, &ins, 0.4)?;
            }
            stage.remove_from_scene(figure.cut);
            stage
                .scene_mut()
                .bring_to_front(&[figure.strip.fill, figure.strip.edge])?;
            let (split, ..) = &sums[i];
            play!(stage;
                figure.strip.turn_to(1.0)?,
                fade_in(stage.arena_mut(), split.head, ORIGIN, 1.0)?.rt(TURN_S),
                fade_in(stage.arena_mut(), split.tail, ORIGIN, 1.0)?.rt(TURN_S),
            );
            stage.add_to_scene(figure.seam)?;
            figure.show_corner(stage, 0.3)?;
        }
        recall(&kit, stage, &figures)?;

        // 14_4: the 25, the 4 and the 9 become 5², 2² and 3²; the first
        // line turns into letters and the other two cross-fade into it; the
        // middle trick's line above
        let line = say(&kit, stage, "14_4")?;
        // the pictures go and the three lines come together in the middle,
        // one under the other, larger, their equals signs in a column
        let finals: Vec<(String, String)> = moves
            .iter()
            .zip(&sums)
            .map(|(m, (_, head, ..))| (head.clone(), format!("{}^2", m.strip)))
            .collect();
        let mut stack_w: f64 = 0.0;
        for (head, squared) in &finals {
            let probe = eq(&kit, stage, &format!("{head} {squared}"), STACK_PT, ORIGIN)?;
            stack_w = stack_w.max(stage.get_width(probe));
        }
        let stack_left: Vec<Vec3> = STACK_Y.iter().map(|&y| v(-stack_w / 2.0, y)).collect();
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for f in &figures {
            // what of each figure is on stage (its cut and its merged
            // rectangle never came in)
            let mut on_stage = f.square();
            on_stage.extend([f.seam, f.corner, f.corner_cut]);
            for m in on_stage {
                stage.clear_updaters(m, true);
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.6))?);
            }
        }
        for ((m, (split, head, pt, _)), left) in moves.iter().zip(&sums).zip(&stack_left) {
            let tail = num(m.corner());
            let (head_at, tail_at, _) = split_places(&kit, stage, head, &tail, STACK_PT, *left)?;
            let k = STACK_PT / pt;
            anims.push(stage.prepare(fly(split.head, head_at, k, 0.8)?)?);
            anims.push(stage.prepare(fly(split.tail, tail_at, k, 0.8)?)?);
        }
        stage.play_prepared(anims)?;
        // the 25, the 4 and the 9 give way to 5², 2² and 3² in place
        let mut news = Vec::new();
        for ((head, squared), left) in finals.iter().zip(&stack_left) {
            let (_, tail_at, _) = split_places(&kit, stage, head, squared, STACK_PT, *left)?;
            news.push(kit.tex(stage, squared, STACK_PT, orange, &[], tail_at)?);
        }
        let olds: Vec<Mob> = sums.iter().map(|(split, ..)| split.tail).collect();
        swap(stage, &olds, &news, 0.5)?;
        // the first line as one formula, with its span map, in place of
        // its two pieces (the same glyphs where they already are)
        let (first_head, first_squared) = &finals[0];
        let (source, source_spans) = kit.tex_spans(
            stage,
            &format!("{first_head} {first_squared}"),
            STACK_PT,
            ink,
            &[],
            ORIGIN,
        )?;
        stage.move_to(source, stack_left[0], LEFT);
        let n = glyphs(stage, source).len();
        for &g in &glyphs(stage, source)[n - 2..] {
            stage.set_color(g, orange, None, true);
        }
        stage.remove_from_scene(sums[0].0.head);
        stage.remove_from_scene(news[0]);
        stage.add_to_scene(source)?;
        let (letters, letter_spans) = kit.tex_spans(stage, LETTERS, LETTERS_PT, ink, &[], LETTERS_AT)?;
        let letter_glyphs = glyphs(stage, letters);
        let letter_keys = keys(&letter_spans);
        if letter_glyphs.len() < letter_keys.len() || letter_keys.len() < 2 {
            return Err(fail(format!("{LETTERS} has fewer glyphs than its span map")));
        }
        let k = letter_keys.len();
        for &g in &letter_glyphs[k - 2..k] {
            stage.set_color(g, orange, None, true);
        }
        // the plus inside (a + b) must not take the plus before the small
        // square: only the last plus pairs with the 35 line's
        let inner_plus = letter_keys
            .iter()
            .position(|&s| s == "+")
            .ok_or_else(|| fail(format!("{LETTERS} has no plus")))?;
        let letter_spans = rekey(&letter_spans, inner_plus, "inner +");
        let morph = TransformMatchingTex::new(stage.arena_mut(), source, letters, &source_spans, &letter_spans)
            .map_err(|e| fail(format!("the 35 line into letters: {e}")))?;
        // the other two lines move onto it as they fade
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(morph.rt(1.4))?];
        for (i, (split, ..)) in sums.iter().enumerate().skip(1) {
            for m in [split.head, news[i]] {
                let c = stage.get_center(m);
                let a = fade_out(stage.arena_mut(), m, v(0.0, LETTERS_AT[1] - c[1]), 1.0)?;
                anims.push(stage.prepare(a.rt(1.0))?);
            }
        }
        stage.play_prepared(anims)?;
        // the middle and the distance under the first a and the last b
        let first_a = letter_keys
            .iter()
            .position(|&s| s == "a")
            .ok_or_else(|| fail(format!("{LETTERS} has no a")))?;
        let last_b = letter_keys
            .iter()
            .rposition(|&s| s == "b")
            .ok_or_else(|| fail(format!("{LETTERS} has no b")))?;
        let under = stage.get_bottom(letters)[1] - 0.45;
        let names = [
            kit.text(
                stage,
                THE_MIDDLE,
                NAME_PT,
                ink,
                v(stage.get_center(letter_glyphs[first_a])[0], under),
            )?,
            kit.text(
                stage,
                THE_DISTANCE,
                NAME_PT,
                ink,
                v(stage.get_center(letter_glyphs[last_b])[0], under),
            )?,
        ];
        // the middle trick's line in ink and large enough to recognise (it
        // was a lower-third at 08_5: R2, M17)
        let middle = kit.tex(stage, MIDDLE_LINE, MIDDLE_PT, ink, &[], v(0.0, 1.55))?;
        line.wait_for(stage, "the middle trick")?;
        fade_in_all(stage, &[middle, names[0], names[1]], 0.6)?;
        line.wait_for(stage, "the small square")?;
        // the small square and its plus, moved across from the other side
        let last_plus = letter_keys
            .iter()
            .rposition(|&s| s == "+")
            .ok_or_else(|| fail(format!("{LETTERS} has no plus")))?;
        // and the minus b squared of the middle line it came from, pulsed
        // with it (its last three glyphs: the minus, b and the 2)
        let middle_glyphs = glyphs(stage, middle);
        let moved_from = &middle_glyphs[middle_glyphs.len().saturating_sub(3)..];
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for &g in letter_glyphs[last_plus..k].iter().chain(moved_from) {
            let a = fmn::animation::indicate(stage.arena_mut(), g, 1.2, Some(rgb(orange)))?;
            pulse.push(stage.prepare(a.rt(1.0))?);
        }
        stage.play_prepared(pulse)?;
        kit.music_breath(stage, HOLD_S)?;

        // 14_5: the video's numbers on a number line, each near the round
        // number it used; Feynman's sentence returns and three words change
        let line = say(&kit, stage, "14_5")?;
        fade_all(stage, 0.5)?;
        let number_line = NumberLine1 {
            x0: -6.5,
            x1: 6.5,
            y: LINE_Y,
            lo: 0.0,
            hi: 110.0,
        };
        let axis = number_line.axis(stage);
        let mut tens = Vec::new();
        for t in (0..=110u64).step_by(10) {
            let brightest = t > 0 && t % FIFTY == 0;
            let (h, c, pt, below) = if brightest {
                (0.56, ink, 42.0, 0.2)
            } else {
                (0.32, Palette::dim(), 30.0, 0.06)
            };
            tens.push(number_line.tick(stage, t as f64, h, c));
            let label = number_line.label(&kit, stage, &num(t), t as f64, pt, c, false)?;
            stage.shift(label, v(0.0, -below));
            tens.push(label);
        }
        play!(stage; fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.5));
        super::draw::fade_in_lagged(stage, &tens, 1.0, 0.3)?;
        // each number and the round number the video used for it: the
        // estimate's for the products, fifty for the others
        let (parts, pair, nothing) = (shown::PARTS, shown::PAIR, shown::NOTHING);
        let (pr, nr) = (estimate(parts.0, parts.1), estimate(nothing.0, nothing.1));
        let near_ones = [
            (parts.0, pr.0),
            (parts.1, pr.1),
            (pair.0, FIFTY),
            (pair.1, FIFTY),
            (below, FIFTY),
            (nothing.0, nr.0),
            (nothing.1, nr.1),
        ];
        let halfway = [shown::FIVE, shown::HOOK];
        // the labels stack where they would cover one another, in the order
        // they land
        let mut placed: Vec<(f64, f64)> = Vec::new();
        let mut spot = |x: f64| -> f64 {
            let mut y = LINE_Y + 0.64;
            while placed
                .iter()
                .any(|&(px, py)| (px - x).abs() < 0.62 && (py - y).abs() < 0.46)
            {
                y += 0.5;
            }
            placed.push((x, y));
            y
        };
        let mut drops: Vec<(Vec<Mob>, Vec<Mob>)> = Vec::new();
        for &(n, round) in &near_ones {
            let x = number_line.p(n as f64)[0];
            let label = kit.tex(stage, &num(n), NUMBER_PT, ink, &[], v(x, spot(x)))?;
            let dot = number_line.dot(stage, n as f64, 0.065, ink);
            let d = (n as f64 - round as f64).abs();
            let bow = number_line.bow(stage, n as f64, round as f64, 0.12 + 0.06 * d, line_style(mover(), 3.5));
            drops.push((vec![label, dot], vec![bow]));
        }
        for &n in &halfway {
            let x = number_line.p(n as f64)[0];
            let label = kit.tex(stage, &num(n), NUMBER_PT, Palette::dim(), &[], v(x, spot(x)))?;
            let dot = number_line.dot(stage, n as f64, 0.065, Palette::dim());
            let ten = n - n % 10;
            let faint = faded_line(Palette::dim(), 2.5, 0.8);
            let bows = vec![
                number_line.bow(stage, n as f64, ten as f64, 0.24, faint),
                number_line.bow(stage, n as f64, (ten + 10) as f64, 0.24, faint),
            ];
            drops.push((vec![label, dot], bows));
        }
        line.wait_sentence(stage, 1)?;
        // each lands and throws its bow while the next one falls
        for k in 0..=drops.len() {
            let mut step: Vec<Box<dyn Animation>> = Vec::new();
            if let Some((falling, _)) = drops.get(k) {
                for &m in falling {
                    let a = fade_in(stage.arena_mut(), m, v(0.0, -0.45), 1.0)?;
                    step.push(stage.prepare(a.rt(0.3))?);
                }
            }
            if let Some((_, bows)) = k.checked_sub(1).and_then(|j| drops.get(j)) {
                for &b in bows {
                    step.push(stage.prepare(show_creation(b).rt(0.3))?);
                }
            }
            // the bow before it fades back, so the newest one is the bright
            // one where they crowd round fifty (R2, M18); a stroke's own
            // opacity, so a bow never fills
            if let Some((_, older)) = k.checked_sub(2).and_then(|j| drops.get(j)) {
                for &b in older {
                    let a = b
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.3),
                            ..AnimateArgs::default()
                        })?
                        .set_stroke(None, None, Some(0.35))?;
                    step.push(stage.prepare(a)?);
                }
            }
            stage.play_prepared(step)?;
        }
        // Feynman's sentence, as 11_4 quoted it, with its source, on one
        // line that fits the frame
        let probe = kit.text_styled(stage, FEYNMAN_LINE, QUOTE_PT, ink, ORIGIN, false, true)?;
        let pt = QUOTE_PT * (QUOTE_ROOM / stage.get_width(probe)).min(1.0);
        let old = Sentence::build(&kit, stage, FEYNMAN_LINE, pt, v(0.0, QUOTE_Y))?;
        let source_line = format!("{BOOK} ({}), {BOOK_CHAPTER}", cited::FEYNMAN_YEAR);
        let sources = [
            kit.text(stage, FEYNMAN_ON, SOURCE_PT, Palette::dim(), v(0.0, QUOTE_Y - 0.62))?,
            kit.text(stage, &source_line, SOURCE_PT, Palette::dim(), v(0.0, QUOTE_Y - 0.98))?,
        ];
        let after = kit.text(stage, AFTER_FEYNMAN, SOURCE_PT, Palette::dim(), v(0.0, QUOTE_Y - 0.62))?;
        let mut quote = vec![old.whole];
        quote.extend(sources);
        fade_in_all(stage, &quote, 0.6)?;
        line.wait_sentence(stage, 2)?;
        // its words as separate texts where they stand; the quotation marks
        // and the first clause go, the source shortens
        let old_words = old.word_mobs(&kit, stage, pt)?;
        stage.remove_from_scene(old.whole);
        for &w in &old_words {
            stage.add_to_scene(w)?;
        }
        let clause_at = old
            .words
            .iter()
            .position(|(w, _)| w.as_str() == "every")
            .ok_or_else(|| fail(format!("{FEYNMAN_LINE} has no clause starting with every")))?;
        let n_words = old.words.len();
        let mut going: Vec<Mob> = old_words[..clause_at].to_vec();
        going.push(old_words[n_words - 1]);
        let mut out = going.clone();
        out.extend(sources);
        fade_out_all(stage, &out, 0.5)?;
        // the clause in its new words: three change in place, the rest
        // slide to where the new sentence puts them, on the same baseline
        let new = Sentence::build(&kit, stage, NEAR_LINE, pt, v(0.0, QUOTE_Y))?;
        let clause = &old.words[clause_at..n_words - 1];
        if clause.len() != new.words.len() {
            return Err(fail(format!(
                "{NEAR_LINE:?} is not {FEYNMAN_LINE:?} with words changed"
            )));
        }
        let dy = clause[0].1[1] - new.words[0].1[1];
        stage.shift(new.whole, v(0.0, dy));
        let new_words: Vec<(String, Vec3)> = new
            .words
            .iter()
            .map(|(w, at)| (w.clone(), v(at[0], at[1] + dy)))
            .collect();
        // the source, shortened, comes back with the new words
        let a = fade_in(stage.arena_mut(), after, ORIGIN, 1.0)?;
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(a.rt(0.9))?];
        let mut changed = 0;
        let mut shown_words = Vec::new();
        for (j, ((word, _), (new_word, to))) in clause.iter().zip(&new_words).enumerate() {
            let m = old_words[clause_at + j];
            if word == new_word {
                anims.push(stage.prepare(fly(m, *to, 1.0, 0.9)?)?);
            } else {
                changed += 1;
                let a = fade_out(stage.arena_mut(), m, v(0.0, 0.15), 1.0)?;
                anims.push(stage.prepare(a.rt(0.9))?);
                let w = kit.text_styled(stage, new_word, pt, ink, *to, false, true)?;
                let a = fade_in(stage.arena_mut(), w, v(0.0, 0.15), 1.0)?;
                anims.push(stage.prepare(a.rt(0.9))?);
                shown_words.push(w);
            }
            shown_words.push(m);
        }
        if changed != 3 {
            return Err(fail(format!("{changed} words change, not three")));
        }
        stage.play_prepared(anims)?;
        for w in shown_words.into_iter().chain(going) {
            stage.remove_from_scene(w);
        }
        stage.add_to_scene(new.whole)?;
        kit.music_breath(stage, HOLD_S)?;

        // 14_6: the receipts, and the motif answered
        let chase = format!(
            "{RECEIPT_CHASE} {} ({RECEIPT_CHASE_PAGE} {})",
            cited::CHASE_YEAR,
            cited::CHASE_PAGE
        );
        let lines = [RECEIPT_CHECKED, RECEIPT_REPO, RECEIPT_NAMES, chase.as_str()];
        let receipts = receipts_card(&kit, stage, &lines)?;
        // a line that runs past the card is an error, never a clipped word
        let parts_of = children(stage, receipts);
        let edge = stage.get_right(parts_of[0])[0] - 0.3;
        for (&m, l) in parts_of[2..].iter().zip(&lines) {
            if stage.get_right(m)[0] > edge {
                return Err(fail(format!("a receipts line does not fit its card: {l}")));
            }
        }
        // the 1844 page is small print: dimmer, and set a little apart
        if let Some(&last) = parts_of.last() {
            stage.set_color(last, Palette::dim(), None, true);
            stage.shift(last, v(0.0, -0.12));
        }
        fade_all(stage, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), receipts, v(0.0, -0.1), 1.0)?.rt(0.8));
        kit.listen(stage, 4.0, "14_near_something_you_know:14_6")?;
        pause(stage, 0.2)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{probe, testing};
    use super::*;

    /// 14_3's recall (bead mx-e-a-f-ms4.10): the listen window after the
    /// three sums carries the move's pluck, and one square's strip hops at
    /// each of its three plucks, in the order the squares came; the picture
    /// is not still through the window, as a bare listen window would be.
    #[test]
    fn the_recall_hops_a_strip_at_each_of_the_three_plucks() {
        let key = "14_near_something_you_know";
        let run = testing::run_voiced(key).unwrap();
        let (_, start, seconds, sounded) = run
            .listens
            .iter()
            .find(|(label, ..)| label.ends_with(":14_3"))
            .cloned()
            .expect("14_3 opens a listen window");
        assert!(sounded, "the window carries a sonification");
        assert!((seconds - sound::RECALL_S).abs() < 1e-9, "{seconds} s");
        let tag = format!("{key}#voiced");
        let hops = probe::take(&tag, "hop");
        assert_eq!(hops.iter().map(|h| h.0).collect::<Vec<_>>(), [0, 1, 2], "{hops:?}");
        let frame = 1.0 / f64::from(testing::FPS);
        for &(i, at, _) in &hops {
            let want = sound::RECALL_AT[i];
            assert!(
                (at - want).abs() <= frame + 1e-9,
                "square {i} hops {at:.2} s into the window; its pluck is at {want} s"
            );
        }
        // every hop is over before the next pluck, and each strip stands again
        let rests = probe::take(&tag, "rest");
        assert_eq!(rests.len(), 3);
        for (&(i, hop_at, _), &(j, rest_at, standing)) in hops.iter().zip(&rests) {
            assert_eq!(i, j);
            assert_eq!(standing, 1.0, "square {i} does not end standing");
            assert!(
                rest_at >= hop_at + super::super::draw::HOP_S - frame,
                "square {i}: a hop of {:.2} s",
                rest_at - hop_at
            );
            if let Some(&next) = sound::RECALL_AT.get(i + 1) {
                assert!(
                    rest_at <= next + frame,
                    "square {i} is still hopping when the next pluck sounds"
                );
            }
        }
        assert!(
            hops[2].1 + super::super::draw::HOP_S <= sound::RECALL_S,
            "the last hop ends inside the window"
        );
        let frames = testing::frames_of(&run.frames_dir).unwrap();
        let fps = f64::from(testing::FPS);
        let (from, to) = (
            (start * fps).round() as usize,
            ((start + seconds) * fps).round() as usize,
        );
        let moving = testing::changes(&frames, from, to);
        assert!(
            moving >= 9,
            "{moving} of the window's {} frames differ from the frame before",
            to - from
        );
    }

    #[test]
    fn the_sentence_breaks_into_its_words_and_marks() {
        let words: Vec<&str> = tokens(FEYNMAN_LINE).iter().map(|&(a, b)| &FEYNMAN_LINE[a..b]).collect();
        assert_eq!(
            words,
            [
                "“",
                "It",
                "was",
                "easy",
                "for",
                "him—",
                "every",
                "number",
                "was",
                "near",
                "something",
                "he",
                "knew.",
                "”"
            ]
        );
        let new: Vec<&str> = tokens(NEAR_LINE).iter().map(|&(a, b)| &NEAR_LINE[a..b]).collect();
        let clause = &words[6..13];
        let changed: Vec<(&str, &str)> = clause
            .iter()
            .zip(&new)
            .filter(|(a, b)| a != b)
            .map(|(a, b)| (*a, *b))
            .collect();
        assert_eq!(changed, [("was", "is"), ("he", "you"), ("knew.", "know.")]);
    }

    #[test]
    fn the_three_moves_are_the_three_sums() {
        for (side, deep, sum) in [(35, 5, "30x40+25"), (48, 2, "46x50+4"), (50, 3, "47x53+9")] {
            let m = one_move(side, deep).unwrap();
            assert_eq!(format!("{}x{}+{}", m.tall(), m.wide(), m.corner()), sum);
            assert_eq!(m.square(), side * side);
        }
    }
}
