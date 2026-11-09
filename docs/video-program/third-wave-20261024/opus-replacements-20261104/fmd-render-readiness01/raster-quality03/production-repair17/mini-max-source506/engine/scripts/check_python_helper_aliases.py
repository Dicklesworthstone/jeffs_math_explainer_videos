#!/usr/bin/env python3
"""Verify the Python/Rust geometry-constructor alias boundary.

The native Rust API intentionally exposes ergonomic snake_case constructor
helpers. The pinned manimlib Reference exports CamelCase classes instead. This
gate keeps those two front doors distinct: every Reference class must remain in
the extracted schema, no Rust-only helper may leak into Python wildcard
exports, the package import guard must enforce root and qualified-module
identity, and the installed-wheel smoke must construct every corresponding
Reference class through those same identities.
"""

from __future__ import annotations

import argparse
import ast
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

MAX_SCHEMA_BYTES = 16 * 1024 * 1024
MAX_SOURCE_BYTES = 2 * 1024 * 1024

REFERENCE_CONSTRUCTOR_ALIASES: dict[str, tuple[str, str]] = {
    "group": ("manimlib.mobject.mobject", "Group"),
    "v_group": ("manimlib.mobject.types.vectorized_mobject", "VGroup"),
    "vectorized_point": (
        "manimlib.mobject.types.vectorized_mobject",
        "VectorizedPoint",
    ),
    "small_dot": ("manimlib.mobject.geometry", "SmallDot"),
    "sector": ("manimlib.mobject.geometry", "Sector"),
    "vector": ("manimlib.mobject.geometry", "Vector"),
    "polyline": ("manimlib.mobject.geometry", "Polyline"),
    "triangle": ("manimlib.mobject.geometry", "Triangle"),
    "rounded_rectangle": ("manimlib.mobject.geometry", "RoundedRectangle"),
    "svg_mobject": ("manimlib.mobject.svg.svg_mobject", "SVGMobject"),
    "tangent_line": ("manimlib.mobject.geometry", "TangentLine"),
    "curved_arrow": ("manimlib.mobject.geometry", "CurvedArrow"),
    "curved_double_arrow": ("manimlib.mobject.geometry", "CurvedDoubleArrow"),
    "curves_as_submobjects": (
        "manimlib.mobject.types.vectorized_mobject",
        "CurvesAsSubmobjects",
    ),
    "dashed_vmobject": (
        "manimlib.mobject.types.vectorized_mobject",
        "DashedVMobject",
    ),
    "v_highlight": ("manimlib.mobject.types.vectorized_mobject", "VHighlight"),
}

WRAPPER_MAPPING_NAME = "_REFERENCE_CLASS_BY_RUST_HELPER"
WRAPPER_CONTRACT_CLASS_MAPPING_NAME = "_CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER"
WRAPPER_CONTRACT_MODULE_MAPPING_NAME = "_CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER"
WRAPPER_HELPER_NAME = "_rust_helper"
WRAPPER_CLASS_NAME = "_reference_class"
WRAPPER_NATIVE_NAME = "_native"
WRAPPER_SYS_NAME = "_sys"
WRAPPER_REFERENCE_MODULE_NAME = "_reference_module"
WRAPPER_MODULE_OBJECT_NAME = "_module"
WRAPPER_REFERENCE_VALUE_NAME = "_reference_value"
WHEEL_MAPPING_NAME = "REFERENCE_CONSTRUCTOR_ALIASES"
WHEEL_VERIFY_FUNCTION = "verify_reference_constructor_aliases"
WHEEL_BUILDERS_NAME = "builders"
WHEEL_ALIAS_NAME = "alias"
WHEEL_MODULE_NAME = "module_name"
WHEEL_CLASS_NAME = "class_name"
WHEEL_ROOT_NAME = "manimlib"
WHEEL_SYS_NAME = "sys"
WHEEL_CONSTRUCTOR_NAME = "constructor"
WHEEL_REQUIRE_NAME = "require"


class AliasPolicyError(ValueError):
    pass


@dataclass(frozen=True)
class Symbol:
    module: str
    name: str
    kind: str
    exported: bool


class _ExecutableNodeCollector(ast.NodeVisitor):
    """Visit one function body without accepting evidence from nested code."""

    def __init__(self) -> None:
        self.assignments: list[ast.Assign | ast.AnnAssign] = []
        self.calls: list[ast.Call] = []
        self.loops: list[ast.For] = []

    def visit_Assign(self, node: ast.Assign) -> None:
        self.assignments.append(node)
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        self.assignments.append(node)
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        self.calls.append(node)
        self.generic_visit(node)

    def visit_For(self, node: ast.For) -> None:
        self.loops.append(node)
        self.generic_visit(node)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        del node

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        del node

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        del node

    def visit_Lambda(self, node: ast.Lambda) -> None:
        del node


