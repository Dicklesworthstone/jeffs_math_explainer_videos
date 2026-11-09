"""Post-assembly provenance for schema-generated ``manimlib`` fallbacks."""

from __future__ import annotations

import inspect
import sys
from dataclasses import dataclass
from types import ModuleType
from typing import Any

SCHEMA_PROVENANCE_VERSION = 1
MAX_SCHEMA_BYTES = 8 * 1024 * 1024
MAX_SCHEMA_ROWS = 100_000
_PLACEHOLDER_MARKER = "_fmn_schema_placeholder"
_PLACEHOLDER_KIND = "_fmn_schema_placeholder_kind"
_PLACEHOLDER_SYMBOL = "_fmn_schema_placeholder_symbol"
_PROVENANCE_VERSION = "_fmn_schema_provenance_version"
_PROVENANCE_COUNTS = "_fmn_schema_provenance_counts"
_MISSING = object()


class SchemaProvenanceError(RuntimeError):
    pass


@dataclass(frozen=True)
class _SchemaRow:
    module_name: str
    qualified: str
    kind: str

    @property
    def symbol(self) -> str:
        return f"{self.module_name}:{self.qualified}"


def _schema_rows(text: str) -> tuple[_SchemaRow, ...]:
    try:
        payload = text.encode("utf-8")
    except UnicodeEncodeError as exc:
        raise SchemaProvenanceError(f"API schema is not valid UTF-8 text: {exc}") from exc
    if len(payload) > MAX_SCHEMA_BYTES:
        raise SchemaProvenanceError(f"API schema exceeds {MAX_SCHEMA_BYTES}-byte limit")
    section: str | None = None
    rows: list[_SchemaRow] = []
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if section != "symbols":
            continue
        columns = raw_line.split("\t")
        if len(columns) != 6:
            raise SchemaProvenanceError(
                f"[symbols] line {line_number} must contain exactly 6 tab-separated columns"
            )
        module_name, qualified, kind = (column.strip() for column in columns[:3])
        if not module_name or not qualified or not kind:
            raise SchemaProvenanceError(
                f"[symbols] line {line_number} contains an empty identity field"
            )
        rows.append(_SchemaRow(module_name, qualified, kind))
        if len(rows) > MAX_SCHEMA_ROWS:
            raise SchemaProvenanceError(f"API schema row count exceeds {MAX_SCHEMA_ROWS}")
    if not rows:
        raise SchemaProvenanceError("API schema contains no [symbols] rows")
    return tuple(rows)


def _resolve_static(module: ModuleType, qualified: str) -> Any | None:
    value: Any = module
    for component in qualified.split("."):
        value = inspect.getattr_static(value, component, _MISSING)
        if value is _MISSING:
            return None
    return value


def _unwrap_callable(value: Any) -> Any:
    if isinstance(value, (classmethod, staticmethod)):
        return value.__func__
    return value


def _code_qualname(value: Any) -> str | None:
    code = getattr(_unwrap_callable(value), "__code__", None)
    qualname = getattr(code, "co_qualname", None)
    return qualname if isinstance(qualname, str) else None


def _direct_namespace(value: Any) -> dict[str, Any] | Any | None:
    try:
        return vars(value)
    except (TypeError, AttributeError):
        return None


def _mark_placeholder(value: Any, *, kind: str, symbol: str) -> bool:
    value = _unwrap_callable(value)
    namespace = _direct_namespace(value)
    if namespace is None:
        return False
    if namespace.get(_PLACEHOLDER_MARKER) is not True:
        setattr(value, _PLACEHOLDER_MARKER, True)
    if not isinstance(namespace.get(_PLACEHOLDER_KIND), str):
        setattr(value, _PLACEHOLDER_KIND, kind)
    if not isinstance(namespace.get(_PLACEHOLDER_SYMBOL), str):
        setattr(value, _PLACEHOLDER_SYMBOL, symbol)
    return True


def _is_generated_class(value: Any) -> bool:
    if not isinstance(value, type):
        return False
    direct = _direct_namespace(value)
    if direct is None:
        return False
    return _code_qualname(direct.get("__init__")) in {
        "_surface_init",
        "_schema_init_refusal.<locals>.refused",
    }


def _module_for_row(native_module: ModuleType, module_name: str) -> ModuleType | None:
    if module_name == "manimlib":
        return native_module
    module = sys.modules.get(module_name)
    return module if isinstance(module, ModuleType) else None


def apply_schema_placeholder_provenance(native_module: ModuleType) -> dict[str, int]:
    schema = getattr(native_module, "_API_SCHEMA_TSV", None)
    if not isinstance(schema, str):
        raise SchemaProvenanceError("native portal does not expose its embedded API schema")
    rows = _schema_rows(schema)
    recognized: dict[str, set[int]] = {
        "classes": set(),
        "constructors": set(),
        "functions": set(),
        "methods": set(),
    }
    assembled_modules: dict[str, ModuleType] = {}
    for row in rows:
        module = _module_for_row(native_module, row.module_name)
        if module is None:
            continue
        assembled_modules[row.module_name] = module
        value = _resolve_static(module, row.qualified)
        if value is None:
            continue
        code_qualname = _code_qualname(value)
        if row.kind == "class" and _is_generated_class(value):
            if _mark_placeholder(value, kind="class", symbol=row.symbol):
                recognized["classes"].add(id(value))
            direct = _direct_namespace(value)
            constructor = None if direct is None else direct.get("__init__")
            constructor_origin = _code_qualname(constructor)
            if constructor_origin == "_surface_init":
                if _mark_placeholder(
                    constructor,
                    kind="constructor",
                    symbol="schema-generated:_surface_init",
                ):
                    recognized["constructors"].add(id(_unwrap_callable(constructor)))
            elif constructor_origin == "_schema_init_refusal.<locals>.refused":
                if _mark_placeholder(
                    constructor,
                    kind="constructor-refusal",
                    symbol=f"{row.symbol}.__init__",
                ):
                    recognized["constructors"].add(id(_unwrap_callable(constructor)))
        elif row.kind in {"function", "leaked_import"}:
            if code_qualname == "_placeholder_function.<locals>.unavailable":
                if _mark_placeholder(value, kind="function", symbol=row.symbol):
                    recognized["functions"].add(id(_unwrap_callable(value)))
        elif row.kind == "method":
            if code_qualname == "_placeholder_method.<locals>.unavailable":
                if _mark_placeholder(value, kind="method", symbol=row.symbol):
                    recognized["methods"].add(id(_unwrap_callable(value)))
            elif code_qualname == "_schema_init_refusal.<locals>.refused":
                if _mark_placeholder(
                    value,
                    kind="constructor-refusal",
                    symbol=row.symbol,
                ):
                    recognized["constructors"].add(id(_unwrap_callable(value)))
    root_package = sys.modules.get("manimlib")
    if isinstance(root_package, ModuleType):
        assembled_modules.setdefault("manimlib-package", root_package)
    counts = {
        "schema_rows": len(rows),
        "modules": len(assembled_modules),
        **{name: len(values) for name, values in recognized.items()},
    }
    frozen_counts = tuple(sorted(counts.items()))
    for module in {native_module, *assembled_modules.values()}:
        setattr(module, _PROVENANCE_VERSION, SCHEMA_PROVENANCE_VERSION)
        setattr(module, _PROVENANCE_COUNTS, frozen_counts)
    return counts
