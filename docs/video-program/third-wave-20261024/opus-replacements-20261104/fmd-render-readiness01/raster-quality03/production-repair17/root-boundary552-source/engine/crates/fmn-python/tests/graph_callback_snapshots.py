"""Real-native callback copies, checkpoint recovery and transactional graph writes."""
import copy
from pathlib import Path
import tempfile
import unittest

import manimlib as m
import numpy as np
from fmn_python import render_scene


def capture(obj, mode, scene):
    if mode == "saved":
        obj.save_state()
        return obj.saved_state
    if mode == "target":
        obj.generate_target()
        return obj.target
    if mode == "checkpoint":
        return scene.get_state().mobjects_to_copies[obj]
    return {"copy": lambda: obj.copy(), "shallow": lambda: copy.copy(obj),
            "deep": lambda: copy.deepcopy(obj)}[mode]()


def recipe(kind, callback):
    axes = m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1), width=2, height=2)
    if kind == "curve":
        obj = m.ParametricCurve(lambda t: (t, callback(), 0),
                                t_range=(-1, 1, .25), use_smoothing=False)
    elif kind == "implicit":
        obj = m.ImplicitFunction(lambda x, y: y-callback(), x_range=(-1, 1),
                                 y_range=(-1, 1), min_depth=2, max_quads=31)
    else:
        obj = axes.get_graph(lambda x: callback(), bind=True, use_smoothing=False)
    return obj, axes


def refresh(obj, kind):
    return obj.update(0) if kind == "graph" else obj.init_points()


