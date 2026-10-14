#!/usr/bin/env python3
"""The narration skill's `plan` and `choose`, with reads that finish their last
word and an editor that listens to what it cuts.

    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS plan \
        SCRIPT.tsv WORK [--alts alts.tsv] [--lexicon lexicon.json] [--voice V] [--model M]
    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS choose \
        SCRIPT.tsv WORK OUT [--lexicon lexicon.json] [--sentence-lufs -23] [--limit-headroom 15]

    uv run --no-project --with numpy python3 -I tools/narration.py SKILL_SCRIPTS edges \
        SCRIPT.tsv OUT [--lexicon lexicon.json]

    plan ... --seeds N     also plan every read with `ftts --seed 1..N` (work/seeds.tsv, not takes.tsv)
    synth WORK --voice V [--model M] [--workers W]    the seeded reads plan listed (after tts_batch.py)
    choose ... --joint     pick each line's reads together (see "Why the reads of a line are chosen together")
    exclude SCRIPT.tsv WORK OUT [--lexicon L]    after the line-level gates failed a --joint build: price the
                           reads it moved in the failing lines as failed (exit 1: nothing new, stop)

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

Why the reads of a line are chosen together (`choose --joint`, `plan --seeds`; bead mx-4kc2).
narrate.py picks every sentence's read on its own, and nothing it scores is defined across a join,
so the register and the pitch reset at a join are chosen by accident. Measured on Video F against
human readers (evidence RS-20261014-narration, T-003, T-007, audit A): 31% of F's sentence joins
reset the pitch by more than 12 st where a human reader's 12% do, because F's sentences end low and
the next one starts high; a choice among the reads F already had cut the joins past +-7 st from 29%
to 12% (T-003); and `ftts --seed 1..7` gives other reads of the same text (up to 57% duration
spread, final contours 10 st apart), deterministic per seed, so a seed is a retake.
- `plan --seeds N` adds every read again with seed 1..N. They go to work/seeds.tsv, never takes.tsv
  (tts_batch.py would read them without the seed), and `synth` makes them. Seed 0 is no seed.
- `choose --joint` runs a Viterbi over each line: a read's cost is its own score plus its defects
  (a stall or hole inside, a statement that rises at its end, a creaky ending, a pitch jump in its last 120 ms), a
  join's cost is the reset beyond +12 st and the register step between the two reads, measured with
  Praat (the autocorrelation trackers read a creaky ending 20-90 Hz high and miss most resets); a
  second tracker, held out of the costs, is counted beside it and must agree on a terminal pitch
  event. It needs `uv run --with numpy --with praat-parselmouth`. A director's pick is not a choice, it stays. The reads
  it moves away from narrate.py's pick are written into the report (`joint`); when nothing at a join
  is bad it picks narrate.py's read, and a run without --joint is byte for byte what it was.
- Taste: whether the joint reads sound better is the owner's to say; a video opts in per run.
"""
import hashlib
import json
import math
import os
import subprocess
import sys
import time
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
BED_UNDER = 9.0            # dB: sound before the first voiced stretch that comes within this of speech
BED_MIN = 0.30             # level, runs into the voice without a pause, and is shorter than this (s) or
                           # has some voice in it, is the first word's onset, not a bed
TAIL_REACH = 0.50          # s after the last syllable searched for it
MIN_PAUSE = 0.12
# kind of pause: ((longest a silent one may be, what it is cut to), (the same for one with a breath
# or a soft consonant in it: cut later, and never closer than 120 ms to the words on either side))
CAPS = {"punct": ((0.42, 0.40), (0.52, 0.44)), "none": ((0.22, 0.18), (0.34, 0.24)),
        "unknown": ((0.34, 0.30), (0.52, 0.40))}
