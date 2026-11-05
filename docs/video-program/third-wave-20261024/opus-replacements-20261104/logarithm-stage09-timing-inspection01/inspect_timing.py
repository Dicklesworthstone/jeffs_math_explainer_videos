#!/usr/bin/env python3
"""Read-only inspection of the stage09 report's remaining timing questions (Mail 3960, bead .63). Version 5.

Run from the repository root:

    python3 -I -B <folder>/inspect_timing.py FRESH_OUT_FOLDER

Inputs, all read-only: the sealed stage09 bundle (its SHA256SUMS is verified first; exit 3 if any
byte differs), the recordings its report names, the retained inventory, and root's PCM provenance
check (stage09-provenance-root-check01.json). It writes only into FRESH_OUT_FOLDER, which must not
exist: timing-inspection.json and timing-rows.tsv.

1. Provenance. The report's two blocking questions say 00_1 download03 and download04 have the
   same samples as download02 and borrow its recognition. This recomputes each 00_1 file's frame
   count, PCM hash and file hash from the WAV, compares them with the inventory, the report and
   root's check, and quotes root's decision.

2. Onsets. For every supported sentence onset (after the file start) and every supported phrase
   mark of every selectable recording, this measures 10 ms RMS energy (dBFS) of the recording's own
   samples. The report flags the onsets that abut the previous word; the flag is kept on each row
   and the summary is split by it, because early onsets also occur where the recognizer left a
   gap. Frame k covers samples [k*hop, (k+1)*hop), so frame k is k/100 s and k*240 samples at
   24 kHz. Parameters (PARAMS, also written to the output):
   - The window runs from 0.3 s before the recognizer's start to the end of its segment.
   - A quiet run is consecutive frames at or below -60 dBFS lasting at least 0.05 s. A run is
     measured in full, also where it begins before the window (up to 1.5 s before it).
   - The pause is the longest quiet run whose end falls inside the window (the later on a tie).
   - leave is the first frame above -60 dBFS after the pause; body is the first frame at or above
     -40 dBFS after it. The word's energy begins inside [leave, body] to frame resolution; a soft
     attack can start a frame or two before leave at levels below -60 dBFS.
   - A pause shorter than 0.15 s is reported as a short quiet run (it may be a stop closure inside
     a word, or a floor that crosses -60 dBFS) and is not classified.
   - Classes, for rows with a pause:
       recognizer_early          leave is at least 0.2 s after the recognizer's onset, so the
                                 recognizer's segment begins inside the pause;
       energy_before_recognizer  body is at least 0.2 s before the recognizer's onset. This can
                                 be the previous word: its start is not in the report, so this
                                 class is a pointer for inspection, not a finding;
       within                    otherwise.

Version 1 (superseded/inspect_timing-v1.py) took the last quiet run and stepped back over every frame above
-75 dBFS. Probes showed both rules fail: a pause floor between -75 and -60 dBFS (04_4 download01)
was swallowed whole, and a 70 ms dip after a pre-onset blip (08_5 download02) was taken as the
pause. Version 2 replaced those two rules and still measured only abutting points and the clarity
recordings. Version 3 measured every supported point with version 2's method. Version 4 added the
loudest frame from the recognizer's onset to leave, which only measured the attack frames just
before leave. Version 5 reports instead, for rows with a pause, the median frame level from the
recognizer's onset to leave (median_db_recognizer_to_leave: near the file floor means the
recognizer's segment begins in silence) and pause_start_minus_recognizer_s (positive: the
measured pause starts after the recognizer's onset, either because the onset is in the previous
word's decay or because a floor that crosses -60 dBFS split the pause). Versions 1-4 are kept in
superseded/.

All of this measures sample levels. It is not listening, not a heard boundary and not a selection.
It is offered so root can decide whether to give `sample` answers; nothing here writes an answers
or selection file.
"""

