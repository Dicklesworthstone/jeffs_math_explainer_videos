"""Authored group roots and timelines over the production Python dispatcher."""
import unittest
from test_composition_lifecycle_protocol import environment, semantics


class CompositionAuthoringTests(unittest.TestCase):
    def setUp(self):
        self.g = environment()
        semantics._install_composition_lifecycle(self.g)
        self.Group = self.g["AnimationGroup"]
        self.Mob = self.g["Mobject"]
        self.Scene = self.g["Scene"]
        self.Transform = self.g["Transform"]
        self.Animation = self.g["Animation"]
    def delayed(self, base=None):
        parent = base or self.Group
        class Delayed(parent):
            def build_animations_with_timings(self, lag_ratio):
                self.anims_with_timings = [(child, i * 2., i * 2. + 1.)
                                           for i, child in enumerate(self.animations)]
        return Delayed
    def test_explicit_root_keeps_identity_and_executes_children(self):
        mob = self.Mob()
        root = self.g["Group"](mob)
        group = self.Group(self.Transform(mob), group=root)
        self.assertIs(group.group, root)
        self.assertIs(group.mobject, root)
        self.assertTrue(self.g["_requires_python_animation"](group))
        scene = self.Scene()
        scene.play(group)
        self.assertIn(root, scene.roots)
        self.assertEqual(mob.x, 1.)
    def test_explicit_root_takes_precedence_over_factory(self):
        root = self.Mob()
        def fail(*args):
            raise AssertionError("factory must not run")
        group = self.Group(self.Transform(root), group=root, group_type=fail)
        self.assertIs(group.mobject, root)
    def test_custom_factory_receives_unique_ordered_members(self):
        a, b = self.Mob(), self.Mob()
        seen = []
        def factory(*members):
            seen.append(members)
            return self.g["Group"](*members)
        group = self.Group(self.Transform(a), self.Animation(a), self.Transform(b), group_type=factory)
        self.assertEqual(seen, [(a, b)])
        scene = self.Scene()
        scene.play(group)
        self.assertEqual(len(scene.cores), 2)
    def test_factory_preserves_nested_group_identity(self):
        child = self.Group(self.Transform(self.Mob()))
        seen = []
        def factory(*members):
            seen.extend(members)
            return self.g["Group"](*members)
        self.Group(child, group_type=factory)
        self.assertEqual(seen, [child.mobject])
    def test_factory_must_produce_mobject(self):
        with self.assertRaisesRegex(TypeError, "return a Mobject"):
            self.Group(self.Transform(self.Mob()), group_type=lambda *args: object())
    def test_invalid_root_type_is_rejected(self):
        with self.assertRaisesRegex(TypeError, "must be a Mobject"):
            self.Group(self.Transform(self.Mob()), group=object())
    def test_invalid_factory_type_is_rejected(self):
        with self.assertRaisesRegex(TypeError, "must be callable"):
            self.Group(self.Transform(self.Mob()), group_type=7)
    def test_existing_shipped_group_attribute_is_preserved(self):
        mob = self.Mob()
        root = self.g["Group"](mob)
        group = self.Group(self.Transform(mob))
        group.group = root
        self.assertIs(group.get_all_mobjects(), root)
    def test_custom_timing_hook_changes_motion_not_only_inspection(self):
        a, b = self.Animation(self.Mob()), self.Transform(self.Mob())
        group = self.delayed()(a, b)
        self.assertTrue(self.g["_requires_python_animation"](group))
        scene = self.Scene()
        scene.play(group)
        self.assertIn(("interpolate", .75), a.events)
        self.assertEqual([event[1] for event in scene.cores[0].events if event[0] == "interpolate"],
                         [0., 0., 0., .25, 1., 1.])
    def test_manual_table_edits_are_detected(self):
        a, b = self.Animation(self.Mob()), self.Animation(self.Mob())
        group = self.Group(a, b)
        group.anims_with_timings[:] = [(a, 0., 1.), (b, 2., 3.)]
        group.max_end_time = 3.
        self.assertTrue(self.g["_requires_python_animation"](group))
        self.Scene().play(group)
        self.assertIn(("interpolate", .25), b.events)
    def test_reordered_interpolation_rows_keep_begin_argument_order(self):
        trace = []
        parent = self.Animation
        class Probe(parent):
            def __init__(self, name):
                super().__init__(self_mob())
                self.name = name
            def begin(self):
                trace.append(("begin", self.name))
                super().begin()
            def interpolate(self, alpha):
                trace.append(("interpolate", self.name))
                super().interpolate(alpha)
        self_mob = self.Mob
        a, b = Probe("a"), Probe("b")
        group = self.Group(a, b)
        group.anims_with_timings.reverse()
        self.Scene().play(group)
        self.assertEqual(trace[:4], [("begin", "a"), ("begin", "b"), ("interpolate", "b"), ("interpolate", "a")])
    def test_succession_uses_authored_windows(self):
        mob = self.Mob()
        first, second = self.Animation(mob), self.Transform(mob)
        group = self.delayed(self.g["Succession"])(first, second)
        scene = self.Scene()
        scene.play(group)
        self.assertEqual(scene.cores[0].events[0], ("begin", 1.))
        self.assertEqual(mob.x, 2.)
    def test_foreign_or_duplicate_timing_rows_fail_before_play(self):
        for foreign in (True, False):
            a, b = self.Animation(self.Mob()), self.Animation(self.Mob())
            group = self.Group(a, b)
            group.anims_with_timings[1] = ((self.Animation(self.Mob()) if foreign else a), 0., 1.)
            scene = self.Scene()
            with self.assertRaisesRegex(ValueError, "foreign or duplicate"):
                scene.play(group)
            self.assertEqual(scene.roots, [])
    def test_invalid_timing_bounds_fail_before_play(self):
        for start, end in [(float("nan"), 1.), (0., float("inf")), (2., 1.)]:
            member = self.Animation(self.Mob())
            group = self.Group(member)
            group.anims_with_timings[:] = [(member, start, end)]
            scene = self.Scene()
            with self.assertRaisesRegex(ValueError, "finite and ordered"):
                scene.play(group)
            self.assertEqual(scene.roots, [])
    def test_missing_timing_rows_fail_before_play(self):
        group = self.Group(self.Animation(self.Mob()))
        group.anims_with_timings.clear()
        with self.assertRaisesRegex(ValueError, "one row"):
            self.Scene().play(group)
    def test_scene_runtime_override_does_not_rescale_authored_windows(self):
        a, b = self.Animation(self.Mob()), self.Animation(self.Mob())
        group = self.delayed()(a, b)
        self.Scene().play(group, run_time=30.)
        self.assertIn(("interpolate", .75), a.events)
        self.assertIn(("interpolate", .25), b.events)
    def test_generator_and_builder_inputs_with_explicit_root(self):
        mob = self.Mob()
        animation = self.Transform(mob)
        builder = self.g["_AnimationBuilder"](animation)
        group = self.Group((entry for entry in [builder]), group=self.g["Group"](mob))
        self.assertEqual(group.animations, [animation])
        self.Scene().play(group)
        self.assertEqual(mob.x, 1.)
    def test_zero_duration_custom_interval_is_defined(self):
        animation = self.Animation(self.Mob())
        group = self.Group(animation)
        group.anims_with_timings[:] = [(animation, 1., 1.)]
        self.Scene().play(group)
        self.assertEqual([x[1] for x in animation.events if x[0] == "interpolate"][:-1], [0.] * 5)
    def test_foreign_scene_root_still_refuses(self):
        mob = self.Mob()
        first, second = self.Scene(), self.Scene()
        first.add(mob)
        group = self.Group(self.Transform(mob), group=mob)
        # Native adoption owns the foreign-stage error. The source-level
        # fixture refuses at the same driver construction boundary.
        with self.assertRaisesRegex(ValueError, "foreign stage"):
            second.add(group.mobject)
        self.assertIs(mob._scene, first)


    def test_shipped_leaf_style_group_lifecycle_is_not_replaced(self):
        g = environment()
        base, animation = g["AnimationGroup"], g["Animation"]
        animation.get_all_mobjects = lambda self: (self.mobject,)
        class FollowingGroup(base):
            _native_kind = None
            def __init__(self, mob):
                super().__init__(g["Transform"](mob))
                self.mobject = mob
            def begin(self):
                animation.begin(self)
        g["FollowingGroup"] = FollowingGroup
        semantics._install_composition_lifecycle(g)
        mob = g["Mobject"]()
        group = FollowingGroup(mob)
        g["Scene"]().play(group)
        self.assertEqual(mob.x, 1.)
        self.assertIn(("update", .25), group.events)
        self.assertFalse(mob.animating)

    def test_shipped_leaf_style_group_abort_releases_state(self):
        g = environment()
        base, animation = g["AnimationGroup"], g["Animation"]
        animation.get_all_mobjects = lambda self: (self.mobject,)
        class FollowingGroup(base):
            _native_kind = None
            def __init__(self, mob):
                super().__init__(g["Transform"](mob))
                self.mobject = mob
            def begin(self):
                animation.begin(self)
        g["FollowingGroup"] = FollowingGroup
        semantics._install_composition_lifecycle(g)
        mob = g["Mobject"]()
        scene = g["Scene"]()
        scene.fail_updater = True
        with self.assertRaisesRegex(ValueError, "updater failed"):
            scene.play(FollowingGroup(mob))
        self.assertFalse(mob.animating)


if __name__ == "__main__":
    unittest.main()
