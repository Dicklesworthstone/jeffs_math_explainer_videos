#!/usr/bin/env python3
"""Facts oracle for "Two Shadows Can Lie" (discrete_tomography).

Run from this folder (`./x discrete_tomography facts-check` does):

    python3 -I -B oracle.py

stdout: one `KEY<TAB>VALUE` line per fact, the same keys `tomography facts`
prints (src/facts.rs), in the rows of docs/fact-sheet.md: DT-F (computed),
DT-S (a source's literal), DT-X (the edges of the always/never sentences,
on every 4x4 picture).

stderr: cross-checks (lines starting with "#") and a summary; exit 1 if any
cross-check failed.

How the values are obtained, independently of the Rust model (which fills
pictures row by row, finds switches as pairs of rows times pairs of columns
and pushes flow along shortest augmenting paths):
  * a picture is a frozenset of its filled (row, column) squares;
  * every picture that fits a pair of shadows is generated column by column,
    each column any choice of rows with room left, and the number of them is
    recounted by a memoized count over the columns;
  * a switch is found from two filled squares in different rows and columns
    whose other two corners are empty;
  * the largest flow through Gale's network is its smallest cut, in closed
    form: with a set W of columns on the sink's side, the cheapest cut costs
    sum_i min(r_i, |W|) plus the counts of the other columns; a depth-first
    augmenting-path flow cross-checks it, and the cut chapter 01 names is
    the smallest source side among all minimum cuts, found by listing them;
  * feasibility by Gale's own test (the dual sequence of the row counts,
    D. Gale 1957, p. 1080), and Gale's n-step construction (largest column
    first, its squares from the rows with the most left) checked to fit;
  * every 4x4 picture by brute force over its 2^16 masks for the edge facts.
"""
import re
import sys
from collections import deque
from functools import lru_cache
from itertools import combinations, product
from pathlib import Path

HERE = Path(__file__).resolve().parent

_EMITTED = {}
_FAILED = []
_CHECKS = [0]

FACE = ("01010", "01010", "00000", "10001", "01110")
TREE = ("00100", "01110", "11111", "00100", "00100")
# the opening switch: its two rows, then its two columns
OPENING = ((0, 3), (1, 4))
NO_FIT_COLS = (5, 1, 1, 1, 1)
# H. J. Ryser, "Combinatorial properties of matrices of zeros and ones",
# Canadian Journal of Mathematics 9 (1957) 371-377, Theorem 3.1
RYSER_YEAR = 1957


def emit(key, value):
    if key in _EMITTED:
        raise SystemExit(f"oracle: {key} emitted twice")
    _EMITTED[key] = str(value)
    print(f"{key}\t{value}")


def check(name, ok):
    _CHECKS[0] += 1
    if not ok:
        _FAILED.append(name)
    print(f"# {'ok  ' if ok else 'FAIL'} {name}", file=sys.stderr)


def csv(values):
    return ",".join(str(v) for v in values)


def flag(ok):
    return "true" if ok else "false"


# ------------------------------------------------------------------ pictures


def parse(rows):
    return frozenset((r, c) for r, row in enumerate(rows) for c, ch in enumerate(row) if ch == "1")


def pattern(cells, m, n):
    return "/".join("".join("1" if (r, c) in cells else "0" for c in range(n)) for r in range(m))


def shadows(cells, m, n):
    rows, cols = [0] * m, [0] * n
    for r, c in cells:
        rows[r] += 1
        cols[c] += 1
    return tuple(rows), tuple(cols)


def switches(cells):
    """Every switch, as (rows, columns): two filled squares in different rows
    and columns whose other two corners are empty."""
    found = set()
    for (a, b), (c, d) in combinations(sorted(cells), 2):
        if a != c and b != d and (a, d) not in cells and (c, b) not in cells:
            found.add(((min(a, c), max(a, c)), (min(b, d), max(b, d))))
    return found


def switched(cells, sw):
    (r0, r1), (c0, c1) = sw
    return cells ^ {(r0, c0), (r0, c1), (r1, c0), (r1, c1)}


def distances(start):
    """Fewest switches from `start` to every picture a chain of them reaches."""
    dist = {start: 0}
    queue = deque([start])
    while queue:
        p = queue.popleft()
        for sw in switches(p):
            q = switched(p, sw)
            if q not in dist:
                dist[q] = dist[p] + 1
                queue.append(q)
    return dist


# ---------------------------------------------------------------- shadows


