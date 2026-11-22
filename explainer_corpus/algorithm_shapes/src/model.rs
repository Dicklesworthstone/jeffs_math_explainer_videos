//! The film's computation, independent of drawing and narration.
//! A sweep emits every offer, including zero offers, so pictures can show why
//! a badge changes. Cyclic graphs are rejected, not silently treated as DAGs.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Road {
    pub from: usize,
    pub to: usize,
    pub value: u64,
    pub open: bool,
}

pub const PLACES: [&str; 4] = ["Start", "Hill", "Lake", "Finish"];
pub const ROADS: [Road; 5] = [
    Road {
        from: 0,
        to: 1,
        value: 2,
        open: true,
    },
    Road {
        from: 0,
        to: 2,
        value: 5,
        open: true,
    },
    Road {
        from: 1,
        to: 2,
        value: 1,
        open: true,
    },
    Road {
        from: 1,
        to: 3,
        value: 7,
        open: true,
    },
    Road {
        from: 2,
        to: 3,
        value: 3,
        open: true,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidVertex,
    InvalidOrder,
    Cycle,
    Overflow,
}

/// Laws belong to each implementation, not to Rust's trait system. The
/// generic traversal cannot make an arbitrary pair of operations a semiring.
pub trait Arithmetic {
    type Value: Clone + std::fmt::Debug + PartialEq;
    fn zero() -> Self::Value;
    fn one() -> Self::Value;
    fn road(road: Road) -> Self::Value;
    fn combine(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error>;
    fn extend(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Offer<V> {
    pub road_index: usize,
    pub source: V,
    pub edge: V,
    pub previous: V,
    pub offered: V,
    pub combined: V,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sweep<V> {
    pub badges: Vec<V>,
    pub order: Vec<usize>,
    pub offers: Vec<Offer<V>>,
}

pub fn topological_order(vertices: usize, roads: &[Road]) -> Result<Vec<usize>, Error> {
    let mut incoming = vec![0usize; vertices];
    for road in roads {
        if road.from >= vertices || road.to >= vertices {
            return Err(Error::InvalidVertex);
        }
        incoming[road.to] += 1;
    }
    let mut ready = std::collections::BTreeSet::new();
    for (vertex, &count) in incoming.iter().enumerate() {
        if count == 0 {
            ready.insert(vertex);
        }
    }
    let mut order = Vec::with_capacity(vertices);
    while let Some(vertex) = ready.pop_first() {
        order.push(vertex);
        for road in roads.iter().filter(|road| road.from == vertex) {
            incoming[road.to] -= 1;
            if incoming[road.to] == 0 {
                ready.insert(road.to);
            }
        }
    }
    if order.len() != vertices {
        return Err(Error::Cycle);
    }
    Ok(order)
}

pub fn sweep<A: Arithmetic>(vertices: usize, roads: &[Road], start: usize) -> Result<Sweep<A::Value>, Error> {
    let order = topological_order(vertices, roads)?;
    sweep_in_order::<A>(vertices, roads, start, &order)
}

pub fn sweep_in_order<A: Arithmetic>(
    vertices: usize,
    roads: &[Road],
    start: usize,
    order: &[usize],
) -> Result<Sweep<A::Value>, Error> {
    if start >= vertices {
        return Err(Error::InvalidVertex);
    }
    if order.len() != vertices {
        return Err(Error::InvalidOrder);
    }
    let mut position = vec![None; vertices];
    for (index, &vertex) in order.iter().enumerate() {
        if vertex >= vertices || position[vertex].replace(index).is_some() {
            return Err(Error::InvalidOrder);
        }
    }
    for road in roads {
        if road.from >= vertices || road.to >= vertices {
            return Err(Error::InvalidVertex);
        }
        if position[road.from] >= position[road.to] {
            return Err(Error::InvalidOrder);
        }
    }
    let mut badges = vec![A::zero(); vertices];
    badges[start] = A::one();
    let mut offers = Vec::with_capacity(roads.len());
    for &vertex in order {
        for (road_index, road) in roads
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, road)| road.from == vertex)
        {
            let source = badges[vertex].clone();
            let edge = A::road(road);
            let previous = badges[road.to].clone();
            let offered = A::extend(&source, &edge)?;
            let combined = A::combine(&previous, &offered)?;
            badges[road.to] = combined.clone();
            offers.push(Offer {
                road_index,
                source,
                edge,
                previous,
                offered,
                combined,
            });
        }
    }
    Ok(Sweep {
        badges,
        order: order.to_vec(),
        offers,
    })
}

/// Layer k contains answers for exactly k roads, not at most k roads.
/// Works on cyclic graphs too because the road budget makes the expansion finite.
pub fn layers<A: Arithmetic>(
    vertices: usize,
    roads: &[Road],
    start: usize,
    budget: usize,
) -> Result<Vec<Vec<A::Value>>, Error> {
    if start >= vertices || roads.iter().any(|r| r.from >= vertices || r.to >= vertices) {
        return Err(Error::InvalidVertex);
    }
    let mut first = vec![A::zero(); vertices];
    first[start] = A::one();
    let mut result = vec![first];
    for k in 0..budget {
        let mut next = vec![A::zero(); vertices];
        for &road in roads {
            let offered = A::extend(&result[k][road.from], &A::road(road))?;
            next[road.to] = A::combine(&next[road.to], &offered)?;
        }
        result.push(next);
    }
    Ok(result)
}

/// None denotes unreachable, never an arbitrary large sentinel integer.
pub struct Quickest;
impl Arithmetic for Quickest {
    type Value = Option<u64>;
    fn zero() -> Self::Value {
        None
    }
    fn one() -> Self::Value {
        Some(0)
    }
    fn road(r: Road) -> Self::Value {
        r.open.then_some(r.value)
    }
    fn combine(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error> {
        Ok(match (a, b) {
            (Some(x), Some(y)) => Some((*x).min(*y)),
            (x, None) | (None, x) => *x,
        })
    }
    fn extend(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error> {
        match (a, b) {
            (Some(x), Some(y)) => x.checked_add(*y).map(Some).ok_or(Error::Overflow),
            _ => Ok(None),
        }
    }
}

pub struct Count;
impl Arithmetic for Count {
    type Value = u64;
    fn zero() -> u64 {
        0
    }
    fn one() -> u64 {
        1
    }
    fn road(r: Road) -> u64 {
        u64::from(r.open)
    }
    fn combine(a: &u64, b: &u64) -> Result<u64, Error> {
        a.checked_add(*b).ok_or(Error::Overflow)
    }
    fn extend(a: &u64, b: &u64) -> Result<u64, Error> {
        a.checked_mul(*b).ok_or(Error::Overflow)
    }
}

pub struct Reachable;
impl Arithmetic for Reachable {
    type Value = bool;
    fn zero() -> bool {
        false
    }
    fn one() -> bool {
        true
    }
    fn road(r: Road) -> bool {
        r.open
    }
    fn combine(a: &bool, b: &bool) -> Result<bool, Error> {
        Ok(*a || *b)
    }
    fn extend(a: &bool, b: &bool) -> Result<bool, Error> {
        Ok(*a && *b)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Width {
    Unreachable,
    Finite(u64),
    Unlimited,
}
pub struct Widest;
impl Arithmetic for Widest {
    type Value = Width;
    fn zero() -> Width {
        Width::Unreachable
    }
    fn one() -> Width {
        Width::Unlimited
    }
    fn road(r: Road) -> Width {
        if r.open {
            Width::Finite(r.value)
        } else {
            Width::Unreachable
        }
    }
    fn combine(a: &Width, b: &Width) -> Result<Width, Error> {
        Ok((*a).max(*b))
    }
    fn extend(a: &Width, b: &Width) -> Result<Width, Error> {
        Ok((*a).min(*b))
    }
}

/// Unreachable has exactly zero routes by construction; (infinity, one) is
/// deliberately not representable in the production arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Best {
    Unreachable,
    Finite { time: u64, count: u64 },
}
pub struct QuickestCount;
impl Arithmetic for QuickestCount {
    type Value = Best;
    fn zero() -> Best {
        Best::Unreachable
    }
    fn one() -> Best {
        Best::Finite { time: 0, count: 1 }
    }
    fn road(r: Road) -> Best {
        if r.open {
            Best::Finite {
                time: r.value,
                count: 1,
            }
        } else {
            Best::Unreachable
        }
    }
    fn combine(a: &Best, b: &Best) -> Result<Best, Error> {
        match (*a, *b) {
            (Best::Unreachable, x) | (x, Best::Unreachable) => Ok(x),
            (Best::Finite { time: t, count: n }, Best::Finite { time: u, count: m }) => {
                if t < u {
                    Ok(*a)
                } else if u < t {
                    Ok(*b)
                } else {
                    Ok(Best::Finite {
                        time: t,
                        count: n.checked_add(m).ok_or(Error::Overflow)?,
                    })
                }
            }
        }
    }
    fn extend(a: &Best, b: &Best) -> Result<Best, Error> {
        match (*a, *b) {
            (Best::Finite { time: t, count: n }, Best::Finite { time: u, count: m }) => Ok(Best::Finite {
                time: t.checked_add(u).ok_or(Error::Overflow)?,
                count: n.checked_mul(m).ok_or(Error::Overflow)?,
            }),
            _ => Ok(Best::Unreachable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_traversal_five_questions_and_an_exact_trace() {
        let quick = sweep::<Quickest>(4, &ROADS, 0).unwrap();
        assert_eq!(quick.badges, [Some(0), Some(2), Some(3), Some(6)]);
        assert_eq!(
            quick.offers.iter().map(|o| o.road_index).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            (
                quick.offers[2].previous,
                quick.offers[2].offered,
                quick.offers[2].combined
            ),
            (Some(5), Some(3), Some(3))
        );
        assert_eq!(sweep::<Count>(4, &ROADS, 0).unwrap().badges, [1, 1, 2, 3]);
        assert_eq!(sweep::<Reachable>(4, &ROADS, 0).unwrap().badges, [true; 4]);
        assert_eq!(
            sweep::<Widest>(4, &ROADS, 0).unwrap().badges,
            [Width::Unlimited, Width::Finite(2), Width::Finite(5), Width::Finite(3)]
        );
        assert_eq!(
            sweep::<QuickestCount>(4, &ROADS, 0).unwrap().badges[3],
            Best::Finite { time: 6, count: 1 }
        );
    }

    #[test]
    fn ties_add_once_and_closed_roads_absorb() {
        let mut roads = ROADS;
        roads[3].value = 4;
        assert_eq!(
            sweep::<QuickestCount>(4, &roads, 0).unwrap().badges[3],
            Best::Finite { time: 6, count: 2 }
        );
        roads[3].open = false;
        roads[4].open = false;
        assert_eq!(
            sweep::<QuickestCount>(4, &roads, 0).unwrap().badges[3],
            Best::Unreachable
        );
    }

    #[test]
    fn road_budget_recovers_the_discarded_arrival() {
        let values = layers::<Quickest>(4, &ROADS, 0, 3).unwrap();
        assert_eq!(values[1][2], Some(5));
        assert_eq!(values[2][2], Some(3));
        assert_eq!(values[2][3], Some(8));
        assert_eq!(values[3][3], Some(6));
    }

    #[test]
    fn cyclic_sweeps_refuse_but_finite_layers_work() {
        let roads = [Road {
            from: 0,
            to: 0,
            value: 0,
            open: true,
        }];
        assert_eq!(sweep::<Count>(1, &roads, 0), Err(Error::Cycle));
        assert_eq!(layers::<Count>(1, &roads, 0, 3).unwrap(), vec![vec![1]; 4]);
        assert_eq!(
            sweep_in_order::<Count>(4, &ROADS, 0, &[0, 2, 1, 3]),
            Err(Error::InvalidOrder)
        );
    }

    #[test]
    fn exhaustive_closures_match_explicit_route_listing() {
        // Independent bounded oracle: the three complete routes are listed,
        // not obtained from the production recurrence or its event trace.
        let paths: [&[usize]; 3] = [&[0, 3], &[1, 4], &[0, 2, 4]];
        for mask in 0..32 {
            let mut roads = ROADS;
            for (i, road) in roads.iter_mut().enumerate() {
                road.open = mask & (1 << i) != 0;
            }
            let times: Vec<u64> = paths
                .iter()
                .filter(|path| path.iter().all(|&i| roads[i].open))
                .map(|path| path.iter().map(|&i| roads[i].value).sum())
                .collect();
            let best = times.iter().copied().min();
            assert_eq!(sweep::<Quickest>(4, &roads, 0).unwrap().badges[3], best);
            assert_eq!(sweep::<Count>(4, &roads, 0).unwrap().badges[3], times.len() as u64);
            assert_eq!(sweep::<Reachable>(4, &roads, 0).unwrap().badges[3], !times.is_empty());
            let expected = best.map_or(Best::Unreachable, |time| Best::Finite {
                time,
                count: times.iter().filter(|&&t| t == time).count() as u64,
            });
            assert_eq!(sweep::<QuickestCount>(4, &roads, 0).unwrap().badges[3], expected);
        }
    }

    #[test]
    fn overflow_is_not_an_unreachable_route() {
        assert_eq!(Quickest::extend(&Some(u64::MAX), &Some(1)), Err(Error::Overflow));
        assert_eq!(Count::combine(&u64::MAX, &1), Err(Error::Overflow));
        assert_eq!(Quickest::extend(&None, &Some(u64::MAX)), Ok(None));
    }
}
