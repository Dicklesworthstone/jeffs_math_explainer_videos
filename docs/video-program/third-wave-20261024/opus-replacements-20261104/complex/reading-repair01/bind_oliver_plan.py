#!/usr/bin/env python3
"""Bind the ElevenLabs Oliver source plan of complex_analysis_visual to its source.

    python3 -I -B bind_oliver_plan.py REPO CRATE OUT [--map-version oliver08] [--check]

REPO is the repository root (read for the kit's helpers). CRATE is the crate to
bind, read only. OUT receives, at paths that mirror the canonical tree:

    explainer_corpus/complex_analysis_visual/narration_units.json
    explainer_corpus/complex_analysis_visual/docs/narration-direction-map-<version>.json
    source-audit.json

Files already in OUT with other bytes are refused, never overwritten: after a
compiler fix, rebind into a new OUT. --check writes nothing and exits 1 unless
OUT already holds exactly what this crate would produce.

The binder reads the Rust source as text; it does not compile it. It refuses,
listing every problem, rather than pass anything it has not resolved: a say whose
cue is not a literal, a say in a loop or a helper, a wait argument that is neither
a literal nor in DYNAMIC, a wait after the voice that nothing classifies, a
prediction that is not a question or that has no next cue in its chapter, a reading
window that the table, the calls and the storyboard do not all agree on, or one that
could let its card go early.

Only chapter files registered in mod.rs are read: an old file left on disk is not in
the film, and the audit names each one it ignored.

Coverage only. This establishes no take, duration, listening or acceptance. A reading
row records that the source adds a wait; the seconds are a proposed teaching pause,
not a measurement.
"""

import argparse
import bisect
import collections
import glob
import hashlib
import json
import os
import re
import sys

TOPIC = "complex_analysis_visual"
TOPIC_REL = f"explainer_corpus/{TOPIC}"
MAX_WORDS = 100
NOT_FILM = {"thumbnail.rs"}
NOT_CHAPTERS = {"draw.rs", "mod.rs", "thumbnail.rs"}

# The wait arguments that are not literals, each resolved by reading the source.
# The key is the exact call text and "from" must still be in the file, so a new or
# changed one is refused instead of passing unread.
DYNAMIC = {
    ("the_bump.rs", "line.wait_for(stage, phrase)?;"): {
        "from": 'for (deg, phrase) in [(4.0, "plus x to the fourth"), (6.0, "minus x to the sixth")] {',
        "phrases": ["plus x to the fourth", "minus x to the sixth"],
    },
    ("gallery.rs", "line.wait_for(stage, phrase)?;"): {
        "from": 'let named = ["a pole at zero", "at minus one", "at minus two"];',
        "phrases": ["a pole at zero", "at minus one", "at minus two"],
    },
    ("gallery.rs", "Some(p) => line.until(stage, p, 2.5)?,"): {
        "from": 'Some("Three zeros inside"),',
        "phrases": ["Three zeros inside"],
        "owners": ["05_1"],
        "note": "walk_and_count's phrase: Some at the 05_1 call; the 05_2 call passes None and walks to line.end().",
    },
    ("counting.rs", "line.wait_sentence(stage, i + 2)?;"): {
        "from": "for (i, (f, name)) in [Func::Identity, Func::Square, Func::Recip]",
        "sentences": [2, 3, 4],
    },
}

# What draw.rs's timing helpers may call. Anything else there is unread.
DRAW_CALLS = {
    "say": {"say"},
    "at": set(),
    "sentence": set(),
    "wait_for": {"at", "wait_until"},
    "wait_sentence": {"sentence", "wait_until"},
    "wait_last": {"wait_sentence"},
    "until": {"at"},
    "wait_until": {"wait_frames"},
    "pause": {"wait_frames"},
    "predict": {"wait_last", "hold", "mark", "listen", "wait_until"},
    "reading_window": {"wait_until", "hold", "pause"},
}

# The kit's definitions the plan's statements about holds rest on.
KIT_MARKERS = {
    "crates/explainer_kit/src/kit.rs": [
        "pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {",
        "pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {",
        "pub fn finish(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {",
        "pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {",
        "pub fn listen(&self, stage: &mut Stage<'_>, seconds: f64, label: &str) -> fmn::Result<()> {",
    ],
    "crates/explainer_kit/src/narration.rs": [
        "fn gap_before(&self, next: &str) -> f64 {",
        "pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {",
        "pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {",
        "pub fn music_breath(&self, stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {",
        "pub fn listen(&self, stage: &mut Stage<'_>, seconds: f64, label: &str) -> fmn::Result<()> {",
        "fn window(&self, stage: &mut Stage<'_>, seconds: f64, label: Option<&str>) -> fmn::Result<()> {",
    ],
}

# Bounds on an authored reading window. Under 4 s in all, breath included, the gap stays a
# hold for the score (HOLD_MERGE) and is not dead air for the cue sheets (--max-gap).
READING_MIN_S = 1.0
GAP_LIMIT_S = 4.0

HOLD_MEANINGS = {
    "predict": "Native prediction window after the cue: no narration; the picture waits. Not silence in the take.",
    "reading_window": "Native reading window after the cue: the voice ends, then the kit's breath, then the authored seconds with the card up and nothing drawn or cleared. Not silence in the take; a proposed pause, not a measured reading time.",
    "kit.music_breath": "Native music window after the cue: no narration; the score plays alone. Not silence in the take.",
    "kit.hold": "Native wait for the voice to end plus the kit's breath. Not authored for reading or thinking.",
    "kit.finish": "Calls kit.hold, then closes the chapter.",
    "wait_until(line.end())": "The picture waits for the voice to end. No time is added after it.",
    "wait_for / wait_sentence / until / sentence": "Lands a picture on words while the voice runs. Word moments are estimates until takes are aligned.",
    "pause": "A picture beat placed before the cue's voice-end wait in source order. Whether the voice is still running then needs measured takes.",
}

