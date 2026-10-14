//! Every number Video I shows or says, read off the pictures `model.rs`
//! builds and checked two ways: `pin` against a literal from the fact sheet
//! (the owner's picture, or the source a cited number comes from), and
//! `./x visual_proofs facts-check` against `oracle.py` (same keys), which
//! rebuilds every picture by its own route.
//!
//! A key that ends in `.through` is a slow check's reach: the largest size
//! up to which the picture was built piece by piece and found to fit with
//! nothing over. If such a check fails, `facts()` fails and nothing renders.

use explainer_kit::Facts;

use crate::model::{
    Ask, Q, QUIZZES, Rect, blocks, choose, circle, cubes, first_failure, layers, lie, pythagoras, q, series, stacks,
    tiles,
};

/// Numbers quoted from the sources (fact sheet §Sources; the oracle prints
/// the same literals).
pub mod cited {
    /// Leo Moser, "On the Danger of Induction", Mathematics Magazine 23(2),
    /// November-December 1949, p. 109.
    pub const MOSER: (u32, u32) = (1949, 109);
    /// Nicomachus of Gerasa, Introduction to Arithmetic, about the year 100:
    /// Book II, chapter 20, section 5 (cubes as runs of odd numbers).
    pub const NICOMACHUS: (u32, &str) = (100, "II.20.5");
    /// Aryabhata, Aryabhatiya (499), Ganitapada stanza 22: "The square of
    /// the sum of the (original) series is the sum of the cubes."
    pub const ARYABHATA: (u32, u32) = (499, 22);
    /// Al-Karaji's proof by L-shaped layers, about the year 1000 (Heath, A
    /// History of Greek Mathematics, vol. 1, 1921, p. 109).
    pub const AL_KARAJI: u32 = 1000;
    /// Alan L. Fry, "Proof without Words: Sum of Cubes", Mathematics
    /// Magazine 58(1), January 1985, p. 11.
    pub const FRY: (u32, u32) = (1985, 11);
    /// Man-Keung Siu, "Proof without Words: Sum of Squares", Mathematics
    /// Magazine 57(2), March 1984, p. 92.
    pub const SIU: (u32, u32) = (1984, 92);
    /// Warren Page, "Proof without Words: Geometric Sums", Mathematics
    /// Magazine 54(4), September 1981, p. 201.
    pub const PAGE: (u32, u32) = (1981, 201);
    /// Sunday A. Ajose and Roger B. Nelsen, "Proof without Words: Geometric
    /// Series", Mathematics Magazine 67(3), June 1994, p. 230.
    pub const AJOSE_NELSEN: (u32, u32) = (1994, 230);
    /// Euclid, Elements, Book II, Proposition 4.
    pub const EUCLID: &str = "II.4";
    /// Martin Gardner, Mathematics, Magic and Mystery (Dover, 1956): Curry's
    /// paradox, p. 139 ("In 1953 he conceived the brilliant notion…"); the
    /// triangle forms, p. 145.
    pub const CURRY: u32 = 1953;
    pub const GARDNER: (u32, u32, u32) = (1956, 139, 145);
    /// Roger B. Nelsen, Proofs Without Words (MAA, 1993) and Proofs Without
    /// Words II (MAA, 2000).
    pub const NELSEN: (u32, u32) = (1993, 2000);
}

/// The sizes the chapters draw (the quizzes are `model::QUIZZES`).
pub mod shown {
    /// The hook: points on the circle.
    pub const POINTS: usize = 6;
    /// L-shaped layers, staircase columns and cubes drawn.
    pub const LAYERS: i64 = 5;
    pub const STAIRS: i64 = 5;
    pub const CUBES: i64 = 5;
    /// Siu's box: stacks of four layers.
    pub const STACK: i64 = 4;
    /// Halvings and quarterings drawn with their labels.
    pub const HALVES: u32 = 5;
    pub const QUARTERS: u32 = 3;
    /// The cut square, cube and fourth power: the long part and the short.
    pub const CUT: (i64, i64) = (3, 2);
    /// The right triangle's legs, and a pair whose long side is no whole number.
    pub const LEGS: (i64, i64) = (3, 4);
    pub const OTHER_LEGS: (i64, i64) = (2, 5);

    /// How far the slow checks run (the `.through` facts).
    pub const LAYERS_THROUGH: i64 = 60;
    pub const CUBES_THROUGH: i64 = 40;
    pub const STACKS_THROUGH: i64 = 24;
    pub const HALVES_THROUGH: u32 = 40;
    pub const QUARTERS_THROUGH: u32 = 20;
    pub const CUT_THROUGH: i64 = 5;
    pub const LEGS_THROUGH: i64 = 60;
    /// How far the sums themselves are checked in whole numbers.
    pub const SUMS_THROUGH: i64 = 1000;
}

fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

/// The reach of a slow check, or the first size at which it fails.
fn through(n: i64, what: &str, check: impl Fn(i64) -> Result<(), String>) -> Result<i64, String> {
    match first_failure(n, check) {
        None => Ok(n),
        Some((k, e)) => Err(format!("{what} fails at size {k}: {e}")),
    }
}

fn ensure(ok: bool, what: impl FnOnce() -> String) -> Result<(), String> {
    if ok { Ok(()) } else { Err(what()) }
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    hook(&mut f)?;
    odd_layers(&mut f)?;
    staircases(&mut f)?;
    sum_of_cubes(&mut f)?;
    sum_of_squares(&mut f)?;
    halves(&mut f)?;
    quarters(&mut f)?;
    cut_blocks(&mut f)?;
    four_triangles(&mut f)?;
    the_lie(&mut f)?;
    quizzes(&mut f)?;
    sources(&mut f)?;
    Ok(f)
}

