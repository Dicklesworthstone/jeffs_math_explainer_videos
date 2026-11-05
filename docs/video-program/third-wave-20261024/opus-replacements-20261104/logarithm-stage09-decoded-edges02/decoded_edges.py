"""Stage09 decoded02 edges: the four swapped recordings 01_2, 07_6, 11_5, 12_2 (87do.63, root Mail 4619).

BronzeJaguar, 2026-11-05. From the repository root:

    python3 -I -B P/logarithm-stage09-decoded-edges02/decoded_edges.py FRESH_OUT --fmn-sound SOUND_RS

SOUND_RS is franken_manim's crates/fmn-output/src/sound.rs at a399d626 (sha256 7bb983c0...). FRESH_OUT must not exist.
Read-only on everything. In this order:

  G  the guard receipt first: exit 0, admitted, not preflight only, inputs hash unchanged at the end and equal to
     decode-inputs02.json (439 files); its rows for the four sources, selection02, the helper and the binding are still at
     their hashes; the command is the installed helper's decode of selection02 into decoded02
  S  selection02 against selection01 and edge-take-review01: exactly the four cues changed, to the four recommended
     recordings; the installed helper's load_selection accepts it
  R  decoded02's receipt: for the four, source path and hash are the inventoried recording, the WAV hash is the file now,
     mono PCM16 24 kHz, samples = ceil(n/2) = edge-take-review01's predicted length; the ffmpeg version and arguments
     equal decoded01's; the other 76 rows equal decoded01's apart from the WAV path (receipt rows only, files not re-read)
  E  the four decoded files' edges: first/last samples, 10 ms and 1 ms levels, LavenderPike's three edge flags
     (production-join-inspection01: her 10 ms levels at 24 kHz), and the fmn voice bus's first and last 48 kHz samples
     (timing-corrections01's mirror of sound.rs); against edge-take-review01's estimate and decoded01's selected takes
  C  sample continuity, decoded file against its 48 kHz source: in every 0.5 s window, the lag (-2..+2 decoded samples)
     of best normalized correlation with every second source sample, and the correlation; 10 ms level differences;
     longest run of exact zeros; peaks; and that root's prepared02 narration WAV of the cue holds the decoded PCM
  X  root's own end measurements (selection02-decision.json, prepared02-actual-endings01.json), read after E and C and
     recomputed from the files they name

Writes only FRESH_OUT: edges.tsv, continuity.tsv, windows.tsv, decoded-edges.json, stdout.txt.
Nothing is decoded, prepared, assembled, rendered or heard. Levels and correlations are measurements, not listening.
"""

