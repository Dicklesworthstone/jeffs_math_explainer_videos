#!/usr/bin/env python3
"""Independent review of WhiteSwan's complex-frederick-current-timing-review01 (BlueSalmon Mail 4474, bead 87do.39).

    python3 -I -B P/complex-frederick-current-timing-independent01/checks/independent_review.py SCRATCH
        run from the repository root; SCRATCH must be new or empty and outside the repository

Read only: WhiteSwan's sealed bundle (its seal is checked first), root's recognition lane, the guard jobs' fw records,
the targeted-small01 job and the takes' 48 kHz PCM. The 36 lane files the bridge reads (plans and ingests 01-09) are
checked against the live lane and copied into SCRATCH/lane; the repaired bridge (47069583) is imported and its report
builder runs in-process on that copy, as root's `report` would. Nothing is decoded, recognized, synthesized, launched
or heard. Writes only SCRATCH/lane and SCRATCH/review.json.

Method, chosen to differ from WhiteSwan's (5 ms frames, -70 dBFS pause, the LAST pause, -30 dBFS body):
- level: 10 ms RMS at a 1 ms hop on the take's own samples; frame k covers [k ms, k ms + 10 ms);
- pause: the LONGEST run of frames at or below -60 dBFS lasting at least 0.05 s that starts at or after the previous
  recognized word's start, starts by the recognizer start and ends by the point's own segment end;
- leave: where that run ends (sound starts within 1 ms after it); body: the end of the first 10 ms window at or above
  -40 dBFS after leave;
- class: in_pause (start <= leave + 0.01 s), at_onset (start <= body + 0.03 s), inside_word (later), no_pause.
Every sentence onset and recognized mark of all 94 takes is measured, not only the flagged ones. Levels are not
listening: they cannot tell a breath from a word or say which word was spoken.
"""
import array
import hashlib
import importlib.util
import itertools
import json
import math
import shutil
import sys
import wave
from pathlib import Path

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
WHITE = REPO / P / "complex-frederick-current-timing-review01"
WHITE_SUMS = "42f34a752736025b6a9b0634e8773b2e2856fa5d7d60cb57f670ea25d984c4c1"  # Mail 4466
BRIDGE_SHA = "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39"
HOP, WIN = 48, 480  # 1 ms hop, 10 ms window at 48 kHz
QUIET, LOUD, MIN_PAUSE = -60.0, -40.0, 0.05
THRESHOLDS = (-50.0, -55.0, -60.0, -65.0, -70.0, -75.0)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def fail(message):
    print(f"FAIL {message}")
    sys.exit(1)


def envelope(path):
    with wave.open(str(path)) as w:
        if (w.getnchannels(), w.getsampwidth(), w.getframerate()) != (1, 2, 48000):
            fail(f"{path.name} is not 48 kHz mono 16-bit")
        x = array.array("h", w.readframes(w.getnframes()))
    if sys.byteorder != "little":
        x.byteswap()
    cs = list(itertools.accumulate((v * v for v in x), initial=0))
    db = []
    for i in range(0, len(x) - WIN + 1, HOP):
        e = (cs[i + WIN] - cs[i]) / WIN
        db.append(20 * math.log10(math.sqrt(e) / 32768) if e > 0 else -120.0)
    return x, db


def runs(db, threshold):
    """Quiet intervals [start_s, end_s): frames a..b at or below the threshold cover [a ms, b ms + 10 ms)."""
    out, a = [], None
    for k, v in enumerate(db + [0.0]):
        if v <= threshold and a is None:
            a = k
        elif v > threshold and a is not None:
            out.append(((a) / 1000, (k - 1 + 10) / 1000))
            a = None
    return out


def pause(db, threshold, p0, start, end, rule="longest", min_len=MIN_PAUSE):
    ok = [r for r in runs(db, threshold) if r[1] - r[0] >= min_len - 1e-9 and r[0] >= p0 - 1e-9
          and r[0] <= start + 1e-9 and r[1] <= end + 1e-9]
    if not ok:
        return None, ok
    return (max(ok, key=lambda r: (round(r[1] - r[0], 6), r[0])) if rule == "longest" else max(ok, key=lambda r: r[0])), ok


