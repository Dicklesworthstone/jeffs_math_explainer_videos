#!/usr/bin/env python3
"""Run from the repo root: python3 -I -B <this file>

Replay the source's beats against the scratch voice and both Oliver takes.

Reads only. Nothing here was compiled, rendered or heard. No take is selected.
Oliver sentence and phrase onsets come from root's large-model ASR word times:
provisional estimates, not listening and not fine sync.

The scratch replay is the positive control: it must land on the frames the
sheets show. Its rule, fitted to those frames: speech is the cue length less
0.4 s, and a sentence starts at its share of the characters.

Beats: `until` waits only when its moment is ahead, as the kit does. A phrase's
time is the kit's fallback `phrase_time` in axioms.rs (its share of the
sentence's characters between this sentence's start and the next one's). The
kit prefers a phrase mark where one exists, so a real run can differ a little.
Frame rounding is ignored.
"""
import difflib
import json
import re
import wave
from pathlib import Path

ROOT = Path("narration_lab/third-wave-20261024/root")
LAB = ROOT / "five-film-release-20261027"
TAKES = ROOT / "elevenlabs-expanded-algebra-20261103"
CUES = LAB / "algebra-chapter10-repair03-preview01/render/10_cyclic.cues.tsv"
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")

LINES = {
    row.split("\t", 1)[0]: row.split("\t", 1)[1]
    for row in SCRIPT.read_text(encoding="utf-8").splitlines()
    if row.startswith("10_")
}
PREVIEW = {row.split("\t")[0]: (float(row.split("\t")[1]), float(row.split("\t")[2])) for row in CUES.read_text(encoding="utf-8").splitlines()}
NUMBER = {"0": "zero", "1": "one", "2": "two", "3": "three", "4": "four", "5": "five", "6": "six", "7": "seven", "8": "eight", "9": "nine", "10": "ten", "11": "eleven", "12": "twelve"}


def spans(text):
    """The kit's rule: a sentence ends at . ? or ! followed by a space or the end."""
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            a = start
            while text[a] == " ":
                a += 1
            out.append((a, i + 1))
            start = i + 1
    return out


def norm(word):
    word = re.sub(r"[^a-z0-9]", "", word.lower())
    return NUMBER.get(word, word)


class Voice:
    def __init__(self, name, cue):
        self.name, self.cue = name, cue
        self.text = LINES[cue]
        self.spans = spans(self.text)
        self.match = None
        if name == "scratch":
            self.length = PREVIEW[cue][1]
            speech = self.length - 0.4
            total = sum(b - a for a, b in self.spans)
            acc, self.starts = 0, []
            for a, b in self.spans:
                self.starts.append(speech * acc / total)
                acc += b - a
            self.end = speech
            return
        folder = LAB / f"expanded-algebra-{cue.replace('_', '-')}-asr01"
        segments = json.loads((folder / f"{name}-large.json").read_text(encoding="utf-8"))["result"]["segments"]
        words = [(float(s["start_sec"]), norm(s["text"])) for s in segments if norm(s["text"])]
        with wave.open(str(TAKES / f"{cue}-{name}.wav")) as w:
            self.length = w.getnframes() / w.getframerate()
        self.end = self.length
        # Align the whole line, word by word, so that a repeated phrase ("add eight")
        # binds to its own sentence.
        said = [m.group(0) for m in re.finditer(r"\S+", self.text)]
        self.word_at = [m.start() for m in re.finditer(r"\S+", self.text)]
        script = [norm(w) for w in said]
        heard = [w for _, w in words]
        pairs = {}
        for block in difflib.SequenceMatcher(None, script, heard, autojunk=False).get_matching_blocks():
            for i in range(block.size):
                pairs[block.a + i] = words[block.b + i][0]
        self.match = (len(pairs), len(script))
        self.pairs = pairs
        self.starts = []
        for j, (a, b) in enumerate(self.spans):
            first = next(i for i, at in enumerate(self.word_at) if a <= at < b and i in pairs)
            self.starts.append(0.0 if j == 0 else pairs[first])

    def phrase(self, j, phrase):
        """The kit's fallback for this phrase's time."""
        a, b = self.spans[j]
        offset = self.text[a:b].index(phrase)
        start = self.starts[j]
        end = self.starts[j + 1] if j + 1 < len(self.starts) else self.end
        return start + (end - start) * offset / (b - a)

    def word(self, j, phrase):
        """When the ASR hears the first aligned word of this phrase in sentence j, if it can."""
        if self.match is None:
            return None
        a, b = self.spans[j]
        byte = a + self.text[a:b].index(phrase)
        for i, at in enumerate(self.word_at):
            if at >= byte and i in self.pairs:
                return self.pairs[i]
        return None


def play(seconds, label=""):
    return ("play", seconds, label)


def s(j):
    return ("s", j)


def p(j, phrase):
    return ("p", j, phrase)


def appear_len(n, each, lag):
    return each + (n - 1) * each * lag


TRIP = 8 * 0.15 + 0.4
HOP = 0.2 + 2 * (4 * 0.08 + 0.6)

