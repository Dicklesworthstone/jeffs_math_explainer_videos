"""Public recursive updates against native records, lifecycle and rendered frames."""
from pathlib import Path
import copy
import gc
import tempfile
import unittest
import weakref

import numpy as np
import manimlib as m


class MovingSquare(m.Square):
    def update(self, dt=0):
        self.shift(float(dt) * m.RIGHT)
        return super().update(dt)


def ticking(mobject):
    # Reference Mobject.update (mobject.py:826) returns before recursing when
    # the family has no updaters, so an override below it never runs. A no-op
    # updater on the override's own subtree keeps the dispatch under test live.
    return mobject.add_updater(lambda obj, dt: None, call=False)


class CopyMotion(m.Animation):
    def interpolate_submobject(self, current, starting, alpha):
        current.set_points(starting.get_points())


class LiteralCopyMotion(CopyMotion):
    def update_mobjects(self, dt):
        self.starting_mobject.shift(float(dt) * m.RIGHT)


def render_copy_motion(path, authored, threads):
    scene = m.Scene()
    leaf = ticking(MovingSquare()) if authored else m.Square()
    root = m.Group(leaf)
    animation = (CopyMotion if authored else LiteralCopyMotion)(
        root, run_time=.125, rate_func=m.linear, suspend_mobject_updating=True)
    with scene.render_session(path, resolution=(64, 40), fps=24, threads=threads):
        scene.play(animation)
    return path.read_bytes(), leaf.get_points()


