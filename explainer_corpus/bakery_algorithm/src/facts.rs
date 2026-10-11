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
    /// The bakery paper's citation, and the pages of the two lines quoted
    /// on screen: "The read may return any arbitrary value!" (10_3) and "a
    /// read of number[i] can never obtain a value larger than one which has
    /// been written there" (11_6).
    pub const BAKERY_CITATION: &str = "CACM 17(8):453-455, Aug 1974";
    pub const ARBITRARY_VALUE_PAGE: u32 = 454;
    pub const NEVER_LARGER_PAGE: u32 = 455;
    /// "Reasoning About Nonatomic Operations" (POPL 10, 1983): read each
    /// slate once (10_4).
    pub const NONATOMIC_OPERATIONS_YEAR: u32 = 1983;
    /// "Time, Clocks, and the Ordering of Events" (CACM 21(7), 1978; 13_3).
    pub const TIME_CLOCKS_YEAR: u32 = 1978;
    /// The Turing lecture's citation, and the section of "Although of little
    /// if any practical use…" (13_1).
    pub const TURING_CITATION: &str = "CACM 58(6):71-76, June 2015";
    pub const PRACTICAL_USE_SECTION: &str = "2.2";
    /// Abdulla et al., "Counter-Example Guided Fence Insertion under TSO"
    /// (LNCS 7214, pp. 204-219): two fences per process (12_5).
    pub const ABDULLA_TACAS: &str = "TACAS 2012";
    /// The Computer History Museum oral history (Levin with Lamport), and
    /// the pages of "…perhaps that's why I'm proudest of it" (13_4).
    pub const CHM_YEAR: u32 = 2016;
    pub const CHM_PAGES: &str = "9-10";

    /// The journal part of a citation: "CACM 17(8)".
    fn journal(citation: &'static str) -> &'static str {
        citation.split(':').next().unwrap_or(citation)
    }

    pub fn bakery_journal() -> &'static str {
        journal(BAKERY_CITATION)
    }

    pub fn turing_journal() -> &'static str {
        journal(TURING_CITATION)
    }
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
    facts_10_13(&mut f)?;
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

// ------------------------------------------------------- chapters 10 to 13

/// A pinned fact's value, for a chapter to print: the facts are built once,
/// so what a chapter shows is the very string `facts-check` compares.
///
/// # Panics
/// On a key the facts do not register (every chapter's keys are covered by
/// the tests), or if the facts fail their own pins.
pub fn fact(key: &str) -> &'static str {
    static FACTS: std::sync::OnceLock<Facts> = std::sync::OnceLock::new();
    FACTS
        .get_or_init(|| facts().unwrap_or_else(|e| panic!("the facts fail their pins: {e}")))
        .get(key)
        .unwrap_or_else(|| panic!("no fact {key}"))
}

/// The same, parsed as a number.
pub fn fact_u64(key: &str) -> u64 {
    fact(key)
        .parse()
        .unwrap_or_else(|e| panic!("fact {key} is not a whole number: {e}"))
}

/// The x86-TSO model was checked with up to this many customers (C-F9,
/// `tso3.py`'s N = 3, tickets ≤ 3, outboxes of 3).
pub const TSO_CHECKED_CUSTOMERS: usize = 3;

/// The three moments 10_2 reads off trace e3 (1-based step numbers), and
/// the glance that lets Green in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct E3Moments {
    /// Green's flag is down: she has finished choosing.
    pub green_done_choosing: usize,
    /// Green's check finds Blue's flag down.
    pub green_saw_flag_down: usize,
    /// Blue starts raising her flag again, for the ticket Green glances at.
    pub blue_raises_again: usize,
    /// Green's glance at Blue's slate in the middle of Blue's write.
    pub glance: usize,
}

