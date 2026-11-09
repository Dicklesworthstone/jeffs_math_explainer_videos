"""Per-play/wait clips through the existing native recording boundary.

The collection never replays construct, copies a Scene, or samples an animation.
Each selected top-level segment owns one ordinary Reel generation. Completed
clips are durable, create-only publications; failure cancels only the current
clip and retains exact receipts for the completed ones.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
import importlib
import os
from pathlib import Path
from typing import Any

from .recording import RecordingSession
from .scene_execution import _note
from .rendering import RenderResult, RenderSession, _positive_integer
from .render_selection import apply_animation_range

_OWNER = "_fmn_subdivision_session"
_AUDIO_FORMATS = frozenset({"wav", "mp4", "mov"})
_FORMATS = frozenset({"png_sequence", "gif", "y4m"}) | _AUDIO_FORMATS


@dataclass(frozen=True)
class RenderSegment:
    """One published clip, with its original Scene segment index and clock."""

    play_index: int
    kind: str
    start_frame: int
    end_frame: int
    render: RenderResult

    def as_dict(self) -> dict[str, Any]:
        return {"play_index": self.play_index, "kind": self.kind,
                "start_frame": self.start_frame, "end_frame": self.end_frame,
                "render": self.render.as_dict()}


@dataclass(frozen=True)
class SubdividedRenderResult:
    """A collection receipt. Incomplete collections name only published clips."""

    destination: Path
    segments: tuple[RenderSegment, ...]
    completed: bool
    unreceipted_artifacts: tuple[Path, ...] = ()

    @property
    def frame_count(self) -> int:
        return sum(segment.render.frame_count or 0 for segment in self.segments)

    @property
    def sample_frames(self) -> int | None:
        counts = [segment.render.sample_frames for segment in self.segments
                  if segment.render.sample_frames is not None]
        return sum(counts) if counts else None

    @property
    def bytes(self) -> int:
        return sum(segment.render.bytes for segment in self.segments)

    def as_dict(self) -> dict[str, Any]:
        return {"schema": "fmn.subdivided-render", "version": 1,
                "destination": str(self.destination), "completed": self.completed,
                "frame_count": self.frame_count, "sample_frames": self.sample_frames, "bytes": self.bytes,
                "segments": [segment.as_dict() for segment in self.segments],
                "unreceipted_artifacts": [str(path) for path in self.unreceipted_artifacts]}


class _Segment:
    def __init__(self, recording: RecordingSession, within_budget: bool) -> None:
        self.recording = recording
        self.within_budget = within_budget
        self.selected = False

    def select(self, selected: bool) -> None:
        self.selected = bool(selected)
        if self.selected and not self.within_budget:
            raise ValueError("subdivision segment budget exhausted")
        if not self.selected:
            # This runs after range/pre_play hooks but before native playback.
            # Abort has no final capture and never resets the populated Stage.
            self.recording.abort()


class SubdividedRecordingSession:
    """Record subsequent play/wait calls into a new clip directory.

    Existing mobjects, live array views, updaters, RNG and rational time retain
    their native owners. Resolution may differ from the scene camera; FPS must
    equal the existing native clock. PNG sequences, GIF and y4m are silent.
    WAV, MP4 and MOV use native sample-exact scene-audio windows: overlapping
    cues retain their original phase, gain and ducking without extending clips.
    Cues are those authored by clip completion; already published clips are
    immutable, not retroactively rewritten by later cues or source-file edits.

    Names retain the zero-based Scene.num_plays index: ``segment-000003.gif``
    or ``segment-000003/`` for a PNG sequence. Skipped segments and empty
    play() calls publish nothing. Nested AnimationGroups remain one clip.
    A selected zero-duration segment uses recording's one-final-frame rule.

    The directory must not exist. Earlier completed clips remain published
    after failure or cancellation; partial_result identifies them without
    claiming whole-collection success. No scene execution is rolled back.
    """

    def __init__(
        self, scene: Any, destination: os.PathLike[str] | str, *,
        format: str = "gif", resolution: tuple[int, int] | None = None,
        fps: int | None = None, threads: int | None = None,
        max_segments: int = 10_000, _native: Any = None,
        _prepare: bool = False, _selection=None,
    ) -> None:
        native = importlib.import_module("manimlib") if _native is None else _native
        text = os.fspath(destination)
        if not isinstance(text, str):
            raise TypeError("subdivision destination must be a text path")
        if not text or "\0" in text:
            raise ValueError("subdivision destination must be nonempty and contain no NUL")
        if not isinstance(format, str) or format not in _FORMATS:
            raise ValueError("subdivision requires png_sequence, gif, y4m, wav, mp4, or mov")
        self.destination = Path(os.path.abspath(text))
        self.max_segments = _positive_integer(max_segments, "max_segments", 100_000)
        # Reuse all camera/writer/native clock validation, without acquiring a
        # sink or touching the filesystem. This is not an entered recording.
        audio_clip = format in _AUDIO_FORMATS
        if audio_clip and not callable(getattr(native, "_portal_begin_audio_clip", None)):
            error_type = getattr(native, "_CapabilityError", RuntimeError)
            raise error_type("audio subdivision requires a matching native wheel")
        configuration = RenderSession if _prepare else RecordingSession
        self._configuration = configuration(
            scene, self.destination, format=format, resolution=resolution,
            fps=fps, threads=threads, _native=native, _allow_subdivide=True,
            **({} if _prepare else {"_scene_audio": audio_clip}),
        )
        width, height = self._configuration.resolution
        if width * height > 16_777_216 or self._configuration.threads > 96:
            raise ValueError("subdivision requires at most 16777216 pixels and 1..96 threads")
        self._prepare, self._selection = _prepare, _selection
        if _prepare and not callable(getattr(native, "_portal_prepare_recording_scene", None)):
            error_type = getattr(native, "_CapabilityError", RuntimeError)
            raise error_type("fresh-scene subdivision requires a matching native wheel; live recording remains available")
        self.scene, self._native = scene, native
        self._state = "new"
        self._segments: list[RenderSegment] = []
        self._unreceipted: list[Path] = []
        self._current: RecordingSession | None = None
        self._last_frame: int | None = None
        self.result: SubdividedRenderResult | None = None

    @property
    def segments(self) -> tuple[RenderSegment, ...]:
        return tuple(self._segments)

    @property
    def partial_result(self) -> SubdividedRenderResult:
        return SubdividedRenderResult(self.destination, self.segments, False, tuple(self._unreceipted))

    @property
    def artifact_published(self) -> bool:
        return bool(self._segments or self._unreceipted)

    def _check_owner(self) -> None:
        self._configuration._check_owner()

    def __enter__(self) -> SubdividedRecordingSession:
        self._check_owner()
        if self._state != "new":
            raise RuntimeError("a SubdividedRecordingSession can be entered only once")
        namespace = vars(self.scene)
        if (namespace.get(_OWNER) is not None
                or namespace.get("_fmn_owned_render_session") is not None):
            raise RuntimeError("this Scene already has an output owner")
        if namespace.get("_fmn_scene_execution") is not None:
            raise RuntimeError("subdivision must start between play/wait calls")
        if self._prepare:
            if os.path.lexists(self.destination):
                raise FileExistsError(f"subdivision destination already exists: {self.destination}")
            config = self._configuration
            self._native._portal_prepare_recording_scene(
                self.scene, *config.resolution, config.fps, config.seed,
            )
            self.scene.camera._core.set_pixel_shape(*config.resolution)
            self.scene.camera.fps = config.fps
            if self._selection is not None:
                apply_animation_range(self.scene, self._selection)
        _, self._last_frame = self._native._portal_scene_clock(self.scene)
        # Claim a fresh namespace before any authored scene execution. Each
        # child is still published atomically/no-clobber by its native sink.
        self.destination.mkdir(parents=True, exist_ok=False)
        namespace[_OWNER] = self
        self._state = "active"
        return self

    def _release(self) -> None:
        if vars(self.scene).get(_OWNER) is self:
            vars(self.scene).pop(_OWNER)

    def abort(self) -> None:
        self._check_owner()
        if self._state != "active":
            return
        self._state = "aborted"
        try:
            if self._current is not None:
                self._current.abort()
        finally:
            self._release()

    def _abort_preserving(self, error: BaseException) -> None:
        try:
            self.abort()
        except BaseException as cleanup:
            _note(error, "subdivision cancellation also failed: " + type(cleanup).__name__)

    @contextmanager
    def _segment(self, kind: str):
        self._check_owner()
        if self._state != "active":
            raise RuntimeError("subdivision is no longer active")
        if self._current is not None:
            raise RuntimeError("cannot nest a clip inside another clip")
        config = self._configuration
        fps, start = self._native._portal_scene_clock(self.scene)
        if fps != config.fps or start < self._last_frame:
            raise RuntimeError("the subdivision clock was reset; finish before restoring scene state")
        index = self.scene.num_plays
        if isinstance(index, bool) or not isinstance(index, int) or not 0 <= index < (1 << 63):
            raise ValueError("Scene.num_plays must be a nonnegative integer for subdivision")
        stem = f"segment-{index:06d}"
        path = self.destination / (stem if config.format == "png_sequence" else f"{stem}.{config.format}")
        recording = RecordingSession(
            self.scene, path, format=config.format, resolution=config.resolution,
            fps=config.fps, threads=config.threads, _native=self._native,
            _allow_subdivide=True, _scene_audio=config.format in _AUDIO_FORMATS,
        )
        self._current = recording
        try:
            with recording:
                # pre_play owns selection and EndScene range termination.
                # A full publication budget must not reject skipped calls,
                # empty plays or the exclusive range boundary before that hook.
                segment = _Segment(recording, len(self._segments) < self.max_segments)
                yield segment
            if recording.result is not None:
                self._segments.append(RenderSegment(
                    index, kind, recording.start_frame, recording.end_frame,
                    recording.result,
                ))
            # Even a skipped segment advances the existing native clock.
            _, self._last_frame = self._native._portal_scene_clock(self.scene)
        except BaseException:
            if recording.artifact_published and recording.result is None:
                self._unreceipted.append(recording.destination)
            raise
        finally:
            self._current = None

    def finish(self) -> SubdividedRenderResult:
        self._check_owner()
        if self._state == "finished":
            return self.result
        if self._state != "active":
            raise RuntimeError("only an active subdivision can finish")
        if self._current is not None or vars(self.scene).get("_fmn_scene_execution") is not None:
            raise RuntimeError("subdivision must finish between play/wait calls")
        if self._unreceipted:
            raise RuntimeError("a published clip has no complete receipt; inspect partial_result")
        fps, frame = self._native._portal_scene_clock(self.scene)
        if fps != self._configuration.fps or frame < self._last_frame:
            raise RuntimeError("the subdivision clock was reset; finish before restoring scene state")
        self.result = SubdividedRenderResult(self.destination, self.segments, True)
        self._state = "finished"
        self._release()
        return self.result

    def __exit__(self, exc_type, exc_value, traceback) -> bool:
        if exc_value is not None:
            self._abort_preserving(exc_value)
            _note(exc_value,
                f"subdivision retained {len(self._segments)} completed clips in {self.destination}"
            )
        elif self._state == "active":
            try:
                self.finish()
            except BaseException as error:
                self._abort_preserving(error)
                raise
        return False


def record_subdivided_scene(
    scene: Any, destination: os.PathLike[str] | str, *, format: str = "gif",
    resolution: tuple[int, int] | None = None, fps: int | None = None,
    threads: int | None = None, max_segments: int = 10_000,
) -> SubdividedRecordingSession:
    """Record each subsequent selected play/wait as a separate native clip."""
    return SubdividedRecordingSession(
        scene, destination, format=format, resolution=resolution, fps=fps,
        threads=threads, max_segments=max_segments,
    )
