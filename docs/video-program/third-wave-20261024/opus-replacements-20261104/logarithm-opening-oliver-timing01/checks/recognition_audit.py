"""Timing audit of the opening F1 and F2 beats against the retained Oliver takes.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-oliver-timing01/checks/recognition_audit.py [--write]

Reads the retained WAVs, the two recognition logs, the takes records and script.tsv. Runs no recognizer, synthesis,
build or render. Without --write it compares its result to recognition-candidates.json; with --write it writes that
one file beside this folder's handoff.

Three kinds of time appear here, and none of them was heard:
  recognizer  the word row's start in the retained whisper-cpp log.
  envelope    a mechanical reading of level and zero-crossing rate. The rules are below; each candidate names its rule.
              Sound names such as /f/ are inferences from those two numbers.
  share       the source's character share, on the report-style sentence start.
"""
import array
import difflib
import hashlib
import json
import math
import re
import sys
import wave
from pathlib import Path

REPO = Path.cwd()
HERE = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-oliver-timing01"
R = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
B = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SCRIPT = REPO / "explainer_corpus/logarithm_mysteries/script.tsv"
SCRIPT_SHA = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"
STAGE07_SCRIPT = REPO / R / "combined-repair-stage07/candidate/explainer_corpus/logarithm_mysteries/script.tsv"
STAGE07_SHA = "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e"
JOBS = {"opening-recognition01": "logarithm-opening-recognition01", "middle-recognition01": "logarithm-middle-recognition01"}
CUES = ("00_1", "00_2", "03_3")
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def rel(path):
    return path[path.index("narration_lab/") :]


print("-- script and takes")
script_bytes = SCRIPT.read_bytes()
check(sha(script_bytes) == SCRIPT_SHA, f"script.tsv is {SCRIPT_SHA[:16]}")
script = dict(row.split("\t", 1) for row in script_bytes.decode().splitlines() if "\t" in row and not row.startswith("#"))
stage07_bytes = STAGE07_SCRIPT.read_bytes()
stage07 = dict(row.split("\t", 1) for row in stage07_bytes.decode().splitlines() if "\t" in row and not row.startswith("#"))
check(sha(stage07_bytes) == STAGE07_SHA and all(stage07[c] == script[c] for c in CUES), f"root's stage07 script is {STAGE07_SHA[:16]}, with the same text in {CUES}")

takes = []
for folder, job in JOBS.items():
    rows = json.loads((REPO / R / folder / "takes.json").read_text())["takes"]
    words = {}
    for line in (REPO / B / job / "command.log").read_text().splitlines():
        row = json.loads(line)
        words[Path(row["input"]).name] = row
    for t in rows:
        if t["cue"] in CUES:
            name = Path(t["wav"]).name
            takes.append(dict(folder=folder, job=job, cue=t["cue"], name=name, rec=t, log=words[name]))
    receipt = json.loads((REPO / B / job / "resource-receipt.json").read_text())
    cmd = receipt["command"]
    check(
        receipt["exit"] == 0
        and receipt["inputs_sha256"] == receipt["final_inputs_sha256"]
        and "whisper-cpp" in cmd
        and "--split-on-word" in cmd
        and cmd[cmd.index("--max-segment-length") + 1] == "1"
        and Path(cmd[cmd.index("--model") + 1]).name == "ggml-large-v3-turbo.bin",
        f"{job}: exit 0, inputs unchanged, whisper-cpp large-v3-turbo, one word per row",
    )
comparison = json.loads((REPO / R / "opening-recognition01/recognition-comparison01.json").read_text())
check(
    comparison["log_sha256"] == sha((REPO / B / "logarithm-opening-recognition01/command.log").read_bytes()),
    "the opening comparison binds the log read here",
)
print("  comparison limits, quoted:", comparison["limits"])

