#!/usr/bin/env python3
"""Profile the pinned Python Reference for the §17.1 W1 baseline.

This is a deliberately self-contained benchmark definition and runner.  It
does not modify the Reference checkout, install a package into the repository,
or clear a cache.  Each timing sample runs in a fresh Python process.  The
first sample for each (scene, ffmpeg mode) pair receives a newly created cache
namespace; the warm samples reuse that namespace.

Run from the repository root inside an environment containing the pinned
Reference requirements and a working OpenGL display:

    xvfb-run -a -s "-screen 0 1920x1080x24" \
      uv run --no-project --python /usr/bin/python3 \
        --with-requirements scripts/manim_ref/requirements.txt \
        python scripts/profile_reference_baseline.py \
          --work-dir /data/tmp/FRESH_EMPTY_DIRECTORY \
          --json-out /data/tmp/reference-baseline.json

The work directory and JSON output are never deleted or overwritten.  Use a
new empty directory for every invocation.  The internal ``--worker`` mode is
an implementation detail used to make process and in-memory cache state
identical across samples.
"""

from __future__ import annotations

import argparse
import cProfile
import functools
import hashlib
import importlib.metadata
import json
import os
import platform
import pstats
import shutil
import statistics
import subprocess
import sys
import time
from collections import defaultdict
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / "scripts" / "manim_ref"
REFERENCE_PIN = "6199a00d4c1b1127ebe45cb629c3f22538b10e13"
RESOLUTION = (1920, 1080)
FPS = 30
WARM_REPETITIONS = 3
WORKER_TIMEOUT_SECONDS = 900

SCENES = {
    "opening_class": {
        "class": "OpeningClassBenchmark",
        "workload": (
            "OpeningManimExample-class composition: native text, a coordinate "
            "plane, concurrent entrance, and an affine grid transform"
        ),
    },
    "text_heavy": {
        "class": "TextHeavyBenchmark",
        "workload": (
            "twelve independently shaped Pango text lines with styling and "
            "two animated group operations"
        ),
    },
    "three_d": {
        "class": "ThreeDBenchmark",
        "workload": (
            "lit 3D sphere plus surface mesh under an oblique camera and a "
            "one-second rotation"
        ),
    },
    "dense_stroke": {
        "class": "DenseStrokeBenchmark",
        "workload": (
            "forty independently stroked 96-vertex polylines animated as one "
            "group"
        ),
    },
}

MODES = ("without_ffmpeg", "with_ffmpeg")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def checked_output(command: list[str], timeout: int = 30) -> str:
    return subprocess.run(
        command,
        check=True,
        capture_output=True,
        text=True,
        timeout=timeout,
    ).stdout.strip()


def reference_identity() -> dict[str, Any]:
    head = checked_output(["git", "-C", str(REFERENCE), "rev-parse", "HEAD"])
    if head != REFERENCE_PIN:
        raise RuntimeError(
            f"Reference pin mismatch: expected {REFERENCE_PIN}, found {head}"
        )
    status = checked_output(
        ["git", "-C", str(REFERENCE), "status", "--porcelain", "--untracked-files=no"]
    )
    if status:
        raise RuntimeError(
            "Reference checkout has tracked changes; refusing an unpinned baseline:\n"
            + status
        )
    return {
        "repository": "3b1b/manim",
        "path": str(REFERENCE.relative_to(ROOT)),
        "commit": head,
        "tracked_worktree_clean": True,
    }


def command_json(command: list[str], timeout: int = 30) -> Any:
    output = checked_output(command, timeout=timeout)
    try:
        return json.loads(output)
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"command emitted invalid JSON: {command}\n{output[:2000]}"
        ) from error


