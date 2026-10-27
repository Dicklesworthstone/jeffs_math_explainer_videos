//! Chapter 10, two shares: what chapter 05's experiment found, drawn on
//! the dial board. Trustees 4 and 5 come back with the vault's own cards, 2
//! and 6, the other three cards face down, and the strip shows what their
//! shares leave, as the experiment did: every secret (10_1). On the board,
//! where a curve is only its points, the dealer's curve runs through the
//! secret 5, trustees 1 to 3's marks and the pair's two points, its guides
//! joining them and labelled as guides, with its spins, 7 and 7, left of its
//! secret's row. The viewer is asked how many curves of the same kind run
//! through the pair's two points and the secret 0 instead (10_2); exactly
//! one, with spins 1 and 4, whose heights for trustees 4 and 5 are worked
//! out and land on 2 and 6; then the secret 1's, with spins 0 and 9 (10_3).
//! All eleven come in, one per secret, each a dealing the dealer could have
//! made, and the board is held for looking (10_4). The 121 spin pairs come
//! back as chapter 05's grid, and each secret lights its one cell as its
//! curve is pointed at (10_5). The chance card is the likelihood, the same
//! 1/121 under every secret, with no prior assumed (10_6).
//!
//! The eleven curves are `model::fitting_spins` for the pair's dealt shares
//! ([`candidates`]): the scene stops unless every secret fits exactly one
//! spin pair, the dealer's is the vault's recipe, and every position away
//! from the pair holds every mark once. The board is `board.rs`'s; the
//! guides join a curve's computed points and nothing on the dial is drawn
//! between marks except them, under a legend that says so. Emphasis never
//! restyles or scales typeset text: tags, labels and cards are pointed at
//! by outlines and rings, and only shapes (points, guides, cells, discs)
//! change color or swell. Chapter 11 rebuilds this board from the same
//! pieces.

use explainer_kit::props::precisely;
use explainer_kit::{BACKGROUND, Kit, Palette, fade_all, hex, v};
use fmn::prelude::*;

use super::board::{BOARD_POINT_R, Board, Grid, now, ring, turns_and_mark};
use super::draw::{
    FRAME_H, FRAME_W, Rail, STRIP_LEFT, STRIP_Y, Strip, TICK_H, TRUSTEE_R, bundle, card, fade_in_all, fade_in_lagged,
    fail, frame_at, panel, pause, read_by, recolorings, say, strip_right, stroke, swaps, think, trustee_at, wait_until,
};
use super::envelope::{PAD, dealt, for_pair, frame_halos, holding, outline, swelling};
use super::two_keys::{check_strip, glow, glowing, leaving, text_halo};
use crate::model::{self, ASKED_SECRETS, Dial, MARKS, PAIR, Ratio, Recipe, SECRET, SPINS, TRUSTEES};
use crate::palette;

pub struct TwoShares {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
pub const BOARD_TITLE: &str = "on the dial: points only";
pub const LEGEND: &str = "lines are guides joining one curve's points";
pub const TAGS_TITLE: &str = "spins";
const FIRST: &str = "1st spin";
const SECOND: &str = "2nd spin";
const FAIR: &str = "each as likely as any other";

// ------------------------------------------------------------------ layout

/// The dial board, left: chapter 08's rows, moved right and a little
/// narrower, so each secret's spins can stand left of its row.
pub const PAIR_BOARD: Grid = Grid {
    x0: -4.7,
    y0: -0.12,
    dx: 0.8,
    dy: 0.29,
    marks: MARKS,
    last: 5,
};
/// The spins' tags: their right edge, left of the row labels, and their
/// size; their title stands over the top row.
pub const TAG_RIGHT: f64 = -5.3;
pub const TAG_SIZE: f64 = 16.0;
/// A guide's width, and the legend under the position numbers.
pub const GUIDE_W: f64 = 1.5;
pub const LEGEND_Y: f64 = -0.85;
pub const LEGEND_SIZE: f64 = 16.0;
/// A ring round a board point, and round a rail disc.
pub const POINT_RING: f64 = 2.0 * BOARD_POINT_R;
pub const DISC_RING: f64 = TRUSTEE_R + 0.08;
/// 10_3's working, top right, one line for each of the pair.
const CHECK_Y: [f64; 2] = [3.3, 2.85];
const CHECK_SIZE: f64 = 22.0;
/// The think ring, lower right (chapters 10 and 11).
pub const THINK_X: f64 = 5.4;
pub const THINK_Y: f64 = -1.5;
/// The spin grid, lower right: chapter 05's, smaller. Its lower-left
/// corner, a cell and the square drawn in it, the empty cells' opacity,
/// the axes' numbers and titles, a lit cell's secret, and the count and
/// the note on fair spins right of it.
const GRID_X0: f64 = 1.6;
const GRID_Y0: f64 = -3.3;
const CELL: f64 = 0.22;
const CELL_IN: f64 = 0.18;
const CELL_O: f64 = 0.22;
const AXIS_GAP: f64 = 0.18;
const AXIS_SIZE: f64 = 13.0;
const AXIS_TITLE_SIZE: f64 = 16.0;
const AXIS_TITLE_GAP: f64 = 0.15;
const CELL_LABEL_SIZE: f64 = 11.0;
const COUNT_X: f64 = 4.2;
const COUNT_Y: f64 = -1.1;
const COUNT_SIZE: f64 = 18.0;
const FAIR_Y: f64 = -1.5;
const FAIR_SIZE: f64 = 16.0;
/// How far a card lifts out and drops in; the board's hold for looking
/// after 10_4, and the hold after the card is read.
pub const DROP: f64 = 0.3;
const HOLD_ON_ELEVEN: f64 = 2.0;
const HOLD_AT_END: f64 = 0.6;

// -------------------------------------------------------------------- data

/// One of the curves of two spins through the pair's points: its secret,
/// its spins, and its marks at positions 0 to the board's last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub secret: u32,
    pub spins: [u32; 2],
    pub marks: Vec<u32>,
}

/// For every secret, the one curve of two spins that deals `held`
/// (position, share), with its marks at positions 0 to `last`; an error
/// unless each secret fits exactly one spin pair (the model's brute force).
pub fn candidates(dial: Dial, held: &[(u32, u32)], last: u32) -> Result<Vec<Candidate>, String> {
    dial.all()
        .map(|s| match model::fitting_spins(dial, held, s, SPINS.len()).as_slice() {
            [w] if w.len() == 2 => {
                let r = Recipe::new(s, w);
                Ok(Candidate {
                    secret: s,
                    spins: [w[0], w[1]],
                    marks: (0..=last).map(|x| r.share(dial, x)).collect(),
                })
            }
            other => Err(format!(
                "secret {s} fits {} spin pairs through {held:?}, not one",
                other.len()
            )),
        })
        .collect()
}