STALL = 0.90               # a pause this long is the model stalling, whatever is in it
CUT_SHORT = 16.0           # dB under speech level: a last sound that stops louder than this was cut off
SIBILANT_GATE = 25.0       # dB under speech level: quieter frames after the last word are breath, not its ending
SIBILANT_MS = 45           # ms of frication a final s/z needs in the last 100 ms of speech
FINISHERS = (("“", "”"),)  # every read is also auditioned inside these; ftts then finishes the sentence
EXTRA = {}                 # script sentence -> the director's extra ftts inputs for it (reads.tsv)
PICKS = {}                 # read tag -> why the director chose it over the score (picks.tsv)
JOINT = {}                 # read tag -> what the joint choice recorded where it moved off narrate.py's pick
HOLE = 0.45                # s: an unlicensed pause this long inside a read is a hole, not a breath (T-005)
TERMINAL = (9.0, 4.5)      # st: the last 120 ms rise this far over the 250 ms before it, by Praat and by the second tracker
CREAK_HZ = 80.0            # Hz: an ending whose median, or half of whose last 150 ms, is under this is creaky (P-2)
JOINT_COST = {             # the Viterbi's weights (the research pilot P-2's, on chapter 02 of Video F), then ours
    "score": 2.0,          # per 1.0 of narrate.py's score under a perfect read
    "creak": 1.0,          # a creaky ending: F's sentences end 62-75 Hz, the cause of the heavy tail of resets (T-007)
    "gate": 5.0,           # a cut-short read, a dropped final s/z, an ASR ratio under 0.85: nothing else if avoidable
    "name": 1.0, "hesitation": 0.5, "stall": 2.0,
    "hole": 1.0, "onset": 1.0,           # ours: a hole inside the read; a heard word before the first scripted one
    "rise": 0.25,          # per st (up to 8) a statement rises over 2 st at its end
    "terminal": 2.0,       # ours: a terminal pitch event (B-C03), until the owner's ear says what it is
    "change": 0.02,        # tie-break toward narrate.py's pick
    "tail": 1.0, "tail_per_st": 0.2, "tail_at": 12.0,   # a reset past +12 st: 1.0 and 0.2 per st beyond
    "register": 0.1, "register_free": 4.0,              # ours: per st of register step past 4 (B-C08)
}
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


def voice_in(seg, rate, top):
    """Some of a voice in a stretch the pitch tracker calls unvoiced: three 10 ms frames, anywhere in it
    and audible (within 30 dB of speech level), whose energy sits under 1 kHz the way `noise` defines a
    voice. A creaky vowel's frames are such frames, though its uneven pulses leave gaps between them;
    an s, a t's aspiration, a breath or a hiss has none (white hiss sits 8 dB the other way)."""
    n = int(0.010 * rate)
    f = np.fft.rfftfreq(n, 1 / rate)
    low, high = (f >= 80) & (f < 1000), (f >= 2000) & (f < 8000)
    run = 0
    for i in range(0, len(seg) - n + 1, n):
        frame = seg[i:i + n]
        power = np.abs(np.fft.rfft(frame * np.hanning(n))) ** 2
        voiced = (10 * np.log10(np.mean(frame ** 2) + 1e-12) > top - 30
                  and 10 * np.log10((power[low].sum() + 1e-12) / (power[high].sum() + 1e-12)) >= 20.0)
        run += bool(voiced)
        if run >= 3:
            return True
    return False


def steady(level):
    """A stretch whose level holds within 3 dB for 0.15 s or more: a hum (leo leads into sentences
    with up to 0.4 s of one, all under 500 Hz, so `noise` passes it). A word attacks and decays."""
    return bool(len(level) >= frames(0.15) and float(np.std(level)) < 3.0)


def wants_sibilant(sentence):
    """The sentence's last word ends in a hissing sound in its spelling: s or z (phones, codes, writes,
    other's, once, those, box) or ch/sh (catch, which the audit heard as "cat"; wash)."""
    words = sentence.split()
    last = "".join(c for c in (words[-1] if words else "").lower() if c.isalpha() or c == "'").rstrip("'")
    return last.endswith(("s", "x", "ce", "se", "ze", "ch", "sh"))


