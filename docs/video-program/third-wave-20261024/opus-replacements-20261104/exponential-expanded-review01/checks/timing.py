#!/usr/bin/env python3
"""Replay the series and product scenes against the Kit's scratch timing rule.

speech = words / 2.6, length = speech + 0.4, and sentences start at their share of
speech by characters (crates/explainer_kit/src/narration.rs). `factor` scales speech for
a voice slower or faster than scratch. This is arithmetic from source, not measurement:
no Oliver take exists for these cues. Each scene below restates the source's own order
of plays, so a source change needs the same change here. Run from the repo root.
"""
import re

ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
BREATH = 0.6


def load(path):
    rows = {}
    for row in open(ROOT + path, encoding="utf-8").read().splitlines():
        cue, text = row.split("\t", 1)
        rows[cue] = text
    return rows


def sentences(text):
    return [s.strip() for s in re.findall(r".+?[.?!](?= |$)", text)]


class Sim:
    def __init__(self, script, factor):
        self.script, self.factor = script, factor
        self.t, self.busy = 0.0, 0.0
        self.said, self.events = {}, []
        self.current = None

    def starts(self, cue):
        text = self.script[cue]
        speech = len(text.split()) / 2.6 * self.factor
        parts = sentences(text)
        total = sum(len(s) for s in parts)
        out, before = [], 0
        for s in parts:
            out.append(speech * before / total)
            before += len(s)
        return out

    def length(self, cue):
        return len(self.script[cue].split()) / 2.6 * self.factor + 0.4

    def say(self, cue):
        self.t = max(self.t, self.busy + BREATH) if self.current else self.t
        self.said[cue] = self.t
        self.busy = self.t + self.length(cue)
        self.current = cue

    def hold(self):
        self.t = max(self.t, self.busy + BREATH)

    def until(self, cue, j):
        self.t = max(self.t, self.said[cue] + self.starts(cue)[j])

    def play(self, seconds, label=None):
        start = self.t
        self.t += seconds
        if label:
            self.events.append((label, start, self.t, self.current))

    def sentence_at(self, cue, j):
        return self.said[cue] + self.starts(cue)[j]


def factorials(sim):
    sim.say("02_1")
    sim.play(0.35)
    sim.play(2 * 0.35)
    sim.play(5 * (4 + 2) * 0.35, "table built")
    table_built = sim.t
    sim.until("02_1", 3)
    sim.play(4 * 2 * 0.35, "derivative row")
    sim.play(0.35, "question on screen")
    question = sim.t
    sim.hold()
    sim.play(4.0)
    sim.play(0.12)
    sim.say("02_2")
    for sentence in (0, 3, 5):
        sim.until("02_2", sentence)
        sim.play(0.35)
    sim.say("02_3")
    sim.until("02_3", 2)
    sim.play(0.35)
    sim.play(0.35, "recurrence on screen")
    sim.until("02_3", 4)
    wipe = 2 + 5 * 6 + 8 + 3 + 2  # rows, columns, derivative pairs, three answers, d and the recurrence
    sim.play(wipe * 0.12, f"table wiped one at a time ({wipe} objects)")
    sim.play(4 * 0.35 + 2 * 0.35, "chart frame")
    first_curve = sim.t
    sim.play(5 * 0.5, "five curves")
    curves_done = sim.t
    sim.hold()
    sim.say("02_4")
    sim.play(0.35)
    sim.until("02_4", 3)
    sim.hold()
    sim.play(0.4)
    sim.play(0.12)
    sim.say("02_5")
    empty_from = sim.t
    sim.until("02_5", 2)
    ratio = sim.t
    sim.play(0.35)
    sim.until("02_5", 3)
    sim.play(0.35)
    sim.play(0.35)
    sim.hold()
    sim.say("02_6")
    sim.play(3 * 0.12)
    sim.play(0.35, "end formula")
    sim.hold()
    return {
        "02_1 table built, seconds after sentence 2 (boxes) starts": table_built - sim.sentence_at("02_1", 2),
        "02_1 question shown, seconds before sentence 4 asks it": sim.sentence_at("02_1", 4) - question,
        "02_3 first curve, seconds after sentence 4 (watch) starts": first_curve - sim.sentence_at("02_3", 4),
        "02_3 last curve done, seconds after the cue's voice ends": curves_done
        - (sim.said["02_3"] + sim.length("02_3")),
        "02_5 no formula on screen, seconds from cue start": ratio - empty_from,
        "total seconds": sim.t,
    }


def product(sim):
    sim.say("03_1")
    sim.play(0.35)
    sim.play(8 * 0.35, "edge terms")
    edges = sim.t
    sim.until("03_1", 3)
    sim.play(16 * 0.35, "sixteen cells")
    sim.play(5 * 0.35)
    sim.play(0.35, "corner note")
    sim.hold()
    sim.say("03_2")
    for degree, sentence in ((0, 1), (1, 2), (2, 4)):
        sim.until("03_2", sentence)
        sim.play(0.3)
        if degree < 2:
            sim.play(0.35, f"total for degree {degree}")
    total_one = [e for e in sim.events if e[0] == "total for degree 1"][0][2]
    sim.play(0.35, "question on screen")
    question = sim.t
    sim.hold()
    sim.play(4.0)
    sim.play(0.25)
    sim.say("03_3")
    sim.play(2 * 0.25 + 0.35)
    for sentence in (1, 3):
        sim.until("03_3", sentence)
        sim.play(0.6)
    sim.hold()
    sim.say("03_4")
    sim.play(0.3 + 0.35)
    sim.until("03_4", 1)
    sim.play(0.35)
    sim.hold()
    sim.say("03_5")
    sim.play(0.35)
    sim.until("03_5", 3)
    sim.play(0.35)
    sim.hold()
    sim.play(0.5)
    sim.say("03_6")
    sim.play(0.35)
    nonzero = sim.t
    sim.until("03_6", 2)
    sim.play(0.35)
    positive = sim.t
    sim.hold()
    return {
        "03_1 edges done, seconds before sentence 2 names them": sim.sentence_at("03_1", 2) - edges,
        "03_2 x+y total shown, seconds before sentence 3 says it": sim.sentence_at("03_2", 3) - total_one,
        "03_2 question shown, seconds before sentence 5 asks it": sim.sentence_at("03_2", 5) - question,
        "03_6 E(0)=1 shown, seconds before sentence 1 says it": sim.sentence_at("03_6", 1) - nonzero,
        "03_6 positive shown, seconds before sentence 3 says it": sim.sentence_at("03_6", 3) - positive,
        "total seconds": sim.t,
    }


series = load("exponential-expanded-series01/script-slice.tsv")
grid = load("exponential-expanded-product01/script-slice.tsv")
for name, scene, script in (("factorials.rs", factorials, series), ("product_grid.rs", product, grid)):
    print(name)
    results = {factor: scene(Sim(script, factor)) for factor in (0.8, 1.0, 1.25)}
    for key in results[1.0]:
        print(f"  {key}: {results[1.0][key]:.2f}  ({results[0.8][key]:.2f} at 0.8, {results[1.25][key]:.2f} at 1.25)")
    for cue, text in script.items():
        print(f"  {cue}: {len(text.split())} words, {len(sentences(text))} sentences")