/// F-C: the pattern in the cubes, and the circle that breaks a pattern.
fn hook(f: &mut Facts) -> Result<(), String> {
    let n = shown::CUBES;
    let totals: Vec<i64> = (1..=n).map(|m| (1..=m).map(|k| k * k * k).sum()).collect();
    let roots: Vec<i64> = totals
        .iter()
        .map(|&t| (1..=t).find(|r| r * r >= t).unwrap_or(0))
        .collect();
    ensure(roots.iter().zip(&totals).all(|(r, t)| r * r == *t), || {
        "a cube total is not a square".to_owned()
    })?;
    f.pin(
        "F-C.cube_totals",
        list(&totals),
        "1,9,36,100,225",
        "the owner's picture: 1^3 + ... + 5^3",
    )?;
    f.pin(
        "F-C.cube_roots",
        list(&roots),
        "1,3,6,10,15",
        "the owner's picture: a 15 by 15 square",
    )?;

    let regions: Vec<u64> = (1..=shown::POINTS)
        .map(|k| circle::regions(&circle::drawn(k)))
        .collect();
    f.pin(
        "F-C.regions",
        list(&regions),
        "1,2,4,8,16,31",
        "Moser 1949, p. 109: f(6) = 31",
    )?;
    f.add(
        "F-C.doubling",
        list((1..=shown::POINTS as u32).map(|k| 1u64 << (k - 1))),
    )?;
    f.add("F-C.regions.seven", circle::regions(&circle::drawn(7)))?;
    let six = circle::drawn(shown::POINTS);
    f.add("F-C.lines", circle::lines(shown::POINTS).len())?;
    f.add("F-C.crossings", circle::crossings(&six).len())?;
    f.add("F-C.fours", choose(shown::POINTS as u64, 4))?;
    f.add("F-C.closed_form", list((1..=7).map(circle::closed_form)))?;
    let through_one = circle::three_through_one();
    f.add("F-C.three_through_one.crossings", circle::crossings(&through_one).len())?;
    f.add("F-C.three_through_one.regions", circle::regions(&through_one))?;
    // 02_6: "six evenly spaced points give 30", by the regular hexagon's stretch
    f.add(
        "F-C.evenly_spaced.regions",
        circle::regions(&circle::evenly_spaced_hexagon()),
    )?;
    Ok(())
}

/// F-O: unit squares in L-shaped layers.
fn odd_layers(f: &mut Facts) -> Result<(), String> {
    let n = shown::LAYERS;
    let sizes: Vec<usize> = (1..=n).map(|k| layers::layer(k).len()).collect();
    f.add("F-O.layers", list(&sizes))?;
    f.add(
        "F-O.totals",
        list((1..=n).map(|m| sizes[..m as usize].iter().sum::<usize>())),
    )?;
    f.add("F-O.sum", sizes.iter().sum::<usize>())?;
    f.add("F-O.square", format!("{n}x{n}"))?;
    f.add("F-O.next", layers::layer(n + 1).len())?;
    f.add(
        "F-O.tiled.through",
        through(shown::LAYERS_THROUGH, "the L-shaped layers", |m| {
            tiles((1..=m).flat_map(layers::layer), Rect::new(0, 0, m, m), &[])?;
            ensure(layers::layer(m).len() as i64 == 2 * m - 1, || {
                "a layer is not 2n - 1".to_owned()
            })
        })?,
    )?;
    f.add(
        "F-O.sum.through",
        through(shown::SUMS_THROUGH, "the sum of odd numbers", |m| {
            ensure((1..=m).map(|k| 2 * k - 1).sum::<i64>() == m * m, || {
                "not n squared".to_owned()
            })
        })?,
    )?;
    f.add("F-O.hundredth", 2 * 100 - 1)?;
    f.add("F-O.sum.hundred", (1..=100).map(|k| 2 * k - 1).sum::<i64>())?;
    Ok(())
}

/// F-T: two staircases make a rectangle.
fn staircases(f: &mut Facts) -> Result<(), String> {
    let n = shown::STAIRS;
    let stair = layers::staircase(n);
    f.add(
        "F-T.columns",
        list((0..n).map(|x| stair.iter().filter(|c| c.0 == x).count())),
    )?;
    f.add("F-T.count", stair.len())?;
    f.add("F-T.rectangle", format!("{}x{}", n, n + 1))?;
    f.add("F-T.rectangle.cells", stair.len() + layers::turned(n).len())?;
    f.add("F-T.counts", list((1..=n).map(|m| layers::staircase(m).len())))?;
    f.add(
        "F-T.tiled.through",
        through(shown::LAYERS_THROUGH, "the two staircases", |m| {
            let both = layers::staircase(m).into_iter().chain(layers::turned(m));
            tiles(both, Rect::new(0, 0, m, m + 1), &[])
        })?,
    )?;
    f.add(
        "F-T.sum.through",
        through(shown::SUMS_THROUGH, "the sum of whole numbers", |m| {
            ensure(2 * (1..=m).sum::<i64>() == m * (m + 1), || "not n(n + 1)/2".to_owned())
        })?,
    )?;
    f.add("F-T.count.hundred", (1..=100).sum::<i64>())?;
    Ok(())
}

