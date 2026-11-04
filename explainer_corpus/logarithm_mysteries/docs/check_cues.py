"""Cheap source check for the logarithm rebuild candidate (no Cargo).

Run from the crate root:  python3 -I -B docs/check_cues.py

It reads what the film compiles: src/chapters/mod.rs and the modules it
declares, split into Rust tokens, with #[cfg(test)] and #[test] items taken
out. A comment or a string literal is never a call. A file mod.rs does not
declare is not compiled, so it is named as ignored and not read.

Checks, against script.tsv and the kit's sentence rule (split after . ? !
followed by a space or the end, as in explainer_kit narration.rs):
  * the registry in mod.rs is a nonempty plain list, with chapters in cue
    order, one per cue prefix, each declared and with its file present;
  * every cue is said exactly once, by its own chapter, in script order;
  * every until_sentence(id, j) names the cue being said, 0 < j < sentences(id),
    and within a cue the indices never go backwards;
  * a say or until_sentence this check cannot read is refused, not skipped;
    the only computed index allowed is a row of REVIEWED_COMPUTED;
  * a say or until_sentence in a declared module the registry does not run,
    or a cfg it cannot evaluate, is refused;
  * spoken text holds no digits (numbers are spelled for the narrator);
  * string literals in chapter code that contain digits are listed, so a
    reviewer can confirm each is a label, an index or a facts.rs value.
Exit status 1 on any failure; the listings are informational.

This reads text. It does not compile anything or run a scene, and it takes
file order within a chapter for the order the scene runs in.
"""

import pathlib
import re
import sys
from collections import namedtuple

ROOT = pathlib.Path(__file__).resolve().parent.parent
CHAPTERS = ROOT / "src" / "chapters"

CUE = re.compile(r"[0-9]{2}_[0-9]+")
CHAPTER_KEY = re.compile(r"[0-9]{2}_[a-z0-9_]+")
HOOKS = ("say", "until_sentence")
# Waits and lookups in the chapter modules that take a cue. Counted, not checked.
UNCHECKED_HELPERS = ("until_phrase", "until_share", "phrase_time", "phrase_seconds", "sentence_time", "sentence_starts")
# What a #[cfg(test)] or #[test] item may start with. Anything else, a match arm
# or a struct field, has an end this check can't find, so it is refused.
ITEM_WORDS = {
    "async", "const", "enum", "extern", "fn", "impl", "let", "macro_rules", "mod",
    "static", "struct", "trait", "type", "union", "unsafe", "use",
}  # fmt: skip

# Sentence hooks with a computed index, each read by a person. Any other computed
# index is refused. `first` is the index the loop starts on. The last is derived
# from the literal array, one step per element, so a changed array can't leave a
# stale bound. A changed loop or fill refuses the row until someone reads it again.
REVIEWED_COMPUTED = [
    {
        "file": "product_rule.rs",
        "cue": "08_4",
        "index": "k + 1",
        "first": 1,
        "array": "steps_src",
        "fill": "for ( k , src ) in steps_src . iter ( ) . enumerate ( ) {",
        "push": "steps . push (",
        "loop": "for ( k , & m ) in steps . iter ( ) . enumerate ( ) {",
    },
]

Token = namedtuple("Token", "kind text line")
Hook = namedtuple("Hook", "line kind cue low high label")
OPENERS = {"(": ")", "[": "]", "{": "}"}


class Refusal(Exception):
    """Source this check can't read with certainty."""


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            part = text[start : i + 1].strip()
            if part:
                out.append(part)
            start = i + 1
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return out


def script(failures):
    """The cues of script.tsv. A row the kit could not parse is a failure, not a crash."""
    path = ROOT / "script.tsv"
    try:
        rows = path.read_text(encoding="utf-8").splitlines()
    except FileNotFoundError:
        failures.append("script.tsv missing")
        return []
    except (OSError, UnicodeDecodeError) as e:
        failures.append(f"script.tsv cannot be read: {type(e).__name__}")
        return []
    cues = []
    for n, line in enumerate(rows, 1):
        if not line.strip() or line.startswith("#"):
            continue
        if "\t" not in line:
            failures.append(f"script.tsv:{n}: no tab between cue id and text")
            continue
        cid, text = line.split("\t", 1)
        if not CUE.fullmatch(cid):
            failures.append(f"script.tsv:{n}: bad cue id {cid!r}")
            continue
        if not text.strip():
            failures.append(f"script.tsv:{n}: {cid} has no text")
            continue
        cues.append((cid, text))
    if not cues:
        failures.append("script.tsv has no cues")
    return cues


