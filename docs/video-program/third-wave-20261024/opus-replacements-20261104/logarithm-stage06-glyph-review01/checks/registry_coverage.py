"""Does the glyph registry cover what the film's source can draw? Reads only.

    python3 -I -B .../logarithm-stage06-glyph-review01/checks/registry_coverage.py

From the repo root. The probe records a string only when a scene built under scratch timing, at 1 fps, reaches a kit text
call. So a registry can be complete in characters and still miss a string behind a branch, in a helper no chapter calls, or
outside the chapter list. This lexes the compiled chapter files itself, with no shared parser, and puts every string literal
outside test code in one class:

  exact      the literal is a registry source
  template   a format! template that matches a registry source
  fragment   the literal sits inside a registry source, such as a t2c key
  not text   an argument to something that cannot draw: asserts, cue ids, sentence marks
  UNMATCHED  none of those. Read each one

Own controls: a planted unreachable literal must land in UNMATCHED, and the lexer must keep a raw string with a quote in it
and ignore a lifetime and a comment. It also checks the sources against the job's frozen inputs, so the text read here is
the text the probe's binary was built from, and the seven literals in RubyRaven's glyph-delta.tsv row by row.
"""
import collections
import hashlib
import json
import pathlib
import re
import sys

REPO = pathlib.Path.cwd()
CRATE = pathlib.Path("explainer_corpus/logarithm_mysteries")
BASE = pathlib.Path("narration_lab/third-wave-20261024/root")
RUN = BASE / "five-film-release-20261027/logarithm-stage06-glyph01"
INPUTS = BASE / "elevenlabs-logarithm-rebuild-20261104/combined-integration02/glyph-inputs01.json"
DELTA = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-preflight01/glyph-delta.tsv")
failed = False


def check(ok, text):
    global failed
    failed = failed or not ok
    print(f"{'PASS' if ok else 'FAIL'}  {text}")


def unescape(body):
    out, i = [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = body[i + 1]
        if n == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3 : end].replace("_", ""), 16)))
            i = end + 1
        elif n == "x":
            out.append(chr(int(body[i + 2 : i + 4], 16)))
            i += 4
        elif n == "\n":
            i += 2
            while i < len(body) and body[i].isspace():
                i += 1
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


def lex(text):
    """(kind, value, line) for strings, identifiers and single punctuation. Comments, lifetimes and chars drop out."""
    toks, i, line, n = [], 0, 1, len(text)
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
        elif c.isspace():
            i += 1
        elif text.startswith("//", i):
            i = text.find("\n", i)
            i = n if i < 0 else i
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            line += text.count("\n", i, j)
            i = j
        elif (m := re.compile(r'b?r(#*)"').match(text, i)):
            close = '"' + m.group(1)
            j = text.index(close, m.end())
            toks.append(("str", text[m.end() : j], line))
            line += text.count("\n", i, j)
            i = j + len(close)
        elif c == '"' or (c == "b" and text.startswith('b"', i)):
            start = i + (2 if c == "b" else 1)
            j = start
            while text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            toks.append(("str", unescape(text[start:j]), line))
            line += text.count("\n", i, j)
            i = j + 1
        elif c == "'":
            if text[i + 1] == "\\":
                i = text.index("'", i + 3) + 1
            elif i + 2 < n and text[i + 2] == "'":
                i += 3
            else:
                i += 1
        elif c.isalpha() or c == "_":
            m = re.compile(r"\w+").match(text, i)
            toks.append(("id", m.group(), line))
            i = m.end()
        elif c.isdigit():
            m = re.compile(r"[\w.]+").match(text, i)
            toks.append(("num", m.group(), line))
            i = m.end()
        else:
            toks.append(("p", c, line))
            i += 1
    return toks


def close_of(toks, k):
    pairs = {"{": "}", "(": ")", "[": "]"}
    want, depth = pairs[toks[k][1]], 0
    for j in range(k, len(toks)):
        if toks[j][1] == toks[k][1] and toks[j][0] == "p":
            depth += 1
        elif toks[j][1] == want and toks[j][0] == "p":
            depth -= 1
            if depth == 0:
                return j
    raise ValueError("unbalanced")


