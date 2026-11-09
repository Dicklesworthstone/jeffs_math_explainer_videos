"""Installed-extension tracker animation acceptance; no engine substitutes."""
import numpy as np
import manimlib as m


def path(start, end, alpha):
    return (1 - alpha) * start + alpha * end


def test_scalar_callback_drives_dependent_geometry():
    scene, tracker, dot = m.Scene(), m.ValueTracker(0), m.Dot()
    observed = []
    def update(obj):
        value = float(tracker.get_value())
        observed.append(value)
        obj.move_to(value * m.RIGHT)
    dot.add_updater(update)
    scene.add(tracker, dot)
    scene.play(m.Transform(tracker, m.ValueTracker(3), path_func=path),
               run_time=.2, rate_func=m.linear)
    assert any(0 < value < 3 for value in observed), observed
    assert np.isclose(tracker.get_value(), 3)
    assert np.isclose(dot.get_center()[0], 3)


def test_complex_callback_updates_real_and_imaginary_native_lanes():
    scene, tracker = m.Scene(), m.ComplexValueTracker(1 + 2j)
    scene.add(tracker)
    scene.play(m.Transform(tracker, m.ComplexValueTracker(5 - 6j), path_func=path),
               run_time=.2, rate_func=m.linear)
    assert np.isclose(tracker.get_value(), 5 - 6j)
    assert np.isclose(complex(*tracker._tracker_complex_value()), 5 - 6j)


def test_exponential_callback_uses_geometric_interpolation():
    tracker = m.ExponentialValueTracker(1)
    animation = m.Transform(tracker, m.ExponentialValueTracker(81), path_func=path,
                            rate_func=m.linear)
    animation.begin()
    animation.interpolate(.5)
    assert np.isclose(tracker.get_value(), 9, rtol=2e-13), tracker.get_value()
    animation.finish()
    assert np.isclose(tracker.get_value(), 81, rtol=2e-13)


def test_native_then_callback_uses_actual_native_endpoint():
    scene, tracker = m.Scene(), m.ValueTracker(0)
    scene.add(tracker)
    scene.play(tracker.animate.set_value(4), run_time=.1, rate_func=m.linear)
    values = []
    class Observe(m.Transform):
        def interpolate_submobject(self, current, start, target, alpha):
            result = super().interpolate_submobject(current, start, target, alpha)
            values.append(float(current.get_value()))
            return result
    scene.play(Observe(tracker, m.ValueTracker(8)), run_time=.1, rate_func=m.linear)
    assert np.isclose(values[0], 4), values
    assert any(4 < value < 8 for value in values), values
    assert np.isclose(tracker.get_value(), 8)


def test_builder_endpoint_fraction_updates_native_tracker():
    scene, tracker = m.Scene(), m.ValueTracker(2)
    scene.add(tracker)
    scene.play(tracker.animate(final_alpha_value=.25).set_value(10),
               run_time=.1, rate_func=m.linear)
    assert np.isclose(tracker.get_value(), 4)


def test_group_callback_animates_nested_tracker_families():
    first, second = m.ValueTracker(0), m.ComplexValueTracker(0)
    source = m.Group(first, m.Group(second))
    target = m.Group(m.ValueTracker(6), m.Group(m.ComplexValueTracker(2 + 4j)))
    scene = m.Scene()
    scene.add(source)
    scene.play(m.Transform(source, target, path_func=path), run_time=.1, rate_func=m.linear)
    assert np.isclose(first.get_value(), 6)
    assert np.isclose(second.get_value(), 2 + 4j)


def test_value_uniform_lock_blocks_callback_tracker_write():
    tracker, start, end = m.ValueTracker(7), m.ValueTracker(0), m.ValueTracker(10)
    tracker.locked_uniform_keys.add("value")
    tracker.interpolate(start, end, .5)
    assert tracker.get_value() == 7
    tracker.locked_uniform_keys.clear()
    tracker.interpolate(start, end, .5)
    assert tracker.get_value() == 5


def test_callback_tracker_retains_float64_precision():
    tracker = m.ValueTracker(0)
    start, end = 1 + 2**-40, 1 + 2**-38
    tracker.interpolate(m.ValueTracker(start), m.ValueTracker(end), .5)
    expected = (start + end) / 2
    assert float(tracker.get_value()) == expected
    assert float(tracker.get_value()) != float(np.float32(expected))


def test_vector_interpolation_reads_aliasing_endpoints_before_writing():
    for receiver in ("start", "end", "other"):
        for alpha in (0., .25, .5, 1.):
            start, end = m.ValueTracker([0., 2.]), m.ValueTracker([8., 10.])
            tracker = {"start": start, "end": end, "other": m.ValueTracker([-5., -6.])}[receiver]
            tracker.interpolate(start, end, alpha)
            expected = np.array([0., 2.]) + alpha * np.array([8., 8.])
            np.testing.assert_array_equal(tracker.get_value(), expected)
            assert tracker._tracker_value() == expected[0]


