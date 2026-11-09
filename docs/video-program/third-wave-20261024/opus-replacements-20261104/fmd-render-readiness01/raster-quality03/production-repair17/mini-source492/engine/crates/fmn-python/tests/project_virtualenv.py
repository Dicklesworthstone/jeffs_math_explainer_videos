"""Installed native portal in a project-local venv, with no network or pip.

Copy the executing installation into an isolated stdlib-created environment.
The child runs the real CLI, SceneSource and autoreload namespace against real
NumPy/native libraries. No import, source-loader or renderer is substituted.
"""
from __future__ import annotations

import contextlib
import hashlib
import importlib.metadata
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
import venv


def child(root):
    import manimlib as m
    import numpy as np
    from fmn_python.__main__ import main
    from fmn_python.scene_loading import SceneSource
    from fmn_python.source_autoreload import SourceNamespace
    from fmn_python.project_watch import _snapshot

    root = Path(root).resolve()
    assert Path(sys.prefix).is_relative_to(root), sys.prefix
    assert Path(np.__file__).resolve().is_relative_to(root), np.__file__
    assert Path(m.__file__).resolve().is_relative_to(root), m.__file__
    core = sys.modules['numpy._core._multiarray_umath']
    helper = root/'venv_helper.py'
    helper.write_text('POSITION=1\n')
    timestamp = helper.stat().st_mtime_ns
    source_path = root/'scene.py'
    source_path.write_text('''from manimlib import *
import manimlib as m
import numpy as np
assert np is m._np, "scene reload replaced the engine NumPy module"
class VenvScene(Scene):
    def construct(self):
        from venv_helper import POSITION
        from numpy.random import default_rng
        assert default_rng(1).random() >= 0
        square = Square(side_length=1, fill_opacity=1, stroke_width=0)
        square.shift(POSITION * RIGHT)
        self.add(square, Text("venv").shift(UP))
        self.play(square.animate.shift(RIGHT / 2), run_time=.25, rate_func=linear)
''')

    def render(name, threads):
        output = root/name
        argv = sys.argv
        stdout, stderr = io.StringIO(), io.StringIO()
        try:
            sys.argv = ['fmn-python', '--robot', str(source_path), 'VenvScene',
                        '--format', 'png_sequence', '--resolution', '96x54',
                        '--fps', '8', '--threads', str(threads), '--video_dir', str(output)]
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                code = main()
        finally:
            sys.argv = argv
        assert code == 0, (code, stdout.getvalue(), stderr.getvalue())
        report = json.loads(stdout.getvalue())
        assert report['frame_count'] == 2 and report['exit']['code'] == 0, report
        frames = [path.read_bytes() for path in sorted(output.glob('*.png'))]
        assert len(frames) == 2 and frames[0] != frames[1], 'requires changing frames'
        assert sys.modules['numpy'] is np
        assert sys.modules['numpy._core._multiarray_umath'] is core
        assert 'reloaded' not in stderr.getvalue(), stderr.getvalue()
        return frames

    first = render('one', 1)
    assert render('four', 4) == first, 'thread count changed output'
    helper.write_text('POSITION=2\n')
    os.utime(helper, ns=(timestamp, timestamp))
    changed = render('edited', 1)
    assert changed != first, 'same-length helper edit was not visible'

    # Exercise the definition-reload path shared with embedded source autoreload.
    reload_path = root/'reload_scene.py'
    reload_path.write_text('import numpy as np\nfrom venv_helper import POSITION\n')
    with SceneSource(reload_path, m.Scene) as source:
        shell = {'POSITION': source.module.POSITION}
        bindings = SourceNamespace(source, shell)
        assert not source._owns_source(Path(np.__file__))
        assert not source._owns_source(Path(core.__file__))
        assert set(source.sources) == {'reload_scene.py', 'venv_helper.py'}
        watched = dict(_snapshot(SimpleNamespace(_source=source), (Path(np.__file__),)))
        assert all('.venv' not in path.parts for path in watched), watched
        helper.write_text('POSITION=3\n')
        os.utime(helper, ns=(timestamp, timestamp))
        bindings.refresh()
        assert shell['POSITION'] == 3 and source.module.np is np
        previous = source.module
        helper.write_text('raise RuntimeError("bad helper")\n')
        try:
            bindings.refresh()
        except RuntimeError as error:
            assert str(error) == 'bad helper'
        else:
            raise AssertionError('failed reload was accepted')
        assert source.module is previous and shell['POSITION'] == 3
        assert sys.modules['numpy'] is np
        assert sys.modules['numpy._core._multiarray_umath'] is core

    print(json.dumps({'case': 'project-local-virtualenv', 'exit_code': 0,
                      'interpreter': sys.executable, 'venv': sys.prefix,
                      'numpy_path': np.__file__, 'numpy_owned_by_scene': False,
                      'frame_sha256': [hashlib.sha256(frame).hexdigest() for frame in first],
                      'edited_sha256': [hashlib.sha256(frame).hexdigest() for frame in changed],
                      'autoreload_position': shell['POSITION']}, sort_keys=True))


class ProjectVirtualenv(unittest.TestCase):
    def test_real_nested_environment_render_and_autoreload(self):
        import manimlib as m
        import numpy as np
        import fmn_python
        self.assertTrue(getattr(m, '__franken_manim__', False), 'requires installed native portal')
        with tempfile.TemporaryDirectory(prefix='fmn-project-venv-') as directory:
            root = Path(directory).resolve()
            environment = root/'.venv'
            # Symlink like `python -m venv` and `uv venv` on POSIX. The class
            # defaults to copies, and a copied python-build-standalone (uv)
            # interpreter cannot find its $ORIGIN/../lib/libpython (exit 127).
            builder = venv.EnvBuilder(with_pip=False, symlinks=os.name != 'nt')
            builder.create(environment)
            context = builder.ensure_directories(environment)
            python = Path(context.env_exe)
            result = subprocess.run([str(python), '-c',
                'import sysconfig; print(sysconfig.get_path("purelib"))'],
                capture_output=True, text=True, check=True, timeout=30)
            target = Path(result.stdout.strip())
            # Copy package trees (including native libraries and bundled data),
            # plus installed metadata. Do not run a package manager or download.
            packages = (Path(fmn_python.__file__).parent, Path(m.__file__).parent,
                        Path(np.__file__).parent)
            for package in packages:
                shutil.copytree(package, target/package.name,
                                ignore=shutil.ignore_patterns('__pycache__'))
            for name in ('franken-manim', 'numpy'):
                distribution = importlib.metadata.distribution(name)
                directories = {Path(str(file)).parts[0] for file in distribution.files or ()
                               if Path(str(file)).parts and (Path(str(file)).parts[0].endswith('.dist-info')
                                                           or Path(str(file)).parts[0].endswith('.libs'))}
                for name in sorted(directories):
                    directory = Path(distribution.locate_file(name))
                    if not (target/name).exists():
                        shutil.copytree(directory, target/name)
            env = dict(os.environ)
            env.pop('PYTHONPATH', None)
            env.pop('PYTHONHOME', None)
            result = subprocess.run([str(python), '-I', str(Path(__file__).resolve()),
                                     '--child', str(root)], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=90)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            receipt = json.loads(result.stdout)
            self.assertFalse(receipt['numpy_owned_by_scene'])
            self.assertEqual(receipt['autoreload_position'], 3)
            print(json.dumps(receipt, sort_keys=True))


if __name__ == '__main__':
    if len(sys.argv) == 3 and sys.argv[1] == '--child':
        child(sys.argv[2])
    else:
        unittest.main()
