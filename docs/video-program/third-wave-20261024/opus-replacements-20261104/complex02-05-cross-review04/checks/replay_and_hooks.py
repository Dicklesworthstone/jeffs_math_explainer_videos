#!/usr/bin/env python3
"""Cross-review checks for complex02-05-repair03. Run from the repo root:

    python3 -I -B replay_and_hooks.py

Reads only, except a temp folder it makes under the system temp directory and leaves for the OS.

1. The patch: its hash, a zero-fuzz replay on canonical and on the candidate, and the result
   against the staged copies. A control proves a moved context line refuses.
2. Every timing hook of chapters 02 to 05, read from the source here, not from the repair's TSV
   or mirror tests: phrase found exactly once in the cue the source is saying, on a word start
   of the phrase-mark producer's regex, sentence in range, and each cue's hooks in the voice's
   order. Then kept, added and removed against canonical, compared with bindings/phrase-hooks.tsv.
   A control proves a wrong phrase, a repeated phrase and a mid-word phrase are all caught.

Source text only. It cannot see types, and a phrase passing here says nothing about any take.
"""
import csv
import hashlib
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
O = HERE.parents[1]
REPAIR = O / "complex02-05-repair03"
CRATE = Path("explainer_corpus/complex_analysis_visual")
CANDIDATE = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-picture-stage01/candidate")
PATCH = REPAIR / "complex02-05-repair03.patch"
PATCH_SHA = "49205a2a577c03b273935609701ddf04e602278245c6fd438fb08331f98b96d2"
FILES = ("turning", "two_planes", "reading_color", "gallery")
STAGED = {f: REPAIR / CRATE / "src/chapters" / f"{f}.rs" for f in FILES}
TOKEN = re.compile(r"[A-Za-z]+(?:'[A-Za-z]+)?|[0-9]+")  # elevenlabs-golay-20261028/phrase_marks.py
SENTENCE_END = re.compile(r"[.?!] ")  # the rule of draw::sentence_offsets
SAY = re.compile(r'let line = say\(&kit, stage, "(\d\d_\d+)"\)\?;')
PHRASE = re.compile(r'\bline\s*\.\s*(wait_for|until|at)\s*\(\s*(?:stage\s*,\s*)?"([^"]*)"', re.S)
NUMBER = re.compile(r"\bline\s*\.\s*(wait_sentence|sentence)\s*\(\s*(?:stage\s*,\s*)?(\d+)\s*\)", re.S)
failures = []


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fail(text):
    failures.append(text)
    print("FAIL", text)


def replay(tree, label):
    tmp = Path(tempfile.mkdtemp(prefix=f"cross04-{label}-"))
    for name in FILES:
        target = tmp / CRATE / "src/chapters" / f"{name}.rs"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(tree / CRATE / "src/chapters" / f"{name}.rs", target)
    done = subprocess.run(["git", "apply", "--whitespace=nowarn", "-C99", str(PATCH.resolve())], cwd=tmp, capture_output=True, text=True)
    if done.returncode:
        fail(f"{label}: patch refused: {done.stderr[:300]}")
        return tmp
    for name in FILES:
        got, want = (tmp / CRATE / "src/chapters" / f"{name}.rs").read_bytes(), STAGED[name].read_bytes()
        if got != want:
            fail(f"{label}: {name}.rs after the patch differs from the staged copy")
    print(f"ok   replay on {label}: applied with full context, all 4 files equal the staged copies")
    return tmp


def replay_control():
    tmp = Path(tempfile.mkdtemp(prefix="cross04-control-"))
    for name in FILES:
        target = tmp / CRATE / "src/chapters" / f"{name}.rs"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(CRATE / "src/chapters" / f"{name}.rs", target)
    victim = tmp / CRATE / "src/chapters/gallery.rs"
    victim.write_text(victim.read_text().replace('pub const POLE2: &str = "pole of order 2";', 'pub const POLE2: &str = "pole of order two";').replace('let (_, cubic) = panels[0];', 'let (_, cube) = panels[0];'))
    done = subprocess.run(["git", "apply", "--whitespace=nowarn", "-C99", str(PATCH.resolve())], cwd=tmp, capture_output=True, text=True)
    if done.returncode == 0:
        fail("control: a moved context line did not refuse the patch")
    else:
        print("ok   control: a changed context line refuses the patch")


def script_of(path):
    rows = {}
    for row in Path(path).read_text(encoding="utf-8").split("\n"):
        if row and not row.startswith("#"):
            cue, text = row.split("\t")
            rows[cue] = text
    return rows


def sentence_starts(text):
    return [0] + [m.end() for m in SENTENCE_END.finditer(text)]


def hooks_in(path):
    """(cue, order in source, kind, value, source line) for each literal hook on the cue being said."""
    source = Path(path).read_text(encoding="utf-8")
    code = source[: source.index("#[cfg(test)]\nmod tests")]
    says = list(SAY.finditer(code))
    found = []
    for k, say in enumerate(says):
        end = says[k + 1].start() if k + 1 < len(says) else len(code)
        for pattern, kind in ((PHRASE, "phrase"), (NUMBER, "sentence")):
            for m in pattern.finditer(code, say.start(), end):
                value = m.group(2) if kind == "phrase" else int(m.group(2))
                found.append({"cue": say.group(1), "at": m.start(), "call": m.group(1), "kind": kind, "value": value, "line": code.count("\n", 0, m.start()) + 1})
    return sorted(found, key=lambda h: h["at"])


