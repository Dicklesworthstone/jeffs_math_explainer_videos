#!/usr/bin/env python3
"""Bind complex_analysis_visual's combined-repair stage04 (and stage03) to exact source, strictly.

    python3 -I -B bind_complex_stage03.py [--stage stage04] [--out DIR] [--map-version oliver12] [--check]
    python3 -I -B bind_complex_stage03.py --stage stage03 [--out DIR] [--check]
    python3 -I -B bind_complex_stage03.py --print-pins

Run from the repository root (or pass --repo). Every path below is relative to it.
stage04 is the default and the plan meant for installation (map oliver12). stage03 is
kept as evidence; its map is labelled oliver12-stage03 so that no two different
oliver12 maps exist.

What it does, in order:

1. Pins. Every input it reads is pinned by SHA-256: root's stage03 and stage04 receipts
   (which pin all 55 files of each stage), the binder copy, RubyRaven's sealed original
   of it, the kit's kit.rs and narration.rs, Lavender's proposed hold delta, teaching02's
   oliver11 map and units patch, Lavender's four review-fixes patches, and the sealed
   oliver10 baseline in baseline/. One changed byte in any of them exits 3 and writes
   nothing.
2. Receipt. Every file the receipt lists must be on disk with its hash, and nothing may
   be on disk that it does not list. For stage03, a file that differs from the receipt's
   baseline must be named in its `changed` list; its one known omission,
   src/plate_layout.rs, is recorded as a finding. For stage04, every file that differs
   from stage03 must be named in `changed` and be one of DECLARED_SUCCESSOR's four, going
   from exactly its stage03 hash to exactly its declared hash; the Rust files' film code
   (everything before the `#[cfg(test)]` / `mod tests {` block, the split the crate's own
   tests use) must be byte-equal to stage03's.
3. Script. stage03's script.tsv must be 1e24aba8 exactly. stage04's must be stage03's
   with the one declared F3 change: in cue 15_2, "though not always everywhere" becomes
   "though there may be none" (12442480). Any other script refuses.
4. RubyRaven's binder (binder/bind_oliver_plan.py, a byte copy) builds the units, the
   map and its source audit. Every problem it raises refuses.
5. Strict checks the binder does not make:
   S1  every phrase a hook waits for or reads occurs exactly once in its cue and starts
       a word (Spoken::at finds the first occurrence).
   S2  every cue's hooks, in execution order with loops unrolled and helpers expanded at
       their call sites, land in spoken order.
   S3  every non-literal hook value in the binder's DYNAMIC table is re-derived from the
       source text (the loop's array, the const it zips or indexes, or the literal at
       each helper call site) and must equal the table.
   S4  closing's two post-voice beats: after the cue's voice-end wait, only comments, the
       authored pause and kit.hold may come before the next cue. Nothing may arrive.
   S5  each chapter's closing tail (the last cue's waits, breaths and finish), against
       the oliver10 baseline.
   S6  every cue whose words changed since oliver10 must have a reviewed hook table in
       REVIEWED_HOOKS below, and its hooks must still match it.
6. Cross-checks. The map must equal Lavender's 25-row delta applied to teaching02's
   oliver11, byte for byte. The units, pointed at teaching02's oliver11, must equal
   teaching02's patched plan 3989e185. For stage04 both proposals are carried forward by
   the declared delta only (F3's script hash, the six rows' file hashes from stage04's
   receipt, 15_2's clause). These check the binding; they edit nothing.
   For stage04 the plan may also differ from stage03's only in the script hashes, the
   15_2 unit's text under F3, the map hash in the units, and the file hash (not line,
   not marker) of exactly the six declared rows. Anything else refuses.
7. Delta. delta-from-oliver10.json lists every change against the sealed oliver10
   baseline: the script, units, hold rows, helpers, hooks per cue, closing tails,
   dynamic values and post-voice beats. delta-from-stage03.json (stage04 only) lists
   every plan difference from stage03's binding, each marked allowed.

Outputs (in OUT, at paths that mirror the canonical tree for the plan files):

    explainer_corpus/complex_analysis_visual/narration_units.json
    explainer_corpus/complex_analysis_visual/docs/narration-direction-map-<version>.json
    source-audit.json            (the binder's)
    strict-audit.json            (this tool's checks, the hook tables, findings)
    delta-from-oliver10.json
    delta-from-stage03.json      (stage04 only)

OUT may not lie in any explainer_corpus tree, in either stage's root (a file there would
break its receipt check), or in another bundle under the opus-replacements folder. A
sibling path such as <L>/combined-repair-stage04-bound-plan01 is fine. A file in OUT with
other bytes is refused, never overwritten. --check writes nothing and exits 1 unless OUT
already holds exactly these bytes.

Exit codes: 0 bound or in step; 1 refused (problems listed) or stale OUT; 2 usage or
OUT refused; 3 a pinned input changed.

Source text only. Nothing is compiled, rendered, synthesised or heard. A word moment
is an estimate until Oliver's takes are aligned; no timing here is measured.
"""

import argparse
import bisect
import collections
import hashlib
import importlib.util
import json
import os
import re
import sys

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
L = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
HERE = f"{P}/complex-stage03-binder01"
TOPIC = "complex_analysis_visual"
CRATE_REL = f"explainer_corpus/{TOPIC}"
STAGE03 = f"{L}/combined-repair-stage03"
STAGE03_RECEIPT = f"{L}/combined-repair-stage03-integration01.json"
STAGE04 = f"{L}/combined-repair-stage04"
STAGE04_RECEIPT = f"{L}/combined-repair-stage04-integration01.json"
BINDER = f"{HERE}/binder/bind_oliver_plan.py"
RUBY_ORIGINAL = f"{P}/complex02-05-cross-review04/binder/bind_oliver_plan.py"
BASE = f"{HERE}/baseline/canonical-oliver10"
LAVENDER_DELTA = f"{P}/complex-stage03-cross-review01/proposed/hold-rebind-stage03.json"
TEACHING02_MAP = f"{P}/complex11-counting-teaching02/holds/narration-direction-map-oliver11.json"
TEACHING02_UNITS_PATCH = f"{P}/complex11-counting-teaching02/patches/narration-units.patch"
# The plan teaching02's units patch gives on stage03's plan: teaching02 HANDOFF line 25,
# confirmed by Lavender (REVIEW section E) and by applying the patch to a scratch copy.
TEACHING02_UNITS_SHA = "3989e1855f617734c38caae8d0113d4915b5c44fbde600f674e5779ed4977f28"
TEACHING02_MAP_SHA = "759e0eba1d40fbd2ff4f715719493dccc35eed27da0f730c6e8c8344a7f1bb91"

PINS = {
    BINDER: "26e10809c39b497cec168e19f462ae13cef74f74bcd671db1ab54fbe07e2efd0",
    RUBY_ORIGINAL: "26e10809c39b497cec168e19f462ae13cef74f74bcd671db1ab54fbe07e2efd0",
    STAGE03_RECEIPT: "3f652c2d8bc510fbbd62e027b9db57943d8d5a1c3e297c0fecdfb87de6dec5e2",
    STAGE04_RECEIPT: "295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b",
    "crates/explainer_kit/src/kit.rs": "ef035faf236498e7485fb60a66d862b383e1a5040fa17962e1633596ed99e106",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    LAVENDER_DELTA: "aa7450d4c7908e23a35548edb0545d6c51fe5e21011509f9ca0761b77191f9ea",
    TEACHING02_MAP: TEACHING02_MAP_SHA,
    TEACHING02_UNITS_PATCH: "34199ed235b12d8a243a47c22550f76af63963418c668f46286478ff62df2c23",
    f"{BASE}/narration_units.json": "3c4f53d7973857295e4f99de681e8537ead08676d1ebdfe9625e69b4bc86702a",
    f"{BASE}/narration-direction-map-oliver10.json": "341356cda34ac129fbca1988699f9f20f1cb583d63a5463f780226a13ab3c5b4",
    f"{BASE}/source-audit.json": "d3423e0d20d4067a3c44e5e1b828795f6380ec395ae1ebf9a4de0e7b256d425f",
    f"{BASE}/script.tsv": "2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e",
}
STAGE03_SCRIPT_SHA = "1e24aba8b520f2e61e8b2bc392a218b664cf5556ad788ceca17fbc45bb92efab"

