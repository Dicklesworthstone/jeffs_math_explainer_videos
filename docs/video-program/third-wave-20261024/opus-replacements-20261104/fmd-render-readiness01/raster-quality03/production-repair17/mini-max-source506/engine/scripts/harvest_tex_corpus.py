#!/usr/bin/env python3
"""G0-4 corpus harvest (fm-or4, plan §11.5 / §20.1 spike 4).

Extracts the TeX-string multiset from the two pinned sources:

  1. the 3b1b/videos tree      -> scripts/videos_ref   (pin: SUITE.lock [reference])
  2. the Reference's manimlib  -> scripts/manim_ref    (pin: SUITE.lock [reference])

and produces the ranked construct table that defines fmd-math tier 1 and
the fixed denominator of the public coverage ratchet.

Method (static extraction; the counting rules live in
docs/g0/G0-4-corpus-harvest.md and are normative there):

  * Every ``*.py`` file in both trees is parsed with ``ast``.
  * Call sites of the four Tex-family constructors at the videos pin
    (``Tex``, ``OldTex``, ``TexText``, ``OldTexText``) plus the internal
    ``SingleStringTex`` are collected.
  * Fully-literal positional string arguments enter the corpus multiset,
    one occurrence per call-site argument.  ``Tex``/``OldTex`` arguments
    are math mode; ``TexText``/``OldTexText`` arguments are text mode
    (their ``$...$`` islands are lexed as math).
  * Non-literal arguments (f-strings, concatenations, variables, ``*``)
    are tallied as *dynamic* and excluded from the denominator; string
    constants nested inside a dynamic argument (f-string fragments, the
    pieces of a ``"".join(...)``) feed an advisory construct tally only —
    a fragment is not a complete formula and must not enter the
    parseable-string denominator.
  * Reference preset strings: a string default value of any function
    parameter whose name contains ``tex`` (e.g. ``Brace.__init__``'s
    ``tex_string=R"\\underbrace{\\qquad}"``) enters the corpus as a
    math-mode occurrence.
  * Each corpus string is lexed into constructs: control words
    (``\\frac``), control symbols (``\\\\``, ``\\,``), environments
    (``env:matrix``), scripts (``script:sup``/``script:sub``), primes,
    alignment tabs, ties, and non-ASCII characters (``char:U+2192``).

Outputs:

  corpus/tex_corpus.jsonl              PRIVATE (gitignored): the strings
                                       themselves with counts + provenance;
                                       CC BY-NC-SA fixture policy, plan §15.3.
  docs/g0/g0-4-corpus/harvest_manifest.json   committed: pins, rule version,
                                       aggregate counts, corpus content hash.
  docs/g0/g0-4-corpus/denominator.tsv  committed: sha256 + mode + count per
                                       distinct string (verifiable, private).
  docs/g0/g0-4-corpus/construct_table.tsv     committed: the ranked table
                                       with the tier assignment.

Deterministic: byte-identical outputs for identical inputs (sorted
iteration everywhere, no timestamps, fixed JSON formatting).

Usage:  python3 scripts/harvest_tex_corpus.py  (from anywhere; paths are
repo-relative to this file).
"""

from __future__ import annotations

import ast
import hashlib
import json
import re
import subprocess
import sys
import warnings
from collections import Counter, defaultdict
from pathlib import Path

# The scanned trees are full of TeX-in-non-raw-strings; CPython warns on
# every `\o`-style escape while *we* compile them.  Not our defect.
warnings.filterwarnings("ignore", category=SyntaxWarning)

RULES_VERSION = "1"  # bump when extraction or lexing rules change

REPO = Path(__file__).resolve().parent.parent
VIDEOS_REF = REPO / "scripts" / "videos_ref"
MANIM_REF = REPO / "scripts" / "manim_ref"

PRIVATE_OUT = REPO / "corpus" / "tex_corpus.jsonl"
PUBLIC_DIR = REPO / "docs" / "g0" / "g0-4-corpus"

MATH_CALLS = {"Tex", "OldTex", "SingleStringTex"}
TEXT_CALLS = {"TexText", "OldTexText"}
ALL_CALLS = MATH_CALLS | TEXT_CALLS

# ---------------------------------------------------------------------------
# The §11.4 language surface, encoded as construct names.  Seed set for the
# tier-1 cut: every construct here is T1 by declaration; the empirical pass
# then extends T1 by occurrence rank until the coverage target is met.
# ---------------------------------------------------------------------------

