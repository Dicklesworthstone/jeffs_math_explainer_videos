#!/usr/bin/env python3
"""Integrate the reviewed logarithm clarity wording (04_4, 08_5) into a stage09 candidate and rebind its plan.

    python3 -I -B <this file> --print-pins
    python3 -I -B <this file> --out NEW_DIR
    python3 -I -B <this file> --check
    python3 -I -B <this file> --verify DIR
    python3 -I -B <this file> --validate-timings FILE [--template]
    python3 -I -B <this file> --validate-receipt FILE [FILE ...]

Run from the repo root. Every input is read only and pinned in pins.sha256 next to this file, whose
own hash is PINS_SHA256 below: root's combined-repair-stage08/candidate (its 47 files, by listing),
root's bound plan combined-repair-stage07/bound-plan03, the sealed clarity bundle
logarithm-narration-clarity-repair01 (patch 2463334a), binder03's binder (hashed and quoted only),
the 80 Oliver request receipts in ui-takes01 with the four 04_4/08_5 takes and their download
records, root's full80-retained-inventory01.json and narration-request-final80-02/requests.json,
the kit's narration.rs and kit.rs, and tools/coherent_narration.py with tools/narration_cache.py.
STALE INPUTS ARE REFUSED: a pinned file that differs or is missing, or a file added to or gone
from the stage08 candidate, stops every mode with exit 3 before anything is computed or written.
Only pinned files are read. To rebind on purpose, run --print-pins, read the difference, and write
a new pins file and PINS_SHA256 in a new helper; this one stays as sealed.

--out NEW_DIR writes, under a folder that must not exist yet:
  candidate/explainer_corpus/logarithm_mysteries/  stage08's 47 files with the sealed patch applied
        (script.tsv, src/chapters/product_rule.rs), narration_units.json rebound, and a new
        docs/narration-direction-map-oliver16.json; oliver15 and the older maps stay as they were
  bound-plan/explainer_corpus/logarithm_mysteries/  copies of the candidate's units and map
  bound-plan/hold-audit.json, phrase-anchors.json, phrase-timings-unmeasured-template.json,
        direction-dispositions-proposed.json, cue-delta.json   bound-plan03's files, rebound
  bound-plan/take-binding.json   which retained Oliver takes carry each stage09 text
  bound-plan/clarity-delta.json  every change against stage08 and bound-plan03, by JSON path
  bound-plan/inputs.sha256       the pinned inputs, for shasum -a 256 -c from the repo root
The bytes depend on the pinned inputs only, not on NEW_DIR, the clock or the machine. Paths inside
them name root's install location, combined-repair-stage09/{candidate,bound-plan}, so the
dispositions' evidence path resolves once root installs there.

--check verifies the copy delivered next to this file (stage09/). --verify DIR verifies any copy,
such as root's install: DIR/candidate and DIR/bound-plan, whose file sets and bytes must equal a
fresh build (exit 0), else exit 1 naming each differing file and JSON path. Anything else in DIR,
such as root's own receipt, is listed and not compared.

--validate-timings FILE checks a phrase-timings.json for stage09 before the kit reads it. The kit
checks hashes and order but not that a mark sits on its anchor, so marks left at 08_5's old bytes
260 and 299 under a refreshed text hash would load, and the hooks would fall back to character
shares. This requires one entry per cue, the stage09 text hash, marks at exactly the anchors' bytes
with their phrases, and, for the 78 unchanged cues, a WAV hash that is a retained Oliver take of
that exact text; for 04_4 and 08_5 it refuses every retained take, including the four of the old
wording. --template accepts the unmeasured template (null WAVs and seconds) instead. It cannot
compare a time with a WAV's length; the kit does that.

--validate-receipt FILE ... checks request receipts for the targeted 04_4 and 08_5 replacements, in
ui-takes01's layout: the exact stage09 text and its hash, script 559afa36, the ElevenLabs Oliver
label and the 80 retained takes' settings (Eleven v4, stability 0.5, similarity 0.75, clone
strength 0, WAV 48 kHz), no context text, and no other voice named. The four original takes must
still be present with their hashes.

Exit status: 0 pass, 1 a check failed, 2 usage, 3 stale input.

Source text and hashes only. Nothing here is compiled, rendered, synthesized, measured or heard,
and no audio is decoded: WAV files are only hashed. A pass says nothing about how a take sounds or
when a word is spoken. Owner direction 2026-11-05T03:22:06Z: logarithm keeps ElevenLabs Oliver; root
re-records 04_4 and 08_5 only, after review, and keeps every original take.
"""
import hashlib
import importlib.util
import json
import math
import re
import sys
from pathlib import Path

TOPIC = "logarithm_mysteries"
REL = f"explainer_corpus/{TOPIC}"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
STAGE08_CAND = f"{EL}/combined-repair-stage08/candidate"
PLAN3 = f"{EL}/combined-repair-stage07/bound-plan03"
INSTALL = f"{EL}/combined-repair-stage09"
TAKES = f"{EL}/ui-takes01"
INVENTORY = f"{EL}/full80-retained-inventory01.json"
REQUESTS = f"{EL}/narration-request-final80-02/requests.json"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
CLARITY = f"{P}/logarithm-narration-clarity-repair01"
BINDER03 = f"{P}/logarithm-stage07-binder03/bind_final80-stage07-r3.py"
HERE = f"{P}/logarithm-clarity-integration01"
PINS_FILE = f"{HERE}/pins.sha256"
PINS_SHA256 = "534016fd42c6a43430a5fd2795c089c03fa86e77ba5da4e35080eacc349eedf6"
DELIVERED = f"{HERE}/stage09"
KIT = ("crates/explainer_kit/src/kit.rs", "crates/explainer_kit/src/narration.rs")
TOOLS = ("tools/coherent_narration.py", "tools/narration_cache.py")
MAP15 = "narration-direction-map-oliver15.json"
MAP16 = "narration-direction-map-oliver16.json"
PLAN3_FILES = (
    "cue-delta.json",
    "direction-dispositions-proposed.json",
    "hold-audit.json",
    "increment-delta.json",
    "inputs.sha256",
    "phrase-anchors.json",
    "phrase-timings-unmeasured-template.json",
    "stage07-delta.json",
    f"{REL}/docs/{MAP15}",
    f"{REL}/narration_units.json",
)
REBOUND = (
    "hold-audit.json",
    "phrase-anchors.json",
    "phrase-timings-unmeasured-template.json",
    "direction-dispositions-proposed.json",
    "cue-delta.json",
)
NOT_REBOUND = {
    "stage07-delta.json": "History: stage07 against stage06. Not rebound; clarity-delta.json records this increment.",
    "increment-delta.json": "History: binder03 against binder02. Not rebound; clarity-delta.json records this increment.",
}
CLARITY_FILES = (
    "HANDOFF.md",
    "SHA256SUMS",
    "baseline-candidate.sha256",
    "binding-inventory.json",
    "checks/source-checks.out",
    "checks/source_checks.py",
    "expected-candidate.sha256",
    "logarithm-narration-clarity.patch",
)
OLD_TAKES = {
    "04_4-download01.wav": "de3abb952c675c2112c949813b2d70f962b95a3adf90d8f3289c6a4e29dce9ab",
    "04_4-download02.wav": "573d2360f72d4c8d451cd684ff8b3febf68168b24add6b070886433080185164",
    "08_5-download01.wav": "9b5d2b73ec6400569a32e03fe2c0d2b91c19308565ea6ee433b9ca99ca0d1ae5",
    "08_5-download02.wav": "5ec634e9c591e1768d69ea5195e6b07b91a3da1c6c7cecefa07799b3a269c493",
}
DOWNLOAD_RECORDS = ("04_4-downloads01.json", "08_5-downloads01.json")

