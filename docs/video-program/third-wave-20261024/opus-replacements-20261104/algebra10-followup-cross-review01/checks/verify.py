"""Checks for WhiteSwan's cyclic-picture-followup.patch. Nothing here builds, renders or listens.

Run from the repo root:  /usr/bin/python3 -I -B <this file>

This imports nothing from WhiteSwan's checks. Their scripts were read only for input paths.

TEXT    the patch against source: round trip, subsets, rustfmt, cue order, hold, tests, bindings;
MODEL   my own replay of the five touched cues on the scratch voice (controlled against frames)
        and on both Oliver takes, using root's ASR word times. Provisional: not listening, not
        fine sync, no take selected;
PIXELS  today's frames, read for empty slots and measured ink. No candidate picture exists;
MIRROR  a Python mirror of the new test, with mutations that must fail;
STACK   the two small corrections staged in ../patches, on top of the patch.

Writes verify.out next to itself. Exit 1 on any failed check.
"""

import difflib
import hashlib
import json
import math
import pathlib
import re
import struct
import subprocess
import sys
import tempfile
import wave
import zlib

HERE = pathlib.Path(__file__).resolve().parent
BUNDLE = HERE.parent
O = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
W = O / "algebra10-repair03-picture-review01"
PATCH = W / "patches/cyclic-picture-followup.patch"
CORRECTION = BUNDLE / "patches/question-at-predict.patch"
CORRECTION2 = BUNDLE / "patches/unit-orders-names-group.patch"
A = pathlib.Path("explainer_corpus/abstract_algebra_expanded")
REL = "explainer_corpus/abstract_algebra_expanded/src/cyclic.rs"
SOURCE = pathlib.Path(REL)
SCRIPT = A / "script.tsv"
MAP = A / "docs/narration-direction-map-oliver24.json"
UNITS = A / "narration_units.json"
ROOT = pathlib.Path("narration_lab/third-wave-20261024/root")
LAB = ROOT / "five-film-release-20261027"
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
FRAMES = LAB / "algebra-chapter10-repair03-frames01/10_cyclic"
CUES = LAB / "algebra-chapter10-repair03-preview01/render/10_cyclic.cues.tsv"
AFM = pathlib.Path("narration_lab/model-sidecar-20261017/.venv-cosyvoice3/lib/python3.10/site-packages/matplotlib/mpl-data/fonts/afm/cmr10.afm")

BASE_SHA = "bee439fdad35eb75a22a05d32d4e9aad059feabac0456fbfd29e24746b391d46"
RESULT_SHA = "bee0eb2c55f2b9ad43fcd10b4f0993947b7fd104cda963e63ed27ccfefa4f17f"
SCRIPT_SHA = "83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0"
MAP_SHA = "7f05be9377a54b2c4e1a03ca83b563eb87d90734818273d59d95cea566784d4a"
PATCH_SHA = "dbb3a64aebb68076edd1be98a24e2709c807cc980a613f1f29d89e8790491d19"

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


# ===================================================================== TEXT
say("== TEXT: inputs")
base_bytes = SOURCE.read_bytes()
check("canonical cyclic.rs is the patch's base", sha(base_bytes) == BASE_SHA, BASE_SHA)
check("script.tsv", sha(SCRIPT.read_bytes()) == SCRIPT_SHA, SCRIPT_SHA)
check("direction map", sha(MAP.read_bytes()) == MAP_SHA, MAP_SHA)
check("patch", sha(PATCH.read_bytes()) == PATCH_SHA, PATCH_SHA)
base = base_bytes.decode()
LINES = {r.split("\t", 1)[0]: r.split("\t", 1)[1] for r in SCRIPT.read_text(encoding="utf-8").splitlines() if "\t" in r}


def parse_patch(path):
    """Own hunk reader: (old_start, old_lines, new_lines) for each hunk, plus the file headers."""
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


def apply(source, hunks, chosen=None, exact=True):
    """Apply chosen hunks in order. exact: at its stated line, no search. Otherwise find it once."""
    lines = source.split("\n")
    shift = 0
    for index, (start, old, new) in enumerate(hunks):
        if chosen is not None and index not in chosen:
            continue
        at = start - 1 + shift
        if lines[at : at + len(old)] != old:
            if exact:
                raise ValueError(f"hunk {index} does not match at line {start}")
            spots = [p for p in range(len(lines)) if lines[p : p + len(old)] == old]
            if len(spots) != 1:
                raise ValueError(f"hunk {index}: {len(spots)} places")
            at = spots[0]
        lines[at : at + len(old)] = new
        shift = at + len(new) - (start - 1) - len(old)
    return "\n".join(lines)


def unapply(source, hunks):
    """Undo hunks in order. Everything above the current hunk is already back in old line numbers."""
    lines = source.split("\n")
    for start, old, new in hunks:
        at = start - 1
        if lines[at : at + len(new)] != new:
            raise ValueError(f"reverse: hunk at {start} does not match")
        lines[at : at + len(new)] = old
    return "\n".join(lines)


say("\n== TEXT: the patch")
heads, hunks = parse_patch(PATCH)
raw = PATCH.read_text(encoding="utf-8").splitlines()
added = sum(1 for r in raw if r.startswith("+") and not r.startswith("+++"))
removed = sum(1 for r in raw if r.startswith("-") and not r.startswith("---"))
check("touches one file only, cyclic.rs", heads == [f"--- a/{REL}", f"+++ b/{REL}"], heads)
check("12 hunks, +92 -17", (len(hunks), added, removed) == (12, 92, 17), (len(hunks), added, removed))
result = apply(base, hunks)
check("my hunk walker reaches the claimed result, compared in full", sha(result) == RESULT_SHA, sha(result))
check("reverse returns the base", sha(unapply(result, hunks)) == BASE_SHA)

with tempfile.TemporaryDirectory() as tmp:
    tmp = pathlib.Path(tmp)
    target = tmp / REL
    target.parent.mkdir(parents=True)
    target.write_bytes(base_bytes)
    pa = PATCH.resolve()
    run = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", str(pa)], cwd=tmp, capture_output=True, text=True)
    check("patch(1) --fuzz=0 forward", run.returncode == 0 and sha(target.read_bytes()) == RESULT_SHA)
    run = subprocess.run(["patch", "-R", "-p1", "--fuzz=0", "-i", str(pa)], cwd=tmp, capture_output=True, text=True)
    check("patch(1) --fuzz=0 reverse", run.returncode == 0 and sha(target.read_bytes()) == BASE_SHA)

FINDINGS = {
    "A": ([3], "d8b37282b641"),
    "B": ([0, 1, 7, 8, 9, 10], "05beefda26a8"),
    "C": ([4, 5], "2816a9c2e766"),
    "D": ([2, 11], "ebf784219c1e"),
    "E": ([6], "4484c5db8342"),
}


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


say("\n== TEXT: each finding alone, and rustfmt")
for name, (chosen, prefix) in FINDINGS.items():
    alone = apply(base, hunks, set(chosen))
    ok, detail = rustfmt_ok(alone, f"subset_{name}")
    check(f"{name}: hunks {chosen} reach {prefix} and pass rustfmt", sha(alone).startswith(prefix) and ok, detail if not ok else "")
ok, detail = rustfmt_ok(result, "full")
check("whole result passes rustfmt", ok, detail)
ok, detail = rustfmt_ok(base, "baseline")
check("control: the baseline passes rustfmt", ok, detail)
broken = result.replace('let group_size = k.text(\n            stage,', "let group_size = k.text(stage,", 1)
ok, _ = rustfmt_ok(broken, "broken")
check("control: rustfmt refuses a misformatted copy", not ok)


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


def scene_segments(src):
    """(cue, [lines]) from each k.say to the next."""
    segments, current = [], None
    for number, line in enumerate(src.split("\n"), 1):
        m = re.search(r'k\.say\(stage, "(\d+_\d+)"\)\?;', line)
        if m:
            current = (m.group(1), [])
            segments.append(current)
        elif current is not None:
            current[1].append((number, line))
    return segments


def phrases_of(src):
    return dict(re.findall(r'^const (\w+_PHRASE): &str = "([^"]+)";', src, re.M))


