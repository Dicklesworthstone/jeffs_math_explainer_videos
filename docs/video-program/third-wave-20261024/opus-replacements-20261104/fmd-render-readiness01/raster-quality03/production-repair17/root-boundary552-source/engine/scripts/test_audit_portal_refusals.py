from __future__ import annotations

import ast
import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import audit_portal_refusals as audit


VALID_SOURCE = '''
import abc as _abc


def _refuse_unrouted(subject, entries):
    return subject, entries


class Base:
    @_abc.abstractmethod
    def get_bounds(self):
        raise NotImplementedError


class Concrete:
    def configure(self, enabled=False):
        _refuse_unrouted(type(self).__name__ + "()", [("enabled", enabled)])
        if enabled:
            raise NotImplementedError(
                "Concrete.configure(enabled=True) requires the native configuration seam"
            )
        return self
'''.lstrip()


class PortalRefusalAuditTests(unittest.TestCase):
    def fixture(self, source: str = VALID_SOURCE) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "bootstrap.py"
        path.write_text(source, encoding="utf-8")
        return path

    def test_valid_named_and_abstract_refusals_are_inventoried(self) -> None:
        path = self.fixture()
        inventory = audit.build_inventory(path)
        self.assertEqual(inventory["schema"], "fmn.portal.refusals")
        self.assertEqual(inventory["version"], 2)
        self.assertGreater(inventory["ast_nodes"], 0)
        self.assertGreater(inventory["ast_depth"], 0)
        self.assertEqual(inventory["counts"]["sites"], 3)
        self.assertEqual(inventory["counts"]["not_implemented"], 2)
        self.assertEqual(inventory["counts"]["abstract_bare"], 1)
        self.assertEqual(inventory["counts"]["refuse_unrouted"], 1)
        self.assertEqual(inventory["counts"]["violations"], 0)
        self.assertEqual(
            [site["qualname"] for site in inventory["sites"]],
            ["Base.get_bounds", "Concrete.configure", "Concrete.configure"],
        )

    def test_bare_blank_and_missing_message_refusals_fail_closed(self) -> None:
        source = '''
def bare():
    raise NotImplementedError


def empty_call():
    raise NotImplementedError()


def blank_call():
    raise NotImplementedError(f"")


def expanded_call(args):
    raise NotImplementedError(*args)
'''.lstrip()
        path = self.fixture(source)
        inventory = audit.build_inventory(path)
        messages = [
            message
            for violation in inventory["violations"]
            for message in violation["messages"]
        ]
        self.assertIn("bare NotImplementedError outside an abstract method", messages)
        self.assertIn("NotImplementedError call has no message", messages)
        self.assertIn("NotImplementedError message is blank", messages)
        self.assertIn("NotImplementedError message uses unverifiable *args expansion", messages)

        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = audit.main(["--source", str(path), "--check"])
        self.assertEqual(status, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("4 anonymous refusal site(s)", stderr.getvalue())

    def test_refuse_unrouted_requires_unambiguous_bounded_arguments(self) -> None:
        source = '''
def _refuse_unrouted(subject, entries=None):
    return subject, entries


def missing_subject():
    _refuse_unrouted()


def missing_entries():
    _refuse_unrouted("Thing()")


def blank_subject():
    _refuse_unrouted("", [("x", True)])


def empty_entries():
    _refuse_unrouted("Thing()", list())


def duplicate_subject():
    _refuse_unrouted("Thing()", [("x", True)], subject="Other()")


def duplicate_entries():
    _refuse_unrouted("Thing()", [("x", True)], entries=[("y", True)])


def expanded(args, kwargs):
    _refuse_unrouted(*args, **kwargs)
'''.lstrip()
        inventory = audit.build_inventory(self.fixture(source))
        messages = [
            message
            for violation in inventory["violations"]
            for message in violation["messages"]
        ]
        self.assertIn("_refuse_unrouted call has no subject", messages)
        self.assertIn("_refuse_unrouted call has no entries argument", messages)
        self.assertIn("_refuse_unrouted subject is blank", messages)
        self.assertIn("_refuse_unrouted entries are statically empty", messages)
        self.assertIn("_refuse_unrouted call supplies the subject more than once", messages)
        self.assertIn("_refuse_unrouted call supplies entries more than once", messages)
        self.assertIn("_refuse_unrouted call uses unverifiable *args expansion", messages)
        self.assertIn("_refuse_unrouted call uses unverifiable **kwargs expansion", messages)

    def test_refuse_unrouted_keyword_arguments_are_supported(self) -> None:
        source = '''
def _refuse_unrouted(class_name, entries):
    return class_name, entries


def configure(enabled=False):
    _refuse_unrouted(
        class_name="Thing()",
        entries=[("enabled", enabled)],
    )
'''.lstrip()
        inventory = audit.build_inventory(self.fixture(source))
        self.assertEqual(inventory["counts"]["refuse_unrouted"], 1)
        self.assertEqual(inventory["counts"]["violations"], 0)
        site = inventory["sites"][0]
        self.assertEqual(site["subject"], 'class_name="Thing()"'.split("=", 1)[1])
        self.assertIn("enabled", site["detail"])

    def test_abstractness_does_not_leak_into_nested_scopes(self) -> None:
        source = '''
import abc


class Outer:
    @abc.abstractmethod
    def method(self):
        class Inner:
            raise NotImplementedError
        def nested():
            raise NotImplementedError
        raise NotImplementedError
'''.lstrip()
        inventory = audit.build_inventory(self.fixture(source))
        sites = [site for site in inventory["sites"] if site["kind"] == "not_implemented"]
        self.assertEqual(len(sites), 3)
        inner = next(site for site in sites if site["qualname"] == "Outer.method.Inner")
        nested = next(site for site in sites if site["qualname"] == "Outer.method.nested")
        method = next(site for site in sites if site["qualname"] == "Outer.method")
        self.assertFalse(inner["abstract"])
        self.assertTrue(inner["violations"])
        self.assertFalse(nested["abstract"])
        self.assertTrue(nested["violations"])
        self.assertTrue(method["abstract"])
        self.assertFalse(method["violations"])

    def test_iterative_scanner_handles_deep_valid_ast_without_recursion(self) -> None:
        leaf: ast.stmt = ast.Raise(exc=ast.Name(id="NotImplementedError"), cause=None)
        for _ in range(1_500):
            leaf = ast.If(test=ast.Constant(value=True), body=[leaf], orelse=[])
        tree = ast.Module(body=[leaf], type_ignores=[])
        scanner = audit.RefusalScanner("")
        with mock.patch.object(audit, "MAX_AST_DEPTH", 2_000):
            nodes, depth = scanner.scan(tree)
        self.assertGreater(nodes, 1_500)
        self.assertGreaterEqual(depth, 1_500)
        self.assertEqual(len(scanner.sites), 1)
        self.assertIn("outside an abstract method", scanner.sites[0]["violations"][0])

    def test_json_output_is_canonical_and_deterministic(self) -> None:
        path = self.fixture()
        outputs: list[str] = []
        for _ in range(2):
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                status = audit.main(
                    ["--source", str(path), "--format", "json", "--check"]
                )
            self.assertEqual(status, 0)
            outputs.append(stdout.getvalue())
        self.assertEqual(outputs[0], outputs[1])
        payload = json.loads(outputs[0])
        self.assertEqual(
            outputs[0],
            json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n",
        )

    def test_output_ast_depth_and_site_budgets_refuse_before_payload(self) -> None:
        path = self.fixture()
        stdout = io.StringIO()
        stderr = io.StringIO()
        with mock.patch.object(audit, "MAX_OUTPUT_BYTES", 32):
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = audit.main(["--source", str(path), "--format", "json"])
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("32-byte output limit", stderr.getvalue())

        with mock.patch.object(audit, "MAX_AST_NODES", 1):
            with self.assertRaisesRegex(audit.AuditError, "1-node AST limit"):
                audit.build_inventory(path)

        with mock.patch.object(audit, "MAX_AST_DEPTH", 1):
            with self.assertRaisesRegex(audit.AuditError, "1-level AST depth limit"):
                audit.build_inventory(path)

        with mock.patch.object(audit, "MAX_SITES", 1):
            with self.assertRaisesRegex(audit.AuditError, "1-site limit"):
                audit.build_inventory(path)

    def test_syntax_parser_resource_and_utf8_errors_are_typed(self) -> None:
        syntax = self.fixture("def broken(:\n")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = audit.main(["--source", str(syntax), "--check"])
        self.assertEqual(status, 2)
        self.assertIn("not valid Python", stderr.getvalue())

        with mock.patch.object(audit.ast, "parse", side_effect=RecursionError("deep")):
            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                status = audit.main(["--source", str(self.fixture()), "--check"])
        self.assertEqual(status, 2)
        self.assertIn("exhausted parser resources", stderr.getvalue())

        binary = self.fixture()
        binary.write_bytes(b"\xff\n")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = audit.main(["--source", str(binary), "--check"])
        self.assertEqual(status, 2)
        self.assertIn("not valid UTF-8", stderr.getvalue())

    def test_source_symlink_and_identity_change_are_refused(self) -> None:
        target = self.fixture()
        link = target.with_name("linked.py")
        try:
            link.symlink_to(target)
        except OSError as exc:
            self.skipTest(f"host cannot create symlinks: {exc}")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = audit.main(["--source", str(link), "--check"])
        self.assertEqual(status, 2)
        self.assertIn("refusing symlink portal source", stderr.getvalue())

        real_lstat = audit.os.lstat
        calls = 0

        def changed(path):
            nonlocal calls
            metadata = real_lstat(path)
            calls += 1
            if calls != 2:
                return metadata
            fields = list(metadata)
            fields[1] += 1
            return os.stat_result(fields)

        with mock.patch.object(audit.os, "lstat", side_effect=changed):
            with self.assertRaisesRegex(audit.AuditError, "changed while opening"):
                audit.read_source(target)


if __name__ == "__main__":
    unittest.main()
