#!/usr/bin/env python3
"""The joint choice of a line's reads and the seeded reads (bead mx-4kc2).

    uv run --no-project --with numpy --with praat-parselmouth python3 -I tools/test_narration_joint.py SKILL_SCRIPTS

No ftts and no ASR run here. The pure parts (the Viterbi, the costs, the trackers) are tested on
numbers and on synthetic buzz, a harmonic tone whose pitch follows a contour; the command line
(`plan --seeds`, `synth`, `choose --joint`) is run for real on a two-sentence script whose takes
are such buzz and whose transcripts are written by hand, and `synth` calls a stand-in for `ftts`
that records its arguments. What these cannot show is whether the joint choice sounds better:
that is the owner's A/B.
"""
import hashlib
import importlib.util
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import unittest

import numpy as np

if len(sys.argv) < 2 or sys.argv[1].startswith("-"):
    sys.exit("pass the narration skill's scripts directory (containing narrate.py)")
SKILL_SCRIPTS = str(Path(sys.argv.pop(1)).resolve())
sys.dont_write_bytecode = True
TOOL = Path(__file__).resolve().parent / "narration.py"


def load_tool():
    """tools/narration.py as a module; it finds narrate.py by argv[1]."""
    spec = importlib.util.spec_from_file_location("narration", TOOL)
    module = importlib.util.module_from_spec(spec)
    saved = sys.argv
    sys.argv = [saved[0], SKILL_SCRIPTS]
    try:
        spec.loader.exec_module(module)
    finally:
        sys.argv = saved
    return module


tool = load_tool()
nr, RATE = tool.nr, tool.nr.RATE


def buzz(f0, peak=0.2):
    """A voiced sound whose pitch follows f0 (Hz per sample): six harmonics, so both trackers hear it."""
    phase = 2 * np.pi * np.cumsum(f0) / RATE
    x = sum(np.sin(k * phase) / k for k in range(1, 7))
    return peak * x / np.max(np.abs(x))


def sentence_audio(onset_hz, final_hz, syllables=5, jump_st=0.0, hiss=False, abrupt=False):
    """Five 180 ms syllables 50 ms apart, the pitch gliding from onset to final (even in st) and, for the last
    120 ms, `jump_st` higher; 0.3 s of digital silence either side: what `hear` needs to find a sentence.
    `hiss` ends it in 120 ms of high-passed noise, an s; `abrupt` stops the take there with no silence after
    it, as ftts does when it cuts a sentence short."""
    span = syllables * 0.18 + (syllables - 1) * 0.05
    t = np.arange(int(span * RATE)) / RATE
    envelope = np.zeros_like(t)
    for i in range(syllables):
        a, b = i * 0.23, i * 0.23 + 0.18
        envelope += np.clip(np.minimum(t - a, b - t) / 0.02, 0, 1)
    f0 = onset_hz * (final_hz / onset_hz) ** (t / span) * 2 ** (jump_st / 12 * (t > span - 0.12))
    sound = buzz(f0) * envelope
    if hiss:
        noise = np.diff(np.random.default_rng(5).standard_normal(int(0.12 * RATE) + 1))
        sound = np.concatenate([sound, noise * (0.04 / noise.std()) * np.hanning(len(noise)) ** 0.25])
    return np.concatenate([np.zeros(int(0.3 * RATE)), sound, np.zeros(0 if abrupt else int(0.3 * RATE))])


def ends(onset, final, median=None):
    """The features of one tracker, as pitch_ends() returns them."""
    return {"onset_hz": onset, "final_hz": final, "median_hz": median or (onset * final) ** 0.5, "final_move_st": 0.0}


def feats(onset, final, median=None, tail=0.0):
    """What read_features() returns: Praat's view (the costs use it) and the second tracker's."""
    return {"praat": ends(onset, final, median), "second": ends(onset, final, median),
            "tail_praat": tail, "tail_second": tail}


def read(score=0.95, **kw):
    """The fields of an `analyze` result that the node cost reads."""
    m = {"score": score, "cut_short": False, "sibilant_missing": False, "asr_ratio": 1.0, "misheard_names": 0,
         "hesitations": 0, "stalls": 0, "pauses": []}
    m.update(kw)
    return m


