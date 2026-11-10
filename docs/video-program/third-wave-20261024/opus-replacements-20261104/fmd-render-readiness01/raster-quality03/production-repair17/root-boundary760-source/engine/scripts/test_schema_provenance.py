from __future__ import annotations

import sys
import types
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PORTAL_PYTHON = ROOT / "crates" / "fmn-python" / "python"
if str(PORTAL_PYTHON) not in sys.path:
    sys.path.insert(0, str(PORTAL_PYTHON))

from fmn_python import schema_provenance as provenance


def _surface_init(self, *args, **kwargs):
    self.args = args
    self.kwargs = kwargs


def _placeholder_function():
    def unavailable(*args, **kwargs):
        raise NotImplementedError((args, kwargs))

    return unavailable


def _placeholder_method():
    def unavailable(self, *args, **kwargs):
        raise NotImplementedError((self, args, kwargs))

    return unavailable


def _schema_init_refusal():
    def refused(self, *args, **kwargs):
        raise NotImplementedError((self, args, kwargs))

    return refused


def schema_text(*rows: str) -> str:
    return "\n".join(
        (
            "[meta]",
            "schema_version\t1",
            "[symbols]",
            "# module\tname\tkind\torigin\texported\tdetail",
            *rows,
            "",
        )
    )


def schema_row(module: str, name: str, kind: str) -> str:
    return f"{module}\t{name}\t{kind}\tdefined\t1\t-"


class SchemaProvenanceTests(unittest.TestCase):
    def set_module(self, name: str, module: types.ModuleType) -> None:
        previous = sys.modules.get(name)
        sys.modules[name] = module

        def restore() -> None:
            if previous is None:
                sys.modules.pop(name, None)
            else:
                sys.modules[name] = previous

        self.addCleanup(restore)

    def fixture(self):
        package = types.ModuleType("manimlib")
        package.__path__ = []
        child = types.ModuleType("manimlib.fake")
        native = types.ModuleType("manimlib.manimlib")
        self.set_module("manimlib", package)
        self.set_module("manimlib.fake", child)

        generated = type(
            "Generated",
            (object,),
            {
                "__module__": "manimlib.fake",
                "__init__": _surface_init,
                "missing": _placeholder_method(),
            },
        )
        refusing = type(
            "Refusing",
            (object,),
            {
                "__module__": "manimlib.fake",
                "__init__": _schema_init_refusal(),
            },
        )

        class Authored:
            def __init__(self):
                self.ready = True

            def real_method(self):
                return 7

        def real_function():
            return 11

        child.Generated = generated
        child.Refusing = refusing
        child.Authored = Authored
        child.placeholder_function = _placeholder_function()
        child.real_function = real_function
        native.root_placeholder = _placeholder_function()
        native._API_SCHEMA_TSV = schema_text(
            schema_row("manimlib", "root_placeholder", "function"),
            schema_row("manimlib.fake", "Generated", "class"),
            schema_row("manimlib.fake", "Generated.__init__", "method"),
            schema_row("manimlib.fake", "Generated.missing", "method"),
            schema_row("manimlib.fake", "Refusing", "class"),
            schema_row("manimlib.fake", "Refusing.__init__", "method"),
            schema_row("manimlib.fake", "Authored", "class"),
            schema_row("manimlib.fake", "Authored.real_method", "method"),
            schema_row("manimlib.fake", "placeholder_function", "function"),
            schema_row("manimlib.fake", "real_function", "function"),
            schema_row("manimlib.absent", "ignored", "function"),
        )
        return package, child, native

    def test_marks_only_schema_generated_runtime_objects(self) -> None:
        package, child, native = self.fixture()
        counts = provenance.apply_schema_placeholder_provenance(native)

        self.assertTrue(vars(child.Generated)["_fmn_schema_placeholder"])
        self.assertEqual(vars(child.Generated)["_fmn_schema_placeholder_kind"], "class")
        self.assertEqual(
            vars(child.Generated)["_fmn_schema_placeholder_symbol"],
            "manimlib.fake:Generated",
        )
        self.assertTrue(_surface_init._fmn_schema_placeholder)
        self.assertEqual(_surface_init._fmn_schema_placeholder_kind, "constructor")
        self.assertTrue(child.Generated.missing._fmn_schema_placeholder)
        self.assertEqual(
            child.Generated.missing._fmn_schema_placeholder_symbol,
            "manimlib.fake:Generated.missing",
        )
        self.assertTrue(vars(child.Refusing)["_fmn_schema_placeholder"])
        self.assertTrue(child.Refusing.__init__._fmn_schema_placeholder)
        self.assertEqual(
            child.Refusing.__init__._fmn_schema_placeholder_kind,
            "constructor-refusal",
        )
        self.assertTrue(child.placeholder_function._fmn_schema_placeholder)
        self.assertTrue(native.root_placeholder._fmn_schema_placeholder)
        self.assertNotIn("_fmn_schema_placeholder", vars(child.Authored))
        self.assertFalse(
            hasattr(child.Authored.real_method, "_fmn_schema_placeholder")
        )
        self.assertFalse(hasattr(child.real_function, "_fmn_schema_placeholder"))
        self.assertEqual(counts["classes"], 2)
        self.assertEqual(counts["constructors"], 2)
        self.assertEqual(counts["functions"], 2)
        self.assertEqual(counts["methods"], 1)

        for module in (package, child, native):
            self.assertEqual(
                module._fmn_schema_provenance_version,
                provenance.SCHEMA_PROVENANCE_VERSION,
            )
            self.assertIsInstance(module._fmn_schema_provenance_counts, tuple)

    def test_application_is_idempotent(self) -> None:
        _package, _child, native = self.fixture()
        first = provenance.apply_schema_placeholder_provenance(native)
        second = provenance.apply_schema_placeholder_provenance(native)
        self.assertEqual(first, second)

    def test_missing_or_malformed_schema_fails_closed(self) -> None:
        native = types.ModuleType("fake_native")
        with self.assertRaisesRegex(
            provenance.SchemaProvenanceError,
            "does not expose",
        ):
            provenance.apply_schema_placeholder_provenance(native)

        malformed = (
            "[symbols]\n"
            "manimlib.fake\tonly\tfive\tcolumns\there\n"
        )
        native._API_SCHEMA_TSV = malformed
        with self.assertRaisesRegex(
            provenance.SchemaProvenanceError,
            "exactly 6",
        ):
            provenance.apply_schema_placeholder_provenance(native)

        native._API_SCHEMA_TSV = "[symbols]\n"
        with self.assertRaisesRegex(
            provenance.SchemaProvenanceError,
            "no \[symbols\] rows",
        ):
            provenance.apply_schema_placeholder_provenance(native)


if __name__ == "__main__":
    unittest.main()