CALL = re.compile(
    r"(?<![\w.:])(?:(?P<recv>[a-z_]\w*)\.)?"
    r"(?P<name>say|predict|reading_window|hold|music_breath|listen|sonify|until_sentence|pause|wait_until|wait_frames|wait"
    r"|wait_for|wait_sentence|wait_last|until|at|sentence|precisely_now|names_now|finish|mark)\s*\("
)
STRICT = {"say", "predict", "reading_window", "hold", "music_breath", "listen", "sonify", "until_sentence", "wait_frames"}
LINE_METHODS = {"wait_for", "wait_sentence", "wait_last", "until", "at", "sentence"}
KIT_METHODS = {"hold", "music_breath", "listen", "sonify", "until_sentence", "finish", "mark"}
FREE_FUNCTIONS = {"pause", "wait_until", "wait_frames", "predict", "reading_window", "precisely_now", "names_now"}
TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:pub\s+)?mod\s+\w+\s*\{")
SAY_STATEMENT = re.compile(r'let line = say\(&kit, stage, "(\d\d_\d+)"\)\?;')
PREDICT_ROW = re.compile(r'^\s*\("(\d\d_\d+)",\s*(\d+(?:\.\d+)?)\),\s*$')
FN_HEAD = re.compile(r"\bfn\s+(\w+)")
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')


class Plan:
    def __init__(self):
        self.problems = []

    def problem(self, text):
        self.problems.append(text)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def text_of(path):
    with open(path, encoding="utf8", newline="") as handle:
        return handle.read()


def bytes_of(path):
    with open(path, "rb") as handle:
        return handle.read()


def read_script(path):
    """The rules of tools/coherent_narration.py read_script."""
    result = {}
    for number, row in enumerate(text_of(path).splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        if len(cells) != 2 or not re.fullmatch(r"\d\d_\d+", cells[0]) or not cells[1].strip():
            raise SystemExit(f"invalid script row {number}")
        if cells[0] in result:
            raise SystemExit(f"duplicate script cue {cells[0]}")
        result[cells[0]] = cells[1]
    return result


def sentence_offsets(text):
    """draw.rs sentence_offsets: what Spoken counts its sentences with."""
    chars = list(text)
    out = [0]
    for i in range(max(0, len(chars) - 2)):
        if chars[i] in ".?!" and chars[i + 1] == " ":
            out.append(i + 2)
    return out


def report_sentences(text):
    """tools/coherent_narration.py sentences: the rows a narration report has."""
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


def blank(src):
    """Comments and string contents to spaces, keeping every offset."""
    out = list(src)
    i, n = 0, len(src)

    def fill(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = src[i]
        if src[i : i + 2] == "//":
            j = src.find("\n", i)
            j = n if j < 0 else j
            fill(i, j)
            i = j
        elif src[i : i + 2] == "/*":
            j = src.find("*/", i) + 2
            fill(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            fill(i + 1, j)
            i = j + 1
        else:
            raw = re.match(r'r(#*)"', src[i : i + 8])
            char = re.match(r"'(\\.|[^\\'])'", src[i : i + 5])
            if raw and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")):
                end = '"' + raw.group(1)
                j = src.find(end, i + raw.end())
                fill(i + raw.end(), j)
                i = j + len(end)
            elif char:
                fill(i + 1, i + char.end() - 1)
                i += char.end()
            else:
                i += 1
    return "".join(out)


def close_of(t, at):
    opener, closer = t[at], {"(": ")", "[": "]", "{": "}"}[t[at]]
    depth = 0
    for i in range(at, len(t)):
        if t[i] == opener:
            depth += 1
        elif t[i] == closer:
            depth -= 1
            if depth == 0:
                return i
    raise SystemExit("unbalanced source")


def without_tests(t):
    """Blank each `#[cfg(test)] mod`; a `#[cfg(test)] use` is not the end of a file."""
    out = list(t)
    for m in TEST_MOD.finditer(t):
        for k in range(m.start(), close_of(t, m.end() - 1) + 1):
            if out[k] != "\n":
                out[k] = " "
    return "".join(out)


def split_args(raw, t, open_at):
    end = close_of(t, open_at)
    parts, depth, start = [], 0, open_at + 1
    for i in range(open_at + 1, end):
        if t[i] in "([{":
            depth += 1
        elif t[i] in ")]}":
            depth -= 1
        elif t[i] == "," and depth == 0:
            parts.append(raw[start:i].strip())
            start = i + 1
    if raw[start:end].strip():
        parts.append(raw[start:end].strip())
    return parts


def recognized(recv, name):
    if name == "say":
        return recv in (None, "kit")
    if name in KIT_METHODS:
        return recv == "kit"
    if name == "wait":
        return recv == "stage"
    if name in LINE_METHODS:
        return recv in ("line", "self")
    return name in FREE_FUNCTIONS and recv is None


class Source:
    """One Rust file: its timing calls with enclosing function and control flow."""

    def __init__(self, name, raw, plan):
        self.name, self.raw = name, raw
        self.t = without_tests(blank(raw))
        self.lines = raw.split("\n")
        self.calls = []
        self.fns = {}
        stack = []
        found = [(m.start(), "call", m) for m in CALL.finditer(self.t)]
        found += [(m.start(), "brace", m.group()) for m in re.finditer(r"[{}]", self.t)]
        for at, kind, item in sorted(found, key=lambda row: row[0]):
            if kind == "brace":
                if item == "{":
                    stack.append(self._head(at))
                else:
                    stack.pop()
                continue
            recv, called = item.group("recv"), item.group("name")
            if self.t[max(0, at - 3) : at] == "fn ":
                continue
            if not recognized(recv, called):
                if called in STRICT:
                    plan.problem(f"{name}:{self.line_at(at)} {called} on an unread receiver {recv!r}")
                continue
            fn = next((head[1] for head in reversed(stack) if head[0] == "fn"), None)
            self.calls.append(
                {
                    "file": name,
                    "at": at,
                    "line": self.line_at(at),
                    "name": called,
                    "recv": recv,
                    "fn": fn,
                    "control": [head[1] for head in stack if head[0] == "control"],
                    "source": self.lines[self.line_at(at) - 1].strip(),
                    "args": split_args(raw, self.t, item.end() - 1),
                }
            )
        for m in FN_HEAD.finditer(self.t):
            paren = self.t.find("(", m.end())
            close = close_of(self.t, paren)
            brace, semi = self.t.find("{", close), self.t.find(";", close)
            if brace >= 0 and (semi < 0 or brace < semi):
                self.fns.setdefault(m.group(1), []).append((brace, close_of(self.t, brace)))
        sweep = re.compile(r"\b(" + "|".join(sorted(STRICT)) + r")\s*\(")
        seen = {c["at"] + (len(c["recv"]) + 1 if c["recv"] else 0) for c in self.calls}
        for m in sweep.finditer(self.t):
            if m.start() not in seen and self.t[max(0, m.start() - 3) : m.start()] != "fn ":
                plan.problem(f"{name}:{self.line_at(m.start())} {m.group(1)} call the scan could not read")

    def line_at(self, at):
        return self.t.count("\n", 0, at) + 1

    def _head(self, at):
        k = at - 1
        while k >= 0 and self.t[k] not in ";{}":
            k -= 1
        head = re.sub(r"\s+", " ", self.t[k + 1 : at]).strip()
        fn = FN_HEAD.search(head)
        if fn:
            return ("fn", fn.group(1))
        if head.startswith(("impl", "mod", "trait", "pub mod", "struct", "pub struct", "enum")):
            return ("item", head)
        word = re.search(r"\b(for|while|loop|if|match|else)\b", head)
        if word:
            return ("control", word.group(1))
        if re.search(r"\|[^|]*\|\s*$", head):
            return ("control", "closure")
        return ("other", head)


def literal(arg):
    m = STRING.fullmatch(arg)
    return m.group(1) if m else None


def check_line_call(call, owners, script, plan):
    """A phrase Spoken::at cannot find, or a sentence it lacks, fails the render."""
    name, args, where = call["name"], call["args"], f"{call['file']}:{call['line']}"
    arg = {"wait_for": 1, "until": 1, "at": 0, "wait_sentence": 1, "sentence": 0}.get(name)
    if arg is None:
        return None
    given = args[arg] if len(args) > arg else ""
    phrases, sentences, resolved = [], [], None
    if name in ("wait_for", "until", "at") and literal(given) is not None:
        phrases = [literal(given)]
    elif name in ("wait_sentence", "sentence") and re.fullmatch(r"\d+", given):
        sentences = [int(given)]
    else:
        entry = DYNAMIC.get((call["file"], call["source"]))
        if entry is None:
            plan.problem(f"{where} non-literal {name} argument nobody has resolved: {call['source']}")
            return None
        phrases, sentences = entry.get("phrases", []), entry.get("sentences", [])
        owners = entry.get("owners", owners)
        resolved = {"from": entry["from"], "values": phrases or sentences}
        if "note" in entry:
            resolved["note"] = entry["note"]
    for owner in owners:
        for phrase in phrases:
            if phrase not in script[owner]:
                plan.problem(f"{where} phrase {phrase!r} is not in cue {owner}")
        for j in sentences:
            if j >= len(sentence_offsets(script[owner])):
                plan.problem(f"{where} sentence {j} does not exist in cue {owner}")
    return resolved


def analyse_chapter(src, script, plan):
    """Who owns each call in one scene file, and where its voice-end wait falls."""
    construct = [c for c in src.calls if c["fn"] == "construct"]
    says = [c for c in construct if c["name"] == "say"]
    rows, positions = [], []
    for call in [c for c in src.calls if c["name"] == "say"]:
        where = f"{src.name}:{call['line']}"
        m = SAY_STATEMENT.fullmatch(call["source"])
        if not m or call["recv"] is not None:
            plan.problem(f"{where} say is not `let line = say(&kit, stage, \"NN_M\")?;`: {call['source']}")
            continue
        if call["fn"] != "construct" or call["control"]:
            plan.problem(f"{where} say of {m.group(1)} sits in {call['fn']} {call['control']}, so it may not run once")
            continue
        rows.append(
            {"cue": m.group(1), "file": src.name, "line": call["line"], "marker": call["source"], "at": call["at"], "calls": []}
        )
        positions.append(call["at"])
    span = src.fns.get("construct", [(0, 0)])[0]
    bindings = len(re.findall(r"\blet\s+(?:mut\s+)?line\b", src.t[span[0] : span[1]]))
    if bindings != len(says):
        plan.problem(f"{src.name}: {bindings} `line` bindings in construct against {len(says)} says")

    def owner(at):
        i = bisect.bisect_right(positions, at) - 1
        return rows[i] if i >= 0 else None

    timed = []  # (position in construct, kind, row, seconds text, call)
    for call in construct:
        if call["name"] == "say":
            continue
        row = owner(call["at"])
        if row is None:
            if call["name"] not in ("finish", "mark"):
                plan.problem(f"{src.name}:{call['line']} {call['name']} runs before any cue")
            continue
        if call["recv"] == "line":
            call["resolved"] = check_line_call(call, [row["cue"]], script, plan)
        elif call["name"] == "predict" and (len(call["args"]) < 3 or call["args"][2] != "&line"):
            plan.problem(f"{src.name}:{call['line']} predict is not given the current line: {call['source']}")
        elif call["name"] == "reading_window" and call["args"] != ["&kit", "stage", "&line"]:
            plan.problem(f"{src.name}:{call['line']} reading_window is not given the current line: {call['source']}")
        timed.append((call["at"], call, row))

    helpers = {}
    for call in src.calls:
        if call["fn"] not in (None, "construct") and call["name"] != "say":
            helpers.setdefault(call["fn"], []).append(call)
    for fn, calls in sorted(helpers.items()):
        sites = []
        for m in re.finditer(r"(?<![\w.])(?:Self::)?" + re.escape(fn) + r"\s*\(", src.t):
            if span[0] <= m.start() <= span[1]:
                sites.append(m)
        if not sites:
            plan.problem(f"{src.name}: helper {fn} waits but construct never calls it")
            continue
        owners = []
        for m in sites:
            row = owner(m.start())
            if row is None:
                plan.problem(f"{src.name}:{src.line_at(m.start())} helper {fn} runs before any cue")
                continue
            owners.append(row)
            args = split_args(src.raw, src.t, m.end() - 1)
            timed.append(
                (
                    m.start(),
                    {
                        "file": src.name,
                        "at": m.start(),
                        "line": src.line_at(m.start()),
                        "name": "helper:" + fn,
                        "recv": None,
                        "fn": "construct",
                        "control": [],
                        "source": src.lines[src.line_at(m.start()) - 1].strip(),
                        "args": args,
                        "inner": [{"line": c["line"], "call": c["name"], "source": c["source"]} for c in calls],
                    },
                    row,
                )
            )
        for call in calls:
            if call["recv"] == "line":
                call["resolved"] = check_line_call(call, [r["cue"] for r in owners], script, plan)
                if call["resolved"] is not None:
                    for row in owners:
                        row.setdefault("helper_dynamic", []).append(
                            {"line": call["line"], "source": call["source"], **call["resolved"]}
                        )
    timed.sort(key=lambda row: row[0])

    for row in rows:
        mine = [(call, at) for at, call, r in timed if r is row]
        voice_end = None
        ends = []
        for call, at in mine:
            call_name = call["name"]
            is_end = call_name in ("hold", "music_breath", "predict", "reading_window", "finish") or (
                call_name == "wait_until" and call["args"] == ["stage", "line.end()"]
            )
            if call_name == "wait_until" and call["args"] != ["stage", "line.end()"]:
                plan.problem(f"{src.name}:{call['line']} wait_until other than line.end(): {call['source']}")
            if is_end:
                ends.append("wait_until(line.end())" if call_name == "wait_until" else call_name)
                if voice_end is None:
                    voice_end = at
        row["voice_end_calls"] = ends
        if voice_end is None:
            plan.problem(f"{src.name}: cue {row['cue']} has no voice-end wait before the next cue")
        for call, at in mine:
            waits = call["name"] in ("pause", "wait", "wait_frames") or (
                call["name"].startswith("helper:") and any(i["call"] == "pause" for i in call["inner"])
            )
            if waits and voice_end is not None and at > voice_end:
                plan.problem(
                    f"{src.name}:{call['line']} {call['name']} after the voice-end wait of {row['cue']}: "
                    "an unclassified post-voice hold; read it and classify it as reading or not"
                )
            entry = {"line": call["line"], "call": call["name"], "source": call["source"]}
            if call.get("resolved"):
                entry["resolved"] = call["resolved"]
            if "inner" in call:
                entry["inner"] = call["inner"]
            row["calls"].append(entry)
    for row in rows:
        row["reading_tasks"] = []
        last_sync = None
        for call, _ in [(c, a) for a, c, r in timed if r is row]:
            if call["name"] in ("wait_for", "wait_sentence", "until", "sentence"):
                last_sync = call
            if call["name"] in ("precisely_now", "names_now"):
                row["reading_tasks"].append(reading_task(src, call, last_sync, script[row["cue"]]))
    for m in re.finditer(r"\bCodePanel::build\s*\(", src.t):
        row = owner(m.start())
        if row is not None:
            row["reading_tasks"].append(
                {
                    "line": src.line_at(m.start()),
                    "call": "CodePanel::build",
                    "marker": src.lines[src.line_at(m.start()) - 1].strip(),
                    "text_chars": None,
                    "last_sync_before": None,
                    "last_sync_sentence": 0,
                    "cue_sentences": len(sentence_offsets(script[row["cue"]])),
                    "cue_words_left_at_most": len(script[row["cue"]].split()),
                }
            )
    return rows, timed


def synced_sentence(call, text):
    """The sentence (from 0) a sync call lands in, when the source says so plainly."""
    args = call["args"]
    if call["name"] in ("wait_sentence", "sentence"):
        given = args[1] if call["name"] == "wait_sentence" else args[0]
        return int(given) if re.fullmatch(r"\d+", given) else None
    phrase = literal(args[1]) if len(args) > 1 else None
    if phrase is None or phrase not in text:
        return None
    return bisect.bisect_right(sentence_offsets(text), text.find(phrase)) - 1


def reading_task(src, call, last_sync, text):
    """A card with words to read: how much of its cue can remain after it arrives.

    Text only. The card arrives at or after its last sync, so this is an upper bound
    on the words left, not a duration.
    """
    arg = call["args"][2]
    strings = STRING.findall(arg)
    if not strings:
        const = re.search(r"const " + re.escape(arg) + r': &str =\s*"((?:[^"\\]|\\.)*)"', src.raw)
        strings = [const.group(1)] if const else []
    card = " · ".join(strings)
    starts = sentence_offsets(text)
    sentence = synced_sentence(last_sync, text) if last_sync else 0
    left = len(text[starts[sentence] :].split()) if sentence is not None and sentence < len(starts) else None
    return {
        "line": call["line"],
        "call": call["name"],
        "marker": call["source"],
        "text_chars": len(card) if card else None,
        "last_sync_before": last_sync["source"] if last_sync else None,
        "last_sync_sentence": sentence,
        "cue_sentences": len(starts),
        "cue_words_left_at_most": left,
    }


def check_draw(src, plan):
    """draw::predict's order is what every prediction hold's basis states."""
    for call in src.calls:
        allowed = DRAW_CALLS.get(call["fn"])
        if allowed is None:
            if call["fn"] not in ("precisely_now", "names_now"):
                plan.problem(f"draw.rs:{call['line']} {call['name']} in {call['fn']}, a helper nobody has read")
        elif call["name"] not in allowed:
            plan.problem(f"draw.rs:{call['line']} {call['fn']} now calls {call['name']}; reread it")
    order = [c["name"] for c in src.calls if c["fn"] == "predict"]
    if order != ["wait_last", "hold", "mark", "listen", "wait_until"]:
        plan.problem(f"draw::predict's order changed to {order}; every hold's basis states the old one")
    order = [c["name"] for c in src.calls if c["fn"] == "reading_window"]
    if order != ["wait_until", "hold", "pause"]:
        plan.problem(f"draw::reading_window's order changed to {order}; every reading hold's basis states the old one")
    window = [c for c in src.calls if c["fn"] == "reading_window"]
    if len(window) == 3 and (window[0]["args"] != ["stage", "line.end()"] or window[2]["args"] != ["stage", "seconds"]):
        plan.problem("draw::reading_window no longer waits for line.end() and then its seconds")
    for start, end in src.fns.get("reading_window", []):
        touched = re.search(r"\b(\w*fade\w*|remove_from_scene|clear_updaters|play|mark|listen|music_breath)\s*\(", src.t[start:end])
        if touched:
            plan.problem(f"draw::reading_window clears or draws: it calls {touched.group(1)}")
    forwarded = [c for c in src.calls if c["fn"] == "say" and c["name"] == "say"]
    if len(forwarded) != 1 or forwarded[0]["source"] != "kit.say(stage, id)?;":
        plan.problem("draw::say no longer forwards exactly once to kit.say")
    return {
        "say_forward": {"line": forwarded[0]["line"], "marker": forwarded[0]["source"]} if forwarded else None,
        "predict": [{"line": c["line"], "call": c["name"], "marker": c["source"]} for c in src.calls if c["fn"] == "predict"],
        "reading_window": [
            {"line": c["line"], "call": c["name"], "marker": c["source"]} for c in src.calls if c["fn"] == "reading_window"
        ],
    }


def read_table(raw, table, plan):
    """The rows of `pub const <table>`, which are `("NN_M", seconds),`, comments apart."""
    rows, inside = [], False
    for number, row in enumerate(raw.split("\n"), 1):
        if row.startswith(f"pub const {table}"):
            inside = True
        elif inside and row.startswith("];"):
            break
        elif inside and not row.strip().startswith("//"):
            m = PREDICT_ROW.match(row)
            if not m:
                plan.problem(f"mod.rs:{number} unreadable {table} row: {row.strip()}")
            else:
                rows.append({"cue": m.group(1), "seconds": float(m.group(2)), "line": number, "marker": row.strip()})
    return rows


def read_storyboard(raw, script, plan):
    rows = {}
    for number, row in enumerate(raw.split("\n"), 1):
        if not re.match(r"\|\s*\d\d_\d+\s*\|", row):
            continue
        cells = [c.strip() for c in re.split(r"(?<!\\)\|", row.strip())[1:-1]]
        if len(cells) != 5:
            plan.problem(f"storyboard:{number} has {len(cells)} cells")
            continue
        cue, words, hold = cells[0], cells[1], cells[4]
        spoken = re.sub(r"\*\(.*?\)\*", " ", words).replace("⟨predict⟩", " ")
        spoken = " ".join(spoken.split())
        if script.get(cue) != spoken:
            plan.problem(f"storyboard:{number} words of {cue} differ from script.tsv")
        rows[cue] = {"predict": "⟨predict⟩" in words, "hold": hold}
    if list(rows) != list(script):
        plan.problem("storyboard rows differ from script cues or their order")
    return rows


def source_row(rel, data):
    return {"path": rel, "sha256": sha(data)}


def registered_modules(raw):
    """The `mod x;` lines of chapters/mod.rs, outside its tests: the files in the film."""
    return re.findall(r"^(?:pub )?mod (\w+);", raw, re.M)


def build(repo, crate, version):
    plan = Plan()
    script_path = os.path.join(crate, "script.tsv")
    script_bytes = bytes_of(script_path)
    script_sha = sha(script_bytes)
    script = read_script(script_path)
    cues = list(script)
    chapters_dir = os.path.join(crate, "src", "chapters")
    sources = {}
    registered_files = {"mod.rs"} | {f"{m}.rs" for m in registered_modules(text_of(os.path.join(chapters_dir, "mod.rs")))}
    ignored = []
    for path in sorted(glob.glob(os.path.join(chapters_dir, "*.rs"))):
        name = os.path.basename(path)
        if name in registered_files:
            sources[name] = Source(name, text_of(path), plan)
        else:
            ignored.append(source_row(f"{TOPIC_REL}/src/chapters/{name}", bytes_of(path)))
    for name in sorted(registered_files - set(sources)):
        plan.problem(f"mod.rs registers {name}, which is not on disk")
    for path in sorted(glob.glob(os.path.join(crate, "src", "*.rs"))):
        extra = Source(os.path.basename(path), text_of(path), plan)
        strict = [c for c in extra.calls if c["name"] in STRICT]
        if strict:
            plan.problem(f"src/{extra.name}:{strict[0]['line']} has a narration call outside the chapters")

    # ------------------------------------------------------- chapters and say
    registry = re.findall(r'key: "(\d\d_\w+)"', sources["mod.rs"].t and sources["mod.rs"].raw)
    registered = {key[:2]: key for key in registry}
    rows = []
    timed = {}
    for name, src in sources.items():
        if name in NOT_CHAPTERS:
            strays = [c for c in src.calls if c["name"] == "say" and name != "draw.rs"]
            if strays:
                plan.problem(f"{name}:{strays[0]['line']} says a cue outside a chapter")
            continue
        found, calls = analyse_chapter(src, script, plan)
        timed[name] = calls
        scene = re.search(r'fn name\(&self\) -> &str \{\s*"(\d\d_\w+)"', src.raw)
        for row in found:
            if not scene or registered.get(row["cue"][:2]) != scene.group(1):
                plan.problem(f"{name}: cue {row['cue']} is not in its registered chapter")
        order = [r["cue"] for r in found]
        expected = sorted(order, key=lambda c: (c[:2], int(c[3:])))
        if order != expected:
            plan.problem(f"{name}: cues said out of script order: {order}")
        rows.extend(found)
    said = collections.Counter(r["cue"] for r in rows)
    for cue in cues:
        if said[cue] != 1:
            plan.problem(f"cue {cue} is said {said[cue]} times in the source")
    for cue in said:
        if cue not in script:
            plan.problem(f"source says {cue}, which is not in script.tsv")
    prefixes = sorted({c[:2] for c in cues})
    if prefixes != sorted(registered):
        plan.problem(f"script chapters {prefixes} differ from the registry {sorted(registered)}")
    by_cue = {r["cue"]: r for r in rows}

    # ------------------------------------------------------- text bounds
    for cue, text in script.items():
        words = len(text.split())
        if not 1 <= words <= MAX_WORDS:
            plan.problem(f"cue {cue} has {words} words, outside 1 to {MAX_WORDS}")

    # ------------------------------------------------------- predictions
    draw_evidence = check_draw(sources["draw.rs"], plan)
    predicts = read_table(sources["mod.rs"].raw, "PREDICTS", plan)
    windows = {p["cue"]: p for p in predicts}
    readings = read_table(sources["mod.rs"].raw, "READING_WINDOWS", plan)
    reading_table = {r["cue"]: r for r in readings}
    called = {}
    for name, calls in timed.items():
        for _, call, row in calls:
            if call["name"] == "predict":
                if row["cue"] in called:
                    plan.problem(f"{name}:{call['line']} second prediction on {row['cue']}")
                called[row["cue"]] = (call, row)
    if set(called) != set(windows):
        plan.problem(f"predict calls and PREDICTS differ: {sorted(set(called) ^ set(windows))}")
    board = read_storyboard(text_of(os.path.join(crate, "docs", "storyboard.md")), script, plan)
    for cue, beat in board.items():
        listen = re.fullmatch(r"listen (\d+(?:\.\d+)?) s", beat["hold"])
        read = re.fullmatch(r"read (\d+(?:\.\d+)?) s", beat["hold"])
        if read and (cue not in reading_table or float(read.group(1)) != reading_table[cue]["seconds"]):
            plan.problem(f"storyboard's reading window on {cue} ({beat['hold']!r}) differs from READING_WINDOWS")
        if cue in reading_table and not read:
            plan.problem(f"READING_WINDOWS has {cue} but the storyboard's hold column says {beat['hold']!r}")
        if beat["predict"] != (cue in windows):
            plan.problem(f"storyboard's predict mark on {cue} differs from PREDICTS")
        if beat["predict"] and (not listen or cue not in windows or float(listen.group(1)) != windows[cue]["seconds"]):
            plan.problem(f"storyboard's window for {cue} ({beat['hold']!r}) differs from PREDICTS")
        if beat["hold"] not in ("", "hold") and not listen and not read:
            plan.problem(f"storyboard's hold column on {cue} is unclassified: {beat['hold']!r}")

    overrides, holds, prediction_rows = [], [], []
    for entry in predicts:
        cue = entry["cue"]
        if cue not in called or cue not in script:
            continue
        call, row = called[cue]
        path = f"{TOPIC_REL}/src/chapters/{row['file']}"
        where = f"{row['file']}:{call['line']}"
        if not script[cue].rstrip().endswith("?"):
            plan.problem(f"{where} prediction {cue} does not end in a question")
        at = cues.index(cue)
        next_cue = cues[at + 1] if at + 1 < len(cues) else None
        if next_cue is None or next_cue[:2] != cue[:2]:
            plan.problem(f"{where} prediction {cue} has no next cue in its chapter, so delivery would not hold it")
            continue
        src = sources[row["file"]]
        following = by_cue.get(next_cue)
        if following is None or following["file"] != row["file"]:
            plan.problem(f"{where} next cue {next_cue} is not said in the same file")
            continue
        stray = [
            c
            for _, c, r in timed[row["file"]]
            if r is row and c["at"] > call["at"] and not c["name"].startswith("mark")
        ]
        if stray:
            plan.problem(f"{where} timing call between predict and {next_cue}: {stray[0]['source']}")
        data = bytes_of(os.path.join(chapters_dir, row["file"]))
        seconds = entry["seconds"]
        overrides.append(
            {
                "video": TOPIC,
                "cue": cue,
                "after": script[cue],
                "role": "visual",
                "reason": (
                    "Prediction window after this complete cue, before the next cue: "
                    "held on the native picture timeline, not as silence in the take."
                ),
            }
        )
        holds.append(
            {
                "video": TOPIC,
                "cue": cue,
                "next_cue": next_cue,
                "kind": "prediction",
                "path": path,
                "sha256": sha(data),
                "line": call["line"],
                "marker": call["source"],
                "basis": (
                    "draw::predict shows the card as the question's last sentence begins, lets the cue finish "
                    f"plus the kit breath, marks predict, then waits the {seconds:g} s listen window that "
                    "chapters::PREDICTS authors, before the next cue. Picture timeline only: no silence is asked "
                    "of the take; no listening claim."
                ),
            }
        )
        prediction_rows.append(
            {
                "cue": cue,
                "next_cue": next_cue,
                "window_s_in_source": seconds,
                "predicts_line": entry["line"],
                "predicts_marker": entry["marker"],
                "file": row["file"],
                "call_line": call["line"],
                "call_marker": call["source"],
                "question": report_sentences(script[cue])[-1],
                "timing_calls_between_predict_and_next_say": 0,
            }
        )

    # ------------------------------------------------------- reading windows
    breath_found = re.search(r"pub const BREATH: f64 = ([\d.]+);", text_of(os.path.join(repo, "crates/explainer_kit/src/narration.rs")))
    breath_s = float(breath_found.group(1)) if breath_found else None
    if breath_s is None:
        plan.problem("narration.rs lost pub const BREATH")
    reading_called = {}
    for name, calls in timed.items():
        for _, call, row in calls:
            if call["name"] == "reading_window":
                if row["cue"] in reading_called:
                    plan.problem(f"{name}:{call['line']} second reading window on {row['cue']}")
                reading_called[row["cue"]] = (call, row)
    if set(reading_called) != set(reading_table):
        plan.problem(f"reading_window calls and READING_WINDOWS differ: {sorted(set(reading_called) ^ set(reading_table))}")
    table_order = [r["cue"] for r in readings]
    if table_order != [c for c in cues if c in reading_table]:
        plan.problem("READING_WINDOWS is not in script order")
    reading_rows = []
    for entry in readings:
        cue = entry["cue"]
        if cue not in reading_called or cue not in script:
            continue
        call, row = reading_called[cue]
        seconds = entry["seconds"]
        where = f"{row['file']}:{call['line']}"
        if cue in windows:
            plan.problem(f"{where} reading window on {cue}, a prediction: it would delay the answer")
        if seconds < READING_MIN_S or (breath_s is not None and seconds + breath_s >= GAP_LIMIT_S):
            plan.problem(f"{where} reading window of {seconds:g} s on {cue} is outside {READING_MIN_S:g} s and a {GAP_LIMIT_S:g} s gap")
        at = cues.index(cue)
        next_cue = cues[at + 1] if at + 1 < len(cues) else None
        if next_cue is None or next_cue[:2] != cue[:2]:
            plan.problem(f"{where} reading window on {cue} has no next cue in its chapter")
            continue
        following = by_cue.get(next_cue)
        if following is None or following["file"] != row["file"]:
            plan.problem(f"{where} next cue {next_cue} is not said in the same file")
            continue
        src = sources[row["file"]]
        cards = [
            c
            for _, c, r in timed[row["file"]]
            if r is row and c["name"] == "precisely_now" and c["at"] < call["at"]
        ]
        named = re.match(r"let (\w+) = precisely_now\(", cards[0]["source"]) if len(cards) == 1 else None
        if named is None:
            plan.problem(f"{where} reading window on {cue} with no precisely card to read ({len(cards)} found)")
            continue
        card = named.group(1)
        if re.search(r"\b(\w*fade_out\w*|remove_from_scene)\b", src.t[cards[0]["at"] : call["at"]]):
            plan.problem(f"{where} the {card} card can go before its window")
        end = src.t.index(";", close_of(src.t, src.t.index("(", call["at"]))) + 1
        between = src.t[end : following["at"]].strip()
        if between != "let line =":
            plan.problem(f"{where} something between the reading window and {next_cue}: {' '.join(between.split())[:80]}")
        later = [r["at"] for r in rows if r["file"] == row["file"] and r["at"] > following["at"]]
        span = src.fns["construct"][0][1]
        cleared = src.t[following["at"] : min(later) if later else span]
        if "fade_out_all" not in cleared or not re.search(rf"\b{card}\b", cleared):
            plan.problem(f"{where} the {card} card is not cleared in {next_cue}")
        constant = cards[0]["args"][2] if len(cards[0]["args"]) > 2 else ""
        task = reading_task(src, cards[0], None, script[cue])
        data = bytes_of(os.path.join(chapters_dir, row["file"]))
        overrides.append(
            {
                "video": TOPIC,
                "cue": cue,
                "after": script[cue],
                "role": "visual",
                "reason": (
                    "Reading window after this complete cue, before the next cue: "
                    "held on the native picture timeline, not as silence in the take."
                ),
            }
        )
        holds.append(
            {
                "video": TOPIC,
                "cue": cue,
                "next_cue": next_cue,
                "kind": "reading",
                "path": f"{TOPIC_REL}/src/chapters/{row['file']}",
                "sha256": sha(data),
                "line": call["line"],
                "marker": call["source"],
                "basis": (
                    f"draw::reading_window lets the cue finish plus the kit breath, then waits the {seconds:g} s that "
                    f"chapters::READING_WINDOWS authors, with the {card} card on screen and nothing drawn or cleared, "
                    "before the next cue. A proposed teaching pause chosen from the card's text, not a measured reading "
                    "time. Picture timeline only: no silence is asked of the take; no listening claim."
                ),
            }
        )
        reading_rows.append(
            {
                "cue": cue,
                "next_cue": next_cue,
                "window_s_in_source": seconds,
                "table_line": entry["line"],
                "table_marker": entry["marker"],
                "file": row["file"],
                "call_line": call["line"],
                "call_marker": call["source"],
                "card": card,
                "card_const": constant,
                "card_line": cards[0]["line"],
                "card_chars": task["text_chars"],
            }
        )
    order_of = {cue: i for i, cue in enumerate(cues)}
    overrides.sort(key=lambda o: order_of[o["cue"]])
    holds.sort(key=lambda h: order_of[h["cue"]])

    # ------------------------------------------------------- music windows
    music = []
    breath_cues = set()
    for name, calls in timed.items():
        for _, call, row in calls:
            if call["name"] != "music_breath":
                continue
            cue = row["cue"]
            breath_cues.add(cue)
            at = cues.index(cue)
            next_cue = cues[at + 1] if at + 1 < len(cues) else None
            if next_cue is not None and next_cue[:2] == cue[:2]:
                plan.problem(f"{name}:{call['line']} music breath inside chapter {cue[:2]}: classify it")
            music.append(
                {
                    "cue": cue,
                    "next_cue": next_cue,
                    "seconds_in_source": call["args"][1] if len(call["args"]) > 1 else None,
                    "file": name,
                    "line": call["line"],
                    "marker": call["source"],
                }
            )
    music.sort(key=lambda m: (cues.index(m["cue"]), m["line"]))
    board_holds = {c for c, b in board.items() if b["hold"] == "hold"}
    if board_holds != breath_cues:
        plan.problem(f"storyboard holds and music breaths differ: {sorted(board_holds ^ breath_cues)}")

    # ------------------------------------------------------- kit evidence
    kit_rows = []
    for rel, markers in KIT_MARKERS.items():
        path = os.path.join(repo, rel)
        raw = text_of(path)
        lines = [s.strip() for s in raw.split("\n")]
        row = source_row(rel, bytes_of(path))
        row["definitions"] = []
        for marker in markers:
            if marker not in lines:
                plan.problem(f"{rel} no longer defines: {marker}")
            else:
                row["definitions"].append({"line": lines.index(marker) + 1, "marker": marker})
        kit_rows.append(row)
    gaps = re.search(r"line_gaps: explainer_kit::narration::LineGaps::(\w+),", text_of(os.path.join(crate, "src", "main.rs")))
    canonical_map = os.path.join(repo, TOPIC_REL, "docs", f"narration-direction-map-{version}.json")

    # ------------------------------------------------------- outputs
    map_rel = f"{TOPIC_REL}/docs/narration-direction-map-{version}.json"
    helper_paths = [
        ("draw.rs", "pub fn predict("),
        ("draw.rs", "pub fn reading_window("),
        ("mod.rs", "pub const PREDICTS"),
        ("mod.rs", "pub const READING_WINDOWS"),
    ]
    helpers = []
    for name, start in helper_paths:
        lines = sources[name].raw.split("\n")
        number = next((i for i, row in enumerate(lines, 1) if row.startswith(start)), None)
        if number is None:
            plan.problem(f"{name} lost {start}")
            continue
        helpers.append(
            {
                "path": f"{TOPIC_REL}/src/chapters/{name}",
                "sha256": sha(bytes_of(os.path.join(chapters_dir, name))),
                "line": number,
                "marker": lines[number - 1].strip(),
            }
        )
    direction = {
        "schema_version": 1,
        "scripts": {TOPIC: script_sha},
        "overrides": overrides,
        "visual_holds": holds,
        "hold_kinds": {"prediction": len(prediction_rows), "reading": len(reading_rows)},
        "helpers": helpers,
        "scope": (
            f"Proposal (bead 87do.39, reading-repair01): {len(prediction_rows)} authored prediction windows and "
            f"{len(reading_rows)} authored reading windows bound to source. A reading row exists only where the source "
            "adds a wait. The reading seconds are a proposed pacing change chosen from each card's text, not a "
            "measurement of any take or reader. These are native picture-timeline waits, not silence in any take. "
            "No take, listening, timing or independent acceptance."
        ),
    }
    map_text = dumps(direction)
    units = {
        "schema_version": 1,
        "script_sha256": script_sha,
        "direction_contract": {
            "schema_version": 1,
            "topic": TOPIC,
            "script_sha256": script_sha,
            "map": {"path": map_rel, "sha256": sha(map_text.encode())},
        },
        "scope": (
            f"Candidate full-cue source plan for ElevenLabs Oliver: {len(cues)} retained cues, one take per cue, "
            "exact script text. No take selection, listening, timing or independent acceptance."
        ),
        "units": [{"id": f"oliver_{cue}", "line_ids": [cue], "text": text} for cue, text in script.items()],
    }

    cue_rows = []
    mismatch = []
    reading = []
    for i, cue in enumerate(cues):
        text = script[cue]
        row = by_cue.get(cue)
        following = cues[i + 1] if i + 1 < len(cues) else None
        offsets = len(sentence_offsets(text))
        if offsets != len(report_sentences(text)):
            mismatch.append(cue)
        cue_rows.append(
            {
                "cue": cue,
                "unit": f"oliver_{cue}",
                "next_cue": following,
                "next_cue_crosses_chapter": following is not None and following[:2] != cue[:2],
                "words": len(text.split()),
                "words_with_next_cue": len(text.split()) + len(script[following].split()) if following else None,
                "sentences": offsets,
                "file": row["file"] if row else None,
                "line": row["line"] if row else None,
                "marker": row["marker"] if row else None,
                "voice_end_calls": row["voice_end_calls"] if row else [],
                "calls": row["calls"] if row else [],
            }
        )
        if row:
            reading.extend(
                {"cue": cue, "file": row["file"], **task}
                for task in row["reading_tasks"]
                if not (cue in reading_called and task["call"] == "precisely_now")
            )
    counts = collections.Counter()
    for calls in timed.values():
        for _, call, _row in calls:
            counts[call["name"].split(":")[0]] += 1
    non_literal = []
    for (name, source), entry in sorted(DYNAMIC.items()):
        if entry["from"] not in sources[name].raw:
            plan.problem(f"{name}: DYNAMIC's source line is gone: {entry['from']}")
        hits = [c for c in sources[name].calls if c["source"] == source]
        if not hits:
            plan.problem(f"{name}: DYNAMIC's call is gone: {source}")
        non_literal.append(
            {
                "file": name,
                "lines": [c["line"] for c in hits],
                "call": source,
                "resolved_from": entry["from"],
                "values": entry.get("phrases") or entry.get("sentences"),
                "note": entry.get("note"),
            }
        )
    words = [len(t.split()) for t in script.values()]
    pairs = [len(script[a].split()) + len(script[b].split()) for a, b in zip(cues, cues[1:])]
    all_sources = [source_row(f"{TOPIC_REL}/script.tsv", script_bytes)]
    all_sources.append(source_row(f"{TOPIC_REL}/docs/storyboard.md", bytes_of(os.path.join(crate, "docs", "storyboard.md"))))
    all_sources.append(source_row(f"{TOPIC_REL}/src/main.rs", bytes_of(os.path.join(crate, "src", "main.rs"))))
    for name in sorted(sources):
        all_sources.append(source_row(f"{TOPIC_REL}/src/chapters/{name}", bytes_of(os.path.join(chapters_dir, name))))
    thumbnail = [
        {"file": c["file"], "line": c["line"], "marker": c["source"]}
        for c in sources["thumbnail.rs"].calls
        if c["name"] == "wait"
    ]
    audit = {
        "schema_version": 1,
        "topic": TOPIC,
        "claim": "Text audit of Rust source. Uncompiled, unrendered. No take, duration, listening or acceptance.",
        "script_sha256": script_sha,
        "map_version": version,
        "sources": all_sources,
        "kit_sources": kit_rows,
        "kit_breath_s": breath_s,
        "crate_line_gaps": gaps.group(1) if gaps else None,
        "counts": {
            "cues": len(cues),
            "units": len(units["units"]),
            "chapters": len(prefixes),
            "say_calls": len(rows),
            "non_literal_say_arguments": 0,
            "words_total": sum(words),
            "words_max": max(words),
            "cues_over_90_words": [c for c in cues if len(script[c].split()) > 90],
            "adjacent_pairs_over_100_words": sum(1 for n in pairs if n > MAX_WORDS),
            "predict_calls": len(called),
            "predicts_rows": len(predicts),
            "prediction_holds": len(prediction_rows),
            "reading_holds": len(reading_rows),
            "reading_windows_total_s": sum(r["window_s_in_source"] for r in reading_rows),
            "reading_windows_by_chapter_s": {
                ch: sum(r["window_s_in_source"] for r in reading_rows if r["cue"][:2] == ch)
                for ch in sorted({r["cue"][:2] for r in reading_rows})
            },
            "music_breath_calls": len(music),
            "music_breath_cues": len(breath_cues),
            "timing_calls_in_chapter_construct": dict(sorted(counts.items())),
            "storyboard_hold_column": dict(sorted(collections.Counter(b["hold"] for b in board.values()).items())),
            "sentence_rule_mismatches": mismatch,
        },
        "meanings": HOLD_MEANINGS,
        "draw_helpers": draw_evidence,
        "non_literal_wait_arguments": non_literal,
        "prediction_holds": prediction_rows,
        "reading_windows": reading_rows,
        "unregistered_files_ignored": ignored,
        "music_windows": music,
        "reading_tasks_without_a_native_hold": reading,
        "not_in_the_film": thumbnail,
        "cues": cue_rows,
    }
    outputs = {
        f"{TOPIC_REL}/narration_units.json": dumps(units),
        map_rel: map_text,
        "source-audit.json": dumps(audit),
    }
    return plan, outputs, audit, canonical_map, map_text


def dumps(value):
    return json.dumps(value, indent=2, ensure_ascii=False) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Bind the Oliver plan of complex_analysis_visual to its source.")
    parser.add_argument("repo")
    parser.add_argument("crate")
    parser.add_argument("out")
    parser.add_argument("--map-version", default="oliver08")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    plan, outputs, audit, canonical_map, map_text = build(args.repo, args.crate, args.map_version)
    if os.path.exists(canonical_map) and bytes_of(canonical_map) != map_text.encode():
        plan.problem(f"map version {args.map_version} already exists in canonical with other bytes; pick the next version")
    if plan.problems:
        for text in plan.problems:
            print("REFUSED: " + text, file=sys.stderr)
        print(f"{len(plan.problems)} problems; nothing written", file=sys.stderr)
        return 1
    if args.check:
        stale = []
        for rel, text in outputs.items():
            path = os.path.join(args.out, rel)
            if not os.path.exists(path) or bytes_of(path) != text.encode():
                stale.append(rel)
        for rel in stale:
            print("STALE: " + rel, file=sys.stderr)
        if stale:
            return 1
    else:
        for rel, text in outputs.items():
            path = os.path.join(args.out, rel)
            if os.path.exists(path) and bytes_of(path) != text.encode():
                print(f"REFUSED: {rel} exists with other bytes; rebind into a new OUT", file=sys.stderr)
                return 1
        for rel, text in outputs.items():
            path = os.path.join(args.out, rel)
            os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
            if not os.path.exists(path):
                with open(path, "x", encoding="utf8", newline="") as handle:
                    handle.write(text)
    c = audit["counts"]
    print(
        f"{'in step' if args.check else 'written'}: {c['units']} units for {c['cues']} cues in {c['chapters']} chapters; "
        f"{c['say_calls']} say calls, {c['non_literal_say_arguments']} non-literal; "
        f"{c['prediction_holds']} prediction holds; {c['reading_holds']} reading holds; "
        f"{c['music_breath_calls']} music breaths on {c['music_breath_cues']} cues; longest cue {c['words_max']} words"
    )
    print("script " + audit["script_sha256"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
