"""Checks for the logarithm 12 closing readability repair. Source and mirrors only: no Cargo, no render.

Usage, from the repo root, with a scratch path that does not exist yet:
  /usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>

Reads canonical, stage06, its bound plan and root's frames. Writes only checks/outputs/, the patch, source-markers.json
and the fresh scratch folder. It starts closing_probe.py with `python3 -B`, because that needs PIL. Canonical is never
written.
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
REL = f"{CRATE}/src/chapters/what_it_forgets.rs"
STAGED = HERE / REL
CANON = REPO / REL
STAGE06 = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06"
BASE_HASH = "e938b750ce579faed820ae7ddd0d289819f7148bdfb991dc99f24d35d6f8491b"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/12_what_it_forgets"
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
patch_path = HERE / "logarithm12-closing-readability.patch"
diff = "".join(difflib.unified_diff(base_text.splitlines(keepends=True), new_text.splitlines(keepends=True), f"a/{REL}", f"b/{REL}", n=3))
patch_path.write_text(diff, encoding="utf8")
trial = SCRATCH / "apply"
(trial / REL).parent.mkdir(parents=True)
shutil.copyfile(CANON, trial / REL)
rc, text = run(["git", "apply", patch_path], cwd=trial)
check("the patch applies to canonical by git apply and gives the staged bytes", rc == 0 and sha((trial / REL).read_bytes()) == sha(new), clean(text).strip())
rc, text = run(["git", "apply", "-R", patch_path], cwd=trial)
check("and reverses", rc == 0 and sha((trial / REL).read_bytes()) == BASE_HASH, clean(text).strip())
say(f"staged what_it_forgets.rs {sha(new)}")
say(f"patch {sha(patch_path.read_bytes())}")

# ---------------------------------------------------------------- what moved
say("\n== what the patch changes in film code")
SPLIT = "#[cfg(test)]\nmod tests"
film_a, film_b = base_text.split(SPLIT)[0], new_text.split(SPLIT)[0]
la, lb = film_a.split("\n"), film_b.split("\n")
KEEP = la.index("        kit.finish(stage)") + 3  # through the impl's closing brace
check("the film's code up to its closing brace keeps its line count", lb[KEEP - 3] == la[KEEP - 3] and lb[KEEP - 1] == "}", f"{KEEP} lines")
changed = [i + 1 for i, (x, y) in enumerate(zip(la[:KEEP], lb[:KEEP])) if x != y]
EXPECT = [50, 104, 141, 1244, 1245, 1246, 1251, 1266, 1273, 1283, 1297, 1298, 1299]
check("exactly the expected film lines change", changed == EXPECT, str(changed))
for i in changed:
    say(f"  {i}: - {la[i - 1].strip()[:110]}\n  {i}: + {lb[i - 1].strip()[:110]}")
added = lb[KEEP:]
check("what follows the film's code is only the footer helper", any("fn footer(" in x for x in added) and not any(re.search(r"kit\.say|until_|\.wait|fade_|play!", x) for x in added), f"{len(added)} lines")
TIMED = re.compile(r"(until_sentence|until_share|until_phrase|kit\.say|kit\.hold|stage\.wait|fade_in|fade_out|swap\(|redeal\(|steady\(|tween|play!|finish|music_breath|predict\()")


def timed_calls(text):
    return [x for x in text.split("\n") if TIMED.search(x)]


check("no timed call changes", timed_calls("\n".join(la[:KEEP])) == timed_calls("\n".join(lb[:KEEP])))

# ---------------------------------------------------------------- the words
say("\n== what the viewer reads")
strings = lambda s: re.findall(r'"((?:[^"\\]|\\.)*)"', s)  # noqa: E731
same_words = lambda a, b: strings(a) == strings(b)  # noqa: E731
check("every string in the film's code stays, byte for byte and in order", same_words("\n".join(la[:KEEP]), "\n".join(lb[:KEEP])))
CLOSING = [
    "how far each of these spins turns in one unit of time:",
    "the slowest two, ±π, tie on the window's open edge",
    "spins ending here: ±{}, ±{}, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)",
    "for the number −1, Log(−1) = iπ by convention",
    "here: the whole family",
]
for words in CLOSING:
    check(f"on screen still: {words}", f'"{words}"' in film_b)
FOOTER_TEXT = "precisely: spins ending here: ±½, ±1½, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)"
check("the footer helper builds what draw::precisely built", 'format!("precisely: {words}")' in film_b and 'format!("precisely: {words}")' in (REPO / CRATE / "src/chapters/draw.rs").read_text(encoding="utf8"))
check("the footer keeps its fact-derived laps", "half_laps(half_rate),\n                half_laps(lap_and_half)" in film_b)
for frag in ("for k in -3..=3_i64 {", "for k in [-3.0, -1.0, 1.0, 3.0] {", 'marks.push(kit.text(stage, "…", 20.0,'):
    check(f"the line keeps every mark and the family's dots: {frag}", frag in film_b)
check("script.tsv, facts.rs and oracle.py are outside the patch", all(n not in diff for n in ("script.tsv", "facts.rs", "oracle.py")))

# ---------------------------------------------------------------- room
say("\n== room, from extents measured on frame 378 and the staged centres")
num = lambda pattern: float(re.search(pattern, film_b).group(1))  # noqa: E731
STAGED_AT = {
    "how": num(r'unit of time:",\n\s+20\.0,\n\s+Palette::ink\(\),\n\s+v\(1\.4, (-?[0-9.]+)\)'),
    "line": num(r"const TURNING_Y: f64 = (-?[0-9.]+);"),
    "off": num(r"pi_label\(k\),\n\s+20\.0,\n\s+Palette::ink\(\),\n\s+v\(p\[0\], p\[1\] - ([0-9.]+)\)"),
    "lh": 0.139,
    "tie": num(r"open edge\",\n\s+20\.0,\n\s+gold,\n\s+v\(1\.4, (-?[0-9.]+)\)"),
    "conv": num(r"by convention\",\n\s+20\.0,\n\s+Palette::ink\(\),\n\s+v\(1\.4, (-?[0-9.]+)\)"),
    "ch": 0.198,
    "fam": num(r"const FAMILY_LABEL_Y: f64 = (-?[0-9.]+);"),
    "foot": num(r"const FOOTER_Y: f64 = (-?[0-9.]+);"),
}
OLD_AT = {"how": -1.1, "line": -1.6, "off": 0.3, "lh": 0.111, "tie": -2.3, "conv": -2.7, "ch": 0.178, "fam": -3.1, "foot": -3.62}
say(f"  staged centres {STAGED_AT}")


def stack(a):
    """Top and bottom of each row, top of frame first; heights measured on frame 378, 20 pt rows scaled from 16 and 18."""
    return [
        ("so", -0.089, -0.311),
        ("how far", a["how"] + 0.1, a["how"] - 0.1),
        ("rings on the line", a["line"] + 0.167, a["line"] - 0.167),
        ("labels", a["line"] - a["off"] + a["lh"] / 2, a["line"] - a["off"] - a["lh"] / 2),
        ("tie", a["tie"] + 0.1, a["tie"] - 0.1),
        ("convention", a["conv"] + a["ch"] / 2, a["conv"] - a["ch"] / 2),
        ("family", a["fam"] + 0.1, a["fam"] - 0.1),
        ("footer", a["foot"] + 0.1165 + 0.0035, a["foot"] - 0.1165 + 0.0035),
    ]


rows_old, rows_new = stack(OLD_AT), stack(STAGED_AT)
gaps = []
for k in range(len(rows_new) - 1):
    g_old, g_new = rows_old[k][2] - rows_old[k + 1][1], rows_new[k][2] - rows_new[k + 1][1]
    gaps.append((rows_new[k][0], rows_new[k + 1][0], g_old, g_new))
    say(f"  {rows_new[k][0]} to {rows_new[k + 1][0]}: {g_old:.3f} -> {g_new:.3f}")
edge_old, edge_new = 4 + rows_old[-1][2], 4 + rows_new[-1][2]
say(f"  footer to the frame's edge: {edge_old:.3f} -> {edge_new:.3f}")
say(f"  lowest of the rod and cube, y -2.556, to the footer: {-2.556 - rows_old[-1][1]:.3f} -> {-2.556 - rows_new[-1][1]:.3f}")
check("no two rows touch, and the smallest gap is the one under the rings, as it was", all(g[3] > 0.08 for g in gaps) and min(gaps, key=lambda g: g[3])[0] == "rings on the line")
check("the rows' gaps stay at least 0.2", all(g[3] >= 0.2 for g in gaps if g[0] != "rings on the line"))
check("the footer clears 5% of the height", edge_new >= 0.40)
check("the how-far row stays clear of the so row, by more than the rows' own gap", gaps[0][3] > 0.45)


def em(c):
    """The test module's table."""
    groups = {0.278: "il,.:;![]", 0.306: "jf", 0.333: " -", 0.361: "I", 0.39: "rst()", 0.444: "acez", 0.5: "go^/0123456789", 0.528: "kqvxy", 0.556: "bdhnpu", 0.625: "L", 0.722: "w", 0.778: "=+−×±<>", 0.833: "m"}
    for value, chars in groups.items():
        if c in chars:
            return value
    return 0.75


