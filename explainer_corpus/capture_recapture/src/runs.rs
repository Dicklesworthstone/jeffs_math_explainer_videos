//! The oracle's seeded experiments, run again here draw for draw with
//! [`crate::numpy::Generator`], so every number the late chapters show from
//! a simulation is the simulation's own: chapter 09's code panel (its
//! 10,000 caught-twice counts are 12_2's rings), verify2.py's stream (the
//! failure modes of chapter 13) and the two reviewers of 14_6. Each
//! function follows its oracle function line by line (`ch09_panel`,
//! `verify2_replay`, `ch14_reviewers`).

use crate::model::SEED;
use crate::numpy::Generator;

/// The true lake of chapters 09–13 and Feller's catches.
pub const LAKE: usize = 10_000;
pub const MARKED: usize = 1_000;
pub const CAUGHT: usize = 1_000;
/// Experiments in the 09_6 panel.
pub const PANEL_RUNS: usize = 10_000;
/// Runs per failure mode (verify2.py section F).
pub const FAILURE_RUNS: usize = 400;
/// The failure models (verify2.py section F): this share of fish is this
/// many times as catchable, both times (bold fish).
pub const BOLD_SHARE: (u64, u64) = (3, 10);
pub const BOLD_FACTOR: f64 = 3.0;
/// Marked fish this many times as catchable the second time (fish that
/// love the bait, fish that learn to avoid it).
pub const HAPPY: f64 = 1.5;
pub const SHY: (u64, u64) = (2, 3);
/// Each mark washes off with this chance.
pub const LOSS: (u64, u64) = (1, 5);
/// Unmarked fish that join between the catches.
pub const NEWCOMERS: usize = 2_000;
/// Each fish dies with this chance between the catches.
pub const DEATHS: (u64, u64) = (3, 10);
/// Unmixed marks: the second catch takes this many from one half of the
/// lake and this many from the other.
pub const HALVES: (usize, usize) = (800, 200);

fn ratio((num, den): (u64, u64)) -> f64 {
    num as f64 / den as f64
}

/// The models' parameters, as the oracle's `A-F22.params` records them.
pub fn failure_params() -> String {
    format!(
        "runs={FAILURE_RUNS};bold={}/{} x{:?};happy=x{HAPPY:?};shy=x{}/{};lost={}/{};newcomers={NEWCOMERS};deaths={}/{};halves={}/{}",
        BOLD_SHARE.0, BOLD_SHARE.1, BOLD_FACTOR, SHY.0, SHY.1, LOSS.0, LOSS.1, DEATHS.0, DEATHS.1, HALVES.0, HALVES.1
    )
}

/// What the soup guess aims at, M·n / E[k] with E[k] exact, as a fraction
/// (oracle `ch13_targets`): the ideal lake; deaths (each fish of the second
/// catch is marked with chance M/N, by symmetry); newcomers (M/(N + 2,000));
/// lost marks (each seen with chance 4/5).
pub fn targets() -> [(&'static str, (u64, u64)); 4] {
    let (big, m, n) = (LAKE as u64, MARKED as u64, CAUGHT as u64);
    let aim = |(ek_num, ek_den): (u64, u64)| (m * n * ek_den, ek_num);
    [
        ("ideal", aim((n * m, big))),
        ("deaths_30pct", aim((n * m, big))),
        ("newcomers_2000", aim((n * m, big + NEWCOMERS as u64))),
        ("marks_lost_20pct", aim((n * (LOSS.1 - LOSS.0) * m, LOSS.1 * big))),
    ]
}

/// The lake expected at the second catch after the deaths, N(1 − 3/10),
/// as a fraction.
pub fn deaths_left() -> (u64, u64) {
    (LAKE as u64 * (DEATHS.1 - DEATHS.0), DEATHS.1)
}

/// The 09_6 panel, verbatim: `marked = np.arange(N) < M`, then
/// `k = [marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)]`
/// with `rng = np.random.default_rng(20261009)`: each run's caught-twice
/// count, in run order.
pub fn panel() -> Vec<u32> {
    let mut rng = Generator::from_seed(SEED);
    (0..PANEL_RUNS)
        .map(|_| rng.choice(LAKE, CAUGHT).iter().filter(|&&i| i < MARKED).count() as u32)
        .collect()
}

/// The median of a sample, numpy's way (the mean of the middle two when
/// the count is even). Infinities sort last.
pub fn median(values: &[f64]) -> f64 {
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    assert!(n > 0, "the median of nothing");
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// One failure mode's experiments: each run's caught-twice count and soup
/// guess (∞ when nobody was caught twice).
#[derive(Clone, Debug)]
pub struct Failure {
    pub key: &'static str,
    pub caught_twice: Vec<usize>,
    pub estimates: Vec<f64>,
}

impl Failure {
    pub fn median(&self) -> f64 {
        median(&self.estimates)
    }

    /// The median caught-twice count (the mosaic of a typical run; the
    /// median soup guess is M·n over it when the middle two runs agree).
    pub fn median_caught_twice(&self) -> f64 {
        let k: Vec<f64> = self.caught_twice.iter().map(|&k| k as f64).collect();
        median(&k)
    }
}

/// verify2.py's stream, replayed: section E (10,000 hypergeometric draws
/// for each of four lakes) and section F (the failure modes, 400 runs
/// each), in its order.
#[derive(Clone, Debug)]
pub struct Verify2 {
    /// Section E's first lake (N 10,000, M = n = 1,000): the caught-twice
    /// counts of A-F19's research simulation.
    pub feller_k: Vec<i64>,
    /// Section F, in the oracle's order: ideal, bold_fish, trap_happy,
    /// trap_shy, marks_lost_20pct, newcomers_2000, deaths_30pct,
    /// release_site, far_side.
    pub failures: Vec<Failure>,
}

impl Verify2 {
    pub fn failure(&self, key: &str) -> &Failure {
        self.failures
            .iter()
            .find(|f| f.key == key)
            .unwrap_or_else(|| panic!("no failure mode {key}"))
    }
}

/// The indices of the `m` largest keys (`np.argpartition(-keys, m)[:m]`
/// as a set; keys are continuous, so there are no ties).
fn largest(keys: &[f64], m: usize) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..keys.len()).collect();
    idx.select_nth_unstable_by(m, |&a, &b| keys[b].total_cmp(&keys[a]));
    idx.truncate(m);
    idx
}

