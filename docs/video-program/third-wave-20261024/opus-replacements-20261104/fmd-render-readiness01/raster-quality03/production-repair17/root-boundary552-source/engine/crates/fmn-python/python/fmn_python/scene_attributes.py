"""Structural Python state accompanying the native SceneState checkpoint.

Copy owned containers and arrays, not executable host objects or native proxy
identities. One memo spans the entire captured family: cross-object references,
shared containers and cycles survive without cloning a second native arena.
"""
from __future__ import annotations

import struct

# These are projections/owners, not user data. SceneState restores them through
# the native checkpoint and its existing topology/updater/camera protocols.
_PROJECTIONS = frozenset({
    "data", "submobjects", "uniforms", "updaters", "parents", "_scene", "_core",
    "locked_data_keys", "const_data_keys", "locked_uniform_keys", "event_listners",
    "_fmn_persistent_controller",
})
_MAX_NODES = 262_144
_MAX_BYTES = 64 * 1024 * 1024
_MAX_DEPTH = 128


def owned(name):
    return (isinstance(name, str) and name not in _PROJECTIONS
            and not (name.startswith("_fmn_") and name.endswith("_busy")))


class _Copier:
    def __init__(self, np):
        self.np, self.memo, self.retained = np, {}, []
        self.nodes, self.bytes = 0, 0

    def copy(self, value, depth=0):
        key = id(value)
        if key in self.memo:
            return self.memo[key]
        self.nodes += 1
        if self.nodes > _MAX_NODES or depth > _MAX_DEPTH:
            raise ValueError("SceneState Python data exceeds its structural budget")
        kind, np = type(value), self.np
        if isinstance(value, np.ndarray):
            self.bytes += value.nbytes
            if self.bytes > _MAX_BYTES:
                raise ValueError("SceneState Python arrays exceed the 64 MiB budget")
            # Strip ndarray subclasses; never run an authored copy/deepcopy.
            result = np.array(value, copy=True, subok=False, order="K")
            self.memo[key] = result
            self.retained.append(value)
            if value.dtype.hasobject:
                self._objects(value, result, depth + 1)
            result.setflags(write=bool(value.flags.writeable))
            return result
        if kind is dict:
            result = {}
            self.memo[key] = result
            for name, item in value.items():
                result[self.copy(name, depth + 1)] = self.copy(item, depth + 1)
        elif kind is list:
            result = []
            self.memo[key] = result
            result.extend(self.copy(item, depth + 1) for item in value)
        elif kind is tuple:
            items = [self.copy(item, depth + 1) for item in value]
            # A tuple -> list -> tuple cycle may have installed the tuple
            # while cloning one of its mutable descendants.
            result = self.memo.get(key, value if all(a is b for a, b in zip(value, items))
                                   else tuple(items))
        elif kind is set:
            result = value.copy()
        elif kind is bytearray:
            self.bytes += len(value)
            if self.bytes > _MAX_BYTES:
                raise ValueError("SceneState Python buffers exceed the 64 MiB budget")
            result = value.copy()
        elif isinstance(value, np.void) and value.dtype.hasobject:
            holder = self.copy(np.asarray(value), depth + 1)
            result = holder[()]
        else:
            # Mobjects, functions, bound methods, modules, and opaque external
            # objects retain identity. Checkpoints do not rewind their state.
            return value
        self.memo[key] = result
        self.retained.append(value)
        return result

    def _objects(self, source, target, depth):
        if source.dtype.names:
            for name in source.dtype.names:
                if source.dtype[name].hasobject:
                    self._objects(source[name], target[name], depth + 1)
        elif source.dtype.kind == "O":
            for index in self.np.ndindex(source.shape):
                target[index] = self.copy(source[index], depth + 1)


def capture(members, np):
    """Capture every member dictionary with one cross-family alias memo."""
    source = {id(member): {key: value for key, value in vars(member).items() if owned(key)}
              for member in members}
    return _Copier(np).copy(source)


