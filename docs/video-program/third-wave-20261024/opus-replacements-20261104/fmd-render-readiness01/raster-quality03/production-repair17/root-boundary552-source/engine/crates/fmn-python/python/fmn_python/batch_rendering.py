"""Ordered multi-scene export over the existing native render-session owner.

Scenes run sequentially on the calling thread; each native renderer still uses
its requested thread budget. This is not the native ``fmn batch`` farm and
never moves a Python scene, its proxies, or a live render across threads.
"""
from __future__ import annotations

import importlib
import os
import unicodedata
from collections.abc import Callable, Iterable, Mapping
from contextlib import nullcontext
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .batch_checkpoint import BatchCheckpoint
from .batch_provenance import BatchProvenance, has_provenance, validate_batch_mode
from .bundle_export import BundleExportResult, _validated_limits, export_bundle, require_bundle_capability
from .render_selection import animation_range as _animation_range
from .rendering import RenderResult, SourceInputs, _FORMATS, _positive_integer, render_scene
from .paired_output import (
    PairedRenderResult, companion_png_path, render_scene_with_still, validate_paired_mode,
)
from .subdivision import SubdividedRenderResult
from .subdivision_rendering import render_subdivided_scene, validate_subdivided_mode


@dataclass(frozen=True)
class RenderJob:
    """A named Scene class or pristine instance, with optional class arguments."""

    name: str
    scene: Any
    scene_kwargs: Mapping[str, Any] | None = None


@dataclass(frozen=True)
class SceneRenderOutcome:
    """One planned destination and its actual execution/publication outcome."""

    name: str
    destination: Path
    status: str
    result: RenderResult | SubdividedRenderResult | PairedRenderResult | BundleExportResult | None = None
    error_type: str | None = None
    message: str | None = None
    notes: tuple[str, ...] = ()

    def as_dict(self) -> dict[str, Any]:
        return {
            "name": self.name, "destination": str(self.destination),
            "status": self.status,
            "result": None if self.result is None else self.result.as_dict(),
            "error": None if self.error_type is None else {
                "type": self.error_type, "message": self.message, "notes": list(self.notes),
            },
        }


@dataclass(frozen=True)
class BatchRenderResult:
    """Ordered receipts, errors, and unattempted jobs; never all-or-nothing."""

    outcomes: tuple[SceneRenderOutcome, ...]
    reproducible: bool = False

    @property
    def all_scenes_certified(self) -> bool:
        return self.reproducible and self.ok and all(has_provenance(item.result) for item in self.outcomes)

    @property
    def ok(self) -> bool:
        return bool(self.outcomes) and all(item.status == "succeeded" for item in self.outcomes)

    @property
    def counts(self) -> dict[str, int]:
        return {status: sum(item.status == status for item in self.outcomes)
                for status in ("succeeded", "failed", "cancelled", "not_run")}

    def as_dict(self) -> dict[str, Any]:
        return {
            "schema": "fmn-python.render-batch", "version": 1,
            "ok": self.ok, "execution": "sequential", "certified": False,
            "reproducible_requested": self.reproducible,
            "all_scenes_certified": self.all_scenes_certified,
            "counts": self.counts,
            "outcomes": [item.as_dict() for item in self.outcomes],
        }


class BatchRenderError(RuntimeError):
    """Fail-fast execution stopped; ``result`` retains every completed receipt."""

    def __init__(self, result: BatchRenderResult):
        self.result = result
        failed = next(item for item in reversed(result.outcomes) if item.status == "failed")
        super().__init__(f"render batch stopped at {failed.name}: {failed.error_type}: {failed.message}")