class CallbackSnapshotTests(unittest.TestCase):
    def test_copies_targets_saved_states_and_checkpoints_are_refreshable(self):
        for kind in ("curve", "implicit", "graph"):
            for mode in ("copy", "shallow", "deep", "saved", "target", "checkpoint"):
                with self.subTest(kind=kind, mode=mode):
                    holder, copies, level = [], [], [.125]
                    scene = m.Scene()
                    def sample():
                        if holder and not copies:
                            copies.append(capture(holder[0], mode, scene))
                        return level[0]
                    obj, axes = recipe(kind, sample)
                    scene.add(obj)
                    holder.append(obj)
                    original = obj.get_points().copy()
                    refresh(obj, kind)
                    published = obj.get_points().copy()
                    self.assertEqual(len(copies), 1)
                    np.testing.assert_array_equal(copies[0].get_points(), original)
                    level[0] = .375
                    refresh(copies[0], kind)
                    points = copies[0].get_points()
                    self.assertGreater(len(points), 2)
                    ys = np.array([axes.p2c(p)[1] for p in points]) if kind == "graph" else points[:, 1]
                    np.testing.assert_allclose(ys, .375, atol=1e-5)
                    np.testing.assert_array_equal(obj.get_points(), published)
                    self.assertIs(obj._scene, scene)
                    self.assertEqual(scene.get_time(), 0.)
                    self.assertNotIn("_fmn_function_graph_busy", vars(copies[0]))

    def test_restoring_a_checkpoint_taken_in_callback_does_not_restore_busy_state(self):
        for kind in ("curve", "implicit", "graph"):
            with self.subTest(kind=kind):
                holder, states, level = [], [], [.125]
                scene = m.Scene()
                def sample():
                    if holder and not states:
                        states.append(scene.get_state())
                    return level[0]
                obj, _ = recipe(kind, sample)
                scene.add(obj)
                holder.append(obj)
                refresh(obj, kind)
                initial = states[0].mobjects_to_copies[obj].get_points().copy()
                obj.shift(m.UP)
                states[0].restore_scene(scene)
                np.testing.assert_array_equal(obj.get_points(), initial)
                level[0] = .375
                refresh(obj, kind)
                self.assertFalse(np.array_equal(obj.get_points(), initial))
                self.assertIs(obj._scene, scene)

    def test_direct_reentry_is_still_refused_and_later_retry_works(self):
        for kind in ("curve", "implicit", "graph"):
            with self.subTest(kind=kind):
                holder, recurse = [], [True]
                def sample():
                    if holder and recurse[0]:
                        refresh(holder[0], kind)
                    return .25
                obj, _ = recipe(kind, sample)
                holder.append(obj)
                before = obj.data.copy()
                with self.assertRaisesRegex(RuntimeError, "reenter"):
                    refresh(obj, kind)
                np.testing.assert_array_equal(obj.data, before)
                recurse[0] = False
                refresh(obj, kind)

    def test_failed_or_cancelled_sampling_releases_invocation(self):
        for kind in ("curve", "implicit", "graph"):
            for error in (LookupError("sample"), KeyboardInterrupt("cancel"), SystemExit("stop")):
                with self.subTest(kind=kind, error=type(error).__name__):
                    holder, failure = [], [error]
                    def sample():
                        if holder and failure[0] is not None:
                            raise failure[0]
                        return .25
                    obj, _ = recipe(kind, sample)
                    holder.append(obj)
                    before = obj.data.copy()
                    with self.assertRaises(type(error)) as caught:
                        refresh(obj, kind)
                    self.assertIs(caught.exception, error)
                    np.testing.assert_array_equal(obj.data, before)
                    failure[0] = None
                    refresh(obj, kind)

    def test_graph_does_not_overwrite_callback_geometry_style_or_family_edits(self):
        for edit in (lambda obj: obj.shift(m.UP),
                     lambda obj: obj.set_stroke(m.RED, width=7),
                     lambda obj: obj.add(m.Dot())):
            with self.subTest(edit=edit):
                holder, expected = [], []
                def sample():
                    if holder and not expected:
                        edit(holder[0])
                        expected.append((holder[0].data.copy(), tuple(holder[0].submobjects)))
                    return .25
                obj, _ = recipe("graph", sample)
                scene = m.Scene()
                scene.add(obj)
                holder.append(obj)
                with self.assertRaisesRegex(RuntimeError, "changed during sampling"):
                    obj.update(0)
                np.testing.assert_array_equal(obj.data, expected[0][0])
                self.assertEqual(tuple(obj.submobjects), expected[0][1])
                obj.update(0)
                self.assertIs(obj._scene, scene)

    def test_graph_unbind_cannot_escape_an_active_invocation(self):
        holder = []
        def sample():
            if holder:
                axes.unbind_graph_from_func(holder[0])
            return .25
        obj, axes = recipe("graph", sample)
        holder.append(obj)
        before = obj.data.copy()
        with self.assertRaisesRegex(RuntimeError, "cannot unbind"):
            obj.update(0)
        np.testing.assert_array_equal(obj.data, before)
        holder.clear()
        axes.unbind_graph_from_func(obj)
        self.assertFalse(obj.updaters)

    def test_native_frames_match_without_callback_copy_and_at_four_threads(self):
        def scene_type(with_copy):
            class Demo(m.Scene):
                def construct(self):
                    copies, holder = [], []
                    def sample():
                        if with_copy and holder and not copies:
                            copies.append(holder[0].copy())
                            copies[0].update(0)
                        return .25
                    graph, _ = recipe("graph", sample)
                    holder.append(graph)
                    self.add(graph)
                    self.wait(.25)
                    if with_copy:
                        assert len(copies) == 1
            return Demo
        with tempfile.TemporaryDirectory(prefix="fmn-callback-snapshots-") as directory:
            frames = []
            for with_copy, threads in ((False, 1), (True, 1), (True, 4)):
                output = Path(directory) / f"graph-{with_copy}-{threads}.y4m"
                render_scene(scene_type(with_copy), output, format="y4m", resolution=(160, 90),
                             fps=8, threads=threads)
                data = output.read_bytes()
                header, body = data.split(b"\n", 1)
                self.assertTrue(header.startswith(b"YUV4MPEG2 "))
                self.assertEqual(len(body), 2 * (6 + 160 * 90 * 3 // 2))
                self.assertEqual(body[:6], b"FRAME\n")
                self.assertGreater(np.ptp(np.frombuffer(body[6:6+160*90], dtype=np.uint8)), 0)
                frames.append(data)
            self.assertEqual(frames[0], frames[1])
            self.assertEqual(frames[1], frames[2])


if __name__ == "__main__":
    unittest.main(verbosity=2)