/// F-K: the cubes' slabs tile a square.
fn sum_of_cubes(f: &mut Facts) -> Result<(), String> {
    let n = shown::CUBES;
    let pieces = cubes::square(n);
    f.add("F-K.cubes", list((1..=n).map(|k| k * k * k)))?;
    f.add("F-K.sides", list((1..=n).map(cubes::side)))?;
    f.add("F-K.side", cubes::side(n))?;
    f.add("F-K.unit_squares", pieces.iter().map(|p| p.rect.area()).sum::<i64>())?;
    let mut slabs: Vec<(i64, i64)> = pieces.iter().map(|p| (p.cube, p.slab)).collect();
    slabs.sort_unstable();
    slabs.dedup();
    f.add("F-K.slabs", slabs.len())?;
    f.add(
        "F-K.cut_slabs",
        pieces.iter().filter(|p| p.part == cubes::Part::HalfRight).count(),
    )?;
    f.add("F-K.pieces", pieces.len())?;
    f.add(
        "F-K.layer_areas",
        list((1..=n).map(|k| cubes::layer(k).iter().map(|p| p.rect.area()).sum::<i64>())),
    )?;
    // each arm of layer k: k wide, as long as the square before it, and how many slabs that is
    for k in [4, 5] {
        let before = cubes::side(k - 1);
        f.add(&format!("F-K.arm.{k}"), format!("{k}x{before}"))?;
        f.add(&format!("F-K.arm.{k}.slabs"), q(i128::from(before), i128::from(k)))?;
    }
    // the next size: the sixth cube's layer round the 15 by 15 square
    let sixth = n + 1;
    f.add(
        "F-K.sixth.cube",
        cubes::layer(sixth).iter().map(|p| p.rect.area()).sum::<i64>(),
    )?;
    f.add("F-K.sixth.side", cubes::side(sixth))?;
    // the sixth total of 02_1's running sum: the five cubes and the sixth
    f.add(
        "F-K.sixth.total",
        (1..=sixth)
            .map(|k| cubes::layer(k).iter().map(|p| p.rect.area()).sum::<i64>())
            .sum::<i64>(),
    )?;
    f.add(&format!("F-K.arm.{sixth}"), format!("{sixth}x{}", cubes::side(n)))?;
    f.add(
        &format!("F-K.arm.{sixth}.slabs"),
        q(i128::from(cubes::side(n)), i128::from(sixth)),
    )?;
    // two more sizes, for 06_3's strips beside the sixth layer: an odd cube's
    // arm is whole slabs and an even cube's ends in a half
    for k in [sixth + 1, sixth + 2] {
        let before = cubes::side(k - 1);
        f.add(&format!("F-K.arm.{k}"), format!("{k}x{before}"))?;
        f.add(&format!("F-K.arm.{k}.slabs"), q(i128::from(before), i128::from(k)))?;
    }
    f.add(
        "F-K.odd_runs",
        (1..=n).map(|k| list(cubes::odd_run(k))).collect::<Vec<_>>().join("|"),
    )?;
    f.add(
        "F-K.tiled.through",
        through(shown::CUBES_THROUGH, "the cubes' slabs", |m| {
            let side = cubes::side(m);
            tiles(
                cubes::square(m).iter().flat_map(|p| p.rect.cells()),
                Rect::new(0, 0, side, side),
                &[],
            )?;
            let layer: i64 = cubes::layer(m).iter().map(|p| p.rect.area()).sum();
            ensure(layer == m * m * m, || "a layer is not a cube".to_owned())
        })?,
    )?;
    f.add(
        "F-K.sum.through",
        through(shown::SUMS_THROUGH, "the sum of cubes", |m| {
            let (cubes, whole): (i64, i64) = ((1..=m).map(|k| k * k * k).sum(), (1..=m).sum());
            ensure(cubes == whole * whole, || "not the square of the sum".to_owned())
        })?,
    )?;
    Ok(())
}

/// F-S: three stacks fill Siu's box.
fn sum_of_squares(f: &mut Facts) -> Result<(), String> {
    let n = shown::STACK;
    let stack = stacks::standard(n);
    f.add(
        "F-S.layers",
        list((0..n).map(|l| stack.iter().filter(|c| c.2 == l).count())),
    )?;
    f.add("F-S.stack", stack.len())?;
    f.add(
        "F-S.three_stacks",
        (0..3).map(|w| stacks::placed(n, w).len()).sum::<usize>(),
    )?;
    // the locked block before the slice, measured off the cubes: how far they
    // reach across and back, and how many layers are full
    let all: Vec<stacks::Cell> = (0..3).flat_map(|w| stacks::placed(n, w)).collect();
    let reach = |pick: fn(&stacks::Cell) -> i64| all.iter().map(pick).max().map_or(0, |m| m + 1);
    let (wide, deep) = (reach(|c| c.0), reach(|c| c.1));
    let full = (0..reach(|c| c.2))
        .take_while(|&z| all.iter().filter(|c| c.2 == z).count() as i64 == wide * deep)
        .count();
    f.add("F-S.block", format!("{wide}x{deep}x{full}"))?;
    let height = Q::int(n) + q(1, 2);
    f.add("F-S.box", format!("{}x{}x{}", n, n + 1, height))?;
    f.add("F-S.box.volume", Q::int(n * (n + 1)) * height)?;
    f.add("F-S.top", stacks::top(n).len())?;
    f.add("F-S.top.halves", 2 * stacks::top(n).len())?;
    f.add("F-S.top.layer", n * (n + 1))?;
    f.add("F-S.sums", list((1..=5).map(|m| (1..=m).map(|k| k * k).sum::<i64>())))?;
    f.add("F-S.turns", list(stacks::TURNS.map(stacks::determinant)))?;
    f.add(
        "F-S.filled.through",
        through(shown::STACKS_THROUGH, "Siu's box", stacks::fills_the_box)?,
    )?;
    // the next height: the same solid lifted onto three new slabs
    let next = n + 1;
    f.add("F-S.grown.box", format!("{}x{}x{}", next, next + 1, next))?;
    f.add("F-S.grown.top", stacks::top(next).len())?;
    f.add("F-S.grown.slab", stacks::new_base(next, 0).len())?;
    f.add(
        "F-S.grows.through",
        through(shown::STACKS_THROUGH, "the step to the next height", stacks::grows)?,
    )?;
    f.add(
        "F-S.one.cubes",
        (0..3).map(|w| stacks::placed(1, w).len()).sum::<usize>(),
    )?;
    f.add("F-S.one.box", format!("1x2x{}", Q::ONE + q(1, 2)))?;
    // 07_2 asks for the fewest stacks: the least number for which a box exists
    // when only the cubes that stick out on top may be sliced, and that box
    let fewest = (1..=12i64)
        .find(|&m| !stacks::half_boxes(shown::STACK, m).is_empty())
        .unwrap_or(0);
    f.add("F-S.fewest", fewest)?;
    f.add(
        "F-S.fewest.box",
        list(
            stacks::half_boxes(shown::STACK, fewest)
                .iter()
                .map(|&(w, d, h)| format!("{w}x{d}x{}", q(i128::from(2 * h + 1), 2))),
        ),
    )?;
    f.add(
        "F-S.sum.through",
        through(shown::SUMS_THROUGH, "the sum of squares", |m| {
            // three stacks are n(n + 1)(n + 1/2), in halves to stay whole
            let squares: i64 = (1..=m).map(|k| k * k).sum();
            ensure(6 * squares == m * (m + 1) * (2 * m + 1), || "not Siu's box".to_owned())
        })?,
    )?;
    Ok(())
}