def _collect_executable(statements: list[ast.stmt]) -> _ExecutableNodeCollector:
    collector = _ExecutableNodeCollector()
    for statement in statements:
        collector.visit(statement)
    return collector


def read_bounded(path: Path, limit: int) -> str:
    try:
        payload = path.read_bytes()
    except OSError as exc:
        raise AliasPolicyError(f"cannot read {path}: {exc}") from exc
    if len(payload) > limit:
        raise AliasPolicyError(f"{path} exceeds the {limit}-byte limit")
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise AliasPolicyError(f"{path} is not UTF-8: {exc}") from exc


def parse_python(path: Path) -> ast.Module:
    text = read_bounded(path, MAX_SOURCE_BYTES)
    try:
        return ast.parse(text, filename=str(path))
    except SyntaxError as exc:
        raise AliasPolicyError(f"{path}: invalid Python: {exc}") from exc


def parse_schema(path: Path) -> list[Symbol]:
    text = read_bounded(path, MAX_SCHEMA_BYTES)
    if text and not text.endswith("\n"):
        raise AliasPolicyError(f"{path} is missing its final LF")
    in_symbols = False
    symbols: list[Symbol] = []
    seen: set[tuple[str, str]] = set()
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if line == "[symbols]":
            in_symbols = True
            continue
        if in_symbols and line.startswith("["):
            break
        if not in_symbols or not line or line.startswith("#"):
            continue
        fields = raw_line.split("\t")
        if len(fields) != 6:
            raise AliasPolicyError(
                f"{path}:{line_number}: expected six symbol fields, got {len(fields)}"
            )
        module, name, kind, _origin, exported, _detail = fields
        if exported not in {"0", "1"}:
            raise AliasPolicyError(
                f"{path}:{line_number}: exported must be 0 or 1, got {exported!r}"
            )
        key = (module, name)
        if key in seen:
            raise AliasPolicyError(f"{path}:{line_number}: duplicate symbol {module}.{name}")
        seen.add(key)
        symbols.append(Symbol(module, name, kind, exported == "1"))
    if not in_symbols:
        raise AliasPolicyError(f"{path} has no [symbols] section")
    return symbols


def assigned_literal(tree: ast.Module, path: Path, name: str) -> Any:
    matches: list[ast.expr] = []
    for node in tree.body:
        if not isinstance(node, (ast.Assign, ast.AnnAssign)):
            continue
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        if any(isinstance(target, ast.Name) and target.id == name for target in targets):
            if node.value is not None:
                matches.append(node.value)
    if len(matches) != 1:
        raise AliasPolicyError(f"{path}: expected one literal {name}, found {len(matches)}")
    try:
        return ast.literal_eval(matches[0])
    except (ValueError, TypeError, SyntaxError) as exc:
        raise AliasPolicyError(f"{path}: {name} is not a literal value") from exc


def tuple_mapping(tree: ast.Module, path: Path, name: str) -> dict[str, tuple[str, str]]:
    raw = assigned_literal(tree, path, name)
    if not isinstance(raw, dict):
        raise AliasPolicyError(f"{path}: {name} must be a dict")
    normalized: dict[str, tuple[str, str]] = {}
    for key, item in raw.items():
        if (
            not isinstance(key, str)
            or not isinstance(item, tuple)
            or len(item) != 2
            or not all(isinstance(part, str) for part in item)
        ):
            raise AliasPolicyError(f"{path}: malformed {name} entry {key!r}: {item!r}")
        normalized[key] = item
    return normalized


def string_mapping(tree: ast.Module, path: Path, name: str) -> dict[str, str]:
    raw = assigned_literal(tree, path, name)
    if not isinstance(raw, dict):
        raise AliasPolicyError(f"{path}: {name} must be a dict")
    normalized: dict[str, str] = {}
    for key, item in raw.items():
        if not isinstance(key, str) or not isinstance(item, str):
            raise AliasPolicyError(f"{path}: malformed {name} entry {key!r}: {item!r}")
        normalized[key] = item
    return normalized


