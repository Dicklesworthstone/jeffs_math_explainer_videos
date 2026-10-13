//! Every number Video E shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet (the owner's brief, or the
//! source a cited number comes from), and `./x fair_coin facts-check`
//! against `oracle.py` (same keys). The chances and costs are exact
//! fractions from `model.rs`, printed at the precision the screen shows; the
//! seeded runs are the ones the chapters draw.

use explainer_kit::Facts;

use crate::model::{
    self, Announce, FAIR_DIE, Face, Game, LOADED, Memory, PairLaw, Q, all_different, bent, blum_first_two,
    count_chance, die, elementary, elias, elias_rate, entropy, expected_tosses, fixed_length_rules, games,
    heads_first_wins, memory_tosses, pascal_row, pattern_chance, patterns, peres, peres_rate, rolls, spread_chances,
    stream, tally, tie, tosses, tosses_used, two_sigma_band, undecided_after, von_neumann,
};

/// Dates and quoted numbers from the sources (fact sheet rows E-C; the
/// oracle prints the same literals).
pub mod cited {
    /// The Monte Carlo symposium at UCLA, June 29 to July 1, 1949.
    pub const TALK_YEAR: u32 = 1949;
    /// "Various Techniques Used in Connection With Random Digits", summary
    /// written by George E. Forsythe, in Monte Carlo Method, National Bureau
    /// of Standards Applied Mathematics Series 12 (issued June 11, 1951).
    pub const PRINTED_YEAR: u32 = 1951;
    pub const PAGES: &str = "36-38";
    pub const SERIES_NUMBER: u32 = 12;
    /// "…at most 25 percent as efficient as ordinary coin-tossing" (p. 36).
    pub const EFFICIENCY_PERCENT: u32 = 25;
    /// Samuelson, JASA 63(324): tosses that follow a common outcome.
    pub const SAMUELSON_YEAR: u32 = 1968;
    /// Hoeffding and Simons, Ann. Math. Statist. 41(2).
    pub const HOEFFDING_SIMONS_YEAR: u32 = 1970;
    /// Elias, Ann. Math. Statist. 43(3).
    pub const ELIAS_YEAR: u32 = 1972;
    /// Blum, Combinatorica 6(2) (first at FOCS 1984).
    pub const BLUM_YEAR: u32 = 1986;
    /// Dijkstra, EWD 1071; Information Processing Letters 36(4): a roulette
    /// with 37 equally likely outcomes from a possibly biased coin.
    pub const DIJKSTRA_YEAR: u32 = 1990;
    pub const DIJKSTRA_OUTCOMES: u32 = 37;
    /// Peres, Ann. Statist. 20(1).
    pub const PERES_YEAR: u32 = 1992;
    /// Juels, Jakobsson, Shriver and Hillyer, IEEE Trans. Inform. Theory 46(3).
    pub const JUELS_YEAR: u32 = 2000;
    /// Gelman and Nolan, The American Statistician 56(4).
    pub const GELMAN_NOLAN_YEAR: u32 = 2002;
    /// Diaconis, Holmes and Montgomery, SIAM Review 49(2): "the chance of
    /// coming up as started is about .51".
    pub const DIACONIS_YEAR: u32 = 2007;
    pub const SAME_SIDE_PERCENT: u32 = 51;
    /// Bartoš et al., JASA 120(552): 350,757 flips, Pr(same side) = 0.508.
    pub const BARTOS_YEAR: u32 = 2025;
    pub const BARTOS_FLIPS: u32 = 350_757;
    pub const BARTOS_SAME_SIDE: &str = "0.508";
    /// Chance News 11.02 (2002): one instructor's classes spun 1960s
    /// pennies 5,520 times and saw 1,878 heads.
    pub const PENNY_SPINS: u32 = 5520;
    pub const PENNY_HEADS: u32 = 1878;
    /// Jun and Kocher, "The Intel Random Number Generator" (Cryptography
    /// Research, 1999): "an average of one bit for every 6 raw binary samples".
    pub const INTEL_YEAR: u32 = 1999;
    pub const INTEL_SAMPLES_PER_BIT: u32 = 6;
}

/// The slider's stops: the coin's chance of heads (one of them below a
/// half, where the coin favours tails).
pub fn stops() -> [Q; 7] {
    [
        Q::new(1, 5),
        Q::HALF,
        Q::new(3, 5),
        Q::new(7, 10),
        bent(),
        Q::new(9, 10),
        Q::new(99, 100),
    ]
}

/// The second coin of the two-coin game: a fair one.
pub fn second_coin() -> Q {
    Q::HALF
}

/// The coin with memory: it repeats its last face as often as the bent coin
/// shows heads.
pub fn sticky() -> Memory {
    Memory::sticky(bent())
}

