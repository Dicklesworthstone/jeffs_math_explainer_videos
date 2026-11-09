"""Real Lumen captures through the native Studio terminal encoders.

No terminal, codec, renderer, snapshot or scene is doubled. The independent
Kitty/sixel readers below verify transport and quantized preview pixels.
"""
from __future__ import annotations

import base64
from concurrent.futures import ThreadPoolExecutor
import re
import unittest

import numpy as np
import manimlib as m


def kitty_png(payload):
    chunks = re.findall(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\', payload)
    assert chunks and b''.join(b'\x1b_G'+control+b';'+body+b'\x1b\\'
                              for control, body in chunks) == payload
    assert chunks[0][0].startswith(b'a=T,f=100,t=d,q=2,')
    for index, (control, body) in enumerate(chunks):
        assert control.endswith(b'm=1' if index+1 < len(chunks) else b'm=0')
        assert len(body) <= 4096
    return base64.b64decode(b''.join(body for _, body in chunks), validate=True)


def sixel_pixels(payload, width, height):
    """Decode the fixed-palette, uncompressed raster subset emitted by Studio."""
    header = f'\x1bP0;0;0q"1;1;{width};{height}'.encode('ascii')
    assert payload.startswith(header) and payload.endswith(b'\x1b\\')
    body = payload[len(header):-2].decode('ascii')
    pixels = np.zeros((height, width, 4), dtype=np.uint8)
    palette, x, y, color, i = {}, 0, 0, None, 0
    while i < len(body):
        char = body[i]
        if char == '#':
            match = re.match(r'#(\d+)(?:;2;(\d+);(\d+);(\d+))?', body[i:])
            assert match is not None
            color = int(match[1])
            if match[2] is not None:
                palette[color] = tuple(int(match[k])*255//100 for k in (2, 3, 4))
            i += len(match[0])
            continue
        if char == '$':
            x = 0
        elif char == '-':
            x, y = 0, y+6
        else:
            bits = ord(char)-63
            assert 0 <= bits <= 63 and x < width and color in palette
            for bit in range(6):
                if bits & (1 << bit):
                    assert y+bit < height
                    pixels[y+bit, x] = (*palette[color], 255)
            x += 1
        i += 1
    return pixels


class TerminalPreviewTests(unittest.TestCase):
    def capture(self, threads=1):
        camera = m.Camera(resolution=(97, 61), background_opacity=0)
        camera.capture_threads = threads
        square = m.Square(side_length=2, fill_color=m.RED, fill_opacity=1, stroke_width=0)
        square.shift(m.LEFT + m.UP)
        circle = m.Circle(radius=.8, fill_color=m.BLUE, fill_opacity=.6, stroke_width=1)
        circle.shift(m.RIGHT)
        return camera, square, circle, camera.capture_snapshot(square, circle)

    def test_kitty_png_is_exact_lossless_snapshot_with_transparency(self):
        camera, square, circle, snapshot = self.capture()
        output = snapshot.terminal_bytes()
        self.assertEqual(kitty_png(output), snapshot.png())
        self.assertFalse(square._is_bound())
        self.assertFalse(circle._is_bound())
        self.assertGreater(len(set(camera.get_pixel_array()[:, :, 3].flat)), 2)

    def test_sixel_matches_native_preview_palette_and_alpha_threshold(self):
        camera, _, _, snapshot = self.capture()
        pixels = camera.get_pixel_array()
        expected = np.zeros_like(pixels)
        painted = pixels[:, :, 3] >= 128
        expected[painted, :3] = (pixels[painted, :3]//51)*51
        expected[painted, 3] = 255
        actual = sixel_pixels(snapshot.terminal_bytes('sixel'), *snapshot.size)
        np.testing.assert_array_equal(actual, expected)

    def test_old_terminal_snapshot_survives_recapture_resize_and_geometry_edits(self):
        camera, square, circle, snapshot = self.capture()
        old = tuple(snapshot.terminal_bytes(protocol) for protocol in ('kitty', 'sixel'))
        square.shift(3*m.RIGHT)
        camera.capture(square, circle)
        camera.reset_pixel_shape(32, 19)
        camera.clear()
        self.assertEqual(tuple(snapshot.terminal_bytes(protocol) for protocol in ('kitty', 'sixel')), old)
        self.assertEqual(snapshot.size, (97, 61))
        self.assertEqual(kitty_png(old[0]), snapshot.png())

    def test_encoding_does_not_advance_clock_tick_updaters_or_bind_objects(self):
        scene = m.Scene()
        scene.camera.reset_pixel_shape(64, 40)
        square = m.Square()
        scene.add(square)
        ticks = []
        square.add_updater(lambda obj, dt: ticks.append(dt))
        snapshot = scene.camera.capture_snapshot(*scene.mobjects)
        clock, points, count = scene.time, square.get_points().copy(), len(ticks)
        for protocol in ('kitty', 'sixel'):
            self.assertTrue(snapshot.terminal_bytes(protocol))
        self.assertEqual(scene.time, clock)
        self.assertEqual(len(ticks), count)
        np.testing.assert_array_equal(square.get_points(), points)

    def test_invalid_protocol_and_output_budgets_refuse_without_damaging_snapshot(self):
        _, _, _, snapshot = self.capture()
        original = snapshot.png()
        for protocol in ('', 'auto', 'Kitty', '\x1b_G', 'png'):
            with self.subTest(protocol=protocol), self.assertRaises(ValueError):
                snapshot.terminal_bytes(protocol)
        for limit in (0, 134217729):
            with self.assertRaises(ValueError):
                snapshot.terminal_bytes(max_bytes=limit)
        for protocol in ('kitty', 'sixel'):
            with self.assertRaisesRegex(Exception, 'limit'):
                snapshot.terminal_bytes(protocol, max_bytes=1)
            output = snapshot.terminal_bytes(protocol)
            self.assertEqual(snapshot.terminal_bytes(protocol, max_bytes=len(output)), output)
            with self.assertRaisesRegex(Exception, 'limit'):
                snapshot.terminal_bytes(protocol, max_bytes=len(output)-1)
        self.assertEqual(snapshot.png(), original)

    def test_both_protocols_are_independent_of_capture_thread_count(self):
        outputs = []
        for threads in (1, 4, 16):
            _, _, _, snapshot = self.capture(threads)
            outputs.append(tuple(snapshot.terminal_bytes(protocol) for protocol in ('kitty', 'sixel')))
        self.assertEqual(outputs[0], outputs[1])
        self.assertEqual(outputs[1], outputs[2])

    def test_immutable_snapshot_encodes_concurrently_without_live_scene_access(self):
        _, _, _, snapshot = self.capture()
        expected = snapshot.terminal_bytes()
        with ThreadPoolExecutor(max_workers=4) as pool:
            observed = list(pool.map(lambda _: snapshot.terminal_bytes(), range(12)))
        self.assertTrue(all(value == expected for value in observed))

    def test_native_kitty_chunking_preserves_large_lossless_payload(self):
        # Alternating opaque squares yield enough native image detail to cross
        # Kitty's chunk boundary without host-made image/codec test doubles.
        camera = m.Camera(resolution=(240, 180), background_opacity=0)
        objects = []
        for row in range(11):
            for column in range(13):
                color = m.RED if (row+column) % 2 else m.BLUE
                item = m.Square(side_length=.31, fill_color=color, fill_opacity=.85,
                                stroke_color=m.WHITE, stroke_width=.7)
                item.shift((column*.47-2.8, row*.53-2.5, 0))
                objects.append(item)
        snapshot = camera.capture_snapshot(*objects)
        output = snapshot.terminal_bytes()
        self.assertGreater(output.count(b'\x1b_G'), 1)
        self.assertEqual(kitty_png(output), snapshot.png())


result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(TerminalPreviewTests))
if not result.wasSuccessful():
    raise AssertionError('native terminal preview acceptance failed')
