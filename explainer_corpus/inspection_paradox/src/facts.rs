//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x inspection_paradox facts-check`). Keys are
//! grouped by idea, as the oracle groups them:
//! - F-T1: the timetable and its gaps
//! - F-M1: your two mornings
//! - F-S1: the two ways of counting
//! - F-G1: stretching one gap
//! - F-W1, F-W2: the waits and the twelve passengers
//! - F-A1: the triangles
//! - F-O1: the slider
//! - F-F1: the formula
//! - F-X1, F-X2: the bound's edges
//! - F-R1, F-R2: random and clumped buses
//! - F-C1, F-P1, F-H1: elsewhere
//! - F-K1, F-K2: the timetable reader and the planners
//! - F-Q1: the last timetable
//!
//! Fractions print in lowest terms ("13/3"), durations as m:ss ("4:20"),
//! lists joined by commas, booleans as true/false.

use std::fmt::Display;

use explainer_kit::Facts;

use crate::model::{
    self, CLASS_SIZES, CLUMPED_GAPS, EARLY, EVEN_OFFSET, HOUR, LONG_CALL_MS, OFFSET, PER_MINUTE, PERIOD, PUZZLE_OFFSET,
    Q, RANDOM_MEAN, ROUTES, SHORT_CALL_MS, SHORT_CALLS, SWEEP_MAX_COUNT, SWEEP_MAX_GAP, clock,
};

/// The year the video is made, for "over a century ago" (10_7).
pub const THIS_YEAR: i64 = 2026;
/// The year of Feld and Grofman's article, as printed on its first page.
pub const FELD_GROFMAN_YEAR: i64 = 1977;
/// The year of Terada's essay, from Masuda and Hiraoka's abstract.
pub const TERADA_YEAR: i64 = 1922;

/// 07_5's example of an offset between whole minutes: two and a half.
pub fn slider_example() -> Q {
    Q::new(5, 2)
}

