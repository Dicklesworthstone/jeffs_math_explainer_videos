//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x crofton facts-check`), under the oracle's
//! keys. This side prints what the native chapters use and what the model
//! already computes for the film's identity (the coast's travel, the
//! average over directions, the fixed-direction estimate, and how often
//! each count turns up over every placement); the oracle prints more, for
//! the draft's later chapters.

use std::f64::consts::PI;

use explainer_kit::Facts;

use crate::model::{self, CARD_H, CARD_W, COAST, COIN, GAP, HEADLAND, OPENING_STEP, Piece, RULER};

/// The divider steps the film walks, cm, and their keys' spelling (1.9 cm
/// is the draft's 25_6 counterexample: it walks less than 2 cm).
pub const DIVIDER_STEPS: [(f64, &str); 5] = [(4.0, "4_0"), (2.0, "2_0"), (1.9, "1_9"), (1.0, "1_0"), (0.5, "0_5")];
/// The four directions the draft's chapter 11 turns the sheet to, degrees.
pub const SHOWN_DIRECTIONS: [u32; 4] = [0, 45, 90, 135];
/// Directions in the midpoint average over half a turn.
pub const AVERAGE_DIRECTIONS: usize = 7200;
/// The fixed sets of directions whose estimate the draft's chapters 23 and
/// 24 show.
pub const LADDER: [usize; 2] = [4, 6];
/// Directions in the midpoint rule for how often each count turns up (the
/// oracle integrates over directions in closed form, and checks that grids
/// from 1,800 directions up round every shown value the same).
pub const SHARE_DIRECTIONS: usize = 1800;
/// Chapter 04's slide, cm, and the samples it is read at (the oracle's
/// `SLIDE_SHOWN` and `SLIDE_SHOWN_STEPS`).
pub const SLIDE_SHOWN: f64 = 1.5;
pub const SLIDE_SHOWN_STEPS: usize = 15000;
/// The sheets of the draft's chapter 25, by gap in cm, and their keys.
pub const WIDE_GAPS: [(f64, &str); 3] = [(2.0, "2"), (4.0, "4"), (8.0, "8")];
/// The draft's 21 and 22: ten drops, then a thousand.
pub const FEW_DROPS: usize = 10;
pub const MANY_DROPS: usize = 1000;

/// `x` rounded half away from zero to `places` decimals, as the oracle's
/// `fixed` writes it (never "-0.0").
pub fn fixed(x: f64, places: usize) -> String {
    let scale = 10f64.powi(places as i32);
    let r = (x * scale).round() / scale;
    let r = if r == 0.0 { 0.0 } else { r };
    format!("{r:.places$}")
}

