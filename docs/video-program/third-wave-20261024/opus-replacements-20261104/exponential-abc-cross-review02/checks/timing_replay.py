"""Replay the three staged scenes against the Kit's scratch-voice timing, at speech factors 0.8, 1.0 and 1.25.

Usage from the repo root:  python3 -I -B timing_replay.py SCRATCH_DIR
The sentence splitter and onset rule are not reimplemented: `sentences` and `estimated_starts` are cut verbatim
from crates/explainer_kit/src/narration.rs and compiled with rustc into SCRATCH_DIR. The block lists below are my
own transcription of each construct(), with source lines. Waits, holds and breaths follow narration.rs (say,
hold, window, wait_gap) under LineGaps::Constant, which is what exponential_magic's main.rs sets.

Scratch timing is not Oliver's timing. Nothing here is a listening or picture claim.
"""
import os
import re
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
SLICES = {
    "rate": R + "exponential-expanded-rate02/script-slice.tsv",
    "series": R + "exponential-expanded-series01/script-slice.tsv",
    "product": R + "exponential-expanded-product01/script-slice.tsv",
}
BREATH = 0.6


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
        self.f, self.t, self.busy, self.said, self.events, self.late, self.gaps = factor, 0.0, 0.0, {}, [], [], {}
        self.hooks, self.voice = [], {}

    def length(self, cue):
        return SPEECH[cue] * self.f + 0.4  # scratch: speech + 0.4, narration.rs:978-982

    def say(self, cue):
        self.t = max(self.t, self.busy + BREATH)
        self.said[cue] = self.t
        self.busy = self.t + self.length(cue)
        self.voice[cue] = (self.t, self.busy)

    def until(self, cue, j, shows=""):
        at = self.said[cue] + STARTS[cue][j] * self.f
        if at > self.t:
            self.t = at
            late = 0.0
        else:
            late = self.t - at
        self.hooks.append((cue, j, late, shows))
        return late

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