/// # Panics
/// If `steps` is not trace e3 (the moments are looked up by label).
pub fn e3_moments(steps: &[Step]) -> E3Moments {
    let first = |who: usize, label: &str| {
        steps
            .iter()
            .position(|s| s.customer == who && s.label == label)
            .unwrap_or_else(|| panic!("trace e3 has no {label:?} by customer {who}"))
            + 1
    };
    let blue_raises_again = steps
        .iter()
        .rposition(|s| s.customer == 1 && s.label == "begin choosing:=1")
        .expect("Blue raises her flag in trace e3")
        + 1;
    let glance = steps
        .iter()
        .position(|s| s.customer == 2 && s.label.starts_with("L3: number[1]=") && s.enters())
        .expect("Green enters on a glance at Blue's slate in trace e3")
        + 1;
    E3Moments {
        green_done_choosing: first(2, "end choosing:=0"),
        green_saw_flag_down: first(2, "L2: choosing[1]=0, pass"),
        blue_raises_again,
        glance,
    }
}

/// The measured runs of the 12_7 panel's lock (`bakery_stress`, bead
/// mx-e-c-7v0.17): on the Arm build host, and on an x86 host.
const STRESS_ARM: &str =
    include_str!("../../../docs/research/bakery_algorithm/scripts/outputs/bakery_stress_m5_ultra.txt");
const STRESS_X86: &str =
    include_str!("../../../docs/research/bakery_algorithm/scripts/outputs/bakery_stress_x86_threadripper.txt");

/// Which machine a measured record comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arch {
    Arm,
    X86,
}

impl Arch {
    /// The tag the oracle's keys carry (`C-M1.<tag>.*`).
    pub fn tag(self) -> &'static str {
        match self {
            Self::Arm => "arm",
            Self::X86 => "x86",
        }
    }

    fn record(self) -> &'static str {
        match self {
            Self::Arm => STRESS_ARM,
            Self::X86 => STRESS_X86,
        }
    }
}

/// One mode of one recorded block: its totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeasuredMode {
    /// `<arch>.n<threads>_r<runs>`, as the oracle names the block.
    pub tag: String,
    pub mode: String,
    pub threads: u64,
    pub runs: u64,
    pub entries_per_run: u64,
    pub entries: u64,
    pub lost: u64,
}

/// The whole record of one machine: its CPU and every block's modes.
#[derive(Clone, Debug)]
pub struct Measured {
    pub arch: Arch,
    pub cpu: String,
    pub modes: Vec<MeasuredMode>,
}

impl Measured {
    /// `mode` in the ten-run block with `threads` customers.
    ///
    /// # Panics
    /// If the record has no such block.
    pub fn mode(&self, mode: &str, threads: u64) -> &MeasuredMode {
        self.modes
            .iter()
            .find(|m| m.mode == mode && m.threads == threads && m.runs == 10)
            .unwrap_or_else(|| panic!("no recorded {mode} run with {threads} threads"))
    }

    /// The long Release/Acquire block (two customers, a hundred runs), if
    /// this machine has one.
    pub fn long_relacq(&self) -> Option<&MeasuredMode> {
        self.modes.iter().find(|m| m.mode == "relacq" && m.runs == 100)
    }
}

/// A machine's record, parsed: each block's header names its CPU and
/// command; its `total` lines carry each mode's sums.
///
/// # Panics
/// If the committed record is malformed (the tests parse it).
pub fn measured(arch: Arch) -> Measured {
    let mut cpu = String::new();
    let mut modes = Vec::new();
    let mut opts: Vec<(String, String)> = Vec::new();
    let mut runs_seen = 0;
    for line in arch.record().lines() {
        if let Some(c) = line.strip_prefix("# cpu: ") {
            cpu = c.split(" (").next().unwrap_or(c).to_owned();
        } else if let Some(c) = line.strip_prefix("# command: bakery_stress ") {
            let a: Vec<&str> = c.split_whitespace().collect();
            opts = a.chunks(2).map(|p| (p[0].to_owned(), p[1].to_owned())).collect();
            runs_seen = 0;
        } else if !line.is_empty() && !line.starts_with('#') && !line.starts_with("mode") {
            let f: Vec<&str> = line.split('\t').collect();
            let num = |i: usize| f[i].parse::<u64>().expect("a recorded count");
            if f[3] == "total" {
                let opt = |k: &str| {
                    opts.iter()
                        .find(|(n, _)| n == k)
                        .map(|(_, v)| v.parse::<u64>().expect("a numeric option"))
                        .expect("the command names its sizes")
                };
                let (threads, per, runs) = (opt("--threads"), opt("--entries"), opt("--runs"));
                assert_eq!(runs_seen, runs, "a recorded block has every run");
                modes.push(MeasuredMode {
                    tag: format!("{}.n{threads}_r{runs}", arch.tag()),
                    mode: f[0].to_owned(),
                    threads,
                    runs,
                    entries_per_run: threads * per,
                    entries: num(4),
                    lost: num(6),
                });
                runs_seen = 0;
            } else {
                runs_seen += 1;
            }
        }
    }
    Measured { arch, cpu, modes }
}

