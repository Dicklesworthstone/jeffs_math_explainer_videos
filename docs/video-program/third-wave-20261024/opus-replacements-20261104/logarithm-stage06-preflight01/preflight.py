#!/usr/bin/env python3
"""Read-only preflight for the logarithm film's combined-repair-stage06 candidate.

    python3 -I -B <this file>                       the staged candidate, before integration
    python3 -I -B <this file> --mode canonical      the installed crate, after integration
    python3 -I -B <this file> --phrase-timings FILE [--narration DIR]
                                                    also gate a measured phrase-timings.json
    other options: --no-pins  --quiet  --repo ROOT

Run from the repo root. Exit 0: no FAIL. Exit 1: at least one FAIL. Exit 2: usage.

It writes nothing in the repo, compiles nothing and runs no scene. It reads bytes, parses
Rust into tokens with its own lexer, and calls the real consumer functions in tools/ on the
real plan. It does not import the binder, so it can disagree with it. The only subprocesses
are `patch` in a temp folder and the crate's own check_cues.py and oracle.py.

Text checks cannot see types, borrows, pixels or sound. A pass here says the plan and the
source agree. It does not say the film builds, looks right or sounds right.

Narration is ElevenLabs Oliver. No Michael, no FrankenTTS.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

sys.dont_write_bytecode = True

TOPIC = "logarithm_mysteries"
REL = Path("explainer_corpus") / TOPIC
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE = LANE / "combined-repair-stage06"
OPUS = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
RELEASE = LANE.parent / "five-film-release-20261027"
BINDING = RELEASE / "logarithm-stage06-binding01/command.log"
PREVIEW = RELEASE / "logarithm-combined-repair-preview01/resource-receipt.json"
MAP_NAME = "narration-direction-map-oliver13.json"

# The frozen candidate this preflight was written against (--no-pins skips only these).
PINS = {
    "script.tsv": "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84",
    "narration_units.json": "d594089297ed09534bf4a291f1ddb33d10402da3d60a2aeee9f7175a4381aee2",
    MAP_NAME: "56f6ad64344514005a5ac385eb7826fa276346fb6c449acf54bf3de191da7e27",
    "hold-audit.json": "610e9f49db586cc9c307a9a16fd0d9d9ae7d70d7b29769f56eed01f1b238bf14",
    "phrase-anchors.json": "01b7d1a54d08062e1b45d725d990e5684c8a92da4ffc495475bc6f4adac60d76",
    "phrase-timings-unmeasured-template.json": "141a635e86e430711b61a527cf5e05763bddb5ed6ad9e11af7126a6b6415fd5a",
    "direction-dispositions-proposed.json": "5239cb21f91543c16dbf647324ec67f524915f1f64c8ba3d08c9b5049d681049",
    "cue-delta.json": "125a049a035dc3f7db59e8fc1931591358e625b5231d64e3ba9ad19e3275d988",
    "inputs.sha256": "b4926a8f35044f8ba7b686c266bd2a6695a7b7a17bf73c1cab34f31e98bb418d",
}
# chapter file -> (bytes before the repair, the repair's patch, bytes in stage06)
REPAIRS = {
    "undoing_growth.rs": ("8e26f3e4ae1e12b65bce5441e260f7beaf5d6fc0ab45b847f7a994f660f5ed36", "28452f65bfcc879235edccbd9025a7ab3190a8fc1dcf8b32a48c3084b053fc4d", "754ea3125c2f7e89440807772713fe57584c96937be3ec0ac8c52d86645cf323"),
    "complex_turns.rs": ("d5f202afb97debcef1a3f7abd872634b7cc7093108e87975167b04b065e212d5", "1e13c30a27349394c09595528e1bc641909a9d1fab178679b94e3fa5a4ebeae0", "2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead"),
    "lifting_loops.rs": ("78af9acdd126c12035ec865280b91b303921c228cab7f79023987fb90ec228c6", "a93717e1eb36ad2d5f041458714c824fb4b4f6bdcc6f65ab37cebf1125a5971a", "ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e"),
    "doing_nothing.rs": ("535e985522f976fe83ed17a36f5ddd9a2fb29467225162aeb9f119ef13398a43", "a19844a879689d120c78d3b3b93fad3ee50f683a7217fcebb3fc29451e12afb8", "eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143"),
    "unreachable.rs": ("43c8dac362df7fac8d6c974b3c108a25dee677faac85d093bfdafc1564db6527", "46054cb0fc6dc0dd1f596071eec41b6c0338559e19d36724c9c8227f44fbc001", "a676dac78580ef73f0e658a21c0395cfe0505fe631bd6aa0d71b1f3ec05a86ee"),
}
# chapter file -> (the patch stage06 keeps, the patch that was reviewed)
PATCH_FILES = {
    "undoing_growth.rs": (STAGE / "patches/04-undoing_growth.patch", OPUS / "logarithm-chapters04-06-review01/patches/04-undoing_growth.patch"),
    "complex_turns.rs": (STAGE / "patches/05-complex_turns.patch", OPUS / "logarithm-chapters04-06-review01/patches/05-complex_turns.patch"),
    "lifting_loops.rs": (STAGE / "patches/06-lifting_loops.patch", OPUS / "logarithm-chapters04-06-review01/patches/06-lifting_loops.patch"),
    "doing_nothing.rs": (STAGE / "patches/interval.patch", OPUS / "logarithm10-interval-repair01/interval.patch"),
    "unreachable.rs": (OPUS / "logarithm11-label-repair01/logarithm11-labels.patch", OPUS / "logarithm11-label-repair01/logarithm11-labels.patch"),
}
# 07 and 12: stage06 keeps the current canonical bytes. stage05 held older ones.
RETAINED = {
    "branch_cut.rs": ("193c4733de9f9081f5671700a6710dd76e989b41ab268824bbb0756313fbdbbd", "27edc168fbc47c1d"),
    "what_it_forgets.rs": ("e938b750ce579faed820ae7ddd0d289819f7148bdfb991dc99f24d35d6f8491b", "0b4c269bd3078c6a"),
}
# Paths whose staged bytes are expected to differ from canonical before integration.
STAGED_DELTA = {f"src/chapters/{name}" for name in REPAIRS} | {"narration_units.json"}
STAGE_ONLY = {"src/chapters/doing_nothing.rs.orig"}
CANONICAL_ONLY = {"docs/narration-direction-map-oliver12.json"}
SENTENCE = re.compile(r".*?[.?!](?= |$)|.+$")
# The tokenizer root's word-mark producer uses (elevenlabs-golay-20261028/phrase_marks.py).
MARK_WORD = re.compile(r"[A-Za-z]+(?:'[A-Za-z]+)?|[0-9]+")
PHRASE_HELPERS = {"until_phrase", "phrase_time"}
WAIT_CALLS = {"wait", "wait_frames", "wait_until", "hold", "finish", "listen", "sonify", "predict_pause"}
SENTENCE_BY_CUE = {"until_sentence", "sentence_time", "sentence_at", "until_share"}
SENTENCE_BY_STARTS = {"sentence_len", "room"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fsha(path):
    return sha(Path(path).read_bytes())


# ---------------------------------------------------------------------- a small Rust lexer
class Tok:
    __slots__ = ("kind", "value", "line")

    def __init__(self, kind, value, line):
        self.kind, self.value, self.line = kind, value, line

    def __repr__(self):
        return f"{self.kind}:{self.value!r}@{self.line}"


def unescape(raw):
    out, i = [], 0
    while i < len(raw):
        c = raw[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = raw[i + 1]
        if n == "\n":
            i += 2
            while i < len(raw) and raw[i] in " \t\n\r":
                i += 1
        elif n == "u":
            end = raw.index("}", i)
            out.append(chr(int(raw[i + 3 : end].replace("_", ""), 16)))
            i = end + 1
        elif n == "x":
            out.append(chr(int(raw[i + 2 : i + 4], 16)))
            i += 4
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


def lex(text):
    """Comments vanish. A string is one token, so a call written in a string is not a call."""
    toks, i, line, n = [], 0, 1, len(text)
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif c.isspace():
            i += 1
        elif text.startswith("//", i):
            while i < n and text[i] != "\n":
                i += 1
        elif text.startswith("/*", i):
            depth, i = 1, i + 2
            while depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    line += text[i] == "\n"
                    i += 1
        elif (m := re.match(r'b?r(#*)"', text[i : i + 12])):
            close = '"' + m.group(1)
            start = i + m.end()
            end = text.index(close, start)
            toks.append(Tok("str", text[start:end], line))
            line += text.count("\n", start, end)
            i = end + len(close)
        elif c == '"' or text.startswith('b"', i):
            start = i + (2 if c == "b" else 1)
            j = start
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            toks.append(Tok("str", unescape(text[start:j]), line))
            line += text.count("\n", start, j)
            i = j + 1
        elif c == "'":
            m = re.match(r"'(\\u\{[0-9a-fA-F_]+\}|\\x[0-9a-fA-F]{2}|\\.|[^\\'])'", text[i : i + 14])
            if m:
                toks.append(Tok("char", m.group(1), line))
                i += m.end()
            else:
                m = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", text[i : i + 60])
                toks.append(Tok("life", m.group(0), line))
                i += m.end()
        elif c.isalpha() or c == "_":
            m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i : i + 200])
            toks.append(Tok("id", m.group(0), line))
            i += m.end()
        elif c.isdigit():
            m = re.match(r"[0-9][0-9A-Za-z_]*(\.[0-9][0-9A-Za-z_]*)?", text[i : i + 80])
            toks.append(Tok("num", m.group(0), line))
            i += m.end()
        else:
            toks.append(Tok("p", c, line))
            i += 1
    return toks


def match_close(toks, at):
    pairs = {"(": ")", "[": "]", "{": "}"}
    opener = toks[at].value
    closer, depth = pairs[opener], 0
    for j in range(at, len(toks)):
        if toks[j].kind == "p":
            if toks[j].value == opener:
                depth += 1
            elif toks[j].value == closer:
                depth -= 1
                if not depth:
                    return j
    raise ValueError(f"unbalanced {opener} at line {toks[at].line}")


def drop_tests(toks):
    """Remove #[cfg(test)] items and #[test] functions: the film does not compile them."""
    out, i = [], 0
    while i < len(toks):
        if toks[i].value == "#" and i + 1 < len(toks) and toks[i + 1].value == "[":
            end = match_close(toks, i + 1)
            attr = "".join(t.value for t in toks[i + 2 : end] if t.kind in ("id", "p"))
            if attr in ("cfg(test)", "test"):
                j = end + 1
                while toks[j].value not in ("{", ";"):
                    j += 1
                i = (match_close(toks, j) if toks[j].value == "{" else j) + 1
                continue
        out.append(toks[i])
        i += 1
    return out