def mapping_drift(
    label: str,
    expected: dict[str, Any],
    observed: dict[str, Any],
) -> AliasPolicyError:
    missing = sorted(expected.keys() - observed.keys())
    extra = sorted(observed.keys() - expected.keys())
    changed = sorted(
        key for key in expected.keys() & observed.keys() if expected[key] != observed[key]
    )
    return AliasPolicyError(
        f"{label} mapping drift: missing={missing}, extra={extra}, changed={changed}"
    )


def function_node(
    tree: ast.Module,
    path: Path,
    function_name: str,
) -> ast.FunctionDef | ast.AsyncFunctionDef:
    functions = [
        node
        for node in tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == function_name
    ]
    if len(functions) != 1:
        raise AliasPolicyError(
            f"{path}: expected exactly one {function_name} function, found {len(functions)}"
        )
    return functions[0]


def function_names(tree: ast.Module) -> set[str]:
    return {
        node.name
        for node in tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    }


def function_calls(tree: ast.Module, path: Path, function_name: str) -> set[str]:
    function = function_node(tree, path, function_name)
    return {
        node.func.id
        for node in _collect_executable(function.body).calls
        if isinstance(node.func, ast.Name)
    }


def exact_expression(node: ast.AST, source: str) -> bool:
    expected = ast.parse(source, mode="eval").body
    return ast.dump(node, include_attributes=False) == ast.dump(
        expected, include_attributes=False
    )


def named_items_call(node: ast.expr, mapping_name: str) -> bool:
    return (
        isinstance(node, ast.Call)
        and not node.args
        and not node.keywords
        and isinstance(node.func, ast.Attribute)
        and node.func.attr == "items"
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == mapping_name
    )


def exact_name_tuple(node: ast.expr, names: tuple[str, ...]) -> bool:
    return (
        isinstance(node, (ast.Tuple, ast.List))
        and len(node.elts) == len(names)
        and all(
            isinstance(item, ast.Name) and item.id == expected
            for item, expected in zip(node.elts, names, strict=True)
        )
    )


def is_hasattr_test(
    node: ast.expr,
    object_name: str,
    attribute_name: str,
    *,
    negated: bool,
) -> bool:
    candidate = node
    if negated:
        if not isinstance(node, ast.UnaryOp) or not isinstance(node.op, ast.Not):
            return False
        candidate = node.operand
    if not isinstance(candidate, ast.Call):
        return False
    return (
        isinstance(candidate.func, ast.Name)
        and candidate.func.id == "hasattr"
        and len(candidate.args) == 2
        and not candidate.keywords
        and isinstance(candidate.args[0], ast.Name)
        and candidate.args[0].id == object_name
        and isinstance(candidate.args[1], ast.Name)
        and candidate.args[1].id == attribute_name
    )


def raises_import_error(node: ast.If) -> bool:
    return any(
        isinstance(child, ast.Raise)
        and isinstance(child.exc, ast.Call)
        and isinstance(child.exc.func, ast.Name)
        and child.exc.func.id == "ImportError"
        for child in node.body
    )


def assignment_matches(node: ast.AST, target_name: str, expression: str) -> bool:
    if isinstance(node, ast.Assign):
        if len(node.targets) != 1:
            return False
        target = node.targets[0]
        value = node.value
    elif isinstance(node, ast.AnnAssign):
        target = node.target
        value = node.value
    else:
        return False
    return (
        value is not None
        and isinstance(target, ast.Name)
        and target.id == target_name
        and exact_expression(value, expression)
    )


def _one_direct_index(
    body: list[ast.stmt],
    predicate: Any,
    *,
    path: Path,
    description: str,
) -> int:
    matches = [index for index, node in enumerate(body) if predicate(node)]
    if len(matches) != 1:
        raise AliasPolicyError(
            f"{path}: expected one {description}, found {len(matches)}"
        )
    return matches[0]