def body_after(db, leave):
    k0 = max(0, int(round(leave * 1000)) - 10)
    return next(((k + 10) / 1000 for k in range(k0, len(db)) if db[k] >= LOUD), None)


def level_db(x, a, b, peak=False):
    seg = x[max(0, a):max(0, b)]
    if not seg:
        return None
    v = max(abs(s) for s in seg) if peak else math.sqrt(sum(s * s for s in seg) / len(seg))
    return round(20 * math.log10(v / 32768), 1) if v > 0 else -120.0


def clean(text):
    return str(text).replace(str(SCR.resolve()), "SCRATCH").replace(str(SCR), "SCRATCH").replace(str(REPO), "REPO")


def r3(v):
    return None if v is None else round(v, 3)


# ----------------------------------------------------------------------------- setup and pins

if len(sys.argv) != 2:
    sys.exit(__doc__)
SCR = Path(sys.argv[1])
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
if not (REPO / "tools/coherent_narration.py").is_file():
    sys.exit("run from the repository root")
SCR.mkdir(parents=True, exist_ok=True)

if sha(WHITE / "SHA256SUMS") != WHITE_SUMS:
    fail("WhiteSwan's SHA256SUMS is not 42f34a75")
listed = [line.split("  ", 1) for line in (WHITE / "SHA256SUMS").read_text().splitlines()]
bad = [name for digest, name in listed if sha(WHITE / name) != digest]
print(f"PIN WhiteSwan's bundle: SHA256SUMS 42f34a75, {len(listed)} files, {len(listed) - len(bad)} verify")
if bad:
    fail(f"files moved: {bad}")
cand = json.loads((WHITE / "candidates.json").read_text())
inputs = cand["inputs"]
if inputs["bridge"]["sha256"] != BRIDGE_SHA or sha(REPO / inputs["bridge"]["path"]) != BRIDGE_SHA:
    fail("the bridge is not 47069583")
lane_live = REPO / inputs["lane"]["path"]
moved = [rel for rel, digest in inputs["lane"]["files"].items() if sha(lane_live / rel) != digest]
if moved:
    fail(f"live lane files differ from WhiteSwan's pins: {moved}")
live_ingests = sorted(p.parent.name for p in lane_live.glob("ingest-*/ingest.json"))
print(f"PIN live lane: the {len(inputs['lane']['files'])} files WhiteSwan pinned are unchanged; live ingests "
      f"{live_ingests[0]}..{live_ingests[-1]} ({len(live_ingests)}); plans "
      f"{len(sorted(lane_live.glob('plan-*/plan.json')))}")
lane = SCR / "lane"
for rel in inputs["lane"]["files"]:
    (lane / rel).parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(lane_live / rel, lane / rel)

spec = importlib.util.spec_from_file_location("bridge_under_review", REPO / inputs["bridge"]["path"])
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)
intake, bron = bridge.dep("intake"), bridge.dep("log_preparation")
sup = inputs["supplement"]
report = bridge.build_report(REPO / intake.CRATE, lane, REPO / intake.WORK, REPO / intake.TAKES,
                             [[str(REPO / sup["job"]), str(REPO / sup["manifest"])]])
mapped = {name: hashlib.sha256(data.replace(str(lane.resolve()), inputs["lane"]["path"]).encode("utf-8")).hexdigest()
          if isinstance(data, str) else None for name, data in report["out"].items()}
same = mapped == inputs["report_outputs_sha256"]
print(f"{'PASS' if same else 'FAIL'} R1 the bridge's report on the copy, lane path mapped back, equals the 5 outputs "
      f"WhiteSwan bound; source side {report['side']['sha256'][:12]}; {len(report['takes'])} takes")
if not same or report["side"]["sha256"] != inputs["source_sha256"]:
    fail("the report differs from WhiteSwan's")

# ----------------------------------------------------------------------------- measure every point

