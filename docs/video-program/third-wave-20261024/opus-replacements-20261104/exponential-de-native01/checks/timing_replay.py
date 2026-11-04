"""Replay chapters 04, 05 and 06 against the Kit's scratch-voice timing, at speech factors 0.8, 1.0 and 1.25.

Usage from the repo root:  python3 -I -B timing_replay.py SCRATCH_DIR
The sentence splitter and onset rule are not reimplemented: `sentences` and `estimated_starts` are cut verbatim from
crates/explainer_kit/src/narration.rs and compiled with rustc into SCRATCH_DIR. The block lists below are my own
transcription of each construct(), checked against the source's literal hooks. Waits, holds and breaths follow
narration.rs under LineGaps::Constant. Phrase hooks replay src/phrase.rs's fallback: a share by characters of the
measured sentence. Word marks on real takes will move them.

Scratch timing is not Oliver's timing. Nothing here is a listening or picture claim.
"""
import os
import re
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
WORK = S + "/timing"
os.makedirs(WORK, exist_ok=True)
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
L, O = HERE + "/candidate/limit01/", HERE + "/candidate/only01/"
SLICES = {"limit": L + "script-slice.tsv", "only": O + "script-slice.tsv"}
BREATH = 0.6
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def cut_fn(source, name):
    """A whole top-level fn, by brace matching from its signature."""
    at = source.index(f"fn {name}(")
    start = source.rfind("\n", 0, at) + 1
    depth = 0
    for i in range(at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    raise ValueError(name)


src = open("crates/explainer_kit/src/narration.rs", encoding="utf-8").read()
wps = re.search(r"pub const SCRATCH_WPS: f64 = ([0-9.]+);", src).group(1)
assert float(re.search(r"pub const BREATH: f64 = ([0-9.]+);", src).group(1)) == BREATH
harness = f"""// Cut verbatim from narration.rs by timing_replay.py.
{cut_fn(src, "sentences")}
{cut_fn(src, "estimated_starts")}
fn main() {{
    let text = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    for line in text.lines() {{
        let Some((id, t)) = line.split_once('\\t') else {{ continue }};
        let words = t.split_whitespace().count() as f64;
        let speech = words / {wps};
        let parts = sentences(t);
        for (j, (s, at)) in parts.iter().zip(estimated_starts(t, speech)).enumerate() {{
            println!("{{id}}\\t{{j}}\\t{{at:.6}}\\t{{speech:.6}}\\t{{s}}");
        }}
    }}
}}
"""
hp = os.path.join(WORK, "sentences_harness.rs")
with open(hp, "w", encoding="utf-8") as out:
    out.write(harness)
binp = os.path.join(WORK, "sentences_harness")
r = subprocess.run(["rustc", "--edition", "2024", "-A", "dead_code", "-o", binp, hp], capture_output=True, text=True, cwd=ROOT)
assert r.returncode == 0, r.stderr

STARTS, SPEECH, TEXT = {}, {}, {}
for name, path in SLICES.items():
    o = subprocess.run([binp, path], capture_output=True, text=True, check=True).stdout
    for line in o.splitlines():
        cue, j, at, speech, sentence = line.split("\t", 4)
        STARTS.setdefault(cue, []).append(float(at))
        TEXT.setdefault(cue, []).append(sentence)
        SPEECH[cue] = float(speech)
print(f"splitter: real sentences() and estimated_starts() from narration.rs; SCRATCH_WPS {wps}")
for cue in TEXT:
    print(f"  {cue}: {len(TEXT[cue])} sentences, {SPEECH[cue]:.2f} s of speech at factor 1.0")


class Sim:
    def __init__(self, factor):
        self.f, self.t, self.busy, self.said, self.events, self.gaps = factor, 0.0, 0.0, {}, [], {}
        self.hooks, self.voice, self.marks, self.phrases = [], {}, {}, []

    def length(self, cue):
        return SPEECH[cue] * self.f + 0.4  # scratch: speech + 0.4

    def say(self, cue):
        self.t = max(self.t, self.busy + BREATH)
        self.said[cue] = self.t
        self.busy = self.t + self.length(cue)
        self.voice[cue] = (self.t, self.busy)

    def until(self, cue, j, shows=""):
        at = self.said[cue] + STARTS[cue][j] * self.f
        late = 0.0 if at > self.t else self.t - at
        self.t = max(self.t, at)
        self.hooks.append((cue, j, late, shows))

    def phrase(self, cue, phrase, shows=""):
        j = next(i for i, sent in enumerate(TEXT[cue]) if phrase in sent)
        sentence = TEXT[cue][j]
        begin = self.said[cue] + STARTS[cue][j] * self.f
        end = self.said[cue] + (STARTS[cue][j + 1] * self.f if j + 1 < len(STARTS[cue]) else self.length(cue))
        at = begin + (end - begin) * sentence.index(phrase) / len(sentence)
        late = 0.0 if at > self.t else self.t - at
        self.t = max(self.t, at)
        self.phrases.append((cue, j, phrase, late, shows))

    def play(self, seconds, label):
        self.events.append((self.t, self.t + seconds, label))
        self.t += seconds

    def hold(self):
        self.t = max(self.t, self.busy + BREATH)

    def breath(self, seconds):
        self.hold()
        start = self.t
        self.t += seconds
        self.busy = start + seconds - BREATH
        return start

    def mark(self, name):
        self.marks[name] = self.t


def expand(s):
    """candidate/limit01/tiny_multipliers.rs, TinyMultipliers"""
    s.say("04_1"); s.play(0.5, "title, axes, four steps")
    s.until("04_1", 1, "height, width, rise, h + h/4"); s.play(0.4, "one step taken apart")
    s.until("04_1", 2, "5h/4"); s.play(0.35, "as a multiple")
    s.until("04_1", 3, "strip, hopper walks"); s.play(0.35, "cells, x5/4"); s.play(0.35, "hopper")
    for i in range(4):
        s.play(0.35, f"hop {i}")
    s.until("04_1", 4, "(5/4)^4, value"); s.play(0.35, "product")
    s.hold()

    s.say("04_2"); s.play(0.3, "working clears"); s.play(0.6, "cells gold, one unit of time")
    s.until("04_2", 1, "y'=xy"); s.play(0.35, "rule")
    s.until("04_2", 2, "1+x/n in the cells"); s.play(0.4, "letters")
    s.until("04_2", 3, "(1+x/n)^n"); s.play(0.35, "power")
    s.until("04_2", 4, "rate one, first row"); s.play(0.35, "rate one")
    s.until("04_2", 5, "cells refine, table fills"); s.play(0.3, "labels clear")
    for n in (8, 16, 64, 1024):
        s.play(0.45, f"{n} cells")
    s.play(0.35, "e")
    s.hold()

    s.say("04_3"); s.play(0.3, "clears"); s.play(0.4, "graph, curve, cells")
    s.until("04_3", 1, "endpoint sweeps both ways, read-outs"); s.play(0.7, "to 3/2"); s.play(2.4, "to -3/2"); s.play(0.35, "read-outs")
    s.until("04_3", 2, "not a proof"); s.play(0.35, "not a proof")
    s.until("04_3", 3, "why this limit"); s.play(0.35, "why")
    s.until("04_3", 4, "power"); s.play(0.35, "power")
    s.hold()

    s.say("04_4"); s.play(0.3, "clears"); s.play(0.35, "offer")
    s.until("04_4", 2, "sixteen strips"); s.play(1.2, "strips")
    s.until("04_4", 3, "powers and ways"); s.play(0.35, "labels")
    s.until("04_4", 4, "x squared framed"); s.play(0.35, "frame")
    s.until("04_4", 5, "coefficients, series row"); s.play(0.4, "rows")
    s.hold()

    s.say("04_5"); s.play(0.3, "clears"); s.play(0.4, "name, empty strip")
    s.until("04_5", 1, "first pick, second pick"); s.play(0.3, "pick 1"); s.play(0.3, "pick 2")
    s.until("04_5", 2, "falling product"); s.play(0.4, "picks")
    s.until("04_5", 3, "the other order, k!, binomial"); s.play(0.4, "recount"); s.play(0.4, "choose")
    s.until("04_5", 4, "binomial pulses"); s.play(0.9, "pulse")
    s.hold()

    s.say("04_6"); s.play(0.3, "clears"); s.play(0.35, "shared")
    s.until("04_6", 1, "chain"); s.play(0.35, "chain")
    s.until("04_6", 2, "chain pulses"); s.play(0.8, "pulse")
    s.until("04_6", 3, "worked"); s.play(0.35, "worked")
    s.until("04_6", 4, "k>n zero"); s.play(0.35, "zero")
    s.hold()


def whole(s):
    """candidate/limit01/tiny_multipliers.rs, WholeSum"""
    s.say("05_1"); s.play(0.4, "title, axis, bars, readings")
    s.until("05_1", 1, "full line"); s.play(0.35, "full")
    s.until("05_1", 2, "n climbs"); s.play(0.6, "16"); s.play(0.6, "64")
    s.until("05_1", 3, "n climbs"); s.play(0.6, "256")
    s.until("05_1", 4, "n climbs, readings pulse"); s.play(0.6, "1024"); s.play(0.8, "pulse")
    s.until("05_1", 5, "question"); s.play(0.35, "question")
    q_end = s.t
    s.hold(); s.mark("predict 04"); s.gaps["predict 05_1"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question clears")

    s.say("05_2"); s.mark("first answer 04"); s.play(0.35, "not enough")
    s.until("05_2", 1, "chains clear"); s.play(0.3, "clears")
    s.until("05_2", 2, "n terms of 1/n"); s.play(0.4, "warning bars")
    s.until("05_2", 3, "n climbs, column three falls"); s.play(0.6, "16"); s.play(0.6, "40")
    s.until("05_2", 4, "tower, total one"); s.play(0.4, "tower")
    s.until("05_2", 5, "100, outward"); s.play(0.6, "100"); s.play(0.35, "outward")
    s.hold()

    s.say("05_3"); s.play(0.3, "clears"); s.play(0.4, "ceilings, indices, axis, x")
    s.until("05_3", 1, "chain range"); s.play(0.35, "range")
    s.phrase("05_3", "and after that", "chain empty"); s.play(0.35, "empty")
    s.until("05_3", 2, "bars rise, bound"); s.play(0.9, "bars"); s.play(0.35, "bound, n")
    s.until("05_3", 3, "n climbs under fixed lids")
    for n in (16, 64, 1024):
        s.play(0.5, f"{n}")
    s.until("05_3", 4, "no ceiling"); s.play(0.35, "no ceiling")
    s.until("05_3", 5, "harmonic"); s.play(0.35, "harmonic")
    s.hold()

    s.say("05_4"); s.play(0.3, "card clears"); s.play(0.5, "n=16, distance")
    s.until("05_4", 1, "gaps solid, sum"); s.play(0.4, "gaps")
    s.until("05_4", 2, "aim"); s.play(0.35, "aim")
    s.until("05_4", 3, "cut at 4"); s.play(0.35, "cut")
    s.until("05_4", 4, "inset, tail too big"); s.play(0.4, "inset")
    s.hold()

    s.say("05_5")
    s.phrase("05_5", "so push", "cut moves to 8"); s.play(0.5, "cut moves")
    s.until("05_5", 1, "head closes, three rows")
    for n in (64, 256, 1024):
        s.play(0.5, f"{n}")
    s.until("05_5", 2, "tail row pulses"); s.play(0.5, "pulse")
    s.phrase("05_5", "then make", "head row pulses"); s.play(0.5, "pulse")
    s.until("05_5", 3, "total below aim"); s.play(0.35, "total")
    s.hold()

    s.say("05_6"); s.play(0.3, "card clears"); s.play(0.5, "n=16, indices step down")
    s.until("05_6", 1, "live, x sweeps to -3/2"); s.play(0.25, "relabel"); s.play(1.5, "sweep")
    s.until("05_6", 2, "lower ceilings, signs"); s.play(0.5, "below")
    s.until("05_6", 3, "same sizes"); s.play(0.35, "same")
    s.until("05_6", 4, "bound, both distances"); s.play(0.35, "rows")
    s.hold()

    s.say("05_7"); s.play(0.3, "clears"); s.play(0.35, "headers")
    s.until("05_7", 1, "one cell"); s.play(0.35, "row 1")
    s.until("05_7", 2, "two cells"); s.play(0.35, "row 2")
    s.phrase("05_7", "and three", "three cells"); s.play(0.35, "row 3")
    s.until("05_7", 3, "the rest, E(-3)"); s.play(0.3, "row 4")
    for n in (16, 64, 1024):
        s.play(0.3, f"{n}")
    s.play(0.35, "E(-3)")
    s.until("05_7", 4, "brackets"); s.play(0.4, "notes")
    s.until("05_7", 5, "same algebra"); s.play(0.35, "same")
    s.until("05_7", 6, "no climb"); s.play(0.35, "no climb")
    s.hold()

    s.say("05_8"); s.play(0.3, "clears"); s.play(0.4, "graph, limit, every real x")
    s.until("05_8", 1, "later cut, more cells"); s.play(0.35, "bigger")
    s.until("05_8", 2, "x=1, e"); s.play(0.35, "special case")
    s.until("05_8", 3, "time swap"); s.play(0.35, "swap")
    s.until("05_8", 4, "walks both ways"); s.play(0.5, "coarse"); s.play(0.5, "fine")
    s.until("05_8", 5, "debt paid"); s.play(0.35, "debt"); s.play(0.5, "green")
    s.hold()


def only(s):
    """candidate/only01/only_solution.rs, in the file's order"""
    s.say("06_1"); s.play(0.4, "title, axes, E, rule")
    s.until("06_1", 1, "the curve in question"); s.play(0.4, "?")
    s.until("06_1", 2, "proved"); s.play(0.35, "proved")
    s.until("06_1", 3, "not yet, the debt"); s.play(0.4, "not yet")
    s.hold()

    s.say("06_2"); s.play(0.3, "rule card clears")
    s.until("06_2", 1, "y E(-x)"); s.play(0.35, "test")
    s.until("06_2", 2, "d/dx"); s.play(0.35, "derivative")
    s.until("06_2", 3, "first part"); s.play(0.35, "first")
    s.until("06_2", 4, "second part, chain rule"); s.play(0.4, "second")
    s.until("06_2", 5, "result"); s.play(0.35, "result")
    s.hold()

    s.say("06_3"); s.play(0.3, "clears"); s.play(0.4, "formula rises"); s.play(0.4, "product axes")
    s.until("06_3", 1, "degree 2"); s.play(0.4, "low")
    s.until("06_3", 2, "its product, slope"); s.play(0.4, "low product")
    s.until("06_3", 3, "degree 4, its product, slope"); s.play(0.4, "high")
    s.until("06_3", 4, "never zero, level when"); s.play(0.35, "never")
    s.until("06_3", 5, "the rule"); s.play(0.35, "rule")
    s.until("06_3", 6, "question"); s.play(0.35, "question")
    q_end = s.t
    s.hold(); s.mark("predict 05"); s.gaps["predict 06_3"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question clears")

    s.say("06_4"); s.mark("first answer 05"); s.play(0.4, "level line")
    s.until("06_4", 1, "up arrow"); s.play(0.35, "plus")
    s.phrase("06_4", "and the other", "down arrow"); s.play(0.35, "minus")
    s.until("06_4", 2, "cancel"); s.play(0.5, "cancel")
    s.until("06_4", 3, "constant"); s.play(0.35, "constant")
    s.phrase("06_4", "and at zero", "value one"); s.play(0.35, "at zero")
    s.until("06_4", 4, "equals one"); s.play(0.35, "equals")
    s.hold()

    s.say("06_5"); s.play(0.3, "clears"); s.play(0.4, "panel, equations")
    s.until("06_5", 1, "grid fact"); s.play(0.35, "grid")
    s.phrase("06_5", "so the left side", "y=E"); s.play(0.35, "collapsed")
    s.until("06_5", 2, "curve lands on E"); s.play(1.0, "lands")
    s.until("06_5", 3, "both ways"); s.play(0.35, "both")
    s.until("06_5", 4, "pulse"); s.play(0.9, "pulse")
    s.hold()

    s.say("06_6"); s.play(0.3, "clears"); s.play(0.35, "title")
    s.until("06_6", 1, "series box"); s.play(0.4, "series")
    s.until("06_6", 2, "E'=E"); s.play(0.4, "derivative")
    s.until("06_6", 3, "E(x)E(-x)=1"); s.play(0.4, "grid")
    s.until("06_6", 4, "the result"); s.play(0.4, "result")
    s.until("06_6", 5, "two arrows, tools"); s.play(0.4, "tools")
    s.until("06_6", 6, "not used, struck"); s.play(0.4, "struck")
    s.until("06_6", 7, "arrows pulse"); s.play(0.9, "pulse")
    s.hold()

    s.say("06_7"); s.play(0.3, "clears"); s.play(0.4, "title, axes, near piece")
    s.until("06_7", 1, "gap, far piece"); s.play(0.4, "far")
    s.until("06_7", 2, "obeys, read-outs"); s.play(0.35, "reads")
    s.until("06_7", 3, "one interval"); s.play(0.35, "interval")
    s.phrase("06_7", "with a derivative", "a derivative everywhere"); s.play(0.35, "derivative")
    s.hold()

    s.say("06_8"); s.play(0.3, "clears"); s.play(0.35, "title")
    s.until("06_8", 1, "rule, product"); s.play(0.35, "rule"); s.play(0.35, "product")
    s.until("06_8", 2, "test"); s.play(0.35, "test")
    s.phrase("06_8", "so y of t", "result"); s.play(0.35, "result")
    s.until("06_8", 3, "curves"); s.play(0.9, "curves")
    s.until("06_8", 4, "units"); s.play(0.35, "units")
    s.until("06_8", 5, "the model question"); s.play(0.35, "question")
    s.hold()

    s.say("06_9"); s.play(0.3, "clears"); s.play(0.35, "title"); s.play(0.5, "doors"); s.play(0.5, "arrows"); s.play(0.35, "E(x)=e^x")
    s.until("06_9", 1, "the structure"); s.play(0.5, "fits")
    s.until("06_9", 2, "debts"); s.play(0.5, "debts")
    s.hold()


def both_d(s):
    expand(s)
    whole(s)


SOURCES = {"tiny_multipliers": L + "tiny_multipliers.rs", "only_solution": O + "only_solution.rs"}
print("\n=== my transcription against the source's literal calls")
for name, scene in (("tiny_multipliers", both_d), ("only_solution", only)):
    code = open(SOURCES[name], encoding="utf-8").read().split("#[cfg(test)]")[0]
    says_src = re.findall(r'\.say\(stage, "(\d\d_\d+)"\)', code)
    literal = [(c, int(j)) for c, j in re.findall(r'until_sentence\(stage, "(\d\d_\d+)", (\d+)\)', code)]
    literal_p = re.findall(r'until_phrase\(k, stage, "(\d\d_\d+)", "([^"]+)"\)', code)
    probe = Sim(1.0)
    scene(probe)
    replay = [(c, j) for c, j, _, _ in probe.hooks]
    replay_p = [(c, p) for c, _, p, _, _ in probe.phrases]
    ck(says_src == list(probe.said), f"{name}: cues in source order", str(len(says_src)))
    ck(replay == literal, f"{name}: every sentence hook replayed, in source order", str(len(replay)))
    ck(replay_p == literal_p, f"{name}: every phrase hook replayed, in source order", str(replay_p))

SCENES = (("04_tiny_multipliers", expand), ("05_whole_sum", whole), ("06_only_solution", only))
encode = open("crates/explainer_kit/src/encode.rs", encoding="utf-8").read()
raw_gib = int(re.search(r"pub const MAX_OUTPUT_BYTES: u64 = (\d+) << 30;", encode).group(1))
# 4K60 raw NV12, 1.5 bytes a pixel: the skill's own 64 GiB, about 92 s, is this same rate.
RAW_SECONDS = (raw_gib << 30) / (3840 * 2160 * 1.5 * 60)
print(f"\nkit raw-output limit: {raw_gib} GiB, which is {RAW_SECONDS:.0f} s of 4K60")
all_totals = {}
for name, scene in SCENES:
    print(f"\n=== {name}")
    totals, runs = {}, {}
    for f in (0.8, 1.0, 1.25):
        s = Sim(f)
        scene(s)
        totals[f], runs[f] = s.t, s
    all_totals[name] = totals
    print("total seconds at 0.8 / 1.0 / 1.25: " + " / ".join(f"{totals[f]:.2f}" for f in (0.8, 1.0, 1.25)))
    s = runs[1.0]
    print("hooks, with lateness at 0.8 / 1.0 / 1.25 (0 means the picture waited for the sentence):")
    late_rows = 0
    for idx, (cue, j, late, shows) in enumerate(s.hooks):
        l8, l125 = runs[0.8].hooks[idx][2], runs[1.25].hooks[idx][2]
        flag = "  LATE" if max(l8, late, l125) > 0.25 else ""
        late_rows += bool(flag)
        print(f"  {cue} s{j}: {l8:4.2f} {late:4.2f} {l125:4.2f}  {shows}  <- {TEXT[cue][j][:48]!r}{flag}")
    for idx, (cue, j, phrase, late, shows) in enumerate(s.phrases):
        l8, l125 = runs[0.8].phrases[idx][3], runs[1.25].phrases[idx][3]
        flag = "  LATE" if max(l8, late, l125) > 0.25 else ""
        late_rows += bool(flag)
        print(f"  {cue} s{j} phrase {phrase!r}: {l8:4.2f} {late:4.2f} {l125:4.2f}  {shows}{flag}")
    print(f"hooks late by over 0.25 s at some factor: {late_rows}")
    for key, (start, q_end, _busy) in s.gaps.items():
        print(f"  {key}: answer withheld from question landing for {start + 4.0 + 0.3 - q_end:.2f} s")
    print("static stretches of 6 s or more at factor 1.0:")
    last = 0.0
    ev = sorted(s.events)
    for a, b, label in ev:
        if a - last >= 6.0:
            print(f"  {a - last:5.2f} s, {last:7.2f} to {a:7.2f}, before {label!r}")
        last = max(last, b)
    if s.t - last >= 6.0:
        print(f"  {s.t - last:5.2f} s, {last:7.2f} to {s.t:7.2f}, end of chapter")
    print("seconds from each cue's start to its first visual change, factor 1.0, and the cue's length:")
    for c, (x, y) in s.voice.items():
        first = next((a for a, b, label in ev if a >= x - 1e-9), None)
        print(f"  {c}: {first - x:5.2f} s of {y - x:5.2f} s")
    ck(totals[1.25] < RAW_SECONDS, f"{name}: under the kit's raw limit even at factor 1.25", f"{totals[1.25]:.0f} s of {RAW_SECONDS:.0f} s")

print("\n=== order claims, at each factor")
one_chapter = Sim(1.25)
both_d(one_chapter)
print(f"the two D chapters as one, at factor 1.25: {one_chapter.t:.0f} s, against {RAW_SECONDS:.0f} s: this is why they are two")
for f in (0.8, 1.0, 1.25):
    s = Sim(f)
    whole(s)
    start, q_end, _ = s.gaps["predict 05_1"]
    ck(s.marks["first answer 04"] >= start + 4.0, f"factor {f}: 05_2's answer comes after the think gap", f"{s.marks['first answer 04'] - q_end:.2f} s after the question")
    s = Sim(f)
    only(s)
    start, q_end, _ = s.gaps["predict 06_3"]
    ck(s.marks["first answer 05"] >= start + 4.0, f"factor {f}: 06_4's answer comes after the think gap", f"{s.marks['first answer 05'] - q_end:.2f} s after the question")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
