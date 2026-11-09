#!/usr/bin/env python3
"""The one API schema, bootstrap layer (fm-vn6, plan §16.2 / §16.1).

Extracts the pinned Reference's public API surface — the wildcard-export
namespace, every class and its bases, every method and function signature
with parameter kinds and default *expressions*, module-level constants, the
argparse flag surface, and the config-key surface — into ``API_SCHEMA.tsv``
at the repo root.

That file is the EXTRACTED layer only: mechanical, regenerable, and never
hand-edited. The AUTHORED layer — canonical names for C-9's public-surface
typos, the semantic-status tiering of §16.1, and the Rust bindings the
generators need — lives beside it in ``API_OVERLAY.tsv`` and is maintained by
hand. The effective schema is the two merged, which is what
``fmn_conformance::schema`` reads and what every generator consumes.

Why static ``ast`` rather than ``import manimlib``: the Reference cannot be
imported here (it wants ``colour``, ``moderngl``, ``pyglet``, a GL context),
and importing it would make the schema a function of whatever happened to be
installed. Static extraction is deterministic, hermetic, and — for the
wildcard surface specifically — exactly reproduces CPython's rules.

Wildcard-namespace modelling (§1.6: ``manimlib`` has NO ``__all__``, so this
is not a formality). For a module without ``__all__``, ``from M import *``
binds every module-level name not starting with an underscore, INCLUDING
names M merely imported. That is why ``from manimlib import *`` really does
leak ``np``, ``math``, and friends into the user's namespace, and the Ledger
has to enumerate symbols rather than trust a curated list. Faithfully
modelled here:

  * a module's namespace = names it defines + names it imports, minus
    underscore-prefixed ones;
  * ``from X import *`` recurses into X's namespace (cycles broken by a
    visited set, matching the import machinery's own memoisation);
  * ``if TYPE_CHECKING:`` bodies are SKIPPED — they never execute at
    runtime, so ``Iterable``/``Self``/``Vect3`` are not exported even though
    they are imported at the top of nearly every Reference module;
  * ``try:``/``except ImportError:`` takes the ``try`` branch, as a working
    installation does.

Defaults are recorded as SOURCE EXPRESSIONS (``TAU / 4``, ``ORIGIN``), never
as evaluated values: many reference module-level constants and arrays, and
resolving them would require the import we just refused. The expression text
is what the parity ledger and the Python front door both need anyway.

Outputs (committed):

  API_SCHEMA.tsv    the extracted schema; section-tagged, tab-separated,
                    std-parseable — no TOML/YAML/JSON dependency anywhere in
                    the reader, per the SUITE.lock doctrine.

Deterministic: sorted iteration everywhere, no timestamps, no host paths.

Usage:  python3 scripts/gen_api_schema.py   (paths are relative to this file)
"""

from __future__ import annotations

import ast
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
REF = REPO / "scripts" / "manim_ref"
PKG = REF / "manimlib"
OUT = REPO / "API_SCHEMA.tsv"

SCHEMA_VERSION = 1

# The config surface is ours (the Reference's key shapes plus the native
# determinism/render sections); the Reference's own document is the parity
# check, not the source.
OUR_CONFIG = REPO / "crates" / "fmn-config" / "src" / "default_config.yml"
REF_CONFIG = PKG / "default_config.yml"


# ---------------------------------------------------------------------------
# TSV field hygiene
# ---------------------------------------------------------------------------


def cell(text: str | None) -> str:
    """Normalise one TSV cell: no tabs, no newlines, no runs of blanks.

    Default expressions and docstring-free signatures can legally span lines
    (`Vect3 = ORIGIN,` inside a wrapped call); collapsing whitespace keeps the
    row format intact and loses nothing a consumer needs.
    """
    if text is None or text == "":
        return "-"
    return " ".join(str(text).split())


# ---------------------------------------------------------------------------
# Module parsing
# ---------------------------------------------------------------------------


