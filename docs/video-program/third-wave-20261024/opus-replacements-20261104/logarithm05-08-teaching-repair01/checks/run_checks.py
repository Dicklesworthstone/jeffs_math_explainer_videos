"""Checks for the logarithm 05-08 teaching repair. Source and mirrors only: no Cargo, no render.

Usage, from the repo root, with a scratch path that does not exist yet:
  /usr/bin/python3 -I -B <this folder>/checks/run_checks.py <fresh-scratch>

Reads root's stage07 candidate, stage06's hold audit, the sealed binder01, root's old frame lists, and
root's recognition log and take list. Writes only the patch, source-markers.json, hold-table.json,
phrase-anchors-proposed.json, binder-rows.json, checks/outputs/run_checks.txt and the fresh scratch folder.
It starts the other checks in this folder and runs rustfmt --check and git apply. Candidate and canonical
are never written. Nothing here compiles, renders, synthesizes or plays audio.
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
CHECKS = HERE / "checks"
OUT = CHECKS / "outputs"
OUT.mkdir(exist_ok=True)
CRATE = "explainer_corpus/logarithm_mysteries"
CH = f"{CRATE}/src/chapters"
LANE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
BASE = REPO / LANE / "combined-repair-stage07/candidate" / CRATE
AUDIT = REPO / LANE / "combined-repair-stage06/bound-plan/hold-audit.json"
RECOG_LOG = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-turns-recognition01/command.log"
RECOG_TAKES = REPO / LANE / "turns-recognition01/takes.json"
FRAMES = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01"
PATCH = HERE / "logarithm05-08-teaching.patch"
BASELINE = {
    "complex_turns.rs": "2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead",
    "lifting_loops.rs": "ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e",
    "branch_cut.rs": "193c4733de9f9081f5671700a6710dd76e989b41ab268824bbb0756313fbdbbd",
    "product_rule.rs": "e9019c828710207b806175a040aab936ee558b102d3cb55328078d42d897f7e0",
}
UNTOUCHED = {
    "src/chapters/draw.rs": "2a4e84504bab636433cd419b2c9990986d7c549bf05aa3432d33bd5a9b60fe28",
    "script.tsv": "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e",
}
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


def film(text):
    return text.split("#[cfg(test)]\nmod tests")[0]


staged = {name: (HERE / CH / name).read_bytes() for name in BASELINE}
base = {name: (BASE / "src/chapters" / name).read_bytes() for name in BASELINE}

# ---------------------------------------------------------------- baseline and patch
say("== baseline and patch")
for name, want in BASELINE.items():
    check(f"{name} in the candidate is the file this repair was written against", sha(base[name]) == want, want[:16])
for name, want in UNTOUCHED.items():
    check(f"{name} in the candidate is the bytes read", sha((BASE / name).read_bytes()) == want, want[:16])
listed = sorted(str(p.relative_to(HERE)) for p in (HERE / "explainer_corpus").rglob("*") if p.is_file())
check("the staged tree holds these four chapter files and nothing else", listed == sorted(f"{CH}/{n}" for n in BASELINE), f"{len(listed)} files")

parts = []
for name in BASELINE:
    rel = f"{CH}/{name}"
    parts.append("".join(difflib.unified_diff(base[name].decode("utf8").splitlines(keepends=True), staged[name].decode("utf8").splitlines(keepends=True), f"a/{rel}", f"b/{rel}", n=3)))
PATCH.write_text("".join(parts), encoding="utf8")
trial = SCRATCH / "apply"
for name in BASELINE:
    (trial / CH).mkdir(parents=True, exist_ok=True)
    (trial / CH / name).write_bytes(base[name])
rc, text = run(["git", "apply", PATCH], cwd=trial)
same = all((trial / CH / n).read_bytes() == staged[n] for n in BASELINE)
check("the patch applies to the candidate's four files by git apply and gives the staged bytes", rc == 0 and same, clean(text).strip())
rc, text = run(["git", "apply", "-R", PATCH], cwd=trial)
same = all(sha((trial / CH / n).read_bytes()) == BASELINE[n] for n in BASELINE)
check("and reverses to the baseline bytes", rc == 0 and same, clean(text).strip())
for name in BASELINE:
    say(f"staged {name} {sha(staged[name])}")
say(f"patch {sha(PATCH.read_bytes())}")

# ---------------------------------------------------------------- what moved
say("\n== what the patch changes")
film_rows = {}
for name in BASELINE:
    old, new = film(base[name].decode("utf8")).split("\n"), film(staged[name].decode("utf8")).split("\n")
    added, removed = 0, 0
    for row in difflib.unified_diff(old, new, lineterm="", n=0):
        if row.startswith("+") and not row.startswith("+++"):
            added += 1
        elif row.startswith("-") and not row.startswith("---"):
            removed += 1
    all_added = sum(1 for row in difflib.unified_diff(base[name].decode("utf8").split("\n"), staged[name].decode("utf8").split("\n"), lineterm="", n=0) if row.startswith("+") and not row.startswith("+++"))
    film_rows[name] = {"film_lines_added": added, "film_lines_removed": removed, "test_lines_added": all_added - added, "lines_old": len(base[name].decode("utf8").split("\n")) - 1, "lines_new": len(staged[name].decode("utf8").split("\n")) - 1}
    say(f"  {name}: {film_rows[name]}")
script_text = (BASE / "script.tsv").read_text(encoding="utf8")
for name in BASELINE:
    said_old = re.findall(r'kit\.say\(stage, "(\d\d_\d+)"\)', film(base[name].decode("utf8")))
    said_new = re.findall(r'kit\.say\(stage, "(\d\d_\d+)"\)', film(staged[name].decode("utf8")))
    check(f"{name}: the same cues are said, in the same order", said_old == said_new, f"{len(said_new)} cues")
    sent_old = re.findall(r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', film(base[name].decode("utf8")))
    sent_new = re.findall(r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', film(staged[name].decode("utf8")))
    check(f"{name}: every old sentence hook stays, in the same order", sent_old == sent_new, f"{len(sent_new)} hooks")
    check(f"{name}: no time from a take is added", "23.70" not in staged[name].decode("utf8") and "24.20" not in staged[name].decode("utf8"))

say("\n== rustfmt --check, the repo's settings")
for name in BASELINE:
    rc, text = run(["rustfmt", "--edition", "2024", "--config-path", "rustfmt.toml", "--check", HERE / CH / name], cwd=REPO)
    check(f"{name} is rustfmt-clean", rc == 0, clean(text).strip()[:300])

# ---------------------------------------------------------------- the other checks
say("\n== mirrors.py: the geometry and timing arithmetic")
rc, text = run([sys.executable, "-I", "-B", CHECKS / "mirrors.py", BASE / "script.tsv"], cwd=REPO)
say(clean(text).rstrip())
check("mirrors.py passes", rc == 0)

say("\n== source_conditions.py: the Rust tests' text conditions, with baseline controls and plants")
rc, text = run([sys.executable, "-I", "-B", CHECKS / "source_conditions.py", BASE / "src/chapters", HERE / CH, BASE / "script.tsv"], cwd=REPO)
say(clean(text).rstrip())
check("source_conditions.py passes", rc == 0)

say("\n== bindings.py: holds, finishes, anchors, the rows the binder needs")
rc, text = run([sys.executable, "-I", "-B", CHECKS / "bindings.py", BASE / "src/chapters", HERE / CH, BASE / "script.tsv", AUDIT, HERE], cwd=REPO)
say(clean(text).rstrip())
check("bindings.py passes", rc == 0)

say("\n== recognition_evidence.py: the 05_20 hooks against root's two word-row sets; provisional, no take selected")
rc, text = run([sys.executable, "-I", "-B", CHECKS / "recognition_evidence.py", RECOG_LOG, RECOG_TAKES, BASE / "script.tsv", HERE / CH / "complex_turns.rs"], cwd=REPO)
say(clean(text).rstrip())
check("recognition_evidence.py passes", rc == 0)

say("\n== binder_dry_run.py: binder01's build() in scratch mirrors")
rc, text = run([sys.executable, "-I", "-B", CHECKS / "binder_dry_run.py", SCRATCH / "binder"], cwd=REPO)
say(clean(text).rstrip())
check("binder_dry_run.py passes", rc == 0)

say("\n== docs/check_cues.py on a scratch copy of the candidate with these four files")
crate_copy = SCRATCH / "crate"
(crate_copy / "docs").mkdir(parents=True)
shutil.copytree(BASE / "src", crate_copy / "src")
shutil.copyfile(BASE / "script.tsv", crate_copy / "script.tsv")
shutil.copyfile(BASE / "docs/check_cues.py", crate_copy / "docs/check_cues.py")
for name in BASELINE:
    shutil.copyfile(HERE / CH / name, crate_copy / "src/chapters" / name)
rc, text = run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=crate_copy)
tail = [row for row in clean(text).splitlines() if re.search(r"(?i)\b(fail|error|ok|passed|refused)\b", row)]
say("\n".join(tail[-8:]))
check("check_cues.py passes on the scratch copy", rc == 0)

# ---------------------------------------------------------------- the native anchors
ANCHORS = [
    {
        "id": "C1-x", "repair": "C1", "chapter": "05_complex_turns", "cue": "05_20", "when": 'phrase "the logarithm of two"', "old_frames": [134, 136],
        "expect": r"Only e^{ln 2 + iy} on the right, blue, where the whole line will stand. No equals sign, no arrow, no 2i label. Lengths, angles and times-i have left at the sentence's start.",
        "fails_if": "The equals sign, the arrow or the 2i label shows, or the line stands before these words.",
    },
    {
        "id": "C1-y", "repair": "C1", "chapter": "05_complex_turns", "cue": "05_20", "when": 'phrase "a half pi"', "old_frames": [136],
        "expect": r"iy gives way to i pi/2 inside the exponent, and e^{ln 2 + stays still through the crossfade. Still no equals sign.",
        "fails_if": "The shared glyphs jump, the old and new exponents read as a smear for longer than the 0.3 s fade, or the answer shows.",
    },
    {
        "id": "C1-length", "repair": "C1", "chapter": "05_complex_turns", "cue": "05_20", "when": 'phrase "the length is two"', "old_frames": [150],
        "expect": "The words card at the left fades in. The line on the right holds, with no answer and no arrow.",
        "fails_if": "The card arrives before this phrase, or the arrow comes with it.",
    },
    {
        "id": "C1-land", "repair": "C1", "chapter": "05_complex_turns", "cue": "05_20", "when": 'phrase "land on two i", to the end of the cue', "old_frames": [158, 168],
        "expect": r"= 2i joins the line, with the left glyphs still. Then the arrow from 0 to 2i and its 2i label fade in during "
                  r'"two i". The finished picture matches old frame 168.',
        "fails_if": 'Any of the three pieces shows before "land", or they have not settled when the cue ends.',
    },
    {
        "id": "C1-out", "repair": "C1", "chapter": "05_complex_turns", "cue": "05_3", "when": "the cue's start", "old_frames": [168],
        "expect": "The worked line, card, arrow and label leave once, with everything else in the old list. The two stages do not flash back.",
        "fails_if": "A stage reappears for a frame or stays behind after the fade.",
    },
    {
        "id": "C2", "repair": "C2", "chapter": "05_complex_turns", "cue": "05_3", "when": "the climb past the strip's top", "old_frames": [215, 216, 217, 218, 227],
        "expect": "The input dot leaves the 3 pi tick and runs up the goes-on arrow, fading, and is gone at the arrow's tip before the laps read 2. The output winds on alone.",
        "fails_if": "The dot rests on the 3 pi tick while the laps read 2 or 3, leaves the arrow, or comes back.",
    },
    {
        "id": "L1-wait", "repair": "L1", "chapter": "06_lifting_loops", "cue": "06_6", "when": 'sentence 3, up to "passes minus one"', "old_frames": [267, 268, 269],
        "expect": "The walk stands short of -1 at turn 0.45, the gap gauge at zero, no ring at -1, no inset.",
        "fails_if": "The dot passes -1, the gauge moves, or the ring or inset shows.",
    },
    {
        "id": "L1-cross", "repair": "L1", "chapter": "06_lifting_loops", "cue": "06_6", "when": 'phrase "passes minus one"', "old_frames": [271, 273],
        "expect": "The red ring at -1 fades in, then the dot crosses and the gauge drops a lap, about 1 s after the phrase starts.",
        "fails_if": "The gauge drops before the phrase, or more than about 2 s after it.",
    },
    {
        "id": "L1-inset", "repair": "L1", "chapter": "06_lifting_loops", "cue": "06_6", "when": 'phrase "leaps from pi to minus pi", up to sentence 4', "old_frames": [271, 283],
        "expect": 'The jump arrow, its caption and axis fade in on these words. The walk finishes before "So that candidate breaks", where "it breaks at -1" appears.',
        "fails_if": 'The inset shows before "leaps", or the walk is still moving when sentence 4 starts.',
    },
    {
        "id": "B8", "repair": "B8", "chapter": "07_branch_cut", "cue": "07_5", "when": 'from "Or cut along a spiral"', "old_frames": [262, 265],
        "expect": "Zoomed at zero (compare the review's zooms/07-f0264-spiral.png): separate turns, about 0.9 scene units apart, and one strand leaving the ring round zero. The spiral still runs from zero to the frame's edge.",
        "fails_if": "Turns merge, the strand crosses itself, or it does not reach zero.",
    },
    {
        "id": "R10-dot", "repair": "R10", "chapter": "08_product_rule", "cue": "08_5", "when": 'sentence 3, "But the product"', "old_frames": [276, 287],
        "expect": "A gold dot at +1 fades in with the gold row, opposite the green dot at -1.",
        "fails_if": "The dot is absent, sits off the ring's +1, or arrives apart from its row.",
    },
    {
        "id": "R10-halving", "repair": "R10", "chapter": "08_product_rule", "cue": "08_5", "when": 'sentence 4, "Halving the lost lap"', "old_frames": [298],
        "expect": r"Only \tfrac{1}{2}(2 pi i) = pi i in purple, on the corner the whole line will stand on. No sign yet.",
        "fails_if": "e^{pi i} = -1 shows yet.",
    },
    {
        "id": "R10-arc", "repair": "R10", "chapter": "08_product_rule", "cue": "08_5", "when": 'phrase "leaves a half turn"', "old_frames": [298, 302],
        "expect": 'A purple arc with a head sweeps over the top of the ring from +1 to -1 in 1.6 s, then "half turn" fades in outside the ring at its upper right, clear of the ring, the axis, the definition and the column.',
        "fails_if": "The arc starts before the phrase, touches a dot, a green arc or a row of text, runs under the ring, or the word collides with anything. Its head must stop short of the green dot.",
    },
    {
        "id": "R10-sign", "repair": "R10", "chapter": "08_product_rule", "cue": "08_5", "when": 'phrase "a factor of minus one"', "old_frames": [310, 325],
        "expect": r", e^{pi i} = -1 joins the purple line with the left glyphs still, and the finished line matches old frame 325. Then the gold \sqrt{zw} row on sentence 5, as before.",
        "fails_if": "The sign shows before the phrase, the shared glyphs jump, or the finished line differs from the old string or place.",
    },
]
say("\n== old frames cited by the anchors, read from each chapter's sampling.json")
times = {}
for row in ANCHORS:
    if row["chapter"] not in times:
        times[row["chapter"]] = json.loads((FRAMES / row["chapter"] / "sampling.json").read_text(encoding="utf8"))["times"]
    samples = times[row["chapter"]]
    row["old_frames"] = [{"frame": f, "scratch_voice_seconds": round(samples[f], 3)} for f in row["old_frames"]]
    say(f"  {row['id']} {row['chapter']} " + ", ".join(f"f{f['frame']} {f['scratch_voice_seconds']:.3f}" for f in row["old_frames"]))
    check(f"{row['id']}: its cue is in the script", f"\n{row['cue']}\t" in "\n" + script_text)

hold_table = json.loads((HERE / "hold-table.json").read_text(encoding="utf8"))
anchors_doc = json.loads((HERE / "phrase-anchors-proposed.json").read_text(encoding="utf8"))
rows_doc = json.loads((HERE / "binder-rows.json").read_text(encoding="utf8"))
markers = {
    "schema": "logarithm05-08-teaching-anchors-v1",
    "baseline": {"candidate": "combined-repair-stage07/candidate", "files_sha256": BASELINE, "script_sha256": UNTOUCHED["script.tsv"], "draw_sha256": UNTOUCHED["src/chapters/draw.rs"]},
    "staged_sha256": {name: sha(staged[name]) for name in BASELINE},
    "patch_sha256": sha(PATCH.read_bytes()),
    "lines": film_rows,
    "timing_rule": "Anchor on the cue, sentence and phrase. Old times are stage06's scratch voice, not Oliver. Old frame numbers are labels in logarithm-stage06-frames01/<chapter>, counted from 0. Old frames show the state being replaced.",
    "provisional": "Without a word mark at a phrase's byte, a hook falls back on the phrase's character share of its measured sentence. That is a provisional estimate, not a heard boundary. measured_seconds stays null in phrase-anchors-proposed.json.",
    "hold_table": "hold-table.json",
    "phrase_anchors": "phrase-anchors-proposed.json",
    "binder_rows": "binder-rows.json",
    "phrase_hooks": [{k: a[k] for k in ("cue", "phrase", "byte", "byte_end", "sentence_index", "word_index", "measured_seconds")} for a in anchors_doc["anchors"]],
    "anchors": ANCHORS,
}
(HERE / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf8")
check("the markers name eight hooks, all unmeasured", len(markers["phrase_hooks"]) == 8 and all(h["measured_seconds"] is None for h in markers["phrase_hooks"]))
check("the binder rows hold five sets", sorted(rows_doc) == ["added_anchors", "fallback", "phrase_mark_uses", "receipts", "sites"], ", ".join(sorted(rows_doc)))
check("the hold table holds seven rows", len(hold_table["holds"]) == 7)

say("\nFAILURES: " + (", ".join(failures) if failures else "none"))
(OUT / "run_checks.txt").write_text(clean("\n".join(log)) + "\n", encoding="utf8")
sys.exit(1 if failures else 0)