/// `M·n / k` as numpy computes it (`M_ * n_ / kk`), ∞ at k = 0.
fn soup(k: usize) -> f64 {
    if k == 0 {
        f64::INFINITY
    } else {
        (MARKED * CAUGHT) as f64 / k as f64
    }
}

fn record(key: &'static str, runs: Vec<usize>) -> Failure {
    Failure {
        key,
        estimates: runs.iter().map(|&k| soup(k)).collect(),
        caught_twice: runs,
    }
}

/// Mark `MARKED` fish with weights `w1`, then catch `CAUGHT` with the
/// weights `w2(tagged)`, both by Efraimidis–Spirakis keys log(u)/w
/// (sampling without replacement in proportion to weight): the
/// caught-twice count of each of `runs` experiments.
fn weighted(rng: &mut Generator, w1: &[f64], w2: impl Fn(&[bool]) -> Vec<f64>) -> Vec<usize> {
    (0..FAILURE_RUNS)
        .map(|_| {
            let u = rng.random_n(LAKE);
            let keys: Vec<f64> = u.iter().zip(w1).map(|(u, w)| u.ln() / w).collect();
            let mut tagged = vec![false; LAKE];
            for i in largest(&keys, MARKED) {
                tagged[i] = true;
            }
            let w = w2(&tagged);
            let u2 = rng.random_n(LAKE);
            let keys2: Vec<f64> = u2.iter().zip(&w).map(|(u, w)| u.ln() / w).collect();
            largest(&keys2, CAUGHT).iter().filter(|&&i| tagged[i]).count()
        })
        .collect()
}

/// `rng.choice(pool, size, replace=False)` for an array `pool`.
fn choose_from(rng: &mut Generator, pool: &[usize], size: usize) -> Vec<usize> {
    rng.choice(pool.len(), size).into_iter().map(|i| pool[i]).collect()
}