/// Values joined by commas.
pub fn csv<T: Display>(xs: impl IntoIterator<Item = T>) -> String {
    xs.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn truth(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

/// The example's short and long gaps, in the order the hour meets them.
pub fn short_and_long() -> (i64, i64) {
    let g = model::hour_gaps(OFFSET);
    (*g.iter().min().expect("gaps"), *g.iter().max().expect("gaps"))
}

/// The red bus's offsets chapter 03 steps through: six down to none.
pub fn stretch_offsets() -> Vec<i64> {
    (0..=EVEN_OFFSET).rev().collect()
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let gaps = model::hour_gaps(OFFSET);
    let (short, long) = short_and_long();
    let mean_gap = model::mean_gap(&gaps);
    let wait = model::mean_wait(&gaps);
    let half = Q::int(2);
    let count = |g: i64| gaps.iter().filter(|&&x| x == g).count() as i64;

    // 01, 02: the stop, the sign, the gaps
    f.add("F-T1.routes", ROUTES)?;
    f.add("F-T1.period", PERIOD)?;
    f.add("F-T1.offset", OFFSET)?;
    f.add("F-T1.hour", HOUR)?;
    f.add("F-T1.cycles_per_hour", HOUR / PERIOD)?;
    f.add("F-T1.blue_times", csv(model::route(0)))?;
    f.add("F-T1.red_times", csv(model::route(OFFSET)))?;
    f.add("F-T1.buses_per_hour", model::bus_times(OFFSET).len())?;
    f.add("F-T1.hour_gaps", csv(&gaps))?;
    f.add("F-T1.gaps", csv(&gaps[..2]))?;
    f.add("F-T1.short", short)?;
    f.add("F-T1.long", long)?;
    f.add("F-T1.short_count", count(short))?;
    f.add("F-T1.long_count", count(long))?;
    f.add("F-T1.mean_gap", mean_gap)?;
    f.add("F-T1.half_mean_gap", mean_gap / half)?;
    f.add("F-T1.long_share_of_gaps", Q::new(count(long), gaps.len() as i64))?;
    let long_time = model::time_share(&gaps, |g| g == long);
    f.add("F-T1.long_minutes", long_time * Q::int(HOUR))?;
    f.add("F-T1.long_time_share", long_time)?;

    // 01, 04: the average wait, and by kind of gap
    f.pin(
        "F-W1.mean_wait",
        wait,
        "13/3",
        "52 minutes of waiting over 12 minutes, by hand",
    )?;
    f.pin(
        "F-W1.mean_wait_clock",
        clock(wait)?,
        "4:20",
        "13/3 min is 4 min 20 s, by hand",
    )?;
    f.add("F-W1.extra_clock", clock(wait - mean_gap / half)?)?;
    let short_wait = Q::new(short, 2);
    let long_wait = Q::new(long, 2);
    f.add("F-W1.short_mean_wait", short_wait)?;
    f.add("F-W1.long_mean_wait", long_wait)?;
    f.add("F-W1.naive_mean_of_waits", (short_wait + long_wait) / half)?;
    // 12_5: your train needs the longest wait, not the average
    let allowance = model::longest_wait(OFFSET);
    f.add("F-W1.allowance", allowance)?;
    f.add("F-W1.allowance_clock", clock(Q::int(allowance))?)?;
    if wait >= Q::int(allowance) {
        return Err("the average wait is not below the train's allowance".into());
    }

    // 04: one passenger in the middle of every minute of one twelve-minute stretch
    let short_waits = model::midpoint_waits(short);
    let long_waits = model::midpoint_waits(long);
    let people = short_waits.len() + long_waits.len();
    let all_waits: Q = short_waits.iter().chain(&long_waits).copied().sum();
    f.add("F-W2.per_minute", PER_MINUTE)?;
    f.add("F-W2.people", people)?;
    f.add("F-W2.short_people", short_waits.len())?;
    f.add("F-W2.long_people", long_waits.len())?;
    f.add("F-W2.short_waits", csv(&short_waits))?;
    let (first, last) = (long_waits[0], long_waits[long_waits.len() - 1]);
    f.add("F-W2.long_waits_range", csv([first, last]))?;
    f.add("F-W2.short_total", short_waits.iter().copied().sum::<Q>())?;
    f.add("F-W2.long_total", long_waits.iter().copied().sum::<Q>())?;
    f.pin("F-W2.total", all_waits, "52", "½ + 1½ + 9½ + 8½ + … + ½, by hand")?;
    f.add("F-W2.mean_wait", all_waits / Q::int(people as i64))?;
    f.add(
        "F-W2.people_ratio",
        Q::new(long_waits.len() as i64, short_waits.len() as i64),
    )?;
    if all_waits / Q::int(people as i64) != wait {
        return Err("the twelve passengers' average is not the average over moments".into());
    }

    // 05: the triangles
    let short_area = model::triangle_area(short);
    let long_area = model::triangle_area(long);
    f.add("F-A1.short_area", short_area)?;
    f.add("F-A1.long_area", long_area)?;
    f.add("F-A1.total_area", short_area + long_area)?;
    // each passenger's bar is as wide as the minute they stand for
    f.add("F-A1.bar_width", Q::new(1, PER_MINUTE))?;
    f.add("F-A1.bars_equal_triangles", truth(all_waits == short_area + long_area))?;
    f.add("F-A1.width_ratio", Q::new(long, short))?;
    // a triangle is as tall as its gap: the wait just after a bus
    f.add("F-A1.height_ratio", Q::new(long, short))?;
    f.add("F-A1.area_ratio", long_area / short_area)?;
    let (up, down) = model::tiling((long / short) as u64);
    f.add("F-A1.tiles.upright", up)?;
    f.add("F-A1.tiles.inverted", down)?;
    f.add("F-A1.mean_wait_ratio", long_wait / short_wait)?;
    f.add("F-A1.double", 2)?;
    f.add("F-A1.double_area_ratio", long_area / model::triangle_area(long / 2))?;

    // 01: your two mornings, just after the red bus and just before the blue
    let unlucky = model::unlucky_morning();
    let lucky = model::lucky_morning();
    f.add("F-M1.unlucky_at", unlucky)?;
    f.add("F-M1.unlucky_wait_clock", clock(model::wait_at(OFFSET, unlucky))?)?;
    f.add("F-M1.lucky_at", lucky)?;
    f.add("F-M1.lucky_wait_clock", clock(model::wait_at(OFFSET, lucky))?)?;

    // 02, 06: averaged over buses (a tag on each bus), averaged over passengers
    let tags = model::gap_before_each_bus(OFFSET);
    let passengers = model::passengers();
    let in_gap = |g: i64| passengers.iter().filter(|&&t| model::gap_at(OFFSET, t) == g).count();
    f.add("F-S1.tags_per_gap", Q::new(tags.len() as i64, gaps.len() as i64))?;
    f.add("F-S1.passengers", passengers.len())?;
    f.add("F-S1.passengers_in_long", in_gap(long))?;
    f.add("F-S1.passengers_in_short", in_gap(short))?;
    f.add("F-S1.bus_gap_tags", csv(&tags))?;
    let tagged_short = tags.iter().filter(|&&t| t == short).count() as i64;
    f.add("F-S1.bus_short_share", Q::new(tagged_short, tags.len() as i64))?;
    f.add("F-S1.bus_mean_gap", model::mean_gap(&tags))?;
    let person_short = model::time_share(&gaps, |g| g == short);
    f.add("F-S1.person_short_share", person_short)?;
    f.add("F-S1.person_long_share", Q::int(1) - person_short)?;
    let person_gap = model::passengers_gap(&gaps);
    f.pin("F-S1.person_gap", person_gap, "26/3", "⅙ × 2 + ⅚ × 10, by hand")?;
    f.add("F-S1.person_gap_clock", clock(person_gap)?)?;
    // two honest averages: over buses (the tags) and over passengers
    let averages = [model::mean_gap(&tags), person_gap];
    if averages[0] == averages[1] {
        return Err("the two averages agree, so there is no paradox to show".into());
    }
    f.add("F-S1.averages", averages.len())?;
    // given its gap, a random moment is uniform in it: on average the part of
    // the gap still ahead is half of it
    let n = Q::int(passengers.len() as i64);
    let ahead = passengers
        .iter()
        .map(|&t| model::wait_at(OFFSET, t) / Q::int(model::gap_at(OFFSET, t)))
        .sum::<Q>()
        / n;
    f.add("F-S1.wait_share_of_gap", ahead)?;
    f.add("F-S1.half_person_gap_clock", clock(person_gap / half)?)?;
    if person_gap / half != wait {
        return Err("half the passengers' gap is not the average wait".into());
    }
    let age = model::mean_since_last(&gaps);
    let by_passengers: Q = passengers.iter().map(|&t| model::since_last(OFFSET, t)).sum::<Q>() / n;
    if by_passengers != age {
        return Err("the passengers' time since the last bus disagrees with the triangles".into());
    }
    f.add("F-S1.age", age)?;
    f.add("F-S1.age_clock", clock(age)?)?;
    f.add("F-S1.age_plus_wait_clock", clock(age + wait)?)?;

    // 03: stretch one gap; the red bus from six minutes after the blue to none
    let offsets = stretch_offsets();
    let shares: Vec<Q> = offsets.iter().map(|&d| model::share_after_red(d)).collect();
    let twelfths: Vec<Q> = shares.iter().map(|&s| s * Q::int(PERIOD)).collect();
    f.add("F-G1.offsets", csv(&offsets))?;
    f.add("F-G1.passenger_shares", csv(&shares))?;
    f.add("F-G1.passengers_of_12", csv(&twelfths))?;
    f.add("F-G1.spoken_of_12", csv(&twelfths[1..5]))?;
    f.add("F-G1.share.d6", model::share_after_red(EVEN_OFFSET))?;
    f.add("F-G1.share.d2", model::share_after_red(OFFSET))?;
    f.add("F-G1.share.d0", model::share_after_red(0))?;
    f.add(
        "F-G1.even_people_per_gap",
        Q::new(HOUR * PER_MINUTE, model::hour_gaps(EVEN_OFFSET).len() as i64),
    )?;
    // the blue buses end every red-to-blue gap; with the buses together the
    // two share a moment, so that offset gives no bus share
    let mut bus_shares: Vec<Q> = offsets
        .iter()
        .filter(|&&d| d > 0)
        .map(|&d| Q::new(model::route(0).len() as i64, model::bus_times(d).len() as i64))
        .collect();
    bus_shares.sort();
    bus_shares.dedup();
    f.add("F-G1.bus_share_every_offset", csv(&bus_shares))?;
    let mut buses: Vec<usize> = offsets.iter().map(|&d| model::bus_times(d).len()).collect();
    buses.sort_unstable();
    buses.dedup();
    f.add("F-G1.buses_every_offset", csv(&buses))?;
    let mut means: Vec<Q> = offsets.iter().map(|&d| model::mean_gap(&model::hour_gaps(d))).collect();
    means.sort();
    means.dedup();
    f.add("F-G1.mean_gap_every_offset", csv(&means))?;

    // 03, 07: even gaps; slide the red bus
    let even_gaps = model::hour_gaps(EVEN_OFFSET);
    let mut even_set = even_gaps.clone();
    even_set.sort_unstable();
    even_set.dedup();
    let even_wait = model::mean_wait(&even_gaps);
    f.add("F-O1.even_offset", EVEN_OFFSET)?;
    f.add("F-O1.even_gap", csv(&even_set))?;
    f.add("F-O1.even_wait", even_wait)?;
    f.add("F-O1.even_wait_clock", clock(even_wait)?)?;
    let by_offset: Vec<Q> = (0..=PERIOD).map(model::offset_wait).collect();
    let clocks = by_offset.iter().map(|&w| clock(w)).collect::<Result<Vec<_>, _>>()?;
    if clocks[OFFSET as usize] != clock(wait)? {
        return Err("the slider at our timetable is not our average wait".into());
    }
    f.add("F-O1.wait_clocks", clocks.join(","))?;
    f.add("F-O1.wait.d0", &clocks[0])?;
    f.add("F-O1.wait.d2", &clocks[2])?;
    f.add("F-O1.wait.d6", &clocks[6])?;
    let mut together = model::bus_times(0);
    together.dedup();
    let mut together_gaps: Vec<i64> = together.windows(2).map(|w| w[1] - w[0]).collect();
    together_gaps.sort_unstable();
    together_gaps.dedup();
    f.add("F-O1.together_period", csv(&together_gaps))?;
    let min = *by_offset.iter().min().expect("offsets");
    f.add("F-O1.min_wait", min)?;
    f.add("F-O1.max_wait", *by_offset.iter().max().expect("offsets"))?;
    f.add(
        "F-O1.best_offset",
        by_offset.iter().position(|&w| w == min).expect("a minimum"),
    )?;
    // 07_5: between whole minutes the slider's readout is rounded; most
    // tenths of a minute are not whole seconds
    let tenths: Vec<Q> = (0..=10 * PERIOD)
        .map(|k| model::offset_wait_rational(Q::new(k, 10)))
        .collect::<Result<_, _>>()?;
    let whole = tenths.iter().filter(|&&w| (w * Q::int(60)).is_integer()).count();
    let between = model::offset_wait_rational(slider_example())?;
    f.add("F-O1.tenth_offsets", tenths.len())?;
    f.add("F-O1.whole_second_tenths", whole)?;
    f.add("F-O1.between_offset", slider_example())?;
    f.add("F-O1.between_wait", between)?;
    f.add("F-O1.between_readout", model::clock_rounded(between)?)?;

    // 08: the formula, with the example's numbers
    let spread = model::spread(&gaps);
    f.add("F-F1.area_factor", short_area / Q::int(short * short))?;
    f.add("F-F1.squares", csv([short * short, long * long]))?;
    f.add("F-F1.mean_square", model::mean_square(&gaps))?;
    f.add("F-F1.mean_gap_squared", mean_gap * mean_gap)?;
    f.add("F-F1.spread", spread)?;
    let mut distances: Vec<Q> = gaps
        .iter()
        .map(|&g| {
            let d = Q::int(g) - mean_gap;
            if d < Q::int(0) { Q::int(0) - d } else { d }
        })
        .collect();
    distances.sort();
    distances.dedup();
    f.add("F-F1.distance", csv(&distances))?;
    let penalty = spread / (half * mean_gap);
    f.add("F-F1.penalty", penalty)?;
    f.add("F-F1.penalty_clock", clock(penalty)?)?;
    f.add("F-F1.cv2", spread / (mean_gap * mean_gap))?;
    f.add("F-F1.wait_by_formula", mean_gap / half + penalty)?;

    // 08: never less than half the average gap; equal only when even
    let sweep = model::bound_sweep(SWEEP_MAX_GAP, SWEEP_MAX_COUNT);
    f.add("F-X1.max_gap", SWEEP_MAX_GAP)?;
    f.add("F-X1.max_count", SWEEP_MAX_COUNT)?;
    f.add("F-X1.cases", sweep.cases)?;
    f.add("F-X1.violations", sweep.violations)?;
    f.add("F-X1.equalities", sweep.equalities)?;
    f.add("F-X1.equal_iff_even", truth(sweep.equal_iff_even))?;
    f.add("F-X1.bound_factor", Q::new(1, 2))?;
    let exact = (1..=SWEEP_MAX_GAP).all(|g| model::midpoint_waits(g).into_iter().sum::<Q>() == model::triangle_area(g));
    f.add("F-X2.midpoint_exact", truth(exact))?;
    f.add("F-X2.checked", SWEEP_MAX_GAP)?;

    // 09: completely random buses (exponential gaps), and more spread still
    let (m1, m2) = model::exponential_moments(RANDOM_MEAN);
    let random_wait = m2 / (half * m1);
    f.add("F-R1.mean_gap", m1)?;
    f.add("F-R1.spread", m2 - m1 * m1)?;
    f.add("F-R1.half_mean", m1 / half)?;
    f.add("F-R1.twice_mean", half * m1)?;
    f.add("F-R1.mean_wait", random_wait)?;
    f.add("F-R1.mean_age", random_wait)?;
    f.add("F-R1.person_gap", m2 / m1)?;
    f.add("F-R1.person_gap_ratio", m2 / m1 / m1)?;
    f.add("F-R1.wait_over_mean", random_wait / m1)?;
    f.add("F-R2.gaps", csv(CLUMPED_GAPS))?;
    f.add("F-R2.mean_gap", model::mean_gap(&CLUMPED_GAPS))?;
    let clumped = model::mean_wait(&CLUMPED_GAPS);
    f.add("F-R2.mean_wait", clumped)?;
    f.add("F-R2.exceeds_random", truth(clumped > random_wait))?;

    // 10: two classes, by student; the profiler; Terada
    let students: i64 = CLASS_SIZES.iter().sum();
    let biggest = *CLASS_SIZES.iter().max().expect("classes");
    f.add("F-C1.classes", CLASS_SIZES.len())?;
    f.add("F-C1.sizes", csv(CLASS_SIZES))?;
    f.add("F-C1.school_mean", model::mean_gap(&CLASS_SIZES))?;
    f.add("F-C1.students", students)?;
    f.add("F-C1.big_class_students", biggest)?;
    f.add("F-C1.student_mean", model::passengers_gap(&CLASS_SIZES))?;
    f.pin(
        "F-C1.year",
        FELD_GROFMAN_YEAR,
        "1977",
        "Feld & Grofman, Research in Higher Education 6 (1977) 215, first page",
    )?;
    let calls_ms = SHORT_CALL_MS * SHORT_CALLS + LONG_CALL_MS;
    f.add("F-P1.long_ms", LONG_CALL_MS)?;
    f.add("F-P1.short_ms", SHORT_CALL_MS)?;
    f.add("F-P1.short_calls", SHORT_CALLS)?;
    f.add("F-P1.long_sample_share", Q::new(LONG_CALL_MS, calls_ms))?;
    f.add("F-P1.long_call_share", Q::new(1, SHORT_CALLS + 1))?;
    f.pin(
        "F-H1.year",
        TERADA_YEAR,
        "1922",
        "Masuda & Hiraoka, arXiv:2004.14081, abstract",
    )?;
    f.add("F-H1.over_a_century", truth(THIS_YEAR - TERADA_YEAR > 100))?;

    // 11: you read the timetable and turn up EARLY before your bus
    let waits_of = |ts: &[Q]| {
        let mut w: Vec<Q> = ts.iter().map(|&t| model::wait_at(OFFSET, t)).collect();
        w.sort();
        w.dedup();
        w
    };
    let reader: Vec<Q> = model::bus_times(OFFSET).iter().map(|&b| Q::int(b - EARLY)).collect();
    f.add("F-K1.early", EARLY)?;
    f.add("F-K1.wait", csv(waits_of(&reader)))?;
    // 11: every passenger plans: ready at their dot, each stands EARLY
    // before the first bus they can make with EARLY to spare
    let ready = model::passengers();
    let planned = model::planned_arrivals(OFFSET);
    let n_planned = Q::int(planned.len() as i64);
    let share_in = |g: i64| {
        let n = planned.iter().filter(|&&t| model::gap_at(OFFSET, t) == g).count();
        Q::new(n as i64, planned.len() as i64)
    };
    // how many stand for each bus of the hour (by its minute past the hour),
    // and the different stack sizes along each route
    let stack = |b: i64| {
        planned
            .iter()
            .filter(|&&t| (t + Q::int(EARLY)).num().rem_euclid(HOUR) == b)
            .count()
    };
    let stacks_of = |start: i64| {
        let mut n: Vec<usize> = model::route(start).into_iter().map(&stack).collect();
        n.sort_unstable();
        n.dedup();
        n
    };
    let mut spots: Vec<i64> = planned.iter().map(|t| t.num().rem_euclid(PERIOD)).collect();
    spots.sort_unstable();
    spots.dedup();
    let planned_wait = planned.iter().map(|&t| model::wait_at(OFFSET, t)).sum::<Q>() / n_planned;
    let planned_ahead = planned
        .iter()
        .map(|&t| model::wait_at(OFFSET, t) / Q::int(model::gap_at(OFFSET, t)))
        .sum::<Q>()
        / n_planned;
    f.add("F-K2.passengers", planned.len())?;
    f.add("F-K2.per_blue", csv(stacks_of(0)))?;
    f.add("F-K2.per_red", csv(stacks_of(OFFSET)))?;
    f.add(
        "F-K2.never_early",
        truth(planned.iter().zip(&ready).all(|(t, r)| t >= r)),
    )?;
    f.add("F-K2.waits", csv(waits_of(&planned)))?;
    f.add("F-K2.mean_wait_clock", clock(planned_wait)?)?;
    f.add("F-K2.in_long_share", share_in(long))?;
    f.add("F-K2.in_short_share", share_in(short))?;
    f.add("F-K2.stand_at", csv(spots))?;
    f.add("F-K2.wait_share_of_gap", planned_ahead)?;
    // 11_7: "Five in six would still stand in a long gap"
    if share_in(long) != long_time {
        return Err("the planners' share of long gaps is not the time share".into());
    }

    // 12: the last timetable, the red bus four minutes after the blue
    let p_gaps = model::hour_gaps(PUZZLE_OFFSET);
    let (p_short, p_long) = (model::triangle_area(p_gaps[0]), model::triangle_area(p_gaps[1]));
    let p_wait = model::offset_wait(PUZZLE_OFFSET);
    f.add("F-Q1.offset", PUZZLE_OFFSET)?;
    f.add("F-Q1.gaps", csv(&p_gaps[..2]))?;
    // 12_3: "The gap of four", "The gap of eight"
    f.add("F-Q1.short", p_gaps.iter().min().ok_or("the puzzle has no gaps")?)?;
    f.add("F-Q1.long", p_gaps.iter().max().ok_or("the puzzle has no gaps")?)?;
    f.add("F-Q1.short_area", p_short)?;
    f.add("F-Q1.long_area", p_long)?;
    f.add("F-Q1.total_area", p_short + p_long)?;
    f.add("F-Q1.mean_wait", p_wait)?;
    f.add("F-Q1.mean_wait_clock", clock(p_wait)?)?;
    f.add("F-Q1.passenger_long_share", model::share_after_red(PUZZLE_OFFSET))?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_facts_compute_and_their_pins_hold() {
        let f = facts().unwrap();
        assert_eq!(f.get("F-W1.mean_wait_clock"), Some("4:20"));
        assert_eq!(f.get("F-T1.long_time_share"), Some("5/6"));
        assert_eq!(f.get("F-S1.person_gap_clock"), Some("8:40"));
        assert_eq!(f.get("F-G1.spoken_of_12"), Some("7,8,9,10"));
        assert_eq!(f.get("F-K2.in_long_share"), Some("5/6"));
        assert_eq!(f.get("F-K2.per_blue"), Some("10"));
        assert_eq!(f.get("F-K2.per_red"), Some("2"));
        assert_eq!(f.get("F-W1.allowance_clock"), Some("10:00"));
        assert_eq!(f.get("F-O1.between_readout"), Some("4:01"));
        assert_eq!(f.get("F-Q1.mean_wait_clock"), Some("3:20"));
        assert_eq!((f.get("F-Q1.short"), f.get("F-Q1.long")), (Some("4"), Some("8")));
    }

    /// The facts print one line per key, no key twice (the oracle's format).
    #[test]
    fn every_key_prints_once() {
        let printed = facts().unwrap().print();
        let keys: Vec<&str> = printed.lines().map(|l| l.split('\t').next().unwrap()).collect();
        let mut unique = keys.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), keys.len());
    }
}