# The reviewed values. Everything below is derived from the pinned inputs; a derivation that
# disagrees with one of these stops the run (exit 1) instead of writing something else.
PATCH_SHA = "2463334a21069f8d725ef722b362bc8b29aebe22a1b4f98bee0e77e4d515f736"
OLD_SCRIPT = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
NEW_SCRIPT = "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a"
OLD_PRODUCT = "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db"
NEW_PRODUCT = "bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635"
OLD_UNITS = "0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c"
OLD_MAP = "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4"
OLD_HOLD_AUDIT = "4040d484c48e930348726fca61a6ee531cdd58f08211e6c6a237c2ad138f1c9a"
CHANGED = {
    "04_4": {
        "old": "6a85a6bdd44b5c9942fa03e43b58ddf00f19acd0e69622ed0f45be218b8591a6",
        "new": "7e4e96121ff25f997a75bc882f70884bd61b3b039203e8df9c9d46fe545d2e83",
        "words": (71, 71),
    },
    "08_5": {
        "old": "f91292cc74fa243f405db101c25532df45d16465a66208ac33819baba40f7216",
        "new": "8e26bcb04dba0673057649b76a1fce2ce0375bcc1ff1a8fffb7d8379c13f4d82",
        "words": (81, 83),
    },
}
MOVED_ANCHORS = {("08_5", "HALF_TURN_PHRASE"): (260, 267), ("08_5", "FACTOR_PHRASE"): (299, 306)}
WORDS_TOTAL = (4592, 4594)
PRODUCT_LINES_CHANGED = [432, 722, 723]
OLIVER_LABELS = ("Oliver", "Oliver - Clean, British and Steady")
OLIVER_SETTINGS = {"model": "Eleven v4", "stability": 0.5, "similarity": 0.75, "clone_strength": 0}
OLIVER_FORMATS = ("WAV 48 kHz", "WAV 48 kHz (Lossless)", "WAV48k")  # the labels ui-takes01's 80 receipts use
CONTEXT_KEY = re.compile(r"context|previous|next_text|prior|following", re.I)
OTHER_VOICE = re.compile(r"frederick|surrey|michael|frankentts", re.I)
BINDER03_REFUSALS = (
    (967, 'if script_sha != EXPECTED_SCRIPT:', "refuses any script but b89eab60"),
    (1348, "the script differs from stage06's in more than the 01_5 rounding wording", "refuses a script that differs from stage06's in more than 01_5"),
    (1454, "a phrase anchor on the reworded cue needs its bytes re-read", "refuses a reworded cue that carries a phrase anchor, as 08_5 now is"),
)

UNITS_SCOPE = (
    "Candidate full-cue source plan for ElevenLabs Oliver, combined-repair-stage09/candidate, script 559afa36: "
    "80 units, one per cue, at most 100 words. 04_4 and 08_5 carry the reviewed clarity wording (patch "
    "2463334a), so no take of their stage08 text can bind; the other 78 texts are stage08's bytes. 01_5's "
    "wording changed in stage07, so no take of its stage06 text can bind. It says nothing about whether a "
    "recording of any other cue exists; unchanged wording is not evidence of a usable take. No synthesis, take "
    "selection, listening, timing or independent acceptance. No Michael or FrankenTTS."
)
MAP_SCOPE = (
    "Candidate final 80-cue map, combined-repair-stage09/candidate: 13 prediction holds and 2 reading holds, "
    "each bound to staged source. Successor to oliver15, which stage07 and stage08 carry with script b89eab60: "
    "the same overrides, holds, lines and intervals, rebound to script 559afa36 and to product_rule.rs "
    "bc9fbe1d after the reviewed clarity wording of 04_4 and 08_5 (patch 2463334a). Neither reworded cue "
    "carries an override or a hold. Hashes are the proposed staged bytes, not installed canonical files. "
    "Authored coverage only: no silence is added to any take, and no take, measured timing, listening or "
    "independent acceptance is claimed. ElevenLabs Oliver; no Michael or FrankenTTS."
)
HOLD_CLAIM = (
    "Text audit of Rust source at combined-repair-stage07/candidate (binder03), rebound by "
    "logarithm-clarity-integration01 to combined-repair-stage09/candidate: script 559afa36 and product_rule.rs "
    "bc9fbe1d, whose changed lines are a comment inside 08_5's span and a test. Every hold interval, marker "
    "and say line was re-read and is unchanged. Uncompiled here, unrendered, unmeasured, unheard. "
    "Author/source binding, not independent AV or listening acceptance."
)
OPEN_FOR_ROOT = (
    "targeted Oliver takes of the reviewed 04_4 and 08_5 wording, then measured marks for them (08_5 at bytes "
    "267 and 306); the four takes of the stage08 wording are preserved and cannot bind"
)
DELTA_CLAIM = (
    "Every difference between this stage09 plan and root's stage08 candidate and bound-plan03, by JSON path, "
    "computed by integrate_clarity.py. Source text and hashes only: nothing compiled, rendered, synthesized, "
    "measured or heard."
)
TAKE_CLAIM = (
    "Which retained Oliver takes carry each stage09 text, from root's full80-retained-inventory01.json. A "
    "match means the take's recorded text hash equals the cue's stage09 text hash. No take is selected, "
    "heard, timed or accepted here. This helper re-hashes only the four 04_4 and 08_5 takes; the other "
    "hashes are as root's inventory records them."
)


class Stale(Exception):
    pass


def fail(message):
    print(f"integrate_clarity: {message}", file=sys.stderr)
    sys.exit(1)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_t(text):
    return sha(text.encode("utf-8"))


def dump(obj):
    return (json.dumps(obj, indent=2, ensure_ascii=False) + "\n").encode("utf-8")


# ---------------------------------------------------------------------------------------- pins


def parse_sums(text):
    rows = {}
    for n, line in enumerate(text.splitlines(), 1):
        m = re.fullmatch(r"([0-9a-f]{64})  (\S.*)", line)
        if not m or m.group(2) in rows:
            raise ValueError(f"line {n}: not a 'sha256  path' row, or a repeated path")
        rows[m.group(2)] = m.group(1)
    return rows


def candidate_listing():
    root = Path(STAGE08_CAND)
    if not root.is_dir():
        return []
    return sorted(f"{STAGE08_CAND}/{p.relative_to(root).as_posix()}" for p in root.rglob("*") if p.is_file())


def required_inputs(cues):
    rows = [f"{PLAN3}/{name}" for name in PLAN3_FILES]
    rows += [f"{CLARITY}/{name}" for name in CLARITY_FILES]
    rows += [BINDER03, INVENTORY, REQUESTS, *KIT, *TOOLS]
    rows += [f"{TAKES}/{name}" for name in (*OLD_TAKES, *DOWNLOAD_RECORDS)]
    rows += [f"{TAKES}/{cue}-request01.json" for cue in cues]
    return rows


