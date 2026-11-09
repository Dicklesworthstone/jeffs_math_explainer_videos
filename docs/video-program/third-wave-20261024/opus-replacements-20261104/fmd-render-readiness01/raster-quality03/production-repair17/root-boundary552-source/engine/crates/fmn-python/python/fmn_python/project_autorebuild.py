"""Content-triggered scene reconstruction at safe host-editor boundaries.

This is full SceneProject reconstruction, not definition-only autoreload. No
thread, second scene clock or worker is started. A failed generation leaves the
existing project/editor transaction in charge of preserving the working scene.
"""
from __future__ import annotations

import os
from pathlib import Path
import time
import weakref

from .project_watch import _MAX_FILES, _snapshot, _timing

_STATE = "_fmn_project_autorebuild"
_MISSING = object()


def _watch_options(*, debounce=0.0, paths=(), idle=None, poll_interval=0.25, preview_protocol=None):
    """Freeze and validate options before acquiring host editor resources."""
    debounce = _timing(debounce, "debounce", positive=False)
    if preview_protocol is not None:
        from .terminal_preview import _protocol
        _protocol(preview_protocol)
    if idle is not None and type(idle) is not bool:
        raise TypeError("idle must be bool or None")
    poll_interval = _timing(poll_interval, "poll_interval", positive=True)
    if not 0.01 <= poll_interval <= 60.0:
        raise ValueError("poll_interval must be in 0.01..60 seconds")
    if isinstance(paths, (str, bytes, os.PathLike)):
        raise TypeError("paths must be an iterable of file paths")
    extras = []
    for path in paths:
        extras.append(Path(path).absolute())
        if len(extras) > _MAX_FILES:
            raise ValueError("scene watch exceeds its file count budget")
    return {"debounce": debounce, "paths": tuple(extras),
            "idle": idle, "poll_interval": poll_interval, "preview_protocol": preview_protocol}


class _RebuildWatch:
    """One attempted reconstruction per stable content generation."""

    def __init__(self, project, check, rebuild, *, debounce=0.0, paths=(),
                 idle=None, poll_interval=0.25, preview_protocol=None):
        options = _watch_options(debounce=debounce, paths=paths, idle=idle,
                                 poll_interval=poll_interval, preview_protocol=preview_protocol)
        self.debounce, self.paths = options["debounce"], options["paths"]
        self.idle_mode, self.poll_interval = options["idle"], options["poll_interval"]
        self.preview_protocol = options["preview_protocol"]
        self.prompt = None
        self.project, self.check, self.rebuild = project, check, rebuild
        self.observed = self.attempted = _MISSING
        self.stable_since = 0.0
        self.initial = True
        self.scan_error = None
        self.closed = False
        self.busy = False

    def arm(self):
        """Observe activation bytes so edits before the first cell are detected."""
        if self.closed or self.busy:
            raise RuntimeError("cannot arm a closed or busy scene rebuild watch")
        self.observed = _snapshot(self.project, self.paths)
        self.stable_since = time.monotonic()

    def poll(self):
        """Rebuild eligible content once; errors propagate to the host caller.

        Initial polling is conditional on the source generation, so enabling
        the watch does not erase interactive edits in an unchanged scene.
        Subsequent asset changes and new Python helpers force reconstruction.
        """
        if self.closed:
            raise RuntimeError("scene rebuild watch is closed")
        if self.busy:
            raise RuntimeError("scene rebuild watch is already polling")
        self.check()
        self.busy = True
        try:
            try:
                current = _snapshot(self.project, self.paths)
            except Exception as error:
                identity = (type(error), str(error))
                self.observed = _MISSING
                self.initial = False
                if identity == self.scan_error:
                    return False
                self.scan_error = identity
                raise
            self.scan_error = None
            now = time.monotonic()
            if current != self.observed:
                if self.observed is not _MISSING:
                    self.initial = False
                self.observed, self.stable_since = current, now
            if current == self.attempted or now - self.stable_since < self.debounce:
                return False
            # Mark before authored execution. Broken source must not repeatedly
            # perform its import/construction effects on every entered cell.
            self.attempted = current
            conditional, self.initial = self.initial, False
            project = self.project
            generation = project.generation
            self.rebuild(if_changed=conditional)
            # Do not rescan here: edits made DURING construction remain pending
            # and must be observed at the next safe cell boundary.
            changed = project.generation != generation
            if changed and not self.closed and self.preview_protocol is not None:
                from .terminal_preview import _show_snapshot
                try:
                    _show_snapshot(project.preview, self.preview_protocol, None, 16_777_216, project._native)
                except BaseException as error:
                    BaseException.add_note(error, "new scene generation is active; terminal display failed after rebuild")
                    raise
            return changed
        finally:
            self.busy = False

    def close(self):
        self.closed = True
        try:
            if self.prompt is not None:
                self.prompt.close()
        finally:
            self.project = self.check = self.rebuild = None