pub fn verify2() -> Verify2 {
    let mut rng = Generator::from_seed(SEED);
    let mut feller_k = Vec::new();
    for (n_lake, m, c) in [
        (10_000i64, 1_000i64, 1_000i64),
        (1_000, 100, 100),
        (1_000, 50, 50),
        (500, 50, 50),
    ] {
        let ks: Vec<i64> = (0..10_000).map(|_| rng.hypergeometric(m, n_lake - m, c)).collect();
        if n_lake == 10_000 {
            feller_k = ks;
        }
    }
    let mut failures = Vec::new();
    let base = vec![1.0; LAKE];
    failures.push(record("ideal", weighted(&mut rng, &base, |_| vec![1.0; LAKE])));
    // three fish in ten three times as catchable, both times
    let bold: Vec<f64> = rng
        .random_n(LAKE)
        .iter()
        .map(|&u| if u < ratio(BOLD_SHARE) { BOLD_FACTOR } else { 1.0 })
        .collect();
    let bold2 = bold.clone();
    failures.push(record("bold_fish", weighted(&mut rng, &bold, move |_| bold2.clone())));
    // marked fish 1.5× and ⅔× as catchable the second time
    for (key, factor) in [("trap_happy", HAPPY), ("trap_shy", ratio(SHY))] {
        let runs = weighted(&mut rng, &base, |t: &[bool]| {
            t.iter().map(|&m| if m { factor } else { 1.0 }).collect()
        });
        failures.push(record(key, runs));
    }
    let lake: Vec<usize> = (0..LAKE).collect();
    // each mark washes off with chance 1 in 5
    let lost = (0..FAILURE_RUNS)
        .map(|_| {
            let mut tagged = vec![false; LAKE];
            for i in choose_from(&mut rng, &lake, MARKED) {
                tagged[i] = true;
            }
            let r = rng.random_n(LAKE);
            let visible: Vec<bool> = tagged.iter().zip(&r).map(|(&t, &u)| t && u >= ratio(LOSS)).collect();
            choose_from(&mut rng, &lake, CAUGHT)
                .iter()
                .filter(|&&i| visible[i])
                .count()
        })
        .collect();
    failures.push(record("marks_lost_20pct", lost));
    // 2,000 unmarked newcomers join before the second catch
    let grown: Vec<usize> = (0..LAKE + NEWCOMERS).collect();
    let newcomers = (0..FAILURE_RUNS)
        .map(|_| {
            let mut tagged = vec![false; LAKE + NEWCOMERS];
            for i in choose_from(&mut rng, &lake, MARKED) {
                tagged[i] = true;
            }
            choose_from(&mut rng, &grown, CAUGHT)
                .iter()
                .filter(|&&i| tagged[i])
                .count()
        })
        .collect();
    failures.push(record("newcomers_2000", newcomers));
    // each fish dies with chance 3 in 10 between the catches
    let deaths = (0..FAILURE_RUNS)
        .map(|_| {
            let mut tagged = vec![false; LAKE];
            for i in choose_from(&mut rng, &lake, MARKED) {
                tagged[i] = true;
            }
            let r = rng.random_n(LAKE);
            let alive: Vec<usize> = (0..LAKE).filter(|&i| r[i] >= ratio(DEATHS)).collect();
            choose_from(&mut rng, &alive, CAUGHT)
                .iter()
                .filter(|&&i| tagged[i])
                .count()
        })
        .collect();
    failures.push(record("deaths_30pct", deaths));
    // the marks never mix: all marked in one half; 800 of the second catch
    // from one half, 200 from the other
    let east: Vec<usize> = (0..LAKE / 2).collect();
    let west: Vec<usize> = (LAKE / 2..LAKE).collect();
    for (key, same_side) in [("release_site", true), ("far_side", false)] {
        let runs = (0..FAILURE_RUNS)
            .map(|_| {
                let mut tagged = vec![false; LAKE];
                for i in choose_from(&mut rng, &east, MARKED) {
                    tagged[i] = true;
                }
                let (pool, other) = if same_side { (&east, &west) } else { (&west, &east) };
                let mut caught = choose_from(&mut rng, pool, HALVES.0);
                caught.extend(choose_from(&mut rng, other, HALVES.1));
                caught.iter().filter(|&&i| tagged[i]).count()
            })
            .collect();
        failures.push(record(key, runs));
    }
    Verify2 { feller_k, failures }
}

/// The two reviewers' models (oracle `REVIEW`): 60 bugs, 20,000 runs each;
/// 'skill': A finds each bug with chance 0.5 and B with 0.4; 'difficulty':
/// 30 easy bugs (0.8, 0.7) and 30 hard ones (0.2, 0.1).
pub const BUGS: usize = 60;
pub const REVIEW_RUNS: usize = 20_000;
pub const SKILL: (f64, f64) = (0.5, 0.4);
pub const EASY: (f64, f64) = (0.8, 0.7);
pub const HARD: (f64, f64) = (0.2, 0.1);

/// One review: bugs found by A, by B, and by both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Review {
    pub a: usize,
    pub b: usize,
    pub both: usize,
}

