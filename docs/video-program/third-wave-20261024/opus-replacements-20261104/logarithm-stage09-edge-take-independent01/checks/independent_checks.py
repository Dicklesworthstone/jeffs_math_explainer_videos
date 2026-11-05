#!/usr/bin/env python3
"""Independent checks of the four stage09 take swaps (root Mail 4614; Bronze's edge-take-review01, 95030197).

    python3 -I -B checks/independent_checks.py OUT.txt        run from the repository root; OUT.txt must not exist

Read-only: it reads the retained takes, their request and download receipts, the script, the recognizer reviews,
selection01/02, answers02/03 and root's decode02/prepare02 receipts, and writes only OUT.txt. It checks what
Bronze's review does not list: per-file voice evidence, the whole take's format and level against the take it
replaces, an independent word comparison, the onset answers against the alternative's own samples, and that
production now consumes exactly the reviewed files. Nothing is heard; levels and words are not listening.
"""
import difflib
import hashlib
import json
import math
import re
import sys
import wave
from array import array
from datetime import datetime, timezone
from pathlib import Path

P = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
EL = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
O = EL / "production-audio-stage09-01"
SCRIPT = EL / "combined-repair-stage09/candidate/explainer_corpus/logarithm_mysteries/script.tsv"
BRONZE = P / "logarithm-stage09-edge-take-review01"
SWAPS = {"01_2": ("01_2-download01.wav", "01_2-download02.wav"), "07_6": ("07_6-download01.wav", "07_6-download02.wav"),
         "11_5": ("11_5-download02.wav", "11_5-download01.wav"), "12_2": ("12_2-download01.wav", "12_2-download02.wav")}
VOICE = "Oliver - Clean, British and Steady"
PROVIDER_TAG = "Oliver - Clean, British and Steady_pvc_s50_sim75_v4"
RATE = 48000
LINES, RESULTS = [], []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def say(text):
    LINES.append(text)


def record(name, ok, detail):
    RESULTS.append(bool(ok))
    say(f"{'PASS' if ok else 'FAIL'} {name}: {detail}")


def rel(path):
    """Repository-relative form of a receipt path, so printed text carries no machine path."""
    text = str(path)
    i = text.find("narration_lab/")
    return text[i:] if i >= 0 else text


def samples(path):
    with wave.open(str(path)) as w:
        fmt = (w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes())
        data = array("h", w.readframes(w.getnframes()))
    return fmt, data


def db(power):
    return 10 * math.log10(power / 32768.0 ** 2) if power > 0 else -200.0


def frame_db(x, start, n=480):
    seg = x[max(0, start):max(0, start) + n]
    return db(sum(v * v for v in seg) / len(seg)) if len(seg) else -200.0


def levels(x):
    frames = [frame_db(x, i) for i in range(0, len(x) - 479, 480)]
    active = [f for f in frames if f >= -50]
    active_db = 10 * math.log10(sum(10 ** (f / 10) for f in active) / len(active))
    peak = max(abs(v) for v in x)
    return {"active_rms_db": round(active_db, 2), "active_s": round(len(active) * 0.01, 2),
            "peak_dbfs": round(20 * math.log10(peak / 32768.0), 2), "clipped": sum(1 for v in x if abs(v) >= 32767),
            "dc": round(sum(x) / len(x), 2)}


def words(text):
    return re.sub(r"[^a-z0-9' ]", " ", text.lower().replace("-", " ")).split()


def differences(script, heard):
    a, b = words(script), words(heard)
    return [f"{' '.join(a[i1:i2])}->{' '.join(b[j1:j2])}" for op, i1, i2, j1, j2 in
            difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if op != "equal"]


def onset_check(x, at48):
    before = max(frame_db(x, s) for s in range(at48 - 4800, at48, 480))
    delay = next((k * 0.01 for k in range(16) if frame_db(x, at48 + 480 * k) >= -40), None)
    return before, delay


