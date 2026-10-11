//! Every number Video C shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet, and `./x bakery_algorithm
//! facts-check` against `oracle.py` (same keys). Orders, traces and the
//! seated runs come from `model.rs` (the checker's semantics, replayed);
//! the chapters call the same functions, so what they draw is what is
//! pinned here. The big model-checker sweeps (C-F4's three customers, C-F8)
//! stay the oracle's.

use explainer_kit::{Facts, choose};

use crate::model::{
    self, Cmp, FIVE, Order, SCORE_START, SLATES, Step, Trace, Variant, bakery_orders, bakery_program,
    blue_takes_a_ticket, purple_and_teal_tie, reads_by, red_point, scoreboard_orders, service_orders, sign_orders,
};

/// Dates and quoted numbers from the sources (fact sheet rows C-F14; the
/// oracle prints the same literals, cited).
pub mod cited {
    /// Dijkstra poses the problem (CACM 8(9), 1965).
    pub const DIJKSTRA_YEAR: u32 = 1965;
    /// Knuth asks that everyone gets in eventually (CACM 9(5), 1966).
    pub const KNUTH_YEAR: u32 = 1966;
    /// "In 1973, this was considered impossible" (Turing lecture).
    pub const CONSIDERED_IMPOSSIBLE_YEAR: u32 = 1973;
    /// The bakery paper (CACM 17(8), 1974).
    pub const BAKERY_YEAR: u32 = 1974;
    /// The Turing lecture (CACM 58(6), 2015).
    pub const TURING_LECTURE_YEAR: u32 = 2015;
    /// "…while it is changing from 9 to 10 obtains the value 2496."
    pub const TORN_FROM: u32 = 9;
    pub const TORN_TO: u32 = 10;
    pub const TORN_VALUE: u32 = 2496;
}

/// Customer `me` (1-based) takes 1 + the biggest other slate (the chalk rule
/// of 03_4, by hand; the model's doorway must agree).
pub fn take_ticket(slates: &[u32], me: usize) -> u32 {
    1 + slates
        .iter()
        .enumerate()
        .filter(|&(i, _)| i + 1 != me)
        .map(|(_, &t)| t)
        .max()
        .unwrap_or(0)
}

/// Who is served first: nonzero tickets in (ticket, customer number) order.
pub fn service_order(slates: &[u32]) -> Vec<usize> {
    let mut held: Vec<(u32, usize)> = slates
        .iter()
        .enumerate()
        .filter(|(_, t)| **t > 0)
        .map(|(i, &t)| (t, i + 1))
        .collect();
    held.sort_unstable();
    held.into_iter().map(|(_, k)| k).collect()
}