loaded = {}
for t in takes:
    raw = (REPO / R / "ui-takes01" / t["name"]).read_bytes()
    with wave.open(str(REPO / R / "ui-takes01" / t["name"]), "rb") as w:
        channels, width, rate, n = w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes()
        pcm = w.readframes(n)
    t.update(rate=rate, seconds=n / rate, wav_sha256=sha(raw), pcm_sha256=sha(pcm))
    loaded[t["name"]] = (channels, width, rate, pcm)
    downloads = json.loads((REPO / R / "ui-takes01" / f"{t['cue']}-downloads01.json").read_text())["downloads"]
    match = [d for d in downloads if Path(d["retained"]).name == t["name"]]
    request = json.loads((REPO / R / "ui-takes01" / f"{t['cue']}-request01.json").read_text())
    good = (
        t["rec"]["text"] == script[t["cue"]]
        and t["wav_sha256"] == t["rec"]["wav_sha256"]
        and len(match) == 1
        and match[0]["sha256"] == t["wav_sha256"]
        and channels == 1
        and width == 2
        and rate == 48000
        and request["text"] == script[t["cue"]]
        and request["voice"] == "Oliver - Clean, British and Steady"
        and request["model"] == "Eleven v4"
        and request["listening"] is False
        and request["fine_sync"] is False
        and t["log"]["status"] == "ok"
    )
    check(good, f"{t['name']}: bytes, text and Oliver request record agree; {t['seconds']:.2f} s; listening false, fine sync false")
    print(f"    wav {t['wav_sha256']}  pcm {t['pcm_sha256']}")
for cue in CUES:
    pair = [t for t in takes if t["cue"] == cue]
    check(len(pair) == 2 and pair[0]["pcm_sha256"] != pair[1]["pcm_sha256"], f"{cue}: two takes with different audio; neither is selected")
selection = [p for p in (REPO / R).glob("**/*") if p.name == "report.json" or p.name == "phrase-timings.json"]
check(not selection, "no report.json or phrase-timings.json under this rebuild's folder yet, so no take's sentence starts are bound")


print("\n-- what the recognizer's rows are")
for t in takes:
    segs = t["log"]["report"]["result"]["segments"]
    t["segs"] = segs
    touching = sum(1 for a, b in zip(segs, segs[1:]) if abs(a["end_sec"] - b["start_sec"]) < 1e-6)
    short = [s["text"] for s in segs if s["end_sec"] - s["start_sec"] < 0.1]
    conf = [s["confidence"] for s in segs]
    print(f"  {t['name']}: {len(segs)} rows, {touching} of {len(segs) - 1} touch the next row, under 0.1 s: {short}, confidence {min(conf):.3f}–{max(conf):.3f}")
print("  Every row touches its neighbour, so a word's start also carries any silence before it. The confidence field is")
print("  shared across runs of words and stays near 0.03, so it says nothing about one word.")


FRAME, HOP = 0.020, 0.010


def envelope(name):
    channels, width, rate, pcm = loaded[name]
    a = array.array("h")
    a.frombytes(pcm)
    frame, hop = int(rate * FRAME), int(rate * HOP)
    level, cross = [], []
    for i in range(0, len(a) - frame, hop):
        f = a[i : i + frame]
        level.append(10 * math.log10(sum(x * x for x in f) / frame / 32768.0**2 + 1e-12))
        cross.append(sum(1 for p, q in zip(f, f[1:]) if (p < 0) != (q < 0)) / (FRAME * 1000))
    top = max(level)
    return [v - top for v in level], cross


def quiet_runs(level, below=30.0, shortest=0.03):
    runs, start = [], None
    for k, v in enumerate(level + [0.0]):
        q = v < -below
        if q and start is None:
            start = k
        if not q and start is not None:
            if (k - start) * HOP >= shortest:
                runs.append((round(start * HOP, 2), round(k * HOP, 2)))
            start = None
    return runs


def rule_q(runs, at, reach=0.08):
    """Quiet run: the run of level 30 dB under the take's top nearest to `at`, within reach. Ties go to the longer run."""
    best = None
    for lo, hi in runs:
        d = 0.0 if lo <= at <= hi else min(abs(at - lo), abs(at - hi))
        if d <= reach + 1e-9:
            key = (round(d, 2), -(hi - lo))
            if best is None or key < best[0]:
                best = (key, (lo, hi))
    return best[1] if best else None


def rule_f(level, cross, after, within=0.5, floor=-32.0, rate=7.0, frames=5):
    """Fricative: the first 5 frames in a row, 50 ms, with 7 or more zero crossings per ms and level above -32 dB."""
    start = int(round(after / HOP))
    for k in range(start, min(len(level) - frames, start + int(within / HOP))):
        if all(cross[k + j] >= rate and level[k + j] >= floor for j in range(frames)):
            return round(k * HOP, 2)
    return None


