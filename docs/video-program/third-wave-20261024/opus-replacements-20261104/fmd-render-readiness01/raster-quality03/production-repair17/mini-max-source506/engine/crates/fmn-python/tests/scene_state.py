"""Installed-wheel scene snapshots through native records, cameras and output.

No storage doubles are used here. Run via scripts/check_portal_runtime.sh in
an environment containing a wheel built from the same checkout.
"""
from __future__ import annotations

import copy
from pathlib import Path
import tempfile

import numpy as np
import manimlib as m
from fmn_python import render_scene
from manimlib.scene.scene import SceneState as QualifiedState


def close(actual, expected):
    np.testing.assert_allclose(actual, expected, atol=2e-5, rtol=1e-6)


def captured_families_styles_and_arrays():
    assert QualifiedState is m.SceneState
    scene = m.Scene()
    child = m.Square(fill_opacity=1, fill_color=m.WHITE)
    group = m.Group(child)
    group.uniforms["scene_gain"] = np.array([1., 2.])
    scene.add(group)
    state = scene.get_state()
    scene.undo_stack.append(state)
    mirror = state.mobjects_to_copies[group]
    saved_points = mirror[0].get_points().copy()
    assert len(group.data) == 0
    child.shift(m.RIGHT).set_fill(m.RED, opacity=.5)
    group.uniforms["scene_gain"][:] = 9
    changed = scene.get_state()
    assert not state.mobjects_match(changed)
    assert state.n_changes(changed) == 1
    assert changed.mobjects_to_copies[group] is not mirror
    np.testing.assert_array_equal(mirror[0].get_points(), saved_points)
    np.testing.assert_array_equal(mirror.uniforms["scene_gain"], [1., 2.])
    # Comparing two captures must not depend on later changes to their sources.
    before = state.n_changes(changed)
    child.shift(100 * m.UP)
    assert state.n_changes(changed) == before


def shared_child_identity():
    scene, shared = m.Scene(), m.Square()
    left, right = m.Group(shared), m.Group(shared)
    root = m.Group(left, right)
    scene.add(root)
    state = scene.get_state()
    replacement = m.Circle()
    left.set_submobjects([replacement])
    right.set_submobjects([])
    shared.shift(3 * m.RIGHT)
    state.restore_scene(scene)
    assert root[0] is left and root[1] is right
    assert left[0] is shared and right[0] is shared
    assert left in shared.parents and right in shared.parents
    assert left not in replacement.parents
    close(shared.get_center(), m.ORIGIN)


def updater_restoration():
    scene, square, events = m.Scene(), m.Square(), []
    def original(obj, dt):
        events.append((obj, dt))
    square.add_updater(original, call=False)
    square.uniforms["state_array"] = np.array([2., 4.])
    scene.add(square)
    state = scene.get_state()
    square.updaters[:] = [lambda obj: events.append("wrong updater")]
    square.uniforms["state_array"][:] = 99
    state.restore_scene(scene)
    assert len(square.updaters) == 1 and square.updaters[0] is original
    scene.update(0.)
    assert events and all(isinstance(event, tuple) and event[0] is square for event in events)
    np.testing.assert_array_equal(square.uniforms["state_array"], [2., 4.])
    square.uniforms["state_array"][:] = 7
    state.restore_scene(scene)
    np.testing.assert_array_equal(square.uniforms["state_array"], [2., 4.])


def ordered_roots():
    scene = m.Scene()
    left, right = m.Square(), m.Circle()
    scene.add(left, right)
    state = scene.get_state()
    scene.clear()
    scene.add(right, left)
    assert not state.mobjects_match(scene.get_state())
    state.restore_scene(scene)
    assert len(scene.mobjects) == 2
    assert scene.mobjects[0] is left and scene.mobjects[1] is right


