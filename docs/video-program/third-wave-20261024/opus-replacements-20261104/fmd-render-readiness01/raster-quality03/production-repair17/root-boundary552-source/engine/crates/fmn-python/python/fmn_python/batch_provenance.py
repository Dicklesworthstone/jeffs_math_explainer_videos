"""Per-scene reproducible batches over the existing guarded RenderSession.

No new manifest format, renderer or checkpoint authority. One runtime snapshot
binds a batch's executions; each scene retains its own sources, seed, native
output and manifest receipt. Each scene's session records its effects
(effect_audit) exactly as a single certified render does.
"""
from __future__ import annotations

import os
from pathlib import Path
import sys

from .effect_audit import paused as paused_effects


def validate_batch_mode(native, format, reproducible, checkpoint=None):
    """Reject unsupported combinations before source execution or output I/O."""
    if not isinstance(reproducible, bool):
        raise TypeError("reproducible must be bool")
    if not reproducible:
        return
    if format not in {"png", "png_sequence", "wav"}:
        raise RuntimeError("CAPABILITY: reproducible batches require png, png_sequence, or wav output")
    if checkpoint is not None:
        raise RuntimeError(
            "CAPABILITY: reproducible batches cannot reuse checkpoint receipts; "
            "render to fresh destinations without checkpoint/resume"
        )
    if sys.platform == "win32":
        raise RuntimeError("CAPABILITY: windows-x86-64 is excluded from certified reproducibility by ADR-0019")
    if not callable(getattr(native, "_portal_publish_manifest", None)):
        raise RuntimeError("CAPABILITY: reproducible batches require the native provenance publisher")


def _sidecar(destination):
    return destination.with_name(destination.name + ".manifest")


def _available(destination):
    # These are diagnostics, not reservations. The shared native session still
    # owns the final no-clobber checks and all atomic artifact publication.
    for path in (destination, _sidecar(destination)):
        if os.path.lexists(path):
            raise FileExistsError(f"reproducible batch destination already exists: {path}")


def has_provenance(receipt):
    """A requested mode alone must never turn an incomplete receipt green."""
    digest = getattr(receipt, "closure_digest", None)
    return (getattr(receipt, "certified", False) is True
            and getattr(receipt, "manifest", None) is not None
            and isinstance(digest, str) and len(digest) == 64
            and all(char in "0123456789abcdef" for char in digest))


class BatchProvenance:
    def __init__(self, native, format, sources, runtime_identities, destinations):
        from . import rendering
        from .runtime_identity import RuntimeIdentityError, capture_runtime

        if sources is None:
            raise ValueError("reproducible batches require sources or a source provider")
        # Do not evaluate a provider before scene execution: it must include
        # lazy imports made by construct/tear_down. Static declarations freeze
        # once now, so a preceding scene cannot mutate the next scene's inputs.
        self.sources = sources if callable(sources) else rendering._source_snapshot(sources)
        for destination in destinations:
            _available(destination)
        try:
            # Hashing the runtime is not a scene effect.
            with paused_effects():
                self.snapshot = capture_runtime(native)
            self.identities = dict(self.snapshot.identities)
            supplied = None if runtime_identities is None else dict(runtime_identities)
            if (supplied is not None and supplied != self.identities
                    and supplied != rendering._runtime_identities(native)):
                raise RuntimeIdentityError("supplied runtime identities disagree with the installed runtime")
        except RuntimeIdentityError as error:
            raise RuntimeError("CAPABILITY: batch runtime input closure unavailable: " + str(error)) from error
        self.format = format

    def options(self, destination):
        # Includes changes caused by a preceding scene or progress observer,
        # BEFORE another constructor runs. RenderSession independently verifies
        # again before/after its own native finalization and sidecar publication.
        with paused_effects():
            self.snapshot.verify()
        _available(destination)
        return {
            "reproducible": True,
            "sources": self.sources if callable(self.sources) else dict(self.sources),
            "runtime_identities": dict(self.identities),
        }

    def accept(self, receipt, destination):
        if (has_provenance(receipt) and receipt.destination == destination
                and receipt.format == self.format
                and Path(receipt.manifest) == _sidecar(destination) / "manifest.fmnp"):
            return
        error = RuntimeError("reproducible scene did not return its complete native provenance receipt")
        # render_scene returned: its artifact may already exist. Do not retry,
        # delete, or convert an unverified publication into a successful result.
        error.add_note(f"render returned for {destination}; an artifact may already be published without complete provenance")
        raise error
