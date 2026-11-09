"""Pure console syntax and option propagation, not native render evidence."""
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import unittest

# This can also run without a built extension: python tests/video_cli_unit.py.
_path = Path(__file__).resolve().parents[1] / "python/fmn_python/video_cli.py"
if not _path.is_file():
    _path = Path(__file__).with_name("video_cli.py")
_spec = importlib.util.spec_from_file_location("fmn_video_cli_unit_target", _path)
v = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(v)


class VideoCliTests(unittest.TestCase):
    def test_every_crf_in_both_spellings(self):
        for crf in range(52):
            for tokens in (["--crf", str(crf)], [f"--crf={crf}"]):
                with self.subTest(tokens=tokens):
                    self.assertEqual(v.take_video_options(tokens), ([], {"video_crf": crf}))

    def test_bit_rates_remain_bits_not_kilobits_or_strings(self):
        for value in (1, 256000, 12000000, (1 << 32) - 1):
            self.assertEqual(v.take_video_options(["--video-bitrate", str(value)])[1],
                             {"video_bitrate": value})

    def test_mixed_forms_keep_native_option_values_verbatim(self):
        original = ["--ffmpeg_bin", "--crf", "--preset=slow", "--file_name", "--tune",
                    "--format", "mov", "--crf", "0", "--tune", "animation"]
        forwarded, overrides = v.take_video_options(original, {"--ffmpeg_bin", "--file_name", "--format"})
        self.assertEqual(forwarded, ["--ffmpeg_bin", "--crf", "--file_name", "--tune", "--format", "mov"])
        self.assertEqual(overrides, {"video_preset": "slow", "video_crf": 0, "video_tune": "animation"})
        self.assertEqual(original[2], "--preset=slow")

    def test_invalid_numbers(self):
        for flag, values in (("--crf", ("52", "-1", "+1", "1.0", "nan", "", " 16", "١٦", "1" * 1000)),
                             ("--video-bitrate", ("0", "4294967296", "12M", "1e6", "-1", ""))):
            for value in values:
                with self.subTest(flag=flag, value=value):
                    with self.assertRaises(ValueError):
                        v.take_video_options([flag, value])

    def test_no_argv_fragments_or_selector_values(self):
        for flag in ("--preset", "--tune"):
            for value in ("Slow", "slow -vf eq", "animation,grain", "\x00", "", "--skip_animations", "a" * 33):
                with self.subTest(flag=flag, value=value):
                    with self.assertRaises(ValueError):
                        v.take_video_options([flag, value])
        # Native Reel, not this lexer, owns the catalog and encoder matrix.
        self.assertEqual(v.take_video_options(["--preset", "nativecatalogcheck"])[1],
                         {"video_preset": "nativecatalogcheck"})

    def test_missing_duplicates_and_conflicting_modes(self):
        for tokens in (["--crf"], ["--preset="], ["--crf", "1", "--crf=1"],
                       ["--preset=slow", "--preset", "fast"],
                       ["--crf=0", "--video-bitrate=1000000"]):
            with self.subTest(tokens=tokens):
                with self.assertRaises(ValueError):
                    v.take_video_options(tokens)

    def test_native_and_alpha_formats_refuse_not_ignore(self):
        for format in ("png", "png_sequence", "svg", "gif", "y4m", "wav", "fmtl"):
            v.validate_video_format(format, {})
            with self.assertRaises(ValueError):
                v.validate_video_format(format, {"video_crf": 0})
        for format in ("mp4", "mov"):
            v.validate_video_format(format, {"video_crf": 0})
            with self.assertRaises(ValueError):
                v.validate_video_format(format, {"video_crf": 0}, transparent=True)

    def test_application_preserves_zero_and_unrelated_writer_state(self):
        writer = SimpleNamespace(video_crf=28, video_preset="fast", saturation=1.0)
        scene = SimpleNamespace(file_writer=writer, camera=object())
        settings = {"video_crf": 0, "video_preset": None, "video_tune": "animation", "unrelated": 99}
        v.apply_video_options(scene, settings)
        self.assertIs(scene.file_writer, writer)
        self.assertEqual(vars(writer), {"video_crf": 0, "video_preset": "fast", "saturation": 1.0,
                                       "video_tune": "animation"})
        self.assertEqual(settings["video_preset"], None)

    def test_video_does_not_evaluate_getters_twice(self):
        class Writer:
            def __getattr__(self, name):
                raise RuntimeError("authored getter")
        for format in ("mp4", "mov"):
            v.validate_writer_video_format(Writer(), format)
        with self.assertRaisesRegex(RuntimeError, "authored getter"):
            v.validate_writer_video_format(Writer(), "png")
        v.validate_writer_video_format(SimpleNamespace(), "png")
        with self.assertRaises(ValueError):
            v.validate_writer_video_format(SimpleNamespace(video_crf=0), "png")


if __name__ == "__main__":
    unittest.main()