/// How many tosses or games each seeded run has.
pub mod runs {
    pub const CALL_IT: usize = 1000;
    pub const TRICK: usize = 1000;
    pub const PRICE: usize = 10_000;
    pub const TWO_COINS: usize = 1000;
    pub const STICKY: usize = 200_000;
    pub const COIN_DIE: usize = 24_000;
    pub const LOADED: usize = 6000;
    pub const SQUEEZE: usize = 100_000;
    /// Peres's procedure run until nothing is left.
    pub const PERES_ALL: u32 = 40;
    pub const ELIAS_BLOCK: usize = 16;
}

/// Run 1: just calling it.
pub fn call_it_run() -> Vec<Face> {
    tosses(&mut stream(1), runs::CALL_IT, bent())
}

/// Run 2: the trick.
pub fn trick_run() -> Vec<Game> {
    games(&mut stream(2), runs::TRICK, bent(), bent())
}

/// Run 4: the price.
pub fn price_run() -> Vec<Game> {
    games(&mut stream(4), runs::PRICE, bent(), bent())
}

/// Run 5: two different coins.
pub fn two_coin_run() -> Vec<Game> {
    games(&mut stream(5), runs::TWO_COINS, bent(), second_coin())
}

/// Run 6: the coin with memory, after a head.
pub fn sticky_run() -> Vec<Face> {
    memory_tosses(&mut stream(6), runs::STICKY, sticky(), Face::H)
}

/// Run 7: tosses to build dice from.
pub fn coin_die_run() -> Vec<Face> {
    tosses(&mut stream(7), runs::COIN_DIE, bent())
}

/// Run 8: the loaded die.
pub fn loaded_run() -> Vec<usize> {
    rolls(&mut stream(8), runs::LOADED, &LOADED)
}

/// Run 9: tosses to squeeze fair ones from.
pub fn squeeze_run() -> Vec<Face> {
    tosses(&mut stream(9), runs::SQUEEZE, bent())
}

fn heads(run: &[Face]) -> usize {
    run.iter().filter(|&&f| f == Face::H).count()
}

fn letters(run: &[Face]) -> String {
    run.iter().map(|f| f.letter()).collect()
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn ratio(n: usize, d: usize) -> Q {
    Q::new(n as i128, d as i128)
}

/// "p80" for four fifths.
pub fn stop_name(p: Q) -> String {
    format!("p{}", p.percent(0))
}

/// Among the pairs of a run that decide and follow a pair `before`, how
/// many are heads-tails and how many tails-heads.
pub fn decided_after(run: &[Face], before: (Face, Face)) -> (usize, usize) {
    let pairs: Vec<(Face, Face)> = run.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])).collect();
    pairs
        .windows(2)
        .filter(|w| w[0] == before)
        .fold((0, 0), |(ht, th), w| match w[1] {
            (Face::H, Face::T) => (ht + 1, th),
            (Face::T, Face::H) => (ht, th + 1),
            _ => (ht, th),
        })
}

/// The tosses of a run that come right after a head, the run itself having
/// started after a head.
pub fn after_heads(run: &[Face]) -> Vec<Face> {
    std::iter::once(&Face::H)
        .chain(run)
        .zip(run)
        .filter(|(before, _)| **before == Face::H)
        .map(|(_, &x)| x)
        .collect()
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    coin(&mut f)?;
    breaks(&mut f)?;
    same_counts(&mut f)?;
    dice(&mut f)?;
    rates(&mut f)?;
    edges(&mut f)?;
    seeded(&mut f)?;
    sources(&mut f)?;
    Ok(f)
}

