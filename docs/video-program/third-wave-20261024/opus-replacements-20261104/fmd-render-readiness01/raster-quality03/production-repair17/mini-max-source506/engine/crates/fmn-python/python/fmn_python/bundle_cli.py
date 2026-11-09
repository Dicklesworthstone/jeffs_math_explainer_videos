"""Console entry for Python Scene -> code-free FMTL/1 export.

The shared parser validates resolution/FPS and destination syntax. Its existing
PNG format is used only to parse those common options, never as an output sink
or fallback. The only output acquired here is BundleExportSession.
"""
from __future__ import annotations

import contextlib
import os
from pathlib import Path
import sys

from .bundle_export import BundleExportSession, require_bundle_capability
from .console_rendering import _tokens, _VALUE_FLAGS, _message, _MAX_SELECTED_SCENES
from .batch_cli import _emit_result
from .checkpoint_cli import CHECKPOINT_HELP, take_checkpoint_options
from .batch_rendering import BatchRenderError, BatchRenderResult, _error_notes, _name_key, render_scenes
from .scene_loading import SceneSource

BUNDLE_HELP = """Portable scene bundles:
  fmn-python [--robot] SOURCE.py [SCENE] --format fmtl
             [--bundle-camera] [--resolution WIDTHxHEIGHT] [--fps FPS]
             [--video_dir FILE.fmtl]
  fmn-python [--robot] SOURCE.py SCENE1 SCENE2 --format fmtl
             [--bundle-camera] [--keep-going] [--video_dir DIRECTORY]
  fmn-python [--robot] SOURCE.py --write_all --format fmtl
             [--bundle-camera] [--keep-going] [--video_dir DIRECTORY]

Runs setup/construct/tear_down once in the host interpreter. Actual captured
frames become a bounded, code-free FMTL/1 artifact for standalone fmn/WASM.
--video_dir names the exact output file (default: media/videos/SOURCE/SCENE.fmtl).
For multiple names or --write_all/-a, --video_dir names a directory containing
SCENE.fmtl for each selected scene (default: media/videos/SOURCE). Names run in
requested order; --write_all uses sorted locally declared names. All destinations
are checked before any constructor runs. A failed scene stops the batch unless
--keep-going is set; completed bundles remain valid. Interrupts always stop.
Publication never overwrites a destination. The default minor-0 bundle preserves
planar vectors/text at the default camera/light/background for native/WASM replay.
--bundle-camera records each captured view, lighting and background together with
3D/raster/vector geometry in a minor-1 bundle for the native fmn player. The
camera-less WASM player refuses this mode. Replay never executes scene code.
Audio, partial playback and source certification remain explicit refusals.
Use the same --resolution for exact replay; camera bundles also record aspect.
"""

_ALLOWED = frozenset({"--format", "--resolution", "--fps", "--video_dir"})


def _bundle_options(arguments):
    """Recognize an actual format option, not 'fmtl' inside another value."""
    normalized, formats = [], []
    index = 0
    while index < len(arguments):
        token = arguments[index]
        if token in _VALUE_FLAGS:
            normalized.append(token)
            if index + 1 < len(arguments):
                value = arguments[index + 1]
                normalized.append(value)
                if token == "--format":
                    formats.append(value)
            index += 2
        elif token.startswith("--") and "=" in token:
            name, value = token.split("=", 1)
            if name in _VALUE_FLAGS:
                normalized.extend((name, value))
                if name == "--format":
                    formats.append(value)
            else:
                normalized.append(token)
            index += 1
        else:
            normalized.append(token)
            index += 1
    return normalized, formats


