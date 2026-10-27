//! Chapter 07, a line through the secret: the dealer's method on a simpler
//! rule, any two of five, on ordinary paper. The rule is announced on its
//! own card, and the five trustees' discs lift off the rail onto positions
//! 1 to 5 of a plot, the secret's gold disc under position 0, with the
//! secret 5 a gold point at height 5 (07_1). The yellow spare dial spins
//! once, to 4, and the line from the secret climbs 4 a step, each step
//! marked (07_2). Each trustee's share is the line's height at its own
//! position, 9, 13, 17, 21 and 25, which drop into the rail's cards (07_3).
//! Trustees 2 and 3 alone redraw the line through their two points, and it
//! meets position 0 at 5 (07_4). One point is not enough: a line pivots
//! through trustee 5's (5, 25), and the strip comes back for anyone
//! outside, all eleven lit, while the viewer is asked which secrets fit
//! (07_5). Only then does the fan of whole spins 0 to 10 come in: three of
//! its lines meet position 0 at a secret 0 to 10 (0, 5 and 10), the strip
//! drops from 11 to 3, and the other eight miss (07_6). The tall heights
//! leak as B's piece did in chapter 02 (its strip of 8 comes back), and a
//! dial stands by for the wrapping, which is chapter 08's (07_7).
//!
//! The heights, the slope, trustee 5's 25 and the fan's three secrets are
//! the model's (`Recipe`, `View::Paper`); the plot is `plot.rs`'s. Emphasis
//! never restyles or scales typeset text: a point label or a frame is
//! pointed at by an outline of its own, and only shapes (points, discs,
//! lines) change color or swell.

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, Timed, fade_all, tween, v};
use fmn::prelude::*;

use super::draw::{
    Rail, Strip, TRUSTEE_R, card, fade_in_all, fail, line_style, pause, read_by, recolorings, say, swaps, think,
    trustee_at, wait_until,
};
use super::envelope::{frame_halos, swelling};
use super::plot::{
    LAST, LINE_W, Marker, Plot, TALLEST, at, at_zero, band, marker_at, point, point_label, secret_name, segment,
    through,
};
use super::two_keys::{
    ARC_LABEL_IN, Face, SPARE_R, check_strip, glow, glowing, leaving, moving, ordinary_secrets, ordinary_view,
    text_halo,
};
use crate::model::{Dial, LINE_SLOPE, ORDINARY_PIECES, Recipe, SECRET, TEN_HOLDER, TRUSTEES, View};
use crate::palette;

pub struct ALineThroughTheSecret {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model)
const FOR_OUTSIDE: &str = "for: anyone outside";
const SPARE_TITLE: &str = "dealer's fair spin";

/// Whole turns the spare dial makes before it stops.
const TURNS: u32 = 2;
/// The pivoting line's slopes in 07_5: from the dealer's 4 down, then up
/// past it (fractions: a line through one point can climb anything).
const PIVOT: [f64; 2] = [1.6, 7.4];

// ------------------------------------------------------------------ layout

/// The rule card, top right.
const RULE_X: f64 = 3.6;
const RULE_Y: f64 = 3.3;
const RULE_SIZE: f64 = 28.0;
/// The spare dial (07_2), right of the plot, and its title over it.
const SPIN_X: f64 = 2.4;
const SPIN_Y: f64 = 1.2;
const SPIN_TITLE_DY: f64 = SPARE_R + 0.35;
const SPIN_TITLE_SIZE: f64 = 22.0;
/// A step's marks: the run's and rise's widths, and the rise's label, right
/// of the rise.
const RUN_W: f64 = 2.0;
const RISE_W: f64 = 3.0;
const STEP_LABEL_DX: f64 = 0.22;
const STEP_LABEL_SIZE: f64 = 18.0;
/// The guides up from each position to the line (07_3).
const GUIDE_W: f64 = 1.5;
/// How far a card drops into its frame as it comes in.
const DROP: f64 = 0.3;
/// A ring round a disc, outside it.
const RING_OUT: f64 = 0.08;
/// The think ring (as chapter 04's), under the strip's caption.
const THINK_X: f64 = 5.4;
const THINK_Y: f64 = -1.2;
/// The fan's line width.
const FAN_W: f64 = 2.5;
/// The dial that stands by for the wrapping (07_7), between the plot and
/// the strip (its labels 12.6 pt, near the spare dial's 13.5).
const WRAP_X: f64 = -0.3;
const WRAP_Y: f64 = 1.6;
const WRAP_R: f64 = 0.7;