def _name_key(name: Any) -> str:
    if not isinstance(name, str) or not name.isidentifier() or len(name.encode("utf-8")) > 128:
        raise ValueError("render job names must be identifiers of at most 128 UTF-8 bytes")
    # Reject names that collide on common case-insensitive/normalizing file
    # systems, including Windows device names, before starting any scene.
    key = unicodedata.normalize("NFKC", name).casefold()
    if key in {"con", "prn", "aux", "nul", "conin$", "conout$"} or (
        len(key) == 4 and key[:3] in {"com", "lpt"} and key[3] in "123456789"
    ):
        raise ValueError(f"render job name {name!r} is a reserved filesystem name")
    return key


def _plan(scenes: Any, directory: Any, format: str, native: Any, maximum: int, existing=frozenset(), *, subdivide=False):
    text = os.fspath(directory)
    if not isinstance(text, str):
        raise TypeError("batch output directory must be a text path")
    if not text or "\0" in text:
        raise ValueError("batch output directory must be nonempty and contain no NUL")
    # Freeze relative paths before authored constructors/run hooks can chdir.
    root = Path(text).resolve()
    if root.exists() and not root.is_dir():
        raise NotADirectoryError(str(root))
    if isinstance(scenes, Mapping):
        entries = (RenderJob(name, scene) for name, scene in scenes.items())
    elif isinstance(scenes, (str, bytes)) or isinstance(scenes, native.Scene) or isinstance(scenes, type):
        raise TypeError("render_scenes expects a mapping or an iterable of scenes/RenderJob entries")
    else:
        entries = iter(scenes)
    jobs, destinations, names, instances = [], [], set(), set()
    for index, entry in enumerate(entries):
        if index >= maximum:
            raise ValueError(f"render batch exceeds max_jobs={maximum}")
        if isinstance(entry, RenderJob):
            job = entry
        else:
            job = RenderJob(entry.__name__ if isinstance(entry, type) else type(entry).__name__, entry)
        key = _name_key(job.name)
        if key in names:
            raise ValueError(f"render job name collision: {job.name!r}")
        names.add(key)
        is_class = isinstance(job.scene, type) and issubclass(job.scene, native.Scene)
        if not is_class and not isinstance(job.scene, native.Scene):
            raise TypeError(f"render job {job.name!r} must contain a Scene class or instance")
        if not is_class:
            if id(job.scene) in instances:
                raise ValueError("a Scene instance cannot be used by two render jobs; supply its class instead")
            instances.add(id(job.scene))
        kwargs = job.scene_kwargs
        if kwargs is not None:
            if not is_class:
                raise TypeError("scene_kwargs is valid only for a Scene class")
            if not isinstance(kwargs, Mapping) or any(not isinstance(key, str) for key in kwargs):
                raise TypeError("scene_kwargs must be a mapping with string keys")
            kwargs = dict(kwargs)
        destination = (root / job.name / "clips" if subdivide else
                       root / job.name / "frames" if format == "png_sequence" else root / (job.name + "." + format))
        # This is an early diagnostic, not a replacement for Reel's atomic
        # no-clobber check (another process may create a destination later).
        if existing is not None and os.path.lexists(destination) and destination not in existing:
            raise FileExistsError(f"render destination already exists: {destination}")
        for parent in destination.parents:
            if parent.exists() and not parent.is_dir():
                raise NotADirectoryError(str(parent))
        jobs.append(RenderJob(job.name, job.scene, kwargs))
        destinations.append(destination)
    if not jobs:
        raise ValueError("render batch must contain at least one scene")
    return jobs, destinations


def _error_fields(error: BaseException) -> tuple[str, str]:
    try:
        message = str(error)
    except BaseException:
        message = "exception message could not be formatted"
    return type(error).__name__[:128], message[:4096]


def _error_notes(error: BaseException) -> tuple[str, ...]:
    # RenderSession notes distinguish a failed cancellation from the primary
    # error, and publication succeeded but provenance retrieval failed.
    try:
        notes = vars(error).get("__notes__", ())
    except BaseException:
        return ()
    if not isinstance(notes, (tuple, list)):
        return ()
    return tuple(note[:1024] for note in notes[:8] if isinstance(note, str))


