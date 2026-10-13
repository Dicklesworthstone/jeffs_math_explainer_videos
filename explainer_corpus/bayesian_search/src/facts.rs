//! Every number Video H shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet (the source a cited number
//! comes from, or the hand arithmetic of the storyboard), and `./x
//! bayesian_search facts-check` against `oracle.py` (same keys). The model
//! works the four-cell sea by the update formula; the oracle counts dots.
//! The 1968 map is `model::y1968::POINTS`; the oracle carries its own
//! transcription of the printed figure.

use explainer_kit::Facts;

use crate::model::{Ask, Q, QUIZZES, days, drift, koopman, q, sea, y1968};

/// Numbers quoted from the sources (fact sheet §Sources gives the passage
/// and page for each; the oracle types the same literals a second time).
pub mod cited {
    /// The USS Scorpion (SSN-589). Naval History and Heritage Command,
    /// DANFS "Scorpion VI (SSN-589)": last message 21 May 1968, reported
    /// overdue 27 May, "presumed lost" 5 June. Navy report NT-24-1 (2024),
    /// p. 7: sank 22 May 1968, "400 miles southwest of the Azores in more
    /// than 10,000 feet of water".
    pub const CREW: u32 = 99;
    pub const LAST_MESSAGE: (i64, i64, i64) = (1968, 5, 21);
    pub const SANK: (i64, i64, i64) = (1968, 5, 22);
    pub const DUE: (i64, i64, i64) = (1968, 5, 27);
    pub const PRESUMED_LOST: (i64, i64, i64) = (1968, 6, 5);
    pub const MILES_FROM_AZORES: u32 = 400;
    pub const DEPTH_FEET: u32 = 10_000;

    /// Richardson and Stone (1971). Mizar's first cruise began 10 June
    /// (p. 142); nine scenarios and their credences on 18 July (p. 143);
    /// 10,000 replications (p. 145); the map reached the ship on 12 August
    /// (p. 149); located 28 October (p. 142); "within 260 yards of the
    /// edge of rectangle E5" (p. 145); 1,026 miles of track, "about 43
    /// days of bottom time at the speed of 1 knot" (p. 149); the forecast
    /// of 1 August, "between 35 and 45 days" (p. 149); 153 miles to reach
    /// F6 under the best allocation with an 800-foot sweep width (p. 146);
    /// five cruises (p. 142).
    pub const SEARCH_BEGAN: (i64, i64, i64) = (1968, 6, 10);
    pub const SCENARIOS_SET: (i64, i64, i64) = (1968, 7, 18);
    pub const SCENARIOS: u32 = 9;
    pub const MAP_ARRIVED: (i64, i64, i64) = (1968, 8, 12);
    pub const FOUND: (i64, i64, i64) = (1968, 10, 28);
    pub const YARDS_FROM_TOP_CELL: u32 = 260;
    pub const TRACK_MILES: u32 = 1026;
    pub const BOTTOM_DAYS: u32 = 43;
    pub const FORECAST_DAYS: (u32, u32) = (35, 45);
    pub const MILES_TO_REACH_F6: u32 = 153;
    pub const CRUISES: u32 = 5;
    /// Sweep widths in feet. Assumed at sea: "typically 800 feet and 400
    /// feet for sonar and magnetometer" (p. 151). Estimated afterwards
    /// against the wreck as found (Table 2): camera 185, magnetometer 220,
    /// sonar 0.
    pub const SWEEP_AT_SEA: (u32, u32) = (800, 400);
    pub const SWEEP_AFTER: (u32, u32, u32) = (185, 220, 0);
    /// How thoroughly F6 had been searched (local effectiveness
    /// probability), in hundredths, on 10 September, 9 October and 28
    /// October: computed on scene (Table 1) and estimated afterwards
    /// (Table 3).
    pub const F6_SEARCHED_AT_SEA: [u32; 3] = [65, 94, 99];
    pub const F6_SEARCHED_AFTER: [u32; 3] = [47, 65, 74];

    /// Palomares. Defense Nuclear Agency, "Palomares Summary Report"
    /// (1975): the collision on 17 January 1966 (p. 15); four weapons;
    /// eleven men, seven killed (pp. 16-17); Alvin found the fourth weapon
    /// on 15 March at 2,550 feet (p. 94); recovered 7 April.
    pub const PALOMARES: (i64, i64, i64) = (1966, 1, 17);
    pub const PALOMARES_BOMBS: u32 = 4;
    pub const PALOMARES_ON_LAND: u32 = 3;
    pub const PALOMARES_AIRMEN: (u32, u32) = (11, 7);
    pub const PALOMARES_FOUND: (i64, i64, i64) = (1966, 3, 15);
    pub const PALOMARES_RECOVERED: (i64, i64, i64) = (1966, 4, 7);
    pub const PALOMARES_DEPTH_FEET: u32 = 2550;

