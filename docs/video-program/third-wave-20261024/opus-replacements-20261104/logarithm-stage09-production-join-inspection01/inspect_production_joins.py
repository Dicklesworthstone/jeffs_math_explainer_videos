"""Read-only inspection of the actual stage09 production narration (87do.63, BlueSalmon Mail 4247).

LavenderPike, 2026-11-05. From the repository root:

    python3 -I -B P/logarithm-stage09-production-join-inspection01/inspect_production_joins.py FRESH_OUT

Reads root's production folder `EL/production-audio-stage09-01` (prepared01/narration, decoded01, prepared01-timings),
the 80 selected 48 kHz recordings, the recognizer runs through the installed helper's own loaders, Bronze's sealed
choices and timing rows, and root's hold audit. Writes only under FRESH_OUT. Nothing is decoded, assembled, timed or
transcribed again, and nothing is heard: every number is a sample count or a 10 ms RMS level.

Energy rule (BronzeJaguar's timing inspection, 8c18a243, applied here to the 24 kHz production files): frame k is samples
[240k, 240k+240); a quiet run is at least 5 consecutive frames at or below -60 dBFS; leave is the first frame above
-60 dBFS after a run; body is the first frame at or above -40 dBFS at or after leave.

For each sentence boundary (the sample where one sentence's caption and beat end and the next begin) and each phrase
mark, the pause is the quiet run containing the point if it lasts 0.15 s or more, else the longest quiet run within
+-1.0 s (+-0.5 s for marks), the nearer on a tie. The point is then before the pause (inside the previous speech), in the
pause (before_leave_s ahead of the next speech), or after leave (after_body_s > 0 means the next speech was already at
body level). Flags: before_pause; early (in the pause, 0.10 s or more before leave); late (after body); tight (pause
under 0.20 s); no_pause (no run of 0.15 s within the window); blip (a further quiet run between leave and body).
"""
import argparse
import array
import difflib
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import sys
import wave

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HELPER = f"{P}/logarithm-stage09-audio-preparation02/prepare_audio.py"
HELPER_SHA = "54df274a468be6122251efa7e4ccd7e11477fc8a93522e66de484ffb9be4eb0d"
BINDING = f"{P}/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json"
CHOICES = f"{P}/logarithm-stage09-production-choices01/inputs/choices.tsv"
TIMING_ROWS = f"{P}/logarithm-stage09-timing-inspection01/outputs/timing-rows.tsv"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
PROD = f"{EL}/production-audio-stage09-01"
NARR = f"{PROD}/prepared01/narration"
HOLD_AUDIT = f"{EL}/combined-repair-stage09/bound-plan/hold-audit.json"
RATE, HOP = 24000, 240
QUIET_DB, BODY_DB, MIN_RUN, PAUSE = -60.0, -40.0, 5, 15
EARLY, TIGHT = 10, 20  # frames
BLIP_ANSWERS = [("00_6", 1), ("04_5", 1), ("05_4", 1), ("06_6", 2), ("06_6", 4), ("08_2", 1), ("08_2", 3)]
PRIORITY = ["00_5", "04_4", "08_5"]
READ = {}