/// E-F1 to E-F4: two tosses of one coin, and what the trick costs.
fn coin(f: &mut Facts) -> Result<(), String> {
    let p = bent();
    let law = PairLaw::same_coin(p);
    let brief = "the owner's brief";
    for ((a, b), want) in model::PAIRS.into_iter().zip(["0.64", "0.16", "0.16", "0.04"]) {
        let key = format!("E-F1.square.{}", format!("{}{}", a.letter(), b.letter()).to_lowercase());
        f.pin(&key, law.prob(a, b).decimal(2), want, brief)?;
        f.add(&format!("{key}.exact"), law.prob(a, b).exact())?;
        f.add(&format!("{key}_percent"), law.prob(a, b).percent(0))?;
    }
    f.pin("E-F1.heads_percent", p.percent(0), "80", brief)?;
    f.add("E-F1.tails_percent", p.other().percent(0))?;
    let decides = law.decides();
    f.pin("E-F2.decides", decides.decimal(2), "0.32", brief)?;
    f.add("E-F2.decides_percent", decides.percent(0))?;
    f.add("E-F2.thrown_away", decides.other().decimal(2))?;
    f.add("E-F2.thrown_away_percent", decides.other().percent(0))?;
    let share = law.heads_first_share().ok_or("the bent coin never decides")?;
    f.pin("E-F2.heads_first_share.exact", share.exact(), "1/2", "the trick")?;
    f.add("E-F2.heads_first_share_percent", share.percent(0))?;
    let cost = expected_tosses(p).ok_or("the bent coin never decides")?;
    f.add("E-F3.expected_pairs", (cost / Q::int(2)).decimal(3))?;
    f.pin("E-F3.expected_tosses", cost.decimal(2), "6.25", brief)?;
    f.add("E-F3.expected_tosses.exact", cost.exact())?;
    for r in [1, 2, 3, 5, 10] {
        f.add(
            &format!("E-F3.undecided_after.{r}_pairs_percent"),
            undecided_after(p, r).percent(1),
        )?;
    }
    // "about one game in fifty is still going after ten pairs"
    f.add(
        "E-F3.undecided_after.10_pairs.one_game_in",
        (Q::ONE / undecided_after(p, 10)).decimal(0),
    )?;
    for s in stops() {
        let law = PairLaw::same_coin(s);
        let cost = expected_tosses(s).ok_or("a stop never decides")?;
        let name = stop_name(s);
        f.add(&format!("E-F4.{name}.ht"), law.prob(Face::H, Face::T).decimal(4))?;
        f.add(&format!("E-F4.{name}.decides_percent"), law.decides().percent(1))?;
        f.add(&format!("E-F4.{name}.expected_tosses"), cost.decimal(2))?;
    }
    // "Nine percent each", at ninety percent heads
    let ninety = PairLaw::same_coin(Q::new(9, 10));
    f.add("E-F4.p90.ht_percent", ninety.prob(Face::H, Face::T).percent(0))?;
    // where the count of a fair game's wins lands about 19 times in 20
    let (low, high) = two_sigma_band(runs::TRICK as u32, Q::HALF);
    f.add("E-F2.fair_band.games1000", format!("{low},{high}"))?;
    let nearly_sure = expected_tosses(Q::new(99, 100)).ok_or("a stop never decides")?;
    f.pin("E-F4.p99.expected_tosses_rounded", nearly_sure.decimal(0), "101", brief)?;
    let fair = expected_tosses(Q::HALF).ok_or("a fair coin never decides")?;
    f.pin("E-F4.p50.expected_tosses.exact", fair.exact(), "4", brief)?;
    Ok(())
}

/// E-F5 to E-F7: two coins, a coin with memory, Blum's two algorithms.
fn breaks(f: &mut Facts) -> Result<(), String> {
    let two = PairLaw::two_coins(bent(), second_coin());
    let (ht, th) = (two.prob(Face::H, Face::T), two.prob(Face::T, Face::H));
    f.add("E-F5.two_coins.second_percent", second_coin().percent(0))?;
    f.add("E-F5.two_coins.ht", ht.decimal(2))?;
    f.add("E-F5.two_coins.th", th.decimal(2))?;
    f.add("E-F5.two_coins.ht_percent", ht.percent(0))?;
    f.add("E-F5.two_coins.th_percent", th.percent(0))?;
    let share = two.heads_first_share().ok_or("the two coins never decide")?;
    f.add("E-F5.two_coins.heads_first_percent", share.percent(0))?;
    f.add("E-F5.two_coins.ratio", (ht / th).exact())?;

    let m = sticky();
    let (after_h, after_t) = (m.pair_after(Face::H), m.pair_after(Face::T));
    let (ht, th) = (after_h.prob(Face::H, Face::T), after_h.prob(Face::T, Face::H));
    f.add("E-F6.sticky.repeat_percent", m.after_h.percent(0))?;
    f.add("E-F6.sticky.after_heads.ht", ht.decimal(2))?;
    f.add("E-F6.sticky.after_heads.th", th.decimal(2))?;
    f.add("E-F6.sticky.after_heads.ht_percent", ht.percent(0))?;
    f.add("E-F6.sticky.after_heads.th_percent", th.percent(0))?;
    f.add("E-F6.sticky.after_heads.ht.exact", ht.exact())?;
    f.add("E-F6.sticky.after_heads.th.exact", th.exact())?;
    f.add("E-F6.sticky.after_heads.ratio", (ht / th).exact())?;
    let stuck = "the sticky coin never decides";
    f.add(
        "E-F6.sticky.after_heads.heads_first_percent",
        after_h.heads_first_share().ok_or(stuck)?.percent(0),
    )?;
    f.add(
        "E-F6.sticky.after_tails.heads_first_percent",
        after_t.heads_first_share().ok_or(stuck)?.percent(0),
    )?;
    let first = m.long_run_pair().and_then(|l| l.heads_first_share()).ok_or(stuck)?;
    f.add("E-F6.sticky.long_run_first_pair.heads_first.exact", first.exact())?;
    f.add(
        "E-F6.sticky.plain_rule.after_heads_percent",
        m.plain_rule(Face::H).ok_or(stuck)?.percent(1),
    )?;
    f.add(
        "E-F6.sticky.plain_rule.after_tails_percent",
        m.plain_rule(Face::T).ok_or(stuck)?.percent(1),
    )?;
    f.add(
        "E-F6.sticky.plain_rule.long_run.exact",
        m.plain_rule_long_run().ok_or(stuck)?.exact(),
    )?;
    for (name, prev) in [("from_heads", Face::H), ("from_tails", Face::T)] {
        f.add(
            &format!("E-F6.sticky.after_heads_only.{name}.exact"),
            m.after_only_rule(Face::H, prev).ok_or(stuck)?.exact(),
        )?;
    }
    let lopsided = Memory {
        after_h: Q::new(9, 10),
        after_t: Q::HALF,
    };
    f.add(
        "E-F6.lopsided.plain_rule.long_run.exact",
        lopsided.plain_rule_long_run().ok_or(stuck)?.exact(),
    )?;
    f.add(
        "E-F6.lopsided.after_heads_only.exact",
        lopsided.after_only_rule(Face::H, Face::H).ok_or(stuck)?.exact(),
    )?;

    let stay = m.after_h.f64();
    let (at_once, on_return) = (
        blum_first_two(stay, Announce::AtOnce),
        blum_first_two(stay, Announce::OnReturn),
    );
    for (i, name) in ["hh", "ht", "th", "tt"].into_iter().enumerate() {
        f.add(&format!("E-F7.blum.at_once.{name}"), format!("{:.4}", at_once[i]))?;
        f.pin(
            &format!("E-F7.blum.on_return.{name}"),
            format!("{:.4}", on_return[i]),
            "0.2500",
            "Blum 1986, Theorem B",
        )?;
    }
    Ok(())
}

