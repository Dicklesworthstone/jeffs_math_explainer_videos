"""Historical live-delivery entry point; production integration is already landed.

Keep older queued workflows safe: validate wiring, never rewrite current source.
The actual live-worker tests, not these marker checks, prove runtime behavior.
"""
from pathlib import Path


contracts = {
    "crates/fmn-python/src/portal_studio/live.rs": (
        "fn validate_inputs", "input_validator: Option<Py<PyAny>>",
        "advance(scene, request.frames, self.input_validator.as_ref())",
        "refresh(scene, self.input_validator.as_ref())",
        'mod input_identity_tests;',
    ),
    "crates/fmn-python/python/fmn_python/studio.py": (
        'scene.__dict__["_fmn_studio_validate_inputs"] = lambda: inputs.verify(loaded)',
        'scene.__dict__.pop("_fmn_studio_validate_inputs", None)',
    ),
}
for name, markers in contracts.items():
    source = Path(name).read_text()
    for marker in markers:
        if marker not in source:
            raise RuntimeError(f"Studio live integration is missing: {name}: {marker}")
print("Studio live input identity is integrated; no files changed")
