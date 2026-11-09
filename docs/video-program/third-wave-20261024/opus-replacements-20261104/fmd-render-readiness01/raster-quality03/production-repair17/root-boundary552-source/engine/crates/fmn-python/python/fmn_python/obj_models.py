"""Material-aware model loading with native parsers and host-owned local inputs.

OBJ geometry, MTL appearance, image decoding and mesh construction stay native.
The host resolves paths relative to each declaring file and prepares all inputs
before initializing the destination Group. This is not a filesystem sandbox.
"""
from __future__ import annotations

from contextlib import contextmanager
import math
import os
from pathlib import Path
import stat

_MAX_FILE_BYTES = 64 * 1024 * 1024
_MAX_TOTAL_BYTES = 128 * 1024 * 1024
_MAX_TEXTURE_BYTES = 256 * 1024 * 1024
_MAX_INPUTS = 4096
_BUSY = "_fmn_obj_import_busy"


def _path(value, parent=None):
    spelling = os.fspath(value)
    if not isinstance(spelling, str):
        raise TypeError("OBJ asset paths must be text paths")
    if "://" in spelling or "\0" in spelling:
        raise ValueError("OBJ assets must be local paths, not URLs or NUL-containing names")
    path = Path(spelling)
    if parent is not None:
        path = parent / path
    return path.resolve()


class _Inputs:
    """One bounded set of local bytes for an import attempt, never a downloader."""
    def __init__(self):
        self.payloads = {}
        self.size = 0

    def read(self, path):
        if path in self.payloads:
            return self.payloads[path]
        if len(self.payloads) >= _MAX_INPUTS:
            raise ValueError("OBJ input count budget exceeded")
        limit = min(_MAX_FILE_BYTES, _MAX_TOTAL_BYTES - self.size)
        # Do not block on devices/FIFOs even if an input is replaced between
        # resolve() and open(). O_NOFOLLOW rejects a final-component link race.
        flags = (os.O_RDONLY | getattr(os, "O_NONBLOCK", 0)
                 | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0))
        descriptor = os.open(path, flags)
        try:
            info = os.fstat(descriptor)
            if not stat.S_ISREG(info.st_mode):
                raise ValueError(f"OBJ input must be a regular file: {path}")
            if info.st_size > limit:
                raise ValueError("OBJ input byte budget exceeded")
            stream = os.fdopen(descriptor, "rb")
        except BaseException:
            os.close(descriptor)
            raise
        with stream:
            payload = stream.read(limit + 1)
        if len(payload) > limit:
            raise ValueError("OBJ input byte budget exceeded")
        self.payloads[path] = payload
        self.size += len(payload)
        return payload


def _prepare(g, obj_file):
    """Resolve only parser-declared assets; Python does not parse OBJ or MTL."""
    root = _path(obj_file)
    inputs = _Inputs()
    document = g["_ObjDocument"](inputs.read(root))
    libraries, parsed, definitions = [], {}, {}
    for filename in document.libraries:
        path = _path(filename, root.parent)
        if path not in parsed:
            library = g["_ObjMaterials"](inputs.read(path))
            parsed[path] = library
            libraries.append(library)
            for key, texture in library.entries:
                if key not in definitions:
                    if len(definitions) >= _MAX_INPUTS:
                        raise ValueError("OBJ aggregate material count budget exceeded")
                    # First declaration in library order is authoritative. Its
                    # map path belongs to THAT library's directory, not cwd.
                    definitions[key] = (path.parent, texture)
    images, decoded, texture_bytes = {}, {}, 0
    for key in document.material_names:
        if key not in definitions:
            raise ValueError(f"OBJ material {key!r} is not defined by its libraries")
        parent, filename = definitions[key]
        if filename is None:
            continue
        path = _path(filename, parent)
        if path not in decoded:
            decoded[path] = g["_RasterImage"].decode(inputs.read(path))
        image = decoded[path]
        width, height = image.size
        # Count each material binding conservatively, matching the native
        # aggregate guard even when bindings share a decoded allocation.
        texture_bytes += width * height * 4
        if texture_bytes > _MAX_TEXTURE_BYTES:
            raise ValueError("OBJ bound textures exceed the 256 MiB budget")
        images[key] = image
    return document, libraries, images, tuple(str(path) for path in inputs.payloads)


@contextmanager
def _construction(target):
    if target._is_bound():
        raise RuntimeError("ThreeDModel construction requires a detached target")
    attrs = vars(target)
    if attrs.get(_BUSY, False):
        raise RuntimeError("OBJ loading cannot reenter the same mobject")
    attrs[_BUSY] = True
    try:
        yield
    finally:
        attrs.pop(_BUSY, None)


def install_obj_models(native):
    """Keep the existing ThreeDModel identity, Group MRO and constructor API."""
    g = vars(native)
    if g.get("_FMN_OBJ_MODELS_INSTALLED", False):
        return
    Model, Group = g["ThreeDModel"], g["Group"]
    for key in ("_ObjDocument", "_ObjMaterials", "_build_obj_parts", "_RasterImage"):
        if key not in g:
            raise ImportError("native OBJ material import is unavailable: " + key)

    def initialize(self, obj_file: str, height=3):
        with _construction(self):
            height = float(height)
            if not math.isfinite(height) or not 0 < height <= 3.4028234663852886e38:
                raise ValueError("model height must be positive, finite and f32-representable")
            # Evaluate path conversion only once. Authored conversion effects
            # are real, but cannot reenter or silently bind this constructor.
            spelling = os.fspath(obj_file)
            document, libraries, images, asset_paths = _prepare(g, spelling)
            parts = iter(document.part_materials)

            def factory():
                material = next(parts)
                cls = g["TexturedGeometry"] if material in images else g["Surface"]
                child = cls.__new__(cls)
                g["_install_live_state"](child)
                child.material_name = material
                if material in images:
                    # Imported triangle soup has no retained trimesh object or
                    # file dependency. Native records own its UVs and normals.
                    child.num_textures = 1
                    child.image_file = child.image_path = child.texture_file = None
                    child.dark_image_file = None
                return child

            specs = g["_build_obj_parts"](document, libraries, images, factory, height)
            children = []
            for child, descendants in specs:
                g["_hang_native_children"](child, descendants)
                child.triangle_indices = g["_np"].arange(child.n_records(), dtype=int)
                children.append(child)
            # Parsing, required-asset IO, native decoding, record allocation and
            # shell construction completed without touching self's old family.
            if self._is_bound():
                raise RuntimeError("OBJ target was bound during input preparation")
            Group.__init__(self, *children)
            self.obj_file = spelling
            self.height = height
            self.asset_paths = asset_paths

    initialize.__name__ = "__init__"
    initialize.__qualname__ = Model.__qualname__ + ".__init__"
    initialize.__module__ = Model.__module__
    initialize.__annotations__["obj_file"] = str
    Model.__init__ = initialize
    g["_FMN_OBJ_MODELS_INSTALLED"] = True