/// F-H: a square halved again and again.
fn halves(f: &mut Facts) -> Result<(), String> {
    let n = shown::HALVES;
    let areas: Vec<Q> = (1..=n).map(|k| series::area(series::half(k))).collect();
    let filled = |k: u32| (1..=k).fold(Q::ZERO, |s, i| s + series::area(series::half(i)));
    f.add("F-H.pieces", list(&areas))?;
    f.add("F-H.filled", list((1..=n).map(filled)))?;
    f.add("F-H.gap", series::area(series::left_after_halves(n)))?;
    // how many pieces before the gap is under a thousandth, and under a millionth
    for (name, bound) in [("thousandth", q(1, 1000)), ("millionth", q(1, 1_000_000))] {
        let k = (1..=shown::HALVES_THROUGH)
            .find(|&k| series::area(series::left_after_halves(k)) < bound)
            .ok_or("the gap never gets that small")?;
        f.add(&format!("F-H.pieces_to.{name}"), k)?;
        f.add(
            &format!("F-H.gap_at.{name}"),
            series::area(series::left_after_halves(k)),
        )?;
    }
    // 10_4's note for programmers: where a 64-bit float's running sum reaches 1.0
    f.add("F-H.float.reaches_one", series::float_pieces_to_one())?;
    f.add(
        "F-H.checked.through",
        through(i64::from(shown::HALVES_THROUGH), "the halved square", |m| {
            let m = m as u32;
            let gap = series::area(series::left_after_halves(m));
            ensure(filled(m) + gap == Q::ONE, || {
                "the pieces and the gap are not the square".to_owned()
            })?;
            ensure(gap == series::area(series::half(m)), || {
                "the gap is not the last piece".to_owned()
            })?;
            ensure(filled(m) < Q::ONE, || "a finite sum reached one".to_owned())
        })?,
    )?;
    Ok(())
}

/// F-R: a square quartered again and again; one colour's share.
fn quarters(f: &mut Facts) -> Result<(), String> {
    let n = shown::QUARTERS;
    let share = |j: u32| (1..=j).fold(Q::ZERO, |s, i| s + series::area(series::quarters(i)[0]));
    f.add(
        "F-R.pieces",
        list((1..=n).map(|j| series::area(series::quarters(j)[0]))),
    )?;
    f.add("F-R.share", list((1..=n).map(share)))?;
    f.add(
        "F-R.left",
        list((1..=n).map(|j| series::area(series::left_after_quarters(j)))),
    )?;
    f.add("F-R.shares", series::quarters(1).len())?;
    f.add("F-R.limit", q(1, 3))?;
    f.add("F-R.short", q(1, 3) - share(n))?;
    f.add(
        "F-R.checked.through",
        through(i64::from(shown::QUARTERS_THROUGH), "the quartered square", |m| {
            let m = m as u32;
            let [a, b, c] = series::quarters(m);
            ensure(
                series::area(a) == series::area(b) && series::area(b) == series::area(c),
                || "the three squares of a level differ".to_owned(),
            )?;
            let left = series::area(series::left_after_quarters(m));
            ensure(share(m) + share(m) + share(m) + left == Q::ONE, || {
                "three shares and what is left are not the square".to_owned()
            })?;
            ensure(q(1, 3) - share(m) == left * q(1, 3), || {
                "the share is not short by a third of what is left".to_owned()
            })
        })?,
    )?;
    Ok(())
}

/// F-B: a square, a cube and a fourth power cut once along every side.
fn cut_blocks(f: &mut Facts) -> Result<(), String> {
    let (a, b) = shown::CUT;
    f.add("F-B.parts", format!("{a},{b}"))?;
    for (name, dim) in [("square", 2usize), ("cube", 3), ("fourth", 4)] {
        let cut = blocks::cut(a, b, dim);
        let kinds = blocks::kinds(dim);
        f.add(&format!("F-B.{name}.blocks"), cut.len())?;
        f.add(&format!("F-B.{name}.kinds"), list(&kinds))?;
        // one block of each kind, by how many short sides it has
        let each: Vec<i64> = (0..=dim)
            .map(|j| cut.iter().find(|x| x.bs == j).map_or(0, blocks::Block::volume))
            .collect();
        f.add(&format!("F-B.{name}.each"), list(&each))?;
        f.add(
            &format!("F-B.{name}.by_kind"),
            list(each.iter().zip(&kinds).map(|(v, &k)| v * k as i64)),
        )?;
        f.add(
            &format!("F-B.{name}.total"),
            cut.iter().map(blocks::Block::volume).sum::<i64>(),
        )?;
        f.add(&format!("F-B.{name}.power"), (a + b).pow(dim as u32))?;
    }
    // Pascal's rule, counted: a fourth-power block is a cube block given a long or a short fourth side
    let (three, four) = (blocks::kinds(3), blocks::kinds(4));
    ensure((1..=3).all(|j| four[j] == three[j] + three[j - 1]), || {
        "Pascal's rule fails".to_owned()
    })?;
    f.add(
        "F-B.fourth.from_cube",
        list((1..=3).map(|j| format!("{}+{}", three[j - 1], three[j]))),
    )?;
    let mut filled = 0;
    for dim in 1..=4 {
        for a in 1..=shown::CUT_THROUGH {
            for b in 1..=shown::CUT_THROUGH {
                blocks::fills_the_cube(a, b, dim)?;
                filled += 1;
            }
        }
    }
    f.add("F-B.filled.cuts", filled)?;
    Ok(())
}