def host_provenance() -> dict[str, Any]:
    ffmpeg = shutil.which("ffmpeg")
    ffprobe = shutil.which("ffprobe")
    if ffmpeg is None or ffprobe is None:
        raise RuntimeError("ffmpeg and ffprobe are required for the with-ffmpeg lane")

    lscpu = command_json(["lscpu", "--json"])
    ffmpeg_version = checked_output([ffmpeg, "-version"]).splitlines()
    return {
        "hostname": platform.node(),
        "platform": platform.platform(),
        "uname": " ".join(platform.uname()),
        "logical_cpus": os.cpu_count(),
        "lscpu": lscpu,
        "ffmpeg": {
            "path": str(Path(ffmpeg).resolve()),
            "sha256": sha256_file(Path(ffmpeg).resolve()),
            "version_line": ffmpeg_version[0],
            "configuration_line": next(
                (line for line in ffmpeg_version if line.startswith("configuration:")),
                "",
            ),
        },
        "ffprobe": {
            "path": str(Path(ffprobe).resolve()),
            "sha256": sha256_file(Path(ffprobe).resolve()),
        },
    }


class PhaseBook:
    """Single-threaded inclusive/exclusive monotonic span accounting."""

    def __init__(self) -> None:
        self.enabled = True
        self.inclusive_ns: dict[str, int] = defaultdict(int)
        self.exclusive_ns: dict[str, int] = defaultdict(int)
        self.calls: dict[str, int] = defaultdict(int)
        self._stack: list[list[Any]] = []

    @contextmanager
    def span(self, name: str) -> Iterator[None]:
        if not self.enabled:
            yield
            return

        frame: list[Any] = [name, time.perf_counter_ns(), 0]
        self._stack.append(frame)
        try:
            yield
        finally:
            stopped = time.perf_counter_ns()
            popped = self._stack.pop()
            if popped is not frame:
                raise RuntimeError("phase stack corruption")
            duration = stopped - frame[1]
            exclusive = duration - frame[2]
            if exclusive < 0:
                raise RuntimeError(f"negative exclusive duration for {name}")
            self.inclusive_ns[name] += duration
            self.exclusive_ns[name] += exclusive
            self.calls[name] += 1
            if self._stack:
                self._stack[-1][2] += duration

    def snapshot(self) -> dict[str, dict[str, int]]:
        names = sorted(
            set(self.inclusive_ns) | set(self.exclusive_ns) | set(self.calls)
        )
        return {
            name: {
                "calls": self.calls[name],
                "inclusive_ns": self.inclusive_ns[name],
                "exclusive_ns": self.exclusive_ns[name],
            }
            for name in names
        }


def timed_method(
    owner: type[Any],
    method_name: str,
    phase: str,
    phases: PhaseBook,
    on_call: Callable[[], None] | None = None,
) -> None:
    original = getattr(owner, method_name)

    @functools.wraps(original)
    def wrapped(*args: Any, **kwargs: Any) -> Any:
        if on_call is not None:
            on_call()
        with phases.span(phase):
            return original(*args, **kwargs)

    setattr(owner, method_name, wrapped)


def import_reference() -> tuple[Any, int]:
    """Import the checkout without letting its global CLI parser see worker args."""

    worker_argv = sys.argv
    sys.argv = [worker_argv[0]]
    sys.path.insert(0, str(REFERENCE))
    started = time.perf_counter_ns()
    try:
        import manimlib as manim
    finally:
        elapsed = time.perf_counter_ns() - started
        sys.argv = worker_argv
    return manim, elapsed


