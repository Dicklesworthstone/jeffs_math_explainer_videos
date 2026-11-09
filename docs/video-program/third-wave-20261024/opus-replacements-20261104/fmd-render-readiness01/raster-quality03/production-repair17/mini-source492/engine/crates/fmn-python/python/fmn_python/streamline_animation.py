"""Animated flow lines on the existing flash and Mobject updater protocols.

StreamLines still owns native RK45 geometry and native named-stream lag draws.
Each displayed line now owns a real VShowPassingFlash: no duplicate Gaussian,
record writer, renderer, or frame clock lives in this adapter.
"""
from __future__ import annotations

from collections.abc import Mapping
from functools import wraps
import math
from typing import Any


def _number(value, name, *, positive=False):
    result = float(value)
    if not math.isfinite(result) or result < 0 or (positive and result == 0):
        domain = "positive" if positive else "nonnegative"
        raise ValueError(name + " must be finite and " + domain)
    return result


def _note(error, cleanup):
    try:
        error.add_note("streamline animation cleanup failed: " + type(cleanup).__name__)
    except BaseException:
        pass


class _StreamlineController:
    """Own begun flashes, not the Scene; steps arrive through its dt updater."""

    def __init__(self, anchor, lines, animations):
        self.anchor, self.lines, self.animations = anchor, tuple(lines), tuple(animations)
        self.begun = []
        self.closed = self.busy = self.cleanup_pending = False

        def update(current, dt):
            # Mobject.copy shares function objects. A copied callback must
            # never advance or restore its source controller's live lines.
            if current is self.anchor:
                self.step(dt)

        update._fmn_streamline_controller = self
        self.update = update

    def start(self):
        self.busy = True
        try:
            for animation in self.animations:
                if self.closed:
                    break
                self.begun.append(animation)
                animation.begin()
        except BaseException as error:
            self.cancel(error)
            raise
        finally:
            self.busy = False
            if self.cleanup_pending:
                self._release()
        if not self.closed:
            try:
                self.anchor.add_updater(self.update)
            except BaseException as error:
                self.cancel(error)
                raise

    def _release(self, original=None):
        self.cleanup_pending = False
        begun, self.begun = self.begun, []
        first = None
        for animation in reversed(begun):
            try:
                # Abort restores flash widths and only the suspension owned
                # by that animation. Never publish/remove an effect's target.
                animation.abort()
            except BaseException as error:
                first = first or error
                if original is not None:
                    _note(original, error)
        try:
            self.anchor.remove_updater(self.update)
        except BaseException as error:
            first = first or error
            if original is not None:
                _note(original, error)
        if first is not None and original is None:
            raise first

    def cancel(self, original=None):
        if self.closed and not self.cleanup_pending:
            return
        self.closed = True
        if self.busy and original is None:
            self.cleanup_pending = True
            return
        self._release(original)

    def step(self, dt):
        if self.closed:
            return
        if self.busy:
            raise RuntimeError("AnimatedStreamLines cannot recursively advance itself")
        self.busy = True
        try:
            delta = _number(dt, "AnimatedStreamLines dt")
            # Validate all live durations and counters before advancing any
            # line. Edits to the public animation's runtime remain effective.
            frames = []
            for line, animation in zip(self.lines, self.animations):
                duration = _number(animation.get_run_time(), "streamline run_time")
                elapsed = float(line.time) + delta
                if not math.isfinite(elapsed):
                    raise ValueError("streamline time must remain finite")
                # Keep the existing signed phase-offset convention. A zero
                # virtual-time integral is stationary, never a modulo by zero.
                alpha = (elapsed % duration) / duration if duration else 0.0
                frames.append((line, animation, elapsed, duration, alpha))
            for line, animation, elapsed, duration, alpha in frames:
                if self.closed:
                    break
                line.time = elapsed
                animation.update_mobjects(delta)
                if self.closed:
                    break
                animation.interpolate(alpha)
            self.anchor._line_times = [line.time for line in self.lines]
            self.anchor._line_run_times = [row[3] for row in frames]
        except BaseException as error:
            self.cancel(error)
            raise
        finally:
            self.busy = False
            if self.cleanup_pending:
                self._release()