def module_name(path: Path) -> str:
    """`manimlib/mobject/geometry.py` -> `manimlib.mobject.geometry`."""
    rel = path.relative_to(REF).with_suffix("")
    parts = list(rel.parts)
    if parts[-1] == "__init__":
        parts.pop()
    return ".".join(parts)


def is_type_checking_guard(test: ast.expr) -> bool:
    """`if TYPE_CHECKING:` / `if typing.TYPE_CHECKING:`."""
    if isinstance(test, ast.Name):
        return test.id == "TYPE_CHECKING"
    if isinstance(test, ast.Attribute):
        return test.attr == "TYPE_CHECKING"
    return False


def runtime_body(body: list[ast.stmt]) -> list[ast.stmt]:
    """Flatten a module body to the statements that actually execute.

    Descends into `try:` (the success branch a working install takes) and into
    `if:` bodies other than `if TYPE_CHECKING:`, whose body is dropped whole —
    those imports exist only for the type checker and are not in the runtime
    namespace.
    """
    out: list[ast.stmt] = []
    for stmt in body:
        if isinstance(stmt, ast.If):
            if is_type_checking_guard(stmt.test):
                out.extend(runtime_body(stmt.orelse))
            else:
                out.append(stmt)
                out.extend(runtime_body(stmt.body))
                out.extend(runtime_body(stmt.orelse))
        elif isinstance(stmt, ast.Try):
            out.append(stmt)
            out.extend(runtime_body(stmt.body))
        else:
            out.append(stmt)
    return out


