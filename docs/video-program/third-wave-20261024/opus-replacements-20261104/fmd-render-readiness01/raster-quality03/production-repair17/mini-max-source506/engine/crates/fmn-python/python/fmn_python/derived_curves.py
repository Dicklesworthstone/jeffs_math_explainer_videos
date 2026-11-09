"""Source-derived curve families over native paths, intervals and record copies.

Construction preserves the Reference's order: initialize the actual receiver,
then sample source geometry and add the resulting children. Explicit init_points
rebuilds only those generated children, retaining authored root data/decorations.
No curve evaluation, arclength solver, renderer or frame clock lives here.
"""
from __future__ import annotations

from itertools import islice
import math
import operator
import sys

from .copying import FamilyRefs
from .invocation import InvocationGuard

_MAX_PARTS = 65_536
_MAX_RECORDS = 16_777_216


def install_derived_curves(native):
    g = vars(native)
    if g.get("_FMN_DERIVED_CURVES_INSTALLED", False):
        return
    Curves, Dashed, VMobject = (g[name] for name in
                               ("CurvesAsSubmobjects", "DashedVMobject", "VMobject"))
    np = g["_np"]
    constructors, sampling = InvocationGuard(), InvocationGuard()
    keys = ("get_bezier_tuples", "get_bezier_tuples_from_points", "get_points",
            "__getattribute__", "__getattr__")
    protocol = {name: getattr(VMobject, name, None) for name in keys}

    def generated(obj, child):
        owners = vars(child).get("_fmn_derived_owner", ())
        return len(owners) == 1 and owners[0] is obj

    def mark(obj, parts):
        # FamilyRefs remaps a whole-owner copy but retains an external owner
        # when alignment copies only one generated child. Thus invisible
        # padding copies remain generated, not accidental permanent decoration.
        for part in parts:
            vars(part)["_fmn_derived_owner"] = FamilyRefs((obj,))

    def idle(obj):
        if vars(obj).get("_is_animating", False) or getattr(obj, "locked_data_keys", ()):
            raise RuntimeError("release the derived path's active animation before rebuilding")

    def source(obj, value):
        if not isinstance(value, VMobject):
            raise TypeError(type(obj).__name__ + " expects a VMobject")
        if value is obj:
            raise ValueError("a derived path cannot be its own source")
        return value

    def bounded(values):
        result = list(islice(iter(values), _MAX_PARTS + 1))
        if len(result) > _MAX_PARTS:
            raise ValueError("derived path exceeds the 65536-part budget")
        return result

    def admit(obj, parts, protected):
        """Validate the complete detached DAG before linking any generated root."""
        if len({id(part) for part in parts}) != len(parts):
            raise ValueError("derived path factory returned duplicate roots")
        pending = [(part, False) for part in reversed(parts)]
        seen, visiting, members, count = set(), set(), [], 0
        while pending:
            member, leaving = pending.pop()
            marker = id(member)
            if leaving:
                visiting.remove(marker)
                continue
            if marker in visiting:
                raise ValueError("derived path factory returned a cyclic family")
            if marker in seen:
                continue
            if not isinstance(member, VMobject) or member is obj or marker in protected:
                raise ValueError("derived path factory must return independent VMobjects")
            if member._is_bound() or getattr(member, "_scene", None) is not None:
                raise ValueError("derived path factory must return detached objects; copy scene objects first")
            if len(seen) >= _MAX_PARTS:
                raise ValueError("derived path family exceeds the 65536-member budget")
            seen.add(marker)
            visiting.add(marker)
            members.append(member)
            count += member.get_num_points()
            if count > _MAX_RECORDS:
                raise ValueError("derived path family exceeds the 16777216-record budget")
            if not np.isfinite(member.get_points()).all():
                raise ValueError("derived path geometry must be finite")
            children = bounded(member.submobjects)
            if len(pending) + len(children) > 2 * _MAX_PARTS:
                raise ValueError("derived path family exceeds the 65536-member budget")
            pending.append((member, True))
            pending.extend((child, False) for child in reversed(children))
        if any(member._is_bound() or getattr(member, "_scene", None) is not None for member in members):
            raise ValueError("derived path ownership changed during admission")

    def family_ids(roots):
        pending, seen = list(roots), set()
        while pending:
            member = pending.pop()
            if id(member) in seen:
                continue
            if len(seen) >= _MAX_PARTS:
                raise ValueError("derived path input family exceeds the 65536-member budget")
            seen.add(id(member))
            children = bounded(member.submobjects)
            if len(pending) + len(children) > 2 * _MAX_PARTS:
                raise ValueError("derived path input family exceeds the 65536-member budget")
            pending.extend(children)
        return seen

    def curve_parts(obj, target, protected):
        module = sys.modules.get(Curves.__module__)
        factory = getattr(module, "VMobject", VMobject)
        # Admission must not execute authored descriptors or get_points twice.
        # Compare the actual MRO/instance slots before choosing the native route.
        def implementation(name):
            if name in vars(target):
                return vars(target)[name]
            return next((vars(cls)[name] for cls in type(target).__mro__
                         if name in vars(cls)), None)
        stock = factory is VMobject and all(implementation(name) is expected
                                            for name, expected in protocol.items())
        if stock:
            if len(target.get_points()) // 2 > _MAX_PARTS:
                raise ValueError("derived path exceeds the 65536-part budget")
            candidate = g["_native_shell_factory"]()
            specs = candidate._build_curves_as_submobjects(g["_native_shell_factory"], target)
            g["_hang_native_children"](candidate, specs)
            parts = list(candidate.submobjects)
            candidate.set_submobjects([])
        else:
            # get_bezier_tuples is a public authored protocol. This copies only
            # its triples; native VMobject.set_points remains the data authority.
            parts, part_ids = [], set()
            for index, value in enumerate(target.get_bezier_tuples()):
                if index >= _MAX_PARTS:
                    raise ValueError("derived path exceeds the 65536-part budget")
                # Snapshot each yield before asking a generator for the next;
                # authored generators may reuse the same mutable point array.
                points = np.array(value, dtype=float, copy=True)
                if points.shape != (3, 3) or not np.isfinite(points).all():
                    raise ValueError("a derived curve requires three finite 3D control points")
                part = factory()
                if not isinstance(part, VMobject):
                    raise TypeError("derived curve factory must return a VMobject")
                if part is obj or id(part) in protected or id(part) in part_ids:
                    raise ValueError("derived curve factory must return an independent object")
                if part._is_bound() or getattr(part, "_scene", None) is not None:
                    raise ValueError("derived curve factory must return detached objects")
                part.set_points(points)
                parts.append(part)
                part_ids.add(id(part))
        admit(obj, parts, protected)
        for part in parts:
            part.match_style(target)
        return parts

    def dash_recipe(obj):
        count = operator.index(obj.num_dashes)
        ratio = float(obj.positive_space_ratio) if count > 0 else None
        # Keep the owning Atlas contract and its named refusal before invoking
        # arbitrary source slicers; non-positive counts mean an empty pattern.
        if count > 4096:
            raise ValueError("dash count exceeds the native 4096-child budget")
        if count > 0 and (not math.isfinite(ratio) or not 0 < ratio <= 1):
            raise ValueError("positive-space ratio must be finite and in (0, 1]")
        return count, ratio

    def rebuild(obj):
        with sampling.hold(obj, message="derived path rebuilding is already in progress"):
            idle(obj)
            target = source(obj, obj._derived_source)
            recipe = dash_recipe(obj) if isinstance(obj, Dashed) else None
            owner, bound = vars(obj).get("_scene"), obj._is_bound()
            before, children = obj.data.copy(), tuple(obj.submobjects)
            previous = tuple(vars(obj).get("_derived_parts", ()))
            protected = family_ids((target, *children))
            if recipe is None:
                parts = curve_parts(obj, target, protected)
            elif recipe[0] <= 0:
                parts = []
            else:
                intervals = bounded(target._dash_curve_intervals(*recipe))
                parts = [target.get_subcurve(a, b) for a, b in intervals]
            admit(obj, parts, protected)
            idle(obj)
            if (vars(obj).get("_scene") is not owner or obj._is_bound() != bound
                    or obj._derived_source is not target or tuple(obj.submobjects) != children
                    or obj.data.dtype != before.dtype or obj.data.tobytes() != before.tobytes()
                    or tuple(vars(obj).get("_derived_parts", ())) != previous
                    or (recipe is not None and dash_recipe(obj) != recipe)):
                raise RuntimeError("derived path changed during sampling; children were not published")
            old = {id(part) for part in previous}
            old.update(id(child) for child in children if generated(obj, child))
            # Replace the generated run in place; retain decorations before,
            # after, or interspersed with it and preserve their exact identities.
            replacement, inserted = [], False
            for child in children:
                if id(child) in old:
                    if not inserted:
                        replacement.extend(parts)
                        inserted = True
                else:
                    replacement.append(child)
            if not inserted:
                replacement.extend(parts)
            obj.set_submobjects(replacement)
            mark(obj, parts)
            obj._derived_parts = FamilyRefs(parts)
        return obj

    def construct(obj, target):
        source(obj, target)
        if obj._is_bound():
            raise RuntimeError("derived path construction requires a detached target; use init_points")
        idle(obj)

    def curves(self, vmobject, **kwargs):
        construct(self, vmobject)
        g["_preflight_vmobject_style_kwargs"](kwargs)
        with constructors.hold(self, message="derived path initialization is already in progress"):
            self._derived_source = vmobject
            super(Curves, self).__init__(**kwargs)
            if self._is_bound():
                raise RuntimeError("derived path ownership changed during initialization")
            rebuild(self)

    def dashed(self, vmobject, num_dashes=15, positive_space_ratio=.5, **kwargs):
        construct(self, vmobject)
        with constructors.hold(self, message="derived path initialization is already in progress"):
            self._derived_source = vmobject
            self.num_dashes, self.positive_space_ratio = num_dashes, positive_space_ratio
            dash_recipe(self)
            super(Dashed, self).__init__(**kwargs)
            if self._is_bound():
                raise RuntimeError("derived path ownership changed during initialization")
            rebuild(self)
            self.match_style(vmobject, recurse=False)
            if "stroke_behind" in kwargs:
                self.set_stroke(behind=kwargs["stroke_behind"], recurse=False)
            if "flat_stroke" in kwargs:
                self.set_flat_stroke(kwargs["flat_stroke"], recurse=False)
                self.flat_stroke = bool(kwargs["flat_stroke"])

    def curve_points(self):
        if constructors.busy(self):
            return super(Curves, self).init_points()
        return rebuild(self)

    def dash_points(self):
        if constructors.busy(self):
            return super(Dashed, self).init_points()
        return rebuild(self)

    def padding(cls):
        def add_n_more_submobjects(self, n):
            empty = not self.submobjects
            result = super(cls, self).add_n_more_submobjects(n)
            if empty:
                # The stock empty-family aligner creates point copies of the
                # container; there is no prior child from which to inherit a role.
                mark(self, self.submobjects)
            return result
        return add_n_more_submobjects

    for cls, name, function in ((Curves, "__init__", curves), (Curves, "init_points", curve_points),
                               (Dashed, "__init__", dashed), (Dashed, "init_points", dash_points),
                               (Curves, "add_n_more_submobjects", padding(Curves)),
                               (Dashed, "add_n_more_submobjects", padding(Dashed))):
        function.__name__ = name
        function.__qualname__ = cls.__qualname__ + "." + name
        function.__module__ = cls.__module__
        setattr(cls, name, function)
    g["_FMN_DERIVED_CURVES_INSTALLED"] = True
