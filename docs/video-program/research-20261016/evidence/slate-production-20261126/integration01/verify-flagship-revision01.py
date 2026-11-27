"""Bind the corrected script and verify the two mathematical repairs exactly."""

import csv
from fractions import Fraction
from hashlib import file_digest
import json
from math import comb
from pathlib import Path


HERE = Path(__file__).resolve().parent
EVIDENCE = HERE.parents[1]
FLAGSHIP = EVIDENCE / "applied-probability-20261126/flagship"
PREVIOUS = HERE / "script02.tsv"
SCRIPT = HERE / "script03.tsv"
CRATE = HERE / "probability-voice03"
REVIEW = HERE.parent / "probability-native/independent-script-review01.md"
REVISED = {"03_1", "06_2", "06_3", "07_1", "07_2", "14_1"}


def read_script(path):
    with path.open(newline="") as handle:
        return {row["id"]: row["text"] for row in csv.DictReader(handle, delimiter="\t")}


def digest(path):
    with path.open("rb") as handle:
        return file_digest(handle, "sha256").hexdigest()


def revision_cues():
    old, new = read_script(PREVIOUS), read_script(SCRIPT)
    assert list(old) == list(new) and len(new) == 41
    changed = {cue for cue in new if old[cue] != new[cue]}
    assert changed == REVISED
    pending = {cue: text for cue, text in new.items() if cue in changed or int(cue[:2]) >= 12}
    assert len(pending) == 13
    native = dict(line.split("\t", 1) for line in (CRATE / "script.tsv").read_text().splitlines()
                  if line and not line.startswith("#"))
    assert list(native.items()) == list(new.items())
    return new, pending


def exact_lattice():
    figure = json.loads((FLAGSHIP / "figure-facts02.json").read_text())["first_hit_lattice"]
    n = len(figure)
    live = [1]
    first_alarms, cutoff_list = [], []
    for t in range(1, n + 1):
        weights = [comb(t, h) * 9 ** (t - h) for h in range(t + 1)]
        cutoff = next(h for h in range(t + 2) if 20 * sum(weights[h:]) <= 10 ** t)
        next_live = [0] * (t + 1)
        for h, w in enumerate(live):
            next_live[h] += 9 * w
            next_live[h + 1] += w
        first = Fraction(sum(next_live[cutoff:]), 10 ** t)
        live = next_live[:cutoff]
        assert first == Fraction(figure[t - 1]["first_alarm"]) and cutoff == figure[t - 1]["cutoff"]
        first_alarms.append(first)
        cutoff_list.append(cutoff)
    zero = [t for t, mass in enumerate(first_alarms, 1) if not mass]
    assert n == 41 and zero == [1, 4, 9, 15, 21, 28, 35]
    return {"looks": n, "possible_first_alarms": n - len(zero), "zero_first_alarm_frames": zero,
            "cutoffs": cutoff_list, "total_first_alarm_probability": str(sum(first_alarms))}


def verify_factor():
    p0 = Fraction(1, 10)
    one_example = (1 - Fraction(1, 20)) / (1 - p0)
    assert one_example == Fraction(19, 18)
    for p in (Fraction(0), Fraction(1, 20), p0):
        for q in (Fraction(1, 20), p0, Fraction(3, 10), Fraction(1)):
            mean = p * q / p0 + (1 - p) * (1 - q) / (1 - p0)
            assert mean - 1 == (p - p0) * (q - p0) / (p0 * (1 - p0))
            assert (q >= p0 and mean <= 1) or (q < p0 and mean >= 1)
    return {"null_upper_rate": str(p0), "minimum_alternative_rate": str(p0),
            "counterexample_true_rate": "0", "counterexample_alternative": "1/20",
            "counterexample_mean_multiplier": str(one_example)}


def main():
    speech, pending = revision_cues()
    paths = (PREVIOUS, SCRIPT, REVIEW, CRATE / "script.tsv", Path(__file__).resolve(),
             FLAGSHIP / "figure-facts02.json")
    print(json.dumps({
        "files_sha256": {str(path.relative_to(EVIDENCE)): digest(path) for path in paths},
        "script_sha256": digest(SCRIPT),
        "cues": list(speech),
        "changed": [cue for cue in speech if cue in REVISED],
        "reused_originals": [cue for cue in speech if cue not in pending],
        "pending": {cue: len(text) for cue, text in pending.items()},
        "pending_characters": sum(map(len, pending.values())),
        "first_alarm_recalculation": exact_lattice(),
        "bet_recalculation": verify_factor(),
        "audio_review": False,
    }, indent=2))


if __name__ == "__main__":
    main()
