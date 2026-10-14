#!/usr/bin/env python3
"""Prosody QA of a narration: the measures that told Video F from human readers, with bands taken
from the human recordings, so a fix is judged by numbers a person's reading also meets.

    uv run --no-project --with numpy --with praat-parselmouth python3 -I tools/prosody_qa.py \
        OUT_DIR [--timeline DIR] [--script spoken.tsv] [--json FILE] [--pitch-range LO,HI] \
        [--gate-room-tone] [--gate-regularity] [--gate-terminal] [--gate-gap-targets]

OUT_DIR is a narration `out/` (report.json and the line WAVs; spoken.tsv beside them gives the
text). --timeline is a renders/scored (or narrated) directory with <chapter>.timeline.tsv and
<chapter>.cues.tsv: it gives the chapters and the silences between lines. Without it a chapter is
a line id's prefix and the line gaps are not measured.

Verdicts: FAIL (exit 1), FLAG (listed, exit 0), ADVISORY (reported, never fails), PASS.

The one gate, always on: a silence inside a sentence over 1.0 s (F has none after pause surgery).
Flags, listed, never fail unless their --gate-* option is given (the owner's ear decides these):
  - the share of sentence-join pitch resets beyond +12 st, per video, shown beside the human
    readers' range and flagged above 26%. Human readers: pooled 7% (T-007, 4 of 55 joins) or 12%
    (audit-7, 5 of 43), LibriVox alone 16% (T-007) or 26% (audit-7), the scripted explainer 0%;
    Video F 29-31%. There is no human-calibrated absolute threshold below 16-26%, and F is not
    separable from LibriVox alone: this is a comparison, not a gate, and there is no --gate for
    it. A single join is never judged: the trackers disagree on some of them. Below MIN_JOINS
    resets the share is advisory.
  - a silence inside a sentence over 0.3 s whose sentence has no comma, colon or dash (B-C09);
  - digital silence between sentences (F: -89 dBFS, 98% exact zeros; human -56..-65 dBFS);
  - regularity: gaps that are one value (F's detected sentence gaps CV 0.19, human 1.15), plain gaps
    between lines that are one value or do not differ between a topic change and a continuation,
    chapter joins that are copies of one value;
  - terminal events: a rise of the last 120 ms over 9 st, confirmed by a second tracker over 4.5 st
    (B-C03: 10 of F's 274 sentences, expected 0 pending the owner's ear);
  - --gate-gap-targets: the class medians of the gaps between lines at 0.99 s (topic change),
    0.81 s (beat), 0.58 s (continuation), plus or minus 0.15 s.
Advisory: the median reset, the register step between adjacent sentences, the level step, the
resets by chapter.

Measures (T-007 definitions, the two trackers are Praat's autocorrelation and cross-correlation
pitch): a reset is the median f0 of the first 150 ms of voiced frames of the sentence after a join
minus that of the last 150 ms before it, windows looking 0.6 s out and clipped to the sentences,
the join being the silence (frames 24 dB under the line's p95 level, 40 ms or more) that holds the
inserted gap. A level step is the median level of the frames within 30 dB of that top in the same
windows. Gaps are detected silences, not word times (ASR word ends smear by up to 0.8 s).
"""
import argparse
import json
import math
import re
import sys
import wave
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True

WIN = 0.15  # s: the pitch windows at a join (T-001, T-007)
SPAN = 0.6  # s: how far out the windows may look
STEP = 0.01  # s: pitch and level frames
JOIN_REL = 24.0  # dB under the line's p95 level: silence at a join (T-007)
INNER_REL = 30.0  # dB under the line's p95 level: silence inside a sentence. B-C09 used 25 dB under Praat's smoothed intensity,
# which on these 20 ms mean-square frames is about 30: both find Video F's one unlicensed pause (06_2.1, 0.48 s)
CLAIM = 0.3  # s: a detected silence belongs to a join if it comes within this of the inserted gap
AGREE_ST = 3.0  # st: the two trackers agree on a reset when they are this close
MIN_JOINS = 20  # resets needed before the share beyond +12 st is judged at all
HI_VOICE_HZ = 165.0  # a median f0 above this uses the 100-500 Hz pitch range (T-007)

