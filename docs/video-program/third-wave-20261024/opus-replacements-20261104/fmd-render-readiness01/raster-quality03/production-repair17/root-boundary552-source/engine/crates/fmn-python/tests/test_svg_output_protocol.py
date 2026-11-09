"""Run the real output/CLI adapters; native rendering lives in svg_output.py."""
import ast
from pathlib import Path
from types import SimpleNamespace
import unittest

from fmn_python.rendering import RenderSession
from fmn_python.console_rendering import _output_overrides
from fmn_python.render_selection import select_still_format
from test_render_session_protocol import Scene, EndScene

ROOT = Path(__file__).resolve().parents[1]


class SvgOutputProtocolTests(unittest.TestCase):
    def setUp(self):
        self.native = SimpleNamespace(Scene=Scene, EndScene=EndScene)

    def test_svg_suffix_selects_native_vector_output_and_receipt(self):
        scene = Scene()
        with RenderSession(scene, 'diagram.svg', _native=self.native) as session:
            self.assertEqual(scene.request[1], 'svg')
        self.assertEqual(session.result.format, 'svg')
        self.assertIsNone(session.result.sample_frames)
        self.assertFalse(session.result.certified)
        self.assertEqual([event[0] for event in scene.events], ['begin', 'finish'])

    def test_svg_can_be_explicit_with_an_extensionless_destination(self):
        with RenderSession(Scene(), 'diagram', format='svg', _native=self.native) as session:
            self.assertEqual(session.format, 'svg')

    def test_failed_svg_generation_uses_existing_cancellation(self):
        scene = Scene()
        session = RenderSession(scene, 'diagram.svg', _native=self.native)
        with self.assertRaisesRegex(ValueError, 'authored failure'):
            with session:
                raise ValueError('authored failure')
        self.assertIsNone(session.result)
        self.assertFalse(scene.active)
        self.assertEqual(scene.events[-1][0], 'abort')

    def test_transparent_vector_output_is_routed_but_video_knobs_refuse(self):
        self.assertEqual(_output_overrides({'format': 'svg', 'transparent': True}), {'transparent': True})
        for name in ('vcodec', 'pix_fmt', 'ffmpeg_bin'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                _output_overrides({'format': 'svg', name: 'arbitrary'})

    def test_png_skip_switch_keeps_its_existing_explicit_contract(self):
        with self.assertRaisesRegex(ValueError, 'final PNG'):
            select_still_format({'format': 'svg'}, ['--format', 'svg'], True)

    def test_actual_bootstrap_parser_accepts_svg_without_relaxing_certification(self):
        source = (ROOT / 'python/manimlib_bootstrap.py').read_text()
        tree = ast.parse(source)
        function = next(node for node in tree.body if isinstance(node, ast.FunctionDef)
                        and node.name == '_portal_cli_render_arguments')
        namespace = {}
        exec(compile(ast.Module(body=[function], type_ignores=[]), '<native-parser>', 'exec'), namespace)
        parser = namespace['_portal_cli_render_arguments']
        for argv in (['--format', 'svg'], ['--format=svg']):
            parsed = parser(['scene.py', *argv, '--transparent', '--resolution', '160x90'])
            self.assertEqual(parsed[1]['format'], 'svg')
            self.assertEqual(parsed[2:4], (160, 90))
        with self.assertRaisesRegex(RuntimeError, r"certified (reproducibility|portal rendering)"):
            parser(['scene.py', '--format', 'svg', '--reproducible'])


if __name__ == '__main__':
    unittest.main()
