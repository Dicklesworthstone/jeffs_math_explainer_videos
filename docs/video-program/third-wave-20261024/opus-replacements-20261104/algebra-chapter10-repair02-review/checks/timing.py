"""Beat timing for baseline, root's candidate and the proposal, against three voices.

Run from the repo root: python3 -I -B <this file>
Reads only. Nothing here was compiled, rendered or heard.

The voices:
- take01, take02: the real ElevenLabs Oliver takes. No take is selected yet, so
  both are shown. Sentence and word onsets are root's large-model ASR word
  times: provisional estimates, not listening and not fine sync.
- scratch: the voice the preview pictures used. Sentence starts are shares of
  the line by sentence length, as the kit does for that voice. Its length is
  the preview's cue length. It has no word times.

The beats replay the source's own run times. An `until_*` waits only if its
moment is ahead, as the kit does; if the moment has passed the beat is LATE.
A phrase's time is the kit's fallback in axioms.rs `phrase_time`: its share of
the sentence's characters, between this sentence's start and the next one's.
The kit prefers a phrase mark where one exists, so a real run can differ a
little. Frame rounding (1/60 s) is ignored.

"lead" is the word's onset minus the beat's start: positive means the picture
starts before the voice gets there.
"""
import json
import re
import wave
from pathlib import Path

ROOT = Path("narration_lab/third-wave-20261024/root")
ASR = ROOT / "five-film-release-20261027"
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
CUES = ASR / "algebra-combined-repair-preview01/render/10_cyclic.cues.tsv"
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")

LINES = {
    l.split("\t")[0]: l.split("\t", 1)[1]
    for l in SCRIPT.read_text(encoding="utf-8").splitlines()
    if l and not l.startswith("#")
}
PREVIEW = {l.split("\t")[0]: float(l.split("\t")[2]) for l in CUES.read_text().splitlines()}


def ranges(text):
    """The kit's sentence rule: split after . ? ! followed by a space or the end."""
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append((start, i + 1))
            start = i + 1
    trimmed = []
    for a, b in out:
        while text[a] == " ":
            a += 1
        trimmed.append((a, b))
    return trimmed


def norm(word):
    return re.sub(r"[^a-z0-9]", "", word.lower().replace("five", "5"))


class Voice:
    def __init__(self, name, cue):
        self.name, self.cue = name, cue
        self.text = LINES[cue]
        self.spans = ranges(self.text)
        if name == "scratch":
            self.words = None
            self.length = PREVIEW[cue]
            total = sum(b - a for a, b in self.spans)
            acc, self.starts = 0, []
            for a, b in self.spans:
                self.starts.append(self.length * acc / total)
                acc += b - a
        else:
            folder = ASR / ("expanded-algebra-" + cue.replace("_", "-") + "-asr01")
            segments = json.loads((folder / f"{name}-large.json").read_text(encoding="utf-8"))["result"]["segments"]
            self.words = [(float(s.get("start_sec", s.get("start"))), norm(s["text"])) for s in segments]
            with wave.open(str(TAKES / f"{cue}-{name}.wav")) as w:
                self.length = w.getnframes() / w.getframerate()
            self.starts = []
            for j, (a, b) in enumerate(self.spans):
                head = [norm(x) for x in self.text[a:b].split()[:2]]
                at = self.onset(" ".join(head))
                self.starts.append(0.0 if j == 0 else at)

    def onset(self, phrase):
        """The first time these words are heard, or None for the scratch voice."""
        if self.words is None:
            return None
        parts = [norm(x) for x in phrase.split()]
        said = [w[1] for w in self.words]
        for i in range(len(said) - len(parts) + 1):
            if said[i : i + len(parts)] == parts:
                return self.words[i][0]
        raise SystemExit(f"{self.cue} {self.name}: words not found: {phrase}")

    def phrase_time(self, j, phrase):
        a, b = self.spans[j]
        offset = self.text[a:b].index(phrase)
        start = self.starts[j]
        end = self.starts[j + 1] if j + 1 < len(self.starts) else self.length
        return start + (end - start) * offset / (b - a)


# An op: ("play", seconds, label, words) or ("s", j) or ("p", j, phrase).
def play(seconds, label="", words=None):
    return ("play", seconds, label, words)


def s(j):
    return ("s", j)


def p(j, phrase):
    return ("p", j, phrase)


NUMBERS = 0.25 + 41 * 0.25 * 0.1  # appear(&numbers, 0.25, 0.1): 42 parts, each 0.025 s after the last.
WHOLE = 0.4 + 4 * 0.4 * 0.4  # draw(&whole, 0.4, 0.4): 5 chunks, each 0.16 s after the last.
SWEEP = 12 * 0.15 + 3 * 0.15  # 12 slides, plus a dot at 4, 8 and 12.

