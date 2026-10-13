//! Act II: a Pratt truss in steel, solved by the stiffness method the way
//! structural programs do it. Each member is a spring k = EA/L along its own
//! line; the members' springs add up, joint by joint, into one stiffness
//! table K; the supports strike out three rows and columns; K u = f gives
//! the moves, the moves give each member's force, and K's own patterns
//! (and, with mass, K φ = ω² M φ) give the modes.
//!
//! The truss is a model built for this video: 24 m span, six panels of 4 m,
//! 4 m deep, a pin at the left end and a roller at the right, the deck on
//! the bottom chord. Joints 0 to 6 run along the bottom, 7 to 11 along the
//! top (above joints 1 to 5).

use super::linalg::{Eigen, Mat, generalized_eigh, jacobi_eigh, solve};

pub const PANELS: usize = 6;
pub const PANEL_M: f64 = 4.0;
pub const DEPTH_M: f64 = 4.0;
/// Young's modulus of structural steel, pascals (29,000 ksi in round metric;
/// FrankenSim's steel frames use the same `200e9`).
pub const E_PA: f64 = 200e9;
/// Cross-sections, square metres: the chords and end posts, and the lighter
/// verticals and diagonals.
pub const CHORD_AREA_M2: f64 = 6.0e-3;
pub const WEB_AREA_M2: f64 = 3.0e-3;
/// Steel's density (490 lb/ft³), kilograms a cubic metre.
pub const STEEL_KG_PER_M3: f64 = 7850.0;
/// The minimum yield stress of ASTM A36 steel (36 ksi), pascals.
pub const YIELD_PA: f64 = 250e6;
/// The worked load: this much, downward, at each of the five deck joints.
pub const DECK_LOAD_N: f64 = 100e3;
/// The deck's mass carried at each interior bottom joint, kilograms (half
/// of it at each end joint).
pub const DECK_KG: f64 = 4000.0;
/// The joint that is pushed alone in the "everything moves" picture.
pub const LONE_JOINT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bottom,
    Top,
    EndPost,
    Vertical,
    Diagonal,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::Bottom, Kind::Top, Kind::EndPost, Kind::Vertical, Kind::Diagonal];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Bottom => "bottom",
            Kind::Top => "top",
            Kind::EndPost => "end",
            Kind::Vertical => "vertical",
            Kind::Diagonal => "diagonal",
        }
    }

    pub fn area_m2(self) -> f64 {
        match self {
            Kind::Bottom | Kind::Top | Kind::EndPost => CHORD_AREA_M2,
            Kind::Vertical | Kind::Diagonal => WEB_AREA_M2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    pub a: usize,
    pub b: usize,
    pub kind: Kind,
}

#[derive(Clone, Debug)]
pub struct Truss {
    pub joints: Vec<(f64, f64)>,
    pub members: Vec<Member>,
    /// The joint directions the supports hold still (2·joint for sideways,
    /// 2·joint + 1 for up and down).
    pub held: Vec<usize>,
}

impl Truss {
    /// The video's truss. A Pratt truss slopes its diagonals down toward
    /// the middle, so that a load on the deck pulls on them.
    pub fn pratt() -> Self {
        let bottom = |i: usize| i;
        let top = |i: usize| PANELS + i;
        let mut joints: Vec<(f64, f64)> = (0..=PANELS).map(|i| (i as f64 * PANEL_M, 0.0)).collect();
        joints.extend((1..PANELS).map(|i| (i as f64 * PANEL_M, DEPTH_M)));
        let mut members = Vec::new();
        let mut put = |a: usize, b: usize, kind: Kind| members.push(Member { a, b, kind });
        for i in 0..PANELS {
            put(bottom(i), bottom(i + 1), Kind::Bottom);
        }
        for i in 1..PANELS - 1 {
            put(top(i), top(i + 1), Kind::Top);
        }
        put(bottom(0), top(1), Kind::EndPost);
        put(bottom(PANELS), top(PANELS - 1), Kind::EndPost);
        for i in 1..PANELS {
            put(bottom(i), top(i), Kind::Vertical);
        }
        let mid = PANELS / 2;
        for i in 1..mid {
            put(top(i), bottom(i + 1), Kind::Diagonal);
        }
        for i in mid + 1..PANELS {
            put(top(i), bottom(i - 1), Kind::Diagonal);
        }
        Self {
            joints,
            members,
            held: vec![0, 1, 2 * PANELS + 1],
        }
    }

    pub fn span_m(&self) -> f64 {
        PANELS as f64 * PANEL_M
    }

    /// The deck joints that carry the worked load: the bottom joints between the supports.
    pub fn deck_joints(&self) -> Vec<usize> {
        (1..PANELS).collect()
    }

    pub fn midspan(&self) -> usize {
        PANELS / 2
    }

    pub fn length_m(&self, m: &Member) -> f64 {
        let (a, b) = (self.joints[m.a], self.joints[m.b]);
        (b.0 - a.0).hypot(b.1 - a.1)
    }

    /// The member's direction from joint `a` to joint `b`, as (cosine, sine).
    pub fn direction(&self, m: &Member) -> (f64, f64) {
        let (a, b) = (self.joints[m.a], self.joints[m.b]);
        let l = self.length_m(m);
        ((b.0 - a.0) / l, (b.1 - a.1) / l)
    }

    /// The member as a spring: EA/L, newtons a metre.
    pub fn spring(&self, m: &Member) -> f64 {
        E_PA * m.kind.area_m2() / self.length_m(m)
    }

    /// The stiffness table of the truss before any support holds it: each
    /// member adds its 4 by 4 block k·ggᵀ, g = (−c, −s, c, s), at its two
    /// joints' rows and columns.
    pub fn stiffness_unsupported(&self) -> Mat {
        let mut k = Mat::zeros(2 * self.joints.len());
        for m in &self.members {
            let (c, s) = self.direction(m);
            let g = [-c, -s, c, s];
            let at = [2 * m.a, 2 * m.a + 1, 2 * m.b, 2 * m.b + 1];
            let spring = self.spring(m);
            for i in 0..4 {
                for j in 0..4 {
                    k.add(at[i], at[j], spring * g[i] * g[j]);
                }
            }
        }
        k
    }

    /// The joint directions left free by the supports, in order.
    pub fn free(&self) -> Vec<usize> {
        (0..2 * self.joints.len()).filter(|d| !self.held.contains(d)).collect()
    }

    /// K: the stiffness table with the supported rows and columns struck out.
    pub fn stiffness(&self) -> Mat {
        self.stiffness_unsupported().sub(&self.free())
    }

    /// The pushes on the free directions for a list of (joint, sideways,
    /// upward) loads in newtons.
    pub fn pushes(&self, loads: &[(usize, f64, f64)]) -> Vec<f64> {
        let mut full = vec![0.0; 2 * self.joints.len()];
        for &(joint, fx, fy) in loads {
            full[2 * joint] += fx;
            full[2 * joint + 1] += fy;
        }
        self.free().iter().map(|&d| full[d]).collect()
    }

    /// The worked load: `DECK_LOAD_N` downward on every deck joint.
    pub fn deck_load(&self) -> Vec<f64> {
        let loads: Vec<_> = self.deck_joints().iter().map(|&j| (j, 0.0, -DECK_LOAD_N)).collect();
        self.pushes(&loads)
    }

    /// The same push on one deck joint alone.
    pub fn lone_load(&self) -> Vec<f64> {
        self.pushes(&[(LONE_JOINT, 0.0, -DECK_LOAD_N)])
    }

    /// K u = f, solved directly: the moves of the free directions, metres.
    pub fn moves(&self, f: &[f64]) -> Vec<f64> {
        solve(&self.stiffness(), f).expect("a supported truss has a stiffness table that is not singular")
    }

    /// The moves of every joint direction (zeros where a support holds).
    pub fn all_moves(&self, free_moves: &[f64]) -> Vec<f64> {
        let mut full = vec![0.0; 2 * self.joints.len()];
        for (&d, &u) in self.free().iter().zip(free_moves) {
            full[d] = u;
        }
        full
    }

    /// Each member's force in newtons, pull positive: its spring times how
    /// much the moves stretch it.
    pub fn member_forces(&self, free_moves: &[f64]) -> Vec<f64> {
        let u = self.all_moves(free_moves);
        self.members
            .iter()
            .map(|m| {
                let (c, s) = self.direction(m);
                let stretch = c * (u[2 * m.b] - u[2 * m.a]) + s * (u[2 * m.b + 1] - u[2 * m.a + 1]);
                self.spring(m) * stretch
            })
            .collect()
    }

    /// K's own patterns, softest first (eigenvalues in newtons a metre).
    pub fn patterns(&self) -> Eigen {
        jacobi_eigh(&self.stiffness())
    }

    /// The moves rebuilt from the softest `count` patterns alone:
    /// Σ (qᵢ·f / λᵢ) qᵢ.
    pub fn moves_from_patterns(&self, e: &Eigen, f: &[f64], count: usize) -> Vec<f64> {
        let mut u = vec![0.0; f.len()];
        for (lambda, q) in e.values.iter().zip(&e.vectors).take(count) {
            let share = super::linalg::dot(q, f) / lambda;
            for (u, q) in u.iter_mut().zip(q) {
                *u += share * q;
            }
        }
        u
    }

    /// The steel in the members, kilograms.
    pub fn steel_kg(&self) -> f64 {
        self.members
            .iter()
            .map(|m| STEEL_KG_PER_M3 * m.kind.area_m2() * self.length_m(m))
            .sum()
    }

    /// The mass gathered at each joint: half of every member that meets it,
    /// and the deck on the bottom joints (`deck_kg` at each interior one,
    /// half that at the two ends).
    pub fn joint_kg(&self, deck_kg: f64) -> Vec<f64> {
        let mut kg = vec![0.0; self.joints.len()];
        for m in &self.members {
            let half = STEEL_KG_PER_M3 * m.kind.area_m2() * self.length_m(m) / 2.0;
            kg[m.a] += half;
            kg[m.b] += half;
        }
        for (i, kg) in kg.iter_mut().enumerate().take(PANELS + 1) {
            *kg += if i == 0 || i == PANELS { deck_kg / 2.0 } else { deck_kg };
        }
        kg
    }

    /// The mass on each free direction (a joint's mass resists both of its directions).
    pub fn mass(&self, deck_kg: f64) -> Vec<f64> {
        let kg = self.joint_kg(deck_kg);
        self.free().iter().map(|&d| kg[d / 2]).collect()
    }

    /// The modes of K φ = ω² M φ, lowest first: `values` are ω² in (rad/s)².
    pub fn modes(&self, deck_kg: f64) -> Eigen {
        generalized_eigh(&self.stiffness(), &self.mass(deck_kg))
    }

    /// The modes' frequencies in hertz, lowest first.
    pub fn hertz(&self, deck_kg: f64) -> Vec<f64> {
        self.modes(deck_kg)
            .values
            .iter()
            .map(|w2| w2.max(0.0).sqrt() / std::f64::consts::TAU)
            .collect()
    }

    /// How often the up-and-down move changes sign along the deck (the
    /// bottom joints between the supports) in a pattern: 0 for one sag, 1
    /// for an S, and so on. A joint that barely moves counts as no change.
    pub fn deck_sign_changes(&self, pattern: &[f64]) -> usize {
        let u = self.all_moves(pattern);
        let deck: Vec<f64> = self.deck_joints().iter().map(|&j| u[2 * j + 1]).collect();
        let big = deck.iter().fold(0.0f64, |s, x| s.max(x.abs()));
        let signs: Vec<f64> = deck
            .iter()
            .filter(|x| x.abs() > 1e-6 * big)
            .map(|x| x.signum())
            .collect();
        signs.windows(2).filter(|w| w[0] != w[1]).count()
    }

    /// How often the sideways move changes sign from one bottom joint to the
    /// next (the pinned end left out): 5 when every neighbour moves the
    /// opposite way.
    pub fn chord_sign_changes(&self, pattern: &[f64]) -> usize {
        let u = self.all_moves(pattern);
        let chord: Vec<f64> = (1..=PANELS).map(|j| u[2 * j]).collect();
        let big = chord.iter().fold(0.0f64, |s, x| s.max(x.abs()));
        let signs: Vec<f64> = chord
            .iter()
            .filter(|x| x.abs() > 1e-6 * big)
            .map(|x| x.signum())
            .collect();
        signs.windows(2).filter(|w| w[0] != w[1]).count()
    }

    /// The upward push each support gives back for the free moves `u` under
    /// the pushes `f`: the struck-out rows of the table times the moves.
    pub fn reactions(&self, free_moves: &[f64]) -> Vec<f64> {
        let k = self.stiffness_unsupported();
        let u = self.all_moves(free_moves);
        self.held.iter().map(|&d| k.mul_vec(&u)[d]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::linalg::dot;

    #[test]
    fn the_truss_is_twelve_joints_and_twenty_one_members() {
        let t = Truss::pratt();
        assert_eq!(t.joints.len(), 12);
        assert_eq!(t.members.len(), 21);
        // as many members as free directions: the forces follow from balance alone
        assert_eq!(t.free().len(), 21);
        let count = |kind| t.members.iter().filter(|m| m.kind == kind).count();
        assert_eq!(
            Kind::ALL.map(count),
            [6, 4, 2, 5, 4],
            "bottom, top, end posts, verticals, diagonals"
        );
    }

    /// Hand statics, with no stiffness in it: the reactions are 250 kN, the
    /// bending moment at midspan is 250·12 − 100·8 − 100·4 = 1,800 kN·m, and
    /// the 4 m depth turns it into 450 kN of squeeze in the top chord.
    #[test]
    fn member_forces_match_statics_by_hand() {
        let t = Truss::pratt();
        let n = t.member_forces(&t.moves(&t.deck_load()));
        let force = |a: usize, b: usize| {
            let i = t
                .members
                .iter()
                .position(|m| (m.a, m.b) == (a, b) || (m.a, m.b) == (b, a))
                .unwrap();
            n[i] / 1e3
        };
        assert!((force(8, 9) + 450.0).abs() < 1e-6);
        assert!((force(2, 3) - 400.0).abs() < 1e-6);
        assert!((force(0, 1) - 250.0).abs() < 1e-6);
        assert!((force(0, 7) + 250.0 * 2f64.sqrt()).abs() < 1e-6);
        assert!((force(7, 2) - 150.0 * 2f64.sqrt()).abs() < 1e-6);
        assert!(force(3, 9).abs() < 1e-6, "the middle vertical carries nothing");
    }

    #[test]
    fn all_twenty_one_patterns_rebuild_the_direct_answer() {
        let t = Truss::pratt();
        let f = t.deck_load();
        let direct = t.moves(&f);
        let e = t.patterns();
        let rebuilt = t.moves_from_patterns(&e, &f, e.values.len());
        for (a, b) in direct.iter().zip(&rebuilt) {
            assert!((a - b).abs() < 1e-12, "{a} vs {b}");
        }
        // the patterns are at right angles
        for i in 0..e.vectors.len() {
            for j in 0..i {
                assert!(dot(&e.vectors[i], &e.vectors[j]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn unsupported_the_truss_has_three_ways_to_move_for_free() {
        let t = Truss::pratt();
        let e = jacobi_eigh(&t.stiffness_unsupported());
        let top = e.values.last().copied().unwrap();
        assert_eq!(e.values.iter().filter(|v| v.abs() < 1e-9 * top).count(), 3);
        // supported, the softest pattern still takes millions of newtons a metre
        assert!(t.patterns().values[0] > 1e6);
    }
}
