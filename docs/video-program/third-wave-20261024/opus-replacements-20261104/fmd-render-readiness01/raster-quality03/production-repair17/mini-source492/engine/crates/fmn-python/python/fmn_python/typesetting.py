"""Host cache configuration for the production native typesetting engines.

The host owns configuration; fmn-cache owns paths, checksums and publication,
fmn-tex owns layouts and source spans. No cache payload is parsed in Python.
"""
from __future__ import annotations

import ast
from copy import deepcopy
from functools import wraps
from importlib import import_module
import inspect
import json
import os
from pathlib import Path
import sys
import textwrap
from threading import local
import time
import tokenize
from typing import Any


_STATE = "_fmn_typesetting_runtime"


def _directory(value: os.PathLike[str] | str | None, enabled: bool) -> str | None:
    if not isinstance(enabled, bool):
        raise TypeError("typeset cache enabled must be bool")
    if value is None:
        return "" if enabled else None
    text = os.fspath(value)
    if not isinstance(text, str):
        raise TypeError("typeset cache directory must be a text path")
    if "\0" in text or len(text.encode("utf-8")) > 4096:
        raise ValueError("typeset cache directory must contain at most 4096 UTF-8 bytes and no NUL")
    if ".." in Path(text).parts:
        raise ValueError("typeset cache directory may not contain parent-directory components")
    # Freeze a relative path before a scene can chdir. Do not resolve symlinks:
    # fmn-cache must see and reject link-like ownership boundaries itself.
    if text:
        text = os.path.abspath(os.path.expanduser(text))
        if len(text.encode("utf-8")) > 4096:
            raise ValueError("resolved typeset cache directory exceeds 4096 UTF-8 bytes")
    return text if enabled else None


def _diagnose(report: dict[str, Any]) -> None:
    errors = sorted({row["error"] for row in report["templates"].values() if row.get("error")})
    if not errors:
        return
    # One bounded, structured diagnostic per attempted binding. A full disk or
    # foreign directory never redirects the cache and never suppresses ink.
    event = {"event": "typeset-cache-unavailable", "fallback": "memory-and-layout",
             "errors": [error[:2048] for error in errors[:3]]}
    try:
        sys.stderr.write(json.dumps(event, ensure_ascii=True, sort_keys=True) + "\n")
    except Exception:
        # Diagnostic sinks are not part of the render contract. Cancellation
        # still propagates; ordinary logging failures cannot suppress ink.
        pass


_MAX_STATIC_REQUESTS = 4096
_MAX_STATIC_SOURCE_BYTES = 262_144
_MAX_STATIC_BATCH_BYTES = 4 * 1024 * 1024
_OPTION_KEYWORDS = ("template", "additional_preamble", "alignment")


def _configured_cache_directory(native: Any) -> str | None:
    """The config's ``directories.cache`` (custom_config.yml), or None."""
    config = getattr(native, "manim_config", None)
    try:
        directories = config["directories"] if config is not None else None
        value = directories.get("cache") if directories is not None else None
    except (KeyError, TypeError, AttributeError):
        return None
    return value if isinstance(value, str) and value else None


def _literal(node: ast.AST) -> str | None:
    return node.value if isinstance(node, ast.Constant) and isinstance(node.value, str) else None


