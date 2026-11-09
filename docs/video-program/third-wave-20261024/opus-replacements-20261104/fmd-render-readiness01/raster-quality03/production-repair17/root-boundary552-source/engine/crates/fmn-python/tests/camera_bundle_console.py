"""Actual installed console -> native camera recorder; no native doubles.

The companion camera_bundle_cli_native suite additionally consumes the output
through Cargo's standalone binary. These cases exercise host CLI ownership and
compare every exported byte with the programmatic native entry in a fresh scene.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest

import manimlib as m
from fmn_python import export_bundle
from fmn_python.scene_loading import SceneSource


SOURCE = textwrap.dedent('''\
    from pathlib import Path
    import manimlib as m
    print("camera import chatter")
    events = Path(__file__).with_suffix(".events")
    with events.open("a") as log:
        log.write("import\\n")

    class Orbit(m.Scene):
        def setup(self):
            with events.open("a") as log:
                log.write("setup\\n")
        def construct(self):
            with events.open("a") as log:
                log.write("construct\\n")
            print("camera construct chatter")
            self.frame.rotate(0.4, axis=m.RIGHT).scale(0.7)
            self.camera.background_color = "#152536"
            self.camera.light_source.move_to((-3, 4, 6))
            self.add(m.Cube(side_length=1.5, color=m.BLUE))
            marker = m.Square(side_length=0.3, fill_opacity=1, stroke_width=0)
            marker.add_updater(lambda mob, dt: mob.move_to((1.1, self.time, 0.5)))
            self.add(marker)
            self.play(m.Rotate(self.frame, angle=0.8, axis=m.UP),
                      run_time=0.25, rate_func=lambda a: a * a)
            self.camera.background_color = "#361525"
            self.wait(0.125)
        def tear_down(self):
            with events.open("a") as log:
                log.write("tear_down\\n")

    class Flat(m.Scene):
        def construct(self):
            self.add(m.Square(fill_opacity=1))
            self.wait(0.25)

    class Fails(Orbit):
        def construct(self):
            super().construct()
            raise ValueError("authored camera failure")
''')


class CameraBundleConsole(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-camera-console-"))
        self.source = self.root / "scene with spaces.py"
        self.source.write_text(SOURCE)
        self.events = self.source.with_suffix(".events")

    def invoke(self, name, scene="Orbit", *, camera=True, fps=8,
               resolution=(96, 54), extra=(), expected=0):
        path = self.root / (name + ".fmtl")
        argv = [sys.executable, "-m", "fmn_python", "--robot", str(self.source), scene,
                "--format", "fmtl", "--video_dir", str(path), "--fps", str(fps),
                "--resolution", f"{resolution[0]}x{resolution[1]}"]
        if camera:
            argv.append("--bundle-camera")
        result = subprocess.run(argv + list(extra), cwd=self.root, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=90)
        self.assertEqual(result.returncode, expected,
                         (result.stdout, result.stderr, str(self.root)))
        self.assertEqual(len(result.stdout.splitlines()), 1, result.stdout)
        return path, json.loads(result.stdout), result.stderr

    def test_live_camera_console_matches_programmatic_native_bytes(self):
        for fps, resolution in ((8, (96, 54)), (24, (96, 64)), (8, (48, 80))):
            with self.subTest(fps=fps, resolution=resolution):
                name = f"camera-{fps}-{resolution[0]}-{resolution[1]}"
                path, report, stderr = self.invoke(name, fps=fps, resolution=resolution)
                self.assertEqual(report["frame_count"], 3 * fps // 8)
                self.assertEqual(report["bundle"]["segment_count"], 2)
                self.assertTrue(report["bundle"]["camera_track"])
                self.assertEqual(report["bundle"]["fmtl_minor"], 1)
                self.assertFalse(report["bundle"]["certified_source"])
                self.assertIn("camera import chatter", stderr)
                self.assertIn("camera construct chatter", stderr)
                data = path.read_bytes()
                self.assertEqual(report["bundle"]["bytes"], len(data))
                self.assertEqual(report["bundle"]["sha256"], hashlib.sha256(data).hexdigest())
                self.assertEqual(self.events.read_text().splitlines()[-4:],
                                 ["import", "setup", "construct", "tear_down"])
                with SceneSource(str(self.source), m.Scene) as loaded:
                    reference = self.root / (name + "-reference.fmtl")
                    export_bundle(loaded.scenes["Orbit"], reference, camera=True,
                                  fps=fps, resolution=resolution)
                self.assertEqual(data, reference.read_bytes())
                duplicate, _, _ = self.invoke(name + "-again", fps=fps, resolution=resolution)
                self.assertEqual(data, duplicate.read_bytes())
                print(json.dumps({"scenario": "camera-console", "fps": fps,
                                  "resolution": resolution, "frames": report["frame_count"],
                                  "sha256": hashlib.sha256(data).hexdigest()}, sort_keys=True))

    def test_default_planar_mode_and_camera_refusal_are_unchanged(self):
        path, report, _ = self.invoke("flat", scene="Flat", camera=False)
        self.assertTrue(path.exists())
        self.assertNotIn("camera_track", report["bundle"])
        self.assertNotIn("fmtl_minor", report["bundle"])
        failed, report, _ = self.invoke("not-planar", camera=False, expected=4)
        self.assertFalse(failed.exists())
        self.assertFalse(report["artifact_published"])

    def test_preflight_refuses_before_source_execution(self):
        for index, extra in enumerate((("--bundle-camera",), ("--format", "png"))):
            _, report, _ = self.invoke(f"bad-{index}", extra=extra, expected=2)
            self.assertEqual(report["phase"], "options")
            self.assertFalse(self.events.exists())
        _, report, _ = self.invoke("partial", extra=("--skip_animations",), expected=4)
        self.assertEqual(report["phase"], "options")
        self.assertFalse(self.events.exists())

    def test_no_clobber_and_authored_failure_never_publish_partial_output(self):
        path, _, _ = self.invoke("protected")
        data, events = path.read_bytes(), self.events.read_bytes()
        _, report, _ = self.invoke("protected", expected=6)
        self.assertEqual(report["phase"], "start")
        self.assertEqual(path.read_bytes(), data)
        self.assertEqual(self.events.read_bytes(), events)
        failed, report, _ = self.invoke("failed", scene="Fails", expected=5)
        self.assertEqual(report["phase"], "execute")
        self.assertIn("authored camera failure", report["message"])
        self.assertFalse(report["artifact_published"])
        self.assertFalse(failed.exists())
        # A failed generation does not poison a subsequent native owner.
        valid, _, _ = self.invoke("recovered")
        self.assertEqual(valid.read_bytes(), data)


if __name__ == "__main__":
    unittest.main()
