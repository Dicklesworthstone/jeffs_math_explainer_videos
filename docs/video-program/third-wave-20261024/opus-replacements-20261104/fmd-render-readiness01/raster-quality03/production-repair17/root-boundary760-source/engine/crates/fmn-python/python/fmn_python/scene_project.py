"""Reconstruct file-backed native scenes without sacrificing a working generation.

One SceneSource owns imports, Scene.run owns lifecycle and the rational clock,
and Camera owns preview capture. The previous scene/module/preview are replaced
only after all three succeed. This is a host authoring boundary, not a sandbox:
arbitrary import/constructor effects and already-published files are not undone.
"""
from __future__ import annotations

from collections.abc import Callable, Iterable, Iterator, Mapping
from contextvars import ContextVar
from dataclasses import dataclass
import importlib
import inspect
from pathlib import Path
import threading
from types import MappingProxyType, ModuleType
from typing import Any

from .scene_loading import SceneSource
from .source_reload import active_source, reload_source

_BUILDING = ContextVar("fmn_building_scene_project", default=None)
_PROJECT = "_fmn_scene_project"


def _idle(scene, *, console=None):
    attrs = vars(scene)
    for key in ("_fmn_scene_execution", "_fmn_owned_render_session", "_fmn_studio_worker_request"):
        if attrs.get(key) is not None:
            raise RuntimeError("finish active scene execution/output before rebuilding its project")
    owner = attrs.get("_fmn_scene_console")
    if owner is not None and (owner is not console or owner._busy):
        raise RuntimeError("cannot rebuild while another console or checkpoint cell owns the scene")


@dataclass(frozen=True)
class _Generation:
    number: int
    scene: Any
    module: ModuleType
    preview: Any
    namespace: Mapping


