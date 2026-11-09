"""Native macro geometry, matching and live numeric spans in published frames."""
from pathlib import Path
import tempfile

import manimlib as m

PREAMBLE = r"\newcommand{\sq}[1]{#1^2}\newcommand{\RR}{\mathbb{R}}"


class PreambleScene(m.Scene):
    def construct(self):
        label = m.TexText(r"area $\sq{x}$ in $\RR$", additional_preamble=PREAMBLE,
                          template="empty", color=m.BLUE).scale(2).shift(m.UP)
        first = m.Tex(r"\sq{x}+1.00", additional_preamble=PREAMBLE,
                      isolate=[r"\sq{x}", "1.00"], color=m.RED).scale(3).shift(m.DOWN)
        second = m.Tex(r"\sq{x}+2.00", additional_preamble=PREAMBLE,
                       isolate=[r"\sq{x}", "2.00"], color=m.RED).scale(3).shift(m.DOWN)
        self.add(label, first)
        self.wait(1 / 8)
        self.play(m.TransformMatchingTex(first, second), run_time=1 / 2)
        number = second.make_number_changeable("2.00")
        self.play(m.ChangeDecimalToValue(number, 4), run_time=1 / 4)
        self.wait(1 / 8)
        assert list(self.mobjects) == [label, second]
        assert number.get_value() == 4
        assert second.get_part_by_tex(r"\decimalmob")[0] is number
        assert second.get_part_by_tex("x").family_members_with_points()


def render_tex_preamble(destination, seed=0):
    root = Path(destination)
    root.mkdir(parents=True, exist_ok=True)
    sequences = []
    for threads in (1, 4, 16):
        scene = PreambleScene(random_seed=seed & 0xFFFF_FFFF)
        receipt = scene.render(root / f"threads-{threads}", resolution=(192, 108), fps=8, threads=threads)
        assert receipt.frame_count == 8 and receipt.certified is False
        frames = [path.read_bytes() for path in sorted(receipt.destination.glob("*.png"))]
        assert len(frames) == 8 and len(set(frames)) >= 4
        sequences.append(frames)
    del scene
    assert sequences[0] == sequences[1] == sequences[2], "native preamble frames depend on threads"

    class FailedPreamble(m.Scene):
        def construct(self):
            self.add(m.Tex(r"\loop", additional_preamble=r"\newcommand{\loop}{\loop}"))
    failed = root / "failed"
    try:
        FailedPreamble().render(failed, resolution=(192, 108), fps=8)
    except ValueError as error:
        assert "loop" in str(error)
    else:
        raise AssertionError("recursive macro was published")
    assert not failed.exists()
    return sequences[0][0], sequences[0][-1], 8, 3, 1


if __name__ in ("__main__", "<run_path>"):
    root = Path(tempfile.mkdtemp(prefix="fmn-tex-preamble-"))
    first, last, frames, threads, refusals = render_tex_preamble(root)
    assert first != last
    print(f"native preamble render: {frames} frames, {threads} thread counts, {refusals} failed generation")
    print(f"artifacts={root}")
