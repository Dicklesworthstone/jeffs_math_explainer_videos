#!/usr/bin/env python3
"""tools/blank_stretches.py on synthetic clips with planted blanks (no render needed; ffmpeg encodes them).

    uv run --quiet --no-project --with numpy python3 -I tools/test_blank_stretches.py

A clip is a list of (seconds, what) segments at 480x270, 30 fps, H.264: `bg` is the background alone,
`black` a digital-black frame, `picture` a lit box with a line under it, `label` one small lit box (a
caption's worth: it is a picture, not a blank). The tests plant the defects of the three chapters that
had them (0.6 s of background mid-chapter, a 3.6 s opening, a bare closing) and check that the opening
lead, a short pause, a small label and a picture-only clip are not flagged.
"""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import blank_stretches as bs  # noqa: E402

TOOL = Path(__file__).resolve().parent / "blank_stretches.py"
W, H, FPS = 480, 270, 30
BG = (14, 17, 26)


def frame(what):
    img = np.zeros((H, W, 3), dtype=np.uint8)
    if what != "black":
        img[:, :] = BG
    if what == "picture":
        img[60:200, 90:390] = (200, 190, 120)
        img[210:213, 40:440] = (120, 160, 220)
    elif what == "label":
        img[130:138, 200:224] = (210, 210, 210)
    return img


def make_clip(path, segments):
    """Encode `segments` [(seconds, what)] to `path` (H.264, yuv420p, 30 fps)."""
    cmd = [
        "ffmpeg", "-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS),
        "-i", "-", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p", str(path),
    ]
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
    for seconds, what in segments:
        one = frame(what).tobytes()
        for _ in range(round(seconds * FPS)):
            proc.stdin.write(one)
    proc.stdin.close()
    assert proc.wait() == 0, "ffmpeg could not encode the test clip"