def functions(toks):
    spans = []
    for i, t in enumerate(toks):
        if t.kind == "id" and t.value == "fn" and toks[i + 1].kind == "id":
            j = i + 2
            while j < len(toks) and toks[j].value not in ("{", ";"):
                j += 1
            if j < len(toks) and toks[j].value == "{":
                spans.append((toks[i + 1].value, i, match_close(toks, j)))
    return spans


def enclosing(spans, at):
    best = None
    for name, a, b in spans:
        if a <= at <= b and (best is None or a > best[1]):
            best = (name, a)
    return best[0] if best else None


def calls(toks):
    """(index, name, args, receiver): every name(...) that is a call, not a definition."""
    rows = []
    for i, t in enumerate(toks[:-1]):
        if t.kind != "id" or toks[i + 1].value != "(":
            continue
        if i and toks[i - 1].value == "fn":
            continue
        end = match_close(toks, i + 1)
        args, cur, depth = [], [], 0
        for k in toks[i + 2 : end]:
            if k.kind == "p" and k.value in "([{":
                depth += 1
            elif k.kind == "p" and k.value in ")]}":
                depth -= 1
            if k.kind == "p" and k.value == "," and not depth:
                args.append(cur)
                cur = []
            else:
                cur.append(k)
        if cur:
            args.append(cur)
        recv = toks[i - 2].value if i >= 2 and toks[i - 1].value == "." else None
        rows.append((i, t.value, args, recv))
    return rows


def str_constants(toks):
    found, clash = {}, []
    for i in range(len(toks) - 5):
        if toks[i].value == "const" and toks[i + 1].kind == "id" and toks[i + 2].value == ":":
            j = i + 3
            while toks[j].value not in ("=", ";"):
                j += 1
            kinds = [t.value for t in toks[i + 3 : j]]
            if toks[j].value == "=" and kinds[-1] == "str" and toks[j + 1].kind == "str":
                name, value = toks[i + 1].value, toks[j + 1].value
                if name in found and found[name] != value:
                    clash.append(name)
                found[name] = value
    return found, clash


def const_value_at(toks, name):
    """The token index just after `const NAME: ... =`."""
    i = next(j for j, t in enumerate(toks) if t.value == name and toks[j - 1].value == "const")
    while toks[i].value != "=":
        i += 1
    return i + 1


def sentences(text):
    return [s.strip() for s in SENTENCE.findall(text) if s.strip()]


def read_script(path):
    """cue<TAB>text rows, in file order. Comment lines start with #."""
    rows = {}
    for number, row in enumerate(Path(path).read_bytes().decode("utf-8").split("\n"), 1):
        if not row or row.startswith("#"):
            continue
        if row.count("\t") != 1:
            raise ValueError(f"script row {number} is not cue<TAB>text")
        cue, text = row.split("\t")
        if cue in rows:
            raise ValueError(f"duplicate cue {cue}")
        rows[cue] = text
    return rows


