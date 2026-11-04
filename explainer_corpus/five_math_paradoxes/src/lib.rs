//! Models behind the five-paradoxes explainer.
//!
//! These values are a starting point for real scenes; the existence of this
//! crate does not mean that a video has been rendered or reviewed.

/// One equally likely car placement, under a host who always reveals a goat
/// and chooses the lowest numbered legal door when there is a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MontyCase {
    pub car: u8,
    pub initial: u8,
    pub revealed: u8,
    pub switched: u8,
    pub switch_wins: bool,
}

/// The three possible car placements, fixing the contestant's first choice.
pub fn monty_cases() -> [MontyCase; 3] {
    std::array::from_fn(|index| {
        let car = (index + 1) as u8;
        let initial = 1;
        let revealed = (1..=3).find(|&door| door != initial && door != car).unwrap();
        let switched = (1..=3).find(|&door| door != initial && door != revealed).unwrap();
        MontyCase {
            car,
            initial,
            revealed,
            switched,
            switch_wins: switched == car,
        }
    })
}

/// Probability of at least one shared birthday under independent, uniformly
/// distributed birthdays on 365 days. This is a model, not a real-room survey.
pub const BIRTHDAY_DAYS: usize = 365;

pub fn birthday_match_probability(people: usize) -> f64 {
    if people > BIRTHDAY_DAYS {
        return 1.0;
    }
    let distinct = (0..people).fold(1.0, |p, used| p * (BIRTHDAY_DAYS - used) as f64 / BIRTHDAY_DAYS as f64);
    1.0 - distinct
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub passed: u32,
    pub total: u32,
}

/// Fictional easy/hard groups, in that order, for machines A and B.
pub fn inspection_groups() -> ([Group; 2], [Group; 2]) {
    (
        [Group { passed: 9, total: 10 }, Group { passed: 18, total: 90 }],
        [Group { passed: 72, total: 90 }, Group { passed: 1, total: 10 }],
    )
}

/// Compare two rates exactly without rounding, using widened integer products.
pub fn higher_rate(a: Group, b: Group) -> Option<bool> {
    if a.total == 0 || b.total == 0 || a.passed > a.total || b.passed > b.total {
        return None;
    }
    Some(u64::from(a.passed) * u64::from(b.total) > u64::from(b.passed) * u64::from(a.total))
}

/// Combine disjoint groups; return None on overflow or invalid input.
pub fn combine(groups: &[Group]) -> Option<Group> {
    groups.iter().try_fold(Group { passed: 0, total: 0 }, |sum, group| {
        if group.passed > group.total {
            return None;
        }
        Some(Group {
            passed: sum.passed.checked_add(group.passed)?,
            total: sum.total.checked_add(group.total)?,
        })
    })
}

/// Exact positive rational time to close a head start at constant speeds.
pub fn catch_time(head_start: u64, pursuer_speed: u64, other_speed: u64) -> Option<(u64, u64)> {
    let relative = pursuer_speed.checked_sub(other_speed)?;
    if relative == 0 {
        return None;
    }
    let divisor = gcd(head_start, relative);
    Some((head_start / divisor, relative / divisor))
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Destination of an original hotel guest; room numbering starts at one.
pub fn moved_room(original: u64) -> Option<u64> {
    if original == 0 {
        return None;
    }
    original.checked_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_reveals_a_goat_and_switching_wins_two_of_three() {
        let cases = monty_cases();
        for case in cases {
            assert_ne!(case.revealed, case.car);
            assert_ne!(case.revealed, case.initial);
            assert_ne!(case.switched, case.initial);
            assert_ne!(case.switched, case.revealed);
        }
        assert_eq!(cases.iter().filter(|case| case.switch_wins).count(), 2);
    }

    #[test]
    fn birthday_threshold_and_boundaries() {
        assert_eq!(birthday_match_probability(0), 0.0);
        assert_eq!(birthday_match_probability(1), 0.0);
        assert!(birthday_match_probability(22) < 0.5);
        assert!((birthday_match_probability(23) - 0.507_297_234_323_985_4).abs() < 1e-12);
        assert_eq!(birthday_match_probability(366), 1.0);
    }

    #[test]
    fn within_group_and_aggregate_rankings_reverse() {
        let (a, b) = inspection_groups();
        assert_eq!(higher_rate(a[0], b[0]), Some(true));
        assert_eq!(higher_rate(a[1], b[1]), Some(true));
        assert_eq!(combine(&a), Some(Group { passed: 27, total: 100 }));
        assert_eq!(combine(&b), Some(Group { passed: 73, total: 100 }));
        assert_eq!(higher_rate(combine(&a).unwrap(), combine(&b).unwrap()), Some(false));
        assert_eq!(higher_rate(Group { passed: 0, total: 0 }, b[0]), None);
    }

    #[test]
    fn meeting_and_room_mapping_cover_edge_cases() {
        assert_eq!(catch_time(100, 10, 1), Some((100, 9)));
        assert_eq!(catch_time(0, 10, 1), Some((0, 1)));
        assert_eq!(catch_time(100, 1, 1), None);
        assert_eq!(catch_time(100, 1, 10), None);
        assert_eq!(moved_room(0), None);
        assert_eq!(moved_room(u64::MAX), None);
        for n in 1..10_000 {
            assert_eq!(moved_room(n), Some(n + 1));
        }
    }
}