import array
import hashlib
import json
import math
import os
import sys
import wave

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BUNDLE = f"{P}/logarithm-stage09-audio-preparation01"
REPORT = f"{BUNDLE}/report"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
ROOT_CHECK = f"{EL}/stage09-provenance-root-check01.json"
CLARITY_DIR = f"{EL}/combined-repair-stage09/oliver-clarity-takes01"
HOP_S = 0.01
FRAMES_PER_S = 100
QUIET_DB = -60.0
BODY_DB = -40.0
MIN_RUN_FRAMES = 5      # 0.05 s
PAUSE_FRAMES = 15       # 0.15 s
LEAD_FRAMES = 30        # 0.3 s
EXTEND_FRAMES = 150     # read 1.5 s beyond the window so that runs are measured in full
MATERIAL_S = 0.2
OUT_PER_FRAME = 240     # 24 kHz samples per 10 ms frame
PARAMS = {"frame_s": HOP_S, "quiet_db": QUIET_DB, "body_db": BODY_DB, "min_run_s": MIN_RUN_FRAMES / FRAMES_PER_S,
          "pause_s": PAUSE_FRAMES / FRAMES_PER_S, "lead_s": LEAD_FRAMES / FRAMES_PER_S,
          "extend_s": EXTEND_FRAMES / FRAMES_PER_S, "material_s": MATERIAL_S}


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(p):
    with open(p, "rb") as f:
        return sha_bytes(f.read())


def verify_seal():
    bad = []
    for line in open(f"{BUNDLE}/SHA256SUMS", encoding="utf8").read().splitlines():
        want, rel = line.split("  ", 1)
        if not os.path.isfile(f"{BUNDLE}/{rel}") or sha_file(f"{BUNDLE}/{rel}") != want:
            bad.append(rel)
    return bad


class Wav:
    def __init__(self, path):
        with wave.open(path, "rb") as w:
            assert (w.getnchannels(), w.getsampwidth()) == (1, 2), path
            self.rate, self.n = w.getframerate(), w.getnframes()
            self.pcm = array.array("h", w.readframes(self.n))
        assert sys.byteorder == "little"
        self.hop = self.rate // FRAMES_PER_S
        assert self.hop * FRAMES_PER_S == self.rate, path
        self.nk = self.n // self.hop
        self.cache = {}

    def db(self, k):
        if k not in self.cache:
            seg = self.pcm[k * self.hop:(k + 1) * self.hop]
            rms = math.sqrt(sum(x * x for x in seg) / len(seg))
            self.cache[k] = 20 * math.log10(rms / 32768) if rms > 0 else -120.0
        return self.cache[k]


def t(k):
    return round(k / FRAMES_PER_S, 2)


