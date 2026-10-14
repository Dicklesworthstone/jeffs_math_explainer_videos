#!/usr/bin/env python3
"""tools/prosody_qa.py on synthetic narrations with planted defects, and on the frozen files of the
research that set its bands. No TTS or ASR runs.

    uv run --no-project --with numpy --with praat-parselmouth python3 -I tools/test_prosody_qa.py

Synthetic sound tests the measures and the verdicts, not whether a real read sounds right: a
constant-gap track, a digital-black track, a +14 st reset on every join, a 1.3 s hole in a
sentence and a sentence that ends with a rise must each be caught (the hole fails, the rest are
flagged: only a hole inside a sentence gates), and a track with human-sized resets, varied gaps
and room tone must not be.

The reproduction tests need the research's frozen files and are skipped without them:
  PROSODY_QA_T007   the T-007 directory: wav/R1..R4.wav (human reads) and measure.json (its rows)
  PROSODY_QA_FROZEN a directory with mental_math_out/ (Video F's narration/out) and
                    mental_math_scored/ (its renders/scored timelines and cues)
Both default to the build host's copies under ~/jmev_orch/narr_exp. They check that the tool's
numbers on those files are inside the confidence intervals of T-007's own, so a change to a
tracker, a window or a threshold that moves a measure fails here.
"""
import json
import math
import os
import subprocess
import sys
import tempfile
import unittest
import wave
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import prosody_qa as qa  # noqa: E402

TOOL = Path(__file__).resolve().parent / "prosody_qa.py"
RATE = 24000
GAPS = [0.2, 0.35, 0.5, 0.8, 1.2, 0.3, 2.0, 0.6, 0.25, 0.9, 1.6, 0.4]  # a spread like a person's: CV 0.7
HOME = Path.home() / "jmev_orch" / "narr_exp"


# ---------------------------------------------------------------- synthetic narrations
def tone(points, amp=0.1):
    """a harmonic tone (12 harmonics at 1/h) on an f0 contour [(seconds, Hz)], interpolated in log f0, with 30 ms fades"""
    t_end = points[-1][0]
    n = int(t_end * RATE)
    t = np.arange(n) / RATE
    f0 = np.exp(np.interp(t, [p[0] for p in points], [math.log(p[1]) for p in points]))
    phase = 2 * np.pi * np.cumsum(f0) / RATE
    x = sum(np.sin(h * phase) / h for h in range(1, 13))
    x *= np.minimum(1.0, np.minimum(t, t_end - t) / 0.03)
    return amp * x / np.abs(x).max()


def sentence_audio(start, end, dur=1.4, rise=0.0, hole=None, floor=None, rng=None):
    """one sentence: starts at `start` Hz, ends at `end` Hz (the first and last 150 ms are flat there); rise st: the last 130 ms jump;
    hole (at s, length s): a silence inside it"""
    pts = [(0.0, start), (0.3 * dur, start), (0.6 * dur, end), (dur, end)]
    if rise:
        pts = [(0.0, start), (0.3 * dur, start), (0.55 * dur, end), (dur - 0.15, end), (dur - 0.14, end * 2 ** (rise / 12)), (dur, end * 2 ** (rise / 12))]
    x = tone(pts)
    if hole:
        at, length = hole
        i = int(at * RATE)
        x = np.concatenate([x[:i], silence(length, floor, rng), x[i:]])
    return x


def silence(seconds, floor=None, rng=None):
    n = int(round(seconds * RATE))
    if floor is None:
        return np.zeros(n)
    return rng.normal(0.0, 10 ** (floor / 20), n)


def write_wav(path, x):
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes((np.clip(x, -1, 1) * 32767).astype("<i2").tobytes())


def write_narration(root, lines, floor=None):
    """lines: {id: [dict(text, start, end, gap, [rise, hole])]}; the gap after a sentence is `gap` seconds of digital zero, or of white
    noise at `floor` dBFS. Writes report.json, spoken.tsv and the WAVs the way narration.py's choose does."""
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    rng = np.random.default_rng(3)
    report, spoken = {}, []
    for lid, sents in lines.items():
        parts, rows = [], []
        for j, s in enumerate(sents):
            x = sentence_audio(s["start"], s["end"], s.get("dur", 1.4), s.get("rise", 0.0), s.get("hole"), floor, rng)
            gap = silence(s["gap"], floor, rng) if j + 1 < len(sents) else np.zeros(0)
            parts += [x, gap]
            rows.append({"j": j, "sentence": s["text"], "out_samples": len(x), "gap_after_samples": len(gap)})
        write_wav(root / f"{lid}.wav", np.concatenate(parts))
        report[lid] = rows
        spoken.append(f"{lid}\t{' '.join(s['text'] for s in sents)}")
    (root / "report.json").write_text(json.dumps({"meta": {"rate": RATE}, "lines": report}), encoding="utf-8")
    (root / "spoken.tsv").write_text("\n".join(spoken) + "\n", encoding="utf-8")
    return root


