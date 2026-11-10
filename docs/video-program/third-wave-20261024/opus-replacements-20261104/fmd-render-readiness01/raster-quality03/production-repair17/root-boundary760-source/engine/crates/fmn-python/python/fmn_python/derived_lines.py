"""Derived lines use public sampling and slicing over native path geometry.

Chisel owns arclength, path samples and partial curves. Python coordinates the
Line lifecycle and authored overrides instead of installing a replacement tree.
"""
from __future__ import annotations

import math
import operator
from itertools import islice

from .shape_matchers import _bind, _finite


def install_derived_lines(native):
    g = vars(native)
    if g.get('_FMN_DERIVED_LINES_INSTALLED', False):
        return
    Dashed, Tangent, VMobject = (g[name] for name in ('DashedLine', 'TangentLine', 'VMobject'))
    original_count = Dashed.calculate_num_dashes

    def dash_parameters(dash_length, ratio):
        dash_length, ratio = float(dash_length), float(ratio)
        if not math.isfinite(dash_length) or dash_length <= 0:
            raise ValueError('dash length must be positive and finite')
        if not math.isfinite(ratio) or not 0 < ratio <= 1:
            raise ValueError('positive-space ratio must be finite and in (0, 1]')
        period = dash_length / ratio
        if not math.isfinite(period) or period <= 0:
            raise ValueError('dash count is not representable')
        return dash_length, ratio, period

    def calculate_num_dashes(self, dash_length, positive_space_ratio):
        dash_length, ratio, period = dash_parameters(dash_length, positive_space_ratio)
        if not self.has_points():
            # After dashing, the existing API queries the virtual endpoints.
            return original_count(self, dash_length, ratio)
        # During construction use the actual authored/buffered contour, not
        # a different arc reconstructed from its endpoints and an old angle.
        length = float(self.get_arc_length())
        count = length / period
        if not math.isfinite(count) or length < 0:
            raise ValueError('dash count is not representable')
        count = max(1, math.ceil(count))
        if count > 4096:
            raise ValueError('requested dash count is above the 4096 cap')
        return count

    def dashed_init(self, start=g['_LEFT'], end=g['_RIGHT'], dash_length=0.05,
                    positive_space_ratio=0.5, **kwargs):
        dash_length, ratio, _ = dash_parameters(dash_length, positive_space_ratio)
        unknown = sorted(set(kwargs) - g['_NATIVE_VMOBJECT_STYLE_KEYS'] - {'shading', 'buff', 'path_arc'})
        g['_refuse_unrouted']('DashedLine()', [(name, True) for name in unknown])
        if self._is_bound():
            raise RuntimeError('DashedLine construction requires a detached target')
        super(Dashed, self).__init__(start, end, **kwargs)
        count = operator.index(self.calculate_num_dashes(dash_length, ratio))
        if not 0 <= count <= 4096:
            raise ValueError('requested dash count is negative or above the 4096 cap')
        # Existing DashedVMobject uses native true-length cut intervals and
        # the public get_subcurve hook; every slice is ready before publication.
        dashes = g['DashedVMobject'](self, num_dashes=count, positive_space_ratio=ratio)
        roots = tuple(dashes.submobjects)
        if len({id(root) for root in roots}) != len(roots):
            raise ValueError('dash slices must have distinct roots')
        forbidden = {id(member) for member in self.get_family()}
        pending, seen, total = list(roots), set(), 0
        while pending:
            member = pending.pop()
            if id(member) in seen:
                continue
            if (not isinstance(member, VMobject) or id(member) in forbidden
                    or getattr(member, '_scene', None) is not None or member._is_bound()):
                raise ValueError('dash slices must be detached independent VMobject families')
            seen.add(id(member))
            points = member.get_points()
            total += len(points)
            if len(seen) > 65536 or total > 1048576:
                raise ValueError('dash slices exceed the family/record budget')
            if not g['_np'].isfinite(points).all():
                raise ValueError('dash slice points must be finite')
            children = tuple(islice(iter(member.submobjects), 65537))
            if len(children) > 65536:
                raise ValueError('dash slices exceed the family budget')
            pending.extend(children)
        # Authored geometry getters may adopt an earlier slice during validation.
        if self._is_bound() or any(getattr(member, '_scene', None) is not None
                                   or member._is_bound() for root in roots
                                   for member in root.get_family()):
            raise ValueError('dash ownership changed before publication')
        self.clear_points()
        self.add(*roots)

    def tangent_init(self, vmob, alpha, length=2, d_alpha=1e-6, **kwargs):
        if not isinstance(vmob, VMobject):
            raise TypeError('TangentLine expects a VMobject')
        alpha = _finite(alpha, 'TangentLine alpha')
        length = _finite(length, 'TangentLine length')
        d_alpha = _finite(d_alpha, 'TangentLine d_alpha')
        g['_preflight_vmobject_style_kwargs'](kwargs)
        if self._is_bound():
            raise RuntimeError('TangentLine construction requires a detached target')
        empty = not vmob.has_points()
        if empty:
            start = end = g['_ORIGIN']
        else:
            samples = []
            for a in (max(0.0, min(1.0, alpha - d_alpha)),
                      max(0.0, min(1.0, alpha + d_alpha))):
                # Call the public sampler exactly at the two authored samples.
                point = g['_vec3'](vmob.pfp(a))
                if not all(math.isfinite(value) for value in point):
                    raise ValueError('TangentLine samples must be finite')
                samples.append(point)
            start, end = samples
        super(Tangent, self).__init__(start, end, **kwargs)
        if empty:
            self.clear_points()
        else:
            current = float(self.get_length())
            if not math.isfinite(current):
                raise ValueError('TangentLine length must remain finite')
            if current:
                self.scale(length / current)

    _bind(Dashed, '__init__', dashed_init)
    _bind(Dashed, 'calculate_num_dashes', calculate_num_dashes)
    _bind(Tangent, '__init__', tangent_init)
    g['_FMN_DERIVED_LINES_INSTALLED'] = True
