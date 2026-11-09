"""One scene execution, a native animation output and a prepared final PNG.

Every artifact is independently atomic and create-only. Two arbitrary output
paths are not a filesystem transaction: a late collision on the PNG can leave
an already published movie. In that case the exception carries an exact
render_pair_result, never a success-shaped pair or a destructive rollback.
"""
from __future__ import annotations

from dataclasses import dataclass
from functools import wraps
import importlib
import os
from pathlib import Path
from typing import Any

from . import rendering

_OWNER = "_fmn_paired_render_session"
_CLIPS = frozenset({"gif", "y4m", "png_sequence", "mp4", "mov", "wav"})


def _path(value):
    text = os.fspath(value)
    if not isinstance(text, str) or not text or "\0" in text:
        raise ValueError("paired output destinations must be nonempty text paths without NUL")
    return Path(os.path.abspath(text))


def companion_png_path(destination, format, *, subdivide=False):
    """A file's stem.png, or a collection directory's adjacent name.png."""
    path = _path(destination)
    return (path.with_name(path.name + ".png") if subdivide or format == "png_sequence"
            else path.with_suffix(".png"))


def validate_paired_mode(native, format, *, reproducible=False, checkpoint=None):
    if format not in _CLIPS:
        raise ValueError("paired output requires gif, y4m, png_sequence, mp4, mov, or wav")
    if reproducible:
        raise RuntimeError("CAPABILITY: paired final PNGs are standard-only, not certified output")
    # BatchCheckpoint binds the paired mode and verifies both artifacts.
    # Subdivision and certified-output exclusions remain with their owners.
    if not callable(getattr(getattr(native, "_CameraCapture", None), "prepare_png", None)):
        raise RuntimeError("CAPABILITY: paired output requires the matching native PNG preparation API")


def take_last_frame_option(options, value_flags):
    """Consume only this switch, never an option value that resembles it."""
    forwarded, selected, index = [], False, 0
    while index < len(options):
        option = options[index]
        if option in {"--save-last-frame", "--save_last_frame"}:
            if selected:
                raise ValueError("--save-last-frame must not be repeated")
            selected = True
        else:
            forwarded.append(option)
            if option in value_flags and index + 1 < len(options):
                index += 1
                forwarded.append(options[index])
        index += 1
    return forwarded, selected


PAIRED_HELP = """Animation plus final image:
  --save-last-frame, --save_last_frame
      Also publish one final RGBA PNG without executing the Scene again.
      Files use their stem plus .png; clip/sequence directories use an adjacent
      directory-name.png. Applies to one scene, named batches, and --write_all.
      --subdivide retains per-call clips and adds a single final image.
      Not combined with -s/--skip_animations, still-only formats, certified
      output. Non-subdivided batches support checkpoint/resume: a completed
      pair is reused only after both artifacts are hash-verified. Partial pairs
      remain no-clobber failures. Each artifact is independently no-clobber.
      Late still-publication failure retains the primary output and reports
      an incomplete pair, never whole-scene success or destructive rollback.
"""


@dataclass(frozen=True)
class PairedRenderResult:
    destination: Path
    still_destination: Path
    primary: Any
    still: rendering.RenderResult | None
    completed: bool
    unreceipted_artifacts: tuple[Path, ...] = ()

    def as_dict(self):
        return {"schema": "fmn.paired-render", "version": 1,
                "destination": str(self.destination),
                "still_destination": str(self.still_destination),
                "completed": self.completed,
                "primary": None if self.primary is None else self.primary.as_dict(),
                "still": None if self.still is None else self.still.as_dict(),
                "unreceipted_artifacts": [str(path) for path in self.unreceipted_artifacts]}