white = {p["id"]: p for p in cand["late_points"]}
white.update({p["id"]: p for p in cand["other_points"]})
answers = cand["proposed_answers"]["answers"]
records_cache, points, takes_meta = {}, {}, {}
for (cue, name), take in report["takes"].items():
    path = REPO / take["path"]
    if sha(path) != take["sha256"]:
        fail(f"{name} moved")
    rec = take["recognition"]
    log = REPO / intake.WORK / rec["job"] / "command.log"
    if rec["job"] not in records_cache:
        if sha(log) != rec["command_log_sha256"]:
            fail(f"{rec['job']} command.log moved")
        records_cache[rec["job"]] = [json.loads(line) for line in log.read_text(encoding="utf-8").splitlines()
                                     if line.startswith("{")]
    record = next(r for r in records_cache[rec["job"]] if r["index"] == rec["record_index"])
    if Path(record["input"]).resolve() != path.resolve():
        fail(f"{name}: record {rec['record_index']} is another take's")
    segments = record["report"]["result"]["segments"]
    obs = bridge.observed(bron, segments)
    x, db = envelope(path)
    takes_meta[(cue, name)] = {"segments": segments, "x": x, "db": db}
    rows = [("onset", s) for s in take["sentences"][1:]] + [("mark", m) for m in take["anchors"]
                                                            if m["status"] != "sentence_onset"]
    for kind, row in rows:
        win = row.get("window_s") if row.get("status") == "supported" else (row.get("candidate") or {}).get("window_s")
        seg = row.get("segment_index") if row.get("status") == "supported" else (row.get("candidate") or {}).get("segment_index")
        if win is None:
            continue
        j = next(i for i, o in enumerate(obs) if o["seg"] == seg and o["k"] == 0)
        prev = obs[j - 1] if j > 0 else None
        p0, start, end = (prev["start"] if prev else 0.0), win[0], win[1]
        chosen, found = pause(db, QUIET, p0, start, end)
        point = {"id": row["id"], "kind": kind, "cue": cue, "file": name, "take_sha256": take["sha256"],
                 "word": row.get("word"), "status": row.get("status"), "window_s": [r3(win[0]), r3(win[1])],
                 "previous_word": {"w": prev["w"], "start_s": r3(prev["start"])} if prev else None}
        if chosen is None:
            point["class"] = "no_pause"
        else:
            leave, body = chosen[1], body_after(db, chosen[1])
            inner = [r for r in runs(db, QUIET) if r[0] > leave and r[0] < start and r[1] - r[0] >= 0.02]
            point.update(pause_s=[r3(chosen[0]), r3(leave)], body_s=r3(body),
                         quiet_runs_in_window=len(found),
                         chosen_is_last=chosen == max(found, key=lambda r: r[0]),
                         quiet_runs_between_leave_and_start=len(inner),
                         **{"class": "in_pause" if start <= leave + 0.010 else
                            "at_onset" if body is not None and start <= body + 0.030 else "inside_word"})
        w = white.get(row["id"])
        point["white"] = {"class": w["class"], "pause_s": w.get("pause_s"), "body_s": w.get("body_s")} if w else None
        points[row["id"]] = point

# ----------------------------------------------------------------------------- 1. the 62 late points and 60 samples

