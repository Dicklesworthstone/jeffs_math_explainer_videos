#!/usr/bin/env python3
"""Tests of tools/release.py on a fixture cut and a local stand-in for the build host. Nothing is deleted."""

import hashlib
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

import release  # noqa: E402

DESC = """A pitch about von Neumann's trick.

Chapters
0:00 One
1:01 Two
1:48 Three

Narration: synthetic speech (FrankenTTS).
"""
SRT = """1
00:00:00,500 --> 00:00:02,000
A pitch about

2
00:00:02,100 --> 00:00:04,000
von Neumann's trick.

3
00:02:10,000 --> 00:02:18,000
The end.
"""
# three chapters of 61.5 s, 47.25 s and 30 s at 60 fps: starts 0, 61, 108; 138.75 s in all
TIMELINES = (("01_one", 3690), ("02_two", 2835), ("03_three", 1800))
GOOD_PROBE = {"width": 3840, "height": 2160, "fps": 60.0, "duration": 138.75}


def png(width, height, pad=0):
    """A PNG's signature, IHDR and IEND (enough for a header reader), padded to make a big file."""
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"tEXt", b"pad\x00" + b"x" * pad) + chunk(b"IEND", b"")


def vtt(srt):
    return "WEBVTT\n\n" + "\n\n".join(
        "\n".join(b.split("\n")[1:]).replace(",", ".", 2) for b in srt.strip().split("\n\n")) + "\n"


def make_cut(root, master=b"master v1", srt=SRT, frames=None):
    """A video 'alpha' with a final, its upload text and a thumbnail under `root`; returns its finals dir."""
    crate = root / "explainer_corpus" / "alpha"
    final = crate / "renders" / "final"
    (crate / "publish" / "michael").mkdir(parents=True, exist_ok=True)
    (crate / "docs").mkdir(parents=True, exist_ok=True)
    final.mkdir(parents=True, exist_ok=True)
    (crate / "script.tsv").write_text("01_1\tA pitch about von Neumann's trick.\n", encoding="utf-8")
    (crate / "docs" / "protected.tsv").write_text("von Neumann\tcaptions,description\twhose trick it is\n", encoding="utf-8")
    pub = crate / "publish"
    title, tags = "Alpha: A Title", "alpha, beta gamma"
    (pub / "michael" / "title.txt").write_text(title + "\n", encoding="utf-8")
    (pub / "michael" / "tags.txt").write_text(tags + "\n", encoding="utf-8")
    (pub / "michael" / "description.txt").write_text(DESC, encoding="utf-8")
    (pub / "youtube.md").write_text(
        f"# Alpha\n\n- Title: {title} (exactly this)\n- Altered or synthetic content: Yes (synthetic narration)\n"
        f"- Thumbnail: `renders/alpha_thumbnail.png` (`./x alpha thumbnail`)\n- Tags: {tags}\n\n"
        f"## Description\n\n```text\n{DESC}```\n", encoding="utf-8")
    (crate / "renders" / "alpha_thumbnail.png").write_bytes(png(1280, 720))
    (final / "alpha_master.mp4").write_bytes(master)
    (final / "alpha.en.srt").write_text(srt, encoding="utf-8")
    (final / "alpha.en.vtt").write_text(vtt(srt), encoding="utf-8")
    for key, n in TIMELINES:
        n = (frames or {}).get(key, n)
        (final / f"{key}.timeline.tsv").write_text(f"chapter\t{key}\nfps\t60\nframes\t{n}\n", encoding="utf-8")
    return final


class LocalHost:
    """The build host as a local directory: the same four calls as release.Host."""

    def __init__(self, corrupt=False):
        self.corrupt = corrupt

    def list(self, directory):
        p = Path(directory)
        return sorted(x.name for x in p.iterdir()) if p.is_dir() else []

    def md5s(self, directory, names):
        return {n: hashlib.md5((Path(directory) / n).read_bytes()).hexdigest()
                for n in names if (Path(directory) / n).is_file()}

    def copy(self, path, local):
        shutil.copyfile(path, local)
        if self.corrupt:
            with open(local, "ab") as f:
                f.write(b"!")

    def sha(self, clone):
        return "abc1234"

    def read(self, path):
        p = Path(path)
        return p.read_text(encoding="utf-8") if p.is_file() else ""


