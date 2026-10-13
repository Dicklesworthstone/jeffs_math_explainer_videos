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
    /// The chance that the whole search so far would have found her (search
    /// effectiveness probability), in hundredths, on the same three dates:
    /// on scene (Table 1) and estimated afterwards (Table 3).
    pub const SEP_AT_SEA: [u32; 3] = [40, 73, 75];
    pub const SEP_AFTER: [u32; 3] = [29, 45, 48];

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
    /// Of those four, the three made under water: the listening search of
    /// June and July 2009 and the sonar searches of August 2009 and of
    /// April and May 2010 (the first of the four was the search of the
    /// surface).
    pub const AF447_UNDERWATER_SEARCHES: u32 = 3;

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
    /// The first map's guess at when he fell, the evening before:
    /// "between 9:30 p.m. and 11:30 p.m."; "Aldridge did not fall in the
    /// water at 10:30 p.m.; he fell in at 3:30 a.m."
    pub const ALDRIDGE_GUESS: (u32, u32) = (21 * 60 + 30, 23 * 60 + 30);
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
    pub const MAP_MARKS: [usize; 2] = [20, 81];
    /// "More dots, same map": how many dots each dot splits into before
    /// the second search, so that whole dots go out.
    pub const SPLIT: i128 = 3;
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
    code_panel(&mut f)?;
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

    // the second choice in dots of the first hundred: what a search of each
    // cell would catch now (each cell's next slice)
    let dots = Q::int(sea::DOTS);
    let mut searched = [0u32; 4];
    searched[b] = 1;
    let next: Vec<Q> = (0..4)
        .map(|c| sea::slices(c, searched[c] + 1)[searched[c] as usize] * dots)
        .collect();
    f.pin("F-M.second.find.dots", list(next.iter()), "10,20/3,40/3,20/3", hand)?;
    f.add("F-M.second.find.dots.rounded", list(next.iter().map(|x| x.fixed(0))))?;

    // "more dots, same map": every dot splits in three, so the second and
    // third searches put out whole dots
    let split = Q::int(shown::SPLIT);
    let lit = |p: &[Q; 4], total: Q| p.map(|x| x * total);
    let lit1 = left * split;
    f.add("F-M.split.factor", shown::SPLIT)?;
    f.pin("F-M.split.lit", lit1, "240", hand)?;
    f.pin(
        "F-M.split.lit.cells",
        list(lit(&run[0].after, lit1)),
        "120,30,60,30",
        hand,
    )?;
    let out2 = run[1].find * lit1;
    let lit2 = lit1 - out2;
    f.pin("F-M.split.second.dots_out", out2, "40", hand)?;
    f.pin(
        "F-M.split.second.lit.cells",
        list(lit(&run[1].after, lit2)),
        "120,30,20,30",
        hand,
    )?;
    let out3 = run[2].find * lit2;
    f.pin("F-M.third.dots_out", out3, "30", hand)?;
    f.add("F-M.split.third.lit.cells", list(lit(&run[2].after, lit2 - out3)))?;
    let run4 = sea::run(sea::Rule::Best, 4);
    f.pin("F-M.fourth.cell", sea::CELLS[run4[3].cell], "A", hand)?;

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

    // the proof in slices: what each cell's successive searches put out,
    // in dots of the first hundred, and what the rule's searches put out
    let dots = Q::int(sea::DOTS);
    for (cell, name) in sea::CELLS.iter().enumerate() {
        f.add(
            &format!("F-U.slices.{name}"),
            list(sea::slices(cell, 4).iter().map(|&x| (x * dots).fixed(1))),
        )?;
    }
    let mut found = Q::ZERO;
    let mut out = Vec::new();
    for step in &run {
        out.push((step.found_by - found) * dots);
        found = step.found_by;
    }
    f.add("F-U.best.dots_out", list(out.iter()))?;
    f.add("F-U.best.dots_out.rounded", list(out.iter().map(|x| x.fixed(1))))?;
    let mut all: Vec<Q> = (0..4).flat_map(|c| sea::slices(c, shown::BRUTE as u32)).collect();
    all.sort_by(|a, b| b.cmp(a));
    f.pin(
        "F-U.slices.rule_takes_biggest",
        out.iter().zip(&all).all(|(o, a)| *o == *a * dots),
        "true",
        "the storyboard's proof (08_2, 08_3)",
    )?;

    // a cell with no dots never gains one: the update multiplies
    let mut p = [q(1, 2), q(1, 2), Q::ZERO, Q::ZERO];
    for step in 0..30 {
        p = sea::bayes(&p, step % 2, shown::MAP_SEEING);
    }
    f.pin(
        "F-U.zero_stays_zero",
        p[2] == Q::ZERO && p[3] == Q::ZERO,
        "true",
        "14_4",
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
    f.add("F-R.E5.one_in", Q::ONE / share(e5))?;
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
    f.add(
        "F-R.grid.miles",
        format!(
            "{}x{}",
            (Q::int(y1968::GRID_ROWS.len() as i128) * y1968::CELL_MILES.0).fixed(0),
            (Q::int(y1968::GRID_COLS.len() as i128) * y1968::CELL_MILES.1).fixed(1)
        ),
    )?;
    f.add("F-R.grid.points", y1968::in_grid())?;
    f.add("F-R.outside.points", y1968::total() - y1968::in_grid())?;

    // the model search (the video's own model on the Navy's map)
    let seeing = shown::MAP_SEEING;
    f.add("F-R.model.seeing", seeing)?;
    let limit = shown::MAP_MARKS[shown::MAP_MARKS.len() - 1];
    let plans = [
        ("best", y1968::model_search(y1968::Rule::Best, seeing, limit)),
        ("once", y1968::model_search(y1968::Rule::Once, seeing, limit)),
        ("sweep", y1968::model_search(y1968::Rule::Sweep, seeing, limit)),
    ];
    f.add(
        "F-R.model.best.first_cells",
        list(plans[0].1[..6].iter().map(|l| y1968::name(l.row, l.col))),
    )?;
    for (name, plan) in &plans {
        for mark in shown::MAP_MARKS {
            f.add(
                &format!("F-R.model.{name}.found_by.{mark}"),
                plan[mark - 1].found_by.percent(1),
            )?;
        }
    }
    // one model search in miles of towing: a cell once over, at the
    // coverage that sees two times in three
    let per_cell = hook_once_miles() / Q::int((y1968::GRID_ROWS.len() * y1968::GRID_COLS.len()) as i128);
    f.add(
        "F-R.model.search_miles",
        format!("{:.0}", per_cell.f64() * koopman::coverage_for(seeing.f64())),
    )?;
    Ok(())
}