print("\n1. WhiteSwan's 62 late points, measured independently (times in s; d = my leave - his pause end, ms)")
late_ids = [p["id"] for p in cand["late_points"]]
held = set(cand["held"])
for pid in late_ids:
    p, w = points[pid], white[pid]
    take = takes_meta[(p["cue"], p["file"])]
    db, x = take["db"], take["x"]
    prev_start = p["previous_word"]["start_s"]
    leaves = {}
    for t in THRESHOLDS:
        c, _ = pause(db, t, prev_start, p["window_s"][0], p["window_s"][1])
        leaves[str(int(t))] = r3(c[1]) if c else None
    his_rule, _ = pause(db, -70.0, prev_start, p["window_s"][0], p["window_s"][1], rule="last", min_len=0.06)
    vals = [v for v in leaves.values() if v is not None]
    p["leave_by_threshold"] = leaves
    p["leave_spread_ms"] = round(1000 * (max(vals) - min(vals))) if vals else None
    p["white_rule_on_my_envelope_leave_s"] = r3(his_rule[1]) if his_rule else None
    a = answers.get(pid)
    if a and a.get("decision") == "sample":
        s48 = 2 * a["sample_24k"]
        p["answer"] = {"sample_24k": a["sample_24k"], "seconds": round(a["sample_24k"] / 24000, 6),
                       "equals_round_pause_end": a["sample_24k"] == round(w["pause_s"][1] * 24000),
                       "peak_dbfs_5ms_before": level_db(x, s48 - 240, s48, peak=True),
                       "rms_dbfs_10ms_after": level_db(x, s48, s48 + 480),
                       "rms_dbfs_10_20ms_after": level_db(x, s48 + 480, s48 + 960)}
    d = round(1000 * (p["pause_s"][1] - w["pause_s"][1])) if p.get("pause_s") else None
    p["d_leave_ms"] = d
    print(f"  {pid[len('onset:'):]:<34} {p['word']!r:<11} mine {p['class']:<11} leave {p.get('pause_s', [None, None])[1]} "
          f"d {d:+d}ms  spread {p['leave_spread_ms']}ms  runs {p.get('quiet_runs_in_window')}"
          f"{'' if p.get('chosen_is_last') else ' LONGEST!=LAST'}  sound->start gaps {p.get('quiet_runs_between_leave_and_start')}"
          + (f"  answer {p['answer']['sample_24k']} ({p['answer']['peak_dbfs_5ms_before']} | {p['answer']['rms_dbfs_10ms_after']}, "
             f"{p['answer']['rms_dbfs_10_20ms_after']})" if p.get("answer") else "  no answer")
          + ("  HELD" if p["cue"] in held else ""))

samples = [pid for pid in late_ids if points[pid].get("answer")]
agree = [pid for pid in samples if points[pid]["class"] == "inside_word" and abs(points[pid]["d_leave_ms"]) <= 10
         and points[pid]["answer"]["equals_round_pause_end"]]
print(f"\n  {len(samples)} sample answers; {len(agree)} where my envelope also finds the start inside the word, my leave is "
      f"within 10 ms of his pause end, and the sample is round(pause end x 24000)")
print(f"  my class for the 62: { {c: sum(points[i]['class'] == c for i in late_ids) for c in ('inside_word', 'at_onset', 'in_pause', 'no_pause')} }")
print(f"  largest |d|: {max(abs(points[i]['d_leave_ms']) for i in late_ids)} ms; largest threshold spread: "
      f"{max(points[i]['leave_spread_ms'] for i in late_ids)} ms; longest != last: "
      f"{[i for i in late_ids if not points[i].get('chosen_is_last')]}")
print(f"  his rule re-run on my 1 ms envelope moves his pause end by at most "
      f"{max(abs(round(1000 * (points[i]['white_rule_on_my_envelope_leave_s'] - white[i]['pause_s'][1]))) for i in late_ids)} ms")
pause_lengths = sorted(round(points[i]["pause_s"][1] - points[i]["pause_s"][0], 3) for i in late_ids)
print(f"  pause lengths: min {pause_lengths[0]} s, median {pause_lengths[len(pause_lengths) // 2]} s; "
      f"rise from leave to body: max {max(round(points[i]['body_s'] - points[i]['pause_s'][1], 3) for i in late_ids)} s")
spans = sorted((round(points[i]["window_s"][0] - points[i]["pause_s"][1], 3), i) for i in late_ids)
print(f"  sound from leave to the recognizer start, longest five: {[(s, i[len('onset:'):]) for s, i in spans[-5:]]}")

# ----------------------------------------------------------------------------- 2. every other point

print("\n2. Every sentence onset and recognized mark of the 94 takes (completeness)")
mine = {}
for p in points.values():
    mine.setdefault(p["class"], []).append(p["id"])
