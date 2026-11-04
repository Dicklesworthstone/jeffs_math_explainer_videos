"""Mirror the_tiles_stop_under_their_names from the Rust text, with mutations it must catch, and walk the call sites.

python3 -I -B pose_mirror.py ARITHMETIC_RS D4_GROUP_RS TILE_MODEL_RS   (from the repo root; prints only)

It does not run Rust. LEFT, RIGHT, SEQUENCE and the test's table are parsed from the source. The Elem and matrix
definitions are written out here, and the script stops unless the Rust lines they copy are still present.
"""
import math
import re
import sys
from pathlib import Path

TILE_LINES = [
    "[[c, -s, 0.], [s, c, 0.], [0., 0., 1.]]",
    "[[1., 0., 0.], [0., c, -s], [0., s, c]]",
    "let (s, c) = (std::f64::consts::FRAC_PI_2 * t).sin_cos();",
    "let (s, c) = (std::f64::consts::PI * t).sin_cos();",
    "(0..3).map(|k| a[i][k] * b[k][j]).sum()",
    "pub const CORNERS: [Point; 4] = [[1., 1., 0.], [1., -1., 0.], [-1., -1., 0.], [-1., 1., 0.]];",
]
D4_LINES = [
    "let carried = if self.flip { (4 - other.turns) % 4 } else { other.turns };",
    "Elem::new(self.turns + carried, self.flip ^ other.flip)",
    "Elem::new(4 - self.turns, false)",
    "let turn = tile_model::turn(f64::from(self.turns));",
    "tile_model::compose(turn, tile_model::flip(1.0))",
]
CORNERS = [(1.0, 1.0, 0.0), (1.0, -1.0, 0.0), (-1.0, -1.0, 0.0), (-1.0, 1.0, 0.0)]
IDENTITY = ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0))


def turn(t):
    s, c = math.sin(math.pi / 2 * t), math.cos(math.pi / 2 * t)
    return ((c, -s, 0.0), (s, c, 0.0), (0.0, 0.0, 1.0))


def flip(t):
    s, c = math.sin(math.pi * t), math.cos(math.pi * t)
    return ((1.0, 0.0, 0.0), (0.0, c, -s), (0.0, s, c))


def compose(a, b):
    return tuple(tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)) for i in range(3))


def apply(m, p):
    return tuple(sum(a * b for a, b in zip(row, p)) for row in m)


def same_corners(a, b):
    return all(all(abs(x - y) < 1e-9 for x, y in zip(apply(a, p), apply(b, p))) for p in CORNERS)


class Elem:
    def __init__(self, turns, is_flip):
        self.turns, self.flip = turns % 4, is_flip

    def mul(self, other):
        carried = (4 - other.turns) % 4 if self.flip else other.turns
        return Elem(self.turns + carried, self.flip ^ other.flip)

    def inv(self):
        return self if self.flip else Elem(4 - self.turns, False)

    def matrix(self):
        t = turn(float(self.turns))
        return compose(t, flip(1.0)) if self.flip else t

    def __repr__(self):
        return f"r^{self.turns}" + (" s" if self.flip else "")


E, R, R2, S = Elem(0, False), Elem(1, False), Elem(2, False), Elem(0, True)
NAMES = {"E": E, "R": R, "R2": R2, "S": S}


def parse_moves(source, name):
    m = re.search(rf"const {name}: \[Move; \d+\] = \[(.*?)\];", source, re.S)
    moves = []
    for token in re.findall(r"Move::Turn\((-?[\d.]+)\)|(Move::Flip)", m.group(1)):
        moves.append(("turn", float(token[0])) if token[0] else ("flip", None))
    return moves


def pose(moves, t):
    state = IDENTITY
    for i, (kind, n) in enumerate(moves):
        fraction = min(max(t - i, 0.0), 1.0)
        state = compose(turn(n * fraction) if kind == "turn" else flip(fraction), state)
    return state