BEATS = {
    "10_1 current": ("10_1", [play(0.4, "whole picture with the question")]),
    "10_1 patched": ("10_1", [play(0.4, "clock"), s(3), play(0.4, "question up")]),
    "10_2 current": (
        "10_2",
        [s(1), play(TRIP, "at 8"), s(2), play(TRIP, "at 4"), s(3), play(TRIP, "at 0"), s(4), play(0.4, "trail"), s(5), play(0.4, "orbit"), s(6), play(0.4, "ord(8)=3")],
    ),
    "10_2 patched": (
        "10_2",
        [s(1), play(TRIP, "at 8"), s(2), play(TRIP, "at 4"), s(3), play(TRIP, "at 0"), s(4), play(0.4, "trail"), s(5), play(0.4, "orbit"), s(6), play(0.4, "ord(8)=3"), s(7), play(0.4, "group size"), s(8), play(0.6, "both pulse")],
    ),
    "10_4 current": (
        "10_4",
        [
            play(0.4 + 0.6 + 0.4, "setting"),
            s(1),
            play(0.4, "d"),
            play(appear_len(4, 0.7, 0.5), "star drawn"),
            play(0.4, "star tint"),
            play(0.3, "orbit count"),
            s(2),
            play(0.3 + 0.3, "star gone"),
            play(appear_len(3, 0.5, 0.6), "block arcs"),
            play(0.3, "block labels"),
            play(0.3, "n split"),
            p(2, "and k as d times"),
            play(HOP, "hop done"),
            play(0.3, "k split"),
            play(0.3, "hop hidden"),
            s(3),
            play(0.4, "coprime"),
            play(0.6, "pulse"),
        ],
    ),
    "10_4 patched": (
        "10_4",
        [
            play(0.4 + 0.6 + 0.4, "setting"),
            s(1),
            play(0.4, "d"),
            play(appear_len(4, 0.7, 0.5), "star drawn"),
            play(0.4, "star tint"),
            play(0.3, "orbit count"),
            s(2),
            play(0.3 + 0.3, "star gone"),
            play(appear_len(3, 0.5, 0.6), "block arcs"),
            play(0.3, "block labels"),
            play(0.3, "n split"),
            p(2, "and k as d times"),
            play(0.3, "k split"),
            play(HOP, "hop done"),
            play(0.3, "hop hidden"),
            s(3),
            play(0.4, "coprime"),
            play(0.6, "pulse"),
        ],
    ),
    "10_6 current": (
        "10_6",
        [
            play(0.4 + 0.8, "rule"),
            s(1),
            play(0.6, "asked title"),
            play(0.3, "k and gcd names"),
            play(appear_len(11, 0.3, 0.3), "k row"),
            play(appear_len(11, 0.3, 0.3), "gcd row"),
            s(2),
            play(0.3, "ord name"),
            play(appear_len(11, 0.3, 0.3), "ord row"),
            play(0.6, "rule pulse"),
            p(2, "one, five, seven"),
            play(0.0, "generators boxed from here"),
        ],
    ),
    "10_6 patched": (
        "10_6",
        [
            play(0.4 + 0.8, "rule"),
            s(1),
            play(0.6, "asked title"),
            play(0.3, "k name"),
            play(appear_len(11, 0.3, 0.3), "k row"),
            s(2),
            play(0.3, "gcd and ord names"),
            play(appear_len(11, 0.3, 0.3), "gcd row"),
            play(appear_len(11, 0.3, 0.3), "ord row"),
            play(0.6, "rule pulse"),
            p(2, "one, five, seven"),
            play(0.0, "generators boxed from here"),
        ],
    ),
    "10_10 current": (
        "10_10",
        [play(0.4, "proof gone"), play(0.4, "whole table"), s(3), play(0.4, "squares"), s(5), play(0.4, "no element")],
    ),
    "10_10 patched": (
        "10_10",
        [
            play(0.4, "proof gone"),
            play(0.4, "U(12) line"),
            s(1),
            play(0.4, "table"),
            s(2),
            play(0.4, "relatively prime"),
            s(3),
            play(0.4, "squares"),
            play(0.4, "diagonal tint"),
            s(4),
            play(0.4, "orders"),
            s(5),
            play(0.4, "no element"),
            p(5, "so this group is not"),
            play(0.4, "not the rotations"),
        ],
    ),
}


def replay(voice, ops):
    t, rows = 0.0, []
    for op in ops:
        if op[0] == "play":
            rows.append((t, t + op[1], op[2]))
            t += op[1]
        else:
            moment = voice.starts[op[1]] if op[0] == "s" else voice.phrase(op[1], op[2])
            late = t - moment
            what = f"sentence {op[1]}" if op[0] == "s" else f"phrase '{op[2]}'"
            rows.append((moment, None, f"[{what}{f' passed by {late:.2f}' if late > 0 else ''}]"))
            t = max(t, moment)
    return t, rows


print("== this preview's cue table against the preview before repair03 (scratch voice both times)")
BASE = LAB / "algebra-combined-repair-preview01/render/10_cyclic.cues.tsv"
print("identical bytes:", CUES.read_bytes() == BASE.read_bytes(), "| cues", len(PREVIEW))

