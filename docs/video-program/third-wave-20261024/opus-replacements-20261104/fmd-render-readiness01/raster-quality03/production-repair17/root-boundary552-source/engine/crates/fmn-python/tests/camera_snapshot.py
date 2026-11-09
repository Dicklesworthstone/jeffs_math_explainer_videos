"""Native image snapshots and observational console previews, including pixels.

Run against the installed wheel or the registered embedded native acceptance.
No storage, camera, PNG encoder or renderer is replaced by a test double.
"""
from pathlib import Path
import os
import struct
import tempfile
import zlib

import numpy as np
import manimlib as m
from fmn_python import SceneConsole, render_scene


def decode_png(data):
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    offset, packed = 8, bytearray()
    while offset < len(data):
        size = int.from_bytes(data[offset:offset + 4], "big")
        kind, body = data[offset + 4:offset + 8], data[offset + 8:offset + 8 + size]
        assert zlib.crc32(kind + body) == int.from_bytes(data[offset + 8 + size:offset + 12 + size], "big")
        offset += size + 12
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", body)
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
        elif kind == b"IDAT":
            packed.extend(body)
        elif kind == b"IEND":
            assert offset == len(data)
            break
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
            paeth = min((left, up, corner), key=lambda candidate: abs(estimate - candidate))
            decoded[y * stride + x] = (value + (0, left, up, (left + up) // 2, paeth)[mode]) & 255
    return np.frombuffer(decoded, dtype=np.uint8).reshape(height, width, 4)


def run_camera_snapshot_acceptance():
    root = Path(os.environ.get("FMN_PREVIEW_ARTIFACT_DIR") or tempfile.mkdtemp(prefix="fmn-preview-snapshot-"))
    root.mkdir(parents=True, exist_ok=True)
    camera = m.Camera(resolution=(128, 72), background_opacity=0)
    square = m.Square(side_length=1, fill_color=m.RED, fill_opacity=1, stroke_width=0)
    square.shift(2 * m.LEFT + m.UP)
    image = camera.capture_snapshot(square)
    assert image.size == (128, 72) and not square._is_bound()
    rgba = camera.get_pixel_array()
    payload = image.png()
    assert image._repr_png_() == camera._repr_png_() == camera.get_png() == payload
    np.testing.assert_array_equal(decode_png(payload), rgba)
    (root / "detached-red.png").write_bytes(payload)
    square.shift(4 * m.RIGHT)
    other = camera.capture_snapshot(square)
    assert other is not image and other.png() != payload
    assert image.png() == payload
    camera.clear()
    camera.reset_pixel_shape(64, 36)
    assert decode_png(camera.get_png()).shape == (36, 64, 4)
    assert image.size == (128, 72) and image.png() == payload

    # Preview and notebook redisplay must not run stateful zero-dt updaters.
    # Only an explicitly executed cell performs its normal post-cell refresh.
    scene = m.Scene()
    scene.camera.reset_pixel_shape(128, 72)
    scene.add(square)
    ticks = []
    square.add_updater(lambda mob, dt: ticks.append(dt), call=False)
    clock, plays, roots = m._portal_scene_clock(scene), scene.num_plays, tuple(scene.mobjects)
    with SceneConsole(scene, {"square": square, "LEFT": m.LEFT}, capture=False) as console:
        preview = console.preview()
        assert ticks == [] and scene.num_plays == plays and tuple(scene.mobjects) == roots
        assert m._portal_scene_clock(scene) == clock
        assert not console.checkpoint_manager.checkpoint_states and console._cells == 0
        assert console._repr_png_() is None
        console.capture = True
        before = preview.png()
        for _ in range(3):
            assert console._repr_png_() == before
        assert ticks == [] and m._portal_scene_clock(scene) == clock
        console.run_cell("# display\nsquare.shift(LEFT)")
        assert console._repr_png_() != before and preview.png() == before
        frame = console._repr_png_()
        calls = len(ticks)
        console._repr_png_()
        console.preview()
        assert len(ticks) == calls and m._portal_scene_clock(scene) == clock
        (root / "console-live.png").write_bytes(frame)
    assert console._repr_png_() is None and preview.png() == before
    square.clear_updaters()

    # A mixed native family exercises payloads that cannot be recovered from
    # path points alone: image texture, glyphs, 3D topology and surface normals.
    fixture = root / "image-source.png"
    fixture.write_bytes(payload)
    class Mixed(m.Scene):
        def construct(self):
            picture = m.ImageMobject(str(fixture)).set_height(2).shift(3 * m.LEFT)
            text = m.Tex(r"x^2+1").shift(2 * m.UP)
            sphere = m.Sphere(radius=.7, resolution=(5, 7)).shift(2 * m.RIGHT)
            self.add(picture, text, sphere)
    output = root / "mixed-native-output.png"
    mixed = Mixed()
    receipt = render_scene(mixed, output, format="png", resolution=(128, 72), fps=8, threads=1)
    assert receipt.frame_count == 1
    expected = decode_png(output.read_bytes())
    assert np.any(expected[:, :, :3])
    observed = mixed.camera.capture_snapshot(*mixed.mobjects)
    np.testing.assert_array_equal(decode_png(observed.png()), expected)
    (root / "mixed-snapshot.png").write_bytes(observed.png())
    for threads in (4, 16):
        mixed.camera.capture_threads = threads
        current = mixed.camera.capture_snapshot(*mixed.mobjects)
        assert current.png() == observed.png()
    print("native snapshot acceptance passed: PNG roundtrip, immutable images, observational console, mixed textures/math/3D, 1/4/16 threads")


run_camera_snapshot_acceptance()