def sha(path):
    with open(path, "rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


def note(path):
    READ[str(path)] = sha(path)
    return path


def jload(path):
    return json.loads(Path(note(path)).read_text(encoding="utf-8"))


def pcm(path):
    with wave.open(str(note(path)), "rb") as w:
        params = (w.getnchannels(), w.getsampwidth(), w.getframerate())
        data = array.array("h", w.readframes(w.getnframes()))
    assert sys.byteorder == "little"
    return params, data


def levels(data):
    out = []
    for k in range(len(data) // HOP):
        seg = data[k * HOP:(k + 1) * HOP]
        rms = math.sqrt(sum(x * x for x in seg) / HOP)
        out.append(round(20 * math.log10(rms / 32768), 2) if rms > 0 else -120.0)
    return out


def quiet_runs(db):
    runs, k = [], 0
    while k < len(db):
        if db[k] <= QUIET_DB:
            j = k
            while j < len(db) and db[j] <= QUIET_DB:
                j += 1
            if j - k >= MIN_RUN:
                runs.append((k, j))
            k = j
        else:
            k += 1
    return runs


def s2(frames):
    return round(frames / 100, 2)


def place(db, runs, k, window):
    """Where frame k sits against the nearest pause."""
    containing = next(((a, b) for a, b in runs if a <= k < b), None)
    near = [(a, b) for a, b in runs if b > k - window and a < k + window]
    if containing and containing[1] - containing[0] >= PAUSE:
        main = containing
    else:
        long = [r for r in near if r[1] - r[0] >= PAUSE]
        main = max(long, key=lambda r: (r[1] - r[0], -abs((r[0] + r[1]) / 2 - k))) if long else None
    row = {"frame": k, "seconds": s2(k), "level_db": db[k] if k < len(db) else None,
           "in_quiet_run": containing is not None}
    flags = []
    if main is None:
        row.update(pause=None)
        flags.append("no_pause")
        last = max((b for a, b in runs if b <= k), default=None)
        row["speech_since_s"] = s2(k - last) if last is not None else None
        row["flags"] = flags
        return row
    a, b = main
    body = next((j for j in range(b, len(db)) if db[j] >= BODY_DB), None)
    between = [r for r in runs if b < r[0] and body is not None and r[1] <= body]
    row.update(pause={"start_s": s2(a), "leave_s": s2(b), "leave_24k": b * HOP, "pause_s": s2(b - a),
                      "body_s": s2(body) if body is not None else None, "leave_to_body_s": s2(body - b) if body is not None else None})
    if k < a:
        row["position"] = "before_pause"
        row["before_pause_s"] = s2(a - k)
        flags.append("before_pause")
    elif k < b:
        row["position"] = "in_pause"
        row["into_pause_s"], row["before_leave_s"] = s2(k - a), s2(b - k)
        if b - k >= EARLY:
            flags.append("early")
    else:
        row["position"] = "after_leave"
        row["after_leave_s"] = s2(k - b)
        if body is not None and k > body:
            row["after_body_s"] = s2(k - body)
            flags.append("late")
    if b - a < TIGHT:
        flags.append("tight")
    if between:
        row["quiet_runs_between_leave_and_body"] = [[s2(x), s2(y)] for x, y in between]
        flags.append("blip")
    row["flags"] = flags
    return row


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("out")
    a = parser.parse_args()
    if os.path.exists(a.out):
        print("FRESH_OUT must not exist", file=sys.stderr)
        return 2
    assert sha(HELPER) == HELPER_SHA, "the installed helper moved"
    spec = importlib.util.spec_from_file_location("installed_prepare_audio", note(HELPER))
    H = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(H)
    b = H.load_binding(note(BINDING))
    inputs = H.load_inputs(b)
    for rel, t in inputs["takes"].items():  # build_report's own preparation (lines 547-555), read-only
        assert sha(rel) == t["sha256"], f"{rel} changed on disk"
        params, frames, data = H.wav_info(rel)
        assert params[:2] == (1, 2) and params[3] == "NONE" and t.get("pcm_sha256", H.sha_bytes(data)) == H.sha_bytes(data), rel
        t.update(rate=params[2], frames=frames, pcm=H.sha_bytes(data))
    recognitions = H.recognition_for(inputs)
    script = inputs["script"]
    receipt = jload(f"{PROD}/prepared01/prepare-receipt.json")
    report = jload(f"{NARR}/report.json")
    spans = jload(f"{PROD}/prepared01/spans.json")
    timings = jload(f"{NARR}/phrase-timings.json")
    plan = jload(f"{PROD}/prepared01/phrase-marks-plan.json")
    decode = jload(f"{PROD}/decoded01/decode-receipt.json")
    holds_audit = jload(HOLD_AUDIT)
    hold_kind = {h["cue"]: "prediction" for h in holds_audit["prediction_holds"]}
    hold_kind.update({h["cue"]: "reading" for h in holds_audit["reading_holds"]})
    hold_floor = {h["cue"]: h.get("source_floor_after_voice_s") for h in holds_audit["prediction_holds"] + holds_audit["reading_holds"]}
    choices = {r.split("\t")[0]: r.split("\t") for r in Path(note(CHOICES)).read_text().splitlines()[1:]}
    discrepant = sorted(c for c, r in choices.items() if "both takes, recognizer" in r[6])
    bronze = {}
    for r in Path(note(TIMING_ROWS)).read_text().splitlines()[1:]:
        f = r.split("\t")
        bronze[f[0]] = f
    chain, problems = {}, []
    units = {u["id"]: u for u in spans["units"]}
    decoded = {r["cue"]: r for r in decode["records"]}
    cues = list(script)
    joins, marks, edges, diffs = [], [], [], []
    plan_marks = {c: e["marks"] if isinstance(e, dict) else e for c, e in plan["cues"].items()}
    for cue in cues:
        text = script[cue]
        wav_n = f"{NARR}/{cue}.wav"
        rec = receipt["cues"][cue]
        dpath = decoded[cue]["wav"]
        params, data = pcm(wav_n)
        dparams, ddata = pcm(dpath)
        sparams, sdata = pcm(rec["recording_path"])
        if params != (1, 2, RATE) or dparams != params or data != ddata:
            problems.append(f"{cue}: the narration WAV is not the decoded WAV sample for sample")
        if sparams[2] != 48000 or len(data) != -(-len(sdata) // 2):
            problems.append(f"{cue}: {len(data)} samples at 24 kHz from {len(sdata)} at {sparams[2]} Hz (not ceil(n/2))")
        rows = report["lines"][cue]
        unit = units[rows[0]["unit_id"]]
        want_sentences = [text[x:y] for x, y in H.kit_sentences(text)]
        if [r["sentence"] for r in rows] != want_sentences:
            problems.append(f"{cue}: the report's sentences are not the kit's split of the script")
        if [r["source_in_sample"] for r in rows] != [0] + [r["source_out_sample"] for r in rows[:-1]] or rows[-1]["source_out_sample"] != len(data) \
                or any(r["gap_after_samples"] or r["pause_cut_s"] for r in rows) or sum(r["out_samples"] for r in rows) != len(data):
            problems.append(f"{cue}: the report's spans are not the whole take, back to back, without gaps or cuts")
        if {r["source_sha256"] for r in rows} != {sha(dpath)} or unit["sha256"] != sha(dpath):
            problems.append(f"{cue}: report or spans name another decoded file")
        starts = [x["sample"] for x in rec["sentence_starts_24k"]]
        if [r["source_in_sample"] for r in rows[1:]] != starts or unit["sentence_ends"][:-1] != starts:
            problems.append(f"{cue}: boundaries differ between receipt, spans and report")
        db = levels(data)
        runs = quiet_runs(db)
        peak = max(abs(x) for x in data)
        speech = [k for k, v in enumerate(db) if v > QUIET_DB]
        bodies = [k for k, v in enumerate(db) if v >= BODY_DB]
        edges.append({"cue": cue, "seconds": round(len(data) / RATE, 3), "samples": len(data), "sentences": len(rows),
                      "lead_s": s2(speech[0]), "body_lead_s": s2(bodies[0]), "tail_s": s2(len(db) - 1 - speech[-1]),
                      "body_tail_s": s2(len(db) - 1 - bodies[-1]), "first_frame_db": db[0], "last_frame_db": db[-1],
                      "peak_dbfs": round(20 * math.log10(peak / 32768), 2), "full_scale_samples": sum(1 for x in data if x in (32767, -32768)),
                      "hold": hold_kind.get(cue), "hold_floor_after_voice_s": hold_floor.get(cue), "recording": rec["recording"]})
        result = recognitions[rec["recording_path"]]["result"]
        canon, obs, match, _run, _replaced = H.align(text, result)
        offsets = [x for x, _ in H.kit_sentences(text)]

        def sentence_of(char):
            return max(n for n, o in enumerate(offsets) if o <= char)

        bounds = [0] + [s // HOP for s in starts] + [len(db)]
        for n, x in enumerate(rec["sentence_starts_24k"], 1):
            k = x["sample"] // HOP
            row = place(db, runs, k, 100)
            first = next(i for i, (_, off) in enumerate(canon) if off >= offsets[n])
            o = obs[match[first]] if first in match else None
            row = {"cue": cue, "sentence": n, "sample_24k": x["sample"], "basis": "answer" if x["basis"].startswith("sample given") else "recognizer",
                   "first_word": canon[first][0], "recognizer_word_start_s": o["start"] if o else None,
                   "bronze_class": (bronze.get(f"onset:{cue}:s{n}:{rec['recording']}") or [None] * 12)[11], **row}
            joins.append(row)
        for m in timings["cues"][cue]["marks"]:
            k = round(m["seconds"] * 100)
            row = place(db, runs, k, 50)
            char = len(text.encode()[: m["byte"]].decode())
            i = next(ix for ix, (_, off) in enumerate(canon) if off == char)
            o = obs[match[i]] if i in match else None
            n = sentence_of(char)
            words = " ".join(text.encode()[m["byte"]:].decode().split()[:4])
            marks.append({"cue": cue, "byte": m["byte"], "seconds": m["seconds"], "basis": m["basis"], "words": words, "sentence": n,
                          "from_sentence_onset_s": s2(k - bounds[n]), "to_next_boundary_s": s2(bounds[n + 1] - k),
                          "recognizer_word": o["w"] if o else None, "recognizer_start_s": o["start"] if o else None,
                          "previous_word_end_s": obs[match[i] - 1]["end"] if o and match[i] > 0 else None,
                          "in_plan": any(p.get("byte") == m["byte"] for p in plan_marks.get(cue, [])), **row})
        if cue in discrepant:
            a_words, o_words = [w for w, _ in canon], [x["w"] for x in obs]
            for op, i1, i2, j1, j2 in difflib.SequenceMatcher(a=a_words, b=o_words, autojunk=False).get_opcodes():
                if op == "equal":
                    continue
                if j2 > j1:
                    t0, t1 = obs[j1]["start"], obs[j2 - 1]["end"]
                else:
                    t0 = obs[j1 - 1]["end"] if j1 > 0 else 0.0
                    t1 = obs[j1]["start"] if j1 < len(obs) else len(data) / RATE
                char = canon[i1][1] if i1 < len(canon) else canon[-1][1]
                near = [f"s{n} boundary" for n, s in enumerate(starts, 1) if t0 - 0.3 <= s / RATE <= t1 + 0.3]
                near += [f"mark b{m['byte']}" for m in timings["cues"][cue]["marks"] if t0 - 0.3 <= m["seconds"] <= t1 + 0.3]
                diffs.append({"cue": cue, "op": op, "script": " ".join(a_words[i1:i2]), "heard": " ".join(o_words[j1:j2]),
                              "start_s": round(t0, 2), "end_s": round(t1, 2), "start_24k": round(t0 * RATE), "end_24k": round(t1 * RATE),
                              "sentence": sentence_of(char), "near": near})
    if timings["prepare_receipt_sha256"] != sha(f"{PROD}/prepared01/prepare-receipt.json") or \
            Path(f"{NARR}/phrase-timings.json").read_bytes() != Path(note(f"{PROD}/prepared01-timings/phrase-timings.json")).read_bytes():
        problems.append("phrase-timings.json is not the timings job's file for this prepare receipt")
    for cue in cues:
        if timings["cues"][cue]["wav_sha256"] != sha(f"{NARR}/{cue}.wav"):
            problems.append(f"{cue}: phrase timings name another WAV")
    chain = {"cues": len(cues), "boundaries": len(joins), "marks": len(marks), "problems": problems,
             "checked": ["narration WAV = decoded WAV, sample for sample, 24 kHz mono 16-bit",
                         "decoded length = ceil(source frames / 2) of the 48 kHz recording",
                         "report sentences = the kit's split of the script; spans back to back over the whole take; no gap or cut",
                         "boundaries equal in prepare receipt, spans and report",
                         "installed phrase-timings = the timings job's copy, bound to this prepare receipt and each WAV"]}
    out = Path(a.out)
    out.mkdir(parents=True)

    def tsv(name, rows, keys):
        lines = ["\t".join(keys)]
        for r in rows:
            lines.append("\t".join("" if r.get(k) is None else (json.dumps(r[k]) if isinstance(r[k], (dict, list)) else str(r[k])) for k in keys))
        (out / name).write_text("\n".join(lines) + "\n")

    tsv("joins.tsv", joins, ["cue", "sentence", "sample_24k", "seconds", "basis", "first_word", "recognizer_word_start_s", "bronze_class",
                             "level_db", "position", "before_pause_s", "into_pause_s", "before_leave_s", "after_leave_s", "after_body_s",
                             "pause", "quiet_runs_between_leave_and_body", "flags"])
    tsv("marks.tsv", marks, ["cue", "byte", "seconds", "words", "sentence", "from_sentence_onset_s", "to_next_boundary_s", "recognizer_word",
                             "recognizer_start_s", "previous_word_end_s", "level_db", "position", "before_pause_s", "into_pause_s",
                             "before_leave_s", "after_leave_s", "after_body_s", "speech_since_s", "pause", "flags"])
    tsv("cue-edges.tsv", edges, ["cue", "recording", "seconds", "samples", "sentences", "lead_s", "body_lead_s", "tail_s", "body_tail_s",
                                 "first_frame_db", "last_frame_db", "peak_dbfs", "full_scale_samples", "hold", "hold_floor_after_voice_s"])
    tsv("discrepancies.tsv", diffs, ["cue", "sentence", "op", "script", "heard", "start_s", "end_s", "start_24k", "end_24k", "near"])
    flagged = {f: sorted((r for r in joins if f in r["flags"]), key=lambda r: (r["cue"], r["sentence"])) for f in
               ("before_pause", "early", "late", "tight", "no_pause", "blip")}
    summary = {"chain": chain, "inputs": dict(sorted(READ.items())),
               "parameters": {"frame_s": 0.01, "quiet_db": QUIET_DB, "body_db": BODY_DB, "min_run_s": 0.05, "pause_s": 0.15,
                              "early_s": 0.10, "tight_s": 0.20, "boundary_window_s": 1.0, "mark_window_s": 0.5},
               "join_flags": {f: [f"{r['cue']} s{r['sentence']}" for r in rows] for f, rows in flagged.items()},
               "mark_flags": {f: [f"{r['cue']} b{r['byte']}" for r in marks if f in r["flags"]] for f in
                              ("before_pause", "early", "late", "tight", "no_pause", "blip")},
               "priority": {cue: {"edges": next(e for e in edges if e["cue"] == cue), "joins": [r for r in joins if r["cue"] == cue],
                                  "marks": [r for r in marks if r["cue"] == cue]} for cue in PRIORITY},
               "blip_answers": [next(r for r in joins if r["cue"] == c and r["sentence"] == n) for c, n in BLIP_ANSWERS],
               "discrepant_cues": discrepant,
               "claim": "Sample counts and 10 ms RMS levels of the production files only. Nothing was heard; no boundary, mark or "
                        "take is accepted or rejected here."}
    (out / "inspection.json").write_text(json.dumps(summary, indent=1) + "\n")
    print(f"chain: {len(cues)} cues, {len(joins)} boundaries, {len(marks)} marks; problems: {problems or 'none'}")
    for f, rows in flagged.items():
        print(f"join {f}: {len(rows)}" + (": " + ", ".join(f"{r['cue']} s{r['sentence']}" for r in rows) if rows else ""))
    for f in ("before_pause", "early", "late", "tight", "no_pause", "blip"):
        got = summary["mark_flags"][f]
        print(f"mark {f}: {len(got)}" + (": " + ", ".join(got) if got else ""))
    print(f"discrepant cues {len(discrepant)}: {len(diffs)} recognizer differences located")
    print("input digest", hashlib.sha256(json.dumps(summary["inputs"], sort_keys=True).encode()).hexdigest())
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())