    /// Air France 447. BEA final report (2012): 1 June 2009, 216
    /// passengers and 12 crew; the wreckage "found on 2 April 2011, 22
    /// months after", "about 6.5 NM" from the last known position at
    /// 3,900 metres. Stone, Keller, Kratzke and Strumpfer (2014): the
    /// 40 NM circle; weights 0.35, 0.35, 0.3 (p. 72); 75,000 points
    /// (p. 73); no sensor credited above 0.9, a beacon surviving with
    /// 0.8, the weighted average of 0.25 and 0.75 giving 0.77 (pp. 76-77);
    /// found "with one week of undersea search" (p. 69).
    pub const AF447_LOST: (i64, i64, i64) = (2009, 6, 1);
    pub const AF447_FOUND: (i64, i64, i64) = (2011, 4, 2);
    pub const AF447_PASSENGERS: u32 = 216;
    pub const AF447_CREW: u32 = 12;
    pub const AF447_CIRCLE_NM: u32 = 40;
    /// In hundredths.
    pub const AF447_WEIGHTS: [u32; 3] = [35, 35, 30];
    pub const AF447_POINTS: u32 = 75_000;
    pub const AF447_SENSOR_CAP: u32 = 90;
    pub const AF447_BEACON_SURVIVES: u32 = 80;
    pub const AF447_INDEPENDENT_WEIGHT: u32 = 25;
    pub const AF447_BEACON_SEARCH: u32 = 77;
    /// In tenths of a nautical mile.
    pub const AF447_FOUND_TENTHS_NM: u32 = 65;
    pub const AF447_DEPTH_M: u32 = 3900;
    pub const AF447_MONTHS: u32 = 22;
    pub const AF447_EMPTY_SEARCHES: u32 = 4;

    /// The US Coast Guard. Kratzke, Stone and Frost (2010): CASP in
    /// service 1974, SAROPS "operational since January, 2007"; 2,500,
    /// 5,000 or 10,000 particles per scenario. Coast Guard Addendum
    /// (COMDTINST M16130.2F), Table H-15: a helicopter's sweep width for
    /// a person in the water, 0.1 NM; section 3.1.1: about 90 percent of
    /// cases involve no searching.
    pub const CASP_YEAR: u32 = 1974;
    pub const SAROPS_YEAR: u32 = 2007;
    pub const SAROPS_DOTS: [u32; 3] = [2500, 5000, 10_000];
    pub const NO_SEARCH_PERCENT: u32 = 90;
    /// John Aldridge, 24 July 2013. Paul Tough, "A Speck in the Sea", The
    /// New York Times Magazine, January 2014: overboard "about 3:30
    /// a.m."; the first map assumed 9:30 to 11:30 the evening before;
    /// "Almost 30 miles south"; the map re-run at 8:30 a.m.; the last
    /// track begun at 2:46 p.m. and the sighting "twelve minutes later".
    /// Coast Guard release, 24 July 2013: "approximately 43 miles south
    /// of Montauk"; the Times has 40.
    pub const ALDRIDGE: (i64, i64, i64) = (2013, 7, 24);
    /// Minutes after midnight.
    pub const ALDRIDGE_FELL: u32 = 3 * 60 + 30;
    pub const ALDRIDGE_LAST_TRACK: u32 = 14 * 60 + 46;
    pub const ALDRIDGE_MINUTES_TO_SIGHTING: u32 = 12;
    pub const ALDRIDGE_RERUN: u32 = 8 * 60 + 30;
    pub const ALDRIDGE_MILES_SOUTH: u32 = 30;
    pub const ALDRIDGE_MILES_OFFSHORE: (u32, u32) = (40, 43);

    /// The names behind the method. Koopman, "Search and Screening", OEG
    /// Report 56 (1946), the random search formula on p. 28. Bayes died
    /// in 1761 and his essay was read to the Royal Society on 23 December
    /// 1763; Laplace's memoir is of 1774. Chew, Annals of Mathematical
    /// Statistics 38 (1967), Theorem 2.1.
    pub const KOOPMAN_YEAR: u32 = 1946;
    pub const BAYES_DIED: u32 = 1761;
    pub const BAYES_READ: u32 = 1763;
    pub const LAPLACE_YEAR: u32 = 1774;
    pub const CHEW_YEAR: u32 = 1967;
    pub const PAPER_YEAR: u32 = 1971;
}

/// The video's own choices (model numbers; each says "model" on screen).
pub mod shown {
    use crate::model::Q;

    /// Searches of the four-cell sea listed for each rule.
    pub const RUN: usize = 12;
    /// Orders tried one by one for "no order does better".
    pub const BRUTE: usize = 8;
    /// The model search over the 1968 map: every search sees a wreck in
    /// its cell two times in three, as in the flat cells of the small sea.
    pub const MAP_SEEING: Q = Q::lowest(2, 3);
    /// Searches at which the model search and the sweep are compared.
    pub const MAP_MARKS: [usize; 4] = [10, 20, 35, 81];
    /// The strip the hook's arithmetic gives the towed sled, in feet:
    /// between the camera's 185 and the magnetometer's 220.
    pub const STRIP_FEET: u32 = 200;
    /// The largest square the drift chapter tries, in miles.
    pub const MAX_SIDE: u32 = 8;
}

/// 0.3048 metres in a foot, exactly.
const METRES_PER_FOOT: Q = Q::lowest(381, 1250);
/// Metres in a nautical mile, exactly.
const METRES_PER_NM: i128 = 1852;

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn percents(items: impl IntoIterator<Item = Q>, places: u32) -> String {
    list(items.into_iter().map(|x| x.percent(places)))
}

fn date(d: (i64, i64, i64)) -> String {
    format!("{:04}-{:02}-{:02}", d.0, d.1, d.2)
}

fn clock(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    small_sea(&mut f)?;
    rules(&mut f)?;
    map_1968(&mut f)?;
    hook(&mut f)?;
    formula(&mut f)?;
    cloud(&mut f)?;
    scorpion(&mut f)?;
    palomares(&mut f)?;
    air_france(&mut f)?;
    coast_guard(&mut f)?;
    names(&mut f)?;
    quizzes(&mut f)?;
    Ok(f)
}