class Module:
    """One parsed Reference module and the names it puts in scope."""

    def __init__(self, name: str, path: Path) -> None:
        self.name = name
        self.path = path
        self.tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        self.body = runtime_body(self.tree.body)
        # name -> "defined" | source module it was imported from
        self.bindings: dict[str, str] = {}
        self.star_imports: list[str] = []
        self.classes: dict[str, ast.ClassDef] = {}
        self.functions: dict[str, ast.FunctionDef | ast.AsyncFunctionDef] = {}
        self.constants: dict[str, str] = {}
        self._collect()

    def _resolve(self, node: ast.ImportFrom) -> str:
        """Absolute module name for a (possibly relative) `from ... import`."""
        if not node.level:
            return node.module or ""
        parts = self.name.split(".")
        # A package's __init__ is its own root; a module drops its leaf.
        if self.path.name != "__init__.py":
            parts = parts[:-1]
        base = parts[: len(parts) - (node.level - 1)] if node.level > 1 else parts
        return ".".join([*base, node.module]) if node.module else ".".join(base)

    def _collect(self) -> None:
        for stmt in self.body:
            if isinstance(stmt, ast.ClassDef):
                self.classes[stmt.name] = stmt
                self.bindings[stmt.name] = "defined"
            elif isinstance(stmt, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.functions[stmt.name] = stmt
                self.bindings[stmt.name] = "defined"
            elif isinstance(stmt, ast.Assign):
                for target in stmt.targets:
                    if isinstance(target, ast.Name):
                        self.bindings[target.id] = "defined"
                        self.constants[target.id] = ast.unparse(stmt.value)
            elif isinstance(stmt, ast.AnnAssign):
                if isinstance(stmt.target, ast.Name):
                    self.bindings[stmt.target.id] = "defined"
                    if stmt.value is not None:
                        self.constants[stmt.target.id] = ast.unparse(stmt.value)
            elif isinstance(stmt, ast.Import):
                for alias in stmt.names:
                    bound = alias.asname or alias.name.split(".")[0]
                    self.bindings[bound] = alias.name
            elif isinstance(stmt, ast.ImportFrom):
                source = self._resolve(stmt)
                for alias in stmt.names:
                    if alias.name == "*":
                        self.star_imports.append(source)
                    else:
                        # The binding refers to the imported OBJECT, so the
                        # origin is the full object path (module plus the
                        # imported name), not the source module alone.
                        bound = alias.asname or alias.name
                        self.bindings[bound] = f"{source}.{alias.name}"


def load_modules() -> dict[str, Module]:
    mods: dict[str, Module] = {}
    for path in sorted(PKG.rglob("*.py")):
        name = module_name(path)
        mods[name] = Module(name, path)
    return mods


def public_namespace(mods: dict[str, Module], root: str) -> dict[str, str]:
    """The names `from <root> import *` binds, mapped to their DEFINING module.

    Two passes, because reachability and authorship are different questions.
    The walk answers "is this name in the surface?"; the home resolution
    answers "whose is it?". A name reachable through several modules — the
    common case, since Reference modules re-import each other freely — is
    attributed to the module that actually defines it, never to a module that
    merely imported it on the way. Only names no reached module defines are
    genuine leaks from outside the package (`np`, `math`, `deepcopy`).

    Where two modules define the same name, the lexicographically first wins,
    so the attribution is deterministic rather than walk-order dependent.
    """
    seen: set[str] = set()
    reached: list[str] = []

    def walk(name: str) -> None:
        if name in seen or name not in mods:
            return
        seen.add(name)
        reached.append(name)
        for star in mods[name].star_imports:
            walk(star)

    walk(root)

    out: dict[str, str] = {}
    for module in sorted(reached):
        for bound, binding in mods[module].bindings.items():
            if bound.startswith("_"):
                continue
            defines = binding == "defined"
            if bound not in out or (defines and mods[out[bound]].bindings.get(bound) != "defined"):
                out[bound] = module
    return out


# ---------------------------------------------------------------------------
# Signatures
# ---------------------------------------------------------------------------

PARAM_ROWS: list[tuple[str, ...]] = []


def parameter_rows(
    owner: str,
    fn: ast.FunctionDef | ast.AsyncFunctionDef,
) -> list[tuple[str, ...]]:
    """Return one `[params]` row per parameter, in declaration order.

    Parameter KIND is recorded because the Python front door has to reproduce
    it exactly: a Reference parameter that is positional-or-keyword must not
    become keyword-only in fmn-python, or source-unedited scenes break.
    """
    rows: list[tuple[str, ...]] = []
    args = fn.args
    ordinal = 0

    def emit(arg: ast.arg, kind: str, default: ast.expr | None) -> None:
        nonlocal ordinal
        rows.append(
            (
                owner,
                str(ordinal),
                arg.arg,
                kind,
                cell(ast.unparse(arg.annotation) if arg.annotation else None),
                cell(ast.unparse(default) if default is not None else None),
            )
        )
        ordinal += 1

    # positional-only, then positional-or-keyword: defaults right-align across
    # the two lists taken together.
    positional = list(args.posonlyargs) + list(args.args)
    pad = len(positional) - len(args.defaults)
    for i, arg in enumerate(positional):
        kind = "positional_only" if i < len(args.posonlyargs) else "positional_or_keyword"
        emit(arg, kind, args.defaults[i - pad] if i >= pad else None)

    if args.vararg is not None:
        emit(args.vararg, "var_positional", None)
    for arg, default in zip(args.kwonlyargs, args.kw_defaults):
        emit(arg, "keyword_only", default)
    if args.kwarg is not None:
        emit(args.kwarg, "var_keyword", None)
    return rows


def record_params(owner: str, fn: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
    PARAM_ROWS.extend(parameter_rows(owner, fn))


def is_property(fn: ast.FunctionDef | ast.AsyncFunctionDef) -> bool:
    for dec in fn.decorator_list:
        target = dec.func if isinstance(dec, ast.Call) else dec
        name = target.attr if isinstance(target, ast.Attribute) else getattr(target, "id", "")
        if name in ("property", "cached_property", "getter", "setter", "deleter"):
            return True
    return False


def public_class_members(cls: ast.ClassDef) -> dict[str, ast.stmt]:
    """Return the class namespace entries that survive body execution.

    A Python class body is an ordinary namespace: a later method or assignment
    replaces an earlier binding with the same name, and ``del`` removes it.
    The extracted schema must describe that final namespace rather than every
    transient statement encountered by the AST walk.
    """
    members: dict[str, ast.stmt] = {}
    for item in cls.body:
        if isinstance(item, ast.Assign):
            for target in item.targets:
                if isinstance(target, ast.Name) and (
                    not target.id.startswith("_") or target.id == "__init__"
                ):
                    members[target.id] = item
        elif isinstance(item, ast.AnnAssign):
            target = item.target
            if isinstance(target, ast.Name) and (
                not target.id.startswith("_") or target.id == "__init__"
            ):
                members[target.id] = item
        elif isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if not item.name.startswith("_") or item.name == "__init__":
                members[item.name] = item
        elif isinstance(item, ast.Delete):
            for target in item.targets:
                if isinstance(target, ast.Name):
                    members.pop(target.id, None)
    return members


def class_schema_rows(
    module_name: str,
    class_name: str,
    cls: ast.ClassDef,
) -> tuple[list[tuple[str, ...]], list[tuple[str, ...]]]:
    """Return final class symbol and parameter rows under Python semantics."""
    symbols: list[tuple[str, ...]] = []
    params: list[tuple[str, ...]] = []
    for member_name, item in sorted(public_class_members(cls).items()):
        qualified = f"{class_name}.{member_name}"
        if isinstance(item, (ast.Assign, ast.AnnAssign)):
            default = ast.unparse(item.value) if item.value is not None else None
            symbols.append(
                (module_name, qualified, "attribute", "defined", "0", cell(default))
            )
            continue
        assert isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef))
        kind = "property" if is_property(item) else "method"
        symbols.append((module_name, qualified, kind, "defined", "0", "-"))
        params.extend(parameter_rows(f"{module_name}:{qualified}", item))
    return symbols, params


