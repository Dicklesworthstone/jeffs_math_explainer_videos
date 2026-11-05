#!/usr/bin/env python3
"""Read-only inspection of the stage09 report's remaining timing questions (Mail 3960, bead .63).

Run from the repository root:

    python3 -I -B <folder>/inspect_timing.py FRESH_OUT_FOLDER

Inputs, all read-only: the sealed stage09 bundle (its SHA256SUMS is verified first; exit 3 if any
byte differs), the recordings its report names, and root's PCM provenance check
(stage09-provenance-root-check01.json). It writes only into FRESH_OUT_FOLDER, which must not exist.

1. Provenance. The report's two blocking questions say 00_1 download03 and download04 have the
   same samples as download02 and borrow its recognition. This recomputes each 00_1 file's frame
   count and PCM hash from the WAV and compares them with the inventory, the report and root's check,
   and quotes root's decision.
2. Onsets that abut the previous word. A whisper-cpp word segment that starts exactly where the
   previous word ended may include the pause before the word. For every supported sentence onset
   and phrase mark marked `abuts_previous_word`, and for every onset and mark of the four clarity
   recordings, this reads the recording's samples inside the recognizer's segment (the clarity
   rows from 0.3 s before it) and measures 10 ms RMS energy. If a quiet run (every frame at most
   QUIET_DB) of at least MIN_PAUSE_S ends inside the window, the energy onset is the first frame
   after the last such run, moved back over adjacent frames above SOFT_DB.

The energy onset is a measurement of sample levels, not a heard boundary, not listening and not a
selection. It is offered so root can decide whether to give `sample` answers; nothing here writes
an answers or selection file.
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
QUIET_DB = -60.0
SOFT_DB = -75.0
MIN_PAUSE_S = 0.05
CLARITY_LEAD_S = 0.3
RATE_OUT = 24000


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
        self.w = wave.open(path, "rb")
        assert (self.w.getnchannels(), self.w.getsampwidth()) == (1, 2), path
        self.rate, self.n = self.w.getframerate(), self.w.getnframes()
        self.hop = round(self.rate * HOP_S)

    def frames_db(self, k0, k1):
        """RMS dBFS of 10 ms frames k0..k1-1."""
        k0, k1 = max(0, k0), min(k1, self.n // self.hop)
        self.w.setpos(k0 * self.hop)
        pcm = array.array("h", self.w.readframes((k1 - k0) * self.hop))
        out = []
        for i in range(k1 - k0):
            seg = pcm[i * self.hop:(i + 1) * self.hop]
            rms = math.sqrt(sum(x * x for x in seg) / len(seg)) if seg else 0.0
            out.append(20 * math.log10(rms / 32768) if rms > 0 else -120.0)
        return k0, out


def energy_onset(wav, start_s, end_s):
    k0, db = wav.frames_db(math.floor(start_s / HOP_S), math.ceil(end_s / HOP_S))
    runs, i = [], 0
    while i < len(db):
        if db[i] <= QUIET_DB:
            j = i
            while j < len(db) and db[j] <= QUIET_DB:
                j += 1
            runs.append((i, j))
            i = j
        else:
            i += 1
    runs = [(a, b) for a, b in runs if (b - a) * HOP_S >= MIN_PAUSE_S - 1e-9]
    if not runs:
        return {"pause_s": 0.0, "energy_onset_s": None, "note": "no quiet run of at least 50 ms in the window"}
    a, b = runs[-1]
    if b >= len(db):
        return {"pause_s": round((b - a) * HOP_S, 2), "energy_onset_s": None, "note": "the window ends in a quiet run"}
    j = b
    while j - 1 > a and db[j - 1] > SOFT_DB:
        j -= 1
    return {"pause_s": round((b - a) * HOP_S, 2), "pause_start_s": round((k0 + a) * HOP_S, 2),
            "energy_onset_s": round((k0 + j) * HOP_S, 2), "note": None}


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
    inv = {os.path.basename(t["wav"]): t for t in json.load(open(f"{EL}/full80-retained-inventory01.json"))["takes"] if t["cue"] == "00_1"}
    prov = []
    for name in sorted(inv):
        w = wave.open(f"{EL}/ui-takes01/{name}", "rb")
        pcm = w.readframes(w.getnframes())
        frames, w_rate = w.getnframes(), w.getframerate()
        w.close()
        e = next((x for x in recs if x["file"] == name), None)
        r = rows.get(name, {})
        prov.append({"file": name, "frames": frames, "rate": w_rate, "pcm_sha256": sha_bytes(pcm),
                     "file_sha256": sha_file(f"{EL}/ui-takes01/{name}"),
                     "inventory_agrees": inv[name]["sha256"] == sha_file(f"{EL}/ui-takes01/{name}") and inv[name]["pcm_sha256"] == sha_bytes(pcm) and inv[name]["frames"] == frames,
                     "report_agrees": e is not None and e["pcm_sha256"] == sha_bytes(pcm) and e["frames"] == frames,
                     "root_check_agrees": r.get("file_sha256") == sha_file(f"{EL}/ui-takes01/{name}") and r.get("pcm_sha256") == sha_bytes(pcm) and r.get("frames") == frames,
                     "recognized_from": e["recognition"]["recognized_file"] if e else None})
    pq = [q["id"] for q in questions if q["kind"] == "provenance"]
    provenance = {"root_check": ROOT_CHECK, "root_check_sha256": sha_file(ROOT_CHECK), "root_check_utc": root.get("utc"),
                  "root_decision": root.get("decision"), "root_limits": root.get("limits"), "questions": pq, "files": prov,
                  "distinct_pcm": len({p["pcm_sha256"] for p in prov})}
    # 2. timing rows
    timing = []
    for e in recs:
        clarity = e["path"].startswith(CLARITY_DIR + "/")
        wav = None
        for kind, rows_ in (("sentence", e["sentences"][1:]), ("mark", e["phrase_anchors"])):
            for s in rows_:
                if s.get("status") != "supported" or not (s.get("abuts_previous_word") or clarity):
                    continue
                wav = wav or Wav(e["path"])
                start = s["asr_start_s"] - (CLARITY_LEAD_S if clarity else 0.0)
                m = energy_onset(wav, start, s["asr_end_s"])
                row = {"id": s["id"], "cue": e["cue"], "file": e["file"], "clarity": clarity, "kind": kind,
                       "word": s.get("word") or s.get("phrase"), "recognizer_onset_s": s["asr_start_s"],
                       "recognizer_sample_24k": s["sample_24k"], "segment_end_s": s["asr_end_s"],
                       "abuts_previous_word": bool(s.get("abuts_previous_word")), "window_start_s": round(start, 6), **m}
                if m["energy_onset_s"] is not None:
                    row["energy_onset_24k"] = round(m["energy_onset_s"] * RATE_OUT)
                    row["delta_s"] = round(m["energy_onset_s"] - s["asr_start_s"], 2)
                timing.append(row)
    ab = [r for r in timing if r["abuts_previous_word"] and r["kind"] == "sentence"]
    with_onset = [r for r in ab if r.get("delta_s") is not None]
    deltas = sorted(r["delta_s"] for r in with_onset)
    q = lambda f: deltas[min(len(deltas) - 1, int(f * len(deltas)))] if deltas else None
    summary = {
        "abutting_sentence_onsets": len(ab), "report_count": coverage["counts"]["supported_onsets_abutting_previous_word"],
        "with_pause_before_word": len(with_onset),
        "without_pause_in_segment": sum(1 for r in ab if r.get("delta_s") is None),
        "delta_s": {"median": q(0.5), "p90": q(0.9), "max": deltas[-1] if deltas else None},
        "delta_at_least_0_3_s": sorted(({"id": r["id"], "delta_s": r["delta_s"]} for r in with_onset if r["delta_s"] >= 0.3), key=lambda r: (-r["delta_s"], r["id"])),
        "abutting_marks": sum(1 for r in timing if r["abuts_previous_word"] and r["kind"] == "mark"),
    }
    os.makedirs(out)
    doc = {"claim": ("Sample-level energy measurements only: not listening, not a heard boundary, not a selection or an answer. "
                     f"10 ms RMS frames; quiet <= {QUIET_DB} dBFS for >= {MIN_PAUSE_S} s; onset moved back over frames > {SOFT_DB} dBFS."),
           "stage09_seal": f"{BUNDLE}/SHA256SUMS", "stage09_seal_sha256": sha_file(f"{BUNDLE}/SHA256SUMS"),
           "binding_sha256": anchors["binding"]["sha256"], "provenance": provenance, "summary": summary, "rows": timing}
    with open(f"{out}/timing-inspection.json", "x", encoding="utf8") as f:
        f.write(json.dumps(doc, indent=1, ensure_ascii=False) + "\n")
    print(json.dumps({"provenance": {k: provenance[k] for k in ("questions", "distinct_pcm")},
                      "provenance_files": [(p["file"], p["inventory_agrees"], p["report_agrees"], p["root_check_agrees"], p["recognized_from"]) for p in prov],
                      "summary": summary}, indent=1))
    for r in timing:
        if r["clarity"]:
            print(f"{r['file']} {r['kind']} {r['word']!r}: recognizer {r['recognizer_onset_s']} s, energy {r['energy_onset_s']} s, pause {r['pause_s']} s, "
                  f"delta {r.get('delta_s')}, {r['note'] or ''}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
