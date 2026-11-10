"""Host file-resolution/admission tests with explicit native parser doubles.

The double describes parser results; it never reimplements OBJ/MTL grammar.
Actual native grammar, records and pixels have separate acceptance suites.
"""
from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

_spec = importlib.util.spec_from_file_location(
    "obj_inputs", Path(__file__).parents[1] / "python/fmn_python/obj_models.py")
obj = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(obj)


class InputsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-obj-inputs-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.write("model.obj", b"exact OBJ bytes")
        self.doc = SimpleNamespace(libraries=[], material_names=[], part_materials=[])
        self.parsed, self.images = [], []
        self.mtls = {}
        def parse(data):
            self.assertEqual(data, b"exact OBJ bytes")
            return self.doc
        def mtl(data):
            self.parsed.append(data)
            return SimpleNamespace(entries=self.mtls[data])
        def decode(data):
            self.images.append(data)
            return SimpleNamespace(size=(2, 2), data=data)
        self.g = dict(_ObjDocument=parse, _ObjMaterials=mtl,
                      _RasterImage=SimpleNamespace(decode=decode))

    def write(self, path, data):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def prepare(self):
        return obj._prepare(self.g, self.root / "model.obj")

    def test_no_materials_open_only_the_obj(self):
        document, libraries, images, paths = self.prepare()
        self.assertIs(document, self.doc)
        self.assertEqual((libraries, images), ([], {}))
        self.assertEqual(paths, (str(self.root / "model.obj"),))

    def test_maps_belong_to_each_declaring_library_not_obj_or_cwd(self):
        self.doc.libraries = ["materials/a.mtl", "other/b.mtl"]
        self.doc.material_names = ["A", "B"]
        self.mtls = {b"a": [("A", "maps/color.png")], b"b": [("B", "color.png")]}
        self.write("materials/a.mtl", b"a"); self.write("other/b.mtl", b"b")
        self.write("materials/maps/color.png", b"correct-a")
        self.write("other/color.png", b"correct-b")
        self.write("color.png", b"incorrect-root")
        _, _, images, paths = self.prepare()
        self.assertEqual(images["A"].data, b"correct-a")
        self.assertEqual(images["B"].data, b"correct-b")
        self.assertNotIn(str(self.root / "color.png"), paths)

    def test_first_library_wins_and_unused_maps_are_not_opened(self):
        self.doc.libraries = ["a.mtl", "b.mtl", "a.mtl"]
        self.doc.material_names = ["A"]
        self.mtls = {b"a": [("A", "good.png"), ("unused", "missing.png")],
                     b"b": [("A", "wrong.png")]}
        self.write("a.mtl", b"a"); self.write("b.mtl", b"b"); self.write("good.png", b"good")
        _, libraries, images, _ = self.prepare()
        self.assertEqual(len(libraries), 2)
        self.assertEqual(images["A"].data, b"good")
        self.assertEqual(self.parsed, [b"a", b"b"])

    def test_shared_texture_is_read_and_decoded_once(self):
        self.doc.libraries = ["a.mtl"]
        self.doc.material_names = ["A", "B"]
        self.mtls = {b"a": [("A", "shared.png"), ("B", "shared.png")]}
        self.write("a.mtl", b"a"); self.write("shared.png", b"pixels")
        _, _, images, paths = self.prepare()
        self.assertIs(images["A"], images["B"])
        self.assertEqual(self.images, [b"pixels"])
        self.assertEqual(len(paths), 3)

    def test_missing_material_and_required_library_or_texture_do_not_fallback(self):
        self.doc.material_names = ["A"]
        with self.assertRaisesRegex(ValueError, "not defined"):
            self.prepare()
        self.doc.libraries = ["a.mtl"]
        with self.assertRaises(FileNotFoundError):
            self.prepare()
        self.write("a.mtl", b"a"); self.mtls = {b"a": [("A", "required.png")]}
        with self.assertRaises(FileNotFoundError):
            self.prepare()

    def test_original_decoder_exception_is_preserved(self):
        failure = ValueError("bad PNG")
        self.doc.libraries = ["a.mtl"]; self.doc.material_names = ["A"]
        self.mtls = {b"a": [("A", "bad.png")]}
        self.write("a.mtl", b"a"); self.write("bad.png", b"bad")
        with patch.object(self.g["_RasterImage"], "decode", side_effect=failure):
            with self.assertRaises(ValueError) as caught:
                self.prepare()
        self.assertIs(caught.exception, failure)

    def test_file_and_aggregate_byte_budgets_are_enforced_before_decode(self):
        with patch.object(obj, "_MAX_FILE_BYTES", 5):
            with self.assertRaisesRegex(ValueError, "byte budget"):
                self.prepare()
        inputs = obj._Inputs()
        a = self.write("a", b"1234"); b = self.write("b", b"5678")
        with patch.object(obj, "_MAX_TOTAL_BYTES", 7):
            self.assertEqual(inputs.read(a), b"1234")
            with self.assertRaisesRegex(ValueError, "byte budget"):
                inputs.read(b)
            self.assertEqual(inputs.size, 4)
            self.assertEqual(inputs.read(a), b"1234")

    def test_texture_binding_and_material_count_budgets_are_aggregate(self):
        self.doc.libraries = ["a.mtl"]; self.doc.material_names = ["A", "B"]
        self.mtls = {b"a": [("A", "same.png"), ("B", "same.png")]}
        self.write("a.mtl", b"a"); self.write("same.png", b"pixels")
        with patch.object(obj, "_MAX_TEXTURE_BYTES", 16):
            with self.assertRaisesRegex(ValueError, "bound textures"):
                self.prepare()
        with patch.object(obj, "_MAX_INPUTS", 1):
            with self.assertRaisesRegex(ValueError, "input count"):
                self.prepare()

    @unittest.skipUnless(hasattr(os, "mkfifo"), "requires FIFO support")
    def test_fifo_is_refused_without_blocking(self):
        path = self.root / "fifo"
        os.mkfifo(path)
        with self.assertRaisesRegex(ValueError, "regular file"):
            obj._Inputs().read(path)

    def test_urls_and_nontext_paths_are_not_downloader_capabilities(self):
        for path in ("https://example.invalid/model.obj", "a\0b"):
            with self.assertRaises(ValueError):
                obj._path(path)
        with self.assertRaises(TypeError):
            obj._path(b"binary-path")

    def test_early_bound_and_reentrant_constructor_guards_are_nonmutating(self):
        class Target:
            def _is_bound(self): return self.bound
        target = Target(); target.bound = True
        with self.assertRaisesRegex(RuntimeError, "detached"):
            with obj._construction(target): self.fail("entered bound target")
        target.bound = False
        with obj._construction(target):
            with self.assertRaisesRegex(RuntimeError, "reenter"):
                with obj._construction(target): self.fail("entered nested constructor")
        self.assertNotIn(obj._BUSY, vars(target))


if __name__ == "__main__":
    unittest.main()