# ---------------------------------------------------------------------------
# The argparse flag surface
# ---------------------------------------------------------------------------


def literal_or_source(node: ast.expr | None) -> str | None:
    """A string literal's VALUE where the node is one, else its source text.

    argparse help strings in the Reference are frequently written as adjacent
    or `+`-joined fragments across several lines. `ast.unparse` faithfully
    reproduces `'a ' + 'b'`, which is the wrong thing to put in a documentation
    table — the reader wants the sentence, not the concatenation that built it.
    Literal-only expressions are folded; anything referring to a name is left
    as source, since its value is not knowable without the import.
    """
    if node is None:
        return None
    folded = fold_str(node)
    return folded if folded is not None else ast.unparse(node)


def fold_str(node: ast.expr) -> str | None:
    """Fold a string-only expression to its value, or `None` if it is not one.

    `ast.literal_eval` deliberately refuses `+` on strings, so concatenation —
    which is how the Reference wraps long help text — has to be folded here.
    Implicit adjacency (`'a' 'b'`) is already one Constant by parse time.
    """
    if isinstance(node, ast.Constant):
        return node.value if isinstance(node.value, str) else None
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
        left, right = fold_str(node.left), fold_str(node.right)
        return None if left is None or right is None else left + right
    return None


def extract_flags(mods: dict[str, Module]) -> list[tuple[str, ...]]:
    """Every `parser.add_argument(...)` in the Reference's `config.py`.

    The CLI table is normative for W9 (fm-c53), which keeps the Reference's
    flag surface "where it still means something" — so the surface has to be
    written down before it can be kept or deliberately dropped.
    """
    mod = mods.get("manimlib.config")
    if mod is None:
        return []
    rows: list[tuple[str, ...]] = []
    for node in ast.walk(mod.tree):
        if not isinstance(node, ast.Call):
            continue
        fn = node.func
        if not (isinstance(fn, ast.Attribute) and fn.attr == "add_argument"):
            continue
        options = [a.value for a in node.args if isinstance(a, ast.Constant) and isinstance(a.value, str)]
        kw = {k.arg: k.value for k in node.keywords if k.arg}
        rows.append(
            (
                cell(",".join(options)),
                cell(ast.unparse(kw["dest"]) if "dest" in kw else None),
                cell(literal_or_source(kw.get("action")) or "store"),
                cell(ast.unparse(kw["nargs"]) if "nargs" in kw else None),
                cell(ast.unparse(kw["default"]) if "default" in kw else None),
                cell(ast.unparse(kw["type"]) if "type" in kw else None),
                cell(literal_or_source(kw.get("help"))),
            )
        )
    return sorted(rows)


