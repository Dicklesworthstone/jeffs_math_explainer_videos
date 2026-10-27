//! Chapter 11, one more share: why chapter 10's count, not its picture, is
//! the proof, and what a third share does. The board comes back as chapter
//! 10 left it, the eleven curves through trustees 4 and 5's points with
//! their spins left of their secrets' rows. The spins' column is pointed at
//! as what the argument used; then, at the lower right, the paper plot,
//! "on paper: ordinary numbers", heights as they are: a smooth curve runs
//! through every secret and the pair's heights, 145 and 215, too (11_1).
//! At that height the eleven lie on top of one another, so a window beside
//! the plot, marked "enlarged" with its outline on the plot, draws
//! positions 0 to 1 bigger, where they start apart at their secrets. On
//! paper the secret 0's curve needs spins 9 + 1/4 and 6 + 3/4, no spin can
//! be that, and only the secret 5's spins are whole, so the paper strip
//! lights 5 alone (11_2). On the dial every curve has real spins, one pair
//! each, all equally likely: that count is the proof, and the paper goes
//! (11_3). Trustee 1 joins, the vault's 8 turning over in its frame, its
//! point on the board, and the viewer is asked how many of the eleven
//! curves pass it (11_4). Each curve's mark at position 1 is written under
//! its secret's tick, every one different; only the curve for 5 passes at
//! 8, the others go and the strip drops to 5 (11_5). The last beat is the
//! scene's own count over every pair of trustees, every two shares and
//! every third share, as a table (11_6).
//!
//! The board, its curves and the cards are chapter 10's (`two_shares.rs`):
//! on the dial a curve is only its points, and its guides are labelled
//! guides. Smooth curves are drawn only on the paper plot and its window.
//! The paper spins are `model::paper_curve_spins`, exact fractions; the
//! scene stops unless only the secret 5 gets whole spins, the paper strip
//! agrees, the curves pass position 1 at eleven different marks and the
//! vault's 8 leaves the secret 5 alone. Emphasis never restyles or scales
//! typeset text.

use std::collections::BTreeSet;

use explainer_kit::{Kit, Palette, Timed, fade_all, v};
use fmn::prelude::*;

use super::board::{Grid, now, ring};
use super::draw::{
    READ_CPS, STRIP_LEFT, Strip, card, disc, fade_in_all, fade_in_lagged, fail, pause, read_by, recolorings, say,
    stroke, swaps, think, tick_at, wait_until,
};
use super::envelope::{PAD, dealt, for_pair, frame_halos, holding, outline, swelling};
use super::two_keys::{check_strip, glow, glowing, leaving, text_halo};
use super::two_shares::{
    Candidate, DROP, Drawn, LEGEND, LEGEND_SIZE, LEGEND_Y, Opening, PAIR_BOARD, POINT_RING, TAG_SIZE, TAGS_TITLE,
    THINK_X, THINK_Y, candidates, column_outline, disc_ring, pair_points, tag_at, text_right, ticks_outline,
};
use crate::model::{
    self, ASKED_SECRETS, Dial, JOINER, PAIR, Ratio, Recipe, SECRET, SPINS, TRUSTEES, View, combinations, csv,
};
use crate::palette;

pub struct OneMoreShare {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const PAPER_TITLE: &str = "on paper: ordinary numbers";
const WINDOW_TITLE: &str = "near position 0, enlarged";

// ------------------------------------------------------------------ layout

/// The paper plot, lower right under the strip: positions 0 to 5 across,
/// heights 0 to 220 up, labelled every 50; position 0's place and one
/// step, height 0's place and one unit.
const PAPER_X0: f64 = 0.9;
const PAPER_DX: f64 = 0.5;
const PAPER_Y0: f64 = -3.35;
const PAPER_DY: f64 = 0.0095;
const PAPER_LAST: u32 = 5;
const PAPER_TOP: i64 = 220;
const PAPER_STEP: usize = 50;
/// The window right of it: the same plot's positions 0 to 1 and heights 0
/// to 25, three times wider and nine times taller, labelled every 5.
const WINDOW_X0: f64 = 4.7;
const WINDOW_DX: f64 = 3.0 * PAPER_DX;
const WINDOW_DY: f64 = 9.0 * PAPER_DY;
const WINDOW_LAST: u32 = 1;
const WINDOW_TOP: i64 = 25;
const WINDOW_STEP: usize = 5;
/// Both plots' axes: numbers' size, how far left of the side and under the
/// bottom they sit, a tick's length, the axes' width and how far the bottom
/// runs on past the last position.
const AXIS_LABEL_SIZE: f64 = 14.0;
const AXIS_LABEL_DX: f64 = 0.3;
const AXIS_LABEL_DY: f64 = 0.2;
const TICK: f64 = 0.06;
const AXIS_W: f64 = 2.0;
const RUN_ON: f64 = 0.25;
/// The two titles, on one line over the plots, left-aligned.
const TITLES_Y: f64 = -0.75;
const PAPER_TITLE_X: f64 = 0.45;
const PAPER_TITLE_SIZE: f64 = 18.0;
const WINDOW_TITLE_X: f64 = 4.3;
const WINDOW_TITLE_SIZE: f64 = 16.0;
/// A paper curve's width, sampled this many steps; the pair's points, their
/// rings and their heights' labels, up and to the left; a secret's point in
/// the window and its ring.
const PAPER_CURVE_W: f64 = 1.5;
const SAMPLES: usize = 60;
const PAPER_POINT_R: f64 = 0.06;
const PAPER_RING: f64 = 0.14;
const PAPER_LABEL_SIZE: f64 = 18.0;
const PAPER_LABEL_DX: f64 = -0.25;
const PAPER_LABEL_DY: f64 = 0.2;
const SECRET_DOT_R: f64 = 0.035;
const SECRET_RING: f64 = 0.09;
/// 11_2's paper spins and note, and 11_3's count, top right over the strip.
const PAPER_TAG_Y: [f64; 2] = [3.35, 2.95];
const PAPER_TAG_SIZE: f64 = 20.0;
const WHOLE_NOTE_Y: f64 = 2.6;
const NOTE_SIZE: f64 = 18.0;
const PROOF_Y: f64 = 3.35;
const PROOF_SIZE: f64 = 20.0;
/// 11_5's row under the strip's caption: its title, and each secret's
/// curve's mark at the joiner's position under that secret's tick.
const ROW_TITLE_Y: f64 = -0.5;
const ROW_TITLE_SIZE: f64 = 18.0;
const ROW_Y: f64 = -0.85;
const ROW_SIZE: f64 = 20.0;
/// 11_6's table, lower right: its lines, its two rows of pairs and the
/// pairs' spacing.
const TABLE_Y: [f64; 4] = [-1.2, -2.4, -2.85, -3.25];
const TABLE_PAIRS_Y: [f64; 2] = [-1.6, -1.95];
const TABLE_SIZE: f64 = 18.0;
const PAIRS_SIZE: f64 = 16.0;
const PAIR_DX: f64 = 1.15;
const PAIRS_A_ROW: usize = 5;
/// The hold on the one curve left, and after the table is read.
const HOLD_ON_ONE: f64 = 1.5;
const HOLD_AT_END: f64 = 0.8;

// -------------------------------------------------------------------- data

/// The pair's heights on paper, before wrapping: (4, 145) and (5, 215).
pub fn paper_pair(recipe: &Recipe) -> [(u32, i64); 2] {
    PAIR.map(|x| (x, recipe.height(x)))
}

/// For every secret, the spins of the one paper curve through it and
/// `points` (the model's Cramer's rule), as exact fractions.
pub fn paper_spins(dial: Dial, points: [(u32, i64); 2]) -> Result<Vec<(u32, Ratio, Ratio)>, String> {
    dial.all()
        .map(|s| {
            model::paper_curve_spins(points, s)
                .map(|(a, b)| (s, a, b))
                .ok_or_else(|| format!("no paper curve through the secret {s} and {points:?}"))
        })
        .collect()
}

/// The secrets whose paper spins are both whole marks, 0 to `marks` − 1.
pub fn whole_spin_secrets(spins: &[(u32, Ratio, Ratio)], marks: u32) -> Vec<u32> {
    let is_mark = |r: Ratio| r.whole().is_some_and(|w| (0..i64::from(marks)).contains(&w));
    spins
        .iter()
        .filter(|&&(_, a, b)| is_mark(a) && is_mark(b))
        .map(|&(s, _, _)| s)
        .collect()
}

/// A paper spin as its tag writes it: a whole number, or its whole part
/// plus the fraction left over: "7", "9 + 1/4".
pub fn spin_text(r: Ratio) -> String {
    match r.mixed() {
        (w, rest) if rest == Ratio::int(0) => w.to_string(),
        (0, rest) => rest.to_string(),
        (w, rest) => format!("{w} + {rest}"),
    }
}

/// The tag of the secret `s`'s paper curve: "secret 0: spins 9 + 1/4 and
/// 6 + 3/4".
pub fn paper_tag(s: u32, a: Ratio, b: Ratio) -> String {
    format!("secret {s}: spins {} and {}", spin_text(a), spin_text(b))
}

/// The paper curve s + a·x + b·x² from position `from` to `to`, sampled.
pub fn paper_curve(s: u32, a: Ratio, b: Ratio, from: f64, to: f64) -> Vec<(f64, f64)> {
    let (s, a, b) = (f64::from(s), a.to_f64(), b.to_f64());
    (0..=SAMPLES)
        .map(|i| {
            let x = from + (to - from) * i as f64 / SAMPLES as f64;
            (x, s + a * x + b * x * x)
        })
        .collect()
}

/// Each secret's curve's mark at position `x`, by secret.
pub fn marks_at(all: &[Candidate], x: u32) -> Vec<u32> {
    all.iter().map(|c| c.marks[x as usize]).collect()
}

/// The secrets whose curve passes position `x` at mark `y`.
pub fn secrets_through(all: &[Candidate], x: u32, y: u32) -> Vec<u32> {
    all.iter()
        .filter(|c| c.marks.get(x as usize) == Some(&y))
        .map(|c| c.secret)
        .collect()
}

/// What 11_6's table reports: how many pairs of trustees and pairs of
/// shares were tried, the different numbers of spin pairs a secret fitted
/// two shares in, and the different numbers of secrets a third share left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tally {
    pub pairs: usize,
    pub share_pairs: usize,
    pub ways: Vec<usize>,
    pub left: Vec<usize>,
}