def prepare_restore(snapshot, np, members=None):
    """Validate and allocate the projection before any native mutation."""
    if members is not None:
        if (type(snapshot) is not dict or set(snapshot) != {id(member) for member in members}
                or any(type(values) is not dict or any(not owned(key) for key in values)
                       for values in snapshot.values())):
            raise ValueError("SceneState Python projection does not match its captured family")
    return _Copier(np).copy(snapshot)


def restore(members, prepared):
    for member in members:
        namespace = vars(member)
        saved = prepared[id(member)]
        for key in tuple(namespace):
            if owned(key) and key not in saved:
                del namespace[key]
        # Bypass authored setters: this is state restoration, not a new edit.
        namespace.update(saved)


def same(np, left, right):
    """Compare data and alias topology without calling opaque __eq__ hooks."""
    forward, backward, retained = {}, {}, []

    def equal(a, b):
        if type(a) is not type(b):
            return False
        kind = type(a)
        structural = (kind in (dict, list, tuple, set, bytearray)
                      or isinstance(a, np.ndarray))
        if structural:
            x, y = id(a), id(b)
            if x in forward:
                return forward[x] == y
            if y in backward:
                return False
            forward[x], backward[y] = y, x
            retained.append((a, b))
        if a is b:
            return True
        if isinstance(a, np.ndarray):
            if (a.shape != b.shape or a.dtype != b.dtype
                    or a.flags.writeable != b.flags.writeable):
                return False
            if a.dtype.names:
                return all(equal(a[name], b[name]) for name in a.dtype.names)
            if a.dtype.hasobject:
                return all(equal(x, y) for x, y in zip(a.flat, b.flat))
            # Signed zero and NaN payloads may affect subsequent computation.
            return a.tobytes() == b.tobytes()
        if isinstance(a, np.generic):
            if a.dtype != b.dtype:
                return False
            if a.dtype.names:
                return all(equal(a[name], b[name]) for name in a.dtype.names)
            return a.tobytes() == b.tobytes()
        if kind is dict:
            return (len(a) == len(b)
                    and all(equal(ak, bk) and equal(av, bv)
                            for (ak, av), (bk, bv) in zip(a.items(), b.items())))
        if kind in (tuple, list):
            return len(a) == len(b) and all(equal(x, y) for x, y in zip(a, b))
        if kind is float:
            return struct.pack("!d", a) == struct.pack("!d", b)
        if kind is complex:
            return struct.pack("!dd", a.real, a.imag) == struct.pack("!dd", b.real, b.imag)
        if kind in (str, bytes, bytearray, bool, int, type(None)):
            return a == b
        if kind in (set, frozenset):
            # Only immutable/hashable values can be members. Canonical keys
            # avoid both arbitrary __eq__ and quadratic candidate searches.
            # Count keys: distinct NaNs can coexist even with identical bits.
            def counts(values):
                result = {}
                for value in values:
                    key = _set_key(np, value)
                    result[key] = result.get(key, 0) + 1
                return result
            return counts(a) == counts(b)
        return False

    return equal(left, right)


def _set_key(np, value):
    kind = type(value)
    if kind in (str, bytes, bool, int, type(None)):
        return (kind.__name__, value)
    if kind is float:
        return ("float", struct.pack("!d", value))
    if kind is complex:
        return ("complex", struct.pack("!dd", value.real, value.imag))
    if kind is tuple:
        return ("tuple", tuple(_set_key(np, item) for item in value))
    if kind is frozenset:
        counts = {}
        for item in value:
            key = _set_key(np, item)
            counts[key] = counts.get(key, 0) + 1
        return ("frozenset", frozenset(counts.items()))
    if isinstance(value, np.generic) and not value.dtype.hasobject:
        return ("numpy", str(value.dtype), value.tobytes())
    return ("identity", id(value))