/// A curve's spins, as its tag shows them: "1 and 4".
pub fn spins_tag(spins: [u32; 2]) -> String {
    format!("{} and {}", spins[0], spins[1])
}

/// The positions 0 to `last` where none of `held` sits, in order.
pub fn free_positions(held: &[(u32, u32)], last: u32) -> Vec<u32> {
    (0..=last).filter(|&x| held.iter().all(|p| p.0 != x)).collect()
}

/// The guides of a curve whose marks at positions 0, 1, … are `marks`: a
/// straight stroke's two ends, (position, mark), between each two
/// neighbouring positions, except between two held positions, whose guide
/// every curve through them shares. They only join computed points.
pub fn guide_ends(marks: &[u32], held: &[(u32, u32)]) -> Vec<[(f64, f64); 2]> {
    let is_held = |x: u32| held.iter().any(|p| p.0 == x);
    (0..(marks.len() as u32).saturating_sub(1))
        .filter(|&x| !(is_held(x) && is_held(x + 1)))
        .map(|x| {
            let (a, b) = (marks[x as usize], marks[x as usize + 1]);
            [(f64::from(x), f64::from(a)), (f64::from(x + 1), f64::from(b))]
        })
        .collect()
}

/// For each position, how many different marks the curves pass it at.
pub fn marks_per_position(all: &[Candidate]) -> Vec<usize> {
    let width = all.first().map_or(0, |c| c.marks.len());
    (0..width)
        .map(|x| {
            let mut at: Vec<u32> = all.iter().map(|c| c.marks[x]).collect();
            at.sort_unstable();
            at.dedup();
            at.len()
        })
        .collect()
}

/// 10_3's working for trustee `x`: the curve's height there on paper, the
/// secret plus each spin times the position's power, and the mark it
/// lands on: "trustee 4: 0 + 1 × 4 + 4 × 4 × 4 = 68 → 2".
pub fn check_row(c: &Candidate, x: u32, marks: u32) -> String {
    let h = Recipe::new(c.secret, &c.spins).height(x);
    let (_, k) = turns_and_mark(h, marks);
    format!(
        "trustee {x}: {} + {} \u{d7} {x} + {} \u{d7} {x} \u{d7} {x} = {h} \u{2192} {k}",
        c.secret, c.spins[0], c.spins[1]
    )
}

/// The grid's count: every pair of marks the two spins could land on.
fn grid_count(marks: u32, cells: usize) -> String {
    format!("{marks} \u{d7} {marks} = {cells} spin pairs")
}

/// 10_6's precisely card: the chance of the pair's shares given each
/// secret, one spin pair in all of them each, is the same for every secret.
/// That comes from the fair spins, not from assuming every secret equally
/// likely, so whatever anyone believed about the secret stands (as 08_8's
/// card, Mail 6906). Short enough to stay two lines (see its test).
fn card_text(held: [u32; 2], chance: Ratio) -> String {
    format!(
        "chance of shares {} and {}, given secret s: {chance} for every s (one spin pair each). \
         Equal likelihoods, not a uniform prior: any belief survives",
        held[0], held[1]
    )
}

/// The middle of the grid's cell for first spin `a` (across) and second
/// spin `b` (up).
fn cell_at(a: u32, b: u32) -> Vec3 {
    v(
        GRID_X0 + CELL * (f64::from(a) + 0.5),
        GRID_Y0 + CELL * (f64::from(b) + 0.5),
    )
}

// ---------------------------------------------------------------- drawing

/// The centre of the tag for the curve with secret `s`, right-aligned
/// left of `grid`'s row `s`.
pub fn tag_at(grid: &Grid, s: u32) -> Vec3 {
    v(TAG_RIGHT, grid.at(0.0, f64::from(s))[1])
}

/// Text right-aligned at `right`. Detached.
pub fn text_right(kit: &Kit, stage: &mut Stage<'_>, s: &str, size: f64, color: Srgb, right: Vec3) -> fmn::Result<Mob> {
    let m = kit.text(stage, s, size, color, right)?;
    stage.move_to(m, right, RIGHT);
    Ok(m)
}

/// A curve on the board: a point at each free position (its secret's, at
/// position 0, in gold), the guides joining them and the held points, and
/// its spins' tag left of its secret's row. Detached. The free positions
/// come before the held ones, as for trustees 4 and 5, so guide `i` ends at
/// point `i + 1`, and the last guide at the first held point.
pub struct Drawn {
    pub secret: u32,
    pub points: Vec<Mob>,
    pub centres: Vec<Vec3>,
    pub guides: Vec<Mob>,
    pub tag: Mob,
}