pub fn list<T: ToString>(v: &[T]) -> String {
    v.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

fn bit(b: bool) -> u8 {
    u8::from(b)
}

/// The lattice point "both have read, neither has written" (01_8).
pub const MIDDLE: (usize, usize) = (1, 1);

/// Orders of a two-customer program: (all, with two inside, through the
/// both-inside point, deadlocked, ending with two inside).
fn tally(orders: &[Order], len: usize) -> (usize, usize, usize, usize, usize) {
    (
        orders.len(),
        orders.iter().filter(|o| o.both_inside).count(),
        orders.iter().filter(|o| o.visits(red_point(len))).count(),
        orders.iter().filter(|o| !o.done).count(),
        orders.iter().filter(|o| o.end_both_inside(len)).count(),
    )
}

fn in_room_at_end(t: Trace, steps: &[Step]) -> usize {
    steps.last().map_or(0, |s| t.model().in_room(&s.after).len())
}

fn max_in_room(t: Trace, steps: &[Step]) -> usize {
    let m = t.model();
    steps.iter().map(|s| m.in_room(&s.after).len()).max().unwrap_or(0)
}

/// The tickets in the state after the last step.
fn final_tickets(steps: &[Step]) -> String {
    steps.last().map(|s| list(&s.after.number)).unwrap_or_default()
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    // 01: two increments, a read and a write each
    let sb = scoreboard_orders();
    let lost: Vec<&Order> = sb.iter().filter(|o| o.end[0] == SCORE_START + 1).collect();
    let middle: Vec<&Order> = sb.iter().filter(|o| o.visits(MIDDLE)).collect();
    f.pin("C-F1.score_start", SCORE_START, "41", "fact sheet C-F1 (01_4)")?;
    f.pin("C-F1.orders", sb.len(), "6", "fact sheet C-F1")?;
    f.pin("C-F1.orders_lose_a_point", lost.len(), "4", "fact sheet C-F1")?;
    f.pin("C-F1.orders_through_middle", middle.len(), "4", "fact sheet C-F1")?;
    f.pin(
        "C-F1.losing_orders_are_exactly_the_middle_orders",
        bit(lost == middle),
        "1",
        "fact sheet C-F1 (01_8)",
    )?;
    let lowest = |os: &[&Order]| os.iter().map(|o| o.end[0]).min().unwrap_or(0);
    let kept: Vec<&Order> = sb.iter().filter(|o| o.end[0] != SCORE_START + 1).collect();
    f.pin(
        "C-F1.score_end_when_a_point_is_lost",
        lowest(&lost),
        "42",
        "fact sheet C-F1",
    )?;
    f.pin("C-F1.score_end_otherwise", lowest(&kept), "43", "fact sheet C-F1")?;
    f.pin("C-F1.binomial_4_choose_2", choose(4, 2), "6", "fact sheet C-F1 (01_7)")?;
    // 02: the door sign
    let (all, both, red, dead, end) = tally(&sign_orders(), model::sign_program().len());
    f.pin("C-F2.orders", all, "14", "fact sheet C-F2")?;
    f.pin("C-F2.orders_both_inside", both, "8", "fact sheet C-F2")?;
    f.pin("C-F2.orders_through_red_square", red, "8", "fact sheet C-F2")?;
    f.pin("C-F2.orders_deadlocked", dead, "0", "fact sheet C-F2")?;
    f.pin(
        "C-F2.orders_ending_with_both_inside",
        end,
        "0",
        "fact sheet C-F2 (finding F1)",
    )?;
    // 05: two customers, one visit each, without and with the flag
    for (flag, tag, want) in [(false, "no_flag", ["28", "4"]), (true, "flag", ["322", "0"])] {
        let (all, both, red, dead, _) = tally(
            &bakery_orders(flag, Cmp::Lex),
            bakery_program(2, 0, flag, Cmp::Lex).len(),
        );
        f.pin(&format!("C-F3.{tag}.orders"), all, want[0], "fact sheet C-F3")?;
        f.pin(
            &format!("C-F3.{tag}.orders_both_inside"),
            both,
            want[1],
            "fact sheet C-F3",
        )?;
        f.pin(
            &format!("C-F3.{tag}.orders_through_red_square"),
            red,
            want[1],
            "fact sheet C-F3",
        )?;
        f.pin(&format!("C-F3.{tag}.orders_deadlocked"), dead, "0", "fact sheet C-F3")?;
    }
    for (cmp, tag) in [(Cmp::Strict, "strict_lt_with_flag"), (Cmp::Le, "le_with_flag")] {
        let (all, both, _, dead, _) = tally(&bakery_orders(true, cmp), bakery_program(2, 0, true, cmp).len());
        f.add(&format!("C-F5.{tag}.orders"), all)?;
        f.add(&format!("C-F5.{tag}.orders_both_inside"), both)?;
        f.add(&format!("C-F5.{tag}.orders_deadlocked"), dead)?;
    }
    // the replayed traces
    let a = Trace::A.steps();
    f.pin("C-F6.a.steps", a.len(), "8", "fact sheet C-F6")?;
    f.pin(
        "C-F6.a.in_room_at_end",
        in_room_at_end(Trace::A, &a),
        "2",
        "fact sheet C-F6 (05_3)",
    )?;
    f.pin(
        "C-F6.a.steps_counted",
        a.iter().filter(|s| s.counted()).count(),
        "6",
        "fact sheet C-F6 (05_3)",
    )?;
    let a2 = Trace::APrime.steps();
    f.pin("C-F6.a_prime.steps", a2.len(), "15", "fact sheet C-F6")?;
    f.pin(
        "C-F6.a_prime.max_in_room",
        max_in_room(Trace::APrime, &a2),
        "1",
        "fact sheet C-F6",
    )?;
    let stop = a2
        .iter()
        .position(|s| s.customer == 2 && s.label.contains("choosing[1]=1, wait"))
        .map_or(0, |k| k + 1);
    f.pin(
        "C-F6.a_prime.customer2_stopped_by_flag_at_step",
        stop,
        "7",
        "fact sheet C-F6 (05_6)",
    )?;
    f.pin(
        "C-F6.a_prime.steps_counted",
        a2.iter().filter(|s| s.counted()).count(),
        "13",
        "fact sheet C-F6 (05_5)",
    )?;
    let b1 = Trace::B1.steps();
    f.add("C-F6.b1.steps", b1.len())?;
    f.pin(
        "C-F6.b1.in_room_at_end",
        in_room_at_end(Trace::B1, &b1),
        "2",
        "fact sheet C-F6 (04_3)",
    )?;
    let b2 = Trace::B2.steps();
    f.add("C-F6.b2.steps", b2.len())?;
    let stuck = b2
        .last()
        .map(|s| s.after.clone())
        .unwrap_or_else(|| Trace::B2.model().initial());
    let succ = Trace::B2.model().successors(&stuck);
    let deadlocked =
        !succ.is_empty() && succ.iter().all(|(_, _, ns)| *ns == stuck) && Trace::B2.model().in_room(&stuck).is_empty();
    f.pin("C-F6.b2.deadlocked", bit(deadlocked), "1", "fact sheet C-F6 (04_3)")?;
    let c = Trace::C.steps();
    f.add("C-F6.c.steps", c.len())?;
    f.pin("C-F6.c.tickets", final_tickets(&c), "1,1", "fact sheet C-F6")?;
    let c3 = Trace::CThree.steps();
    f.add("C-F6.c_three_customers.steps", c3.len())?;
    f.add("C-F6.c_three_customers.tickets", final_tickets(&c3))?;
    let d = Trace::D.steps();
    let entries: Vec<(u32, usize)> = d
        .iter()
        .filter(|s| s.enters())
        .map(|s| (s.after.number[s.customer - 1], s.customer))
        .collect();
    let pairs = |v: &[(u32, usize)]| v.iter().map(|(t, p)| format!("({t},{p})")).collect::<String>();
    f.pin("C-F6.d.steps", d.len(), "70", "fact sheet C-F6")?;
    f.add("C-F6.d.room_entries", entries.len())?;
    f.add("C-F6.d.service_order", pairs(&entries))?;
    f.add("C-F6.d.service_order_is_lexicographic", bit(entries.is_sorted()))?;
    let taken: Vec<String> = d
        .iter()
        .filter_map(Step::ticket_written)
        .map(|(p, t)| format!("P{p}:{t}"))
        .collect();
    f.add("C-F6.d.tickets_taken", taken.join(","))?;
    f.add("C-F6.d.max_in_room", max_in_room(Trace::D, &d))?;
    f.pin(
        "C-F6.d.steps_counted",
        d.iter().filter(|s| s.counted()).count(),
        "64",
        "fact sheet C-F6 (07_1)",
    )?;
    let e1 = Trace::E1.steps();
    f.add("C-F6.e1.steps", e1.len())?;
    let garbage = e1
        .iter()
        .find(|s| s.customer == 2 && s.label.starts_with("read number[1]="))
        .and_then(|s| s.label.rsplit_once('=').and_then(|(_, x)| x.parse::<u32>().ok()));
    f.add("C-F6.e1.overlapping_read_value", garbage.unwrap_or(0))?;
    let e1_ticket = e1
        .iter()
        .filter_map(Step::ticket_written)
        .find(|&(p, _)| p == 2)
        .map_or(0, |(_, t)| t);
    f.add("C-F6.e1.ticket_taken_after_garbage", e1_ticket)?;
    f.add("C-F6.e1.max_in_room", max_in_room(Trace::E1, &e1))?;
    let e3 = Trace::E3.steps();
    f.add("C-F6.e3.steps", e3.len())?;
    // the glance at Blue's slate that lets Green in, mid-write
    let glance = e3
        .iter()
        .find(|s| s.customer == 2 && s.label.starts_with("L3: number[1]=") && s.enters())
        .ok_or("trace e3: Green never enters on a glance at Blue's slate")?;
    let seen: u32 = glance
        .label
        .trim_start_matches("L3: number[1]=")
        .split(',')
        .next()
        .and_then(|x| x.parse().ok())
        .ok_or("trace e3: unreadable glance")?;
    f.add("C-F6.e3.overlapping_read_value", seen)?;
    f.add(
        "C-F6.e3.overlapping_read_equals_pre_write_value",
        bit(glance.before.number[0] == seen),
    )?;
    f.add("C-F6.e3.customer2_enters_on_that_read", bit(glance.enters()))?;
    f.add("C-F6.e3.final_tickets", final_tickets(&e3))?;
    f.add("C-F6.e3.max_in_room", max_in_room(Trace::E3, &e3))?;
    let leap = Trace::Leapfrog.steps();
    f.add("C-F6.leapfrog.steps", leap.len())?;
    let leap_tickets: Vec<u32> = leap.iter().filter_map(Step::ticket_written).map(|(_, t)| t).collect();
    f.pin(
        "C-F6.leapfrog.tickets",
        list(&leap_tickets),
        "1,2,3,4,5,6",
        "fact sheet C-F6 (11_3)",
    )?;
    f.add("C-F6.leapfrog.final_tickets", final_tickets(&leap))?;
    // 10_4: the max that reads a slate twice (safe slates)
    let df3 = model::explore(&model::Bakery::new(3, 3, Variant::DoubleFetch, true, false));
    f.add(
        "C-F6.double_fetch.n3_k3_safe.shortest_violation_steps",
        df3.violation.as_ref().map_or(0, Vec::len),
    )?;
    f.add("C-F6.double_fetch.n3_k3_safe.states", df3.states)?;
    f.add(
        "C-F6.double_fetch.n3_k3_safe.violation_found",
        bit(df3.violation.is_some()),
    )?;
    for k in [6, 9] {
        let r = model::explore(&model::Bakery::new(2, k, Variant::DoubleFetch, true, true));
        f.add(&format!("C-F6.double_fetch.n2_k{k}_safe.states"), r.states)?;
        f.add(
            &format!("C-F6.double_fetch.n2_k{k}_safe.violation_found"),
            bit(r.violation.is_some()),
        )?;
    }
    // 03: Blue reads 3, 0, 0, 2 and takes 4, by hand and in the model
    let reads: Vec<u32> = SLATES
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 0)
        .map(|(_, &t)| t)
        .collect();
    f.pin("C-F7.slates_before", list(&SLATES), "0,3,0,0,2", "fact sheet C-F7")?;
    f.pin(
        "C-F7.customer1_reads",
        list(&reads),
        "3,0,0,2",
        "R1 Maya M3 (checked by hand)",
    )?;
    let ticket = take_ticket(&SLATES, 1);
    f.pin("C-F7.customer1_ticket", ticket, "4", "R1 Maya M3 (checked by hand)")?;
    let mut after = SLATES;
    after[0] = ticket;
    f.pin("C-F7.slates_after_customer1", list(&after), "4,3,0,0,2", "R1 Maya M3")?;
    f.pin(
        "C-F7.service_order_customer1",
        list(&service_order(&after)),
        "5,2,1",
        "R1 Maya M3",
    )?;
    let blue = blue_takes_a_ticket();
    let st1 = &blue.last().ok_or("Blue's doorway has no steps")?.after;
    f.pin(
        "C-F7.model.customer1_reads",
        list(&reads_by(&blue, 1)),
        "3,0,0,2",
        "fact sheet C-F7",
    )?;
    f.pin("C-F7.model.customer1_ticket", st1.number[0], "4", "fact sheet C-F7")?;
    let orders1 = service_orders(&FIVE, st1, &[1, 2, 5]);
    f.pin(
        "C-F7.model.customer1_distinct_service_orders",
        orders1.len(),
        "1",
        "fact sheet C-F7",
    )?;
    let first = |o: &std::collections::BTreeSet<Vec<usize>>| o.iter().next().map(|v| list(v)).unwrap_or_default();
    f.pin(
        "C-F7.model.customer1_service_order",
        first(&orders1),
        "5,2,1",
        "fact sheet C-F7 (03_5)",
    )?;
    // 04: Purple and Teal read at the same moment and both write 5
    let tie = purple_and_teal_tie();
    let st2 = &tie.last().ok_or("the tie has no steps")?.after;
    f.pin(
        "C-F7.model.tie.customer3_reads",
        list(&reads_by(&tie, 3)),
        "4,3,0,2",
        "fact sheet C-F7 (04_2)",
    )?;
    f.pin(
        "C-F7.model.tie.customer4_reads",
        list(&reads_by(&tie, 4)),
        "4,3,0,2",
        "fact sheet C-F7 (04_2)",
    )?;
    f.pin(
        "C-F7.model.tie.slates_after",
        list(&st2.number),
        "4,3,5,5,2",
        "fact sheet C-F7 (04_2)",
    )?;
    let orders2 = service_orders(&FIVE, st2, &[1, 2, 3, 4, 5]);
    f.pin(
        "C-F7.model.tie.distinct_service_orders",
        orders2.len(),
        "1",
        "fact sheet C-F7",
    )?;
    f.pin(
        "C-F7.model.tie.service_order",
        first(&orders2),
        "5,2,1,3,4",
        "fact sheet C-F7 (04_6)",
    )?;
    // 11: the ticket bound
    let ms_per_year: u64 = 1_000 * 86_400 * 36_525 / 100;
    f.pin("C-F10.ms_per_year", ms_per_year, "31557600000", "fact sheet C-F10")?;
    f.pin("C-F10.two_pow_35", 1u64 << 35, "34359738368", "fact sheet C-F10")?;
    f.pin(
        "C-F10.ms_per_year_below_two_pow_35",
        u8::from(ms_per_year < 1 << 35),
        "1",
        "fact sheet C-F10",
    )?;
    f.pin(
        "C-F10.u32_wrap_days_at_1_per_ms_1dp",
        format!("{:.1}", (1u64 << 32) as f64 / 86_400_000.0),
        "49.7",
        "R1 Dev dev-02",
    )?;
    f.pin(
        "C-F10.i32_wrap_days_at_1_per_ms_1dp",
        format!("{:.1}", (1u64 << 31) as f64 / 86_400_000.0),
        "24.9",
        "claim walk (signed, as in Java's int)",
    )?;
    f.pin(
        "C-F11.u64_years_at_1e9_per_s_1dp",
        format!("{:.1}", 2f64.powi(64) / 1e9 / (365.25 * 86_400.0)),
        "584.5",
        "fact sheet C-F11",
    )?;
    f.pin(
        "C-F11.i64_years_at_1e9_per_s_1dp",
        format!("{:.1}", 2f64.powi(63) / 1e9 / (365.25 * 86_400.0)),
        "292.3",
        "claim walk (signed, as in Java's long)",
    )?;
    // 11: reads of other customers' flags and slates per entry, alone
    f.pin(
        "C-F12.other_slate_reads_alone.model.n5",
        3 * (5 - 1),
        "12",
        "fact sheet C-F12",
    )?;
    // cited (fact sheet C-F14)
    f.pin(
        "C-F14.dijkstra_problem.year",
        cited::DIJKSTRA_YEAR,
        "1965",
        "CACM 8(9):569",
    )?;
    f.pin(
        "C-F14.knuth_eventual_entry.year",
        cited::KNUTH_YEAR,
        "1966",
        "CACM 9(5):321",
    )?;
    f.pin("C-F14.bakery_paper.year", cited::BAKERY_YEAR, "1974", "CACM 17(8):453")?;
    f.pin(
        "C-F14.turing_lecture.year",
        cited::TURING_LECTURE_YEAR,
        "2015",
        "CACM 58(6):71",
    )?;
    f.pin(
        "C-F14.considered_impossible.year",
        cited::CONSIDERED_IMPOSSIBLE_YEAR,
        "1973",
        "Turing lecture p. 4",
    )?;
    f.pin(
        "C-F14.turing_lecture_garbage.from",
        cited::TORN_FROM,
        "9",
        "Turing lecture",
    )?;
    f.pin(
        "C-F14.turing_lecture_garbage.to",
        cited::TORN_TO,
        "10",
        "Turing lecture",
    )?;
    f.pin(
        "C-F14.turing_lecture_garbage.value",
        cited::TORN_VALUE,
        "2496",
        "Turing lecture (F8)",
    )?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slate_examples_follow_the_rules() {
        // the plan's original example: customer 4 reads 0, 3, 0, 2 and writes 4
        assert_eq!(take_ticket(&SLATES, 4), 4);
        let mut after = SLATES;
        after[3] = 4;
        assert_eq!(service_order(&after), [5, 2, 4]);
        // ties go to the lower customer number (04_4): Purple 3 before Teal 4
        assert_eq!(service_order(&[0, 0, 5, 5, 0]), [3, 4]);
        // 04_5: Teal holds 2, Blue holds 3, so Teal goes first
        assert_eq!(service_order(&[3, 0, 0, 2, 0]), [4, 1]);
    }

    #[test]
    fn the_chalk_rule_and_the_model_agree() {
        // 03_4–03_5: by hand and replayed, the same reads, ticket and order
        let blue = blue_takes_a_ticket();
        let st1 = &blue.last().expect("steps").after;
        assert_eq!(st1.number[0], take_ticket(&SLATES, 1));
        assert_eq!(service_order(&st1.number), [5, 2, 1]);
        // 04_6: the room takes the smallest pair of everyone holding a ticket
        let tie = purple_and_teal_tie();
        let st2 = &tie.last().expect("steps").after;
        assert_eq!(service_order(&st2.number), [5, 2, 1, 3, 4]);
    }
}
