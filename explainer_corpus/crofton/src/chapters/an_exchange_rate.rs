//! Chapter 03, an exchange rate. Seven crossings, but a count is not a
//! length: "7 crossings → ? cm" (03_1). Crossings ought to know something
//! about length. The beach redrawn as one big half circle is about 2 cm
//! longer and crosses the lower line twice more (03_2). As seven small half
//! circles it is just as much longer and crosses no line at this placement,
//! so one placement cannot follow length; the hope is about many placements,
//! on average (03_3). Is each crossing worth the gap, 2 cm? (03_4, a
//! prediction.) It cannot be: 7 × 2 cm = 14 cm, a bar laid from the coast's
//! start toward its end stops short of the 14.7 cm between the ends, and no
//! curve joining them is shorter than that (03_5). The rate must come from a
//! curve of known length (03_6). The coast's true length is known exactly,
//! four straight runs and seven arcs, and goes in a sealed envelope; then
//! the title over the dimmed card (03_7). Starts where chapter 02 ends.
//! Integrated by root from the preserved candidate; qualification evidence is external.

use explainer_kit::{Kit, Palette, tween, v};
use fmn::prelude::*;

use super::draw::{
    EVERY, card_panel, card_point, dashed, dim_line, fade_in_all, fade_out_all, fade_to, fail, flat, line_style, now,
    path, say, stand, stop, swap, think, together, traced,
};
use super::glyphs::TITLE;
use super::sheet::{
    self, COUNT_AT, COUNT_SIZE, Crossing, LABEL_SIZE, LOW_Y, Sheet, THINK_AT, count_words, crossing_dot, gap_bracket,
    gap_label_at,
};
use crate::facts;
use crate::model::{self, BEACH, Curve, GAP, P2, Part, Seg};
use crate::palette;

