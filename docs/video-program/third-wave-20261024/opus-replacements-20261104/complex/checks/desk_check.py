"""Desk check, not a compile: imported names never used again in that file,
and items whose only mention outside test modules is their own definition.
Run from the crate. Trait methods are skipped by name."""
import glob
import re

files = glob.glob("src/**/*.rs", recursive=True)
text = {f: open(f, encoding="utf8").read() for f in files}
bad = 0
for f, t in sorted(text.items()):
    if f.endswith(("thumbnail.rs", "score.rs")):
        continue
    for m in re.finditer(r"^\s*use\s+([^;]+);", t, re.M):
        body = m.group(1)
        if "{" in body:
            inner = body[body.index("{") + 1 : body.rindex("}")]
            names = [n.strip().split(" as ")[-1].split("::")[-1] for n in inner.split(",") if n.strip()]
        else:
            names = [body.split("::")[-1].split(" as ")[-1].strip()]
        rest = t[: m.start()] + t[m.end() :]
        for n in names:
            if n in ("*", "self", "super") or not n or "}" in n or "{" in n:
                continue
            if not re.search(r"\b" + re.escape(n) + r"\b", rest):
                print("UNUSED", f, n)
                bad += 1

prod = {f: t.split("#[cfg(test)]\nmod tests")[0] for f, t in text.items()}
allp = "\n".join(prod.values())
skip = {"main", "name", "construct", "new", "mul", "div", "neg", "add", "sub", "STORYBOARD"}
item = re.compile(r"^[ \t]*(?:pub(?:\([a-z]+\))? )?(?:const fn|const|fn|static) ([A-Za-z_0-9]+)", re.M)
for f, t in sorted(prod.items()):
    if f.endswith(("thumbnail.rs", "score.rs")):
        continue
    for m in item.finditer(t):
        n = m.group(1)
        if n in skip:
            continue
        if len(re.findall(r"\b" + re.escape(n) + r"\b", allp)) < 2:
            print("DEAD", f, n)
            bad += 1
print("desk problems", bad)
