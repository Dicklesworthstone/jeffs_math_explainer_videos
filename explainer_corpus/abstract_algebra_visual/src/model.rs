//! Exact square-symmetry model for the planned abstract-algebra scenes.
//! Products are composed right-to-left: `a.compose(b)` means b, then a.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Symmetry {
    /// Number of counterclockwise quarter turns, in 0..4.
    pub turns: u8,
    /// Reflect across the horizontal axis before those turns.
    pub reflected: bool,
}

impl Symmetry {
    pub const ID: Self = Self {
        turns: 0,
        reflected: false,
    };
    pub const R: Self = Self {
        turns: 1,
        reflected: false,
    };
    pub const S: Self = Self {
        turns: 0,
        reflected: true,
    };

    pub const fn new(turns: u8, reflected: bool) -> Self {
        Self {
            turns: turns % 4,
            reflected,
        }
    }

    pub fn compose(self, right: Self) -> Self {
        let right_turns = if self.reflected {
            (4 - right.turns) % 4
        } else {
            right.turns
        };
        Self::new(self.turns + right_turns, self.reflected ^ right.reflected)
    }

    pub fn apply(self, mut point: (i8, i8)) -> (i8, i8) {
        if self.reflected {
            point.1 = -point.1;
        }
        for _ in 0..self.turns {
            point = (-point.1, point.0);
        }
        point
    }
}

pub fn all() -> [Symmetry; 8] {
    std::array::from_fn(|i| Symmetry::new((i % 4) as u8, i >= 4))
}

/// Membership in the left coset representative * subgroup.
pub fn left_coset(representative: Symmetry, subgroup: &[Symmetry]) -> Vec<Symmetry> {
    subgroup.iter().map(|h| representative.compose(*h)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    const CORNERS: [(i8, i8); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

    #[test]
    fn formula_matches_composition_on_every_corner() {
        for a in all() {
            for b in all() {
                for p in CORNERS {
                    assert_eq!(a.compose(b).apply(p), a.apply(b.apply(p)));
                }
            }
        }
    }

    #[test]
    fn all_eight_actions_are_distinct_and_form_a_group() {
        let g = all();
        let pictures: std::collections::HashSet<_> = g.iter().map(|x| CORNERS.map(|p| x.apply(p))).collect();
        assert_eq!(pictures.len(), 8);
        for a in g {
            assert_eq!(a.compose(Symmetry::ID), a);
            assert_eq!(Symmetry::ID.compose(a), a);
            assert!(
                g.iter()
                    .any(|b| a.compose(*b) == Symmetry::ID && b.compose(a) == Symmetry::ID)
            );
            for b in g {
                for c in g {
                    assert_eq!(a.compose(b).compose(c), a.compose(b.compose(c)));
                }
            }
        }
    }

    #[test]
    fn centralizer_and_bad_representatives_are_concrete() {
        let s = Symmetry::S;
        let r = Symmetry::R;
        let commuting: Vec<_> = all().into_iter().filter(|x| x.compose(s) == s.compose(*x)).collect();
        assert_eq!(
            commuting,
            vec![Symmetry::ID, Symmetry::new(2, false), s, Symmetry::new(2, true)]
        );
        let h = [Symmetry::ID, s];
        let r_h = left_coset(r, &h);
        assert!(r_h.contains(&Symmetry::ID.compose(r)));
        assert!(!r_h.contains(&s.compose(r)));
        assert!(left_coset(Symmetry::new(3, false), &h).contains(&s.compose(r)));
    }

    #[test]
    fn rotation_cosets_multiply_independently_of_all_representatives() {
        let n = [
            Symmetry::ID,
            Symmetry::R,
            Symmetry::new(2, false),
            Symmetry::new(3, false),
        ];
        for a in all() {
            for b in all() {
                let expected = left_coset(a.compose(b), &n);
                for x in left_coset(a, &n) {
                    for y in left_coset(b, &n) {
                        assert!(expected.contains(&x.compose(y)));
                    }
                }
            }
        }
    }
}
