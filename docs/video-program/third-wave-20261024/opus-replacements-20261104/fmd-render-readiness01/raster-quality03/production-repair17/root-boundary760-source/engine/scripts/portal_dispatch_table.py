#!/usr/bin/env python3
"""The portal's effective dispatch table (fm-5wq.15).

One row per method and property of the pinned Reference's API (the [symbols]
section of API_SCHEMA.tsv, as embedded in the extension): where the portal
resolves it, and every implementation a call passes through.

    symbol  resolved_in  origin  depth  layers

`resolved_in` is the class in the portal class's MRO whose own namespace
holds the member. `origin` is where the outermost implementation was
defined:

- `native`: a builtin of the Rust extension (method or getset descriptor,
  builtin function);
- `bootstrap`: the embedded manimlib bootstrap (the extension's globals);
- `semantics`: the embedded animation semantics;
- `fmn_python.<module>`: an installer module of the portal package;
- `placeholder`: a schema placeholder that refuses when called;
- `absent-class` / `absent`: the class or member does not exist;
- any other module name: code defined elsewhere (stdlib, third party).

`layers` follows the installer wrap pattern to the innermost implementation:
a function whose closure holds a callable named like the symbol or like the
wrapper itself (the one it replaced), or `__wrapped__`. `depth` counts them. Origins come from each
function's globals, never from file paths, so an installed wheel and the
embedded interpreter produce the same table.

Run where the portal's `manimlib` imports:

    python scripts/portal_dispatch_table.py > docs/api/portal_dispatch.tsv
    python scripts/portal_dispatch_table.py --summary
"""
from __future__ import annotations

import collections
import importlib
import sys
import types

SCHEMA = "fmn.portal-dispatch"
VERSION = 1
NATIVE_TYPES = (types.BuiltinFunctionType, types.MethodDescriptorType,
                types.WrapperDescriptorType, types.GetSetDescriptorType,
                types.MemberDescriptorType, types.ClassMethodDescriptorType)
_class_dict = type.__dict__["__dict__"].__get__
_class_mro = type.__dict__["__mro__"].__get__


def reference_members(schema_tsv):
    """(module, Class, member) for every Reference method and property, sorted."""
    section, members = None, []
    for line in schema_tsv.splitlines():
        if line.startswith("["):
            section = line.strip()
            continue
        if section != "[symbols]" or not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 3 and fields[2] in ("method", "property") and "." in fields[1]:
            cls, _, member = fields[1].partition(".")
            members.append((fields[0], cls, member))
    return sorted(set(members))


def callable_of(value):
    if isinstance(value, (staticmethod, classmethod)):
        return value.__func__
    if isinstance(value, property):
        return value.fget
    return value


def origin(function, native_globals):
    if getattr(function, "_fmn_schema_placeholder", False) is True:
        return "placeholder"
    if isinstance(function, NATIVE_TYPES) or not hasattr(function, "__globals__"):
        return "native"
    scope = function.__globals__
    if scope is native_globals:
        return "bootstrap"
    name = scope.get("__name__", "?")
    return "semantics" if name == "manimlib._animation_semantics" else name


def layers(function, name, native_globals):
    """Origins from the outermost implementation to the innermost."""
    chain, seen = [], set()
    while function is not None and id(function) not in seen:
        seen.add(id(function))
        chain.append(origin(function, native_globals))
        inner = getattr(function, "__wrapped__", None)
        # The replaced callable carries the symbol's name or the wrapper's own
        # (an installer's `def requires` wrapping the previous `requires`).
        names = {name, getattr(function, "__name__", None)}
        if inner is None and isinstance(function, types.FunctionType):
            for cell in function.__closure__ or ():
                try:
                    candidate = cell.cell_contents
                except ValueError:
                    continue
                candidate = getattr(candidate, "__func__", candidate)
                if (callable(candidate) and candidate is not function
                        and getattr(candidate, "__name__", None) in names):
                    inner = candidate
                    break
        function = inner
    return chain


def portal_class(namespace, module, class_name):
    """The portal's class: exported by manimlib, else at its Reference module."""
    cls = namespace.get(class_name)
    if isinstance(cls, type):
        return cls
    try:
        cls = getattr(importlib.import_module(module), class_name, None)
    except ImportError:
        return None
    return cls if isinstance(cls, type) else None


def rows(native):
    namespace, native_globals = vars(native), vars(native)
    for module, class_name, member in reference_members(namespace["_API_SCHEMA_TSV"]):
        cls = portal_class(namespace, module, class_name)
        if cls is None:
            yield f"{class_name}.{member}", "-", "absent-class", 0, []
            continue
        owner = next((entry for entry in _class_mro(cls) if member in _class_dict(entry)), None)
        if owner is None:
            yield f"{class_name}.{member}", "-", "absent", 0, []
            continue
        function = callable_of(_class_dict(owner)[member])
        chain = layers(function, member, native_globals)
        yield f"{class_name}.{member}", owner.__name__, chain[0], len(chain), chain


def table(native):
    lines = [f"# {SCHEMA} v{VERSION}: generated by scripts/portal_dispatch_table.py; do not edit",
             "symbol\tresolved_in\torigin\tdepth\tlayers"]
    for symbol, owner, first, depth, chain in rows(native):
        lines.append(f"{symbol}\t{owner}\t{first}\t{depth}\t{' > '.join(chain) or '-'}")
    return "\n".join(lines) + "\n"


def summary(native):
    data = list(rows(native))
    by_origin = collections.Counter(row[2] for row in data)
    installer = lambda o: o.startswith("fmn_python.")  # noqa: E731
    out = [f"Reference methods and properties: {len(data)}",
           "outermost origin: " + ", ".join(
               f"{name} {count}" for name, count in sorted(
                   collections.Counter("fmn_python.*" if installer(o) else o
                                       for o in (row[2] for row in data)).items(),
                   key=lambda item: -item[1])),
           f"installer modules owning an outermost layer: {len({r[2] for r in data if installer(r[2])})}",
           f"symbols wrapping an earlier implementation: {sum(1 for row in data if row[3] > 1)}",
           "deepest:"]
    for symbol, owner, first, depth, chain in sorted(data, key=lambda row: (-row[3], row[0]))[:10]:
        out.append(f"  {symbol} ({owner}) depth {depth}: {' > '.join(chain)}")
    del by_origin
    return "\n".join(out) + "\n"


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    sys.argv = sys.argv[:1]
    import manimlib
    native = getattr(manimlib, "_native", manimlib)
    sys.stdout.write(summary(native) if "--summary" in argv else table(native))
    return 0


if __name__ == "__main__":
    sys.exit(main())