def order_problems(src):
    """Every wait in its own cue, in range, not going backwards, its phrase inside its sentence."""
    problems = []
    named = phrases_of(src)
    for cue, rows in scene_segments(src):
        spans = sentence_ranges(LINES[cue])
        last = (-1, -1)
        for number, line in rows:
            until = re.search(r'k\.until_sentence\(stage, "(\d+_\d+)", (\d+)\)', line)
            loop = re.search(r'k\.until_sentence\(stage, "(\d+_\d+)", index \+ 1\)', line)
            phrase = re.search(r'until_phrase\(stage, &k, "(\d+_\d+)", (\d+), (\w+)\)', line)
            if loop and loop.group(1) != cue:
                problems.append(f"{number}: wrong cue")
            if until:
                j = int(until.group(2))
                if until.group(1) != cue:
                    problems.append(f"{number}: names {until.group(1)} inside {cue}")
                if j >= len(spans):
                    problems.append(f"{number}: sentence {j} of {len(spans)}")
                if (j, 0) < last:
                    problems.append(f"{number}: goes back to sentence {j}")
                last = max(last, (j, 0))
            if phrase:
                j = int(phrase.group(2))
                if phrase.group(1) != cue:
                    problems.append(f"{number}: names {phrase.group(1)} inside {cue}")
                if j >= len(spans):
                    problems.append(f"{number}: sentence {j} of {len(spans)}")
                    continue
                a, b = spans[j]
                text = LINES[cue][a:b]
                needle = named[phrase.group(3)]
                if needle not in text:
                    problems.append(f"{number}: {needle!r} not in sentence {j}")
                else:
                    at = (j, a + text.index(needle) - a)
                    if at < last:
                        problems.append(f"{number}: phrase goes backwards")
                    last = max(last, at)
    return problems


say("\n== TEXT: cue order, sentence indices and phrases")
says = lambda s: re.findall(r'k\.say\(stage, "(\d+_\d+)"\)', s)
check("k.say order is unchanged", says(base) == says(result), f"{len(says(result))} cues in cyclic.rs")
check("baseline waits all legal", order_problems(base) == [], order_problems(base))
check("patched waits all legal", order_problems(result) == [], order_problems(result))
bad = result.replace('k.until_sentence(stage, "10_2", 8)', 'k.until_sentence(stage, "10_2", 9)')
check("control: an out-of-range sentence is caught", any("sentence 9" in p for p in order_problems(bad)))
bad = result.replace('"so this group is not"', '"so this group is"').replace('"10_10", 5, ROTATIONS_PHRASE', '"10_10", 4, ROTATIONS_PHRASE')
check("control: a phrase in the wrong sentence is caught", any("not in sentence" in p for p in order_problems(bad)))
counts = {c: len(sentence_ranges(LINES[c])) for c in ("10_1", "10_2", "10_4", "10_6", "10_10")}
check("sentence counts the patch relies on", counts == {"10_1": 5, "10_2": 9, "10_4": 4, "10_6": 6, "10_10": 6}, counts)

say("\n== TEXT: the prediction hold")
base_lines = base.split("\n")
result_lines = result.split("\n")
marker = "k.hold(stage)?;"
base_holds = [i + 1 for i, r in enumerate(base_lines) if r.strip() == marker]
result_holds = [i + 1 for i, r in enumerate(result_lines) if r.strip() == marker]
direction = json.loads(MAP.read_text())
holds = direction["visual_holds"]
ours = [h for h in holds if h.get("path") == REL or "cyclic" in json.dumps(h)]
say(f"     the map holds {len(holds)} rows in all; rows naming cyclic.rs: {len(ours)}")
row = ours[0]
check("exactly one row names cyclic.rs, and it is 10_1 to 10_2", len(ours) == 1 and (row["cue"], row["next_cue"], row["kind"]) == ("10_1", "10_2", "prediction"))
check("the map row binds the base hash, line 448 and the marker", (row["sha256"], row["line"], row["marker"]) == (BASE_SHA, 448, marker))
check("the marker is unique and at 448 in the base", base_holds == [448], base_holds)
check("the marker is unique and at 453 after the whole patch", result_holds == [453], result_holds)
check(
    "wait and next cue still follow the marker",
    [r.strip() for r in result_lines[453:455]] == ["stage.wait(3.0)?;", 'k.say(stage, "10_2")?;'],
)
for name, (chosen, _) in FINDINGS.items():
    lines = apply(base, hunks, set(chosen)).split("\n")
    say(f"     hold line with only {name}: {[i + 1 for i, r in enumerate(lines) if r.strip() == marker]}")
window = "\n".join(r for _, r in dict(scene_segments(result))["10_1"])
named_mobs = re.findall(r"(?:fade|show|appear|draw)\(stage, &([\w\[\]\.]+)", window)
check("only opening and question are shown before the hold", named_mobs == ["opening", "[question]"], named_mobs)
check("nothing in the 10_1 window names an answer", not re.search(r"\b(dots?|trail|orbit|order_tex|walk)\b", window))
check("the map binds the script unchanged", direction["scripts"]["abstract_algebra_expanded"] == SCRIPT_SHA)
units = json.loads(UNITS.read_text())
contract = units["direction_contract"]
check("narration_units binds this script and this map today", (contract["script_sha256"], contract["map"]["sha256"]) == (SCRIPT_SHA, MAP_SHA))
say("     (native 'predict' mark in cyclic.rs: " + str(len(re.findall(r"predict_pause|\.mark\(stage", base))) + "; see REVIEW.md)")

say("\n== TEXT: what else binds this source")
needle = BASE_SHA.encode()
hits = []
for tree in (A / "docs", A / "src", pathlib.Path("tools"), pathlib.Path("docs/publish")):
    for path in tree.rglob("*"):
        if path.is_file() and needle in path.read_bytes():
            hits.append(str(path))
for path in (A / "narration_units.json", A / "script.tsv", pathlib.Path("docs/video-program/catalog.json")):
    if needle in path.read_bytes():
        hits.append(str(path))
check("in the crate, tools and catalog, only the current map binds the hash", hits == [str(MAP)], hits)
tool_text = "".join(p.read_text(errors="replace") for p in pathlib.Path("tools").glob("*.py"))
say("     tools that read a hold's line or marker: " + str(len(re.findall(r"""\[['"](?:marker)['"]\]|get\(['"]marker['"]\)""", tool_text))))
timings = [str(p) for p in TAKES.rglob("phrase-timings.json")]
say(f"     phrase-timings.json files under the algebra narration folder: {len(timings)}")
docs = [p.name for p in (A / "docs").iterdir() if not p.name.startswith("narration-direction-map")]
check("the crate's docs folder holds only direction maps, so no doc names these beats", docs == [], docs)

say("\n== TEXT: tests and facts")
tests_in = lambda s: len(re.findall(r"#\[test\]", s))
others = sum(tests_in(p.read_text(encoding="utf-8")) for p in (A / "src").rglob("*.rs") if p.name != "cyclic.rs")
check("110 tests today", others + tests_in(base) == 110, others + tests_in(base))
check("111 after the patch", others + tests_in(result) == 111, others + tests_in(result))


def phrases_table(src):
    m = re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\] = \[(.*?)\n\];", src, re.S)
    return int(m.group(1)), re.findall(r'\("(\d+_\d+)", (\d+), (\w+)\)', m.group(2))