/// E-F8: patterns with the same counts.
fn same_counts(f: &mut Facts) -> Result<(), String> {
    let p = bent();
    for n in [2, 3, 4] {
        f.add(&format!("E-F8.pascal.row{n}"), list(pascal_row(n)))?;
    }
    let six = patterns(4, 2);
    f.pin(
        "E-F8.four_two.patterns",
        six.len(),
        "6",
        "the owner's brief: C(4,2) = 6",
    )?;
    f.add("E-F8.four_two.list", list(six.iter().map(|s| letters(s))))?;
    f.add("E-F8.four_two.each", pattern_chance(p, 4, 2).decimal(4))?;
    f.add("E-F8.four_two.each.exact", pattern_chance(p, 4, 2).exact())?;
    let works = count_chance(p, 4, 2);
    f.add("E-F8.four_two.works_percent", works.percent(1))?;
    f.add("E-F8.four_two.expected_tosses", (Q::int(4) / works).decimal(1))?;
    f.add("E-F8.four_two.works_per_hundred", works.percent(0))?;
    f.add("E-F8.four_two.expected_tosses_rounded", (Q::int(4) / works).decimal(0))?;
    f.add("E-F8.four_two.each_percent", pattern_chance(p, 4, 2).percent(2))?;
    // the cube of three tosses: one piece for each pattern, by its count of heads
    for k in 0..=3 {
        f.add(
            &format!("E-F8.three.heads{k}.each_percent"),
            pattern_chance(p, 3, k).percent(1),
        )?;
    }
    let fair_works = count_chance(Q::HALF, 4, 2);
    f.add("E-F8.four_two.fair_coin.works_percent", fair_works.percent(1))?;
    f.add(
        "E-F8.four_two.fair_coin.expected_tosses",
        (Q::int(4) / fair_works).decimal(1),
    )?;
    for k in 0..=4 {
        f.add(&format!("E-F8.row4.heads{k}_percent"), count_chance(p, 4, k).percent(2))?;
    }
    let three = count_chance(p, 3, 1) + count_chance(p, 3, 2);
    f.add("E-F8.three.works_percent", three.percent(0))?;
    let coin = expected_tosses(p).ok_or("the bent coin never decides")?;
    f.add(
        "E-F8.three_plus_coin.expected_tosses",
        (Q::int(3) / three + coin).decimal(1),
    )?;
    Ok(())
}