def name_of(expr):
    expr = expr.strip()
    m = re.fullmatch(r"(\w+)\.mul\((\w+)\)", expr)
    if m:
        return NAMES[m.group(1)].mul(NAMES[m.group(2)])
    m = re.fullmatch(r"(\w+)\.inv\(\)", expr)
    if m:
        return NAMES[m.group(1)].inv()
    return NAMES[expr]


def split_top(text):
    """Split the table's `(t, name), (t, name)` text at its top-level commas."""
    rows, depth, current = [], 0, ""
    for ch in text:
        if ch == "(":
            depth += 1
        if ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            rows.append(current)
            current = ""
        else:
            current += ch
    rows.append(current)
    return [r.strip() for r in rows if r.strip()]


def parse_table(source):
    body = source[source.index("fn the_tiles_stop_under_their_names"):]
    body = body[: body.index("\n    }\n")]
    rows = re.search(r"for \(t, name\) in \[(.*?)\] \{", body, re.S).group(1)
    table = []
    for row in split_top(rows):
        m = re.fullmatch(r"\(([\d.]+), (.+)\)", row)
        table.append((float(m.group(1)), m.group(2)))
    right = re.search(r"pose\(&RIGHT, ([\d.]+)\), (.+?)\.matrix\(\)\)\);", body)
    return table, (float(right.group(1)), right.group(2))


def test_passes(left, right, table, right_row):
    for t, expr in table:
        if not same_corners(pose(left, t), name_of(expr).matrix()):
            return False, f"left at {t}: {expr}"
    t, expr = right_row
    if not same_corners(pose(right, t), name_of(expr).matrix()):
        return False, f"right at {t}: {expr}"
    return True, ""


def label_after(lines, i):
    """The next caption text after line i, up to two statements on."""
    for line in lines[i + 1 : i + 3]:
        m = re.search(r'(?:caption|replace)\(stage, &kit,(?: \w+,)? "([^"]+)"', line)
        if m:
            return m.group(1)
    return None


# What each label on these tiles means, written from the spoken lines, independent of the test's table.
MEANINGS = {
    "r²": R2, "r³": R.mul(R).mul(R), "r⁴ = e": E, "s": S, "s² = e": E, "s r": S.mul(R), "r s": R.mul(S),
}