# ---------------------------------------------------------------------- the run
class Run:
    def __init__(self, repo, mode, pins=True, quiet=False):
        self.repo, self.mode, self.pins, self.quiet = Path(repo).resolve(), mode, pins, quiet
        self.ok, self.fail, self.warn = 0, [], []
        self.stage = self.repo / STAGE
        self.plan = self.stage / "bound-plan"
        self.plan_crate = self.plan / REL
        self.stage_crate = self.stage / REL
        self.canon_crate = self.repo / REL
        self.crate = self.stage_crate if mode == "stage" else self.canon_crate
        self.chapters = self.crate / "src/chapters"
        self.units_path = (self.plan_crate if mode == "stage" else self.canon_crate) / "narration_units.json"
        self.map_path = (self.plan_crate if mode == "stage" else self.canon_crate) / "docs" / MAP_NAME

    def check(self, cid, good, detail=""):
        if good:
            self.ok += 1
            if not self.quiet:
                print(f"ok   {cid}  {detail}".rstrip())
        else:
            self.fail.append(cid)
            print(f"FAIL {cid}  {detail}".rstrip())
        return good

    def warning(self, cid, detail):
        self.warn.append(cid)
        print(f"WARN {cid}  {detail}")

    def info(self, cid, detail):
        if not self.quiet:
            print(f"info {cid}  {detail}")

    def load_tool(self, name):
        path = self.repo / "tools" / f"{name}.py"
        spec = importlib.util.spec_from_file_location(f"preflight_{name}", path)
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)
        return module

    def path_in_view(self, rel):
        """Where a repo-relative path a plan row names lives in the tree under test."""
        rel = Path(rel)
        if rel.is_relative_to(REL):
            return self.crate / rel.relative_to(REL)
        return self.repo / rel

    # ------------------------------------------------------------------ load
    def load(self):
        self.script_path = self.crate / "script.tsv"
        self.script = read_script(self.script_path)
        self.cues = list(self.script)
        self.script_sha = fsha(self.script_path)
        self.units = json.loads(self.units_path.read_bytes())
        self.map = json.loads(self.map_path.read_bytes())
        self.audit = json.loads((self.plan / "hold-audit.json").read_bytes())
        self.anchors = json.loads((self.plan / "phrase-anchors.json").read_bytes())
        self.template = json.loads((self.plan / "phrase-timings-unmeasured-template.json").read_bytes())
        self.dispositions = json.loads((self.plan / "direction-dispositions-proposed.json").read_bytes())
        self.delta = json.loads((self.plan / "cue-delta.json").read_bytes())
        self.requests = json.loads((self.repo / LANE / "narration-request-final80-01/requests.json").read_bytes())

    def parse_registry(self):
        toks = drop_tests(lex((self.chapters / "mod.rs").read_text(encoding="utf-8")))
        declared = {toks[i + 1].value for i, t in enumerate(toks[:-2]) if t.value == "mod" and toks[i + 2].value == ";"}
        reg = []
        for i, t in enumerate(toks[:-2]):
            if t.value == "key" and toks[i + 1].value == ":" and toks[i + 2].kind == "str":
                j = i + 3
                while not (toks[j].value == "Box" and toks[j + 1].value == ":" and toks[j + 3].value == "new"):
                    j += 1
                reg.append((toks[i + 2].value, toks[j + 5].value))
        return declared, reg

    def census(self, reg):
        """Every say, hold, wait and phrase or sentence hook in the compiled scene code, from tokens."""
        self.sources, self.lines, self.toks = {}, {}, {}
        says, holds, waits, hooks, sentence_hooks, marks, trouble = [], [], [], [], [], [], []
        for key, module in reg:
            name = f"{module}.rs"
            data = (self.chapters / name).read_bytes()
            text = data.decode("utf-8")
            self.sources[name] = data
            self.lines[name] = text.split("\n")
            toks = drop_tests(lex(text))
            self.toks[name] = toks
            spans = functions(toks)
            consts, clash = str_constants(toks)
            if clash:
                trouble.append(f"{name}: const with two values: {clash}")
            current = None
            for i, call, args, recv in calls(toks):
                line = toks[i].line
                position = next((k for k, a in enumerate(args) if len(a) == 1 and a[0].kind == "str" and re.fullmatch(r"\d\d_\d+", a[0].value)), None)
                literal = args[position][0].value if position is not None else None
                if call == "say" and recv is not None:
                    if literal is None:
                        trouble.append(f"{name}:{line}: a say with no literal cue")
                        continue
                    current = literal
                    says.append({"file": name, "line": line, "cue": literal})
                elif call == "predict" and recv is None:
                    holds.append({"file": name, "line": line, "cue": current, "kind": "prediction"})
                elif call == "music_breath":
                    holds.append({"file": name, "line": line, "cue": current, "kind": "reading"})
                elif call in WAIT_CALLS:
                    if call == "wait" and recv != "stage":
                        trouble.append(f"{name}:{line}: a wait on {recv}")
                    waits.append({"file": name, "line": line, "cue": current, "call": call})
                elif call == "mark" and recv == "kit":
                    marks.append({"file": name, "line": line})
                elif call in PHRASE_HELPERS:
                    if literal is None:
                        where = enclosing(spans, i)
                        if where not in PHRASE_HELPERS:
                            trouble.append(f"{name}:{line}: {call} with no literal cue, inside {where}")
                        continue
                    last = args[-1]
                    if len(last) == 1 and last[0].kind == "str":
                        phrase, const = last[0].value, None
                    elif len(last) == 1 and last[0].value in consts:
                        phrase, const = consts[last[0].value], last[0].value
                    else:
                        trouble.append(f"{name}:{line}: {call} whose phrase this cannot read")
                        continue
                    hooks.append({"file": name, "line": line, "call": call, "cue": literal, "phrase": phrase, "const": const, "current": current})
                elif call == "phrase_marks" and literal is not None:
                    hooks.append({"file": name, "line": line, "call": "phrase_marks", "cue": literal, "phrase": None, "const": None, "current": current})
                elif call in SENTENCE_BY_CUE and literal is not None:
                    index = args[position + 1] if position + 1 < len(args) else []
                    number = int(index[0].value) if len(index) == 1 and index[0].kind == "num" else None
                    sentence_hooks.append({"file": name, "line": line, "cue": literal, "index": number, "current": current, "call": call})
                elif call in SENTENCE_BY_STARTS and args:
                    held = [t.value for t in args[0] if t.kind == "str" and re.fullmatch(r"\d\d_\d+", t.value)]
                    if held and any(t.value == "sentence_starts" for t in args[0]):
                        index = args[1] if len(args) > 1 else []
                        number = int(index[0].value) if len(index) == 1 and index[0].kind == "num" else None
                        sentence_hooks.append({"file": name, "line": line, "cue": held[0], "index": number, "current": current, "call": call})
        return says, holds, waits, hooks, sentence_hooks, marks, trouble

    # ------------------------------------------------------------------ groups
    def group_pins(self):
        if not self.pins:
            self.info("PIN", "skipped by --no-pins")
            return
        self.check("PIN.script", self.script_sha == PINS["script.tsv"], self.script_sha[:16])
        self.check("PIN.units", fsha(self.units_path) == PINS["narration_units.json"])
        self.check("PIN.map", fsha(self.map_path) == PINS[MAP_NAME])
        for name in ("hold-audit.json", "phrase-anchors.json", "phrase-timings-unmeasured-template.json", "direction-dispositions-proposed.json", "cue-delta.json", "inputs.sha256"):
            self.check(f"PIN.plan.{name}", fsha(self.plan / name) == PINS[name])
        for name, (_, _, after) in REPAIRS.items():
            self.check(f"PIN.chapter.{name}", fsha(self.chapters / name) == after)
        rows = [ln.split("  ", 1) for ln in (self.repo / BINDING).read_text().splitlines() if re.match(r"[0-9a-f]{64}  ", ln)]
        bad = [p for h, p in rows if fsha(self.plan / p) != h]
        self.check("PIN.binding_receipt", len(rows) == 8 and not bad, f"{len(rows)} plan files equal root's guarded run's log; different: {bad}")

    def group_inputs(self):
        rows = [ln.split("  ", 1) for ln in (self.plan / "inputs.sha256").read_text().splitlines()]
        moved = [p for h, p in rows if not (self.repo / p).is_file() or fsha(self.repo / p) != h]
        self.check("INPUT.plan_inputs_unmoved", not moved, f"{len(rows)} binder inputs; moved: {moved[:6]}")

    def group_assembly(self):
        tmp = Path(tempfile.mkdtemp(prefix="stage06-preflight-"))
        try:
            for name, (before, patch_sha, after) in REPAIRS.items():
                staged, reviewed = (self.repo / p for p in PATCH_FILES[name])
                same = fsha(staged) == fsha(reviewed)
                if self.pins:
                    same = same and fsha(staged) == patch_sha
                self.check(f"ASM.patch_is_the_reviewed_patch.{name}", same, staged.name)
                base = self.stage / "baseline" / REL / "src/chapters" / name
                if name == "unreachable.rs":
                    base = self.stage / "before-rebinding-source/unreachable-before-labels.rs"
                self.check(f"ASM.base_bytes.{name}", fsha(base) == before, fsha(base)[:16])
                work = tmp / name.replace(".", "_")
                target = work / REL / "src/chapters" / name
                target.parent.mkdir(parents=True)
                shutil.copyfile(base, target)
                run = subprocess.run(["patch", "-s", "-p1", "-F0", "-d", str(work), "-i", str(staged.resolve())], capture_output=True, text=True)
                expected = self.stage_crate / "src/chapters" / name
                self.check(f"ASM.replay.{name}", run.returncode == 0 and fsha(target) == fsha(expected), f"base + patch at zero fuzz == staged candidate; patch exit {run.returncode}")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        for name, (expected, stale_prefix) in RETAINED.items():
            here = fsha(self.chapters / name)
            self.check(f"ASM.retained.{name}", here == expected and not here.startswith(stale_prefix), f"{here[:16]}; stage05's stale {stale_prefix} is not back")
            older = self.stage / "before-rebinding-source" / REL / "src/chapters" / name
            self.check(f"ASM.retained_refreshed.{name}", fsha(older).startswith(stale_prefix), "the kept copy of what it replaced")

        stage = {p.relative_to(self.stage_crate).as_posix(): p for p in self.stage_crate.rglob("*") if p.is_file()}
        canon = {p.relative_to(self.canon_crate).as_posix(): p for p in self.canon_crate.rglob("*") if p.is_file() and "target" not in p.relative_to(self.canon_crate).parts}
        only_stage, only_canon = set(stage) - set(canon), set(canon) - set(stage)
        differ = {k for k in set(stage) & set(canon) if fsha(stage[k]) != fsha(canon[k])}
        if self.mode == "stage":
            self.check("ASM.delta_is_exactly_the_repairs", differ == STAGED_DELTA, f"differ from canonical: {sorted(differ)}")
            self.check("ASM.only_in_stage", only_stage == STAGE_ONLY, f"{sorted(only_stage)}")
            self.check("ASM.only_in_canonical", only_canon == CANONICAL_ONLY, f"{sorted(only_canon)}")
        else:
            self.check("ASM.installed_equals_stage", differ == {"narration_units.json"}, f"differ from stage06's crate: {sorted(differ)}")
            self.check("ASM.installed_adds_only_map13", only_canon == CANONICAL_ONLY | {f"docs/{MAP_NAME}"}, f"{sorted(only_canon)}")
            self.check("ASM.installed_units_are_the_plan", fsha(self.canon_crate / "narration_units.json") == fsha(self.plan_crate / "narration_units.json"))
            self.check("ASM.installed_map_is_the_plan", fsha(self.canon_crate / "docs" / MAP_NAME) == fsha(self.plan_crate / "docs" / MAP_NAME))
            self.check("ASM.old_map_preserved", (self.canon_crate / "docs/narration-direction-map-oliver12.json").is_file())
            self.check("ASM.orig_not_installed", not (self.canon_crate / "src/chapters/doing_nothing.rs.orig").exists())
        stale_units = json.loads((self.stage_crate / "narration_units.json").read_bytes())
        if stale_units.get("script_sha256") != self.script_sha:
            self.warning("ASM.stage_crate_units_are_stale", f"stage06's crate holds {len(stale_units['units'])} units for script {stale_units['script_sha256'][:12]}, bound to {stale_units['direction_contract']['map']['path'].rsplit('/', 1)[1]}. Install the bound-plan copy; never copy this one.")
        if (self.stage_crate / "src/chapters/doing_nothing.rs.orig").is_file():
            self.warning("ASM.stray_orig", "stage06's chapters hold doing_nothing.rs.orig, carried from stage04 and stage05. mod.rs does not declare it, so it does not compile. Do not install it.")

    def group_text(self):
        cues, script = self.cues, self.script
        self.check("TXT.eighty_cues", len(cues) == 80, f"{len(cues)}")
        self.check("TXT.cue_ids", all(re.fullmatch(r"\d\d_\d+", c) for c in cues))
        prefixes = [c[:2] for c in cues]
        self.check("TXT.chapters_contiguous", prefixes == sorted(prefixes) and sorted(set(prefixes)) == [f"{n:02d}" for n in range(13)], f"{len(set(prefixes))} chapters; ids are not numeric order (05_20 follows 05_2)")
        self.check("TXT.clean_text", all(t == t.strip() and "  " not in t and t[-1] in ".?!" for t in script.values()), "no stray spaces; every cue ends a sentence")
        self.check("TXT.no_digits", not [c for c, t in script.items() if re.search(r"\d", t)], "numbers spelled for the narrator")
        self.check("TXT.ascii", all(t.isascii() for t in script.values()), "no typographic characters to mismatch")
        u = self.units
        self.check("TXT.units_bind_script", u.get("schema_version") == 1 and u.get("script_sha256") == self.script_sha)
        rows = u.get("units", [])
        self.check("TXT.units_one_per_cue_in_order", [r["line_ids"] for r in rows] == [[c] for c in cues], f"{len(rows)} units")
        self.check("TXT.unit_ids", [r["id"] for r in rows] == [f"oliver_{c}" for c in cues])
        wrong = [r["id"] for r in rows if r["line_ids"] and r["text"] != script.get(r["line_ids"][0])]
        self.check("TXT.unit_text_exact", not wrong, f"different: {wrong[:5]}")
        words = [len(t.split()) for t in script.values()]
        self.check("TXT.unit_word_bound", max(words) <= 100 and min(words) >= 1, f"max {max(words)}, total {sum(words)}")
        contract = u.get("direction_contract") or {}
        self.check("TXT.contract_names_map13", contract.get("map", {}).get("path") == f"{REL.as_posix()}/docs/{MAP_NAME}", str(contract.get("map", {}).get("path")))
        self.check("TXT.contract_map_hash", contract.get("map", {}).get("sha256") == fsha(self.map_path))
        self.check("TXT.contract_script_and_topic", contract.get("script_sha256") == self.script_sha and contract.get("topic") == TOPIC)

        req = self.requests
        rows = req["requests"]
        self.check("TXT.requests_bind_script", req["script_sha256"] == self.script_sha)
        self.check("TXT.requests_one_per_cue", [r["cue"] for r in rows] == cues and all(r["id"] == r["cue"] and r["parts"] == 1 for r in rows))
        wrong = []
        for r in rows:
            data = (self.repo / r["text_path"]).read_bytes()
            if data != script.get(r["cue"], "").encode("utf-8") or sha(data) != r["text_sha256"] or sha(data) != r["file_sha256"] or len(data.decode().split()) != r["words"]:
                wrong.append(r["cue"])
        self.check("TXT.request_files_exact", not wrong, f"the text that would go to Oliver; different: {wrong[:5]}")
        named = self.repo / req["script_path"]
        self.check("TXT.requests_script_bytes", named.is_file() and fsha(named) == self.script_sha, "the script file requests.json names holds the same bytes")
        if "combined-repair-stage06" not in req["script_path"]:
            self.warning("TXT.requests_name_older_stage", f"requests.json's script_path is {req['script_path'].split('elevenlabs-logarithm-rebuild-20261104/')[-1].split('/')[0]}; its hash matches, so this is a label, not a wrong binding")
        stray = [p.name for p in (self.repo / LANE / "narration-request-final80-01").iterdir() if p.suffix == ".txt" and p.stem not in script]
        self.check("TXT.no_stray_request_text", not stray, f"{stray[:5]}")

        delta = self.delta
        self.check("TXT.delta_binds", [d["cue"] for d in delta["cues"]] == cues and all(d["text_sha256"] == sha(script[d["cue"]].encode()) for d in delta["cues"]))
        prior = {r["cue"]: r["prior_request_id"] for r in rows}
        self.check("TXT.delta_matches_requests", all(prior[d["cue"]] == d["request_id_with_exact_bytes"] for d in delta["cues"]), "cue-delta and requests.json name the same older request")
        chars = {r["cue"]: (self.repo / r["prior_text_path"]).read_bytes() for r in rows}
        wrong = [c for c in cues if chars[c] != script[c].encode()]
        self.check("TXT.older_request_bytes_same", not wrong, f"each older request id's text is byte-equal to its final cue; different: {wrong[:5]}")

    def group_registry(self):
        declared, reg = self.parse_registry()
        self.reg = reg
        self.check("REG.thirteen", len(reg) == 13, f"{len(reg)} registered")
        keys = [k for k, _ in reg]
        self.check("REG.in_cue_order", [k[:2] for k in keys] == sorted({c[:2] for c in self.cues}), "one chapter per cue prefix, in order")
        self.check("REG.modules_declared", all(m in declared for _, m in reg))
        self.check("REG.files_present", all((self.chapters / f"{m}.rs").is_file() for _, m in reg))
        self.check("REG.only_helpers_unregistered", declared - {m for _, m in reg} == {"draw", "thumbnail"}, f"{sorted(declared - {m for _, m in reg})}")
        recorded = [(r["key"], r["file"]) for r in self.audit["registered_chapters"]]
        self.check("REG.audit_agrees", recorded == [(k, f"{m}.rs") for k, m in reg])
        wrong = [r["file"] for r in self.audit["registered_chapters"] if fsha(self.chapters / r["file"]) != r["sha256"]]
        self.check("REG.audit_hashes", not wrong, f"different: {wrong}")
        names = []
        for key, module in reg:
            toks = drop_tests(lex((self.chapters / f"{module}.rs").read_text(encoding="utf-8")))
            found = None
            for i, t in enumerate(toks[:-1]):
                if t.value == "fn" and toks[i + 1].value == "name":
                    j = i + 2
                    while toks[j].value != "{":
                        j += 1
                    found = toks[j + 1].value
            if found != key:
                names.append((key, found))
        self.check("REG.scene_name_is_key", not names, f"each scene's name() returns its registry key; different: {names}")
        ignored = {r["path"].rsplit("/", 1)[1] for r in self.audit["unregistered_files_ignored"]}
        on_disk = {p.name for p in self.chapters.iterdir() if p.is_file()} - {f"{m}.rs" for _, m in reg} - {"draw.rs", "mod.rs", "thumbnail.rs"}
        if self.mode == "canonical":
            ignored = ignored - {"doing_nothing.rs.orig"}
        self.check("REG.unregistered_listed", ignored == on_disk, f"{sorted(on_disk)}")
        command = json.loads((self.repo / PREVIEW).read_bytes())["command"]
        rendered = [a for a in command if re.fullmatch(r"\d\d_[a-z_]+", a)]
        self.check("REG.last_render_used_these_keys", rendered == keys, "the argv of root's last preview names exactly these chapters")

        says, holds, waits, hooks, sentence_hooks, marks, trouble = self.census(reg)
        self.says, self.holds, self.waits, self.hooks, self.sentence_hooks, self.marks = says, holds, waits, hooks, sentence_hooks, marks
        self.check("REG.nothing_unreadable", not trouble, f"{trouble[:4]}")
        self.check("REG.every_cue_said_once_in_order", [s["cue"] for s in says] == self.cues, f"{len(says)} says")
        by_file = {f"{m}.rs": k[:2] for k, m in reg}
        wrong = [s for s in says if by_file[s["file"]] != s["cue"][:2]]
        self.check("REG.each_said_by_its_chapter", not wrong, f"{wrong[:3]}")
        counts = Counter(w["file"] for w in waits if w["call"] == "finish")
        self.check("REG.each_chapter_finishes", all(counts[f"{m}.rs"] == 1 for _, m in reg), "one kit.finish per chapter")
        run = subprocess.run([sys.executable, "-I", "-B", "docs/check_cues.py"], cwd=self.crate, capture_output=True, text=True)
        last = (run.stdout.strip().splitlines() or [""])[-1]
        self.check("REG.check_cues_passes", run.returncode == 0 and last == "OK", f"the crate's own checker, exit {run.returncode}, last line {last!r}; {' '.join(run.stderr.split())[:200]}")
        bad = []
        for h in sentence_hooks:
            if h["index"] is not None and not 0 <= h["index"] < len(sentences(self.script.get(h["cue"], ""))):
                bad.append(h)
        literal = [h for h in sentence_hooks if h["index"] is not None]
        self.check("REG.sentence_hooks_in_range", not bad, f"{len(literal)} literal hooks name a sentence their cue has; bad: {bad[:3]}")
        self.check("REG.sentence_hook_count", len(literal) == self.audit["sentence_hooks"]["literal"], f"{len(literal)} found, audit says {self.audit['sentence_hooks']['literal']}")
        computed = [(h["file"], h["line"], h["cue"]) for h in sentence_hooks if h["index"] is None]
        listed = [(r["file"], r["line"], r["cue"]) for r in self.audit["sentence_hooks"]["non_literal"]]
        self.check("REG.computed_sentence_hooks_listed", computed == listed, f"{computed}")
        elsewhere = sorted({(h["file"], h["line"], h["cue"]) for h in sentence_hooks if h["cue"] != h["current"]})
        allowed = sorted({(r["file"], r["line"], r["cue"]) for r in self.audit["sentence_hooks"]["cue_differs_from_current"]})
        self.check("REG.sentence_hooks_on_current_cue", elsewhere == allowed, f"hooks off the cue being said are only the listed helper: {elsewhere}")
        run = subprocess.run([sys.executable, "-I", "-B", str(self.crate / "oracle.py")], capture_output=True, text=True)
        self.check("REG.oracle_reproduces", run.returncode == 0 and run.stdout == (self.stage / "oracle04.tsv").read_text(), f"{len(run.stdout.splitlines())} oracle facts; Rust's facts are root's build check")

    def group_holds(self):
        m = self.map
        visual, over = m["visual_holds"], m["overrides"]
        self.check("HOLD.map_binds_script", m.get("scripts", {}).get(TOPIC) == self.script_sha)
        self.check("HOLD.counts", len(visual) == 15 and len(over) == 15 and m.get("hold_kinds") == {"prediction": 13, "reading": 2}, f"{len(visual)} holds, {len(over)} overrides, {m.get('hold_kinds')}")
        order = [self.cues.index(v["cue"]) for v in visual]
        self.check("HOLD.in_script_order", order == sorted(order))
        self.check("HOLD.overrides_pair_rows", [o["cue"] for o in over] == [v["cue"] for v in visual])
        self.check("HOLD.after_is_whole_cue", all(o["after"] == self.script.get(o["cue"]) for o in over), "every direction covers its whole cue")
        self.check("HOLD.role_and_video", all(o["role"] == "visual" and o["video"] == TOPIC and v["video"] == TOPIC for o, v in zip(over, visual)))
        # Counters, not sets: a second call on a line already listed must still count.
        census = Counter((h["file"], h["line"], h["cue"], h["kind"]) for h in self.holds)
        rows = Counter((v["path"].rsplit("/", 1)[1], v["line"], v["cue"], v["kind"]) for v in visual)
        self.check("HOLD.census_equals_map", census == rows, f"source has {sum(census.values())} hold calls; only in source {sorted(census - rows, key=str)[:4]}; only in map {sorted(rows - census, key=str)[:4]}")
        say_at = {s["cue"]: s for s in self.says}
        for v in visual:
            cue, name = v["cue"], v["path"].rsplit("/", 1)[1]
            cid = f"HOLD.{cue}"
            path = self.path_in_view(v["path"])
            data = path.read_bytes() if path.is_file() else b""
            self.check(f"{cid}.source_hash", sha(data) == v["sha256"] and v["evidence"] == {"path": v["path"], "sha256": v["sha256"]}, f"{name} {v['sha256'][:12]}")
            lines = data.decode("utf-8").split("\n")
            marker_ok = 0 < v["line"] <= len(lines) and lines[v["line"] - 1].strip() == v["marker"]
            kind_ok = (v["kind"] == "prediction" and re.search(r"\bpredict\(", v["marker"])) or (v["kind"] == "reading" and "kit.music_breath(" in v["marker"])
            self.check(f"{cid}.marker_line", bool(marker_ok and kind_ok), f"{name}:{v['line']} {v['marker'][:60]}")
            say = say_at.get(cue, {}).get("line")
            interval = "\n".join(lines[say - 1 : v["line"]]) if say else ""
            self.check(f"{cid}.interval_hash", bool(say) and sha(interval.encode("utf-8")) == v["interval_sha256"], f"say at {say}")
            at = self.cues.index(cue)
            self.check(f"{cid}.next_cue", at + 1 < len(self.cues) and self.cues[at + 1] == v["next_cue"], v["next_cue"])
            next_say = say_at.get(v["next_cue"])
            if v["kind"] == "prediction":
                same_chapter = bool(next_say) and next_say["file"] == name and v["line"] < next_say["line"]
                crossing = bool(next_say) and next_say["file"] != name
                self.check(f"{cid}.before_next_say", same_chapter or crossing, "before the answer's cue" if same_chapter else "last hold before a chapter join")
        self.check("HOLD.predict_helper_order", self.helper_order(), "draw::predict holds, marks, waits THINK, fades, in that order")
        self.check("HOLD.no_scene_mark", not self.marks, f"no scene emits a mark itself, so only draw::predict can extend a caption: {self.marks[:2]}")
        recorded = sorted((r["file"], r["line"]) for r in self.audit["other_waits"] + self.audit["chapter_finishes"])
        found = sorted((w["file"], w["line"]) for w in self.waits)
        self.check("HOLD.every_other_wait_classified", recorded == found, f"{len(found)} waits and finishes; only in source {sorted(set(found) - set(recorded))[:4]}; only in audit {sorted(set(recorded) - set(found))[:4]}")
        rows = []
        for r in self.audit["helpers"] + self.audit["consumers"]:
            p = self.path_in_view(r["path"])
            text = p.read_bytes()
            line_ok = text.decode("utf-8").split("\n")[r["line"] - 1].strip() == r["marker"]
            if sha(text) != r["sha256"] or not line_ok:
                rows.append(f"{r['path'].rsplit('/', 1)[1]}:{r['line']}")
        self.check("HOLD.helpers_and_consumers_unmoved", not rows, f"{len(self.audit['helpers'])} helper and {len(self.audit['consumers'])} consumer rows, hash and line; moved: {rows[:5]}")
        wrong = [h["path"] for h in m["shared_helpers"] if fsha(self.path_in_view(h["path"])) != h["sha256"]]
        self.check("HOLD.shared_helpers", not wrong, f"{len(m['shared_helpers'])} shared files; moved: {wrong}")
        retired = {(r["cue"], r["next_cue"]) for r in self.audit["removed_pairs"]}
        self.check("HOLD.stale_pairs_gone", len(retired) == 4 and not ({(v["cue"], v["next_cue"]) for v in visual} & retired), "oliver11's four retired pairs")
        self.reading_windows()

    def helper_order(self):
        toks = drop_tests(lex((self.chapters / "draw.rs").read_text(encoding="utf-8")))
        body = next(((a, b) for name, a, b in functions(toks) if name == "predict"), None)
        if not body:
            return False
        names = [c[1] for c in calls(toks[body[0] : body[1] + 1])]
        want = ["hold", "mark", "music_breath", "fade_out_all"]
        at = [names.index(w) if w in names else -1 for w in want]
        return -1 not in at and at == sorted(at)

    def reading_windows(self):
        toks = self.toks["unreachable.rs"]
        row = next(r for r in self.audit["reading_holds"] if r["cue"] == "11_20")
        at = const_value_at(toks, "THEOREM")
        rule = [t.value for t in toks[at : match_close(toks, at)] if t.kind == "str"]
        self.check("HOLD.11_20_rule_lines", rule == row["rule_lines"] and sum(map(len, rule)) == row["rule_chars"], f"{sum(map(len, rule))} characters from the source's own literals")
        # One verdict is built with format!, so its literal is a template with {} holes.
        templates = [re.compile(".+?".join(map(re.escape, t.value.split("{}")))) for t in toks if t.kind == "str"]
        missing = [v for v in row["row_verdicts"] if not any(p.fullmatch(v) for p in templates)]
        self.check("HOLD.11_20_row_verdicts", not missing and sum(map(len, row["row_verdicts"])) == row["row_chars"], f"{row['row_chars']} characters; missing: {missing}")
        rate = float(toks[const_value_at(toks, "READING_RATE")].value.replace("_f64", ""))
        # From the source's rule text, so a shortened theorem cannot keep the audit's old window.
        window = (sum(map(len, rule)) + sum(map(len, row["row_verdicts"]))) / rate
        self.check("HOLD.11_20_window", abs(window - row["floor_composition"]["window_s"]) < 1e-3, f"{window:.4f} s after voice end plus breath, at {rate} characters a second")
        lines = self.lines["doing_nothing.rs"]
        hold = next(r for r in self.audit["reading_holds"] if r["cue"] == "10_4")
        self.check("HOLD.10_4_conditional", hold["conditional"] is True and "if read_for < FINE_PRINT_S {" in lines[hold["line_evidence"]["condition"] - 1], "the window is a property of the take, and can be zero")

    def group_phrases(self):
        rows = self.anchors["anchors"]
        self.check("PHR.binds_script", self.anchors["script_sha256"] == self.script_sha and self.template["script_sha256"] == self.script_sha)
        self.check("PHR.none_measured", self.anchors["counts"]["measured"] == 0 and all(r["measured_seconds"] is None for r in rows), f"{len(rows)} anchors, no invented seconds")
        sites = Counter((s["file"], s["line"], s["call"], r["cue"], r["phrase"]) for r in rows for s in r["hook_sites"] if s["call"] in PHRASE_HELPERS)
        census = Counter((h["file"], h["line"], h["call"], h["cue"], h["phrase"]) for h in self.hooks if h["call"] != "phrase_marks")
        self.check("PHR.census_equals_plan", sites == census, f"{sum(census.values())} hook calls; only in source {sorted(census - sites)[:3]}; only in plan {sorted(sites - census)[:3]}")
        direct = {(h["file"], h["cue"]) for h in self.hooks if h["call"] == "phrase_marks"}
        self.check("PHR.direct_mark_readers", direct == {("undoing_growth.rs", "04_2")}, f"{sorted(direct)}")
        wrong = [h for h in self.hooks if h["call"] != "phrase_marks" and h["cue"] != h["current"]]
        self.check("PHR.every_hook_on_the_cue_being_said", not wrong, f"{[(h['file'], h['line'], h['cue'], h['current']) for h in wrong][:3]}")
        phrase_of = {}
        for h in self.hooks:
            phrase_of.setdefault((h["file"], h["line"]), []).append(h["phrase"])
        say_line = {s["cue"]: s["line"] for s in self.says}
        for r in rows:
            cue, phrase = r["cue"], r["phrase"]
            cid = f"PHR.{cue}@{r['byte']}"
            text = self.script.get(cue, "")
            raw, want = text.encode("utf-8"), phrase.encode("utf-8")
            at = raw.find(want)
            starts = {len(text[: m.start()].encode("utf-8")) for m in MARK_WORD.finditer(text)}
            good = raw.count(want) == 1 and at == r["byte"] and r["byte_end"] == at + len(want) and at in starts
            self.check(f"{cid}.exact_byte", good, f"{phrase[:40]!r}; a word start the mark producer can hit")
            opens = [0] + [m.start() + 2 for m in re.finditer(r"[.?!] ", text)]
            index = max(i for i, o in enumerate(opens) if o <= at) if at >= 0 else -1
            self.check(f"{cid}.sentence_index", index == r["sentence_index"], f"sentence {index}")
            good = r["cue_text_sha256"] == sha(raw) and fsha(self.path_in_view(r["source_path"])) == r["source_sha256"]
            self.check(f"{cid}.hashes", good, r["source_path"].rsplit("/", 1)[1])
            lines = self.lines[r["source_path"].rsplit("/", 1)[1]]
            miss = []
            for s in r["hook_sites"]:
                seen = phrase_of.get((s["file"], s["line"]), [])
                if s["call"] in PHRASE_HELPERS and phrase not in seen:
                    miss.append((s["line"], seen))
                # climb_onset is a helper defined above the scene; its caller is the wait_until row.
                if s["call"] != "climb_onset" and s["line"] <= say_line[cue]:
                    miss.append((s["line"], "before the say"))
                if not lines[s["line"] - 1].strip():
                    miss.append((s["line"], "blank"))
            self.check(f"{cid}.destination", not miss, f"{len(r['hook_sites'])} source site(s) wait on this phrase of {cue}; {miss[:2]}")
            mark = [x for x in self.template["cues"].get(cue, {}).get("marks", []) if x["byte"] == r["byte"]]
            self.check(f"{cid}.template", len(mark) == 1 and mark[0]["phrase"] == phrase and mark[0]["seconds"] is None)
        toks = self.toks["undoing_growth.rs"]
        consts, _ = str_constants(toks)
        row = next((r for r in rows if r["cue"] == "04_2"), None)
        body = next(((a, b) for name, a, b in functions(toks) if name == "climb_onset"), None)
        uses = bool(body) and any(t.value == "CLIMB_PHRASE" for t in toks[body[0] : body[1]])
        self.check("PHR.04_2_constant", bool(row) and consts.get("CLIMB_PHRASE") == row["phrase"] and uses, "climb_onset reads the anchor's constant, directly from phrase_marks")
        tcues = self.template["cues"]
        self.check("PHR.template_covers_all_cues", list(tcues) == self.cues, "the loader needs an entry for every cue")
        self.check("PHR.template_text_hashes", all(tcues.get(c, {}).get("text_sha256") == sha(self.script[c].encode()) for c in self.cues))
        self.check("PHR.template_mark_count", sum(len(v["marks"]) for v in tcues.values()) == len(rows))
        self.check("PHR.template_cannot_load", check_marks(self.template, self.script, None, None) != [], "an unfilled template is refused by the loader's own rules")
        filled = fill_template(self.template, self.script)
        problems = check_marks(filled, self.script, {c: filled["cues"].get(c, {}).get("wav_sha256") for c in self.cues}, rows)
        self.check("PHR.filled_shape_would_load", problems == [], f"the template's shape, with stand-in values, passes the loader's rules and the anchor gate: {problems[:2]}")
        source = (self.repo / "crates/explainer_kit/src/narration.rs").read_text(encoding="utf-8")
        self.check("PHR.destination_file", 'dir.join("phrase-timings.json")' in source, "<narration dir>/phrase-timings.json, read only when the file exists")
        share = self.anchors["sentence_share_hooks"]["rows"]
        wrong = [r for r in share if not 0 <= r["sentence_index"] < len(sentences(self.script[r["cue"]]))]
        self.check("PHR.share_hooks_in_range", not wrong, f"{len(share)} sentence-share hooks; {wrong[:2]}")

    def group_consumers(self):
        coherent = self.load_tool("coherent_narration")
        if self.mode == "stage":
            # As root's binding validation did: the map is looked for under the plan, not canonical.
            coherent.REPO = self.plan.resolve()
        script, units = coherent.read_units(self.script_path, self.units_path, require_directions=True)
        self.check("CON.read_units", len(script) == 80 and len(units) == 80, "real coherent_narration.read_units, directions required")
        plan = json.loads(self.units_path.read_bytes())
        binding = coherent.direction_contract(self.script_path, plan)
        resolved = binding["resolved"]
        ends = [r for r in resolved if r["end_word_index"] != len(coherent.direction_words(script[r["cue"]])) - 1]
        self.check("CON.direction_contract", len(resolved) == 15 and not ends, f"real direction_contract: {len(resolved)} directions, each at its cue's last word")
        manifest = {"direction_dispositions": self.dispositions["direction_dispositions"]}
        self.check("CON.dispositions_bind_map", self.dispositions["map_sha256"] == fsha(self.map_path))
        self.check("CON.dispositions_all_native", all(r["kind"] == "native_visual_hold" for r in manifest["direction_dispositions"]) and [r["cue"] for r in manifest["direction_dispositions"]] == [r["cue"] for r in resolved], "no inserted silence proposed")
        here = Path.cwd()
        try:
            os.chdir(self.repo)
            directions = coherent.review_directions(binding, units, manifest)
        finally:
            os.chdir(here)
        self.check("CON.review_directions", len(directions["dispositions"]) == 15, "real review_directions, evidence hashes included; it reads paths from the cwd, so assemble from the repo root")
        lines = {cue: [{"sentence": s, "gap_after_samples": 0} for s in coherent.sentences(text)] for cue, text in script.items()}
        coherent.check_dispositions(directions, lines)
        self.check("CON.check_dispositions", True, "real check_dispositions: every hold ends its cue, against the sentence rows an assembly would write")
        target = next(r["cue"] for r in resolved if len(lines[r["cue"]]) > 1)
        refused = False
        try:
            coherent.check_dispositions(directions, {**lines, target: lines[target][:-1]})
        except ValueError:
            refused = True
        self.check("CON.check_dispositions_control", refused, f"the same call refuses {target} when its last sentence is gone")
        wrong = []
        for u in units:
            text = coherent.request_text(u, script)
            data = (self.repo / LANE / "narration-request-final80-01" / f"{u['line_ids'][0]}.txt").read_bytes()
            if text.encode("utf-8") != data or len(u["line_ids"]) != 1:
                wrong.append(u["id"])
        self.check("CON.request_text", not wrong, f"what unit_spans will demand of an Oliver receipt equals the request file, for {80 - len(wrong)} of 80; different: {wrong[:4]}")

        delivery = self.load_tool("narration_delivery")
        declared = Counter((h.get("cue"), h.get("next_cue")) for h in self.map["visual_holds"] if h.get("video") == TOPIC and h.get("kind") == "prediction")
        self.check("CON.declared_pairs", len(declared) == 13 and set(declared.values()) == {1}, "narration_delivery's filter finds 13 prediction pairs")
        # A stand-in timeline with a predict mark after EVERY cue: a mark alone must not declare a hold.
        fps, frame, rows, predicts, outputs = 30, 0, [], [], {}
        for cue in self.cues:
            frame += 30
            rows.append(["line", cue, str(frame)])
            outputs[cue] = {"frames": 24000, "sample_rate": 24000}
            frame += 30
            predicts.append((frame + 15) / fps)
            frame += 60
        held = []
        for i, line in enumerate(rows):
            following = rows[i + 1] if i + 1 < len(rows) else None
            if delivery.prediction_end(line, following, fps, predicts, outputs, declared, 0.0) is not None:
                held.append(line[1])
        want = [v["cue"] for v in self.map["visual_holds"] if v["kind"] == "prediction"]
        self.check("CON.prediction_end", held == want, f"real prediction_end with a mark after all 80 cues extends the 13 declared predictions, and neither reading hold; extended {len(held)}")
        if self.mode == "stage":
            refused = None
            try:
                delivery.verify(argparse.Namespace(topic=self.stage_crate))
            except (ValueError, AttributeError) as error:
                refused = str(error)
            self.check("CON.delivery_refuses_stage_path", refused == "topic must be in this repository", f"{refused}: a voiced consumer cannot run on the staged path, so integration comes first")
        else:
            self.check("CON.delivery_topic_path", self.canon_crate.resolve().parent == Path(delivery.REPO) / "explainer_corpus", "the installed crate is where narration_delivery looks")
        self.director()

    def director(self):
        try:
            director = self.load_tool("narration_director")
        except ImportError as error:
            self.warning("CON.director_not_run", f"narration_director needs numpy here: {error}")
            return
        video = {"id": TOPIC, "script": str(self.script_path)}
        as_written = json.loads(json.dumps(self.map))
        for hold in as_written["visual_holds"]:
            hold["evidence"]["path"] = str(self.path_in_view(hold["evidence"]["path"]))
        refused = None
        try:
            director.validate_map({"videos": [video]}, as_written)
        except (KeyError, ValueError) as error:
            refused = repr(error)
        if refused:
            self.warning("CON.director_map_shape", f"narration_director.validate_map on map13 as written: {refused}. Oliver maps carry no policy or range_s (oliver12 is refused the same way), so this is not a regression. A director audit needs a derived map.")
        derived = json.loads(json.dumps(as_written))
        derived["policy"] = director.POLICY
        for row in derived["overrides"]:
            row["range_s"] = [0.0, 30.0]
        director.validate_map({"videos": [video]}, derived)
        self.check("CON.director_holds_valid", True, "with policy and range_s added in memory, real validate_map accepts all 15 holds: evidence hashes and adjacent cues")

    def group_stale(self):
        rows = []

        def walk(value, trail):
            if isinstance(value, dict):
                if isinstance(value.get("path"), str) and isinstance(value.get("sha256"), str):
                    rows.append((trail, value["path"], value["sha256"]))
                for k, v in value.items():
                    walk(v, f"{trail}/{k}")
            elif isinstance(value, list):
                for i, v in enumerate(value):
                    walk(v, f"{trail}[{i}]")

        for name, data in (("map", self.map), ("audit", self.audit), ("dispositions", self.dispositions), ("units", self.units)):
            walk(data, name)
        bad = []
        for trail, path, want in rows:
            if self.mode == "canonical" and path.endswith(".rs.orig"):
                continue
            p = self.map_path if path.endswith(MAP_NAME) else self.path_in_view(path)
            if not p.is_file() or fsha(p) != want:
                bad.append((trail, path))
        self.check("STALE.every_named_path_and_hash", not bad, f"{len(rows)} path and hash pairs in the plan resolve in the tree checked; bad: {bad[:4]}")
        plan_files = [p for p in self.plan.rglob("*") if p.is_file()]
        home = ("/" + "Users/").encode()
        self.check("STALE.no_absolute_homes", not [p.name for p in plan_files if home in p.read_bytes()], f"{len(plan_files)} plan files")
        text = {p.relative_to(self.plan).as_posix(): p.read_text(encoding="utf-8") for p in plan_files}
        hits = {k: [w for w in ("oliver12", "combined-repair-stage05", "combined-repair-stage04", "oliver11") if w in v] for k, v in text.items()}
        loud = {k: v for k, v in hits.items() if set(v) - {"oliver11"}}
        self.check("STALE.no_older_stage_names", not loud, f"{loud}")
        named = {k for k, v in hits.items() if v}
        self.check("STALE.oliver11_only_as_history", named <= {f"{REL.as_posix()}/docs/{MAP_NAME}", "inputs.sha256"} and "replaces oliver11's" in self.map["scope"].lower(), f"named only in the map's scope note and the binder's input list: {sorted(named)}")
        canon_units = json.loads((self.canon_crate / "narration_units.json").read_bytes())
        crate_files = [p for p in self.crate.rglob("*") if p.is_file() and "target" not in p.relative_to(self.crate).parts and p.suffix in (".rs", ".py", ".toml", ".md", ".tsv")]
        naming = sorted(p.name for p in crate_files if re.search(rb"direction-map-oliver|oliver1[0-9]", p.read_bytes()))
        self.check("STALE.no_source_names_a_map", not naming, f"no source, doc or test in the crate names a map: {naming}")
        if self.mode == "stage":
            canonical_map = self.canon_crate / "docs/narration-direction-map-oliver12.json"
            now = canon_units["direction_contract"]["map"]["path"].rsplit("/", 1)[1]
            self.check("STALE.canonical_still_binds_map12", now == canonical_map.name and canon_units["direction_contract"]["map"]["sha256"] == fsha(canonical_map), "canonical is self-consistent until integration")
            old = json.loads(canonical_map.read_bytes())
            wrong = [h["cue"] for h in old["visual_holds"] if fsha(self.repo / h["path"]) != h["sha256"]]
            self.check("STALE.canonical_map12_matches_canonical", not wrong, f"different: {wrong}")
            refuse = [v["cue"] for v in self.map["visual_holds"] if fsha(self.repo / v["path"]) != v["sha256"]]
            self.info("STALE.holds_canonical_would_refuse_today", f"{len(refuse)} of 15 until integration, by design: {refuse}")
            self.check("STALE.map13_not_in_canonical", not (self.canon_crate / "docs" / MAP_NAME).exists(), "nothing half-installed")
        else:
            self.check("STALE.canonical_binds_map13", canon_units["direction_contract"]["map"]["path"].endswith(MAP_NAME))
            wrong = [v["cue"] for v in self.map["visual_holds"] if fsha(self.repo / v["path"]) != v["sha256"]]
            self.check("STALE.all_holds_match_canonical", not wrong, f"different: {wrong}")

    def run(self):
        try:
            self.load()
        except Exception as error:  # a broken plan must report, not stack-trace
            self.check("LOAD", False, f"{type(error).__name__}: {error}")
            return self
        for group in (self.group_pins, self.group_inputs, self.group_assembly, self.group_text, self.group_registry, self.group_holds, self.group_phrases, self.group_consumers, self.group_stale):
            try:
                group()
            except Exception as error:  # the other groups still report
                self.check(f"{group.__name__}.completed", False, f"{type(error).__name__}: {error}")
        return self