def install_project_autorebuild(native):
    """Extend the installed project editor without changing class identities."""
    if getattr(native, "_FMN_PROJECT_AUTOREBUILD_INSTALLED", False) is True:
        return
    from .embedded_shell import _classes, _method
    from .project_editor import _check_editor
    from .scene_project import _PROJECT
    from .source_autoreload import _STATE as definition_state

    _, Embedded = _classes(native)
    original_launch = Embedded.launch
    original_shortcuts = Embedded.get_shortcuts
    original_auto_reload = Embedded.auto_reload
    original_update = Embedded.ensure_frame_update_post_cell

    def remove_hook(self, callback):
        for index, (events, event, installed) in enumerate(self._fmn_hooks):
            if installed is callback:
                events.unregister(event, installed)
                self._fmn_hooks.pop(index)
                return

    def stop(self):
        state = vars(self).get(_STATE)
        if state is None:
            return
        # Validate ownership before releasing an active host watch. Launch
        # cleanup calls release directly after the shell is no longer active.
        _check_editor(self, vars(self)[_PROJECT])
        release(self, state)

    def release(self, state):
        if vars(self).get(_STATE) is state:
            vars(self).pop(_STATE)
        state.close()  # Retained callbacks are inert before unregistering.
        remove_hook(self, state.callback)

    def auto_rebuild(self, *, debounce=0.0, paths=(), idle=None, poll_interval=0.25, preview_protocol=None):
        project = vars(self).get(_PROJECT)
        if project is None:
            raise native._CapabilityError("auto_rebuild requires an active SceneProject editor")
        _check_editor(self, project)
        owner = weakref.ref(self)

        def check():
            embedded = owner()
            if embedded is None or vars(embedded).get(_STATE) is not state:
                raise RuntimeError("scene rebuild watch no longer owns its editor")
            _check_editor(embedded, project)

        def rebuild(*, if_changed):
            embedded = owner()
            if embedded is None:
                raise RuntimeError("scene rebuild editor no longer exists")
            return embedded.reload_scene(if_changed=if_changed)

        state = _RebuildWatch(project, check, rebuild, debounce=debounce, paths=paths,
                              idle=idle, poll_interval=poll_interval, preview_protocol=preview_protocol)

        def before_cell(*_args, **_kwargs):
            embedded = owner()
            if (embedded is None or not embedded._fmn_launching
                    or vars(embedded).get(_STATE) is not state or state.closed):
                return
            # IPython reports event-hook errors and keeps the previous scene
            # available. The content generation is already marked attempted.
            state.poll()

        from .project_prompt import prepare_prompt
        previous = vars(self).get(_STATE)

        def active():
            embedded = owner()
            return (embedded is not None and embedded._fmn_launching
                    and vars(embedded).get(_STATE) is state and not state.closed)

        try:
            if preview_protocol is not None:
                from .terminal_preview import _validate_project_preview
                _validate_project_preview(project, preview_protocol)
            state.arm()
            state.prompt = prepare_prompt(self.shell, state, active,
                                          replacing=None if previous is None else previous.prompt)
        except BaseException:
            state.close()
            raise
        state.callback = before_cell
        # Validate options before disturbing an existing mode. Definition-only
        # autoreload and full reconstruction must not execute the same source
        # generation twice at a single cell boundary.
        if previous is not None:
            release(self, previous)
        auto = vars(self).get(definition_state)
        if auto is not None and auto.callback is not None:
            remove_hook(self, auto.callback)
            auto.callback = None
        self.shell.events.register("pre_run_cell", before_cell)
        self._fmn_hooks.append((self.shell.events, "pre_run_cell", before_cell))
        vars(self)[_STATE] = state
        try:
            if state.prompt is not None:
                state.prompt.install()
            if preview_protocol is not None:
                from .terminal_preview import _show_snapshot
                _show_snapshot(project.preview, preview_protocol, None, 16_777_216, project._native)
        except BaseException:
            release(self, state)
            raise

    def auto_reload(self):
        # Explicitly switching back to definitions disables full reconstruction.
        stop(self)
        return original_auto_reload(self)

    def shortcuts(self):
        values = dict(original_shortcuts(self))
        if vars(self).get(_PROJECT) is not None:
            values.update(auto_rebuild=self.auto_rebuild, stop_auto_rebuild=self.stop_auto_rebuild)
        return values

    def ensure_update(self):
        result = original_update(self)
        options = vars(self).pop("_fmn_project_autorebuild_options", None)
        if options is not None:
            self.auto_rebuild(**options)
        return result

    def launch(self):
        previous = vars(self).get(_STATE)
        primary = None
        try:
            return original_launch(self)
        except BaseException as error:
            primary = error
            raise
        finally:
            state = vars(self).get(_STATE)
            if state is not None and state is not previous:
                try:
                    release(self, state)
                except BaseException as error:
                    if primary is None:
                        raise
                    BaseException.add_note(primary, "scene watch cleanup also failed: " + type(error).__name__)

    for name, function in (("auto_rebuild", auto_rebuild), ("stop_auto_rebuild", stop),
                           ("auto_reload", auto_reload), ("get_shortcuts", shortcuts), ("launch", launch),
                           ("ensure_frame_update_post_cell", ensure_update)):
        _method(Embedded, name, function)
    native._FMN_PROJECT_AUTOREBUILD_INSTALLED = True