def fits(rows, cols):
    """Every picture with these shadows, column by column."""
    m, n = len(rows), len(cols)
    out = []

    def column(j, room, cells):
        if j == n:
            if not any(room):
                out.append(frozenset(cells))
            return
        for chosen in combinations([i for i in range(m) if room[i] > 0], cols[j]):
            for i in chosen:
                room[i] -= 1
            column(j + 1, room, cells + [(i, j) for i in chosen])
            for i in chosen:
                room[i] += 1

    if sum(rows) == sum(cols):
        column(0, list(rows), [])
    return out


@lru_cache(maxsize=None)
def count_fits(rows, cols):
    """How many pictures fit, counted over the columns; the rows' remaining
    counts are kept sorted (the count does not depend on their order)."""
    if not cols:
        return 1 if not any(rows) else 0
    total = 0
    for chosen in combinations(range(len(rows)), cols[0]):
        if all(rows[i] > 0 for i in chosen):
            rest = list(rows)
            for i in chosen:
                rest[i] -= 1
            total += count_fits(tuple(sorted(rest)), cols[1:])
    return total


def dual(rows, length):
    """Gale's dual sequence: s_j = how many rows hold at least j squares."""
    return [sum(1 for r in rows if r >= j) for j in range(1, length + 1)]


def gale_feasible(rows, cols):
    """Gale's test: totals agree, and with the columns sorted from largest,
    every k of them ask for at most the first k terms of the dual sequence."""
    if sum(rows) != sum(cols):
        return False
    b = sorted(cols, reverse=True)
    s = dual(rows, len(b))
    return all(sum(b[:k]) <= sum(s[:k]) for k in range(1, len(b) + 1))


def gale_build(rows, cols):
    """Gale's n-step method: the largest column first takes its squares from
    the rows with the most left, then the next. None if it gets stuck."""
    left = list(rows)
    cells = set()
    for j in sorted(range(len(cols)), key=lambda j: -cols[j]):
        takers = sorted(range(len(left)), key=lambda i: -left[i])[: cols[j]]
        if len(takers) < cols[j] or any(left[i] == 0 for i in takers):
            return None
        for i in takers:
            left[i] -= 1
            cells.add((i, j))
    return None if any(left) else frozenset(cells)


def cut_value(rows, cols, sink_cols):
    """The cheapest cut with the columns `sink_cols` on the sink's side."""
    k = len(sink_cols)
    return sum(min(r, k) for r in rows) + sum(c for j, c in enumerate(cols) if j not in sink_cols)


def max_flow_cut(rows, cols):
    n = len(cols)
    return min(cut_value(rows, cols, set(w)) for k in range(n + 1) for w in combinations(range(n), k))


def max_flow_dfs(rows, cols):
    """Unit pushes along depth-first augmenting paths, residual capacities in
    dictionaries (a second, unrelated implementation)."""
    cap = {}

    def edge(u, w, c):
        cap.setdefault(u, {})
        cap[u][w] = cap[u].get(w, 0) + c
        cap.setdefault(w, {}).setdefault(u, 0)

    for i, r in enumerate(rows):
        edge("s", ("r", i), r)
        for j in range(len(cols)):
            edge(("r", i), ("c", j), 1)
    for j, c in enumerate(cols):
        edge(("c", j), "t", c)
    flow = 0
    while True:
        seen = set()

        def push(u):
            if u == "t":
                return True
            seen.add(u)
            for w, c in cap[u].items():
                if c > 0 and w not in seen and push(w):
                    cap[u][w] -= 1
                    cap[w][u] += 1
                    return True
            return False

        if not push("s"):
            return flow
        flow += 1


def smallest_cut_sink_cols(rows, cols):
    """The columns on the sink's side of the minimum cut with the smallest
    source side (the intersection of every minimum cut's source side): list
    every cut (rows X and columns Z with the source) and keep the cheapest."""
    m, n = len(rows), len(cols)
    best, sides = None, []
    for xs in product((0, 1), repeat=m):
        for zs in product((0, 1), repeat=n):
            sink_cols = [j for j in range(n) if not zs[j]]
            value = sum(r for i, r in enumerate(rows) if not xs[i])
            value += sum(xs) * len(sink_cols)
            value += sum(c for j, c in enumerate(cols) if zs[j])
            if best is None or value < best:
                best, sides = value, []
            if value == best:
                sides.append(({i for i in range(m) if xs[i]}, {j for j in range(n) if zs[j]}))
    source_cols = set.intersection(*(z for _, z in sides))
    return best, [j for j in range(n) if j not in source_cols]