def width(s, size):
    return sum(em(c) for c in s) * size * 0.01


TEXT_RIGHT = 7.11 - 0.25
WIDTH_ROOM = 1.03
check("the convention at 20 pt fits the table's limit", 1.4 + width(CLOSING[3], 20.0) * WIDTH_ROOM < TEXT_RIGHT, f"right edge by table {1.4 + width(CLOSING[3], 20.0):.3f}; measured from 18 pt {1.4 + 3.744 * 20 / 18:.3f}")
check("the footer's width still fits", width(FOOTER_TEXT, 22.0) * WIDTH_ROOM < 2 * TEXT_RIGHT)
say(f"  footer: measured 12.622 wide at 22 pt; at 23 pt {12.622 * 23 / 22:.3f}, at 24 pt {12.622 * 24 / 22:.3f}, against the lint's {2 * TEXT_RIGHT:.2f}; {len(FOOTER_TEXT)} characters")
PI_LABELS = ["−3π", "−2π", "−π", "0", "π", "2π", "3π"]
tab = [width(s, 20.0) for s in PI_LABELS]
least = min(0.62 - (tab[i] + tab[i + 1]) / 2 for i in range(6))
check("the labels at 20 pt stay apart under marks 0.62 apart", least > 0.15, f"least gap by table {least:.3f}; widest measured at 16 pt 0.256, so 0.320 at 20 pt")