class Check(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="release_check_"))
        self.final = make_cut(self.root)
        self.probe = dict(GOOD_PROBE)

    def tearDown(self):
        shutil.move(str(self.root), tempfile.gettempdir() + f"/{self.root.name}.done")

    def check(self):
        return release.check("alpha", self.final, "michael", self.root, probe=lambda _: dict(self.probe))

    def failing(self, needle):
        bad, sheet = self.check()
        hits = [b for b in bad if needle in b]
        self.assertEqual(len(hits), 1, f"want one line with {needle!r}, got {bad}")
        self.assertEqual(sheet, [])
        return bad

    def test_a_ready_cut_passes_and_prints_the_upload_sheet(self):
        bad, sheet = self.check()
        self.assertEqual(bad, [])
        text = "\n".join(sheet)
        self.assertIn("Title: Alpha: A Title", text)
        self.assertIn("explainer_corpus/alpha/renders/alpha_thumbnail.png", text)
        self.assertIn("Visibility: Private", text)
        self.assertIn(hashlib.md5(b"master v1").hexdigest(), text)

    def test_chapter_times_of_another_final_fail(self):
        # the second chapter two seconds longer: the third starts at 1:50, not the upload text's 1:48
        self.final = make_cut(self.root, frames={"02_two": 2955})
        self.probe["duration"] = 140.75
        bad = self.failing("03_three starts at 1:50")
        self.assertEqual(len(bad), 1, bad)

    def test_a_master_that_is_not_this_finals_fails(self):
        self.probe["duration"] = 140.75
        self.failing("its timelines add to 138.750 s")

    def test_the_delivery_format(self):
        self.probe.update(width=1920, height=1080)
        self.failing("1920x1080")

    def test_captions(self):
        overlap = SRT.replace("00:00:02,100 --> 00:00:04,000", "00:00:01,900 --> 00:00:04,000")
        self.final = make_cut(self.root, srt=overlap)
        self.failing("before cue 1 ends")
        late = SRT.replace("00:02:10,000 --> 00:02:18,000", "00:02:10,000 --> 00:02:19,500")
        self.final = make_cut(self.root, srt=late)
        self.failing("after the master")
        self.final = make_cut(self.root)
        (self.final / "alpha.en.vtt").write_text(vtt(SRT.replace("The end.", "The End!")), encoding="utf-8")
        self.failing("VTT: its cue texts are not the SRT's")
        self.final = make_cut(self.root, srt=SRT.replace("von Neumann's", "his"))
        self.failing("alpha.en.srt: the protected phrase 'von Neumann'")
        # the last word of a sentence alone on a cue (E's first captions had nine)
        orphan = SRT.replace("von Neumann's trick.", "von Neumann's trick, the").replace("The end.", "end.")
        self.final = make_cut(self.root, srt=orphan)
        self.failing("SRT cue 3: 'end.' (8.00 s) is the tail of the sentence cue 2 began")
        # a brief tail of more words is one too; a whole short sentence after a full stop is not
        orphan = SRT.replace("von Neumann's trick.", "von Neumann's trick, and").replace(
            "00:02:10,000 --> 00:02:18,000\nThe end.", "00:02:10,000 --> 00:02:10,600\nthat is the end.")
        self.final = make_cut(self.root, srt=orphan)
        self.failing("'that is the end.' (0.60 s) is the tail")
        self.final = make_cut(self.root, srt=SRT.replace("The end.", "End."))
        self.assertEqual(self.check()[0], [])

    def test_vtt_times_match_srt(self):
        target = self.final/'alpha.en.vtt'
        original = vtt(SRT)
        for old, new in (('00:00:00.500', '00:00:00.501'), ('00:00:02.000', '00:00:02.001')):
            target.write_text(original.replace(old, new, 1))
            self.failing("VTT: its cue times are not the SRT's")

    def test_the_thumbnail(self):
        thumb = self.root / "explainer_corpus" / "alpha" / "renders" / "alpha_thumbnail.png"
        thumb.write_bytes(png(1280, 719))
        self.failing("(1280, 719)")
        thumb.write_bytes(png(1280, 720, pad=2 * 1024 * 1024))
        self.failing("YouTube takes 2 MB")
        thumb.rename(thumb.with_name("aside.png"))
        self.failing("./x alpha thumbnail")

    def test_a_manifest_row_must_carry_the_masters_md5(self):
        manifest = self.root / "docs" / "publish" / "MANIFEST.md"
        manifest.parent.mkdir(parents=True)
        path = "explainer_corpus/alpha/renders/final/alpha_master.mp4"
        manifest.write_text(f"| alpha | `{path}` (`{'0' * 32}`) | Private |\n", encoding="utf-8")
        self.failing("names explainer_corpus/alpha/renders/final/alpha_master.mp4 with md5 000")
        manifest.write_text(f"| alpha | `{path}` (`{hashlib.md5(b'master v1').hexdigest()}`) |\n", encoding="utf-8")
        self.assertEqual(self.check()[0], [])

    def test_a_missing_file_is_named(self):
        (self.final / "alpha.en.vtt").rename(self.final / "alpha.en.vtt.aside")
        self.failing("0 VTT, want one")

    @unittest.skipUnless(shutil.which("ffmpeg") and shutil.which("ffprobe"), "needs ffmpeg")
    def test_the_real_probe_reads_a_4k60_file(self):
        out = self.root / "probe.mp4"
        subprocess.run(["ffmpeg", "-v", "error", "-f", "lavfi", "-i", "testsrc=size=3840x2160:rate=60",
                        "-frames:v", "2", "-pix_fmt", "yuv420p", str(out)], check=True, timeout=120)
        p = release.ffprobe(out)
        self.assertEqual((p["width"], p["height"], p["fps"]), (3840, 2160, 60.0))


