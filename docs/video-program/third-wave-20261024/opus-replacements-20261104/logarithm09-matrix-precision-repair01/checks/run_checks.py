"""Checks for the logarithm 09 matrix precision repair. Source and mirrors only: no Cargo, no render.

Usage, from the repo root, with a scratch path that does not exist yet:
  /usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>

Reads canonical, stage06 and root's frames. Writes only checks/outputs/, the patch, source-markers.json
and the fresh scratch folder. It starts caveat_contrast.py and the sheet with `python3 -B`, because they
need PIL. Canonical is never written.
"""

import difflib
import hashlib
import json
import math
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
SCRATCH = Path(sys.argv[1]).resolve()
if SCRATCH.exists():
    raise SystemExit("scratch path exists: give a fresh one")
SCRATCH.mkdir(parents=True)
HERE = Path(__file__).resolve().parent.parent
OUT = HERE / "checks/outputs"
OUT.mkdir(exist_ok=True)
CRATE = "explainer_corpus/logarithm_mysteries"
REL = f"{CRATE}/src/chapters/matrix_logs.rs"
STAGED = HERE / REL
CANON = REPO / REL
LANE = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE06 = LANE / "combined-repair-stage06"
BASE_HASH = "c82b2a4d10f3c5fda758ad110b6297efc5512a643bfc26f4f05e4532c6ebf2cd"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/09_matrix_logs"
failures, log = [], []


def say(text):
    log.append(text)
    print(text)


