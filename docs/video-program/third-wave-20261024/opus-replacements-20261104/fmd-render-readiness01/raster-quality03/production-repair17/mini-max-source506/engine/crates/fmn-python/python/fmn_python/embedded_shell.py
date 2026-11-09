"""Literal IPython/checkpoint-paste editing on explicitly owned host sessions.

The optional IPython shell stays in its host interpreter. Studio's framed
worker stdin/stdout is never repurposed as a terminal, and no Python code is
accepted from an HTTP request. Geometry, clocks and snapshots remain native.
"""
from __future__ import annotations

from collections.abc import Mapping
from contextlib import contextmanager
from contextvars import ContextVar
import importlib
import sys
import threading
from types import ModuleType
import weakref

from .scene_console import SceneConsole, _MAX_CHECKPOINTS, _OWNER, _note, checkpoint
from .scene_console import _STUDIO_WORKER

_SESSION = "_fmn_embedded_scene"
_ACTIVE = ContextVar("fmn_embedded_shell", default=None)
_SHELL_LOCK = threading.RLock()
_MISSING = object()


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _caller_namespace():
    # Copy values, never retain an inspect frame/traceback cycle. Find the
    # authored caller rather than relying on a fixed constructor stack depth.
    frame = sys._getframe(1)
    try:
        while frame is not None:
            name = frame.f_globals.get("__name__", "")
            if not (name == __name__ or name.startswith("fmn_python.")
                    or name == "manimlib" or name.startswith("manimlib.")):
                return dict(frame.f_globals) | dict(frame.f_locals)
            frame = frame.f_back
        return {}
    finally:
        del frame


@contextmanager
def _ipython_state(cls, shell=None):
    """Restore IPython singleton slots and exception hook owned by embedding."""
    classes = tuple(cls._walk_mro()) if hasattr(cls, "_walk_mro") else ()
    previous = [(owner, vars(owner).get("_instance", _MISSING)) for owner in classes]
    exception_hook = sys.excepthook
    try:
        if shell is not None:
            for owner in classes:
                owner._instance = shell
        yield
    finally:
        for owner, value in reversed(previous):
            if value is _MISSING:
                if "_instance" in vars(owner):
                    delattr(owner, "_instance")
            else:
                owner._instance = value
        sys.excepthook = exception_hook


def _classes(native):
    module = sys.modules.get("manimlib.scene.scene_embed")
    # The installed package forwards runtime names through __getattr__. Do
    # not mistake its own module dictionary for the extension's class table.
    manager = getattr(native, "CheckpointManager", None) or getattr(module, "CheckpointManager", None)
    embedded = getattr(native, "InteractiveSceneEmbed", None) or getattr(module, "InteractiveSceneEmbed", None)
    if manager is None or embedded is None:
        raise ImportError("native checkpoint/embed classes are absent from their canonical module")
    return manager, embedded


