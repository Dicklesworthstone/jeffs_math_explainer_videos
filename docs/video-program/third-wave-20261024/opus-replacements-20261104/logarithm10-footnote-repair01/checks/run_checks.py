"""Checks for the logarithm 10 footnote repair. Source and mirrors only: no Cargo, no render.

Usage, from the repo root, with a scratch path that does not exist yet:
  /usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>

Reads canonical, stage06 and root's frames. Writes only checks/outputs/, the patch, source-markers.json
and the fresh scratch folder. It starts footnote_probe.py with `python3 -B`, because that needs PIL.
Canonical is never written.
"""

import difflib
import hashlib
import json
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
REL = f"{CRATE}/src/chapters/doing_nothing.rs"
STAGED = HERE / REL
CANON = REPO / REL
LANE = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE06 = LANE / "combined-repair-stage06"
BASE_HASH = "eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/10_doing_nothing"
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


def const_str(source, name):
    return re.search(rf'^const {name}: &str = "(.*)";$', source, re.M).group(1)


base, new = CANON.read_bytes(), STAGED.read_bytes()
base_text, new_text = base.decode("utf8"), new.decode("utf8")

# ---------------------------------------------------------------- the patch, from the staged file
say("== baseline and patch")
check("canonical is the file this repair was written against", sha(base) == BASE_HASH, sha(base))
check("stage06 carries the same bytes", sha((STAGE06 / REL).read_bytes()) == BASE_HASH)
for name in ("src/chapters/draw.rs", "script.tsv", "docs/storyboard.md"):
    a, b = (REPO / CRATE / name).read_bytes(), (STAGE06 / CRATE / name).read_bytes()
    say(f"  {name}: canonical {sha(a)[:16]} stage06 {sha(b)[:16]} {'same' if a == b else 'DIFFER'}")
patch_path = HERE / "logarithm10-footnote.patch"
diff = "".join(difflib.unified_diff(base_text.splitlines(keepends=True), new_text.splitlines(keepends=True), f"a/{REL}", f"b/{REL}", n=3))
patch_path.write_text(diff, encoding="utf8")
trial = SCRATCH / "apply"
(trial / REL).parent.mkdir(parents=True)
shutil.copyfile(CANON, trial / REL)
rc, text = run(["git", "apply", patch_path], cwd=trial)
check("the patch applies to canonical by git apply and gives the staged bytes", rc == 0 and sha((trial / REL).read_bytes()) == sha(new), clean(text).strip())
rc, text = run(["git", "apply", "-R", patch_path], cwd=trial)
check("and reverses", rc == 0 and sha((trial / REL).read_bytes()) == BASE_HASH, clean(text).strip())
say(f"staged doing_nothing.rs {sha(new)}")
say(f"patch {sha(patch_path.read_bytes())}")

# ---------------------------------------------------------------- what moved
say("\n== what the patch changes in film code")
split = "#[cfg(test)]\nmod tests"
film_a, film_b = base_text.split(split)[0], new_text.split(split)[0]
la, lb = film_a.split("\n"), film_b.split("\n")
check("film code keeps its line count", len(la) == len(lb), f"{len(la)} lines")
changed = [i + 1 for i, (x, y) in enumerate(zip(la, lb)) if x != y]
check("exactly the expected film lines change", changed == [41, 66, 67, 69, 594, 595], str(changed))
for i in changed:
    say(f"  {i}: - {la[i - 1].strip()[:118]}\n  {i}: + {lb[i - 1].strip()[:118]}")
timed = re.compile(r"(until_sentence|until_phrase|kit\.say|kit\.hold|stage\.wait|fade_in_all|fade_out_all|swap\(|tween\(|play!|finish|music_breath|FINE_PRINT_S)")
check("no timed call and no reading window changes", [x for x in la if timed.search(x)] == [x for x in lb if timed.search(x)])

# ---------------------------------------------------------------- the words
say("\n== what the viewer reads")
OLD_T, OLD_S = const_str(base_text, "TEXTBOOK"), const_str(base_text, "SOURCE")
NEW_T, NEW_S = const_str(new_text, "TEXTBOOK"), const_str(new_text, "SOURCE")
say(f"  upper row: {NEW_T}")
say(f"  lower row: {NEW_S}")


def words_kept(source):
    return const_str(source, "TEXTBOOK") == OLD_T and const_str(source, "SOURCE") == f"precisely: {OLD_S}"


