"""CameraFrame initialization over its authoritative native pose.

Own and validate that pose before authored hooks, then finalize dimensions and
placement through the public methods without replacing an observed core.
"""
from __future__ import annotations

from .camera_lifecycle import _bind


def install_camera_frame_lifecycle(native):
    g = vars(native)
    if g.get("_FMN_CAMERA_FRAME_LIFECYCLE_INSTALLED", False):
        return
    Frame = g["CameraFrame"]

    def __init__(
        self,
        frame_shape=g["_FRAME_SHAPE"],
        center_point=g["_ORIGIN"],
        fovy=45 * g["_DEG"],
        euler_axes="zxz",
        z_index=-1,
        **kwargs,
    ):
        # Validate and own constructor inputs before running authored hooks.
        # CameraFrame's positional methods use a native pose, not drawable
        # records, so that pose must already exist during Mobject initialization.
        self._core = g["_CameraFrameCore"](
            (float(frame_shape[0]), float(frame_shape[1])),
            g["_vec3"](center_point),
            float(fovy),
            euler_axes,
        )
        width, height = self._core.shape()
        center = self._core.center()
        fovy, euler_axes = self._core.field_of_view(), self._core.euler_axes()
        self._core.set_shape((2.0, 2.0))
        self._core.set_center((0.0, 0.0, 0.0))
        super(Frame, self).__init__(z_index=z_index, **kwargs)

        # The Reference installs a unit frame and camera uniforms AFTER base
        # hooks, then calls these public sizing/placement methods. Preserve
        # that finalization order and actual override dispatch. Do not replace
        # the core a hook may have observed or copied during initialization.
        self._core.set_orientation((0.0, 0.0, 0.0, 1.0))
        self._core.make_orientation_default()
        self._core.set_field_of_view(fovy)
        self._core.set_euler_axes(euler_axes)
        self._core.set_shape((2.0, 2.0))
        self._core.set_center((0.0, 0.0, 0.0))
        self.set_width(width, stretch=True)
        self.set_height(height, stretch=True)
        self.move_to(center)

    _bind(Frame, "__init__", __init__)
    g["_FMN_CAMERA_FRAME_LIFECYCLE_INSTALLED"] = True
