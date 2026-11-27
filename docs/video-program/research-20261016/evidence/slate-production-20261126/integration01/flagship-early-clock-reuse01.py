"""Reuse exact native schedule rows when the final package retains their voice."""

from hashlib import file_digest
import json
import os
from pathlib import Path


TABLES = ("timeline.tsv", "cues.tsv", "cuelog.tsv")


def read(path):
    return json.loads(path.read_text())


def sha(path):
    with path.open("rb") as stream:
        return file_digest(stream, "sha256").hexdigest()


def bind(work, keys, narration, lines, binary, script):
    """Return the reusable schedules and each file their checks rely on."""
    reusable, paths = {}, set()
    if (narration / "phrase-timings.json").exists():
        return reusable, paths
    binary_sha, script_sha = sha(binary), sha(script)
    for key in keys:
        prior = work / f"anytime-valid-early-picture01-{key[:2]}"
        guard = prior / "resource-receipt.json"
        if not guard.is_file():
            continue
        job = read(guard)
        if (not job.get("finished") or job.get("exit") != 0 or not job["admitted"]
                or job["phase"] != "native" or job.get("stopped_for_resource_limit")
                or job.get("stop_reason") or job.get("error") or not job.get("inputs_sha256")
                or job.get("final_inputs_sha256") != job["inputs_sha256"]):
            continue
        manifest = work / "navier-production-20261120" / f"{prior.name}-inputs.json"
        assert sha(manifest) == job["inputs_sha256"]
        receipt = prior / "early-picture-receipt01.json"
        record = read(receipt)
        if record["binary_sha256"] != binary_sha or record["script_sha256"] != script_sha:
            continue
        assert record["chapter"]["key"] == key
        source = prior / "narration"
        early_report = source / "report.json"
        assert not (source / "phrase-timings.json").exists()
        assert sha(early_report) == record["narration_report_sha256"]
        early = read(early_report)
        cue_ids = [row["cue"] for row in record["chapter"]["cues"]]
        assert early["meta"]["rate"] == 48000
        assert early["meta"]["script_sha256"] == script_sha
        assert cue_ids == list(early["lines"]) == list(record["narration"])
        assert cue_ids == [cue for cue in lines if cue.startswith(key[:3])]
        if any(early["lines"][cue] != lines[cue] for cue in cue_ids):
            continue
        live_wavs = [narration / f"{cue}.wav" for cue in cue_ids]
        if any(sha(wav) != record["narration"][wav.stem]["sha256"] for wav in live_wavs):
            continue
        clock = prior / "clock60"
        tables = {ext: clock / f"{key}.{ext}" for ext in TABLES}
        assert all(sha(path) == record["sha256"][ext] for ext, path in tables.items())
        reusable[key] = (source, tables, tuple(cue_ids), record["chapter"]["frames"])
        paths.update((guard, receipt, manifest, early_report, *live_wavs, *tables.values()))
    if reusable:
        paths.add(Path(__file__).resolve())
    return reusable, paths


def relocate(output, narration, reusable):
    for key, (old_narration, tables, cue_ids, _) in reusable.items():
        for ext, source in tables.items():
            contents = source.read_bytes()
            if ext != "cues.tsv":
                for cue in cue_ids:
                    old = os.fsencode(old_narration / f"{cue}.wav")
                    new = os.fsencode(narration / f"{cue}.wav")
                    assert contents.count(old) == 1, (source, cue)
                    contents = contents.replace(old, new)
            with (output / f"{key}.{ext}").open("xb") as handle:
                handle.write(contents)