def t_numeric(a):
    """The test's numeric half, mirrored: the two const asserts and the probe's bounds."""
    foot = stack(a)[-1]
    label = stack(a)[3]
    return all(
        (
            a["how"] - 0.11 > a["line"] + 0.15,
            a["line"] - a["off"] - 0.09 > a["tie"] + 0.11,
            a["fam"] - 0.11 > a["foot"] + 0.13 + 0.15,
            foot[2] > -3.6,
            foot[1] < a["fam"] - 0.1 - 0.2,
            label[1] < a["line"] - 0.15 - 0.05,
            label[2] > a["tie"] + 0.1 + 0.15,
        )
    )


check("the test's numeric half holds on the staged centres, mirrored", t_numeric(STAGED_AT))
for label, change in (
    ("the footer back at -3.62", {"foot": -3.62}),
    ("the family's row left at -3.1, under the raised footer", {"fam": -3.1}),
    ("the line left at -1.6 with the rows raised", {"line": -1.6}),
    ("the labels pushed down onto the tie's row", {"off": 0.55}),
    ("the labels back up against the rings", {"off": 0.2}),
):
    check(f"planted, caught by the numeric half: {label}", not t_numeric({**STAGED_AT, **change}))

# ---------------------------------------------------------------- the new test's source half, with plants
say("\n== the new test's source half, and plants")


def t_source(source):
    film = source.split("#[cfg(test)]")[0]
    return all(
        (
            "pi_label(k),\n                20.0,\n                Palette::ink()," in film,
            '"…", 20.0, Palette::ink(), turning_at' in film,
            'by convention",\n            20.0,\n            Palette::ink(),' in film,
            film.count(" = footer(") == 1,
            " = precisely(" not in film,
            "Palette::ink(),\n        v(0.0, FOOTER_Y)," in film,
        )
    )


def no_stale_import(source):
    code = re.sub(r'"(?:[^"\\]|\\.)*"', '""', source)
    code = re.sub(r"//.*", "", code)
    return not re.search(r"\bprecisely\b", code)


