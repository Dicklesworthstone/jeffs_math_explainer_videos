"""Checks for the logarithm 11 gauge and footer repair. Source and mirrors only: no Cargo, no render.

Usage, from the repo root, with a scratch path that does not exist yet:
  /usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>

Reads canonical, stage06, root's frames and preview. Writes only checks/outputs/, the patch, source-markers.json and
the fresh scratch folder. It starts gauge_footer_probe.py and dense_readout.py with `python3 -B`, because they need
PIL; dense_readout.py also runs ffmpeg for sixty small crops. Canonical is never written.
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
REL = f"{CRATE}/src/chapters/unreachable.rs"
STAGED = HERE / REL
CANON = REPO / REL
LANE = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE06 = LANE / "combined-repair-stage06"
BASE_HASH = "a676dac78580ef73f0e658a21c0395cfe0505fe631bd6aa0d71b1f3ec05a86ee"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/11_unreachable"
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
for name in ("src/chapters/draw.rs", "src/facts.rs", "src/model.rs", "script.tsv", "oracle.py"):
    a, b = (REPO / CRATE / name).read_bytes(), (STAGE06 / CRATE / name).read_bytes()
    say(f"  {name}: canonical {sha(a)[:16]} stage06 {sha(b)[:16]} {'same' if a == b else 'DIFFER'}")
patch_path = HERE / "logarithm11-gauge-footers.patch"
diff = "".join(difflib.unified_diff(base_text.splitlines(keepends=True), new_text.splitlines(keepends=True), f"a/{REL}", f"b/{REL}", n=3))
patch_path.write_text(diff, encoding="utf8")
trial = SCRATCH / "apply"
(trial / REL).parent.mkdir(parents=True)
shutil.copyfile(CANON, trial / REL)
rc, text = run(["git", "apply", patch_path], cwd=trial)
check("the patch applies to canonical by git apply and gives the staged bytes", rc == 0 and sha((trial / REL).read_bytes()) == sha(new), clean(text).strip())
rc, text = run(["git", "apply", "-R", patch_path], cwd=trial)
check("and reverses", rc == 0 and sha((trial / REL).read_bytes()) == BASE_HASH, clean(text).strip())
say(f"staged unreachable.rs {sha(new)}")
say(f"patch {sha(patch_path.read_bytes())}")

# ---------------------------------------------------------------- what moved
say("\n== what the patch changes in film code")
split = "#[cfg(test)]\nmod tests"
film_a, film_b = base_text.split(split)[0], new_text.split(split)[0]
la, lb = film_a.split("\n"), film_b.split("\n")
KEEP = la.index("        kit.finish(stage)") + 3  # through the impl's closing brace
check("the film's code up to its closing brace keeps its line count", la[:KEEP] != [] and lb[KEEP - 3] == la[KEEP - 3] and lb[KEEP - 1] == "}", f"{KEEP} lines")
changed = [i + 1 for i, (x, y) in enumerate(zip(la[:KEEP], lb[:KEEP])) if x != y]
check("exactly the expected film lines change", changed == [64, 904, 905, 906, 914, 915, 916, 955, 1502, 1653], str(changed))
for i in changed:
    say(f"  {i}: - {la[i - 1].strip()[:110]}\n  {i}: + {lb[i - 1].strip()[:110]}")
added = lb[KEEP:]
check("what follows the film's code is only the footer helper", any("fn footer(" in x for x in added) and not any(re.search(r"kit\.say|until_|\.wait|fade_|play!", x) for x in added), f"{len(added)} lines")
timed = re.compile(r"(until_sentence|until_phrase|kit\.say|kit\.hold|stage\.wait|fade_in|fade_out|swap\(|tween|play!|finish|music_breath|predict\()")
# line 64 is the import list, which names tween_all and the fades without calling them
check("no timed call changes", [x for i, x in enumerate(la[:KEEP]) if timed.search(x) and i != 63] == [x for i, x in enumerate(lb[:KEEP]) if timed.search(x) and i != 63])

# ---------------------------------------------------------------- the words
say("\n== what the viewer reads")
strings = lambda s: re.findall(r'"((?:[^"\\]|\\.)*)"', s)  # noqa: E731
gone = sorted(set(strings("\n".join(la[:KEEP]))) - set(strings("\n".join(lb[:KEEP]))))
came = sorted(set(strings("\n".join(lb[:KEEP]))) - set(strings("\n".join(la[:KEEP]))))
check("the only film strings that change are the two notes", gone == ["t = 1: {}", "t = 3: {}"] and came == ["t = 1: ≈ {}", "t = 3: ≈ {}"], f"{gone} -> {came}")
FOOTERS = {
    "Liouville": ("for any real linear flow, steady or not, det = exp(∫ trace dt) > 0 (Liouville)", ["real linear flow", "steady or not", "> 0", "(Liouville)"], 955),
    "Culver": ("Culver, Proc. AMS 17 (1966) 1146–1151, as stated by Higham (Handbook of Linear Algebra) and Gallier (Thm 3.4)", ["Culver", "as stated by Higham", "Gallier (Thm 3.4)"], 1502),
    "Gallier": ("Gallier, arXiv 0805.0245, Theorem 3.3, proved via the Jordan form", ["Gallier", "Theorem 3.3", "Jordan form"], 1653),
}
for name, (words, needles, line) in FOOTERS.items():
    ok = all(lines[line + 2].strip() == f'"{words}",' for lines in (la, lb)) and all(n in words for n in needles)
    check(f"{name}: its words stay, byte for byte, with {', '.join(needles)}", ok)
check("the footer helper builds what draw::precisely built", 'format!("precisely: {words}")' in film_b and 'format!("precisely: {words}")' in (REPO / CRATE / "src/chapters/draw.rs").read_text(encoding="utf8"))
for frag in ("starts at 1, never touches 0: stays positive", "let one = ring(stage, gauge(1.0), 0.16, blue, 2.5);"):
    check(f"the identity start stays on screen: {frag}", frag in film_b)
script = (REPO / CRATE / "script.tsv").read_text(encoding="utf8")
check("and in the voice", "It starts at one." in script and "script.tsv" not in diff)

# ---------------------------------------------------------------- sheet 020's question
say("\n== root's question: 1.222 beside the note's 1.221 on sheet 020, frame 81")
X = [[0.3, -1.0], [0.8, -0.1]]
TR = X[0][0] + X[1][1]
check("the flow's matrix here is the one in the test module", "const FLOW: M2 = [[0.3, -1.0], [0.8, -0.1]];" in new_text)
d1, d3 = math.exp(TR * 1.0), math.exp(TR * 3.0)
say(f"  det = e^(t tr X), tr X = {TR:.1f}: at t = 1 {d1:.6f}, at t = 3 {d3:.6f}; largest printing error {max(abs(d1 - 1.221), abs(d3 - 1.822)):.6f}")
check("both notes print the correct rounding", f"{d1:.3f}" == "1.221" and f"{d3:.3f}" == "1.822")
flip = math.log(1.2215) / TR
say(f"  the readout first prints 1.222 at t = {flip:.5f}, just after the clock leaves 1")


def smooth(u):
    return u * u * u * (10 - 15 * u + 6 * u * u)


# read by eye from checks/outputs/readout-around-t1.png and -t3.png; the run to t = 3 is 2.0 s from :910
SEEN = {
    35.933: "1.221", 35.967: "1.222", 36.000: "1.222", 36.033: "1.223", 36.067: "1.224", 36.100: "1.226", 36.133: "1.228", 36.167: "1.231",
    37.600: "1.803", 37.633: "1.808", 37.667: "1.813", 37.700: "1.816", 37.733: "1.818", 37.767: "1.820", 37.800: "1.821", 37.833: "1.822", 38.567: "1.822",
}
START = 35.9  # the one fitted number
bad = []
for at, shown in SEEN.items():
    u = min(1.0, max(0.0, (at - START) / 2.0))
    value = math.exp(TR * (1.0 + 2.0 * smooth(u)))
    if f"{value:.3f}" != shown:
        bad.append((at, shown, f"{value:.4f}"))
check(f"the run to t = 3, starting at {START} s with the engine's smooth easing, reproduces all {len(SEEN)} readings", not bad, str(bad))
rate = (Path.home() / ".cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62/crates/fmn-anim/src/animation.rs").read_text(encoding="utf8")
check("smooth is an easing the engine has at the pinned commit", "pub fn smooth() -> Self" in rate)
say("  verdict: a transient. The clock left t = 1 at about 35.9 s and the readout left 1.221 at 35.967 s. The note is right; there is no endpoint mismatch.")
say("  at t = 3 the readout reaches 1.822 at 37.833 s, before the note's fade, and holds it through the sampled window")

# ---------------------------------------------------------------- geometry
say("\n== geometry, mirrored from widths and heights measured on frames 115, 550, 598")
FOOTER_Y = float(re.search(r"const FOOTER_Y: f64 = (-?[0-9.]+);", new_text).group(1))
EXTENTS = {"Liouville": (8.378, -3.756, -3.489, -3.62), "Culver": (12.267, -3.733, -3.500, -3.62), "Gallier": (7.467, -3.722, -3.511, -3.62)}
LOWEST = {"Liouville": -2.655, "Culver": -2.11, "Gallier": -2.62}


def fits(y):
    out = {}
    for name, (w, lo, hi, centre) in EXTENTS.items():
        out[name] = (lo - centre + y, hi - centre + y, w / 2)
    return out


def t_numeric(y):
    return all(lo > -3.6 and LOWEST[n] - hi > 0.3 and half < 7.11 - 0.25 for n, (lo, hi, half) in fits(y).items())


for name, (lo, hi, half) in fits(FOOTER_Y).items():
    say(f"  {name}: y {lo:.3f}..{hi:.3f}, half-width {half:.3f}; to the frame's edge {4 + lo:.3f}, was {4 + EXTENTS[name][1]:.3f}; to what stands lowest {LOWEST[name] - hi:.3f}")
check("the new test's numeric half holds, mirrored", t_numeric(FOOTER_Y))
check("inside the film's safe area ±6.96 × ±3.85", all(lo > -3.85 and half < 6.96 for lo, _, half in fits(FOOTER_Y).values()))
check("the rows' old bracket test is updated: its bound moves with the footer", "BRACKET_FOOT > FOOTER_Y + 0.2" in new_text and -1.65 - 0.46 > FOOTER_Y + 0.2)
check("the gauge test's -3.2 stays true of the gauge and clears the footers' tops", -2.655 > -3.2 and max(hi for _, hi, _ in fits(FOOTER_Y).values()) < -3.2)


def em(c):
    groups = {0.278: "il,.:;![]", 0.333: " -", 0.39: "rst()", 0.444: "acez", 0.5: "go^/0123456789", 0.528: "kqvxy", 0.556: "bdhnpu", 0.778: "=+−×±<>≈"}
    for value, chars in groups.items():
        if c in chars:
            return value
    return 0.75


for note in ("t = 1: ≈ 1.221", "t = 3: ≈ 1.822"):
    w = sum(em(c) for c in note) * 22 * 0.01
    say(f"  {note}: about {w:.3f} wide at 22 pt, so its right edge is near x = {1.2 + w:.3f}; the sampled frames below show that room empty")
    check(f"{note} stays clear of the mirror card at x = 4.7", 1.2 + w < 4.7)
say("  x-height by arithmetic, CM 0.43 em: 22 pt at 4K 25.5 px, 18 pt at 4K 20.9 px")

# ---------------------------------------------------------------- the new tests' source half, with plants
say("\n== the new tests' conditions on the staged source, and plants")


def t_notes(source):
    film = source.split("#[cfg(test)]")[0]
    return all(film.count(f'"t = {c}: ≈ {{}}", fact("matrix.flow_det_{k}")?),\n            22.0,\n            blue,') == 1 for c, k in (("1", "t1"), ("3", "t3")))


def t_footers(source):
    film = source.split("#[cfg(test)]")[0]
    return film.count(" = footer(") == 3 and " = precisely(" not in film and "Palette::ink(),\n        v(0.0, FOOTER_Y)," in film


def no_stale_import(source):
    code = re.sub(r'"(?:[^"\\]|\\.)*"', '""', source)
    code = re.sub(r"//.*", "", code)
    return not re.search(r"\bprecisely\b", code)


check("the notes' condition passes on staged", t_notes(new_text))
check("the footers' condition passes on staged", t_footers(new_text))
check("control: both fail on canonical", not t_notes(base_text) and not t_footers(base_text))
check("no stale precisely import remains, or Clippy would refuse it", no_stale_import(new_text))
check("control: canonical does use it", not no_stale_import(base_text))
others = sorted(p.name for p in (REPO / CRATE / "src/chapters").glob("*.rs") if p.name not in ("unreachable.rs", "draw.rs") and "precisely(" in p.read_text(encoding="utf8"))
check("draw::precisely keeps other callers, so it does not go dead", bool(others), ", ".join(others))
plants = [
    ("the t = 1 note loses its sign", '"t = 1: ≈ {}"', '"t = 1: {}"', t_notes),
    ("the t = 3 note goes back to 18 pt", 'fact("matrix.flow_det_t3")?),\n            22.0,', 'fact("matrix.flow_det_t3")?),\n            18.0,', t_notes),
    ("the t = 1 note goes back to dim", 'fact("matrix.flow_det_t1")?),\n            22.0,\n            blue,', 'fact("matrix.flow_det_t1")?),\n            22.0,\n            Palette::dim(),', t_notes),
    ("one footer goes back to the helper", "let cite = footer(", "let cite = precisely(", t_footers),
    ("the footers go back to dim", "Palette::ink(),\n        v(0.0, FOOTER_Y),", "Palette::dim(),\n        v(0.0, FOOTER_Y),", t_footers),
    ("a stale import returns", "path, position_c, predict, ring,", "path, position_c, precisely, predict, ring,", no_stale_import),
]
for label, old_s, new_s, test in plants:
    if old_s not in film_b:
        check(f"plant applies: {label}", False, "not in film code")
        continue
    check(f"planted, caught: {label}", not test(new_text.replace(old_s, new_s, 1)))
for label, y in (("the footer back at -3.62", -3.62), ("the footer up onto the film, at -2.8", -2.8)):
    check(f"planted, caught by the numeric half: {label}", not t_numeric(y))


def t_values(printed_1, printed_3):
    """The new test's numeric half: each note is the three-place rounding, and really is rounded."""
    return all(f"{d:.3f}" == p and 1e-6 < abs(float(p) - d) < 0.0005 for d, p in ((d1, printed_1), (d3, printed_3)))