# ------------------------------------------------------------------- Rust tokens

RAW_STRING = re.compile(r'(?:b|c)?r(#*)"')
PREFIXED_STRING = re.compile(r'[bc]"')
IDENT = re.compile(r"(?:r#)?[^\W\d]\w*")
NUMBER = re.compile(r"[0-9]\w*")


def lex(src):
    """Rust tokens with their lines. Comments go; each string is one token.

    Kinds: id, num, str (a plain "..."), raw (raw, byte or C string), char, life, p.
    Punctuation is one character at a time, so `::` is two tokens.
    """
    tokens, i, line, n = [], 0, 1, len(src)
    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
        elif c.isspace():
            i += 1
        elif src.startswith("//", i):
            end = src.find("\n", i)
            i = n if end < 0 else end
        elif src.startswith("/*", i):
            depth, j = 1, i + 2
            while depth:
                if j >= n:
                    raise Refusal(f"line {line}: a block comment never closes")
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            line += src.count("\n", i, j)
            i = j
        elif RAW_STRING.match(src, i):
            opened = RAW_STRING.match(src, i)
            close = '"' + opened.group(1)
            end = src.find(close, opened.end())
            if end < 0:
                raise Refusal(f"line {line}: a raw string never closes")
            tokens.append(Token("raw", src[opened.end() : end], line))
            line += src.count("\n", i, end)
            i = end + len(close)
        elif c == '"' or PREFIXED_STRING.match(src, i):
            start = src.index('"', i) + 1
            j = start
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            if j >= n:
                raise Refusal(f"line {line}: a string never closes")
            tokens.append(Token("str" if c == '"' else "raw", src[start:j], line))
            line += src.count("\n", i, j)
            i = j + 1
        elif c == "'":
            if src.startswith("\\", i + 1):
                # the escaped character can be a quote, so the search starts after it
                end = src.find("'", i + 3)
            elif src.startswith("'", i + 2):
                end = i + 2
            else:
                life = IDENT.match(src, i + 1)
                if not life:
                    raise Refusal(f"line {line}: a stray quote")
                tokens.append(Token("life", life.group(), line))
                i = life.end()
                continue
            if end < 0:
                raise Refusal(f"line {line}: a character literal never closes")
            tokens.append(Token("char", src[i + 1 : end], line))
            i = end + 1
        else:
            word = IDENT.match(src, i)
            number = NUMBER.match(src, i)
            if word:
                text = word.group()
                tokens.append(Token("id", text[2:] if text.startswith("r#") else text, line))
                i = word.end()
            elif number:
                tokens.append(Token("num", number.group(), line))
                i = number.end()
            else:
                tokens.append(Token("p", c, line))
                i += 1
    return tokens


def at(tokens, i, pattern):
    """Whether the tokens from i are these words and punctuation. Never matches a literal."""
    words = pattern.split()
    if i < 0 or i + len(words) > len(tokens):
        return False
    return all(tokens[i + k].kind in ("id", "p") and tokens[i + k].text == w for k, w in enumerate(words))


def close_of(tokens, i):
    """The index of the bracket that closes the one at i."""
    stack = []
    for j in range(i, len(tokens)):
        t = tokens[j]
        if t.kind != "p":
            continue
        if t.text in OPENERS:
            stack.append(OPENERS[t.text])
        elif t.text in ")]}":
            if not stack or stack.pop() != t.text:
                raise Refusal(f"line {t.line}: unbalanced {t.text}")
            if not stack:
                return j
    raise Refusal(f"line {tokens[i].line}: {tokens[i].text} never closes")


def end_of_test_item(tokens, i, line):
    """The index just past the item a test attribute covers."""
    while at(tokens, i, "# ["):
        i = close_of(tokens, i + 1) + 1
    if at(tokens, i, "pub"):
        i += 1
        if at(tokens, i, "("):
            i = close_of(tokens, i) + 1
    if i >= len(tokens) or not (tokens[i].kind == "id" and tokens[i].text in ITEM_WORDS or at(tokens, i, "{")):
        raise Refusal(f"line {line}: a test attribute on something that is not an item")
    while i < len(tokens):
        t = tokens[i]
        if t.kind == "p":
            if t.text == ";":
                return i + 1
            if t.text == "{":
                return close_of(tokens, i) + 1
            if t.text in "([":
                i = close_of(tokens, i)
            elif t.text in ")]}":
                raise Refusal(f"line {t.line}: a test attribute's item ends at an unmatched {t.text}")
        i += 1
    raise Refusal(f"line {line}: a test attribute's item never ends")