GREEK = """alpha beta gamma delta epsilon varepsilon zeta eta theta vartheta
iota kappa lambda mu nu xi pi varpi rho varrho sigma varsigma tau upsilon
phi varphi chi psi omega Gamma Delta Theta Lambda Xi Pi Sigma Upsilon Phi
Psi Omega""".split()

OPERATOR_NAMES = """sin cos tan cot sec csc arcsin arccos arctan sinh cosh
tanh coth exp log ln lg det dim ker deg arg gcd hom inf sup lim liminf
limsup max min Pr mod pmod bmod""".split()

BIG_OPERATORS = """sum prod coprod int oint iint iiint idotsint bigcup
bigcap bigsqcup bigvee bigwedge bigodot bigotimes bigoplus biguplus""".split()

ACCENTS = """hat vec dot ddot tilde bar breve check acute grave mathring
widehat widetilde overline underline overbrace underbrace overrightarrow
overleftarrow""".split()

DELIMS = """left right big Big bigg Bigg bigl bigr Bigl Bigr biggl biggr
Biggl Biggr langle rangle lceil rceil lfloor rfloor lbrace rbrace lvert
rvert lVert rVert vert Vert backslash""".split()

STYLES = """mathbb mathcal mathrm mathbf mathsf mathtt mathit mathscr
mathfrak boldsymbol bm text textbf textit textrm texttt emph bf it rm sf tt
cal frak scr displaystyle textstyle scriptstyle scriptscriptstyle""".split()

SPACING = """quad qquad thinspace negthinspace enspace hspace vspace phantom
vphantom hphantom smash""".split()

SYMBOLS = """cdot cdots ldots dots dotsc dotsb vdots ddots hdots times div
pm mp ast star circ bullet cap cup uplus sqcap sqcup vee wedge setminus
wr diamond bigtriangleup bigtriangledown triangleleft triangleright oplus
ominus otimes oslash odot bigcirc dagger ddagger amalg leq geq le ge ll gg
equiv sim simeq asymp approx cong neq ne doteq propto models perp mid
parallel subset supset subseteq supseteq sqsubseteq sqsupseteq in ni notin
vdash dashv smile frown leftarrow rightarrow to leftrightarrow Leftarrow
Rightarrow Leftrightarrow mapsto hookleftarrow hookrightarrow rightharpoonup
rightharpoondown leftharpoonup leftharpoondown longleftarrow longrightarrow
longleftrightarrow Longleftarrow Longrightarrow Longleftrightarrow
longmapsto uparrow downarrow updownarrow Uparrow Downarrow Updownarrow
nearrow searrow swarrow nwarrow aleph hbar imath jmath ell wp Re Im mho
prime emptyset varnothing nabla surd top bot angle triangle forall exists
neg lnot flat natural sharp clubsuit diamondsuit heartsuit spadesuit
partial infty Box cdotp colon implies iff land lor because therefore
subsetneq supsetneq geqslant leqslant""".split()

CONSTRUCTS_11_4 = (
    ["\\" + w for w in (
        GREEK + OPERATOR_NAMES + BIG_OPERATORS + ACCENTS + DELIMS + STYLES
        + SPACING + SYMBOLS
        + ["frac", "dfrac", "tfrac", "binom", "over", "sqrt", "stackrel",
           "overset", "underset", "operatorname", "limits", "nolimits",
           "substack",  # NOTE: §11.5 names \substack as canonical T2; kept
                        # out of the seed set — it earns its tier by rank.
           "newcommand", "not", "middle", "color", "textcolor", "mathstrut",
           "strut", "atop", "choose", "label", "ref"]
    )]
    + ["\\" + s for s in ["\\", ",", ";", ":", "!", " ", "{", "}", "$", "%",
                          "&", "#", "_", "^", "|"]]
    + ["env:" + e for e in ["matrix", "pmatrix", "bmatrix", "Bmatrix",
                            "vmatrix", "Vmatrix", "smallmatrix", "cases",
                            "array", "align", "align*", "aligned"]]
    + ["script:sup", "script:sub", "prime", "alignment-tab", "tie",
       "math-island"]
)
# \substack is the plan's canonical T2 example; exclude from the seed.
T1_SEED = frozenset(c for c in CONSTRUCTS_11_4 if c != "\\substack")

# ---------------------------------------------------------------------------
# Extraction
# ---------------------------------------------------------------------------


def call_name(node: ast.Call) -> str | None:
    f = node.func
    if isinstance(f, ast.Name):
        return f.id
    if isinstance(f, ast.Attribute):
        return f.attr
    return None