def judge(hook, text):
    """What is wrong with this hook against this cue's words, or None. Also sets its byte."""
    if hook["kind"] == "sentence":
        starts = sentence_starts(text)
        if hook["value"] >= len(starts):
            return f"sentence {hook['value']} out of range"
        hook["byte"] = starts[hook["value"]]
        return None
    n = text.count(hook["value"])
    if n != 1:
        return f"phrase {hook['value']!r} occurs {n} times"
    byte = text.index(hook["value"])
    hook["byte"] = byte
    if byte not in {m.start() for m in TOKEN.finditer(text)}:
        return f"phrase {hook['value']!r} does not start on a word"
    tail = byte + len(hook["value"])
    if tail < len(text) and text[tail].isalnum():
        return f"phrase {hook['value']!r} ends inside a word"
    return None


def check_tree():
    script = script_of(CRATE / "script.tsv")
    now, then = [], []
    for name in FILES:
        for hook in hooks_in(STAGED[name]):
            hook["file"] = f"{name}.rs"
            now.append(hook)
        for hook in hooks_in(CRATE / "src/chapters" / f"{name}.rs"):
            hook["file"] = f"{name}.rs"
            then.append(hook)
    for hook in now:
        bad = judge(hook, script[hook["cue"]])
        if bad:
            fail(f"{hook['cue']} {hook['file']}:{hook['line']} {bad}")
    print(f"ok   {len(now)} literal hooks judged on the staged source")
    by_cue = {}
    for hook in now:
        by_cue.setdefault(hook["cue"], []).append(hook)
    for cue, hooks in by_cue.items():
        for a, b in zip(hooks, hooks[1:]):
            if a.get("byte", 0) > b.get("byte", 0):
                fail(f"{cue}: {a['value']!r} (byte {a['byte']}) runs before {b['value']!r} (byte {b['byte']}) in the source but after it in the voice")
    print(f"ok   order within each of {len(by_cue)} cues checked")
    key = lambda h: (h["cue"], h["file"], h["call"], str(h["value"]))
    new, old = Counter(map(key, now)), Counter(map(key, then))
    added, removed, kept = new - old, old - new, new & old
    print(f"     against canonical: kept {sum(kept.values())}, added {sum(added.values())}, removed {sum(removed.values())}")
    with open(REPAIR / "bindings/phrase-hooks.tsv", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    tsv = {s: Counter((r["cue"], r["file"], r["call"], r["hook"]) for r in rows if r["status"] == s) for s in ("kept", "added", "removed")}
    for name, mine in (("kept", kept), ("added", added), ("removed", removed)):
        if tsv[name] != mine:
            fail(f"phrase-hooks.tsv {name} differs: {sorted((tsv[name] - mine).elements())} / {sorted((mine - tsv[name]).elements())}")
    print("ok   phrase-hooks.tsv matches this independent read" if not failures else "     (TSV compared)")
    for r in rows:
        if r["byte_in_cue"]:
            if script[r["cue"]].find(r["hook"]) != int(r["byte_in_cue"]):
                fail(f"TSV byte for {r['cue']} {r['hook']!r}")
    print("\ncue\tfile:line\tcall\thook\tbyte\tcount_in_cue\tspoken_from_there")
    for hook in now:
        if key(hook) in added:
            text = script[hook["cue"]]
            count = text.count(hook["value"]) if hook["kind"] == "phrase" else 1
            print(f"{hook['cue']}\t{hook['file']}:{hook['line']}\t{hook['call']}\t{hook['value']}\t{hook['byte']}\t{count}\t{text[hook['byte']:hook['byte'] + 42]}")
    print()


def hook_controls():
    for hook, text, want in (
        ({"kind": "phrase", "value": "angles add"}, "Lengths multiply.", "occurs 0"),
        ({"kind": "phrase", "value": "two"}, "two and two", "occurs 2"),
        ({"kind": "phrase", "value": "ngle"}, "one angle", "does not start"),
        ({"kind": "phrase", "value": "ang"}, "one angle", "ends inside"),
        ({"kind": "sentence", "value": 3}, "One. Two.", "out of range"),
    ):
        got = judge(hook, text)
        if not got or want not in got:
            fail(f"control {hook}: {got}")
    print("ok   controls: missing, repeated, mid-word, cut-short and out-of-range hooks are all caught")


def main():
    if sha(PATCH.read_bytes()) != PATCH_SHA:
        fail("patch hash")
    else:
        print(f"ok   patch sha256 {PATCH_SHA}")
    replay(Path("."), "canonical")
    replay(CANDIDATE, "candidate")
    replay_control()
    hook_controls()
    check_tree()
    print(f"{len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