LINE_107 = [
    play(0.3, "line and axis"),
    play(NUMBERS, "line finished", "lets prove"),
]
AFTER_S2_107 = [
    s(2),
    play(0.4, "group line"),
    play(0.5, "example shown"),
    play(0.6, "example pulse"),
    s(3),
]
TAIL_107 = [
    p(4, "and choose the one"),
    play(4 * 0.25, "scan marks"),
    p(4, "call that exponent d"),
    play(0.4, "d ring"),
    s(5),
]
PROGRAMS = {
    ("10_7", "baseline"): [play(0.4), play(0.6, "title landed"), s(1), *LINE_107, *AFTER_S2_107,
                           play(0.4, "trivial case"), s(4), play(0.3), play(0.2), play(SWEEP, "sweep"), *TAIL_107],
    ("10_7", "candidate"): [play(0.4), play(0.6, "title landed"), *LINE_107, *AFTER_S2_107,
                            play(0.3, "example hidden"), play(0.4, "trivial case"), s(4), play(0.3),
                            play(0.3, "example back"), play(0.2), play(SWEEP, "sweep"), *TAIL_107],
    ("10_7", "proposed"): [play(0.4), play(0.6, "title landed"), p(0, "of a cyclic group"), *LINE_107, *AFTER_S2_107,
                           play(0.3, "example hidden"), play(0.4, "trivial case"), s(4), play(0.3),
                           play(0.3, "example back"), play(0.2), play(SWEEP, "sweep"), *TAIL_107],
    ("10_10", "candidate"): [play(0.4), play(0.4, "table"), s(2), play(0.4, "squares", "5 squared"),
                             s(3), play(0.4, "no element", "no single")],
    ("10_10", "proposed"): [play(0.4), play(0.4, "table"), s(3), play(0.4, "squares", "5 squared"),
                            s(5), play(0.4, "no element", "no single")],
    ("10_11", "candidate"): [play(0.4), play(0.4, "dial"), play(1.2, "path", "repeating a move"),
                             play(0.4, "ord(8)=3", "measures its"), s(1), play(0.4, "formula", "the arithmetic"),
                             play(0.6, "pulse path+order", "predicts that"), s(2), play(0.4),
                             play(0.4, "proof back", "the subgroup"), play(WHOLE, "chunks", "divide exponents"),
                             play(0.6, "pulse d ring", "the smallest positive"), s(3),
                             play(0.6, "pulse members", "next we")],
    ("10_11", "proposed"): [play(0.4), play(0.4, "dial"), p(0, "repeating a move"),
                            play(2.2, "path", "repeating a move"), p(0, "measures its order"),
                            play(0.4, "ord(8)=3", "measures its"), s(1), play(0.4, "formula", "the arithmetic"),
                            p(1, "predicts that return"), play(0.6, "pulse order", "predicts that"), s(2), play(0.4),
                            play(0.4, "proof back", "the subgroup"), p(2, "divide exponents"),
                            play(WHOLE, "chunks", "divide exponents"), p(2, "the smallest positive step"),
                            play(0.6, "pulse d ring", "the smallest positive")],
}


def run(voice, ops):
    t, rows, late = 0.0, [], []
    for op in ops:
        if op[0] == "play":
            _, seconds, label, words = op
            if label:
                rows.append((label, t, t + seconds, words))
            t += seconds
        else:
            at = voice.starts[op[1]] if op[0] == "s" else voice.phrase_time(op[1], op[2])
            if at < t:
                late.append((op, t - at))
            t = max(t, at)
    return rows, late, t


print("Voice onsets, seconds from the start of the cue")
for cue, phrases in (
    ("10_7", ["every subgroup", "of a cyclic", "lets prove", "suppose a", "if h", "otherwise look", "and choose the", "call that", "if a power"]),
    ("10_10", ["consider the", "each is", "5 squared", "every nonidentity", "no single"]),
    ("10_11", ["repeating a move", "and the first", "measures its", "the arithmetic", "predicts that", "the subgroup", "divide exponents", "then use", "the smallest positive", "rule out", "next we"]),
):
    for name in ("take01", "take02", "scratch"):
        v = Voice(name, cue)
        print(f"  {cue} {name}: length {v.length:.2f}; sentences " + " ".join(f"{x:.2f}" for x in v.starts))
        if v.words:
            print("    " + "; ".join(f"{q}={v.onset(q):.2f}" for q in phrases))

print()
problems = 0
for (cue, version), ops in PROGRAMS.items():
    print(f"== {cue} {version}")
    for name in ("take01", "take02", "scratch"):
        v = Voice(name, cue)
        rows, late, end = run(v, ops)
        print(f"  {name}: last picture ends {end:.2f} of {v.length:.2f}; beats late: {len(late)}")
        for op, by in late:
            print(f"    LATE by {by:.2f}: {op}")
            problems += version != "baseline"
        if end > v.length:
            print("    PAST THE END OF THE LINE")
            problems += 1
        for label, a, b, words in rows:
            note = ""
            if words and v.words:
                note = f"  \"{words}\" at {v.onset(words):.2f}, lead {v.onset(words) - a:+.2f}"
            print(f"    {label:<18} {a:6.2f} {b:6.2f}{note}")

print()
print("The exponent line and what moves around it (10_7, seconds)")
print("  voice    version    title alone  line builds     still until 'Suppose'")
for name in ("take01", "take02", "scratch"):
    v = Voice(name, "10_7")
    for version in ("baseline", "candidate", "proposed"):
        rows, _, _ = run(v, PROGRAMS[("10_7", version)])
        built = dict((r[0], (r[1], r[2])) for r in rows)
        begin, done = built["line and axis"][0], built["line finished"][1]
        print(f"  {name:<8} {version:<10} {begin - 1.0:9.2f}   {begin:5.2f}-{done:5.2f}   {v.starts[2] - done:9.2f}")

print()
print("Example off screen during the trivial case (10_7, candidate)")
for name in ("take01", "take02", "scratch"):
    v = Voice(name, "10_7")
    rows, _, _ = run(v, PROGRAMS[("10_7", "candidate")])
    built = dict((r[0], (r[1], r[2])) for r in rows)
    print(f"  {name}: on screen {built['example shown'][1]:.2f}-{built['example hidden'][0]:.2f}, back {built['example back'][1]:.2f}")

print()
print("TIMING OK" if problems == 0 else f"TIMING PROBLEMS: {problems}")
