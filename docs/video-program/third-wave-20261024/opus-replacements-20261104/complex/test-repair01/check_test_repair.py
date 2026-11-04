"""Checks on test-repair01. No Rust runs here: `cargo test` is the only real check, and root runs it.

    python3 -I -B check_test_repair.py REPO EMPTY_SCRATCH

Run from anywhere; paths in the output are relative. It writes only under EMPTY_SCRATCH.

It checks that:
- the two baseline files and build03's log still have the hashes this patch was made against;
- the patch regenerates byte for byte, applies to canonical, gives the candidate files, and
  reverses back to canonical;
- no byte before `#[cfg(test)]` changes, the test count is the same, and every assertion of the
  old tests is still there except the one exact frame comparison the assignment replaces;
- rustfmt, by file name on the whole module tree, accepts the result;
- a Python mirror of both tests, reading its constants and test rows from the Rust text and using
  the same IEEE arithmetic, reproduces both of build03's panics on canonical (down to the printed
  values) and passes on the candidate, including the assertions no build has reached yet;
- defects planted in a scratch copy of the Rust text are caught by that mirror.

The mirror is Python reading Rust. It checks the geometry and the rounding, not that the Rust
compiles or that the test binary agrees.
"""

import hashlib
import json
import math
import re
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/complex_analysis_visual"
CHAPTERS = f"{CRATE}/src/chapters"
HERE = Path(__file__).resolve().parent
FILES = ("real_integral", "two_planes")
LOG = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-build03/command.log"
PATCH = "test-repair01.patch"
FAILURES = []


def report(ok, text):
    print(f"{'ok  ' if ok else 'FAIL'} {text}")
    if not ok:
        FAILURES.append(text)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


# ---------------------------------------------------------------- the arithmetic of model.rs


class C:
    __slots__ = ("re", "im")

    def __init__(self, re, im=0.0):
        self.re, self.im = re, im

    def __add__(self, o):
        return C(self.re + o.re, self.im + o.im)

    def __sub__(self, o):
        return C(self.re - o.re, self.im - o.im)

    def __mul__(self, o):
        return C(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)

    def recip(self):
        d = self.re * self.re + self.im * self.im
        return C(self.re / d, -self.im / d)

    def __truediv__(self, o):
        return self * o.recip()

    def abs(self):
        return math.hypot(self.re, self.im)

    def arg(self):
        return math.atan2(self.im, self.re)

    def scale(self, k):
        return C(self.re * k, self.im * k)

    @staticmethod
    def polar(r, theta):
        return C(r * math.cos(theta), r * math.sin(theta))


def winding_count(q, path, max_turn):
    """`model::winding_count(|z| Some(z - q), path)`, and the largest step it saw."""
    values = [z - q for z in path]
    if any(w.abs() == 0.0 for w in values):
        return None, None
    total, worst = 0.0, 0.0
    for k in range(len(values)):
        turn = (values[(k + 1) % len(values)] / values[k]).arg()
        if abs(turn) > max_turn:
            return None, abs(turn)
        total += turn
        worst = max(worst, abs(turn))
    w = total / math.tau
    n = float(round(w))
    return (int(n) if abs(w - n) < 1e-6 else None), worst


def body(source, head):
    start = source.index(head)
    return source[start : start + source[start:].index("\n    }\n")]


# ---------------------------------------------------------------- the two tests, read from Rust


def contour_mirror(source, r):
    """`contour(r)` from the text of `contour` and `arc`."""
    start = re.search(r"let mut zs = vec!\[C::real\((-?)r\), C::real\((-?)r\)\];", source)
    arc = re.search(r"\(0\.\.=(\d+)\)\.map\(\|k\| C::polar\(r, (-?)PI \* f64::from\(k\) / (\d+)\.0\)\)", source)
    first = -r if start.group(1) else r
    second = -r if start.group(2) else r
    sign = -1.0 if arc.group(2) else 1.0
    points = [C.polar(r, sign * math.pi * k / float(arc.group(3))) for k in range(0, int(arc.group(1)) + 1)]
    return [C(first), C(second)] + points[1:]


