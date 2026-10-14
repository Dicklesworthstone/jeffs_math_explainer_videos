#!/usr/bin/env python3
"""Every number in a video's docs/description.md is one of its facts (or is on its short documented allow-list).

    python3 -I tools/description_numbers.py <slug> [--oracle FILE] [--facts FILE] [-v]

The description is published text, and doctrine 2 holds for it as for the narration: a number typed beside
the facts is a defect (Video J's said 70.7 times closer for 8.1, three wrong signs for two, and another
run's digits). The tool reads explainer_corpus/<slug>/docs/description.md, extracts every number, and
requires each one to be

  * a fact: a number in a value of the oracle's output (oracle.py, run here with numpy via uv, as
    `facts-check` runs it; the oracle prints every key the video does, with identical values, and its
    research rows besides), or of --facts (the video's own `facts` output). The description may round a
    fact (9.3 million for 9303002.5, 0.35 for 0.35013, 34% for 0.34) but never change it;
  * on docs/description_numbers.tsv, one row per number: `number TAB context TAB kind TAB reason`. The
    context is a piece of the text the number stands in. The kinds: `year`, `cited` (a number of a cited
    source or a stated threshold, with its source named), `structural` (a dimension, an index, a count
    that is the statement itself: SO(3), "2 by 2"), `derived` (the reason is `=expression` over facts,
    evaluated here and required to give the number, rounded as the text likes: `=f("F-C.mvl.size")/256`;
    `f(key, i)` is the i-th number of a value, `n_facts()` the count of keys). A row nothing uses is an error, so the list cannot
    go stale. A small whole number (under 100, no point) must equal a fact's number exactly; any other may
    be a rounding of one;
  * bound to the fact it is about. The text is read in units: a list item with its continuation lines, a
    paragraph, a heading, a table or a code block. A unit that names fact keys ([F-D.h1.coverage],
    `F-K.lanes.*`, `[K-B1.twist_per_min, .twist_hz]` with a `.suffix` of the key before it) is about those
    facts, and every number in it must agree with a number of THAT fact, or have a row. Matching any
    fact of the oracle is not enough there: the oracle has hundreds of numbers, so a wrong 22.5 or 36
    is nearly always some other key's value. In a unit that names no key a number needs a row, however
    many facts it equals elsewhere: that proves nothing. (A line `#! legacy` in description_numbers.tsv
    keeps the old rule for a description not tagged yet: any fact of the oracle will do, and the summary
    says so.)
    Not checked against facts: 0 and 1, and the digits of a formula: a whole number under 100 that is a
    coefficient, an exponent or a term of an expression with a variable in it (`2k + 1`, `n(n + 1) / 2`,
    `x^2`, `√2`, `SE(3)`). A term of a sum of plain numbers is no formula digit (`6 + 29 = 35`, `3-4-5`): it
    must be bound or have a row, and the sum itself is evaluated. A unit after a number is no variable
    (`45 × 23 × 18 m` is a product of claims, `2k + 1` a formula). The digits of a formula are checked as
    algebra instead: an equation with letters on both sides (`(a + b)² = a² + 2ab + b²`) is evaluated at
    fixed points, and one that is no identity but would be with ONE number or sign changed (`3ab` for
    `2ab`) fails; a conditional equation or a definition has no such neighbour and is left alone.

Not read as numbers: HTML comments, the fact keys (G-A.joints), the beat ids (01_6), "chapter 13", "joints 5 and 7",
"axis 4", "row 5", "Table 3" and "column B"-style indices, "§6.2.2", "57(2)" (the volume and issue of a journal),
timestamps (12:34), list markers (1.) and a bold chapter label (- **06.**), URLs and DOIs, ISO dates, and every line under a Sources or
References heading (bibliographic volumes, pages and years belong to the source, not to the facts).
Signs are not checked (an unsigned match is a match): a claim about a sign needs its own test.
Spelled-out numbers are not read either. A match by truncation is no match (49.8 is not "49").
Every fact key the text names ([F-T.at.t1], `F-L.bracket.*`, `F-G.walks.k1..k4`) must be a key of the
oracle: a stale tag points the reader at a fact that is not there.

Exit 0 when every number is accounted for, 1 when any is not (each is printed with its line), 2 on usage.
"""

import ast
import math
import pathlib
import re
import sys
from decimal import ROUND_HALF_EVEN, ROUND_HALF_UP, Decimal, InvalidOperation, getcontext
from fractions import Fraction

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
getcontext().prec = 80

FACT_NUM = re.compile(r"[-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?")
FACT_KEY = re.compile(r"\b[A-Z]-[A-Z0-9]+\.[\w.*\-]*[\w*]")
BEAT_ID = re.compile(r"\b\d\d_\d+\b")
CHAPTER = re.compile(r"\b[Cc]hapters?\s+\d+(?:\s*(?:[–-]|and|to)\s*\d+)?")
INDEX_LABEL = re.compile(r"\b(?:[Jj]oints?|[Aa]x(?:is|es)|[Rr]ows?|[Cc]olumns?|[Tt]ables?|[Ff]igures?)\s+\d+(?:\s*(?:,|[–-]|and|or|to)\s*\d+)*")
VOLUME = re.compile(r"\b\d{1,3}\(\d{1,4}\)(?=[,;:.\s)]|$)")
SECTION = re.compile(r"§\s*\d+(?:\.\d+)*")
TIMESTAMP = re.compile(r"\b\d{1,2}:\d{2}(?::\d{2})?\b")
ISO_DATE = re.compile(r"\b\d{4}-\d{2}-\d{2}\b")
URL = re.compile(r"https?://\S+|\bdoi:\s*\S+|\bDOI\s+\S+|\barXiv:\s*\S+", re.I)
LIST_MARK = re.compile(r"^\s*(?:\d+[.)]\s|[-*+]\s+\*\*\d{1,2}\.\*\*)")  # "1. " and a bold chapter label "- **06.**"
SOURCES = re.compile(r"^#+\s*(Sources|References|Cited|Credits|Further reading)\b", re.I)
SUPER = {"⁰": "0", "¹": "1", "²": "2", "³": "3", "⁴": "4", "⁵": "5", "⁶": "6", "⁷": "7", "⁸": "8", "⁹": "9", "⁻": "-"}
UNITS = {"thousand": 3, "million": 6, "billion": 9, "trillion": 12}
NUMBER = re.compile(
    r"""(?P<int>\d{1,3}(?:,\d{3})+|\d+)(?:\.(?P<frac>\d+))?
        (?:\s*[×x]\s*10(?:(?P<sup>[⁻⁰¹²³⁴⁵⁶⁷⁸⁹]+)|\^\{?(?P<exp>-?\d+)\}?))?
        (?P<pct>\s?%)?(?P<deg>°)?
        (?:\s+(?P<unit>thousand|million|billion|trillion)\b)?""",
    re.X,
)
BARE_POWER = re.compile(r"\b10(?:(?P<sup>[⁻⁰¹²³⁴⁵⁶⁷⁸⁹]+)|\^\{?(?P<exp>-?\d+)\}?)")
KINDS = {"year", "cited", "structural", "derived"}
BULLET = re.compile(r"^\s{0,3}([-*+]|\d+[.)])\s")
BRACKET = re.compile(r"\[([^\[\]]*)\]")
FORMULA_OPS = "+−-×÷*/^√"
FORMULA_SUPERSCRIPTS = "⁰¹²³⁴⁵⁶⁷⁸⁹ⁿ"
SUM = re.compile(
    r"(?<![\w.,])(?P<expr>\d[\d,]*(?:\.\d+)?(?:\s*[-+−×x÷*/]\s*\d[\d,]*(?:\.\d+)?)+)\s*=\s*(?P<rhs>\d[\d,]*(?:\.\d+)?)(?![\w.]|,\d)"
)


