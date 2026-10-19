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


def smallest_cut(rows, cols):
    """The minimum cut with the smallest source side (the intersection of
    every minimum cut's source side): list every cut (rows X and columns Z
    with the source), keep the cheapest. Returns its value and the rows and
    columns on the source's side."""
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
    source_rows = set.intersection(*(x for x, _ in sides))
    source_cols = set.intersection(*(z for _, z in sides))
    return best, sorted(source_rows), sorted(source_cols)


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
    cut_flow, _, source_cols = smallest_cut(rows, no_cols)
    cut_cols = [j for j in range(len(no_cols)) if j not in source_cols]
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
    pairs = failed = diagonal_kept = 0
    for p in pics:
        here = diagonal(p, m, n)
        for sw in switches(p):
            q = switched(p, sw)
            pairs += 1
            failed += shadows(q, m, n) != shadows(p, m, n) or len(p ^ q) != 4
            diagonal_kept += diagonal(q, m, n) == here
    emit("DT-X1.pairs", pairs)
    emit("DT-X1.failed", failed)
    emit("DT-X2.shadow_pairs", len(groups))
    emit("DT-X2.connected", sum(1 for g in groups.values() if len(distances(g[0])) == len(g)))
    alone = [len(groups[shadows(p, m, n)]) == 1 for p in pics]
    lonely = [not switches(p) for p in pics]
    emit("DT-X3.alone", sum(alone))
    emit("DT-X3.no_switch", sum(lonely))
    emit("DT-X3.disagree", sum(a != b for a, b in zip(alone, lonely)))
    # the water's picture settles uniqueness on every pair of shadows some
    # picture casts: it fits, and it has no switch exactly when it is alone
    wrong = 0
    for (r, c), group in groups.items():
        got = water_states(r, c, water_trace(r, c))
        built = got[-1][0] if got else frozenset()
        if built not in set(group) or (not switches(built)) != (len(group) == 1):
            wrong += 1
    emit("DT-X4.disagree", wrong)
    # a picture's rows nest exactly when it has no switch
    nested = [rows_nest(p, m) for p in pics]
    emit("DT-X5.nested", sum(nested))
    emit("DT-X5.disagree", sum(a != b for a, b in zip(nested, lonely)))
    # no switch keeps the diagonal shadow
    emit("DT-X6.kept", diagonal_kept)

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


# ------------------------------------------------------- chapter 02, water

# a square's pipe carries at most one unit: a square is filled or empty
SQUARE = 1
# L. R. Ford Jr. and D. R. Fulkerson, "A simple algorithm for finding maximal
# network flows and an application to the Hitchcock problem", Canadian Journal
# of Mathematics 9 (1957) 210-218; D. Gale, Pacific J. Math. 7 (1957) 1073-1082
FORD_FULKERSON_YEAR = 1957
GALE_YEAR = 1957


def water_trace(rows, cols):
    """The method's paths, rebuilt with room kept per directed edge in a
    dictionary: each path is the first one found breadth first, scanning the
    source, the rows top to bottom, the columns left to right, then the sink;
    it sends as much as its tightest edge allows. Returns (nodes, units) per
    path, nodes as "s", ("r", i), ("c", j), "t"."""
    m, n = len(rows), len(cols)
    order = ["s"] + [("r", i) for i in range(m)] + [("c", j) for j in range(n)] + ["t"]
    room = {}

    def edge(u, w, c):
        room[(u, w)] = room.get((u, w), 0) + c
        room.setdefault((w, u), 0)

    for i, r in enumerate(rows):
        edge("s", ("r", i), r)
        for j in range(n):
            edge(("r", i), ("c", j), SQUARE)
    for j, c in enumerate(cols):
        edge(("c", j), "t", c)
    paths = []
    while True:
        parent = {"s": None}
        queue = deque(["s"])
        while queue:
            u = queue.popleft()
            for w in order:
                if w not in parent and room.get((u, w), 0) > 0:
                    parent[w] = u
                    queue.append(w)
        if "t" not in parent:
            return paths
        nodes = ["t"]
        while nodes[-1] != "s":
            nodes.append(parent[nodes[-1]])
        nodes.reverse()
        units = min(room[e] for e in zip(nodes, nodes[1:]))
        for u, w in zip(nodes, nodes[1:]):
            room[(u, w)] -= units
            room[(w, u)] += units
        paths.append((nodes, units))


def path_text(nodes):
    return "".join(f"{kind}{k}" for kind, k in nodes[1:-1])


