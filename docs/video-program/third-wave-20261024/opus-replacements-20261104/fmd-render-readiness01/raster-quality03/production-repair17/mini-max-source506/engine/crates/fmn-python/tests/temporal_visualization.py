"""Installed-wheel flow/trace acceptance through native geometry and output.

No storage or animation doubles are used here. The companion protocol suites
exercise adapter control flow; this script requires the built native extension.
"""
import hashlib
from pathlib import Path
import tempfile
from types import MethodType

import numpy as np
import manimlib as m
from fmn_python import render_scene
from manimlib.mobject.vector_field import AnimatedStreamLines as QualifiedFlow
from manimlib.mobject.changing import TracedPath as QualifiedTrace
from manimlib.mobject.changing import TracingTail as QualifiedTail


def stream_lines():
    plane = m.NumberPlane(x_range=(-1, 1, 1), y_range=(-1, 1, 1))

    def field(rows):
        result = np.zeros_like(rows, dtype=float)
        result[:, 0] = 1.0
        return result

    lines = m.StreamLines(
        field, plane, density=1, noise_factor=0, solution_time=1.0,
        dt=.05, arc_len=2, n_samples_per_line=12, cutoff_norm=15,
        color_by_magnitude=False, stroke_color=m.WHITE, stroke_width=12,
    )
    assert len(lines) > 1
    assert len(lines) == len(lines._stream_virtual_times)
    assert any(time > 0 for time in lines._stream_virtual_times)
    return lines


def native_streamline_identity():
    assert m.AnimatedStreamLines is QualifiedFlow
    lines = stream_lines()
    original = list(lines)
    points = [line.get_points().copy() for line in lines]
    flow = m.AnimatedStreamLines(lines, lag_range=0)
    assert flow.stream_lines is lines
    assert all(actual is expected for actual, expected in zip(flow, original))
    for line, expected_points, virtual_time in zip(flow, points, lines._stream_virtual_times):
        assert isinstance(line.anim, m.VShowPassingFlash)
        assert line.anim.mobject is line
        assert line.virtual_time == virtual_time
        assert np.isfinite(line.get_points()).all()
        np.testing.assert_array_equal(line.get_points(), expected_points)
    flow.clear_updaters()


def flash_protocol_equivalence():
    lines = stream_lines()
    controls = [line.copy() for line in lines]
    calls = []

    class Rate:
        __hash__ = None

        def __call__(self, alpha):
            calls.append(alpha)
            return alpha * alpha

    rate = Rate()
    config = dict(rate_func=rate, taper_width=0, time_width=.8, remover=False)
    flow = m.AnimatedStreamLines(lines, lag_range=0, line_anim_config=config)
    flow.update(.37)
    assert calls and any(0 < value < 1 for value in calls)
    for line, control in zip(lines, controls):
        flash = m.VShowPassingFlash(control, run_time=line.anim.get_run_time(), **config)
        flash.begin()
        flash.interpolate((line.time % flash.get_run_time()) / flash.get_run_time())
        np.testing.assert_allclose(line.get_stroke_widths(), control.get_stroke_widths(), atol=1e-6)
        np.testing.assert_array_equal(line.get_points(), control.get_points())
        flash.abort()
    flow.clear_updaters()


def signed_phases_stationary_lines_and_lag_reproducibility():
    first, second = stream_lines(), stream_lines()
    # Zero virtual time is legal controller input even when this particular
    # nonzero field's native integrator produced a nondegenerate line.
    first[0].virtual_time = second[0].virtual_time = 0.0
    a, b = m.AnimatedStreamLines(first), m.AnimatedStreamLines(second)
    np.testing.assert_array_equal(a._line_times, b._line_times)
    assert all(time <= 0 for time in a._line_times)
    a.update(.13)
    b.update(.13)
    for left, right in zip(a, b):
        np.testing.assert_array_equal(left.get_stroke_widths(), right.get_stroke_widths())
        assert np.isfinite(left.get_stroke_widths()).all()
    a.clear_updaters()
    b.clear_updaters()