def scene_mobjects_follow_every_structural_change():
    # fm-5wq.31: Scene.mobjects reuses its projection of the native roots
    # while the Stage topology and the proxy registry are unchanged. Every
    # read below follows a change the projection must see.
    import gc

    def roots(scene):
        return [id(mob) for mob in scene.mobjects]

    scene = m.Scene()
    a, b, c = m.Square(), m.Circle(), m.Dot()
    group = m.Group(a)
    scene.add(group, b)
    assert roots(scene) == [id(group), id(b)] == roots(scene)
    assert all(x is y for x, y in zip(scene.mobjects, scene.mobjects))
    scene.remove(b)
    assert roots(scene) == [id(group)]
    scene.add(c)
    assert roots(scene) == [id(group), id(c)]
    scene.bring_to_back(c)
    assert roots(scene) == [id(c), id(group)]
    group.add(b)
    assert roots(scene) == [id(c), id(group)]
    assert [id(x) for x in group.submobjects] == [id(a), id(b)]
    group.set_submobjects([b, a])
    assert [id(x) for x in scene.mobjects[1].submobjects] == [id(b), id(a)]
    group.remove(b)
    assert [id(x) for x in scene.mobjects[1].submobjects] == [id(a)]
    # Removing a rooted member ungroups its ancestor in place.
    scene.remove(a)
    assert roots(scene) == [id(c)]
    # A root whose proxy may die with its last Python reference: the next
    # read must still project it, then reuse whatever proxy it projected.
    scene.add(m.Square())
    gc.collect()
    shells = scene.mobjects
    assert len(shells) == 2 and shells[0] is c and isinstance(shells[1], m.Mobject)
    kept = shells[1]
    del shells
    gc.collect()
    assert scene.mobjects[1] is kept


def native_only_edits_reach_python_through_scene_mobjects():
    # fm-5wq.31: after an edge change, Scene.mobjects re-verifies only the
    # lists whose native child epoch moved. A native-only edit (as Choreo
    # makes, here through the binding's own commit primitive, so the live
    # list is not told) must still reach the Python list and back-edges.
    scene = m.Scene()
    a, b, c = m.Square(), m.Circle(), m.Dot()
    group = m.VGroup(a, b)
    scene.add(group, c)
    scene.mobjects
    group._replace_submobjects([b, c])
    assert [id(x) for x in group.submobjects] == [id(a), id(b)]  # not yet told
    scene.mobjects
    assert [id(x) for x in group.submobjects] == [id(b), id(c)]
    assert group not in a.parents and group in c.parents


def projection_rereads_only_moved_child_lists():
    # fm-5wq.31 work counter: after an edge change among already-projected
    # members, Scene.mobjects reads the moved list only. The previous
    # projection read every member's `submobjects` on every such read.
    reads = []

    class Counted(m.VGroup):
        @property
        def submobjects(self):
            reads.append(self)
            return self.__dict__["submobjects"]

        @submobjects.setter
        def submobjects(self, value):
            self.__dict__["submobjects"] = value

    leaves = [Counted() for _ in range(40)]
    root = Counted(*leaves)
    scene = m.Scene()
    scene.add(root)
    scene.mobjects
    leaves[3].add(leaves[4])  # a bound member gains a second parent
    reads.clear()
    scene.mobjects
    assert reads == [leaves[3]], len(reads)


def camera_pose_and_callbacks():
    scene = m.Scene()
    frame, core, events = scene.frame, scene.frame._core, []
    core.set_orientation((0., 0., 1., 0.))
    core.make_orientation_default()
    core.set_orientation((0., 1., 0., 0.))
    core.set_euler_axes("zxy")
    core.set_center((2., -1., 3.))
    core.set_shape((12., 7.))
    core.set_field_of_view(.6)
    def rotate(obj, dt):
        events.append(dt)
    frame.add_updater(rotate, call=False)
    expected = (tuple(core.center()), core.shape(), tuple(core.orientation()), core.field_of_view())
    state = scene.get_state()
    frame.shift(3 * m.RIGHT).scale(.5).set_theta(.7)
    core.set_euler_axes("zxz")
    core.set_field_of_view(.9)
    core.make_orientation_default()
    frame.updaters.clear()
    assert not state.mobjects_match(scene.get_state())
    state.restore_scene(scene)
    assert scene.frame is frame and frame._core is core
    for actual, saved in zip((core.center(), core.shape(), core.orientation(), core.field_of_view()), expected):
        close(actual, saved)
    assert core.euler_axes() == "zxy"
    restored_default = copy.copy(core)
    restored_default.to_default_state()
    close(restored_default.orientation(), (0., 0., 1., 0.))
    assert len(frame.updaters) == 1 and frame.updaters[0] is rotate
    assert all(obj is not frame for obj in scene.mobjects)


def native_clock_and_bytes():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.wait(.125)
    scene.num_plays = 7
    state = scene.get_state()
    assert state._checkpoint
    before = scene.get_time()
    scene.wait(.125)
    scene.num_plays = 99
    square.shift(3 * m.RIGHT)
    state.restore_scene(scene)
    assert scene.get_time() == before and scene.num_plays == 7
    close(square.get_center(), m.ORIGIN)
    assert bytes(scene._checkpoint_bytes()) == state._checkpoint


