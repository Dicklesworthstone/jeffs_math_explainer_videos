"""Console rendering through the same generation owner as Scene.render.

The native parser remains the option authority; RenderSession owns cancellation
and publication. Authored source runs in the calling interpreter, with sibling
imports available and robot stdout reserved for the terminal receipt.
"""
from __future__ import annotations

import contextlib
import os
from pathlib import Path
import sys
from typing import Any

from .batch_cli import _BATCH_HELP, _VALUE_FLAGS, _emit_result
from .batch_provenance import validate_batch_mode
from .effect_audit import recording as recording_effects
from .checkpoint_cli import CHECKPOINT_HELP, CHECKPOINT_VALUES, take_checkpoint_options
from .batch_rendering import (
    BatchRenderError, BatchRenderResult, _error_fields, _error_notes,
    _name_key, render_scenes,
)
from .rendering import (
    RenderSession, _positive_integer, _apply_output_options, _runtime_identities,
)
from .scene_loading import SceneSource
from .typesetting import describe_receipt, typesetting_receipt
from .render_selection import PLAYBACK_HELP, take_playback_options, select_still_format

from .paired_output import (
    PAIRED_HELP, PairedRenderResult, PairedRenderSession, companion_png_path,
    take_last_frame_option, validate_paired_mode,
)
from .subdivision import SubdividedRenderResult
from .subdivision_rendering import (
    SUBDIVISION_HELP, subdivided_render_session, take_subdivision_option, validate_subdivided_mode,
)

from .video_cli import (
    VIDEO_HELP, VIDEO_VALUE_FLAGS, VIDEO_WRITER_OPTIONS, take_video_options, validate_video_format,
)

_VALUE_FLAGS = _VALUE_FLAGS | CHECKPOINT_VALUES | VIDEO_VALUE_FLAGS

_CONTROL_FLAGS = frozenset({"--version", "--list-scenes", "--construct-only", "--audit-parity"})
_SELECTION_FLAGS = frozenset({"--robot", "--write_all", "-a", "--keep-going"})
_MAX_SELECTED_SCENES = 1024
_SELECTION_HELP = """Named scene selection:
  fmn-python [--robot] SOURCE.py SCENE [SCENE ...] [--keep-going]

Named scenes render in command-line order; -a is an alias for --write_all.
All selected names and destinations are checked before any scene is constructed.
With multiple names, --video_dir is a batch root containing per-scene outputs.
With one scene, --video_dir retains its existing single-output destination meaning.
--keep-going requires multiple names or --write_all. It never ignores Ctrl-C.
"""

_OUTPUT_HELP = """Native output profiles:
  --transparent, -t       Preserve alpha in PNG, SVG or qtrle MOV.
  --vcodec ENCODER        Installed ffmpeg encoder name, or auto (video only).
  --pix_fmt FORMAT        Native wire: rgba, bgra, nv12/yuv420p, p010le.
  --ffmpeg_bin PATH       Video encoder or WAV input decoder; paths with spaces work.

Transparent MOV defaults to RGBA/qtrle. Explicit transparent video profiles
require rgba/bgra and qtrle/auto. MP4, GIF, y4m and WAV do not accept -t.
Output options apply to every selected scene without constructor changes.
Video remains uncertified. P010 is a 10-bit transport, not an HDR claim.
--format svg exports one final-state vector document, including native text
and math outlines, camera pan/zoom, flat paints, background and stroke order.
It does not rasterize or embed fonts. Depth, lighting, user clip planes,
per-vertex gradients and perspective-varying curves require PNG instead.
"""