/// The pieces of a kind.
fn count(pick: fn(&Piece) -> bool) -> usize {
    COAST.iter().filter(|p| pick(p)).count()
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let coast_for_drops = model::coast();
    let drops = crate::drop_samples::counts(&crate::drop_samples::samples()?, &coast_for_drops);
    let few = &drops[..FEW_DROPS];
    f.add("drops.seed", crate::drop_samples::SEED)?;
    f.add("drops.few", FEW_DROPS)?;
    f.add(
        "drops.few.counts",
        few.iter().map(usize::to_string).collect::<Vec<_>>().join(","),
    )?;
    f.add(
        "drops.few.mean",
        fixed(few.iter().sum::<usize>() as f64 / FEW_DROPS as f64, 1),
    )?;
    let estimate = crate::drop_samples::estimate(few.iter().sum(), FEW_DROPS);
    f.add("drops.few.estimate", fixed(estimate, 1))?;
    f.add("drops.few.within", fixed((estimate - coast_for_drops.length).abs(), 2))?;
    f.add("drops.few.lowest", few.iter().min().unwrap())?;
    f.add("drops.few.highest", few.iter().max().unwrap())?;
    f.add("drops.many", drops.len())?;
    f.add(
        "drops.sample_range",
        format!("{}..{}", drops.iter().min().unwrap(), drops.iter().max().unwrap()),
    )?;
    f.add("drops.error_cm", fixed(crate::drop_samples::ERROR_CM, 0))?;
    f.add("drops.repeat_seed", crate::drop_samples::REPEAT_SEED)?;
    f.add(
        "drops.many.mean",
        fixed(drops.iter().sum::<usize>() as f64 / drops.len() as f64, 2),
    )?;
    f.add(
        "drops.many.estimate",
        fixed(crate::drop_samples::estimate(drops.iter().sum(), drops.len()), 1),
    )?;
    f.add(
        "drops.many.histogram",
        crate::drop_samples::histogram(&drops)
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| format!("{k}:{n}"))
            .collect::<Vec<_>>()
            .join(","),
    )?;
    let repeat = crate::drop_samples::totals(&crate::drop_samples::counts(
        &crate::drop_samples::repeated()?,
        &coast_for_drops,
    ));
    f.add("drops.ten.repeats", repeat.len())?;
    f.add(
        "drops.ten.far_percent",
        fixed(
            100. * repeat
                .iter()
                .filter(|&&n| crate::drop_samples::far(n, &coast_for_drops))
                .count() as f64
                / repeat.len() as f64,
            0,
        ),
    )?;
    let pins = crate::pin_lines::samples()?;
    let hits = crate::pin_lines::totals(&pins);
    f.add("lines.seed", crate::pin_lines::SEED)?;
    f.add("lines.count", pins.len())?;
    let [a, b] = crate::pin_lines::MIDDLE;
    f.add("lines.stick", fixed((a[0] - b[0]).hypot(a[1] - b[1]), 0))?;
    f.add("lines.pins.middle", hits[0])?;
    f.add("lines.pins.corner", hits[1])?;
    let even = crate::line_chart::samples()?;
    let even_hits = crate::pin_lines::totals(&even);
    f.add("lines.even.middle", even_hits[0])?;
    f.add("lines.even.corner", even_hits[1])?;
    f.add("lines.chart_shown", even.len() / 10)?;
    f.add("lines.half_turn", fixed(std::f64::consts::PI.to_degrees(), 0))?;
    f.add("lines.reach", fixed(crate::line_chart::reach(), 1))?;
    let band = crate::stick_band::example([0., 0.], PI / 2.);
    f.add("chart.stick_band", fixed(band.area(), 2))?;
    let areas = [[0., 0.], [3., 0.]].map(|center| crate::stick_band::example(center, PI / 2.).area());
    let rotated = crate::stick_band::example([0., 0.], 0.).area();
    f.add(
        "chart.stick_poses",
        [areas[0], areas[1], rotated].map(|x| fixed(x, 2)).join(","),
    )?;
    f.add("chart.stick_quadrature", fixed(band.quadrature(7200), 2))?;
    let coast = model::coast();
    if model::crosses_itself(&coast) {
        return Err("the coast meets itself".into());
    }

    // the card, the sheet, the coast
    f.add("card.width", fixed(CARD_W, 0))?;
    f.add("card.height", fixed(CARD_H, 0))?;
    f.add("sheet.gap", fixed(GAP, 0))?;
    f.add("coast.pieces", COAST.len())?;
    f.add("coast.arcs", count(|p| matches!(p, Piece::Arc { .. })))?;
    f.add("coast.straight_runs", count(|p| matches!(p, Piece::Straight(_))))?;
    let (straight, pi_part) = model::exact_length(&COAST);
    f.pin(
        "coast.length_exact",
        format!("{straight} + {pi_part}pi"),
        "15/2 + 1399/240pi",
        "the eleven pieces added by hand (GentleLantern, 2026-10-24)",
    )?;
    let length = straight.value() + pi_part.value() * PI;
    if (length - coast.length).abs() > 1e-9 {
        return Err(format!(
            "the placed coast is {} cm, not its exact {length}",
            coast.length
        ));
    }
    f.add("coast.length", fixed(length, 1))?;
    f.add("coast.length_2dp", fixed(length, 2))?;
    let (start, end) = coast.ends();
    f.add(
        "coast.ends_apart",
        fixed((start[0] - end[0]).hypot(start[1] - end[1]), 1),
    )?;

    // 01_3: the ruler under the beach, and on the headland's top
    f.add("ruler.length", fixed(RULER, 0))?;
    let (at, along) = model::ruler_on_beach(&coast);
    f.add(
        "ruler.beach_contact",
        fixed(coast.contact_along(at, along, RULER / 2.0), 1),
    )?;
    let (top, out) = coast.headland_top();
    let heights = coast.heights_above(top, out);
    if heights.iter().any(|&h| h > 1e-9) {
        return Err("part of the coast rises above the headland's top".into());
    }
    // a piece touches the ruler where its highest point reaches the line;
    // an arc tangent to the line touches it at one point
    f.add("ruler.headland_touches", heights.iter().filter(|&&h| h > -1e-9).count())?;

    // 01_4 to 01_7: the dividers
    f.add("dividers.opening_step", fixed(OPENING_STEP, 0))?;
    for (step, key) in DIVIDER_STEPS {
        let walk = coast.walk(step);
        f.add(&format!("dividers.{key}.steps"), walk.steps())?;
        f.add(&format!("dividers.{key}.length"), fixed(walk.length(), 1))?;
    }
    let walk = coast.walk(OPENING_STEP);
    f.add("dividers.2_0.last", fixed(walk.last, 2))?;
    let (step, skipped) = headland_step(&coast, &walk)?;
    f.add("dividers.2_0.headland_step", step)?;
    f.add("dividers.2_0.headland_coast", fixed(skipped, 2))?;

    // Chapters 02/03: the shown sheet and controlled equal-length beach replacements.
    let slide = crate::chapters::sheet::shown_slide(&coast, 0.0);
    let crossings = crate::chapters::sheet::crossings(&coast, 0.0, slide);
    let lines = crate::chapters::sheet::per_line(&crossings);
    let lowest = lines.first().ok_or("the shown sheet meets no coast")?.0;
    f.add("open.slide", fixed(slide, 2))?;
    f.add("open.crossings", crossings.len())?;
    f.add("open.lines_crossed", lines.len())?;
    f.add(
        "open.per_line",
        lines.iter().map(|(_, n)| n.to_string()).collect::<Vec<_>>().join(","),
    )?;
    f.add(
        "open.crossing_pieces",
        crossings
            .iter()
            .map(|c| model::PIECE_NAMES[c.piece])
            .collect::<Vec<_>>()
            .join(","),
    )?;
    f.add(
        "open.crossing_lines",
        crossings
            .iter()
            .map(|c| (c.line - lowest).to_string())
            .collect::<Vec<_>>()
            .join(","),
    )?;
    f.add("open.gap_guess", fixed(crossings.len() as f64 * GAP, 0))?;
    f.add("open.lower_crossings", lines.first().ok_or("no lower line")?.1)?;
    f.add("open.upper_crossings", lines.get(1).ok_or("no upper line")?.1)?;
    f.add(
        "open.bay_crossings",
        crossings
            .iter()
            .filter(|c| model::PIECE_NAMES[c.piece] == "bay")
            .count(),
    )?;
    f.add(
        "open.end_crossings",
        crossings
            .iter()
            .filter(|c| model::PIECE_NAMES[c.piece] == "end")
            .count(),
    )?;
    f.add("scale.ripples", 7)?;
    for (key, n) in [("big", 1), ("small", 7)] {
        let replaced = crate::chapters::sheet::rippled(&coast, n)?;
        let count = crate::chapters::sheet::crossings(&replaced, 0.0, slide).len();
        f.add(&format!("swap.{key}_crossings"), count)?;
        f.add(
            &format!("swap.{key}_more"),
            count.checked_sub(crossings.len()).ok_or("replacement lost crossings")?,
        )?;
        if n == 1 {
            f.add("swap.extra", fixed(replaced.length - coast.length, 0))?;
            f.add("swap.extra_3dp", fixed(replaced.length - coast.length, 3))?;
        }
    }

    // Chapter 04: the slide, sampled as the oracle samples it, and the
    // eighth of a turn at the rule's slide
    let first = crate::chapters::placement::Pose::new(0.0, slide);
    let slid = crate::chapters::placement::Move::slide(first, SLIDE_SHOWN);
    let runs = crate::chapters::placement::runs(&coast, &slid, SLIDE_SHOWN_STEPS);
    f.add("slide.distance", fixed(SLIDE_SHOWN, 1))?;
    f.add(
        "slide.sequence",
        runs.iter().map(|r| r.count.to_string()).collect::<Vec<_>>().join(","),
    )?;
    let turned = crate::chapters::sheet::shown_slide(&coast, PI / 4.0);
    f.add("turn.45.slide", fixed(turned, 2))?;
    f.add(
        "turn.45.crossings",
        crate::chapters::sheet::crossings(&coast, PI / 4.0, turned).len(),
    )?;

    // Chapter 05: the coin, as wide as the gap, at every twentieth of a
    // centimetre across one gap; each answer must be the same everywhere
    let coin = crate::chapters::a_coin_that_never_changes::Survey::across_a_gap(COIN / 2.0, slide);
    f.add("coin.width", fixed(COIN, 0))?;
    f.add("coin.rim", fixed(PI * COIN, 2))?;
    for (key, seen) in [
        ("coin.crossings", &coin.counts),
        ("coin.lines_through", &coin.through),
        ("coin.touching_lines", &coin.touched),
        ("coin.touch_points", &coin.points),
    ] {
        let [n] = seen.as_slice() else {
            return Err(format!("{key}: the coin shows {seen:?} across one gap"));
        };
        f.add(key, *n)?;
    }

    // Chapter 06: the same coast, first short and then long at a new placement.
    let worth = PI * COIN / 2.0;
    let bet = crossings.len() as f64 * worth;
    f.add("open.estimate", fixed(bet, 1))?;
    f.add("open.short_by", fixed(length - bet, 1))?;
    f.add("open.error_percent", fixed((bet - length) / length * 100.0, 0))?;
    let nine_slide = slide + crate::chapters::the_bet::NINE_BY;
    let nine = crate::chapters::sheet::crossings(&coast, 0.0, nine_slide).len();
    f.add("bet.nine_slide", fixed(nine_slide, 2))?;
    f.add("bet.nine_crossings", nine)?;
    f.add("bet.nine_estimate", fixed(nine as f64 * worth, 1))?;
    f.add("bet.nine_long_by", fixed(nine as f64 * worth - length, 1))?;

    // Chapter 07: one stick and one centre, with the shadow and crossings
    // computed from the direction actually shown.
    let home = crate::chapters::straight_sticks::HOME;
    let [lying_across, lying_along] = [
        crate::chapters::straight_sticks::ACROSS,
        crate::chapters::straight_sticks::ALONG,
    ]
    .map(|theta| crate::chapters::straight_sticks::crossings(home, theta, slide).len());
    f.add("stick.length", fixed(crate::chapters::straight_sticks::LENGTH, 0))?;
    f.add("stick.home_across", lying_across)?;
    f.add("stick.home_along", lying_along)?;
    f.add("stick.across_estimate", fixed(lying_across as f64 * worth, 1))?;
    f.add("stick.along_estimate", fixed(lying_along as f64 * worth, 0))?;
    f.add(
        "stick.turn_counts",
        crate::chapters::straight_sticks::turn_counts(home, slide)
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(","),
    )?;
    for (key, theta) in [
        ("stick.shadow_across", crate::chapters::straight_sticks::ACROSS),
        ("stick.shadow_along", crate::chapters::straight_sticks::ALONG),
    ] {
        let [lo, hi] = crate::chapters::straight_sticks::shadow(home, theta);
        f.add(key, fixed(hi - lo, 0))?;
    }

    // Chapters 08 and 09: tilt the stick from across, then turn partway
    // back for a longer test shadow. At each angle slide one gap; then
    // average the shadow over a half turn.
    let tilted = crate::chapters::the_shadow_rule::TILTED;
    let tested = crate::chapters::the_shadow_rule::tilted_to(crate::chapters::the_shadow_rule::TEST_SHADOW);
    let slides = [tilted, tested].map(|theta| crate::chapters::the_shadow_rule::slide_runs(theta, slide));
    f.add(
        "stick.tilt",
        fixed(crate::chapters::the_shadow_rule::TILT.to_degrees(), 0),
    )?;
    for (theta, runs, [first, counts]) in [
        (tilted, &slides[0], ["stick.tilted_first", "stick.tilted_counts"]),
        (tested, &slides[1], ["stick.test_first", "stick.test_counts"]),
    ] {
        f.add(
            first,
            crate::chapters::straight_sticks::crossings(home, theta, slide).len(),
        )?;
        let passed: Vec<String> = runs.iter().map(|r| r.count.to_string()).collect();
        f.add(counts, passed.join(","))?;
    }
    let share = crate::chapters::the_shadow_rule::share;
    let reach = crate::chapters::the_shadow_rule::reach;
    f.add("stick.tilted_one_share", fixed(share(&slides[0], 1), 2))?;
    f.add("stick.tilted_two_share", fixed(share(&slides[0], 2), 2))?;
    f.add(
        "stick.at30_mean",
        fixed(crate::chapters::the_shadow_rule::mean(&slides[0]), 1),
    )?;
    f.add("stick.reach30", fixed(reach(tilted), 0))?;
    f.add("stick.tilted_leftover", fixed(reach(tilted) - GAP, 0))?;
    let (whole, held) = crate::chapters::the_shadow_rule::cut_counts(tilted, slide);
    let whole: Vec<String> = whole.iter().map(i64::to_string).collect();
    f.add("stick.gap_piece_counts", whole.join(","))?;
    f.add("stick.leftover_share", fixed(held, 2))?;
    f.add("stick.test_shadow", fixed(reach(tested), 1))?;
    f.add("stick.test_three_share", fixed(share(&slides[1], 3), 2))?;
    f.add(
        "stick.test_mean",
        fixed(crate::chapters::the_shadow_rule::mean(&slides[1]), 2),
    )?;
    f.add(
        "stick.across_rule",
        fixed(reach(crate::chapters::straight_sticks::ACROSS) / GAP, 0),
    )?;
    f.add(
        "stick.along_rule",
        fixed(reach(crate::chapters::straight_sticks::ALONG) / GAP, 0),
    )?;
    let shadow_mean = crate::chapters::turning_the_stick::mean_shadow();
    let stick = crate::chapters::straight_sticks::LENGTH;
    f.add("stick.mean_reach", fixed(shadow_mean, 2))?;
    f.add("stick.mean_reach_share", fixed(shadow_mean / stick, 2))?;
    f.add("stick.mean_reach_percent", fixed(shadow_mean / stick * 100.0, 0))?;
    f.add("stick.turned_mean", fixed(shadow_mean / GAP, 2))?;
    f.add("stick.turned_estimate", fixed(shadow_mean / GAP * worth, 1))?;

    // the film's identity, for the draft's later chapters: a crossing's
    // worth, each shown direction's travel, the average over every
    // direction, and fixed sets of directions
    f.add("coin.worth", fixed(PI * COIN / 2.0, 2))?;
    for beta in SHOWN_DIRECTIONS {
        let travel = coast.travel(f64::from(beta));
        f.add(&format!("turn.{beta}.reach"), fixed(travel, 2))?;
        f.add(&format!("turn.{beta}.slide_mean"), fixed(travel / GAP, 2))?;
        // the draft's 11_1 table halves the travel shown to three places
        f.add(&format!("turn.{beta}.reach_3dp"), fixed(travel, 3))?;
    }
    let four: f64 = SHOWN_DIRECTIONS
        .iter()
        .map(|&b| coast.travel(f64::from(b)) / GAP)
        .sum::<f64>()
        / 4.0;
    f.add("turn.slide_mean", fixed(four, 2))?;
    f.add("turn.slide_estimate", fixed(four * model::worth(GAP), 2))?;
    let mean = coast.mean_travel(AVERAGE_DIRECTIONS);
    f.add("crofton.mean_crossings", fixed(mean / GAP, 2))?;
    f.add("crofton.mean_reach", fixed(mean, 2))?;
    f.add("crofton.twice_length", fixed(2.0 * length, 2))?;
    // the places the draft's 11_4, 11_5 and 19_4 show, so that each shown
    // result follows from its shown operands
    f.add("turn.slide_mean_3dp", fixed(four, 3))?;
    f.add("crofton.mean_reach_3dp", fixed(mean, 3))?;
    f.add("crofton.mean_crossings_4dp", fixed(mean / GAP, 4))?;
    f.add("coast.length_3dp", fixed(length, 3))?;
    for m in LADDER {
        f.add(
            &format!("ladder.{m}.coast_estimate"),
            fixed(coast.fixed_directions_estimate(m, GAP), 2),
        )?;
    }

    // Chapter23 uses the exact closed-form extrema, rounded for the labels.
    for m in [1, 2, 4] {
        let (lo, hi) = crate::chapters::fixed_directions::bounds(m);
        let places = usize::from(m > 1);
        f.add(&format!("ladder.{m}.low_percent"), fixed((lo - 1.) * 100., places))?;
        f.add(&format!("ladder.{m}.high_percent"), fixed((hi - 1.) * 100., places))?;
    }
    f.add(
        "ladder.4.coast_error_percent",
        fixed((coast.fixed_directions_estimate(4, GAP) / length - 1.) * 100., 1),
    )?;

    // Chapter24: exact calibration, distinct from finite-position counts.
    let (lo, hi) = crate::chapters::fixed_directions::bounds(6);
    f.add("ladder.6.low_percent", fixed((lo - 1.) * 100., 1))?;
    f.add("ladder.6.high_percent", fixed((hi - 1.) * 100., 1))?;
    f.add("steinhaus.directions", crate::chapters::the_longimeter::DIRECTIONS)?;
    f.add(
        "steinhaus.angle",
        fixed(180. / crate::chapters::the_longimeter::DIRECTIONS as f64, 0),
    )?;
    f.add("steinhaus.gap_mm", fixed(crate::chapters::the_longimeter::gap_mm(), 2))?;
    f.add(
        "steinhaus.coast_mean_mm",
        fixed(crate::chapters::the_longimeter::slide_mean_mm(&coast), 2),
    )?;
    f.add("steinhaus.coast_length_mm", fixed(length * 10., 2))?;
    for (i, pose) in crate::chapters::the_longimeter::POSES.into_iter().enumerate() {
        f.add(
            &format!("steinhaus.placement.{i}.count"),
            crate::chapters::the_longimeter::points(&coast, pose, 3).len(),
        )?;
    }

    // Chapters 10 and 11: the coast cut into sticks, each one way across
    // the first lines; the beach's shadow along those lines and across
    // them; how far the four directions fall short; and evenly spread sets
    // of directions from 0°, the last the computer's (numerical averages,
    // which agree with the identity's 2L/π only to the places shown)
    let sticks = crate::chapters::a_crowd_of_sticks::sticks(&coast);
    f.add("sticks.count", sticks.len())?;
    let beach = crate::chapters::a_crowd_of_sticks::piece(
        &coast,
        coast.parts[model::BEACH].start,
        coast.parts[model::BEACH].end(),
    );
    f.add("beach.shadow_0", fixed(beach.travel(0.0), 0))?;
    f.add("beach.shadow_90", fixed(beach.travel(90.0), 1))?;
    let four_cm = coast.fixed_directions_estimate(SHOWN_DIRECTIONS.len(), GAP);
    f.add(
        "turn.slide_short_percent",
        fixed((length - four_cm) / length * 100.0, 0),
    )?;
    for m in crate::chapters::turning_the_sheet::GRID {
        f.add(
            &format!("grid.{m}.estimate"),
            fixed(coast.fixed_directions_estimate(m, GAP), 2),
        )?;
    }
    let most = crate::chapters::turning_the_sheet::MOST;
    f.add("grid.most", most)?;
    let height = crate::chapters::turning_the_sheet::mean_total(&coast, most);
    f.add(&format!("grid.{most}.mean_reach"), fixed(height, 3))?;
    f.add(&format!("grid.{most}.mean_crossings"), fixed(height / GAP, 4))?;

    // Chapters 12 and 13: Barbier's route on the card, each number from the
    // model the chapters draw. 12: the pieces' counts at the first placement,
    // the moved stick and the sheet moved back, the halves, the nested
    // sticks along the ends' line, the chains on the coast. 13: the bent
    // wire at home, the half turn about its corner's line, each wire's count
    // shares over 3,600 directions (a midpoint rule over direction), the 4
    // and the line through its crossing. And the two years the voice names
    {
        use crate::chapters::placement::Pose;
        use crate::chapters::straight_sticks::{ACROSS, ALONG, HOME, LENGTH};
        use crate::chapters::{sheet, the_bent_stick as bent, why_every_curve as every};
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let joined = |v: &[usize]| v.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
        f.add("barbier.year", every::BARBIER_YEAR)?;
        let crossings = sheet::crossings(&coast, first.beta, first.slide);
        let by_piece: Vec<usize> = (0..coast.parts.len())
            .map(|i| crossings.iter().filter(|c| c.piece == i).count())
            .collect();
        f.add("open.piece_counts", joined(&by_piece))?;
        let home = every::placed(every::halves, HOME, ACROSS, 0.0);
        let moved_to = [HOME[0] + every::MOVE_RIGHT, HOME[1]];
        let moved = every::placed(every::halves, moved_to, every::MOVE_TO, 0.0);
        f.add("move.count", every::count_at(&moved, first))?;
        let moved_back = every::moved_back(first).end();
        f.add("move.sheet_count", every::count_at(&home, moved_back))?;
        let split = sheet::crossings(&home, first.beta, first.slide);
        let halves: Vec<usize> = (0..2).map(|i| split.iter().filter(|c| c.piece == i).count()).collect();
        f.add("halves.counts", joined(&halves))?;
        let crossed = every::placed(every::halves, HOME, ACROSS, 1.0);
        f.add("cross.count", every::count_at(&crossed, first))?;
        let meet = (HOME[1] + LENGTH / 4.0).rem_euclid(GAP);
        f.add("cross.through", sheet::crossings(&crossed, 0.0, meet).len())?;
        f.add("pieces.third", fixed(LENGTH / 3.0, 0))?;
        f.add("pieces.quarter", fixed(LENGTH / 4.0, 1))?;
        f.add("pieces.three_quarters", fixed(3.0 * LENGTH / 4.0, 1))?;
        let (a, b) = coast.ends();
        let apart = every::dist(a, b);
        f.add("ends.apart_2dp", fixed(apart, 2))?;
        for k in 0..3 {
            let scale = 10f64.powi(k);
            let lo = (apart * scale).floor() / scale;
            let (low, high) = (fixed(lo, k as usize), fixed(lo + 1.0 / scale, k as usize));
            f.add(&format!("ends.bracket.{k}"), format!("{low},{high}"))?;
        }
        let nested = [every::NESTED[0], apart, every::NESTED[1]].map(|l| every::along_ends(&coast, l));
        for (i, mv) in every::nested_moves(first).iter().enumerate() {
            let counts: Vec<usize> = nested.iter().map(|c| every::count_at(c, mv.end())).collect();
            f.add(&format!("ends.nested.{i}"), joined(&counts))?;
        }
        for (name, at) in ["joints", "cuts", "sticks"].iter().zip(every::chain_points(&coast)) {
            let chain = every::chain(&coast, &at);
            f.add(&format!("chain.{name}.chords"), chain.parts.len())?;
            f.add(&format!("chain.{name}.length"), fixed(chain.length, 2))?;
            f.add(&format!("chain.{name}.crossings"), every::count_at(&chain, first))?;
        }
        let wire = every::placed(bent::shape, HOME, ALONG, bent::BENT);
        f.add("hook.home_count", every::count_at(&wire, first))?;
        let [half_turn] = bent::sweep(first);
        let mut seen: Vec<usize> = Vec::new();
        for i in 0..=3600 {
            let n = every::count_at(&wire, half_turn.at(f64::from(i) / 3600.0));
            if seen.last() != Some(&n) {
                seen.push(n);
            }
        }
        f.add("hook.sweep", joined(&seen))?;
        for (key, counts_key, u) in [
            ("stick", "stick.all_counts", bent::STRAIGHT),
            ("hook", "hook.counts", bent::BENT),
        ] {
            let shares = bent::shares(u);
            let low = shares.iter().position(|&p| p > 1e-9).unwrap_or(0);
            let shown: Vec<String> = shares[low..].iter().map(|&p| fixed(p * 100.0, 1)).collect();
            f.add(&format!("{key}.shares_percent"), shown.join(","))?;
            f.add(counts_key, joined(&(low..shares.len()).collect::<Vec<_>>()))?;
        }
        f.add("hook.mean", fixed(model::moments(&bent::shares(bent::BENT)).0, 2))?;
        let four = every::placed(bent::shape, HOME, ALONG, bent::FOUR);
        let at = bent::four_move(first).end();
        let here = sheet::crossings(&four, at.beta, at.slide);
        f.add("four.count", here.len())?;
        let meet = bent::four_meet();
        let through = here.iter().filter(|c| every::dist(c.at, meet) < 1e-9).count();
        f.add("four.through", through)?;
        f.add("four.mean", fixed(model::moments(&bent::shares(bent::FOUR)).0, 2))?;
        f.add("noodle.year", bent::NOODLE_YEAR)?;
    }
    // Chapters 14–15: values from the same geometry the native scene draws.
    {
        use crate::chapters::{barbiers_floorboards as floor, the_same_width as width};
        use crate::chapters::{placement::Pose, sheet, why_every_curve::count_at};
        let home = width::reuleaux(width::BENCH, width::END_TURN);
        let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
        let rulers = width::rulers(&home);
        f.add("coin.width_2dp", fixed(COIN, 2))?;
        f.add("reuleaux.width", fixed(width::SIDE, 0))?;
        f.add("reuleaux.width_2dp", fixed(rulers.width(), 2))?;
        f.add("reuleaux.arcs", home.parts.len())?;
        let arc_degrees = home.parts[0].seg.len() / width::SIDE * 180.0 / PI;
        f.add("reuleaux.arc_degrees", fixed(arc_degrees, 0))?;
        f.add("reuleaux.rim", fixed(home.length, 2))?;
        f.add("reuleaux.home_count", count_at(&home, first))?;
        let between = (-4..=4)
            .filter(|&k| {
                let h = first.slide + f64::from(k) * GAP;
                rulers.heights[0] < h && h < rulers.heights[1]
            })
            .count();
        f.add("reuleaux.lines_between", between)?;
        let (seen, _) = floor::shown_counts(first);
        f.add(
            "reuleaux.wander_counts",
            seen.iter().map(usize::to_string).collect::<Vec<_>>().join(","),
        )?;
        f.add(
            "reuleaux.mean_crossings",
            fixed(
                model::moments(&home.count_shares(floor::SHARE_DIRECTIONS, Some(GAP))).0,
                0,
            ),
        )?;
        f.add(
            "reuleaux.chain_corners",
            floor::CHAINS.iter().map(usize::to_string).collect::<Vec<_>>().join(","),
        )?;
        f.add(
            "reuleaux.chain_lengths",
            floor::CHAINS
                .iter()
                .map(|&n| fixed(floor::chain(width::BENCH, floor::ARRANGE_TURN, n).length, 2))
                .collect::<Vec<_>>()
                .join(","),
        )?;
        f.add(
            "triangle.perimeter",
            fixed(width::triangle(width::BENCH, 0.0).length, 0),
        )?;
        f.add(
            "triangle.width_flat",
            fixed(
                width::rulers(&width::triangle(width::BENCH, width::HALF_TURN)).width(),
                2,
            ),
        )?;
        f.add(
            "triangle.width_upright",
            fixed(width::rulers(&width::triangle(width::BENCH, width::UPRIGHT)).width(), 2),
        )?;
    }
    // the draft's 19_1: how much of the chart each single line's count fills
    let lines = coast.count_shares(SHARE_DIRECTIONS, None);
    let meet: f64 = lines.iter().skip(1).sum();
    let card_lines = 2.0 * (CARD_W + CARD_H);
    f.add("chart.line_max", lines.len() - 1)?;
    f.add("chart.coast_lines", fixed(meet, 2))?;
    f.add("chart.meet_percent", fixed(meet / card_lines * 100.0, 0))?;
    f.add("chart.miss_percent", fixed((1.0 - meet / card_lines) * 100.0, 0))?;
    f.add("chart.once_percent", fixed(lines[1] / meet * 100.0, 0))?;
    let grid = crate::coast_chart::Grid::build(&coast, crate::coast_chart::COLUMNS, crate::coast_chart::ROWS);
    let bins = grid.histogram();
    f.add("chart.grid_columns", grid.columns)?;
    f.add("chart.grid_rows", grid.rows)?;
    f.add(
        "chart.grid_bins",
        bins.iter().map(usize::to_string).collect::<Vec<_>>().join(","),
    )?;
    f.add("chart.grid_total", fixed(*grid.prefix().last().unwrap(), 5))?;
    f.add("chart.grid_total_2dp", fixed(*grid.prefix().last().unwrap(), 2))?;
    f.add(
        "chart.grid_miss_percent",
        fixed(100. * bins[0] as f64 / bins.iter().sum::<usize>() as f64, 0),
    )?;
    f.add(
        "chart.grid_once_percent",
        fixed(100. * bins[1] as f64 / bins.iter().skip(1).sum::<usize>() as f64, 0),
    )?;
    f.add("chart.grid_max", bins.len() - 1)?;
    f.add(
        "chart.selected_counts",
        [0, 1, bins.len() - 1]
            .map(|n| {
                crate::coast_chart::intersections(&coast, grid.representative(&coast, n))
                    .len()
                    .to_string()
            })
            .join(","),
    )?;
    f.add("chart.coast_total", fixed(mean * PI, 2))?;
    f.add("chart.exact_total_5dp", fixed(2. * length, 5))?;
    f.add("crofton.year", crate::coast_chart::CROFTON_YEAR)?;
    let radius = COIN / 2.;
    let coin_band = crate::chapters::the_formula::coin_band(radius);
    f.add("chart.coin_band", fixed(coin_band, 2))?;
    f.add("chart.coin_total", fixed(2. * coin_band, 2))?;
    f.add("coin.radius", fixed(radius, 0))?;
    f.add(
        "formula.coin_counts",
        crate::chapters::the_formula::selected_coin_counts(),
    )?;
    f.add(
        "cauchy.reported_year",
        crate::chapters::the_formula::CAUCHY_REPORTED_YEAR,
    )?;
    let rippled = crate::chapters::sheet::rippled(&coast, 7)?;
    let beach = coast.parts[model::BEACH].seg.len();
    f.add("scale.beach", fixed(beach, 1))?;
    f.add("scale.radius", fixed(beach / 14., 2))?;
    f.add("scale.fine_radius", fixed(beach / 28., 3))?;
    f.add("scale.rippled_beach", fixed(beach * PI / 2., 1))?;
    f.add("scale.factor", fixed(PI / 2., 2))?;
    f.add("scale.rippled_length", fixed(rippled.length, 1))?;
    let Piece::Straight(beach_ratio) = COAST[model::BEACH] else {
        return Err("beach not straight".into());
    };
    let new_straight = straight.plus(beach_ratio.times(model::Ratio::whole(-1)));
    let beach_pi = beach_ratio.times(model::Ratio::new(1, 2));
    let new_pi = pi_part.plus(beach_pi);
    f.add("scale.rippled_length_exact", format!("{new_straight} + {new_pi}pi"))?;
    f.add("scale.beach_exact", format!("{beach_pi}pi"))?;
    f.add(
        "scale.rippled_mean_crossings",
        fixed(2. * rippled.length / (PI * GAP), 2),
    )?;
    f.add("scale.ripples_fine", 14)?;
    f.add(
        "scale.rippled_beach_fine",
        fixed(
            crate::chapters::sheet::rippled(&coast, 14)?.length - coast.length + beach,
            1,
        ),
    )?;
    f.add("scale.divider_original_4dp", fixed(coast.walk(2.).length(), 4))?;
    f.add("scale.divider_rippled_4dp", fixed(rippled.walk(2.).length(), 4))?;
    f.add("dividers.2_0.rippled_length", fixed(rippled.walk(2.).length(), 1))?;
    f.add("scale.one_ripple_mean", fixed(beach / 14., 2))?;
    f.add("scale.beach_mean", fixed(beach / 2., 2))?;
    let height = coast.parts[model::BEACH].seg.point(0.)[1] + 0.125;
    let count: usize = rippled.parts[model::BEACH..model::BEACH + 7]
        .iter()
        .map(|part| {
            model::Curve {
                parts: vec![*part],
                length: part.seg.len(),
            }
            .meetings(0., height)
        })
        .sum();
    f.add("scale.selected_beach_crossings", count)?;
    // Mandelbrot1967 p636 explicitly quotes Steinhaus1954; not measured data.
    f.add("history.vistula_multipliers", "10,100,1000")?;
    use crate::chapters::how_long_is_a_coastline as scaling;
    let lengths = scaling::lengths(&coast);
    f.add("richardson.west_coast_d", fixed(scaling::REPORTED_D, 2))?;
    f.add(
        "richardson.halving_percent",
        fixed((scaling::fitted_ratio(0.5) - 1.) * 100., 0),
    )?;
    f.add("richardson.halving_factor", fixed(scaling::fitted_ratio(0.5), 6))?;
    f.add("richardson.halving_factor_3dp", fixed(scaling::fitted_ratio(0.5), 3))?;
    f.add(
        "richardson.normalized_law",
        scaling::STEPS.map(|g| fixed(scaling::fitted_ratio(g), 4)).join(","),
    )?;
    f.add("scaling.steps", scaling::STEPS.map(|g| fixed(g, 3)).join(","))?;
    f.add("scaling.smooth_lengths", lengths.map(|l| fixed(l, 4)).join(","))?;
    f.add(
        "scaling.smooth_normalized",
        lengths.map(|l| fixed(l / lengths[0], 4)).join(","),
    )?;
    f.add(
        "dividers.halving_percent",
        fixed((lengths[1] / lengths[0] - 1.) * 100., 0),
    )?;
    use crate::chapters::a_square_on_the_sheet as square;
    f.add("square.side", fixed(square::SIDE, 0))?;
    f.add("square.rim", fixed(square::square().length, 0))?;
    f.add("square.mean", fixed(square::mean_count(), 2))?;
    f.add("square.four_share_percent", fixed(100. * square::probability_four(), 0))?;
    f.add("square.four_share_exact", "4/pi - 1")?;
    f.add(
        "square.two_share_percent",
        fixed(100. * (1. - square::probability_four()), 0),
    )?;
    f.add("square.max_width", fixed(square::width(std::f64::consts::PI / 4.), 2))?;
    f.add(
        "square.selected_counts",
        square::POSES
            .map(|(b, u)| square::contacts(b.to_radians(), u).0.len().to_string())
            .join(","),
    )?;
    f.add(
        "square.corner_contacts",
        square::contacts(std::f64::consts::PI / 4., std::f64::consts::SQRT_2)
            .0
            .len()
            .to_string(),
    )?;
    f.add("square.coincident_edges", square::contacts(0., 1.).1.len().to_string())?;
    // Numerical population spread (midpoint orientation quadrature), distinct
    // from the retained finite experiment. SD is not a coverage interval.
    for (gap, key) in WIDE_GAPS {
        let counts = crate::chapters::the_gap::retained_counts(&coast, gap)?;
        let histogram = crate::drop_samples::histogram(&counts);
        f.add(&format!("gap.{key}.mean"), fixed(2. * length / (PI * gap), 2))?;
        f.add(&format!("gap.{key}.worth"), fixed(model::worth(gap), 2))?;
        f.add(
            &format!("gap.{key}.drops_mean_estimate"),
            fixed(
                counts.iter().sum::<usize>() as f64 / counts.len() as f64 * model::worth(gap),
                1,
            ),
        )?;
        f.add(
            &format!("gap.{key}.counts"),
            format!("{}..{}", counts.iter().min().unwrap(), counts.iter().max().unwrap()),
        )?;
        f.add(
            &format!("gap.{key}.retained_histogram"),
            histogram.iter().map(usize::to_string).collect::<Vec<_>>().join(","),
        )?;
        let shares = coast.count_shares(SHARE_DIRECTIONS, Some(gap));
        let (_, sd) = model::moments(&shares);
        let one = sd * model::worth(gap);
        let range = format!(
            "{}..{}",
            shares.iter().position(|&p| p > 0.0).unwrap_or(0),
            shares.len() - 1
        );
        f.add(&format!("gap.{key}.one_drop_spread"), fixed(one, 1))?;
        f.add(&format!("gap.{key}.count_range"), &range)?;
        if gap != GAP {
            continue;
        }
        f.add("drops.one.spread", fixed(one, 1))?;
        f.add("drops.few.spread", fixed(one / (FEW_DROPS as f64).sqrt(), 1))?;
        f.add("drops.many.spread", fixed(one / (MANY_DROPS as f64).sqrt(), 2))?;
        // the fewest drops whose spread, one ÷ √n, is a tenth or less
        let for_tenth = (one / 0.1).powi(2).ceil() as u64;
        f.add("drops.for_tenth", for_tenth)?;
        f.add(
            "drops.for_tenth_about",
            (for_tenth as f64 / 1000.0).round() as u64 * 1000,
        )?;
        f.add("drops.count_range", &range)?;
        let top_count = shares.len() - 1;
        f.add("drops.top_count", top_count)?;
        f.add("drops.top_count_percent", fixed(shares[top_count] * 100.0, 3))?;
        let far: f64 = model::sum_shares(&shares, FEW_DROPS)
            .iter()
            .enumerate()
            .filter(|&(s, _)| (s as f64 / FEW_DROPS as f64 * model::worth(gap) - length).abs() > 1.0)
            .map(|(_, p)| p)
            .sum();
        f.add("drops.ten.far_exact_percent", fixed(far * 100.0, 0))?;
    }
    Ok(f)
}