def water_states(rows, cols, paths, allow_repeats=False):
    """Apply the paths to an empty grid, keeping each square's water and
    each tap's and drain's count; raise ValueError on any path that is not
    source, row, column, (row, column)..., sink, that visits a row or a column
    twice, sends other than one unit, wets a wet square, takes back from a dry
    one, overdraws a tap or overfills a drain. Those rules keep every row and
    column balanced; the balance is asserted after each path as an invariant
    they imply, not as a separate way a path can fail."""
    m, n = len(rows), len(cols)
    flow = {(i, j): 0 for i in range(m) for j in range(n)}
    sent, drained = [0] * m, [0] * n
    states = []
    for k, (nodes, units) in enumerate(paths):
        if units != SQUARE:
            raise ValueError(f"path {k} sends {units} units")
        if len(nodes) < 4 or nodes[0] != "s" or nodes[-1] != "t":
            raise ValueError(f"path {k} does not run from the source to the sink")
        inner = nodes[1:-1]
        kinds = [node[0] for node in inner]
        if kinds != ["r", "c"] * (len(inner) // 2) or len(inner) % 2:
            raise ValueError(f"path {k} is not rows and columns in turn")
        if any(not 0 <= (node[1]) < (m if node[0] == "r" else n) for node in inner):
            raise ValueError(f"path {k} leaves the grid")
        if not allow_repeats and len(set(inner)) != len(inner):
            raise ValueError(f"path {k} visits a row or a column twice")
        tap, drain = inner[0][1], inner[-1][1]
        if sent[tap] + 1 > rows[tap]:
            raise ValueError(f"path {k}: row {tap}'s tap is empty")
        sent[tap] += 1
        for u, w in zip(inner, inner[1:]):
            if u[0] == "r":
                if flow[(u[1], w[1])] != 0:
                    raise ValueError(f"path {k} wets the wet square {(u[1], w[1])}")
                flow[(u[1], w[1])] = 1
            else:
                if flow[(w[1], u[1])] != 1:
                    raise ValueError(f"path {k} takes back from the dry square {(w[1], u[1])}")
                flow[(w[1], u[1])] = 0
        if drained[drain] + 1 > cols[drain]:
            raise ValueError(f"path {k}: column {drain}'s drain is full")
        drained[drain] += 1
        for i in range(m):
            if sent[i] != sum(flow[(i, j)] for j in range(n)):
                raise ValueError(f"path {k}: row {i} is unbalanced")
        for j in range(n):
            if drained[j] != sum(flow[(i, j)] for i in range(m)):
                raise ValueError(f"path {k}: column {j} is unbalanced")
        # (the two balance loops above are the invariant, asserted)
        wet = frozenset(sq for sq, f in flow.items() if f)
        states.append((wet, list(sent), list(drained)))
    return states


def refused(rows, cols, paths, allow_repeats=False):
    try:
        water_states(rows, cols, paths, allow_repeats)
    except ValueError:
        return True
    return False


def chapter_02(rows, cols, every, from_face):
    m, n = len(rows), len(cols)
    face = parse(FACE)
    paths = water_trace(rows, cols)
    states = water_states(rows, cols, paths)
    total = sum(rows)
    k = next(i for i, (nodes, _) in enumerate(paths) if len(nodes) > 4)
    before = states[k - 1]
    wet, sent, drained = before
    nodes = paths[k][0]
    (_, stuck), (_, into), (_, from_row), (_, onto) = nodes[1:-1]
    dry = [j for j in range(n) if (stuck, j) not in wet]
    forward = any(
        sent[i] < rows[i] and any((i, j) not in wet and drained[j] < cols[j] for j in range(n)) for i in range(m)
    )
    built = states[-1][0]
    emit("DT-F8.square", SQUARE)
    emit("DT-F8.paths", len(paths))
    emit("DT-F8.room", csv(sorted({units for _, units in paths})))
    emit("DT-F8.trace", ",".join(path_text(p) for p, _ in paths))
    emit("DT-F8.direct", sum(1 for p, _ in paths if len(p) == 4))
    emit("DT-F8.takebacks", sum(1 for p, _ in paths if len(p) > 4))
    emit("DT-F8.before.through", sum(drained))
    emit("DT-F8.before.wet", pattern(wet, m, n))
    emit("DT-F8.before.forward_room", flag(forward))
    emit("DT-F8.stuck.row", stuck)
    emit("DT-F8.stuck.left", rows[stuck] - sent[stuck])
    emit("DT-F8.stuck.dry_full", csv(dry))
    emit("DT-F8.takeback.column", into)
    emit("DT-F8.takeback.row", from_row)
    emit("DT-F8.takeback.to_column", onto)
    emit("DT-F8.takeback.to_needs", cols[onto] - drained[onto])
    emit("DT-F8.short", total - sum(drained))
    emit("DT-F8.through", sum(states[-1][2]))
    emit("DT-F8.built", pattern(built, m, n))
    emit("DT-F8.built.fits", flag(built in set(every)))
    emit("DT-F8.built.is_face", flag(built == face))
    emit("DT-F8.built.only_one", flag(len(every) == 1))
    emit("DT-F8.built.from_face", from_face[built])
    emit("DT-S2.ford_fulkerson", FORD_FULKERSON_YEAR)
    emit("DT-S3.gale", GALE_YEAR)

    check("chapter 02: exactly one path takes water back, and it is the last", k == len(paths) - 1)
    check("chapter 02: the stuck row's dry squares all lead to full drains", all(drained[j] == cols[j] for j in dry))
    check("chapter 02: the water taken back had been sent there by that row", (from_row, into) in wet)
    check("chapter 02: the flow reaches the smallest cut", sum(states[-1][2]) == max_flow_cut(rows, cols) == total)
    check("chapter 02: the water's picture casts the face's shadows", shadows(built, m, n) == (tuple(rows), tuple(cols)))
    check("chapter 02: the water's picture is the depth-first flow's size too", max_flow_dfs(rows, cols) == len(paths))

    # the checker refuses broken traces built from the real one
    first = paths[0]
    empty_row = rows.index(0)
    full_col = paths[0][0][2][1]
    broken = {
        "a unit wetting a wet square": [first, first],
        "a unit from an empty tap": [(["s", ("r", empty_row), ("c", 0), "t"], 1)],
        "a unit into a full drain": [first, (["s", ("r", 1), ("c", full_col), "t"], 1)],
        "taking back from a dry square": [(["s", ("r", 1), ("c", 1), ("r", 0), ("c", 3), "t"], 1)],
        "two units at once": [(first[0], 2)],
        "a path that skips its column": [(["s", ("r", 0), "t"], 1)],
        "the take-back path without its first eight": [paths[k]],
        # within every count, yet a walk through the top row twice
        "a cyclic walk s,r0,c0,r0,c1,t": [(["s", ("r", 0), ("c", 0), ("r", 0), ("c", 1), "t"], 1)],
    }
    check(
        "chapter 02: with the no-repeat rule off, the same checker accepts the cyclic walk",
        not refused(rows, cols, broken["a cyclic walk s,r0,c0,r0,c1,t"], allow_repeats=True),
    )
    for name, trace in broken.items():
        check(f"chapter 02: the checker refuses {name}", refused(rows, cols, trace))
    check("chapter 02: the checker accepts the real trace", not refused(rows, cols, paths))

    # the same method on every 3x3 pair of shadows with equal totals: each
    # trace replays within its capacities and reaches the smallest cut
    bad = 0
    for r in product(range(4), repeat=3):
        for c in product(range(4), repeat=3):
            if sum(r) != sum(c):
                continue
            trace = water_trace(r, c)
            try:
                got = water_states(r, c, trace)
            except ValueError:
                bad += 1
                continue
            if (sum(got[-1][2]) if got else 0) != max_flow_cut(r, c):
                bad += 1
    check("chapter 02: every 3x3 trace replays and reaches the smallest cut", bad == 0)


# ------------------------------------------------- chapter 03, the stop


def still_reaches(rows, cols, state):
    """The rows and columns the water can still reach after a flow, taking
    back included: a tap with water left leads into its row, a row through a
    dry square into that square's column, a column back through a wet square
    into that square's row. Returns them sorted."""
    wet, sent, _ = state
    m, n = len(rows), len(cols)
    seen_rows = {i for i in range(m) if sent[i] < rows[i]}
    seen_cols = set()
    queue = deque(("r", i) for i in sorted(seen_rows))
    while queue:
        kind, k = queue.popleft()
        if kind == "r":
            for j in range(n):
                if (k, j) not in wet and j not in seen_cols:
                    seen_cols.add(j)
                    queue.append(("c", j))
        else:
            for i in range(m):
                if (i, k) in wet and i not in seen_rows:
                    seen_rows.add(i)
                    queue.append(("r", i))
    return sorted(seen_rows), sorted(seen_cols)


def gale_rows(rows, cols):
    """Gale's table by his dual sequence: the k largest columns' asks, and
    the first k terms of the rows' dual sequence summed (what the rows can
    give any k columns), for k = 1 to the number of columns."""
    b = sorted(cols, reverse=True)
    s = dual(rows, len(b))
    asks = [sum(b[:k]) for k in range(1, len(b) + 1)]
    gives = [sum(s[:k]) for k in range(1, len(b) + 1)]
    return b, asks, gives


def chapter_03(rows, face_cols):
    m, cols = len(rows), NO_FIT_COLS
    n = len(cols)
    paths = water_trace(rows, cols)
    states = water_states(rows, cols, paths)
    end = states[-1]
    wet, sent, drained = end
    flow = sum(drained)
    best, source_rows, source_cols = smallest_cut(rows, cols)
    cut_off = [j for j in range(n) if j not in source_cols]
    left = [i for i in range(m) if sent[i] < rows[i]]
    first = cut_off[0]
    pipes = len(source_rows) * len(cut_off)
    reached_drains = sum(cols[j] for j in source_cols)
    emit("DT-F9.paths", len(paths))
    emit("DT-F9.trace", ",".join(path_text(p) for p, _ in paths))
    emit("DT-F9.takebacks", sum(1 for p, _ in paths if len(p) > 4))
    emit("DT-F9.flow", flow)
    emit("DT-F9.left.row", left[0])
    emit("DT-F9.left.units", rows[left[0]] - sent[left[0]])
    emit("DT-F9.first.wet", sum(1 for i in range(m) if (i, first) in wet))
    emit("DT-F9.first.needs", cols[first])
    emit("DT-F9.rows_with_water", sum(1 for r in rows if r > 0))
    emit("DT-F9.reached.rows", csv(source_rows))
    emit("DT-F9.reached.cols", csv(source_cols))
    emit("DT-F9.cut.unreached_taps", sum(r for i, r in enumerate(rows) if i not in source_rows))
    emit("DT-F9.cut.into_unreached", pipes)
    emit("DT-F9.cut.reached_drains", reached_drains)
    emit("DT-F9.cut", best)
    for key, cs in (("DT-F9", cols), ("DT-F9.face", face_cols)):
        b, asks, gives = gale_rows(rows, cs)
        failing = [k for k in range(1, len(b) + 1) if asks[k - 1] > gives[k - 1]]
        emit(f"{key}.sorted", csv(b))
        emit(f"{key}.asks", csv(asks))
        emit(f"{key}.gives", csv(gives))
        emit(f"{key}.failing", csv(failing) if failing else "none")

    check("chapter 03: one row is left with water and one column cut off", len(left) == 1 and len(cut_off) == 1)
    check(
        "chapter 03: the water's own search reaches exactly the smallest cut's source side",
        still_reaches(rows, cols, end) == (source_rows, source_cols),
    )
    check("chapter 03: the flow equals the smallest cut, by both cut methods", flow == best == max_flow_cut(rows, cols))
    check(
        "chapter 03: the cut-off columns ask for more than the dual sequence lets the rows give them",
        sum(cols[j] for j in cut_off) > sum(dual(rows, n)[: len(cut_off)]),
    )
    # equal totals matter: these pass every size of the table and fit nothing
    _, asks, gives = gale_rows((2, 2), (1, 1))
    check(
        "chapter 03: rows 2, 2 with columns 1, 1 pass every size, have unequal totals, and fit nothing",
        all(a <= g for a, g in zip(asks, gives)) and not gale_feasible((2, 2), (1, 1)) and not fits((2, 2), (1, 1)),
    )

    # on every 3x3 pair of shadows with equal totals, the water stops short
    # exactly when the table fails, and then the columns its own search
    # cannot reach ask for more than the rows can give them
    bad = 0
    for r in product(range(4), repeat=3):
        for c in product(range(4), repeat=3):
            if sum(r) != sum(c):
                continue
            got = water_states(r, c, water_trace(r, c))
            final = got[-1] if got else (frozenset(), [0] * 3, [0] * 3)
            short = sum(final[2]) < sum(r)
            _, a, g = gale_rows(r, c)
            if short != any(x > y for x, y in zip(a, g)):
                bad += 1
            elif short:
                _, reached = still_reaches(r, c, final)
                off = [j for j in range(3) if j not in reached]
                if not off or sum(c[j] for j in off) <= sum(dual(r, 3)[: len(off)]):
                    bad += 1
    check("chapter 03: on every 3x3 equal-total pair, short exactly when the table fails, cut off asks too much", bad == 0)

    # "each row gives a group at most one square per column, and no more than
    # its count": every group of columns of every 4x4 picture
    bad = 0
    for mask in range(1 << 16):
        cells = frozenset((i // 4, i % 4) for i in range(16) if (mask >> i) & 1)
        r, c = shadows(cells, 4, 4)
        s = dual(r, 4)
        for k in range(1, 5):
            for group in combinations(range(4), k):
                if sum(c[j] for j in group) > sum(s[:k]):
                    bad += 1
    check("chapter 03: every group of columns of every 4x4 picture holds at most what the rows can give", bad == 0)


# ------------------------------------------- chapter 04, the only picture


def rows_nest(cells, m):
    """Of any two rows, one holds every square the other holds."""
    held = [frozenset(c for r, c in cells if r == i) for i in range(m)]
    return all(a <= b or b <= a for a, b in combinations(held, 2))


def alternates(cells, rect):
    """Whether a rectangle's corners, taken around it, go filled, empty,
    filled, empty (or the other way): read corner by corner."""
    (r0, r1), (c0, c1) = rect
    tl, tr, br, bl = ((r0, c0) in cells, (r0, c1) in cells, (r1, c1) in cells, (r1, c0) in cells)
    return tl == br and tr == bl and tl != tr


def chapter_04(rows, cols):
    m, n = len(rows), len(cols)
    every = set(fits(rows, cols))
    built = water_states(rows, cols, water_trace(rows, cols))[-1][0]
    # in the model's order: pairs of rows, then pairs of columns
    found = sorted(switches(built))
    first = found[0]
    second = switched(built, first)
    emit("DT-F10.switch.rows", len(first[0]))
    emit("DT-F10.face.switches", len(found))
    emit("DT-F10.face.first", f"{csv(first[0])};{csv(first[1])}")
    emit("DT-F10.face.second", pattern(second, m, n))
    emit("DT-F10.face.second.differ", len(built ^ second))
    emit("DT-F10.face.second.fits", flag(second in every))
    check(
        "chapter 04: every switch of the water's picture makes another of the 64",
        all(switched(built, sw) in every and switched(built, sw) != built for sw in found),
    )
    rects = list(product(combinations(range(m), 2), combinations(range(n), 2)))
    check(
        "chapter 04: the water's switches are its alternating rectangles, read corner by corner",
        set(found) == {rc for rc in rects if alternates(built, rc)},
    )

    tree = parse(TREE)
    t_rows, t_cols = shadows(tree, m, n)
    t_paths = water_trace(t_rows, t_cols)
    t_built = water_states(t_rows, t_cols, t_paths)[-1][0]
    emit("DT-F10.tree.total", len(tree))
    emit("DT-F10.tree.paths", len(t_paths))
    emit("DT-F10.tree.trace", ",".join(path_text(p) for p, _ in t_paths))
    emit("DT-F10.tree.takebacks", sum(1 for p, _ in t_paths if len(p) > 4))
    emit("DT-F10.tree.built", pattern(t_built, m, n))
    emit("DT-F10.tree.built.is_tree", flag(t_built == tree))
    emit("DT-F10.tree.built.switches", len(switches(t_built)))
    emit("DT-F10.tree.rectangles", len(rects))
    emit("DT-F10.tree.nested", flag(rows_nest(t_built, m)))
    check(
        "chapter 04: none of the tree's rectangles alternates, read corner by corner",
        not any(alternates(t_built, rc) for rc in rects),
    )
    check("chapter 04: Gale's construction builds the tree too", gale_build(t_rows, t_cols) == tree)
    check(
        "chapter 04: the tree's shadows fit one picture, counted over the columns",
        count_fits(tuple(sorted(t_rows)), t_cols) == 1,
    )
    check("chapter 04: the depth-first flow agrees (tree)", max_flow_dfs(t_rows, t_cols) == len(t_paths) == len(tree))

    # Ryser's chain is not always one switch: two 3x3 pictures with one
    # square in each row and column, six squares apart, two switches apart
    a, b = parse(("100", "010", "001")), parse(("010", "001", "100"))
    check(
        "chapter 04: same shadows, six squares apart, no one switch between them, two switches do it",
        shadows(a, 3, 3) == shadows(b, 3, 3)
        and len(a ^ b) == 6
        and all(switched(a, sw) != b for sw in switches(a))
        and distances(a).get(b) == 2,
    )


# ---------------------------------------------- chapter 05, a third shadow


def diagonal(cells, m, n):
    """Squares on each diagonal from lower left to upper right, diagonal k
    walked square by square: (k, 0), (k - 1, 1), ... while inside."""
    return tuple(
        sum(1 for r in range(m) if 0 <= k - r < n and (r, k - r) in cells) for k in range(m + n - 1)
    )


def falling(cells, m, n):
    """The other diagonals, upper left to lower right (column minus row)."""
    return tuple(sum(1 for r in range(m) if 0 <= r + j < n and (r, r + j) in cells) for j in range(1 - m, n))


def chapter_05(rows, cols):
    m, n = len(rows), len(cols)
    face = parse(FACE)
    twin = switched(face, OPENING)
    every = fits(rows, cols)
    target = diagonal(face, m, n)
    t = diagonal(twin, m, n)
    lose = [k for k in range(len(target)) if t[k] < target[k]]
    gain = [k for k in range(len(target)) if t[k] > target[k]]
    survivors = sorted((p for p in every if diagonal(p, m, n) == target), key=lambda p: pattern(p, m, n))
    others = [p for p in survivors if p != face]
    other = others[0]
    six = face ^ other
    changes = []
    for line_of in (lambda rc: rc[0], lambda rc: rc[1], lambda rc: rc[0] + rc[1]):
        for line in sorted({line_of(rc) for rc in six}):
            on = [rc for rc in six if line_of(rc) == line]
            lost = sum(1 for rc in on if rc in face)
            changes.append((lost, len(on) - lost))
    emit("DT-F11.shadows", len((rows, cols, target)))
    emit("DT-F11.diagonals", len(target))
    emit("DT-F11.face.diag", csv(target))
    emit("DT-F11.twin.diag", csv(t))
    emit("DT-F11.twin.lose", csv(lose))
    emit("DT-F11.twin.losers", len(lose))
    emit("DT-F11.twin.gain", csv(gain))
    emit("DT-F11.twin.gain.by", t[gain[0]] - target[gain[0]])
    emit("DT-F11.face.switches.keeping", sum(1 for sw in switches(face) if diagonal(switched(face, sw), m, n) == target))
    near = [p for p, d in distances(face).items() if d == 1]
    emit("DT-F11.one_switch", len(near))
    emit("DT-F11.one_switch.pass", sum(1 for p in near if diagonal(p, m, n) == target))
    emit("DT-F11.survivors", len(survivors))
    emit("DT-F11.dropped", len(every) - len(survivors))
    emit("DT-F11.others", len(others))
    emit("DT-F11.other", pattern(other, m, n))
    emit("DT-F11.other.from_face", distances(face)[other])
    emit("DT-F11.other.differ", len(six))
    emit("DT-F11.other.lines", len(changes))
    emit("DT-F11.other.loses", changes[0][0])
    emit("DT-F11.other.gains", changes[0][1])
    emit("DT-F11.only_one", flag(len(survivors) == 1))

    check("chapter 05: each diagonal count adds up to the face's squares", sum(target) == len(face))
    check("chapter 05: every line through the six squares loses one and gains one", set(changes) == {(1, 1)})
    check(
        "chapter 05: the other survivor casts all three of the face's shadows",
        shadows(other, m, n) == (tuple(rows), tuple(cols)) and diagonal(other, m, n) == target,
    )
    check(
        "chapter 05: the count over the columns agrees that 64 pictures were tested",
        count_fits(tuple(sorted(rows)), cols) == len(every),
    )
    # this example only, for the notes: the other diagonals also leave two,
    # and both directions together leave the face alone
    down = falling(face, m, n)
    check(
        "chapter 05: the other diagonal direction alone also leaves two of the 64, both together one",
        sum(1 for p in every if falling(p, m, n) == down) == 2
        and [p for p in every if falling(p, m, n) == down and diagonal(p, m, n) == target] == [face],
    )


def main():
    rows, cols = facts()
    chapter_02(rows, cols, fits(rows, cols), distances(parse(FACE)))
    chapter_03(rows, cols)
    chapter_04(rows, cols)
    chapter_05(rows, cols)
    edges()
    print(f"# {_CHECKS[0] - len(_FAILED)}/{_CHECKS[0]} cross-checks passed", file=sys.stderr)
    return 1 if _FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