def _static_tex_requests(scene_type: type, native: Any) -> list[tuple[str, str, str, bool, str]]:
    """Statically discover the literal Tex/TexText constructions a scene makes.

    Every user-authored class in the MRO is read once: the scene class, its
    scene bases, and mixins wherever C3 linearization places them (a mixin
    listed first must not hide the bases after it, and one listed after a
    scene base sits behind the portal's ``Scene`` itself). The portal's own
    classes and the standard library are skipped. Only calls whose callee resolves, in the defining module, to the portal's
    exact ``Tex`` or ``TexText`` class and whose layout inputs (positional
    strings, ``template``, ``additional_preamble``, ``alignment``) are all
    string literals are collected. Anything dynamic is left to construction:
    a missed or mismatched guess only costs an unused warm entry, never ink.
    Unreadable source (none recorded, or a file edited since import) is
    skipped, never an error. Returns de-duplicated
    ``(source, template, preamble, text_mode, align)``.
    """
    kinds = {id(native.Tex): False, id(native.TexText): True}
    stop = native.Scene
    found: dict[tuple[str, str, str, bool, str], None] = {}
    for cls in scene_type.__mro__:
        name = getattr(cls, "__module__", None)
        if (cls is stop or not isinstance(name, str)
                or name.startswith(("manimlib", "fmn_python"))
                or name.partition(".")[0] in sys.stdlib_module_names):
            continue
        module = sys.modules.get(name)
        if module is None:
            continue
        try:
            tree = ast.parse(textwrap.dedent(inspect.getsource(cls)))
        except (OSError, TypeError, SyntaxError, ValueError, tokenize.TokenError):
            # Python 3.13+ reports a source file edited since import (an open
            # string or bracket) as tokenize.TokenError, not SyntaxError.
            continue
        namespace = vars(module)
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name):
                continue
            text_mode = kinds.get(id(namespace.get(node.func.id)))
            if text_mode is None or not node.args:
                continue
            parts = [_literal(arg) for arg in node.args]
            if any(part is None for part in parts):
                continue
            options = {"template": "", "additional_preamble": "", "alignment": "\\centering"}
            dynamic = False
            for keyword in node.keywords:
                if keyword.arg is None:
                    dynamic = True  # **kwargs may carry layout options
                elif keyword.arg in _OPTION_KEYWORDS:
                    value = _literal(keyword.value)
                    if value is None:
                        dynamic = True
                    else:
                        options[keyword.arg] = value
            if dynamic:
                continue
            # Tex.__init__: strip the outer ends, join on one space, strip.
            parts[0], parts[-1] = parts[0].lstrip(), parts[-1].rstrip()
            source = " ".join(parts).strip() or "\\\\"
            try:
                align = native._tex_line_align(options["alignment"], text_mode)
            except Exception:
                continue
            key = (source, options["template"], options["additional_preamble"], text_mode, align)
            found.setdefault(key)
            if len(found) >= _MAX_STATIC_REQUESTS:
                return list(found)
    return list(found)


