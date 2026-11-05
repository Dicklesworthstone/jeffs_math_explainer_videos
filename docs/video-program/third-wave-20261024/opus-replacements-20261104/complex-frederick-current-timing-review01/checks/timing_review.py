"""Checks behind complex-frederick-current-timing-review01 (root Mail 4394): the current Frederick timing questions,
read with the repaired bridge. Read-only on every input.

From the repository root:

    python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-current-timing-review01/checks/timing_review.py NEW_SCRATCH_DIR

1. Pins: the repaired bridge and its seal, the 36 lane files its report reads (plans and ingests 01-09) and root's
   targeted-small01 job. The bridge pins its own sources. Exit 3 if one moved.
2. Copies those 36 files into scratch and runs the repaired bridge's read-only `report` on the copy, with
   targeted-small01 as a supplement. While root's lane still holds exactly plans and ingests 01-09, it also runs the
   report on the live lane and requires the same bytes, with the lane path mapped back.
3. Content questions: the recognizer's raw segment text at every normalized difference (the guard jobs' fw records),
   against the reading stated in CONTENT.
4. Timing: every sentence onset and anchor mark the report holds (supported points and its candidates) against the
   take's own level envelope. PCM is read with the wave module: no ffmpeg, no conversion, nothing written by a take.
5. The proposed answers go through the bridge's own resolve() (sentence order, range, marks inside their sentence).
6. Writes candidates.json; run from the bundle, the bundle's copy must be byte-identical. Hygiene.

Nothing here synthesizes, decodes, runs recognition or the guard, launches a job, listens or accepts a take.
Exit 0 PASS, 1 FAIL, 2 usage, 3 a pinned input moved.
"""
import array
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import shutil
import subprocess
import sys
import wave

