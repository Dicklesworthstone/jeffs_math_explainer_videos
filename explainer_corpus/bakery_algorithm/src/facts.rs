//! Every number Video C shows or says that plain arithmetic can recompute,
//! computed here and checked two ways: `pin` against a literal from the fact
//! sheet, and `./x bakery_algorithm facts-check` against `oracle.py` (same
//! keys). The model checker's counts (orders, states, traces) stay the
//! oracle's: the chapters replay its recorded traces (plan §11.7).

use explainer_kit::{Facts, choose};

/// The slates of 03_4: customers 1–5 (Blue, Green, Purple, Teal, Pink).
pub const SLATES: [u64; 5] = [0, 3, 0, 0, 2];

/// Customer `me` (1-based) takes 1 + the biggest other slate.
pub fn take_ticket(slates: &[u64], me: usize) -> u64 {
    1 + slates
        .iter()
        .enumerate()
        .filter(|&(i, _)| i + 1 != me)
        .map(|(_, &t)| t)
        .max()
        .unwrap_or(0)
}

/// Who is served first: nonzero tickets in (ticket, customer number) order.
pub fn service_order(slates: &[u64]) -> Vec<usize> {
    let mut held: Vec<(u64, usize)> = slates
        .iter()
        .enumerate()
        .filter(|(_, t)| **t > 0)
        .map(|(i, &t)| (t, i + 1))
        .collect();
    held.sort_unstable();
    held.into_iter().map(|(_, k)| k).collect()
}

fn list<T: ToString>(v: &[T]) -> String {
    v.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    // 01: two increments, a read and a write each
    f.pin("C-F1.orders", choose(4, 2), "6", "fact sheet C-F1")?;
    f.pin("C-F1.score_end_when_a_point_is_lost", 41 + 1, "42", "fact sheet C-F1")?;
    f.pin("C-F1.score_end_otherwise", 41 + 2, "43", "fact sheet C-F1")?;
    // 03: Blue reads 3, 0, 0, 2 and takes 4
    let reads: Vec<u64> = SLATES
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 0)
        .map(|(_, &t)| t)
        .collect();
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
        "C-F11.u64_years_at_1e9_per_s_1dp",
        format!("{:.1}", 2f64.powi(64) / 1e9 / (365.25 * 86_400.0)),
        "584.5",
        "fact sheet C-F11",
    )?;
    // 11: reads of other customers' flags and slates per entry, alone
    f.pin(
        "C-F12.other_slate_reads_alone.model.n5",
        3 * (5 - 1),
        "12",
        "fact sheet C-F12",
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
}
