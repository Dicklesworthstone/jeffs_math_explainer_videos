"""Focused acceptance tests for the shared Animation/Transform protocol."""

import math

import numpy as np

from manimlib import (
    Animation,
    AnimationGroup,
    Circle,
    CyclicReplace,
    DrawBorderThenFill,
    FadeTransform,
    FadeTransformPieces,
    Group,
    LaggedStart,
    LaggedStartMap,
    Mobject,
    ReplacementTransform,
    Scene,
    ShowCreation,
    Square,
    Succession,
    Swap,
    Transform,
    TransformFromCopy,
    VMobject,
    VGroup,
)


class AnimationProbe(Animation):
    def __init__(self, mobject, **kwargs):
        self.calls = []
        super().__init__(mobject, **kwargs)

    def interpolate_submobject(self, current, starting, alpha):
        self.calls.append((current, starting, float(alpha)))


assert str(Mobject()) == "Mobject"
try:
    Animation(None)
except TypeError as error:
    assert str(error) == "Animation only works for Mobjects."
else:
    raise AssertionError("Animation accepted a non-Mobject target")
assert str(Animation(Mobject())) == "AnimationMobject"

# Public helpers must operate on real native-backed copies and records. A
# successful animation endpoint alone never exercises these callable contracts.
outline_source = VGroup(
    Square(fill_opacity=0.75, stroke_color="#FF0000", stroke_width=5),
    Circle(fill_opacity=0.5, stroke_color="#00FF00", stroke_width=7).shift((3, 0, 0)),
)
outline_source.stroke_behind = True
outline_points = [mob.get_points().copy() for mob in outline_source.family_members_with_points()]
outline_animation = DrawBorderThenFill(outline_source, stroke_width=3)
outline_copy = outline_animation.get_outline()
assert outline_copy is not outline_source
assert len(outline_copy.family_members_with_points()) == len(outline_points) == 2
for index, (actual, original) in enumerate(zip(
    outline_copy.family_members_with_points(), outline_source.family_members_with_points(),
)):
    assert actual is not original
    assert np.array_equal(actual.get_points(), outline_points[index])
    assert actual.get_fill_opacity() == 0
    assert actual.get_stroke_width() == 3
    assert actual.get_stroke_color() == original.get_stroke_color()
    assert actual.get_uniforms()["stroke_behind"]
assert outline_source[0].get_fill_opacity() == 0.75
assert outline_source[0].get_stroke_width() == 5
outline_copy.shift((0, 2, 0))
assert np.array_equal(outline_source[0].get_points(), outline_points[0])
colored_outline = DrawBorderThenFill(
    outline_source, stroke_color="#0000FF",
).get_outline()
assert all(mob.get_stroke_color().upper() == "#0000FF"
           for mob in colored_outline.family_members_with_points())

ghost_source = Square(side_length=2, fill_opacity=0.8, stroke_width=3)
ghost_target = Square(side_length=1).stretch(3, 0).shift((4, 2, 0))
ghost_target.set_uniform(flat_stroke=True, fixed_orientation_center=(4.0, 2.0, 0.0))
ghost_target_points = ghost_target.get_points().copy()
ghost = FadeTransform(ghost_source, ghost_target, stretch=True)
assert ghost.ghost_to(ghost_source, ghost_target) is None
assert np.allclose(ghost_source.get_center(), ghost_target.get_center())
assert np.allclose([ghost_source.get_width(), ghost_source.get_height()], [3, 1])
assert ghost_source.get_fill_opacity() == ghost_source.get_stroke_opacity() == 0
assert ghost_source.get_uniforms()["flat_stroke"]
assert np.allclose(ghost_source.get_uniforms()["fixed_orientation_center"], [4, 2, 0])
assert np.array_equal(ghost_target.get_points(), ghost_target_points)

aspect_source = Square(side_length=2)
aspect_ghost = FadeTransform(aspect_source, ghost_target, stretch=False, dim_to_match=1)
aspect_ghost.ghost_to(aspect_source, ghost_target)
assert np.allclose([aspect_source.get_width(), aspect_source.get_height()], [1, 1])
assert np.allclose(aspect_source.get_center(), ghost_target.get_center())

piece_source = VGroup(Square(), Square().shift((2, 0, 0)))
piece_target = VGroup(Square().scale(0.25).shift((-3, 1, 0)),
                      Square().scale(0.75).shift((4, -2, 0)))
piece_targets = [mob.get_points().copy() for mob in piece_target.submobjects]
pieces = FadeTransformPieces(piece_source, piece_target)
assert pieces.ghost_to(piece_source, piece_target) is None
assert len(piece_source.submobjects) == len(piece_target.submobjects) == 2
for actual, target, before in zip(piece_source, piece_target, piece_targets):
    assert np.allclose(actual.get_center(), target.get_center())
    assert np.allclose([actual.get_width(), actual.get_height()],
                       [target.get_width(), target.get_height()])
    assert actual.get_fill_opacity() == actual.get_stroke_opacity() == 0
    assert np.array_equal(target.get_points(), before)

