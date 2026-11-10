"""Scene snapshots over Proscenium checkpoints and live Python projections.

Native checkpoints own records, roots, time and RNG. This module retains the
Python identities and callbacks those bytes cannot encode, and the separate
camera pose. It does not serialize executable code or rewind closure state.
"""
from __future__ import annotations

import copy
from functools import wraps
import operator
from typing import Any

from fmn_python import scene_attributes


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _freeze_value(np, value, memo=None):
    """Freeze mutable uniform containers; opaque host objects stay by identity."""
    memo = {} if memo is None else memo
    if id(value) in memo:
        return memo[id(value)]
    if isinstance(value, np.ndarray):
        result = value.copy()
    elif type(value) is dict:
        result = {}
        memo[id(value)] = result
        result.update((key, _freeze_value(np, item, memo)) for key, item in value.items())
    elif type(value) is list:
        result = []
        memo[id(value)] = result
        result.extend(_freeze_value(np, item, memo) for item in value)
    elif type(value) is tuple:
        items = tuple(_freeze_value(np, item, memo) for item in value)
        result = memo.get(id(value), items)
    elif type(value) is set:
        result = value.copy()
    else:
        return value
    memo[id(value)] = result
    return result


def _same_value(np, left, right, seen=None):
    if left is right:
        return True
    if isinstance(left, np.ndarray) or isinstance(right, np.ndarray):
        return (isinstance(left, np.ndarray) and isinstance(right, np.ndarray)
                and left.dtype == right.dtype and np.array_equal(left, right))
    if type(left) is not type(right):
        return False
    seen = set() if seen is None else seen
    pair = (id(left), id(right))
    if pair in seen:
        return True
    seen.add(pair)
    if isinstance(left, np.generic):
        return left.dtype == right.dtype and bool(np.array_equal(left, right))
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(
            _same_value(np, left[key], right[key], seen) for key in left
        )
    if isinstance(left, (tuple, list)):
        return len(left) == len(right) and all(
            _same_value(np, a, b, seen) for a, b in zip(left, right)
        )
    if isinstance(left, (str, bytes, bool, int, float, complex, set, frozenset)) or left is None:
        return bool(left == right)
    # Arbitrary Python objects in extension uniforms are identity-bearing;
    # never call their possibly stateful or array-valued equality operators.
    return False


def _same_mobject(np, left, right, resources_equal=None):
    """Compare complete ordered families, including point-free roots/aliasing."""
    pending, forward, backward = [(left, right)], {}, {}
    while pending:
        a, b = pending.pop()
        if id(a) in forward:
            if forward[id(a)] != id(b):
                return False
            continue
        if id(b) in backward or type(a) is not type(b):
            return False
        forward[id(a)], backward[id(b)] = id(b), id(a)
        if a.data.dtype != b.data.dtype or not np.array_equal(a.data, b.data):
            return False
        if resources_equal is not None and not resources_equal(a, b):
            return False
        if not _same_value(np, dict(a.uniforms), dict(b.uniforms)):
            return False
        if (len(a.updaters) != len(b.updaters)
                or any(x is not y for x, y in zip(a.updaters, b.updaters))):
            return False
        for name in ("locked_data_keys", "const_data_keys", "locked_uniform_keys"):
            if getattr(a, name, set()) != getattr(b, name, set()):
                return False
        children_a, children_b = tuple(a.submobjects), tuple(b.submobjects)
        if len(children_a) != len(children_b):
            return False
        pending.extend(zip(children_a, children_b))
    return True


def _family(roots):
    """Keep every original proxy alive, including removed/shared descendants."""
    seen, visiting, result = set(), set(), []
    stack = [(root, False) for root in reversed(roots)]
    while stack:
        member, leaving = stack.pop()
        marker = id(member)
        if leaving:
            visiting.remove(marker)
            continue
        if marker in visiting:
            raise ValueError("SceneState cannot capture a cyclic Mobject family")
        if marker in seen:
            continue
        seen.add(marker)
        visiting.add(marker)
        result.append(member)
        stack.append((member, True))
        stack.extend((child, False) for child in reversed(tuple(member.submobjects)))
    return result


def _camera_values(core):
    default = copy.copy(core)
    default.to_default_state()
    return (tuple(core.center()), tuple(core.shape()), tuple(core.orientation()),
            core.field_of_view(), core.euler_axes(), tuple(default.orientation()))


def _restore_camera(core, values):
    center, shape, orientation, fovy, axes, default_orientation = values
    # Keep the very same native camera object; invalidate its derived caches
    # through the real setters rather than replacing a renderer-held handle.
    core.set_orientation(default_orientation)
    core.make_orientation_default()
    core.set_euler_axes(axes)
    core.set_shape(shape)
    core.set_center(center)
    core.set_field_of_view(fovy)
    core.set_orientation(orientation)