/// Every pair of `trustees`, every two shares they could hold, and every
/// third trustee's every share: the curves through the two shares, one
/// per secret ([`candidates`]), and the secrets each third share leaves.
/// Where some secret does not fit exactly one spin pair, its counts are
/// recorded as they are and no third share is tried.
pub fn every_pair(dial: Dial, trustees: &[u32], last: u32) -> Tally {
    let pairs = combinations(trustees, 2);
    let share_pairs: Vec<Vec<u32>> = model::tuples(dial, 2).collect();
    let (mut ways, mut left) = (BTreeSet::new(), BTreeSet::new());
    for pair in &pairs {
        for values in &share_pairs {
            let held: Vec<(u32, u32)> = pair.iter().copied().zip(values.iter().copied()).collect();
            let Ok(all) = candidates(dial, &held, last) else {
                ways.extend(
                    dial.all()
                        .map(|s| model::fitting_spins(dial, &held, s, SPINS.len()).len()),
                );
                continue;
            };
            ways.insert(1);
            for &j in trustees.iter().filter(|j| !pair.contains(j)) {
                for y in dial.all() {
                    left.insert(secrets_through(&all, j, y).len());
                }
            }
        }
    }
    Tally {
        pairs: pairs.len(),
        share_pairs: share_pairs.len(),
        ways: ways.into_iter().collect(),
        left: left.into_iter().collect(),
    }
}

/// 11_2's note: what a spin can be.
fn whole_note(marks: u32) -> String {
    format!("a spin is a whole mark, 0 to {}", marks - 1)
}

/// 11_3's count: each secret's spin pairs among all of them.
fn proof_note(ways: &[usize], spin_pairs: usize) -> String {
    format!("each secret: exactly {} of the {spin_pairs} spin pairs", csv(ways))
}

/// The strip's caption for the pair on paper.
fn for_paper(pair: [u32; 2]) -> String {
    format!("for: trustees {} and {}, on paper", pair[0], pair[1])
}

/// The strip's caption for three trustees.
fn for_trio(trio: [u32; 3]) -> String {
    format!("for: trustees {}, {} and {}", trio[0], trio[1], trio[2])
}

/// 11_5's row title.
fn at_joiner_title(x: u32) -> String {
    format!("each secret's curve at position {x}:")
}

/// 11_6's lines: the pairs of trustees, one pair, the pairs of shares,
/// what two shares leave and what a third does.
fn table_head(pairs: usize) -> String {
    format!("every pair of trustees ({pairs} pairs):")
}

fn pair_text(pair: &[u32]) -> String {
    format!("{} and {}", pair[0], pair[1])
}

fn table_shares(share_pairs: usize) -> String {
    format!("each holding any of the {share_pairs} pairs of shares:")
}

fn table_two(shares: usize, ways: &[usize]) -> String {
    format!("{shares} shares: every secret fits exactly {} spin pair", csv(ways))
}

fn table_third(left: &[usize]) -> String {
    format!("+ any third share: exactly {} secret left", csv(left))
}

/// When a viewer still reading until `busy` has also read `text`, fully in
/// now (padlocks.rs's rule: one argument's lines are read one after
/// another).
fn read_on(stage: &Stage<'_>, busy: f64, text: &str) -> f64 {
    read_by(stage, text).max(busy + text.chars().count() as f64 / READ_CPS)
}

// ---------------------------------------------------------------- drawing

/// The scene point at position `x`, height `h`, on the paper plot.
fn paper_at(x: f64, h: f64) -> Vec3 {
    v(PAPER_X0 + PAPER_DX * x, PAPER_Y0 + PAPER_DY * h)
}

/// The same, on the window.
fn window_at(x: f64, h: f64) -> Vec3 {
    v(WINDOW_X0 + WINDOW_DX * x, PAPER_Y0 + WINDOW_DY * h)
}