/// F-P: four triangles in a square, two ways.
fn four_triangles(f: &mut Facts) -> Result<(), String> {
    use pythagoras::{area2, corners, is_square, left_over, slid, slides, tilted};
    let (a, b) = shown::LEGS;
    f.add("F-P.legs", format!("{a},{b}"))?;
    f.add("F-P.big", (a + b) * (a + b))?;
    f.add("F-P.triangle", q(i128::from(area2(&corners(a, b)[0])), 2))?;
    f.add(
        "F-P.triangles",
        q(i128::from(corners(a, b).iter().map(|t| area2(t)).sum::<i64>()), 2),
    )?;
    f.add("F-P.tilted", q(i128::from(area2(&tilted(a, b))), 2))?;
    let (on_a, on_b) = left_over(a, b);
    f.add("F-P.squares", format!("{},{}", on_a.area(), on_b.area()))?;
    f.add(
        "F-P.long_side",
        (1..=a + b).find(|c| c * c == a * a + b * b).unwrap_or(0),
    )?;
    f.add("F-P.slides", slides(a, b).map(|d| format!("{} {}", d.0, d.1)).join(","))?;
    let (c, d) = shown::OTHER_LEGS;
    f.add("F-P.other.legs", format!("{c},{d}"))?;
    f.add("F-P.other.tilted", q(i128::from(area2(&tilted(c, d))), 2))?;
    f.add(
        "F-P.other.whole_side",
        (1..=c + d)
            .find(|s| s * s == c * c + d * d)
            .map_or("none".to_owned(), |s| s.to_string()),
    )?;
    f.add(
        "F-P.checked.through",
        through(shown::LEGS_THROUGH, "the four triangles", |x| {
            (1..=shown::LEGS_THROUGH).try_for_each(|y| {
                let hole = tilted(x, y);
                ensure(is_square(&hole), || "the tilted shape is not a square".to_owned())?;
                ensure(area2(&hole) == 2 * (x * x + y * y), || {
                    "the tilted square is not a² + b²".to_owned()
                })?;
                // the slid triangles pair up into two x by y rectangles cut corner to corner
                let u = slid(x, y);
                let pair = |i: usize, j: usize| {
                    let mut c: Vec<_> = u[i].iter().chain(&u[j]).copied().collect();
                    c.sort_unstable();
                    c.dedup();
                    c
                };
                let s = x + y;
                ensure(pair(0, 2) == [(0, 0), (0, y), (x, 0), (x, y)], || {
                    "the first pair is not a rectangle".to_owned()
                })?;
                ensure(pair(1, 3) == [(x, y), (x, s), (s, y), (s, s)], || {
                    "the second pair is not a rectangle".to_owned()
                })?;
                let (on_x, on_y) = left_over(x, y);
                let rects = [Rect::new(0, 0, x, y), Rect::new(x, y, y, x), on_x, on_y];
                tiles(rects.iter().flat_map(Rect::cells), Rect::new(0, 0, x + y, x + y), &[])
            })
        })?,
    )?;
    Ok(())
}

