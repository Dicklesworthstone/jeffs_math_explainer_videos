"""Source and binding checks for the F2 follow-on. Reads only.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-oliver-timing01/checks/source_checks.py

The binder and preflight are read, never run. Their regexes are copied here from combined-repair-stage06, which may
have moved on by the time root rebinds.
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
OPUS = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = OPUS / "logarithm-opening-oliver-timing01"
SEALED = OPUS / "logarithm-opening-timing-repair01"
R = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE07 = R / "combined-repair-stage07/candidate"
CRATE = "explainer_corpus/logarithm_mysteries"
FILE = f"{CRATE}/src/chapters/folded_decade.rs"
BASELINE = "c4f023ce5284dd768767b28c9d68a4da1e05c40385eb750dc9d4fbe583b6347e"
SUMS = "63a3fa29b3fbd7aedfbacee9d77a274f51ea5ed2f95d9b04fd2d5ffad61e0f5e"
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(data):
    return hashlib.sha256(data).hexdigest()


print("-- the sealed bundle is unchanged")
raw = (SEALED / "SHA256SUMS").read_bytes()
check(sha(raw) == SUMS, f"sealed SHA256SUMS is {SUMS[:16]}")
rows = [line.split("  ", 1) for line in raw.decode().splitlines()]
bad = [name for value, name in rows if sha((SEALED / name).read_bytes()) != value]
check(not bad, f"all {len(rows)} sealed files match it {bad}")

print("\n-- baseline")
base = (SEALED / FILE).read_bytes()
cand = (HERE / FILE).read_bytes()
check(sha(base) == BASELINE, f"sealed folded_decade.rs is {BASELINE[:16]}")
for name, want in (
    ("ratio_ruler", "198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1"),
    ("folded_decade", BASELINE),
    ("slide_rule", "68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd"),
):
    check(sha((STAGE07 / CRATE / f"src/chapters/{name}.rs").read_bytes()) == want, f"root's stage07 candidate {name}.rs is {want[:16]}")
print("  canonical folded_decade.rs at this run:", sha((REPO / FILE).read_bytes()))
print("  candidate folded_decade.rs:", sha(cand))
print(f"  lines: baseline {base.count(b'\n')}, candidate {cand.count(b'\n')}")
script = (STAGE07 / CRATE / "script.tsv").read_bytes()
canonical_script = (REPO / CRATE / "script.tsv").read_bytes()


def cues_of(data):
    return dict(row.split("\t", 1) for row in data.decode().splitlines() if "\t" in row and not row.startswith("#"))


print(f"  script.tsv: stage07 {sha(script)}, canonical {sha(canonical_script)}")
check(sha(canonical_script) == "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84", "canonical script.tsv is the one the takes' records cite")
check(sha(script) == "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e", "stage07's script.tsv is root's rounding wording")
new_cues, old_cues = cues_of(script), cues_of(canonical_script)
differing = [k for k in new_cues if new_cues[k] != old_cues.get(k)]
check(differing == ["01_5"] and set(new_cues) == set(old_cues), f"the two scripts differ only in {differing}; 00_1, 00_2 and 03_3 are the same text")
for cue in ("00_1", "00_2", "03_3"):
    print(f"    {cue} text sha256 {sha(new_cues[cue].encode())}")

print("\n-- the edit is small and moves no scene line")
a, b = base.decode().split("\n"), cand.decode().split("\n")
scene_a, scene_b = base.decode().split("#[cfg(test)]")[0].split("\n"), cand.decode().split("#[cfg(test)]")[0].split("\n")
const_at_a = next(i for i, x in enumerate(scene_a) if "How far through 03_3's first sentence" in x)
const_at_b = next(i for i, x in enumerate(scene_b) if "How far through 03_3's first sentence" in x)
check(const_at_a == const_at_b == 456, f"the constants still start on line {const_at_a + 1}")
different = [i + 1 for i in range(const_at_a) if scene_a[i] != scene_b[i]]
check(len(different) == 1, f"above them, exactly one line differs: {different}")
print("    was:", scene_a[different[0] - 1].strip())
print("    now:", scene_b[different[0] - 1].strip())
check("lay_beats(" in scene_b[different[0] - 1] and "forgets_mark(&kit)" in scene_b[different[0] - 1], "that line is the call to lay_beats")
check("fn forgets_mark(kit: &Kit)" in cand.decode() and "fn forgets_mark" not in base.decode(), "one helper is new")

print("\n-- what the binder's rules see, from its regexes")
code = "\n".join(x for x in scene_b if not x.strip().startswith("//"))
base_code = "\n".join(x for x in scene_a if not x.strip().startswith("//"))
TIMING = re.compile(r"\b(predict_pause|predict|kit\.hold|kit\.music_breath|kit\.listen|kit\.sonify|kit\.mark|kit\.finish|stage\.wait|wait_frames|wait_until)\(")
check(len(TIMING.findall(code)) == len(TIMING.findall(base_code)), f"timing sites unchanged, {len(TIMING.findall(code))}; no SITES row")
sentence = [r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', r'sentence_time\(&kit, "(\d\d_\d+)", (\d+)\)', r'\.sentence_at\("(\d\d_\d+)", (\d+)\)']
check(all(re.findall(p, code) == re.findall(p, base_code) for p in sentence), "sentence hooks unchanged, on the same lines")
hooks = re.compile(r"until_phrase\(|phrase_time\(")
check(not hooks.search(code), "no until_phrase or phrase_time line for HOOKS to classify")
uses = len(re.findall(r"phrase_marks\(", code))
check(uses == 1 and len(re.findall(r"phrase_marks\(", base_code)) == 0, "phrase_marks uses go from 0 to 1, so PHRASE_MARK_USES needs a row")
consts = dict(re.findall(r'const ([A-Z_]+): &str =\s*"([^"\\]*)";', code))
check(consts.get("FORGETS_PHRASE") == "forgets", f"the binder's constant regex reads {consts}")

text = dict(row.split("\t", 1) for row in script.decode().splitlines() if "\t" in row and not row.startswith("#"))["03_3"]
data = text.encode()
byte = data.find(b"forgets")
check(data.count(b"forgets") == 1, "the phrase occurs once in 03_3, as the binder demands")
check(byte == 35 and data[byte - 1] == 0x20, "it begins a word, at byte 35")
check(byte < text.index(". "), "it is in sentence 0")
words_before = len(re.findall(r"[A-Za-z]+(?:'[A-Za-z]+)?", data[:byte].decode()))
print(f"  plain-word count before it, as a guide to the binder's word_index: {words_before}")

binder = (R / "combined-repair-stage06/bind_final80-stage06.py").read_text()
check('PHRASE_MARK_USES = {"slide_rule.rs": 1, "undoing_growth.rs": 1, "doing_nothing.rs": 1, "unreachable.rs": 1, "what_it_forgets.rs": 1}' in binder, "stage06's table has no folded_decade.rs row today, so the binder stops until it gets one")
check('stop(f"{name}: {uses} phrase_marks uses; {PHRASE_MARK_USES.get(name, 0)} expected")' in binder, "that stop is in the binder")
check('    add_anchor(anchors, "04_2", climb, "CLIMB_PHRASE", "undoing_growth.rs",' in binder, "04_2's direct reader is the precedent for adding an anchor by hand")
preflight = (OPUS / "logarithm-stage06-preflight01/preflight.py").read_text()
check('direct == {("undoing_growth.rs", "04_2")}' in preflight, "the preflight's direct reader set needs the new pair")

print("\n-- holds")
plan = json.loads((R / "combined-repair-stage06/bound-plan/hold-audit.json").read_text())
holds = plan["prediction_holds"] + plan["reading_holds"]
check(holds and not any("folded_decade" in json.dumps(h) for h in holds), f"none of the {len(holds)} holds in the stage06 audit binds folded_decade.rs")
m = (R / "combined-repair-stage06/bound-plan" / CRATE / "docs/narration-direction-map-oliver13.json").read_text()
check("folded_decade" not in m, "map13 does not name folded_decade.rs")

print("\n-- source-markers.json agrees with the bytes")
markers = json.loads((HERE / "source-markers.json").read_text())
row = markers["sources"][0]
check(row["baseline_sha256"] == BASELINE and row["candidate_sha256"] == sha(cand), "hashes")
check(row["lines"] == [base.count(b"\n"), cand.count(b"\n")], f"line counts {row['lines']}")
anchor = markers["phrase_anchor_delta"][0]
sites = {s["call"]: s["line"] for s in anchor["hook_sites"]}
check('phrase_marks("03_3")' in b[sites["forgets_mark"] - 1] and "forgets_mark(&kit)" in b[sites["lay_beats"] - 1], f"anchor sites {sites}")
say = next(i + 1 for i, x in enumerate(b) if 'kit.say(stage, "03_3")' in x)
check(all(line > say for line in sites.values()), f"both sites follow say 03_3 on line {say}, as the preflight's destination check needs")
check(anchor["byte"] == byte and anchor["byte_end"] == byte + len("forgets") and anchor["cue_text_sha256"] == sha(text.encode()), "anchor byte and text hash")
check(anchor["measured_seconds"] is None and markers["template_mark_delta"]["03_3"][0]["seconds"] is None, "nothing measured is claimed")
check(sum(1 for x in b if 'phrase_marks("03_3")' in x) == 1 and sum(1 for x in b if "spoken, forgets_mark(&kit));" in x) == 1, "each site needle is on one line, as the binder's one_line demands")

print("\n-- format")
done = subprocess.run(["rustfmt", "--edition", "2024", "--check", "--config", "max_width=120", str(HERE / FILE)], cwd=REPO, capture_output=True, text=True)
check(done.returncode == 0, "rustfmt --check, max_width 120")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