/// F-M: the four-cell sea, from the two stories to the second empty search.
fn small_sea(f: &mut Facts) -> Result<(), String> {
    let hand = "the storyboard's hand arithmetic";
    f.add("F-M.dots", sea::DOTS)?;
    for (i, s) in sea::stories().iter().enumerate() {
        let key = format!("F-M.story{}", i + 1);
        f.add(&format!("{key}.weight"), s.weight.percent(0))?;
        f.add(&format!("{key}.share"), list(s.share))?;
        f.add(&format!("{key}.dots"), list(sea::story_dots(s)))?;
    }
    let p = sea::prior();
    f.pin(
        "F-M.prior.dots",
        list(p.map(|x| x * Q::int(sea::DOTS))),
        "40,30,20,10",
        hand,
    )?;
    f.add("F-M.prior.percent", percents(p, 0))?;
    f.pin("F-M.seeing", list(sea::seeing()), "1/4,2/3,2/3,2/3", "the brief")?;

    let find = sea::find(&p);
    f.add("F-M.find.fraction", list(find))?;
    f.pin("F-M.find.percent", percents(find, 1), "10.0,20.0,13.3,6.7", hand)?;
    let run = sea::run(sea::Rule::Best, 3);
    let likeliest = (0..4).max_by_key(|&i| (p[i], std::cmp::Reverse(i))).ok_or("no cells")?;
    f.pin("F-M.likeliest.cell", sea::CELLS[likeliest], "A", hand)?;
    f.add("F-M.likeliest.find.percent", find[likeliest].percent(0))?;
    f.pin("F-M.first.cell", sea::CELLS[run[0].cell], "B", hand)?;
    f.add("F-M.first.find.percent", run[0].find.percent(0))?;
    f.add("F-M.first.over_likeliest", run[0].find / find[likeliest])?;

    // the empty search of B, in dots
    let b = run[0].cell;
    let out = p[b] * sea::seeing()[b] * Q::int(sea::DOTS);
    let left = Q::int(sea::DOTS) - out;
    f.pin("F-M.empty1.dots_out", out, "20", hand)?;
    f.pin("F-M.empty1.dots_left", left, "80", hand)?;
    f.add("F-M.empty1.cell_dots_left", p[b] * Q::int(sea::DOTS) - out)?;
    f.pin("F-M.empty1.after.fraction", list(run[0].after), "1/2,1/8,1/4,1/8", hand)?;
    f.pin(
        "F-M.empty1.after.percent",
        percents(run[0].after, 1),
        "50.0,12.5,25.0,12.5",
        hand,
    )?;
    f.add("F-M.empty1.rise", run[0].after[0] / p[0])?;
    f.add("F-M.empty1.rise.decimal", (run[0].after[0] / p[0]).fixed(2))?;

    let second = sea::find(&run[0].after);
    f.add("F-M.second.find.fraction", list(second))?;
    f.pin(
        "F-M.second.find.percent",
        percents(second, 1),
        "12.5,8.3,16.7,8.3",
        hand,
    )?;
    f.pin("F-M.second.cell", sea::CELLS[run[1].cell], "C", hand)?;
    f.pin(
        "F-M.empty2.after.percent",
        percents(run[1].after, 0),
        "60,15,10,15",
        hand,
    )?;
    f.add("F-M.empty2.found_by", run[1].found_by)?;
    f.add("F-M.third.find.percent", percents(sea::find(&run[1].after), 1))?;
    f.pin("F-M.third.cell", sea::CELLS[run[2].cell], "A", hand)?;

    let each = sea::seeing()[1];
    for n in 1..=3u32 {
        let seen = sea::seen_within(each, n);
        f.add(&format!("F-M.seen_within.{n}"), seen)?;
        f.add(&format!("F-M.seen_within.{n}.percent"), seen.percent(1))?;
    }
    Ok(())
}

/// F-U: the four rules for where to search next, and "no order does better".
fn rules(f: &mut Facts) -> Result<(), String> {
    for (name, rule) in [
        ("best", sea::Rule::Best),
        ("likeliest", sea::Rule::Likeliest),
        ("likeliest_fresh", sea::Rule::LikeliestFresh),
        ("sweep", sea::Rule::Sweep),
    ] {
        let run = sea::run(rule, shown::RUN);
        f.add(
            &format!("F-U.{name}.order"),
            run.iter().map(|s| sea::CELLS[s.cell]).collect::<String>(),
        )?;
        f.add(
            &format!("F-U.{name}.found_by.percent"),
            percents(run.iter().map(|s| s.found_by), 1),
        )?;
        f.add(
            &format!("F-U.{name}.expected_searches"),
            format!("{:.2}", sea::expected_searches(rule)),
        )?;
    }
    let run = sea::run(sea::Rule::Best, shown::BRUTE);
    let brute: Vec<Q> = (1..=shown::BRUTE).map(sea::best_of_all_orders).collect();
    f.add("F-U.brute.searches", shown::BRUTE)?;
    f.add("F-U.brute.orders", 4u64.pow(shown::BRUTE as u32))?;
    f.add("F-U.brute.best", list(brute.iter()))?;
    f.pin(
        "F-U.brute.rule_is_best",
        run.iter().zip(&brute).all(|(s, b)| s.found_by == *b),
        "true",
        "Chew (1967), Theorem 2.1",
    )?;
    Ok(())
}

