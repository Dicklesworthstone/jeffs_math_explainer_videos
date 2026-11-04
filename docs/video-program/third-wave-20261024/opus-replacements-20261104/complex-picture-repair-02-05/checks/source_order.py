"""Source-reading mirrors, on the candidate: tables, phrases, cards, cue words.

    python3 -I -B checks/source_order.py

Mirrors what these tests read from source, so a mistake shows before root's build:
no_supplement_is_empty_or_duplicated, each_card_outlasts_its_reading_window (the
two sites this repair edits), every_colour_picture_is_crisp. Adds what no test
reads: every phrase a chapter waits on is literal in its cue and in order, and the
cue words are unchanged. Reading source is not compiling it. Exit 1 on any failure.
"""
import hashlib
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
STAGE = HERE.parent
CAND = STAGE / "candidate" / "explainer_corpus" / "complex_analysis_visual"
BASE = STAGE / "baseline" / "explainer_corpus" / "complex_analysis_visual"
CH = CAND / "src" / "chapters"
failures = []


def check(name, ok, detail=""):
    print(f"  {'ok  ' if ok else 'FAIL'} {name}{'  ' + detail if detail else ''}")
    if not ok:
        failures.append(name)


# ----------------------------------------------------------- the two tables
print("-- TEXT and TEX: no exact duplicate, nothing empty, every name resolves")
mod = (CH / "mod.rs").read_text()
# the stage holds only the files this repair touches; other chapters' constants come from canonical
REPO = HERE.parents[5]
sources = {p.stem: p.read_text() for p in (REPO / "explainer_corpus" / "complex_analysis_visual" / "src" / "chapters").glob("*.rs")}
sources.update({p.stem: p.read_text() for p in CH.glob("*.rs")})
STR = r'(r#?"(?:.|\n)*?"#?|"(?:[^"\\]|\\.)*")'


def literal(tok):
    if tok.startswith("r"):
        return tok[tok.index('"') + 1 : tok.rindex('"')]
    return tok[1:-1].replace('\\"', '"').replace("\\\\", "\\")


def resolve(module, name, index):
    text = sources[module]
    alias = re.search(rf"use super::(\w+)::\w+ as {name};|use super::(\w+)::{name};", text)
    single = re.search(rf"pub const {name}: &str =\s*{STR};", text)
    if single:
        return literal(single.group(1))
    many = re.search(rf"pub const {name}: \[&str; \d+\] = \[(.*?)\];", text, re.S)
    if many:
        items = [literal(m.group(1)) for m in re.finditer(STR, many.group(1))]
        return items[int(index)] if index is not None else items
    if alias:
        return resolve(alias.group(1) or alias.group(2), name, index)
    raise KeyError(f"{module}::{name}")


for table in ("TEXT", "TEX"):
    body = re.search(rf"pub const {table}: &\[&str\] = &\[(.*?)\n\];", mod, re.S).group(1)
    values = []
    unresolved = []
    for raw in body.splitlines():
        line = raw.split("//")[0].strip().rstrip(",")
        if not line:
            continue
        m = re.fullmatch(r"(\w+)::(\w+)(?:\[(\d+)\])?", line)
        if m:
            try:
                v = resolve(m.group(1), m.group(2), m.group(3))
                values.append(v)
            except KeyError as e:
                unresolved.append(str(e))
        else:
            for lit in re.finditer(STR, line):
                values.append(literal(lit.group(1)))
    flat = [v for v in values if isinstance(v, str)]
    dup = sorted({v for v in flat if flat.count(v) > 1})
    check(f"{table}: {len(flat)} entries, no duplicate", not dup, repr(dup) if dup else "")
    check(f"{table}: nothing empty", all(v.strip() for v in flat))
    check(f"{table}: every name resolves", not unresolved, repr(unresolved) if unresolved else "")
    if table == "TEX":
        for need in ("z", "f(z)", "1", "i", r"2\pi i", r"f(z) = z", r"f(z) = \frac{1}{1 + z^2}", r"z^3 - 1", r"\frac{z-1}{(z+1)^2}", r"e^z", r"\sin z", r"\Gamma(z)", r"\zeta(s)"):
            check(f"TEX has {need!r}", need in flat)
    else:
        for need in ("new windows: the ticks still mark whole numbers", "gold segment: where the recipe works", "output f(z), sizes squashed", "1/2 + ", " turn, at −i: ", " turn"):
            check(f"TEXT has {need!r}", need in flat)

# ------------------------------------------------------------- phrase order
print("-- every phrase waited on is literal in its cue, and in order")
script = {}
for raw in (CAND / "script.tsv").read_text().splitlines():
    if "\t" in raw:
        k, v = raw.split("\t", 1)
        script[k] = v
