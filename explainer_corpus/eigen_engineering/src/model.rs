//! The mathematics of Video K, one module per system the video visits. Pure
//! computation with unit tests; the chapters only draw.
//!
//! - [`linalg`]: a square matrix, Gaussian elimination, and symmetric
//!   eigenpairs by cyclic Jacobi rotations.
//! - [`carts`]: two carts and three springs (Act I), and the chain of carts
//!   that a tube of air turns out to be.
//! - [`truss`]: a Pratt truss in steel by the stiffness method (Act II).
//! - [`rooms`]: the modes of a box of air, counted exactly, and of a room
//!   that is not a box, cut into cells (Act III).
//! - [`economy`]: a two-industry input-output table in exact fractions
//!   (Act IV).
//! - [`factory`]: the linear program that is not an eigenproblem.

pub mod carts;
pub mod economy;
pub mod factory;
pub mod linalg;
pub mod rooms;
pub mod truss;

/// One "your turn" quiz: the storyboard line that asks it and the seconds
/// its countdown runs. The answers are computed in `facts.rs`
/// (`K-Q.<line>.answer`), never typed here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quiz {
    pub line: &'static str,
    pub seconds: u32,
}

/// Every quiz, in video order.
pub const QUIZZES: &[Quiz] = &[
    Quiz {
        line: "02_6",
        seconds: 5,
    },
    Quiz {
        line: "04_5",
        seconds: 5,
    },
    Quiz {
        line: "06_3",
        seconds: 5,
    },
    Quiz {
        line: "07_4",
        seconds: 4,
    },
    Quiz {
        line: "09_4",
        seconds: 6,
    },
    Quiz {
        line: "11_3",
        seconds: 5,
    },
    Quiz {
        line: "12_2",
        seconds: 6,
    },
];

/// A calendar date as days since 1 March of year 0 (the proleptic Gregorian
/// calendar; Howard Hinnant's `days_from_civil`), for "four months old" and
/// "two days later".
pub fn day_number(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_numbers_count_days() {
        assert_eq!(day_number(2000, 6, 12) - day_number(2000, 6, 10), 2);
        assert_eq!(day_number(1940, 11, 7) - day_number(1940, 7, 1), 129);
        assert_eq!(day_number(2001, 1, 1) - day_number(2000, 1, 1), 366);
    }
}
