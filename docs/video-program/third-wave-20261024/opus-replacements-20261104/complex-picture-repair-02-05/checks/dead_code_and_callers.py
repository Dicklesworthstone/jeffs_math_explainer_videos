"""What a compiler would warn about first, read from source; and who calls the shared code.

    python3 -I -B checks/dead_code_and_callers.py

Two passes over candidate/ laid over canonical, because the stage holds only the
files it touches. (1) Every name a touched chapter imports is used outside its `use`
in non-test code, or in tests only if the import is cfg(test). (2) Every item this
repair added is used in non-test code, or is cfg(test): this crate is a binary, so an
unused `pub` item is a dead_code warning. Text matching, so it can miss what rustc
would not, and it does not replace the build. (3) Lists the callers of each shared
change, by file and line, from canonical. Exit 1 on (1) or (2).
"""
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parents[5]
CANON = REPO / "explainer_corpus" / "complex_analysis_visual" / "src"
CAND = HERE.parent / "candidate" / "explainer_corpus" / "complex_analysis_visual" / "src"
BASE = HERE.parent / "baseline" / "explainer_corpus" / "complex_analysis_visual" / "src"
failures = []


def overlay():
    files = {p.relative_to(CANON).as_posix(): p.read_text() for p in CANON.rglob("*.rs")}
    files.update({p.relative_to(CAND).as_posix(): p.read_text() for p in CAND.rglob("*.rs")})
    return files


def split_tests(text):
    m = re.search(r"#\[cfg\(test\)\]\nmod tests", text)
    return (text[: m.start()], text[m.start() :]) if m else (text, "")


def strip_comments(text):
    return re.sub(r"//[^\n]*", "", text)


files = overlay()
print("-- imports in the touched files are used in non-test code")
for rel in ("chapters/turning.rs", "chapters/two_planes.rs", "chapters/reading_color.rs", "chapters/gallery.rs", "chapters/draw.rs", "model.rs"):
    code, _ = split_tests(files[rel])
    code_nc = strip_comments(code)
    uses = re.findall(r"^use [^;]+;", code_nc, re.M)
    body = code_nc
    for u in uses:
        body = body.replace(u, "")
    for u in uses:
        if "prelude::*" in u:
            continue
        names = re.findall(r"\b(\w+)\b(?!\s*::)", u.split("::", 1)[1] if "{" not in u else u[u.index("{") :])
        for name in names:
            if name in ("self", "as", "use"):
                continue
            alias = re.search(rf"\b{name} as (\w+)", u)
            look = alias.group(1) if alias else name
            n = len(re.findall(rf"\b{look}\b", body))
            if n == 0 and name != "as":
                # a trait imported for its methods reads as unused; the baseline shows which that is
                base_code = strip_comments(split_tests((BASE / rel).read_text())[0])
                for bu in re.findall(r"^use [^;]+;", base_code, re.M):
                    base_code = base_code.replace(bu, "")
                if re.search(rf"\b{look}\b", base_code) or not re.search(rf"\b{name}\b", (BASE / rel).read_text()):
                    print(f"  FAIL {rel}: {name} is imported and unused outside tests")
                    failures.append(f"{rel}:{name}")
                else:
                    print(f"  note {rel}: {name} reads as unused here and in the baseline (a trait import)")
print("  done")

print("-- items this repair added are used outside tests, or are cfg(test)")
base_all = {p.relative_to(BASE).as_posix(): p.read_text() for p in BASE.rglob("*.rs")}
non_test = {rel: strip_comments(split_tests(t)[0]) for rel, t in files.items()}
all_code = "\n".join(non_test.values())
for rel, base_text in base_all.items():
    old = set(re.findall(r"(?:pub )?(?:const|fn) (\w+)", split_tests(base_text)[0]))
    code = split_tests(files[rel])[0]
    new = set(re.findall(r"(?:pub )?(?:const|fn) (\w+)", code)) - old
    for name in sorted(new):
        uses = len(re.findall(rf"\b{name}\b", all_code))
        decl = re.search(rf"(#\[cfg\(test\)\]\s*\n)?\s*(?:pub )?(?:const|fn) {name}\b", code)
        testonly = bool(decl and decl.group(1))
        ok = uses >= 2 or testonly
        print(f"  {'ok  ' if ok else 'FAIL'} {rel}: {name}: {uses - 1} use(s) outside tests{' (cfg(test))' if testonly else ''}")
        if not ok:
            failures.append(f"{rel}:{name}")
    # an item that lost its last use
    for name in sorted(old):
        if name in ("main", "name", "construct"):
            continue
        if not re.search(rf"\b{name}\b", code):
            continue
        uses = len(re.findall(rf"\b{name}\b", all_code))
        decl = re.search(rf"(#\[cfg\(test\)\]\s*\n)?\s*(?:pub )?(?:const|fn) {name}\b", code)
        if uses < 2 and not (decl and decl.group(1)):
            # the same name in another scope, or a method, can hide or fake this: name it and judge
            was = len(re.findall(rf"\b{name}\b", strip_comments(split_tests(base_text)[0])))
            if was >= 2:
                print(f"  FAIL {rel}: {name} lost its last use")
                failures.append(f"{rel}:{name} lost its last use")

print("-- callers of the shared changes, from canonical")
for label, pattern in (
    ("Painter::build (P1 crisp)", r"Painter::build\("),
    ("draw::portrait (P1 crisp)", r"(?<![\w:])portrait\(stage"),
    ("draw::portrait_colors (P3)", r"portrait_colors\("),
    ("model::portrait, model::shade (P2)", r"model::portrait\(|model::shade\(|[^_\w]shade\("),
    ("wheel_legend (P2)", r"wheel_legend\("),
    ("band_step (P2)", r"band_step\("),
    ("TILE_SEED, tile (P2)", r"TILE_SEED|[^_\w]tile\(\)"),
    ("RowStrips::build (other pictures)", r"RowStrips::build\("),
):
    rows = []
    for p in sorted(CANON.rglob("*.rs")):
        code, tests = split_tests(p.read_text())
        at = len(code.splitlines())
        for i, line in enumerate(p.read_text().splitlines(), 1):
            if re.search(pattern, line) and not line.strip().startswith("//"):
                rows.append(f"{p.relative_to(CANON).as_posix()}:{i}{' (test)' if i > at else ''}")
    print(f"  {label}: {len(rows)}")
    print("    " + ", ".join(rows))

print(f"\n{len(failures)} failed" + (": " + "; ".join(failures) if failures else ""))
sys.exit(1 if failures else 0)