PHRASE = re.compile(r'line\.(?:wait_for|until|at)\(\s*(?:stage,\s*)?"([^"]+)"')
SAY = re.compile(r'say\(&kit, stage, "(\d\d_\d+)"\)')
WALK = re.compile(r'Self::walk_and_count\(\s*&kit,\s*stage,\s*&line,\s*"([^"]+)"')
for name in ("turning", "two_planes", "reading_color", "gallery"):
    text = sources[name]
    code = text[: text.index("#[cfg(test)]")]
    marks = [(m.start(), m.group(1)) for m in SAY.finditer(code)]
    total = 0
    for i, (start, cue) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(code)
        chunk = code[start:end]
        pos = -1
        found = sorted(
            [(m.start(), m.group(1)) for m in PHRASE.finditer(chunk)] + [(m.start(), m.group(1)) for m in WALK.finditer(chunk)]
        )
        for _, phrase in found:
            total += 1
            at = script[cue].find(phrase)
            ok = at >= 0
            check(f"{cue}: {phrase!r}", ok and at > pos, f"at char {at}")
            pos = max(pos, at)
    named = re.search(r"pub const NAMED: \[\(&str, f64\); 4\] = \[(.*?)\];", code, re.S)
    if named:
        pos = -1
        for m in re.finditer(r'\("([^"]+)",', named.group(1)):
            at = script["04_1"].find(m.group(1))
            check(f"04_1: {m.group(1)!r} (NAMED)", at > pos, f"at char {at}")
            pos = at

print("-- sentence indices waited on exist in their cues")


def sentences(text):
    chars = list(text)
    return 1 + sum(1 for i in range(len(chars) - 2) if chars[i] in ".?!" and chars[i + 1] == " ")


for name in ("turning", "two_planes", "reading_color", "gallery"):
    text = sources[name]
    code = text[: text.index("#[cfg(test)]")]
    marks = [(m.start(), m.group(1)) for m in SAY.finditer(code)]
    for i, (start, cue) in enumerate(marks):
        end = marks[i + 1][0] if i + 1 < len(marks) else len(code)
        asked = [int(x) for x in re.findall(r"line\.wait_sentence\(stage, (\d+)\)", code[start:end])]
        have = sentences(script[cue])
        check(f"{cue}: sentences {asked} of {have}, rising", all(a < have for a in asked) and asked == sorted(asked))

# ------------------------------------------------------- cards and their windows
print("-- each_card_outlasts_its_reading_window, the two edited sites")
for cue, nxt, card, source in (("04_4", "04_5", "exact", sources["reading_color"]), ("05_8", "05_9", "four", sources["gallery"])):
    say = "let line = say(&kit, stage, "
    window = "reading_window(&kit, stage, &line)?;"
    here = source.index(f'{say}"{cue}")?;')
    shown = source.index(f"let {card} = ", here)
    held = source.index(window, here)
    then = source.index(f'{say}"{nxt}")?;', here)
    rest = source[then + 1 :]
    cleared = rest[: rest.index(say)] if say in rest else rest
    check(f"{cue}: shown before held", shown < held)
    check(f"{cue}: nothing cleared before the window", "fade_out" not in source[shown:held] and "remove_from_scene" not in source[shown:held])
    check(f"{cue}: voice ends before the card", cue != "04_4" or "wait_until(stage, line.end())?;" in source[here:shown])
    check(f"{cue}: next cue follows the window at once", source[held + len(window) : then].strip() == "")
    check(f"{cue}: no other cue between", say not in source[here + 1 : then])
    check(f"{cue}: the next cue clears the card", "fade_out_all" in cleared and re.search(rf"\b{card}\b", cleared) is not None)

print("-- every_colour_picture_is_crisp")
draw = sources["draw"]
code = draw[: draw.index("#[cfg(test)]\nmod tests")]
check("2 pictures, 2 crisp calls", code.count("RowStrips::build(") == 2 and code.count("crisp(stage, &strips);") == 2)

# ----------------------------------------------------------------- cue words
print("-- cue words")
a, b = (CAND / "script.tsv").read_bytes(), (BASE / "script.tsv").read_bytes()
check("script.tsv byte-identical to the baseline", a == b, hashlib.sha256(a).hexdigest())


def word_column(path):
    """Storyboard table rows: every cell but the picture column. Rows are matched by their first cell."""
    rows = {}
    for raw in path.read_text().splitlines():
        if raw.startswith("|") and re.match(r"\|\s*`?\d\d_\d+", raw):
            cells = [c.strip() for c in raw.strip().strip("|").split("|")]
            rows[cells[0]] = cells
    return rows


base_rows, cand_rows = word_column(BASE / "docs" / "storyboard.md"), word_column(CAND / "docs" / "storyboard.md")
check("storyboard: same rows", list(base_rows) == list(cand_rows), f"{len(cand_rows)} rows")
differ = [k for k in base_rows if base_rows[k] != cand_rows[k]]
print(f"       storyboard rows whose picture column changed: {differ}")
check("storyboard: the words column of every row is unchanged", all(base_rows[k][1] == cand_rows[k][1] for k in base_rows))
check("storyboard: only the picture column changed", all(base_rows[k][:2] + base_rows[k][3:] == cand_rows[k][:2] + cand_rows[k][3:] for k in base_rows))
base_text, cand_text = (BASE / "docs" / "storyboard.md").read_text(), (CAND / "docs" / "storyboard.md").read_text()
for cue, words in script.items():
    if cue[:2] in ("02", "03", "04", "05"):
        check(f"storyboard still holds the words of {cue}", (words in cand_text) == (words in base_text))

print(f"\n{len(failures)} failed" + (": " + "; ".join(failures) if failures else ""))
sys.exit(1 if failures else 0)
