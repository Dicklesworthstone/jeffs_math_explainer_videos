#!/usr/bin/env python3
"""Unit tests for scripts/corpus_speed_ratio.py (fm-5wq.35).

The producer's statistics and ordering are deterministic and tested on
constructed inputs. The planted negative injects a fixed sleep on the portal
side of one scene, through in-process runners, and checks that scene's ratio
moves by the expected amount.
"""
import pathlib
import sys
import tempfile
import threading
import time
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import corpus_speed_ratio as csr  # noqa: E402


def run(module, scene, engine, rep, seconds, exit_code=0):
    return {"module": module, "scene": scene, "engine": engine, "rep": rep,
            "seconds": seconds, "exit": exit_code}


class Collect:
    def __init__(self):
        self.records, self.done, self.lock = [], {}, threading.Lock()

    def __call__(self, record):
        self.records.append(record)


class SpeedRatioTests(unittest.TestCase):
    def test_bootstrap_is_seeded_and_deterministic(self):
        values = [0.5, 0.8, 1.2, 2.0, 3.5, 0.9, 1.1]
        median = lambda sample: csr.percentile(sample, 50)  # noqa: E731
        first = csr.bootstrap_interval(values, median, resamples=500, seed=7)
        self.assertEqual(first, csr.bootstrap_interval(values, median, resamples=500, seed=7))
        low, high = first
        self.assertLessEqual(low, median(values))
        self.assertGreaterEqual(high, median(values))
        # The seed matters. Over 7 values the resample statistics are few and
        # the interval's order statistics can coincide across seeds; over 50
        # distinct values they do not.
        spread = [1.0 + 0.037 * index * index % 5.3 for index in range(50)]
        mean = lambda sample: sum(sample) / len(sample)  # noqa: E731
        self.assertNotEqual(csr.bootstrap_interval(spread, mean, resamples=500, seed=7),
                            csr.bootstrap_interval(spread, mean, resamples=500, seed=8))

    def test_percentile_is_linear_interpolation(self):
        self.assertEqual(csr.percentile([1, 2, 3, 4], 50), 2.5)
        self.assertEqual(csr.percentile([10], 99), 10)
        self.assertAlmostEqual(csr.percentile([0, 10], 90), 9.0)
        with self.assertRaises(ValueError):
            csr.percentile([], 50)

    def test_ratio_is_the_median_of_paired_repetitions(self):
        runs = [run("m.py", "A", "portal", 0, 2.0), run("m.py", "A", "reference", 0, 1.0),
                run("m.py", "A", "portal", 1, 3.0), run("m.py", "A", "reference", 1, 1.0),
                run("m.py", "A", "portal", 2, 1.0), run("m.py", "A", "reference", 2, 2.0)]
        included, excluded = csr.scene_ratios(runs)
        row = included[("m.py", "A")]
        self.assertEqual(row["ratio"], 2.0)  # median of 2.0, 3.0, 0.5
        self.assertEqual((row["min"], row["max"], row["pairs"]), (0.5, 3.0, 3))
        self.assertEqual(excluded, {})

    def test_a_missing_or_failed_side_is_excluded_and_counted(self):
        runs = [run("m.py", "Timeout", "portal", 0, 300.0, None),
                run("m.py", "Refail", "portal", 0, 1.0), run("m.py", "Refail", "reference", 0, 1.0, 1),
                run("m.py", "Half", "portal", 0, 1.0),
                run("m.py", "Ok", "portal", 0, 1.0), run("m.py", "Ok", "reference", 0, 4.0)]
        included, excluded = csr.scene_ratios(runs)
        self.assertEqual(list(included), [("m.py", "Ok")])
        self.assertEqual(included[("m.py", "Ok")]["ratio"], 0.25)
        self.assertEqual(excluded[("m.py", "Timeout")], ["portal:timeout"])
        self.assertEqual(excluded[("m.py", "Refail")], ["reference:exit1"])
        self.assertEqual(excluded[("m.py", "Half")], ["incomplete"])
        self.assertEqual(csr.corpus_summary(included)["scenes"], 1)
        self.assertEqual(included[("m.py", "Ok")]["failed"], [])

    def test_portal_timeouts_count_at_their_lower_bounds_only_where_exact(self):
        runs = [run("m.py", s, e, 0, sec)
                for s, factor in (("A", 0.5), ("B", 1.0), ("C", 2.0))
                for e, sec in (("portal", factor), ("reference", 1.0))]
        runs += [run("m.py", "T1", "portal", 0, 300.0, None), run("m.py", "T1", "reference", 0, 30.0),
                 run("m.py", "T2", "portal", 0, 300.0, None), run("m.py", "T2", "reference", 0, 15.0),
                 run("m.py", "Both", "portal", 0, 300.0, None), run("m.py", "Both", "reference", 0, 2.0, 1),
                 run("m.py", "Refail", "portal", 0, 1.0), run("m.py", "Refail", "reference", 0, 1.0, 1)]
        included, excluded = csr.scene_ratios(runs)
        bounds = csr.timeout_bounds(runs, excluded)
        self.assertEqual(bounds, {("m.py", "T1"): 10.0, ("m.py", "T2"): 20.0})
        ratios = [row["ratio"] for row in included.values()]
        # Dropping the timeouts would report 1.0; counted at their bounds the
        # median is the third of five values, which no bound can move.
        self.assertEqual(csr.percentile(ratios, 50), 1.0)
        self.assertEqual(csr.censored_percentile(ratios, list(bounds.values()), 50), 2.0)
        # p90 reads the two largest values, both only bounded: unknown.
        self.assertIsNone(csr.censored_percentile(ratios, list(bounds.values()), 90))
        # A bound inside the interpolated pair makes the median unknown too.
        self.assertIsNone(csr.censored_percentile([0.5, 1.0, 2.0], [0.7], 50))
        with tempfile.TemporaryDirectory() as tmp:
            out = pathlib.Path(tmp)
            (out / "runs.ndjson").write_text("\n".join(
                __import__("json").dumps(r) for r in runs) + "\n")
            csr.write_dashboard(out, out / "dash.md")
            text = (out / "dash.md").read_text()
        self.assertIn("| ratio median counting 2 portal timeouts at their lower bounds | 2.00 |", text)
        self.assertIn("| ratio p90 counting 2 portal timeouts at their lower bounds |"
                      " not determinable (a timeout bound reaches it) |", text)

    def test_frame_grid_mode_runs_the_reference_through_the_replacement_program(self):
        args = type("Args", (), {"reference_python": "py", "videos": pathlib.Path("/v"),
                                 "reference_frame_grid": False})()
        plain = csr.reference_argv(args, "m.py", "A", pathlib.Path("/t"))
        self.assertEqual(plain[:3], ["py", "-m", "manimlib"])
        args.reference_frame_grid = True
        grid = csr.reference_argv(args, "m.py", "A", pathlib.Path("/t"))
        self.assertEqual(grid[:3], ["py", "-c", csr.REFERENCE_FRAME_GRID])
        # Everything after the entry point is the same Reference command line.
        self.assertEqual(grid[3:], plain[3:])
        self.assertNotEqual(csr.FRAME_GRID_MODE, csr.SKIP_MODE)
        self.assertIn("mode", csr.IDENTITY)
        runs = [run("m.py", "A", "portal", 0, 2.0), run("m.py", "A", "reference", 0, 1.0)]
        for mode, note in ((csr.FRAME_GRID_MODE, "same updater work"),
                           (csr.SKIP_MODE, "one step per skipped segment")):
            with tempfile.TemporaryDirectory() as tmp:
                out = pathlib.Path(tmp)
                (out / "header.json").write_text(__import__("json").dumps({"mode": mode}))
                (out / "runs.ndjson").write_text("\n".join(
                    __import__("json").dumps(r) for r in runs) + "\n")
                csr.write_dashboard(out, out / "dash.md")
                text = (out / "dash.md").read_text()
            self.assertIn(note, text)
            self.assertIn(f"| mode | {mode} |", text)

    def test_frame_grid_program_steps_skipped_segments_like_playback(self):
        # Run the program exactly as the producer does (python -c), over
        # stand-in Reference modules with its skip behavior (scene.py:467,
        # :567). The stand-in entry point reports what the program installed.
        import json
        import subprocess
        stand_ins = {
            "manimlib/__init__.py": "",
            "manimlib/scene/__init__.py": "",
            "manimlib/scene/scene.py": (
                "class Scene:\n"
                "    skip_animations, fps = True, 4\n"
                "    def get_time_progression(self, run_time, n_iterations=None, desc='',\n"
                "                             override_skip_animations=False):\n"
                "        if self.skip_animations and not override_skip_animations:\n"
                "            return [run_time]\n"
                "        return [(i + 1) / self.fps for i in range(int(run_time * self.fps))]\n"
                "    def finish_animations(self, animations):\n"
                "        for animation in animations:\n"
                "            animation.finish()\n"
                "        self.update_mobjects(self.get_time_progression(1.0)[-1])\n"
                "    def update_mobjects(self, dt):\n"
                "        self.calls.append(['update_mobjects', dt])\n"),
            "manimlib/__main__.py": (
                "import json, sys\n"
                "from manimlib.scene.scene import Scene\n"
                "class Animation:\n"
                "    def __init__(self, calls): self.calls = calls\n"
                "    def finish(self): self.calls.append('finish')\n"
                "    def clean_up_from_scene(self, scene): self.calls.append('clean_up')\n"
                "def main():\n"
                "    scene = Scene()\n"
                "    scene.calls = []\n"
                "    grid = scene.get_time_progression(1.0)\n"
                "    scene.finish_animations([Animation(scene.calls)])\n"
                "    print(json.dumps({'argv': sys.argv, 'grid': grid, 'finish': scene.calls}))\n"),
        }
        with tempfile.TemporaryDirectory() as tmp:
            for relative, text in stand_ins.items():
                path = pathlib.Path(tmp) / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(text)
            proc = subprocess.run([sys.executable, "-c", csr.REFERENCE_FRAME_GRID, "/v/m.py", "A", "-s"],
                                  env={"PYTHONPATH": tmp, "PATH": "/usr/bin:/bin"},
                                  capture_output=True, text=True, timeout=60)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        try:
            report = json.loads(proc.stdout)
        except json.JSONDecodeError:
            self.fail(f"the stand-in entry point printed no report: {proc.stdout!r} {proc.stderr!r}")
        self.assertEqual(report["argv"], ["manimlib", "/v/m.py", "A", "-s"])
        self.assertEqual(report["grid"], [0.25, 0.5, 0.75, 1.0])
        self.assertEqual(report["finish"], ["finish", "clean_up", ["update_mobjects", 0]])

    def test_a_later_failure_keeps_the_pairs_and_stays_visible(self):
        runs = [run("m.py", "A", "portal", 0, 2.0), run("m.py", "A", "reference", 0, 1.0),
                run("m.py", "A", "portal", 1, 300.0, None)]
        included, excluded = csr.scene_ratios(runs)
        self.assertEqual(excluded, {})
        self.assertEqual((included[("m.py", "A")]["ratio"], included[("m.py", "A")]["pairs"]),
                         (2.0, 1))
        self.assertEqual(included[("m.py", "A")]["failed"], ["portal:timeout"])
        with tempfile.TemporaryDirectory() as tmp:
            out = pathlib.Path(tmp)
            (out / "runs.ndjson").write_text("\n".join(
                __import__("json").dumps(r) for r in runs) + "\n")
            csr.write_dashboard(out, out / "dash.md")
            self.assertIn("| 1 | 2.0 | 1.0 | portal:timeout |", (out / "dash.md").read_text())

    def test_runs_interleave_engines_in_the_log(self):
        self.assertEqual(csr.interleaved_plan([("m.py", "A")], 2),
                         [(("m.py", "A"), 0, "portal"), (("m.py", "A"), 0, "reference"),
                          (("m.py", "A"), 1, "portal"), (("m.py", "A"), 1, "reference")])
        collect = Collect()
        runners = {engine: (lambda scene, rep: (0, "")) for engine in csr.ENGINES}
        csr.measure_scene(("m.py", "A"), 3, runners, collect, collect.done, collect.lock)
        self.assertEqual([(r["engine"], r["rep"]) for r in collect.records],
                         [("portal", 0), ("reference", 0), ("portal", 1), ("reference", 1),
                          ("portal", 2), ("reference", 2)])
        self.assertTrue(all(r["label"] == csr.LABEL for r in collect.records))

    def test_resume_skips_finished_runs_and_failure_stops_the_scene(self):
        collect = Collect()
        collect.done[("m.py", "A", "portal", 0)] = 0
        calls = []

        def runner(engine, code):
            def call(scene, rep):
                calls.append((engine, rep))
                return code, "Traceback ...\n\x1b[31mTypeError: bad x_min\x1b[0m\n\n"
            return call

        runners = {"portal": runner("portal", 0), "reference": runner("reference", 2)}
        finished = csr.measure_scene(("m.py", "A"), 3, runners, collect, collect.done, collect.lock)
        self.assertFalse(finished)
        # portal rep 0 was already done; the reference failure stops later reps.
        self.assertEqual(calls, [("reference", 0)])
        # The failure keeps its cause; a success records none.
        self.assertEqual(collect.records[0]["error"], "TypeError: bad x_min")

    def test_a_truncated_last_line_is_dropped_but_other_damage_raises(self):
        json = __import__("json")
        whole = json.dumps(run("m.py", "A", "portal", 0, 1.0))
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "runs.ndjson"
            path.write_text(whole + "\n" + whole[:17], encoding="utf-8")
            self.assertEqual(len(csr.load_runs(path)), 1)
            path.write_text(whole[:17] + "\n" + whole + "\n", encoding="utf-8")
            with self.assertRaises(ValueError):
                csr.load_runs(path)

    def test_resume_refuses_a_different_wheel_or_settings(self):
        header = {"portal_id": "a", "reference_id": "r", "reps": 3, "timeout_s": 300,
                  "mode": "m", "schema": csr.SCHEMA, "version": csr.VERSION, "cpu": "x"}
        self.assertEqual(csr.resume_conflicts(header, dict(header, cpu="y")), [])
        self.assertEqual(csr.resume_conflicts(header, dict(header, portal_id="b", reps=5)),
                         ["portal_id", "reps"])

    def test_the_selection_is_described_as_it_was_made(self):
        json = __import__("json")
        with tempfile.TemporaryDirectory() as tmp:
            candidates = pathlib.Path(tmp) / "ok.ndjson"
            candidates.write_text("".join(
                json.dumps({"module": "m.py", "scene": f"S{i}", "outcome": "ok"}) + "\n"
                for i in range(10)) + json.dumps({"module": "m.py", "scene": "Bad",
                                                   "outcome": "error"}) + "\n")
            args = type("Args", (), {"scene": [], "candidates": candidates,
                                     "sample": 4, "sample_seed": 35})()
            sampled, described = csr.select_scenes(args)
            self.assertEqual(len(sampled), 4)
            self.assertEqual(described, "seeded sample of 4 (seed 35) of 10 candidates in ok.ndjson")
            self.assertEqual(sampled, csr.select_scenes(args)[0])
            args.sample = 0
            everything, described = csr.select_scenes(args)
            self.assertEqual((len(everything), described), (10, "all 10 candidates in ok.ndjson"))
            args.scene = ["m.py:S1"]
            self.assertEqual(csr.select_scenes(args), ([("m.py", "S1")], "explicit --scene list"))

    def test_a_failure_still_measures_the_other_side_of_its_repetition(self):
        # A portal timeout next to the Reference's time is the worst case's
        # lower bound; without the pair it is only "excluded".
        collect, calls = Collect(), []

        def runner(engine, code):
            def call(scene, rep):
                calls.append((engine, rep))
                return code, "boom" if code else ""
            return call

        runners = {"portal": runner("portal", None), "reference": runner("reference", 0)}
        self.assertFalse(csr.measure_scene(("m.py", "A"), 3, runners, collect, collect.done,
                                           collect.lock))
        self.assertEqual(calls, [("portal", 0), ("reference", 0)])

    def test_a_logged_failure_stops_the_scene_on_resume(self):
        collect = Collect()
        collect.done[("m.py", "A", "portal", 0)] = 0
        collect.done[("m.py", "A", "reference", 0)] = 1
        calls = []
        runners = {engine: (lambda scene, rep, engine=engine: calls.append((engine, rep)) or (0, ""))
                   for engine in csr.ENGINES}
        self.assertFalse(csr.measure_scene(("m.py", "A"), 3, runners, collect, collect.done,
                                           collect.lock))
        self.assertEqual(calls, [], "no repetition after the logged reference failure")

    def test_planted_portal_sleep_moves_the_ratio_by_the_expected_amount(self):
        base, planted = 0.05, 0.10

        def sleeper(extra):
            def call(scene, rep):
                time.sleep(base + (extra if scene[1] == "Planted" else 0.0))
                return 0, ""
            return call

        collect = Collect()
        runners = {"portal": sleeper(planted), "reference": sleeper(0.0)}
        for scene in (("m.py", "Control"), ("m.py", "Planted")):
            csr.measure_scene(scene, 3, runners, collect, collect.done, collect.lock)
        included, _ = csr.scene_ratios(collect.records)
        control = included[("m.py", "Control")]["ratio"]
        planted_ratio = included[("m.py", "Planted")]["ratio"]
        expected = (base + planted) / base  # 3.0
        self.assertAlmostEqual(control, 1.0, delta=0.35)
        self.assertAlmostEqual(planted_ratio, expected, delta=0.6)
        self.assertGreater(planted_ratio - control, 1.4)

    def test_dashboard_states_label_median_and_exclusions(self):
        runs = [run("m.py", s, e, r, sec)
                for s, factor in (("A", 0.5), ("B", 2.0), ("C", 1.0))
                for r in range(2) for e, sec in (("portal", factor), ("reference", 1.0))]
        runs.append(dict(run("m.py", "D", "portal", 0, 9.0, None), error="stuck | in a loop"))
        with tempfile.TemporaryDirectory() as tmp:
            out = pathlib.Path(tmp)
            (out / "runs.ndjson").write_text("\n".join(
                __import__("json").dumps(dict(r, schema=csr.SCHEMA)) for r in runs) + "\n")
            summary, excluded = csr.write_dashboard(out, out / "dash.md")
            text = (out / "dash.md").read_text()
        self.assertEqual((summary["scenes"], excluded, summary["median"]), (3, 1, 1.0))
        self.assertIn(csr.LABEL, text)
        self.assertIn("| m.py:B | 2.00 |", text)
        self.assertIn("| m.py:D | portal:timeout | timeout 9.0 s | not run | stuck \\| in a loop |", text)


if __name__ == "__main__":
    unittest.main(verbosity=1)