def outside_tests(tokens):
    """What a normal build compiles. Refuses what this check can't evaluate."""
    kept, i = [], 0
    while i < len(tokens):
        t = tokens[i]
        if at(tokens, i, "# [") or at(tokens, i, "# ! ["):
            inner = at(tokens, i, "# !")
            opened = i + (2 if inner else 1)
            closed = close_of(tokens, opened)
            words = [a.text for a in tokens[opened + 1 : closed]]
            head = words[0] if words else ""
            if head == "path":
                raise Refusal(f"line {t.line}: #[path] moves a module where this check won't look")
            if head in ("cfg", "cfg_attr"):
                if inner or words != ["cfg", "(", "test", ")"]:
                    raise Refusal(f"line {t.line}: a cfg this check can't evaluate: {' '.join(words)}")
                i = end_of_test_item(tokens, closed + 1, t.line)
                continue
            if words == ["test"] and not inner:
                i = end_of_test_item(tokens, closed + 1, t.line)
                continue
            kept.extend(tokens[i : closed + 1])
            i = closed + 1
            continue
        if at(tokens, i, "cfg ! ("):
            raise Refusal(f"line {t.line}: cfg! switches code at build time")
        if at(tokens, i, "include ! ("):
            raise Refusal(f"line {t.line}: include! pulls in source this check won't read")
        kept.append(t)
        i += 1
    return kept


def read_rust(path):
    try:
        src = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as e:
        raise Refusal(f"cannot be read: {type(e).__name__}") from None
    return outside_tests(lex(src))


# ------------------------------------------------------------------- the registry


def chapter_entry(tokens, i):
    """(key, module) from the tokens just inside `Chapter {`."""
    line = tokens[i - 1].line
    make = ", make : | kit | Box : : new ("
    words = len(make.split())
    named = at(tokens, i, "key :") and tokens[i + 2].kind == "str" and at(tokens, i + 3, make)
    if not named or not CHAPTER_KEY.fullmatch(tokens[i + 2].text):
        raise Refusal(f"line {line}: a Chapter that does not start key: \"NN_name\", make: |kit| Box::new(")
    module = tokens[i + 3 + words]
    if module.kind != "id" or not at(tokens, i + 4 + words, ": :"):
        raise Refusal(f"line {line}: a Chapter whose make names no module")
    return tokens[i + 2].text, module.text


def registry(tokens):
    """[(key, module)] from fn registry, refusing anything but a plain vec![Chapter { .. }, ..]."""
    starts = [i for i in range(len(tokens)) if at(tokens, i, "fn registry (")]
    if len(starts) != 1:
        raise Refusal(f"{len(starts)} fn registry found, not 1")
    body = starts[0]
    while body < len(tokens) and not at(tokens, body, "{"):
        body += 1
    if body == len(tokens):
        raise Refusal("fn registry has no body")
    end = close_of(tokens, body)
    if not at(tokens, body + 1, "vec ! [") or close_of(tokens, body + 3) != end - 1:
        raise Refusal("fn registry is not a plain vec![..]")
    chapters, i = [], body + 4
    while i < end - 1:
        if not at(tokens, i, "Chapter {"):
            raise Refusal(f"line {tokens[i].line}: a registry entry that is not Chapter {{ .. }}")
        chapters.append(chapter_entry(tokens, i + 2))
        i = close_of(tokens, i + 1) + 1
        if at(tokens, i, ","):
            i += 1
    return chapters


def declared_modules(tokens):
    return {tokens[i + 1].text for i in range(len(tokens) - 2) if at(tokens, i, "mod") and tokens[i + 1].kind == "id" and at(tokens, i + 2, ";")}


# ------------------------------------------------------------------- the hooks


def array_len(tokens, name):
    """The length of the literal array in `let name = [ .. ]`, or None."""
    for i in range(len(tokens)):
        if at(tokens, i, f"let {name} = ["):
            items, pending, j, closed = 0, False, i + 4, close_of(tokens, i + 3)
            while j < closed:
                if at(tokens, j, ","):
                    items, pending = items + 1, False
                    j += 1
                elif tokens[j].kind == "p" and tokens[j].text in OPENERS:
                    pending = True
                    j = close_of(tokens, j) + 1
                else:
                    pending = True
                    j += 1
            return items + (1 if pending else 0)
    return None