def sibilant_missing(x, rate, end, top):
    """The last word's s/z is gone or clipped: the last 100 ms of speech before `end` (frames within
    SIBILANT_GATE dB of the read's speech level `top`, so a breath after the word does not count)
    holds under SIBILANT_MS ms of frication, 15 ms frames with more energy at 3-10 kHz than at
    300-2000 Hz (bands that also take sh and a voiced z, whose voice bar is under 300 Hz). ftts
    drops a final s/z ("a million phone") or ends the take 30-50 ms into it ("cell-"); Whisper
    writes the word in full, so only the spectrum shows it. Calibrated on four michael videos: the
    226 chosen reads, whose endings the directors checked, had 45-100 ms (sh, a voiced z and a short
    s the least: "fish?", "sees", "else" are full on a spectrogram); the replaced reads whose
    spectrogram shows the ending missing or clipped had 0-40. The check this replaces (the strongest 4-10 kHz frame in the last 0.3 s)
    passed "slots", "pieces" and "doubles" on the s at the start of the word, and failed a full
    "months" on the breath after it."""
    n, h = int(0.015 * rate), int(HOP * rate)
    seg = x[max(0, int((end - 0.6) * rate)):int(end * rate)]
    win, f = np.hanning(n), np.fft.rfftfreq(n, 1 / rate)
    hi, lo = (f >= 3000) & (f < 10000), (f >= 300) & (f < 2000)
    frication = []  # per frame: None for no speech, else whether it is frication
    for i in range(0, len(seg) - n + 1, h):
        frame = seg[i:i + n]
        if 10 * np.log10(np.mean(frame ** 2) + 1e-12) <= top - SIBILANT_GATE:
            frication.append(None)
            continue
        power = np.abs(np.fft.rfft(frame * win)) ** 2
        frication.append(bool(power[hi].sum() > power[lo].sum()))
    last = max((k for k, s in enumerate(frication) if s is not None), default=None)
    if last is None:
        return False  # no speech in reach: nothing to judge
    ms = 1000 * HOP * sum(1 for s in frication[max(0, last - frames(0.1) + 1):last + 1] if s)
    return bool(ms < SIBILANT_MS)


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
    # can follow, and this rule once took one for a bed and dropped the word. Nor is the first word's
    # own onset: at speech level, running into the voice without a pause, and short or with some
    # voice in it. michael opens words with an s, an aspirated t or a vowel the tracker does not hear
    # as voiced for up to 0.45 s ("Take its rate" was cut to "ake its rate", heard "It gets rate";
    # "Subtracting" to "tracting", heard "attracting"). A bed is quieter (robert's: about 12 dB
    # under), set apart from the word by a pause (a blip at the take's start), or a hiss with no voice.
    voiced = first_voiced(x, rate, top)
    bed_end = frames(voiced - 0.20) if voiced is not None else 0
    stretch = x[int(first * HOP * rate):int((voiced - 0.20) * rate)] if voiced is not None else x[:0]
    onset = (bed_end > first and db[first:bed_end].max() >= top - BED_UNDER
             and not any(b - a + 1 >= frames(HOLD) for a, b in runs(db[first:bed_end] < top - FAINT, 0))
             and ((bed_end - first) * HOP < BED_MIN or voice_in(stretch, rate, top)))
    if voiced is not None and first < bed_end and not onset and (noise(stretch, rate) or steady(db[first:bed_end])):
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


MEASURED = {}  # (take, sentence) -> measure(); the joint choice reads every candidate before `choose` does


def analyze(path, asr, target_text):
    """measure(), marked with who chose the read: the director (picks.tsv) or the joint choice. A chosen
    read ranks first under narrate.py's key and keeps its measured score."""
    key = (str(path), target_text)
    if key not in MEASURED:
        MEASURED[key] = measure(path, asr, target_text)
    if MEASURED[key] is None:
        return None
    m, tag = dict(MEASURED[key]), Path(path).stem
    picked, joint = PICKS.get(tag), None if tag in PICKS else JOINT.get(tag)
    if picked is not None or joint is not None:
        m["score"] = Picked(m["score"])
    for name, value in (("picked", picked), ("joint", joint)):
        if value:
            m[name] = value
        else:
            del m[name]
    return m