import array
import csv
import hashlib
import importlib.util
import io
import json
import math
import os
import sys
import wave
from operator import mul
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
O = f"{EL}/production-audio-stage09-01"
GUARD = "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage09-audio-decode02"
HELPER = f"{P}/logarithm-stage09-audio-preparation02/prepare_audio.py"
BINDING = f"{P}/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json"
SELECTION01, SELECTION02 = f"{O}/selection01.json", f"{O}/selection02.json"
DECODE01, DECODE02 = f"{O}/decoded01/decode-receipt.json", f"{O}/decoded02/decode-receipt.json"
DECODE_INPUTS02 = f"{O}/decode-inputs02.json"
DECISION, ENDINGS = f"{O}/selection02-decision.json", f"{O}/prepared02-actual-endings01.json"
NARR02 = f"{O}/prepared02/narration"
ETR = f"{P}/logarithm-stage09-edge-take-review01"
TC_DIR = f"{P}/logarithm-stage09-timing-corrections01"
JOINS_DIR = f"{P}/logarithm-stage09-production-join-inspection01"
SEALS = {
    ETR: "95030197c86e638b9bfaf20e2b1cf34976731709d99facc6a87cf5bac136c512",
    TC_DIR: "26279b53a90d46988422b7d5f0b489b8d3481b9bdabc104281284c9062a2336d",
    JOINS_DIR: "e05ea847fe1439d9108ff6c42f4b2f6225bd253a4d604d1d6e97715d6f962bc9",
}
PINS = {
    f"{GUARD}/resource-receipt.json": "5426d0f52e83c39b34105fa426b3efc543bd34f4551f1553dcd3f74159fdfd8c",
    f"{GUARD}/command.log": "f177a60276e1184673388f71a4ef7e982f8a01538f3dc248c03c7a14621c7801",
    DECODE_INPUTS02: "6a4d4caa1a659bfd5c5d3d626a0d998c3916de69abd7b0fb43d8a2a80ec0c9f5",
    SELECTION01: "a5c4dfa119806d8ad46a1539c40ffd4de372dcd3541b5b62d6b673eead53a4bc",
    SELECTION02: "871e7d0b4ea3c47edd8d059804fb94178bd2fd05c1992f9adb99dff1b82fe2b6",
    DECODE01: "4e442a6c27f192a995cdb6e7e2b5a8fb46ac676b6fadc2e09ca32cce06a51cc2",
    DECODE02: "f9f49524a29496ede6425691bfa7485553d1304d403c8ba95ab0cff8ffc6f61b",
    DECISION: "9d3441b16cc56e9ad9c056dac59e5fa258ed587831e7382e0156a060a6522770",
    ENDINGS: "9e5bd7b2fb05adf729f2aa91a919950d3b5b99c3f79eb36d64a2cd7ad59adc5a",
    HELPER: "54df274a468be6122251efa7e4ccd7e11477fc8a93522e66de484ffb9be4eb0d",
    BINDING: "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803",
}
CUES = ["01_2", "07_6", "11_5", "12_2"]
WIN = 12000  # 0.5 s of 24 kHz samples per correlation window
LAGS = range(-2, 3)
SPEECH_WIN_DB = -50.0  # windows at or above this RMS are scored for correlation


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


def read_wav(path):
    with wave.open(path, "rb") as w:
        params = (w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getcomptype())
        x = array.array("h", w.readframes(w.getnframes()))
    need(sys.byteorder == "little", "little-endian host required")
    return params, x


def longest_zero_run(x):
    best = cur = 0
    for v in x:
        cur = cur + 1 if v == 0 else 0
        best = max(best, cur)
    return best


def tsv(rows, cols):
    buf = io.StringIO()
    w = csv.writer(buf, delimiter="\t", lineterminator="\n")
    w.writerow(cols)
    for r in rows:
        w.writerow([json.dumps(r.get(c)) if isinstance(r.get(c), (list, dict, bool)) or r.get(c) is None else r.get(c) for c in cols])
    return buf.getvalue()