def fill_template(template, script):
    """A stand-in with the shape of a measured file. Its seconds are ascending counters, not timings."""
    cues = {}
    for cue, entry in template["cues"].items():
        marks = [{"byte": m["byte"], "seconds": 0.001 * (i + 1)} for i, m in enumerate(entry["marks"])]
        cues[cue] = {"wav_sha256": sha(cue.encode()), "text_sha256": sha(script[cue].encode()), "marks": marks}
    return {"cues": cues}


def check_marks(timing, script, wavs, anchors, lengths=None):
    """The loader's rules (narration.rs, the phrase_marks loop), then every anchor's mark.

    wavs: cue -> the assembled WAV's sha256. lengths: cue -> its seconds, or None to skip that rule.
    A mark the kit accepts can still miss an anchor, and then the chapter falls back to a
    character share without saying so, so the anchor gate is a separate rule here.
    """
    problems = []
    cues = timing.get("cues")
    if not isinstance(cues, dict):
        return ["no cues"]
    for cue, text in script.items():
        entry = cues.get(cue)
        if entry is None:
            problems.append(f"{cue}: no entry; the loader needs a complete cue map")
            continue
        if wavs is None or entry.get("wav_sha256") is None or entry.get("wav_sha256") != wavs.get(cue):
            problems.append(f"{cue}: WAV differs")
        if entry.get("text_sha256") != sha(text.encode("utf-8")):
            problems.append(f"{cue}: text differs")
        marks = entry.get("marks")
        if not isinstance(marks, list):
            problems.append(f"{cue}: no marks")
            continue
        previous, previous_byte = 0.0, None
        raw = text.encode("utf-8")
        for row in marks:
            at, seconds = row.get("byte"), row.get("seconds")
            if type(at) is not int or at < 0:
                problems.append(f"{cue}: bad offset")
                break
            if not isinstance(seconds, (int, float)) or isinstance(seconds, bool):
                problems.append(f"{cue}: bad time")
                break
            boundary = at < len(raw) and (raw[at] & 0xC0) != 0x80
            limit = lengths.get(cue) if lengths else None
            if not boundary or seconds != seconds or seconds < previous or (limit is not None and seconds >= limit) or (previous_byte is not None and at <= previous_byte):
                problems.append(f"{cue}: invalid mark at byte {at}")
                break
            previous, previous_byte = seconds, at
    for row in anchors or []:
        entry = cues.get(row["cue"]) or {}
        have = {m.get("byte") for m in entry.get("marks") or []}
        if row["byte"] not in have:
            problems.append(f"{row['cue']}: no mark at byte {row['byte']} for {row['phrase']!r}; the chapter would fall back to a character share")
    return problems