def scene_updaters_suspension_and_cleanup():
    lines = stream_lines()
    original_widths = [line.get_stroke_widths().copy() for line in lines]
    flow = m.AnimatedStreamLines(lines, lag_range=0, line_anim_config=dict(time_width=.5))
    observed = []
    flow.add_updater(lambda current, dt: observed.append(dt), call=False)
    scene = m.Scene()
    scene.add(flow)
    scene.wait(.2)
    assert observed and any(dt > 0 for dt in observed)
    assert all(line.time > 0 for line in lines)
    before = list(flow._line_times)
    flow.suspend_updating()
    scene.wait(.2)
    assert flow._line_times == before
    flow.resume_updating(call_updater=False)
    scene.wait(.2)
    assert all(after > before for after, before in zip(flow._line_times, before))
    flow.clear_updaters()
    for line, width in zip(lines, original_widths):
        np.testing.assert_allclose(line.get_stroke_widths(), width, atol=1e-6)
    assert any(root is flow for root in scene.mobjects)


def failure_releases_owned_flashes():
    lines = stream_lines()
    widths = [line.get_stroke_widths().copy() for line in lines]
    # The preexisting suspension on this line belongs to the author, not us.
    lines[-1].suspend_updating()
    flow = m.AnimatedStreamLines(lines, lag_range=0, line_anim_config=dict(
        time_width=.6, suspend_mobject_updating=True,
    ))

    def fail(self, alpha):
        raise RuntimeError("authored flow interpolation failed")

    lines[0].anim.interpolate = MethodType(fail, lines[0].anim)
    try:
        flow.update(.2)
    except RuntimeError as error:
        assert str(error) == "authored flow interpolation failed"
    else:
        raise AssertionError("authored error was swallowed")
    assert flow._streamline_controller.closed and not flow.updaters
    for line, original in zip(lines, widths):
        np.testing.assert_allclose(line.get_stroke_widths(), original, atol=1e-6)
    assert not lines[0]._is_updating_suspended()
    assert lines[-1]._is_updating_suspended()


def copied_flow_cannot_drive_source():
    lines = stream_lines()
    flow = m.AnimatedStreamLines(lines, lag_range=0)
    copied = flow.copy()
    before = list(flow._line_times)
    widths = [line.get_stroke_widths().copy() for line in flow]
    copied.update(.3)
    copied.clear_updaters()
    assert flow._line_times == before and not flow._streamline_controller.closed
    for line, width in zip(flow, widths):
        np.testing.assert_array_equal(line.get_stroke_widths(), width)
    flow.update(.3)
    assert any(after > before for after, before in zip(flow._line_times, before))
    flow.clear_updaters()


def native_trace_windows_and_spacing():
    assert m.TracedPath is QualifiedTrace
    paths = []
    for steps in ([.1] * 10, [.1, .3, .1, .5]):
        position = np.zeros(3)
        trace = m.TracedPath(lambda: position, time_traced=.45, time_per_anchor=.2)
        time = 0
        for dt in steps:
            time += dt
            position[:] = (time, 2 * time, 0)
            trace.update(dt)
        np.testing.assert_allclose(trace.get_start(), [.55, 1.1, 0], atol=2e-6)
        np.testing.assert_allclose(trace.get_end(), [1, 2, 0], atol=2e-6)
        np.testing.assert_allclose(trace.get_arc_length(), .45 * np.sqrt(5), atol=2e-5)
        assert trace.get_num_points() % 2 == 1
        assert len(trace.traced_points) == 4
        paths.append(trace.get_points().copy())
    np.testing.assert_allclose(*paths, atol=2e-6)


def native_tail_and_copy():
    assert m.TracingTail is QualifiedTail
    point = m.Dot(m.ORIGIN)
    tail = m.TracingTail(point, time_traced=.4, time_per_anchor=.1)
    assert isinstance(tail, m.TracedPath)
    point.shift(m.RIGHT)
    tail.update(.1)
    np.testing.assert_allclose(tail.get_start(), m.ORIGIN, atol=1e-6)
    np.testing.assert_allclose(tail.get_end(), m.RIGHT, atol=1e-6)
    assert tail.get_stroke_widths()[0] < tail.get_stroke_widths()[-1]
    assert tail.get_stroke_opacities()[0] < tail.get_stroke_opacities()[-1]
    copy = tail.copy()
    original = tail.get_points().copy()
    point.shift(m.RIGHT)
    copy.update(.1)
    np.testing.assert_array_equal(tail.get_points(), original)
    np.testing.assert_allclose(copy.get_end(), 2 * m.RIGHT, atol=1e-6)
    assert copy.time > tail.time


