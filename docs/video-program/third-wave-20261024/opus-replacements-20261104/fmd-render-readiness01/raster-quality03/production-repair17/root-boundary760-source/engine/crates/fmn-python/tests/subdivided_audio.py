"""Actual native scene-audio windows, WAV samples, muxed video and CLI receipts."""
from pathlib import Path
import contextlib
import hashlib
import io
import json
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import wave

import numpy as np
import manimlib as m
from fmn_python import (record_scene, record_subdivided_scene, render_session,
                        render_subdivided_scene, subdivided_render_session)
from fmn_python.__main__ import main


def write_wave(path, rate=48000, count=48000, channels=1, scale=3000):
    values = ((np.arange(count * channels) % 97) - 48) * scale // 48
    values = values.astype('<i2').reshape(-1, channels)
    with wave.open(str(path), 'wb') as stream:
        stream.setparams((channels, 2, rate, 0, 'NONE', 'not compressed'))
        stream.writeframes(values.tobytes())
    return values


def pcm(path):
    with wave.open(str(path), 'rb') as stream:
        assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth()) == (48000, 2, 2)
        return np.frombuffer(stream.readframes(stream.getnframes()), dtype='<i2').reshape(-1, 2)


def sample_boundary(frame, fps):
    return (2 * frame * 48000 + fps) // (2 * fps)


