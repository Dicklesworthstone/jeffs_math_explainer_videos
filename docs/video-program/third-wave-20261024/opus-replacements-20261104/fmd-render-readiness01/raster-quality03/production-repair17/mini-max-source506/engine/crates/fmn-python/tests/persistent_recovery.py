"""Native persistent-animation failure and cancellation ownership."""
import numpy as np
import manimlib as m

_MISSING = object()


class Probe(m.Animation):
    def __init__(self, root, affected, phase=None, broken_abort=False, self_cancel=False):
        self.affected = affected
        self.phase = phase
        self.broken_abort = broken_abort
        self.self_cancel = self_cancel
        self.error = LookupError("persistent primary error")
        self.abort_error = RuntimeError("persistent abort error")
        self.abort_calls = 0
        self.failed_updates = None
        self.updates = []
        super().__init__(root, run_time=1, rate_func=m.linear)

    def corrupt(self):
        for member in self.affected:
            member.locked_data_keys = {"temporary-lock"}
            member.const_data_keys.add("point")
            member.locked_uniform_keys = {"temporary-uniform"}
            member._is_animating = not member._is_animating
            if member._is_updating_suspended():
                member.resume_updating(recurse=False, call_updater=False)
            else:
                member.suspend_updating(recurse=False)

    def fail(self, phase):
        if self.phase == phase:
            self.failed_updates = len(self.updates)
            self.corrupt()
            self.mobject.shift(m.UP)
            raise self.error

    def begin(self):
        super().begin()
        self.fail("begin")

    def interpolate_mobject(self, alpha):
        if alpha > 0:
            if self.self_cancel:
                self.mobject.clear_updaters(recurse=False)
            self.fail("interpolate")

    def update_mobjects(self, dt):
        if dt > 0:
            self.fail("helpers")

    def finish(self):
        self.fail("finish")
        super().finish()

    def abort(self):
        self.abort_calls += 1
        self.corrupt()
        if self.broken_abort:
            raise self.abort_error


def save(objects):
    return [(obj, obj._is_updating_suspended(), vars(obj).get("_is_animating", _MISSING),
             [(key, getattr(obj, key), set(getattr(obj, key)))
              for key in ("locked_data_keys", "const_data_keys", "locked_uniform_keys")])
            for obj in objects]


def restored(states):
    for obj, suspended, animating, locks in states:
        assert obj._is_updating_suspended() == suspended
        assert vars(obj).get("_is_animating", _MISSING) == animating
        for key, original, values in locks:
            assert getattr(obj, key) is original, key
            assert getattr(obj, key) == values, key


def fixture(bound):
    child, sibling = m.Square(), m.Circle()
    root = m.VGroup(child, sibling)
    parent = m.Group(root)
    scene = m.Scene()
    if bound:
        scene.add(parent)
    for index, obj in enumerate((parent, root, child, sibling)):
        obj._is_animating = index % 2 == 0
        obj.locked_data_keys = {"prior-" + str(index)}
        obj.const_data_keys = {"prior-const"}
        obj.locked_uniform_keys = {"prior-uniform"}
    child.suspend_updating()
    # Ancestors are initially unsuspended so a root updater can execute.
    return scene, root, (parent, root, child, sibling)


def failure_phase_matrix():
    count = 0
    for bound in (False, True):
        for phase in ("begin", "interpolate", "helpers", "finish"):
            for broken_abort in (False, True):
                scene, root, objects = fixture(bound)
                before = save(objects)
                animation = Probe(root, objects, phase, broken_abort)
                for obj in objects:
                    obj.add_updater(lambda obj, dt: animation.updates.append(dt), call=False)
                try:
                    m.turn_animation_into_updater(animation)
                    root.update(1)
                    if phase == "interpolate":
                        animation.total_time = .5
                    root.update(0)
                except LookupError as error:
                    assert error is animation.error
                else:
                    raise AssertionError("persistent failure did not reach caller")
                assert animation.abort_calls == 1
                assert len(animation.updates) == animation.failed_updates
                restored(before)
                assert all(not hasattr(updater, "_fmn_persistent_controller") for updater in root.updaters)
                if broken_abort:
                    assert any("abort" in note for note in animation.error.__notes__)
                # Transients are restored, never the geometry already executed.
                np.testing.assert_allclose(root.get_center(), m.UP)
                count += 1
    return count