check("the notes' numeric condition passes on the printed values", t_values("1.221", "1.822"))
check("planted, caught: the t = 1 note prints frame 81's 1.222 instead", not t_values("1.222", "1.822"))
check("planted, caught: the t = 3 note is truncated differently", not t_values("1.221", "1.823"))

# ---------------------------------------------------------------- what binds this file
say("\n== bindings")
map13 = json.loads((REPO / CRATE / "docs/narration-direction-map-oliver13.json").read_text(encoding="utf8"))
holds = [h for h in map13["visual_holds"] if h["path"].endswith("unreachable.rs")]
check("map13 has two visual holds on this file", len(holds) == 2, str([h["cue"] for h in holds]))
audit = json.loads((STAGE06 / "bound-plan/hold-audit.json").read_text(encoding="utf8"))
paired = {r["cue"]: r for key in ("prediction_holds", "reading_holds") for r in audit[key] if r["file"] == "unreachable.rs"}


def interval(text, say_line, hold_line):
    return sha("\n".join(text.split("\n")[say_line - 1 : hold_line]))


rebinds = []
for h in holds:
    say_line = paired[h["cue"]]["say_line"]
    old, now = interval(base_text, say_line, h["line"]), interval(new_text, say_line, h["line"])
    check(f"{h['cue']}: the interval formula reproduces map13's value on canonical", old == h["interval_sha256"] == paired[h["cue"]]["interval_sha256"], old[:16])
    check(f"{h['cue']}: its line and marker survive", h["marker"] in new_text.split("\n")[h["line"] - 1], f"line {h['line']}")
    say(f"  {h['cue']} -> {h['next_cue']}, lines {say_line}..{h['line']}: interval {'UNCHANGED' if old == now else 'CHANGES'}: {now}")
    rebinds.append({"cue": h["cue"], "next_cue": h["next_cue"], "kind": h["kind"], "line": h["line"], "marker": h["marker"], "say_line": say_line, "old": {"sha256": BASE_HASH, "interval_sha256": old}, "new": {"sha256": sha(new), "interval_sha256": now}})