# The one deliberate script change this tool accepts (Lavender's F3, accepted by root).
F3 = {
    "finding": "complex-stage03-cross-review01 F3",
    "cue": "15_2",
    "old": "though not always everywhere",
    "new": "though there may be none",
    "script_sha256": "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3",
}

# The one successor this tool binds: root's planned stage04, which is stage03 plus
# LavenderPike's four sealed patches (complex-stage03-review-fixes01, SHA256SUMS checked).
# Each file must go from exactly its stage03 bytes to exactly these; no other file of the
# film may change. Film code (everything before `#[cfg(test)]` + newline + `mod tests {`)
# of each Rust file must stay byte-equal. The first bare `#[cfg(test)]` is not the split:
# draw.rs has a `#[cfg(test)] pub fn contains` at line 416, and two_planes.rs,
# one_derivative.rs and real_integral.rs carry test-only items near their tops.
# A later stage needs a new, reviewed declaration in a new bundle.
TESTS_BLOCK = "\n#[cfg(test)]\nmod tests {"


def film_code(text):
    """Text before the file's test module, or None when the file has none or several."""
    if text.count(TESTS_BLOCK) != 1:
        return None
    return text[: text.index(TESTS_BLOCK)]



REVIEW_FIXES = f"{P}/complex-stage03-review-fixes01"
DECLARED_SUCCESSOR = {
    "name": "stage04: stage03 + complex-stage03-review-fixes01 F3 F4 F5 F6",
    "patches": {
        f"{REVIEW_FIXES}/patches/F3-15_2-clause.patch": "0fa7cb79a41615885a96f7917832ac4021ce172b86c8edffa18067f1ac4b2320",
        f"{REVIEW_FIXES}/patches/F4-turning-one-sample-guard.patch": "8e90cf7e7fa0aa47813a2ac5e07ce66ce2637f07b02d98c359487da5f03328b8",
        f"{REVIEW_FIXES}/patches/F5-budget-painted-planes.patch": "f3f5907519ed2234046ba82a87cb6e6bf6d24947525d5e99a3eb026c4e1dcd7a",
        f"{REVIEW_FIXES}/patches/F6-storyboard-quotes.patch": "ad92990c8217ea74656d981eebb8591c390e12eb33e205a76340a5bd89018341",
    },
    "files": {
        f"{CRATE_REL}/script.tsv": (
            "1e24aba8b520f2e61e8b2bc392a218b664cf5556ad788ceca17fbc45bb92efab",
            "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3",
        ),
        f"{CRATE_REL}/docs/storyboard.md": (
            "57508d96103f0a4a86ed2265a51bb65d28fc68d8e6c96b3bd4438d2d97c83b5f",
            "e204cf7726dadefe46dccb419e0cc0a641acbeeb4e884c3f55ff6d39a980536b",
        ),
        f"{CRATE_REL}/src/chapters/counting.rs": (
            "c1df16ea04d4283d490598ccabe039cdc846be96458668d63544aef9ac81d36a",
            "d41d414134c4ffc553df9773bc1b082fab376f1a4344d248ab7bdb3f443fe8f9",
        ),
        f"{CRATE_REL}/src/chapters/draw.rs": (
            "865599e94331038782be128189fc1acb642e1d00b2f7fe32bd9fe68c958951b9",
            "3f1c581dbc028a4d1a50a46d15028a0e005e0a76cf58709c6fbbbce2d30ea2db",
        ),
    },
    # The map rows whose file hash moves, and nothing else of them: the three counting.rs
    # holds and the three draw.rs helpers (Lavender's HANDOFF, "What the hold rows need").
    "row_sha_moves": [
        ["visual_holds", "11_5", "prediction"],
        ["visual_holds", "11_6", "reading"],
        ["visual_holds", "11_8", "reading"],
        ["helpers", f"{CRATE_REL}/src/chapters/draw.rs", "pub fn predict"],
        ["helpers", f"{CRATE_REL}/src/chapters/draw.rs", "pub fn reading_window"],
        ["helpers", f"{CRATE_REL}/src/chapters/draw.rs", "pub fn precisely_over"],
    ],
}
PINS.update(DECLARED_SUCCESSOR["patches"])
# Files a successor may lack or keep (Lavender F2: not in the film; the binder reads none).
F2_STRAYS = {
    f"{CRATE_REL}/src/chapters/counting.rs.orig",
    f"{CRATE_REL}/src/chapters/mod.rs.orig",
    f"{CRATE_REL}/src/chapters/one_derivative.rs.orig",
    f"{CRATE_REL}/.ee/hook-state/session_start_orient.last.json",
}

# Changes stage03's receipt does not name in `changed`, recorded rather than refused.
KNOWN_UNNAMED_STAGE03 = {f"{CRATE_REL}/src/plate_layout.rs": "new file, not in the receipt's changed list"}

# Map names already used for complex_analysis_visual somewhere in P (oliver11 three times).
TAKEN_MAP_VERSIONS = {"oliver11"}

# Reviewed hooks of every cue whose words changed since oliver10. Each row is
# (call, value, sentence index, the words that sentence starts with), in execution
# order. Read against the source and the cue's text; see HANDOFF.md.
REVIEWED_HOOKS = {
    "11_7": [
        ("wait_for", "complex polynomial", 0, "The same trick proves"),
        ("sentence", 1, 1, "Divide by its leading coefficient"),
        ("wait_sentence", 1, 1, "Divide by its leading coefficient"),
        ("until", "which leaves", 1, "Divide by its leading coefficient"),
        ("wait_for", "with z", 2, "Now it starts with z"),
        ("wait_sentence", 3, 3, "On a huge circle"),
        ("wait_sentence", 4, 4, "Picture z to the n as a walker"),
        ("sentence", 6, 6, "So there are exactly n roots inside"),
    ],
    "15_2": [
        ("wait_sentence", 1, 1, "The complex derivative must be the same"),
        ("wait_sentence", 2, 2, "On an open region"),
        ("wait_sentence", 3, 3, "It lets boundary values decide"),
        ("wait_sentence", 4, 4, "It lets a loop count zeros"),
        ("wait_for", "and it turns a real integral", 4, "It lets a loop count zeros"),
        ("wait_sentence", 5, 5, "It allows at most one extension"),
    ],
}

HOOKS = {"wait_for", "until", "at", "wait_sentence", "sentence", "wait_last"}
ARG = {"wait_for": 1, "until": 1, "at": 0, "wait_sentence": 1, "sentence": 0}
CLAIM = (
    "Text audit of Rust source and plan files. Uncompiled, unrendered, unsynthesised, unheard. "
    "Word moments are estimates until Oliver's takes are aligned. No take, duration, listening, "
    "picture or independent acceptance."
)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def dumps(value):
    return json.dumps(value, indent=2, ensure_ascii=False) + "\n"


class Refuse(Exception):
    pass


# --------------------------------------------------------------------- pins

def current_pins(repo):
    out = {}
    for rel in PINS:
        path = os.path.join(repo, rel)
        out[rel] = sha(read_bytes(path)) if os.path.isfile(path) else None
    return out


def stale_pins(repo):
    now = current_pins(repo)
    return [(rel, PINS[rel], now[rel]) for rel in PINS if now[rel] != PINS[rel]]


def load_binder(repo):
    spec = importlib.util.spec_from_file_location("bind_oliver_plan_pinned", os.path.join(repo, BINDER))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# --------------------------------------------------------------------- receipts

def tree_listing(root):
    found = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for name in sorted(filenames):
            found.append(os.path.relpath(os.path.join(dirpath, name), root).replace(os.sep, "/"))
    return sorted(found)


def check_receipt(repo, stage_root, receipt, problems, label):
    """Every listed file on disk with its hash, nothing unlisted on disk."""
    root = os.path.join(repo, stage_root)
    files = receipt.get("files")
    if not isinstance(files, list) or not all(isinstance(f, dict) and "path" in f and "sha256" in f for f in files):
        problems.append(f"{label}: receipt has no `files` list of path and sha256")
        return {}
    listed = {f["path"]: f["sha256"] for f in files}
    for rel, want in sorted(listed.items()):
        path = os.path.join(root, rel)
        if not os.path.isfile(path):
            problems.append(f"{label}: receipt lists {rel}, which is not on disk")
        elif sha(read_bytes(path)) != want:
            problems.append(f"{label}: {rel} differs from its receipt hash")
    on_disk = set(tree_listing(root))
    for rel in sorted(on_disk - set(listed)):
        problems.append(f"{label}: {rel} is on disk but not in the receipt")
    return listed


