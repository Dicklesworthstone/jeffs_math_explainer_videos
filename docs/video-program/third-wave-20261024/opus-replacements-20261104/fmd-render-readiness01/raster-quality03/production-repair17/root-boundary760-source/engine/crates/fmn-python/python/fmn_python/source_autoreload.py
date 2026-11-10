"""Embedded source autoreload without replacing the live native Scene.

This is definition reload, not full scene reconstruction or worker restart.
Project modules execute once per changed generation through SceneSource. The
shell keeps authored local bindings, scene shortcuts, and its own internals.
"""
from __future__ import annotations

from pathlib import Path
import sys
import threading
import weakref

from .scene_loading import SceneSource
from .source_reload import active_source

_STATE = "_fmn_source_autoreload"
_MISSING = object()
_SHELL_NAMES = frozenset({"self", "scene", "get_ipython", "exit", "quit", "In", "Out",
                          "_", "__", "___", "_i", "_ii", "_iii", "_ih", "_oh", "_dh"})


def _exports(module):
    return {name: value for name, value in vars(module).items()
            if not name.startswith("__") and name not in _SHELL_NAMES}


class SourceNamespace:
    """Merge source generations by identity without erasing interactive locals."""
    def __init__(self, source, namespace, *, protected=(), previous=None):
        self.source = source
        self.namespace = namespace
        self.protected = _SHELL_NAMES | frozenset(protected)
        self.previous = _exports(source.module) if previous is None else dict(previous)
        self.module = source.module

    def publish(self, module):
        values = _exports(module)
        namespace = self.namespace
        # A cell-overridden value wins. Remove deleted source names only while
        # the shell still holds the exact binding we previously published.
        for name, previous in self.previous.items():
            if (name not in values and name not in self.protected
                    and namespace.get(name, _MISSING) is previous):
                namespace.pop(name)
        for name, value in values.items():
            if name in self.protected:
                continue
            current = namespace.get(name, _MISSING)
            if current is _MISSING or current is self.previous.get(name, _MISSING):
                namespace[name] = value
        self.previous, self.module = values, module
        return module

    def refresh(self, *, force=False):
        return self.publish(self.source.reload(if_changed=not force))


class _Controller:
    def __init__(self, embedded, native, worker_context):
        self.embedded = weakref.ref(embedded)
        self.native = native
        self.worker_context = worker_context
        self.shell = embedded.shell
        self.module = self.shell.user_module
        # InteractiveShellEmbed creates a temporary user_ns when its mainloop
        # starts. Bind the active cell namespace on first refresh, not while
        # installing the pre-cell hook before that activation.
        self.namespace = None
        filename = vars(self.module).get("__file__") or self.shell.user_ns.get("__file__")
        if not isinstance(filename, (str, Path)):
            raise native._CapabilityError("source autoreload requires a file-backed scene module")
        self.path = Path(filename).resolve()
        if self.path.suffix.lower() not in (".py", ".pyw"):
            raise native._CapabilityError("source autoreload requires a .py or .pyw scene file")
        self.source = None
        self.owns_source = False
        self.bindings = None
        self.global_bindings = None
        self.closed = False
        self.callback = None
        # Only an active SceneSource knows which names it published. Without
        # one, the shell namespace also holds the caller's locals (a Scene, its
        # mobjects), so the first refresh claims only names that the freshly
        # executed source also defines; see `_first_published`.
        self.previous = None
        existing = active_source(self.path)
        if existing is not None:
            self.previous = _exports(existing.module)

    def check(self):
        embedded = self.embedded()
        if self.closed or embedded is None or not embedded._fmn_launching:
            raise RuntimeError("source reload requires its active embedded scene session")
        if threading.get_ident() != embedded._fmn_thread:
            raise RuntimeError("embedded source reload belongs to its creating thread")
        if self.worker_context.get():
            raise self.native._CapabilityError("Studio worker sources reload through their supervisor, not host cells")
        user_module = self.shell.user_module
        module_dict = getattr(user_module, "__dict__", None)
        underlying = getattr(module_dict, "_module_dict", None)
        if (embedded.shell is not self.shell
                or (user_module is not self.module and underlying is not getattr(self.module, "__dict__", None))
                or (self.namespace is not None and self.shell.user_ns is not self.namespace)):
            raise RuntimeError("source reload cannot switch its owning shell or namespace")
        scene = embedded.scene
        generation = vars(scene).get("_fmn_owned_render_session")
        if getattr(generation, "reproducible", False):
            raise self.native._CapabilityError("source autoreload cannot change a certified render's input closure")
        if vars(scene).get("_fmn_scene_execution") is not None:
            raise RuntimeError("source reload cannot interrupt an executing scene segment")
        return embedded

    def _first_published(self, source, namespace):
        if self.previous is not None:
            return self.previous
        return {name: namespace[name] for name in _exports(source.module) if name in namespace}

    def refresh(self, *, force=False):
        embedded = self.check()
        if self.namespace is None:
            if not isinstance(self.shell.user_ns, dict):
                raise TypeError("source reload requires a dictionary cell namespace")
            self.namespace = self.shell.user_ns
        acquired = False
        if self.source is None:
            source = active_source(self.path)
            if source is None:
                source = SceneSource(self.path, self.native.Scene)
                source.__enter__()
                self.owns_source = acquired = True
            self.source = source
            protected = embedded.get_shortcuts()
            self.bindings = SourceNamespace(source, self.namespace, protected=protected,
                                             previous=self._first_published(source, self.namespace))
            # Embedded cells have separate locals and module globals. Update
            # both source-owned projections, so a function defined in a cell
            # sees refreshed helpers too. Interactive overrides still win.
            if vars(self.module) is not self.namespace:
                self.global_bindings = SourceNamespace(
                    source, vars(self.module), protected=protected,
                    previous=self._first_published(source, vars(self.module)))
        # Initial acquisition already executed the module once. All other
        # refreshes execute at most one generation, regardless of namespace count.
        module = self.source.module if acquired else self.source.reload(if_changed=not force)
        self.bindings.publish(module)
        if self.global_bindings is not None:
            self.global_bindings.publish(module)
        return module

    def close(self):
        if self.closed:
            return
        self.closed = True
        source, self.source = self.source, None
        self.bindings = self.global_bindings = None
        self.previous = None
        self.namespace = self.module = self.shell = self.native = None
        self.callback = None
        if self.owns_source and source is not None:
            source.__exit__(None, None, None)


