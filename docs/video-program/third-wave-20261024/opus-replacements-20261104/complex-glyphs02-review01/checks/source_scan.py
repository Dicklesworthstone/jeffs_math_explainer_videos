"""Static scans of the registry: what to look for on the pages. Reads only.

Run from the repo root with python3 -I -B. Nothing here decides a defect; each hit is a row
to read on its page.
"""
import collections
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(sys.argv[0]).resolve().parent))
RUN = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02")


def decode(literal):
    body = literal[1:-1]
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
            out.append(chr(int(body[i + 3 : end], 16)))
            i = end + 1
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


rows = []
for number, line in enumerate((RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines(), 1):
    kind, chapters, literal = line.split("\t", 2)
    rows.append((number, kind, chapters, decode(literal)))


def show(title, hits, limit=80):
    print(f"\n# {title}: {len(hits)}")
    for n, kind, chapters, text in hits[:limit]:
        print(f"  {n:4d} {kind:4s} {chapters[:34]:34s} {text!r}")


print("# composition")
print(" ", collections.Counter(k for _, k, _, _ in rows))
only_sup = [r for r in rows if r[2] == "supplement"]
no_sup = [r for r in rows if "supplement" not in r[2]]
both = [r for r in rows if "supplement" in r[2] and r[2] != "supplement"]
print(f"  captured from a chapter and in the supplement {len(both)}; captured only {len(no_sup)}; supplement only {len(only_sup)}")

print("\n# non-ASCII characters, by kind")
for kind in ("Text", "Tex", "Mono"):
    count = collections.Counter(c for _, k, _, t in rows if k == kind for c in t if ord(c) > 127)
    print(f"  {kind}: " + " ".join(f"{c}(U+{ord(c):04X})x{n}" for c, n in sorted(count.items())))

show("Text or Mono rows holding TeX-looking markup (^ _ \\ $ { })", [r for r in rows if r[1] != "Tex" and re.search(r"[\^_\\${}]", r[3])])
show("Text rows with ASCII hyphen beside a digit or as a sign", [r for r in rows if r[1] == "Text" and re.search(r"(^|[\s(=+])-[\d.a-zA-Z(]|\d\s?-\s?\d", r[3])], 60)
show("Text rows with an ASCII apostrophe beside a letter used as a prime or an asterisk", [r for r in rows if r[1] == "Text" and re.search(r"\bf'|\w\*\w|\*", r[3])])
show("Text rows with ascii x between numbers", [r for r in rows if r[1] == "Text" and re.search(r"\d\s*x\s*\d", r[3])])
show("Number artefacts: exponents, NaN, inf, negative zero, doubled signs, long decimals", [r for r in rows if re.search(r"\d[eE][-+]?\d|NaN|\binf\b|[-−]0(\.0+)?(?![\d.])|\+\s*[-−]|[-−]\s*[-−]|\d\.\d{6,}", r[3])], 60)

# TeX: control sequences, bare words and group balance.
tex = [r for r in rows if r[1] == "Tex"]
commands = collections.Counter(c for _, _, _, t in tex for c in re.findall(r"\\([A-Za-z]+|.)", t))
print("\n# TeX control sequences used")
print("  " + " ".join(f"\\{c}x{n}" for c, n in sorted(commands.items())))


def strip_text(t):
    t = re.sub(r"\\(?:text|mathrm|operatorname|mathbf|mathbb|mathcal|textit|textbf)\s*\{[^{}]*\}", " ", t)
    return re.sub(r"\\[A-Za-z]+", " ", t)


bare = [r for r in tex if re.search(r"[A-Za-z]{3,}", strip_text(r[3]))]
show("TeX rows with a bare run of 3+ letters outside text commands (would set as italic product)", bare)
show("TeX rows with non-ASCII source", [r for r in tex if any(ord(c) > 127 for c in r[3])])
spaces = [r for r in tex if re.search(r"[A-Za-z]{2,} [A-Za-z]{2,}", strip_text(r[3]))]
show("TeX rows with words separated by a plain space (spaces vanish in math)", spaces)

print("\n# longest rows")
for n, kind, chapters, text in sorted(rows, key=lambda r: -len(r[3]))[:12]:
    print(f"  {n:4d} {kind:4s} len {len(text):4d} {text[:90]!r}")