/// Two customers, one visit each, over every order: how many orders have
/// both flags up at once, who can go in first then, and whether, when one
/// finishes choosing before the other starts, she always goes in first
/// (C-X10, 11_1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoosingOrder {
    pub overlap: usize,
    pub first_can_be: Vec<usize>,
    pub disjoint: usize,
    pub first_finisher_first: bool,
}

/// # Panics
/// If an order does not replay on the two bakery programs (the orders come
/// from the same programs).
pub fn choosing_order(orders: &[Order]) -> ChoosingOrder {
    let progs = [
        bakery_program(2, 0, true, Cmp::Lex),
        bakery_program(2, 1, true, Cmp::Lex),
    ];
    let mut out = ChoosingOrder {
        overlap: 0,
        first_can_be: Vec::new(),
        disjoint: 0,
        first_finisher_first: true,
    };
    for o in orders {
        let (mut mem, mut ms, mut pcs) = (vec![0u32; 4], [0u32; 2], [0usize; 2]);
        let (mut overlapped, mut first_in, mut first_chooser) = (false, None, None);
        for &i in &o.moves {
            let op = progs[i][pcs[i]];
            let (m2, v) = model::apply(op, i, 2, &mem, ms[i]).expect("a listed order replays");
            mem = m2;
            ms[i] = v;
            if op == model::Op::SetFlag(0) && first_chooser.is_none() {
                first_chooser = Some(i);
            }
            pcs[i] += 1;
            overlapped |= mem[0] == 1 && mem[1] == 1;
            if first_in.is_none() && pcs[i] == progs[i].len() - 1 {
                first_in = Some(i);
            }
        }
        let first = first_in.expect("someone goes in") + 1;
        if overlapped {
            out.overlap += 1;
            if !out.first_can_be.contains(&first) {
                out.first_can_be.push(first);
            }
        } else {
            out.disjoint += 1;
            out.first_finisher_first &= first_chooser.map(|c| c + 1) == Some(first);
        }
    }
    out.first_can_be.sort_unstable();
    out
}

/// 11_2's "at most once" (C-X5, three customers, tickets ≤ 4, atomic).
pub fn bypass_worst() -> u8 {
    model::bypass_bfs(&model::Bakery::new(3, 4, Variant::Correct, false, true)).1
}

/// 11_7's "after at most one more visit each" (C-X8, the same bound).
pub fn frozen_worst() -> u8 {
    let m = model::Bakery::new(3, 4, Variant::Correct, false, true);
    let (choosing, _) = model::customer1_choosing_and_in_line(&model::reachable(&m));
    model::frozen_bfs(&m, &choosing, 0).worst
}

/// 13_3's grid: the pairs of 04_2's tie (slates 4, 3, 5, 5, 2), as
/// (ticket, customer number).
pub fn tie_pairs() -> Vec<(u32, usize)> {
    let steps = purple_and_teal_tie();
    let last = &steps.last().expect("the tie runs").after;
    last.number
        .iter()
        .enumerate()
        .filter(|(_, t)| **t > 0)
        .map(|(i, &t)| (t, i + 1))
        .collect()
}

