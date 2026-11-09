"""Native dynamic-field copies, private paint routing and callback recovery."""
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


def axes():
    return m.Axes(x_range=(-1, 1, 1), y_range=(-1, 1, 1), width=2, height=2)


def horizontal(rows):
    return np.tile([1., 0.], (len(rows), 1))


def vertical(rows):
    return np.tile([0., 1.], (len(rows), 1))


def field(function=horizontal):
    return m.VectorField(function, axes(), sample_coords=[[-.5, 0.], [.5, 0.]],
                         color=m.WHITE, max_vect_len=1.)


def streamlines(function=horizontal):
    return m.StreamLines(function, axes(), solution_time=.2, dt=.05,
                         noise_factor=0., n_samples_per_line=4,
                         color_by_magnitude=True, stroke_width=6)


MODES = ("copy", "shallow", "deep", "saved", "target", "checkpoint")


class FieldCallbackSnapshots(unittest.TestCase):
    def test_vector_field_sampling_and_paint_copies_can_update_independently(self):
        for phase in ("field", "color", "opacity"):
            for mode in MODES:
                with self.subTest(phase=phase, mode=mode):
                    obj, clones, scene = field(), [], m.Scene()
                    scene.add(obj)
                    def take():
                        if not clones:
                            clones.append(capture(obj, mode, scene))
                    def sample(rows):
                        take()
                        return horizontal(rows)
                    def color(values):
                        take()
                        return np.tile([1., 0., 0., 1.], (len(values), 1))
                    def opacity(values):
                        take()
                        return .5
                    if phase == "field":
                        obj.func = sample
                    elif phase == "color":
                        obj.color_map = color
                    else:
                        obj.norm_to_opacity_func = opacity
                    before = obj.data.copy()
                    obj.update_vectors()
                    self.assertEqual(len(clones), 1)
                    clone = clones[0]
                    np.testing.assert_array_equal(clone.data, before)
                    published = obj.data.copy()
                    clone.func = vertical
                    clone.update_vectors()
                    vectors = clone.get_points()[6::8] - clone.get_points()[0::8]
                    np.testing.assert_allclose(vectors[:, 0], 0., atol=1e-6)
                    self.assertTrue(np.all(vectors[:, 1] > 0))
                    np.testing.assert_array_equal(obj.data, published)
                    self.assertNotIn("_fmn_vector_field_updating", vars(clone))
                    self.assertNotIn("_fmn_vector_field_style_target", vars(clone))
                    self.assertEqual(scene.get_time(), 0.)

    def test_copy_paint_cannot_write_into_another_invocations_scratch(self):
        obj, clones = field(), []
        def color(values):
            if not clones:
                clones.append(obj.copy())
                clone = clones[0]
                clone.color_map = lambda a: np.tile([0., 0., 1., 1.], (len(a), 1))
                clone._apply_callback_style(horizontal(clone.sample_coords))
            return np.tile([1., 0., 0., 1.], (len(values), 1))
        obj.color_map = color
        obj.update_vectors()
        np.testing.assert_array_equal(obj.data['stroke_rgba'][:, :3],
                                      np.tile([1., 0., 0.], (obj.get_num_points(), 1)))
        np.testing.assert_array_equal(clones[0].data['stroke_rgba'][:, :3],
                                      np.tile([0., 0., 1.], (clones[0].get_num_points(), 1)))

    def test_vector_field_refuses_obsolete_candidates_preserving_authored_edits(self):
        for edit in (lambda o: o.shift(m.UP), lambda o: o.set_stroke(m.BLUE),
                     lambda o: o.add(m.Dot()),
                     lambda o: o.sample_coords.__setitem__((0, 0), -.75)):
            with self.subTest(edit=edit):
                obj, captured = field(), []
                def sample(rows):
                    if not captured:
                        edit(obj)
                        captured.append((obj.data.copy(), obj.sample_coords.copy(), tuple(obj.submobjects)))
                    return vertical(rows)
                obj.func = sample
                with self.assertRaisesRegex(RuntimeError, "changed during sampling"):
                    obj.update_vectors()
                np.testing.assert_array_equal(obj.data, captured[0][0])
                np.testing.assert_array_equal(obj.sample_coords, captured[0][1])
                self.assertEqual(tuple(obj.submobjects), captured[0][2])
                obj.update_vectors()

    def test_streamline_integration_and_paint_copies_rebuild(self):
        for phase in ("integrate", "paint"):
            for mode in MODES:
                with self.subTest(phase=phase, mode=mode):
                    obj, clones, scene = streamlines(), [], m.Scene()
                    scene.add(obj)
                    def sample(rows):
                        if not clones:
                            clones.append(capture(obj, mode, scene))
                        return horizontal(rows)
                    obj.func = sample
                    records = [line.data.copy() for line in obj]
                    obj.draw_lines() if phase == "integrate" else obj.init_style()
                    self.assertEqual(len(clones), 1)
                    clone = clones[0]
                    for line, before in zip(clone, records):
                        np.testing.assert_array_equal(line.data, before)
                    published = [line.data.copy() for line in obj]
                    clone.func = vertical
                    clone.draw_lines()
                    self.assertGreater(len(clone), 0)
                    for line in clone:
                        np.testing.assert_allclose(line.get_end()-line.get_start(), [0., .15, 0.], atol=1e-6)
                    for line, before in zip(obj, published):
                        np.testing.assert_array_equal(line.data, before)
                    self.assertNotIn("_fmn_streamline_authoring_busy", vars(clone))
                    self.assertEqual(scene.get_time(), 0.)

    def test_mixed_color_family_copies_can_be_recolored(self):
        for mode in MODES:
            with self.subTest(mode=mode):
                obj = m.Group(m.Point(), m.Square(fill_opacity=1))
                clones, scene = [], m.Scene()
                scene.add(obj)
                def color(points):
                    if not clones:
                        clones.append(capture(obj, mode, scene))
                    return [1., 0., 0.]
                before = [member.data.copy() for member in obj]
                obj.set_color_by_rgb_func(color)
                for member, data in zip(clones[0], before):
                    np.testing.assert_array_equal(member.data, data)
                clones[0].set_color_by_rgba_func(lambda p: [0., 0., 1., .5])
                for original, clone in zip(obj, clones[0]):
                    names = ('rgba',) if 'rgba' in clone.data.dtype.names else ('fill_rgba', 'stroke_rgba')
                    for name in names:
                        np.testing.assert_array_equal(clone.data[name], np.tile([0., 0., 1., .5], (len(clone.data), 1)))
                        np.testing.assert_array_equal(original.data[name], np.tile([1., 0., 0., 1.], (len(original.data), 1)))

    def test_shared_family_overlap_refuses_without_poisoning_other_members(self):
        shared, other = m.Square(), m.Circle()
        first, second = m.VGroup(shared), m.VGroup(other, shared)
        def overlap(points):
            second.set_color_by_rgb_func(lambda p: [0., 0., 1.])
            return [1., 0., 0.]
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            first.set_color_by_rgb_func(overlap)
        second.set_color_by_rgb_func(lambda p: [0., 1., 0.])
        np.testing.assert_array_equal(other.data['stroke_rgba'][:, 1], 1.)

    def test_field_checkpoint_restoration_and_cancellation_leave_no_busy_state(self):
        for kind in ("vectors", "lines", "color"):
            with self.subTest(kind=kind):
                obj = field() if kind == "vectors" else streamlines() if kind == "lines" else m.Square()
                scene, states = m.Scene(), []
                scene.add(obj)
                def checkpoint(values):
                    if not states:
                        states.append(scene.get_state())
                    return horizontal(values)
                if kind == "vectors":
                    obj.func = checkpoint
                    refresh = obj.update_vectors
                elif kind == "lines":
                    obj.func = checkpoint
                    refresh = obj.draw_lines
                else:
                    def paint(values):
                        checkpoint(values)
                        return [1., 0., 0.]
                    refresh = lambda: obj.set_color_by_rgb_func(paint)
                refresh()
                states[0].restore_scene(scene)
                refresh()
                self.assertIs(obj._scene, scene)
                def cancel(values):
                    raise KeyboardInterrupt('cancel field')
                if kind == "vectors":
                    obj.func = cancel
                elif kind == "lines":
                    obj.func = cancel
                else:
                    refresh = lambda: obj.set_color_by_rgb_func(cancel)
                with self.assertRaises(KeyboardInterrupt):
                    refresh()
                if kind == "color":
                    obj.set_color_by_rgb_func(lambda p: [0., 1., 0.])
                else:
                    obj.func = horizontal
                    refresh()

    def test_native_vector_frames_match_copy_free_scene_and_thread_counts(self):
        def scene_type(with_copy):
            class Demo(m.Scene):
                def construct(self):
                    obj, clones = field(), []
                    def color(values):
                        if with_copy and not clones:
                            clones.append(obj.copy())
                            clones[0].update_vectors()
                        return np.tile([0., .5, 1., 1.], (len(values), 1))
                    obj.color_map = color
                    obj.add_updater(lambda current: current.update_vectors(), call=False)
                    self.add(obj)
                    self.wait(.25)
            return Demo
        with tempfile.TemporaryDirectory(prefix='fmn-field-snapshots-') as directory:
            outputs = []
            for with_copy, threads in ((False, 1), (True, 1), (True, 4)):
                path = Path(directory) / f'{with_copy}-{threads}.y4m'
                render_scene(scene_type(with_copy), path, format='y4m', resolution=(160, 90), fps=8, threads=threads)
                data = path.read_bytes()
                body = data.split(b'\n', 1)[1]
                self.assertEqual(len(body), 2 * (6 + 160 * 90 * 3 // 2))
                self.assertGreater(np.ptp(np.frombuffer(body[6:6+160*90], dtype=np.uint8)), 0)
                outputs.append(data)
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[1], outputs[2])


if __name__ == '__main__':
    unittest.main(verbosity=2)