def rule_v(level, at, within=0.5):
    """Vowel rise: the first frame at or above -6 dB that follows a dip to -15 dB or lower in the 60 ms before it."""
    start = int(round(at / HOP))
    for k in range(start, min(len(level), start + int(within / HOP))):
        if level[k] >= -6.0 and min(level[max(0, k - 6) : k] or [0.0]) <= -15.0:
            return round(k * HOP, 2)
    return None


for t in takes:
    t["level"], t["cross"] = envelope(t["name"])
    t["runs"] = quiet_runs(t["level"])


def word(t, text, nth=0):
    hits = [s for s in t["segs"] if s["text"].strip() == text]
    return hits[nth]


print("\n-- controls for the fricative rule, on script words whose first sound is known")
for name, text, nth, want in (
    ("00_1-download01.wav", "seven", 0, True),
    ("00_1-download02.wav", "seven", 0, True),
    ("00_1-download01.wav", "second.", 0, True),
    ("00_1-download02.wav", "second.", 0, True),
    ("03_3-download01.wav", "say", 0, True),
    ("03_3-download02.wav", "say", 0, True),
    ("00_1-download01.wav", "low", 0, False),
    ("00_1-download02.wav", "low", 0, False),
    ("03_3-download01.wav", "dial", 0, False),
    ("03_3-download02.wav", "dial", 0, False),
    ("03_3-download01.wav", "many", 0, False),
    ("03_3-download02.wav", "many", 0, False),
):
    t = next(x for x in takes if x["name"] == name)
    s = word(t, text, nth)["start_sec"]
    got = rule_f(t["level"], t["cross"], max(0.0, s - 0.1), within=0.25)
    check((got is not None) == want, f"{name} {text!r} at {s:.2f}: {'expects' if want else 'expects no'} fricative start within 0.25 s, got {got}")


def text_share(sentence, phrase):
    return len(sentence[: sentence.index(phrase)]) / len(sentence)


TEXT = script["00_1"]
SENTENCE = TEXT[TEXT.index(". ") + 2 :]
READ = (
    ("their frequencies", "their", "q"),
    ("four hundred and forty", "440,", "q"),
    ("eight hundred and eighty", "880,", "q"),
    ("seventeen sixty", "1760,", "q"),
    ("thirty-five twenty", "3520", "f"),
    ("vibrations per second", "vibrations", "v"),
)
shares = [text_share(SENTENCE, p) for p, _, _ in READ]

print("\n-- positive control: this mirror of read_shares and read_out gives the sealed scratch times")
scratch = [round(5.368 + (19.077 - 5.368) * s, 2) for s in shares]
check(scratch == [6.63, 11.73, 13.32, 15.04, 16.36, 17.62], f"scratch arrivals {scratch}")


print("\n-- F1, 00_1 sentence 1: seconds from the cue's start. Late is positive.")
rows = []
worst_share = worst_word = 0.0
biggest_early = 0.0
for t in [x for x in takes if x["cue"] == "00_1"]:
    s1_word = word(t, "In")["start_sec"]
    gap = rule_q(t["runs"], s1_word)
    end = t["seconds"]
    print(f"  {t['name']}: recognizer sentence start {s1_word:.2f}, quiet run {gap}, length {end:.2f}")
    print("    phrase                     token      recognizer  envelope (rule)   share    share-env  rec-env  share if s1 is the run's end")
    for (phrase, token, kind), share in zip(READ, shares):
        w = word(t, token)
        rec = w["start_sec"]
        if kind == "q":
            run = rule_q(t["runs"], rec)
            env, basis = run[1], f"end of quiet run {run[0]:.2f}-{run[1]:.2f}"
        elif kind == "f":
            env, basis = rule_f(t["level"], t["cross"], rec - 0.1), "fricative start; no quiet run before it"
        else:
            env, basis = rule_v(t["level"], rec), "vowel rise; the voiced /v/ starts up to 0.1 s before it"
        at = s1_word + (end - s1_word) * share
        at2 = gap[1] + (end - gap[1]) * share
        worst_share = max(worst_share, abs(at - env))
        worst_word = max(worst_word, abs(rec - env))
        biggest_early = min(biggest_early, rec - env)
        print(f"    {phrase:26} {token:10} {rec:7.2f}    {env:7.2f} ({kind})   {at:7.2f}  {at - env:+7.2f}  {rec - env:+7.2f}  {at2:7.2f}")
        rows.append(
            dict(
                beat="F1",
                cue="00_1",
                take=t["name"],
                wav_sha256=t["wav_sha256"],
                pcm_sha256=t["pcm_sha256"],
                phrase=phrase,
                byte=len(TEXT[: TEXT.index(phrase)].encode()),
                recognizer_token=token,
                recognizer_start=round(rec, 3),
                recognizer_end=round(w["end_sec"], 3),
                envelope_start=env,
                envelope_basis=basis,
                character_share_arrival=round(at, 3),
                sentence_start_used=round(s1_word, 3),
            )
        )
