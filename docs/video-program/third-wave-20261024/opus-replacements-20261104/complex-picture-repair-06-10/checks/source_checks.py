#!/usr/bin/env python3
"""Static checks of the staged chapter 06-10 repair, since nothing here is compiled.

Run from the repo root, with python3 -I -B. Reads the staged sources, the frozen script
and the canonical sources. Writes nothing. Prints each finding; exit 1 if any.

1. Every phrase a cue waits on is in that cue's script text; a missing one fails at run time.
2. Every sentence number a cue waits on exists in that cue.
3. A cue's waits run in script order, so no wait can stand after the moment it names.
4. Every new constant is used by the chapter's code, and every new string constant is on a
   supplement list.
5. Every new picture object is faded out or cleared after it comes in, or named here as held.
6. Each list of TEXT and TEX strings stays free of duplicates (the crate has a test for this;
   this mirrors it, with the staged constants).
7. The script itself is byte-identical to canonical.
"""
import hashlib
import os
import re
import sys

R = (
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
    "complex-picture-repair-06-10/explainer_corpus/complex_analysis_visual/"
)
C = "explainer_corpus/complex_analysis_visual/"
CHAPTERS = ["one_derivative", "one_point", "loop_sums", "boundary", "rigidity"]
problems = []


def read(path):
    return open(path, encoding="utf-8").read()


def problem(s):
    problems.append(s)
    print("PROBLEM:", s)


script = {}
for row in read(C + "script.tsv").splitlines():
    if "\t" in row:
        cue, text = row.split("\t", 1)
        script[cue] = text


def sentence_starts(text):
    # sentences end at . ? ! followed by a space, as the script's text is plain words
    starts = [0]
    for m in re.finditer(r"[.?!]\s+", text):
        starts.append(m.end())
    return starts


waits = 0
for ch in CHAPTERS:
    src = read(R + "src/chapters/" + ch + ".rs")
    body = src.split("#[cfg(test)]\nmod tests")[0]
    pieces = re.split(r'say\(&kit, stage, "(\d\d_\d+)"\)\?;', body)
    for k in range(1, len(pieces), 2):
        cue, code = pieces[k], pieces[k + 1]
        text = script.get(cue)
        if text is None:
            problem(f"{cue}: not in the script")
            continue
        starts = sentence_starts(text)
        last = -1
        for m in re.finditer(
            r'line\s*\.(wait_for|until|wait_sentence|at)\(\s*(?:stage,\s*)?(?:"([^"]+)"|(\d+))', code
        ):
            kind, phrase, number = m.groups()
            waits += 1
            if number is not None:
                j = int(number)
                if j >= len(starts):
                    problem(f"{cue}: sentence {j} of {len(starts)}")
                    continue
                at = starts[j]
            else:
                if phrase not in text:
                    problem(f"{cue}: phrase not in the script: {phrase!r}")
                    continue
                at = text.index(phrase)
                if text.count(phrase) != 1:
                    problem(f"{cue}: phrase occurs {text.count(phrase)} times: {phrase!r}")
            if kind in ("wait_for", "wait_sentence"):
                if at < last:
                    problem(f"{cue}: a wait stands before an earlier one: {m.group(0)!r}")
                last = at
print(f"waits checked: {waits}")

# 4. new constants
mod_rs = read(R + "src/chapters/mod.rs")
for ch in CHAPTERS:
    staged = read(R + "src/chapters/" + ch + ".rs")
    old = read(C + "src/chapters/" + ch + ".rs")
    old_names = set(re.findall(r"pub const (\w+)", old))
    for name, ty in re.findall(r"pub const (\w+): ([^=]+)=", staged):
        if name in old_names:
            continue
        uses = len(re.findall(r"\b" + name + r"\b", staged))
        if uses < 2:
            problem(f"{ch}: {name} is never used")
        # the split's two terms only colour SPLIT_TEX, and are never typeset alone
        if "&str" in ty and name != "SPLIT_TERMS" and f"{ch}::{name}" not in mod_rs:
            problem(f"{ch}: {name} is on no supplement list")
for ch, name in re.findall(r"^\s+(\w+)::(\w+)(?:\[\d+\])?,$", mod_rs, re.M):
    path = R + "src/chapters/" + ch + ".rs"
    if ch in CHAPTERS and not re.search(r"pub const " + name + r"\b", read(path)):
        problem(f"mod.rs names {ch}::{name}, which the staged chapter does not define")