def rate(s):
    """exponential-expanded-rate02/rate_law.rs"""
    s.say("01_1")
    s.play(0.6, "frame, axes, start dots :106")
    s.until("01_1", 2, "constant rate = 1 :108"); s.play(0.4, "left rule :109")
    s.until("01_1", 3, "rate = current height :111"); s.play(0.4, "right rule :112")
    s.until("01_1", 4, "first slopes and read-outs :118"); s.play(0.4, "first slopes :126")
    s.until("01_1", 5, "question :130"); s.play(0.4, "question :135")
    q_end = s.t
    s.hold(); s.gaps["predict 01_1"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question fades :140")
    s.say("01_2"); s.play(0.6, "first step :168")
    s.until("01_2", 2, "height read again :186"); s.play(0.6, "reading morph :186"); s.wait(0.6)
    s.hold(); s.say("01_3"); s.play(0.6, "second step :168")
    s.until("01_3", 1, "reading morph :186"); s.play(0.6, "reading morph :186"); s.wait(0.6)
    s.until("01_3", 2, "third step :168"); s.play(0.6, "third step"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.until("01_3", 3, "fourth step :168"); s.play(0.6, "fourth step"); s.play(0.6, "reading morph"); s.wait(0.6)
    s.until("01_3", 4, "straight steps: an approximation :199"); s.play(0.4, "approximation :206")
    s.until("01_3", 5, "last step read-out :209"); s.play(0.4, "last step read-out :216")
    s.hold(); s.say("01_4")
    s.play(0.3, "slope tick fades :223"); s.play(0.3, "endpoint read-out :231")
    s.until("01_4", 2, "old drawing greys :245"); s.play(0.4, "old drawing greys")
    s.until("01_4", 3, "8 steps :250")
    s.play(0.3, "read-out"); s.play(8 * 0.15, "8 steps :267"); s.wait(1.0)
    s.until("01_4", 4, "grey, then 16 steps :233"); s.play(0.4, "grey")
    s.play(0.3, "read-out"); s.play(16 * 0.15, "16 steps :267"); s.wait(1.0)
    s.hold(); s.say("01_5")
    s.until("01_5", 2, "promise :275"); s.play(0.4, "promise :282")
    s.hold()


def series(s):
    """exponential-expanded-review01/candidate/series01/factorials.rs"""
    s.say("02_1"); s.play(0.35, "title :84")
    s.until("02_1", 2, "boxes, headers, coefficients :88"); s.play(0.6, "table :106")
    s.until("02_1", 3, "derivative row and moving terms :108"); s.play(0.35, "derivative label :109")
    for i in range(4):
        s.play(0.35, f"copy {i} :114"); s.play(0.6, f"move {i} :116"); s.play(0.35, f"connector {i} :119")
    s.until("02_1", 4, "question :123"); s.play(0.35, "question :124")
    q_end = s.t
    s.hold(); s.gaps["predict 02_1"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.3, "question clears :128")
    s.say("02_2")
    for sentence, label in ((0, "a = 1"), (3, "b = 1/2"), (5, "c = 1/6")):
        s.until("02_2", sentence, label + " :132"); s.play(0.35, label)
    s.say("02_3")
    s.until("02_3", 1, "recurrence :144"); s.play(0.35, "recurrence :145")
    s.until("02_3", 2, "d = 1/24 :148"); s.play(0.35, "d = 1/24 :155")
    s.until("02_3", 4, "table clears, axes :158"); s.play(0.3, "table clears :158"); s.play(0.4, "axes :172")
    for n in range(5):
        s.play(0.5, f"curve degree {n} :193"); s.wait(0.6)
    s.hold(); s.say("02_4"); s.play(0.35, "finite polynomial card :201")
    s.until("02_4", 3, "tail needs a proof :204"); s.play(0.35, "proof card :205")
    s.hold(); s.play(0.3, "chart clears :208")
    s.say("02_5"); s.play(0.4, "interval :216")
    s.until("02_5", 1, "ratio formula :218"); s.play(0.35, "ratio formula :219")
    s.until("02_5", 3, "tail formula, read-out :222"); s.play(0.35, "tail formula"); s.play(0.35, "read-out")
    s.until("02_5", 5, "uniform convergence :239"); s.play(0.35, "uniform card :240")
    s.hold(); s.say("02_6"); s.play(0.3, "proof clears :244"); s.play(0.35, "E formula :246")
    s.hold()


def product(s):
    """exponential-expanded-review01/candidate/product01/product_grid.rs"""
    s.say("03_1"); s.play(0.35, "title :85")
    s.until("03_1", 2, "edges and headers :91"); s.play(0.4, "edge labels :96")
    for i in range(8):
        s.play(0.35, f"header {i} :107")
    s.until("03_1", 3, "rules and cells :125"); s.play(0.4, "rules :131")
    for i in range(16):
        s.play(0.2, f"cell {i} :145")
    s.until("03_1", 5, "corner card :151"); s.play(0.35, "corner card :152")
    s.hold(); s.say("03_2")
    s.until("03_2", 1, "degree 0 :159"); s.play(0.3, "degree 0 lit"); s.play(0.35, "total 1 :172")
    s.until("03_2", 2, "degree 1 :159"); s.play(0.3, "degree 1 lit")
    s.until("03_2", 3, "total x+y :162"); s.play(0.35, "total x+y :172")
    s.until("03_2", 4, "degree 2 :159"); s.play(0.3, "degree 2 lit")
    s.until("03_2", 5, "question :177"); s.play(0.35, "question :178")
    q_end = s.t
    s.hold(); s.gaps["predict 03_2"] = (s.breath(4.0), q_end, s.busy)
    s.play(0.25, "question hides :182")
    s.say("03_3"); s.play(0.35, "sum of three cells :185")
    s.until("03_3", 1, "factor one half :188"); s.play(0.6, "morph :189")
    s.until("03_3", 3, "(x+y)^2/2 :188"); s.play(0.6, "morph :189"); s.play(0.35, "total :192")
    s.hold(); s.say("03_4"); s.play(0.3, "degree 3 lit :196"); s.play(0.35, "total :198")
    s.until("03_4", 1, "binomial :201"); s.play(0.35, "binomial :202")
    s.until("03_4", 2, "identity :204"); s.play(0.35, "identity :205")
    s.until("03_4", 3, "complete diagonals card :207"); s.play(0.35, "card :208")
    s.hold(); s.say("03_5"); s.play(0.25, "card hides :211")
    s.until("03_5", 1, "permission :213"); s.play(0.35, "permission :214")
    s.until("03_5", 2, "series :216"); s.play(0.35, "series :217")
    s.until("03_5", 3, "law :219"); s.play(0.35, "law :220")
    s.hold(); s.play(0.5, "everything fades :229")
    s.say("03_6")
    s.until("03_6", 1, "E(x)E(-x) :232"); s.play(0.35, "non-vanishing :233")
    s.until("03_6", 2, "half law :235"); s.play(0.35, "half law :236")
    s.until("03_6", 3, "> 0 :239"); s.play(0.5, "morph :240")
    s.until("03_6", 4, "E'=E>0 :242"); s.play(0.35, "E'=E>0 :243")
    s.hold()


SOURCES = {
    "rate_law": R + "exponential-expanded-rate02/rate_law.rs",
    "factorials": R + "exponential-expanded-review01/candidate/series01/factorials.rs",
    "product_grid": R + "exponential-expanded-review01/candidate/product01/product_grid.rs",
}
print("\n=== my transcription against the source's literal calls")
for name, scene in (("rate_law", rate), ("factorials", series), ("product_grid", product)):
    code = open(SOURCES[name], encoding="utf-8").read().split("#[cfg(test)]")[0]
    says_src = re.findall(r'\.say\(\w+, "(\d\d_\d+)"\)', code)
    literal = [(c, int(j)) for c, j in re.findall(r'until_sentence\(\w+, "(\d\d_\d+)", (\d+)\)', code)]
    probe_sim = Sim(1.0)
    scene(probe_sim)
    replay = [(c, j) for c, j, _, _ in probe_sim.hooks]
    missing = [h for h in literal if h not in replay]
    extra = [h for h in replay if h not in literal]
    ok = list(probe_sim.said) == says_src and not missing
    # every hook must also name a sentence that exists and follow its cue's say
    ok = ok and all(j < len(STARTS[c]) for c, j in replay)
    print(f"{'PASS' if ok else 'FAIL'} {name}: cues in source order {says_src == list(probe_sim.said)}; literal hooks all replayed {not missing}; loop-driven hooks read from the loops {extra}")
    assert ok, (name, missing)

for name, scene in (("rate_law", rate), ("factorials", series), ("product_grid", product)):
    print(f"\n=== {name}")
    totals = {}
    runs = {}
    for f in (0.8, 1.0, 1.25):
        s = Sim(f)
        scene(s)
        totals[f] = s.t
        runs[f] = s
    print("total seconds at 0.8 / 1.0 / 1.25: " + " / ".join(f"{totals[f]:.2f}" for f in (0.8, 1.0, 1.25)))
    s = runs[1.0]
    print("hooks, factor 1.0, with lateness at 0.8 / 1.0 / 1.25 (0 means the picture waited for the sentence):")
    for idx, (cue, j, late, shows) in enumerate(s.hooks):
        l8, l125 = runs[0.8].hooks[idx][2], runs[1.25].hooks[idx][2]
        flag = "  LATE" if max(l8, late, l125) > 0.25 else ""
        print(f"  {cue} s{j}: {l8:4.2f} {late:4.2f} {l125:4.2f}  {shows}  <- {TEXT[cue][j][:58]!r}{flag}")
    for key, (start, q_end, busy) in s.gaps.items():
        print(f"  {key}: answer withheld from question landing for {start + 4.0 + 0.3 - q_end:.2f} s; think gap after voice end {start + 4.0 + 0.3 - (start - BREATH):.2f} s")
    print("static stretches of 6 s or more at factor 1.0 (no reveal, morph or fade running):")
    last = 0.0
    ev = sorted(s.events)
    spans = []
    for a, b, label in ev:
        if a - last >= 6.0:
            spans.append((last, a, label))
        last = max(last, b)
    if s.t - last >= 6.0:
        spans.append((last, s.t, "end of chapter"))
    for a, b, label in spans:
        cues = [c for c, (x, y) in s.voice.items() if x < b and y > a]
        said = []
        for c in cues:
            for j, st in enumerate(STARTS[c]):
                at = s.said[c] + st
                nxt = s.said[c] + (STARTS[c][j + 1] if j + 1 < len(STARTS[c]) else SPEECH[c])
                if at < b and nxt > a:
                    said.append(f"{c} s{j}")
        print(f"  {b - a:5.2f} s, {a:7.2f} to {b:7.2f}, before {label!r}; voice: {', '.join(said)}")
    # how long into each cue before anything new lands
    print("seconds from each cue's start to its first visual change, factor 1.0:")
    for c, (x, y) in s.voice.items():
        first = next((a for a, b, label in ev if a >= x - 1e-9), None)
        print(f"  {c}: {first - x:5.2f} s of {y - x:5.2f} s")

s = Sim(1.0)
series(s)
a = s.said["02_3"] + STARTS["02_3"][3]
b = s.said["02_5"] + STARTS["02_5"][3]
print(f"\n'Those denominators are factorials' starts at {a:.2f} s; the first factorial sign on screen (TEX[2]) at {b:.2f} s: {b - a:.2f} s later, factor 1.0")

print("\n=== sentence windows findings lean on (seconds, factors 0.8 / 1.0 / 1.25)")
for c, j in (("02_1", 0), ("02_1", 1), ("02_3", 3), ("02_5", 1), ("02_5", 2), ("02_5", 4), ("03_2", 0), ("03_5", 2), ("01_5", 0), ("01_5", 1)):
    st = STARTS[c]
    w = (st[j + 1] if j + 1 < len(st) else SPEECH[c]) - st[j]
    print(f"  {c} s{j}: {w * 0.8:5.2f} / {w:5.2f} / {w * 1.25:5.2f}  {TEXT[c][j]!r}")
