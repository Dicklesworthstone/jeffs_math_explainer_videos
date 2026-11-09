"""Identity orchestration tests; the Rust suite tests the actual filesystem scan."""
import hashlib
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest

from fmn_python.studio_inputs import StudioInputs, project_directory


def digest(value):
    return hashlib.sha256(value.encode()).hexdigest()


class Host:
    def __init__(self):
        self.fingerprint = digest("first")
        self.files = [("/project/scene.py", digest("scene")), ("/project/helper.py", digest("helper"))]
        self.calls = []
        self.error = None

    def watch_sources(self, roots, debounce):
        self.calls.append((roots, debounce))
        if self.error:
            raise self.error
        return SimpleNamespace(fingerprint=self.fingerprint, files=list(self.files))


class StudioInputTests(unittest.TestCase):
    def setUp(self):
        self.host = Host()
        self.native = SimpleNamespace(_StudioHost=self.host)
        self.inputs = StudioInputs(self.native, ["/project"])

    def test_roundtrip_and_snapshot_does_not_poll(self):
        value = self.inputs.as_request()
        other = StudioInputs.from_request(self.native, value)
        self.assertEqual(other.fingerprint, self.inputs.fingerprint)
        self.assertEqual(self.host.calls, [(["/project"], 0)] * 2)
        value["roots"].append("/other")
        self.assertEqual(self.inputs.roots, ("/project",))

    def test_launch_refuses_changed_helper_or_asset_identity(self):
        request = self.inputs.as_request()
        self.host.fingerprint = digest("second")
        with self.assertRaisesRegex(RuntimeError, "between launch"):
            StudioInputs.from_request(self.native, request)

    def test_publication_refuses_changed_inputs(self):
        self.host.fingerprint = digest("asset edited")
        with self.assertRaisesRegex(RuntimeError, "during capture"):
            self.inputs.verify()

    def test_actual_compiled_bytes_match_the_frozen_snapshot(self):
        loaded = SimpleNamespace(source_digests={Path(path): value for path, value in self.host.files})
        self.inputs.verify(loaded)
        loaded.source_digests[Path("/project/helper.py")] = digest("temporary edit then reverted")
        with self.assertRaisesRegex(RuntimeError, "executed changed project source"):
            self.inputs.verify(loaded)

    def test_late_import_outside_declared_scope_is_not_silently_certified(self):
        loaded = SimpleNamespace(source_digests={Path("/other/helper.py"): digest("helper")})
        with self.assertRaisesRegex(RuntimeError, "undeclared project source.*watch_paths"):
            self.inputs.verify(loaded)

    def test_source_observation_cannot_disagree_with_native_scan(self):
        self.inputs.require_source(Path("/project/scene.py"), digest("scene"))
        with self.assertRaisesRegex(RuntimeError, "entry source differs"):
            self.inputs.require_source(Path("/project/scene.py"), digest("later scene"))

    def test_scanner_refusal_is_not_converted_to_an_empty_identity(self):
        self.host.error = RuntimeError("native source-watch byte budget exceeded")
        with self.assertRaisesRegex(RuntimeError, "budget exceeded"):
            self.inputs.verify()

    def test_request_rejects_missing_unknown_or_malformed_fields(self):
        for value in (None, {}, {"roots": ["/project"]},
                      {"roots": ["/project"], "fingerprint": "x"},
                      {"roots": ["/project"], "fingerprint": digest("first"), "extra": True}):
            with self.subTest(value=value), self.assertRaises(ValueError):
                StudioInputs.from_request(self.native, value)

    def test_roots_are_bounded_and_frozen_absolute_paths(self):
        for roots in ([], "path", ["relative"], ["/bad\0path"], ["/a/../b"],
                      ["/project", "/project"], ["/p" + str(n) for n in range(65)], [None]):
            with self.subTest(roots=roots), self.assertRaises(ValueError):
                StudioInputs(self.native, roots)
        self.assertEqual(len(self.host.calls), 1, "invalid input reached the scanner")

    def test_native_file_table_is_a_frozen_copy(self):
        self.host.files[0] = ("/project/scene.py", digest("replaced"))
        self.inputs.require_source(Path("/project/scene.py"), digest("scene"))

    def test_package_root_contains_parent_initializers_and_sibling_helpers(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            package = root / "project"
            nested = package / "scenes"
            nested.mkdir(parents=True)
            (package / "__init__.py").write_text("")
            (nested / "__init__.py").write_text("")
            self.assertEqual(project_directory(nested / "scene.py"), package)
            self.assertEqual(project_directory(root / "standalone.py"), root)

    def test_non_package_subdirectory_keeps_its_sibling_root(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            child = root / "scenes"
            child.mkdir()
            (root / "__init__.py").write_text("")
            self.assertEqual(project_directory(child / "scene.py"), child)


if __name__ == "__main__":
    unittest.main()


def fake_snapshot(roots, debounce_ms):
    """Only a protocol fixture; source_identity.rs exercises the native scanner."""
    files, markers = {}, []
    for root in roots:
        path = Path(root)
        markers.append((root, "directory" if path.is_dir() else "file" if path.is_file() else "missing"))
        candidates = path.rglob("*.py") if path.is_dir() else ([path] if path.is_file() else [])
        for file in candidates:
            files[str(file)] = hashlib.sha256(file.read_bytes()).hexdigest()
    fingerprint = digest(repr((sorted(set(markers)), sorted(files.items()))))
    return SimpleNamespace(fingerprint=fingerprint, files=sorted(files.items()),
                           poll=lambda: False, watched=(roots, debounce_ms))