def verify_wrapper_guard(tree: ast.Module, path: Path) -> None:
    class_contracts = [
        node
        for node in tree.body
        if isinstance(node, ast.If)
        and exact_expression(
            node.test,
            f"{WRAPPER_MAPPING_NAME} != {WRAPPER_CONTRACT_CLASS_MAPPING_NAME}",
        )
        and raises_import_error(node)
    ]
    if len(class_contracts) != 1:
        raise AliasPolicyError(
            f"{path}: wrapper must compare its literal class map with the authority manifest"
        )
    module_contracts = [
        node
        for node in tree.body
        if isinstance(node, ast.If)
        and exact_expression(
            node.test,
            f"set({WRAPPER_MAPPING_NAME}) != "
            f"set({WRAPPER_CONTRACT_MODULE_MAPPING_NAME})",
        )
        and raises_import_error(node)
    ]
    if len(module_contracts) != 1:
        raise AliasPolicyError(
            f"{path}: wrapper must compare helper keys with the authority module map"
        )

    loops = [
        node
        for node in tree.body
        if isinstance(node, ast.For)
        and exact_name_tuple(
            node.target,
            (WRAPPER_HELPER_NAME, WRAPPER_CLASS_NAME),
        )
        and named_items_call(node.iter, WRAPPER_MAPPING_NAME)
    ]
    if len(loops) != 1:
        raise AliasPolicyError(
            f"{path}: expected one import-guard loop over {WRAPPER_MAPPING_NAME}, "
            f"found {len(loops)}"
        )
    loop = loops[0]
    helper_index = _one_direct_index(
        loop.body,
        lambda node: (
            isinstance(node, ast.If)
            and is_hasattr_test(
                node.test,
                WRAPPER_NATIVE_NAME,
                WRAPPER_HELPER_NAME,
                negated=False,
            )
            and raises_import_error(node)
        ),
        path=path,
        description="Rust-only helper ImportError guard",
    )
    class_index = _one_direct_index(
        loop.body,
        lambda node: (
            isinstance(node, ast.If)
            and is_hasattr_test(
                node.test,
                WRAPPER_NATIVE_NAME,
                WRAPPER_CLASS_NAME,
                negated=True,
            )
            and raises_import_error(node)
        ),
        path=path,
        description="missing Reference class ImportError guard",
    )
    reference_module_index = _one_direct_index(
        loop.body,
        lambda node: assignment_matches(
            node,
            WRAPPER_REFERENCE_MODULE_NAME,
            f"{WRAPPER_CONTRACT_MODULE_MAPPING_NAME}[{WRAPPER_HELPER_NAME}]",
        ),
        path=path,
        description="qualified Reference module assignment",
    )
    module_lookup_index = _one_direct_index(
        loop.body,
        lambda node: assignment_matches(
            node,
            WRAPPER_MODULE_OBJECT_NAME,
            f"{WRAPPER_SYS_NAME}.modules.get({WRAPPER_REFERENCE_MODULE_NAME})",
        ),
        path=path,
        description="qualified Reference module lookup",
    )
    module_guard_index = _one_direct_index(
        loop.body,
        lambda node: (
            isinstance(node, ast.If)
            and exact_expression(
                node.test,
                f"{WRAPPER_MODULE_OBJECT_NAME} is None",
            )
            and raises_import_error(node)
        ),
        path=path,
        description="missing qualified Reference module ImportError guard",
    )
    reference_value_index = _one_direct_index(
        loop.body,
        lambda node: assignment_matches(
            node,
            WRAPPER_REFERENCE_VALUE_NAME,
            f"getattr({WRAPPER_NATIVE_NAME}, {WRAPPER_CLASS_NAME})",
        ),
        path=path,
        description="root constructor identity assignment",
    )
    identity_guard_index = _one_direct_index(
        loop.body,
        lambda node: (
            isinstance(node, ast.If)
            and exact_expression(
                node.test,
                f"vars({WRAPPER_MODULE_OBJECT_NAME}).get({WRAPPER_CLASS_NAME}) "
                f"is not {WRAPPER_REFERENCE_VALUE_NAME}",
            )
            and raises_import_error(node)
        ),
        path=path,
        description="qualified constructor identity ImportError guard",
    )
    ordered = [
        helper_index,
        class_index,
        reference_module_index,
        module_lookup_index,
        module_guard_index,
        reference_value_index,
        identity_guard_index,
    ]
    if ordered != sorted(ordered):
        raise AliasPolicyError(
            f"{path}: constructor import guards are not ordered before qualified use"
        )

    cleanup_names = {
        target.id
        for node in tree.body
        if isinstance(node, ast.Delete)
        for target in node.targets
        if isinstance(target, ast.Name)
    }
    required_cleanup = {
        WRAPPER_MAPPING_NAME,
        WRAPPER_CONTRACT_CLASS_MAPPING_NAME,
        WRAPPER_CONTRACT_MODULE_MAPPING_NAME,
        WRAPPER_HELPER_NAME,
        WRAPPER_CLASS_NAME,
        WRAPPER_SYS_NAME,
        WRAPPER_REFERENCE_MODULE_NAME,
        WRAPPER_MODULE_OBJECT_NAME,
        WRAPPER_REFERENCE_VALUE_NAME,
    }
    missing_cleanup = sorted(required_cleanup - cleanup_names)
    if missing_cleanup:
        raise AliasPolicyError(
            f"{path}: import guard does not delete private names {missing_cleanup}"
        )