root = Mobject().add(Mobject(), Mobject())
square_rate = lambda value: value * value
animation = AnimationProbe(
    root,
    run_time=2.0,
    lag_ratio=0.5,
    rate_func=square_rate,
    name="probe",
)
assert str(animation) == "probe"
assert animation.get_run_time() == 2.0
assert animation.get_rate_func() is square_rate
animation.begin()
assert root.is_changing()
assert animation.starting_mobject is not root
assert len(animation.families) == 3
assert all(len(row) == 2 for row in animation.families)
assert [row[0] for row in animation.families] == root.get_family()
assert [row[1] for row in animation.families] == animation.starting_mobject.get_family()

animation.calls.clear()
assert animation.interpolate(0.5) is None
assert [call[2] for call in animation.calls] == [1.0, 0.25, 0.0]
animation.calls.clear()
assert animation.update(0.25) is None
assert [call[2] for call in animation.calls] == [0.25, 0.0, 0.0]

helper_ticks = []
live_ticks = []
helper_updater = lambda mob, dt=0: helper_ticks.append(float(dt))
live_updater = lambda mob, dt=0: live_ticks.append(float(dt))
animation.starting_mobject.add_updater(helper_updater, call=False)
root.add_updater(live_updater, call=False)
assert animation.get_all_mobjects_to_update() == [animation.starting_mobject]
animation.update_mobjects(0.125)
assert helper_ticks == [0.125]
assert live_ticks == []
root.remove_updater(live_updater)

original_get_all = animation.get_all_mobjects
animation.get_all_mobjects = lambda: (
    animation.mobject,
    animation.starting_mobject,
    animation.starting_mobject,
)
assert animation.get_all_mobjects_to_update() == [animation.starting_mobject]
animation.get_all_mobjects = original_get_all

clone = animation.copy()
assert clone is not animation
assert clone.mobject is not animation.mobject
linear_rate = lambda value: value
assert animation.update_rate_info(4.0, linear_rate, 0.75) is animation
assert (animation.run_time, animation.lag_ratio) == (4.0, 0.75)
assert animation.rate_func is linear_rate
# BN-12 deliberately fixes the Reference's truthiness-based no-op.
animation.update_rate_info(0, None, 0)
assert (animation.run_time, animation.lag_ratio) == (0, 0)
assert animation.set_run_time(3.0) is animation
assert animation.set_rate_func(square_rate) is animation
assert animation.set_name("renamed") is animation
assert str(animation) == "renamed"
animation.finish()
assert not root.is_changing()

suspended_root = Mobject()
suspended = AnimationProbe(
    suspended_root,
    suspend_mobject_updating=True,
)
suspended.begin()
assert suspended_root._is_updating_suspended()
suspended.finish()
assert not suspended_root._is_updating_suspended()
assert not suspended_root.is_changing()

spanned = AnimationProbe(
    Mobject(),
    run_time=1.0,
    time_span=(0.25, 0.75),
)
assert spanned.time_spanned_alpha(0.0) == 0.0
assert spanned.time_spanned_alpha(0.5) == 0.5
assert spanned.time_spanned_alpha(1.0) == 1.0

native_animation = ShowCreation(VMobject().set_points([[0.0, 0.0, 0.0]]))
assert str(native_animation) == "ShowCreationVMobject"
assert native_animation.is_remover() is False

source = Mobject().set_points([[0.0, 0.0, 0.0]])
target = Mobject().set_points([
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
])
transform = Transform(
    source,
    target,
    run_time=1.0,
    rate_func=linear_rate,
)
transform.begin()
assert source.is_changing()
assert transform.target_copy is not target
assert source.get_num_points() == 3
assert transform.starting_mobject.get_num_points() == 3
assert len(transform.families) == 1
assert transform.families[0] == (
    source,
    transform.starting_mobject,
    transform.target_copy,
)
assert "rgba" in source.locked_data_keys
transform.interpolate(0.5)
assert np.allclose(source.get_points()[-1], [1.0, 0.0, 0.0])

transform_ticks = []
source_ticks = []
source_updater = lambda mob, dt=0: source_ticks.append(float(dt))
source.add_updater(source_updater, call=False)
for label, helper in (
    ("start", transform.starting_mobject),
    ("target", target),
    ("target_copy", transform.target_copy),
):
    helper.add_updater(
        lambda mob, dt=0, helper_label=label: transform_ticks.append(
            (helper_label, float(dt))
        ),
        call=False,
    )
assert transform.get_all_mobjects_to_update() == [
    transform.starting_mobject,
    target,
    transform.target_copy,
]
transform.update_mobjects(0.2)
assert transform_ticks == [
    ("start", 0.2),
    ("target", 0.2),
    ("target_copy", 0.2),
]
assert source_ticks == []
source.remove_updater(source_updater)
transform.finish()
assert not source.is_changing()
assert source.locked_data_keys == set()