# the human calibration (RS-20261014-narration: T-007, audit-7, P-1, T-008, B-C03, B-C09)
BANDS = {
    "reset_beyond": 12.0,  # st
    "reset_share_human": (0.07, 0.12),  # pooled readers: T-007 4 of 55, audit-7 5 of 43; LibriVox alone 16% / 26%, R4 0%
    "reset_share_top": 0.26,  # flagged above the highest human reading; F 29-31%
    "reset_median": (6.0, 9.5),  # st, human readers +6.0 (scripted explainer) .. +9.5 (LibriVox); pooled +7.4
    "register_step": (1.7, 0.10),  # median |step| st and share beyond 4 st, human; F 2.6 and 29%
    "level_step_db": 6.7,  # human pooled median; F +2.7 (advisory only)
    "inner_fail_s": 1.0,
    "inner_flag_s": 0.3,
    "floor_dbfs": (-65.0, -56.0),  # human silences at joins; ACX allows up to -60 dB RMS
    "digital_black_dbfs": -75.0,  # F -89 dBFS with 98% exact zeros
    "gap_cv_min": 0.5,  # human 0.86 (inside sentences) .. 1.15 (sentence ends); F 0.19
    "terminal_rise_st": 9.0,
    "terminal_confirm_st": 4.5,
    "flat_share": 0.85,  # plain line gaps inside one 0.1 s band (F: 91% within 0.60-0.70 s)
    "class_gap_min_s": 0.2,  # topic change minus continuation (human 0.99 - 0.58 = 0.41 s)
    "line_gap_cv_min": 0.12,  # plain line gaps under 1.0 s CV (narration.rs MIN_GAP_CV = 0.12; F Constant ~0.04, Thought ~0.22)
    "join_copies_share": 0.7,  # chapter joins within 0.05 s of their median (F: 10 of 13)
}
GAP_TARGETS = {"TOPIC": 0.99, "BEAT": 0.81, "CONTINUATION": 0.58}  # s, P-1/P-4; the owner's A/B decides
GAP_TOLERANCE = 0.15

TOPIC = ("your turn", "take ", "here's", "last one", "one more", "most ", "the next")
CONTINUES = {"and", "but", "so", "then", "now", "because", "which", "plus", "or", "that's"}
LICENCE = re.compile(r"[,;:—–…]|\s-\s")  # a comma, colon, dash or ellipsis the voice may stop at


# ---------------------------------------------------------------- signal
def read_wav(path):
    with wave.open(str(path)) as w:
        rate, channels, width = w.getframerate(), w.getnchannels(), w.getsampwidth()
        raw = w.readframes(w.getnframes())
    if width != 2:
        raise ValueError(f"{path}: {8 * width}-bit WAV; 16-bit PCM expected")
    x = np.frombuffer(raw, dtype="<i2").astype(np.float64) / 32768.0
    return (x.reshape(-1, channels).mean(1) if channels > 1 else x), rate


def st(a, b):
    return 12.0 * math.log2(a / b)


def levels(x, rate):
    """(times, dB) of 20 ms frames every 10 ms: 10 log10 of the mean square, 1e-12 under silence"""
    n, h = int(0.02 * rate), int(STEP * rate)
    starts = np.arange(0, max(1, len(x) - n), h)
    c = np.concatenate(([0.0], np.cumsum(x * x)))
    end = np.minimum(starts + n, len(x))
    ms = (c[end] - c[starts]) / np.maximum(end - starts, 1)
    return np.arange(len(starts)) * STEP + STEP, 10.0 * np.log10(np.maximum(ms, 0.0) + 1e-12)


def runs_of(mask):
    """[(first, one past last)] of the True runs"""
    d = np.diff(np.concatenate(([0], np.asarray(mask, dtype=np.int8), [0])))
    return list(zip(np.flatnonzero(d == 1).tolist(), np.flatnonzero(d == -1).tolist()))


def pitch(x, rate, floor, ceil, kind):
    """(frame times, f0 with nan unvoiced) from Praat: kind 'ac' is T-007's default autocorrelation tracker, 'cc' the
    cross-correlation tracker with audit-7's settings"""
    import parselmouth
    snd = parselmouth.Sound(x, rate)
    if kind == "ac":
        p = snd.to_pitch_ac(time_step=STEP, pitch_floor=floor, pitch_ceiling=ceil)
    else:
        p = snd.to_pitch_cc(time_step=STEP, pitch_floor=floor, pitch_ceiling=ceil, very_accurate=True, silence_threshold=0.03,
                            voicing_threshold=0.5, octave_cost=0.05, octave_jump_cost=0.9, voiced_unvoiced_cost=0.2)
    f0 = p.selected_array["frequency"].copy()
    f0[f0 == 0] = np.nan
    return p.xs(), f0


def voice_bounds(x, rate):
    """the pitch range: 60-300 Hz, or 100-500 Hz when a wide first pass finds a median f0 above 165 Hz (T-007)"""
    _, f0 = pitch(x, rate, 60.0, 600.0, "ac")
    voiced = f0[~np.isnan(f0)]
    return (60.0, 300.0) if not len(voiced) or float(np.median(voiced)) <= HI_VOICE_HZ else (100.0, 500.0)