check("the turning-rate explanation is byte for byte the same", OLD_T == NEW_T)
check("the lower row is the same string draw::precisely built, byte for byte", words_kept(new_text), "format!(\"precisely: {words}\")")
check("draw::precisely still formats it so", 'format!("precisely: {words}")' in (REPO / CRATE / "src/chapters/draw.rs").read_text(encoding="utf8"))
for needle in ("−π < Im < π", "(−∞, 0]", "real if A is", "Higham; Gallier Thm 3.11", "no eigenvalue of A"):
    check(f"the qualification stays: {needle}", needle in NEW_S)
characters = len(NEW_T) + len(NEW_S)
check("197 characters, as docs/storyboard.md:99 says", characters == 197 and "197 characters in all" in (REPO / CRATE / "docs/storyboard.md").read_text(encoding="utf8"), str(characters))
window = float(re.search(r"const FINE_PRINT_S: f64 = ([0-9.]+);", new_text).group(1))
check("the reading window is unchanged and still meets the crate's own test", window == 13.2 and characters / window <= 15.0, f"{characters / window:.2f} chars/s over {window} s")

# ---------------------------------------------------------------- geometry
say("\n== geometry, mirrored from widths and heights measured on frame 229")
SIZE = float(re.search(r"const FINE_SIZE: f64 = ([0-9.]+);", new_text).group(1))
ROWS = [float(x) for x in re.search(r"const FINE_ROWS: \[f64; 2\] = \[(-?[0-9.]+), (-?[0-9.]+)\];", new_text).groups()]
UPPER_W18, UPPER_H18, LOWER_W22, LOWER_H22 = 5.444, 0.167, 12.667, 0.233
GRID_BOTTOM = -0.4 - 0.72 * 3.0


def layout(rows, size=SIZE):
    k = size / 18.0
    n = size / 22.0
    return {
        "upper": (rows[0] - UPPER_H18 * k / 2, rows[0] + UPPER_H18 * k / 2, UPPER_W18 * k / 2),
        "lower": (rows[1] - LOWER_H22 * n / 2, rows[1] + LOWER_H22 * n / 2, LOWER_W22 * n / 2),
    }


def t_numeric(rows, size=SIZE):
    g = layout(rows, size)
    return GRID_BOTTOM - g["upper"][1] > 0.15 and g["upper"][0] - g["lower"][1] > 0.15 and g["lower"][0] > -3.6 and g["lower"][2] < 6.86


g = layout(ROWS)
say(f"  upper row y {g['upper'][0]:.3f}..{g['upper'][1]:.3f}, half-width {g['upper'][2]:.3f}; lower row y {g['lower'][0]:.3f}..{g['lower'][1]:.3f}, half-width {g['lower'][2]:.3f}")
say(f"  to the grid {GRID_BOTTOM - g['upper'][1]:.3f}; between the rows {g['upper'][0] - g['lower'][1]:.3f}; to the frame's edge {4 + g['lower'][0]:.3f} (was 0.267)")
say(f"  x-height by arithmetic, CM 0.43 em: 22 pt at 4K {0.43 * 0.22 * 270:.1f} px, 1080p {0.43 * 0.22 * 135:.1f} px; 18 pt at 4K {0.43 * 0.18 * 270:.1f} px")
check("the new test's numeric half holds, mirrored", t_numeric(ROWS))
check("inside the film's safe area ±6.96 × ±3.85", g["lower"][2] < 6.96 and g["lower"][0] > -3.85)
check("the right column's lowest object, log I = 0, stays above the upper row", -1.5 - 0.3 > g["upper"][1])

# ---------------------------------------------------------------- the new test's source half, with plants
say("\n== the new test's conditions on the staged source, and plants")
ROW_U = "kit.text(stage, TEXTBOOK, FINE_SIZE, Palette::ink(), v(0.0, FINE_ROWS[0]))"
ROW_L = "kit.text(stage, SOURCE, FINE_SIZE, Palette::ink(), v(0.0, FINE_ROWS[1]))"


def t_film(source):
    film = source.split("#[cfg(test)]")[0]
    return film.count(ROW_U) == 1 and film.count(ROW_L) == 1


def t_prefix(source):
    return const_str(source, "SOURCE").startswith("precisely: ")


def no_stale_import(source):
    code = re.sub(r'"(?:[^"\\]|\\.)*"', '""', source)
    code = re.sub(r"//.*", "", code)
    return not re.search(r"\bprecisely\b", code)


check("source half passes on staged", t_film(new_text))
check("the prefix check passes on staged", t_prefix(new_text))
check("control: the source half fails on canonical", not t_film(base_text))
check("control: the prefix check fails on canonical", not t_prefix(base_text))
check("no stale precisely import remains, or Clippy would refuse it", no_stale_import(new_text))
check("control: canonical does use it", not no_stale_import(base_text))
others = sorted(p.name for p in (REPO / CRATE / "src/chapters").glob("*.rs") if p.name not in ("doing_nothing.rs", "draw.rs") and "precisely(" in p.read_text(encoding="utf8"))
check("draw::precisely keeps other callers, so it does not go dead", bool(others), ", ".join(others))

