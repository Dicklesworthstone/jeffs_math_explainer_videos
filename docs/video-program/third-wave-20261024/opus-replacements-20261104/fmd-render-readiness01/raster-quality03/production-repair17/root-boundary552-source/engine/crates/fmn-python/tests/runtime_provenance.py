"""Installed-wheel runtime provenance and native output acceptance.

Uses actual installed payloads, Scene rendering and native manifest publication.
No injected inventory, monkeypatch, or renderer double satisfies these cases.
"""
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib

import manimlib as ml
from fmn_python.console_rendering import try_render_cli
from fmn_python.runtime_identity import capture_runtime


_SOURCE = {"runtime_provenance.py": Path(__file__).read_bytes()}
_ROOT = Path(tempfile.mkdtemp(prefix="fmn-runtime-provenance-native-"))
_SVG = b'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="6" height="4"/></svg>'


def _items(result):
    text = result.manifest.with_name("manifest.txt").read_text()
    return tomllib.loads(text)["items"]


def _reads(result):
    return {item["virtual_path"]: item["digest"] for item in _items(result)
            if item["id"] == 6 and item.get("virtual_path", "").startswith("read/")}


class _Reads(ml.Scene):
    """Places a dot from a CSV the scene reads and draws an SVG file."""
    data = svg = None

    def construct(self):
        with open(self.data) as stream:
            x = float(stream.read().split(",")[1])
        self.add(ml.Dot().shift(x * ml.RIGHT), ml.SVGMobject(str(self.svg)))


def _render_reads(name, csv):
    folder = _ROOT / name
    folder.mkdir()
    (folder / "data.csv").write_bytes(csv)
    (folder / "shape.svg").write_bytes(_SVG)
    scene = _Reads()
    scene.data, scene.svg = folder / "data.csv", folder / "shape.svg"
    return scene.render(folder / "reads.png", reproducible=True, sources=_SOURCE,
                        resolution=(96, 54), fps=4, threads=1)


def test_scene_file_reads_are_content_named_c6_inputs():
    first = _render_reads("reads-a", b"x,1.5\n")
    again = _render_reads("reads-b", b"x,1.5\n")
    moved = _render_reads("reads-c", b"x,-1.5\n")
    expected = {
        "read/" + hashlib.sha256(b"x,1.5\n").hexdigest() + "/data.csv",
        "read/" + hashlib.sha256(_SVG).hexdigest() + "/shape.svg",
    }
    # The native SVG reader raises the audit event Python's open() would.
    assert set(_reads(first)) == expected, _reads(first)
    for path, digest in _reads(first).items():
        assert digest == path.split("/")[1], (path, digest)
    # Content-named, host-path-free: equal inputs in another folder give the
    # same closure; a different CSV gives a different closure and output.
    assert again.closure_digest == first.closure_digest
    assert again.destination.read_bytes() == first.destination.read_bytes()
    assert moved.closure_digest != first.closure_digest
    assert moved.destination.read_bytes() != first.destination.read_bytes()
    c6 = [item for item in _items(first) if item["id"] == 6 and "virtual_path" not in item]
    assert [item["detail"] for item in c6] == [
        "scene file and sound-cue asset reads on the portal route; bundled fonts listed"
    ]


class _Effect(ml.Scene):
    effect = None

    def construct(self):
        self.effect()
        self.add(ml.Dot())


def _missing():
    try:
        open(_ROOT / "absent.csv").close()
    except FileNotFoundError:
        pass


def _rewrite():
    path = _ROOT / "rewritten.csv"
    path.write_bytes(b"before")
    path.read_bytes()
    path.write_bytes(b"after!")


