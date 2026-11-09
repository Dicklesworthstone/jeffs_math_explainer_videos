"""Deferred adoption and mixed composition use production Python drivers.

Arena storage, native animation cores, interval construction and group
construction are explicit doubles. This is not native-renderer acceptance.
"""
import ast
from pathlib import Path
import types
import unittest

from test_animation_updater_protocol import adapter
from test_native_animation_updater_protocol import native_environment

ROOT = Path(__file__).resolve().parents[1]


def composition_environment():
    native = native_environment()
    g = vars(native)
    def intervals(durations, lag):
        start, result = 0., []
        for duration in durations:
            result.append((start, start + duration))
            start += lag * duration
        return result
    g['_FMN_ROOT'] = types.SimpleNamespace(_composition_intervals=intervals)
    g['_linear_rate'] = g['smooth']
    path = ROOT / 'python/manimlib_bootstrap.py'
    tree = ast.parse(path.read_text())
    names = ('_composition_member_run_time', '_composition_timings',
             '_composition_timeline_position', '_CompositionCallbackDriver')
    for name in names:
        node = next(n for n in tree.body if isinstance(n, (ast.ClassDef, ast.FunctionDef)) and n.name == name)
        exec(compile(ast.Module([node], []), str(path), 'exec'), g)
    Group = native.AnimationGroup
    Group._native_kind, Group._default_lag_ratio = 'animation_group', 0.
    def group_init(self, *children, group=None, run_time=None, lag_ratio=0., **kwargs):
        self.animations = list(children)
        root = native.Mobject(0., *(child.mobject for child in children)) if group is None else group
        maximum = max((end for _, end in intervals([child.get_run_time() for child in children], lag_ratio)), default=0.)
        native.Animation.__init__(self, root, run_time=maximum if run_time is None else run_time,
                                  lag_ratio=lag_ratio, **kwargs)
    Group.__init__ = group_init
    path = ROOT / 'python/manimlib/_animation_semantics.py'
    tree = ast.parse(path.read_text())
    installer = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == '_install_composition_lifecycle')
    cg = dict(g, CallbackDriver=g['_CompositionCallbackDriver'], make_driver=g['_fmn_make_animation_driver'],
              ensure_root=lambda animation:animation.mobject)
    functions = ('abort_children', 'release', 'abort', 'drive', 'begin', 'update_mobjects',
                 'interpolate', 'finish', 'clean_up_from_scene')
    for name in functions:
        node = next(n for n in installer.body if isinstance(n, ast.FunctionDef) and n.name == name)
        exec(compile(ast.Module([node], []), str(path), 'exec'), cg)
        if name in ('abort', 'begin', 'update_mobjects', 'interpolate', 'finish', 'clean_up_from_scene'):
            setattr(Group, name, cg[name])
    g['_fmn_ensure_composition_root'] = lambda animation:animation.mobject
    g['_fmn_abort_animation_driver'] = cg['abort_children']
    class Succession(Group):
        _native_kind, _default_lag_ratio = 'succession', 1.
        def __init__(self, *children, **kwargs):
            kwargs.setdefault('lag_ratio', 1.)
            super().__init__(*children, **kwargs)
    native.Succession = Succession
    return native