def define_scenes(manim: Any) -> dict[str, type[Any]]:
    import numpy as np

    class OpeningClassBenchmark(manim.Scene):
        def construct(self) -> None:
            title = manim.Text(
                "Functions become visible transformations",
                font="DejaVu Sans",
                font_size=44,
            )
            title.to_edge(manim.UP)
            grid = manim.NumberPlane((-8, 8), (-4, 4))
            grid.set_stroke(manim.BLUE_E, width=1.25)
            self.play(
                manim.FadeIn(title),
                manim.ShowCreation(grid),
                run_time=0.5,
            )
            self.play(
                grid.animate.apply_matrix([[1.0, 0.55], [0.2, 1.0]]),
                title.animate.shift(0.15 * manim.UP),
                run_time=0.5,
            )

    class TextHeavyBenchmark(manim.Scene):
        def construct(self) -> None:
            phrases = [
                "A proof is a program with geometry.",
                "Curves retain their shared anchors.",
                "The clock advances by rational samples.",
                "Color conversion is explicit and measured.",
                "Every frame carries its input closure.",
                "Text and mathematics use native layout.",
                "Render order is stable under scheduling.",
                "Dirty tiles reveal eliminated work.",
                "Cache hits are evidence, not folklore.",
                "The scheduler is free; semantics are fixed.",
                "Profiling begins before optimization.",
                "Beautiful output remains the final judge.",
            ]
            lines = manim.VGroup(
                *[
                    manim.Text(
                        phrase,
                        font="DejaVu Sans",
                        font_size=30 + (index % 3) * 2,
                    )
                    for index, phrase in enumerate(phrases)
                ]
            )
            lines.arrange(manim.DOWN, aligned_edge=manim.LEFT, buff=0.12)
            lines.set_height(6.5)
            lines.set_color_by_gradient(manim.BLUE_B, manim.WHITE, manim.YELLOW)
            self.play(manim.FadeIn(lines), run_time=0.5)
            self.play(lines.animate.shift(0.18 * manim.UP), run_time=0.5)

    class ThreeDBenchmark(manim.ThreeDScene):
        def construct(self) -> None:
            self.frame.reorient(20, 70)
            sphere = manim.Sphere(radius=2.25, resolution=(31, 16))
            sphere.set_color(manim.BLUE_E)
            mesh = manim.SurfaceMesh(sphere)
            mesh.set_stroke(manim.WHITE, width=0.75, opacity=0.45)
            sphere.add(mesh)
            self.add(sphere)
            self.play(
                manim.Rotate(
                    sphere,
                    manim.PI / 2,
                    axis=manim.UP + 0.35 * manim.RIGHT,
                ),
                run_time=1.0,
            )

    class DenseStrokeBenchmark(manim.Scene):
        def construct(self) -> None:
            curves = []
            xs = np.linspace(-7.0, 7.0, 96)
            colors = [manim.BLUE_B, manim.TEAL, manim.YELLOW, manim.RED_B]
            for row in range(40):
                baseline = -3.45 + row * (6.9 / 39)
                phase = row * 0.31
                ys = baseline + 0.09 * np.sin(1.9 * xs + phase)
                zs = np.zeros_like(xs)
                curve = manim.VMobject()
                curve.set_points_as_corners(np.column_stack((xs, ys, zs)))
                curve.set_stroke(colors[row % len(colors)], width=2.25)
                curves.append(curve)
            group = manim.VGroup(*curves)
            self.add(group)
            self.play(
                group.animate.shift(0.18 * manim.RIGHT),
                run_time=1.0,
            )

    return {
        "opening_class": OpeningClassBenchmark,
        "text_heavy": TextHeavyBenchmark,
        "three_d": ThreeDBenchmark,
        "dense_stroke": DenseStrokeBenchmark,
    }


def package_versions() -> dict[str, str]:
    names = [
        "numpy",
        "scipy",
        "moderngl",
        "moderngl-window",
        "PyOpenGL",
        "Pillow",
        "manimpango",
        "fonttools",
        "diskcache",
        "colour",
        "pydub",
    ]
    versions = {}
    for name in names:
        try:
            versions[name] = importlib.metadata.version(name)
        except importlib.metadata.PackageNotFoundError:
            versions[name] = "absent"
    return versions