print(f"  largest |share - envelope| {worst_share:.2f} s; largest |recognizer - envelope| {worst_word:.2f} s; recognizer earliest {biggest_early:+.2f} s")
f1 = [r for r in rows if r["beat"] == "F1"]
close = [r["phrase"] for r in f1 if abs(r["character_share_arrival"] - r["envelope_start"]) < 0.08]
far = [r["phrase"] for r in f1 if abs(r["character_share_arrival"] - r["envelope_start"]) >= 0.08]
check(len(f1) == 12 and len(close) == 10 and set(far) == {"seventeen sixty"}, f"{len(close)} of {len(f1)} shares are within 0.08 s of the envelope; the others are {far}")
check(all(r["recognizer_start"] < r["envelope_start"] for r in f1), "the recognizer is earlier than the envelope on all 12")
for t in [x for x in takes if x["cue"] == "00_1"]:
    run = rule_q(t["runs"], word(t, "1760,")["start_sec"])
    after = next(r for r in t["runs"] if r[0] > run[1] + 0.5)
    print(f"  {t['name']}: 'seventeen sixty' spans about {run[1]:.2f}-{after[0]:.2f}, {after[0] - run[1]:.2f} s")
for t in [x for x in takes if x["cue"] == "00_1"]:
    w = word(t, "55,")["start_sec"]
    run = rule_q(t["runs"], w, reach=0.12)
    env = run[1] if run else rule_f(t["level"], t["cross"], w)
    low = word(t, "In")["start_sec"] + (t["seconds"] - word(t, "In")["start_sec"]) * shares[0]
    print(f"  {t['name']}: low label at {low:.2f}; 'fifty-five' recognizer {w:.2f}, envelope {env:.2f}; label leads the spoken 55 by {env - low:.2f} s, by design")
    print(f"    title alone from 1.20 s to {0.6 + low:.2f} s on the scene clock if 00_1 still starts at 0.60: {0.6 + low - 1.2:.2f} s (scratch 5.43 s)")


print("\n-- F1, the existing aligner: golay's phrase_marks.py logic, reimplemented here, on these rows. Not root's run, not marks.")


def tokens(text):
    return list(re.finditer(r"[A-Za-z]+(?:'[A-Za-z]+)?|[0-9]+", text))