def edge_f0(tf, f0, t0, t1, end):
    """median f0 of the last (end) or first WIN seconds of the voiced frames in [t0, t1]; nan with fewer than 4 frames"""
    m = (tf >= t0) & (tf <= t1) & ~np.isnan(f0)
    tv, fv = tf[m], f0[m]
    if len(tv) < 4:
        return math.nan
    sel = tv >= tv[-1] - WIN if end else tv <= tv[0] + WIN
    return float(np.median(fv[sel])) if sel.sum() >= 4 else math.nan


class Track:
    """one waveform: both pitch trackers and the level frames (T-007's Track plus audit-7's second tracker)"""

    def __init__(self, x, rate, bounds=None):
        self.x, self.rate = x, rate
        self.bounds = bounds or voice_bounds(x, rate)
        self.pitch = {k: pitch(x, rate, self.bounds[0], self.bounds[1], k) for k in ("ac", "cc")}
        self.tl, self.lev = levels(x, rate)
        self.top = float(np.percentile(self.lev, 95)) if len(self.lev) else -120.0

    def edge(self, kind, t0, t1, end):
        tf, f0 = self.pitch[kind]
        return edge_f0(tf, f0, t0, t1, end)

    def median_f0(self, t0, t1, kind="ac"):
        tf, f0 = self.pitch[kind]
        v = f0[(tf >= t0) & (tf <= t1) & ~np.isnan(f0)]
        return float(np.median(v)) if len(v) >= 15 else math.nan

    def level_in(self, t0, t1):
        m = (self.tl >= t0) & (self.tl <= t1) & (self.lev > self.top - 30)
        return float(np.median(self.lev[m])) if m.sum() >= 3 else math.nan

    def silences(self, rel, minlen):
        """[(start, end, touches an end of the file)] of the runs of frames more than `rel` dB under the p95 level"""
        out = []
        low = self.lev < self.top - rel
        for s, i in runs_of(low):
            if (i - s) * STEP >= minlen - 1e-9:
                out.append((float(self.tl[s] - STEP), float(self.tl[i - 1] + STEP), s == 0 or i == len(low)))
        return out


def measure_join(tr, a, b, lo, hi):
    """a join whose silence runs from a to b between sentences spanning [lo, a] and [b, hi]: the reset per tracker and the level step"""
    row = {"gap": b - a}
    for kind in ("ac", "cc"):
        before = tr.edge(kind, max(a - SPAN, lo), a, True)
        after = tr.edge(kind, b, min(b + SPAN, hi), False)
        row[f"reset_{kind}"] = None if math.isnan(before) or math.isnan(after) else st(after, before)
    lb, la = tr.level_in(max(a - SPAN, lo), a), tr.level_in(b, min(b + SPAN, hi))
    row["level_db"] = None if math.isnan(lb) or math.isnan(la) else la - lb
    row.update(consensus(row["reset_ac"], row["reset_cc"]))
    return row


def consensus(ac, cc):
    """{'reset', 'status'}: the mean of the two trackers when both read and are within AGREE_ST ('agreed'); None when they are
    further apart ('disputed'), when only one reads ('single') or neither ('none')"""
    if ac is not None and cc is not None:
        agreed = abs(ac - cc) <= AGREE_ST
        return {"reset": 0.5 * (ac + cc) if agreed else None, "status": "agreed" if agreed else "disputed"}
    return {"reset": None, "status": "single" if (ac is not None or cc is not None) else "none"}


def claim_gaps(runs, gaps):
    """the detected silence of each inserted gap (e0, s1): the longest unclaimed one within CLAIM s of it, else the gap itself"""
    claimed, out = set(), []
    for e0, s1 in gaps:
        cand = [k for k, r in enumerate(runs) if k not in claimed and not r[2] and r[1] > e0 - CLAIM and r[0] < s1 + CLAIM]
        if cand:
            k = max(cand, key=lambda k: runs[k][1] - runs[k][0])
            claimed.add(k)
            out.append((runs[k][0], runs[k][1]))
        else:
            out.append((e0, s1))
    return out


def terminal_rise(seg, rate, kind):
    """B-C03's tail rise of one sentence, st: the median f0 of the voiced frames in the last 120 ms of speech minus those 150-400 ms
    before it, speech being the frames within 25 dB of the sentence's p95 intensity; nan when too few frames are voiced"""
    import parselmouth
    snd = parselmouth.Sound(seg, rate)
    kw = dict(time_step=STEP, pitch_floor=60.0, max_number_of_candidates=15, very_accurate=False, silence_threshold=0.03,
              voicing_threshold=0.45, octave_cost=0.01, octave_jump_cost=0.35, voiced_unvoiced_cost=0.14, pitch_ceiling=400.0)
    p = snd.to_pitch_ac(**kw) if kind == "ac" else snd.to_pitch_cc(**kw)
    f0 = p.selected_array["frequency"].copy()
    f0[f0 == 0] = np.nan
    tf = p.xs()
    it = snd.to_intensity(minimum_pitch=100.0, time_step=STEP)
    db = it.values[0]
    speech = np.flatnonzero(db > np.percentile(db, 95) - 25)
    if not len(speech) or np.isnan(f0).all():
        return math.nan
    t_last = float(it.xs()[speech[-1]])
    voiced = ~np.isnan(f0) & (tf <= t_last)
    end, mid = voiced & (tf >= t_last - 0.12), voiced & (tf >= t_last - 0.40) & (tf <= t_last - 0.15)
    if voiced.sum() < 12 or end.sum() < 3 or mid.sum() < 4:
        return math.nan
    return st(float(np.median(f0[end])), float(np.median(f0[mid])))