def try_bundle_cli(native, arguments):
    arguments, formats = _bundle_options(arguments)
    native_options, selectors, switches = _tokens(
        arguments, {"--robot", "--bundle-camera", "--write_all", "-a", "--keep-going"}
    )
    camera = "--bundle-camera" in switches
    if "fmtl" not in formats and not camera:
        return None
    robot = "--robot" in switches
    if "--help" in switches or "-h" in switches:
        text = BUNDLE_HELP + "\n" + CHECKPOINT_HELP
        return native._portal_cli_emit(0, "success", "help", text, robot, help=text)
    source = selected = destination = None
    session = report = None
    phase = "options"
    try:
        if len(formats) != 1 or formats[0] != "fmtl":
            raise ValueError("bundle export requires exactly one --format fmtl")
        if switches.count("--robot") > 1 or switches.count("--bundle-camera") > 1:
            raise ValueError("--robot and --bundle-camera must not be repeated")
        write_all = switches.count("--write_all") + switches.count("-a")
        keep_going = switches.count("--keep-going")
        if write_all > 1 or keep_going > 1:
            raise ValueError("--write_all/-a and --keep-going must not be repeated")
        requested = selectors[1:]
        batch = bool(write_all or len(requested) > 1)
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
        native_options, recovery = take_checkpoint_options(native_options, _VALUE_FLAGS)
        if recovery and not batch:
            raise ValueError("checkpoint recovery requires multiple scene names or --write_all")
        # Refuse unsupported semantics before importing or constructing source.
        # Unsupported modes cannot be silently ignored by a bundle export.
        index = 0
        while index < len(native_options):
            token = native_options[index]
            if token in _VALUE_FLAGS:
                if token not in _ALLOWED:
                    raise native._CapabilityError(f"{token} does not apply to FMTL export")
                index += 2
            else:
                raise native._CapabilityError(
                    f"{token} is not supported for full-scene FMTL export; use the PNG/video path"
                )
        require_bundle_capability(native, camera=camera)
        parser_options = list(native_options)
        parser_options[parser_options.index("--format") + 1] = "png_sequence"
        # Only common syntax/config validation is delegated. Never render PNG.
        positionals, values, width, height, fps, _threads = native._portal_cli_render_arguments(
            (selectors[:1] if batch else selectors) + parser_options
        )
        if width * height > 16_777_216 or (fps is not None and not 1 <= fps <= 240):
            raise ValueError("bundle export requires at most 16M pixels and 1..240 FPS")
        source = str(Path(positionals[0]).resolve())
        selected = requested[0] if len(requested) == 1 else None
        supplied = values["video_dir"]
        if supplied is not None:
            if not str(supplied) or "\0" in str(supplied):
                raise ValueError("bundle destination must be nonempty and contain no NUL")
            destination = Path(os.path.abspath(supplied))
        default_root = (Path("media") / "videos" / Path(source).stem).resolve()
        if destination is not None and os.path.lexists(destination):
            phase = "start"
            if not batch:
                raise FileExistsError(f"bundle destination already exists: {destination}")
            if not destination.is_dir():
                raise NotADirectoryError(str(destination))
        phase = "load"
        redirect = contextlib.redirect_stdout(sys.stderr) if robot else contextlib.nullcontext()
        with redirect, SceneSource(source, native.Scene) as loaded:
            names = sorted(loaded.scenes)
            phase = "select"
            if batch:
                requested = names if write_all else requested
                if not requested:
                    raise ValueError("no locally declared Scene classes found in " + source)
                if len(requested) > _MAX_SELECTED_SCENES:
                    raise ValueError(f"scene selection exceeds {_MAX_SELECTED_SCENES} names")
                missing = [name for name in requested if name not in loaded.scenes]
                if missing:
                    raise ValueError("selected scenes were not declared by " + source + ": " + ", ".join(missing))
                destination = default_root if destination is None else destination
                phase = "batch"
                # The shared planner validates every destination before any
                # constructor. It also owns cancellation, no-clobber publication
                # and partial receipts; keep the source loader alive for all jobs.
                report = render_scenes(
                    {name: loaded.scenes[name] for name in requested}, destination,
                    format="fmtl", resolution=(width, height), fps=fps,
                    bundle_camera=camera, continue_on_error=bool(keep_going),
                    max_jobs=_MAX_SELECTED_SCENES, **recovery,
                    on_result=lambda outcome: print(
                        f"fmn-python: {outcome.name}: {outcome.status}: {outcome.destination}",
                        file=sys.stderr,
                    ),
                )
            else:
                if selected is None:
                    if len(names) != 1:
                        raise ValueError("select one Scene explicitly; discovered: " + (", ".join(names) or "none"))
                    selected = names[0]
                if selected not in loaded.scenes:
                    raise ValueError(f"scene {selected!r} was not declared by {source}")
                if destination is None:
                    destination = default_root / (selected + ".fmtl")
                phase = "start"
                if os.path.lexists(destination):
                    raise FileExistsError(f"bundle destination already exists: {destination}")
                phase = "construct"
                scene = loaded.scenes[selected]()
                phase = "start"
                session = BundleExportSession(
                    scene, destination, resolution=(width, height), fps=fps,
                    camera=camera, _native=native,
                )
                with session:
                    phase = "execute"
                    try:
                        scene.run()
                    except native.EndScene:
                        pass
                    phase = "finish"
                if session.result is None:
                    raise RuntimeError("bundle export returned without a publication receipt")
    except BatchRenderError as error:
        return _emit_result(native, error.result, robot, source, destination)
    except BaseException as error:
        partial = getattr(error, "render_batch_result", None)
        if phase == "batch" and isinstance(partial, BatchRenderResult):
            if isinstance(error, (KeyboardInterrupt, SystemExit)):
                return _emit_result(native, partial, robot, source, destination,
                                    interrupted=True,
                                    interrupt_code=130 if isinstance(error, KeyboardInterrupt) else 5)
            return native._portal_cli_emit(
                6, "render", "render-batch-reporting-failed", _message(error), robot,
                source=source, destination=str(destination), batch=partial.as_dict(),
                notes=list(_error_notes(error)),
            )
        if isinstance(error, KeyboardInterrupt):
            code, identity = 130, "interrupted"
        elif isinstance(error, native._CapabilityError):
            code, identity = 4, "capability"
        elif phase in {"options", "select", "batch"} and isinstance(error, (ValueError, TypeError)):
            code, identity = 2, "usage"
        elif phase in {"start", "finish"} or (phase == "batch" and isinstance(error, OSError)):
            code, identity = 6, "render"
        else:
            code, identity = 5, "scene"
        return native._portal_cli_emit(
            code, identity, "bundle-export-failed", _message(error), robot,
            source=source, scene=selected, phase=phase, format="fmtl", exported=False,
            destination=None if destination is None else str(destination),
            artifact_published=session is not None and session.artifact_published,
            notes=list(_error_notes(error)),
        )
    if report is not None:
        return _emit_result(native, report, robot, source, destination)
    receipt = session.result
    return native._portal_cli_emit(
        0, "success", "bundle-export", f"exported {receipt.frame_count} frames to {destination}", robot,
        source=source, scene=selected, format="fmtl", exported=True,
        destination=str(destination), frame_count=receipt.frame_count,
        bundle=receipt.as_dict(),
    )
