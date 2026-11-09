"""Public native SceneConsole acceptance; no camera/storage/IPython doubles."""
from __future__ import annotations

import numpy as np
import manimlib as m
from fmn_python import SceneConsole


def run_console_acceptance():
    scene = m.Scene()
    scene.camera.reset_pixel_shape(96, 64)
    box = m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)
    scene.add(box)
    origin = box.get_points().copy()
    with SceneConsole(scene, {"box": box, "RIGHT": m.RIGHT, "m": m}) as console:
        console.run_cell("# move\nbox.shift(RIGHT)")
        first = scene.camera.get_pixel_array()
        assert first.shape == (64, 96, 4)
        assert np.any(first[:, :, :3] > 128), "native preview must contain visible geometry"
        console.run_cell("# later\nbox.shift(3 * RIGHT)")
        assert not np.array_equal(first, scene.camera.get_pixel_array())
        console.run_cell("# move\nbox.shift(RIGHT)")
        assert scene.mobjects[0] is box
        np.testing.assert_allclose(box.get_points(), origin + m.RIGHT, atol=1e-6)
        np.testing.assert_array_equal(first, scene.camera.get_pixel_array())
        assert list(console.checkpoint_manager.checkpoint_states) == ["# move"]

        # Scene time/play counts come from the native checkpoint, not a second
        # console clock. A retry must not accumulate another wait/play index.
        console.run_cell("# timing\nscene.wait(0.1)")
        timing = (scene.get_time(), scene.num_plays)
        assert timing[0] > 0
        console.run_cell("# timing\nscene.wait(0.1)")
        assert timing == (scene.get_time(), scene.num_plays)
        before = box.get_points().copy()
        keys = list(console.checkpoint_manager.checkpoint_states)
        try:
            console.run_cell("# move\ninvalid(")
        except SyntaxError:
            pass
        else:
            raise AssertionError("invalid cell was accepted")
        np.testing.assert_array_equal(before, box.get_points())
        assert list(console.checkpoint_manager.checkpoint_states) == keys

        # A failed cell retains actual effects; re-running its key restores the
        # original state. The same cancellation object crosses the front door.
        stop = KeyboardInterrupt("native console cancellation")
        console.namespace["stop"] = stop
        try:
            console.run_cell("# retry\nbox.shift(RIGHT)\nraise stop", skip=True)
        except KeyboardInterrupt as error:
            assert error is stop
        else:
            raise AssertionError("cell swallowed KeyboardInterrupt")
        console.run_cell("# retry\npass")
        np.testing.assert_array_equal(before, box.get_points())
        assert not scene.skip_animations

        console.clipboard = lambda: "# clipboard\nbox.shift(RIGHT)"
        console.checkpoint_paste()
        copied = box.get_points().copy()
        console.checkpoint_paste()
        np.testing.assert_array_equal(copied, box.get_points())
        for threads in (1, 4, 16):
            scene.camera.capture_threads = threads
            console.run_cell("# clipboard\nbox.shift(RIGHT)")
            pixels = scene.camera.get_pixel_array()
            if threads == 1:
                serial = pixels
            else:
                np.testing.assert_array_equal(serial, pixels)
    assert "_fmn_scene_console" not in vars(scene)
    assert console.scene is None and not console.checkpoint_manager.checkpoint_states

    # Restore a shared family without substituting proxy identities or losing
    # Python callbacks. The existing native SceneState owns these semantics.
    shared = m.Square()
    left, right = m.Group(shared), m.Group(shared)
    graph = m.Scene()
    graph.add(m.Group(left, right))
    events = []
    def updater(obj, dt):
        events.append((obj, dt))
    shared.add_updater(updater, call=False)
    with SceneConsole(graph, {"left": left, "right": right, "shared": shared}, capture=False) as editor:
        editor.run_cell("# graph\nleft.set_submobjects([])\nshared.clear_updaters()")
        editor.run_cell("# graph\npass")
        assert left[0] is right[0] is shared
        assert shared.updaters[0] is updater
        graph.update_mobjects(0)
        assert events and all(obj is shared for obj, dt in events)
    # The public compatibility method now executes an owned cell instead of
    # exposing a native snapshot byte string under the misleading paste name.
    interactive = m.InteractiveScene()
    shape = m.Square()
    interactive.add(shape)
    initial = shape.get_center().copy()
    widgets = ("selection_highlight", "selection_rectangle", "crosshair")
    assert all(not hasattr(interactive, name) for name in widgets)
    before_clock = m._portal_scene_clock(interactive)
    with SceneConsole(interactive, {"shape": shape, "RIGHT": m.RIGHT},
                      clipboard=lambda: "# public\nshape.shift(RIGHT)", capture=False):
        interactive.checkpoint_paste()
        interactive.checkpoint_paste()
        np.testing.assert_allclose(shape.get_center(), initial + m.RIGHT, atol=1e-6)
        assert all(not hasattr(interactive, name) for name in widgets)
        assert m._portal_scene_clock(interactive) == before_clock
        # Undo/redo share the same pre-setup checkpoint path, including time.
        interactive.save_state()
        interactive.wait(.125)
        later = m._portal_scene_clock(interactive)
        shape.shift(m.RIGHT)
        interactive.undo()
        assert m._portal_scene_clock(interactive) == before_clock
        np.testing.assert_allclose(shape.get_center(), initial + m.RIGHT, atol=1e-6)
        interactive.redo()
        assert m._portal_scene_clock(interactive) == later
        np.testing.assert_allclose(shape.get_center(), initial + 2 * m.RIGHT, atol=1e-6)
        assert all(not hasattr(interactive, name) for name in widgets)
    try:
        interactive.checkpoint_paste()
    except m._CapabilityError:
        pass
    else:
        raise AssertionError("unowned public paste did not refuse")
    return 7


if __name__ == "__main__":
    print("native scene console acceptance passed:", run_console_acceptance(), "cases")
