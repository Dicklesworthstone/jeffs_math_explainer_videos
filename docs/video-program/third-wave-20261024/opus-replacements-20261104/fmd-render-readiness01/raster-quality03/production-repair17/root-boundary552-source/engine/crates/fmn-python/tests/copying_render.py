"""Authored array state through target creation, interpolation and native Reel.

Decode the native Y4M planes independently. Distinct intermediate frames prove
that source and target arrays are isolated; final geometry alone cannot do so.
"""
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from fmn_python import render_scene


class ControlSquare(m.Square):
    def __init__(self):
        super().__init__(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
        self.controls = np.array([-2.])
        self.set_x(-2.)

    def set_control(self, value):
        self.controls[0] = value
        self.set_x(value)
        return self

    def interpolate(self, start, end, alpha, path_func=None):
        super().interpolate(start, end, alpha, path_func)
        # Read immutable authored endpoints, not the array we are mutating.
        low, high = float(start.controls[0]), float(end.controls[0])
        self.controls[0] = (1. - alpha) * low + alpha * high
        self.set_y(2. * (high - low) * alpha * (1. - alpha))
        return self


class ArrayDrivenScene(m.Scene):
    default_camera_config = dict(resolution=(96, 54), fps=8)

    def construct(self):
        square = ControlSquare()
        self.add(square)
        square.save_state()
        animation = square.animate(run_time=.5, rate_func=m.linear).set_control(2.)
        assert square.controls[0] == -2. and square.saved_state.controls[0] == -2.
        self.play(animation)
        assert square.controls[0] == 2.
        assert square.saved_state.controls[0] == -2.
        np.testing.assert_allclose(square.get_center(), [2., 0., 0.], atol=2e-5)


def frames(path):
    header, payload = path.read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2", header
    size = 96 * 54 * 3 // 2
    assert len(payload) == 4 * (6 + size), len(payload)
    result = []
    for offset in range(0, len(payload), 6 + size):
        assert payload[offset:offset + 6] == b"FRAME\n"
        luma = np.frombuffer(payload, np.uint8, count=96 * 54, offset=offset + 6).reshape(54, 96)
        ys, xs = np.nonzero(luma > 180)
        assert len(xs) >= 16, "native renderer did not draw the authored object"
        result.append((float(xs.mean()), float(ys.mean())))
    return result


root = Path(tempfile.mkdtemp(prefix="fmn-copying-render-"))
outputs = []
for threads in (1, 4):
    result = render_scene(ArrayDrivenScene, root / f"array-copy-{threads}.y4m", threads=threads)
    assert result.frame_count == 4 and not result.certified
    centers = frames(result.destination)
    # x=-1,0,1,2; y=1.5,2,1.5,0. Camera y is upward, image y downward.
    expected = [(40.75, 16.375), (47.5, 13.), (54.25, 16.375), (61., 26.5)]
    np.testing.assert_allclose(centers, expected, atol=1.1, rtol=0.)
    outputs.append(result.destination.read_bytes())
    print("native array-copy rendering:", threads, "threads", centers)
assert outputs[0] == outputs[1], "the same scene changed with renderer thread count"
print("native array-copy render artifacts:", root)