class Harvest:
    def __init__(self) -> None:
        # (mode, text) -> count
        self.corpus: Counter[tuple[str, str]] = Counter()
        # (mode, text) -> sorted set of "tree:relpath:line"
        self.sources: defaultdict[tuple[str, str], set[str]] = defaultdict(set)
        self.call_sites: Counter[str] = Counter()
        # top-level dir ("videos_ref/_2019", "manim_ref/manimlib") -> literal-arg count
        self.per_dir: Counter[str] = Counter()
        self.dynamic_args: Counter[str] = Counter()
        self.fragments: Counter[tuple[str, str]] = Counter()
        self.files_scanned = 0
        self.files_failed: list[str] = []

    def scan_tree(self, root: Path, tree_label: str) -> None:
        for path in sorted(root.rglob("*.py")):
            if ".git" in path.parts:
                continue
            rel = f"{tree_label}/{path.relative_to(root)}"
            try:
                src = path.read_text(encoding="utf-8", errors="replace")
                mod = ast.parse(src)
            except SyntaxError:
                self.files_failed.append(rel)
                continue
            self.files_scanned += 1
            for node in ast.walk(mod):
                if isinstance(node, ast.Call):
                    name = call_name(node)
                    if name in ALL_CALLS:
                        self._take_call(node, name, rel)
                elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    self._take_tex_defaults(node, rel)

    def _take_tex_defaults(self, fn: ast.FunctionDef, rel: str) -> None:
        """Preset strings: str defaults of parameters named *tex*."""
        a = fn.args
        for params, defaults in (
            (a.posonlyargs + a.args, a.defaults),
            (a.kwonlyargs, a.kw_defaults),
        ):
            for param, default in zip(params[len(params) - len(defaults):],
                                      defaults):
                if (
                    default is not None
                    and "tex" in param.arg.lower()
                    and isinstance(default, ast.Constant)
                    and isinstance(default.value, str)
                    and default.value
                ):
                    key = ("math", default.value)
                    self.corpus[key] += 1
                    self.sources[key].add(f"{rel}:{default.lineno}")
                    self.per_dir["/".join(rel.split("/")[:2])] += 1

    def _take_call(self, node: ast.Call, name: str, rel: str) -> None:
        self.call_sites[name] += 1
        mode = "math" if name in MATH_CALLS else "text"
        for arg in node.args:
            if isinstance(arg, ast.Starred):
                self.dynamic_args[name] += 1
            elif isinstance(arg, ast.Constant) and isinstance(arg.value, str):
                key = (mode, arg.value)
                self.corpus[key] += 1
                self.sources[key].add(f"{rel}:{arg.lineno}")
                self.per_dir["/".join(rel.split("/")[:2])] += 1
            else:
                self.dynamic_args[name] += 1
                for sub in ast.walk(arg):
                    if isinstance(sub, ast.Constant) and isinstance(
                        sub.value, str
                    ):
                        self.fragments[(mode, sub.value)] += 1


# ---------------------------------------------------------------------------
# Construct lexing.  A TeX-lexer-level pass, not a parser: rank needs token
# identity, not tree shape.  Math mode counts scripts and primes; text mode
# defers its $...$ islands to the math lexer.
# ---------------------------------------------------------------------------

CONTROL_WORD = re.compile(r"\\([a-zA-Z]+)")


def lex_math(s: str, out: Counter[str]) -> None:
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == "\\":
            m = CONTROL_WORD.match(s, i)
            if m:
                word = m.group(1)
                i = m.end()
                if word in ("begin", "end"):
                    em = re.match(r"\s*\{([^{}]*)\}", s[i:])
                    if em:
                        if word == "begin":
                            out[f"env:{em.group(1).strip()}"] += 1
                        i += em.end()
                else:
                    out["\\" + word] += 1
            elif i + 1 < n:
                out["\\" + s[i + 1]] += 1
                i += 2
            else:
                i += 1  # trailing lone backslash: not a construct
        elif c == "^":
            out["script:sup"] += 1
            i += 1
        elif c == "_":
            out["script:sub"] += 1
            i += 1
        elif c == "'":
            out["prime"] += 1
            i += 1
        elif c == "&":
            out["alignment-tab"] += 1
            i += 1
        elif c == "~":
            out["tie"] += 1
            i += 1
        elif c == "%":
            while i < n and s[i] != "\n":
                i += 1
        elif ord(c) > 127:
            out[f"char:U+{ord(c):04X}"] += 1
            i += 1
        else:
            i += 1


