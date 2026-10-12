#!/usr/bin/env python3
"""The narration skill's `plan` and `choose`, with reads that finish their last
word and an editor that listens to what it cuts.

    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS plan \
        SCRIPT.tsv WORK [--alts alts.tsv] [--lexicon lexicon.json] [--voice V] [--model M]
    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS choose \
        SCRIPT.tsv WORK OUT [--lexicon lexicon.json] [--sentence-lufs -23] [--limit-headroom 15]

    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS edges \
        SCRIPT.tsv OUT [--lexicon lexicon.json]

`choose` exits 1 when the limiter had to work harder than LIMITER_MAX (the lines are still written).
`edges` (after asr_batch.py has written OUT/lines_asr.json) exits 1 when a sentence's first or
last word is missing from its line's transcript: the words an edit can cut off, and one word in
a long line does not fail the line gate.

SKILL_SCRIPTS is the narration skill's scripts directory (narrate.py lives
there). Everything else is narrate.py's own: the respellings and alternates,
the score's weights, the gaps, the limiter, the report and its length proof.

Why every sentence is also read in quotation marks (`plan`). ftts often stops
a plain sentence inside its last word: of the fish video's 241 chosen reads,
158 were still within 25 dB of speech level 50 ms before the WAV ended, and
48 within 15 dB ("...halves the range." stopped on a rising "rai-", "...red
spots." in the middle of its "s"). Whisper writes the whole word anyway. The
same sentence inside “ ” is read to its end, with a falling close and
silence after it. So each read is planned twice, plain and quoted (the quotes
go to ftts only, like a respelling: the spoken text and the captions do not
change), and `choose` marks a read that ftts cut short and scores it down.

Two optional files beside the script are the director's (see director_files):
reads.tsv adds ftts inputs for a sentence without changing its text, and
picks.tsv names the read that wins when the director's eyes and the score
disagree.

Why the ear is replaced (`choose`). narrate.py calls a frame silent when it is under
-48 dBFS and ends a read at its last frame above that. ftts voices sit near
-34 LUFS, so that line is about 20 dB under the vowels, and three things
followed, each seen in spectrograms of the chosen takes:

1. ftts sometimes keeps generating after a short sentence: noise bursts,
   clicks and breaths for seconds. Whisper does not transcribe them, so the
   read scored 1.0 and the bursts shipped with it.
2. Quiet consonants and breaths counted as pauses, and pause surgery cut
   their middles out: skips inside phrases, breaths chopped in half.
3. A read ended 120 ms after its last loud frame, which is before the end
   of a final "s", "th" or "ts".

What this ear does instead, with every level measured against the take's
own speech level:

- The sentence ends at the last syllable that starts no later than 0.3 s
  after the ASR's last word (or runs on from one that does). What follows
  that is not the sentence and is dropped.
- A read starts and ends at the nearest real silence (0.12 s of it: less is
  a stop consonant's closure), so a soft first or last consonant stays
  whole; with no silence within reach (a breath or the voice's hiss runs
  into the word), 120 ms out or where the sound is faint, with a longer
  fade. The report says how much it kept (`lead_s`, `tail_s`).
- A pause is still measured the way narrate.py measured it, so the rates
  and the score mean what they meant. It is shortened only when it is
  plainly too long, never to within 90 ms of the words beside it, and a
  pause with a breath in it gets a longer leash and a 40 ms crossfade.

Nothing is stretched: sample positions are cut or kept, as before.
"""
import json
import math
import os
import sys
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True  # importing narrate must leave no __pycache__ in the skill
sys.path.insert(0, sys.argv[1])
import narrate as nr  # noqa: E402

respellings = nr.tts_variants  # narrate.py's own, before main() puts tts_variants in its place