sys.dont_write_bytecode = True
REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
BUNDLE = HERE.parent
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
BRIDGE_DIR = f"{P}/complex-frederick-timing-bridge-repair01"
BRIDGE = f"{BRIDGE_DIR}/frederick_timing_bridge.py"
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
LANE = f"{EC}/frederick-stage05-recognition01"
W = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SUPPLEMENT = (f"{W}/complex-frederick-targeted-small01", f"{EC}/frederick-targeted-small01/inputs.json")
PINS = {
    BRIDGE: "47069583af82e2aa28688f1ed4179f3b40694450f2bf282acff3549aaf406a39",
    f"{BRIDGE_DIR}/SHA256SUMS": "d5a413dd5856d91338c3f3c248bd194cc7feb80a9a9d635a65043a904c3b025d",
    f"{W}/complex-frederick-targeted-small01/command.log":
        "6d7c6216917a82f6cf30dfb406335efc3e7dbb5ad7a504378b35236b5fbbc80b",
    f"{W}/complex-frederick-targeted-small01/resource-receipt.json":
        "ba98ad86093e993d055f0eb8735b3496c0b8abeb21f37a0001c1fdb213a53b34",
    f"{EC}/frederick-targeted-small01/inputs.json": "d38866483000cfa563139cdaad56f793d58d9d37b397cd80461adadfac4812e1",
    f"{EC}/frederick-targeted-small01/inputs.txt": "89b603ba58c0d5de0fd597d95391bd47b845617909c2a62a619471bdb4b9c376",
}
LANE_PINS = {  # every lane file the bridge's report reads, as of root's Mail 4394
    "plan-01/plan.json": "cb9ae156d97567bc2eb4cf871dcceb9c9eed0ac35df634aa9223233aacea3419",
    "plan-01/frozen-inputs.json": "54acd3096bc0c08cc54df0a7d24db5bda9a9b9e228593258de96673e09b7c473",
    "ingest-01/ingest.json": "b3f24232e71b6c3e6d6d03147aa46317e6d3ee7f8d305a734860cf6311b4a84a",
    "ingest-01/transcripts.json": "db2d0b13e716004b65e3bba2f16210cc3fcdf75cf3def6cb4433344530dae436",
    "plan-02/plan.json": "8d63900896b4b5737b903fb2cad2c37cfa070c7113d7f0d0d7d889066c91475b",
    "plan-02/frozen-inputs.json": "aaeba9acf529152342c5942b23eb55beae26a20ff326f1aba96d256cb0c29cb4",
    "ingest-02/ingest.json": "22b4ebb2e08c1bceb7a09be64246b4ba3747bec74c728629ebb529082213edee",
    "ingest-02/transcripts.json": "6637cf5de0dd6d9a61eb348c7d7c30175bc4ce276692d600fb87d2b15b8540b1",
    "plan-03/plan.json": "bb32e783855475fdcd1a93d55a9bf5a63d87314d7fad3adeab3f621969e7d20e",
    "plan-03/frozen-inputs.json": "327456005cb394af01f4cf35acb1c484124090509c9d3b073983eff01bada38b",
    "ingest-03/ingest.json": "9463a6825fe8e023b53c5747f78dabffa2fc7812d0ff2e2f1a67faa8f7e4d11d",
    "ingest-03/transcripts.json": "3b0ff70ffc8b6cadf7b2697f2641a1f221a93524236128671834160d5df1cf71",
    "plan-04/plan.json": "5382ffe6f8cc633899e42d5e09dd86f8cf0ccb593f27bbb28a5925e6c10408d2",
    "plan-04/frozen-inputs.json": "f5a8c19588e9f1af3b96bb7f8b5e54a26eab57b45e1f1cb8052f80a49f9b3a62",
    "ingest-04/ingest.json": "9eb272028a29b1cfb9b039e35132a550721aa3779d926747b3ba2c54964c8f36",
    "ingest-04/transcripts.json": "b1e0c9001daa7e920fa252c8c71a8704db9b39829b9c38f2dad3f478feb41a58",
    "plan-05/plan.json": "02d0b3443ffa01f4884bebafa191ade7f57f76bd76eee2c899920dae53ddac54",
    "plan-05/frozen-inputs.json": "c7e8fafc82795a094c6afaf689da122e7edf92444dcece26682a7db451180ec5",
    "ingest-05/ingest.json": "33a2d387ab4cf21bf573d21e5af8e8ab0aac307489fc9d7723328f4787f0d8af",
    "ingest-05/transcripts.json": "a1636c9c3b808c512d319de4bbf65201b63a30edaa54c5b003086a8d4fce59f1",
    "plan-06/plan.json": "45fb55e52aa206f5b3bc7e6677683e350c1cc9a93689cdcc4a44e936b7d12723",
    "plan-06/frozen-inputs.json": "c99060f3fa40531d4fe461034f98ae0b2ba28cee8f3b6a7d0f5e53c1a5712b69",
    "ingest-06/ingest.json": "b41205b1f65762b8a1a24a6023ee23e7f8f6f2f8017fc6b8a10a0717ba967668",
    "ingest-06/transcripts.json": "68abf4f6ed41e2a1869c5ab3f8977ae0db3206642623de2a951bf2e00a2a0460",
    "plan-07/plan.json": "2758f1c4ee1c202c5ad87ab3b0b59ac9d0b7656ce62408b0d38351341647b845",
    "plan-07/frozen-inputs.json": "ba701d39fc2ab579fd436a246dedcfa4c1474bb914be3bc94149de9074808a94",
    "ingest-07/ingest.json": "fc10e44b1164f47c5d3becd2c64e23069e249f9470b78068cf5e54514f848cfe",
    "ingest-07/transcripts.json": "3fa932c27b9df4c4b25cb1f90f8495766cd477499826f12abbb3be8cc21b12d5",
    "plan-08/plan.json": "26e1ae7338a83b3eba666d925f6ae64e221abedd1705ad5b2bf0422895afc55f",
    "plan-08/frozen-inputs.json": "0e307f4be77639c14acb6478dbbb0b57217188f7140e2361f2e76ae0ea69397b",
    "ingest-08/ingest.json": "8170bd5ce34c2a3ff35698bf680d61defa626b039f0c1844568388f0fa22047b",
    "ingest-08/transcripts.json": "39a1936d48dfb99544a3e2c80599211af4e08455fe7c014cda225465605e54f6",
    "plan-09/plan.json": "150c3a0f0a7232fec03e6585af33873ae44c895f6574878d118976c2d603d661",
    "plan-09/frozen-inputs.json": "e13e203ce63835e0e815e6996783adde311124d2fbfb76a1b3427a3dc574766f",
    "ingest-09/ingest.json": "5390d6e1e0dc61bbd05137d88a7e4dd7aa143e46cb383bd25e209ca4808d2041",
    "ingest-09/transcripts.json": "28a8056eae39f2719d9ad7629151aba74221edd199239039270a230bc74691ab",
}
# Root's Mail 4394: these stay open; their points are reported, never proposed.
HELD = {"02_5": "hearing open: large-v3-turbo writes nothing for both 'one plus i', small.en (targeted-small01) "
                "writes them; root's instruction keeps the cue open",
        "08_2": "f(z)=z uncertain pending root's queued targeted-small02; root's instruction keeps the cue open"}