/// The rule card: any `k` of `n`.
fn rule(k: usize, n: usize) -> String {
    format!("a simpler rule: any {k} of {n}")
}

/// A step's label: the climb the spin gives.
fn climb(slope: u32) -> String {
    format!("+{slope}")
}

/// The strip's caption for trustee `t` alone, on paper.
fn for_paper(t: u32) -> String {
    format!("for: trustee {t}, on paper")
}

/// The strip's caption for chapter 02's B and its piece `b`.
fn for_b(b: u32) -> String {
    format!("for: B, holding {b} (chapter 02)")
}

/// A signed whole number with a typographic minus.
fn signed(n: i64) -> String {
    if n < 0 {
        format!("\u{2212}{}", n.unsigned_abs())
    } else {
        n.to_string()
    }
}

/// "0, 5 and 10".
fn and_list(xs: &[u32]) -> String {
    let words: Vec<String> = xs.iter().map(ToString::to_string).collect();
    match words.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// 07_6's precisely card: on paper, the line through `(x, h)` with whole
/// spin a meets position 0 at h − x·a; spins 0 to marks − 1 give these,
/// and only `fits` are secrets on the dial.
fn paper_card(x: u32, h: i64, marks: u32, fits: &[u32]) -> String {
    let met: Vec<String> = (0..marks)
        .map(|a| signed(at_zero(i64::from(x), h, i64::from(a))))
        .collect();
    format!(
        "secret = {h} \u{2212} {x} \u{d7} spin; spins 0 to {top} give {}; only {} are secrets 0 to {top}",
        met.join(", "),
        and_list(fits),
        top = marks - 1,
    )
}

/// The fan through `(x, h)`: for each whole spin 0 to marks − 1, the
/// line's part inside the plot, where it meets position 0, and whether
/// that is one of `fits`.
type FanLine = ([(f64, f64); 2], i64, bool);

fn fan(x: u32, h: i64, marks: u32, fits: &[u32]) -> Vec<FanLine> {
    (0..marks)
        .filter_map(|a| {
            let zero = at_zero(i64::from(x), h, i64::from(a));
            let fits_here = u32::try_from(zero).is_ok_and(|z| fits.contains(&z));
            through(f64::from(x), h as f64, f64::from(a)).map(|ends| (ends, zero, fits_here))
        })
        .collect()
}

/// A ring of radius `r` round `at`, in `color`. Detached.
fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let c = Circle::new()
        .radius(r)
        .style(line_style(color, 3.0))
        .build()
        .moved_to(at);
    stage.arena_mut().add(c)
}

/// Rings round position `t`'s disc, one each, in `color`. Detached.
fn marker_rings(stage: &mut Stage<'_>, ts: &[u32], color: Srgb) -> Vec<Mob> {
    ts.iter()
        .map(|&t| ring(stage, marker_at(t), TRUSTEE_R + RING_OUT, color))
        .collect()
}