class DeferredNativeTests(unittest.TestCase):
    def setUp(self):
        self.native = native_environment()
        self.scene = self.native.Scene()
        self.mob = self.native.Mobject(2.)
        self.animation = self.native.NativeOnly(self.mob)
    def attach(self):
        return self.native.turn_animation_into_updater(self.animation)
    def test_registers_without_allocating_a_scene_or_driver(self):
        self.assertIs(self.attach(), self.mob)
        owner = self.mob.updaters[0]._fmn_persistent_controller
        self.assertFalse(owner.begun)
        self.assertIsNone(owner.scene)
        self.assertEqual(self.scene.cores, [])
        self.assertFalse(self.mob.animating)
    def test_zero_dt_updates_leave_pending_clock_alone(self):
        self.attach()
        for _ in range(5):
            self.mob.update(0.)
        self.assertEqual(self.animation.total_time, 0.)
        self.assertEqual(self.scene.cores, [])
    def test_adoption_activates_once_at_first_updater_boundary(self):
        self.attach()
        self.scene.add(self.mob)
        self.assertEqual(self.scene.cores, [])
        self.mob.update(.5).update(0.)
        self.assertEqual(self.mob.value, 4.)
        self.assertEqual(len(self.scene.cores), 1)
        self.assertEqual(self.scene.cores[0].events.count(('begin',)), 1)
    def test_pending_nonnative_advance_refuses_without_losing_registration(self):
        self.attach()
        with self.assertRaisesRegex(RuntimeError, 'Scene adoption'):
            self.mob.update(.2)
        self.assertTrue(self.mob.updaters)
        self.assertEqual(self.animation.total_time, 0.)
        self.scene.add(self.mob)
        self.mob.update(1.).update(0.)
        self.assertEqual(self.mob.value, 6.)
    def test_native_snapshot_uses_state_at_activation(self):
        self.attach()
        self.mob.value = 20.
        self.scene.add(self.mob)
        self.mob.update(.5).update(0.)
        self.assertEqual(self.mob.value, 22.)
    def test_rate_configuration_can_change_while_pending(self):
        self.attach()
        self.animation.run_time = 2.
        self.animation.rate_func = 'linear'
        self.scene.add(self.mob)
        self.mob.update(1.).update(0.)
        self.assertEqual(self.scene.cores[0].spec[3:5], (2., 'linear'))
        self.assertEqual(self.mob.value, 4.)
    def test_native_duration_freezes_after_activation(self):
        self.attach()
        self.scene.add(self.mob)
        self.mob.update(.5)
        self.animation.run_time = 2.
        self.mob.update(0.)
        self.assertEqual(self.mob.value, 4.)
    def test_pending_removal_never_starts_the_animation(self):
        self.attach()
        updater, = self.mob.updaters
        self.mob.remove_updater(updater)
        self.scene.add(self.mob)
        self.mob.update(1.)
        self.assertEqual(self.scene.cores, [])
        self.assertIsNone(updater._fmn_persistent_controller.driver)
    def test_pending_copy_in_other_scene_cannot_activate_source(self):
        self.attach()
        duplicate = self.mob.copy()
        other = self.native.Scene()
        other.add(duplicate)
        duplicate.update(1.)
        self.assertEqual(other.cores, [])
        self.assertEqual(self.animation.total_time, 0.)
    def test_later_native_hook_change_is_revalidated(self):
        self.attach()
        self.animation.begin = lambda:None
        self.scene.add(self.mob)
        with self.assertRaisesRegex(NotImplementedError, 'authored lifecycle'):
            self.mob.update(0.)
        self.assertFalse(self.mob.updaters)
        self.assertEqual(self.scene.cores, [])
    def test_later_endpoint_change_is_not_dropped(self):
        self.attach()
        self.animation.final_alpha_value = .5
        self.scene.add(self.mob)
        with self.assertRaisesRegex(NotImplementedError, 'final_alpha'):
            self.mob.update(0.)
        self.assertFalse(self.mob.updaters)
    def test_bad_dt_conversion_detaches_pending_controller(self):
        self.attach()
        with self.assertRaises(ValueError):
            self.mob.update('invalid')
        self.assertFalse(self.mob.updaters)
    def test_nonfinite_pending_dt_detaches(self):
        self.attach()
        with self.assertRaises(ValueError):
            self.mob.update(float('nan'))
        self.assertFalse(self.mob.updaters)
    def test_later_foreign_target_cancels_before_native_lowering(self):
        self.attach()
        target = self.native.Mobject()
        self.native.Scene().add(target)
        self.animation.path = target
        self.scene.add(self.mob)
        with self.assertRaisesRegex(ValueError, 'multiple Scenes'):
            self.mob.update(0.)
        self.assertEqual(self.scene.cores, [])
        self.assertFalse(self.mob.updaters)
    def test_duplicate_start_does_not_modify_live_clock_or_options(self):
        self.attach()
        with self.assertRaisesRegex(RuntimeError, 'already'):
            self.native.turn_animation_into_updater(self.animation, run_time=5.)
        self.assertEqual(self.animation.run_time, 1.)
        self.assertEqual(len(self.mob.updaters), 1)
    def test_zero_duration_native_finishes_after_adoption(self):
        self.animation.run_time = 0.
        self.attach()
        self.assertTrue(self.mob.updaters)
        self.scene.add(self.mob)
        self.mob.update(0.)
        self.assertEqual(self.mob.value, 6.)
        self.assertFalse(self.mob.updaters)


