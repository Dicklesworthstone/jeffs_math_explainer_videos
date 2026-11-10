#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

python3 - <<'PY'
import importlib
import importlib.metadata
import sys

# The gate's host environment (fm-jfrr): CPython 3.13 with the installed
# wheel (it brings numpy==2.5.2) and IPython==9.14.0, the pin the repo's
# workflows use. source_autoreload drives a real embedded IPython, so a host
# without it stops here instead of four errors deep in the run.
try:
    ipython = importlib.metadata.version("ipython")
except importlib.metadata.PackageNotFoundError:
    raise SystemExit("clean-wheel gate host environment: install IPython==9.14.0 "
                     "(source_autoreload drives an embedded IPython)")
if ipython != "9.14.0":
    print(f"note: IPython {ipython} is installed; the gate's pin is 9.14.0", file=sys.stderr)
module = importlib.import_module("manimlib")
path = getattr(module, "__file__", None)
if not path:
    raise SystemExit("installed-wheel parity gate requires an importable manimlib with __file__")
print(f"auditing imported manimlib: {path}")
PY

for suite in paired_output_cli paired_output batch_checkpoint_acceptance capture_output copying_semantics copying_render tex_preamble tex_preamble_render decimal_authoring complex_matrix complex_readouts_render checkpoint_identity fill_profile_pixels fill_profile_animation functional_color pointwise_color pointwise_paint_integration portal_initialization point_editing point_free_positions text_authoring code_authoring persistent_recovery fading_family bridge animation_semantics matching_transform_semantics matching_authoring composition_lifecycle camera_animation_semantics camera_motion camera_execution updater_family drawing_runtime creation_semantics subset_reveal text_reveal native_outputs programmatic_rendering runtime_provenance batch_rendering reproducible_batch builder_playback restore_playback cyclic_replace deferred_transform_playback fading_semantics movement_semantics rotation_semantics indication_semantics deferred_indications composite_effects tracker_interpolation live_tex live_tex_render live_rates animation_updaters native_animation_updaters deferred_animation_updaters console_rendering scene_state temporal_visualization detached_tracing_tail streamline_authoring streamline_rebuild control_interaction color_sliders scene_execution matrix_authoring source_autoreload scene_project scene_project_editor live_graphing live_vector_fields video_options semantic_witness typeset_cache typeset_preflight typeset_static_preflight; do
    python3 - "$suite" <<'PY'
import ast
import pathlib
import runpy
import sys
import tomllib


def main_only(node):
    """A top-level `if __name__ == "__main__":` with no else branch."""
    if not isinstance(node, ast.If) or node.orelse or not isinstance(node.test, ast.Compare):
        return False
    test = node.test
    operands = [test.left, *test.comparators]
    return (len(test.ops) == 1 and isinstance(test.ops[0], ast.Eq)
            and any(isinstance(item, ast.Name) and item.id == "__name__" for item in operands)
            and any(isinstance(item, ast.Constant) and item.value == "__main__" for item in operands))


root = pathlib.Path.cwd()
version = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
suite = root / "crates" / "fmn-python" / "tests" / (sys.argv[1] + ".py")
# runpy names the module "<run_path>", so a suite that starts its tests only
# under `__main__` would run none of them here and still print "passed".
if any(main_only(node) for node in ast.parse(suite.read_text(encoding="utf-8"), str(suite)).body):
    raise SystemExit(f"{suite.name} starts its tests only under __main__, where this runner "
                     "runs none of them; run it directly instead")
runpy.run_path(str(suite), init_globals={"_expected_package_version": version})
print(f"installed-wheel acceptance passed: {suite.name}")
PY
done

# These suites start their tests only under `if __name__ == "__main__"`, so
# they run directly; the loop above refuses such a suite.
python3 crates/fmn-python/tests/surface_alignment.py
python3 crates/fmn-python/tests/textured_surfaces.py
python3 crates/fmn-python/tests/callback_cleanup.py

