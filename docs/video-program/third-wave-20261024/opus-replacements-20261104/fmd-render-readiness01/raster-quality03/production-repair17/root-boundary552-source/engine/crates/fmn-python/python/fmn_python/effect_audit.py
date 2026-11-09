"""Scene effects of certified portal renders (plan §15.5 and §16.7 C6).

A certified manifest claims that its output is a function of its input
closure. Python scene code can read any file, start processes or open
sockets, and none of that reaches the closure by itself. While a reproducible
render is active, this module observes CPython's audit events (PEP 578):

- every regular file the scene opens for reading is hashed when it is opened
  and becomes a C6 input named ``read/<sha256>/<basename>``. It is hashed
  again before publication, so an input that changes during the render is
  refused rather than misrecorded;
- effects the closure cannot capture refuse certification: a read of a
  missing or non-regular file, a directory listing, a subprocess, a network
  connection or name lookup, and an SQLite connection.

These are not scene inputs:
- reads by the import system, which SceneSource and the runtime identity
  govern;
- reads inside the interpreter's library directories and the portal package;
- source-line reads for diagnostics;
- the portal's own provenance reads, which run under ``paused()``.

Environment, clock and entropy reads raise no audit events and are not
observed. The hook is installed once, on the first certified render, and
costs one global check per audit event while no render is being recorded.
"""
from __future__ import annotations

import contextlib
import hashlib
import os
import stat
import sys
import threading

from .runtime_paths import library_roots

_LISTINGS = frozenset({"os.listdir", "os.scandir", "glob.glob", "glob.glob/2"})
_PROCESSES = frozenset({
    "subprocess.Popen", "os.system", "os.exec", "os.posix_spawn", "os.spawn",
    "os.fork", "os.forkpty", "os.startfile",
})
_NETWORK = frozenset({
    "socket.connect", "socket.sendto", "socket.sendmsg", "socket.getaddrinfo",
    "socket.gethostbyname", "socket.gethostbyaddr", "socket.getnameinfo", "urllib.Request",
})
_WATCHED = frozenset({"open", "sqlite3.connect"}) | _LISTINGS | _PROCESSES | _NETWORK
_MAX_READS = 4096
_MAX_FILE_BYTES = 512 * 1024 * 1024
_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
_SHOWN_REFUSALS = 4

_lock = threading.RLock()
_local = threading.local()
_active: list[EffectRecorder] = []
_roots: tuple[str, ...] = ()
_installed = False


def _library_roots() -> tuple[str, ...]:
    return tuple(str(root) + os.sep for root in library_roots())


def _install() -> None:
    global _installed, _roots
    with _lock:
        if _installed:
            return
        _roots = _library_roots()
        sys.addaudithook(_hook)
        _installed = True


def _hook(event: str, args: tuple) -> None:
    if not _active or event not in _WATCHED or getattr(_local, "quiet", False):
        return
    _local.quiet = True
    try:
        _observe(event, args, sys._getframe(1))
    except Exception as error:
        # An audit hook must never break the scene's own call. Fail closed:
        # the render cannot be certified if its effects were not recorded.
        _refuse(f"an effect whose audit failed ({event}: {type(error).__name__})")
    finally:
        _local.quiet = False


def _observe(event: str, args: tuple, caller) -> None:
    if caller.f_code.co_filename.startswith("<frozen importlib"):
        return
    if event == "open":
        path, _mode, flags = args
        if isinstance(path, int) or not _reads(flags):
            return
        if caller.f_globals.get("__name__") in ("linecache", "tokenize"):
            return
        name = _absolute(path)
        if not _library(name):
            _read(name)
    elif event in _LISTINGS:
        path = args[0] if args else None
        if isinstance(path, int):
            _refuse("a directory listing by file descriptor")
            return
        name = _absolute("." if path is None else path)
        if not _library(name):
            _refuse("a directory listing: " + name)
    elif event == "sqlite3.connect":
        database = args[0] if args else None
        if database not in (":memory:", b":memory:", ""):
            _refuse("an SQLite connection: " + str(database))
    elif event in _PROCESSES:
        _refuse("a subprocess (" + event + ")")
    else:
        _refuse("a network effect (" + event + ")")


def _reads(flags) -> bool:
    if not isinstance(flags, int):
        return True
    return flags & getattr(os, "O_ACCMODE", 3) != os.O_WRONLY


def _absolute(path) -> str:
    return os.path.abspath(os.fsdecode(os.fspath(path)))


def _library(name: str) -> bool:
    real = os.path.realpath(name)
    return any(real.startswith(root) for root in _roots)