class Number:
    """A number as the description writes it."""

    def __init__(self, text, mantissa, decimals, shift, percent, line, context):
        self.unit = 0  # the unit of text (list item, paragraph, table...) it stands in
        self.formula = False  # a digit of a formula: next to an operator or a variable
        self.text = text  # as written, thousands separators dropped, for the allow-list
        self.mantissa = mantissa  # Decimal of the digits as written
        self.decimals = decimals  # digits after the point as written
        self.shift = shift  # the power of ten written with it (× 10⁷, million)
        self.percent = percent
        self.line = line
        self.context = context

    @property
    def strong(self):
        return self.decimals > 0 or self.shift != 0 or self.percent or abs(self.mantissa) >= 100

    def value(self):
        return self.mantissa.scaleb(self.shift)

    def __str__(self):
        return f"{self.text}{'%' if self.percent else ''}"


def quantize_equal(x, want, places):
    """Does x round (half up or half even; truncation is no rounding) to `want` at `places` decimals?"""
    try:
        q = Decimal(1).scaleb(-places)
        return any(x.quantize(q, rounding=mode) == want for mode in (ROUND_HALF_UP, ROUND_HALF_EVEN))
    except InvalidOperation:
        return False


def agrees(num, fact):
    """Does the description's number say this fact value (rounded as it likes, never changed)?"""
    fact = abs(fact)
    if not num.strong:
        return fact == abs(num.mantissa)  # a small whole number is no rounding of anything: it must BE a fact's number
    pool = [fact, fact * 100] if num.percent else [fact]
    return any(quantize_equal(f.scaleb(-num.shift), abs(num.mantissa), num.decimals) for f in pool)


_AGREES_MEMO = {}


def agrees_memo(num, fact):
    """agrees(), remembered: a description repeats its numbers and the oracle has thousands."""
    key = (num.mantissa, num.decimals, num.shift, num.percent, fact)
    hit = _AGREES_MEMO.get(key)
    if hit is None:
        hit = _AGREES_MEMO[key] = agrees(num, fact)
    return hit


def agrees_float(num, x):
    """Does an expression's value say this number? A derived row may round, a small whole number too."""
    try:
        x = Decimal(repr(abs(float(x))))
        if not num.strong:
            return quantize_equal(x, abs(num.mantissa), 0)
        return agrees(num, x)
    except (InvalidOperation, ValueError):
        return False


def fact_numbers(text):
    """(Decimal, key) for every number in every value of an oracle or facts output."""
    out = []
    for line in text.splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _, value = line.partition("\t")
        for tok in FACT_NUM.findall(value):
            try:
                out.append((abs(Decimal(tok)), key.strip()))
            except InvalidOperation:
                pass
    return out


def fact_values(text):
    """key -> value string."""
    vals = {}
    for line in text.splitlines():
        if line.strip() and not line.startswith("#"):
            key, _, value = line.partition("\t")
            vals[key.strip()] = value.strip()
    return vals


def blank(pattern, line):
    return pattern.sub(lambda m: " " * len(m.group(0)), line)