declared, rows = phrases_table(result)
check("PHRASES declares 13 and lists 13", (declared, len(rows)) == (13, 13), (declared, len(rows)))
named = phrases_of(result)
check("every listed phrase is in its listed sentence", all(named[n] in LINES[c][slice(*sentence_ranges(LINES[c])[int(j)])] for c, j, n in rows))
scene_waits = set(re.findall(r'until_phrase\(stage, &k, "(\d+_\d+)", (\d+), (\w+)\)', result))
check("every phrase the scene waits for is in the test's list", scene_waits <= {tuple(r) for r in rows}, scene_waits - {tuple(r) for r in rows})
m = re.search(r"for \(cue, sentences\) in \[(.*?)\]", result, re.S)
listed = re.findall(r'\("(\d+_\d+)", (\d+)\)', m.group(1))
check("the sentence-count list adds 10_1 at 5, and each row is true", ("10_1", "5") in listed and all(len(sentence_ranges(LINES[c])) == int(n) for c, n in listed), listed[:3])
facts_touched = any("main.rs" in h or "oracle" in h for h in heads)
check("main.rs and oracle.py are outside the patch", not facts_touched)
oracle = subprocess.run([sys.executable, "-I", "-B", str(A / "oracle.py")], capture_output=True, text=True)
rows_oracle = [r for r in oracle.stdout.splitlines() if r.strip()]
check("the oracle still prints 240 facts (the Rust side was not run)", len(rows_oracle) == 240, len(rows_oracle))
oracle_text = (A / "oracle.py").read_text(encoding="utf-8")
say("     oracle mentions of unit squares or unit orders: " + str(len(re.findall(r"\bU\(12\)|unit_order|units|square_mod", oracle_text))))

say("\n== TEXT: new pictures come on once and leave once")
for name in ("group_size", "each_coprime", "unit_orders", "rotations"):
    made = len(re.findall(rf"let {name} = ", result))
    shown = len(re.findall(rf"fade\(stage, &\[{name}\], true\)", result))
    check(f"{name}: made once, shown once", (made, shown) == (1, 1), (made, shown))
check("group_size leaves with the question at 10_3", "d4g::hide(stage, &[question, trail, group_size], 0.4)?;" in result)
check("the 10_10 rows leave with the table at 10_11", "table.extend([each_coprime, squares, unit_orders, no, rotations]);" in result)
check("table[0] is units, so [..1] and [1..] split it there", "let mut table = vec![units];" in result)
check("question faded in exactly once", len(re.findall(r"fade\(stage, &\[question\], true\)", result)) == 1)
check("the opening no longer carries the question", "opening.extend([question, marker]);" not in result and "opening.push(marker);" in result)


def segment_lines(src, cue):
    return [r for _, r in dict(scene_segments(src))[cue]]


c_base, c_result = segment_lines(base, "10_4"), segment_lines(result, "10_4")
check("C is a pure reorder: same 10_4 lines, other order", sorted(c_base) == sorted(c_result) and c_base != c_result)
at = {name: next(i for i, r in enumerate(c_result) if pat in r) for name, pat in (("wait", "K_SPLIT_PHRASE"), ("k", "show(stage, &[k_split]"), ("hop", "show(stage, &[hop]"), ("hide", "hide(stage, &[hop]"))}
check("C: phrase, then k_split, then hop, then the hop's hide", at["wait"] < at["k"] < at["hop"] < at["hide"], at)
e_base, e_result = segment_lines(base, "10_6"), segment_lines(result, "10_6")
drop_b = [r for r in e_base if "row_names[..2]" not in r and "row_names[2..]" not in r]
drop_r = [r for r in e_result if "row_names[..1]" not in r and "row_names[1..]" not in r and "question stands" not in r]
check("E: apart from the two name slices and a comment, 10_6 holds the same lines", sorted(drop_b) == sorted(drop_r) and drop_b != drop_r)
want = ["row_names[..1]", "appear(stage, &moves", 'until_sentence(stage, "10_6", 2)', "row_names[1..]", "appear(stage, &divisors", "appear(stage, &orders", "pulse(stage, &[rule])", "GENERATORS_PHRASE"]
spots = [next(i for i, r in enumerate(e_result) if w in r) for w in want]
check("E: k row, then the question's end, then gcd, ord, the rule's pulse, the phrase", spots == sorted(spots), spots)


# ===================================================================== MODEL
say("\n== MODEL: voices")
NUMBER = {"0": "zero", "1": "one", "2": "two", "3": "three", "4": "four", "5": "five", "6": "six", "7": "seven", "8": "eight", "9": "nine", "10": "ten", "11": "eleven", "12": "twelve"}


def tokens(text):
    out_words = []
    for word in text.replace("-", " ").split():
        clean = re.sub(r"[^a-z0-9]", "", word.lower())
        if clean:
            out_words.append(NUMBER.get(clean, clean))
    return out_words


class Voice:
    """Sentence starts, cue length, and word times where there is a recording."""

    def __init__(self, cue, name):
        self.cue, self.name = cue, name
        self.text = LINES[cue]
        self.spans = sentence_ranges(self.text)
        if name == "scratch":
            speech = len(self.text.split()) / 2.6
            total = sum(b - a for a, b in self.spans)
            before, self.starts = 0, []
            for a, b in self.spans:
                self.starts.append(speech * before / total)
                before += b - a
            self.end = speech
            self.length = speech + 0.4
            self.words = None
            return
        folder = LAB / f"expanded-algebra-{cue.replace('_', '-')}-asr01"
        segments = json.loads((folder / f"{name}-large.json").read_text(encoding="utf-8"))["result"]["segments"]
        heard, times = [], []
        for s in segments:
            for w in tokens(s["text"]):
                heard.append(w)
                times.append(s["start_sec"])
        said = tokens(self.text)
        match = {}
        for a, b, n in difflib.SequenceMatcher(None, said, heard, autojunk=False).get_matching_blocks():
            for k in range(n):
                match[a + k] = times[b + k]
        self.word_at = match
        self.said = said
        self.matched = len(match)
        self.total = len(said)
        self.starts = []
        for a, b in self.spans:
            index = len(tokens(self.text[:a]))
            while index not in match:
                index += 1
            self.starts.append(match[index])
        self.starts[0] = 0.0
        with wave.open(str(TAKES / f"{cue}-{name}.wav")) as w:
            self.length = w.getnframes() / w.getframerate()
        self.end = self.length
        self.words = True

    def phrase(self, j, needle):
        """The kit's fallback: the phrase's share of the sentence between two sentence starts."""
        a, b = self.spans[j]
        byte = a + self.text[a:b].index(needle)
        start = self.starts[j]
        stop = self.starts[j + 1] if j + 1 < len(self.starts) else self.length
        return start + (stop - start) * (byte - a) / (b - a)

    def heard(self, words):
        """When the recording reaches these words, from the ASR. None on the scratch voice."""
        if not self.words:
            return None
        wanted = tokens(words)
        for i in range(len(self.said) - len(wanted) + 1):
            if self.said[i : i + len(wanted)] == wanted:
                for k in range(len(wanted)):
                    if i + k in self.word_at:
                        return self.word_at[i + k]
        return None


def lagged(n, each, lag):
    return each * (1 + (n - 1) * lag)


def s(j):
    return ("s", j)


def p(j, needle):
    return ("p", j, needle)


def b(name, dur):
    return ("b", name, dur)


STAR = lagged(4, 0.7, 0.5)
ROW = lagged(11, 0.3, 0.3)
BLOCKS = lagged(3, 0.5, 0.6)
# A play shorter than a whole frame takes the next whole frame; the sampled preview runs at 30 fps.
# Only the short per-hour steps are affected enough to matter.
FPS = 30
STEP_08 = math.ceil(0.08 * FPS) / FPS
STEP_15 = math.ceil(0.15 * FPS) / FPS
ROUND = 4 * STEP_08 + 0.6
HOP = 2 * ROUND
K_PHRASE = "and k as d times"
GEN = "one, five, seven"
ROT = "so this group is not"

P_10_1 = {
    "base": [b("opening", 0.4)],
    "patch": [b("opening", 0.4), s(3), b("question", 0.4)],
    "fix": [b("opening", 0.4), s(4), b("question", 0.4)],
}
TRIPS = []
for trip in (1, 2, 3):
    TRIPS += [s(trip), b(f"step{trip}", 8 * STEP_15), b(f"dot{trip}", 0.4)]