def unnamed_changes(repo, baseline_root, listed, changed):
    """Files the receipt lists that differ from its baseline, and are not named in changed."""
    out = []
    base = os.path.join(repo, baseline_root)
    for rel, want in sorted(listed.items()):
        path = os.path.join(base, rel)
        before = sha(read_bytes(path)) if os.path.isfile(path) else None
        if before != want and rel not in changed:
            out.append({"path": rel, "before": before, "after": want})
    return out


# --------------------------------------------------------------------- script

def script_delta(data, stage03_bytes):
    if data == stage03_bytes:
        return "none"
    old, new = F3["old"].encode(), F3["new"].encode()
    if stage03_bytes.count(old) == 1 and data == stage03_bytes.replace(old, new) and sha(data) == F3["script_sha256"]:
        return "F3"
    return None


# --------------------------------------------------------------------- strict source analysis

def const_body(raw, name):
    m = re.search(r"(?:pub )?const " + re.escape(name) + r": \[([^\]]*?); (\d+)\] = \[(.*?)\];", raw, re.S)
    if not m:
        return None
    return {"type": m.group(1), "length": int(m.group(2)), "body": m.group(3)}


def const_strings(B, raw, name):
    c = const_body(raw, name)
    if c is None:
        return None
    items = B.STRING.findall(c["body"])
    return items if len(items) == c["length"] else None


def const_tuple_firsts(raw, name):
    c = const_body(raw, name)
    if c is None:
        return None
    items = re.findall(r'\(\s*"((?:[^"\\]|\\.)*)"\s*,', c["body"])
    return items if len(items) == c["length"] else None


def const_length(raw, name):
    c = const_body(raw, name)
    return c["length"] if c else None


def loops_in(B, t, a, b):
    """(for-keyword, body open brace, body close brace) of each for loop in t[a:b]."""
    out = []
    for m in re.finditer(r"\bfor\b", t[a:b]):
        s = a + m.start()
        depth, i = 0, s + 3
        while i < b:
            ch = t[i]
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif ch == "{" and depth == 0:
                break
            i += 1
        if i >= b:
            continue
        out.append((s, i, B.close_of(t, i)))
    return out


def norm(text):
    return " ".join(text.split())


