//! Every number Adding a Seat shows or says, computed from `model` (exact
//! integers) and printed as `key<TAB>value` for `tools/facts_check.py`,
//! which compares them with `oracle.py` (exact rationals, a different
//! formulation). Historical and published numbers are pinned literals with
//! their sources; the oracle prints the same literals.

use explainer_kit::Facts;

use crate::model::{
    self, ASH, CEDAR, EXAMPLE_YARDSTICK, GPP_A, GPP_B, GPP_C, GPP_HOUSE, HOUSE4, KEYS, KEYS4, POP, POP_GROWN,
    POP_PUZZLE, POP2, POP4, Ratio, Rule, decimal, divisor, grouped, hamilton, number, people, quota_rule, share,
    signposts, within_quota,
};

const YOUNG_P13: &str = "Young, Fairness in Apportionment (census PDF p. 13), quoting C. W. Seaton";
const YOUNG_P10: &str = "Young, Fairness in Apportionment (census PDF p. 10), Table 4";
const YOUNG_P16: &str = "Young, Fairness in Apportionment (census PDF p. 16)";
const GPP_T1: &str = "Goelz, Peters and Procaccia 2024, arXiv 2202.11061v2, Table 1";
const GPP_P7: &str = "Goelz, Peters and Procaccia 2024, p. 7, citing Balinski and Young 1975";
const GPP_P9: &str = "Goelz, Peters and Procaccia 2024, p. 9, on Balinski and Young 1982";
const GPP_P1: &str = "Goelz, Peters and Procaccia 2024, v2 accepted by Operations Research, June 2024";