/// F-L: the picture that lies.
fn the_lie(f: &mut Facts) -> Result<(), String> {
    use pythagoras::area2;
    let (first, second) = (lie::first(), lie::second());
    let pieces = [
        q(i128::from(area2(&first.big).abs()), 2),
        q(i128::from(area2(&first.small).abs()), 2),
        Q::int(first.long_l.len() as i64),
        Q::int(first.short_l.len() as i64),
    ];
    f.add("F-L.pieces", list(pieces))?;
    f.add("F-L.pieces.total", pieces.into_iter().fold(Q::ZERO, |s, x| s + x))?;
    f.add("F-L.frame", format!("{}x{}", lie::WIDE, lie::TALL))?;
    f.add("F-L.true_triangle", q(i128::from(lie::WIDE * lie::TALL), 2))?;
    f.add(
        "F-L.slopes",
        format!("{},{}", lie::slope(&first.big), lie::slope(&first.small)),
    )?;
    f.add("F-L.straight_slope", q(i128::from(lie::TALL), i128::from(lie::WIDE)))?;
    f.add("F-L.outline.first", q(i128::from(area2(&lie::outline(&first))), 2))?;
    f.add("F-L.outline.second", q(i128::from(area2(&lie::outline(&second))), 2))?;
    f.add(
        "F-L.sliver",
        q(
            i128::from(area2(&lie::outline(&second)) - area2(&lie::outline(&first))),
            2,
        ),
    )?;
    let thickest = (0..=lie::WIDE)
        .map(|x| lie::edge_at(&second, x) - lie::edge_at(&first, x))
        .max()
        .unwrap_or(Q::ZERO);
    f.add("F-L.sliver.thickest", thickest)?;
    f.add("F-L.holes.first", lie::holes(&first).len())?;
    f.add("F-L.holes.second", lie::holes(&second).len())?;
    f.add(
        "F-L.hole.at",
        list(lie::holes(&second).iter().map(|c| format!("{} {}", c.0, c.1))),
    )?;
    // how far each outline's bend is from the straight edge, in squares
    f.add("F-L.dip", lie::straight_edge_at(first.bend.0) - Q::int(first.bend.1))?;
    f.add(
        "F-L.bulge",
        Q::int(second.bend.1) - lie::straight_edge_at(second.bend.0),
    )?;
    f.add("F-L.slides", lie::SLIDES.map(|d| format!("{} {}", d.0, d.1)).join(","))?;
    // the sliver cut along the grid and slid, by whole squares, into the hole
    f.add("F-L.sliver.pieces", lie::sliver_pieces().len())?;
    f.add(
        "F-L.sliver.fold",
        lie::fold()
            .iter()
            .map(|d| format!("{} {}", d.0, d.1))
            .collect::<Vec<_>>()
            .join(","),
    )?;
    // the slopes as the decimals a ruler would have to tell apart
    let thousandths = [lie::slope(&first.big), lie::slope(&first.small)].map(|s| s * Q::int(1000));
    f.add("F-L.slopes.thousandths", list(thousandths))?;
    // 14_6: how far apart the two slopes are, 2/5 less 3/8
    f.add("F-L.slopes.differ", lie::slope(&first.small) - lie::slope(&first.big))?;
    // neither bend lies on the straight edge from corner to corner
    let corner = (lie::WIDE, lie::TALL);
    f.add(
        "F-L.bends.on_the_edge",
        [&first, &second]
            .iter()
            .filter(|a| pythagoras::straight((0, 0), a.bend, corner))
            .count(),
    )?;
    Ok(())
}

/// F-Q: every quiz, by name: its countdown and its answer.
fn quizzes(f: &mut Facts) -> Result<(), String> {
    f.add("F-Q.count", QUIZZES.len())?;
    f.add("F-Q.names", list(QUIZZES.iter().map(|x| x.name)))?;
    for quiz in QUIZZES {
        let key = format!("F-Q.{}", quiz.name);
        f.add(&format!("{key}.line"), quiz.line)?;
        f.add(&format!("{key}.seconds"), quiz.seconds)?;
        let answer = match quiz.ask {
            Ask::Regions => circle::regions(&circle::drawn(shown::POINTS)).to_string(),
            Ask::NextLayer => layers::layer(shown::LAYERS + 1).len().to_string(),
            Ask::Rectangle => format!("{}x{}", shown::STAIRS, shown::STAIRS + 1),
            Ask::CutSlabs => cubes::layer(4)
                .iter()
                .filter(|p| p.part == cubes::Part::HalfRight)
                .count()
                .to_string(),
            Ask::Stacks => stacks::TURNS.len().to_string(),
            Ask::Share => q(1, 3).to_string(),
            Ask::Kinds => list(blocks::kinds(3)),
            Ask::Tilted => {
                let (a, b) = shown::LEGS;
                q(i128::from(pythagoras::area2(&pythagoras::tilted(a, b))), 2).to_string()
            }
            Ask::Slopes => {
                let first = lie::first();
                let (big, small) = (lie::slope(&first.big), lie::slope(&first.small));
                (if small > big { "small" } else { "big" }).to_owned()
            }
        };
        f.add(&format!("{key}.answer"), answer)?;
    }
    Ok(())
}