def gate_phrase_timings(run, path, narration=None):
    timing = json.loads(Path(path).read_bytes())
    lengths = None
    if narration:
        outputs = json.loads((Path(narration) / "assembly.json").read_bytes())["outputs"]
        wavs = {c: o["sha256"] for c, o in outputs.items()}
        lengths = {c: o["frames"] / o["sample_rate"] for c, o in outputs.items()}
    else:
        wavs = {c: (timing.get("cues", {}).get(c) or {}).get("wav_sha256") for c in run.script}
    problems = check_marks(timing, run.script, wavs, run.anchors["anchors"], lengths)
    bound = "WAV hashes and lengths from assembly.json" if narration else "WAV hashes unbound: pass --narration"
    run.check("GATE.phrase_timings", not problems, f"{len(run.anchors['anchors'])} anchors; {bound}; {len(problems)} problem(s): {problems[:6]}")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--mode", choices=("stage", "canonical"), default="stage")
    parser.add_argument("--no-pins", action="store_true", help="skip the frozen hash pins; the semantic checks still run")
    parser.add_argument("--quiet", action="store_true", help="print only FAIL and WARN, and the total")
    parser.add_argument("--phrase-timings", help="also gate this measured phrase-timings.json")
    parser.add_argument("--narration", help="with --phrase-timings: the narration folder, to bind its WAV hashes and lengths")
    args = parser.parse_args(argv)
    run = Run(args.repo, args.mode, pins=not args.no_pins, quiet=args.quiet).run()
    if args.phrase_timings:
        gate_phrase_timings(run, args.phrase_timings, args.narration)
    print(f"{run.ok} ok, {len(run.fail)} FAIL, {len(run.warn)} WARN  (mode {args.mode}; source and plan text only: not compiled, rendered, heard or watched)")
    return 1 if run.fail else 0


if __name__ == "__main__":
    sys.exit(main())