print("\n== word alignment of each take to the script (matched / script words), and lengths")
for cue in LINES:
    row = [cue, f"preview {PREVIEW[cue][1]:.3f}"]
    for name in ("take01", "take02"):
        v = Voice(name, cue)
        row.append(f"{name} len {v.length:.2f} matched {v.match[0]}/{v.match[1]}")
    print(" | ".join(row))

print("\n== sentence starts, seconds into the cue")
for cue in LINES:
    for name in ("scratch", "take01", "take02"):
        v = Voice(name, cue)
        print(cue, name.ljust(7), [round(x, 2) for x in v.starts], "| speech ends", round(v.end, 2))

print("\n== positive control: scratch replay against frames read on the sheets")
# Frame anchors: (label, cue, seconds into the cue the replay gives, first frame on the sheets showing it).
TIMES = json.loads((LAB / "algebra-chapter10-repair03-frames01/10_cyclic/sampling.json").read_text(encoding="utf-8"))["times"]
ANCHORS = [
    ("10_2 ord(8)=3 begins", "10_2 current", "ord(8)=3", 94),
    ("10_4 d row begins", "10_4 current", "d", 185),
    ("10_4 k split begins", "10_4 current", "k split", 202),
    ("10_4 coprime begins", "10_4 current", "coprime", 203),
    ("10_6 k row begins", "10_6 current", "k row", 282),
    ("10_6 gcd row begins", "10_6 current", "gcd row", 284),
    ("10_6 ord row begins", "10_6 current", "ord row", 288),
    ("10_10 table begins", "10_10 current", "whole table", 505),
    ("10_10 squares begins", "10_10 current", "squares", 536),
    ("10_10 no element begins", "10_10 current", "no element", 551),
]
for label, key, beat, frame in ANCHORS:
    cue, ops = BEATS[key]
    v = Voice("scratch", cue)
    _, rows = replay(v, ops)
    a = next(x for x in rows if x[2] == beat)
    at = PREVIEW[cue][0] + a[0]
    print(f"{label}: replay {at:.2f} | first seen in frame {frame} at {TIMES[frame]:.2f} | sample before at {TIMES[frame - 1]:.2f} | replay falls inside that interval or within one sample: {TIMES[frame - 1] - 0.5 <= at <= TIMES[frame] + 0.5}")

print("\n== beats, seconds into the cue: start-end label, one block per voice")
for key, (cue, ops) in BEATS.items():
    for name in ("scratch", "take01", "take02"):
        v = Voice(name, cue)
        end, rows = replay(v, ops)
        print(f"-- {key} | {name} | speech ends {v.end:.2f} | last beat ends {end:.2f} | fits inside the voice: {end <= v.end}")
        for a, b, label in rows:
            print(f"   {a:6.2f}" + (f"-{b:6.2f} {label}" if b is not None else f"        {label}"))

print("\n== how long the 10_10 table would sit unchanged, seconds")
for name in ("scratch", "take01", "take02"):
    v = Voice(name, "10_10")
    print(name, "current: table full at 0.80, squares at", round(v.starts[3], 2), "=", round(v.starts[3] - 0.8, 2), "| patched longest gap:", round(max(v.starts[1] - 0.8, v.starts[2] - v.starts[1] - 0.4, v.starts[3] - v.starts[2] - 0.4), 2))

print("\n== what the voice is on when the question and the rows are up, seconds into the cue")
for name in ("scratch", "take01", "take02"):
    a = Voice(name, "10_1")
    print(name, "10_1: 'add eight' sentence at", round(a.starts[3], 2), "| 'predict' sentence at", round(a.starts[4], 2), "| question up since 0.40")
    b = Voice(name, "10_6")
    print(name, "10_6: question sentence", round(b.starts[1], 2), "to", round(b.starts[2], 2), "| answer word 'one, five, seven' by phrase rule", round(b.phrase(2, "one, five, seven"), 2), "| by ASR", b.word(2, "one, five, seven"))
    c = Voice(name, "10_4")
    print(name, "10_4: 'and k as d times' by phrase rule", round(c.phrase(2, "and k as d times"), 2), "| by ASR", c.word(2, "and k as d times"), "| 'k prime' by ASR", c.word(2, "k prime"), "| 'Those two' at", round(c.starts[3], 2))
    d = Voice(name, "10_2")
    print(name, "10_2: 'The whole clock' at", round(d.starts[7], 2), "| 'These are' at", round(d.starts[8], 2), "| speech ends", round(d.end, 2))

print("\n== the end of 10_11: last picture change against the voice, seconds into the cue")
for name in ("scratch", "take01", "take02"):
    v = Voice(name, "10_11")
    t = v.phrase(2, "the smallest positive step")
    print(name, "d ring pulse", round(t, 2), "to", round(t + 0.6, 2), "| 'Next we' at", round(v.starts[3], 2), "| speech ends", round(v.end, 2), "| still after the pulse", round(v.end - t - 0.6, 2))
