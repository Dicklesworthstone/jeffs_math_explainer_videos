"""Historical delivery entry point, now a read-only wiring check.

The implementation is committed in the production sources. Retain this entry
point for previously queued builds without allowing them to replay source edits.
This verifies integration presence only; the native tests remain authoritative.
"""
from pathlib import Path


contracts = {
    "crates/fmn-studio/src/project_watch.rs": (
        "pub fn content_fingerprint", "pub fn source_files",
    ),
    "crates/fmn-python/src/portal_studio.rs": (
        "fn fingerprint(&self)", "self.watch.content_fingerprint().to_hex()",
        "fn files(&self)",
    ),
    "crates/fmn-python/python/fmn_python/studio.py": (
        "source_inputs=inputs.files", "StudioInputs.from_request",
        "inputs.verify(loaded)", "inputs.fingerprint", '"version": 2',
    ),
}
for name, markers in contracts.items():
    source = Path(name).read_text()
    for marker in markers:
        if marker not in source:
            raise RuntimeError(f"Studio input integration is missing: {name}: {marker}")
print("Studio input identity is integrated; no files changed")