def lex_text(s: str, out: Counter[str]) -> None:
    """Text mode: islands between unescaped ``$`` are math; the mainland
    counts control sequences, environments, ties, and non-ASCII, but not
    scripts or primes (an apostrophe in prose is not a construct)."""
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == "\\":
            m = CONTROL_WORD.match(s, i)
            if m:
                word = m.group(1)
                i = m.end()
                if word in ("begin", "end"):
                    em = re.match(r"\s*\{([^{}]*)\}", s[i:])
                    if em:
                        if word == "begin":
                            out[f"env:{em.group(1).strip()}"] += 1
                        i += em.end()
                else:
                    out["\\" + word] += 1
            elif i + 1 < n:
                out["\\" + s[i + 1]] += 1
                i += 2
            else:
                i += 1
        elif c == "$":
            j = i + 1
            while j < n:
                if s[j] == "\\":
                    j += 2
                    continue
                if s[j] == "$":
                    break
                j += 1
            island = s[i + 1 : min(j, n)]
            out["math-island"] += 1
            lex_math(island, out)
            i = min(j, n) + 1
        elif c == "~":
            out["tie"] += 1
            i += 1
        elif c == "%":
            while i < n and s[i] != "\n":
                i += 1
        elif ord(c) > 127:
            out[f"char:U+{ord(c):04X}"] += 1
            i += 1
        else:
            i += 1


def lex(mode: str, s: str) -> Counter[str]:
    out: Counter[str] = Counter()
    (lex_math if mode == "math" else lex_text)(s, out)
    return out


# ---------------------------------------------------------------------------
# Ranking + the tier-1 cut
# ---------------------------------------------------------------------------


def kind_of(construct: str) -> str:
    if construct.startswith("env:"):
        return "environment"
    if construct.startswith("char:"):
        return "unicode-char"
    if construct.startswith("script:") or construct in (
        "prime",
        "alignment-tab",
        "tie",
        "math-island",
    ):
        return "structural"
    if len(construct) == 2 and not construct[1].isalpha():
        return "control-symbol"
    return "control-word"