class PairedRenderSession:
    """Compose existing RenderSession/subdivision ownership with one still.

    Preparation failures cancel the live primary before publication. Completed
    subdivision clips survive failures, exactly as in their ordinary owner.
    Native PNG preparation precedes primary finalization; PNG publication only
    follows successful primary finalization. No lifecycle or updater is replayed.
    """
    def __init__(self, primary, still_destination):
        self.primary = primary
        config = getattr(primary, "_configuration", primary)
        self._config, self.scene, self._native = config, primary.scene, primary._native
        self.destination = _path(primary.destination)
        self.still_destination = _path(still_destination)
        first, second = self.destination.resolve(), self.still_destination.resolve()
        if first == second or first in second.parents or second in first.parents:
            raise ValueError("paired output paths must be distinct and non-overlapping")
        if config.format not in _CLIPS:
            raise ValueError("a paired primary requires gif, y4m, png_sequence, mp4, mov, or wav")
        if config.reproducible:
            raise self._native._CapabilityError(
                "paired output's observational final PNG is standard-only; "
                "use a separate certified render for a complete input closure")
        if getattr(self.scene.file_writer, "png_mode", "RGBA") != "RGBA":
            raise ValueError("paired final frames require png_mode='RGBA'")
        if not callable(getattr(self._native._CameraCapture, "prepare_png", None)):
            raise self._native._CapabilityError("paired output requires the matching native PNG preparation API")
        self.result = None
        self._state, self._finishing = "new", False
        self._pending, self._still = None, None
        self._still_published = False

    @property
    def artifact_published(self):
        return self.primary.artifact_published or self._still_published

    @property
    def partial_result(self):
        primary = self.primary.result
        missing = []
        if primary is None:
            primary = getattr(self.primary, "partial_result", None)
            if primary is None and self.primary.artifact_published:
                missing.append(self.destination)
        if self._still_published and self._still is None:
            missing.append(self.still_destination)
        return PairedRenderResult(self.destination, self.still_destination,
                                  primary, self._still, self._state == "finished", tuple(missing))

    def _release(self):
        if vars(self.scene).get(_OWNER) is self:
            vars(self.scene).pop(_OWNER)

    def __enter__(self):
        self.primary._check_owner()
        if self._state != "new":
            raise RuntimeError("a PairedRenderSession can be entered only once")
        if vars(self.scene).get(_OWNER) is not None:
            raise RuntimeError("this Scene already has a paired output owner")
        for path in (self.destination, self.still_destination):
            if os.path.lexists(path):
                raise FileExistsError(f"paired output destination already exists: {path}")
        # Freeze the primary path too: authored constructors may later chdir.
        self.primary.destination = self.destination
        self.primary.__enter__()
        self._state = "active"
        try:
            vars(self.scene)[_OWNER] = self
        except BaseException:
            self.abort()
            raise
        return self

    def abort(self):
        self.primary._check_owner()
        if self._state != "active":
            return
        self._state = "aborted"
        try:
            if self._pending is not None:
                self._pending.abort()
        finally:
            try:
                self.primary.abort()
            finally:
                self._release()

    def _fail(self, error):
        try:
            self.abort()
        except BaseException as cleanup:
            try:
                error.add_note("paired output cancellation also failed: " + type(cleanup).__name__)
            except BaseException:
                pass
        try:
            error.render_pair_result = self.partial_result
        except BaseException:
            pass
        try:
            if self.artifact_published:
                error.add_note("paired output is incomplete; published artifacts are listed in render_pair_result")
        except BaseException:
            pass

    def finish(self):
        self.primary._check_owner()
        if self._state == "finished":
            return self.result
        if self._state != "active" or self._finishing:
            raise RuntimeError("only an active, non-finalizing pair can finish")
        if vars(self.scene).get("_fmn_scene_execution") is not None:
            raise RuntimeError("paired output must finish between play/wait calls")
        self._finishing = True
        try:
            camera = self.scene.camera
            capture_threads = rendering._positive_integer(getattr(camera, "capture_threads", 1), "capture_threads", 96)
            snapshot = camera.capture_snapshot(*tuple(self.scene.mobjects))
            if not isinstance(snapshot, self._native._CameraCapture):
                raise TypeError("paired output requires a native immutable camera capture")
            if tuple(snapshot.size) != self._config.resolution:
                raise ValueError("final PNG dimensions differ from the selected render resolution")
            if self._state != "active":
                raise RuntimeError("paired output was cancelled during final capture")
            self._pending = snapshot.prepare_png(self.still_destination, threads=capture_threads)
            self.primary.finish()
            if self._state != "active":
                raise RuntimeError("paired output was cancelled during primary finalization")
            if self.primary.result is None:
                raise RuntimeError("primary output finished without a complete receipt")
            receipt = self._pending.commit()
            self._still_published = True
            path, size, digest = receipt
            self._still = rendering.RenderResult(
                destination=Path(path), format="png", resolution=self._config.resolution,
                fps=self._config.fps, threads=capture_threads, engine="camera-capture/fast-cpu",
                bytes=int(size), digest=digest, frame_count=1, sample_frames=None,
                seed=self._config.seed, certified=False,
                animation_range=self._config.animation_range,
            )
            self._state = "finished"
            self.result = self.partial_result
            return self.result
        except BaseException as error:
            self._fail(error)
            raise
        finally:
            self._pending = None
            self._finishing = False
            self._release()

    def __exit__(self, exc_type, error, traceback):
        if error is not None:
            self._fail(error)
        elif self._state == "active":
            self.finish()
        return False


