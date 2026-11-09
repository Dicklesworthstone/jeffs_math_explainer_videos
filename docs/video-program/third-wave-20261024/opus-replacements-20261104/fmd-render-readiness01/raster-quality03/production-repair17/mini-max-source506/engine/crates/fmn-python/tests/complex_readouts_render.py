"""Complex readouts and matrix entries through actual native PNG publication."""
from pathlib import Path
import hashlib
import runpy
import tempfile

import manimlib as m
import numpy as np


def _decode(payload):
    # Reuse the existing independent stdlib PNG decoder, not the engine's codec.
    helpers = runpy.run_path(str(Path(__file__).with_name("live_tex_render.py")),
                            run_name="_complex_readout_pixel_helpers")
    return helpers["_pixels"](payload)


class AuthoredCoefficient(m.DecimalNumber):
    def get_formatter(self, **kwargs):
        kwargs.setdefault("num_decimal_places", 1)
        return super().get_formatter(**kwargs)

    def char_to_mob(self, char):
        if char == "i":
            options = dict(self.text_config, slant="ITALIC")
            return m.Text(char, **options)
        return super().char_to_mob(char)


class CoefficientMatrix(m.DecimalMatrix):
    def element_to_mobject(self, element, **config):
        return AuthoredCoefficient(element, **config)


class ComplexReadouts(m.Scene):
    default_camera_config = dict(resolution=(192, 108), fps=8)

    def construct(self):
        self.formula = m.Tex("z = 1.00", color=m.RED).scale(2.5).shift(1.7 * m.UP)
        self.readout = self.formula.make_number_changeable("1.00")
        self.readout.set_value(1 + 2j)
        self.matrix = CoefficientMatrix(
            [[1 + 2j, -3j], [4 + 0j, 5 - 6j]],
            decimal_config=dict(color=m.BLUE,
                                text_config={"font": "IBM Plex Sans", "weight": "BOLD"}),
        ).scale(1.7).shift(m.DOWN)
        self.add(self.formula, self.matrix)
        self.wait(1 / 8)
        entries = list(self.matrix.elements)
        self.play(m.ChangeDecimalToValue(self.readout, -2j),
                  *(m.ChangeDecimalToValue(entry, target)
                    for entry, target in zip(entries, (2 - 1j, 4, 5j, 0j))), run_time=1 / 2)
        self.wait(1 / 8)
        assert self.readout.get_value() == -2j
        assert tuple(entry.get_value() for entry in entries) == (2 - 1j, 4, 5j, 0j)
        assert list(self.mobjects) == [self.formula, self.matrix]
        assert self.formula.get_part_by_tex(r"\decimalmob")[0] is self.readout
        assert list(self.matrix.elements) == entries
        assert all(len(entry) == len(entry.num_string) for entry in entries)
        assert [entry.get_tex() for entry in entries] == ["2.0–1.0i", "4.0", "5.0i", "0.0"]


def render_complex_readouts(destination, seed=0):
    root = Path(destination)
    root.mkdir(parents=True, exist_ok=True)
    sequences = []
    for threads in (1, 4, 16):
        scene = ComplexReadouts(random_seed=seed & 0xFFFF_FFFF)
        receipt = scene.render(root / f"threads-{threads}", threads=threads)
        paths = sorted(receipt.destination.glob("*.png"))
        assert len(paths) == receipt.frame_count == 6
        assert receipt.certified is False
        assert receipt.resolution == (192, 108) and receipt.fps == 8
        sequence = [path.read_bytes() for path in paths]
        assert len(set(sequence[:5])) == 5, "complex readouts jumped instead of animating"
        sequences.append(sequence)
    del scene
    assert sequences[0] == sequences[1] == sequences[2], "thread count changed complex readout pixels"
    images = [_decode(payload) for payload in sequences[0]]
    for image in images:
        rgb = image[:, :, :3].astype(np.int16)
        assert np.count_nonzero((rgb[:, :, 0] > rgb[:, :, 1] + 20) & (rgb[:, :, 0] > rgb[:, :, 2] + 20)) > 10
        assert np.count_nonzero((rgb[:, :, 2] > rgb[:, :, 0] + 20)) > 10
        assert np.all(image[:, :, 3] == 255)
    assert not np.array_equal(images[0][:54], images[-1][:54]), "complex formula did not change pixels"
    assert not np.array_equal(images[0][54:], images[-1][54:]), "complex matrix did not change pixels"

    sentinel = ValueError("invalid live complex component")
    class BrokenReadout(ComplexReadouts):
        def construct(self):
            super().construct()
            try:
                self.readout.set_value(complex(0, float("nan")))
            except ValueError:
                raise sentinel from None
            raise AssertionError("nonfinite imaginary component reached output")
    failed = root / "failed-generation"
    try:
        BrokenReadout().render(failed, threads=4)
    except ValueError as error:
        assert error is sentinel
    else:
        raise AssertionError("invalid complex render published output")
    finally:
        # `raise sentinel from None` suppresses display, not ownership of the
        # original exception. Its traceback owns the failed Scene and every
        # native proxy in it. Break both traceback edges on the owner thread;
        # do not weaken the embedding host's native-proxy teardown assertion.
        sentinel.__traceback__ = None
        sentinel.__context__ = None
        del sentinel
    del BrokenReadout
    assert not failed.exists()
    return sequences[0][0], sequences[0][-1], 6, 3, 1


if __name__ in ("__main__", "<run_path>"):
    root = Path(tempfile.mkdtemp(prefix="fmn-complex-readouts-"))
    first, last, frames, threads, refusals = render_complex_readouts(root)
    print(f"complex readout render: {frames} frames, {threads} thread counts, {refusals} failed generation")
    print(f"first_sha256={hashlib.sha256(first).hexdigest()}")
    print(f"last_sha256={hashlib.sha256(last).hexdigest()}")
    print(f"artifacts={root}")
