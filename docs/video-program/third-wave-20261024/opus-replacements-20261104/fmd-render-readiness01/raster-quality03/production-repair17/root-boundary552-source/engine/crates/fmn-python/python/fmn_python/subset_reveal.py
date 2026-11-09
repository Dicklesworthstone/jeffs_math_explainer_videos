"""Authored subset selection over native, identity-bearing child lists.

The host selects a child count; Marionette owns every object and topology
mutation, and Choreo owns sampling. No rounding-rule approximation or second
clock is involved. The Reference's one-by-one endpoint convention is retained.
"""
from __future__ import annotations

import math
from typing import Any

from fmn_python.text_reveal import _note, _release, _transients


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def install_subset_reveal(native: Any) -> None:
    """Complete the existing classes, without replacing qualified identities."""
    g = vars(native)
    if g.get("_FMN_SUBSET_REVEAL_INSTALLED", False):
        return
    Subsets, Animation, Mobject = (g[name] for name in
                                  ("ShowIncreasingSubsets", "Animation", "Mobject"))
    np = g["_np"]

    def initialize(self, group, int_func=None, suspend_mobject_updating=False, **kwargs):
        if not isinstance(group, Mobject):
            raise TypeError(type(self).__name__ + " requires a Mobject family")
        if int_func is None:
            int_func = np.ceil if self._int_round_default == "ceil" else np.round
        if not callable(int_func):
            raise TypeError(type(self).__name__ + " int_func must be callable")
        # The Reference intentionally freezes these identities at construction,
        # not at begin. In particular, an empty group is a valid empty reveal.
        self.all_submobs = list(group.submobjects)
        self.int_func = int_func
        Animation.__init__(self, group, suspend_mobject_updating=suspend_mobject_updating,
                           **kwargs)
        self._subset_active = False
        self._subset_states = []
        self._subset_original_children = ()

    def native_params(self):
        # Compatibility for diagnostics of the two native fixed rules. An
        # authored callback cannot truthfully be encoded as a rounding enum.
        if self.int_func is np.round:
            return {"int_round": "round"}
        if self.int_func is np.ceil:
            return {"int_round": "ceil"}
        return {}

    def abort(self):
        if not self._subset_active:
            return
        error = None
        try:
            self.mobject.set_submobjects(list(self._subset_original_children))
        except BaseException as failure:
            error = failure
        try:
            _release(self._subset_states)
        except BaseException as failure:
            if error is None:
                error = failure
            else:
                _note(error, failure)
        if error is not None:
            # A setter may raise after mutation. Keep ownership so an explicit
            # retry can finish restoring topology without replaying callbacks.
            raise error
        self._subset_active = False
        self._subset_states = []
        self._subset_original_children = ()

    def unwind(self, error):
        try:
            abort(self)
        except BaseException as cleanup:
            _note(error, cleanup)

    def begin(self):
        abort(self)
        self._subset_original_children = tuple(self.mobject.submobjects)
        states, seen = [], set()
        # Detached candidates can become visible later, and ancestors outside
        # the animated group can be marked animating by native propagation.
        for root in (self.mobject, *self.all_submobs):
            for state in _transients(root):
                if id(state[0]) not in seen:
                    seen.add(id(state[0]))
                    states.append(state)
        self._subset_states = states
        self._subset_active = True
        try:
            Animation.begin(self)
            # begin's alpha-zero selection may have hidden these candidates.
            # Suspend/mark the whole candidate family, not just visible nodes.
            for root in self.all_submobs:
                root.set_animating_status(True)
                if self.suspend_mobject_updating:
                    root.suspend_updating()
        except BaseException as error:
            unwind(self, error)
            raise

    def interpolate_mobject(self, alpha):
        try:
            alpha = float(alpha)
            if not math.isfinite(alpha):
                raise ValueError("subset reveal alpha must be finite")
            # Do not pre-sample, cache, clamp, or substitute authored functions.
            # Reference creation.py:190 uses raw alpha (not time_spanned_alpha),
            # one rate call, one int_func call, then Python int conversion.
            rated = self.rate_func(alpha)
            index = int(self.int_func(rated * len(self.all_submobs)))
            self.update_submobject_list(index)
        except BaseException as error:
            unwind(self, error)
            raise

    def update_submobject_list(self, index):
        if self._one_by_one:
            # Keep the pinned Reference's clip-to-n-1 convention; changing its
            # endpoint independently of native Choreo would split semantics.
            clipped = int(min(max(index, 0), len(self.all_submobs) - 1))
            desired = [] if clipped <= 0 else [self.all_submobs[clipped - 1]]
        else:
            desired = self.all_submobs[:index]
        self.mobject.set_submobjects(list(desired))

    def interpolate(self, alpha):
        try:
            Animation.interpolate(self, alpha)
        except BaseException as error:
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
            if self._subset_active:
                _release(self._subset_states)
                if self.suspend_mobject_updating and self.mobject_was_updating:
                    self.mobject.update(0)
                self._subset_active = False
                self._subset_states = []
                self._subset_original_children = ()
        except BaseException as error:
            unwind(self, error)
            raise

    for name, function in (("__init__", initialize), ("_native_params", native_params),
                           ("begin", begin), ("abort", abort), ("finish", finish),
                           ("interpolate", interpolate),
                           ("interpolate_mobject", interpolate_mobject),
                           ("update_mobjects", update_mobjects),
                           ("update_submobject_list", update_submobject_list)):
        _method(Subsets, name, function)
    g["_FMN_SUBSET_REVEAL_INSTALLED"] = True
