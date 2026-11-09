"""Playback selection, shared by render sessions and the wheel CLI.

Ranges count play/wait segments from zero with an exclusive end. Skipped
preroll still executes scene construction and animation endpoints; Choreo,
not this adapter, owns the sampling, updater order, and effective clock.
"""
from __future__ import annotations

import operator
import re
from typing import Any

_RANGE_FLAGS = frozenset({"-n", "--start_at_animation_number"})
_STILL_FLAGS = frozenset({"-s", "--skip_animations"})
_NATIVE_VALUES = frozenset({"--format", "--resolution", "--fps", "--threads", "--video_dir"})
_MAX_INDEX = (1 << 63) - 1


def animation_range(value: Any) -> tuple[int, int | None] | None:
    """Validate and freeze a programmatic (start, exclusive_end) selection."""
    if value is None:
        return None
    if not isinstance(value, (tuple, list)) or len(value) != 2:
        raise TypeError("animation_range must be (start, exclusive_end), with end optionally None")
    result = []
    for name, index in zip(("start", "end"), value):
        if name == "end" and index is None:
            result.append(None)
            continue
        if isinstance(index, bool):
            raise TypeError(f"animation range {name} must be an integer, not bool")
        try:
            index = operator.index(index)
        except TypeError:
            raise TypeError(f"animation range {name} must be an integer") from None
        if not 0 <= index <= _MAX_INDEX:
            raise ValueError(f"animation range {name} must lie between 0 and {_MAX_INDEX}")
        result.append(index)
    start, end = result
    if end is not None and end <= start:
        raise ValueError("animation range end is exclusive and must be greater than start")
    return start, end


def apply_animation_range(scene: Any, selection: tuple[int, int | None]) -> None:
    """Configure a newly owned generation, without changing its constructor.

    This runs after the native pristine-scene/ownership check and before run()
    or imperative segment calls. Explicit selection replaces an authored range
    but does not override an authored request to skip all animation.
    """
    if getattr(scene, "num_plays", 0) != 0:
        raise ValueError("animation_range requires a Scene with no completed play/wait segments")
    original = bool(getattr(scene, "original_skipping_status",
                            getattr(scene, "skip_animations", False)))
    scene.start_at_animation_number, scene.end_at_animation_number = selection
    scene.original_skipping_status = original
    scene.skip_animations = original or selection[0] > 0


def take_playback_options(options: list[str]):
    """Remove only the new playback flags; leave native values uninterpreted."""
    forwarded, selection, still = [], None, False
    index = 0
    while index < len(options):
        option = options[index]
        text = None
        if option in _RANGE_FLAGS:
            if index + 1 >= len(options):
                raise ValueError(f"{option} requires START or START,END")
            index += 1
            text = options[index]
        elif option.startswith("--start_at_animation_number="):
            text = option.split("=", 1)[1]
        elif option.startswith("-n") and len(option) > 2 and option[2].isascii() and option[2].isdigit():
            text = option[2:]
        elif option in _STILL_FLAGS:
            if still:
                raise ValueError("--skip_animations/-s must not be repeated")
            still = True
        else:
            forwarded.append(option)
            if option in _NATIVE_VALUES and index + 1 < len(options):
                index += 1
                forwarded.append(options[index])
        if text is not None:
            if selection is not None:
                raise ValueError("--start_at_animation_number/-n must not be repeated")
            if not re.fullmatch(r"[0-9]{1,19}(?:,[0-9]{1,19})?", text):
                raise ValueError("-n requires START or START,END using nonnegative integer indices")
            parts = text.split(",")
            selection = animation_range((int(parts[0]), int(parts[1]) if len(parts) == 2 else None))
        index += 1
    return forwarded, selection, still


def select_still_format(options: dict, forwarded: list[str], still: bool) -> dict:
    if not still:
        return options
    explicit, index = False, 0
    while index < len(forwarded):
        option = forwarded[index]
        explicit |= option == "--format" or option.startswith("--format=")
        index += 2 if option in _NATIVE_VALUES else 1
    if explicit and options["format"] != "png":
        raise ValueError("--skip_animations/-s saves one final PNG; it conflicts with an explicit non-PNG format")
    return dict(options, format="png")


PLAYBACK_HELP = """Playback selection:
  -n, --start_at_animation_number START[,END]
      Render from zero-based segment START up to exclusive END (or scene end).
      Both play() and wait() count. Preroll reaches real animation endpoints
      without capturing frames. Sound cues use the selected output timeline.
  -s, --skip_animations
      Save one final-state PNG; combines with -n to inspect a selected endpoint.
      An explicitly selected non-PNG output format is a usage error.

Programmatic renders accept animation_range=(START, END), with END optionally None.
Selection does not skip Python source execution or promise certified output.
"""