def short_window_and_native_failure_boundary():
    position = np.zeros(3)
    trace = m.TracedPath(lambda: position, time_traced=.01, time_per_anchor=.5)
    for time, dt in ((.1, .1), (.12, .02)):
        position[0] = time
        trace.update(dt)
    np.testing.assert_allclose(trace.get_start(), [.11, 0, 0], atol=1e-6)
    np.testing.assert_allclose(trace.get_end(), [.12, 0, 0], atol=1e-6)
    before, history = trace.get_points().copy(), trace._trace_anchors
    position[0] = np.nan
    try:
        trace.update(.2)
    except ValueError:
        pass
    else:
        raise AssertionError("nonfinite trace source was accepted")
    np.testing.assert_array_equal(trace.get_points(), before)
    assert trace._trace_anchors == history
    np.testing.assert_allclose(trace.time, .12, atol=1e-15)


def luma_frames(path):
    payload = Path(path).read_bytes()
    header, body = payload.split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H64 F8:1 Ip A1:1 C420mpeg2", header
    size = 96 * 64 * 3 // 2
    assert len(body) % (size + 6) == 0
    result = []
    for offset in range(0, len(body), size + 6):
        assert body[offset:offset + 6] == b"FRAME\n"
        result.append(np.frombuffer(body[offset + 6:offset + 6 + 96 * 64],
                                    dtype=np.uint8).reshape(64, 96))
    return result


def render_pair(scene_type):
    with tempfile.TemporaryDirectory(prefix="fmn-temporal-output-") as root:
        one = render_scene(scene_type, Path(root) / "one.y4m", threads=1)
        four = render_scene(scene_type, Path(root) / "four.y4m", threads=4)
        for result in (one, four):
            data = result.destination.read_bytes()
            assert result.bytes == len(data)
            assert result.digest == hashlib.sha256(data).hexdigest()
            assert result.frame_count == 6 and not result.certified
        assert one.destination.read_bytes() == four.destination.read_bytes()
        frames = luma_frames(one.destination)
        assert len(frames) == 6
        assert len({frame.tobytes() for frame in frames}) > 1
        assert any((frame > 80).sum() > 5 for frame in frames)
        return frames


def rendered_flow_motion():
    class FlowScene(m.Scene):
        default_camera_config = dict(resolution=(96, 64), fps=8)

        def construct(self):
            lines = stream_lines().set_stroke(width=30)
            flow = m.AnimatedStreamLines(lines, lag_range=0, line_anim_config=dict(
                time_width=.5, taper_width=0, rate_func=m.linear,
            ))
            self.add(flow)
            self.wait(.75)
            flow.clear_updaters()

    render_pair(FlowScene)


def rendered_trail_motion():
    class TrailScene(m.Scene):
        default_camera_config = dict(resolution=(96, 64), fps=8)

        def construct(self):
            point = m.Dot(3 * m.LEFT, radius=.2, color=m.WHITE)
            point.add_updater(lambda mob, dt: mob.shift(8 * dt * m.RIGHT))
            trail = m.TracingTail(point, time_traced=.4, time_per_anchor=.1,
                                  stroke_width=(0, 24), stroke_color=m.WHITE)
            self.add(point, trail)
            self.wait(.75)

    frames = render_pair(TrailScene)
    centers = [np.nonzero(frame > 160)[1].mean() for frame in frames]
    assert all(np.isfinite(centers))
    assert centers[-1] - centers[0] > 12


CASES = (
    native_streamline_identity, flash_protocol_equivalence,
    signed_phases_stationary_lines_and_lag_reproducibility,
    scene_updaters_suspension_and_cleanup, failure_releases_owned_flashes,
    copied_flow_cannot_drive_source, native_trace_windows_and_spacing,
    native_tail_and_copy, short_window_and_native_failure_boundary,
    rendered_flow_motion, rendered_trail_motion,
)
for case in CASES:
    case()
print(f"native temporal visualization acceptance: {len(CASES)} cases passed")