/// F-R: the 1968 map as printed, and the model search over it.
fn map_1968(f: &mut Facts) -> Result<(), String> {
    let paper = "Richardson and Stone (1971), Figure 2 and p. 145";
    let share = |n: u32| q(i128::from(n), i128::from(y1968::REPLICATIONS));
    let points = |name: &str| y1968::points(name).ok_or(format!("no cell {name}"));
    f.pin("F-R.replications", y1968::REPLICATIONS, "10000", paper)?;
    f.pin("F-R.total", y1968::total(), "10000", paper)?;
    f.add("F-R.printed_cells", y1968::printed())?;
    f.add(
        "F-R.rows",
        list(
            y1968::POINTS
                .iter()
                .map(|r| r.iter().map(|&n| u32::from(n)).sum::<u32>()),
        ),
    )?;
    f.add(
        "F-R.columns",
        list((0..y1968::COLUMNS.len()).map(|c| y1968::POINTS.iter().map(|r| u32::from(r[c])).sum::<u32>())),
    )?;
    f.add("F-R.shading", list(y1968::shading()))?;
    let (e5, b7, f6) = (points("E5")?, points("B7")?, points("F6")?);
    f.pin("F-R.E5", e5, "1250", paper)?;
    f.add("F-R.E5.percent", share(e5).percent(1))?;
    f.pin("F-R.B7", b7, "1096", paper)?;
    f.pin(
        "F-R.top_two.percent",
        share(e5 + b7).percent(0),
        "23",
        "p. 145: E5 and B7 \"have 23 percent\"",
    )?;
    f.pin("F-R.F6", f6, "85", paper)?;
    f.add("F-R.F6.percent", share(f6).percent(2))?;
    f.add("F-R.F6.rank", y1968::rank("F6").ok_or("no F6")?)?;
    f.add("F-R.E5.rank", y1968::rank("E5").ok_or("no E5")?)?;
    f.add("F-R.E5_over_F6", q(i128::from(e5), i128::from(f6)).fixed(1))?;
    f.add("F-R.grid.cells", y1968::GRID_ROWS.len() * y1968::GRID_COLS.len())?;
    f.pin(
        "F-R.cell.miles",
        format!("{}x{}", y1968::CELL_MILES.0.fixed(0), y1968::CELL_MILES.1.fixed(2)),
        "1x0.84",
        "p. 143",
    )?;
    f.add("F-R.grid.area", y1968::grid_area().fixed(1))?;
    f.add("F-R.grid.points", y1968::in_grid())?;
    f.add("F-R.outside.points", y1968::total() - y1968::in_grid())?;

    // the model search (the video's own model on the Navy's map)
    let seeing = shown::MAP_SEEING;
    f.add("F-R.model.seeing", seeing)?;
    let limit = 400;
    let best = y1968::model_search(y1968::Rule::Best, seeing, limit);
    let sweep = y1968::model_search(y1968::Rule::Sweep, seeing, limit);
    f.add(
        "F-R.model.best.first_cells",
        list(best[..6].iter().map(|l| y1968::name(l.row, l.col))),
    )?;
    let f6_best = y1968::first_look_at("F6", y1968::Rule::Best, seeing, limit).ok_or("the model never reaches F6")?;
    let f6_sweep = y1968::first_look_at("F6", y1968::Rule::Sweep, seeing, limit).ok_or("the sweep never reaches F6")?;
    f.add("F-R.model.best.F6_search", f6_best)?;
    f.add("F-R.model.sweep.F6_search", f6_sweep)?;
    for mark in shown::MAP_MARKS {
        f.add(
            &format!("F-R.model.best.found_by.{mark}"),
            best[mark - 1].found_by.percent(1),
        )?;
        f.add(
            &format!("F-R.model.sweep.found_by.{mark}"),
            sweep[mark - 1].found_by.percent(1),
        )?;
    }
    Ok(())
}

/// F-H: "search everywhere" on the 1968 grid.
fn hook(f: &mut Facts) -> Result<(), String> {
    // the paper's miles are sea miles: 1,026 of them at 1 knot are its 43 days
    let days_at_one_knot = q(i128::from(cited::TRACK_MILES), 24);
    f.pin(
        "F-H.track.miles",
        cited::TRACK_MILES,
        "1026",
        "Richardson and Stone (1971), p. 149",
    )?;
    f.pin(
        "F-H.track.days_at_one_knot",
        days_at_one_knot.fixed(0),
        &cited::BOTTOM_DAYS.to_string(),
        "p. 149: \"about 43 days of bottom time at the speed of 1 knot\"",
    )?;
    f.add("F-H.strip.feet", shown::STRIP_FEET)?;
    let feet_per_nm = Q::int(METRES_PER_NM) / METRES_PER_FOOT;
    f.add("F-H.feet_per_sea_mile", feet_per_nm.fixed(0))?;
    let strip_miles = Q::int(i128::from(shown::STRIP_FEET)) / feet_per_nm;
    let once = y1968::grid_area() / strip_miles;
    f.add("F-H.once.miles", once.fixed(0))?;
    f.add("F-H.once.hundreds", (once / Q::int(100)).fixed(0))?;
    f.add("F-H.once.days_at_one_knot", (once / Q::int(24)).fixed(0))?;
    f.add(
        "F-H.once.over_track",
        (once / Q::int(i128::from(cited::TRACK_MILES))).fixed(1),
    )?;
    for (name, feet) in [("camera", cited::SWEEP_AFTER.0), ("magnetometer", cited::SWEEP_AFTER.1)] {
        let miles = y1968::grid_area() / (Q::int(i128::from(feet)) / feet_per_nm);
        f.add(&format!("F-H.once.miles.{name}"), miles.fixed(0))?;
    }
    Ok(())
}

