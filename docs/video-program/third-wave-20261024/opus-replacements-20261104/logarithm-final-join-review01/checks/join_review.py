#!/usr/bin/env python3
"""Read-only join review of the logarithm final master's encoded ASR.

Run from the repository root with a fresh OUT path:

    python3 -I -B <this file> OUT

Standard library only. Runs no recognizer, no decoder and no job. It reads the
retained ASR, triage, script, placements and PCM files, re-derives the word
comparison, and writes join-ledger.tsv, summary.json and windows-read.txt.
These are recognizer and signal measurements: nothing here is listening.
"""
import array
import difflib
import hashlib
import json
import math
import re
import statistics
import sys
import wave
from pathlib import Path

L = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
B = L / "logarithm-stage09-final-asr01"
T = L / "logarithm-stage09-final-targeted-asr01"
V = L / "logarithm-stage09-final-verify02"
EL = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
SCRIPT = EL / "combined-repair-stage09/candidate/explainer_corpus/logarithm_mysteries/script.tsv"
TRIAGE = EL / "final-export-review01/encoded-asr-triage01.json"
TARGETED = EL / "final-export-review01/targeted-asr-triage01.json"
NARRATION = EL / "production-audio-stage09-01/prepared02/narration"
DECODED = EL / "production-audio-stage09-01/decoded02/decoded"
WAV = B / "encoded-master-16k-mono.wav"

MASTER = "00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24"
FS = 32768.0
FLOOR = -38.0  # above the music bed, so the envelope follows the voice
ENV_S = 5.0
HOP_S = 0.01
MAX_LAG = 30  # hops, so 300 ms
GAP_SPEECH_DB = -36.0

