"""Native image path conversion, publication and animation input regressions."""
from __future__ import annotations

import struct
import tempfile
from pathlib import Path
import unittest
import zlib

import numpy as np
import manimlib as m


def pixels(color=(255, 0, 0, 255)):
    return np.full((3, 5, 4), color, dtype=np.uint8)


def png(data):
    height, width, _ = data.shape
    def chunk(kind, payload):
        return (struct.pack('>I', len(payload)) + kind + payload
                + struct.pack('>I', zlib.crc32(kind + payload)))
    return (b'\x89PNG\r\n\x1a\n'
            + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(b''.join(b'\0' + row.tobytes() for row in data)))
            + chunk(b'IEND', b''))


def capture(image):
    return m.Camera(resolution=(48, 32)).capture_snapshot(image)


class RasterPathTests(unittest.TestCase):
    def test_path_protocol_is_used_once_without_stringifying_the_object(self):
        import os
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'actual image.png'
            path.write_bytes(png(pixels((0, 0, 255, 255))))
            calls = []
            class InputPath:
                def __init__(self, raw):
                    self.raw = raw
                def __fspath__(self):
                    calls.append(self.raw)
                    return self.raw
                def __str__(self):
                    raise AssertionError('a path is not its display string')
            for raw in (str(path), os.fsencode(path)):
                supplied = InputPath(raw)
                calls.clear()
                image = m.ImageMobject(supplied)
                self.assertEqual(calls, [raw])
                np.testing.assert_array_equal(image.get_pixel_array(), pixels((0, 0, 255, 255)))
                calls.clear()
                image.set_image(supplied)
                self.assertEqual(calls, [raw])
                calls.clear()
                animation = image.animate.set_image(supplied).build()
                self.assertEqual(calls, [raw])
                animation.begin(); animation.finish()
                self.assertEqual(calls, [raw])  # playback never reopens the path
                self.assertEqual(image.image_path, str(path))

    def test_path_conversion_failure_preserves_identity_pixels_and_constructor_state(self):
        image = m.ImageMobject(pixels())
        before = image.data.copy(), capture(image).png(), image.image_path
        for failure in (ValueError('path failed'), KeyboardInterrupt(), SystemExit(7)):
            class InputPath:
                def __fspath__(self):
                    raise failure
            for operation in (lambda: image.set_image(InputPath()),
                              lambda: image.animate.set_image(InputPath()),
                              lambda: image.__init__(InputPath())):
                with self.assertRaises(type(failure)) as caught:
                    operation()
                self.assertIs(caught.exception, failure)
                np.testing.assert_array_equal(image.data, before[0])
                self.assertEqual(capture(image).png(), before[1])
                self.assertEqual(image.image_path, before[2])
                self.assertNotIn('_fmn_image_authoring_busy', vars(image))

    def test_path_reentry_and_invalid_path_return_do_not_publish(self):
        image = m.ImageMobject(pixels())
        class Reentrant:
            def __fspath__(self):
                image.set_pixel_array(pixels((0, 255, 0, 255)))
                return 'unreachable.png'
        class Invalid:
            def __fspath__(self):
                return 42
        for value, kind in ((Reentrant(), RuntimeError), (Invalid(), TypeError)):
            with self.assertRaises(kind):
                image.set_image(value)
            np.testing.assert_array_equal(image.get_pixel_array(), pixels())
            self.assertNotIn('_fmn_image_authoring_busy', vars(image))


if __name__ == '__main__':
    unittest.main()
