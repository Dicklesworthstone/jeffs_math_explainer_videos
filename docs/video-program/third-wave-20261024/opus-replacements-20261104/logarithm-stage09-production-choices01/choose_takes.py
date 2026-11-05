#!/usr/bin/env python3
"""Provisional stage09 take selection and measured onset answers for root (Mail 4153, bead .63).

Run from the repository root, naming an output folder that does not exist yet:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-production-choices01/choose_takes.py OUT

Read-only on the repository except OUT, which gets four new files:
  selection.json   audio-preparation-selection-v1 for binding 2d7f4731, all 80 cues, each by its path;
  answers.json     audio-preparation-answers-v1: a `sample` answer at the measured energy onset for every
                   recognizer_early point of every selectable recording (the helper applies only those of
                   the selected recordings, so a changed selection keeps its answers), with the 08_5
                   download02 s2 exception; the report's four questions are left unanswered;
  evidence.tsv     one row per selectable recording: integrity screen, recognizer differences, pauses;
  choices.tsv      one row per cue: the choice, the rule that decided it, and the open discrepancies.

The rule, applied per cue to its selectable recordings, first difference wins:
  R1 directly recognized and free of blocking questions (00_1 download03/04 are PCM twins of download02);
  R2 fewer integrity flags (energy of the take's own 48 kHz samples, 10 ms frames):
       clip  - a run of 3 or more full-scale samples;
       edge  - speech (a frame at or above -40 dBFS) in the file's first or last 10 ms frame (these downloads
               normally start speech 0.01-0.10 s in, so a nearness threshold would flag ordinary takes);
       gap   - a quiet run (frames at or below -60 dBFS) of 2.0 s or more between the first and last speech;
       join  - a sentence boundary whose measured pause is shorter than 0.15 s (timing inspection 8c18a243);
  R3 fewer recognizer word differences that the other take does not share, after the reviewed notation
     pairs below (digits, spacing, homophones);
  R4 the longer shortest sentence-boundary pause, when the two differ by 0.05 s or more (more room at the
     tightest join; the inspection's pause_s, an energy measurement);
  R5 the lower download number.
Nothing here was heard. The rule cannot judge voice, accent, emphasis or cadence; root decides.
"""

import array
import csv
import hashlib
import io
import json
import math
import os
import sys
import wave
from operator import mul

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
STAGE09 = f"{P}/logarithm-stage09-audio-preparation01"
INSPECTION = f"{P}/logarithm-stage09-timing-inspection01/outputs/timing-inspection.json"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
SCRIPT = f"{EL}/combined-repair-stage09/candidate/explainer_corpus/logarithm_mysteries/script.tsv"
PINS = {
    f"{STAGE09}/SHA256SUMS": "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13",
    f"{STAGE09}/bindings/logarithm-stage09-oliver.json": "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803",
    f"{STAGE09}/report/anchors.json": None,
    f"{STAGE09}/report/questions.json": None,
    f"{STAGE09}/report/asr-differences.json": None,
    f"{P}/logarithm-stage09-timing-inspection01/SHA256SUMS": "8c18a24363d745e65be1a3ffec89b1d1bfc8d4c2a77185c85b0a2fcd805490bf",
    INSPECTION: "7d19519935c529227b42d2baf94f5614c841061526e4be7b2b9f8896b86a81de",
    SCRIPT: "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a",
}
BINDING_SHA = "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803"
BY = "BronzeJaguar, provisional for root (Mail 4153)"
FRAME, QUIET_DB, BODY_DB = 480, -60.0, -40.0
CLIP_RUN, GAP_S, JOIN_S, PAUSE_TIE_S, TIGHT_S = 3, 2.0, 0.15, 0.05, 0.25
EXCEPTIONS = {  # point id suffix -> (sample_24k, basis); timing inspection NOTE, clarity section, probe 4
    "onset:08_5:s2:narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/oliver-clarity-takes01/08_5-download02.wav":
        (167040, "energy: the vowel of 'each' rises at 6.97-6.99 s after the 0.50 s pause; the first frame above -60 dBFS (6.86 s, "
                 "sample 164640) is a small blip, so the answer is placed at 6.96 s (timing inspection 8c18a243, probe 4); not heard"),
}
NOTATION = {  # (script, heard) pairs reviewed as the recognizer's notation, spacing or a homophone, not a different reading
    ("thousandfold", "thousand fold"): "spacing of one word",
    ("by", "x"): "'two by two matrix' written as 2 x 2",
    ("too", "two"): "homophone",
}