TAIL_2 = [s(4), b("trail", 0.4), s(5), b("orbit", 0.4), s(6), b("order", 0.4)]
P_10_2 = {"base": TRIPS + TAIL_2, "patch": TRIPS + TAIL_2 + [s(7), b("group", 0.4), s(8), b("pulse", 0.6)]}
HEAD_4 = [b("hide", 0.4), b("title", 0.6), b("setting", 0.4), s(1), b("d", 0.4), b("star", STAR), b("tint", 0.4), b("count", 0.3), s(2), b("hide3", 0.3), b("ink", 0.3), b("blocks", BLOCKS), b("labels", 0.3), b("n", 0.3), p(2, K_PHRASE)]
TAIL_4 = [s(3), b("coprime", 0.4), b("pulse", 0.6)]
P_10_4 = {
    "base": HEAD_4 + [b("disc", 0.2), b("hop", HOP), b("k", 0.3), b("dischide", 0.3)] + TAIL_4,
    "patch": HEAD_4 + [b("k", 0.3), b("disc", 0.2), b("hop", HOP), b("dischide", 0.3)] + TAIL_4,
}
TAIL_6 = [p(2, GEN), b("columns", lagged(4, 0.4, 0.8)), b("sort", 0.4), s(3), b("faces", lagged(4, 0.3, 0.2)), b("tours", 2.4), s(4), b("trap", 0.3 + 1.2 + 0.3 + 0.4), s(5), b("caught", 0.4 + 0.3 + 0.6)]
HEAD_6 = [b("hide", 0.4), b("rule", 0.8), s(1), b("asked", 0.6)]
P_10_6 = {
    "base": HEAD_6 + [b("names2", 0.3), b("k", ROW), b("gcd", ROW), s(2), b("name3", 0.3), b("ord", ROW), b("pulse", 0.6)] + TAIL_6,
    "patch": HEAD_6 + [b("name1", 0.3), b("k", ROW), s(2), b("names2", 0.3), b("gcd", ROW), b("ord", ROW), b("pulse", 0.6)] + TAIL_6,
}
P_10_10 = {
    "base": [b("hide", 0.4), b("table", 0.4), s(3), b("squares", 0.4), s(5), b("no", 0.4)],
    "patch": [b("hide", 0.4), b("units", 0.4), s(1), b("table", 0.4), s(2), b("each", 0.4), s(3), b("squares", 0.4), b("tint", 0.4), s(4), b("orders", 0.4), s(5), b("no", 0.4), p(5, ROT), b("rotations", 0.4)],
}
PROGRAMS = {"10_1": P_10_1, "10_2": P_10_2, "10_4": P_10_4, "10_6": P_10_6, "10_10": P_10_10}


def replay(voice, program):
    t, events = 0.0, {}
    for op in program:
        if op[0] == "s":
            t = max(t, voice.starts[op[1]])
        elif op[0] == "p":
            t = max(t, voice.starts[op[1]], voice.phrase(op[1], op[2]))
        else:
            events[op[1]] = (t, t + op[2])
            t += op[2]
    return events, t


VOICES = {cue: {n: Voice(cue, n) for n in ("scratch", "take01", "take02")} for cue in PROGRAMS}
for cue, by in VOICES.items():
    for n in ("take01", "take02"):
        v = by[n]
        say(f"     {cue} {n}: {v.matched}/{v.total} words aligned; length {v.length:.2f}; starts {[round(x, 2) for x in v.starts]}")

cue_table = {r.split("\t")[0]: (float(r.split("\t")[1]), float(r.split("\t")[2])) for r in CUES.read_text().splitlines()}
for cue in PROGRAMS:
    check(f"scratch length of {cue} matches the preview's cue table", abs(VOICES[cue]["scratch"].length - cue_table[cue][1]) < 0.02, (round(VOICES[cue]["scratch"].length, 3), cue_table[cue][1]))


# Transcription guard: my programs must hold as many timed calls as the source has, cue by cue.
def timed_calls(rows):
    text = "\n".join(r for _, r in rows)
    return len(re.findall(r"\b(?:fade|appear|draw|pulse|recolor|tint|swap|slide)\(stage|d4g::(?:show|hide)\(stage|play!\(stage", text))


say("\n== MODEL: transcription guard (differential: my patched programs change by what the patch changes)")
base_segs, patch_segs = dict(scene_segments(base)), dict(scene_segments(result))
check("10_1 gains one timed call", timed_calls(patch_segs["10_1"]) - timed_calls(base_segs["10_1"]) == 1)
check("10_2 gains two timed calls", timed_calls(patch_segs["10_2"]) - timed_calls(base_segs["10_2"]) == 2)
check("10_4 keeps its count of timed calls", timed_calls(patch_segs["10_4"]) == timed_calls(base_segs["10_4"]))
check("10_6 keeps its count of timed calls", timed_calls(patch_segs["10_6"]) == timed_calls(base_segs["10_6"]))
check("10_10 gains five timed calls", timed_calls(patch_segs["10_10"]) - timed_calls(base_segs["10_10"]) == 5, timed_calls(patch_segs["10_10"]) - timed_calls(base_segs["10_10"]))


# ===================================================================== PIXELS (loaded here for the control)
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


cache = {}


def frame(n):
    if n not in cache:
        cache[n] = read_png(FRAMES / f"frame-{n:04d}.png")
    return cache[n]


def px(x, y):
    return (x + 640 / 90) * 90, 360 - 90 * y


def ink_box(n, x0, y0, x1, y1, floor=40):
    """Unit-coordinates box of pixels brighter than the background by `floor`. None if empty."""
    width, height, bpp, rows = frame(n)
    bg = rows[5][5 * bpp : 5 * bpp + 3]
    (a, top), (c, bottom) = px(x0, y1), px(x1, y0)
    a, c = max(int(a), 0), min(int(c), width - 1)
    top, bottom = max(int(top), 0), min(int(bottom), height - 1)
    found = None
    for y in range(top, bottom + 1):
        line = rows[y]
        for x in range(a, c + 1):
            i = x * bpp
            if abs(line[i] - bg[0]) + abs(line[i + 1] - bg[1]) + abs(line[i + 2] - bg[2]) > floor:
                if found is None:
                    found = [x, y, x, y]
                else:
                    found[0] = min(found[0], x)
                    found[1] = min(found[1], y)
                    found[2] = max(found[2], x)
                    found[3] = max(found[3], y)
    if found is None:
        return None
    return (found[0] / 90 - 640 / 90, (360 - found[3]) / 90, (found[2] + 1) / 90 - 640 / 90, (360 - found[1]) / 90)


def changed(n, m, floor=30):
    """Pixels that differ between two sampled frames, on every second row and column."""
    fa, fb = frame(n), frame(m)
    count = 0
    for y in range(0, 720, 2):
        la, lb = fa[3][y], fb[3][y]
        for x in range(0, 1280 * fa[2], fa[2] * 2):
            if abs(la[x] - lb[x]) + abs(la[x + 1] - lb[x + 1]) + abs(la[x + 2] - lb[x + 2]) > floor:
                count += 1
    return count


times = json.loads((FRAMES / "sampling.json").read_text())["times"]
SHA_SAMPLING = sha((FRAMES / "sampling.json").read_bytes())


def first_ink(box, lo, hi):
    for n in range(lo, hi + 1):
        if ink_box(n, *box) is not None:
            return n
    return None


say("\n== MODEL: positive control, scratch replay of TODAY's source against measured ink in the frames")
ANCHORS = [
    ("10_2", "order", "ord(8)=3", (1.2, -1.6, 3.4, -1.1), 90, 98),
    ("10_4", "d", "d row", (1.0, 1.15, 4.8, 1.65), 182, 188),
    ("10_4", "count", "orbit label", (1.0, 0.5, 4.8, 0.9), 187, 193),
    ("10_4", "n", "n split", (1.0, -0.2, 4.8, 0.2), 193, 200),
    ("10_4", "k", "k split", (1.0, -0.9, 4.8, -0.5), 198, 205),
    ("10_4", "coprime", "coprime", (0.6, -1.6, 5.2, -1.2), 200, 207),
    ("10_6", "k", "k row", (-4.4, 1.1, 5.0, 1.4), 278, 286),
    ("10_6", "gcd", "gcd row", (-4.4, 0.55, 5.0, 0.85), 281, 288),
    ("10_6", "ord", "ord row", (-4.4, 0.0, 5.0, 0.3), 285, 292),
    ("10_10", "table", "table header row", (-4.4, 0.5, -1.4, 0.85), 503, 508),
    ("10_10", "squares", "squares", (0.6, 0.0, 4.4, 0.3), 532, 540),
    ("10_10", "no", "no", (0.6, -1.3, 4.4, -1.0), 546, 554),
]
for cue, beat, label, box, lo, hi in ANCHORS:
    events, _ = replay(VOICES[cue]["scratch"], PROGRAMS[cue]["base"])
    start, stop = events[beat]
    at = cue_table[cue][0] + start
    seen = first_ink(box, lo, hi)
    prior = times[seen - 1]
    # The beat must start before the first sample showing ink, and no more than a sample before the last clean one.
    good = prior - 0.55 <= at <= times[seen] + 0.05
    check(f"{cue} {label}: replay {at:.2f} | first ink frame {seen} at {times[seen]:.2f}, sample before {prior:.2f}", good)

