#!/usr/bin/env python3
"""Unit tests for tools/final_check.py (mx-tkp6).

Tests a fixture final in a temporary directory with mocked probe and ebur128:
- Positive test: fixture passes with 0 defects.
- 9 planted negatives, each asserting the exact defect:
  1. caption word missing
  2. three-line cue
  3. 43-character line
  4. cue crossing a sentence boundary
  5. VTT text differing from SRT
  6. chapter longer than 85 s
  7. chapter 0.5 s off its timeline
  8. A/V delta 0.3 s
  9. 1920x1080 master
- Strict exit code verification.
"""
import copy
import json
import shutil
import tempfile
import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import final_check


class TestFinalCheck(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp(prefix="test_final_check_")
        self.root = Path(self.temp_dir)
        self.slug = "test_video"
        self.crate = self.root / "explainer_corpus" / self.slug
        self.final = self.crate / "renders" / "final"
        self.final.mkdir(parents=True, exist_ok=True)
        (self.crate / "narration" / "out").mkdir(parents=True, exist_ok=True)
        (self.crate / ".x").mkdir(parents=True, exist_ok=True)

        # Marks
        (self.crate / ".x" / "final.ok").write_text("ok", encoding="utf-8")
        (self.crate / ".x" / "master.ok").write_text("ok", encoding="utf-8")
        (self.crate / ".x" / "verify.ok").write_text("ok", encoding="utf-8")

        # script.tsv
        self.script_tsv = (
            "id\ttext\n"
            "01_1\tFirst sentence.\n"
            "01_2\tSecond sentence. Third sentence.\n"
        )
        (self.crate / "script.tsv").write_text(self.script_tsv, encoding="utf-8")

        # report.json
        self.report_json = {
            "lines": {
                "01_1": [{"sentence": "First sentence.", "out_samples": 24000}],
                "01_2": [
                    {"sentence": "Second sentence.", "out_samples": 24000},
                    {"sentence": "Third sentence.", "out_samples": 24000},
                ],
            }
        }
        (self.crate / "narration" / "out" / "report.json").write_text(
            json.dumps(self.report_json), encoding="utf-8"
        )

        # Chapter MP4 & timeline
        (self.final / "01_chap.mp4").write_text("", encoding="utf-8")
        self.timeline_tsv = (
            "fps\t60\n"
            "frames\t1800\n"
            "line\t01_1\t0\t0.0\t01_1.wav\n"
            "line\t01_2\t600\t0.0\t01_2.wav\n"
        )
        (self.final / "01_chap.timeline.tsv").write_text(self.timeline_tsv, encoding="utf-8")

        # Master MP4
        (self.final / f"{self.slug}_master.mp4").write_text("", encoding="utf-8")

        # SRT and VTT
        self.srt_content = (
            "1\n"
            "00:00:00,000 --> 00:00:01,000\n"
            "First sentence.\n"
            "\n"
            "2\n"
            "00:00:10,000 --> 00:00:11,000\n"
            "Second sentence.\n"
            "\n"
            "3\n"
            "00:00:11,500 --> 00:00:12,500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.srt").write_text(self.srt_content, encoding="utf-8")

        self.vtt_content = (
            "WEBVTT\n"
            "\n"
            "00:00:00.000 --> 00:00:01.000\n"
            "First sentence.\n"
            "\n"
            "00:00:10.000 --> 00:00:11.000\n"
            "Second sentence.\n"
            "\n"
            "00:00:11.500 --> 00:00:12.500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.vtt").write_text(self.vtt_content, encoding="utf-8")

        # Default probe responses
        self.probe_data = {
            "master_video": {
                "codec_name": "h264",
                "profile": "High",
                "width": 3840,
                "height": 2160,
                "r_frame_rate": "60/1",
                "avg_frame_rate": "60/1",
                "pix_fmt": "yuv420p",
                "color_range": "tv",
                "color_space": "bt709",
                "color_primaries": "bt709",
                "color_transfer": "bt709",
                "nb_frames": 1800,
                "duration": "30.000000",
                "bit_rate": "50000000",
            },
            "master_audio": {
                "codec_name": "pcm_s16le",
                "sample_rate": "48000",
                "channels": 2,
                "duration": "30.000000",
                "bits_per_sample": 16,
                "bit_rate": "1536000",
            },
            "master_format": {
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
                "duration": "30.000000",
                "size": "187500000",
                "bit_rate": "50000000",
            },
            "chapter_video": {
                "codec_name": "h264",
                "width": 3840,
                "height": 2160,
                "r_frame_rate": "60/1",
                "nb_frames": 1800,
                "color_primaries": "bt709",
                "color_transfer": "bt709",
                "color_space": "bt709",
            },
            "chapter_format": {"duration": "30.000000"},
            "audio_wav_format": {"duration": "1.000000"},
        }

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def make_probe_fn(self, overrides=None):
        data = copy.deepcopy(self.probe_data)
        if overrides:
            for k, v in overrides.items():
                if isinstance(v, dict):
                    data[k].update(v)
                else:
                    data[k] = v

        def probe(path, entries, stream=None):
            p = Path(path)
            if "_master" in p.name:
                if stream == "v:0":
                    return {"streams": [data["master_video"]]}
                elif stream == "a:0":
                    return {"streams": [data["master_audio"]]}
                else:
                    return {"format": data["master_format"]}
            elif p.suffix == ".mp4":
                if stream == "v:0":
                    return {"streams": [data["chapter_video"]]}
                else:
                    return {"format": data["chapter_format"]}
            elif p.suffix == ".wav":
                return {"format": data["audio_wav_format"]}
            return {}

        return probe

    def make_ebur128_fn(self, lufs=-16.0, tp=-1.0, lra="5.0"):
        return lambda _path: (lufs, tp, lra)

    def test_fixture_pass(self):
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            strict=True,
            loud=True,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertEqual(defects, [], f"Expected 0 defects, got: {defects}")

    def test_caption_word_missing(self):
        bad_srt = (
            "1\n"
            "00:00:00,000 --> 00:00:01,000\n"
            "First.\n"  # "sentence" dropped
            "\n"
            "2\n"
            "00:00:10,000 --> 00:00:11,000\n"
            "Second sentence.\n"
            "\n"
            "3\n"
            "00:00:11,500 --> 00:00:12,500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.srt").write_text(bad_srt, encoding="utf-8")
        (self.final / f"{self.slug}.en.vtt").write_text("WEBVTT\n\n" + bad_srt.replace(",", "."), encoding="utf-8")

        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("SRT's words differ" in d for d in defects), f"Expected word diff defect, got: {defects}")

    def test_three_line_cue(self):
        bad_srt = (
            "1\n"
            "00:00:00,000 --> 00:00:01,000\n"
            "First\nline\nsentence.\n"
            "\n"
            "2\n"
            "00:00:10,000 --> 00:00:11,000\n"
            "Second sentence.\n"
            "\n"
            "3\n"
            "00:00:11,500 --> 00:00:12,500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.srt").write_text(bad_srt, encoding="utf-8")
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("cue 1: 3 lines" in d for d in defects), f"Expected 3-lines defect, got: {defects}")

    def test_43_character_line(self):
        # 43 characters with space
        long_line = "A" * 21 + " " + "B" * 21  # 21 + 1 + 21 = 43 chars
        bad_srt = (
            "1\n"
            "00:00:00,000 --> 00:00:01,000\n"
            f"{long_line}\n"
            "\n"
            "2\n"
            "00:00:10,000 --> 00:00:11,000\n"
            "Second sentence.\n"
            "\n"
            "3\n"
            "00:00:11,500 --> 00:00:12,500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.srt").write_text(bad_srt, encoding="utf-8")
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("line over 42 characters" in d for d in defects), f"Expected >42 chars defect, got: {defects}")

    def test_cue_crossing_sentence_boundary(self):
        # Merge parts of sentence 2 and 3 into one cue
        bad_srt = (
            "1\n"
            "00:00:00,000 --> 00:00:01,000\n"
            "First sentence.\n"
            "\n"
            "2\n"
            "00:00:10,000 --> 00:00:12,000\n"
            "Second sentence. Third\n"
            "\n"
            "3\n"
            "00:00:12,000 --> 00:00:12,500\n"
            "sentence.\n"
        )
        (self.final / f"{self.slug}.en.srt").write_text(bad_srt, encoding="utf-8")
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("exact text" in d for d in defects), f"Expected boundary defect, got: {defects}")

    def test_vtt_text_differing(self):
        bad_vtt = (
            "WEBVTT\n"
            "\n"
            "00:00:00.000 --> 00:00:01.000\n"
            "First different sentence.\n"
            "\n"
            "00:00:10.000 --> 00:00:11.000\n"
            "Second sentence.\n"
            "\n"
            "00:00:11.500 --> 00:00:12.500\n"
            "Third sentence.\n"
        )
        (self.final / f"{self.slug}.en.vtt").write_text(bad_vtt, encoding="utf-8")
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("the VTT does not hold the SRT's cues and text" in d for d in defects), f"Expected VTT diff defect, got: {defects}")

    def test_chapter_longer_than_85s(self):
        probe_fn = self.make_probe_fn({"chapter_format": {"duration": "86.000000"}})
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=probe_fn,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("is over 85" in d for d in defects), f"Expected chapter length defect, got: {defects}")

    def test_chapter_off_timeline(self):
        # Timeline says 30.0 s, but file is 30.5 s
        probe_fn = self.make_probe_fn({"chapter_format": {"duration": "30.500000"}})
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=probe_fn,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("file 30.50 s vs timeline 30.00 s" in d for d in defects), f"Expected timeline delta defect, got: {defects}")

    def test_av_delta(self):
        # Audio duration 30.3 s vs video 30.0 s -> delta 0.3 s > 0.2 s
        probe_fn = self.make_probe_fn({"master_audio": {"duration": "30.300000"}})
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=probe_fn,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("A/V duration delta 0.300 s > 0.2 s" in d for d in defects), f"Expected A/V delta defect, got: {defects}")

    def test_master_resolution(self):
        probe_fn = self.make_probe_fn({"master_video": {"width": 1920, "height": 1080}})
        defects = final_check.check_final(
            slug=self.slug,
            root=self.root,
            probe_fn=probe_fn,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertTrue(any("master is 1920x1080, not 3840x2160" in d for d in defects), f"Expected 1920x1080 defect, got: {defects}")

    def test_strict_exit_code(self):
        # Clean fixture with strict returns 0
        ret_clean = final_check.main(
            [self.slug, str(self.root), "--strict", "--no-loudness"],
            probe_fn=self.make_probe_fn(),
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertEqual(ret_clean, 0)

        # Negative fixture with strict returns 1
        bad_probe = self.make_probe_fn({"master_video": {"width": 1920, "height": 1080}})
        ret_bad = final_check.main(
            [self.slug, str(self.root), "--strict", "--no-loudness"],
            probe_fn=bad_probe,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertEqual(ret_bad, 1)

        # Reader mode (no --strict) returns 0 even with defects
        ret_reader = final_check.main(
            [self.slug, str(self.root), "--no-loudness"],
            probe_fn=bad_probe,
            ebur128_fn=self.make_ebur128_fn(),
        )
        self.assertEqual(ret_reader, 0)


if __name__ == "__main__":
    unittest.main()