def _output_overrides(options):
    """Validate output combinations after the existing parser/still selector."""
    result = {name: options[name] for name in ("vcodec", "pix_fmt", "ffmpeg_bin", "transparent")
              if options.get(name) is not None and options.get(name) is not False}
    result.update({name: options[name] for name in VIDEO_WRITER_OPTIONS
                   if options.get(name) is not None})
    format = options["format"]
    validate_video_format(format, result, transparent=bool(result.get("transparent")))
    if any(name in result for name in ("vcodec", "pix_fmt")) and format not in {"mp4", "mov"}:
        raise ValueError("--vcodec/--pix_fmt require mp4 or mov output")
    if "ffmpeg_bin" in result and format not in {"mp4", "mov", "wav"}:
        raise ValueError("--ffmpeg_bin requires mp4, mov, or wav output")
    if result.get("transparent"):
        if format not in {"png", "png_sequence", "svg", "mov"}:
            raise ValueError("--transparent requires png, png_sequence, svg, or mov output")
        if format == "mov":
            if result.get("pix_fmt", "rgba").lower() not in {"rgba", "rgba8", "bgra", "bgra8"}:
                raise ValueError("transparent video requires an rgba or bgra wire pixel format")
            if result.get("vcodec", "auto").lower() not in {"auto", "qtrle"}:
                raise ValueError("transparent video requires --vcodec qtrle or auto")
    executable = result.get("ffmpeg_bin")
    if executable is not None and Path(executable).name != executable:
        # Freeze explicit paths before authored imports/constructors can chdir.
        # Bare executable names remain the native governed locator's job.
        result["ffmpeg_bin"] = str(Path(executable).resolve())
    return result


def _tokens(arguments, omit=()):
    """Separate positional selectors without interpreting native option values."""
    options, positionals, switches = [], [], []
    index = 0
    while index < len(arguments):
        value = arguments[index]
        if value in _VALUE_FLAGS:
            options.extend(arguments[index:index + 2])
            index += 2
            continue
        if value.startswith("-"):
            if value not in omit:
                options.append(value)
            switches.append(value)
        else:
            positionals.append(value)
        index += 1
    return options, positionals, switches


def _message(error):
    name, message = _error_fields(error)
    return f"{name}: {message}"


def _single_result(native, result, robot, source, selected, typesetting=None):
    if isinstance(result, PairedRenderResult):
        return native._portal_cli_emit(
            0, "success", "render-paired",
            f"rendered {result.destination} and final PNG {result.still_destination}",
            robot, source=source, scene=selected, rendered=True,
            destination=str(result.destination), still_destination=str(result.still_destination),
            pair=result.as_dict(),
        )
    if isinstance(result, SubdividedRenderResult):
        samples = getattr(result, "sample_frames", None)
        count = samples if samples is not None else result.frame_count
        unit = "sample frames" if samples is not None else "frames"
        return native._portal_cli_emit(
            0, "success", "render-subdivided",
            f"rendered {len(result.segments)} clips ({count} {unit}) to {result.destination}",
            robot, source=source, scene=selected, rendered=True,
            destination=str(result.destination), frame_count=result.frame_count,
            sample_frames=samples, subdivision=result.as_dict(),
        )
    details = result.as_dict()
    # Preserve the established CLI frame_count field for WAV consumers while
    # also exposing the unambiguous native sample-frame count.
    count = result.sample_frames if result.format == "wav" else result.frame_count
    details["frame_count"] = count
    unit = "sample frames" if result.format == "wav" else "frames"
    manifest_part = f"; manifest {result.manifest}" if result.manifest else ""
    typeset_part = "" if typesetting is None else describe_receipt(typesetting)
    if typesetting is not None:
        details["typesetting"] = typesetting
    return native._portal_cli_emit(
        0, "success", "render",
        f"rendered {count} {result.format} {unit} to {result.destination}{manifest_part}{typeset_part}",
        robot, source=source, scene=selected, rendered=True, **details,
    )


def _partial_output(session, subdivide):
    if isinstance(session, PairedRenderSession):
        return {"pair": session.partial_result.as_dict()}
    if subdivide and session is not None:
        return {"subdivision": session.partial_result.as_dict()}
    return {}