def local_lambda_dict_keys(
    function: ast.FunctionDef | ast.AsyncFunctionDef,
    path: Path,
    name: str,
) -> set[str]:
    matches: list[ast.Dict] = []
    for node in _collect_executable(function.body).assignments:
        targets = node.targets if isinstance(node, ast.Assign) else [node.target]
        if not any(isinstance(target, ast.Name) and target.id == name for target in targets):
            continue
        if not isinstance(node.value, ast.Dict):
            raise AliasPolicyError(f"{path}: {name} must be a literal dict")
        matches.append(node.value)
    if len(matches) != 1:
        raise AliasPolicyError(f"{path}: expected one local {name} table, found {len(matches)}")

    keys: list[str] = []
    for key_node, value_node in zip(matches[0].keys, matches[0].values, strict=True):
        if not isinstance(key_node, ast.Constant) or not isinstance(key_node.value, str):
            raise AliasPolicyError(f"{path}: every {name} key must be a string literal")
        if not isinstance(value_node, ast.Lambda):
            raise AliasPolicyError(
                f"{path}: {name}[{key_node.value!r}] must be an explicit constructor lambda"
            )
        keys.append(key_node.value)
    if len(set(keys)) != len(keys):
        raise AliasPolicyError(f"{path}: {name} contains duplicate class keys")
    return set(keys)


def wheel_loop_target(node: ast.expr) -> bool:
    return (
        isinstance(node, (ast.Tuple, ast.List))
        and len(node.elts) == 2
        and isinstance(node.elts[0], ast.Name)
        and node.elts[0].id == WHEEL_ALIAS_NAME
        and exact_name_tuple(
            node.elts[1],
            (WHEEL_MODULE_NAME, WHEEL_CLASS_NAME),
        )
    )


def call_named_pair(
    node: ast.AST,
    function_name: str,
    first: str,
    second: str,
) -> bool:
    return (
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == function_name
        and len(node.args) == 2
        and not node.keywords
        and isinstance(node.args[0], ast.Name)
        and node.args[0].id == first
        and isinstance(node.args[1], ast.Name)
        and node.args[1].id == second
    )


def builder_invocation(node: ast.AST) -> bool:
    return (
        isinstance(node, ast.Call)
        and not node.args
        and not node.keywords
        and isinstance(node.func, ast.Subscript)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == WHEEL_BUILDERS_NAME
        and isinstance(node.func.slice, ast.Name)
        and node.func.slice.id == WHEEL_CLASS_NAME
    )


def require_condition(node: ast.AST, expression: str) -> bool:
    return (
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == WHEEL_REQUIRE_NAME
        and len(node.args) >= 1
        and exact_expression(node.args[0], expression)
    )


def verify_wheel_probe(tree: ast.Module, path: Path) -> None:
    function = function_node(tree, path, WHEEL_VERIFY_FUNCTION)
    expected_classes = {
        class_name for _module, class_name in REFERENCE_CONSTRUCTOR_ALIASES.values()
    }
    builders = local_lambda_dict_keys(function, path, WHEEL_BUILDERS_NAME)
    if builders != expected_classes:
        missing = sorted(expected_classes - builders)
        extra = sorted(builders - expected_classes)
        raise AliasPolicyError(
            f"wheel constructor table drift: missing={missing}, extra={extra}"
        )

    function_nodes = _collect_executable(function.body)
    loops = [
        node
        for node in function_nodes.loops
        if wheel_loop_target(node.target)
        and named_items_call(node.iter, WHEEL_MAPPING_NAME)
    ]
    if len(loops) != 1:
        raise AliasPolicyError(
            f"{path}: expected one runtime loop over {WHEEL_MAPPING_NAME}, found {len(loops)}"
        )
    nodes = _collect_executable(loops[0].body).calls
    if not any(
        call_named_pair(node, "getattr", WHEEL_ROOT_NAME, WHEEL_CLASS_NAME)
        for node in nodes
    ):
        raise AliasPolicyError(
            f"{path}: wheel probe does not resolve each Reference class from manimlib"
        )
    if not any(
        require_condition(
            node,
            f"{WHEEL_MODULE_NAME} in {WHEEL_SYS_NAME}.modules",
        )
        for node in nodes
    ):
        raise AliasPolicyError(
            f"{path}: wheel probe does not require each qualified module to exist"
        )
    if not any(
        require_condition(
            node,
            f"getattr({WHEEL_SYS_NAME}.modules[{WHEEL_MODULE_NAME}], "
            f"{WHEEL_CLASS_NAME}, None) is {WHEEL_CONSTRUCTOR_NAME}",
        )
        for node in nodes
    ):
        raise AliasPolicyError(
            f"{path}: wheel probe does not require qualified constructor identity"
        )
    if not any(builder_invocation(node) for node in nodes):
        raise AliasPolicyError(
            f"{path}: wheel probe does not invoke builders[class_name]()"
        )
    if not any(
        call_named_pair(node, "hasattr", WHEEL_ROOT_NAME, WHEEL_ALIAS_NAME)
        for node in nodes
    ):
        raise AliasPolicyError(
            f"{path}: wheel probe does not recheck each Rust-only alias at runtime"
        )


