//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for `oracle.py` (`./x parrondo facts-check`). Fractions print as
//! the oracle's (`-73443/8446300`), percentages rounded half away from zero
//! (`38.4`), dollars as signed cents (`+1.57`), lists comma-separated and
//! corners in the order 0, 1, 2.

use explainer_kit::Facts;

use crate::model::{
    self, Coins, DEN, Game, MODULUS, PUBLISHED, Play, ROUND, RUN_ROUNDS, Ratio, SEED, START, lcm, periodic, schedule,
};

/// The rounds of the player's runs chapter 01 draws one toss at a time.
pub const FIRST: usize = 8;
/// The rounds after which chapter 17 looks back at B's run.
pub const SHORT: usize = 1000;
/// The crowd's rounds chapter 08 shows one by one.
pub const CROWD_ROUNDS: usize = 6;
/// How far chapter 08 looks for the crowd's settling, and how close counts.
pub const SETTLE_LOOK: usize = 200;
pub const SETTLE_WITHIN: f64 = 0.5;
/// Chapter 08's real crowd: the rounds shown one by one, and the rounds
/// counted.
pub const REAL_SHOWN: usize = 8;
pub const REAL_ROUNDS: usize = 1000;
/// Chapter 11's split crowd under the mix, round by round from B's 10, 4, 12.
pub const MIX_SHIFT_ROUNDS: usize = 6;
/// The finite-play horizons of chapter 17.
pub const HORIZON: usize = 100;
pub const LONG_HORIZON: usize = 1000;
/// The two shares of game A chapter 16 tests, as (num, den).
pub const SHARES: [(i64, i64); 2] = [(1, 20), (9, 10)];
/// The ordinary coins of chapter 04, in hundredths.
pub const ORDINARY: [i64; 2] = [49, 48];
/// The counts chapter 02 and K03 take remainders of.
pub const REMAINDERS: [i64; 8] = [6, 7, 8, 9, 10, 600, -1, -3];
/// The schedules chapter 16 compares (and Ethier & Lee pin).
pub const PATTERNS: [&str; 4] = ["AB", "AABB", "ABB", "AAB"];
/// Chapter 16's rounds of each schedule played one at a time, from game
/// B's settled crowd.
pub const PLAYED_ROUNDS: usize = 24;
/// Shares of A's rounds either side of the curve's crossings, 7.0, 7.1,
/// 84.7 and 84.8 % (16_14), and the whole percentages nearest them.
pub const EDGE_SHARES: [(i64, i64); 4] = [(70, 1000), (71, 1000), (847, 1000), (848, 1000)];
pub const WHOLE_EDGES: [(i64, i64); 2] = [(7, 100), (85, 100)];
/// Where chapter 17's crowds start, in dollars (on corner 0), and its
/// window of dollars won or lost, every histogram's.
pub const CROWD_START: i64 = 0;
pub const WINDOW: (i64, i64) = (-100, 140);

const HARMER_ABBOTT: &str =
    "Harmer & Abbott, Nature 402, 864 (1999): p = 1/2 - e, p2 = 1/10 - e, p3 = 3/4 - e, e = 0.005";
const ETHIER_LEE: &str = "Ethier & Lee, Electron. J. Probab. 14 (2009), paper 62, capital-dependent games at e = 0";

fn csv<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

fn third() -> Ratio {
    Ratio::new(1, 3)
}

/// The three coins as fractions: A, bad, good.
pub fn coins(c: &Coins) -> [Ratio; 3] {
    [c.a, c.bad, c.good].map(|x| Ratio::new(x.into(), DEN.into()))
}

/// Exact percentages of several chances ("49.5,9.5,74.5").
fn exact_pcts(values: impl IntoIterator<Item = Ratio>) -> Result<String, String> {
    Ok(csv(values
        .into_iter()
        .map(Ratio::exact_pct)
        .collect::<Result<Vec<_>, _>>()?))
}

/// B's chance of winning if each remainder took a third of the rounds.
pub fn thirds_win(c: &Coins) -> Ratio {
    c.game_b().win_at(&[third(); 3])
}

/// The average of the two games' averages: what "halfway between" predicts.
pub fn halfway(c: &Coins) -> Ratio {
    (c.game_a().mean() + c.game_b().mean()) / Ratio::int(2)
}

/// The share at corner 0 where B breaks even, on its line (good coin minus
/// good-minus-bad per unit of share).
pub fn fair_share(c: &Coins) -> Ratio {
    let [_, bad, good] = coins(c);
    (good - Ratio::new(1, 2)) / (good - bad)
}

