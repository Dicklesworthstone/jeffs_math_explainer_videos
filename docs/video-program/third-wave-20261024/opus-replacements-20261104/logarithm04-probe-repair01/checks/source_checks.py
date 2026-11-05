"""Patch round trip, the hold binding and every source pin this edit could disturb. Writes source-markers.json only.

Run from the repo root:  python3 -I -B .../logarithm04-probe-repair01/checks/source_checks.py

The interval hash and the census follow the root's binder, combined-repair-stage06/bind_final80-stage06.py: scene code
is the text before #[cfg(test)], comment lines are skipped, and interval = lines[say_line - 1 : hold_line] joined by
newlines, sha256 of the UTF-8 bytes. Reproducing map13's stored values on the baseline is the control for that reading.
"""
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm04-probe-repair01")
CRATE = "explainer_corpus/logarithm_mysteries"
REL = f"{CRATE}/src/chapters/undoing_growth.rs"
MAP = f"{CRATE}/docs/narration-direction-map-oliver13.json"
SCRIPT_SHA = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"
SAY = re.compile(r'\bkit\.say\(stage, "(\d\d_\d+)"\)')
TIMING = re.compile(r"\b(predict_pause|predict|kit\.hold|kit\.music_breath|kit\.listen|kit\.sonify|kit\.mark|kit\.finish|stage\.wait|wait_frames|wait_until)\(")
SITE = {351: "super::draw::wait_until(stage, onset)?;"}
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def census(data):
    text = data.decode("utf-8")
    at = text.find("#[cfg(test)]")
    scene = text[:at]
    says, sites = {}, []
    for n, line in enumerate(scene.split("\n"), 1):
        if line.strip().startswith("//"):
            continue
        m = SAY.search(line)
        if m:
            says[m.group(1)] = n
        t = TIMING.search(line)
        if t and not re.search(r"\bfn (predict|wait_until)\(", line):
            sites.append((n, t.group(1), line.strip()))
    return text.split("\n"), scene, says, sites


base = Path(REL).read_bytes()
new = (O / REL).read_bytes()
blines, bscene, bsays, bsites = census(base)
nlines, nscene, nsays, nsites = census(new)

print("== script and neighbours")
check(sha(Path(f"{CRATE}/script.tsv").read_bytes()) == SCRIPT_SHA, f"script.tsv {SCRIPT_SHA[:16]}: not touched, every take's text stands")
files = sorted(p.relative_to(O).as_posix() for p in (O / "explainer_corpus").rglob("*") if p.is_file())
check(files == [REL], f"the candidate tree holds one file: {files}")

print("\n== the scene lines that change")
end = blines.index("#[cfg(test)]")
changed = [i + 1 for i in range(end - 1) if blines[i] != nlines[i]]
print("scene lines changed:", changed)
check(changed == [360, 402, 551], f"of lines 1 to {end - 1}, exactly 360, 402 and 551 differ, and all of them keep their numbers")
check(nlines[end - 3 : end - 1] == blines[end - 3 : end - 1] == ["    }", "}"], "the impl still closes on the same two lines")
check(bsays == nsays, f"all {len(bsays)} says on the same lines")
check(bsites == nsites, f"all {len(bsites)} timing sites identical, line and text: {[s[0] for s in bsites]}")
for line, marker in SITE.items():
    check(nlines[line - 1].strip() == marker, f"the binder's SITES row (undoing_growth.rs, {line}) still holds: {marker}")
check(nscene.count("phrase_marks(") == bscene.count("phrase_marks(") == 1, "phrase_marks uses stay at the binder's expected 1")
check(all(re.search(r'\bcfg\(test\)', l) is None for l in nscene.split("\n")), "no cfg(test) item before the tests module")
ANCHOR = re.compile(r'climb_onset\(text, said, s3, s4, &n\.phrase_marks\("04_2"\)\)|super::draw::wait_until\(stage, onset\)\?;')
ba = [i for i, l in enumerate(blines) if ANCHOR.search(l)]
check(ba == [i for i, l in enumerate(nlines) if ANCHOR.search(l)], f"04_2's phrase anchor sites stay on lines {[i + 1 for i in ba]}")
for needle in ("const EXP_NAME_X: f64 = 1.68;", 'kit.tex(stage, r"y = e^{x}", 34.0, green, &[], p(EXP_NAME_X, 2.4))?', "fade_out_all(stage, &[name, run0, rise0, floor, nowhere], 0.25)?;", "play!(stage; tween(mirror, 1.0, 2.6)?);"):
    check(bscene.count(needle) == 1 and nscene.count(needle) == 1, f"name and flip code unchanged: {needle}")