def test_uncapturable_scene_effects_refuse_certification_without_output():
    cases = {
        "subprocess": (lambda: subprocess.run([sys.executable, "-c", "pass"], check=True),
                       "a subprocess (subprocess.Popen)"),
        "missing": (_missing, "a read of a missing file: " + str(_ROOT / "absent.csv")),
        "listing": (lambda: os.listdir(_ROOT), "a directory listing: " + str(_ROOT)),
        "device": (lambda: open("/dev/urandom", "rb").close(),
                   "a read of a file that is not a regular file: /dev/urandom"),
        "rewritten": (_rewrite, "a file that changed during rendering: " + str(_ROOT / "rewritten.csv")),
    }
    for name, (effect, message) in cases.items():
        scene = _Effect()
        scene.effect = effect
        destination = _ROOT / ("refused-" + name + ".png")
        try:
            scene.render(destination, reproducible=True, sources=_SOURCE, resolution=(96, 54), fps=4)
        except ml._CapabilityError as error:
            assert str(error).startswith("CAPABILITY: certified output cannot capture " + message), error
        else:
            raise AssertionError(name + ": an uncapturable effect was certified")
        assert not destination.exists(), name
        assert not destination.with_name(destination.name + ".manifest").exists(), name
    # Standard output makes no closure claim: the same effects render.
    scene = _Effect()
    scene.effect = cases["subprocess"][0]
    standard = scene.render(_ROOT / "standard-subprocess.png", resolution=(96, 54), fps=4)
    assert not standard.certified and standard.destination.is_file()


def _cli(folder, module):
    folder.mkdir()
    (folder / "data.csv").write_bytes(b"x,0.5\n")
    (folder / "scene.py").write_text(
        "from pathlib import Path\n"
        "import subprocess, sys\n"
        "from manimlib import Scene, Dot, RIGHT\n"
        + module +
        "class Shown(Scene):\n"
        "    def construct(self):\n"
        "        self.add(Dot().shift(X * RIGHT))\n"
    )
    stdout, stderr = io.StringIO(), io.StringIO()
    with redirect_stdout(stdout), redirect_stderr(stderr):
        code = try_render_cli(ml._native, ["--robot", str(folder / "scene.py"), "Shown", "--reproducible",
                                           "--format", "png", "--resolution", "96x54", "--fps", "4",
                                           "--threads", "1", "--video_dir", str(folder / "out.png")])
    return code, stdout.getvalue(), stderr.getvalue()


def test_certified_cli_records_scene_module_effects_from_import_time():
    folder = _ROOT / "cli-read"
    code, stdout, stderr = _cli(folder, "X = float(Path(__file__).with_name('data.csv').read_text().split(',')[1])\n")
    assert code == 0, (code, stdout, stderr)
    manifest = folder / "out.png.manifest" / "manifest.txt"
    items = tomllib.loads(manifest.read_text())["items"]
    read = "read/" + hashlib.sha256(b"x,0.5\n").hexdigest() + "/data.csv"
    assert [item["virtual_path"] for item in items if item.get("virtual_path", "").startswith("read/")] == [read]
    # The scene module itself is C1 source, loaded by the import system,
    # not a scene read.
    assert any(item["id"] == 1 and item.get("virtual_path") == "scene.py" for item in items), items

    folder = _ROOT / "cli-subprocess"
    code, stdout, stderr = _cli(folder, "subprocess.run([sys.executable, '-c', 'pass'], check=True)\nX = 1\n")
    assert code == 4, (code, stdout, stderr)
    receipt = json.loads(stdout.splitlines()[-1])
    assert receipt["kind"] == "render-capability-unavailable", receipt
    assert "a subprocess (subprocess.Popen)" in stdout, stdout
    assert not (folder / "out.png").exists() and not (folder / "out.png.manifest").exists()


def test_loaded_runtime_has_content_identities():
    snapshot = capture_runtime(ml)
    identities = snapshot.identities
    assert set(identities) == {"cpython", "abi", "wheel", "numpy"}
    assert all("installed-payload-sha256=" in identities[name] for name in ("cpython", "wheel", "numpy"))
    assert identities["abi"].startswith("cpython-abi-sha256=")
    snapshot.verify()


