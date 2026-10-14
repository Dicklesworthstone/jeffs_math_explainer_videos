#!/usr/bin/env python3
"""Tests of tools/publish.py's pieces on small real files (ffmpeg makes them). Nothing is deleted."""

import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

import publish  # noqa: E402

FFMPEG = shutil.which("ffmpeg") and shutil.which("ffprobe")


def mean_luma(png: Path) -> float:
    out = subprocess.run(
        ["ffmpeg", "-hide_banner", "-i", str(png), "-vf", "signalstats,metadata=print:key=lavfi.signalstats.YAVG",
         "-f", "null", "-"], capture_output=True, text=True, timeout=60).stderr
    return float(re.findall(r"YAVG=([0-9.]+)", out)[-1])


class Publish(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="publish_pieces_"))

    def tearDown(self):
        shutil.move(str(self.root), tempfile.gettempdir() + f"/{self.root.name}.done")

    @unittest.skipUnless(FFMPEG, "needs ffmpeg")
    def test_the_contact_sheet_samples_the_whole_cut(self):
        # one second of black, then three of white: a sheet of the first frames is black, one over the
        # whole cut is three quarters white
        video = self.root / "cut.mp4"
        subprocess.run(
            ["ffmpeg", "-v", "error", "-f", "lavfi", "-i", "color=c=black:s=320x180:r=30:d=1",
             "-f", "lavfi", "-i", "color=c=white:s=320x180:r=30:d=3",
             "-filter_complex", "[0:v][1:v]concat=n=2:v=1[v]", "-map", "[v]", "-pix_fmt", "yuv420p", str(video)],
            check=True, timeout=60)
        sheet = self.root / "sheet.png"
        publish.build_contact_sheet(video, sheet)
        luma = mean_luma(sheet)
        self.assertGreater(luma, 150, f"mean luma {luma}: the sheet does not reach past the first second")
        # every tile is a still: the last (bottom right) is white, not the tile filter's black padding
        last = self.root / "last.png"
        subprocess.run(["ffmpeg", "-v", "error", "-i", str(sheet), "-vf", "crop=480:270:1440:270", str(last)],
                       check=True, timeout=60)
        self.assertGreater(mean_luma(last), 200, "the eighth tile is empty")

    def test_the_franken_manim_commit_is_read_from_the_lock(self):
        sha = "a399d626c43ecc852a5c795288979f52b446bc5c"
        (self.root / "Cargo.toml").write_text(
            f'fmn = {{ git = "https://github.com/Dicklesworthstone/franken_manim", rev = "{sha}" }}\n', encoding="utf-8")
        self.assertEqual(publish.fmn_commit(self.root), sha[:12])
        (self.root / "Cargo.lock").write_text(
            'source = "git+https://github.com/Dicklesworthstone/franken_manim?rev=a399d62#'
            '0123456789abcdef0123456789abcdef01234567"\n', encoding="utf-8")
        self.assertEqual(publish.fmn_commit(self.root), "0123456789ab")


if __name__ == "__main__":
    unittest.main()
