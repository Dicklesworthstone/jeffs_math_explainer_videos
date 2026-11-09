"""The wheel's write-all front door over the shared multi-scene renderer."""
from __future__ import annotations

import contextlib
import sys
from pathlib import Path
from typing import Any

from .batch_provenance import validate_batch_mode
from .effect_audit import recording as recording_effects
from .batch_rendering import BatchRenderError, BatchRenderResult, _error_fields, render_scenes
from .rendering import _positive_integer
from .scene_loading import SceneSource
from .render_selection import PLAYBACK_HELP, take_playback_options, select_still_format

_VALUE_FLAGS = frozenset({"--format", "--resolution", "--fps", "--threads", "--video_dir",
                          "--vcodec", "--pix_fmt", "--ffmpeg_bin",
                          "-n", "--start_at_animation_number"})
_BATCH_HELP = """Multi-scene output:
  fmn-python [--robot] SOURCE.py --write_all [--keep-going]
             [--format png|png_sequence|gif|y4m|wav|svg|mp4|mov]
             [--resolution WIDTHxHEIGHT] [--fps FPS] [--threads N]
             [--video_dir DIRECTORY] [--reproducible]

--write_all renders locally declared scenes in sorted name order. DIRECTORY
is the batch root, not a single output file. --keep-going continues after
ordinary scene failures; any failed scene still makes the command exit 5.
Completed artifacts are retained. Ctrl-C stops with exit 130 and progress.
Robot stdout contains one aggregate receipt; ordinary scene print output and
per-scene progress go to stderr. Scenes run sequentially; --threads controls
each native renderer, not Python scene concurrency. Output is uncertified by
default. --reproducible supports PNG stills/sequences and WAV with independent
per-scene native manifests, source providers and a fixed runtime generation.
The aggregate remains uncertified; all_scenes_certified summarizes completed
per-scene receipts. Checkpoint/resume is excluded from this mode.
"""


def _switches(arguments: list[str]):
    flags = {"--write_all": 0, "--keep-going": 0, "--robot": 0}
    remaining, index = [], 0
    while index < len(arguments):
        value = arguments[index]
        if value in _VALUE_FLAGS:
            remaining.extend(arguments[index:index + 2])
            index += 2
            continue
        if value in flags:
            flags[value] += 1
        else:
            remaining.append(value)
        index += 1
    return flags, remaining


@contextlib.contextmanager
def _source_import_path(source: Path):
    # Keep sibling modules importable during both source loading and lazy
    # imports inside construct(). Remove only our own entry afterward.
    entry = str(source.parent)
    sys.path.insert(0, entry)
    try:
        yield
    finally:
        for index, value in enumerate(sys.path):
            if value is entry:
                del sys.path[index]
                break


def _emit_result(native: Any, report: BatchRenderResult, robot: bool, source: str, directory: Path,
                 *, interrupted: bool = False, interrupt_code: int = 130) -> int:
    counts = report.counts
    code = interrupt_code if interrupted else (0 if report.ok else 5)
    identity = "interrupted" if interrupted else ("success" if report.ok else "scene")
    message = (f"{counts['succeeded']} succeeded, {counts['failed']} failed, "
               f"{counts['cancelled']} cancelled, {counts['not_run']} not run")
    return native._portal_cli_emit(
        code, identity, "render-batch-interrupted" if interrupted else "render-batch",
        message, robot, source=source, destination=str(directory),
        all_succeeded=report.ok, batch=report.as_dict(),
    )