class Inputs:
    """The pinned inputs. Construction exits 3 on any stale input; read() refuses an unpinned path."""

    def __init__(self):
        problems = []
        pins = Path(PINS_FILE)
        if not pins.is_file():
            self.stale([f"MISSING  {PINS_FILE}"])
        if sha(pins.read_bytes()) != PINS_SHA256:
            self.stale([f"CHANGED  {PINS_FILE}\n         pinned {PINS_SHA256}\n         found  {sha(pins.read_bytes())}"])
        self.pins = parse_sums(pins.read_text(encoding="utf-8"))
        for path, want in self.pins.items():
            p = Path(path)
            if not p.is_file():
                problems.append(f"MISSING  {path}")
            elif sha(p.read_bytes()) != want:
                problems.append(f"CHANGED  {path}\n         pinned {want}\n         found  {sha(p.read_bytes())}")
        pinned_candidate = sorted(p for p in self.pins if p.startswith(STAGE08_CAND + "/"))
        listed = candidate_listing()
        if listed != pinned_candidate:
            added, gone = sorted(set(listed) - set(pinned_candidate)), sorted(set(pinned_candidate) - set(listed))
            problems.append(f"LISTING  {STAGE08_CAND}: added {added}, gone {gone}")
        if problems:
            self.stale(problems)
        cues = list(script_rows(self.read(f"{STAGE08_CAND}/{REL}/script.tsv")))
        others = set(self.pins) - set(pinned_candidate)
        wanted = set(required_inputs(cues))
        if others != wanted:
            self.stale([f"PINS     the pin table does not name exactly this helper's inputs: {sorted(others ^ wanted)}"])

    @staticmethod
    def stale(problems):
        print("integrate_clarity: STALE INPUT, refused; nothing computed or written.", file=sys.stderr)
        for row in problems:
            print(row, file=sys.stderr)
        sys.exit(3)

    def read(self, path):
        if path not in self.pins:
            fail(f"internal: {path} is not pinned, so it is not read")
        return Path(path).read_bytes()

    def text(self, path):
        return self.read(path).decode("utf-8")

    def json(self, path):
        return json.loads(self.text(path))


def print_pins():
    pins = parse_sums(Path(PINS_FILE).read_text(encoding="utf-8")) if Path(PINS_FILE).is_file() else {}
    cues = list(script_rows(Path(f"{STAGE08_CAND}/{REL}/script.tsv").read_bytes()))
    rows = sorted(set(candidate_listing()) | set(required_inputs(cues)))
    for path in rows:
        p = Path(path)
        got = sha(p.read_bytes()) if p.is_file() else "MISSING"
        mark = "" if pins.get(path) == got else "   <- differs from pins.sha256"
        print(f"{got}  {path}{mark}")
    print(f"# {len(rows)} inputs; pins.sha256 {sha(Path(PINS_FILE).read_bytes()) if Path(PINS_FILE).is_file() else 'MISSING'}, "
          f"PINS_SHA256 {PINS_SHA256}")
    return 0


# ---------------------------------------------------------------------------------------- text rules