def install_typesetting(native: Any) -> None:
    """Install once per native module; each host thread has its own engines.

    Scene construction configures the persistent cache before construct():
    the config's ``directories.cache`` when set, else the platform default.
    A config value the cache refuses, like any refused root, leaves the
    thread typesetting in memory with the reason in its report; it never
    fails a Scene. Direct Tex construction outside a Scene uses the same
    binding through the native option-validation seam. Importing the
    package alone does no I/O.

    ``Scene.run`` also runs the static preflight before setup(): every literal
    Tex/TexText string the scene class's own source (and its user-authored
    bases and mixins) constructs is typeset on the native worker pool, so
    construct() and play() find them warm.
    """
    g = vars(native)
    if _STATE in g:
        return
    backend, inspect_backend = g["_configure_tex_cache"], g["_tex_cache_info"]
    state = local()

    def configure(directory=None, *, enabled=True):
        selected = _directory(directory, enabled)
        # Reconfiguration is explicit even when the path is unchanged: after
        # --clear-cache, reopen the new owned-root generation and cold-start
        # the memory front. Ordinary constructors use ensure() and do NOT
        # repeatedly reopen the store or discard its working set.
        report = backend(selected)
        state.report = deepcopy(report)
        _diagnose(report)
        return deepcopy(report)

    def ensure():
        if hasattr(state, "report"):
            return
        configured = _configured_cache_directory(native)
        try:
            configure(configured)
        except (TypeError, ValueError) as error:
            # configure_tex_cache() refuses a bad explicit path loudly, but a
            # config value (`..`, NUL, over-long) must not fail every Scene and
            # Tex: like any refused root, typeset in memory and say why.
            report = backend(None)
            refusal = f"directories.cache {configured!r} refused: {error}"[:2048]
            for row in report["templates"].values():
                row["error"] = refusal
            state.report = deepcopy(report)
            _diagnose(report)

    def info():
        ensure()
        report = deepcopy(state.report)
        current = inspect_backend()
        for name, values in current.items():
            report["templates"][name].update(values)
        return report

    def static_preflight(scene) -> dict[str, Any]:
        """Typeset a scene's literal Tex strings before its lifecycle runs."""
        started = time.perf_counter_ns()
        report = {"schema": "fmn-python.static-tex-preflight", "version": 1,
                  "enabled": getattr(state, "static_preflight", True),
                  "discovered": 0, "requests": 0, "succeeded": 0, "failed": 0,
                  "batches": 0, "worker_limit": 0, "wall_ns": 0}
        if not report["enabled"]:
            return report
        requests = _static_tex_requests(type(scene), native)
        report["discovered"] = len(requests)
        groups: dict[tuple[str, str, bool, str], list[str]] = {}
        total = 0
        for source, template, preamble, text_mode, align in requests:
            size = len(source.encode("utf-8")) + len(preamble.encode("utf-8"))
            if (len(source.encode("utf-8")) > _MAX_STATIC_SOURCE_BYTES
                    or len(preamble.encode("utf-8")) > _MAX_STATIC_SOURCE_BYTES
                    or len(template.encode("utf-8")) > 1024
                    or total + size > _MAX_STATIC_BATCH_BYTES):
                continue
            total += size
            groups.setdefault((template, preamble, text_mode, align), []).append(source)
        workers = max(1, min(32, os.cpu_count() or 1))
        report["worker_limit"] = workers
        for (template, preamble, text_mode, align), sources in groups.items():
            try:
                receipt = g["_preflight_tex"](sources, template=template, preamble=preamble,
                                              text_mode=text_mode, alignment=align,
                                              max_workers=workers)
            except (ValueError, TypeError, NotImplementedError, MemoryError):
                # Template/preamble refusals surface again, precisely, at
                # construction; the preflight is only a warm-up.
                report["failed"] += len(sources)
                continue
            report["batches"] += 1
            report["requests"] += receipt["count"]
            report["succeeded"] += receipt["succeeded"]
            report["failed"] += receipt["failed"]
        report["wall_ns"] = time.perf_counter_ns() - started
        return report

    Scene = g["Scene"]
    previous_init = Scene.__init__
    previous_validate = g["_validate_tex_options"]
    previous_lifecycle = Scene._run_lifecycle

    @wraps(previous_init)
    def scene_init(self, *args, **kwargs):
        previous_init(self, *args, **kwargs)
        ensure()

    @wraps(previous_validate)
    def validate(*args, **kwargs):
        # Preserve template/preamble refusals at their original boundary.
        result = previous_validate(*args, **kwargs)
        ensure()
        return result

    @wraps(previous_lifecycle)
    def run_lifecycle(self):
        ensure()
        # The static strings are known before setup()/construct() build any
        # Tex, so their layouts run in parallel ahead of the first play().
        before = inspect_backend()
        report = static_preflight(self)
        report["before"], report["after"] = before, inspect_backend()
        self._fmn_static_tex_preflight = report
        return previous_lifecycle(self)

    def set_static_preflight(enabled=True):
        if not isinstance(enabled, bool):
            raise TypeError("static Tex preflight enabled must be bool")
        state.static_preflight = enabled

    Scene.__init__ = scene_init
    Scene._run_lifecycle = run_lifecycle
    g["_validate_tex_options"] = validate
    g["_fmn_configure_tex_cache"] = configure
    g["_fmn_tex_cache_info"] = info
    g["_fmn_ensure_tex_cache"] = ensure
    g["_fmn_set_static_tex_preflight"] = set_static_preflight
    g[_STATE] = state