check("tween(c, 0.15, 2.4)" in nscene and "tween(c, -1.2, 1.4)" in nscene and "tween(c, PROBE_REST, 1.6)" in nscene, "the sweep's three legs keep 2.4, 1.4 and 1.6 s")

print("\n== hold rows of this file in map13")
the_map = json.loads(Path(MAP).read_bytes())
rows = [r for r in the_map["visual_holds"] if r["path"] == REL]
check(len(rows) == 1 and rows[0]["cue"] == "04_6", f"one hold binds this file: {[r['cue'] for r in rows]}")
row = rows[0]
check(row["sha256"] == sha(base), "control: the row's source hash is the baseline")
interval = lambda lines, says: sha("\n".join(lines[says[row["cue"]] - 1 : row["line"]]).encode("utf-8"))
check(interval(blines, bsays) == row["interval_sha256"], "control: my interval hash reproduces map13's on the baseline")
new_interval = interval(nlines, nsays)
check(new_interval == row["interval_sha256"], f"04_6's interval hash survives: {new_interval}")
check(nlines[row["line"] - 1].strip() == row["marker"], f"marker still on line {row['line']}: {row['marker']}")
print("old sha256", sha(base))
print("new sha256", sha(new))
print("line", row["line"], "unchanged; interval", new_interval, "unchanged; evidence.sha256 follows sha256")
helpers = {h["path"] for h in the_map["shared_helpers"]}
check(REL not in helpers, "this file is not one of map13's shared helpers")

print("\n== other pins of the old hash in canonical's crate docs (not changed here)")
old = sha(base)
pins = {}
for p in Path(f"{CRATE}/docs").rglob("*"):
    if p.is_file():
        n = p.read_bytes().count(old.encode())
        if n:
            pins[p.as_posix()] = n
for p, n in sorted(pins.items()):
    print(p, n)

print("\n== patch round trip on a scratch copy")
patch = O / "undoing_growth.patch"
with tempfile.TemporaryDirectory() as tmp:
    dest = Path(tmp) / REL
    dest.parent.mkdir(parents=True)
    dest.write_bytes(base)
    subprocess.run(["git", "init", "-q", tmp], check=True)
    done = subprocess.run(["git", "apply", "--check", str(patch.resolve())], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0, "git apply --check on the baseline")
    done = subprocess.run(["git", "apply", str(patch.resolve())], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0 and sha(dest.read_bytes()) == sha(new), "applying it gives the candidate's bytes")
    done = subprocess.run(["git", "apply", "-R", str(patch.resolve())], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0 and sha(dest.read_bytes()) == sha(base), "reversing it gives the baseline's bytes")

print("\n== format")
done = subprocess.run(["rustfmt", "--edition", "2024", "--check", "--config", "max_width=120", str(O / REL)], capture_output=True, text=True)
check(done.returncode == 0 and not done.stdout.strip(), "rustfmt --check, edition 2024, width 120")

changes = [
    ("360", "04_2", "the probe's start, the top of its whole path: under the name, over the tick label 2"),
    ("402", "04_2", "the probe's rest, half way between the tick labels 1 and 2; run time unchanged"),
    ("551", "04_4", "the standalone caption says about"),
]
markers = {
    "topic": "logarithm_mysteries",
    "source": {"path": REL, "baseline_sha256": sha(base), "patched_sha256": sha(new)},
    "line_count": {"baseline": len(blines) - 1, "patched": len(nlines) - 1},
    "scene_lines_changed": changed,
    "added_after_scene": {"from_line": end, "what": "PROBE_START and PROBE_REST, between the impl's closing brace and #[cfg(test)]"},
    "changes": [
        {
            "line": int(line),
            "cue": cue,
            "what": what,
            "old": blines[int(line) - 1].strip(),
            "new": nlines[int(line) - 1].strip(),
            "old_line_sha256": sha(blines[int(line) - 1].encode()),
            "new_line_sha256": sha(nlines[int(line) - 1].encode()),
        }
        for line, cue, what in changes
    ],
    "map13_hold_delta": [
        {
            "cue": row["cue"],
            "next_cue": row["next_cue"],
            "path": REL,
            "sha256": {"old": row["sha256"], "new": sha(new)},
            "evidence_sha256": {"old": row["evidence"]["sha256"], "new": sha(new)},
            "line": {"old": row["line"], "new": row["line"]},
            "marker": {"old": row["marker"], "new": row["marker"]},
            "interval_sha256": {"old": row["interval_sha256"], "new": new_interval},
        }
    ],
    "binder_sites_delta": [],
    "phrase_anchor_delta": [],
    "cue_text_delta": [],
}
(O / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print("\nRESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