# ---------------------------------------------------------------- statistics
def wilson(k, n, z=1.96):
    """95% interval of a share of k in n"""
    if not n:
        return (math.nan, math.nan)
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (c - h, c + h)


def describe(v):
    """{n, median, p10, p90, cv, distinct (10 ms)} of a list of numbers; None for an empty one"""
    v = np.array([a for a in v if a is not None and not (isinstance(a, float) and math.isnan(a))], dtype=float)
    if not len(v):
        return None
    return {"n": int(len(v)), "median": float(np.median(v)), "p10": float(np.percentile(v, 10)), "p90": float(np.percentile(v, 90)),
            "cv": float(v.std() / v.mean()) if v.mean() > 0 else math.nan, "distinct": int(len(set(np.round(v, 2).tolist())))}


# ---------------------------------------------------------------- inputs
def read_tsv(path):
    out = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#") and "\t" in line:
            k, t = line.split("\t", 1)
            out[k] = t
    return out


def read_timeline(timeline_dir):
    """{chapter: {'fps', 'frames', 'lines': [(id, start s, length s)], 'listens', 'breaths', 'marks'}} from <chapter>.timeline.tsv and .cues.tsv"""
    out = {}
    for tl in sorted(Path(timeline_dir).glob("*.timeline.tsv")):
        chapter = tl.name[: -len(".timeline.tsv")]
        cues = tl.with_name(f"{chapter}.cues.tsv")
        if not cues.exists():
            raise ValueError(f"{cues} is missing")
        lengths = {}
        for line in cues.read_text(encoding="utf-8").splitlines():
            if line.strip():
                k, _, n = line.split("\t")
                lengths[k] = float(n)
        c = {"fps": 60, "frames": 0, "lines": [], "listens": [], "breaths": [], "marks": []}
        starts = []
        for line in tl.read_text(encoding="utf-8").splitlines():
            f = line.split("\t")
            if f[0] == "fps":
                c["fps"] = int(f[1])
            elif f[0] == "frames":
                c["frames"] = int(f[1])
            elif f[0] == "line":
                starts.append((f[1], int(f[2])))
            elif f[0] == "mark":
                c["marks"].append((f[1], int(f[2])))
            elif f[0] == "breath":
                c["breaths"].append((int(f[1]), float(f[2])))
            elif f[0] == "listen":
                c["listens"].append((int(f[1]), float(f[2])))
        if c["fps"] <= 0:
            raise ValueError(f"{tl}: fps {c['fps']} is not positive")
        c["lines"] = [(k, fr / c["fps"], lengths[k]) for k, fr in starts]
        out[chapter] = c
    return out


def gap_class(prev_text, next_text, window):
    """QUIZ / PREDICT / WINDOW when the timeline put one in the silence, else TOPIC / CONTINUATION / BEAT from the two lines' text (P-4)"""
    if window:
        return window
    nxt = next_text.strip().lower()
    first = re.sub(r"[^a-z']", "", nxt.split()[0]) if nxt.split() else ""
    if nxt.startswith(TOPIC):
        return "TOPIC"
    if first in CONTINUES or (prev_text.strip() and prev_text.strip()[-1] not in ".?!"):
        return "CONTINUATION"
    return "BEAT"


def line_gaps(timeline, spoken):
    """the silence between every two lines of each chapter (next start minus this line's end), its class, and each chapter's join to the
    next (this tail plus the next lead); tails and leads need the chapter length"""
    gaps, tails, leads = [], {}, {}
    for chapter, c in timeline.items():
        fps = c["fps"]
        lines = c["lines"]
        if not lines:
            continue
        leads[chapter] = lines[0][1]
        tails[chapter] = max(0.0, c["frames"] / fps - (lines[-1][1] + lines[-1][2])) if c["frames"] else None
        for (a, sa, la), (b, sb, _) in zip(lines, lines[1:]):
            ea = sa + la
            window = None
            if any(ea - 0.05 <= f / fps <= sb + 0.01 for f, _ in c["listens"]):
                window = "QUIZ"
            elif any(m == "predict" and ea - 0.05 <= f / fps <= sb + 0.01 for m, f in c["marks"]):
                window = "PREDICT"
            elif any(ea - 0.05 <= f / fps <= sb + 0.01 for f, _ in c["breaths"]):
                window = "WINDOW"
            if not spoken or (a in spoken and b in spoken):
                gaps.append({"chapter": chapter, "from": a, "to": b, "start": ea, "len": sb - ea, "class": gap_class(spoken.get(a, ""), spoken.get(b, ""), window)})
    names = list(timeline)
    joins = [{"from": a, "to": b, "len": tails[a] + leads[b]} for a, b in zip(names, names[1:]) if tails.get(a) is not None and b in leads]
    return gaps, joins