def main(argv):
    if len(argv) != 4 or argv[2] != "--fmn-sound":
        raise SystemExit(__doc__)
    out, sound_rs = Path(argv[1]), Path(argv[3])
    need(not out.exists(), f"{out} exists")
    need(Path(".git").exists() and Path(P).is_dir(), "run from the repository root")
    for folder, want in SEALS.items():
        need(sha(f"{folder}/SHA256SUMS") == want, f"{folder} seal changed")
        for line in open(f"{folder}/SHA256SUMS", encoding="utf8").read().splitlines():
            h, rel = line.split("  ", 1)
            need(sha(f"{folder}/{rel}") == h, f"{folder}/{rel} changed")
    for path, want in PINS.items():
        need(sha(path) == want, f"{path} changed (sha256 {sha(path)})")
    tc = load_module("tc", f"{TC_DIR}/timing_corrections.py")
    text_rs = sound_rs.read_text(encoding="utf8")
    need(sha(sound_rs) == tc.FMN_SOUND_SHA and all(t in text_rs for t in tc.FMN_NEEDLES), "SOUND_RS is not fmn-output sound.rs at a399d626")
    need(not any(w in text_rs.lower() for w in ("fade", "envelope", "taper", "declick")), "sound.rs names an edge treatment")
    lj = load_module("lj", f"{JOINS_DIR}/inspect_production_joins.py")
    pa = load_module("pa", HELPER)
    etr = json.loads(Path(f"{ETR}/outputs/review.json").read_text())
    out.mkdir(parents=True)
    lines = []

    def say(s):
        lines.append(s)
        print(s, flush=True)

    say(f"pins: {len(SEALS)} seals verified file by file; {len(PINS)} inputs at their hashes; fmn sound.rs {tc.FMN_SOUND_SHA[:8]}")
    result = {"claim": "Measurements of the four swapped decoded recordings and the guard and receipts that bind them; nothing was "
                       "decoded, prepared, heard or changed.", "pins": {**{k + "/SHA256SUMS": v for k, v in SEALS.items()}, **PINS}}

    # ---------------------------------------------------------------- G. the guard receipt
    rc = json.loads(Path(f"{GUARD}/resource-receipt.json").read_text())
    cmd = rc["command"]
    want_cmd = ["python3", "-I", "-B", HELPER, "--binding", BINDING, "decode", "--selection", SELECTION02, "--out", f"{O}/decoded02"]
    files = json.loads(Path(DECODE_INPUTS02).read_text())["files"]
    log = Path(f"{GUARD}/command.log").read_text()
    g = {"exit": rc["exit"], "admitted": rc["admitted"], "preflight_only": rc["preflight_only"], "phase": rc["phase"],
         "inputs_sha256": rc["inputs_sha256"], "final_inputs_sha256": rc["final_inputs_sha256"], "inputs_files": len(files),
         "command_is_helper_decode_of_selection02": cmd == want_cmd, "started": rc["started"], "finished": rc["finished"],
         "log_says": log.strip().split(";")[0].replace(f"{O}/", "O/")}
    need(rc["exit"] == 0 and rc["admitted"] is True and rc["preflight_only"] is False, "the guard did not finish an admitted run")
    need(rc["inputs_sha256"] == rc["final_inputs_sha256"] == PINS[DECODE_INPUTS02], "the guard's inputs hash moved or names another list")
    need(cmd == want_cmd, f"the guard ran {cmd}")
    need(log.startswith("decoded 80 complete recordings into "), "the command log does not report 80 decoded recordings")
    root = os.getcwd() + os.sep  # the list names absolute paths; compare repository-relative ones, write none
    by_path = {(f["path"][len(root):] if f["path"].startswith(root) else f["path"]): f["sha256"] for f in files}
    need(len(by_path) == len(files) == 439, "decode-inputs02 does not list 439 distinct files")
    g["inputs_outside_repository"] = sum(1 for p in by_path if p.startswith(os.sep))
    sel01 = json.loads(Path(SELECTION01).read_text())
    sel02 = json.loads(Path(SELECTION02).read_text())
    recommended = {c: v["alternative"] for c, v in etr["verdicts"].items()}
    four = {cue: sel02["takes"][cue] for cue in CUES}
    for rel in [*four.values(), SELECTION02, HELPER, BINDING]:
        need(by_path.get(rel) is not None and by_path[rel] == sha(rel), f"{rel}: not in decode-inputs02 at its current hash")
    g["rows_rechecked"] = len(four) + 3
    result["guard"] = g
    say(f"G guard {PINS[f'{GUARD}/resource-receipt.json'][:8]}: exit 0, admitted, not preflight only, phase {rc['phase']}; inputs "
        f"{rc['inputs_sha256'][:8]} = final {rc['final_inputs_sha256'][:8]} = decode-inputs02.json ({len(files)} files); its rows for "
        f"the four sources, selection02, helper and binding are at their current hashes; command = the helper's decode of "
        f"selection02 into decoded02; log: '{g['log_says']}'; {rc['started']} to {rc['finished']}")

    # ---------------------------------------------------------------- S. selection02
    changed = sorted(c for c in sel01["takes"] if sel01["takes"][c] != sel02["takes"].get(c))
    need(list(sel02["takes"]) == list(sel01["takes"]), "selection02 names other cues or another order")
    need(changed == CUES, f"selection02 changes {changed}")
    need(all(os.path.basename(four[c]) == recommended[c] for c in CUES), "selection02 is not edge-take-review01's four recordings")
    b = pa.load_binding(BINDING)
    _files, _questions, _recordings, inputs = pa.build_report(b)
    _sel, chosen = pa.load_selection(b, SELECTION02, inputs, None)
    need(all(chosen[c]["rel"] == four[c] for c in CUES), "the helper resolves selection02 to other recordings")
    result["selection02"] = {"changed_cues": changed, "takes": four, "by": sel02.get("by"), "helper_accepts": True}
    say(f"S selection02 {PINS[SELECTION02][:8]} (by {sel02.get('by')}): differs from selection01 in exactly {', '.join(changed)}, each to "
        f"edge-take-review01's recommended recording; the installed helper's load_selection accepts it")

    # ---------------------------------------------------------------- R. the decode receipt
    r1 = json.loads(Path(DECODE01).read_text())
    r2 = json.loads(Path(DECODE02).read_text())
    rows1 = {r["cue"]: r for r in r1["records"]}
    rows2 = {r["cue"]: r for r in r2["records"]}
    need(list(rows2) == list(rows1) and len(rows2) == 80, "decoded02's receipt does not hold the 80 cues in order")
    need(r2["ffmpeg"] == r1["ffmpeg"] and r2["conversion"] == r1["conversion"], "decoded02 used another ffmpeg or conversion")
    need(r2["selection_sha256"] == PINS[SELECTION02] and r2["binding"] == r1["binding"] and r2["production"] is True, "decoded02's receipt binds another selection or binding")
    same76 = []
    for cue in rows1:
        if cue in CUES:
            continue
        a, z = dict(rows1[cue]), dict(rows2[cue])
        need(a.pop("wav") == f"{O}/decoded01/decoded/{cue}.wav" and z.pop("wav") == f"{O}/decoded02/decoded/{cue}.wav", f"{cue}: WAV path")
        need(a == z, f"{cue}: decoded02's receipt row differs from decoded01's")
        same76.append(cue)
    rrows = {}
    for cue in CUES:
        row, t = rows2[cue], chosen[cue]
        predicted = next(x for x in etr["takes"].values() if x["file"] == os.path.basename(t["rel"]) and x["cue"] == cue)
        params, d = read_wav(row["wav"])
        need(row["wav"] == f"{O}/decoded02/decoded/{cue}.wav" and sha(row["wav"]) == row["wav_sha256"], f"{cue}: the decoded file is not the receipt's")
        need(row["source"] == t["rel"] and row["source_sha256"] == t["sha256"] == sha(t["rel"]) and row["source_frames"] == t["frames"]
             and row["source_rate"] == 48000 and row["text"] == inputs["script"][cue], f"{cue}: the receipt's source is not the selected recording")
        need(params == (1, 2, 24000, "NONE") and len(d) == row["samples"] == pa.expected_frames(t)[0] == predicted["frames_24k"],
             f"{cue}: {params}, {len(d)} samples; receipt {row['samples']}, predicted {predicted['frames_24k']}")
        rrows[cue] = {"wav": row["wav"], "wav_sha256": row["wav_sha256"], "samples": len(d), "source": row["source"],
                      "source_frames": row["source_frames"], "predicted_samples": predicted["frames_24k"],
                      "decoded01_samples": rows1[cue]["samples"], "seconds": round(len(d) / 24000, 3),
                      "seconds_change": round((len(d) - rows1[cue]["samples"]) / 24000, 3)}
    result["receipt"] = {"same_rows_76": len(same76), "four": rrows}
    say(f"R decoded02 receipt {PINS[DECODE02][:8]}: ffmpeg and conversion equal decoded01's; selection hash = selection02; the 76 other rows "
        f"equal decoded01's apart from the WAV path ({len(same76)}); the four: " + "; ".join(
            f"{c} {os.path.basename(v['source'])} -> {v['samples']} samples = ceil({v['source_frames']}/2) = predicted, "
            f"{v['seconds_change']:+} s against decoded01" for c, v in rrows.items()))

    # ---------------------------------------------------------------- E. edges
    edges = []
    for cue in CUES:
        _p, d = read_wav(rows2[cue]["wav"])
        _p1, d1 = read_wav(rows1[cue]["wav"])
        need(sha(rows1[cue]["wav"]) == rows1[cue]["wav_sha256"], f"{cue}: decoded01 file changed")
        m, m1 = len(d), len(d1)
        db = lj.levels(d)  # LavenderPike's 10 ms frames at 24 kHz
        db1 = lj.levels(d1)
        flags = (["sound_in_first_10ms"] if db[0] > lj.QUIET_DB else []) + (["sound_in_last_10ms"] if db[-1] > lj.QUIET_DB else []) + \
            (["rise_in_last_10ms"] if db[-1] > lj.QUIET_DB and db[-1] > db[-2] + 15 else [])
        flags1 = (["sound_in_first_10ms"] if db1[0] > lj.QUIET_DB else []) + (["sound_in_last_10ms"] if db1[-1] > lj.QUIET_DB else []) + \
            (["rise_in_last_10ms"] if db1[-1] > lj.QUIET_DB and db1[-1] > db1[-2] + 15 else [])
        em = tc.edge_measures(d, 24000)
        bus = tc.fmn_resample_at(d, 24000, 48000, [0, 1, 2, 2 * m - 3, 2 * m - 2, 2 * m - 1])
        bus1 = tc.fmn_resample_at(d1, 24000, 48000, [0, 2 * m1 - 1])
        alt = next(x for x in etr["takes"].values() if x["file"] == os.path.basename(four[cue]) and x["cue"] == cue)
        was = next(x for x in etr["takes"].values() if x["cue"] == cue and x["selected"])
        end_abs = abs(bus[2 * m - 1] * 32768)
        row = {"cue": cue, "file": os.path.basename(four[cue]), "samples": m, "first_samples": list(d[:3]), "last_samples": list(d[-3:]),
               "first_10ms_db": db[0], "last_two_10ms_db": db[-2:], "first_1ms_db": em["first_1ms_db"], "last_1ms_db": em["last_1ms_db"],
               "lead_60_s": em["lead_60_s"], "tail_60_s": em["tail_60_s"], "tail_40_s": em["tail_40_s"],
               "last_10ms_peak_abs": max(abs(v) for v in d[-240:]), "lavender_edge_flags": flags,
               "bus_first_48k": [round(bus[k] * 32768, 2) for k in (0, 1, 2)],
               "bus_last_48k": [round(bus[k] * 32768, 2) for k in (2 * m - 3, 2 * m - 2, 2 * m - 1)],
               "bus_start_step_dbfs": tc.dbfs(bus[0] * 32768), "bus_end_step_dbfs": tc.dbfs(bus[2 * m - 1] * 32768),
               "predicted_bus_end_estimate_dbfs": alt["edges"]["bus_end_estimate_dbfs"], "predicted_bus_end_estimate_abs": alt["edges"]["bus_end_estimate_abs"],
               "within_prediction": end_abs <= alt["edges"]["bus_end_estimate_abs"],
               "decoded01": {"file": was["file"], "last_samples": list(d1[-3:]), "last_two_10ms_db": db1[-2:], "lavender_edge_flags": flags1,
                             "bus_end_step_dbfs": tc.dbfs(bus1[2 * m1 - 1] * 32768), "bus_start_step_dbfs": tc.dbfs(bus1[0] * 32768),
                             "edge_take_review_bus_end_dbfs": was["edges"].get("bus_end_dbfs")}}
        need(row["decoded01"]["bus_end_step_dbfs"] == row["decoded01"]["edge_take_review_bus_end_dbfs"], f"{cue}: decoded01 bus end differs from edge-take-review01")
        edges.append(row)
        say(f"E {cue} {row['file']}: last samples {row['last_samples']}, last two 10 ms {row['last_two_10ms_db']} dB, last 1 ms {row['last_1ms_db']} dB, "
            f"-60 tail {row['tail_60_s']} s; Lavender flags {flags or 'none'}; bus ends {row['bus_last_48k']} ({row['bus_end_step_dbfs']} dBFS; "
            f"predicted at most {row['predicted_bus_end_estimate_dbfs']}: {row['within_prediction']}), starts {row['bus_first_48k']} "
            f"({row['bus_start_step_dbfs']} dBFS); decoded01 {was['file']}: flags {flags1}, bus end {row['decoded01']['bus_end_step_dbfs']} dBFS")
    result["edges"] = edges

    # ---------------------------------------------------------------- C. sample continuity
    cont, windows = [], []
    for cue in CUES:
        _p, d = read_wav(rows2[cue]["wav"])
        _q, x = read_wav(four[cue])
        need(_q[:3] == (1, 2, 48000), f"{cue}: source format")
        lag_set, rmin, scored = {}, None, 0
        for a in range(0, len(d) - WIN + 1, WIN):
            seg = d[a:a + WIN]
            e = sum(map(mul, seg, seg))
            rms_db = 10 * math.log10(e / WIN / 32768 ** 2) if e > 0 else -120.0
            if rms_db < SPEECH_WIN_DB:
                continue
            rs = {}
            for lag in LAGS:
                lo = 2 * (a + lag)
                if lo < 0 or lo + 2 * WIN > len(x):
                    continue
                xs = x[lo:lo + 2 * WIN:2]
                den = math.sqrt(e * sum(map(mul, xs, xs)))
                rs[lag] = sum(map(mul, seg, xs)) / den if den > 0 else 0.0
            best = max(rs, key=lambda k: rs[k])
            lag_set[best] = lag_set.get(best, 0) + 1
            scored += 1
            r0 = rs.get(0)
            rmin = r0 if rmin is None or (r0 is not None and r0 < rmin) else rmin
            windows.append({"cue": cue, "start_s": round(a / 24000, 2), "rms_db": round(rms_db, 1), "best_lag": best,
                            "r_lag0": round(r0, 5) if r0 is not None else None, "r_best": round(rs[best], 5)})
        dbd, dbx = lj.levels(d), [round(tc.db_raw(x, k * 480, 480), 2) for k in range(len(x) // 480)]
        n = min(len(dbd), len(dbx))
        speech = [abs(dbd[k] - dbx[k]) for k in range(n) if dbx[k] >= lj.BODY_DB]
        worst_k = max((k for k in range(n) if dbx[k] >= lj.BODY_DB), key=lambda k: (abs(dbd[k] - dbx[k]), -k))
        quiet = sorted(dbd[k] - dbx[k] for k in range(n) if dbx[k] <= lj.QUIET_DB)
        # where the decode is quietest relative to the source: content above 12 kHz, which 24 kHz cannot hold, is the
        # expected reason; the share of the 10 ms frame's source energy above 12 kHz is estimated by the first difference
        seg = x[worst_k * 480:(worst_k + 1) * 480]
        hf = sum((seg[i + 1] - seg[i]) ** 2 for i in range(len(seg) - 1)) / max(1, sum(v * v for v in seg))
        zc = sum(1 for p_, q_ in zip(seg, seg[1:]) if (p_ < 0) != (q_ < 0)) / (len(seg) - 1)
        narr = f"{NARR02}/{cue}.wav"
        narr_equal = None
        if os.path.isfile(narr):
            _n, y = read_wav(narr)
            narr_equal = _n[:3] == (1, 2, 24000) and y.tobytes() == d.tobytes()
        row = {"cue": cue, "windows_scored": scored, "best_lags": {str(k): v for k, v in sorted(lag_set.items())},
               "min_r_lag0": round(rmin, 5) if rmin is not None else None,
               "frames_compared": n, "speech_frames": len(speech), "max_abs_level_diff_speech_db": round(max(speech), 2),
               "max_diff_at_s": round(worst_k / 100, 2), "max_diff_source_db": dbx[worst_k], "max_diff_decoded_db": dbd[worst_k],
               "max_diff_source_zero_crossing_rate": round(zc, 3), "max_diff_source_first_difference_ratio": round(hf, 3),
               "median_level_diff_quiet_db": quiet[len(quiet) // 2] if quiet else None,
               "longest_zero_run_decoded": longest_zero_run(d), "longest_zero_run_source": longest_zero_run(x),
               "peak_decoded_dbfs": tc.dbfs(max(max(d), -min(d))), "peak_source_dbfs": tc.dbfs(max(max(x), -min(x))),
               "first_sample_vs_source": [d[0], x[0]], "last_sample_vs_source_last_two": [d[-1], list(x[-2:])],
               "prepared02_narration_pcm_equals_decoded": narr_equal}
        cont.append(row)
        say(f"C {cue}: {scored} speech windows of 0.5 s, best lag {row['best_lags']} (decoded samples), min r at lag 0 {row['min_r_lag0']}; "
            f"10 ms levels within {row['max_abs_level_diff_speech_db']} dB over {len(speech)} speech frames (largest at {row['max_diff_at_s']} s, "
            f"source {row['max_diff_source_db']} / decoded {row['max_diff_decoded_db']} dB, source zero-crossing rate "
            f"{row['max_diff_source_zero_crossing_rate']}), quiet frames median "
            f"{row['median_level_diff_quiet_db']:+} dB; longest zero run {row['longest_zero_run_decoded']} (source {row['longest_zero_run_source']}); "
            f"peak {row['peak_decoded_dbfs']} / {row['peak_source_dbfs']} dBFS; prepared02 narration PCM = decoded: {narr_equal}")
    result["continuity"] = cont

    # ---------------------------------------------------------------- X. root's measurements, recomputed
    dec = json.loads(Path(DECISION).read_text())["independent_root_end_measurements"]
    ends = json.loads(Path(ENDINGS).read_text())["rows"]
    xrows = []

    def repo_rel(p):  # root's files name absolute paths: read them repository-relative, write none
        need(p.startswith(root) or not p.startswith(os.sep), f"{p[-60:]}: outside the repository")
        return p[len(root):] if p.startswith(root) else p

    for m in dec:
        for tk in m["takes"]:
            rel = repo_rel(tk["file"])
            need(rel in inputs["takes"] and inputs["takes"][rel]["cue"] == m["cue"], f"{rel}: not an inventoried recording of {m['cue']}")
            _p, x = read_wav(rel)
            mine = {"sha256": sha(rel), "last_10ms_dbfs": round(tc.db_raw(x, len(x) - 480, 480), 2), "last_samples": list(x[-3:])}
            ok = mine["sha256"] == tk["sha256"] and abs(mine["last_10ms_dbfs"] - tk["last_10ms_dbfs"]) <= 0.05 and mine["last_samples"] == tk["last_samples"]
            xrows.append({"source": "selection02-decision", "cue": m["cue"], "file": rel, "root": {k: tk[k] for k in ("last_10ms_dbfs", "last_samples")},
                          "mine": mine, "agree": ok})
    for r in ends:
        for side in ("previous", "current"):
            v = r[side]
            path = repo_rel(v["path"])
            need(path == f"{O}/prepared0{1 if side == 'previous' else 2}/narration/{r['cue']}.wav", f"{path}: not the {side} narration WAV")
            _p, y = read_wav(path)
            h = _p[2] // 100
            seg = y[-h:]
            mine = {"sha256": sha(path), "frames": len(y), "last10ms_rms_dbfs": round(tc.db_raw(y, len(y) - h, h), 2),
                    "last_sample": y[-1], "last10ms_peak": max(abs(s) for s in seg)}
            ok = (mine["sha256"] == v["sha256"] and mine["frames"] == v["frames"] and abs(mine["last10ms_rms_dbfs"] - v["last10ms_rms_dbfs"]) <= 0.05
                  and mine["last_sample"] == v["last_sample"] and mine["last10ms_peak"] == v["last10ms_peak"])
            xrows.append({"source": "prepared02-actual-endings01", "cue": r["cue"], "side": side, "path": path,
                          "root": {k: v[k] for k in ("frames", "last10ms_rms_dbfs", "last_sample", "last10ms_peak")}, "mine": mine, "agree": ok})
    result["root_measurements"] = xrows
    say(f"X root's measurements recomputed from the files they name: {sum(r['agree'] for r in xrows)} of {len(xrows)} agree "
        "(hash, frames, last-10 ms RMS within 0.05 dB, last samples, last-10 ms peak)"
        + "".join(f"; DIFFERS {r['source']} {r['cue']} {r.get('side', r.get('file'))}: root {r['root']} mine {r['mine']}" for r in xrows if not r["agree"]))

    # ---------------------------------------------------------------- verdict
    verdict = {}
    for e, c in zip(edges, cont):
        conds = {"no_lavender_edge_flag": not e["lavender_edge_flags"], "bus_end_within_prediction": e["within_prediction"],
                 "bus_end_below_minus_60_dbfs": e["bus_end_step_dbfs"] <= -60, "bus_start_below_minus_60_dbfs": e["bus_start_step_dbfs"] <= -60,
                 "length_as_predicted": rrows[e["cue"]]["samples"] == rrows[e["cue"]]["predicted_samples"],
                 # continuity: one alignment in every speech window and near-identical waveforms there; 10 ms levels are
                 # reported, not tested (24 kHz holds nothing above 12 kHz, so fricative frames read quieter)
                 "lag_0_in_every_speech_window": c["best_lags"] == {"0": c["windows_scored"]},
                 "r_at_lag_0_at_least_0_99": c["min_r_lag0"] is not None and c["min_r_lag0"] >= 0.99,
                 "no_new_zero_run": c["longest_zero_run_decoded"] <= max(8, c["longest_zero_run_source"]),
                 "prepared02_narration_holds_decoded_pcm": c["prepared02_narration_pcm_equals_decoded"] is True}
        verdict[e["cue"]] = {"pass": all(conds.values()), "conditions": conds}
        say(f"VERDICT {e['cue']}: {'PASS' if all(conds.values()) else 'FAIL ' + str([k for k, v in conds.items() if not v])}")
    result["verdict"] = verdict

    ecols = ["cue", "file", "samples", "first_samples", "last_samples", "first_10ms_db", "last_two_10ms_db", "first_1ms_db", "last_1ms_db",
             "lead_60_s", "tail_60_s", "tail_40_s", "last_10ms_peak_abs", "lavender_edge_flags", "bus_first_48k", "bus_last_48k",
             "bus_start_step_dbfs", "bus_end_step_dbfs", "predicted_bus_end_estimate_dbfs", "within_prediction", "decoded01"]
    (out / "edges.tsv").write_text(tsv(edges, ecols))
    (out / "continuity.tsv").write_text(tsv(cont, list(cont[0])))
    (out / "windows.tsv").write_text(tsv(windows, list(windows[0])))
    (out / "decoded-edges.json").write_text(json.dumps(result, indent=1, ensure_ascii=False) + "\n")
    (out / "stdout.txt").write_text("\n".join(lines) + "\n")
    print("DECODED EDGES: DONE")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
