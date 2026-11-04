#!/usr/bin/env python3
"""Timing and label arithmetic for the logarithm 01-04 source review.

Run from the repo root with: python3 -I -B <this file>
Reads script.tsv only. This is arithmetic, not a scene stand-in: it mirrors the
Kit's scratch estimate (words / 2.6 for speech, sentence onsets by character
share) and the beat lengths written in the chapter source. The time factors
0.8, 1.0 and 1.25 are illustrative. None of this is measured Oliver timing.
"""

SCRIPT = "explainer_corpus/logarithm_mysteries/script.tsv"
WPS = 2.6
FACTORS = (0.8, 1.0, 1.25)


def cue(cue_id):
    with open(SCRIPT, encoding="utf-8") as handle:
        for row in handle:
            if row.startswith(cue_id + "\t"):
                return row.rstrip("\n").split("\t", 1)[1]
    raise SystemExit("missing cue " + cue_id)


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return out


def starts(text, factor):
    parts = sentences(text)
    speech = len(text.split()) / WPS * factor
    total = sum(len(s) for s in parts)
    at, before = [], 0
    for s in parts:
        at.append(speech * before / total)
        before += len(s)
    return at, speech


def nudge_02_1():
    text = cue("02_1")
    # slide_rule.rs phrase_time: chars before the phrase over chars through ". "
    share = text.index("one that slides") / (text.index(". ") + 2)
    print("02_1: phrase 'one that slides', share of sentence 0 =", round(share, 3))
    for f in FACTORS:
        at, _ = starts(text, f)
        phrase = at[1] * share
        print(
            f"  x{f}: nudge now 1.40..2.60; phrase at {phrase:.2f}; "
            f"corrected nudge {phrase:.2f}..{phrase + 1.2:.2f}; sentence 1 at {at[1]:.2f}"
        )


def climb_04_2():
    text = cue("04_2")
    s3 = sentences(text)[3]
    before = s3.index("it only ever climbs")
    share = before / len(s3)
    print(f"04_2: 'it only ever climbs' at {before}/{len(s3)} chars of sentence 3, share {share:.4f}")
    for f in FACTORS:
        at, _ = starts(text, f)
        begin, end = at[3], at[4]
        phrase = begin + (end - begin) * share
        top = begin + 4.4
        moving = max(0.0, top - phrase)
        still = end - max(top, phrase)
        run = max(end - phrase - 0.3, 2.0)
        print(
            f"  x{f}: sentence {end - begin:.2f} s; tween now +0.40..+4.40; phrase at +{phrase - begin:.2f}; "
            f"across the phrase moving {moving:.2f} s, frozen {still:.2f} s; "
            f"corrected run {run:.2f} s, ending {end - phrase - run:.2f} s before sentence 4"
        )


def labels_04_4():
    text = cue("04_4")
    for f in FACTORS:
        at, _ = starts(text, f)
        s1, s2 = at[1], at[2]
        hold = max(0.0, s1 - 1.3)
        print(
            f"04_4 x{f}: S1 {s1:.2f}, S2 {s2:.2f}. Now: labels full strength 0.00 s, on screen 0.65 s, "
            f"flip ends {s1 + 3.25:.2f}, after-labels in {s1 + 3.65:.2f}. "
            f"Moved wait: labels in by 1.30, full strength {hold:.2f} s, "
            f"flip {s1 + 0.25:.2f}..{s1 + 2.85:.2f}, after-labels in {s1 + 3.25:.2f}"
        )


def unit_label_02():
    # CMR10 widths in ems; scene width = ems x size x 0.01.
    ems = {
        "0": 0.5, "3": 0.5, "8": 0.5, "9": 0.5, ".": 0.278, " ": 0.333, "o": 0.5, "f": 0.306,
        "a": 0.5, "u": 0.556, "l": 0.278, "e": 0.444, "n": 0.556, "g": 0.5, "t": 0.389, "h": 0.556,
    }
    label = "0.398 of a full length"
    width = sum(ems[c] for c in label) * 22 * 0.01
    # ONE_X -6, LEN 8, a = 2.5
    left, right = -6.0, -6.0 + 8 * 0.39794
    mid = (left + right) / 2
    print(
        f"02_2 label {label!r}: width {width:.2f}, centre {mid:.2f}, "
        f"span {mid - width / 2:.2f}..{mid + width / 2:.2f}; bar {left:.2f}..{right:.2f}"
    )


if __name__ == "__main__":
    print("Scratch-rule arithmetic only; not measured Oliver timing.")
    nudge_02_1()
    climb_04_2()
    labels_04_4()
    unit_label_02()