def repeated_history():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    frame, core = scene.frame, scene.frame._core
    for x in (0., 1., 2.):
        square.set_x(x)
        frame.set_x(x / 2)
        scene.save_state()
    square.set_x(3.)
    frame.set_x(1.5)
    for x in (2., 1., 0.):
        scene.undo()
        close(square.get_x(), x)
        close(frame.get_x(), x / 2)
    for x in (1., 2., 3.):
        scene.redo()
        close(square.get_x(), x)
        close(frame.get_x(), x / 2)
    assert scene.frame is frame and frame._core is core


def history_branch_and_limit():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    undo, redo = scene.undo_stack, scene.redo_stack
    scene.save_state()
    scene.save_state()
    assert len(undo) == 1
    square.shift(m.RIGHT)
    scene.undo()
    assert len(redo) == 1
    square.shift(2 * m.UP)
    scene.save_state()
    assert scene.redo_stack is redo and not redo
    scene.max_num_saved_states = 2
    for _ in range(4):
        square.shift(m.RIGHT)
        scene.save_state()
    assert scene.undo_stack is undo and len(undo) == 2
    scene.max_num_saved_states = 0
    scene.save_state()
    assert not undo and not redo


def rejected_checkpoint_preserves_history():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.save_state()
    state = scene.undo_stack[-1]
    state._checkpoint = b"deliberately-invalid-checkpoint"
    square.shift(m.RIGHT)
    undo, redo = tuple(scene.undo_stack), tuple(scene.redo_stack)
    try:
        scene.undo()
    except Exception:
        pass
    else:
        raise AssertionError("native checkpoint decoder must reject malformed bytes")
    assert len(scene.undo_stack) == len(undo)
    assert all(a is b for a, b in zip(scene.undo_stack, undo))
    assert tuple(scene.redo_stack) == redo
    close(square.get_center(), m.RIGHT)


def foreign_owner_refusal():
    source, target = m.Scene(), m.Scene()
    a, b = m.Square(), m.Square().shift(3 * m.RIGHT)
    source.add(a)
    target.add(b)
    state = source.get_state()
    try:
        state.restore_scene(target)
    except m._ForeignStageError as error:
        assert "another Scene" in str(error)
    else:
        raise AssertionError("a checkpoint cannot be restored into another arena")
    close(a.get_center(), m.ORIGIN)
    close(b.get_center(), 3 * m.RIGHT)
    assert source.mobjects[0] is a and target.mobjects[0] is b


def ignored_camera():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    state = m.SceneState(scene, ignore=[scene.frame])
    assert state._checkpoint is not None
    scene.frame.shift(2 * m.RIGHT)
    scene.wait(.125)
    square.shift(m.UP)
    state.restore_scene(scene)
    close(scene.frame.get_center(), 2 * m.RIGHT)
    close(square.get_center(), m.ORIGIN)
    assert scene.get_time() == state.time


def rendered_restore():
    class Original(m.Scene):
        default_camera_config = dict(resolution=(96, 54), fps=8)
        def construct(self):
            self.subject = m.Square(side_length=1, fill_color=m.WHITE,
                                    fill_opacity=1, stroke_width=0)
            self.subject.shift(2 * m.LEFT + m.UP)
            self.add(self.subject)

    class Restored(Original):
        def construct(self):
            super().construct()
            self.save_state()
            self.subject.shift(4 * m.RIGHT).set_fill(m.RED)
            self.frame.shift(m.RIGHT).scale(.5)
            self.undo()

    directory = Path(tempfile.mkdtemp(prefix="fmn-scene-state-"))
    original = render_scene(Original, directory / "original.y4m", threads=1)
    restored = render_scene(Restored, directory / "restored.y4m", threads=4)
    assert original.frame_count == restored.frame_count == 1
    expected = original.destination.read_bytes()
    actual = restored.destination.read_bytes()
    assert actual == expected, "restored geometry/style/camera must render the original view"
    header, payload = actual.split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F8:1 Ip A1:1 C420mpeg2"
    assert payload[:6] == b"FRAME\n" and len(payload) == 6 + 96 * 54 * 3 // 2
    luma = np.frombuffer(payload[6:6 + 96 * 54], dtype=np.uint8).reshape(54, 96)
    ys, xs = np.nonzero(luma > 200)
    assert len(xs) > 10 and xs.mean() < 48 and ys.mean() < 27
    assert original.certified is False and restored.certified is False
    print("scene snapshot rendering artifacts:", directory)