class Viterbi(unittest.TestCase):
    def test_it_finds_the_cheapest_choice_among_all_of_them(self):
        rng = random.Random(7)
        for _ in range(40):
            sizes = [rng.randint(1, 4) for _ in range(rng.randint(1, 5))]
            layers = [[(rng.random(), (i, k)) for k in range(n)] for i, n in enumerate(sizes)]
            table = {}

            def join(a, b):
                return table.setdefault((a, b), rng.random() * 2)

            def total(path):
                return sum(layers[i][k][0] for i, k in enumerate(path)) + \
                    sum(join(layers[i - 1][path[i - 1]][1], layers[i][path[i]][1]) for i in range(1, len(path)))

            best = min(itertools.product(*[range(n) for n in sizes]), key=total)
            self.assertEqual(tool.best_path(layers, join), list(best))

    def test_a_planted_bad_join_is_avoided(self):
        # the sentences' best reads are 0 and 0; 0 -> 0 resets the pitch by 13.6 st (and steps the register
        # 6.1 st), 1 -> 0 by 6.9 st. The second read of the first sentence costs 0.02 more, the way out.
        layers = [[(0.10, feats(120, 100)), (0.12, feats(140, 150))],
                  [(0.10, feats(220, 110)), (0.15, feats(150, 110))]]
        self.assertEqual(tool.best_path(layers, tool.join_cost), [1, 0])
        self.assertAlmostEqual(tool.join_cost(layers[0][0][1], layers[1][0][1]), 1.0 + 0.2 * 1.6 + 0.1 * 2.1, delta=0.03)
        self.assertEqual(tool.join_cost(layers[0][1][1], layers[1][0][1]), 0.0)

    def test_without_a_join_cost_it_is_the_cheapest_node(self):
        layers = [[(0.10, feats(120, 100)), (0.12, feats(140, 150))],
                  [(0.10, feats(220, 110)), (0.15, feats(150, 110))]]
        self.assertEqual(tool.best_path(layers, lambda a, b: 0.0), [0, 0])

    def test_a_tie_goes_to_the_earlier_read(self):
        layers = [[(0.3, None), (0.3, None), (0.3, None)]] * 2
        self.assertEqual(tool.best_path(layers, lambda a, b: 0.0), [0, 0])

    def test_with_every_extra_cost_off_it_is_narrate_pys_pick(self):
        # node = w_score * (1 - score) and a tie-break toward the best: the cheapest node is the best score
        off = {k: 0.0 for k in tool.JOINT_COST} | {"score": 2.0, "change": 0.02}
        rng = random.Random(3)
        for _ in range(50):
            scores = [round(rng.choice([0.9, 0.92, 0.95, 0.95, 0.97]), 4) for _ in range(rng.randint(1, 5))]
            top = max(range(len(scores)), key=scores.__getitem__)  # the first of the equal best, like a stable sort
            layer = [(tool.node_cost(read(s, pauses=[{"len": 0.6, "punct": ""}]), feats(100, 100, tail=30), False,
                                     True, w=off) + off["change"] * (i != top), None) for i, s in enumerate(scores)]
            self.assertEqual(tool.best_path([layer], lambda a, b: 0.0), [top])
            self.assertEqual(top, sorted(range(len(scores)), key=lambda i: -scores[i])[0])  # narrate.py's stable sort