def script_rows(data):
    """tools/coherent_narration.py read_script(), on bytes."""
    rows = {}
    for number, row in enumerate(data.decode("utf-8").splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        if len(cells) != 2 or not re.fullmatch(r"\d\d_\d+", cells[0]) or not cells[1].strip() or cells[0] in rows:
            fail(f"script row {number} is not a cue")
        rows[cells[0]] = cells[1]
    return rows


def sentence_opens(text):
    """binder03's sentence_opens(): bytes after '.', '?' or '!' and a space, as phrase_seconds counts them."""
    raw = text.encode("utf-8")
    return [0] + [i + 2 for i in range(len(raw) - 1) if raw[i] in b".?!" and raw[i + 1] == 0x20]


def anchor_fields(coherent, cue, text, phrase):
    """binder03's anchor rule: text.find(phrase) bytes, once, at a word start; its sentence; direction words before it."""
    raw, wanted = text.encode("utf-8"), phrase.encode("utf-8")
    if raw.count(wanted) != 1:
        fail(f"{cue}: {phrase!r} occurs {raw.count(wanted)} times")
    byte = raw.find(wanted)
    if byte and raw[byte - 1] != 0x20:
        fail(f"{cue}: {phrase!r} does not begin a word")
    word_index = len(coherent.direction_words(raw[:byte].decode("utf-8")))
    words, said = coherent.direction_words(text), coherent.direction_words(phrase)
    if words[word_index : word_index + len(said)] != said:
        fail(f"{cue}: {phrase!r} is not at word {word_index}")
    return {
        "byte": byte,
        "byte_end": byte + len(wanted),
        "sentence_index": max(i for i, at in enumerate(sentence_opens(text)) if at <= byte),
        "word_index": word_index,
    }


def interval(lines, say, marker):
    """binder03's interval hash: the source lines from the cue's say through the hold's marker."""
    return sha_t("\n".join(lines[say - 1 : marker]))


def json_changes(old, new, path="$"):
    """binder03's json_changes(): every difference between two JSON values, by path."""
    if isinstance(old, dict) and isinstance(new, dict):
        rows = []
        for key in sorted(set(old) | set(new)):
            if key not in old:
                rows.append({"path": f"{path}.{key}", "added": new[key]})
            elif key not in new:
                rows.append({"path": f"{path}.{key}", "removed": old[key]})
            else:
                rows += json_changes(old[key], new[key], f"{path}.{key}")
        return rows
    if isinstance(old, list) and isinstance(new, list):
        if len(old) == len(new):
            return [row for i, (a, b) in enumerate(zip(old, new)) for row in json_changes(a, b, f"{path}[{i}]")]
        return [{"path": path, "added": [x for x in new if x not in old], "removed": [x for x in old if x not in new]}]
    return [] if old == new else [{"path": path, "old": old, "new": new}]


def apply_patch(files, patch):
    """A strict unified-diff applier: no fuzz, no offset, no new or removed file, every context line exact."""
    if not patch.endswith("\n") or "\\ No newline" in patch or "/dev/null" in patch:
        fail("patch: not a plain two-file text patch")
    lines, out, touched, i = patch[:-1].split("\n"), dict(files), [], 0
    while i < len(lines):
        if i + 1 >= len(lines) or not lines[i].startswith("--- a/") or not lines[i + 1].startswith("+++ b/"):
            fail(f"patch line {i + 1}: expected a file header")
        name = lines[i][len("--- a/") :]
        if lines[i + 1][len("+++ b/") :] != name or name not in out or name in touched:
            fail(f"patch: {name} is renamed, missing from the candidate, or patched twice")
        src, result, pos, i = out[name].decode("utf-8").split("\n"), [], 0, i + 2
        if not i < len(lines) or not lines[i].startswith("@@ "):
            fail(f"patch: {name} has no hunk")
        while i < len(lines) and lines[i].startswith("@@ "):
            m = re.fullmatch(r"@@ -(\d+),(\d+) \+(\d+),(\d+) @@", lines[i])
            if not m:
                fail(f"patch line {i + 1}: not a hunk header")
            old_at, old_n, new_at, new_n = map(int, m.groups())
            if old_at - 1 < pos:
                fail(f"patch: {name} hunks overlap")
            result += src[pos : old_at - 1]
            pos = old_at - 1
            if new_at - 1 != len(result):
                fail(f"patch: {name} hunk at {old_at} would land at {len(result) + 1}, not {new_at}")
            seen_old = seen_new = 0
            i += 1
            while i < len(lines) and lines[i][:1] in (" ", "-", "+") and not lines[i].startswith(("--- a/", "+++ b/")):
                tag, body = lines[i][0], lines[i][1:]
                if tag in " -":
                    if pos >= len(src) or src[pos] != body:
                        fail(f"patch: {name} line {pos + 1} is not the patch's context")
                    pos, seen_old = pos + 1, seen_old + 1
                if tag in " +":
                    result.append(body)
                    seen_new += 1
                i += 1
            if (seen_old, seen_new) != (old_n, new_n):
                fail(f"patch: {name} hunk at {old_at} has {seen_old}/{seen_new} lines, not {old_n}/{new_n}")
        out[name] = "\n".join(result + src[pos:]).encode("utf-8")
        touched.append(name)
    return out, touched


def load_coherent(inputs):
    for path in TOOLS:
        inputs.read(path)
    spec = importlib.util.spec_from_file_location("coherent_narration", TOOLS[0])
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def repo_relative(path):
    """Root's inventory records absolute paths; the plan keeps the repository-relative part."""
    at = path.find("narration_lab/")
    if at < 0:
        fail(f"{path}: not under narration_lab/")
    return path[at:]


# ---------------------------------------------------------------------------------------- build


def build(inputs):
    """Everything --out writes, as {relative path: bytes}, plus the expectations the validators use."""
    coherent = load_coherent(inputs)
    cand_rel = sorted(p[len(STAGE08_CAND) + 1 :] for p in inputs.pins if p.startswith(STAGE08_CAND + "/"))
    stage08 = {rel: inputs.read(f"{STAGE08_CAND}/{rel}") for rel in cand_rel}
    script_rel, product_rel = f"{REL}/script.tsv", f"{REL}/src/chapters/product_rule.rs"
    units_rel, map15_rel, map16_rel = f"{REL}/narration_units.json", f"{REL}/docs/{MAP15}", f"{REL}/docs/{MAP16}"
    if map16_rel in stage08:
        fail(f"stage08 already has {MAP16}")
    for name in (units_rel, map15_rel):
        if stage08[name] != inputs.read(f"{PLAN3}/{name}"):
            fail(f"stage08's {name} is not bound-plan03's")
    if sha(stage08[units_rel]) != OLD_UNITS or sha(stage08[map15_rel]) != OLD_MAP:
        fail("stage08's units or map are not the reviewed bytes")
    plan3 = {name: inputs.json(f"{PLAN3}/{name}") for name in REBOUND}
    if sha(inputs.read(f"{PLAN3}/hold-audit.json")) != OLD_HOLD_AUDIT:
        fail("bound-plan03's hold-audit.json is not the reviewed bytes")

    # ------------------------------------------------------------ the sealed patch, checked against its own seal
    sums = parse_sums(inputs.text(f"{CLARITY}/SHA256SUMS"))
    for name in CLARITY_FILES:
        if name != "SHA256SUMS" and sums.get(name) != sha(inputs.read(f"{CLARITY}/{name}")):
            fail(f"the clarity bundle's {name} is not what its SHA256SUMS seals")
    if set(sums) != set(CLARITY_FILES) - {"SHA256SUMS"}:
        fail("the clarity bundle's SHA256SUMS names other files")
    patch = inputs.read(f"{CLARITY}/logarithm-narration-clarity.patch")
    if sha(patch) != PATCH_SHA:
        fail("the clarity patch is not 2463334a")
    patched, touched = apply_patch(stage08, patch.decode("utf-8"))
    if touched != [script_rel, product_rel]:
        fail(f"the patch touches {touched}")
    for name, rows in (("baseline-candidate.sha256", stage08), ("expected-candidate.sha256", patched)):
        want = parse_sums(inputs.text(f"{CLARITY}/{name}"))
        got = {path: sha(rows[path]) for path in want}
        if got != want:
            fail(f"{name}: {sorted(p for p in want if got[p] != want[p])} differ")
    if sha(patched[script_rel]) != NEW_SCRIPT or sha(patched[product_rel]) != NEW_PRODUCT:
        fail("the patched script or product_rule.rs is not the reviewed bytes")
    if sha(stage08[script_rel]) != OLD_SCRIPT or sha(stage08[product_rel]) != OLD_PRODUCT:
        fail("stage08's script or product_rule.rs is not the reviewed bytes")
    old, new = script_rows(stage08[script_rel]), script_rows(patched[script_rel])
    if old != coherent.read_script(f"{STAGE08_CAND}/{script_rel}"):
        fail("this helper's script reader disagrees with tools/coherent_narration.py read_script()")
    if list(old) != list(new) or [c for c in old if old[c] != new[c]] != list(CHANGED):
        fail("the patch changes other cues than 04_4 and 08_5, or the cue list")
    for cue, want in CHANGED.items():
        if (sha_t(old[cue]), sha_t(new[cue])) != (want["old"], want["new"]):
            fail(f"{cue}: the texts are not the reviewed ones")
        if (len(old[cue].split()), len(new[cue].split())) != want["words"]:
            fail(f"{cue}: the word counts are not the reviewed ones")
        if len(sentence_opens(old[cue])) != len(sentence_opens(new[cue])):
            fail(f"{cue}: the sentence count changed")
    old_lines = stage08[product_rel].decode("utf-8").split("\n")
    new_lines = patched[product_rel].decode("utf-8").split("\n")
    if len(old_lines) != len(new_lines) or [n for n, (a, b) in enumerate(zip(old_lines, new_lines), 1) if a != b] != PRODUCT_LINES_CHANGED:
        fail("product_rule.rs changes other lines than 432, 722 and 723, or moves a line")
    unchanged = [c for c in new if c not in CHANGED]

    def source_lines(files, rel):
        return files[rel].decode("utf-8").split("\n")

    def crate_rel(path):
        if not path.startswith(REL + "/"):
            fail(f"{path}: not a crate path")
        return path

    # ------------------------------------------------------------ the map, oliver16
    hold3 = plan3["hold-audit.json"]
    says = {(r["cue"], r["file"]): r["say_line"] for r in hold3["prediction_holds"] + hold3["reading_holds"]}
    m16 = json.loads(stage08[map15_rel])
    if m16["scripts"] != {TOPIC: OLD_SCRIPT}:
        fail("oliver15 binds another script")
    m16["scripts"][TOPIC] = NEW_SCRIPT
    for row in m16["visual_holds"]:
        rel = crate_rel(row["path"])
        if not (row["sha256"] == row["evidence"]["sha256"] == sha(stage08[rel]) and row["evidence"]["path"] == row["path"]):
            fail(f"oliver15 {row['cue']}: the file hash is not stage08's")
        say = says[(row["cue"], Path(rel).name)]
        for files in (stage08, patched):
            lines = source_lines(files, rel)
            if interval(lines, say, row["line"]) != row["interval_sha256"] or row["marker"] not in lines[row["line"] - 1]:
                fail(f"oliver15 {row['cue']}: the interval or marker does not hold in {'stage08' if files is stage08 else 'stage09'}")
        row["sha256"] = row["evidence"]["sha256"] = sha(patched[rel])
    for helper in m16["shared_helpers"]:
        current = sha(patched[helper["path"]]) if helper["path"].startswith(REL + "/") else sha(inputs.read(helper["path"]))
        if current != helper["sha256"]:
            fail(f"oliver15 shared helper {helper['path']} is not current")
    for item in m16["overrides"]:
        if item["cue"] in CHANGED:
            fail(f"an override sits on {item['cue']}, which the patch rewords")
        if item["after"] not in new[item["cue"]] or old[item["cue"]] != new[item["cue"]]:
            fail(f"override on {item['cue']} no longer reads its words")
    m16["scope"] = MAP_SCOPE
    map16 = dump(m16)

    # ------------------------------------------------------------ the units
    units = json.loads(stage08[units_rel])
    for unit in units["units"]:
        if unit["text"] != " ".join(old[c] for c in unit["line_ids"]):
            fail(f"{unit['id']}: stage08's unit text is not its script rows")
        unit["text"] = " ".join(new[c] for c in unit["line_ids"])
        if not 1 <= len(unit["text"].split()) <= 100:
            fail(f"{unit['id']}: outside the coherent-unit word bound")
    if [c for u in units["units"] for c in u["line_ids"]] != list(new):
        fail("the units do not cover the script in order")
    contract = units["direction_contract"]
    if units["script_sha256"] != OLD_SCRIPT or contract["script_sha256"] != OLD_SCRIPT or contract["map"] != {"path": map15_rel, "sha256": OLD_MAP}:
        fail("stage08's units bind another script or map")
    units["script_sha256"] = contract["script_sha256"] = NEW_SCRIPT
    contract["map"] = {"path": map16_rel, "sha256": sha(map16)}
    units["scope"] = UNITS_SCOPE
    units_bytes = dump(units)
    candidate = dict(patched)
    candidate[units_rel] = units_bytes
    candidate[map16_rel] = map16

    # ------------------------------------------------------------ phrase anchors and the unmeasured template
    a3 = plan3["phrase-anchors.json"]
    anchors = json.loads(json.dumps(a3))
    if anchors["script_sha256"] != OLD_SCRIPT:
        fail("bound-plan03's anchors bind another script")
    anchors["script_sha256"] = NEW_SCRIPT
    expected_marks = {cue: [] for cue in new}
    for row in anchors["anchors"]:
        cue, rel = row["cue"], crate_rel(row["source_path"])
        was = anchor_fields(coherent, cue, old[cue], row["phrase"])
        if any(row[k] != v for k, v in was.items()) or row["cue_text_sha256"] != sha_t(old[cue]) or row["source_sha256"] != sha(stage08[rel]):
            fail(f"{cue} {row['const']}: binder03's rule does not reproduce bound-plan03's row on stage08")
        for site in row["hook_sites"]:
            site_rel = f"{REL}/src/chapters/{site['file']}"
            for files in (stage08, patched):
                line = source_lines(files, site_rel)[site["line"] - 1]
                needles = [n for n in (site["call"], row["const"], f'phrase_marks("{cue}")') if n]  # 10_3's const is null
                if not any(needle in line for needle in needles):
                    fail(f"{cue} {row['const']}: {site['file']}:{site['line']} names neither {site['call']}, the phrase nor its marks")
        now = anchor_fields(coherent, cue, new[cue], row["phrase"])
        key = (cue, row["const"])
        if key in MOVED_ANCHORS:
            if (was["byte"], now["byte"]) != MOVED_ANCHORS[key] or now["sentence_index"] != was["sentence_index"]:
                fail(f"{cue} {row['const']}: moves {was['byte']} -> {now['byte']}, not the reviewed bytes")
        elif now != was:
            fail(f"{cue} {row['const']}: an anchor moved that the review does not move")
        row.update(now)
        row["cue_text_sha256"] = sha_t(new[cue])
        row["source_sha256"] = sha(patched[rel])
        row["byte_or_cue_moved"] = row["byte_or_cue_moved"] or now["byte"] != was["byte"]
        expected_marks[cue].append((now["byte"], row["phrase"]))
    if {k for k in MOVED_ANCHORS} - {(r["cue"], r["const"]) for r in anchors["anchors"]}:
        fail("a reviewed anchor is missing from bound-plan03")
    for cue in expected_marks:
        expected_marks[cue].sort()
    counts = anchors["counts"]
    counts["byte_or_cue_moved"] = sum(1 for r in anchors["anchors"] if r["byte_or_cue_moved"])
    if counts["anchors"] != len(anchors["anchors"]) or counts["affected_by_cue_split"] != sum(1 for r in anchors["anchors"] if r["affected_by_cue_split"]):
        fail("bound-plan03's anchor counts do not match its rows")
    moves = ", ".join(f"{k[1]} from byte {b[0]} to {b[1]}" for k, b in sorted(MOVED_ANCHORS.items()))
    anchors["wording_since_stage06"] = (
        "01_5 was reworded in stage07 and carries no anchor. The reviewed clarity patch (2463334a) rewords 04_4, "
        "which carries no anchor, and 08_5, whose sentence 4 now opens \"Taking half of\" where it said \"Halving\": "
        f"7 bytes and 2 words longer, so both 08_5 anchors move within that sentence, {moves}. A mark at an "
        "old byte cannot serve; integrate_clarity.py --validate-timings refuses one."
    )
    before = "byte_or_cue_moved: the anchor also has a new cue id or byte, as it does in 11_20 and 12_21."
    if anchors["affected_means"].count(before) != 1:
        fail("bound-plan03's affected_means no longer says what this helper amends")
    anchors["affected_means"] = anchors["affected_means"].replace(
        before, before[:-1] + ", and in 08_5 after the clarity rewording (same cue, new bytes)."
    )

    t3 = plan3["phrase-timings-unmeasured-template.json"]
    template = json.loads(json.dumps(t3))
    if template["script_sha256"] != OLD_SCRIPT or list(template["cues"]) != list(old):
        fail("bound-plan03's template binds another script or cue list")
    template["script_sha256"] = NEW_SCRIPT
    for cue, entry in template["cues"].items():
        was_marks = [(m["byte"], m["phrase"]) for m in entry["marks"]]
        was_expected = sorted((r["byte"], r["phrase"]) for r in a3["anchors"] if r["cue"] == cue)
        if entry["text_sha256"] != sha_t(old[cue]) or entry["wav_sha256"] is not None or was_marks != was_expected or any(m["seconds"] is not None for m in entry["marks"]):
            fail(f"{cue}: bound-plan03's template entry is not its anchors on stage08")
        entry["text_sha256"] = sha_t(new[cue])
        entry["marks"] = [{"byte": b, "seconds": None, "phrase": p} for b, p in expected_marks[cue]]

    # ------------------------------------------------------------ the hold audit
    hold = json.loads(json.dumps(hold3))
    if hold["script_sha256"] != OLD_SCRIPT or hold["map"] != map15_rel:
        fail("bound-plan03's hold audit binds another script or map")
    hold["claim"] = HOLD_CLAIM
    hold["script_sha256"] = NEW_SCRIPT
    hold["map"] = map16_rel
    if OPEN_FOR_ROOT not in hold["open_for_root"]:
        hold["open_for_root"].append(OPEN_FOR_ROOT)
    for row in hold["cues"]:
        cue, rel = row["cue"], f"{REL}/src/chapters/{row['file']}"
        if row["words"] != len(old[cue].split()):
            fail(f"{cue}: bound-plan03's word count is not stage08's text")
        for files in (stage08, patched):
            if f'"{cue}"' not in source_lines(files, rel)[row["line"] - 1]:
                fail(f"{cue}: {row['file']}:{row['line']} no longer says it")
        row["words"] = len(new[cue].split())
    words = [len(new[c].split()) for c in new]
    if hold["counts"]["words_total"] != sum(len(old[c].split()) for c in old) or (hold["counts"]["words_total"], sum(words)) != WORDS_TOTAL:
        fail("the word total is not the reviewed one")
    hold["counts"]["words_total"], hold["counts"]["words_max"] = sum(words), max(words)
    for row in hold["prediction_holds"] + hold["reading_holds"]:
        cue, rel = row["cue"], f"{REL}/src/chapters/{row['file']}"
        if row["source_sha256"] != sha(stage08[rel]):
            fail(f"{cue}: bound-plan03's hold names other bytes than stage08's {row['file']}")
        if cue in CHANGED or row["cue_text"] != new[cue] or row["cue_text_sha256"] != sha_t(new[cue]):
            fail(f"{cue}: a held cue's text changed")
        for files in (stage08, patched):
            lines = source_lines(files, rel)
            if interval(lines, row["say_line"], row["marker_line"]) != row["interval_sha256"] or row["marker"] not in lines[row["marker_line"] - 1]:
                fail(f"{cue}: the hold's interval or marker does not hold")
        row["source_sha256"] = sha(patched[rel])
    for row in hold["registered_chapters"]:
        rel = f"{REL}/src/chapters/{row['file']}"
        if row["sha256"] != sha(stage08[rel]):
            fail(f"{row['file']}: bound-plan03's registered hash is not stage08's")
        row["sha256"] = sha(patched[rel])
    for row in hold["other_waits"] + hold["chapter_finishes"]:
        rel = f"{REL}/src/chapters/{row['file']}"
        if row["marker"] not in source_lines(patched, rel)[row["line"] - 1]:
            fail(f"{row['cue']}: {row['file']}:{row['line']} no longer holds {row['marker']!r}")
    hold_bytes = dump(hold)

    # ------------------------------------------------------------ dispositions
    disp = json.loads(json.dumps(plan3["direction-dispositions-proposed.json"]))
    if disp["map_sha256"] != OLD_MAP:
        fail("bound-plan03's dispositions bind another map")
    disp["map_sha256"] = sha(map16)
    for row in disp["direction_dispositions"]:
        if row["evidence"] != {"path": f"{PLAN3}/hold-audit.json", "sha256": OLD_HOLD_AUDIT}:
            fail(f"disposition {row['override_index']}: evidence is not bound-plan03's hold audit")
        row["evidence"] = {"path": f"{INSTALL}/bound-plan/hold-audit.json", "sha256": sha(hold_bytes)}
    if sorted(r["override_index"] for r in disp["direction_dispositions"]) != list(range(len(m16["overrides"]))):
        fail("the dispositions do not cover the map's overrides")

    # ------------------------------------------------------------ cue delta
    delta3 = plan3["cue-delta.json"]
    cue_delta = json.loads(json.dumps(delta3))
    if [r["cue"] for r in cue_delta["cues"]] != list(old):
        fail("bound-plan03's cue delta is not one row per cue, in order")
    if cue_delta["current_request_list"]["path"] != REQUESTS or cue_delta["current_request_list"]["sha256"] != sha(inputs.read(REQUESTS)):
        fail("bound-plan03's cue delta names another request list")
    for row in cue_delta["cues"]:
        cue = row["cue"]
        if row["text_sha256"] != sha_t(old[cue]) or row["words"] != len(old[cue].split()):
            fail(f"{cue}: bound-plan03's cue delta is not stage08's text")
        if cue not in CHANGED:
            continue
        if row["text_changed_from_baseline"] or row["reworded_since_stage06"] or row["current_request"]["sha256"] != sha_t(old[cue]):
            fail(f"{cue}: bound-plan03's row is not the unchanged stage08 text it reviews")
        row["words"] = len(new[cue].split())
        row["text_sha256"] = sha_t(new[cue])
        row["text_changed_from_baseline"] = True
        row["reworded_since_stage06"] = True
        row["request_id_with_exact_bytes"] = None
        row["superseded_request"] = row["current_request"]
        row["current_request"] = {
            "path": None,
            "sha256": None,
            "submitted": False,
            "submission_state": "no request file: root prepares the targeted Oliver take of the reviewed wording",
            "retained_pair_receipt": None,
        }
    note = "null for 01_5, whose new text is not in that plan."
    if cue_delta["request_id_note"].count(note) != 1:
        fail("bound-plan03's request_id_note no longer says what this helper amends")
    cue_delta["request_id_note"] = cue_delta["request_id_note"].replace(
        note,
        "null for 01_5, 04_4 and 08_5, whose current texts are not in that plan. No request file has 04_4's or 08_5's "
        "current bytes yet; superseded_request keeps each one's bound-plan03 row.",
    )
    rows = cue_delta["cues"]
    want_counts = {
        "cues": len(rows),
        "unchanged_from_baseline": sum(1 for r in rows if not r["text_changed_from_baseline"]),
        "split_cues": delta3["counts"]["split_cues"],
        "reworded_since_stage06": sum(1 for r in rows if r["reworded_since_stage06"]),
        "with_exact_request_bytes": sum(1 for r in rows if r["request_id_with_exact_bytes"]),
        "with_current_request_file": sum(1 for r in rows if r["current_request"]["path"]),
    }
    cue_delta["counts"] = want_counts

    # ------------------------------------------------------------ receipts and the take binding
    inventory = inputs.json(INVENTORY)
    if inventory["script_sha256"] != OLD_SCRIPT or inventory["cue_count"] != len(old) or len(inventory["takes"]) != inventory["retained_file_count"]:
        fail("root's retained inventory binds another script or miscounts its takes")
    for cue in old:
        receipt = inputs.json(f"{TAKES}/{cue}-request01.json")
        settings_ok = receipt.get("voice") in OLIVER_LABELS and receipt.get("format") in OLIVER_FORMATS and all(
            receipt.get(k) == v for k, v in OLIVER_SETTINGS.items()
        )
        if receipt.get("cue") != cue or receipt.get("text") != old[cue] or receipt.get("text_sha256") != sha_t(old[cue]) or not settings_ok:
            fail(f"{cue}: ui-takes01's request receipt is not an Oliver request for stage08's text")
    records = {cue: inputs.json(f"{TAKES}/{cue}-downloads01.json") for cue in CHANGED}
    for name, want in OLD_TAKES.items():
        cue = name[:4]
        if sha(inputs.read(f"{TAKES}/{name}")) != want or want not in [d["sha256"] for d in records[cue]["downloads"]]:
            fail(f"{name}: the original take is not the recorded bytes")
    binding_rows = []
    all_takes = set()
    for cue in new:
        takes = [t for t in inventory["takes"] if t["cue"] == cue]
        match = [t for t in takes if t["text_sha256"] == sha_t(new[cue])]
        stale = [t for t in takes if t["text_sha256"] != sha_t(new[cue])]
        all_takes |= {t["sha256"] for t in takes}
        if cue in CHANGED:
            if match or sorted(t["sha256"] for t in stale) != sorted(v for k, v in OLD_TAKES.items() if k.startswith(cue)):
                fail(f"{cue}: the retained takes are not the two of the stage08 wording")
            if any(t["text_sha256"] != sha_t(old[cue]) for t in stale):
                fail(f"{cue}: a retained take carries neither text")
        elif len(match) < 2 or stale:
            fail(f"{cue}: fewer than two retained takes of the unchanged text, or a take of another text")
        binding_rows.append({
            "cue": cue,
            "text_sha256": sha_t(new[cue]),
            "status": "needs_new_oliver_take" if cue in CHANGED else "text_matches_retained_takes",
            "takes": [{"path": repo_relative(t["wav"]), "sha256": t["sha256"], "frames": t["frames"]} for t in match],
            "old_text_takes": [{"path": repo_relative(t["wav"]), "sha256": t["sha256"], "text_sha256": t["text_sha256"], "preserved": True} for t in stale],
        })
    binding = {
        "schema_version": 1,
        "topic": TOPIC,
        "script_sha256": NEW_SCRIPT,
        "claim": TAKE_CLAIM,
        "voice": "ElevenLabs Oliver for logarithm, owner direction 2026-11-05T03:22:06Z. No mixed-voice final.",
        "source": {"path": INVENTORY, "sha256": sha(inputs.read(INVENTORY)), "script_sha256": inventory["script_sha256"]},
        "counts": {
            "cues": len(binding_rows),
            "text_matches_retained_takes": sum(1 for r in binding_rows if r["takes"]),
            "needs_new_oliver_take": sum(1 for r in binding_rows if not r["takes"]),
            "retained_takes": len(inventory["takes"]),
            "takes_matching_stage09_text": sum(len(r["takes"]) for r in binding_rows),
            "takes_of_old_text_preserved": sum(len(r["old_text_takes"]) for r in binding_rows),
        },
        "cues": binding_rows,
    }
    if (binding["counts"]["text_matches_retained_takes"], binding["counts"]["needs_new_oliver_take"]) != (len(unchanged), len(CHANGED)):
        fail("the take binding does not leave exactly 04_4 and 08_5 open")

    # ------------------------------------------------------------ the plan and its delta
    plan = {
        f"{REL}/narration_units.json": units_bytes,
        f"{REL}/docs/{MAP16}": map16,
        "hold-audit.json": hold_bytes,
        "phrase-anchors.json": dump(anchors),
        "phrase-timings-unmeasured-template.json": dump(template),
        "direction-dispositions-proposed.json": dump(disp),
        "cue-delta.json": dump(cue_delta),
        "take-binding.json": dump(binding),
        "inputs.sha256": "".join(f"{v}  {k}\n" for k, v in sorted(inputs.pins.items())).encode("utf-8"),
    }
    stale_tokens = {
        OLD_SCRIPT: "stage08 script",
        OLD_PRODUCT: "stage08 product_rule.rs",
        OLD_UNITS: "stage08 units",
        OLD_MAP: "oliver15's hash",
        OLD_HOLD_AUDIT: "bound-plan03's hold audit",
        MAP15: "oliver15's name",
        **{v["old"]: f"{k}'s stage08 text" for k, v in CHANGED.items()},
        **{v: f"{k[:4]} old take" for k, v in OLD_TAKES.items()},
    }
    allowed = {
        "cue-delta.json": re.compile(r"\$\.current_request_list\.script_sha256|\$\.cues\[\d+\]\.superseded_request\..*"),
        "take-binding.json": re.compile(r"\$\.source\.script_sha256|\$\.cues\[\d+\]\.old_text_takes\[\d+\]\.(sha256|text_sha256)"),
        f"{REL}/docs/{MAP16}": re.compile(r"\$\.scope"),
    }
    leftovers = []
    for name, data in plan.items():
        if name.endswith(".json"):
            found = []
            walk_strings(json.loads(data), "$", found)
            for path, value in found:
                hits = [label for token, label in stale_tokens.items() if token in value]
                if hits and not (name in allowed and allowed[name].fullmatch(path)):
                    leftovers.append(f"{name} {path}: {hits}")
    if leftovers:
        fail("stale values left in the rebound plan:\n  " + "\n  ".join(leftovers))

    files = []
    for name, against, old_bytes in (
        (units_rel, f"{STAGE08_CAND}/{units_rel}", stage08[units_rel]),
        (map16_rel, f"{STAGE08_CAND}/{map15_rel}", stage08[map15_rel]),
        *((name, f"{PLAN3}/{name}", inputs.read(f"{PLAN3}/{name}")) for name in REBOUND),
    ):
        files.append({
            "file": name,
            "against": against,
            "old_sha256": sha(old_bytes),
            "new_sha256": sha(plan[name]),
            "changes": json_changes(json.loads(old_bytes), json.loads(plan[name])),
        })
    changed_files = []
    for rel in (script_rel, product_rel):
        a, b = source_lines(stage08, rel), source_lines(patched, rel)
        changed_files.append({
            "path": rel,
            "old_sha256": sha(stage08[rel]),
            "new_sha256": sha(patched[rel]),
            "lines_changed": [n for n, (x, y) in enumerate(zip(a, b), 1) if x != y],
        })
    changed_files.append({"path": units_rel, "old_sha256": OLD_UNITS, "new_sha256": sha(units_bytes), "lines_changed": None})
    binder_lines = inputs.text(BINDER03).split("\n")
    for line, needle, _ in BINDER03_REFUSALS:
        if needle not in binder_lines[line - 1]:
            fail(f"binder03 line {line} no longer reads {needle!r}")
    delta = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": DELTA_CLAIM,
        "patch": {"path": f"{CLARITY}/logarithm-narration-clarity.patch", "sha256": PATCH_SHA, "against": STAGE08_CAND},
        "script": {"old": OLD_SCRIPT, "new": NEW_SCRIPT},
        "cues_changed": [
            {"cue": c, "old_text_sha256": v["old"], "new_text_sha256": v["new"], "words": list(v["words"])} for c, v in CHANGED.items()
        ],
        "cues_unchanged": len(unchanged),
        "anchors_moved": [
            {"cue": k[0], "const": k[1], "old_byte": b[0], "new_byte": b[1]} for k, b in sorted(MOVED_ANCHORS.items())
        ],
        "candidate": {
            "from": STAGE08_CAND,
            "to": f"{INSTALL}/candidate",
            "files_changed": changed_files,
            "files_added": [{"path": map16_rel, "sha256": sha(map16)}],
            "files_unchanged": sum(1 for rel in stage08 if candidate[rel] == stage08[rel]),
        },
        "plan": {"from": PLAN3, "to": f"{INSTALL}/bound-plan", "files": files, "not_rebound": NOT_REBOUND,
                 "added": {"take-binding.json": TAKE_CLAIM, "clarity-delta.json": "This file."}},
        "binder03": {
            "path": BINDER03,
            "sha256": sha(inputs.read(BINDER03)),
            "not_rerun": [f"line {line}: {why}" for line, _, why in BINDER03_REFUSALS],
            "pins_revised_here": [
                {"input": script_rel, "binder03": OLD_SCRIPT, "stage09": NEW_SCRIPT},
                {"input": product_rel, "binder03": OLD_PRODUCT, "stage09": NEW_PRODUCT},
                {"input": units_rel, "binder03": OLD_UNITS, "stage09": sha(units_bytes)},
                {"input": "MAP_NAME", "binder03": MAP15, "stage09": MAP16},
                {"input": "candidate", "binder03": f"{EL}/combined-repair-stage07/candidate", "stage09": f"{INSTALL}/candidate"},
            ],
            "note": "binder03 stays sealed and unedited. This helper pins stage08 and bound-plan03 instead and rebinds only what the patch moves.",
        },
    }
    plan["clarity-delta.json"] = dump(delta)
    out = {f"candidate/{rel}": data for rel, data in candidate.items()}
    out.update({f"bound-plan/{name}": data for name, data in plan.items()})
    expect = {
        "new": new,
        "old": old,
        "marks": expected_marks,
        "takes": {r["cue"]: {t["sha256"] for t in r["takes"]} for r in binding_rows},
        "all_takes": all_takes,
    }
    return out, expect