# The recognizer's raw segment text at each normalized difference (agreeing neighbour to agreeing neighbour), and
# what that text is. A reading of the recognizer's text, never of the audio.
NOTATION = "the recognizer wrote the script's words as notation, which the normalizer reads as other words"
SOUND = "the recognizer spelled a word that sounds like the script's word"
CONTENT = {
    "content:01_3:01_3-download01.wav": ("notation", ["x^40 gives"]),
    "content:01_3:01_3-download02.wav": ("notation", ["x^40 gives"]),
    "content:01_6:01_6-download01.wav": ("same_sound", ["count routes we"]),
    "content:01_6:01_6-download02.wav": ("same_sound", ["count routes we"]),
    "content:02_4:02_4-download02.wav": ("open", ["a term."]),
    "content:02_5:02_5-download01.wav": ("held", ["by once", "a term. What", "is squared?"]),
    "content:02_5:02_5-download02.wav": ("held", ["by once", "a term. What", "is squared?"]),
    "content:03_2:03_2-download01.wav": ("notation", ["1/2. Its", "1/4, twice."]),
    "content:04_7:04_7-download01.wav": ("notation", ["and -i,"]),
    "content:04_7:04_7-download02.wav": ("notation", ["and -i,"]),
    "content:05_4:05_4-download01.wav": ("same_sound", ["Sign has", "fast. Sign of", "and sign of"]),
    "content:05_4:05_4-download02.wav": ("same_sound", ["Sign has", "fast. Sign of", "and sign of"]),
    "content:06_4:06_4-download01.wav": ("notation", ["1+i", "1+i,", "2+2i."]),
    "content:06_4:06_4-download02.wav": ("notation", ["1+i", "1+i,", "2 + 2i."]),
    "content:07_2:07_2-download01.wav": ("notation", ["a non-zero derivative"]),
    "content:07_2:07_2-download02.wav": ("notation", ["a non-zero derivative"]),
    "content:08_2:08_2-download01.wav": ("held", ["f = z,"]),
    "content:08_2:08_2-download02.wav": ("held", ["f = z,"]),
}
READING = {
    "notation": NOTATION,
    "same_sound": SOUND + " ('Sign' for sine; 'routes' for roots, which British English says alike)",
    "open": "the recognizer wrote a different word ('term' for turn); only hearing settles it",
    "held": "held by root's instruction",
}
# The envelope rule (5 ms RMS frames, dBFS of the 16-bit take).
FRAME_S, QUIET_DB, PAUSE_S, RISE_DB, BODY_DB, LATE_S, GAP_DB = 0.005, -70.0, 0.06, -60.0, -30.0, 0.03, -50.0
RULE = ("A pause is a run of at least 0.06 s of 5 ms frames at or below -70 dBFS. A point is assessed only when "
        "such a pause lies wholly after the start of the previous recognized word and ends by the point's own "
        "segment end. The pause used is the last of those that began by the recognizer start, so the sound after "
        "it can only be this word (or a breath before it). 'leave' is that pause's end. 'body' is the first frame at "
        "or above -30 dBFS after leave, where the word is clearly sounding. inside_word: the recognizer start is "
        "more than 0.03 s after body, so a boundary there puts the first part of the word in the sentence before; "
        "late_by_s = start - body is a lower bound. at_onset: within (leave, body + 0.03]. in_pause: at or before "
        "leave. before_pause: no such pause began by the start, and the start lies more than 0.03 s before the "
        "first one in the window, in sound that precedes it. not_assessed: no such pause (connected speech, or the "
        "previous word starts inside the pause).")
SCRATCH, FAILS = None, []


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha_bytes(Path(path).read_bytes())


def say(text=""):
    text = str(text)
    for path, name in ((SCRATCH.resolve(), "SCRATCH"), (SCRATCH, "SCRATCH"), (REPO.resolve(), "REPO"), (REPO, "REPO")):
        text = text.replace(str(path), name)
    print(text, flush=True)


