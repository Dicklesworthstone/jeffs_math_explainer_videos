//! Act I: two carts between two walls, joined by three equal springs. The
//! rule that turns a pair of moves into the pair of pushes that holds them
//! there is the smallest stiffness table worth drawing, and its two own
//! patterns can be found by hand.

use super::linalg::{Eigen, Mat, dot, jacobi_eigh, solve};

/// The spring every example uses for a vibration number, newtons a metre.
pub const SPRING_N_PER_M: f64 = 100.0;
/// A cart's mass, kilograms.
pub const CART_KG: f64 = 1.0;

/// The pushes that hold `n` carts at the moves `u`, in units of one spring:
/// a cart is pulled back by the spring on each side of it, and each of
/// those springs is also stretched by the neighbour's move.
pub fn chain(n: usize) -> Mat {
    let mut k = Mat::zeros(n);
    for i in 0..n {
        k.add(i, i, 2.0);
        if i + 1 < n {
            k.add(i, i + 1, -1.0);
            k.add(i + 1, i, -1.0);
        }
    }
    k
}

/// The two carts' table: [[2, −1], [−1, 2]].
pub fn table() -> Mat {
    chain(2)
}

/// The pushes that hold the carts at the moves `u`.
pub fn pushes(u: [f64; 2]) -> [f64; 2] {
    let f = table().mul_vec(&u);
    [f[0], f[1]]
}

/// The moves a pair of pushes `f` causes.
pub fn moves(f: [f64; 2]) -> [f64; 2] {
    let u = solve(&table(), &f).expect("the two carts' table is not singular");
    [u[0], u[1]]
}

pub fn eigen() -> Eigen {
    jacobi_eigh(&table())
}

/// The angle, in degrees, between a move pointing along `degrees` and the
/// pushes that hold it.
pub fn turn_degrees(degrees: f64) -> f64 {
    let u = [degrees.to_radians().cos(), degrees.to_radians().sin()];
    turn_between(u, pushes(u))
}

/// The angle in degrees between two arrows, from 0 to 180 (by the arctangent
/// of cross over dot, which stays accurate when the arrows nearly line up).
pub fn turn_between(u: [f64; 2], f: [f64; 2]) -> f64 {
    (u[0] * f[1] - u[1] * f[0]).abs().atan2(dot(&u, &f)).to_degrees()
}

/// How many directions (lines through the origin) a 2 by 2 rule sends back
/// along themselves, by a sweep of half a turn in steps of a tenth of a
/// degree. The sideways part of the answer, u × (rule u), is zero exactly
/// in an own direction: the sweep counts the steps where it is zero and the
/// gaps between steps where it changes sign.
pub fn own_directions(rule: &Mat) -> usize {
    let sideways: Vec<f64> = (0..1800)
        .map(|step| {
            let at = (f64::from(step) / 10.0).to_radians();
            let u = [at.cos(), at.sin()];
            let f = rule.mul_vec(&u);
            u[0] * f[1] - u[1] * f[0]
        })
        .collect();
    let zero = 1e-9 * rule.a.iter().fold(0.0f64, |s, x| s.max(x.abs()));
    (0..sideways.len())
        .filter(|&i| {
            // half a turn on, the sideways part is the same again
            let (here, next) = (sideways[i], sideways[(i + 1) % sideways.len()]);
            here.abs() <= zero || (next.abs() > zero && here * next < 0.0)
        })
        .count()
}

/// A sweep of the move once round the circle in steps of a tenth of a
/// degree: the directions where the pushes line up with the move (to within
/// a millionth of a degree), and the largest turn met on the way.
pub fn sweep() -> (Vec<f64>, f64) {
    let mut lined_up = Vec::new();
    let mut worst = 0.0f64;
    for step in 0..3600 {
        let at = f64::from(step) / 10.0;
        let turn = turn_degrees(at);
        if turn < 1e-6 {
            lined_up.push(at);
        }
        worst = worst.max(turn);
    }
    (lined_up, worst)
}

/// How many times a second each pattern swings when the carts are let go,
/// softest first: √(λ k / m) / 2π.
pub fn swings_per_second() -> Vec<f64> {
    eigen()
        .values
        .iter()
        .map(|lambda| (lambda * SPRING_N_PER_M / CART_KG).sqrt() / std::f64::consts::TAU)
        .collect()
}

/// A tube of still air cut into `slices` equal slices is this chain of
/// carts: the ratio of the chain's `mode`-th frequency (1-based) to the
/// tube's own, which is exactly `mode` half-waves along its length.
pub fn chain_against_the_tube(slices: usize, mode: usize) -> f64 {
    let e = jacobi_eigh(&chain(slices));
    // slices of air h = L / (slices + 1) apart: ω = (c / h) √λ, and the tube's is mode · π c / L
    e.values[mode - 1].sqrt() * (slices + 1) as f64 / (mode as f64 * std::f64::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_quiz_arrows() {
        assert_eq!(pushes([1.0, 0.0]), [2.0, -1.0]);
        assert_eq!(pushes([1.0, 1.0]), [1.0, 1.0]);
        assert_eq!(pushes([1.0, -1.0]), [3.0, -3.0]);
    }

    #[test]
    fn the_pushes_line_up_with_the_move_in_two_lines_only() {
        let (lined_up, worst) = sweep();
        assert_eq!(lined_up, [45.0, 135.0, 225.0, 315.0]);
        // sin(turn) = (3 − 1) / (3 + 1) at the worst
        assert!((worst - 30.0).abs() < 1e-3, "{worst}");
    }

    /// The split over the two patterns (how much of each the pushes hold,
    /// each divided by its eigenvalue) gives back the direct answer; the
    /// facts do the same in exact fractions.
    #[test]
    fn the_split_gives_back_the_direct_answer() {
        let f = [1.0, 0.0];
        let e = eigen();
        let mut u = [0.0, 0.0];
        for (lambda, q) in e.values.iter().zip(&e.vectors) {
            let share = dot(q, &f) / lambda;
            u[0] += share * q[0];
            u[1] += share * q[1];
        }
        let direct = moves(f);
        assert!((u[0] - direct[0]).abs() < 1e-14 && (u[1] - direct[1]).abs() < 1e-14);
        assert!((direct[0] - 2.0 / 3.0).abs() < 1e-14 && (direct[1] - 1.0 / 3.0).abs() < 1e-14);
    }

    #[test]
    fn a_shear_has_one_own_direction_and_a_quarter_turn_none() {
        assert_eq!(own_directions(&table()), 2);
        assert_eq!(own_directions(&Mat::from_rows(&[&[1.0, 1.0], &[0.0, 1.0]])), 1);
        assert_eq!(own_directions(&Mat::from_rows(&[&[0.0, -1.0], &[1.0, 0.0]])), 0);
    }

    #[test]
    fn a_long_chain_rings_like_the_tube() {
        assert!((chain_against_the_tube(40, 1) - 1.0).abs() < 3e-4);
        assert!(chain_against_the_tube(4, 1) < 0.99);
    }
}