/// F-K: the random search formula, and lanes laid without regard to what
/// is already swept.
fn formula(f: &mut Facts) -> Result<(), String> {
    for (name, coverage) in [("half", 0.5), ("1", 1.0), ("2", 2.0), ("3", 3.0)] {
        f.add(
            &format!("F-K.formula.{name}"),
            format!("{:.1}", 100.0 * koopman::random_search(coverage)),
        )?;
        f.add(
            &format!("F-K.perfect.{name}"),
            format!("{:.0}", 100.0 * koopman::perfect_lanes(coverage)),
        )?;
    }
    f.add(
        "F-K.formula.1.unseen",
        format!("{:.1}", 100.0 * (1.0 - koopman::random_search(1.0))),
    )?;
    let seeing = sea::seeing();
    f.add(
        "F-K.coverage.flat",
        format!("{:.2}", koopman::coverage_for(seeing[1].f64())),
    )?;
    f.add(
        "F-K.coverage.canyon",
        format!("{:.2}", koopman::coverage_for(seeing[0].f64())),
    )?;

    f.add("F-K.lanes.seed", koopman::SEED)?;
    f.add("F-K.lanes.side", koopman::SIDE)?;
    f.add("F-K.lanes.share.percent", koopman::lane_share().percent(0))?;
    f.add("F-K.lanes.per_coverage", koopman::LANES_PER_COVERAGE)?;
    f.add("F-K.lanes.runs", koopman::RUNS)?;
    let n = 3 * koopman::LANES_PER_COVERAGE;
    let one = koopman::swept_after(&koopman::the_run(n));
    let sums = koopman::swept_sums(n);
    let points = (koopman::SIDE * koopman::SIDE) as i128;
    for coverage in 1..=3usize {
        let lanes = coverage * koopman::LANES_PER_COVERAGE;
        f.add(
            &format!("F-K.lanes.one_run.{coverage}"),
            q(one[lanes - 1] as i128, points).percent(1),
        )?;
        f.add(
            &format!("F-K.lanes.mean.{coverage}"),
            q(i128::from(sums[lanes - 1]), points * koopman::RUNS as i128).percent(1),
        )?;
        f.add(
            &format!("F-K.lanes.exact.{coverage}"),
            format!("{:.1}", 100.0 * koopman::expected_share(lanes as u32)),
        )?;
    }
    f.add(
        "F-K.lanes.first",
        list(
            koopman::the_run(4)
                .iter()
                .map(|l| format!("{}{}", if l.vertical { 'v' } else { 'h' }, l.at)),
        ),
    )?;
    Ok(())
}

/// F-D: the drifting cloud (model), and one flight over it.
fn cloud(f: &mut Facts) -> Result<(), String> {
    f.add("F-D.seed", drift::SEED)?;
    f.add("F-D.dots", drift::DOTS)?;
    f.add("F-D.start_radius", drift::START_RADIUS)?;
    f.add("F-D.drift", drift::DRIFT.fixed(1))?;
    f.add("F-D.error_share", drift::ERROR_SHARE.fixed(1))?;
    f.add("F-D.track", drift::TRACK)?;
    f.pin(
        "F-D.strip",
        drift::STRIP.fixed(1),
        "0.1",
        "US Coast Guard Addendum (COMDTINST M16130.2F), Table H-15",
    )?;
    f.add("F-D.strip.metres", (drift::STRIP * Q::int(METRES_PER_NM)).fixed(0))?;
    f.add("F-D.swept", drift::swept())?;
    let dots = drift::cloud();
    for h in drift::hours() {
        let key = format!("F-D.h{h}");
        f.add(&format!("{key}.radius"), drift::radius(h).fixed(2))?;
        f.add(&format!("{key}.area"), format!("{:.1}", drift::area(h)))?;
        f.add(&format!("{key}.coverage"), format!("{:.2}", drift::coverage(h)))?;
        f.add(&format!("{key}.seeing"), format!("{:.0}", 100.0 * drift::seeing(h)))?;
        f.add(
            &format!("{key}.farthest"),
            format!("{:.2}", drift::farthest(&dots, h.f64())),
        )?;
    }
    for h in [3u32, 6] {
        let hours = f64::from(h);
        let side = drift::best_side(&dots, hours, shown::MAX_SIDE);
        let best = drift::flight(&dots, hours, side);
        let key = format!("F-D.flight.h{h}");
        f.add(&format!("{key}.best_side"), side)?;
        f.add(&format!("{key}.inside"), best.inside)?;
        f.add(&format!("{key}.seeing"), format!("{:.0}", 100.0 * best.seeing))?;
        f.add(&format!("{key}.find"), format!("{:.0}", 100.0 * best.find))?;
        f.add(
            &format!("{key}.find_by_side"),
            list((1..=shown::MAX_SIDE).map(|s| format!("{:.0}", 100.0 * drift::flight(&dots, hours, s).find))),
        )?;
    }
    // twice the radius: four times the area, a quarter of the coverage
    let r = drift::radius(Q::ONE);
    let twice = r * Q::int(2);
    f.add("F-D.double.area_factor", (twice * twice) / (r * r))?;
    f.add("F-D.double.coverage_factor", (r * r) / (twice * twice))?;
    Ok(())
}

