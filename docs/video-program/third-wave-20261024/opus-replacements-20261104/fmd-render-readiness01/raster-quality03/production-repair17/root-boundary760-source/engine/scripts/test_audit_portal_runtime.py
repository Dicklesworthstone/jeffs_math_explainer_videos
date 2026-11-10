from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
import sys
import tempfile
import types
import unittest
from pathlib import Path

import audit_portal_runtime as audit
from fmn_python.schema_provenance import SCHEMA_PROVENANCE_VERSION


def skip_or_fail(test: unittest.TestCase, name: str, missing: str) -> None:
    """A required input is absent (fm-5wq.48): fail under
    FMN_REQUIRE_FULL_INPUTS=1, otherwise record a SKIPPED ledger line and skip."""
    if os.environ.get("FMN_REQUIRE_FULL_INPUTS") == "1":
        test.fail(f"{name}: required input missing: {missing} (FMN_REQUIRE_FULL_INPUTS=1)")
    line = f"SKIPPED {name} {missing}\n"
    sys.stderr.write(line)
    ledger = os.environ.get("FMN_SKIP_LEDGER")
    if ledger:
        with open(ledger, "a", encoding="utf-8") as handle:
            handle.write(line)
    test.skipTest(f"{missing} is absent")


def status_text(*rows: str) -> str:
    return "\n".join(("[status]", "# symbol\tstatus\tevidence\ttests\tnotes", *rows, ""))


def row(symbol: str, status: str = "same") -> str:
    return f"{symbol}\t{status}\tevidence\ttests\tnote"


def mark_placeholder(value, *, kind: str, symbol: str):
    value._fmn_schema_placeholder = True
    value._fmn_schema_placeholder_kind = kind
    value._fmn_schema_placeholder_symbol = symbol
    return value


