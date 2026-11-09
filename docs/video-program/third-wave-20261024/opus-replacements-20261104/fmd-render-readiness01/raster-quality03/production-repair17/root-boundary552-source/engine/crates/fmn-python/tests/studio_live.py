"""Installed-wheel live Studio: real callbacks, IPC, HTTP and decoded pixels.

Run with the wheel interpreter and -I. Reuse only the test-side HTTP/PNG readers
from studio_preview.py; the engine and host must come from the installed wheel.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import runpy
import tempfile
import time
import unittest
import urllib.error

from fmn_python.studio import Studio

_helpers = runpy.run_path(str(Path(__file__).with_name("studio_preview.py")))
Preview, rgba_png = _helpers["Preview"], _helpers["rgba_png"]


class LiveStudioTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn live studio ")
        self.root = Path(self.temp.name)
        self.source = self.root / "scene.py"
        self.events = self.root / "events.jsonl"
        self.runs = self.root / "runs.txt"
        (self.root / "live_helper.py").write_text("DISTANCE = 1\n")
        self.source.write_text("from manimlib import *\nimport json, os\n" + f'''
class Live(Scene):
    def log(self, kind, **facts):
        with open({str(self.events)!r}, 'a') as stream:
            stream.write(json.dumps(dict(kind=kind, **facts)) + '\\n')
    def construct(self):
        with open({str(self.runs)!r}, 'a') as stream:
            stream.write(str(os.getpid()) + '\\n')
        self.box = Square(fill_opacity=1, fill_color=BLUE, stroke_width=0)
        self.box.add_mouse_press_listner(self.press)
        self.box.add_mouse_drag_listner(self.drag)
        self.fixed = Square(side_length=0.5, fill_opacity=1).shift(3 * LEFT + UP).fix_in_frame()
        self.fixed.add_mouse_press_listner(self.fixed_press)
        self.add(self.box, self.fixed)
        self.wait(0.25)
    def fixed_press(self, mob, event):
        self.log('fixed', point=event['point'].tolist())
        mob.set_color(GREEN)
        return False
    def press(self, mob, event):
        self.log('press', point=event['point'].tolist())
        mob.set_color(RED)
    def drag(self, mob, event):
        self.log('drag', delta=event['d_point'].tolist())
        mob.shift(event['d_point'])
        return False  # Consume the drag instead of also panning the camera.
    def on_key_press(self, symbol, modifiers):
        self.log('key', symbol=symbol, modifiers=modifiers)
        super().on_key_press(symbol, modifiers)
        if symbol == ord('k'):
            import live_helper
            self.play(self.box.animate.shift(live_helper.DISTANCE * RIGHT), run_time=0.125, rate_func=linear)
            self.wait(0.125)
        elif symbol == ord('f'):
            self.box.shift(UP)
            self.wait(0.125)
            raise ValueError('intentional live callback failure')
        elif symbol == ord('h'):
            while True:
                pass
        elif symbol == ord('c'):
            self.frame.shift(RIGHT + UP).scale(2)
    def on_key_release(self, symbol, modifiers):
        self.log('release', symbol=symbol)
        super().on_key_release(symbol, modifiers)
    def on_mouse_motion(self, point, d_point):
        assert point.shape == (3,) and d_point.shape == (3,)
        self.log('motion')
        super().on_mouse_motion(point, d_point)
''')

    def tearDown(self):
        self.temp.cleanup()

    def host(self, **extra):
        return Studio(self.source, "Live", interactive=True, resolution=(96, 54),
                      fps=8, threads=1, max_frames=2, max_bytes=2 * 1024 * 1024, **extra)

    def inspect(self, api):
        with api.request("/api/inspect") as response:
            generation = int(response.headers["X-FMN-Worker-Generation"])
            snapshot = json.load(response)
        return snapshot, generation

    def facts(self):
        return [json.loads(line) for line in self.events.read_text().splitlines()] if self.events.exists() else []

    def event(self, api, kind, **fields):
        snapshot, generation = self.inspect(api)
        view = snapshot["view"]
        form = dict(type=kind, worker_generation=generation, frame=view["frame_index"],
                    revision=view["input_revision"], **fields)
        result = api.json("/api/event", form)
        api.expected_digest = result["sha256"]
        after, _ = self.inspect(api)
        self.assertEqual(after["view"]["input_revision"], view["input_revision"] + 1)
        self.assertEqual(after["view"]["frame_count"], 2)
        return result

    def test_native_guards_callbacks_clock_numpy_vectors_and_camera_mapping(self):
        with self.host() as host:
            api = Preview(host)
            first, generation = self.inspect(api)
            self.assertEqual(first["view"]["frame_count"], 2)
            self.assertFalse(first["view"]["input_events"])
            history = api.frame()
            # No callback runs against a historical capture or without guards.
            for form in [dict(type="key_press", key="k"), dict(type="key_press", key="k",
                         worker_generation=generation, frame=0, revision=0)]:
                with self.assertRaises(urllib.error.HTTPError):
                    api.json("/api/event", form)
            self.assertEqual(self.facts(), [])
            api.json("/api/scrub", {"frame": 1})
            live, _ = self.inspect(api)
            self.assertTrue(live["view"]["input_events"])
            self.assertEqual(live["view"]["input_revision"], 0)
            before = api.frame()
            self.event(api, "key_press", key="k", modifiers=4)
            self.assertEqual(self.facts()[0]["modifiers"], 64, "Command must use the pinned Python token")
            moved = api.frame()
            self.assertNotEqual(rgba_png(before)[2], rgba_png(moved)[2])
            after, _ = self.inspect(api)
            self.assertAlmostEqual(after["scene_time"], live["scene_time"] + 0.25)
            self.event(api, "key_release", key="k")
            count = len(self.facts())
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/event", dict(type="key_press", key="k", worker_generation=generation,
                                           frame=1, revision=0))
            self.assertEqual(len(self.facts()), count)
            self.event(api, "mouse_motion", x=1, y=0, dx=0, dy=0)
            self.event(api, "mouse_press", x=1, y=0, button="left")
            red = rgba_png(api.frame())[2]
            self.assertTrue(any(red[i] > 150 and red[i] > 2*red[i+2] for i in range(0, len(red), 4)))
            self.event(api, "mouse_drag", x=4, y=-1, dx=3, dy=-1, button="left")
            self.event(api, "mouse_release", x=4, y=-1, button="left")
            drag = next(f for f in self.facts() if f["kind"] == "drag")
            self.assertEqual(drag["delta"], [3.0, 1.0, 0.0])
            # Camera shifted by (1,1), scaled by 2. Fixed-plane (1.5,0)
            # therefore picks the moved world-space square at (4,1).
            self.event(api, "key_press", key="c")
            self.event(api, "mouse_press", x=1.5, y=0, button="left")
            press = [f for f in self.facts() if f["kind"] == "press"][-1]
            self.assertEqual(press["point"], [4.0, 1.0, 0.0])
            self.event(api, "mouse_release", x=1.5, y=0, button="left")
            # A fixed-frame control receives its original fixed coordinates,
            # not the camera-transformed world point used for ordinary objects.
            self.event(api, "mouse_press", x=-3, y=-1, button="left")
            fixed = [f for f in self.facts() if f["kind"] == "fixed"][-1]
            self.assertEqual(fixed["point"], [-3.0, 1.0, 0.0])
            self.event(api, "mouse_release", x=-3, y=-1, button="left")
            self.event(api, "mouse_scroll", x=1.5, y=0, offset_x=0, offset_y=1)
            api.json("/api/scrub", {"frame": 0})
            self.assertEqual(api.frame(), history)
            self.assertFalse(self.inspect(api)[0]["view"]["input_events"])
            self.assertEqual(len(self.runs.read_text().splitlines()), 1)

    def test_failed_callback_preserves_last_good_view_and_reload_rejects_old_events(self):
        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            before = api.frame()
            snapshot, generation = self.inspect(api)
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.event(api, "key_press", key="f")
            self.assertIn(b"intentional live callback failure", error.exception.read())
            self.assertEqual(api.frame(), before)
            frozen, _ = self.inspect(api)
            self.assertEqual(frozen["nodes"], snapshot["nodes"])
            self.assertFalse(frozen["view"]["input_events"])
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/event", dict(type="key_press", key="k", worker_generation=generation,
                                           frame=1, revision=0))
            self.assertEqual(len(self.facts()), 1)
            host.reload()
            api.json("/api/scrub", {"frame": 1})
            restored, replacement = self.inspect(api)
            self.assertNotEqual(replacement, generation)
            self.assertEqual(restored["view"]["input_revision"], 0)
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/event", dict(type="key_press", key="k", worker_generation=generation,
                                           frame=1, revision=0))
            self.assertEqual(len(self.facts()), 1)
            self.event(api, "key_press", key="k")
            self.assertEqual(len(self.facts()), 2)
            pids = self.runs.read_text().splitlines()
            self.assertEqual(len(pids), 2)
            self.assertNotEqual(pids[0], pids[1])
            self.assertNotIn(str(os.getpid()), pids)

    def test_live_edit_runs_python_followers_without_advancing_the_clock(self):
        self.source.write_text('''from manimlib import *
class Live(Scene):
    def construct(self):
        self.box = Square(fill_color=BLUE, fill_opacity=1, stroke_width=0)
        self.follower = Square(side_length=0.5, fill_color=RED, fill_opacity=1, stroke_width=0)
        self.follower.add_updater(lambda mob: mob.move_to(self.box.get_center() + 2 * UP))
        self.add(self.box, self.follower)
        self.wait(0.25)
    def on_key_press(self, symbol, modifiers):
        self.box.shift(RIGHT)
''')
        def red_center(data):
            width, _, rgba = rgba_png(data)
            xs = [p % width for p in range(len(rgba) // 4)
                  if rgba[p * 4] > 150 and rgba[p * 4] > 2 * rgba[p * 4 + 2]]
            self.assertGreater(len(xs), 3)
            return sum(xs) / len(xs)

        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            original_time = self.inspect(api)[0]["scene_time"]
            before = red_center(api.frame())
            self.event(api, "key_press", key="j")
            self.assertGreater(red_center(api.frame()), before + 4)
            self.assertEqual(self.inspect(api)[0]["scene_time"], original_time)

    def test_interactive_scene_selection_and_nudge_use_existing_native_objects(self):
        self.source.write_text('''from manimlib import *
class Live(InteractiveScene):
    def construct(self):
        self.box = Square(fill_color=BLUE, fill_opacity=1, stroke_width=0)
        self.add(self.box)
        self.wait(0.25)
''')
        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            self.event(api, "key_press", key="a", modifiers=2)
            selected = api.frame()
            self.event(api, "key_press", key="arrow_right", modifiers=1)
            nudged = api.frame()
            self.assertNotEqual(rgba_png(selected)[2], rgba_png(nudged)[2])
            self.assertTrue(self.inspect(api)[0]["view"]["input_events"])

    def test_held_keys_pan_and_orbit_the_native_camera_without_a_window(self):
        self.source.write_text("from manimlib import *\nimport json\n" + f'''
class Live(Scene):
    def construct(self):
        self.add(Square(fill_color=BLUE, fill_opacity=1, stroke_width=0))
        self.wait(0.25)
    def on_mouse_motion(self, point, d_point):
        super().on_mouse_motion(point, d_point)
        assert self.window is None
        with open({str(self.events)!r}, 'a') as stream:
            stream.write(json.dumps(dict(kind='camera', center=self.frame.get_center().tolist(),
                                         angles=self.frame.get_euler_angles().tolist())) + '\\n')
''')
        with self.host() as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            scene_time = self.inspect(api)[0]["scene_time"]
            self.event(api, "key_press", key="f")
            self.event(api, "mouse_motion", x=1, y=0, dx=1, dy=0)
            panned = self.facts()[-1]
            self.assertEqual(panned["center"], [-1.0, 0.0, 0.0])
            self.event(api, "key_release", key="f")
            self.event(api, "mouse_motion", x=2, y=0, dx=1, dy=0)
            self.assertEqual(self.facts()[-1], panned, "released pan key remained active")
            before = api.frame()
            self.event(api, "key_press", key="d")
            self.event(api, "mouse_motion", x=3, y=-1, dx=1, dy=-1)
            orbited = self.facts()[-1]
            self.assertEqual(orbited["center"], panned["center"])
            self.assertNotEqual(orbited["angles"], panned["angles"])
            self.assertNotEqual(rgba_png(api.frame())[2], rgba_png(before)[2])
            self.event(api, "key_release", key="d")
            self.event(api, "mouse_motion", x=4, y=-1, dx=1, dy=0)
            self.assertEqual(self.facts()[-1], orbited, "released orbit key remained active")
            self.assertEqual(self.inspect(api)[0]["scene_time"], scene_time)

    def test_hung_callback_is_killed_without_reexecution_and_host_can_reload(self):
        with self.host(timeout=3) as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 1})
            started = time.monotonic()
            with self.assertRaises(urllib.error.HTTPError):
                self.event(api, "key_press", key="h")
            self.assertLess(time.monotonic() - started, 12)
            self.assertTrue(host.alive)
            self.assertEqual(len(self.facts()), 1)
            self.assertEqual(len(self.runs.read_text().splitlines()), 1)
            host.reload()
            api.json("/api/scrub", {"frame": 1})
            self.event(api, "key_press", key="k")
            self.assertEqual(len(self.runs.read_text().splitlines()), 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
