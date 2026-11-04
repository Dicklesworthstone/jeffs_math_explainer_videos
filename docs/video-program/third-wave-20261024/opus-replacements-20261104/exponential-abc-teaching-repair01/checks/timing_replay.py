"""Replay the three repaired scenes against the Kit's scratch-voice timing, at speech factors 0.8, 1.0 and 1.25.

Usage from the repo root:  python3 -I -B timing_replay.py SCRATCH_DIR
The sentence splitter and onset rule are not reimplemented: `sentences` and `estimated_starts` are cut verbatim
from crates/explainer_kit/src/narration.rs and compiled with rustc into SCRATCH_DIR. The block lists below are my
own transcription of each construct(), checked against the source's literal hooks. Waits, holds and breaths follow
narration.rs (say, hold, window, wait_gap) under LineGaps::Constant.

Scratch timing is not Oliver's timing. Nothing here is a listening or picture claim.
"""
import os
import re
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
D = "docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/candidate/"
SLICES = {
    "rate": D + "rate02/script-slice.tsv",
    "series": D + "series01/script-slice.tsv",
    "product": D + "product01/script-slice.tsv",
}
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
hp = os.path.join(S, "sentences_harness.rs")
with open(hp, "w", encoding="utf-8") as out:
    out.write(harness)
binp = os.path.join(S, "sentences_harness")
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
        self.hooks, self.voice, self.marks, self.clauses = [], {}, {}, []

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

    def clause(self, cue, j, phrase, shows=""):
        """The scenes' until_clause: the sentence's window, shared out by the sentence's characters."""
        sentence = TEXT[cue][j]
        begin = self.said[cue] + STARTS[cue][j] * self.f
        end = self.said[cue] + (STARTS[cue][j + 1] * self.f if j + 1 < len(STARTS[cue]) else self.length(cue))
        at = begin + (end - begin) * sentence.index(phrase) / len(sentence)
        late = 0.0 if at > self.t else self.t - at
        self.t = max(self.t, at)
        self.clauses.append((cue, j, phrase, late, shows))

    def play(self, seconds, label):
        self.events.append((self.t, self.t + seconds, label))
        self.t += seconds

    def wait(self, seconds):
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