def walked_mirror(path, step):
    zs = []
    for k, a in enumerate(path):
        b = path[(k + 1) % len(path)]
        pieces = int(max(math.ceil((b - a).abs() / step), 1.0))
        zs.extend(a + (b - a).scale(j / pieces) for j in range(pieces))
    return zs


def number(text, **names):
    return eval(text, {"__builtins__": {}}, names)  # our own Rust text, simple arithmetic only


def contour_test(source, model, base):
    """The panic of `the_contour_encloses_exactly_the_upper_poles`, or None if it passes; and notes."""
    max_turn = number(re.search(r"pub const MAX_STEP_TURN: f64 = PI / 2\.0;", model) and "pi / 2.0", pi=math.pi)
    radii = re.search(r"pub const ARC_RADII: \[f64; 3\] = \[([\d.]+),", model)
    test = body(source, "fn the_contour_encloses_exactly_the_upper_poles()")
    if base:
        r = float(radii.group(1))
        path = contour_mirror(source, r)
        rows = []
    else:
        fixed = re.search(r"let r = ([\w:\[\]. ]+);", test).group(1)
        r = float(radii.group(1)) if fixed == "model::ARC_RADII[0]" else float(fixed)
        step = re.search(r"walked\(&contour\(r\), ([^)]+)\)", test).group(1)
        path = walked_mirror(contour_mirror(source, r), number(step))
        rows = [
            (C(number(a, r=r), number(b, r=r)), int(want))
            for a, b, want in re.findall(r"\(C::new\(([^,()]+), ([^,()]+)\), ([01])\),", test)
        ]
    worst = 0.0
    roots = [C.polar(1.0, math.pi * (2 * k + 1) / 4.0) for k in range(4)]
    for q in roots:
        turns, seen = winding_count(q, path, max_turn)
        worst = max(worst, seen or 0.0)
        if turns != int(q.im > 0.0):
            return ("root", turns, int(q.im > 0.0)), worst, len(path), len(rows)
    for q, want in [(C(0.0, 1.0), 1), (C(0.0, -1.0), 0)] + rows:
        turns, seen = winding_count(q, path, max_turn)
        worst = max(worst, seen or 0.0)
        if turns != want:
            return ((q.re, q.im), turns, want), worst, len(path), len(rows)
    return None, worst, len(path), len(rows)


def plane(center, width, x, y):
    return {"center": center, "width": width, "height": width * (y[1] - y[0]) / (x[1] - x[0]), "x": x, "y": y}


def v(x, y):
    return (x, y, 0.0)


def planes_mirror(source, input_half, output_half):
    fn = source[source.index("pub fn planes(") :]
    fn = fn[: fn.index("\n}\n")]
    rows = re.findall(r"Plane::new\(\s*(v\([^)]*\)),\s*([\d.]+),\s*(\([^)]*\)),\s*(\([^)]*\)),?\s*\)", fn)
    built = [
        plane(*[number(e, v=v, input_half=input_half, output_half=output_half) for e in row]) for row in rows
    ]
    return built[0], built[1]


def halves(source, name):
    m = re.search(rf"pub const {name}: \(f64, f64\) = \(([\d.]+), ([\d.]+)\);", source)
    return float(m.group(1)), float(m.group(2))


def frame_text(a):
    return f"([{a['center'][0]}, {a['center'][1]}, {a['center'][2]}], {a['width']}, {a['height']})"


