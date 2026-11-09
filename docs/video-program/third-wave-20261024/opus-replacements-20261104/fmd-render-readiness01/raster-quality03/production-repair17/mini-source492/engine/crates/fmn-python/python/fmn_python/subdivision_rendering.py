"""Fresh-scene subdivision, sharing the live clip controller and native clock."""
from __future__ import annotations

import importlib
import os
from typing import Any

from .render_selection import animation_range as _animation_range
from .subdivision import SubdividedRecordingSession, SubdividedRenderResult, _FORMATS as _CLIP_FORMATS
from .rendering import _apply_output_options


def validate_subdivided_mode(native, format, *, reproducible=False, checkpoint=None):
    """Reject unsupported output modes before any Scene constructor executes."""
    if not isinstance(format, str) or format not in _CLIP_FORMATS:
        raise ValueError("subdivision requires an animated clip format or WAV, not a final-state still")
    if reproducible:
        raise RuntimeError("CAPABILITY: subdivided Python recordings do not certify arbitrary host history")
    if checkpoint is not None:
        raise ValueError("subdivided collections cannot reuse single-artifact checkpoint receipts")
    if not callable(getattr(native, "_portal_prepare_recording_scene", None)):
        raise RuntimeError("CAPABILITY: fresh-scene subdivision requires a matching native wheel")
    if format in {"wav", "mp4", "mov"} and not callable(getattr(native, "_portal_begin_audio_clip", None)):
        raise RuntimeError("CAPABILITY: audio subdivision requires a matching native wheel")


def subdivided_render_session(
    scene: Any, destination: os.PathLike[str] | str, *, format: str = "gif",
    resolution: tuple[int, int] | None = None, fps: int | None = None,
    threads: int | None = None, animation_range=None,
    max_segments: int = 10_000, _native: Any = None,
) -> SubdividedRecordingSession:
    """Own per-call clips on a pristine Scene with an explicitly configured FPS.

    This context never runs setup/construct/tear_down itself. For a populated
    scene, use record_subdivided_scene, which preserves the existing clock.
    """
    return SubdividedRecordingSession(
        scene, destination, format=format, resolution=resolution, fps=fps,
        threads=threads, max_segments=max_segments, _native=_native,
        _prepare=True, _selection=_animation_range(animation_range),
    )


def render_subdivided_scene(
    scene: Any, destination: os.PathLike[str] | str, *, format: str = "gif",
    resolution: tuple[int, int] | None = None, fps: int | None = None,
    threads: int | None = None, scene_kwargs: dict[str, Any] | None = None,
    animation_range=None, max_segments: int = 10_000,
    _output_options: dict[str, Any] | None = None,
) -> SubdividedRenderResult:
    """Run one Scene lifecycle, emitting a native clip for each selected call.

    Constructor, setup, construct and tear_down run only once. EndScene is
    successful range termination, not an output failure. Earlier clips remain
    published after execution failure; no scene or filesystem rollback occurs.
    The original exception carries render_subdivision_result with partial
    receipts whenever a collection was created. Constructor failures precede
    collection ownership and therefore do not carry a collection receipt.
    """
    selection = _animation_range(animation_range)
    native = importlib.import_module("manimlib")
    validate_subdivided_mode(native, format)
    if isinstance(scene, type) and issubclass(scene, native.Scene):
        scene = scene(**({} if scene_kwargs is None else dict(scene_kwargs)))
    elif scene_kwargs is not None:
        raise TypeError("scene_kwargs is valid only when rendering a Scene class")
    if _output_options:
        _apply_output_options(scene, _output_options)
    session = subdivided_render_session(
        scene, destination, format=format, resolution=resolution, fps=fps,
        threads=threads, animation_range=selection, max_segments=max_segments,
        _native=native,
    )
    try:
        with session:
            try:
                scene.run()
            except native.EndScene:
                pass
        if session.result is None:
            raise RuntimeError("scene execution ended without completing its subdivided generation")
        return session.result
    except BaseException as error:
        # Keep the exact authored exception (including Ctrl-C) and attach only
        # immutable publication receipts, never the live Scene or its proxies.
        try:
            error.render_subdivision_result = session.partial_result
        except BaseException:
            pass
        raise


def take_subdivision_option(options: list[str], value_flags) -> tuple[list[str], bool]:
    """Consume only --subdivide; never interpret another option's value."""
    forwarded, selected, index = [], False, 0
    while index < len(options):
        option = options[index]
        if option == "--subdivide":
            if selected:
                raise ValueError("--subdivide must not be repeated")
            selected = True
        else:
            forwarded.append(option)
            if option in value_flags and index + 1 < len(options):
                index += 1
                forwarded.append(options[index])
        index += 1
    return forwarded, selected


SUBDIVISION_HELP = """Per-call clip output:
  --subdivide             One native clip for each selected play() or wait().

Use one or more named scenes, or --write_all, with a supported clip format.
--video_dir is a fresh collection directory for one scene; for batches it is
an output root containing <Scene>/clips/. Source ranges retain original call
indices and apply to each scene; nested AnimationGroups remain one clip.
--keep-going preserves partial clips and proceeds after ordinary scene failures.
--skip_animations, checkpoint recovery and certified output are not combined
with subdivision. Audio-bearing formats require the matching native capability.
Completed clips survive later failures. Robot output includes per-scene
completed/partial collection receipts; interruptions preserve batch progress.
"""
