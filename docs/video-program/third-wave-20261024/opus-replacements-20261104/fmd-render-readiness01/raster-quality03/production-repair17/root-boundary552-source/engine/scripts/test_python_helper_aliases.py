from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import check_python_helper_aliases as policy


class AliasPolicyTests(unittest.TestCase):
    def write(self, name: str, text: str) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / name
        path.write_text(text, encoding="utf-8")
        return path

    def schema(self, *, leaked: str | None = None, omit: str | None = None) -> Path:
        rows = []
        for alias, (module, class_name) in policy.REFERENCE_CONSTRUCTOR_ALIASES.items():
            if class_name != omit:
                rows.append(f"{module}\t{class_name}\tclass\tdefined\t1\t-")
            if alias == leaked:
                rows.append(f"{module}\t{alias}\tfunction\tdefined\t1\t-")
        return self.write(
            "API_SCHEMA.tsv",
            "[meta]\nschema_version\t1\n[symbols]\n"
            "# module\tname\tkind\torigin\texported\tdetail\n"
            + "\n".join(rows)
            + "\n",
        )

    def wheel(
        self,
        mapping: dict[str, tuple[str, str]] | None = None,
        *,
        call: bool = True,
        builders: set[str] | None = None,
        lambda_builders: bool = True,
        resolve_class: bool = True,
        require_module: bool = True,
        require_identity: bool = True,
        invoke_builder: bool = True,
        recheck_alias: bool = True,
        nested_call_decoy: bool = False,
    ) -> Path:
        selected = (
            policy.REFERENCE_CONSTRUCTOR_ALIASES if mapping is None else mapping
        )
        expected_builders = {
            class_name for _module, class_name in selected.values()
        }
        selected_builders = expected_builders if builders is None else builders
        if lambda_builders:
            builder_items = ", ".join(
                f"{name!r}: (lambda: None)" for name in sorted(selected_builders)
            )
        else:
            builder_items = ", ".join(
                f"{name!r}: None" for name in sorted(selected_builders)
            )
        constructor_line = (
            "        constructor = getattr(manimlib, class_name)\n"
            if resolve_class
            else "        constructor = None\n"
        )
        module_line = (
            "        require(module_name in sys.modules, 'module missing')\n"
            if require_module
            else "        pass\n"
        )
        identity_line = (
            "        require(\n"
            "            getattr(sys.modules[module_name], class_name, None) "
            "is constructor,\n"
            "            'identity split',\n"
            "        )\n"
            if require_identity
            else "        pass\n"
        )
        builder_line = (
            "        instance = builders[class_name]()\n"
            if invoke_builder
            else "        instance = None\n"
        )
        alias_line = (
            "        hasattr(manimlib, alias)\n"
            if recheck_alias
            else "        pass\n"
        )
        if call:
            call_line = "    verify_reference_constructor_aliases(manimlib)\n"
        elif nested_call_decoy:
            call_line = (
                "    def decoy():\n"
                "        verify_reference_constructor_aliases(manimlib)\n"
                "    return decoy\n"
            )
        else:
            call_line = "    pass\n"
        return self.write(
            "wheel_smoke.py",
            "import sys\n\n"
            "REFERENCE_CONSTRUCTOR_ALIASES = "
            + repr(selected)
            + "\n\n"
            + "def require(condition, message):\n"
            + "    if not condition:\n"
            + "        raise AssertionError(message)\n\n"
            + "def verify_reference_constructor_aliases(manimlib):\n"
            + f"    builders = {{{builder_items}}}\n"
            + "    for alias, (module_name, class_name) in "
            + "REFERENCE_CONSTRUCTOR_ALIASES.items():\n"
            + constructor_line
            + module_line
            + identity_line
            + builder_line
            + alias_line
            + "    return None\n\n"
            + "def verify_installed_distribution():\n"
            + "    manimlib = object()\n"
            + call_line,
        )

    def wrapper(
        self,
        mapping: dict[str, str] | None = None,
        *,
        class_contract: bool = True,
        module_contract: bool = True,
        helper_guard: bool = True,
        class_guard: bool = True,
        module_assignment: bool = True,
        module_lookup: bool = True,
        module_guard: bool = True,
        value_assignment: bool = True,
        identity_guard: bool = True,
        cleanup: bool = True,
    ) -> Path:
        expected = {
            alias: class_name
            for alias, (_module, class_name) in (
                policy.REFERENCE_CONSTRUCTOR_ALIASES.items()
            )
        }
        modules = {
            alias: module
            for alias, (module, _class_name) in (
                policy.REFERENCE_CONSTRUCTOR_ALIASES.items()
            )
        }
        selected = expected if mapping is None else mapping
        class_contract_block = (
            "if _REFERENCE_CLASS_BY_RUST_HELPER != "
            "_CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER:\n"
            "    raise ImportError('class contract drift')\n"
            if class_contract
            else ""
        )
        module_contract_block = (
            "if set(_REFERENCE_CLASS_BY_RUST_HELPER) != "
            "set(_CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER):\n"
            "    raise ImportError('module contract drift')\n"
            if module_contract
            else ""
        )
        helper_block = (
            "    if hasattr(_native, _rust_helper):\n"
            "        raise ImportError('helper leaked')\n"
            if helper_guard
            else ""
        )
        class_block = (
            "    if not hasattr(_native, _reference_class):\n"
            "        raise ImportError('class missing')\n"
            if class_guard
            else ""
        )
        module_assignment_line = (
            "    _reference_module = "
            "_CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER[_rust_helper]\n"
            if module_assignment
            else ""
        )
        module_lookup_line = (
            "    _module = _sys.modules.get(_reference_module)\n"
            if module_lookup
            else ""
        )
        module_guard_block = (
            "    if _module is None:\n"
            "        raise ImportError('module missing')\n"
            if module_guard
            else ""
        )
        value_assignment_line = (
            "    _reference_value = getattr(_native, _reference_class)\n"
            if value_assignment
            else ""
        )
        identity_guard_block = (
            "    if vars(_module).get(_reference_class) is not _reference_value:\n"
            "        raise ImportError('identity split')\n"
            if identity_guard
            else ""
        )
        cleanup_line = (
            "del _REFERENCE_CLASS_BY_RUST_HELPER, "
            "_CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER, "
            "_CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER\n"
            "del _rust_helper, _reference_class, _reference_module\n"
            "del _module, _reference_value, _sys\n"
            if cleanup
            else ""
        )
        return self.write(
            "__init__.py",
            "import sys as _sys\n"
            "_native = object()\n"
            "_CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER = "
            + repr(expected)
            + "\n"
            + "_CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER = "
            + repr(modules)
            + "\n"
            + "_REFERENCE_CLASS_BY_RUST_HELPER = "
            + repr(selected)
            + "\n"
            + class_contract_block
            + module_contract_block
            + "for _rust_helper, _reference_class in "
            + "_REFERENCE_CLASS_BY_RUST_HELPER.items():\n"
            + helper_block
            + class_block
            + module_assignment_line
            + module_lookup_line
            + module_guard_block
            + value_assignment_line
            + identity_guard_block
            + cleanup_line,
        )

    def verify(
        self,
        *,
        schema: Path | None = None,
        wheel: Path | None = None,
        wrapper: Path | None = None,
    ) -> None:
        policy.verify(
            self.schema() if schema is None else schema,
            self.wheel() if wheel is None else wheel,
            self.wrapper() if wrapper is None else wrapper,
        )

    def test_complete_policy_passes(self) -> None:
        self.verify()

    def test_snake_case_export_leak_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "leaked into Python exports",
        ):
            self.verify(schema=self.schema(leaked="small_dot"))

    def test_missing_reference_class_is_rejected(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "VHighlight is missing"):
            self.verify(schema=self.schema(omit="VHighlight"))

    def test_wheel_mapping_drift_is_rejected(self) -> None:
        mapping = dict(policy.REFERENCE_CONSTRUCTOR_ALIASES)
        mapping.pop("v_highlight")
        with self.assertRaisesRegex(policy.AliasPolicyError, "wheel mapping drift"):
            self.verify(wheel=self.wheel(mapping))

    def test_wheel_acceptance_must_call_runtime_probe(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not call"):
            self.verify(wheel=self.wheel(call=False))

    def test_nested_runtime_probe_call_is_not_accepted(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not call"):
            self.verify(
                wheel=self.wheel(
                    call=False,
                    nested_call_decoy=True,
                )
            )

    def test_wheel_constructor_table_must_cover_every_reference_class(self) -> None:
        builders = {
            class_name
            for _module, class_name in (
                policy.REFERENCE_CONSTRUCTOR_ALIASES.values()
            )
        }
        builders.remove("VHighlight")
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "constructor table drift",
        ):
            self.verify(wheel=self.wheel(builders=builders))

    def test_wheel_constructor_table_must_not_add_unrelated_classes(self) -> None:
        builders = {
            class_name
            for _module, class_name in (
                policy.REFERENCE_CONSTRUCTOR_ALIASES.values()
            )
        }
        builders.add("UnrelatedMobject")
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "constructor table drift",
        ):
            self.verify(wheel=self.wheel(builders=builders))

    def test_wheel_constructor_table_requires_explicit_lambdas(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "constructor lambda"):
            self.verify(wheel=self.wheel(lambda_builders=False))

    def test_wheel_probe_must_resolve_each_root_constructor(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not resolve"):
            self.verify(wheel=self.wheel(resolve_class=False))

    def test_wheel_probe_must_require_each_qualified_module(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "qualified module",
        ):
            self.verify(wheel=self.wheel(require_module=False))

    def test_wheel_probe_must_require_qualified_constructor_identity(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "qualified constructor identity",
        ):
            self.verify(wheel=self.wheel(require_identity=False))

    def test_wheel_probe_must_invoke_each_constructor_builder(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not invoke"):
            self.verify(wheel=self.wheel(invoke_builder=False))

    def test_wheel_probe_must_recheck_alias_absence_after_construction(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not recheck"):
            self.verify(wheel=self.wheel(recheck_alias=False))

    def test_wrapper_mapping_drift_is_rejected(self) -> None:
        mapping = {
            alias: class_name
            for alias, (_module, class_name) in (
                policy.REFERENCE_CONSTRUCTOR_ALIASES.items()
            )
        }
        mapping["small_dot"] = "Dot"
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "wrapper mapping drift",
        ):
            self.verify(wrapper=self.wrapper(mapping))

    def test_wrapper_must_check_class_manifest(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "literal class map",
        ):
            self.verify(wrapper=self.wrapper(class_contract=False))

    def test_wrapper_must_check_module_manifest(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "authority module map",
        ):
            self.verify(wrapper=self.wrapper(module_contract=False))

    def test_wrapper_must_reject_helper_leaks(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "Rust-only helper",
        ):
            self.verify(wrapper=self.wrapper(helper_guard=False))

    def test_wrapper_must_reject_missing_reference_classes(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "missing Reference class",
        ):
            self.verify(wrapper=self.wrapper(class_guard=False))

    def test_wrapper_must_resolve_qualified_module_name(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "qualified Reference module assignment",
        ):
            self.verify(wrapper=self.wrapper(module_assignment=False))

    def test_wrapper_must_lookup_qualified_module(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "qualified Reference module lookup",
        ):
            self.verify(wrapper=self.wrapper(module_lookup=False))

    def test_wrapper_must_reject_missing_qualified_module(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "missing qualified Reference module",
        ):
            self.verify(wrapper=self.wrapper(module_guard=False))

    def test_wrapper_must_resolve_root_constructor_identity(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "root constructor identity",
        ):
            self.verify(wrapper=self.wrapper(value_assignment=False))

    def test_wrapper_must_reject_qualified_identity_splits(self) -> None:
        with self.assertRaisesRegex(
            policy.AliasPolicyError,
            "qualified constructor identity",
        ):
            self.verify(wrapper=self.wrapper(identity_guard=False))

    def test_wrapper_must_clean_private_guard_state(self) -> None:
        with self.assertRaisesRegex(policy.AliasPolicyError, "does not delete"):
            self.verify(wrapper=self.wrapper(cleanup=False))

    def test_main_is_machine_stable(self) -> None:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = policy.main(
                [
                    "--schema",
                    str(self.schema()),
                    "--wheel-smoke",
                    str(self.wheel()),
                    "--wrapper",
                    str(self.wrapper()),
                ]
            )
        self.assertEqual(status, 0)
        self.assertEqual(stderr.getvalue(), "")
        self.assertIn("16 Reference classes verified", stdout.getvalue())
        self.assertIn("root/qualified import guards", stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