pub struct AnExchangeRate {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const ARROW: &str = r"\rightarrow\ ?\text{ cm}";
const ASK: &str = "?";
const HOPE: &str = "hope: on average, more length means more crossings";
const SEALED: &str = "true length";
/// 03_2 and 03_3 redraw the beach as this many half circles, in turn.
const GHOSTS: [usize; 2] = [1, 7];

// ------------------------------------------------------------------ layout

const COAST_W: f64 = 4.0;
/// A piece picked out (03_5, 03_7): wider, its colour unchanged.
const PICKED_W: f64 = 8.0;
/// 03_2 and 03_3's words, in the band under the coast, under the beach:
/// the extra length and the count, and under them the exact extra.
const GHOST_X: f64 = -3.6;
const GHOST_Y: f64 = LOW_Y + 0.15;
const PRECISELY_Y: f64 = LOW_Y - 0.25;
const GHOST_SIZE: f64 = 26.0;
const NOTE_SIZE: f64 = 20.0;
const EQUATION_SIZE: f64 = 30.0;
/// 03_5's sum and the ends' distance, side by side in the band under the
/// coast.
const SUM_AT: Vec3 = [-1.2, LOW_Y, 0.0];
const APART_AT: Vec3 = [3.6, LOW_Y, 0.0];
const BAR_W: f64 = 6.0;
/// The envelope, in the band over the coast at the card's left, frame
/// units, with its label under it.
const ENVELOPE_AT: Vec3 = [-5.6, 3.05, 0.0];
const ENVELOPE: [f64; 2] = [1.0, 0.62];
const SEALED_Y: f64 = 2.5;
const TITLE_AT: Vec3 = [0.0, 0.25, 0.0];
const TITLE_SIZE: f64 = 50.0;
/// What the title dims the card and everything on it to.
const DIMMED: f64 = 0.25;

// ------------------------------------------------------------------ timing

const GHOST_DRAW_S: f64 = 1.0;
const BAR_S: f64 = 1.0;
const WIDEN_S: f64 = 0.35;
const TITLE_IN: f64 = 1.0;

/// The chapter's words that carry numbers, each from the model or the
/// facts. Refused unless the sheet's gap is the facts', the two redrawn
/// beaches add the same length, the ends' distance is the facts', and the
/// guess falls short of it.
struct Words {
    count: String,
    longer: String,
    ghost_counts: [String; 2],
    precisely: String,
    gap: String,
    guess: String,
    sum: String,
    guess_cm: f64,
    apart: String,
}

fn words(coast: &Curve, found: usize, ghosts: [&Curve; 2], ghost_counts: [usize; 2]) -> Result<Words, String> {
    let f = facts::facts()?;
    let get = |key: &str| f.get(key).map(str::to_owned).ok_or_else(|| format!("no fact {key}"));
    let gap = get("sheet.gap")?;
    if gap != facts::fixed(GAP, 0) {
        return Err(format!("the sheet is drawn {GAP} cm apart, the facts say {gap}"));
    }
    let extra = ghosts.map(|g| g.length - coast.length);
    if (extra[0] - extra[1]).abs() > 1e-9 {
        return Err(format!("03_3 says just as much longer: {} and {}", extra[0], extra[1]));
    }
    let (start, end) = coast.ends();
    let ends = (start[0] - end[0]).hypot(start[1] - end[1]);
    let apart = get("coast.ends_apart")?;
    if facts::fixed(ends, 1) != apart {
        return Err(format!("the ends are {ends} cm apart, the facts say {apart}"));
    }
    let guess_cm = found as f64 * GAP;
    if guess_cm >= ends {
        return Err(format!("03_5 needs {guess_cm} cm to fall short of {ends}"));
    }
    Ok(Words {
        count: count_words(found),
        longer: format!(r"\approx {}\text{{ cm longer}}", facts::fixed(extra[0], 0)),
        ghost_counts: ghost_counts.map(count_words),
        precisely: format!("both about {} cm longer", facts::fixed(extra[0], 3)),
        gap: format!("{gap} cm"),
        guess: format!(r"1\text{{ crossing}} = {gap}\text{{ cm}}\ ?"),
        sum: format!(
            r"{found} \times {gap}\text{{ cm}} = {}\text{{ cm}}",
            facts::fixed(guess_cm, 0)
        ),
        guess_cm,
        apart: format!("{apart} cm end to end"),
    })
}

/// What 03_2 and 03_3 say about the redrawn beaches, at the shown
/// placement: each drawing's count, and where the big half circle crosses.
/// Refused unless it crosses the lower line twice and only that, and the
/// small half circles cross no line.
struct Ghosts {
    counts: [usize; 2],
    new_points: Vec<P2>,
}

fn ghost_facts(coast: &Curve, one: &Curve, seven: &Curve, slide: f64) -> Result<Ghosts, String> {
    let base = sheet::crossings(coast, 0.0, slide);
    let lower = base.iter().map(|c| c.line).min().ok_or("the coast crosses no line")?;
    let on_beach = |c: &&Crossing, count: usize| (BEACH..BEACH + count).contains(&c.piece);
    let big = sheet::crossings(one, 0.0, slide);
    let new: Vec<&Crossing> = big.iter().filter(|c| on_beach(c, GHOSTS[0])).collect();
    if new.len() != 2 || new.iter().any(|c| c.line != lower) || big.len() != base.len() + 2 {
        return Err(format!(
            "03_2 says the big half circle crosses the lower line twice more: {} new, {} in all against {}",
            new.len(),
            big.len(),
            base.len()
        ));
    }
    let small = sheet::crossings(seven, 0.0, slide);
    if small.iter().any(|c| on_beach(&c, GHOSTS[1])) || small.len() != base.len() {
        return Err(format!(
            "03_3 says the small half circles cross no line: {} in all",
            small.len()
        ));
    }
    Ok(Ghosts {
        counts: [big.len(), small.len()],
        new_points: new.iter().map(|c| c.at).collect(),
    })
}

/// The half circles that replace the beach in `ghost` (drawn with `count`
/// of them), as a curve of their own from the beach's start.
fn ghost_beach(ghost: &Curve, count: usize) -> Curve {
    let first = ghost.parts[BEACH].start;
    let parts: Vec<Part> = ghost.parts[BEACH..BEACH + count]
        .iter()
        .map(|p| Part {
            seg: p.seg,
            start: p.start - first,
        })
        .collect();
    let length = parts.iter().map(|p| p.seg.len()).sum();
    Curve { parts, length }
}

/// The guess laid from the coast's start toward its end, `length` cm: a
/// straight run along the line between the ends.
fn guess_bar(start: P2, end: P2, length: f64) -> Curve {
    let d = (end[0] - start[0]).hypot(end[1] - start[1]);
    let to = [
        start[0] + length * (end[0] - start[0]) / d,
        start[1] + length * (end[1] - start[1]) / d,
    ];
    Curve {
        parts: vec![Part {
            seg: Seg::Line { from: start, to },
            start: 0.0,
        }],
        length,
    }
}

/// Stroked pieces widened (or narrowed) to `width`, their colour and fill
/// unchanged.
fn widen(mobs: &[Mob], width: f64) -> fmn::Result<Vec<AnimBuilder>> {
    mobs.iter()
        .map(|&m| -> fmn::Result<AnimBuilder> {
            Ok(m.animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(WIDEN_S),
                    ..AnimateArgs::default()
                })?
                .set_stroke(None, Some(width), None)?)
        })
        .collect()
}