def install_streamline_animation(native: Any) -> None:
    """Install on the existing AnimatedStreamLines class and qualified aliases."""
    g = vars(native)
    if g.get("_FMN_STREAMLINE_ANIMATION_INSTALLED", False):
        return
    Animated = g.get("AnimatedStreamLines")
    if Animated is None:
        return
    StreamLines, Mobject = g["StreamLines"], g["Mobject"]

    def initialize(self, stream_lines, lag_range=4, rate_multiple=1.0,
                   line_anim_config=None, **kwargs):
        try:
            lines = list(stream_lines)
        except TypeError:
            raise TypeError(
                "AnimatedStreamLines requires an iterable of lines; got "
                + type(stream_lines).__name__
            ) from None
        if isinstance(stream_lines, StreamLines):
            metadata = list(stream_lines._stream_virtual_times)
            if len(lines) != len(metadata):
                raise ValueError("StreamLines virtual-time metadata must match its lines")
            draws = stream_lines._stream_rng_draws
        else:
            # vector_field.py:461 only iterates the group and reads each
            # line's virtual_time, so any family of timed lines animates
            # (hairy_ball flows plain Lines).
            for line in lines:
                if not hasattr(line, "virtual_time"):
                    raise AttributeError(
                        f"'{type(line).__name__}' object has no attribute 'virtual_time'"
                    )
            metadata = [line.virtual_time for line in lines]
            draws = 0
        lag = _number(lag_range, "AnimatedStreamLines lag_range")
        speed = _number(rate_multiple, "AnimatedStreamLines rate_multiple", positive=True)
        if line_anim_config is not None and not isinstance(line_anim_config, Mapping):
            raise TypeError("line_anim_config must be a mapping or None")
        config = dict(rate_func=g["linear"], time_width=1.0)
        if line_anim_config is not None:
            config.update(line_anim_config)
        if "run_time" in config:
            raise TypeError("line_anim_config.run_time conflicts with virtual_time / rate_multiple")
        virtual_times = [_number(getattr(line, "virtual_time", value), "streamline virtual_time")
                         for line, value in zip(lines, metadata)]
        durations = [_number(value / speed, "streamline run_time") for value in virtual_times]
        # Use exactly the existing native substream and draw offset, including
        # stationary lines; do not consume Python or NumPy global randomness.
        uniforms = list(g["_BridgeMobject"]._stream_line_lag_uniforms(0, draws, len(lines)))
        if len(uniforms) != len(lines) or any(not 0 <= value < 1 for value in uniforms):
            raise ValueError("native streamline lag draws must be one [0, 1) value per line")
        animations = [g["VShowPassingFlash"](line, run_time=duration, **config)
                      for line, duration in zip(lines, durations)]
        if any(not isinstance(animation, g["Animation"]) for animation in animations):
            raise TypeError("streamline flash constructor must return an Animation")
        # The original lines remain the displayed objects. kwargs are normal
        # VGroup styling/configuration rather than silently discarded values.
        super(Animated, self).__init__(*(animation.mobject for animation in animations), **kwargs)
        self.stream_lines, self.lag_range, self.rate_multiple = stream_lines, lag, speed
        self.time_width = config["time_width"]
        self.line_anim_config = dict(config)
        for line, animation, virtual_time, uniform in zip(lines, animations, virtual_times, uniforms):
            line.virtual_time, line.anim, line.time = virtual_time, animation, -lag * uniform
        self._line_times = [line.time for line in lines]
        self._line_run_times = durations
        controller = _StreamlineController(self, lines, animations)
        self._streamline_controller = controller
        controller.start()

    def update(self, dt=0, recurse=True):
        # The old override skipped normal updater dispatch and recursively
        # called itself from an updater. Use the inherited child-first pass;
        # its suspension, recurse flag and additional authored updaters stay live.
        return super(Animated, self).update(dt, recurse=recurse)

    for name, method in {"__init__": initialize, "update": update}.items():
        method.__name__ = name
        method.__qualname__ = Animated.__qualname__ + "." + name
        method.__module__ = Animated.__module__
        setattr(Animated, name, method)

    previous_remove, previous_clear = Mobject.remove_updater, Mobject.clear_updaters

    @wraps(previous_remove)
    def remove(self, updater):
        owner = getattr(updater, "_fmn_streamline_controller", None)
        if isinstance(owner, _StreamlineController) and owner.anchor is self and not owner.closed:
            owner.cancel()
        return previous_remove(self, updater)

    @wraps(previous_clear)
    def clear(self, recurse=True):
        first = None
        for member in self.get_family() if recurse else (self,):
            for updater in tuple(member.updaters):
                owner = getattr(updater, "_fmn_streamline_controller", None)
                if isinstance(owner, _StreamlineController) and owner.anchor is member:
                    try:
                        owner.cancel()
                    except BaseException as error:
                        first = first or error
        result = previous_clear(self, recurse=recurse)
        if first is not None:
            raise first
        return result

    Mobject.remove_updater, Mobject.clear_updaters = remove, clear
    g["_FMN_STREAMLINE_ANIMATION_INSTALLED"] = True