/// F-X: the cited numbers (years and pages), as the fact sheet gives them.
fn sources(f: &mut Facts) -> Result<(), String> {
    f.add("F-X.moser", format!("{},{}", cited::MOSER.0, cited::MOSER.1))?;
    f.add(
        "F-X.nicomachus",
        format!("{},{}", cited::NICOMACHUS.0, cited::NICOMACHUS.1),
    )?;
    f.add(
        "F-X.aryabhata",
        format!("{},{}", cited::ARYABHATA.0, cited::ARYABHATA.1),
    )?;
    f.add("F-X.al_karaji", cited::AL_KARAJI)?;
    f.add("F-X.fry", format!("{},{}", cited::FRY.0, cited::FRY.1))?;
    f.add("F-X.siu", format!("{},{}", cited::SIU.0, cited::SIU.1))?;
    f.add("F-X.page", format!("{},{}", cited::PAGE.0, cited::PAGE.1))?;
    f.add(
        "F-X.ajose_nelsen",
        format!("{},{}", cited::AJOSE_NELSEN.0, cited::AJOSE_NELSEN.1),
    )?;
    f.add("F-X.euclid", cited::EUCLID)?;
    f.add("F-X.curry", cited::CURRY)?;
    f.add(
        "F-X.gardner",
        format!("{},{},{}", cited::GARDNER.0, cited::GARDNER.1, cited::GARDNER.2),
    )?;
    f.add("F-X.nelsen", format!("{},{}", cited::NELSEN.0, cited::NELSEN.1))?;
    // the spans the script counts in words
    f.add("F-X.al_karaji.after_nicomachus", cited::AL_KARAJI - cited::NICOMACHUS.0)?;
    f.add("F-X.gardner.after_curry", cited::GARDNER.0 - cited::CURRY)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pin_holds() {
        facts().unwrap().self_check().unwrap();
    }

    /// Narration truth: every number the script speaks, as the words that
    /// speak it, against the fact it rests on. A rewording that drops the
    /// words, or a model change that moves the number, fails here.
    #[test]
    fn the_script_s_spoken_numbers() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let f = facts().unwrap();
        let rows: &[(&str, &str, &str, &str)] = &[
            (
                "02_2",
                "One, three, six, ten and fifteen",
                "F-C.cube_roots",
                "1,3,6,10,15",
            ),
            ("02_4", "Five points, sixteen", "F-C.regions", "1,2,4,8,16,31"),
            ("02_6", "Thirty-one", "F-Q.regions.answer", "31"),
            ("02_6", "said thirty-two", "F-C.doubling", "1,2,4,8,16,32"),
            ("03_1", "three more tiles", "F-O.layers", "1,3,5,7,9"),
            ("03_2", "five. Then seven. Then nine", "F-O.layers", "1,3,5,7,9"),
            ("03_4", "Eleven", "F-Q.next_layer.answer", "11"),
            ("03_5", "is five times five", "F-O.sum", "25"),
            ("03_5", "the hundredth layer", "F-O.hundredth", "199"),
            ("03_5", "one hundred and ninety-nine tiles", "F-O.hundredth", "199"),
            ("03_5", "ten thousand", "F-O.sum.hundred", "10000"),
            ("04_1", "fifteen tiles", "F-T.count", "15"),
            ("04_4", "Five wide and six tall", "F-Q.rectangle.answer", "5x6"),
            ("04_4", "Thirty tiles", "F-T.rectangle.cells", "30"),
            ("04_5", "five thousand and fifty", "F-T.count.hundred", "5050"),
            ("05_1", "fifteen slabs", "F-K.slabs", "15"),
            ("05_3", "six on a side", "F-K.sides", "1,3,6,10,15"),
            ("05_5", "Just one", "F-Q.cut_slabs.answer", "1"),
            ("05_5", "six long", "F-K.arm.4", "4x6"),
            ("05_5", "one and a half slabs", "F-K.arm.4.slabs", "3/2"),
            ("05_6", "fifteen on a side", "F-K.side", "15"),
            ("05_7", "two hundred and twenty-five tiles", "F-K.unit_squares", "225"),
            ("06_2", "six wide and fifteen long", "F-K.arm.6", "6x15"),
            ("06_2", "two and a half sixes", "F-K.arm.6.slabs", "5/2"),
            ("06_2", "twenty-one on a side", "F-K.sixth.side", "21"),
            ("06_6", "the year one thousand", "F-X.al_karaji", "1000"),
            ("06_6", "nineteen eighty-five", "F-X.fry", "1985,11"),
            ("07_1", "One, four, nine, sixteen", "F-S.layers", "16,9,4,1"),
            ("07_1", "Thirty little cubes", "F-S.stack", "30"),
            ("07_3", "Three.", "F-Q.stacks.answer", "3"),
            ("07_4", "Four slabs of twenty cubes", "F-S.top.layer", "20"),
            ("07_4", "a top of ten", "F-S.top", "10"),
            ("07_5", "Four wide, five deep and four tall", "F-S.block", "4x5x4"),
            ("07_5", "ten cubes", "F-S.top", "10"),
            ("07_5", "exactly half", "F-S.top.layer", "20"),
            ("08_3", "four and a half tall", "F-S.box", "4x5x9/2"),
            ("08_3", "makes ninety", "F-S.box.volume", "90"),
            ("08_3", "Three stacks of thirty", "F-S.three_stacks", "90"),
            ("08_5", "nineteen eighty-four", "F-X.siu", "1984,92"),
            ("09_2", "five by five slab", "F-S.grown.slab", "25"),
            ("09_3", "three new slabs of five by five", "F-S.grown.slab", "25"),
            ("09_4", "three single cubes", "F-S.one.cubes", "3"),
            (
                "10_2",
                "thirty-one out of thirty-two",
                "F-H.filled",
                "1/2,3/4,7/8,15/16,31/32",
            ),
            ("10_2", "one thirty-second", "F-H.gap", "1/32"),
            ("10_4", "ten pieces pass it", "F-H.pieces_to.thousandth", "10"),
            ("10_4", "twenty pieces pass it", "F-H.pieces_to.millionth", "20"),
            (
                "11_3",
                "A quarter, plus a sixteenth, plus a sixty-fourth",
                "F-R.pieces",
                "1/4,1/16,1/64",
            ),
            ("11_4", "One third", "F-Q.share.answer", "1/3"),
            ("11_4", "three equal squares", "F-R.shares", "3"),
            ("11_6", "nineteen ninety-four", "F-X.ajose_nelsen", "1994,230"),
            ("12_1", "Three and two", "F-B.parts", "3,2"),
            ("12_2", "Nine, six, six and four", "F-B.square.each", "9,6,4"),
            ("12_2", "make twenty-five", "F-B.square.total", "25"),
            ("12_5", "Eight blocks", "F-B.cube.blocks", "8"),
            ("12_5", "One, three, three, one", "F-B.cube.kinds", "1,3,3,1"),
            ("12_6", "one hundred and twenty-five", "F-B.cube.total", "125"),
            ("12_7", "One, four, six, four, one", "F-B.fourth.kinds", "1,4,6,4,1"),
            ("13_1", "three and four", "F-P.legs", "3,4"),
            ("13_5", "Nine plus sixteen", "F-P.squares", "9,16"),
            ("13_5", "Twenty-five", "F-Q.tilted.answer", "25"),
            ("13_5", "the long side is five", "F-P.long_side", "5"),
            ("13_5", "forty-nine", "F-P.big", "49"),
            ("13_5", "twenty-four", "F-P.triangles", "24"),
            ("13_6", "Shorter sides two and five", "F-P.other.legs", "2,5"),
            ("13_6", "twenty-nine", "F-P.other.tilted", "29"),
            ("13_6", "no whole long side", "F-P.other.whole_side", "none"),
            ("14_1", "thirteen wide and five tall", "F-L.frame", "13x5"),
            ("14_2", "one tile is empty", "F-L.holes.second", "1"),
            ("14_3", "thirty-two", "F-L.pieces.total", "32"),
            ("14_3", "thirty-three", "F-L.outline.second", "33"),
            ("14_5", "two in five", "F-L.slopes", "3/8,2/5"),
            ("14_5", "three in eight", "F-L.slopes", "3/8,2/5"),
            ("14_5", "The small one", "F-Q.slopes.answer", "small"),
            ("14_6", "exactly one tile", "F-L.sliver", "1"),
            ("14_7", "nineteen fifty-three", "F-X.curry", "1953"),
            ("14_7", "three years later", "F-X.gardner.after_curry", "3"),
            ("15_5", "the cubes up to forty", "F-K.tiled.through", "40"),
            ("15_5", "twenty-four levels", "F-S.filled.through", "24"),
            ("15_5", "up to a thousand", "F-K.sum.through", "1000"),
        ];
        let mut bad = Vec::new();
        for (line, words, key, value) in rows {
            match script.text(line) {
                Some(text) if text.contains(words) => {}
                Some(_) => bad.push(format!("{line}: the script no longer says {words:?}")),
                None => bad.push(format!("{line}: no such line")),
            }
            if f.get(key) != Some(value) {
                bad.push(format!(
                    "{line}: {key} is {:?}, the words {words:?} say {value}",
                    f.get(key)
                ));
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }

    /// The fact sheet is current: it has a row for every fact, with the
    /// value the video computes.
    #[test]
    fn the_fact_sheet_lists_every_fact() {
        let sheet = include_str!("../docs/fact-sheet.md");
        let printed = facts().unwrap().print();
        let missing: Vec<String> = printed
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .filter(|(k, v)| !sheet.contains(&format!("| `{k}` | `{v}` |")))
            .map(|(k, v)| format!("{k} = {v}"))
            .collect();
        assert!(
            missing.is_empty(),
            "docs/fact-sheet.md lacks or misstates: {}",
            missing.join("; ")
        );
    }

    /// The owner's four pictures, as his image shows them.
    #[test]
    fn the_owner_s_pictures() {
        let f = facts().unwrap();
        // the cubes 1 to 5 tile a 15 by 15 square
        assert_eq!(f.get("F-K.side"), Some("15"));
        assert_eq!(f.get("F-K.unit_squares"), Some("225"));
        // one of three equal shares at every scale
        assert_eq!(f.get("F-R.limit"), Some("1/3"));
        assert_eq!(f.get("F-R.shares"), Some("3"));
        // a^3 + 3a^2 b + 3ab^2 + b^3, and 1, 4, 6, 4, 1
        assert_eq!(f.get("F-B.cube.kinds"), Some("1,3,3,1"));
        assert_eq!(f.get("F-B.fourth.kinds"), Some("1,4,6,4,1"));
        assert_eq!(f.get("F-B.cube.total"), f.get("F-B.cube.power"));
        assert_eq!(f.get("F-B.fourth.total"), f.get("F-B.fourth.power"));
        // Siu: n by n + 1 by n + 1/2
        assert_eq!(f.get("F-S.box"), Some("4x5x9/2"));
        assert_eq!(f.get("F-S.box.volume"), f.get("F-S.three_stacks"));
    }

    /// 07_2 asks for the fewest stacks: three, and the box they make is
    /// the one 08_3 shows.
    #[test]
    fn the_fewest_stacks_make_siu_s_box() {
        let f = facts().unwrap();
        assert_eq!(f.get("F-S.fewest"), Some("3"));
        assert_eq!(f.get("F-S.fewest"), f.get("F-Q.stacks.answer"));
        assert_eq!(f.get("F-S.fewest.box"), f.get("F-S.box"));
        // 02_6: evenly spaced points lose one region to the three lines meeting
        assert_eq!(f.get("F-C.evenly_spaced.regions"), Some("30"));
        // 06_2: five cubes and the sixth are the square of 21
        assert_eq!(f.get("F-K.sixth.total"), Some("441"));
        // 14_6: the two slopes differ by one over the product of 5 and 8
        assert_eq!(f.get("F-L.slopes.differ"), Some("1/40"));
    }

    /// Brute force against the closed forms, on sizes small enough to count.
    #[test]
    fn closed_forms_agree_with_the_pictures() {
        for n in 1..=30i64 {
            let squares = stacks::standard(n).len() as i64;
            assert_eq!(6 * squares, n * (n + 1) * (2 * n + 1));
            assert_eq!(
                cubes::square(n).iter().map(|p| p.rect.area()).sum::<i64>(),
                cubes::side(n).pow(2)
            );
            assert_eq!(cubes::side(n), layers::staircase(n).len() as i64);
        }
    }

    /// The edges of the always and never sentences the pictures stand for.
    #[test]
    fn the_edges_of_every_always() {
        // "every cube": the first cube is one slab and needs no arm; the second is the first to be cut
        assert_eq!(cubes::layer(1).len(), 1);
        assert_eq!(
            cubes::layer(2).iter().filter(|p| p.part != cubes::Part::Whole).count(),
            2
        );
        // "never reaches one": every partial sum of the halves is below one
        let mut filled = Q::ZERO;
        for k in 1..=60 {
            filled = filled + series::area(series::half(k));
            assert!(filled < Q::ONE);
        }
        // "every right triangle": legs that are equal, and legs far apart
        for (a, b) in [(1, 1), (1, 60), (60, 1), (7, 7)] {
            assert!(pythagoras::is_square(&pythagoras::tilted(a, b)));
        }
        // "the doubling holds up to five points and no further"
        for n in 1..=7usize {
            let doubled = circle::regions(&circle::drawn(n)) == 1 << (n - 1);
            assert_eq!(doubled, n <= 5, "n = {n}");
        }
    }
}