arc_source = Mobject().set_points([[1.0, 0.0, 0.0]])
arc_target = Mobject().set_points([[-1.0, 0.0, 0.0]])
arc_source.set_color("#000000")
arc_target.set_color("#FFFFFF")
arc_source.set_shading(0.0, 0.0, 0.0)
arc_target.set_shading(1.0, 0.5, 0.25)
arc_source.uniforms["clip_planes"] = [[0.0] * 4 for _ in range(4)]
arc_target.uniforms["clip_planes"] = [[2.0, 4.0, 6.0, 8.0] for _ in range(4)]
arc_source.uniforms["depth_test"] = True
arc_target.uniforms["depth_test"] = False
arc_source.uniforms["joint_type"] = 2.0
arc_target.uniforms["joint_type"] == 3.0
arc_transform = Transform(
    arc_source,
    arc_target,
    path_arc=math.pi,
    run_time=1.0,
    rate_func=linear_rate,
)
arc_transform.begin()
assert arc_transform.target_copy is arc_target
assert arc_transform.get_all_mobjects_to_update() == [
    arc_transform.starting_mobject,
    arc_target,
]
arc_transform.interpolate(0.5)
midpoint = arc_source.get_points()[0]
assert abs(float(midpoint[0])) < 1e-5
assert abs(abs(float(midpoint[1])) - 1.0) < 1e-5
assert np.allclose(
    arc_source.data["rgba"][0, :3],
    [0.5, 0.5, 0.5],
    atol=1e-6,
)
assert np.allclose(
    arc_source.get_shading(),
    [0.5, 0.25, 0.125],
    atol=1e-6,
)
assert np.allclose(arc_source.uniforms["clip_planes"], [[1.0, 2.0, 3.0, 4.0]] * 4)
assert arc_source.uniforms["depth_test"] is True
assert arc_source.uniforms["joint_type"] == 2.0
arc_transform.finish()
assert arc_source.uniforms["depth_test"] is True
assert arc_source.uniforms["joint_type"] == 2.0

copy_source = Mobject().set_points([[0.0, 0.0, 0.0]])
copy_target = Mobject().set_points([[1.0, 0.0, 0.0]])
copy_transform = TransformFromCopy(copy_source, copy_target)
assert copy_transform.mobject is not copy_source
assert np.array_equal(copy_transform.mobject.get_points(), copy_source.get_points())
assert ReplacementTransform.replace_mobject_with_target_in_scene
assert TransformFromCopy.replace_mobject_with_target_in_scene


class SceneProbe:
    def __init__(self):
        self.added = []
        self.removed = []

    def add(self, *mobjects):
        self.added.extend(mobjects)

    def remove(self, *mobjects):
        self.removed.extend(mobjects)


replacement_source = Mobject()
replacement_target = Mobject()
replacement = ReplacementTransform(replacement_source, replacement_target)
replacement.target_mobject = replacement_target
scene_probe = SceneProbe()
replacement.clean_up_from_scene(scene_probe)
assert scene_probe.removed == [replacement_source]
assert scene_probe.added == [replacement_target]

# The wheel installer must preserve the native composition specification.
# Choreo derives the mobject root at lowering. Public timing is available
# immediately from that same native interval builder, as in the Reference.
composition_children = [Animation(Mobject()), Animation(Mobject())]
for composition_class, duration in ((AnimationGroup, 1.0), (LaggedStart, 1.05), (Succession, 2.0)):
    composition = composition_class(*composition_children)
    assert composition.mobject is None
    assert composition.run_time == composition.max_end_time == duration
    assert composition.get_run_time() == duration
    assert composition.animations == composition_children
    assert str(composition) == composition_class.__name__
mapped_members = Mobject().add(Mobject(), Mobject())
mapped = LaggedStartMap(Animation, mapped_members)
assert [child.mobject for child in mapped.animations] == list(mapped_members)
for cyclic_class in (CyclicReplace, Swap):
    cyclic = cyclic_class(Mobject(), Mobject())
    target = cyclic._native_target()
    assert target is not None
    assert cyclic.target_mobject is target
    assert cyclic._target_attr == "target_mobject"


# A Transform subclass's leaf hook must survive the installed dispatch.
class TransformHookProbe(Transform):
    def update_mobjects(self, dt):
        self.seen_dts.append(float(dt))
        super().update_mobjects(dt)

    def interpolate_submobject(self, current, starting, target, alpha):
        self.seen_alphas.append(float(alpha))


hook_probe = TransformHookProbe(Mobject(), Mobject(), rate_func=linear_rate)
hook_probe.seen_alphas = []
hook_probe.seen_dts = []
hook_probe.begin()
hook_probe.interpolate(0.5)
assert hook_probe.seen_alphas == [0.0, 0.5]
hook_probe.finish()

# The same override must run through Scene.play, where native-kind lowering
# previously bypassed Python subclass methods. This hook deliberately leaves
# points alone, so endpoint-only native interpolation is also a negative.
play_hook_source = Mobject().set_points([[0.0, 0.0, 0.0]])
play_hook_target = Mobject().set_points([[1.0, 0.0, 0.0]])
play_hook_probe = TransformHookProbe(
    play_hook_source, play_hook_target, rate_func=linear_rate,
)
play_hook_probe.seen_alphas = []
play_hook_probe.seen_dts = []
Scene().play(play_hook_probe, run_time=2.0 / 30.0)
assert play_hook_probe.seen_alphas, "Scene.play bypassed the Transform subclass hook"
assert any(0.0 < alpha < 1.0 for alpha in play_hook_probe.seen_alphas)
assert play_hook_probe.seen_alphas[0] == 0.0
assert play_hook_probe.seen_alphas[-1] == 1.0
# Plan §9.2: begin interpolates at zero separately; progression samples are
# k/fps for k=1..N, each with the same rational frame delta.
assert np.allclose(play_hook_probe.seen_dts, [1.0 / 30.0, 1.0 / 30.0]), play_hook_probe.seen_dts
assert np.array_equal(play_hook_source.get_points(), [[0.0, 0.0, 0.0]])
assert not play_hook_source.locked_data_keys