class Strict:
    """Hooks per cue, in execution order, from one crate's registered chapter files."""

    def __init__(self, B, crate, script):
        self.B = B
        self.script = script
        self.problems = []
        self.derived = {}
        self.by_owner = collections.defaultdict(lambda: collections.defaultdict(list))
        chapters = os.path.join(crate, "src", "chapters")
        registered = [f"{m}.rs" for m in B.registered_modules(B.text_of(os.path.join(chapters, "mod.rs")))]
        scratch = B.Plan()
        self.src = {}
        for name in sorted(registered):
            if name in B.NOT_CHAPTERS:
                continue
            self.src[name] = B.Source(name, B.text_of(os.path.join(chapters, name)), scratch)
        self.derive_dynamic()
        self.events = {}
        for name, src in self.src.items():
            self.sequence_file(name, src)

    # ---- S3: re-derive DYNAMIC values from the source text
    def enclosing_loop(self, src, call):
        fn = call["fn"]
        best = None
        for a, b in src.fns.get(fn, []):
            if a <= call["at"] <= b:
                for s, o, c in loops_in(self.B, src.t, a, b):
                    if o < call["at"] < c and (best is None or s > best[0]):
                        best = (s, o, c)
        return best

    def header(self, src, loop):
        return norm(src.raw[loop[0] : loop[1]])

    def derive_dynamic(self):
        B = self.B
        for (name, source), entry in sorted(B.DYNAMIC.items()):
            src = self.src.get(name)
            calls = [c for c in src.calls if c["source"] == source] if src else []
            key = (name, source)
            if not calls:
                self.problems.append(f"S3 {name}: DYNAMIC's call is not in the source: {source}")
                continue
            if "by_owner" in entry:
                self.derived[key] = {"how": "helper parameter, from the literal at each call site", "values": None}
                continue
            got = None
            how = None
            loop = self.enclosing_loop(src, calls[0])
            head = self.header(src, loop) if loop else ""
            try:
                if name == "the_bump.rs" and source == "line.wait_for(stage, phrase)?;":
                    m = re.fullmatch(r"for \((\w+), phrase\) in \[(.*)\]", head)
                    if m:
                        got = re.findall(r'\(\s*[^,()]+,\s*"((?:[^"\\]|\\.)*)"\s*\)', m.group(2))
                        how = f"loop array: {head}"
                elif name == "gallery.rs" and source == "line.wait_for(stage, phrase)?;":
                    fn_span = next((s for s in src.fns.get(calls[0]["fn"], []) if s[0] <= calls[0]["at"] <= s[1]), None)
                    guard = src.raw[max(0, calls[0]["at"] - 200) : calls[0]["at"]]
                    named = re.findall(r"let named = \[(.*?)\];", src.raw[fn_span[0] : calls[0]["at"]]) if fn_span else []
                    if (
                        len(named) == 1
                        and "if let Some(phrase) = named.get(k) {" in guard
                        and re.fullmatch(r'for \(k, &p0\) in fact_list\("gallery\.gamma\.poles"\)\?\.iter\(\)\.enumerate\(\)', head)
                    ):
                        got = B.STRING.findall(named[0])
                        how = "named.get(k) over fact_list(\"gallery.gamma.poles\"): the first len(named) poles are named"
                elif name == "counting.rs" and source == "line.wait_sentence(stage, i + 2)?;":
                    m = re.fullmatch(r"for \(i, \(f, name\)\) in \[([^\]]*)\] \.into_iter\(\) \.zip\((\w+)\) \.enumerate\(\)", head)
                    if m:
                        n = min(len([x for x in m.group(1).split(",") if x.strip()]), const_length(src.raw, m.group(2)) or 0)
                        got = [k + 2 for k in range(n)]
                        how = f"i over {n} iterations: {head}"
                elif name == "counting.rs" and source == "let until = if i < 2 { line.sentence(i + 3)? } else { line.end() };":
                    m = re.fullmatch(r"for \(i, \(f, name\)\) in \[([^\]]*)\] \.into_iter\(\) \.zip\((\w+)\) \.enumerate\(\)", head)
                    if m:
                        n = min(len([x for x in m.group(1).split(",") if x.strip()]), const_length(src.raw, m.group(2)) or 0)
                        got = [k + 3 for k in range(n) if k < 2]
                        how = f"i < 2 over {n} iterations; the last walk lands on line.end()"
                elif name == "counting.rs" and source == "line.wait_for(stage, NEEDS_SAID[i])?;":
                    m = re.fullmatch(r"for \(i, n\) in (\w+)\.iter\(\)\.enumerate\(\)", head)
                    said = const_strings(B, src.raw, "NEEDS_SAID")
                    if m and said is not None:
                        n = const_length(src.raw, m.group(1))
                        if n is not None and n <= len(said):
                            got = said[:n]
                            how = f"NEEDS_SAID[i] for i over {m.group(1)} ({n})"
                elif name == "reading_color.rs" and source == "line.wait_for(stage, phrase)?;":
                    m = re.fullmatch(r"for \(&m, \(phrase, th\)\) in names\.iter\(\)\.zip\((\w+)\)", head)
                    named = const_tuple_firsts(src.raw, m.group(1)) if m else None
                    built = re.search(r"for \(i, d\) in (\w+)\.iter\(\)\.enumerate\(\) \{\s*names\.push\(", src.raw)
                    if named is not None and built:
                        n = min(len(named), const_length(src.raw, built.group(1)) or 0)
                        got = named[:n]
                        how = f"{m.group(1)}'s phrases zipped with names from {built.group(1)} ({n})"
            except (AttributeError, TypeError, IndexError):
                got = None
            if got is None:
                self.problems.append(f"S3 {name}: could not re-derive DYNAMIC's values for {source} from the source; read it")
                continue
            self.derived[key] = {"how": how, "values": got}
            table = entry.get("phrases") or entry.get("sentences")
            if got != table:
                self.problems.append(f"S3 {name}: source gives {got} for {source}, DYNAMIC says {table}")

    # ---- event sequences
    def hook_fns(self, src):
        direct = {c["fn"] for c in src.calls if c["recv"] == "line" and c["name"] in HOOKS and c["fn"] not in (None, "construct")}
        return sorted(direct)

    def params(self, src, fn):
        m = re.search(r"\bfn\s+" + re.escape(fn) + r"\s*(?:<[^>]*>)?\s*\(", src.t)
        if not m:
            return []
        parts = self.B.split_args(src.raw, src.t, m.end() - 1)
        return [p.split(":")[0].replace("mut ", "").strip() for p in parts]

    def resolve(self, src, call, subst, k):
        """(kind, value) or None when this iteration does not make the call."""
        name = call["name"]
        if name == "wait_last":
            return ("last", None)
        given = call["args"][ARG[name]] if len(call["args"]) > ARG[name] else ""
        lit = self.B.literal(given)
        if name in ("wait_for", "until", "at") and lit is not None:
            return ("phrase", lit)
        if name in ("wait_sentence", "sentence") and re.fullmatch(r"\d+", given):
            return ("sentence", int(given))
        if given in subst:
            return ("phrase" if name in ("wait_for", "until", "at") else "sentence", subst[given])
        key = (src.name, call["source"])
        d = self.derived.get(key)
        if d is None or d["values"] is None:
            self.problems.append(f"S3 {src.name}:{call['line']} hook value not derived: {call['source']}")
            return None
        if k is None:
            self.problems.append(f"S2 {src.name}:{call['line']} dynamic hook outside a loop: {call['source']}")
            return None
        if k >= len(d["values"]):
            return None
        return ("phrase" if name in ("wait_for", "until", "at") else "sentence", d["values"][k])

    def items_of(self, src, fn, span):
        items = []
        for c in src.calls:
            if c["fn"] == fn and c["recv"] == "line" and c["name"] in HOOKS and span[0] <= c["at"] <= span[1]:
                items.append((c["at"], "hook", c))
        for h in self.hook_fns(src):
            if h == fn:
                continue
            for m in re.finditer(r"(?<![\w.])(?:Self::)?" + re.escape(h) + r"\s*\(", src.t):
                if span[0] <= m.start() <= span[1] and src.t[max(0, m.start() - 3) : m.start()] != "fn ":
                    items.append((m.start(), "helper", (h, m)))
        items.sort(key=lambda row: row[0])
        return items

    def run_fn(self, src, fn, subst, depth, via):
        if depth > 3:
            self.problems.append(f"S2 {src.name}: helpers nest deeper than 3 at {fn}")
            return []
        spans = src.fns.get(fn, [])
        if not spans:
            return []
        span = spans[0]
        items = self.items_of(src, fn, span)
        loops = loops_in(self.B, src.t, *span)

        def loop_of(at):
            inner = [lp for lp in loops if lp[1] < at < lp[2]]
            return max(inner, key=lambda lp: lp[0]) if inner else None

        def dynamic(item):
            if item[1] != "hook":
                return False
            c = item[2]
            if c["name"] == "wait_last":
                return False
            given = c["args"][ARG[c["name"]]] if len(c["args"]) > ARG[c["name"]] else ""
            return self.B.literal(given) is None and not re.fullmatch(r"\d+", given) and given not in subst

        out = []
        done = set()
        for item in items:
            lp = loop_of(item[0])
            if lp is not None and lp in done:
                continue
            members = [it for it in items if loop_of(it[0]) == lp] if lp is not None else [item]
            if lp is not None and any(dynamic(it) for it in members):
                done.add(lp)
                n = 0
                for it in members:
                    if dynamic(it):
                        d = self.derived.get((src.name, it[2]["source"]))
                        n = max(n, len(d["values"]) if d and d["values"] else 0)
                for k in range(n):
                    for it in members:
                        out.extend(self.emit(src, it, subst, k, depth, via, f"loop iteration {k}"))
            else:
                out.extend(self.emit(src, item, subst, None, depth, via, "in a loop, source order" if lp else None))
        return out

    def emit(self, src, item, subst, k, depth, via, note):
        at, kind, obj = item
        if kind == "hook":
            got = self.resolve(src, obj, subst, k)
            if got is None:
                return []
            ev = {
                "pos": at,
                "file": src.name,
                "line": obj["line"],
                "call": obj["name"],
                "kind": got[0],
                "value": got[1],
                "source": obj["source"],
            }
            if via:
                ev["via"] = via
            if note:
                ev["note"] = note
            return [ev]
        h, m = obj
        args = self.B.split_args(src.raw, src.t, m.end() - 1)
        names = self.params(src, h)
        if names and names[0] == "&self":
            names = names[1:]
        sub = {}
        for p, a in zip(names, args):
            lit = self.B.literal(a)
            if lit is not None:
                sub[p] = lit
        evs = self.run_fn(src, h, sub, depth + 1, f"{h} called at {src.name}:{src.line_at(m.start())}")
        for e in evs:
            e["pos"] = at
            e["call_site_line"] = src.line_at(m.start())
            e["subst"] = {p: v for p, v in sub.items()}
            if note and "note" not in e:
                e["note"] = note
        return evs

    def sequence_file(self, name, src):
        says = []
        for c in src.calls:
            if c["name"] == "say" and c["fn"] == "construct":
                m = self.B.SAY_STATEMENT.fullmatch(c["source"])
                if m:
                    says.append((c["at"], m.group(1)))
        says.sort()
        positions = [a for a, _ in says]
        evs = self.run_fn(src, "construct", {}, 0, None)
        for cue in [c for _, c in says]:
            self.events.setdefault(cue, [])
        for e in evs:
            i = bisect.bisect_right(positions, e["pos"]) - 1
            if i < 0:
                self.problems.append(f"S2 {name}:{e['line']} hook before any cue")
                continue
            cue = says[i][1]
            self.events[cue].append(e)
            if "subst" in e:
                key = (name, e["source"])
                self.by_owner[key][cue].append(e["value"])

    # ---- S1, S2, S3 by_owner, and the hook table
    def check(self):
        B = self.B
        table = {}
        checked_phrases = 0
        for cue, evs in sorted(self.events.items()):
            text = self.script.get(cue)
            if text is None:
                continue
            offs = B.sentence_offsets(text)
            rows = []
            last = None
            for e in evs:
                if e["kind"] == "phrase":
                    phrase = e["value"]
                    count = sum(1 for i in range(len(text)) if text.startswith(phrase, i))
                    at = text.find(phrase)
                    checked_phrases += 1
                    if count != 1:
                        self.problems.append(f"S1 {cue} {e['file']}:{e['line']}: phrase {phrase!r} occurs {count} times in the cue")
                    if at > 0 and text[at - 1].isalnum():
                        self.problems.append(f"S1 {cue} {e['file']}:{e['line']}: phrase {phrase!r} does not start a word")
                    if at < 0:
                        continue
                    pos = at
                elif e["kind"] == "last":
                    pos = offs[-1]
                else:
                    j = e["value"]
                    if not isinstance(j, int) or j >= len(offs):
                        self.problems.append(f"S2 {cue} {e['file']}:{e['line']}: sentence {j} of {len(offs)}")
                        continue
                    pos = offs[j]
                s = bisect.bisect_right(offs, pos) - 1
                end = offs[s + 1] if s + 1 < len(offs) else len(text)
                row = {
                    "file": e["file"],
                    "line": e["line"],
                    "call": e["call"],
                    "value": e["value"],
                    "char": pos,
                    "sentence": s,
                    "sentence_text": text[offs[s] : end].strip(),
                }
                for extra in ("via", "call_site_line", "note"):
                    if extra in e:
                        row[extra] = e[extra]
                if last is not None and pos < last["char"]:
                    self.problems.append(
                        f"S2 {cue} {e['file']}:{e['line']}: {e['call']} {e['value']!r} lands at char {pos}, "
                        f"before the previous hook ({last['call']} {last['value']!r} at char {last['char']}, line {last['line']})"
                    )
                rows.append(row)
                last = row
            table[cue] = rows
        for (name, source), entry in sorted(B.DYNAMIC.items()):
            if "by_owner" not in entry:
                continue
            got = {cue: vals for cue, vals in sorted(self.by_owner.get((name, source), {}).items())}
            want = {cue: list(vals) for cue, vals in sorted(entry["by_owner"].items())}
            self.derived[(name, source)]["values"] = got
            if got != want:
                self.problems.append(f"S3 {name}: call-site literals give {got} for {source}, DYNAMIC says {want}")
        return table, checked_phrases


