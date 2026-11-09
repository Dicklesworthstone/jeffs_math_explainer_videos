"""Explicit terminal display of immutable snapshots through native Studio codecs.

Only transport is host-owned. No image encoder, resampler, renderer, scene
capture or terminal autodetection is implemented here. Default stdout must be
a terminal; an explicit text stream may instead capture a protocol transcript.
"""
from __future__ import annotations

import importlib
import sys


def _protocol(protocol):
    if type(protocol) is not str:
        raise TypeError("terminal preview protocol must be 'kitty' or 'sixel'")
    if protocol not in ("kitty", "sixel"):
        raise ValueError("terminal preview protocol must be 'kitty' or 'sixel'")
    return protocol


def _stream(stream):
    if stream is None:
        stream = sys.stdout
        if not getattr(stream, "isatty", lambda: False)():
            raise RuntimeError("terminal preview requires a terminal stdout or an explicit text stream")
    if not callable(getattr(stream, "write", None)) or not callable(getattr(stream, "flush", None)):
        raise TypeError("terminal preview stream must implement write(text) and flush()")
    return stream


def _encoding_type(native):
    capture = getattr(native, "_CameraCapture", None)
    if not isinstance(capture, type) or not callable(getattr(capture, "terminal_bytes", None)):
        capability = getattr(native, "_CapabilityError", RuntimeError)
        raise capability("native terminal snapshot encoding is unavailable in this runtime")
    return capture


def _snapshot(native, snapshot):
    if not isinstance(snapshot, _encoding_type(native)):
        raise TypeError("terminal preview requires a native Camera.capture_snapshot image")


def _validate_project_preview(project, protocol):
    _protocol(protocol)
    if not project.capture or project.preview is None:
        raise ValueError("automatic terminal preview requires SceneProject(capture=True)")
    _snapshot(project._native, project.preview)
    _stream(None)


def _show_snapshot(snapshot, protocol, stream, max_bytes, native):
    _protocol(protocol)
    if type(max_bytes) is not int:
        raise TypeError("max_bytes must be an integer")
    if not 1 <= max_bytes <= 134217728:
        raise ValueError("max_bytes must be in 1..=134217728")
    stream = _stream(stream)
    _snapshot(native, snapshot)
    # Complete native encoding before the first write. A budget/codec refusal
    # cannot leave a partial escape sequence. Stream I/O itself is not atomic.
    payload = snapshot.terminal_bytes(protocol, max_bytes=max_bytes)
    text = payload.decode("ascii")
    # Use the current text stream, not its .buffer: IPython's raw stdout proxy
    # serializes these already-encoded bytes above a live prompt. Never retain
    # that temporary proxy past this display or select a different GUI loop.
    for start in range(0, len(text), 65536):
        remaining = text[start:start+65536]
        while remaining:
            written = stream.write(remaining)
            if type(written) is not int or not 0 < written <= len(remaining):
                raise OSError("terminal preview stream made no valid write progress")
            remaining = remaining[written:]
    stream.flush()
    return snapshot


def show_snapshot(snapshot, *, protocol="kitty", stream=None, max_bytes=16_777_216):
    """Display a frozen native image without recapture, files or scene effects.

    Select a protocol supported by the destination terminal explicitly. Kitty
    preserves the image's exact PNG; sixel is Studio's 216-color, one-bit-alpha
    preview. ``stream`` is an optional text writer for a protocol transcript.
    When omitted, the current stdout must be a terminal. Encoding finishes
    before writes begin; I/O failures may still leave a partial terminal image.
    Return the same immutable snapshot, usable for PNG redisplay or saving.
    """
    native = importlib.import_module("manimlib")
    return _show_snapshot(snapshot, protocol, stream, max_bytes, native)