def two_sentence_lines(n, reset_st, gaps, texts=("That is one thing to keep.", "And this is the next thing.")):
    """n lines of two sentences: the first ends at 110 Hz, the second starts reset_st above that"""
    return {f"{k // 4 + 1:02d}_{k % 4 + 1}": [
        {"text": texts[0], "start": 150.0, "end": 110.0, "gap": gaps[k % len(gaps)]},
        {"text": texts[1], "start": 110.0 * 2 ** (reset_st / 12), "end": 105.0, "gap": 0.0}] for k in range(n)}


def run_tool(*args):
    """the tool as a command, bounded: its exit code is the thing under test, so a non-zero one is not an error here"""
    return subprocess.run([sys.executable, "-I", str(TOOL), *map(str, args)], capture_output=True, text=True, check=False, timeout=300)


def check(checks, name, scope="video"):
    return next(c for c in checks if c["check"] == name and c["scope"] == scope)


def audit(root, gates=(), **kw):
    res = qa.scan(root, **kw)
    return res, qa.verdicts(res, gates)


# ---------------------------------------------------------------- the measures
class Measures(unittest.TestCase):
    def test_levels_and_runs(self):
        x = np.concatenate([0.1 * np.sin(2 * np.pi * 200 * np.arange(RATE) / RATE), np.zeros(RATE // 2)])
        tl, lev = qa.levels(x, RATE)
        self.assertAlmostEqual(float(np.median(lev[:90])), 10 * math.log10(0.1 ** 2 / 2), delta=0.1)
        self.assertEqual(float(lev[-5]), -120.0)  # digital silence
        self.assertEqual(qa.runs_of([0, 1, 1, 0, 1]), [(1, 3), (4, 5)])
        self.assertEqual(qa.runs_of([1, 1]), [(0, 2)])
        self.assertEqual(qa.runs_of([]), [])

    def test_wilson_and_describe(self):
        lo, hi = qa.wilson(15, 100)
        self.assertAlmostEqual(lo, 0.093, delta=0.003)
        self.assertAlmostEqual(hi, 0.233, delta=0.003)
        self.assertTrue(math.isnan(qa.wilson(0, 0)[0]))
        d = qa.describe([0.38, 0.38, 0.38, None, math.nan])
        self.assertEqual((d["n"], d["distinct"]), (3, 1))
        self.assertAlmostEqual(d["cv"], 0.0, places=9)
        self.assertIsNone(qa.describe([]))

    def test_two_trackers_must_agree(self):
        self.assertEqual(qa.consensus(8.0, 9.0), {"reset": 8.5, "status": "agreed"})
        self.assertEqual(qa.consensus(8.0, 17.0), {"reset": None, "status": "disputed"})  # an octave slip in one tracker
        self.assertEqual(qa.consensus(8.0, None), {"reset": None, "status": "single"})
        self.assertEqual(qa.consensus(None, None), {"reset": None, "status": "none"})

    def test_a_planted_reset_is_read_back(self):
        for planted in (12.0, 6.0, -5.0):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as d:
                write_narration(d, two_sentence_lines(1, planted, [0.4]))
                res = qa.scan(d, bounds=(60.0, 300.0))
                (join,) = res["joins"]
                for kind in ("ac", "cc", "reset"):
                    self.assertAlmostEqual(join[kind if kind == "reset" else f"reset_{kind}"], planted, delta=0.6, msg=kind)
                self.assertEqual(join["status"], "agreed")
                self.assertAlmostEqual(join["gap"], 0.4, delta=0.06)

    def test_a_join_without_a_gap_is_still_measured(self):
        with tempfile.TemporaryDirectory() as d:
            write_narration(d, two_sentence_lines(1, 7.0, [0.0]))
            (join,) = qa.scan(d, bounds=(60.0, 300.0))["joins"]
            self.assertAlmostEqual(join["reset_ac"], 7.0, delta=0.8)

    def test_a_report_that_does_not_describe_the_wav_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            write_narration(d, two_sentence_lines(1, 6.0, [0.4]))
            rep = json.loads((Path(d) / "report.json").read_text())
            rep["lines"]["01_1"][0]["out_samples"] += 4800
            (Path(d) / "report.json").write_text(json.dumps(rep))
            with self.assertRaises(ValueError):
                qa.scan(d)


# ---------------------------------------------------------------- planted negatives and their controls
class Planted(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        t = Path(cls.tmp.name)
        # the control: human-sized resets (+6 st), varied gaps, room tone at -62 dBFS
        cls.good = write_narration(t / "good", two_sentence_lines(24, 6.0, GAPS), floor=-62.0)
        cls.good_res, cls.good_checks = audit(cls.good, bounds=(60.0, 300.0))
        # planted: +14 st on every join
        cls.jumpy = write_narration(t / "jumpy", two_sentence_lines(24, 14.0, GAPS), floor=-62.0)
        # planted: one gap length for every join (with room tone, so only the regularity is wrong)
        cls.constant = write_narration(t / "constant", two_sentence_lines(24, 6.0, [0.38]), floor=-62.0)
        # planted: digital black between sentences (varied gaps, so only the floor is wrong)
        cls.black = write_narration(t / "black", two_sentence_lines(24, 6.0, GAPS), floor=None)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_control_passes_everything(self):
        self.assertEqual([c["check"] for c in self.good_checks if c["verdict"] in ("FAIL", "FLAG")], [])
        share = check(self.good_checks, "reset_share")
        self.assertEqual(share["verdict"], "PASS")
        self.assertEqual(share["value"], 0.0)
        floor = check(self.good_checks, "silence_floor")
        self.assertAlmostEqual(floor["value"], -62.0, delta=2.5)
        self.assertGreater(check(self.good_checks, "gap_regularity")["value"], qa.BANDS["gap_cv_min"])

    def test_resets_beyond_twelve_are_flagged_and_never_fail(self):
        res, checks = audit(self.jumpy, bounds=(60.0, 300.0))
        share = check(checks, "reset_share")
        self.assertEqual(share["verdict"], "FLAG")
        self.assertGreater(share["value"], 0.9)
        self.assertIn("LibriVox alone 16-26%", share["band"])  # shown beside the human readers' range
        run = lambda *a: run_tool(self.jumpy, *a)
        self.assertEqual(run().returncode, 0)
        self.assertEqual(run("--gate-room-tone", "--gate-regularity", "--gate-terminal", "--gate-gap-targets").returncode, 0)  # no gate for it

    def test_few_resets_are_advisory(self):
        with tempfile.TemporaryDirectory() as d:
            write_narration(d, two_sentence_lines(5, 14.0, GAPS), floor=-62.0)
            _, checks = audit(d, bounds=(60.0, 300.0))
            self.assertEqual(check(checks, "reset_share")["verdict"], "ADVISORY")

    def test_a_constant_gap_track_is_flagged(self):
        res, checks = audit(self.constant, bounds=(60.0, 300.0))
        gap = check(checks, "gap_regularity")
        self.assertEqual(gap["verdict"], "FLAG")
        self.assertLess(gap["value"], 0.1)
        self.assertEqual(check(checks, "silence_floor")["verdict"], "PASS")  # nothing else is wrong with it
        self.assertEqual(check(checks, "reset_share")["verdict"], "PASS")
        _, gated = audit(self.constant, ("regularity",), bounds=(60.0, 300.0))
        self.assertEqual(check(gated, "gap_regularity")["verdict"], "FAIL")

    def test_a_digital_black_track_is_flagged(self):
        res, checks = audit(self.black, bounds=(60.0, 300.0))
        floor = check(checks, "silence_floor")
        self.assertEqual(floor["verdict"], "FLAG")
        self.assertLess(floor["value"], -100.0)
        self.assertIn("100% are exact zeros", floor["note"])
        self.assertEqual(check(checks, "gap_regularity")["verdict"], "PASS")
        _, gated = audit(self.black, ("room-tone",), bounds=(60.0, 300.0))
        self.assertEqual(check(gated, "silence_floor")["verdict"], "FAIL")

    def test_the_exit_code_follows_the_verdicts(self):
        run = lambda *a: run_tool(*a)
        self.assertEqual(run(str(self.good)).returncode, 0)
        self.assertEqual(run(str(self.black)).returncode, 0)  # a flag does not fail
        self.assertEqual(run(str(self.black), "--gate-room-tone").returncode, 1)
        self.assertEqual(run(str(self.constant), "--gate-regularity").returncode, 1)
        self.assertEqual(run(str(self.good), "--gate-room-tone", "--gate-regularity", "--gate-terminal").returncode, 0)
        missing = run(str(Path(self.tmp.name) / "nothing"))
        self.assertEqual(missing.returncode, 2)
        self.assertIn("prosody_qa:", missing.stderr)
        for name, text in (("bad", "{"), ("norate", '{"meta": {"rate": 0}, "lines": {}}'), ("nolines", '{"meta": {"rate": 24000}, "lines": []}')):
            broken = Path(self.tmp.name) / name
            broken.mkdir()
            (broken / "report.json").write_text(text, encoding="utf-8")
            self.assertEqual(run(str(broken)).returncode, 2, name)
            self.assertIn("report.json", run(str(broken)).stderr, name)

    def test_json_carries_every_row(self):
        out = Path(self.tmp.name) / "q.json"
        r = run_tool(self.good, "--json", out)
        self.assertEqual(r.returncode, 0, r.stderr)
        data = json.loads(out.read_text())
        self.assertEqual(len(data["result"]["joins"]), 24)
        self.assertIn("reset_share", [c["check"] for c in data["checks"]])


class ResetShare(unittest.TestCase):
    """the share beyond +12 st is shown beside the human readers' range and flagged above it (audit G-04, PP-2): never a gate"""

    def verdict(self, beyond, total):
        joins = [{"reset": 14.0 if k < beyond else 5.0, "status": "agreed", "level_db": None, "gap": 0.5, "chapter": "01"} for k in range(total)]
        res = {"joins": joins, "chapters": ["01"], "inner_pauses": [], "floors": [], "line_gaps": [], "chapter_joins": [], "sentences": []}
        return check(qa.verdicts(res, ("room-tone", "regularity", "terminal", "gap-targets")), "reset_share")

    def test_a_librivox_sized_share_is_not_flagged(self):
        self.assertEqual(self.verdict(4, 50)["verdict"], "PASS")  # 8%, the pooled human readers
        self.assertEqual(self.verdict(8, 50)["verdict"], "PASS")  # 16%, LibriVox alone in T-007
        self.assertEqual(self.verdict(13, 50)["verdict"], "PASS")  # 26%, LibriVox alone in audit-7

    def test_above_the_highest_human_reading_is_flagged_not_failed(self):
        flagged = self.verdict(15, 50)  # 30%, Video F
        self.assertEqual(flagged["verdict"], "FLAG")
        self.assertAlmostEqual(flagged["value"], 0.30)

    def test_a_short_video_is_advisory(self):
        self.assertEqual(self.verdict(10, qa.MIN_JOINS - 1)["verdict"], "ADVISORY")
        self.assertEqual(self.verdict(0, 0)["verdict"], "ADVISORY")


class InnerPauses(unittest.TestCase):
    def pauses(self, text, hole):
        with tempfile.TemporaryDirectory() as d:
            lines = {"01_1": [{"text": text, "start": 140.0, "end": 110.0, "gap": 0.0, "dur": 2.2, "hole": hole}]}
            write_narration(d, lines, floor=-62.0)
            return audit(d, bounds=(60.0, 300.0))

    def test_a_hole_over_a_second_fails(self):
        res, checks = self.pauses("That is one thing to keep.", (1.0, 1.3))
        self.assertEqual(check(checks, "inner_pause_over_1s")["verdict"], "FAIL")
        self.assertAlmostEqual(res["inner_pauses"][0]["len"], 1.3, delta=0.1)

    def test_an_unlicensed_stop_is_flagged_and_a_comma_licenses_it(self):
        _, checks = self.pauses("That is one thing to keep.", (1.0, 0.5))
        self.assertEqual(check(checks, "inner_pause_unlicensed")["verdict"], "FLAG")
        self.assertEqual(check(checks, "inner_pause_over_1s")["verdict"], "PASS")
        _, checks = self.pauses("That is one thing, and then another.", (1.0, 0.5))
        self.assertEqual(check(checks, "inner_pause_unlicensed")["verdict"], "PASS")

    def test_a_short_stop_and_a_hyphenated_number_are_not_flagged(self):
        _, checks = self.pauses("Eighty-five squared is a number.", (1.0, 0.2))
        self.assertEqual(check(checks, "inner_pause_unlicensed")["verdict"], "PASS")
        _, checks = self.pauses("Eighty-five squared is a number.", (1.0, 0.45))
        self.assertEqual(check(checks, "inner_pause_unlicensed")["verdict"], "FLAG")  # the hyphen is not a licence

    def test_the_silence_of_a_join_is_not_an_inner_pause(self):
        with tempfile.TemporaryDirectory() as d:
            write_narration(d, two_sentence_lines(2, 6.0, [2.5]), floor=-62.0)
            res, checks = audit(d, bounds=(60.0, 300.0))
            self.assertEqual(res["inner_pauses"], [])
            self.assertEqual(check(checks, "inner_pause_over_1s")["verdict"], "PASS")


class TerminalEvents(unittest.TestCase):
    def events(self, text, rise):
        with tempfile.TemporaryDirectory() as d:
            lines = {"01_1": [{"text": text, "start": 140.0, "end": 100.0, "gap": 0.0, "dur": 1.6, "rise": rise}]}
            write_narration(d, lines, floor=-62.0)
            return audit(d, bounds=(60.0, 300.0))

    def test_a_sentence_that_ends_with_a_rise_is_an_event(self):
        res, checks = self.events("That is one thing to keep.", 14.0)
        (s,) = res["sentences"]
        self.assertTrue(s["event"])
        self.assertGreater(s["rise_ac"], 12.0)
        self.assertGreater(s["rise_cc"], 12.0)
        self.assertEqual(check(checks, "terminal_events")["verdict"], "FLAG")
        self.assertEqual(check(checks, "terminal_events")["value"], 1)
        self.assertEqual(check(qa.verdicts(res, ("terminal",)), "terminal_events")["verdict"], "FAIL")

    def test_a_falling_ending_is_none(self):
        res, checks = self.events("That is one thing to keep.", 0.0)
        self.assertFalse(res["sentences"][0]["event"])
        self.assertEqual(check(checks, "terminal_events")["verdict"], "PASS")

    def test_a_question_may_rise(self):
        res, checks = self.events("Is that one thing to keep?", 14.0)
        self.assertTrue(res["sentences"][0]["question"])
        self.assertEqual(check(checks, "terminal_events")["verdict"], "PASS")
        self.assertEqual(check(checks, "terminal_unconfirmed")["verdict"], "ADVISORY")


# ---------------------------------------------------------------- the silence between lines
def write_timeline(root, chapter, lines, listens=(), breaths=(), marks=(), tail=3.0):
    """lines: [(id, start s, length s)]; the chapter ends `tail` s after its last line"""
    root = Path(root)
    last = lines[-1]
    rows = ["# explainer timeline v1", f"chapter\t{chapter}", "fps\t60", f"frames\t{int(round((last[1] + last[2] + tail) * 60))}"]
    rows += [f"line\t{k}\t{int(round(s * 60))}\t0\t/x/{k}.wav" for k, s, _ in lines]
    rows += [f"mark\t{m}\t{int(round(s * 60))}" for m, s in marks]
    rows += [f"breath\t{int(round(s * 60))}\t{n}" for s, n in breaths]
    rows += [f"listen\t{int(round(s * 60))}\t{n}\tx\t/x/s.wav" for s, n in listens]
    (root / f"{chapter}.timeline.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    (root / f"{chapter}.cues.tsv").write_text("".join(f"{k}\t{s:.3f}\t{n:.3f}\n" for k, s, n in lines), encoding="utf-8")


SCRIPT = {
    "01_1": "Quick question. What's eighty-five squared?",
    "01_2": "Seven thousand two hundred twenty-five.",
    "01_3": "And that took two steps.",
    "01_4": "Your turn. Try one.",
    "01_5": "Take twenty-five.",
    "01_6": "It ends in five",
    "01_7": "so the answer ends in twenty-five.",
    "01_8": "Here's the next one.",
}


class LineGaps(unittest.TestCase):
    def test_classes_follow_the_windows_then_the_text(self):
        for prev, nxt, window, want in (
            ("A thing.", "Your turn. Try one.", None, "TOPIC"),
            ("A thing.", "Take twenty-five.", None, "TOPIC"),
            ("A thing.", "And that took two steps.", None, "CONTINUATION"),
            ("It ends in five", "so the answer.", None, "CONTINUATION"),  # the line before is unfinished
            ("A thing.", "That's the whole trick.", None, "CONTINUATION"),
            ("A thing?", "Seven.", None, "BEAT"),
            ("A thing.", "Seven.", None, "BEAT"),
            ("A thing.", "Your turn.", "QUIZ", "QUIZ"),
        ):
            self.assertEqual(qa.gap_class(prev, nxt, window), want, (prev, nxt))

    def test_gaps_windows_and_chapter_joins_from_a_timeline(self):
        with tempfile.TemporaryDirectory() as d:
            lines = [("01_1", 0.6, 3.0), ("01_2", 4.2, 2.0), ("01_3", 6.8, 1.5), ("01_4", 8.9, 2.0), ("01_5", 13.5, 1.0), ("01_6", 15.1, 1.5),
                     ("01_7", 17.8, 2.0), ("01_8", 21.2, 1.0)]
            write_timeline(d, "01_first", lines, listens=[(11.0, 2)], marks=[("predict", 17.0)], breaths=[(20.0, 1.5)], tail=3.0)
            write_timeline(d, "02_second", [("02_1", 0.6, 2.0)], tail=2.5)
            gaps, joins = qa.line_gaps(qa.read_timeline(d), {**SCRIPT, "02_1": "Next."})
            by = {(g["from"], g["to"]): g for g in gaps}
            self.assertEqual(by["01_1", "01_2"]["class"], "BEAT")
            self.assertAlmostEqual(by["01_1", "01_2"]["len"], 0.6, delta=0.02)
            self.assertEqual(by["01_2", "01_3"]["class"], "CONTINUATION")
            self.assertEqual(by["01_3", "01_4"]["class"], "TOPIC")
            self.assertEqual(by["01_4", "01_5"]["class"], "QUIZ")
            self.assertEqual(by["01_5", "01_6"]["class"], "BEAT")
            self.assertEqual(by["01_6", "01_7"]["class"], "PREDICT")
            self.assertEqual(by["01_7", "01_8"]["class"], "WINDOW")
            (join,) = joins
            self.assertAlmostEqual(join["len"], 3.0 + 0.6, delta=0.05)  # this chapter's tail, the next one's lead

    def test_a_timeline_without_a_frame_rate_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            write_timeline(d, "01_first", [("01_1", 0.6, 3.0), ("01_2", 4.2, 2.0)])
            path = Path(d) / "01_first.timeline.tsv"
            path.write_text(path.read_text().replace("fps\t60", "fps\t0"), encoding="utf-8")
            with self.assertRaises(ValueError):
                qa.read_timeline(d)
            write_narration(Path(d) / "out", two_sentence_lines(1, 6.0, [0.4]))
            r = run_tool(Path(d) / "out", "--timeline", d)
            self.assertEqual(r.returncode, 2)
            self.assertIn("fps", r.stderr)

    def plain_res(self, lengths):
        gaps = [{"chapter": "01", "from": "a", "to": "b", "start": 0.0, "len": n, "class": c} for c, n in lengths]
        return {"joins": [], "chapters": ["01"], "inner_pauses": [], "floors": [], "line_gaps": gaps, "chapter_joins": [], "sentences": []}

    def test_one_value_for_every_class_is_flagged(self):
        flat = [(c, 0.61) for c in ("TOPIC", "BEAT", "CONTINUATION") for _ in range(6)]
        checks = qa.verdicts(self.plain_res(flat))
        self.assertEqual(check(checks, "line_gap_regularity")["verdict"], "FLAG")
        self.assertEqual(check(qa.verdicts(self.plain_res(flat), ("regularity",)), "line_gap_regularity")["verdict"], "FAIL")

    def test_gaps_that_follow_the_thought_pass_and_meet_the_targets(self):
        varied = [("TOPIC", 0.99 + e) for e in (-0.1, 0.0, 0.1, 0.2)] + [("BEAT", 0.81 + e) for e in (-0.2, 0.0, 0.1, 0.3, -0.1, 0.05)] \
            + [("CONTINUATION", 0.58 + e) for e in (-0.15, 0.0, 0.05, 0.1, 0.25, -0.05)]
        checks = qa.verdicts(self.plain_res(varied), ("gap-targets",))
        self.assertEqual(check(checks, "line_gap_regularity")["verdict"], "PASS")
        self.assertEqual(check(checks, "line_gap_targets")["verdict"], "PASS")

    def test_targets_fail_on_the_flat_value_only_when_asked(self):
        flat = [(c, 0.61) for c in ("TOPIC", "BEAT", "CONTINUATION") for _ in range(6)]
        self.assertNotIn("line_gap_targets", [c["check"] for c in qa.verdicts(self.plain_res(flat))])
        self.assertEqual(check(qa.verdicts(self.plain_res(flat), ("gap-targets",)), "line_gap_targets")["verdict"], "FAIL")

    def test_long_plain_gaps_are_listed_not_failed(self):
        gaps = [("BEAT", 0.6)] * 9 + [("BEAT", 1.54), ("CONTINUATION", 0.7), ("TOPIC", 0.9)]
        waits = check(qa.verdicts(self.plain_res(gaps)), "line_gap_over_1s")
        self.assertEqual((waits["verdict"], waits["value"]), ("ADVISORY", 1))

    def test_chapter_joins_that_are_copies_are_flagged(self):
        res = self.plain_res([])
        res["chapter_joins"] = [{"from": f"{k}", "to": f"{k + 1}", "len": v} for k, v in enumerate([3.52, 3.52, 3.53, 3.52, 3.53, 3.3, 5.5, 3.52])]
        self.assertEqual(check(qa.verdicts(res), "chapter_join_regularity")["verdict"], "FLAG")
        res["chapter_joins"] = [{"from": "a", "to": "b", "len": v} for v in (2.1, 3.5, 2.8, 4.4, 3.0, 5.2, 2.6)]
        self.assertEqual(check(qa.verdicts(res), "chapter_join_regularity")["verdict"], "PASS")


# ---------------------------------------------------------------- the frozen files of the research
def frozen(var, default):
    return Path(os.environ.get(var, default)).expanduser()


T007 = frozen("PROSODY_QA_T007", HOME / "T-007")
FROZEN = frozen("PROSODY_QA_FROZEN", HOME / "frozen" / "prosody_qa")


def boot_median_ci(v, n=4000, seed=1):
    rng = np.random.default_rng(seed)
    m = [np.median(rng.choice(v, len(v))) for _ in range(n)]
    return float(np.percentile(m, 2.5)), float(np.percentile(m, 97.5))


@unittest.skipUnless((T007 / "measure.json").exists() and (T007 / "wav" / "R4.wav").exists(), "T-007's frozen files are not here (PROSODY_QA_T007)")
class HumanReads(unittest.TestCase):
    """the four human reads, at T-007's sentence boundaries: the tool's resets are T-007's"""

    @classmethod
    def setUpClass(cls):
        data = json.loads((T007 / "measure.json").read_text())
        rows, info = data["rows"], data["info"]
        cls.mine = []  # (T-007's reset, the tool's per tracker and consensus, T-007's level step, the tool's)
        for key in ("R1", "R2", "R3", "R4"):
            x, rate = qa.read_wav(T007 / "wav" / f"{key}.wav")
            tr = qa.Track(x, rate, tuple(info[key]["pitch_bounds"]))
            s = sorted((r for r in rows if r["src"] == key and r["type"] == "S"), key=lambda r: r["t"])
            for k, r in enumerate(s):
                a, b = r["t"] - r["gap"] / 2, r["t"] + r["gap"] / 2
                lo = s[k - 1]["t"] + s[k - 1]["gap"] / 2 if k else a - qa.SPAN
                hi = s[k + 1]["t"] - s[k + 1]["gap"] / 2 if k + 1 < len(s) else b + qa.SPAN
                cls.mine.append((r["reset_st"], qa.measure_join(tr, a, b, lo, hi), r["level_db"]))
        cls.theirs = np.array([m[0] for m in cls.mine if m[0] is not None])

    def test_each_join_is_t007s_within_a_semitone(self):
        diff = np.array([abs(t - m["reset_ac"]) for t, m, _ in self.mine if t is not None and m["reset_ac"] is not None])
        self.assertEqual(len(diff), 55)
        self.assertGreaterEqual(float(np.mean(diff <= 1.0)), 0.95)
        self.assertLessEqual(float(diff.max()), 3.0)

    def test_the_pooled_median_and_the_share_beyond_twelve_are_inside_t007s_intervals(self):
        ac = np.array([m["reset_ac"] for _, m, _ in self.mine if m["reset_ac"] is not None])
        lo, hi = boot_median_ci(self.theirs)
        self.assertTrue(6.0 < lo < 6.8 and 8.4 < hi < 9.2, (lo, hi))  # T-007: +6.4 .. +8.8
        self.assertTrue(lo <= float(np.median(ac)) <= hi, float(np.median(ac)))
        self.assertAlmostEqual(float(np.median(ac)), 7.4, delta=0.3)
        wl, wh = qa.wilson(int(np.sum(self.theirs > 12)), len(self.theirs))  # T-007: 7% (4 of 55)
        self.assertTrue(wl <= float(np.mean(ac > 12)) <= wh)

    def test_the_level_step_is_t007s(self):
        lv = np.array([m["level_db"] for _, m, _ in self.mine if m["level_db"] is not None])
        self.assertAlmostEqual(float(np.median(lv)), 6.7, delta=0.5)  # T-007: +6.7 dB (CI +5.6 .. +7.7)

    def test_human_reads_are_not_flagged_for_their_resets(self):
        agreed = [m["reset"] for _, m, _ in self.mine if m["reset"] is not None]
        self.assertGreaterEqual(len(agreed), qa.MIN_JOINS)
        self.assertLessEqual(sum(v > 12 for v in agreed) / len(agreed), qa.BANDS["reset_share_top"])


@unittest.skipUnless((FROZEN / "mental_math_out" / "report.json").exists() and (T007 / "measure.json").exists(),
                     "Video F's frozen narration is not here (PROSODY_QA_FROZEN)")
class VideoF(unittest.TestCase):
    """Video F's narration as T-007, T-008, B-C03 and B-C09 measured it"""

    @classmethod
    def setUpClass(cls):
        scored = FROZEN / "mental_math_scored"
        cls.res = qa.scan(FROZEN / "mental_math_out", scored if scored.exists() else None)
        cls.checks = qa.verdicts(cls.res)
        cls.rows = [r for r in json.loads((T007 / "measure.json").read_text())["rows"] if r["src"] == "F" and r["type"] == "S" and r["reset_st"] is not None]

    def test_resets_are_inside_t007s_intervals(self):
        theirs = np.array([r["reset_st"] for r in self.rows])
        ac = np.array([j["reset_ac"] for j in self.res["joins"] if j["reset_ac"] is not None])
        self.assertEqual((len(self.res["joins"]), len(theirs)), (186, 155))
        lo, hi = boot_median_ci(theirs)  # T-007: median +8.7
        self.assertTrue(lo <= float(np.median(ac)) <= hi, (lo, float(np.median(ac)), hi))
        wl, wh = qa.wilson(int(np.sum(theirs > 12)), len(theirs))  # T-007: 29%
        self.assertTrue(wl <= float(np.mean(ac > 12)) <= wh, float(np.mean(ac > 12)))
        lv = np.array([r["level_db"] for r in self.rows if r["level_db"] is not None])
        mine = np.array([j["level_db"] for j in self.res["joins"] if j["level_db"] is not None])
        self.assertTrue(boot_median_ci(lv)[0] <= float(np.median(mine)) <= boot_median_ci(lv)[1] + 0.8)  # T-007: +2.7 dB

    def test_f_is_flagged_beside_the_human_range_and_the_second_tracker_rarely_disagrees(self):
        share = check(self.checks, "reset_share")
        self.assertEqual(share["verdict"], "FLAG")
        self.assertTrue(0.25 <= share["value"] <= 0.36, share["value"])  # T-007 29%, audit-7 31%
        both = [(j["reset_ac"], j["reset_cc"]) for j in self.res["joins"] if j["reset_ac"] is not None and j["reset_cc"] is not None]
        self.assertLess(sum(abs(a - c) > qa.AGREE_ST for a, c in both) / len(both), 0.10)
        median = check(self.checks, "reset_median")
        self.assertTrue(qa.BANDS["reset_median"][0] <= median["value"] <= qa.BANDS["reset_median"][1])  # the median is a human number

    def test_inner_pauses_are_b_c09s(self):
        self.assertEqual(check(self.checks, "inner_pause_over_1s")["verdict"], "PASS")
        self.assertLessEqual(max(p["len"] for p in self.res["inner_pauses"]), 0.55)  # B-C09: max 0.48
        odd = {f"{p['line']}.{p['j']}" for p in self.res["inner_pauses"] if not p["licensed"]}
        self.assertEqual(odd, {"06_2.1"})

    def test_the_silence_is_digital_black(self):
        floor = check(self.checks, "silence_floor")
        self.assertEqual(floor["verdict"], "FLAG")
        self.assertLess(floor["value"], -85.0)  # T-007: -89 dBFS
        self.assertTrue(0.40 <= np.mean([f["zeros"] >= 0.99 for f in self.res["floors"]]) <= 0.55)  # T-007: 48%
        self.assertEqual(check(self.checks, "gap_regularity")["verdict"], "FLAG")  # CV 0.19 against a person's 0.86-1.15

    def test_terminal_events_are_a_subset_of_b_c03s(self):
        found = {f"{s['line']}.{s['j']}" for s in self.res["sentences"] if s["event"] and not s["question"]}
        b_c03 = {"01_6.0", "01_7.3", "02_7.3", "10_3.0", "03_2.1", "05_7.4", "07_2.1", "08_6.1", "12_1.0", "13_5.4"}
        self.assertTrue(found <= b_c03, found - b_c03)
        self.assertEqual(found, {"01_6.0", "02_7.3", "07_2.1", "10_3.0", "12_1.0", "13_5.4"})
        # what B-C03 counted and this does not: a question (08_6.1) and three the second tracker does not confirm
        unsure = {f"{s['line']}.{s['j']}" for s in self.res["sentences"] if (s["rise_ac"] or 0) > 9 and not (s["event"] and not s["question"])}
        self.assertEqual(unsure | found, b_c03)
        self.assertEqual(len(self.res["sentences"]), 274)

    def test_line_gaps_are_t008s(self):
        if not self.res["timeline"]:
            self.skipTest("no timelines in the frozen directory")
        classes = {}
        for g in self.res["line_gaps"]:
            classes.setdefault(g["class"], []).append(g["len"])
        self.assertEqual({k: len(v) for k, v in classes.items()}, {"QUIZ": 19, "PREDICT": 5, "WINDOW": 3, "CONTINUATION": 16, "BEAT": 25, "TOPIC": 6})
        plain = [n for c in ("TOPIC", "BEAT", "CONTINUATION") for n in classes[c]]
        self.assertGreaterEqual(sum(0.595 <= n <= 0.705 for n in plain), 43)  # T-008: 43 of 47 within 0.60-0.70 s
        joins = [j["len"] for j in self.res["chapter_joins"]]
        self.assertEqual(len(joins), 13)
        self.assertEqual(sum(3.515 <= n <= 3.535 for n in joins), 10)  # T-008: ten of 3.52-3.53 s
        self.assertEqual(check(self.checks, "line_gap_regularity")["verdict"], "FLAG")
        self.assertEqual(check(self.checks, "chapter_join_regularity")["verdict"], "FLAG")


if __name__ == "__main__":
    unittest.main()