def post_voice(B, strict, script):
    """S4: after each beat's voice-end wait, only comments, the pause, kit.hold, the next say."""
    rows = []
    cues = list(script)
    for (name, cue, call), why in sorted(B.POST_VOICE_BEATS.items()):
        src = strict.src.get(name)
        said = [c for c in src.calls if c["name"] == "say" and c["fn"] == "construct"] if src else []
        mine = [c for c in said if B.SAY_STATEMENT.fullmatch(c["source"]) and B.SAY_STATEMENT.fullmatch(c["source"]).group(1) == cue]
        if len(mine) != 1:
            strict.problems.append(f"S4 {name}: cue {cue} is not said exactly once")
            continue
        at = mine[0]["at"]
        nxt = cues[cues.index(cue) + 1] if cues.index(cue) + 1 < len(cues) else None
        follow = [c for c in said if c["at"] > at]
        if not follow:
            strict.problems.append(f"S4 {name}: nothing is said after {cue}")
            continue
        nat = min(c["at"] for c in follow)
        nsay = B.SAY_STATEMENT.fullmatch(next(c for c in follow if c["at"] == nat)["source"])
        end = src.t.find(B.VOICE_END, at, nat)
        if end < 0:
            strict.problems.append(f"S4 {name}: {cue} has no voice-end wait before the next say")
            continue
        tail = src.t[end + len(B.VOICE_END) : nat]
        m = re.fullmatch(r"\s*pause\(stage, (\d+(?:\.\d+)?)\)\?;\s*kit\.hold\(stage\)\?;\s*let line =\s*", tail)
        if m is None or call != f"pause(stage, {m.group(1)})?;":
            strict.problems.append(
                f"S4 {name}: between {cue}'s voice end and {nxt} the source is not just `{call}` and kit.hold: {norm(tail)[:120]}"
            )
            continue
        if nsay is None or nsay.group(1) != nxt:
            strict.problems.append(f"S4 {name}: the say after {cue}'s beat is not {nxt}")
            continue
        last_hook = strict.events.get(cue, [])[-1] if strict.events.get(cue) else None
        end_line = src.line_at(end)
        arrivals = []
        if last_hook:
            for number in range(last_hook["line"] + 1, end_line):
                text = src.lines[number - 1].strip()
                fade = re.match(r"fade_in_all\(stage, &?[\w\[\]., ]+, (\d+(?:\.\d+)?)\)\?;", text)
                if fade:
                    arrivals.append({"line": number, "marker": text, "seconds": float(fade.group(1))})
        rows.append(
            {
                "file": name,
                "cue": cue,
                "next_cue": nxt,
                "voice_end_line": end_line,
                "beat_line": src.line_at(end + len(B.VOICE_END) + tail.index("pause")),
                "beat": call,
                "beat_seconds_in_source": float(m.group(1)),
                "hold_line": src.line_at(end + len(B.VOICE_END) + tail.index("kit.hold")),
                "between_voice_end_and_next_say": "only comments, the beat and kit.hold",
                "last_hook": {k: last_hook[k] for k in ("line", "call", "value")} if last_hook else None,
                "fades_after_last_hook_in_source": arrivals,
                "why": why,
            }
        )
    return rows


# --------------------------------------------------------------------- deltas

def json_diff(a, b, path="$"):
    if type(a) is not type(b):
        return [(path, a, b)]
    if isinstance(a, dict):
        out = []
        for k in list(a) + [k for k in b if k not in a]:
            if k not in a or k not in b:
                out.append((f"{path}.{k}", a.get(k), b.get(k)))
            else:
                out.extend(json_diff(a[k], b[k], f"{path}.{k}"))
        return out
    if isinstance(a, list):
        out = []
        for i in range(max(len(a), len(b))):
            if i >= len(a) or i >= len(b):
                out.append((f"{path}[{i}]", a[i] if i < len(a) else None, b[i] if i < len(b) else None))
            else:
                out.extend(json_diff(a[i], b[i], f"{path}[{i}]"))
        return out
    return [] if a == b else [(path, a, b)]


def rows_delta(old_rows, new_rows, key):
    out = []
    old = {key(r): r for r in old_rows}
    new = {key(r): r for r in new_rows}
    for k in [key(r) for r in new_rows] + [k for k in old if k not in new]:
        a, b = old.get(k), new.get(k)
        if a == b:
            continue
        entry = {"row": list(k) if isinstance(k, tuple) else k}
        if a is None or b is None:
            entry["added" if a is None else "removed"] = b or a
        else:
            entry["changed"] = {f: {"old": a.get(f), "new": b.get(f)} for f in list(a) + [f for f in b if f not in a] if a.get(f) != b.get(f)}
        out.append(entry)
    return out


TAIL_CALLS = {"music_breath", "finish", "hold", "wait_until(line.end())", "wait_until", "predict", "reading_window", "pause"}


def last_cue_per_file(audit):
    out = {}
    for row in audit["cues"]:
        if row["file"]:
            out[row["file"]] = row
    return out


def tails(audit):
    out = {}
    for name, row in last_cue_per_file(audit).items():
        calls = [{"line": c["line"], "call": c["call"], "source": c["source"]} for c in row["calls"] if c["call"] in TAIL_CALLS]
        out[name] = {"cue": row["cue"], "calls": calls}
    return out


def hooks_delta(old_audit, new_audit):
    old = {r["cue"]: r for r in old_audit["cues"]}
    out = {"changed": [], "moved": [], "unchanged": 0}
    for row in new_audit["cues"]:
        a = old.get(row["cue"])
        o = [(c["call"], c["source"]) for c in (a["calls"] if a else [])]
        n = [(c["call"], c["source"]) for c in row["calls"]]
        ol = [c["line"] for c in (a["calls"] if a else [])]
        nl = [c["line"] for c in row["calls"]]
        if a is None or o != n:
            removed = list((collections.Counter(o) - collections.Counter(n)).elements())
            added = list((collections.Counter(n) - collections.Counter(o)).elements())
            out["changed"].append(
                {
                    "cue": row["cue"],
                    "file": row["file"],
                    "say_line": {"old": a["line"] if a else None, "new": row["line"]},
                    "removed": [{"call": c, "source": s} for c, s in removed],
                    "added": [{"call": c, "source": s} for c, s in added],
                    "reordered_only": not removed and not added,
                    "voice_end_calls": {"old": a["voice_end_calls"] if a else None, "new": row["voice_end_calls"]},
                }
            )
        elif ol != nl or a["line"] != row["line"]:
            shifts = sorted({nl[i] - ol[i] for i in range(len(nl))} | {row["line"] - a["line"]})
            out["moved"].append({"cue": row["cue"], "file": row["file"], "say_line": {"old": a["line"], "new": row["line"]}, "line_shifts": shifts})
        else:
            out["unchanged"] += 1
    return out