def named_keys(text):
    """(line, key) for every fact key the description names outside its comments."""
    text = re.sub(r"<!--.*?-->", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
    return [(text.count("\n", 0, m.start()) + 1, m.group(0)) for m in FACT_KEY.finditer(text)]


def key_exists(key, values):
    """A named key is a key of the oracle, a family of keys under it, a prefix with `.*`, or a range `a.k1..k4` of keys."""
    if key in values or any(k.startswith(key + ".") for k in values):
        return True  # a key, or a family of keys under it
    if key.endswith(".*"):
        return any(k == key[:-2] or k.startswith(key[:-1]) for k in values)
    if ".." in key:
        first, _, last = key.partition("..")
        return key_exists(first, values) and key_exists(first.rsplit(".", 1)[0] + "." + last, values)
    return False


# ------------------------------------------------------------------ units and tags

def blank_comments(text):
    """The text with its HTML comments blanked, line for line."""
    return re.sub(r"<!--.*?-->", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S)


def unit_ids(text):
    """One unit id per line (0 for a blank line): a list item with its continuation lines, a paragraph, a
    heading, a table (its rows and a line directly under it) or a code block is one unit."""
    ids, uid, new, fence, table = [], 0, True, False, False
    for line in blank_comments(text).splitlines():
        s = line.strip()
        if s.startswith("```"):
            if not fence:
                uid += 1
            fence, new, table = not fence, True, False
            ids.append(uid)
            continue
        if fence:
            ids.append(uid)
            continue
        if not s:
            ids.append(0)
            new, table = True, False
            continue
        is_row = s.startswith("|")
        if new or s.startswith("#") or BULLET.match(line) or (is_row and not table):
            uid += 1
        ids.append(uid)
        table = is_row or (table and not BULLET.match(line) and not s.startswith("#"))
        new = s.startswith("#")
    return ids


def unit_tags(text):
    """(unit id per line, {unit id: [tag]}) where a tag is a fact key as written or ('FAMILY', '.suffix')
    for the `.suffix` that follows a key inside the same brackets."""
    ids = unit_ids(text)
    lines_of = {}
    for uid, line in zip(ids, blank_comments(text).splitlines()):
        if uid:
            lines_of.setdefault(uid, []).append(line)
    tags = {}
    for uid, lines in lines_of.items():
        unit = "\n".join(lines)  # a bracket may wrap: [K-R1.error_percent,\n.error_bound_percent]
        for m in FACT_KEY.finditer(unit):
            tags.setdefault(uid, []).append(m.group(0))
        for b in BRACKET.finditer(unit):
            last = None
            for tok in re.split(r"[\s,;]+", b.group(1)):
                if FACT_KEY.fullmatch(tok):
                    last = tok
                elif last and re.fullmatch(r"\.[\w.*\-]*[\w*]", tok):
                    tags.setdefault(uid, []).append((last.split(".", 1)[0], tok))
    return ids, tags


def resolve_tag(tag, values):
    """The oracle keys a tag stands for: the key itself, its children, `prefix.*` (the key `prefix` too), a range `a.k1..k4`
    (every key sharing the prefix), or for `.suffix` every key of the family ending so."""
    if isinstance(tag, tuple):
        family, suffix = tag
        want = suffix.rstrip("*")
        return [k for k in values if k.startswith(family + ".") and k.endswith(want)]
    if tag in values:
        return [tag]
    if tag.endswith(".*"):
        return [k for k in values if k == tag[:-2] or k.startswith(tag[:-1])]
    if ".." in tag:
        first, _, _ = tag.partition("..")
        return [k for k in values if k.startswith(first.rsplit(".", 1)[0] + ".")]
    return [k for k in values if k.startswith(tag + ".")]


_OPERATORS = tuple("+−-×÷*/^=")


def _is_letter(c):
    return c.isalpha()


def _expression_around(clean, start, end):
    """The arithmetic expression the number at clean[start:end] belongs to, and whether it has a variable in
    it: a one-letter name joined by an operator or a bracket (`n + 1`, `k(k - 1)`, `2n`), a superscript or a
    root. A lone letter beside a number with only a space between (`a 3-4-5 triangle`) is a word."""
    plain = "0123456789.,+−-×÷*/^()√…⋯ "
    n = len(clean)
    lo = start
    while lo > 0:
        c = clean[lo - 1]
        if c in plain or c in FORMULA_SUPERSCRIPTS:
            lo -= 1
        elif _is_letter(c) and not (lo >= 2 and _is_letter(clean[lo - 2])) and not (lo < n and _is_letter(clean[lo])):
            nxt = clean[lo:].lstrip()[:1]
            if nxt and (nxt in "+−-×÷*/^=()" or nxt in FORMULA_SUPERSCRIPTS) or clean[lo:lo + 1] in "(":
                lo -= 1
            else:
                break
        else:
            break
    hi = end
    while hi < n:
        c = clean[hi]
        if c in plain or c in FORMULA_SUPERSCRIPTS:
            hi += 1
        elif _is_letter(c) and not (hi + 1 < n and _is_letter(clean[hi + 1])) and not (hi > 0 and _is_letter(clean[hi - 1])):
            prv = clean[:hi].rstrip()[-1:]
            # a letter joined to the expression: after an operator or bracket, or glued to a digit (2k). A letter a
            # space after a number is a unit or a word (`18 m`, `20 Hz`) and its superscript is the unit's (`m³`),
            # unless it is used like a variable: `50 e^{-0.2t}`, `3 x (y + 1)`, `2 n + 1`
            used = clean[hi + 1:].lstrip()[:1] in tuple("^+−-×÷*/=(")
            if prv and (prv in "+−-×÷*/^=()" or (prv.isdigit() and (clean[hi - 1] == prv or used))):
                hi += 1
            else:
                break
        else:
            break
    run = clean[lo:hi]
    variable = any(_is_letter(ch) for ch in run) or any(ch in FORMULA_SUPERSCRIPTS or ch == "√" for ch in run)
    return run, variable


def is_formula(clean, start, end):
    """A digit of a formula: a small whole number that is a coefficient, an exponent or a term of an
    expression with a variable in it (`2k + 1`, `n(n + 1) / 2`, `x^2`, `√2`). Not one: a term of a
    sum of plain numbers (`6 + 29 = 35`, `3-4-5`), whose operands are claims like any number (the sum
    itself is evaluated). Also not read: the argument of a name (`SE(3)`), the number that labels a
    vector (`2 (0, 1, 0)`), and the digits of a hyphenated name's suffix (`MTNS-83` is a name)."""
    prev_char = clean[:start].rstrip()[-1:]
    before = prev_char
    if prev_char == "-" and clean[:start].rstrip()[-2:-1].isalpha():
        before = ""
    after = clean[end:].lstrip()[:1]
    if clean[start - 1:start] == "(" and clean[start - 2:start - 1].isalpha() and clean[end:end + 1] == ")":
        return True  # a name with an argument: SE(3), f(2)
    if re.match(r"\s*\(\s*[-−]?\d+(?:\.\d+)?\s*,", clean[end:]):
        return True  # 2 (0, 1, 0): a number that labels a vector
    if clean[end:end + 1].isalpha():
        return True  # 2k, 3x
    if (after and after in FORMULA_SUPERSCRIPTS) or after == "^" or prev_char == "^":
        return True  # an exponent or the base of one
    if (before and before in FORMULA_OPS) or (after and after in FORMULA_OPS):
        # an operator beside it: a formula digit only when a variable is part of the expression
        return _expression_around(clean, start, end)[1]
    return False


ARITH_CHARS = set("0123456789.,+−-–×÷*/() ")


def _sum_left(line, eq):
    """The arithmetic expression that ends at the `=` at index `eq`: (start, text) or None."""
    i = eq
    while i > 0:
        c = line[i - 1]
        if c == "," and line[i:i + 1].isspace():
            break  # a comma and a space separate things (`23 x 46, (2 + 3) x ...`); a thousands comma has none
        if c in ARITH_CHARS or (c == "x" and i - 2 >= 0 and not line[i - 2].isalpha() and line[i:i + 1] in (" ", "")):
            i -= 1
            continue
        break
    text = line[i:eq]
    # take the expression whole: drop a leading operator or unmatched ")" until the parentheses balance
    while text.strip() and (text.lstrip()[0] in "+−-–×÷*/x.," or text.lstrip()[0] == ")"):
        k = len(text) - len(text.lstrip())
        text = text[k + 1:]
        i += k + 1
    depth = 0
    for k, c in enumerate(text):
        depth += (c == "(") - (c == ")")
        if depth < 0:
            return None
    while depth > 0:  # an unmatched "(" at the start: the expression is what follows it
        k = text.index("(")
        text = text[k + 1:]
        i += k + 1
        depth = 0
        for c in text:
            depth += (c == "(") - (c == ")")
    k = len(text) - len(text.lstrip())
    return i + k, text.strip()


def arithmetic_problems(text):
    """(line, message) for each complete sum in the text (`6 + 29 = 35`, `(2 + 3) x (4 + 6) - 8 - 18 = 24`)
    that does not come out, allowing for the rounding of the numbers in it. A sum that is only part of
    a longer expression or term (`k(k - 1) = ...`, `2 + 3 = 5 x 2`) is left alone."""
    problems = []
    lines = blank_comments(text).splitlines()
    ids = unit_ids(text)
    units = {}
    for no, (uid, line) in enumerate(zip(ids, lines), 1):
        units.setdefault(uid or -no, (no, []))[1].append(line.strip())
    for no, parts in units.values():
        line = " ".join(parts)  # a sum may wrap across the lines of its unit
        for m in re.finditer(r"=\s*(?P<rhs>(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?)", line):
            if line[:m.start()].rstrip()[-1:] in "<>≤≥≈=!":
                continue
            left = _sum_left(line, m.start())
            if left is None:
                continue
            start, expr = left
            if not re.search(r"\d\s*[-+−–×x÷*/]\s*[\d(]|\)\s*[-+−–×x÷*/]|\d\s*[-+−–×x÷*/]\s*\(", expr):
                continue
            glued = line[start - 1:start] if start else ""  # directly against the expression: k(k - 1), 2k + 1
            spaced = line[:start].rstrip()[-1:]  # the last thing before it: an operator makes it a part
            separated = spaced == "," and line[:start].endswith(" ")  # `23 x 46, (2 + 3) x ...`: a comma and a space end what came before
            after = line[m.end():].lstrip()[:1]
            if (glued and not glued.isspace()) or (spaced and spaced in "+−-–×÷*/^(.,_" and not separated) or (after and (after in "+−-–×÷*/^(%°" or after in FORMULA_SUPERSCRIPTS or after.isdigit())):
                continue
            py = expr.replace(",", "").replace("−", "-").replace("–", "-").replace("×", "*").replace("÷", "/")
            py = re.sub(r"(?<=[\d)\s])x(?=[\d(\s])", "*", py)
            if re.search(r"[\d)]\s*\(|\)\s*\d", py):
                continue  # implicit multiplication: not read
            try:
                result = Evaluator({}).run(py.strip())  # numbers and + - * / ** only, as a derived row is evaluated
                if not math.isfinite(result):
                    continue
                value = Decimal(repr(result))
            except (SyntaxError, ZeroDivisionError, ValueError, TypeError, InvalidOperation, OverflowError):
                continue
            rhs = Decimal(m.group("rhs").replace(",", ""))
            places = len(m.group("rhs").partition(".")[2])
            if quantize_equal(value, rhs, places):
                continue
            if abs(value - rhs) > Decimal(5).scaleb(-places - 1) + _operand_slack(expr, value):
                problems.append((no, f"{expr.strip()} = {m.group('rhs')} does not come out (the left side is {value:.6g})"))
    return problems


def _operand_slack(expr, value):
    """Half a unit of the last written place of every rounded (decimal) operand, relative to it, times the value."""
    rel = Decimal(0)
    for t in re.findall(r"\d[\d,]*\.\d+", expr):  # a whole number is a count, exact; a decimal is rounded
        x = Decimal(t.replace(",", ""))
        if x != 0:
            rel += Decimal(5).scaleb(-len(t.partition(".")[2]) - 1) / x
    return abs(value) * rel


# ------------------------------------------------------------------ identities with letters
#
# `(a + b)² = a² + 2ab + b²` is a claim of algebra, and a digit in it is the claim: 2ab typed 3ab publishes a false
# identity. Both sides are evaluated, exactly (Fractions), at several fixed points; the equation is an identity when
# they agree at all of them. An equation that is no identity may still be right as typed (a conditional equation like
# `p⁴ = 4p³(1 − p)` "exactly when p = 4(1 − p)", a definition, a law in other units): it is reported only when ONE
# number (a coefficient or an exponent) changed to a nearby value would make it an identity, which is what a typo in
# an identity looks like. Anything the small grammar does not read (functions, sums, ellipses, a letter glued to a
# digit, words) is left alone, and so is an equation with a lone symbol or a number on either side.

_SUP = {c: str(i) for i, c in enumerate("⁰¹²³⁴⁵⁶⁷⁸⁹")}
_SUB = "₀₁₂₃₄₅₆₇₈₉"
_VULGAR = {"½": Fraction(1, 2), "⅓": Fraction(1, 3), "⅔": Fraction(2, 3), "¼": Fraction(1, 4), "¾": Fraction(3, 4)}
_STOP_WORDS = {"is", "as", "an", "of", "or", "to", "in", "on", "at", "by", "if", "it", "no", "so", "we", "be", "do", "up", "us", "my", "me", "am"}
_MATH_OPERATORS = set("+−-–×÷*/^·±")
_OP_MAP = {"+": "+", "−": "-", "-": "-", "–": "-", "×": "*", "·": "*", "*": "*", "÷": "/", "/": "/"}
_MATH_CHARS = re.compile(r"^[A-Za-zα-ωΑ-Ω0-9.,()\[\]{}+−\-–×÷*/^·±⁰¹²³⁴⁵⁶⁷⁸⁹ⁿ₀₁₂₃₄₅₆₇₈₉½⅓⅔¼¾]+$")
_POINTS = [
    [2, 3, 5, 7, 4, 6, 8, 9], [3, 5, 2, 4, 9, 7, 6, 8], [-2, 3, -5, 4, 7, -3, 5, 2], [5, -2, 3, 8, -4, 2, 9, -6],
    [Fraction(1, 2), Fraction(2, 3), Fraction(3, 2), Fraction(5, 4), Fraction(7, 3), Fraction(3, 5), 2, 3],
    [Fraction(-3, 2), 2, Fraction(1, 3), Fraction(5, 2), -4, Fraction(7, 5), 3, Fraction(2, 7)],
]


def _math_tokens(s, base=0):
    """Tokens of an algebraic expression: ('num', Fraction, id), ('var', name), ('sup', int or name), ('op', c), ('(',),
    (')',). None when it holds anything the grammar does not read."""
    tokens, i, n, ident = [], 0, len(s), base
    while i < n:
        c = s[i]
        if c.isspace():
            i += 1
        elif c.isdigit() and c.isascii() or (c == "." and s[i + 1:i + 2].isdigit()):
            m = re.match(r"\d*\.?\d+", s[i:])
            tokens.append(("num", Fraction(m.group(0)), ident))
            ident += 1
            i += m.end()
        elif c in _VULGAR:
            tokens.append(("num", _VULGAR[c], ident))
            ident += 1
            i += 1
        elif c.isalpha():
            if s[i + 1:i + 2].isdigit() and s[i + 1:i + 2].isascii():
                return None  # b2: a square, a subscript or a product of 2; not guessed
            j = i + 1
            while j < n and s[j] in _SUB:
                j += 1
            name = c + s[i + 1:j]
            tokens.append(("var", name))
            i = j
        elif c in _SUP or c == "ⁿ":
            if c == "ⁿ":
                tokens.append(("sup", "n", None))
                i += 1
            else:
                j = i
                while j < n and s[j] in _SUP:
                    j += 1
                tokens.append(("sup", int("".join(_SUP[x] for x in s[i:j])), ident))
                ident += 1
                i = j
        elif c in _OP_MAP:
            tokens.append(("op", _OP_MAP[c]))
            i += 1
        elif c == "^":
            tokens.append(("op", "^"))
            i += 1
        elif c in "([{":
            tokens.append(("(",))
            i += 1
        elif c in ")]}":
            tokens.append((")",))
            i += 1
        else:
            return None
    return tokens


class _Syntax(ValueError):
    pass


class _ExprParser:
    """Precedence parser: + - below * / below implicit multiplication below powers."""

    def __init__(self, tokens, base=0):
        self.t, self.i, self.base = tokens, 0, base

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def starts_atom(self, tok):
        return tok is not None and (tok[0] in ("num", "var", "("))

    def parse(self):
        node = self.expr()
        if self.i != len(self.t):
            raise _Syntax("trailing")
        return node

    def expr(self):
        sign = None
        tok = self.peek()
        if tok == ("op", "-") or tok == ("op", "+"):
            sign = tok[1]
            self.i += 1
        node = self.term()
        if sign == "-":
            node = ("neg", node)
        while self.peek() in (("op", "+"), ("op", "-")):
            op = self.t[self.i][1]
            self.i += 1
            rhs = self.term()
            node = ("add" if op == "+" else "sub", node, rhs, self.base + self.i)  # the last is this sign's id
        return node

    def term(self):
        node = self.implicit()
        while self.peek() in (("op", "*"), ("op", "/")):
            op = self.t[self.i][1]
            self.i += 1
            rhs = self.implicit()
            node = ("mul" if op == "*" else "div", node, rhs)
        return node

    def implicit(self):
        if self.peek() in (("op", "-"), ("op", "+")):  # a × −b
            neg = self.t[self.i][1] == "-"
            self.i += 1
            node = self.implicit()
            return ("neg", node) if neg else node
        node = self.power()
        while self.starts_atom(self.peek()):
            if node[0] == "num" and self.peek()[0] == "num":
                raise _Syntax("two numbers")
            node = ("mul", node, self.power())
        return node

    def power(self):
        node = self.atom()
        while True:
            tok = self.peek()
            if tok is not None and tok[0] == "sup":
                self.i += 1
                exp = ("num", Fraction(tok[1]), tok[2]) if isinstance(tok[1], int) else ("var", tok[1])
                node = ("pow", node, exp)
            elif tok == ("op", "^"):
                self.i += 1
                node = ("pow", node, self.exponent())
            else:
                return node

    def exponent(self):
        tok = self.peek()
        if tok == ("op", "-"):
            self.i += 1
            return ("neg", self.exponent())
        if tok is None or tok[0] not in ("num", "var", "("):
            raise _Syntax("exponent")
        return self.atom()

    def atom(self):
        tok = self.peek()
        if tok is None:
            raise _Syntax("end")
        self.i += 1
        if tok[0] == "num":
            return ("num", tok[1], tok[2])
        if tok[0] == "var":
            return ("var", tok[1])
        if tok[0] == "(":
            node = self.expr()
            if self.peek() != (")",):
                raise _Syntax("bracket")
            self.i += 1
            return node
        raise _Syntax("atom")


def _eval_tree(node, point, override):
    kind = node[0]
    if kind == "num":
        return override.get(node[2], node[1])
    if kind == "var":
        return point[node[1]]
    if kind == "neg":
        return -_eval_tree(node[1], point, override)
    a = _eval_tree(node[1], point, override)
    b = _eval_tree(node[2], point, override)
    if kind in ("add", "sub"):
        kind = override.get(("sign", node[3]), kind)  # a candidate fix flips one sign
    if kind == "add":
        return a + b
    if kind == "sub":
        return a - b
    if kind == "mul":
        return a * b
    if kind == "div":
        return a / b  # ZeroDivisionError: the point is skipped
    if kind == "pow":
        if isinstance(b, Fraction) and b.denominator != 1 or abs(b) > 16:
            raise ValueError("exponent")
        return a**int(b)
    raise _Syntax(kind)


def _tree_vars(node, out):
    if node[0] == "var":
        out.add(node[1])
    elif node[0] not in ("num",):
        for child in node[1:]:
            if isinstance(child, tuple):
                _tree_vars(child, out)


def _tree_literals(node, out, signs=None):
    if node[0] == "num" and node[2] is not None:
        out.append((node[2], node[1]))
    elif node[0] not in ("var",):
        if signs is not None and node[0] in ("add", "sub"):
            signs.append((node[3], node[0]))
        for child in node[1:]:
            if isinstance(child, tuple):
                _tree_literals(child, out, signs)


def _agrees_at_points(left, right, names, override):
    """True/False when both sides were evaluated at 3 or more points; None when fewer points could be evaluated."""
    valid = 0
    for p in _POINTS:
        point = {name: Fraction(p[k % len(p)]) for k, name in enumerate(sorted(names))}
        try:
            a = _eval_tree(left, point, override)
            b = _eval_tree(right, point, override)
        except (ZeroDivisionError, ValueError):
            continue
        valid += 1
        if a != b:
            return False
    return True if valid >= 3 else None


def check_identity(left_text, right_text):
    """('identity' | 'not' | 'near', detail) for `left = right`, or None when it is not read (see the note above).
    'near' carries the literal change that makes it an identity."""
    for text in (left_text, right_text):
        if "±" in text:  # both signs, taken together: (50 ± b)² = 2500 ± 100b + b²
            plus = check_identity(left_text.replace("±", "+"), right_text.replace("±", "+"))
            minus = check_identity(left_text.replace("±", "−"), right_text.replace("±", "−"))
            if plus is None or minus is None:
                return None
            return plus if plus[0] != "identity" else minus
    sides = []
    for base, text in ((0, left_text), (1000, right_text)):
        tokens = _math_tokens(text, base)
        if not tokens:
            return None
        try:
            sides.append(_ExprParser(tokens, base).parse())
        except _Syntax:
            return None
    left, right = sides
    names_l, names_r = set(), set()
    _tree_vars(left, names_l)
    _tree_vars(right, names_r)
    names = names_l | names_r
    if not names_l or not names_r:
        return None  # a number on a side: an instance of the formula, not the identity
    if left[0] == "var" or right[0] == "var" or "ⁿ" in left_text + right_text:
        return None  # x = 2y + 1: a definition or a condition, not an identity
    agree = _agrees_at_points(left, right, names, {})
    if agree is None:
        return None
    if agree:
        return ("identity", "")
    literals, signs = [], []
    _tree_literals(left, literals, signs)
    _tree_literals(right, literals, signs)
    fixes = []
    for ident, kind in signs:
        flipped = "sub" if kind == "add" else "add"
        if _agrees_at_points(left, right, names, {("sign", ident): flipped}):
            fixes.append(("+" if kind == "add" else "−", "−" if kind == "add" else "+"))
    for ident, value in literals:
        if value.denominator != 1:
            continue
        v = int(value)
        for cand in sorted({v - 1, v + 1, 2 * v, 10 * v, v // 2, v // 10, 0, 1, 2, 3, 4, 5, 6, 8, 10} - {v}):
            if cand >= 0 and _agrees_at_points(left, right, names, {ident: Fraction(cand)}):
                fixes.append((v, cand))
    if fixes:
        return ("near", "; ".join(f"{a} to {b}" for a, b in dict.fromkeys(fixes)))
    return ("not", "")


def _math_like(token, letters):
    t = token.strip("*`_")
    if not t or t.startswith("["):  # a [tag] or [citation] is no part of a formula
        return False
    if t == "=" or t in _MATH_OPERATORS:
        return True
    core = t.rstrip(",.;:")
    if not core or not _MATH_CHARS.match(core):
        return False
    if re.search(r"[A-Za-zα-ωΑ-Ω]{3,}", core):
        return False
    if core.isalpha() and len(core) >= 2:
        return core.lower() not in _STOP_WORDS and set(core) <= letters
    return True


def identity_reports(text):
    """[(line, 'a = b' as written, status, detail)] for each equation with letters on both sides that the small
    algebra grammar reads; status is 'identity', 'not' (no identity, nothing close: left alone) or 'near'."""
    reports = []
    rows = []  # (token, line)
    fence = False
    for no, line in enumerate(blank_comments(text).splitlines(), 1):
        if line.strip().startswith("```"):
            fence = not fence
            continue
        if fence or line.strip().startswith("|"):
            continue
        line = re.sub(r"^\s*(?:[-*+]|\d+[.)])\s+", " ", line)  # a list marker
        line = line.replace("**", " ").replace("`", " ")
        line = re.sub(r"\s*=\s*", " = ", line)
        for tok in line.split():
            rows.append((tok, no))
    toks = [t for t, _ in rows]
    single = set()  # the letters used as symbols: standalone, or inside a token with brackets, digits or operators
    for t in toks:
        core = t.strip("()[]{}.,;:")
        if t.startswith("[") or re.search(r"[A-Za-zα-ωΑ-Ω]{3,}", core):
            continue
        if len(core) == 1 and core.isalpha():
            single.add(core)
        elif not core.isalpha():
            single.update(ch for ch in core if ch.isalpha())
    i = 0
    while i < len(toks):
        if not _math_like(toks[i], single):
            i += 1
            continue
        j = i
        while j < len(toks):
            if not _math_like(toks[j], single):
                break
            j += 1
            if toks[j - 1].endswith((",", ";", ":", ".")) and len(toks[j - 1]) > 1:
                break  # a comma or a full stop ends the run (`..., and`)
        run = toks[i:j]
        before = toks[i - 1] if i > 0 else ""
        after = toks[j] if j < len(toks) else ""
        # what stands beside the run must not be a piece of the formula the grammar failed to read (`log₂`, `Σ`)
        plain = lambda w: (not w) or w.startswith("[") or not re.search(r"[0-9+−×÷*/^=²³⁴ⁿ₀₁₂₃₄₅₆₇₈₉…⋯]|\.\.\.", w)  # noqa: E731
        stops = (",", ";", ":", ".")
        cut_before = not (plain(before) or before.endswith(stops))  # the first side may be the tail of something longer
        cut_after = not (plain(after) or run[-1].endswith(stops))  # and the last the head of something longer
        if "=" in run:
            joined = " ".join(run).rstrip(",.;:")
            sides = [s.strip() for s in joined.split("=")]
            if all(sides):
                for k, (a, b) in enumerate(zip(sides, sides[1:])):
                    if (k == 0 and cut_before) or (k == len(sides) - 2 and cut_after):
                        continue
                    result = check_identity(a, b)
                    if result is not None:
                        reports.append((rows[i][1], f"{a} = {b}", result[0], result[1]))
        i = max(j, i + 1)
    return reports


def identity_problems(text):
    """(line, message) for each equation with letters that is one number away from an identity."""
    return [
        (no, f"{eq} is no identity, but it would be with one number changed ({detail}): a typo in a formula")
        for no, eq, status, detail in identity_reports(text)
        if status == "near"
    ]


def extract(text):
    """(numbers to check, exempt count by reason) from a description's text."""
    numbers, exempt = [], {}
    in_comment, in_sources = False, False
    uids = unit_ids(text)
    for no, raw in enumerate(text.splitlines(), 1):
        line = raw
        if in_comment:
            if "-->" in line:
                in_comment, line = False, line.split("-->", 1)[1]
            else:
                continue
        while "<!--" in line:
            head, _, rest = line.partition("<!--")
            if "-->" in rest:
                line = head + " " + rest.split("-->", 1)[1]
            else:
                in_comment, line = True, head
        if line.startswith("#"):
            in_sources = bool(SOURCES.match(line))
        clean = line
        for reason, pattern in (
            ("url", URL),
            ("key", FACT_KEY),
            ("beat", BEAT_ID),
            ("chapter", CHAPTER),
            ("index", INDEX_LABEL),
            ("section", SECTION),
            ("volume", VOLUME),
            ("timestamp", TIMESTAMP),
            ("date", ISO_DATE),
            ("list", LIST_MARK),
        ):
            n = len(pattern.findall(clean))
            if n:
                exempt[reason] = exempt.get(reason, 0) + n
                clean = blank(pattern, clean)
        found = []
        for m in BARE_POWER.finditer(clean):
            if clean[: m.start()].rstrip()[-1:] in ("×", "x") or (m.start() > 0 and clean[m.start() - 1].isalnum()):
                continue  # the power of "3.4 × 10⁻⁵", which the number reads with its mantissa
            exp = int("".join(SUPER[c] for c in m.group("sup"))) if m.group("sup") else int(m.group("exp"))
            power = Number(f"1e{exp}", Decimal(1), 0, exp, False, no, line.strip()[:160])
            power.unit = uids[no - 1] if no <= len(uids) else 0
            found.append((m.start(), power))
            clean = clean[: m.start()] + " " * (m.end() - m.start()) + clean[m.end():]
        for m in NUMBER.finditer(clean):
            start, end = m.span()
            if start > 0 and (clean[start - 1].isalpha() or clean[start - 1] in "_^"):
                continue  # an identifier (x264, f32, A_2) or an exponent
            tail = re.match(r"[A-Za-z]+", clean[end:])
            if tail and len(tail.group(0)) > 3:
                continue  # a name (3Blue1Brown)
            if in_sources:
                exempt["sources"] = exempt.get("sources", 0) + 1
                continue
            whole = m.group("int").replace(",", "")
            frac = m.group("frac") or ""
            shift = 0
            if m.group("sup"):
                shift = int("".join(SUPER[c] for c in m.group("sup")))
            elif m.group("exp"):
                shift = int(m.group("exp"))
            if m.group("unit"):
                shift += UNITS[m.group("unit")]
            num = Number(
                whole + ("." + frac if frac else "") + (f"e{shift}" if m.group("sup") or m.group("exp") else ""),
                Decimal(whole + ("." + frac if frac else "")),
                len(frac),
                shift,
                bool(m.group("pct")),
                no,
                line.strip()[:160],
            )
            num.unit = uids[no - 1] if no <= len(uids) else 0
            num.formula = not num.strong and (num.mantissa in (0, 1) or is_formula(clean, start, end))
            found.append((start, num))
        numbers.extend(n for _, n in sorted(found, key=lambda t: t[0]))
    return numbers, exempt


# ------------------------------------------------------------------ the allow-list

class AllowList(list):
    """The rows of docs/description_numbers.tsv. `legacy` is set by a line `#! legacy ...` in it: the video's
    numbers are still matched against any fact of the oracle, not bound to the facts their text names."""

    legacy = False


def read_allow(path):
    rows = AllowList()
    if not path.is_file():
        return rows
    for no, raw in enumerate(path.read_text().splitlines(), 1):
        if raw.startswith("#! legacy"):
            rows.legacy = True
        if not raw.strip() or raw.startswith("#"):
            continue
        cells = raw.split("\t")
        if len(cells) < 4 or cells[2] not in KINDS or not cells[3].strip():
            sys.exit(f"{path.name}:{no}: needs 'number TAB context TAB kind TAB reason' with kind in {sorted(KINDS)}")
        rows.append({"no": no, "number": cells[0].strip(), "context": cells[1], "kind": cells[2], "reason": cells[3].strip(), "used": 0})
    return rows


class Evaluator(ast.NodeVisitor):
    """`=expression` over facts: numbers, + - * / **, f(key[, i]), n_facts(), abs, sqrt, hypot, log, log10, exp,
    sin, cos, asin, acos, atan, atan2, degrees, radians, floor, ceil, round, pi."""

    def __init__(self, facts):
        self.facts = facts

    def run(self, expr):
        return self.visit(ast.parse(expr, mode="eval").body)

    def visit_Constant(self, n):
        if isinstance(n.value, (int, float)):
            return float(n.value)
        raise ValueError("only numbers and fact keys")

    def visit_Name(self, n):
        if n.id == "pi":
            return math.pi
        raise ValueError(f"unknown name {n.id}")

    def visit_UnaryOp(self, n):
        v = self.visit(n.operand)
        if isinstance(n.op, ast.USub):
            return -v
        if isinstance(n.op, ast.UAdd):
            return v
        raise ValueError("unary operator")

    def visit_BinOp(self, n):
        a, b = self.visit(n.left), self.visit(n.right)
        ops = {ast.Add: lambda: a + b, ast.Sub: lambda: a - b, ast.Mult: lambda: a * b, ast.Div: lambda: a / b, ast.Pow: lambda: a**b}
        if type(n.op) not in ops:
            raise ValueError("operator")
        return ops[type(n.op)]()

    def visit_Call(self, n):
        name = n.func.id if isinstance(n.func, ast.Name) else ""
        if name == "f":
            args = [a.value for a in n.args if isinstance(a, ast.Constant)]
            key, idx = args[0], (args[1] if len(args) > 1 else 0)
            if key not in self.facts:
                raise ValueError(f"no fact {key}")
            return float(FACT_NUM.findall(self.facts[key])[idx])
        if name == "n_facts":
            return float(len(self.facts))
        fns = {"abs": abs, "sqrt": math.sqrt, "log": math.log, "log10": math.log10, "exp": math.exp, "hypot": math.hypot,
               "degrees": math.degrees, "radians": math.radians, "sin": math.sin, "cos": math.cos, "asin": math.asin,
               "acos": math.acos, "atan": math.atan, "atan2": math.atan2, "round": round, "floor": math.floor, "ceil": math.ceil}
        if name in fns:
            return fns[name](*[self.visit(a) for a in n.args])
        raise ValueError(f"unknown function {name}")


# ------------------------------------------------------------------ the check

def check(description, oracle_text, allow_rows, verbose=False):
    """-> (report lines, problems, counts)."""
    numbers, exempt = extract(description)
    facts = fact_numbers(oracle_text)
    values = fact_values(oracle_text)
    by_key = {}
    for fv, key in facts:
        by_key.setdefault(key, []).append(fv)
    ev = Evaluator(values)
    counts = {"numbers": len(numbers), "fact": 0, "weak": 0, "bound": 0, "free": 0, "formula": 0, "allowed": 0, "unmatched": 0, "exempt": exempt}
    report, problems = [], []
    keys = named_keys(description)
    counts["keys"] = len(keys)
    for no, key in keys:
        if not key_exists(key, values):
            problems.append(f"line {no}: {key} is named but is no key of the oracle")
    for no, message in arithmetic_problems(description) + identity_problems(description):
        problems.append(f"line {no}: {message}")
    _, unit_tag = unit_tags(description)
    scope = {}  # unit id -> the oracle keys its tags stand for
    for uid, tags in unit_tag.items():
        found = {k for t in tags for k in resolve_tag(t, values)}
        if found:
            scope[uid] = found
    legacy = getattr(allow_rows, "legacy", False)
    counts["legacy"] = legacy
    for num in numbers:
        elsewhere = list(dict.fromkeys(key for fv, key in facts if agrees_memo(num, fv)))
        if legacy and elsewhere:
            counts["fact" if num.strong else "weak"] += 1
            counts["free"] += 1
            if verbose:
                report.append(f"  line {num.line}: {num} = {', '.join(elsewhere[:4])}")
            continue
        if num.formula and not legacy:
            counts["formula"] += 1
            continue
        tagged = None if legacy else scope.get(num.unit)
        if tagged:
            bound = [k for k in sorted(tagged) if any(agrees_memo(num, fv) for fv in by_key.get(k, []))]
            if bound:
                counts["bound"] += 1
                counts["fact" if num.strong else "weak"] += 1
                if verbose:
                    report.append(f"  line {num.line}: {num} = {', '.join(bound[:4])} (bound)")
                continue
        row = next((r for r in allow_rows if r["number"] == str(num).replace("%", "") and r["context"] in num.context), None)
        if row is not None and row["kind"] == "derived":
            try:
                ok = agrees_float(num, ev.run(row["reason"].lstrip("=")))
            except (ValueError, SyntaxError, ZeroDivisionError, IndexError, TypeError) as e:
                ok = False
                problems.append(f"description_numbers.tsv:{row['no']}: the expression {row['reason']!r} fails ({e})")
            if not ok:
                row = None
                problems.append(f"line {num.line}: {num} is allow-listed as derived but {row_expr(allow_rows, num)} does not give it")
        if row is not None:
            row["used"] += 1
            counts["allowed"] += 1
            continue
        counts["unmatched"] += 1
        where = f"line {num.line}: {num}"
        if tagged:
            names = ", ".join(sorted({t if isinstance(t, str) else "".join(t) for t in unit_tag[num.unit]})[:4])
            also = f"; it equals {', '.join(elsewhere[:3])}, which the text does not name" if elsewhere else ""
            problems.append(f"{where} agrees with none of the facts its text names ({names}){also}: {num.context}")
        elif elsewhere:
            problems.append(f"{where} only equals {', '.join(elsewhere[:3])}; tag its text with the key it is about or give it a row: {num.context}")
        else:
            problems.append(f"{where} is not a fact: {num.context}")
    for r in allow_rows:
        if not r["used"]:
            problems.append(f"description_numbers.tsv:{r['no']}: {r['number']} ({r['context']!r}) is used by nothing")
    return report, problems, counts


def row_expr(rows, num):
    r = next((r for r in rows if r["number"] == str(num).replace("%", "") and r["context"] in num.context), None)
    return r["reason"] if r else "its expression"


def oracle_output(crate):
    """The oracle's stdout, run as facts-check runs it (numpy, and what its `# oracle-packages:` line names)."""
    import subprocess

    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    import facts_check

    oracle = crate / "oracle.py"
    run = subprocess.run(
        ["uv", "run", "--quiet", "--no-project", *(a for n in facts_check.oracle_packages(oracle) for a in ("--with", n)), "python3", "-I", "-B", "oracle.py"],
        cwd=crate, capture_output=True, text=True, timeout=facts_check.ORACLE_TIMEOUT_S,
    )
    if run.returncode != 0:
        sys.stderr.write(run.stderr[-4000:])
        sys.exit(f"description-numbers: oracle.py failed ({run.returncode})")
    return run.stdout


def main(argv):
    args = [a for a in argv if not a.startswith("-")]
    opts = {}
    it = iter(argv)
    for a in it:
        if a in ("--oracle", "--facts"):
            opts[a] = next(it, None)
            if opts[a] in args:
                args.remove(opts[a])
    if len(args) != 1 or any(a.startswith("-") and a not in ("--oracle", "--facts", "-v") for a in argv):
        sys.exit(__doc__)
    slug = args[0]
    crate = ROOT / "explainer_corpus" / slug
    description = crate / "docs" / "description.md"
    if not description.is_file():
        sys.exit(f"description-numbers: no {description}")
    oracle = pathlib.Path(opts["--oracle"]).read_text() if opts.get("--oracle") else oracle_output(crate)
    if opts.get("--facts"):
        oracle += "\n" + pathlib.Path(opts["--facts"]).read_text()
    rows = read_allow(crate / "docs" / "description_numbers.tsv")
    report, problems, c = check(description.read_text(), oracle, rows, "-v" in argv)
    for line in report:
        print(line)
    for p in problems:
        print("MISMATCH " + p)
    exempt = ", ".join(f"{n} {r}" for r, n in sorted(c["exempt"].items())) or "none"
    legacy = " LEGACY (not tag-bound: a number only has to equal some fact of the oracle; see docs/description_numbers.tsv)" if c["legacy"] else ""
    print(
        f"description-numbers {slug}:{' LEGACY (not tag-bound)' if c['legacy'] else ''} {c['numbers']} numbers: {c['bound']} bound to the facts their text names, {str(c['free']) + ' equal some fact of the oracle, ' if c['legacy'] else ''}"
        f"{c['formula']} formula digits, {c['allowed']} allow-listed, {c['unmatched']} unmatched; not read: {exempt}: {'PASS' if not problems else f'{len(problems)} FAIL'}{legacy}"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