/// F-S: the Scorpion, as the sources give it.
fn scorpion(f: &mut Facts) -> Result<(), String> {
    let navy = "Naval History and Heritage Command, DANFS \"Scorpion VI (SSN-589)\"";
    let paper = "Richardson and Stone (1971)";
    f.pin("F-S.crew", cited::CREW, "99", navy)?;
    f.pin("F-S.last_message", date(cited::LAST_MESSAGE), "1968-05-21", navy)?;
    f.pin(
        "F-S.sank",
        date(cited::SANK),
        "1968-05-22",
        "Navy report NT-24-1 (2024), p. 7",
    )?;
    f.pin("F-S.due", date(cited::DUE), "1968-05-27", navy)?;
    f.pin("F-S.presumed_lost", date(cited::PRESUMED_LOST), "1968-06-05", navy)?;
    f.pin("F-S.found", date(cited::FOUND), "1968-10-28", paper)?;
    f.add("F-S.days.sank_to_due", days::between(cited::SANK, cited::DUE))?;
    f.add(
        "F-S.days.due_to_presumed",
        days::between(cited::DUE, cited::PRESUMED_LOST),
    )?;
    f.add("F-S.days.due_to_found", days::between(cited::DUE, cited::FOUND))?;
    f.add(
        "F-S.months.due_to_found",
        days::months_between(cited::DUE, cited::FOUND),
    )?;
    f.pin("F-S.search_began", date(cited::SEARCH_BEGAN), "1968-06-10", paper)?;
    f.pin("F-S.scenarios_set", date(cited::SCENARIOS_SET), "1968-07-18", paper)?;
    f.pin("F-S.scenarios", cited::SCENARIOS, "9", paper)?;
    f.pin("F-S.map_arrived", date(cited::MAP_ARRIVED), "1968-08-12", paper)?;
    f.add(
        "F-S.days.search_to_map",
        days::between(cited::SEARCH_BEGAN, cited::MAP_ARRIVED),
    )?;
    f.add(
        "F-S.months.search_to_map",
        days::months_between(cited::SEARCH_BEGAN, cited::MAP_ARRIVED),
    )?;
    f.add("F-S.days.map_to_found", days::between(cited::MAP_ARRIVED, cited::FOUND))?;
    f.pin("F-S.cruises", cited::CRUISES, "5", paper)?;
    f.pin("F-S.yards", cited::YARDS_FROM_TOP_CELL, "260", paper)?;
    f.add(
        "F-S.yards.metres",
        (Q::int(i128::from(cited::YARDS_FROM_TOP_CELL) * 3) * METRES_PER_FOOT).fixed(0),
    )?;
    f.pin("F-S.miles_from_azores", cited::MILES_FROM_AZORES, "400", navy)?;
    f.pin("F-S.depth.feet", cited::DEPTH_FEET, "10000", navy)?;
    f.add(
        "F-S.depth.metres",
        (Q::int(i128::from(cited::DEPTH_FEET)) * METRES_PER_FOOT).fixed(0),
    )?;
    f.add("F-S.depth.miles", q(i128::from(cited::DEPTH_FEET), 5280).fixed(1))?;
    f.pin(
        "F-S.forecast.days",
        format!("{}-{}", cited::FORECAST_DAYS.0, cited::FORECAST_DAYS.1),
        "35-45",
        paper,
    )?;
    f.pin("F-S.bottom_days", cited::BOTTOM_DAYS, "43", paper)?;
    f.pin("F-S.miles_to_reach_F6", cited::MILES_TO_REACH_F6, "153", paper)?;

    // how wide the sensors saw: assumed at sea, and estimated afterwards
    f.pin(
        "F-S.sweep.at_sea",
        list([cited::SWEEP_AT_SEA.0, cited::SWEEP_AT_SEA.1]),
        "800,400",
        paper,
    )?;
    f.pin(
        "F-S.sweep.after",
        list([cited::SWEEP_AFTER.0, cited::SWEEP_AFTER.1, cited::SWEEP_AFTER.2]),
        "185,220,0",
        paper,
    )?;
    f.add(
        "F-S.sweep.magnetometer.ratio",
        q(i128::from(cited::SWEEP_AT_SEA.1), i128::from(cited::SWEEP_AFTER.1)).fixed(1),
    )?;
    f.pin(
        "F-S.F6.searched.at_sea",
        list(cited::F6_SEARCHED_AT_SEA),
        "65,94,99",
        paper,
    )?;
    f.pin(
        "F-S.F6.searched.after",
        list(cited::F6_SEARCHED_AFTER),
        "47,65,74",
        paper,
    )?;
    // what an empty search leaves of a cell's chance before the map is rescaled: 1 − searched
    let left = |hundredths: u32| q(i128::from(100 - hundredths), 100);
    let (sea_oct, after_oct) = (left(cited::F6_SEARCHED_AT_SEA[1]), left(cited::F6_SEARCHED_AFTER[1]));
    f.add("F-S.F6.left.at_sea.percent", sea_oct.percent(0))?;
    f.add("F-S.F6.left.after.percent", after_oct.percent(0))?;
    f.add("F-S.F6.left.ratio", (after_oct / sea_oct).fixed(1))?;
    Ok(())
}

