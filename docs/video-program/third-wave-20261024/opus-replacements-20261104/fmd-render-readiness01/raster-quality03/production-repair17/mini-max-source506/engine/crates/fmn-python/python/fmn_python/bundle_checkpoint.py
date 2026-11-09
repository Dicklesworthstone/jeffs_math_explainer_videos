"""Typed FMTL publication receipts for the existing batch checkpoint journal.

This is not an FMTL reader, source cache, or second persistence format. The
native exporter validates bundle bytes before publication; BatchCheckpoint
independently rehashes those files before trusting a completed publication.
"""
from pathlib import Path

from .bundle_export import BundleExportResult, _CAP

_REQUIRED = frozenset({"schema", "version", "format", "destination", "frame_count",
                       "segment_count", "bytes", "sha256", "fps", "replay_resolution",
                       "certified_source"})
_CAMERA = frozenset({"camera_track", "fmtl_minor"})


def restore_bundle_receipt(data, *, destination=None, options=None):
    """Validate native receipt shape and admitted settings before reusing it.

    The key/plan and output inventory are checked by the journal owner. No
    field in a checkpoint can turn a captured Python picture into certified
    source execution, or change a planar artifact into a camera-bearing one.
    """
    if not isinstance(data, dict):
        raise ValueError("invalid checkpoint bundle receipt")
    keys = frozenset(data)
    if keys not in (_REQUIRED, _REQUIRED | _CAMERA):
        raise ValueError("invalid checkpoint bundle receipt fields")
    if (data["schema"] != "fmn.python-bundle-export" or type(data["version"]) is not int
            or data["version"] != 1 or data["format"] != "fmtl"
            or data["certified_source"] is not False):
        raise ValueError("invalid checkpoint bundle receipt schema or certification")
    camera = keys == _REQUIRED | _CAMERA
    if camera and (data["camera_track"] is not True or type(data["fmtl_minor"]) is not int
                   or data["fmtl_minor"] != 1):
        raise ValueError("invalid checkpoint bundle camera metadata")
    name = data["destination"]
    if (not isinstance(name, str) or not name or "\0" in name
            or not Path(name).is_absolute() or ".." in Path(name).parts):
        raise ValueError("invalid checkpoint bundle destination")
    path = Path(name)
    if destination is not None and path != destination:
        raise ValueError("checkpoint bundle destination differs from the scene plan")
    for field, maximum in (("frame_count", (1 << 32) - 1),
                           ("segment_count", (1 << 64) - 1), ("bytes", _CAP), ("fps", 240)):
        if type(data[field]) is not int or not 1 <= data[field] <= maximum:
            raise ValueError("invalid checkpoint bundle receipt field: " + field)
    resolution = data["replay_resolution"]
    if (not isinstance(resolution, list) or len(resolution) != 2
            or any(type(value) is not int or value <= 0 for value in resolution)
            or resolution[0] * resolution[1] > 16_777_216):
        raise ValueError("invalid checkpoint bundle resolution")
    digest = data["sha256"]
    if (not isinstance(digest, str) or len(digest) != 64
            or any(char not in "0123456789abcdef" for char in digest)):
        raise ValueError("invalid checkpoint bundle digest")
    if options is not None:
        if camera != options["bundle_camera"]:
            raise ValueError("checkpoint bundle camera mode differs from the scene plan")
        if options["fps"] is not None and data["fps"] != options["fps"]:
            raise ValueError("checkpoint bundle FPS differs from the scene plan")
        if options["resolution"] is not None and list(options["resolution"]) != resolution:
            raise ValueError("checkpoint bundle resolution differs from the scene plan")
        limits = options["bundle_limits"]
        if data["frame_count"] > limits["max_frames"] or data["bytes"] > limits["max_output_bytes"]:
            raise ValueError("checkpoint bundle exceeds the admitted recording limits")
    return BundleExportResult(path, data["frame_count"], data["segment_count"],
                              data["bytes"], digest, data["fps"], tuple(resolution), camera)
