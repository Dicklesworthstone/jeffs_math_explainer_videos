"""Desk check, not a compile. For each call in the crate's Rust source:
  * crate free functions, `kit.x(...)`, `stage.x(...)`, `stage.arena_mut().x(...)`:
    does the argument count match a definition of that name?
  * does a `?` follow a function that never returns Result/Option?
  * is a Result-returning call discarded as a bare statement (`x(...);`)?
Text only: it cannot see types, trait methods or macros.

Usage (from the crate): python3 -I -B arity_check.py KIT_SRC FMN_CRATES
"""
import glob
import os
import re
import sys


def blank(src):
    """Blank comments and string/char literals, keeping every offset."""
    out = list(src)
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        two = src[i : i + 2]
        if two == "//":
            j = src.find("\n", i)
            j = n if j < 0 else j
            out[i:j] = " " * (j - i)
            i = j
        elif two == "/*":
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            out[i:j] = [" " if ch != "\n" else ch for ch in src[i:j]]
            i = j
        elif c == "r" and re.match(r'r#*"', src[i:i + 8]) and not (src[i - 1].isalnum() or src[i - 1] == "_"):
            hashes = re.match(r'r(#*)"', src[i:i + 8]).group(1)
            end = '"' + hashes
            j = src.find(end, i + 2 + len(hashes))
            j = n if j < 0 else j + len(end)
            out[i:j] = [" " if ch != "\n" else ch for ch in src[i:j]]
            out[i] = '"'
            out[j - 1] = '"'
            i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j = min(n, j + 1)
            out[i:j] = [" " if ch != "\n" else ch for ch in src[i:j]]
            out[i] = '"'
            out[j - 1] = '"'
            i = j
        elif c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", src[i:i + 12])
            if m:
                j = i + m.end()
                out[i:j] = " " * (j - i)
                out[i] = "'"
                out[j - 1] = "'"
                i = j
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def close_of(s, start):
    """Offset of the `)` matching the `(` at `start`."""
    depth = 0
    for i in range(start, len(s)):
        if s[i] in "([{":
            depth += 1
        elif s[i] in ")]}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def top_commas(s):
    """Split on top-level commas, stepping over closure pipes and turbofish."""
    parts, depth, cur, i = [], 0, [], 0
    angle = 0
    while i < len(s):
        c = s[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif c == "<" and depth == 0 and (s[max(0, i - 2) : i] == "::"):
            angle += 1
        elif c == ">" and angle and s[i - 1] != "-":
            angle -= 1
        elif c == "|" and depth == 0:
            prev = "".join(cur).strip()
            if prev == "" or prev.endswith("move"):
                j = s.find("|", i + 1)
                if j > 0:
                    cur.append(s[i : j + 1])
                    i = j + 1
                    continue
        if c == "," and depth == 0 and angle == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(c)
        i += 1
    if "".join(cur).strip():
        parts.append("".join(cur))
    return parts


def param_split(s):
    """Split a parameter list: angle brackets count here, `->` does not."""
    parts, depth, cur = [], 0, []
    for i, c in enumerate(s):
        if c in "([{<":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif c == ">" and s[i - 1] != "-":
            depth -= 1
        if c == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(c)
    if "".join(cur).strip():
        parts.append("".join(cur))
    return parts


FN = re.compile(r"\bfn\s+(\w+)\s*(<[^(]*>)?\s*\(")


def definitions(paths):
    defs = {}
    for path in paths:
        t = blank(open(path, encoding="utf8").read())
        for m in FN.finditer(t):
            open_at = m.end() - 1
            end = close_of(t, open_at)
            if end < 0:
                continue
            params = param_split(t[open_at + 1 : end])
            has_self = bool(params) and re.match(r"\s*(mut\s+)?&?\s*('\w+\s+)?(mut\s+)?self\b", params[0])
            count = len(params) - (1 if has_self else 0)
            rest = t[end + 1 : end + 200]
            ret = re.match(r"\s*->\s*([^{;]+)", rest)
            ret = ret.group(1).strip() if ret else ""
            defs.setdefault(m.group(1), []).append((count, ret, path, bool(has_self)))
    return defs


FALLIBLE = re.compile(r"\b(Result|Option)\b|\bStageResult\b|io::Result")


def main():
    kit_src, fmn_crates = sys.argv[1], sys.argv[2]
    crate_files = sorted(glob.glob("src/**/*.rs", recursive=True))
    crate = definitions(crate_files)
    kit = definitions(sorted(glob.glob(os.path.join(kit_src, "*.rs"))))
    engine = definitions(
        [os.path.join(fmn_crates, "fmn/src/lib.rs")]
        + sorted(glob.glob(os.path.join(fmn_crates, "fmn-mobject/src/**/*.rs"), recursive=True))
        + sorted(glob.glob(os.path.join(fmn_crates, "fmn-library/src/**/*.rs"), recursive=True))
        + sorted(glob.glob(os.path.join(fmn_crates, "fmn-anim/src/**/*.rs"), recursive=True))
    )
    problems = 0
    checked = 0
    call = re.compile(r"(?P<recv>\bkit\.|\bstage\.arena_mut\(\)\.|\bstage\.|(?<![\w.:]))(?P<name>[a-z_][a-z_0-9]*)\s*(?:::<[^(]*>)?\(")
    for path in crate_files:
        if path.endswith(("thumbnail.rs", "score.rs")):
            continue
        t = blank(open(path, encoding="utf8").read())
        closures = set(re.findall(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:move\s+)?\|", t))
        for m in call.finditer(t):
            recv, name = m.group("recv"), m.group("name")
            if not recv and name in closures:
                continue
            before = t[max(0, m.start() - 4) : m.start()]
            if before.endswith("fn ") or name in ("if", "for", "while", "match", "return", "move", "in", "as", "let", "mut", "fn", "else", "loop"):
                continue
            if name == "clone":
                continue
            if recv == "kit.":
                table, label = kit, "kit"
            elif recv.startswith("stage."):
                table, label = engine, "stage"
            else:
                table, label = crate, "crate"
                if name not in table or all(d[3] for d in table[name]):
                    continue
            if name not in table:
                if label != "crate":
                    print(f"NO DEFINITION {path}:{t[:m.start()].count(chr(10)) + 1} {recv}{name}")
                    problems += 1
                continue
            open_at = m.end() - 1
            end = close_of(t, open_at)
            if end < 0:
                continue
            args = top_commas(t[open_at + 1 : end])
            line = t[: m.start()].count("\n") + 1
            options = table[name]
            checked += 1
            if not any(o[0] == len(args) for o in options):
                wants = sorted({o[0] for o in options})
                print(f"ARITY {path}:{line} {recv}{name}: {len(args)} args, definitions take {wants}")
                problems += 1
            following = t[end + 1 : end + 3].lstrip()[:1]
            never = all(not FALLIBLE.search(o[1]) for o in options)
            always = all(FALLIBLE.search(o[1]) for o in options)
            if following == "?" and never:
                rets = sorted({o[1] or "()" for o in options})
                print(f"? ON INFALLIBLE {path}:{line} {recv}{name} returns {rets}")
                problems += 1
            if following == ";" and always:
                head = t[t.rfind("\n", 0, m.start()) + 1 : m.start()].strip()
                if head == "":
                    print(f"RESULT DROPPED {path}:{line} {recv}{name}")
                    problems += 1
    print(f"calls checked: {checked}; problems: {problems}")

    # Second pass: every other call, matched by name alone, so a hit only means
    # no definition anywhere with that name takes this many arguments.
    std = set("""map filter fold iter into_iter collect enumerate zip rev take skip chain flat_map sum min max abs
        len push extend get get_mut contains windows chunks last first split join to_string to_owned parse unwrap
        unwrap_or unwrap_or_else ok ok_or ok_or_else map_err and_then is_some is_none is_some_and clone powi powf
        sqrt sin cos tan atan2 hypot exp ln round floor ceil rem_euclid clamp replace strip_prefix ends_with
        starts_with chars trim nth find position any all count cloned copied pop insert remove sort sort_by
        partial_cmp cmp expect then then_some borrow borrow_mut set as_ref as_str concat retain dedup step_by
        to_degrees to_radians signum atan asin acos sinh cosh mul_add is_finite is_nan recip neg add sub mul div
        lines get_or_init""".split())
    everything = {}
    for table in (crate, kit, engine):
        for k, v2 in table.items():
            everything.setdefault(k, []).extend(v2)
    method = re.compile(r"\.(?P<name>[a-z_][a-z_0-9]*)\s*(?:::<[^(]*>)?\(")
    assoc = re.compile(r"\b(?P<ty>[A-Z]\w*)::(?P<name>[a-z_][a-z_0-9]*)\s*(?:::<[^(]*>)?\(")
    free = re.compile(r"(?<![\w.:])(?P<name>[a-z_][a-z_0-9]*)\s*(?:::<[^(]*>)?\(")
    keywords = {"if", "for", "while", "match", "return", "move", "in", "as", "let", "mut", "fn", "else", "loop", "pub"}
    second, hits = 0, 0
    for path in crate_files:
        if path.endswith(("thumbnail.rs", "score.rs")):
            continue
        t = blank(open(path, encoding="utf8").read())
        closures = set(re.findall(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:move\s+)?\|", t))
        params = set(re.findall(r"\b(\w+)\s*:\s*(?:&\s*)?(?:dyn|impl)\s+Fn", t))
        for pattern, kind in ((method, "method"), (assoc, "assoc"), (free, "free")):
            for m in pattern.finditer(t):
                name = m.group("name")
                if name in keywords or name in std:
                    continue
                if t[max(0, m.start() - 3) : m.start()] == "fn ":
                    continue
                if kind == "free" and (name in closures or name in params or name in crate):
                    continue
                if kind == "method" and t[max(0, m.start() - 3) : m.start()] in ("kit", "age"):
                    continue
                if kind == "assoc" and m.group("ty") in ("Self", "Some", "Ok", "Err", "None", "Vec", "String", "Box", "Option", "Result"):
                    continue
                options = everything.get(name)
                if not options:
                    continue
                open_at = m.end() - 1
                end = close_of(t, open_at)
                if end < 0:
                    continue
                args = top_commas(t[open_at + 1 : end])
                second += 1
                # a method called as Type::method(recv, ...) carries self as an argument
                counts = {o[0] for o in options} | ({o[0] + 1 for o in options if o[3]} if kind == "assoc" else set())
                if len(args) not in counts:
                    line = t[: m.start()].count("\n") + 1
                    print(f"BY NAME {path}:{line} {kind} {name}: {len(args)} args, definitions take {sorted(counts)}")
                    hits += 1
    print(f"second pass, by name only: {second} calls, {hits} hits")

    # Third pass: names called that no crate, kit or engine source defines at all.
    wide = set(everything)
    for path in glob.glob(os.path.join(fmn_crates, "**/*.rs"), recursive=True):
        if "/fmn-python/" in path or "/fmn-wasm/" in path:
            continue
        wide.update(x[0] for x in FN.findall(open(path, encoding="utf8", errors="replace").read()))
    unknown = {}
    for path in crate_files:
        if path.endswith(("thumbnail.rs", "score.rs")):
            continue
        t = blank(open(path, encoding="utf8").read())
        closures = set(re.findall(r"let\s+(?:mut\s+)?(\w+)\s*=\s*(?:move\s+)?\|", t))
        params = set(re.findall(r"\b(\w+)\s*:\s*(?:&\s*)?(?:dyn|impl)\s+Fn", t))
        for pattern in (method, assoc, free):
            for m in pattern.finditer(t):
                name = m.group("name")
                if name in keywords or name in std or name in wide or name in closures or name in params:
                    continue
                line = t[: m.start()].count("\n") + 1
                unknown.setdefault(name, []).append(f"{path}:{line}")
    for name, where in sorted(unknown.items()):
        print(f"UNKNOWN {name} x{len(where)} e.g. {where[0]}")


main()