def walk_strings(x, path, out):
    if isinstance(x, dict):
        for k, v in x.items():
            walk_strings(v, f"{path}.{k}", out)
    elif isinstance(x, list):
        for i, v in enumerate(x):
            walk_strings(v, f"{path}[{i}]", out)
    elif isinstance(x, str):
        out.append((path, x))


# ---------------------------------------------------------------------------------------- modes


GUARDED = ("explainer_corpus", STAGE08_CAND, PLAN3, CLARITY, TAKES, f"{P}/logarithm-stage07-binder03")


def write_out(out_dir, out):
    target = Path(out_dir)
    resolved = target.resolve()
    for g in GUARDED:
        if resolved == Path(g).resolve() or Path(g).resolve() in resolved.parents:
            fail(f"{out_dir}: refused; this helper never writes in canonical, stage08, bound-plan03, ui-takes01 or a sealed bundle")
    if target.exists():
        fail(f"{out_dir}: exists; give a new folder")
    for rel, data in sorted(out.items()):
        path = target / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "xb") as f:
            f.write(data)
    print(f"wrote {len(out)} files under {out_dir}")
    for rel in sorted(out):
        if not rel.startswith("candidate/") or rel.endswith((MAP16, "narration_units.json", "script.tsv", "product_rule.rs")):
            print(f"{sha(out[rel])}  {rel}")
    return 0