def computed_bounds(name, tokens, kit_at, cue, index):
    """(first, last) for a reviewed computed index; a Refusal for any other."""
    for row in REVIEWED_COMPUTED:
        if (row["file"], row["cue"], row["index"]) != (name, cue, index):
            continue
        if not at(tokens, kit_at - len(row["loop"].split()), row["loop"]):
            raise Refusal(f"its reviewed loop `{row['loop']}` changed")
        for pattern in ("fill", "push"):
            found = sum(at(tokens, i, row[pattern]) for i in range(len(tokens)))
            if found != 1:
                raise Refusal(f"its reviewed {pattern} `{row[pattern]}` appears {found} times, not once")
        length = array_len(tokens, row["array"])
        if not length:
            raise Refusal(f"its reviewed array {row['array']} is gone or empty")
        return row["first"], row["first"] + length - 1
    raise Refusal("nobody has reviewed this computed index")


def hooks(name, tokens, failures):
    """Every say and until_sentence, in file order. Refuses a form it can't read."""
    out = []
    for i, t in enumerate(tokens):
        if t.kind != "id" or t.text not in HOOKS:
            continue
        called = at(tokens, i + 1, "(")
        reached = i > 0 and (at(tokens, i - 1, ".") or at(tokens, i - 1, ":"))
        if not called and not reached:
            continue  # a local variable
        where = f"{name}:{t.line}"
        if not called or not at(tokens, i - 2, "kit ."):
            failures.append(f"{where}: {t.text} not called as kit.{t.text}(stage, ..)")
            continue
        args = tokens[i + 2 : close_of(tokens, i + 1)]
        if args and at(args, len(args) - 1, ","):
            args = args[:-1]
        shown = " ".join(a.text for a in args).replace("\n", " ")
        if not (at(args, 0, "stage ,") and len(args) > 2 and args[2].kind == "str" and CUE.fullmatch(args[2].text)):
            failures.append(f"{where}: {t.text}({shown}) does not name a literal cue")
            continue
        cue = args[2].text
        if t.text == "say":
            if len(args) != 3:
                failures.append(f"{where}: say({shown}) has extra arguments")
                continue
            out.append(Hook(t.line, "say", cue, None, None, None))
            continue
        if len(args) < 5 or not at(args, 3, ","):
            failures.append(f"{where}: until_sentence({shown}) has no index")
            continue
        index = args[4:]
        if len(index) == 1 and index[0].kind == "num" and index[0].text.isdigit():
            j = int(index[0].text)
            out.append(Hook(t.line, "until", cue, j, j, str(j)))
            continue
        label = " ".join(a.text for a in index)
        try:
            low, high = computed_bounds(name, tokens, i - 2, cue, label)
        except Refusal as e:
            failures.append(f"{where}: until_sentence({cue}, {label}) has a computed index: {e}")
            continue
        out.append(Hook(t.line, "until", cue, low, high, label))
    return out


