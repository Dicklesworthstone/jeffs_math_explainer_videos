"""Content-based, synchronous live reconstruction of an owned SceneProject.

No watcher thread touches Python imports or native scene state. Polling and
reconstruction happen when the host advances the iterator on its owning thread.
"""
from __future__ import annotations

from collections.abc import Callable, Iterable, Iterator
import hashlib
import math
import os
from pathlib import Path
import stat
import threading
import time
from typing import Any, TYPE_CHECKING

from .runtime_paths import RuntimePaths

if TYPE_CHECKING:
    from .scene_project import SceneProject

_MAX_FILES = 4096
_MAX_BYTES = 64 * 1024 * 1024
_IGNORED_DIRECTORIES = {"__pycache__", "node_modules"}
_MISSING = object()


def _timing(value: float, name: str, *, positive: bool) -> float:
    if type(value) not in (int, float):
        raise TypeError(f"{name} must be a finite number")
    try:
        result = float(value)
    except OverflowError as error:
        raise ValueError(f"{name} exceeds its supported range") from error
    if (not math.isfinite(result) or result < 0 or (positive and result == 0)
            or result > threading.TIMEOUT_MAX):
        raise ValueError(f"{name} is outside its supported range")
    return result


def _walk_error(error: OSError) -> None:
    raise error


def _snapshot(project: SceneProject, extra_paths: tuple[Path, ...]) -> tuple[tuple[Path, str | None], ...]:
    """Watch local Python additions as well as already imported dependencies.

    Hidden/cache directories and directory symlinks are not recursively walked.
    Explicit files and observed imports can include hidden project locations,
    but runtime installations are never part of the source watch set.
    Hash bytes, not mtimes: editors can preserve timestamps and file sizes.
    """
    source = project._source
    runtime = getattr(source, "_runtime_paths", None) or RuntimePaths(source.root)
    paths = {path for path in (source.path, *source.source_digests, *extra_paths)
             if not runtime.contains(path)}
    directories = 0
    for directory, dirs, files in os.walk(source.root, followlinks=False, onerror=_walk_error):
        if runtime.contains(Path(directory)):
            dirs[:] = []
            continue
        directories += 1
        if directories > _MAX_FILES:
            raise ValueError("scene watch exceeds its directory count budget")
        dirs[:] = sorted(name for name in dirs
                         if not name.startswith(".") and name not in _IGNORED_DIRECTORIES
                         and not runtime.contains(Path(directory) / name)
                         and not (Path(directory) / name / "pyvenv.cfg").is_file())
        paths.update(Path(directory) / name for name in files if name.endswith(".py")
                     and not runtime.contains(Path(directory) / name))
        if len(paths) > _MAX_FILES:
            raise ValueError("scene watch exceeds its file count budget")
    if len(paths) > _MAX_FILES:
        raise ValueError("scene watch exceeds its file count budget")
    fingerprints, remaining = {}, _MAX_BYTES
    for path in sorted(paths):
        try:
            # Resolve file links, but never open a FIFO/device while polling.
            target = path.resolve()
            if runtime.contains(target):
                continue
            before = target.stat()
            if not stat.S_ISREG(before.st_mode):
                raise ValueError(f"scene watch input must be a regular file: {path}")
            flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                     | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0))
            descriptor = os.open(target, flags)
        except FileNotFoundError:
            fingerprints[path] = None
            continue
        try:
            opened = os.fstat(descriptor)
            if not stat.S_ISREG(opened.st_mode):
                raise ValueError(f"scene watch input must be a regular file: {path}")
            stream = os.fdopen(descriptor, "rb")
        except BaseException:
            os.close(descriptor)
            raise
        with stream:
            data = stream.read(remaining + 1)
        if len(data) > remaining:
            raise ValueError("scene watch exceeds its source byte budget")
        remaining -= len(data)
        fingerprints[path] = hashlib.sha256(data).hexdigest()
    return tuple(fingerprints.items())


def watch_project(
    project: SceneProject, *, poll_interval: float = 0.25, debounce: float = 0.1,
    stop: threading.Event | None = None, on_error: Callable[[Exception], Any] | None = None,
    paths: Iterable[os.PathLike[str] | str] = (),
) -> Iterator[Any]:
    """Yield the current scene, then each successfully rebuilt scene generation.

    Call inside the project's context and advance only on its creating thread.
    ``stop`` is an optional threading.Event; waiting is interruptible. ``paths``
    adds explicit asset files to the local Python source/dependency watch set.
    Changes are coalesced until their content has been stable for ``debounce``
    seconds. A first conditional rebuild also catches edits made before watch.

    By default a build failure propagates. With ``on_error(exception)``, retain
    the last working generation and retry only after another content change.
    Newly created helpers can therefore recover a previously missing import.
    Errors from the host callback, KeyboardInterrupt and SystemExit propagate.
    Explicitly close a retained iterator when its host stops consuming it.
    """
    project._check()
    if project._editor is not None:
        raise RuntimeError("use the active editor's reload() shortcut to rebuild")
    poll_interval = _timing(poll_interval, "poll_interval", positive=True)
    debounce = _timing(debounce, "debounce", positive=False)
    if stop is None:
        stop = threading.Event()
    elif not isinstance(stop, threading.Event):
        raise TypeError("stop must be a threading.Event or None")
    if on_error is not None and not callable(on_error):
        raise TypeError("on_error must be callable or None")
    if isinstance(paths, (str, bytes, os.PathLike)):
        raise TypeError("paths must be an iterable of file paths")
    # Freeze the host's cwd; authored construction may change it later.
    extras = []
    for path in paths:
        extras.append(Path(path).absolute())
        if len(extras) > _MAX_FILES:
            raise ValueError("scene watch exceeds its file count budget")
    extra_paths = tuple(extras)
    observed = attempted = _MISSING
    stable_since = 0.0
    initial = True
    scan_error = None
    first = True
    while not stop.is_set():
        project._check()
        if project._editor is not None:
            raise RuntimeError("use the active editor's reload() shortcut to rebuild")
        try:
            current = _snapshot(project, extra_paths)
        except Exception as error:
            if on_error is None:
                raise
            identity = (type(error), str(error))
            if identity != scan_error:
                on_error(error)
            scan_error = identity
            # A scan outage is not proof that pending bytes stayed stable.
            observed = _MISSING
            initial = False
        else:
            if stop.is_set():
                return
            scan_error = None
            now = time.monotonic()
            if current != observed:
                if observed is not _MISSING:
                    initial = False
                observed, stable_since = current, now
            if not first and current != attempted and now - stable_since >= debounce:
                # Mark BEFORE running authored code. A broken scene must not
                # repeatedly execute side effects on every polling interval.
                attempted = current
                generation = project.generation
                conditional, initial = initial, False
                try:
                    project.rebuild(if_changed=conditional)
                except Exception as error:
                    if on_error is None:
                        raise
                    on_error(error)
                else:
                    if project.generation != generation:
                        yield project.scene
                # Do not resnapshot here: edits made during construction must
                # remain different from attempted and trigger another build.
        if first:
            first = False
            if stop.is_set():
                return
            yield project.scene
            continue
        if stop.wait(poll_interval):
            return