class TimelineMove(Animation):
    def __init__(self, mobject, destination, events, **kwargs):
        self.destination = float(destination)
        self.events = events
        self.helper_ticks = []
        super().__init__(mobject, rate_func=linear_rate, **kwargs)

    def begin(self):
        self.events.append(("begin", self.destination, float(self.mobject.get_x())))
        super().begin()

    def update_mobjects(self, dt):
        self.helper_ticks.append(float(dt))
        super().update_mobjects(dt)

    def interpolate_mobject(self, alpha):
        start = float(self.starting_mobject.get_x())
        self.mobject.set_x(start + (self.destination - start) * alpha)

    def finish(self):
        super().finish()
        self.events.append(("finish", self.destination, float(self.mobject.get_x())))


def timeline_point():
    return Mobject().set_points([[0.0, 0.0, 0.0]])


# Changing a group's wall-clock duration preserves its member timeline.
# Compare Python and native leaves at every observed frame, including any
# final refresh, with an independent first-frame value and intermediate state.
for play_override, timeline_rate in (
    (False, None), (True, linear_rate), (False, "linear"), (True, "linear"),
):
    timeline_source, timeline_native, timeline_observer = (
        timeline_point(), timeline_point(), timeline_point()
    )
    mixed_samples = []
    timeline_observer.add_updater(
        lambda mob, dt: mixed_samples.append(
            (timeline_source.get_x(), timeline_native.get_x())
        ) if dt > 0 else None,
        call=False,
    )
    timeline_scene = Scene().add(timeline_source, timeline_native, timeline_observer)
    timeline_group = AnimationGroup(
        TimelineMove(timeline_source, 1.0, [], run_time=0.1),
        Transform(
            timeline_native, timeline_native.copy().set_x(1.0),
            run_time=0.1, rate_func=linear_rate,
        ),
        run_time=-1 if play_override else 0.2,
        rate_func=None if play_override else timeline_rate,
    )
    if play_override:
        timeline_scene.play(timeline_group, run_time=0.2, rate_func=timeline_rate)
    else:
        timeline_scene.play(timeline_group)
    assert len(mixed_samples) >= 6, mixed_samples
    assert np.allclose(mixed_samples[0], [1.0 / 6.0] * 2, atol=1e-6), mixed_samples
    assert np.allclose(
        [pair[0] for pair in mixed_samples],
        [pair[1] for pair in mixed_samples], atol=1e-6,
    ), mixed_samples
    assert any(0.0 < pair[0] < 1.0 for pair in mixed_samples)
    assert np.allclose(mixed_samples[-1], [1.0, 1.0], atol=1e-6)

# A future Succession child must begin from the previous child's final state;
# its helpers must not tick while another child is active.
timeline_source, timeline_observer = timeline_point(), timeline_point()
successive_samples, timeline_events = [], []
timeline_observer.add_updater(
    lambda mob, dt: successive_samples.append(timeline_source.get_x()) if dt > 0 else None,
    call=False,
)
timeline_first = TimelineMove(timeline_source, 1.0, timeline_events, run_time=0.1)
timeline_second = TimelineMove(timeline_source, 2.0, timeline_events, run_time=0.1)
Scene().add(timeline_source, timeline_observer).play(
    Succession(timeline_first, timeline_second, rate_func=linear_rate)
)
assert np.allclose(successive_samples[:6], [k / 3.0 for k in range(1, 7)], atol=1e-6), successive_samples
assert timeline_events == [
    ("begin", 1.0, 0.0), ("finish", 1.0, 1.0),
    ("begin", 2.0, 1.0), ("finish", 2.0, 2.0),
], timeline_events
assert len(timeline_first.helper_ticks) == 3, timeline_first.helper_ticks
assert len(timeline_second.helper_ticks) >= 3, timeline_second.helper_ticks

# Native and Python leaves share the same succession lifecycle, even when
# both mutate one object. The second begin must see the first final state.
for python_first in (True, False):
    mixed_source, mixed_observer = timeline_point(), timeline_point()
    mixed_successive_samples, mixed_successive_events = [], []
    mixed_observer.add_updater(
        lambda mob, dt: mixed_successive_samples.append(mixed_source.get_x())
        if dt > 0 else None, call=False,
    )
    mixed_members = [
        TimelineMove(mixed_source, destination, mixed_successive_events, run_time=0.1)
        if (index == 0) == python_first else Transform(
            mixed_source, mixed_source.copy().set_x(destination),
            run_time=0.1, rate_func=linear_rate,
        )
        for index, destination in enumerate((1.0, 2.0))
    ]
    Scene().add(mixed_source, mixed_observer).play(Succession(*mixed_members))
    assert np.allclose(
        mixed_successive_samples[:6], [k / 3.0 for k in range(1, 7)], atol=1e-6,
    ), (python_first, mixed_successive_samples)
    expected_start = 0.0 if python_first else 1.0
    assert mixed_successive_events[0][2] == expected_start, mixed_successive_events

