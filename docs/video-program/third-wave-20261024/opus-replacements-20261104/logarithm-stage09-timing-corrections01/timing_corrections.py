"""Stage09 timing corrections: 04_2 b306, 07_6 s2-s5, and the cue-edge reconciliation (87do.63, root Mail 4383).

BronzeJaguar, 2026-11-05. From the repository root:

    python3 -I -B P/logarithm-stage09-timing-corrections01/timing_corrections.py FRESH_OUT --fmn-sound SOUND_RS

SOUND_RS is franken_manim's crates/fmn-output/src/sound.rs at a399d626 (the revision the stage09 crate pins; the cargo
checkout cargo compiled has the same bytes, sha256 7bb983c0...). FRESH_OUT must not exist.

Read-only on every input. It writes only FRESH_OUT:
  points.tsv / points.json   the five points (and the alternative take's five): script text, the recognizer's own
                             words and times, 10 ms leave/body (timing-inspection01's measure, unchanged), every 1 ms
                             window and sample of the pause, the first 1 ms window at speech level, periodicity of the
                             lead-in and of the first 100 ms of speech, the checks and the proposal
  edges-recordings.tsv       all 162 selectable recordings (48 kHz): production-choices01's own screen (lead/tail at
                             -40 dBFS) beside -60 dBFS tails, the last/first 1 ms and the edge samples
  edges-production.tsv       the 80 production narration WAVs (24 kHz) with the same edge measures, and the first/last
                             48 kHz samples of fmn's resampler (mirrored from sound.rs) that the voice bus would hold
  answers02.json             production-choices01's answers.json plus a `sample` answer for each point whose checks
                             all pass (five selected, four alternative)
  validation.json            the installed helper (54df274a), in process: answers accepted, the resolved points equal
                             root's prepared01 receipt and plan, exactly five points change, the assembler's dry run
                             (on a scratch copy of the prepared01 workspace) accepts the changed spans, and unit_spans
                             gives all 80 cues the same PCM with only 07_6's rows changed; the predicted rows and mark
  scratch/                   the dry run's spans and workspace copy (not for use)

Nothing is decoded, assembled, rendered or heard. Levels and periodicity are measurements, not listening.
"""

import array
import copy
import hashlib
import importlib.util
import json
import math
import os
import shutil
import sys
import wave
from operator import mul
from pathlib import Path

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-timing-corrections01"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
O = f"{EL}/production-audio-stage09-01"
PREP = f"{O}/prepared01"
NARR = f"{PREP}/narration"
SELECTION = f"{O}/selection01.json"
BINDING = f"{P}/logarithm-stage09-audio-preparation01/bindings/logarithm-stage09-oliver.json"
HELPER = f"{P}/logarithm-stage09-audio-preparation02/prepare_audio.py"
ANSWERS01 = f"{P}/logarithm-stage09-production-choices01/inputs/answers.json"
INSPECT = f"{P}/logarithm-stage09-timing-inspection01/inspect_timing.py"
CHOOSE = f"{P}/logarithm-stage09-production-choices01/choose_takes.py"
SEALS = {
    f"{P}/logarithm-stage09-production-join-inspection01": "e05ea847fe1439d9108ff6c42f4b2f6225bd253a4d604d1d6e97715d6f962bc9",
    f"{P}/logarithm-stage09-timing-inspection01": "8c18a24363d745e65be1a3ffec89b1d1bfc8d4c2a77185c85b0a2fcd805490bf",
    f"{P}/logarithm-stage09-production-choices01": "357613e5a21a23ce525a99d46da33d74cc8578eb23003d2c9dcfe2f79092f8cf",
}
PINS = {
    HELPER: "54df274a468be6122251efa7e4ccd7e11477fc8a93522e66de484ffb9be4eb0d",
    BINDING: "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803",
    ANSWERS01: "8f9a0cd0fe196a0a0586b79b9f1bd68b4f1c9593d32f20ef03e4d13b372c269d",
    INSPECT: "5d4661b0e58c800da548c0b3710a6983e2465703cf3e7fabf89b4f91d32425e3",
    CHOOSE: "524277ebaa091583094fb6af215b7686a1c106a95af3a58e0ef227fe54e40374",
    SELECTION: "a5c4dfa119806d8ad46a1539c40ffd4de372dcd3541b5b62d6b673eead53a4bc",
    f"{PREP}/prepare-receipt.json": "0b30ba1fa1c3dcacd9411decdc33aa86f995de1a1e26110377f225cf2dcb1637",
    f"{PREP}/spans.json": "9bbfcd065dd75025eccecfda94a195f50f95b905a5cb5602e6e71c0e835dec4a",
    f"{PREP}/phrase-marks-plan.json": "1302b36a3ca9f17b60976dda6f9cd739b7e5944b97745ba99aa0821a55d60fbf",
    f"{NARR}/report.json": "77dd97b08cd34498b60ef4d06ac3539f34e074ebadfd97b05733d4d05585b000",
    f"{NARR}/phrase-timings.json": "de62cf1c2c22badf60f11d181950a0567163c7c54b7241b9be3df894842709c6",
    f"{NARR}/assembly.json": "c79dbceeb0fd5ea28d3fb0526a92bf85fc7f8001f03701c2da106dc982668b9b",
    f"{O}/decoded01/decode-receipt.json": "4e442a6c27f192a995cdb6e7e2b5a8fb46ac676b6fadc2e09ca32cce06a51cc2",
}
FMN_SOUND_SHA = "7bb983c03c3759b73c5963cb539dfcc78e5ba059eb2ecc90e22c6d9dbccd608c"
FMN_NEEDLES = [  # the mixer lines the mirror and the "no edge treatment" statement rest on
    "let output_frames = numerator.div_ceil(u128::from(input_rate));",
    "for delta in (-RESAMPLER_RADIUS + 1)..=RESAMPLER_RADIUS {",
    "const RESAMPLER_RADIUS: i64 = 32;",
    "let window = 0.42",
    "let weight = cutoff * sinc * window;",
    "output[output_at + channel] = sums[channel] / weight_sum;",
    "//! Placement converts `(frame, fps)` to the nearest sample by exact integer",
    "//! against sample zero; positive ends extend the mix. Final samples clamp to",
]
BY = "BronzeJaguar, provisional for root (Mail 4383)"
POINTS = [("anchor", "04_2", "b306"), ("onset", "07_6", "s2"), ("onset", "07_6", "s3"), ("onset", "07_6", "s4"), ("onset", "07_6", "s5")]
EXPECT = {  # Lavender's production-join-inspection01 leave values (selected) and timing-inspection01's (alternative)
    "anchor:04_2:b306:04_2-download01.wav": 530160, "onset:07_6:s2:07_6-download01.wav": 238080,
    "onset:07_6:s3:07_6-download01.wav": 382080, "onset:07_6:s4:07_6-download01.wav": 499680,
    "onset:07_6:s5:07_6-download01.wav": 634800,
    "anchor:04_2:b306:04_2-download02.wav": 527760, "onset:07_6:s2:07_6-download02.wav": 240720,
    "onset:07_6:s3:07_6-download02.wav": 376560, "onset:07_6:s4:07_6-download02.wav": 492240,
    "onset:07_6:s5:07_6-download02.wav": 625920,
}
EDGE_CUES = ["07_6", "11_5", "01_2", "12_2", "04_3", "00_5", "09_4"]  # Lavender's F3
QUIET, BODY, VOICED_R = -60.0, -40.0, 0.6


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