def check(name, ok, detail):
    if not ok:
        FAILS.append(name)
    say(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def dumps(value):
    return json.dumps(value, indent=1, ensure_ascii=False) + "\n"


def report(lane, out):
    r = subprocess.run([sys.executable, "-I", "-B", str(REPO / BRIDGE), "report", "--lane", str(lane), "--supplement",
                        *SUPPLEMENT, "--out", str(out)], cwd=REPO, capture_output=True, text=True)
    if r.returncode == 3:
        say(f"PIN MOVED (bridge): {r.stderr.strip()}")
        sys.exit(3)
    return r


# ----------------------------------------------------------------------------- envelope

def levels(path):
    with wave.open(str(path)) as w:
        if w.getcomptype() != "NONE" or w.getsampwidth() != 2 or w.getnchannels() != 1:
            raise SystemExit(f"{path.name}: not 16-bit mono PCM")
        rate = w.getframerate()
        pcm = array.array("h", w.readframes(w.getnframes()))
    step = round(rate * FRAME_S)
    out = []
    for k in range(len(pcm) // step):
        chunk = pcm[k * step:(k + 1) * step]
        rms = math.sqrt(sum(x * x for x in chunk) / step)
        out.append(20 * math.log10(rms / 32768) if rms else -120.0)
    return out


def pauses(lv):
    runs, k = [], 0
    while k < len(lv):
        if lv[k] <= QUIET_DB:
            j = k
            while j < len(lv) and lv[j] <= QUIET_DB:
                j += 1
            if (j - k) * FRAME_S >= PAUSE_S - 1e-9:
                runs.append((k, j))
            k = j
        else:
            k += 1
    return runs


def assess(lv, runs, start, end, prev_start):
    inside = [(a, b) for a, b in runs if a * FRAME_S > prev_start and b * FRAME_S <= end + 1e-9]
    if not inside:
        return {"class": "not_assessed"}
    begun = [(a, b) for a, b in inside if a * FRAME_S <= start + 1e-9]
    a, b = begun[-1] if begun else inside[0]
    q0, leave = round(a * FRAME_S, 3), round(b * FRAME_S, 3)
    body = next((round(i * FRAME_S, 3) for i in range(b, len(lv)) if lv[i] >= BODY_DB), None)
    rise = next((round(i * FRAME_S, 3) for i in range(b, len(lv)) if lv[i] >= RISE_DB), None)
    if not begun and start < q0 - LATE_S:
        cls = "before_pause"
    elif start <= leave + 0.0005:
        cls = "in_pause"
    elif body is not None and start > body + LATE_S:
        cls = "inside_word"
    else:
        cls = "at_onset"
    row = {"class": cls, "pause_s": [q0, leave], "rise_s": rise, "body_s": body}
    if cls == "inside_word":
        row["late_by_s"] = round(start - body, 3)
        between = lv[round(body / FRAME_S):round(start / FRAME_S)]
        run = longest = 0
        for x in between:
            run = run + 1 if x < GAP_DB else 0
            longest = max(longest, run)
        row["body_to_start"] = {"min_dbfs": round(min(between), 1), "longest_below_-50dbfs_s": round(longest * FRAME_S, 3)}
    if cls == "in_pause" and rise is not None:
        row["early_by_s"] = round(rise - start, 3)
    return row


# ----------------------------------------------------------------------------- sections

def section_pins():
    say("== 1. Pins")
    moved = [p for p, h in PINS.items() if not (REPO / p).is_file() or sha_file(REPO / p) != h]
    moved += [f"{LANE}/{p}" for p, h in LANE_PINS.items() if not (REPO / LANE / p).is_file()
              or sha_file(REPO / LANE / p) != h]
    if moved:
        say("PIN MOVED: " + ", ".join(moved))
        sys.exit(3)
    sums = (REPO / BRIDGE_DIR / "SHA256SUMS").read_text().splitlines()
    sealed = all(sha_file(REPO / BRIDGE_DIR / x.split("  ", 1)[1]) == x.split("  ", 1)[0] for x in sums)
    check("p1-pins", sealed, f"the repaired bridge {PINS[BRIDGE][:8]} and its seal d5a413dd ({len(sums)} files) verify; "
          f"the {len(LANE_PINS)} lane files the report reads (plans and ingests 01-09) and root's targeted-small01 "
          "job are at their pins; the bridge checks its own source pins as it runs")


def section_report():
    say("== 2. The repaired bridge's read-only report on a pinned copy of the lane")
    copy = SCRATCH / "lane-copy"
    for name in LANE_PINS:
        (copy / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / LANE / name, copy / name)
    out = SCRATCH / "report"
    r = report(copy, out)
    check("r1-report", r.returncode == 0 and r.stdout.startswith("source: 85 hook sites"),
          r.stdout.strip().split("; written to")[0] if r.returncode == 0 else r.stderr.strip())
    if r.returncode != 0:
        raise SystemExit(1)
    lane_rel = str(copy.resolve())
    texts = {n: (out / n).read_text(encoding="utf-8").replace(lane_rel, LANE) for n in sorted(p.name for p in
                                                                                             out.iterdir())}
    live_names = sorted(p.name for p in (REPO / LANE).iterdir() if re.fullmatch(r"(plan|ingest)-\d\d|release-\d\d\.json",
                                                                                 p.name))
    pinned_names = sorted({n.split("/")[0] for n in LANE_PINS})
    if live_names == pinned_names:
        live = SCRATCH / "report-live"
        r2 = report(REPO / LANE, live)
        same = r2.returncode == 0 and all((live / n).read_text(encoding="utf-8") == t for n, t in texts.items())
        check("r2-live-lane-same", same, "root's lane still holds exactly plans and ingests 01-09, and the report on it is "
              "byte-identical to the copy's (5 files, lane path mapped back)")
    else:
        say(f"NOTE r2: root's lane has moved on ({', '.join(sorted(set(live_names) - set(pinned_names)))} new): only "
            "the pinned copy is read")
    return {n: json.loads(t) for n, t in texts.items()}, {n: sha_bytes(t.encode()) for n, t in texts.items()}


def fw_segments(take, jobs):
    log = jobs / take["recognition"]["job"] / "command.log"
    if sha_file(log) != take["recognition"]["command_log_sha256"]:
        raise SystemExit(f"{take['file']}: command.log moved")
    for line in log.read_text(encoding="utf-8").splitlines():
        if line.startswith("{"):
            record = json.loads(line)
            if record["index"] == take["recognition"]["record_index"]:
                return record["report"]["result"]["segments"]
    raise SystemExit(f"{take['file']}: no record")


def raw_at(bridge, bron, text, segments):
    canon, obs, match, run, replaced, diffs = bridge.align(bron, text, segments)
    out = []
    for d in diffs:
        i = d["script_token_index"]
        n = len(bron.tokens(d["script"])) if d["script"] else 0
        lo = obs[match[i - 1]]["seg"] if i - 1 in match else 0
        hi = obs[match[i + n]]["seg"] if i + n in match else len(segments) - 1
        out.append({"op": d["op"], "script": d["script"], "recognized": d["recognized"],
                    "raw": " ".join(str(segments[k]["text"]).strip() for k in range(lo, hi + 1)),
                    "raw_segments": [lo, hi]})
    return out


def section_content(bridge, bron, data, script, jobs):
    say("== 3. Content questions: the recognizer's raw text at each difference")
    takes = {t["file"]: t for t in data["takes.json"]["takes"]}
    small = {}
    for line in Path(REPO / SUPPLEMENT[0] / "command.log").read_text(encoding="utf-8").splitlines():
        if line.startswith("{"):
            record = json.loads(line)
            small[bridge.rel(Path(record["input"]))] = record
    rows, ok, wrong = [], True, []
    content = [q for q in data["questions.json"]["questions"] if q["kind"] == "content"]
    for q in content:
        take = takes[q["files"][0]]
        found = raw_at(bridge, bron, script[q["cue"]], fw_segments(take, jobs))
        cls, expected = CONTENT.get(q["id"], (None, None))
        if [f["raw"] for f in found] != expected:
            wrong.append(q["id"])
        row = {"id": q["id"], "cue": q["cue"], "file": take["file"], "take_sha256": take["sha256"],
               "recognition": {"job": take["recognition"]["job"],
                               "command_log_sha256": take["recognition"]["command_log_sha256"],
                               "record_index": take["recognition"]["record_index"]},
               "class": cls, "reading": READING.get(cls), "differences": found}
        if take["path"] in small:
            rec = small[take["path"]]
            row["small_en"] = {"job": Path(SUPPLEMENT[0]).name, "record_index": rec["index"],
                               "differences": raw_at(bridge, bron, script[q["cue"]], rec["report"]["result"]["segments"])}
        if q["cue"] in HELD:
            row["held"] = HELD[q["cue"]]
        rows.append(row)
    counts = {c: sum(r["class"] == c for r in rows) for c in READING}
    check("c1-content-raw-text", not wrong and len(rows) == len(CONTENT) and set(CONTENT) == {r["id"] for r in rows},
          f"all {len(rows)} content questions; the fw records' raw text at each difference is as stated: "
          f"{counts['notation']} notation (x^40, 1/2 and 1/4, -i, 1+i and 2+2i, non-zero), {counts['same_sound']} "
          f"same sound (Sign, routes), {counts['open']} open (02_4 'term'), {counts['held']} held (02_5, 08_2)"
          + (f"; differs: {wrong}" if wrong else ""))
    for r in rows:
        say(f"   {r['id']}: {r['class']}: " + "; ".join(f"{d['op']} {d['script']!r} -> {d['recognized']!r} raw "
                                                     f"{d['raw']!r}" for d in r["differences"])
            + ("; small.en: " + "; ".join(f"{d['op']} {d['script']!r} -> {d['recognized']!r} raw {d['raw']!r}"
                                          for d in r["small_en"]["differences"]) if "small_en" in r else ""))
    return rows


def section_timing(bridge, bron, data, script, jobs):
    say("== 4. Timing: every onset and mark against its take's own level envelope")
    rows, by_class = [], {}
    for t in data["takes.json"]["takes"]:
        path = REPO / t["path"]
        if sha_file(path) != t["sha256"]:
            raise SystemExit(f"{t['file']} moved")
        lv = levels(path)
        runs = pauses(lv)
        canon, obs, *_ = bridge.align(bron, script[t["cue"]], fw_segments(t, jobs))
        for p in t["sentences"][1:] + t["anchors"]:
            if p.get("status") == "sentence_onset":
                continue
            point = p if p.get("status") == "supported" else p.get("candidate")
            kind = "onset" if p["id"].startswith("onset:") else "mark"
            if point is None:
                by_class[(kind, "no point")] = by_class.get((kind, "no point"), 0) + 1
                continue
            j = next(i for i, o in enumerate(obs) if o["seg"] == point["segment_index"] and o["k"] == 0)
            prev = obs[j - 1] if j else {"w": None, "start": -1.0, "end": 0.0}
            a = assess(lv, runs, point["asr_start_s"], point["asr_end_s"], prev["start"])
            for x in p.get("supplementary") or []:
                if x["status"] == "supported":
                    xa = assess(lv, runs, x["window_s"][0], x["window_s"][1], prev["start"])
                    a.setdefault("small_en", []).append({"job": x["job"], "window_s": x["window_s"],
                                                         "class": xa["class"]})
            source = "supported" if point is p else "candidate"
            by_class[(kind, source, a["class"])] = by_class.get((kind, source, a["class"]), 0) + 1
            rows.append(dict({"id": p["id"], "cue": t["cue"], "file": t["file"], "take_sha256": t["sha256"],
                              "kind": kind, "point": source, "word": p.get("word") or p.get("phrase"),
                              "asr_window_s": point["window_s"], "abuts_previous_word": point.get("abuts_previous_word"),
                              "sample_24k_now": point["sample_24k"],
                              "previous_word": {"w": prev["w"], "window_s": [round(prev["start"], 6),
                                                                              round(prev["end"], 6)]},
                              "recognition": {"job": t["recognition"]["job"], "record_index":
                                              t["recognition"]["record_index"],
                                              "command_log_sha256": t["recognition"]["command_log_sha256"],
                                              "segment_index": point["segment_index"]}}, **a))
    for key in sorted(by_class):
        say(f"   {' '.join(key)}: {by_class[key]}")
    late = [r for r in rows if r["class"] == "inside_word"]
    early = sorted(r["early_by_s"] for r in rows if r["class"] == "in_pause" and "early_by_s" in r)
    check("e1-envelope", len(rows) == 266 + 56 - 2 + 2 and len({r["take_sha256"] for r in rows}) == 94,
          f"{len(rows)} points in 94 takes (264 supported onsets, the 2 candidates for the 2 open onsets, 56 supported "
          f"marks); {len(late)} start inside their word (late by at least {min(r['late_by_s'] for r in late):.3f} s, "
          f"at most {max(r['late_by_s'] for r in late):.3f} s, measured from body); {len(early)} lie in the pause "
          f"before their word (rise minus start: median {early[len(early) // 2]:.3f} s, max {early[-1]:.3f} s); every "
          "take's WAV is at its plan hash")
    for r in sorted(late, key=lambda r: -r["late_by_s"]):
        say(f"   late {r['late_by_s']:.3f} s: {r['id']} {r['word']!r} start {r['asr_window_s'][0]} after pause "
            f"{r['pause_s']} body {r['body_s']}{' (held)' if r['cue'] in HELD else ''}")
    gaps = sorted((r["body_to_start"]["longest_below_-50dbfs_s"], r["id"]) for r in late)
    check("e2-one-word", all(g <= 0.02 for g, _ in gaps),
          f"between body and the recognizer start the sound never drops below -50 dBFS for more than 0.02 s, so it is "
          f"one stretch of sound, not a breath then a word: {sum(g == 0 for g, _ in gaps)} of {len(gaps)} never drop "
          "below -50 dBFS; the others: " + ", ".join(f"{i.split(':')[1]} {i.rsplit(':', 1)[1][:-4]} {g:.3f} s"
                                                     for g, i in gaps if g))
    for r in rows:
        if r["class"] == "before_pause":
            say(f"   before the pause: {r['id']} {r['word']!r} start {r['asr_window_s'][0]}, pause {r['pause_s']}, "
                f"body {r['body_s']}: no proposal")
    crossed = [r for r in rows if r.get("small_en")]
    agree = all(x["class"] != "inside_word" for r in crossed if r["class"] == "inside_word" for x in r["small_en"])
    check("x1-independent-recognizer", len(crossed) == 6 and agree and any(r["class"] == "inside_word" for r in crossed),
          "on the 6 points root's targeted-small01 (small.en) also places, every point the envelope finds late in "
          "large-v3-turbo is not late in small.en: " + "; ".join(
              f"{r['id'].split(':')[2]} {r['file'][:-4]} turbo {r['asr_window_s'][0]} {r['class']}, small.en "
              f"{r['small_en'][0]['window_s'][0]} {r['small_en'][0]['class']}" for r in crossed))
    return rows


def proposals(timing, content, data):
    answers = {}
    questions = {q["id"]: q for q in data["questions.json"]["questions"]}
    for r in timing:
        if r["cue"] in HELD:
            continue
        if r["point"] == "candidate":
            if r["class"] in ("in_pause", "at_onset"):
                answers[r["id"]] = {"decision": "use_candidate", "by": None, "basis":
                                    f"the recognizer's 'sign' in this slot starts at {r['asr_window_s'][0]} s, inside the "
                                    f"take's pause {r['pause_s']} s just before the word sounds (body {r['body_s']} s); "
                                    "envelope reading, no listening"}
            elif r["class"] == "inside_word":
                answers[r["id"]] = {"decision": "sample", "sample_24k": round(r["pause_s"][1] * 24000), "by": None,
                                    "basis": f"the recognizer's 'sign' starts at {r['asr_window_s'][0]} s, {r['late_by_s']} "
                                             f"s after the word sounds (body {r['body_s']} s); this is the end of the "
                                             f"take's pause {r['pause_s']} s; envelope reading, no listening"}
        elif r["class"] == "inside_word":
            answers[r["id"]] = {"decision": "sample", "sample_24k": round(r["pause_s"][1] * 24000), "by": None,
                                "basis": f"the recognizer start {r['asr_window_s'][0]} s is {r['late_by_s']} s after the "
                                         f"word sounds (body {r['body_s']} s); this is the end of the take's pause "
                                         f"{r['pause_s']} s; envelope reading, no listening"}
    for r in content:
        if r["class"] in ("notation", "same_sound") and "proceed" in questions[r["id"]]["decisions"]:
            answers[r["id"]] = {"decision": "proceed", "by": None, "basis": "the recognizer's raw text: " + "; ".join(
                f"{d['raw']!r}" for d in r["differences"]) + f": {READING[r['class']]}; a transcript reading, no "
                "listening"}
    return answers


def section_resolve(bridge, data, answers, timing):
    say("== 5. The proposals through the bridge's own resolve()")
    takes = data["takes.json"]["takes"]
    moved, refused, inherited = [], [], 0
    by_take = {}
    for qid, a in answers.items():
        by_take.setdefault(qid.rsplit(":", 1)[1], {})[qid] = dict(a, by="simulation")
    before = {}
    for t in takes:
        try:
            before[t["file"]] = bridge.resolve(t, {}, bridge.expected_frames(t))
        except Exception as error:
            before[t["file"]] = str(error)
    open_before = sorted(f for f, v in before.items() if isinstance(v, str))
    for t in takes:
        applied = {k: v for k, v in by_take.get(t["file"], {}).items() if not k.startswith("content:")}
        try:
            starts, ends, marks = bridge.resolve(t, applied, bridge.expected_frames(t))
        except Exception as error:
            refused.append(f"{t['file']}: {error}")
            continue
        if not isinstance(before[t["file"]], str):
            old_marks = before[t["file"]][2]
            inherited += sum(1 for a, b in zip(old_marks, marks) if a["sample_24k"] != b["sample_24k"]
                             and b["basis"].startswith("sentence "))
        for s in t["sentences"][1:]:
            if s["id"] in applied:
                moved.append(s["id"])
    n_onsets = sum(1 for k in answers if k.startswith("onset:"))
    n_marks = sum(1 for k in answers if k.startswith("anchor:"))
    check("s1-resolve", not refused and open_before == ["05_4-download01.wav", "05_4-download02.wav"],
          f"without answers the bridge resolves 92 of 94 takes (the 2 without: {open_before}); with the {n_onsets} "
          f"onset and {n_marks} mark proposals it resolves all 94 under its sentence-order, range and mark rules; "
          f"{inherited} sentence-onset marks follow their moved onsets" + (f"; refused: {refused}" if refused else ""))


def section_hygiene(candidates_text):
    if BUNDLE.name != "complex-frederick-current-timing-review01":
        say("== 7. Hygiene: skipped (not run from the bundle)")
        return
    say("== 7. Hygiene of this bundle")
    machine = re.compile(r"/Users/|/private/|/home/|/var/folders|jemanuel|scratchpad|claude-501|@gmail")  # hygiene-pattern
    pronoun = re.compile(r"\b(he|she|his|her|him|hers|himself|herself)\b", re.I)  # hygiene-pattern
    files = sorted(p for p in BUNDLE.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
                   and "__pycache__" not in p.relative_to(BUNDLE).parts)
    names = [str(p.relative_to(BUNDLE)) for p in files]
    want = ["FINDINGS.md", "candidates.json", "checks/outputs/timing_review.txt", "checks/timing_review.py"]
    dirty = [n for n, p in zip(names, files) if any(
        (machine.search(x) or pronoun.search(x)) and not x.endswith("# hygiene-pattern")
        for x in p.read_text(encoding="utf-8", errors="replace").splitlines())]
    same = (BUNDLE / "candidates.json").is_file() and (BUNDLE / "candidates.json").read_text(encoding="utf-8") == \
        candidates_text
    check("h1-hygiene", names == want and not dirty and same,
          f"exactly the {len(want)} bundle files; candidates.json is byte-identical to this run's; no machine path, "
          f"user name, address or gendered pronoun{(' ' + str(dirty)) if dirty else ''}"
          f"{(' ' + str(names)) if names != want else ''}")


def main():
    global SCRATCH
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    SCRATCH = Path(sys.argv[1]).absolute()
    if SCRATCH.exists() or not (REPO / "tools/coherent_narration.py").is_file():
        print("give a NEW scratch folder and run from the repository root")
        return 2
    SCRATCH.mkdir(parents=True)
    section_pins()
    bridge = load("frederick_timing_bridge", REPO / BRIDGE)
    bron = bridge.dep("log_preparation")
    jobs = REPO / bridge.dep("intake").WORK
    data, hashes = section_report()
    script = bridge.source_side(REPO / bridge.dep("intake").CRATE)[1]
    content = section_content(bridge, bron, data, script, jobs)
    timing = section_timing(bridge, bron, data, script, jobs)
    answers = proposals(timing, content, data)
    section_resolve(bridge, data, answers, timing)
    say("== 6. candidates.json")
    side = data["source.json"]["sha256"]
    candidates = {
        "schema": "complex-frederick-current-timing-review-v1",
        "claim": "Readings of root's recognition records and of each take's own level envelope, bound to the hashes "
                 "below. Every proposal has `by: null`: the bridge refuses it until root decides and signs it. "
                 "Nothing here was heard, decoded, measured by ear or accepted.",
        "inputs": {"bridge": {"path": BRIDGE, "sha256": PINS[BRIDGE]}, "source_sha256": side,
                   "lane": {"path": LANE, "files": LANE_PINS},
                   "supplement": {"job": SUPPLEMENT[0], "manifest": SUPPLEMENT[1],
                                  "command_log_sha256": PINS[f"{SUPPLEMENT[0]}/command.log"],
                                  "resource_receipt_sha256": PINS[f"{SUPPLEMENT[0]}/resource-receipt.json"]},
                   "report_outputs_sha256": hashes},
        "envelope_rule": RULE, "held": HELD,
        "content": content,
        "late_points": [r for r in timing if r["class"] == "inside_word"],
        "other_points": [{k: r[k] for k in ("id", "class", "asr_window_s", "pause_s", "body_s", "early_by_s")
                          if k in r} for r in timing if r["class"] != "inside_word"],
        "proposed_answers": {"schema": data["answers-template.json"]["schema"], "source_sha256": side,
                             "answers": dict(sorted(answers.items()))},
    }
    text = dumps(candidates)
    (SCRATCH / "candidates.json").write_text(text, encoding="utf-8")
    decisions = {}
    for a in answers.values():
        decisions[a["decision"]] = decisions.get(a["decision"], 0) + 1
    check("w1-candidates", all(a["by"] is None for a in answers.values()),
          f"candidates.json ({sha_bytes(text.encode())[:12]}): {len(answers)} proposals ({decisions}), every one with "
          "by null; held cues and open content carry none")
    section_hygiene(text)
    say(f"RESULT {'PASS' if not FAILS else 'FAIL'}, {len(FAILS)} failed{(': ' + ', '.join(FAILS)) if FAILS else ''}")
    return 0 if not FAILS else 1


if __name__ == "__main__":
    sys.exit(main())
