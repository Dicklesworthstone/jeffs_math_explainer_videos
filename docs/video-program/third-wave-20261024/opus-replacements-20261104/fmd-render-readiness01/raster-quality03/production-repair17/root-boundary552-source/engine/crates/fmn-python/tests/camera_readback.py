"""Actual extension/Lumen acceptance, not a mocked renderer."""
import numpy as np
import manimlib as m


def run_camera_readback_acceptance():
    camera = m.Camera(resolution=(96, 64), background_opacity=0)
    camera.clear()
    empty = camera.get_pixel_array()
    assert empty.shape == (64, 96, 4) and empty.dtype == np.uint8
    assert not empty.any()
    top = m.Square(side_length=1.0, fill_color=m.RED, fill_opacity=1, stroke_width=0).shift(m.UP * 2)
    before = np.array(top.get_points(), copy=True)
    assert camera.capture(top) is None
    image = camera.get_pixel_array()
    ys, xs = np.where(image[:, :, 3] > 128)
    assert len(ys) > 5 and ys.mean() < 32, "camera projection must produce top-row-first images"
    assert not top._is_bound(), "readback must not adopt detached objects"
    np.testing.assert_array_equal(before, top.get_points())
    image[:] = 0
    assert camera.get_pixel_array().any(), "returned pixel arrays must own their data"
    frozen = camera.get_pixel_array()
    top.shift(m.DOWN * 4)
    np.testing.assert_array_equal(frozen, camera.get_pixel_array())
    camera.capture(top)
    assert np.where(camera.get_pixel_array()[:, :, 3] > 128)[0].mean() > 32

    # A point-free group and shared children must retain their Python topology.
    group = m.Group(top, m.Circle(radius=.5, fill_opacity=1, stroke_width=0))
    camera.capture(group)
    grouped = camera.get_pixel_array()
    camera.capture(*group.submobjects)
    np.testing.assert_array_equal(grouped, camera.get_pixel_array())
    left = m.Group(top)
    right = m.Group(top)
    camera.capture(left, right)
    assert left.submobjects[0] is right.submobjects[0] is top
    assert not left._is_bound() and not right._is_bound()

    # Preserve explicit root order, even if z_index differs from that order.
    red = m.Square(side_length=2, fill_color=m.RED, fill_opacity=1, stroke_width=0).set_z_index(10)
    blue = m.Square(side_length=2, fill_color=m.BLUE, fill_opacity=1, stroke_width=0).set_z_index(-10)
    camera.capture(red, blue)
    pixel = camera.get_pixel_array()[32, 48]
    assert int(pixel[2]) > int(pixel[0])
    camera.capture(blue, red)
    pixel = camera.get_pixel_array()[32, 48]
    assert int(pixel[0]) > int(pixel[2])

    scene = m.Scene()
    scene.add(red)
    ticks = []
    red.add_updater(lambda obj, dt: ticks.append(dt), call=False)
    time, roots = scene.get_time(), tuple(scene.mobjects)
    camera.capture(red, blue)
    assert scene.get_time() == time and tuple(scene.mobjects) == roots and ticks == []
    assert red._scene is scene and not blue._is_bound()
    previous = camera.get_pixel_array()
    # Failed capture must leave the last successful frame available.
    try:
        camera.capture(object())
    except TypeError:
        pass
    else:
        raise AssertionError("non-Mobject capture must refuse")
    np.testing.assert_array_equal(previous, camera.get_pixel_array())
    for bad in (0, 97):
        camera.capture_threads = bad
        try:
            camera.capture(red)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid thread count must refuse")
        np.testing.assert_array_equal(previous, camera.get_pixel_array())
    camera.capture_threads = 1
    camera.capture(red)
    serial = camera.get_pixel_array()
    for threads in (4, 16):
        camera.capture_threads = threads
        camera.capture(red)
        np.testing.assert_array_equal(serial, camera.get_pixel_array())

    # Geometry writes through an exported NumPy view must affect the next read.
    points = red.get_points()
    points[:, 0] += 2
    camera.capture(red)
    assert not np.array_equal(serial, camera.get_pixel_array())
    assert ticks == [] and scene.get_time() == time
    camera.frame.shift(m.RIGHT * 2)
    camera.capture(red)
    np.testing.assert_array_equal(serial, camera.get_pixel_array())

    # Direct calls cannot bypass graph validation, including disconnected cycles.
    for adjacency, roots in (([[0]], [0]), ([[1]], [0]), ([[]], [1])):
        try:
            m._CameraCapture(camera._core, camera.frame._core, (0., 0., 0., 0.),
                             (-10., 10., 10.), [red], adjacency, roots, 1)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid native graph must refuse")
    camera.clear()
    assert not camera.get_pixel_array().any()
    camera.reset_pixel_shape(48, 32)
    assert camera.get_pixel_array().shape == (32, 48, 4)
    assert ticks == []
    red.clear_updaters()
    print("camera readback: native pixels, family identity, view writes, camera pose, order, failures, threads 1/4/16 passed")


run_camera_readback_acceptance()