def _attach_result(error: BaseException, outcomes: list[SceneRenderOutcome], *, reproducible: bool = False) -> None:
    # Preserve KeyboardInterrupt/SystemExit and authored observer exceptions
    # rather than translating cancellation into an ordinary failed scene.
    try:
        error.render_batch_result = BatchRenderResult(tuple(outcomes), reproducible=reproducible)
    except BaseException:
        pass


def render_scenes(
    scenes: Mapping[str, Any] | Iterable[Any], directory: os.PathLike[str] | str, *,
    format: str = "png_sequence", resolution: tuple[int, int] | None = None,
    fps: int | None = None, threads: int | None = None,
    continue_on_error: bool = False,
    on_result: Callable[[SceneRenderOutcome], Any] | None = None,
    max_jobs: int = 1024,
    subdivide: bool = False, max_segments: int = 10_000,
    save_last_frame: bool = False,
    animation_range: tuple[int, int | None] | None = None,
    checkpoint: os.PathLike[str] | str | None = None,
    resume: bool = False, resume_key: str | None = None,
    _output_options: dict[str, Any] | None = None,
    reproducible: bool = False,
    sources: SourceInputs | None = None,
    runtime_identities: dict[str, str] | None = None,
    bundle_camera: bool = False,
    bundle_limits: Mapping[str, int] | None = None,
) -> BatchRenderResult:
    """Render named scenes in input order using independent native sessions.

    Mappings provide names; iterables accept Scene classes, pristine instances,
    or RenderJob(name, scene, scene_kwargs). All jobs and common options are
    validated before constructing a scene. Native publication remains atomic
    per artifact, NOT for the batch. Completed artifacts are never rolled back.

    subdivide=True emits each selected play/wait into directory/name/clips
    through the same native per-call recording owner. max_segments is the
    completed-clip budget per scene. Successful outcomes carry collection
    receipts; failed/cancelled outcomes retain partial collection receipts in
    result, without claiming whole-scene success. This mode does not accept
    still-only output, reproducibility, or single-artifact checkpoint/resume. All
    selection, name/collision and mode checks precede scene construction.

    save_last_frame=True composes each primary with one prepared native PNG,
    including subdivided output. File primaries use stem.png; directories use
    adjacent directory-name.png. Both paths for every job are preflighted.
    Outcomes retain complete/partial paired receipts. With checkpoint/resume,
    non-subdivided jobs bind and hash-verify BOTH outputs before a completed
    scene is skipped. Incomplete pairs remain no-clobber failures, not reusable
    successes. Single-artifact plans cannot be reused as paired plans. Neither
    mode inherits certification from another output.

    By default the first ordinary failure raises BatchRenderError with partial
    receipts and the original cause. continue_on_error returns a report with
    failed outcomes instead. KeyboardInterrupt/SystemExit always propagate;
    render_batch_result on the exception preserves progress. on_result runs
    after each outcome and cannot turn a published artifact into a failure.
    Observer exceptions stop the batch and carry the same progress attribute.

    checkpoint writes an atomic progress journal after each native publication,
    before observers run. Supply a nonempty resume_key identifying scene/asset
    inputs. With resume=True, matching completed artifacts are hash-verified
    and reused; failed, cancelled and unattempted jobs run afresh. Reused scenes
    are not constructed. Checkpoint constructor kwargs must be plain JSON
    values; each job receives an independent copy of its admitted inputs. Reuse
    is explicit, uncertified, and not a source-code cache:
    change resume_key when inputs for completed scenes change. A crash between
    publication and checkpointing leaves an unrecorded artifact which fails
    no-clobber preflight; it is never silently reused, deleted, or overwritten.

    reproducible uses the existing certified session for each scene, with a
    separate native manifest. Supply shared compilation bytes in sources, or
    a provider such as lambda: loaded.sources evaluated after each scene runs.
    Runtime identity is held fixed across the batch. All artifacts/sidecars are
    preflighted before construction; successful scenes survive later failures.
    Checkpoint/resume is excluded because its receipts verify artifacts only.
    The aggregate is not itself a certified artifact; all_scenes_certified
    reports whether every selected scene returned a complete certified receipt.

    format="fmtl" captures independent code-free scene bundles through the same
    job planner and ordered execution loop. bundle_camera=True records each
    frame's camera/light/background for native 3D/raster replay; False retains
    the planar/browser-compatible format. bundle_limits may set max_frames,
    max_capture_bytes and max_output_bytes per scene. Limits are frozen and
    validated before any scene constructor runs. Completed bundles survive
    later failures. Pixel-output options, threads, partial playback, paired or
    subdivided outputs and certification are not applicable to this mode and
    refuse before constructing any scene. Checkpoint/resume retains the same
    caller-owned resume_key contract: completed bundles are hash-verified and
    skipped, never reconstructed as live Scenes. Camera mode, capture limits,
    explicit resolution/FPS and constructor inputs must match the saved plan.
    """
    if not isinstance(format, str) or format not in _FORMATS | {"fmtl"}:
        raise ValueError("render format must be png, png_sequence, gif, y4m, wav, svg, mp4, mov, or fmtl")
    if not isinstance(bundle_camera, bool):
        raise TypeError("bundle_camera must be bool")
    if format != "fmtl" and (bundle_camera or bundle_limits is not None):
        raise ValueError("bundle_camera and bundle_limits require format='fmtl'")
    if not isinstance(save_last_frame, bool):
        raise TypeError("save_last_frame must be bool")
    if not isinstance(subdivide, bool):
        raise TypeError("subdivide must be bool")
    if subdivide:
        max_segments = _positive_integer(max_segments, "max_segments", 100_000)
    elif max_segments != 10_000:
        raise ValueError("max_segments requires subdivide=True")
    if not isinstance(resume, bool):
        raise TypeError("resume must be bool")
    if checkpoint is None and (resume or resume_key is not None):
        raise ValueError("resume and resume_key require a checkpoint path")
    if not isinstance(continue_on_error, bool):
        raise TypeError("continue_on_error must be bool")
    if on_result is not None and not callable(on_result):
        raise TypeError("on_result must be callable or None")
    maximum = _positive_integer(max_jobs, "max_jobs", 65536)
    selection = _animation_range(animation_range)
    if resolution is not None:
        try:
            width, height = resolution
        except (TypeError, ValueError):
            raise ValueError("render resolution must contain width and height") from None
        resolution = (_positive_integer(width, "width"), _positive_integer(height, "height"))
    fps = None if fps is None else _positive_integer(fps, "fps")
    threads = None if threads is None else _positive_integer(threads, "threads")
    native = importlib.import_module("manimlib")
    limits = None
    if format == "fmtl":
        require_bundle_capability(native, camera=bundle_camera)
        if (threads is not None or selection is not None or subdivide or save_last_frame
                or reproducible or sources is not None
                or runtime_identities is not None or _output_options):
            raise native._CapabilityError(
                "FMTL batches require complete scene capture without pixel-output options, "
                "threads, subdivision, paired output or source certification"
            )
        if ((resolution is not None and resolution[0] * resolution[1] > 16_777_216)
                or (fps is not None and fps > 240)):
            raise ValueError("bundle export requires at most 16M pixels and 1..240 FPS")
        if bundle_limits is not None and not isinstance(bundle_limits, Mapping):
            raise TypeError("bundle_limits must be a mapping or None")
        limits = _validated_limits(**({} if bundle_limits is None else dict(bundle_limits)))
    if subdivide and ((resolution is not None and resolution[0] * resolution[1] > 16_777_216)
                      or (threads is not None and threads > 96)):
        raise ValueError("subdivision requires at most 16777216 pixels and 1..96 threads")
    if subdivide:
        validate_subdivided_mode(native, format, reproducible=reproducible, checkpoint=checkpoint)
    if save_last_frame:
        validate_paired_mode(native, format, reproducible=reproducible, checkpoint=checkpoint)
    validate_batch_mode(native, format, reproducible, checkpoint)
    owner = nullcontext(None) if checkpoint is None else BatchCheckpoint(checkpoint, resume=resume, key=resume_key)
    jobs, destinations = _plan(scenes, directory, format, native, maximum, None, subdivide=subdivide)
    still_destinations = ([companion_png_path(path, format, subdivide=subdivide) for path in destinations]
                          if save_last_frame else [])
    if checkpoint is not None:
        owner.validate_destinations([*destinations, *still_destinations])
    with owner as journal:
        existing = frozenset() if journal is None else journal.existing_destinations
        for destination in destinations:
            if os.path.lexists(destination) and destination not in existing:
                raise FileExistsError(f"render destination already exists: {destination}")
        for still_destination in still_destinations:
            if os.path.lexists(still_destination) and still_destination not in existing:
                raise FileExistsError(f"final PNG destination already exists: {still_destination}")
        provenance = (BatchProvenance(native, format, sources, runtime_identities, destinations)
                      if reproducible else None)
        outcomes = [SceneRenderOutcome(job.name, path, "not_run") for job, path in zip(jobs, destinations)]
        completed = {}
        if journal is not None:
            journal.prepare(jobs, destinations, {
                "format": format, "resolution": resolution, "fps": fps,
                "threads": threads, "animation_range": selection,
                "output_options": _output_options,
                **({"save_last_frame": True} if save_last_frame else {}),
                **({"bundle_camera": bundle_camera, "bundle_limits": limits}
                   if format == "fmtl" else {}),
            })
            completed = journal.completed
            for index, job in enumerate(jobs):
                if job.name in completed:
                    outcomes[index] = _restored_outcome(completed[job.name])
            _record_checkpoint(journal, outcomes)
        for index, (job, path) in enumerate(zip(jobs, destinations)):
            failure = None
            if job.name not in completed:
                try:
                    provenance_options = {} if provenance is None else provenance.options(path)
                    render = (render_scene_with_still if save_last_frame else
                              render_subdivided_scene if subdivide else render_scene)
                    kwargs = job.scene_kwargs
                    if journal is not None and kwargs is not None:
                        kwargs = journal.constructor_kwargs(index)
                    if format == "fmtl":
                        receipt = export_bundle(job.scene, path, resolution=resolution, fps=fps,
                                                camera=bundle_camera, scene_kwargs=kwargs, **limits)
                    else:
                        receipt = render(job.scene, path, format=format, resolution=resolution,
                                           fps=fps, threads=threads, scene_kwargs=kwargs,
                                           **provenance_options,
                                           **({"still_destination": still_destinations[index], "subdivide": subdivide}
                                              if save_last_frame else {}),
                                           **({"max_segments": max_segments} if subdivide else {}),
                                           **({} if not _output_options else {"_output_options": _output_options}),
                                           **({} if selection is None else {"animation_range": selection}))
                    if provenance is not None:
                        provenance.accept(receipt, path)
                except Exception as error:
                    failure = error
                    kind, message = _error_fields(error)
                    outcomes[index] = SceneRenderOutcome(job.name, path, "failed", error_type=kind, message=message,
                                                        notes=_error_notes(error),
                                                        result=(_partial_pair(error, path) if save_last_frame else
                                                                _partial_subdivision(error, path) if subdivide else None))
                except BaseException as error:
                    kind, message = _error_fields(error)
                    outcomes[index] = SceneRenderOutcome(job.name, path, "cancelled", error_type=kind, message=message,
                                                        notes=_error_notes(error),
                                                        result=(_partial_pair(error, path) if save_last_frame else
                                                                _partial_subdivision(error, path) if subdivide else None))
                    _attach_result(error, outcomes, reproducible=reproducible)
                    try:
                        _record_checkpoint(journal, outcomes)
                    except BaseException as reporting_error:
                        error.add_note("batch checkpoint also failed: " + _error_fields(reporting_error)[1])
                    raise
                else:
                    outcomes[index] = SceneRenderOutcome(job.name, receipt.destination, "succeeded", result=receipt)
                _record_checkpoint(journal, outcomes)
            if on_result is not None:
                try:
                    on_result(outcomes[index])
                except BaseException as error:
                    _attach_result(error, outcomes, reproducible=reproducible)
                    raise
            if failure is not None and not continue_on_error:
                raise BatchRenderError(BatchRenderResult(tuple(outcomes), reproducible=reproducible)) from failure
        return BatchRenderResult(tuple(outcomes), reproducible=reproducible)