WORD = re.compile(r"[a-z]+(?:'[a-z]+)?|\d+(?:\.\d+)?|[-%]")
UNITS = dict(zip("one two three four five six seven eight nine".split(), range(1, 10)))
TEENS = dict(zip("ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split(), range(10, 20)))
TENS = dict(zip("twenty thirty forty fifty sixty seventy eighty ninety".split(), range(20, 100, 10)))
DIGIT = {"zero": 0, **UNITS}

# Written by hand after reading windows-read.txt; the ledger carries them verbatim.
NOTES = {
    "01_4": "Hold: recognizer stamps Use 4.44 s before placement, inside the question hold; the take is at placement.",
    "02_3": "OPEN closing: turn or turned at 307.72-308.10; passes disagree 2 to 1.",
    "04_2": "Followed by a 2.61 s gap while the picture's probe sweep finishes; source runs the sweep before 04_3 is said.",
    "04_3": "Loudest file start of 80 (first 5 ms peak -35.5 dBFS); voicing starts at 25 ms; recognized That. Complete recording, no trim. Listening lead only.",
    "04_4": "OPEN closing: a small step 537.48-537.50, three tokens in 0.02 s, absent from both unsplit passes; Wow 538.63-538.86 in one of three passes.",
    "05_1": "Closing window ends with Two, the next cue's first word, stamped 4.82 s before its placement.",
    "05_2": "Hold: recognizer stamps Two 4.82 s before placement, inside the question hold; the take is at placement.",
    "05_5": "Cue is 4.40 s, so its two windows overlap and show the same words.",
    "06_3": "Hold: recognizer stamps Two 4.27 s before placement, inside the question hold; the take is at placement.",
    "10_4": "Followed by an authored fine-print hold of 7.45 s across the chapter join.",
    "11_1": "Recognizer stamps Can 6.83 s before placement, inside the authored hold; the take is at placement.",
    "11_20": "Followed by an authored reading hold of 18.35 s, the longest in the film.",
    "11_8": "Recognizer stamps Allow 17.58 s before placement, inside the authored hold; the take is at placement.",
    "12_21": "Film edge: last word family. ends 1936.32; cue ends 1936.82; video ends 1937.42.",
}
OPEN_ROWS = {30: "02_3", 83: "04_4"}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def toks(text):
    return WORD.findall(text.lower().replace("’", "'"))


def spoken_digits(tokens):
    """Digits of a spoken number run, concatenated. Punctuation is not compared."""
    out, cur, thou, prev, i = [], None, 0, None, 0

    def flush():
        nonlocal cur, thou
        if cur is not None or thou:
            out.append(str(thou + (cur or 0)))
        cur, thou = None, 0

    while i < len(tokens):
        t = tokens[i].split("'")[0]
        if t == "point":
            flush()
            i += 1
            while i < len(tokens) and tokens[i].split("'")[0] in DIGIT:
                out.append(str(DIGIT[tokens[i].split("'")[0]]))
                i += 1
            prev = None
            continue
        if t == "zero":
            flush()
            out.append("0")
        elif t == "a":
            flush()
            cur = 0
        elif t == "hundred":
            cur = (cur or 1) * 100
        elif t == "thousand":
            thou, cur = (cur or 0) * 1000, None
        elif t in UNITS:
            v = UNITS[t]
            if prev == "-" and cur is not None and cur % 10 == 0:
                cur += v
            elif prev == "and" and cur is not None and cur % 100 == 0:
                cur += v
            elif cur is None:
                cur = v
            else:
                flush()
                cur = v
        elif t in TEENS or t in TENS:
            v = TEENS.get(t, TENS.get(t))
            if cur is not None and cur >= 100 and cur % 100 == 0:
                cur += v
            elif cur is None:
                cur = v
            else:
                flush()
                cur = v
        prev = t
        i += 1
    flush()
    return "".join(out)


def rec_digits(tokens):
    return "".join(t.replace(".", "") for t in tokens if re.fullmatch(r"\d+(\.\d+)?", t))


def row_class(text):
    if text.startswith("Unresolved"):
        return "open"
    if "homophone" in text:
        return "homophone"
    if "hyphenation" in text:
        return "hyphenation"
    if "regional spelling" in text:
        return "spelling"
    return "numeric"


def load_pcm(path):
    with wave.open(str(path)) as w:
        assert w.getsampwidth() == 2 and w.getnchannels() == 1, path
        rate = w.getframerate()
        a = array.array("h")
        a.frombytes(w.readframes(w.getnframes()))
    return rate, a


def level(a, i0, i1):
    i0, i1 = max(0, i0), min(len(a), i1)
    if i1 <= i0:
        return -120.0
    s = sum(x * x for x in a[i0:i1])
    return -120.0 if s == 0 else 10 * math.log10(s / (i1 - i0)) - 20 * math.log10(FS)


def peak(a, i0, i1):
    p = max((abs(x) for x in a[max(0, i0):min(len(a), i1)]), default=0)
    return -120.0 if p == 0 else 20 * math.log10(p / FS)


def env(a, rate, t0, frames):
    hop = round(rate * HOP_S)
    i0 = round(t0 * rate)
    return [max(FLOOR, level(a, i0 + k * hop, i0 + (k + 1) * hop)) for k in range(frames)]


def pearson(x, y):
    n = len(x)
    mx, my = sum(x) / n, sum(y) / n
    sxy = sum((p - mx) * (q - my) for p, q in zip(x, y))
    sxx = sum((p - mx) ** 2 for p in x)
    syy = sum((q - my) ** 2 for q in y)
    return 0.0 if sxx == 0 or syy == 0 else sxy / math.sqrt(sxx * syy)


def best_lag(x, y):
    """x against y, where y carries MAX_LAG extra hops on both sides."""
    best = (-2.0, 0)
    for lag in range(-MAX_LAG, MAX_LAG + 1):
        r = pearson(x, y[MAX_LAG + lag:MAX_LAG + lag + len(x)])
        if r > best[0]:
            best = (r, lag)
    return best


def main(out):
    out.mkdir(parents=False, exist_ok=False)
    s = {}

    # Bindings
    cb = json.loads((B / "cue-boundaries.json").read_text())
    fl = json.loads((B / "full-large.json").read_text())
    cp = json.loads((B / "comparison.json").read_text())
    tr = json.loads(TRIAGE.read_text())
    tg = json.loads(TARGETED.read_text())
    ver = json.loads((V / "verification.json").read_text())
    res = json.loads((T / "results.json").read_text())
    assert cb["master_sha256"] == cp["master_sha256"] == tr["master_sha256"] == ver["sha256"] == res["master_sha256"] == MASTER
    assert cp["script_sha256"] == sha(SCRIPT)
    assert cp["wav_sha256"] == sha(WAV)
    assert tr["comparison_sha256"] == sha(B / "comparison.json")
    s["bindings"] = {
        "master": MASTER,
        "script": sha(SCRIPT),
        "wav": sha(WAV),
        "cue_boundaries": sha(B / "cue-boundaries.json"),
        "full_large": sha(B / "full-large.json"),
        "comparison": sha(B / "comparison.json"),
        "triage": sha(TRIAGE),
        "targeted_triage": sha(TARGETED),
        "targeted_results": sha(T / "results.json"),
        "verification": sha(V / "verification.json"),
    }

    # Script, placements and window provenance
    script = []
    for line in SCRIPT.read_text().splitlines():
        if line.strip() and not line.startswith("#"):
            p = line.split("\t")
            script.append((p[0], p[1]))
    text = dict(script)
    cues = cb["cues"]
    assert [c for c, _ in script] == [p["cue"] for p in ver["placements"]] == [c["cue"] for c in cues]
    assert len(cues) == cb["cue_count"] == 80
    for p, c in zip(ver["placements"], cues):
        assert abs(p["start_frame"] / ver["fps"] - c["start_s"]) < 1e-6
        assert abs(p["start_frame"] / ver["fps"] + p["duration_s"] - c["end_s"]) < 1e-6
        w = toks(text[c["cue"]])
        assert c["expected_first_words"] == w[:12] and c["expected_last_words"] == w[-12:], c["cue"]
    segs = fl["result"]["segments"]
    windows = 0
    for c in cues:
        assert [w["edge"] for w in c["windows"]] == ["opening", "closing"]
        for w in c["windows"]:
            lo, hi = w["window_s"]
            edge = c["start_s"] if w["edge"] == "opening" else c["end_s"]
            # a window reaches 5 s into its cue, or to the far end of a shorter cue
            assert abs(lo - (edge - 0.5 if w["edge"] == "opening" else max(edge - 5.0, c["start_s"]))) < 1e-6
            assert abs(hi - (min(edge + 5.0, c["end_s"]) if w["edge"] == "opening" else edge + 0.5)) < 1e-6
            want = [(x["start_sec"], x["end_sec"], x["text"]) for x in segs if x["start_sec"] < hi and x["end_sec"] > lo]
            assert want == [(x["start_sec"], x["end_sec"], x["text"]) for x in w["recognized_segments"]], c["cue"]
            assert want and not w["no_recognition_in_window"], c["cue"]
            windows += 1
    s["windows_bound_to_full_large"] = windows

    # Replicate the word comparison row by row
    E = [(w, c) for c, t in script for w in toks(t)]
    R = []
    for k, x in enumerate(segs):
        for w in toks(x["text"]):
            R.append((w, x["start_sec"], x["end_sec"], k))
    assert len(E) == cp["expected_words"] and len(R) == cp["recognized_words"], (len(E), len(R))
    ops = difflib.SequenceMatcher(None, [e[0] for e in E], [r[0] for r in R], autojunk=False).get_opcodes()
    diffs = [o for o in ops if o[0] != "equal"]
    assert len(diffs) == len(cp["differences"]) == len(tr["rows"]) == tr["differences_reviewed"] == 157
    expected_status, rec_status = {}, {}
    for o in ops:
        if o[0] == "equal":
            for a, b in zip(range(o[1], o[2]), range(o[3], o[4])):
                expected_status[a], rec_status[b] = ("equal", b), ("equal", a)
    for k, (o, row) in enumerate(zip(diffs, cp["differences"])):
        assert [e[0] for e in E[o[1]:o[2]]] == [x["word"] for x in row["expected"]], k
        assert [r[0] for r in R[o[3]:o[4]]] == [x["word"] for x in row["recognized"]], k
        for a in range(o[1], o[2]):
            expected_status[a] = ("row", k)
        for b in range(o[3], o[4]):
            rec_status[b] = ("row", k)
    s["replicated_difference_rows"] = len(diffs)
    s["difference_operations"] = {n: sum(1 for o in diffs if o[0] == n) for n in sorted({o[0] for o in diffs})}

    # Classify root's rows and prove the numeric ones digit for digit
    classes = [row_class(r["assessment"]) for r in tr["rows"]]
    s["triage_classes"] = {c: classes.count(c) for c in sorted(set(classes))}
    assert {k for k, c in enumerate(classes) if c == "open"} == set(OPEN_ROWS)
    numeric_bad = []
    numeric_rows = 0
    for k, (o, c) in enumerate(zip(diffs, classes)):
        if c != "numeric" or k == 1:
            continue
        if k == 0:  # rows 0 and 1 are one list, split by a matched "and"
            n = diffs[1]
            exp = [e[0] for e in E[o[1]:n[2]]]
            rec = [r[0] for r in R[o[3]:n[4]]]
            numeric_rows += 2
        else:
            exp = [e[0] for e in E[o[1]:o[2]]]
            rec = [r[0] for r in R[o[3]:o[4]]]
            numeric_rows += 1
        if spoken_digits(exp) != rec_digits(rec):
            numeric_bad.append({"row": k, "expected": " ".join(exp), "digits": spoken_digits(exp), "recognized": " ".join(rec)})
    s["numeric_rows_digits_equal"] = {"rows": numeric_rows, "mismatch": numeric_bad}
    assert spoken_digits(toks("eight hundred and eighty one")) != rec_digits(["880"]), "digit proof must fail on a wrong value"

    # Per-cue reading of ownership and time
    cue_index = {c["cue"]: i for i, c in enumerate(cues)}
    owner = {}
    last_cue = E[0][1]
    for j in range(len(R)):
        st = rec_status[j]
        if st[0] == "equal":
            owner[j] = E[st[1]][1]
            last_cue = owner[j]
        else:
            o = diffs[st[1]]
            owner[j] = E[o[1]][1] if o[2] > o[1] else last_cue
    token_of_seg = {}
    for j, r in enumerate(R):
        token_of_seg.setdefault(r[3], []).append(j)
    tokens_of_cue = {}
    for i, e in enumerate(E):
        tokens_of_cue.setdefault(e[1], []).append(i)

    def partner(i):
        st = expected_status[i]
        if st[0] == "equal":
            return R[st[1]]
        o = diffs[st[1]]
        return R[o[3]] if o[4] > o[3] else None

    ledger = []
    reading = []
    outside = []
    leads = []
    neighbours_total = 0
    foreign_total = 0
    inserted = []
    shared_ngrams = []
    long_words = []
    window_open = []
    overruns = []
    for n, c in enumerate(cues):
        cue = c["cue"]
        idx = tokens_of_cue[cue]
        first, last = idx[0], idx[-1]
        p_first, p_last = partner(first), partner(last)
        lead = p_first[1] - c["start_s"]
        leads.append((round(lead, 3), cue, R[expected_status[first][1]][0] if expected_status[first][0] == "equal" else E[first][0]))
        if p_last[1] >= c["end_s"]:
            overruns.append(cue)
        for i in idx:
            q = partner(i)
            if q and i != first and not c["start_s"] - 0.05 <= q[1] <= c["end_s"]:
                outside.append({"cue": cue, "word": E[i][0], "start_s": round(q[1], 3)})
        stats = {}
        for w in c["windows"]:
            lo, hi = w["window_s"]
            seg_ids = [k for k, x in enumerate(segs) if x["start_sec"] < hi and x["end_sec"] > lo]
            tok_ids = [j for k in seg_ids for j in token_of_seg.get(k, [])]
            own = sum(1 for j in tok_ids if owner[j] == cue)
            near = [j for j in tok_ids if owner[j] != cue and abs(cue_index[owner[j]] - n) == 1]
            foreign = [j for j in tok_ids if owner[j] != cue and abs(cue_index[owner[j]] - n) > 1]
            rows = sorted({rec_status[j][1] for j in tok_ids if rec_status[j][0] == "row"})
            ins = [j for j in tok_ids if rec_status[j][0] == "row" and diffs[rec_status[j][1]][0] == "insert" and R[j][0] != "-"]
            for j in tok_ids:
                dur = R[j][2] - R[j][1]
                if dur > 1.2 and not re.search(r"[.,?:;]$", segs[R[j][3]]["text"].strip()):
                    long_words.append({"cue": cue, "edge": w["edge"], "word": segs[R[j][3]]["text"].strip(), "start_s": round(R[j][1], 2), "end_s": round(R[j][2], 2)})
            opened = [k for k in rows if classes[k] == "open"]
            stats[w["edge"]] = {"own": own, "near": len(near), "rows": rows, "open": opened}
            neighbours_total += len(near)
            foreign_total += len(foreign)
            inserted += [(cue, w["edge"], R[j][0], round(R[j][1], 3)) for j in ins]
            if opened:
                window_open.append(f"{cue} {w['edge']}")
            marked, words = False, []
            mark_at = c["start_s"] if w["edge"] == "opening" else c["end_s"]
            for k in seg_ids:
                if not marked and segs[k]["start_sec"] >= mark_at:
                    words.append("||")
                    marked = True
                words.append(segs[k]["text"].strip())
            if not marked:
                words.append("||")
            plain = text[cue].split()
            side = plain[:22] if w["edge"] == "opening" else plain[-22:]
            reading.append(f"{cue} {w['edge']} window {lo:.3f}-{hi:.3f} edge {mark_at:.3f}")
            reading.append(f"  script: {' '.join(side)}")
            reading.append(f"  recognized: {' '.join(words)}")
        tail = toks(text[cue])[-12:]
        if n + 1 < len(cues):
            head = toks(text[cues[n + 1]["cue"]])[:12]
            for size in range(6, 2, -1):
                hit = {tuple(tail[i:i + size]) for i in range(len(tail) - size + 1)} & {tuple(head[i:i + size]) for i in range(len(head) - size + 1)}
                if hit:
                    shared_ngrams.append({"cue": cue, "next": cues[n + 1]["cue"], "shared": " ".join(sorted(hit)[0])})
                    break
        ledger.append({"n": n, "cue": cue, "c": c, "lead": lead, "p_last": p_last, "stats": stats})
    (out / "windows-read.txt").write_text("\n".join(reading) + "\n")
    s["cues"] = len(ledger)
    s["word_outside_own_placement_excluding_first_words"] = outside
    s["last_word_starts_after_cue_end"] = overruns
    s["neighbour_tokens_in_windows"] = neighbours_total
    s["tokens_from_a_non_adjacent_cue_in_windows"] = foreign_total
    s["inserted_non_hyphen_tokens_in_windows"] = inserted
    s["shared_three_plus_word_runs_across_joins"] = shared_ngrams
    s["windows_with_open_rows"] = window_open
    unique_long = {(x["word"], x["start_s"], x["end_s"]) for x in long_words}
    s["unpunctuated_words_over_1.2s_in_windows"] = sorted(unique_long, key=lambda x: x[1])
    touched = sorted({k for r in ledger for e in r["stats"].values() for k in e["rows"]})
    s["difference_rows_touching_windows"] = {
        "rows": len(touched),
        "by_class": {c: sum(1 for k in touched if classes[k] == c) for c in sorted(set(classes))},
    }
    edge = {}
    for c in cues:
        idx = tokens_of_cue[c["cue"]]
        for i in idx[:12] + idx[-12:]:
            st = expected_status[i]
            name = "exact" if st[0] == "equal" else classes[st[1]]
            edge[name] = edge.get(name, 0) + 1
    s["edge_tokens_first_and_last_12_by_status"] = edge
    s["first_word_lead_s"] = {
        "median": round(statistics.median(x[0] for x in leads), 3),
        "earlier_than_0.2s": sum(1 for x in leads if x[0] <= -0.2),
        "earlier_than_1s": sorted(x for x in leads if x[0] <= -1.0),
    }
    conf = [x["confidence"] for x in segs]
    d = json.loads((T / "04_4-large-unsplit.json").read_text())
    d = d.get("result", d)
    s["confidence_field_sums"] = {"full_large": round(sum(conf), 4), "04_4_large_unsplit": round(sum(x["confidence"] for x in d["segments"]), 4)}

    # Narration files against placements, and against the encoded master
    rate_m, mast = load_pcm(WAV)
    assert rate_m == 16000
    envs = {}
    for row in ledger:
        cue, c = row["cue"], row["c"]
        rate, a = load_pcm(NARRATION / f"{cue}.wav")
        rate_d, b = load_pcm(DECODED / f"{cue}.wav")
        assert rate == rate_d == 24000 and a == b, cue
        dur = len(a) / rate
        assert abs(dur - (c["end_s"] - c["start_s"])) < 1e-6, cue
        row["peak_first5"] = peak(a, 0, rate // 200)
        row["peak_last5"] = peak(a, len(a) - rate // 200, len(a))
        hop = rate // 100
        head = [(k * HOP_S, level(a, k * hop, (k + 2) * hop)) for k in range(30)]
        row["voice_on"] = next(t for t, v in head if v > -40.0)
        tail = [(k * HOP_S, level(a, len(a) - (k + 2) * hop, len(a) - k * hop)) for k in range(60)]
        row["quiet_tail"] = next(t for t, v in tail if v > -40.0)
        frames = min(round(ENV_S / HOP_S), int(dur / HOP_S))
        late = dur - frames * HOP_S
        x_open, x_close = env(a, rate, 0.0, frames), env(a, rate, late, frames)
        lag_s = MAX_LAG * HOP_S
        y_open = env(mast, rate_m, c["start_s"] - lag_s, frames + 2 * MAX_LAG)
        y_close = env(mast, rate_m, c["start_s"] + late - lag_s, frames + 2 * MAX_LAG)
        row["open_match"], row["close_match"] = best_lag(x_open, y_open), best_lag(x_close, y_close)
        envs[cue] = (x_open, x_close, y_open, y_close)
    controls = []
    for row, other in zip(ledger[1:], ledger):
        xo, xc, *_ = envs[other["cue"]]
        _, _, yo, yc = envs[row["cue"]]
        m = min(len(xo), len(yo) - 2 * MAX_LAG)
        controls.append(best_lag(xo[:m], yo[:m + 2 * MAX_LAG])[0])
        m = min(len(xc), len(yc) - 2 * MAX_LAG)
        controls.append(best_lag(xc[:m], yc[:m + 2 * MAX_LAG])[0])
    matches = [row["open_match"] for row in ledger] + [row["close_match"] for row in ledger]
    s["pcm_equal_to_complete_recording"] = sum(1 for _ in ledger)
    s["file_length_equals_placement"] = sum(1 for _ in ledger)
    s["envelope_match"] = {
        "windows": len(matches),
        "min_r": round(min(m[0] for m in matches), 3),
        "median_r": round(statistics.median(m[0] for m in matches), 3),
        "lag_ms_values": {str(k * 10): sum(1 for m in matches if m[1] == k) for k in sorted({m[1] for m in matches})},
        "negative_control_neighbour_take": {"windows": len(controls), "max_r": round(max(controls), 3), "median_r": round(statistics.median(controls), 3)},
    }
    s["tail_peak_last5ms_dbfs_max"] = round(max(r["peak_last5"] for r in ledger), 1)
    s["quiet_tail_s_min"] = round(min(r["quiet_tail"] for r in ledger), 2)
    s["head_peak_first5ms_top3"] = sorted(((round(r["peak_first5"], 1), r["cue"]) for r in ledger), reverse=True)[:3]

    # What the master holds between cues
    for row, nxt in zip(ledger, ledger[1:]):
        en, st = row["c"]["end_s"], nxt["c"]["start_s"]
        i = round((en + 0.02) * rate_m)
        stop = round((st - 0.02) * rate_m)
        hop = round(rate_m * HOP_S)
        row["gap_peak"] = max((level(mast, k, k + hop) for k in range(i, stop - hop + 1, hop)), default=-120.0)
        row["gap"] = st - en
    holds = [r for r in ledger[:-1] if 4.8 < r["gap"] < 4.9]
    s["thinking_holds"] = {
        "count": len(holds),
        "cues": [r["cue"] for r in holds],
        "ending_in_a_question_mark": sum(1 for r in holds if text[r["cue"]].rstrip().endswith("?")),
        "max_gap_peak_dbfs": round(max(r["gap_peak"] for r in holds), 1),
        "next_word_stamped_0.5s_or_more_early": [ledger[h["n"] + 1]["cue"] for h in holds if ledger[h["n"] + 1]["lead"] <= -0.5],
    }
    s["gap_classes"] = {
        "near_0.6": sum(1 for r in ledger[:-1] if r["gap"] < 0.7),
        "near_1.2": sum(1 for r in ledger[:-1] if 1.1 < r["gap"] < 1.3),
        "near_4.85": sum(1 for r in ledger[:-1] if 4.8 < r["gap"] < 4.9),
        "other": [(r["cue"], round(r["gap"], 3)) for r in ledger[:-1] if r["gap"] >= 1.3 and not 4.8 < r["gap"] < 4.9],
    }
    s["gaps_above_speech_threshold"] = [(r["cue"], round(r["gap_peak"], 1)) for r in ledger[:-1] if r["gap_peak"] > GAP_SPEECH_DB]
    s["gap_peak_dbfs_max"] = round(max(r["gap_peak"] for r in ledger[:-1]), 1)

    # The disputed 04_4 ending, in the complete recording and in the master
    rate, a = load_pcm(NARRATION / "04_4.wav")
    c44 = next(c for c in cues if c["cue"] == "04_4")
    p0, p1 = round(30.00 * rate), round(30.23 * rate)
    where = max(range(p0, p1), key=lambda k: abs(a[k]))
    s["wow_span"] = {
        "narration_peak_dbfs": round(peak(a, p0, p1), 2),
        "narration_peak_at_s": round(where / rate, 4),
        "narration_rms_dbfs_by_span": {f"{t0:.2f}-{t1:.2f}": round(level(a, round(t0 * rate), round(t1 * rate)), 1) for t0, t1 in ((29.90, 30.00), (30.00, 30.05), (30.05, 30.10), (30.10, 30.15), (30.15, 30.24))},
        "master_rms_dbfs_by_span": {f"{t0:.2f}-{t1:.2f}": round(level(mast, round((c44["start_s"] + t0) * rate_m), round((c44["start_s"] + t1) * rate_m)), 1) for t0, t1 in ((29.90, 30.00), (30.00, 30.05), (30.05, 30.10), (30.10, 30.15), (30.15, 30.24), (30.24, 30.60))},
        "master_bed_rms_dbfs_next_gap": round(level(mast, round((c44["end_s"] + 0.05) * rate_m), round((c44["end_s"] + 0.55) * rate_m)), 1),
    }
    s["passes"] = {
        "02_3": {x["model"]: ("turned" if " turned " in x["text"] else "turn" if " turn " in x["text"] else "neither") for x in res["results"] if x["cue"] == "02_3"},
        "04_4_a_small_step": {x["model"]: "base e is a small step" in x["text"].lower() for x in res["results"] if x["cue"] == "04_4"},
        "04_4_wow": {x["model"]: "Wow" in x["text"] for x in res["results"] if x["cue"] == "04_4"},
        "full_large_02_3": "turn" if any(x["text"].strip() == "turn" and 307 < x["start_sec"] < 309 for x in segs) else "other",
        "full_large_a_small_step_s": [(round(x["start_sec"], 3), round(x["end_sec"], 3), x["text"].strip()) for x in segs if 537.47 < x["start_sec"] < 537.5],
        "full_large_wow": any("wow" in x["text"].lower() for x in segs),
        "targeted_triage_decision": tg["decision"],
    }

    # One row per cue
    head = "cue\tchapter\tstart_s\tend_s\tgap_after_s\tfirst_words\tlast_words\topening_own\topening_neighbour\tclosing_own\tclosing_neighbour\topening_rows\tclosing_rows\trec_first_word_lead_s\trec_last_word_end_past_cue_s\topening_lag_ms\topening_r\tclosing_lag_ms\tclosing_r\tfirst5ms_peak_dbfs\tlast5ms_peak_dbfs\tgap_peak_dbfs\tverdict\tnote\n"
    rows = [head]
    for r in ledger:
        c, st = r["c"], r["stats"]
        o, e = st["opening"], st["closing"]
        verdict = "open" if o["open"] or e["open"] else "clear"
        fw = " ".join(text[r["cue"]].split()[:4])
        lw = " ".join(text[r["cue"]].split()[-4:])
        rows.append("\t".join(str(x) for x in (
            r["cue"], c["chapter"], f"{c['start_s']:.3f}", f"{c['end_s']:.3f}",
            f"{r['gap']:.3f}" if "gap" in r else "", fw, lw, o["own"], o["near"], e["own"], e["near"],
            ",".join(map(str, o["rows"])) or "-", ",".join(map(str, e["rows"])) or "-",
            f"{r['lead']:.3f}", f"{r['p_last'][2] - c['end_s']:.3f}",
            r["open_match"][1] * 10, f"{r['open_match'][0]:.3f}", r["close_match"][1] * 10, f"{r['close_match'][0]:.3f}",
            f"{r['peak_first5']:.1f}", f"{r['peak_last5']:.1f}", f"{r['gap_peak']:.1f}" if "gap_peak" in r else "",
            verdict, NOTES.get(r["cue"], "-"))) + "\n")
    (out / "join-ledger.tsv").write_text("".join(rows))
    s["verdicts"] = {"clear": sum(1 for x in rows[1:] if "\tclear\t" in x), "open": sum(1 for x in rows[1:] if "\topen\t" in x)}
    (out / "summary.json").write_text(json.dumps(s, indent=1, sort_keys=True) + "\n")
    print(json.dumps(s, indent=1, sort_keys=True))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: join_review.py OUT")
    main(Path(sys.argv[1]))
