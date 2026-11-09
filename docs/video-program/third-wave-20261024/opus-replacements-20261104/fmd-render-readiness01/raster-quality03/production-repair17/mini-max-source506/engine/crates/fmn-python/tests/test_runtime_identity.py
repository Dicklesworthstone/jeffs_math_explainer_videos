"""Runtime content identities tested on real files and installed metadata.

No native renderer is substituted here. Temporary input trees are retained so
failure evidence survives the test process; their paths are printed at exit.
"""
from __future__ import annotations

import csv
import importlib.metadata as metadata
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

_PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/runtime_identity.py"
_SPEC = importlib.util.spec_from_file_location("runtime_identity_under_test", _PATH)
identity = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(identity)

_ROOT = Path(tempfile.mkdtemp(prefix="fmn-runtime-identity-tests-"))


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(dir=_ROOT))
        self.groups = {}
        for role in ("cpython", "wheel", "numpy"):
            path = self.root / role
            path.write_bytes((role + " payload").encode())
            self.groups[role] = (role + " 1.0", [("payload", path)])
        self.abi = {"version": "CPython 3.13", "bits": 64}

    def capture(self):
        return identity.RuntimeSnapshot(lambda: (self.groups, self.abi))

    def test_same_inputs_have_stable_content_identity(self):
        first = self.capture()
        self.assertEqual(first.identities, self.capture().identities)
        first.verify()
        self.assertEqual(set(first.identities), {"cpython", "wheel", "numpy", "abi"})
        self.assertIn("installed-payload-sha256=", first.identities["wheel"])

    def test_changed_binary_without_version_change_invalidates(self):
        for role in ("cpython", "wheel", "numpy"):
            with self.subTest(role=role):
                snapshot = self.capture()
                self.groups[role][1][0][1].write_bytes(b"different binary")
                with self.assertRaisesRegex(identity.RuntimeIdentityError, role):
                    snapshot.verify()

    def test_same_size_same_mtime_edit_is_not_a_cache_hit(self):
        path = self.groups["numpy"][1][0][1]
        stamp = path.stat()
        snapshot = self.capture()
        path.write_bytes(b"X" * stamp.st_size)
        os.utime(path, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "numpy"):
            snapshot.verify()

    def test_added_and_removed_inventory_members_invalidate(self):
        snapshot = self.capture()
        path = self.root / "extra.py"
        path.write_bytes(b"extra")
        self.groups["wheel"][1].append(("extra.py", path))
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "wheel"):
            snapshot.verify()
        snapshot = self.capture()
        self.groups["wheel"][1].pop()
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "wheel"):
            snapshot.verify()

    def test_abi_and_package_labels_participate(self):
        snapshot = self.capture()
        self.abi["bits"] = 32
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "abi"):
            snapshot.verify()
        snapshot = self.capture()
        self.groups["wheel"] = ("wheel 2.0", self.groups["wheel"][1])
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "wheel"):
            snapshot.verify()

    def test_relocation_preserves_identity(self):
        old = self.capture().identities
        relocated = self.root / "relocated"
        relocated.mkdir()
        for role, (label, files) in self.groups.items():
            new = relocated / role
            new.write_bytes(files[0][1].read_bytes())
            self.groups[role] = (label, [(files[0][0], new)])
        self.assertEqual(old, self.capture().identities)

    def test_virtual_name_and_order_are_canonical(self):
        path = self.root / "second"
        path.write_bytes(b"second")
        self.groups["wheel"][1].append(("second", path))
        snapshot = self.capture()
        self.groups["wheel"][1].reverse()
        snapshot.verify()
        self.groups["wheel"][1][0] = ("renamed", path)
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "wheel"):
            snapshot.verify()

    def test_returned_mapping_cannot_rewrite_snapshot(self):
        snapshot = self.capture()
        values = snapshot.identities
        values["wheel"] = "forged"
        snapshot.verify()
        self.assertNotEqual(values, snapshot.identities)

    def test_missing_file_directory_and_fifo_refuse(self):
        for path in (self.root / "missing", self.root):
            with self.assertRaises(identity.RuntimeIdentityError):
                identity._hash_file(path, 1024)
        if hasattr(os, "mkfifo"):
            path = self.root / "fifo"
            os.mkfifo(path)
            with self.assertRaisesRegex(identity.RuntimeIdentityError, "regular file"):
                identity._hash_file(path, 1024)

    def test_single_total_and_count_budgets(self):
        with patch.object(identity, "_MAX_FILE_BYTES", 2), self.assertRaisesRegex(identity.RuntimeIdentityError, "byte budget"):
            self.capture()
        with patch.object(identity, "_MAX_TOTAL_BYTES", 2), self.assertRaisesRegex(identity.RuntimeIdentityError, "byte budget"):
            self.capture()
        with patch.object(identity, "_MAX_FILES", 2), self.assertRaisesRegex(identity.RuntimeIdentityError, "file budget"):
            self.capture()

    def test_duplicate_and_empty_inventory_refuse(self):
        self.groups["wheel"][1].append(self.groups["wheel"][1][0])
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "duplicate"):
            self.capture()
        self.groups["wheel"][1].clear()
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "missing runtime payload"):
            self.capture()

    def test_symlink_retarget_and_path_replacement_invalidate(self):
        real = self.root / "real"
        real.write_bytes(b"one")
        link = self.root / "link"
        link.symlink_to(real)
        self.groups["cpython"] = ("CPython 3.13", [("executable", link)])
        snapshot = self.capture()
        replacement = self.root / "other"
        replacement.write_bytes(b"two")
        replacement.replace(real)
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "cpython"):
            snapshot.verify()

    def test_replacement_during_read_is_refused(self):
        path = self.root / "race"
        path.write_bytes(b"initial")
        replacement = self.root / "race-next"
        replacement.write_bytes(b"changed")
        original = identity.os.fstat
        calls = []
        def fstat(fd):
            value = original(fd)
            calls.append(fd)
            if len(calls) == 2:
                replacement.replace(path)
            return value
        with patch.object(identity.os, "fstat", fstat), self.assertRaisesRegex(identity.RuntimeIdentityError, "changed while hashing"):
            identity._hash_file(path, 1024)

    def test_unknown_and_unbounded_identity_labels_refuse(self):
        for label in ("", "unknown", "bad\nlabel", "x" * 4097, None):
            with self.subTest(label=label), self.assertRaises(identity.RuntimeIdentityError):
                identity._label(label, "runtime")


class DistributionTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(dir=_ROOT))
        self.package = self.root / "example"
        self.package.mkdir()
        self.source = self.package / "__init__.py"
        self.source.write_text("__version__ = '1.0'\n")
        self.info = self.root / "example-1.0.dist-info"
        self.info.mkdir()
        (self.info / "METADATA").write_text("Metadata-Version: 2.1\nName: example\nVersion: 1.0\n")
        (self.info / "WHEEL").write_text("Wheel-Version: 1.0\n")
        self.entries = ["example/__init__.py", "example-1.0.dist-info/METADATA", "example-1.0.dist-info/WHEEL"]
        self.write_record()
        self.dist = next(metadata.distributions(path=[str(self.root)]))

    def write_record(self):
        with (self.info / "RECORD").open("w", newline="") as stream:
            csv.writer(stream).writerows((name, "", "") for name in self.entries)

    def inventory(self, required=None):
        with patch.object(identity.metadata, "distribution", return_value=self.dist):
            return identity._distribution("example", [self.source] if required is None else required)

    def test_real_importlib_inventory_and_module_binding(self):
        version, records = self.inventory()
        self.assertEqual(version, "1.0")
        self.assertEqual([name for name, _ in records], sorted(self.entries))

    def test_actual_bytes_not_record_claimed_hashes(self):
        self.source.write_text("# changed without rewriting RECORD\n")
        _, records = self.inventory()
        path = dict(records)["example/__init__.py"]
        self.assertEqual(identity._hash_file(path, 1024)[1], identity.hashlib.sha256(path.read_bytes()).hexdigest())

    def test_generated_cache_and_console_script_are_not_payload(self):
        self.entries += ["example/__pycache__/__init__.cpython-313.pyc", "../../../bin/f2py",
                         "example-1.0.dist-info/RECORD", "example-1.0.dist-info/INSTALLER"]
        self.write_record()
        _, records = self.inventory()
        self.assertEqual(len(records), 3)

    def test_bundled_libraries_and_package_data_are_payload(self):
        for name in ("example.libs/libblas.so", "example/data/table.bin"):
            path = self.root / name
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(b"native data")
            self.entries.append(name)
        self.write_record()
        self.assertEqual(len(self.inventory()[1]), 5)

    def test_unregistered_loaded_module_is_refused(self):
        other = self.root / "extra.py"
        other.write_text("pass")
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "outside its installed payload"):
            self.inventory([other])

    def test_traversal_and_duplicate_record_are_refused(self):
        for name in ("../outside.so", "../bin/other", "/absolute.so", "example/../outside.py", "example/__init__.py"):
            self.entries.append(name)
            self.write_record()
            with self.subTest(name=name), self.assertRaises(identity.RuntimeIdentityError):
                self.inventory()
            self.entries.pop()

    def test_external_symlink_is_refused(self):
        outside = _ROOT / (self.root.name + "-outside")
        outside.write_bytes(b"outside")
        (self.package / "external.so").symlink_to(outside)
        self.entries.append("example/external.so")
        self.write_record()
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "escapes"):
            self.inventory()

    def test_missing_distribution_and_record_refuse(self):
        with patch.object(identity.metadata, "distribution", side_effect=metadata.PackageNotFoundError), self.assertRaisesRegex(identity.RuntimeIdentityError, "installed"):
            identity._distribution("missing", [])
        with patch.object(identity.metadata, "distribution", return_value=types.SimpleNamespace(read_text=lambda name: None)), self.assertRaisesRegex(identity.RuntimeIdentityError, "built wheel"):
            identity._distribution("missing", [])

    def test_missing_recorded_payload_cannot_be_filtered_out(self):
        self.entries.append("example/missing_module.py")
        self.write_record()
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "missing"):
            self.inventory()

    def test_malformed_and_oversized_record_refuse(self):
        (self.info / "RECORD").write_text("only-one-field\n")
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "malformed"):
            self.inventory()
        self.write_record()
        with patch.object(identity, "_MAX_RECORD_BYTES", 10), self.assertRaisesRegex(identity.RuntimeIdentityError, "byte budget"):
            self.inventory()

    def test_file_backed_native_extension_is_required(self):
        with self.assertRaisesRegex(identity.RuntimeIdentityError, "native extension"):
            identity.capture_runtime(types.SimpleNamespace(__file__="fake.py"))

    def test_loaded_runtime_modules_cannot_be_sourceless_bytecode(self):
        module = types.ModuleType("fmn_test_runtime")
        module.__file__ = "/fake/module.pyc"
        with patch.dict(sys.modules, {module.__name__: module}), self.assertRaisesRegex(identity.RuntimeIdentityError, "sourceless"):
            identity._module_paths((module.__name__,))

    def test_static_python_library_and_missing_shared_library(self):
        with patch.object(identity.sysconfig, "get_config_var", side_effect=lambda key: 0 if key == "Py_ENABLE_SHARED" else None):
            self.assertEqual(identity._python_library(), [])
        with patch.object(identity.sysconfig, "get_config_var", return_value=None), self.assertRaisesRegex(identity.RuntimeIdentityError, "configuration"):
            identity._python_library()
        with patch.object(identity.sysconfig, "get_config_var", side_effect=lambda key: 1 if key == "Py_ENABLE_SHARED" else None), self.assertRaisesRegex(identity.RuntimeIdentityError, "shared CPython"):
            identity._python_library()