/// E-F9: a loaded die.
fn dice(f: &mut Facts) -> Result<(), String> {
    let (w, fair) = (die(&LOADED), die(&FAIR_DIE));
    f.add("E-F9.loaded.percents", list(w.iter().map(|x| x.percent(0))))?;
    f.add("E-F9.loaded.six_percent", w[5].percent(0))?;
    let ties = tie(&w);
    f.add("E-F9.two_rolls.tie_percent", ties.percent(1))?;
    f.add("E-F9.two_rolls.first_bigger_percent", elementary(&w, 2).percent(2))?;
    f.add("E-F9.two_rolls.decides_percent", ties.other().percent(1))?;
    f.add("E-F9.two_rolls.expected_rolls", (Q::int(2) / ties.other()).decimal(1))?;
    f.add(
        "E-F9.two_rolls.fair_die.expected_rolls",
        (Q::int(2) / tie(&fair).other()).decimal(1),
    )?;
    f.pin(
        "E-F9.three_rolls.orders",
        (1..=3).product::<u32>(),
        "6",
        "the owner's brief: 3! = 6",
    )?;
    f.add("E-F9.three_rolls.each_order_percent", elementary(&w, 3).percent(2))?;
    let works = all_different(&w, 3);
    f.add("E-F9.three_rolls.works_percent", works.percent(1))?;
    f.add("E-F9.three_rolls.works_per_hundred", works.percent(0))?;
    f.add("E-F9.three_rolls.expected_rolls", (Q::int(3) / works).decimal(1))?;
    f.add(
        "E-F9.three_rolls.expected_rolls_rounded",
        (Q::int(3) / works).decimal(0),
    )?;
    // one step further: four rolls that all differ, and what that costs
    let four = all_different(&w, 4);
    f.add("E-F9.four_rolls.works_per_hundred", four.percent(0))?;
    f.add("E-F9.four_rolls.expected_rolls_rounded", (Q::int(4) / four).decimal(0))?;
    let fair_works = all_different(&fair, 3);
    f.add("E-F9.three_rolls.fair_die.works.exact", fair_works.exact())?;
    f.add(
        "E-F9.three_rolls.fair_die.expected_rolls",
        (Q::int(3) / fair_works).decimal(1),
    )?;
    let trios = explainer_kit::choose(6, 3);
    f.add("E-F9.three_rolls.trios", trios)?;
    f.add("E-F9.three_rolls.boxes_all_different", trios * 6)?;
    f.add("E-F9.three_rolls.boxes", 6u32.pow(3))?;
    f.add("E-F9.brick.sides_percent", list(w[3..].iter().map(|x| x.percent(0))))?;
    f.add("E-F9.brick.volume_percent", (w[3] * w[4] * w[5]).percent(1))?;
    for n in 2..=6u64 {
        f.add(&format!("E-F9.orders.{n}"), (1..=n).product::<u64>())?;
    }
    Ok(())
}

/// The depths of Peres's procedure and the block sizes of Elias's rule the
/// last picture draws.
pub const PERES_DEPTHS: [u32; 6] = [1, 2, 3, 4, 6, 8];
pub const ELIAS_BLOCKS: [u32; 6] = [2, 4, 8, 16, 32, 64];

/// E-F10: what the trick throws away.
fn rates(f: &mut Facts) -> Result<(), String> {
    let exact = bent() * bent().other();
    let p = bent().f64();
    let h = entropy(p);
    f.pin("E-F10.von_neumann.rate", exact.decimal(2), "0.16", "the owner's brief")?;
    f.add("E-F10.von_neumann.per_hundred", exact.percent(0))?;
    f.add("E-F10.entropy", format!("{h:.4}"))?;
    f.pin(
        "E-F10.entropy.per_hundred",
        format!("{:.0}", 100.0 * h),
        "72",
        "the owner's brief",
    )?;
    // "just over seventy-two"
    f.add("E-F10.entropy.per_hundred_1dp", format!("{:.1}", 100.0 * h))?;
    f.add(
        "E-F10.von_neumann.share_of_entropy_percent",
        format!("{:.0}", 100.0 * p * (1.0 - p) / h),
    )?;
    let fair = Q::HALF * Q::HALF;
    f.add("E-F10.fair_coin.von_neumann.rate", fair.decimal(2))?;
    f.pin(
        "E-F10.fair_coin.von_neumann.percent",
        fair.percent(0),
        &cited::EFFICIENCY_PERCENT.to_string(),
        "von Neumann 1951, p. 36",
    )?;
    f.add("E-F10.fair_coin.entropy", format!("{:.0}", entropy(0.5)))?;
    // "three tosses in four"
    f.add("E-F10.fair_coin.wasted_percent", fair.other().percent(0))?;
    for d in PERES_DEPTHS {
        let r = peres_rate(d, p);
        f.add(&format!("E-F10.peres.depth{d}.rate"), format!("{r:.4}"))?;
        f.add(
            &format!("E-F10.peres.depth{d}.per_hundred"),
            format!("{:.0}", 100.0 * r),
        )?;
    }
    for n in ELIAS_BLOCKS {
        let r = elias_rate(n, p);
        f.add(&format!("E-F10.elias.block{n}.rate"), format!("{r:.4}"))?;
        f.add(
            &format!("E-F10.elias.block{n}.per_hundred"),
            format!("{:.0}", 100.0 * r),
        )?;
    }
    Ok(())
}

/// E-X: the edges of the always/never sentences that are counts.
fn edges(f: &mut Facts) -> Result<(), String> {
    for n in 1..=5 {
        let (lists, fair) = fixed_length_rules(n, &spread_chances(n));
        f.add(&format!("E-X1.fixed_rules.n{n}.count_lists"), lists)?;
        f.pin(
            &format!("E-X1.fixed_rules.n{n}.fair_for_every_coin"),
            fair,
            "0",
            "at p near 0 one pattern has nearly all the chance",
        )?;
    }
    Ok(())
}