def verify(dir_, out):
    root = Path(dir_)
    if not root.is_dir():
        fail(f"{dir_}: not a folder")
    disk = {p.relative_to(root).as_posix(): p.read_bytes() for sub in ("candidate", "bound-plan") for p in (root / sub).rglob("*") if p.is_file()}
    others = sorted(p.relative_to(root).as_posix() for p in root.iterdir() if p.name not in ("candidate", "bound-plan"))
    if others:
        print(f"NOTE     not compared, outside candidate/ and bound-plan/: {others}")
    problems = []
    for rel in sorted(set(disk) - set(out)):
        problems.append(f"EXTRA    {rel}")
    for rel in sorted(set(out) - set(disk)):
        problems.append(f"MISSING  {rel}")
    for rel in sorted(set(out) & set(disk)):
        if disk[rel] == out[rel]:
            continue
        problems.append(f"DIFFERS  {rel}  (found {sha(disk[rel])[:12]}, want {sha(out[rel])[:12]})")
        if rel.endswith(".json"):
            try:
                rows = json_changes(json.loads(out[rel]), json.loads(disk[rel]))
            except ValueError:
                rows = [{"path": "$", "note": "not JSON"}]
            for row in rows[:12]:
                problems.append(f"         {row.get('path')}: want {json.dumps(row.get('old', row.get('removed')))[:90]}, "
                                f"found {json.dumps(row.get('new', row.get('added')))[:90]}")
    for row in problems:
        print(row)
    print(f"VERIFY {dir_}: " + ("PASS" if not problems else f"FAIL ({sum(1 for r in problems if not r.startswith(' '))} files)"))
    return 0 if not problems else 1