def frames_test(source, base):
    """The panic of the frame comparison in `the_grid_and_its_images_fit_the_windows`, or None; and notes."""
    square, inverse, walk = (halves(source, n) for n in ("SQUARE_HALVES", "INVERSE_HALVES", "WALK_HALVES"))
    walk_r = float(re.search(r"pub const WALK_R: f64 = ([\d.]+);", source).group(1))
    inp, out = planes_mirror(source, *square)
    inp2, out2 = planes_mirror(source, *inverse)
    inp3, out3 = planes_mirror(source, *walk)
    pairs = [(inp, inp2), (inp, inp3), (out, out2), (out, out3)]
    notes = {"negatives": 0, "bound": None}

    if base:
        same = lambda a, b: (a["center"], a["width"], a["height"]) == (b["center"], b["width"], b["height"])
    else:
        test = source[source.index("fn same_rectangle(") :]
        factor = re.search(
            r"\(a\.frame\.height - b\.frame\.height\)\.abs\(\) <= ([\d.e]+) \* f64::EPSILON \* a\.frame\.width", test
        ).group(1)
        k = float(factor)

        def same(a, b):
            return (
                a["center"] == b["center"]
                and a["width"] == b["width"]
                and abs(a["height"] - b["height"]) <= k * sys.float_info.epsilon * a["width"]
            )

        notes["bound"] = k * sys.float_info.epsilon * 5.4

    for a, b in pairs:
        if not same(a, b):
            return ("frame", frame_text(a), frame_text(b)), notes

    if not base:
        test = body(source, "fn the_grid_and_its_images_fit_the_windows()")
        let = re.search(r"let \(centre, square\) = \((v\([^)]*\)), \(-SQUARE_HALVES\.0, SQUARE_HALVES\.0\)\);", test)
        centre = number(let.group(1), v=v)
        sq = (-square[0], square[0])
        env = {"v": v, "centre": centre, "square": sq}

        def expr(e):
            return number(re.sub(r"square\.([01])", r"square[\1]", e), **env)

        control = re.search(r"assert!\(same_rectangle\(inp, Plane::new\(centre, 5\.4, square, square\)\)\);", test)
        if control is None or not same(inp, plane(centre, 5.4, sq, sq)):
            return ("control", "the recipe itself", False), notes
        block = test[test.index("for other in [") : test.index("] {\n            assert!(!same_rectangle")]
        rows = re.findall(r"Plane::new\((v\([^)]*\)|centre), ([\d.e+\- ]+), (square|\([^()]*\)), (square|\([^()]*\))\),", block)
        others = [plane(expr(c), expr(w), expr(x), expr(y)) for c, w, x, y in rows]
        if re.search(r"^\s*out,$", block, re.M):
            others.append(out)
        notes["negatives"] = len(others)
        for other in others:
            if same(inp, other):
                return ("negative", frame_text(other), True), notes

    # the assertions after the panic, which no build has reached
    for a, b in [(inp, out), (inp2, out2), (inp3, out3)]:
        if a["width"] / (a["x"][1] - a["x"][0]) != b["width"] / (b["x"][1] - b["x"][0]):
            return ("unit", frame_text(a), frame_text(b)), notes
    if not (out3["x"][0] <= walk_r <= out3["x"][1] and out3["y"][0] <= 0.0 <= out3["y"][1]) or not walk_r < walk[0] / 2.0:
        return ("walk", walk_r, walk), notes
    return None, notes


# ---------------------------------------------------------------- the checks


def tests_of(text):
    return text[text.index("#[cfg(test)]") :]


def asserts(text):
    return re.findall(r"assert(?:_eq|_ne)?!\((?:[^;]|\n)*?\);", tests_of(text))