python3 crates/fmn-conformance/python/test_structural_facts.py
# Structural parity with the pinned Reference (fm-5wq.36): the installed
# portal must match the checked-in Reference facts except for BN/open-bead rows.
python3 crates/fmn-conformance/python/structural_facts.py check \
    --engine-id installed-wheel \
    --reference crates/fmn-conformance/fixtures/structural_facts/reference_constructions.v1.ndjson \
    --exclusions crates/fmn-conformance/fixtures/structural_facts/exclusions.json
# Appendix-A class sweep (fm-5wq.26): every public mobject class, default and
# declared calls, public methods included; any untriaged divergence fails.
python3 crates/fmn-conformance/python/structural_facts.py check \
    --engine-id installed-wheel \
    --sweep crates/fmn-conformance/fixtures/structural_facts/class_sweep.json \
    --reference crates/fmn-conformance/fixtures/structural_facts/reference_classes.v1.ndjson \
    --exclusions crates/fmn-conformance/fixtures/structural_facts/exclusions.json
python3 crates/fmn-python/tests/project_virtualenv.py
python3 crates/fmn-python/tests/test_runtime_source_ownership.py
python3 crates/fmn-python/tests/test_runtime_project_watch.py
python3 crates/fmn-python/tests/bundle_batching.py
python3 crates/fmn-python/tests/bundle_batch_console.py
python3 crates/fmn-python/tests/bundle_checkpoint_native.py
python3 crates/fmn-python/tests/camera_configuration.py
python3 crates/fmn-python/tests/camera_frame_lifecycle.py
python3 crates/fmn-python/tests/public_update_dispatch.py
python3 crates/fmn-python/tests/scene_update_phase.py
python3 crates/fmn-python/tests/native_scene_update_dispatch.py
python3 crates/fmn-python/tests/vector_group_protocol.py
python3 crates/fmn-python/tests/tracker_lifecycle.py
python3 crates/fmn-python/tests/raster_initialization.py
python3 crates/fmn-python/tests/raster_lifecycle.py
python3 crates/fmn-python/tests/live_surface_mesh.py
python3 crates/fmn-python/tests/calculus_area.py
python3 crates/fmn-python/tests/surface_plotting.py
python3 crates/fmn-python/tests/surface_regridding.py
python3 crates/fmn-python/tests/surface_admission.py
python3 crates/fmn-python/tests/graph_admission.py
python3 crates/fmn-python/tests/coordinate_mapping.py
python3 crates/fmn-python/tests/number_line_lifecycle.py
python3 crates/fmn-python/tests/decimal_lifecycle.py
python3 crates/fmn-python/tests/infinite_readouts.py
python3 crates/fmn-python/tests/string_lifecycle.py
python3 crates/fmn-python/tests/brace_lifecycle.py
python3 crates/fmn-python/tests/legacy_tex_lifecycle.py
python3 crates/fmn-python/tests/copied_string_regeneration.py
python3 crates/fmn-python/tests/coordinate_lifecycle.py
python3 crates/fmn-python/tests/plane_lifecycle.py
python3 crates/fmn-python/tests/coordinate_labels.py
python3 crates/fmn-python/tests/native_surface_sampling.py
python3 crates/fmn-python/tests/native_point_cloud_lifecycle.py
python3 crates/fmn-python/tests/native_point_cloud_materials.py
python3 crates/fmn-python/tests/native_solid_lifecycle.py
python3 crates/fmn-python/tests/native_axial_solids.py
python3 crates/fmn-python/tests/solid_group_lifecycle.py
python3 crates/fmn-python/tests/vector_solid_group_lifecycle.py
python3 crates/fmn-python/tests/native_surface_lifecycle.py
python3 crates/fmn-python/tests/surface_record_export.py
python3 crates/fmn-python/tests/native_graph_sampling.py
python3 crates/fmn-python/tests/live_implicit.py
python3 crates/fmn-python/tests/native_implicit_lifecycle.py
python3 crates/fmn-python/tests/live_curves.py
python3 crates/fmn-python/tests/native_curve_lifecycle.py
python3 crates/fmn-python/tests/native_geometry_lifecycle.py
python3 crates/fmn-python/tests/native_brace_lifecycle.py
python3 crates/fmn-python/tests/native_derived_curves.py
python3 crates/fmn-python/tests/native_arrow_lifecycle.py
python3 crates/fmn-python/tests/native_curved_lifecycle.py
python3 crates/fmn-python/tests/test_curved_constructor_protocol.py
python3 crates/fmn-python/tests/native_shape_matcher_lifecycle.py
python3 crates/fmn-python/tests/test_shape_matcher_protocol.py
python3 crates/fmn-python/tests/shape_matcher_lifecycle.py
python3 crates/fmn-python/tests/derived_line_lifecycle.py
python3 crates/fmn-python/tests/line_stroke_lifecycle.py
python3 crates/fmn-python/tests/native_polygon_lifecycle.py
python3 crates/fmn-python/tests/graph_callback_snapshots.py
python3 crates/fmn-python/tests/field_callback_snapshots.py
python3 crates/fmn-python/tests/vector_field_lifecycle.py
python3 crates/fmn-python/tests/vector_field_profiles.py
python3 crates/fmn-python/tests/test_invocation.py
python3 crates/fmn-python/tests/text_selection.py
python3 crates/fmn-python/tests/live_bar_chart.py
python3 crates/fmn-python/tests/native_network_layout.py
python3 crates/fmn-python/tests/network_graph.py
python3 crates/fmn-python/tests/native_table.py
python3 crates/fmn-python/tests/live_table.py
python3 crates/fmn-python/tests/native_markdown.py
python3 crates/fmn-python/tests/live_markdown.py
python3 crates/fmn-python/tests/native_math_markdown.py
python3 crates/fmn-python/tests/markdown_documents.py
python3 crates/fmn-python/tests/markdown_restoration.py
python3 crates/fmn-python/tests/native_svg_paints.py
python3 crates/fmn-python/tests/svg_paint_ingress.py
python3 crates/fmn-python/tests/svg_lifecycle.py