# ---------------------------------------------------------------------------
# The config-key surface
# ---------------------------------------------------------------------------


def flatten_yaml(path: Path) -> list[tuple[str, str, str]]:
    """(dotted path, value kind, default literal) for one config document.

    A deliberately small indentation-driven reader for the same YAML subset
    `fmn_config::yaml` accepts — this script must not grow a PyYAML dependency
    to read a document the engine parses with std alone.
    """
    rows: list[tuple[str, str, str]] = []
    stack: list[tuple[int, str]] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        line = raw.strip()
        if ":" not in line:
            continue
        key, _, rest = line.partition(":")
        key = key.strip()
        value = rest.split("#", 1)[0].strip() if "#" in rest else rest.strip()
        while stack and stack[-1][0] >= indent:
            stack.pop()
        dotted = ".".join([*(k for _, k in stack), key])
        if value == "":
            # A parent. Emitted as a `map` row rather than skipped, because
            # some parents ARE the config key: `colors`, `key_bindings`, and
            # `directories.subdirs` are open, user-extensible maps bound as a
            # whole, and a schema that only knew their current children could
            # not express that.
            rows.append((dotted, "map", "-"))
            stack.append((indent, key))
            continue
        rows.append((dotted, yaml_kind(value), cell(value)))
    return rows


def yaml_kind(value: str) -> str:
    if value.startswith("(") and value.endswith(")"):
        return "tuple"
    if value in ("True", "False", "true", "false"):
        return "bool"
    unquoted = value.strip("\"'")
    if unquoted != value:
        return "string"
    try:
        int(value)
        return "int"
    except ValueError:
        pass
    try:
        float(value)
        return "float"
    except ValueError:
        pass
    return "string"


# ---------------------------------------------------------------------------
# Emission
# ---------------------------------------------------------------------------


