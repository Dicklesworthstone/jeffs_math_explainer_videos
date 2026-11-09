"""Read-only Studio for existing Python scenes, using the native Studio host.

The selected source runs in a disposable instance of *this* host interpreter.
The supervisor does not import scene code. Captures are bounded, native PNGs
and native inspectors; scrubbing never executes source again. The Studio UI's
Reload button creates a fresh process, including fresh helper-module imports.
This is process isolation for robustness, not a Python security sandbox.
"""
from __future__ import annotations

import argparse
import contextlib
import copy
import hashlib
import json
import os
from pathlib import Path
import secrets
import sys
import threading
import time
from typing import Any
import weakref

from . import _ensure_exclusive_manimlib_namespace
from .studio_inputs import StudioInputs, project_directory

_SCHEMA = "fmn-python.studio-worker"
_MAX_SOURCE_BYTES = 16 * 1024 * 1024
_DEFAULT_FRAMES = 7200
_DEFAULT_BYTES = 256 * 1024 * 1024
_MAX_RELOAD_RESPONSE = 65536


class _Parser(argparse.ArgumentParser):
    def error(self, message):
        raise ValueError(message)


def _integer(value: Any, name: str, low: int, high: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not low <= value <= high:
        raise ValueError(f"{name} must be an integer in {low}..{high}")
    return value


def _source(path: Path) -> bytes:
    if path.suffix != ".py" or not path.is_file():
        raise ValueError("Studio source must be an existing .py file")
    with path.open("rb") as stream:
        data = stream.read(_MAX_SOURCE_BYTES + 1)
    if len(data) > _MAX_SOURCE_BYTES:
        raise ValueError("Studio source exceeds the 16 MiB source budget")
    # Compilation has no authored effects. Catch syntax/encoding errors before
    # replacing a working preview on an explicit Reload.
    compile(data, str(path), "exec", dont_inherit=True)
    return data


def _file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    budget = 512 * 1024 * 1024
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        if before.st_size > budget:
            raise ValueError("Studio runtime component exceeds the 512 MiB identity budget")
        while chunk := stream.read(min(1024 * 1024, budget + 1)):
            budget -= len(chunk)
            if budget < 0:
                raise ValueError("Studio runtime component grew past the identity budget")
            digest.update(chunk)
        after = os.fstat(stream.fileno())
    if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
        raise RuntimeError("Studio runtime changed while reading its identity")
    return digest.hexdigest()


def _canonical(value: dict[str, Any]) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False)


def _build_id(request: dict[str, Any]) -> str:
    return hashlib.sha256(_canonical({k: v for k, v in request.items() if k != "build_id"}).encode()).hexdigest()