# Runtime inventories exercise real files; protocol sinks are explicitly doubled.
python3 crates/fmn-python/tests/test_bundle_checkpoint_protocol.py
python3 crates/fmn-python/tests/test_bundle_resume_cli_protocol.py
python3 crates/fmn-python/tests/test_paired_output_protocol.py
# A unittest main() suite: the runpy loop above uses run_name "<run_path>",
# where its `if __name__ == "__main__"` entry never runs a test (fm-djox).
python3 crates/fmn-python/tests/paired_checkpoint_unit.py
python3 crates/fmn-python/tests/test_scene_attributes.py
python3 crates/fmn-python/tests/test_project_editor_facade.py
python3 crates/fmn-python/tests/test_batch_checkpoint_identity.py
python3 crates/fmn-python/tests/test_batch_checkpoint_inputs.py
python3 crates/fmn-python/tests/test_runtime_identity.py
python3 crates/fmn-python/tests/test_sampling_callbacks_protocol.py
python3 crates/fmn-python/tests/test_runtime_provenance_protocol.py
python3 crates/fmn-python/tests/test_reproducible_batch_protocol.py
python3 crates/fmn-python/tests/test_reproducible_batch_cli_protocol.py
python3 crates/fmn-python/tests/test_persistent_scene_ownership.py
python3 crates/fmn-python/tests/test_solid_lifecycle_protocol.py

# These unittest suites include native geometry/clock and failure-recovery cases.
FMN_TEST_NATIVE=1 python3 crates/fmn-python/tests/test_speed.py
FMN_TEST_NATIVE=1 python3 crates/fmn-python/tests/test_speed_updaters.py

# Every protocol suite, not only the ones named above, so a fake Scene or
# lifted namespace that falls behind production code fails here (fm-fmgv:
# 52 such errors went unseen for a day).
python3 -m unittest discover -s crates/fmn-python/tests -p 'test_*.py'

report_file="$(mktemp)"
echo "retaining installed-wheel parity receipt: $report_file" >&2
set +e
python3 -m fmn_python --audit-parity --robot >"$report_file"
audit_status=$?
set -e
cat "$report_file"
if [[ "$audit_status" -ne 0 ]]; then
    exit "$audit_status"
fi

python3 scripts/verify_portal_runtime_receipt.py "$report_file"