def reference_commit() -> str:
    out = subprocess.run(
        ["git", "-C", str(REF), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    )
    return out.stdout.strip()


def main() -> int:
    if not PKG.is_dir():
        print(
            f"error: the pinned Reference checkout is missing at {REF}\n"
            "       clone 3b1b/manim there at the SUITE.lock [reference] pin",
            file=sys.stderr,
        )
        return 1

    commit = reference_commit()
    mods = load_modules()
    exported = public_namespace(mods, "manimlib")

    # Being in the wildcard surface is a property of the NAME, not of the path
    # the star-walk happened to take. `CameraFrame` is exported even though
    # `manimlib/__init__.py` never stars its defining module — it arrives
    # bound because some reached module imported it plainly. Marking it
    # unexported because of the route would be false, and emitting it as a
    # "leak" would be doubly false: the package defines it.
    def is_exported(symbol: str) -> str:
        return "1" if symbol in exported else "0"

    symbol_rows: list[tuple[str, ...]] = []
    for name in sorted(mods):
        mod = mods[name]
        for cls_name in sorted(mod.classes):
            cls = mod.classes[cls_name]
            bases = ",".join(ast.unparse(b) for b in cls.bases)
            symbol_rows.append(
                (
                    name,
                    cls_name,
                    "class",
                    "defined",
                    is_exported(cls_name),
                    cell(bases),
                )
            )
            # Class-level attributes are public surface, not decoration:
            # the Reference configures whole lineages through them
            # (`Checkmark.tex`, `Cross.stroke_color`), and C-9's
            # `tickness_multiplier` lives here rather than in any signature.
            # Apply Python's last-binding-wins class namespace semantics so
            # transient duplicate definitions never become duplicate rows.
            class_symbols, class_params = class_schema_rows(name, cls_name, cls)
            symbol_rows.extend(class_symbols)
            PARAM_ROWS.extend(class_params)

        for fn_name in sorted(mod.functions):
            fn = mod.functions[fn_name]
            symbol_rows.append(
                (
                    name,
                    fn_name,
                    "function",
                    "defined",
                    is_exported(fn_name),
                    "-",
                )
            )
            record_params(f"{name}:{fn_name}", fn)

        for const in sorted(mod.constants):
            if const in mod.classes or const in mod.functions:
                continue
            symbol_rows.append(
                (
                    name,
                    const,
                    "constant",
                    "defined",
                    is_exported(const),
                    cell(mod.constants[const]),
                )
            )

    # Names the wildcard surface binds that this walk did not define anywhere:
    # modules and third-party symbols leaked through `import *`. They are part
    # of the surface whether or not anyone wanted them to be (§1.6).
    defined_names = {s for _, s, *_ in symbol_rows}
    for symbol in sorted(exported):
        if symbol in defined_names:
            continue
        origin = exported[symbol]
        symbol_rows.append(
            (origin, symbol, "leaked_import", mods[origin].bindings[symbol], "1", "-")
        )

    lines: list[str] = [
        "# fmn API schema — the EXTRACTED layer (fm-vn6, plan §16.2)",
        f"# generated from 3b1b/manim @ {commit}",
        "# by scripts/gen_api_schema.py — regenerate, never hand-edit",
        "#",
        "# The authored layer (canonical names for C-9 typos, semantic status,",
        "# Rust bindings) lives in API_OVERLAY.tsv. The effective schema is the",
        "# two merged; fmn_conformance::schema is the reader, and every",
        "# generator consumes the merged form.",
        "#",
        "# Section-tagged, tab-separated, std-parseable — no TOML/YAML/JSON",
        "# dependency anywhere in the reader (the SUITE.lock doctrine).",
        "",
        "[meta]",
        f"schema_version\t{SCHEMA_VERSION}",
        f"reference_commit\t{commit}",
        "generator\tscripts/gen_api_schema.py",
        f"wildcard_exports\t{len(exported)}",
        "",
        "[symbols]",
        "# module\tname\tkind\torigin\texported\tdetail",
    ]
    lines.extend("\t".join(row) for row in sorted(symbol_rows))

    lines += ["", "[params]", "# owner\tordinal\tname\tkind\tannotation\tdefault"]
    lines.extend("\t".join(row) for row in sorted(PARAM_ROWS, key=lambda r: (r[0], int(r[1]))))

    lines += ["", "[flags]", "# options\tdest\taction\tnargs\tdefault\ttype\thelp"]
    lines.extend("\t".join(row) for row in extract_flags(mods))

    lines += ["", "[config]", "# path\tkind\tdefault\treference"]
    ref_keys = {p for p, _, _ in flatten_yaml(REF_CONFIG)} if REF_CONFIG.exists() else set()
    for path, kind, default in flatten_yaml(OUR_CONFIG):
        lines.append("\t".join((path, kind, default, "1" if path in ref_keys else "0")))

    lines.append("")
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(
        f"wrote {OUT.relative_to(REPO)}: "
        f"{len(symbol_rows)} symbols, {len(PARAM_ROWS)} params, "
        f"{len(extract_flags(mods))} flags, {len(exported)} wildcard exports"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