class Studio:
    """An authenticated loopback preview. Close it, or use it as a context manager.

    Construction executes the selected scene once in a disposable worker.
    ``url`` contains a bearer capability: do not share it with untrusted users.
    Preview is silent/read-only; ordinary offline rendering remains separate.
    ``autoreload=True`` explicitly authorizes rerunning source on stabilized
    edits. It watches local Python sources and any additional ``watch_paths``;
    files named explicitly may be non-Python assets. Worker failures leave the
    previous healthy capture available and are exposed through ``reload_status``.
    """
    def __init__(
        self, source: os.PathLike[str] | str, scene: str, *,
        resolution: tuple[int, int] = (640, 360), fps: int = 30, threads: int = 1,
        max_frames: int = _DEFAULT_FRAMES, max_bytes: int = _DEFAULT_BYTES,
        timeout: int = 120, port: int = 0,
        autoreload: bool = False, watch_paths: tuple[os.PathLike[str] | str, ...] | list = (),
        debounce_ms: int = 200, interactive: bool = False,
    ) -> None:
        _ensure_exclusive_manimlib_namespace()
        from manimlib import _native

        if not isinstance(scene, str) or not scene.isidentifier() or len(scene) > 512:
            raise ValueError("Studio requires one explicit Scene class name")
        if not isinstance(autoreload, bool):
            raise ValueError("autoreload must be a bool")
        if not isinstance(interactive, bool):
            raise ValueError("interactive must be a bool")
        if not isinstance(watch_paths, (tuple, list)) or len(watch_paths) > 62:
            raise ValueError("watch_paths must be a list or tuple of at most 62 paths")
        if watch_paths and not autoreload:
            raise ValueError("watch_paths requires autoreload=True")
        debounce_ms = _integer(debounce_ms, "debounce_ms", 0, 10000)
        path = Path(source).resolve()
        _source(path)
        roots = list(dict.fromkeys([str(path), str(project_directory(path)),
                                   *(str(Path(value).resolve()) for value in watch_paths)]))
        try:
            width, height = resolution
        except (TypeError, ValueError):
            raise ValueError("resolution must contain width and height") from None
        width = _integer(width, "width", 1, 16384)
        height = _integer(height, "height", 1, 16384)
        if width * height > 16_777_216:
            raise ValueError("Studio resolution exceeds the 16M-pixel capture budget")
        options = {
            "width": width, "height": height, "fps": _integer(fps, "fps", 1, 240),
            "threads": _integer(threads, "threads", 1, 96),
            "max_frames": _integer(max_frames, "max_frames", 1, 100000),
            "max_bytes": _integer(max_bytes, "max_bytes", 1, 1024 * 1024 * 1024),
            "interactive": interactive,
        }
        timeout = _integer(timeout, "timeout", 1, 900)
        port = _integer(port, "port", 0, 65535)
        # abspath, NOT realpath: resolving a virtualenv interpreter symlink
        # would discard that environment's installed engine and dependencies.
        executable = Path(os.path.abspath(sys.executable))
        extension = Path(_native.__file__).resolve()
        runtime = {"python": _file_digest(executable), "native": _file_digest(extension),
                   "portal": _file_digest(Path(__file__).resolve()),
                   "inputs": _file_digest(Path(sys.modules[StudioInputs.__module__].__file__).resolve()),
                   "loader": _file_digest(Path(__file__).with_name("scene_loading.py")),
                   "abi": sys.implementation.cache_tag}
        # Freeze environment and interpreter authority at the host front door.
        # This is not a claim to record the complete C1-C10 certified closure.
        environment = sorted(os.environ.items())

        def rebuild():
            data = _source(path)
            inputs = StudioInputs(_native, roots)
            source_sha256 = hashlib.sha256(data).hexdigest()
            inputs.require_source(path, source_sha256)
            request = {"schema": _SCHEMA, "version": 2, "source": str(path), "scene": scene,
                       "source_sha256": source_sha256, "inputs": inputs.as_request(),
                       "runtime": runtime, **options}
            request["build_id"] = _build_id(request)
            return (str(executable), ["-I", "-m", "fmn_python.studio", "--worker", _canonical(request)],
                    environment, str(path.parent), request["build_id"])

        self._operation = threading.RLock()
        self._status_lock = threading.Lock()
        self._stop = threading.Event()
        self._watch_thread = None
        # Snapshot before executing the initial generation. An edit made while
        # that worker is rendering must be noticed on the first later poll.
        self._watch = _native._StudioHost.watch_sources(roots, debounce_ms) if autoreload else None
        self._timeout = timeout + 5
        self._autoreload = autoreload
        self._interactive = interactive
        self._reload_status = {"revision": 0, "completed": 0, "error": None, "result": None}
        self._host = _native._StudioHost(rebuild, scene, secrets.token_hex(32), port, timeout)
        try:
            if autoreload:
                self._watch_thread = threading.Thread(
                    target=_watch_loop, args=(weakref.ref(self), self._stop, self._watch),
                    name="fmn-python-source-watch", daemon=True,
                )
                self._watch_thread.start()
        except BaseException:
            self._stop.set()
            self._host.close()
            raise

    @property
    def url(self) -> str:
        with self._operation:
            return self._host.url

    @property
    def alive(self) -> bool:
        with self._operation:
            return self._host.alive

    @property
    def autoreload(self) -> bool:
        return self._autoreload

    @property
    def interactive(self) -> bool:
        return self._interactive

    @property
    def reload_status(self) -> dict[str, Any]:
        """Detached status; a failed edit is not a terminal host failure."""
        with self._status_lock:
            return copy.deepcopy(self._reload_status)

    def _record_reload(self, result=None, error=None) -> None:
        with self._status_lock:
            if error is not None and error == self._reload_status["error"]:
                return  # A refused scan must not flood robot output every poll.
            self._reload_status = {
                "revision": self._reload_status["revision"] + 1,
                "completed": self._reload_status["completed"] + int(error is None),
                "error": error, "result": result,
            }

    def reload(self) -> dict[str, Any]:
        """Reexecute in a fresh worker through the same authenticated native
        route as the browser's Reload button, including its operation lock,
        rate limits, opaque-effect handling and frame publication.
        """
        with self._operation:
            if self._stop.is_set() or not self._host.alive:
                raise RuntimeError("Studio is closed")
            try:
                result = _reload_request(self._host.url, self._timeout)
            except Exception as error:
                self._record_reload(error=str(error)[:4096])
                raise
            self._record_reload(result=result)
            return copy.deepcopy(result)

    def advance(self, frames: int = 1) -> dict[str, Any]:
        """Execute nominal live frames and return the native final-frame receipt.

        Select the final timeline frame first. This runs real authored updaters
        and does not add an authored wait. One command is bounded to one second
        and one optimistic revision; conflicts and timeouts are never retried.
        """
        from .studio_clock import advance
        with self._operation:
            if self._stop.is_set() or not self._host.alive:
                raise RuntimeError("Studio is closed")
            if not self._interactive:
                raise RuntimeError("Live stepping requires interactive=True")
            return advance(self._host.url, self._timeout, frames)

    def close(self) -> None:
        self._stop.set()
        thread = self._watch_thread
        # Never hold the operation lock while joining a thread which may need
        # it to finish. Native worker requests retain their configured deadline.
        if thread is not None and thread is not threading.current_thread() and thread.ident is not None:
            thread.join()
        with self._operation:
            self._host.close()

    def __enter__(self) -> Studio:
        if not self.alive:
            raise RuntimeError("Studio is closed")
        return self

    def __exit__(self, *_exc) -> None:
        self.close()