def drop_tests(toks):
    out, i = [], 0
    while i < len(toks):
        if [t[1] for t in toks[i : i + 7]] == ["#", "[", "cfg", "(", "test", ")", "]"]:
            j = i + 7
            while toks[j][1] not in ("{", ";"):
                j += 1
            i = close_of(toks, j) + 1 if toks[j][1] == "{" else j + 1
            continue
        out.append(toks[i])
        i += 1
    return out


def callee(toks, k):
    """The call or const a string token sits directly inside: (name, is_macro)."""
    depth = 0
    j = k - 1
    while j >= 0:
        v = toks[j][1] if toks[j][0] == "p" else None
        if v in (")", "]", "}"):
            depth += 1
        elif v in ("(", "[", "{"):
            if depth == 0:
                if v == "(" and j >= 1:
                    if toks[j - 1][0] == "id":
                        return toks[j - 1][1], False
                    if toks[j - 1][1] == "!" and j >= 2:
                        return toks[j - 2][1], True
                if v == "{" or (v == "[" and j >= 1 and toks[j - 1][1] != "&"):
                    pass
            else:
                depth -= 1
        elif v == ";" and depth == 0:
            break
        j -= 1
    head = [t[1] for t in toks[max(0, k - 8) : k] if t[0] == "id"]
    if "const" in head or "static" in head:
        return "const", False
    return "?", False


def decode(literal):
    return unescape(literal[1:-1])