/// Strokes growing one after another, the whole taking `rt`.
fn create_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims = Vec::with_capacity(mobs.len());
    for &m in mobs {
        anims.push(stage.prepare(show_creation(m))?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// The scene's time now, in seconds.
fn now(stage: &Stage<'_>) -> f64 {
    stage.scene().time().to_f64()
}

impl SceneConstruct for ALineThroughTheSecret {
    fn name(&self) -> &str {
        "07_a_line_through_the_secret"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let needed = line.coefficients().len();
        let heights: Vec<i64> = TRUSTEES.iter().map(|&t| line.height(t)).collect();
        if heights.iter().any(|&h| h > TALLEST) || line.height(0) != i64::from(SECRET) {
            return Err(fail(format!(
                "the line's heights {heights:?} leave the plot, or it misses the secret"
            )));
        }
        // trustee 5 alone, on paper, and the secrets its height leaves
        let held = line.height(TEN_HOLDER);
        let view = View::Paper {
            points: vec![(TEN_HOLDER, held)],
            spins: line.spins.len(),
        };
        let fits = view.possible(dial);
        let fanned = fan(TEN_HOLDER, held, dial.marks(), &fits);
        let mut met: Vec<u32> = fanned
            .iter()
            .filter(|f| f.2)
            .filter_map(|f| u32::try_from(f.1).ok())
            .collect();
        met.sort_unstable();
        if met != fits || fanned.len() != dial.marks() as usize {
            return Err(fail(format!(
                "the fan meets {met:?} on the dial, the view leaves {fits:?}"
            )));
        }
        let b = ORDINARY_PIECES[1];

        // 07_1: a simpler rule, any two of five; the secret at position 0
        let spoken = say(&kit, stage, "07_1")?;
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        rail.reveal(stage, 0.6)?;
        spoken.wait_for(stage, "start with a simpler rule")?;
        let rule_card = kit.text(
            stage,
            &rule(needed, TRUSTEES.len()),
            RULE_SIZE,
            Palette::ink(),
            v(RULE_X, RULE_Y),
        )?;
        fade_in_all(stage, &[rule_card], 0.5)?;
        spoken.wait_for(stage, "Mark the secret")?;
        let plot = Plot::build(&kit, stage)?;
        plot.show(stage, 0.6)?;
        // the trustees' discs lift off the rail onto their positions: each
        // lifts from exactly over its rail disc, so it comes in unseen
        let mut lifts = Vec::with_capacity(TRUSTEES.len() + 2);
        for &t in &TRUSTEES {
            let m = Marker::build(&kit, stage, t, palette::share(), trustee_at(t))?;
            stage.add_to_scene(m.whole)?;
            lifts.push(moving(stage, m.whole, marker_at(t), 1.0)?);
        }
        let zero = Marker::build(&kit, stage, 0, palette::secret(), marker_at(0))?;
        let name = secret_name(&kit, stage)?;
        lifts.extend(swaps(stage, &[], &[zero.whole, name], 1.0)?);
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), lifts, 0.12)?.with_run_time(1.4);
        stage.play(g)?;
        spoken.wait_for(stage, "position zero")?;
        let rings = marker_rings(stage, &[0], palette::secret());
        glow(stage, &rings, 0.6)?;
        spoken.wait_for(stage, "height five")?;
        let secret_point = point(stage, 0.0, f64::from(SECRET), palette::secret());
        fade_in_all(stage, &[secret_point], 0.3)?;
        let five = plot
            .label(i64::from(SECRET))
            .ok_or_else(|| fail(format!("the side has no label {SECRET}")))?;
        let mut anims = vec![swelling(stage, secret_point, 1.6, palette::secret(), 0.6)?];
        let halo = text_halo(stage, five, palette::secret())?;
        anims.extend(glowing(stage, &[halo], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;

        // 07_2: one fair spin, 4; the line climbs 4 a step from the secret
        let spoken = say(&kit, stage, "07_2")?;
        let spare = Face::build(&kit, stage, dial, v(SPIN_X, SPIN_Y), SPARE_R, palette::spin(), 0)?;
        let title = kit.text(
            stage,
            SPARE_TITLE,
            SPIN_TITLE_SIZE,
            palette::spin(),
            v(SPIN_X, SPIN_Y + SPIN_TITLE_DY),
        )?;
        fade_in_all(stage, &[title], 0.4)?;
        spare.show(stage, 0.4)?;
        spoken.wait_for(stage, "spins once")?;
        // two whole turns and on to where it stops, slowing down
        let stop = f64::from(LINE_SLOPE + TURNS * dial.marks());
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
        spoken.wait_for(stage, "say it gives four")?;
        let tag = kit.text(
            stage,
            &climb(LINE_SLOPE),
            STEP_LABEL_SIZE,
            palette::spin(),
            spare.point(f64::from(LINE_SLOPE), SPARE_R * ARC_LABEL_IN),
        )?;
        let spun = spare.halo(stage, LINE_SLOPE, palette::spin());
        let mut anims = swaps(stage, &[], &[tag], 0.4)?;
        anims.extend(glowing(stage, &[spun], 0.6)?);
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Draw the straight line")?;
        let top = (f64::from(LAST), line.height(LAST) as f64);
        let drawn = segment(stage, (0.0, f64::from(SECRET)), top, Palette::ink(), LINE_W);
        let a = stage.prepare(show_creation(drawn).rt(1.4))?;
        stage.play_prepared(vec![a])?;
        spoken.wait_for(stage, "climbs four for every step")?;
        // each step: a run of one position, then a rise of the spin
        let (mut strokes, mut labels) = (Vec::new(), Vec::new());
        for x in 0..LAST {
            let (h0, h1) = (line.height(x) as f64, line.height(x + 1) as f64);
            let (x0, x1) = (f64::from(x), f64::from(x + 1));
            strokes.push(segment(stage, (x0, h0), (x1, h0), Palette::dim(), RUN_W));
            strokes.push(segment(stage, (x1, h0), (x1, h1), palette::spin(), RISE_W));
            let mid = at(x1, (h0 + h1) / 2.0);
            labels.push(v(mid[0] + STEP_LABEL_DX, mid[1]));
        }
        let mut steps = Vec::with_capacity(labels.len());
        steps.push(tag);
        for &at_label in &labels[1..] {
            steps.push(kit.text(stage, &climb(LINE_SLOPE), STEP_LABEL_SIZE, palette::spin(), at_label)?);
        }
        let flight = moving(stage, tag, labels[0], 0.8)?;
        stage.play_prepared(vec![flight])?;
        create_lagged(stage, &strokes, 1.6, 0.4)?;
        fade_in_all(stage, &steps[1..], 0.4)?;
        spoken.wait_end(stage)?;

        // 07_3: each trustee's share is the line's height at its position
        let spoken = say(&kit, stage, "07_3")?;
        let mut dealer = spare.parts().to_vec();
        dealer.push(title);
        let anims = leaving(stage, &dealer, ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        for (phrase, ts) in [
            ("Trustee one sits at position one", &TRUSTEES[..1]),
            ("trustee two at position two", &TRUSTEES[1..2]),
            ("and so on", &TRUSTEES[2..]),
        ] {
            spoken.wait_for(stage, phrase)?;
            let mut halos = marker_rings(stage, ts, palette::share());
            halos.extend(frame_halos(stage, ts, palette::share())?);
            glow(stage, &halos, 0.7)?;
        }
        spoken.wait_for(stage, "Each trustee's share is the line's height")?;
        let (mut guides, mut points) = (Vec::new(), Vec::new());
        for (&t, &h) in TRUSTEES.iter().zip(&heights) {
            guides.push(segment(
                stage,
                (f64::from(t), 0.0),
                (f64::from(t), h as f64),
                Palette::dim(),
                GUIDE_W,
            ));
            points.push(point(stage, f64::from(t), h as f64, palette::share()));
        }
        create_lagged(stage, &guides, 1.0, 0.2)?;
        fade_in_all(stage, &points, 0.3)?;
        spoken.wait_for(stage, "Nine, thirteen, seventeen, twenty-one and twenty-five")?;
        // a connected list: each height by its point as it is said, and
        // dropping into its trustee's card
        let rt = (spoken.start + spoken.len - now(stage)).clamp(1.5, 3.0);
        let (mut heights_shown, mut cards) = (Vec::new(), Vec::new());
        let mut pairs = Vec::with_capacity(TRUSTEES.len());
        for (&t, &h) in TRUSTEES.iter().zip(&heights) {
            let label = point_label(&kit, stage, f64::from(t), h, palette::share())?;
            let c = card(&kit, stage, t, &h.to_string(), palette::share())?;
            let a = fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?;
            let label_in = stage.prepare(a)?;
            let a = fade_in(stage.arena_mut(), c, v(0.0, -DROP), 1.0)?;
            let card_in = stage.prepare(a)?;
            let both = AnimationGroup::new(stage.arena_mut(), vec![label_in, card_in])?;
            pairs.push(stage.prepare(both)?);
            heights_shown.push(label);
            cards.push(c);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), pairs, 0.6)?.with_run_time(rt);
        stage.play(g)?;
        spoken.wait_end(stage)?;

        // 07_4: any two trustees redraw the line, and it meets 0 at 5
        let spoken = say(&kit, stage, "07_4")?;
        let pair = [TRUSTEES[1], TRUSTEES[2]];
        let keep = |t: u32| pair.contains(&t);
        let mut going = vec![drawn, secret_point];
        going.extend(strokes);
        going.extend(steps);
        going.extend(guides);
        for (i, &t) in TRUSTEES.iter().enumerate() {
            if !keep(t) {
                going.extend([points[i], heights_shown[i], cards[i]]);
            }
        }
        let mut anims = leaving(stage, &going, ORIGIN, 0.5)?;
        let mut halos = marker_rings(stage, &pair, palette::share());
        halos.extend(frame_halos(stage, &pair, palette::share())?);
        anims.extend(glowing(stage, &halos, 0.8)?);
        stage.play_prepared(anims)?;
        let index = |t: u32| (t - TRUSTEES[0]) as usize;
        spoken.wait_for(stage, "lay down their two points")?;
        let mut anims = Vec::with_capacity(pair.len());
        for &t in &pair {
            anims.push(swelling(stage, points[index(t)], 1.6, palette::share(), 0.6)?);
        }
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "redraw the line")?;
        // the climb a step from their two points alone, and where that line
        // meets position 0
        let [(px, ph), (qx, qh)] = pair.map(|t| (i64::from(t), line.height(t)));
        let (rise, run) = (qh - ph, qx - px);
        let back = at_zero(qx, qh, rise / run);
        if rise % run != 0 || back != i64::from(SECRET) {
            return Err(fail(format!(
                "trustees {pair:?} climb {rise} in {run} and meet 0 at {back}, not the secret"
            )));
        }
        // from the second point back through the first to position 0
        let redrawn = segment(
            stage,
            (qx as f64, qh as f64),
            (0.0, back as f64),
            Palette::ink(),
            LINE_W,
        );
        let a = stage.prepare(show_creation(redrawn).rt(1.2))?;
        stage.play_prepared(vec![a])?;
        spoken.wait_for(stage, "the height at zero")?;
        let found = point(stage, 0.0, back as f64, palette::secret());
        fade_in_all(stage, &[found], 0.3)?;
        let mut anims = vec![swelling(stage, found, 1.6, palette::secret(), 0.7)?];
        let halo = text_halo(stage, five, palette::secret())?;
        anims.extend(glowing(stage, &[halo], 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;

        // 07_5: one point: a line pivots through it; the strip for anyone
        // outside; which secrets fit trustee 5's 25?
        let spoken = say(&kit, stage, "07_5")?;
        let mut going = vec![redrawn, found];
        for &t in &pair {
            let i = index(t);
            going.extend([points[i], heights_shown[i], cards[i]]);
        }
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let (hx, hh) = (f64::from(TEN_HOLDER), held as f64);
        let alone = point(stage, hx, hh, palette::share());
        let alone_label = point_label(&kit, stage, hx, held, palette::share())?;
        fade_in_all(stage, &[alone, alone_label], 0.4)?;
        spoken.wait_for(stage, "not enough to redraw it")?;
        let slope = stage.add_value_tracker(f64::from(LINE_SLOPE));
        let pivot = stage.always_redraw(move |s| {
            let k = s.tracker_value(slope).unwrap_or(0.0);
            let [a, b] = through(hx, hh, k).unwrap_or([(hx, hh), (hx, hh)]);
            let l = polyline(vec![at(a.0, a.1), at(b.0, b.1)])
                .style(line_style(Palette::dim(), LINE_W))
                .build();
            s.add(l)
        });
        stage.add_to_scene(pivot)?;
        stage.play(tween(slope, PIVOT[0], 0.9)?)?;
        stage.play(tween(slope, PIVOT[1], 1.4)?)?;
        let anims = leaving(stage, &[pivot], ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "on ordinary paper")?;
        let mut strip = Strip::build(&kit, stage, dial)?;
        strip.reveal(&kit, stage, 0.6)?;
        strip.show(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.3)?;
        spoken.wait_for(stage, "Trustee five holds twenty-five")?;
        let held_card = card(&kit, stage, TEN_HOLDER, &held.to_string(), palette::share())?;
        let a = fade_in(stage.arena_mut(), held_card, v(0.0, -DROP), 1.0)?;
        let mut anims = vec![stage.prepare(a.rt(0.5))?];
        let mut halos = marker_rings(stage, &[TEN_HOLDER], palette::share());
        halos.extend(frame_halos(stage, &[TEN_HOLDER], palette::share())?);
        anims.extend(glowing(stage, &halos, 0.8)?);
        stage.play_prepared(anims)?;
        spoken.wait_end(stage)?;
        // the count stays withheld: the strip still shows anyone outside's
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 07_6: the fan of whole spins: three meet 0 at a secret 0 to 10
        let spoken = say(&kit, stage, "07_6")?;
        let mut lines = Vec::with_capacity(fanned.len());
        for &(ends, ..) in &fanned {
            // drawn from trustee 5's point back towards position 0
            let [left, right] = ends;
            lines.push(segment(stage, right, left, Palette::dim(), FAN_W));
        }
        let dial_band = band(stage, 0, i64::from(dial.marks() - 1), palette::secret())?;
        let anims = leaving(stage, &[question], ORIGIN, 0.3)?;
        stage.play_prepared(anims)?;
        fade_in_all(stage, &[dial_band], 0.3)?;
        create_lagged(stage, &lines, 1.5, 0.15)?;
        spoken.wait_for(stage, "zero, five or ten")?;
        let (mut hits, mut misses) = (Vec::new(), Vec::new());
        let mut met_points = Vec::new();
        for (&m, &(_, zero, fits_here)) in lines.iter().zip(&fanned) {
            if fits_here {
                hits.push((m, palette::secret()));
                met_points.push(point(stage, 0.0, zero as f64, palette::secret()));
            } else {
                misses.push((m, palette::ghost()));
            }
        }
        let mut with = recolorings(stage, &hits, 0.6)?;
        with.extend(swaps(stage, &[], &met_points, 0.6)?);
        strip.show_with(&kit, stage, &view, &for_paper(TEN_HOLDER), 0.6, with)?;
        check_strip(&strip, &fits, "for trustee 5 on paper")?;
        spoken.wait_for(stage, "No other secret")?;
        let text = paper_card(TEN_HOLDER, held, dial.marks(), &fits);
        let exact = precisely(&kit, stage, &text)?;
        let mut anims = recolorings(stage, &misses, 0.6)?;
        anims.extend(swaps(stage, &[], &[exact], 0.5)?);
        stage.play_prepared(anims)?;
        let read = read_by(stage, &text);
        spoken.wait_end(stage)?;
        wait_until(stage, read)?;

        // 07_7: ordinary heights leak, as B's piece did; wrap them instead
        let spoken = say(&kit, stage, "07_7")?;
        let anims = leaving(stage, &[exact], ORIGIN, 0.4)?;
        stage.play_prepared(anims)?;
        spoken.wait_for(stage, "Ordinary numbers keep growing")?;
        let tall = band(stage, i64::from(dial.marks() - 1), TALLEST, palette::leak())?;
        glow(stage, &[tall], 1.0)?;
        spoken.wait_for(stage, "their size leaks")?;
        let mut halos = vec![text_halo(stage, alone_label, palette::leak())?];
        halos.extend(frame_halos(stage, &[TEN_HOLDER], palette::leak())?);
        glow(stage, &halos, 0.8)?;
        spoken.wait_for(stage, "just as B's piece did")?;
        strip.show(&kit, stage, &ordinary_view(b), &for_b(b), 0.6)?;
        check_strip(&strip, &ordinary_secrets(dial, b), "for B's piece")?;
        spoken.wait_for(stage, "So wrap the heights")?;
        let wrap = Face::build(&kit, stage, dial, v(WRAP_X, WRAP_Y), WRAP_R, Palette::dim(), 0)?;
        wrap.show(stage, 0.5)?;
        let halo = band(stage, 0, i64::from(dial.marks() - 1), palette::secret())?;
        glow(stage, &[halo], 1.0)?;
        spoken.wait_end(stage)?;
        pause(stage, 0.5)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{STRIP_CAPTION_Y, STRIP_LEFT, STRIP_TITLE_Y, THINK_R, strip_right};
    use super::super::plot::{PLOT_X0, PLOT_Y0, POINT_LABEL_SIZE, axis_end, label_at};
    use super::*;
    use crate::facts;
    use crate::model::{MARKS, csv, paper_line_secrets};

    /// The production narration: registering chapter 07 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Every phrase the scene waits for is in its line of the script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("07_1", "start with a simpler rule"),
            ("07_1", "Mark the secret"),
            ("07_1", "position zero"),
            ("07_1", "height five"),
            ("07_2", "spins once"),
            ("07_2", "say it gives four"),
            ("07_2", "Draw the straight line"),
            ("07_2", "climbs four for every step"),
            ("07_3", "Trustee one sits at position one"),
            ("07_3", "trustee two at position two"),
            ("07_3", "and so on"),
            ("07_3", "Each trustee's share is the line's height"),
            ("07_3", "Nine, thirteen, seventeen, twenty-one and twenty-five"),
            ("07_4", "lay down their two points"),
            ("07_4", "redraw the line"),
            ("07_4", "the height at zero"),
            ("07_5", "not enough to redraw it"),
            ("07_5", "on ordinary paper"),
            ("07_5", "Trustee five holds twenty-five"),
            ("07_6", "zero, five or ten"),
            ("07_6", "No other secret"),
            ("07_7", "Ordinary numbers keep growing"),
            ("07_7", "their size leaks"),
            ("07_7", "just as B's piece did"),
            ("07_7", "So wrap the heights"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the scene draws is the facts': the rule's 2 of 5, the slope 4,
    /// the heights 9 to 25, the secret at position 0, trustee 5's 25 and
    /// the three secrets it leaves on paper, out of the dial's 11, and B's
    /// piece 3 leaving 8.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        assert_eq!(
            f.get("S-F4.needed"),
            Some(line.coefficients().len().to_string().as_str())
        );
        assert_eq!(
            rule(line.coefficients().len(), TRUSTEES.len()),
            "a simpler rule: any 2 of 5"
        );
        assert_eq!(f.get("S-F4.slope"), Some(LINE_SLOPE.to_string().as_str()));
        assert_eq!(climb(LINE_SLOPE), "+4");
        let heights: Vec<i64> = TRUSTEES.iter().map(|&t| line.height(t)).collect();
        assert_eq!(f.get("S-F4.heights"), Some(csv(&heights).as_str()));
        // every step climbs the spin
        for x in 0..LAST {
            assert_eq!(line.height(x + 1) - line.height(x), i64::from(LINE_SLOPE));
        }
        assert_eq!(f.get("S-F4.secret_position"), Some("0"));
        assert_eq!(line.height(0), i64::from(SECRET));
        assert_eq!(f.get("S-F4.paper.holder"), Some(TEN_HOLDER.to_string().as_str()));
        let held = line.height(TEN_HOLDER);
        assert_eq!(f.get("S-F4.paper.share"), Some(held.to_string().as_str()));
        let dial = Dial::vault();
        let view = View::Paper {
            points: vec![(TEN_HOLDER, held)],
            spins: 1,
        };
        let fits = view.possible(dial);
        assert_eq!(fits, paper_line_secrets(MARKS, TEN_HOLDER, held));
        assert_eq!(f.get("S-F4.paper.candidates"), Some(csv(&fits).as_str()));
        assert_eq!(f.get("S-F4.paper.count"), Some(fits.len().to_string().as_str()));
        assert_eq!(View::Outsider.possible(dial).len(), 11);
        assert_eq!(for_paper(TEN_HOLDER), "for: trustee 5, on paper");
        let b = ORDINARY_PIECES[1];
        assert_eq!(b, 3);
        assert_eq!(ordinary_view(b).possible(dial), ordinary_secrets(dial, b));
        assert_eq!(ordinary_secrets(dial, b).len(), 8);
        assert_eq!(for_b(b), "for: B, holding 3 (chapter 02)");
        assert_eq!(
            paper_card(TEN_HOLDER, held, MARKS, &fits),
            "secret = 25 \u{2212} 5 \u{d7} spin; spins 0 to 10 give 25, 20, 15, 10, 5, 0, \u{2212}5, \u{2212}10, \
             \u{2212}15, \u{2212}20, \u{2212}25; only 0, 5 and 10 are secrets 0 to 10"
        );
    }

    /// The fan: one line per whole spin 0 to 10, all through (5, 25);
    /// exactly spins 3, 4 and 5 meet position 0 on the dial (at 10, 5 and
    /// 0); spins 0 to 2 meet it above 10 and spins 6 to 10 stop on the
    /// bottom axis, below 0 at position 0. A planted negative: with the
    /// dial's secrets taken as 0 to 25 the fan would fit six.
    #[test]
    fn the_fan_meets_three_secrets() {
        let fits = vec![0, 5, 10];
        let fanned = fan(TEN_HOLDER, 25, MARKS, &fits);
        assert_eq!(fanned.len(), 11);
        let hits: Vec<(usize, i64)> = fanned
            .iter()
            .enumerate()
            .filter(|(_, f)| f.2)
            .map(|(a, f)| (a, f.1))
            .collect();
        assert_eq!(hits, vec![(3, 10), (4, 5), (5, 0)]);
        for (a, &([(x0, h0), (x1, h1)], zero, _)) in fanned.iter().enumerate() {
            assert!((x1 - 5.0).abs() < 1e-12 && (h1 - 25.0).abs() < 1e-12);
            assert_eq!(zero, 25 - 5 * a as i64);
            if a <= 5 {
                assert!(x0.abs() < 1e-12 && (h0 - zero as f64).abs() < 1e-12, "{a}");
            } else {
                assert!(x0 > 0.0 && h0.abs() < 1e-9, "{a}: stops on the bottom axis");
            }
        }
        let wide: Vec<u32> = (0..=25).collect();
        let six = fan(TEN_HOLDER, 25, MARKS, &wide).iter().filter(|f| f.2).count();
        assert_eq!(six, 6);
    }

    /// Two trustees' line, rebuilt from their points alone, meets position
    /// 0 at the secret, for every pair of the five.
    #[test]
    fn any_two_points_give_back_the_secret() {
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        for (i, &p) in TRUSTEES.iter().enumerate() {
            for &q in &TRUSTEES[i + 1..] {
                let (hp, hq) = (line.height(p), line.height(q));
                let rise = hq - hp;
                let run = i64::from(q) - i64::from(p);
                assert_eq!(rise % run, 0);
                assert_eq!(at_zero(i64::from(q), hq, rise / run), i64::from(SECRET), "{p} {q}");
            }
        }
    }

    /// The precisely card stays two lines at every wrap the lower third
    /// can narrow to.
    #[test]
    fn the_card_is_two_lines() {
        let text = paper_card(TEN_HOLDER, 25, MARKS, &[0, 5, 10]);
        for width in (72..=92).rev().step_by(4) {
            let lines = explainer_kit::animatic::wrap(&text, width, 3);
            assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
        }
    }

    /// The rule card, the spare dial and its title, the step labels, the
    /// wrap dial and the think ring keep apart, inside the frame, clear of
    /// the plot and of the strip; the captions end inside the right edge.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const {
            assert!(
                SPIN_Y + SPIN_TITLE_DY + 0.1 < RULE_Y - 0.15,
                "the spare dial's title under the rule"
            );
            assert!(WRAP_Y + WRAP_R < RULE_Y - 0.3, "the wrap dial under the rule");
            assert!(WRAP_Y - WRAP_R > PLOT_Y0 + 0.3, "the wrap dial over the position title");
            assert!(WRAP_X + WRAP_R < STRIP_LEFT - 0.3, "the wrap dial left of the strip");
            assert!(
                THINK_Y + THINK_R < STRIP_CAPTION_Y - 0.2,
                "the think ring under the caption"
            );
            assert!(STRIP_TITLE_Y < RULE_Y - 0.4, "the strip's title under the rule");
        }
        let rule_w = text_w(&rule(2, 5), RULE_SIZE);
        assert!(RULE_X - rule_w / 2.0 > axis_end() + 0.3 && RULE_X + rule_w / 2.0 < FRAME_X_RADIUS - 0.25);
        assert!(RULE_Y + text_h(RULE_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        // the spare dial and its tag right of the plot's position title
        let plot_right = axis_end() + 0.11 + text_w("position", 18.0);
        assert!(SPIN_X - SPARE_R * ARC_LABEL_IN - 0.2 > plot_right, "{plot_right}");
        let title_w = text_w(SPARE_TITLE, SPIN_TITLE_SIZE);
        assert!(SPIN_X - title_w / 2.0 > plot_right);
        // the step labels (07_2, with the spare dial) left of its tag, the
        // highest under the tallest point's label; the wrap dial (07_7,
        // with the fan) right of the plot's last column and of the tallest
        // point's label
        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
        let last_x = at(f64::from(LAST), 0.0)[0];
        let label_right = last_x + STEP_LABEL_DX + text_w(&climb(LINE_SLOPE), STEP_LABEL_SIZE);
        assert!(label_right < SPIN_X - SPARE_R * ARC_LABEL_IN - 0.3, "{label_right}");
        let highest = at(f64::from(LAST), line.height(LAST) as f64 - f64::from(LINE_SLOPE) / 2.0)[1];
        let top_label = label_at(f64::from(LAST), line.height(LAST) as f64);
        assert!(highest + text_h(STEP_LABEL_SIZE) / 2.0 < top_label[1] - text_h(POINT_LABEL_SIZE) / 2.0);
        assert!(top_label[1] + text_h(POINT_LABEL_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        assert!(WRAP_X - WRAP_R > last_x + 0.3);
        assert!(WRAP_X - WRAP_R > top_label[0] + text_w("25", POINT_LABEL_SIZE) / 2.0 + 0.3);
        const { assert!(PLOT_X0 < WRAP_X) };
        // the captions end inside the right edge and left of the think ring
        for caption in [FOR_OUTSIDE.to_owned(), for_paper(TEN_HOLDER), for_b(3)] {
            let right = STRIP_LEFT + text_w(&caption, 22.0);
            assert!(right < FRAME_X_RADIUS - 0.25, "{caption}");
            assert!(right < THINK_X - THINK_R - 0.2, "{caption}");
        }
        assert!(THINK_X + THINK_R < strip_right(MARKS) + 0.3);
    }
}
