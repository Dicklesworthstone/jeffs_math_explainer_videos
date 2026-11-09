"""Host-owned, checkpointed editing of an existing native Scene.

This is an execution front door, not a renderer, snapshot format or sandbox.
Cells run on the Scene's owning thread with the host interpreter's authority.
Named checkpoints use the existing SceneState implementation. They restore
scene state, NOT Python globals, files, network effects or published output.
"""
from __future__ import annotations

from collections.abc import Callable, Mapping
from contextlib import contextmanager, nullcontext
from contextvars import ContextVar
import importlib
import inspect
import textwrap
import threading
from typing import Any

_STUDIO_WORKER = ContextVar("fmn_studio_worker_stdio", default=False)
_OWNER = "_fmn_scene_console"
_MAX_SOURCE_BYTES = 1024 * 1024
_MAX_CHECKPOINTS = 64


def _positive(value: Any, name: str, maximum: int) -> int:
    if type(value) is not int or not 1 <= value <= maximum:
        raise ValueError(f"{name} must be an integer in 1..{maximum}")
    return value


def _note(error: BaseException, message: str) -> None:
    try:
        BaseException.add_note(error, message)
    except BaseException:
        pass


def _source(value: str, maximum: int) -> str:
    if not isinstance(value, str):
        raise TypeError("scene cell source must be text")
    try:
        size = len(value.encode("utf-8"))
    except UnicodeError:
        raise ValueError("scene cell source must be valid UTF-8 text") from None
    if size > maximum:
        raise ValueError("scene cell source exceeds its byte budget")
    if "\0" in value:
        raise ValueError("scene cell source must not contain NUL")
    return textwrap.dedent("\n".join(line.rstrip() for line in value.splitlines()))


def leading_comment(source: str) -> str:
    """The Reference key is exactly the first line, when it is a comment."""
    line = source.partition("\n")[0].lstrip()
    return line if line.startswith("#") else ""


def checkpoint(manager: Any, scene: Any, key: str, maximum: int) -> None:
    """Change history only after native snapshot/restore succeeds.

    The dictionary is the existing CheckpointManager's authoritative history;
    no copies of native records, callbacks, camera state or RNG live here.
    """
    if not key:
        return
    states = manager.checkpoint_states
    if key in states:
        scene.restore_state(states[key])
        keys = tuple(states)
        for later in keys[keys.index(key) + 1:]:
            states.pop(later)
    else:
        if len(states) >= maximum:
            raise ValueError("scene console checkpoint budget exhausted; clear checkpoints explicitly")
        state = scene.get_state()
        states[key] = state


def _manager(native: Any) -> Any:
    cls = getattr(native, "CheckpointManager", None)
    if cls is None:
        module = importlib.import_module("manimlib.scene.scene_embed")
        cls = module.CheckpointManager
    return cls()