/// F-P: Palomares.
fn palomares(f: &mut Facts) -> Result<(), String> {
    let report = "Defense Nuclear Agency, Palomares Summary Report (1975)";
    f.pin("F-P.date", date(cited::PALOMARES), "1966-01-17", report)?;
    f.pin("F-P.bombs", cited::PALOMARES_BOMBS, "4", report)?;
    f.pin("F-P.on_land", cited::PALOMARES_ON_LAND, "3", report)?;
    f.add("F-P.in_sea", cited::PALOMARES_BOMBS - cited::PALOMARES_ON_LAND)?;
    f.pin(
        "F-P.airmen",
        format!("{} of {}", cited::PALOMARES_AIRMEN.1, cited::PALOMARES_AIRMEN.0),
        "7 of 11",
        report,
    )?;
    f.pin("F-P.found", date(cited::PALOMARES_FOUND), "1966-03-15", report)?;
    f.pin("F-P.recovered", date(cited::PALOMARES_RECOVERED), "1966-04-07", report)?;
    f.add(
        "F-P.days.to_found",
        days::between(cited::PALOMARES, cited::PALOMARES_FOUND),
    )?;
    f.add(
        "F-P.days.to_recovered",
        days::between(cited::PALOMARES, cited::PALOMARES_RECOVERED),
    )?;
    f.pin("F-P.depth.feet", cited::PALOMARES_DEPTH_FEET, "2550", report)?;
    f.add(
        "F-P.depth.metres",
        (Q::int(i128::from(cited::PALOMARES_DEPTH_FEET)) * METRES_PER_FOOT).fixed(0),
    )?;
    Ok(())
}

/// F-A: Air France 447.
fn air_france(f: &mut Facts) -> Result<(), String> {
    let bea = "BEA final report on AF 447 (2012)";
    let paper = "Stone, Keller, Kratzke and Strumpfer (2014)";
    f.pin("F-A.lost", date(cited::AF447_LOST), "2009-06-01", bea)?;
    f.pin("F-A.found", date(cited::AF447_FOUND), "2011-04-02", bea)?;
    f.pin("F-A.aboard", cited::AF447_PASSENGERS + cited::AF447_CREW, "228", bea)?;
    f.pin(
        "F-A.months",
        days::months_between(cited::AF447_LOST, cited::AF447_FOUND),
        &cited::AF447_MONTHS.to_string(),
        "BEA final report: \"22 months after\"",
    )?;
    f.add("F-A.days", days::between(cited::AF447_LOST, cited::AF447_FOUND))?;
    f.pin("F-A.circle.nm", cited::AF447_CIRCLE_NM, "40", paper)?;
    let radius_km = q(i128::from(cited::AF447_CIRCLE_NM) * METRES_PER_NM, 1000);
    f.add("F-A.circle.km", radius_km.fixed(0))?;
    f.add(
        "F-A.circle.area_km2",
        format!("{:.0}", std::f64::consts::PI * radius_km.f64().powi(2)),
    )?;
    f.pin("F-A.weights", list(cited::AF447_WEIGHTS), "35,35,30", paper)?;
    f.add("F-A.weights.sum", cited::AF447_WEIGHTS.iter().sum::<u32>())?;
    f.pin("F-A.points", cited::AF447_POINTS, "75000", paper)?;
    f.pin("F-A.empty_searches", cited::AF447_EMPTY_SEARCHES, "4", paper)?;

    // the beacon search, as the paper works it (pp. 76-77)
    let hundredths = |n: u32| q(i128::from(n), 100);
    let (cap, survives) = (
        hundredths(cited::AF447_SENSOR_CAP),
        hundredths(cited::AF447_BEACON_SURVIVES),
    );
    let miss = Q::ONE - cap;
    let both = survives * survives;
    let one = Q::int(2) * survives * (Q::ONE - survives);
    let independent = (Q::ONE - miss * miss) * both + cap * one;
    let dependent = cap * survives;
    f.add("F-A.beacon.independent", independent.fixed(4))?;
    f.pin("F-A.beacon.independent.rounded", independent.fixed(2), "0.92", paper)?;
    f.pin("F-A.beacon.dependent", dependent.fixed(2), "0.72", paper)?;
    let w = hundredths(cited::AF447_INDEPENDENT_WEIGHT);
    // the paper averages its two rounded figures
    let used = w * q(92, 100) + (Q::ONE - w) * dependent;
    f.pin(
        "F-A.beacon.used.percent",
        used.percent(0),
        &cited::AF447_BEACON_SEARCH.to_string(),
        paper,
    )?;
    f.add("F-A.beacon.left.percent", (Q::ONE - used).percent(0))?;
    f.pin(
        "F-A.found.nm",
        q(i128::from(cited::AF447_FOUND_TENTHS_NM), 10).fixed(1),
        "6.5",
        bea,
    )?;
    f.add(
        "F-A.found.km",
        q(i128::from(cited::AF447_FOUND_TENTHS_NM) * METRES_PER_NM, 10_000).fixed(0),
    )?;
    f.add(
        "F-A.found.share_of_radius",
        q(
            i128::from(cited::AF447_FOUND_TENTHS_NM),
            10 * i128::from(cited::AF447_CIRCLE_NM),
        )
        .percent(0),
    )?;
    f.pin("F-A.depth.metres", cited::AF447_DEPTH_M, "3900", bea)?;
    Ok(())
}

