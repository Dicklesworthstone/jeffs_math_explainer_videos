"""Adapt intact sample-partition reports to the native caption reader."""

from copy import deepcopy


def compatible_report(source):
    report = deepcopy(source)
    rate = report["meta"]["rate"]
    assert type(rate) is int and rate > 0
    for cue, sentences in report["lines"].items():
        next_start = 0
        for sentence in sentences:
            start = sentence["source_in_sample"]
            stop = sentence["source_out_sample"]
            assert type(start) is type(stop) is int, cue
            assert next_start == start < stop, cue
            assert stop - start == sentence["out_samples"], cue
            assert sentence["gap_after_samples"] == 0, cue
            values = {"start": start / rate, "end": stop / rate, "pause_cut_s": 0.0}
            for key, value in values.items():
                assert key not in sentence or sentence[key] == value, (cue, key)
                sentence[key] = value
            next_start = stop
    return report
