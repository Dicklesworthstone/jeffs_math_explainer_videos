"""Installed portal console -> actual native vector documents and receipts.

Only argv and output streams are scoped. No parser, scene, renderer or sink is
substituted; generated scene modules execute through the normal source owner.
"""
from __future__ import annotations

import contextlib
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import xml.etree.ElementTree as ET

from fmn_python.__main__ import main


def invoke(source, destination, *names, extra=()):
    arguments = ['fmn-python', '--robot', str(source), *names, '--format', 'svg',
                 '--resolution', '160x90', '--fps', '8', '--threads', '1',
                 '--video_dir', str(destination), *extra]
    stdout, stderr = io.StringIO(), io.StringIO()
    old_arguments, old_path = sys.argv, list(sys.path)
    try:
        sys.argv = arguments
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = main()
    finally:
        sys.argv = old_arguments
    assert sys.path == old_path, 'CLI leaked its project import path'
    lines = stdout.getvalue().splitlines()
    assert len(lines) == 1, stdout.getvalue()
    report = json.loads(lines[0])
    assert report['schema'] == 'fmn-python.cli' and report['version'] == 1
    assert report['exit']['code'] == code
    return code, report, stderr.getvalue()


def document(path):
    data = path.read_bytes()
    tree = ET.fromstring(data)
    assert tree.tag == '{http://www.w3.org/2000/svg}svg'
    assert float(tree.attrib['width']) == 160 and float(tree.attrib['height']) == 90
    assert tree.findall('{http://www.w3.org/2000/svg}path')
    assert not tree.findall('.//{http://www.w3.org/2000/svg}image')
    assert not tree.findall('.//{http://www.w3.org/2000/svg}text')
    return data


with tempfile.TemporaryDirectory(prefix='fmn-svg-console-') as temp:
    root = Path(temp)
    source = root / 'lesson.py'
    source.write_text('''from manimlib import Scene, Square, Circle, Text, RIGHT, UP
print('authored import chatter')
class Diagram(Scene):
    def construct(self):
        print('authored construct chatter')
        self.add(Text('Vector').shift(UP))
        box = Square(fill_opacity=1)
        self.add(box)
        self.play(box.animate.shift(RIGHT), run_time=0.25)
class Curves(Scene):
    def construct(self):
        self.add(Circle(fill_opacity=1))
''')
    destination = root / 'single.svg'
    code, report, chatter = invoke(source, destination, 'Diagram')
    assert code == 0, report
    data = document(destination)
    assert report['format'] == 'svg' and report['engine'] == 'native-svg'
    assert report['frame_count'] == 1 and report['certified'] is False
    assert report['bytes'] == len(data) and report['digest'] == hashlib.sha256(data).hexdigest()
    assert 'authored import chatter' in chatter and 'authored construct chatter' in chatter

    transparent = root / 'transparent.svg'
    code, report, _ = invoke(source, transparent, 'Curves', extra=('--transparent',))
    assert code == 0, report
    tree = ET.fromstring(document(transparent))
    assert len(tree.findall('{http://www.w3.org/2000/svg}path')) == 1, 'background was not transparent'

    batch = root / 'batch'
    code, report, _ = invoke(source, batch, 'Curves', 'Diagram')
    assert code == 0, report
    document(batch / 'Curves.svg')
    document(batch / 'Diagram.svg')
    assert sorted(path.name for path in batch.iterdir()) == ['Curves.svg', 'Diagram.svg']

    all_output = root / 'all'
    code, report, _ = invoke(source, all_output, extra=('--write_all',))
    assert code == 0, report
    assert sorted(path.name for path in all_output.iterdir()) == ['Curves.svg', 'Diagram.svg']

    invalid_batch = root / 'invalid-batch'
    code, report, _ = invoke(source, invalid_batch, 'Diagram', 'Missing')
    assert code != 0 and not invalid_batch.exists(), report

    code, report, _ = invoke(source, destination, 'Diagram')
    assert code != 0 and destination.read_bytes() == data, report
    for options in (('--vcodec', 'libx264'), ('--pix_fmt', 'rgba'), ('--reproducible',)):
        refused = root / 'refused.svg'
        code, report, _ = invoke(source, refused, 'Diagram', extra=options)
        assert code in (2, 4) and not refused.exists(), report

    failure_source = root / 'failure.py'
    failure_source.write_text('''from manimlib import Scene, Circle
class Broken(Scene):
    def construct(self):
        self.add(Circle())
        raise RuntimeError('vector execution deliberately fails')
''')
    failure = root / 'failed.svg'
    code, report, _ = invoke(failure_source, failure, 'Broken')
    assert code == 5 and not failure.exists(), report
    assert not report['artifact_published'], report

print('native SVG console acceptance passed')