class Fetch(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="release_fetch_"))
        self.local = self.tmp / "laptop"
        self.clone = self.tmp / "host" / "clone"
        make_cut(self.clone, master=b"master v2")
        self.final = make_cut(self.local, master=b"master v1")
        self.thumb = self.local / "explainer_corpus" / "alpha" / "renders" / "alpha_thumbnail.png"
        (self.clone / "explainer_corpus" / "alpha" / "renders" / "alpha_thumbnail.png").write_bytes(png(1280, 720, pad=7))

    def tearDown(self):
        shutil.move(str(self.tmp), tempfile.gettempdir() + f"/{self.tmp.name}.done")

    def fetch(self, host=None):
        return release.fetch("alpha", self.final, f"host:{self.clone}", host or LocalHost(), self.local)

    def test_a_new_final_replaces_the_old_which_is_kept_whole(self):
        log = self.fetch()
        self.assertIn("preserved the old final as final_v1", "\n".join(log))
        self.assertEqual((self.final / "alpha_master.mp4").read_bytes(), b"master v2")
        self.assertEqual((self.final.with_name("final_v1") / "alpha_master.mp4").read_bytes(), b"master v1")
        self.assertEqual(len(list(self.final.glob("*.timeline.tsv"))), 3)
        # the thumbnail came too, the old one kept beside it
        self.assertEqual(self.thumb.read_bytes(), png(1280, 720, pad=7))
        self.assertTrue(self.thumb.with_name("alpha_thumbnail.v1.png").is_file())
        # the same final again moves nothing
        log = self.fetch()
        self.assertNotIn("preserved", "\n".join(log))
        self.assertFalse(self.final.with_name("final_v2").exists())
        # a changed caption on the host is a new cut: the whole local final goes aside again
        host_final = self.clone / "explainer_corpus" / "alpha" / "renders" / "final"
        again = SRT.replace("The end.", "The end, again.")
        (host_final / "alpha.en.srt").write_text(again, encoding="utf-8")
        (host_final / "alpha.en.vtt").write_text(vtt(again), encoding="utf-8")
        self.fetch()
        self.assertTrue((self.final.with_name("final_v2") / "alpha_master.mp4").is_file())
        self.assertIn("again", (self.final / "alpha.en.srt").read_text(encoding="utf-8"))
        # and the fetched cut passes its check
        bad, _ = release.check("alpha", self.final, "michael", self.local, probe=lambda _: dict(GOOD_PROBE))
        self.assertEqual(bad, [])

    def test_a_bad_copy_is_refused_and_kept_aside(self):
        with self.assertRaises(RuntimeError):
            self.fetch(LocalHost(corrupt=True))
        self.assertTrue(list(self.final.glob("*.part.bad")))
        self.assertFalse((self.final / "alpha_master.mp4").exists())  # the old final went aside first
        self.assertEqual((self.final.with_name("final_v1") / "alpha_master.mp4").read_bytes(), b"master v1")

    def test_a_host_without_a_final_is_refused(self):
        with self.assertRaises(RuntimeError):
            release.fetch("alpha", self.final, f"host:{self.tmp / 'nowhere'}", LocalHost(), self.local)

    def test_the_commit_the_final_was_rendered_at_comes_along(self):
        # no mark with a rev on the host: said, and nothing recorded
        log = "\n".join(self.fetch())
        self.assertIn("RENDERED-AT UNKNOWN", log)
        self.assertFalse((self.final / ".rendered_at").exists())
        # the host's final.ok, as x now writes it
        marks = self.clone / "explainer_corpus" / "alpha" / ".x"
        marks.mkdir()
        (marks / "final.ok").write_text("Wed Oct 14 16:52:01 EDT 2026 rev fbe959c\n", encoding="utf-8")
        self.fetch()
        self.assertTrue((self.final / ".rendered_at").read_text(encoding="utf-8").startswith("fbe959c (from host:"))
        # --rev wins
        release.fetch("alpha", self.final, f"host:{self.clone}", LocalHost(), self.local, rev="0720552")
        self.assertTrue((self.final / ".rendered_at").read_text(encoding="utf-8").startswith("0720552 (from --rev)"))


