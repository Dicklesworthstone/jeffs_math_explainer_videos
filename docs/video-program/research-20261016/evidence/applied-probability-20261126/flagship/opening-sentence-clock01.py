"""Attach provisional word-derived sentence boundaries to intact 48 kHz takes."""

import csv
from difflib import SequenceMatcher
from hashlib import sha256
import json
from pathlib import Path
import re


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
RECOGNITION = WORK / "anytime-valid-frederick-opening-asr01"
SELECTION = HERE / "opening-provisional-selection01.json"
SCRIPT = HERE / "script01.tsv"
RATE = 48000
WORD = re.compile(r"[a-z]+(?:'[a-z]+)?|\d+(?:\.\d+)?%?")
NUMBERS = dict(zip(
    "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split(),
    map(str, range(20)),
))


def digest(path):
    with path.open("rb") as stream:
        return sha256(stream.read()).hexdigest()


def words(text):
    return [NUMBERS.get(word, word) for word in WORD.findall(text.lower().replace("’", "'"))]


def rows_for_cue(text, segments, samples):
    sentences = [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]
    expected = []
    starts = []
    for sentence in sentences:
        starts.append(len(expected))
        expected.extend(words(sentence))
    observed = [(word, seg["start_sec"]) for seg in segments for word in words(seg["text"])]
    matches = {}
    for kind, ea, eb, oa, ob in SequenceMatcher(
        None, expected, [word for word, _ in observed], autojunk=False,
    ).get_opcodes():
        if kind == "equal":
            matches.update((ea + i, (oa + i, "matched word")) for i in range(eb - ea))
        elif kind == "replace" and oa < ob:
            matches[ea] = (oa, "first word of recognized replacement")
    boundaries = [(0, "start of full take")]
    for i in starts[1:]:
        if i in matches:
            index, basis = matches[i]
            position = round(RATE * observed[index][1])
        else:
            left = max(k for k in matches if k < i)
            right = min(k for k in matches if k > i)
            before, after = (observed[matches[k][0]][1] for k in (left, right))
            position = round(RATE * (before + (after - before) * (i - left) / (right - left)))
            basis = "interpolated between recognized words"
        assert 0 < position < samples and position > boundaries[-1][0], (text, position, boundaries)
        boundaries.append((position, basis))
    ends = [n for n, _ in boundaries[1:]] + [samples]
    return [
        {"sentence": sentence, "source_in_sample": start, "source_out_sample": end,
         "out_samples": end - start, "gap_after_samples": 0, "boundary_basis": basis}
        for sentence, (start, basis), end in zip(sentences, boundaries, ends, strict=True)
    ]


def main():
    guard = json.loads((RECOGNITION / "resource-receipt.json").read_text())
    assert guard["exit"] == 0 and guard["finished"] and guard["inputs_sha256"] == guard["final_inputs_sha256"]
    choices = json.loads(SELECTION.read_text())
    assert not choices["perceptual_acceptance"]
    assert digest(SCRIPT) == choices["script_sha256"]
    with SCRIPT.open(newline="") as stream:
        script = {row["id"]: row["text"] for row in csv.DictReader(stream, delimiter="\t")}
    log = RECOGNITION / "command.log"
    reports = {Path(row["input"]): row for row in map(json.loads, log.read_text().splitlines())}
    assert len(reports) == 42
    lines = {}
    for choice in choices["selections"]:
        cue = choice["cue"]
        audio = REPO / choice["path"]
        assert digest(audio) == choice["sha256"] and choice["sample_rate"] == RATE
        report = reports[audio]
        assert report["status"] == "ok"
        lines[cue] = rows_for_cue(script[cue], report["report"]["result"]["segments"], choice["sample_frames"])
        assert sum(row["out_samples"] for row in lines[cue]) == choice["sample_frames"]
        for line in lines[cue]:
            line.update(source=choice["path"], source_sha256=choice["sha256"])
    assert list(lines) == list(script)[:21]
    print(json.dumps({
        "meta": {"rate": RATE, "script_sha256": choices["script_sha256"],
                 "selection_sha256": digest(SELECTION), "asr_log_sha256": digest(log),
                 "scope": "Provisional word-model boundaries for animating the original, unchanged audio. Listening and fine sync remain open."},
        "lines": lines,
    }, indent=2))


if __name__ == "__main__":
    main()