def configure_tex_cache(
    directory: os.PathLike[str] | str | None = None, *, enabled: bool = True,
) -> dict[str, Any]:
    """Select the current thread's persistent typeset cache.

    None/empty directory selects the native owned per-user cache. An explicit
    directory must be an absent or already-owned dedicated cache leaf, not an
    arbitrary existing folder. enabled=False detaches without deleting files.
    Storage failures return a diagnostic receipt with persistent=False; normal
    native typesetting remains available. Repeating this call reopens the root
    generation, useful after clearing the cache. No public class is replaced.
    """
    return import_module("manimlib")._fmn_configure_tex_cache(directory, enabled=enabled)


def configure_static_tex_preflight(enabled: bool = True) -> None:
    """Enable or disable the current thread's automatic static preflight.

    When enabled (the default), ``Scene.run`` first typesets every literal
    ``Tex``/``TexText`` string found in the scene class's own source, and in
    its user-authored bases and mixins, on the native worker pool. The
    receipt is ``scene._fmn_static_tex_preflight``.
    """
    import_module("manimlib")._fmn_set_static_tex_preflight(enabled)


_RECEIPT_COUNTERS = ("memory_hits", "disk_hits", "layout_computations", "disk_bytes_read",
                     "disk_bytes_written", "disk_rejected", "disk_errors")


def typesetting_receipt(scene: Any) -> dict[str, Any] | None:
    """One ``Scene.run``'s typesetting, as observed: cache traffic since its
    static preflight began, summed over the template engines, plus the
    preflight itself. Diagnostic only; never part of a certified manifest.
    ``misses`` counts fresh layouts (requests no cache layer served).
    """
    report = getattr(scene, "_fmn_static_tex_preflight", None)
    if not isinstance(report, dict) or "before" not in report:
        return None
    now = import_module("manimlib")._fmn_tex_cache_info()["templates"]
    totals = dict.fromkeys(_RECEIPT_COUNTERS, 0)
    for name, row in now.items():
        start = report["before"].get(name, {})
        for key in _RECEIPT_COUNTERS:
            totals[key] += int(row.get(key, 0)) - int(start.get(key, 0))
    after = report["after"].get("default", {})
    binding = now.get("default", {})
    return {
        "persistent": bool(binding.get("persistent")),
        # Why the store is not attached (a refused or unavailable root), as
        # the CLI's record names it; None while it is attached.
        "cache_error": binding.get("error") or None,
        "hits": totals["memory_hits"] + totals["disk_hits"],
        "memory_hits": totals["memory_hits"], "disk_hits": totals["disk_hits"],
        "misses": totals["layout_computations"],
        "bytes_read": totals["disk_bytes_read"], "bytes_written": totals["disk_bytes_written"],
        "rejected": totals["disk_rejected"],
        # Reads/writes an attached store failed: attached, but not caching.
        "store_errors": totals["disk_errors"],
        "preflight": {key: report[key] for key in ("enabled", "discovered", "requests",
                                                  "succeeded", "failed", "wall_ns")}
        | {"workers": int(after.get("preflight_workers", 0)),
           "active_workers": int(after.get("preflight_active_workers", 0))},
    }


def describe_receipt(receipt: dict[str, Any]) -> str:
    """The human render line's typesetting clause, worded like ``fmn``'s."""
    if receipt.get("cache_error"):
        store = f"typeset cache unavailable ({receipt['cache_error']})"
    else:
        store = "typeset cache" if receipt["persistent"] else "typeset memory cache"
    text = (f"; {store}: {receipt['hits']} hits ({receipt['disk_hits']} from disk), "
            f"{receipt['misses']} misses, {receipt['bytes_read']} bytes read, "
            f"{receipt['bytes_written']} bytes written")
    if receipt["rejected"]:
        text += f", {receipt['rejected']} corrupt entries recomputed"
    if receipt.get("store_errors"):
        text += f", {receipt['store_errors']} cache reads or writes failed (typeset in memory)"
    preflight = receipt["preflight"]
    if preflight["requests"]:
        text += (f"; preflight {preflight['requests']} static strings on "
                 f"{preflight['active_workers']} workers in {preflight['wall_ns'] / 1e6:.1f} ms")
    return text