def load_lines(out):
    """[(line id, wav, sentences)] from report.json, each sentence with its [t0, t1) in the line and the gap that follows; the WAV length
    must be what the report says"""
    try:
        rep = json.loads((Path(out) / "report.json").read_text(encoding="utf-8"))
        rate, report_lines = rep["meta"]["rate"], rep["lines"]
    except (json.JSONDecodeError, KeyError, TypeError) as e:
        raise ValueError(f"{out}/report.json is not a narration report ({e!r})") from e
    if not isinstance(rate, int) or rate <= 0 or not isinstance(report_lines, dict):
        raise ValueError(f"{out}/report.json: expected a positive integer meta.rate and a lines object, got rate {rate!r}")
    lines = []
    for lid, sents in report_lines.items():
        pos, spans = 0, []
        for s in sents:
            n, g = s["out_samples"], s.get("gap_after_samples", 0)
            spans.append({"j": s["j"], "text": s["sentence"], "t0": pos / rate, "t1": (pos + n) / rate, "gap": g / rate})
            pos += n + g
        lines.append((lid, Path(out) / f"{lid}.wav", spans, pos))
    return rate, lines


# ---------------------------------------------------------------- one video
def scan(out, timeline_dir=None, script=None, bounds=None):
    """measure a narration out dir -> a dict of rows (joins, sentences, inner pauses, gaps, floors) for verdicts()"""
    out = Path(out)
    rate, lines = load_lines(out)
    spoken = {}
    if script:
        spoken = read_tsv(script)
    elif (out / "spoken.tsv").exists():
        spoken = read_tsv(out / "spoken.tsv")
    timeline = read_timeline(timeline_dir) if timeline_dir else {}
    chapter_of = {k: c for c, v in timeline.items() for k, _, _ in v["lines"]}
    joins, sentences, pauses, floors = [], [], [], []
    xs = {}
    for lid, wav, spans, total in lines:
        x, r = read_wav(wav)
        if r != rate or abs(len(x) - total) > 2:
            raise ValueError(f"{wav}: {len(x)} samples at {r} Hz, but report.json accounts for {total} at {rate} Hz")
        xs[lid] = x
    if bounds is None:
        probes = [voice_bounds(x, rate) for x in xs.values()]
        bounds = max(set(probes), key=probes.count) if probes else (60.0, 300.0)
    for lid, wav, spans, total in lines:
        x = xs[lid]
        chapter = chapter_of.get(lid, lid.split("_")[0])
        tr = Track(x, rate, bounds)
        first_join = len(joins)
        runs = tr.silences(JOIN_REL, 0.04)
        gaps = [(a["t1"], b["t0"]) for a, b in zip(spans, spans[1:])]
        own = claim_gaps(runs, gaps)
        lead = next((r[1] for r in runs if r[2] and r[0] <= 0.02), 0.0)
        tail = next((r[0] for r in runs if r[2] and r[1] >= len(x) / rate - 0.02), len(x) / rate)
        for k, ((a, b), s0, s1) in enumerate(zip(own, spans, spans[1:])):
            lo = own[k - 1][1] if k else lead
            hi = own[k + 1][0] if k + 1 < len(own) else tail
            row = measure_join(tr, a, b, lo, hi)
            row.update(chapter=chapter, line=lid, j=s0["j"], a=a, b=b, inserted=s0["gap"], prev=s0["text"], next=s1["text"])
            joins.append(row)
            if b - a >= 0.25:
                seg = x[int((a + 0.06) * rate): int((b - 0.06) * rate)]
                if len(seg) >= int(0.05 * rate):
                    floors.append({"line": lid, "j": s0["j"], "dbfs": float(10 * np.log10(np.mean(seg * seg) + 1e-12)),
                                   "zeros": float(np.mean(np.abs(seg) < 0.5 / 32768))})
        meds = []
        for s in spans:
            meds.append(tr.median_f0(s["t0"], s["t1"]))
            seg = x[int(s["t0"] * rate): int(s["t1"] * rate)]
            ac = terminal_rise(seg, rate, "ac") if len(seg) > 0.4 * rate else math.nan
            cc = terminal_rise(seg, rate, "cc") if not math.isnan(ac) and ac > BANDS["terminal_rise_st"] else math.nan
            event = (not math.isnan(ac) and ac > BANDS["terminal_rise_st"]
                     and not math.isnan(cc) and cc > BANDS["terminal_confirm_st"])
            sentences.append({"chapter": chapter, "line": lid, "j": s["j"], "text": s["text"], "median_hz": meds[-1],
                              "rise_ac": None if math.isnan(ac) else ac, "rise_cc": None if math.isnan(cc) else cc, "event": event,
                              "question": s["text"].rstrip().endswith("?")})
        for row, u, v in zip(joins[first_join:], meds, meds[1:]):
            row["step_st"] = None if math.isnan(u) or math.isnan(v) else st(v, u)
        # silences inside a sentence: not at a file edge and not the silence of a join
        for r0, r1, edge in tr.silences(INNER_REL, BANDS["inner_flag_s"]):
            if edge or any(r1 > a - 1e-6 and r0 < b + 1e-6 for a, b in own):
                continue
            mid = 0.5 * (r0 + r1)
            sent = next((s for s in spans if s["t0"] <= mid <= s["t1"]), None)
            if sent is None:
                continue
            pauses.append({"chapter": chapter, "line": lid, "j": sent["j"], "t": r0, "len": r1 - r0, "text": sent["text"],
                           "licensed": bool(LICENCE.search(sent["text"].rstrip()[:-1]))})
    gap_rows, chapter_joins = line_gaps(timeline, spoken) if timeline else ([], [])
    return {"rate": rate, "bounds": list(bounds), "lines": len(lines), "chapters": sorted({j["chapter"] for j in joins} | set(timeline)),
            "joins": joins, "sentences": sentences, "inner_pauses": pauses, "floors": floors, "line_gaps": gap_rows,
            "chapter_joins": chapter_joins, "timeline": bool(timeline)}