fn facts_10_13(f: &mut Facts) -> Result<(), String> {
    use crate::model::tso::{self, Tso};
    // 10_2: the three moments of trace e3
    let e3 = Trace::E3.steps();
    let at = e3_moments(&e3);
    f.pin(
        "C-F6.e3.customer2_finished_choosing_at_step",
        at.green_done_choosing,
        "14",
        "fact sheet C-F6 (M6)",
    )?;
    f.pin(
        "C-F6.e3.customer2_saw_customer1_flag_down_at_step",
        at.green_saw_flag_down,
        "17",
        "fact sheet C-F6 (M6)",
    )?;
    f.pin(
        "C-F6.e3.customer1_began_raising_flag_again_at_step",
        at.blue_raises_again,
        "21",
        "fact sheet C-F6 (M6)",
    )?;
    // 12: the x86-TSO model, ported (model::tso)
    let voiced = Tso::new(2, 4, false, false);
    let (states, moves) = voiced.replay(&tso::VOICED)?;
    let end = states.last().ok_or("the voiced order replays")?;
    f.pin("C-F9.voiced.steps", moves.len(), "12", "fact sheet C-F9 (R11)")?;
    f.pin(
        "C-F9.voiced.in_room_at_end",
        voiced.in_room(end),
        "2",
        "fact sheet C-F9",
    )?;
    f.pin(
        "C-F9.voiced.memory_choosing_at_end",
        list(&end.flags),
        "0,0",
        "fact sheet C-F9",
    )?;
    f.pin(
        "C-F9.voiced.memory_tickets_at_end",
        list(&end.slates),
        "0,0",
        "fact sheet C-F9",
    )?;
    let boxes: Vec<String> = end
        .outbox
        .iter()
        .map(|b| b.iter().map(tso::Store::short).collect::<Vec<_>>().join(","))
        .collect();
    f.pin(
        "C-F9.voiced.outboxes_at_end",
        boxes.join(";"),
        "c1=1,n1=1,c1=0;c2=1,n2=1,c2=0",
        "fact sheet C-F9",
    )?;
    for (tag, f1, f2, states, shortest) in [
        ("no_fence", false, false, "48646", Some("12")),
        ("f1_only", true, false, "2298", Some("18")),
        ("f2_only", false, true, "7484", Some("20")),
        ("f1_f2", true, true, "1135", None),
    ] {
        let (n, bad) = Tso::new(2, 4, f1, f2).explore();
        f.pin(
            &format!("C-F9.{tag}.states"),
            n,
            states,
            "fact sheet C-F9 (bakery_tso_out.txt)",
        )?;
        f.pin(
            &format!("C-F9.{tag}.violation_found"),
            bit(bad.is_some()),
            if shortest.is_some() { "1" } else { "0" },
            "fact sheet C-F9",
        )?;
        if let (Some(len), Some(want)) = (bad, shortest) {
            f.pin(
                &format!("C-F9.{tag}.shortest_violation_steps"),
                len,
                want,
                "fact sheet C-F9",
            )?;
        }
    }
    let (n, bad) = Tso::new(2, 6, true, true).explore();
    f.pin("C-F9.f1_f2.n2_k6_b4.states", n, "1811", "fact sheet C-F9 (tso3.txt)")?;
    f.pin(
        "C-F9.f1_f2.n2_k6_b4.violation_found",
        bit(bad.is_some()),
        "0",
        "fact sheet C-F9",
    )?;
    let mut n3 = Tso::new(TSO_CHECKED_CUSTOMERS, 3, true, true);
    n3.b = 3;
    let (n, bad) = n3.explore();
    f.pin("C-F9.f1_f2.n3_k3_b3.states", n, "37580", "fact sheet C-F9 (tso3.txt)")?;
    f.pin(
        "C-F9.f1_f2.n3_k3_b3.violation_found",
        bit(bad.is_some()),
        "0",
        "fact sheet C-F9",
    )?;
    let mut ooo = Tso::new(2, 4, true, true);
    ooo.reorder = true;
    let (n, bad) = ooo.explore();
    f.pin("C-F9.ooo_outbox.f1_f2.states", n, "1673", "fact sheet C-F9 (R2 dev-02)")?;
    f.pin(
        "C-F9.ooo_outbox.f1_f2.violation_found",
        bit(bad.is_some()),
        "1",
        "fact sheet C-F9",
    )?;
    f.pin(
        "C-F9.ooo_outbox.f1_f2.shortest_violation_steps",
        bad.ok_or("the reordering outbox fails")?,
        "22",
        "fact sheet C-F9",
    )?;
    // 11: fairness and its edges (BFS ports of the oracle's checks)
    let co = choosing_order(&bakery_orders(true, Cmp::Lex));
    f.pin(
        "C-X10.choosing_overlaps.n2_one_visit.orders",
        co.overlap,
        "272",
        "fact sheet C-X10",
    )?;
    f.pin(
        "C-X10.choosing_overlaps.n2_one_visit.first_to_enter_can_be",
        list(&co.first_can_be),
        "1,2",
        "fact sheet C-X10 (11_1)",
    )?;
    f.pin(
        "C-X10.choosing_disjoint.n2_one_visit.orders",
        co.disjoint,
        "50",
        "fact sheet C-X10",
    )?;
    f.pin(
        "C-X10.choosing_disjoint.n2_one_visit.first_to_finish_choosing_enters_first",
        bit(co.first_finisher_first),
        "1",
        "fact sheet C-X10 (11_1)",
    )?;
    let m34 = model::Bakery::new(3, 4, Variant::Correct, false, true);
    let (ghosts, worst) = model::bypass_bfs(&m34);
    f.pin(
        "C-X5.bypass_from_ticket_write.n3_k4_atomic.ghost_states",
        ghosts,
        "45868",
        "fact sheet C-X5",
    )?;
    f.pin(
        "C-X5.bypass_from_ticket_write.n3_k4_atomic.worst",
        worst,
        "1",
        "fact sheet C-X5 (11_2)",
    )?;
    let all = model::reachable(&m34);
    let (choosing, in_line) = model::customer1_choosing_and_in_line(&all);
    let frozen = model::frozen_bfs(&m34, &choosing, 0);
    f.pin(
        "C-X8.frozen_while_choosing.n3_k4_atomic.start_states",
        choosing.len(),
        "9763",
        "fact sheet C-X8",
    )?;
    f.pin(
        "C-X8.frozen_while_choosing.n3_k4_atomic.worst_entries_per_other_customer",
        frozen.worst,
        "1",
        "fact sheet C-X8 (11_7)",
    )?;
    let paused = model::frozen_bfs(&m34, &in_line, 0);
    f.pin(
        "C-X9.paused_in_line.n3_k4_atomic.start_states",
        in_line.len(),
        "7098",
        "fact sheet C-X9",
    )?;
    f.pin(
        "C-X9.paused_in_line.n3_k4_atomic.worst_entries_per_other_customer",
        paused.worst,
        "1",
        "fact sheet C-X9",
    )?;
    f.pin(
        "C-X9.paused_in_line.n3_k4_atomic.entries_by_customers_behind_her",
        paused.behind,
        "0",
        "fact sheet C-X9 (11 costs card)",
    )?;
    let (drops, to_nonzero) = model::largest_ticket_drops(&model::Bakery::new(3, 5, Variant::Correct, false, true));
    f.pin(
        "C-X7.largest_ticket_drops.n3_k5_atomic",
        drops,
        "545",
        "fact sheet C-X7",
    )?;
    f.pin(
        "C-X7.largest_ticket_drops_to_nonzero.n3_k5_atomic",
        to_nonzero,
        "0",
        "fact sheet C-X7 (11_3)",
    )?;
    // 11: the cards' arithmetic
    f.pin(
        "C-F10.days_per_year",
        format!("{}", 36_525.0 / 100.0),
        "365.25",
        "Lamport's year at 1/ms",
    )?;
    f.pin("C-F10.u32_max", u32::MAX, "4294967295", "2^32 - 1")?;
    f.pin(
        "C-F10.ms_per_year_billions_to_nearest_half",
        format!("{}", (1_000.0 * 86_400.0 * 365.25 / 1e9 * 2.0_f64).round() / 2.0),
        "31.5",
        "fact sheet C-F10 (M7)",
    )?;
    f.pin("C-F11.huge_glance", 1u64 << 63, "9223372036854775808", "2^63")?;
    f.pin(
        "C-F11.ticket_after_huge_glance",
        (1u64 << 63) + 1,
        "9223372036854775809",
        "2^63 + 1",
    )?;
    // 12: the measured records (C-M1, RECORDED; the oracle parses the same
    // files on its own)
    for arch in [Arch::Arm, Arch::X86] {
        let rec = measured(arch);
        f.add(&format!("C-M1.{}.cpu", arch.tag()), &rec.cpu)?;
        let mut tags: Vec<(String, u64)> = Vec::new();
        for m in &rec.modes {
            if !tags.iter().any(|(t, _)| t == &m.tag) {
                tags.push((m.tag.clone(), m.threads));
            }
        }
        for (tag, threads) in &tags {
            f.add(&format!("C-M1.{tag}.threads"), threads)?;
            for m in rec.modes.iter().filter(|m| &m.tag == tag) {
                f.add(&format!("C-M1.{tag}.{}.runs", m.mode), m.runs)?;
                f.add(&format!("C-M1.{tag}.{}.entries_per_run", m.mode), m.entries_per_run)?;
                f.add(&format!("C-M1.{tag}.{}.entries", m.mode), m.entries)?;
                f.add(&format!("C-M1.{tag}.{}.lost", m.mode), m.lost)?;
            }
        }
    }
    let (arm, x86) = (measured(Arch::Arm), measured(Arch::X86));
    f.pin(
        "C-M1.arm.n2_r10.relaxed.lost",
        arm.mode("relaxed", 2).lost,
        "2314899",
        "mx-e-c-7v0.17 (48f3e37)",
    )?;
    f.pin(
        "C-M1.x86.n2_r10.relacq.lost",
        x86.mode("relacq", 2).lost,
        "1173616",
        "mx-e-c-7v0.17 (eb668c7)",
    )?;
    f.pin(
        "C-M1.x86.n2_r10.relaxed.lost",
        x86.mode("relaxed", 2).lost,
        "1006846",
        "mx-e-c-7v0.17 (eb668c7)",
    )?;
    // cited (fact sheet C-F14, C-S21, C-S26, C-S27)
    f.pin(
        "C-F14.bakery_paper.citation",
        cited::BAKERY_CITATION,
        "CACM 17(8):453-455, Aug 1974",
        "fact sheet C-F14",
    )?;
    f.pin(
        "C-F14.bakery_paper.arbitrary_value_quote_page",
        cited::ARBITRARY_VALUE_PAGE,
        "454",
        "bakery.pdf p. 454 (fact sheet open item 3)",
    )?;
    f.pin(
        "C-F14.bakery_paper.never_larger_quote_page",
        cited::NEVER_LARGER_PAGE,
        "455",
        "bakery.pdf p. 455 (the C-F10 bound)",
    )?;
    f.pin(
        "C-F14.nonatomic_operations.year",
        cited::NONATOMIC_OPERATIONS_YEAR,
        "1983",
        "POPL 10 (C-F14)",
    )?;
    f.pin(
        "C-F14.time_clocks.year",
        cited::TIME_CLOCKS_YEAR,
        "1978",
        "CACM 21(7) (C-F14)",
    )?;
    f.pin(
        "C-F14.turing_lecture.citation",
        cited::TURING_CITATION,
        "CACM 58(6):71-76, June 2015",
        "fact sheet C-F14",
    )?;
    f.pin(
        "C-F14.turing_lecture.practical_use_quote_section",
        cited::PRACTICAL_USE_SECTION,
        "2.2",
        "C-S26",
    )?;
    f.pin(
        "C-F14.abdulla_tso_fences.citation",
        cited::ABDULLA_TACAS,
        "TACAS 2012",
        "C-S21; LNCS 7214",
    )?;
    f.pin("C-F14.chm_oral_history.year", cited::CHM_YEAR, "2016", "C-S27")?;
    f.pin("C-F14.chm_oral_history.pages", cited::CHM_PAGES, "9-10", "F4, C-S27")?;
    Ok(())
}