def main():
    source = Path(sys.argv[1]).read_text(encoding="utf-8")
    d4 = Path(sys.argv[2]).read_text(encoding="utf-8")
    tile = Path(sys.argv[3]).read_text(encoding="utf-8")
    for needle in TILE_LINES:
        if needle not in tile:
            raise SystemExit(f"tile_model no longer holds: {needle}")
    for needle in D4_LINES:
        if needle not in d4:
            raise SystemExit(f"d4_group no longer holds: {needle}")
    print("Rust definitions match the lines this mirror copies")

    left, right = parse_moves(source, "LEFT"), parse_moves(source, "RIGHT")
    sequence = parse_moves(source, "SEQUENCE")
    table, right_row = parse_table(source)
    print("LEFT", left)
    print("RIGHT", right)
    print("table", table, "right", right_row)
    ok, why = test_passes(left, right, table, right_row)
    print("the test's table holds:", ok, why)
    if not ok:
        raise SystemExit(1)

    print("\nMutations the test must catch")
    wrong = 0
    mutations = []
    for i, (t, expr) in enumerate(table):
        for other in ["E", "R", "R2", "S", "R.inv()", "R.mul(S)", "S.mul(R)"]:
            if name_of(other).matrix() == name_of(expr).matrix() or same_corners(name_of(other).matrix(), name_of(expr).matrix()):
                continue
            mutated = list(table)
            mutated[i] = (t, other)
            mutations.append((f"left t={t:g}: {expr} -> {other}", mutated, right_row, left, right))
    for other in ["R.mul(S)", "E", "S"]:
        mutations.append((f"right t=2: S.mul(R) -> {other}", table, (right_row[0], other), left, right))
    swapped = list(left)
    swapped[6], swapped[7] = swapped[7], swapped[6]
    mutations.append(("LEFT: swap moves 7 and 8 (turn first, then flip)", table, right_row, swapped, right))
    swapped = list(right)
    swapped[0], swapped[1] = swapped[1], swapped[0]
    mutations.append(("RIGHT: swap moves 1 and 2", table, right_row, left, swapped))
    shifted = list(left)
    shifted[3] = ("turn", 2.0)
    mutations.append(("LEFT: fourth turn doubled", table, right_row, shifted, right))
    for label, t_table, t_right, t_left, t_rmoves in mutations:
        passed, _ = test_passes(t_left, t_rmoves, t_table, t_right)
        if passed:
            wrong += 1
            print("  NOT CAUGHT:", label)
    print(f"  {len(mutations)} mutations, {len(mutations) - wrong} caught, {wrong} not caught")

    print("\nMoves the test cannot see: same code, wrong call-site number")
    for t in (7.0,):
        print(f"  perform(left, {t:g}) under `r s`: pose is {'r s' if same_corners(pose(left, t), R.mul(S).matrix()) else 'not r s'};",
              "the table still passes, because it does not read the call sites")

    print("\nEvery named stop on the left and right tiles, walked from the call sites")
    lines = source.splitlines()
    say3 = next(i for i, ln in enumerate(lines) if ln.strip() == 'kit.say(stage, "13_3")?;')
    say6 = next(i for i, ln in enumerate(lines) if ln.strip() == 'kit.say(stage, "13_6")?;')
    positions = {"left": 0.0, "right": 0.0}
    named = []
    for i in range(say3, say6):
        line = lines[i]
        m = re.search(r"(?:perform|advance)\(stage, (left|right), ([\d.]+)", line)
        s = re.search(r'step\(stage, &kit, (left|right), ([\d.]+), \w+, "([^"]+)"', line)
        if s:
            tile, t, label = s.group(1), float(s.group(2)), s.group(3)
            named.append((i + 1, tile, t, label))
            positions[tile] = t
        elif m:
            positions[m.group(1)] = float(m.group(2))
        else:
            c = re.search(r'(left|right)_label = (?:caption|replace)\(stage, &kit,(?: \w+,)? "([^"]+)"', line)
            if c:
                named.append((i + 1, c.group(1), positions[c.group(1)], c.group(2)))
    tested = {t for t, _ in table}
    bad = 0
    for line_no, tile, t, label in named:
        moves = left if tile == "left" else right
        expected = MEANINGS[label]
        holds = same_corners(pose(moves, t), expected.matrix())
        in_table = (t in tested) if tile == "left" else (t == right_row[0])
        bad += 0 if holds else 1
        print(f"  line {line_no}: {tile} at t={t:g} named {label!r}: pose agrees {holds}; in the test's table {in_table}")
    print(f"  {len(named)} named stops, {bad} disagree")

    print("\nStops outside this test, checked here")
    demo = compose(turn(1.0), flip(1.0))
    print("  13_2 right demo at t=2 (closure: turn(t-1) after flip(1)), named r s:", same_corners(demo, R.mul(S).matrix()))
    print("  13_6 SEQUENCE at t=3, named s r s:", same_corners(pose(sequence, 3.0), S.mul(R).mul(S).matrix()))
    print("  13_6 opposite at t=1 (turn(-t)), named r⁻¹:", same_corners(turn(-1.0), R.inv().matrix()))
    print("  13_8 undo at t=4 (turn(t)), named r³ r = e:", same_corners(turn(4.0), E.matrix()))
    print("  13_8 undo at t=6 and t=8, half-turn then half-turn:", same_corners(turn(6.0), R2.matrix()), same_corners(turn(8.0), E.matrix()))


if __name__ == "__main__":
    main()