/// Axes for a plot drawn by `at`: the bottom one, with a tick and a number
/// under each position 0 to `last`, running on a little, and the side up to
/// height `top`, with a tick and a number at every `step`th. Detached.
fn axes(
    kit: &Kit,
    stage: &mut Stage<'_>,
    at: fn(f64, f64) -> Vec3,
    last: u32,
    top: i64,
    step: usize,
) -> fmn::Result<Vec<Mob>> {
    let origin = at(0.0, 0.0);
    let right = at(f64::from(last), 0.0)[0] + RUN_ON;
    let mut parts = vec![
        stroke(stage, vec![origin, v(right, origin[1])], Palette::dim(), AXIS_W),
        stroke(stage, vec![origin, at(0.0, top as f64)], Palette::dim(), AXIS_W),
    ];
    for h in (0..=top).step_by(step) {
        let p = at(0.0, h as f64);
        if h > 0 {
            parts.push(stroke(stage, vec![p, v(p[0] - TICK, p[1])], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &h.to_string(),
            AXIS_LABEL_SIZE,
            Palette::dim(),
            v(p[0] - AXIS_LABEL_DX, p[1]),
        )?);
    }
    for x in 0..=last {
        let p = at(f64::from(x), 0.0);
        if x > 0 {
            parts.push(stroke(stage, vec![p, v(p[0], p[1] - TICK)], Palette::dim(), AXIS_W));
        }
        parts.push(kit.text(
            stage,
            &x.to_string(),
            AXIS_LABEL_SIZE,
            Palette::dim(),
            v(p[0], p[1] - AXIS_LABEL_DY),
        )?);
    }
    Ok(parts)
}

/// An outline `PAD` outside the rectangle from `lo` to `hi`, in `color`.
/// Detached.
fn outline_between(stage: &mut Stage<'_>, lo: Vec3, hi: Vec3, color: Srgb) -> fmn::Result<Mob> {
    outline(
        stage,
        v((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0),
        hi[0] - lo[0] + 2.0 * PAD,
        hi[1] - lo[1] + 2.0 * PAD,
        color,
    )
}

/// An outline round all of `mobs` as they stand, in `color`. Detached.
fn outline_round(stage: &mut Stage<'_>, mobs: &[Mob], color: Srgb) -> fmn::Result<Mob> {
    if mobs.is_empty() {
        return Err(fail("an outline round nothing".into()));
    }
    let (mut lo, mut hi) = (v(f64::INFINITY, f64::INFINITY), v(f64::NEG_INFINITY, f64::NEG_INFINITY));
    for &m in mobs {
        let (c, w, h) = (stage.get_center(m), stage.get_width(m), stage.get_height(m));
        lo = v(lo[0].min(c[0] - w / 2.0), lo[1].min(c[1] - h / 2.0));
        hi = v(hi[0].max(c[0] + w / 2.0), hi[1].max(c[1] + h / 2.0));
    }
    outline_between(stage, lo, hi, color)
}

/// The corners of the board with its row and position numbers.
fn board_bounds(grid: &Grid) -> (Vec3, Vec3) {
    let lo = v(grid.row_label_at(0)[0] - 0.1, grid.position_label_at(0)[1] - 0.1);
    (lo, v(grid.right(), grid.top()))
}

/// The corners of the paper plot and its window, with their numbers and
/// titles.
fn paper_bounds() -> (Vec3, Vec3) {
    let lo = v(PAPER_TITLE_X, PAPER_Y0 - AXIS_LABEL_DY - 0.05);
    let hi = v(window_at(f64::from(WINDOW_LAST), 0.0)[0] + RUN_ON, TITLES_Y + 0.08);
    (lo, hi)
}

/// The card turning over: its back lifts away, then its number drops in,
/// prepared (chapter 09's).
fn turning_over(stage: &mut Stage<'_>, back: Mob, face: Mob) -> fmn::Result<Box<dyn Animation>> {
    let a = fade_out(stage.arena_mut(), back, v(0.0, DROP), 1.0)?;
    let out = stage.prepare(a.rt(0.2))?;
    let a = fade_in(stage.arena_mut(), face, v(0.0, -DROP), 1.0)?;
    let inn = stage.prepare(a.rt(0.25))?;
    let both = Succession::new(stage.arena_mut(), vec![out, inn])?;
    stage.prepare(both)
}

/// The paper comparison, lower right: the plot with the pair's two heights
/// and a smooth curve through each secret and them, and the window near
/// position 0 where the curves stand apart at their secrets. Detached.
struct Paper {
    /// Both plots' axes and titles, and the window's outline on the plot
    /// and round the window.
    frame: Vec<Mob>,
    /// The pair's two points, and their heights' labels.
    points: Vec<Mob>,
    labels: Vec<Mob>,
    /// Each secret's curve, on the plot and in the window, by secret.
    curves: Vec<[Mob; 2]>,
    /// Each secret's point in the window, by secret.
    secrets: Vec<Mob>,
}

impl Paper {
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        pair: [(u32, i64); 2],
        spins: &[(u32, Ratio, Ratio)],
    ) -> fmn::Result<Self> {
        let mut frame = axes(kit, stage, paper_at, PAPER_LAST, PAPER_TOP, PAPER_STEP)?;
        frame.extend(axes(kit, stage, window_at, WINDOW_LAST, WINDOW_TOP, WINDOW_STEP)?);
        frame.push(kit.text_left(
            stage,
            PAPER_TITLE,
            PAPER_TITLE_SIZE,
            Palette::dim(),
            v(PAPER_TITLE_X, TITLES_Y),
        )?);
        frame.push(kit.text_left(
            stage,
            WINDOW_TITLE,
            WINDOW_TITLE_SIZE,
            Palette::dim(),
            v(WINDOW_TITLE_X, TITLES_Y),
        )?);
        let corner = (f64::from(WINDOW_LAST), WINDOW_TOP as f64);
        frame.push(outline_between(
            stage,
            paper_at(0.0, 0.0),
            paper_at(corner.0, corner.1),
            Palette::dim(),
        )?);
        frame.push(outline_between(
            stage,
            window_at(0.0, 0.0),
            window_at(corner.0, corner.1),
            Palette::dim(),
        )?);
        let (mut points, mut labels) = (Vec::new(), Vec::new());
        for &(x, h) in &pair {
            let p = paper_at(f64::from(x), h as f64);
            points.push(disc(stage, palette::pair(), p, PAPER_POINT_R));
            labels.push(kit.text(
                stage,
                &h.to_string(),
                PAPER_LABEL_SIZE,
                palette::pair(),
                v(p[0] + PAPER_LABEL_DX, p[1] + PAPER_LABEL_DY),
            )?);
        }
        let (mut curves, mut secrets) = (Vec::new(), Vec::new());
        for &(s, a, b) in spins {
            let on_plot: Vec<Vec3> = paper_curve(s, a, b, 0.0, f64::from(PAPER_LAST))
                .into_iter()
                .map(|(x, h)| paper_at(x, h))
                .collect();
            let in_window: Vec<Vec3> = paper_curve(s, a, b, 0.0, f64::from(WINDOW_LAST))
                .into_iter()
                .map(|(x, h)| window_at(x, h))
                .collect();
            curves.push([
                stroke(stage, on_plot, Palette::dim(), PAPER_CURVE_W),
                stroke(stage, in_window, Palette::dim(), PAPER_CURVE_W),
            ]);
            secrets.push(disc(
                stage,
                palette::secret(),
                window_at(0.0, f64::from(s)),
                SECRET_DOT_R,
            ));
        }
        Ok(Self {
            frame,
            points,
            labels,
            curves,
            secrets,
        })
    }

    /// Everything it is drawn with.
    fn parts(&self) -> Vec<Mob> {
        let mut all = self.frame.clone();
        all.extend(self.curves.iter().flatten());
        all.extend(&self.secrets);
        all.extend(&self.points);
        all.extend(&self.labels);
        all
    }
}

impl SceneConstruct for OneMoreShare {
    fn name(&self) -> &str {
        "11_one_more_share"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let marks = dial.marks();
        let recipe = Recipe::example();
        let last = PAIR_BOARD.last;
        // chapter 10's curves through the pair's cards, one spin pair each
        let held = dealt(dial, &PAIR);
        let all = candidates(dial, &held, last).map_err(fail)?;
        let ways: Vec<usize> = dial
            .all()
            .map(|s| model::fitting_spins(dial, &held, s, SPINS.len()).len())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let spin_pairs = model::tuples(dial, SPINS.len()).count();
        // on paper: a curve through every secret and the pair's heights,
        // whole spins for the secret 5 alone, and the paper strip agrees
        let paper = paper_pair(&recipe);
        let on_paper = paper_spins(dial, paper).map_err(fail)?;
        let whole = whole_spin_secrets(&on_paper, marks);
        let paper_view = View::Paper {
            points: paper.to_vec(),
            spins: SPINS.len(),
        };
        if whole != [SECRET] || paper_view.possible(dial) != whole {
            return Err(fail(format!("on paper the secrets with whole spins are {whole:?}")));
        }
        let [asked, _] = ASKED_SECRETS;
        let (s0, a0, b0) = on_paper[asked as usize];
        let (s5, a5, b5) = on_paper[SECRET as usize];
        // the joiner: trustee 1's vault card, and the one curve through it
        let (jx, jy) = dealt(dial, &[JOINER])[0];
        let passing = marks_at(&all, jx);
        let left = secrets_through(&all, jx, jy);
        let different: BTreeSet<u32> = passing.iter().copied().collect();
        if different.len() != marks as usize || left != [SECRET] {
            return Err(fail(format!(
                "at position {jx} the curves pass {passing:?}, and {jy} leaves {left:?}"
            )));
        }
        let trio = [JOINER, PAIR[0], PAIR[1]];
        let trio_view = holding(dial, &trio);
        let back_index = TRUSTEES
            .iter()
            .filter(|t| !PAIR.contains(t))
            .position(|&t| t == JOINER)
            .ok_or_else(|| fail(format!("trustee {JOINER} has no card face down")))?;
        // every pair, every two shares, every third share: the table's count
        let tally = every_pair(dial, &TRUSTEES, last);
        if tally.ways != [1] || tally.left != [1] {
            return Err(fail(format!("over every pair: {tally:?}")));
        }
        let everyone: Vec<u32> = dial.all().collect();

        // the opening: chapter 10's end, the board with its eleven curves,
        // the pair's cards face up, and the pair's strip
        let open = Opening::build(&kit, stage, dial)?;
        let board = &open.board;
        let crowd = [Palette::dim(), palette::ghost()];
        let mut drawn = Vec::with_capacity(all.len());
        for c in &all {
            drawn.push(Drawn::build(&kit, stage, board, c, &held, crowd)?);
        }
        if drawn.iter().map(|d| d.secret).ne(dial.all()) {
            return Err(fail("the board does not hold one curve per secret".into()));
        }
        let (pair_dots, pair_guide) = pair_points(stage, board, &held)?;
        let title = text_right(
            &kit,
            stage,
            TAGS_TITLE,
            TAG_SIZE,
            Palette::dim(),
            tag_at(&board.grid, marks),
        )?;
        let legend = kit.text(
            stage,
            LEGEND,
            LEGEND_SIZE,
            Palette::dim(),
            v((board.grid.left() + board.grid.right()) / 2.0, LEGEND_Y),
        )?;
        open.rail.reveal(stage, 0.5)?;
        let mut cards = open.backs.clone();
        cards.extend(&open.pair_cards);
        let mut anims = swaps(stage, &[], &cards, 0.4)?;
        anims.extend(recolorings(stage, &open.tints(), 0.4)?);
        stage.play_prepared(anims)?;
        board.show(stage, 0.5)?;
        let mut pieces = vec![pair_guide];
        for d in &drawn {
            pieces.extend(d.parts());
        }
        pieces.extend(&pair_dots);
        pieces.extend([title, legend]);
        fade_in_all(stage, &pieces, 0.6)?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 0.6)?;
        strip.show(&kit, stage, &holding(dial, &PAIR), &for_pair(PAIR), 0.3)?;
        check_strip(&strip, &everyone, "for the pair")?;
        let tags: Vec<Mob> = drawn.iter().map(|d| d.tag).collect();

        // 11_1: what the argument used; the same picture on paper
        let spoken = say(&kit, stage, "11_1")?;
        spoken.wait_for(stage, "Notice what that argument used")?;
        let column = outline_round(stage, &tags, palette::spin())?;
        glow(stage, &[column], 1.0)?;
        spoken.wait_for(stage, "On paper, a curve also runs")?;
        let sheet = Paper::build(&kit, stage, paper, &on_paper)?;
        let mut anims = swaps(stage, &[], &sheet.frame, 0.5)?;
        let mut dots = sheet.points.clone();
        dots.extend(&sheet.labels);
        anims.extend(swaps(stage, &[], &dots, 0.5)?);
        stage.play_prepared(anims)?;
        // each secret's curve grows on the plot and in the window together
        let rt = (spoken.at("the same two points")? - now(stage)).clamp(1.0, 2.2);
        let mut growing = Vec::with_capacity(sheet.curves.len());
        for (&[on_plot, in_window], &dot) in sheet.curves.iter().zip(&sheet.secrets) {
            let mut both = vec![
                stage.prepare(show_creation(on_plot))?,
                stage.prepare(show_creation(in_window))?,
            ];
            let a = fade_in(stage.arena_mut(), dot, ORIGIN, 1.0)?;
            both.push(stage.prepare(a)?);
            let g = AnimationGroup::new(stage.arena_mut(), both)?;
            growing.push(stage.prepare(g)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), growing, 0.25)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "the same two points")?;
        let mut rings: Vec<Mob> = paper
            .iter()
            .map(|&(x, h)| ring(stage, paper_at(f64::from(x), h as f64), PAPER_RING, palette::pair()))
            .collect();
        for &(x, k) in &held {
            let at = board.grid.at(f64::from(x), f64::from(k));
            rings.push(ring(stage, at, POINT_RING, palette::pair()));
        }
        glow(stage, &rings, 0.8)?;
        spoken.wait_for(stage, "so the picture alone")?;
        let (lo, hi) = board_bounds(&board.grid);
        let mut pictures = vec![outline_between(stage, lo, hi, Palette::ink())?];
        let (lo, hi) = paper_bounds();
        pictures.push(outline_between(stage, lo, hi, Palette::ink())?);
        glow(stage, &pictures, 1.2)?;
        spoken.wait_end(stage)?;

        // 11_2: on paper only the secret 5 gets whole spins, so the heights
        // give it away
        let spoken = say(&kit, stage, "11_2")?;
        spoken.wait_for(stage, "their heights are")?;
        let halos = vec![
            text_halo(stage, sheet.labels[0], palette::pair())?,
            text_halo(stage, sheet.labels[1], palette::pair())?,
        ];
        glow(stage, &halos, 1.0)?;
        spoken.wait_for(stage, "For the secret zero")?;
        let first = sheet.curves[asked as usize];
        let inked: Vec<(Mob, Srgb)> = first.iter().map(|&m| (m, Palette::ink())).collect();
        let mut anims = recolorings(stage, &inked, 0.5)?;
        let r = ring(stage, window_at(0.0, f64::from(s0)), SECRET_RING, palette::secret());
        anims.extend(glowing(stage, &[r], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "needs spins of nine and a quarter")?;
        let tag0 = kit.text_left(
            stage,
            &paper_tag(s0, a0, b0),
            PAPER_TAG_SIZE,
            Palette::ink(),
            v(STRIP_LEFT, PAPER_TAG_Y[0]),
        )?;
        fade_in_all(stage, &[tag0], 0.4)?;
        spoken.wait_for(stage, "and no spin can be that")?;
        let note = kit.text_left(
            stage,
            &whole_note(marks),
            NOTE_SIZE,
            Palette::dim(),
            v(STRIP_LEFT, WHOLE_NOTE_Y),
        )?;
        let ghosted: Vec<(Mob, Srgb)> = first.iter().map(|&m| (m, palette::ghost())).collect();
        let mut anims = swaps(stage, &[], &[note], 0.4)?;
        anims.extend(recolorings(stage, &ghosted, 0.5)?);
        let halo = text_halo(stage, tag0, palette::leak())?;
        anims.extend(glowing(stage, &[halo], 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Only the secret five gets whole spins")?;
        let tag5 = kit.text_left(
            stage,
            &paper_tag(s5, a5, b5),
            PAPER_TAG_SIZE,
            palette::share(),
            v(STRIP_LEFT, PAPER_TAG_Y[1]),
        )?;
        let mut changes = Vec::new();
        for (&(s, _, _), both) in on_paper.iter().zip(&sheet.curves) {
            let color = if whole.contains(&s) {
                palette::share()
            } else {
                palette::ghost()
            };
            changes.extend(both.iter().map(|&m| (m, color)));
        }
        let mut anims = swaps(stage, &[], &[tag5], 0.4)?;
        anims.extend(recolorings(stage, &changes, 0.6)?);
        let r = ring(stage, window_at(0.0, f64::from(s5)), SECRET_RING, palette::share());
        anims.extend(glowing(stage, &[r], 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "give the secret away")?;
        strip.show(&kit, stage, &paper_view, &for_paper(PAIR), 0.6)?;
        check_strip(&strip, &whole, "for the pair, on paper")?;
        let ticks = ticks_outline(stage, marks, palette::leak())?;
        glow(stage, &[ticks], 0.9)?;
        spoken.wait_end(stage)?;

        // 11_3: on the dial every curve has real spins, one pair each: the
        // count is the proof, and the paper goes
        let spoken = say(&kit, stage, "11_3")?;
        spoken.wait_for(stage, "On the dial, every one")?;
        let column = outline_round(stage, &tags, palette::spin())?;
        let mut anims = leaving(stage, &[tag0, tag5, note], ORIGIN, 0.5)?;
        anims.extend(glowing(stage, &[column], 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "comes from real spins")?;
        strip.show(&kit, stage, &holding(dial, &PAIR), &for_pair(PAIR), 0.6)?;
        check_strip(&strip, &everyone, "for the pair, on the dial")?;
        spoken.wait_for(stage, "one pair each")?;
        let mut halos = Vec::with_capacity(tags.len());
        for &t in &tags {
            halos.push(text_halo(stage, t, palette::spin())?);
        }
        glow(stage, &halos, 1.0)?;
        spoken.wait_for(stage, "all equally likely")?;
        let proof = kit.text_left(
            stage,
            &proof_note(&ways, spin_pairs),
            PROOF_SIZE,
            palette::spin(),
            v(STRIP_LEFT, PROOF_Y),
        )?;
        fade_in_all(stage, &[proof], 0.5)?;
        spoken.wait_for(stage, "That count is the proof")?;
        let mut anims = leaving(stage, &sheet.parts(), ORIGIN, 0.6)?;
        let halo = text_halo(stage, proof, palette::spin())?;
        anims.extend(glowing(stage, &[halo], 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "the pictures only help us see it")?;
        let (lo, hi) = board_bounds(&board.grid);
        let picture = outline_between(stage, lo, hi, Palette::ink())?;
        glow(stage, &[picture], 1.0)?;
        spoken.wait_end(stage)?;

        // 11_4: trustee 1 joins with the vault's 8: how many curves pass it?
        let spoken = say(&kit, stage, "11_4")?;
        spoken.wait_for(stage, "Now let trustee one join them")?;
        let r = disc_ring(stage, JOINER, palette::trio());
        let mut anims = leaving(stage, &[proof], ORIGIN, 0.4)?;
        anims.extend(recolorings(stage, &open.rail.tint(&[JOINER], palette::trio()), 0.5)?);
        anims.extend(glowing(stage, &[r], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "holding eight")?;
        let face = card(&kit, stage, JOINER, &jy.to_string(), palette::share())?;
        let mut anims = vec![turning_over(stage, open.backs[back_index], face)?];
        let halos = frame_halos(stage, &[JOINER], palette::trio())?;
        anims.extend(glowing(stage, &halos, 0.9)?);
        stage.play_prepared(anims)?;
        let joint = board.point(stage, jx, jy, palette::trio());
        let r = ring(
            stage,
            board.grid.at(f64::from(jx), f64::from(jy)),
            POINT_RING,
            palette::trio(),
        );
        let row = board
            .row_label(jy)
            .ok_or_else(|| fail(format!("the board has no row {jy}")))?;
        let row_halo = text_halo(stage, row, palette::trio())?;
        let mut anims = swaps(stage, &[], &[joint], 0.4)?;
        anims.extend(glowing(stage, &[r, row_halo], 0.9)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "pass through eight at position one")?;
        let column = column_outline(stage, &board.grid, jx, palette::trio())?;
        glow(stage, &[column], 1.0)?;
        spoken.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 11_5: just one; each curve passes position 1 at its own mark, and
        // only the curve for 5 at 8
        let spoken = say(&kit, stage, "11_5")?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Just one")?;
        let dealer = &drawn[SECRET as usize];
        let r = ring(stage, dealer.centres[jx as usize], POINT_RING, palette::trio());
        let mut anims = vec![swelling(stage, joint, 1.6, palette::trio(), 0.8)?];
        anims.extend(glowing(stage, &[r], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Each curve passes position one")?;
        let head = kit.text_left(
            stage,
            &at_joiner_title(jx),
            ROW_TITLE_SIZE,
            Palette::dim(),
            v(STRIP_LEFT, ROW_TITLE_Y),
        )?;
        fade_in_all(stage, &[head], 0.4)?;
        // each secret in turn: its curve's point at position 1 ringed, its
        // mark there written under its tick
        let rt = (spoken.at("at a different mark")? - now(stage)).clamp(1.6, 3.0);
        let mut values = Vec::with_capacity(drawn.len());
        let mut each = Vec::with_capacity(drawn.len());
        for (d, &k) in drawn.iter().zip(&passing) {
            let value = kit.text(
                stage,
                &k.to_string(),
                ROW_SIZE,
                Palette::ink(),
                v(tick_at(d.secret)[0], ROW_Y),
            )?;
            let r = ring(stage, d.centres[jx as usize], POINT_RING, Palette::ink());
            let mut both = swaps(stage, &[], &[value], 1.0)?;
            both.extend(glowing(stage, &[r], 1.0)?);
            let g = AnimationGroup::new(stage.arena_mut(), both)?;
            each.push(stage.prepare(g)?);
            values.push(value);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), each, 0.3)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_for(stage, "at a different mark")?;
        let column = column_outline(stage, &board.grid, jx, palette::trio())?;
        let row = outline_round(stage, &values, Palette::ink())?;
        glow(stage, &[column, row], 1.0)?;
        spoken.wait_for(stage, "only the curve for five passes at eight")?;
        let (mut changes, mut gone) = (Vec::new(), Vec::new());
        for d in &drawn {
            if left.contains(&d.secret) {
                changes.extend(d.tinted([palette::share(); 2]));
            } else {
                changes.extend(d.tinted([palette::ghost(); 2]));
                changes.push((d.points[0], palette::ghost()));
                gone.push(d.tag);
            }
        }
        let mut anims = recolorings(stage, &changes, 0.6)?;
        anims.extend(leaving(stage, &gone, ORIGIN, 0.6)?);
        let mut halos = dealer.rings(stage, palette::share());
        halos.push(text_halo(stage, values[SECRET as usize], palette::trio())?);
        halos.push(text_halo(stage, dealer.tag, palette::spin())?);
        anims.extend(glowing(stage, &halos, 1.0)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "The strip drops from eleven secrets to one")?;
        let with = recolorings(stage, &open.rail.tint(&trio, palette::trio()), 0.6)?;
        strip.show_with(&kit, stage, &trio_view, &for_trio(trio), 0.6, with)?;
        check_strip(&strip, &left, "for the trio")?;
        spoken.wait_end(stage)?;
        pause(stage, HOLD_ON_ONE)?;

        // 11_6: the same count for every pair, every two shares and every
        // third share, as the scene counted them above
        let spoken = say(&kit, stage, "11_6")?;
        let mut going = vec![head];
        going.extend(&values);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "The same count holds for every pair of trustees")?;
        let words = table_head(tally.pairs);
        let first_line = kit.text_left(stage, &words, TABLE_SIZE, Palette::ink(), v(STRIP_LEFT, TABLE_Y[0]))?;
        fade_in_all(stage, &[first_line], 0.4)?;
        let mut read = read_by(stage, &words);
        let pairs = combinations(&TRUSTEES, 2);
        let mut pair_mobs = Vec::with_capacity(pairs.len());
        for (i, pair) in pairs.iter().enumerate() {
            let at = v(
                STRIP_LEFT + PAIR_DX * (i % PAIRS_A_ROW) as f64,
                TABLE_PAIRS_Y[i / PAIRS_A_ROW],
            );
            pair_mobs.push(kit.text_left(stage, &pair_text(pair), PAIRS_SIZE, Palette::ink(), at)?);
        }
        fade_in_lagged(stage, &pair_mobs, 1.2, 0.2)?;
        let listed: Vec<String> = pairs.iter().map(|p| pair_text(p)).collect();
        read = read_on(stage, read, &listed.join("   "));
        spoken.wait_for(stage, "every two shares they could hold")?;
        let words = table_shares(tally.share_pairs);
        let shares_line = kit.text_left(stage, &words, TABLE_SIZE, Palette::ink(), v(STRIP_LEFT, TABLE_Y[1]))?;
        fade_in_all(stage, &[shares_line], 0.4)?;
        read = read_on(stage, read, &words);
        spoken.wait_for(stage, "Two shares leave all eleven secrets")?;
        let words = table_two(PAIR.len(), &tally.ways);
        let two_line = kit.text_left(stage, &words, TABLE_SIZE, palette::pair(), v(STRIP_LEFT, TABLE_Y[2]))?;
        fade_in_all(stage, &[two_line], 0.4)?;
        read = read_on(stage, read, &words);
        spoken.wait_for(stage, "any third share leaves exactly one")?;
        let words = table_third(&tally.left);
        let third_line = kit.text_left(stage, &words, TABLE_SIZE, palette::trio(), v(STRIP_LEFT, TABLE_Y[3]))?;
        fade_in_all(stage, &[third_line], 0.4)?;
        read = read_on(stage, read, &words);
        let halo = text_halo(stage, third_line, palette::trio())?;
        glow(stage, &[halo], 0.9)?;
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
    use std::collections::BTreeMap;

    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS, STROKE_WIDTH_CONVERSION};

    use super::super::draw::{STRIP_CAPTION_Y, STRIP_DX, STRIP_TITLE_Y};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, TEN, THRESHOLD};

    /// The production narration: registering chapter 11 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// A share as the definition writes it, wrapped onto `p` marks: apart
    /// from the model's Horner rule.
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
            ("11_1", "Notice what that argument used"),
            ("11_1", "On paper, a curve also runs"),
            ("11_1", "the same two points"),
            ("11_1", "so the picture alone"),
            ("11_2", "their heights are"),
            ("11_2", "For the secret zero"),
            ("11_2", "needs spins of nine and a quarter"),
            ("11_2", "and no spin can be that"),
            ("11_2", "Only the secret five gets whole spins"),
            ("11_2", "give the secret away"),
            ("11_3", "On the dial, every one"),
            ("11_3", "comes from real spins"),
            ("11_3", "one pair each"),
            ("11_3", "all equally likely"),
            ("11_3", "That count is the proof"),
            ("11_3", "the pictures only help us see it"),
            ("11_4", "Now let trustee one join them"),
            ("11_4", "holding eight"),
            ("11_4", "pass through eight at position one"),
            ("11_5", "Just one"),
            ("11_5", "Each curve passes position one"),
            ("11_5", "at a different mark"),
            ("11_5", "only the curve for five passes at eight"),
            ("11_5", "The strip drops from eleven secrets to one"),
            ("11_6", "The same count holds for every pair of trustees"),
            ("11_6", "every two shares they could hold"),
            ("11_6", "Two shares leave all eleven secrets"),
            ("11_6", "any third share leaves exactly one"),
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

    /// What the scene draws and writes is the facts': the paper heights and
    /// spins and their tags, the joiner's card and the marks at its
    /// position, what it leaves, the table's counts and every caption.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let dial = Dial::vault();
        let paper = paper_pair(&Recipe::example());
        assert_eq!(f.get("S-F7.paper.heights"), Some(csv(&paper.map(|p| p.1)).as_str()));
        let on_paper = paper_spins(dial, paper).unwrap();
        assert_eq!(f.get("S-F7.paper.curves"), Some(on_paper.len().to_string().as_str()));
        let whole = whole_spin_secrets(&on_paper, MARKS);
        assert_eq!(f.get("S-F7.paper.whole_spin_secrets"), Some(csv(&whole).as_str()));
        let [asked, _] = ASKED_SECRETS;
        let (s0, a0, b0) = on_paper[asked as usize];
        assert_eq!(f.get("S-F7.asked.first"), Some(s0.to_string().as_str()));
        assert_eq!(f.get("S-F7.paper.secret0_spins"), Some(csv(&[a0, b0]).as_str()));
        assert_eq!(paper_tag(s0, a0, b0), "secret 0: spins 9 + 1/4 and 6 + 3/4");
        let (s5, a5, b5) = on_paper[SECRET as usize];
        assert_eq!(f.get("S-F7.paper.secret5_spins"), Some(csv(&[a5, b5]).as_str()));
        assert_eq!(paper_tag(s5, a5, b5), "secret 5: spins 7 and 7");
        // the joiner, and what its card leaves
        let held = dealt(dial, &PAIR);
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        let (jx, jy) = dealt(dial, &[JOINER])[0];
        assert_eq!(f.get("S-F7.join.trustee"), Some(jx.to_string().as_str()));
        assert_eq!(f.get("S-F7.join.share"), Some(jy.to_string().as_str()));
        assert_eq!(f.get("S-F7.join.point"), Some(csv(&[jx, jy]).as_str()));
        let passing = marks_at(&all, jx);
        assert_eq!(f.get("S-F7.join.values_by_secret"), Some(csv(&passing).as_str()));
        assert_eq!(passing, vec![5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0]);
        let left = secrets_through(&all, jx, jy);
        assert_eq!(f.get("S-F7.join.secrets_left"), Some(csv(&left).as_str()));
        assert_eq!(f.get("S-F7.join.count"), Some(left.len().to_string().as_str()));
        assert_eq!(f.get("S-F7.join.drop"), Some(csv(&[all.len(), left.len()]).as_str()));
        // the table's counts
        let tally = every_pair(dial, &TRUSTEES, PAIR_BOARD.last);
        assert_eq!(f.get("S-F7.all_pairs.pairs"), Some(tally.pairs.to_string().as_str()));
        assert_eq!(
            f.get("S-F7.all_pairs.share_pairs"),
            Some(tally.share_pairs.to_string().as_str())
        );
        assert_eq!(f.get("S-F7.all_pairs.ways_each"), Some(csv(&tally.ways).as_str()));
        assert_eq!(
            f.get("S-F7.all_pairs.two_leave"),
            Some(format!("{},{MARKS}", PAIR.len()).as_str())
        );
        assert_eq!(
            f.get("S-F7.all_pairs.third_leaves"),
            Some(format!("{THRESHOLD},{}", csv(&tally.left)).as_str())
        );
        // the words on screen
        assert_eq!(table_head(tally.pairs), "every pair of trustees (10 pairs):");
        assert_eq!(
            table_shares(tally.share_pairs),
            "each holding any of the 121 pairs of shares:"
        );
        assert_eq!(
            table_two(PAIR.len(), &tally.ways),
            "2 shares: every secret fits exactly 1 spin pair"
        );
        assert_eq!(table_third(&tally.left), "+ any third share: exactly 1 secret left");
        assert_eq!(pair_text(&[4, 5]), "4 and 5");
        let spin_pairs = model::tuples(dial, 2).count();
        assert_eq!(f.get("S-F6.spin_pairs"), Some(spin_pairs.to_string().as_str()));
        assert_eq!(f.get("S-F7.ways_each"), Some("1"));
        assert_eq!(
            proof_note(&[1], spin_pairs),
            "each secret: exactly 1 of the 121 spin pairs"
        );
        assert_eq!(whole_note(MARKS), "a spin is a whole mark, 0 to 10");
        assert_eq!(for_paper(PAIR), "for: trustees 4 and 5, on paper");
        assert_eq!(for_trio([JOINER, PAIR[0], PAIR[1]]), "for: trustees 1, 4 and 5");
        assert_eq!(at_joiner_title(jx), "each secret's curve at position 1:");
    }

    /// Every pair of trustees and every third trustee, with all 1331
    /// recipes dealt by the definition, apart from the model: two shares
    /// come from each secret exactly once for every pair of values, and
    /// three shares from exactly one recipe, whose secret the model's
    /// rebuild finds. The scene's own count, from chapter 10's curves, says
    /// the same.
    #[test]
    fn a_third_share_leaves_one_secret_for_every_pair() {
        let dial = Dial::vault();
        let p = MARKS;
        let recipes: Vec<[u32; 3]> = (0..p)
            .flat_map(|s| spin_pairs(p).map(move |[a, b]| [s, a, b]))
            .collect();
        assert_eq!(recipes.len(), 1331);
        let everyone: Vec<u32> = (0..p).collect();
        for pair in combinations(&TRUSTEES, 2) {
            let mut two: BTreeMap<[u32; 2], Vec<u32>> = BTreeMap::new();
            for &[s, a, b] in &recipes {
                let k = [share(s, a, b, pair[0], p), share(s, a, b, pair[1], p)];
                two.entry(k).or_default().push(s);
            }
            assert_eq!(two.len(), 121, "{pair:?}");
            for (values, mut secrets) in two {
                secrets.sort_unstable();
                assert_eq!(secrets, everyone, "{pair:?} holding {values:?}");
            }
            for j in TRUSTEES.iter().copied().filter(|j| !pair.contains(j)) {
                let mut three: BTreeMap<[u32; 3], Vec<u32>> = BTreeMap::new();
                for &[s, a, b] in &recipes {
                    let k = [
                        share(s, a, b, pair[0], p),
                        share(s, a, b, pair[1], p),
                        share(s, a, b, j, p),
                    ];
                    three.entry(k).or_default().push(s);
                }
                assert_eq!(three.len(), recipes.len(), "{pair:?} and {j}");
                for (k, secrets) in three {
                    assert_eq!(secrets.len(), 1, "{pair:?} and {j} holding {k:?}");
                    let points = [(pair[0], k[0]), (pair[1], k[1]), (j, k[2])];
                    assert_eq!(model::rebuild(dial, &points), Some(secrets[0]), "{points:?}");
                }
            }
        }
        let tally = every_pair(dial, &TRUSTEES, PAIR_BOARD.last);
        let want = Tally {
            pairs: 10,
            share_pairs: 121,
            ways: vec![1],
            left: vec![1],
        };
        assert_eq!(tally, want);
    }

    /// The wrong card: trustee 1 holding anything but the vault's 8 still
    /// leaves exactly one secret, never 5, and each card a different one,
    /// so the card picks the secret. The definition, apart from the model,
    /// agrees for every card.
    #[test]
    fn a_wrong_card_picks_another_secret() {
        let dial = Dial::vault();
        let held = dealt(dial, &PAIR);
        let all = candidates(dial, &held, PAIR_BOARD.last).unwrap();
        let (jx, jy) = dealt(dial, &[JOINER])[0];
        assert_eq!((jx, jy), (1, 8));
        let mut picked = Vec::new();
        for y in dial.all() {
            let left = secrets_through(&all, jx, y);
            assert_eq!(left.len(), 1, "card {y}");
            assert_eq!(left[0] == SECRET, y == jy, "card {y} leaves {left:?}");
            let by_definition: Vec<u32> = (0..MARKS)
                .filter(|&s| {
                    spin_pairs(MARKS).any(|[a, b]| {
                        held.iter().all(|&(x, k)| share(s, a, b, x, MARKS) == k) && share(s, a, b, jx, MARKS) == y
                    })
                })
                .collect();
            assert_eq!(by_definition, left, "card {y}");
            picked.push(left[0]);
        }
        picked.sort_unstable();
        assert_eq!(picked, dial.all().collect::<Vec<_>>());
    }

    /// The wrong modulus: on ten marks two shares fit some secrets no spin
    /// pair at all and others several, so the scene's count fails there,
    /// and the vault's pair, dealt on ten marks, gives no curve per secret.
    #[test]
    fn ten_marks_break_the_count() {
        let ten = Dial::new(TEN);
        let tally = every_pair(ten, &TRUSTEES, PAIR_BOARD.last);
        assert_ne!(tally.ways, vec![1]);
        assert!(
            tally.ways.contains(&0) && tally.ways.iter().any(|&w| w > 1),
            "{tally:?}"
        );
        let held: Vec<(u32, u32)> = PAIR
            .iter()
            .map(|&x| (x, share(SECRET, SPINS[0], SPINS[1], x, TEN)))
            .collect();
        assert!(candidates(ten, &held, PAIR_BOARD.last).is_err());
    }

    /// On paper a curve runs through every secret and both heights, exactly,
    /// but only the secret 5's spins are whole marks: the model's fractions,
    /// a search over every recipe written out, and the paper strip all say
    /// the secret is 5. The dial's control: the same pair's shares leave
    /// every secret. A nudged height gives another secret away: on paper the
    /// heights leak.
    #[test]
    fn on_paper_the_heights_give_the_secret_away() {
        let dial = Dial::vault();
        let paper = paper_pair(&Recipe::example());
        assert_eq!(paper, [(4, 145), (5, 215)]);
        let on_paper = paper_spins(dial, paper).unwrap();
        assert_eq!(on_paper.len(), MARKS as usize);
        for &(s, a, b) in &on_paper {
            for (x, h) in paper {
                let x = i64::from(x);
                let at = Ratio::int(i64::from(s)).plus(a.times(x)).plus(b.times(x * x));
                assert_eq!(at, Ratio::int(h), "secret {s} at {x}");
            }
        }
        assert_eq!(whole_spin_secrets(&on_paper, MARKS), vec![SECRET]);
        let searched: Vec<u32> = (0..MARKS)
            .filter(|&s| {
                spin_pairs(MARKS).any(|[a, b]| paper.iter().all(|&(x, h)| i64::from(s + a * x + b * x * x) == h))
            })
            .collect();
        assert_eq!(searched, vec![SECRET]);
        let view = View::Paper {
            points: paper.to_vec(),
            spins: 2,
        };
        assert_eq!(view.possible(dial), vec![SECRET]);
        assert_eq!(holding(dial, &PAIR).possible(dial).len(), MARKS as usize);
        let nudged = [(4, 146), (5, 215)];
        assert_eq!(whole_spin_secrets(&paper_spins(dial, nudged).unwrap(), MARKS), vec![10]);
    }

    /// Paper spins are written as a whole part and what is left over.
    #[test]
    fn spins_are_written_as_mixed_numbers() {
        assert_eq!(spin_text(Ratio::new(37, 4)), "9 + 1/4");
        assert_eq!(spin_text(Ratio::new(27, 4)), "6 + 3/4");
        assert_eq!(spin_text(Ratio::int(7)), "7");
        assert_eq!(spin_text(Ratio::new(14, 2)), "7");
        assert_eq!(spin_text(Ratio::new(1, 2)), "1/2");
        assert_eq!(Ratio::new(37, 4).mixed(), (9, Ratio::new(1, 4)));
        assert!((Ratio::new(37, 4).to_f64() - 9.25).abs() < 1e-12);
    }

    /// On the plot neighbouring curves start closer than a curve is wide,
    /// which is why the window exists; in the window they stand apart at
    /// both of its ends, and every curve stays inside what it is drawn on.
    #[test]
    fn the_window_shows_the_curves_apart() {
        let dial = Dial::vault();
        let on_paper = paper_spins(dial, paper_pair(&Recipe::example())).unwrap();
        let width = PAPER_CURVE_W * STROKE_WIDTH_CONVERSION;
        let height = |&(s, a, b): &(u32, Ratio, Ratio), x: f64| f64::from(s) + a.to_f64() * x + b.to_f64() * x * x;
        let gaps = |x: f64, dy: f64| {
            let mut ys: Vec<f64> = on_paper.iter().map(|c| height(c, x) * dy).collect();
            ys.sort_by(f64::total_cmp);
            ys.windows(2).map(|w| w[1] - w[0]).fold(f64::INFINITY, f64::min)
        };
        assert!(gaps(0.0, PAPER_DY) < width, "on the plot the curves start apart");
        for x in [0.0, f64::from(WINDOW_LAST)] {
            assert!(gaps(x, WINDOW_DY) > 2.0 * width, "window, position {x}");
        }
        for &(s, a, b) in &on_paper {
            for (_, h) in paper_curve(s, a, b, 0.0, f64::from(PAPER_LAST)) {
                assert!((0.0..=PAPER_TOP as f64).contains(&h), "{s}: {h}");
            }
            for (_, h) in paper_curve(s, a, b, 0.0, f64::from(WINDOW_LAST)) {
                assert!((0.0..=WINDOW_TOP as f64).contains(&h), "{s}: {h}");
            }
        }
    }

    /// The pieces keep apart, inside the frame: the paper plot and window
    /// right of the board and under the strip's caption, the tags and notes
    /// over the strip, the row under it, the table at the lower right, and
    /// the outlines round the two pictures.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        let edge = FRAME_X_RADIUS - 0.3;
        let top = FRAME_Y_RADIUS - 0.3;
        let g = PAIR_BOARD;
        let caption_bottom = STRIP_CAPTION_Y - text_h(22.0) / 2.0;
        // the plot right of the board, its titles under the caption
        assert!(PAPER_X0 - AXIS_LABEL_DX - text_w("200", AXIS_LABEL_SIZE) / 2.0 > g.right() + 0.3);
        assert!(PAPER_TITLE_X > g.right() + 0.3);
        assert!(PAPER_TITLE_X + text_w(PAPER_TITLE, PAPER_TITLE_SIZE) < WINDOW_TITLE_X - 0.3);
        assert!(WINDOW_TITLE_X + text_w(WINDOW_TITLE, WINDOW_TITLE_SIZE) < edge);
        assert!(TITLES_Y + text_h(PAPER_TITLE_SIZE) / 2.0 < caption_bottom - 0.2);
        let titles_bottom = TITLES_Y - text_h(PAPER_TITLE_SIZE) / 2.0;
        // the tallest things under the titles: the 215's label, the side's
        // top numbers
        let tall = paper_at(5.0, 215.0)[1] + PAPER_LABEL_DY + text_h(PAPER_LABEL_SIZE) / 2.0;
        assert!(tall < titles_bottom - 0.1, "{tall}");
        assert!(paper_at(0.0, PAPER_TOP as f64)[1] < titles_bottom - 0.1);
        assert!(window_at(0.0, WINDOW_TOP as f64)[1] + text_h(AXIS_LABEL_SIZE) / 2.0 < titles_bottom - 0.1);
        // the window right of the plot, inside the frame; the numbers under
        // both over the frame's foot
        let plot_end = paper_at(f64::from(PAPER_LAST), 0.0)[0] + RUN_ON;
        assert!(WINDOW_X0 - AXIS_LABEL_DX - text_w("25", AXIS_LABEL_SIZE) / 2.0 > plot_end + 0.3);
        assert!(PAPER_Y0 - AXIS_LABEL_DY - text_h(AXIS_LABEL_SIZE) / 2.0 > -top);
        let (lo, hi) = paper_bounds();
        assert!(lo[1] - PAD > -top && hi[0] + PAD < edge && hi[1] + PAD < caption_bottom - 0.2);
        assert!(lo[1] < PAPER_Y0 - AXIS_LABEL_DY - text_h(AXIS_LABEL_SIZE) / 2.0);
        assert!(hi[1] > TITLES_Y + text_h(PAPER_TITLE_SIZE) / 2.0);
        // the board's outline between the tags and the row numbers, over
        // the legend and under the title
        let (lo, hi) = board_bounds(&g);
        assert!(lo[0] - PAD > tag_at(&g, 0)[0] + 0.05);
        assert!(lo[0] < g.row_label_at(10)[0] - text_w("10", 16.0) / 2.0);
        assert!(lo[1] - PAD > LEGEND_Y + text_h(LEGEND_SIZE) / 2.0);
        assert!(hi[1] + PAD < g.title_at()[1] - text_h(20.0) / 2.0);
        assert!(lo[1] < g.position_label_at(0)[1] - text_h(16.0) / 2.0);
        // 11_2's tags and note, and 11_3's count, over the strip's title
        let dial = Dial::vault();
        let on_paper = paper_spins(dial, paper_pair(&Recipe::example())).unwrap();
        for &(s, a, b) in &on_paper {
            assert!(STRIP_LEFT + text_w(&paper_tag(s, a, b), PAPER_TAG_SIZE) < edge, "{s}");
        }
        assert!(STRIP_LEFT + text_w(&proof_note(&[1], 121), PROOF_SIZE) < edge);
        assert!(STRIP_LEFT + text_w(&whole_note(MARKS), NOTE_SIZE) < edge);
        assert!(PAPER_TAG_Y[0] + text_h(PAPER_TAG_SIZE) / 2.0 < top);
        assert!(PROOF_Y + text_h(PROOF_SIZE) / 2.0 < top);
        assert!(PAPER_TAG_Y[0] - PAPER_TAG_Y[1] > text_h(PAPER_TAG_SIZE) + 0.2);
        assert!(PAPER_TAG_Y[1] - WHOLE_NOTE_Y > (text_h(PAPER_TAG_SIZE) + text_h(NOTE_SIZE)) / 2.0 + 0.15);
        assert!(WHOLE_NOTE_Y - text_h(NOTE_SIZE) / 2.0 > STRIP_TITLE_Y + text_h(24.0) / 2.0 + 0.2);
        // 11_5's row under the caption, a mark under each tick
        assert!(ROW_TITLE_Y + text_h(ROW_TITLE_SIZE) / 2.0 < caption_bottom - 0.15);
        assert!(ROW_TITLE_Y - ROW_Y > (text_h(ROW_TITLE_SIZE) + text_h(ROW_SIZE)) / 2.0 + 0.1);
        assert!(text_w("10", ROW_SIZE) < STRIP_DX - 0.1);
        assert!(STRIP_LEFT + text_w(&at_joiner_title(JOINER), ROW_TITLE_SIZE) < edge);
        // 11_6's table, lower right
        let tally = Tally {
            pairs: 10,
            share_pairs: 121,
            ways: vec![1],
            left: vec![1],
        };
        let lines = [
            table_head(tally.pairs),
            table_shares(tally.share_pairs),
            table_two(PAIR.len(), &tally.ways),
            table_third(&tally.left),
        ];
        for line in &lines {
            assert!(STRIP_LEFT + text_w(line, TABLE_SIZE) < edge, "{line}");
        }
        let widest_pair = text_w("1 and 2", PAIRS_SIZE);
        assert!(PAIR_DX > widest_pair + 0.3);
        assert!(STRIP_LEFT + PAIR_DX * (PAIRS_A_ROW - 1) as f64 + widest_pair < edge);
        assert_eq!(combinations(&TRUSTEES, 2).len(), PAIRS_A_ROW * TABLE_PAIRS_Y.len());
        // top to bottom, each line clear of the next
        let ys = [
            TABLE_Y[0],
            TABLE_PAIRS_Y[0],
            TABLE_PAIRS_Y[1],
            TABLE_Y[1],
            TABLE_Y[2],
            TABLE_Y[3],
        ];
        assert!(ys[0] + text_h(TABLE_SIZE) / 2.0 < ROW_Y - text_h(ROW_SIZE) / 2.0 - 0.1);
        assert!(ys[5] - text_h(TABLE_SIZE) / 2.0 > -top);
        assert!(ys.windows(2).all(|w| w[0] - w[1] > text_h(TABLE_SIZE) + 0.15));
    }
}
