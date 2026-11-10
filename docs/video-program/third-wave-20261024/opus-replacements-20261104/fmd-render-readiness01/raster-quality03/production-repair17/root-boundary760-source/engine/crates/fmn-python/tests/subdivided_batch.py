"""Native multi-scene clip collections, partial output, and failure ownership."""
from pathlib import Path
import hashlib
import json
import os
import tempfile
import traceback
import unittest

import manimlib as m
from fmn_python import BatchRenderError, RenderJob, render_scenes, render_subdivided_scene


def payloads(path):
    header, data = path.read_bytes().split(b'\n', 1)
    assert header == b'YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2', header
    size = 96 * 54 * 3 // 2
    assert len(data) % (size + 6) == 0
    frames = []
    for start in range(0, len(data), size + 6):
        assert data[start:start + 6] == b'FRAME\n'
        frames.append(data[start + 6:start + 6 + size])
    return frames


def clear_error(error):
    seen = set()
    while error is not None and id(error) not in seen:
        seen.add(id(error))
        if error.__traceback__ is not None:
            traceback.clear_frames(error.__traceback__)
            error.__traceback__ = None
        error = error.__cause__


class SubdividedBatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn subdivided batch ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.calls = []
        calls = self.calls

        class Motion(m.Scene):
            def __init__(self, sign=1, label='motion', **kwargs):
                calls.append(('init', label))
                super().__init__(**kwargs)
                self.sign, self.label = sign, label
            def setup(self):
                calls.append(('setup', self.label))
            def construct(self):
                calls.append(('construct', self.label))
                square = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
                square.shift(-2 * self.sign * m.RIGHT)
                self.add(square)
                self.wait(.125)
                self.play(square.animate.shift(4 * self.sign * m.RIGHT), run_time=.25, rate_func=m.linear)
            def tear_down(self):
                calls.append(('tear_down', self.label))

        self.Motion = Motion

    def run_batch(self, scenes, directory='batch', **options):
        settings = dict(format='y4m', resolution=(96, 54), fps=8, threads=1, subdivide=True)
        settings.update(options)
        return render_scenes(scenes, self.root / directory, **settings)

    def assert_collection(self, outcome, count):
        result = outcome.result
        self.assertEqual(result.destination, outcome.destination)
        self.assertEqual(len(result.segments), count)
        self.assertEqual(outcome.destination.name, 'clips')
        for segment in result.segments:
            data = segment.render.destination.read_bytes()
            self.assertEqual(segment.render.bytes, len(data))
            self.assertEqual(segment.render.digest, hashlib.sha256(data).hexdigest())
            self.assertFalse(segment.render.certified)
        return result

    def test_named_jobs_keep_order_kwargs_lifecycles_and_native_motion(self):
        jobs = [RenderJob('rightward', self.Motion, {'sign': 1, 'label': 'right'}),
                RenderJob('leftward', self.Motion, {'sign': -1, 'label': 'left'})]
        report = self.run_batch(jobs)
        self.assertTrue(report.ok)
        self.assertFalse(report.all_scenes_certified)
        self.assertEqual([outcome.name for outcome in report.outcomes], ['rightward', 'leftward'])
        for sign, outcome in zip((1, -1), report.outcomes):
            result = self.assert_collection(outcome, 2)
            self.assertTrue(result.completed)
            self.assertEqual(result.frame_count, 3)
            self.assertEqual([segment.play_index for segment in result.segments], [0, 1])
            frames = [frame for segment in result.segments for frame in payloads(segment.render.destination)]
            centers = []
            for frame in frames:
                positions = [index % 96 for index, value in enumerate(frame[:96 * 54]) if value > 200]
                self.assertGreater(len(positions), 10)
                centers.append(sum(positions) / len(positions))
            self.assertTrue(all(sign * (b - a) > 5 for a, b in zip(centers, centers[1:])), centers)
        self.assertEqual(self.calls, [(phase, name) for name in ('right', 'left')
                                     for phase in ('init', 'setup', 'construct', 'tear_down')])
        decoded = json.loads(json.dumps(report.as_dict()))
        self.assertEqual(decoded['outcomes'][0]['result']['schema'], 'fmn.subdivided-render')

    def test_one_and_four_threads_publish_identical_clips(self):
        first = self.run_batch({'motion': self.Motion}, 'one')
        other = self.run_batch({'motion': self.Motion}, 'four', threads=4)
        for left, right in zip(first.outcomes[0].result.segments, other.outcomes[0].result.segments):
            self.assertEqual(left.render.destination.read_bytes(), right.render.destination.read_bytes())
            self.assertEqual(left.render.digest, right.render.digest)

    def test_fail_fast_preserves_completed_scene_and_partial_failing_scene(self):
        class Failed(m.Scene):
            def construct(self):
                self.wait(.125)
                raise ValueError('authored scene failure')
        try:
            self.run_batch({'first': self.Motion, 'bad': Failed, 'later': self.Motion})
        except BatchRenderError as error:
            report = error.result
            self.assertIsInstance(error.__cause__, ValueError)
            clear_error(error)
        else:
            self.fail('failed scene was accepted')
        self.assertEqual([outcome.status for outcome in report.outcomes], ['succeeded', 'failed', 'not_run'])
        self.assert_collection(report.outcomes[0], 2)
        partial = self.assert_collection(report.outcomes[1], 1)
        self.assertFalse(partial.completed)
        self.assertIsNone(report.outcomes[2].result)
        self.assertFalse((self.root / 'batch/later').exists())
        self.assertEqual(self.calls.count(('construct', 'motion')), 1)

    def test_continue_on_error_runs_later_scene_and_keeps_partial_receipts(self):
        class Failed(m.Scene):
            def construct(self):
                self.wait(.125)
                raise ValueError('keep going')
        observations = []
        report = self.run_batch({'bad': Failed, 'good': self.Motion}, continue_on_error=True,
                                on_result=lambda outcome: observations.append(outcome.status))
        self.assertFalse(report.ok)
        self.assertEqual(observations, ['failed', 'succeeded'])
        self.assertEqual(report.counts, dict(succeeded=1, failed=1, cancelled=0, not_run=0))
        self.assertFalse(self.assert_collection(report.outcomes[0], 1).completed)
        self.assertTrue(self.assert_collection(report.outcomes[1], 2).completed)

    def test_interrupt_is_never_swallowed_and_retains_partial_current_scene(self):
        class Interrupted(m.Scene):
            def construct(self):
                self.wait(.125)
                raise KeyboardInterrupt('stop now')
        try:
            self.run_batch({'bad': Interrupted, 'later': self.Motion}, continue_on_error=True)
        except KeyboardInterrupt as error:
            report = error.render_batch_result
            clear_error(error)
        else:
            self.fail('interrupt was swallowed')
        self.assertEqual([outcome.status for outcome in report.outcomes], ['cancelled', 'not_run'])
        self.assertFalse(self.assert_collection(report.outcomes[0], 1).completed)
        self.assertEqual(self.calls, [])

    def test_observer_failure_does_not_relabel_published_scene(self):
        def observe(outcome):
            self.assertEqual(outcome.status, 'succeeded')
            raise ValueError('observer failed')
        try:
            self.run_batch({'first': self.Motion, 'later': self.Motion}, on_result=observe)
        except ValueError as error:
            self.assertEqual(str(error), 'observer failed')
            report = error.render_batch_result
            clear_error(error)
        else:
            self.fail('observer error was swallowed')
        self.assertEqual([outcome.status for outcome in report.outcomes], ['succeeded', 'not_run'])
        self.assertTrue(self.assert_collection(report.outcomes[0], 2).completed)

    def test_all_job_and_destination_checks_precede_scene_construction(self):
        with self.assertRaises(TypeError):
            self.run_batch({'valid': self.Motion, 'invalid': object()})
        with self.assertRaises(ValueError):
            self.run_batch([RenderJob('SAME', self.Motion), RenderJob('same', self.Motion)])
        occupied = self.root / 'batch/second/clips'
        occupied.mkdir(parents=True)
        sentinel = occupied / 'keep'
        sentinel.write_bytes(b'existing output')
        with self.assertRaises(FileExistsError):
            self.run_batch({'first': self.Motion, 'second': self.Motion})
        self.assertEqual(sentinel.read_bytes(), b'existing output')
        self.assertEqual(self.calls, [])
        self.assertFalse((self.root / 'batch/first').exists())

    def test_still_certification_checkpoint_and_budget_modes_fail_before_construction(self):
        for options in ({'format': 'png'}, {'format': 'svg'}, {'reproducible': True},
                        {'checkpoint': self.root / 'checkpoint.json', 'resume_key': 'v1'},
                        {'subdivide': 1}, {'max_segments': 0}, {'max_segments': True},
                        {'max_jobs': 1}):
            with self.subTest(options=options):
                with self.assertRaises((ValueError, TypeError, RuntimeError)):
                    self.run_batch({'first': self.Motion, 'second': self.Motion}, **options)
        self.assertEqual(self.calls, [])
        self.assertEqual(list(self.root.iterdir()), [])

    def test_source_selection_applies_independently_to_each_scene(self):
        report = self.run_batch({'first': self.Motion, 'second': self.Motion}, animation_range=(1, 2))
        self.assertTrue(report.ok)
        for outcome in report.outcomes:
            result = self.assert_collection(outcome, 1)
            self.assertEqual(result.segments[0].play_index, 1)
            self.assertEqual(result.frame_count, 2)

    def test_clip_budget_failure_is_local_to_one_scene(self):
        class One(m.Scene):
            def construct(self):
                self.wait(.125)
        report = self.run_batch({'two': self.Motion, 'one': One}, max_segments=1, continue_on_error=True)
        self.assertEqual([outcome.status for outcome in report.outcomes], ['failed', 'succeeded'])
        self.assertFalse(self.assert_collection(report.outcomes[0], 1).completed)
        self.assertTrue(self.assert_collection(report.outcomes[1], 1).completed)

    def test_constructor_failure_has_no_invented_collection_receipt(self):
        class Failed(m.Scene):
            def __init__(self):
                raise ValueError('constructor failed')
        report = self.run_batch({'bad': Failed, 'good': self.Motion}, continue_on_error=True)
        self.assertIsNone(report.outcomes[0].result)
        self.assertFalse(report.outcomes[0].destination.exists())
        self.assert_collection(report.outcomes[1], 2)

    def test_direct_single_scene_error_carries_partial_collection(self):
        class Failed(m.Scene):
            def construct(self):
                self.wait(.125)
                raise ValueError('direct scene failure')
        try:
            render_subdivided_scene(Failed, self.root / 'direct', format='y4m', resolution=(96, 54), fps=8, threads=1)
        except ValueError as error:
            result = error.render_subdivision_result
            clear_error(error)
        else:
            self.fail('direct scene failure was swallowed')
        self.assertFalse(result.completed)
        self.assertEqual(result.frame_count, 1)
        self.assertEqual(len(payloads(result.segments[0].render.destination)), 1)

    def test_cwd_changes_cannot_redirect_later_scene_destinations(self):
        changed = self.root / 'new cwd'
        changed.mkdir()
        class ChangesCwd(m.Scene):
            def construct(self):
                os.chdir(changed)
                self.wait(.125)
        original = os.getcwd()
        try:
            report = self.run_batch({'changing': ChangesCwd, 'later': self.Motion})
        finally:
            os.chdir(original)
        self.assertTrue(report.ok)
        self.assertEqual([outcome.destination for outcome in report.outcomes],
                         [self.root / 'batch' / name / 'clips' for name in ('changing', 'later')])
        self.assertEqual(list(changed.iterdir()), [])

    def test_ordinary_batch_output_remains_a_single_artifact(self):
        report = self.run_batch({'motion': self.Motion}, subdivide=False)
        self.assertEqual(report.outcomes[0].destination, self.root / 'batch/motion.y4m')
        self.assertEqual(len(payloads(report.outcomes[0].destination)), 3)
        self.assertEqual(report.outcomes[0].result.frame_count, 3)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(SubdividedBatchTests)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), 'native subdivided batch acceptance failed'