def validate_timings(path, expect, template):
    try:
        doc = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"FAIL {path}: {e}")
        return 1
    problems = []
    new = expect["new"]
    cues = doc.get("cues") if isinstance(doc, dict) else None
    if not isinstance(cues, dict):
        problems.append("no cues object")
        cues = {}
    if doc.get("script_sha256", NEW_SCRIPT) != NEW_SCRIPT:
        problems.append(f"script_sha256 {str(doc.get('script_sha256'))[:12]} is not stage09's 559afa36")
    if set(cues) != set(new):
        problems.append(f"cues: missing {sorted(set(new) - set(cues))}, extra {sorted(set(cues) - set(new))}")
    for cue, text in new.items():
        entry = cues.get(cue)
        if not isinstance(entry, dict):
            continue
        raw = text.encode("utf-8")
        if entry.get("text_sha256") != sha_t(text):
            what = "its stage08 text" if cue in CHANGED and entry.get("text_sha256") == CHANGED[cue]["old"] else "not its stage09 text"
            problems.append(f"{cue}: text_sha256 {str(entry.get('text_sha256'))[:12]} is {what}")
        marks = entry.get("marks")
        if not isinstance(marks, list) or not all(isinstance(m, dict) for m in marks):
            problems.append(f"{cue}: marks is not a list of objects")
            continue
        got = [m.get("byte") for m in marks]
        want = [b for b, _ in expect["marks"][cue]]
        if got != want:
            problems.append(f"{cue}: marks at bytes {got}; the stage09 anchors are {want}")
        for m, (b, phrase) in zip(marks, expect["marks"][cue]):
            if "phrase" in m and m["phrase"] != phrase:
                problems.append(f"{cue}: the mark at {m.get('byte')} names {m['phrase']!r}, not {phrase!r}")
            if m.get("byte") == b and not raw[b:].startswith(phrase.encode("utf-8")):
                problems.append(f"{cue}: byte {b} does not start {phrase!r}")
        wav, secs = entry.get("wav_sha256"), [m.get("seconds") for m in marks]
        if template:
            if wav is not None or any(s is not None for s in secs):
                problems.append(f"{cue}: a template holds no WAV and no seconds")
            continue
        if not isinstance(wav, str) or not re.fullmatch(r"[0-9a-f]{64}", wav):
            problems.append(f"{cue}: wav_sha256 is not a SHA-256")
        elif cue in CHANGED:
            if wav in expect["all_takes"]:
                old_take = " (a take of the stage08 wording)" if wav in OLD_TAKES.values() else ""
                problems.append(f"{cue}: wav_sha256 {wav[:12]} is a retained take{old_take}, not a new take of the reviewed wording")
        elif wav not in expect["takes"][cue]:
            problems.append(f"{cue}: wav_sha256 {wav[:12]} is not a retained Oliver take of this exact text")
        previous = 0.0
        for s in secs:
            if isinstance(s, bool) or not isinstance(s, (int, float)) or not math.isfinite(s) or s < previous:
                problems.append(f"{cue}: seconds {secs} are not finite, non-negative and non-decreasing")
                break
            previous = s
    for row in problems:
        print(f"FAIL {row}")
    print(f"TIMINGS {path}: " + ("PASS" if not problems else f"FAIL ({len(problems)})") + (" (template)" if template else ""))
    return 0 if not problems else 1