def read_pcm(path):
    with wave.open(path, "rb") as w:
        need((w.getnchannels(), w.getsampwidth()) == (1, 2), f"{path}: not mono PCM16")
        rate, n = w.getframerate(), w.getnframes()
        x = array.array("h", w.readframes(n))
    need(sys.byteorder == "little", "little-endian host required")
    return x, rate


def db_raw(x, a, n):
    """RMS level in dBFS, unrounded (thresholds compare this, as production-choices01's screen does)."""
    seg = x[max(0, a):a + n]
    if not seg:
        return None
    e = sum(map(mul, seg, seg)) / len(seg)
    return 10 * math.log10(e / 32768 ** 2) if e > 0 else -120.0


def db(x, a, n):
    v = db_raw(x, a, n)
    return None if v is None else round(v, 1)


def dbfs(v):
    return round(20 * math.log10(abs(v) / 32768), 1) if v else -120.0


def periodicity(x, rate, start, win_s=0.03):
    """Largest normalized autocorrelation for 70-300 Hz in a window: voiced speech is periodic, breath and noise are not."""
    n = round(win_s * rate)
    seg = x[start:start + n]
    if len(seg) < n:
        return None
    m = sum(seg) / n
    s = array.array("d", (v - m for v in seg))
    best, lag_at = 0.0, 0
    for lag in range(rate // 300, rate // 70 + 1):
        a, b = s[:n - lag], s[lag:]
        den = math.sqrt(sum(map(mul, a, a)) * sum(map(mul, b, b)))
        r = sum(map(mul, a, b)) / den if den > 0 else 0.0
        if r > best:
            best, lag_at = r, lag
    zc = sum(1 for p, q in zip(seg, seg[1:]) if (p < 0) != (q < 0)) / (n - 1)
    return {"r": round(best, 3), "f0_hz": round(rate / lag_at, 1) if lag_at else None, "zcr": round(zc, 3)}


def recognizer_context(pa, result, asr_start, word):
    obs = pa.observed_words(result)
    j = next(k for k, o in enumerate(obs) if abs(o["start"] - asr_start) < 1e-5 and o["w"].lower() == word.lower())
    return [{"word": o["w"], "start_s": round(o["start"], 3), "end_s": round(o["end"], 3), "dur_s": round(o["end"] - o["start"], 3),
             "here": k == j} for k, o in enumerate(obs) if j - 2 <= k <= j + 2]


def analyse_point(x, rate, m, asr_start):
    """Sample-level evidence around one point. m is timing-inspection01's measure() row for the same file.

    The pause is measure()'s 10 ms quiet run; here every 1 ms window and every sample in it is read. A take's pause
    floor can sit near -60 dBFS (07_6 download02 idles at -62 to -75 dB), so 1 ms windows are not split at -60;
    instead the pause's loudest 1 ms window and sample are reported. `speech` is the first 1 ms window at or above
    -40 dBFS from 10 ms before leave, and the voicing measure reads 40 ms windows over the first 100 ms of speech."""
    hop, ms = rate // 100, rate // 1000
    a10, leave = round(m["pause_start_s"] * rate), round(m["leave_s"] * rate)
    pause_ms = [db(x, k, ms) for k in range(a10, leave - ms + 1, ms)]
    seg = x[a10:leave]
    peak = max(max(seg), -min(seg))
    k = leave - hop
    while db_raw(x, k, ms) < BODY:
        k += ms
    speech = k
    windows = [{"from_speech_ms": 10 * j, "db_10ms": db(x, speech + j * hop, hop), **periodicity(x, rate, speech + j * hop, 0.04)}
               for j in range(11)]
    best = max(windows, key=lambda w: w["r"])
    # the sound between leave and speech level: periodic (a soft voiced onset) or not (breath or aspiration)?
    lead_in = [{"at_s": round((leave + j * hop) / rate, 3), "db_10ms": db(x, leave + j * hop, hop), **periodicity(x, rate, leave + j * hop, 0.03)}
               for j in range(max(0, (speech - leave) // hop))]
    asr = round(asr_start * rate)
    return {
        "lead_in_10ms": lead_in,
        "pause_s": [round(a10 / rate, 3), round(leave / rate, 3)], "pause_len_s": round((leave - a10) / rate, 3),
        "pause_1ms_median_db": sorted(pause_ms)[len(pause_ms) // 2], "pause_1ms_max_db": max(pause_ms),
        "pause_peak_sample": peak, "pause_peak_dbfs": dbfs(peak),
        "leave_to_speech_1ms_db": [db(x, j, ms) for j in range(leave, speech + ms, ms)][:40],
        "speech_1ms_s": round(speech / rate, 3), "speech_minus_leave_s": round((speech - leave) / rate, 3),
        "voicing": windows, "voicing_best": {"from_speech_ms": best["from_speech_ms"], "r": best["r"], "f0_hz": best["f0_hz"]},
        "recognizer_minus_speech_s": round((asr - speech) / rate, 3), "db_10ms_at_recognizer": db(x, asr, hop),
    }


def edge_measures(x, rate):
    n, hop, ms = len(x), rate // 100, rate // 1000
    last = (n + hop - 1) // hop - 1  # a partial final frame counts, as in production-choices01's screen
    k = last
    while k >= 0 and db_raw(x, k * hop, hop) <= QUIET:
        k -= 1
    first = 0
    while first <= last and db_raw(x, first * hop, hop) <= QUIET:
        first += 1
    body = [j for j in range(last + 1) if db_raw(x, j * hop, hop) >= BODY]
    return {"frames": n, "rate": rate, "frames_mod_3840_48k": (n * 48000 // rate) % 3840,
            "lead_40_s": round(body[0] * 0.01, 2), "tail_40_s": round(n / rate - (body[-1] + 1) * 0.01, 2),
            "lead_60_s": round(first * 0.01, 2), "tail_60_s": round(n / rate - (k + 1) * 0.01, 2),
            "first_10ms_db": db(x, 0, hop), "last_10ms_db": db(x, last * hop, hop),
            "first_1ms_db": db(x, 0, ms), "last_1ms_db": db(x, n - ms, ms),
            "first_samples": list(x[:3]), "last_samples": list(x[-3:]),
            "start_step_dbfs": dbfs(x[0]), "end_step_dbfs": dbfs(x[-1])}


def fmn_resample_at(x, rate_in, rate_out, frames):
    """fmn-output sound.rs resample() at chosen output frames (mono), on samples normalized s16 -> v/32768."""
    n, cutoff, out = len(x), min(rate_out / rate_in, 1.0), {}
    for o in frames:
        num = o * rate_in
        base, frac = num // rate_out, (num % rate_out) / rate_out
        s = w = 0.0
        for delta in range(-31, 33):
            src = base + delta
            if not 0 <= src < n:
                continue
            dist = delta - frac
            ph = math.pi * dist / 32
            win = 0.42 + 0.5 * math.cos(ph) + 0.08 * math.cos(2 * ph)
            sx = cutoff * dist
            sinc = 1.0 if sx == 0 else math.sin(math.pi * sx) / (math.pi * sx)
            wt = cutoff * sinc * win
            s += x[src] / 32768 * wt
            w += wt
        out[o] = s / w
    return out


def tsv(rows, cols):
    return "\t".join(cols) + "\n" + "".join("\t".join(json.dumps(r.get(c)) if isinstance(r.get(c), (list, dict, bool)) or r.get(c) is None
                                                     else str(r.get(c)) for c in cols) + "\n" for r in rows)


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
    text = sound_rs.read_text(encoding="utf8")
    need(sha(sound_rs) == FMN_SOUND_SHA, "SOUND_RS is not fmn-output sound.rs at a399d626")
    need(all(t in text for t in FMN_NEEDLES), "a mirrored mixer line is missing")
    need(not any(w in text.lower() for w in ("fade", "envelope", "taper", "declick")), "sound.rs names an edge treatment")
    pa, it, ct = load_module("pa", HELPER), load_module("it", INSPECT), load_module("ct", CHOOSE)
    b = pa.load_binding(BINDING)
    files, questions, recordings, inputs = pa.build_report(b)
    script = inputs["script"]
    sel, chosen = pa.load_selection(b, SELECTION, inputs, None)
    selected = {cue: t["rel"] for cue, t in chosen.items()}
    by_file = {(e["cue"], e["file"]): e for e in recordings}
    recog = pa.recognition_for(inputs)
    out.mkdir(parents=True)
    lines = []

    def say(s):
        lines.append(s)
        print(s, flush=True)

    say(f"pins: 3 seals verified file by file; {len(PINS)} inputs at their hashes; fmn sound.rs {FMN_SOUND_SHA[:8]} with "
        f"{len(FMN_NEEDLES)} mirrored lines, no fade/envelope/taper/declick")

    # ---------------------------------------------------------------- A. the points
    points = []
    for kind, cue, tag in POINTS:
        takes = sorted((r for r, t in inputs["takes"].items() if t["cue"] == cue), key=lambda r: (r != selected[cue], r))
        for rel in takes:
            name = os.path.basename(rel)
            pid = f"{kind}:{cue}:{tag}:{name}"
            e = by_file[(cue, name)]
            row = next(r for r in (e["phrase_anchors"] if kind == "anchor" else e["sentences"]) if r.get("id") == pid)
            need(row["status"] == "supported", f"{pid}: not a supported recognizer point")
            is_sel = rel == selected[cue]
            src_x, src_rate = read_pcm(rel)
            m48 = it.measure(it.Wav(rel), row["asr_start_s"], row["asr_end_s"])
            if is_sel:
                x, rate = read_pcm(f"{NARR}/{cue}.wav")
                m = it.measure(it.Wav(f"{NARR}/{cue}.wav"), row["asr_start_s"], row["asr_end_s"])
                need(m["leave_24k"] == m48["leave_24k"] and m["body_24k"] == m48["body_24k"], f"{pid}: 24 kHz and 48 kHz leave differ")
            else:
                x, rate, m = src_x, src_rate, m48
            need(m["leave_24k"] == EXPECT[pid], f"{pid}: leave {m['leave_24k']}, expected {EXPECT[pid]}")
            s = analyse_point(x, rate, m, row["asr_start_s"])
            ctx = recognizer_context(pa, recog[rel]["result"], row["asr_start_s"], row["word"])
            byte = row["byte"] if kind == "anchor" else len(script[cue][:row["char"]].encode())
            before = script[cue].encode()[max(0, byte - 40):byte].decode(errors="replace")
            after = script[cue].encode()[byte:byte + 40].decode(errors="replace")
            here = next(c for c in ctx if c["here"])
            prev = ctx[ctx.index(here) - 1]
            late = row["asr_start_s"] > m["leave_s"]
            checks = {
                "script_opens_here": (before.rstrip().endswith((".", "?", "!")) if kind == "onset" else before.rstrip().endswith(",")),
                "recognizer_abuts_previous_word": bool(row.get("abuts_previous_word")),
                "recognizer_off_leave_by_50ms_or_more": abs(row["asr_start_s"] - m["leave_s"]) >= 0.05 - 1e-9,
                "pause_100ms_every_1ms_below_minus_50db": s["pause_len_s"] >= 0.10 and s["pause_1ms_max_db"] <= -50,
                "speech_level_within_120ms_of_leave": 0 <= s["speech_minus_leave_s"] <= 0.12,
                "voiced_in_first_100ms_of_speech": s["voicing_best"]["r"] >= VOICED_R,
                # late: voiced speech-level sound precedes the recognizer's word; early: the recognizer's onset sits in quiet
                "recognizer_side": (s["recognizer_minus_speech_s"] >= 0.03 if late else s["db_10ms_at_recognizer"] <= -55),
            }
            points.append({"id": pid, "cue": cue, "file": name, "selected": is_sel, "kind": kind, "word": row["word"],
                           "phrase": row.get("phrase"), "script_before": before, "script_after": after, "byte": byte,
                           "recognizer": {"onset_s": row["asr_start_s"], "sample_24k": row["sample_24k"], "context": ctx,
                                          "previous_word_dur_s": prev["dur_s"], "word_dur_s": here["dur_s"]},
                           "energy_10ms": {k: m.get(k) for k in ("status", "class", "pause_start_s", "pause_s", "leave_s", "leave_24k",
                                                                 "body_s", "body_24k", "delta_leave_s", "delta_body_s")},
                           "pcm": s, "checks": checks})
    by_id = {p["id"]: p for p in points}
    for p in points:
        other = next(q for q in points if q["cue"] == p["cue"] and q["id"].rsplit(":", 1)[0] == p["id"].rsplit(":", 1)[0] and q is not p)
        p["other_take"] = {"file": other["file"], "recognizer_minus_leave_s": round(other["recognizer"]["onset_s"] - other["energy_10ms"]["leave_s"], 3)}
        same_sign = (p["recognizer"]["onset_s"] - p["energy_10ms"]["leave_s"]) * p["other_take"]["recognizer_minus_leave_s"] > 0
        p["checks"]["other_take_same_direction"] = same_sign and abs(p["other_take"]["recognizer_minus_leave_s"]) >= 0.03
        p["proposal"] = ({"decision": "sample", "sample_24k": p["energy_10ms"]["leave_24k"]} if all(p["checks"].values())
                         else {"decision": None, "unresolved": [k for k, v in p["checks"].items() if not v]})
    (out / "points.json").write_text(json.dumps(points, indent=1) + "\n")
    flat = [{"id": p["id"], "selected": p["selected"], "word": p["word"], "recognizer_onset_s": p["recognizer"]["onset_s"],
             "recognizer_sample_24k": p["recognizer"]["sample_24k"], "word_dur_s": p["recognizer"]["word_dur_s"],
             "previous_word_dur_s": p["recognizer"]["previous_word_dur_s"], "pause_s": p["pcm"]["pause_s"],
             "pause_1ms_median_db": p["pcm"]["pause_1ms_median_db"], "pause_1ms_max_db": p["pcm"]["pause_1ms_max_db"],
             "pause_peak_dbfs": p["pcm"]["pause_peak_dbfs"], "leave_s": p["energy_10ms"]["leave_s"], "leave_24k": p["energy_10ms"]["leave_24k"],
             "body_s": p["energy_10ms"]["body_s"], "speech_1ms_s": p["pcm"]["speech_1ms_s"], "voicing_best": p["pcm"]["voicing_best"],
             "recognizer_minus_speech_s": p["pcm"]["recognizer_minus_speech_s"], "db_10ms_at_recognizer": p["pcm"]["db_10ms_at_recognizer"],
             "lead_in_r": [w["r"] for w in p["pcm"]["lead_in_10ms"]], "lead_in_zcr": [w["zcr"] for w in p["pcm"]["lead_in_10ms"]],
             "other_take_recognizer_minus_leave_s": p["other_take"]["recognizer_minus_leave_s"],
             "failed_checks": [k for k, v in p["checks"].items() if not v], "proposal_24k": p["proposal"].get("sample_24k")} for p in points]
    cols = list(flat[0])
    (out / "points.tsv").write_text(tsv(flat, cols))
    for f in flat:
        say(f"{f['id']}: recognizer {f['recognizer_onset_s']} s ('{f['word']}' {f['word_dur_s']} s, previous word {f['previous_word_dur_s']} s); "
            f"pause {f['pause_s'][0]}-{f['pause_s'][1]} s, 1 ms median/max {f['pause_1ms_median_db']}/{f['pause_1ms_max_db']} dB, peak "
            f"{f['pause_peak_dbfs']} dBFS; speech level (-40 dB, 1 ms) at {f['speech_1ms_s']} s; voicing r {f['voicing_best']['r']} at "
            f"{f['voicing_best']['f0_hz']} Hz; recognizer - speech {f['recognizer_minus_speech_s']:+} s, {f['db_10ms_at_recognizer']} dB "
            f"at the recognizer's onset; failed {f['failed_checks'] or 'none'}; proposal {f['proposal_24k']}")
    unresolved = [p for p in points if p["proposal"]["decision"] is None]
    need(not [p for p in unresolved if p["selected"]], "a selected point is unresolved: " +
         "; ".join(f"{p['id']} {p['proposal']['unresolved']}" for p in unresolved if p["selected"]))

    # ---------------------------------------------------------------- B. cue edges
    rec_rows = []
    for rel, t in sorted(inputs["takes"].items(), key=lambda kv: (list(script).index(kv[1]["cue"]), kv[0])):
        x, rate = read_pcm(rel)
        m, flags = ct.screen(rel)
        em = edge_measures(x, rate)
        need((m["lead_s"], m["tail_s"]) == (em["lead_40_s"], em["tail_40_s"]), f"{rel}: the -40 dBFS mirror differs from the screen")
        rec_rows.append({"cue": t["cue"], "file": os.path.basename(rel), "selected": rel == selected[t["cue"]],
                         "screen_lead_s": m["lead_s"], "screen_tail_s": m["tail_s"], "screen_flags": flags, **em})
    ecols = ["cue", "file", "selected", "frames", "rate", "frames_mod_3840_48k", "screen_lead_s", "screen_tail_s", "screen_flags",
             "lead_60_s", "tail_60_s", "first_10ms_db", "last_10ms_db", "first_1ms_db", "last_1ms_db", "first_samples", "last_samples",
             "start_step_dbfs", "end_step_dbfs"]
    (out / "edges-recordings.tsv").write_text(tsv(rec_rows, ecols))
    prod = []
    for cue in script:
        x, rate = read_pcm(f"{NARR}/{cue}.wav")
        em = edge_measures(x, rate)
        n48 = math.ceil(len(x) * 48000 / rate)
        need(n48 == 2 * len(x), "48 kHz length")
        ends = fmn_resample_at(x, rate, 48000, [0, 1, 2, n48 - 3, n48 - 2, n48 - 1])
        src = next(r for r in rec_rows if r["cue"] == cue and r["selected"])
        prod.append({"cue": cue, "file": src["file"], **em, "bus_first_48k": [round(ends[k] * 32768, 1) for k in (0, 1, 2)],
                     "bus_last_48k": [round(ends[k] * 32768, 1) for k in (n48 - 3, n48 - 2, n48 - 1)],
                     "bus_end_step_dbfs": dbfs(ends[n48 - 1] * 32768), "bus_start_step_dbfs": dbfs(ends[0] * 32768),
                     "source_last_1ms_db": src["last_1ms_db"], "source_first_1ms_db": src["first_1ms_db"]})
    pcols = ["cue", "file", "frames", "lead_40_s", "tail_40_s", "lead_60_s", "tail_60_s", "first_1ms_db", "last_1ms_db",
             "source_first_1ms_db", "source_last_1ms_db", "first_samples", "last_samples", "bus_first_48k", "bus_last_48k",
             "bus_start_step_dbfs", "bus_end_step_dbfs"]
    (out / "edges-production.tsv").write_text(tsv(prod, pcols))
    screen_tails = [r["screen_tail_s"] for r in rec_rows]
    sound_end = [r for r in rec_rows if r["last_1ms_db"] > QUIET]
    sound_start = [r for r in rec_rows if r["first_1ms_db"] > QUIET or r["lead_60_s"] < 0.005]
    say(f"edges: {len(rec_rows)} recordings; production-choices01's screen tails (after the last 10 ms frame at or above -40 dBFS) "
        f"{min(screen_tails)}-{max(screen_tails)} s, edge flags {sum(1 for r in rec_rows if r['screen_flags'])}; "
        f"to the last frame above -60 dBFS, tails {min(r['tail_60_s'] for r in rec_rows)}-{max(r['tail_60_s'] for r in rec_rows)} s; "
        f"{len(sound_end)} recordings have their last 1 ms above -60 dBFS "
        f"({', '.join(r['file'] + ('*' if r['selected'] else '') for r in sound_end)}; * = selected); {len(sound_start)} have "
        f"sound above -60 dBFS in their first 1 ms or first 10 ms frame "
        f"({', '.join(r['file'] + ('*' if r['selected'] else '') for r in sound_start)}); "
        f"every length is a multiple of 3840 samples at 48 kHz (80 ms): {all(r['frames_mod_3840_48k'] == 0 for r in rec_rows)}")
    for cue in EDGE_CUES:
        for r in [r for r in rec_rows if r["cue"] == cue]:
            pr = next(p for p in prod if p["cue"] == cue) if r["selected"] else None
            say(f"  {cue} {r['file']}{' (selected)' if r['selected'] else ''}: screen lead/tail {r['screen_lead_s']}/{r['screen_tail_s']} s; "
                f"-60 lead/tail {r['lead_60_s']}/{r['tail_60_s']} s; first/last 10 ms {r['first_10ms_db']}/{r['last_10ms_db']} dB, "
                f"1 ms {r['first_1ms_db']}/{r['last_1ms_db']} dB; edge samples {r['first_samples']} .. {r['last_samples']}"
                + (f"; voice bus at 48 kHz starts {pr['bus_first_48k']} ({pr['bus_start_step_dbfs']} dBFS), ends {pr['bus_last_48k']} "
                   f"({pr['bus_end_step_dbfs']} dBFS), then 0" if pr else ""))
    big = sorted(prod, key=lambda p: -p["bus_end_step_dbfs"])[:6]
    say("  largest end steps in the 80 production cues (dBFS of the last 48 kHz bus sample): "
        + ", ".join(f"{p['cue']} {p['bus_end_step_dbfs']}" for p in big))
    bigs = sorted(prod, key=lambda p: -p["bus_start_step_dbfs"])[:4]
    say("  largest start steps: " + ", ".join(f"{p['cue']} {p['bus_start_step_dbfs']}" for p in bigs))

    # ---------------------------------------------------------------- C. answers02 and the helper's own checks
    a01 = json.loads(Path(ANSWERS01).read_text())
    a02 = copy.deepcopy(a01)
    for p in points:
        if p["proposal"]["decision"] is None:
            continue
        prior = a02["answers"].get(p["id"])
        need(prior is None or prior.get("decision") is None, f"{p['id']} is already answered")
        s, e = p["pcm"], p["energy_10ms"]
        what = ("the recognizer starts the next sentence's first word" if p["kind"] == "onset" else "the recognizer starts the phrase")
        a02["answers"][p["id"]] = {
            "decision": "sample", "sample_24k": p["proposal"]["sample_24k"], "by": BY,
            "basis": (f"timing-corrections01: {what} '{p['word']}' at {p['recognizer']['onset_s']} s, abutting the previous word "
                      f"(word {p['recognizer']['word_dur_s']} s, previous {p['recognizer']['previous_word_dur_s']} s); the PCM pause "
                      f"{s['pause_s'][0]}-{s['pause_s'][1]} s stays at or below {s['pause_1ms_max_db']} dB in every 1 ms window, and "
                      f"speech-level voiced sound (autocorrelation {s['voicing_best']['r']} at {s['voicing_best']['f0_hz']} Hz) begins at "
                      f"{s['speech_1ms_s']} s, {abs(s['recognizer_minus_speech_s'])} s {'before' if s['recognizer_minus_speech_s'] > 0 else 'after'} "
                      f"the recognizer's onset; sample = the 10 ms leave frame {e['leave_s']} s, as for the 92 answers of "
                      f"production-choices01; in the other take ({p['other_take']['file']}) the recognizer's onset is "
                      f"{p['other_take']['recognizer_minus_leave_s']:+} s from leave, the same direction; not heard")}
    a02_path = out / "answers02.json"
    a02_path.write_text(json.dumps(a02, indent=2, ensure_ascii=False) + "\n")
    ap1 = pa.check_answers(b, ANSWERS01, questions, recordings, chosen)
    ap2 = pa.check_answers(b, str(a02_path), questions, recordings, chosen)
    receipt = json.loads(Path(f"{PREP}/prepare-receipt.json").read_text())
    plan = json.loads(Path(f"{PREP}/phrase-marks-plan.json").read_text())["cues"]
    need(receipt["answers"]["sha256"] == PINS[ANSWERS01] and receipt["answers"]["applied"] == ap1, "root's prepare applied other answers")
    changes, resolved2 = [], {}
    for cue, t in chosen.items():
        entry = by_file[(cue, os.path.basename(t["rel"]))]
        _p, frames24, _pcm = pa.wav_info(f"{NARR}/{cue}.wav")
        s1, e1, m1 = pa.resolved_points(entry, ap1, frames24, script[cue])
        s2, e2, m2 = pa.resolved_points(entry, ap2, frames24, script[cue])
        need(receipt["cues"][cue]["sentence_starts_24k"] == [{"sample": s, "basis": w} for s, w in s1], f"{cue}: replay differs from root's receipt")
        need(plan[cue] == m1, f"{cue}: replay differs from root's phrase-marks plan")
        for k, (u, v) in enumerate(zip(s1, s2), 1):
            if u != v:
                changes.append({"cue": cue, "point": f"s{k}", "from_24k": u[0], "to_24k": v[0], "shift_s": round((v[0] - u[0]) / 24000, 3)})
        for u, v in zip(m1, m2):
            if u != v:
                changes.append({"cue": cue, "point": f"b{u['byte']}", "from_s": u["seconds"], "to_s": v["seconds"],
                                "shift_s": round(v["seconds"] - u["seconds"], 3)})
        resolved2[cue] = (s2, e2, m2)
    want = {(p["cue"], p["id"].split(":")[2]) for p in points if p["selected"] and p["proposal"]["decision"] == "sample"}
    need({(c["cue"], c["point"]) for c in changes} == want and len(changes) == 5, f"unexpected changes {changes}")
    # the assembler's own dry run and unit_spans, on a scratch copy of root's prepared01 workspace
    scratch = out / "scratch"
    shutil.copytree(f"{PREP}/workspace", scratch / "workspace")
    for src, dst in ((b["assembler"], "tools/coherent_narration.py"), (b["assembler_cache"], "tools/narration_cache.py"), (b["map"], b["map_in_tree"])):
        need(sha(scratch / "workspace" / dst) == b["pins"][src], f"workspace copy {dst} differs from the binding's pin")
    spans1 = json.loads(Path(f"{PREP}/spans.json").read_text())
    spans2 = copy.deepcopy(spans1)
    unit_of = {u["line_ids"][0]: u["id"] for u in json.loads(Path(b["units"]).read_text())["units"]}
    for rec in spans2["units"]:
        cue = next(c for c, uid in unit_of.items() if uid == rec["id"])
        _p, frames24, _pcm = pa.wav_info(f"{NARR}/{cue}.wav")
        need(rec["sentence_ends"] == pa.resolved_points(by_file[(cue, os.path.basename(chosen[cue]["rel"]))], ap1, frames24, script[cue])[1],
             f"{cue}: root's spans.json ends differ from the replay")
        rec["sentence_ends"] = resolved2[cue][1]
    (scratch / "spans02-dry-run.json").write_text(json.dumps(spans2, indent=2) + "\n")
    accepted = pa.dry_run(str(scratch / "workspace/tools/coherent_narration.py"), b, str(scratch / "spans02-dry-run.json"), None)
    mod = pa.import_assembler(str(scratch / "workspace/tools/coherent_narration.py"), "tc")
    kscript, kunits = mod.read_units(Path(b["script"]), Path(b["units"]), True)
    report = json.loads(Path(f"{NARR}/report.json").read_text())
    keep = ("start", "end", "out_samples", "source_in_sample", "source_out_sample")
    predicted_rows, rows_changed = {}, []
    for unit in kunits:  # every cue: same PCM as root's narration WAV under both span sets; old rows = root's report rows
        cue = unit["line_ids"][0]
        r1 = next(r for r in spans1["units"] if r["id"] == unit["id"])
        r2 = next(r for r in spans2["units"] if r["id"] == unit["id"])
        old, new = mod.unit_spans(unit, kscript, r1), mod.unit_spans(unit, kscript, r2)
        wav_pcm = read_pcm(f"{NARR}/{cue}.wav")[0].tobytes()
        need(b"".join(pcm for _c, _r, pcm in old) == wav_pcm == b"".join(pcm for _c, _r, pcm in new), f"{cue}: PCM changes")
        need([row for _c, row, _p in old] == report["lines"][cue], f"{cue}: the in-process rows differ from root's report.json")
        predicted_rows[cue] = [{k: row[k] for k in keep} for _c, row, _p in new]
        if [row for _c, row, _p in new] != report["lines"][cue]:
            rows_changed.append(cue)
    need(len(kunits) == 80 and rows_changed == ["07_6"], f"report rows change for {rows_changed}")
    pt = json.loads(Path(f"{NARR}/phrase-timings.json").read_text())
    marks_changed = []
    for cue in script:  # root's phrase-timings marks = the plan; under answers02 only 04_2 b306 moves
        now = pt["cues"].get(cue, {}).get("marks", [])
        need(now == [{"byte": m["byte"], "seconds": m["seconds"], "basis": m["basis"]} for m in plan[cue]], f"{cue}: phrase-timings marks differ from the plan")
        marks_changed += [(cue, u["byte"]) for u, v in zip(plan[cue], resolved2[cue][2]) if u != v]
    need(marks_changed == [("04_2", 306)], f"marks change for {marks_changed}")
    predicted = {
        "wav_sha256_unchanged": {cue: sha(f"{NARR}/{cue}.wav") for cue in script},
        "report_lines_changed": {"07_6": predicted_rows["07_6"]},
        "phrase_timings_04_2_marks": [{"byte": m["byte"], "seconds": m["seconds"], "basis": m["basis"]} for m in resolved2["04_2"][2]],
        "checked_here": "unit_spans on all 80 cues: the same PCM as root's narration WAVs under both span sets, and only 07_6's "
                        "report rows change; phrase marks: only 04_2 b306 changes",
        "not_run_here": "the assembler's assemble and the helper's timings commands (they write production files); their "
                        "receipts will name the new prepare receipt and spans",
    }
    validation = {"answers01_applied": len(ap1), "answers02_applied": len(ap2), "answers02_sha256": sha(a02_path),
                  "replay_equals_root_prepared01": True, "changes": changes, "dry_run_cues_accepted": accepted,
                  "unit_spans_pcm_unchanged_cues": len(kunits), "report_rows_changed": rows_changed,
                  "marks_changed": [f"{c} b{b_}" for c, b_ in marks_changed], "predicted": predicted}
    (out / "validation.json").write_text(json.dumps(validation, indent=1) + "\n")
    say(f"answers02 {sha(a02_path)[:8]}: the installed helper applies {len(ap2)} answers ({len(ap1)} before); its resolved points for all "
        f"80 selected recordings equal root's prepared01 receipt and plan under answers.json, and under answers02 exactly "
        f"{len(changes)} change: " + "; ".join(f"{c['cue']} {c['point']} {c.get('from_24k', c.get('from_s'))} -> "
                                              f"{c.get('to_24k', c.get('to_s'))} ({c['shift_s']:+} s)" for c in changes))
    say(f"assembler (scratch copy of root's workspace, pins equal): dry run accepts {accepted} cues with the changed sentence ends; "
        f"unit_spans gives all {len(kunits)} cues the same PCM as root's narration WAVs under both span sets and root's report rows "
        f"under the old ends; report rows change only for {', '.join(rows_changed)}; phrase marks change only for "
        + ", ".join(f"{c} b{b_}" for c, b_ in marks_changed))
    bundled = Path(HERE) / "inputs/answers02.json"
    if bundled.exists():
        need(bundled.read_bytes() == a02_path.read_bytes(), "the bundled inputs/answers02.json differs from the regenerated one")
        say("the bundled inputs/answers02.json equals the regenerated file")
    (out / "stdout.txt").write_text("\n".join(lines) + "\n")
    print("TIMING CORRECTIONS: DONE")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