def plan_delta(base_map, new_map, base_units, new_units, base_audit, new_audit, base_script, new_script):
    cues_changed = [
        {"cue": c, "old": base_script.get(c), "new": new_script.get(c)}
        for c in list(new_script) + [c for c in base_script if c not in new_script]
        if base_script.get(c) != new_script.get(c)
    ]
    old_units = {u["id"]: u for u in base_units["units"]}
    units_changed = [
        {"id": u["id"], "old_text": old_units.get(u["id"], {}).get("text"), "new_text": u["text"]}
        for u in new_units["units"]
        if old_units.get(u["id"]) != u
    ]
    t_old, t_new = tails(base_audit), tails(new_audit)
    finishes = []
    for name in sorted(set(t_old) | set(t_new)):
        a, b = t_old.get(name), t_new.get(name)
        if a == b:
            state = "unchanged"
        elif a and b and [(c["call"], c["source"]) for c in a["calls"]] == [(c["call"], c["source"]) for c in b["calls"]] and a["cue"] == b["cue"]:
            state = "moved"
        else:
            state = "changed"
        finishes.append({"file": name, "state": state, "old": a, "new": b})
    other = [
        {"path": p, "old": o, "new": n}
        for p, o, n in json_diff(
            {k: v for k, v in base_map.items() if k not in ("visual_holds", "helpers")},
            {k: v for k, v in new_map.items() if k not in ("visual_holds", "helpers")},
        )
    ]
    return {
        "script": {"cues_changed": cues_changed},
        "units": {
            "direction_contract": [{"path": p, "old": o, "new": n} for p, o, n in json_diff(base_units["direction_contract"], new_units["direction_contract"])],
            "script_sha256": {"old": base_units["script_sha256"], "new": new_units["script_sha256"]},
            "units_changed": units_changed,
            "unit_count": {"old": len(base_units["units"]), "new": len(new_units["units"])},
        },
        "map": {
            "visual_holds": rows_delta(base_map["visual_holds"], new_map["visual_holds"], lambda r: (r["cue"], r["kind"])),
            "helpers": rows_delta(base_map["helpers"], new_map["helpers"], lambda r: (r["path"], r["marker"].split("(")[0])),
            "other_fields": other,
        },
        "hooks_per_cue": hooks_delta(base_audit, new_audit),
        "chapter_closing_tails": finishes,
    }


def row_identity(section, row):
    if section == "visual_holds":
        return [section, row["cue"], row["kind"]]
    return [section, row["path"], row["marker"].split("(")[0]]


def allowed_successor_diffs(stage_map, new_map, stage_units, new_units, changed_paths, delta):
    """Successor plan differences from stage03's, each allowed or a problem.

    Only the declared ones pass: the script hash under F3, the file hash (not the line, not
    the marker) of exactly the declared rows, and in the units the script hashes, the map
    hash and 15_2's text under F3."""
    problems, rows = [], []
    declared = [r for r in DECLARED_SUCCESSOR["row_sha_moves"]]
    moved = []
    for p, o, n in json_diff(stage_map, new_map):
        ok = False
        if p == f"$.scripts.{TOPIC}" and delta == "F3":
            ok = True
        m = re.fullmatch(r"\$\.(visual_holds|helpers)\[(\d+)\]\.sha256", p)
        if m:
            row = stage_map[m.group(1)][int(m.group(2))]
            ident = row_identity(m.group(1), row)
            ok = ident in declared and row["path"] in changed_paths
            if ok:
                moved.append(ident)
        rows.append({"file": "map", "path": p, "old": o, "new": n, "allowed": ok})
        if not ok:
            problems.append(f"successor map differs from stage03's at {p} beyond the declared changes")
    for ident in declared:
        if ident not in moved:
            problems.append(f"successor map: declared row {ident} did not move its file hash")
    ids = [u["id"] for u in stage_units["units"]]
    for p, o, n in json_diff(stage_units, new_units):
        ok = p in ("$.direction_contract.map.sha256",) or (delta == "F3" and p in ("$.script_sha256", "$.direction_contract.script_sha256"))
        m = re.fullmatch(r"\$\.units\[(\d+)\]\.text", p)
        if m and delta == "F3" and ids[int(m.group(1))] == f"oliver_{F3['cue']}":
            ok = o.count(F3["old"]) == 1 and n == o.replace(F3["old"], F3["new"])
        rows.append({"file": "units", "path": p, "old": o, "new": n, "allowed": ok})
        if not ok:
            problems.append(f"successor units differ from stage03's at {p} beyond the declared changes")
    return rows, problems


# --------------------------------------------------------------------- one bind

def bind(repo, B, crate_rel, version, problems):
    crate = os.path.join(repo, crate_rel)
    plan, outputs, audit, _canonical_map, map_text = B.build(repo, crate, version)
    problems.extend(f"binder: {p}" for p in plan.problems)
    return outputs, audit, map_text


