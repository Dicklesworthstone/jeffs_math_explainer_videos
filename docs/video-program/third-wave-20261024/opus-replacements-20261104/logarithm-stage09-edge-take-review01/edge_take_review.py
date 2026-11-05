"""Stage09 edge-take review: the quiet-end alternative takes of 01_2, 07_6, 11_5 and 12_2 (87do.63, root Mail 4534).

BronzeJaguar, 2026-11-05. From the repository root:

    python3 -I -B P/logarithm-stage09-edge-take-review01/edge_take_review.py FRESH_OUT --fmn-sound SOUND_RS

SOUND_RS is franken_manim's crates/fmn-output/src/sound.rs at a399d626 (sha256 7bb983c0...), as for
timing-corrections01 (`git show a399d626:crates/fmn-output/src/sound.rs`). FRESH_OUT must not exist.

In each of the four cues the selected take (root's selection01) ends on sound and the other retained take of the same
request ends in quiet (timing-corrections01). This compares the two complete takes, cue by cue:

  A  words: the request text against the script; each take's recognizer word record (the run log the installed
     helper reads) and root's transcript review of the same log; script differences, shared or not
  B  timing coverage: the installed helper (54df274a) in process, on a check selection that swaps in the four
     alternatives (check-selection-swap4.json: an input for the helper's own load_selection, not a selection), under
     root's answers01 and timing-corrections01's answers02: blocking questions, every sentence end and phrase mark with
     its basis; each point placed against its pause by LavenderPike's rule (production-join-inspection01 place(), on
     10 ms frames of the 48 kHz source), with a control that repeats her rows for the four selected takes
  C  pauses: every internal quiet run of at least 0.10 s, at the script text it falls in, both takes side by side
  D  edges: how each take ends (speech level, the floor after it, LavenderPike's rise-in-the-last-10-ms test, 1 ms levels),
     the -40 dBFS screen and the -60 dBFS edges; for the selected takes the voice bus's last samples (the fmn
     resampler mirror on root's decoded files); for the alternatives, which have no decode, an estimate of the bus's
     last sample from the source's last 256 samples, calibrated on root's 80 decodes and checked on the selected takes
  E  the rule: production-choices01's R1-R5 replayed on its own evidence (it must give choices.tsv's choice and rule),
     then with R2's edge test moved from -40 dBFS speech to -60 dBFS sound

Writes only FRESH_OUT: takes.tsv, points.tsv, pauses.tsv, rules.tsv, edge-calibration.tsv, review.json,
check-selection-swap4.json, stdout.txt. Nothing is decoded, selected, answered, cut, faded, retimed, assembled or heard.
Transcripts, levels and periodicity are measurements, not listening.
"""

import array
import collections
import csv
import hashlib
import importlib.util
import io
import json
import math
import os
import sys
import wave
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-edge-take-review01"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
O = f"{EL}/production-audio-stage09-01"
SELECTION = f"{O}/selection01.json"
DECODE_RECEIPT = f"{O}/decoded01/decode-receipt.json"
BINDING = f"{P}/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json"
HELPER = f"{P}/logarithm-stage09-audio-preparation02/prepare_audio.py"
CHOICES_DIR = f"{P}/logarithm-stage09-production-choices01"
ANSWERS01 = f"{CHOICES_DIR}/inputs/answers.json"
EVIDENCE = f"{CHOICES_DIR}/inputs/evidence.tsv"
CHOICES = f"{CHOICES_DIR}/inputs/choices.tsv"
CHOOSE = f"{CHOICES_DIR}/choose_takes.py"
TIMING_ROWS = f"{P}/logarithm-stage09-timing-inspection01/outputs/timing-rows.tsv"
JOINS_DIR = f"{P}/logarithm-stage09-production-join-inspection01"
JOINS = f"{JOINS_DIR}/inspect_production_joins.py"
TC_DIR = f"{P}/logarithm-stage09-timing-corrections01"
TC = f"{TC_DIR}/timing_corrections.py"
ANSWERS02 = f"{TC_DIR}/inputs/answers02.json"
EDGES_ALL = f"{TC_DIR}/outputs/edges-recordings.tsv"
REVIEW_OPENING = f"{EL}/opening-recognition01/recognition-comparison01.json"
REVIEW_REMAINING = f"{EL}/remaining-recognition01/transcript-review01.json"
SEALS = {
    JOINS_DIR: "e05ea847fe1439d9108ff6c42f4b2f6225bd253a4d604d1d6e97715d6f962bc9",
    f"{P}/logarithm-stage09-timing-inspection01": "8c18a24363d745e65be1a3ffec89b1d1bfc8d4c2a77185c85b0a2fcd805490bf",
    CHOICES_DIR: "357613e5a21a23ce525a99d46da33d74cc8578eb23003d2c9dcfe2f79092f8cf",
    TC_DIR: "26279b53a90d46988422b7d5f0b489b8d3481b9bdabc104281284c9062a2336d",
}
PINS = {
    HELPER: "54df274a468be6122251efa7e4ccd7e11477fc8a93522e66de484ffb9be4eb0d",
    BINDING: "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803",
    SELECTION: "a5c4dfa119806d8ad46a1539c40ffd4de372dcd3541b5b62d6b673eead53a4bc",
    DECODE_RECEIPT: "4e442a6c27f192a995cdb6e7e2b5a8fb46ac676b6fadc2e09ca32cce06a51cc2",
    ANSWERS01: "8f9a0cd0fe196a0a0586b79b9f1bd68b4f1c9593d32f20ef03e4d13b372c269d",
    ANSWERS02: "075c83bd99081ef2e7b749f17c71dda50ade228f35d647e80ecdc99ae67abdca",
    REVIEW_OPENING: "6e894b9aacad3e2ea26ae17f219bbcb6b505709197126586177a329a5be45d78",
    REVIEW_REMAINING: "bf3727e27c28967c6442d3143470c21779f8951506f32e363ce902f85275c8ef",
    f"{EL}/ui-takes01/01_2-request01.json": "185088fd2df8609774d2b7c26fba897fa2f2804ad2a28a4569b68fa7d084a392",
    f"{EL}/ui-takes01/07_6-request01.json": "7170d9474bc211204fe00955e71931f1f3e057e6983e3e5f64ee40d1d9143047",
    f"{EL}/ui-takes01/11_5-request01.json": "b0904ec91b8bec0f46ea5b32e26505b2df506cb4dabfd7b36726e48df6d8737e",
    f"{EL}/ui-takes01/12_2-request01.json": "7f4c81b4bbfbec83284ecd8b0af602bcbe58540f2e0217062a72b7ff8551d0be",
    f"{EL}/ui-takes01/01_2-downloads01.json": "71b34501ca49d31928be6696c2b99dd261b8c4f8c0b2593f322108212dcd028b",
    f"{EL}/ui-takes01/07_6-downloads01.json": "4472b45b1d08fbb02eb9385ead440fa094f94020201d767887303608072f2549",
    f"{EL}/ui-takes01/11_5-downloads01.json": "a7a64cf2b6c94127122c2c140b3f045111b9344d9f92b3e6b347c20d30597c85",
    f"{EL}/ui-takes01/12_2-downloads01.json": "970461ad05ed3ec5c20d4e34803ad60de811fc1cd395f8b1406060995fe20fe8",
}
CUES = ["01_2", "07_6", "11_5", "12_2"]
CHECK_BY = "BronzeJaguar (check input for the helper's load_selection and check_answers only; root selects)"
TAIL_SRC, TAIL_DEC = 256, 64  # source samples (48 kHz) and decoded samples (24 kHz) read for the end-step estimate
PAUSE_MIN = 10  # 10 ms frames: internal quiet runs of at least 0.10 s are listed
CADENCE_S = 0.10  # a pause that differs by this much between the takes is listed for hearing


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def need(ok, msg):
    if not ok:
        raise SystemExit(f"REFUSED: {msg}")


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def read_tail(path, n):
    with wave.open(path, "rb") as w:
        need((w.getnchannels(), w.getsampwidth()) == (1, 2), f"{path}: not mono PCM16")
        total = w.getnframes()
        w.setpos(max(0, total - n))
        x = array.array("h", w.readframes(n))
    need(sys.byteorder == "little", "little-endian host required")
    return x, total


