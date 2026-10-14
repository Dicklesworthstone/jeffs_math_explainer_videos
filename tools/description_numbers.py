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
    evaluated here and required to give the number: `=f("F-C.mvl.size")/256`; `f(key, i)` is the i-th
    number of a value, `n_facts()` the count of keys). A row nothing uses is an error, so the list cannot
    go stale. A small whole number (under 100, no point) must equal a fact's number exactly; any other may
    be a rounding of one.

Not read as numbers: HTML comments, the fact keys (G-A.joints), the beat ids (01_6), "chapter 13",
timestamps (12:34), list markers (1.), URLs and DOIs, ISO dates, and every line under a Sources or
References heading (bibliographic volumes, pages and years belong to the source, not to the facts).
Signs are not checked (an unsigned match is a match): a claim about a sign needs its own test.
Spelled-out numbers are not read either.

Exit 0 when every number is accounted for, 1 when any is not (each is printed with its line), 2 on usage.
"""

import ast
import math
import pathlib
import re
import sys
from decimal import ROUND_DOWN, ROUND_HALF_EVEN, ROUND_HALF_UP, Decimal, InvalidOperation, getcontext

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
getcontext().prec = 80

FACT_NUM = re.compile(r"[-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?")
FACT_KEY = re.compile(r"\b[A-Z]-[A-Z0-9]\.[\w.*\-]*[\w*]")
BEAT_ID = re.compile(r"\b\d\d_\d+\b")
CHAPTER = re.compile(r"\b[Cc]hapters?\s+\d+(?:\s*(?:[–-]|and|to)\s*\d+)?")
TIMESTAMP = re.compile(r"\b\d{1,2}:\d{2}(?::\d{2})?\b")
ISO_DATE = re.compile(r"\b\d{4}-\d{2}-\d{2}\b")
URL = re.compile(r"https?://\S+|\bdoi:\s*\S+|\bDOI\s+\S+|\barXiv:\s*\S+", re.I)
LIST_MARK = re.compile(r"^\s*\d+[.)]\s")
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


class Number:
    """A number as the description writes it."""

    def __init__(self, text, mantissa, decimals, shift, percent, line, context):
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
    """Does x round (half up, half even or by truncation) to `want` at `places` decimals?"""
    try:
        q = Decimal(1).scaleb(-places)
        return any(x.quantize(q, rounding=mode) == want for mode in (ROUND_HALF_UP, ROUND_HALF_EVEN, ROUND_DOWN))
    except InvalidOperation:
        return False


def agrees(num, fact):
    """Does the description's number say this fact value (rounded as it likes, never changed)?"""
    fact = abs(fact)
    if not num.strong:
        return fact == abs(num.mantissa)  # a small whole number is no rounding of anything: it must BE a fact's number
    pool = [fact, fact * 100] if num.percent else [fact]
    return any(quantize_equal(f.scaleb(-num.shift), abs(num.mantissa), num.decimals) for f in pool)


def agrees_float(num, x):
    try:
        return agrees(num, Decimal(repr(abs(float(x)))))
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


def extract(text):
    """(numbers to check, exempt count by reason) from a description's text."""
    numbers, exempt = [], {}
    in_comment, in_sources = False, False
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
            found.append((m.start(), Number(f"1e{exp}", Decimal(1), 0, exp, False, no, line.strip()[:160])))
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
            found.append(
                (
                    start,
                    Number(
                        whole + ("." + frac if frac else "") + (f"e{shift}" if m.group("sup") or m.group("exp") else ""),
                        Decimal(whole + ("." + frac if frac else "")),
                        len(frac),
                        shift,
                        bool(m.group("pct")),
                        no,
                        line.strip()[:160],
                    ),
                )
            )
        numbers.extend(n for _, n in sorted(found, key=lambda t: t[0]))
    return numbers, exempt


# ------------------------------------------------------------------ the allow-list

def read_allow(path):
    rows = []
    if not path.is_file():
        return rows
    for no, raw in enumerate(path.read_text().splitlines(), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        cells = raw.split("\t")
        if len(cells) < 4 or cells[2] not in KINDS or not cells[3].strip():
            sys.exit(f"{path.name}:{no}: needs 'number TAB context TAB kind TAB reason' with kind in {sorted(KINDS)}")
        rows.append({"no": no, "number": cells[0].strip(), "context": cells[1], "kind": cells[2], "reason": cells[3].strip(), "used": 0})
    return rows


class Evaluator(ast.NodeVisitor):
    """`=expression` over facts: numbers, + - * / **, f(key[, i]), n_facts(), abs, sqrt, log, log10, exp, pi."""

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
        fns = {"abs": abs, "sqrt": math.sqrt, "log": math.log, "log10": math.log10, "exp": math.exp}
        if name in fns:
            return fns[name](*[self.visit(a) for a in n.args])
        raise ValueError(f"unknown function {name}")


# ------------------------------------------------------------------ the check

def check(description, oracle_text, allow_rows, verbose=False):
    """-> (report lines, problems, counts)."""
    numbers, exempt = extract(description)
    facts = fact_numbers(oracle_text)
    values = fact_values(oracle_text)
    ev = Evaluator(values)
    counts = {"numbers": len(numbers), "fact": 0, "weak": 0, "allowed": 0, "unmatched": 0, "exempt": exempt}
    report, problems = [], []
    for num in numbers:
        hits = []
        for fv, key in facts:
            if agrees(num, fv):
                hits.append(key)
        if hits:
            counts["fact" if num.strong else "weak"] += 1
            if verbose:
                report.append(f"  line {num.line}: {num} = {', '.join(dict.fromkeys(hits[:4]))}")
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
        problems.append(f"line {num.line}: {num} is not a fact: {num.context}")
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
    print(
        f"description-numbers {slug}: {c['numbers']} numbers: {c['fact']} are facts, {c['weak']} small whole numbers equal a fact, "
        f"{c['allowed']} allow-listed, {c['unmatched']} unmatched; not read: {exempt}: {'PASS' if not problems else f'{len(problems)} FAIL'}"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