def bottleneck(rows, cols):
    """The smallest set of columns asking for more than the dual sequence
    lets the rows give (the first such set of that size, in order)."""
    s = dual(rows, len(cols))
    for k in range(1, len(cols) + 1):
        for w in combinations(range(len(cols)), k):
            if sum(cols[j] for j in w) > sum(s[:k]):
                return list(w)
    return None


# ------------------------------------------------------------------- facts


def facts():
    m, n = len(FACE), len(FACE[0])
    face = parse(FACE)
    rows, cols = shadows(face, m, n)
    emit("DT-F1.size", csv((m, n)))
    emit("DT-F1.directions", len((rows, cols)))
    emit("DT-F1.face", pattern(face, m, n))
    emit("DT-F1.face.rows", csv(rows))
    emit("DT-F1.face.cols", csv(cols))
    emit("DT-F1.face.total", len(face))
    emit("DT-F1.face.empty_rows", csv(i for i, r in enumerate(rows) if r == 0))
    emit("DT-F1.face.gale", flag(gale_feasible(rows, cols)))

    sw_rows, sw_cols = OPENING
    twin = switched(face, OPENING)
    check("the opening switch is a switch of the face", OPENING in switches(face))
    emit("DT-F2.switch", f"{csv(sw_rows)};{csv(sw_cols)}")
    emit("DT-F2.twin", pattern(twin, m, n))
    emit("DT-F2.twin.rows", csv(shadows(twin, m, n)[0]))
    emit("DT-F2.twin.cols", csv(shadows(twin, m, n)[1]))
    differ = sorted(face ^ twin)
    emit("DT-F2.differ", len(differ))
    emit("DT-F2.differ.cells", ";".join(csv(rc) for rc in differ))
    # what each of the switch's rows and columns loses and gains
    lost = face - twin
    gained = twin - face
    per_line = [
        (sum(1 for r, _ in lost if r == i), sum(1 for r, _ in gained if r == i)) for i in sw_rows
    ] + [(sum(1 for _, c in lost if c == j), sum(1 for _, c in gained if c == j)) for j in sw_cols]
    check("each of the switch's rows and columns loses as many as it gains", len(set(per_line)) == 1)
    emit("DT-F2.loses", per_line[0][0])
    emit("DT-F2.gains", per_line[0][1])

    every = fits(rows, cols)
    from_face = distances(face)
    from_twin = distances(twin)
    farthest = max(from_face.values())
    emit("DT-F3.fits", len(every))
    emit("DT-F3.face.switches", len(switches(face)))
    emit("DT-F3.rings", csv(sum(1 for d in from_face.values() if d == k) for k in range(farthest + 1)))
    emit("DT-F3.farthest", farthest)
    emit("DT-F3.connected", flag(set(from_face) == set(every)))
    end = min(
        (p for p, d in from_face.items() if d == farthest and from_twin.get(p) == farthest - 1),
        key=lambda p: pattern(p, m, n),
    )
    emit("DT-F3.walk.end", pattern(end, m, n))
    emit("DT-F3.walk.steps", 1 + from_twin[end])
    check("the count over the columns agrees", count_fits(tuple(sorted(rows)), cols) == len(every))
    check("every picture that fits has a switch", all(switches(p) for p in every))
    check(
        "every two of the 64 are joined by at most the farthest walk",
        all(max(distances(p).values()) <= farthest for p in every),
    )

    no_cols = NO_FIT_COLS
    flow = max_flow_cut(rows, no_cols)
    cut_flow, cut_cols = smallest_cut_sink_cols(rows, no_cols)
    emit("DT-F4.cols", csv(no_cols))
    emit("DT-F4.totals", csv((sum(rows), sum(no_cols))))
    emit("DT-F4.fits", len(fits(rows, no_cols)))
    emit("DT-F4.gale", flag(gale_feasible(rows, no_cols)))
    emit("DT-F4.flow", flow)
    emit("DT-F4.cut", csv(cut_cols))
    neck = bottleneck(rows, no_cols)
    emit("DT-F4.bottleneck", csv(neck))
    emit("DT-F4.bottleneck.asks", sum(no_cols[j] for j in neck))
    emit("DT-F4.bottleneck.can_give", sum(dual(rows, len(no_cols))[: len(neck)]))
    check("the depth-first flow agrees (no fit)", max_flow_dfs(rows, no_cols) == flow)
    check("the listed cuts agree with the closed form (no fit)", cut_flow == flow)
    check("Gale's construction gets stuck (no fit)", gale_build(rows, no_cols) is None)
    check("the count over the columns agrees (no fit)", count_fits(tuple(sorted(rows)), no_cols) == 0)

    full = max_flow_cut(rows, cols)
    emit("DT-F5.face.flow", full)
    built = gale_build(rows, cols)
    emit("DT-F5.face.built", flag(built is not None and shadows(built, m, n) == (rows, cols)))
    check("the depth-first flow agrees (face)", max_flow_dfs(rows, cols) == full)
    check("Gale's construction is one of the 64", built in set(every))

    tree = parse(TREE)
    t_rows, t_cols = shadows(tree, m, n)
    emit("DT-F6.tree", pattern(tree, m, n))
    emit("DT-F6.tree.rows", csv(t_rows))
    emit("DT-F6.tree.cols", csv(t_cols))
    emit("DT-F6.tree.fits", len(fits(t_rows, t_cols)))
    emit("DT-F6.tree.switches", len(switches(tree)))

    # the questions on screen at 01_11, read from the scene's source
    source = (HERE / "src/chapters/same_shadows.rs").read_text(encoding="utf-8")
    found = re.search(r"pub const QUESTIONS: \[&str; \d+\] = \[(.*?)\];", source, re.S)
    questions = re.findall(r'"([^"]*)"', found.group(1)) if found else []
    check("the questions on screen are questions", bool(questions) and all(q.endswith("?") for q in questions))
    emit("DT-F7.questions", len(questions))
    emit("DT-S1.ryser", RYSER_YEAR)
    return rows, cols