/// The step of `walk` that leaves the headland (01_6's close-up): the first
/// whose start lies on the headland, numbered from 1, and the coast it cuts
/// across, cm.
pub fn headland_step(coast: &model::Curve, walk: &model::Walk) -> Result<(usize, f64), String> {
    let head = coast.parts[HEADLAND];
    walk.landings
        .windows(2)
        .position(|w| head.start < w[0].1 && w[0].1 < head.end())
        .map(|k| (k + 1, walk.landings[k + 1].1 - walk.landings[k].1))
        .ok_or_else(|| "no step starts on the headland".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_rounds_half_away_from_zero_as_the_oracle_does() {
        assert_eq!(fixed(2.25, 1), "2.3");
        assert_eq!(fixed(-2.25, 1), "-2.3");
        assert_eq!(fixed(0.04, 1), "0.0");
        assert_eq!(fixed(-0.04, 1), "0.0");
        assert_eq!(fixed(25.8128, 0), "26");
        assert_eq!(fixed(22.6414, 1), "22.6");
        assert_eq!(fixed(16.0, 0), "16");
    }

    /// The numbers chapter 01 shows and says, at their values (the oracle
    /// recomputes every one; these are the ones the scene builds words from).
    #[test]
    fn chapter_one_numbers() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        for (key, value) in [
            ("card.width", "16"),
            ("coast.ends_apart", "14.7"),
            ("ruler.beach_contact", "3.5"),
            ("ruler.headland_touches", "1"),
            ("dividers.opening_step", "2"),
            ("dividers.2_0.steps", "11"),
            ("dividers.2_0.last", "0.64"),
            ("dividers.2_0.length", "22.6"),
            ("dividers.2_0.headland_step", "6"),
            ("dividers.2_0.headland_coast", "3.08"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }

    /// A finding the review raised: the estimates hold their value off the
    /// 2 cm sheet (and the identity facts sit where the oracle put them).
    #[test]
    fn identity_numbers_do_not_lean_on_a_two_centimetre_gap() {
        let f = facts().unwrap();
        assert_eq!(f.get("ladder.4.coast_estimate"), Some("25.53"));
        assert_eq!(f.get("ladder.6.coast_estimate"), Some("25.73"));
        let coast = model::coast();
        for gap in [1.0, 4.0] {
            assert_eq!(fixed(coast.fixed_directions_estimate(6, gap), 2), "25.73", "gap {gap}");
        }
        assert_eq!(f.get("crofton.mean_reach"), Some("16.43"));
        assert_eq!(f.get("crofton.twice_length"), Some("51.63"));
    }

    /// The review's F1 to F4 (Mail 6835): the chart's single lines, the
    /// spreads over every placement (not the seeded draws), the extra places
    /// that let each shown product follow from its shown operands, and the
    /// smaller step that walks less. The oracle gets each in closed form.
    #[test]
    fn review_numbers_come_from_every_placement() {
        let f = facts().unwrap();
        for (key, value) in [
            ("chart.line_max", "6"),
            ("chart.coast_lines", "32.73"),
            ("chart.miss_percent", "35"),
            ("chart.once_percent", "71"),
            ("drops.one.spread", "4.4"),
            ("drops.for_tenth", "1965"),
            ("drops.for_tenth_about", "2000"),
            ("drops.count_range", "5..13"),
            ("drops.top_count", "13"),
            ("drops.top_count_percent", "0.036"),
            ("drops.ten.far_exact_percent", "43"),
            ("gap.4.one_drop_spread", "8.1"),
            ("gap.8.one_drop_spread", "14.9"),
            ("gap.8.count_range", "0..6"),
            ("turn.slide_mean_3dp", "8.127"),
            ("crofton.mean_crossings_4dp", "8.2165"),
            ("coast.length_3dp", "25.813"),
            ("dividers.1_9.steps", "11"),
            ("dividers.1_9.length", "21.6"),
        ] {
            assert_eq!(f.get(key), Some(value), "{key}");
        }
    }
}
