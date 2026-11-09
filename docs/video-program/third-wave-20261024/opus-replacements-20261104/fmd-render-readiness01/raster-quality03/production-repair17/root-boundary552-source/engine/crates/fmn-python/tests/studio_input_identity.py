"""Actual native scan + SceneSource + Lumen capture generation acceptance."""
import hashlib
import json
from pathlib import Path
import tempfile
from unittest.mock import patch

import manimlib as m
from manimlib import _native
from fmn_python.studio import _build_id, _capture
from fmn_python.studio_inputs import StudioInputs, project_directory
from fmn_python.scene_loading import _FreshSource


def request(path, extras=()):
    inputs = StudioInputs(_native, list(dict.fromkeys([
        str(path), str(project_directory(path)), *(str(p) for p in extras),
    ])))
    value = dict(schema="fmn-python.studio-worker", version=2,
                 source=str(path), scene="Lesson",
                 source_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                 inputs=inputs.as_request(), width=64, height=36, fps=4, threads=1,
                 max_frames=16, max_bytes=1024 * 1024, interactive=False)
    value["build_id"] = _build_id(value)
    return value


def refused(call, message):
    try:
        call()
    except RuntimeError as error:
        assert message in str(error), str(error)
    else:
        raise AssertionError("stale input published a native recording")


with tempfile.TemporaryDirectory(prefix="fmn-studio-inputs-") as temp:
    root = Path(temp)
    package = root / "lesson_package"
    scenes = package / "scenes"
    scenes.mkdir(parents=True)
    (package / "__init__.py").write_text("")
    (scenes / "__init__.py").write_text("")
    helper = package / "helper.py"
    helper.write_text("VALUE = 1\n")
    source = scenes / "lesson.py"
    source.write_text('''from manimlib import Scene, Square, RIGHT
from .. import helper
class Lesson(Scene):
    def construct(self):
        self.add(Square().shift(helper.VALUE * RIGHT))
        self.wait(0.25)
''')
    first_request = request(source)
    first = _capture(first_request, _native)
    assert first.frame_count == 1
    first_nodes = json.loads(first.inspect(0))["nodes"]
    helper.write_text("VALUE = 2\n")
    refused(lambda: _capture(first_request, _native), "between launch and capture")
    second_request = request(source)
    assert first_request["source_sha256"] == second_request["source_sha256"]
    assert first_request["build_id"] != second_request["build_id"]
    second = _capture(second_request, _native)
    assert json.loads(second.inspect(0))["nodes"] != first_nodes
    assert json.loads(first.inspect(0))["nodes"] == first_nodes, "prior healthy capture changed"

    # Actual native assets share the identity, generated PNG output does not.
    asset = package / "values.csv"
    asset.write_text("123")
    before_asset = request(source, [asset])
    (package / "preview.png").write_bytes(b"unrelated output")
    assert before_asset["build_id"] == request(source, [asset])["build_id"]
    asset.write_text("456")
    refused(lambda: _capture(before_asset, _native), "between launch and capture")
    assert before_asset["build_id"] != request(source, [asset])["build_id"]

    # A lazy import during construct is included by the real SceneSource owner.
    source.write_text('''from manimlib import Scene, Circle
class Lesson(Scene):
    def construct(self):
        from .. import helper
        self.add(Circle(radius=helper.VALUE))
''')
    assert _capture(request(source), _native).frame_count == 1

    # Same-length temporary edit+revert: aggregate re-scan alone is insufficient.
    expected = request(source)
    original = _FreshSource.get_code
    def edited_import(loader, fullname):
        if Path(loader.path) == helper:
            old = helper.read_bytes()
            helper.write_text("VALUE = 3\n")
            try:
                return original(loader, fullname)
            finally:
                helper.write_bytes(old)
        return original(loader, fullname)
    with patch.object(_FreshSource, "get_code", edited_import):
        refused(lambda: _capture(expected, _native), "executed changed project source")

    # Persistent edits during construct cannot be attached to the old build.
    source.write_text('''from pathlib import Path
from manimlib import Scene, Circle
class Lesson(Scene):
    def construct(self):
        self.add(Circle())
        Path(__file__).parents[1].joinpath("helper.py").write_text("VALUE = 4\\n")
''')
    refused(lambda: _capture(request(source), _native), "changed during capture")

print("native Studio input identity acceptance passed")
