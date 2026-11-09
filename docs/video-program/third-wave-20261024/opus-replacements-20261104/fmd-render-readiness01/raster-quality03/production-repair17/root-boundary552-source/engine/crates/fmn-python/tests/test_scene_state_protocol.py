"""Snapshot orchestration with fake native storage, not a native-render receipt."""
from __future__ import annotations

import copy
import importlib.util
from pathlib import Path
import pickle
from types import SimpleNamespace
import unittest

import numpy as np

SOURCE = Path(__file__).resolve().parents[1] / "python/fmn_python/scene_state.py"
spec = importlib.util.spec_from_file_location("scene_state_under_test", SOURCE)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# These are the production bootstrap's existing SceneState entry points.
# The local backend deliberately restores only engine data, NOT Python state.
LEGACY = '''
def _mobject_looks_identical(mobject, other):
    if type(mobject) is not type(other):
        return False
    count = mobject.get_num_points()
    if count != other.get_num_points():
        return False
    if count == 0:
        return True
    return bool(_np.allclose(mobject.get_points(), other.get_points()))

class SceneState:
    def __init__(self, scene, ignore=None):
        if not isinstance(scene, Scene):
            raise TypeError("SceneState scene must be a Scene")
        self._scene = scene
        self.time = scene.get_time()
        self.num_plays = int(getattr(scene, "num_plays", 0))
        skip = set() if ignore is None else set(ignore)
        last = {}
        if getattr(scene, "undo_stack", None):
            last = scene.undo_stack[-1].mobjects_to_copies
        copies = {}
        for mobject in list(scene.mobjects):
            if mobject in skip:
                continue
            prior = last.get(mobject)
            if prior is not None and _mobject_looks_identical(mobject, prior):
                copies[mobject] = prior
            else:
                copies[mobject] = mobject.copy()
        self.mobjects_to_copies = copies
        self._checkpoint = None if skip else bytes(scene._checkpoint_bytes())

    def restore_scene(self, scene):
        if not isinstance(scene, Scene):
            raise TypeError("restore_scene expects a Scene")
        if scene is self._scene and self._checkpoint is not None:
            scene._restore_checkpoint_bytes(self._checkpoint)
            scene._reseat_engine_roots(*self.mobjects_to_copies)
            scene.num_plays = self.num_plays
            return
        restored = [mobject.become(saved, match_updaters=True)
                    for mobject, saved in self.mobjects_to_copies.items()]
        scene.clear()
        if restored:
            scene.add(*restored)
        scene.num_plays = self.num_plays
'''


class Children(list):
    def __init__(self, owner):
        self.owner = owner
        super().__init__()

    def _replace_projection(self, children):
        for old in self:
            if old not in children and self.owner in old.parents:
                old.parents.remove(self.owner)
        for child in children:
            if self.owner not in child.parents:
                child.parents.append(self.owner)
        self[:] = children


class Uniforms(dict):
    def __init__(self):
        super().__init__(opacity=1., flag=False)
        self._extras = {"gain": np.array([1., 2.])}

    def keys(self):
        return dict(self.items()).keys()

    def __getitem__(self, name):
        return self._extras[name] if name in self._extras else super().__getitem__(name)

    def __iter__(self):
        return iter([*super().__iter__(), *self._extras])

    def items(self):
        return [*super().items(), *self._extras.items()]


class Mob:
    def __init__(self, *children, x=None):
        self.data = np.zeros(0 if x is None else 1,
                             dtype=[("point", "f8", 3), ("rgba", "f8", 4)])
        if x is not None:
            self.data["point"][0, 0] = x
        self.uniforms = Uniforms()
        self.updaters, self.parents = [], []
        self.locked_data_keys, self.const_data_keys, self.locked_uniform_keys = set(), set(), set()
        self.submobjects = Children(self)
        self.submobjects._replace_projection(children)

    def get_num_points(self):
        return len(self.data)

    def get_points(self):
        return self.data["point"]

    def looks_identical(self, other):
        return type(self) is type(other)  # Public hook; installer checks actual records too.

    def copy(self):
        memo = {}
        def clone(source):
            if id(source) in memo:
                return memo[id(source)]
            result = type(source)(x=None)
            memo[id(source)] = result
            result.data = source.data.copy()
            result.uniforms = copy.deepcopy(source.uniforms)
            # Production shallow-copy behavior; SceneState must freeze these.
            result.uniforms._extras = dict(source.uniforms._extras)
            result.updaters = list(source.updaters)
            for name in ("locked_data_keys", "const_data_keys", "locked_uniform_keys"):
                setattr(result, name, set(getattr(source, name)))
            result.submobjects._replace_projection([clone(child) for child in source.submobjects])
            return result
        return clone(self)

    def become(self, saved, match_updaters=False):
        self.data = saved.data.copy()
        self.uniforms = copy.deepcopy(saved.uniforms)
        self.submobjects._replace_projection([child.copy() for child in saved.submobjects])
        if match_updaters:
            self.updaters = list(saved.updaters)
        return self