# Mixed simultaneous leaves also preserve argument order on shared records.
for python_last in (True, False):
    shared_source, shared_observer = timeline_point(), timeline_point()
    shared_samples = []
    shared_observer.add_updater(
        lambda mob, dt: shared_samples.append(shared_source.get_x()) if dt > 0 else None,
        call=False,
    )
    shared_python = TimelineMove(shared_source, 2.0, [], run_time=0.1)
    shared_native = Transform(
        shared_source, shared_source.copy().set_x(1.0), run_time=0.1, rate_func=linear_rate,
    )
    shared_members = (shared_native, shared_python) if python_last else (shared_python, shared_native)
    Scene().add(shared_source, shared_observer).play(AnimationGroup(*shared_members))
    end = 2.0 if python_last else 1.0
    assert np.allclose(shared_samples[:3], [end * k / 3 for k in (1, 2, 3)], atol=1e-6), shared_samples

# A host exception releases a begun native child's locks and suspension
# without snapping the partial frame to its endpoint.
class FailingTimelineMove(TimelineMove):
    def interpolate_mobject(self, alpha):
        if alpha > 0.0:
            raise RuntimeError("mixed-leaf failure witness")
        super().interpolate_mobject(alpha)


failed_source = timeline_point()
failed_scene = Scene().add(failed_source)
try:
    failed_scene.play(AnimationGroup(
        Transform(
            failed_source, failed_source.copy().set_x(1.0), run_time=0.1,
            rate_func=linear_rate, suspend_mobject_updating=True,
        ),
        FailingTimelineMove(failed_source, 2.0, [], run_time=0.1),
    ))
except RuntimeError as error:
    assert str(error) == "mixed-leaf failure witness", error
else:
    raise AssertionError("mixed composition swallowed the host exception")
assert 0.0 < failed_source.get_x() < 1.0, failed_source.get_x()
assert not failed_source.locked_data_keys
assert not failed_source._is_updating_suspended()

# TransformFromCopy's public constructor owns exactly one source copy. The
# native play must remove it after replacement, without an orphaned family.
copy_source = Group(timeline_point(), timeline_point().shift((0.0, 1.0, 0.0)))
copy_target = copy_source.copy().shift((2.0, 0.0, 0.0))
copy_animation = TransformFromCopy(copy_source, copy_target, run_time=0.1, rate_func=linear_rate)
constructor_copy = copy_animation.mobject
assert constructor_copy is not copy_source
copy_scene = Scene().add(copy_source)
copy_scene.play(copy_animation)
assert copy_scene.mobjects == [copy_source, copy_target], copy_scene.mobjects
assert constructor_copy not in copy_scene.get_mobject_family_members()
assert copy_scene._engine_facts()[:2] == (2, 6), copy_scene._engine_facts()
assert np.allclose(copy_source.get_center(), [0.0, 0.5, 0.0])
assert np.allclose(copy_target.get_center(), [2.0, 0.5, 0.0])

# Nested coarse sampling still runs every crossed child in order (BN-11).
coarse_source, coarse_events = timeline_point(), []
Scene().play(AnimationGroup(Succession(*[
    TimelineMove(coarse_source, destination, coarse_events, run_time=0.1)
    for destination in (1.0, 2.0, 3.0)
]), run_time=1.0 / 30.0))
assert coarse_events == [
    ("begin", 1.0, 0.0), ("finish", 1.0, 1.0),
    ("begin", 2.0, 1.0), ("finish", 2.0, 2.0),
    ("begin", 3.0, 2.0), ("finish", 3.0, 3.0),
], coarse_events


def render_animation_lifecycle(destination, seed):
    """Exercise shared hooks and native composition through real PNG output."""
    class AnimationLifecycleScene(Scene):
        random_seed = seed & 0xFFFF_FFFF

        def construct(self):
            square = Square(side_length=0.5).move_to((1.0, 0.0, 0.0))
            target = square.copy().move_to((-1.0, 0.0, 0.0))
            self.add(square)
            morph = Transform(square, target, path_arc=math.pi, rate_func=linear_rate)
            morph.begin()
            morph.interpolate(0.5)
            midpoint = square.get_center()
            assert np.allclose(midpoint, [0.0, 1.0, 0.0], atol=1e-5), midpoint
            self.wait(1.0 / 30.0)
            morph.finish()
            assert np.allclose(square.get_center(), [-1.0, 0.0, 0.0], atol=1e-5)
            assert not square.locked_data_keys
            self.wait(1.0 / 30.0)

            circle = Circle(radius=0.25).move_to((1.0, 0.0, 0.0))
            self.play(
                Succession(
                    Transform(square, square.copy().shift((0.0, -1.0, 0.0)), run_time=0.1),
                    ShowCreation(circle, run_time=0.1),
                    rate_func=linear_rate,
                ),
            )
            assert np.allclose(square.get_center(), [-1.0, -1.0, 0.0], atol=1e-5)
            roots = self.mobjects
            assert len(roots) == 1, roots
            root = roots[0]
            assert root.submobjects == [square, circle]
            assert self.mobjects[0] is root
            assert root in square.parents and root in circle.parents
            assert self.get_mobject_family_members() == [root, square, circle]
            # A projected container is a live Stage identity: changing its
            # Python family must affect subsequent native captures too.
            root.remove(circle)
            assert root.submobjects == [square] and root not in circle.parents
            assert self.get_mobject_family_members() == [root, square]
            root.add(circle)
            assert self.get_mobject_family_members() == [root, square, circle]
            # The published frames include a mixed, same-object succession;
            # the native second child must capture the Python endpoint.
            mixed_positions = []
            circle.add_updater(
                lambda mob, dt: mixed_positions.append(square.get_x()) if dt > 0 else None,
                call=False,
            )
            self.play(Succession(
                TimelineMove(square, 0.0, [], run_time=0.1),
                Transform(square, square.copy().set_x(1.0), run_time=0.1, rate_func=linear_rate),
            ))
            assert np.allclose(
                mixed_positions[:6], [-1.0 + k / 3.0 for k in range(1, 7)], atol=1e-5,
            ), mixed_positions
            circle.clear_updaters()
            self.wait(1.0 / 30.0)

    scene = AnimationLifecycleScene()
    scene._begin_png_sequence(destination, 96, 54, 30, 1, seed)
    try:
        scene.run()
        return scene._finish_render(
            scene.frame._core, scene.camera.light_source.get_center(),
        )
    except Exception:
        scene._abort_render()
        raise