def test_both_scene_front_doors_publish_real_png_and_manifest():
    class Static(ml.Scene):
        def construct(self):
            self.add(ml.Dot().shift(ml.RIGHT))

    rendered = Static().render(_ROOT / "render.png", reproducible=True,
                               sources=_SOURCE, resolution=(96, 54), fps=4, threads=1)
    scene = ml.Scene()
    with scene.render_session(_ROOT / "session.png", reproducible=True, sources=_SOURCE,
                              resolution=(96, 54), fps=4, threads=4) as session:
        scene.add(ml.Dot().shift(ml.RIGHT))
    imperative = session.result
    for result in (rendered, imperative):
        assert result.certified and result.closure_digest and result.frame_count == 1
        assert result.manifest.is_file()
        assert result.destination.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    assert rendered.destination.read_bytes() == imperative.destination.read_bytes()
    assert "installed-payload-sha256=" in session.runtime_identities["wheel"]


class _OnePlay(ml.Scene):
    """One half-second play: its frame count follows fps."""

    def construct(self):
        self.play(ml.Dot().animate.shift(ml.RIGHT), run_time=0.5)


def _c4(result):
    return [item["digest"] for item in _items(result) if item["id"] == 4]


def _render_one_play(tag, fps, threads, seed=None):
    folder = _ROOT / f"c4-{tag}"
    folder.mkdir()
    scene = _OnePlay()
    if seed is not None:
        scene.random_seed = seed
    return folder, scene.render(folder / "frames.png", format="png_sequence", reproducible=True,
                                sources=_SOURCE, resolution=(64, 36), fps=fps, threads=threads)


def test_certified_closure_states_fps_and_seed_but_not_threads():
    # fm-arog: C4 is the configuration the render resolved, not the defaults
    # document, so frames that differ in fps differ in closure (§16.7).
    _, slow = _render_one_play("fps10", 10, 1)
    _, fast = _render_one_play("fps20", 20, 1)
    _, wide = _render_one_play("fps10-threads4", 10, 4)
    _, seeded = _render_one_play("seed7", 10, 1, seed=7)
    assert (slow.frame_count, fast.frame_count) == (5, 10)
    assert fast.closure_digest != slow.closure_digest
    assert _c4(fast) != _c4(slow)
    assert _c4(seeded) != _c4(slow) and seeded.closure_digest != slow.closure_digest
    # The scheduler width is inert: the same frames under the same closure.
    assert wide.digest == slow.digest
    assert wide.closure_digest == slow.closure_digest and _c4(wide) == _c4(slow)
    # A seed the configuration cannot state is refused before any output exists.
    folder = _ROOT / "c4-unstatable-seed"
    folder.mkdir()
    scene = _OnePlay()
    scene.random_seed = 2**63
    try:
        scene.render(folder / "frames.png", format="png_sequence", reproducible=True,
                     sources=_SOURCE, resolution=(64, 36), fps=10, threads=1)
    except Exception as error:
        assert "C4" in str(error) and "signed 64-bit" in str(error), error
    else:
        raise AssertionError("a seed C4 cannot state was certified")
    assert list(folder.iterdir()) == []


def test_false_runtime_labels_refuse_without_native_publication():
    destination = _ROOT / "refused.png"
    scene = ml.Scene()
    try:
        with scene.render_session(destination, reproducible=True, sources=_SOURCE,
                                  runtime_identities={"wheel": "unverified build"}):
            raise AssertionError("runtime refusal occurred too late")
    except Exception as error:
        assert "supplied runtime identities" in str(error), error
    else:
        raise AssertionError("arbitrary runtime identity was accepted")
    assert not destination.exists()
    assert not destination.with_name(destination.name + ".manifest").exists()


assert getattr(ml, "__franken_manim__", False), "requires the installed native FrankenManim portal"
_cases = sorted((name, case) for name, case in globals().items()
                if name.startswith("test_") and callable(case))
assert len(_cases) == 7, "runtime provenance native acceptance inventory changed"
for _name, _case in _cases:
    _case()
    print("runtime provenance native acceptance passed:", _name)
print("retaining native runtime provenance outputs:", _ROOT)