def measure(wav, asr_start, asr_end):
    lo = max(0, round(asr_start * FRAMES_PER_S) - LEAD_FRAMES)
    hi = min(wav.nk, round(asr_end * FRAMES_PER_S))
    a0, b0 = max(0, lo - EXTEND_FRAMES), min(wav.nk, hi + EXTEND_FRAMES)
    runs, k = [], a0
    while k < b0:
        if wav.db(k) <= QUIET_DB:
            j = k
            while j < b0 and wav.db(j) <= QUIET_DB:
                j += 1
            if j - k >= MIN_RUN_FRAMES:
                runs.append((k, j))
            k = j
        else:
            k += 1
    cands = [(a, b) for a, b in runs if lo < b <= hi and b < b0]
    out = {"window_start_s": t(lo), "window_end_s": t(hi), "quiet_runs_ending_in_window": len(cands)}
    if not cands:
        return {**out, "status": "no_quiet_run_ends_in_window"}
    a, b = max(cands, key=lambda r: (r[1] - r[0], r[1]))
    body = next((j for j in range(b, b0) if wav.db(j) >= BODY_DB), None)
    k_r = round(asr_start * FRAMES_PER_S)
    span = [wav.db(j) for j in range(k_r, b)]
    out["median_db_recognizer_to_leave"] = round(sorted(span)[len(span) // 2], 1) if span else None
    out["pause_start_minus_recognizer_s"] = round(a / FRAMES_PER_S - asr_start, 2)
    out.update({"pause_start_s": t(a), "pause_s": t(b - a), "pause_clipped_at_read_start": a == a0 and a0 > 0,
                "leave_s": t(b), "leave_24k": b * OUT_PER_FRAME,
                "body_s": t(body) if body is not None else None,
                "body_24k": body * OUT_PER_FRAME if body is not None else None,
                "delta_leave_s": round(b / FRAMES_PER_S - asr_start, 2),
                "delta_body_s": round(body / FRAMES_PER_S - asr_start, 2) if body is not None else None})
    if b - a < PAUSE_FRAMES:
        out["status"] = "short_quiet_run"
    else:
        out["status"] = "pause"
        if b / FRAMES_PER_S - asr_start >= MATERIAL_S - 1e-9:
            out["class"] = "recognizer_early"
        elif body is not None and asr_start - body / FRAMES_PER_S >= MATERIAL_S - 1e-9:
            out["class"] = "energy_before_recognizer"
        else:
            out["class"] = "within"
    return out


def stats(v):
    v = sorted(v)
    if not v:
        return None
    return {"n": len(v), "min": v[0], "median": v[len(v) // 2], "p90": v[min(len(v) - 1, int(0.9 * len(v)))], "max": v[-1]}


TSV_COLS = ["id", "cue", "file", "clarity", "kind", "word", "abuts_previous_word", "recognizer_onset_s", "recognizer_sample_24k",
            "segment_end_s", "status", "class", "pause_start_s", "pause_s", "leave_s", "leave_24k", "body_s", "body_24k",
            "delta_leave_s", "delta_body_s", "median_db_recognizer_to_leave", "pause_start_minus_recognizer_s", "quiet_runs_ending_in_window"]


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]):
        print("usage: from the repository root, inspect_timing.py FRESH_OUT_FOLDER (must not exist)", file=sys.stderr)
        return 2
    bad = verify_seal()
    if bad:
        print("STALE: the sealed stage09 bundle differs: " + ", ".join(bad))
        return 3
    out = sys.argv[1]
    anchors = json.load(open(f"{REPORT}/anchors.json"))
    questions = json.load(open(f"{REPORT}/questions.json"))["questions"]
    coverage = json.load(open(f"{REPORT}/coverage.json"))
    recs = anchors["recordings"]
    # 1. provenance
    root = json.load(open(ROOT_CHECK))
    rows = {os.path.basename(r["path"]): r for r in root["rows"]}
    inv = {os.path.basename(x["wav"]): x for x in json.load(open(f"{EL}/full80-retained-inventory01.json"))["takes"] if x["cue"] == "00_1"}
    prov = []
    for name in sorted(inv):
        path = f"{EL}/ui-takes01/{name}"
        with wave.open(path, "rb") as w:
            frames, rate = w.getnframes(), w.getframerate()
            pcm_sha = sha_bytes(w.readframes(frames))
        file_sha = sha_file(path)
        e = next((x for x in recs if x["file"] == name), None)
        r = rows.get(name, {})
        prov.append({"file": name, "frames": frames, "rate": rate, "pcm_sha256": pcm_sha, "file_sha256": file_sha,
                     "inventory_agrees": inv[name]["sha256"] == file_sha and inv[name]["pcm_sha256"] == pcm_sha and inv[name]["frames"] == frames,
                     "report_agrees": e is not None and e["pcm_sha256"] == pcm_sha and e["frames"] == frames,
                     "root_check_agrees": r.get("file_sha256") == file_sha and r.get("pcm_sha256") == pcm_sha and r.get("frames") == frames,
                     "recognized_from": e["recognition"]["recognized_file"] if e else None})
    provenance = {"root_check": ROOT_CHECK, "root_check_sha256": sha_file(ROOT_CHECK), "root_check_utc": root.get("utc"),
                  "root_decision": root.get("decision"), "root_limits": root.get("limits"),
                  "questions": [q["id"] for q in questions if q["kind"] == "provenance"], "files": prov,
                  "distinct_pcm": len({p["pcm_sha256"] for p in prov})}
    # 2. onsets
    timing = []
    for e in recs:
        clarity = e["path"].startswith(CLARITY_DIR + "/")
        wav = None
        for kind, pts in (("sentence", e["sentences"][1:]), ("mark", e["phrase_anchors"])):
            for s in pts:
                if s.get("status") != "supported":
                    continue
                wav = wav or Wav(e["path"])
                timing.append({"id": s["id"], "cue": e["cue"], "file": e["file"], "clarity": clarity, "kind": kind,
                               "word": s.get("word"), "abuts_previous_word": bool(s.get("abuts_previous_word")),
                               "recognizer_onset_s": s["asr_start_s"], "recognizer_sample_24k": s["sample_24k"],
                               "segment_end_s": s["asr_end_s"], **measure(wav, s["asr_start_s"], s["asr_end_s"])})
    groups = {f"{kind}_{'abutting' if flag else 'not_abutting'}": [r for r in timing if r["kind"] == kind and r["abuts_previous_word"] == flag]
              for kind in ("sentence", "mark") for flag in (True, False)}

    def tally(rs):
        c = {}
        for r in rs:
            key = r["status"] + (":" + r["class"] if "class" in r else "")
            c[key] = c.get(key, 0) + 1
        return dict(sorted(c.items()))

    def listing(cls):
        sel = [r for r in timing if r.get("class") == cls]
        return [{"id": r["id"], "abuts_previous_word": r["abuts_previous_word"], "recognizer_sample_24k": r["recognizer_sample_24k"],
                 "pause_start_s": r["pause_start_s"], "pause_s": r["pause_s"], "leave_24k": r["leave_24k"], "body_24k": r["body_24k"],
                 "delta_leave_s": r["delta_leave_s"], "delta_body_s": r["delta_body_s"],
                 "median_db_recognizer_to_leave": r["median_db_recognizer_to_leave"],
                 "pause_start_minus_recognizer_s": r["pause_start_minus_recognizer_s"]}
                for r in sorted(sel, key=lambda r: (-abs(r["delta_leave_s"] if cls == "recognizer_early" else r["delta_body_s"]), r["id"]))]

    summary = {
        "report_counts": {k: coverage["counts"].get(k) for k in sorted(coverage["counts"])},
        "rows": {g: len(v) for g, v in groups.items()},
        "clarity_rows": sum(1 for r in timing if r["clarity"]),
        "by_status": {g: tally(v) for g, v in groups.items()},
        "clarity_by_status": tally(r for r in timing if r["clarity"]),
        "delta_leave_s_pause_rows": {g: stats([r["delta_leave_s"] for r in v if r["status"] == "pause"]) for g, v in groups.items()},
        "recognizer_early_level": {
            "median_db_recognizer_to_leave_at_most_minus_75": sum(1 for r in timing if r.get("class") == "recognizer_early" and r["median_db_recognizer_to_leave"] <= -75),
            "median_db_recognizer_to_leave_above_minus_75": sum(1 for r in timing if r.get("class") == "recognizer_early" and r["median_db_recognizer_to_leave"] > -75),
            "recognizer_onset_before_pause_start": sum(1 for r in timing if r.get("class") == "recognizer_early" and r["pause_start_minus_recognizer_s"] > 0),
            "by_cue_count": len({r["cue"] for r in timing if r.get("class") == "recognizer_early"}),
            "recordings": len({r["id"].split(":", 3)[3] for r in timing if r.get("class") == "recognizer_early"}),
        },
        "recognizer_early": listing("recognizer_early"),
        "energy_before_recognizer": listing("energy_before_recognizer"),
    }
    os.makedirs(out)
    doc = {"claim": ("Sample-level energy measurements only: not listening, not a heard boundary, not a selection or an answer. "
                     "10 ms RMS frames of each recording's own samples; method and parameters as in the script's docstring."),
           "params": PARAMS, "stage09_seal": f"{BUNDLE}/SHA256SUMS", "stage09_seal_sha256": sha_file(f"{BUNDLE}/SHA256SUMS"),
           "binding_sha256": anchors["binding"]["sha256"], "provenance": provenance, "summary": summary, "rows": timing}
    with open(f"{out}/timing-inspection.json", "x", encoding="utf8") as f:
        f.write(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
    with open(f"{out}/timing-rows.tsv", "x", encoding="utf8") as f:
        f.write("\t".join(TSV_COLS) + "\n")
        for r in timing:
            f.write("\t".join("" if r.get(c) is None else str(r.get(c)) for c in TSV_COLS) + "\n")
    print(json.dumps({"provenance_files": [(p["file"], p["inventory_agrees"], p["report_agrees"], p["root_check_agrees"], p["recognized_from"]) for p in prov],
                      "distinct_pcm": provenance["distinct_pcm"],
                      **{k: summary[k] for k in summary if k not in ("recognizer_early", "energy_before_recognizer")},
                      "recognizer_early_n": len(summary["recognizer_early"]),
                      "energy_before_recognizer_n": len(summary["energy_before_recognizer"])}, indent=1))
    for r in timing:
        if r["clarity"]:
            print(f"{r['file']} {r['kind']} {r['word']!r}: recognizer {r['recognizer_onset_s']} s; {r['status']} {r.get('class') or ''}; "
                  f"pause {r.get('pause_start_s')}+{r.get('pause_s')} s; leave {r.get('leave_s')} body {r.get('body_s')}; "
                  f"delta leave {r.get('delta_leave_s')} body {r.get('delta_body_s')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