HOP = 0.005                # s per level frame (15 ms window)
NUCLEUS = 18.0             # dB under the take's speech level: a syllable is at least this loud
PAUSE = 20.0               # dB under: a stretch below this is a pause (narrate.py's -48 dBFS, for these voices)
AUDIBLE = 46.0             # dB under (or 6 dB over the take's noise floor): below this is silence
FAINT = 28.0               # dB under: a breath or hiss this far down can take a fade without a seam
SENTENCE_GAP = 0.30        # s of no syllable that ends the sentence once the ASR's last word is behind
HOLD = 0.12                # s of silence that is a pause; shorter is a stop consonant's closure
ONSET_REACH = 0.45         # s before the first syllable searched for that silence
TAIL_REACH = 0.50          # s after the last syllable searched for it
MIN_PAUSE = 0.12
# kind of pause: ((longest a silent one may be, what it is cut to), (the same for one with a breath
# or a soft consonant in it: cut later, and never closer than 120 ms to the words on either side))
CAPS = {"punct": ((0.42, 0.40), (0.52, 0.44)), "none": ((0.22, 0.18), (0.34, 0.24)),
        "unknown": ((0.34, 0.30), (0.52, 0.40))}
STALL = 0.90               # a pause this long is the model stalling, whatever is in it
CUT_SHORT = 16.0           # dB under speech level: a last sound that stops louder than this was cut off
SIBILANT_TAIL = 0.30       # s of kept speech searched for the final s/z of a word that ends in one
FINISHERS = (("“", "”"),)  # every read is also auditioned inside these; ftts then finishes the sentence
EXTRA = {}                 # script sentence -> the director's extra ftts inputs for it (reads.tsv)
PICKS = {}                 # read tag -> why the director chose it over the score (picks.tsv)
# The limiter is for plosives. More work than this and it is flattening the stressed syllables: raise
# the headroom (the narration skill's criterion; its limiter_sweep.py shows each ceiling's cost).
LIMITER_MAX = (1.5, 0.15)  # % of samples it may turn down by more than 1 dB, and by more than 3 dB


def frames(seconds):
    return int(round(seconds / HOP))