def _assert_authored_transform_native_paths():
    """Run against the real extension in the existing production suite."""
    duration = 2.0 / 30.0

    def excursion(start, end, alpha):
        return (
            (1.0 - alpha) * start + alpha * end
            + np.array([0.0, 12.0 * alpha * (1.0 - alpha), 0.0])
        )

    def observe(source, animation):
        samples = []
        observer = timeline_point()
        observer.add_updater(
            lambda mob, dt: samples.append(source.get_points().copy())
            if dt > 0 else None,
            call=False,
        )
        Scene().add(source, observer).play(animation)
        observer.clear_updaters()
        return np.asarray(samples)

    # Midpoint geometry, not just endpoints, proves the authored callable ran
    # through Scene.play and wrote native RecordBuffer fields before capture.
    source = timeline_point()
    target = source.copy().set_x(4.0)
    samples = observe(source, Transform(
        source, target, path_func=excursion,
        run_time=duration, rate_func=linear_rate,
    ))
    assert len(samples) >= 2, samples
    assert np.allclose(samples[:2, 0], [[2, 3, 0], [4, 0, 0]], atol=1e-6), samples
    assert np.array_equal(target.get_points(), [[4, 0, 0]])
    assert not source.locked_data_keys

    # Equal endpoint records must not suppress a closed authored excursion.
    source = timeline_point()
    closed = Transform(
        source, source.copy(), path_func=excursion,
        run_time=duration, rate_func=linear_rate,
    )
    closed.begin()
    closed.interpolate(0.5)
    assert np.allclose(source.get_points(), [[0, 3, 0]], atol=1e-6)
    assert "point" not in source.locked_data_keys
    closed.finish()
    assert np.allclose(source.get_points(), [[0, 0, 0]], atol=1e-6)

    # Each direction crosses the Python/native boundary on one live source.
    # A nested group also exercises recursive callback member discovery.
    for python_first in (True, False):
        source = timeline_point()
        members = [
            Transform(
                source, source.copy().set_x(destination),
                path_func=excursion if (index == 0) == python_first else None,
                run_time=duration, rate_func=linear_rate,
            )
            for index, destination in enumerate((2.0, 4.0))
        ]
        samples = observe(source, AnimationGroup(Succession(*members)))
        expected = [[1, 3 if python_first else 0, 0], [2, 0, 0],
                    [3, 0 if python_first else 3, 0], [4, 0, 0]]
        assert len(samples) >= 4, samples
        assert np.allclose(samples[:4, 0], expected, atol=1e-6), (python_first, samples)

    class FromLeft(Transform):
        def create_starting_mobject(self):
            return self.mobject.copy().shift((-2.0, 0.0, 0.0))

    source = timeline_point()
    samples = observe(source, FromLeft(
        source, source.copy().set_x(4.0),
        run_time=duration, rate_func=linear_rate,
    ))
    assert len(samples) >= 2, samples
    assert np.allclose(samples[:2, 0], [[1, 0, 0], [4, 0, 0]], atol=1e-6), samples

    class InitializedPath(Transform):
        def init_path_func(self):
            self.path_func = excursion

    source = timeline_point()
    samples = observe(source, InitializedPath(
        source, source.copy(), run_time=duration, rate_func=linear_rate,
    ))
    assert len(samples) >= 2, samples
    assert np.allclose(samples[:2, 0], [[0, 3, 0], [0, 0, 0]], atol=1e-6), samples

    class QuarterProgress(Transform):
        def get_sub_alpha(self, alpha, index, num_submobjects):
            return 0.25

    source = timeline_point()
    samples = observe(source, QuarterProgress(
        source, source.copy().set_x(4.0),
        run_time=duration, rate_func=linear_rate,
    ))
    assert len(samples) >= 2, samples
    assert np.allclose(samples[:, 0], [1, 0, 0], atol=1e-6), samples
    assert np.allclose(source.get_points(), [[1, 0, 0]], atol=1e-6)

    # A post-construction assignment must not use a stale native-route flag.
    source = timeline_point()
    late_path = Transform(source, source.copy().set_x(4.0), run_time=duration)
    late_path.path_func = excursion
    late_path.rate_func = linear_rate
    samples = observe(source, late_path)
    assert len(samples) >= 2, samples
    assert np.allclose(samples[:2, 0], [[2, 3, 0], [4, 0, 0]], atol=1e-6), samples

    # Replacement cleanup uses original object identities, with exactly the
    # constructor-owned source copy and no leaked temporary root.
    source = timeline_point()
    target = source.copy().set_x(4.0)
    copied = TransformFromCopy(
        source, target, path_func=excursion,
        run_time=duration, rate_func=linear_rate,
    )
    temporary = copied.mobject
    scene = Scene().add(source)
    scene.play(copied)
    assert scene.mobjects == [source, target], scene.mobjects
    assert temporary not in scene.get_mobject_family_members()
    assert np.array_equal(source.get_points(), [[0, 0, 0]])
    assert np.array_equal(target.get_points(), [[4, 0, 0]])

    def broken_path(start, end, alpha):
        if alpha > 0.0:
            raise RuntimeError("authored-path failure witness")
        return start

    source = timeline_point()
    scene = Scene().add(source)
    try:
        scene.play(Transform(
            source, source.copy().set_x(4.0), path_func=broken_path,
            run_time=duration, rate_func=linear_rate,
            suspend_mobject_updating=True,
        ))
    except RuntimeError as error:
        assert str(error) == "authored-path failure witness", error
    else:
        raise AssertionError("Scene.play swallowed an authored path exception")
    assert not source.locked_data_keys
    assert not source._is_updating_suspended()
    assert np.array_equal(source.get_points(), [[0, 0, 0]])


