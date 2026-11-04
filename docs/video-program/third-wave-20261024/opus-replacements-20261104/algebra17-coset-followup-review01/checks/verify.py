"""Checks for root's algebra17 coset bridge (candidate02). Nothing here builds, renders or listens.

Run from the repo root:  /usr/bin/python3 -I -B <this file>

This imports nothing from root's check or from any other review's scripts.

BIND    inputs, root's patch, script, units and map against the hashes in Mail 2049;
INDEX   the sentence each wait lands on, with mutations that must fail;
MATHS   my own permutations: S3 and the film's square, both sides, with negative controls;
MODEL   a replay of 17_12 and 17_13 at voice scales, controlled against the baseline cue sheet.
        No Oliver take of chapter 17 exists, so this is an estimate of pace, not sync;
PIXELS  baseline frames for slots and widths. No picture of candidate02 exists;
STAGED  the corrections in ../patches: round trip, rustfmt, commuting, hold line, test mirror.

Writes verify.out next to itself. Exit 1 on any failed check.
"""

import hashlib
import json
import pathlib
import re
import statistics
import struct
import subprocess
import sys
import tempfile
import wave
import zlib
from itertools import permutations, product

HERE = pathlib.Path(__file__).resolve().parent
BUNDLE = HERE.parent
O = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
A = pathlib.Path("explainer_corpus/abstract_algebra_expanded")
ROOT = pathlib.Path("narration_lab/third-wave-20261024/root")
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
T = TAKES / "applications-repair-stage01"
STAGE02 = TAKES / "combined-repair-stage02/explainer_corpus/abstract_algebra_expanded"
LAB = ROOT / "five-film-release-20261027"
PREVIEW = LAB / "algebra-combined-repair-preview01/render"
FRAMES = LAB / "algebra-combined-repair-frames01/17_applications"
C1C2 = O / "algebra17-patch-cross-review01/candidate-C1C2/applications.rs"
AFM_DIR = pathlib.Path("narration_lab/model-sidecar-20261017/.venv-cosyvoice3/lib/python3.10/site-packages/matplotlib/mpl-data/fonts/afm")
REL = "explainer_corpus/abstract_algebra_expanded/src/applications.rs"
SCRIPT_REL = "explainer_corpus/abstract_algebra_expanded/script.tsv"

BASE_SOURCE_SHA = "7cc3f0d08b610872"
CANDIDATE_SHA = "9d3ed41d35cc8879"
SOURCE_SHA = "93dd73b8526a6a54c7adddab3ff2ef32bd6e0c1bc4b8e90f4f9708c822be5019"
SCRIPT_SHA = "c065e38e031156703a6060a86f714f6d359293695833024791bb6f7c13293e50"
BASE_SCRIPT_SHA = "83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0"
MAP_SHA = "d188cc6d207f21ba84ed12bc6ed884eff8c1a8d9297f841c3cb9ee7de20ea803"
STAGE_MAP_SHA = "01c516b59aaec80449ba5ef67384f5ee6a0820072198eebc2435964ca122247d"

W1_OLD = "Finally, explain why multiplying a coset representative on the right by a subgroup element does not change that coset."
W1_NEW = "Finally, pick a left coset and explain why multiplying its representative on the right by a subgroup element does not change that coset."
W1_FALLBACK = "Finally, explain why multiplying a left coset's representative on the right by a subgroup element does not change that coset."
W2_OLD = "on the left of every result: the same elements form the same coset."
W2_NEW = "on the left of every result: by associativity, the same elements form the same coset."
PROMPT_OLD = "3. Why does changing representative within a coset keep it?"
PROMPT_NEW = "3. Why does gh name the same left coset as g, when h is in H?"

out = []
failed = 0


def say(text=""):
    out.append(text)
    print(text)


def check(name, ok, detail=""):
    global failed
    if not ok:
        failed += 1
    say(f"{'ok  ' if ok else 'FAIL'} {name}{' | ' + str(detail) if detail != '' else ''}")


def sha(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode()).hexdigest()


def rows_of(path):
    return {r.split("\t", 1)[0]: r.split("\t", 1)[1] for r in pathlib.Path(path).read_text(encoding="utf-8").splitlines() if "\t" in r}


def parse_patch(path):
    text = pathlib.Path(path).read_text(encoding="utf-8").split("\n")
    heads = [t for t in text if t.startswith(("--- ", "+++ "))]
    hunks = []
    i = 0
    while i < len(text):
        m = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", text[i])
        if not m:
            i += 1
            continue
        old_n = int(m.group(2) or 1)
        new_n = int(m.group(4) or 1)
        old, new = [], []
        i += 1
        while len(old) < old_n or len(new) < new_n:
            row = text[i]
            mark, body = row[:1], row[1:]
            if mark in (" ", ""):
                old.append(body)
                new.append(body)
            elif mark == "-":
                old.append(body)
            elif mark == "+":
                new.append(body)
            else:
                raise SystemExit(f"bad hunk row: {row!r}")
            i += 1
        hunks.append((int(m.group(1)), old, new))
    return heads, hunks


def apply(source, hunks, exact=True):
    lines = source.split("\n")
    shift = 0
    for start, old, new in hunks:
        at = start - 1 + shift
        if lines[at : at + len(old)] != old:
            if exact:
                raise ValueError(f"hunk does not match at line {start}")
            spots = [p for p in range(len(lines)) if lines[p : p + len(old)] == old]
            if len(spots) != 1:
                raise ValueError(f"hunk at {start}: {len(spots)} places")
            at = spots[0]
        lines[at : at + len(old)] = new
        shift = at + len(new) - (start - 1) - len(old)
    return "\n".join(lines)


def unapply(source, hunks):
    lines = source.split("\n")
    for start, old, new in hunks:
        at = start - 1
        if lines[at : at + len(new)] != new:
            raise ValueError(f"reverse: hunk at {start} does not match")
        lines[at : at + len(new)] = old
    return "\n".join(lines)