def numeric_history_restores_values_and_rendered_glyphs():
    scene = m.Scene()
    scene.camera.reset_pixel_shape(160, 90)
    number = m.DecimalNumber(1.25)
    scene.add(number)
    def pixels():
        return bytes(scene.camera.capture_snapshot(*scene.mobjects).pixels())
    original = pixels()
    scene.save_state()
    number.set_value(99)
    edited = pixels()
    assert original != edited
    for _ in range(2):
        scene.undo()
        close(number.get_value(), 1.25)
        assert number.num_string == "1.25" and pixels() == original
        scene.redo()
        close(number.get_value(), 99)
        assert number.num_string == "99.00" and pixels() == edited
    scene.undo()
    number.increment_value(.75)
    close(number.get_value(), 2.)
    assert number.num_string == "2.00" and pixels() != original


def live_tex_source_maps_remain_editable_after_undo():
    scene, formula = m.Scene(), m.Tex("x=2")
    scene.add(formula)
    spans = copy.deepcopy(formula._string_sub_spans)
    paths = copy.deepcopy(formula._string_sub_paths)
    children = tuple(formula.submobjects)
    scene.save_state()
    number = formula.make_number_changeable(2)
    number.set_value(19)
    assert formula.string == r"x=\decimalmob"
    scene.undo()
    assert formula.string == formula.tex_string == "x=2"
    assert formula.tex_strings == ["x=2"]
    assert formula._string_sub_spans == spans and formula._string_sub_paths == paths
    assert all(a is b for a, b in zip(formula.submobjects, children))
    # The source selector must still target the restored glyph family, not
    # dangling paths from the newer decimal splice.
    replacement = formula.make_number_changeable(2)
    assert isinstance(replacement, m.DecimalNumber)
    replacement.set_value(17)
    close(replacement.get_value(), 17)
    assert formula.string == r"x=\decimalmob"


def matrix_entry_tables_and_live_values_restore_together():
    scene, matrix = m.Scene(), m.DecimalMatrix([[1., 2.], [3., 4.]])
    scene.add(matrix)
    entries = tuple(matrix.elements)
    matrix.array_aliases = np.empty((2, 2), dtype=object)
    for index, entry in enumerate(entries):
        matrix.array_aliases.flat[index] = entry
    state = scene.get_state()
    matrix.float_matrix[0][0] = 99
    entries[0].set_value(99)
    matrix.mob_matrix[0][0] = entries[-1]
    matrix.array_aliases[0, 0] = entries[-1]
    matrix.elements.reverse()
    matrix.ellipses.append(entries[-1])
    state.restore_scene(scene)
    assert matrix.float_matrix == [[1., 2.], [3., 4.]]
    assert not matrix.ellipses
    assert all(a is b for a, b in zip(matrix.elements, entries))
    assert matrix.mob_matrix[0][0] is entries[0]
    assert matrix.array_aliases[0, 0] is entries[0]
    close(entries[0].get_value(), 1.)
    # The restored indexes drive the next actual authoring operation.
    matrix.swap_entries_for_ellipses(row_index=0)
    assert matrix.ellipses and len(matrix.elements) < len(entries)


def shared_owned_data_cycles_and_camera_attributes_roundtrip():
    scene, left, right = m.Scene(), m.Square(), m.Circle()
    scene.add(left, right)
    shared = {"values": [1, 2]}
    shared["self"] = shared
    left.model = right.model = scene.frame.model = shared
    left.partner, right.partner = right, left
    left.lookup = np.empty(2, dtype=object)
    left.lookup[0], left.lookup[1] = right, shared
    state = scene.get_state()
    shared["values"][:] = [99]
    left.partner = left
    right.new_attribute = "after snapshot"
    scene.frame.model = None
    for _ in range(2):
        state.restore_scene(scene)
        assert left.model is right.model is scene.frame.model
        assert left.model["self"] is left.model
        assert left.model["values"] == [1, 2]
        assert left.partner is right and right.partner is left
        assert left.lookup[0] is right and left.lookup[1] is left.model
        assert not hasattr(right, "new_attribute")
        left.model["values"].append(3)


