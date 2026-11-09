"""Source-ordered text reveals over the native, identity-bearing glyph tree.

A glyph span can address any descendant, including a complete live numeric
family. Revealing it changes native child lists, never clones, flattens, or
re-typesets glyph geometry. Scribe's span order selects what is visible;
Marionette's original sibling order still controls compositing.
"""
from __future__ import annotations

from functools import wraps
import math
import operator
import sys
from typing import Any

_MISSING = object()


def _same_children(left, right):
    return len(left) == len(right) and all(a is b for a, b in zip(left, right))


def _note(error, cleanup):
    try:
        error.add_note("text reveal rollback also failed: " + type(cleanup).__name__)
    except BaseException:
        pass


class _GlyphRevealPlan:
    """Freeze paths and parent identities before hiding any part of a string."""

    def __init__(self, root):
        paths = []
        for raw in root._string_sub_paths:
            path = []
            for index in raw:
                if isinstance(index, bool):
                    raise ValueError("text span paths require non-negative integer indices")
                index = operator.index(index)
                if index < 0:
                    raise ValueError("text span paths require non-negative integer indices")
                path.append(index)
            paths.append(tuple(path))
        if not paths or len(paths) != len(root._string_sub_spans):
            raise ValueError("text reveal requires matching, nonempty source spans and family paths")
        ordered = sorted(paths)
        for left, right in zip(ordered, ordered[1:]):
            if right[:len(left)] == left:
                raise ValueError("text span paths must not duplicate or contain another glyph path")

        nodes, children, first, last = {(): root}, {}, {}, {}
        for ordinal, path in enumerate(paths):
            prefix, ancestors = (), {id(root)}
            for index in path:
                node = nodes[prefix]
                if prefix not in children:
                    children[prefix] = tuple(node.submobjects)
                if index >= len(children[prefix]):
                    raise ValueError("text span path no longer matches its native family")
                last[prefix] = ordinal + 1
                following = (*prefix, index)
                first.setdefault(following, ordinal + 1)
                node = children[prefix][index]
                if id(node) in ancestors:
                    raise ValueError("text reveal cannot traverse a cyclic glyph family")
                ancestors.add(id(node))
                nodes[following] = node
                prefix = following
        if paths == [()]:
            # A native whole-entry span may live in the root's own records.
            children[()] = tuple(root.submobjects)
            last[()] = 1

        self.root, self.count = root, len(paths)
        self.visible_count = self.count
        self._rows, self._data_visible = [], {}
        identities = set()
        for path in sorted(children, key=lambda item: (-len(item), item)):
            node, original = nodes[path], children[path]
            if id(node) in identities:
                raise ValueError("text reveal paths alias a mutable parent through multiple branches")
            identities.add(id(node))
            # Unmapped ornaments become visible with their completed parent,
            # not early with an unrelated glyph; restore every ornament at 1.
            thresholds = tuple(first.get((*path, i), last[path]) for i in range(len(original)))
            data = node.data.copy() if len(node.data) else None
            self._rows.append((node, original, thresholds, data, last[path]))
            self._data_visible[id(node)] = True

    def apply(self, count):
        """Apply a prefix transactionally, including backward timeline seeks."""
        count = min(max(operator.index(count), 0), self.count)
        operations, visible = [], {}
        for node, original, thresholds, data, completed in self._rows:
            desired = tuple(child for child, threshold in zip(original, thresholds) if count >= threshold)
            prior = tuple(node.submobjects)
            if data is not None:
                show = count >= completed
                visible[id(node)] = show
                if show != self._data_visible[id(node)]:
                    operations.append((node.set_data, node.data.copy(), data if show else data[:0]))
            if not _same_children(prior, desired):
                operations.append((node.set_submobjects, list(prior), list(desired)))
        attempted = []
        try:
            for setter, old, new in operations:
                # An authored setter can mutate the native family and THEN
                # raise. Include that setter in rollback, not just successes.
                attempted.append((setter, old))
                setter(new)
        except BaseException as error:
            for setter, old in reversed(attempted):
                try:
                    setter(old)
                except BaseException as cleanup:
                    _note(error, cleanup)
            raise
        self._data_visible.update(visible)
        self.visible_count = count

    def restore(self):
        self.apply(self.count)


def _apply_glyph_reveal(root, plan, count):
    if plan.root is not root:
        raise ValueError("text reveal plan belongs to a different StringMobject")
    plan.apply(count)


def _transients(root):
    """Capture even glyphs about to detach, plus affected outside ancestors."""
    pending, states, seen = list(root.get_family()), [], set()
    while pending:
        node = pending.pop()
        if id(node) in seen:
            continue
        seen.add(id(node))
        states.append((node, node._is_updating_suspended(),
                       vars(node).get("_is_animating", _MISSING)))
        pending.extend(tuple(getattr(node, "parents", ())))
    return states


def _release(states):
    first = None
    for node, suspended, animating in states:
        try:
            now = node._is_updating_suspended()
            if suspended and not now:
                node.suspend_updating(recurse=False)
            elif not suspended and now:
                node.resume_updating(recurse=False, call_updater=False)
        except BaseException as error:
            if first is None:
                first = error
            else:
                _note(first, error)
        try:
            if animating is _MISSING:
                vars(node).pop("_is_animating", None)
            else:
                vars(node)["_is_animating"] = animating
        except BaseException as error:
            if first is None:
                first = error
            else:
                _note(first, error)
    if first is not None:
        raise first


