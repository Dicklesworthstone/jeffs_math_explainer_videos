"""One native scene execution publishes an animation and its final RGBA PNG."""
import hashlib
import json
import os
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

import numpy as np
import manimlib as m
from fmn_python import PairedRenderResult, paired_render_session, render_scene_with_still


def png_pixels(path):
    data = Path(path).read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    packed, offset, shape = bytearray(), 8, None
    while offset < len(data):
        size = int.from_bytes(data[offset:offset + 4], "big")
        kind, body = data[offset + 4:offset + 8], data[offset + 8:offset + 8 + size]
        crc = int.from_bytes(data[offset + 8 + size:offset + 12 + size], "big")
        assert len(body) == size and zlib.crc32(kind + body) & 0xffffffff == crc
        offset += size + 12
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", body)
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
            shape = width, height
        elif kind == b"IDAT":
            packed.extend(body)
        elif kind == b"IEND":
            assert not size and offset == len(data)
            break
    assert shape is not None
    width, height = shape
    raw, stride = zlib.decompress(packed), width * 4
    assert len(raw) == height * (stride + 1)
    decoded = bytearray(height * stride)
    for y in range(height):
        mode = raw[y * (stride + 1)]
        assert mode in range(5)
        for x, value in enumerate(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)]):
            left = decoded[y * stride + x - 4] if x >= 4 else 0
            up = decoded[(y - 1) * stride + x] if y else 0
            corner = decoded[(y - 1) * stride + x - 4] if y and x >= 4 else 0
            estimate = left + up - corner
            paeth = min((left, up, corner), key=lambda item: abs(estimate - item))
            decoded[y * stride + x] = (value + (0, left, up, (left + up) // 2, paeth)[mode]) & 255
    return np.frombuffer(decoded, np.uint8).reshape(height, width, 4)


class Moving(m.Scene):
    default_camera_config = dict(resolution=(96, 54), fps=8)

    def __init__(self, **kwargs):
        self.calls = []
        super().__init__(**kwargs)

    def setup(self):
        self.calls.append("setup")

    def construct(self):
        self.calls.append("construct")
        self.box = m.Square(side_length=1, fill_color=m.RED, fill_opacity=1, stroke_width=0)
        self.add(self.box)
        self.play(self.box.animate.shift(m.RIGHT), run_time=0.25, rate_func=m.linear)
        self.wait(0.125)

    def tear_down(self):
        self.calls.append("tear_down")


class PairedOutput(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-paired-output-"))

    def receipt(self, result):
        data = result.destination.read_bytes()
        self.assertEqual(len(data), result.bytes)
        self.assertEqual(hashlib.sha256(data).hexdigest(), result.digest)
        return data

    def test_y4m_and_png_from_one_scene_lifecycle(self):
        scene = Moving()
        result = render_scene_with_still(scene, self.root / "movie.y4m", self.root / "still.png", threads=1)
        self.assertIsInstance(result, PairedRenderResult)
        self.assertTrue(result.completed)
        self.assertIs(scene.render_result, result)
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])
        self.assertAlmostEqual(scene.time, 0.375)
        self.assertEqual(result.primary.frame_count, 3)
        self.assertTrue(self.receipt(result.primary).startswith(b"YUV4MPEG2 "))
        self.receipt(result.still)
        snapshot = scene.camera.capture_snapshot(*scene.mobjects)
        np.testing.assert_array_equal(png_pixels(result.still.destination),
            np.frombuffer(snapshot.pixels(), np.uint8).reshape(54, 96, 4))
        self.assertGreater(int(png_pixels(result.still.destination)[:, :, :3].max()), 0)
        self.assertEqual(json.loads(json.dumps(result.as_dict()))["still"]["certified"], False)

    def test_stock_writer_preferences_publish_both_configured_paths(self):
        scene = Moving(file_writer_config=dict(write_to_movie=True, save_last_frame=True,
            movie_file_extension=".y4m", output_directory=str(self.root), file_name="chosen"))
        result = scene.render(threads=1)
        self.assertEqual(result.primary.destination, self.root / "chosen.y4m")
        self.assertEqual(result.still.destination, self.root / "chosen.png")
        self.receipt(result.primary)
        self.receipt(result.still)
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])

    def test_imperative_session_does_not_run_construct(self):
        scene = Moving()
        with paired_render_session(scene, self.root / "imperative.gif", self.root / "imperative.png", threads=1) as session:
            scene.add(m.Circle(fill_opacity=1))
            scene.wait(0.25)
        self.assertEqual(scene.calls, [])
        self.assertEqual(session.result.primary.frame_count, 2)
        self.assertTrue(self.receipt(session.result.primary).startswith(b"GIF89a"))
        self.receipt(session.result.still)
        self.assertIs(session.finish(), session.result)

    def test_configured_subdivision_preserves_clips_and_final_still(self):
        scene = Moving(file_writer_config=dict(write_to_movie=True, save_last_frame=True,
            subdivide_output=True, movie_file_extension=".y4m", output_directory=str(self.root), file_name="parts"))
        result = scene.render(threads=1)
        self.assertTrue(result.completed)
        self.assertEqual(result.destination, self.root / "parts" / "clips")
        self.assertEqual(len(result.primary.segments), 2)
        self.assertTrue(result.primary.completed)
        self.assertEqual(result.still.destination, self.root / "parts.png")
        for segment in result.primary.segments:
            self.receipt(segment.render)
        self.receipt(result.still)
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])

    def test_transparent_gif_keeps_full_resolution_rgba_still(self):
        scene = Moving(camera_config=dict(resolution=(96, 54), fps=8, background_opacity=0))
        result = render_scene_with_still(scene, self.root / "alpha.gif", self.root / "alpha.png", threads=1)
        pixels = png_pixels(result.still.destination)
        self.assertEqual(pixels.shape, (54, 96, 4))
        self.assertEqual(int(pixels[0, 0, 3]), 0)
        self.assertGreater(int(pixels[:, :, 3].max()), 0)
        self.receipt(result.still)

    def test_threads_preserve_native_animation_and_still_bytes(self):
        results = [render_scene_with_still(Moving, self.root / f"t{threads}.y4m",
            self.root / f"t{threads}.png", threads=threads) for threads in (1, 4)]
        self.assertEqual(self.receipt(results[0].primary), self.receipt(results[1].primary))
        self.assertEqual(self.receipt(results[0].still), self.receipt(results[1].still))

    def test_failed_construct_aborts_both_without_rerunning(self):
        original = RuntimeError("authored construction failed")
        class Failing(Moving):
            def construct(self):
                super().construct()
                raise original
        scene = Failing()
        with self.assertRaises(RuntimeError) as caught:
            render_scene_with_still(scene, self.root / "failure.y4m", self.root / "failure.png", threads=1)
        self.assertIs(caught.exception, original)
        self.assertFalse(original.render_pair_result.completed)
        self.assertIsNone(original.render_pair_result.primary)
        self.assertIsNone(original.render_pair_result.still)
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])
        self.assertEqual(list(self.root.iterdir()), [])
        self.assertNotIn("_fmn_owned_render_session", vars(scene))
        self.assertNotIn("_fmn_paired_render_session", vars(scene))

    def test_collision_created_during_construct_prevents_both_publications(self):
        still = self.root / "collision.png"
        class Colliding(Moving):
            def construct(self):
                super().construct()
                still.write_bytes(b"another output")
        with self.assertRaises(FileExistsError):
            render_scene_with_still(Colliding, self.root / "collision.y4m", still, threads=1)
        self.assertEqual(still.read_bytes(), b"another output")
        self.assertFalse((self.root / "collision.y4m").exists())
        self.assertEqual(list(self.root.iterdir()), [still])

    def test_primary_collision_after_png_preparation_leaves_png_unpublished(self):
        movie, still = self.root / "race.y4m", self.root / "race.png"
        class Collision(Moving):
            def _finish_render(self):
                movie.write_bytes(b"concurrent primary")
                return super()._finish_render()
        with self.assertRaises(Exception) as caught:
            render_scene_with_still(Collision, movie, still, threads=1)
        self.assertEqual(movie.read_bytes(), b"concurrent primary")
        self.assertFalse(still.exists())
        self.assertIsNone(caught.exception.render_pair_result.primary)
        self.assertEqual(list(self.root.iterdir()), [movie])

    def test_late_png_collision_retains_receipted_primary(self):
        movie, still = self.root / "late.y4m", self.root / "late.png"
        class Collision(Moving):
            def _finish_render(self):
                receipt = super()._finish_render()
                still.write_bytes(b"concurrent still")
                return receipt
        with self.assertRaises(Exception) as caught:
            render_scene_with_still(Collision, movie, still, threads=1)
        partial = caught.exception.render_pair_result
        self.assertFalse(partial.completed)
        self.assertIsNone(partial.still)
        self.assertEqual(partial.primary.destination, movie)
        self.receipt(partial.primary)
        self.assertEqual(still.read_bytes(), b"concurrent still")
        self.assertEqual(set(self.root.iterdir()), {movie, still})

    def test_final_capture_failure_aborts_live_primary(self):
        scene = Moving()
        def fail(*mobjects):
            raise RuntimeError("final capture refused")
        scene.camera.capture_snapshot = fail
        with self.assertRaisesRegex(RuntimeError, "final capture refused"):
            render_scene_with_still(scene, self.root / "failed.y4m", self.root / "failed.png", threads=1)
        self.assertEqual(list(self.root.iterdir()), [])

    def test_end_scene_still_finishes_once(self):
        class Early(Moving):
            def construct(self):
                super().construct()
                raise m.EndScene("normal stop")
        scene = Early()
        result = render_scene_with_still(scene, self.root / "early.y4m", self.root / "early.png", threads=1)
        self.assertTrue(result.completed)
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])
        self.receipt(result.primary)
        self.receipt(result.still)

    def test_static_scene_publishes_movie_and_still_without_advancing_clock(self):
        class Static(Moving):
            def construct(self):
                self.calls.append("construct")
                self.add(m.Square(fill_opacity=1))
        scene = Static()
        result = render_scene_with_still(scene, self.root / "static.y4m", self.root / "static.png", threads=1)
        self.assertEqual(scene.time, 0.)
        self.assertEqual(result.primary.frame_count, 1)
        self.receipt(result.still)

    def test_audio_output_and_still_share_one_scene(self):
        import wave
        tone = self.root / "tone.wav"
        with wave.open(str(tone), "wb") as output:
            output.setparams((1, 2, 48000, 0, "NONE", "not compressed"))
            output.writeframes(struct.pack("<h", 1000) * 12000)
        class Sound(Moving):
            def construct(self):
                self.add_sound(str(tone))
                super().construct()
        scene = Sound()
        result = render_scene_with_still(scene, self.root / "audio.wav", self.root / "audio.png", threads=1)
        self.assertIsNone(result.primary.frame_count)
        self.assertEqual(result.primary.sample_frames, 18000)
        self.receipt(result.primary)
        self.receipt(result.still)
        with wave.open(str(result.primary.destination), "rb") as reader:
            self.assertTrue(any(reader.readframes(reader.getnframes())))
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])

    def test_failed_subdivision_retains_only_completed_clips(self):
        original = RuntimeError("stop after one clip")
        class Failing(Moving):
            def construct(self):
                self.add(m.Square(fill_opacity=1))
                self.wait(0.25)
                raise original
        scene = Failing(file_writer_config=dict(write_to_movie=True, save_last_frame=True,
            subdivide_output=True, movie_file_extension=".y4m", output_directory=str(self.root), file_name="partial"))
        with self.assertRaises(RuntimeError) as caught:
            scene.render(threads=1)
        self.assertIs(caught.exception, original)
        partial = original.render_pair_result
        self.assertFalse(partial.completed)
        self.assertFalse(partial.primary.completed)
        self.assertEqual(len(partial.primary.segments), 1)
        self.receipt(partial.primary.segments[0].render)
        self.assertIsNone(partial.still)
        self.assertFalse((self.root / "partial.png").exists())

    def test_explicit_single_output_does_not_silently_add_a_png(self):
        scene = Moving(file_writer_config=dict(write_to_movie=True, save_last_frame=True,
            output_directory=str(self.root), file_name="unused"))
        result = scene.render(self.root / "single.y4m", threads=1)
        self.assertEqual(result.format, "y4m")
        self.assertEqual(list(self.root.iterdir()), [self.root / "single.y4m"])

    def test_paths_are_frozen_before_authored_constructor_changes_cwd(self):
        previous, other = Path.cwd(), self.root / "other"
        other.mkdir()
        class Changing(Moving):
            def __init__(self):
                super().__init__()
                os.chdir(other)
        try:
            os.chdir(self.root)
            result = render_scene_with_still(Changing, "before.y4m", "before.png", threads=1)
        finally:
            os.chdir(previous)
        self.assertEqual(result.primary.destination, self.root / "before.y4m")
        self.assertEqual(result.still.destination, self.root / "before.png")
        self.assertEqual(list(other.iterdir()), [])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(PairedOutput)
assert suite.countTestCases() == 17
if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
    raise AssertionError("native paired-output acceptance failed")