/// Sea miles of track to pass once over the whole lettered grid with the
/// hook's strip.
fn hook_once_miles() -> Q {
    let feet_per_nm = Q::int(METRES_PER_NM) / METRES_PER_FOOT;
    y1968::grid_area() / (Q::int(i128::from(shown::STRIP_FEET)) / feet_per_nm)
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
    for (name, feet) in [
        ("camera", cited::SWEEP_AFTER.0),
        ("magnetometer", cited::SWEEP_AFTER.1),
        ("believed", cited::SWEEP_AT_SEA.0),
    ] {
        let miles = y1968::grid_area() / (Q::int(i128::from(feet)) / feet_per_nm);
        f.add(&format!("F-H.once.miles.{name}"), miles.fixed(0))?;
    }
    // one cell, and how many cells the whole search's track would paint
    let cells = Q::int((y1968::GRID_ROWS.len() * y1968::GRID_COLS.len()) as i128);
    let per_cell = once / cells;
    f.add("F-H.once.per_cell", per_cell.fixed(0))?;
    f.add("F-H.once.passes_per_cell", (y1968::CELL_MILES.1 / strip_miles).fixed(0))?;
    let track = Q::int(i128::from(cited::TRACK_MILES));
    f.add("F-H.track.cells", (track / per_cell).fixed(0))?;
    f.add("F-H.track.share.percent", (track / once).percent(0))?;
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

    // F6's chance on the map on 9 October, by the paper's own update
    // L' = L (1 − E) ÷ (1 − SEP): with the ship's numbers, and with the
    // ones measured afterwards
    f.pin("F-S.SEP.at_sea", list(cited::SEP_AT_SEA), "40,73,75", paper)?;
    f.pin("F-S.SEP.after", list(cited::SEP_AFTER), "29,45,48", paper)?;
    let prior = q(
        i128::from(y1968::points("F6").ok_or("no F6")?),
        i128::from(y1968::REPLICATIONS),
    );
    let chance = |searched: u32, sep: u32| prior * left(searched) / left(sep);
    f.add(
        "F-S.F6.chance.at_sea",
        chance(cited::F6_SEARCHED_AT_SEA[1], cited::SEP_AT_SEA[1]).percent(2),
    )?;
    f.add(
        "F-S.F6.chance.after",
        chance(cited::F6_SEARCHED_AFTER[1], cited::SEP_AFTER[1]).percent(2),
    )?;
    // the same two figures as coverage on Koopman's curve (the paper scores
    // each cell by the random search formula, Appendix A)
    for (name, searched) in [
        ("at_sea", cited::F6_SEARCHED_AT_SEA[1]),
        ("after", cited::F6_SEARCHED_AFTER[1]),
    ] {
        f.add(
            &format!("F-S.F6.coverage.{name}"),
            format!("{:.2}", koopman::coverage_for(f64::from(searched) / 100.0)),
        )?;
    }
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
    f.add("F-P.years_before_scorpion", cited::FOUND.0 - cited::PALOMARES.0)?;
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
    f.pin("F-A.underwater_searches", cited::AF447_UNDERWATER_SEARCHES, "3", paper)?;

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
    f.pin(
        "F-C.aldridge.guess",
        format!("{}-{}", clock(cited::ALDRIDGE_GUESS.0), clock(cited::ALDRIDGE_GUESS.1)),
        "21:30-23:30",
        times,
    )?;
    // the guess's middle is the evening before: minutes before midnight count back from the fall
    let middle = (cited::ALDRIDGE_GUESS.0 + cited::ALDRIDGE_GUESS.1) / 2;
    f.pin("F-C.aldridge.guess.middle", clock(middle), "22:30", times)?;
    f.add(
        "F-C.aldridge.hours_off",
        q(i128::from(24 * 60 - middle + cited::ALDRIDGE_FELL), 60),
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

/// F-M.code: the code panel's four lines, as the storyboard pins them (the
/// oracle runs them on the four-cell sea).
fn code_panel(f: &mut Facts) -> Result<(), String> {
    for (n, line) in crate::chapters::glyphs::CODE.iter().enumerate() {
        f.add(&format!("F-M.code.{}", n + 1), line)?;
    }
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
        f.add(&format!("{key}.line"), quiz.line)?;
        f.add(&format!("{key}.seconds"), quiz.seconds)?;
        let answer = match quiz.ask {
            Ask::Stories => p[0].percent(0),
            Ask::FirstSearch => sea::CELLS[run[0].cell].to_owned(),
            Ask::AfterEmpty => run[0].after[run[0].cell].percent(1),
            Ask::NextSearch => sea::CELLS[run[1].cell].to_owned(),
            // of nine dots, how many two passes catch
            Ask::TwoSearches => (sea::seen_within(sea::seeing()[1], 2) * Q::int(9)).to_string(),
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

    /// Narration truth: every number the script speaks, as the words that
    /// speak it, against the fact it rests on. A rewording that drops the
    /// words, or a model change that moves the number, fails here.
    #[test]
    fn the_script_s_spoken_numbers() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let f = facts().unwrap();
        let rows: &[(&str, &str, &str, &str)] = &[
            ("02_1", "ninety-nine men", "F-S.crew", "99"),
            ("02_2", "on the twenty-seventh", "F-S.due", "1968-05-27"),
            ("02_2", "For nine days", "F-S.days.due_to_presumed", "9"),
            ("02_3", "five days before", "F-S.days.sank_to_due", "5"),
            ("02_3", "more than ten thousand feet", "F-S.depth.feet", "10000"),
            ("02_3", "All ninety-nine men", "F-S.crew", "99"),
            ("02_5", "about twelve miles on a side", "F-R.grid.miles", "12x12.6"),
            ("02_5", "about two hundred feet", "F-H.strip.feet", "200"),
            ("02_6", "thousands of miles", "F-H.once.hundreds", "46"),
            (
                "02_6",
                "several times all the open-sea searching",
                "F-H.once.over_track",
                "4.5",
            ),
            ("03_4", "sixty to forty", "F-M.story1.weight", "60"),
            ("03_4", "forty from the second", "F-M.story2.weight", "40"),
            ("03_4", "a hundred dots", "F-M.dots", "100"),
            ("03_5", "half of its dots", "F-M.story1.share", "1/2,1/3,1/6,0"),
            ("03_5", "a quarter of its dots", "F-M.story2.share", "1/4,1/4,1/4,1/4"),
            ("03_6", "Half of sixty is thirty", "F-M.story1.dots", "30,20,10,0"),
            ("03_6", "A quarter of forty is ten", "F-M.story2.dots", "10,10,10,10"),
            ("03_6", "Forty percent", "F-Q.stories.answer", "40"),
            ("03_7", "Forty, thirty, twenty, ten", "F-M.prior.percent", "40,30,20,10"),
            ("04_1", "two years earlier", "F-P.years_before_scorpion", "2"),
            ("04_2", "In July of sixty-eight", "F-S.scenarios_set", "1968-07-18"),
            ("04_2", "nine stories", "F-S.scenarios", "9"),
            ("04_3", "ten thousand times", "F-R.replications", "10000"),
            ("04_5", "one dot in every eight", "F-R.E5.one_in", "8"),
            ("04_6", "within two hundred and sixty yards", "F-S.yards", "260"),
            ("04_7", "five months", "F-S.months.due_to_found", "5"),
            ("05_1", "Forty percent", "F-M.prior.percent", "40,30,20,10"),
            ("05_2", "one time in four", "F-M.seeing", "1/4,2/3,2/3,2/3"),
            ("05_2", "two times in three", "F-M.seeing", "1/4,2/3,2/3,2/3"),
            ("05_5", "Cell B", "F-Q.first_search.answer", "B"),
            (
                "05_5",
                "A quarter of forty is ten",
                "F-M.find.percent",
                "10.0,20.0,13.3,6.7",
            ),
            (
                "05_5",
                "Two thirds of thirty is twenty",
                "F-M.find.percent",
                "10.0,20.0,13.3,6.7",
            ),
            ("05_5", "Twice the chance", "F-M.first.over_likeliest", "2"),
            ("06_2", "Twenty dots", "F-M.empty1.dots_out", "20"),
            ("06_4", "Eighty dots are left", "F-M.empty1.dots_left", "80"),
            ("06_4", "Ten are still in B", "F-M.empty1.cell_dots_left", "10"),
            ("06_6", "Twelve and a half percent", "F-Q.after_empty.answer", "12.5"),
            (
                "06_7",
                "forty dots out of eighty is one half",
                "F-M.empty1.after.fraction",
                "1/2,1/8,1/4,1/8",
            ),
            ("06_8", "grew by a quarter", "F-M.empty1.rise", "5/4"),
            (
                "06_8",
                "Forty to fifty",
                "F-M.empty1.after.percent",
                "50.0,12.5,25.0,12.5",
            ),
            ("07_1", "seventeen sixty-one", "F-N.bayes.died", "1761"),
            ("07_2", "One third for cell B", "F-M.seeing", "1/4,2/3,2/3,2/3"),
            ("07_2", "what is left, eighty", "F-M.empty1.dots_left", "80"),
            (
                "07_3",
                "half the chance",
                "F-M.empty1.after.percent",
                "50.0,12.5,25.0,12.5",
            ),
            ("07_4", "Cell C", "F-Q.next_search.answer", "C"),
            (
                "07_4",
                "and a search of C thirteen and a third",
                "F-M.second.find.dots",
                "10,20/3,40/3,20/3",
            ),
            ("08_2", "the first hundred dots", "F-M.dots", "100"),
            ("08_3", "the eight biggest slices", "F-U.brute.searches", "8"),
            (
                "08_3",
                "takes them biggest first",
                "F-U.slices.rule_takes_biggest",
                "true",
            ),
            ("08_3", "no other order is ever ahead", "F-U.brute.rule_is_best", "true"),
            ("08_4", "two times in three", "F-R.model.seeing", "2/3"),
            (
                "08_5",
                "less than one chance in a hundred",
                "F-R.model.sweep.found_by.20",
                "0.8",
            ),
            ("08_5", "about one in two", "F-R.model.best.found_by.20", "49.8"),
            ("09_1", "two times in three", "F-M.seeing", "1/4,2/3,2/3,2/3"),
            ("09_3", "sixty-three percent", "F-K.formula.1", "63.2"),
            ("09_3", "on average", "F-K.lanes.mean.1", "63.4"),
            ("09_4", "A little more track", "F-K.coverage.flat", "1.10"),
            ("09_5", "six dots of nine", "F-M.seen_within.1", "2/3"),
            ("09_6", "Eight", "F-Q.two_searches.answer", "8"),
            ("09_6", "One dot in nine", "F-M.seen_within.2", "8/9"),
            ("09_6", "one in twenty-seven", "F-M.seen_within.3", "26/27"),
            ("10_3", "a tenth of a mile", "F-D.strip", "0.1"),
            ("10_4", "four times the area", "F-D.double.area_factor", "4"),
            ("10_4", "a quarter of the track", "F-D.double.coverage_factor", "1/4"),
            (
                "10_4",
                "one hour in, a flight would find the person ninety-three times in a hundred",
                "F-D.h1.seeing",
                "93",
            ),
            (
                "10_4",
                "Twelve hours in, the same flight would find that person thirteen times in a hundred",
                "F-D.h12.seeing",
                "13",
            ),
            ("11_1", "nineteen seventy-four", "F-C.casp", "1974"),
            ("11_2", "July of twenty thirteen", "F-C.aldridge.date", "2013-07-24"),
            ("11_2", "about forty miles", "F-C.aldridge.miles_offshore", "40-43"),
            ("11_3", "the evening before", "F-C.aldridge.guess", "21:30-23:30"),
            ("11_3", "almost thirty miles", "F-C.aldridge.miles_south", "30"),
            ("11_6", "almost twelve hours", "F-C.aldridge.in_water", "11h28"),
            ("12_1", "June, two thousand nine", "F-A.lost", "2009-06-01"),
            ("12_1", "two hundred and twenty-eight", "F-A.aboard", "228"),
            ("12_2", "forty nautical miles", "F-A.circle.nm", "40"),
            ("12_2", "nearly two years", "F-A.months", "22"),
            ("12_2", "three underwater searches", "F-A.underwater_searches", "3"),
            ("12_3", "thirty-five, thirty-five and thirty", "F-A.weights", "35,35,30"),
            ("13_1", "nineteen sixty-eight", "F-S.map_arrived", "1968-08-12"),
            (
                "13_1",
                "in August, two months into the search",
                "F-S.months.search_to_map",
                "2",
            ),
            ("13_1", "since June", "F-S.search_began", "1968-06-10"),
            ("13_2", "ninety-four percent", "F-S.F6.searched.at_sea", "65,94,99"),
            ("13_2", "ninety-four went out", "F-S.F6.left.at_sea.percent", "6"),
            ("13_4", "eight hundred feet", "F-S.sweep.at_sea", "800,400"),
            ("13_5", "had not seen her at all", "F-S.sweep.after", "185,220,0"),
            ("13_5", "about two hundred feet", "F-S.sweep.after", "185,220,0"),
            (
                "13_6",
                "sixty-five percent, not ninety-four",
                "F-S.F6.searched.after",
                "47,65,74",
            ),
            ("13_7", "at the end of October", "F-S.found", "1968-10-28"),
            ("14_3", "five hours off", "F-C.aldridge.hours_off", "5"),
            ("14_5", "ninety-nine men", "F-S.crew", "99"),
        ];
        let mut bad = Vec::new();
        for (line, words, key, value) in rows {
            match script.text(line) {
                Some(text) if text.contains(words) => {}
                Some(_) => bad.push(format!("{line}: the script no longer says {words:?}")),
                None => bad.push(format!("{line}: no such line")),
            }
            if f.get(key) != Some(value) {
                bad.push(format!(
                    "{line}: {key} is {:?}, the words {words:?} say {value}",
                    f.get(key)
                ));
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
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