/// E-S: the seeded runs.
fn seeded(f: &mut Facts) -> Result<(), String> {
    f.add("E-S0.seed", model::SEED)?;
    let s1 = call_it_run();
    f.add("E-S1.call_it.tosses", s1.len())?;
    f.add("E-S1.call_it.heads", heads(&s1))?;
    f.add("E-S1.call_it.tails", s1.len() - heads(&s1))?;

    let g = trick_run();
    let longest = |g: &[Game]| g.iter().map(Vec::len).max().unwrap_or(0);
    f.add("E-S2.trick.games", g.len())?;
    f.add("E-S2.trick.heads_first", heads_first_wins(&g))?;
    f.add("E-S2.trick.tails_first", g.len() - heads_first_wins(&g))?;
    f.add("E-S2.trick.tosses", tosses_used(&g))?;
    f.add("E-S2.trick.mean_tosses", ratio(tosses_used(&g), g.len()).decimal(2))?;
    f.add("E-S2.trick.longest_pairs", longest(&g))?;
    let first_games: Vec<String> = g
        .iter()
        .take(8)
        .map(|game| game.iter().flat_map(|p| [p.0.letter(), p.1.letter()]).collect())
        .collect();
    f.add("E-S2.trick.first_games", first_games.join(" "))?;

    // every pair of the run is a dot in the square: the deciding ones and the rest
    let pairs: usize = g.iter().map(Vec::len).sum();
    f.add("E-S2.trick.pairs", pairs)?;
    f.add("E-S2.trick.thrown_away", pairs - g.len())?;

    let g = price_run();
    f.add("E-S4.price.games", g.len())?;
    f.add("E-S4.price.mean_tosses", ratio(tosses_used(&g), g.len()).decimal(2))?;
    let with = |pairs: usize| g.iter().filter(|x| x.len() == pairs).count();
    f.add("E-S4.price.first_pair_percent", ratio(with(1), g.len()).percent(1))?;
    let hist: Vec<usize> = (1..=8).map(with).collect();
    f.add("E-S4.price.pairs_9_plus", g.len() - hist.iter().sum::<usize>())?;
    f.add("E-S4.price.pairs_1_to_8", list(hist))?;
    f.add("E-S4.price.longest_pairs", longest(&g))?;
    f.add("E-S4.price.longest_tosses", 2 * longest(&g))?;

    let g = two_coin_run();
    f.add("E-S5.two_coins.games", g.len())?;
    f.add("E-S5.two_coins.heads_first", heads_first_wins(&g))?;

    let run = sticky_run();
    f.add("E-S6.sticky.tosses", run.len())?;
    f.add("E-S6.sticky.heads_percent", ratio(heads(&run), run.len()).percent(1))?;
    let (ht, th) = decided_after(&run, (Face::H, Face::H));
    f.add("E-S6.sticky.after_hh.ht", ht)?;
    f.add("E-S6.sticky.after_hh.th", th)?;
    f.add(
        "E-S6.sticky.after_hh.heads_first_percent",
        ratio(ht, ht + th).percent(1),
    )?;
    let (ht, th) = decided_after(&run, (Face::T, Face::T));
    f.add("E-S6.sticky.after_tt.ht", ht)?;
    f.add("E-S6.sticky.after_tt.th", th)?;
    let fixed = von_neumann(&after_heads(&run));
    f.add("E-S6.sticky.after_heads_only.heads_first", heads(&fixed))?;
    f.add("E-S6.sticky.after_heads_only.tails_first", fixed.len() - heads(&fixed))?;
    f.add("E-S6.sticky.after_heads_only.games", fixed.len())?;
    f.add(
        "E-S6.sticky.after_heads_only.heads_first_percent",
        ratio(heads(&fixed), fixed.len()).percent(1),
    )?;

    let run = coin_die_run();
    let made = model::blocks(&run, 4, model::die_from_four_tosses);
    f.add("E-S7.coin_die.tosses", run.len())?;
    f.add("E-S7.coin_die.dice", made.len())?;
    f.add("E-S7.coin_die.faces", list(tally(&made, 6)))?;
    f.add("E-S7.coin_die.tosses_per_die", ratio(run.len(), made.len()).decimal(1))?;
    let sixth = Q::new(1, 6);
    let (low, high) = two_sigma_band(made.len() as u32, sixth);
    f.add("E-S7.coin_die.fair_band", format!("{low},{high}"))?;

    let run = loaded_run();
    f.add("E-S8.loaded.rolls", run.len())?;
    let faces: Vec<usize> = run.iter().map(|r| r - 1).collect();
    f.add("E-S8.loaded.faces", list(tally(&faces, 6)))?;
    let made = model::blocks(&run, 3, model::order_of_rolls);
    f.add("E-S8.three_rolls.dice", made.len())?;
    f.add("E-S8.three_rolls.orders", list(tally(&made, 6)))?;
    let (low, high) = two_sigma_band(made.len() as u32, sixth);
    f.add("E-S8.three_rolls.fair_band", format!("{low},{high}"))?;
    f.add(
        "E-S8.three_rolls.rolls_per_die",
        ratio(run.len(), made.len()).decimal(1),
    )?;
    let (pairs, _) = run.as_chunks::<2>();
    let two = |keep: fn(usize, usize) -> bool| pairs.iter().filter(|c| keep(c[0], c[1])).count();
    f.add("E-S8.two_rolls.first_bigger", two(|a, b| a > b))?;
    f.add("E-S8.two_rolls.second_bigger", two(|a, b| b > a))?;
    f.add("E-S8.two_rolls.ties", two(|a, b| a == b))?;

    let run = squeeze_run();
    f.add("E-S9.squeeze.tosses", run.len())?;
    f.add("E-S9.squeeze.heads", heads(&run))?;
    for (name, out) in [
        ("von_neumann", von_neumann(&run)),
        ("peres2", peres(&run, 2)),
        ("peres3", peres(&run, 3)),
        ("peres_all", peres(&run, runs::PERES_ALL)),
        ("elias16", elias(&run, runs::ELIAS_BLOCK)),
    ] {
        f.add(&format!("E-S9.squeeze.{name}.fair_tosses"), out.len())?;
        f.add(&format!("E-S9.squeeze.{name}.heads"), heads(&out))?;
        f.add(
            &format!("E-S9.squeeze.{name}.per_hundred"),
            ratio(100 * out.len(), run.len()).decimal(1),
        )?;
    }
    Ok(())
}