def test_vector_interpolation_freezes_endpoint_before_authored_path():
    start, end = m.ValueTracker([0., 2.]), m.ValueTracker([8., 10.])
    tracker = m.ValueTracker([-1., -2.])
    for mob in (tracker, start, end):
        mob.set_points([[0., 0., 0.]])
    calls = []
    def mutate_endpoint(first, last, alpha):
        calls.append(alpha)
        start.set_value([100., 200.])
        return path(first, last, alpha)
    tracker.interpolate(start, end, .5, path_func=mutate_endpoint)
    assert calls == [.5]
    np.testing.assert_array_equal(tracker.get_value(), [4., 6.])


def test_vector_bad_shape_refuses_before_record_or_uniform_mutation():
    tracker = m.ValueTracker([1., 2., 3.])
    start, end = m.ValueTracker([0., 2.]), m.ValueTracker([8., 10., 12.])
    for index, mob in enumerate((tracker, start, end)):
        mob.set_points([[float(index), 0., 0.]])
    before, value = tracker.data.copy(), tracker.get_value().copy()
    try:
        tracker.interpolate(start, end, .5)
    except ValueError:
        pass
    else:
        raise AssertionError("incompatible tracker width was accepted")
    np.testing.assert_array_equal(tracker.data, before)
    np.testing.assert_array_equal(tracker.get_value(), value)
    assert tracker._tracker_value() == value[0]


def test_vector_wrong_encoding_refuses_before_geometry_mutation():
    tracker = m.ValueTracker([1., 2.])
    tracker.set_points([[1., 0., 0.]])
    before = tracker.data.copy()
    for end in (m.ComplexValueTracker(3+4j), m.ExponentialValueTracker(4)):
        try:
            tracker.interpolate(m.ValueTracker([0., 2.]), end, .5)
        except TypeError:
            pass
        else:
            raise AssertionError("incompatible tracker encoding was accepted")
        np.testing.assert_array_equal(tracker.data, before)
        np.testing.assert_array_equal(tracker.get_value(), [1., 2.])


def test_vector_destination_width_follows_endpoint_broadcasting():
    tracker = m.ValueTracker([-1., -2., -3.])
    tracker.interpolate(m.ValueTracker([0., 2.]), m.ValueTracker([8., 10.]), .5)
    np.testing.assert_array_equal(tracker.get_value(), [4., 6.])
    tracker.interpolate(m.ValueTracker(1.), m.ValueTracker(5.), .5)
    assert tracker.get_value() == 3.
    assert tracker._tracker_value() == 3.


def test_vector_interpolation_retains_float64_and_broadcasts_scalars():
    start = 1 + 2**-40
    tracker, scalar = m.ValueTracker([start, 2.]), m.ValueTracker(3.)
    tracker.interpolate(tracker, scalar, .5)
    np.testing.assert_array_equal(tracker.get_value(), [(start+3.)/2, 2.5])
    assert tracker.get_value()[0] != float(np.float32(tracker.get_value()[0]))


def test_vector_scalar_endpoint_uses_native_state_not_stale_mirror():
    scalar = m.ValueTracker(0)
    scalar._set_tracker_value(8.)
    tracker = m.ValueTracker([0., 2.])
    tracker.interpolate(tracker, scalar, .5)
    np.testing.assert_array_equal(tracker.get_value(), [4., 5.])


def test_locked_vector_does_not_read_or_validate_endpoints():
    tracker = m.ValueTracker([1., 2.])
    tracker.locked_uniform_keys.add("value")
    tracker.interpolate(m.ValueTracker([3., 4.]), m.ValueTracker([5., 6.]), .5)
    np.testing.assert_array_equal(tracker.get_value(), [1., 2.])
    assert tracker._tracker_value() == 1.


def test_vector_callback_alias_drives_actual_scene_geometry():
    scene, tracker, dot = m.Scene(), m.ValueTracker([0., 2.]), m.Dot()
    target = m.ValueTracker([8., 10.])
    observed = []
    dot.add_updater(lambda obj: obj.move_to([*tracker.get_value(), 0.]))
    scene.add(tracker, dot)
    def update(current, alpha):
        before = current.get_value().copy()
        current.interpolate(current, target, .5)
        expected = .5 * (before + [8., 10.])
        np.testing.assert_array_equal(current.get_value(), expected)
        observed.append(expected)
    scene.play(m.UpdateFromAlphaFunc(tracker, update), run_time=.1, rate_func=m.linear)
    assert observed
    np.testing.assert_allclose(dot.get_center()[:2], observed[-1], atol=1e-6)