class CameraCore:
    def __init__(self):
        self._center, self._shape = (0., 0., 0.), (14., 8.)
        self._orientation = self._default = (0., 0., 0., 1.)
        self._axes, self._fov = "zxz", .7
    def center(self): return self._center
    def shape(self): return self._shape
    def orientation(self): return self._orientation
    def field_of_view(self): return self._fov
    def euler_axes(self): return self._axes
    def set_center(self, value): self._center = tuple(value)
    def set_shape(self, value): self._shape = tuple(value)
    def set_orientation(self, value): self._orientation = tuple(value)
    def set_euler_axes(self, value): self._axes = value
    def set_field_of_view(self, value): self._fov = value
    def make_orientation_default(self): self._default = self._orientation
    def to_default_state(self): self._orientation = self._default


class Frame:
    def __init__(self):
        self._core, self.updaters = CameraCore(), []


def environment(install=True):
    class Scene:
        def __init__(self, *roots):
            self.roots = list(roots)
            self.registry = {id(mob): mob for mob in module._family(roots)}
            self._time, self.num_plays = 0., 0
            self.undo_stack, self.redo_stack = [], []
            self.max_num_saved_states = 3
            self.frame = Frame()
            self.restore_calls = 0
        @property
        def mobjects(self): return list(self.roots)
        def add(self, *objects):
            self.roots.extend(objects)
            self.registry.update((id(mob), mob) for mob in module._family(objects))
        def clear(self): self.roots.clear()
        def get_time(self): return self._time
        def _checkpoint_bytes(self):
            native = [(id(mob), mob.data.copy(), dict(dict.items(mob.uniforms)))
                      for mob in self.registry.values()]
            return pickle.dumps(([id(m) for m in self.roots], native, self._time), protocol=5)
        def _restore_checkpoint_bytes(self, payload):
            if getattr(self, "refuse_restore", False):
                raise RuntimeError("backend rejected checkpoint")
            self.restore_calls += 1
            roots, entries, self._time = pickle.loads(payload)
            self.roots = [self.registry[identity] for identity in roots]
            for identity, data, uniforms in entries:
                obj = self.registry[identity]
                obj.data = data.copy()
                dict.clear(obj.uniforms)
                dict.update(obj.uniforms, uniforms)
        def _reseat_engine_roots(self, *objects):
            assert [id(obj) for obj in self.roots] == [id(obj) for obj in objects]
        def get_state(self): return g["SceneState"](self)
        def restore_state(self, state): return state.restore_scene(self)
    g = {"Scene": Scene, "_np": np, "_ForeignStageError": ValueError,
         # This suite doubles native records and owns no image resources.
         "_raster_images_equal": lambda left, right: True}
    exec(LEGACY, g)
    native = SimpleNamespace(**g)
    # Real bootstrap methods share the native module dictionary, including
    # the dynamically installed identity comparator.
    g = vars(native)
    exec(LEGACY, g)
    if install:
        module.install_scene_state(native)
    return native