def measure(path, asr, target_text):
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
    no_s = wants_sibilant(target_text) and sibilant_missing(x, rate, h["cut_out"], h["top_db"])
    score -= 0.12 * no_s  # a dropped final s/z changes the word ("phone" for "phones")
    score = round(score, 4)
    return {
        "score": score, "asr_ratio": round(ratio, 3), "misheard_names": names_missed,
        "conf": round(float(np.mean(probs)), 3), "syl_rate": round(syl_rate, 2), "pauses": pauses,
        "hesitations": len(hesitations), "pitch_std_st": round(pitch_std, 2), "final_st": round(final_st, 2),
        "start": round(h["cut_in"], 4), "end": round(h["cut_out"], 4), "heard": asr.get("text", ""),
        "cuts": cuts, "clean_in": h["clean_in"], "clean_out": h["clean_out"], "junk_after_s": h["junk_after_s"],
        "cut_short": h["cut_short"], "sibilant_missing": no_s, "stalls": stalls,
        "lead_s": round(h["first"] - h["cut_in"], 3), "tail_s": round(h["cut_out"] - h["last"], 3),
        "picked": None, "joint": None,  # placed here for the report's key order; analyze() fills or drops them
        "take_s": round(len(x) / rate, 2), "top_db": h["top_db"], "floor_db": h["floor_db"],
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


def seed_tag(tag, seed):
    """The tag of a read made with `ftts --seed`: content-addressed like narrate.py's, so a re-run reuses it."""
    digest = hashlib.sha1(f"{tag}\x00seed={seed}".encode()).hexdigest()[:8]
    return f"{tag.rsplit('__', 1)[0]}__{digest}"


def add_seeds(work, seeds):
    """Plan every read again for seeds 1..`seeds`. They are listed in work/seeds.tsv (`tag TAB seed TAB ftts
    input`) and not in takes.tsv: tts_batch.py would make them without the seed, under a seeded tag."""
    work, rows = Path(work), []
    plan = json.loads((work / "plan.json").read_text())
    for sents in plan.values():
        for s in sents:
            for r in list(s["reads"]):
                for seed in range(1, seeds + 1):
                    tag = seed_tag(r["tag"], seed)
                    s["reads"].append({"tag": tag, "text": r["text"], "tts": r["tts"], "seed": seed})
                    rows.append(f"{tag}\t{seed}\t{r['tts']}")
    if rows:  # a plan without seeds is narrate.py's plan.json as it wrote it
        (work / "plan.json").write_text(json.dumps(plan, indent=1))
    (work / "seeds.tsv").write_text("".join(row + "\n" for row in rows))
    return len(rows)


def synth(work, voice, model, workers):
    """Make the seeded reads of work/seeds.tsv with tts_batch.py's own batch (resumable, serial retries),
    each with its `--seed`. The batch splits a job line at the first tab, so a job's text is
    `<seed> TAB <ftts input>` and the one function that runs ftts is replaced by a copy that knows it."""
    import tts_batch  # the narration skill's, found through SKILL_SCRIPTS on sys.path

    def synth_seeded(tag, job, out_dir, voice, model):
        seed, text = job.split("\t", 1)
        wav = out_dir / f"{tag}.wav"
        if wav.exists() and wav.stat().st_size > 1000:
            return tag, "kept", 0.0
        partial = out_dir / f"{tag}.partial.wav"
        cmd = [tts_batch.FTTS, "say", "--no-resident", "--voice", voice, "--seed", seed]
        if model:
            cmd += ["--model", model]
        cmd += ["--", text, str(partial)]
        started = time.time()
        with open(out_dir / f"{tag}.log", "ab") as sink:
            rc = subprocess.run(cmd, stdin=subprocess.DEVNULL, stdout=sink, stderr=sink, env=tts_batch.CHILD_ENV).returncode
        if rc != 0 or not partial.exists() or partial.stat().st_size <= 1000:
            return tag, f"FAILED rc={rc}", time.time() - started
        partial.replace(wav)
        return tag, "ok", time.time() - started

    jobs = Path(work) / "seeds.tsv"
    if not jobs.exists() or not jobs.read_text().strip():
        print("synth: no seeded reads planned (plan --seeds N)")
        return 0
    tts_batch.synth = synth_seeded
    sys.argv = ["tts_batch.py", str(jobs), str(Path(work) / "takes"), "--voice", voice, "--workers", str(workers)]
    if model:
        sys.argv += ["--model", model]
    tts_batch.main()  # exits 1 when a read failed every retry


# ---- the joint choice: one line's reads picked together ------------------------------------------

def f0_viterbi(x, rate, fmin=65.0, fmax=330.0, vthr=0.40, jump=0.9, vu=0.25, ncand=5):
    """A second pitch tracker, held out of the costs (the research pilot P-2's; T-003's third estimate):
    up to `ncand` autocorrelation peaks above `vthr` per 10 ms frame, then the cheapest path through the
    frames, where an octave of jump costs `jump` and a voiced/unvoiced change `vu`. Hz per frame, nan when
    unvoiced. Where a frame's peaks disagree about the octave, the path keeps the one that fits its
    neighbours, which a per-frame maximum cannot."""
    n, h = int(0.04 * rate), int(0.01 * rate)
    lo, hi = int(rate / fmax), min(int(rate / fmin), n - 2)
    win, states = np.hanning(n), []  # per frame: (Hz of each state, the state's own price); the last state is unvoiced
    for s in range(0, len(x) - n, h):
        frame = x[s:s + n]
        strength, hz = np.empty(0), np.empty(0)
        if 10 * np.log10(np.mean(frame ** 2) + 1e-18) >= -45:
            spec = np.fft.rfft((frame - frame.mean()) * win, 2 * n)
            ac = np.fft.irfft(spec * np.conj(spec))[:n]
            seg = (ac / (ac[0] + 1e-12))[lo:hi]
            peaks = np.flatnonzero((seg[1:-1] > seg[:-2]) & (seg[1:-1] >= seg[2:]) & (seg[1:-1] > vthr)) + 1
            peaks = peaks[np.argsort(-seg[peaks], kind="stable")][:ncand]
            left, strength, right = seg[peaks - 1], seg[peaks], seg[peaks + 1]
            curve = left - 2 * strength + right
            shift = np.where(curve != 0, 0.5 * (left - right) / np.where(curve != 0, curve, 1.0), 0.0)
            hz = rate / (lo + peaks + shift)
        states.append((np.append(hz, np.nan), np.append(-strength, -(vthr + 0.02))))
    if not states:
        return np.array([])
    cost, back = states[0][1], []
    for (prev, _), (f, own) in zip(states, states[1:]):
        voiced = ~np.isnan(prev)[:, None] & ~np.isnan(f)[None, :]
        silent = np.isnan(prev)[:, None] & np.isnan(f)[None, :]
        move = np.where(voiced, jump * np.abs(np.log2(np.where(voiced, f[None, :] / prev[:, None], 1.0))),
                        np.where(silent, 0.0, vu))
        total = cost[:, None] + move
        back.append(np.argmin(total, axis=0))
        cost = total[back[-1], np.arange(len(f))] + own
    k, path = int(np.argmin(cost)), []
    for ptr, (f, _) in zip(reversed(back), reversed(states[1:])):
        path.append(f[k])
        k = int(ptr[k])
    path.append(states[0][0][k])
    return np.array(path[::-1])


def praat_f0(x, rate, floor=60.0, ceiling=300.0):
    """Per 10 ms frame Hz from Praat (T-007's definition: floor 60, ceiling 300 Hz), nan when unvoiced. The
    autocorrelation trackers read a creaky ending (62-75 Hz on Video F) 20 to 90 Hz too high and see
    almost none of the resets that Praat and the owner's gate count, so Praat is the one the costs use."""
    try:
        import parselmouth
    except ImportError:
        sys.exit("--joint measures pitch with Praat: run it with `uv run --with numpy --with praat-parselmouth`")
    pitch = parselmouth.Sound(x, sampling_frequency=rate).to_pitch(time_step=0.01, pitch_floor=floor, pitch_ceiling=ceiling)
    f0 = pitch.selected_array["frequency"].copy()
    f0[f0 == 0] = np.nan
    return f0


def st_between(a, b):
    return float(12 * np.log2(a / b))


def pitch_ends(f0, hop=0.01, span=0.15):
    """What one tracker says about a sentence's ends: Hz of its first and its last 150 ms of voiced sound, Hz
    of the whole, how far the last 150 ms moved from the 300 ms before it (st; nan with nothing before), and
    whether the ending is creaky (P-2: its median under CREAK_HZ, or half its frames). None with under 8
    voiced frames."""
    ok = ~np.isnan(f0)
    if ok.sum() < 8:
        return None
    t, v = np.flatnonzero(ok) * hop, f0[ok]
    before = v[(t >= t[-1] - span - 0.30) & (t < t[-1] - span)]
    last = v[t >= t[-1] - span]
    final = float(np.median(last))
    return {"onset_hz": float(np.median(v[t <= t[0] + span])), "final_hz": final, "median_hz": float(np.median(v)),
            "final_move_st": st_between(final, np.median(before)) if len(before) else float("nan"),
            "creaky": bool(final < CREAK_HZ or np.mean(last < CREAK_HZ) >= 0.5)}


def tail_rise(f0, hop=0.01):
    """The last 120 ms of voiced sound against the 250 ms before it, in st (nan with either missing): the
    terminal event of B-C03, which the fold in narrate.py's `semitones` hides by construction."""
    ok = ~np.isnan(f0)
    if ok.sum() < 8:
        return float("nan")
    t, v = np.flatnonzero(ok) * hop, f0[ok]
    tail, before = v[t >= t[-1] - 0.12], v[(t >= t[-1] - 0.37) & (t < t[-1] - 0.12)]
    return st_between(np.median(tail), np.median(before)) if len(tail) and len(before) else float("nan")


def read_features(seg, rate):
    """A sentence as it will be assembled, seen by two trackers: Praat, whose numbers the costs use, and the
    Viterbi autocorrelation path as the second opinion (a terminal event needs both; the run's report counts
    the resets each sees)."""
    praat, second = praat_f0(seg, rate), f0_viterbi(seg, rate)
    return {"praat": pitch_ends(praat), "second": pitch_ends(second),
            "tail_praat": tail_rise(praat), "tail_second": tail_rise(second)}


def terminal_event(feats):
    return bool(feats and feats["tail_praat"] > TERMINAL[0] and feats["tail_second"] > TERMINAL[1])


def node_cost(m, feats, question, lead_extra=False, w=JOINT_COST):
    """What a read costs on its own: narrate.py's score and its gates, then the defects the score cannot
    see: an unlicensed pause of HOLE or more, a heard word before the first scripted one, a statement
    that rises at its end, a creaky ending, a terminal pitch event."""
    cost = w["score"] * (1 - m["score"])
    cost += w["gate"] * (bool(m["cut_short"]) + bool(m["sibilant_missing"]) + (m["asr_ratio"] < 0.85))
    cost += w["name"] * m["misheard_names"] + w["hesitation"] * m["hesitations"] + w["stall"] * m["stalls"]
    cost += w["hole"] * sum(1 for p in m["pauses"] if p["len"] > HOLE and p["punct"] not in (",", "."))
    cost += w["onset"] * bool(lead_extra)
    if feats:
        move = feats["praat"]["final_move_st"] if feats["praat"] else float("nan")
        if not question and move == move:
            cost += w["rise"] * min(8.0, max(0.0, move - 2.0))
        cost += w["creak"] * bool(feats["praat"] and feats["praat"].get("creaky")) + w["terminal"] * terminal_event(feats)
    return float(cost)


def join_resets(prev, nxt, tracker="praat"):
    """(reset, register step) across a join in st by one tracker (None, None without it): the next sentence's
    first 150 ms against the last 150 ms before it, and the move of the sentences' medians."""
    p, n = (prev or {}).get(tracker), (nxt or {}).get(tracker)
    if not (p and n):
        return None, None
    return st_between(n["onset_hz"], p["final_hz"]), abs(st_between(n["median_hz"], p["median_hz"]))


def join_cost(prev, nxt, w=JOINT_COST):
    """What a pair of reads does at their join: a reset past +12 st (P-2's price) and a register step past 4 st."""
    reset, step = join_resets(prev, nxt)
    cost = 0.0
    if reset is not None and reset > w["tail_at"]:
        cost += w["tail"] + w["tail_per_st"] * (reset - w["tail_at"])
    if step is not None:
        cost += w["register"] * max(0.0, step - w["register_free"])
    return cost


def best_path(layers, join):
    """The cheapest choice of one read per sentence. layers[i] = [(node cost, features), ...]; join(a, b) is the
    cost of features a followed by features b. A tie goes to the earlier read (narrate.py's order)."""
    cost, back = [c for c, _ in layers[0]], []
    for i in range(1, len(layers)):
        now, ptr = [], []
        for c, feats in layers[i]:
            options = [cost[p] + join(layers[i - 1][p][1], feats) for p in range(len(layers[i - 1]))]
            p = min(range(len(options)), key=options.__getitem__)
            now.append(options[p] + c)
            ptr.append(p)
        cost, back = now, back + [ptr]
    k = min(range(len(cost)), key=cost.__getitem__)
    path = [k]
    for ptr in reversed(back):
        k = ptr[k]
        path.append(k)
    return path[::-1]


def usable_take(work, asr, tag):
    """narrate.py's test for a candidate: the take exists and its transcript belongs to this exact audio."""
    wav = Path(work) / "takes" / f"{tag}.wav"
    if not wav.exists() or tag not in asr:
        return False
    sha = asr[tag].get("sha1")
    return sha is None or sha == hashlib.sha1(wav.read_bytes()).hexdigest()


def lead_extra(sentence, heard):
    """A word the ASR heard before the sentence's first (T-006's 14_3.0: "that" for a stuttered "th-"). A leading
    "and" is the ASR's own, not the read's."""
    want = [t for t, _ in nr.tokens(sentence)]
    got = [t for t, _ in nr.tokens(heard)]
    _, to_want, _ = nr.align(want, got)
    lead = next((j for j in range(len(got)) if j in to_want), len(got))
    return any(t != "and" for t in got[:lead])


def joint_choice(script, work, lexicon, sentence_lufs):
    """Choose every line's reads together and record, in JOINT, the reads that differ from narrate.py's pick.
    Returns the totals the run prints."""
    nr.load_lexicon(nr.resolve_optional(lexicon, "lexicon.json", "lexicon"))
    plan = json.loads((Path(work) / "plan.json").read_text())
    asr = json.loads((Path(work) / "asr.json").read_text())
    excluded = set((Path(work) / "joint_exclude.txt").read_text().split()) if (Path(work) / "joint_exclude.txt").exists() else set()
    total = {"sentences": 0, "moved": 0, "lines": [], "joins": {"praat": 0, "second": 0},
             "bad_joins": {"praat": [0, 0], "second": [0, 0]}}  # past +12 st, [narrate.py's picks, the joint picks]
    for line_id, sents in plan.items():
        layers, reads, best = [], [], []
        for s in sents:
            cands = []
            for r in s["reads"]:
                wav = Path(work) / "takes" / f"{r['tag']}.wav"
                if not usable_take(work, asr, r["tag"]):
                    continue
                m = analyze(wav, asr[r["tag"]], r["text"])
                if m:
                    cands.append((r, wav, m))
            if not cands:
                break  # narrate.py's choose stops on this line; leave it to say so
            forced = [c for c in cands if c[0]["tag"] in PICKS]
            cands = forced or cands
            top = max(range(len(cands)), key=lambda i: cands[i][2]["score"])  # the first of equal scores, like its stable sort
            layer = []
            for i, (r, wav, m) in enumerate(cands):
                seg, *_ = edit(wav, m, sentence_lufs)
                feats = read_features(seg, nr.RATE)
                node = node_cost(m, feats, s["text"].rstrip().endswith("?"), lead_extra(r["text"], m["heard"]))
                node += JOINT_COST["gate"] * (r["tag"] in excluded and i != top)  # failed a line-level gate once
                layer.append((node + JOINT_COST["change"] * (i != top), feats))
            layers.append(layer)
            reads.append(cands)
            best.append(top)
        else:
            path = best_path(layers, join_cost)
            for i, k in enumerate(path):
                total["sentences"] += 1
                if k != best[i]:
                    total["moved"] += 1
                    total["lines"].append(f"{line_id}.{sents[i]['j']}")
                    JOINT[reads[i][k][0]["tag"]] = {
                        "from": reads[i][best[i]][0]["tag"], "node": round(layers[i][k][0], 3),
                        "join": round(join_cost(layers[i - 1][path[i - 1]][1], layers[i][k][1]), 3) if i else 0.0}
            for tracker in ("praat", "second"):
                for when, picks in enumerate((best, path)):
                    resets = [join_resets(layers[i - 1][picks[i - 1]][1], layers[i][picks[i]][1], tracker)[0]
                              for i in range(1, len(picks))]
                    total["bad_joins"][tracker][when] += sum(1 for r in resets if r is not None and r > JOINT_COST["tail_at"])
                    if when == 0:
                        total["joins"][tracker] += sum(1 for r in resets if r is not None)
    return total


def edge_failures(script, out, lexicon):
    """(line id, what is wrong) for every sentence the report does not bind to the canonical script, and
    every first or last word of a sentence that is missing from its assembled line's transcript."""
    nr.load_lexicon(nr.resolve_optional(lexicon, "lexicon.json", "lexicon"))
    report = json.loads((out / "report.json").read_text())["lines"]
    heard = json.loads((out / "lines_asr.json").read_text())
    scripted = dict(nr.checked_script(script))
    missing = [(lid, f"{lid}: not in the script") for lid in report if lid not in scripted]
    for lid, text in scripted.items():
        expected = nr.sentences(text)
        sents = report.get(lid, [])
        if len(sents) != len(expected):
            missing.append((lid, f"{lid}: report has {len(sents)} sentences, script has {len(expected)}"))
        for j, (sentence, recorded) in enumerate(zip(expected, sents)):
            if recorded.get("j") != j:
                missing.append((lid, f"{lid}.{j}: report sentence index is {recorded.get('j')!r}"))
            want = [t for t, _ in nr.tokens(sentence)]
            have = [t for t, _ in nr.tokens(recorded.get("sentence", ""))]
            if have != want:
                missing.append((lid, f"{lid}.{j}: report sentence differs from the script | {sentence}"))
        if not heard.get(lid, {}).get("text", "").strip():
            missing.append((lid, f"{lid}: no transcript in lines_asr.json"))
    for lid, sents in report.items():
        want, spans = [], []
        for s in sents:
            words = [t for t, _ in nr.tokens(s["sentence"])]
            if words:
                spans.append((s["j"], len(want), len(want) + len(words) - 1, s["sentence"]))
                want += words
        text = heard.get(lid, {}).get("text", "").replace("%", " percent")  # the ASR writes the sign
        _, _, hit = nr.align(want, [t for t, _ in nr.tokens(text)])
        missing += [(lid, f"{lid}.{j}: {which} word {want[k]!r} is not in the line's transcript | {sentence}")
                    for j, a, b, sentence in spans for k, which in ((a, "first"), (b, "last")) if k not in hit]
    return missing


def edges(script, out, lexicon):
    """Bind every ordered sentence to the canonical script, then check its first and last word
    against the assembled line's transcript. Nothing to check is not a pass."""
    missing = edge_failures(script, out, lexicon)
    for _, m in missing:
        print("  ", m)
    print(f"edges: {len(missing)} coverage or first/last-word failures -> "
          f"{'FAIL (look at the sheet: cut off, misread, or the ASR spelling, which goes in the lexicon)' if missing else 'PASS'}")
    sys.exit(1 if missing else 0)


def exclude(script, work, out, lexicon):
    """After a `choose --joint` build: a line that fails a line-level gate (qa_lines.py's, or edges: the
    assembled line's transcript can lose a first word that the take's own transcript has, P-2's lost
    "But") gives up the reads the joint choice moved in it. They go to work/joint_exclude.txt, which the
    next `choose --joint` prices as a failed gate; the loop ends when a failing line holds none that is
    new, and a line with every moved read excluded is narrate.py's. Exits 1 when nothing new was excluded."""
    out, work = Path(out), Path(work)
    report = json.loads((out / "report.json").read_text())["lines"]
    failing = {lid for lid, _ in edge_failures(script, out, lexicon)}
    if (out / "qa.json").exists():
        failing |= {r["line"] for r in json.loads((out / "qa.json").read_text())["failing"]}
    path = work / "joint_exclude.txt"
    known = set(path.read_text().split()) if path.exists() else set()
    new = [c["read"] for lid in sorted(failing) for c in report.get(lid, []) if "joint" in c and c["read"] not in known]
    if new:
        path.write_text("".join(f"{tag}\n" for tag in sorted(known | set(new))))
    print(f"exclude: {len(failing)} lines fail a line-level gate; {len(new)} reads the joint choice moved there "
          f"are now excluded ({len(known) + len(new)} in all)")
    sys.exit(0 if new else 1)


def option(name, default=None):
    """The value of `name VALUE` or `name=VALUE` on the command line, left in place."""
    for i, arg in enumerate(sys.argv):
        if arg == name and i + 1 < len(sys.argv):
            return sys.argv[i + 1]
        if arg.startswith(name + "="):
            return arg.split("=", 1)[1]
    return default


def take_value(name, default):
    """option(), removed from the command line: narrate.py's parser would refuse what is ours."""
    value = option(name, default)
    for i, arg in enumerate(sys.argv):
        if arg == name or arg.startswith(name + "="):
            del sys.argv[i:i + (2 if arg == name else 1)]
            break
    return value


def take_flag(name):
    if name in sys.argv:
        sys.argv.remove(name)
        return True
    return False


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
    if len(sys.argv) >= 6 and sys.argv[2] == "exclude":
        exclude(sys.argv[3], sys.argv[4], sys.argv[5], option("--lexicon"))
    if len(sys.argv) >= 4 and sys.argv[2] == "synth":
        synth(sys.argv[3], take_value("--voice", "robert"), take_value("--model", os.environ.get("FTTS_MODEL", "")),
              take_value("--workers", "8"))
        sys.exit(0)
    if len(sys.argv) < 3 or sys.argv[2] not in ("plan", "choose"):
        sys.exit(__doc__)
    command, out = sys.argv[2], Path(sys.argv[5]) if len(sys.argv) > 5 else None
    joint, seeds = take_flag("--joint"), int(take_value("--seeds", "0"))
    if (joint and command != "choose") or (seeds and command != "plan"):
        sys.exit("--joint belongs to choose and --seeds to plan")
    nr.tts_variants, nr.analyze, nr.edit, nr.limit_peaks = tts_variants, analyze, edit, counted_limit
    director_files(sys.argv[3], sys.argv[4], command)
    if joint:
        total = joint_choice(sys.argv[3], sys.argv[4], option("--lexicon"), float(option("--sentence-lufs", "-23")))
        bad = total["bad_joins"]
        print(f"joint: {total['moved']} of {total['sentences']} sentences are read differently from narrate.py's pick"
              + (f" ({', '.join(total['lines'])})" if total["lines"] else "") + f"; joins that reset the pitch past "
              f"+{JOINT_COST['tail_at']:.0f} st: {bad['praat'][0]} -> {bad['praat'][1]} of {total['joins']['praat']} by Praat, "
              f"{bad['second'][0]} -> {bad['second'][1]} of {total['joins']['second']} by the second tracker (held out)")
    sys.argv = [str(Path(sys.argv[1]) / "narrate.py"), *sys.argv[2:]]
    nr.main()
    if command == "plan":
        planned = add_seeds(sys.argv[3], seeds)  # also empties a seeds.tsv left by an earlier run
        if seeds:
            print(f"seeds: {planned} more reads (seed 1..{seeds}) -> {Path(sys.argv[3]) / 'seeds.tsv'}"
                  f" (make them with: narration.py SKILL_SCRIPTS synth {sys.argv[3]})")
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
