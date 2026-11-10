"""Textured surfaces and meshes on native durable image resources.

The portal resolves local image paths and preserves Python object metadata.
Atlas owns mesh normals, fmn-codec owns decoding, and Lumen owns perspective
sampling, alpha, lighting and depth. Both sides of a light/dark pair are
immutable native resources carried by copies, snapshots and replay.
"""
from __future__ import annotations

import math
from typing import Any

from .image_authoring import _source, _editing

_MAX_IMAGE_BYTES = 64 * 1024 * 1024


def _payload(path):
    with path.open("rb") as stream:
        data = stream.read(_MAX_IMAGE_BYTES + 1)
    if len(data) > _MAX_IMAGE_BYTES:
        raise ValueError("texture input exceeds the 64 MiB encoded budget")
    return data


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def install_surface_textures(native: Any) -> None:
    g = vars(native)
    if g.get("_FMN_SURFACE_TEXTURES_INSTALLED", False):
        return
    Surface, Textured, Geometry = (g[name] for name in ("Surface", "TexturedSurface", "TexturedGeometry"))
    # Resolve native functions through this module's dictionary. Capturing
    # builtins directly also captures their __self__ module and prevents an
    # embedded owner from releasing that module by clearing its globals.
    np = g["_np"]

    def options(kwargs):
        result = dict(kwargs)
        unknown = set(result) - {"opacity", "shading", "depth_test", "z_index", "is_fixed_in_frame"}
        if unknown:
            raise TypeError("textured object got unexpected keyword arguments: " + ", ".join(sorted(unknown)))
        if "opacity" in result:
            value = float(result["opacity"])
            if not math.isfinite(value):
                raise ValueError("texture opacity must be finite")
            result["opacity"] = value
        if "z_index" in result:
            # Normalize once; an authored __int__ must not produce a different
            # native draw key and Python mirror on two successive calls.
            result["z_index"] = int(result["z_index"])
        if "is_fixed_in_frame" in result:
            value = float(result["is_fixed_in_frame"])
            if not math.isfinite(value):
                raise ValueError("texture fixed-in-frame mix must be finite")
            result["is_fixed_in_frame"] = value
        if "shading" in result:
            values = tuple(float(value) for value in result["shading"])
            if len(values) != 3 or not all(math.isfinite(value) for value in values):
                raise ValueError("texture shading must contain three finite values")
            result["shading"] = values
        return result

    def apply_options(self, values):
        if "opacity" in values:
            self.set_opacity(values["opacity"])
        if "shading" in values:
            self.set_shading(*values["shading"])
        if "z_index" in values:
            self.set_z_index(values["z_index"])
            self.z_index = int(values["z_index"])
        if "depth_test" in values:
            if values["depth_test"]:
                self.apply_depth_test()
            else:
                self.deactivate_depth_test()
        if "is_fixed_in_frame" in values:
            # Camera projection defines a float mix, not a boolean switch.
            self.uniforms["is_fixed_in_frame"] = values["is_fixed_in_frame"]

    def surface_init(self, uv_surface, image_file, dark_image_file=None, **kwargs):
        if self._is_bound():
            raise RuntimeError("a texture constructor requires a detached target")
        with _editing(self):
            if not isinstance(uv_surface, Surface):
                raise TypeError("TexturedSurface uv_surface must be a Surface")
            values = options(kwargs)
            payload, path = _source(g, image_file)
            if path is not None:
                path = str(g["_pathlib"].Path(path).resolve())
            dark, dark_payload = path, None
            if dark_image_file is not None:
                dark_payload, dark = _source(g, dark_image_file)
                if dark is not None:
                    dark = str(g["_pathlib"].Path(dark).resolve())
                if path is not None and path == dark:
                    dark_payload = None
            metadata = {name: getattr(uv_surface, name) for name in (
                "resolution", "u_range", "v_range", "preferred_creation_axis", "epsilon", "normal_nudge",
            )}
            g["_install_live_state"](self)
            specs = g["_build_textured_surface"](self, uv_surface, payload, g["_native_surface_shell_factory"],
                                  dark_payload)
            g["_hang_native_children"](self, specs)
            self.__dict__.update(metadata)
            self.uv_surface = uv_surface
            self.image_file = self.image_path = path
            self.dark_image_file = dark
            self.num_textures = 1 if dark_payload is None else 2
            self.compute_triangle_indices()
            apply_options(self, values)

    def geometry_init(self, geometry, texture_file, **kwargs):
        if self._is_bound():
            raise RuntimeError("a texture constructor requires a detached target")
        with _editing(self):
            values = options(kwargs)
            # Accept the Reference's geometry protocol without importing trimesh.
            # The native mesh builder validates indices and constructs C-4 normals.
            try:
                vertices, faces, uv = geometry.vertices, geometry.faces, geometry.visual.uv
            except AttributeError:
                raise TypeError("TexturedGeometry requires vertices, triangle faces and visual.uv") from None
            vertices, faces, uv = np.asarray(vertices), np.asarray(faces), np.asarray(uv)
            if vertices.ndim != 2 or vertices.shape[1] != 3:
                raise ValueError("textured geometry vertices must have shape (N, 3)")
            if faces.ndim != 2 or faces.shape[1] != 3 or not len(faces):
                raise ValueError("textured geometry faces must contain triangles with shape (M, 3)")
            if uv.shape != (len(vertices), 2):
                raise ValueError("textured geometry requires one UV pair per vertex")
            if faces.dtype.kind not in "iu":
                raise TypeError("textured geometry triangle indices must be integers")
            payload, path = _source(g, texture_file)
            if path is not None:
                path = str(g["_pathlib"].Path(path).resolve())
            g["_install_live_state"](self)
            specs = g["_build_textured_geometry"](self, vertices, faces.reshape(-1), uv, payload,
                               g["_native_surface_shell_factory"])
            g["_hang_native_children"](self, specs)
            self.geometry = geometry
            self.image_file = self.image_path = self.texture_file = path
            self.dark_image_file = path
            self.num_textures = 1
            self.triangle_indices = np.arange(self.n_records(), dtype=int)
            apply_options(self, values)

    def get_opacities(self):
        return self.data["opacity"][:, 0]

    def get_opacity(self):
        values = self.get_opacities()
        return float(values[0]) if len(values) else float(self.opacity)

    def set_opacity(self, opacity, recurse=True):
        values = np.asarray(g["_listify"](opacity), dtype=float)
        if not np.isfinite(values).all() or np.any(np.abs(values) > np.finfo(np.float32).max):
            raise ValueError("texture opacity must be finite and f32-representable")
        g["ImageMobject"].set_opacity(self, opacity, recurse=recurse)
        if len(values):
            self.opacity = float(values.flat[0])
        return self

    def set_color(self, color, opacity=None, recurse=True):
        # Textures supply RGB; opacity still uses the native opacity column.
        del color
        if opacity is not None:
            self.set_opacity(opacity, recurse=recurse)
        return self

    def set_image_coords(self, uv_func):
        if not callable(uv_func):
            raise TypeError("texture coordinates require a callable UV map")
        if isinstance(self, Geometry):
            raise g["_CapabilityError"]("UV-grid remapping requires TexturedSurface, not an indexed TexturedGeometry")
        nu, nv = self.resolution
        values = np.asarray([uv_func(float(u), float(v))
                             for u in np.linspace(0, 1, nu)
                             for v in np.linspace(1, 0, nv)], dtype=float)
        if (values.shape != (self.n_records(), 2) or not np.isfinite(values).all()
                or np.any(np.abs(values) > np.finfo(np.float32).max)):
            raise ValueError("texture UV map must return two finite f32-representable coordinates per vertex")
        self.data["im_coords"][:] = values
        return self

    def mesh_indices(self):
        self.triangle_indices = np.arange(self.n_records(), dtype=int)
        return self.triangle_indices

    _method(Textured, "__init__", surface_init)
    _method(Geometry, "__init__", geometry_init)
    for name, function in (("get_opacities", get_opacities), ("get_opacity", get_opacity),
                           ("set_opacity", set_opacity), ("set_color", set_color),
                           ("set_image_coords_by_uv_func", set_image_coords)):
        _method(Textured, name, function)
    _method(Geometry, "compute_triangle_indices", mesh_indices)
    g["_FMN_SURFACE_TEXTURES_INSTALLED"] = True