def _reload_request(url: str, timeout: int) -> dict[str, Any]:
    import urllib.error
    import urllib.parse
    import urllib.request

    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or parsed.username or parsed.password:
        raise RuntimeError("Studio reload requires its native loopback host")
    authority = "http://" + parsed.netloc
    request = urllib.request.Request(authority + "/api/restart?" + parsed.query, data=b"",
        headers={"Origin": authority, "Content-Type": "application/x-www-form-urlencoded"})

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *_args, **_kwargs):
            return None  # Never forward the bearer capability to another host.

    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    try:
        with opener.open(request, timeout=timeout) as response:
            data = response.read(_MAX_RELOAD_RESPONSE + 1)
        if len(data) > _MAX_RELOAD_RESPONSE:
            raise RuntimeError("Studio reload response exceeds its 64 KiB budget")
        result = json.loads(data)
        if not isinstance(result, dict) or not isinstance(result.get("sha256"), str):
            raise RuntimeError("Studio reload returned no native frame receipt")
        return result
    except urllib.error.HTTPError as error:
        with error:
            detail = error.read(4096).decode("utf-8", "replace")
        raise RuntimeError(f"Studio reload refused ({error.code}): {detail}") from None
    except (urllib.error.URLError, TimeoutError, OSError, ValueError) as error:
        raise RuntimeError(f"Studio reload failed: {error}") from error


def _watch_loop(owner_ref, stop, watch) -> None:
    # A weak owner lets a forgotten Python Studio object release its native
    # host/worker rather than being kept alive forever by its polling thread.
    while not stop.wait(0.1):
        owner = owner_ref()
        if owner is None:
            return
        try:
            if not owner.alive:
                return
            if watch.poll() and not stop.is_set():
                owner.reload()
        except Exception as error:
            if not stop.is_set():
                owner._record_reload(error=str(error)[:4096])
        finally:
            del owner


