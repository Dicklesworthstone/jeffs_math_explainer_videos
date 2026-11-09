"""The one production initializer for embedded and installed portal modules.

The native extension supplies the object model and shared animation protocol.
All entry points install these same adapters, in the same order, before exposing
that module. The pure-Python package is part of the portal, not an optional set
of monkey patches whose omission silently changes a scene's meaning.
"""
from __future__ import annotations

from importlib import import_module
from types import ModuleType


# Resolve the entire installation plan before mutating any class. In particular,
# an incomplete wheel must fail closed, not expose a partly initialized engine.
_STEPS = (
    ("grouping", "install_grouping"),
    ("camera_lifecycle", "install_scene_camera_configuration"),
    ("camera_frame_lifecycle", "install_camera_frame_lifecycle"),
    ("copying", "install_mobject_copying"),
    ("updater_dispatch", "install_updater_dispatch"),
    ("point_editing", "install_point_editing"),
    ("point_cloud_lifecycle", "install_point_cloud_lifecycle"),
    ("point_cloud_materials", "install_point_cloud_materials"),
    ("tracker_lifecycle", "install_tracker_lifecycle"),
    ("arrow_geometry", "install_arrow_geometry"),
    ("shape_matchers", "install_shape_matchers"),
    ("shape_matchers", "install_annotation_marks"),
    ("derived_lines", "install_derived_lines"),
    ("derived_curves", "install_derived_curves"),
    ("functional_color", "install_functional_color"),
    ("rendering", "install_scene_rendering"),
    ("camera_capture", "install_camera_capture"),
    ("surface_textures", "install_surface_textures"),
    ("image_authoring", "install_image_authoring"),
    ("raster_animation", "install_raster_animation"),
    ("raster_transform", "install_raster_transform"),
    ("obj_models", "install_obj_models"),
    ("surface_admission", "install_surface_admission"),
    ("graph_admission", "install_graph_admission"),
    ("coordinate_mapping", "install_coordinate_mapping"),
    ("number_line_lifecycle", "install_number_line_lifecycle"),
    ("coordinate_lifecycle", "install_axes_lifecycle"),
    ("plane_lifecycle", "install_plane_lifecycle"),
    ("surface_geometry", "install_surface_geometry"),
    ("surface_lifecycle", "install_surface_lifecycle"),
    ("solid_lifecycle", "install_solid_lifecycle"),
    ("solid_groups", "install_solid_groups"),
    ("vector_solid_groups", "install_vector_solid_groups"),
    ("surface_mesh", "install_surface_mesh"),
    ("surface_plotting", "install_surface_plotting"),
    ("svg_ingress", "install_svg_ingress"),
    ("text_authoring", "install_text_authoring"),
    ("string_lifecycle", "install_string_lifecycle"),
    ("brace_lifecycle", "install_brace_lifecycle"),
    ("legacy_tex_lifecycle", "install_legacy_tex_lifecycle"),
    ("decimal_authoring", "install_decimal_authoring"),
    ("coordinate_mapping", "install_coordinate_labels"),
    ("live_tex", "install_live_tex"),
    ("matrix", "install_matrix"),
    ("graphing", "install_graphing"),
    ("graphing", "install_implicit_regeneration"),
    ("curve_regeneration", "install_curve_regeneration"),
    ("bar_chart", "install_bar_chart"),
    ("calculus", "install_graph_calculus"),
    ("vector_fields", "install_vector_fields"),
    ("vector_field_lifecycle", "install_vector_field_lifecycle"),
    ("text_reveal", "install_text_reveal"),
    ("subset_reveal", "install_subset_reveal"),
    ("fading", "install_fading"),
    ("playback", "install_scene_playback"),
    ("surface_alignment", "install_surface_alignment"),
    ("camera_callbacks", "install_camera_callbacks"),
    ("scene_state", "install_scene_state"),
    ("streamline_authoring", "install_streamline_authoring"),
    ("streamline_rebuild", "install_streamline_rebuild"),
    ("streamlines", "install_streamlines"),
    ("streamline_animation", "install_streamline_animation"),
    ("traced_path", "install_traced_path"),
    ("interactive_editing", "install_interactive_editing"),
    ("interaction", "install_interaction"),
    ("control_events", "install_control_events"),
    ("color_sliders", "install_color_sliders"),
    ("scene_execution", "install_scene_execution"),
    ("matching", "install_matching"),
    ("matching", "install_matching_strings"),
    ("speed", "install_speed"),
    ("speed_updaters", "install_speed_updaters"),
    ("embedded_shell", "install_embedded_shell"),
    ("source_autoreload", "install_source_autoreload"),
    ("project_editor", "install_scene_project_editor"),
    ("typesetting", "install_typesetting"),
    ("runtime_provenance", "install_runtime_provenance"),
    ("paired_output", "install_paired_output"),
)
_STATE = "_FMN_PORTAL_RUNTIME_STATE"


def initialize(native: ModuleType) -> ModuleType:
    """Initialize one actual native module exactly once, or refuse publication.

    State belongs to the native module, never to this reusable Python module.
    Independent embedded interpreters/workers must not share installed classes,
    and repeated package imports must not stack wrappers or replace identities.
    """
    namespace = vars(native)
    state = namespace.get(_STATE)
    if state == "ready":
        return native
    if state is not None:
        raise ImportError(f"FrankenManim portal runtime initialization is {state}")
    if not namespace.get("_FMN_ANIMATION_SEMANTICS_INSTALLED", False):
        raise ImportError("FrankenManim shared animation semantics are not initialized")
    namespace[_STATE] = "initializing"
    try:
        from fmn_python import _ensure_exclusive_manimlib_namespace
        from fmn_python.schema_provenance import (
            SchemaProvenanceError, apply_schema_placeholder_provenance,
        )

        _ensure_exclusive_manimlib_namespace()
        installers = []
        for module_name, attribute in _STEPS:
            module = import_module("fmn_python." + module_name)
            installer = getattr(module, attribute, None)
            if not callable(installer):
                raise ImportError(f"missing portal runtime installer: {module_name}.{attribute}")
            installers.append(installer)
        try:
            apply_schema_placeholder_provenance(native)
        except SchemaProvenanceError as error:
            raise ImportError(f"invalid manimlib schema provenance: {error}") from error
        for installer in installers:
            installer(native)
        # Dispatch compares authored hooks against the completed runtime, not
        # intermediate implementations replaced by later shipping adapters.
        # Consume this one-shot barrier before publishing the native module.
        finalize_dispatch = namespace.pop("_fmn_finalize_transform_dispatch", None)
        if finalize_dispatch is not None:
            finalize_dispatch()
        finalize_updaters = namespace.pop("_fmn_finalize_updater_dispatch", None)
        if finalize_updaters is not None:
            finalize_updaters()
    except BaseException:
        # Do not retain an exception/traceback cycle owning native proxies. A
        # failed instance must be discarded, never retried as a success-shaped
        # mixture of two initialization attempts.
        namespace[_STATE] = "failed"
        raise
    namespace[_STATE] = "ready"
    return native