class RuntimeAuditTests(unittest.TestCase):
    def module(self, name: str) -> types.ModuleType:
        module = types.ModuleType(name)
        self.addCleanup(sys.modules.pop, name, None)
        sys.modules[name] = module
        return module

    def test_reviewed_real_function_and_method_pass(self) -> None:
        module = self.module("fake_portal.real")

        def function():
            return 1

        class Widget:
            def method(self):
                return 2

        module.function = function
        module.Widget = Widget
        rows = audit.parse_status_rows(
            status_text(
                row("fake_portal.real:function", "same"),
                row("fake_portal.real:Widget.method", "improved"),
            )
        )
        report = audit.audit_rows(rows)
        self.assertTrue(report["ok"])
        self.assertEqual(report["counts"]["reviewed_implemented"], 2)
        self.assertEqual(report["counts"]["contradictions"], 0)

    def test_schema_placeholders_fail_reviewed_claims(self) -> None:
        module = self.module("fake_portal.placeholder")

        def unavailable():
            raise NotImplementedError

        mark_placeholder(
            unavailable,
            kind="function",
            symbol="fake_portal.placeholder:function",
        )
        module.function = unavailable
        rows = audit.parse_status_rows(status_text(row("fake_portal.placeholder:function")))
        report = audit.audit_rows(rows)
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["runtime_placeholders"], 1)
        contradiction = report["contradictions"][0]
        self.assertEqual(contradiction["code"], "reviewed-symbol-is-placeholder")
        self.assertIn("kind=function", contradiction["detail"])
        self.assertIn("declared=fake_portal.placeholder:function", contradiction["detail"])

    def test_wrapped_placeholder_descriptors_fail_reviewed_claims(self) -> None:
        module = self.module("fake_portal.wrapped_placeholder")

        def static_unavailable():
            raise NotImplementedError

        def class_unavailable(cls):
            raise NotImplementedError(cls)

        mark_placeholder(
            static_unavailable,
            kind="method",
            symbol="fake_portal.wrapped_placeholder:Widget.static_unavailable",
        )
        mark_placeholder(
            class_unavailable,
            kind="method",
            symbol="fake_portal.wrapped_placeholder:Widget.class_unavailable",
        )

        class Widget:
            pass

        Widget.static_unavailable = staticmethod(static_unavailable)
        Widget.class_unavailable = classmethod(class_unavailable)
        module.Widget = Widget
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(
                    row(
                        "fake_portal.wrapped_placeholder:Widget.static_unavailable",
                        "same",
                    ),
                    row(
                        "fake_portal.wrapped_placeholder:Widget.class_unavailable",
                        "improved",
                    ),
                )
            )
        )
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["runtime_placeholders"], 2)
        self.assertEqual(
            {item["symbol"] for item in report["contradictions"]},
            {
                "fake_portal.wrapped_placeholder:Widget.static_unavailable",
                "fake_portal.wrapped_placeholder:Widget.class_unavailable",
            },
        )
        self.assertEqual(
            {item["code"] for item in report["contradictions"]},
            {"reviewed-symbol-is-placeholder"},
        )

    def test_synthesized_class_identity_fails_reviewed_claim(self) -> None:
        module = self.module("fake_portal.placeholder_class")

        class Generated:
            pass

        mark_placeholder(
            Generated,
            kind="class",
            symbol="fake_portal.placeholder_class:Generated",
        )
        module.Generated = Generated
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(row("fake_portal.placeholder_class:Generated"))
            )
        )
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["runtime_placeholders"], 1)
        self.assertEqual(
            report["contradictions"][0]["code"],
            "reviewed-symbol-is-placeholder",
        )

    def test_placeholder_owner_invalidates_inherited_lifecycle_member(self) -> None:
        module = self.module("fake_portal.placeholder_owner")

        class Generated:
            def setup(self):
                return None

        mark_placeholder(
            Generated,
            kind="class",
            symbol="fake_portal.placeholder_owner:Generated",
        )
        module.Generated = Generated
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(row("fake_portal.placeholder_owner:Generated.setup"))
            )
        )
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["runtime_placeholders"], 1)
        contradiction = report["contradictions"][0]
        self.assertEqual(
            contradiction["code"],
            "reviewed-symbol-has-placeholder-owner",
        )
        self.assertIn("fake_portal.placeholder_owner:Generated", contradiction["detail"])

    def test_placeholder_marker_is_direct_not_inherited(self) -> None:
        module = self.module("fake_portal.direct_marker")

        class Generated:
            pass

        mark_placeholder(
            Generated,
            kind="class",
            symbol="fake_portal.direct_marker:Generated",
        )

        class Authored(Generated):
            value = 7

        module.Authored = Authored
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(row("fake_portal.direct_marker:Authored.value"))
            )
        )
        self.assertTrue(report["ok"])
        self.assertEqual(report["counts"]["runtime_placeholders"], 0)

    def test_tiered_and_excluded_placeholders_do_not_claim_implementation(self) -> None:
        module = self.module("fake_portal.boundary")

        def unavailable():
            raise NotImplementedError

        mark_placeholder(
            unavailable,
            kind="function",
            symbol="fake_portal.boundary:tiered",
        )
        module.tiered = unavailable
        module.excluded = unavailable
        rows = audit.parse_status_rows(
            status_text(
                row("fake_portal.boundary:tiered", "tiered"),
                row("fake_portal.boundary:excluded", "excluded"),
            )
        )
        report = audit.audit_rows(rows)
        self.assertTrue(report["ok"])
        self.assertEqual(report["counts"]["reviewed_implemented"], 0)
        self.assertEqual(report["counts"]["runtime_placeholders"], 0)

    def test_missing_reviewed_symbol_and_module_fail_closed(self) -> None:
        self.module("fake_portal.missing_symbol")
        rows = audit.parse_status_rows(
            status_text(
                row("fake_portal.missing_symbol:nope"),
                row("fake_portal.no_such_module:nope"),
            )
        )
        report = audit.audit_rows(rows)
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["missing_reviewed"], 2)
        self.assertEqual(
            {item["code"] for item in report["contradictions"]},
            {"missing-reviewed-symbol", "module-import-failed"},
        )

    def test_manimlib_rows_require_schema_provenance(self) -> None:
        module = types.ModuleType("manimlib.audit_fixture")
        module.value = 1
        rows = audit.parse_status_rows(
            status_text(row("manimlib.audit_fixture:value"))
        )
        importer = lambda _name: module

        missing = audit.audit_rows(rows, importer=importer)
        self.assertFalse(missing["ok"])
        self.assertEqual(
            missing["contradictions"][0]["code"],
            "runtime-schema-provenance-missing",
        )

        module._fmn_schema_provenance_version = SCHEMA_PROVENANCE_VERSION + 1
        mismatched = audit.audit_rows(rows, importer=importer)
        self.assertFalse(mismatched["ok"])
        self.assertEqual(
            mismatched["contradictions"][0]["code"],
            "runtime-schema-provenance-version-mismatch",
        )

        module._fmn_schema_provenance_version = SCHEMA_PROVENANCE_VERSION
        valid = audit.audit_rows(rows, importer=importer)
        self.assertTrue(valid["ok"])

    def test_dynamic_resolution_failure_is_a_bounded_contradiction(self) -> None:
        module = self.module("fake_portal.dynamic_failure")
        message = "x" * 10_000

        def dynamic(name):
            raise RuntimeError(f"{name}:{message}")

        module.__getattr__ = dynamic
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(row("fake_portal.dynamic_failure:explodes"))
            )
        )
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["missing_reviewed"], 1)
        contradiction = report["contradictions"][0]
        self.assertEqual(
            contradiction["code"],
            "reviewed-symbol-resolution-failed",
        )
        self.assertLess(len(contradiction["detail"]), 4_300)
        self.assertTrue(contradiction["detail"].endswith("…"))

    def test_static_resolution_does_not_execute_descriptors(self) -> None:
        module = self.module("fake_portal.descriptor")

        class ExplosiveDescriptor:
            def __get__(self, instance, owner):
                raise RuntimeError("descriptor executed")

        class Widget:
            member = ExplosiveDescriptor()

        module.Widget = Widget
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(row("fake_portal.descriptor:Widget.member"))
            )
        )
        self.assertTrue(report["ok"])

    def test_contradictions_are_sorted_by_symbol(self) -> None:
        module = self.module("fake_portal.sort")
        rows = audit.parse_status_rows(
            status_text(
                row("fake_portal.sort:zeta"),
                row("fake_portal.sort:alpha"),
            )
        )
        report = audit.audit_rows(rows)
        self.assertEqual(
            [item["symbol"] for item in report["contradictions"]],
            ["fake_portal.sort:alpha", "fake_portal.sort:zeta"],
        )

    def test_parser_rejects_duplicate_unknown_and_malformed_rows(self) -> None:
        cases = (
            status_text(row("fake:x"), row("fake:x")),
            status_text(row("fake:x", "pretend")),
            "[status]\nfake:x\tsame\ttoo\tfew\n",
            "[status]\nmissing-colon\tsame\te\tt\tn\n",
            "[status]\n",
        )
        for text in cases:
            with self.subTest(text=text):
                with self.assertRaises(audit.AuditError):
                    audit.parse_status_rows(text)

    def test_json_envelope_binds_exact_overlay_bytes(self) -> None:
        module = self.module("fake_portal.json")
        module.x = 1
        text = status_text(row("fake_portal.json:x"))
        report = audit.audit_overlay(text)
        payload = json.loads(audit.render_json(report))
        self.assertEqual(payload["schema"], "fmn.portal.runtime-audit")
        self.assertEqual(payload["version"], 1)
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["overlay_sha256"],
            hashlib.sha256(text.encode("utf-8")).hexdigest(),
        )

    def test_main_check_returns_one_without_hiding_report(self) -> None:
        module = self.module("fake_portal.cli")

        def placeholder():
            raise NotImplementedError

        mark_placeholder(
            placeholder,
            kind="function",
            symbol="fake_portal.cli:placeholder",
        )
        module.placeholder = placeholder
        text = status_text(row("fake_portal.cli:placeholder"))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "overlay.tsv"
            path.write_text(text, encoding="utf-8")
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                code = audit.main(["--overlay", str(path), "--check"])
        payload = json.loads(stdout.getvalue())
        self.assertEqual(code, 1)
        self.assertEqual(stderr.getvalue(), "")
        self.assertFalse(payload["ok"])
        self.assertEqual(
            payload["overlay_sha256"],
            hashlib.sha256(text.encode("utf-8")).hexdigest(),
        )

    # fm-5wq.11: a placeholder-free stub must not substantiate a claim.
    def test_every_trivial_body_shape_fails_a_reviewed_claim(self) -> None:
        module = self.module("fake_portal.trivial")
        import numpy as np

        def body_pass(self):
            pass

        def body_none(self, value):
            return None

        def body_empty_string(value):
            return ""

        def body_zero(value):
            return 0

        def body_false(value):
            return False

        def body_empty_tuple(value):
            return ()

        def body_empty_list(self):
            return []

        def body_empty_dict(self):
            return {}

        def body_pair_of_empties(self, is_labelled=False):
            return ("", "")

        def body_constant_ignoring_arguments(args=None, config=None):
            return "videos"

        def body_empty_array(self):
            return np.empty((0, 3))

        shapes = {
            body_pass: "empty",
            body_none: "empty",
            body_empty_string: "empty",
            body_zero: "empty",
            body_false: "empty",
            body_empty_tuple: "empty",
            body_empty_list: "empty",
            body_empty_dict: "empty",
            body_pair_of_empties: "empty",
            body_constant_ignoring_arguments: "ignores-arguments",
            body_empty_array: "empty-array",
        }
        rows = []
        for function, shape in shapes.items():
            setattr(module, function.__name__, staticmethod(function))
            rows.append(row(f"fake_portal.trivial:{function.__name__}", "same"))
        report = audit.audit_rows(audit.parse_status_rows(status_text(*rows)))
        self.assertFalse(report["ok"])
        self.assertEqual(report["counts"]["trivial_bodies"], len(shapes))
        details = {entry["symbol"]: entry for entry in report["contradictions"]}
        for function, shape in shapes.items():
            entry = details[f"fake_portal.trivial:{function.__name__}"]
            self.assertEqual(entry["code"], "reviewed-symbol-has-trivial-body")
            self.assertIn(f"is {shape};", entry["detail"])

    def test_real_bodies_justified_hooks_and_refusals_are_not_trivial(self) -> None:
        module = self.module("fake_portal.nontrivial")

        def reads_argument(value):
            return value + 1

        def constant_without_arguments():
            return ".mp4"

        def fluent(self):
            return self

        def refuses(self):
            raise NotImplementedError("named refusal")

        def stub(self):
            pass

        for function in (reads_argument, constant_without_arguments, fluent, refuses, stub):
            setattr(module, function.__name__, function)
        justified = next(iter(audit_contract().TRIVIAL_BODY_JUSTIFICATIONS))
        justified_module_name, justified_path = justified.split(":", 1)
        justified_module = self.module(justified_module_name)
        owner_name, member = justified_path.split(".")
        setattr(justified_module, owner_name, type(owner_name, (), {member: stub}))
        report = audit.audit_rows(
            audit.parse_status_rows(
                status_text(
                    row("fake_portal.nontrivial:reads_argument"),
                    row("fake_portal.nontrivial:constant_without_arguments"),
                    row("fake_portal.nontrivial:fluent", "improved"),
                    row("fake_portal.nontrivial:refuses"),
                    row("fake_portal.nontrivial:stub", "excluded"),
                )
            ),
        )
        self.assertTrue(report["ok"], report["contradictions"])
        self.assertEqual(report["counts"]["trivial_bodies"], 0)
        self.assertIsNone(
            audit_contract()._trivial_body_contradiction(
                audit.StatusRow(justified, "same", "e", "t", "n"),
                stub,
            )
        )

    def test_trivial_body_justifications_are_verified_against_the_sources(self) -> None:
        """Each exemption must be true today: no silent allowlist growth."""
        import ast

        root = Path(__file__).resolve().parents[1]
        if not (root / "scripts/manim_ref/manimlib").is_dir():
            skip_or_fail(
                self,
                "scripts/test_audit_portal_runtime.py::"
                "test_trivial_body_justifications_are_verified_against_the_sources",
                "scripts/manim_ref (pinned Reference checkout)",
            )
        dispatch = (root / "crates/fmn-python/src/lib.rs").read_text(encoding="utf-8")
        table = audit_contract().TRIVIAL_BODY_JUSTIFICATIONS
        self.assertTrue(table)
        for symbol, reason in table.items():
            module_name, qualified = symbol.split(":", 1)
            name = qualified.rsplit(".", 1)[-1]
            if reason == "native-hook":
                self.assertIn(f'call_cached0(slf.as_any(), "{name}")', dispatch, symbol)
                continue
            self.assertEqual(reason, "reference-no-op", symbol)
            source = root / "scripts/manim_ref" / (module_name.replace(".", "/") + ".py")
            nodes = ast.parse(source.read_text(encoding="utf-8")).body
            target = None
            for part in qualified.split("."):
                target = next(
                    node
                    for node in nodes
                    if isinstance(node, (ast.ClassDef, ast.FunctionDef)) and node.name == part
                )
                nodes = target.body
            body = [
                statement
                for statement in target.body
                if not (
                    isinstance(statement, ast.Expr)
                    and isinstance(statement.value, ast.Constant)
                )
            ]
            abstract = any(
                isinstance(decorator, (ast.Name, ast.Attribute))
                and ast.unparse(decorator).endswith("abstractmethod")
                for decorator in target.decorator_list
            )
            empty = not body or (
                len(body) == 1
                and (
                    isinstance(body[0], ast.Pass)
                    or isinstance(body[0], ast.Return)
                    and (
                        body[0].value is None
                        or isinstance(body[0].value, ast.Constant)
                        and not body[0].value.value
                        or isinstance(body[0].value, (ast.List, ast.Tuple, ast.Dict))
                        and not ast.unparse(body[0].value).strip("[](){} ")
                    )
                )
            )
            self.assertTrue(abstract or empty, f"{symbol}: Reference body is not a no-op")


def audit_contract():
    import fmn_python.parity_audit as contract

    return contract


if __name__ == "__main__":
    unittest.main()