def main() -> int:
    for root, label in ((VIDEOS_REF, "videos_ref"), (MANIM_REF, "manim_ref")):
        if not root.is_dir():
            print(f"error: {root} missing — clone per SUITE.lock [reference]",
                  file=sys.stderr)
            return 1

    pins = {}
    for root, label in ((VIDEOS_REF, "3b1b/videos"), (MANIM_REF, "3b1b/manim")):
        pins[label] = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True, timeout=30,
        ).stdout.strip()

    h = Harvest()
    h.scan_tree(VIDEOS_REF, "videos_ref")
    h.scan_tree(MANIM_REF / "manimlib", "manim_ref/manimlib")

    # ---- per-string records, lexed ------------------------------------
    records = []
    construct_weight: Counter[str] = Counter()          # occurrence-weighted strings
    construct_uses: Counter[str] = Counter()            # occurrence-weighted uses
    construct_distinct: Counter[str] = Counter()        # distinct strings
    string_constructs: dict[tuple[str, str], set[str]] = {}

    for (mode, text), count in sorted(h.corpus.items()):
        toks = lex(mode, text)
        string_constructs[(mode, text)] = set(toks)
        for c, uses in toks.items():
            construct_weight[c] += count
            construct_uses[c] += count * uses
            construct_distinct[c] += 1
        records.append(
            {
                "sha256": hashlib.sha256(
                    f"{mode}\x00{text}".encode()
                ).hexdigest(),
                "mode": mode,
                "count": count,
                "constructs": sorted(toks),
                "sources": sorted(h.sources[(mode, text)]),
                "text": text,
            }
        )

    # advisory: constructs seen inside dynamic-argument string fragments
    advisory: Counter[str] = Counter()
    for (mode, frag), count in sorted(h.fragments.items()):
        for c, uses in lex(mode, frag).items():
            advisory[c] += count * uses

    total_occ = sum(h.corpus.values())
    distinct = len(h.corpus)

    # ---- the tier-1 cut ----------------------------------------------
    # T1 = §11.4 seed ∪ greedy extension by occurrence-weighted rank until
    # ≥ TARGET of corpus occurrences are strings made ONLY of T1 constructs.
    TARGET = 0.995
    t1 = set(c for c in T1_SEED if c in construct_weight)

    def covered_mass(tier: set[str]) -> int:
        return sum(
            count
            for (mode, text), count in h.corpus.items()
            if string_constructs[(mode, text)] <= tier
        )

    seed_only_mass = covered_mass(t1)
    ranked = sorted(
        construct_weight, key=lambda c: (-construct_weight[c], c)
    )
    for c in ranked:
        if covered_mass(t1) / total_occ >= TARGET:
            break
        if c not in t1:
            t1.add(c)
    final_mass = covered_mass(t1)

    # ---- private corpus (gitignored) ----------------------------------
    PRIVATE_OUT.parent.mkdir(parents=True, exist_ok=True)
    with PRIVATE_OUT.open("w", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, ensure_ascii=False, sort_keys=True) + "\n")

    # ---- committed artifacts ------------------------------------------
    PUBLIC_DIR.mkdir(parents=True, exist_ok=True)

    denom_lines = [
        f"{rec['sha256']}\t{rec['mode']}\t{rec['count']}" for rec in records
    ]
    (PUBLIC_DIR / "denominator.tsv").write_text(
        "# sha256(mode + NUL + string)\tmode\toccurrences  — G0-4 ratchet "
        "denominator (fm-or4); strings are private fixtures per §15.3\n"
        + "\n".join(denom_lines)
        + "\n",
        encoding="utf-8",
    )
    corpus_hash = hashlib.sha256(
        "\n".join(denom_lines).encode()
    ).hexdigest()

    rows = []
    for rank, c in enumerate(ranked, 1):
        rows.append(
            "\t".join(
                [
                    str(rank),
                    c,
                    kind_of(c),
                    str(construct_weight[c]),
                    str(construct_uses[c]),
                    str(construct_distinct[c]),
                    f"{100.0 * construct_weight[c] / total_occ:.3f}",
                    "T1" if c in t1 else "T2",
                    "seed" if c in T1_SEED else "rank",
                ]
            )
        )
    (PUBLIC_DIR / "construct_table.tsv").write_text(
        "# G0-4 ranked construct table (fm-or4).  weight = corpus occurrences "
        "whose string contains the construct;\n"
        "# uses = occurrence-weighted total uses; strings = distinct strings "
        "containing it; pct = weight/total_occurrences.\n"
        "# tier: T1 = fmd-math tier 1 (§11.4 seed ∪ rank extension to ≥99.5% "
        "string-mass coverage); origin: seed|rank.\n"
        "rank\tconstruct\tkind\tweight\tuses\tstrings\tpct\ttier\torigin\n"
        + "\n".join(rows)
        + "\n",
        encoding="utf-8",
    )

    per_call = {k: h.call_sites[k] for k in sorted(h.call_sites)}
    manifest = {
        "bead": "fm-or4",
        "rules_version": RULES_VERSION,
        "pins": pins,
        "trees": ["scripts/videos_ref (all *.py)",
                  "scripts/manim_ref/manimlib (all *.py)"],
        "call_names": {"math": sorted(MATH_CALLS), "text": sorted(TEXT_CALLS)},
        "files_scanned": h.files_scanned,
        "files_failed_to_parse": sorted(h.files_failed),
        "call_sites": per_call,
        "dynamic_args_excluded": {
            k: h.dynamic_args[k] for k in sorted(h.dynamic_args)
        },
        "occurrences_by_dir": {k: h.per_dir[k] for k in sorted(h.per_dir)},
        "dynamic_fragments_advisory": len(h.fragments),
        "denominator": {
            "distinct_strings": distinct,
            "total_occurrences": total_occ,
            "corpus_sha256": corpus_hash,
        },
        "constructs": {
            "distinct": len(construct_weight),
            "t1_count": len(t1),
            "t2_count": len(construct_weight) - len(t1),
            "t1_seed_only_coverage": round(seed_only_mass / total_occ, 6),
            "t1_final_coverage": round(final_mass / total_occ, 6),
            "coverage_target": TARGET,
        },
        "advisory_dynamic_constructs": {
            c: advisory[c] for c in sorted(advisory)
        },
    }
    (PUBLIC_DIR / "harvest_manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )

    print(f"files scanned      {h.files_scanned}")
    print(f"call sites         {dict(per_call)}")
    print(f"dynamic (excluded) {sum(h.dynamic_args.values())}")
    print(f"denominator        {distinct} distinct / {total_occ} occurrences")
    print(f"corpus sha256      {corpus_hash}")
    print(f"constructs         {len(construct_weight)} distinct")
    print(f"T1 seed-only coverage  {seed_only_mass / total_occ:.4%}")
    print(f"T1 final coverage      {final_mass / total_occ:.4%} "
          f"({len(t1)} constructs)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