say("\n== MODEL: picture beats against the voice (seconds into the cue)")
VERSIONS = ("base", "patch")
for cue, program in PROGRAMS.items():
    for name, voice in VOICES[cue].items():
        for version in VERSIONS:
            events, end = replay(voice, program[version])
            check(f"{cue} {name} {version}: picture ends {end:.2f}, inside speech end {voice.end:.2f}", end <= voice.end + 1e-9)


def beat(cue, version, voice, name):
    return replay(VOICES[cue][voice], PROGRAMS[cue][version])[0][name]


say("\n== MODEL: claims")
for voice in ("scratch", "take01", "take02"):
    # Unchanged stretches, from one beat's end to the next beat's start.
    b2 = beat("10_2", "base", voice, "order")[1]
    end2 = VOICES["10_2"][voice].end
    b10 = beat("10_10", "base", voice, "table")[1]
    s10 = beat("10_10", "base", voice, "squares")[0]
    evs = sorted(replay(VOICES["10_10"][voice], PROGRAMS["10_10"]["patch"])[0].values())
    gaps = [(evs[i + 1][0] - evs[i][1]) for i in range(len(evs) - 1)]
    say(f"     {voice}: A unchanged today {end2 - b2:.2f} | B unchanged today {s10 - b10:.2f} | B longest gap patched {max(gaps):.2f}")
for voice in ("take01", "take02"):
    want = {"take01": (9.8, 17.6, 7.5), "take02": (9.2, 17.4, 7.3)}[voice]
    got = (
        VOICES["10_2"][voice].end - beat("10_2", "base", voice, "order")[1],
        beat("10_10", "base", voice, "squares")[0] - beat("10_10", "base", voice, "table")[1],
    )
    evs = sorted(replay(VOICES["10_10"][voice], PROGRAMS["10_10"]["patch"])[0].values())
    longest = max(evs[i + 1][0] - evs[i][1] for i in range(len(evs) - 1))
    check(f"{voice}: WhiteSwan's A and B figures reproduce within 0.05", abs(got[0] - want[0]) < 0.05 and abs(got[1] - want[1]) < 0.05 and abs(longest - want[2]) < 0.06, (round(got[0], 2), round(got[1], 2), round(longest, 2)))

for voice in ("scratch", "take01", "take02"):
    same = [
        ("C coprime", beat("10_4", "base", voice, "coprime"), beat("10_4", "patch", voice, "coprime")),
        ("E columns", beat("10_6", "base", voice, "columns"), beat("10_6", "patch", voice, "columns")),
        ("E tours", beat("10_6", "base", voice, "tours"), beat("10_6", "patch", voice, "tours")),
        ("B squares", beat("10_10", "base", voice, "squares"), beat("10_10", "patch", voice, "squares")),
        ("B no", beat("10_10", "base", voice, "no"), beat("10_10", "patch", voice, "no")),
        ("A order", beat("10_2", "base", voice, "order"), beat("10_2", "patch", voice, "order")),
    ]
    check(f"{voice}: beats meant to stay do stay", all(abs(x[0] - y[0]) < 1e-9 for _, x, y in same), [n for n, x, y in same if abs(x[0] - y[0]) >= 1e-9])
    # C: k_split must come before the sentence 3 wait is released on real voices.
    k_b, k_p = beat("10_4", "base", voice, "k"), beat("10_4", "patch", voice, "k")
    say(f"     {voice}: C k_split today {k_b[0]:.2f}, patched {k_p[0]:.2f}; coprime {beat('10_4', 'patch', voice, 'coprime')[0]:.2f}; S3 {VOICES['10_4'][voice].starts[3]:.2f}")
    e = beat("10_6", "patch", voice, "pulse")
    say(f"     {voice}: E rows and pulse end {e[1]:.2f}; question ends {VOICES['10_6'][voice].starts[2]:.2f}; boxes begin {beat('10_6', 'patch', voice, 'columns')[0]:.2f}")

say("\n== MODEL: when each new reveal starts, against the words it shows (ASR; negative: the picture leads)")
LEADS = [
    ("10_1", "question", "add eight", "D: 'Add 8' vs 'add eight'"),
    ("10_1", "question", "predict how many", "D: 'how many' vs 'predict how many'"),
    ("10_2", "group", "whole clock group", "A"),
    ("10_4", "k", "and k as", "C"),
    ("10_4", "k", "k prime", "C"),
    ("10_6", "gcd", "greatest common divisor", "E"),
    ("10_10", "units", "a group with four", "B"),
    ("10_10", "table", "consider the numbers", "B"),
    ("10_10", "each", "relatively prime", "B"),
    ("10_10", "squares", "five squared", "B"),
    ("10_10", "orders", "order two", "B"),
    ("10_10", "no", "no single one", "B"),
    ("10_10", "rotations", "so this group is not", "B"),
    ("10_10", "rotations", "four rotations", "B"),
]
lead_rows = {}
for cue, name, words, tag in LEADS:
    row = []
    for voice in ("take01", "take02"):
        reveal = beat(cue, "patch", voice, name)[0]
        heard = VOICES[cue][voice].heard(words)
        row.append(reveal - heard)
    lead_rows[(cue, name, words)] = row
    say(f"     {cue} {name:9s} vs {words!r:27s} {row[0]:+.2f} / {row[1]:+.2f}  {tag}")

v1, v2 = VOICES["10_1"]["take01"], VOICES["10_1"]["take02"]
check("WhiteSwan's 13.36/13.28 is sentence 3's first word, 'On'", abs(v1.starts[3] - 13.36) < 0.02 and abs(v2.starts[3] - 13.28) < 0.02)
check("the voice reaches 'add eight' at about 16.00/15.90, not 13.36/13.28", abs(v1.heard("add eight") - 16.0) < 0.02 and abs(v2.heard("add eight") - 15.9) < 0.02, (round(v1.heard("add eight"), 2), round(v2.heard("add eight"), 2)))
check("so the patched question leads its words by more than 2.5 s on both takes", all(x < -2.5 for x in lead_rows[("10_1", "question", "add eight")]), lead_rows[("10_1", "question", "add eight")])
say(f"     a phrase wait on 'add eight' would land, by the kit's fallback, at {v1.phrase(3, 'add eight'):.2f} / {v2.phrase(3, 'add eight'):.2f}; sentence 4 starts {v1.starts[4]:.2f} / {v2.starts[4]:.2f}")

say("\n== MODEL: how long each new label stays, as a lower bound (it leaves at the next cue, after the voice ends)")
STAY = [
    ("10_1", "question", "Add 8: how many positions?"),
    ("10_2", "group", "Whole clock group: 12 elements"),
    ("10_10", "each", "Each relatively prime to 12"),
    ("10_10", "orders", "U(12): 5, 7 and 11 each have order 2"),
    ("10_10", "rotations", "Not the four rotations of a square"),
]
for cue, name, text in STAY:
    need = len(text) / 15
    for voice in ("take01", "take02"):
        vv = VOICES[cue][voice]
        program = P_10_1["fix"] if name == "question" else PROGRAMS[cue]["patch"]
        have = vv.end - replay(vv, program)[0][name][1]
        check(f"{voice} {name}: at least {have:.2f} s on screen, {need:.2f} s needed at 15 characters a second", have >= need)