def cprofile_rows(profile: cProfile.Profile, limit: int = 40) -> list[dict[str, Any]]:
    stats = pstats.Stats(profile)
    rows = []
    for (filename, line, function), values in stats.stats.items():
        primitive_calls, total_calls, own_seconds, cumulative_seconds, _ = values
        path = Path(filename)
        try:
            display_path = str(path.resolve().relative_to(ROOT))
        except (OSError, ValueError):
            display_path = str(path)
        rows.append(
            {
                "function": f"{display_path}:{line}:{function}",
                "primitive_calls": primitive_calls,
                "total_calls": total_calls,
                "own_seconds": own_seconds,
                "cumulative_seconds": cumulative_seconds,
            }
        )
    rows.sort(
        key=lambda row: (row["cumulative_seconds"], row["own_seconds"]),
        reverse=True,
    )
    return rows[:limit]


def probe_movie(path: Path) -> dict[str, Any]:
    ffprobe = shutil.which("ffprobe")
    if ffprobe is None:
        raise RuntimeError("ffprobe disappeared during the benchmark")
    data = command_json(
        [
            ffprobe,
            "-v",
            "error",
            "-count_frames",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,pix_fmt,width,height,r_frame_rate,nb_read_frames",
            "-of",
            "json",
            str(path),
        ]
    )
    return data


def validate_movie_probe(probe: dict[str, Any]) -> None:
    streams = probe.get("streams", [])
    if len(streams) != 1:
        raise RuntimeError(f"expected one video stream, found {len(streams)}")
    stream = streams[0]
    expected = {
        "codec_name": "h264",
        "pix_fmt": "yuv420p",
        "width": RESOLUTION[0],
        "height": RESOLUTION[1],
        "r_frame_rate": f"{FPS}/1",
        "nb_read_frames": str(FPS),
    }
    drift = {
        key: {"expected": value, "found": stream.get(key)}
        for key, value in expected.items()
        if stream.get(key) != value
    }
    if drift:
        raise RuntimeError(f"published movie contract drift: {drift}")


def gl_identity(scene: Any) -> dict[str, str]:
    info = scene.camera.ctx.info
    return {
        "vendor": str(info.get("GL_VENDOR", "unknown")),
        "renderer": str(info.get("GL_RENDERER", "unknown")),
        "version": str(info.get("GL_VERSION", "unknown")),
    }