def registry():
    rows = []
    for line in (RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines():
        kind, chapters, literal = line.split("\t", 2)
        rows.append((kind, chapters, decode(literal)))
    return rows


def template_regex(template):
    parts = re.split(r"(\{\{|\}\}|\{[^{}]*\})", template)
    out = []
    for part in parts:
        if part == "{{":
            out.append(re.escape("{"))
        elif part == "}}":
            out.append(re.escape("}"))
        elif part.startswith("{") and part.endswith("}"):
            out.append(".*?")
        else:
            out.append(re.escape(part))
    return re.compile("^" + "".join(out) + "$", re.S)


NOT_TEXT_CALLS = {
    "expect", "assert", "assert_eq", "assert_ne", "debug_assert", "panic", "unreachable", "println", "eprintln",
    "write", "writeln", "until_sentence", "until_phrase", "until_word", "say", "line", "wait_line", "wait_until", "sentence",
    "name", "todo", "cue", "phrase", "mark", "listen", "ok_or", "ok_or_else", "include_str",
    "fact", "fact_f64", "fact_list", "fact_matrix", "fail", "phrase_time", "allow",
}
# Error text built with format!, passed to fail(), so it cannot reach a page: read in the UNMATCHED pass, then listed here.
FACT_KEY = re.compile(r"[a-z0-9_]+\.[a-z0-9_]+")
SCENE_NAME = re.compile(r"\d\d_[a-z_]+")
# A const holding the exact words of a spoken line is a phrase hook for timing. It is checked against the script, not assumed.
SPOKEN = (CRATE / "script.tsv").read_text(encoding="utf-8")


def classify(value, name, sources, joined):
    if value in sources:
        return "exact"
    if "{" in value and name == "format":
        rx = template_regex(value)
        if any(rx.match(s) for s in sources):
            return "template"
        # a helper may add a prefix, as precisely() adds "precisely: "
        inner = re.compile(rx.pattern[1:-1], re.S)
        if any(inner.search(s) for s in sources):
            return "template"
        return "UNMATCHED"
    if value.strip() and value in joined:
        return "fragment"
    if not value.strip():
        return "not text"
    if name == "const" and value in SPOKEN:
        return "not text"
    if name in NOT_TEXT_CALLS or re.fullmatch(r"\d\d_\d+(-part\d+)?", value) or FACT_KEY.fullmatch(value) or SCENE_NAME.fullmatch(value):
        return "not text"
    return "UNMATCHED"


def compiled_files():
    mod = (CRATE / "src/chapters/mod.rs").read_text()
    names = re.findall(r"^(?:pub )?mod (\w+);", mod, re.M)
    return [CRATE / "src/chapters" / f"{n}.rs" for n in names], names


def main():
    print("# 1. lexer controls")
    toks = lex('fn a<\'b>(x: &\'b str) { // "no"\n let c = \'"\'; f(r#"q"r"#, "t\\"u", "\\u{2212}"); /* "no" */ }')
    strs = [t[1] for t in toks if t[0] == "str"]
    check(strs == ['q"r', 't"u', "−"], f"lexer keeps only the real strings: {strs}")
    t = lex('#[cfg(test)]\nmod tests { fn x() { "gone"; } }\nfn y() { kit.text(stage, "kept"); }')
    t = drop_tests(t)
    check([x[1] for x in t if x[0] == "str"] == ["kept"], "test modules drop out")

    print("\n# 2. the source read here is the source the job froze")
    inputs = json.loads(INPUTS.read_text())["files"]
    by_rel = {pathlib.Path(f["path"]).relative_to(REPO).as_posix(): f["sha256"] for f in inputs if pathlib.Path(f["path"]).is_relative_to(REPO)}
    check(len(inputs) == 226, f"{len(inputs)} frozen inputs; job receipt inputs_sha256 is on file")
    files, names = compiled_files()
    print(f"  compiled chapter modules: {len(files)}: {' '.join(names)}")
    for path in files + [CRATE / "script.tsv", CRATE / "src/main.rs", pathlib.Path("crates/explainer_kit/src/kit.rs"), pathlib.Path("crates/explainer_kit/src/glyphs.rs"), pathlib.Path("target/debug/logarithm_mysteries")]:
        want = by_rel.get(path.as_posix())
        seen = hashlib.sha256(path.read_bytes()).hexdigest()
        check(want == seen, f"{path.as_posix()} {seen[:16]} {'matches the frozen input' if want == seen else f'frozen {want}'}")
    all_rs = sorted((CRATE / "src/chapters").glob("*.rs"))
    extra = [p.name for p in all_rs if p not in files and p.name != "mod.rs"]
    print(f"  chapter files not compiled, ignored: {extra}")

    rows = registry()
    sources = {r[2] for r in rows}
    joined = "\n".join(sources)
    print(f"\n# 3. registry: {len(rows)} rows, {collections.Counter(r[0] for r in rows)}")

    print("\n# 4. every literal outside test code")
    counts = collections.Counter()
    unmatched = []
    by_chapter = collections.Counter()
    cross = []
    chapter_of = {
        "ratio_ruler": "00_ratio_ruler", "building_ruler": "01_building_ruler", "slide_rule": "02_slide_rule", "folded_decade": "03_folded_decade",
        "undoing_growth": "04_undoing_growth", "complex_turns": "05_complex_turns", "lifting_loops": "06_lifting_loops", "branch_cut": "07_branch_cut",
        "product_rule": "08_product_rule", "matrix_logs": "09_matrix_logs", "doing_nothing": "10_doing_nothing", "unreachable": "11_unreachable",
        "what_it_forgets": "12_what_it_forgets",
    }
    text_row = {r[2]: r for r in rows}
    seen_values = collections.defaultdict(set)
    for path in files:
        toks = drop_tests(lex(path.read_text(encoding="utf-8")))
        for k, tok in enumerate(toks):
            if tok[0] != "str":
                continue
            name, is_macro = callee(toks, k)
            cls = classify(tok[1], name, sources, joined)
            counts[(path.stem, cls)] += 1
            counts[("all", cls)] += 1
            if cls == "UNMATCHED":
                unmatched.append((path.name, tok[2], name, tok[1]))
            if cls == "exact":
                seen_values[tok[1]].add(path.stem)
                want = chapter_of.get(path.stem)
                if want and want not in text_row[tok[1]][1].split(","):
                    cross.append((path.name, tok[2], tok[1], text_row[tok[1]][1]))
    classes = ["exact", "template", "fragment", "not text", "UNMATCHED"]
    print(f"  {'file':24s}" + "".join(f"{c:>11s}" for c in classes))
    for stem in [p.stem for p in files] + ["all"]:
        print(f"  {stem:24s}" + "".join(f"{counts[(stem, c)]:11d}" for c in classes))
    print(f"\n  UNMATCHED, each to read in place: {len(unmatched)}")
    for name, line, call, value in unmatched:
        print(f"    {name}:{line} in {call}: {value!r}")
    print(f"\n  exact in a chapter's own file but not attributed to that chapter by the probe: {len(cross)}")
    for name, line, value, chapters in cross:
        print(f"    {name}:{line} {value!r}; registry says {chapters}")

    print("\n# 5. control: a literal the probe never reaches must come out UNMATCHED")
    planted = lex('fn never() { if false { kit.text(stage, "a string no scene reaches", 20.0, c, at)?; } }')
    k = [i for i, t in enumerate(planted) if t[0] == "str"][0]
    cls = classify(planted[k][1], callee(planted, k)[0], sources, joined)
    check(cls == "UNMATCHED", f"planted literal classed {cls}")
    gone = "\\text{singular } A"
    check(classify(gone, "tex", sources, joined) == "UNMATCHED", "a retired label, singular A, is not on this registry")

    print("\n# 6. RubyRaven's glyph-delta.tsv, row by row")
    delta = []
    for line in DELTA.read_text(encoding="utf-8").splitlines()[1:]:
        cols = line.split("\t")
        delta.append((cols[0], int(cols[1]), cols[2].replace("\\\\", "\\")))
    check(len(delta) == 7, f"{len(delta)} delta literals")
    for name, line, literal in delta:
        source_line = (CRATE / "src/chapters" / name).read_text(encoding="utf-8").splitlines()[line - 1]
        in_source = literal in source_line or literal.replace("\\", "\\\\") in source_line
        hit = [(n, r[1]) for n, r in enumerate(rows, 1) if r[2] == literal]
        inside = [n for n, r in enumerate(rows, 1) if literal in r[2] and r[2] != literal and r[0] == "Tex"]
        where = f"registry row {hit[0][0]}, {hit[0][1]}" if hit else f"no row of its own; inside TeX rows {inside}"
        check(in_source and (hit or inside), f"{name}:{line} {literal!r}: {where}")

    print("\n# 7. strings outside the chapter list")
    thumb = drop_tests(lex((CRATE / "src/chapters/thumbnail.rs").read_text(encoding="utf-8")))
    for k, tok in enumerate(thumb):
        if tok[0] == "str" and tok[1].strip():
            print(f"    thumbnail.rs:{tok[2]} in {callee(thumb, k)[0]}: {tok[1]!r} {'on the registry' if tok[1] in sources else 'NOT on the registry'}")

    print("\n# 8. strings the film draws in a style the probe page does not")
    print("  The probe sets every Text row in the regular face. text_styled draws bold or italic, from other faces.")
    for path in files:
        toks = drop_tests(lex(path.read_text(encoding="utf-8")))
        for k, tok in enumerate(toks):
            if tok[0] == "id" and tok[1] == "text_styled":
                j = k + 1
                end = close_of(toks, j)
                args = toks[j:end]
                vals = [t[1] for t in args if t[0] == "str"]
                flags = [t[1] for t in args if t[1] in ("true", "false")]
                print(f"    {path.name}:{tok[2]} {vals} bold,italic={flags}")

    print("\n# 9. registry rows not spelled as one literal: computed, read on the pages")
    literal_values = set()
    for path in files:
        literal_values.update(t[1] for t in lex(path.read_text(encoding="utf-8")) if t[0] == "str")
    computed = [(n, r) for n, r in enumerate(rows, 1) if r[2] not in literal_values]
    print(f"  {len(computed)} of {len(rows)}")
    for n, r in computed:
        print(f"    {n:4d} {r[0]:4s} {r[1][:30]:30s} {r[2]!r}")

    print("\nRESULT", "FAILED" if failed else "OK: controls and bindings hold; read the UNMATCHED list by hand")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
