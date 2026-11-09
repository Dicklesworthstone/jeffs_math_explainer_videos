"""Data-only checkpoint admission for a complete animation + final PNG pair.

The existing journal owns locking, plan identity and atomic persistence. This
adapter binds both native receipts to plan-derived paths, then verifies their
actual bytes through the same bounded, no-symlink inventory as single outputs.
It never reconstructs a Scene, publishes a file, or resumes an incomplete pair.
"""
from __future__ import annotations

from pathlib import Path

_FORMATS = frozenset({"gif", "y4m", "png_sequence", "mp4", "mov", "wav"})
_FIELDS = frozenset({"schema", "version", "destination", "still_destination",
                     "primary", "still", "completed", "unreceipted_artifacts"})


def paired_destinations(destination, format):
    """Derive paths from the admitted plan, never from a saved nested receipt."""
    from .paired_output import companion_png_path

    if format not in _FORMATS:
        raise ValueError("paired checkpoints require an animation or WAV primary")
    primary = Path(destination)
    if not primary.is_absolute():
        raise ValueError("paired checkpoint destinations must be absolute")
    return primary, companion_png_path(primary, format)


def restore_pair_receipt(data, *, destination=None, options=None):
    """Validate the whole pair before restoring the existing immutable results."""
    from .batch_checkpoint import _json
    from .batch_rendering import _restored_outcome
    from .paired_output import PairedRenderResult

    if (not isinstance(data, dict) or set(data) != _FIELDS
            or data.get("schema") != "fmn.paired-render"
            or type(data.get("version")) is not int or data["version"] != 1
            or data.get("completed") is not True
            or type(data.get("unreceipted_artifacts")) is not list
            or data["unreceipted_artifacts"]):
        raise ValueError("checkpoint requires a complete, fully receipted output pair")
    primary, still = data.get("primary"), data.get("still")
    if not isinstance(primary, dict) or not isinstance(still, dict):
        raise ValueError("checkpoint pair requires both publication receipts")
    format = primary.get("format")
    if options is not None:
        if options.get("save_last_frame") is not True or options.get("subdivide", False):
            raise ValueError("checkpoint pair does not match the output mode")
        if format != options.get("format"):
            raise ValueError("checkpoint pair does not match the selected format")
    text = data.get("destination")
    if not isinstance(text, str) or not text or "\0" in text:
        raise ValueError("invalid checkpoint pair destination")
    destination = Path(text) if destination is None else Path(destination)
    destination, still_destination = paired_destinations(destination, format)
    if data["destination"] != str(destination) or data.get("still_destination") != str(still_destination):
        raise ValueError("checkpoint pair paths do not match the scene plan")
    results = []
    for role, receipt, path, expected_format in (
        ("primary", primary, destination, format),
        ("still", still, still_destination, "png"),
    ):
        if (receipt.get("destination") != str(path) or receipt.get("format") != expected_format
                or receipt.get("certified") is not False
                or any(key in receipt for key in ("manifest", "closure_digest", "artifact_digest"))):
            raise ValueError("invalid checkpoint pair " + role + " receipt")
        # Reuse the ordinary data-only RenderResult restoration, including its
        # strict numeric checks and invocation/audio-input field checks.
        result = _restored_outcome({"name": role, "result": receipt}).result
        if result.bytes <= 0:
            raise ValueError("checkpoint pair artifacts must contain bytes")
        if expected_format == "wav":
            if (result.frame_count is not None or result.sample_frames is None
                    or type(receipt.get("sample_rate")) is not int or receipt["sample_rate"] != 48000
                    or type(receipt.get("channels")) is not int or receipt["channels"] != 2):
                raise ValueError("invalid checkpoint pair WAV receipt")
        elif result.sample_frames is not None or result.frame_count is None or result.frame_count <= 0:
            raise ValueError("invalid checkpoint pair frame receipt")
        if role == "still" and result.frame_count != 1:
            raise ValueError("checkpoint final PNG must contain exactly one frame")
        if not 0 <= result.seed < 1 << 64:
            raise ValueError("invalid checkpoint pair seed")
        results.append(result)
    # Camera readback has its own thread/engine identity. It must agree on
    # scene identity, selected view dimensions, frame rate and playback range.
    for key in ("resolution", "fps", "seed", "animation_range"):
        if _json(primary.get(key)) != _json(still.get(key)):
            raise ValueError("checkpoint pair receipts disagree on " + key)
    if options is not None:
        for key in ("resolution", "fps"):
            if options.get(key) is not None and _json(options[key]) != _json(primary.get(key)):
                raise ValueError("checkpoint pair does not match the selected " + key)
        if _json(options.get("animation_range")) != _json(primary.get("animation_range")):
            raise ValueError("checkpoint pair does not match the selected animation_range")
    return PairedRenderResult(destination, still_destination, results[0], results[1], True)


def pair_inventory(data, destination, options):
    """Verify both artifacts; one valid primary is never evidence of a pair."""
    from .batch_checkpoint import _inventory, _MAX_FILES

    pair = restore_pair_receipt(data, destination=destination, options=options)
    inventories = {}
    count = 0
    for role, receipt in (("primary", pair.primary), ("still", pair.still)):
        files = _inventory(receipt.destination)
        count += len(files)
        if count > _MAX_FILES:
            raise ValueError("checkpoint pair inventory exceeds its combined file budget")
        if receipt.format == "png_sequence":
            if not receipt.destination.is_dir():
                raise ValueError("checkpoint PNG sequence must be a directory")
        elif (len(files) != 1 or files[0]["path"] != ""
              or files[0]["bytes"] != receipt.bytes or files[0]["sha256"] != receipt.digest):
            raise ValueError("checkpoint pair " + role + " is missing or modified: native receipt mismatch")
        inventories[role] = files
    return inventories
