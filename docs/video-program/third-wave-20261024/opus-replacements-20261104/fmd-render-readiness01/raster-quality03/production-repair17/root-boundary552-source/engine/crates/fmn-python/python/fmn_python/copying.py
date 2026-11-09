"""Owned ndarray attributes over Marionette's existing family-copy operation.

The native copier remains the owner of records, arena membership, shared family
identity and updater lists. This adapter completes plan section 8.3: ordinary
copying must also detach authored ndarray attributes before a generated target
can mutate them. Plain Python containers retain ordinary shallow-copy semantics.
"""
from __future__ import annotations

from functools import wraps

_INTERNAL = frozenset({
    "data", "submobjects", "uniforms", "updaters", "parents", "_scene",
    "target", "saved_state",
})


class FamilyRefs(tuple):
    """Generated family members recorded on their owner.

    The copier remaps these to the copy's own members, as it does object
    ndarray slots. Use this, not an object ndarray, for family references: an
    ndarray is invisible to the cycle collector, so owner -> array -> child ->
    parents -> owner could never be freed. A tuple is collected, deep-copied
    through the memo, and pickled by reference to this module.
    """
    __slots__ = ()


def _array_copies(np, projections, family_map):
    """One memo keeps repeated array identities coherent across the family.

    Object slots which name family members point at their corresponding native
    copies. Other Python payloads remain shallow references; this is not a new
    deep-copy policy for lists, callbacks, resources, or external mobjects.
    Structured object fields and array-to-array cycles use the same rule.
    """
    arrays, pending = {}, []

    def clone(value):
        key = id(value)
        if key in arrays:
            return arrays[key]
        result = np.array(value, copy=True, order="K", subok=False)
        arrays[key] = result
        if value.dtype.hasobject:
            pending.append((value, result))
        return result

    prepared = [(namespace, name, clone(value)) for namespace, name, value in projections]
    # A work list, not Python recursion: an array may contain another array,
    # itself, or a shared chain much deeper than the host recursion limit.
    while pending:
        source, target = pending.pop()
        if source.dtype.names:
            for name in source.dtype.names:
                if source.dtype[name].hasobject:
                    pending.append((source[name], target[name]))
        elif source.dtype.kind == "O":
            for index in np.ndindex(source.shape):
                item = source[index]
                if isinstance(item, np.ndarray):
                    target[index] = clone(item)
                else:
                    target[index] = family_map.get(id(item), item)
    return prepared


def _install_family_preorder(g):
    """Keep identity-preorder semantics without a Python recursion limit.

    The live graph can be a DAG, not just a tree. Check the active path before
    the seen set so a back-edge is still rejected, but shared descendants are
    visited only once. Retain each live child iterator, rather than snapshotting
    or reversing child lists, to preserve authored iteration and exception order.
    """
    Mobject = g.get("_BridgeMobject")
    if Mobject is None:
        return  # A reduced copier table owns its supplied family traversal.
    cycle_error = g["_FamilyCycleError"]

    @wraps(g["_family_preorder"])
    def family_preorder(root):
        result, seen, visiting = [], set(), set()

        def enter(member):
            marker = id(member)
            if marker in visiting:
                raise cycle_error("submobjects would create a family cycle")
            if marker in seen:
                return False
            if not isinstance(member, Mobject):
                raise TypeError("submobjects must be Mobject instances")
            visiting.add(marker)
            seen.add(marker)
            result.append(member)
            return True

        enter(root)
        stack = [(id(root), iter(root.submobjects))]
        while stack:
            marker, children = stack[-1]
            try:
                child = next(children)
            except StopIteration:
                visiting.remove(marker)
                stack.pop()
                continue
            if enter(child):
                stack.append((id(child), iter(child.submobjects)))
        return result

    g["_family_preorder"] = family_preorder


def _install_mobject_restoration(g):
    """Use the existing aligned become protocol in both proxy states.

    The old detached shortcut restored paired nursery roots directly and
    refused any family edit made after save_state. Mobject.become already
    owns family alignment, record/schema validation, named-child remapping
    and owner checks. Restoration must not maintain a second partial copy
    protocol or bypass an authored become override.
    """
    Mobject = g.get("Mobject")
    if Mobject is None:
        return  # Reduced copier-only embedding tables have no public class.

    @wraps(Mobject.restore)
    def restore(self):
        saved_state = getattr(self, "saved_state", None)
        if saved_state is None:
            raise Exception("Trying to restore without having saved")
        self.become(saved_state)
        return self

    Mobject.restore = restore


def install_mobject_copying(native):
    """Complete copying and restoration without replacing public classes."""
    g = vars(native)
    if g.get("_FMN_MOBJECT_COPYING_INSTALLED", False):
        return
    _install_family_preorder(g)
    original, family, np = g["_copy_mobject_graph"], g["_family_preorder"], g["_np"]

    @wraps(original)
    def copy_graph(root, deep, memo=None, detach_bound=False):
        memo = {} if memo is None else memo
        if deep or id(root) in memo:
            # The existing deep path already copies arrays through its global
            # deepcopy memo and owns authored __deepcopy__ dispatch. Never copy
            # those results a second time or override a caller's memo identity.
            return original(root, deep, memo, detach_bound=detach_bound)
        members = family(root)
        # Freeze attribute references, not native record views. Existing members
        # already in the caller's memo must not have their copies overwritten.
        attributes = [(member, tuple((name, value) for name, value in vars(member).items()
                                     if name not in _INTERNAL
                                     and isinstance(value, (np.ndarray, FamilyRefs))))
                      for member in members if id(member) not in memo]
        result = original(root, deep, memo, detach_bound=detach_bound)
        family_map = {id(member): memo[id(member)] for member in members}
        projections = [(vars(family_map[id(member)]), name, value)
                       for member, values in attributes for name, value in values]
        references = [(namespace, name, FamilyRefs(family_map.get(id(item), item) for item in value))
                      for namespace, name, value in projections if isinstance(value, FamilyRefs)]
        arrays = [projection for projection in projections if not isinstance(projection[2], FamilyRefs)]
        # Finish every array before publishing attributes onto the copy. The
        # source dictionaries are never cleared, stashed or temporarily edited.
        for namespace, name, value in [*_array_copies(np, arrays, family_map), *references]:
            namespace[name] = value
        return result

    _install_mobject_restoration(g)
    g["_copy_mobject_graph"] = copy_graph
    g["_FMN_MOBJECT_COPYING_INSTALLED"] = True
