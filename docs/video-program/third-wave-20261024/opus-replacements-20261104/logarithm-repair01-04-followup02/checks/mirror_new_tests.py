#!/usr/bin/env python3
"""Mirror the numbers and source orders that the new Rust tests assert.

Run from the repo root with: python3 -I -B <this file>
Cargo is out of scope for this stage, so this catches a wrong tolerance or a
wrong search string before root's native run. It does not replace that run: it
proves nothing about types, the borrow checker or scene behaviour.
"""

import math

STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair01-04-followup02/candidate/explainer_corpus/logarithm_mysteries/"
SCRIPT = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage02/explainer_corpus/logarithm_mysteries/script.tsv"
failures = []


def check(name, ok, detail=""):
    print(("ok   " if ok else "FAIL ") + name + (f"  [{detail}]" if detail else ""))
    if not ok:
        failures.append(name)


def cue(cue_id):
    with open(SCRIPT, encoding="utf-8") as handle:
        for row in handle:
            if row.startswith(cue_id + "\t"):
                return row.rstrip("\n").split("\t", 1)[1]
    raise SystemExit("missing cue " + cue_id)


def scene(path):
    with open(STAGE + path, encoding="utf-8") as handle:
        return handle.read().split("#[cfg(test)]")[0]


def in_order(source, *needles):
    at = [source.find(n) for n in needles]
    return all(a >= 0 for a in at) and at == sorted(at), at


def onset(text, said, s3, s4, marks):
    """climb_onset, in Python."""
    phrase = "it only ever climbs"
    byte = text.find(phrase)
    if byte < 0:
        return None
    if byte in marks:
        return said + marks[byte]
    k = text[:byte].rfind(". ")
    start = 0 if k < 0 else k + 2
    k = text[byte:].find(". ")
    end = len(text) if k < 0 else byte + k + 1
    return s3 + (s4 - s3) * (byte - start) / (end - start)


text_04 = cue("04_2")
phrase = "it only ever climbs"
byte = text_04.find(phrase)
check("04_2 phrase occurs once", text_04.count(phrase) == 1, f"byte {byte}")
check("04_2 phrase is in sentence 3", text_04[:byte].count(". ") == 3)
check("a mark at the exact byte wins", onset(text_04, 10.0, 20.0, 30.0, {byte: 17.5}) == 27.5)
estimate = onset(text_04, 10.0, 20.0, 30.0, {byte + 1: 17.5})
check("a mark elsewhere is ignored: 54 of 139", abs(estimate - (20.0 + 10.0 * 54.0 / 139.0)) < 1e-9, f"{estimate:.6f}")
check("no marks gives the same estimate", onset(text_04, 10.0, 20.0, 30.0, {}) == estimate)
check("another line gives none", onset("Some other line.", 10.0, 20.0, 30.0, {}) is None)

undoing = scene("src/chapters/undoing_growth.rs")
ok, at = in_order(
    undoing,
    'kit.until_sentence(stage, "04_2", 3)',
    "super::draw::wait_until(stage, onset)",
    "tween(climb, CLIMB_TO, run)",
    'kit.until_sentence(stage, "04_2", 4)',
)
check("04_2 order: sentence 3, wait, climb, sentence 4", ok, str(at))
ok, at = in_order(
    undoing,
    "fade_in_all(stage, &[run0, rise0], 0.4)",
    'kit.until_sentence(stage, "04_4", 1)',
    "&[name, run0, rise0, floor, nowhere]",
    "tween(mirror, 1.0, 2.6)",
)
check("04_4 order: labels, sentence 1, fade, flip", ok, str(at))
check("04_4 has one wait for sentence 1", undoing.count('kit.until_sentence(stage, "04_4", 1)') == 1)
check(r"no literal \ln 1000 in the scene", r"\ln 1000" not in undoing)
ok, at = in_order(undoing, r"\frac{{\ln {}}}", 'fact("real.growth_target")?')
check("the formula reads the target right after its template", ok, str(at))
template = undoing[undoing.find(r"t = \frac{{\ln {}}}"):]
template = template[: template.find('",')]
holes = template.replace("{{", "").replace("}}", "").count("{}")
arguments = undoing[undoing.find(r"t = \frac{{\ln {}}}"):]
arguments = arguments[: arguments.find("),\n")].count('fact("')
check("the formula's holes match its arguments", holes == arguments == 5, f"{holes} holes, {arguments} args")

text_02 = cue("02_1")
nudge = text_02.find("one that slides")
check("02_1 phrase is in the first sentence", 0 <= nudge < text_02.find(". "), f"byte {nudge}")
slide = scene("src/chapters/slide_rule.rs")
ok, at = in_order(
    slide,
    'until_phrase(stage, &kit, "02_1", NUDGE_PHRASE)',
    "v(0.8, 0.0)",
    "v(-0.8, 0.0)",
    'kit.until_sentence(stage, "02_1", 1)',
)
check("02_1 order: wait, out, back, sentence 1", ok, str(at))
check("the 02_2 label carries its unit", '"{} of a full length"' in slide)

# CMR10 widths in ems. Scene width = ems x size x 0.01.
EMS = {
    "0": 0.5, "3": 0.5, "8": 0.5, "9": 0.5, ".": 0.278, " ": 0.333, "o": 0.5, "f": 0.306,
    "a": 0.5, "u": 0.556, "l": 0.278, "e": 0.444, "n": 0.556, "g": 0.5, "t": 0.389, "h": 0.556,
}
ems = sum(EMS[c] for c in "0.398 of a full length")
check("the label's ems are at most the test's 9.06", ems <= 9.06, f"{ems:.3f}")
bar = 8.0 * math.log10(2.5)
check("the test's width is narrower than the bar", 9.06 * 22.0 * 0.01 < bar, f"{9.06 * 22.0 * 0.01:.3f} < {bar:.3f}")

target = 1000.0
ln_target = float(f"{math.log(target):.2f}")
check("ln of the target within the test's tolerance", abs(math.log(target) - ln_target) < 0.005, f"{ln_target}")
check("years from the target", round(math.log(target) / 0.05) == 138)

print("failures:", len(failures))
raise SystemExit(1 if failures else 0)