def try_render_cli(native: Any, arguments: list[str]) -> int | None:
    """Own ordinary renders; leave non-render commands on their native route."""
    native_options, raw_positionals, switches = _tokens(arguments, _SELECTION_FLAGS)
    if (_CONTROL_FLAGS.intersection(switches)
            or (raw_positionals and raw_positionals[0] == "studio")):
        return None
    if not arguments or arguments == ["--robot"]:
        return None
    robot = switches.count("--robot") != 0
    write_all = switches.count("--write_all") + switches.count("-a")
    keep_going = switches.count("--keep-going")
    if max(switches.count("--robot"), write_all, keep_going) > 1:
        return native._portal_cli_emit(2, "usage", "usage-error", "render switches must not be repeated", robot)
    if not raw_positionals and native_options in (["--help"], ["-h"]):
        text = native._portal_cli_help().replace(
            "Certified output, opener flags, write-all, and Studio",
            "Certified output, opener flags, and Studio",
        ).replace("Certified output, opener flags, and Studio", "Certified output and opener flags")
        from .studio import _HELP as _STUDIO_HELP
        text += "\n\n" + _BATCH_HELP + "\n" + _SELECTION_HELP + "\n" + PLAYBACK_HELP + "\n" + _OUTPUT_HELP + "\n" + SUBDIVISION_HELP + "\n" + PAIRED_HELP + "\n" + CHECKPOINT_HELP + "\n" + _STUDIO_HELP
        text += "\n" + VIDEO_HELP
        if robot:
            return native._portal_cli_emit(0, "success", "help", "fmn-python usage", True, help=text)
        print(text)
        return 0
    batch = bool(write_all or len(raw_positionals) > 2)
    try:
        native_options, video_options = take_video_options(native_options, _VALUE_FLAGS)
        native_options, paired = take_last_frame_option(native_options, _VALUE_FLAGS)
        native_options, subdivide = take_subdivision_option(native_options, _VALUE_FLAGS)
        native_options, recovery = take_checkpoint_options(native_options, _VALUE_FLAGS)
        if recovery and not batch:
            raise ValueError("checkpoint recovery requires multiple scene names or --write_all")
        # Strip playback selection only. Delegate output options and their
        # values to the existing native parser, with exactly one source. Put
        # that source first so a missing trailing option value stays missing.
        native_options, selection, still = take_playback_options(native_options)
        parser_args = raw_positionals[:1] + native_options
        positionals, options, width, height, fps, threads = native._portal_cli_render_arguments(parser_args)
        options = select_still_format(options, native_options, still)
        options.update(video_options)
        output_options = _output_overrides(options)
        if paired:
            if still:
                raise ValueError("--save-last-frame cannot be combined with --skip_animations/-s")
            validate_paired_mode(native, options["format"],
                                reproducible=bool(options.get("reproducible")), checkpoint=recovery.get("checkpoint"))
        if subdivide:
            if recovery:
                raise ValueError("--subdivide cannot reuse single-artifact checkpoint receipts")
            if still:
                raise ValueError("--subdivide cannot be combined with --skip_animations")
            validate_subdivided_mode(native, options["format"],
                                    reproducible=bool(options.get("reproducible")))
        for name, value in (("width", width), ("height", height), ("fps", fps), ("threads", threads)):
            if value is not None:  # fps None: the scene camera's fps
                _positive_integer(value, name)
        source = positionals[0]
        requested = raw_positionals[1:]
        if write_all and requested:
            raise ValueError("--write_all/-a cannot be combined with explicit scene names")
        if keep_going and not batch:
            raise ValueError("--keep-going requires multiple scene names or --write_all")
        if len(requested) > _MAX_SELECTED_SCENES:
            raise ValueError(f"scene selection exceeds {_MAX_SELECTED_SCENES} names")
        if batch:
            keys = [_name_key(name) for name in requested]
            if len(set(keys)) != len(keys):
                raise ValueError("selected scene names collide; each output must be unique")
        selected = requested[0] if len(requested) == 1 else None
        source_path = Path(source).resolve()
        supplied = options["video_dir"]
        if supplied is not None and (not str(supplied) or "\0" in str(supplied)):
            raise ValueError("render destination must be nonempty and contain no NUL")
        # Freeze both paths before module code or a constructor changes cwd.
        destination = None if supplied is None else Path(supplied).resolve()
        default_root = (Path("media") / "videos" / source_path.stem).resolve()
        if subdivide and not batch and destination is not None and os.path.lexists(destination):
            raise FileExistsError(f"subdivision destination already exists: {destination}")
        if batch:
            validate_batch_mode(native, options["format"], bool(options.get("reproducible")),
                                recovery.get("checkpoint"))
        if options.get("reproducible"):
            if getattr(native, "_portal_publish_manifest", None) is None:
                raise RuntimeError(
                    "CAPABILITY: certified portal rendering awaits the complete "
                    "content-hashed input closure and provenance sidecar"
                )
            if not source_path.is_file():
                raise RuntimeError(
                    f"CAPABILITY: portal input closure cannot be established: scene source does not exist: {source_path}"
                )
            if destination is not None and not batch:
                sidecar = destination.parent / (destination.name + ".manifest")
                if os.path.lexists(sidecar):
                    raise FileExistsError(
                        f"manifest destination {sidecar} already exists; sidecars are no-clobber generations"
                    )
    except RuntimeError as error:
        message = _error_fields(error)[1]
        if message.startswith("CAPABILITY: "):
            return native._portal_cli_emit(4, "capability", "render-capability-unavailable",
                                           message.removeprefix("CAPABILITY: "), robot)
        return native._portal_cli_emit(2, "usage", "usage-error", message, robot)
    except (TypeError, ValueError, OSError) as error:
        return native._portal_cli_emit(2, "usage", "usage-error", _message(error), robot)

    phase, session, report = "load", None, None
    redirect = contextlib.redirect_stdout(sys.stderr) if robot else contextlib.nullcontext()
    try:
        # A certified invocation records scene effects from before its
        # module loads: import-time reads are inputs of every render.
        with (redirect, recording_effects(bool(options.get("reproducible"))),
              SceneSource(source_path, native.Scene) as loaded):
            scenes = loaded.scenes
            names = sorted(scenes)
            if batch:
                requested = names if write_all else requested
                if not requested:
                    raise ValueError(f"no locally declared Scene classes found in {source}")
                missing = [name for name in requested if name not in scenes]
                if missing:
                    raise ValueError("selected scenes were not declared by " + source + ": " + ", ".join(missing))
                destination = default_root if destination is None else destination
                phase = "batch"
                report = render_scenes(
                    {name: scenes[name] for name in requested}, destination,
                    format=options["format"], resolution=(width, height), fps=fps, threads=threads,
                    continue_on_error=bool(keep_going), max_jobs=_MAX_SELECTED_SCENES,
                    **({"subdivide": True} if subdivide else {}),
                    **({"save_last_frame": True} if paired else {}),
                    **({} if selection is None else {"animation_range": selection}),
                    **({} if not output_options else {"_output_options": output_options}),
                    **({"reproducible": True, "sources": lambda: loaded.sources}
                       if options.get("reproducible") else {}),
                    **recovery,
                    on_result=lambda outcome: print(
                        f"fmn-python: {outcome.name}: {outcome.status}: {outcome.destination}",
                        file=sys.stderr,
                    ),
                )
            else:
                if selected is None:
                    if len(names) != 1:
                        raise ValueError("select one scene explicitly; discovered: " + (", ".join(names) or "none"))
                    selected = names[0]
                if selected not in scenes:
                    raise ValueError(f"scene {selected!r} was not declared by {source}; discovered: "
                                     + (", ".join(names) or "none"))
                if destination is None:
                    destination = (default_root / selected / "clips" if subdivide else
                                   default_root / selected / "frames" if options["format"] == "png_sequence"
                                   else default_root / (selected + "." + options["format"]))
                if options.get("reproducible"):
                    sidecar = destination.parent / (destination.name + ".manifest")
                    if os.path.lexists(sidecar):
                        raise FileExistsError(
                            f"manifest destination {sidecar} already exists; sidecars are no-clobber generations"
                        )
                if paired:
                    still_destination = companion_png_path(destination, options["format"], subdivide=subdivide)
                    if os.path.lexists(still_destination):
                        phase = "start"
                        raise FileExistsError(f"final PNG destination already exists: {still_destination}")
                phase = "construct"
                scene = scenes[selected]()
                phase = "start"
                _apply_output_options(scene, output_options)
                if subdivide:
                    session = subdivided_render_session(
                        scene, destination, format=options["format"],
                        resolution=(width, height), fps=fps, threads=threads,
                        animation_range=selection, _native=native,
                    )
                else:
                    session = RenderSession(
                        scene, destination, format=options["format"],
                        resolution=(width, height), fps=fps, threads=threads,
                        animation_range=selection,
                        reproducible=bool(options.get("reproducible")),
                        sources=(lambda: loaded.sources) if options.get("reproducible") else None,
                        runtime_identities=_runtime_identities(native),
                        _native=native,
                    )
                if paired:
                    session = PairedRenderSession(session, still_destination)
                with session:
                    phase = "execute"
                    try:
                        scene.run()
                    except native.EndScene:
                        pass
                    phase = "finish"
                if session.result is None:
                    raise RuntimeError("scene execution ended without publishing its render generation")
    except BatchRenderError as error:
        return _emit_result(native, error.result, robot, source, destination)
    except (KeyboardInterrupt, SystemExit) as error:
        code = 130 if isinstance(error, KeyboardInterrupt) else 5
        partial = getattr(error, "render_batch_result", None)
        if phase == "batch" and isinstance(partial, BatchRenderResult):
            return _emit_result(native, partial, robot, source, destination,
                                interrupted=True, interrupt_code=code)
        return native._portal_cli_emit(code, "interrupted", "render-interrupted", _message(error), robot,
                                       source=source, scene=selected, phase=phase,
                                       destination=None if destination is None else str(destination),
                                       notes=list(_error_notes(error)),
                                       artifact_published=session is not None and session.artifact_published,
                                       **_partial_output(session, subdivide))
    except Exception as error:
        partial = getattr(error, "render_batch_result", None)
        if phase == "batch" and isinstance(partial, BatchRenderResult):
            return native._portal_cli_emit(6, "render", "render-batch-reporting-failed", _message(error), robot,
                                           source=source, destination=str(destination), batch=partial.as_dict())
        capability = getattr(native, "_CapabilityError", ())
        if phase in {"start", "batch", "finish"} and (isinstance(error, capability)
                or _error_fields(error)[1].startswith("CAPABILITY: ")):
            code, identity, kind = 4, "capability", "render-capability-unavailable"
        elif phase in {"load", "construct"}:
            code, identity, kind = 5, "scene", "scene-load-failed"
        elif phase in {"start", "finish"}:
            code, identity, kind = 6, "render", "render-" + phase + "-failed"
        elif phase == "batch":
            if isinstance(error, OSError):
                code, identity, kind = 6, "render", "render-start-failed"
            else:
                code, identity, kind = 2, "usage", "usage-error"
        elif any(marker in _error_fields(error)[1] for marker in ("lumen:", "reel:", "portal-render:")):
            code, identity, kind = 6, "render", "render-failed"
        else:
            code, identity, kind = 5, "scene", "scene-execution-failed"
        return native._portal_cli_emit(code, identity, kind, _message(error), robot,
                                       source=source, scene=selected, phase=phase,
                                       destination=None if destination is None else str(destination),
                                       notes=list(_error_notes(error)),
                                       artifact_published=session is not None and session.artifact_published,
                                       **_partial_output(session, subdivide))
    if batch:
        return _emit_result(native, report, robot, source, destination)
    try:
        typesetting = typesetting_receipt(scene)
    except Exception:
        # Diagnostics never turn a published render into a failure.
        typesetting = None
    return _single_result(native, session.result, robot, source, selected, typesetting)