class Costs(unittest.TestCase):
    def test_a_clean_read_costs_what_its_score_does(self):
        self.assertAlmostEqual(tool.node_cost(read(0.95), feats(100, 100), False), 2.0 * 0.05)

    def test_the_gates_cost_more_than_anything_a_better_read_can_save(self):
        base = tool.node_cost(read(), None, False)
        for gate in ({"cut_short": True}, {"sibilant_missing": True}, {"asr_ratio": 0.8}):
            self.assertAlmostEqual(tool.node_cost(read(**gate), None, False) - base, tool.JOINT_COST["gate"])

    def test_a_hole_is_an_unlicensed_pause_over_the_limit(self):
        base = tool.node_cost(read(), None, False)
        hole = tool.node_cost(read(pauses=[{"len": 0.5, "punct": ""}]), None, False)
        self.assertAlmostEqual(hole - base, tool.JOINT_COST["hole"])
        for quiet in ({"len": 0.5, "punct": ","}, {"len": 0.40, "punct": ""}):
            self.assertEqual(tool.node_cost(read(pauses=[quiet]), None, False), base)

    def test_a_statement_that_rises_at_its_end_costs_and_a_question_does_not(self):
        rising = feats(100, 100)
        rising["praat"]["final_move_st"] = 6.0
        rise = tool.node_cost(read(), rising, False) - tool.node_cost(read(), feats(100, 100), False)
        self.assertAlmostEqual(rise, tool.JOINT_COST["rise"] * 4.0)
        self.assertEqual(tool.node_cost(read(), rising, True), tool.node_cost(read(), feats(100, 100), True))

    def test_a_creaky_ending_costs_nothing_unless_it_is_priced(self):
        creaky = feats(100, 100)
        creaky["praat"]["creaky"] = True
        self.assertEqual(tool.JOINT_COST["creak"], 0.0, "whether creak is a defect is the owner's ear: off by default")
        self.assertEqual(tool.node_cost(read(), creaky, False), tool.node_cost(read(), feats(100, 100), False))
        priced = {**tool.JOINT_COST, "creak": 1.0}
        extra = tool.node_cost(read(), creaky, False, w=priced) - tool.node_cost(read(), feats(100, 100), False, w=priced)
        self.assertAlmostEqual(extra, 1.0)

    def test_a_statement_that_ends_more_than_eight_semitones_up_fails_a_gate_and_a_question_does_not(self):
        rising = feats(100, 100)
        rising["praat"]["final_move_st"] = 9.0
        self.assertEqual(tool.failed_gates(read(), rising, False), {"rise"})
        self.assertEqual(tool.failed_gates(read(), rising, True), set())
        rising["praat"]["final_move_st"] = 8.0
        self.assertEqual(tool.failed_gates(read(), rising, False), set())
        more = tool.node_cost(read(), {**rising, "praat": {**rising["praat"], "final_move_st": 9.0}}, False)
        self.assertAlmostEqual(more - tool.node_cost(read(), rising, False),
                               tool.JOINT_COST["gate"] + tool.JOINT_COST["rise"])

    def test_the_gates_a_read_fails(self):
        self.assertEqual(tool.failed_gates(read(), feats(100, 100), False), set())
        every = read(cut_short=True, sibilant_missing=True, asr_ratio=0.8)
        self.assertEqual(tool.failed_gates(every, None, False), {"cut_short", "sibilant_missing", "asr"})

    def test_a_read_is_never_moved_into_a_failure(self):
        # narrate.py's pick (index 1) is cut short; a read that drops the final s/z instead is no way out
        gates = [{"sibilant_missing"}, {"cut_short"}, set(), {"cut_short", "asr"}, {"cut_short"}]
        self.assertEqual(tool.admissible(gates, 1), [1, 2, 4])
        # a pick that fails nothing keeps only the reads that fail nothing
        self.assertEqual(tool.admissible(gates, 2), [2])
        # where the pick fails something, a read failing the same thing or less may take its place
        self.assertEqual(tool.admissible([{"cut_short"}, set(), {"cut_short"}], 0), [0, 1, 2])

    def test_a_joint_choice_with_more_reads_without_their_final_s_than_narrate_pys_fails(self):
        tool.check_sibilants(2, 2)
        tool.check_sibilants(2, 1)
        with self.assertRaises(SystemExit) as stop:
            tool.check_sibilants(0, 1)
        self.assertIn("final s/z", str(stop.exception))

    def test_an_exclusion_file_belongs_to_the_script_it_failed_on(self):
        script = Path(tempfile.mkdtemp(prefix="narration_joint_script_")) / "script.tsv"
        script.write_text("01_1\tOne. Two.\n")
        first = tool.exclusion_file(script, "work")
        self.assertEqual(first, tool.exclusion_file(script, "work"))
        script.write_text("01_1\tOne. Three.\n")
        second = tool.exclusion_file(script, "work")
        self.assertNotEqual(first, second)
        self.assertTrue(first.name.startswith("joint_exclude.") and first.name.endswith(".txt"))

    def test_an_ending_is_creaky_when_its_median_or_half_its_frames_are_under_80_hz(self):
        def creaky(last):  # 40 frames at 110 Hz, then the last 150 ms (15 or 16 frames) as given
            return tool.pitch_ends(np.concatenate([np.full(40, 110.0), last]))["creaky"]
        self.assertFalse(creaky(np.full(16, 100.0)))
        self.assertTrue(creaky(np.full(16, 70.0)))
        self.assertFalse(creaky(np.array([70.0] * 6 + [110.0] * 10)))
        self.assertTrue(creaky(np.array([110.0] * 4 + [70.0] * 12)))

    def test_the_costs_read_praat_and_not_the_second_tracker(self):
        second = feats(100, 100)
        second["second"]["final_move_st"] = 8.0
        self.assertEqual(tool.node_cost(read(), second, False), tool.node_cost(read(), feats(100, 100), False))
        no_praat = {**feats(100, 100), "praat": None}
        self.assertEqual(tool.node_cost(read(), no_praat, False), tool.node_cost(read(), feats(100, 100), False))

    def test_a_terminal_event_needs_both_trackers(self):
        self.assertTrue(tool.terminal_event({"tail_praat": 12.0, "tail_second": 6.0}))
        self.assertFalse(tool.terminal_event({"tail_praat": 12.0, "tail_second": 3.0}))
        self.assertFalse(tool.terminal_event({"tail_praat": 8.0, "tail_second": 9.0}))
        self.assertFalse(tool.terminal_event({"tail_praat": float("nan"), "tail_second": 9.0}))
        event = tool.node_cost(read(), {**feats(100, 100), "tail_praat": 15.0, "tail_second": 9.0}, False)
        self.assertAlmostEqual(event - tool.node_cost(read(), feats(100, 100), False), tool.JOINT_COST["terminal"])

    def test_a_heard_word_before_the_first_is_an_extra_onset_and_a_leading_and_is_not(self):
        self.assertTrue(tool.lead_extra("Take n times five.", "that take n times five."))
        self.assertFalse(tool.lead_extra("Take n times five.", "and take n times five."))
        self.assertFalse(tool.lead_extra("Take n times five.", "take n times five."))
        self.assertFalse(tool.lead_extra("Take n times five.", "take n times"))

    def test_a_reset_is_what_praat_measures_and_the_second_tracker_is_held_out(self):
        a = feats(100, 100)
        up = feats(100 * 2 ** (16 / 12), 100, median=100)  # the next sentence starts 16 st over where this one ended
        self.assertAlmostEqual(tool.join_resets(a, up)[0], 16.0)
        self.assertAlmostEqual(tool.join_cost(a, up), 1.0 + 0.2 * 4.0)
        flat = {**up, "second": ends(100, 100)}  # Praat sees the jump, the second tracker does not
        self.assertAlmostEqual(tool.join_cost(a, flat), 1.0 + 0.2 * 4.0)
        self.assertAlmostEqual(tool.join_resets(a, flat, "second")[0], 0.0)
        hidden = {**up, "praat": ends(100, 100)}  # the second tracker sees it and Praat does not: no cost
        self.assertEqual(tool.join_cost(a, hidden), 0.0)
        self.assertAlmostEqual(tool.join_resets(a, hidden, "second")[0], 16.0)
        at_twelve = feats(100 * 2 ** (12 / 12), 100, median=100)
        self.assertEqual(tool.join_cost(a, at_twelve), 0.0, "the price starts past +12 st")

    def test_the_register_step_costs_past_four_semitones(self):
        a = feats(100, 100, median=100)
        for step, cost in ((3.0, 0.0), (7.0, 0.1 * 3.0), (-7.0, 0.1 * 3.0)):
            b = feats(100, 100, median=100 * 2 ** (step / 12))
            self.assertAlmostEqual(tool.join_cost(a, b), cost)

    def test_no_tracker_no_cost(self):
        self.assertEqual(tool.join_cost({"praat": None, "second": None}, feats(100, 100)), 0.0)
        self.assertEqual(tool.join_cost(None, feats(100, 100)), 0.0)


