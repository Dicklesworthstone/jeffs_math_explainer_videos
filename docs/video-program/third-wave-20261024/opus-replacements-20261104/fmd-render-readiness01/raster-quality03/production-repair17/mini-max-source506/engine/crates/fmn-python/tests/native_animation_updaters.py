"""Installed-extension witnesses for scene-bound persistent native animations."""
import numpy as np
from manimlib import Scene, Square, Swap, LEFT, RIGHT, linear, turn_animation_into_updater, cycle_animation


def setup_scene():
    scene = Scene()
    left, right = Square().shift(2 * LEFT), Square().shift(2 * RIGHT)
    scene.add(left, right)
    return scene, left, right


def native_swap_midpoint_and_finish():
    scene, left, right = setup_scene()
    roots = list(scene.mobjects)
    animation = Swap(left, right, path_arc=0, rate_func=linear, run_time=1)
    # As in the Reference, the updater lives on Swap's public group root
    # (animation.mobject), not on an operand.
    group = turn_animation_into_updater(animation)
    assert group is animation.mobject
    assert all(any(member is operand for member in group.get_family()) for operand in (left, right))
    group.update(.5)
    group.update(0)
    np.testing.assert_allclose(left.get_center(), (0, 0, 0), atol=1e-5)
    np.testing.assert_allclose(right.get_center(), (0, 0, 0), atol=1e-5)
    group.update(.5)
    group.update(0)
    np.testing.assert_allclose(left.get_center(), 2 * RIGHT, atol=1e-5)
    np.testing.assert_allclose(right.get_center(), 2 * LEFT, atol=1e-5)
    assert not group.updaters
    assert all(any(root is actual for actual in scene.mobjects) for root in roots)


def native_swap_cycles():
    _, left, right = setup_scene()
    group = cycle_animation(Swap(left, right, path_arc=0, rate_func=linear, run_time=1))
    group.update(2.25)
    group.update(0)
    np.testing.assert_allclose(left.get_center(), LEFT, atol=1e-5)
    np.testing.assert_allclose(right.get_center(), RIGHT, atol=1e-5)
    assert group.updaters
    group.clear_updaters()


def explicit_removal_keeps_current_pose():
    _, left, right = setup_scene()
    group = cycle_animation(Swap(left, right, path_arc=0, rate_func=linear))
    group.update(.25)
    group.update(0)
    expected = left.get_points().copy(), right.get_points().copy()
    group.clear_updaters()
    group.update(2)
    np.testing.assert_array_equal(left.get_points(), expected[0])
    np.testing.assert_array_equal(right.get_points(), expected[1])
    assert not group.updaters


def copied_updaters_do_not_control_the_source():
    _, left, right = setup_scene()
    animation = Swap(left, right, path_arc=0, rate_func=linear)
    group = cycle_animation(animation)
    duplicate = group.copy()
    duplicate.update(5)
    duplicate.clear_updaters()
    assert animation.total_time == 0
    assert group.updaters
    group.clear_updaters()


def wait_drives_native_background_effect():
    scene, left, right = setup_scene()
    group = turn_animation_into_updater(Swap(left, right, path_arc=0, rate_func=linear, run_time=.1))
    scene.add(group)  # the Scene updates its roots; the updater is on the group
    scene.wait(.3)
    np.testing.assert_allclose(left.get_center(), 2 * RIGHT, atol=1e-5)
    np.testing.assert_allclose(right.get_center(), 2 * LEFT, atol=1e-5)
    assert not group.updaters


def invalid_dt_recovers():
    _, left, right = setup_scene()
    group = turn_animation_into_updater(Swap(left, right, path_arc=0, rate_func=linear))
    try:
        group.update(float("nan"))
    except ValueError:
        pass
    else:
        raise AssertionError("nonfinite updater dt must fail")
    assert not group.updaters
    assert not group._is_updating_suspended() and not left._is_updating_suspended()
    group = turn_animation_into_updater(Swap(left, right, path_arc=0, rate_func=linear))
    group.update(1)
    group.update(0)
    np.testing.assert_allclose(left.get_center(), 2 * RIGHT, atol=1e-5)


for case in (native_swap_midpoint_and_finish, native_swap_cycles,
             explicit_removal_keeps_current_pose, copied_updaters_do_not_control_the_source,
             wait_drives_native_background_effect, invalid_dt_recovers):
    case()
print("native persistent animation acceptance: 6 cases passed")
