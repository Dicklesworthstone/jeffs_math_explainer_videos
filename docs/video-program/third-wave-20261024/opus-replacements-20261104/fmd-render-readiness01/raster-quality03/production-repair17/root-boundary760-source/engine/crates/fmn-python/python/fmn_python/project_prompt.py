"""Owner-thread scene reconstruction while a terminal IPython prompt is idle.

Use prompt_toolkit's input-hook boundary, not a background scene thread or a
second clock. The hook belongs to one shell and one watch. Other GUI/asyncio
integrations are left alone; no global IPython input-hook registry is mutated.
"""
from __future__ import annotations

import sys
import threading
import time
import weakref

_MISSING = object()


def prepare_prompt(shell, watch, is_active, *, replacing=None):
    """Prepare an idle hook without changing the currently active watch.

    ``idle=None`` selects idle polling only where supported, retaining ordinary
    pre-cell rebuilding elsewhere. ``idle=True`` requires that capability;
    ``idle=False`` explicitly selects the pre-cell boundary alone.
    """
    if watch.idle_mode is False:
        return None
    previous = getattr(shell, "_inputhook", _MISSING)
    owned = replacing is not None and previous is replacing.callback
    reason = None
    if getattr(shell, "simple_prompt", True) or getattr(shell, "pt_app", None) is None:
        reason = "idle scene rebuilding requires the terminal prompt_toolkit prompt"
    elif getattr(shell, "_use_asyncio_inputhook", False):
        reason = "idle scene rebuilding cannot replace the shell's asyncio integration"
    elif previous is _MISSING or (previous is not None and not owned):
        reason = "idle scene rebuilding cannot replace another GUI input hook"
    if reason is not None:
        if watch.idle_mode is True:
            raise RuntimeError(reason + "; use idle=False for pre-cell rebuilding")
        return None
    return _PromptRebuild(shell, watch, is_active)


class _PromptRebuild:
    def __init__(self, shell, watch, is_active):
        self.shell = weakref.ref(shell)
        self.watch = watch
        self.is_active = is_active
        self.thread = threading.get_ident()
        self.next_poll = 0.0
        self.closed = False
        self.busy = False
        self.installed = False
        self.previous = None
        self.reported = None
        # A stable callable identity permits conditional restoration even
        # though Python creates a new bound-method object on each lookup.
        self.callback = self.run

    def install(self):
        shell = self.shell()
        if self.closed or self.installed or shell is None:
            raise RuntimeError("cannot install an inactive scene prompt hook")
        if getattr(shell, "_inputhook", _MISSING) is not None:
            raise RuntimeError("the shell's input hook changed before scene watch activation")
        self.previous = (shell._inputhook, getattr(shell, "active_eventloop", None))
        shell._inputhook = self.callback
        shell.active_eventloop = "frankenmanim-rebuild"
        self.installed = True

    def current(self):
        shell = self.shell()
        return (not self.closed and self.installed and shell is not None
                and getattr(shell, "_inputhook", None) is self.callback
                and self.is_active())

    def run(self, context):
        if self.closed:
            return
        if threading.get_ident() != self.thread:
            raise RuntimeError("scene prompt rebuilding belongs to its creating thread")
        if self.busy or not self.current():
            return
        self.busy = True
        try:
            while self.current() and not context.input_is_ready():
                now = time.monotonic()
                if now >= self.next_poll:
                    try:
                        changed = self.watch.poll()
                    except Exception as error:
                        # Source/scan failures are already deduplicated by the
                        # content watch. Also bound repeated ownership errors,
                        # without retaining exceptions or native traceback owners.
                        try:
                            message = str(error)[:2048]
                        except Exception:
                            message = "<unprintable error>"
                        identity = (type(error), message)
                        if identity != self.reported:
                            self.reported = identity
                            shell = self.shell()
                            try:
                                shell.showtraceback()
                            except Exception:
                                print("Scene rebuild failed: " + type(error).__name__
                                      + ": " + message, file=sys.stderr)
                    else:
                        self.reported = None
                        if changed and self.current():
                            project = self.watch.project
                            print(f"Rebuilt {project.scene_name} (generation {project.generation}).")
                    # Rate-limit across input-hook invocations and slow builds.
                    # New source edits during a build remain for the next poll.
                    if not self.current():
                        return
                    self.next_poll = time.monotonic() + self.watch.poll_interval
                # Yield to keyboard/window readiness promptly. The selector's
                # own helper handles terminal I/O only; all scene work stays
                # on this thread, including import, preview and error handling.
                if not context.input_is_ready():
                    time.sleep(min(0.02, max(0.0, self.next_poll - time.monotonic())))
        finally:
            self.busy = False

    def close(self):
        if self.closed:
            return
        if threading.get_ident() != self.thread:
            raise RuntimeError("scene prompt rebuilding belongs to its creating thread")
        self.closed = True
        shell = self.shell()
        if self.installed and shell is not None and shell._inputhook is self.callback:
            shell._inputhook, shell.active_eventloop = self.previous
        # Never undo a GUI integration explicitly selected by the user later.
        self.watch = self.is_active = self.previous = self.reported = None