fn town_seats(f: &mut Facts, prefix: &str, seats: &[u64]) -> Result<(), String> {
    for (i, s) in seats.iter().enumerate() {
        f.add(&format!("{prefix}.{}", KEYS[i]), s)?;
    }
    Ok(())
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let total = model::total(&POP);
    for (i, key) in KEYS.iter().enumerate() {
        f.add(&format!("pop.{key}"), grouped(u128::from(POP[i])))?;
    }
    f.add("pop.total", grouped(u128::from(total)))?;

    // shares, whole parts and tails
    for h in [14, 15, 16, 17] {
        let mut wholes = 0;
        for (i, key) in KEYS.iter().enumerate() {
            let q = share(&POP, i, h);
            f.add(&format!("share.{h}.{key}"), decimal(q, 2))?;
            f.add(&format!("whole.{h}.{key}"), q.floor())?;
            // the total is 39 thousand, so a tail is a whole number of 39ths
            f.add(&format!("tail39.{h}.{key}"), q.tail().num * 39 / q.den)?;
            f.add(&format!("tail.{h}.{key}"), decimal(q.tail(), 2))?;
            wholes += q.floor();
        }
        f.add(&format!("leftover.{h}"), u128::from(h) - wholes)?;
    }
    for (i, key) in [(ASH, "ash"), (model::BIRCH, "birch")] {
        let t = share(&POP, i, 15).tail();
        f.add(&format!("tail13.15.{key}"), t.num * 13 / t.den)?;
    }
    f.add("std_divisor.15", grouped(u128::from(total / 15)))?;
    for (i, key) in KEYS.iter().enumerate() {
        let speed = Ratio::new(u128::from(POP[i]), u128::from(total));
        f.add(&format!("speed39.{key}"), speed.num * 39 / speed.den)?;
        f.add(&format!("speed.{key}"), decimal(speed, 2))?;
    }

    // rounding each share to the nearest whole seat
    for h in [14, 15, 16] {
        let r = model::naive(&POP, h);
        town_seats(&mut f, &format!("naive.{h}"), &r)?;
        f.add(&format!("naive.{h}.total"), r.iter().sum::<u64>())?;
    }

    // Hamilton
    for h in [14, 15, 16, 17] {
        town_seats(&mut f, &format!("hamilton.{h}"), &hamilton(&POP, h)?)?;
    }
    let events = model::alabama_events(&POP)?;
    f.add("alabama.additions", 99)?;
    f.add("alabama.count", events.len())?;
    let mut losers: Vec<&str> = events.iter().flat_map(|e| e.1.iter().map(|&i| KEYS[i])).collect();
    losers.sort_unstable();
    losers.dedup();
    f.add("alabama.loser", losers.join(","))?;
    let houses: Vec<String> = events.iter().map(|e| format!("{}to{}", e.0, e.0 + 1)).collect();
    f.add("alabama.houses", houses.join(","))?;
    let mut quota_ok = true;
    for h in 1..=100 {
        quota_ok &= within_quota(&hamilton(&POP, h)?, &POP, h);
    }
    f.add("hamilton.within_quota_1_100", yes(quota_ok))?;

    // Jefferson: the yardstick window at 15 seats and the example yardstick
    let (low, high) = model::window(Rule::Jefferson, &POP, 15)?;
    f.add("jefferson.15.window_low", number(low))?;
    f.add("jefferson.15.window_high", number(high))?;
    f.add("jefferson.example_divisor", grouped(u128::from(EXAMPLE_YARDSTICK)))?;
    let d = u128::from(EXAMPLE_YARDSTICK);
    let mut at2300 = 0;
    for (i, key) in KEYS.iter().enumerate() {
        let x = Ratio::new(u128::from(POP[i]), d);
        f.add(&format!("jefferson.at2300.{key}"), decimal(x, 2))?;
        at2300 += x.floor();
    }
    f.add("jefferson.at2300.total", at2300)?;
    let mut at2600 = 0;
    for (i, key) in KEYS.iter().enumerate() {
        let q = share(&POP, i, 15);
        f.add(&format!("at2600.{key}"), decimal(q, 2))?;
        at2600 += q.floor();
    }
    f.add("at2600.whole_total", at2600)?;
    for (i, key) in [(ASH, "ash"), (CEDAR, "cedar")] {
        // the dropped tail as a percent of the quotient: (p mod d) / p
        let p = u128::from(POP[i]);
        f.add(
            &format!("jefferson.drop_pct.{key}"),
            decimal(Ratio::new(100 * (p % d), p), 1),
        )?;
    }
    for h in [14, 15, 16, 17] {
        town_seats(&mut f, &format!("jefferson.{h}"), &divisor(Rule::Jefferson, &POP, h)?)?;
    }

    // the signpost list
    let list = signposts(Rule::Jefferson, &POP, 19);
    for (rank, (i, k, value)) in list.iter().enumerate() {
        f.add(
            &format!("opening.{}", rank + 1),
            format!("{}{k}@{}", KEYS[*i], people(*value)),
        )?;
    }
    let tie = (1..list.len())
        .find(|&r| list[r].2 == list[r - 1].2)
        .ok_or("no tie in the list")?;
    f.add("opening.first_tie_rank", tie)?;
    f.add("opening.first_tie_value", number(list[tie].2))?;

    // Adams, Webster, Hill
    town_seats(&mut f, "adams.15", &divisor(Rule::Adams, &POP, 15)?)?;
    for rule in [Rule::Webster, Rule::Hill] {
        for h in [15, 16, 17] {
            town_seats(&mut f, &format!("{}.{h}", rule.key()), &divisor(rule, &POP, h)?)?;
        }
    }
    // scans over 1..100 seats; a tie house has no single answer and is skipped
    for rule in model::RULES {
        let (mut bad, mut above, mut below, mut ties) = (0, 0, 0, Vec::new());
        let mut below_houses = Vec::new();
        for h in 1..=100_u64 {
            let Ok(s) = divisor(rule, &POP, h) else {
                ties.push(h.to_string());
                continue;
            };
            let qs: Vec<Ratio> = (0..3).map(|i| share(&POP, i, h)).collect();
            bad += usize::from(!within_quota(&s, &POP, h));
            above += usize::from((0..3).any(|i| u128::from(s[i]) > qs[i].ceil()));
            let low = (0..3).any(|i| u128::from(s[i]) < qs[i].floor());
            below += usize::from(low);
            if low {
                below_houses.push(h.to_string());
            }
        }
        let key = rule.key();
        f.add(&format!("{key}.quota_breaks_1_100"), bad)?;
        f.add(&format!("{key}.above_quota_1_100"), above)?;
        f.add(&format!("{key}.below_quota_1_100"), below)?;
        f.add(
            &format!("{key}.tie_houses_1_100"),
            if ties.is_empty() { "none".into() } else { ties.join(",") },
        )?;
        if rule == Rule::Hill {
            f.add("hill.below_quota_houses_1_100", below_houses.join(","))?;
            town_seats(&mut f, "hill.3", &divisor(Rule::Hill, &POP, 3)?)?;
            f.add("share.3.cedar", decimal(share(&POP, CEDAR, 3), 2))?;
        }
    }

    // the four-town map where Webster leaves quota
    for (i, key) in KEYS4.iter().enumerate() {
        f.add(&format!("w4.pop.{key}"), grouped(u128::from(POP4[i])))?;
    }
    f.add("w4.pop.total", grouped(u128::from(model::total(&POP4))))?;
    for (i, key) in KEYS4.iter().enumerate() {
        f.add(&format!("w4.share.14.{key}"), decimal(share(&POP4, i, HOUSE4), 2))?;
    }
    let w4 = divisor(Rule::Webster, &POP4, HOUSE4)?;
    for (i, key) in KEYS4.iter().enumerate() {
        f.add(&format!("w4.webster.14.{key}"), w4[i])?;
    }
    f.add("w4.webster.14.within_quota", yes(within_quota(&w4, &POP4, HOUSE4)))?;
    let (low4, high4) = model::window(Rule::Webster, &POP4, HOUSE4)?;
    f.add("w4.window_low", number(low4))?;
    f.add("w4.window_high", number(high4))?;
    let mut three = 0;
    for h in 1..=100 {
        three += usize::from(!within_quota(&divisor(Rule::Webster, &POP, h)?, &POP, h));
    }
    f.add("webster.three_town_quota_breaks_1_100", three)?;

    // the quota rule, until its first tie
    let mut last = 100;
    let hist = loop {
        match quota_rule(&POP, last) {
            Ok(h) => break h,
            Err(_) => last -= 1,
        }
    };
    for h in [14, 15, 16, 17] {
        town_seats(&mut f, &format!("quotamethod.{h}"), &hist[h])?;
    }
    f.add(
        "quotamethod.first_tie_house",
        if last < 100 {
            (last + 1).to_string()
        } else {
            "none".into()
        },
    )?;
    f.add("quotamethod.checked_through", last)?;
    let ok = (1..=last).all(|h| within_quota(&hist[h as usize], &POP, h));
    let mono = (0..last as usize).all(|h| (0..3).all(|i| hist[h + 1][i] >= hist[h][i]));
    f.add("quotamethod.within_quota_checked", yes(ok))?;
    f.add("quotamethod.never_takes_back_checked", yes(mono))?;
    f.add("quotamethod.15.cedar_ceiling", share(&POP, CEDAR, 15).ceil())?;
    f.add(
        "quotamethod.15.birch_priority",
        people(model::signpost(Rule::Jefferson, POP[1], 4)),
    )?;
    f.add(
        "quotamethod.15.ash_priority",
        people(model::signpost(Rule::Jefferson, POP[ASH], 2)),
    )?;

    // the population paradox at the later census
    for (i, key) in KEYS.iter().enumerate() {
        f.add(&format!("census2.pop.{key}"), grouped(u128::from(POP2[i])))?;
    }
    f.add("census2.pop.total", grouped(u128::from(model::total(&POP2))))?;
    for (i, key) in KEYS.iter().enumerate() {
        f.add(&format!("census2.share.16.{key}"), decimal(share(&POP2, i, 16), 2))?;
    }
    town_seats(&mut f, "census2.hamilton.16", &hamilton(&POP2, 16)?)?;
    for rule in [Rule::Jefferson, Rule::Webster] {
        let (s1, s2) = (divisor(rule, &POP, 16)?, divisor(rule, &POP2, 16)?);
        town_seats(&mut f, &format!("census2.{}.16", rule.key()), &s2)?;
        f.add(
            &format!("census2.{}.paradox", rule.key()),
            yes(s2[CEDAR] < s1[CEDAR] && s2[ASH] > s1[ASH]),
        )?;
    }

    // the signpost lists the film moves its cards through
    let lists: [(&str, Rule, &[u64; 3], usize); 4] = [
        ("census2.grown", Rule::Jefferson, &POP_GROWN, 18),
        ("census2", Rule::Jefferson, &POP2, 18),
        ("webster", Rule::Webster, &POP, 17),
        ("puzzle.webster", Rule::Webster, &POP_PUZZLE, 17),
    ];
    for (name, rule, pops, count) in lists {
        for (rank, (i, k, value)) in signposts(rule, pops, count).iter().enumerate() {
            f.add(
                &format!("{name}.opening.{}", rank + 1),
                format!("{}{k}@{}", KEYS[*i], people(*value)),
            )?;
        }
    }
    f.add("puzzle.pop.cedar", grouped(u128::from(POP_PUZZLE[CEDAR])))?;
    // a quota lottery with Ash's share on average: 2 seats as often as its tail
    let tail = share(&POP, ASH, 15).tail();
    f.add(
        "lottery.15.ash.two_pct",
        decimal(Ratio::new(100 * tail.num, tail.den), 0),
    )?;
    f.add(
        "lottery.15.ash.one_pct",
        decimal(Ratio::new(100 * (tail.den - tail.num), tail.den), 0),
    )?;
    town_seats(&mut f, "puzzle.webster.15", &divisor(Rule::Webster, &POP_PUZZLE, 15)?)?;

    // Goelz, Peters and Procaccia's five-state proof, ten seats
    let printed: [(&str, [u64; 5], [&str; 5]); 3] = [
        ("A", GPP_A, ["8.24", "0.44", "0.44", "0.44", "0.44"]),
        ("B", GPP_B, ["6.99", "0.37", "0.37", "0.37", "1.88"]),
        ("C", GPP_C, ["9.02", "0.01", "0.01", "0.48", "0.48"]),
    ];
    for (name, pops, table) in printed {
        f.add(&format!("gpp.{name}.total"), model::total(&pops))?;
        for i in 0..5 {
            let q = share(&pops, i, GPP_HOUSE);
            let n = i + 1;
            f.add(&format!("gpp.{name}.pop.{n}"), pops[i])?;
            f.pin(&format!("gpp.{name}.share.{n}"), decimal(q, 2), table[i], GPP_T1)?;
            f.add(&format!("gpp.{name}.floor.{n}"), q.floor())?;
            f.add(&format!("gpp.{name}.ceil.{n}"), q.ceil())?;
        }
    }

    // history: cannot be computed, pinned with sources
    f.pin("history.seaton.house_before", 299, "299", YOUNG_P13)?;
    f.pin("history.seaton.house_after", 300, "300", YOUNG_P13)?;
    f.pin("history.seaton.alabama_before", 8, "8", YOUNG_P13)?;
    f.pin("history.seaton.alabama_after", 7, "7", YOUNG_P13)?;
    f.pin("history.seaton.range_low", 275, "275", YOUNG_P13)?;
    f.pin("history.seaton.range_high", 350, "350", YOUNG_P13)?;
    f.pin("history.ny1830.quota", "38.593", "38.593", YOUNG_P10)?;
    f.pin("history.ny1830.seats", 40, "40", YOUNG_P10)?;
    f.pin("history.hill_law_year", 1941, "1941", YOUNG_P16)?;
    f.pin("history.quota_method_year", 1975, "1975", GPP_P7)?;
    f.pin("history.by_impossibility_year", 1982, "1982", GPP_P9)?;
    f.pin("history.gpp_year", 2024, "2024", GPP_P1)?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The values the storyboard's words say, by key (spelled out there).
    #[test]
    fn the_spoken_numbers() {
        let f = facts().expect("facts build");
        for (key, want) in [
            ("hamilton.15.ash", "2"),
            ("hamilton.16.ash", "1"),
            ("hamilton.16.cedar", "11"),
            ("hamilton.17.ash", "2"),
            ("share.15.ash", "1.54"),
            ("share.16.birch", "3.69"),
            ("tail39.16.ash", "25"),
            ("tail39.16.birch", "27"),
            ("tail39.16.cedar", "26"),
            ("speed39.cedar", "26"),
            ("naive.14.total", "13"),
            ("naive.16.total", "17"),
            ("alabama.count", "5"),
            ("alabama.loser", "ash"),
            ("jefferson.15.window_high", "2,363.6"),
            ("jefferson.drop_pct.ash", "42.5"),
            ("jefferson.drop_pct.cedar", "2.7"),
            ("opening.16", "birch4@2,250"),
            ("opening.17", "cedar12@2,167"),
            ("opening.first_tie_rank", "18"),
            ("adams.15.cedar", "9"),
            ("webster.16.ash", "2"),
            ("hill.below_quota_houses_1_100", "3"),
            ("w4.webster.14.d", "8"),
            ("w4.share.14.d", "9.00"),
            ("quotamethod.15.birch", "4"),
            ("quotamethod.first_tie_house", "36"),
            ("census2.hamilton.16.cedar", "10"),
            ("census2.hamilton.16.ash", "2"),
            ("census2.webster.paradox", "no"),
            ("gpp.C.floor.1", "9"),
            ("gpp.B.ceil.1", "7"),
            ("gpp.B.ceil.5", "2"),
            // chapter 13: growth alone lifts Birch's fourth above Cedar's eleventh
            ("census2.grown.opening.15", "birch4@2,375"),
            ("census2.grown.opening.16", "cedar11@2,373"),
            ("census2.opening.9", "ash1@3,900"),
            // chapter 15: a grown Cedar pushes Ash's second below the cutoff
            ("webster.opening.15", "ash2@2,667"),
            ("puzzle.webster.opening.15", "cedar11@2,857"),
            ("puzzle.webster.15.ash", "1"),
            ("lottery.15.ash.two_pct", "54"),
            ("lottery.15.ash.one_pct", "46"),
        ] {
            assert_eq!(f.get(key), Some(want), "{key}");
        }
    }
}