def install_embedded_shell(native):
    """Complete existing class identities after scene and animation adapters."""
    g = vars(native)
    if g.get("_FMN_EMBEDDED_SHELL_INSTALLED", False):
        return
    Scene, InteractiveScene = g["Scene"], g["InteractiveScene"]
    Manager, Embedded = _classes(native)
    capability = g.get("_CapabilityError", RuntimeError)

    def check_manager(self):
        if threading.get_ident() != self._fmn_thread:
            raise RuntimeError("CheckpointManager belongs to its creating thread")

    def manager_init(self):
        self.checkpoint_states = {}
        self._fmn_thread = threading.get_ident()
        self._fmn_scene = None

    def handle_checkpoint_key(self, scene, key):
        check_manager(self)
        if not isinstance(scene, Scene):
            raise TypeError("checkpoint requires a Scene")
        if not isinstance(key, str) or len(key.encode("utf-8")) > 4096:
            raise ValueError("checkpoint key must be text of at most 4096 UTF-8 bytes")
        if self._fmn_scene is not None and self._fmn_scene is not scene:
            raise ValueError("CheckpointManager belongs to another Scene")
        checkpoint(self, scene, key, _MAX_CHECKPOINTS)
        if key:
            self._fmn_scene = scene

    def clear_checkpoints(self):
        check_manager(self)
        self.checkpoint_states.clear()
        self._fmn_scene = None

    def manager_paste(self, shell, scene):
        check_manager(self)
        if not isinstance(scene, Scene):
            raise TypeError("checkpoint requires a Scene")
        owner = vars(scene).get(_OWNER)
        if not isinstance(owner, SceneConsole) or owner.checkpoint_manager is not self:
            raise capability("checkpoint_paste requires its active host SceneConsole or embedded session")
        if shell is not owner.shell:
            raise ValueError("checkpoint_paste shell does not own this Scene console")
        return owner.checkpoint_paste()

    for name, fn in {"__init__": manager_init, "handle_checkpoint_key": handle_checkpoint_key,
                     "clear_checkpoints": clear_checkpoints, "checkpoint_paste": manager_paste}.items():
        _method(Manager, name, fn)

    def initialize(self, scene):
        if not isinstance(scene, Scene):
            raise TypeError("InteractiveSceneEmbed scene must be a Scene")
        self.scene = scene
        self.checkpoint_manager = Manager()
        self.shell = None
        self.clipboard = None
        self._fmn_thread = threading.get_ident()
        self._fmn_namespace = None
        self._fmn_console = None
        self._fmn_hooks = []
        self._fmn_launching = False

    def check(self):
        if threading.get_ident() != self._fmn_thread:
            raise RuntimeError("embedded scene shell belongs to its creating thread")
        if _STUDIO_WORKER.get():
            raise capability("Studio worker stdio belongs to its native protocol; use a host SceneConsole for cells")

    def get_shell(self):
        check(self)
        if not _SHELL_LOCK.acquire(blocking=False):
            raise RuntimeError("another embedded scene shell is active")
        try:
            try:
                module = importlib.import_module("IPython.terminal.embed")
            except ImportError as error:
                raise capability("IPython is not installed in this host; use fmn_python.SceneConsole.run_cell(text)") from error
            namespace = _caller_namespace() if self._fmn_namespace is None else dict(self._fmn_namespace)
            namespace.update(self.get_shortcuts())
            namespace.setdefault("scene", self.scene)
            namespace.setdefault("self", self.scene)
            user_module = ModuleType("_fmn_scene_embed")
            # IPython registers user_module under its __name__. Copying the
            # scene's identity would replace the active SceneSource module in
            # sys.modules. Keep only package/file context for relative imports.
            user_module.__dict__.update({
                key: value for key, value in namespace.items()
                if key not in {"__name__", "__spec__", "__loader__"}
            })
            cls = module.InteractiveShellEmbed
            # Constructing IPython changes sys.excepthook and singleton state.
            # Creating a shell must not replace a notebook's active shell.
            with _ipython_state(cls):
                return cls(user_module=user_module, user_ns=user_module.__dict__, display_banner=False)
        finally:
            _SHELL_LOCK.release()

    original_shortcuts = Embedded.get_shortcuts

    def shortcuts(self):
        values = dict(original_shortcuts(self))
        values["checkpoint_paste"] = self.checkpoint_paste
        values["clear_checkpoints"] = self.checkpoint_manager.clear_checkpoints
        return values

    def ensure_frame_update(self):
        check(self)
        if self.shell is None:
            raise capability("create an IPython shell before installing native post-cell preview")
        if self._fmn_hooks:
            return
        owner = weakref.ref(self)
        shell = self.shell
        def post_cell(*_args, **_kwargs):
            current = owner()
            if current is None or not current._fmn_launching or current.shell is not shell:
                return
            check(current)
            console = current._fmn_console
            # A nested checkpoint-paste run refreshes in its own finally block.
            # Ordinary IPython cells (which bypass run_cell) still get preview.
            if console is not None and not console._closed and not console._busy:
                console._refresh()
        shell.events.register("post_run_cell", post_cell)
        self._fmn_hooks.append((shell.events, "post_run_cell", post_cell))

    def paste(self, skip=False, record=False, progress_bar=True, *, record_to=None, recording_options=None):
        check(self)
        if self._fmn_console is None or not self._fmn_launching:
            raise capability("checkpoint_paste requires an active embedded scene session")
        options = {} if record_to is None and recording_options is None else {
            "record_to": record_to, "recording_options": recording_options,
        }
        return self._fmn_console.checkpoint_paste(skip=skip, record=record, progress_bar=progress_bar, **options)

    def cleanup(self, primary):
        first = None
        self._fmn_launching = False  # Retained/stale callbacks become inert first.
        for events, event, callback in reversed(self._fmn_hooks):
            try:
                events.unregister(event, callback)
            except BaseException as error:
                if first is None:
                    first = error
                if primary is not None:
                    _note(primary, "embedded event cleanup also failed: " + type(error).__name__)
        self._fmn_hooks.clear()
        console, self._fmn_console = self._fmn_console, None
        if console is not None:
            try:
                console.close()
            except BaseException as error:
                if first is None:
                    first = error
                if primary is not None:
                    _note(primary, "embedded console cleanup also failed: " + type(error).__name__)
        if vars(self.scene).get(_SESSION) is self:
            vars(self.scene).pop(_SESSION)
        if first is not None and primary is None:
            raise first

    def launch(self):
        check(self)
        if self._fmn_launching or _ACTIVE.get() is not None:
            raise RuntimeError("cannot nest an active embedded scene shell")
        if not _SHELL_LOCK.acquire(blocking=False):
            raise RuntimeError("another embedded scene shell is active")
        token = _ACTIVE.set(self)
        primary = None
        try:
            # Acquire Scene ownership before importing IPython or creating any
            # terminal resources. A refused launch must not steal a renderer's
            # ownership or clear a manually prepared checkpoint manager.
            console = SceneConsole(self.scene, clipboard=self.clipboard, _native=native)
            self._fmn_console = console
            console.__enter__()
            if self.shell is None:
                self.shell = self.get_ipython_shell_for_embedded_scene()
            if not callable(self.shell) or not callable(getattr(self.shell, "run_cell", None)):
                raise TypeError("embedded shell must be callable and implement run_cell")
            if not isinstance(getattr(self.shell, "user_ns", None), dict):
                raise TypeError("embedded shell.user_ns must be a dictionary")
            if getattr(self.shell, "user_module", None) is None:
                raise TypeError("embedded shell must expose its user_module")
            console.shell = self.shell
            # IPython/caller bindings take precedence over the default aliases;
            # supplying an explicit self or scene must not lose that reference.
            console.namespace = dict(self.shell.user_ns)
            console.namespace.setdefault("scene", self.scene)
            console.namespace.setdefault("self", self.scene)
            console._bind_shell()
            console.checkpoint_manager = self.checkpoint_manager
            self.shell.user_ns.update(self.get_shortcuts())
            vars(self.scene)[_SESSION] = self
            self._fmn_launching = True
            self.ensure_frame_update_post_cell()
            with _ipython_state(type(self.shell), self.shell):
                self.shell(local_ns=self.shell.user_ns, module=self.shell.user_module)
        except BaseException as error:
            primary = error
            raise
        finally:
            try:
                cleanup(self, primary)
            finally:
                _ACTIVE.reset(token)
                _SHELL_LOCK.release()
        return self.shell

    for name, fn in {"__init__": initialize, "launch": launch,
                     "get_ipython_shell_for_embedded_scene": get_shell,
                     "get_shortcuts": shortcuts,
                     "ensure_frame_update_post_cell": ensure_frame_update,
                     "checkpoint_paste": paste}.items():
        _method(Embedded, name, fn)

    def portal_embed(scene=None, namespace=None):
        if scene is None or not isinstance(scene, Scene):
            raise TypeError("scene embedding requires a Scene")
        if namespace is not None and not isinstance(namespace, Mapping):
            raise TypeError("embedded scene namespace must be a mapping")
        embedded = Embedded(scene)
        embedded._fmn_namespace = _caller_namespace() if namespace is None else dict(namespace)
        return embedded.launch()

    def interactive_embed(self, namespace=None):
        return portal_embed(self, namespace)

    def interactive_paste(self):
        embedded = vars(self).get(_SESSION)
        if embedded is not None:
            return embedded.checkpoint_paste()
        owner = vars(self).get(_OWNER)
        if isinstance(owner, SceneConsole):
            return owner.checkpoint_paste()
        raise capability("checkpoint_paste requires an active host scene console; use _checkpoint_bytes() for snapshots")

    _method(InteractiveScene, "embed", interactive_embed)
    _method(InteractiveScene, "checkpoint_paste", interactive_paste)
    g["_portal_embed"] = portal_embed
    g["_portal_checkpoint_paste"] = interactive_paste
    g["_FMN_EMBEDDED_SHELL_INSTALLED"] = True


def embed_scene(scene, namespace=None, *, clipboard=None):
    """Launch the host IPython editor with an optional explicit clipboard reader.

    IPython is optional and loaded only at launch. This never launches another
    interpreter, changes a notebook's active shell, or grants worker/HTTP code
    execution. The returned shell retains the interactive namespace.
    """
    if namespace is not None and not isinstance(namespace, Mapping):
        raise TypeError("embedded scene namespace must be a mapping")
    if clipboard is not None and not callable(clipboard):
        raise TypeError("embedded scene clipboard must be callable or None")
    native = importlib.import_module("manimlib")
    _, Embedded = _classes(native)
    embedded = Embedded(scene)
    embedded._fmn_namespace = _caller_namespace() if namespace is None else dict(namespace)
    embedded.clipboard = clipboard
    return embedded.launch()