def rate(s):
    """candidate/rate02/rate_law.rs: only 01_5 differs from rate02"""
    s.say("01_1")
    s.play(0.6, "frame, axes, start dots")
    s.until("01_1", 2, "constant rate = 1"); s.play(0.4, "left rule")
    s.until("01_1", 3, "rate = current height"); s.play(0.4, "right rule")
    s.until("01_1", 4, "first slopes and read-outs"); s.play(0.4, "first slopes")
    s.until("01_1", 5, "question"); s.play(0.4, "question")
    q_end = s.t
    s.hold(); s.gaps["predict 01_1"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question fades")
    s.say("01_2"); s.play(0.6, "first step")
    s.until("01_2", 2, "height read again"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.hold(); s.say("01_3"); s.play(0.6, "second step")
    s.until("01_3", 1, "reading morph"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.until("01_3", 2, "third step"); s.play(0.6, "third step"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.until("01_3", 3, "fourth step"); s.play(0.6, "fourth step"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.until("01_3", 4, "straight steps: an approximation"); s.play(0.4, "approximation")
    s.until("01_3", 5, "last step read-out"); s.play(0.4, "last step read-out")
    s.hold(); s.say("01_4")
    s.play(0.3, "slope tick fades"); s.play(0.3, "endpoint read-out")
    s.until("01_4", 2, "old drawing greys"); s.play(0.4, "old drawing greys")
    s.until("01_4", 3, "8 steps")
    s.play(0.3, "read-out"); s.play(8 * 0.15, "8 steps"); s.wait(1.0)
    s.until("01_4", 4, "grey, then 16 steps"); s.play(0.4, "grey")
    s.play(0.3, "read-out"); s.play(16 * 0.15, "16 steps"); s.wait(1.0)
    s.hold(); s.say("01_5")
    s.play(0.4, "y'=y")
    s.until("01_5", 2, "promise"); s.play(0.4, "promise")
    s.hold()


def series(s):
    """candidate/series01/factorials.rs, in the file's order"""
    s.say("02_1"); s.play(0.35, "title, rule")
    s.until("02_1", 1, "polynomial, no answer"); s.play(0.35, "polynomial")
    s.until("02_1", 2, "boxes and headers, coefficients fall"); s.play(0.3, "boxes"); s.play(0.6, "coefficients fall")
    s.until("02_1", 3, "derivative row, terms move left"); s.play(0.35, "derivative label")
    for n in range(1, 5):
        s.play(0.35, f"copy {n}"); s.play(0.6, f"moves left {n}"); s.play(0.35, f"link {n}")
    s.until("02_1", 4, "frame and question"); s.play(0.35, "frame, question")
    q_end = s.t
    s.hold(); s.gaps["predict 02_1"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question clears")

    s.say("02_2"); s.mark("first answer")
    s.play(0.4, "frame, a = 1"); s.play(0.5, "value rises")
    s.until("02_2", 1, "pulse b term and 2b"); s.play(0.8, "pulse")
    s.until("02_2", 2, "2b = 1"); s.play(0.4, "2b = 1")
    s.until("02_2", 3, "b = 1/2, rises"); s.play(0.5, "b = 1/2")
    s.until("02_2", 4, "pulse c term and 3c"); s.play(0.8, "pulse")
    s.until("02_2", 5, "3c = 1/2"); s.play(0.4, "3c = 1/2"); s.wait(0.5); s.play(0.5, "c = 1/6")

    s.say("02_3"); s.play(0.4, "4d = 1/6"); s.play(0.5, "d = 1/24")
    s.until("02_3", 1, "derivative row leaves, chain"); s.play(0.3, "clears"); s.play(0.4, "chain and recurrence")
    s.until("02_3", 2, "running products gather"); s.play(0.2, "name")
    for i in range(5):
        s.play(0.42, f"product {i}")
    s.until("02_3", 3, "products become k!"); s.mark("first factorial"); s.play(0.6, "k! morph")
    s.until("02_3", 4, "factorials rise into polynomial, axes"); s.play(0.6, "rise"); s.play(0.4, "axes")
    for n in range(5):
        if n == 3:
            s.until("02_3", 5, "note, degree three")
        s.play(0.5, f"curve {n}, term {n}"); s.wait(0.4)
    s.hold()

    s.say("02_4"); s.play(0.3, "chart clears"); s.play(0.3, "y'=")
    s.until("02_4", 1, "derivatives land left, hole"); s.play(0.7, "derivatives"); s.play(0.35, "hole, caption")
    s.until("02_4", 2, "two more terms, dots")
    for n in (5, 6):
        s.play(0.35, f"term {n}"); s.play(0.6, f"fills hole {n}")
    s.play(0.35, "dots")
    s.until("02_4", 3, "dots pulse, needs a proof"); s.play(0.6, "pulse")
    s.until("02_4", 4, "two obligations"); s.play(0.35, "sum exists"); s.clause("02_4", 4, "and that differentiating", "term by term"); s.play(0.35, "term by term")
    s.hold(); s.play(0.3, "clears")

    s.say("02_5"); s.play(0.4, "interval, R, dot")
    s.until("02_5", 1, "bars, ratio row"); s.play(1.0, "bars grow"); s.play(0.35, "ticks, ratios, formula")
    s.until("02_5", 2, "ratios from one half green"); s.play(0.6, "morph, green")
    s.until("02_5", 3, "zoom, ceilings, sum, bound")
    s.play(0.6, "zoom"); s.play(0.4, "ceilings"); s.play(0.35, "sum, total, read-outs")
    s.until("02_5", 4, "bars one column left"); s.play(0.7, "shift"); s.play(0.35, "derivative formula")
    s.until("02_5", 5, "uniform card, input sweeps"); s.play(0.35, "card, ceilings"); s.play(3.2, "sweep")
    s.hold()

    s.say("02_6"); s.play(0.3, "clears"); s.play(0.35, "E row with dots")
    s.until("02_6", 1, "E' row, frame walks, definition")
    s.play(0.7, "shifted row"); s.play(0.2, "frame")
    for i in range(6):
        s.play(0.22, f"walk {i}")
    s.play(0.3, "frame clears"); s.play(0.35, "definition")
    s.until("02_6", 2, "y=E solves"); s.play(0.35, "solves")
    s.until("02_6", 3, "three debts")
    for phrase in ("why adding", "why the stepped", "why it is"):
        s.clause("02_6", 3, phrase, "debt"); s.play(0.35, "debt")
    s.until("02_6", 4, "debts lit"); s.play(0.9, "lit")
    s.hold()


def product(s):
    """candidate/product01/product_grid.rs, in the file's order"""
    s.say("03_1"); s.play(0.35, "title")
    s.until("03_1", 2, "edges and headers"); s.play(0.4, "edge labels")
    for i in range(8):
        s.play(0.35, f"header {i}")
    s.until("03_1", 3, "rules and cells"); s.play(0.4, "rules")
    for i in range(16):
        s.play(0.2, f"cell {i}")
    s.until("03_1", 5, "corner card"); s.play(0.35, "corner card")
    s.hold(); s.say("03_2")
    s.play(0.35, "a row framed"); s.clause("03_2", 0, "collect them", "row becomes diagonal"); s.play(0.35, "frame becomes a diagonal line")
    s.until("03_2", 1, "line hides"); s.play(0.25, "line hides")
    s.until("03_2", 1, "degree 0"); s.play(0.3, "degree 0 lit"); s.play(0.35, "total 1")
    s.until("03_2", 2, "degree 1"); s.play(0.3, "degree 1 lit")
    s.until("03_2", 3, "total x+y"); s.play(0.35, "total x+y")
    s.until("03_2", 4, "degree 2"); s.play(0.3, "degree 2 lit")
    s.until("03_2", 5, "question"); s.play(0.35, "question")
    q_end = s.t
    s.hold(); s.gaps["predict 03_2"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.25, "question hides")
    s.say("03_3"); s.play(0.35, "sum of three cells")
    s.until("03_3", 1, "factor one half"); s.play(0.6, "morph")
    s.until("03_3", 3, "(x+y)^2/2"); s.play(0.6, "morph"); s.play(0.35, "total")
    s.hold(); s.say("03_4"); s.play(0.3, "degree 3 lit"); s.play(0.35, "total")
    s.until("03_4", 1, "binomial"); s.play(0.35, "binomial")
    s.until("03_4", 2, "identity"); s.play(0.35, "identity")
    s.until("03_4", 3, "complete diagonals card"); s.play(0.35, "card")
    s.hold(); s.say("03_5"); s.play(0.25, "card hides")
    s.until("03_5", 1, "permission"); s.play(0.35, "permission")
    s.until("03_5", 2, "diagonals and totals share colours, series"); s.play(0.3, "linked colours"); s.play(0.35, "series")
    s.until("03_5", 3, "law"); s.play(0.35, "law")
    s.hold(); s.play(0.5, "everything fades")
    s.say("03_6")
    s.until("03_6", 1, "E(x)E(-x)"); s.play(0.35, "non-vanishing")
    s.until("03_6", 2, "half law"); s.play(0.35, "half law")
    s.until("03_6", 3, "> 0"); s.play(0.5, "morph")
    s.until("03_6", 4, "E'=E>0"); s.play(0.35, "E'=E>0")
    s.hold()
    s.say("03_7"); s.play(0.3, "03_6 fades"); s.play(0.35, "empty input line")
    s.until("03_7", 1, "e = E(1)"); s.play(0.35, "name")
    s.until("03_7", 2, "E(2), E(3), whole marks"); s.play(0.4, "whole marks")
    s.until("03_7", 3, "roots, reciprocals, solid line")
    s.play(0.4, "root, fractions")
    s.clause("03_7", 3, "reciprocals", "reciprocal, negatives"); s.play(0.4, "reciprocal, negatives")
    s.clause("03_7", 3, "continuity", "solid line"); s.play(0.9, "solid line")
    s.until("03_7", 4, "E(x)=e^x"); s.play(0.35, "E(x)=e^x")
    s.hold()


SOURCES = {"rate_law": D + "rate02/rate_law.rs", "factorials": D + "series01/factorials.rs", "product_grid": D + "product01/product_grid.rs"}
SCENES = (("rate_law", rate), ("factorials", series), ("product_grid", product))
print("\n=== my transcription against the source's literal calls")
for name, scene in SCENES:
    code = open(SOURCES[name], encoding="utf-8").read().split("#[cfg(test)]")[0]
    says_src = re.findall(r'\.say\(\w+, "(\d\d_\d+)"\)', code)
    literal = [(c, int(j)) for c, j in re.findall(r'until_sentence\(\w+, "(\d\d_\d+)", (\d+)\)', code)]
    probe = Sim(1.0)
    scene(probe)
    source_clauses = len(re.findall(r"until_clause\(&?k, \w+, ", code))
    # The debts' one call runs three times, through its loop.
    ck(source_clauses + (2 if name == "factorials" else 0) == len(probe.clauses), f"{name}: every clause hook is replayed", f"{len(probe.clauses)}")
    replay = [(c, j) for c, j, _, _ in probe.hooks]
    missing = [h for h in literal if h not in replay]
    extra = sorted(set(h for h in replay if h not in literal))
    exists = all(j < len(STARTS[c]) for c, j in replay)
    ck(says_src == list(probe.said) and not missing and exists, f"{name}: cues in source order, every literal hook replayed, every hook names a real sentence", f"loop-driven hooks read from the loops: {extra}")

for name, scene in SCENES:
    print(f"\n=== {name}")
    totals, runs = {}, {}
    for f in (0.8, 1.0, 1.25):
        s = Sim(f)
        scene(s)
        totals[f], runs[f] = s.t, s
    print("total seconds at 0.8 / 1.0 / 1.25: " + " / ".join(f"{totals[f]:.2f}" for f in (0.8, 1.0, 1.25)))
    s = runs[1.0]
    print("hooks, with lateness at 0.8 / 1.0 / 1.25 (0 means the picture waited for the sentence):")
    worst = {}
    for idx, (cue, j, late, shows) in enumerate(s.hooks):
        l8, l125 = runs[0.8].hooks[idx][2], runs[1.25].hooks[idx][2]
        flag = "  LATE" if max(l8, late, l125) > 0.25 else ""
        worst[(cue, j)] = max(l8, late, l125)
        print(f"  {cue} s{j}: {l8:4.2f} {late:4.2f} {l125:4.2f}  {shows}  <- {TEXT[cue][j][:52]!r}{flag}")
    for idx, (cue, j, phrase, late, shows) in enumerate(s.clauses):
        l8, l125 = runs[0.8].clauses[idx][3], runs[1.25].clauses[idx][3]
        flag = "  LATE" if max(l8, late, l125) > 0.25 else ""
        print(f"  {cue} s{j} clause {phrase!r}: {l8:4.2f} {late:4.2f} {l125:4.2f}  {shows}{flag}")
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
    print("seconds from each cue's start to its first visual change, factor 1.0:")
    for c, (x, y) in s.voice.items():
        first = next((a for a, b, label in ev if a >= x - 1e-9), None)
        print(f"  {c}: {first - x:5.2f} s of {y - x:5.2f} s")

print("\n=== order claims, at each factor")
for f in (0.8, 1.0, 1.25):
    s = Sim(f)
    series(s)
    gap_start, q_end, _ = s.gaps["predict 02_1"]
    ck(s.marks["first answer"] >= gap_start + 4.0, f"factor {f}: the first answer comes after 02_1's think gap", f"{s.marks['first answer'] - q_end:.2f} s after the question")
    named = s.said["02_3"] + STARTS["02_3"][3] * f
    ck(abs(s.marks["first factorial"] - named) < 0.26, f"factor {f}: the first factorial sign lands with 'Those denominators are factorials'", f"{s.marks['first factorial'] - named:+.2f} s")

print("\n=== sentence windows the dense beats lean on (seconds, factors 0.8 / 1.0 / 1.25)")
for c, j in (("01_5", 0), ("02_1", 1), ("02_1", 3), ("02_3", 2), ("02_3", 3), ("02_4", 2), ("02_5", 1), ("02_5", 2), ("02_5", 3), ("02_5", 4), ("02_5", 5), ("02_6", 1), ("03_2", 0), ("03_5", 2), ("03_7", 3)):
    st = STARTS[c]
    w = (st[j + 1] if j + 1 < len(st) else SPEECH[c]) - st[j]
    print(f"  {c} s{j}: {w * 0.8:5.2f} / {w:5.2f} / {w * 1.25:5.2f}  {TEXT[c][j][:70]!r}")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