print(f"  measured {len(points)} points: " + ", ".join(f"{c} {len(v)}" for c, v in sorted(mine.items())))
extra_late = [i for i in mine.get("inside_word", []) if i not in white or white[i]["class"] != "inside_word"]
missed = [i for i in late_ids if points[i]["class"] != "inside_word"]
print(f"  inside_word by my rule but not in WhiteSwan's late list: {len(extra_late)}")
for i in extra_late:
    p = points[i]
    print(f"    {i}: {p['word']!r} window {p['window_s']} pause {p['pause_s']} body {p['body_s']}; his class "
          f"{p['white']['class'] if p['white'] else None} {p['white']}")
print(f"  in WhiteSwan's late list but not inside_word by my rule: {missed}")

# lag = recognizer start minus the first sound after the pause: no body threshold, so soft onsets count fully
answered = [pid for pid in late_ids if points[pid].get("answer")]
floor_lag = min(points[i]["window_s"][0] - white[i]["pause_s"][1] for i in answered)
print(f"\n  lag from the first sound (his pause end) to the recognizer start: his {len(answered)} answered points "
      f"start at {floor_lag:.3f} s; onsets outside his late list with at least that lag:")
same_lag = []
for p in sorted(points.values(), key=lambda q: q["id"]):
    w = p["white"]
    if p["kind"] != "onset" or p["id"] in set(late_ids) or not w or not w.get("pause_s"):
        continue
    lag = p["window_s"][0] - w["pause_s"][1]
    if lag >= floor_lag - 1e-9:
        p["lag_s"] = round(lag, 3)
        p["provisional_sample_24k"] = round(w["pause_s"][1] * 24000)
        same_lag.append(p["id"])
        print(f"    {p['id']}: {p['word']!r} start {p['window_s'][0]}, his pause end {w['pause_s'][1]} (mine {p['pause_s'][1]}), "
              f"lag {lag:.3f} s, his class {w['class']}; round(pause end x 24000) = {p['provisional_sample_24k']}"
              + ("  HELD" if p["cue"] in held else ""))
disagree = sorted((p["id"], p["class"], p["white"]["class"]) for p in points.values()
                  if p["white"] and p["white"]["class"] != p["class"])
print(f"  class disagreements over all points: {len(disagree)}")
for row in disagree:
    print(f"    {row[0]}: mine {row[1]}, his {row[2]}")

# ----------------------------------------------------------------------------- 3. the two timing questions and 03_4

print("\n3. Questions and single points")
for pid in ("onset:05_4:s3:05_4-download02.wav", "onset:05_4:s3:05_4-download01.wav", "onset:03_4:s1:03_4-download02.wav",
            "anchor:03_2:b90:03_2-download02.wav", "anchor:03_2:b90:03_2-download01.wav"):
    p = points[pid]
    print(f"  {pid}: word {p['word']!r}, status {p['status']}, window {p['window_s']}, previous {p['previous_word']}, "
          f"my pause {p.get('pause_s')} body {p.get('body_s')} class {p['class']}; his {p['white']}; "
          f"answer {answers.get(pid)}")
    take = takes_meta[(p["cue"], p["file"])]
    a = int((p["window_s"][0] - 0.6) * 1000)
    trace = " ".join(f"{k / 1000:.2f}:{round(take['db'][k])}" for k in range(max(0, a), min(len(take["db"]), a + 900), 20))
    print(f"    20 ms trace: {trace}")

anchors_inherit = sorted(m["id"] for t in report["takes"].values() for m in t["anchors"]
                         if m["status"] == "sentence_onset" and m["same_as"] in late_ids)
print(f"  anchor marks at sentence starts that inherit a late onset: {len(anchors_inherit)}")

# ----------------------------------------------------------------------------- 4. content questions

print("\n4. The 18 content questions: script, the bridge's differences, the raw recognizer segments")
script = report["script"]
for row in cand["content"]:
    take = report["takes"][(row["cue"], row["file"])]
    segs = takes_meta[(row["cue"], row["file"])]["segments"]
    a = answers.get(row["id"], {})
    print(f"  {row['id']} [{row['class']}] proposed {a.get('decision')}")
    print(f"    script: {script[row['cue']]!r}")
    for d in take["differences"]:
        print(f"    bridge difference: {d['op']} {d['script']!r} -> {d['recognized']!r}")
    for n in take.get("supplementary_recognition", []):
        for d in n["differences"]:
            print(f"    {n['model']['name']} difference: {d['op']} {d['script']!r} -> {d['recognized']!r}")
    for d in row["differences"]:
        k0, k1 = min(d["raw_segments"]), max(d["raw_segments"])
        ctx = " ".join(f"[{s['start_sec']:.2f}-{s['end_sec']:.2f}]{s['text']!r}" for s in segs[max(0, k0 - 3):k1 + 4])
        print(f"    raw segments {k0}-{k1} +-3: {ctx}")

