//! Every number the video shows or says, computed from [`crate::model`] and
//! printed for [`oracle.py`](../oracle.py) (`./x shamir_secret_sharing
//! facts-check`), which recomputes each one by search. S-F rows are computed,
//! S-X rows are the primary source's own numbers. By the full draft's
//! chapters (docs/storyboard-draft.md): S-F0 and S-F1 serve 01 and 02, S-F2
//! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
//! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper, S-F7.join and
//! S-F7.all_pairs 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11
//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 23 are
//! implemented so far; the rest are computed and checked now so the draft's
//! numbers are settled before its scenes exist.

use std::collections::{BTreeMap, BTreeSet};

use explainer_kit::Facts;

use crate::model::{
    self, A_SHARE, ADJACENT, ASKED_SECRETS, Dial, FOUR_OF_SEVEN, FOUR_OF_SEVEN_SEEN, FOURTH, JOINER, LIAR, LINE_HOLDER,
    LINE_SLOPE, MARKS, OPENING_TRIO, ORDINARY_PIECES, OTHER_SPINS, PAIR, Padlocks, Ratio, Recipe, SECOND_TRIO, SECRET,
    SPINS, TEN, TEN_HOLDER, TEN_PARITY_HOLDER, THRESHOLD, TRUSTEES, View, combinations, csv,
};

/// Shamir's opening example (section 1, a problem he introduces as Liu's,
/// from a 1968 book): eleven scientists, a cabinet that opens when six or
/// more are present. The 462 locks and 252 keys are Shamir's own sentence.
pub const SHAMIR_SCIENTISTS: u32 = 11;
pub const SHAMIR_NEEDED: usize = 6;
pub const SHAMIR_YEAR: u32 = 1979;
const SHAMIR: &str = "Shamir, How to Share a Secret, Comm. ACM 22(11) 612-613 (November 1979), section 1: \
                      eleven scientists, six or more present, 462 locks and 252 keys per scientist";

/// The shares the worked example deals at `xs`, paired with their positions.
fn dealt(dial: Dial, recipe: &Recipe, xs: &[u32]) -> Vec<(u32, u32)> {
    xs.iter().map(|&x| (x, recipe.share(dial, x))).collect()
}

/// For each secret, the one spin pair that deals `points`; an error if a
/// secret fits any other number of ways (the video says "exactly one").
fn one_way_each(dial: Dial, points: &[(u32, u32)]) -> Result<Vec<Vec<u32>>, String> {
    dial.all()
        .map(|s| match model::fitting_spins(dial, points, s, 2).as_slice() {
            [spins] => Ok(spins.clone()),
            other => Err(format!(
                "{points:?}: secret {s} fits {} spin pairs, not one",
                other.len()
            )),
        })
        .collect()
}

fn distinct<T: Ord>(values: impl IntoIterator<Item = T>) -> usize {
    values.into_iter().collect::<BTreeSet<_>>().len()
}