def tex_cache_info() -> dict[str, Any]:
    """Current binding and actual native memory/disk-hit/layout counters.

    Counters are diagnostics, not a certified manifest or a performance claim.
    Configuration and statistics belong to the calling thread, just like the
    unsendable portal objects and their native engine slots.
    """
    return import_module("manimlib")._fmn_tex_cache_info()


def _preflight_arguments(sources, *, template="", preamble="", text_mode=False,
                         alignment=None, max_workers=32):
    """Own a bounded batch before cache configuration or native work begins."""
    if isinstance(sources, (str, bytes)):
        raise TypeError("Tex preflight sources must be an iterable of strings, not one string")
    if not isinstance(template, str) or not isinstance(preamble, str):
        raise TypeError("Tex preflight template and preamble must be strings")
    if not isinstance(text_mode, bool):
        raise TypeError("Tex preflight text_mode must be bool")
    if isinstance(max_workers, bool) or not isinstance(max_workers, int):
        raise TypeError("Tex preflight max_workers must be an integer")
    if not 1 <= max_workers <= 64:
        raise ValueError("Tex preflight max_workers must be in 1..=64")
    if alignment is not None and (not isinstance(alignment, str)
            or alignment not in (("left", "center", "right") if text_mode else ("left",))):
        raise ValueError("Tex preflight alignment must be left for mathematics, or left/center/right for text")

    def size(value, limit, name):
        # Check code points before encoding so a rejected giant input cannot
        # allocate another giant temporary UTF-8 buffer merely to be refused.
        if str.__len__(value) > limit:
            raise ValueError(f"Tex preflight {name} exceeds {limit} UTF-8 bytes")
        count = len(str.encode(value, "utf-8"))
        if count > limit:
            raise ValueError(f"Tex preflight {name} exceeds {limit} UTF-8 bytes")
        return count

    size(template, 1024, "template")
    preamble_bytes = size(preamble, 262_144, "preamble")
    batch, total = [], 0
    for index, source in enumerate(sources):
        if index >= 4096:
            raise ValueError("Tex preflight exceeds 4096 sources")
        if not isinstance(source, str):
            raise TypeError(f"Tex preflight source {index} must be str")
        total += size(source, 262_144, f"source {index}") + preamble_bytes
        if total > 4 * 1024 * 1024:
            raise ValueError("Tex preflight exceeds 4 MiB of source and preamble bytes")
        batch.append(source)
    return batch, dict(template=template, preamble=preamble, text_mode=text_mode,
                       alignment=alignment, max_workers=max_workers)


def preflight_tex(sources, *, template="", preamble="", text_mode=False,
                  alignment=None, max_workers=32) -> dict[str, Any]:
    r"""Typeset a batch before constructing Tex/TexText objects or playing frames.

    This warms the SAME native engine and content-addressed cache that ordinary
    constructors use. Mathematics defaults to display style, TexText to centered
    text with math islands. Use the same template, preamble and alignment as the
    subsequent constructors. Scale, color and isolate selections are not layout
    inputs. No Python layout worker or temporary engine is created.

    For example: preflight_tex([r"x^2", r"\frac{1}{2}"], max_workers=4).
    Invalid batch types and size/worker/alignment limits fail before cache
    configuration. Native template/preamble validation is retained. Formula errors
    are returned per source in input order; later valid sources still warm. The
    receipt includes actual before/after native cache counters and a worker
    ceiling, NOT a guarantee about the number of threads the host can start.
    This explicit batch does not discover dynamic strings in arbitrary Python.
    """
    batch, options = _preflight_arguments(
        sources, template=template, preamble=preamble, text_mode=text_mode,
        alignment=alignment, max_workers=max_workers,
    )
    module = import_module("manimlib")
    module._fmn_ensure_tex_cache()
    return module._preflight_tex(batch, **options)