impl Drawn {
    pub fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &Board,
        c: &Candidate,
        held: &[(u32, u32)],
        colors: [Srgb; 2],
    ) -> fmn::Result<Self> {
        let free = free_positions(held, board.grid.last);
        if free.iter().zip(0..).any(|(&x, i)| x != i) || free.len() + held.len() != c.marks.len() {
            return Err(fail(format!(
                "positions {free:?} are not the first ones before {held:?}"
            )));
        }
        let [point_color, guide_color] = colors;
        let mut guides = Vec::new();
        for [a, b] in guide_ends(&c.marks, held) {
            let ends = vec![board.grid.at(a.0, a.1), board.grid.at(b.0, b.1)];
            guides.push(stroke(stage, ends, guide_color, GUIDE_W));
        }
        let (mut points, mut centres) = (Vec::with_capacity(free.len()), Vec::with_capacity(free.len()));
        for &x in &free {
            let k = c.marks[x as usize];
            let color = if x == 0 { palette::secret() } else { point_color };
            points.push(board.point(stage, x, k, color));
            centres.push(board.grid.at(f64::from(x), f64::from(k)));
        }
        let tag = text_right(
            kit,
            stage,
            &spins_tag(c.spins),
            TAG_SIZE,
            palette::spin(),
            tag_at(&board.grid, c.secret),
        )?;
        Ok(Self {
            secret: c.secret,
            points,
            centres,
            guides,
            tag,
        })
    }

    /// Everything it is drawn with, the guides first so the points sit on
    /// them.
    pub fn parts(&self) -> Vec<Mob> {
        let mut all = self.guides.clone();
        all.extend(&self.points);
        all.push(self.tag);
        all
    }

    /// Its colored shapes, each with `colors` (points, guides), to recolor
    /// together: never the gold secret, never the tag.
    pub fn tinted(&self, colors: [Srgb; 2]) -> Vec<(Mob, Srgb)> {
        let mut changes: Vec<(Mob, Srgb)> = self.points[1..].iter().map(|&m| (m, colors[0])).collect();
        changes.extend(self.guides.iter().map(|&m| (m, colors[1])));
        changes
    }

    /// Rings round each of its points, in `color`. Detached.
    pub fn rings(&self, stage: &mut Stage<'_>, color: Srgb) -> Vec<Mob> {
        self.centres
            .iter()
            .map(|&at| ring(stage, at, POINT_RING, color))
            .collect()
    }

    /// From its secret's point on: each guide growing with the point it
    /// reaches, one step a guide, prepared, to run one after another.
    pub fn steps(&self, stage: &mut Stage<'_>) -> fmn::Result<Vec<Box<dyn Animation>>> {
        let mut steps = Vec::with_capacity(self.guides.len());
        for (i, &g) in self.guides.iter().enumerate() {
            let mut both = vec![stage.prepare(show_creation(g))?];
            if let Some(&p) = self.points.get(i + 1) {
                let a = fade_in(stage.arena_mut(), p, ORIGIN, 1.0)?;
                both.push(stage.prepare(a)?);
            }
            let g = AnimationGroup::new(stage.arena_mut(), both)?;
            steps.push(stage.prepare(g)?);
        }
        Ok(steps)
    }

    /// Its secret's point, its [`Drawn::steps`], then its tag, one after
    /// another, prepared, over `rt`.
    pub fn arriving(&self, stage: &mut Stage<'_>, rt: f64) -> fmn::Result<Box<dyn Animation>> {
        let a = fade_in(stage.arena_mut(), self.points[0], ORIGIN, 1.0)?;
        let mut steps = vec![stage.prepare(a)?];
        steps.extend(self.steps(stage)?);
        let a = fade_in(stage.arena_mut(), self.tag, ORIGIN, 1.0)?;
        steps.push(stage.prepare(a)?);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), steps, 0.7)?.with_run_time(rt);
        stage.prepare(g)
    }
}

/// The pair's two points on the board, in their role's color, and the one
/// guide between them every curve shares. Detached.
pub fn pair_points(stage: &mut Stage<'_>, board: &Board, held: &[(u32, u32)]) -> fmn::Result<(Vec<Mob>, Mob)> {
    let points: Vec<Mob> = held
        .iter()
        .map(|&(x, k)| board.point(stage, x, k, palette::pair()))
        .collect();
    let ends: Vec<Vec3> = held
        .iter()
        .map(|&(x, k)| board.grid.at(f64::from(x), f64::from(k)))
        .collect();
    if ends.len() != 2 {
        return Err(fail(format!("a pair of {} points", ends.len())));
    }
    let guide = stroke(stage, ends, palette::pair(), GUIDE_W);
    Ok((points, guide))
}

/// A card face down in trustee `t`'s frame on the rail (chapter 09's).
/// Detached.
pub fn card_back(stage: &mut Stage<'_>, t: u32) -> fmn::Result<Mob> {
    panel(stage, Palette::dim(), 0.85, frame_at(t), FRAME_W - 0.14, FRAME_H - 0.12)
}

/// A ring round trustee `t`'s disc on the rail. Detached.
pub fn disc_ring(stage: &mut Stage<'_>, t: u32, color: Srgb) -> Mob {
    ring(stage, trustee_at(t), DISC_RING, color)
}

/// An outline round the board's column at position `x`, its every mark.
/// Detached.
pub fn column_outline(stage: &mut Stage<'_>, grid: &Grid, x: u32, color: Srgb) -> fmn::Result<Mob> {
    let mid = grid.at(f64::from(x), (f64::from(grid.marks) - 1.0) / 2.0);
    outline(
        stage,
        mid,
        4.0 * BOARD_POINT_R + 2.0 * PAD,
        grid.top() - grid.bottom() + 2.0 * PAD,
        color,
    )
}

/// An outline round the strip's ticks. Detached.
pub fn ticks_outline(stage: &mut Stage<'_>, marks: u32, color: Srgb) -> fmn::Result<Mob> {
    let right = strip_right(marks);
    outline(
        stage,
        v((STRIP_LEFT + right) / 2.0, STRIP_Y),
        right - STRIP_LEFT + 2.0 * PAD,
        TICK_H + 2.0 * PAD,
        color,
    )
}

/// The opening both chapters share: the rail with every vault card in its
/// frame, the pair's face up and in their role's color, the others face
/// down and dimmed; and the board, empty. Detached, but the rail.
pub struct Opening {
    pub rail: Rail,
    pub backs: Vec<Mob>,
    pub pair_cards: Vec<Mob>,
    pub board: Board,
}

impl Opening {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, dial: Dial) -> fmn::Result<Self> {
        let rail = Rail::build(kit, stage, &TRUSTEES)?;
        let mut backs = Vec::new();
        for t in TRUSTEES.iter().copied().filter(|t| !PAIR.contains(t)) {
            backs.push(card_back(stage, t)?);
        }
        let mut pair_cards = Vec::new();
        for (t, k) in dealt(dial, &PAIR) {
            pair_cards.push(card(kit, stage, t, &k.to_string(), palette::share())?);
        }
        let board = Board::build(kit, stage, PAIR_BOARD, BOARD_TITLE, true)?;
        Ok(Self {
            rail,
            backs,
            pair_cards,
            board,
        })
    }

    /// The rail's discs as the pair's view tints them: the pair in their
    /// role's color, the others dimmed in place.
    pub fn tints(&self) -> Vec<(Mob, Srgb)> {
        let others: Vec<u32> = TRUSTEES.iter().copied().filter(|t| !PAIR.contains(t)).collect();
        let mut changes = self.rail.tint(&PAIR, palette::pair());
        changes.extend(self.rail.tint(&others, palette::ghost()));
        changes
    }
}