class Trackers(unittest.TestCase):
    def test_both_trackers_follow_a_glide(self):
        x = sentence_audio(160.0, 100.0)
        for name, f0 in (("praat", tool.praat_f0(x, RATE)), ("viterbi", tool.f0_viterbi(x, RATE))):
            e = tool.pitch_ends(f0)
            self.assertIsNotNone(e, name)
            self.assertAlmostEqual(tool.st_between(e["onset_hz"], 160.0 * (100 / 160.0) ** 0.05), 0.0, delta=1.2, msg=name)
            self.assertAlmostEqual(tool.st_between(e["final_hz"], 100.0 / (100 / 160.0) ** 0.07), 0.0, delta=1.2, msg=name)

    def test_a_tone_with_a_strong_second_harmonic_stays_on_its_fundamental(self):
        t = np.arange(int(1.0 * RATE)) / RATE
        x = 0.2 * (0.5 * np.sin(2 * np.pi * 110 * t) + np.sin(2 * np.pi * 220 * t) + 0.5 * np.sin(2 * np.pi * 330 * t))
        f0 = tool.f0_viterbi(x, RATE)
        voiced = f0[~np.isnan(f0)]
        self.assertGreater(len(voiced), 80)
        self.assertLess(np.max(np.abs(12 * np.log2(voiced / np.median(voiced)))), 1.0)

    def test_praat_matches_pitch_ends_on_a_glide_and_reads_a_low_ending(self):
        # a clean 70 Hz ending (the creaky 62-75 Hz endings of Video F are not clean, and are not synthesized here)
        e = tool.pitch_ends(tool.praat_f0(sentence_audio(150.0, 70.0), RATE))
        self.assertAlmostEqual(tool.st_between(e["final_hz"], 70.0 / (70 / 150.0) ** 0.07), 0.0, delta=1.5)

    def test_a_sentence_that_ends_in_a_jump_is_a_terminal_event_and_a_fall_is_not(self):
        fall, jump = sentence_audio(140.0, 100.0), sentence_audio(140.0, 100.0, jump_st=15.0)
        self.assertFalse(tool.terminal_event(tool.read_features(fall, RATE)))
        features = tool.read_features(jump, RATE)
        self.assertTrue(tool.terminal_event(features), features)
        self.assertGreater(features["tail_praat"], tool.TERMINAL[0])
        self.assertGreater(features["tail_second"], tool.TERMINAL[1])

    def test_a_clip_with_no_voice_has_no_features(self):
        f = tool.read_features(np.zeros(RATE), RATE)
        self.assertIsNone(f["praat"])
        self.assertIsNone(f["second"])


class Seeds(unittest.TestCase):
    def test_a_seeded_tag_is_content_addressed_and_keeps_the_shape(self):
        base = "01_1__s0__0123abcd"
        tags = {tool.seed_tag(base, s) for s in (1, 2, 3)}
        self.assertEqual(len(tags), 3)
        self.assertNotIn(base, tags)
        self.assertEqual(tool.seed_tag(base, 2), tool.seed_tag(base, 2))
        self.assertTrue(all(t.startswith("01_1__s0__") and len(t) == len(base) for t in tags))
        self.assertNotEqual(tool.seed_tag(base, 1), tool.seed_tag("01_1__s0__0123abce", 1))


