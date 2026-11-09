"""Fresh installed-wheel live clock acceptance: real worker, HTTP and pixels.

No host-side animation substitute: authored updaters run only in the retained
scene process, and all final frames are decoded independently by the test.
"""
from pathlib import Path
import runpy
import time
import unittest
import urllib.error

_helpers = runpy.run_path(str(Path(__file__).with_name("studio_live.py")))
LiveStudioTests = _helpers["LiveStudioTests"]
Preview, rgba_png = _helpers["Preview"], _helpers["rgba_png"]


class LiveClockTests(LiveStudioTests):
    def setUp(self):
        super().setUp()
        source = self.source.read_text().replace("        self.wait(0.25)\n", """        self.wait(0.25)
        self.ticks = 0
        self.box.add_updater(self.tick)
""", 1)
        source += """
    def tick(self, mob, dt):
        self.log('tick', dt=dt, time=self.get_time(), plays=self.num_plays, pid=os.getpid())
        if dt > 0:
            self.ticks += 1
            mob.shift(dt * RIGHT)
"""
        self.source.write_text(source)

    def test_startup_exception_reports_the_worker_cause_without_terminal_escapes(self):
        self.source.write_text(r"raise ValueError('visible startup cause\x1b[2J')" + "\n")
        with self.assertRaisesRegex(RuntimeError, "visible startup cause") as failure:
            self.host()
        self.assertLess(len(str(failure.exception)), 8192)
        self.assertNotIn("\x1b", str(failure.exception))

    def test_nominal_ticks_execute_once_without_wait_indices_or_growing_history(self):
        with self.host() as host:
            api = Preview(host)
            history = api.frame()
            with self.assertRaisesRegex(RuntimeError, "final frame"):
                host.advance()
            api.json("/api/scrub", {"frame": 1})
            original, generation = self.inspect(api)
            before = api.frame()
            self.events.write_text("")
            time.sleep(0.15)
            self.assertEqual(self.facts(), [], "paused worker must not execute idle callbacks")
            receipt = host.advance(3)
            api.expected_digest = receipt["sha256"]
            after, same_generation = self.inspect(api)
            self.assertEqual(generation, same_generation)
            self.assertEqual(after["view"]["frame_count"], 2)
            self.assertEqual(after["view"]["input_revision"], 1)
            self.assertAlmostEqual(after["scene_time"], original["scene_time"] + 3 / 8)
            ticks = self.facts()
            self.assertEqual(len(ticks), 3, "no extra zero-dt refresh/updater pass per live command")
            self.assertEqual([t["dt"] for t in ticks], [1 / 8] * 3)
            self.assertEqual([t["time"] for t in ticks], [original["scene_time"] + n / 8 for n in (1, 2, 3)])
            self.assertEqual([t["plays"] for t in ticks], [1] * 3, "idle is not an authored wait")
            self.assertEqual({t["pid"] for t in ticks}, {int(self.runs.read_text().strip())})
            self.assertNotEqual(rgba_png(before)[2], rgba_png(api.frame())[2])
            # Live events use the next shared revision, then another clock step.
            self.event(api, "key_release", key="k")
            receipt = host.advance()
            api.expected_digest = receipt["sha256"]
            after, _ = self.inspect(api)
            self.assertEqual(after["view"]["input_revision"], 3)
            self.assertAlmostEqual(after["scene_time"], original["scene_time"] + 0.5)
            api.json("/api/scrub", {"frame": 0})
            self.assertEqual(api.frame(), history)

    def test_live_advance_refuses_stale_unbounded_and_incomplete_commands_before_effects(self):
        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            snapshot, generation = self.inspect(api)
            self.events.write_text("")
            valid = dict(worker_generation=generation, frame=1, revision=0, frames=1)
            for changes in ({"worker_generation": generation + 1}, {"frame": 0}, {"revision": 99},
                            {"frames": 0}, {"frames": -1}, {"frames": 9}, {"frames": 241},
                            {"frames": 1.5}, {"elapsed": 1}):
                with self.subTest(changes=changes), self.assertRaises(urllib.error.HTTPError):
                    api.json("/api/advance", dict(valid, **changes))
            for field in valid:
                with self.subTest(missing=field), self.assertRaises(urllib.error.HTTPError):
                    api.json("/api/advance", {k: v for k, v in valid.items() if k != field})
            self.assertEqual(self.facts(), [])
            after, _ = self.inspect(api)
            self.assertEqual(after["scene_time"], snapshot["scene_time"])
            host.advance(8)  # Inclusive one-second limit is a single command.
            self.assertEqual(len(self.facts()), 8)
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/advance", valid)
            self.assertEqual(len(self.facts()), 8, "stale retries do not execute effects")

    def test_partial_updater_failure_keeps_last_good_frame_and_freezes_until_reload(self):
        fault = "            if self.ticks == 2: raise ValueError('intentional idle failure')\n"
        self.source.write_text(self.source.read_text() + fault)
        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            before, generation = self.inspect(api)
            image = api.frame()
            self.events.write_text("")
            with self.assertRaisesRegex(RuntimeError, "intentional idle failure"):
                host.advance(3)
            after, _ = self.inspect(api)
            self.assertFalse(after["view"]["input_events"])
            self.assertFalse(after["view"].get("live_advance", False))
            self.assertEqual(after["scene_time"], before["scene_time"], "inspector must describe the retained good frame")
            self.assertEqual(api.frame(), image)
            self.assertEqual(len(self.facts()), 2, "authored effects happened and are not rolled back")
            with self.assertRaises(RuntimeError):
                host.advance()
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/event", dict(type="key_press", key="k", worker_generation=generation,
                                           frame=1, revision=0))
            self.assertEqual(len(self.facts()), 2, "frozen code must not be retried")
            self.source.write_text(self.source.read_text().replace(fault, ""))
            host.reload()
            api.json("/api/scrub", {"frame": 1})
            self.events.write_text("")
            host.advance()
            self.assertEqual(len(self.facts()), 1)
            after, fresh = self.inspect(api)
            self.assertGreater(fresh, generation)
            self.assertEqual(after["view"]["input_revision"], 1)


# Keep the original live-input suite in its own invocation. Inherited methods
# are useful shared helpers but must not silently double its accepted coverage.
if __name__ == "__main__":
    suite = unittest.TestSuite(LiveClockTests(name) for name in (
        "test_startup_exception_reports_the_worker_cause_without_terminal_escapes",
        "test_nominal_ticks_execute_once_without_wait_indices_or_growing_history",
        "test_live_advance_refuses_stale_unbounded_and_incomplete_commands_before_effects",
        "test_partial_updater_failure_keeps_last_good_frame_and_freezes_until_reload",
    ))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    raise SystemExit(not result.wasSuccessful())