def _partial_pair(error, destination):
    """Keep this job's immutable partial pair, including a published primary."""
    try:
        result = vars(error).get("render_pair_result")
        if (isinstance(result, PairedRenderResult)
                and result.destination == destination and not result.completed):
            return result
    except BaseException:
        pass
    return None


def _partial_subdivision(error, destination):
    """Accept only a collection receipt for this job, not a live session owner."""
    try:
        result = vars(error).get("render_subdivision_result")
        if (isinstance(result, SubdividedRenderResult)
                and result.destination == destination and not result.completed):
            return result
    except BaseException:
        pass
    return None


def _record_checkpoint(journal, outcomes):
    if journal is not None:
        try:
            journal.record(outcomes)
        except BaseException as error:
            _attach_result(error, outcomes)
            raise


def _restored_outcome(row):
    """Restore data only, never a pickled Scene or executable Python object."""
    data = dict(row["result"])
    if data.get("schema") == "fmn.paired-render":
        from .paired_checkpoint import restore_pair_receipt
        result = restore_pair_receipt(data, destination=Path(row["destination"]))
        return SceneRenderOutcome(row["name"], result.destination, "succeeded", result=result)
    if data.get("format") == "fmtl":
        from .bundle_checkpoint import restore_bundle_receipt
        result = restore_bundle_receipt(data)
        return SceneRenderOutcome(row["name"], result.destination, "succeeded", result=result)
    for name in ("fps", "threads", "bytes", "seed", "frame_count", "sample_frames"):
        value = data.get(name)
        if value is None and name in {"frame_count", "sample_frames"}:
            continue
        if type(value) is not int or value < (1 if name in {"fps", "threads"} else 0):
            raise ValueError("invalid checkpoint receipt field: " + name)
    resolution = data.get("resolution")
    if not isinstance(resolution, list) or len(resolution) != 2 or any(type(value) is not int or value <= 0 for value in resolution):
        raise ValueError("invalid checkpoint receipt resolution")
    digest = data.get("digest")
    if not isinstance(digest, str) or len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        raise ValueError("invalid checkpoint receipt digest")
    if not isinstance(data.get("engine"), str) or not data["engine"]:
        raise ValueError("invalid checkpoint receipt engine")
    if data.get("format") == "wav":
        data.pop("sample_rate", None)
        data.pop("channels", None)
    data["destination"] = Path(data["destination"])
    data["resolution"] = tuple(resolution)
    for name in ("ffmpeg_invocations", "audio_inputs"):
        items = data.get(name, [])
        if not isinstance(items, list) or any(not isinstance(item, dict) for item in items):
            raise ValueError("invalid checkpoint receipt field: " + name)
        data[name] = tuple(items)
    if "animation_range" in data:
        data["animation_range"] = _animation_range(data["animation_range"])
    try:
        result = RenderResult(**data)
    except TypeError as error:
        raise ValueError("invalid checkpoint publication receipt") from error
    return SceneRenderOutcome(row["name"], result.destination, "succeeded", result=result)
