"""Declared-input identities for Python Studio generations.

The native SourceWatch owns traversal, filtering, budgets and content hashing.
This adapter binds that observed input set to worker launch and publication,
then checks the bytes actually compiled by SceneSource against the same set.
It is an authoring freshness boundary, NOT the C1-C10 certified input closure:
undeclared data files, third-party imports and arbitrary host effects remain
outside this contract. No scene code runs in the supervising interpreter.
"""
from __future__ import annotations

import os
from pathlib import Path
from typing import Any


def project_directory(source: Path) -> Path:
    """Watch the whole containing package, without traversing its import root.

    A scene in pkg/scenes/example.py can import pkg/helpers.py or a parent
    __init__.py. Watching only the scene's immediate directory misses both.
    Non-package scenes keep the existing sibling-directory behavior.
    """
    directory = source.parent
    while (directory / "__init__.py").is_file():
        parent = directory.parent
        if parent == directory:
            raise ValueError("the filesystem root cannot be a Studio scene package")
        if not (parent / "__init__.py").is_file():
            break
        directory = parent
    return directory


def _roots(values: Any) -> tuple[str, ...]:
    if not isinstance(values, (list, tuple)) or not 1 <= len(values) <= 64:
        raise ValueError("Studio input identity requires 1..64 declared paths")
    roots = []
    for value in values:
        if not isinstance(value, str) or not value or "\0" in value or not Path(value).is_absolute():
            raise ValueError("Studio input identity paths must be absolute strings without NUL")
        # Paths are frozen by the host. Do not silently resolve a different
        # symlink target or reinterpret them relative to an authored chdir.
        if os.path.normpath(value) != value:
            raise ValueError("Studio input identity paths must be normalized")
        roots.append(value)
    if len(set(roots)) != len(roots):
        raise ValueError("Studio input identity paths must be unique")
    return tuple(roots)


def _digest(value: Any) -> str:
    if (not isinstance(value, str) or len(value) != 64
            or any(char not in "0123456789abcdef" for char in value)):
        raise ValueError("Studio input identity requires a lowercase SHA-256 fingerprint")
    return value


class StudioInputs:
    """A frozen native snapshot, separate from the autoreload polling cursor."""

    def __init__(self, native: Any, roots: list[str] | tuple[str, ...], *, expected=None):
        self.roots = _roots(roots)
        self._native = native
        # A fresh native watcher captures exactly once. No poll consumes or
        # resets the separate autoreload watcher's debounce/pending edit state.
        snapshot = native._StudioHost.watch_sources(list(self.roots), 0)
        self.fingerprint = _digest(snapshot.fingerprint)
        self.files = dict(snapshot.files)
        if expected is not None and self.fingerprint != _digest(expected):
            raise RuntimeError("Studio declared inputs changed between launch and capture; reload")

    @classmethod
    def from_request(cls, native: Any, request: Any) -> StudioInputs:
        if not isinstance(request, dict) or set(request) != {"roots", "fingerprint"}:
            raise ValueError("Studio worker requires a declared-input identity")
        return cls(native, request["roots"], expected=_digest(request["fingerprint"]))

    def as_request(self) -> dict[str, Any]:
        # Only the bounded root list and aggregate fingerprint cross argv.
        # Thousands of per-file rows stay local to the native scan.
        return {"roots": list(self.roots), "fingerprint": self.fingerprint}

    def require_source(self, path: Path, digest: str) -> None:
        if self.files.get(str(path)) != digest:
            raise RuntimeError("Studio entry source differs from its declared-input snapshot; reload")

    def verify(self, loaded=None) -> None:
        """Refuse changed inputs or source bytes compiled outside the snapshot.

        Comparing executed bytes also catches an edit temporarily installed
        during import and then reverted before the publication-time scan.
        """
        if loaded is not None:
            for path, digest in loaded.source_digests.items():
                expected = self.files.get(str(path))
                if expected is None:
                    raise RuntimeError(
                        f"Studio imported undeclared project source {path}; "
                        "include it with autoreload=True and watch_paths (CLI: --autoreload --watch PATH)"
                    )
                if digest != expected:
                    raise RuntimeError(f"Studio executed changed project source {path}; reload")
        current = self._native._StudioHost.watch_sources(list(self.roots), 0)
        if current.fingerprint != self.fingerprint:
            raise RuntimeError("Studio declared inputs changed during capture; reload")