def install_scene_state(native: Any) -> None:
    """Complete the existing public SceneState class without changing aliases."""
    g = vars(native)
    if g.get("_FMN_SCENE_STATE_INSTALLED", False):
        return
    State, Scene, np = g["SceneState"], g["Scene"], g["_np"]
    original_restore = State.restore_scene

    def initialize(self, scene, ignore=None):
        if not isinstance(scene, Scene):
            raise TypeError("SceneState scene must be a Scene")
        ignored = set() if ignore is None else {id(obj) for obj in ignore}
        all_roots = tuple(scene.mobjects)
        roots = tuple(root for root in all_roots if id(root) not in ignored)
        members = _family(roots)
        frame = scene.frame
        attribute_members = list(members)
        if id(frame) not in ignored and all(frame is not member for member in members):
            attribute_members.append(frame)
        self._attribute_members = tuple(attribute_members)
        self._attributes = scene_attributes.capture(attribute_members, np)
        self._scene, self.time, self.num_plays = scene, scene.get_time(), int(scene.num_plays)
        self._root_topologies = {
            id(root): tuple((id(member), tuple(id(child) for child in member.submobjects))
                            for member in _family([root]))
            for root in roots
        }
        previous = scene.undo_stack[-1] if scene.undo_stack else None
        last = {} if previous is None else previous.mobjects_to_copies
        old_topologies = getattr(previous, "_root_topologies", {})
        self.mobjects_to_copies = {}
        for root in roots:
            prior = last.get(root)
            if (prior is not None
                    and self._root_topologies[id(root)] == old_topologies.get(id(root))
                    and attributes_match(self, previous, id(root))
                    and bool(prior.looks_identical(root))
                    and _same_mobject(np, root, prior, g["_raster_images_equal"])):
                saved = prior
            else:
                saved = root.copy()
                for member in _family([saved]):
                    # Ordinary Mobject.copy deliberately shallow-copies extras.
                    # A checkpoint mirror must not change with a live ndarray.
                    extras = getattr(member.uniforms, "_extras", None)
                    if extras is not None:
                        member.uniforms._extras = _freeze_value(np, extras)
                    for name in ("locked_data_keys", "const_data_keys", "locked_uniform_keys"):
                        setattr(member, name, set(getattr(member, name, ())))
            self.mobjects_to_copies[root] = saved
        # Native bytes cannot encode Python callback identities or arbitrary
        # extension uniforms. Freeze those without copying executable objects.
        self._python_projections = []
        for member in members:
            extras = getattr(member.uniforms, "_extras", None)
            self._python_projections.append((
                member, tuple(member.submobjects), tuple(member.updaters),
                None if extras is None else _freeze_value(np, extras),
                tuple(set(getattr(member, name, ())) for name in (
                    "locked_data_keys", "const_data_keys", "locked_uniform_keys",
                )),
            ))
        self._camera_snapshot = None if id(frame) in ignored else (
            frame, _camera_values(frame._core), tuple(frame.updaters),
        )
        # Copying a bound Mobject allocates native helper entries. Capture
        # AFTER those allocations. Ignoring a non-drawable camera does not
        # require discarding the native record/clock/RNG checkpoint.
        self._checkpoint = (bytes(scene._checkpoint_bytes())
                            if len(roots) == len(all_roots) else None)

    def attributes_match(self, other, root_id):
        ids = [row[0] for row in self._root_topologies[root_id]]
        other_values = getattr(other, "_attributes", {})
        return (all(key in other_values for key in ids)
                and scene_attributes.same(np,
                    {key: self._attributes[key] for key in ids},
                    {key: other_values[key] for key in ids}))

    def camera_matches(self, other):
        a, b = self._camera_snapshot, other._camera_snapshot
        if a is None or b is None:
            return a is b
        return (a[0] is b[0] and _same_value(np, a[1], b[1])
                and len(a[2]) == len(b[2])
                and all(x is y for x, y in zip(a[2], b[2]))
                and scene_attributes.same(np, self._attributes[id(a[0])],
                                           other._attributes[id(b[0])]))

    def mobjects_match(self, state):
        if not isinstance(state, State):
            raise TypeError("mobjects_match expects a SceneState")
        left, right = self.mobjects_to_copies, state.mobjects_to_copies
        # A plain dict equality loses root order, which controls compositing.
        return (len(left) == len(right)
                and self._root_topologies == state._root_topologies
                and all(a is b and _same_mobject(np, ac, bc, g["_raster_images_equal"])
                        for (a, ac), (b, bc) in zip(left.items(), right.items()))
                and scene_attributes.same(np, self._attributes, state._attributes)
                and camera_matches(self, state))

    def equals(self, state):
        if not isinstance(state, State):
            return NotImplemented
        return (self.time == state.time and self.num_plays == state.num_plays
                and self.mobjects_match(state))

    def n_changes(self, state):
        if not isinstance(state, State):
            raise TypeError("n_changes expects a SceneState")
        if self is state:
            live = scene_attributes.capture(self._attribute_members, np)
            count = sum(not _same_mobject(np, mob, saved, g["_raster_images_equal"]) or not scene_attributes.same(
                np, {key: self._attributes[key] for key, _ in self._root_topologies[id(mob)]},
                {key: live[key] for key, _ in self._root_topologies[id(mob)]})
                for mob, saved in self.mobjects_to_copies.items())
            if self._camera_snapshot is not None:
                frame, values, callbacks = self._camera_snapshot
                count += int(not _same_value(np, values, _camera_values(frame._core))
                             or len(callbacks) != len(frame.updaters)
                             or any(a is not b for a, b in zip(callbacks, frame.updaters))
                             or not scene_attributes.same(np, self._attributes[id(frame)],
                                                          live[id(frame)]))
            return count or int(not scene_attributes.same(np, self._attributes, live))
        other = state.mobjects_to_copies
        # Compare two captured states, not a source object edited afterward.
        count = sum(mob not in other
                    or self._root_topologies[id(mob)] != state._root_topologies.get(id(mob))
                    or not attributes_match(self, state, id(mob))
                    or not _same_mobject(np, saved, other[mob], g["_raster_images_equal"])
                    for mob, saved in self.mobjects_to_copies.items())
        count += int(not camera_matches(self, state))
        # Cross-root container aliasing can change while each root still has
        # equal values in isolation. It must still register as a state edit.
        return count or int(not scene_attributes.same(np, self._attributes, state._attributes))

    @wraps(original_restore)
    def restore(self, scene):
        if not isinstance(scene, Scene):
            raise TypeError("restore_scene expects a Scene")
        if scene is not self._scene:
            error = g.get("_ForeignStageError", ValueError)
            raise error("SceneState belongs to another Scene; copy its mobjects instead")
        # Allocate and validate the complete Python projection first. A
        # refusal must not leave restored geometry with newer authored data.
        prepared = scene_attributes.prepare_restore(self._attributes, np, self._attribute_members)
        original_restore(self, scene)
        if self._checkpoint is not None:
            # Native restore has already restored all graph edges and records.
            # Refresh ONLY Python projections, without become/alignment writes
            # that would replace identities or invalidate restored view epochs.
            for member, children, updaters, extras, locks in self._python_projections:
                member.submobjects._replace_projection(children)
                member.updaters[:] = updaters
                if extras is not None:
                    member.uniforms._extras = _freeze_value(np, extras)
                for name, values in zip(("locked_data_keys", "const_data_keys", "locked_uniform_keys"), locks):
                    setattr(member, name, set(values))
            # Rehydrate native-only roots and refresh back-edges before a caller
            # reads a retained child reference without first reading mobjects.
            _ = scene.mobjects
        scene_attributes.restore(self._attribute_members, prepared)
        if self._camera_snapshot is not None:
            frame, values, updaters = self._camera_snapshot
            scene.frame = frame
            _restore_camera(frame._core, values)
            frame.updaters[:] = updaters

    for name, function in {
        "__init__": initialize, "__eq__": equals, "mobjects_match": mobjects_match,
        "n_changes": n_changes, "restore_scene": restore,
    }.items():
        _method(State, name, function)
    State.__hash__ = None
    _install_history(Scene)
    g["_FMN_SCENE_STATE_INSTALLED"] = True