/// The players arriving at each corner, by the corner they come from, lowest
/// first ("1+9" at corner 0: one back from 1, nine forward from 2).
pub fn arrivals(forward: &[Ratio; 3], back: &[Ratio; 3]) -> [String; 3] {
    [0usize, 1, 2].map(|j| {
        (0..3)
            .filter_map(|i| {
                if (i + 1) % 3 == j {
                    Some(forward[i].to_string())
                } else if (i + 2) % 3 == j {
                    Some(back[i].to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("+")
    })
}

/// The crowd of `n` at corner 0 after each of the first `rounds` rounds of
/// `g`, exactly.
pub fn crowd_rounds(g: &Game, n: i64, rounds: usize) -> Vec<[Ratio; 3]> {
    rounds_from([Ratio::int(n), Ratio::ZERO, Ratio::ZERO], g, rounds)
}

/// A crowd split by the chances, started at `start`, after each of the
/// first `rounds` rounds of `g`, exactly.
pub fn rounds_from(start: [Ratio; 3], g: &Game, rounds: usize) -> Vec<[Ratio; 3]> {
    let mut v = start;
    (0..rounds)
        .map(|_| {
            v = model::evolve(&v, g);
            v
        })
        .collect()
}

/// The verdict on every game whose three chances are tenths 1 to 9: how
/// many there are, how many whose summed mean disagrees in sign with the
/// products (none may), and how many are fair.
pub fn sign_sweep() -> (usize, usize, usize) {
    let (mut games, mut wrong, mut fair) = (0, 0, 0);
    for p0 in 1..10 {
        for p1 in 1..10 {
            for p2 in 1..10 {
                let g = Game {
                    up: [p0, p1, p2].map(|t| t * DEN / 10),
                    den: DEN,
                };
                let s = g.shares();
                let summed = (0..3).fold(Ratio::ZERO, |a, i| {
                    a + s[i] * (Ratio::int(2) * g.chance(i) - Ratio::ONE)
                });
                let (f, b) = g.products();
                games += 1;
                wrong += usize::from(summed.signum() != (f - b).signum());
                fair += usize::from(f == b);
            }
        }
    }
    (games, wrong, fair)
}

/// Every mix of two losing ordinary coins (hundredths 1 to 49) at shares
/// 1/10, 1/2 and 9/10: how many, and how many do not lose.
pub fn ordinary_sweep() -> (usize, usize) {
    let (mut cases, mut winning) = (0, 0);
    for p in 1..50 {
        for q in 1..50 {
            for (num, den) in [(1, 10), (1, 2), (9, 10)] {
                let a = Game { up: [p; 3], den: 100 };
                let b = Game { up: [q; 3], den: 100 };
                cases += 1;
                winning += usize::from(a.mixed(&b, num, den).mean() >= Ratio::ZERO);
            }
        }
    }
    (cases, winning)
}

/// A float percentage at one decimal ("68.9"); the oracle checks each value
/// it is used for is far from a rounding boundary.
fn pct_f64(x: f64) -> String {
    format!("{:.1}", 100.0 * x)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let (pub_a, pub_b, pub_mix) = (PUBLISHED.game_a(), PUBLISHED.game_b(), PUBLISHED.mix());
    let (r_a, r_b, r_mix) = (ROUND.game_a(), ROUND.game_b(), ROUND.mix());

    // the rules (01, 02)
    f.add("rules.modulus", MODULUS)?;
    f.add("rules.handicap", Ratio::new(model::HANDICAP.into(), DEN.into()))?;
    f.pin(
        "coins.published",
        csv(coins(&PUBLISHED)),
        "99/200,19/200,149/200",
        HARMER_ABBOTT,
    )?;
    f.add("coins.published.heads.pct", exact_pcts(coins(&PUBLISHED))?)?;
    f.add(
        "coins.published.tails.pct",
        exact_pcts(coins(&PUBLISHED).map(|p| Ratio::ONE - p))?,
    )?;
    // the paper itself (13_10's card)
    f.pin("source.volume", 402, "402", HARMER_ABBOTT)?;
    f.pin("source.page", 864, "864", HARMER_ABBOTT)?;
    f.pin("source.year", 1999, "1999", HARMER_ABBOTT)?;
    f.add("coins.round", csv(coins(&ROUND)))?;
    f.add("coins.round.heads.pct", exact_pcts(coins(&ROUND))?)?;
    f.add(
        "coins.round.tails.pct",
        exact_pcts(coins(&ROUND).map(|p| Ratio::ONE - p))?,
    )?;
    f.add(
        "remainder.examples",
        csv(REMAINDERS.map(|n| format!("{n}:{}", n.rem_euclid(MODULUS)))),
    )?;

    // game A's hundred rounds (01_6 to 01_8)
    let a_coin = coins(&PUBLISHED)[0];
    f.add("a.hundred.wins", a_coin.exact_pct()?)?;
    f.add("a.hundred.losses", (Ratio::ONE - a_coin).exact_pct()?)?;
    f.add("a.hundred.net", Ratio::int(100) * (Ratio::int(2) * a_coin - Ratio::ONE))?;

    // the player (01, 03, 04, 05, 07, 17)
    f.add("run.seed", SEED)?;
    f.add("run.start", START)?;
    f.add("run.rounds", RUN_ROUNDS)?;
    let runs = [("a", Play::A), ("b", Play::B), ("mix", Play::Mix)].map(|(k, p)| (k, model::player(p)));
    for (key, run) in &runs {
        f.add(&format!("run.{key}.net"), run.net())?;
        f.add(&format!("run.{key}.at0.rounds"), run.at_zero)?;
        f.add(&format!("run.{key}.at0.pct"), run.share_at_zero().pct(1))?;
        f.add(&format!("run.{key}.low"), run.low())?;
        f.add(&format!("run.{key}.high"), run.high())?;
    }
    f.add("run.a.first", csv(&runs[0].1.dollars[..=FIRST]))?;
    f.add("run.b.net.short", runs[1].1.net_after(SHORT))?;

    // B by equal thirds, and B's verdict (03)
    f.add("pub.b.thirds.win", thirds_win(&PUBLISHED))?;
    f.add("pub.b.thirds.win.pct", thirds_win(&PUBLISHED).pct(1))?;
    f.add("thirds.pct", third().pct(1))?;
    let pb = pub_b.shares();
    f.add("pub.b.weights", csv(pb))?;
    f.add("pub.b.weights.pct", csv(pb.map(|w| w.pct(1))))?;
    f.add("pub.b.win", pub_b.win())?;
    f.add("pub.b.mean", pub_b.mean())?;
    f.add("pub.b.mean.cents", pub_b.mean().cents(2))?;
    f.add("pub.a.mean", pub_a.mean())?;
    f.add("pub.a.mean.cents", pub_a.mean().cents(2))?;
    // the dashed line of 01_9 and its "about a thousand dollars below"
    f.add("run.a.expected.net", Ratio::int(RUN_ROUNDS as i64) * pub_a.mean())?;

    // why the mix looks hopeless (04)
    let ordinary = ORDINARY.map(|p| Ratio::new(p.into(), 100));
    f.add("ordinary.coins.pct", exact_pcts(ordinary)?)?;
    f.add(
        "ordinary.mixed.pct",
        ((ordinary[0] + ordinary[1]) / Ratio::int(2)).exact_pct()?,
    )?;
    f.add("pub.halfway.mean", halfway(&PUBLISHED))?;
    f.add("pub.halfway.mean.cents", halfway(&PUBLISHED).cents(2))?;

    // the mix with the published coins (04, 05, 13)
    let pm = pub_mix.shares();
    f.add("pub.mix.coins", csv([pub_mix.chance(0), pub_mix.chance(1)]))?;
    f.add("pub.mix.coins.pct", exact_pcts([pub_mix.chance(0), pub_mix.chance(1)])?)?;
    f.add("pub.mix.weights", csv(pm))?;
    f.add("pub.mix.weights.pct", csv(pm.map(|w| w.pct(1))))?;
    f.add("pub.mix.win", pub_mix.win())?;
    f.add("pub.mix.win.pct", pub_mix.win().pct(1))?;
    f.add("pub.mix.mean", pub_mix.mean())?;
    f.add("pub.mix.mean.cents", pub_mix.mean().cents(2))?;
    f.add("pub.mix.b_rounds.win", pub_b.win_at(&pm))?;
    f.add("pub.mix.b_rounds.win.pct", pub_b.win_at(&pm).pct(1))?;
    f.add("pub.b.fair_share", fair_share(&PUBLISHED))?;
    f.add("pub.b.fair_share.pct", fair_share(&PUBLISHED).pct(1))?;

    // the round-number coins (07 to 12)
    f.add("round.b.thirds.win", thirds_win(&ROUND))?;
    f.add("round.b.thirds.win.pct", thirds_win(&ROUND).pct(1))?;
    let rb = r_b.shares();
    f.pin("round.b.weights", csv(rb), "5/13,2/13,6/13", ETHIER_LEE)?;
    f.add("round.b.weights.pct", csv(rb.map(|w| w.pct(1))))?;
    f.add("round.b.win", r_b.win())?;
    f.add("round.b.mean", r_b.mean())?;
    let [_, bad, good] = coins(&ROUND);
    f.add("round.b.line", csv([good, good - bad]))?;
    f.add("round.b.line.pct", exact_pcts([good, good - bad])?)?;
    f.add("round.b.fair_share", fair_share(&ROUND))?;
    f.add("round.die.win", ROUND.die_chosen_win())?;
    f.add("pub.die.win", PUBLISHED.die_chosen_win())?;

    // the crowd of 26 (08, 09)
    let n = model::smallest_whole_crowd(&r_b);
    let counts = rb.map(|w| Ratio::int(n) * w);
    let (forward, back) = model::flows(&r_b, n);
    f.add("crowd.size", n)?;
    f.add("crowd.counts", csv(counts))?;
    f.add("crowd.forward", csv(forward))?;
    f.add("crowd.back", csv(back))?;
    // 14_12: no edge of B's settled crowd carries a net flow
    f.add("crowd.net", csv([0usize, 1, 2].map(|i| forward[i] - back[(i + 1) % 3])))?;
    f.add("crowd.arrivals", csv(arrivals(&forward, &back)))?;
    f.add("crowd.winners", forward.iter().fold(Ratio::ZERO, |a, &x| a + x))?;
    f.add("crowd.share0.pct", rb[0].pct(1))?;
    for (k, v) in crowd_rounds(&r_b, n, CROWD_ROUNDS).iter().enumerate() {
        f.add(&format!("crowd.round.{}", k + 1), csv(v.map(|x| x.fixed(1))))?;
    }
    let target = counts.map(Ratio::to_f64);
    let settle = model::settles_from(&r_b, n as f64, &target, SETTLE_WITHIN, SETTLE_LOOK)
        .ok_or("the crowd never settles within half a player")?;
    f.add("crowd.within_half_from", settle)?;
    // 10_4: of the good coin's wins (corners 1 and 2), the share at corner 2
    f.add("crowd.good_wins_at2", forward[2] / (forward[1] + forward[2]))?;

    // 08_7: 26 real players started at 10, 4, 12, tossing real coins
    let start = counts.map(|c| usize::try_from(c.num()).expect("whole players"));
    let real = model::real_crowd(&r_b, start, REAL_ROUNDS, SEED);
    for (k, c) in real[..REAL_SHOWN].iter().enumerate() {
        f.add(&format!("realcrowd.round.{}", k + 1), csv(c))?;
    }
    f.add("realcrowd.rounds", REAL_ROUNDS)?;
    f.add("realcrowd.exact", real.iter().filter(|&&c| c == start).count())?;
    let real_mean = [0, 1, 2].map(|i| {
        let total: usize = real.iter().map(|c| c[i]).sum();
        Ratio::new(total as i128, REAL_ROUNDS as i128).fixed(1)
    });
    f.add("realcrowd.mean", csv(real_mean))?;

    // 11_5: the mix's split crowd, round by round from B's settled 10, 4, 12
    for (k, v) in rounds_from(counts, &r_mix, MIX_SHIFT_ROUNDS).iter().enumerate() {
        f.add(
            &format!("mixshift.round.{}", k + 1),
            csv(v.map(|x| (x / Ratio::int(n)).pct(1))),
        )?;
    }

    // the mix with the round-number coins (11, 12, 14)
    let rm = r_mix.shares();
    f.add("round.mix.coins", csv([r_mix.chance(0), r_mix.chance(1)]))?;
    f.add("round.mix.coins.pct", exact_pcts([r_mix.chance(0), r_mix.chance(1)])?)?;
    f.add(
        "round.mix.tails.pct",
        exact_pcts([Ratio::ONE - r_mix.chance(0), Ratio::ONE - r_mix.chance(1)])?,
    )?;
    f.add("round.mix.weights", csv(rm))?;
    f.add("round.mix.weights.pct", csv(rm.map(|w| w.pct(1))))?;
    f.add("round.mix.win", r_mix.win())?;
    f.add("round.mix.win.pct", r_mix.win().pct(1))?;
    f.add("round.mix.win.pct2", r_mix.win().pct(2))?;
    f.add("round.mix.b_rounds.win", r_b.win_at(&rm))?;
    f.add("round.mix.b_rounds.win.pct", r_b.win_at(&rm).pct(1))?;
    f.add("round.mix.b_rounds.win.pct2", r_b.win_at(&rm).pct(2))?;
    f.add("round.mix.a_rounds.win", r_a.win_at(&rm))?;
    f.pin("round.mix.mean", r_mix.mean(), "18/709", ETHIER_LEE)?;
    f.add("round.mix.mean.cents", r_mix.mean().cents(2))?;
    let toward = (rb[0] - rm[0]) / (rb[0] - third());
    f.add("round.mix.toward_thirds.pct", toward.pct(0))?;
    let cost = r_mix.mean() - pub_mix.mean();
    f.add("handicap.mix_cost", cost)?;
    f.add("handicap.mix_cost.cents", (cost * Ratio::int(100)).fixed(2))?;

    let m = model::smallest_whole_crowd(&r_mix);
    let mcounts = rm.map(|w| Ratio::int(m) * w);
    let (mforward, mback) = model::flows(&r_mix, m);
    let net = [0usize, 1, 2].map(|i| mforward[i] - mback[(i + 1) % 3]);
    let sum = |xs: &[Ratio; 3]| xs.iter().fold(Ratio::ZERO, |a, &x| a + x);
    f.add("mixcrowd.size", m)?;
    f.add("mixcrowd.counts", csv(mcounts))?;
    f.add("mixcrowd.forward", csv(mforward))?;
    f.add("mixcrowd.back", csv(mback))?;
    f.add("mixcrowd.net", csv(net))?;
    f.add("mixcrowd.gain", sum(&mforward) - sum(&mback))?;
    f.add("mixcrowd.winners", sum(&mforward))?;
    f.add("mixcrowd.losers", sum(&mback))?;

    // 14_10, 14_11: 1,418 real players started at 490, 360, 568, tossing
    // real coins (08_7's stream and player order), counted as crossings
    let mstart = mcounts.map(|c| usize::try_from(c.num()).expect("whole players"));
    let crossings = model::real_crossings(&r_mix, mstart, REAL_ROUNDS, SEED);
    let edge_net = |moved: &[[usize; 3]; 2], e: usize| moved[0][e] as i64 - moved[1][(e + 1) % 3] as i64;
    let first = &crossings[0];
    let rnet = [0, 1, 2].map(|e| edge_net(first, e));
    f.add("realmix.round.1.forward", csv(first[0]))?;
    f.add("realmix.round.1.back", csv(first[1]))?;
    f.add("realmix.round.1.net", csv(rnet))?;
    f.add(
        "realmix.round.1.after",
        csv([0, 1, 2].map(|i| first[0][(i + 2) % 3] + first[1][(i + 1) % 3])),
    )?;
    f.add("realmix.round.1.gain", rnet.iter().sum::<i64>())?;
    f.add("realmix.rounds", REAL_ROUNDS)?;
    let totals = [0, 1, 2].map(|e| crossings.iter().map(|m| edge_net(m, e)).sum::<i64>());
    let per_round = |t: i64| Ratio::new(t.into(), REAL_ROUNDS as i128).fixed(1);
    f.add("realmix.mean.net", csv(totals.map(per_round)))?;
    f.add("realmix.mean.gain", per_round(totals.iter().sum()))?;

    // the two products (15)
    let (fb, bb) = r_b.products();
    f.add("round.products.b", csv([fb, bb]))?;
    let (fm, bm) = r_mix.products();
    f.add("round.products.mix", csv([fm, bm]))?;
    let common = lcm(fm.den(), bm.den());
    f.add(
        "round.products.mix.common",
        format!(
            "{}/{common},{}/{common}",
            fm.num() * (common / fm.den()),
            bm.num() * (common / bm.den())
        ),
    )?;
    // 15_12: the one line on the mix's crowd, net × (p₂ + q₁ q₂) = w₀ ×
    // (p₀ p₁ p₂ − q₀ q₁ q₂), in players on both sides
    let factor = r_mix.chance(2) + (Ratio::ONE - r_mix.chance(1)) * (Ratio::ONE - r_mix.chance(2));
    let sides = net[0] * factor;
    if sides != mcounts[0] * (fm - bm) {
        return Err("15_12's one line disagrees on the mix's crowd".into());
    }
    f.add("round.mix.sign.factor", factor)?;
    f.add("round.products.mix.diff", fm - bm)?;
    f.add("round.mix.sign.sides", sides)?;
    for (name, game) in [("a", pub_a), ("b", pub_b), ("mix", pub_mix)] {
        let (fw, bw) = game.products();
        f.add(&format!("pub.products.{name}"), csv([fw, bw]))?;
        f.add(&format!("pub.products.{name}.fixed"), csv([fw.fixed(4), bw.fixed(4)]))?;
    }

    // schedules (16)
    let pinned = ["0", "4/163", "2416/35601", "32/1609"];
    for (pattern, want) in PATTERNS.iter().zip(pinned) {
        let (value, back) = periodic(&schedule(pattern, &ROUND));
        if !back {
            return Err(format!("{pattern}: the shares do not return after a period"));
        }
        f.pin(
            &format!("round.sched.{}", pattern.to_lowercase()),
            value,
            want,
            ETHIER_LEE,
        )?;
    }
    for pattern in ["AB", "AABB"] {
        let (value, _) = periodic(&schedule(pattern, &PUBLISHED));
        let key = pattern.to_lowercase();
        f.add(&format!("pub.sched.{key}"), value)?;
        f.add(&format!("pub.sched.{key}.cents"), value.cents(2))?;
    }

    // A's share in the mix (16_5)
    f.add("pub.share.low.pct", pct_f64(model::crossing(&PUBLISHED, 0.0, 0.5)))?;
    f.add("pub.share.high.pct", pct_f64(model::crossing(&PUBLISHED, 0.5, 1.0)))?;
    for (num, den) in SHARES {
        f.add(
            &format!("pub.share.{num}_{den}.cents"),
            PUBLISHED.mix_share(num, den).mean().cents(2),
        )?;
    }

    // finite play (17)
    let (ahead, behind, level) = model::horizon(&pub_mix, HORIZON, 0);
    f.add("horizon.short.mix", csv([ahead, behind, level].map(pct_f64)))?;
    f.add(
        "horizon.long.mix.ahead.pct",
        pct_f64(model::horizon(&pub_mix, LONG_HORIZON, 0).0),
    )?;
    f.add(
        "horizon.long.a.behind.pct",
        pct_f64(model::horizon(&pub_a, LONG_HORIZON, 0).1),
    )?;
    f.add(
        "horizon.long.b.behind.pct",
        pct_f64(model::horizon(&pub_b, LONG_HORIZON, 0).1),
    )?;

    // 16: what each schedule's rounds meet. A's share of the rounds is a
    // half in the coin-toss mix, in AB and in AABB alike
    let half = Ratio::new(1, 2);
    if (0..3).any(|i| pub_mix.chance(i) != PUBLISHED.mix_share(1, 2).chance(i)) {
        return Err("16_2 says the coin-toss mix plays A on half the rounds".into());
    }
    let a_share = |p: &str| Ratio::new(p.matches('A').count() as i128, p.len() as i128);
    f.add("sched.a_share", csv([half, a_share("AB"), a_share("AABB")]))?;
    // 16_6: alternation played round by round from B's settled crowd
    let ab_played = model::played(&schedule("AB", &PUBLISHED), pb.map(|w| w.to_f64()), PLAYED_ROUNDS);
    f.add("pub.sched.ab.first.b.at0.pct", pct_f64(ab_played[1][0]))?;
    // 16_7, 16_9, 16_10: the cycles AB and AABB settle into (exact); each
    // round's gain at the shares it meets averages to the schedule's
    let gain = |g: &Game, v: &[Ratio; 3]| Ratio::int(2) * g.win_at(v) - Ratio::ONE;
    let pct2 = |v: &[Ratio; 3]| csv(v.iter().map(|x| x.pct(2)));
    let mut cycles = Vec::with_capacity(2);
    for pattern in ["AB", "AABB"] {
        let games = schedule(pattern, &PUBLISHED);
        let phases = model::cycle(&games);
        let total = games
            .iter()
            .zip(&phases)
            .fold(Ratio::ZERO, |acc, (g, v)| acc + gain(g, v));
        if total / Ratio::int(games.len() as i64) != periodic(&games).0 {
            return Err(format!(
                "{pattern}: its cycle's rounds do not average to the schedule's"
            ));
        }
        cycles.push(phases);
    }
    let (ab, aabb) = (&cycles[0], &cycles[1]);
    f.add("pub.sched.ab.cycle.a.pct2", pct2(&ab[0]))?;
    f.add("pub.sched.ab.cycle.b.pct2", pct2(&ab[1]))?;
    f.add("pub.sched.ab.cycle.b.pct", csv(ab[1].map(|x| x.pct(1))))?;
    f.add("pub.sched.ab.b.cents", gain(&pub_b, &ab[1]).cents(2))?;
    f.add("pub.sched.aabb.b.at0.pct", csv([aabb[2][0].pct(1), aabb[3][0].pct(1)]))?;
    f.add(
        "pub.sched.aabb.b.cents",
        csv([2, 3].map(|k| gain(&pub_b, &aabb[k]).cents(2))),
    )?;
    // 16_7: A's round refills corner 0 from the crowd B's round leaves
    let (from2, from1) = (a_coin * ab[0][2], (Ratio::ONE - a_coin) * ab[0][1]);
    if from2 + from1 != ab[1][0] {
        return Err("16_7 says corner 0 is refilled from corners 2 and 1".into());
    }
    f.add(
        "pub.sched.ab.refill.pct2",
        csv([from2, from1, ab[1][0]].map(|x| x.pct(2))),
    )?;
    // 16_11: two ordinary coins, which ignore the money, in each schedule
    let plain = Coins {
        a: ORDINARY[0] * DEN / 100,
        bad: ORDINARY[1] * DEN / 100,
        good: ORDINARY[1] * DEN / 100,
    };
    let plain_sched = [
        plain.mix().mean(),
        periodic(&schedule("AB", &plain)).0,
        periodic(&schedule("AABB", &plain)).0,
    ];
    f.add("ordinary.sched.cents", csv(plain_sched.map(|x| x.cents(2))))?;
    // 16_12 to 16_14: where A's share stops winning, rounded and exactly
    f.add(
        "pub.share.points",
        csv(SHARES.map(|(n, d)| Ratio::new(n.into(), d.into()))),
    )?;
    let (low, high) = (
        model::crossing(&PUBLISHED, 0.0, 0.5),
        model::crossing(&PUBLISHED, 0.5, 1.0),
    );
    f.add(
        "pub.share.crossings.fine",
        format!("{:.2},{:.2}", 100.0 * low, 100.0 * high),
    )?;
    f.add(
        "pub.share.crossings.whole",
        format!("{:.0},{:.0}", 100.0 * low, 100.0 * high),
    )?;
    let edge = EDGE_SHARES.map(|(n, d)| PUBLISHED.mix_share(n, d).mean());
    f.add(
        "pub.share.edge.pct",
        csv(EDGE_SHARES.map(|(n, d)| Ratio::new(n.into(), d.into()).pct(1))),
    )?;
    f.add("pub.share.edge.cents", csv(edge.map(|m| m.cents(4))))?;
    f.add("pub.share.edge.sign", csv(edge.map(|m| m.signum())))?;
    f.add(
        "pub.share.whole.cents",
        csv(WHOLE_EDGES.map(|(n, d)| PUBLISHED.mix_share(n, d).mean().cents(4))),
    )?;

    // 17: the crowd's exact distributions of dollars won or lost, one
    // pass per game (floats; the oracle's integer programme agrees to
    // every printed digit)
    let rounds = [HORIZON, LONG_HORIZON];
    f.add("horizon.rounds", csv(rounds))?;
    f.add("horizon.rounds.ratio", LONG_HORIZON / HORIZON)?;
    f.add("horizon.start", CROWD_START)?;
    if CROWD_START.rem_euclid(MODULUS) != 0 {
        return Err("17_2 starts the crowd on corner 0".into());
    }
    f.add("run.short", SHORT)?;
    let snaps = model::snapshots(&pub_mix, CROWD_START, &rounds);
    let cats: Vec<_> = snaps.iter().map(|d| model::categories(d, LONG_HORIZON)).collect();
    let short = model::horizon(&pub_mix, HORIZON, CROWD_START);
    if [cats[0].0 - short.0, cats[0].1 - short.1, cats[0].2 - short.2]
        .iter()
        .any(|d| d.abs() > 1e-12)
    {
        return Err("17's bars disagree with the horizon's shares".into());
    }
    f.add("horizon.long.mix", csv([cats[1].0, cats[1].1, cats[1].2].map(pct_f64)))?;
    let (top, tallest) = snaps[0]
        .iter()
        .enumerate()
        .fold((0, 0.0), |best, (k, &w)| if w > best.1 { (k, w) } else { best });
    let net = |k: usize| k as i64 - LONG_HORIZON as i64;
    f.add(
        "horizon.short.mix.tallest",
        format!("{}:{}", net(top), pct_f64(tallest)),
    )?;
    let from_start = model::snapshots(&pub_b, START, &[LONG_HORIZON]).remove(0);
    let inside = |d: &[f64]| -> f64 {
        d.iter()
            .enumerate()
            .filter(|&(k, _)| (WINDOW.0..=WINDOW.1).contains(&net(k)))
            .map(|(_, w)| w)
            .sum()
    };
    f.add(
        "horizon.window.pct",
        csv([inside(&snaps[0]), inside(&snaps[1]), inside(&from_start)].map(pct_f64)),
    )?;
    let moments: Vec<(f64, f64)> = snaps.iter().map(|d| model::moments(d, LONG_HORIZON)).collect();
    f.add("horizon.mix.mean", csv(moments.iter().map(|m| format!("{:+.2}", m.0))))?;
    f.add("horizon.mix.spread", csv(moments.iter().map(|m| format!("{:.1}", m.1))))?;
    f.add(
        "horizon.mix.spread2",
        csv(moments.iter().map(|m| format!("{:.2}", m.1))),
    )?;
    let ratio = moments[1].1 / moments[0].1;
    f.add("horizon.mix.spread.ratio", format!("{ratio:.2}"))?;
    f.add("horizon.mix.spread.ratio.fine", format!("{ratio:.4}"))?;
    let sqrt = ((LONG_HORIZON / HORIZON) as f64).sqrt();
    f.add("horizon.rounds.sqrt.fine", format!("{sqrt:.4}"))?;
    // the crowd's average against rounds × the mix's average: starting on
    // corner 0 costs the same once, at 100 rounds and at 1,000
    let at_rate = rounds.map(|n| Ratio::int(n as i64) * pub_mix.mean());
    f.add("horizon.mix.at_rate", csv(at_rate.map(|x| x.fixed(2))))?;
    let cost = [0, 1].map(|k| at_rate[k].to_f64() - moments[k].0);
    if (cost[0] - cost[1]).abs() > 0.005 {
        return Err("17_5 says starting on corner 0 costs the same, once".into());
    }
    f.add("horizon.mix.start_cost", format!("{:.2}", cost[0]))?;
    // 17_10: the player's start and its 1,000 rounds of game B
    if SHORT != LONG_HORIZON {
        return Err("17_10 sets the player's 1,000 rounds against the crowd's".into());
    }
    let b_ahead = from_start[LONG_HORIZON + 1..].iter().sum::<f64>();
    let at_least = usize::try_from(LONG_HORIZON as i64 + runs[1].1.net_after(SHORT)).map_err(|e| e.to_string())?;
    let b_lucky = from_start[at_least..].iter().sum::<f64>();
    f.add("horizon.long.b.from_start", csv([b_ahead, b_lucky].map(pct_f64)))?;

    // the closing puzzle (18)
    let (reverse, swapped) = (PUBLISHED.b_at_zero(), PUBLISHED.a_at_zero());
    f.add("pub.reverse.mean", reverse.mean())?;
    f.add("pub.reverse.mean.cents", reverse.mean().cents(1))?;
    f.add("pub.swapped.mean", swapped.mean())?;
    f.add("pub.swapped.mean.cents", swapped.mean().cents(1))?;

    // the edges of the always/never sentences
    let (games, wrong, fair) = sign_sweep();
    f.add("sign.games", games)?;
    f.add("sign.violations", wrong)?;
    f.add("sign.fair_games", fair)?;
    let (cases, winning) = ordinary_sweep();
    f.add("ordinary.sweep.cases", cases)?;
    f.add("ordinary.sweep.winning", winning)?;

    f.self_check()?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(f: &Facts, key: &str) -> String {
        f.get(key).unwrap_or_else(|| panic!("no fact {key}")).to_owned()
    }

    /// The facts the oracle printed on 2026-10-24 for the chapters' visible
    /// arithmetic (`python3 -I -B oracle.py`, 45/45 cross-checks); the full
    /// comparison is `./x parrondo facts-check`.
    #[test]
    fn the_headline_numbers() {
        let f = facts().unwrap();
        for (key, want) in [
            ("pub.a.mean.cents", "-1.00"),
            ("pub.b.mean.cents", "-0.87"),
            ("pub.mix.mean.cents", "+1.57"),
            ("pub.b.thirds.win.pct", "52.8"),
            ("pub.halfway.mean.cents", "-0.93"),
            ("crowd.counts", "10,4,12"),
            ("crowd.arrivals", "1+9,1+3,9+3"),
            ("crowd.round.2", "18.2,5.9,2.0"),
            ("crowd.within_half_from", "24"),
            ("mixcrowd.counts", "490,360,568"),
            ("mixcrowd.net", "12,12,12"),
            ("round.products.mix.common", "75/640,63/640"),
            ("pub.sched.ab.cents", "-0.67"),
            ("pub.sched.aabb.cents", "+1.47"),
            ("pub.share.low.pct", "7.0"),
            ("pub.share.high.pct", "84.7"),
            ("horizon.short.mix", "51.7,39.6,8.7"),
            ("horizon.long.mix.ahead.pct", "68.9"),
            ("pub.reverse.mean.cents", "-27.6"),
            ("pub.swapped.mean.cents", "+31.4"),
            ("run.a.net", "-1062"),
            ("run.a.expected.net", "-1000"),
            ("run.b.net", "-548"),
            ("run.mix.net", "1748"),
            ("run.b.at0.pct", "38.4"),
            ("run.mix.at0.pct", "34.5"),
            ("run.b.net.short", "16"),
            ("crowd.good_wins_at2", "3/4"),
            ("realcrowd.round.1", "9,6,11"),
            ("realcrowd.round.6", "14,2,10"),
            ("realcrowd.exact", "40"),
            ("realcrowd.mean", "10.0,4.0,12.0"),
            ("mixshift.round.1", "34.6,28.8,36.5"),
            ("mixshift.round.2", "33.7,24.1,42.3"),
            ("sign.violations", "0"),
            ("ordinary.sweep.winning", "0"),
        ] {
            assert_eq!(get(&f, key), want, "{key}");
        }
    }

    /// The visible sums check: 2.54 − 0.97 = 1.57 (13_7), ½ · 50 + ½ · 52.54
    /// = 51.27 (12_9 prints it at one decimal, with ≈), 3 × 12 = 36 = 727 −
    /// 691 (14_8), 1 + 3 + 9 = 13 (09_6).
    #[test]
    fn the_sums_on_screen_add_up() {
        let f = facts().unwrap();
        let cents = |k: &str| get(&f, k).parse::<f64>().unwrap();
        let shown = cents("round.mix.mean.cents") - cents("handicap.mix_cost.cents");
        assert!((shown - cents("pub.mix.mean.cents")).abs() < 1e-9);
        let half = 0.5 * 50.0 + 0.5 * cents("round.mix.b_rounds.win.pct2");
        assert!((half - cents("round.mix.win.pct2")).abs() < 0.0051);
        assert_eq!(get(&f, "mixcrowd.gain"), "36");
        assert_eq!(get(&f, "crowd.winners"), "13");
        let halfway = 0.5 * (cents("pub.a.mean.cents") + cents("pub.b.mean.cents"));
        assert!((halfway - cents("pub.halfway.mean.cents")).abs() < 0.0051);
    }

    /// 08_7 and 11_5 say what the computed crowds do: the real crowd jumps
    /// off 10, 4, 12 at once, lands on it only now and then, and averages
    /// it; the mix's split crowd leaves corner 0 within a round, then
    /// sloshes across its limit rather than falling steadily.
    #[test]
    fn the_crowds_do_what_the_words_say() {
        let start = [10, 4, 12];
        let real = model::real_crowd(&ROUND.game_b(), start, REAL_ROUNDS, SEED);
        assert_ne!(real[0], start);
        let exact = real.iter().filter(|&&c| c == start).count();
        assert!(exact > 0 && exact < REAL_ROUNDS / 10, "{exact}");
        let limit = ROUND.mix().shares()[0];
        let zero: Vec<Ratio> = rounds_from(start.map(|c| Ratio::int(c as i64)), &ROUND.mix(), MIX_SHIFT_ROUNDS)
            .iter()
            .map(|v| v[0] / Ratio::int(26))
            .collect();
        assert!(zero[0] < ROUND.game_b().shares()[0]);
        assert!((zero[0] - limit).to_f64().abs() < 0.005);
        let crossings = zero
            .windows(2)
            .filter(|w| (w[0] - limit).signum() * (w[1] - limit).signum() < 0)
            .count();
        assert!(crossings >= 2, "{crossings}");
    }

    /// The negative controls: the attractive wrong answers are wrong.
    #[test]
    fn the_attractive_wrong_answers_fail() {
        // averaging the averages predicts a loss; the mix wins
        assert!(halfway(&PUBLISHED) < Ratio::ZERO && PUBLISHED.mix().mean() > Ratio::ZERO);
        // equal thirds would make B a winner; B loses
        assert!(thirds_win(&PUBLISHED) > Ratio::new(1, 2) && PUBLISHED.game_b().mean() < Ratio::ZERO);
        // a die choosing B's coin wins: the money's rule is what hurts
        assert!(PUBLISHED.die_chosen_win() > Ratio::new(1, 2));
        // strict alternation loses: switching is not always good
        assert!(periodic(&schedule("AB", &PUBLISHED)).0 < Ratio::ZERO);
        // switching by the count the other way round loses badly
        assert!(PUBLISHED.b_at_zero().mean() < PUBLISHED.game_a().mean());
        // no crowd smaller than 26 has whole flows
        for k in 1..26 {
            let (fw, bw) = model::flows(&ROUND.game_b(), k);
            assert!(fw.iter().chain(&bw).any(|x| !x.is_integer()), "{k}");
        }
    }
}