/// E-C: the cited literals, each pinned to what its source prints.
fn sources(f: &mut Facts) -> Result<(), String> {
    let ams = "Monte Carlo Method, NBS Applied Mathematics Series 12, title page and p. 36";
    f.pin("E-C1.von_neumann.talk_year", cited::TALK_YEAR, "1949", ams)?;
    f.pin("E-C1.von_neumann.printed_year", cited::PRINTED_YEAR, "1951", ams)?;
    f.pin("E-C1.von_neumann.pages", cited::PAGES, "36-38", ams)?;
    f.pin("E-C1.von_neumann.series_number", cited::SERIES_NUMBER, "12", ams)?;
    f.pin(
        "E-C1.von_neumann.efficiency_percent",
        cited::EFFICIENCY_PERCENT,
        "25",
        ams,
    )?;
    f.pin("E-C2.samuelson.year", cited::SAMUELSON_YEAR, "1968", "JASA 63(324)")?;
    f.pin(
        "E-C2.hoeffding_simons.year",
        cited::HOEFFDING_SIMONS_YEAR,
        "1970",
        "Ann. Math. Statist. 41(2):341-352",
    )?;
    f.pin(
        "E-C2.elias.year",
        cited::ELIAS_YEAR,
        "1972",
        "Ann. Math. Statist. 43(3):865-870",
    )?;
    f.pin("E-C2.blum.year", cited::BLUM_YEAR, "1986", "Combinatorica 6(2):97-108")?;
    f.pin(
        "E-C2.dijkstra.year",
        cited::DIJKSTRA_YEAR,
        "1990",
        "Inform. Process. Lett. 36(4):193",
    )?;
    f.pin("E-C2.dijkstra.outcomes", cited::DIJKSTRA_OUTCOMES, "37", "EWD 1071")?;
    f.pin(
        "E-C2.peres.year",
        cited::PERES_YEAR,
        "1992",
        "Ann. Statist. 20(1):590-597",
    )?;
    f.pin(
        "E-C2.juels.year",
        cited::JUELS_YEAR,
        "2000",
        "IEEE Trans. Inform. Theory 46(3):911-921",
    )?;
    f.pin(
        "E-C3.gelman_nolan.year",
        cited::GELMAN_NOLAN_YEAR,
        "2002",
        "Amer. Statist. 56(4):308-311",
    )?;
    f.pin(
        "E-C3.diaconis.year",
        cited::DIACONIS_YEAR,
        "2007",
        "SIAM Review 49(2):211-235",
    )?;
    f.pin(
        "E-C3.diaconis.same_side_percent",
        cited::SAME_SIDE_PERCENT,
        "51",
        "SIAM Review 49(2), abstract",
    )?;
    f.pin(
        "E-C3.bartos.year",
        cited::BARTOS_YEAR,
        "2025",
        "JASA 120(552):2118-2127",
    )?;
    f.pin(
        "E-C3.bartos.flips",
        cited::BARTOS_FLIPS,
        "350757",
        "JASA 120(552), title",
    )?;
    f.pin(
        "E-C3.bartos.same_side",
        cited::BARTOS_SAME_SIDE,
        "0.508",
        "JASA 120(552), abstract",
    )?;
    let chance_news = "Chance News 11.02 (2002)";
    f.pin("E-C3.pennies.spins", cited::PENNY_SPINS, "5520", chance_news)?;
    f.pin("E-C3.pennies.heads", cited::PENNY_HEADS, "1878", chance_news)?;
    f.pin(
        "E-C3.pennies.heads_percent",
        ratio(cited::PENNY_HEADS as usize, cited::PENNY_SPINS as usize).percent(0),
        "34",
        chance_news,
    )?;
    let intel = "Jun and Kocher 1999, section 4.3";
    f.pin("E-C4.intel.year", cited::INTEL_YEAR, "1999", intel)?;
    f.pin(
        "E-C4.intel.raw_samples_per_bit",
        cited::INTEL_SAMPLES_PER_BIT,
        "6",
        intel,
    )?;
    // Dijkstra's roulette: the 2^37 − 2 records that are not all one face
    // fall into groups of 37 rotations
    let n = u64::from(cited::DIJKSTRA_OUTCOMES);
    f.add("E-C2.dijkstra.patterns_per_outcome", ((1u64 << n) - 2) / n)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(key: &str) -> String {
        facts().unwrap().get(key).map(str::to_owned).ok_or(key).unwrap()
    }

    fn number(key: &str) -> f64 {
        fact(key).parse::<f64>().map_err(|e| format!("{key}: {e}")).unwrap()
    }

    #[test]
    fn the_facts_register_and_pass_their_pins() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert!(f.len() > 200);
    }

    /// The seeded runs agree with the model: each count sits within three
    /// standard errors of what the chance says (a run that did not would be
    /// shown as it is, and said so).
    #[test]
    fn the_seeded_runs_sit_where_the_model_says() {
        let near = |got: f64, n: f64, chance: f64| {
            let sd = (n * chance * (1.0 - chance)).sqrt();
            assert!((got - n * chance).abs() <= 3.0 * sd, "{got} of {n} at chance {chance}");
        };
        near(number("E-S1.call_it.heads"), 1000.0, 0.8);
        near(number("E-S2.trick.heads_first"), 1000.0, 0.5);
        near(number("E-S5.two_coins.heads_first"), 1000.0, 0.8);
        let (ht, th) = (number("E-S6.sticky.after_hh.ht"), number("E-S6.sticky.after_hh.th"));
        near(ht, ht + th, 0.8);
        let (h, t) = (
            number("E-S6.sticky.after_heads_only.heads_first"),
            number("E-S6.sticky.after_heads_only.tails_first"),
        );
        near(h, h + t, 0.5);
        near(number("E-S7.coin_die.dice"), 6000.0, 0.1536);
        near(number("E-S8.three_rolls.dice"), 2000.0, 0.3255);
        for name in ["von_neumann", "peres2", "peres3", "peres_all", "elias16"] {
            let n = number(&format!("E-S9.squeeze.{name}.fair_tosses"));
            near(number(&format!("E-S9.squeeze.{name}.heads")), n, 0.5);
        }
        // the measured rates against the formulas, per hundred tosses
        for (name, model) in [
            ("von_neumann", peres_rate(1, 0.8)),
            ("peres2", peres_rate(2, 0.8)),
            ("peres3", peres_rate(3, 0.8)),
            ("elias16", elias_rate(16, 0.8)),
        ] {
            let got = number(&format!("E-S9.squeeze.{name}.per_hundred"));
            assert!((got - 100.0 * model).abs() < 0.6, "{name}: {got} vs {}", 100.0 * model);
        }
        let all = number("E-S9.squeeze.peres_all.per_hundred");
        assert!(all < 100.0 * entropy(0.8) && all > 100.0 * peres_rate(8, 0.8));
    }

    /// Each face of the two made dice, and each order, against one sixth.
    #[test]
    fn the_made_dice_are_fair_within_chance() {
        for (key, made) in [
            ("E-S7.coin_die.faces", "E-S7.coin_die.dice"),
            ("E-S8.three_rolls.orders", "E-S8.three_rolls.dice"),
        ] {
            let n = number(made);
            let counts: Vec<f64> = fact(key).split(',').map(|c| c.parse().unwrap()).collect();
            assert_eq!(counts.len(), 6);
            assert_eq!(counts.iter().sum::<f64>(), n);
            // chi-squared with 5 degrees of freedom: 15.09 is its 99th percentile
            let chi: f64 = counts.iter().map(|c| (c - n / 6.0).powi(2) / (n / 6.0)).sum();
            assert!(chi < 15.09, "{key}: chi-squared {chi}");
        }
        // and the loaded die really is loaded
        let faces: Vec<f64> = fact("E-S8.loaded.faces")
            .split(',')
            .map(|c| c.parse().unwrap())
            .collect();
        assert!(faces[5] > 2800.0 && faces[0] < 400.0);
    }

    #[test]
    fn the_sticky_helpers_read_a_run_the_way_the_oracle_does() {
        use Face::{H, T};
        let run = [H, H, H, T, T, T, T, H, H, H, T, H];
        // pairs: HH HT TT TH HH TH
        assert_eq!(decided_after(&run, (H, H)), (1, 1));
        assert_eq!(decided_after(&run, (T, T)), (0, 1));
        // the tosses after a head (the run itself starts after one)
        assert_eq!(after_heads(&[H, T, T, H, H]), [H, T, H]);
        assert_eq!(stop_name(bent()), "p80");
    }
}