/// The sealed envelope: a green body, its flap, "true length" under it.
/// Detached.
fn envelope(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    let ([w, h], [x, y, _]) = (ENVELOPE, ENVELOPE_AT);
    let body = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.04)
        .style(flat(palette::length(), 0.35))
        .build()?
        .moved_to(ENVELOPE_AT);
    let body = stage.arena_mut().add(body);
    let flap = polyline(vec![
        v(x - w / 2.0, y + h / 2.0),
        v(x, y - 0.05),
        v(x + w / 2.0, y + h / 2.0),
    ])
    .style(line_style(palette::length(), 2.5))
    .build();
    let flap = stage.arena_mut().add(flap);
    let label = kit.text(stage, SEALED, NOTE_SIZE, palette::length(), v(x, SEALED_Y))?;
    Ok(vec![body, flap, label])
}

impl SceneConstruct for AnExchangeRate {
    fn name(&self) -> &str {
        "03_an_exchange_rate"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        let found = sheet::crossings(&coast, 0.0, slide);
        let one = sheet::rippled(&coast, GHOSTS[0]).map_err(fail)?;
        let seven = sheet::rippled(&coast, GHOSTS[1]).map_err(fail)?;
        let ghosts = ghost_facts(&coast, &one, &seven, slide).map_err(fail)?;
        let words = words(&coast, found.len(), [&one, &seven], ghosts.counts).map_err(fail)?;

        // where chapter 02 ended: the card, the coast (piece by piece, so
        // pieces can be picked out), the sheet, the seven dots, the count
        let card = card_panel(stage)?;
        let pieces: Vec<Mob> = coast
            .parts
            .iter()
            .map(|p| {
                path(
                    stage,
                    &coast.between(p.start, p.end(), EVERY),
                    palette::coast(),
                    COAST_W,
                )
            })
            .collect();
        let ruled = Sheet::new(stage, slide, 0.0)?;
        let dots: Vec<Mob> = found.iter().map(|c| crossing_dot(stage, c.at)).collect();
        let count = kit.text(stage, &words.count, COUNT_SIZE, palette::touch(), COUNT_AT)?;
        let mut opening = vec![card];
        opening.extend(&pieces);
        opening.extend(ruled.all());
        opening.extend(&dots);
        opening.push(count);
        for m in opening {
            stage.add_to_scene(m)?;
        }

        // 03_1: a count, not a length
        let line = say(&kit, stage, "03_1")?;
        line.wait_for(stage, "a count, not a length")?;
        let arrow = kit.tex(
            stage,
            ARROW,
            COUNT_SIZE,
            Palette::ink(),
            &[(ASK, palette::touch())],
            COUNT_AT,
        )?;
        stage.next_to(arrow, count, RIGHT, 0.25, ORIGIN);
        fade_in_all(stage, &[arrow], 0.4)?;

        // 03_2: the beach as one big half circle, about 2 cm longer, two
        // more crossings on the lower line
        let line = say(&kit, stage, "03_2")?;
        // The original coast's count must not label either replacement.
        fade_out_all(stage, &[count, arrow], 0.4)?;
        line.wait_for(stage, "Swap the straight beach")?;
        together(stage, vec![dim_line(pieces[BEACH], DIMMED, 0.4)?])?;
        let drawn = stage.add_value_tracker(0.0);
        let [pen, big] = traced(stage, &ghost_beach(&one, GHOSTS[0]), drawn, palette::coast(), COAST_W)?;
        fade_in_all(stage, &[pen, big], 0.1)?;
        stage.play(tween(drawn, 1.0, GHOST_DRAW_S)?)?;
        stop(stage, pen);
        line.wait_for(stage, "about two centimetres longer")?;
        let longer = kit.tex(
            stage,
            &words.longer,
            GHOST_SIZE,
            Palette::ink(),
            &[],
            v(GHOST_X, GHOST_Y),
        )?;
        fade_in_all(stage, &[longer], 0.4)?;
        line.wait_for(stage, "twice more")?;
        let mut more: Vec<Mob> = ghosts.new_points.iter().map(|&p| crossing_dot(stage, p)).collect();
        let nine = kit.text(
            stage,
            &words.ghost_counts[0],
            GHOST_SIZE,
            palette::touch(),
            v(GHOST_X, GHOST_Y),
        )?;
        stage.next_to(nine, longer, RIGHT, 0.3, ORIGIN);
        more.push(nine);
        fade_in_all(stage, &more, 0.4)?;
        // Let viewers find the two added crossings and read the new total.
        let counted = now(stage);
        stand(&kit, stage, counted)?;

        // 03_3: seven small half circles, just as much longer, no crossing;
        // one placement cannot follow length, and the hope
        let line = say(&kit, stage, "03_3")?;
        line.wait_for(stage, "seven small half circles")?;
        let mut gone = vec![big];
        gone.extend(&more);
        fade_out_all(stage, &gone, 0.4)?;
        let drawn = stage.add_value_tracker(0.0);
        let [pen, small] = traced(stage, &ghost_beach(&seven, GHOSTS[1]), drawn, palette::coast(), COAST_W)?;
        fade_in_all(stage, &[pen, small], 0.1)?;
        stage.play(tween(drawn, 1.0, GHOST_DRAW_S)?)?;
        stop(stage, pen);
        line.wait_for(stage, "just as much length")?;
        let precisely = kit.text(
            stage,
            &words.precisely,
            NOTE_SIZE,
            Palette::dim(),
            v(GHOST_X, PRECISELY_Y),
        )?;
        fade_in_all(stage, &[precisely], 0.4)?;
        line.wait_for(stage, "no line at all")?;
        let seven_count = kit.text(
            stage,
            &words.ghost_counts[1],
            GHOST_SIZE,
            palette::touch(),
            v(GHOST_X, GHOST_Y),
        )?;
        stage.next_to(seven_count, longer, RIGHT, 0.3, ORIGIN);
        fade_in_all(stage, &[seven_count], 0.4)?;
        line.wait_for(stage, "The hope is")?;
        fade_out_all(stage, &[small, longer, seven_count, precisely], 0.5)?;
        together(stage, vec![dim_line(pieces[BEACH], 1.0, 0.5)?])?;
        fade_in_all(stage, &[count, arrow], 0.4)?;
        let hope = kit.text(stage, HOPE, GHOST_SIZE, palette::touch(), v(0.0, LOW_Y))?;
        fade_in_all(stage, &[hope], 0.5)?;

        // 03_4: one gap a crossing? The bracket again, and time to think
        let line = say(&kit, stage, "03_4")?;
        fade_out_all(stage, &[hope], 0.4)?;
        line.wait_for(stage, "two centimetres apart")?;
        let tick = path(stage, &gap_bracket(&ruled.heights), Palette::ink(), 2.0);
        let gap = kit.text(
            stage,
            &words.gap,
            LABEL_SIZE,
            Palette::dim(),
            card_point(gap_label_at(&ruled.heights)),
        )?;
        fade_in_all(stage, &[tick, gap], 0.4)?;
        line.wait_for(stage, "Is each crossing worth")?;
        let guess = kit.tex(stage, &words.guess, EQUATION_SIZE, palette::touch(), &[], v(0.0, LOW_Y))?;
        fade_in_all(stage, &[guess], 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 03_5: it cannot be; the guess laid along the ends' line from the
        // start stops short of the end, and the coast joins them
        let line = say(&kit, stage, "03_5")?;
        fade_out_all(stage, &[q, tick, gap], 0.3)?;
        line.wait_for(stage, "Seven times two")?;
        let sum = kit.tex(stage, &words.sum, EQUATION_SIZE, palette::dividers(), &[], SUM_AT)?;
        swap(stage, &[guess], &[sum], 0.4)?;
        line.wait_for(stage, "fourteen centimetres")?;
        let (start, end) = coast.ends();
        let grown = stage.add_value_tracker(0.0);
        let [pen, bar] = traced(
            stage,
            &guess_bar(start, end, words.guess_cm),
            grown,
            palette::dividers(),
            BAR_W,
        )?;
        fade_in_all(stage, &[pen, bar], 0.1)?;
        stage.play(tween(grown, 1.0, BAR_S)?)?;
        stop(stage, pen);
        line.wait_for(stage, "the two ends of the coast")?;
        let between = dashed(stage, start, end, Palette::dim())?;
        let apart = kit.text(stage, &words.apart, GHOST_SIZE, Palette::dim(), APART_AT)?;
        fade_in_all(stage, &[between, apart], 0.5)?;
        stage.bring_to_front(bar)?;
        line.wait_for(stage, "No curve joining them")?;
        together(stage, widen(&pieces, PICKED_W)?)?;
        together(stage, widen(&pieces, COAST_W)?)?;

        // 03_6: the rate has to come from elsewhere
        say(&kit, stage, "03_6")?;
        fade_out_all(stage, &[sum, bar, between, apart], 0.5)?;

        // 03_7: four runs and seven arcs, a length known exactly, sealed;
        // then the title over the dimmed card, the envelope still bright
        let line = say(&kit, stage, "03_7")?;
        let of_kind = |run: bool| -> Vec<Mob> {
            pieces
                .iter()
                .zip(&coast.parts)
                .filter(|(_, p)| matches!(p.seg, Seg::Line { .. }) == run)
                .map(|(&m, _)| m)
                .collect()
        };
        let (runs, arcs) = (of_kind(true), of_kind(false));
        line.wait_for(stage, "four straight runs")?;
        together(stage, widen(&runs, PICKED_W)?)?;
        line.wait_for(stage, "seven pieces of circles")?;
        let mut moves = widen(&runs, COAST_W)?;
        moves.extend(widen(&arcs, PICKED_W)?);
        together(stage, moves)?;
        line.wait_for(stage, "I know its true length")?;
        together(stage, widen(&arcs, COAST_W)?)?;
        line.wait_for(stage, "this envelope")?;
        let sealed = envelope(&kit, stage)?;
        fade_in_all(stage, &sealed, 0.5)?;
        kit.hold(stage)?;
        let mut dim = vec![fade_to(card, DIMMED, 0.8)?];
        for &m in pieces.iter().chain(&ruled.lines) {
            dim.push(dim_line(m, DIMMED, 0.8)?);
        }
        for &m in dots.iter().chain([&count, &arrow]) {
            dim.push(fade_to(m, DIMMED, 0.8)?);
        }
        together(stage, dim)?;
        let title = kit.text(stage, TITLE, TITLE_SIZE, Palette::ink(), TITLE_AT)?;
        fade_in_all(stage, &[title], TITLE_IN)?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use crate::model::{CARD_CENTRE, CARD_H, CARD_UNIT, CARD_W};

    struct Shown {
        coast: Curve,
        slide: f64,
        found: Vec<Crossing>,
        one: Curve,
        seven: Curve,
    }

    fn shown() -> Shown {
        let coast = model::coast();
        let slide = sheet::shown_slide(&coast, 0.0);
        Shown {
            found: sheet::crossings(&coast, 0.0, slide),
            one: sheet::rippled(&coast, GHOSTS[0]).unwrap(),
            seven: sheet::rippled(&coast, GHOSTS[1]).unwrap(),
            coast,
            slide,
        }
    }

    fn shown_words(s: &Shown) -> Words {
        let ghosts = ghost_facts(&s.coast, &s.one, &s.seven, s.slide).unwrap();
        words(&s.coast, s.found.len(), [&s.one, &s.seven], ghosts.counts).unwrap()
    }

    fn dist(a: P2, b: P2) -> f64 {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }

    /// The words on screen say what the model and the facts say (the
    /// oracle's `open.crossings`, `sheet.gap`, `open.gap_guess`,
    /// `coast.ends_apart`; the redrawn beaches' counts and extra length
    /// are computed here, from the same model).
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        let w = shown_words(&shown());
        assert_eq!(w.count, "7 crossings");
        assert_eq!(w.longer, r"\approx 2\text{ cm longer}");
        assert_eq!(w.ghost_counts, ["9 crossings", "7 crossings"]);
        assert_eq!(w.precisely, "both about 1.998 cm longer");
        assert_eq!(w.gap, "2 cm");
        assert_eq!(w.guess, r"1\text{ crossing} = 2\text{ cm}\ ?");
        assert_eq!(w.sum, r"7 \times 2\text{ cm} = 14\text{ cm}");
        assert_eq!(w.guess_cm, 14.0);
        assert_eq!(w.apart, "14.7 cm end to end");
        // seven half circles, as the oracle's `scale.ripples` draws them
        assert_eq!(GHOSTS[1], 7);
    }

    /// The true length stays sealed: no word on screen gives it away, nor
    /// either redrawn coast's length (which, less 2, would).
    #[test]
    fn the_true_length_is_not_on_screen() {
        let f = facts::facts().unwrap();
        let s = shown();
        let w = shown_words(&s);
        let on_screen = [
            w.count,
            w.longer,
            w.ghost_counts[0].clone(),
            w.ghost_counts[1].clone(),
            w.precisely,
            w.gap,
            w.guess,
            w.sum,
            w.apart,
            ARROW.into(),
            HOPE.into(),
            SEALED.into(),
            TITLE.into(),
            super::super::draw::PAUSE_NOTE.into(),
        ];
        let mut sealed: Vec<String> = ["coast.length", "coast.length_2dp", "coast.length_3dp"]
            .iter()
            .map(|k| f.get(k).unwrap().to_owned())
            .collect();
        for g in [&s.one, &s.seven] {
            sealed.extend((1..=3).map(|places| facts::fixed(g.length, places)));
        }
        for text in &on_screen {
            for length in &sealed {
                assert!(!text.contains(length.as_str()), "{text} shows {length}");
            }
        }
    }

    /// 03_2 and 03_3's drawings are the computed ones: the big half circle
    /// adds two crossings, both on the lower line, the small ones none; the
    /// half circles run from the beach's start to its end and are π/2 times
    /// as long. At a placement through the small half circles the words'
    /// claims fail and are refused.
    #[test]
    fn the_redrawn_beaches_are_the_computed_illustration() {
        let s = shown();
        let ghosts = ghost_facts(&s.coast, &s.one, &s.seven, s.slide).unwrap();
        assert_eq!(ghosts.counts, [9, 7]);
        assert_eq!(ghosts.new_points.len(), 2);
        let lower = sheet::lines_on_card(s.slide)[1];
        for p in &ghosts.new_points {
            assert!((p[1] - lower).abs() < 1e-9);
        }
        let Seg::Line { from, to } = s.coast.parts[BEACH].seg else {
            panic!("the beach is a straight run")
        };
        let beach = dist(from, to);
        for (g, n) in [(&s.one, GHOSTS[0]), (&s.seven, GHOSTS[1])] {
            let b = ghost_beach(g, n);
            assert_eq!(b.parts.len(), n);
            assert!((b.length - PI / 2.0 * beach).abs() < 1e-12);
            let (a, z) = b.ends();
            assert!(dist(a, from) < 1e-12 && dist(z, to) < 1e-12, "{a:?} {z:?}");
        }
        let through = (from[1] + 0.15).rem_euclid(GAP);
        assert!(ghost_facts(&s.coast, &s.one, &s.seven, through).is_err());
    }

    /// The guess, laid from the start along the ends' line, stops 0.73 cm
    /// short of the end: visibly short, and on that line.
    #[test]
    fn the_guess_falls_short_of_the_end() {
        let s = shown();
        let w = shown_words(&s);
        let (start, end) = s.coast.ends();
        let bar = guess_bar(start, end, w.guess_cm);
        let (_, tip) = bar.ends();
        assert!((dist(start, tip) - 14.0).abs() < 1e-12);
        assert_eq!(facts::fixed(dist(tip, end), 2), "0.73");
        let side = (end[0] - start[0]) * (tip[1] - start[1]) - (end[1] - start[1]) * (tip[0] - start[0]);
        assert!(side.abs() < 1e-9);
        assert!(CARD_UNIT * dist(tip, end) > 0.5, "the gap is visible");
    }

    /// 03_7's "four straight runs and seven pieces of circles" are the
    /// coast's pieces (`coast.straight_runs`, `coast.arcs`).
    #[test]
    fn the_coast_is_four_runs_and_seven_arcs() {
        let f = facts::facts().unwrap();
        let coast = model::coast();
        let runs = coast.parts.iter().filter(|p| matches!(p.seg, Seg::Line { .. })).count();
        assert_eq!(runs.to_string(), f.get("coast.straight_runs").unwrap());
        assert_eq!((coast.parts.len() - runs).to_string(), f.get("coast.arcs").unwrap());
        assert_eq!((runs, coast.parts.len() - runs), (4, 7));
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("03_1", "a count, not a length"),
            ("03_2", "Swap the straight beach"),
            ("03_2", "about two centimetres longer"),
            ("03_2", "twice more"),
            ("03_3", "seven small half circles"),
            ("03_3", "just as much length"),
            ("03_3", "no line at all"),
            ("03_3", "The hope is"),
            ("03_4", "two centimetres apart"),
            ("03_4", "Is each crossing worth"),
            ("03_5", "Seven times two"),
            ("03_5", "fourteen centimetres"),
            ("03_5", "the two ends of the coast"),
            ("03_5", "No curve joining them"),
            ("03_7", "four straight runs"),
            ("03_7", "seven pieces of circles"),
            ("03_7", "I know its true length"),
            ("03_7", "this envelope"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
        assert!(script.text("03_6").is_some());
    }

    /// The layout keeps its rooms: the redrawn beaches' words in the band
    /// under every drawing of the coast and over the lowest line; the bar
    /// and the ends' line above that band; the envelope on the card, in the
    /// band over the coast, left of the count; the title on the card.
    #[test]
    fn the_layout_keeps_its_rooms() {
        let s = shown();
        let heights = sheet::lines_on_card(s.slide);
        let lowest_line = card_point([0.0, heights[0]])[1];
        let low = |c: &Curve| {
            c.upto(c.length, 0.02)
                .into_iter()
                .map(|p| card_point(p)[1])
                .fold(f64::INFINITY, f64::min)
        };
        for c in [&s.coast, &s.one, &s.seven] {
            assert!(GHOST_Y + 0.15 < low(c) - 0.15, "the words meet a drawing at {}", low(c));
        }
        assert!(PRECISELY_Y - 0.1 > lowest_line + 0.12 && GHOST_Y - 0.15 > PRECISELY_Y + 0.1);
        assert!(SUM_AT[1] - 0.2 > lowest_line && SUM_AT[1] + 0.2 < low(&s.coast) - 0.1);
        const {
            assert!(APART_AT[0] - SUM_AT[0] > 4.0);
        }
        let (start, end) = s.coast.ends();
        assert!(card_point(start)[1].min(card_point(end)[1]) > LOW_Y + 0.5);
        let card_top = CARD_CENTRE[1] + CARD_UNIT * CARD_H / 2.0;
        let card_left = CARD_CENTRE[0] - CARD_UNIT * CARD_W / 2.0;
        let band_lo = card_point([0.0, heights[3]])[1];
        assert!(ENVELOPE_AT[1] + ENVELOPE[1] / 2.0 < card_top - 0.1);
        assert!(SEALED_Y - 0.12 > band_lo && SEALED_Y + 0.12 < ENVELOPE_AT[1] - ENVELOPE[1] / 2.0);
        assert!(ENVELOPE_AT[0] - 0.7 > card_left && ENVELOPE_AT[0] + 0.7 < COUNT_AT[0] - 1.6);
        assert!(TITLE_AT[1].abs() + 0.5 < FRAME_Y_RADIUS && TITLE_AT[0].abs() < FRAME_X_RADIUS);
    }
}