class SceneStateProtocol(unittest.TestCase):
    def setUp(self):
        self.native = environment()
        self.scene = self.native.Scene(Mob(Mob(x=1.), Mob(x=2.)))
        self.group = self.scene.roots[0]
        self.child = self.group.submobjects[0]

    def state(self): return self.native.SceneState(self.scene)

    def test_negative_control_old_group_reuses_changed_child(self):
        native = environment(False)
        child = Mob(x=1.)
        scene = native.Scene(Mob(child))
        first = native.SceneState(scene)
        scene.undo_stack.append(first)
        child.data["point"] += 3
        second = native.SceneState(scene)
        self.assertIs(first.mobjects_to_copies[scene.roots[0]], second.mobjects_to_copies[scene.roots[0]])

    def test_child_geometry_edit_prevents_stale_reuse(self):
        first = self.state()
        self.scene.undo_stack.append(first)
        self.child.data["point"] += 3
        second = self.state()
        self.assertIsNot(first.mobjects_to_copies[self.group], second.mobjects_to_copies[self.group])
        self.assertFalse(first.mobjects_match(second))
        self.assertEqual(first.n_changes(second), 1)

    def test_unchanged_family_reuses_native_copy(self):
        first = self.state()
        self.scene.undo_stack.append(first)
        self.assertIs(first.mobjects_to_copies[self.group], self.state().mobjects_to_copies[self.group])

    def test_style_only_change_is_detected(self):
        first = self.state()
        self.scene.undo_stack.append(first)
        self.child.data["rgba"][0, 3] = .5
        self.assertFalse(first.mobjects_match(self.state()))

    def test_point_free_root_uniform_is_detected(self):
        first = self.state()
        self.group.uniforms["opacity"] = .4
        self.assertFalse(first.mobjects_match(self.state()))

    def test_callbacks_and_locks_are_part_of_state(self):
        first = self.state()
        self.child.updaters.append(lambda m: None)
        second = self.state()
        self.assertFalse(first.mobjects_match(second))
        self.child.locked_data_keys.add("point")
        self.assertFalse(second.mobjects_match(self.state()))

    def test_root_order_controls_snapshot_equality(self):
        scene = self.native.Scene(Mob(x=1.), Mob(x=2.))
        first = self.native.SceneState(scene)
        scene.roots.reverse()
        self.assertFalse(first.mobjects_match(self.native.SceneState(scene)))

    def test_shared_descendant_topology_matters(self):
        left = Mob(self.child, self.child)
        right = Mob(self.child.copy(), self.child.copy())
        self.assertFalse(module._same_mobject(np, left, right))
        self.assertTrue(module._same_mobject(np, left, left.copy()))

    def test_comparison_uses_snapshots_not_later_source_edits(self):
        first, second = self.state(), self.state()
        self.child.data["point"] += 100
        self.assertEqual(first.n_changes(second), 0)
        self.assertTrue(first.mobjects_match(second))

    def test_restore_retains_original_child_and_callback_identities(self):
        updater = lambda obj: None
        self.child.updaters.append(updater)
        first = self.state()
        self.group.submobjects._replace_projection([Mob(x=8.)])
        self.child.updaters.clear()
        self.child.data["point"] += 100
        first.restore_scene(self.scene)
        self.assertIs(self.group.submobjects[0], self.child)
        self.assertIs(self.child.updaters[0], updater)
        self.assertIn(self.group, self.child.parents)
        np.testing.assert_array_equal(self.child.data["point"], [[1., 0., 0.]])

    def test_extension_uniforms_restored_independently_on_each_restore(self):
        first = self.state()
        self.child.uniforms._extras["gain"][:] = 7
        first.restore_scene(self.scene)
        np.testing.assert_array_equal(self.child.uniforms._extras["gain"], [1., 2.])
        self.child.uniforms._extras["gain"][:] = 9
        first.restore_scene(self.scene)
        np.testing.assert_array_equal(self.child.uniforms._extras["gain"], [1., 2.])

    def test_mutable_uniform_snapshot_does_not_follow_live_array(self):
        first = self.state()
        self.scene.undo_stack.append(first)
        self.child.uniforms._extras["gain"][:] = 9
        saved = first.mobjects_to_copies[self.group].submobjects[0]
        np.testing.assert_array_equal(saved.uniforms._extras["gain"], [1., 2.])
        self.assertFalse(first.mobjects_match(self.state()))

    def test_identical_geometry_with_replaced_child_is_a_different_state(self):
        first = self.state()
        self.scene.undo_stack.append(first)
        replacement = self.child.copy()
        self.group.submobjects._replace_projection([replacement, self.group.submobjects[1]])
        second = self.state()
        self.assertFalse(first.mobjects_match(second))
        self.assertIsNot(first.mobjects_to_copies[self.group], second.mobjects_to_copies[self.group])

    def test_opaque_uniform_objects_are_neither_copied_nor_compared(self):
        class Opaque:
            def __eq__(self, other): raise AssertionError("must not evaluate equality")
            def __deepcopy__(self, memo): raise AssertionError("must not copy host resources")
        opaque = Opaque()
        value = {"opaque": opaque, "array": np.array([1.]), "scalar": np.float32(2.)}
        frozen = module._freeze_value(np, value)
        self.assertIs(frozen["opaque"], opaque)
        self.assertIsNot(frozen["array"], value["array"])
        self.assertTrue(module._same_value(np, frozen, value))

    def test_uniform_container_cycles_do_not_recurse_forever(self):
        value = []
        value.append(value)
        frozen = module._freeze_value(np, value)
        self.assertIs(frozen[0], frozen)
        self.assertTrue(module._same_value(np, frozen, value))

    def test_full_camera_pose_defaults_and_core_identity_roundtrip(self):
        frame, core = self.scene.frame, self.scene.frame._core
        core.set_orientation((0., 1., 0., 0.))
        core.make_orientation_default()
        core.set_orientation((1., 0., 0., 0.))
        core.set_shape((10., 6.))
        first = self.state()
        expected = module._camera_values(core)
        core.set_center((8., 9., 10.))
        core.set_shape((3., 2.))
        core.set_euler_axes("zxy")
        core.set_field_of_view(.4)
        core.set_orientation((0., 0., 1., 0.))
        core.make_orientation_default()
        self.assertFalse(first.mobjects_match(self.state()))
        first.restore_scene(self.scene)
        self.assertIs(self.scene.frame, frame)
        self.assertIs(frame._core, core)
        self.assertEqual(module._camera_values(core), expected)

    def test_time_play_count_and_camera_callback_roundtrip(self):
        self.scene._time, self.scene.num_plays = 3., 4
        callback = lambda m, dt: None
        self.scene.frame.updaters.append(callback)
        first = self.state()
        self.scene._time, self.scene.num_plays = 99., 100
        self.scene.frame.updaters.clear()
        first.restore_scene(self.scene)
        self.assertEqual((self.scene.get_time(), self.scene.num_plays), (3., 4))
        self.assertEqual(self.scene.frame.updaters, [callback])

    def test_foreign_scene_refused_before_any_source_or_target_write(self):
        state = self.state()
        other = self.native.Scene(Mob(x=7.))
        with self.assertRaisesRegex(ValueError, "another Scene"):
            state.restore_scene(other)
        self.assertEqual(other.restore_calls, 0)
        self.assertEqual(self.scene.restore_calls, 0)
        self.assertIs(self.scene.roots[0], self.group)

    def test_ignore_frame_leaves_live_pose_alone(self):
        state = self.native.SceneState(self.scene, ignore=[self.scene.frame])
        self.scene.frame._core.set_center((4., 0., 0.))
        state.restore_scene(self.scene)
        self.assertEqual(self.scene.frame._core.center(), (4., 0., 0.))

    def test_equality_handles_types_time_and_hash_policy(self):
        a, b = self.state(), self.state()
        self.assertEqual(a, b)
        self.scene._time = 1.
        self.assertNotEqual(a, self.state())
        self.assertNotEqual(a, object())
        with self.assertRaises(TypeError): hash(a)

    def test_installer_is_idempotent_and_keeps_class_identity(self):
        cls = self.native.SceneState
        init = cls.__init__
        module.install_scene_state(self.native)
        self.assertIs(self.native.SceneState, cls)
        self.assertIs(cls.__init__, init)
        self.assertEqual(cls.restore_scene.__name__, "restore_scene")

    def test_cycles_are_rejected_by_projection_capture(self):
        bad = Mob()
        bad.submobjects.append(bad)
        with self.assertRaisesRegex(ValueError, "cyclic"):
            module._family([bad])


if __name__ == "__main__":
    unittest.main()