/// Each model's 20,000 reviews, skill first, with
/// `np.random.default_rng([20261009, 14])`; `rng.random((R, B)) < p` fills
/// row by row (run by run).
pub fn reviewers() -> [(&'static str, Vec<Review>); 2] {
    let mut rng = Generator::from_words(&[SEED as u32, 14]);
    let half = BUGS / 2;
    let skill = (vec![SKILL.0; BUGS], vec![SKILL.1; BUGS]);
    let mut pa = vec![EASY.0; half];
    pa.extend(vec![HARD.0; BUGS - half]);
    let mut pb = vec![EASY.1; half];
    pb.extend(vec![HARD.1; BUGS - half]);
    let mut draw = |(pa, pb): (Vec<f64>, Vec<f64>)| -> Vec<Review> {
        let found = |rng: &mut Generator, p: &[f64]| -> Vec<Vec<bool>> {
            (0..REVIEW_RUNS)
                .map(|_| p.iter().map(|&p| rng.random() < p).collect())
                .collect()
        };
        let a = found(&mut rng, &pa);
        let b = found(&mut rng, &pb);
        a.iter()
            .zip(&b)
            .map(|(a, b)| Review {
                a: a.iter().filter(|&&x| x).count(),
                b: b.iter().filter(|&&x| x).count(),
                both: a.iter().zip(b).filter(|&(&x, &y)| x && y).count(),
            })
            .collect()
    };
    let skill_runs = draw(skill);
    let difficulty_runs = draw((pa, pb));
    [("skill", skill_runs), ("difficulty", difficulty_runs)]
}

/// The reviewers' models, as the oracle's `A-CH14.reviewers.params`
/// records them.
pub fn review_params() -> String {
    format!(
        "bugs={BUGS};runs={REVIEW_RUNS};skill=({:?}, {:?});easy=({:?}, {:?});hard=({:?}, {:?});seed=[{SEED}, 14]",
        SKILL.0, SKILL.1, EASY.0, EASY.1, HARD.0, HARD.1
    )
}

/// Where a review's careful range lies against the true count of bugs:
/// 0 wholly below it, 1 holding it, 2 wholly above it. P_N(K ≤ k) only
/// rises with N and P_N(K ≥ k) only falls, so the range holds N = 60
/// exactly when that lake keeps the catch, and lies below it exactly when
/// the catch is too many for 60 bugs. Decided in exact integers at N = 60.
pub fn range_side(r: &Review) -> Result<u8, String> {
    use explainer_kit::choose;
    let (big, m, n, k) = (BUGS as u64, r.a as u64, r.b as u64, r.both as u64);
    if r.both == 0 || m + n - k > big {
        return Err(format!("review {r:?}: no range, or more bugs seen than exist"));
    }
    let netfuls = |j: u64| {
        if j > m || n - j > big - m {
            0
        } else {
            choose(m, j) * choose(big - m, n - j)
        }
    };
    let total = choose(big, n);
    let le: u128 = (0..=k).map(netfuls).sum();
    let ge = total - (0..k).map(netfuls).sum::<u128>();
    let (le_ok, ge_ok) = (le * 40 > total, ge * 40 > total);
    Ok(match (le_ok, ge_ok) {
        (true, true) => 1,
        (true, false) => 0,
        (false, true) => 2,
        (false, false) => return Err(format!("review {r:?}: both tails under 2.5%")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_the_oracles() {
        let k = panel();
        let first: Vec<u32> = k[..12].to_vec();
        assert_eq!(
            first,
            [101, 103, 91, 97, 94, 103, 100, 91, 108, 101, 90, 107],
            "A-CH09.panel.first12_k"
        );
        assert_eq!(k.iter().filter(|&&k| k == 0).count(), 0, "A-CH09.panel.k_zero_count");
        let missed: Vec<usize> = (0..100)
            .filter(|&i| !(83..=118).contains(&k[i]))
            .map(|i| i + 1)
            .collect();
        assert_eq!(missed, [13, 71], "A-CH12.ring_toss_panel.first100_missed_runs_1based");
        assert_eq!(
            k.iter().filter(|&&k| !(83..=118).contains(&k)).count(),
            464,
            "A-CH12.ring_toss_panel.all_missed"
        );
    }

    #[test]
    fn verify2_replays_the_oracles_stream() {
        let v = verify2();
        assert_eq!(
            v.feller_k.iter().filter(|&&k| !(83..=118).contains(&k)).count(),
            447,
            "A-CH12.ring_toss.all_missed (section E's HRUA draws)"
        );
        let medians: Vec<f64> = v.failures.iter().map(Failure::median).collect();
        assert_eq!(
            medians,
            [
                10_000.0,
                7_751.937_984_496_124,
                7_142.857_142_857_143,
                14_492.753_623_188_406,
                12_500.0,
                12_121.657_361_151_925,
                10_000.0,
                6_250.0,
                25_000.0
            ],
            "A-F22.median.*"
        );
    }

    #[test]
    fn the_reviewers_are_the_oracles() {
        let [(_, skill), (_, difficulty)] = reviewers();
        let est = |r: &[Review]| -> Vec<f64> {
            r.iter()
                .filter(|r| r.both > 0)
                .map(|r| (r.a * r.b) as f64 / r.both as f64)
                .collect()
        };
        assert_eq!(median(&est(&skill)), 60.0, "A-CH14.reviewers.skill.sim_median_estimate");
        assert_eq!(
            median(&est(&difficulty)),
            41.25,
            "A-CH14.reviewers.difficulty.sim_median_estimate"
        );
        let below = est(&skill).iter().filter(|&&e| e < BUGS as f64).count();
        assert_eq!(below, 9_815, "A-CH14.reviewers.skill.sim_share_below_true 0.49075");
    }
}