# ----------------------------------------------------------------------------- 4b. the bridge's own resolve()

print("\n4b. The bridge's resolve() on all 94 takes (answers signed 'independent-check' in memory only)")
timing = {k: v for k, v in answers.items() if v["decision"] in ("sample", "use_candidate")}
extra = {pid: {"decision": "sample", "sample_24k": points[pid]["provisional_sample_24k"]} for pid in same_lag
         if points[pid]["cue"] not in held}
for label, chosen_answers in (("no answers", {}), ("WhiteSwan's 61 timing answers", timing),
                              ("his 61 plus the 6 same-lag onsets", {**timing, **extra})):
    ok, refusals = 0, []
    for take in report["takes"].values():
        applied = {k: dict(v, by="independent-check", basis="checks only") for k, v in chosen_answers.items()
                   if k.rsplit(":", 1)[1] == take["file"] and k.split(":")[1] == take["cue"]}
        try:
            bridge.resolve(take, applied, bridge.expected_frames(take))
            ok += 1
        except Exception as error:  # the bridge's Refused
            refusals.append(f"{take['cue']} {take['file']}: {str(error)[:90]}")
    print(f"  {label}: {ok}/{len(report['takes'])} takes resolve" + (f"; refused: {refusals}" if refusals else ""))

# ----------------------------------------------------------------------------- 5. held cues: durations only

print("\n5. Held cues 02_5 and 08_2: sound stretches (split at >= 0.15 s at or below -60 dBFS) and the recognizers' words")
sup_log = REPO / sup["job"] / "command.log"
if sha(sup_log) != sup["command_log_sha256"]:
    fail("targeted-small01 command.log moved")
sup_records = [json.loads(line) for line in sup_log.read_text(encoding="utf-8").splitlines() if line.startswith("{")]
for (cue, name), take in sorted(report["takes"].items()):
    if cue not in held:
        continue
    db = takes_meta[(cue, name)]["db"]
    stretches, a, quiet_ms, last = [], None, 0, 0
    for k, v in enumerate(db):
        if v > QUIET:
            a, quiet_ms, last = (k if a is None else a), 0, k
        else:
            quiet_ms += 1
            if a is not None and quiet_ms >= 150:
                stretches.append((a / 1000, (last + 10) / 1000))
                a = None
    if a is not None:
        stretches.append((a / 1000, (last + 10) / 1000))
    print(f"  {cue} {name}: " + " ".join(f"[{s:.2f}-{e:.2f}]{e - s:.2f}s" for s, e in stretches))
    words = takes_meta[(cue, name)]["segments"]
    print("    large-v3-turbo: " + " ".join(f"[{s['start_sec']:.2f}-{s['end_sec']:.2f}]{s['text']}" for s in words))
    for r in sup_records:
        if Path(r["input"]).resolve() == (REPO / take["path"]).resolve():
            segs = r["report"]["result"]["segments"]
            print("    small.en:       " + " ".join(f"[{s['start_sec']:.2f}-{s['end_sec']:.2f}]{s['text']}" for s in segs))

with (SCR / "review.json").open("x") as handle:
    json.dump({"schema": "complex-frederick-current-timing-independent-v1",
               "white_sums": WHITE_SUMS, "bridge_sha256": BRIDGE_SHA, "source_sha256": report["side"]["sha256"],
               "method": __doc__.split("Method")[1].strip(), "same_lag_onsets": same_lag,
               "points": [points[k] for k in sorted(points)]}, handle, indent=1)
    handle.write("\n")
print(f"\nwrote SCRATCH/review.json ({len(points)} points)")