def _install_lifecycle(g):
    Animation = g["Animation"]

    def configure(self):
        plan = _GlyphRevealPlan(self.mobject)
        if isinstance(self, g["AddTextWordByWord"]):
            groups = g["_string_word_groups"](self.mobject)
            if not groups:
                raise ValueError("AddTextWordByWord requires span-map word groups")
            boundaries = [0, *(group[-1] + 1 for group in groups)]
        else:
            boundaries = list(range(plan.count + 1))
        self._nested_plan, self._boundaries = plan, boundaries
        self._all_submobs = []

    def abort(self):
        if not getattr(self, "_text_reveal_active", False):
            return
        first = None
        try:
            self._nested_plan.restore()
        except BaseException as error:
            first = error
        try:
            _release(self._text_reveal_states)
        except BaseException as error:
            if first is None:
                first = error
            else:
                _note(first, error)
        if first is not None:
            # Keep ownership for an explicit retry; never claim clean state
            # when an authored/native setter refused restoration.
            raise first
        self._text_reveal_active = False
        self._text_reveal_states = []

    def unwind(self, error):
        try:
            abort(self)
        except BaseException as cleanup:
            _note(error, cleanup)

    def begin(self):
        abort(self)
        if self._nested_plan.visible_count != self._nested_plan.count:
            self._nested_plan.restore()
        # Scene construction can regroup glyphs or change live numeric spans
        # after creating this animation. Freeze the actual begin-time tree.
        configure(self)
        self._text_reveal_states = _transients(self.mobject)
        self._text_reveal_active = True
        try:
            Animation.begin(self)
        except BaseException as error:
            unwind(self, error)
            raise

    def interpolate_mobject(self, alpha):
        try:
            alpha = float(alpha)
            if not math.isfinite(alpha):
                raise ValueError("text reveal alpha must be finite")
            # Pinned ShowIncreasingSubsets applies its rate to raw alpha,
            # then calls the live int_func once on the number of reveal units.
            # Preserve authored stateful selectors instead of approximating
            # their behavior with a fixed rounding enum.
            shaped = float(self.rate_func(alpha))
            if not math.isfinite(shaped):
                raise ValueError("text reveal rate function must return a finite value")
            units = len(self._boundaries) - 1
            index = int(self.int_func(shaped * units))
            self.update_submobject_list(index)
        except BaseException as error:
            unwind(self, error)
            raise

    def update_submobject_list(self, index):
        # Python prefix slicing includes negative indices. Translate the unit
        # count through Scribe's source spans without flattening the glyph tree
        # or changing the native painter order. range keeps huge indices cheap.
        count = len(range(len(self._boundaries) - 1)[:index])
        self._nested_plan.apply(self._boundaries[count])

    def interpolate(self, alpha):
        try:
            Animation.interpolate(self, alpha)
        except BaseException as error:
            # Authored interpolate_mobject overrides can fail after super has
            # already hidden glyphs. Own that failure boundary as well.
            unwind(self, error)
            raise

    def update_mobjects(self, dt):
        try:
            Animation.update_mobjects(self, dt)
        except BaseException as error:
            unwind(self, error)
            raise

    def finish(self):
        try:
            self.interpolate(self.final_alpha_value)
            if getattr(self, "_text_reveal_active", False):
                _release(self._text_reveal_states)
                # Preserve successful resume's zero-dt updater opportunity,
                # after restoring each child's original suspension state.
                if self.suspend_mobject_updating and self.mobject_was_updating:
                    self.mobject.update(0)
                self._text_reveal_active = False
                self._text_reveal_states = []
        except BaseException as error:
            unwind(self, error)
            raise

    def initializer(original):
        @wraps(original)
        def initialize(self, *args, **kwargs):
            selector = kwargs.pop("int_func", g["_np"].round)
            if not callable(selector):
                raise TypeError("text reveal int_func must be callable")
            original(self, *args, **kwargs)
            self.int_func = selector
            if not math.isfinite(float(self.run_time)) or self.run_time < 0:
                raise ValueError("text reveal run_time must be finite and non-negative")
            if not callable(self.rate_func):
                raise TypeError("text reveal rate_func must be callable")
            configure(self)
            self._text_reveal_active = False
            self._text_reveal_states = []
        return initialize

    # The schema deliberately omits non-wildcard exports from the root.
    # AddTextLetterByLetter remains available on its qualified module; patch
    # that same class without inventing another root export or class identity.
    letter = g.get("AddTextLetterByLetter")
    if letter is None:
        creation = sys.modules.get("manimlib.animation.creation")
        letter = getattr(creation, "AddTextLetterByLetter", None)
    if not isinstance(letter, type) or not issubclass(letter, Animation):
        raise ImportError("missing authored manimlib.animation.creation.AddTextLetterByLetter")
    for cls in (g["AddTextWordByWord"], letter):
        cls.__init__ = initializer(cls.__init__)
        cls.begin, cls.finish, cls.abort = begin, finish, abort
        cls.interpolate, cls.interpolate_mobject = interpolate, interpolate_mobject
        cls.update_submobject_list = update_submobject_list
        cls.update_mobjects = update_mobjects


def install_text_reveal(native: Any) -> None:
    """Install on the shared bootstrap namespace without replacing classes."""
    g = vars(native)
    if g.get("_FMN_TEXT_REVEAL_INSTALLED", False):
        return
    g["_string_glyph_reveal_plan"] = _GlyphRevealPlan
    g["_apply_glyph_reveal"] = _apply_glyph_reveal
    _install_lifecycle(g)
    g["_FMN_TEXT_REVEAL_INSTALLED"] = True