plants = [
    ("the upper row goes back to dim", ROW_U, ROW_U.replace("ink", "dim"), lambda s: t_film(s)),
    ("the lower row goes back to the helper", ROW_L, "precisely(&kit, stage, SOURCE)", lambda s: t_film(s)),
    ("a stale import returns", "pi_label, position_c, ring,", "pi_label, position_c, precisely, ring,", no_stale_import),
    ("the prefix is dropped", 'const SOURCE: &str = "precisely: no', 'const SOURCE: &str = "no', t_prefix),
    ("the condition loses its zero", "of A in (−∞, 0]", "of A in (−∞, 0)", words_kept),
    ("the upper row's words change", "imaginary part of an eigenvalue", "imaginary part of the eigenvalue", words_kept),
]
for label, old_s, new_s, test in plants:
    if old_s not in new_text.split(split)[0]:
        check(f"plant applies: {label}", False, "not in film code")
        continue
    check(f"planted, caught: {label}", not test(new_text.replace(old_s, new_s, 1)))
for label, rows in (
    ("the old rows, -3.15 and -3.62", [-3.15, -3.62]),
    ("rows onto the grid, -2.7 and -3.2", [-2.7, -3.2]),
    ("rows onto each other, -2.93 and -3.1", [-2.93, -3.1]),
):
    check(f"planted, caught by the numeric half: {label}", not t_numeric(rows))
check("planted, caught by the numeric half: 25 pt runs too wide", not t_numeric(ROWS, 25.0))

# ---------------------------------------------------------------- what binds this file
say("\n== bindings")
map13 = json.loads((REPO / CRATE / "docs/narration-direction-map-oliver13.json").read_text(encoding="utf8"))
holds = [h for h in map13["visual_holds"] if h["path"].endswith("doing_nothing.rs")]
check("map13 has one visual hold on this file", len(holds) == 1 and holds[0]["cue"] == "10_4", str(len(holds)))
hold = holds[0]


def interval(text, say_line, hold_line):
    return sha("\n".join(text.split("\n")[say_line - 1 : hold_line]))


audit = json.loads((STAGE06 / "bound-plan/hold-audit.json").read_text(encoding="utf8"))
reading = [r for r in audit["reading_holds"] if r["file"] == "doing_nothing.rs"]
check("hold-audit has the same one reading hold", len(reading) == 1 and reading[0]["marker_line"] == hold["line"])
say_line = reading[0]["say_line"]
old_interval = interval(base_text, say_line, hold["line"])
new_interval = interval(new_text, say_line, hold["line"])
check("the interval formula reproduces map13's value on canonical", old_interval == hold["interval_sha256"] == reading[0]["interval_sha256"], old_interval)
check("the hold's interval includes the two changed rows, so its hash must change", say_line < 594 and 595 < hold["line"] and old_interval != new_interval, new_interval)
check("the hold's line and marker survive", hold["marker"] in new_text.split("\n")[hold["line"] - 1], f"line {hold['line']}")
rows = [(key, r) for key in ("cues", "other_waits", "chapter_finishes") for r in audit[key] if r.get("file") == "doing_nothing.rs"]
lines = new_text.split("\n")
moved = []
for key, r in rows:
    marker = r.get("marker") or f'kit.say(stage, "{r["cue"]}")'
    if marker not in lines[r["line"] - 1]:
        moved.append((key, r["line"], marker))
check(f"all {len(rows)} other hold-audit rows on this file keep their line and marker", not moved, str(moved))
draw_hash = sha((REPO / CRATE / "src/chapters/draw.rs").read_bytes())
shared = {h["path"]: h["sha256"] for h in map13["shared_helpers"]}
check("draw.rs, a shared helper map13 names, is untouched by this patch", shared[f"{CRATE}/src/chapters/draw.rs"] == draw_hash, draw_hash[:16])
units = (REPO / CRATE / "narration_units.json").read_text(encoding="utf8")
check("narration_units.json does not name the source hash", BASE_HASH not in units)
inputs = (STAGE06 / "bound-plan/inputs.sha256").read_text(encoding="utf8")
check("stage06's bound-plan/inputs.sha256 names the old hash, so root's binder must take the new input", BASE_HASH in inputs)
check("script.tsv is outside the patch", "script.tsv" not in diff)