def parse_wheel_smoke(path: Path) -> tuple[dict[str, tuple[str, str]], ast.Module]:
    tree = parse_python(path)
    return tuple_mapping(tree, path, WHEEL_MAPPING_NAME), tree


def parse_wrapper(path: Path) -> tuple[dict[str, str], ast.Module]:
    tree = parse_python(path)
    return string_mapping(tree, path, WRAPPER_MAPPING_NAME), tree


def verify(schema_path: Path, wheel_smoke_path: Path, wrapper_path: Path) -> None:
    symbols = parse_schema(schema_path)
    exported_by_name: dict[str, list[Symbol]] = {}
    by_identity = {(symbol.module, symbol.name): symbol for symbol in symbols}
    for symbol in symbols:
        if symbol.exported:
            exported_by_name.setdefault(symbol.name, []).append(symbol)

    for alias, (module, class_name) in REFERENCE_CONSTRUCTOR_ALIASES.items():
        leaked = exported_by_name.get(alias, [])
        if leaked:
            locations = ", ".join(f"{row.module}.{row.name}" for row in leaked)
            raise AliasPolicyError(
                f"Rust-only helper {alias!r} leaked into Python exports at {locations}"
            )
        row = by_identity.get((module, class_name))
        if row is None:
            raise AliasPolicyError(
                f"Reference class {module}.{class_name} is missing from the schema"
            )
        if row.kind != "class" or not row.exported:
            raise AliasPolicyError(
                f"Reference class {module}.{class_name} is not an exported class"
            )

    wheel_mapping, wheel_tree = parse_wheel_smoke(wheel_smoke_path)
    if wheel_mapping != REFERENCE_CONSTRUCTOR_ALIASES:
        raise mapping_drift("wheel", REFERENCE_CONSTRUCTOR_ALIASES, wheel_mapping)
    if WHEEL_VERIFY_FUNCTION not in function_names(wheel_tree):
        raise AliasPolicyError(f"{wheel_smoke_path}: missing {WHEEL_VERIFY_FUNCTION}")
    calls = function_calls(
        wheel_tree,
        wheel_smoke_path,
        "verify_installed_distribution",
    )
    if WHEEL_VERIFY_FUNCTION not in calls:
        raise AliasPolicyError(
            f"{wheel_smoke_path}: verify_installed_distribution does not call "
            f"{WHEEL_VERIFY_FUNCTION}"
        )
    verify_wheel_probe(wheel_tree, wheel_smoke_path)

    wrapper_mapping, wrapper_tree = parse_wrapper(wrapper_path)
    expected_wrapper = {
        alias: class_name
        for alias, (_module, class_name) in REFERENCE_CONSTRUCTOR_ALIASES.items()
    }
    if wrapper_mapping != expected_wrapper:
        raise mapping_drift("wrapper", expected_wrapper, wrapper_mapping)
    verify_wrapper_guard(wrapper_tree, wrapper_path)


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--schema", type=Path, default=Path("API_SCHEMA.tsv"))
    parser.add_argument(
        "--wheel-smoke",
        type=Path,
        default=Path("crates/fmn-python/tests/wheel_smoke.py"),
    )
    parser.add_argument(
        "--wrapper",
        type=Path,
        default=Path("crates/fmn-python/python/manimlib/__init__.py"),
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        verify(args.schema, args.wheel_smoke, args.wrapper)
    except AliasPolicyError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print(
        "Python geometry alias policy: "
        f"{len(REFERENCE_CONSTRUCTOR_ALIASES)} Reference classes verified; "
        "schema, root/qualified import guards, and constructed-wheel probe agree; "
        "no Rust-only helpers exported"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