def _digest(name: str) -> tuple[str | None, int, str | None]:
    """Hash one regular file without blocking on FIFOs or devices."""
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)
    try:
        descriptor = os.open(name, flags)
    except FileNotFoundError:
        return None, 0, "a read of a missing file: " + name
    except OSError as error:
        return None, 0, f"a read of an unreadable file ({error.strerror}): {name}"
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            return None, 0, "a read of a file that is not a regular file: " + name
        hasher, size = hashlib.sha256(), 0
        while chunk := os.read(descriptor, 1 << 20):
            size += len(chunk)
            if size > _MAX_FILE_BYTES:
                return None, size, "a read of a file over the 512 MiB input budget: " + name
            hasher.update(chunk)
        return hasher.hexdigest(), size, None
    finally:
        os.close(descriptor)


def _read(name: str) -> None:
    digest, size, problem = _digest(name)
    with _lock:
        for recorder in _active:
            if problem is None:
                recorder._record(name, digest, size)
            else:
                recorder._refuse(problem)


def _refuse(effect: str) -> None:
    with _lock:
        for recorder in _active:
            recorder._refuse(effect)


@contextlib.contextmanager
def paused():
    """Exempt the portal's own provenance reads on this thread."""
    previous = getattr(_local, "quiet", False)
    _local.quiet = True
    try:
        yield
    finally:
        _local.quiet = previous


class EffectRecorder:
    """Scene effects since a certified render, or its CLI invocation, began.

    Mutation happens under the module lock: audit events arrive on whichever
    thread performs them.
    """

    def __init__(self, invocation: bool) -> None:
        self.invocation = invocation
        self._reads: dict[str, str] = {}
        self._refusals: dict[str, None] = {}
        self._refused = 0
        self._bytes = 0

    def _refuse(self, effect: str) -> None:
        self._refused += 1
        if len(self._refusals) < _SHOWN_REFUSALS:
            self._refusals.setdefault(effect)

    def _record(self, name: str, digest: str, size: int) -> None:
        prior = self._reads.get(name)
        if prior is None:
            if len(self._reads) >= _MAX_READS or self._bytes + size > _MAX_TOTAL_BYTES:
                self._refuse("more file reads than the 4096-file, 2 GiB input budget")
                return
            self._bytes += size
            self._reads[name] = digest
        elif prior != digest:
            self._refuse("a file that changed between reads: " + name)

    def stop(self) -> None:
        with _lock:
            if self in _active:
                _active.remove(self)

    def check(self, error_type: type[BaseException]) -> None:
        """Refuse certification if the scene performed an uncapturable effect."""
        with _lock:
            refusals, refused = list(self._refusals), self._refused
        if refused:
            more = refused - len(refusals)
            raise error_type(
                "CAPABILITY: certified output cannot capture " + "; ".join(refusals)
                + (f" (and {more} more effects)" if more > 0 else "")
                + "; render without --reproducible, or read inputs as regular files"
            )

    def inputs(self, error_type: type[BaseException]) -> list[tuple[str, str]]:
        """Re-hash every recorded read and name it as a C6 input.

        Names are content-addressed and carry only the basename, so the same
        inputs give the same closure on every host.
        """
        self.check(error_type)
        with _lock:
            reads = sorted(self._reads.items())
        named = {}
        with paused():
            for name, digest in reads:
                current, _size, _problem = _digest(name)
                if current != digest:
                    with _lock:
                        self._refuse("a file that changed during rendering: " + name)
                    continue
                base = os.path.basename(name)
                named["read/" + digest + "/" + (base if base.isprintable() else "file")] = digest
        self.check(error_type)
        return sorted(named.items())


def begin(*, invocation: bool = False) -> EffectRecorder:
    """Start recording, inheriting what an enclosing certified invocation saw.

    Only an invocation recorder (the certified CLI's, which starts before its
    scene module loads) is inherited: effects at import time are part of every
    render it makes. A render's own recorder is never inherited.
    """
    _install()
    recorder = EffectRecorder(invocation)
    with _lock:
        for outer in _active:
            if outer.invocation:
                for name, digest in outer._reads.items():
                    recorder._reads.setdefault(name, digest)
                recorder._refusals.update(outer._refusals)
                recorder._refused += outer._refused
                recorder._bytes += outer._bytes
        _active.append(recorder)
    return recorder


@contextlib.contextmanager
def recording(enabled: bool = True):
    """Record a certified CLI invocation from before its scene module loads."""
    if not enabled:
        yield None
        return
    recorder = begin(invocation=True)
    try:
        yield recorder
    finally:
        recorder.stop()