by_cue = {r["cue"]: r for r in rebinds}
check("11_3's interval survives, since it starts after the edits at :955", by_cue["11_3"]["old"]["interval_sha256"] == by_cue["11_3"]["new"]["interval_sha256"])
check("11_20's interval changes, since it holds the Culver call at :1502", by_cue["11_20"]["old"]["interval_sha256"] != by_cue["11_20"]["new"]["interval_sha256"])
rows = [(key, r) for key in ("cues", "other_waits", "chapter_finishes") for r in audit[key] if r.get("file") == "unreachable.rs"]
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
check("stage06's bound-plan/inputs.sha256 names the old hash, so root's binder must take the new input", BASE_HASH in (STAGE06 / "bound-plan/inputs.sha256").read_text(encoding="utf8"))
check("script.tsv, facts.rs and oracle.py are outside the patch", all(n not in diff for n in ("script.tsv", "facts.rs", "oracle.py")))

# ---------------------------------------------------------------- cue check on a scratch copy
say("\n== docs/check_cues.py on a scratch copy")
scr = SCRATCH / "cues" / CRATE
(scr / "docs").mkdir(parents=True)
shutil.copytree(REPO / CRATE / "src", scr / "src")
shutil.copyfile(REPO / CRATE / "script.tsv", scr / "script.tsv")
shutil.copyfile(REPO / CRATE / "docs/check_cues.py", scr / "docs/check_cues.py")
rc0, text0 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
shutil.copyfile(STAGED, scr / "src/chapters/unreachable.rs")
rc1, text1 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
say(f"  canonical copy exit {rc0}; staged copy exit {rc1}: {clean(text1).strip()[:120]}")
# it lists string literals with digits for review, so the two notes' literals are the expected difference
only_a = sorted(x.strip() for x in clean(text0).split("\n") if x not in clean(text1).split("\n"))
only_b = sorted(x.strip() for x in clean(text1).split("\n") if x not in clean(text0).split("\n"))
say(f"  only on canonical {only_a}\n  only on staged {only_b}")
check(
    "the cue checker passes both and differs only in listing the two notes",
    rc0 == rc1 == 0 and only_a == ['unreachable.rs:904: "t = 1: {}"', 'unreachable.rs:914: "t = 3: {}"'] and only_b == ['unreachable.rs:904: "t = 1: ≈ {}"', 'unreachable.rs:914: "t = 3: ≈ {}"'],
)
(scr / "src/chapters/unreachable.rs").write_text(new_text.replace('kit.say(stage, "11_20")?;', ""), encoding="utf8")
rc2, _ = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
check("control: it refuses the file with a say removed", rc2 != 0, f"exit {rc2}")