# ---------------------------------------------------------------- verdicts
def share_beyond(rows):
    """(resets beyond +12 st, resets, resets used) of the consensus resets of rows"""
    v = [r["reset"] for r in rows if r.get("reset") is not None]
    return sum(x > BANDS["reset_beyond"] for x in v), len(v)


def verdicts(res, gates=()):
    """[{check, scope, verdict, value, band, note, items}] for the whole video and, as ADVISORY rows, per chapter"""
    out = []

    def add(check, verdict, value, band, note="", items=(), scope="video"):
        out.append({"check": check, "scope": scope, "verdict": verdict, "value": value, "band": band, "note": note, "items": list(items)})

    def gated(name, found):
        """PASS, or FLAG - FAIL when the video opted in to the gate"""
        if not found:
            return "PASS"
        names = (name,) if isinstance(name, str) else name
        return "FAIL" if any(g in gates for g in names) else "FLAG"

    joins = res["joins"]
    k, n = share_beyond(joins)
    lo, hi = wilson(k, n)
    status = {s: sum(j["status"] == s for j in joins) for s in ("agreed", "disputed", "single", "none")}
    note = (f"{k} beyond +12 st of {n} resets that both trackers agree on (95% interval {lo * 100:.0f}-{hi * 100:.0f}%); "
            f"{status['disputed']} joins disputed, {status['single'] + status['none']} read by one tracker or none") if n else "no join both trackers read"
    beside = "human pooled 7-12% (4-5 joins), LibriVox alone 16-26%, R4 0%; flagged above 26%"
    if n < MIN_JOINS:
        add("reset_share", "ADVISORY", None if not n else k / n, beside, f"fewer than {MIN_JOINS} resets: {note}")
    else:
        add("reset_share", "FLAG" if k / n > BANDS["reset_share_top"] else "PASS", k / n, beside, note)
    d = describe([j["reset"] for j in joins])
    if d:
        a, b = BANDS["reset_median"]
        add("reset_median", "ADVISORY", d["median"], f"human +{a:.1f}..+{b:.1f} st", "inside the human range" if a <= d["median"] <= b else "outside the human range")
    for chapter in res["chapters"]:
        rows = [j for j in joins if j["chapter"] == chapter]
        kc, nc = share_beyond(rows)
        if nc:
            add("reset_share", "ADVISORY", kc / nc, f"{kc} of {nc} resets", scope=chapter)
    steps = [j["step_st"] for j in joins if j.get("step_st") is not None]
    if steps:
        big = float(np.mean(np.abs(steps) > 4))
        add("register_step", "ADVISORY", float(np.median(np.abs(steps))), f"human median {BANDS['register_step'][0]} st, {BANDS['register_step'][1] * 100:.0f}% beyond 4 st (F 2.6, 29%)",
            f"{big * 100:.0f}% beyond 4 st of {len(steps)} adjacent sentences")
    lev = [j["level_db"] for j in joins if j["level_db"] is not None]
    if lev:
        add("level_step", "ADVISORY", float(np.median(lev)), f"human median +{BANDS['level_step_db']} dB (F +2.7); never a gate",
            f"{np.mean(np.abs(lev) > 4) * 100:.0f}% beyond 4 dB of {len(lev)} joins")
    # pauses inside sentences
    long_ = [p for p in res["inner_pauses"] if p["len"] > BANDS["inner_fail_s"]]
    add("inner_pause_over_1s", "FAIL" if long_ else "PASS", len(long_), "none", items=[f"{p['line']}.{p['j']} {p['len']:.2f} s at {p['t']:.2f} s: {p['text']}" for p in long_])
    odd = [p for p in res["inner_pauses"] if p["len"] > BANDS["inner_flag_s"] and not p["licensed"] and p["len"] <= BANDS["inner_fail_s"]]
    add("inner_pause_unlicensed", "FLAG" if odd else "PASS", len(odd), "none (no comma, colon or dash to stop at)",
        items=[f"{p['line']}.{p['j']} {p['len']:.2f} s: {p['text']}" for p in odd])
    # the silence between sentences
    if res["floors"]:
        floor = float(np.median([f["dbfs"] for f in res["floors"]]))
        zero = float(np.mean([f["zeros"] >= 0.99 for f in res["floors"]]))
        black = floor < BANDS["digital_black_dbfs"] or zero >= 0.5
        add("silence_floor", gated("room-tone", black), floor, f"human {BANDS['floor_dbfs'][0]:.0f}..{BANDS['floor_dbfs'][1]:.0f} dBFS (F -89)",
            f"median of {len(res['floors'])} silences of 0.25 s or more, 60 ms in from their edges; {zero * 100:.0f}% are exact zeros" + ("; digital black" if black else ""))
    inline = describe([j["gap"] for j in joins])
    if inline and inline["n"] >= 12:
        flat = inline["cv"] < BANDS["gap_cv_min"] or inline["distinct"] <= 3
        add("gap_regularity", gated("regularity", flat), inline["cv"], f"CV >= {BANDS['gap_cv_min']} (human 0.86-1.15, F 0.19)",
            f"{inline['n']} sentence gaps: median {inline['median']:.2f} s, p10-p90 {inline['p10']:.2f}-{inline['p90']:.2f}, {inline['distinct']} distinct values")
    plain = [g for g in res["line_gaps"] if g["class"] in GAP_TARGETS]
    if len(plain) >= 10 or (any(g in gates for g in ("regularity", "line-gaps")) and len(plain) >= 3):
        v = np.array([g["len"] for g in plain])
        band = max(int(np.sum((v >= t) & (v < t + 0.1))) for t in np.arange(0.0, v.max() + 0.1, 0.05))
        by = {c: [g["len"] for g in plain if g["class"] == c] for c in GAP_TARGETS}
        med = {c: float(np.median(x)) for c, x in by.items() if x}
        under_1s = np.array([g["len"] for g in plain if g["len"] < 1.0], dtype=float)
        cv_1s = float(under_1s.std() / under_1s.mean()) if len(under_1s) >= 3 and under_1s.mean() > 0 else 0.0
        low_cv = len(under_1s) < 3 or cv_1s < BANDS["line_gap_cv_min"]
        flat = band / len(v) >= BANDS["flat_share"]
        undiff = len(by["TOPIC"]) >= 3 and len(by["CONTINUATION"]) >= 3 and med["TOPIC"] - med["CONTINUATION"] < BANDS["class_gap_min_s"]
        add("line_gap_regularity", gated(("regularity", "line-gaps"), flat or undiff or low_cv), band / len(v),
            f"under {BANDS['flat_share'] * 100:.0f}% in one 0.1 s band; topic - continuation >= {BANDS['class_gap_min_s']} s (human 0.41); plain CV >= {BANDS['line_gap_cv_min']}",
            "; ".join(f"{c} n={len(by[c])} median {med[c]:.2f} s" for c in med) + f"; {band} of {len(v)} plain gaps within 0.1 s of each other; plain CV {cv_1s:.2f}")
        if "gap-targets" in gates:
            off = [c for c in med if abs(med[c] - GAP_TARGETS[c]) > GAP_TOLERANCE]
            add("line_gap_targets", "FAIL" if off else "PASS", None, ", ".join(f"{c} {t:.2f} s" for c, t in GAP_TARGETS.items()) + f" +/- {GAP_TOLERANCE} s",
                "off target: " + ", ".join(f"{c} {med[c]:.2f}" for c in off) if off else "on target")
        waits = [g for g in plain if g["len"] > 1.0]
        if waits:
            add("line_gap_over_1s", "ADVISORY", len(waits), "picture waits, listed", items=[f"{g['from']}->{g['to']} {g['len']:.2f} s ({g['class']})" for g in waits])
    cj = [j["len"] for j in res["chapter_joins"]]
    if len(cj) >= 5:
        m = float(np.median(cj))
        same = sum(abs(x - m) <= 0.05 for x in cj)
        add("chapter_join_regularity", gated("regularity", same / len(cj) >= BANDS["join_copies_share"]), same / len(cj),
            f"under {BANDS['join_copies_share'] * 100:.0f}% within 0.05 s of the median", f"{same} of {len(cj)} chapter joins within 0.05 s of {m:.2f} s")
    events = [s for s in res["sentences"] if s["event"] and not s["question"]]
    readable = sum(s["rise_ac"] is not None for s in res["sentences"])
    add("terminal_events", gated("terminal", bool(events)), len(events), f"0 (B-C03: 10 of 274 in Video F); {BANDS['terminal_rise_st']:.0f} st rise confirmed over {BANDS['terminal_confirm_st']} st",
        f"of {len(res['sentences'])} sentences, {readable} with a readable tail", items=[f"{s['line']}.{s['j']} +{s['rise_ac']:.0f}/+{s['rise_cc']:.0f} st: {s['text']}" for s in events])
    unsure = [s for s in res["sentences"] if (s["rise_ac"] or 0) > BANDS["terminal_rise_st"] and not s["event"] or (s["event"] and s["question"])]
    if unsure:
        add("terminal_unconfirmed", "ADVISORY", len(unsure), "second tracker does not confirm, or a question (a rising end is its own)",
            items=[f"{s['line']}.{s['j']} autocorrelation +{s['rise_ac']:.0f}, cross-correlation " + ("unread" if s["rise_cc"] is None else f"{s['rise_cc']:+.0f}") + f" st: {s['text']}" for s in unsure])
    return out