/// F-C: the Coast Guard.
fn coast_guard(f: &mut Facts) -> Result<(), String> {
    let paper = "Kratzke, Stone and Frost (2010)";
    let times = "Paul Tough, \"A Speck in the Sea\", The New York Times Magazine (2014)";
    f.pin("F-C.casp", cited::CASP_YEAR, "1974", paper)?;
    f.pin("F-C.sarops", cited::SAROPS_YEAR, "2007", paper)?;
    f.pin("F-C.sarops.dots", list(cited::SAROPS_DOTS), "2500,5000,10000", paper)?;
    f.pin(
        "F-C.no_search.percent",
        cited::NO_SEARCH_PERCENT,
        "90",
        "US Coast Guard Addendum (COMDTINST M16130.2F), section 3.1.1",
    )?;
    f.pin("F-C.aldridge.date", date(cited::ALDRIDGE), "2013-07-24", times)?;
    f.pin("F-C.aldridge.fell", clock(cited::ALDRIDGE_FELL), "03:30", times)?;
    let seen = cited::ALDRIDGE_LAST_TRACK + cited::ALDRIDGE_MINUTES_TO_SIGHTING;
    f.pin("F-C.aldridge.seen", clock(seen), "14:58", times)?;
    let minutes = seen - cited::ALDRIDGE_FELL;
    f.add("F-C.aldridge.in_water", format!("{}h{:02}", minutes / 60, minutes % 60))?;
    f.add("F-C.aldridge.in_water.hours", q(i128::from(minutes), 60).fixed(1))?;
    f.pin("F-C.aldridge.rerun", clock(cited::ALDRIDGE_RERUN), "08:30", times)?;
    f.add(
        "F-C.aldridge.rerun.hours_after_fall",
        q(i128::from(cited::ALDRIDGE_RERUN - cited::ALDRIDGE_FELL), 60).fixed(0),
    )?;
    f.pin("F-C.aldridge.miles_south", cited::ALDRIDGE_MILES_SOUTH, "30", times)?;
    f.pin(
        "F-C.aldridge.miles_offshore",
        format!(
            "{}-{}",
            cited::ALDRIDGE_MILES_OFFSHORE.0,
            cited::ALDRIDGE_MILES_OFFSHORE.1
        ),
        "40-43",
        "the Times (40) and the Coast Guard release of 24 July 2013 (43)",
    )?;
    Ok(())
}

/// F-N: the names behind the method.
fn names(f: &mut Facts) -> Result<(), String> {
    f.pin(
        "F-N.koopman",
        cited::KOOPMAN_YEAR,
        "1946",
        "Koopman, Search and Screening, OEG Report 56",
    )?;
    f.pin(
        "F-N.bayes.died",
        cited::BAYES_DIED,
        "1761",
        "Oxford Dictionary of National Biography",
    )?;
    f.pin(
        "F-N.bayes.read",
        cited::BAYES_READ,
        "1763",
        "Philosophical Transactions 53, p. 370",
    )?;
    f.pin("F-N.laplace", cited::LAPLACE_YEAR, "1774", "Stigler (1986)")?;
    f.pin(
        "F-N.chew",
        cited::CHEW_YEAR,
        "1967",
        "Annals of Mathematical Statistics 38",
    )?;
    f.pin(
        "F-N.paper",
        cited::PAPER_YEAR,
        "1971",
        "Naval Research Logistics Quarterly 18(2)",
    )?;
    f.add("F-N.paper.years_after", i64::from(cited::PAPER_YEAR) - cited::FOUND.0)?;
    Ok(())
}

/// F-Q: every quiz, by name: its countdown and its answer.
fn quizzes(f: &mut Facts) -> Result<(), String> {
    f.add("F-Q.count", QUIZZES.len())?;
    f.add("F-Q.names", list(QUIZZES.iter().map(|x| x.name)))?;
    let p = sea::prior();
    let run = sea::run(sea::Rule::Best, 2);
    for quiz in QUIZZES {
        let key = format!("F-Q.{}", quiz.name);
        f.add(&format!("{key}.seconds"), quiz.seconds)?;
        let answer = match quiz.ask {
            Ask::Stories => p[0].percent(0),
            Ask::FirstSearch => sea::CELLS[run[0].cell].to_owned(),
            Ask::AfterEmpty => run[0].after[run[0].cell].percent(1),
            Ask::NextSearch => sea::CELLS[run[1].cell].to_owned(),
            Ask::TwoSearches => sea::seen_within(sea::seeing()[1], 2).to_string(),
            Ask::RadiusDoubles => {
                let r = drift::radius(Q::ONE);
                ((r * r) / (r * Q::int(2) * r * Q::int(2))).to_string()
            }
        };
        f.add(&format!("{key}.answer"), answer)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pin_holds() {
        facts().unwrap().self_check().unwrap();
    }

    /// Narration truth for the cited sums the script leans on.
    #[test]
    fn the_cited_numbers_agree_with_one_another() {
        // 1,026 miles at one knot are the paper's 43 days, and its 153 miles its 6.4 days
        assert_eq!(q(i128::from(cited::TRACK_MILES), 24).fixed(0), "43");
        assert_eq!(q(i128::from(cited::MILES_TO_REACH_F6), 24).fixed(1), "6.4");
        assert_eq!(cited::AF447_WEIGHTS.iter().sum::<u32>(), 100);
        // the actual bottom time fell inside the forecast
        assert!((cited::FORECAST_DAYS.0..=cited::FORECAST_DAYS.1).contains(&cited::BOTTOM_DAYS));
        // at sea F6 always looked more thoroughly searched than it had been
        for (at_sea, after) in cited::F6_SEARCHED_AT_SEA.iter().zip(cited::F6_SEARCHED_AFTER) {
            assert!(*at_sea > after);
        }
        // constants written in lowest terms are in lowest terms
        for (c, n, d) in [(shown::MAP_SEEING, 2, 3), (METRES_PER_FOOT, 3048, 10_000)] {
            assert_eq!(c, q(n, d));
        }
    }
}