def paired_render_session(scene, destination, still_destination, *, format=None,
                          resolution=None, fps=None, threads=None, animation_range=None):
    """Record imperative scene calls to one animation output plus one final PNG."""
    destination, still_destination = _path(destination), _path(still_destination)
    primary = rendering.render_session(scene, destination, format=format,
        resolution=resolution, fps=fps, threads=threads, animation_range=animation_range)
    return PairedRenderSession(primary, still_destination)


def render_scene_with_still(scene, destination, still_destination, *, scene_kwargs=None,
                            format=None, resolution=None, fps=None, threads=None,
                            animation_range=None, subdivide=False, max_segments=10_000,
                            _output_options=None):
    """Execute a scene exactly once and return the receipts of both artifacts."""
    native = importlib.import_module("manimlib")
    # Freeze caller paths and admit the whole mode before an authored constructor.
    destination, still_destination = _path(destination), _path(still_destination)
    if format is None:
        format = destination.suffix.lower().lstrip(".") or "png_sequence"
    validate_paired_mode(native, format)
    options = dict(format=format, resolution=resolution, fps=fps, threads=threads,
                   animation_range=animation_range)
    if not isinstance(subdivide, bool):
        raise TypeError("subdivide must be bool")
    if subdivide:
        max_segments = rendering._positive_integer(max_segments, "max_segments", 100_000)
    elif max_segments != 10_000:
        raise ValueError("max_segments requires subdivide=True")
    if isinstance(scene, type) and issubclass(scene, native.Scene):
        scene = scene(**({} if scene_kwargs is None else dict(scene_kwargs)))
    elif scene_kwargs is not None:
        raise TypeError("scene_kwargs is valid only when rendering a Scene class")
    if _output_options:
        rendering._apply_output_options(scene, _output_options)
    if subdivide:
        from .subdivision_rendering import subdivided_render_session
        primary = subdivided_render_session(scene, destination, max_segments=max_segments, **options)
        session = PairedRenderSession(primary, still_destination)
    else:
        session = paired_render_session(scene, destination, still_destination, **options)
    return rendering._run_owned_scene_render(scene, session, native)


def install_paired_output(native):
    """Honor both stock writer preferences only for implicit Scene output.

    Explicit ordinary render_scene/render_session calls continue selecting one
    artifact. The public paired functions select two explicitly. Capture keeps
    its standard-mode identity and cannot inherit a certified primary claim.
    """
    if vars(native).get("_FMN_PAIRED_OUTPUT_INSTALLED", False):
        return
    original = rendering._configured_scene_session

    @wraps(original)
    def configured(scene, destination, format, resolution, fps, threads,
                   animation_range, owner, **options):
        writer = getattr(scene, "file_writer", None)
        if (destination is None and format is None and writer is not None
                and writer.write_to_movie and writer.save_last_frame):
            movie = _path(writer.get_movie_file_path())
            still = _path(writer.get_image_file_path())
            format = movie.suffix.lower().lstrip(".")
            destination = (_path(writer.get_output_file_rootname()) / "clips"
                           if writer.subdivide_output else movie)
            primary = original(scene, destination, format, resolution, fps,
                               threads, animation_range, owner, **options)
            return PairedRenderSession(primary, still)
        return original(scene, destination, format, resolution, fps,
                        threads, animation_range, owner, **options)

    rendering._configured_scene_session = configured
    native._FMN_PAIRED_OUTPUT_INSTALLED = True