def main():
    failures = []
    cues = script(failures)
    order = {cid: k for k, (cid, _) in enumerate(cues)}
    count = {cid: len(sentences(text)) for cid, text in cues}
    if len(order) != len(cues):
        seen, repeated = set(), []
        for cid, _ in cues:
            if cid in seen:
                repeated.append(cid)
            seen.add(cid)
        failures.append(f"duplicate cue ids in script.tsv: {', '.join(repeated)}")
    for cid, text in cues:
        if re.search(r"[0-9]", text):
            failures.append(f"{cid}: digits in spoken text")

    # The registry says which modules the film runs, and in what order.
    chapters, read, declared = [], {}, set()
    mod_rs = CHAPTERS / "mod.rs"
    if not mod_rs.is_file():
        failures.append("src/chapters/mod.rs missing")
    else:
        try:
            read["mod.rs"] = read_rust(mod_rs)
            declared = declared_modules(read["mod.rs"])
            chapters = registry(read["mod.rs"])
            if not chapters:
                failures.append("mod.rs registry is empty")
        except Refusal as e:
            failures.append(f"mod.rs: {e}")
    keys = [key[:2] for key, _ in chapters]
    if chapters:
        prefixes = sorted({cid[:2] for cid, _ in cues})
        if keys != sorted(keys):
            failures.append(f"mod.rs registry out of order: {keys}")
        if len(set(keys)) != len(keys):
            failures.append(f"mod.rs registry repeats a chapter: {keys}")
        if cues and sorted(set(keys)) != prefixes:
            failures.append(f"mod.rs registry {keys} does not cover cue prefixes {prefixes}")

    # Read each module once. A file that is not declared is not compiled.
    modules = [module for _, module in chapters]
    for module in sorted({m for m in modules if modules.count(m) > 1}):
        failures.append(f"mod.rs registry runs {module} more than once")
    for module in sorted(set(modules) | declared):
        path = CHAPTERS / f"{module}.rs"
        if module in modules and module not in declared:
            failures.append(f"mod.rs registry names {module}, which mod.rs does not declare")
        if not path.is_file():
            failures.append(f"mod.rs {'registry names' if module in modules else 'declares'} {module}, but {module}.rs is missing")
            continue
        try:
            read[path.name] = read_rust(path)
        except Refusal as e:
            failures.append(f"{path.name}: {e}")

    # With no cues or no chapters, the one failure above says it. A line per
    # hook or per cue would only bury it.
    comparable = bool(cues and chapters)

    # A module the registry does not run has nowhere to say anything.
    for name in sorted(read):
        if name[:-3] in modules or not comparable:
            continue
        try:
            for hook in hooks(name, read[name], failures):
                failures.append(f"{name}:{hook.line}: {hook.kind} {hook.cue} in a module the registry does not run")
        except Refusal as e:
            failures.append(f"{name}: {e}")

    # Registry order, then file order within a chapter.
    said_by, computed = {}, []
    last_anywhere = None
    for key, module in chapters:
        name = f"{module}.rs"
        if name not in read or not comparable:
            continue
        try:
            found = hooks(name, read[name], failures)
        except Refusal as e:
            failures.append(f"{name}: {e}")
            continue
        last_said, last_j = None, {}
        for hook in found:
            where = f"{name}:{hook.line}"
            cid = hook.cue
            if cid not in order:
                failures.append(f"{where}: unknown cue {cid}")
                continue
            if hook.kind == "say":
                said_by.setdefault(cid, []).append(name)
                if cid[:2] != key[:2]:
                    failures.append(f"{where}: {cid} said in chapter {key}")
                if last_anywhere is not None and order[cid] <= order[last_anywhere]:
                    failures.append(f"{where}: {cid} said after {last_anywhere}")
                last_anywhere = cid
                last_said = cid
                continue
            if cid != last_said:
                failures.append(f"{where}: until_sentence({cid}, {hook.label}) while {last_said} is the current line")
            if not (0 < hook.low and hook.high < count[cid]):
                failures.append(f"{where}: until_sentence({cid}, {hook.label}) but {cid} has {count[cid]} sentences")
            if hook.low < last_j.get(cid, 0):
                failures.append(f"{where}: until_sentence({cid}, {hook.label}) goes backwards")
            last_j[cid] = hook.high
            if hook.low != hook.high:
                computed.append(f"{where} until_sentence({cid}, {hook.label}) runs {hook.low} to {hook.high} of {count[cid]} sentences")
    if comparable:
        for cid, _ in cues:
            who = said_by.get(cid, [])
            if len(who) != 1:
                failures.append(f"{cid}: said {len(who)} times ({', '.join(who) or 'never'})")

    print(f"{len(cues)} cues, {sum(count.values())} sentences, {len(said_by)} said")
    print(f"registered modules read, in registry order: {', '.join(modules) or 'none'}")
    print(f"other declared modules read, none may narrate: {', '.join(sorted(n[:-3] for n in read if n[:-3] not in modules)) or 'none'}")
    if CHAPTERS.is_dir():
        known = {f"{m}.rs" for m in declared} | {"mod.rs"}
        ignored = sorted(p.name for p in CHAPTERS.iterdir() if p.name not in known)
        print(f"not declared in mod.rs, so not compiled and not read: {', '.join(ignored) or 'none'}")
    print("computed sentence indices, checked at both ends of the reviewed range:")
    for line in computed:
        print(f"  {line}")
    unchecked = {helper: 0 for helper in UNCHECKED_HELPERS}
    for tokens in read.values():
        for i, t in enumerate(tokens):
            if t.kind == "id" and t.text in unchecked and at(tokens, i + 1, "(") and not at(tokens, i - 1, "fn"):
                unchecked[t.text] += 1
    print("other cue helpers called, not checked here: " + ", ".join(f"{h} {n}" for h, n in unchecked.items()))
    print("string literals with digits (review: label, index or facts value):")
    for name in sorted(read):
        for t in read[name]:
            if t.kind not in ("str", "raw"):
                continue
            if re.search(r"[0-9]", t.text) and not re.fullmatch(r"[0-9]{2}_[0-9]+|[0-9]{2}_[a-z_]+|[a-z_]+\.[a-z0-9_]+", t.text):
                shown = t.text.replace("\n", "\\n")
                print(f"  {name}:{t.line}: \"{shown}\"")
    if failures:
        print("FAIL")
        for f in failures:
            print("  " + f)
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