class CommandLine(unittest.TestCase):
    """The real tools/narration.py on a two-sentence script. The takes are synthetic buzz."""
    SCRIPT = "01_1\tHere is the first one. And here is the next.\n"

    def setUp(self):
        self.dir = Path(tempfile.mkdtemp(prefix="narration_joint_"))  # kept: nothing is deleted
        (self.dir / "script.tsv").write_text(self.SCRIPT)

    def run_tool(self, *args, env=None, check=True):
        proc = subprocess.run([sys.executable, "-I", str(TOOL), SKILL_SCRIPTS, *args], cwd=self.dir, text=True,
                              capture_output=True, env={**os.environ, **(env or {})})
        if check:
            self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        return proc

    def test_seeds_are_planned_beside_the_takes_and_not_in_them(self):
        self.run_tool("plan", "script.tsv", "plain")
        plain_plan = (self.dir / "plain" / "plan.json").read_bytes()
        plain_takes = (self.dir / "plain" / "takes.tsv").read_bytes()
        out = self.run_tool("plan", "script.tsv", "seeded", "--seeds", "2").stdout
        self.assertEqual((self.dir / "seeded" / "takes.tsv").read_bytes(), plain_takes)
        plan = json.loads((self.dir / "seeded" / "plan.json").read_text())
        rows = [row.split("\t") for row in (self.dir / "seeded" / "seeds.tsv").read_text().splitlines()]
        for sentence, base in zip(plan["01_1"], json.loads(plain_plan)["01_1"]):
            self.assertEqual(sentence["reads"][:len(base["reads"])], base["reads"], "the unseeded reads come first, untouched")
            seeded = sentence["reads"][len(base["reads"]):]
            self.assertEqual(len(seeded), 2 * len(base["reads"]))
            self.assertEqual(sorted({r["seed"] for r in seeded}), [1, 2])
            self.assertEqual(len({r["tag"] for r in sentence["reads"]}), len(sentence["reads"]))
            self.assertTrue(all(r["text"] == base["text"] for r in seeded))
        self.assertEqual(len(rows), sum(len(s["reads"]) for s in plan["01_1"]) - sum(
            len(s["reads"]) for s in json.loads(plain_plan)["01_1"]))
        self.assertTrue(all(len(r) == 3 and r[1] in ("1", "2") for r in rows))
        self.assertIn(f"{len(rows)} more reads", out)
        # a plan without --seeds over the same work dir is narrate.py's again, and the seed list is empty
        self.run_tool("plan", "script.tsv", "seeded")
        self.assertEqual((self.dir / "seeded" / "plan.json").read_bytes(), plain_plan)
        self.assertEqual((self.dir / "seeded" / "seeds.tsv").read_text(), "")

    def test_synth_makes_each_seeded_read_with_its_seed_and_keeps_what_it_has(self):
        self.run_tool("plan", "script.tsv", "work", "--seeds", "2")
        log = self.dir / "ftts.log"
        fake = self.dir / "ftts"
        fake.write_text(f"""#!{sys.executable}
import json, os, sys, wave
args = sys.argv[1:]
open(os.environ["FAKE_FTTS_LOG"], "a").write(json.dumps(args) + "\\n")
with wave.open(args[-1], "wb") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(24000); w.writeframes(b"\\x01\\x00" * 4000)
""")
        fake.chmod(0o755)
        env = {"FTTS": str(fake), "FAKE_FTTS_LOG": str(log)}
        rows = [r.split("\t") for r in (self.dir / "work" / "seeds.tsv").read_text().splitlines()]
        self.run_tool("synth", "work", "--voice", "michael", "--workers", "3", env=env)
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        self.assertEqual(len(calls), len(rows))
        for tag, seed, text in rows:
            self.assertTrue((self.dir / "work" / "takes" / f"{tag}.wav").stat().st_size > 1000, tag)
            call = next(c for c in calls if c[-2] == text and c[c.index("--seed") + 1] == seed)
            self.assertEqual(call[:4], ["say", "--no-resident", "--voice", "michael"])
            self.assertEqual(call[call.index("--") + 1], text)
        self.assertEqual(sorted((c[c.index("--seed") + 1], c[-2]) for c in calls), sorted((s, t) for _, s, t in rows))
        self.run_tool("synth", "work", "--voice", "michael", "--workers", "3", env=env)
        self.assertEqual(len(log.read_text().splitlines()), len(rows), "a second synth finds every take and calls nothing")

    def test_synth_with_nothing_planned_says_so(self):
        self.run_tool("plan", "script.tsv", "work")
        out = self.run_tool("synth", "work", "--voice", "michael").stdout
        self.assertIn("no seeded reads planned", out)

    # (onset Hz, final Hz, ASR confidence) of the reads of each sentence, in plan order: plain, in quotation
    # marks, in doubled marks (narrate.py wraps and tools/narration.py wraps again), then the same three with
    # seed 1. The best-scored read of each sentence has confidence .97. Those two join 95 Hz -> 235 Hz, +15.7 st;
    # the other reads of the second sentence start at 160-170 Hz, +9.0 to +10.0 st over the first's best.
    FIRST = [(125, 95, 0.97), (150, 105, 0.90), (140, 110, 0.90), (135, 100, 0.90), (138, 104, 0.90), (142, 108, 0.90)]
    SECOND = [(235, 110, 0.97), (160, 105, 0.90), (170, 110, 0.90), (165, 100, 0.90), (168, 108, 0.90), (162, 103, 0.90)]
    SECOND_FINE = [(150, 110, 0.97)] + SECOND[1:]  # the best read starts 7.9 st over: no bad join anywhere

    def write_takes(self, work, sentences):
        """sentences[j][i] = (onset Hz, final Hz, ASR confidence[, sentence_audio options]) for the i-th planned
        read of sentence j."""
        plan = json.loads((work / "plan.json").read_text())
        (work / "takes").mkdir(exist_ok=True)
        asr = {}
        for j, sentence in enumerate(plan["01_1"]):
            self.assertEqual(len(sentence["reads"]), len(sentences[j]))
            words = sentence["text"].rstrip(".").split()
            for r, (onset, final, conf, *options) in zip(sentence["reads"], sentences[j]):
                nr.write_wav(work / "takes" / f"{r['tag']}.wav", sentence_audio(onset, final, **(options or [{}])[0]))
                asr[r["tag"]] = {"text": sentence["text"], "engine": "test",
                                 "words": [{"w": w, "s": 0.3 + 0.23 * i, "e": 0.48 + 0.23 * i, "p": conf} for i, w in enumerate(words)]}
        (work / "asr.json").write_text(json.dumps(asr))
        return plan["01_1"]

    def choose(self, work, out, *flags):
        proc = self.run_tool("choose", "script.tsv", str(work), str(out), "--limit-headroom", "18", *flags)
        report = json.loads((out / "report.json").read_text())["lines"]["01_1"]
        return proc.stdout, report, hashlib.sha1((out / "01_1.wav").read_bytes()).hexdigest()

    def test_the_joint_choice_moves_off_a_pick_whose_join_resets_16_semitones_and_says_so(self):
        self.run_tool("plan", "script.tsv", "work", "--seeds", "1")
        sentences = self.write_takes(self.dir / "work", [self.FIRST, self.SECOND])
        tags = [[r["tag"] for r in s["reads"]] for s in sentences]
        _, plain, plain_sha = self.choose(self.dir / "work", self.dir / "out_plain")
        self.assertEqual([c["read"] for c in plain], [tags[0][0], tags[1][0]], "narrate.py picks the best score twice")
        self.assertTrue(all("joint" not in c for c in plain))
        said, joint, joint_sha = self.choose(self.dir / "work", self.dir / "out_joint", "--joint")
        self.assertEqual([c["read"] for c in joint], [tags[0][0], tags[1][1]])
        self.assertEqual(joint[1]["joint"]["from"], tags[1][0])
        self.assertNotIn("joint", joint[0])
        self.assertNotIn("picked", joint[1])
        self.assertLess(joint[1]["score"], plain[1]["score"], "the joint pick keeps its own, lower, measured score")
        self.assertIn("1 of 2 sentences are read differently", said)
        self.assertIn("past +12 st: 1 -> 0 of 1 by Praat, 1 -> 0 of 1 by the second tracker (held out)", said)
        self.assertNotEqual(joint_sha, plain_sha)
        # the first sentence's line audio is the same samples: only the second sentence's take changed
        a = nr.load(self.dir / "out_plain" / "01_1.wav")[0]
        b = nr.load(self.dir / "out_joint" / "01_1.wav")[0]
        self.assertEqual(joint[0]["out_samples"], plain[0]["out_samples"])
        self.assertTrue(np.array_equal(a[:joint[0]["out_samples"]], b[:joint[0]["out_samples"]]))

    def test_a_director_pick_is_not_moved(self):
        self.run_tool("plan", "script.tsv", "work")
        plan = self.write_takes(self.dir / "work", [self.FIRST[:3], self.SECOND[:3]])
        pick = plan[1]["reads"][0]
        (self.dir / "picks.tsv").write_text(f"01_1\t1\t{pick['tts']}\tthe director heard this one\n")
        _, report, _ = self.choose(self.dir / "work", self.dir / "out", "--joint")
        self.assertEqual(report[1]["read"], pick["tag"], "the director's read of the second sentence, whatever its join")
        self.assertIn("picked", report[1])
        self.assertNotIn("joint", report[1])

    def test_where_no_read_has_a_defect_the_joint_choice_is_narrate_pys_byte_for_byte(self):
        # no join is bad, no read has a gate, a hole, a rise or a terminal event, and the creak is not priced;
        # a read with one of those would be re-ranked even where no join is bad (a creak: the next test)
        self.run_tool("plan", "script.tsv", "work", "--seeds", "1")
        self.write_takes(self.dir / "work", [self.FIRST, self.SECOND_FINE])
        _, plain, plain_sha = self.choose(self.dir / "work", self.dir / "out_plain")
        said, joint, joint_sha = self.choose(self.dir / "work", self.dir / "out_joint", "--joint")
        self.assertIn("0 of 2 sentences are read differently", said)
        self.assertEqual(joint_sha, plain_sha)
        self.assertEqual([c["read"] for c in joint], [c["read"] for c in plain])
        self.assertEqual((self.dir / "out_joint" / "report.json").read_text(),
                         (self.dir / "out_plain" / "report.json").read_text())

    def test_a_line_that_fails_a_line_level_gate_gives_up_the_reads_the_joint_choice_moved(self):
        # P-2's lost "But": every take passes its own transcript and the assembled line does not. Each round
        # prices the moved read of the failing line, the next picks another, and the last is narrate.py's.
        self.run_tool("plan", "script.tsv", "work", "--seeds", "1")
        plan = self.write_takes(self.dir / "work", [self.FIRST, self.SECOND])
        tags = [[r["tag"] for r in s["reads"]] for s in plan]
        spoken = "Here is the first one. And here is the next."
        picks, code = [], None
        for round_ in range(14):
            out = self.dir / f"out_{round_}"
            _, report, _ = self.choose(self.dir / "work", out, "--joint")
            (out / "lines_asr.json").write_text(json.dumps({"01_1": {"text": spoken}}))
            (out / "qa.json").write_text(json.dumps({"failing": [{"line": "01_1", "ratio": 0.8}]}))
            picks.append([c["read"] for c in report])
            code = self.run_tool("exclude", "script.tsv", "work", str(out), check=False)
            if code.returncode:
                break
        self.assertEqual(picks[0][1], tags[1][1], "round 0 moves the second sentence off its 16 st join")
        self.assertEqual(picks[1][1], tags[1][2], "and when that line fails, to the next read")
        self.assertEqual(picks[-1], [tags[0][0], tags[1][0]], "with every moved read priced it is narrate.py's pick")
        self.assertLess(len(picks), 14)
        self.assertEqual(code.returncode, 1, code.stdout)
        self.assertIn("0 reads the joint choice moved there are now excluded", code.stdout)
        excluded = set(tool.exclusion_file(self.dir / "script.tsv", self.dir / "work").read_text().split())
        self.assertTrue({tags[1][1], tags[1][2], tags[1][3]} <= excluded)
        self.assertFalse(excluded & {tags[0][0], tags[1][0]}, "narrate.py's own picks are never excluded")

    def test_a_passing_build_excludes_nothing(self):
        self.run_tool("plan", "script.tsv", "work")
        self.write_takes(self.dir / "work", [self.FIRST[:3], self.SECOND_FINE[:3]])
        self.choose(self.dir / "work", self.dir / "out", "--joint")
        (self.dir / "out" / "lines_asr.json").write_text(json.dumps({"01_1": {"text": "Here is the first one. And here is the next."}}))
        code = self.run_tool("exclude", "script.tsv", "work", str(self.dir / "out"), check=False)
        self.assertEqual(code.returncode, 1)
        self.assertFalse(tool.exclusion_file(self.dir / "script.tsv", self.dir / "work").exists())

    def test_a_line_that_loses_its_first_word_gives_up_the_moved_reads_though_qa_is_clean(self):
        # P-2's lost "But" exactly: qa_lines passes (the line is 0.99 of its words) and only `edges` sees that
        # the first word of a sentence is not in the assembled line's transcript
        self.run_tool("plan", "script.tsv", "work")
        plan = self.write_takes(self.dir / "work", [self.FIRST[:3], self.SECOND[:3]])
        tags = [[r["tag"] for r in s["reads"]] for s in plan]
        out = self.dir / "out"
        _, report, _ = self.choose(self.dir / "work", out, "--joint")
        self.assertEqual(report[1]["read"], tags[1][1], "the joint choice moved the second sentence")
        (out / "qa.json").write_text(json.dumps({"failing": []}))
        (out / "lines_asr.json").write_text(json.dumps({"01_1": {"text": "is the first one. And here is the next."}}))
        code = self.run_tool("exclude", "script.tsv", "work", str(out), check=False)
        self.assertEqual(code.returncode, 0, code.stdout + code.stderr)
        self.assertIn("1 lines fail a line-level gate", code.stdout)
        excluded = set(tool.exclusion_file(self.dir / "script.tsv", self.dir / "work").read_text().split())
        self.assertEqual(excluded, {tags[1][1]})

    def test_exclusions_belong_to_the_script_they_failed_on(self):
        self.run_tool("plan", "script.tsv", "work")
        plan = self.write_takes(self.dir / "work", [self.FIRST[:3], self.SECOND[:3]])
        tags = [[r["tag"] for r in s["reads"]] for s in plan]
        script, work = self.dir / "script.tsv", self.dir / "work"
        out = self.dir / "out_0"
        self.choose(work, out, "--joint")
        (out / "lines_asr.json").write_text(json.dumps({"01_1": {"text": "Here is the first one. And here is the next."}}))
        (out / "qa.json").write_text(json.dumps({"failing": [{"line": "01_1", "ratio": 0.8}]}))
        self.run_tool("exclude", "script.tsv", "work", str(out))
        learned = tool.exclusion_file(script, work)
        self.assertEqual(learned.read_text().split(), [tags[1][1]])
        self.assertEqual(self.choose(work, self.dir / "out_1", "--joint")[1][1]["read"], tags[1][2], "priced")
        script.write_text(script.read_text() + "# edited, same lines\n")
        again = self.choose(work, self.dir / "out_2", "--joint")[1][1]["read"]
        self.assertEqual(again, tags[1][1], "a script edit starts over: the old exclusions are not read")
        self.assertNotEqual(tool.exclusion_file(script, work), learned)
        self.assertEqual(learned.read_text().split(), [tags[1][1]], "and the old file is left as it was")

    def test_the_joint_choice_never_trades_a_cut_short_read_for_one_without_its_final_s(self):
        # Video E's regression: where narrate.py's best read is cut short, a read that has lost the s of "ones"
        # looked as good to the cost model and the join (the best read steps 15.7 st up into the next sentence)
        # made it better. B and B' are clean, join well and are missing the s: they must not be candidates.
        (self.dir / "script.tsv").write_text("01_1\tHere are the first ones. Now say the next ones.\n")
        self.run_tool("plan", "script.tsv", "work")
        hiss, cut = {"hiss": True}, {"hiss": True, "abrupt": True}
        plan = self.write_takes(self.dir / "work", [
            [(125, 95, 0.99, cut), (175, 130, 0.70), (175, 130, 0.65)],
            [(235, 110, 0.97, hiss), (235, 110, 0.96, hiss), (235, 110, 0.95, hiss)]])
        tags = [[r["tag"] for r in s["reads"]] for s in plan]
        _, plain, plain_sha = self.choose(self.dir / "work", self.dir / "out_plain")
        self.assertEqual([c["read"] for c in plain], [tags[0][0], tags[1][0]])
        self.assertTrue(plain[0]["cut_short"] and not plain[0]["sibilant_missing"], "the best read is cut short, with its s")
        self.assertFalse(plain[1]["cut_short"] or plain[1]["sibilant_missing"])
        said, joint, joint_sha = self.choose(self.dir / "work", self.dir / "out_joint", "--joint")
        self.assertEqual([c["read"] for c in joint], [tags[0][0], tags[1][0]], "narrate.py's reads, not the ones without the s")
        self.assertEqual(joint_sha, plain_sha)
        self.assertIn("reads without their final s/z: 0 -> 0", said)
        self.assertIn("2 reads refused for failing a gate narrate.py's pick passes", said)
        self.assertIn("past +12 st: 1 -> 1 of 1 by Praat", said, "the join that was worth the trade stays bad: a read is not moved into a failure")

    def test_a_creaky_ending_is_the_ears_call_unless_it_is_priced(self):
        self.run_tool("plan", "script.tsv", "work")
        plan = self.write_takes(self.dir / "work", [
            [(125, 70, 0.97), (125, 100, 0.90), (125, 100, 0.88)], [(120, 100, 0.97), (120, 100, 0.96), (120, 100, 0.95)]])
        tags = [[r["tag"] for r in s["reads"]] for s in plan]
        _, plain, _ = self.choose(self.dir / "work", self.dir / "out_plain")
        self.assertEqual(plain[0]["read"], tags[0][0], "narrate.py's pick ends at 70 Hz")
        said, off, off_sha = self.choose(self.dir / "work", self.dir / "out_off", "--joint")
        self.assertEqual([c["read"] for c in off], [c["read"] for c in plain])
        self.assertIn("0 of 2 sentences are read differently", said)
        said, on, _ = self.choose(self.dir / "work", self.dir / "out_on", "--joint", "--creak", "1")
        self.assertEqual(on[0]["read"], tags[0][1], "priced, the creaky ending is moved off")
        self.assertEqual(on[0]["joint"]["from"], tags[0][0])
        self.assertIn("1 of 2 sentences are read differently", said)

    def test_the_flags_belong_to_their_commands(self):
        self.run_tool("plan", "script.tsv", "work")
        self.assertNotEqual(self.run_tool("plan", "script.tsv", "work", "--joint", check=False).returncode, 0)
        self.assertNotEqual(self.run_tool("choose", "script.tsv", "work", "out", "--seeds", "2", check=False).returncode, 0)


if __name__ == "__main__":
    unittest.main()