class SceneProject:
    """An explicitly owned recipe for fresh Scene instances from edited sources.

    Enter the context to import, instantiate and run the selected local class.
    ``rebuild()`` resets native scene state by constructing a NEW instance; old
    references continue to refer to the old instance. ``if_changed=True`` skips
    reconstruction when all observed source bytes are unchanged. Changes made
    through a separate definition reload still cause the next rebuild to run.

    Syntax, import, construction, lifecycle and preview failures preserve the
    last successful generation and its project imports. Authored effects outside
    that generation (including shared mutable constructor arguments) are real
    effects and cannot be rolled back. No output file is opened by this class.

    An active SceneSource may be supplied instead of a filename. Such a source
    remains owned by its caller and is never closed by this project. Declared
    Studio-worker sources must continue to rebuild through their supervisor.
    """

    def __init__(self, source: str | Path | SceneSource, scene: str, *,
                 scene_kwargs: Mapping[str, Any] | None = None, capture: bool = True,
                 _native: Any = None):
        if not isinstance(scene, str) or not scene.isidentifier() or len(scene) > 512:
            raise ValueError("SceneProject requires an explicit local Scene class name")
        if scene_kwargs is not None and not isinstance(scene_kwargs, Mapping):
            raise TypeError("scene_kwargs must be a mapping")
        if type(capture) is not bool:
            raise TypeError("capture must be bool")
        self._kwargs = dict(scene_kwargs or {})
        if any(not isinstance(key, str) for key in self._kwargs):
            raise TypeError("scene_kwargs keys must be strings")
        self._native = importlib.import_module("manimlib") if _native is None else _native
        self._owns_source = not isinstance(source, SceneSource)
        self._source = SceneSource(source, self._native.Scene) if self._owns_source else source
        if self._source.scene_type is not self._native.Scene:
            raise TypeError("SceneSource must belong to the project's native Scene class")
        if self._source._source_inputs is not None:
            raise RuntimeError("declared Studio sources must rebuild through their worker supervisor")
        self.scene_name = scene
        self.capture = capture
        self._thread = threading.get_ident()
        self._state = "new"
        self._busy = False
        self._current: _Generation | None = None
        self._candidate_scene = None
        self._candidate_locals = None
        self._editor = None

    def _check_thread(self):
        if threading.get_ident() != self._thread:
            raise RuntimeError("SceneProject belongs to its creating thread")

    def _check(self):
        self._check_thread()
        if self._state != "active":
            raise RuntimeError("SceneProject requires its active context")
        if self._busy:
            raise RuntimeError("scene project reconstruction is already in progress")
        if active_source(self._source.path) is not self._source:
            raise RuntimeError("SceneProject no longer owns its source context")

    @property
    def scene(self):
        self._check_thread()
        if self._current is None:
            raise RuntimeError("SceneProject has no successfully built scene")
        return self._current.scene

    @property
    def generation(self) -> int:
        self._check_thread()
        return 0 if self._current is None else self._current.number

    @property
    def module(self) -> ModuleType:
        self._check_thread()
        if self._current is None:
            raise RuntimeError("SceneProject has no successfully built module")
        return self._current.module

    @property
    def namespace(self) -> dict[str, Any]:
        self._check_thread()
        if self._current is None:
            raise RuntimeError("SceneProject has no successfully built namespace")
        return dict(self._current.namespace)

    @property
    def preview(self):
        """Last successful native snapshot, or None with capture=False."""
        self._check_thread()
        return None if self._current is None else self._current.preview

    def _build(self, module):
        cls = vars(module).get(self.scene_name)
        if (not isinstance(cls, type) or cls is self._native.Scene
                or not issubclass(cls, self._native.Scene) or cls.__module__ != module.__name__):
            raise ValueError(f"scene {self.scene_name!r} was not declared by {self._source.path}")
        scene = cls(**dict(self._kwargs))
        if not isinstance(scene, self._native.Scene):
            raise TypeError("scene construction must return a Scene")
        if self._current is not None and scene is self._current.scene:
            raise RuntimeError("scene reconstruction must return a fresh instance")
        _idle(scene)
        self._candidate_scene = scene
        self._candidate_locals = None
        token = _BUILDING.set(self)
        try:
            try:
                result = scene.run()
                if inspect.isawaitable(result):
                    if inspect.iscoroutine(result):
                        result.close()
                    raise TypeError("Scene.run must complete synchronously")
            except self._native.EndScene:
                pass
            _idle(scene)
            preview = scene.camera.capture_snapshot(*tuple(scene.mobjects)) if self.capture else None
            namespace = dict(vars(module))
            namespace.update(self._candidate_locals or {})
            namespace.update(scene=scene, self=scene)
            return _Generation(self.generation + 1, scene, module, preview, MappingProxyType(namespace))
        finally:
            _BUILDING.reset(token)
            self._candidate_scene = self._candidate_locals = None

    def __enter__(self):
        self._check_thread()
        if self._state != "new":
            raise RuntimeError("a SceneProject can be entered only once")
        self._state = "entering"
        self._busy = True
        opened = False
        token = _BUILDING.set(self)
        try:
            if self._owns_source:
                self._source.__enter__()
                opened = True
            elif active_source(self._source.path) is not self._source:
                raise RuntimeError("supplied SceneSource must already be active")
            self._current = self._build(self._source.module)
            self._state = "active"
            return self
        except BaseException:
            self._state = "closed"
            if opened:
                self._source.__exit__(None, None, None)
            raise
        finally:
            _BUILDING.reset(token)
            self._busy = False

    def rebuild(self, *, if_changed: bool = False):
        """Prepare a fresh scene and preview before replacing the current pair."""
        self._check()
        if type(if_changed) is not bool:
            raise TypeError("if_changed must be bool")
        if self._editor is not None:
            raise RuntimeError("use the active editor's reload() shortcut to rebuild")
        _idle(self.scene)
        return self._rebuild(if_changed=if_changed)

    def _rebuild(self, *, if_changed=False, prepare=None):
        previous = self._current
        self._busy = True
        token = _BUILDING.set(self)
        def build(module):
            candidate = self._build(module)
            if prepare is not None:
                prepare(candidate)
            self._current = candidate
        try:
            reload_source(self._source,
                          if_changed=if_changed and self._source.module is previous.module,
                          _prepare=build)
            return self.scene
        except BaseException:
            self._current = previous
            raise
        finally:
            _BUILDING.reset(token)
            self._busy = False

    def watch(
        self, *, poll_interval: float = 0.25, debounce: float = 0.1,
        stop: threading.Event | None = None,
        on_error: Callable[[Exception], Any] | None = None,
        paths: Iterable[str | Path] = (),
    ) -> Iterator[Any]:
        """Yield the current scene and successful file-change rebuilds.

        Advance the iterator inside this context, on its owning thread. Source
        edits (including new local helpers) are content-hashed and debounced.
        Add non-Python assets with ``paths``. Each new yield has a matching
        ``preview`` and ``generation`` from the existing rebuild transaction.
        Pass ``on_error`` to report a failed build and continue watching while
        preserving the previous scene; by default failures propagate. Setting
        the optional ``stop`` Event interrupts polling without another rebuild.
        This host iterator neither starts a thread nor replaces Studio's worker
        supervisor or an active IPython editor's reload shortcut.
        """
        self._check()
        from .project_watch import watch_project
        return watch_project(self, poll_interval=poll_interval, debounce=debounce,
                             stop=stop, on_error=on_error, paths=paths)

    def edit(self, *, clipboard=None, auto_rebuild: bool = False,
             debounce: float = 0.0, paths: Iterable[str | Path] = (),
             idle: bool | None = None, poll_interval: float = 0.25,
             preview_protocol: str | None = None):
        """Edit in one IPython shell with optional live rebuilding and preview.

        With ``auto_rebuild=True``, changed sources and explicit asset ``paths``
        reconstruct the full scene on its owning thread. ``idle=None`` enables
        idle polling on a compatible terminal prompt, retaining pre-cell
        rebuilding for simple prompts or existing GUI/asyncio integrations.
        ``idle=True`` requires that capability; ``idle=False`` is pre-cell only.
        ``poll_interval`` is 0.01..60 seconds; ``debounce`` coalesces saved edits.

        Select ``preview_protocol="kitty"`` or ``"sixel"`` to display the initial
        and rebuilt native snapshots in a compatible terminal. No protocol is
        guessed. Failed builds retain the working generation; a terminal write
        failure after a successful build does not undo that new generation.
        Unchanged files preserve interactive edits. Watch options require
        ``auto_rebuild=True`` and do not enable definition-only autoreload.
        """
        from .project_editor import edit_project
        return edit_project(self, clipboard=clipboard, auto_rebuild=auto_rebuild,
                            debounce=debounce, paths=paths, idle=idle,
                            poll_interval=poll_interval, preview_protocol=preview_protocol)

    def close(self):
        self._check_thread()
        if self._state == "closed":
            return
        if self._busy or self._editor is not None:
            raise RuntimeError("cannot close a scene project during reconstruction or editing")
        if self._state == "active":
            _idle(self.scene)
            if self._owns_source:
                self._source.__exit__(None, None, None)
        self._current = None
        self._state = "closed"

    def __exit__(self, exc_type, exc_value, traceback):
        try:
            self.close()
        except BaseException as error:
            if exc_value is None:
                raise
            BaseException.add_note(exc_value, "scene project close also failed: " + type(error).__name__)
        return False

    def _repr_png_(self):
        if threading.get_ident() != self._thread or self._busy or self._current is None:
            return None
        snapshot = self._current.preview
        return None if snapshot is None else snapshot._repr_png_()