impl SceneConstruct for TwoShares {
    fn name(&self) -> &str {
        "10_two_shares"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let marks = dial.marks();
        let recipe = Recipe::example();
        let last = PAIR_BOARD.last;
        // the pair's cards are the vault's; for every secret one curve goes
        // through them, the dealer's is the vault's recipe, and every
        // position away from the pair holds every mark once
        let held = dealt(dial, &PAIR);
        let held_shares = [held[0].1, held[1].1];
        let all = candidates(dial, &held, last).map_err(fail)?;
        let dealer = &all[SECRET as usize];
        let dealt_marks: Vec<u32> = (0..=last).map(|x| recipe.share(dial, x)).collect();
        if dealer.spins != SPINS || dealer.marks != dealt_marks {
            return Err(fail(format!("the dealer's curve through the pair is {dealer:?}")));
        }
        let free = free_positions(&held, last);
        let per = marks_per_position(&all);
        for x in 0..=last {
            let want = if free.contains(&x) { marks as usize } else { 1 };
            if per[x as usize] != want {
                return Err(fail(format!(
                    "position {x} holds {} marks, not {want}",
                    per[x as usize]
                )));
            }
        }
        let mut cells: Vec<[u32; 2]> = all.iter().map(|c| c.spins).collect();
        cells.sort_unstable();
        cells.dedup();
        let pairs = model::tuples(dial, SPINS.len()).count();
        if cells.len() != all.len() || pairs != (marks * marks) as usize {
            return Err(fail("two secrets share a spin pair".into()));
        }
        let [asked, next] = ASKED_SECRETS;
        let everyone: Vec<u32> = dial.all().collect();

        // the opening: the rail with the vault's cards, the pair's face up;
        // the empty board
        let open = Opening::build(&kit, stage, dial)?;
        let board = &open.board;
        open.rail.reveal(stage, 0.6)?;
        let mut cards = open.backs.clone();
        cards.extend(&open.pair_cards);
        let mut anims = swaps(stage, &[], &cards, 0.5)?;
        anims.extend(recolorings(stage, &open.tints(), 0.5)?);
        stage.play_prepared(anims)?;
        board.show(stage, 0.7)?;

        // 10_1: the pair, the experiment's answer on the strip, and the
        // dealer's curve on the board
        let spoken = say(&kit, stage, "10_1")?;
        spoken.wait_for(stage, "Back to trustees four and five")?;
        let mut halos: Vec<Mob> = PAIR.iter().map(|&t| disc_ring(stage, t, palette::pair())).collect();
        halos.extend(frame_halos(stage, &PAIR, palette::pair())?);
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "with their two and six")?;
        let (pair_dots, pair_guide) = pair_points(stage, board, &held)?;
        let mut anims = swaps(stage, &[], &pair_dots, 0.5)?;
        let mut labels = Vec::new();
        for &(_, k) in &held {
            let row = board
                .row_label(k)
                .ok_or_else(|| fail(format!("the board has no row {k}")))?;
            labels.push(text_halo(stage, row, palette::pair())?);
        }
        anims.extend(glowing(stage, &labels, 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "the experiment from before")?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 1.0)?;
        strip.show(&kit, stage, &holding(dial, &PAIR), &for_pair(PAIR), 0.4)?;
        check_strip(&strip, &everyone, "for the pair")?;
        spoken.wait_for(stage, "Now we can see what it found")?;
        let ticks = ticks_outline(stage, marks, palette::secret())?;
        glow(stage, &[ticks], 0.9)?;
        spoken.wait_for(stage, "Through their two points")?;
        let rings: Vec<Mob> = held
            .iter()
            .map(|&(x, k)| {
                ring(
                    stage,
                    board.grid.at(f64::from(x), f64::from(k)),
                    POINT_RING,
                    palette::pair(),
                )
            })
            .collect();
        glow(stage, &rings, 0.7)?;
        spoken.wait_for(stage, "and the secret five")?;
        let d = Drawn::build(&kit, stage, board, dealer, &held, [palette::share(); 2])?;
        let r = ring(stage, d.centres[0], POINT_RING, palette::secret());
        let mut anims = swaps(stage, &[], &[d.points[0]], 0.4)?;
        anims.extend(glowing(stage, &[r], 0.7)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "runs the dealer's curve")?;
        let legend = kit.text(
            stage,
            LEGEND,
            LEGEND_SIZE,
            Palette::dim(),
            v((board.grid.left() + board.grid.right()) / 2.0, LEGEND_Y),
        )?;
        let mut steps = d.steps(stage)?;
        steps.push(stage.prepare(show_creation(pair_guide))?);
        let rt = (spoken.at("with spins seven and seven")? - now(stage)).clamp(1.2, 2.4);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), steps, 0.8)?.with_run_time(rt);
        let mut anims = vec![stage.prepare(g)?];
        anims.extend(swaps(stage, &[], &[legend], rt)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "with spins seven and seven")?;
        let title = text_right(
            &kit,
            stage,
            TAGS_TITLE,
            TAG_SIZE,
            Palette::dim(),
            tag_at(&board.grid, marks),
        )?;
        fade_in_all(stage, &[title, d.tag], 0.4)?;
        let halo = text_halo(stage, d.tag, palette::spin())?;
        glow(stage, &[halo], 0.7)?;
        spoken.wait_end(stage)?;
        let dealer_curve = d;

        // 10_2: through the same two points, and the secret 0 instead: how
        // many? (the dealer's curve dims; nothing answers yet)
        let spoken = say(&kit, stage, "10_2")?;
        let crowd = [Palette::dim(), palette::ghost()];
        let anims = recolorings(stage, &dealer_curve.tinted(crowd), 0.6)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "their two points")?;
        let rings: Vec<Mob> = held
            .iter()
            .map(|&(x, k)| {
                ring(
                    stage,
                    board.grid.at(f64::from(x), f64::from(k)),
                    POINT_RING,
                    palette::pair(),
                )
            })
            .collect();
        glow(stage, &rings, 0.7)?;
        spoken.wait_for(stage, "the secret zero instead")?;
        let first = Drawn::build(&kit, stage, board, &all[asked as usize], &held, [Palette::ink(); 2])?;
        let r = ring(stage, first.centres[0], POINT_RING, palette::secret());
        let mut anims = swaps(stage, &[], &[first.points[0]], 0.4)?;
        anims.extend(glowing(stage, &[r], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 10_3: exactly one, spins 1 and 4, checked; then the secret 1's
        let spoken = say(&kit, stage, "10_3")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Exactly one")?;
        let steps = first.steps(stage)?;
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), steps, 0.8)?.with_run_time(1.2);
        stage.play(g)?;
        spoken.wait_for(stage, "with spins one and four")?;
        fade_in_all(stage, &[first.tag], 0.4)?;
        let halo = text_halo(stage, first.tag, palette::spin())?;
        glow(stage, &[halo], 0.7)?;
        let mut checks = Vec::with_capacity(held.len());
        for (j, ((x, k), [cue, lands])) in held
            .iter()
            .zip([
                ["trustee four's height would be sixty-eight", "which lands on two"],
                ["and trustee five's", "which lands on six"],
            ])
            .enumerate()
        {
            spoken.wait_for(stage, cue)?;
            let row = kit.text_left(
                stage,
                &check_row(&all[asked as usize], *x, marks),
                CHECK_SIZE,
                Palette::ink(),
                v(STRIP_LEFT, CHECK_Y[j]),
            )?;
            let mut anims = swaps(stage, &[], &[row], 0.5)?;
            let halos = frame_halos(stage, &[*x], palette::pair())?;
            anims.extend(glowing(stage, &halos, 0.8)?);
            stage.play_prepared(anims)?;
            spoken.wait_for(stage, lands)?;
            let r = ring(
                stage,
                board.grid.at(f64::from(*x), f64::from(*k)),
                POINT_RING,
                palette::pair(),
            );
            let mut anims = glowing(stage, &[r], 0.8)?;
            anims.push(swelling(stage, pair_dots[j], 1.6, palette::pair(), 0.7)?);
            stage.play_prepared(anims)?;
            checks.push(row);
        }
        spoken.wait_for(stage, "For the secret one")?;
        let second = Drawn::build(&kit, stage, board, &all[next as usize], &held, [Palette::ink(); 2])?;
        let mut anims = leaving(stage, &checks, ORIGIN, 0.5)?;
        anims.extend(recolorings(stage, &first.tinted(crowd), 0.5)?);
        anims.push(second.arriving(stage, 1.4)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "the spins would be zero and nine")?;
        let halo = text_halo(stage, second.tag, palette::spin())?;
        glow(stage, &[halo], 0.7)?;
        spoken.wait_for(stage, "Every secret gets its own curve")?;
        let anims = recolorings(stage, &second.tinted(crowd), 0.5)?;
        stage.play_prepared(anims)?;
        let tags = vec![
            text_halo(stage, first.tag, palette::spin())?,
            text_halo(stage, second.tag, palette::spin())?,
        ];
        glow(stage, &tags, 0.8)?;
        spoken.wait_end(stage)?;

        // 10_4: all eleven, one for each secret; the board held for looking
        let spoken = say(&kit, stage, "10_4")?;
        spoken.wait_for(stage, "Here are all eleven")?;
        let shown = [SECRET, asked, next];
        let mut drawn = vec![dealer_curve, first, second];
        let mut arriving = Vec::new();
        for c in all.iter().filter(|c| !shown.contains(&c.secret)) {
            let d = Drawn::build(&kit, stage, board, c, &held, crowd)?;
            arriving.push(d.arriving(stage, 1.0)?);
            drawn.push(d);
        }
        let rt = (spoken.at("Each one is a dealing")? - now(stage)).clamp(2.0, 3.2);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), arriving, 0.3)?.with_run_time(rt);
        stage.play(g)?;
        // in secret order from here, one curve each
        drawn.sort_by_key(|d| d.secret);
        if drawn.iter().map(|d| d.secret).ne(dial.all()) {
            return Err(fail("the board does not hold one curve per secret".into()));
        }
        spoken.wait_for(stage, "Each one is a dealing")?;
        let column = column_outline(stage, &board.grid, 0, palette::secret())?;
        glow(stage, &[column], 1.0)?;
        spoken.wait_for(stage, "if the secret had been different")?;
        let swells: Vec<Box<dyn Animation>> = drawn
            .iter()
            .map(|d| swelling(stage, d.points[0], 1.5, palette::secret(), 0.8))
            .collect::<fmn::Result<_>>()?;
        stage.play_prepared(swells)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_ELEVEN)?;

        // 10_5: the 121 spin pairs, all equally likely; each secret's curve
        // lights its one cell
        let spoken = say(&kit, stage, "10_5")?;
        spoken.wait_for(stage, "the hundred and twenty-one spin pairs")?;
        let mut columns: Vec<Vec<Mob>> = vec![Vec::new(); marks as usize];
        for t in model::tuples(dial, SPINS.len()) {
            let c = panel(stage, palette::spin(), CELL_O, cell_at(t[0], t[1]), CELL_IN, CELL_IN)?;
            columns[t[0] as usize].push(c);
        }
        let mut column_mobs = Vec::with_capacity(columns.len());
        for c in &columns {
            column_mobs.push(bundle(stage, c)?);
        }
        let mut axes = Vec::new();
        for k in dial.all() {
            let label = k.to_string();
            axes.push(kit.text(
                stage,
                &label,
                AXIS_SIZE,
                palette::spin(),
                v(cell_at(k, 0)[0], GRID_Y0 - AXIS_GAP),
            )?);
            axes.push(kit.text(
                stage,
                &label,
                AXIS_SIZE,
                palette::spin(),
                v(GRID_X0 - AXIS_GAP, cell_at(0, k)[1]),
            )?);
        }
        let grid_right = GRID_X0 + CELL * f64::from(marks);
        let grid_top = GRID_Y0 + CELL * f64::from(marks);
        axes.push(kit.text_left(
            stage,
            FIRST,
            AXIS_TITLE_SIZE,
            Palette::dim(),
            v(grid_right + AXIS_TITLE_GAP, GRID_Y0 - AXIS_GAP),
        )?);
        axes.push(kit.text(
            stage,
            SECOND,
            AXIS_TITLE_SIZE,
            Palette::dim(),
            v(GRID_X0 - AXIS_GAP, grid_top + AXIS_TITLE_GAP + 0.05),
        )?);
        let count = kit.text_left(
            stage,
            &grid_count(marks, pairs),
            COUNT_SIZE,
            palette::spin(),
            v(COUNT_X, COUNT_Y),
        )?;
        axes.push(count);
        fade_in_all(stage, &axes, 0.5)?;
        fade_in_lagged(stage, &column_mobs, 1.2, 0.2)?;
        spoken.wait_for(stage, "are all equally likely")?;
        let fair = kit.text_left(stage, FAIR, FAIR_SIZE, Palette::dim(), v(COUNT_X, FAIR_Y))?;
        let mut anims = swaps(stage, &[], &[fair], 0.5)?;
        let halo = text_halo(stage, count, palette::spin())?;
        anims.extend(glowing(stage, &[halo], 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Whatever the secret")?;
        // each secret in turn: its point and tag pointed at, its one cell lit
        let rt = (spoken.at("hands trustees four and five")? - now(stage)).clamp(2.2, 4.4);
        let mut lit = Vec::with_capacity(all.len());
        let mut each = Vec::with_capacity(all.len());
        for (c, d) in all.iter().zip(&drawn) {
            let at = cell_at(c.spins[0], c.spins[1]);
            let cell = panel(stage, palette::secret(), 1.0, at, CELL_IN, CELL_IN)?;
            let label = kit.text(stage, &c.secret.to_string(), CELL_LABEL_SIZE, hex(BACKGROUND), at)?;
            let mut both = swaps(stage, &[], &[cell, label], 1.0)?;
            let r = ring(stage, d.centres[0], POINT_RING, palette::secret());
            let mut halos = vec![r, text_halo(stage, d.tag, palette::spin())?];
            halos.push(ring(stage, at, CELL * 0.75, palette::secret()));
            both.extend(glowing(stage, &halos, 1.0)?);
            let g = AnimationGroup::new(stage.arena_mut(), both)?;
            each.push(stage.prepare(g)?);
            lit.extend([cell, label]);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.35)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "hands trustees four and five")?;
        let mut halos: Vec<Mob> = held
            .iter()
            .map(|&(x, k)| {
                ring(
                    stage,
                    board.grid.at(f64::from(x), f64::from(k)),
                    POINT_RING,
                    palette::pair(),
                )
            })
            .collect();
        halos.extend(frame_halos(stage, &PAIR, palette::pair())?);
        glow(stage, &halos, 0.9)?;
        spoken.wait_end(stage)?;

        // 10_6: what they hold is as likely under every secret: the card
        let spoken = say(&kit, stage, "10_6")?;
        let mut going = axes;
        going.push(fair);
        going.extend(column_mobs);
        going.extend(lit);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let chance = Ratio::new(1, pairs as i64);
        let exact = card_text(held_shares, chance);
        let note = precisely(&kit, stage, &exact)?;
        fade_in_all(stage, &[note], 0.5)?;
        let read = read_by(stage, &exact);
        spoken.wait_for(stage, "just as likely under every secret")?;
        let ticks = ticks_outline(stage, marks, palette::secret())?;
        glow(stage, &[ticks], 1.0)?;
        wait_until(stage, read)?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_AT_END)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::board::{POSITION_LABEL_SIZE, ROW_LABEL_SIZE, TITLE_SIZE};
    use super::super::draw::{FRAME_Y, PAUSE_NOTE, STRIP_CAPTION_Y, STRIP_TITLE_Y, THINK_R};
    use super::*;
    use crate::facts;
    use crate::model::{OTHER_SPINS, TEN, combinations, csv};

    /// The production narration: registering chapter 10 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// A share as the definition writes it, the secret plus each spin
    /// times the position's power, wrapped onto `p` marks: apart from the
    /// model's Horner rule.
    fn share(s: u32, a: u32, b: u32, x: u32, p: u32) -> u32 {
        (s + a * x + b * x * x) % p
    }

    /// Every spin pair on `p` marks.
    fn spin_pairs(p: u32) -> impl Iterator<Item = [u32; 2]> {
        (0..p).flat_map(move |a| (0..p).map(move |b| [a, b]))
    }

    /// Every phrase the scene waits for is in its line of the script, once,
    /// and they come in the order they are said.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        let phrases = [
            ("10_1", "Back to trustees four and five"),
            ("10_1", "with their two and six"),
            ("10_1", "the experiment from before"),
            ("10_1", "Now we can see what it found"),
            ("10_1", "Through their two points"),
            ("10_1", "and the secret five"),
            ("10_1", "runs the dealer's curve"),
            ("10_1", "with spins seven and seven"),
            ("10_2", "their two points"),
            ("10_2", "the secret zero instead"),
            ("10_3", "Exactly one"),
            ("10_3", "with spins one and four"),
            ("10_3", "trustee four's height would be sixty-eight"),
            ("10_3", "which lands on two"),
            ("10_3", "and trustee five's"),
            ("10_3", "which lands on six"),
            ("10_3", "For the secret one"),
            ("10_3", "the spins would be zero and nine"),
            ("10_3", "Every secret gets its own curve"),
            ("10_4", "Here are all eleven"),
            ("10_4", "Each one is a dealing"),
            ("10_4", "if the secret had been different"),
            ("10_5", "the hundred and twenty-one spin pairs"),
            ("10_5", "are all equally likely"),
            ("10_5", "Whatever the secret"),
            ("10_5", "hands trustees four and five"),
            ("10_6", "just as likely under every secret"),
        ];
        let mut last: Option<(&str, usize)> = None;
        for (id, phrase) in phrases {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id}: {phrase:?} in {text}");
            let at = text.find(phrase).unwrap();
            if let Some((prev_id, prev_at)) = last {
                assert!(
                    prev_id != id || prev_at < at,
                    "{id}: {phrase:?} is waited for out of order"
                );
            }
            last = Some((id, at));
        }
    }

    /// What the scene draws and writes is the facts': the pair's cards, the
    /// eleven curves' marks and spins, 10_3's working, the marks each
    /// position holds, the grid's count and the chance on the card.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        assert_eq!(f.get("S-F1.pair_shares"), Some(csv(&[held[0].1, held[1].1]).as_str()));
        assert_eq!(f.get("S-F7.pair.count"), Some(PAIR.len().to_string().as_str()));
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        for c in &all {
            assert_eq!(f.get(&format!("S-F7.board.{}", c.secret)), Some(csv(&c.marks).as_str()));
        }
        let spins: Vec<String> = all.iter().map(|c| format!("{}:{}", c.spins[0], c.spins[1])).collect();
        assert_eq!(f.get("S-F7.spins_by_secret"), Some(spins.join(",").as_str()));
        assert_eq!(spins_tag(all[SECRET as usize].spins), "7 and 7");
        let [asked, next] = ASKED_SECRETS;
        assert_eq!(f.get("S-F7.asked.first"), Some(asked.to_string().as_str()));
        assert_eq!(f.get("S-F7.asked.second"), Some(next.to_string().as_str()));
        assert_eq!(spins_tag(all[asked as usize].spins), "1 and 4");
        assert_eq!(spins_tag(all[next as usize].spins), "0 and 9");
        assert_eq!(f.get("S-F7.spins.0"), Some(csv(&all[asked as usize].spins).as_str()));
        assert_eq!(f.get("S-F7.spins.1"), Some(csv(&all[next as usize].spins).as_str()));
        // the working: each held trustee's height on paper, and where it lands
        for &(x, y) in &held {
            let c = &all[asked as usize];
            let h = Recipe::new(c.secret, &c.spins).height(x);
            assert_eq!(f.get(&format!("S-F7.check.{x}")), Some(format!("{x},{h},{y}").as_str()));
            assert!(check_row(c, x, MARKS).ends_with(&format!("= {h} \u{2192} {y}")));
        }
        assert_eq!(
            check_row(&all[asked as usize], 4, MARKS),
            "trustee 4: 0 + 1 \u{d7} 4 + 4 \u{d7} 4 \u{d7} 4 = 68 \u{2192} 2"
        );
        assert_eq!(
            check_row(&all[asked as usize], 5, MARKS),
            "trustee 5: 0 + 1 \u{d7} 5 + 4 \u{d7} 5 \u{d7} 5 = 105 \u{2192} 6"
        );
        assert_eq!(
            f.get("S-F7.board.columns"),
            Some(csv(&marks_per_position(&all)).as_str())
        );
        assert_eq!(
            grid_count(MARKS, model::tuples(dial, 2).count()),
            format!("11 \u{d7} 11 = {} spin pairs", f.get("S-F6.spin_pairs").unwrap())
        );
        let chance = Ratio::new(1, 121);
        assert_eq!(f.get("S-F7.chance_each"), Some(chance.to_string().as_str()));
        let text = card_text([held[0].1, held[1].1], chance);
        assert!(text.starts_with("chance of shares 2 and 6, given secret s: 1/121 for every s"));
        assert!(text.contains("not a uniform prior"));
        assert_eq!(for_pair(PAIR), "for: trustees 4 and 5");
    }

    /// The precisely card stays two lines at every wrap the lower third
    /// can narrow to, so its band stays below the rail's discs.
    #[test]
    fn the_card_is_two_lines() {
        let text = card_text([2, 6], Ratio::new(1, 121));
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// For every pair of trustees and every two shares they could hold,
    /// every secret is dealt by exactly one of the 121 spin pairs, counted
    /// here with the definition written out, apart from the model; the
    /// scene's eleven curves for the vault's 2 and 6 are those spin pairs.
    /// The negative control, the wrong modulus: on ten marks the same pair's
    /// shares fit some secrets more ways than others, and the scene's
    /// builder refuses them.
    #[test]
    fn every_secret_fits_one_spin_pair_for_every_pair() {
        let p = MARKS;
        let pairs = combinations(&TRUSTEES, 2);
        assert_eq!(pairs.len(), 10);
        for pair in &pairs {
            for [y1, y2] in spin_pairs(p) {
                for s in 0..p {
                    let ways = spin_pairs(p)
                        .filter(|&[a, b]| share(s, a, b, pair[0], p) == y1 && share(s, a, b, pair[1], p) == y2)
                        .count();
                    assert_eq!(ways, 1, "{pair:?} holding {y1} and {y2}, secret {s}");
                }
            }
        }
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        assert_eq!(all.len(), MARKS as usize);
        for c in &all {
            let [a, b] = c.spins;
            assert!(held.iter().all(|&(x, y)| share(c.secret, a, b, x, p) == y), "{c:?}");
            for (x, &k) in (0..).zip(&c.marks) {
                assert_eq!(k, share(c.secret, a, b, x, p), "{c:?} at {x}");
            }
        }
        // the control: ten marks
        let ten_held: Vec<(u32, u32)> = PAIR
            .iter()
            .map(|&x| (x, share(SECRET, SPINS[0], SPINS[1], x, TEN)))
            .collect();
        let counts: Vec<usize> = (0..TEN)
            .map(|s| {
                spin_pairs(TEN)
                    .filter(|&[a, b]| ten_held.iter().all(|&(x, y)| share(s, a, b, x, TEN) == y))
                    .count()
            })
            .collect();
        assert!(counts.windows(2).any(|w| w[0] != w[1]), "ten marks: {counts:?}");
        assert!(candidates(Dial::new(TEN), &ten_held, PAIR_BOARD.last).is_err());
    }

    /// Away from the pair the eleven curves pass each position at eleven
    /// different marks, so every lattice point there lies on exactly one of
    /// them, and position 0's is the curve's own secret; at the pair's
    /// positions they all meet. Planted negatives: one share, or one trustee
    /// holding two different cards, gives no curve per secret.
    #[test]
    fn the_curves_fill_every_column_away_from_the_pair() {
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        assert_eq!(marks_per_position(&all), vec![11, 11, 11, 11, 1, 1]);
        assert_eq!(free_positions(&held, PAIR_BOARD.last), vec![0, 1, 2, 3]);
        for x in 0..4 {
            let mut column: Vec<u32> = all.iter().map(|c| c.marks[x]).collect();
            column.sort_unstable();
            assert_eq!(column, dial.all().collect::<Vec<_>>(), "position {x}");
        }
        assert!(all.iter().all(|c| c.marks[0] == c.secret));
        assert!(all.iter().all(|c| c.marks[4] == 2 && c.marks[5] == 6));
        assert!(candidates(dial, &held[..1], PAIR_BOARD.last).is_err());
        assert!(candidates(dial, &[(4, 2), (4, 6)], PAIR_BOARD.last).is_err());
    }

    /// Every guide joins two of one curve's own points at neighbouring
    /// positions; none is drawn between two held positions, which every
    /// curve shares, and the last ends on the pair's first point. The
    /// control: held positions that are not neighbours share no guide, so
    /// every neighbouring step gets one.
    #[test]
    fn guides_only_join_a_curves_points() {
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
        for c in candidates(dial, &held, PAIR_BOARD.last).unwrap() {
            let ends = guide_ends(&c.marks, &held);
            assert_eq!(ends.len(), 4, "{c:?}");
            for [(xa, ka), (xb, kb)] in &ends {
                assert!(near(xb - xa, 1.0));
                assert!(near(*ka, f64::from(c.marks[*xa as usize])));
                assert!(near(*kb, f64::from(c.marks[*xb as usize])));
                let both_held = [*xa, *xb].iter().all(|&x| held.iter().any(|p| near(f64::from(p.0), x)));
                assert!(!both_held);
            }
            assert!(near(ends[3][1].0, 4.0) && near(ends[3][1].1, 2.0));
        }
        let apart = [(1, 4), (3, 9)];
        assert_eq!(guide_ends(&[5, 4, 0, 9, 1, 2], &apart).len(), 5);
    }

    /// The rail holds the vault's cards: trustees 4 and 5 hold 2 and 6, as
    /// chapter 04's sealed method dealt them, and the dealer's curve passes
    /// trustees 1 to 3 at their own face-down cards, 8, 3 and 1. The planted
    /// negative: chapter 17's other spins deal the pair other cards, and no
    /// curve here starts at the secret with those spins.
    #[test]
    fn the_cards_are_the_vaults() {
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        assert_eq!(held, vec![(4, 2), (5, 6)]);
        let vault: Vec<u32> = dealt(dial, &TRUSTEES).iter().map(|p| p.1).collect();
        assert_eq!(vault, vec![8, 3, 1, 2, 6]);
        assert_eq!(vault, Recipe::example().deal(dial, &TRUSTEES));
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        let dealer = &all[SECRET as usize];
        assert_eq!(dealer.spins, SPINS);
        assert_eq!(&dealer.marks[1..], vault.as_slice());
        let other = Recipe::new(SECRET, &OTHER_SPINS);
        assert_ne!(other.deal(dial, &PAIR), vec![2, 6]);
        assert_ne!(dealer.spins, [OTHER_SPINS[0], OTHER_SPINS[1]]);
    }

    /// The pieces keep apart, inside the frame: the tags left of the board,
    /// the board's title and legend, the strip right of it, the working top
    /// right, the think ring and the grid lower right.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.3;
        let top = FRAME_Y_RADIUS - 0.3;
        let g = PAIR_BOARD;
        let dial = Dial::vault();
        let all = candidates(dial, &dealt(dial, &PAIR), g.last).unwrap();
        // the tags: right of the frame's edge, left of the row labels
        let widest = all
            .iter()
            .map(|c| text_w(&spins_tag(c.spins), TAG_SIZE))
            .fold(text_w(TAGS_TITLE, TAG_SIZE), f64::max);
        assert!(TAG_RIGHT - widest > -edge);
        assert!(TAG_RIGHT < g.row_label_at(10)[0] - text_w("10", ROW_LABEL_SIZE) / 2.0 - 0.1);
        assert!(tag_at(&g, g.marks)[1] + text_h(TAG_SIZE) / 2.0 < top);
        assert!(tag_at(&g, g.marks)[1] - text_h(TAG_SIZE) / 2.0 > g.top() + 0.05);
        // the title over the board, the legend under its numbers, clear of
        // the rail's frames
        assert!(g.title_at()[1] + text_h(TITLE_SIZE) / 2.0 < top);
        assert!(text_w(BOARD_TITLE, TITLE_SIZE) < g.right() - g.left());
        let mid = (g.left() + g.right()) / 2.0;
        assert!(mid - text_w(LEGEND, LEGEND_SIZE) / 2.0 > -edge);
        assert!(LEGEND_Y + text_h(LEGEND_SIZE) / 2.0 < g.position_label_at(0)[1] - text_h(POSITION_LABEL_SIZE) / 2.0);
        const {
            assert!(LEGEND_Y - LEGEND_SIZE * 0.0065 / 2.0 > FRAME_Y + FRAME_H / 2.0 + 0.1);
        }
        // the strip right of the board
        assert!(g.right() < STRIP_LEFT - 0.5);
        // the working, top right, over the strip's title
        let [asked, _] = ASKED_SECRETS;
        for x in PAIR {
            let row = check_row(&all[asked as usize], x, MARKS);
            assert!(STRIP_LEFT + text_w(&row, CHECK_SIZE) < edge, "{row}");
        }
        assert!(CHECK_Y[0] + text_h(CHECK_SIZE) / 2.0 < top);
        assert!(CHECK_Y[1] - text_h(CHECK_SIZE) / 2.0 > STRIP_TITLE_Y + 0.3);
        assert!(CHECK_Y[0] - CHECK_Y[1] > text_h(CHECK_SIZE) + 0.2);
        // the think ring, lower right, under the strip's caption
        const {
            assert!(THINK_X + THINK_R < FRAME_X_RADIUS - 0.3);
            assert!(THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2);
        }
        assert!(THINK_X + text_w(PAUSE_NOTE, 18.0) / 2.0 < edge);
        assert!(THINK_Y - THINK_R - 0.3 - text_h(18.0) / 2.0 > -FRAME_Y_RADIUS + 0.3);
        // the grid, lower right: clear of the board, under the caption, over
        // the frame's foot; its count and note right of it
        let right = GRID_X0 + CELL * f64::from(MARKS);
        let grid_top = GRID_Y0 + CELL * f64::from(MARKS);
        assert!(GRID_X0 - AXIS_GAP - text_w("10", AXIS_SIZE) / 2.0 > g.right() + 0.3);
        assert!(grid_top + AXIS_TITLE_GAP + 0.05 + text_h(AXIS_TITLE_SIZE) / 2.0 < STRIP_CAPTION_Y - 0.2);
        assert!(GRID_Y0 - AXIS_GAP - text_h(AXIS_SIZE) / 2.0 > -FRAME_Y_RADIUS + 0.25);
        assert!(right + AXIS_TITLE_GAP + text_w(FIRST, AXIS_TITLE_SIZE) < edge);
        assert!(COUNT_X > right + 0.1);
        assert!(COUNT_X + text_w(&grid_count(MARKS, 121), COUNT_SIZE) < edge);
        assert!(COUNT_X + text_w(FAIR, FAIR_SIZE) < edge);
        assert!(COUNT_Y + text_h(COUNT_SIZE) / 2.0 < STRIP_CAPTION_Y - 0.3);
        assert!(COUNT_Y - FAIR_Y > text_h(COUNT_SIZE) + 0.1);
        assert!(text_w("10", CELL_LABEL_SIZE) < CELL_IN);
    }
}