def sha_file(p):
    with open(p, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def listed(sums, rel):
    for line in open(sums, encoding="utf8").read().splitlines():
        h, r = line.split("  ", 1)
        if r == rel:
            return h
    return None


def screen(path):
    """Integrity screen of one recording: (metrics, flags)."""
    with wave.open(path, "rb") as w:
        assert (w.getnchannels(), w.getsampwidth(), w.getframerate()) == (1, 2, 48000), path
        n = w.getnframes()
        a = array.array("h")
        a.frombytes(w.readframes(n))
    if sys.byteorder != "little":
        a.byteswap()
    peak = max(max(a), -min(a))
    full = a.count(32767) + a.count(-32768)
    run = best = 0
    if full:
        for x in a:
            run = run + 1 if x in (32767, -32768) else 0
            best = max(best, run)
    db = []
    for i in range(0, n, FRAME):
        seg = a[i:i + FRAME]
        e = sum(map(mul, seg, seg)) / len(seg)
        db.append(10 * math.log10(e / 32768 ** 2) if e > 0 else -120.0)
    body = [k for k, v in enumerate(db) if v >= BODY_DB]
    first, last = body[0], body[-1]
    gap = cur = 0
    for v in db[first:last + 1]:
        cur = cur + 1 if v <= QUIET_DB else 0
        gap = max(gap, cur)
    body_levels = sorted(db[k] for k in body)
    m = {"seconds": round(n / 48000, 3), "peak_dbfs": round(20 * math.log10(peak / 32768), 2), "full_scale_samples": full,
         "longest_full_scale_run": best, "lead_s": round(first * 0.01, 2), "tail_s": round(n / 48000 - (last + 1) * 0.01, 2),
         "longest_internal_quiet_s": round(gap * 0.01, 2), "median_speech_dbfs": round(body_levels[len(body_levels) // 2], 1)}
    flags = []
    if best >= CLIP_RUN:
        flags.append(f"clip: {best} consecutive full-scale samples")
    if first == 0 or last == len(db) - 1:
        flags.append(f"edge: speech in the first or last 10 ms frame (lead {m['lead_s']} s, tail {m['tail_s']} s)")
    if m["longest_internal_quiet_s"] >= GAP_S:
        flags.append(f"gap: {m['longest_internal_quiet_s']} s quiet inside the speech")
    return m, flags


def main():
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]):
        print("usage: choose_takes.py OUT (a folder that does not exist yet)")
        return 2
    out = sys.argv[1]
    stale = []
    for p, want in PINS.items():
        want = want or listed(f"{STAGE09}/SHA256SUMS", os.path.relpath(p, STAGE09))
        if sha_file(p) != want:
            stale.append(p)
    if stale:
        print("STALE: " + ", ".join(stale))
        return 3

    cues = [ln.split("\t", 1)[0] for ln in open(SCRIPT, encoding="utf8").read().splitlines() if ln.strip() and not ln.startswith("#")]
    recordings = json.load(open(f"{STAGE09}/report/anchors.json"))["recordings"]
    questions = json.load(open(f"{STAGE09}/report/questions.json"))["questions"]
    asr = {t["file"]: t["differences"] for t in json.load(open(f"{STAGE09}/report/asr-differences.json"))["transcripts"]}
    assert not any(f.startswith(("04_4", "08_5")) for f in asr), "a difference list could name a superseded take"
    rows = json.load(open(INSPECTION))["rows"]
    by_label = {}
    for r in rows:
        by_label.setdefault(r["id"].split(":", 3)[3], []).append(r)
    blocking = {f for q in questions if q["blocking"] for f in q["files"]}

    evidence = {}
    for e in recordings:
        m, flags = screen(e["path"])
        sent = [r for r in by_label.get(e["label"], []) if r["kind"] == "sentence"]
        pauses = [float(r["pause_s"]) for r in sent if r.get("pause_s") is not None]
        for r in sent:
            if r.get("pause_s") is None or float(r["pause_s"]) < JOIN_S:
                flags.append(f"join: {r['id'].split(':')[2]} pause {r.get('pause_s')} s ({r['status']})")
        early = [r for r in by_label.get(e["label"], []) if r.get("class") == "recognizer_early"]
        evidence[e["label"]] = {"e": e, "metrics": m, "flags": flags, "diffs": asr.get(e["file"], []),
                                "min_pause": min(pauses) if pauses else None, "early": early,
                                "eligible": e["file"] not in blocking}

    def key_diffs(label):
        return [(d["script"], d["heard"]) for d in evidence[label]["diffs"]]

    selection, choices = {}, []
    for cue in cues:
        labels = sorted(l for l, v in evidence.items() if v["e"]["cue"] == cue)
        pool = [l for l in labels if evidence[l]["eligible"]]
        decided, reason = None, None
        notes = []
        if len(pool) < len(labels):
            notes.append("R1 excludes " + ", ".join(os.path.basename(l) for l in labels if l not in pool) + " (PCM twins; blocking provenance question)")
        a, b = pool[0], pool[1]
        assert len(pool) == 2, cue
        fa, fb = len(evidence[a]["flags"]), len(evidence[b]["flags"])
        own = {}
        for x, y in ((a, b), (b, a)):
            extra = [d for d in key_diffs(x) if key_diffs(x).count(d) > key_diffs(y).count(d)]
            own[x] = [d for d in extra if d not in NOTATION]
            for d in extra:
                if d in NOTATION:
                    notes.append(f"{os.path.basename(x)} alone: recognizer '{d[1]}' for '{d[0]}' ({NOTATION[d]})")
        pa, pb = evidence[a]["min_pause"], evidence[b]["min_pause"]
        if fa != fb:
            decided, reason = (a if fa < fb else b), f"R2 integrity: {min(fa, fb)} flag(s) against {max(fa, fb)}"
        elif len(own[a]) != len(own[b]):
            decided, reason = (a if len(own[a]) < len(own[b]) else b), "R3 recognizer: fewer unshared word differences"
        elif pa is not None and pb is not None and abs(pa - pb) >= PAUSE_TIE_S:
            decided, reason = (a if pa > pb else b), f"R4 room: shortest boundary pause {max(pa, pb):.2f} s against {min(pa, pb):.2f} s"
        else:
            decided = a
            reason = "R5 lower download number" + (f" (shortest boundary pauses {pa:.2f} / {pb:.2f} s)" if pa is not None else " (one sentence)")
        other = b if decided == a else a
        for l in (decided, other):
            for fl in evidence[l]["flags"]:
                notes.append(f"{os.path.basename(l)} {fl}")
        shared = [f"'{h}' for '{s}'" for s, h in key_diffs(decided) if (s, h) in key_diffs(other)]
        if shared:
            notes.append("both takes, recognizer: " + "; ".join(shared) + " (notation or a real reading; only listening can tell)")
        tight = [r for r in sorted(by_label.get(decided, []), key=lambda r: r["id"])
                 if r["kind"] == "sentence" and r.get("pause_s") is not None and JOIN_S <= float(r["pause_s"]) < TIGHT_S]
        for r in tight:
            notes.append(f"tight join: {r['id'].split(':')[2]} pause {r['pause_s']} s in the chosen take "
                         f"(other take's shortest {evidence[other]['min_pause']} s)")
        if cue == "04_3":
            notes.append("advisory voice question open for both takes (download names lack the voice)")
        if cue in ("04_4", "08_5"):
            notes.append("clarity takes: one recognizer (large-v3-turbo) only")
        blips = [r for r in evidence[decided]["early"] if r["body_24k"] - r["leave_24k"] > 2400 and r["id"] not in EXCEPTIONS]
        for r in blips:
            notes.append(f"{r['id'].split(':')[2]}: leave {r['leave_s']} s is {round(r['body_s'] - r['leave_s'], 2)} s before the body; the answer uses leave")
        selection[cue] = evidence[decided]["e"]["path"]
        choices.append({"cue": cue, "chosen": os.path.basename(decided), "path": evidence[decided]["e"]["path"], "other": os.path.basename(other),
                        "rule": reason, "answers_applied": len(evidence[decided]["early"]), "discrepancies": " | ".join(notes)})

    answers = {"schema": "audio-preparation-answers-v1", "binding_sha256": BINDING_SHA, "answers": {}}
    for q in questions:
        answers["answers"][q["id"]] = {"decision": None, "by": None, "basis": None}
    n_answers = 0
    for label, v in sorted(evidence.items()):
        for r in sorted(v["early"], key=lambda r: r["id"]):
            sample, basis = EXCEPTIONS.get(r["id"], (r["leave_24k"], None))
            if basis is None:
                basis = (f"energy: first 10 ms frame above -60 dBFS after a {r['pause_s']} s quiet run, at {r['leave_s']} s "
                         f"(body at or above -40 dBFS at {r['body_s']} s); the recognizer's onset {r['recognizer_onset_s']} s is "
                         f"{r['delta_leave_s']} s earlier (timing inspection 8c18a243); not heard")
            answers["answers"][r["id"]] = {"decision": "sample", "by": BY, "basis": basis, "sample_24k": int(sample)}
            n_answers += 1
    sel = {"schema": "audio-preparation-selection-v1", "binding_sha256": BINDING_SHA,
           "script_sha256": PINS[SCRIPT], "by": BY,
           "basis": "Deterministic rule R1-R5 of choose_takes.py on recognizer, energy and timing evidence; nothing heard. "
                    "Voice, accent, emphasis and cadence are not judged; root's decision.",
           "takes": selection}

    os.makedirs(out)
    with open(f"{out}/selection.json", "x", encoding="utf8") as f:
        f.write(json.dumps(sel, indent=1) + "\n")
    with open(f"{out}/answers.json", "x", encoding="utf8") as f:
        f.write(json.dumps(answers, indent=1) + "\n")
    cols = ["cue", "file", "path", "eligible", "seconds", "peak_dbfs", "full_scale_samples", "longest_full_scale_run", "lead_s", "tail_s",
            "longest_internal_quiet_s", "median_speech_dbfs", "shortest_boundary_pause_s", "recognizer_early_points", "recognizer_differences", "flags"]
    buf = io.StringIO()
    w = csv.writer(buf, delimiter="\t", lineterminator="\n")
    w.writerow(cols)
    for label in sorted(evidence, key=lambda l: (cues.index(evidence[l]["e"]["cue"]), l)):
        v = evidence[label]
        w.writerow([v["e"]["cue"], v["e"]["file"], v["e"]["path"], v["eligible"], *[v["metrics"][k] for k in cols[4:12]],
                    v["min_pause"], len(v["early"]), "; ".join(f"{d['op']} '{d['script']}'->'{d['heard']}'" for d in v["diffs"]),
                    " | ".join(v["flags"])])
    with open(f"{out}/evidence.tsv", "x", encoding="utf8") as f:
        f.write(buf.getvalue())
    buf = io.StringIO()
    w = csv.writer(buf, delimiter="\t", lineterminator="\n")
    w.writerow(list(choices[0]))
    for c in choices:
        w.writerow(list(c.values()))
    with open(f"{out}/choices.tsv", "x", encoding="utf8") as f:
        f.write(buf.getvalue())

    rules = {}
    for c in choices:
        rules[c["rule"].split(" ")[0]] = rules.get(c["rule"].split(" ")[0], 0) + 1
    flagged = {l: v["flags"] for l, v in evidence.items() if v["flags"]}
    print(f"{len(recordings)} recordings of {len(cues)} cues screened; flagged recordings {len(flagged)}")
    for l, fl in sorted(flagged.items()):
        print(f"  {os.path.basename(l)}: {' | '.join(fl)}")
    print(f"decided by: {dict(sorted(rules.items()))}")
    print(f"download01 chosen {sum(c['chosen'].endswith('download01.wav') for c in choices)}, download02 {sum(c['chosen'].endswith('download02.wav') for c in choices)}")
    print(f"answers: {n_answers} sample answers for every recognizer_early point; applying to the selection "
          f"{sum(c['answers_applied'] for c in choices)}; questions left open {len(questions)}")
    for name in ("selection.json", "answers.json", "evidence.tsv", "choices.tsv"):
        print(f"{sha_file(f'{out}/{name}')}  {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