# ---------------------------------------------------------------- rustfmt
say("\n== rustfmt")
config = REPO / "rustfmt.toml"
for name, path in (("canonical", CANON), ("staged", STAGED)):
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", config, "--check", path])
    check(f"rustfmt --check {name}", rc == 0, clean(text).strip()[:200])

# ---------------------------------------------------------------- frames
say("\n== dense crops from root's preview")
rc, text = run(["python3", "-B", HERE / "checks/dense_readout.py", SCRATCH / "dense"])
say(clean(text).strip())
check("dense_readout.py exits 0", rc == 0)
for name in ("readout-around-t1.png", "readout-around-t3.png"):
    shutil.copyfile(SCRATCH / "dense" / name, OUT / name)
say("\n== old frames")
rc, text = run(["python3", "-B", HERE / "checks/gauge_footer_probe.py", "report", FRAMES, OUT / "gauge-footers-NOT-A-RENDER.png"])
say(clean(text).strip())
check("gauge_footer_probe.py exits 0 with its controls", rc == 0)

# ---------------------------------------------------------------- anchors, with values computed here
markers = {
    "schema": "logarithm11-gauge-footer-anchors-v1",
    "file": REL,
    "baseline_sha256": BASE_HASH,
    "patched_sha256": sha(new),
    "film_lines_changed": changed,
    "film_lines_moved": 0,
    "timing_rule": "Anchor on the cue and sentence. Old times are stage06's scratch voice, not Oliver. Old frame numbers are labels in logarithm-stage06-frames01/11_unreachable, counted from 0.",
    "map13_rebind": rebinds,
    "rebind_note": "Values only; map13 is not patched here. They are good only while unreachable.rs is exactly the staged file.",
    "footers": {name: f"precisely: {words}" for name, (words, _, _) in FOOTERS.items()},
    "anchors": [
        {
            "id": "G1-t1-note",
            "cue": "11_2",
            "from": "sentence 3, after the fade at line 909",
            "until": "11_3's fade (line 993)",
            "old_frames": [81, 115, 150],
            "expect": "t = 1: ≈ 1.221, blue, 22 pt, level with where the dot stood at t = 1, clear of the ring at 1.",
            "not_a_failure": "The live readout leaving 1.221 just after the note has faded in: the clock has started its run to t = 3.",
            "fail_if": "no sign; grey; the readout printing anything but 1.221 while the clock stands at t = 1",
        },
        {
            "id": "G2-t3-note",
            "cue": "11_2",
            "from": "after the fade at line 919",
            "until": "11_3's fade (line 993)",
            "old_frames": [90, 115, 150],
            "expect": "t = 3: ≈ 1.822, blue, 22 pt, under the tick at 2; the readout holds 1.822 beside it until the backward run at line 925; checks/gauge_footer_probe.py notes passes on a frame in the hold.",
            "fail_if": "no sign; grey; touching the tick at 2 or det < 0's row",
        },
        {
            "id": "F1-liouville",
            "cue": "11_2 into 11_3",
            "from": "sentence 5, after the fade at line 960",
            "until": "line 993's fade",
            "old_frames": [112, 115, 150],
            "old_time_s": [51.5, 69.5],
            "expect": "The line in ink at y -3.4, with the words in footers.Liouville. The ring at 1 and starts at 1, never touches 0 stay above it. checks/gauge_footer_probe.py footer passes.",
            "fail_if": "grey; under 0.40 from the frame's edge; the integral's foot clipped; any change of words",
        },
        {
            "id": "F2-culver",
            "cue": "11_20",
            "from": "after the swap at line 1517",
            "until": "line 1600's fade",
            "old_frames": [491, 550],
            "old_time_s": 223.7,
            "expect": "The line in ink at y -3.4, with the words in footers.Culver; the probe passes.",
            "fail_if": "grey; under 0.40 from the edge; touching the rows; any change of words",
        },
        {
            "id": "F3-gallier",
            "cue": "11_8",
            "from": "sentence 2, after the fade at line 1658",
            "until": "the chapter's end (line 1676)",
            "old_frames": [573, 598],
            "old_time_s": [263.0, 273.783],
            "expect": "The line in ink at y -3.4, with the words in footers.Gallier; the probe passes.",
            "fail_if": "grey; under 0.40 from the edge; touching flattening matrices' row; any change of words",
        },
        {
            "id": "N-glyphs",
            "cue": "all of the above",
            "from": "a final-size frame in each hold",
            "expect": "Read by eye: the sign on both notes is a real ≈, as at line 1639, and ∫ – ½ in the footers are not fallback glyphs.",
            "fail_if": "a clipped or fallback glyph",
        },
    ],
}
(HERE / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf8")

(OUT / "run_checks.txt").write_text(clean("\n".join(log)) + "\n", encoding="utf8")
print("\n" + ("all expectations hold" if not failures else "FAILED: " + "; ".join(failures)))
sys.exit(1 if failures else 0)