def levels(x, rate):
    """dB RMS per HOP, 15 ms window centred on the frame."""
    n, h = int(0.015 * rate), int(HOP * rate)
    c = np.cumsum(np.concatenate([[0.0], np.pad(x, (n // 2, n - n // 2)) ** 2]))
    i = np.arange(0, len(x), h)
    return 10 * np.log10((c[i + n] - c[i]) / n + 1e-12)


def runs(mask, bridge):
    """[first, last] frame of every run of True, runs with at most `bridge` False frames between joined."""
    out = []
    for i in np.flatnonzero(mask):
        if out and i - out[-1][1] <= bridge + 1:
            out[-1][1] = i
        else:
            out.append([i, i])
    return out


def edge(db, audible, faint, start, step, reach, skip):
    """Walk outward from the frame beside a syllable to the nearest HOLD of
    silence: (the silent frame next to the sound, True). The take's own edge
    counts when the sound stops before it. With no silence within reach:
    (a frame under `faint`, or the quietest, False)."""
    i, quiet, hold = start, 0, frames(HOLD)
    while 0 <= i < len(db) and abs(i - start) < reach + hold:
        quiet = quiet + 1 if db[i] < audible else 0
        if quiet >= hold:
            return i - step * (hold - 1), True
        i += step
    if not 0 <= i < len(db) and (quiet or step > 0):
        # ran off the take: clean only if it had gone quiet (20 ms) before its edge. A take that
        # ends in sound keeps all of it; one that opens in sound is judged below.
        return (i - step * quiet, quiet >= 4) if quiet else (i - step, False)
    # sound all the way: past `skip`, which keeps a soft consonant. A level that holds steady out
    # there is the voice's hiss, not a word (syllables swing by far more than 3 dB): cut at `skip`.
    # Otherwise the nearest frame that is faint next to speech; failing that, the quietest.
    span = [k for k in range(start + step * skip, start + step * reach, step) if 0 <= k < len(db)] or [start]
    if len(span) > 8 and float(np.std(db[span])) < 3.0:
        return span[0], False
    return next((k for k in span if db[k] < faint), min(span, key=lambda k: db[k])), False


def voiced_run(seg, rate, top):
    """The longest run of voiced 10 ms frames in a stretch, judged by periodicity alone: the
    stretch is raised to speech level first, because f0_track calls anything under -40 dBFS
    unvoiced and a weak first word is under that."""
    best = run = 0
    for voiced in ~np.isnan(nr.f0_track(seg * 10 ** ((-3.0 - top) / 20), rate)):
        run = run + 1 if voiced else 0
        best = max(best, run)
    return best


def noise(seg, rate):
    """A stretch is breath or hiss, not a word: its energy is spread over the band. A voice, even
    a creaky one with no pitch to track, puts its energy under 1 kHz (20 dB and more over what it
    has above 2 kHz); breath and hiss stay within a few dB of level."""
    if len(seg) < int(0.03 * rate):
        return False
    power = np.abs(np.fft.rfft(seg * np.hanning(len(seg)))) ** 2
    f = np.fft.rfftfreq(len(seg), 1 / rate)
    low, high = power[(f >= 80) & (f < 1000)].sum(), power[(f >= 2000) & (f < 8000)].sum()
    return bool(10 * np.log10((low + 1e-12) / (high + 1e-12)) < 10.0)


def steady(level):
    """A stretch whose level holds within 3 dB for 0.15 s or more: a hum (leo leads into sentences
    with up to 0.4 s of one, all under 500 Hz, so `noise` passes it). A word attacks and decays."""
    return bool(len(level) >= frames(0.15) and float(np.std(level)) < 3.0)


def wants_sibilant(sentence):
    """The sentence's last word ends in an s or z sound in its spelling: phones, codes, writes, other's,
    once, those, box (every ending the owner and the audit heard dropped was one of these)."""
    words = sentence.split()
    last = "".join(c for c in (words[-1] if words else "").lower() if c.isalpha() or c == "'").rstrip("'")
    return last.endswith(("s", "x", "ce", "se", "ze"))


def sibilant_missing(x, rate, end):
    """No frication at the end of a read whose last word ends in s/z: over the last SIBILANT_TAIL s
    before `end`, even the 15 ms frame with the most 4-10 kHz energy has more energy under 2 kHz.
    ftts drops a final s/z ("a million phone"); Whisper writes the word in full, so only the
    spectrum shows it. Calibrated on 113 reads: every missing ending had this ratio below 0 dB
    (-7 to -31), every present one above +13."""
    seg = x[max(0, int((end - SIBILANT_TAIL) * rate)):int(end * rate)]
    n, h = int(0.015 * rate), int(HOP * rate)
    if len(seg) < n:
        return False
    win, f = np.hanning(n), np.fft.rfftfreq(n, 1 / rate)
    hi, lo = (f >= 4000) & (f < 10000), (f >= 100) & (f < 2000)
    best_hf, ratio = -np.inf, -np.inf
    for i in range(0, len(seg) - n + 1, h):
        power = np.abs(np.fft.rfft(seg[i:i + n] * win)) ** 2
        hf, lf = 10 * np.log10(power[hi].sum() + 1e-12), 10 * np.log10(power[lo].sum() + 1e-12)
        if hf > best_hf:
            best_hf, ratio = hf, hf - lf
    return bool(ratio < 0.0)


def first_voiced(x, rate, top):
    """Seconds to the take's first 40 ms of voiced sound (None if it has none), judged like voiced_run."""
    run = 0
    for i, voiced in enumerate(~np.isnan(nr.f0_track(x * 10 ** ((-3.0 - top) / 20), rate))):
        run = run + 1 if voiced else 0
        if run == 4:
            return (i - 3) * 0.01
    return None


def hear(x, rate, last_word_end):
    """Where the sentence is inside the take: its cut-in and cut-out, whether
    each lands in silence, and the pauses between its syllables."""
    db = levels(x, rate)
    top = float(np.percentile(db[db > db.max() - 45], 95))
    floor = max(-96.0, float(np.percentile(db, 5)))
    if floor > top - 34.0:  # a take with no silence in it has no floor to measure
        floor = top - AUDIBLE - 6.0
    audible = max(top - AUDIBLE, floor + 6.0)
    phrases = runs(db >= top - NUCLEUS, frames(SENTENCE_GAP))
    if not phrases:
        return None
    # the sentence: every phrase that starts by the ASR's last word; what starts later is not it
    limit = frames(last_word_end + SENTENCE_GAP) if last_word_end is not None else len(db)
    kept = [p for p in phrases if p[0] <= limit] or phrases[:1]
    first, last = kept[0][0], kept[-1][1]
    junk_after = sum((b - a + 1) * HOP for a, b in phrases[len(kept):])
    # A syllable has a voice in it. Sound this loud that starts more than 0.2 s (a loud "s" or
    # "sh" takes less) before the take's first voiced stretch is a noise bed, not the first word
    # (robert opens takes with up to 0.8 s of it, 12 dB under the speech).
    # The stretch must also sound like noise (see `noise`): a creaky "And" has no pitch the tracker
    # can follow, and this rule once took one for a bed and dropped the word.
    voiced = first_voiced(x, rate, top)
    if voiced is not None and first < frames(voiced - 0.20) and (
            noise(x[int(first * HOP * rate):int((voiced - 0.20) * rate)], rate)
            or steady(db[first:frames(voiced - 0.20)])):
        first = min(last, next((i for i in range(frames(voiced - 0.20), len(db)) if db[i] >= top - NUCLEUS), first))
    # Nor is a hum: a steady, modest tone that opens the take and gives way before the first
    # syllable (robert: about 0.4 s near 300 Hz, 18 to 21 dB under the speech). It is periodic, so
    # the voice test passes it; but a syllable attacks and swings, and this holds within 2 dB for
    # a quarter of a second and never comes within 12 dB of speech level.
    held = db[first:first + frames(0.25)]
    if first + len(held) < last and float(np.std(held)) < 2.0 and held.max() < top - 12.0:
        first = next((i for i in range(first + len(held), last + 1) if db[i] >= top - 12.0), first)
    cut_in, clean_in = edge(db, audible, top - FAINT, first - 1, -1, frames(ONSET_REACH), frames(0.12))
    cut_out, clean_out = edge(db, audible, top - FAINT, last + 1, 1, frames(TAIL_REACH), frames(0.12))
    cut_in = min(first, max(0, cut_in - (4 if clean_in else 0)))
    cut_out = max(last, min(len(db) - 1, cut_out + (6 if clean_out else 0)))
    # A long lead-in with no voice in it is a breath or hiss before the sentence, not its first
    # word: start 120 ms out, which is what a soft consonant can take.
    if first - cut_in > frames(0.18):
        a, b = int(cut_in * HOP * rate), int((first - frames(0.12)) * HOP * rate)
        if steady(db[cut_in:first - frames(0.12)]) or (voiced_run(x[a:b], rate, top) < 4 and noise(x[a:b], rate)):
            cut_in, clean_in = first - frames(0.12), False
    # ftts stopped inside the last sound: over its last 80 ms the sound still runs near speech
    # level (a sound that decays ends 19 dB or more under it, and a final "t" or "k" is mostly its
    # closure), and the WAV ends there, give or take the 60 ms of digital silence ftts may add.
    stop = next((i for i in range(min(cut_out, len(db) - 1), last - 1, -1) if db[i] >= audible), last)
    ending = 10 * np.log10(np.mean(10 ** (db[max(0, stop - 15):stop + 1] / 10)))
    cut_short = bool((len(db) - 1 - stop) * HOP < 0.15 and ending > top - CUT_SHORT)
    pauses = []
    for a, b in runs(db[first:last + 1] < top - PAUSE, 0):
        a, b = a + first, b + first
        if (b - a + 1) * HOP >= MIN_PAUSE:
            inner = db[a + 8:b - 7]
            pauses.append({"a": a * HOP, "b": (b + 1) * HOP, "silent": bool(len(inner) == 0 or inner.max() < audible)})
    return {"cut_in": cut_in * HOP, "cut_out": (cut_out + 1) * HOP, "clean_in": clean_in, "clean_out": clean_out,
            "first": first * HOP, "last": (last + 1) * HOP, "pauses": pauses, "junk_after_s": round(junk_after, 2),
            "cut_short": cut_short, "top_db": round(top, 1), "floor_db": round(floor, 1)}


class Picked(float):
    """The measured score of a read the director picked. narrate.py ranks candidates by `-score`;
    this one sorts first, and stays the measured number in the report and in the weak-read flags."""

    def __neg__(self):
        return float("-inf")


def tts_variants(text):
    """narrate.py's respellings of one sentence, each also inside quotation marks (FINISHERS)."""
    plain = respellings(text)
    reads = plain + [f"{a}{v}{b}" for a, b in FINISHERS for v in plain]
    return reads + [t for t in EXTRA.get(text, []) if t not in reads]


def director_files(script, work, command):
    """reads.tsv and picks.tsv beside the script (both optional, # comments allowed).

    reads.tsv  `<line_id> TAB <sentence index> TAB <ftts input>`: one more read of that sentence.
               The input goes to ftts only, like a respelling; the sentence's text stays the
               script's. For what no alternate can do in correct English: other quotation marks
               around a short sentence that ftts keeps cutting short, a number spelled another way.
    picks.tsv  `<line_id> TAB <sentence index> TAB <ftts input> TAB <why>`: that read wins whatever
               its score, because the director saw in the sheets what the score cannot (the report
               says `picked`). The why is required. NARRATION_PICKS names another picks file: a
               variant read by another voice has its own (the script's picks name reads in the
               video's own voice)."""
    here = Path(script).parent
    lines = {lid: nr.sentences(text) for lid, text in nr.checked_script(script)}

    def rows(name, columns):
        path = Path(os.environ["NARRATION_PICKS"]) if name == "picks.tsv" and os.environ.get("NARRATION_PICKS") \
            else here / name
        for n, row in enumerate(path.read_text().splitlines() if path.exists() else [], 1):
            if row.strip() and not row.startswith("#"):
                cells = row.split("\t")
                if len(cells) != columns or not cells[1].isdigit() or cells[0].strip() not in lines \
                        or int(cells[1]) >= len(lines[cells[0].strip()]) or not all(c.strip() for c in cells):
                    sys.exit(f"{path}:{n}: want {columns} tab-separated cells, a line id of the script and a "
                             f"sentence index within it: {row!r}")
                yield path, n, cells[0].strip(), int(cells[1]), *cells[2:]

    for *_, lid, j, tts in rows("reads.tsv", 3):
        EXTRA.setdefault(lines[lid][j], []).append(tts)
    if command == "choose":
        plan = json.loads((Path(work) / "plan.json").read_text())
        for path, n, lid, j, tts, why in rows("picks.tsv", 4):
            tag = next((r["tag"] for r in plan[lid][j]["reads"] if r["tts"] == tts), None)
            if tag is None:
                sys.exit(f"{path}:{n}: no planned read of {lid} sentence {j} has the ftts input {tts!r} "
                         f"(see {Path(work) / 'takes.tsv'}; a new input goes in reads.tsv first)")
            PICKS[tag] = why


def analyze(path, asr, target_text):
    """narrate.py's analyze on the sentence `hear` found (same score, same fields)."""
    x, rate = nr.load(path)
    want_tok = nr.tokens(target_text)
    want = [w for w, _ in want_tok]
    words = asr.get("words", [])
    h = hear(x, rate, words[-1]["e"] if words else None)
    if h is None:
        return None
    heard_tok = [(t, w["s"], w["e"], w["p"]) for w in words for t, _ in nr.tokens(w["w"])]
    ratio, _, hit = nr.align(want, [t for t, _ in nr.tokens(asr.get("text", ""))])
    names_missed = nr.misheard_names(want, hit)
    _, to_want, _ = nr.align(want, [t for t, *_ in heard_tok])
    pauses, cuts = [], []
    for p in h["pauses"]:
        a, length = p["a"], p["b"] - p["a"]
        before = [j for j, (_, s, e, _) in enumerate(heard_tok) if e <= a + 0.25]
        punct = want_tok[to_want[before[-1]]][1] if before and before[-1] in to_want else nr.UNMATCHED
        kind = "punct" if punct in (",", ".") else ("unknown" if punct == nr.UNMATCHED else "none")
        limit, cap = CAPS[kind][0 if p["silent"] else 1]
        if length >= STALL:
            limit, cap = STALL, CAPS[kind][0][1]
        # A pause with sound in it may hold a weak word ("go up, and ... the order"): what the cut
        # would remove must have no voice in it, or the pause stays as long as it is.
        if length > limit and (p["silent"] or voiced_run(
                x[int((a + cap / 2) * rate):int((p["b"] - cap / 2) * rate)], rate, h["top_db"]) < 4):
            cuts.append([round(a + cap / 2, 4), round(p["b"] - cap / 2, 4), 0.01 if p["silent"] else 0.04])
        pauses.append({"at": round(a, 2), "len": round(length, 2), "punct": punct, "silent": p["silent"]})
    hesitations = [p for p in pauses if p["punct"] == "" and p["len"] >= 0.22]
    articulation = (h["last"] - h["first"]) - sum(p["len"] for p in pauses)
    syl_rate = sum(nr.syllables(w) for w in want) / articulation if articulation > 0 else 0.0
    probs = [p for *_, p in heard_tok] or [0.0]
    st = nr.semitones(nr.f0_track(x[int(h["first"] * rate):int(h["last"] * rate)], rate))
    pitch_std = float(np.std(st)) if len(st) > 8 else 0.0
    final_st = float(np.median(st[-6:])) if len(st) > 8 else 0.0
    question = target_text.rstrip().endswith("?")
    if question and target_text.split()[0].lower() in nr.YES_NO:
        s_contour = float(np.clip((final_st + 0.5) / 2.5, 0, 1))
    elif question:
        s_contour = 1.0
    else:
        s_contour = float(np.clip((2.5 - final_st) / 2.5, 0, 1))
    s_conf = float(np.clip((np.mean(probs) - 0.5) / 0.45, 0, 1))
    s_rate = math.exp(-((min(syl_rate, nr.SYL_MAX + 0.6) - nr.SYL_TARGET) / nr.SYL_SIGMA) ** 2 / 2)
    s_pause = 1.0 / (1.0 + sum(p["len"] for p in hesitations) * 3)
    s_pitch = pitch_std / 1.2 if pitch_std < 1.2 else (max(0.0, 1 - (pitch_std - 6.5) / 4) if pitch_std > 6.5 else 1.0)
    score = 0.40 * ratio + 0.15 * s_conf + 0.15 * s_rate + 0.10 * s_pause + 0.10 * s_pitch + 0.10 * s_contour
    score -= nr.NAME_PENALTY * names_missed + nr.HESITATION_PENALTY * len(hesitations)
    if ratio < 0.85:
        score -= 0.5
    # what the old ear could not hear: a read that ran on, stalled, has no clean way in or out,
    # or (the worst of them: the last word is not all there) was cut short by ftts
    stalls = sum(1 for p in pauses if p["len"] >= STALL)
    score -= 0.02 * (h["junk_after_s"] > 0) + 0.04 * stalls + 0.02 * (not h["clean_in"]) + 0.02 * (not h["clean_out"])
    score -= 0.12 * h["cut_short"]
    no_s = wants_sibilant(target_text) and sibilant_missing(x, rate, h["cut_out"])
    score -= 0.12 * no_s  # a dropped final s/z changes the word ("phone" for "phones")
    picked = PICKS.get(Path(path).stem)
    score = round(score, 4)
    if picked is not None:
        score = Picked(score)
    return {
        "score": score, "asr_ratio": round(ratio, 3), "misheard_names": names_missed,
        "conf": round(float(np.mean(probs)), 3), "syl_rate": round(syl_rate, 2), "pauses": pauses,
        "hesitations": len(hesitations), "pitch_std_st": round(pitch_std, 2), "final_st": round(final_st, 2),
        "start": round(h["cut_in"], 4), "end": round(h["cut_out"], 4), "heard": asr.get("text", ""),
        "cuts": cuts, "clean_in": h["clean_in"], "clean_out": h["clean_out"], "junk_after_s": h["junk_after_s"],
        "cut_short": h["cut_short"], "sibilant_missing": no_s, "stalls": stalls,
        "lead_s": round(h["first"] - h["cut_in"], 3), "tail_s": round(h["cut_out"] - h["last"], 3),
        **({"picked": picked} if picked else {}), "take_s": round(len(x) / rate, 2), "top_db": h["top_db"], "floor_db": h["floor_db"],
    }


def edit(path, m, sentence_lufs):
    """Cut the winning read where `hear` said. Sample positions are only ever cut
    or joined (a crossfade inside the pause), never resampled or stretched."""
    x, rate = nr.load(path)
    if rate != nr.RATE:
        sys.exit(f"{path}: {rate} Hz (expected {nr.RATE}); refusing to resample a read")
    seg, pos, fade = x[:0], m["start"], 0.0
    for a, b, next_fade in m["cuts"] + [[m["end"], m["end"], 0.0]]:
        part, n = x[int(pos * rate):int(a * rate)], int(fade * rate)
        if n and len(seg) > n and len(part) > n:
            mix = seg[-n:] * np.linspace(1, 0, n) + part[:n] * np.linspace(0, 1, n)
            seg = np.concatenate([seg[:-n], mix, part[n:]])
        else:
            seg = np.concatenate([seg, part])
        pos, fade = b, next_fade
    span = int(m["end"] * rate) - int(m["start"] * rate)
    lufs = nr.integrated_lufs(seg)
    if lufs is None:  # too short to gate: RMS of loud frames stands in
        db = nr.frame_db(seg, rate)
        lufs = float(np.mean(db[db > db.max() - 25])) - 3.0
    seg = seg * 10 ** ((sentence_lufs - lufs) / 20)
    # a cut in silence needs only a click guard; a cut in sound gets a fade one can't hear as a cut
    n_in = min(len(seg) // 4, int((0.012 if m["clean_in"] else 0.04) * rate))
    n_out = min(len(seg) // 4, int((0.02 if m["clean_out"] else 0.08) * rate))
    seg[:n_in] *= np.linspace(0, 1, n_in)
    seg[len(seg) - n_out:] *= np.linspace(1, 0, n_out)
    proof = {"src_span_s": round(span / rate, 4), "kept_s": round(len(seg) / rate, 4), "out_s": round(len(seg) / rate, 4)}
    return seg, round((span - len(seg)) / rate, 3), round(lufs, 1), proof


def edges(script, out, lexicon):
    """Bind every ordered sentence to the canonical script, then check its first and last word
    against the assembled line's transcript. Nothing to check is not a pass."""
    nr.load_lexicon(nr.resolve_optional(lexicon, "lexicon.json", "lexicon"))
    report = json.loads((out / "report.json").read_text())["lines"]
    heard = json.loads((out / "lines_asr.json").read_text())
    scripted = dict(nr.checked_script(script))
    missing = [f"{lid}: not in the script" for lid in report if lid not in scripted]
    for lid, text in scripted.items():
        expected = nr.sentences(text)
        sents = report.get(lid, [])
        if len(sents) != len(expected):
            missing.append(f"{lid}: report has {len(sents)} sentences, script has {len(expected)}")
        for j, (sentence, recorded) in enumerate(zip(expected, sents)):
            if recorded.get("j") != j:
                missing.append(f"{lid}.{j}: report sentence index is {recorded.get('j')!r}")
            want = [t for t, _ in nr.tokens(sentence)]
            have = [t for t, _ in nr.tokens(recorded.get("sentence", ""))]
            if have != want:
                missing.append(f"{lid}.{j}: report sentence differs from the script | {sentence}")
        if not heard.get(lid, {}).get("text", "").strip():
            missing.append(f"{lid}: no transcript in lines_asr.json")
    for lid, sents in report.items():
        want, spans = [], []
        for s in sents:
            words = [t for t, _ in nr.tokens(s["sentence"])]
            if words:
                spans.append((s["j"], len(want), len(want) + len(words) - 1, s["sentence"]))
                want += words
        text = heard.get(lid, {}).get("text", "").replace("%", " percent")  # the ASR writes the sign
        _, _, hit = nr.align(want, [t for t, _ in nr.tokens(text)])
        missing += [f"{lid}.{j}: {which} word {want[k]!r} is not in the line's transcript | {sentence}"
                    for j, a, b, sentence in spans for k, which in ((a, "first"), (b, "last")) if k not in hit]
    for m in missing:
        print("  ", m)
    print(f"edges: {len(missing)} coverage or first/last-word failures -> "
          f"{'FAIL (look at the sheet: cut off, misread, or the ASR spelling, which goes in the lexicon)' if missing else 'PASS'}")
    sys.exit(1 if missing else 0)


def main():
    worked = [0, 0, 0]  # samples turned down by more than 1 dB, by more than 3 dB, and all of them
    limit = nr.limit_peaks

    def counted_limit(x, ceiling, *args, **kwargs):
        y, frac, max_db = limit(x, ceiling, *args, **kwargs)
        on = np.abs(x) >= 1e-6
        kept = np.abs(y[on]) / np.abs(x[on])
        worked[0] += int(np.sum(kept < 10 ** (-1 / 20)))
        worked[1] += int(np.sum(kept < 10 ** (-3 / 20)))
        worked[2] += len(x)
        return y, frac, max_db

    if len(sys.argv) >= 5 and sys.argv[2] == "edges":
        edges(sys.argv[3], Path(sys.argv[4]),
              sys.argv[sys.argv.index("--lexicon") + 1] if "--lexicon" in sys.argv else None)
    if len(sys.argv) < 3 or sys.argv[2] not in ("plan", "choose"):
        sys.exit(__doc__)
    command, out = sys.argv[2], Path(sys.argv[5]) if len(sys.argv) > 5 else None
    nr.tts_variants, nr.analyze, nr.edit, nr.limit_peaks = tts_variants, analyze, edit, counted_limit
    director_files(sys.argv[3], sys.argv[4], command)
    sys.argv = [str(Path(sys.argv[1]) / "narrate.py"), *sys.argv[2:]]
    nr.main()
    if command == "plan":
        return
    reads = [s for line in json.loads((out / "report.json").read_text())["lines"].values() for s in line]
    print(f"endings: {sum(s['cut_short'] for s in reads)} of {len(reads)} chosen reads were cut short by ftts "
          f"(the last sound stops at speech level); {sum(s['tts'].startswith(FINISHERS[0][0]) for s in reads)} "
          f"are the read in quotation marks; {sum('picked' in s for s in reads)} were picked by the director")
    missing = [f"{s['read'].split('__')[0]}.{s['j']} {s['sentence'].split()[-1]}" for s in reads if s.get("sibilant_missing")]
    print(f"final s/z: {len(missing)} chosen reads end without the s or z their last word needs"
          + (f" (each needs another read or a pick: {', '.join(missing)})" if missing else ""))
    over1, over3 = (100 * w / max(1, worked[2]) for w in worked[:2])
    ok = over1 <= LIMITER_MAX[0] and over3 <= LIMITER_MAX[1]
    print(f"limiter: turned down >1 dB for {over1:.2f}% of samples, >3 dB for {over3:.3f}% "
          f"(may be {LIMITER_MAX[0]}% and {LIMITER_MAX[1]}%): {'ok' if ok else 'TOO MUCH, raise --limit-headroom'}")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