def render(res, checks):
    lines = [f"prosody QA: {res['lines']} lines, {len(res['chapters'])} chapters, {len(res['joins'])} sentence joins; Praat autocorrelation + cross-correlation, "
             f"{res['bounds'][0]:.0f}-{res['bounds'][1]:.0f} Hz" + ("" if res["timeline"] else "; no --timeline: line gaps not measured")]
    percent = ("reset_share", "line_gap_regularity", "chapter_join_regularity")
    for c in checks:
        if c["scope"] != "video":
            continue
        v = c["value"]
        if v is None:
            v = ""
        elif c["check"] in percent:
            v = f"{v * 100:.0f}%"
        else:
            v = str(v) if isinstance(v, int) else f"{v:.2f}"
        lines.append(f"{c['verdict']:8s} {c['check']:24s} {v:>7s}  [{c['band']}]  {c['note']}".rstrip())
        lines += [f"           {i}" for i in c["items"]]
    chapters = [c for c in checks if c["scope"] != "video"]
    if chapters:
        lines.append("resets beyond +12 st by chapter (advisory):")
        lines += [f"  {c['scope']:32s} {c['value'] * 100:3.0f}%  {c['band']}" for c in chapters]
    return "\n".join(lines)


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("out", help="narration out dir: report.json and the line WAVs")
    ap.add_argument("--timeline", help="renders/scored or renders/narrated: <chapter>.timeline.tsv and .cues.tsv")
    ap.add_argument("--script", help="id<TAB>text file for the line gaps (default OUT/spoken.tsv)")
    ap.add_argument("--json", help="write every row and verdict here")
    ap.add_argument("--pitch-range", help="LO,HI in Hz (default 60,300 or 100,500 by the voice)")
    for g in ("room-tone", "regularity", "line-gaps", "terminal", "gap-targets"):
        ap.add_argument(f"--gate-{g}", action="store_true", help=f"make the {g} flag a FAIL")
    a = ap.parse_args(argv)
    try:
        bounds = tuple(float(v) for v in a.pitch_range.split(",")) if a.pitch_range else None
        res = scan(a.out, a.timeline, a.script, bounds)
    except (OSError, ValueError, KeyError) as e:
        print(f"prosody_qa: {e}", file=sys.stderr)
        return 2
    gates = tuple(g for g in ("room-tone", "regularity", "line-gaps", "terminal", "gap-targets") if getattr(a, f"gate_{g.replace('-', '_')}"))
    checks = verdicts(res, gates)
    print(render(res, checks))
    if a.json:
        Path(a.json).write_text(json.dumps({"result": res, "checks": checks}, indent=1, default=lambda o: None if (isinstance(o, float) and math.isnan(o)) else float(o)), encoding="utf-8")
    return 1 if any(c["verdict"] == "FAIL" for c in checks) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