# ---------------------------------------------------------------- cue check on a scratch copy
say("\n== docs/check_cues.py on a scratch copy")
scr = SCRATCH / "cues" / CRATE
(scr / "docs").mkdir(parents=True)
shutil.copytree(REPO / CRATE / "src", scr / "src")
shutil.copyfile(REPO / CRATE / "script.tsv", scr / "script.tsv")
shutil.copyfile(REPO / CRATE / "docs/check_cues.py", scr / "docs/check_cues.py")
rc0, text0 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
shutil.copyfile(STAGED, scr / "src/chapters/doing_nothing.rs")
rc1, text1 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
say(f"  canonical copy exit {rc0}; staged copy exit {rc1}: {clean(text1).strip()[:160]}")
# it lists string literals with digits for review, so line 69's own literal is the one expected difference
only_a = [x.strip() for x in clean(text0).split("\n") if x not in clean(text1).split("\n")]
only_b = [x.strip() for x in clean(text1).split("\n") if x not in clean(text0).split("\n")]
check(
    "the cue checker passes both and differs only in its listing of the lower row's literal",
    rc0 == rc1 == 0 and only_a == [f'doing_nothing.rs:69: "{OLD_S}"'] and only_b == [f'doing_nothing.rs:69: "{NEW_S}"'],
    f"{len(only_a)} line against {len(only_b)}",
)
(scr / "src/chapters/doing_nothing.rs").write_text(new_text.replace('kit.say(stage, "10_4")?;', ""), encoding="utf8")
rc2, _ = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
check("control: it refuses the file with a say removed", rc2 != 0, f"exit {rc2}")

# ---------------------------------------------------------------- rustfmt
say("\n== rustfmt")
config = REPO / "rustfmt.toml"
for name, path in (("canonical", CANON), ("staged", STAGED)):
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", config, "--check", path])
    check(f"rustfmt --check {name}", rc == 0, clean(text).strip()[:200])

# ---------------------------------------------------------------- frames
say("\n== old frames")
rc, text = run(["python3", "-B", HERE / "checks/footnote_probe.py", "report", FRAMES / "frame-0229.png", OUT / "footnotes-before-after-NOT-A-RENDER.png"])
say(clean(text).strip())
check("footnote_probe.py exits 0 with its controls", rc == 0)

# ---------------------------------------------------------------- anchors, with values computed here
markers = {
    "schema": "logarithm10-footnote-anchors-v1",
    "file": REL,
    "baseline_sha256": BASE_HASH,
    "patched_sha256": sha(new),
    "film_lines_changed": changed,
    "film_lines_moved": 0,
    "timing_rule": "Anchor on the cue and sentence. Old times are stage06's scratch voice, not Oliver. Old frame numbers are labels in logarithm-stage06-frames01/10_doing_nothing, counted from 0.",
    "rows": {"upper": NEW_T, "lower": NEW_S},
    "map13_rebind": {
        "cue": hold["cue"],
        "next_cue": hold["next_cue"],
        "line": hold["line"],
        "marker": hold["marker"],
        "say_line": say_line,
        "old": {"sha256": BASE_HASH, "interval_sha256": old_interval},
        "new": {"sha256": sha(new), "interval_sha256": new_interval},
        "note": "Values only; map13 is not patched here. They are good only while doing_nothing.rs is exactly the staged file.",
    },
    "anchors": [
        {
            "id": "N1-both-rows",
            "cue": "10_4",
            "from": "sentence 4 (line 593), after its 0.5 s fade (line 596)",
            "until": "the chapter's end (line 604)",
            "old_frames": [200, 229],
            "old_time_s": [90.0, 103.183],
            "expect": "Two rows under the grid, both in ink at 22 pt, centred; checks/footnote_probe.py probe passes on a frame in this hold.",
            "fail_if": "either row grey; lower row under 0.40 scene units from the frame's edge; upper row under 0.15 from the grid; rows under 0.15 apart; a side past 6.86",
        },
        {
            "id": "N2-exact-words",
            "cue": "10_4",
            "from": "the same hold, on a final-size frame",
            "expect": "Exactly the strings in `rows`, read by eye: no clipped or fallback glyph in ⇒ − ∞ π.",
            "fail_if": "any difference in wording, signs or the citation",
        },
        {
            "id": "N3-reading-window",
            "cue": "10_4",
            "from": "shown_from (line 597) to the chapter's end",
            "expect": "At least FINE_PRINT_S, 13.2 s, on Oliver's real take: the source does not change this logic.",
            "fail_if": "shorter than 13.2 s",
        },
    ],
}
(HERE / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf8")

(OUT / "run_checks.txt").write_text(clean("\n".join(log)) + "\n", encoding="utf8")
print("\n" + ("all expectations hold" if not failures else "FAILED: " + "; ".join(failures)))
sys.exit(1 if failures else 0)