def main():
    repo, scratch = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if scratch.exists() and any(scratch.iterdir()):
        sys.exit("EMPTY_SCRATCH is not empty")
    scratch.mkdir(parents=True, exist_ok=True)
    baseline = json.loads((HERE / "baseline.json").read_text())
    relative = [f"{CHAPTERS}/{name}.rs" for name in FILES]
    canon = {name: (repo / CHAPTERS / f"{name}.rs").read_text() for name in FILES}
    cand = {name: (HERE / CHAPTERS / f"{name}.rs").read_text() for name in FILES}
    model = (repo / CRATE / "src/model.rs").read_text()

    print("# baseline")
    for path in relative:
        report(sha(repo / path) == baseline["patched_files"][path], f"canonical base hash {path}")
    report(sha(repo / LOG) == baseline["build_log"]["sha256"], "build03 log hash")
    for path, want in baseline["read_dependencies"].items():
        report(sha(repo / path) == want, f"dependency unchanged {path}")

    print("# patch")
    diff = []
    for path in relative:
        r = run(["diff", "-u", "--label", f"a/{path}", "--label", f"b/{path}", str(repo / path), str(HERE / path)])
        report(r.returncode == 1, f"differs {path}")
        diff.append(r.stdout)
    report("".join(diff) == (HERE / PATCH).read_text(), "patch regenerates byte for byte")
    root = scratch / "patch-root"
    for path in relative:
        (root / path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / path, root / path)
    r = run(["patch", "-p1", "-s", "-i", str(HERE / PATCH)], cwd=root)
    report(r.returncode == 0, "patch -p1 applies to canonical")
    for path in relative:
        report((root / path).read_bytes() == (HERE / path).read_bytes(), f"applied equals candidate {path}")
    r = run(["patch", "-p1", "-R", "-s", "-i", str(HERE / PATCH)], cwd=root)
    report(r.returncode == 0, "patch -R reverses")
    for path in relative:
        report((root / path).read_bytes() == (repo / path).read_bytes(), f"reversed equals canonical {path}")

    print("# scope: tests only, nothing weakened")
    for name in FILES:
        a, b = canon[name], cand[name]
        report(a[: a.index("#[cfg(test)]")] == b[: b.index("#[cfg(test)]")], f"{name}: no byte changes before #[cfg(test)]")
        report(a.count("#[test]") == b.count("#[test]"), f"{name}: {a.count('#[test]')} tests, same count")
    kept = {name: [x for x in asserts(canon[name]) if x not in asserts(cand[name])] for name in FILES}
    report(kept["real_integral"] == [], "real_integral: every old assertion is still there, byte for byte")
    report(
        len(kept["two_planes"]) == 1 and "a.frame.height" in kept["two_planes"][0],
        "two_planes: only the exact frame comparison is replaced",
    )
    added = {name: len(asserts(cand[name])) - len(asserts(canon[name])) for name in FILES}
    report(added["real_integral"] >= 1 and added["two_planes"] >= 2, f"assertions added: {added}")
    staged = HERE.parent / "reading-repair01" / CRATE
    if staged.exists():
        files = {str(p.relative_to(staged)) for p in staged.rglob("*") if p.is_file()}
        ours = {f"src/chapters/{name}.rs" for name in FILES}
        report(not (files & ours), "shares no file with reading-repair01: either order applies")

    print("# rustfmt, whole module tree (canonical + this overlay)")
    tree = scratch / "fmt"
    shutil.copytree(repo / CRATE / "src", tree / "src")
    for name in FILES:
        shutil.copyfile(HERE / CHAPTERS / f"{name}.rs", tree / f"src/chapters/{name}.rs")
    r = run(
        ["rustfmt", "--check", "--edition", "2024", "--config-path", str(repo / "rustfmt.toml"), str(tree / "src/main.rs")]
    )
    report(r.returncode == 0, "rustfmt --check")
    if r.returncode:
        print(r.stdout[:3000])

    print("# build03's panics, reproduced on canonical")
    log = (repo / LOG).read_text()
    panic, _, _, _ = contour_test(canon["real_integral"], model, base=True)
    report(panic == ("root", None, 1), f"canonical contour: {panic}")
    report("real_integral.rs:677:13" in log and "left: None" in log and "right: Some(1)" in log, "the log has that panic")
    panic, _ = frames_test(canon["two_planes"], base=True)
    report(panic is not None and panic[0] == "frame", f"canonical frames: {panic}")
    report(
        panic is not None and f"left: {panic[1]}" in log and f"right: {panic[2]}" in log,
        "the mirror's two printed tuples are the log's, character for character",
    )
    path = contour_mirror(canon["real_integral"], 2.0)
    report(len(path) == 122 and path[0].re == -2.0 and path[1].re == 2.0, "the diameter is one chord of the 122-point path")
    turn = (path[1] - C.polar(1.0, math.pi / 4.0)) / (path[0] - C.polar(1.0, math.pi / 4.0))
    print(f"     that chord turns z − e^(iπ/4) by {abs(turn.arg()):.4f} rad; the guard allows {math.pi / 2.0:.4f}")

    print("# the candidate")
    panic, worst, samples, rows = contour_test(cand["real_integral"], model, base=False)
    report(panic is None, f"contour test passes in the mirror: {panic}")
    report(rows == 8, f"{rows} near-edge rows read from the Rust text")
    report(worst < math.pi / 2.0, f"{samples} samples; largest step {worst:.4f} rad, guard {math.pi / 2.0:.4f}")
    panic, notes = frames_test(cand["two_planes"], base=False)
    report(panic is None, f"frame test and its tail pass in the mirror: {panic}")
    report(notes["negatives"] == 5, f"{notes['negatives']} mismatched layouts all told apart")
    print(f"     height bound {notes['bound']:.3e} scene units; a 4K pixel is about {14.22 / 3840:.3e}")

    print("# planted defects, each must be caught by the mirror")
    plants = [
        ("real_integral", "(C::new(0.0, r - 0.01), 1),", "(C::new(0.0, r - 0.01), 0),", "near-edge row flipped"),
        ("real_integral", "(C::new(0.0, -0.01), 0),", "(C::new(0.0, -0.01), 1),", "row below the diameter flipped"),
        ("real_integral", "walked(&contour(r), 1.0 / 256.0)", "walked(&contour(r), 8.0)", "no refinement: raw chords"),
        ("real_integral", "walked(&contour(r), 1.0 / 256.0)", "walked(&contour(r), 1.0 / 16.0)", "steps too long for the near-edge rows"),
        ("real_integral", "C::polar(r, PI * f64::from(k) / 120.0)", "C::polar(r, -PI * f64::from(k) / 120.0)", "the lower half circle"),
        ("real_integral", "vec![C::real(-r), C::real(r)]", "vec![C::real(r), C::real(-r)]", "diameter walked backwards"),
        ("real_integral", "let r = model::ARC_RADII[0];\n        let path = walked", "let r = 0.5;\n        let path = walked", "a radius that misses the poles"),
        ("two_planes", "<= 2.0 * f64::EPSILON", "<= 0.0 * f64::EPSILON", "bound zero: the build03 failure again"),
        ("two_planes", "<= 2.0 * f64::EPSILON", "<= 2e7 * f64::EPSILON", "bound wide enough to pass a 1e-9 change"),
        ("two_planes", "v(3.5, -0.35),", "v(-3.5, -0.35),", "both planes on one spot"),
        ("two_planes", "(-input_half, input_half),\n            (-input_half, input_half),", "(-input_half, input_half),\n            (-input_half / 2.0, input_half / 2.0),", "planes() stops being square"),
        ("two_planes", "pub const WALK_R: f64 = 0.5;", "pub const WALK_R: f64 = 1.0;", "the tail: the walk leaves its half"),
        ("two_planes", "pub const WALK_HALVES: (f64, f64) = (1.5, 1.5);", "pub const WALK_HALVES: (f64, f64) = (1.5, 2.0);", "the tail: unequal scales"),
    ]
    for name, old, new, what in plants:
        text = cand[name]
        if text.count(old) != 1:
            report(False, f"plant not applicable ({text.count(old)} matches): {what}")
            continue
        planted = text.replace(old, new)
        if name == "real_integral":
            caught = contour_test(planted, model, base=False)[0]
        else:
            caught = frames_test(planted, base=False)[0]
        report(caught is not None, f"caught: {what}: {caught}")

    print("# hashes")
    for path in [PATCH, "baseline.json", "check_test_repair.py"] + relative:
        print(f"{sha(HERE / path)}  {path}")
    print(f"\n{'ALL PASS' if not FAILURES else f'{len(FAILURES)} FAILED'}")
    sys.exit(1 if FAILURES else 0)


if __name__ == "__main__":
    main()