def test_scalar_receiver_snapshots_vector_endpoints_before_authored_path():
    start, end = m.ValueTracker([0., 2.]), m.ValueTracker([8., 10.])
    tracker = m.ValueTracker(-1.)
    for mob in (tracker, start, end):
        mob.set_points([[0., 0., 0.]])
    def mutate_endpoint(first, last, alpha):
        start.set_value([100., 200.])
        return path(first, last, alpha)
    tracker.interpolate(start, end, .5, path_func=mutate_endpoint)
    np.testing.assert_array_equal(tracker.get_value(), [4., 6.])
    assert tracker._tracker_value() == 4.


def test_scalar_receiver_reads_live_vector_uniform_not_stale_native_lane():
    start, end = m.ValueTracker([0., 2.]), m.ValueTracker([8., 10.])
    start.get_value()[:] = [20., 30.]
    tracker = m.ValueTracker(-1.)
    tracker.interpolate(start, end, .5)
    np.testing.assert_array_equal(tracker.get_value(), [14., 20.])
    assert tracker._tracker_value() == 14.


def test_scalar_promotion_bad_width_refuses_before_geometry_changes():
    tracker, start, end = m.ValueTracker(7.), m.ValueTracker([0., 2.]), m.ValueTracker([1., 2., 3.])
    for index, mob in enumerate((tracker, start, end)):
        mob.set_points([[float(index), 0., 0.]])
    before = tracker.data.copy()
    try:
        tracker.interpolate(start, end, .5)
    except ValueError:
        pass
    else:
        raise AssertionError("incompatible promotion widths were accepted")
    np.testing.assert_array_equal(tracker.data, before)
    assert tracker.get_value() == tracker._tracker_value() == 7.


def test_bound_tracker_can_promote_demote_and_reenter_vector_callback_playback():
    scene, tracker, dot = m.Scene(), m.ValueTracker(0.), m.Dot()
    seen = []
    def update(current):
        value = np.asarray(tracker.get_value()).reshape(-1)
        assert tracker._tracker_value() == value[0]
        current.set_x(float(value[0]))
        seen.append(value.copy())
    dot.add_updater(update)
    scene.add(tracker, dot)
    for low, high in (([1., 3.], [5., 7.]), (2., 6.), ([0., 2., 4.], [8., 10., 12.])):
        start, end = m.ValueTracker(low), m.ValueTracker(high)
        scene.play(m.UpdateFromAlphaFunc(tracker, lambda current, alpha:
                   current.interpolate(start, end, alpha)), run_time=.1, rate_func=m.linear)
        np.testing.assert_array_equal(tracker.get_value(), high)
    assert {len(value) for value in seen} == {1, 2, 3}
    assert dot.get_x() == 8.


def test_vector_alias_frames_match_independent_native_controls():
    from pathlib import Path
    import tempfile
    from fmn_python import render_scene

    root = Path(tempfile.mkdtemp(prefix="fmn-tracker-alias-"))
    outputs = []
    for threads in (1, 4, 16):
        pair = []
        for independent in (False, True):
            class TrackerScene(m.Scene):
                default_camera_config = dict(resolution=(96, 54), fps=8)
                def construct(self):
                    tracker, end = m.ValueTracker([-2., -1.]), m.ValueTracker([2., 1.])
                    square = m.Square(side_length=.6, fill_opacity=1, stroke_width=0)
                    square.add_updater(lambda mob: mob.move_to([*tracker.get_value(), 0.]))
                    self.add(tracker, square)
                    def update(current, alpha):
                        if independent:
                            current.set_value(.75 * current.get_value() + .25 * end.get_value())
                        else:
                            current.interpolate(current, end, .25)
                    self.play(m.UpdateFromAlphaFunc(tracker, update), run_time=.5,
                              rate_func=m.linear)
            result = render_scene(TrackerScene, root / f"{threads}-{independent}.y4m",
                                  threads=threads)
            assert result.frame_count == 4
            pair.append(result.destination.read_bytes())
        assert pair[0] == pair[1], "aliased tracker changed native rendered frames"
        header, payload = pair[0].split(b"\n", 1)
        assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2"
        size = 96 * 54 * 3 // 2
        assert len(payload) == 4 * (6 + size)
        frames = [payload[i+6:i+6+size] for i in range(0, len(payload), 6+size)]
        assert len(set(frames)) > 1, "tracker motion never reached the renderer"
        assert all(max(frame[:96*54]) > 100 for frame in frames), "blank render control"
        outputs.append(pair[0])
    assert outputs[0] == outputs[1] == outputs[2]


count = 0
for name, function in tuple(globals().items()):
    if name.startswith("test_") and callable(function):
        function()
        count += 1
print(f"tracker interpolation: {count} installed-extension cases passed")
