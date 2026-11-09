"""Native clip output for an existing Scene, without replaying its lifecycle.

Unlike render_session, a recording preserves the populated arena and inherits
its rational frame clock. It starts a fresh Reel publication, not a new Scene.
Only sound cues authored inside the recording are included. This is standard
mode: neither arbitrary host history nor edited cells are a certified closure.
"""
from __future__ import annotations

import importlib
import os
from pathlib import Path
from typing import Any

from .rendering import RenderResult, RenderSession

_FORMATS = frozenset({"png_sequence", "gif", "y4m", "wav", "mp4", "mov"})


class RecordingSession(RenderSession):
    """Own one native output generation attached to the existing live Scene.

    The context never calls setup/construct/tear_down, reseeds the RNG, changes
    the camera, or replaces mobjects. FPS must equal the existing native clock;
    changing camera.fps alone does not resample an already running Scene.
    Successful exit returns the ordinary native RenderResult in ``result``.
    ``start_frame``/``end_frame`` describe the interval on the unchanged Scene
    clock, not the number of emitted frames (skipped segments emit no frames).

    Exceptions cancel publication, not scene effects. Finish before restoring
    checkpoints. An existing output or Studio capture cannot be stolen.
    """

    def __init__(
        self, scene: Any, destination: os.PathLike[str] | str, *,
        format: str | None = None, resolution: tuple[int, int] | None = None,
        fps: int | None = None, threads: int | None = None, _native: Any = None,
        _allow_subdivide: bool = False,
        _scene_audio: bool = False,
    ) -> None:
        native = importlib.import_module("manimlib") if _native is None else _native
        if not isinstance(scene, native.Scene):
            raise TypeError("record_scene requires a Scene instance")
        # Freeze relative paths before authored descriptors can change cwd.
        text = os.fspath(destination)
        if not isinstance(text, str):
            raise TypeError("recording destination must be a text path")
        if not text or "\0" in text:
            raise ValueError("recording destination must be nonempty and contain no NUL")
        path = Path(os.path.abspath(text))
        if format is None:
            format = path.suffix.lower().lstrip(".") if path.suffix else "png_sequence"
        if not isinstance(format, str) or format not in _FORMATS:
            raise ValueError("live recording requires png_sequence, gif, y4m, wav, mp4, or mov; use Camera.capture for a still")
        for name in ("_portal_scene_clock", "_portal_begin_recording"):
            if not callable(getattr(native, name, None)):
                error_type = getattr(native, "_CapabilityError", RuntimeError)
                raise error_type("the native portal does not provide live recording; rebuild the matching wheel")
        clock_fps, _ = native._portal_scene_clock(scene)
        super().__init__(scene, path, format=format, resolution=resolution,
                         fps=clock_fps if fps is None else fps, threads=threads, _native=native,
                         _allow_subdivide=_allow_subdivide)
        if self.fps != clock_fps:
            raise ValueError("recording FPS must equal the live Scene clock; recording cannot resample it")
        self._scene_audio = _scene_audio
        if _scene_audio and not callable(getattr(native, "_portal_begin_audio_clip", None)):
            error_type = getattr(native, "_CapabilityError", RuntimeError)
            raise error_type("audio subdivision requires a matching native wheel")
        self.start_frame: int | None = None
        self.end_frame: int | None = None

    def _between_segments(self) -> None:
        if vars(self.scene).get("_fmn_scene_execution") is not None:
            raise RuntimeError("recording must begin and finish between play/wait calls")

    def __enter__(self) -> RecordingSession:
        self._check_owner()
        if self._state != "new":
            raise RuntimeError("a RecordingSession can be entered only once")
        self._between_segments()
        self._check_output_owner()
        # A refusal must not abort someone else's live generation. Native code
        # rechecks ownership after getters and enforces the live FPS again.
        begin = (self._native._portal_begin_audio_clip if self._scene_audio else
                 self._native._portal_begin_recording)
        start = begin(
            self.scene, str(self.destination), self.format, *self.resolution,
            self.fps, self.threads,
        )
        self._state = "active"
        try:
            vars(self.scene)["_fmn_owned_render_session"] = self
            self.start_frame = start
        except BaseException as error:
            self._cancel_preserving(error)
            raise
        return self

    def finish(self) -> RenderResult:
        self._check_owner()
        if self._state != "active":
            return super().finish()
        try:
            self._between_segments()
            fps, end = self._native._portal_scene_clock(self.scene)
            if fps != self.fps or end < self.start_frame:
                raise RuntimeError("the live recording clock was reset; cancel before restoring checkpoints")
        except BaseException as error:
            self._cancel_preserving(error)
            raise
        result = super().finish()
        self.end_frame = end
        return result


def record_scene(
    scene: Any, destination: os.PathLike[str] | str, *, format: str | None = None,
    resolution: tuple[int, int] | None = None, fps: int | None = None,
    threads: int | None = None,
) -> RecordingSession:
    """Record subsequent play/wait calls on a populated Scene into a new clip.

    With no frames, native finish captures the current frame once. WAV still
    requires a sound cue. Existing destinations are never overwritten.
    """
    return RecordingSession(scene, destination, format=format, resolution=resolution,
                            fps=fps, threads=threads)