class SinceCut(unittest.TestCase):
    """since_cut on a real git history: a source fix after the cut is named, thumbnail wiring and docs are not."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="release_since_"))
        self.git("init", "-q")
        self.git("config", "user.email", "t@example.invalid")
        self.git("config", "user.name", "t")
        self.final = make_cut(self.root)
        self.commit("explainer_corpus/alpha/src/a.rs", "the cut")
        self.cut = self.git("rev-parse", "--short", "HEAD").strip()

    def tearDown(self):
        shutil.move(str(self.root), tempfile.gettempdir() + f"/{self.root.name}.done")

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], capture_output=True, text=True, check=True).stdout

    def commit(self, path, message, *more):
        for p in (path, *more):
            f = self.root / p
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(f"{message}\n", encoding="utf-8")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)
        return self.git("rev-parse", "--short", "HEAD").strip()

    def test_fixes_after_the_cut_are_named(self):
        self.assertIn("not recorded", release.since_cut("alpha", self.final, self.root)[0])
        (self.final / ".rendered_at").write_text(f"{self.cut} (from --rev)\n", encoding="utf-8")
        self.assertEqual(release.since_cut("alpha", self.final, self.root), [])
        fix = self.commit("explainer_corpus/alpha/src/chapters/ten.rs", "the band stands longer")
        self.commit("explainer_corpus/alpha/src/chapters/thumbnail.rs", "the thumbnail scene",
                    "explainer_corpus/alpha/src/chapters/mod.rs", "explainer_corpus/alpha/src/main.rs")
        self.commit("explainer_corpus/alpha/docs/notes.md", "notes")
        kit = self.commit("crates/explainer_kit/src/captions.rs", "captions split evenly")
        notes = release.since_cut("alpha", self.final, self.root)
        self.assertEqual(len(notes), 2, notes)
        self.assertTrue(notes[0].startswith(f"NOTE: {kit} came after this cut ({self.cut})"))
        self.assertIn(f"{fix} came after", notes[1])
        self.assertIn("the band stands longer", notes[1])
        (self.final / ".rendered_at").write_text("deadbee (from --rev)\n", encoding="utf-8")
        self.assertIn("not in this checkout's history", release.since_cut("alpha", self.final, self.root)[0])


if __name__ == "__main__":
    unittest.main()