class Clips(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dir = Path(cls.tmp.name)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def flags(self, name, segments, **kw):
        path = self.dir / f"{name}.mp4"
        make_clip(path, segments)
        found = bs.stretches(bs.flat_frames(path, FPS), FPS)
        return bs.flagged(found, **kw), found

    def test_a_blank_in_the_middle_is_flagged_with_its_place(self):
        flags, _ = self.flags("inside", [(0.6, "bg"), (2.0, "picture"), (0.8, "bg"), (2.0, "picture")])
        self.assertEqual(len(flags), 1, flags)
        start, end, kind = flags[0]
        self.assertEqual(kind, "inside")
        self.assertAlmostEqual(start, 2.6, delta=0.1)
        self.assertAlmostEqual(end - start, 0.8, delta=0.1)

    def test_the_opening_lead_is_excused_and_a_long_opening_is_not(self):
        flags, found = self.flags("lead", [(0.7, "bg"), (2.0, "picture")])
        self.assertEqual(flags, [], "a 0.7 s lead is how every chapter opens")
        self.assertEqual(found[0][2], "opening")
        flags, _ = self.flags("opening", [(3.6, "bg"), (2.0, "picture")])
        self.assertEqual([k for *_, k in flags], ["opening"])
        self.assertAlmostEqual(flags[0][1] - flags[0][0], 3.6, delta=0.1)

    def test_a_closing_blank_is_flagged(self):
        flags, _ = self.flags("closing", [(0.6, "bg"), (2.0, "picture"), (0.9, "bg")])
        self.assertEqual([k for *_, k in flags], ["closing"])

    def test_a_digital_black_stretch_is_a_blank_too(self):
        flags, _ = self.flags("black", [(0.6, "bg"), (2.0, "picture"), (0.8, "black"), (2.0, "picture")])
        self.assertEqual([k for *_, k in flags], ["inside"])

    def test_a_short_pause_and_a_small_label_are_pictures_not_blanks(self):
        flags, found = self.flags(
            "quiet", [(0.6, "bg"), (2.0, "picture"), (0.4, "bg"), (2.0, "picture"), (1.5, "label"), (1.0, "picture")]
        )
        self.assertEqual(flags, [], found)

    def test_a_clip_that_never_shows_anything_is_whole_blank(self):
        flags, _ = self.flags("whole", [(2.0, "bg")])
        self.assertEqual([k for *_, k in flags], ["whole"])

    def test_the_command_prints_flags_and_exits_zero_unless_gated(self):
        path = self.dir / "cmd.mp4"
        make_clip(path, [(0.6, "bg"), (2.0, "picture"), (0.8, "bg"), (2.0, "picture")])
        run = lambda *a: subprocess.run([sys.executable, "-I", str(TOOL), str(path), *a], capture_output=True, text=True)
        r = run()
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("FLAG cmd.mp4: no picture", r.stdout)
        self.assertIn("(0.80 s, inside)", r.stdout)
        self.assertIn("ADVISORY blank stretches: 1 flagged in 1 chapter file(s)", r.stdout)
        self.assertEqual(run("--gate").returncode, 1)
        good = self.dir / "good.mp4"
        make_clip(good, [(0.6, "bg"), (3.0, "picture")])
        r = subprocess.run([sys.executable, "-I", str(TOOL), str(good), "--gate"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("PASS good.mp4", r.stdout)

    def test_a_directory_is_read_in_name_order_and_watch_files_are_skipped(self):
        d = self.dir / "scored"
        d.mkdir()
        make_clip(d / "02_b.mp4", [(0.6, "bg"), (1.0, "picture")])
        make_clip(d / "01_a.mp4", [(0.6, "bg"), (1.0, "picture")])
        make_clip(d / "x_scored_watch.mp4", [(2.0, "bg")])
        files = bs.files_of([str(d)])
        self.assertEqual([f.name for f in files], ["01_a.mp4", "02_b.mp4"])


class Timelines(unittest.TestCase):
    """the opening is excused until the chapter's first line starts, read from the timeline beside the MP4"""

    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.dir = Path(cls.tmp.name)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def clip(self, name, segments, first_line_frame=None, fps=30):
        path = self.dir / f"{name}.mp4"
        make_clip(path, segments)
        if first_line_frame is not None:
            (self.dir / f"{name}.timeline.tsv").write_text(
                f"# explainer timeline v1\nchapter\t{name}\nfps\t{fps}\nframes\t200\n"
                f"line\t{name[:2]}_2\t{first_line_frame + 90}\t0\t/x/y.wav\nline\t{name[:2]}_1\t{first_line_frame}\t0\t/x/z.wav\n",
                encoding="utf-8",
            )
        return path

    def test_the_first_line_of_the_timeline_sets_how_long_the_opening_may_be(self):
        # Thought gaps: the line starts 1.2 s in, and the picture comes up with it
        path = self.clip("01_late_line", [(1.2, "bg"), (2.0, "picture")], first_line_frame=36)
        self.assertAlmostEqual(bs.first_line_s(path), 1.2, delta=1e-9)
        lines, n = bs.report(path, FPS)
        self.assertEqual(n, 0, lines)
        # the same clip with no timeline beside it is held to the 0.75 s lead
        bare = self.clip("02_no_timeline", [(1.2, "bg"), (2.0, "picture")])
        self.assertIsNone(bs.first_line_s(bare))
        self.assertEqual(bs.report(bare, FPS)[1], 1)

    def test_a_picture_that_comes_after_the_first_line_is_flagged_with_the_line_named(self):
        # K 05: the voice at 0.6 s, the first mark at 0.9 s (0.3 s after it)
        path = self.clip("03_after_the_voice", [(0.9, "bg"), (2.0, "picture")], first_line_frame=18)
        lines, n = bs.report(path, FPS)
        self.assertEqual(n, 1, lines)
        self.assertIn("(0.90 s, opening; its first line starts at 0.60 s)", lines[0])
        # G 15: the voice at 0.65 s, the first icon 3.6 s in
        path = self.clip("04_long", [(3.6, "bg"), (2.0, "picture")], first_line_frame=20)
        self.assertEqual(bs.report(path, FPS)[1], 1)

    def test_the_picture_with_the_voice_is_not_flagged_even_a_frame_late(self):
        path = self.clip("05_with_the_voice", [(0.7, "bg"), (2.0, "picture")], first_line_frame=18)
        self.assertEqual(bs.report(path, FPS)[1], 0)

    def test_the_timeline_of_a_60_fps_render_is_read_in_its_own_frames(self):
        path = self.clip("06_sixty", [(1.0, "bg"), (2.0, "picture")], first_line_frame=60, fps=60)
        self.assertAlmostEqual(bs.first_line_s(path), 1.0, delta=1e-9)
        self.assertEqual(bs.report(path, FPS)[1], 0)


class Runs(unittest.TestCase):
    def test_stretches_and_flags_on_a_plain_list(self):
        flat = [True] * 18 + [False] * 30 + [True] * 12 + [False] * 10 + [True] * 20
        found = bs.stretches(flat, 30)
        self.assertEqual([k for *_, k in found], ["opening", "inside", "closing"])
        self.assertEqual(found[0][:2], (0.0, 0.6))
        # the 0.6 s lead is excused, the 0.4 s pause is short, the 0.667 s closing is over the bound
        self.assertEqual([k for *_, k in bs.flagged(found)], ["closing"])
        # exactly the bound is not over it
        self.assertEqual(bs.flagged([(1.0, 1.5, "inside")]), [])
        self.assertEqual(bs.flagged([(1.0, 1.54, "inside")]), [(1.0, 1.54, "inside")])
        # an opening that ends with the first line (and its slack) is excused whatever its length
        self.assertEqual(bs.flagged([(0.0, 1.3, "opening")], first_line=1.2), [])
        self.assertEqual(bs.flagged([(0.0, 1.4, "opening")], first_line=1.2), [(0.0, 1.4, "opening")])
        self.assertEqual(bs.flagged([(0.0, 0.8, "opening")]), [(0.0, 0.8, "opening")])


if __name__ == "__main__":
    unittest.main()