def number(n):
    ones = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
    tens = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()
    if n < 20:
        return [ones[n]]
    if n < 100:
        return [tens[n // 10]] + ([ones[n % 10]] if n % 10 else [])
    if n < 1000:
        return [ones[n // 100], "hundred"] + (number(n % 100) if n % 100 else [])
    return [str(n)]


def aligned(cue, t):
    canonical = [(m.group().lower(), len(script[cue][: m.start()].encode())) for m in tokens(script[cue])]
    observed = []
    for s in t["segs"]:
        for w in tokens(s["text"]):
            v = w.group().lower()
            for e in number(int(v)) if v.isdigit() else [v]:
                observed.append((e, s["start_sec"]))
    ops = difflib.SequenceMatcher(a=[w for w, _ in canonical], b=[w for w, _ in observed], autojunk=False).get_opcodes()
    match = {i + k: j + k for op, i, e, j, _ in ops if op == "equal" for k in range(e - i)}
    return {canonical[i][1]: observed[j][1] for i, j in sorted(match.items())}


source = (REPO / "narration_lab/third-wave-20261024/root/elevenlabs-golay-20261028/phrase_marks.py").read_text()
check(
    "return [str(n)]" in source and "autojunk=False" in source and "automatic recognition estimates, not heard boundaries" in source,
    "phrase_marks.py still has the logic mirrored here, and calls its output recognition estimates",
)
for t in [x for x in takes if x["cue"] == "00_1"]:
    marks = aligned("00_1", t)
    row = []
    for phrase, _, _ in READ:
        b = len(TEXT[: TEXT.index(phrase)].encode())
        row.append(f"{phrase!r} {marks.get(b, 'NO MARK')}")
    print(f"  {t['name']}: " + "; ".join(row))
    check(
        len(TEXT[: TEXT.index("seventeen sixty")].encode()) not in marks and len(TEXT[: TEXT.index("thirty-five twenty")].encode()) not in marks,
        f"{t['name']}: that logic gives no mark for the two spoken thousands, because 1760 and 3520 stay digits",
    )


print("\n-- F1, the ruler in 00_2: axis 1.00 + dots 7 x 0.18 + lead 0.20 = 2.46 s after the cue starts")
for t in [x for x in takes if x["cue"] == "00_2"]:
    s1 = word(t, "The")["start_sec"]
    run = rule_q(t["runs"], s1)
    print(f"  {t['name']}: 'frequency.' voice ends near {run[0]:.2f}, sentence 1 recognizer {s1:.2f}, envelope {run[1]:.2f}; ruler whole at 2.46, {s1 - 2.46:.2f} s before the recognizer's sentence start")
    check(2.46 < run[0] < s1, f"{t['name']}: the ruler is whole before the first sentence's voice ends")


print("\n-- F2, 03_3 sentence 0: 'The dial remembers the digits, and forgets the laps.'")
DIGITS_SHARE, DIGITS_FADE, LAPS_RUN_SHORTEST, LAY_SHORTEST, LAY_UNTIMED = 0.58, 0.3, 0.6, 1.0, 2.0


def lay_beats(now, spoken, forgets=None):
    if spoken is None:
        return LAPS_RUN_SHORTEST, LAY_UNTIMED
    s0, s1 = spoken
    mark = None if forgets is None else s0 + forgets
    mark = mark if mark is not None and s0 <= mark < s1 else None
    named = s0 + (s1 - s0) * DIGITS_SHARE if mark is None else mark - DIGITS_FADE
    laps = max(named - now, LAPS_RUN_SHORTEST)
    return laps, max(s1 - now - laps - DIGITS_FADE, LAY_SHORTEST)


print("-- positive control: this mirror of lay_beats gives the sealed handoff's table and scratch beats")
table = [(3.6, 1.59, 1.21), (3.2, 1.36, 1.04), (3.0, 1.24, 1.00), (2.6, 1.01, 1.00)]
check(all(tuple(round(v, 2) for v in lay_beats(0.5, (0.0, a))) == (b, c) for a, b, c in table), f"handoff table {table}")
scratch_laps, scratch_lay = lay_beats(42.367, (41.867, 45.871))
check((round(42.367 + scratch_laps, 2), round(scratch_lay, 2)) == (44.19, 1.38), f"scratch: digits go at {42.367 + scratch_laps:.2f}, lay {scratch_lay:.2f} s")

text03 = script["03_3"]
byte_forgets = len(text03[: text03.index("forgets")].encode())
byte_and = len(text03[: text03.index("and forgets")].encode())
check(byte_forgets == 35 and byte_and == 31, f"source bytes: 'and' {byte_and}, 'forgets' {byte_forgets}")
first03 = text03[: text03.index(". ") + 1]
named03 = "The dial remembers the digits,"
check(first03.startswith(named03) and len(named03) == 30 and len(first03) == 52, f"{len(named03)} of {len(first03)} characters is {30 / 52:.3f}; DIGITS_SHARE rounds it to {DIGITS_SHARE}")
f2_ok = []
for t in [x for x in takes if x["cue"] == "03_3"]:
    d, an, fo = word(t, "digits"), word(t, "and"), word(t, "forgets")
    s1 = word(t, "To")["start_sec"]
    gap_s1 = rule_q(t["runs"], s1)
    comma = next(r for r in t["runs"] if r[0] > d["start_sec"] and r[1] - r[0] >= 0.06)
    sibilant = None
    for k in range(int(comma[0] / HOP), int(d["start_sec"] / HOP), -1):
        if t["cross"][k] >= 7.0:
            sibilant = round((k + 2) * HOP, 2)
            break
    f_on = rule_f(t["level"], t["cross"], comma[1])
    print(f"  {t['name']}")
    print(f"    recognizer: digits {d['start_sec']:.2f}-{d['end_sec']:.2f}, and {an['start_sec']:.2f}-{an['end_sec']:.2f}, forgets {fo['start_sec']:.2f}-{fo['end_sec']:.2f}, 'To' {s1:.2f}")
    print(f"    envelope: high-crossing run ends {sibilant}, comma quiet run {comma}, next fricative start {f_on}, voice before sentence 1 ends {gap_s1[0]:.2f}, sentence 1 run ends {gap_s1[1]:.2f}")
    print(f"    the recognizer's 'digits' ends {comma[0] - d['end_sec']:.2f} s before the comma's quiet run; its 'and' starts inside the same word's last sound")
    now = 0.5
    for label, mark in (("share, today", None), ("mark on 'forgets'", fo["start_sec"]), ("rejected: mark on 'and' as the fade's start", an["start_sec"] + DIGITS_FADE)):
        laps, lay = lay_beats(now, (0.0, s1), mark)
        go = now + laps
        over = max(0.0, go + DIGITS_FADE - f_on)
        print(
            f"      {label:44} laps {now:.2f}-{go:.2f} ({laps:.2f}), digits {go:.2f}-{go + DIGITS_FADE:.2f}, lay {lay:.2f} to {go + DIGITS_FADE + lay:.2f}; "
            f"fade start vs digits' end {go - sibilant:+.2f}, fade end vs /f/ {go + DIGITS_FADE - f_on:+.2f}, overlap with 'forgets' {over:.2f}"
        )
        if label.startswith("share"):
            f2_ok.append(("share", over))
            at_share = go
        if label.startswith("mark"):
            f2_ok.append(("mark", over))
            at_mark = go
    speech_only = gap_s1[0] * 30 / 52
    print(f"    30/52 of the speech alone, without the pause before sentence 1 ({gap_s1[1] - gap_s1[0]:.2f} s of quiet): {speech_only:.2f}, inside or at the comma run: {comma[0] - 0.05 <= speech_only <= comma[1] + 0.05}")
    marks = aligned("03_3", t)
    check(marks.get(byte_forgets) == fo["start_sec"], f"{t['name']}: the aligner logic gives byte {byte_forgets} the recognizer's 'forgets', {marks.get(byte_forgets):.2f}")
    print(f"    the same logic gives byte {byte_and} ('and') {marks.get(byte_and)}")
    rows.append(
        dict(
            beat="F2",
            cue="03_3",
            take=t["name"],
            wav_sha256=t["wav_sha256"],
            pcm_sha256=t["pcm_sha256"],
            phrase="forgets",
            byte=byte_forgets,
            recognizer_token="forgets",
            recognizer_start=round(fo["start_sec"], 3),
            recognizer_end=round(fo["end_sec"], 3),
            envelope_start=f_on,
            envelope_basis="fricative start after the comma's quiet run",
            digits_word_envelope_end=sibilant,
            comma_quiet_run=list(comma),
            digits_fade_start_share=round(at_share, 3),
            digits_fade_start_mark=round(at_mark, 3),
            sentence_start_used=round(s1, 3),
        )
    )
check(all(o > 0.2 for k, o in f2_ok if k == "share"), "on both takes the share-timed digits fade overlaps 'forgets' by more than 0.2 s")
check(all(o < 0.01 for k, o in f2_ok if k == "mark"), "on both takes a recognizer mark on 'forgets' ends the digits fade before the /f/ start read here")


print("\n-- candidates file")
data = {
    "claim": "Source-bound recognition candidates, for root. NOT a phrase-timings.json and NOT marks: no take is selected, nothing was heard, and every seconds value below is a recognizer row, a level and crossing reading, or a character share.",
    "script_sha256": {"canonical": SCRIPT_SHA, "stage07": STAGE07_SHA},
    "cue_text_sha256": {c: sha(script[c].encode()) for c in CUES},
    "voice": "ElevenLabs Oliver - Clean, British and Steady, Eleven v4, as the request records say",
    "recognition": {k: f"{B}/{v}/command.log" for k, v in JOBS.items()},
    "times": "Seconds from the start of the retained take. They hold for an assembled cue WAV only if its samples keep the take's positions, with no pause cut.",
    "rows": rows,
}
text = json.dumps(data, indent=1) + "\n"
path = HERE / "recognition-candidates.json"
if "--write" in sys.argv:
    path.write_text(text, encoding="utf-8")
check(path.is_file() and path.read_text(encoding="utf-8") == text, f"recognition-candidates.json matches this run, {len(rows)} rows")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