say("\n== MODEL: F, the star, on today's source")
for voice in ("scratch", "take01", "take02"):
    star = beat("10_4", "base", voice, "star")
    tint_ = beat("10_4", "base", voice, "tint")
    count = beat("10_4", "base", voice, "count")
    d_row = beat("10_4", "base", voice, "d")
    gone = beat("10_4", "base", voice, "hide3")
    say(
        f"     {voice}: d {d_row[0]:.2f}; star {star[0]:.2f}-{star[1]:.2f}; tinted {tint_[1]:.2f}; label full {count[1]:.2f}; "
        f"star leaves {gone[0]:.2f}-{gone[1]:.2f}; whole picture held {gone[0] - tint_[1]:.2f}; label with star {gone[0] - count[1]:.2f}; "
        f"if retired, unchanged after d row {gone[0] - d_row[1]:.2f}"
    )
READING = len("4 orbits, 3 positions each") / 15
say(f"     the label at 15 characters a second needs {READING:.2f} s")
for voice in ("take01", "take02"):
    gone = beat("10_4", "base", voice, "hide3")
    count = beat("10_4", "base", voice, "count")
    check(f"{voice}: the label is readable while the star is still up", gone[0] - count[1] >= READING, round(gone[0] - count[1], 2))


# ===================================================================== PIXELS
say("\n== PIXELS: controls on the reader")
check("control: ord(8)=3 is ink on frame 100", ink_box(100, 1.2, -1.6, 3.4, -1.1) is not None)
check("control: 'No element' is ink on frame 560", ink_box(560, 0.6, -1.3, 4.4, -1.0) is not None)
check("control: the table's own row is ink on frame 560", ink_box(560, -5.2, -2.3, -1.4, 1.0) is not None)
check("control: an empty corner is empty on frame 100", ink_box(100, 5.0, 2.0, 6.9, 3.5) is None)
check("control: frames straddling the star differ", changed(185, 186) > 50 and changed(192, 193) > 50, (changed(185, 186), changed(192, 193)))

say("\n== PIXELS: today's still stretches the fixes aim at")
for label, a, b_, limit in (
    ("D 10_1, frames 4, 30, 62", 4, 30, 0),
    ("D 10_1", 30, 62, 0),
    ("A 10_2, frames 95, 107, 119", 95, 107, 0),
    ("A 10_2", 107, 119, 0),
    ("B 10_10, frames 506, 520, 535", 506, 520, 0),
    ("B 10_10", 520, 535, 0),
):
    n = changed(a, b_)
    check(f"{label}: {times[a]:.2f} to {times[b_]:.2f}, {n} changed sampled pixels", n <= limit + 20, n)

AFM_TEXT = AFM.read_text()
WIDTH = {}
KERN = {}
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


say("\n== PIXELS: the width law, against labels measured on these frames")
CONTROL = [
    ("Add 8: how many positions?", 30, 100, (0.2, 0.85, 4.8, 1.2)),
    ("4 orbits, 3 positions each", 26, 197, (1.0, 0.55, 4.8, 0.9)),
    ("No element generates all four", 31, 560, (0.3, -1.35, 4.8, -0.95)),
    ("Multiply modulo 12", 27, 560, (-5.0, 1.2, -0.5, 1.55)),
    ("The general rule", 36, 197, (-2.5, 2.8, 2.5, 3.3)),
]
for text, size, n, box in CONTROL:
    found = ink_box(n, *box)
    measured = found[2] - found[0]
    predicted = law(text, size)
    check(f"{text!r} {size} pt: law {predicted:.2f}, ink {measured:.2f}", abs(predicted - measured) < 0.08 + 0.02 * measured)

say("\n== PIXELS: A, the new label on frame 100")
gw = law("Whole clock group: 12 elements", 28)
g_x = (2.3 - gw / 2, 2.3 + gw / 2)
order_box = ink_box(100, 1.2, -1.75, 3.4, -0.95)
label_box = ink_box(197, 1.0, 0.45, 4.8, 0.95)
text_h_26 = label_box[3] - label_box[1]
g_h = text_h_26 * 28 / 26
say(f"     group_size width {gw:.2f}, x {g_x[0]:.2f} to {g_x[1]:.2f}; ord(8)=3 ink x {order_box[0]:.2f}..{order_box[2]:.2f}, y {order_box[1]:.2f}..{order_box[3]:.2f}")
check("the label's box is empty today, with the pulse's 1.25 scale allowed", ink_box(100, 2.3 - gw * 0.63, -2.15 - g_h * 0.63, 2.3 + gw * 0.63, -2.15 + g_h * 0.63) is None)
ring = ink_box(100, -2.0, -2.5, 0.5, 2.5)
check("the pulsed label clears the clock ring", 2.3 - gw * 0.625 > ring[2], (round(2.3 - gw * 0.625, 2), round(ring[2], 2)))
middle = (order_box[1] + order_box[3]) / 2
half = (order_box[3] - order_box[1]) / 2
pulsed = (middle - half * 1.25) - (-2.15 + g_h * 0.625)
check("order_tex and the label stay apart while both pulse", pulsed > 0.08, f"{pulsed:.2f} units, {pulsed * 90:.0f} px")
check("the pulsed label stays inside the safe frame", 2.3 + gw * 0.625 < 6.96 and -2.15 - g_h * 0.625 > -3.85)
check("10_3 draws divides in that slot, but only after group_size has left", re.search(r'hide\(stage, &\[question, trail, group_size\], 0\.4\)\?;\n\s+k\.until_sentence\(stage, "10_3", 1\)', result) is not None)

say("\n== PIXELS: B, the right column on frame 560")
sq = ink_box(560, 0.2, -0.25, 5.0, 0.6)
no = ink_box(560, 0.3, -1.45, 4.8, -0.85)
tab = ink_box(560, -5.3, -2.4, -0.6, 1.0)
say(f"     squares y {sq[1]:.2f}..{sq[3]:.2f}; no y {no[1]:.2f}..{no[3]:.2f}; table ink ends x {tab[2]:.2f}")
ord_w = order_box[2] - order_box[0]
ord_h = order_box[3] - order_box[1]
law_w = (2.726 + 0.778 + 0.5562 + 0.5) * 32 * 0.01
check(f"control: ord(8)=3 at 32 pt, em count {law_w:.2f}, ink {ord_w:.2f}", abs(law_w - ord_w) < 0.12)
uw = ((3 * 2.726) + 0.5 + 3 * (0.778 + 0.5562) + 0.5) * 32 * 0.01
u_x = (2.5 - uw / 2, 2.5 + uw / 2)
u_y = (-0.5 - ord_h / 2, -0.5 + ord_h / 2)
say(f"     unit_orders width {uw:.2f}, x {u_x[0]:.2f}..{u_x[1]:.2f}, height {ord_h:.2f}, y {u_y[0]:.2f}..{u_y[1]:.2f}")
above = sq[1] - u_y[1]
below = u_y[0] - no[3]
check(f"unit_orders clears squares by {above * 90:.0f} px and no by {below * 90:.0f} px", above * 90 >= 15 and below * 90 >= 15)
check("the unit_orders slot is empty today", ink_box(560, u_x[0], u_y[0] + 0.02, u_x[1], u_y[1] - 0.02) is None)
ew = law("Each relatively prime to 12", 28)
rw = law("Not the four rotations of a square", 27)
say(f"     each_coprime width {ew:.2f}; rotations width {rw:.2f}")
check("the each_coprime slot is empty today", ink_box(560, 2.5 - ew / 2, 1.0, 2.5 + ew / 2, 1.3) is None)
check("the rotations slot is empty today", ink_box(560, 2.5 - rw / 2, -1.97, 2.5 + rw / 2, -1.63) is None)
left = min(u_x[0], 2.5 - ew / 2, 2.5 - rw / 2)
check(f"every new row clears the table: leftmost x {left:.2f} against {tab[2]:.2f}", left > tab[2] + 0.5)
check("every new row is inside the safe frame", max(u_x[1], 2.5 + ew / 2, 2.5 + rw / 2) < 6.96)
rot_top = -1.8 + text_h_26 * 27 / 26 / 2
check(f"rotations clears no by {(no[1] - rot_top) * 90:.0f} px", (no[1] - rot_top) * 90 >= 15, (round(rot_top, 2), round(no[1], 2)))