class SceneConsole:
    """Execute Python cells against one existing Scene using native checkpoints.

    ``namespace`` is shallow-copied; mobject references retain their identity.
    A leading comment keys a checkpoint taken BEFORE the first execution.
    Re-running that key restores it and invalidates later checkpoints. Syntax
    errors do not alter history; runtime errors leave their actual partial
    effects, allowing the next retry to restore the original scene checkpoint.

    ``clipboard`` is an explicitly supplied host callable returning text. No
    clipboard executable or optional package is discovered by this class.
    ``shell`` may be a host-owned IPython-compatible shell; absent one, cells
    are ordinary Python. The default preview uses the real Camera.capture.
    Use a context manager, or explicitly close the console when finished.
    """

    def __init__(
        self, scene: Any, namespace: Mapping[str, Any] | None = None, *,
        clipboard: Callable[[], str] | None = None, shell: Any = None,
        capture: bool = True, max_checkpoints: int = _MAX_CHECKPOINTS,
        max_source_bytes: int = _MAX_SOURCE_BYTES, _native: Any = None,
    ) -> None:
        native = importlib.import_module("manimlib") if _native is None else _native
        if not isinstance(scene, native.Scene):
            raise TypeError("SceneConsole requires a Scene instance")
        if namespace is not None and not isinstance(namespace, Mapping):
            raise TypeError("scene console namespace must be a mapping")
        if clipboard is not None and not callable(clipboard):
            raise TypeError("scene console clipboard must be callable or None")
        if shell is not None and not callable(getattr(shell, "run_cell", None)):
            raise TypeError("scene console shell must implement run_cell")
        if type(capture) is not bool:
            raise TypeError("scene console capture must be bool")
        self.max_checkpoints = _positive(max_checkpoints, "max_checkpoints", 1024)
        self.max_source_bytes = _positive(max_source_bytes, "max_source_bytes", 16 * 1024 * 1024)
        self.scene = scene
        self.namespace = dict(namespace or {})
        self.namespace.setdefault("scene", scene)
        self.namespace.setdefault("self", scene)
        self.checkpoint_manager = _manager(native)
        self.clipboard = clipboard
        self.shell = shell
        self.capture = capture
        self._native = native
        self._thread = threading.get_ident()
        self._closed = False
        self._busy = False
        self._cells = 0
        self._bound_shell = None
        self.last_recording = None

    def _check(self) -> None:
        # Check BEFORE reading an unsendable native proxy on another thread.
        if threading.get_ident() != self._thread:
            raise RuntimeError("a SceneConsole is confined to its creating thread")
        if self._closed:
            raise RuntimeError("scene console is closed")
        if self._busy:
            raise RuntimeError("cannot reenter a running scene cell")

    def __enter__(self) -> SceneConsole:
        self._check()
        attrs = vars(self.scene)
        owner = attrs.get(_OWNER)
        if owner is not None and owner is not self:
            raise RuntimeError("this Scene already has an active console")
        if attrs.get("_fmn_scene_execution") is not None:
            raise RuntimeError("cannot start a scene console during play/wait")
        if attrs.get("_fmn_owned_render_session") is not None:
            raise RuntimeError("cannot checkpoint an active output generation; finish it first")
        if _STUDIO_WORKER.get() or attrs.get("_fmn_studio_worker_request") is not None:
            raise RuntimeError("Studio worker stdio belongs to its native protocol, not an embedded shell")
        attrs[_OWNER] = self
        return self

    @contextmanager
    def _operation(self):
        self.__enter__()
        self._busy = True
        try:
            yield
        finally:
            self._busy = False

    def _bind_shell(self) -> None:
        if self.shell is None:
            return
        namespace = getattr(self.shell, "user_ns", None)
        if namespace is not None:
            if not isinstance(namespace, dict):
                raise TypeError("scene console shell.user_ns must be a dictionary")
            if self._bound_shell is not self.shell:
                namespace.update(self.namespace)
            self.namespace = namespace
        self._bound_shell = self.shell

    def _compile(self, source: str):
        filename = f"<fmn-scene-cell-{self._cells + 1}>"
        if self.shell is not None:
            transform = getattr(self.shell, "transform_cell", None)
            transformed = transform(source) if callable(transform) else source
            if not isinstance(transformed, str):
                raise TypeError("shell.transform_cell must return text")
            if len(transformed.encode("utf-8")) > self.max_source_bytes:
                raise ValueError("transformed scene cell exceeds its byte budget")
            # IPython owns execution, including magics and top-level await.
            # Validate transformed syntax before restoring a scene checkpoint.
            import ast
            compile(transformed, filename, "exec", ast.PyCF_ALLOW_TOP_LEVEL_AWAIT,
                    dont_inherit=True)
            return None
        return compile(source, filename, "exec", dont_inherit=True)

    def _refresh(self) -> None:
        if not self.capture or self.scene.is_window_closing():
            return
        self.scene.update_frame(dt=0, force_draw=True)
        # Native capture is read-only and atomically replaces the last image.
        # It neither advances the clock nor opens/emits a file generation.
        self.scene.camera.capture(*tuple(self.scene.mobjects))

    def preview(self):
        """Return a native PNG-displayable snapshot without executing a cell.

        Unlike a cell's post-execution refresh, this is observational: it does
        not tick updaters, change checkpoints, open output, or advance time.
        It is also available when automatic post-cell capture is disabled.
        """
        with self._operation():
            return self.scene.camera.capture_snapshot(*tuple(self.scene.mobjects))

    def _repr_png_(self):
        # IPython can inspect a console during run_cell. Never reenter a cell,
        # touch an unsendable native Scene on another thread, or resurrect a
        # closed console just to format a representation.
        if (threading.get_ident() != self._thread or self._closed
                or self._busy or not self.capture):
            return None
        return self.scene.camera.get_png()

    def _execute(self, source: str, *, skip: bool, record: bool, progress_bar: bool,
                 record_to=None, recording_options=None):
        for name, value in (("skip", skip), ("record", record), ("progress_bar", progress_bar)):
            if type(value) is not bool:
                raise TypeError(name + " must be bool")
        source = _source(source, self.max_source_bytes)
        self._bind_shell()
        code = self._compile(source)
        key = leading_comment(source)
        recording = None
        if recording_options is not None and record_to is None:
            raise ValueError("recording_options requires record_to")
        if record_to is not None:
            from .recording import RecordingSession
            if recording_options is not None and not isinstance(recording_options, Mapping):
                raise TypeError("recording_options must be a mapping")
            recording = RecordingSession(self.scene, record_to, _native=self._native,
                                         **dict(recording_options or {}))
        # Capability/configuration refusal precedes checkpoint creation/restore.
        # A clip is opened AFTER restore, so retries never rewind a live sink.
        # record=True without an explicit destination still refuses guessing.
        with self.scene.temp_config_change(skip, False if recording else record, progress_bar):
            checkpoint(self.checkpoint_manager, self.scene, key, self.max_checkpoints)
            try:
                with recording if recording is not None else nullcontext():
                    return self._run_compiled(source, code)
            finally:
                if recording is not None and recording.result is not None:
                    self.last_recording = recording.result

    def _run_compiled(self, source, code):
        self._cells += 1
        primary = None
        try:
            if self.shell is None:
                exec(code, self.namespace, self.namespace)
                return None
            result = self.shell.run_cell(source)
            if inspect.isawaitable(result):
                # run_cell is synchronous; never leak a returned coroutine.
                if inspect.iscoroutine(result):
                    result.close()
                raise TypeError("scene console shell.run_cell must be synchronous")
            error = getattr(result, "error_before_exec", None)
            if error is None:
                error = getattr(result, "error_in_exec", None)
            if error is not None:
                if not isinstance(error, BaseException):
                    raise TypeError("shell execution error must be an exception")
                raise error
            return result
        except BaseException as error:
            primary = error
            raise
        finally:
            try:
                self._refresh()
            except BaseException as error:
                if primary is None:
                    raise
                _note(primary, "native cell preview also failed: " + type(error).__name__)

    def run_cell(self, source: str, *, skip: bool = False, record: bool = False,
                 progress_bar: bool = True, record_to=None, recording_options=None):
        """Run a cell, optionally recording its animation to an explicit path.

        recording_options accepts record_scene's format/resolution/fps/threads.
        Read last_recording after success. Retry with a new output path; native
        publication never overwrites an earlier successful clip.
        """
        with self._operation():
            return self._execute(source, skip=skip, record=record, progress_bar=progress_bar,
                                 record_to=record_to, recording_options=recording_options)

    def checkpoint_paste(self, *, skip: bool = False, record: bool = False,
                         progress_bar: bool = True, record_to=None, recording_options=None):
        """Read one explicitly granted clipboard value and run its keyed cell."""
        with self._operation():
            if self.clipboard is None:
                error_type = getattr(self._native, "_CapabilityError", RuntimeError)
                raise error_type("checkpoint_paste requires a host clipboard callable; use run_cell(text) otherwise")
            return self._execute(self.clipboard(), skip=skip, record=record, progress_bar=progress_bar,
                                 record_to=record_to, recording_options=recording_options)

    def clear_checkpoints(self) -> None:
        with self._operation():
            self.checkpoint_manager.clear_checkpoints()

    def close(self) -> None:
        if self._closed and threading.get_ident() == self._thread:
            return
        self._check()
        scene = self.scene
        if vars(scene).get(_OWNER) is self:
            vars(scene).pop(_OWNER)
        try:
            self.checkpoint_manager.clear_checkpoints()
        finally:
            self.namespace = {}
            self._bound_shell = None
            self.shell = self.clipboard = self.scene = None
            self._closed = True

    def __exit__(self, exc_type, exc_value, traceback) -> bool:
        try:
            self.close()
        except BaseException as error:
            if exc_value is None:
                raise
            _note(exc_value, "scene console cleanup also failed: " + type(error).__name__)
        return False
