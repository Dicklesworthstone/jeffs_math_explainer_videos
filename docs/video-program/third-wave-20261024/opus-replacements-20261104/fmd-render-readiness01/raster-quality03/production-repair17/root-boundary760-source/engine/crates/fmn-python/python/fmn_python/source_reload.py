"""Source-only live reload through SceneSource's existing import boundary.

Reload replaces project module bindings, not a live Scene or its native arena.
Failed imports restore the previous module graph. Arbitrary authored effects
(files, sockets, external mutable objects) cannot be rolled back. Nothing here
reloads the engine, NumPy, stdlib, or a Studio worker's declared input table.
"""
from __future__ import annotations

import hashlib
import importlib
from pathlib import Path
import sys
import threading
from types import ModuleType

from .scene_loading import SceneSource, _MISSING, _ProjectFinder

_MAX_INPUTS = 4096
_MAX_BYTES = 64 * 1024 * 1024


class _ReloadInputs:
    """Budgeted immutable compilation inputs for one reload attempt."""
    def __init__(self, owner):
        self.owner = owner
        self.sources = {}
        self.size = 0
        self.parents = {}

    def check_thread(self):
        if threading.get_ident() != self.owner._owner_thread:
            raise RuntimeError("scene source reload belongs to its owning thread")

    def read(self, path):
        self.check_thread()
        if path in self.sources:
            source = self.sources[path]
            if source is None:
                raise FileNotFoundError(f"scene source disappeared before reload: {path}")
            return source
        if len(self.sources) >= _MAX_INPUTS:
            raise ValueError("scene reload exceeds its source count budget")
        try:
            with path.open("rb") as stream:
                source = stream.read(_MAX_BYTES - self.size + 1)
        except FileNotFoundError:
            self.sources[path] = None
            raise
        if len(source) > _MAX_BYTES - self.size:
            raise ValueError("scene reload exceeds its source byte budget")
        # Match SourceFileLoader.source_to_code: honor coding cookies and do
        # not inherit compiler future flags from this adapter.
        compile(source, str(path), "exec", dont_inherit=True)
        self.size += len(source)
        self.sources[path] = source
        return source

    def remember_parent(self, fullname):
        self.check_thread()
        parent_name, _, leaf = fullname.rpartition(".")
        parent = sys.modules.get(parent_name)
        if isinstance(parent, ModuleType):
            key = (id(parent), leaf)
            if key not in self.parents:
                self.parents[key] = (parent, leaf, vars(parent).get(leaf, _MISSING))

    def restore_parents(self):
        for parent, leaf, value in reversed(tuple(self.parents.values())):
            if value is _MISSING:
                vars(parent).pop(leaf, None)
            else:
                vars(parent)[leaf] = value


def active_source(path: str | Path | None = None) -> SceneSource | None:
    """Find the existing scoped project owner without acquiring another one."""
    path = None if path is None else Path(path).resolve()
    for finder in tuple(sys.meta_path):
        if isinstance(finder, _ProjectFinder) and finder.owner._active:
            owner = finder.owner
            if path is None or owner.path == path:
                if threading.get_ident() != owner._owner_thread:
                    raise RuntimeError("scene source belongs to another thread")
                return owner
    return None


def reload_source(owner: SceneSource, *, if_changed: bool = False, _prepare=None) -> ModuleType:
    """Reload a live project once; unchanged conditional reload returns its module.

    All previously observed source files are syntax-checked before any module
    is replaced. Newly imported helpers use the same fresh loader and budget.
    Deleted helpers are allowed only when the new project no longer imports
    them. Module identity is fresh on success; existing objects keep their old
    class/function identities until the author deliberately replaces them.
    """
    # The scene-project front door validates a candidate scene before the
    # import generation commits. Lazy imports during construction participate
    # in the same rollback and compilation-input budget as module execution.
    if _prepare is not None and not callable(_prepare):
        raise TypeError("source reload preparation must be callable")
    if not isinstance(if_changed, bool):
        raise TypeError("if_changed must be bool")
    if not owner._active or owner.module is None or active_source(owner.path) is not owner:
        raise RuntimeError("reload requires its active SceneSource context")
    if owner._reload_inputs is not None:
        raise RuntimeError("scene source reload is already in progress")
    if owner._source_inputs is not None:
        raise RuntimeError("declared Studio sources must reload through their worker supervisor")
    candidate = SceneSource(owner.path, owner.scene_type)
    if (candidate.name, candidate.root) != (owner.name, owner.root):
        raise ImportError("scene package structure changed; open a new SceneSource context")
    owner._check_collisions()
    inputs = _ReloadInputs(owner)
    old_digests = dict(owner.source_digests)
    inputs.read(owner.path)
    for path in sorted(old_digests):
        if path == owner.path:
            continue
        try:
            inputs.read(path)
        except FileNotFoundError:
            # Import resolution, not a stale prior dependency list, decides
            # whether a removed helper is still required by the edited source.
            pass
    changed = any(data is None or hashlib.sha256(data).hexdigest() != old_digests.get(path)
                  for path, data in inputs.sources.items())
    loaded = owner._loaded
    previous = {name: sys.modules.get(name, _MISSING) for name in loaded}
    for name, module in loaded.items():
        current = previous[name]
        if current is not _MISSING and current is not module:
            raise ImportError(f"scene module {name!r} was replaced outside its source owner")
    if if_changed and not changed:
        return owner.module
    old_module, old_scenes = owner.module, owner.scenes
    owner._reload_inputs = inputs
    owner._loaded = {}
    owner.source_digests = {}
    try:
        for name, module in loaded.items():
            if sys.modules.get(name) is module:
                sys.modules.pop(name)
        importlib.invalidate_caches()
        module = owner._load()
        scenes = {
            name: value for name, value in vars(module).items()
            if isinstance(value, type) and value is not owner.scene_type
            and issubclass(value, owner.scene_type) and value.__module__ == owner.name
        }
        owner.module, owner.scenes = module, scenes
        if _prepare is not None:
            _prepare(module)
        return module
    except BaseException:
        for name, module in owner._loaded.items():
            if sys.modules.get(name) is module:
                sys.modules.pop(name, None)
        for name, module in previous.items():
            if module is not _MISSING:
                sys.modules[name] = module
        inputs.restore_parents()
        owner._loaded = loaded
        owner.source_digests = old_digests
        owner.module, owner.scenes = old_module, old_scenes
        # _record_source's mixed-version refusal is deliberately NOT rolled
        # back: an unsuccessful reload may have executed real authored effects.
        raise
    finally:
        owner._reload_inputs = None