say("\n== PIXELS: hero against ink, the tint must read")
# Hero and ink as the kit defines them; the frames' header digits are hero, the cells ink.
fw, fh, fb, frows = frame(560)
blues = {"header": [], "cell": []}
for name, box in (("header", (-5.2, -0.3, -4.6, 0.1)), ("cell", (-4.4, -0.3, -3.8, 0.1))):
    a, c = int(px(box[0], 0)[0]), int(px(box[2], 0)[0])
    t, u = int(px(0, box[3])[1]), int(px(0, box[1])[1])
    best = (0, None)
    for y in range(t, u):
        for x in range(a, c):
            i = x * fb
            g = frows[y][i + 1]
            if g > best[0]:
                best = (g, tuple(frows[y][i : i + 3]))
    blues[name] = best[1]
check("brightest header pixel is yellow, brightest cell pixel is near white", blues["header"][2] < 170 and blues["cell"][2] > 200, blues)


# ===================================================================== MIRROR
say("\n== MIRROR: the new test, and the diagonal rule")


def gcd(a, b):
    while b:
        a, b = b, a % b
    return a


def generators(n):
    return [k for k in range(1, n) if gcd(k, n) == 1]


def mirror(n, literal):
    """The patch's test, written again, for modulus n and the list it asserts. The first failure, or None."""
    units = generators(n)
    if units != literal:
        return "the list"
    for u in units:
        if gcd(u, n) != 1:
            return f"gcd {u}"
        for w in units:
            if (u * w) % n not in units:
                return f"closure {u} {w}"
        if (u * u) % n != 1:
            return f"square {u}"
        for power in range(4):
            reached = (u**power) % n
            if not (reached == 1 or reached == u):
                return f"power {u} {power}"
    return None


check("mirror passes on the film's group", mirror(12, [1, 5, 7, 11]) is None)
for n, literal, why in (
    (12, [1, 5, 7, 10], "a wrong list"),
    (12, [1, 5, 7], "a short list"),
    (5, [1, 2, 3, 4], "U(5), a cyclic group of four"),
    (10, [1, 3, 7, 9], "U(10), a cyclic group of four"),
):
    check(f"mutation fails: {why}", mirror(n, literal) is not None, mirror(n, literal))