def edges():
    """Every 4x4 picture: switches keep both shadows, every pair of shadows'
    pictures is joined by switches, and a picture is alone with its shadows
    exactly when it has no switch."""
    m = n = 4
    pics = [
        frozenset((i // n, i % n) for i in range(m * n) if (mask >> i) & 1) for mask in range(1 << (m * n))
    ]
    groups = {}
    for p in pics:
        groups.setdefault(shadows(p, m, n), []).append(p)
    pairs = failed = 0
    for p in pics:
        for sw in switches(p):
            q = switched(p, sw)
            pairs += 1
            failed += shadows(q, m, n) != shadows(p, m, n) or len(p ^ q) != 4
    emit("DT-X1.pairs", pairs)
    emit("DT-X1.failed", failed)
    emit("DT-X2.shadow_pairs", len(groups))
    emit("DT-X2.connected", sum(1 for g in groups.values() if len(distances(g[0])) == len(g)))
    alone = [len(groups[shadows(p, m, n)]) == 1 for p in pics]
    lonely = [not switches(p) for p in pics]
    emit("DT-X3.alone", sum(alone))
    emit("DT-X3.no_switch", sum(lonely))
    emit("DT-X3.disagree", sum(a != b for a, b in zip(alone, lonely)))

    # every pair of 4x4 shadows (counts 0 to 4): Gale's test, Gale's
    # construction and the closed-form flow each pick out exactly the
    # shadows some picture casts
    cast = set(groups)
    wrong = 0
    for r in product(range(5), repeat=4):
        for c in product(range(5), repeat=4):
            feasible = (r, c) in cast
            if gale_feasible(r, c) != feasible:
                wrong += 1
            elif sum(r) == sum(c) and (max_flow_cut(r, c) == sum(r)) != feasible:
                wrong += 1
            elif feasible and gale_build(r, c) not in set(groups[(r, c)]):
                wrong += 1
    check(f"4x4: Gale's test, Gale's construction and the flow agree with brute force on all {5 ** 8} pairs", wrong == 0)

    # the slate's toy enumeration (WIZARD_IDEAS_CODEX.md, 3x3), recomputed
    small = {}
    for mask in range(1 << 9):
        p = frozenset((i // 3, i % 3) for i in range(9) if (mask >> i) & 1)
        small.setdefault(shadows(p, 3, 3), []).append(p)
    sizes = [len(g) for g in small.values()]
    check(
        "3x3: 512 pictures, 328 pairs of shadows, 98 shared, largest 6 (the slate's toy check)",
        (sum(sizes), len(small), sum(1 for s in sizes if s > 1), max(sizes)) == (512, 328, 98, 6),
    )


def main():
    facts()
    edges()
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