def explicit_cancellation_restores_state_and_reports_abort_failure():
    for broken in (False, True):
        scene, root, objects = fixture(True)
        before = save(objects)
        animation = Probe(root, objects, broken_abort=broken)
        m.cycle_animation(animation)
        try:
            root.clear_updaters(recurse=False)
        except RuntimeError as error:
            assert broken and error is animation.abort_error
        else:
            assert not broken
        restored(before)
        assert animation.abort_calls == 1 and not root.updaters
        root.clear_updaters()
        assert animation.abort_calls == 1
        assert scene.mobjects == [objects[0]]


def self_cancellation_then_failure_unwinds_once():
    scene, root, objects = fixture(True)
    before = save(objects)
    animation = Probe(root, objects, "interpolate", broken_abort=True, self_cancel=True)
    m.cycle_animation(animation)
    root.update(.5)
    try:
        root.update(0)
    except LookupError as error:
        assert error is animation.error
    else:
        raise AssertionError("post-cancellation failure disappeared")
    restored(before)
    assert animation.abort_calls == 1 and not root.updaters


def nested_children_and_external_ancestors_are_recovered():
    scene, root, objects = fixture(True)
    first = Probe(root, objects, "interpolate", broken_abort=True)
    other = m.Square().shift(2 * m.RIGHT)
    scene.add(other)
    other._is_animating = True
    group = m.AnimationGroup(m.AnimationGroup(first), m.Rotate(other, angle=.5),
                              group=m.Group(root, other), run_time=1, rate_func=m.linear)
    before = save((*objects, other, group.mobject))
    anchor = m.turn_animation_into_updater(group)
    anchor.update(.5)
    try:
        anchor.update(0)
    except LookupError as error:
        assert error is first.error
    else:
        raise AssertionError("nested background failure disappeared")
    restored(before)
    assert first.abort_calls == 1 and not anchor.updaters
    # The original scene roots are not removed or replaced by updater cleanup.
    assert scene.mobjects == [objects[0], other]


def wait_failure_leaves_the_same_native_scene_usable():
    scene, root, objects = fixture(True)
    before = save(objects)
    animation = Probe(root, objects, "interpolate", broken_abort=True)
    m.turn_animation_into_updater(animation)
    try:
        scene.wait(.25)
    except LookupError as error:
        assert error is animation.error
    else:
        raise AssertionError("native wait hid a persistent failure")
    restored(before)
    assert not root.updaters and animation.abort_calls == 1
    assert "_fmn_scene_execution" not in vars(scene)
    at_failure = scene.time
    scene.wait(.125)
    assert scene.time > at_failure
    np.testing.assert_allclose(root.get_center(), m.UP)


def cancellation_before_adoption_never_begins_a_group():
    scene, root, objects = fixture(False)
    animation = Probe(root, objects, "begin")
    group = m.AnimationGroup(animation)
    before = save(objects)
    anchor = m.turn_animation_into_updater(group)
    anchor.clear_updaters()
    assert animation.abort_calls == 0
    restored(before)
    scene.add(anchor)
    scene.wait(.125)
    np.testing.assert_allclose(root.get_center(), m.ORIGIN)


count = failure_phase_matrix()
for case in (explicit_cancellation_restores_state_and_reports_abort_failure,
             self_cancellation_then_failure_unwinds_once,
             nested_children_and_external_ancestors_are_recovered,
             wait_failure_leaves_the_same_native_scene_usable,
             cancellation_before_adoption_never_begins_a_group):
    case()
print(f"persistent recovery: {count} phase/ownership combinations and 5 cancellation/composition/wait cases passed")
