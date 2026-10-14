#!/usr/bin/env python3
"""tools/band_dwell.py on synthetic clips with planted lower-third bands (ffmpeg encodes them).

    uv run --quiet --no-project --with numpy python3 -I tools/test_band_dwell.py

A clip is 480x270 at 30 fps, H.264, the background with a wide panel at the foot of the frame whose opacity
follows a schedule: a ramp up over 0.5 s, a hold, a ramp down over 0.5 s. The tests plant a band that is
whole for 1.0 s (flagged, with the line then speaking named from a timeline), one that is whole for 3.0 s
(not flagged), a one-frame cross-fade blip (not a band), a band still on when the chapter ends (flagged and
said so), and a clip with no band at all.
"""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import band_dwell as bd  # noqa: E402

TOOL = Path(__file__).resolve().parent / "band_dwell.py"
W, H, FPS = 480, 270, 30
BG = np.array((14, 17, 26), dtype=np.float64)
PANEL = np.array((70, 84, 120), dtype=np.float64)


def frame(alpha):
    img = np.zeros((H, W, 3), dtype=np.uint8)
    img[:, :] = BG.astype(np.uint8)
    if alpha > 0:
        a = BG * (1 - alpha) + PANEL * alpha
        img[236:264, 12:468] = a.astype(np.uint8)
    return img


def band(fade_in, hold, fade_out):
    """per-frame opacities of a band: a ramp up, a hold, a ramp down (seconds)"""
    up = [(k + 1) / round(fade_in * FPS) for k in range(round(fade_in * FPS))]
    down = [1 - (k + 1) / round(fade_out * FPS) for k in range(round(fade_out * FPS))]
    return up + [1.0] * round(hold * FPS) + down


def make_clip(path, alphas):
    cmd = [
        "ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS),
        "-i", "-", "-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-pix_fmt", "yuv420p", str(path),
    ]
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
    for a in alphas:
        proc.stdin.write(frame(a).tobytes())
    proc.stdin.close()
    assert proc.wait() == 0, "ffmpeg could not encode the test clip"


def quiet(seconds):
    return [0.0] * round(seconds * FPS)


class Bands(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dir = Path(cls.tmp.name)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def clip(self, name, alphas, lines=None):
        path = self.dir / f"{name}.mp4"
        make_clip(path, alphas)
        if lines:
            rows = "".join(f"line\t{i}\t{round(t * FPS)}\t0\t/x/{i}.wav\n" for i, t in lines)
            (self.dir / f"{name}.timeline.tsv").write_text(f"# explainer timeline v1\nfps\t{FPS}\n{rows}", encoding="utf-8")
        return path

    def test_a_band_whole_for_one_second_is_flagged_with_its_line(self):
        path = self.clip("short", quiet(1.0) + band(0.5, 1.0, 0.5) + quiet(2.0), lines=[("03_1", 0.0), ("03_2", 1.0)])
        found = bd.bands(bd.strip_frames(path, FPS), FPS)
        self.assertEqual(len(found), 1, found)
        start, on, whole, closing = found[0]
        self.assertAlmostEqual(start, 1.0, delta=0.2)
        self.assertAlmostEqual(whole, 1.0, delta=0.4)
        self.assertFalse(closing)
        lines, n = bd.report(path)
        self.assertEqual(n, 1, lines)
        self.assertIn("(line 03_2)", lines[0])

    def test_a_band_whole_for_three_seconds_is_not_flagged(self):
        path = self.clip("long", quiet(1.0) + band(0.5, 3.0, 0.5) + quiet(1.0))
        lines, n = bd.report(path)
        self.assertEqual(n, 0, lines)
        self.assertIn("PASS long.mp4: 1 bands", lines[0])

    def test_a_one_frame_cross_fade_blip_is_not_a_band(self):
        path = self.clip("blip", quiet(1.0) + [1.0] + quiet(2.0))
        self.assertEqual(bd.bands(bd.strip_frames(path, FPS), FPS), [])

    def test_a_band_still_on_when_the_chapter_ends_is_flagged_and_says_so(self):
        path = self.clip("closing", quiet(1.0) + band(0.5, 0.3, 0.1)[:-3])
        lines, n = bd.report(path)
        self.assertEqual(n, 1, lines)
        self.assertIn("still on when the chapter ends", lines[0])

    def test_a_clip_with_no_band_passes(self):
        path = self.clip("none", quiet(3.0))
        lines, n = bd.report(path)
        self.assertEqual((n, lines[0][:4]), (0, "PASS"))

    def test_the_command_prints_flags_and_exits_zero_unless_gated(self):
        path = self.clip("cmd", quiet(1.0) + band(0.5, 1.0, 0.5) + quiet(1.0))
        run = lambda *a: subprocess.run([sys.executable, "-I", str(TOOL), str(path), *a], capture_output=True, text=True)
        r = run()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("FLAG cmd.mp4: band from", r.stdout)
        self.assertIn("ADVISORY band dwell: 1 bands under 2.6 s in 1 chapter file(s)", r.stdout)
        self.assertEqual(run("--gate").returncode, 1)
        self.assertEqual(run("--floor-s", "0.5", "--gate").returncode, 0)


class Runs(unittest.TestCase):
    def test_runs_of_true(self):
        self.assertEqual(bd.runs([False, True, True, False, True]), [(1, 3), (4, 5)])
        self.assertEqual(bd.runs([]), [])


if __name__ == "__main__":
    unittest.main()