def check(name, ok, detail=""):
    say(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failures.append(name)


def sha(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode("utf8")).hexdigest()


def clean(text):
    for path, name in ((str(SCRATCH), "<scratch>"), (str(REPO), "<repo>"), (str(Path.home()), "<home>")):
        text = text.replace(path, name)
    return text


def run(argv, cwd=None):
    r = subprocess.run([str(a) for a in argv], cwd=cwd, capture_output=True, text=True)
    return r.returncode, r.stdout + r.stderr


base, new = CANON.read_bytes(), STAGED.read_bytes()
base_text, new_text = base.decode("utf8"), new.decode("utf8")

# ---------------------------------------------------------------- the patch, from the staged file
say("== baseline and patch")
check("canonical is the file this repair was written against", sha(base) == BASE_HASH, sha(base))
check("stage06 carries the same bytes", sha((STAGE06 / REL).read_bytes()) == BASE_HASH)
for name in ("src/model.rs", "src/facts.rs", "src/chapters/draw.rs", "script.tsv", "oracle.py"):
    a, b = (REPO / CRATE / name).read_bytes(), (STAGE06 / CRATE / name).read_bytes()
    say(f"  {name}: canonical {sha(a)[:16]} stage06 {sha(b)[:16]} {'same' if a == b else 'DIFFER'}")
patch_path = HERE / "logarithm09-matrix-precision.patch"
diff = "".join(difflib.unified_diff(base_text.splitlines(keepends=True), new_text.splitlines(keepends=True), f"a/{REL}", f"b/{REL}", n=3))
patch_path.write_text(diff, encoding="utf8")
trial = SCRATCH / "apply"
(trial / REL).parent.mkdir(parents=True)
shutil.copyfile(CANON, trial / REL)
rc, text = run(["git", "apply", patch_path], cwd=trial)
check("the patch applies to canonical by git apply and gives the staged bytes", rc == 0 and sha((trial / REL).read_bytes()) == sha(new), clean(text).strip())
rc, text = run(["git", "apply", "-R", patch_path], cwd=trial)
check("and reverses", rc == 0 and sha((trial / REL).read_bytes()) == BASE_HASH, clean(text).strip())
say(f"staged matrix_logs.rs {sha(new)}")
say(f"patch {sha(patch_path.read_bytes())}")

# ---------------------------------------------------------------- what moved
say("\n== what the patch changes in film code")
split = "#[cfg(test)]\nmod tests"
film_a, film_b = base_text.split(split)[0], new_text.split(split)[0]
la, lb = film_a.split("\n"), film_b.split("\n")
check("film code keeps its line count", len(la) == len(lb), f"{len(la)} lines")
changed = [i + 1 for i, (x, y) in enumerate(zip(la, lb)) if x != y]
check("exactly three film lines change", changed == [179, 213, 266], str(changed))
for i in changed:
    say(f"  {i}: - {la[i - 1].strip()}\n  {i}: + {lb[i - 1].strip()}")
timed = re.compile(r"(until_sentence|kit\.say|stage\.wait|fade_in_all|fade_out_all|swap\(|tween\(|play!|finish)")
check("no timed call changes", [x for x in la if timed.search(x)] == [x for x in lb if timed.search(x)])
for what, needle in (
    ("the series keeps its exact =", r'r"e^{X} = I + X + \tfrac{1}{2}X^{2} + \tfrac{1}{6}X^{3} + \text{...}",'),
    ("the definition keeps its exact =", r'r"X \text{ is a logarithm of } A \iff e^{X} = A",'),
):
    check(what, film_a.count(needle) == 1 and film_b.count(needle) == 1)
strings = lambda s: re.findall(r'"((?:[^"\\]|\\.)*)"', s)  # noqa: E731
gone = sorted(set(strings(film_a)) - set(strings(film_b)))
added = sorted(set(strings(film_b)) - set(strings(film_a)))
check("no on-screen string changes except the e^X word", gone == ["e^{X} ="] and added == [r"e^{X} \approx"], f"{gone} -> {added}")
say("  (the card entries come from the matrix, so they are not strings here)")

# ---------------------------------------------------------------- the mathematics, closed form
say("\n== the numbers, independently of model.rs")
X = [[0.3, -1.0], [0.8, -0.1]]
s = (X[0][0] + X[1][1]) / 2
Y = [[X[0][0] - s, X[0][1]], [X[1][0], X[1][1] - s]]
w = math.sqrt(-(Y[0][0] * Y[0][0] + Y[0][1] * Y[1][0]))
ex = [[math.exp(s) * (math.cos(w) * (i == j) + math.sin(w) / w * Y[i][j]) for j in range(2)] for i in range(2)]
say(f"e^X, closed form e^s (cos w I + sin w / w Y), w = {w:.6f}: {ex}")


def series(m, terms=60):
    total, term = [[1.0, 0.0], [0.0, 1.0]], [[1.0, 0.0], [0.0, 1.0]]
    for k in range(1, terms):
        term = [[(term[i][0] * m[0][j] + term[i][1] * m[1][j]) / k for j in range(2)] for i in range(2)]
        total = [[total[i][j] + term[i][j] for j in range(2)] for i in range(2)]
    return total


sr = series(X)
check("closed form and series agree", max(abs(ex[i][j] - sr[i][j]) for i in range(2) for j in range(2)) < 1e-12)


def entry(x):
    r = round(x * 1000.0) / 1000.0
    return str(int(r)) if r == int(r) else f"{r:g}"


printed = [[entry(ex[i][j]) for j in range(2)] for i in range(2)]
check("the card prints what root read", printed == [["0.905", "-0.97"], ["0.776", "0.517"]], str(printed))
loss = max(abs(float(printed[i][j]) - ex[i][j]) for i in range(2) for j in range(2))
check("so e^X = is false of the printed digits, and approximately is right", 1e-6 < loss < 0.0005, f"largest entry error {loss:.6f}")
exact = lambda m: all(abs(float(entry(x)) - x) < 1e-12 for row in m for x in row)  # noqa: E731
EX = {"turned": [[0.6, -0.8], [0.8, 0.6]], "stretched": [[1.6, 0.0], [0.0, 0.6]], "sheared": [[1.0, 1.0], [0.0, 1.0]], "flipped": [[-1.0, 0.0], [0.0, 1.0]]}
for name, m in EX.items():
    check(f"{name} is exact on its card", exact(m), str([[entry(x) for x in r] for r in m]))
check("X is exact on its card", exact(X))
check("e^X is not", not exact(ex))
old = [[math.cos(math.tau / 6), -math.sin(math.tau / 6)], [math.sin(math.tau / 6), math.cos(math.tau / 6)]]
check("control: the old turned example was not exact", not exact(old), str([[entry(x) for x in r] for r in old]))
T = EX["turned"]
check("the new example is a true turn: orthonormal, det 1", abs(T[0][0] * T[1][1] - T[0][1] * T[1][0] - 1) < 1e-12 and abs(T[0][0] ** 2 + T[1][0] ** 2 - 1) < 1e-12 and abs(T[0][0] * T[0][1] + T[1][0] * T[1][1]) < 1e-12)
say(f"  it turns by {math.degrees(math.atan2(0.8, 0.6)):.2f} degrees; the old one by 60; no script, card or fact names the angle")
say("\n== geometry of the turned picture, mirrored")
CENTRE, UNIT, N = (-3.2, -0.4), 0.78, 3


def extents(m):
    xs, ys = [], []
    for a in (-N, N):
        for b in (-N, N):
            xs.append(CENTRE[0] + UNIT * (m[0][0] * a + m[0][1] * b))
            ys.append(CENTRE[1] + UNIT * (m[1][0] * a + m[1][1] * b))
    return min(xs), max(xs), min(ys), max(ys)


ne, oe = extents(T), extents(old)
say(f"  grid extents new {tuple(round(v, 3) for v in ne)}, old {tuple(round(v, 3) for v in oe)}")
check("the turned grid stays in the safe area ±6.96 × ±3.85", ne[0] > -6.96 and ne[3] < 3.85 and ne[2] > -3.85)
check("it stays clear of the right-hand column at x = 1.4", ne[1] < 1.4)
check("it stays under the title, whose ink starts near y = 3.27 on frame 48", ne[3] < 3.2)
COMPARE = [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [-1.0, 1.0]]
ok = True
for m in EX.values():
    for p in COMPARE:
        q = (m[0][0] * p[0] + m[0][1] * p[1], m[1][0] * p[0] + m[1][1] * p[1])
        sx, sy = CENTRE[0] + UNIT * q[0], CENTRE[1] + UNIT * q[1]
        ok = ok and -7.0 < sx < 0.9 and -3.0 < sy < 2.9
check("the_marked_landings_of_every_example_stay_on_the_panel, mirrored on the new example", ok)
check("the_clip_holds_the_grid, mirrored: a turned n = 3 grid reaches sqrt 18 < 4.4", math.sqrt(18) < 4.4)
landed = [(T[0][0] * p[0] + T[0][1] * p[1], T[1][0] * p[0] + T[1][1] * p[1]) for p in COMPARE]
say(f"  marked points land at {[(round(a, 3), round(b, 3)) for a, b in landed]}")
dmin = min(math.dist(a, b) for i, a in enumerate(landed) for b in landed[i + 1 :])
check("no two landing dots merge: dot radius 0.07 scene", dmin * UNIT > 0.3, f"closest {dmin * UNIT:.3f} scene")

# ---------------------------------------------------------------- the new tests' source conditions, with plants
say("\n== the new tests' conditions on the staged source, and plants")


def t_rounded(source):
    film = source.split(split)[0]
    return all(
        (
            "[[0.6, -0.8], [0.8, 0.6]]," in film,
            r'let e_word = kit.tex(stage, r"e^{X} \approx", 30.0, green, &[], v(2.8, -1.5))?;' in film,
            'r"e^{X} ="' not in film,
            r"\iff e^{X} = A" in film,
        )
    )


def t_ink(source):
    at = source.index('"before and after only: where each point lands, not how it travels",')
    return source[at:].split("\n")[2].strip() == "Palette::ink(),"


check("only_the_rounded_card_says_approximately, its source half, passes", t_rounded(new_text))
check("the_before_and_after_caveat_is_in_ink passes", t_ink(new_text))
check("both fail on canonical (the control)", not t_rounded(base_text) and not t_ink(base_text))
for label, old_s, new_s, test in (
    ("the word goes back to =", r"e^{X} \approx", "e^{X} =", t_rounded),
    ("the definition goes approximate", r"\iff e^{X} = A", r"\iff e^{X} \approx A", t_rounded),
    ("the turned matrix is 60 degrees again", "[[0.6, -0.8], [0.8, 0.6]], // a true", "model::rotation(TAU / 6.0), // a true", None),
    ("the caveat goes back to dim", "not how it travels\",\n            20.0,\n            Palette::ink(),", "not how it travels\",\n            20.0,\n            Palette::dim(),", t_ink),
):
    # the film's occurrence comes first; the test quotes some of these again below it
    if old_s not in new_text.split(split)[0]:
        check(f"plant applies: {label}", False, "not in film code")
        continue
    planted = new_text.replace(old_s, new_s, 1)
    if test is None:
        caught = "[[0.6, -0.8], [0.8, 0.6]]," not in planted.split(split)[0]
    else:
        caught = not test(planted)
    check(f"planted, caught: {label}", caught)

# ---------------------------------------------------------------- what binds this file
say("\n== bindings")
map13 = (REPO / CRATE / "docs/narration-direction-map-oliver13.json").read_text(encoding="utf8")
units = (REPO / CRATE / "narration_units.json").read_text(encoding="utf8")
m13 = json.loads(map13)
check("map13 has no visual hold on this file", not [h for h in m13["visual_holds"] if h["path"].endswith("matrix_logs.rs")])
check("neither map13 nor narration_units.json names the source hash", BASE_HASH not in map13 and BASE_HASH not in units)
audit = json.loads((STAGE06 / "bound-plan/hold-audit.json").read_text(encoding="utf8"))
rows = []
for key in ("cues", "other_waits", "chapter_finishes"):
    rows += [(key, r) for r in audit[key] if r.get("file") == "matrix_logs.rs"]
lines = new_text.split("\n")
moved = []
for key, r in rows:
    marker = r.get("marker")
    if marker is None:
        marker = f'kit.say(stage, "{r["cue"]}")'
    if marker not in lines[r["line"] - 1]:
        moved.append((key, r["line"], marker))
check(f"all {len(rows)} hold-audit rows on this file keep their line and marker", not moved, str(moved))
say("  the source hash appears in stage06's bound-plan/hold-audit.json and inputs.sha256: root's binder must take the new input")

# ---------------------------------------------------------------- cue check on a scratch copy
say("\n== docs/check_cues.py on a scratch copy")
scr = SCRATCH / "cues" / CRATE
(scr / "docs").mkdir(parents=True)
shutil.copytree(REPO / CRATE / "src", scr / "src")
shutil.copyfile(REPO / CRATE / "script.tsv", scr / "script.tsv")
shutil.copyfile(REPO / CRATE / "docs/check_cues.py", scr / "docs/check_cues.py")
rc0, text0 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
shutil.copyfile(STAGED, scr / "src/chapters/matrix_logs.rs")
rc1, text1 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
say(f"  canonical copy exit {rc0}; staged copy exit {rc1}: {clean(text1).strip()[:160]}")
check("the cue checker says the same of both", rc0 == rc1 == 0 and clean(text0) == clean(text1))
broken = new_text.replace('kit.say(stage, "09_3")?;', "")
(scr / "src/chapters/matrix_logs.rs").write_text(broken, encoding="utf8")
rc2, _ = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
check("control: it refuses the file with a say removed", rc2 != 0, f"exit {rc2}")

# ---------------------------------------------------------------- rustfmt
say("\n== rustfmt")
config = REPO / "rustfmt.toml"
for name, path in (("canonical", CANON), ("staged", STAGED)):
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", config, "--check", path])
    check(f"rustfmt --check {name}", rc == 0, clean(text).strip()[:200])

# ---------------------------------------------------------------- frames
say("\n== old frames, for the caveat")
rc, text = run(["python3", "-B", HERE / "checks/caveat_contrast.py", "report", FRAMES, OUT / "caveat-dim-vs-ink-NOT-A-RENDER.png"])
say(clean(text).strip())
check("caveat_contrast.py exits 0 with its controls", rc == 0)

(OUT / "run_checks.txt").write_text(clean("\n".join(log)) + "\n", encoding="utf8")
print("\n" + ("all expectations hold" if not failures else "FAILED: " + "; ".join(failures)))
sys.exit(1 if failures else 0)