check("control: U(8), also not cyclic, rightly passes the test's body", mirror(8, [1, 3, 5, 7]) is None)
ranks = {u: next(k for k in range(1, 13) if pow(u, k, 12) == 1) for u in [1, 5, 7, 11]}
check("brute force: ord(5)=ord(7)=ord(11)=2 under multiplication mod 12", ranks == {1: 1, 5: 2, 7: 2, 11: 2}, ranks)
powers = {u: {pow(u, k, 12) for k in range(6)} for u in [1, 5, 7, 11]}
check("brute force: no element generates all four", all(len(x) < 4 for x in powers.values()), powers)
labels = [1, 5, 7, 11]
diag = [(v, (v * v) % 12) for i, v in enumerate(labels) for j, o in enumerate(labels) if i == j and v != 1]
check("the tint rule picks exactly 5×5, 7×7, 11×11, each equal to 1", diag == [(5, 1), (7, 1), (11, 1)], diag)
wrong = [(v, (v * v) % 12) for i, v in enumerate(labels) for j, o in enumerate(labels) if i == j]
check("control: without value != 1 the rule also takes 1×1", len(wrong) == 4)
rots = {k: next(m for m in range(1, 5) if (m * k) % 4 == 0) for k in range(4)}
check("the square's rotations do have an element of order 4, so the new label is true", max(rots.values()) == 4, rots)
check("group_size's number is N, spoken as twelve", "{N} elements" in result and re.search(r"pub\(crate\) const N: i32 = 12|const N: i32 = 12", result) is not None)
orbit_rows = [[(h + 8 * i) % 12 for i in range(3)] for h in range(4)]
check("F: four orbits of three, covering all twelve hours once", sorted(x for o in orbit_rows for x in o) == list(range(12)))
check("F: the label's 4 and 3 are the factors of n = d n' = 4·3", (gcd(12, 8), 12 // gcd(12, 8)) == (4, 3))
cosets = [{(x + shift) % 12 for x in (0, 4, 8)} for shift in range(4)]
check("F: the four triangles are chapter 14's four cosets of {0,4,8}, in the same shift order", [set(o) for o in orbit_rows] == cosets)
check("F: 14_8 says so aloud", "The twelve-clock is four cosets of three." in LINES["14_8"] and "the shift carried the triangle around the clock" in LINES["14_7"])
first = next(c for c in LINES if re.search(r"\borbit", LINES[c]))
check("F: the first spoken 'orbit' in script order is 10_11", first == "10_11", first)
as_moves = {k: next(m for m in range(1, 13) if m * k % 12 == 0) for k in (5, 7, 11)}
check("the clash: as moves, ord(5)=ord(7)=ord(11)=12, the boxed columns on sheet 080", as_moves == {5: 12, 7: 12, 11: 12}, as_moves)
check("the clash: the rule on screen in 10_6 gives the same", all(12 // gcd(k, 12) == 12 for k in (5, 7, 11)))
check("the clash: chapter 12 already words it with the group's name", '"U(12): no element has order 4"' in (A / "src/isomorphism.rs").read_text(encoding="utf-8"))
main_text = (A / "src/main.rs").read_text(encoding="utf-8")
glyphs = main_text[main_text.index("glyph_text: &["):main_text.index("glyph_tex: &[")]
letters = set("".join(re.findall(r'"([^"]*)"', glyphs)))
for text in ("Whole clock group: 12 elements", "Each relatively prime to 12", "Not the four rotations of a square", "U(12): 5, 7 and 11 each have order 2"):
    check(f"the glyph list already covers every character of {text!r}", set(text) <= letters, set(text) - letters)


say("\n== TEXT: what retiring F would really touch")
tests_at = base.index("#[cfg(test)]\nmod tests")
scene_part, test_part = base[:tests_at], base[tests_at:]
check("orbit_color: only the scene uses it, no test does, so it would be dead code", len(re.findall(r"orbit_color\(", scene_part)) == 3 and "orbit_color" not in test_part)
check("orbits: the facts in main.rs use it outside any test", "cyclic::orbits(N, K)" in main_text[: main_text.index("#[cfg(test)]")])
check("orbits: two tests also use it", len(re.findall(r"\borbits\(", test_part)) == 2)
check("only triangles[1..] leave at sentence 2", "d4g::hide(stage, &triangles[1..], 0.3)?;" in scene_part)
check("the walk's own triangle lives on: 10_5 pulses it, 10_6 hides it", "pulse(stage, &[formula, triangles[0]])?;" in scene_part and "vec![ring_mob, marker, triangles[0]]" in scene_part)
check("the label leaves at 10_5", "d4g::hide(stage, &[setting, divisor, count], 0.3)?;" in scene_part)
on_screen = []
for path in sorted((A / "src").glob("*.rs")):
    text = path.read_text(encoding="utf-8")
    cut = text.find("#[cfg(test)]\nmod tests")
    for line in (text if cut < 0 else text[:cut]).split("\n"):
        if re.search(r'"[^"]*orbit[^"]*"', line) and "facts.add" not in line:
            on_screen.append(line.strip())
check("in the whole film, the word 'orbit' is on screen in one place: this label", len(on_screen) == 1 and "orbits, {} positions each" in on_screen[0], on_screen)
for voice in ("scratch", "take01", "take02"):
    vv = VOICES["10_4"][voice]
    gone = beat("10_4", "base", voice, "hide3")
    ink = beat("10_4", "base", voice, "ink")
    say(f"     {voice}: hours back to ink {ink[1]:.2f}; the label outlives the three triangles by at least {vv.length - gone[0]:.2f} s, to the end of the cue")

say("\n== TEXT: root's combined-repair-stage02, as found when this ran")
STAGE02 = TAKES / "combined-repair-stage02/explainer_corpus/abstract_algebra_expanded"
units2 = json.loads((STAGE02 / "narration_units.json").read_text())
map2 = units2["direction_contract"]["map"]
direction2 = json.loads((STAGE02 / "docs" / pathlib.Path(map2["path"]).name).read_text())
rows2 = [h for h in direction2["visual_holds"] if h["cue"] == "10_1"]
check("stage02 holds this same source", sha((STAGE02 / "src/cyclic.rs").read_bytes()) == BASE_SHA)
check("its map binds it at line 448, so the rebind applies there too", len(rows2) == 1 and rows2[0]["line"] == 448 and rows2[0]["sha256"] == BASE_SHA, map2["path"])
check("its map holds 16 rows", len(direction2["visual_holds"]) == 16)

probe = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=r_frame_rate,width,height", "-of", "csv=p=0", str(CUES.with_suffix("").with_suffix(".mp4"))], capture_output=True, text=True)
check("the preview is 1280x720 at 30 fps, as the model and reader assume", probe.stdout.strip() == "1280,720,30/1", probe.stdout.strip())


# ===================================================================== STACK
say("\n== STACK: the correction, on top of the patch")
fixed = result.replace('k.until_sentence(stage, "10_1", 3)?;', 'k.until_sentence(stage, "10_1", 4)?;', 1)
if CORRECTION.exists():
    heads2, hunks2 = parse_patch(CORRECTION)
    stacked = apply(result, hunks2)
    check("it applies at its stated line, and changes one line", stacked == fixed and len(hunks2) == 1, sha(stacked))
    check("reverse returns the patch's result", sha(unapply(stacked, hunks2)) == RESULT_SHA)
    d_only = apply(base, hunks, {2, 11})
    alone = apply(d_only, hunks2, exact=False)
    check("it also applies on D alone, by unique context", 'k.until_sentence(stage, "10_1", 4)?;' in alone and order_problems(alone) == [])
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        target = tmp / REL
        target.parent.mkdir(parents=True)
        target.write_text(result, encoding="utf-8")
        run = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", str(CORRECTION.resolve())], cwd=tmp, capture_output=True, text=True)
        check("patch(1) --fuzz=0 agrees", run.returncode == 0 and target.read_text(encoding="utf-8") == fixed)
    ok, detail = rustfmt_ok(stacked, "stacked")
    check("rustfmt passes", ok, detail)
    check("waits remain legal", order_problems(stacked) == [], order_problems(stacked))
    check("the hold stays at line 453", [i + 1 for i, r in enumerate(stacked.split("\n")) if r.strip() == marker] == [453])
    check("no test or phrase table changes: still 111 tests, 13 phrases", tests_in(stacked) == tests_in(result) and phrases_table(stacked)[0] == 13)
    say("     result sha256 " + sha(stacked))
    for voice in ("scratch", "take01", "take02"):
        vv = VOICES["10_1"][voice]
        q = replay(vv, P_10_1["fix"])[0]["question"]
        q0 = replay(vv, P_10_1["patch"])[0]["question"]
        base_q = replay(vv, P_10_1["base"])[0]["opening"]
        line = f"     {voice}: question today {base_q[0]:.2f}, patch {q0[0]:.2f}, corrected {q[0]:.2f}-{q[1]:.2f}; speech ends {vv.end:.2f}"
        if voice != "scratch":
            line += f"; 'add eight' {vv.heard('add eight'):.2f}, 'predict' {vv.heard('predict'):.2f}, 'how many' {vv.heard('how many'):.2f}"
        say(line)
        check(f"{voice}: the corrected question is in place before the voice ends", q[1] < vv.end)
        if voice != "scratch":
            check(f"{voice}: and it follows 'add eight' and comes ahead of 'predict'", vv.heard("add eight") < q[0] < vv.heard("predict"))
    for voice in ("take01", "take02"):
        vv = VOICES["10_1"][voice]
        total = sum(y - x for x, y in vv.spans)
        s3_share = vv.length * sum(y - x for x, y in vv.spans[:3]) / total
        s4_share = vv.length * sum(y - x for x, y in vv.spans[:4]) / total
        say(f"     {voice}: if the kit's onsets were character shares instead of ASR: sentence 3 {s3_share:.2f}, sentence 4 {s4_share:.2f}")
        check(f"{voice}: index 3 is early and index 4 is right under either rule", s3_share < vv.heard("add eight") < s4_share < vv.heard("predict"))
    bad = fixed.replace('k.until_sentence(stage, "10_1", 4)?;', 'k.until_sentence(stage, "10_1", 5)?;', 1)
    check("control: index 5 would be caught, 10_1 has five sentences", any("sentence 5" in p for p in order_problems(bad)))

    say("\n== STACK: the second correction, the label that names its group")
    heads3, hunks3 = parse_patch(CORRECTION2)
    second = apply(result, hunks3)
    check("it applies at its stated line", "U(12): 5, 7 and 11 each have order 2" in second and len(hunks3) == 1)
    check("reverse returns the patch's result", sha(unapply(second, hunks3)) == RESULT_SHA)
    check("the patch's ord(5)=2 row is gone, and 10_6's ord(k) rows stay", "ord}(5)" not in second and "ord}(k)" in second)
    check("the row is still made once and shown once", (len(re.findall(r"let unit_orders = ", second)), len(re.findall(r"fade\(stage, &\[unit_orders\], true\)", second))) == (1, 1))
    ok, detail = rustfmt_ok(second, "second")
    check("rustfmt passes", ok, detail)
    check("waits remain legal; hold still at 453", order_problems(second) == [] and [i + 1 for i, r in enumerate(second.split("\n")) if r.strip() == marker] == [453])
    check("no test or phrase table changes", tests_in(second) == tests_in(result) and phrases_table(second)[0] == 13)
    both_a = apply(stacked, hunks3, exact=False)
    both_b = apply(second, hunks2, exact=False)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        target = tmp / REL
        target.parent.mkdir(parents=True)
        target.write_text(result, encoding="utf-8")
        run = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", str(CORRECTION2.resolve())], cwd=tmp, capture_output=True, text=True)
        check("patch(1) --fuzz=0 agrees", run.returncode == 0 and target.read_text(encoding="utf-8") == second)
    check("the two corrections commute", both_a == both_b)
    ok, detail = rustfmt_ok(both_a, "both")
    check("rustfmt passes on both together", ok, detail)
    say("     patch + question correction  " + sha(stacked))
    say("     patch + label correction     " + sha(second))
    say("     patch + both                 " + sha(both_a))
    w2 = law("U(12): 5, 7 and 11 each have order 2", 28)
    h2 = text_h_26 * 28 / 26
    x2 = (2.5 - w2 / 2, 2.5 + w2 / 2)
    y2 = (-0.5 - h2 / 2, -0.5 + h2 / 2)
    say(f"     label width {w2:.2f}, x {x2[0]:.2f}..{x2[1]:.2f}, y {y2[0]:.2f}..{y2[1]:.2f}")
    check("its slot is empty today", ink_box(560, x2[0], y2[0] + 0.02, x2[1], y2[1] - 0.02) is None)
    check(f"it clears squares by {(sq[1] - y2[1]) * 90:.0f} px and no by {(y2[0] - no[3]) * 90:.0f} px", (sq[1] - y2[1]) * 90 >= 15 and (y2[0] - no[3]) * 90 >= 15)
    check("it clears the table and stays in the safe frame", x2[0] > tab[2] + 0.5 and x2[1] < 6.96)
else:
    say("     (not staged yet)")

say(f"\nframes decoded and measured, not viewed: {len(cache)}; {sorted(cache)}")
say("\n== canonical")
check("canonical cyclic.rs still equals the base at the end", sha(SOURCE.read_bytes()) == BASE_SHA)
check("canonical script still unchanged", sha(SCRIPT.read_bytes()) == SCRIPT_SHA)
check("canonical map still unchanged", sha(MAP.read_bytes()) == MAP_SHA)
say("")
say(f"failures: {failed}")
(HERE / "verify.out").write_text("\n".join(out) + "\n", encoding="utf-8")
sys.exit(1 if failed else 0)