_PORTAL_PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/__init__.py"
_PORTAL_SPEC = importlib.util.spec_from_file_location("fmn_python_namespace_under_test", _PORTAL_PATH)
portal = importlib.util.module_from_spec(_PORTAL_SPEC)
_PORTAL_SPEC.loader.exec_module(portal)


class NamespaceCollisionTests(unittest.TestCase):
    """The exclusive-manimlib scan over real dist-info trees (fm-5wq.38)."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp(dir=_ROOT))
        (self.root / "manimlib").mkdir()
        (self.root / "manimlib" / "__init__.py").write_text("")
        self.stats = 0
        locate = metadata.PathDistribution.locate_file

        def counted(distribution, path):
            self.stats += 1
            return locate(distribution, path)

        patcher = patch.object(metadata.PathDistribution, "locate_file", counted)
        patcher.start()
        self.addCleanup(patcher.stop)

    def install(self, name, paths):
        info = self.root / f"{name}-1.0.dist-info"
        info.mkdir()
        (info / "METADATA").write_text(f"Metadata-Version: 2.1\nName: {name}\nVersion: 1.0\n")
        with (info / "RECORD").open("w", newline="") as stream:
            csv.writer(stream).writerows((path, "", "") for path in paths)

    def providers(self):
        with patch.object(portal, "_distributions",
                          lambda: metadata.distributions(path=[str(self.root)])):
            return portal._foreign_manimlib_providers()

    def test_an_existing_claim_is_a_collision_and_a_missing_one_is_not(self):
        self.install("franken-manim", ["manimlib/__init__.py"])
        self.install("claims-existing", ["manimlib/__init__.py"])
        self.install("claims-missing", ["manimlib/gone.py"])
        # Distribution.files drops RECORD rows missing on disk; so does the scan.
        self.assertEqual(self.providers(), ("claims-existing",))

    def test_unrelated_files_are_never_statted(self):
        many = [f"bulky/module_{index}.py" for index in range(500)]
        for index in range(0, 500, 50):
            (self.root / "bulky").mkdir(exist_ok=True)
            (self.root / many[index]).write_text("")
        self.install("bulky", many)
        self.install("claims-existing", ["manimlib/__init__.py", *many[:3]])
        self.assertEqual(self.providers(), ("claims-existing",))
        # One located claim. Filtering Distribution.files stats all 503 rows.
        self.assertEqual(self.stats, 1)

    def test_the_installed_set_is_scanned_once_per_process(self):
        calls = []

        def scan():
            calls.append(1)
            return ("other-provider",)

        with patch.object(portal, "_scanned_providers", None), \
                patch.object(portal, "_foreign_manimlib_providers", scan):
            for _ in range(3):
                with self.assertRaises(portal._ManimlibNamespaceCollision) as refused:
                    portal._ensure_exclusive_manimlib_namespace()
                self.assertEqual(refused.exception.providers, ("other-provider",))
        self.assertEqual(len(calls), 1)


if __name__ == "__main__":
    print("retaining runtime-identity fixture inputs:", _ROOT)
    unittest.main()