class PublicUpdateTests(unittest.TestCase):
    def test_nested_dt_only_override_changes_native_geometry(self):
        for bound in (False, True):
            with self.subTest(bound=bound):
                child = ticking(MovingSquare())
                root = m.Group(m.Group(child))
                before = child.get_points().copy()
                if bound:
                    scene = m.Scene().add(root)
                self.assertIs(root.update(.25), root)
                np.testing.assert_array_equal(child.get_points(), before + .25 * m.RIGHT)
                self.assertIs(root[0][0], child)

    def test_updater_free_family_returns_before_descendant_updates(self):
        # Reference Mobject.update (mobject.py:826): `if not self.has_updaters()
        # ...: return self`. The pinned Reference (6199a00d) leaves each of these
        # untouched; an updater in the override's subtree makes it run.
        child = MovingSquare()
        m.Group(m.Group(child)).update(.25)
        np.testing.assert_array_equal(child.get_center(), m.ORIGIN)
        child = MovingSquare()
        m.Group(ticking(m.Group(child))).update(.25)
        np.testing.assert_array_equal(child.get_center(), .25 * m.RIGHT)
        # Called directly, an override is the receiver and runs.
        alone = MovingSquare().update(.25)
        np.testing.assert_array_equal(alone.get_center(), .25 * m.RIGHT)
        events = []
        child = m.Square(); root = m.Group(child)
        child.update = lambda dt: events.append(('instance', dt))
        root.update(.2)
        self.assertEqual(events, [])
        class Failing(m.Square):
            def update(self, dt):
                raise RuntimeError('an updater-free family must not reach this')
        self.assertIsInstance(m.Group(Failing()).update(.1), m.Group)
        child = MovingSquare(); scene = m.Scene().add(m.Group(child))
        scene.update_mobjects(.25)
        np.testing.assert_array_equal(child.get_center(), m.ORIGIN)
        root = m.Group(MovingSquare())
        animation = CopyMotion(root, rate_func=m.linear)
        animation.begin(); animation.update_mobjects(.25); animation.interpolate(.5)
        np.testing.assert_array_equal(root[0].get_center(), m.ORIGIN)
        animation.finish()
        # A native slot in the family keeps the walk that runs it.
        leaf = m.Square(); root = m.Group(m.Group(leaf)); scene = m.Scene().add(root)
        probe = scene._record_field_probe(leaf, 'point', 0)
        root.update(.25)
        self.assertEqual(len(probe.values()), 1)

    def test_virtual_entry_super_and_exit_order(self):
        events = []
        class Child(m.Square):
            def update(self, dt):
                events.append(('enter', dt))
                super().update(dt)
                events.append(('exit', dt))
                return self
        child = Child()
        child.add_updater(lambda obj, dt: events.append(('leaf', dt)), call=False)
        root = m.Group(child)
        root.add_updater(lambda obj, dt: events.append(('root', dt)), call=False)
        root.update(.5)
        self.assertEqual(events, [('enter', .5), ('leaf', .5), ('exit', .5), ('root', .5)])

    def test_native_phase_runs_once_child_first_after_all_host_callbacks(self):
        events = []
        class Probe:
            def _update_native_mobject(self, dt, recurse):
                events.append(('native', self.label, dt, recurse))
                return super()._update_native_mobject(dt, recurse)
        class Child(Probe, m.Square):
            def update(self, dt):
                return super().update(2 * dt)
        class Root(Probe, m.Group):
            pass
        child = Child(); child.label = 'child'
        root = Root(child); root.label = 'root'
        for obj in (root, child):
            obj.add_updater(lambda obj, dt: events.append(('host', obj.label, dt)), call=False)
        m.Scene().add(root)
        root.update(.25)
        self.assertEqual(events, [('host', 'child', .5), ('host', 'root', .25),
                                  ('native', 'child', .5, False), ('native', 'root', .25, False)])

    def test_ordinary_family_keeps_one_native_crossing(self):
        events = []
        class Root(m.Group):
            def _update_native_mobject(self, dt, recurse):
                events.append((dt, recurse))
                return super()._update_native_mobject(dt, recurse)
        root = Root(m.Group(ticking(m.Square()), m.Circle()))
        root.update(.25)
        self.assertEqual(events, [(.25, True)])
        # With no updater anywhere there is nothing to run: no crossing.
        events.clear()
        Root(m.Group(m.Square(), m.Circle())).update(.25)
        self.assertEqual(events, [])

    def test_override_selects_nonrecursive_super_for_both_phases(self):
        events = []
        class Child(m.Group):
            def update(self, dt):
                return super().update(dt, recurse=False)
        grandchild = MovingSquare()
        grandchild.add_updater(lambda obj, dt: events.append('grandchild'), call=False)
        child = Child(grandchild)
        child.add_updater(lambda obj, dt: events.append('child'), call=False)
        root = m.Group(child)
        before = grandchild.get_points().copy()
        root.update(.25)
        self.assertEqual(events, ['child'])
        np.testing.assert_array_equal(grandchild.get_points(), before)

    def test_override_omitting_super_owns_its_subtree(self):
        events = []
        class Child(m.Group):
            def update(self, dt):
                events.append(('override', dt))
                return self
            def _update_native_mobject(self, dt, recurse):
                raise AssertionError('an omitted base call must not receive native slots')
        child = Child(MovingSquare())
        child.add_updater(lambda obj, dt: events.append('base'), call=False)
        m.Group(child).update(.1)
        self.assertEqual(events, [('override', .1)])
        np.testing.assert_array_equal(child[0].get_center(), m.ORIGIN)

    def test_shared_children_keep_pathwise_public_visits(self):
        child = ticking(MovingSquare())
        root = m.Group(m.Group(child), m.Group(child))
        root.update(.125)
        np.testing.assert_array_equal(child.get_center(), .25 * m.RIGHT)
        self.assertIs(root[0][0], root[1][0])

    def test_suspension_and_self_only_scope_preserve_pruning(self):
        child = MovingSquare()
        root = ticking(m.Group(child))
        root.update(.2, recurse=False)
        root.suspend_updating(recurse=False).update(.2)
        np.testing.assert_array_equal(child.get_center(), m.ORIGIN)
        root.resume_updating(recurse=False, call_updater=False).update(.25)
        np.testing.assert_array_equal(child.get_center(), .25 * m.RIGHT)

    def test_updater_snapshot_is_taken_at_each_nodes_turn(self):
        events = []
        def later(obj, dt): events.append('later')
        class Child(m.Square):
            def update(self, dt):
                root.add_updater(later, call=False)
                return super().update(dt)
        child = Child()
        root = m.Group(child)
        def initial(obj, dt):
            events.append('initial')
            root.remove_updater(later)
        root.add_updater(initial, call=False)
        root.update(.1)
        self.assertEqual(events, ['initial', 'later'])

    def test_children_snapshot_survives_family_edits(self):
        events = []
        class Child(m.Square):
            def update(self, dt):
                events.append(self.label)
                if self.label == 'a':
                    root.remove(b); root.add(c)
                return super().update(dt)
        a, b, c = Child(), Child(), Child()
        a.label, b.label, c.label = 'a', 'b', 'c'
        root = ticking(m.Group(a, b))
        root.update(.1)
        self.assertEqual(events, ['a', 'b'])
        events.clear(); root.update(.1)
        self.assertEqual(events, ['a', 'c'])

    def test_late_instance_and_class_updates_are_live(self):
        events = []
        child = m.Square(); root = ticking(m.Group(child))
        child.update = lambda dt: events.append(('instance', dt))
        root.update(.2)
        class Child(m.Square): pass
        child = Child(); root = ticking(m.Group(child))
        Child.update = lambda self, dt: events.append(('class', dt))
        root.update(.3)
        self.assertEqual(events, [('instance', .2), ('class', .3)])

    def test_authored_failure_preserves_exception_and_releases_pending_work(self):
        events = []; failure = RuntimeError('child update failed')
        class Child(m.Square):
            def update(self, dt):
                super().update(dt)
                if dt > 0: raise failure
                return self
            def _update_native_mobject(self, dt, recurse):
                events.append(dt)
                return super()._update_native_mobject(dt, recurse)
        child = ticking(Child()); root = m.Group(child)
        with self.assertRaises(RuntimeError) as caught: root.update(.1)
        self.assertIs(caught.exception, failure)
        self.assertEqual(events, [])
        child.update(0)
        self.assertEqual(events, [0])
        root.update(0)
        self.assertEqual(events, [0, 0])

    def test_explicit_nested_update_is_an_independent_invocation(self):
        events = []
        class Child(m.Square):
            def _update_native_mobject(self, dt, recurse):
                events.append(('native', dt))
                return super()._update_native_mobject(dt, recurse)
        child = Child()
        def tick(obj, dt):
            events.append(('host', dt))
            if dt: obj.update(0)
        child.add_updater(tick, call=False)
        m.Group(child).update(.5)
        self.assertEqual(events, [('host', .5), ('host', 0.), ('native', 0.), ('native', .5)])

    def test_add_updater_and_resume_run_one_public_family_pass(self):
        events = []
        class Child(m.Square):
            def update(self, dt):
                events.append(dt)
                return super().update(dt)
        root = m.Group(Child())
        root.add_updater(lambda obj: None)
        self.assertEqual(events, [0.])
        root.suspend_updating().resume_updating()
        self.assertEqual(events, [0., 0.])

    def test_private_host_only_entry_does_not_run_native_slots(self):
        events = []
        class Child(m.Square):
            def update(self, dt):
                events.append(dt); return super().update(dt)
            def _update_native_mobject(self, dt, recurse):
                raise AssertionError('private host-only traversal ran native slots')
        m.Group(Child())._update_python_family(.25, True)
        self.assertEqual(events, [.25])

    def test_nested_private_host_pass_does_not_join_native_queue(self):
        events = []
        class Child(m.Square):
            def _update_native_mobject(self, dt, recurse):
                events.append(dt)
                return super()._update_native_mobject(dt, recurse)
        child = ticking(Child())
        other = m.Group(child)
        root = m.Group()
        root.add_updater(lambda obj, dt: other._update_python_family(dt, True), call=False)
        root.update(.25)
        self.assertEqual(events, [])
        other.update(.5)
        self.assertEqual(events, [.5])

    def test_copies_and_collection_do_not_retain_invocation_state(self):
        child = MovingSquare(); root = m.Group(child)
        root.add_updater(lambda obj, dt: copy.deepcopy(obj), call=False)
        root.update(.125)
        duplicate = root.copy()
        duplicate.update(.125)
        np.testing.assert_array_equal(child.get_center(), .125 * m.RIGHT)
        np.testing.assert_array_equal(duplicate[0].get_center(), .25 * m.RIGHT)
        references = [weakref.ref(obj) for obj in (root, child, duplicate)]
        del root, child, duplicate
        gc.collect()
        self.assertTrue(all(ref() is None for ref in references))

    def test_animation_helper_updates_reach_descendant_override(self):
        root = m.Group(ticking(MovingSquare()))
        animation = CopyMotion(root, rate_func=m.linear)
        animation.begin()
        animation.update_mobjects(.25)
        animation.interpolate(.5)
        np.testing.assert_array_equal(root[0].get_center(), .25 * m.RIGHT)
        animation.finish()

    def test_admission_scan_resolves_each_class_once(self):
        # fm-5wq.31: every frame's admission scan compared each hook of each
        # family member through inspect.getattr_static, O(members x hooks x
        # MRO) static lookups per frame (9.3M in 240 s of clt_proof's
        # DirectMGFInterpretation). The count must not grow with the family.
        import inspect
        original = inspect.getattr_static
        calls = []

        def counting(obj, name, *default):
            calls.append(name)
            return original(obj, name, *default)

        def lookups(count):
            scene = m.Scene()
            scene.add(*(m.Square() for _ in range(count)))
            calls.clear()
            inspect.getattr_static = counting
            try:
                self.assertFalse(scene._fmn_requires_public_scene_update())
            finally:
                inspect.getattr_static = original
            return len(calls)

        few, many = lookups(4), lookups(64)
        self.assertEqual(few, many)

        # The public update walk checked each member's _update_native_mobject
        # through getattr_static every frame (19 s of PrimeRace's first 120 s).
        import fmn_python.updater_dispatch as dispatch

        def walk_lookups(count):
            scene = m.Scene()
            scene.add(m.VGroup(*(m.Square() for _ in range(count))))
            calls.clear()
            # updater_dispatch binds getattr_static at import; count both names.
            inspect.getattr_static = dispatch.getattr_static = counting
            try:
                scene.update_mobjects(1 / 30)
            finally:
                inspect.getattr_static = dispatch.getattr_static = original
            return len(calls)

        self.assertEqual(walk_lookups(4), walk_lookups(64))
        # An instance-level override still defeats the per-class answer.
        scene = m.Scene()
        squares = [m.Square() for _ in range(8)]
        squares[5].update = lambda dt=0: None
        scene.add(*squares)
        self.assertTrue(scene._fmn_requires_public_scene_update())

    def test_admission_runs_no_authored_attribute_hook(self):
        # fm-5wq.31: the native admission scan reads own dictionaries with
        # plain getattr only for classes whose lookup is object's own. A
        # member whose class authors __getattribute__ or __getattr__ is
        # decided without calling either, and still needs the public path.
        calls = []

        class Intercepting(m.Square):
            def __getattribute__(self, name):
                calls.append(name)
                return object.__getattribute__(self, name)

        class Fallback(m.Square):
            def __getattr__(self, name):
                calls.append(name)
                raise AttributeError(name)

        for cls in (Intercepting, Fallback):
            with self.subTest(cls.__name__):
                scene = m.Scene()
                scene.add(m.VGroup(m.Square(), cls()))
                # Projecting the new roots reads each member's `submobjects`
                # by ordinary attribute access; that happens once per
                # topology change. This pins the scan itself.
                scene.mobjects
                calls.clear()
                self.assertTrue(scene._fmn_requires_public_scene_update())
                self.assertEqual(calls, [])

    def test_admission_scan_matches_the_per_member_oracle(self):
        # fm-5wq.31: the scan decides each class once and checks only the
        # instance dictionaries per member. The oracle is the previous loop,
        # which asked the movement helpers about every member.
        scan = m.Scene._fmn_requires_public_scene_update
        cells = dict(zip(scan.__code__.co_freevars, (cell.cell_contents for cell in scan.__closure__)))

        def oracle(scene):
            memo = {}
            changed, implementation = cells["_changed"], cells["_class_implementation"]
            if changed(scene, cells["scene_protocols"], memo):
                return True
            expected = next(cells["scene_descriptors"][cls] for cls in type(scene).__mro__
                            if cls in cells["scene_descriptors"])
            if any(implementation(type(scene), name, memo) is not method
                   for name, method in expected.items()):
                return True
            pending, seen = [scene.frame, *scene.mobjects], set()
            while pending:
                member = pending.pop()
                if id(member) in seen:
                    continue
                seen.add(id(member))
                if not isinstance(member, cells["Mobject"]) or changed(
                        member, cells["object_protocols"], memo):
                    return True
                expected = next(cells["child_descriptors"][cls] for cls in type(member).__mro__
                                if cls in cells["child_descriptors"])
                if implementation(type(member), "submobjects", memo) is not expected:
                    return True
                children = member.submobjects
                methods = cells["child_protocols"].get(type(children))
                if methods is None or any(
                        cells["_scan_implementation"](children, name, memo) is not method
                        for name, method in methods.items()):
                    return True
                pending.extend(children)
            return False

        class Authored(m.Square):
            def update(self, dt=0, recurse=True):
                return super().update(dt, recurse)

        class Late(m.Square):
            pass

        class Children(list):
            pass

        def scene_with(edit):
            squares = [m.Square() for _ in range(64)]
            scene = m.Scene()
            scene.add(m.VGroup(*squares[:32]), m.VGroup(m.VGroup(*squares[32:])), m.Circle())
            edit(scene, squares)
            return scene

        edits = {
            "plain": lambda scene, squares: None,
            "ordinary instance attributes": lambda scene, squares: [
                setattr(square, "label", index) for index, square in enumerate(squares)],
            "authored class deep in a group": lambda scene, squares: squares[40].add(Authored()),
            "instance hook on one member": lambda scene, squares: setattr(
                squares[50], "update", lambda dt=0: None),
            "instance entry that is the class method": lambda scene, squares: setattr(
                squares[7], "update", squares[7].update),
            # The first Square the scan visits holds the class's own function:
            # decided per object, never as the class's answer.
            "shipped function in the first member's dict": lambda scene, squares: vars(
                squares[-1]).__setitem__("update", m.Square.update),
            "shipped function first, a real hook later": lambda scene, squares: (
                vars(squares[-1]).__setitem__("update", m.Square.update),
                setattr(squares[33], "update", lambda dt=0: None)),
            "class patched after construction": lambda scene, squares: (
                squares[3].add(Late()), setattr(Late, "update", lambda self, dt=0: None)),
            "foreign container type": lambda scene, squares: setattr(
                squares[9], "submobjects", Children()),
            "container with its own __iter__": lambda scene, squares: vars(
                squares[11].submobjects).__setitem__("__iter__", lambda: iter(())),
        }
        verdicts = {}
        for name, edit in edits.items():
            with self.subTest(name):
                try:
                    scene = scene_with(edit)
                except Exception as error:  # a refusal at authoring time is not a scan case
                    verdicts[name] = f"refused: {type(error).__name__}"
                    continue
                verdicts[name] = oracle(scene)
                self.assertEqual(scene._fmn_requires_public_scene_update(), verdicts[name])
        self.assertIs(verdicts["plain"], False)
        self.assertIs(verdicts["ordinary instance attributes"], False)
        self.assertIs(verdicts["authored class deep in a group"], True)
        self.assertIs(verdicts["instance hook on one member"], True)
        self.assertIs(verdicts["shipped function first, a real hook later"], True)

    def test_play_admission_walks_resolve_each_class_once(self):
        # fm-5wq.31: every play walked its animations' families four times
        # (playback's authored_family, rotation, and indication's two
        # walks), resolving every hook of every member through
        # getattr_static: ~27,500 calls per play of a 50-square group, 90%
        # of the play's time. The count must not grow with the family.
        import inspect
        original, calls = inspect.getattr_static, []

        def counting(obj, name, *default):
            calls.append(name)
            return original(obj, name, *default)

        def play_lookups(count):
            scene = m.Scene()
            group = m.VGroup(*(m.Square() for _ in range(count)))
            scene.add(group)
            calls.clear()
            inspect.getattr_static = counting
            try:
                scene.play(group.animate.shift(0.1 * m.UP), run_time=1 / 30)
            finally:
                inspect.getattr_static = original
            return len(calls)

        self.assertEqual(play_lookups(4), play_lookups(64))
        # The per-class answer never hides one member's own override.
        requires = vars(getattr(m, "_native", m))["_requires_python_animation"]
        squares = [m.Square() for _ in range(8)]
        group = m.VGroup(*squares)
        self.assertFalse(requires(m.Rotate(group, 0.1)))
        squares[5].rotate = lambda *args, **kwargs: squares[5]
        self.assertTrue(requires(m.Rotate(group, 0.1)))

    def test_scene_public_update_mobjects_dispatches_children(self):
        child = ticking(MovingSquare()); scene = m.Scene().add(m.Group(child))
        scene.update_mobjects(.25)
        np.testing.assert_array_equal(child.get_center(), .25 * m.RIGHT)

    def test_rendered_snapshot_updates_match_literal_control(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            actual, points = render_copy_motion(path / 'one.y4m', True, 1)
            for threads in (4, 16):
                self.assertEqual(actual, render_copy_motion(path / f'{threads}.y4m', True, threads)[0])
            expected, expected_points = render_copy_motion(path / 'expected.y4m', False, 1)
            self.assertEqual(actual, expected)
            np.testing.assert_array_equal(points, expected_points)
            frames = actual.split(b'FRAME\n')[1:]
            self.assertEqual(len(frames), 3)
            self.assertNotEqual(frames[0], frames[1])
            self.assertNotEqual(frames[1], frames[2])


class PublicSceneUpdateTests(unittest.TestCase):
    def test_camera_only_callbacks_are_detected(self):
        scene = m.Scene()
        self.assertIs(scene.should_update_mobjects(), False)
        callback = lambda frame, dt: None
        scene.frame.add_updater(callback, call=False)
        self.assertIs(scene.should_update_mobjects(), True)
        scene.frame.remove_updater(callback)
        self.assertIs(scene.should_update_mobjects(), False)

    def test_deep_descendant_callbacks_are_detected_without_running_them(self):
        calls = []
        leaf = m.Square()
        leaf.add_updater(lambda obj, dt: calls.append(dt), call=False)
        root = m.Group(m.Group(leaf))
        scene = m.Scene().add(root)
        self.assertFalse(root.updaters)
        self.assertIs(scene.should_update_mobjects(), True)
        self.assertEqual(calls, [])
        leaf.clear_updaters()
        self.assertIs(scene.should_update_mobjects(), False)

    def test_authored_has_updaters_is_called_on_actual_root(self):
        calls = []
        class Group(m.Group):
            def has_updaters(self):
                calls.append(self)
                return True
        root = Group()
        scene = m.Scene().add(root)
        self.assertIs(scene.should_update_mobjects(), True)
        self.assertEqual(calls, [root])

    def test_always_update_short_circuits_family_queries(self):
        scene = m.Scene(always_update_mobjects=True)
        scene.frame = None
        self.assertIs(scene.should_update_mobjects(), True)

    def test_camera_is_updated_before_drawable_callbacks(self):
        scene = m.Scene()
        events = []
        root = m.Group(m.Square())
        def camera(frame, dt):
            events.append(('camera', dt))
            frame.shift(dt * m.RIGHT)
        scene.frame.add_updater(camera, call=False)
        root.add_updater(lambda obj, dt: events.append(('root', scene.frame.get_x())), call=False)
        scene.add(root)
        self.assertIsNone(scene.update_mobjects(.25))
        self.assertEqual(events, [('camera', .25), ('root', .25)])

    def test_replacement_frame_uses_public_dt_only_update(self):
        calls = []
        class Frame(m.CameraFrame):
            def update(self, dt):
                calls.append(dt)
                return super().update(dt)
        scene = m.Scene()
        scene.frame = Frame()
        scene.update_mobjects(.125)
        self.assertEqual(calls, [.125])

    def test_camera_suspension_prunes_its_callbacks_not_drawable_roots(self):
        scene = m.Scene()
        events = []
        scene.frame.add_updater(lambda obj, dt: events.append('camera'), call=False)
        leaf = m.Square().add_updater(lambda obj, dt: events.append('leaf'), call=False)
        scene.add(leaf)
        scene.frame.suspend_updating()
        scene.update_mobjects(.25)
        self.assertEqual(events, ['leaf'])
        scene.frame.resume_updating(call_updater=False)
        scene.update_mobjects(.25)
        self.assertEqual(events, ['leaf', 'camera', 'leaf'])

    def test_camera_family_edits_apply_to_the_next_root_snapshot(self):
        scene = m.Scene()
        events = []
        old = m.Square().add_updater(lambda obj, dt: events.append('old'), call=False)
        new = m.Circle().add_updater(lambda obj, dt: events.append('new'), call=False)
        scene.add(old)
        scene.frame.add_updater(lambda frame, dt: scene.remove(old).add(new), call=False)
        scene.update_mobjects(.125)
        self.assertEqual(events, ['old'])
        scene.update_mobjects(.125)
        self.assertEqual(events, ['old', 'new'])

    def test_frame_failure_stops_root_callbacks_and_scene_can_recover(self):
        scene = m.Scene()
        events = []
        error = RuntimeError('camera updater failed')
        def fail(frame, dt): raise error
        scene.frame.add_updater(fail, call=False)
        scene.add(m.Square().add_updater(lambda obj, dt: events.append(dt), call=False))
        with self.assertRaises(RuntimeError) as caught: scene.update_mobjects(.25)
        self.assertIs(caught.exception, error)
        self.assertEqual(events, [])
        scene.frame.remove_updater(fail)
        scene.update_mobjects(.5)
        self.assertEqual(events, [.5])

    def test_update_frame_uses_post_advance_clock_even_when_skipping(self):
        scene = m.Scene(skip_animations=True)
        times = []
        scene.frame.add_updater(lambda frame, dt: times.append((dt, scene.get_time())), call=False)
        self.assertIsNone(scene.update_frame(.25))
        self.assertEqual(times, [(.25, .25)])

    def test_public_frame_pixels_match_independent_camera_and_geometry_updates(self):
        def render(mode, threads):
            scene = m.Scene(camera_config={'resolution': (64, 40)})
            square = m.Square(side_length=1, fill_opacity=1, stroke_width=0)
            scene.add(m.Group(square))
            camera = scene.camera
            camera.capture_threads = threads
            if mode == 'public':
                scene.frame.add_updater(lambda frame, dt: frame.shift(.5 * dt * m.UP), call=False)
                square.add_updater(lambda obj, dt: obj.shift(dt * m.RIGHT), call=False)
            frames = []
            for _ in range(3):
                if mode == 'public':
                    scene.update_frame(.125)
                else:
                    scene.increment_time(.125)
                    square.shift(.125 * m.RIGHT)
                    if mode == 'control': scene.frame.shift(.0625 * m.UP)
                frames.append(camera.capture_snapshot(*scene.mobjects).png())
            return frames
        actual = render('public', 1)
        self.assertEqual(actual, render('control', 1))
        self.assertNotEqual(actual, render('without_camera', 1))
        for threads in (4, 16): self.assertEqual(actual, render('public', threads))
        self.assertEqual(len(set(actual)), 3)


if __name__ == '__main__':
    unittest.main()