def rustfmt_ok(text, name):
    with tempfile.TemporaryDirectory() as tmp:
        path = pathlib.Path(tmp) / f"{name}.rs"
        path.write_text(text, encoding="utf-8")
        run = subprocess.run(
            ["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(path)],
            capture_output=True,
            text=True,
        )
        return run.returncode == 0, run.stdout[:300] + run.stderr[:300]


def sentence_ranges(text):
    """The kit's rule, written again: . ? or ! then a space or the end; trimmed."""
    spans, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            a = start
            while a <= i and text[a] == " ":
                a += 1
            if a <= i:
                spans.append((a, i + 1))
            start = i + 1
    return spans


def sentence_list(text):
    return [text[a:b] for a, b in sentence_ranges(text)]


def read_png(path):
    data = pathlib.Path(path).read_bytes()
    pos = 8
    idat = bytearray()
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos : pos + 8])
        chunk = data[pos + 8 : pos + 8 + n]
        if kind == b"IHDR":
            width, height, bits, ctype = struct.unpack(">IIBB", chunk[:10])
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + n
    if bits != 8 or ctype not in (2, 6):
        raise SystemExit(f"{path}: unsupported png {bits} {ctype}")
    bpp = 3 if ctype == 2 else 4
    raw = zlib.decompress(bytes(idat))
    stride = width * bpp
    rows = []
    prior = bytearray(stride)
    at = 0
    for _ in range(height):
        f = raw[at]
        line = bytearray(raw[at + 1 : at + 1 + stride])
        at += 1 + stride
        if f == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 255
        elif f == 2:
            for i in range(stride):
                line[i] = (line[i] + prior[i]) & 255
        elif f == 3:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prior[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b_ = prior[i]
                c = prior[i - bpp] if i >= bpp else 0
                pp = a + b_ - c
                pa, pb, pc = abs(pp - a), abs(pp - b_), abs(pp - c)
                pr = a if pa <= pb and pa <= pc else (b_ if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        rows.append(line)
        prior = line
    return width, height, bpp, rows


# ===================================================================== BIND
say("== BIND: inputs against Mail 2049 and earlier stages")
base_src = (T / "baseline/applications.rs").read_bytes()
cand_src = (T / "candidate/applications.rs").read_bytes()
cand2_src = (T / "candidate02/applications.rs").read_bytes()
check("baseline source", sha(base_src).startswith(BASE_SOURCE_SHA), sha(base_src))
check("candidate source", sha(cand_src).startswith(CANDIDATE_SHA), sha(cand_src))
check("candidate02 source, Mail 2049", sha(cand2_src) == SOURCE_SHA)
check("candidate02 script, Mail 2049", sha((T / "candidate02/script.tsv").read_bytes()) == SCRIPT_SHA)
check("baseline script", sha((T / "baseline/script.tsv").read_bytes()) == BASE_SCRIPT_SHA)
check("candidate is the C1C2 stack I already reviewed, byte for byte", cand_src == C1C2.read_bytes())
canon = (A / "src/applications.rs").read_bytes()
check("canonical source is still the baseline: root's bridge is not installed", canon == base_src)
check("canonical script is still the baseline", (A / "script.tsv").read_bytes() == (T / "baseline/script.tsv").read_bytes())
check("stage02 carries candidate02's source", (STAGE02 / "src/applications.rs").read_bytes() == cand2_src)
check("stage02 carries candidate02's script", (STAGE02 / "script.tsv").read_bytes() == (T / "candidate02/script.tsv").read_bytes())

say("\n== BIND: root's patch, candidate to candidate02")
PATCH = T / "coset-answer-followup.patch"
heads, hunks = parse_patch(PATCH)
check("one file, one hunk", len(hunks) == 1 and heads == ["--- candidate/applications.rs", "+++ candidate02/applications.rs"], heads)
walked = apply(cand_src.decode(), hunks)
check("my hunk walker reaches candidate02, compared in full", sha(walked) == SOURCE_SHA)
check("reverse returns candidate", sha(unapply(walked, hunks)).startswith(CANDIDATE_SHA))
with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    (tmp / "candidate").mkdir()
    target = tmp / "candidate/applications.rs"
    target.write_bytes(cand_src)
    pa = PATCH.resolve()
    run = subprocess.run(["patch", "-p0", "--fuzz=0", "-i", str(pa)], cwd=tmp, capture_output=True, text=True)
    check("patch(1) --fuzz=0 forward", run.returncode == 0 and sha(target.read_bytes()) == SOURCE_SHA, run.stdout[:160] if run.returncode else "")
    run = subprocess.run(["patch", "-R", "-p0", "--fuzz=0", "-i", str(pa)], cwd=tmp, capture_output=True, text=True)
    check("patch(1) --fuzz=0 reverse", run.returncode == 0 and sha(target.read_bytes()).startswith(CANDIDATE_SHA))
ok, detail = rustfmt_ok(cand2_src.decode(), "candidate02")
check("candidate02 passes rustfmt", ok, detail)
added = [r[1:] for r in PATCH.read_text().splitlines() if r.startswith("+") and not r.startswith("+++")]
check("the patch only adds: comment, wait, fade c, tex, fade, and removes nothing", len(added) == 5 and not [r for r in PATCH.read_text().splitlines() if r.startswith("-") and not r.startswith("---")], len(added))

say("\n== BIND: only two cue texts move, and only as Mail 2049 says")
base_rows = rows_of(T / "baseline/script.tsv")
cand_rows = rows_of(T / "candidate02/script.tsv")
moved = sorted(c for c in cand_rows if cand_rows[c] != base_rows.get(c))
check("rows changed", moved == ["17_12", "17_13"] and set(base_rows) == set(cand_rows), moved)
b12, c12 = sentence_list(base_rows["17_12"]), sentence_list(cand_rows["17_12"])
b13, c13 = sentence_list(base_rows["17_13"]), sentence_list(cand_rows["17_13"])
check("17_12 changes only its sentence 3", [i for i in range(5) if b12[i] != c12[i]] == [3] and c12[3] == W1_OLD)
check("17_13 adds one sentence at index 3, keeps the others in order", c13[:3] == b13[:3] and c13[4:] == b13[3:] and len(c13) == len(b13) + 1)
check("sentence 3 opens 'Now apply'", c13[3].startswith("Now apply"), c13[3])

say("\n== BIND: units and the proposed map")
units = json.loads((T / "candidate02/narration_units.json").read_text())
by_id = {u["id"]: u for u in units["units"]}
check("186 units", len(units["units"]) == 186)
for cue in ("17_12", "17_13"):
    u = by_id[f"oliver_{cue}"]
    check(f"unit {cue} text equals the script row", u["text"] == cand_rows[cue] and u["line_ids"] == [cue])
check("units name the script", units["script_sha256"] == SCRIPT_SHA)
map_path = T / "candidate02/narration-direction-map-applications-proposed.json"
check("proposed map is the one the units contract names", sha(map_path.read_bytes()) == units["direction_contract"]["map"]["sha256"] == MAP_SHA, sha(map_path.read_bytes()))
base_map = json.loads((T / "baseline/narration-direction-map-oliver24.json").read_text())
new_map = json.loads(map_path.read_text())
over_a = {json.dumps(x, sort_keys=True) for x in base_map["overrides"]}
over_b = {json.dumps(x, sort_keys=True) for x in new_map["overrides"]}
hold_a = {json.dumps(x, sort_keys=True) for x in base_map["visual_holds"]}
hold_b = {json.dumps(x, sort_keys=True) for x in new_map["visual_holds"]}
check("one override and one hold row differ from the baseline map", len(over_a - over_b) == len(over_b - over_a) == 1 and len(hold_a - hold_b) == len(hold_b - hold_a) == 1)
row = next(r for r in new_map["overrides"] if r.get("cue") == "17_12")
check("the 17_12 row's 'after' is the new cue, whole", row["after"] == cand_rows["17_12"])
check("the map holds no row for 17_13", not [r for r in new_map["overrides"] + new_map["visual_holds"] if r.get("cue") == "17_13"])
hold = next(h for h in new_map["visual_holds"] if h["cue"] == "17_12")
source_lines = cand2_src.decode().split("\n")
check("hold row binds candidate02, line 606, the marker", hold["sha256"] == SOURCE_SHA and hold["line"] == 606 and source_lines[605].strip() == hold["marker"], source_lines[605].strip())
check("line 607 is the authored 5 s, line 608 says 17_13", source_lines[606].strip() == "stage.wait(5.)?;" and 'say(stage, "17_13")' in source_lines[607])
stage_map_path = STAGE02 / "docs/narration-direction-map-oliver26.json"
check("stage02 map hash, its assembly note", sha(stage_map_path.read_bytes()) == STAGE_MAP_SHA)
stage_map = json.loads(stage_map_path.read_text())
stage_hold = next(h for h in stage_map["visual_holds"] if h["cue"] == "17_12")
check("stage02 repeats the same bindings", stage_hold == hold and stage_map["scripts"]["abstract_algebra_expanded"] == SCRIPT_SHA)

say("\n== BIND: takes")
wavs = [p for p in TAKES.rglob("*.wav") if re.search(r"(^|[^0-9])17[_-]\d", p.name)]
check("no chapter 17 WAV under root's algebra folder, by name, so no take is lost to a wording change", wavs == [], len(wavs))
coverage = json.loads((TAKES / "current-retained-coverage40.json").read_text())
check("control: the newest coverage file lists 17_12 and 17_13 as missing", {"17_12", "17_13"} <= set(coverage["missing_current_cues"]))
check("control: the same search does find takes for chapter 10", any(True for _ in TAKES.glob("10_1-take01.wav")))

# ===================================================================== INDEX
say("\n== INDEX: the kit's split, written again, against each wait")
src = cand2_src.decode()
segment_13 = src[src.index('k.say(stage, "17_13")') : src.index("k.finish(stage)")]
waits_13 = [int(j) for j in re.findall(r'k\.until_sentence\(stage, "17_13", (\d+)\)', segment_13)]
check("17_13 waits are 1, 2, 3 in that order", waits_13 == [1, 2, 3], waits_13)
check("17_12 has one wait, i + 1 over three prompts", len(re.findall(r'until_sentence\(stage, "17_12", i \+ 1\)', src)) == 1 and 'for (i, p) in prompts.iter().enumerate()' in src)
check("17_12 has five sentences", len(c12) == 5, len(c12))
check("17_13 has five sentences", len(c13) == 5, len(c13))
for cue, j, opening in (("17_12", 1, "Find the subgroup"), ("17_12", 2, "Decide whether"), ("17_12", 3, "Finally"), ("17_13", 1, "The four-clock"), ("17_13", 2, "And multiplying"), ("17_13", 3, "Now apply")):
    check(f"{cue} sentence {j} opens {opening!r}", sentence_list(cand_rows[cue])[j].startswith(opening))
check("no colon or semicolon splits a sentence, so 'result:' stays inside sentence 3", ":" in c13[3] and ";" in c12[4])
check("control: my split gives the baseline 17_13 four sentences", len(b13) == 4)
check("control: on the baseline text, index 3 would land on 'These are small', with no error from the kit", b13[3].startswith("These are small"))
check("no test in candidate02 names 17_12 or 17_13: nothing would catch that", not re.search(r'"17_1[23]"', src[src.index("mod tests") :]))


def mirror_pinned(lines):
    """Python mirror of the staged test: the wait table, and each opening."""
    for cue, j, opening in (("17_12", 1, "Find the subgroup"), ("17_12", 2, "Decide whether"), ("17_12", 3, "Finally"), ("17_13", 1, "The four-clock"), ("17_13", 2, "And multiplying"), ("17_13", 3, "Now apply")):
        spans = sentence_ranges(lines[cue])
        if j >= len(spans):
            return f"{cue} sentence {j} out of range"
        a, b_ = spans[j]
        if not lines[cue][a:b_].startswith(opening):
            return f"{cue} sentence {j}: {opening}"
    return None


check("test mirror passes on candidate02", mirror_pinned(cand_rows) is None, mirror_pinned(cand_rows))
check("test mirror fails on the baseline script", mirror_pinned(base_rows) is not None, mirror_pinned(base_rows))
split = dict(cand_rows)
split["17_13"] = cand_rows["17_13"].replace("the subgroup: closure", "the subgroup. Closure")
check("test mirror fails if a colon becomes a full stop", mirror_pinned(split) is not None, mirror_pinned(split))

# ===================================================================== MATHS
say("\n== MATHS: my own permutations, rightmost move first")


def compose(a, b_):
    return tuple(a[b_[i]] for i in range(len(b_)))


def left(g, H):
    return frozenset(compose(g, h) for h in H)


def right(H, g):
    return frozenset(compose(h, g) for h in H)


S3 = list(permutations(range(3)))
H3 = [(0, 1, 2), (1, 0, 2)]
e, r_, s_ = (0, 1, 2, 3), (1, 2, 3, 0), (0, 3, 2, 1)
D4 = {e}
while True:
    grown = D4 | {compose(x, y) for x in D4 for y in (r_, s_)}
    if grown == D4:
        break
    D4 = grown
D4 = sorted(D4)
H4 = [e, s_]
inverse = lambda a: next(b_ for b_ in D4 if compose(a, b_) == e)
check("the square has 8 moves", len(D4) == 8)
check("H = {e, s} is not normal in the square: left and right cosets differ", left(r_, H4) != right(H4, r_))
check("H = {swap, e} is not normal in S3", any(left(g, H3) != right(H3, g) for g in S3))
for label, G, H in (("S3", S3, H3), ("square", D4, H4)):
    n = len(G) * len(H)
    check(f"{label}: (gh)H = gH in all {n} cases", all(left(compose(g, h), H) == left(g, H) for g in G for h in H))
    check(f"{label}: hH = H", all(left(h, H) == frozenset(H) for h in H))
    check(f"{label}: (gh)H = g(hH), element by element", all(compose(compose(g, h), x) == compose(g, compose(h, x)) for g in G for h in H for x in H))
    wrong = sum(left(compose(h, g), H) != left(g, H) for g in G for h in H)
    check(f"{label}: wrong side, (hg)H = gH fails in {wrong} of {n}", wrong > 0)
    contrast = sum(right(H, compose(g, h)) != right(H, g) for g in G for h in H)
    check(f"{label}: RIGHT coset, H(gh) = Hg fails in {contrast} of {n}: the sentence is false for a right coset", contrast > 0)
    check(f"{label}: right cosets do absorb on the left, H(hg) = Hg", all(right(H, compose(h, g)) == right(H, g) for g in G for h in H))
check("the film's own coset: Hr (14_14 says it holds r and s r) has r and s r", right(H4, r_) == frozenset({r_, compose(s_, r_)}))
check("the film's own coset: multiply r on the right by s, and H(rs) is a different set", right(H4, compose(r_, s_)) != right(H4, r_), sorted(right(H4, compose(r_, s_))))
check("the same two letters on the left coset: (rs)H = rH", left(compose(r_, s_), H4) == left(r_, H4))
theirs = json.loads((T / "coset-orientation-check01.json").read_text())
mine = sorted(g for g in S3 for h in H3 if left(compose(h, g), H3) != left(g, H3))
check("my wrong-side list equals root's four counterexamples", mine == sorted(tuple(c["g"]) for c in theirs["wrong_side_counterexamples"]), mine)
check("root's cases count equals mine", theirs["right_multiplication_cases"] == len(S3) * len(H3) == theirs["passed"])

say("\n== MATHS: controls that must fail or differ")
not_subgroup = [e, r_]
check("control: {e, r} is not a subgroup, and hH = H fails", any(left(h, not_subgroup) != frozenset(not_subgroup) for h in not_subgroup))
check("control: with it, (gh)H = gH fails too", any(left(compose(g, h), not_subgroup) != left(g, not_subgroup) for g in D4 for h in not_subgroup))
closed_no_inverse = [(0, 1), (0, 0)]
check("control: a closed set of maps without inverses keeps results inside and still loses elements: the inverse clause earns its place", all(compose(a, b_) in closed_no_inverse for a in closed_no_inverse for b_ in closed_no_inverse) and left((0, 0), closed_no_inverse) != frozenset(closed_no_inverse))


def leftfirst(a, b_):
    return tuple(b_[a[i]] for i in range(len(a)))


lf = lambda g, H: frozenset(leftfirst(g, h) for h in H)
check("the other reading convention changes which moves are which, not the identity", all(lf(leftfirst(g, h), H4) == lf(g, H4) for g in D4 for h in H4))

# ===================================================================== MODEL
say("\n== MODEL: controls against the baseline cue sheet")
cue_rows = {r.split("\t")[0]: (float(r.split("\t")[1]), float(r.split("\t")[2])) for r in (PREVIEW / "17_applications.cues.tsv").read_text().splitlines() if "\t" in r}


def speech(text, scale=1.0):
    return scale * len(text.split()) / 2.6


def length(text, scale=1.0):
    return speech(text, scale) + 0.4


def starts(text, scale=1.0):
    parts = sentence_list(text)
    total = sum(len(p) for p in parts)
    at, before = [], 0
    for p in parts:
        at.append(speech(text, scale) * before / total)
        before += len(p)
    return at


FADE = 0.35
BREATH = 0.6
PICTURE_BEFORE_S1 = 4 * FADE + 2 * (0.8 + 0.4)
start12, len12 = cue_rows["17_12"]
start13, len13 = cue_rows["17_13"]
check(f"17_12 length {len12} is the scratch rule", abs(length(base_rows["17_12"]) - len12) < 0.02, length(base_rows["17_12"]))
check(f"17_13 length {len13} is the scratch rule", abs(length(base_rows["17_13"]) - len13) < 0.02, length(base_rows["17_13"]))
check("line + breath + 5 s reaches the next cue, to one frame", abs(start12 + len12 + BREATH + 5 - start13) < 0.034, start12 + len12 + BREATH + 5)


def padded_share(text, j, scale=1.0):
    """The rule my picture review and the C1C2 mirror used: the 0.4 s pad inside the share."""
    parts = sentence_list(text)
    return (speech(text, scale) + 0.4) * sum(len(p) for p in parts[:j]) / sum(len(p) for p in parts)


kit_text = pathlib.Path("crates/explainer_kit/src/narration.rs").read_text(encoding="utf-8")
check("the kit shares out speech only, with the pad left out", "estimated_starts(text, speech(text))" in kit_text and ".map(|(id, text)| (id.clone(), speech(text) + 0.4))" in kit_text)

say("\n== MODEL: which rule the render follows, on decoded frames of the baseline preview")
speech_rule = cue_rows["17_8"][0] + starts(base_rows["17_8"])[5]
pad_rule = cue_rows["17_8"][0] + padded_share(base_rows["17_8"], 5)
check("the source waits for 17_8 sentence 5, then fades the red label in at once", 'k.until_sentence(stage, "17_8", 5)?;\n        let limit = k.text(' in base_src.decode())
say(f"     17_8 sentence 5: speech rule {speech_rule:.3f}, padded rule {pad_rule:.3f}")
first_ink = None
with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    run = subprocess.run(
        ["ffmpeg", "-nostdin", "-v", "error", "-ss", "203.7", "-i", str(PREVIEW / "17_applications.mp4"), "-t", "1.1", "-start_number", "0", str(tmp / "f-%03d.png")],
        capture_output=True,
        text=True,
    )
    check("33 frames decoded, none kept", run.returncode == 0 and len(list(tmp.glob("f-*.png"))) == 33, run.stderr[:160])
    counts = []
    for n, path in enumerate(sorted(tmp.glob("f-*.png"))):
        width, height, bpp, rows = read_png(path)
        counts.append(sum(1 for y in range(612, 648) for x in range(200, 1080) if rows[y][x * bpp] > 40))
        if counts[-1] > 0 and first_ink is None:
            first_ink = 203.7 + n / 30
check(f"first ink at {first_ink:.3f}: the next frame after the speech rule, one frame", 0 <= first_ink - speech_rule <= 2 / 30 + 0.001)
check("the padded rule would be 0.3 s later than the ink, so it is not what renders", pad_rule - first_ink > 0.25)
check("control: no ink before the wait, and the whole label by the end", counts[0] == 0 and counts[-1] > 3000, (counts[0], counts[-1]))

say("\n== MODEL: my older numbers carry the pad, and the difference is small")
for j, want in ((1, 364.415), (2, 367.821), (3, 372.528)):
    got = start12 + padded_share(base_rows["17_12"], j)
    now = start12 + starts(base_rows["17_12"])[j]
    check(f"17_12 sentence {j}: picture review said {want}; padded rule gives {got:.3f}; the kit's rule gives {now:.3f}", abs(got - want) < 0.01)
got = start13 + padded_share(base_rows["17_13"], 2)
now = start13 + starts(base_rows["17_13"])[2]
check(f"17_13 sentence 2: picture review said 397.185; padded rule gives {got:.3f}; the kit's rule gives {now:.3f}", abs(got - 397.185) < 0.01)
for scale, want in ((0.75, 1.72), (0.85, 1.44), (1.0, 1.02), (1.07, 0.83), (1.25, 0.33)):
    parts = sentence_list(base_rows["17_13"])
    padded = (scale * len(base_rows["17_13"].split()) / 2.6 + 0.4 * scale) * len(parts[0]) / sum(len(p) for p in parts)
    got = PICTURE_BEFORE_S1 - padded
    now = PICTURE_BEFORE_S1 - starts(base_rows["17_13"], scale)[1]
    check(f"C1C2 text, scale {scale}: sentence 1 late {want} in that review; reproduced {got:.2f} with the pad scaled; {now:.2f} by the kit's rule", abs(got - want) < 0.011)


def replay(text12, text13, scale, keep=False):
    """Seconds from say(17_13) unless named; 'keep' is the staged patch A."""
    st12 = starts(text12, scale)
    st13 = starts(text13, scale)
    prompts_up = st12[3] + FADE
    gap = length(text12, scale) + BREATH + 5
    s1 = max(PICTURE_BEFORE_S1, st13[1])
    s1_done = s1 + FADE
    s2 = max(s1_done, st13[2])
    d_done = s2 + 2 * FADE
    s3 = max(d_done, st13[3])
    eq_done = s3 + (FADE if keep else 2 * FADE)
    end = length(text13, scale)
    at_phrase = st13[3] + speech(text13, scale) * text13[text13.index("Now apply") :].index("the same elements") / sum(len(p) for p in sentence_list(text13))
    return {
        "think": gap - prompts_up,
        "prompt3": gap - st12[3],
        "len12": length(text12, scale),
        "len13": end,
        "late1": s1 - st13[1],
        "late2": s2 - st13[2],
        "late3": s3 - st13[3],
        "eq_done": eq_done,
        "eq_stay": end + BREATH - eq_done,
        "phrase_after_eq": at_phrase - eq_done,
        "s3_len": speech(text13, scale) * len(sentence_list(text13)[3]) / sum(len(p) for p in sentence_list(text13)),
    }


TEXT_W1 = cand_rows["17_12"].replace(W1_OLD, W1_NEW)
TEXT_W2 = cand_rows["17_13"].replace(W2_OLD, W2_NEW)
say("\n== MODEL: candidate02 and the staged forms, by voice scale")
say("     scale   17_12    17_13  think  s1 late  s2 late  s3 late  eq stays  'the same elements' after eq")
for scale in (0.75, 0.85, 1.0, 1.07, 1.25):
    m = replay(cand_rows["17_12"], cand_rows["17_13"], scale)
    say(f"     {scale:5.2f} {m['len12']:7.2f} {m['len13']:8.2f} {m['think']:6.2f} {m['late1']:8.2f} {m['late2']:8.2f} {m['late3']:8.2f} {m['eq_stay']:9.2f} {m['phrase_after_eq']:8.2f}")
    check(f"scale {scale}: the sentence 1 lateness does not carry into sentence 2 or 3", m["late2"] == 0 and m["late3"] == 0)
    check(f"scale {scale}: at least 5 s of think time after the last prompt", m["think"] >= 5)
    check(f"scale {scale}: prompt 3 stays long enough at 15 characters a second, old wording", m["prompt3"] >= len(PROMPT_OLD) / 15, f"{m['prompt3']:.2f}")
    check(f"scale {scale}: the equation stays long enough at 15 characters a second", m["eq_stay"] >= len("(gh)H=g(hH)=gH") / 15, f"{m['eq_stay']:.2f}")
say("     with W1, W2 and patch A")
for scale in (0.75, 1.0, 1.07, 1.25):
    m = replay(TEXT_W1, TEXT_W2, scale, keep=True)
    say(f"     {scale:5.2f} {m['len12']:7.2f} {m['len13']:8.2f} {m['think']:6.2f} {m['late1']:8.2f} {m['late2']:8.2f} {m['late3']:8.2f} {m['eq_stay']:9.2f} {m['phrase_after_eq']:8.2f}")
    check(f"scale {scale}: W1 + W2 + A keep the same guarantees, with the new prompt", m["late2"] == 0 and m["late3"] == 0 and m["think"] >= 5 and m["prompt3"] >= len(PROMPT_NEW) / 15 and m["eq_stay"] >= len("(gh)H=g(hH)=gH") / 15)

base_chapter = length(base_rows["17_12"]) + length(base_rows["17_13"])
new_chapter = length(cand_rows["17_12"]) + length(cand_rows["17_13"])
staged_chapter = length(TEXT_W1) + length(TEXT_W2)
say(f"     chapter grows {new_chapter - base_chapter:.2f} s at scale 1 from candidate02; W1 + W2 add {staged_chapter - new_chapter:.2f} s more")
say(f"     new sentence 3: {len(c13[3].split())} words, {replay(cand_rows['17_12'], cand_rows['17_13'], 1.0)['s3_len']:.2f} s at scale 1")
for cue, text in (("17_12", cand_rows["17_12"]), ("17_13", cand_rows["17_13"]), ("17_12 W1", TEXT_W1), ("17_13 W2", TEXT_W2)):
    say(f"     {cue}: {len(text.split())} words")
    check(f"{cue} stays under 100 words and each sentence under 28", len(text.split()) < 100 and max(len(p.split()) for p in sentence_list(text)) <= 28, max(len(p.split()) for p in sentence_list(text)))

ratios = []
for cues in sorted(PREVIEW.glob("*.cues.tsv")):
    for r in cues.read_text().splitlines():
        if "\t" not in r:
            continue
        cue, _, scratch_len = r.split("\t")[:3]
        take = TAKES / f"{cue}-take01.wav"
        if take.is_file():
            with wave.open(str(take)) as w:
                ratios.append((w.getnframes() / w.getframerate()) / float(scratch_len))
say(f"     Oliver take01 over scratch length, this preview's other chapters: {len(ratios)} cues, min {min(ratios):.3f}, median {statistics.median(ratios):.3f}, max {max(ratios):.3f}")
check("the 0.75 to 1.25 scales cover the measured median, and chapter 17 itself is unmeasured", 0.75 < statistics.median(ratios) < 1.25)

# ===================================================================== PIXELS
say("\n== PIXELS: baseline frames; c, d, a, b sit at the same coordinates in candidate02")
for needle in (r'k.tex(stage, r"hH=H\quad(h\in H)", 32., Palette::green(), &[], v(2., -1.))', "v(0., -2.5)", "v(2., 1.4)", "v(2., 0.3)"):
    check(f"baseline and candidate02 share {needle[:34]}", needle in base_src.decode() and needle in src)


cache = {}


def frame(n):
    if n not in cache:
        cache[n] = read_png(FRAMES / f"frame-{n:04d}.png")
    return cache[n]


def px(x, y):
    return (x + 640 / 90) * 90, 360 - 90 * y


def ink_columns(n, x0, y0, x1, y1, floor=40):
    """Unit-coordinates ink box plus the sorted set of columns holding ink."""
    width, height, bpp, rows = frame(n)
    bg = rows[5][5 * bpp : 5 * bpp + 3]
    (a, top), (c, bottom) = px(x0, y1), px(x1, y0)
    a, c = max(int(a), 0), min(int(c), width - 1)
    top, bottom = max(int(top), 0), min(int(bottom), height - 1)
    found, cols = None, set()
    for y in range(top, bottom + 1):
        line = rows[y]
        for x in range(a, c + 1):
            i = x * bpp
            if abs(line[i] - bg[0]) + abs(line[i + 1] - bg[1]) + abs(line[i + 2] - bg[2]) > floor:
                cols.add(x)
                found = [x, y, x, y] if found is None else [min(found[0], x), min(found[1], y), max(found[2], x), max(found[3], y)]
    if found is None:
        return None, []
    return (found[0] / 90 - 640 / 90, (360 - found[3]) / 90, (found[2] + 1) / 90 - 640 / 90, (360 - found[1]) / 90), sorted(cols)


def ink_box(n, x0, y0, x1, y1, floor=40):
    return ink_columns(n, x0, y0, x1, y1, floor)[0]


AFM_TEXT = (AFM_DIR / "cmr10.afm").read_text()
WIDTH, KERN = {}, {}
NAMES = {"space": " ", "hyphen": "-", "comma": ",", "colon": ":", "question": "?", "period": ".", "parenleft": "(", "parenright": ")", "equal": "=", "zero": "0", "one": "1", "two": "2", "three": "3", "four": "4", "five": "5", "six": "6", "seven": "7", "eight": "8", "nine": "9"}
for m in re.finditer(r"^C -?\d+ ; WX ([\d.]+) ; N (\w+) ;", AFM_TEXT, re.M):
    WIDTH[NAMES.get(m.group(2), m.group(2))] = float(m.group(1))
for m in re.finditer(r"^KPX (\w+) (\w+) (-?[\d.]+)", AFM_TEXT, re.M):
    KERN[(NAMES.get(m.group(1), m.group(1)), NAMES.get(m.group(2), m.group(2)))] = float(m.group(3))
WIDTH.setdefault(" ", 333)


def law(text, size):
    total = sum(WIDTH[c] for c in text)
    total += sum(KERN.get((x, y), 0) for x, y in zip(text, text[1:]))
    return total / 1000 * size * 0.01


def afm_widths(name):
    return {m.group(2): float(m.group(1)) for m in re.finditer(r"^C -?\d+ ; WX ([\d.]+) ; N (\w+) ;", (AFM_DIR / name).read_text(), re.M)}


MI = afm_widths("cmmi10.afm")
SY = afm_widths("cmsy10.afm")
THICK = 1000 * 5 / 18


def tex_law(tokens, size):
    em = {"(": WIDTH["("], ")": WIDTH[")"], "=": WIDTH["="] + 2 * THICK, "in": SY["element"] + 2 * THICK}
    return sum(em.get(t, MI.get(t, 0)) for t in tokens) / 1000 * size * 0.01


N = 920
text_controls = [
    ("Answers you can test with the moves", 34, (-4.5, 2.7, 4.5, 3.3)),
    ("Four-clock: a move needs four repeats", 25, (-0.5, 1.2, 4.8, 1.6)),
    ("Two toggles: every move repeats in at most two", 24, (-0.6, 0.1, 4.8, 0.5)),
    ("Closure keeps it in. The inverse brings everything back.", 23, (-3.0, -2.8, 3.0, -2.2)),
]
for text, size, box in text_controls:
    found = ink_box(N, *box)
    measured = found[2] - found[0]
    predicted = law(text, size)
    check(f"text law, {text[:30]!r} {size} pt: law {predicted:.2f}, ink {measured:.2f}", abs(predicted - measured) < 0.08 + 0.02 * measured)
prompt_box = ink_box(836, -4.5, -0.75, 4.5, -0.25)
check(f"text law, the old prompt 3 at 27 pt on frame 836: law {law(PROMPT_OLD, 27):.2f}, ink {prompt_box[2] - prompt_box[0]:.2f}", abs(law(PROMPT_OLD, 27) - (prompt_box[2] - prompt_box[0])) < 0.2)

c_box, c_cols = ink_columns(N, 0.0, -1.4, 4.4, -0.6)
d_box = ink_box(N, -3.0, -2.8, 3.0, -2.2)
say(f"     c ink x {c_box[0]:.2f}..{c_box[2]:.2f}, y {c_box[1]:.2f}..{c_box[3]:.2f}")
say(f"     d ink x {d_box[0]:.2f}..{d_box[2]:.2f}, y {d_box[1]:.2f}..{d_box[3]:.2f}")
gaps = [(p, q) for p, q in zip(c_cols, c_cols[1:]) if q - p > 15]
check("the \\quad in hH=H (h in H) is the widest gap in c", len(gaps) == 1, gaps)
split_at = gaps[0]
left_w = (split_at[0] + 1 - c_cols[0]) / 90
right_w = (c_cols[-1] + 1 - split_at[1]) / 90
pred_left = tex_law(["h", "H", "=", "H"], 32)
pred_right = tex_law(["(", "h", "in", "H", ")"], 32)
check(f"TeX law, hH=H at 32 pt: law {pred_left:.2f}, ink {left_w:.2f}", abs(pred_left - left_w) < 0.08)
check(f"TeX law, (h in H) at 32 pt: law {pred_right:.2f}, ink {right_w:.2f}", abs(pred_right - right_w) < 0.08)
eq_w = tex_law(list("(gh)H=g(hH)=gH"), 32)
say(f"     (gh)H=g(hH)=gH at 32 pt, by the law: {eq_w:.2f} units")
eq_x = (2 - eq_w / 2, 2 + eq_w / 2)
c_mid = (c_box[1] + c_box[3]) / 2
Y_NEW = -1.75
eq_y = (c_box[1] - (-1) + Y_NEW, c_box[3] - (-1) + Y_NEW)
say(f"     at v(2, {Y_NEW}): x {eq_x[0]:.2f}..{eq_x[1]:.2f}, y {eq_y[0]:.2f}..{eq_y[1]:.2f}")


def afm_depth(name, glyph):
    m = re.search(rf"^C -?\d+ ; WX [\d.]+ ; N {glyph} ; B -?[\d.]+ (-?[\d.]+) -?[\d.]+ (-?[\d.]+) ;", (AFM_DIR / name).read_text(), re.M)
    return float(m.group(1)), float(m.group(2))


g_low, _ = afm_depth("cmmi10.afm", "g")
paren_low, paren_high = afm_depth("cmr10.afm", "parenleft")
_, big_h_high = afm_depth("cmmi10.afm", "H")
check(f"font metrics: g reaches {g_low}, the parenthesis {paren_low}; H rises to {big_h_high}, the parenthesis {paren_high}: c's height serves", g_low >= paren_low and big_h_high <= paren_high)
check("patch A's slot is empty on the baseline frame", ink_box(N, eq_x[0] - 0.1, eq_y[0] - 0.05, eq_x[1] + 0.1, eq_y[1] + 0.05) is None)
check("control: the same box raised to c's row does find ink", ink_box(N, eq_x[0] - 0.1, eq_y[0] + 0.75, eq_x[1] + 0.1, eq_y[1] + 0.75) is not None)
gap_c, gap_d = (c_box[1] - eq_y[1]) * 90, (eq_y[0] - d_box[3]) * 90
check(f"it clears c by {gap_c:.0f} px and d by {gap_d:.0f} px", gap_c >= 15 and gap_d >= 15)
check("it stays in the safe frame and right of the clock", eq_x[1] < 6.96 and eq_x[0] > -1.0)
clock_box = ink_box(N, -5.5, -2.1, -0.8, 2.3)
check(f"clock ink ends at x {clock_box[2]:.2f}, left of the equation by {(eq_x[0] - clock_box[2]) * 90:.0f} px", eq_x[0] - clock_box[2] > 0.3)

say("\n== PIXELS: the new prompt's width")
w_new, w_old = law(PROMPT_NEW, 27), law(PROMPT_OLD, 27)
w_2 = law("2. Can a four-clock be isomorphic to two two-position clocks?", 27)
say(f"     new {w_new:.2f}, old {w_old:.2f}, prompt 2 {w_2:.2f}; from x -3.6 the new one ends at {w_new - 3.6:.2f}")
check("it ends inside the safe frame", w_new - 3.6 < 6.96)
check("it is no wider than prompt 2 by more than half a unit", w_new <= w_2 + 0.5)
main_text = (A / "src/main.rs").read_text(encoding="utf-8")
glyphs = main_text[main_text.index("glyph_text: &[") : main_text.index("glyph_tex: &[")]
letters = set("".join(re.findall(r'"([^"]*)"', glyphs)))
new_chars = set(PROMPT_NEW) - set(PROMPT_OLD)
check("every character the new prompt adds is in the glyph list", new_chars <= letters, sorted(new_chars))
check("the one character outside the list, the full stop, is in the old prompt, which frame 836 shows", set(PROMPT_NEW) - letters == {"."} and "." in PROMPT_OLD)
check("the equation's letters are already typeset in chapter 14 and 17", all(x in (A / "src/cosets.rs").read_text() for x in (r"kH=g(hH)=gH", r"h\longmapsto gh")))

# ===================================================================== STAGED
say("\n== STAGED: each source patch against candidate02")
NAMES_P = {"A": "keep-the-step-in-view.patch", "B": "prompt-names-the-side.patch", "C": "waits-are-pinned.test.patch"}
parsed = {k: parse_patch(BUNDLE / "patches" / v)[1] for k, v in NAMES_P.items()}
results = {}
for key, name in NAMES_P.items():
    path = BUNDLE / "patches" / name
    head, _ = parse_patch(path)
    check(f"{key}: one file, a/ b/ paths", head == [f"--- a/{REL}", f"+++ b/{REL}"])
    result = apply(src, parsed[key])
    results[key] = result
    check(f"{key}: reverse returns candidate02", sha(unapply(result, parsed[key])) == SOURCE_SHA)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        target = tmp / REL
        target.parent.mkdir(parents=True)
        target.write_bytes(cand2_src)
        run = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", str(path.resolve())], cwd=tmp, capture_output=True, text=True)
        check(f"{key}: patch(1) --fuzz=0 forward equals my walker", run.returncode == 0 and target.read_text() == result)
        run = subprocess.run(["patch", "-R", "-p1", "--fuzz=0", "-i", str(path.resolve())], cwd=tmp, capture_output=True, text=True)
        check(f"{key}: patch(1) --fuzz=0 reverse", run.returncode == 0 and sha(target.read_bytes()) == SOURCE_SHA)
    ok, detail = rustfmt_ok(result, f"patch_{key}")
    check(f"{key}: passes rustfmt", ok, detail)
    lines = result.split("\n")
    check(f"{key}: the hold stays at line 606", lines[605].strip() == "k.hold(stage)?;" and [i + 1 for i, r in enumerate(lines) if r.strip() == "k.hold(stage)?;"].count(606) == 1)
    say(f"     {key}  {sha(result)}")

all_three = apply(apply(apply(src, parsed["A"], exact=False), parsed["B"], exact=False), parsed["C"], exact=False)
reversed_order = apply(apply(apply(src, parsed["C"], exact=False), parsed["B"], exact=False), parsed["A"], exact=False)
check("the three commute", all_three == reversed_order)
ok, detail = rustfmt_ok(all_three, "all_three")
check("all three pass rustfmt", ok, detail)
say(f"     A + B + C  {sha(all_three)}")
ok, _ = rustfmt_ok(all_three.replace('let same_coset = k.tex(stage, r"(gh)H=g(hH)=gH"', 'let same_coset = k.tex(stage,r"(gh)H=g(hH)=gH"'), "broken")
check("control: rustfmt refuses a misformatted copy", not ok)

say("\n== STAGED: what A, B and C change and leave")
check("A: no fade-out of c remains, and c still fades in once", "fade(stage, &[c], false)" not in results["A"] and results["A"].count("fade(stage, &[c], true)") == 1)
check("A: the equation is made once and shown once, at y -1.75", results["A"].count("let same_coset") == 1 and results["A"].count("fade(stage, &[same_coset], true)") == 1 and "v(2., -1.75)" in results["A"])
check("A: c is still bound, so nothing is left unused", "let c = k.tex" in results["A"])
check("B: only the third prompt's text moves", PROMPT_NEW in results["B"] and PROMPT_OLD not in results["B"] and results["B"].replace(PROMPT_NEW, PROMPT_OLD) == src)
check("B: still three prompts", len(re.findall(r'^\s+"\d\. ', results["B"], re.M)) >= 3)
test_block = results["C"][results["C"].index("fn quiz_and_answer_waits") :].split("#[test]")[0]
check("C: adds one test and touches no scene line", results["C"][: results["C"].index("mod tests")] == src[: src.index("mod tests")] and results["C"].count("#[test]") == src.count("#[test]") + 1)
table = re.findall(r'\("(17_1[23])", (\d), "([^"]+)"\)', test_block)
check("C: its table is the six waits, and each row holds on candidate02", len(table) == 6 and all(sentence_list(cand_rows[c])[int(j)].startswith(o) for c, j, o in table), len(table))
check("C: uses the same helpers as the test beside it", "use crate::axioms::{line_text, sentence_ranges};" in test_block and "sentence_ranges(text)[j]" in test_block)

say("\n== STAGED: the spoken wording, reporting only")
script_patch = BUNDLE / "patches/left-coset-wording.script.patch"
head, script_hunks = parse_patch(script_patch)
check("W1: script row only", head == [f"--- a/{SCRIPT_REL}", f"+++ b/{SCRIPT_REL}"] and len(script_hunks) == 1)
script_text = (T / "candidate02/script.tsv").read_text(encoding="utf-8")
w1_script = apply(script_text, script_hunks)
check("W1: reverse returns candidate02's script", sha(unapply(w1_script, script_hunks)) == SCRIPT_SHA)
with tempfile.TemporaryDirectory() as tmp:
    w1_path = pathlib.Path(tmp) / "script.tsv"
    w1_path.write_text(w1_script, encoding="utf-8")
    w1_rows = rows_of(w1_path)
check("W1: only 17_12 differs", sorted(c for c in w1_rows if w1_rows[c] != cand_rows[c]) == ["17_12"])
check("W1: only its sentence 3", [i for i, (x, y) in enumerate(zip(sentence_list(w1_rows["17_12"]), c12)) if x != y] == [3] and len(sentence_list(w1_rows["17_12"])) == 5)
check("W1: says left coset; the mirror still passes", "left coset" in w1_rows["17_12"] and mirror_pinned(w1_rows) is None)
w2_rows = dict(w1_rows)
w2_rows["17_13"] = TEXT_W2
check("W2: still five sentences, sentence 3 opens the same, the mirror still passes", len(sentence_list(TEXT_W2)) == 5 and mirror_pinned(w2_rows) is None)
fallback = cand_rows["17_12"].replace(W1_OLD, W1_FALLBACK)
check("W1 fallback: still five sentences", len(sentence_list(fallback)) == 5)
spoken = (T / "candidate02/script.tsv").read_text(encoding="utf-8")
check("no line of the film says \"coset's\", so the fallback's possessive is untried; W1 avoids it", "coset's" not in spoken)
check("associativity has been spoken before chapter 17, in 06_6: parentheses, same order", "Associativity changes the parentheses while keeping the moves in the same order" in cand_rows["06_6"])
check("14_14 names both sides before the question", "left and right cosets need not agree" in cand_rows["14_14"])
check("14_5 already says representatives 'name' a coset, so prompt B's verb has a precedent", "name the same coset" in cand_rows["14_5"])
check("14_15 already wrote k as g times a correction h", "write k as g times some correction h" in cand_rows["14_15"])
for text in (W1_NEW, W2_NEW):
    check(f"no digit, slash, bracket or dash in {text[:24]!r}", not re.search(r"[0-9/()\[\]—–]", text))
say(f"     W1 script         {sha(w1_script)}")
say(f"     W1 + W2 script    {sha(w1_script.replace(W2_OLD, W2_NEW))}")
check("W1 + W2 change exactly one row each", w1_script.count(W2_OLD) == 1)

say("\n== END")
check("canonical source still the baseline at the end of this run", (A / "src/applications.rs").read_bytes() == base_src)
check("candidate02 unchanged at the end of this run", sha((T / "candidate02/applications.rs").read_bytes()) == SOURCE_SHA)

say(f"\n{failed} FAIL")
(HERE / "verify.out").write_text("\n".join(out) + "\n", encoding="utf-8")
sys.exit(1 if failed else 0)