def install_source_autoreload(native):
    """Install after the host embedded-shell adapter, retaining class aliases."""
    namespace = vars(native)
    if namespace.get("_FMN_SOURCE_AUTORELOAD_INSTALLED", False):
        return
    from .scene_console import _STUDIO_WORKER

    module = sys.modules.get("manimlib.scene.scene_embed")
    Embedded = namespace.get("InteractiveSceneEmbed") or getattr(module, "InteractiveSceneEmbed", None)
    if Embedded is None:
        raise ImportError("native InteractiveSceneEmbed class is absent")
    original_launch = Embedded.launch
    original_shortcuts = Embedded.get_shortcuts
    original_update = Embedded.ensure_frame_update_post_cell

    def controller(self):
        if not self._fmn_launching or self.shell is None:
            raise native._CapabilityError("source reload requires an active embedded scene session")
        state = vars(self).get(_STATE)
        if state is None:
            state = _Controller(self, native, _STUDIO_WORKER)
            state.check()
            vars(self)[_STATE] = state
        return state

    def reload_source(self):
        """Refresh definitions now; preserve this Scene, clock and checkpoints."""
        return controller(self).refresh(force=True)

    def auto_reload(self):
        state = controller(self)
        state.check()
        if state.callback is not None:
            return
        owner = weakref.ref(self)
        def before_cell(*_args, **_kwargs):
            current = owner()
            # Retained callbacks from an exited/replaced shell are inert.
            if current is None or not current._fmn_launching or vars(current).get(_STATE) is not state:
                return
            state.refresh()
        self.shell.events.register("pre_run_cell", before_cell)
        self._fmn_hooks.append((self.shell.events, "pre_run_cell", before_cell))
        state.callback = before_cell

    def shortcuts(self):
        values = dict(original_shortcuts(self))
        values["reload_source"] = self.reload_source
        values["auto_reload"] = self.auto_reload
        # Keep the existing reload() scene-reconstruction refusal. Definition
        # reload is not a success-shaped substitute for that larger operation.
        return values

    def ensure_update(self):
        original_update(self)
        config_reader = namespace.get("_pinned_manim_config")
        config = config_reader() if callable(config_reader) else None
        if getattr(getattr(config, "embed", None), "autoreload", False):
            self.auto_reload()

    def launch(self):
        primary = None
        # A refused nested launch must not close the outer session's source.
        prior = vars(self).get(_STATE)
        try:
            return original_launch(self)
        except BaseException as error:
            primary = error
            raise
        finally:
            state = vars(self).get(_STATE)
            if state is not None and state is not prior:
                vars(self).pop(_STATE)
                try:
                    state.close()
                except BaseException as error:
                    if primary is None:
                        raise
                    BaseException.add_note(primary, "source reload cleanup also failed: " + type(error).__name__)

    for name, function in (("auto_reload", auto_reload), ("reload_source", reload_source),
                           ("get_shortcuts", shortcuts), ("ensure_frame_update_post_cell", ensure_update),
                           ("launch", launch)):
        function.__name__ = name
        function.__qualname__ = Embedded.__qualname__ + "." + name
        function.__module__ = Embedded.__module__
        setattr(Embedded, name, function)
    namespace["_FMN_SOURCE_AUTORELOAD_INSTALLED"] = True