_assert_authored_transform_native_paths()


# Shared pre-begin timing, exercised against the installed native engine. These
# are not a modeled frame loop: Scene.play and render_session own every sample.
import unittest as _timing_unittest


class TimingContract(_timing_unittest.TestCase):
    def test_explicit_zero_and_none_are_distinct(self):
        import manimlib as m
        animation = m.Animation(m.Mobject(), run_time=3, lag_ratio=.5)
        rate = animation.rate_func
        self.assertIs(animation.update_rate_info(0, None, 0), animation)
        self.assertEqual((animation.run_time, animation.lag_ratio), (0, 0))
        self.assertIs(animation.rate_func, rate)
        animation.update_rate_info(None, None, None)
        self.assertEqual((animation.run_time, animation.lag_ratio), (0, 0))

    def test_rate_override_never_inspects_authored_truthiness(self):
        import manimlib as m
        class Rate:
            __hash__ = None
            def __bool__(self):
                raise AssertionError("a rate is not an optional boolean")
            def __eq__(self, other):
                raise AssertionError("rate admission must use identity")
            def __call__(self, alpha):
                return alpha * alpha
        source = m.VectorizedPoint()
        scene = m.Scene().add(source)
        rate = Rate()
        animation = m.Transform(source, source.copy().shift(m.RIGHT))
        animation.update_rate_info(rate_func=rate)
        self.assertIs(animation.rate_func, rate)
        scene.play(animation, run_time=.1, rate_func=rate)
        self.assertTrue(np.allclose(source.get_center(), m.RIGHT))

    def test_native_default_duration_is_queryable_without_consuming_sentinel(self):
        import manimlib as m
        source = m.Square()
        animations = [m.Transform(source, source.copy()), m.ShowCreation(source),
                      m.MoveAlongPath(source, m.Line())]
        for animation in animations:
            with self.subTest(kind=type(animation).__name__):
                self.assertIsNone(animation.run_time)
                self.assertEqual(animation.get_run_time(), 1)
                self.assertIsNone(animation.run_time)
                self.assertFalse(hasattr(animation, 'starting_mobject'))

    def test_time_windows_widen_default_and_explicit_durations(self):
        import manimlib as m
        source = m.Square()
        for run_time, span, expected in [
            (None, (.25, .5), 1), (None, (.25, 2), 2),
            (0, (.25, 2), 2), (3, (.25, 2), 3),
        ]:
            with self.subTest(run_time=run_time, span=span):
                animation = m.Transform(source, source.copy(), run_time=run_time, time_span=span)
                self.assertEqual(animation.get_run_time(), expected)
                self.assertEqual(animation.run_time, run_time)
                group = m.AnimationGroup(animation)
                self.assertEqual(group.get_run_time(), expected)
                self.assertEqual(group.anims_with_timings[0][1:], (0, expected))

    def test_query_does_not_invoke_authored_setup_or_target_factory(self):
        import manimlib as m
        calls = []
        class Deferred(m.ApplyFunction):
            def _ensure_runtime_defaults(self):
                raise AssertionError('a duration query is not begin')
            def create_starting_mobject(self):
                raise AssertionError('a duration query must not copy')
        source = m.Square()
        animation = Deferred(lambda target: calls.append(target) or target, source,
                             time_span=(.25, 2))
        self.assertEqual(animation.get_run_time(), 2)
        self.assertEqual(m.Succession(animation).get_run_time(), 2)
        self.assertEqual(calls, [])
        self.assertIsNone(animation.run_time)
        self.assertIsNone(animation.target_mobject)

    def test_specialized_constructor_durations_are_preserved(self):
        import manimlib as m
        source = m.Square()
        for animation, expected in [
            (m.Rotating(source), 5),
            (m.Homotopy(lambda x, y, z, t: [x, y, z], source), 3),
            (m.DrawBorderThenFill(source), 2),
            (m.Write(source), 1),
        ]:
            with self.subTest(kind=type(animation).__name__):
                self.assertEqual(animation.get_run_time(), expected)
                self.assertEqual(m.AnimationGroup(animation).get_run_time(), expected)

    def test_nested_time_window_playback_uses_the_real_clock(self):
        import manimlib as m
        from fmn_python import render_session
        from pathlib import Path
        import tempfile
        for callback in (False, True):
            with self.subTest(callback=callback):
                scene, source = m.Scene(), m.VectorizedPoint()
                samples = []
                def observe(member, dt):
                    if member is source and dt > 0:
                        samples.append((float(scene.time), member.get_x()))
                source.add_updater(observe, call=False)
                options = {'path_func': lambda a, b, t: (1-t)*a+t*b} if callback else {}
                first = m.Transform(source, source.copy().shift(m.RIGHT),
                                    time_span=(.25, 1.25), rate_func=m.linear, **options)
                last = m.Transform(source, source.copy().shift(2*m.RIGHT),
                                   run_time=.25, rate_func=m.linear, **options)
                inner = m.AnimationGroup(first)
                outer = m.Succession(inner, last)
                self.assertEqual(outer.get_run_time(), 1.5)
                with tempfile.TemporaryDirectory() as directory:
                    with render_session(scene, Path(directory) / 'timed.y4m',
                                        resolution=(16, 8), fps=8, threads=1) as output:
                        scene.add(source)
                        scene.play(outer)
                    self.assertEqual(output.result.frame_count, 12)
                times = np.arange(1, 13) / 8
                values = [max(0, t-.25) if t <= 1.25 else 1+4*(t-1.25) for t in times]
                self.assertTrue(np.allclose(samples, np.column_stack((times, values)), atol=1e-6), samples)
                self.assertEqual(scene.time, 1.5)
                self.assertTrue(np.allclose(source.get_center(), 2*m.RIGHT))

    def test_zero_duration_play_finishes_without_advancing_native_clock(self):
        import manimlib as m
        for callback in (False, True):
            with self.subTest(callback=callback):
                source = m.Square()
                scene = m.Scene().add(source)
                ticks = []
                source.add_updater(lambda member, dt: ticks.append(dt), call=False)
                options = {'path_func': lambda a, b, t: (1-t)*a+t*b} if callback else {}
                animation = m.Transform(source, source.copy().shift(m.RIGHT),
                                        run_time=3, lag_ratio=.5, rate_func=m.linear, **options)
                scene.play(animation, run_time=0, lag_ratio=0)
                self.assertEqual(scene.time, 0)
                self.assertEqual(scene.num_plays, 1)
                self.assertEqual((animation.run_time, animation.lag_ratio), (0, 0))
                self.assertTrue(all(dt == 0 for dt in ticks), ticks)
                self.assertTrue(np.allclose(source.get_center(), m.RIGHT))
                self.assertFalse(source._is_updating_suspended())

    def test_zero_lag_update_reaches_native_and_callback_family_interpolation(self):
        import manimlib as m
        for callback in (False, True):
            with self.subTest(callback=callback):
                left, right = m.VectorizedPoint(m.LEFT), m.VectorizedPoint(m.RIGHT)
                source, samples = m.Group(left, right), []
                scene = m.Scene().add(source)
                def observe(member, dt):
                    if member is source and dt > 0:
                        samples.append((scene.time, left.get_x()+1, right.get_x()-1))
                source.add_updater(observe, call=False)
                options = {'path_func': lambda a, b, t: (1-t)*a+t*b} if callback else {}
                animation = m.Transform(source, source.copy().shift(2*m.RIGHT),
                                        run_time=.2, lag_ratio=.75, rate_func=m.linear, **options)
                animation.update_rate_info(lag_ratio=0)
                scene.play(animation)
                self.assertTrue(samples)
                for time, first, second in samples:
                    expected = 2*min(time/.2, 1)
                    self.assertAlmostEqual(first, expected, places=6)
                    self.assertAlmostEqual(second, expected, places=6)