def try_batch_cli(native: Any, arguments: list[str]) -> int | None:
    """Handle write-all/help; None delegates other commands unchanged."""
    flags, remaining = _switches(arguments)
    requested = bool(flags["--write_all"] or flags["--keep-going"])
    help_only = remaining in (["--help"], ["-h"])
    if not requested and not help_only:
        return None
    robot = bool(flags["--robot"])
    if any(count > 1 for count in flags.values()):
        return native._portal_cli_emit(2, "usage", "usage-error", "batch switches must not be repeated", robot)
    if help_only:
        text = native._portal_cli_help().replace(
            "Certified output, opener flags, write-all, and Studio",
            "Certified output, opener flags, and Studio",
        ) + "\n\n" + _BATCH_HELP + "\n" + PLAYBACK_HELP
        if robot:
            return native._portal_cli_emit(0, "success", "help", "fmn-python usage", True, help=text)
        print(text)
        return 0
    if not flags["--write_all"]:
        return native._portal_cli_emit(2, "usage", "usage-error", "--keep-going requires --write_all", robot)
    try:
        remaining, selection, still = take_playback_options(remaining)
        positionals, options, width, height, fps, threads = native._portal_cli_render_arguments(remaining)
        options = select_still_format(options, remaining, still)
        if len(positionals) != 1:
            raise ValueError("--write_all accepts SOURCE.py without an individual scene name")
        for name, value in (("width", width), ("height", height), ("fps", fps), ("threads", threads)):
            if value is not None:  # fps None: the scene camera's fps
                _positive_integer(value, name)
        source = positionals[0]
        source_path = Path(source).resolve()
        directory = options["video_dir"]
        if directory is None:
            directory = Path("media") / "videos" / source_path.stem
        if not str(directory) or "\0" in str(directory):
            raise ValueError("batch output directory must be nonempty and contain no NUL")
        # Source module code, not just constructors, may change cwd.
        directory = Path(directory).resolve()
        validate_batch_mode(native, options["format"], bool(options.get("reproducible")))
    except RuntimeError as error:
        message = _error_fields(error)[1]
        if message.startswith("CAPABILITY: "):
            return native._portal_cli_emit(4, "capability", "render-capability-unavailable",
                                           message.removeprefix("CAPABILITY: "), robot)
        return native._portal_cli_emit(2, "usage", "usage-error", message, robot)
    except (TypeError, ValueError, OSError) as error:
        return native._portal_cli_emit(2, "usage", "usage-error", _error_fields(error)[1], robot)

    def progress(outcome):
        print(f"fmn-python: {outcome.name}: {outcome.status}: {outcome.destination}", file=sys.stderr)

    # Scope redirection to authored source/scene execution, never the receipt.
    # It is bounded in memory (streamed), not an unbounded StringIO capture.
    redirect = contextlib.redirect_stdout(sys.stderr) if robot else contextlib.nullcontext()
    phase = "load"
    try:
        with (redirect, recording_effects(bool(options.get("reproducible"))),
              SceneSource(source_path, native.Scene) as loaded):
            discovered = loaded.scenes
            if not discovered:
                raise ValueError(f"no locally declared Scene classes found in {source}")
            phase = "render"
            report = render_scenes(
                {name: discovered[name] for name in sorted(discovered)}, directory,
                format=options["format"], resolution=(width, height), fps=fps, threads=threads,
                continue_on_error=bool(flags["--keep-going"]), on_result=progress,
                **({} if selection is None else {"animation_range": selection}),
                **({"reproducible": True, "sources": lambda: loaded.sources}
                   if options.get("reproducible") else {}),
            )
    except BatchRenderError as error:
        return _emit_result(native, error.result, robot, source, directory)
    except (KeyboardInterrupt, SystemExit) as error:
        code = 130 if isinstance(error, KeyboardInterrupt) else 5
        report = getattr(error, "render_batch_result", None)
        if report is not None:
            return _emit_result(native, report, robot, source, directory, interrupted=True, interrupt_code=code)
        return native._portal_cli_emit(code, "interrupted", "render-batch-interrupted",
                                       _error_fields(error)[1] or type(error).__name__, robot, source=source)
    except Exception as error:
        report = getattr(error, "render_batch_result", None)
        if report is not None:
            return native._portal_cli_emit(6, "render", "render-batch-reporting-failed",
                                           _error_fields(error)[1], robot, source=source,
                                           destination=str(directory), batch=report.as_dict())
        # Batch execution errors normally carry a report. Preflight/load
        # failures have no successful scene to imply or fabricate.
        if phase == "load":
            code, identity, kind = 5, "scene", "scene-load-failed"
        elif _error_fields(error)[1].startswith("CAPABILITY: "):
            code, identity, kind = 4, "capability", "render-capability-unavailable"
        elif isinstance(error, OSError):
            code, identity, kind = 6, "render", "render-start-failed"
        else:
            code, identity, kind = 2, "usage", "usage-error"
        return native._portal_cli_emit(code, identity, kind, _error_fields(error)[1], robot,
                                       source=source, destination=str(directory))
    return _emit_result(native, report, robot, source, directory)