def run(args):
    repo = args.repo
    here = os.path.realpath(os.path.join(repo, HERE, "bind_complex_stage03.py"))
    if os.path.realpath(__file__) != here:
        print(f"refused: run the copy at {HERE}/bind_complex_stage03.py against its own repo", file=sys.stderr)
        return 2
    if args.print_pins:
        print(dumps(current_pins(repo)), end="")
        return 0
    stale = stale_pins(repo)
    if stale:
        for rel, want, got in stale:
            print(f"STALE: {rel} is {got or 'missing'}, pinned {want}", file=sys.stderr)
        print(f"{len(stale)} pinned inputs changed; nothing written", file=sys.stderr)
        return 3
    successor = args.stage == "stage04"
    stage_root = STAGE04 if successor else STAGE03
    receipt_rel = STAGE04_RECEIPT if successor else STAGE03_RECEIPT
    if args.out is None:
        args.out = f"{HERE}/bound-plan-{args.stage}"
    out = os.path.realpath(os.path.join(repo, args.out))
    for bad in (os.path.join(repo, "explainer_corpus"), os.path.join(repo, STAGE04), os.path.join(repo, STAGE03)):
        b = os.path.realpath(bad)
        if out == b or out.startswith(b + os.sep):
            print(f"refused: OUT {args.out} lies in {os.path.relpath(b, os.path.realpath(repo))}", file=sys.stderr)
            return 2
    p_real, here_real = os.path.realpath(os.path.join(repo, P)), os.path.realpath(os.path.join(repo, HERE))
    if out.startswith(p_real + os.sep) and not (out == here_real or out.startswith(here_real + os.sep)):
        print(f"refused: OUT {args.out} lies in another bundle under {P}", file=sys.stderr)
        return 2
    if args.map_version is None:
        args.map_version = "oliver12" if successor else "oliver12-stage03"
    if args.map_version in TAKEN_MAP_VERSIONS or not re.fullmatch(r"[a-z0-9][a-z0-9-]*", args.map_version):
        print(f"refused: map version {args.map_version} is already used for {TOPIC} or is not a plain name", file=sys.stderr)
        return 2

    problems, findings = [], []
    B = load_binder(repo)
    stage03_receipt = json.loads(read_bytes(os.path.join(repo, STAGE03_RECEIPT)))
    stage03_listed = check_receipt(repo, STAGE03, stage03_receipt, problems, "stage03 receipt")
    stage03_script = read_bytes(os.path.join(repo, STAGE03, CRATE_REL, "script.tsv"))
    if sha(stage03_script) != STAGE03_SCRIPT_SHA:
        problems.append("stage03 script.tsv is not 1e24aba8")
    receipt_info = {"path": receipt_rel, "sha256": sha(read_bytes(os.path.join(repo, receipt_rel)))}
    installed_plan = {f"{CRATE_REL}/narration_units.json", f"{CRATE_REL}/docs/narration-direction-map-{args.map_version}.json"}
    installed_seen = []
    if successor:
        receipt = json.loads(read_bytes(os.path.join(repo, receipt_rel)))
        listed = check_receipt(repo, stage_root, receipt, problems, "successor receipt")
        changed = receipt.get("changed")
        if not isinstance(changed, list):
            problems.append("successor receipt has no `changed` list")
            changed = []
        differing = sorted(p for p in set(listed) | set(stage03_listed) if listed.get(p) != stage03_listed.get(p))
        declared = DECLARED_SUCCESSOR["files"]
        strays_gone = []
        for p in differing:
            if p in F2_STRAYS and p not in listed:
                strays_gone.append(p)
                continue
            if p not in changed:
                problems.append(f"successor receipt: {p} differs from stage03 but is not named in `changed`")
            if p in declared:
                before, after = declared[p]
                if stage03_listed.get(p) != before or listed.get(p) != after:
                    problems.append(f"successor: {p} is {str(listed.get(p))[:12]}, declared {after[:12]} (from {before[:12]})")
            elif p in installed_plan:
                installed_seen.append(p)
            else:
                problems.append(f"successor: {p} differs from stage03 and is not in the declared successor ({DECLARED_SUCCESSOR['name']})")
        for p in declared:
            if p not in differing:
                problems.append(f"successor: declared change missing, {p} still has stage03's bytes")
        film_equal = {}
        for p in declared:
            if p.endswith(".rs") and p in listed and os.path.isfile(os.path.join(repo, stage_root, p)):
                old = film_code(read_bytes(os.path.join(repo, STAGE03, p)).decode("utf8"))
                new = film_code(read_bytes(os.path.join(repo, stage_root, p)).decode("utf8"))
                film_equal[p] = old is not None and old == new
                if not film_equal[p]:
                    problems.append(f"successor: {p}'s film code (everything before its one `#[cfg(test)] mod tests`) differs from stage03 or has no single test module; only its tests are declared")
        receipt_info.update(
            {
                "changed": changed,
                "differs_from_stage03": differing,
                "named_but_unchanged": sorted(set(changed) - set(differing)),
                "declared_successor": DECLARED_SUCCESSOR["name"],
                "film_code_equal_to_stage03": film_equal,
                "f2_strays_moved_aside": strays_gone,
            }
        )
    else:
        changed = stage03_receipt.get("changed", [])
        unnamed = unnamed_changes(repo, stage03_receipt.get("baseline", ""), stage03_listed, changed)
        known = [u for u in unnamed if u["path"] in KNOWN_UNNAMED_STAGE03]
        other = [u for u in unnamed if u["path"] not in KNOWN_UNNAMED_STAGE03]
        receipt_info.update({"baseline": stage03_receipt.get("baseline"), "changed": changed, "unnamed_changes": unnamed})
        for u in known:
            findings.append(f"receipt: {u['path']} differs from the receipt's baseline but is not in `changed` ({KNOWN_UNNAMED_STAGE03[u['path']]})")
        for u in other:
            problems.append(f"stage03 receipt: {u['path']} differs from its baseline but is not named in `changed`")
    crate_rel = os.path.join(stage_root, CRATE_REL).replace(os.sep, "/")
    crate = os.path.join(repo, crate_rel)
    script_bytes = read_bytes(os.path.join(crate, "script.tsv"))
    delta = script_delta(script_bytes, stage03_script)
    if delta is None:
        problems.append(
            f"script.tsv {sha(script_bytes)[:12]} is neither stage03's {STAGE03_SCRIPT_SHA[:12]} nor stage03's with the one "
            f"declared change ({F3['finding']}: 15_2 {F3['old']!r} -> {F3['new']!r}, {F3['script_sha256'][:12]}); refused"
        )
    if delta == "F3" and not successor:
        problems.append("the F3 script is only accepted from a successor stage whose receipt names it")
    if successor and delta == "F3" and f"{CRATE_REL}/script.tsv" not in changed:
        problems.append("F3 script without the receipt naming script.tsv")

    outputs, audit, map_text = bind(repo, B, crate_rel, args.map_version, problems)
    units_rel = f"{CRATE_REL}/narration_units.json"
    map_rel = f"{CRATE_REL}/docs/narration-direction-map-{args.map_version}.json"
    new_map, new_units = json.loads(map_text), json.loads(outputs[units_rel])
    script = B.read_script(os.path.join(crate, "script.tsv"))

    strict = Strict(B, crate, script)
    hook_table, phrases_checked = strict.check()
    beats = post_voice(B, strict, script)
    base_script = B.read_script(os.path.join(repo, BASE, "script.tsv"))
    reviewed = {}
    for cue, text in script.items():
        if base_script.get(cue) == text:
            continue
        want = REVIEWED_HOOKS.get(cue)
        rows = hook_table.get(cue, [])
        got = [(r["call"], r["value"], r["sentence"], r["sentence_text"]) for r in rows]
        if want is None:
            strict.problems.append(f"S6 cue {cue}'s words changed since oliver10 and it has no reviewed hook table")
            continue
        ok = len(got) == len(want) and all(
            g[0] == w[0] and g[1] == w[1] and g[2] == w[2] and g[3].startswith(w[3]) for g, w in zip(got, want)
        )
        if not ok:
            strict.problems.append(f"S6 cue {cue}'s hooks no longer match its reviewed table: {[(g[0], g[1], g[2], g[3][:40]) for g in got]}")
        reviewed[cue] = {"reviewed": [list(w) for w in want], "matches": ok}
    for cue in sorted(REVIEWED_HOOKS):
        if cue not in reviewed:
            rows = hook_table.get(cue, [])
            got = [(r["call"], r["value"], r["sentence"], r["sentence_text"]) for r in rows]
            want = REVIEWED_HOOKS[cue]
            ok = len(got) == len(want) and all(
                g[0] == w[0] and g[1] == w[1] and g[2] == w[2] and g[3].startswith(w[3]) for g, w in zip(got, want)
            )
            if not ok:
                strict.problems.append(f"S6 cue {cue}'s hooks no longer match its reviewed table")
            reviewed[cue] = {"reviewed": [list(w) for w in want], "matches": ok, "words_changed_since_oliver10": False}
    problems.extend(strict.problems)

    base_map = json.loads(read_bytes(os.path.join(repo, BASE, "narration-direction-map-oliver10.json")))
    base_units = json.loads(read_bytes(os.path.join(repo, BASE, "narration_units.json")))
    base_audit = json.loads(read_bytes(os.path.join(repo, BASE, "source-audit.json")))

    # Cross-checks against two proposals made without this tool. For stage04 the proposals
    # are carried forward by the declared delta only: the F3 script hash, and the six rows'
    # file hashes from stage04's receipt.
    lav = json.loads(read_bytes(os.path.join(repo, LAVENDER_DELTA)))
    t02 = json.loads(read_bytes(os.path.join(repo, TEACHING02_MAP)))
    applied, bad = json.loads(json.dumps(t02)), []
    for r in lav["rows"]:
        row = applied[r["section"]][r["index"]]
        if any(row.get(f) != v for f, v in r["old"].items()):
            bad.append(f"{r['section']}[{r['index']}]")
        row.update(r["new"])
    carried = []
    if successor:
        applied["scripts"][TOPIC] = F3["script_sha256"]
        for section in ("visual_holds", "helpers"):
            for row in applied[section]:
                if row_identity(section, row) in DECLARED_SUCCESSOR["row_sha_moves"]:
                    row["sha256"] = listed.get(row["path"])
                    carried.append(row_identity(section, row))
    same_map = B.dumps(applied).encode() == map_text.encode()
    if bad or not same_map:
        problems.append(f"cross-check: the map is not Lavender's delta applied to teaching02's oliver11{' carried to stage04' if successor else ''} (bad old rows {bad})")
    pointed = json.loads(outputs[units_rel])
    pointed["direction_contract"]["map"] = {"path": f"{CRATE_REL}/docs/narration-direction-map-oliver11.json", "sha256": TEACHING02_MAP_SHA}
    if successor:
        pointed["script_sha256"] = STAGE03_SCRIPT_SHA
        pointed["direction_contract"]["script_sha256"] = STAGE03_SCRIPT_SHA
        for u in pointed["units"]:
            if u["id"] == f"oliver_{F3['cue']}":
                u["text"] = u["text"].replace(F3["new"], F3["old"])
    same_units = sha(B.dumps(pointed).encode()) == TEACHING02_UNITS_SHA
    if not same_units:
        problems.append("cross-check: the units, pointed at teaching02's oliver11 (and with F3 undone for stage04), are not teaching02's patched plan 3989e185")
    cross = {
        "map_equals_lavender_delta_on_teaching02_oliver11": same_map and not bad,
        "lavender_rows": len(lav["rows"]),
        "carried_to_stage04": {"script": F3["script_sha256"], "row_sha_from_stage04_receipt": carried} if successor else None,
        "units_equal_teaching02_patched_plan_but_for_the_map": same_units,
        "note": "Checks of the binding against two independent proposals. Nothing is edited from them.",
    }
    successor_rows = None
    if successor:
        stage_problems = []
        stage_out, _stage_audit, stage_map_text = bind(repo, B, f"{STAGE03}/{CRATE_REL}", args.map_version, stage_problems)
        successor_rows, extra = allowed_successor_diffs(
            json.loads(stage_map_text), new_map, json.loads(stage_out[units_rel]), new_units, set(changed), delta
        )
        problems.extend(stage_problems)
        problems.extend(extra)
        for p in installed_seen:
            want = outputs[units_rel] if p.endswith("narration_units.json") else map_text
            if read_bytes(os.path.join(repo, stage_root, p)) != want.encode():
                problems.append(f"successor: {p} is installed but is not this binding")
    for crate_dir in (os.path.join(repo, CRATE_REL), crate, os.path.join(repo, STAGE03, CRATE_REL)):
        path = os.path.join(crate_dir, "docs", f"narration-direction-map-{args.map_version}.json")
        if os.path.isfile(path) and read_bytes(path) != map_text.encode():
            problems.append(f"map version {args.map_version} already exists with other bytes at {os.path.relpath(path, repo)}; pick the next")

    live = canonical_live(repo, B, base_audit)
    hygiene = {
        "orig_files": [p for p in tree_listing(crate) if p.endswith(".orig")],
        "hook_state": [p for p in tree_listing(crate) if p.startswith(".ee/")],
        "unregistered_chapter_files_ignored": [r["path"] for r in audit["unregistered_files_ignored"]],
        "note": "The binder reads neither the .orig files nor .ee/. Move them aside before promotion (Lavender F2).",
    }
    if hygiene["orig_files"] or hygiene["hook_state"]:
        findings.append(f"hygiene: {len(hygiene['orig_files'])} .orig files and {len(hygiene['hook_state'])} .ee/ files in the crate (Lavender F2)")

    dyn = []
    for (name, source), entry in sorted(B.DYNAMIC.items()):
        d = strict.derived.get((name, source), {})
        dyn.append(
            {
                "file": name,
                "call": source,
                "table": entry.get("phrases") or entry.get("sentences") or entry.get("by_owner"),
                "from_source": d.get("values"),
                "how": d.get("how"),
                "equal": (d.get("values") == (entry.get("phrases") or entry.get("sentences") or entry.get("by_owner"))),
            }
        )
    map_sha = sha(map_text.encode())
    strict_audit = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": CLAIM,
        "mode": args.stage,
        "crate": crate_rel,
        "receipt": receipt_info,
        "script": {"sha256": sha(script_bytes), "delta": delta, "declared_change": F3},
        "pins": {rel: PINS[rel] for rel in PINS},
        "binder": {"path": BINDER, "sha256": PINS[BINDER], "original": RUBY_ORIGINAL, "author": "RubyRaven (complex02-05-cross-review04), on BronzeJaguar's earlier binder"},
        "map": {"version": args.map_version, "path": map_rel, "sha256": map_sha},
        "units": {"path": units_rel, "sha256": sha(outputs[units_rel].encode())},
        "findings": findings,
        "strict": {
            "S1_phrases_checked": phrases_checked,
            "S2_cues_with_hooks": sum(1 for rows in hook_table.values() if rows),
            "S2_hooks": sum(len(rows) for rows in hook_table.values()),
            "S3_dynamic": dyn,
            "S4_post_voice_beats": beats,
            "S6_reviewed_cues": reviewed,
            "problems": strict.problems,
        },
        "cross_checks": cross,
        "successor_plan_differences": successor_rows,
        "promotion_hygiene": hygiene,
        "hooks": hook_table,
        "limits": [
            "Source text only: not compiled, not run by Cargo or Clippy, not rendered, not synthesised, not heard.",
            "Hook positions are characters in the cue text. Times are estimates until Oliver word marks exist.",
            "Reading and prediction seconds are authored pauses in source, not measurements.",
            "Not independent: BronzeJaguar wrote parts of these scenes and earlier repairs.",
        ],
    }
    delta_doc = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": CLAIM,
        "base": {"map": f"{BASE}/narration-direction-map-oliver10.json", "map_sha256": PINS[f"{BASE}/narration-direction-map-oliver10.json"], "note": "canonical's oliver10, rebuilt byte for byte by the pinned binder from canonical at seal time"},
        "new": {"crate": crate_rel, "map_version": args.map_version, "map_sha256": map_sha, "script_delta": delta},
        **plan_delta(base_map, new_map, base_units, new_units, base_audit, audit, base_script, script),
        "dynamic_values": dyn,
        "post_voice_beats": beats,
    }
    files = dict(outputs)
    files["strict-audit.json"] = dumps(strict_audit)
    files["delta-from-oliver10.json"] = dumps(delta_doc)
    if successor:
        files["delta-from-stage03.json"] = dumps(
            {"schema_version": 1, "topic": TOPIC, "claim": CLAIM, "receipt": receipt_info, "script_delta": delta, "differences": successor_rows}
        )
    if problems:
        for text in problems:
            print("REFUSED: " + text, file=sys.stderr)
        print(f"{len(problems)} problems; nothing written", file=sys.stderr)
        return 1
    if args.check:
        stale_out = [rel for rel, text in files.items() if not os.path.isfile(os.path.join(out, rel)) or read_bytes(os.path.join(out, rel)) != text.encode()]
        for rel in stale_out:
            print("STALE: " + rel, file=sys.stderr)
        if stale_out:
            return 1
    else:
        for rel, text in files.items():
            path = os.path.join(out, rel)
            if os.path.exists(path) and read_bytes(path) != text.encode():
                print(f"REFUSED: {rel} exists in OUT with other bytes; bind into a new OUT", file=sys.stderr)
                return 1
        for rel, text in files.items():
            path = os.path.join(out, rel)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            if not os.path.exists(path):
                with open(path, "x", encoding="utf8", newline="") as handle:
                    handle.write(text)
    c = audit["counts"]
    print(
        f"{'in step' if args.check else 'written'} ({strict_audit['mode']}, script delta {delta}): "
        f"{c['units']} units, {c['cues']} cues, {c['chapters']} chapters; {c['prediction_holds']} prediction and "
        f"{c['reading_holds']} reading holds; {strict_audit['strict']['S2_hooks']} hooks on "
        f"{strict_audit['strict']['S2_cues_with_hooks']} cues, {phrases_checked} phrases unique; "
        f"{len(dyn)} dynamic values re-derived; {len(beats)} post-voice beats read"
    )
    print(f"map {args.map_version} {map_sha}")
    print(f"units {strict_audit['units']['sha256']}")
    if live["equals_baseline"]:
        print("canonical: live canonical still rebuilds to the sealed oliver10 baseline byte for byte")
    else:
        print(f"canonical: moved since the seal ({len(live['moved'])} files); the sealed oliver10 baseline was used")
    for text in findings:
        print("FINDING: " + text)
    return 0


