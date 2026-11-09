"""Real live-equation rendering, thread replay and failed-generation acceptance.

This is the same source used by the embedded Gauntlet and installed-wheel
checks. Pixel assertions decode the native PNGs independently using Python's
standard-library zlib; no alternative scene renderer participates.
"""
from pathlib import Path
import hashlib
import struct
import tempfile
import zlib

import manimlib as m
import numpy as np


def _pixels(payload):
    assert payload[:8] == b"\x89PNG\r\n\x1a\n"
    offset, packed, shape, ended = 8, bytearray(), None, False
    while offset < len(payload):
        size = int.from_bytes(payload[offset:offset + 4], "big")
        assert offset + 12 + size <= len(payload)
        kind = payload[offset + 4:offset + 8]
        body = payload[offset + 8:offset + 8 + size]
        crc = int.from_bytes(payload[offset + 8 + size:offset + 12 + size], "big")
        assert zlib.crc32(kind + body) & 0xffffffff == crc
        offset += size + 12
        if kind == b"IHDR":
            assert shape is None
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", body)
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
            shape = width, height
        elif kind == b"IDAT":
            packed.extend(body)
        elif kind == b"IEND":
            assert size == 0 and offset == len(payload)
            ended = True
            break
    assert shape == (192, 108) and ended
    width, height = shape
    raw, stride = zlib.decompress(packed), width * 4
    assert len(raw) == height * (stride + 1)
    result = bytearray(height * stride)
    for y in range(height):
        mode = raw[y * (stride + 1)]
        assert mode in range(5)
        row = raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)]
        for x, byte in enumerate(row):
            left = result[y * stride + x - 4] if x >= 4 else 0
            up = result[(y - 1) * stride + x] if y else 0
            corner = result[(y - 1) * stride + x - 4] if y and x >= 4 else 0
            estimate = left + up - corner
            paeth = min((left, up, corner), key=lambda v: abs(estimate - v))
            result[y * stride + x] = (byte + (0, left, up, (left + up) // 2, paeth)[mode]) & 255
    return np.frombuffer(result, dtype=np.uint8).reshape(height, width, 4)


class LiveEquations(m.Scene):
    default_camera_config = dict(resolution=(192, 108), fps=8)

    def construct(self):
        self.formula = m.Tex("y =", "1.00 x + 2.00", color=m.RED)
        parts = list(self.formula.submobjects)
        wrapper = m.VGroup(m.VGroup(*parts))
        self.formula.set_submobjects([wrapper])
        self.formula._string_sub_paths = [[0, 0, *p] for p in self.formula._string_sub_paths]
        self.formula.scale(2.5).shift(m.UP)
        self.matrix = m.TexMatrix([["12.50", "x"]]).scale(2).shift(1.3 * m.DOWN)
        self.add(self.formula, self.matrix)
        first = self.formula.make_number_changeable("1.00")
        second = self.formula.make_number_changeable("2.00")
        entry = self.matrix.get_mob_matrix()[0][0]
        third = entry.make_number_changeable("12.50")
        self.values = first, second, third
        assert all(isinstance(value, m.DecimalNumber) for value in self.values)
        assert first.get_color() == m.RED and second.get_color() == m.RED
        # The authored nested wrapper survives. Each number's glyph run is
        # replaced in place by its DecimalNumber, as the Reference's
        # make_number_changeable does, now inside the wrapper: modern Tex is a
        # flat glyph family since f3555bf9 (y, =, 1.00, x, +, 2.00).
        assert self.formula[0] is wrapper
        assert list(wrapper[0]) == [parts[0], parts[1], first, parts[6], parts[7], second]
        assert self.matrix.get_mob_matrix()[0][0] is entry
        self.wait(1 / 8)
        self.play(*(m.ChangeDecimalToValue(value, target) for value, target in
                    zip(self.values, (4.25, -3.5, 27.75))), run_time=1 / 2, rate_func=m.linear)
        self.wait(1 / 8)
        assert tuple(value.get_value() for value in self.values) == (4.25, -3.5, 27.75)
        assert self.formula.get_part_by_tex(r"\decimalmob", 0)[0] is first
        assert self.formula.get_part_by_tex(r"\decimalmob", 1)[0] is second
        assert entry.get_part_by_tex(r"\decimalmob")[0] is third
        assert list(self.mobjects) == [self.formula, self.matrix], "animated descendants became extra draw roots"


def render_live_tex(destination, seed=0):
    root = Path(destination)
    root.mkdir(parents=True, exist_ok=True)
    # The public constructor also seeds host NumPy, whose legacy seed is u32.
    seed &= 0xFFFF_FFFF
    sequences = []
    for threads in (1, 4, 16):
        scene = LiveEquations(random_seed=seed)
        receipt = scene.render(root / f"threads-{threads}", threads=threads)
        files = sorted(receipt.destination.glob("*.png"))
        assert len(files) == receipt.frame_count == 6
        assert receipt.certified is False
        assert receipt.resolution == (192, 108) and receipt.fps == 8
        assert receipt.bytes == sum(path.stat().st_size for path in files)
        payloads = [path.read_bytes() for path in files]
        assert payloads[0] != payloads[-1], "live values must change actual pixels"
        assert len(set(payloads[:5])) == 5, "live values must interpolate, not jump to the final value"
        sequences.append(payloads)
    del scene
    assert sequences[0] == sequences[1] == sequences[2], "thread scheduling changed a live equation"
    images = [_pixels(payload) for payload in sequences[0]]
    for pixels in images:
        rgb = pixels[:, :, :3].astype(np.int16)
        assert np.count_nonzero((rgb[:, :, 0] > rgb[:, :, 1] + 20) & (rgb[:, :, 0] > rgb[:, :, 2] + 20)) > 10
        assert np.count_nonzero(rgb[54:, :, :].max(axis=2) > 100) > 10
        assert np.all(pixels[:, :, 3] == 255)
    assert not np.array_equal(images[0][:54], images[-1][:54]), "formula values did not animate"
    assert not np.array_equal(images[0][54:], images[-1][54:]), "matrix value did not animate"

    sentinel = RuntimeError("authored live-equation failure")

    class BrokenEquation(LiveEquations):
        def construct(self):
            super().construct()
            raise sentinel

    failed = root / "failed-generation"
    try:
        BrokenEquation(random_seed=seed).render(failed, threads=4)
    except RuntimeError as error:
        assert error is sentinel
    else:
        raise AssertionError("authored failure published an incomplete equation")
    finally:
        sentinel.__traceback__ = None
        del sentinel
    del BrokenEquation
    assert not failed.exists(), "failed render generation escaped native cancellation"

    # A real existing destination remains byte-identical on refusal.
    before = {path.name: path.read_bytes() for path in (root / "threads-1").iterdir() if path.is_file()}
    try:
        LiveEquations(random_seed=seed).render(root / "threads-1", threads=1)
    except (RuntimeError, ValueError, OSError) as err:
        err.__traceback__ = None
        del err
    else:
        raise AssertionError("live-equation render clobbered an existing generation")
    assert before == {path.name: path.read_bytes() for path in (root / "threads-1").iterdir() if path.is_file()}
    return sequences[0][0], sequences[0][-1], 6, 3, 2


if __name__ in ("__main__", "<run_path>"):
    root = Path(tempfile.mkdtemp(prefix="fmn-live-equations-"))
    first, last, frames, replays, refusals = render_live_tex(root)
    print(f"live equation render: {frames} frames, {replays} thread counts, {refusals} failure paths")
    print(f"first_sha256={hashlib.sha256(first).hexdigest()}")
    print(f"last_sha256={hashlib.sha256(last).hexdigest()}")
    print(f"artifacts={root}")