class PersistentCompositionTests(unittest.TestCase):
    def setUp(self):
        self.native = composition_environment()
        self.scene = self.native.Scene()
        self.a, self.b = self.native.Mobject(), self.native.Mobject()
    def pair(self, **kwargs):
        return self.native.AnimationGroup(self.native.NativeOnly(self.a), self.native.PerMember(self.b), **kwargs)
    def start(self, animation):
        root = self.native.turn_animation_into_updater(animation)
        self.scene.add(root)
        return root
    def test_original_group_requires_scene_as_negative_control(self):
        group = self.pair()
        with self.assertRaisesRegex(RuntimeError, 'scene-bound'):
            group.begin()
        root = self.start(group)
        root.update(.5).update(0.)
        self.assertEqual((self.a.value, self.b.value), (2., .5))
    def test_detached_mixed_group_retains_actual_driver_timing(self):
        group = self.pair(run_time=2.)
        root = self.start(group)
        root.update(1.).update(0.)
        self.assertEqual((self.a.value, self.b.value), (2., .5))
        self.assertEqual(group.total_time, 1.)
    def test_mixed_succession_snapshots_after_predecessor_finishes(self):
        group = self.native.Succession(self.native.NativeOnly(self.a), self.native.PerMember(self.a))
        root = self.start(group)
        root.update(1.5).update(0.)
        self.assertEqual(self.a.value, 4.5)
        self.assertEqual(self.scene.cores[0].events.count(('finish',)), 1)
    def test_python_then_native_snapshots_just_in_time(self):
        group = self.native.Succession(self.native.PerMember(self.a), self.native.NativeOnly(self.a))
        root = self.start(group)
        root.update(1.5).update(0.)
        self.assertEqual(self.a.value, 3.)
    def test_completion_never_calls_scene_cleanup(self):
        group = self.pair()
        group.clean_up_from_scene = lambda scene:self.fail('no persistent scene cleanup')
        root = self.start(group)
        root.update(1.).update(0.)
        self.assertEqual((self.a.value, self.b.value), (4., 1.))
        self.assertFalse(root.updaters)
        self.assertIn(root, self.scene.roots)
    def test_completed_group_releases_native_children_without_abort(self):
        group = self.pair()
        root = self.start(group)
        root.update(1.).update(0.)
        self.assertIsNone(group._composition_driver)
        self.assertNotIn(('abort',), self.scene.cores[0].events)
        self.assertEqual(self.scene.cores[0].events.count(('finish',)), 1)
    def test_completed_nested_groups_release_each_retained_driver(self):
        inner = self.pair()
        outer = self.native.AnimationGroup(inner)
        root = self.start(outer)
        root.update(1.).update(0.)
        self.assertIsNone(inner._composition_driver)
        self.assertIsNone(outer._composition_driver)
        self.assertNotIn(('abort',), self.scene.cores[0].events)
    def test_shortened_succession_does_not_finish_unbegun_future_child(self):
        native = self.native.NativeOnly(self.a)
        later = self.native.PerMember(self.b)
        group = self.native.Succession(native, later, final_alpha_value=.1)
        root = self.start(group)
        root.update(2.).update(0.)
        self.assertEqual(self.a.value, 4.)
        self.assertEqual(self.b.value, 0.)
        self.assertNotIn(('finish',), later.events)
        self.assertIsNone(group._composition_driver)
    def test_explicit_group_root_preserves_identity(self):
        root = self.native.Mobject()
        group = self.pair(group=root)
        self.assertIs(self.start(group), root)
        root.update(.5).update(0.)
        self.assertEqual((self.a.value, self.b.value), (2., .5))
    def test_nested_authored_groups_receive_temporary_scene_context(self):
        observations = []
        native, scene = self.native, self.scene
        class Authored(native.AnimationGroup):
            def begin(self):
                observations.append(self._composition_scene)
                super().begin()
        inner = Authored(native.NativeOnly(self.a), group=native.Mobject())
        outer = native.AnimationGroup(inner, native.PerMember(self.b))
        prior = object()
        inner._composition_scene = prior
        root = self.start(outer)
        root.update(.5).update(0.)
        self.assertEqual(observations, [scene])
        self.assertIs(inner._composition_scene, prior)
        self.assertNotIn('_composition_scene', outer.__dict__)
    def test_native_child_failure_unwinds_started_siblings(self):
        group = self.pair()
        root = self.start(group)
        root.update(0.)
        core = self.scene.cores[0]
        core.fail = 'interpolate'
        with self.assertRaisesRegex(RuntimeError, 'interpolate'):
            root.update(.1)
        self.assertFalse(root.updaters)
        self.assertFalse(self.a.animating)
        self.assertFalse(self.b.animating)
        self.assertIsNone(group._composition_driver)
    def test_pending_composition_cycle_is_rejected_without_recursion(self):
        group = self.pair()
        group.animations.append(group)
        with self.assertRaisesRegex(ValueError, 'cycle'):
            self.native.turn_animation_into_updater(group)
        self.assertFalse(group.mobject.updaters)
    def test_shared_nested_composition_context_is_restored_once(self):
        inner = self.pair()
        outer = self.native.AnimationGroup(inner, inner)
        prior = object()
        inner._composition_scene = prior
        with adapter._composition_context(vars(self.native), outer, self.scene):
            self.assertIs(inner._composition_scene, self.scene)
        self.assertIs(inner._composition_scene, prior)
    def test_child_targets_in_other_scene_are_rejected_before_begin(self):
        group = self.pair(group=self.native.Mobject())
        self.scene.add(group.mobject)
        target = self.native.Mobject()
        self.native.Scene().add(target)
        group.animations[0].path = target
        with self.assertRaisesRegex(ValueError, 'multiple Scenes'):
            self.native.turn_animation_into_updater(group)
        self.assertEqual(self.scene.cores, [])
    def test_removal_inside_authored_group_callback_is_boundary_safe(self):
        group = self.pair()
        root = self.start(group)
        root.update(0.)
        child = group.animations[1]
        original = child.interpolate
        def interpolate(alpha):
            original(alpha)
            root.clear_updaters()
        child.interpolate = interpolate
        root.update(.1)
        self.assertFalse(root.updaters)
        self.assertFalse(self.a.animating)
        self.assertFalse(self.b.animating)
        self.assertIsNone(group._composition_driver)
    def test_outer_suspension_stops_pending_activation(self):
        group = self.pair()
        root = self.start(group)
        root.suspend_updating()
        root.update(2.)
        self.assertEqual(self.scene.cores, [])
        root.resume_updating(call_updater=False)
        root.update(.5).update(0.)
        self.assertEqual((self.a.value, self.b.value), (2., .5))
    def test_lagged_native_and_python_members_preserve_intervals(self):
        group = self.pair(lag_ratio=1., run_time=2.)
        root = self.start(group)
        root.update(.5).update(0.)
        self.assertEqual((self.a.value, self.b.value), (2., 0.))
        root.update(1.).update(0.)
        self.assertEqual((self.a.value, self.b.value), (4., .5))


if __name__ == '__main__':
    unittest.main()