class AudioClips(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn audio subdivision ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'background.wav'
        self.samples = write_wave(self.source)

    def clips(self, scene, name='clips', format='wav', threads=1):
        return record_subdivided_scene(scene, self.root / name, format=format,
                                       resolution=(96, 54), threads=threads)

    def assert_receipt(self, result):
        data = result.destination.read_bytes()
        self.assertEqual(result.bytes, len(data))
        self.assertEqual(result.digest, hashlib.sha256(data).hexdigest())
        self.assertFalse(result.certified)

    def test_background_cue_continues_at_its_original_phase_in_every_clip(self):
        scene = m.Scene()
        scene.add_sound(str(self.source))
        scene.wait(.125)
        with self.clips(scene) as run:
            scene.wait(.125)
            scene.wait(.125)
        for segment in run.segments:
            self.assert_receipt(segment.render)
            start = sample_boundary(segment.start_frame, 30)
            end = sample_boundary(segment.end_frame, 30)
            expected = np.repeat(self.samples[start:end], 2, axis=1)
            np.testing.assert_array_equal(pcm(segment.render.destination), expected)
            self.assertEqual(segment.render.sample_frames, end - start)
            self.assertEqual(len(segment.render.audio_inputs), 1)
            self.assertEqual(segment.render.ffmpeg_invocations, ())
        self.assertEqual(run.result.sample_frames, 12800)
        self.assertEqual(m._portal_scene_clock(scene), (30, 12))

    def test_rational_partitions_resampling_and_ducking_match_whole_native_mix(self):
        foreground = self.root / 'foreground.wav'
        write_wave(foreground, rate=44100, count=20000, channels=2, scale=1000)
        def cues(scene):
            scene.add_sound(str(self.source), gain=-3)
            scene.add_sound(str(foreground), time_offset=.1, gain=4, gain_to_background=-6)
        scene = m.Scene()
        with render_session(scene, self.root / 'whole.wav', fps=29, threads=1):
            cues(scene)
            for _ in range(4):
                scene.wait(.125)
        expected = pcm(self.root / 'whole.wav')[:sample_boundary(16, 29)]
        for threads in (1, 4):
            scene = m.Scene()
            with subdivided_render_session(scene, self.root / f'parts-{threads}', format='wav',
                                            resolution=(96, 54), fps=29, threads=threads) as run:
                cues(scene)
                for _ in range(4):
                    scene.wait(.125)
            joined = np.concatenate([pcm(s.render.destination) for s in run.segments])
            np.testing.assert_array_equal(joined, expected)
            self.assertEqual(run.result.sample_frames, sample_boundary(16, 29))
            self.assertGreater(len(set(s.render.sample_frames for s in run.segments)), 1)
            self.assertTrue(all(len(s.render.audio_inputs) == 2 for s in run.segments))

    def test_no_cues_produce_bounded_silence_instead_of_missing_wav(self):
        scene = m.Scene(file_writer_config={'ffmpeg_bin': str(self.root / 'absent')})
        with self.clips(scene) as run:
            scene.wait(.125)
            scene.wait(.125)
        self.assertEqual(run.result.sample_frames, 12800)
        for segment in run.segments:
            self.assertEqual(pcm(segment.render.destination).shape, (6400, 2))
            self.assertFalse(pcm(segment.render.destination).any())
            self.assertEqual(segment.render.audio_inputs, ())
            self.assertEqual(segment.render.ffmpeg_invocations, ())

    def test_long_cue_tail_never_extends_clip_duration(self):
        scene = m.Scene()
        scene.add_sound(str(self.source), time_offset=.1)
        with self.clips(scene) as run:
            scene.wait(.125)
        samples = pcm(run.segments[0].render.destination)
        self.assertEqual(len(samples), 6400)
        self.assertFalse(samples[:4800].any())
        self.assertTrue(samples[4800:].any())

    def test_negative_offset_uses_native_source_clipping_not_restart(self):
        scene = m.Scene()
        scene.add_sound(str(self.source), time_offset=-.125)
        with self.clips(scene) as run:
            scene.wait(.125)
        np.testing.assert_array_equal(pcm(run.segments[0].render.destination),
                                      np.repeat(self.samples[6000:12400], 2, axis=1))

    def test_zero_duration_wav_is_zero_samples_not_a_whole_sound(self):
        scene = m.Scene()
        scene.add_sound(str(self.source))
        with self.clips(scene) as run:
            scene.wait(0)
        self.assertEqual(run.result.sample_frames, 0)
        self.assertEqual(pcm(run.segments[0].render.destination).shape, (0, 2))

    def test_skipped_calls_do_not_publish_or_restart_background(self):
        scene = m.Scene()
        scene.add_sound(str(self.source))
        with self.clips(scene) as run:
            scene.wait(.125)
            with scene.temp_skip():
                scene.wait(.125)
            scene.wait(.125)
        self.assertEqual([s.play_index for s in run.segments], [0, 2])
        np.testing.assert_array_equal(pcm(run.segments[1].render.destination),
                                      np.repeat(self.samples[12800:19200], 2, axis=1))

    def test_missing_later_cue_preserves_earlier_completed_clip(self):
        scene = m.Scene()
        scene.add_sound(str(self.source))
        run = self.clips(scene)
        with self.assertRaises(OSError):
            with run:
                scene.wait(.125)
                scene.add_sound(str(self.root / 'missing.wav'))
                scene.wait(.125)
        self.assertIsNone(run.result)
        self.assertEqual(len(run.partial_result.segments), 1)
        self.assert_receipt(run.segments[0].render)
        self.assertFalse((run.destination / 'segment-000001.wav').exists())
        self.assertNotIn('_fmn_owned_render_session', vars(scene))
        self.assertNotIn('_fmn_subdivision_session', vars(scene))

    def test_ordinary_live_recording_still_excludes_preexisting_cues(self):
        scene = m.Scene()
        scene.add_sound(str(self.root / 'unrelated-missing.wav'))
        scene.wait(.125)
        with record_scene(scene, self.root / 'insert.wav', threads=1) as run:
            scene.add_sound(str(self.source))
            scene.wait(.125)
        self.assertEqual(len(run.result.audio_inputs), 1)
        # Ordinary inserts retain their established extend-to-cue-end behavior.
        self.assertEqual(run.result.sample_frames, 48000)

    def test_source_changes_affect_only_later_publications_and_are_receipted(self):
        scene = m.Scene()
        scene.add_sound(str(self.source))
        with self.clips(scene) as run:
            scene.wait(.125)
            first_digest = run.segments[0].render.digest
            first_source = hashlib.sha256(self.source.read_bytes()).hexdigest()
            changed = write_wave(self.source, scale=7000)
            scene.wait(.125)
        self.assert_receipt(run.segments[0].render)
        self.assertEqual(run.segments[0].render.digest, first_digest)
        self.assertEqual(run.segments[0].render.audio_inputs[0]['source_sha256'], first_source)
        self.assertNotEqual(run.segments[1].render.audio_inputs[0]['source_sha256'], first_source)
        np.testing.assert_array_equal(pcm(run.segments[1].render.destination),
                                      np.repeat(changed[6400:12800], 2, axis=1))

    def test_mp4_and_alpha_mov_have_real_motion_audio_and_governed_mux_receipts(self):
        executable = shutil.which('ffmpeg')
        if not executable:
            self.assertNotEqual(os.environ.get('FMN_REQUIRE_FFMPEG'), '1', 'ffmpeg required')
            self.skipTest('ffmpeg unavailable')
        for format in ('mp4', 'mov'):
            with self.subTest(format=format):
                scene = m.Scene()
                if format == 'mov':
                    scene.camera.background_rgba[3] = 0
                square = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
                square.shift(2 * m.LEFT)
                scene.add(square)
                scene.add_sound(str(self.source))
                with self.clips(scene, format, format=format) as run:
                    for _ in range(2):
                        scene.play(square.animate.shift(2 * m.RIGHT), run_time=.125, rate_func=m.linear)
                for segment in run.segments:
                    receipt = segment.render
                    self.assert_receipt(receipt)
                    self.assertEqual(receipt.frame_count, 4)
                    self.assertEqual(len(receipt.audio_inputs), 1)
                    self.assertGreaterEqual(len(receipt.ffmpeg_invocations), 2)
                    self.assertTrue(any('-c:a' in call['argv'] for call in receipt.ffmpeg_invocations))
                    video = subprocess.run([executable, '-v', 'error', '-nostdin', '-i', str(receipt.destination),
                                             '-an', '-pix_fmt', 'rgba', '-f', 'rawvideo', 'pipe:1'],
                                            check=True, capture_output=True, timeout=30)
                    frames = np.frombuffer(video.stdout, dtype=np.uint8).reshape(-1, 54, 96, 4)
                    self.assertEqual(len(frames), 4)
                    self.assertTrue(np.all(frames[:, 0, 0, 3] == (0 if format == 'mov' else 255)))
                    centers = [np.nonzero(frame[:, :, 0] > 220)[1].mean() for frame in frames]
                    self.assertTrue(all(b > a for a, b in zip(centers, centers[1:])), centers)
                    audio = subprocess.run([executable, '-v', 'error', '-nostdin', '-i', str(receipt.destination),
                                             '-vn', '-f', 's16le', '-ac', '2', '-ar', '48000', 'pipe:1'],
                                            check=True, capture_output=True, timeout=30)
                    samples = np.frombuffer(audio.stdout, dtype='<i2')
                    self.assertGreaterEqual(len(samples), 6400 * 2)
                    self.assertGreater(np.abs(samples.astype(np.int32)).mean(), 100)

    def test_missing_encoder_refuses_before_advancing_scene(self):
        scene = m.Scene(file_writer_config={'ffmpeg_bin': str(self.root / 'absent')})
        run = self.clips(scene, format='mp4')
        with self.assertRaises((m._CapabilityError, RuntimeError)):
            with run:
                scene.wait(.125)
        self.assertEqual(m._portal_scene_clock(scene), (30, 0))
        self.assertFalse(run.artifact_published)

    def test_console_wav_returns_one_machine_receipt_with_sample_counts(self):
        source = self.root / 'source.py'
        source.write_text('from manimlib import *\nclass Motion(Scene):\n    def construct(self):\n'
                          f'        self.add_sound({str(self.source)!r})\n'
                          '        self.wait(.125)\n        self.wait(.125)\n')
        original = sys.argv
        output, errors = io.StringIO(), io.StringIO()
        try:
            sys.argv = ['fmn-python', '--robot', str(source), 'Motion', '--subdivide', '--format', 'wav',
                        '--fps', '30', '--threads', '1', '--video_dir', str(self.root / 'cli')]
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                code = main()
        finally:
            sys.argv = original
        self.assertEqual(code, 0, errors.getvalue() + output.getvalue())
        self.assertEqual(len(output.getvalue().splitlines()), 1)
        result = json.loads(output.getvalue())
        self.assertEqual(result['kind'], 'render-subdivided')
        self.assertEqual(result['subdivision']['sample_frames'], 12800)
        self.assertEqual([s['render']['sample_frames'] for s in result['subdivision']['segments']], [6400, 6400])


assert callable(getattr(m, '_portal_begin_audio_clip', None)), 'matching native audio-window wheel required'
suite = unittest.defaultTestLoader.loadTestsFromTestCase(AudioClips)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), 'native audio subdivision acceptance failed'