fn set_csv(values: impl IntoIterator<Item = usize>) -> String {
    csv(&values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>())
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let dial = Dial::vault();
    let ten = Dial::new(TEN);
    let recipe = Recipe::example();
    let shares = recipe.deal(dial, &TRUSTEES);
    let share_at = |x: u32| recipe.share(dial, x);

    // 01_the_vault, 02_cutting_it_up: one keeper, copies, ordinary pieces
    f.add("S-F0.copies.who_can_open_alone", TRUSTEES.len())?;
    f.add(
        "S-F0.copies.secrets_left_for_one",
        View::Copy(SECRET).possible(dial).len(),
    )?;
    let keeper = [TRUSTEES[0]];
    f.add("S-F0.keepers", keeper.len())?;
    f.add("S-F0.keeper.losses_to_shut", model::losses_to_shut(&keeper))?;
    let one_bribe = model::bribes_to_open(&keeper).ok_or("the keeper holds no copy")?;
    f.add("S-F0.keeper.bribes_to_open", one_bribe)?;
    f.add("S-F0.copies.losses_to_shut", model::losses_to_shut(&TRUSTEES))?;
    let any_bribe = model::bribes_to_open(&TRUSTEES).ok_or("no trustee holds a copy")?;
    f.add("S-F0.copies.bribes_to_open", any_bribe)?;
    f.add("S-F0.pieces", csv(&ORDINARY_PIECES))?;
    f.add("S-F0.pieces.a", ORDINARY_PIECES[0])?;
    f.add("S-F0.pieces.b", ORDINARY_PIECES[1])?;
    let pieces_sum: u32 = ORDINARY_PIECES.iter().sum();
    if pieces_sum != SECRET {
        return Err(format!("the ordinary pieces add up to {pieces_sum}, not the secret"));
    }
    f.add("S-F0.pieces.sum", pieces_sum)?;
    let b_alone = View::OrdinaryPiece(ORDINARY_PIECES[1]).possible(dial);
    f.add("S-F0.pieces.b_alone_candidates", csv(&b_alone))?;
    f.add("S-F0.pieces.b_alone_left", b_alone.len())?;
    let ruled_out: Vec<u32> = dial.all().filter(|s| !b_alone.contains(s)).collect();
    f.add("S-F0.pieces.b_alone_ruled_out", csv(&ruled_out))?;
    f.add("S-F0.pieces.both_needed", ORDINARY_PIECES.len())?;

    // 01_the_vault: the trustees, the threshold, the dial
    f.add("S-F1.trustees", TRUSTEES.len())?;
    f.add("S-F1.threshold", THRESHOLD)?;
    f.add("S-F1.too_few", THRESHOLD - 1)?;
    f.add("S-F1.can_be_away", model::can_be_away(TRUSTEES.len(), THRESHOLD))?;
    f.add("S-F1.marks", dial.marks())?;
    f.add("S-F1.opening_marks", dial.all().filter(|&m| m == SECRET).count())?;
    f.add("S-F1.secret", SECRET)?;
    f.add("S-F1.opening_trio", csv(&OPENING_TRIO))?;
    f.add("S-F1.opening_shares", csv(&recipe.deal(dial, &OPENING_TRIO)))?;
    f.add("S-F1.pair", csv(&PAIR))?;
    f.add("S-F1.pair_shares", csv(&recipe.deal(dial, &PAIR)))?;

    // 03_the_dial: two of two on the dial
    let b_share = dial.sub(SECRET, A_SHARE);
    f.add("S-F2.top", dial.marks() - 1)?;
    f.add("S-F2.a_share", A_SHARE)?;
    f.add("S-F2.b_share", b_share)?;
    f.add("S-F2.sum_unwrapped", A_SHARE + b_share)?;
    f.add("S-F2.sum", dial.add(A_SHARE, b_share))?;
    f.add("S-F2.turn_back", dial.sub(SECRET, b_share))?;
    f.add("S-F2.ten_plus_one", dial.add(dial.marks() - 1, 1))?;
    let by_a: Vec<u32> = dial.all().map(|a| dial.add(a, b_share)).collect();
    f.add("S-F2.secret_if_a_held", csv(&by_a))?;
    f.add("S-F2.if_a_held.0", dial.add(0, b_share))?;
    f.add("S-F2.if_a_held.1", dial.add(1, b_share))?;
    f.add("S-F2.distinct_secrets", distinct(by_a.iter().copied()))?;
    let most = dial
        .all()
        .map(|s| by_a.iter().filter(|&&v| v == s).count())
        .max()
        .unwrap_or(0);
    f.add("S-F2.ways_each", most)?;
    f.add("S-F2.chance_each", Ratio::new(1, i64::from(MARKS)))?;

    // 06_padlocks: one lock per pair, its keys with the other three
    let locks = Padlocks::new(&TRUSTEES, THRESHOLD);
    let trios = combinations(&TRUSTEES, THRESHOLD);
    let pairs = combinations(&TRUSTEES, THRESHOLD - 1);
    f.add("S-F3.locks", locks.locks().len())?;
    f.add("S-F3.keys_each", set_csv(TRUSTEES.iter().map(|&t| locks.keys(t))))?;
    f.add("S-F3.trios", trios.len())?;
    f.add("S-F3.trios_that_open", trios.iter().filter(|g| locks.opens(g)).count())?;
    f.add("S-F3.pairs", pairs.len())?;
    f.add("S-F3.pairs_stuck", pairs.iter().filter(|g| !locks.opens(g)).count())?;
    f.add(
        "S-F3.locks_a_pair_cannot_open",
        set_csv(pairs.iter().map(|g| locks.stuck_at(g))),
    )?;
    f.add(
        "S-F3.keys_total",
        TRUSTEES.iter().map(|&t| locks.keys(t)).sum::<usize>(),
    )?;
    let scientists: Vec<u32> = (1..=SHAMIR_SCIENTISTS).collect();
    let liu = Padlocks::new(&scientists, SHAMIR_NEEDED);
    f.add("S-F3.shamir.scientists", SHAMIR_SCIENTISTS)?;
    f.add("S-F3.shamir.needed", SHAMIR_NEEDED)?;
    f.pin("S-F3.shamir.locks", liu.locks().len(), "462", SHAMIR)?;
    f.pin("S-F3.shamir.keys_each", liu.keys(1), "252", SHAMIR)?;
    f.add("S-F3.shamir.too_few", SHAMIR_NEEDED - 1)?;
    // one lock for each pair; the lock for 4·5 has its keys with the three
    // outside it
    f.add(
        "S-F3.locks_per_pair",
        set_csv(pairs.iter().map(|g| locks.locks().iter().filter(|l| *l == g).count())),
    )?;
    let pair_lock = locks
        .locks()
        .iter()
        .find(|g| g.as_slice() == PAIR)
        .ok_or_else(|| "no lock for the pair".to_owned())?;
    let pair_lock_holders: Vec<u32> = TRUSTEES.iter().copied().filter(|t| !pair_lock.contains(t)).collect();
    f.add("S-F3.pair_lock.holders", csv(&pair_lock_holders))?;
    // one lock serving two different pairs has its keys only outside both,
    // so the three or four trustees in those pairs together hold none
    let two = 2;
    let ids: Vec<u32> = (0..pairs.len() as u32).collect();
    let unions: BTreeSet<usize> = combinations(&ids, two)
        .iter()
        .map(|pick| {
            TRUSTEES
                .iter()
                .filter(|t| pick.iter().any(|&i| pairs[i as usize].contains(t)))
                .count()
        })
        .collect();
    f.add("S-F3.merged.pairs", two)?;
    f.add("S-F3.merged.unions", csv(&unions.into_iter().collect::<Vec<_>>()))?;
    f.add(
        "S-F3.fewest_locks",
        model::fewest_locks(&TRUSTEES, THRESHOLD).ok_or_else(|| "no plan of locks works".to_owned())?,
    )?;
    // the dealer's method instead: one number each
    let handed = dealt(dial, &Recipe::example(), &TRUSTEES);
    f.add(
        "S-F3.dealer.numbers_each",
        set_csv(
            TRUSTEES
                .iter()
                .map(|&t| handed.iter().filter(|&&(x, _)| x == t).count()),
        ),
    )?;

    // 07, 08: a line through the secret (any two of five)
    let line = Recipe::new(SECRET, &[LINE_SLOPE]);
    f.add("S-F4.slope", LINE_SLOPE)?;
    let line_heights: Vec<i64> = TRUSTEES.iter().map(|&x| line.height(x)).collect();
    f.add("S-F4.heights", csv(&line_heights))?;
    f.add("S-F4.shares", csv(&line.deal(dial, &TRUSTEES)))?;
    f.add("S-F4.times2", csv(&dial.times_row(2)))?;
    let tall = line.height(TEN_HOLDER);
    f.add("S-F4.paper.holder", TEN_HOLDER)?;
    f.add("S-F4.paper.share", tall)?;
    let paper = model::paper_line_secrets(MARKS, TEN_HOLDER, tall);
    f.add("S-F4.paper.candidates", csv(&paper))?;
    f.add("S-F4.paper.count", paper.len())?;
    // 07: two points fix a line, one doesn't; the secret is its height at
    // position 0, and each trustee sits at the position of its number
    f.add("S-F4.needed", line.coefficients().len())?;
    f.add("S-F4.too_few", line.coefficients().len() - 1)?;
    f.add("S-F4.spin_count", line.spins.len())?;
    let secret_at = (0..=TEN_HOLDER)
        .find(|&x| line.height(x) == i64::from(SECRET))
        .ok_or_else(|| "the line never has the secret's height".to_owned())?;
    f.add("S-F4.secret_position", secret_at)?;
    for &t in &TRUSTEES[..2] {
        f.add(&format!("S-F4.position_of.{t}"), t)?;
    }
    // neighbours: one position apart, so the climb between them is the spin
    let [(x0, u), (_, w)] = [ADJACENT[0], ADJACENT[1]].map(|x| (x, line.share(dial, x)));
    let step = dial.sub(w, u);
    let back = i64::from(u) - i64::from(x0) * i64::from(step);
    f.add("S-F4.adjacent", csv(&ADJACENT))?;
    f.add("S-F4.adjacent.shares", csv(&[u, w]))?;
    f.add("S-F4.adjacent.step", step)?;
    f.add("S-F4.adjacent.secret_unwrapped", back)?;
    f.add("S-F4.adjacent.secret", dial.wrap(back))?;
    // 08: each height wrapped: the height, its whole turns, its mark
    let m = i64::from(MARKS);
    for (&t, &h) in TRUSTEES.iter().zip(&line_heights) {
        f.add(
            &format!("S-F4.wrap.{t}"),
            csv(&[h, h.div_euclid(m), i64::from(dial.wrap(h))]),
        )?;
    }
    f.add("S-F4.adjacent.apart", ADJACENT[1] - ADJACENT[0])?;
    // 08: back from trustee 2 one step a position, by the neighbours' step,
    // to the secret at position 0
    let one_back = dial.sub(u, step);
    let two_back = dial.sub(one_back, step);
    if two_back != SECRET {
        return Err(format!("back from {u} by {step} twice is {two_back}"));
    }
    f.add("S-F4.back.steps", x0)?;
    f.add("S-F4.back.from", x0)?;
    f.add("S-F4.back.1", csv(&[u, step, one_back]))?;
    f.add("S-F4.back.2", csv(&[one_back, step, two_back]))?;
    // 08: the doubles of every mark hit every mark once
    let doubles = dial.times_row(2);
    f.add("S-F4.times2.of", csv(&dial.all().collect::<Vec<_>>()))?;
    f.add(
        "S-F4.times2.hits_each",
        set_csv(dial.all().map(|k| doubles.iter().filter(|&&d| d == k).count())),
    )?;

    // 08: one share on the dial
    let held = line.share(dial, LINE_HOLDER);
    let by_slope: Vec<u32> = dial.all().map(|a| dial.sub(held, dial.mul(a, LINE_HOLDER))).collect();
    f.add("S-F5.holder", LINE_HOLDER)?;
    f.add("S-F5.share", held)?;
    f.add("S-F5.secret_by_slope", csv(&by_slope))?;
    f.add("S-F5.distinct", distinct(by_slope.iter().copied()))?;
    f.add("S-F5.true_slope_secret", by_slope[LINE_SLOPE as usize])?;
    // 08: the secrets each spin allows (one), spin 0's and spin 1's
    f.add(
        "S-F5.per_spin",
        set_csv(dial.all().map(|a| {
            dial.all()
                .filter(|&s| dial.add(s, dial.mul(a, LINE_HOLDER)) == held)
                .count()
        })),
    )?;
    f.add("S-F5.spin.0", by_slope[0])?;
    f.add("S-F5.spin.1", by_slope[1])?;
    f.add("S-F5.spin.1.working", csv(&[held, LINE_HOLDER, by_slope[1]]))?;

    // 09: a curve for three
    f.add("S-F6.spins", csv(&SPINS))?;
    let heights: Vec<i64> = TRUSTEES.iter().map(|&x| recipe.height(x)).collect();
    f.add("S-F6.heights", csv(&heights))?;
    f.add("S-F6.shares", csv(&shares))?;
    f.add("S-F6.spin_pairs", model::tuples(dial, SPINS.len()).count())?;
    f.add("S-F6.spin_count", SPINS.len())?;
    // 09: trustees 1 and 2's heights term by term (position, the secret,
    // the first spin's term, the second's, the height), then the others'
    for &x in &TRUSTEES[..2] {
        let xi = i64::from(x);
        let terms = [
            i64::from(SECRET),
            i64::from(SPINS[0]) * xi,
            i64::from(SPINS[1]) * xi * xi,
        ];
        let total: i64 = terms.iter().sum();
        if total != recipe.height(x) {
            return Err(format!("trustee {x}'s terms add to {total}"));
        }
        f.add(
            &format!("S-F6.terms.{x}"),
            csv(&[xi, terms[0], terms[1], terms[2], total]),
        )?;
    }
    f.add("S-F6.rest", csv(&TRUSTEES[2..]))?;
    f.add("S-F6.rest.heights", csv(&heights[2..]))?;
    // 09: on paper, every three of the five give back one curve, through
    // all five points
    let paper_points: Vec<(u32, i64)> = TRUSTEES.iter().copied().zip(heights.iter().copied()).collect();
    let mut curves: Vec<(Ratio, Ratio, Ratio)> = Vec::new();
    for trio in combinations(&TRUSTEES, THRESHOLD) {
        let points: Vec<(u32, i64)> = paper_points.iter().copied().filter(|p| trio.contains(&p.0)).collect();
        let s = model::paper_sum(&points);
        let secret = s
            .whole()
            .and_then(|w| u32::try_from(w).ok())
            .ok_or_else(|| format!("trustees {trio:?} give the paper secret {s:?}"))?;
        let (a, b) = model::paper_curve_spins([points[0], points[1]], secret)
            .ok_or_else(|| format!("trustees {trio:?} give no paper curve"))?;
        let misses = paper_points.iter().any(|&(x, h)| {
            let x = i64::from(x);
            Ratio::int(i64::from(secret)).plus(a.times(x)).plus(b.times(x * x)) != Ratio::int(h)
        });
        if misses {
            return Err(format!("trustees {trio:?}'s paper curve misses a point"));
        }
        if !curves.contains(&(s, a, b)) {
            curves.push((s, a, b));
        }
    }
    f.add("S-F6.paper.curves", curves.len())?;
    // 09: trustee 2's 47 asked about, wrapped: four whole turns and 3; every
    // height's whole turns
    let asked = TRUSTEES[1];
    let h = recipe.height(asked);
    f.add("S-F6.asked", csv(&[i64::from(asked), h]))?;
    f.add(
        &format!("S-F6.wrap.{asked}"),
        csv(&[h, h.div_euclid(m), i64::from(dial.wrap(h))]),
    )?;
    let turns: Vec<i64> = heights.iter().map(|h| h.div_euclid(m)).collect();
    f.add("S-F6.turns", csv(&turns))?;

    // 05, 10: two shares, one spin pair per secret
    let pair_points = dealt(dial, &recipe, &PAIR);
    let fitting: Vec<usize> = dial
        .all()
        .map(|s| model::fitting_spins(dial, &pair_points, s, 2).len())
        .collect();
    f.add("S-F7.fitting", fitting.iter().sum::<usize>())?;
    f.add("S-F7.ways_each", set_csv(fitting.iter().copied()))?;
    let per_secret = one_way_each(dial, &pair_points)?;
    let spins_by_secret: Vec<String> = per_secret.iter().map(|t| format!("{}:{}", t[0], t[1])).collect();
    f.add("S-F7.spins_by_secret", spins_by_secret.join(","))?;
    f.add("S-F7.chance_each", Ratio::new(1, i64::from(MARKS * MARKS)))?;
    let true_spins = &per_secret[SECRET as usize];
    f.add("S-F7.true_spins", format!("{}:{}", true_spins[0], true_spins[1]))?;
    let straight = (0..MARKS)
        .find(|&s| per_secret[s as usize][1] == 0)
        .ok_or("no secret's curve through the pair is a line")?;
    f.add("S-F7.straight_one", straight)?;
    // 04 and 05: every secret the experiment feeds in, and the marks the
    // pair's two shares rule out
    f.add("S-F7.secrets", csv(&dial.all().collect::<Vec<_>>()))?;
    let pair_left = View::Shares {
        points: pair_points.clone(),
        spins: SPINS.len(),
    }
    .possible(dial);
    f.add("S-F7.ruled_out", dial.marks() as usize - pair_left.len())?;
    // 10: the pair's two points; the secrets asked about after the
    // dealer's, their spins, and the first one's heights at the pair worked
    // out and wrapped; the board, each secret's curve at positions 0 to 5,
    // and how many different marks each position holds
    f.add("S-F7.pair.count", pair_points.len())?;
    let [asked, next] = ASKED_SECRETS;
    f.add("S-F7.asked.first", asked)?;
    f.add("S-F7.asked.second", next)?;
    f.add("S-F7.spins.0", csv(&per_secret[asked as usize]))?;
    f.add("S-F7.spins.1", csv(&per_secret[next as usize]))?;
    let asked_recipe = Recipe::new(asked, &per_secret[asked as usize]);
    for &(x, y) in &pair_points {
        let h = asked_recipe.height(x);
        if dial.wrap(h) != y {
            return Err(format!(
                "the secret {asked}'s curve reaches {h} at {x}, which is not {y}"
            ));
        }
        f.add(&format!("S-F7.check.{x}"), csv(&[i64::from(x), h, i64::from(y)]))?;
    }
    let last = TRUSTEES.len() as u32;
    let board: Vec<Vec<u32>> = dial
        .all()
        .map(|s| {
            let r = Recipe::new(s, &per_secret[s as usize]);
            (0..=last).map(|x| r.share(dial, x)).collect()
        })
        .collect();
    for (s, along) in board.iter().enumerate() {
        f.add(&format!("S-F7.board.{s}"), csv(along))?;
    }
    let columns: Vec<usize> = (0..=last as usize)
        .map(|x| distinct(board.iter().map(|along| along[x])))
        .collect();
    f.add("S-F7.board.columns", csv(&columns))?;
    // 11: the same pair on ordinary paper
    let paper = [PAIR[0], PAIR[1]].map(|x| (x, recipe.height(x)));
    f.add("S-F7.paper.heights", csv(&paper.map(|p| p.1)))?;
    let curves = dial
        .all()
        .filter(|&s| model::paper_curve_spins(paper, s).is_some())
        .count();
    f.add("S-F7.paper.curves", curves)?;
    f.add(
        "S-F7.paper.whole_spin_secrets",
        csv(&model::paper_whole_spin_secrets(MARKS, paper)),
    )?;
    let (a0, b0) = model::paper_curve_spins(paper, 0).ok_or("no paper curve for secret 0")?;
    f.add("S-F7.paper.secret0_spins", csv(&[a0, b0]))?;
    let (a5, b5) = model::paper_curve_spins(paper, SECRET).ok_or("no paper curve for the secret")?;
    f.add("S-F7.paper.secret5_spins", csv(&[a5, b5]))?;
    // 11: a third share joins
    let at_joiner: Vec<u32> = dial
        .all()
        .map(|s| Recipe::new(s, &per_secret[s as usize]).share(dial, JOINER))
        .collect();
    f.add("S-F7.join.trustee", JOINER)?;
    f.add("S-F7.join.share", share_at(JOINER))?;
    f.add("S-F7.join.values_by_secret", csv(&at_joiner))?;
    f.add("S-F7.join.distinct", distinct(at_joiner.iter().copied()))?;
    let left: Vec<u32> = dial
        .all()
        .filter(|&s| at_joiner[s as usize] == share_at(JOINER))
        .collect();
    f.add("S-F7.join.secrets_left", csv(&left))?;
    f.add("S-F7.join.point", csv(&[JOINER, share_at(JOINER)]))?;
    f.add("S-F7.join.count", left.len())?;
    f.add("S-F7.join.drop", csv(&[pair_left.len(), left.len()]))?;
    // 11: every pair of trustees, every two shares they could hold and
    // every third trustee's every share: how many spin pairs each secret
    // fits, how many secrets two shares leave, and how many a third leaves
    let pairs = combinations(&TRUSTEES, 2);
    let (mut ways, mut two_leave, mut third_leaves) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for pair in &pairs {
        for values in model::tuples(dial, 2) {
            let points: Vec<(u32, u32)> = pair.iter().copied().zip(values).collect();
            let fits: Vec<Vec<Vec<u32>>> = dial.all().map(|s| model::fitting_spins(dial, &points, s, 2)).collect();
            ways.extend(fits.iter().map(Vec::len));
            two_leave.insert(fits.iter().filter(|w| !w.is_empty()).count());
            for &j in TRUSTEES.iter().filter(|j| !pair.contains(j)) {
                for y in dial.all() {
                    let count = (0..)
                        .zip(&fits)
                        .filter(|(s, w)| w.iter().any(|t| Recipe::new(*s, t).share(dial, j) == y))
                        .count();
                    third_leaves.insert(count);
                }
            }
        }
    }
    f.add("S-F7.all_pairs.pairs", pairs.len())?;
    f.add("S-F7.all_pairs.share_pairs", model::tuples(dial, 2).count())?;
    f.add("S-F7.all_pairs.ways_each", set_csv(ways))?;
    f.add(
        "S-F7.all_pairs.two_leave",
        format!("{},{}", PAIR.len(), set_csv(two_leave)),
    )?;
    f.add(
        "S-F7.all_pairs.third_leaves",
        format!("{THRESHOLD},{}", set_csv(third_leaves)),
    )?;

    // 12, 13: the same recipe, and the line, on ten marks
    let ten_shares = recipe.deal(ten, &TRUSTEES);
    let ten_pair = dealt(ten, &recipe, &PAIR);
    let ten_pair_values = [ten_pair[0].1, ten_pair[1].1];
    f.add("S-F8.ten.marks", TEN)?;
    f.add("S-F8.ten.shares", csv(&ten_shares))?;
    f.add("S-F8.ten.pair_shares", csv(&ten_pair_values))?;
    let ten_pair_view = View::Shares {
        points: ten_pair.clone(),
        spins: 2,
    };
    f.add("S-F8.ten.pair_secrets", csv(&ten_pair_view.possible(ten)))?;
    let (su, sw) = model::pair_shortcut(ten, PAIR).ok_or("no shortcut for the pair on ten marks")?;
    f.add("S-F8.ten.pair_formula", csv(&[su, sw]))?;
    let applied = |y: [u32; 2]| su * i64::from(y[0]) + sw * i64::from(y[1]);
    f.add("S-F8.ten.pair_formula_value", applied(ten_pair_values))?;
    // in ordinary numbers u·y4 + w·y5 is these multiples of b, a and s
    let [p4, p5] = PAIR.map(i64::from);
    let terms = [su * p4 * p4 + sw * p5 * p5, su * p4 + sw * p5, su + sw];
    f.add("S-F8.pair_sum.spin_terms", csv(&terms))?;
    f.add("S-F8.ten.leftover", ten.wrap(terms[0]))?;
    f.add("S-F8.eleven.leftover", dial.wrap(terms[0]))?;
    let pair_values = [pair_points[0].1, pair_points[1].1];
    f.add("S-F8.eleven.pair_formula_unwrapped", applied(pair_values))?;
    f.add("S-F8.eleven.pair_formula_value", dial.wrap(applied(pair_values)))?;
    let leaky_pairs = combinations(&TRUSTEES, 2)
        .iter()
        .filter(|p| !model::learns_nothing(ten, p, 2, model::any_spins))
        .count();
    f.add("S-F8.ten.pairs_that_leak", leaky_pairs)?;
    let (z1, z2) = model::zero_product(ten).ok_or("ten marks: no two nonzero marks multiply to zero")?;
    f.add("S-F8.ten.zero_product", csv(&[z1, z2]))?;
    f.add("S-F8.ten.times5", csv(&ten.times_row(5)))?;
    f.add("S-F8.eleven.times5", csv(&dial.times_row(5)))?;
    let ten_shufflers: Vec<u32> = (1..TEN).filter(|&k| ten.shuffles(k)).collect();
    f.add("S-F8.ten.rows_that_shuffle", csv(&ten_shufflers))?;
    f.add(
        "S-F8.eleven.rows_that_shuffle",
        (1..MARKS).filter(|&k| dial.shuffles(k)).count(),
    )?;
    let smallest_prime = (TRUSTEES.len() as u32 + 1..)
        .find(|&q| Dial::new(q).is_prime())
        .ok_or("no prime above five")?;
    f.add("S-F8.smallest_prime_for_five", smallest_prime)?;
    let ten_line = line.deal(ten, &TRUSTEES);
    let candidates = |x: u32| {
        View::Shares {
            points: vec![(x, line.share(ten, x))],
            spins: 1,
        }
        .possible(ten)
    };
    f.add("S-F8.line.shares", csv(&ten_line))?;
    f.add("S-F8.line.holder", TEN_HOLDER)?;
    f.add("S-F8.line.holder_share", line.share(ten, TEN_HOLDER))?;
    f.add("S-F8.line.candidates", csv(&candidates(TEN_HOLDER)))?;
    f.add("S-F8.line.parity_holder", TEN_PARITY_HOLDER)?;
    f.add("S-F8.line.parity_share", line.share(ten, TEN_PARITY_HOLDER))?;
    f.add("S-F8.line.parity_candidates", csv(&candidates(TEN_PARITY_HOLDER)))?;
    let leaky_marks: Vec<u32> = (1..TEN).filter(|&x| !ten.shuffles(x)).collect();
    f.add("S-F8.line.leaky_marks", csv(&leaky_marks))?;
    // 12 (native): the pair's tally on each dial
    let tally = |d: Dial, points: &[(u32, u32)]| -> Vec<usize> {
        d.all().map(|s| model::fitting_spins(d, points, s, 2).len()).collect()
    };
    let ten_ways = tally(ten, &ten_pair);
    f.add("S-F8.ten.pair_ways", csv(&ten_ways))?;
    f.add("S-F8.eleven.pair_ways", csv(&tally(dial, &pair_points)))?;
    f.add("S-F8.ten.pair_spin_pairs", ten_ways.iter().sum::<usize>())?;
    // 12_4, 12_5: each height's amounts of the secret and the two spins, and
    // the shortcut's weights as copies, which hold the same amount of the first
    let amounts = PAIR.map(|x| [1, i64::from(x), i64::from(x * x)]);
    let copies = [su, -sw];
    for (x, a) in PAIR.iter().zip(&amounts) {
        f.add(&format!("S-F8.pair_sum.height.{x}"), csv(a))?;
    }
    for (&x, c) in PAIR.iter().zip(copies) {
        f.add(&format!("S-F8.pair_sum.copies.{x}"), csv(&[c, i64::from(x)]))?;
    }
    let scaled = [0, 1].map(|i| amounts[i].map(|t| copies[i] * t));
    f.add("S-F8.pair_sum.first_spin", scaled[0][1])?;
    for (x, s) in PAIR.iter().zip(&scaled) {
        f.add(&format!("S-F8.pair_sum.scaled.{x}"), csv(s))?;
    }
    f.add("S-F8.pair_sum.first_spin_left", scaled[0][1] - scaled[1][1])?;
    // 12_6, 12_7: the −20 on each dial, and the pair's sum term by term
    let twenty = terms[0];
    f.add("S-F8.ten.twenty_lands", csv(&[-twenty, i64::from(ten.wrap(-twenty))]))?;
    let sum_terms = |y: [u32; 2]| [su, i64::from(y[0]), sw, i64::from(y[1])];
    let products = |y: [u32; 2]| [su * i64::from(y[0]), sw * i64::from(y[1])];
    f.add("S-F8.ten.pair_formula_terms", csv(&sum_terms(ten_pair_values)))?;
    f.add("S-F8.ten.pair_formula_products", csv(&products(ten_pair_values)))?;
    f.add(
        "S-F8.eleven.minus_twenty_lands",
        csv(&[twenty, i64::from(dial.wrap(twenty))]),
    )?;
    f.add("S-F8.eleven.pair_formula_terms", csv(&sum_terms(pair_values)))?;
    f.add("S-F8.eleven.pair_formula_products", csv(&products(pair_values)))?;
    let eight = dial.wrap(applied(pair_values));
    let second_spins = dial.all().map(|s| {
        dial.all()
            .filter(|&b| dial.add(s, dial.mul(dial.wrap(twenty), b)) == eight)
            .count()
    });
    f.add("S-F8.eleven.second_spin_each", set_csv(second_spins))?;
    // 12_8: what each pair's two shares leave on ten marks, by dealing every recipe
    let ten_pairs = combinations(&TRUSTEES, 2);
    let mut by_left: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut even_left = 0;
    for pair in &ten_pairs {
        let leaves = model::what_shares_leave(ten, pair, 2);
        let counts: Vec<usize> = leaves
            .iter()
            .map(|&(n, _)| n)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let [n] = counts[..] else {
            return Err(format!("ten marks: {pair:?}'s shares leave {counts:?} secrets"));
        };
        by_left.entry(n).or_default().push(format!("{}-{}", pair[0], pair[1]));
        even_left += usize::from(leaves.iter().all(|&(_, even)| even));
    }
    for (n, names) in &by_left {
        f.add(&format!("S-F8.ten.pairs_left.{n}"), names.join(","))?;
    }
    f.add("S-F8.ten.pairs_that_find", by_left.get(&1).map_or(0, Vec::len))?;
    let silent: Vec<&Vec<u32>> = ten_pairs
        .iter()
        .filter(|p| model::learns_nothing(ten, p, 2, model::any_spins))
        .collect();
    let [quiet] = silent[..] else {
        return Err(format!("ten marks: {silent:?} learn nothing, not one pair"));
    };
    f.add("S-F8.ten.pair_that_learns_nothing", csv(quiet))?;
    f.add("S-F8.ten.pairs_even_left", even_left)?;

    // 13 (native): multiplying by five on both dials
    f.add("S-F8.ten.zero_product_sum", csv(&[z1, z2, z1 * z2]))?;
    f.add("S-F8.ten.zero_product_mark", ten.mul(z1, z2))?;
    f.add("S-F8.multiplier", TEN_HOLDER)?;
    let hits = |d: Dial| -> Vec<usize> {
        let row = d.times_row(TEN_HOLDER);
        d.all().map(|m| row.iter().filter(|&&y| y == m).count()).collect()
    };
    let (ten_hits, eleven_hits) = (hits(ten), hits(dial));
    let reached: Vec<u32> = ten.all().filter(|&m| ten_hits[m as usize] > 0).collect();
    f.add("S-F8.ten.times5_values", csv(&reached))?;
    f.add("S-F8.ten.times5_reached", reached.len())?;
    f.add("S-F8.ten.times5_hits", csv(&ten_hits))?;
    f.add("S-F8.line.candidate_count", candidates(TEN_HOLDER).len())?;
    f.add(
        "S-F8.eleven.times5_reached",
        eleven_hits.iter().filter(|&&h| h > 0).count(),
    )?;
    f.add("S-F8.eleven.times5_hits", csv(&eleven_hits))?;
    f.add("S-F8.eleven.times5_each", set_csv(eleven_hits.iter().copied()))?;
    for x in 1..=3 {
        let k = TEN_HOLDER * x;
        let step = if k < MARKS {
            vec![x, k]
        } else {
            vec![x, k, dial.mul(TEN_HOLDER, x)]
        };
        f.add(&format!("S-F8.eleven.times5_step.{x}"), csv(&step))?;
    }
    let one_share = View::Shares {
        points: vec![(TEN_HOLDER, line.share(dial, TEN_HOLDER))],
        spins: 1,
    };
    f.add("S-F8.eleven.line_candidates", one_share.possible(dial).len())?;
    let splits: Vec<[u32; 3]> = (2..TEN)
        .flat_map(|a| (a..TEN).map(move |b| [TEN, a, b]))
        .filter(|&[n, a, b]| a * b == n)
        .collect();
    let [split] = splits[..] else {
        return Err(format!("ten splits as {splits:?}, not one way"));
    };
    f.add("S-F8.ten.split", csv(&split))?;
    let factors: Vec<u32> = (1..=MARKS).filter(|&d| MARKS.is_multiple_of(d)).collect();
    f.add("S-F8.eleven.factors", csv(&factors))?;
    f.add("S-F8.eleven.zero_product_check", csv(&[z1, z2, dial.mul(z1, z2)]))?;
    if let Some(pair) = model::zero_product(dial) {
        return Err(format!("eleven marks: {pair:?} multiply to zero"));
    }
    f.add("S-F8.eleven.nonzero_range", csv(&[1, MARKS - 1]))?;
    let reach = |d: Dial| -> Vec<usize> { (1..d.marks()).map(|k| distinct(d.times_row(k))).collect() };
    f.add("S-F8.ten.reach", csv(&reach(ten)))?;
    f.add("S-F8.eleven.reach", csv(&reach(dial)))?;
    let primes: Vec<u32> = (TRUSTEES.len() as u32 + 1..)
        .filter(|&q| Dial::new(q).is_prime())
        .take(3)
        .collect();
    f.add("S-F8.primes_for_five", csv(&primes))?;

    // 14, 15: helper curves and three shares (the opening trio)
    let trio_points = dealt(dial, &recipe, &OPENING_TRIO);
    let own: Vec<i64> = (0..OPENING_TRIO.len())
        .map(|i| model::helper_at_own(&OPENING_TRIO, i))
        .collect();
    let at0: Vec<i64> = (0..OPENING_TRIO.len())
        .map(|i| model::helper_at(&OPENING_TRIO, i, 0))
        .collect();
    f.add("S-F9.helper_at_own", csv(&own))?;
    f.add("S-F9.helper_at_zero", csv(&at0))?;
    f.add("S-F9.weights", csv(&model::ordinary_weights(&OPENING_TRIO)))?;
    let trio_weights = model::weights_at(dial, &OPENING_TRIO, 0).ok_or("the opening trio has no dial weights")?;
    f.add("S-F9.weights_on_dial", csv(&trio_weights))?;
    let tall = |xs: &[u32]| xs.iter().map(|&x| (x, recipe.height(x))).collect::<Vec<_>>();
    let tall_trio = tall(&OPENING_TRIO);
    f.add(
        "S-F9.paper.heights",
        csv(&tall_trio.iter().map(|p| p.1).collect::<Vec<_>>()),
    )?;
    f.add("S-F9.paper.terms", csv(&model::paper_terms(&tall_trio)))?;
    let paper_total = model::paper_sum(&tall_trio);
    f.add("S-F9.paper.sum", paper_total)?;
    let elevens: Vec<i64> = tall_trio
        .iter()
        .zip(&trio_points)
        .map(|(&(_, h), &(_, y))| (h - i64::from(y)) / i64::from(MARKS))
        .collect();
    f.add("S-F9.paper.elevens", csv(&elevens))?;
    let trio_total = model::ordinary_sum(&trio_points);
    f.add("S-F9.sum_unwrapped", trio_total)?;
    f.add("S-F9.sum_minus_secret", trio_total.minus(paper_total))?;
    f.add(
        "S-F9.secret",
        model::rebuild(dial, &trio_points).ok_or("the opening trio rebuilds nothing")?,
    )?;
    {
        // 14 (native): each helper at the trio's positions, the positions where it is
        // zero, its factors and its value at its own trustee; the scaled helpers summed
        // at each trustee's position
        let xs = OPENING_TRIO;
        let helper = |i: usize, at: i64| Ratio::new(model::helper_at(&xs, i, at), own[i]);
        let positions: Vec<i64> = xs.iter().map(|&x| i64::from(x)).collect();
        let heights: Vec<i64> = tall_trio.iter().map(|p| p.1).collect();
        for (i, &x) in xs.iter().enumerate() {
            let n = i + 1;
            f.add(&format!("S-F9.helper.{n}.position"), x)?;
            let row: Vec<Ratio> = positions.iter().map(|&p| helper(i, p)).collect();
            f.add(&format!("S-F9.helper.{n}.at_trio"), csv(&row))?;
            let zeros: Vec<u32> = xs
                .iter()
                .zip(&row)
                .filter(|&(_, v)| v.whole() == Some(0))
                .map(|(&y, _)| y)
                .collect();
            f.add(&format!("S-F9.helper.{n}.zeros"), csv(&zeros))?;
            let factors: Vec<i64> = xs
                .iter()
                .filter(|&&y| y != x)
                .map(|&y| i64::from(x) - i64::from(y))
                .collect();
            if factors.iter().product::<i64>() != own[i] {
                return Err(format!(
                    "helper {n}'s factors {factors:?} do not multiply to {}",
                    own[i]
                ));
            }
            f.add(&format!("S-F9.helper.{n}.factors_at_own"), csv(&factors))?;
            f.add(&format!("S-F9.helper.{n}.own"), own[i])?;
            f.add(&format!("S-F9.helper.{n}.scaled"), csv(&[heights[i], n as i64]))?;
        }
        for (i, &p) in positions.iter().enumerate() {
            let sum = heights
                .iter()
                .enumerate()
                .fold(Ratio::int(0), |acc, (j, &h)| acc.plus(helper(j, p).times(h)));
            if sum.whole() != Some(heights[i]) {
                return Err(format!(
                    "the scaled helpers give {sum} at position {p}, not {}",
                    heights[i]
                ));
            }
            f.add(&format!("S-F9.sum_at.{}", i + 1), sum)?;
        }
        let zero_at_first: Vec<usize> = (0..xs.len())
            .filter(|&i| helper(i, positions[0]).whole() == Some(0))
            .map(|i| i + 1)
            .collect();
        f.add("S-F9.sum_at.1.zero_helpers", csv(&zero_at_first))?;
        let (mut on, mut off) = (Vec::new(), Vec::new());
        for i in 0..xs.len() {
            for (j, &p) in positions.iter().enumerate() {
                if i == j {
                    on.push(helper(i, p));
                } else {
                    off.push(helper(i, p));
                }
            }
        }
        let alike = |values: &[Ratio], what: &str| -> Result<Ratio, String> {
            match values.first() {
                Some(&first) if values.iter().all(|&v| v == first) => Ok(first),
                _ => Err(format!("the helpers {what} are {}", csv(values))),
            }
        };
        f.add("S-F9.helper.on", alike(&on, "at their own trustees")?)?;
        f.add("S-F9.helper.off", alike(&off, "at the other trustees")?)?;
        f.add("S-F9.helper.others", xs.len() - 1)?;

        // 15 (native): each helper at position zero, worked as the scene works it; the
        // weighted heights and cards, the cards' sum wrapped onto the dial, and each height
        // split into its card and whole elevens
        let weights = model::ordinary_weights(&xs);
        for (i, &x) in xs.iter().enumerate() {
            let n = i + 1;
            let factors: Vec<i64> = xs.iter().filter(|&&y| y != x).map(|&y| -i64::from(y)).collect();
            let value: i64 = factors.iter().product();
            if value != at0[i] || Ratio::new(value, own[i]) != weights[i] {
                return Err(format!(
                    "helper {n} at zero is {value}, not weight {} times {}",
                    weights[i], own[i]
                ));
            }
            f.add(&format!("S-F9.helper.{n}.factors_at_zero"), csv(&factors))?;
            f.add(&format!("S-F9.helper.{n}.at_zero"), value)?;
            f.add(&format!("S-F9.helper.{n}.weight"), weights[i])?;
        }
        let whole: Vec<i64> = weights
            .iter()
            .map(|w| w.whole().ok_or("the opening trio's weights are not whole"))
            .collect::<Result<_, _>>()?;
        let cards: Vec<i64> = trio_points.iter().map(|p| i64::from(p.1)).collect();
        let interleave =
            |values: &[i64]| -> Vec<i64> { whole.iter().zip(values).flat_map(|(&w, &v)| [w, v]).collect() };
        f.add("S-F9.paper.weighted", csv(&interleave(&heights)))?;
        f.add("S-F9.cards.weighted", csv(&interleave(&cards)))?;
        let share_terms: Vec<i64> = whole.iter().zip(&cards).map(|(w, c)| w * c).collect();
        f.add("S-F9.share_terms", csv(&share_terms))?;
        let total: i64 = share_terms.iter().sum();
        if Ratio::int(total) != trio_total {
            return Err(format!("the cards' terms add to {total}, not {trio_total}"));
        }
        f.add("S-F9.sum_lands", csv(&[total, i64::from(dial.wrap(total))]))?;
        for (i, (&h, &c)) in heights.iter().zip(&cards).enumerate() {
            if h != c + m * elevens[i] {
                return Err(format!("height {h} is not card {c} and {} elevens", elevens[i]));
            }
            f.add(&format!("S-F9.split.{}", i + 1), csv(&[h, c, elevens[i]]))?;
        }
        let weighted_elevens: i64 = whole.iter().zip(&elevens).map(|(w, e)| w * e).sum();
        f.add("S-F9.weighted_elevens", weighted_elevens)?;
        if paper_total != Ratio::int(total + m * weighted_elevens) {
            return Err(format!(
                "paper sum {paper_total} is not {total} and {weighted_elevens} elevens"
            ));
        }
        if total.rem_euclid(m) != i64::from(dial.wrap(total)) {
            return Err(format!("{total} leaves {} over whole elevens", total.rem_euclid(m)));
        }
        f.add(
            "S-F9.sum_split",
            csv(&[total, total.rem_euclid(m), total.div_euclid(m)]),
        )?;
    }

    // 16 to 18: fractional weights, and dividing on the dial (trustees 2, 4 and 5)
    let second = dealt(dial, &recipe, &SECOND_TRIO);
    f.add("S-F10.trio", csv(&SECOND_TRIO))?;
    f.add(
        "S-F10.trio_shares",
        csv(&second.iter().map(|p| p.1).collect::<Vec<_>>()),
    )?;
    let own2: Vec<i64> = (0..SECOND_TRIO.len())
        .map(|i| model::helper_at_own(&SECOND_TRIO, i))
        .collect();
    let at02: Vec<i64> = (0..SECOND_TRIO.len())
        .map(|i| model::helper_at(&SECOND_TRIO, i, 0))
        .collect();
    f.add("S-F10.helper_at_own", csv(&own2))?;
    f.add("S-F10.helper_at_zero", csv(&at02))?;
    f.add("S-F10.weights", csv(&model::ordinary_weights(&SECOND_TRIO)))?;
    let tall_second = tall(&SECOND_TRIO);
    f.add(
        "S-F10.paper.heights",
        csv(&tall_second.iter().map(|p| p.1).collect::<Vec<_>>()),
    )?;
    f.add("S-F10.paper.sum", model::paper_sum(&tall_second))?;
    let as_tall = |points: &[(u32, u32)]| points.iter().map(|&(x, y)| (x, i64::from(y))).collect::<Vec<_>>();
    f.add("S-F10.fraction_terms", csv(&model::paper_terms(&as_tall(&second))))?;
    f.add("S-F10.fraction_sum", model::ordinary_sum(&second))?;
    let other = Recipe::new(SECRET, &OTHER_SPINS);
    let other_points = dealt(dial, &other, &SECOND_TRIO);
    f.add("S-F10.other.spins", csv(&OTHER_SPINS))?;
    f.add(
        "S-F10.other.heights",
        csv(&SECOND_TRIO.iter().map(|&x| other.height(x)).collect::<Vec<_>>()),
    )?;
    f.add(
        "S-F10.other.shares",
        csv(&other_points.iter().map(|p| p.1).collect::<Vec<_>>()),
    )?;
    f.add("S-F10.other.fraction_sum", model::ordinary_sum(&other_points))?;
    f.add(
        "S-F10.other.secret",
        model::rebuild(dial, &other_points).ok_or("the other dealing rebuilds nothing")?,
    )?;
    let fractional = model::tuples(dial, SPINS.len())
        .filter(|t| {
            model::ordinary_sum(&dealt(dial, &Recipe::new(SECRET, t), &SECOND_TRIO))
                .whole()
                .is_none()
        })
        .count();
    f.add("S-F10.fractional_spin_pairs", fractional)?;
    let (clear, cleared) = model::cleared_weights(&SECOND_TRIO);
    f.add("S-F10.clear", clear)?;
    f.add("S-F10.cleared_weights", csv(&cleared))?;
    f.add(
        "S-F10.other.cleared_terms",
        csv(&cleared
            .iter()
            .zip(&other_points)
            .map(|(w, p)| w * i64::from(p.1))
            .collect::<Vec<_>>()),
    )?;
    let cleared_sum =
        |points: &[(u32, u32)]| -> i64 { cleared.iter().zip(points).map(|(w, p)| w * i64::from(p.1)).sum() };
    let other_cleared = cleared_sum(&other_points);
    f.add("S-F10.other.cleared_sum", other_cleared)?;
    f.add("S-F10.other.cleared_lands", dial.wrap(other_cleared))?;
    let real_cleared = cleared_sum(&second);
    f.add("S-F10.cleared_sum", real_cleared)?;
    let lands = dial.wrap(real_cleared);
    f.add("S-F10.cleared_lands", lands)?;
    let clear = u32::try_from(clear).map_err(|_| "a negative clearing number")?;
    f.add("S-F10.times3", csv(&dial.times_row(clear)))?;
    let thrice = model::undo(dial, clear, lands).ok_or("3 has no partner on eleven marks")?;
    f.add("S-F10.thrice", thrice)?;
    f.add("S-F10.thrice_unwrapped", clear * thrice)?;
    f.add(
        "S-F10.third",
        dial.inverse(clear).ok_or("3 has no partner on eleven marks")?,
    )?;
    let second_weights = model::weights_at(dial, &SECOND_TRIO, 0).ok_or("the second trio has no dial weights")?;
    f.add("S-F10.weights_on_dial", csv(&second_weights))?;
    f.add(
        "S-F10.sum_unwrapped",
        model::dial_sum(dial, &second).ok_or("no dial sum")?,
    )?;
    f.add(
        "S-F10.secret",
        model::rebuild(dial, &second).ok_or("the second trio rebuilds nothing")?,
    )?;
    let inverses = (1..MARKS)
        .map(|d| dial.inverse(d).map(|p| format!("{d}:{p}")))
        .collect::<Option<Vec<_>>>()
        .ok_or("a nonzero mark of eleven has no partner")?;
    f.add("S-F10.inverses", inverses.join(","))?;
    let invertible: Vec<u32> = (1..TEN).filter(|&d| ten.inverse(d).is_some()).collect();
    f.add("S-F10.ten.invertible", csv(&invertible))?;
    let partner = dial.inverse(clear).ok_or("missing partner")?;
    f.add("S-F10.partner_product", clear * partner)?;
    f.add("S-F10.undo_product", lands * partner)?;
    let products = model::ordinary_weights(&SECOND_TRIO)
        .iter()
        .map(|w| match w.whole() {
            Some(n) => Ok(n),
            None => Ok(w.times(i64::from(clear)).whole().ok_or("uncleared fraction")? * i64::from(partner)),
        })
        .collect::<Result<Vec<_>, String>>()?;
    f.add("S-F10.weight_products", csv(&products))?;
    f.add(
        "S-F10.cleared_terms",
        csv(&cleared
            .iter()
            .zip(&second)
            .map(|(w, p)| w * i64::from(p.1))
            .collect::<Vec<_>>()),
    )?;
    let terms = second_weights
        .iter()
        .zip(&second)
        .map(|(&w, p)| w * p.1)
        .collect::<Vec<_>>();
    f.add("S-F10.dial_terms", csv(&terms))?;
    f.add("S-F10.whole_turns", terms.iter().sum::<u32>() / dial.marks())?;
    f.add(
        "S-F10.ten.noninvertible",
        csv(&(1..TEN).filter(|&x| ten.inverse(x).is_none()).collect::<Vec<_>>()),
    )?;

    // 19 and 20: any three, and the ten-mark trio that cannot
    f.add("S-F11.trios", trios.len())?;
    let giving = trios
        .iter()
        .filter(|g| model::rebuild(dial, &dealt(dial, &recipe, g)) == Some(SECRET))
        .count();
    f.add("S-F11.trios_giving_secret", giving)?;
    let [a, b, third] = SECOND_TRIO;
    let difference = Recipe::new(dial.mul(a, b), &[dial.wrap(-i64::from(a) - i64::from(b)), 1]);
    let trial = recipe.plus(dial, &difference);
    f.add("S-F11.trial", csv(&trial.coefficients()))?;
    f.add("S-F11.difference", csv(&difference.coefficients()))?;
    f.add("S-F11.trial_shares", csv(&trial.deal(dial, &SECOND_TRIO)))?;
    f.add(
        "S-F11.difference_row",
        csv(&dial.all().map(|x| difference.share(dial, x)).collect::<Vec<_>>()),
    )?;
    f.add(
        "S-F11.left_row",
        csv(&dial
            .all()
            .map(|x| dial.wrap(i64::from(x) - i64::from(a)))
            .collect::<Vec<_>>()),
    )?;
    f.add(
        "S-F11.right_row",
        csv(&dial
            .all()
            .map(|x| dial.wrap(i64::from(x) - i64::from(b)))
            .collect::<Vec<_>>()),
    )?;
    f.add(
        "S-F11.roots",
        csv(&dial
            .all()
            .filter(|&x| difference.share(dial, x) == 0)
            .collect::<Vec<_>>()),
    )?;
    f.add("S-F11.third_difference", difference.share(dial, third))?;
    f.add("S-F11.most_zeros", model::most_zeros(dial, 2))?;
    f.add("S-F11.line_most_zeros", model::most_zeros(dial, 1))?;
    f.add("S-F11.ten.line_most_zeros", model::most_zeros(ten, 1))?;
    let ten_view = |xs: &[u32]| View::Shares {
        points: dealt(ten, &recipe, xs),
        spins: 2,
    };
    f.add(
        "S-F11.ten.opening_trio_secrets",
        csv(&ten_view(&OPENING_TRIO).possible(ten)),
    )?;
    let (pin, loose): (Vec<&Vec<u32>>, Vec<&Vec<u32>>) = trios
        .iter()
        .partition(|g| ten_view(g.as_slice()).possible(ten).len() == 1);
    f.add("S-F11.ten.trios_that_pin", pin.len())?;
    f.add(
        "S-F11.ten.trio_table",
        trios
            .iter()
            .map(|xs| format!("{}:{}", csv(xs), csv(&ten_view(xs).possible(ten))))
            .collect::<Vec<_>>()
            .join(";"),
    )?;
    let [ten_trio] = loose.as_slice() else {
        return Err(format!(
            "ten marks: {} trios leave more than one secret, not one",
            loose.len()
        ));
    };
    let ten_trio: &[u32] = ten_trio.as_slice();
    f.add("S-F11.ten.trio", csv(ten_trio))?;
    f.add("S-F11.ten.trio_shares", csv(&recipe.deal(ten, ten_trio)))?;
    f.add("S-F11.ten.trio_secrets", csv(&ten_view(ten_trio).possible(ten)))?;
    let ghost = model::ghost(ten, ten_trio).ok_or("ten marks: no recipe vanishes at the trio")?;
    f.add("S-F11.ten.ghost", csv(&ghost.coefficients()))?;
    f.add(
        "S-F11.ten.ghost.heights",
        csv(&ten_trio.iter().map(|&x| ghost.height(x)).collect::<Vec<_>>()),
    )?;
    let ghost_zeros: Vec<u32> = ten.all().filter(|&x| ghost.share(ten, x) == 0).collect();
    f.add("S-F11.ten.ghost.zeros", csv(&ghost_zeros))?;
    f.add("S-F11.ten.moved_recipe", csv(&recipe.plus(ten, &ghost).coefficients()))?;

    // 04 (the guess), 21: what it promises, 22: a liar
    f.add("S-F12.guess_chance", Ratio::new(1, i64::from(MARKS)))?;
    f.add("S-F12.guess_misses", dial.all().filter(|&s| s != SECRET).count())?;
    let excluded: Vec<u32> = dial.all().filter(|&s| per_secret[s as usize][1] == 0).collect();
    f.add("S-F12.nonzero_last.excluded", csv(&excluded))?;
    f.add("S-F12.nonzero_last.left", dial.marks() as usize - excluded.len())?;
    let straight_recipe = Recipe::new(straight, &per_secret[straight as usize]);
    f.add("S-F12.straight_recipe", csv(&straight_recipe.coefficients()))?;
    f.add(
        "S-F12.straight_heights",
        csv(&PAIR.iter().map(|&x| straight_recipe.height(x)).collect::<Vec<_>>()),
    )?;
    let liar_points: Vec<(u32, u32)> = trio_points
        .iter()
        .map(|&(x, y)| if x == LIAR.0 { (x, LIAR.1) } else { (x, y) })
        .collect();
    f.add("S-F12.liar", LIAR.0)?;
    f.add("S-F12.liar_true", share_at(LIAR.0))?;
    f.add("S-F12.liar_hands_in", LIAR.1)?;
    f.add("S-F12.liar_result_unwrapped", model::ordinary_sum(&liar_points))?;
    f.add(
        "S-F12.liar_result",
        model::rebuild(dial, &liar_points).ok_or("the lie rebuilds nothing")?,
    )?;
    f.add("S-F12.fourth", FOURTH)?;
    let predicted = model::interpolate(dial, &liar_points, i64::from(FOURTH)).ok_or("no curve through the lie")?;
    f.add("S-F12.fourth.predicted", predicted)?;
    f.add("S-F12.fourth.actual", share_at(FOURTH))?;
    let handed: Vec<(u32, u32)> = TRUSTEES
        .iter()
        .map(|&x| (x, if x == LIAR.0 { LIAR.1 } else { share_at(x) }))
        .collect();
    let four: Vec<(u32, u32)> = handed.iter().copied().filter(|p| p.0 <= FOURTH).collect();
    f.add(
        "S-F12.four.curves_through_all",
        model::curves_through(dial, &four, four.len()).len(),
    )?;
    let if_liar = four
        .iter()
        .map(|suspect| {
            let rest: Vec<(u32, u32)> = four.iter().copied().filter(|p| p != suspect).collect();
            model::rebuild(dial, &rest)
        })
        .collect::<Option<Vec<u32>>>()
        .ok_or("three of the four rebuild nothing")?;
    f.add("S-F12.four.secret_if_liar", csv(&if_liar))?;
    let liar_predictions = TRUSTEES
        .iter()
        .map(|&x| model::interpolate(dial, &liar_points, i64::from(x)))
        .collect::<Option<Vec<_>>>()
        .ok_or("liar curve cannot predict")?;
    f.add("S-F12.liar.predictions", csv(&liar_predictions))?;
    let mut candidate_rows = Vec::new();
    for suspect in &four {
        let rest = four.iter().copied().filter(|p| p != suspect).collect::<Vec<_>>();
        let secret = model::rebuild(dial, &rest).ok_or("no candidate secret")?;
        let along = TRUSTEES
            .iter()
            .map(|&x| model::interpolate(dial, &rest, i64::from(x)))
            .collect::<Option<Vec<_>>>()
            .ok_or("candidate cannot predict")?;
        candidate_rows.push(format!("{secret}:{}", csv(&along)));
    }
    f.add("S-F12.four.candidates", candidate_rows.join(";"))?;
    f.add("S-F12.five.min_common", 2 * (TRUSTEES.len() - 1) - TRUSTEES.len())?;
    let near = model::curves_through(dial, &handed, handed.len() - 1);
    f.add("S-F12.five.curves_through_four", near.len())?;
    let [(near_secret, near_shares)] = near.as_slice() else {
        return Err(format!("five shares: {} curves fit four, not one", near.len()));
    };
    f.add("S-F12.five.curve_secret", near_secret)?;
    let odd: Vec<u32> = handed
        .iter()
        .zip(near_shares)
        .filter(|&(&(_, y), &c)| c != y)
        .map(|(&(x, _), _)| x)
        .collect();
    f.add("S-F12.five.odd_one_out", csv(&odd))?;

    // 23: the rule again, and four of seven
    f.add("S-F13.shares_seen", THRESHOLD - 1)?;
    f.add("S-F13.curves_left", fitting.iter().sum::<usize>())?;
    let (needed, _) = FOUR_OF_SEVEN;
    let fits: Vec<usize> = dial
        .all()
        .map(|s| model::fitting_spins(dial, &FOUR_OF_SEVEN_SEEN, s, needed - 1).len())
        .collect();
    f.add("S-F13.four_of_seven.spins", needed - 1)?;
    f.add(
        "S-F13.four_of_seven.spin_triples",
        model::tuples(dial, needed - 1).count(),
    )?;
    f.add("S-F13.four_of_seven.curves_left", fits.iter().sum::<usize>())?;
    f.add("S-F13.four_of_seven.ways_each", set_csv(fits))?;
    f.add("S-F13.four_of_seven.needed", FOUR_OF_SEVEN.0)?;
    f.add("S-F13.four_of_seven.trustees", FOUR_OF_SEVEN.1)?;
    let candidates = dial
        .all()
        .map(|s| {
            let spins = model::fitting_spins(dial, &FOUR_OF_SEVEN_SEEN, s, needed - 1);
            match spins.as_slice() {
                [a] => Ok(Recipe::new(s, a)),
                _ => Err("cubic candidate count"),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let example = candidates
        .iter()
        .find(|r| r.secret == SECRET)
        .ok_or("missing cubic example")?;
    let positions = (1..=FOUR_OF_SEVEN.1 as u32).collect::<Vec<_>>();
    f.add("S-F13.four_of_seven.recipe", csv(&example.coefficients()))?;
    f.add("S-F13.four_of_seven.cards", csv(&example.deal(dial, &positions)))?;
    f.add(
        "S-F13.four_of_seven.fourth_by_secret",
        csv(&candidates
            .iter()
            .map(|r| r.share(dial, needed as u32))
            .collect::<Vec<_>>()),
    )?;

    // S-X: the primary source's own numbers
    f.add("S-X1.shamir.year", SHAMIR_YEAR)?;
    f.pin("S-X1.shamir.locks", liu.locks().len(), "462", SHAMIR)?;
    f.pin("S-X1.shamir.keys", liu.keys(1), "252", SHAMIR)?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key and value the oracle printed on 2026-10-24 (`python3 -I -B
    /// oracle.py`, with four independently checked chapter02/03 additions; `./x
    /// shamir_secret_sharing facts-check` compares the two outputs in full.
    const ORACLE: &str = "\
S-F0.copies.who_can_open_alone	5
S-F0.copies.secrets_left_for_one	1
S-F0.keepers	1
S-F0.keeper.losses_to_shut	1
S-F0.keeper.bribes_to_open	1
S-F0.copies.losses_to_shut	5
S-F0.copies.bribes_to_open	1
S-F0.pieces	2,3
S-F0.pieces.a	2
S-F0.pieces.b	3
S-F0.pieces.sum	5
S-F0.pieces.b_alone_candidates	3,4,5,6,7,8,9,10
S-F0.pieces.b_alone_left	8
S-F0.pieces.b_alone_ruled_out	0,1,2
S-F0.pieces.both_needed	2
S-F1.trustees	5
S-F1.threshold	3
S-F1.too_few	2
S-F1.can_be_away	2
S-F1.marks	11
S-F1.opening_marks	1
S-F1.secret	5
S-F1.opening_trio	1,2,3
S-F1.opening_shares	8,3,1
S-F1.pair	4,5
S-F1.pair_shares	2,6
S-F2.top	10
S-F2.a_share	9
S-F2.b_share	7
S-F2.sum_unwrapped	16
S-F2.sum	5
S-F2.turn_back	9
S-F2.ten_plus_one	0
S-F2.secret_if_a_held	7,8,9,10,0,1,2,3,4,5,6
S-F2.if_a_held.0	7
S-F2.if_a_held.1	8
S-F2.distinct_secrets	11
S-F2.ways_each	1
S-F2.chance_each	1/11
S-F3.locks	10
S-F3.keys_each	6
S-F3.trios	10
S-F3.trios_that_open	10
S-F3.pairs	10
S-F3.pairs_stuck	10
S-F3.locks_a_pair_cannot_open	1
S-F3.keys_total	30
S-F3.shamir.scientists	11
S-F3.shamir.needed	6
S-F3.shamir.locks	462
S-F3.shamir.keys_each	252
S-F3.shamir.too_few	5
S-F3.locks_per_pair	1
S-F3.pair_lock.holders	1,2,3
S-F3.merged.pairs	2
S-F3.merged.unions	3,4
S-F3.fewest_locks	10
S-F3.dealer.numbers_each	1
S-F4.slope	4
S-F4.heights	9,13,17,21,25
S-F4.shares	9,2,6,10,3
S-F4.times2	0,2,4,6,8,10,1,3,5,7,9
S-F4.paper.holder	5
S-F4.paper.share	25
S-F4.paper.candidates	0,5,10
S-F4.paper.count	3
S-F4.needed	2
S-F4.too_few	1
S-F4.spin_count	1
S-F4.secret_position	0
S-F4.position_of.1	1
S-F4.position_of.2	2
S-F4.adjacent	2,3
S-F4.adjacent.shares	2,6
S-F4.adjacent.step	4
S-F4.adjacent.secret_unwrapped	-6
S-F4.adjacent.secret	5
S-F4.wrap.1	9,0,9
S-F4.wrap.2	13,1,2
S-F4.wrap.3	17,1,6
S-F4.wrap.4	21,1,10
S-F4.wrap.5	25,2,3
S-F4.adjacent.apart	1
S-F4.back.steps	2
S-F4.back.from	2
S-F4.back.1	2,4,9
S-F4.back.2	9,4,5
S-F4.times2.of	0,1,2,3,4,5,6,7,8,9,10
S-F4.times2.hits_each	1
S-F5.holder	2
S-F5.share	2
S-F5.secret_by_slope	2,0,9,7,5,3,1,10,8,6,4
S-F5.distinct	11
S-F5.true_slope_secret	5
S-F5.per_spin	1
S-F5.spin.0	2
S-F5.spin.1	0
S-F5.spin.1.working	2,2,0
S-F6.spins	7,7
S-F6.heights	19,47,89,145,215
S-F6.shares	8,3,1,2,6
S-F6.spin_pairs	121
S-F6.spin_count	2
S-F6.terms.1	1,5,7,7,19
S-F6.terms.2	2,5,14,28,47
S-F6.rest	3,4,5
S-F6.rest.heights	89,145,215
S-F6.paper.curves	1
S-F6.asked	2,47
S-F6.wrap.2	47,4,3
S-F6.turns	1,4,8,13,19
S-F7.fitting	11
S-F7.ways_each	1
S-F7.spins_by_secret	1:4,0:9,10:3,9:8,8:2,7:7,6:1,5:6,4:0,3:5,2:10
S-F7.chance_each	1/121
S-F7.true_spins	7:7
S-F7.straight_one	8
S-F7.secrets	0,1,2,3,4,5,6,7,8,9,10
S-F7.ruled_out	0
S-F7.pair.count	2
S-F7.asked.first	0
S-F7.asked.second	1
S-F7.spins.0	1,4
S-F7.spins.1	0,9
S-F7.check.4	4,68,2
S-F7.check.5	5,105,6
S-F7.board.0	0,5,7,6,2,6
S-F7.board.1	1,10,4,5,2,6
S-F7.board.2	2,4,1,4,2,6
S-F7.board.3	3,9,9,3,2,6
S-F7.board.4	4,3,6,2,2,6
S-F7.board.5	5,8,3,1,2,6
S-F7.board.6	6,2,0,0,2,6
S-F7.board.7	7,7,8,10,2,6
S-F7.board.8	8,1,5,9,2,6
S-F7.board.9	9,6,2,8,2,6
S-F7.board.10	10,0,10,7,2,6
S-F7.board.columns	11,11,11,11,1,1
S-F7.paper.heights	145,215
S-F7.paper.curves	11
S-F7.paper.whole_spin_secrets	5
S-F7.paper.secret0_spins	37/4,27/4
S-F7.paper.secret5_spins	7,7
S-F7.join.trustee	1
S-F7.join.share	8
S-F7.join.values_by_secret	5,10,4,9,3,8,2,7,1,6,0
S-F7.join.distinct	11
S-F7.join.secrets_left	5
S-F7.join.point	1,8
S-F7.join.count	1
S-F7.join.drop	11,1
S-F7.all_pairs.pairs	10
S-F7.all_pairs.share_pairs	121
S-F7.all_pairs.ways_each	1
S-F7.all_pairs.two_leave	2,11
S-F7.all_pairs.third_leaves	3,1
S-F8.ten.marks	10
S-F8.ten.shares	9,7,9,5,5
S-F8.ten.pair_shares	5,5
S-F8.ten.pair_secrets	5
S-F8.ten.pair_formula	5,-4
S-F8.ten.pair_formula_value	5
S-F8.pair_sum.spin_terms	-20,0,1
S-F8.ten.leftover	0
S-F8.eleven.leftover	2
S-F8.eleven.pair_formula_unwrapped	-14
S-F8.eleven.pair_formula_value	8
S-F8.ten.pairs_that_leak	9
S-F8.ten.zero_product	2,5
S-F8.ten.times5	0,5,0,5,0,5,0,5,0,5
S-F8.eleven.times5	0,5,10,4,9,3,8,2,7,1,6
S-F8.ten.rows_that_shuffle	1,3,7,9
S-F8.eleven.rows_that_shuffle	10
S-F8.smallest_prime_for_five	7
S-F8.line.shares	9,3,7,1,5
S-F8.line.holder	5
S-F8.line.holder_share	5
S-F8.line.candidates	0,5
S-F8.line.parity_holder	2
S-F8.line.parity_share	3
S-F8.line.parity_candidates	1,3,5,7,9
S-F8.line.leaky_marks	2,4,5,6,8
S-F8.ten.pair_ways	0,0,0,0,0,10,0,0,0,0
S-F8.eleven.pair_ways	1,1,1,1,1,1,1,1,1,1,1
S-F8.ten.pair_spin_pairs	10
S-F8.pair_sum.height.4	1,4,16
S-F8.pair_sum.height.5	1,5,25
S-F8.pair_sum.copies.4	5,4
S-F8.pair_sum.copies.5	4,5
S-F8.pair_sum.first_spin	20
S-F8.pair_sum.scaled.4	5,20,80
S-F8.pair_sum.scaled.5	4,20,100
S-F8.pair_sum.first_spin_left	0
S-F8.ten.twenty_lands	20,0
S-F8.ten.pair_formula_terms	5,5,-4,5
S-F8.ten.pair_formula_products	25,-20
S-F8.eleven.minus_twenty_lands	-20,2
S-F8.eleven.pair_formula_terms	5,2,-4,6
S-F8.eleven.pair_formula_products	10,-24
S-F8.eleven.second_spin_each	1
S-F8.ten.pairs_left.1	2-5,4-5
S-F8.ten.pairs_left.2	1-5,3-5
S-F8.ten.pairs_left.5	1-2,1-4,2-3,2-4,3-4
S-F8.ten.pairs_left.10	1-3
S-F8.ten.pairs_that_find	2
S-F8.ten.pair_that_learns_nothing	1,3
S-F8.ten.pairs_even_left	10
S-F8.ten.zero_product_sum	2,5,10
S-F8.ten.zero_product_mark	0
S-F8.multiplier	5
S-F8.ten.times5_values	0,5
S-F8.ten.times5_reached	2
S-F8.ten.times5_hits	5,0,0,0,0,5,0,0,0,0
S-F8.line.candidate_count	2
S-F8.eleven.times5_reached	11
S-F8.eleven.times5_hits	1,1,1,1,1,1,1,1,1,1,1
S-F8.eleven.times5_each	1
S-F8.eleven.times5_step.1	1,5
S-F8.eleven.times5_step.2	2,10
S-F8.eleven.times5_step.3	3,15,4
S-F8.eleven.line_candidates	11
S-F8.ten.split	10,2,5
S-F8.eleven.factors	1,11
S-F8.eleven.zero_product_check	2,5,10
S-F8.eleven.nonzero_range	1,10
S-F8.ten.reach	10,5,10,5,2,5,10,5,10
S-F8.eleven.reach	11,11,11,11,11,11,11,11,11,11
S-F8.primes_for_five	7,11,13
S-F9.helper_at_own	2,-1,2
S-F9.helper_at_zero	6,3,2
S-F9.weights	3,-3,1
S-F9.weights_on_dial	3,8,1
S-F9.paper.heights	19,47,89
S-F9.paper.terms	57,-141,89
S-F9.paper.sum	5
S-F9.paper.elevens	1,4,8
S-F9.sum_unwrapped	16
S-F9.sum_minus_secret	11
S-F9.secret	5
S-F9.helper.1.position	1
S-F9.helper.1.at_trio	1,0,0
S-F9.helper.1.zeros	2,3
S-F9.helper.1.factors_at_own	-1,-2
S-F9.helper.1.own	2
S-F9.helper.1.scaled	19,1
S-F9.helper.2.position	2
S-F9.helper.2.at_trio	0,1,0
S-F9.helper.2.zeros	1,3
S-F9.helper.2.factors_at_own	1,-1
S-F9.helper.2.own	-1
S-F9.helper.2.scaled	47,2
S-F9.helper.3.position	3
S-F9.helper.3.at_trio	0,0,1
S-F9.helper.3.zeros	1,2
S-F9.helper.3.factors_at_own	2,1
S-F9.helper.3.own	2
S-F9.helper.3.scaled	89,3
S-F9.sum_at.1	19
S-F9.sum_at.2	47
S-F9.sum_at.3	89
S-F9.sum_at.1.zero_helpers	2,3
S-F9.helper.on	1
S-F9.helper.off	0
S-F9.helper.others	2
S-F9.helper.1.factors_at_zero	-2,-3
S-F9.helper.1.at_zero	6
S-F9.helper.1.weight	3
S-F9.helper.2.factors_at_zero	-1,-3
S-F9.helper.2.at_zero	3
S-F9.helper.2.weight	-3
S-F9.helper.3.factors_at_zero	-1,-2
S-F9.helper.3.at_zero	2
S-F9.helper.3.weight	1
S-F9.paper.weighted	3,19,-3,47,1,89
S-F9.cards.weighted	3,8,-3,3,1,1
S-F9.share_terms	24,-9,1
S-F9.sum_lands	16,5
S-F9.split.1	19,8,1
S-F9.split.2	47,3,4
S-F9.split.3	89,1,8
S-F9.weighted_elevens	-1
S-F9.sum_split	16,5,1
S-F10.trio	2,4,5
S-F10.trio_shares	3,2,6
S-F10.helper_at_own	6,-2,3
S-F10.helper_at_zero	20,10,8
S-F10.weights	10/3,-5,8/3
S-F10.paper.heights	47,145,215
S-F10.paper.sum	5
S-F10.fraction_terms	10,-10,16
S-F10.fraction_sum	16
S-F10.other.spins	0,1
S-F10.other.heights	9,21,30
S-F10.other.shares	9,10,8
S-F10.other.fraction_sum	4/3
S-F10.other.secret	5
S-F10.fractional_spin_pairs	80
S-F10.clear	3
S-F10.cleared_weights	10,-15,8
S-F10.other.cleared_terms	90,-150,64
S-F10.other.cleared_sum	4
S-F10.other.cleared_lands	4
S-F10.cleared_sum	48
S-F10.cleared_lands	4
S-F10.times3	0,3,6,9,1,4,7,10,2,5,8
S-F10.thrice	5
S-F10.thrice_unwrapped	15
S-F10.third	4
S-F10.weights_on_dial	7,6,10
S-F10.sum_unwrapped	93
S-F10.secret	5
S-F10.inverses	1:1,2:6,3:4,4:3,5:9,6:2,7:8,8:7,9:5,10:10
S-F10.ten.invertible	1,3,7,9
S-F10.partner_product	12
S-F10.undo_product	16
S-F10.weight_products	40,-5,32
S-F10.cleared_terms	30,-30,48
S-F10.dial_terms	21,12,60
S-F10.whole_turns	8
S-F10.ten.noninvertible	2,4,5,6,8
S-F11.trios	10
S-F11.trios_giving_secret	10
S-F11.trial	2,1,8
S-F11.difference	8,5,1
S-F11.trial_shares	3,2,9
S-F11.difference_row	8,3,0,10,0,3,8,4,2,2,4
S-F11.left_row	9,10,0,1,2,3,4,5,6,7,8
S-F11.right_row	7,8,9,10,0,1,2,3,4,5,6
S-F11.roots	2,4
S-F11.third_difference	3
S-F11.most_zeros	2
S-F11.line_most_zeros	1
S-F11.ten.line_most_zeros	5
S-F11.ten.opening_trio_secrets	5
S-F11.ten.trios_that_pin	9
S-F11.ten.trio_table	1,2,3:5;1,2,4:5;1,2,5:5;1,3,4:5;1,3,5:0,5;1,4,5:5;2,3,4:5;2,3,5:5;2,4,5:5;3,4,5:5
S-F11.ten.trio	1,3,5
S-F11.ten.trio_shares	9,9,5
S-F11.ten.trio_secrets	0,5
S-F11.ten.ghost	5,5,0
S-F11.ten.ghost.heights	10,20,30
S-F11.ten.ghost.zeros	1,3,5,7,9
S-F11.ten.moved_recipe	0,2,7
S-F12.guess_chance	1/11
S-F12.guess_misses	10
S-F12.nonzero_last.excluded	8
S-F12.nonzero_last.left	10
S-F12.straight_recipe	8,4,0
S-F12.straight_heights	24,28
S-F12.liar	3
S-F12.liar_true	1
S-F12.liar_hands_in	2
S-F12.liar_result_unwrapped	17
S-F12.liar_result	6
S-F12.fourth	4
S-F12.fourth.predicted	5
S-F12.fourth.actual	2
S-F12.four.curves_through_all	0
S-F12.four.secret_if_liar	8,3,5,6
S-F12.liar.predictions	8,3,2,5,1
S-F12.four.candidates	8:5,3,2,2,3;3:8,4,2,2,4;5:8,3,1,2,6;6:8,3,2,5,1
S-F12.five.min_common	3
S-F12.five.curves_through_four	1
S-F12.five.curve_secret	5
S-F12.five.odd_one_out	3
S-F13.shares_seen	2
S-F13.curves_left	11
S-F13.four_of_seven.spins	3
S-F13.four_of_seven.spin_triples	1331
S-F13.four_of_seven.curves_left	11
S-F13.four_of_seven.ways_each	1
S-F13.four_of_seven.needed	4
S-F13.four_of_seven.trustees	7
S-F13.four_of_seven.recipe	5,4,7,10
S-F13.four_of_seven.cards	4,0,9,3,9,10,0
S-F13.four_of_seven.fourth_by_secret	8,7,6,5,4,3,2,1,0,10,9
S-X1.shamir.year	1979
S-X1.shamir.locks	462
S-X1.shamir.keys	252
";

    #[test]
    fn the_facts_print_what_the_oracle_prints() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
        assert!(diff.is_empty(), "{}", diff.join("\n"));
        assert_eq!(f.len(), 379);
    }

    /// Planted negatives: a different secret, spins or dial changes facts
    /// the video speaks, so the comparison above is not vacuous.
    #[test]
    fn a_changed_example_is_caught() {
        let dial = Dial::vault();
        assert_ne!(
            Recipe::new(SECRET, &[7, 6]).deal(dial, &TRUSTEES),
            Recipe::example().deal(dial, &TRUSTEES)
        );
        let ten = Dial::new(TEN);
        let leaked = View::Shares {
            points: dealt(ten, &Recipe::example(), &PAIR),
            spins: 2,
        };
        assert_eq!(leaked.possible(ten).len(), 1, "on ten marks the pair learns the secret");
        let one = View::Shares {
            points: dealt(dial, &Recipe::example(), &[4]),
            spins: 2,
        };
        assert_eq!(one.possible(dial).len(), 11);
        assert!(
            one_way_each(dial, &dealt(dial, &Recipe::example(), &[4])).is_err(),
            "one share fits eleven ways"
        );
    }

    /// The fact the opening speaks, "two shares leave every secret", at its
    /// edges: every pair of trustees and every pair of share values.
    #[test]
    fn every_pair_of_trustees_and_values_leaves_all_eleven() {
        let dial = Dial::vault();
        for pair in combinations(&TRUSTEES, 2) {
            for values in model::tuples(dial, 2) {
                let points: Vec<(u32, u32)> = pair.iter().copied().zip(values).collect();
                assert_eq!(one_way_each(dial, &points).unwrap().len(), 11, "{points:?}");
            }
        }
    }
}
