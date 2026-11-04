//! Numbers for the Monty chapter, computed from its enumerated cases.
use explainer_kit::Facts;
use five_math_paradoxes::{BIRTHDAY_DAYS, Group, birthday_match_probability, catch_time, combine, inspection_groups, monty_cases};

pub fn facts() -> Result<Facts, String> {
    let cases = monty_cases();
    let mut facts = Facts::new();
    facts.add("monty.doors", cases.len())?;
    facts.add(
        "monty.switch_wins",
        cases.iter().filter(|case| case.switch_wins).count(),
    )?;
    facts.add(
        "monty.switch_losses",
        cases.iter().filter(|case| !case.switch_wins).count(),
    )?;
    let people = (0..=BIRTHDAY_DAYS + 1)
        .find(|&n| birthday_match_probability(n) > 0.5)
        .ok_or("no birthday threshold")?;
    let match_probability = birthday_match_probability(people);
    facts.add("birthday.days", BIRTHDAY_DAYS)?;
    facts.add("birthday.people", people)?;
    facts.add("birthday.second_numerator", BIRTHDAY_DAYS - 1)?;
    facts.add("birthday.third_numerator", BIRTHDAY_DAYS - 2)?;
    facts.add(
        "birthday.no_match_pct",
        format!("{:.2}", (1.0 - match_probability) * 100.0),
    )?;
    facts.add("birthday.match_pct", format!("{:.2}", match_probability * 100.0))?;
    facts.add("birthday.pairs", people * (people - 1) / 2)?;
    let (a, b) = inspection_groups();
    let rate = |g: Group| format!("{}/{} ({}%)", g.passed, g.total, 100 * g.passed / g.total);
    for (key, group) in [
        ("simpson.easy.a", a[0]), ("simpson.easy.b", b[0]),
        ("simpson.hard.a", a[1]), ("simpson.hard.b", b[1]),
        ("simpson.total.a", combine(&a).ok_or("invalid A groups")?),
        ("simpson.total.b", combine(&b).ok_or("invalid B groups")?),
    ] { facts.add(key, rate(group))?; }
    let (head, achilles, tortoise) = (100_u64, 10_u64, 1_u64);
    let (numerator, denominator) = catch_time(head, achilles, tortoise).ok_or("no catch")?;
    facts.add("zeno.head_start", head)?;
    facts.add("zeno.pursuer_speed", achilles)?;
    facts.add("zeno.other_speed", tortoise)?;
    facts.add("zeno.relative_speed", achilles - tortoise)?;
    let mut stage = head as f64 / achilles as f64;
    for i in 0..4 {
        let displayed = format!("{stage:.2}");
        let displayed = displayed.trim_end_matches('0').trim_end_matches('.');
        facts.add(&format!("zeno.stage{i}"), displayed)?;
        stage *= tortoise as f64 / achilles as f64;
    }
    facts.add("zeno.catch_fraction", format!("{numerator}/{denominator}"))?;
    facts.add("zeno.catch_decimal", format!("{:.2}", numerator as f64 / denominator as f64))?;
    facts.add("hotel.examples", 4)?;
    facts.add("hotel.new_room", 1)?;
    facts.self_check()?;
    Ok(facts)
}
