"""Actual native SVG output: geometry, lifecycle, view, and publication failure."""
from pathlib import Path
import hashlib
import tempfile
import xml.etree.ElementTree as ET

import manimlib as m
from fmn_python import render_scenes

NS = {'s': 'http://www.w3.org/2000/svg'}


def paths(path):
    root = ET.fromstring(path.read_bytes())
    assert root.tag == '{http://www.w3.org/2000/svg}svg'
    assert not root.findall('.//s:image', NS), 'vector export embedded raster pixels'
    assert not root.findall('.//s:text', NS), 'text must retain native outlines, not host fonts'
    return root, root.findall('s:path', NS)


class Diagram(m.Scene):
    def construct(self):
        self.box = m.Square(fill_color=m.RED, fill_opacity=1)
        self.add(self.box)
        self.play(self.box.animate.shift(2 * m.RIGHT), run_time=0.5)
        self.add(m.Text('Native').shift(m.UP), m.Tex(r'x^2 + y^2').shift(m.DOWN))


with tempfile.TemporaryDirectory(prefix='fmn-native-svg-') as directory:
    root = Path(directory)
    first = root / 'diagram.svg'
    scene = Diagram()
    result = scene.render(first, resolution=(160, 90), fps=8, threads=1)
    assert result.format == 'svg' and result.frame_count == 1
    assert result.engine == 'native-svg' and not result.certified
    assert result.digest == hashlib.sha256(first.read_bytes()).hexdigest()
    assert result.bytes == first.stat().st_size
    assert abs(scene.box.get_center()[0] - 2) < 1e-5, 'skip mode lost animation endpoint'
    document, shapes = paths(first)
    assert float(document.attrib['width']) == 160
    assert float(document.attrib['height']) == 90
    assert len(shapes) > 5, 'native text or math glyphs missing'
    for threads in (4, 16):
        replay = root / f'replay-{threads}.svg'
        Diagram().render(replay, resolution=(160, 90), fps=8, threads=threads)
        assert first.read_bytes() == replay.read_bytes(), 'thread budget changed vector bytes'
    before = first.read_bytes()
    try:
        Diagram().render(first, resolution=(160, 90), fps=8, threads=1)
    except Exception:
        pass
    else:
        raise AssertionError('SVG overwrite succeeded')
    assert first.read_bytes() == before

    class Transparent(m.Scene):
        def construct(self):
            self.camera.background_rgba[3] = 0
            self.box = m.Square(fill_opacity=1)
            self.box.set_stroke(behind=True, width=4)
            self.add(self.box)
    transparent = root / 'transparent.svg'
    Transparent().render(transparent, resolution=(160, 90), fps=8)
    _, elements = paths(transparent)
    assert len(elements) == 2, 'expected only stroke-behind then fill, no background'
    assert elements[0].attrib['fill'] == 'none'
    assert elements[1].attrib['stroke'] == 'none'

    class Failed(m.Scene):
        def construct(self):
            self.add(m.Circle())
            self.wait(0.25)
            raise RuntimeError('deliberate vector scene failure')
    failed = root / 'failed.svg'
    try:
        Failed().render(failed, resolution=(160, 90), fps=8)
    except RuntimeError as error:
        assert 'deliberate vector' in str(error)
    else:
        raise AssertionError('failed scene published')
    assert not failed.exists()

    class Depth(m.Scene):
        def construct(self):
            self.add(m.Square().apply_depth_test())
    rejected = root / 'depth.svg'
    try:
        Depth().render(rejected, resolution=(160, 90), fps=8)
    except Exception as error:
        assert 'SVG export' in str(error) and 'depth' in str(error)
    else:
        raise AssertionError('depth semantics were discarded')
    assert not rejected.exists()

    imperative = m.Scene()
    with imperative.render_session(root / 'imperative.svg', resolution=(160, 90), fps=8) as session:
        box = m.Square()
        imperative.add(box)
        imperative.play(box.animate.shift(m.LEFT), run_time=0.25)
    assert session.result.frame_count == 1
    assert abs(box.get_center()[0] + 1) < 1e-5
    assert paths(root / 'imperative.svg')[1]

    batch = render_scenes({'first': Diagram, 'second': Transparent}, root / 'batch',
                         format='svg', resolution=(160, 90), fps=8, threads=1)
    assert batch.ok, batch.as_dict()
    assert (root / 'batch' / 'first.svg').exists()
    assert (root / 'batch' / 'second.svg').exists()
    _svg_document = first.read_bytes()
    _svg_thread_counts = 3
    _svg_failure_paths = 3

print('native SVG output acceptance passed')