def canonical_live(repo, B, base_audit):
    """Whether live canonical still has the sealed baseline's bytes; if so, rebuild and compare."""
    moved = []
    for row in base_audit["sources"] + base_audit["kit_sources"]:
        path = os.path.join(repo, row["path"])
        if not os.path.isfile(path) or sha(read_bytes(path)) != row["sha256"]:
            moved.append(row["path"])
    if moved:
        return {"equals_baseline": False, "moved": moved, "note": "canonical has moved since the seal; the sealed baseline is used"}
    plan, outputs, audit, _m, map_text = B.build(repo, os.path.join(repo, CRATE_REL), "oliver10")
    same = (
        not plan.problems
        and map_text.encode() == read_bytes(os.path.join(repo, BASE, "narration-direction-map-oliver10.json"))
        and outputs[f"{CRATE_REL}/narration_units.json"].encode() == read_bytes(os.path.join(repo, BASE, "narration_units.json"))
        and outputs["source-audit.json"].encode() == read_bytes(os.path.join(repo, BASE, "source-audit.json"))
    )
    return {"equals_baseline": same, "moved": [], "note": "live canonical rebuilt at oliver10 by the pinned binder and compared byte for byte"}


def main():
    parser = argparse.ArgumentParser(description="Bind complex_analysis_visual's combined-repair-stage03 strictly.")
    parser.add_argument("--repo", default=".")
    parser.add_argument("--stage", choices=["stage04", "stage03"], default="stage04")
    parser.add_argument("--out", help="default: this bundle's bound-plan-<stage>")
    parser.add_argument("--map-version", help="default: oliver12 for stage04, oliver12-stage03 for stage03")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--print-pins", action="store_true")
    args = parser.parse_args()
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