def _capture(request: dict[str, Any], native: Any):
    from .scene_loading import SceneSource
    from .rendering import _seed

    path = Path(request["source"])
    inputs = StudioInputs.from_request(native, request.get("inputs"))
    inputs.require_source(path, request["source_sha256"])
    if hashlib.sha256(_source(path)).hexdigest() != request["source_sha256"]:
        raise RuntimeError("Studio source changed between launch and capture; reload")
    with SceneSource(path, native.Scene, source_inputs=inputs.files) as loaded:
        if loaded.source_digests.get(path) != request["source_sha256"]:
            raise RuntimeError("Studio executed source differs from the requested generation")
        inputs.verify(loaded)
        if request["scene"] not in loaded.scenes:
            raise ValueError(f"Scene {request['scene']!r} was not defined in {path.name}")
        scene = loaded.scenes[request["scene"]]()
        inputs.verify(loaded)
        # No source rewrite, no constructor kwargs, no duplicated frame clock.
        scene._begin_studio_capture(
            request["scene"], request["build_id"], inputs.fingerprint,
            request["width"], request["height"], request["fps"], request["threads"],
            _seed(scene.random_seed), request["max_frames"], request["max_bytes"],
        )
        try:
            scene.camera._core.set_pixel_shape(request["width"], request["height"])
            scene.camera.fps = request["fps"]
            try:
                scene.run()
            except native.EndScene:
                pass
            inputs.verify(loaded)
            if request.get("interactive", False):
                # Retain both the source/import context and the actual scene
                # on this worker thread for event callbacks. Never reconstruct
                # their behavior from snapshots or execute them in the host.
                # Enable only the existing scene-scoped dispatcher's key
                # state while this worker owns live input. No fake Window is
                # installed and ordinary offline Scene input stays unchanged.
                scene.__dict__["_fmn_studio_live_input"] = True
                scene.__dict__["_fmn_studio_validate_inputs"] = lambda: inputs.verify(loaded)
                try:
                    scene._serve_studio_live()
                finally:
                    scene.__dict__.pop("_fmn_studio_live_input", None)
                    scene.__dict__.pop("_fmn_studio_validate_inputs", None)
                return None
            recording = scene._finish_studio_capture()
            # Finishing an empty scene can capture a frame and invoke authored
            # camera/updater hooks. Validate after those effects, before serve.
            inputs.verify(loaded)
            return recording
        except BaseException as error:
            try:
                scene._abort_render()
            except BaseException as cleanup:
                error.add_note("Studio capture cleanup also failed: " + type(cleanup).__name__)
            raise


def _worker(encoded: str) -> int:
    if len(encoded) > 65536:
        raise ValueError("Studio worker request exceeds its launch budget")
    request = json.loads(encoded)
    if not isinstance(request, dict) or request.get("schema") != _SCHEMA or request.get("version") != 2:
        raise ValueError("unsupported Studio worker request")
    if request.get("build_id") != _build_id(request):
        raise ValueError("Studio worker request identity mismatch")
    # Authored stdout must never enter the framed native protocol. Low-level
    # writes to fd 1 remain arbitrary Python authority, not a sandbox promise;
    # corrupted framing is rejected by the supervisor.
    with contextlib.redirect_stdout(sys.stderr):
        from manimlib import _native
        actual = {"python": _file_digest(Path(sys.executable)),
                  "native": _file_digest(Path(_native.__file__).resolve()),
                  "portal": _file_digest(Path(__file__).resolve()),
                  "inputs": _file_digest(Path(sys.modules[StudioInputs.__module__].__file__).resolve()),
                  "loader": _file_digest(Path(__file__).with_name("scene_loading.py")),
                  "abi": sys.implementation.cache_tag}
        if request["runtime"] != actual:
            raise RuntimeError("Studio worker interpreter/engine differs from the selected host runtime")
        from .scene_console import _STUDIO_WORKER
        token = _STUDIO_WORKER.set(True)
        try:
            recording = _capture(request, _native)
        finally:
            _STUDIO_WORKER.reset(token)
    if recording is not None:
        recording.serve()
    return 0


