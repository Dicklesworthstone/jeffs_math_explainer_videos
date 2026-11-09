"""Distribution helpers for the optional FrankenManim CPython portal."""

import csv as _csv
import re as _re
from importlib.metadata import distributions as _distributions
from pathlib import PurePosixPath as _PurePosixPath


class _ManimlibNamespaceCollision(ImportError):
    """Two installed distributions claim the exclusive ``manimlib`` tree."""

    def __init__(self, providers):
        self.providers = tuple(providers)
        joined = ", ".join(self.providers)
        super().__init__(
            "the exclusive manimlib package is claimed by franken-manim and "
            f"{joined}; install each provider in a separate virtual environment"
        )


def _canonical_distribution_name(name):
    """Apply the comparison part of Python's distribution-name normalization."""

    return _re.sub(r"[-_.]+", "-", str(name)).lower()


def _claims_manimlib(package_path):
    parts = _PurePosixPath(str(package_path).replace("\\", "/")).parts
    return bool(parts) and parts[0] == "manimlib"


def _manimlib_claims(distribution):
    """The ``manimlib`` paths a distribution's file list claims that exist.

    This is ``Distribution.files`` filtered to ``manimlib`` claims, without
    its missing-file filter's stat of every file of every distribution
    (26,492 stats in a scientific venv, 0.65 s per scan): RECORD rows are
    parsed as text and only the ``manimlib`` claims are located on disk.
    """
    record = distribution.read_text("RECORD")
    if not record:
        return [path for path in (distribution.files or ()) if _claims_manimlib(path)]
    # A path whose first part is manimlib contains the substring; the
    # substring test spares building a path for every unrelated row.
    return [
        row[0]
        for row in _csv.reader(record.splitlines())
        if row
        and "manimlib" in row[0]
        and _claims_manimlib(row[0])
        and distribution.locate_file(row[0]).exists()
    ]


def _foreign_manimlib_providers():
    """Return installed non-FrankenManim distributions owning ``manimlib``."""

    providers = set()
    for distribution in _distributions():
        try:
            if not _manimlib_claims(distribution):
                continue
            name = distribution.metadata.get("Name") or "<unknown distribution>"
        except (OSError, UnicodeError, ValueError, _csv.Error):
            # Broken metadata in an unrelated distribution must not disable the
            # portal. A malformed claimant whose files cannot be enumerated is
            # outside the package manager's own collision model as well.
            continue
        if _canonical_distribution_name(name) != "franken-manim":
            providers.add(str(name))
    return tuple(sorted(providers, key=lambda name: (name.casefold(), name)))


_scanned_providers = None


def _ensure_exclusive_manimlib_namespace():
    """Refuse a detectable package-file collision before loading native code.

    The CLI, the ``manimlib`` package and portal initialization each check;
    the installed set is scanned once per process.
    """

    global _scanned_providers
    if _scanned_providers is None:
        _scanned_providers = _foreign_manimlib_providers()
    if _scanned_providers:
        raise _ManimlibNamespaceCollision(_scanned_providers)


def __getattr__(name):
    if name in {"BundleExportSession", "BundleExportResult", "export_bundle"}:
        from . import bundle_export

        return getattr(bundle_export, name)
    if name in {"PairedRenderSession", "PairedRenderResult", "paired_render_session", "render_scene_with_still"}:
        from . import paired_output

        return getattr(paired_output, name)
    if name == "SceneProject":
        from .scene_project import SceneProject

        return SceneProject
    if name in {"RecordingSession", "record_scene"}:
        from . import recording

        return getattr(recording, name)
    if name in {"RenderSegment", "SubdividedRenderResult", "SubdividedRecordingSession",
                "record_subdivided_scene"}:
        from . import subdivision

        return getattr(subdivision, name)
    if name in {"subdivided_render_session", "render_subdivided_scene"}:
        from . import subdivision_rendering

        return getattr(subdivision_rendering, name)
    if name == "embed_scene":
        from .embedded_shell import embed_scene

        return embed_scene
    if name == "SceneConsole":
        from .scene_console import SceneConsole

        return SceneConsole
    if name == "__version__":
        _ensure_exclusive_manimlib_namespace()
        from manimlib import __version__

        return __version__
    if name in {"RenderResult", "RenderSession", "render_scene", "render_session"}:
        from . import rendering

        return getattr(rendering, name)
    if name in {"RenderJob", "SceneRenderOutcome", "BatchRenderResult", "BatchRenderError", "render_scenes"}:
        from . import batch_rendering

        return getattr(batch_rendering, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