def attribute_only_edits_are_real_history_entries():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.payload = {"value": 1}
    scene.save_state()
    scene.save_state()
    assert len(scene.undo_stack) == 1
    first = scene.undo_stack[-1]
    square.payload["value"] = 2
    scene.save_state()
    assert len(scene.undo_stack) == 2
    second = scene.undo_stack[-1]
    assert first != second and first.n_changes(second) == 1
    # Captures remain immutable despite later edits to the live source.
    square.payload["value"] = 3
    assert first.n_changes(second) == 1
    scene.undo(); assert square.payload["value"] == 2
    scene.undo(); assert square.payload["value"] == 1
    scene.redo(); assert square.payload["value"] == 2
    scene.redo(); assert square.payload["value"] == 3


def undo_restores_inputs_seen_by_the_next_scene_updater():
    scene, square, observed = m.Scene(), m.Square(), []
    square.motion = {"speed": 1.}
    def update(obj, dt):
        observed.append(obj.motion["speed"])
        obj.shift(obj.motion["speed"] * dt * m.RIGHT)
    square.add_updater(update, call=False)
    scene.add(square)
    state = scene.get_state()
    square.motion["speed"] = 20.
    scene.update(.25)
    close(square.get_x(), 5.)
    state.restore_scene(scene)
    assert square.updaters[0] is update
    scene.update(.25)
    close(square.get_x(), .25)
    assert observed[-1] == 1.
    # The Python closure itself is deliberately not rolled back.
    assert 20. in observed


def malformed_python_projection_refuses_before_geometry_restore():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.value = [1]
    scene.save_state()
    state = scene.undo_stack[-1]
    square.shift(m.RIGHT)
    square.value[:] = [2]
    del state._attributes[id(square)]
    undo, redo = tuple(scene.undo_stack), tuple(scene.redo_stack)
    try:
        scene.undo()
    except ValueError as error:
        assert "captured family" in str(error)
    else:
        raise AssertionError("incomplete Python projection must fail before native mutation")
    close(square.get_center(), m.RIGHT)
    assert square.value == [2]
    assert all(a is b for a, b in zip(undo, scene.undo_stack))
    assert tuple(scene.redo_stack) == redo


def capture_refusal_preserves_history_and_native_arena():
    from unittest.mock import patch
    from fmn_python import scene_attributes
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    square.payload = np.arange(4)
    scene.save_state()
    square.shift(m.RIGHT)
    before = bytes(scene._checkpoint_bytes())
    undo, redo = tuple(scene.undo_stack), tuple(scene.redo_stack)
    with patch.object(scene_attributes, "_MAX_BYTES", 1):
        try:
            scene.save_state()
        except ValueError as error:
            assert "budget" in str(error)
        else:
            raise AssertionError("oversized owned state must be rejected before copying native objects")
    assert bytes(scene._checkpoint_bytes()) == before
    assert all(a is b for a, b in zip(undo, scene.undo_stack))
    assert tuple(scene.redo_stack) == redo


def ignored_camera_keeps_its_authored_attributes():
    scene, square = m.Scene(), m.Square()
    scene.add(square)
    scene.frame.label = [1]
    square.label = [1]
    state = m.SceneState(scene, ignore=[scene.frame])
    scene.frame.label[:] = [2]
    square.label[:] = [2]
    state.restore_scene(scene)
    assert scene.frame.label == [2] and square.label == [1]


CASES = (
    captured_families_styles_and_arrays, shared_child_identity, updater_restoration,
    ordered_roots, scene_mobjects_follow_every_structural_change,
    native_only_edits_reach_python_through_scene_mobjects,
    projection_rereads_only_moved_child_lists,
    camera_pose_and_callbacks, native_clock_and_bytes,
    repeated_history, history_branch_and_limit, rejected_checkpoint_preserves_history,
    foreign_owner_refusal, ignored_camera, rendered_restore,
    numeric_history_restores_values_and_rendered_glyphs,
    live_tex_source_maps_remain_editable_after_undo,
    matrix_entry_tables_and_live_values_restore_together,
    shared_owned_data_cycles_and_camera_attributes_roundtrip,
    attribute_only_edits_are_real_history_entries,
    undo_restores_inputs_seen_by_the_next_scene_updater,
    malformed_python_projection_refuses_before_geometry_restore,
    capture_refusal_preserves_history_and_native_arena,
    ignored_camera_keeps_its_authored_attributes,
)
assert len(CASES) == 24
for case in CASES:
    case()
    print("scene snapshot acceptance:", case.__name__)
print(f"scene snapshot acceptance: {len(CASES)} cases passed")