_HELP = """Read-only native Studio for Python scenes:
  fmn-python studio SOURCE.py SCENE [--resolution WxH] [--fps N] [--threads N]
                    [--max_frames N] [--max_bytes N] [--timeout SECONDS] [--port N]
                    [--autoreload] [--watch PATH ...] [--debounce_ms N] [--interactive]

Executes once in a disposable host-CPython worker; scrub and inspect the captured
frames in the authenticated native Studio UI. Reload explicitly executes fresh
source and imports. Ctrl-C closes the host and worker. This is not a sandbox.
Launch and publication bind the containing package's .py/.pyw inputs and
explicitly watched assets to the native capture identity. Changed or undeclared
compiled project inputs refuse publication, rather than serving a stale build.
--autoreload explicitly reruns source on content changes in that package;
--watch adds directories or explicit asset files. Native bounded scans
ignore caches/virtualenvs and debounce edits. A failed edit retains the healthy
preview and is retried only after another edit, not in an execution loop.
--interactive retains the live scene in its worker. Select the last timeline
frame and enable Scene input to send keyboard, pointer, drag and wheel events
through existing scene callbacks. Prior frames remain read-only. Failed callbacks
freeze input until reload; they are not rolled back or automatically retried.
Live commands also check the declared-input generation before callbacks and
before final capture: a helper/asset edit cannot silently mix two generations.
Select the final frame to Step live frame or Run live nominal updater ticks.
Run is bounded and pauses on input, navigation, focus loss or worker changes.
No sound playback, callback checkpoints, or certified render claim.
Default: 640x360, 30 FPS, 7200 frames, 256 MiB encoded capture budget.
"""


def try_studio_cli(native: Any, arguments: list[str]) -> int | None:
    args = list(arguments)
    robot = "--robot" in args
    args = [a for a in args if a != "--robot"]
    if not args or args[0] != "studio":
        return None
    if "--help" in args or "-h" in args:
        if robot:
            return native._portal_cli_emit(0, "success", "help", "Python Studio usage", True, help=_HELP)
        print(_HELP)
        return 0
    # These are preview resource controls, not a replacement scene/config parser.
    # Rendering itself still resolves the existing native camera/runtime config.
    parser = _Parser(prog="fmn-python studio", add_help=False, exit_on_error=False, allow_abbrev=False)
    parser.add_argument("source")
    parser.add_argument("scene")
    parser.add_argument("--resolution", default="640x360")
    parser.add_argument("--autoreload", action="store_true")
    parser.add_argument("--interactive", action="store_true")
    parser.add_argument("--watch", dest="watch_paths", action="append", default=[])
    parser.add_argument("--debounce_ms", type=int, default=200)
    for flag, default in (("fps", 30), ("threads", 1), ("max_frames", _DEFAULT_FRAMES),
                          ("max_bytes", _DEFAULT_BYTES), ("timeout", 120), ("port", 0)):
        parser.add_argument("--" + flag, type=int, default=default)
    phase = "arguments"
    try:
        options, extra = parser.parse_known_args(args[1:])
        if extra:
            raise ValueError("unsupported Studio options: " + " ".join(extra))
        dimensions = tuple(int(v) for v in options.resolution.lower().split("x"))
        config = vars(options).copy()
        config["resolution"] = dimensions
        phase = "start"
        with Studio(**config) as host:
            if robot:
                native._portal_cli_emit(0, "success", "studio", "Python Studio is ready", True,
                                       url=host.url, scene=options.scene,
                                       preview_mode="live-final-frame" if host.interactive else "captured-read-only",
                                       certified=False, autoreload=host.autoreload)
            else:
                print("Python Studio (" + ("live final frame" if host.interactive else "read-only") + "): " + host.url)
                print("Reload runs fresh source. Ctrl-C closes Studio.", file=sys.stderr)
            sys.stdout.flush()
            phase = "serve"
            revision = 0
            while host.alive:
                status = host.reload_status
                if status["revision"] != revision:
                    revision = status["revision"]
                    failed = status["error"] is not None
                    native._portal_cli_emit(6 if failed else 0, "execution" if failed else "success",
                        "studio-reload", status["error"] or "Python Studio reloaded", robot,
                        reload_status=status, terminal=False)
                    sys.stdout.flush()
                time.sleep(0.05)
        return 0
    except KeyboardInterrupt:
        return 130
    except (ValueError, OSError, RuntimeError, SyntaxError, argparse.ArgumentError, SystemExit) as error:
        code = 2 if phase == "arguments" else 6
        return native._portal_cli_emit(code, "usage" if code == 2 else "execution", "studio", str(error)[:4096], robot, phase=phase)


if __name__ == "__main__":
    try:
        if len(sys.argv) != 3 or sys.argv[1] != "--worker":
            raise ValueError("use fmn-python studio SOURCE.py SCENE")
        raise SystemExit(_worker(sys.argv[2]))
    except Exception as error:
        print("Python Studio worker failed: " + str(error)[:4096], file=sys.stderr)
        raise SystemExit(6) from None