def levels(x, hop):
    """LavenderPike's levels() (production-join-inspection01) for any rate: 10 ms RMS frames, dB rounded to 0.01."""
    out = []
    for k in range(len(x) // hop):
        seg = x[k * hop:(k + 1) * hop]
        rms = math.sqrt(sum(v * v for v in seg) / hop)
        out.append(round(20 * math.log10(rms / 32768), 2) if rms > 0 else -120.0)
    return out


def tsv(rows, cols):
    buf = io.StringIO()
    w = csv.writer(buf, delimiter="\t", lineterminator="\n")
    w.writerow(cols)
    for r in rows:
        w.writerow([json.dumps(r.get(c)) if isinstance(r.get(c), (list, dict, bool)) or r.get(c) is None else r.get(c) for c in cols])
    return buf.getvalue()


def read_tsv(path):
    return list(csv.DictReader(open(path, encoding="utf8"), delimiter="\t"))


def fmn_last_weights():
    """The fmn resampler's weights (sound.rs, mirrored as timing-corrections01's fmn_resample_at) for the last 48 kHz
    output frame of a 24 kHz input: frac 0.5 and only taps at or before the last input sample."""
    ws = []
    for delta in range(-31, 1):
        dist = delta - 0.5
        ph = math.pi * dist / 32
        win = 0.42 + 0.5 * math.cos(ph) + 0.08 * math.cos(2 * ph)
        ws.append(math.sin(math.pi * dist) / (math.pi * dist) * win)
    return ws  # ws[j] multiplies x[n - 32 + j]


def main(argv):
    if len(argv) != 4 or argv[2] != "--fmn-sound":
        raise SystemExit(__doc__)
    out, sound_rs = Path(argv[1]), Path(argv[3])
    need(not out.exists(), f"{out} exists")
    need(Path(".git").exists() and Path(P).is_dir(), "run from the repository root")
    # ---------------------------------------------------------------- pins
    for folder, want in SEALS.items():
        need(sha(f"{folder}/SHA256SUMS") == want, f"{folder} seal changed")
        for line in open(f"{folder}/SHA256SUMS", encoding="utf8").read().splitlines():
            h, rel = line.split("  ", 1)
            need(sha(f"{folder}/{rel}") == h, f"{folder}/{rel} changed")
    for path, want in PINS.items():
        need(sha(path) == want, f"{path} changed (sha256 {sha(path)})")
    tc = load_module("tc", TC)
    text_rs = sound_rs.read_text(encoding="utf8")
    need(sha(sound_rs) == tc.FMN_SOUND_SHA and all(t in text_rs for t in tc.FMN_NEEDLES), "SOUND_RS is not fmn-output sound.rs at a399d626")
    need(not any(w in text_rs.lower() for w in ("fade", "envelope", "taper", "declick")), "sound.rs names an edge treatment")
    pa, ct, lj = load_module("pa", HELPER), load_module("ct", CHOOSE), load_module("lj", JOINS)
    b = pa.load_binding(BINDING)
    files, questions, recordings, inputs = pa.build_report(b)
    script = inputs["script"]
    sel01, chosen = pa.load_selection(b, SELECTION, inputs, None)
    by_file = {(e["cue"], e["file"]): e for e in recordings}
    recog = pa.recognition_for(inputs)
    decode = {r["cue"]: r for r in json.loads(Path(DECODE_RECEIPT).read_text())["records"]}
    timing_rows = {r["id"]: r for r in read_tsv(TIMING_ROWS)}
    out.mkdir(parents=True)
    lines = []

    def say(s):
        lines.append(s)
        print(s, flush=True)

    say(f"pins: {len(SEALS)} seals verified file by file; {len(PINS)} inputs at their hashes; fmn sound.rs {tc.FMN_SOUND_SHA[:8]} "
        f"with timing-corrections01's {len(tc.FMN_NEEDLES)} mirrored lines, no fade/envelope/taper/declick")
    pair = {}
    for cue in CUES:
        sel_rel = chosen[cue]["rel"]
        others = sorted(r for r, t in inputs["takes"].items() if t["cue"] == cue and r != sel_rel)
        need(len(others) == 1, f"{cue}: {len(others) + 1} retained recordings of the current text")
        pair[cue] = (sel_rel, others[0])

    # ---------------------------------------------------------------- A. words
    rev_open = json.loads(Path(REVIEW_OPENING).read_text())
    rev_rem = json.loads(Path(REVIEW_REMAINING).read_text())
    review_logs = {"opening": rev_open["log_sha256"], "remaining": rev_rem["log_sha256"]}
    takes = {}
    for cue in CUES:
        text = script[cue]
        req = json.loads(Path(f"{EL}/ui-takes01/{cue}-request01.json").read_text())
        dls = json.loads(Path(f"{EL}/ui-takes01/{cue}-downloads01.json").read_text())
        need(req["cue"] == cue and req["text"] == text and req["text_sha256"] == hashlib.sha256(text.encode()).hexdigest(),
             f"{cue}: the request is not the script's text")
        listed = sorted(os.path.basename(d["retained"]) for d in dls["downloads"])
        need(listed == sorted(os.path.basename(r) for r in pair[cue]), f"{cue}: the downloads receipt lists {listed}")
        counts = {}
        for rel in pair[cue]:
            name = os.path.basename(rel)
            t, e, r = inputs["takes"][rel], by_file[(cue, name)], recog[rel]
            need(r["via"] is None, f"{rel}: recognized through a PCM twin")
            run = r["run"]["name"]
            need(review_logs.get(run) == r["run"]["log_sha256"], f"{rel}: root's review is not of the {run} run's log")
            if run == "opening":
                rows = [x for x in rev_open["records"] if x["cue"] == cue and x["take"] == name]
                transcript, rdiffs, listened = (rows[0]["recognition"], rows[0]["differences"], None) if len(rows) == 1 else (None, None, None)
            else:
                rows = [x for x in rev_rem["rows"] if x["cue"] == cue and x["input"].endswith(f"/ui-takes01/{name}")]
                transcript, rdiffs, listened = (rows[0]["transcript"], rows[0]["differences"], rows[0]["listened"]) if len(rows) == 1 else (None, None, None)
            need(len(rows) == 1, f"{rel}: root's review has {len(rows)} rows")
            obs = pa.observed_words(r["result"])
            heard = [o["w"] for o in obs]
            review_words = [w for w, _ in pa.tokens(transcript)]
            diffs = pa.differences(text, r["result"])
            canon = pa.tokens(text)
            al = pa.align(text, r["result"])
            last_i = len(canon) - 1
            last_o = obs[al[2][last_i]] if last_i in al[2] else None
            counts[rel] = collections.Counter((d["script"], d["heard"]) for d in diffs)
            takes[rel] = {
                "cue": cue, "file": name, "path": rel, "selected": rel == chosen[cue]["rel"], "sha256": t["sha256"],
                "frames_48k": t["frames"], "seconds": round(t["frames"] / t["rate"], 3),
                "request": {"utc_downloads": dls["utc"], "voice": req["voice"], "model": req["model"], "stability": req["stability"],
                            "similarity": req["similarity"], "format": req["format"], "listening": dls.get("listening")},
                "recognizer": {"run": run, "model": r["run"]["model"], "log_sha256": r["run"]["log_sha256"], "words": len(heard),
                               "differences": diffs, "last_script_word": canon[-1][0],
                               "last_word_recognized": last_o is not None,
                               "last_word_end_s": round(last_o["end"], 3) if last_o else None},
                "root_review": {"file": REVIEW_OPENING if run == "opening" else REVIEW_REMAINING, "words_equal_record": review_words == heard,
                                "differences": rdiffs, "listened": listened, "transcript": transcript},
            }
        a, z = pair[cue]
        for x, y in ((a, z), (z, a)):
            extra = counts[x] - counts[y]
            takes[x]["recognizer"]["unshared"] = sorted(f"{s}->{h}" for (s, h) in extra.elements())
            takes[x]["recognizer"]["unshared_not_notation"] = sorted(f"{s}->{h}" for (s, h) in extra.elements() if (s, h) not in ct.NOTATION)
            takes[x]["recognizer"]["shared"] = sorted(f"{s}->{h}" for (s, h) in (counts[x] & counts[y]).elements())
        ta, tz = takes[a]["root_review"]["transcript"], takes[z]["root_review"]["transcript"]
        punct = [p for p in _punct_diff(ta, tz)]
        takes[a]["root_review"]["punctuation_vs_other"] = punct
        takes[z]["root_review"]["punctuation_vs_other"] = [{"word_after": p["word_after"], "this": p["other"], "other": p["this"]} for p in punct]
    for rel, t in takes.items():
        say(f"A {t['cue']} {t['file']}{' (selected)' if t['selected'] else ''}: request = script text; run {t['recognizer']['run']} "
            f"({t['recognizer']['model']}, {t['recognizer']['words']} words); root's review of the same log agrees word for word: "
            f"{t['root_review']['words_equal_record']}; differences {[d['script'] + '->' + d['heard'] for d in t['recognizer']['differences']] or 'none'}; "
            f"unshared {t['recognizer']['unshared'] or 'none'}; last word '{t['recognizer']['last_script_word']}' recognized "
            f"{t['recognizer']['last_word_recognized']} (ends {t['recognizer']['last_word_end_s']} s of {t['seconds']} s)")

    # ---------------------------------------------------------------- B. timing coverage under a check selection
    swap = {cue: pair[cue][1] for cue in CUES}
    check = dict(sel01, by=CHECK_BY,
                 basis=("selection01 with the other retained take of 01_2, 07_6, 11_5 and 12_2 in place of the selected one, so that "
                        "the installed helper's load_selection, check_answers and resolved_points can be run on those takes. A check "
                        "input written by edge_take_review.py; it selects nothing."),
                 takes={c: swap.get(c, p) for c, p in sel01["takes"].items()})
    check_path = out / "check-selection-swap4.json"
    check_path.write_text(json.dumps(check, indent=1) + "\n")
    _c, chosen_swap = pa.load_selection(b, str(check_path), inputs, None)
    need([c for c in chosen_swap if chosen_swap[c]["rel"] != chosen[c]["rel"]] == CUES, "the check selection swaps other cues")
    applied, refusals = {}, {}
    for label, path in (("answers01", ANSWERS01), ("answers02", ANSWERS02)):
        for which, ch in (("selection01", chosen), ("swap4", chosen_swap)):
            try:
                applied[(label, which)] = pa.check_answers(b, path, questions, recordings, ch)
            except pa.Refused as ex:
                refusals[f"{label}/{which}"] = str(ex)
    blocking = {rel: [q["id"] for q in questions if q["blocking"] and by_file[(takes[rel]["cue"], takes[rel]["file"])]["label"] in q["files"]]
                for rel in takes}
    points = []
    db48 = {}
    for rel in takes:
        x, rate = tc.read_pcm(rel)
        need(rate == 48000, f"{rel}: {rate} Hz")
        db48[rel] = levels(x, 480)
    for cue in CUES:
        text = script[cue]
        for rel in pair[cue]:
            t, name = takes[rel], os.path.basename(rel)
            entry = by_file[(cue, name)]
            frames24 = pa.expected_frames(inputs["takes"][rel])[0]
            if t["selected"]:
                need(decode[cue]["source"] == rel and decode[cue]["samples"] == frames24, f"{cue}: decoded01 length {decode[cue]['samples']} != {frames24}")
            t["frames_24k"] = frames24
            db = db48[rel]
            runs = lj.quiet_runs(db)
            which = "selection01" if t["selected"] else "swap4"
            for label in ("answers01", "answers02"):
                ap = applied.get((label, which))
                if ap is None:
                    t.setdefault("unresolved", []).append(f"{label}: check_answers refused")
                    continue
                try:
                    starts, ends, marks = pa.resolved_points(entry, ap, frames24, text)
                except pa.Refused as ex:
                    t.setdefault("unresolved", []).append(f"{label}: {ex}")
                    continue
                used = sorted(q for q in ap if q.endswith(":" + entry["label"]) and f":{cue}:" in q)
                t[f"answers_used_{label}"] = used
                for n, (s, why) in enumerate(starts, 1):
                    k = s // 240
                    pl = lj.place(db, runs, k, 100, True)
                    pid = f"onset:{cue}:s{n}:{entry['label']}"
                    tr = timing_rows.get(pid, {})
                    points.append({"cue": cue, "file": name, "selected": t["selected"], "answers": label, "point": f"s{n}", "id": pid,
                                   "sample_24k": s, "seconds": round(s / 24000, 3), "basis": "answer" if why.startswith("sample given") else "recognizer onset",
                                   "timing_inspection_class": tr.get("class"), "recognizer_onset_s": tr.get("recognizer_onset_s"),
                                   "leave_s": tr.get("leave_s"), "body_s": tr.get("body_s"), "pause_s": tr.get("pause_s"),
                                   "level_db": pl.get("level_db"), "position": pl.get("position"), "after_body_s": pl.get("after_body_s"),
                                   "into_pause_s": pl.get("into_pause_s"), "before_leave_s": pl.get("before_leave_s"), "join_pause": pl.get("pause"),
                                   "flags": pl.get("flags")})
                for m in marks:
                    k = round(m["seconds"] * 100)
                    pl = lj.place(db, runs, k, 50, False)
                    pid = f"anchor:{cue}:b{m['byte']}:{entry['label']}"
                    tr = timing_rows.get(pid, {})
                    points.append({"cue": cue, "file": name, "selected": t["selected"], "answers": label, "point": f"b{m['byte']}", "id": pid,
                                   "sample_24k": round(m["seconds"] * 24000), "seconds": round(m["seconds"], 3),
                                   "basis": "answer" if m["basis"].startswith("sample given") else "recognizer onset",
                                   "timing_inspection_class": tr.get("class") or tr.get("status"), "recognizer_onset_s": tr.get("recognizer_onset_s"),
                                   "leave_s": tr.get("leave_s"), "body_s": tr.get("body_s"), "pause_s": tr.get("pause_s"),
                                   "level_db": pl.get("level_db"), "position": pl.get("position"), "after_body_s": pl.get("after_body_s"),
                                   "into_pause_s": pl.get("into_pause_s"), "before_leave_s": pl.get("before_leave_s"), "join_pause": pl.get("pause"),
                                   "dip_db": pl.get("dip_db"),
                                   "dip_offset_s": pl.get("dip_offset_s"), "flags": pl.get("flags")})
                if label == "answers02":
                    t["sentence_ends_24k_answers02"] = ends
                    t["marks_answers02"] = marks
    # control: LavenderPike's rows for the selected takes (24 kHz production files, answers01) against this 48 kHz replay
    lav = {}
    for r in read_tsv(f"{JOINS_DIR}/outputs/joins.tsv"):
        lav[(r["cue"], f"s{r['sentence']}")] = r
    for r in read_tsv(f"{JOINS_DIR}/outputs/marks.tsv"):
        lav[(r["cue"], f"b{r['byte']}")] = r
    control = []
    for p in points:
        if p["selected"] and p["answers"] == "answers01" and (p["cue"], p["point"]) in lav:
            r = lav[(p["cue"], p["point"])]
            theirs = {"sample": int(r["sample_24k"]) if "sample_24k" in r else round(float(r["seconds"]) * 24000),
                      "flags": json.loads(r["flags"]), "leave_s": (json.loads(r["pause"]) or {}).get("leave_s") if r["pause"] else None}
            mine = {"sample": p["sample_24k"], "flags": p["flags"], "leave_s": (p["join_pause"] or {}).get("leave_s")}
            control.append({"cue": p["cue"], "point": p["point"], "lavender_24k": theirs, "replay_48k": mine, "equal": theirs == mine})
    n_eq = sum(c["equal"] for c in control)
    say(f"B check selection {sha(check_path)[:8]} (selection01 with the four alternatives): the helper's load_selection accepts it; "
        f"check_answers refusals: {refusals or 'none'}; blocking questions naming the eight takes: "
        f"{ {takes[r]['file']: q for r, q in blocking.items() if q} or 'none'}")
    say(f"B control: LavenderPike's place() on 10 ms frames of the 48 kHz source repeats her 24 kHz rows for the selected takes in "
        f"{n_eq} of {len(control)} points (sample, flags and leave)" + ("" if n_eq == len(control) else "; differing: " + "; ".join(
            f"{c['cue']} {c['point']} hers {c['lavender_24k']} replay {c['replay_48k']}" for c in control if not c["equal"])))
    for cue in CUES:
        for rel in pair[cue]:
            t = takes[rel]
            rows = [p for p in points if p["file"] == t["file"] and p["cue"] == cue and p["answers"] == "answers02"]
            t["join_flags_answers02"] = {p["point"]: p["flags"] for p in rows if p["flags"]}
            say(f"B {cue} {t['file']}{' (selected)' if t['selected'] else ''} under answers02: "
                + ("; ".join(f"{p['point']} {p['seconds']} s ({p['basis']}{', ' + p['timing_inspection_class'] if p['timing_inspection_class'] else ''}"
                             f"{', ' + p['position'] if p['position'] else ''}{', flags ' + ','.join(p['flags']) if p['flags'] else ''})" for p in rows)
                   or "one sentence, no marks")
                + (f"; unresolved {t['unresolved']}" if t.get("unresolved") else "")
                + f"; answers used {len(t.get('answers_used_answers02', []))}")

    # ---------------------------------------------------------------- C. pauses
    pause_rows = []
    for cue in CUES:
        text = script[cue]
        per = {}
        for rel in pair[cue]:
            t, db = takes[rel], db48[rel]
            al = pa.align(text, recog[rel]["result"])
            canon, obs, match = al[0], al[1], al[2]
            inv = {j: i for i, j in {**al[4], **match}.items()}  # agreeing words, and one-for-one replaced words ('lapse' for 'laps')
            spoken = [k for k, v in enumerate(db) if v >= lj.BODY_DB]  # internal: between the first and last frame at speech level
            runs, k = [], spoken[0]
            while k <= spoken[-1]:
                if db[k] <= lj.QUIET_DB:
                    j = k
                    while db[j] <= lj.QUIET_DB:
                        j += 1
                    if j - k >= PAUSE_MIN:
                        runs.append((k, j))
                    k = j
                else:
                    k += 1
            ends = t.get("sentence_ends_24k_answers02", [])[:-1]
            per[rel] = {}
            for a, z in runs:
                before = [j for j, o in enumerate(obs) if o["start"] < a / 100 - 0.02]
                jb = before[-1] if before else None
                i = inv.get(jb) if jb is not None else None
                between = (text[canon[i][1]:canon[i + 1][1]] if i is not None and i + 1 < len(canon) else None)
                key = i if i is not None else f"unmatched@{a / 100:.2f}"
                at = [f"s{n}" for n, s in enumerate(ends, 1) if a - 10 <= s // 240 <= z + 10]
                per[rel][key] = per[rel].get(key, 0) + (z - a)
                pause_rows.append({"cue": cue, "file": t["file"], "selected": t["selected"], "start_s": a / 100, "end_s": z / 100,
                                   "pause_s": round((z - a) / 100, 2), "after_heard_word": obs[jb]["w"] if jb is not None else None,
                                   "script_between": between, "boundary": ",".join(at) or None})
        a_rel, z_rel = pair[cue]
        keys = sorted(set(per[a_rel]) | set(per[z_rel]), key=lambda k: (isinstance(k, str), k if isinstance(k, int) else 0, str(k)))
        cmp = []
        for key in keys:
            u, v = per[a_rel].get(key, 0) / 100, per[z_rel].get(key, 0) / 100
            cn = canon_of(pa, text)
            between = (text[cn[key][1]:cn[key + 1][1]] if key + 1 < len(cn) else text[cn[key][1]:]) if isinstance(key, int) else key
            cmp.append({"script_between": between.strip(), "char": cn[key][1] if isinstance(key, int) else None,
                        "selected_s": round(u, 2), "alternative_s": round(v, 2), "delta_s": round(v - u, 2)})
        takes[a_rel]["pause_compare"] = cmp
        takes[a_rel]["pause_total_s"] = round(sum(per[a_rel].values()) / 100, 2)
        takes[z_rel]["pause_total_s"] = round(sum(per[z_rel].values()) / 100, 2)
        hear = [c for c in cmp if abs(c["delta_s"]) >= CADENCE_S - 1e-9]
        takes[z_rel]["cadence_items"] = hear
        say(f"C {cue}: internal pauses >= 0.10 s total {takes[a_rel]['pause_total_s']} s (selected) / {takes[z_rel]['pause_total_s']} s "
            f"(alternative); differing by >= {CADENCE_S} s: " + ("; ".join(f"'{c['script_between']}' (char {c['char']}) {c['selected_s']} -> {c['alternative_s']} s" for c in hear) or "none"))

    # ---------------------------------------------------------------- D. edges and the end step
    ws = fmn_last_weights()
    gain = sum(abs(w) for w in ws) / abs(sum(ws))
    calib = []
    for cue in script:
        src = chosen[cue]["rel"]
        d, nd = read_tail(decode[cue]["wav"], TAIL_DEC)
        x, nx = read_tail(src, TAIL_SRC)
        need(nd == decode[cue]["samples"], f"{cue}: decoded length")
        calib.append({"cue": cue, "source": os.path.basename(src), "source_tail_max_abs": max(abs(v) for v in x),
                      "decoded_tail_max_abs": max(abs(v) for v in d),
                      "ratio": round(max(abs(v) for v in d) / max(1, max(abs(v) for v in x)), 4)})
    ratio = max(c["ratio"] for c in calib)
    worst = max(calib, key=lambda c: c["ratio"])
    for cue in CUES:
        for rel in pair[cue]:
            t = takes[rel]
            x, rate = tc.read_pcm(rel)
            em = tc.edge_measures(x, rate)
            n = len(x)
            tail_max = max(abs(v) for v in x[-TAIL_SRC:])
            est = gain * ratio * max(1, tail_max)
            db = db48[rel]
            body_last = max(k for k, v in enumerate(db) if v >= lj.BODY_DB)
            quiet_before_last = 0
            while db[-2 - quiet_before_last] <= lj.QUIET_DB:
                quiet_before_last += 1
            t["edges"] = {**{k: em[k] for k in ("lead_40_s", "tail_40_s", "lead_60_s", "tail_60_s", "first_10ms_db", "last_10ms_db",
                                                "first_1ms_db", "last_1ms_db", "first_samples", "last_samples")},
                          "first_10ms_db_raw": tc.db_raw(x, 0, 480), "last_10ms_db_raw": tc.db_raw(x, n - 480, 480),
                          # how the take ends: speech level last reached, the floor after it, and LavenderPike's rise test
                          # (cue-edges: last frame above -60 dB and more than 15 dB above the frame before it)
                          "speech_level_ends_s": round((body_last + 1) / 100, 2), "floor_after_speech_db": min(db[body_last:-1]),
                          "quiet_frames_before_last_frame": quiet_before_last, "last_two_frames_db": db[-2:],
                          "rise_in_last_10ms": db[-1] > lj.QUIET_DB and db[-1] > db[-2] + 15,
                          "last_10ms_1ms_db": [tc.db(x, n - (10 - i) * 48, 48) for i in range(10)],
                          "source_last_256_max_abs": tail_max, "bus_end_estimate_dbfs": tc.dbfs(est), "bus_end_estimate_abs": round(est, 1)}
            if t["selected"]:
                d, rate24 = tc.read_pcm(decode[cue]["wav"])
                need(sha(decode[cue]["wav"]) == decode[cue]["wav_sha256"], f"{cue}: decoded file changed")
                m = len(d)
                bus = tc.fmn_resample_at(d, rate24, 48000, [2 * m - 3, 2 * m - 2, 2 * m - 1])
                mine = sum(w * d[m - 32 + j] / 32768 for j, w in enumerate(ws)) / sum(ws)
                need(abs(mine - bus[2 * m - 1]) < 1e-12, f"{cue}: the last-frame weights differ from the mirror")
                t["edges"]["bus_last_48k"] = [round(bus[k] * 32768, 1) for k in (2 * m - 3, 2 * m - 2, 2 * m - 1)]
                t["edges"]["bus_end_dbfs"] = tc.dbfs(bus[2 * m - 1] * 32768)
                t["edges"]["estimate_covers_actual"] = est >= abs(bus[2 * m - 1] * 32768)
    say(f"D fmn's last 48 kHz frame weighs the last 32 decoded samples with gain sum|w|/|sum w| = {gain:.3f}; root's 80 decodes: "
        f"max |last {TAIL_DEC} decoded| / max |last {TAIL_SRC} source| <= {ratio} ({worst['cue']}); estimate = gain x ratio x the source's "
        f"largest last-{TAIL_SRC} sample (at least 1)")
    for cue in CUES:
        for rel in pair[cue]:
            e = takes[rel]["edges"]
            say(f"D {cue} {takes[rel]['file']}{' (selected)' if takes[rel]['selected'] else ''}: speech level ends {e['speech_level_ends_s']} s, "
                f"floor after it {e['floor_after_speech_db']} dB, {e['quiet_frames_before_last_frame']} quiet 10 ms frames before the last, "
                f"last two frames {e['last_two_frames_db']} dB (rise in the last 10 ms: {e['rise_in_last_10ms']}; 1 ms {e['last_10ms_1ms_db']}); "
                f"-60 tail {e['tail_60_s']} s, last 10 ms "
                f"{e['last_10ms_db']} dB, last 1 ms {e['last_1ms_db']} dB, last samples {e['last_samples']}, largest of the last {TAIL_SRC} "
                f"{e['source_last_256_max_abs']}; bus end estimate {e['bus_end_estimate_dbfs']} dBFS"
                + (f"; mirrored bus end {e['bus_end_dbfs']} dBFS {e['bus_last_48k']} (estimate covers it: {e['estimate_covers_actual']})"
                   if takes[rel]["selected"] else "") + f"; first 1 ms {e['first_1ms_db']} dB, first samples {e['first_samples']}")

    # ---------------------------------------------------------------- E. the rule, replayed
    ev = {os.path.basename(r["path"]): r for r in read_tsv(EVIDENCE)}
    ch = {r["cue"]: r for r in read_tsv(CHOICES)}
    rule_rows = []

    def decide(cue, flags_of):
        a, z = sorted(os.path.basename(r) for r in pair[cue])
        ra = next(r for r in pair[cue] if r.endswith(a))
        rz = next(r for r in pair[cue] if r.endswith(z))
        fa, fz = len(flags_of(a)), len(flags_of(z))
        own_a = [d for d in takes[ra]["recognizer"]["unshared_not_notation"]]
        own_z = [d for d in takes[rz]["recognizer"]["unshared_not_notation"]]
        pa_, pz = float(ev[a]["shortest_boundary_pause_s"]), float(ev[z]["shortest_boundary_pause_s"])
        if fa != fz:
            return (a if fa < fz else z), f"R2 integrity: {min(fa, fz)} flag(s) against {max(fa, fz)}"
        if len(own_a) != len(own_z):
            return (a if len(own_a) < len(own_z) else z), "R3 recognizer: fewer unshared word differences"
        if abs(pa_ - pz) >= ct.PAUSE_TIE_S:
            return (a if pa_ > pz else z), f"R4 room: shortest boundary pause {max(pa_, pz):.2f} s against {min(pa_, pz):.2f} s"
        return a, f"R5 lower download number (shortest boundary pauses {pa_:.2f} / {pz:.2f} s)"

    def sealed_flags(name):
        return [f for f in ev[name]["flags"].split(" | ") if f]

    def flags60(name):
        rel = next(r for r in takes if takes[r]["file"] == name and takes[r]["cue"] == name.split("-")[0])
        e = takes[rel]["edges"]
        extra = ["edge60: sound above -60 dBFS in the first or last 10 ms frame"] if max(e["first_10ms_db_raw"], e["last_10ms_db_raw"]) > lj.QUIET_DB else []
        return sealed_flags(name) + extra

    for cue in CUES:
        for name in sorted(os.path.basename(r) for r in pair[cue]):
            need(ev[name]["eligible"] == "True", f"{name}: not eligible in production-choices01")
        got, rule = decide(cue, sealed_flags)
        need((got, rule) == (ch[cue]["chosen"], ch[cue]["rule"]), f"{cue}: the replay gives {got} {rule}, choices.tsv {ch[cue]['chosen']} {ch[cue]['rule']}")
        got60, rule60 = decide(cue, flags60)
        rule_rows.append({"cue": cue, "sealed_choice": got, "sealed_rule": rule, "edge60_choice": got60, "edge60_rule": rule60,
                          "flags60": {n: flags60(n) for n in sorted(os.path.basename(r) for r in pair[cue])},
                          "shortest_pause_s": {n: float(ev[n]["shortest_boundary_pause_s"]) for n in sorted(os.path.basename(r) for r in pair[cue])}})
        say(f"E {cue}: the replay of R1-R5 gives choices.tsv's {got} ({rule}); with R2's edge at -60 dBFS: {got60} ({rule60})")
    # pointer only (not audited): other cues an edge test at -60 dBFS would touch, from timing-corrections01's sealed edge rows
    rows_all = read_tsv(EDGES_ALL)
    pointer = []
    for cue in script:
        if cue in CUES:
            continue
        rs = [r for r in rows_all if r["cue"] == cue]
        flagged = {r["file"]: max(float(r["first_10ms_db"]), float(r["last_10ms_db"])) > lj.QUIET_DB for r in rs}
        sel = os.path.basename(chosen[cue]["rel"])
        if flagged.get(sel) and not all(flagged.values()):
            pointer.append({"cue": cue, "selected": sel, "where": "start" if float(next(r for r in rs if r["file"] == sel)["first_10ms_db"]) > lj.QUIET_DB else "end",
                            "sealed_rule": ch[cue]["rule"].split(" ")[0]})
    say("E pointer, not audited: outside the four cues an edge test at -60 dBFS would flag only the selected take of "
        + (", ".join(f"{p['cue']} ({p['selected']}, {p['where']}; sealed {p['sealed_rule']})" for p in pointer) or "none"))

    # ---------------------------------------------------------------- verdict per cue
    verdicts = {}
    for cue in CUES:
        s_rel, a_rel = pair[cue]
        s, a = takes[s_rel], takes[a_rel]
        rr = next(r for r in rule_rows if r["cue"] == cue)
        flags_new = {k: v for k, v in a.get("join_flags_answers02", {}).items() if v != s.get("join_flags_answers02", {}).get(k)}
        conds = {
            "selected_take_has_the_final_10ms_rise": s["edges"]["rise_in_last_10ms"],
            "alternative_has_no_final_rise": not a["edges"]["rise_in_last_10ms"],
            "alternative_ends_in_quiet": a["edges"]["last_10ms_db_raw"] <= lj.QUIET_DB and a["edges"]["bus_end_estimate_dbfs"] <= -60,
            "alternative_starts_in_quiet": a["edges"]["first_10ms_db_raw"] <= lj.QUIET_DB,
            "same_request_and_script_text": True,
            "no_unshared_recognizer_difference": not a["recognizer"]["unshared_not_notation"],
            "last_word_recognized": a["recognizer"]["last_word_recognized"],
            "helper_resolves_every_point_answers02": "unresolved" not in a and not blocking[a_rel],
            "shortest_boundary_pause_not_tight": rr["shortest_pause_s"][a["file"]] >= ct.TIGHT_S,
            "edge60_rule_picks_alternative": rr["edge60_choice"] == a["file"],
        }
        # a join flag the selected take (under answers02) does not have at the same point; a recognizer onset at most 0.05 s
        # after speech level gets the same kind of leave sample timing-corrections01 gave 07_6 download01 s5 (+0.04 s),
        # listed for root as an optional answer, not written to any answers file
        # Classes: 'late' (onset at most 0.05 s after speech level) -> optional answer; 'blip' (a sound inside the pause
        # before the body) with the boundary itself at or below -60 dB -> the boundary still lies in quiet, the sound
        # stays at the end of the previous sentence's span: a hearing item; a mark's 'no_pause' with a dip at or below
        # -60 dB within 0.03 s -> the mark sits at a word junction: a hearing item. Anything else fails.
        proposals, items, other_flags = [], [], {}
        for k, fl in flags_new.items():
            p = next(p for p in points if p["file"] == a["file"] and p["cue"] == cue and p["answers"] == "answers02" and p["point"] == k)
            if fl == ["late"] and p["basis"] == "recognizer onset" and p["after_body_s"] is not None and p["after_body_s"] <= 0.05 + 1e-9:
                proposals.append({"id": p["id"], "decision": "sample", "sample_24k": p["join_pause"]["leave_24k"],
                                  "why": (f"the recognizer's onset {p['seconds']} s is {p['after_body_s']} s after speech level "
                                          f"({p['join_pause']['body_s']} s); leave {p['join_pause']['leave_s']} s after a "
                                          f"{p['join_pause']['pause_s']} s pause; optional, not heard")})
            elif fl == ["blip"] and p["level_db"] is not None and p["level_db"] <= lj.QUIET_DB:
                items.append(f"{p['id']}: a sound inside the pause, from leave {p['join_pause']['leave_s']} s, before speech level at "
                             f"{p['join_pause']['body_s']} s; the boundary {p['seconds']} s lies after it at {p['level_db']} dB")
            elif fl == ["no_pause"] and k.startswith("b") and p["dip_db"] is not None and p["dip_db"] <= lj.QUIET_DB \
                    and abs(p["dip_offset_s"]) <= 0.03 + 1e-9:
                items.append(f"{p['id']}: the mark {p['seconds']} s has no pause of 50 ms or more around it; the lowest 10 ms frame within "
                             f"0.10 s is {p['dip_db']} dB at {p['dip_offset_s']:+} s (a word junction)")
            else:
                other_flags[k] = fl
        conds["no_join_flag_beyond_the_selected_take_unclassified"] = not other_flags
        ok = all(conds.values())
        verdicts[cue] = {"selected": s["file"], "alternative": a["file"], "conditions": conds,
                         "recommendation": ("switch to the alternative" + (f" ({len(proposals)} optional answer listed)" if proposals else ""))
                         if ok else "keep the selected take",
                         "failed": [k for k, v in conds.items() if not v], "new_join_flags": flags_new, "optional_answers": proposals,
                         "seconds_delta": round(a["seconds"] - s["seconds"], 3),
                         "hearing_items": {"pauses": a.get("cadence_items", []), "flags": items,
                                           "punctuation_in_root_review": a["root_review"]["punctuation_vs_other"]}}
        say(f"VERDICT {cue}: {verdicts[cue]['recommendation']} ({a['file']} for {s['file']}; {verdicts[cue]['seconds_delta']:+} s)"
            + (f"; failed {verdicts[cue]['failed']}" if verdicts[cue]["failed"] else "; every condition holds")
            + (f"; new join flags {flags_new}" if flags_new else "")
            + "".join(f"; optional answer {q['id']} sample {q['sample_24k']} ({q['why']})" for q in proposals)
            + "".join(f"; hearing item {i}" for i in items))

    # ---------------------------------------------------------------- write
    tcols = ["cue", "file", "selected", "seconds", "frames_48k", "frames_24k"]
    trows = []
    for rel, t in takes.items():
        e = t["edges"]
        trows.append({**{k: t.get(k) for k in tcols}, "run": t["recognizer"]["run"], "review_words_equal": t["root_review"]["words_equal_record"],
                      "differences": [d["script"] + "->" + d["heard"] for d in t["recognizer"]["differences"]],
                      "unshared": t["recognizer"]["unshared"], "last_word_end_s": t["recognizer"]["last_word_end_s"],
                      "speech_level_ends_s": e["speech_level_ends_s"], "quiet_frames_before_last_frame": e["quiet_frames_before_last_frame"],
                      "last_two_frames_db": e["last_two_frames_db"], "rise_in_last_10ms": e["rise_in_last_10ms"],
                      "lead_40_s": e["lead_40_s"], "tail_40_s": e["tail_40_s"], "lead_60_s": e["lead_60_s"], "tail_60_s": e["tail_60_s"],
                      "first_10ms_db": e["first_10ms_db"], "last_10ms_db": e["last_10ms_db"], "last_1ms_db": e["last_1ms_db"],
                      "last_samples": e["last_samples"], "source_last_256_max_abs": e["source_last_256_max_abs"],
                      "bus_end_dbfs": e.get("bus_end_dbfs"), "bus_end_estimate_dbfs": e["bus_end_estimate_dbfs"],
                      "pause_total_s": t.get("pause_total_s"), "answers_used_answers02": t.get("answers_used_answers02"),
                      "join_flags_answers02": t.get("join_flags_answers02")})
    (out / "takes.tsv").write_text(tsv(trows, list(trows[0])))
    pcols = ["cue", "file", "selected", "answers", "point", "sample_24k", "seconds", "basis", "timing_inspection_class", "recognizer_onset_s",
             "leave_s", "body_s", "pause_s", "level_db", "position", "after_body_s", "into_pause_s", "before_leave_s", "dip_db", "dip_offset_s",
             "flags"]
    (out / "points.tsv").write_text(tsv(points, pcols))
    (out / "pauses.tsv").write_text(tsv(pause_rows, list(pause_rows[0])))
    (out / "rules.tsv").write_text(tsv(rule_rows, list(rule_rows[0])))
    (out / "edge-calibration.tsv").write_text(tsv(calib, list(calib[0])))
    review = {"claim": "Comparison of complete retained takes on recognizer, helper, energy and rule evidence. Nothing was heard, "
                       "selected, answered, decoded or changed; root selects and prepares.",
              "pins": {**{k + "/SHA256SUMS": v for k, v in SEALS.items()}, **PINS, "fmn sound.rs": tc.FMN_SOUND_SHA},
              "check_selection_sha256": sha(check_path), "check_answers_refusals": refusals,
              "answers_applied": {f"{k[0]}/{k[1]}": len(v) for k, v in sorted(applied.items())},
              "lavender_control": control, "end_step_estimate": {"fmn_last_frame_gain": round(gain, 6), "decode_ratio": ratio,
                                                                 "decode_ratio_cue": worst["cue"], "tail_source": TAIL_SRC, "tail_decoded": TAIL_DEC},
              "rules": rule_rows, "pointer_not_audited": pointer, "takes": takes, "verdicts": verdicts}
    (out / "review.json").write_text(json.dumps(review, indent=1, ensure_ascii=False) + "\n")
    (out / "stdout.txt").write_text("\n".join(lines) + "\n")
    print("EDGE TAKE REVIEW: DONE")
    return 0


def canon_of(pa, text, _cache={}):
    if text not in _cache:
        _cache[text] = pa.tokens(text)
    return _cache[text]


def _punct_diff(ta, tz):
    """Punctuation after each word that differs between two transcripts of the same words (diagnostic only)."""
    import re
    def marks(t):
        out = []
        for m in re.finditer(r"([A-Za-z0-9'’\-]+)([^A-Za-z0-9'’\s\-]*)", t):
            out.append((m.group(1).lower(), m.group(2)))
        return out
    a, z = marks(ta), marks(tz)
    if [w for w, _ in a] != [w for w, _ in z]:
        return [{"word_after": None, "this": "word lists differ", "other": "word lists differ"}]
    return [{"word_after": a[k][0], "this": a[k][1], "other": z[k][1]} for k in range(len(a)) if a[k][1] != z[k][1]]


if __name__ == "__main__":
    sys.exit(main(sys.argv))
