"""Public console multi-scene subdivision over actual native output generations."""
from pathlib import Path
import contextlib
import io
import json
import os
import sys
import tempfile
import textwrap
import unittest

from fmn_python.__main__ import main


class BatchClipConsole(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='fmn batch clip console ')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'source.py'

    def write(self, source):
        self.source.write_text('from manimlib import *\n' + textwrap.dedent(source))

    def invoke(self, *selectors, extra=(), destination='clips'):
        argv = ['fmn-python', '--robot', str(self.source), *selectors, '--subdivide',
                '--format', 'y4m', '--resolution', '96x54', '--fps', '8', '--threads', '1']
        if destination is not None:
            argv.extend(['--video_dir', str(self.root / destination)])
        argv.extend(extra)
        original = sys.argv
        original_path = list(sys.path)
        output, errors = io.StringIO(), io.StringIO()
        try:
            sys.argv = argv
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
                code = main()
        finally:
            sys.argv = original
        self.assertEqual(sys.path, original_path)
        self.assertEqual(len(output.getvalue().splitlines()), 1, output.getvalue())
        report = json.loads(output.getvalue())
        self.assertEqual(report['schema'], 'fmn-python.cli')
        self.assertEqual(report['exit']['code'], code)
        return code, report, errors.getvalue()

    def assert_clips(self, outcome, indices=(0, 1)):
        result = outcome['result']
        self.assertEqual(result['schema'], 'fmn.subdivided-render')
        self.assertEqual([segment['play_index'] for segment in result['segments']], list(indices))
        for segment in result['segments']:
            payload = Path(segment['render']['destination']).read_bytes()
            self.assertTrue(payload.startswith(b'YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2\n'))
            self.assertIn(b'FRAME\n', payload)
        return result

    def ordinary_source(self):
        self.write('''
            print("module imported")
            class First(Scene):
                def construct(self):
                    print("constructed First")
                    self.add(Square(fill_color=WHITE, fill_opacity=1))
                    self.wait(.125)
                    self.wait(.25)
            class Second(Scene):
                def construct(self):
                    print("constructed Second")
                    self.add(Square(fill_color=WHITE, fill_opacity=1).shift(RIGHT))
                    self.wait(.125)
                    self.wait(.25)
        ''')

    def test_explicit_names_keep_order_and_publish_distinct_collections(self):
        self.ordinary_source()
        code, report, errors = self.invoke('Second', 'First')
        self.assertEqual(code, 0, report)
        self.assertEqual(report['kind'], 'render-batch')
        outcomes = report['batch']['outcomes']
        self.assertEqual([item['name'] for item in outcomes], ['Second', 'First'])
        for item in outcomes:
            self.assertTrue(self.assert_clips(item)['completed'])
            self.assertEqual(Path(item['destination']), self.root / 'clips' / item['name'] / 'clips')
        self.assertLess(errors.index('constructed Second'), errors.index('constructed First'))
        self.assertEqual(errors.count('module imported'), 1)
        self.assertEqual(errors.count('constructed First'), 1)
        self.assertEqual(errors.count('constructed Second'), 1)

    def test_write_all_and_source_range_apply_to_each_locally_declared_scene(self):
        self.ordinary_source()
        code, report, _ = self.invoke('--write_all', extra=('-n', '1,2'))
        self.assertEqual(code, 0, report)
        outcomes = report['batch']['outcomes']
        self.assertEqual([item['name'] for item in outcomes], ['First', 'Second'])
        for item in outcomes:
            result = self.assert_clips(item, (1,))
            self.assertEqual(result['frame_count'], 2)

    def test_keep_going_preserves_failed_scene_clips_and_renders_later_scene(self):
        self.write('''
            class Failed(Scene):
                def construct(self):
                    self.wait(.125)
                    raise ValueError("authored failure")
            class Later(Scene):
                def construct(self):
                    self.wait(.125)
        ''')
        code, report, _ = self.invoke('--write_all', '--keep-going')
        self.assertEqual(code, 5, report)
        outcomes = report['batch']['outcomes']
        self.assertEqual([item['status'] for item in outcomes], ['failed', 'succeeded'])
        self.assertFalse(self.assert_clips(outcomes[0], (0,))['completed'])
        self.assertTrue(self.assert_clips(outcomes[1], (0,))['completed'])
        self.assertFalse(report['all_succeeded'])

    def test_fail_fast_keeps_partial_collection_without_constructing_later_scene(self):
        self.write('''
            class Failed(Scene):
                def construct(self):
                    self.wait(.125)
                    raise ValueError("failure")
            class Later(Scene):
                def __init__(self):
                    raise AssertionError("later constructor must not run")
        ''')
        code, report, errors = self.invoke('Failed', 'Later')
        self.assertEqual(code, 5, report)
        outcomes = report['batch']['outcomes']
        self.assertEqual([item['status'] for item in outcomes], ['failed', 'not_run'])
        self.assertFalse(self.assert_clips(outcomes[0], (0,))['completed'])
        self.assertNotIn('later constructor must not run', errors)
        self.assertFalse((self.root / 'clips/Later').exists())

    def test_interrupt_returns_one_partial_batch_receipt(self):
        self.write('''
            class Interrupted(Scene):
                def construct(self):
                    self.wait(.125)
                    raise KeyboardInterrupt("stop")
            class Later(Scene):
                def construct(self):
                    raise AssertionError("must not continue after interrupt")
        ''')
        code, report, errors = self.invoke('Interrupted', 'Later', '--keep-going')
        self.assertEqual(code, 130, report)
        self.assertEqual(report['kind'], 'render-batch-interrupted')
        outcomes = report['batch']['outcomes']
        self.assertEqual([item['status'] for item in outcomes], ['cancelled', 'not_run'])
        self.assertFalse(self.assert_clips(outcomes[0], (0,))['completed'])
        self.assertNotIn('must not continue', errors)

    def test_existing_batch_root_is_allowed_but_any_collection_collision_preflights_all_jobs(self):
        self.ordinary_source()
        (self.root / 'clips').mkdir()
        code, report, _ = self.invoke('First', 'Second')
        self.assertEqual(code, 0, report)
        content = (self.root / 'clips/Second/clips/segment-000000.y4m').read_bytes()
        code, report, errors = self.invoke('First', 'Second')
        self.assertEqual(code, 6, report)
        self.assertNotIn('constructed ', errors)
        self.assertEqual((self.root / 'clips/Second/clips/segment-000000.y4m').read_bytes(), content)

    def test_unknown_selection_rejects_before_any_scene_constructor(self):
        self.ordinary_source()
        code, report, errors = self.invoke('First', 'Missing')
        self.assertEqual(code, 5, report)
        self.assertNotIn('constructed ', errors)
        self.assertFalse((self.root / 'clips').exists())

    def test_unsupported_modes_refuse_before_source_import(self):
        self.write('raise AssertionError("module must not execute")')
        for extra in (('-s',), ('--format', 'png'), ('--reproducible',),
                      ('--checkpoint', str(self.root / 'state.json'), '--resume-key', 'v1')):
            with self.subTest(extra=extra):
                code, report, errors = self.invoke('First', 'Second', extra=extra)
                self.assertIn(code, (2, 4), report)
                self.assertNotIn('module must not execute', errors)
                self.assertFalse((self.root / 'clips').exists())

    def test_default_batch_root_and_single_scene_envelope_remain_distinct(self):
        self.ordinary_source()
        original = os.getcwd()
        try:
            os.chdir(self.root)
            code, report, _ = self.invoke('First', 'Second', destination=None)
        finally:
            os.chdir(original)
        self.assertEqual(code, 0, report)
        self.assertEqual(Path(report['destination']), self.root / 'media/videos/source')
        code, report, _ = self.invoke('First', destination='single')
        self.assertEqual(code, 0, report)
        self.assertEqual(report['kind'], 'render-subdivided')
        self.assertTrue(report['subdivision']['completed'])
        self.assertEqual(Path(report['destination']), self.root / 'single')


suite = unittest.defaultTestLoader.loadTestsFromTestCase(BatchClipConsole)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), 'native subdivided batch console acceptance failed'