def validate_receipts(paths, expect):
    problems = []
    new = expect["new"]
    for path in paths:
        try:
            raw = Path(path).read_text(encoding="utf-8")
            r = json.loads(raw)
        except (OSError, ValueError) as e:
            problems.append(f"{path}: {e}")
            continue
        if not isinstance(r, dict):
            problems.append(f"{path}: not an object")
            continue
        cue = r.get("cue")
        rows = []
        if cue not in CHANGED:
            rows.append(f"cue {cue!r} is not 04_4 or 08_5, the only cues re-recorded")
        else:
            if r.get("text") != new[cue]:
                rows.append("text is not the stage09 text" + (" (it is the stage08 wording)" if r.get("text") == expect["old"][cue] else ""))
            if r.get("text_sha256") != sha_t(new[cue]):
                rows.append(f"text_sha256 {str(r.get('text_sha256'))[:12]} is not {sha_t(new[cue])[:12]}")
        if "script_sha256" in r and r["script_sha256"] != NEW_SCRIPT:
            rows.append(f"script_sha256 {str(r.get('script_sha256'))[:12]} is not 559afa36")
        if r.get("voice") not in OLIVER_LABELS:
            rows.append(f"voice {r.get('voice')!r} is not ElevenLabs Oliver")
        for k, v in OLIVER_SETTINGS.items():
            if r.get(k) != v:
                rows.append(f"{k} {r.get(k)!r}, not {v!r} as the 80 retained takes")
        if r.get("format") not in OLIVER_FORMATS:
            rows.append(f"format {r.get('format')!r} is not one of the retained takes' WAV 48 kHz labels")
        context = [k for k, v in r.items() if CONTEXT_KEY.search(k) and v not in (None, "", [], {})]
        if context:
            rows.append(f"context fields {context}; the retained takes carry none")
        if OTHER_VOICE.search(raw):
            rows.append("names another voice or engine")
        problems += [f"{path}: {row}" for row in rows]
        print(("PASS " if not rows else "FAIL ") + f"{path}: cue {cue}, voice {r.get('voice')!r}")
        for row in rows:
            print(f"     {row}")
    kept = [n for n, h in OLD_TAKES.items() if not Path(f"{TAKES}/{n}").is_file() or sha(Path(f"{TAKES}/{n}").read_bytes()) != h]
    if kept:
        print(f"FAIL original takes missing or changed: {kept}")
    else:
        print("PASS the four original 04_4 and 08_5 Oliver takes are present with their hashes")
    print("RECEIPTS: " + ("PASS" if not problems and not kept else "FAIL"))
    return 0 if not problems and not kept else 1


USAGE = __doc__.split("\n\n")[1]


def main(argv):
    if not Path("tools/coherent_narration.py").is_file() or not Path(P).is_dir():
        print("integrate_clarity: run from the repo root", file=sys.stderr)
        return 2
    if argv == ["--print-pins"]:
        return print_pins()
    modes = {"--out": 1, "--check": 0, "--verify": 1, "--validate-timings": None, "--validate-receipt": None}
    if not argv or argv[0] not in modes:
        print(USAGE, file=sys.stderr)
        return 2
    mode, rest = argv[0], argv[1:]
    template = False
    if mode == "--validate-timings":
        template = "--template" in rest
        rest = [a for a in rest if a != "--template"]
        if len(rest) != 1:
            print(USAGE, file=sys.stderr)
            return 2
    elif mode == "--validate-receipt":
        if not rest:
            print(USAGE, file=sys.stderr)
            return 2
    elif len(rest) != modes[mode]:
        print(USAGE, file=sys.stderr)
        return 2
    inputs = Inputs()
    out, expect = build(inputs)
    if mode == "--out":
        return write_out(rest[0], out)
    if mode == "--check":
        return verify(DELIVERED, out)
    if mode == "--verify":
        return verify(rest[0], out)
    if mode == "--validate-timings":
        return validate_timings(rest[0], expect, template)
    return validate_receipts(rest, expect)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
