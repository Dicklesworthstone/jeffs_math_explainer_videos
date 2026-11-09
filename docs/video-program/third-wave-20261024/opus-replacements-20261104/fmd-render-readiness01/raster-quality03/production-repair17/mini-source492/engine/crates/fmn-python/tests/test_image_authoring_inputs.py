"""Input-normalization tests only; no native image or renderer is simulated."""
from __future__ import annotations
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch
import numpy as np

spec = importlib.util.spec_from_file_location('raster_input_test_subject',
    Path(__file__).resolve().parents[1] / 'python/fmn_python/image_authoring.py')
subject = importlib.util.module_from_spec(spec)
spec.loader.exec_module(subject)

class ImageInputTests(unittest.TestCase):
    def test_channels_and_top_row_orientation(self):
        gray = np.arange(12, dtype=np.uint8).reshape(3, 4)
        for channels in range(1, 5):
            data = np.stack([gray+i for i in range(channels)], axis=-1)
            width, height, raw = subject._pixels(np, data)
            self.assertEqual((width, height), (4, 3))
            actual = np.frombuffer(raw, dtype=np.uint8).reshape(height, width, 4)
            np.testing.assert_array_equal(actual[..., 0], gray)
            np.testing.assert_array_equal(actual[..., 1], gray if channels < 3 else gray+1)
            np.testing.assert_array_equal(actual[..., 2], gray if channels < 3 else gray+2)
            np.testing.assert_array_equal(actual[..., 3], data[..., -1] if channels in (2,4) else 255)
        self.assertEqual(subject._pixels(np, gray), subject._pixels(np, gray[..., None]))

    def test_noncontiguous_inputs_are_owned(self):
        data = np.arange(96, dtype=np.uint8).reshape(4, 6, 4)
        for value in (data[::-1, ::-1], data[:, ::2], data.transpose(1,0,2)):
            expected = value.copy()
            w, h, raw = subject._pixels(np, value)
            actual = np.frombuffer(raw, dtype=np.uint8).reshape(h,w,4)
            np.testing.assert_array_equal(actual, expected)
        data[:] = 0
        np.testing.assert_array_equal(actual, expected)

    def test_no_implicit_float_normalization_integer_wrapping_or_empty_pixels(self):
        for array in (np.zeros((3,4,4)), np.ones((3,4,4),dtype=np.int16),
                      np.ones((3,4,4),dtype=bool), np.ones((3,4,4),dtype=complex)):
            with self.assertRaisesRegex(TypeError,'uint8'):
                subject._pixels(np, array)
        for shape in ((0,1,4),(1,0,4),(3,4,0),(3,4,5),(4,), (1,2,3,4)):
            with self.assertRaises(ValueError):
                subject._pixels(np, np.zeros(shape,dtype=np.uint8))

    def test_budget_checked_before_allocating_or_copying_expanded_array(self):
        array = np.broadcast_to(np.zeros((1,1,1),dtype=np.uint8),(4097,4096,1))
        with patch.object(np,'empty',side_effect=AssertionError('must refuse before expanding')):
            with self.assertRaisesRegex(ValueError,'budget'):
                subject._pixels(np,array)

    def test_encoded_budget_counts_memoryview_bytes_not_elements(self):
        with patch.object(subject,'_MAX_ENCODED_BYTES',15):
            for data in (b'x'*16,bytearray(16),memoryview(np.zeros(4,dtype=np.uint32))):
                with self.assertRaisesRegex(ValueError,'budget'):
                    subject._encoded(data)
        raw=bytearray(b'owned')
        frozen=subject._encoded(raw)
        raw[:]=b'xxxxx'
        self.assertEqual(frozen,b'owned')

    def test_reentrancy_guard_releases_on_failure(self):
        class Target: pass
        obj=Target()
        with self.assertRaisesRegex(RuntimeError,'reenter'):
            with subject._editing(obj), subject._editing(obj):
                pass
        self.assertNotIn(subject._BUSY,vars(obj))
        with subject._editing(obj):
            self.assertTrue(vars(obj)[subject._BUSY])
        self.assertNotIn(subject._BUSY,vars(obj))

if __name__=='__main__':
    unittest.main()