def run_worker(args: argparse.Namespace) -> int:
    process_started = time.perf_counter_ns()
    cache_dir = Path(args.cache_dir).resolve()
    output_dir = Path(args.output_dir).resolve()
    if not cache_dir.is_dir():
        raise RuntimeError(f"worker cache directory does not exist: {cache_dir}")
    output_dir.mkdir(parents=True, exist_ok=False)

    expected_cache_home = str(cache_dir)
    if os.environ.get("XDG_CACHE_HOME") != expected_cache_home:
        raise RuntimeError(
            "XDG_CACHE_HOME must exactly match --cache-dir before Reference import"
        )
    if os.environ.get("PYTHONDONTWRITEBYTECODE") != "1":
        raise RuntimeError("PYTHONDONTWRITEBYTECODE=1 is required")

    manim, import_ns = import_reference()
    scene_classes = define_scenes(manim)
    scene_class = scene_classes[args.scene]
    with_ffmpeg = args.mode == "with_ffmpeg"

    phases = PhaseBook()
    frame_count = 0

    def count_frame() -> None:
        nonlocal frame_count
        frame_count += 1

    timed_method(manim.Scene, "run", "scene_run", phases)
    timed_method(manim.Scene, "update_frame", "frame", phases)
    timed_method(manim.Scene, "update_mobjects", "scene_update", phases)
    timed_method(manim.Animation, "update_mobjects", "animation_update", phases)
    timed_method(manim.Animation, "interpolate", "animation_interpolate", phases)
    timed_method(manim.Mobject, "render", "render_dispatch", phases)
    timed_method(manim.Camera, "capture", "camera_capture", phases)
    timed_method(manim.Camera, "get_raw_fbo_data", "readback", phases)
    timed_method(
        manim.SceneFileWriter,
        "write_frame",
        "ffmpeg_feed",
        phases,
        on_call=count_frame,
    )
    timed_method(
        manim.SceneFileWriter,
        "close_movie_pipe",
        "encode_drain",
        phases,
    )

    original_construct = scene_class.construct

    @functools.wraps(original_construct)
    def timed_construct(scene: Any) -> Any:
        with phases.span("scene_construct"):
            return original_construct(scene)

    scene_class.construct = timed_construct

    writer_config = {
        "write_to_movie": with_ffmpeg,
        "save_last_frame": False,
        "subdivide_output": False,
        "output_directory": str(output_dir),
        "file_name": f"{args.scene}-{args.cache_state}-{args.repetition}",
        "quiet": True,
        "ffmpeg_bin": str(Path(shutil.which("ffmpeg") or "ffmpeg").resolve()),
        "video_codec": "libx264",
        "pixel_format": "yuv420p",
    }
    camera_config = {"resolution": RESOLUTION, "fps": FPS}

    profile = cProfile.Profile() if args.cprofile else None
    if profile is not None:
        profile.enable()
    try:
        with phases.span("scene_init"):
            scene = scene_class(
                camera_config=camera_config,
                file_writer_config=writer_config,
                skip_animations=False,
                show_animation_progress=False,
                leave_progress_bars=False,
            )
        scene.run()
    finally:
        if profile is not None:
            profile.disable()

    phases.enabled = False
    if tuple(scene.camera.get_pixel_shape()) != RESOLUTION:
        raise RuntimeError(
            f"camera resolution drifted: {scene.camera.get_pixel_shape()} != {RESOLUTION}"
        )
    if scene.camera.fps != FPS:
        raise RuntimeError(f"camera fps drifted: {scene.camera.fps} != {FPS}")
    if frame_count != FPS:
        raise RuntimeError(f"expected exactly {FPS} emitted frames, found {frame_count}")

    final_raw = scene.camera.get_raw_fbo_data()
    expected_raw_bytes = RESOLUTION[0] * RESOLUTION[1] * 4
    if len(final_raw) != expected_raw_bytes:
        raise RuntimeError(
            f"final framebuffer has {len(final_raw)} bytes, expected "
            f"{expected_raw_bytes}"
        )
    first_pixel = final_raw[:4]
    if final_raw == first_pixel * (len(final_raw) // len(first_pixel)):
        raise RuntimeError("final framebuffer liveness check failed")

    movie = output_dir / f"{args.scene}-{args.cache_state}-{args.repetition}.mp4"
    movie_evidence = None
    if with_ffmpeg:
        if not movie.is_file() or movie.stat().st_size == 0:
            raise RuntimeError(f"ffmpeg lane did not publish a movie: {movie}")
        movie_probe = probe_movie(movie)
        validate_movie_probe(movie_probe)
        movie_evidence = {
            "path": str(movie),
            "bytes": movie.stat().st_size,
            "sha256": sha256_file(movie),
            "ffprobe": movie_probe,
        }
    elif movie.exists():
        raise RuntimeError("without-ffmpeg lane unexpectedly published a movie")

    result = {
        "schema": "fmn-reference-baseline-worker/1",
        "scene": args.scene,
        "scene_class": scene_class.__name__,
        "mode": args.mode,
        "cache_state": args.cache_state,
        "repetition": args.repetition,
        "profiled": args.cprofile,
        "resolution": list(RESOLUTION),
        "fps": FPS,
        "emitted_frames": frame_count,
        "reference_import_ns": import_ns,
        "worker_process_ns": time.perf_counter_ns() - process_started,
        "phases": phases.snapshot(),
        "final_frame_sha256": hashlib.sha256(final_raw).hexdigest(),
        "movie": movie_evidence,
        "python": {
            "version": sys.version,
            "executable": sys.executable,
            "packages": package_versions(),
        },
        "gl": gl_identity(scene),
        "cprofile_top_cumulative": cprofile_rows(profile) if profile else [],
    }
    print(json.dumps(result, sort_keys=True))
    return 0


def worker_command(
    scene: str,
    mode: str,
    cache_state: str,
    repetition: int,
    cache_dir: Path,
    output_dir: Path,
    cprofile_enabled: bool,
) -> list[str]:
    command = [
        sys.executable,
        str(Path(__file__).resolve()),
        "--worker",
        "--scene",
        scene,
        "--mode",
        mode,
        "--cache-state",
        cache_state,
        "--repetition",
        str(repetition),
        "--cache-dir",
        str(cache_dir),
        "--output-dir",
        str(output_dir),
    ]
    if cprofile_enabled:
        command.append("--cprofile")
    return command


def run_sample(
    scene: str,
    mode: str,
    cache_state: str,
    repetition: int,
    cache_dir: Path,
    output_dir: Path,
    cprofile_enabled: bool,
) -> dict[str, Any]:
    command = worker_command(
        scene,
        mode,
        cache_state,
        repetition,
        cache_dir,
        output_dir,
        cprofile_enabled,
    )
    environment = os.environ.copy()
    environment.update(
        {
            "PYTHONDONTWRITEBYTECODE": "1",
            "XDG_CACHE_HOME": str(cache_dir),
            "LC_ALL": "C.UTF-8",
            "LANG": "C.UTF-8",
        }
    )
    started = time.perf_counter_ns()
    completed = subprocess.run(
        command,
        check=False,
        cwd=ROOT,
        env=environment,
        capture_output=True,
        text=True,
        timeout=WORKER_TIMEOUT_SECONDS,
    )
    outer_wall_ns = time.perf_counter_ns() - started
    if completed.returncode != 0:
        raise RuntimeError(
            "Reference worker failed\n"
            f"command: {command}\n"
            f"stdout:\n{completed.stdout}\n"
            f"stderr:\n{completed.stderr}"
        )
    lines = [line for line in completed.stdout.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError(f"worker emitted no JSON: {command}")
    try:
        result = json.loads(lines[-1])
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"worker's final stdout line was not JSON: {lines[-1]}"
        ) from error
    if result.get("schema") != "fmn-reference-baseline-worker/1":
        raise RuntimeError(f"unexpected worker schema: {result.get('schema')}")

    result["outer_wall_ns"] = outer_wall_ns
    result["worker_command"] = command
    result["stderr_sha256"] = hashlib.sha256(completed.stderr.encode()).hexdigest()
    result["stderr_tail"] = completed.stderr.splitlines()[-20:]
    return result


def aggregate(samples: list[dict[str, Any]]) -> dict[str, Any]:
    timing_samples = [sample for sample in samples if not sample["profiled"]]
    groups: dict[tuple[str, str, str], list[dict[str, Any]]] = defaultdict(list)
    for sample in timing_samples:
        groups[(sample["scene"], sample["mode"], sample["cache_state"])].append(
            sample
        )

    summaries = []
    for (scene, mode, cache_state), members in sorted(groups.items()):
        wall = [member["outer_wall_ns"] for member in members]
        scene_run = [
            member["phases"]["scene_run"]["inclusive_ns"] for member in members
        ]
        phase_names = sorted(
            {
                phase
                for member in members
                for phase in member["phases"]
            }
        )
        phase_medians = {}
        for phase in phase_names:
            phase_medians[phase] = {
                "inclusive_ns": int(
                    statistics.median(
                        member["phases"].get(phase, {}).get("inclusive_ns", 0)
                        for member in members
                    )
                ),
                "exclusive_ns": int(
                    statistics.median(
                        member["phases"].get(phase, {}).get("exclusive_ns", 0)
                        for member in members
                    )
                ),
            }
        summaries.append(
            {
                "scene": scene,
                "mode": mode,
                "cache_state": cache_state,
                "samples": len(members),
                "outer_wall_median_ns": int(statistics.median(wall)),
                "outer_wall_min_ns": min(wall),
                "outer_wall_max_ns": max(wall),
                "scene_run_median_ns": int(statistics.median(scene_run)),
                "phase_medians": phase_medians,
            }
        )
    return {"groups": summaries}


def ensure_empty_work_dir(path: Path) -> None:
    if path.exists():
        if not path.is_dir():
            raise RuntimeError(f"--work-dir is not a directory: {path}")
        if any(path.iterdir()):
            raise RuntimeError(
                "--work-dir must be a newly created empty directory; "
                f"refusing to overwrite or reuse {path}"
            )
    else:
        path.mkdir(parents=True)


def run_controller(args: argparse.Namespace) -> int:
    work_dir = Path(args.work_dir).resolve()
    ensure_empty_work_dir(work_dir)
    if not os.environ.get("DISPLAY"):
        raise RuntimeError(
            "DISPLAY is unset; run under xvfb-run or a recorded real GL display"
        )

    identity = reference_identity()
    host = host_provenance()
    samples: list[dict[str, Any]] = []

    for scene_index, scene in enumerate(SCENES):
        cache_dirs = {
            mode: work_dir / "cache" / scene / mode
            for mode in MODES
        }
        for cache_dir in cache_dirs.values():
            cache_dir.mkdir(parents=True)

        cold_modes = MODES if scene_index % 2 == 0 else tuple(reversed(MODES))
        schedule = [("cold", 0, cold_modes)]
        for repetition in range(1, args.warm_repetitions + 1):
            modes = (
                MODES
                if (scene_index + repetition) % 2 == 0
                else tuple(reversed(MODES))
            )
            schedule.append(("warm", repetition, modes))

        for cache_state, repetition, modes in schedule:
            for mode in modes:
                output_dir = (
                    work_dir
                    / "outputs"
                    / scene
                    / mode
                    / f"{cache_state}-{repetition}"
                )
                print(
                    f"[{scene}/{mode}] {cache_state} repetition {repetition}",
                    file=sys.stderr,
                    flush=True,
                )
                samples.append(
                    run_sample(
                        scene,
                        mode,
                        cache_state,
                        repetition,
                        cache_dirs[mode],
                        output_dir,
                        cprofile_enabled=False,
                    )
                )

        profile_cache = work_dir / "cache" / scene / "without_ffmpeg"
        profile_output = (
            work_dir / "outputs" / scene / "without_ffmpeg" / "warm-profile"
        )
        print(
            f"[{scene}/without_ffmpeg] warm cProfile attribution",
            file=sys.stderr,
            flush=True,
        )
        samples.append(
            run_sample(
                scene,
                "without_ffmpeg",
                "warm_profile",
                args.warm_repetitions + 1,
                profile_cache,
                profile_output,
                cprofile_enabled=True,
            )
        )

    gl_identity_value = samples[0]["gl"]
    if any(sample["gl"] != gl_identity_value for sample in samples[1:]):
        raise RuntimeError(
            "GL identity changed across samples: "
            f"{[sample['gl'] for sample in samples]}"
        )
    python_identity_value = samples[0]["python"]
    if any(
        sample["python"] != python_identity_value
        for sample in samples[1:]
    ):
        raise RuntimeError("Python environment changed across samples")
    for scene in SCENES:
        hashes = {
            sample["final_frame_sha256"]
            for sample in samples
            if sample["scene"] == scene
        }
        if len(hashes) != 1:
            raise RuntimeError(
                f"final framebuffer changed across cache/mode samples for {scene}: "
                f"{sorted(hashes)}"
            )
    scene_hashes = {
        scene: next(
            sample["final_frame_sha256"]
            for sample in samples
            if sample["scene"] == scene
        )
        for scene in SCENES
    }
    if len(set(scene_hashes.values())) != len(scene_hashes):
        raise RuntimeError(f"two benchmark scenes share a final frame: {scene_hashes}")

    raw = {
        "schema": "fmn-reference-baseline/1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "policy": {
            "status": (
                "calibration-only host evidence; not a pinned bare-metal PG gate"
            ),
            "cache": (
                "fresh namespace for each scene/mode cold sample; three later "
                "fresh-process samples reuse that namespace; no cache is cleared"
            ),
            "without_ffmpeg": (
                "no per-frame framebuffer readback, pipe feed, or encode; one "
                "post-timing readback validates the final framebuffer"
            ),
            "with_ffmpeg": (
                "Reference libx264/yuv420p path including per-frame readback, "
                "stdin feed, concurrent encode, final drain, and publication"
            ),
            "profiling": (
                "monotonic explicit spans on every timing sample; one separate "
                "warm cProfile attribution sample per scene, excluded from medians"
            ),
        },
        "benchmark": {
            "resolution": list(RESOLUTION),
            "fps": FPS,
            "seconds_per_scene": 1.0,
            "frames_per_scene": FPS,
            "warm_repetitions": args.warm_repetitions,
            "scenes": SCENES,
            "modes": list(MODES),
            "sample_order": (
                "cold modes alternate by scene; warm modes alternate by "
                "scene-plus-repetition parity"
            ),
            "harness": str(Path(__file__).resolve().relative_to(ROOT)),
            "harness_sha256": sha256_file(Path(__file__).resolve()),
        },
        "reference": identity,
        "host": host,
        "display": gl_identity_value,
        "python": python_identity_value,
        "controller": {
            "argv": sys.argv,
            "executable": sys.executable,
            "work_dir": str(work_dir),
            "display": os.environ["DISPLAY"],
        },
        "summary": aggregate(samples),
        "samples": samples,
    }

    serialized = json.dumps(raw, indent=2, sort_keys=True) + "\n"
    if args.json_out:
        output = Path(args.json_out).resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("x", encoding="utf-8") as handle:
            handle.write(serialized)
        print(f"wrote {output}", file=sys.stderr)
    else:
        sys.stdout.write(serialized)
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(
        description="Profile the pinned 3b1b/manim Reference for fm-bgr"
    )
    result.add_argument("--work-dir")
    result.add_argument("--json-out")
    result.add_argument(
        "--warm-repetitions",
        type=int,
        default=WARM_REPETITIONS,
        help="fresh-process warm samples per scene/mode (default: 3)",
    )

    result.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    result.add_argument("--scene", choices=sorted(SCENES), help=argparse.SUPPRESS)
    result.add_argument("--mode", choices=MODES, help=argparse.SUPPRESS)
    result.add_argument("--cache-state", help=argparse.SUPPRESS)
    result.add_argument("--repetition", type=int, help=argparse.SUPPRESS)
    result.add_argument("--cache-dir", help=argparse.SUPPRESS)
    result.add_argument("--output-dir", help=argparse.SUPPRESS)
    result.add_argument("--cprofile", action="store_true", help=argparse.SUPPRESS)
    return result


def main() -> int:
    args = parser().parse_args()
    if args.warm_repetitions < 1:
        raise RuntimeError("--warm-repetitions must be at least 1")
    if args.worker:
        required = {
            "--scene": args.scene,
            "--mode": args.mode,
            "--cache-state": args.cache_state,
            "--repetition": args.repetition,
            "--cache-dir": args.cache_dir,
            "--output-dir": args.output_dir,
        }
        missing = [name for name, value in required.items() if value is None]
        if missing:
            raise RuntimeError(f"worker arguments missing: {', '.join(missing)}")
        return run_worker(args)
    if not args.work_dir:
        raise RuntimeError("--work-dir is required")
    return run_controller(args)


if __name__ == "__main__":
    raise SystemExit(main())