def _check_interpolate_record_view_reads():
    # fm-5wq.31: mobject_interpolate builds each `.data` view once per call
    # under the shipped straight_path, and rebuilds them after an authored
    # path_func, which may replace the records it was handed.
    data_property = Mobject.__dict__["data"]
    reads = []
    Mobject.data = property(lambda self: (reads.append(self), data_property.fget(self))[1])
    try:
        square, start, end = Square(), Square(), Square().shift(np.array([1., 0., 0.]))
        square.interpolate(start, end, .25)  # first-call work is not the steady state
        counts = {}
        for label, path_func in (("default", None), ("authored", lambda a, b, t: (1 - t) * a + t * b)):
            reads.clear()
            square.interpolate(start, end, .5, path_func=path_func)
            counts[label] = len(reads)
    finally:
        Mobject.data = data_property
    pointlike = len(square.pointlike_data_keys)
    assert counts == {"default": 3, "authored": 3 + 3 * pointlike}, counts
    np.testing.assert_allclose(square.get_points(), (start.get_points() + end.get_points()) / 2, atol=1e-6)


_check_interpolate_record_view_reads()


_timing_result = _timing_unittest.TextTestRunner(verbosity=2).run(
    _timing_unittest.defaultTestLoader.loadTestsFromTestCase(TimingContract)
)
if not _timing_result.wasSuccessful():
    raise AssertionError('native animation timing contract failed')