def main(out):
    script = {r.split("\t")[0]: r.split("\t")[1] for r in SCRIPT.read_text().splitlines() if r and not r.startswith("#")}
    s1, s2 = (json.loads((O / n).read_text()) for n in ("selection01.json", "selection02.json"))
    decision = json.loads((O / "selection02-decision.json").read_text())
    root_sha = {Path(t["file"]).name: t["sha256"] for c in decision["independent_root_end_measurements"] for t in c["takes"]}
    opening = json.loads((EL / "opening-recognition01/recognition-comparison01.json").read_text())["records"]
    remaining = json.loads((EL / "remaining-recognition01/transcript-review01.json").read_text())["rows"]
    heard = {}
    for r in opening:
        heard.setdefault(r["take"], []).append(r["recognition"])
    for r in remaining:
        heard.setdefault(Path(r["input"]).name, []).append(r["transcript"])
    bronze_sums = sha(BRONZE / "SHA256SUMS")
    say(f"inputs: Bronze edge-take-review01 SHA256SUMS {bronze_sums[:12]}; script {sha(SCRIPT)[:12]}; selection02 "
        f"{sha(O / 'selection02.json')[:12]}; checks {sha(Path(__file__))[:12]}")
    record("R0 Bronze's sealed review is the one root's decision names, and selection02 names the same script",
           bronze_sums == decision["review_seal"] and s2["script_sha256"] == sha(SCRIPT) == s1["script_sha256"],
           f"review seal {bronze_sums[:12]}; script {sha(SCRIPT)[:12]}")
    diff_cues = sorted(c for c in set(s1["takes"]) | set(s2["takes"]) if s1["takes"].get(c) != s2["takes"].get(c))
    record("P1 selection02 differs from selection01 in exactly the four reviewed cues, each now the reviewed alternative",
           diff_cues == sorted(SWAPS) and all(Path(s2["takes"][c]).name == SWAPS[c][1] and Path(s1["takes"][c]).name == SWAPS[c][0]
                                              for c in SWAPS) and len(s2["takes"]) == 80,
           f"changed {diff_cues} of {len(s2['takes'])}")

    for cue, (old, new) in SWAPS.items():
        say(f"--- {cue}: {old} (selection01) -> {new} (selection02)")
        req = json.loads((EL / f"ui-takes01/{cue}-request01.json").read_text())
        dls = {Path(d["retained"]).name: d for d in json.loads((EL / f"ui-takes01/{cue}-downloads01.json").read_text())["downloads"]}
        asked = datetime.fromisoformat(req.get("utc") or req["at"])
        stamps = {n: datetime.strptime(re.search(r"ElevenLabs_(\d{4}-\d\d-\d\dT\d\d_\d\d_\d\d)_", d["source"]).group(1),
                                       "%Y-%m-%dT%H_%M_%S").replace(tzinfo=timezone.utc) for n, d in dls.items()}
        files = {}
        for name in (old, new):
            path = EL / "ui-takes01" / name
            fmt, x = samples(path)
            files[name] = (fmt, x)
            d = dls[name]
            ok = (sha(path) == d["sha256"] == root_sha[name] and hashlib.sha256(x.tobytes()).hexdigest() == d["pcm_sha256"]
                  and fmt == (1, 2, RATE, d["frames"]) and PROVIDER_TAG in d["source"]
                  and len(set(stamps.values())) == 1 and 0 <= (stamps[name] - asked).total_seconds() + 120 <= 3720)
            record(f"{cue} V {name}: the file is the receipt's and root's (file and PCM hash); the provider's own download "
                   f"name says {VOICE}, s50 sim75 v4, from the same generation as its pair, within an hour after the "
                   "request receipt", ok, f"{Path(d['source']).name}; provider stamp {(stamps[name] - asked).total_seconds():+.0f} s "
                                          f"from the request receipt; {fmt[3]} frames, {fmt[3] / RATE:.2f} s")
        text_sha = hashlib.sha256(script[cue].encode()).hexdigest()
        record(f"{cue} T the request's text hash is the current script line's; voice Oliver, Eleven v4, stability 0.5, "
               "similarity 0.75", req["text_sha256"] == text_sha and req.get("text", script[cue]) == script[cue]
               and req["voice"] in ("Oliver", VOICE) and req["model"] == "Eleven v4"
               and (req["stability"], req["similarity"]) == (0.5, 0.75),
               f"text {text_sha[:12]}, {len(script[cue].split())} words; receipt voice '{req['voice']}'")
        lv = {n: levels(files[n][1]) for n in (old, new)}
        gap = round(lv[new]["active_rms_db"] - lv[old]["active_rms_db"], 2)
        record(f"{cue} L the alternative's level matches the take it replaces (speech-frame RMS within 1 dB), no clipping, "
               "no DC offset", abs(gap) <= 1.0 and lv[new]["clipped"] == 0 and abs(lv[new]["dc"]) < 20,
               f"{old} {lv[old]}; {new} {lv[new]}; difference {gap:+.2f} dB")
        d_old = [differences(script[cue], h) for h in heard.get(old, [])]
        d_new = [differences(script[cue], h) for h in heard.get(new, [])]
        record(f"{cue} W my own word comparison of each take's recognizer transcript with the script: one pass each; "
               "the alternative has no difference the replaced take lacks",
               len(d_old) == len(d_new) == 1 and not set(d_new[0]) - set(d_old[0]),
               f"replaced {d_old[0] if d_old else None}; alternative {d_new[0] if d_new else None}")

    a2 = json.loads((P / "logarithm-stage09-timing-corrections01/inputs/answers02.json").read_text())["answers"]
    a3 = json.loads((O / "answers03.json").read_text())["answers"]
    added = sorted(set(a3) - set(a2))
    record("A1 answers03 is answers02 plus one answer, Bronze's optional 07_6 s5 for the alternative (625920 at 24 kHz); no "
           "answer changed or removed", added == ["onset:07_6:s5:07_6-download02.wav"] and not set(a2) - set(a3)
           and all(a2[k] == a3[k] for k in a2) and a3[added[0]]["sample_24k"] == 625920, f"added {added}")
    x7 = samples(EL / "ui-takes01/07_6-download02.wav")[1]
    rows = []
    for k in sorted(k for k in a3 if k.startswith("onset:07_6:") and k.endswith(":07_6-download02.wav")):
        at = a3[k]["sample_24k"] * 2
        before, delay = onset_check(x7, at)
        rows.append((k.split(":")[2], at / RATE, before, delay))
    record("A2 each 07_6 alternative onset answer, read against the alternative's own 48 kHz samples: the 0.10 s before it is "
           "quiet (every 10 ms frame below -55 dBFS) and speech (a frame at -40 dBFS or more) starts within 0.06 s",
           len(rows) == 5 and all(b < -55 and d is not None and d <= 0.06 for _, _, b, d in rows),
           "; ".join(f"{s} {t:.2f} s quiet max {b:.1f} dB, speech after {d:.2f} s" for s, t, b, d in rows))

    dec = json.loads((O / "decoded02/decode-receipt.json").read_text())
    prep = json.loads((O / "prepared02/prepare-receipt.json").read_text())
    comp = json.loads((O / "prepared02-comparison01.json").read_text())
    record("P2 production consumed exactly this: decode02 and prepare02 bind selection02, prepare02 binds answers03, and "
           "root's comparison finds 76 cues' PCM unchanged and only the four swapped",
           dec["selection_sha256"] == prep["selection"]["sha256"] == sha(O / "selection02.json")
           and prep["answers"]["sha256"] == sha(O / "answers03.json") and sorted(comp["changed_selected_takes"]) == sorted(SWAPS)
           and len(comp["same_pcm_cues"]) == 76, f"selection {sha(O / 'selection02.json')[:12]}, answers "
                                                  f"{sha(O / 'answers03.json')[:12]}")
    say(f"{sum(RESULTS)} PASS, {len(RESULTS) - sum(RESULTS)} FAIL")
    with open(out, "x") as handle:
        handle.write("\n".join(LINES) + "\n")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