# 5. new picture objects: declared, then faded in, then cleared
HELD = {
    # (chapter, name): why it needs no clearing of its own
    ("one_derivative", "fan_in"): "a list; fan_all holds and clears its members",
    ("one_derivative", "fan_out"): "a list; fan_all holds and clears its members",
    ("one_derivative", "fan_all"): "cleared as fan_gone",
    ("one_derivative", "fan_gone"): "the clearing list",
    ("one_point", "origin"): "cleared at 07_5 by gone.extend(origin)",
    ("boundary", "recipe_shown"): "a list of objects cleared under their own names",
    ("boundary", "small_ring"): "a list of objects cleared under their own names",
    ("boundary", "shares"): "moved into flash, cleared at 09_6",
    ("boundary", "lit"): "faded out on the next line",
    ("rigidity", "first"): "a list of objects cleared under their own names",
    ("rigidity", "size"): "removed after the climb",
    ("rigidity", "size_says"): "removed after the climb",
    ("one_derivative", "plus_i"): "removed with the live values",
    ("one_derivative", "dir"): "faded out at the end of its pass",
    ("one_derivative", "live"): "removed after the three approaches",
    ("one_derivative", "ts"): "value trackers, not pictures",
    ("one_derivative", "equals"): "cleared as a member of numbers",
    ("boundary", "pair_up"): "pushed onto vp_frame, cleared at 09_6",
}
BUILD = r"(?:kit\.tex|kit\.text|kit\.text_left|carded|dot|stage\s*\.arena_mut\(\)|\w+\.path|\w+\.circle|\[\s*kit|\[\s*stage)"
for ch in CHAPTERS:
    staged = read(R + "src/chapters/" + ch + ".rs").split("#[cfg(test)]\nmod tests")[0]
    old = read(C + "src/chapters/" + ch + ".rs")
    old_names = set(re.findall(r"let (?:mut )?(\w+)\b", old))
    i = staged.index("fn construct")
    code = staged[i:]
    for m in re.finditer(r"let (?:mut )?(\w+)(?::[^=]+)? =\s*" + BUILD, code):
        name = m.group(1)
        if name in old_names or (ch, name) in HELD:
            continue
        rest = code[m.end():]
        if not re.search(r"\b" + name + r"\b", rest):
            problem(f"{ch}: {name} is built and never used")
            continue
        cleared = False
        for fade in re.finditer(r"(?:fade_out_all\(|gone\.(?:extend|push)\(|let mut gone|let mut all|all\.extend\(|remove_from_scene\()[^;]*;", rest):
            if re.search(r"\b" + name + r"\b", fade.group(0)):
                cleared = True
                break
        if not cleared:
            problem(f"{ch}: {name} comes in and is never cleared")

# 6. the supplement lists, every constant they name evaluated from source
def constants(path):
    s = read(path)
    out = {}
    for m in re.finditer(r'pub const (\w+): &str =\s*(r?)"((?:[^"\\]|\\.)*)"', s):
        v = m.group(3)
        out[m.group(1)] = v if m.group(2) else v.replace('\\"', '"').replace("\\\\", "\\")
    for m in re.finditer(r"pub const (\w+): \[&str; \d+\] =\s*\[(.*?)\];", s, re.S):
        items = []
        for x in re.finditer(r'(r?)"((?:[^"\\]|\\.)*)"', m.group(2)):
            v = x.group(2)
            items.append(v if x.group(1) else v.replace('\\"', '"').replace("\\\\", "\\"))
        out[m.group(1)] = items
    return out


mods = {}
for name in os.listdir(C + "src/chapters"):
    if name.endswith(".rs"):
        staged_path = R + "src/chapters/" + name
        mods[name[:-3]] = constants(staged_path if os.path.exists(staged_path) else C + "src/chapters/" + name)
for label, start in (("TEXT", "pub const TEXT: &[&str] = &["), ("TEX", "pub const TEX: &[&str] = &[")):
    a = mod_rs.index(start)
    seen = {}
    skipped = 0
    for line in mod_rs[a : mod_rs.index("\n];", a)].splitlines()[1:]:
        line = line.strip().rstrip(",")
        if not line or line.startswith("//"):
            continue
        m = re.fullmatch(r"(\w+)::(\w+)(?:\[(\d+)\])?", line)
        if m:
            value = mods.get(m.group(1), {}).get(m.group(2))
            if value is None:
                problem(f"{label}: cannot resolve {line}")
                continue
            if m.group(3) is not None:
                value = value[int(m.group(3))]
        elif line.startswith('"'):
            value = line[1:-1]
        elif line.startswith('r"'):
            value = line[2:-1]
        else:
            # closing::CAPTIONS[i][j], a nested array; its ten strings are compared by eye
            skipped += 1
            continue
        if value in seen:
            problem(f"{label}: {line} repeats {seen[value]}: {value!r}")
        seen[value] = line
    print(f"{label}: {len(seen)} distinct strings, {skipped} nested entries not read")

# 7. the script
a = hashlib.sha256(open(C + "script.tsv", "rb").read()).hexdigest()
print("script.tsv", a[:16], "(frozen at 2968b0fd7654)")
if not a.startswith("2968b0fd7654"):
    problem("script.tsv is not the frozen one")
if os.path.exists(R + "script.tsv"):
    problem("a staged script.tsv exists; the words are frozen")

print("problems:", len(problems))
sys.exit(1 if problems else 0)