check("its source half passes on staged", t_source(new_text))
check("control: it fails on canonical", not t_source(base_text))
check("no stale precisely import remains, or Clippy would refuse it", no_stale_import(new_text))
check("control: canonical does use it", not no_stale_import(base_text))
callers = sorted(p.name for p in (REPO / CRATE / "src/chapters").glob("*.rs") if p.name != "draw.rs" and "precisely(" in p.read_text(encoding="utf8"))
after_three = [n for n in callers if n not in ("what_it_forgets.rs", "doing_nothing.rs", "unreachable.rs")]
check("draw::precisely keeps callers once chapters 10, 11 and 12's patches land, so it does not go dead", bool(after_three), ", ".join(after_three))
for name in ("doing_nothing.rs", "unreachable.rs"):
    other = (REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104" / {"doing_nothing.rs": "logarithm10-footnote-repair01", "unreachable.rs": "logarithm11-gauge-repair01"}[name] / CRATE / "src/chapters" / name).read_text(encoding="utf8")
    check(f"and my sealed patch for {name} does drop it, so that list is right", "precisely(" not in other.split("#[cfg(test)]")[0].replace('format!("precisely:', ""))
plants = [
    ("the labels go back to 16 pt", "pi_label(k),\n                20.0,", "pi_label(k),\n                16.0,", t_source),
    ("the labels go back to dim", "pi_label(k),\n                20.0,\n                Palette::ink(),", "pi_label(k),\n                20.0,\n                Palette::dim(),", t_source),
    ("the dots go back to dim", '"…", 20.0, Palette::ink(),', '"…", 20.0, Palette::dim(),', t_source),
    ("the convention goes back to 18 pt", 'by convention",\n            20.0,', 'by convention",\n            18.0,', t_source),
    ("the convention goes back to dim", 'by convention",\n            20.0,\n            Palette::ink(),', 'by convention",\n            20.0,\n            Palette::dim(),', t_source),
    ("the footer goes back to the helper", "let source = footer(", "let source = precisely(", t_source),
    ("the footer goes back to dim", "Palette::ink(),\n        v(0.0, FOOTER_Y),", "Palette::dim(),\n        v(0.0, FOOTER_Y),", t_source),
    ("a stale import returns", "path, position_c, predict, ring,", "path, position_c, precisely, predict, ring,", no_stale_import),
]
for label, old_s, new_s, test in plants:
    if film_b.count(old_s) != 1:
        check(f"plant applies once: {label}", False, f"{film_b.count(old_s)} times in film code")
        continue
    check(f"planted, caught: {label}", not test(new_text.replace(old_s, new_s, 1)))
word_plant = "\n".join(lb[:KEEP]).replace("by convention", "by rule", 1)
check("planted, caught: a word changes on the convention", not same_words("\n".join(la[:KEEP]), word_plant))
timed_plant = "\n".join(lb[:KEEP]).replace('until_phrase(&kit, stage, "12_21", CONVENTION)?;', "", 1)
check("planted, caught: the convention's phrase wait goes", timed_calls(timed_plant) != timed_calls("\n".join(la[:KEEP])))
for stale in ("v(1.4, -1.1)", "v(1.4, -2.3)", "v(1.4, -2.7)", "-3.62"):
    check(f"no stale value remains in the new file: {stale}", stale not in new_text and stale in base_text)

# ---------------------------------------------------------------- what binds this file
say("\n== bindings")
map13 = json.loads((REPO / CRATE / "docs/narration-direction-map-oliver13.json").read_text(encoding="utf8"))
holds = [h for h in map13["visual_holds"] if h["path"].endswith("what_it_forgets.rs")]
check("map13 has one visual hold on this file", [h["cue"] for h in holds] == ["12_5"], str([h["cue"] for h in holds]))
audit = json.loads((STAGE06 / "bound-plan/hold-audit.json").read_text(encoding="utf8"))
paired = {r["cue"]: r for key in ("prediction_holds", "reading_holds") for r in audit[key] if r["file"] == "what_it_forgets.rs"}


def interval(text, say_line, hold_line):
    return sha("\n".join(text.split("\n")[say_line - 1 : hold_line]))


rebinds = []
for h in holds:
    say_line = paired[h["cue"]]["say_line"]
    old, now = interval(base_text, say_line, h["line"]), interval(new_text, say_line, h["line"])
    check(f"{h['cue']}: the interval formula reproduces map13's value on canonical", old == h["interval_sha256"] == paired[h["cue"]]["interval_sha256"], old[:16])
    check(f"{h['cue']}: its line and marker survive", h["marker"] in new_text.split("\n")[h["line"] - 1], f"line {h['line']}")
    check(f"{h['cue']}: its interval is unchanged, since the block's edits start after it", old == now, now)
    check(f"{h['cue']}: the file's two evidence hashes are the ones to rebind", h["sha256"] == h["evidence"]["sha256"] == BASE_HASH)
    say(f"  {h['cue']} -> {h['next_cue']}, lines {say_line}..{h['line']}: sha256 and evidence.sha256 {BASE_HASH[:16]} -> {sha(new)[:16]}; interval UNCHANGED {now}")
    rebinds.append({"cue": h["cue"], "next_cue": h["next_cue"], "kind": h["kind"], "line": h["line"], "marker": h["marker"], "say_line": say_line, "old": {"sha256": BASE_HASH, "evidence_sha256": BASE_HASH, "interval_sha256": old}, "new": {"sha256": sha(new), "evidence_sha256": sha(new), "interval_sha256": now}})
check("hold-audit: no reading hold names this file", not [r for r in audit["reading_holds"] if r["file"] == "what_it_forgets.rs"])
rows = [(key, r) for key in ("cues", "other_waits", "chapter_finishes") for r in audit[key] if r.get("file") == "what_it_forgets.rs"]
lines = new_text.split("\n")
moved = []
for key, r in rows:
    marker = r.get("marker") or f'kit.say(stage, "{r["cue"]}")'
    if marker not in lines[r["line"] - 1]:
        moved.append((key, r["line"], marker))
check(f"all {len(rows)} other hold-audit rows on this file keep their line and marker", len(rows) == 12 and not moved, str(moved))
anchors_doc = json.loads((STAGE06 / "bound-plan/phrase-anchors.json").read_text(encoding="utf8"))
anchors = [a for a in anchors_doc["anchors"] if any(s["file"] == "what_it_forgets.rs" for s in a["hook_sites"])]
bad_sites, phrase_rows = [], []
for a in anchors:
    for s in a["hook_sites"]:
        if s["file"] == "what_it_forgets.rs" and (a["const"] not in lines[s["line"] - 1] or s["call"] not in lines[s["line"] - 1]):
            bad_sites.append((a["phrase"], s["line"]))
    if a["const"] not in new_text or f'"{a["phrase"]}"' not in new_text:
        bad_sites.append((a["phrase"], "const"))
    phrase_rows.append({"cue": a["cue"], "phrase": a["phrase"], "lines": [s["line"] for s in a["hook_sites"] if s["file"] == "what_it_forgets.rs"]})
check(f"all {len(anchors)} phrase anchors on this file keep their hook line, const and words", len(anchors) == 5 and not bad_sites, str(bad_sites))
check("their source_sha256 names the old hash, so the binder must regenerate them", all(a["source_sha256"] == BASE_HASH for a in anchors))
shares = [r for r in anchors_doc["sentence_share_hooks"]["rows"] if r["file"] == "what_it_forgets.rs"]
share_bad = [r for r in shares if "until_share" not in lines[r["line"] - 1] or r["const"] not in lines[r["line"] - 1]]
check(f"the {len(shares)} sentence-share hooks on this file keep their lines and consts", len(shares) == 3 and not share_bad, str(share_bad))
say(f"  sentence-share rows on this file: {[(r['cue'], r['line'], r['const']) for r in shares]}")
draw_hash = sha((REPO / CRATE / "src/chapters/draw.rs").read_bytes())
shared = {h["path"]: h["sha256"] for h in map13["shared_helpers"]}
check("draw.rs, a shared helper map13 names, is untouched by this patch", shared[f"{CRATE}/src/chapters/draw.rs"] == draw_hash and "draw.rs" not in diff, draw_hash[:16])
units = (REPO / CRATE / "narration_units.json").read_text(encoding="utf8")
check("narration_units.json does not name the source hash", BASE_HASH not in units)
NAMING = [
    f"{CRATE}/docs/narration-direction-map-oliver13.json",
    f"{CRATE}/docs/narration-direction-map-oliver12.json",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/hold-audit.json",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/inputs.sha256",
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/phrase-anchors.json",
    f"narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage06/bound-plan/{CRATE}/docs/narration-direction-map-oliver13.json",
]
for rel in NAMING:
    check(f"names the old hash, so it needs the new input: {rel.split('combined-repair-stage06/')[-1]}", BASE_HASH in (REPO / rel).read_text(encoding="utf8"))

# ---------------------------------------------------------------- cue check on a scratch copy
say("\n== docs/check_cues.py on a scratch copy")
scr = SCRATCH / "cues" / CRATE
(scr / "docs").mkdir(parents=True)
shutil.copytree(REPO / CRATE / "src", scr / "src")
shutil.copyfile(REPO / CRATE / "script.tsv", scr / "script.tsv")
shutil.copyfile(REPO / CRATE / "docs/check_cues.py", scr / "docs/check_cues.py")
rc0, text0 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
shutil.copyfile(STAGED, scr / "src/chapters/what_it_forgets.rs")
rc1, text1 = run(["/usr/bin/python3", "-I", "-B", scr / "docs/check_cues.py"], cwd=scr)
say(f"  canonical copy exit {rc0}; staged copy exit {rc1}: {clean(text1).strip()[:120]}")
check("the cue checker passes both, with identical output", rc0 == rc1 == 0 and clean(text0) == clean(text1))
(scr / "src/chapters/what_it_forgets.rs").write_text(new_text.replace('kit.say(stage, "12_21")?;', ""), encoding="utf8")
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
rc, text = run(["python3", "-B", HERE / "checks/closing_probe.py", "report", FRAMES, OUT / "closing-block-NOT-A-RENDER.png"])
say(clean(text).strip())
check("closing_probe.py exits 0 with its controls", rc == 0)

# ---------------------------------------------------------------- anchors, with values computed here
markers = {
    "schema": "logarithm12-closing-anchors-v1",
    "file": REL,
    "baseline_sha256": BASE_HASH,
    "patched_sha256": sha(new),
    "film_lines_changed": changed,
    "film_lines_moved": 0,
    "timing_rule": "Anchor on the cue and phrase. Old times are stage06's scratch voice, not Oliver. Old frame numbers are labels in logarithm-stage06-frames01/12_what_it_forgets, counted from 0.",
    "map13_rebind": rebinds,
    "phrase_anchor_rebind": {"source_sha256": {"old": BASE_HASH, "new": sha(new)}, "rows": phrase_rows, "bytes_and_lines": "unchanged"},
    "rebind_note": "Values only; map13 and the bound plan are not patched here. They are good only while what_it_forgets.rs is exactly the staged file.",
    "staged_centres": STAGED_AT,
    "footer": FOOTER_TEXT,
    "anchors": [
        {
            "id": "L1-labels-and-dots",
            "cue": "12_21",
            "from": "after the fades at lines 1275-1277",
            "until": "the chapter's end (line 1309)",
            "old_frames": [337, 340, 378],
            "expect": "−3π … 3π in ink at 20 pt, under the marks at y -1.72; both runs of dots in ink; the rings clear of −π and π.",
            "fail_if": "grey; a label touching a ring, a neighbour or the tie row; a missing mark",
        },
        {
            "id": "F1-footer",
            "cue": "12_21",
            "from": "sentence 2, after the fade at line 1292",
            "until": "the chapter's end (line 1309)",
            "old_frames": [360, 378],
            "old_time_s": [157.5, 165.016],
            "expect": "The line in ink at y -3.4, with the exact words in footer; ±6.31 wide, inside the safe area.",
            "fail_if": "grey; under 0.40 from the frame's edge; touching the family's row; any change of words",
        },
        {
            "id": "C1-convention",
            "cue": "12_21",
            "from": "a convention broke the same tie, after the fade at line 1301",
            "until": "the chapter's end (line 1309)",
            "old_frames": [364, 378],
            "expect": "for the number −1, Log(−1) = iπ by convention, in ink at 20 pt, at y -2.5, left edge x 1.4.",
            "fail_if": "grey; touching the tie or family rows; any change of words",
        },
        {
            "id": "S1-the-whole-stack",
            "cue": "12_21",
            "from": "the honest answer is the whole family, after the fade at line 1308",
            "until": "the chapter's end (line 1309)",
            "old_frames": [370, 378],
            "expect": "From the top: so, how far, the line, labels, the tie, the convention, the family, the footer; no two touching; checks/closing_probe.py closing passes.",
            "fail_if": "any two rows touching; the how-far row reading as part of the so row; the family's row on the footer",
        },
        {
            "id": "N-glyphs",
            "cue": "12_21",
            "from": "a final-size frame after line 1308",
            "expect": "Read by eye: π, −, ½, ±, §, … and the exact words. No new glyph is asked for; these are the same characters at new sizes.",
            "fail_if": "a clipped or fallback glyph; any change of words",
        },
    ],
}
(HERE / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf8")

(OUT / "run_checks.txt").write_text(clean("\n".join(log)) + "\n", encoding="utf8")
print("\n" + ("all expectations hold" if not failures else "FAILED: " + "; ".join(failures)))
sys.exit(1 if failures else 0)