def _install_history(Scene):
    """Commit history transitions only after the existing restore hook returns."""
    def limit(self):
        value = self.max_num_saved_states
        if isinstance(value, bool):
            raise TypeError("max_num_saved_states must be a nonnegative integer")
        try:
            value = operator.index(value)
        except TypeError:
            raise TypeError("max_num_saved_states must be a nonnegative integer") from None
        if value < 0:
            raise ValueError("max_num_saved_states must be nonnegative")
        return value

    def trim(stack, maximum):
        if len(stack) > maximum:
            del stack[:len(stack) - maximum]

    def save_state(self):
        maximum = limit(self)
        if maximum == 0:
            self.undo_stack.clear()
            self.redo_stack.clear()
            return self
        state = self.get_state()
        self.redo_stack.clear()
        if self.undo_stack and state == self.undo_stack[-1]:
            trim(self.undo_stack, maximum)
            return self
        self.undo_stack.append(state)
        trim(self.undo_stack, maximum)
        return self

    def undo(self):
        if not self.undo_stack:
            return
        maximum = limit(self)
        target = self.undo_stack[-1]
        current = self.get_state()
        self.restore_state(target)
        self.undo_stack.pop()
        self.redo_stack.append(current)
        trim(self.redo_stack, maximum)

    def redo(self):
        if not self.redo_stack:
            return
        maximum = limit(self)
        target = self.redo_stack[-1]
        current = self.get_state()
        self.restore_state(target)
        self.redo_stack.pop()
        self.undo_stack.append(current)
        trim(self.undo_stack, maximum)

    for name, function in {"save_state": save_state, "undo": undo, "redo": redo}.items():
        _method(Scene, name, function)
